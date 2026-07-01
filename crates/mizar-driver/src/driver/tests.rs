use mizar_build::cancel::CancellationPolicy;
use mizar_session::{
    DependencyArtifactRef, Hash, InMemorySessionIdAllocator, SnapshotRegistry, ToolchainInfo,
    WorkspaceRoot,
};

use super::*;
use crate::{
    events::BuildEventKind,
    request::{
        BatchInvocation, BatchRequest, BuildLaneId, BuildProfile, BuildRequestGeneration,
        BuildRequestOrigin, BuildTargets, DependencyInputSet, SourceInputSet, VerifierConfigInput,
    },
};

#[test]
fn cancel_active_session_updates_build_policy_once() {
    let ids = InMemorySessionIdAllocator::new();
    let snapshots = SnapshotRegistry::new();
    let lane = BuildLaneId::new(9);
    let session = request_with_lane_generation(9, lane, BuildRequestGeneration::new(0))
        .allocate(&ids)
        .unwrap()
        .capture_snapshot(&snapshots)
        .unwrap();
    let newer = request_with_lane_generation(12, lane, BuildRequestGeneration::new(1))
        .allocate(&ids)
        .unwrap()
        .capture_snapshot(&snapshots)
        .unwrap();
    let session_id = session.id;
    let snapshot_id = session.captured.snapshot.id;
    let mut driver = CompilerDriver::default();
    assert!(driver.lanes.mark_current(&session));
    driver.store_session(
        session,
        CancellationPolicy::default(),
        BuildEventStream::empty(session_id, true),
    );
    assert!(driver.lanes.mark_current(&newer));

    let first = driver.cancel(session_id, DriverCancelReason::Superseded, &snapshots);
    let second = driver.cancel(session_id, DriverCancelReason::Superseded, &snapshots);

    assert!(first.changed);
    assert_eq!(
        first.state,
        Some(BuildSessionState::Finished(BuildSessionOutcome::Superseded))
    );
    assert!(!second.changed);
    assert_eq!(
        second.state,
        Some(BuildSessionState::Finished(BuildSessionOutcome::Superseded))
    );
    assert_eq!(
        driver
            .cancellation_policy(session_id)
            .unwrap()
            .superseded_snapshots(),
        &[snapshot_id]
    );
    let stream = driver.events(session_id);
    assert_eq!(
        stream
            .events()
            .iter()
            .filter(|event| matches!(event.kind, BuildEventKind::PublicationSuppressed))
            .count(),
        1
    );
    let terminal_events: Vec<_> = stream
        .events()
        .iter()
        .filter(|event| {
            matches!(
                event.kind,
                BuildEventKind::SessionFinished {
                    outcome: BuildSessionOutcome::Superseded
                }
            )
        })
        .collect();
    assert_eq!(terminal_events.len(), 1);
    match terminal_events[0].identity.publication {
        PublicationDecision::Suppressed(obsolete) => {
            assert!(!obsolete.lane_current);
            assert!(obsolete.request_snapshot_current);
        }
        PublicationDecision::Current => panic!("superseded cancel must not publish current"),
    }
}

#[test]
fn explicit_and_shutdown_cancel_do_not_claim_snapshot_supersession() {
    for (seed, reason) in [
        (10, DriverCancelReason::ExplicitRequest),
        (11, DriverCancelReason::Shutdown),
    ] {
        let ids = InMemorySessionIdAllocator::new();
        let snapshots = SnapshotRegistry::new();
        let session = request(seed)
            .allocate(&ids)
            .unwrap()
            .capture_snapshot(&snapshots)
            .unwrap();
        let session_id = session.id;
        let mut driver = CompilerDriver::default();
        assert!(driver.lanes.mark_current(&session));
        driver.store_session(
            session,
            CancellationPolicy::default(),
            BuildEventStream::empty(session_id, true),
        );

        let outcome = driver.cancel(session_id, reason, &snapshots);

        assert!(outcome.changed);
        assert_eq!(
            outcome.state,
            Some(BuildSessionState::Finished(BuildSessionOutcome::Cancelled))
        );
        assert!(
            driver
                .cancellation_policy(session_id)
                .unwrap()
                .superseded_snapshots()
                .is_empty()
        );
        let stream = driver.events(session_id);
        assert!(stream.events().iter().any(|event| {
            matches!(
                event.kind,
                BuildEventKind::SessionFinished {
                    outcome: BuildSessionOutcome::Cancelled
                }
            )
        }));
        assert!(
            !stream
                .events()
                .iter()
                .any(|event| matches!(event.kind, BuildEventKind::PublicationSuppressed))
        );
    }
}

fn request(seed: u8) -> BuildRequestDraft {
    request_with_lane_generation(
        seed,
        BuildLaneId::new(u64::from(seed)),
        BuildRequestGeneration::new(0),
    )
}

fn request_with_lane_generation(
    seed: u8,
    lane: BuildLaneId,
    generation: BuildRequestGeneration,
) -> BuildRequestDraft {
    BuildRequestDraft {
        lane,
        origin: BuildRequestOrigin::Batch(BatchRequest {
            invocation: BatchInvocation::default(),
        }),
        generation,
        workspace_root: WorkspaceRoot::new("workspace"),
        profile: BuildProfile::new("check"),
        targets: BuildTargets::default(),
        source_inputs: SourceInputSet::default(),
        dependency_inputs: DependencyInputSet::new(
            vec![DependencyArtifactRef {
                artifact: format!("dep-{seed}"),
                content_hash: Hash::from_bytes([seed; Hash::BYTE_LEN]),
            }],
            Hash::from_bytes([seed.wrapping_add(1); Hash::BYTE_LEN]),
            ToolchainInfo::new("mizar-evo-test"),
        ),
        verifier_config: VerifierConfigInput::new(Hash::from_bytes(
            [seed.wrapping_add(2); Hash::BYTE_LEN],
        )),
    }
}
