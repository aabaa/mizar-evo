use mizar_build::{
    scheduler::{SchedulerEventKind, SchedulerOrderKey, TaskState},
    task_graph::PipelinePhase,
};
use mizar_driver::{
    events::{
        BuildEvent, BuildEventError, BuildEventIdentity, BuildEventIdentityKey, BuildEventKind,
        BuildEventLog, BuildEventOrderKey, BuildEventStream, EventOwner, OwnerGapClassification,
        OwnerRecordRef, TaskEventRef,
    },
    registry::{PhaseServiceAvailability, PhaseStatus},
    request::{
        BuildLaneId, BuildRequestGeneration, BuildSessionOutcome, ObsoletePublication,
        PublicationDecision,
    },
};
use mizar_session::{
    BuildSessionId, BuildSnapshotId, Hash, InMemorySessionIdAllocator, SessionIdAllocator,
};

#[test]
fn shuffled_task_completion_events_sort_to_identical_sequences() {
    let session = session_id();
    let base_identity = identity(session, PublicationDecision::Current);
    let source = task_event(
        &base_identity,
        "task-source",
        SchedulerOrderKey {
            graph_index: 1,
            lifecycle_rank: 4,
        },
        PipelinePhase::SourceLoad,
    );
    let frontend = task_event(
        &base_identity,
        "task-frontend",
        SchedulerOrderKey {
            graph_index: 2,
            lifecycle_rank: 4,
        },
        PipelinePhase::Frontend,
    );
    let finish = BuildEvent::new(
        base_identity.clone(),
        BuildEventOrderKey::new(90, 90),
        BuildEventKind::SessionFinished {
            outcome: BuildSessionOutcome::Succeeded,
        },
    );

    let canonical = BuildEventStream::from_events(
        session,
        true,
        vec![source.clone(), frontend.clone(), finish.clone()],
    )
    .unwrap();
    let shuffled =
        BuildEventStream::from_events(session, true, vec![finish, frontend, source]).unwrap();

    assert_eq!(canonical.events(), shuffled.events());
    assert!(
        canonical
            .events()
            .windows(2)
            .all(|window| { window[0].order <= window[1].order })
    );
}

