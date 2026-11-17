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

#[cfg(test)]
mod imported_cache_lifecycle_tests {
    use std::sync::Arc;

    use super::*;
    use crate::{
        driver::{CompilerDriver, NativeCacheMode, SubmitOptions},
        frontend_adapter::ImportedFixture,
        registry::{PhaseStatus, SourceLoadInputs, StageConfig, assemble_registry},
    };
    use mizar_build::{
        cache_seam::CacheSchedulingOutcome,
        cancel::{CancellationReason, CancellationSignal},
        scheduler::{SchedulerDispatchTask, SchedulerTaskDispatcher},
        task_graph::{TaskKind, WorkUnit},
    };
    use mizar_cache::cache_store::CacheStoreRoot;
    use mizar_ir::{
        dispatch_input::{PhaseDispatchInputBundle, SealedParentOutputHandle},
        identity::{
            OutputKind, PipelinePhase as IrPhase, SnapshotHandleRegistry, WorkUnit as IrUnit,
        },
        publisher::{AllowedWorkUnit, PhaseOutputPublisher},
        storage::{CollectInput, IrStorageService, StoragePolicy},
    };
    use mizar_session::{InMemorySessionIdAllocator, SnapshotRegistry};

    struct CacheCase {
        _fixture: ImportedFixture,
        ids: InMemorySessionIdAllocator,
        policy: mizar_proof::policy::VerifierPolicy,
        store: CacheStoreRoot,
        publisher: Arc<PhaseOutputPublisher>,
        driver: CompilerDriver,
        submission: crate::driver::BuildSubmission,
    }

    impl CacheCase {
        fn new() -> Self {
            let fixture = ImportedFixture::new();
            let ids = InMemorySessionIdAllocator::new();
            let snapshots = SnapshotRegistry::new();
            let (session, input) = fixture.capture(&ids, &snapshots);
            let snapshot = session.captured().snapshot().id;
            let policy = mizar_proof::policy::VerifierPolicy::release();
            let store = CacheStoreRoot::new(fixture.root.join("alpha/.mizar-cache"));
            let mut config = StageConfig::frontend(vec![]);
            config.verifier_policy = policy.clone();
            let registry = assemble_registry(&config).unwrap();
            let publisher = Arc::new(PhaseOutputPublisher::new(
                Arc::new(IrStorageService::with_policy(StoragePolicy::default())),
                Arc::new(SnapshotHandleRegistry::new()),
            ));
            publisher.register_current_snapshot(snapshot);
            for module in ["main", "leaf", "twig"] {
                for (phase, kind) in [("SourceLoad", "SourceUnit"), ("Frontend", "FrontendOutput")]
                {
                    publisher.allow_work_unit(AllowedWorkUnit::new(
                        IrPhase::new(phase),
                        OutputKind::new(kind),
                        IrUnit::new(format!("{:?}:{:?}", "alpha", module)),
                    ));
                }
            }
            let mut driver = CompilerDriver::new(registry).with_output_publisher(publisher.clone());
            let submission = driver
                .submit_captured(
                    session,
                    &ids,
                    &snapshots,
                    input,
                    SubmitOptions {
                        native_cache: Some(NativeCacheOptions {
                            policy: &policy,
                            store: &store,
                            mode: NativeCacheMode::PersistentPrefix,
                        }),
                        ..SubmitOptions::default()
                    },
                )
                .unwrap();
            let run = submission.scheduler_run.as_ref().unwrap();
            for module in ["main", "leaf", "twig"] {
                for kind in [TaskKind::SourceLoad, TaskKind::Frontend] {
                    let task = submission
                        .task_graph
                        .as_ref()
                        .unwrap()
                        .tasks()
                        .iter()
                        .find(|task| {
                            task.kind == kind
                                && matches!(&task.unit, WorkUnit::Module { module: id } if id.path.as_str() == module)
                        })
                        .unwrap();
                    let [result] = run.phase_results[&task.id].as_slice() else {
                        panic!("cold run must produce one real result for {module} {kind:?}");
                    };
                    assert_eq!(result.status, PhaseStatus::Complete);
                    assert_eq!(result.output_refs.len(), 1);
                }
            }
            Self {
                _fixture: fixture,
                ids,
                policy,
                store,
                publisher,
                driver,
                submission,
            }
        }

