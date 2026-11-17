use mizar_build::cancel::CancellationPolicy;
use mizar_session::{
    DependencyArtifactRef, Hash, InMemorySessionIdAllocator, ModulePath, PackageId,
    SnapshotRegistry, ToolchainInfo, WorkspaceRoot,
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
    use std::{fs, sync::Arc};

    use super::*;
    use crate::{
        driver::{CompilerDriver, NativeCacheMode, SubmitOptions},
        frontend_adapter::ImportedFixture,
        registry::{PhaseStatus, SourceLoadInputs, StageConfig, assemble_registry},
    };
    use mizar_build::{
        cache_seam::CacheSchedulingOutcome,
        cancel::{CancellationReason, CancellationSignal},
        module_index::ModuleId,
        scheduler::{SchedulerDispatchTask, SchedulerTaskDispatcher},
        task_graph::{
            ModuleDependencyEdge, ModuleDependencyKind, ModuleDependencyOverlay, TaskKind, WorkUnit,
        },
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
        cross_package: bool,
        ids: InMemorySessionIdAllocator,
        policy: mizar_proof::policy::VerifierPolicy,
        store: CacheStoreRoot,
        publisher: Arc<PhaseOutputPublisher>,
        driver: CompilerDriver,
        submission: crate::driver::BuildSubmission,
    }

    impl CacheCase {
        fn new(intermediate: bool) -> Self {
            Self::build(intermediate, false)
        }

        fn cross_package_chain() -> Self {
            Self::build(true, true)
        }

        fn intermediate(cross_package: bool) -> Self {
            if cross_package {
                Self::cross_package_chain()
            } else {
                Self::new(true)
            }
        }

        fn build(intermediate: bool, cross_package: bool) -> Self {
            let fixture = if cross_package {
                ImportedFixture::cross_package_chain()
            } else {
                ImportedFixture::new()
            };
            if intermediate && !cross_package {
                fs::write(
                    fixture.root.join("alpha/src/main.miz"),
                    b"import alpha.leaf;\ndefinition\nend;\n",
                )
                .unwrap();
                fs::write(
                    fixture.root.join("alpha/src/leaf.miz"),
                    b"import alpha.twig;\ndefinition\nend;\n",
                )
                .unwrap();
            }
            let ids = InMemorySessionIdAllocator::new();
            let snapshots = SnapshotRegistry::new();
            let (session, mut input) = fixture.capture(&ids, &snapshots);
            if intermediate && !cross_package {
                input.dependency_overlay = ModuleDependencyOverlay::complete(vec![
                    ModuleDependencyEdge::new(
                        ModuleId::new(PackageId::new("alpha"), ModulePath::new("main")),
                        ModuleId::new(PackageId::new("alpha"), ModulePath::new("leaf")),
                        ModuleDependencyKind::ImportSummary,
                    ),
                    ModuleDependencyEdge::new(
                        ModuleId::new(PackageId::new("alpha"), ModulePath::new("leaf")),
                        ModuleId::new(PackageId::new("alpha"), ModulePath::new("twig")),
                        ModuleDependencyKind::ImportSummary,
                    ),
                ]);
            }
            let snapshot = session.captured().snapshot().id;
            let policy = mizar_proof::policy::VerifierPolicy::release();
            let store = CacheStoreRoot::new(
                fixture
                    .root
                    .join(if cross_package { "app" } else { "alpha" })
                    .join(".mizar-cache"),
            );
            let mut config = StageConfig::frontend(vec![]);
            config.verifier_policy = policy.clone();
            let registry = assemble_registry(&config).unwrap();
            let publisher = Arc::new(PhaseOutputPublisher::new(
                Arc::new(IrStorageService::with_policy(StoragePolicy::default())),
                Arc::new(SnapshotHandleRegistry::new()),
            ));
            publisher.register_current_snapshot(snapshot);
            for (package, module) in if cross_package {
                [("app", "main"), ("app", "mid"), ("core", "mid")]
            } else {
                [("alpha", "main"), ("alpha", "leaf"), ("alpha", "twig")]
            } {
                for (phase, kind) in [("SourceLoad", "SourceUnit"), ("Frontend", "FrontendOutput")]
                {
                    publisher.allow_work_unit(AllowedWorkUnit::new(
                        IrPhase::new(phase),
                        OutputKind::new(kind),
                        IrUnit::new(format!("{:?}:{:?}", package, module)),
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
            for (package, module) in if cross_package {
                [("app", "main"), ("app", "mid"), ("core", "mid")]
            } else {
                [("alpha", "main"), ("alpha", "leaf"), ("alpha", "twig")]
            } {
                for kind in [TaskKind::SourceLoad, TaskKind::Frontend] {
                    let task = submission
                        .task_graph
                        .as_ref()
                        .unwrap()
                        .tasks()
                        .iter()
                        .find(|task| {
                            task.kind == kind
                                && matches!(&task.unit, WorkUnit::Module { module: id } if id.package.as_str() == package && id.path.as_str() == module)
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
                cross_package,
                ids,
                policy,
                store,
                publisher,
                driver,
                submission,
            }
        }

        fn middle(&self) -> &'static str {
            if self.cross_package {
                "app/mid"
            } else {
                "leaf"
            }
        }

        fn ancestor(&self) -> &'static str {
            if self.cross_package {
                "core/mid"
            } else {
                "twig"
            }
        }

        fn package(&self, target: &str) -> &'static str {
            if self.cross_package {
                if target.starts_with("core/") {
                    "core"
                } else {
                    "app"
                }
            } else {
                "alpha"
            }
        }

        fn path<'a>(&self, target: &'a str) -> &'a str {
            target.split_once('/').map_or(target, |(_, path)| path)
        }

        fn task(&self, kind: TaskKind, target: &str) -> &BuildTask {
            let package = self.package(target);
            let path = self.path(target);
            self.submission.task_graph.as_ref().unwrap().tasks().iter().find(|task| {
                task.kind == kind
                    && matches!(&task.unit, WorkUnit::Module { module: id } if id.package.as_str() == package && id.path.as_str() == path)
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
            self.importer_cache_key_with_order(false)
        }

        fn importer_cache_key_with_order(&self, reverse: bool) -> mizar_cache::cache_key::CacheKey {
            use mizar_frontend::{
                orchestration::FrontendOutput,
                parsing::{MizarParserSeam, ParserSeam},
            };

            let run = self.submission.scheduler_run.as_ref().unwrap();
            let snapshot = self.submission.session.captured().snapshot();
            let graph = self.submission.task_graph.as_ref().unwrap();
            let importer = self.task(TaskKind::Frontend, "main");
            let (mut direct, mut supplemental) =
                native_frontend_cache_parents(importer, graph, &run.phase_results)
                    .expect("genuine imported closure supplies cache parents");
            if reverse {
                direct.reverse();
                supplemental.reverse();
            }
            let parents = direct
                .into_iter()
                .map(|parent| {
                    SealedParentOutputHandle::from_published_output(
                        &self.publisher,
                        self.submission.session.captured().snapshot().id,
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
                importer.unit.clone(),
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
                    &run.phase_results[&importer.id][0].output_refs[0],
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
                    &supplemental,
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
        let case = CacheCase::new(false);
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
        let case = CacheCase::new(false);
        let run = case.submission.scheduler_run.as_ref().unwrap();
        let missing_later_parent = run.phase_results
            [&case.task(TaskKind::SourceLoad, case.ancestor()).id][0]
            .output_refs[0]
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
    fn intermediate_chain_cache_hit_retains_every_ancestor_and_releases_partial_acquisition() {
        for cross_package in [false, true] {
            let case = CacheCase::intermediate(cross_package);
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
            assert_eq!(
                case.publisher
                    .storage()
                    .collect(CollectInput {
                        snapshot: case.submission.session.captured().snapshot().id,
                        protected_outputs: Vec::new(),
                    })
                    .retained_outputs,
                0
            );

            for (kind, module) in [
                (TaskKind::SourceLoad, "main"),
                (TaskKind::Frontend, case.middle()),
                (TaskKind::SourceLoad, case.middle()),
                (TaskKind::Frontend, case.ancestor()),
                (TaskKind::SourceLoad, case.ancestor()),
            ] {
                let case = CacheCase::intermediate(cross_package);
                let missing = case
                    .submission
                    .scheduler_run
                    .as_ref()
                    .unwrap()
                    .phase_results[&case.task(kind, module).id][0]
                    .output_refs[0]
                    .output();
                let protected = case
                    .submission
                    .scheduler_run
                    .as_ref()
                    .unwrap()
                    .phase_results
                    .values()
                    .flatten()
                    .flat_map(|result| result.output_refs.iter().map(|output| output.output()))
                    .filter(|output| *output != missing)
                    .collect();
                case.publisher.storage().collect(CollectInput {
                    snapshot: case.submission.session.captured().snapshot().id,
                    protected_outputs: protected,
                });
                let mut dispatcher = case.dispatcher();
                for output in dispatcher
                    .phase_results
                    .values()
                    .flatten()
                    .flat_map(|result| &result.output_refs)
                {
                    assert_eq!(
                        case.publisher.storage().validate_handle(output).is_ok(),
                        output.output() != missing,
                        "only {kind:?} {module} is missing: {output:?}"
                    );
                }
                assert!(case.invoke(&mut dispatcher).is_none(), "{kind:?} {module}");
                assert!(dispatcher.cache_owners.is_empty(), "{kind:?} {module}");
                assert_eq!(
                    case.publisher
                        .storage()
                        .collect(CollectInput {
                            snapshot: case.submission.session.captured().snapshot().id,
                            protected_outputs: Vec::new(),
                        })
                        .retained_outputs,
                    0,
                    "partial owners are released after {kind:?} {module} is missing"
                );
            }
        }
    }

    #[test]
    fn intermediate_chain_cache_requires_each_ancestor_result_to_be_accepted() {
        for cross_package in [false, true] {
            let case = CacheCase::intermediate(cross_package);
            let mut dispatcher = case.dispatcher();
            let middle = case.task(TaskKind::Frontend, case.middle()).id.clone();
            let ancestor = case.task(TaskKind::Frontend, case.ancestor()).id.clone();
            let middle_result = dispatcher.phase_results.remove(&middle).unwrap();
            let ancestor_result = dispatcher.phase_results.remove(&ancestor).unwrap();
            assert!(case.invoke(&mut dispatcher).is_none());
            assert!(dispatcher.cache_owners.is_empty());
            dispatcher.phase_results.insert(ancestor, ancestor_result);
            assert!(case.invoke(&mut dispatcher).is_none());
            assert!(dispatcher.cache_owners.is_empty());
            dispatcher.phase_results.insert(middle, middle_result);
            assert!(matches!(
                case.invoke(&mut dispatcher),
                Some(CacheSchedulingOutcome::ValidatedHit(_))
            ));
            drop(dispatcher);
            for (kind, module) in [
                (TaskKind::Frontend, case.middle()),
                (TaskKind::SourceLoad, case.middle()),
                (TaskKind::Frontend, case.ancestor()),
                (TaskKind::SourceLoad, case.ancestor()),
            ] {
                let case = CacheCase::intermediate(cross_package);
                let mut dispatcher = case.dispatcher();
                let result = &mut dispatcher
                    .phase_results
                    .get_mut(&case.task(kind, module).id)
                    .unwrap()[0];
                result.status = PhaseStatus::Recoverable;
                assert!(case.invoke(&mut dispatcher).is_none(), "{kind:?} {module}");
                assert!(dispatcher.cache_owners.is_empty());
                assert!(
                    !dispatcher
                        .phase_results
                        .contains_key(&case.task(TaskKind::Frontend, "main").id)
                );
            }

            let case = CacheCase::intermediate(cross_package);
            let mut dispatcher = case.dispatcher();
            let middle = case.task(TaskKind::Frontend, case.middle()).id.clone();
            let ancestor = case.task(TaskKind::Frontend, case.ancestor()).id.clone();
            let wrong = dispatcher.phase_results[&ancestor][0].output_refs[0].clone();
            dispatcher.phase_results.get_mut(&middle).unwrap()[0].output_refs[0] = wrong;
            assert!(case.invoke(&mut dispatcher).is_none());
            assert!(dispatcher.cache_owners.is_empty());
            assert!(
                !dispatcher
                    .phase_results
                    .contains_key(&case.task(TaskKind::Frontend, "main").id)
            );

            for status in [PhaseStatus::Fatal, PhaseStatus::Cancelled] {
                let case = CacheCase::intermediate(cross_package);
                let mut dispatcher = case.dispatcher();
                dispatcher
                    .phase_results
                    .get_mut(&case.task(TaskKind::Frontend, case.ancestor()).id)
                    .unwrap()[0]
                    .status = status;
                assert!(
                    case.invoke(&mut dispatcher).is_none(),
                    "{status:?} ancestor"
                );
                assert!(dispatcher.cache_owners.is_empty());
                assert!(
                    !dispatcher
                        .phase_results
                        .contains_key(&case.task(TaskKind::Frontend, "main").id)
                );
            }

            let case = CacheCase::intermediate(cross_package);
            let mut dispatcher = case.dispatcher();
            let middle = case.task(TaskKind::SourceLoad, case.middle()).id.clone();
            let ancestor = case.task(TaskKind::SourceLoad, case.ancestor()).id.clone();
            let wrong = dispatcher.phase_results[&ancestor][0].output_refs[0].clone();
            dispatcher.phase_results.get_mut(&middle).unwrap()[0].output_refs[0] = wrong;
            assert!(case.invoke(&mut dispatcher).is_none());
            assert!(dispatcher.cache_owners.is_empty());
            assert!(
                !dispatcher
                    .phase_results
                    .contains_key(&case.task(TaskKind::Frontend, "main").id)
            );
        }
    }

    #[test]
    fn intermediate_chain_cache_rejects_coherently_rehashed_corrupt_payload() {
        use mizar_cache::cache_store::{CacheLookupOutcome, CacheRecord};

        for cross_package in [false, true] {
            let case = CacheCase::intermediate(cross_package);
            let key = case.importer_cache_key();
            let CacheLookupOutcome::Hit(record) = case.store.lookup(&key) else {
                panic!("cold chain run writes a genuine intermediate record");
            };
            let mut corrupted = record.output.clone();
            *corrupted.last_mut().expect("cache payload is nonempty") ^= 0x01;
            fs::remove_file(case.store.record_path(&key)).unwrap();
            case.store
                .insert(&CacheRecord::new_inline(
                    key,
                    record.header.produced_by.clone(),
                    corrupted,
                ))
                .unwrap();
            let CacheLookupOutcome::Hit(rewritten) = case.store.lookup(&case.importer_cache_key())
            else {
                panic!("rewritten corrupt record remains addressable");
            };
            assert_ne!(rewritten.output, record.output);

            let mut dispatcher = case.dispatcher();
            assert!(case.invoke(&mut dispatcher).is_none());
            assert!(dispatcher.cache_owners.is_empty());
            assert!(
                !dispatcher
                    .phase_results
                    .contains_key(&case.task(TaskKind::Frontend, "main").id)
            );
        }
    }

    #[test]
    fn native_frontend_closure_keys_ignore_parent_permutation() {
        for case in [CacheCase::new(true), CacheCase::cross_package_chain()] {
            let canonical = case.importer_cache_key();
            let reversed = case.importer_cache_key_with_order(true);
            assert_eq!(canonical, reversed);
            if !case.cross_package {
                let legacy = canonical
                    .input_hashes
                    .iter()
                    .find(|hash| hash.name == "frontend-dependencies")
                    .unwrap();
                assert_eq!(
                    legacy.domain,
                    "mizar-driver/native-disk-frontend-dependencies/v2"
                );
                assert_eq!(
                    legacy.digest,
                    Hash::from_bytes([
                        132, 224, 188, 102, 97, 213, 41, 144, 15, 166, 227, 196, 57, 182, 124, 200,
                        180, 9, 224, 75, 39, 137, 226, 185, 203, 67, 239, 104, 185, 54, 171, 45,
                    ])
                );
            }
        }
    }

    #[test]
    fn intermediate_chain_refuses_incomplete_duplicate_and_foreign_closure_for_reads_and_writes() {
        use mizar_frontend::{
            orchestration::FrontendOutput,
            parsing::{MizarParserSeam, ParserSeam},
        };
        use mizar_ir::cache_adapter::IrCacheAdapter;
        for cross_package in [false, true] {
            let case = CacheCase::intermediate(cross_package);
            let foreign = CacheCase::intermediate(cross_package);
            let snapshot = case.submission.session.captured().snapshot();
            let graph = case.submission.task_graph.as_ref().unwrap();
            let run = case.submission.scheduler_run.as_ref().unwrap();
            let task = case.task(TaskKind::Frontend, "main");
            let (direct, supplemental) =
                native_frontend_cache_parents(task, graph, &run.phase_results).unwrap();
            let version = snapshot
                .source_versions
                .iter()
                .find(|version| version.module_path.as_str() == "main")
                .unwrap();
            let input = crate::registry::PhaseInput::new(
                task.unit.clone(),
                PhaseDispatchInputBundle::new(
                    snapshot.id,
                    crate::frontend_adapter::source_input_hash(version),
                    Vec::new(),
                    direct
                        .iter()
                        .map(|parent| {
                            SealedParentOutputHandle::from_published_output(
                                &case.publisher,
                                snapshot.id,
                                parent.clone(),
                            )
                            .unwrap()
                        })
                        .collect(),
                )
                .unwrap(),
            );
            let inputs = SourceLoadInputs {
                captured: Some(case.submission.session.captured()),
                snapshot,
                build_plan: case.submission.build_plan.as_ref().unwrap(),
                module_index: case.submission.module_index.as_ref().unwrap(),
                allocator: &case.ids,
            };
            let adapter = IrCacheAdapter::new(case.publisher.clone());
            let service = case.driver.registry().native_frontend_service().unwrap();
            let handle = case
                .publisher
                .storage()
                .typed_handle::<FrontendOutput<<MizarParserSeam as ParserSeam>::Ast>>(
                    &run.phase_results[&task.id][0].output_refs[0],
                    &OutputKind::new("FrontendOutput"),
                )
                .unwrap();
            assert!(matches!(
                case.store.lookup(&case.importer_cache_key()),
                mizar_cache::cache_store::CacheLookupOutcome::Hit(_)
            ));
            for index in 0..supplemental.len() {
                let mut missing = supplemental.clone();
                missing.remove(index);
                let mut duplicate = supplemental.clone();
                duplicate.push(supplemental[index].clone());
                let mut wrong = supplemental.clone();
                wrong[index] = direct[0].clone();
                let mut foreign_parents = supplemental.clone();
                foreign_parents[index] = foreign
                    .submission
                    .scheduler_run
                    .as_ref()
                    .unwrap()
                    .phase_results
                    .values()
                    .flatten()
                    .flat_map(|result| &result.output_refs)
                    .find(|output| {
                        output.phase() == supplemental[index].phase()
                            && output.work_unit() == supplemental[index].work_unit()
                    })
                    .unwrap()
                    .clone();
                for invalid in [missing, duplicate, wrong, foreign_parents] {
                    assert!(
                        inputs
                            .rehydrate_native_disk_frontend_cache_with_dependencies(
                                &input,
                                &case.policy,
                                &case.store,
                                &adapter,
                                service,
                                &invalid
                            )
                            .is_err()
                    );
                    assert!(
                        inputs
                            .encode_native_disk_frontend_cache_record_with_frontend(
                                &input,
                                &case.policy,
                                &adapter,
                                handle.clone(),
                                service,
                                &invalid
                            )
                            .is_err()
                    );
                    assert_eq!(adapter.successful_rehydrations(), 0);
                }
            }
            // The complete genuine set still prepares the same record after every refusal.
            assert!(matches!(
                inputs
                    .encode_native_disk_frontend_cache_record_with_frontend(
                        &input,
                        &case.policy,
                        &adapter,
                        handle,
                        service,
                        &supplemental
                    )
                    .unwrap(),
                mizar_ir::cache_adapter::EncodeCacheRecordOutcome::Encoded(_)
            ));
        }
    }

    #[test]
    fn intermediate_chain_rejects_ancestor_with_forged_semantic_commitment() {
        use mizar_frontend::{
            orchestration::FrontendOutput,
            parsing::{MizarParserSeam, ParserSeam},
            source::SourceUnit,
        };
        use mizar_ir::{
            publisher::{OutputOrigin, PublicationTarget, PublishOutputInput},
            storage::{AnyPhaseOutputRef, BlobDecodeError, BlobDecoder, SchemaVersion},
        };

        for cross_package in [false, true] {
            let case = CacheCase::intermediate(cross_package);
            let captured = case.submission.session.captured();
            let snapshot = captured.snapshot();
            let run = case.submission.scheduler_run.as_ref().unwrap();
            let graph = case.submission.task_graph.as_ref().unwrap();
            let middle = case.middle();
            let ancestor = case.ancestor();
            for attack in [
                None,
                Some((TaskKind::SourceLoad, ancestor)),
                Some((TaskKind::Frontend, ancestor)),
                Some((TaskKind::SourceLoad, middle)),
                Some((TaskKind::Frontend, middle)),
            ]
            .into_iter()
            .chain(cross_package.then_some(Some((TaskKind::Frontend, "swapped-source"))))
            {
                let swapped_lineage = attack == Some((TaskKind::Frontend, "swapped-source"));
                let publisher = Arc::new(PhaseOutputPublisher::new(
                    Arc::new(IrStorageService::with_policy(StoragePolicy::default())),
                    Arc::new(SnapshotHandleRegistry::new()),
                ));
                publisher.register_current_snapshot(snapshot.id);
                publisher.allow_work_unit(AllowedWorkUnit::new(
                    IrPhase::new("Frontend"),
                    OutputKind::new("FrontendOutput"),
                    IrUnit::new(format!("{:?}:{:?}", case.package("main"), "main")),
                ));
                let mut copied: Vec<(mizar_ir::identity::PhaseOutputId, AnyPhaseOutputRef)> =
                    Vec::new();
                let mut results = run.phase_results.clone();
                for (kind, module) in [
                    (TaskKind::SourceLoad, ancestor),
                    (TaskKind::Frontend, ancestor),
                    (TaskKind::SourceLoad, middle),
                    (TaskKind::Frontend, middle),
                    (TaskKind::SourceLoad, "main"),
                ] {
                    let task = case.task(kind, module);
                    let original = &run.phase_results[&task.id][0].output_refs[0];
                    publisher.allow_work_unit(AllowedWorkUnit::new(
                        original.phase().clone(),
                        original.output_kind().clone(),
                        original.work_unit().clone(),
                    ));
                    let mut parents: Vec<AnyPhaseOutputRef> = original
                        .lineage()
                        .parents
                        .iter()
                        .map(|id| copied.iter().find(|(old, _)| old == id).unwrap().1.clone())
                        .collect();
                    if swapped_lineage && kind == TaskKind::Frontend && module == middle {
                        let ancestor_source = &run.phase_results
                            [&case.task(TaskKind::SourceLoad, ancestor).id][0]
                            .output_refs[0];
                        let wrong_source = copied
                            .iter()
                            .find(|(old, _)| *old == ancestor_source.output())
                            .unwrap()
                            .1
                            .clone();
                        let own_source = parents
                            .iter_mut()
                            .find(|parent| parent.phase().as_str() == "SourceLoad")
                            .unwrap();
                        *own_source = wrong_source;
                    }
                    let descriptor = captured
                        .native_sources()
                        .unwrap()
                        .iter()
                        .find(|(_, _, source)| {
                            source.package_id.as_str() == case.package(module)
                                && source.module_path.as_str() == case.path(module)
                        })
                        .unwrap()
                        .2
                        .clone();
                    let forged = attack == Some((kind, module));
                    let replacement = if kind == TaskKind::SourceLoad {
                        let typed = case
                            .publisher
                            .storage()
                            .typed_handle::<SourceUnit>(original, &OutputKind::new("SourceUnit"))
                            .unwrap();
                        let payload = case.publisher.storage().get(&typed).unwrap();
                        let source_id = payload.source_id;
                        let semantic = if forged {
                            b"forged source commitment".to_vec()
                        } else {
                            original.lineage().named_input_hashes[0]
                                .digest
                                .as_bytes()
                                .to_vec()
                        };
                        publisher
                            .publish(PublishOutputInput {
                                slot: publisher.allocate(
                                    snapshot.id,
                                    original.phase().clone(),
                                    original.work_unit().clone(),
                                    original.output_kind().clone(),
                                    SchemaVersion::new(1),
                                ),
                                snapshot: snapshot.id,
                                phase: original.phase().clone(),
                                work_unit: original.work_unit().clone(),
                                output_kind: original.output_kind().clone(),
                                schema_version: SchemaVersion::new(1),
                                payload: (*payload).clone(),
                                canonical_payload: Some(semantic),
                                storage_payload: payload.canonical_disk_bytes(),
                                decode: BlobDecoder::new(move |bytes| {
                                    SourceUnit::from_canonical_storage_bytes(
                                        bytes,
                                        source_id,
                                        &descriptor,
                                    )
                                    .ok_or_else(|| BlobDecodeError::new("invalid source bytes"))
                                }),
                                parents,
                                named_input_hashes: original.lineage().named_input_hashes.clone(),
                                side_tables: (*case
                                    .publisher
                                    .storage()
                                    .side_tables(&typed)
                                    .unwrap())
                                .clone(),
                                origin: OutputOrigin::PackageSource,
                                target: PublicationTarget::CurrentPackage,
                            })
                            .unwrap()
                            .erase()
                    } else {
                        let typed = case
                            .publisher
                            .storage()
                            .typed_handle::<FrontendOutput<<MizarParserSeam as ParserSeam>::Ast>>(
                                original,
                                &OutputKind::new("FrontendOutput"),
                            )
                            .unwrap();
                        let payload = case.publisher.storage().get(&typed).unwrap();
                        let source_id = payload.source.source_id;
                        let mut semantic = b"mizar-frontend/output-publication/v1\0".to_vec();
                        semantic.extend_from_slice(
                            payload
                                .cache_keys
                                .ast
                                .as_ref()
                                .unwrap()
                                .stable_hash()
                                .as_bytes(),
                        );
                        if forged {
                            semantic = b"forged frontend commitment".to_vec();
                        }
                        publisher
                            .publish(PublishOutputInput {
                                slot: publisher.allocate(
                                    snapshot.id,
                                    original.phase().clone(),
                                    original.work_unit().clone(),
                                    original.output_kind().clone(),
                                    SchemaVersion::new(1),
                                ),
                                snapshot: snapshot.id,
                                phase: original.phase().clone(),
                                work_unit: original.work_unit().clone(),
                                output_kind: original.output_kind().clone(),
                                schema_version: SchemaVersion::new(1),
                                payload: (*payload).clone(),
                                canonical_payload: Some(semantic),
                                storage_payload: payload.canonical_disk_bytes(),
                                decode: BlobDecoder::new(move |bytes| {
                                    FrontendOutput::from_canonical_storage_bytes(
                                        bytes,
                                        source_id,
                                        &descriptor,
                                    )
                                    .ok_or_else(|| BlobDecodeError::new("invalid frontend bytes"))
                                }),
                                parents,
                                named_input_hashes: original.lineage().named_input_hashes.clone(),
                                side_tables: (*case
                                    .publisher
                                    .storage()
                                    .side_tables(&typed)
                                    .unwrap())
                                .clone(),
                                origin: OutputOrigin::PackageSource,
                                target: PublicationTarget::CurrentPackage,
                            })
                            .unwrap()
                            .erase()
                    };
                    if swapped_lineage && kind == TaskKind::Frontend && module == middle {
                        assert!(publisher.storage().validate_handle(&replacement).is_ok());
                        assert!(
                            publisher
                                .validate_output(
                                    snapshot.id,
                                    &replacement,
                                    mizar_ir::publisher::OutputAuthorization::CurrentPackage
                                )
                                .is_ok()
                        );
                    }
                    if forged {
                        assert_ne!(replacement.content_hash(), original.content_hash());
                    } else if attack.is_none() {
                        assert_eq!(
                            replacement.content_hash(),
                            original.content_hash(),
                            "{kind:?} {module}"
                        );
                    }
                    assert_eq!(replacement.side_table_hash(), original.side_table_hash());
                    copied.push((original.output(), replacement.clone()));
                    results.get_mut(&task.id).unwrap()[0].output_refs = vec![replacement];
                }
                let (mut direct, mut supplemental) = native_frontend_cache_parents(
                    case.task(TaskKind::Frontend, "main"),
                    graph,
                    if swapped_lineage {
                        &run.phase_results
                    } else {
                        &results
                    },
                )
                .expect("genuine direct relations are current and structurally valid");
                if swapped_lineage {
                    for parent in direct.iter_mut().chain(&mut supplemental) {
                        *parent = copied
                            .iter()
                            .find(|(old, _)| *old == parent.output())
                            .unwrap()
                            .1
                            .clone();
                    }
                }
                let parents = direct
                    .into_iter()
                    .map(|parent| {
                        SealedParentOutputHandle::from_published_output(
                            &publisher,
                            snapshot.id,
                            parent,
                        )
                        .unwrap()
                    })
                    .collect();
                let version = snapshot
                    .source_versions
                    .iter()
                    .find(|v| v.module_path.as_str() == "main")
                    .unwrap();
                let input = crate::registry::PhaseInput::new(
                    case.task(TaskKind::Frontend, "main").unit.clone(),
                    PhaseDispatchInputBundle::new(
                        snapshot.id,
                        crate::frontend_adapter::source_input_hash(version),
                        Vec::new(),
                        parents,
                    )
                    .unwrap(),
                );
                let inputs = SourceLoadInputs {
                    captured: Some(captured),
                    snapshot,
                    build_plan: case.submission.build_plan.as_ref().unwrap(),
                    module_index: case.submission.module_index.as_ref().unwrap(),
                    allocator: &case.ids,
                };
                let adapter = mizar_ir::cache_adapter::IrCacheAdapter::new(publisher);
                let outcome = inputs.rehydrate_native_disk_frontend_cache_with_dependencies(
                    &input,
                    &case.policy,
                    &case.store,
                    &adapter,
                    case.driver.registry().native_frontend_service().unwrap(),
                    &supplemental,
                );
                if let Some((kind, _)) = attack {
                    let Err(error) = outcome else {
                        panic!("forged {attack:?} must fail before Store lookup");
                    };
                    assert_eq!(
                        error,
                        if swapped_lineage {
                            "native Frontend dependency authentication failed"
                        } else if kind == TaskKind::SourceLoad {
                            "native Frontend source commitment mismatch"
                        } else {
                            "native Frontend independent commitment mismatch"
                        }
                    );
                    assert_eq!(adapter.successful_rehydrations(), 0);
                    assert!(matches!(
                        case.store.lookup(&case.importer_cache_key()),
                        mizar_cache::cache_store::CacheLookupOutcome::Hit(_)
                    ));
                } else {
                    let outcome = outcome.unwrap();
                    assert!(
                        matches!(
                            outcome,
                            mizar_ir::cache_adapter::CacheRehydrateOutcome::Rehydrated(_)
                        ),
                        "{outcome:?}"
                    );
                }
            }
        }
    }

    #[test]
    fn multi_leaf_cache_lookup_requires_all_accepted_frontend_and_source_results() {
        for missing in [(TaskKind::Frontend, "twig"), (TaskKind::SourceLoad, "twig")] {
            let case = CacheCase::new(false);
            let task = case.task(missing.0, missing.1);
            let mut dispatcher = case.dispatcher();
            dispatcher.phase_results.remove(&task.id);
            assert!(case.invoke(&mut dispatcher).is_none());
            assert!(dispatcher.cache_owners.is_empty());
        }
    }

    #[test]
    fn multi_leaf_cache_lookup_observes_cancellation_and_publisher_replacement() {
        for (cancelled, replaced) in [
            (CacheCase::new(false), CacheCase::new(false)),
            (CacheCase::new(true), CacheCase::new(true)),
            (
                CacheCase::cross_package_chain(),
                CacheCase::cross_package_chain(),
            ),
        ] {
            let mut dispatcher = cancelled.dispatcher();
            let (_, _, signal) = dispatcher.native_cache.as_mut().unwrap();
            signal.request(CancellationReason::ExplicitRequest);
            assert!(cancelled.invoke(&mut dispatcher).is_none());
            assert!(dispatcher.cache_owners.is_empty());

            let snapshot = replaced.submission.session.captured().snapshot().id;
            replaced.publisher.mark_obsolete(snapshot).unwrap();
            let mut dispatcher = replaced.dispatcher();
            assert!(replaced.invoke(&mut dispatcher).is_none());
            assert!(dispatcher.cache_owners.is_empty());
        }
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

        for (intermediate, cross_package) in [(false, false), (true, false), (true, true)] {
            for interruption in [
                Some(CancellationReason::ExplicitRequest),
                Some(CancellationReason::SupersededSnapshot),
                None,
            ] {
                let case = if cross_package {
                    CacheCase::cross_package_chain()
                } else {
                    CacheCase::new(intermediate)
                };
                let key = case.importer_cache_key();
                let path = case.store.record_path(&key);
                let bytes =
                    fs::read(&path).expect("cold submission writes the real importer record");
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
    }

    #[test]
    fn multi_leaf_cache_miss_and_decode_error_release_all_acquired_parents() {
        use std::fs;

        for corrupt in [false, true] {
            let case = CacheCase::new(false);
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