#[test]
fn events_must_reference_the_stream_session_and_snapshot() {
    let (session, other) = two_session_ids();
    let stream = BuildEventStream::from_events(
        session,
        true,
        vec![BuildEvent::new(
            identity(session, PublicationDecision::Current),
            BuildEventOrderKey::new(0, 0),
            BuildEventKind::SnapshotCaptured,
        )],
    )
    .unwrap();

    assert!(stream.known_session);
    assert!(stream.events().iter().all(|event| {
        event.identity.session == stream.session && event.identity.snapshot == Some(snapshot(1))
    }));

    let error = BuildEventStream::from_events(
        session,
        true,
        vec![BuildEvent::new(
            identity(other, PublicationDecision::Current),
            BuildEventOrderKey::new(0, 0),
            BuildEventKind::SnapshotCaptured,
        )],
    )
    .unwrap_err();
    assert_eq!(
        error,
        BuildEventError::SessionMismatch {
            expected: session,
            actual: other,
        }
    );

    let unknown_error = BuildEventStream::from_events(
        session,
        false,
        vec![BuildEvent::new(
            identity(session, PublicationDecision::Current),
            BuildEventOrderKey::new(0, 0),
            BuildEventKind::SnapshotCaptured,
        )],
    )
    .unwrap_err();
    assert_eq!(unknown_error, BuildEventError::UnknownSession { session });

    let missing_snapshot = BuildEventStream::from_events(
        session,
        true,
        vec![BuildEvent::new(
            identity_without_snapshot(session, PublicationDecision::Current),
            BuildEventOrderKey::new(0, 0),
            BuildEventKind::SnapshotCaptured,
        )],
    )
    .unwrap_err();
    assert_eq!(
        missing_snapshot,
        BuildEventError::MissingSnapshot {
            kind: "SnapshotCaptured".to_owned()
        }
    );

    let mixed_snapshot = BuildEventStream::from_events(
        session,
        true,
        vec![
            BuildEvent::new(
                identity(session, PublicationDecision::Current),
                BuildEventOrderKey::new(0, 0),
                BuildEventKind::SnapshotCaptured,
            ),
            BuildEvent::new(
                identity_with_snapshot(session, PublicationDecision::Current, snapshot(2)),
                BuildEventOrderKey::new(1, 0),
                BuildEventKind::PlanningReady {
                    status: mizar_driver::events::PlanningEventStatus::Ready,
                },
            ),
        ],
    )
    .unwrap_err();
    assert_eq!(
        mixed_snapshot,
        BuildEventError::IdentityMismatch {
            expected: BuildEventIdentityKey {
                lane: BuildLaneId::new(1),
                generation: BuildRequestGeneration::new(0),
                snapshot: Some(snapshot(1)),
            },
            actual: BuildEventIdentityKey {
                lane: BuildLaneId::new(1),
                generation: BuildRequestGeneration::new(0),
                snapshot: Some(snapshot(2)),
            },
        }
    );

    let mixed_lane = BuildEventStream::from_events(
        session,
        true,
        vec![
            BuildEvent::new(
                identity(session, PublicationDecision::Current),
                BuildEventOrderKey::new(0, 0),
                BuildEventKind::SnapshotCaptured,
            ),
            BuildEvent::new(
                identity_with_lane_generation(
                    session,
                    PublicationDecision::Current,
                    BuildLaneId::new(2),
                    BuildRequestGeneration::new(0),
                    snapshot(1),
                ),
                BuildEventOrderKey::new(1, 0),
                BuildEventKind::PlanningReady {
                    status: mizar_driver::events::PlanningEventStatus::Ready,
                },
            ),
        ],
    )
    .unwrap_err();
    assert_eq!(
        mixed_lane,
        BuildEventError::IdentityMismatch {
            expected: BuildEventIdentityKey {
                lane: BuildLaneId::new(1),
                generation: BuildRequestGeneration::new(0),
                snapshot: Some(snapshot(1)),
            },
            actual: BuildEventIdentityKey {
                lane: BuildLaneId::new(2),
                generation: BuildRequestGeneration::new(0),
                snapshot: Some(snapshot(1)),
            },
        }
    );

    let mixed_generation = BuildEventStream::from_events(
        session,
        true,
        vec![
            BuildEvent::new(
                identity(session, PublicationDecision::Current),
                BuildEventOrderKey::new(0, 0),
                BuildEventKind::SnapshotCaptured,
            ),
            BuildEvent::new(
                identity_with_lane_generation(
                    session,
                    PublicationDecision::Current,
                    BuildLaneId::new(1),
                    BuildRequestGeneration::new(1),
                    snapshot(1),
                ),
                BuildEventOrderKey::new(1, 0),
                BuildEventKind::PlanningReady {
                    status: mizar_driver::events::PlanningEventStatus::Ready,
                },
            ),
        ],
    )
    .unwrap_err();
    assert_eq!(
        mixed_generation,
        BuildEventError::IdentityMismatch {
            expected: BuildEventIdentityKey {
                lane: BuildLaneId::new(1),
                generation: BuildRequestGeneration::new(0),
                snapshot: Some(snapshot(1)),
            },
            actual: BuildEventIdentityKey {
                lane: BuildLaneId::new(1),
                generation: BuildRequestGeneration::new(1),
                snapshot: Some(snapshot(1)),
            },
        }
    );
}