        fn task(&self, kind: TaskKind, module: &str) -> &BuildTask {
            self.submission.task_graph.as_ref().unwrap().tasks().iter().find(|task| {
                task.kind == kind
                    && matches!(&task.unit, WorkUnit::Module { module: id } if id.path.as_str() == module)
            }).unwrap()
        }

        fn dispatcher(&self) -> RegistrySchedulerDispatcher<'_> {
            let captured = self.submission.session.captured();
            let mut dispatcher = RegistrySchedulerDispatcher::new(
                self.driver.registry(),
                None,
                Some(&self.publisher),
                Some(SourceLoadInputs {
                    captured: Some(captured),
                    snapshot: captured.snapshot(),
                    build_plan: self.submission.build_plan.as_ref().unwrap(),
                    module_index: self.submission.module_index.as_ref().unwrap(),
                    allocator: &self.ids,
                }),
            );
            dispatcher.phase_results = self
                .submission
                .scheduler_run
                .as_ref()
                .unwrap()
                .phase_results
                .clone();
            let importer_id = self.task(TaskKind::Frontend, "main").id.clone();
            dispatcher.phase_results.remove(&importer_id);
            dispatcher.native_cache = Some((
                NativeCacheOptions {
                    policy: &self.policy,
                    store: &self.store,
                    mode: NativeCacheMode::PersistentPrefix,
                },
                self.submission.task_graph.as_ref().unwrap(),
                CancellationSignal::default(),
            ));
            let protected_outputs = dispatcher
                .phase_results
                .values()
                .flatten()
                .flat_map(|result| result.output_refs.iter().map(|output| output.output()))
                .collect();
            self.publisher.storage().collect(CollectInput {
                snapshot: captured.snapshot().id,
                protected_outputs,
            });
            dispatcher
        }

        fn invoke(
            &self,
            dispatcher: &mut RegistrySchedulerDispatcher<'_>,
        ) -> Option<CacheSchedulingOutcome> {
            let task = self.task(TaskKind::Frontend, "main");
            dispatcher.cache_decision(SchedulerDispatchTask {
                task,
                snapshot: self.submission.session.captured().snapshot().id,
                cancellation: None,
            })
        }

        fn importer_cache_key(&self) -> mizar_cache::cache_key::CacheKey {
            use mizar_frontend::{
                orchestration::FrontendOutput,
                parsing::{MizarParserSeam, ParserSeam},
            };

            let snapshot = self.submission.session.captured().snapshot();
            let run = self.submission.scheduler_run.as_ref().unwrap();
            let get = |kind, module: &str| {
                let task = self.task(kind, module);
                run.phase_results[&task.id][0].output_refs[0].clone()
            };
            let own_source = get(TaskKind::SourceLoad, "main");
            let leaf_frontend = get(TaskKind::Frontend, "leaf");
            let twig_frontend = get(TaskKind::Frontend, "twig");
            let leaf_source = get(TaskKind::SourceLoad, "leaf");
            let twig_source = get(TaskKind::SourceLoad, "twig");
            let parents = [own_source, leaf_frontend, twig_frontend]
                .into_iter()
                .map(|parent| {
                    SealedParentOutputHandle::from_published_output(
                        &self.publisher,
                        snapshot.id,
                        parent,
                    )
                    .unwrap()
                })
                .collect();
            let main_version = snapshot
                .source_versions
                .iter()
                .find(|version| version.module_path.as_str() == "main")
                .unwrap();
            let input = crate::registry::PhaseInput::new(
                self.task(TaskKind::Frontend, "main").unit.clone(),
                PhaseDispatchInputBundle::new(
                    snapshot.id,
                    crate::frontend_adapter::source_input_hash(main_version),
                    Vec::new(),
                    parents,
                )
                .unwrap(),
            );
            let source_inputs = SourceLoadInputs {
                captured: Some(self.submission.session.captured()),
                snapshot,
                build_plan: self.submission.build_plan.as_ref().unwrap(),
                module_index: self.submission.module_index.as_ref().unwrap(),
                allocator: &self.ids,
            };
            let adapter = mizar_ir::cache_adapter::IrCacheAdapter::new(self.publisher.clone());
            let service = self.driver.registry().native_frontend_service().unwrap();
            let handle = self
                .publisher
                .storage()
                .typed_handle::<FrontendOutput<<MizarParserSeam as ParserSeam>::Ast>>(
                    &get(TaskKind::Frontend, "main"),
                    &OutputKind::new("FrontendOutput"),
                )
                .unwrap();
            let mizar_ir::cache_adapter::EncodeCacheRecordOutcome::Encoded(record) = source_inputs
                .encode_native_disk_frontend_cache_record_with_frontend(
                    &input,
                    &self.policy,
                    &adapter,
                    handle,
                    service,
                    &[leaf_source, twig_source],
                )
                .unwrap()
            else {
                panic!("genuine imported Frontend output must encode a cache record")
            };
            record.header.key.clone()
        }
    }

    #[test]
    fn multi_leaf_cache_hit_retains_every_real_parent_until_scheduler_consumption() {
        let case = CacheCase::new();
        let mut dispatcher = case.dispatcher();
        assert!(matches!(
            case.invoke(&mut dispatcher),
            Some(CacheSchedulingOutcome::ValidatedHit(_))
        ));
        assert_eq!(dispatcher.cache_owners.len(), 6);
        let retained = case.publisher.storage().collect(CollectInput {
            snapshot: case.submission.session.captured().snapshot().id,
            protected_outputs: Vec::new(),
        });
        assert_eq!(retained.retained_outputs, 6);
        for (_, output, owner) in &dispatcher.cache_owners {
            case.publisher.storage().release(*output, owner).unwrap();
        }
        let released = case.publisher.storage().collect(CollectInput {
            snapshot: case.submission.session.captured().snapshot().id,
            protected_outputs: Vec::new(),
        });
        assert_eq!(released.retained_outputs, 0);
    }

    #[test]
    fn multi_leaf_cache_hit_releases_partial_acquisition_when_later_parent_is_collected() {
        let case = CacheCase::new();
        let run = case.submission.scheduler_run.as_ref().unwrap();
        let missing_later_parent =
            run.phase_results[&case.task(TaskKind::SourceLoad, "twig").id][0].output_refs[0]
                .output();
        let protected = run
            .phase_results
            .values()
            .flatten()
            .flat_map(|result| result.output_refs.iter().map(|output| output.output()))
            .filter(|output| *output != missing_later_parent)
            .collect();
        case.publisher.storage().collect(CollectInput {
            snapshot: case.submission.session.captured().snapshot().id,
            protected_outputs: protected,
        });

        let mut dispatcher = case.dispatcher();
        assert!(case.invoke(&mut dispatcher).is_none());
        assert!(dispatcher.cache_owners.is_empty());
        let after = case.publisher.storage().collect(CollectInput {
            snapshot: case.submission.session.captured().snapshot().id,
            protected_outputs: Vec::new(),
        });
        assert_eq!(
            after.retained_outputs, 0,
            "owners acquired before the missing parent are released"
        );
    }

    #[test]
    fn multi_leaf_cache_lookup_requires_all_accepted_frontend_and_source_results() {
        for missing in [(TaskKind::Frontend, "twig"), (TaskKind::SourceLoad, "twig")] {
            let case = CacheCase::new();
            let task = case.task(missing.0, missing.1);
            let mut dispatcher = case.dispatcher();
            dispatcher.phase_results.remove(&task.id);
            assert!(case.invoke(&mut dispatcher).is_none());
            assert!(dispatcher.cache_owners.is_empty());
        }
    }

    #[test]
    fn multi_leaf_cache_lookup_observes_cancellation_and_publisher_replacement() {
        let cancelled = CacheCase::new();
        let mut dispatcher = cancelled.dispatcher();
        let (_, _, signal) = dispatcher.native_cache.as_mut().unwrap();
        signal.request(CancellationReason::ExplicitRequest);
        assert!(cancelled.invoke(&mut dispatcher).is_none());
        assert!(dispatcher.cache_owners.is_empty());

        let replaced = CacheCase::new();
        let snapshot = replaced.submission.session.captured().snapshot().id;
        replaced.publisher.mark_obsolete(snapshot).unwrap();
        let mut dispatcher = replaced.dispatcher();
        assert!(replaced.invoke(&mut dispatcher).is_none());
        assert!(dispatcher.cache_owners.is_empty());
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn multi_leaf_cache_rechecks_cancellation_and_publisher_during_store_read() {
        use std::{
            fs::{self, OpenOptions},
            io::Write,
            process::Command,
            thread,
        };

        for interruption in [
            Some(CancellationReason::ExplicitRequest),
            Some(CancellationReason::SupersededSnapshot),
            None,
        ] {
            let case = CacheCase::new();
            let key = case.importer_cache_key();
            let path = case.store.record_path(&key);
            let bytes = fs::read(&path).expect("cold submission writes the real importer record");
            fs::remove_file(&path).unwrap();
            assert!(
                Command::new("mkfifo")
                    .arg(&path)
                    .status()
                    .unwrap()
                    .success()
            );

            let mut dispatcher = case.dispatcher();
            let signal = dispatcher.native_cache.as_ref().unwrap().2.clone();
            let publisher = case.publisher.clone();
            let snapshot = case.submission.session.captured().snapshot().id;
            let storage = publisher.storage().clone();
            let fifo_path = path.clone();
            let unblock_path = path.clone();
            thread::scope(|scope| {
                scope.spawn(move || {
                    let mut fifo = OpenOptions::new().write(true).open(fifo_path).unwrap();
                    let retained = storage.collect(CollectInput {
                        snapshot,
                        protected_outputs: Vec::new(),
                    });
                    assert_eq!(retained.retained_outputs, 5);
                    if let Some(reason) = interruption {
                        signal.request(reason);
                    } else {
                        publisher.mark_obsolete(snapshot).unwrap();
                    }
                    fifo.write_all(&bytes).unwrap();
                });
                let outcome = case.invoke(&mut dispatcher);
                // If a regression skips Store I/O, this nonblocking FIFO endpoint lets the
                // writer finish so the retained-parent assertion fails instead of hanging.
                let unblock = OpenOptions::new()
                    .read(true)
                    .write(true)
                    .open(unblock_path)
                    .unwrap();
                drop(unblock);
                assert!(outcome.is_none());
            });
            assert!(dispatcher.cache_owners.is_empty());
            assert!(
                !dispatcher
                    .phase_results
                    .contains_key(&case.task(TaskKind::Frontend, "main").id)
            );
            let released = case.publisher.storage().collect(CollectInput {
                snapshot,
                protected_outputs: Vec::new(),
            });
            assert_eq!(released.retained_outputs, 0);
            fs::remove_file(path).unwrap();
        }
    }

    #[test]
    fn multi_leaf_cache_miss_and_decode_error_release_all_acquired_parents() {
        use std::fs;

        for corrupt in [false, true] {
            let case = CacheCase::new();
            let path = case.store.record_path(&case.importer_cache_key());
            if corrupt {
                fs::write(&path, b"not a cache record").unwrap();
            } else {
                fs::remove_file(&path).unwrap();
            }
            let mut dispatcher = case.dispatcher();
            assert!(case.invoke(&mut dispatcher).is_none());
            assert!(dispatcher.cache_owners.is_empty());
            let released = case.publisher.storage().collect(CollectInput {
                snapshot: case.submission.session.captured().snapshot().id,
                protected_outputs: Vec::new(),
            });
            assert_eq!(released.retained_outputs, 0);
        }
    }
}