#[test]
fn session_accepted_may_precede_snapshot_scoped_identity() {
    let session = session_id();
    let stream = BuildEventStream::from_events(
        session,
        true,
        vec![
            BuildEvent::new(
                identity_without_snapshot(session, PublicationDecision::Current),
                BuildEventOrderKey::new(0, 0),
                BuildEventKind::SessionAccepted,
            ),
            BuildEvent::new(
                identity(session, PublicationDecision::Current),
                BuildEventOrderKey::new(10, 0),
                BuildEventKind::SnapshotCaptured,
            ),
            BuildEvent::new(
                identity(session, PublicationDecision::Current),
                BuildEventOrderKey::new(20, 0),
                BuildEventKind::PlanningReady {
                    status: mizar_driver::events::PlanningEventStatus::Ready,
                },
            ),
        ],
    )
    .unwrap();

    assert_eq!(stream.events()[0].kind, BuildEventKind::SessionAccepted);
    assert_eq!(stream.events()[0].identity.snapshot, None);
    assert!(stream.events()[1..].iter().all(|event| {
        event.identity.snapshot == Some(snapshot(1))
            && event.identity.lane == BuildLaneId::new(1)
            && event.identity.generation == BuildRequestGeneration::new(0)
    }));
}

#[test]
fn stale_publication_uses_suppression_event_not_current_payload() {
    let session = session_id();
    let suppressed = PublicationDecision::Suppressed(ObsoletePublication {
        lane_current: false,
        request_snapshot_current: true,
    });
    let stream = BuildEventStream::from_events(
        session,
        true,
        vec![BuildEvent::new(
            identity(session, suppressed),
            BuildEventOrderKey::new(80, 1),
            BuildEventKind::PublicationSuppressed,
        )],
    )
    .unwrap();

    let event = &stream.events()[0];
    assert_eq!(event.kind, BuildEventKind::PublicationSuppressed);
    assert!(matches!(
        event.identity.publication,
        PublicationDecision::Suppressed(ObsoletePublication {
            lane_current: false,
            request_snapshot_current: true,
        })
    ));

    let stale_diagnostics = BuildEventStream::from_events(
        session,
        true,
        vec![BuildEvent::new(
            identity(session, suppressed),
            BuildEventOrderKey::new(81, 1),
            BuildEventKind::DiagnosticsReady {
                records: OwnerRecordRef::new(EventOwner::Diagnostics, "owner-record-ref"),
            },
        )],
    )
    .unwrap_err();
    assert_eq!(
        stale_diagnostics,
        BuildEventError::SuppressedCurrentPayload {
            kind: "DiagnosticsReady".to_owned()
        }
    );

    let stale_artifact = BuildEventStream::from_events(
        session,
        true,
        vec![BuildEvent::new(
            identity(session, suppressed),
            BuildEventOrderKey::new(82, 1),
            BuildEventKind::ArtifactBoundary {
                committed: OwnerRecordRef::new(EventOwner::Artifact, "artifact-owner-ref"),
            },
        )],
    )
    .unwrap_err();
    assert_eq!(
        stale_artifact,
        BuildEventError::SuppressedCurrentPayload {
            kind: "ArtifactBoundary".to_owned()
        }
    );

    let invalid_suppression = BuildEventStream::from_events(
        session,
        true,
        vec![BuildEvent::new(
            identity(session, PublicationDecision::Current),
            BuildEventOrderKey::new(83, 1),
            BuildEventKind::PublicationSuppressed,
        )],
    )
    .unwrap_err();
    assert_eq!(
        invalid_suppression,
        BuildEventError::InvalidPublicationSuppression {
            kind: "PublicationSuppressed".to_owned()
        }
    );

    let contradictory_suppression = PublicationDecision::Suppressed(ObsoletePublication {
        lane_current: true,
        request_snapshot_current: true,
    });
    let contradictory_error = BuildEventStream::from_events(
        session,
        true,
        vec![BuildEvent::new(
            identity(session, contradictory_suppression),
            BuildEventOrderKey::new(84, 1),
            BuildEventKind::PublicationSuppressed,
        )],
    )
    .unwrap_err();
    assert_eq!(
        contradictory_error,
        BuildEventError::InvalidPublicationSuppression {
            kind: "PublicationSuppressed".to_owned()
        }
    );
}

#[test]
fn failed_push_does_not_poison_event_log_identity() {
    let session = session_id();
    let mut log = BuildEventLog::new(session, true);
    let contradictory_suppression = PublicationDecision::Suppressed(ObsoletePublication {
        lane_current: true,
        request_snapshot_current: true,
    });

    let first_error = log.push(BuildEvent::new(
        identity_with_lane_generation(
            session,
            contradictory_suppression,
            BuildLaneId::new(1),
            BuildRequestGeneration::new(0),
            snapshot(1),
        ),
        BuildEventOrderKey::new(0, 0),
        BuildEventKind::PublicationSuppressed,
    ));
    assert_eq!(
        first_error,
        Err(BuildEventError::InvalidPublicationSuppression {
            kind: "PublicationSuppressed".to_owned()
        })
    );

    log.push(BuildEvent::new(
        identity_with_lane_generation(
            session,
            PublicationDecision::Current,
            BuildLaneId::new(2),
            BuildRequestGeneration::new(1),
            snapshot(2),
        ),
        BuildEventOrderKey::new(10, 0),
        BuildEventKind::SnapshotCaptured,
    ))
    .unwrap();
    let stream = log.into_stream();

    assert_eq!(stream.events().len(), 1);
    assert_eq!(stream.events()[0].identity.lane, BuildLaneId::new(2));
    assert_eq!(
        stream.events()[0].identity.generation,
        BuildRequestGeneration::new(1)
    );
    assert_eq!(stream.events()[0].identity.snapshot, Some(snapshot(2)));
}

#[test]
fn gap_events_cover_dispatch_phase_service_and_owner_readiness_without_fake_payloads() {
    let session = session_id();
    let base_identity = identity(session, PublicationDecision::Current);
    let stream = BuildEventStream::from_events(
        session,
        true,
        vec![
            BuildEvent::new(
                base_identity.clone(),
                BuildEventOrderKey::new(30, 1).with_phase(PipelinePhase::Frontend),
                BuildEventKind::PhaseServiceGap {
                    phase: PipelinePhase::Frontend,
                    availability: PhaseServiceAvailability::ExternalDependencyGap,
                },
            ),
            BuildEvent::new(
                base_identity.clone(),
                BuildEventOrderKey::new(31, 1).with_phase(PipelinePhase::SourceLoad),
                BuildEventKind::DispatchGap {
                    phases: vec![PipelinePhase::SourceLoad, PipelinePhase::Frontend],
                },
            ),
            BuildEvent::new(
                base_identity,
                BuildEventOrderKey::new(32, 1),
                BuildEventKind::OwnerReadinessGap {
                    owner: EventOwner::Artifact,
                    classification: OwnerGapClassification::ExternalDependencyGap,
                },
            ),
            BuildEvent::new(
                identity(session, PublicationDecision::Current),
                BuildEventOrderKey::new(33, 1),
                BuildEventKind::OwnerReadinessGap {
                    owner: EventOwner::Diagnostics,
                    classification: OwnerGapClassification::ExternalDependencyGap,
                },
            ),
            BuildEvent::new(
                identity(session, PublicationDecision::Current),
                BuildEventOrderKey::new(34, 1),
                BuildEventKind::OwnerReadinessGap {
                    owner: EventOwner::ProducerOutput,
                    classification: OwnerGapClassification::Deferred,
                },
            ),
            BuildEvent::new(
                identity(session, PublicationDecision::Current),
                BuildEventOrderKey::new(35, 1),
                BuildEventKind::OwnerReadinessGap {
                    owner: EventOwner::LspBridge,
                    classification: OwnerGapClassification::Unavailable,
                },
            ),
        ],
    )
    .unwrap();

    assert!(stream.events().iter().any(|event| matches!(
        event.kind,
        BuildEventKind::PhaseServiceGap {
            phase: PipelinePhase::Frontend,
            availability: PhaseServiceAvailability::ExternalDependencyGap,
        }
    )));
    assert!(
        stream
            .events()
            .iter()
            .any(|event| matches!(event.kind, BuildEventKind::DispatchGap { .. }))
    );
    assert!(stream.events().iter().any(|event| matches!(
        event.kind,
        BuildEventKind::OwnerReadinessGap {
            owner: EventOwner::Artifact,
            classification: OwnerGapClassification::ExternalDependencyGap,
        }
    )));
    for owner in [
        EventOwner::Diagnostics,
        EventOwner::Artifact,
        EventOwner::ProducerOutput,
        EventOwner::LspBridge,
    ] {
        assert!(stream.events().iter().any(|event| matches!(
            event.kind,
            BuildEventKind::OwnerReadinessGap { owner: seen, .. } if seen == owner
        )));
    }
}

#[test]
fn completed_task_progress_does_not_become_artifact_boundary() {
    let session = session_id();
    let stream = BuildEventStream::from_events(
        session,
        true,
        vec![task_event(
            &identity(session, PublicationDecision::Current),
            "artifact-task",
            SchedulerOrderKey {
                graph_index: 9,
                lifecycle_rank: 4,
            },
            PipelinePhase::ArtifactCommit,
        )],
    )
    .unwrap();

    assert!(matches!(
        stream.events()[0].kind,
        BuildEventKind::TaskProgress { .. }
    ));
    assert!(
        !stream
            .events()
            .iter()
            .any(|event| matches!(event.kind, BuildEventKind::ArtifactBoundary { .. }))
    );
}

#[test]
fn phase_ready_accepts_cancelled_status_without_claiming_session_success() {
    let session = session_id();
    let event = BuildEvent::new(
        identity(session, PublicationDecision::Current),
        BuildEventOrderKey::new(40, 1).with_phase(PipelinePhase::Frontend),
        BuildEventKind::PhaseReady {
            phase: PipelinePhase::Frontend,
            status: PhaseStatus::Cancelled,
        },
    );
    let stream = BuildEventStream::from_events(session, true, vec![event]).unwrap();

    assert!(matches!(
        stream.events()[0].kind,
        BuildEventKind::PhaseReady {
            phase: PipelinePhase::Frontend,
            status: PhaseStatus::Cancelled,
        }
    ));
}

#[test]
fn owner_ready_events_use_owner_refs_without_diagnostic_or_lsp_authority() {
    let session = session_id();
    let base_identity = identity(session, PublicationDecision::Current);
    let stream = BuildEventStream::from_events(
        session,
        true,
        vec![BuildEvent::new(
            base_identity,
            BuildEventOrderKey::new(50, 1).with_owner_order("diagnostics-owner-order-1"),
            BuildEventKind::DiagnosticsReady {
                records: OwnerRecordRef::new(EventOwner::Diagnostics, "owner-record-ref"),
            },
        )],
    )
    .unwrap();

    assert!(matches!(
        &stream.events()[0].kind,
        BuildEventKind::DiagnosticsReady { records }
            if records.owner == EventOwner::Diagnostics && records.identity == "owner-record-ref"
    ));

    let wrong_diagnostics_owner = BuildEventStream::from_events(
        session,
        true,
        vec![BuildEvent::new(
            identity(session, PublicationDecision::Current),
            BuildEventOrderKey::new(51, 1),
            BuildEventKind::DiagnosticsReady {
                records: OwnerRecordRef::new(EventOwner::Artifact, "wrong-owner"),
            },
        )],
    )
    .unwrap_err();
    assert_eq!(
        wrong_diagnostics_owner,
        BuildEventError::OwnerMismatch {
            kind: "DiagnosticsReady".to_owned(),
            expected: EventOwner::Diagnostics,
            actual: EventOwner::Artifact,
        }
    );

    let wrong_artifact_owner = BuildEventStream::from_events(
        session,
        true,
        vec![BuildEvent::new(
            identity(session, PublicationDecision::Current),
            BuildEventOrderKey::new(52, 1),
            BuildEventKind::ArtifactBoundary {
                committed: OwnerRecordRef::new(EventOwner::Diagnostics, "wrong-owner"),
            },
        )],
    )
    .unwrap_err();
    assert_eq!(
        wrong_artifact_owner,
        BuildEventError::OwnerMismatch {
            kind: "ArtifactBoundary".to_owned(),
            expected: EventOwner::Artifact,
            actual: EventOwner::Diagnostics,
        }
    );
}

#[test]
fn event_replay_preserves_order_and_publication_decisions() {
    let session = session_id();
    let stream = BuildEventStream::from_events(
        session,
        true,
        vec![
            BuildEvent::new(
                identity(session, PublicationDecision::Current),
                BuildEventOrderKey::new(0, 0),
                BuildEventKind::SessionAccepted,
            ),
            BuildEvent::new(
                identity(session, PublicationDecision::Current),
                BuildEventOrderKey::new(10, 0),
                BuildEventKind::PlanningReady {
                    status: mizar_driver::events::PlanningEventStatus::Ready,
                },
            ),
        ],
    )
    .unwrap();

    assert_eq!(stream.replay(), stream);
}

#[test]
fn event_source_does_not_claim_diagnostics_artifact_scheduler_or_lsp_authority() {
    let source = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/events.rs"),
    )
    .unwrap();

    for forbidden in [
        "DiagnosticId",
        "DiagnosticCode",
        "DiagnosticRecord",
        "DiagnosticAggregator",
        "dedup",
        "render_cli",
        "JsonRpc",
        "LspDiagnostic",
        "DiagnosticSeverity",
        "CodeAction",
        "DocumentUri",
        "DocumentVersion",
        "ProgressToken",
        "PublicationToken",
        "SyntheticOutputRef",
        "SchedulerResult",
        ".output_refs",
        "AnyPhaseOutputRef",
        "PhaseOutputPublisher",
        "VerifiedArtifact",
    ] {
        assert!(
            !source.contains(forbidden),
            "event source must not own forbidden authority term {forbidden}"
        );
    }
}

fn task_event(
    identity: &BuildEventIdentity,
    task_id: &str,
    scheduler_order: SchedulerOrderKey,
    phase: PipelinePhase,
) -> BuildEvent {
    BuildEvent::new(
        identity.clone(),
        BuildEventOrderKey::new(20, 1)
            .with_scheduler_order(scheduler_order)
            .with_phase(phase)
            .with_work_unit(task_id),
        BuildEventKind::TaskProgress {
            task: TaskEventRef::new(
                task_id,
                SchedulerEventKind::TaskFinished,
                Some(TaskState::Completed),
            ),
        },
    )
}

fn identity(session: BuildSessionId, publication: PublicationDecision) -> BuildEventIdentity {
    identity_with_snapshot(session, publication, snapshot(1))
}

fn identity_with_snapshot(
    session: BuildSessionId,
    publication: PublicationDecision,
    snapshot: BuildSnapshotId,
) -> BuildEventIdentity {
    identity_with_lane_generation(
        session,
        publication,
        BuildLaneId::new(1),
        BuildRequestGeneration::new(0),
        snapshot,
    )
}

fn identity_with_lane_generation(
    session: BuildSessionId,
    publication: PublicationDecision,
    lane: BuildLaneId,
    generation: BuildRequestGeneration,
    snapshot: BuildSnapshotId,
) -> BuildEventIdentity {
    BuildEventIdentity {
        session,
        lane,
        generation,
        snapshot: Some(snapshot),
        publication,
    }
}

fn identity_without_snapshot(
    session: BuildSessionId,
    publication: PublicationDecision,
) -> BuildEventIdentity {
    BuildEventIdentity {
        session,
        lane: BuildLaneId::new(1),
        generation: BuildRequestGeneration::new(0),
        snapshot: None,
        publication,
    }
}

fn session_id() -> BuildSessionId {
    InMemorySessionIdAllocator::new()
        .next_session_id()
        .expect("fixture session id allocates")
}

fn two_session_ids() -> (BuildSessionId, BuildSessionId) {
    let ids = InMemorySessionIdAllocator::new();
    (
        ids.next_session_id()
            .expect("first fixture session id allocates"),
        ids.next_session_id()
            .expect("second fixture session id allocates"),
    )
}

fn snapshot(seed: u8) -> BuildSnapshotId {
    let serialized = format!(
        "mizar-session-build-snapshot-v1:{}",
        format!("{seed:02x}").repeat(Hash::BYTE_LEN)
    );
    BuildSnapshotId::from_published_schema_str(&serialized).unwrap()
}
