use super::*;

#[test]
fn scheduled_theorem_atp_solve_matches_genuine_publication() {
    use mizar_atp::{
        backend::{
            BackendCandidateEvidence, EncodedBackendProblem,
            build_source_theorem_candidate_inputs as build,
        },
        problem::*,
    };
    use mizar_build::task_graph::{
        BackendProfileId, EvidenceCandidateId, VcOrderKey, VcTaskDescriptor, VcTaskDescriptorId,
    };
    use mizar_core::core_ir::{CoreIr, CoreProofStatus, ObligationSeedStatus};
    use mizar_ir::storage::CollectInput;
    use mizar_vc::{
        discharge::{ComputationLimit, DischargeOutput, DischargePolicy},
        vc_ir::{
            GenerationSchemaVersion, PolicyKey, SeedOriginRef, VcSchemaVersion, VcSet, VcStatus,
        },
    };
    let text = "theorem T: for X being set holds X = X proof let X be set; thus X = X; end;";
    for (open, threshold, fragment, format) in [
        (false, usize::MAX, LogicFragment::Fof, ConcreteFormat::Tptp),
        (
            true,
            0,
            LogicFragment::SmtLibUninterpreted,
            ConcreteFormat::SmtLib,
        ),
    ] {
        with_native_summary_parent(
            text,
            open,
            threshold,
            |fixture, _, old, _, _, (session, ids, snapshots)| {
                let generation = GenerationSchemaVersion::new("λ\n\0");
                let schema = VcSchemaVersion::new("");
                let policy = DischargePolicy {
                    computation_limit: ComputationLimit {
                        policy: PolicyKey::new("α\n\0"),
                        max_steps: u32::MAX,
                    },
                };
                let profile = LogicProfile::try_new(
                    "caller λ\n\0",
                    fragment,
                    EqualitySupport::Supported,
                    QuantifierPolicy::FirstOrder,
                    SoftTypeStrategy::ScopedGuardPredicates,
                    NativePropertySupport::Supported,
                    [ConcreteFormat::Tptp, ConcreteFormat::SmtLib]
                        .into_iter()
                        .collect(),
                )
                .unwrap();
                let WorkUnit::Module { module } = &old.work_unit else {
                    panic!()
                };
                let label = VcTaskDescriptorId::new("opaque-atp-query:09");
                let baseline = publisher(old.snapshot, threshold, &old.work_unit);
                let owner = publisher(old.snapshot, threshold, &old.work_unit);
                for publisher in [&baseline, &owner] {
                    allow_frontend(publisher, &old.work_unit);
                    for (phase, kind) in [
                        ("ModuleResolve", "SourceTheoremModuleCandidates"),
                        ("SignatureCollection", "SourceTheoremModuleSummary"),
                        ("TypeChecking", "SourceTheoremTypedAst"),
                        ("RegistrationResolution", "SourceTheoremRegistrationClosure"),
                        ("OverloadResolution", "SourceTheoremMetadata"),
                        ("Elaboration", "SourceTheoremCore"),
                        ("AlgorithmPreparation", "SourceTheoremPreparation"),
                        ("VcGenerate", "SourceTheoremVC"),
                        ("VcDischarge", "SourceTheoremDischarge"),
                        ("AtpSolve", "SourceTheoremCandidateInputs"),
                    ] {
                        publisher.allow_work_unit(AllowedWorkUnit::new(
                            IrPipelinePhase::new(phase),
                            OutputKind::new(kind),
                            IrWorkUnit::new("\"alpha\":\"main\""),
                        ));
                    }
                }
                let mut builders = [PhaseRegistryBuilder::new(), PhaseRegistryBuilder::new()];
                for builder in &mut builders {
                    builder
                        .register_source_load()
                        .register_frontend(vec![])
                        .register_source_theorem_module_resolver()
                        .register_source_theorem_signature_collection()
                        .register_source_theorem_type_checking()
                        .register_source_theorem_registration_resolution()
                        .register_source_theorem_overload_resolution()
                        .register_source_theorem_elaboration()
                        .register_source_theorem_algorithm_preparation()
                        .register_source_theorem_vc_generation(generation.clone(), schema.clone())
                        .register_source_theorem_vc_discharge(policy.clone());
                }
                let [baseline_builder, mut builder] = builders;
                let (_, mut submit) = fixture.submit_request(vec![], vec![]);
                submit.vc_descriptors = vec![VcTaskDescriptor::new(
                    label.clone(),
                    module.clone(),
                    VcOrderKey::new("opaque-order"),
                    vec![],
                    vec![],
                )];
                let baseline_submission = CompilerDriver::new(baseline_builder.build().unwrap())
                    .with_output_publisher(baseline.clone())
                    .submit_captured(session.clone(), ids, snapshots, submit)
                    .unwrap();
                let (inputs, original) = native_disk_lookup_context(&baseline_submission, ids);
                let outputs = baseline_submission
                    .scheduler_run
                    .as_ref()
                    .unwrap()
                    .phase_results
                    .values()
                    .flatten()
                    .flat_map(|row| &row.output_refs)
                    .collect::<Vec<_>>();
                assert_eq!(outputs.len(), 11);
                let parent = |kind: &str| {
                    *outputs
                        .iter()
                        .find(|output| output.output_kind() == &OutputKind::new(kind))
                        .unwrap()
                };
                let core_ref = parent("SourceTheoremCore");
                let vc_ref = parent("SourceTheoremVC");
                let discharge_ref = parent("SourceTheoremDischarge");
                let core = baseline
                    .storage()
                    .get(
                        &baseline
                            .storage()
                            .typed_handle::<CoreIr>(core_ref, core_ref.output_kind())
                            .unwrap(),
                    )
                    .unwrap();
                let vcs = baseline
                    .storage()
                    .get(
                        &baseline
                            .storage()
                            .typed_handle::<VcSet>(vc_ref, vc_ref.output_kind())
                            .unwrap(),
                    )
                    .unwrap();
                let discharge = baseline
                    .storage()
                    .get(
                        &baseline
                            .storage()
                            .typed_handle::<DischargeOutput>(
                                discharge_ref,
                                discharge_ref.output_kind(),
                            )
                            .unwrap(),
                    )
                    .unwrap();
                let id = vcs.vcs()[0].id;
                let input = PhaseInput::new(
                    original.work_unit.clone(),
                    PhaseDispatchInputBundle::new(
                        old.snapshot,
                        original.identities().input_hash(),
                        vec![],
                        [core_ref, vc_ref, discharge_ref]
                            .into_iter()
                            .map(|parent| {
                                SealedParentOutputHandle::from_published_output(
                                    &baseline,
                                    old.snapshot,
                                    parent.clone(),
                                )
                                .unwrap()
                            })
                            .collect(),
                    )
                    .unwrap(),
                );
                let expected = build(
                    &core,
                    &vcs,
                    id,
                    &discharge,
                    &policy,
                    profile.clone(),
                    format,
                )
                .unwrap();
                assert_eq!(
                    inputs
                        .native_theorem_candidate_with_core_vc_discharge_parents(
                            &input,
                            &baseline,
                            id,
                            &policy,
                            profile.clone(),
                            format
                        )
                        .unwrap(),
                    Some(expected.clone())
                );
                let published = inputs
                    .publish_native_theorem_candidate(
                        &input,
                        &baseline,
                        id,
                        &policy,
                        profile.clone(),
                        format,
                    )
                    .unwrap()
                    .unwrap();
                assert_eq!(*baseline.storage().get(&published).unwrap(), expected);
                builder
                    .register_source_theorem_atp_solve(policy.clone(), profile.clone(), format)
                    .unwrap();
                let registry = builder.build().unwrap();
                let descriptor = registry
                    .descriptor_for_phase(PipelinePhase::AtpSolve)
                    .unwrap();
                assert_eq!(descriptor.service_name, "SourceTheoremAtpSolve");
                assert_eq!(
                    descriptor.owner,
                    mizar_driver::registry::PhaseOwner::MizarAtp
                );
                assert_eq!(descriptor.phases, [PipelinePhase::AtpSolve]);
                assert_eq!(descriptor.output_kind, "SourceTheoremCandidateInputs");
                assert_eq!(
                    descriptor.schema_version,
                    format!(
                        "mizar-atp/source-theorem-candidate-publication/v1;computation_policy={}:α\n\0;max_steps={};profile={}:caller λ\n\0;fragment={};equality=1;quantifiers=1;soft_types=3;native_properties=1;formats=3;format={}",
                        "α\n\0".len(),
                        u32::MAX,
                        "caller λ\n\0".len(),
                        if open { 2 } else { 0 },
                        if open { 1 } else { 0 }
                    )
                );
                let mut driver = CompilerDriver::new(registry).with_output_publisher(owner.clone());
                let (_, mut submit) = fixture.submit_request(vec![], vec![]);
                submit.worker_count = if open { 4 } else { 1 };
                submit.completion_order = if open {
                    CompletionOrder::Reverse
                } else {
                    CompletionOrder::Canonical
                };
                submit.vc_descriptors = vec![VcTaskDescriptor::new(
                    label.clone(),
                    module.clone(),
                    VcOrderKey::new("opaque-order"),
                    vec![BackendProfileId::new("actual-missing-backend")],
                    vec![EvidenceCandidateId::new("declared-evidence")],
                )];
                let before = (policy.clone(), profile.clone(), session.captured().clone());
                fixture.write(&[0xff]);
                fs::remove_file(fixture.root.join("alpha/src/main.miz")).unwrap();
                let submission = driver
                    .submit_captured(session.clone(), ids, snapshots, submit)
                    .unwrap();
                let (actual_inputs, _) = native_disk_lookup_context(&submission, ids);
                let graph = submission.task_graph.as_ref().unwrap();
                let run = submission.scheduler_run.as_ref().unwrap();
                let atp = graph
                    .tasks()
                    .iter()
                    .find(|task| task.kind == TaskKind::AtpSolve)
                    .unwrap();
                assert_eq!(
                    atp.unit,
                    WorkUnit::Vc {
                        module: module.clone(),
                        descriptor: label
                    }
                );
                assert_eq!(atp.phases, [PipelinePhase::AtpSolve]);
                assert_eq!(
                    run.task_states
                        .iter()
                        .find(|row| row.task_id == atp.id)
                        .unwrap()
                        .state,
                    TaskState::Completed
                );
                let [row] = run.phase_results[&atp.id].as_slice() else {
                    panic!()
                };
                assert_eq!(row.status, PhaseStatus::Complete);
                assert!(row.diagnostics.is_empty());
                let [actual] = row.output_refs.as_slice() else {
                    panic!()
                };
                let handle = owner
                    .storage()
                    .typed_handle::<(EncodedBackendProblem, BackendCandidateEvidence)>(
                        actual,
                        actual.output_kind(),
                    )
                    .unwrap();
                assert_eq!(*owner.storage().get(&handle).unwrap(), expected);
                assert_eq!(actual.work_unit(), &IrWorkUnit::new("\"alpha\":\"main\""));
                assert_eq!(actual.snapshot(), old.snapshot);
                assert_eq!(
                    actual.schema_version(),
                    mizar_ir::storage::SchemaVersion::new(1)
                );
                assert_eq!(actual.phase(), &IrPipelinePhase::new("AtpSolve"));
                assert_eq!(
                    actual.output_kind(),
                    &OutputKind::new("SourceTheoremCandidateInputs")
                );
                assert_eq!(actual.content_hash(), published.content_hash());
                assert_eq!(actual.side_table_hash(), published.side_table_hash());
                assert_eq!(
                    *owner.storage().side_tables(&handle).unwrap(),
                    *baseline.storage().side_tables(&published).unwrap()
                );
                assert_eq!(
                    matches!(actual.placement(), StoragePlacement::Blob { .. }),
                    open
                );
                let refs = run
                    .phase_results
                    .values()
                    .flatten()
                    .flat_map(|row| &row.output_refs)
                    .collect::<Vec<_>>();
                assert_eq!(refs.len(), 12);
                let parent = |kind: &str| {
                    *refs
                        .iter()
                        .find(|output| output.output_kind() == &OutputKind::new(kind))
                        .unwrap()
                };
                let parents = [
                    parent("SourceTheoremCore"),
                    parent("SourceTheoremVC"),
                    parent("SourceTheoremDischarge"),
                ];
                assert_eq!(actual.lineage().parents.len(), 3);
                for parent in parents {
                    assert!(actual.lineage().parents.contains(&parent.output()));
                }
                assert_eq!(
                    actual.lineage().named_input_hashes,
                    parents[0].lineage().named_input_hashes
                );
                let query = PhaseInput::new(
                    atp.unit.clone(),
                    PhaseDispatchInputBundle::new(
                        old.snapshot,
                        source_key(&actual_inputs.snapshot.source_versions[0]),
                        vec![],
                        parents
                            .into_iter()
                            .map(|parent| {
                                SealedParentOutputHandle::from_published_output(
                                    &owner,
                                    old.snapshot,
                                    parent.clone(),
                                )
                                .unwrap()
                            })
                            .collect(),
                    )
                    .unwrap(),
                );
                let mut hashes = parents.map(|parent| parent.output().hash());
                hashes.sort_by(|a, b| a.as_bytes().cmp(b.as_bytes()));
                assert_eq!(query.identities().parent_output_hashes(), hashes);
                let cache = driver
                    .registry()
                    .cache_key_for_phase(PipelinePhase::AtpSolve, &query)
                    .unwrap();
                assert_eq!(
                    cache.intent,
                    PhaseCacheIntent::NoKey {
                        reason: "source theorem ATP solve has no complete dependency footprint"
                            .to_owned()
                    }
                );
                let probe = driver
                    .registry()
                    .execute_phase_with_resources(
                        PipelinePhase::AtpSolve,
                        query,
                        PhaseExecutionResources::default(),
                    )
                    .unwrap();
                assert_eq!(probe.result.status, PhaseStatus::Blocking);
                assert!(probe.result.output_refs.is_empty());
                assert_eq!(row.cache_observation, Some(probe.observation));
                for kind in [
                    TaskKind::BackendRun,
                    TaskKind::KernelCheck,
                    TaskKind::ArtifactCommit,
                ] {
                    let task = graph.tasks().iter().find(|task| task.kind == kind).unwrap();
                    assert_eq!(
                        run.task_states
                            .iter()
                            .find(|row| row.task_id == task.id)
                            .unwrap()
                            .state,
                        TaskState::Blocked
                    );
                    assert!(!run.phase_results.contains_key(&task.id));
                }
                assert_eq!(
                    submission.session.state(),
                    BuildSessionState::Finished(BuildSessionOutcome::Blocked)
                );
                assert_eq!(
                    core.source_id(),
                    actual_inputs.snapshot.source_versions[0].source_id
                );
                assert_eq!(vcs.source(), core.source_id());
                assert_eq!(vcs.snapshot(), old.snapshot);
                assert!(
                    core.proofs()
                        .iter()
                        .all(|(_, row)| row.status == CoreProofStatus::PendingAutomaticProof)
                );
                for row in vcs.seed_accounting() {
                    let SeedOriginRef::ExistingCore { seed } = row.origin else {
                        panic!()
                    };
                    assert_eq!(row.seed_status, ObligationSeedStatus::Active);
                    assert_eq!(
                        core.obligation_seeds().get(seed).unwrap().status,
                        ObligationSeedStatus::Active
                    );
                }
                assert!(vcs.vcs().iter().all(|row| row.status == VcStatus::Open));
                assert!(
                    discharge
                        .vc_set()
                        .vcs()
                        .iter()
                        .all(|row| row.status == VcStatus::NeedsAtp)
                );
                assert_eq!(expected.0.concrete_format(), format);
                assert_eq!(expected.0.logic_profile_name(), profile.name().as_str());
                assert_eq!((policy, profile, session.captured().clone()), before);
                assert_eq!(*baseline.storage().get(&published).unwrap(), expected);
                for (publisher, count) in [(&baseline, 12), (&owner, 12)] {
                    let collected = publisher.storage().collect(CollectInput {
                        snapshot: old.snapshot,
                        protected_outputs: vec![],
                    });
                    assert_eq!(collected.outputs_dropped, count);
                    assert_eq!(collected.abandoned_slots_dropped, 0);
                }
            },
        );
    }
}

#[test]
fn scheduled_theorem_atp_solve_preserves_resource_refusals() {
    use mizar_atp::{
        backend::{BackendCandidateEvidence, EncodedBackendProblem},
        problem::{
            ConcreteFormat::{SmtLib, Tptp},
            EqualitySupport::{Supported, Unsupported},
            LogicFragment::{Fof, TffLike},
            LogicProfile,
            NativePropertySupport::{
                Supported as NativeSupported, Unsupported as NativeUnsupported,
            },
            QuantifierPolicy::{FirstOrder, PropositionalOnly},
            SoftTypeStrategy::{GuardPredicates, ScopedGuardPredicates},
        },
    };
    use mizar_build::task_graph::{VcOrderKey, VcTaskDescriptor, VcTaskDescriptorId};
    use mizar_ir::storage::CollectInput;
    use mizar_vc::{
        discharge::{ComputationLimit, DischargePolicy},
        vc_ir::{GenerationSchemaVersion, PolicyKey, VcSchemaVersion, VcSet},
    };
    use std::sync::Arc;
    let text = "theorem T: for X being set holds X = X proof let X be set; thus X = X; end;";
    with_native_summary_parent(
        text,
        false,
        0,
        |fixture, _, old, _, _, (session, ids, snapshots)| {
            let generation = GenerationSchemaVersion::new("λ\n\0");
            let schema = VcSchemaVersion::new("");
            let policy = DischargePolicy {
                computation_limit: ComputationLimit {
                    policy: PolicyKey::new("α\n\0"),
                    max_steps: 0,
                },
            };
            let profile = LogicProfile::try_new(
                "caller λ\n\0",
                Fof,
                Supported,
                FirstOrder,
                ScopedGuardPredicates,
                NativeSupported,
                [Tptp, SmtLib].into_iter().collect(),
            )
            .unwrap();
            let WorkUnit::Module { module } = &old.work_unit else {
                panic!()
            };
            let unit = IrWorkUnit::new("\"alpha\":\"main\"");
            let prerequisites = |owner: Arc<PhaseOutputPublisher>| {
                allow_frontend(&owner, &old.work_unit);
                for (phase, kind) in [
                    ("ModuleResolve", "SourceTheoremModuleCandidates"),
                    ("SignatureCollection", "SourceTheoremModuleSummary"),
                    ("TypeChecking", "SourceTheoremTypedAst"),
                    ("RegistrationResolution", "SourceTheoremRegistrationClosure"),
                    ("OverloadResolution", "SourceTheoremMetadata"),
                    ("Elaboration", "SourceTheoremCore"),
                    ("AlgorithmPreparation", "SourceTheoremPreparation"),
                    ("VcGenerate", "SourceTheoremVC"),
                    ("VcDischarge", "SourceTheoremDischarge"),
                ] {
                    owner.allow_work_unit(AllowedWorkUnit::new(
                        IrPipelinePhase::new(phase),
                        OutputKind::new(kind),
                        unit.clone(),
                    ));
                }
                let mut builder = PhaseRegistryBuilder::new();
                builder
                    .register_source_load()
                    .register_frontend(vec![])
                    .register_source_theorem_module_resolver()
                    .register_source_theorem_signature_collection()
                    .register_source_theorem_type_checking()
                    .register_source_theorem_registration_resolution()
                    .register_source_theorem_overload_resolution()
                    .register_source_theorem_elaboration()
                    .register_source_theorem_algorithm_preparation()
                    .register_source_theorem_vc_generation(generation.clone(), schema.clone())
                    .register_source_theorem_vc_discharge(policy.clone());
                let (_, mut submit) = fixture.submit_request(vec![], vec![]);
                submit.vc_descriptors = vec![VcTaskDescriptor::new(
                    VcTaskDescriptorId::new("opaque-prerequisites"),
                    module.clone(),
                    VcOrderKey::new("order"),
                    vec![],
                    vec![],
                )];
                CompilerDriver::new(builder.build().unwrap())
                    .with_output_publisher(owner)
                    .submit_captured(session.clone(), ids, snapshots, submit)
                    .unwrap()
            };
            let outputs = |submission: &mizar_driver::driver::BuildSubmission| {
                submission
                    .scheduler_run
                    .as_ref()
                    .unwrap()
                    .phase_results
                    .values()
                    .flatten()
                    .flat_map(|row| row.output_refs.iter().cloned())
                    .collect::<Vec<_>>()
            };
            let input = |work_unit, owner: &PhaseOutputPublisher, parents: &[AnyPhaseOutputRef]| {
                PhaseInput::new(
                    work_unit,
                    PhaseDispatchInputBundle::new(
                        old.snapshot,
                        source_key(&session.captured().snapshot().source_versions[0]),
                        vec![],
                        parents
                            .iter()
                            .filter(|parent| {
                                matches!(
                                    parent.output_kind().as_str(),
                                    "SourceTheoremCore"
                                        | "SourceTheoremVC"
                                        | "SourceTheoremDischarge"
                                )
                            })
                            .map(|parent| {
                                SealedParentOutputHandle::from_published_output(
                                    owner,
                                    old.snapshot,
                                    parent.clone(),
                                )
                                .unwrap()
                            })
                            .collect(),
                    )
                    .unwrap(),
                )
            };
            let baseline = publisher(old.snapshot, 0, &old.work_unit);
            baseline.allow_work_unit(AllowedWorkUnit::new(
                IrPipelinePhase::new("AtpSolve"),
                OutputKind::new("SourceTheoremCandidateInputs"),
                unit.clone(),
            ));
            let baseline_submission = prerequisites(baseline.clone());
            let (baseline_inputs, _) = native_disk_lookup_context(&baseline_submission, ids);
            let refs = outputs(&baseline_submission);
            assert_eq!(refs.len(), 11);
            let module_input = input(old.work_unit.clone(), &baseline, &refs);
            let vc_ref = refs
                .iter()
                .find(|parent| parent.output_kind() == &OutputKind::new("SourceTheoremVC"))
                .unwrap();
            let vcs = baseline
                .storage()
                .get(
                    &baseline
                        .storage()
                        .typed_handle::<VcSet>(vc_ref, vc_ref.output_kind())
                        .unwrap(),
                )
                .unwrap();
            let selected = vcs.vcs()[0].id;
            let expected = baseline_inputs
                .native_theorem_candidate_with_core_vc_discharge_parents(
                    &module_input,
                    &baseline,
                    selected,
                    &policy,
                    profile.clone(),
                    Tptp,
                )
                .unwrap()
                .unwrap();
            let publication = baseline_inputs
                .publish_native_theorem_candidate(
                    &module_input,
                    &baseline,
                    selected,
                    &policy,
                    profile.clone(),
                    Tptp,
                )
                .unwrap()
                .unwrap();
            assert_eq!(*baseline.storage().get(&publication).unwrap(), expected);
            for case in [
                "cancel",
                "foreign-token",
                "no-publisher",
                "no-inputs",
                "source-snapshot",
                "module",
                "wrong-module",
                "missing",
                "foreign-publisher",
                "obsolete",
                "collected",
                "denied",
            ] {
                let owner = publisher(old.snapshot, 0, &old.work_unit);
                if case != "denied" {
                    owner.allow_work_unit(AllowedWorkUnit::new(
                        IrPipelinePhase::new("AtpSolve"),
                        OutputKind::new("SourceTheoremCandidateInputs"),
                        unit.clone(),
                    ));
                }
                let submission = prerequisites(owner.clone());
                let (inputs, _) = native_disk_lookup_context(&submission, ids);
                let mut refs = outputs(&submission);
                assert_eq!(refs.len(), 11);
                if case == "missing" {
                    refs.retain(|parent| {
                        parent.output_kind() != &OutputKind::new("SourceTheoremDischarge")
                    });
                }
                let work_unit = if case == "module" {
                    old.work_unit.clone()
                } else {
                    WorkUnit::Vc {
                        module: mizar_build::module_index::ModuleId::new(
                            PackageId::new("alpha"),
                            ModulePath::new(if case == "wrong-module" {
                                "other"
                            } else {
                                "main"
                            }),
                        ),
                        descriptor: VcTaskDescriptorId::new("opaque-resource-query"),
                    }
                };
                let query = input(work_unit, &owner, &refs);
                let foreign = (case == "source-snapshot")
                    .then(|| native_disk_frontend_record_submission(fixture, ids, snapshots, 0));
                let foreign_inputs = foreign
                    .as_ref()
                    .map(|(_, submission, _, _, _)| native_disk_lookup_context(submission, ids).0);
                let alien = publisher(old.snapshot, 0, &old.work_unit);
                if case == "obsolete" {
                    owner.mark_obsolete(old.snapshot).unwrap();
                }
                if case == "collected" {
                    assert_eq!(
                        owner
                            .storage()
                            .collect(CollectInput {
                                snapshot: old.snapshot,
                                protected_outputs: vec![]
                            })
                            .outputs_dropped,
                        11
                    );
                }
                let mut builder = PhaseRegistryBuilder::new();
                builder
                    .register_source_theorem_atp_solve(policy.clone(), profile.clone(), Tptp)
                    .unwrap();
                let registry = builder.build().unwrap();
                let result = registry
                    .execute_phase_with_resources(
                        PipelinePhase::AtpSolve,
                        query,
                        PhaseExecutionResources {
                            cancellation: matches!(case, "cancel" | "foreign-token").then_some(
                                CancellationToken {
                                    snapshot: if case == "foreign-token" {
                                        snapshot_id(0xd5)
                                    } else {
                                        old.snapshot
                                    },
                                    generation: CancellationGeneration::new(1),
                                    reason: CancellationReason::ExplicitRequest,
                                },
                            ),
                            output_publisher: if case == "no-publisher" {
                                None
                            } else {
                                Some(if case == "foreign-publisher" {
                                    alien
                                } else {
                                    owner.clone()
                                })
                            },
                            source_load: if case == "no-inputs" {
                                None
                            } else {
                                Some(foreign_inputs.unwrap_or(inputs))
                            },
                            ..PhaseExecutionResources::default()
                        },
                    )
                    .unwrap();
                assert_eq!(
                    result.result.status,
                    if case == "cancel" {
                        PhaseStatus::Cancelled
                    } else {
                        PhaseStatus::Blocking
                    },
                    "{case}"
                );
                assert!(
                    result.result.output_refs.is_empty() && result.result.diagnostics.is_empty()
                );
                assert_eq!(result.result.cache_observation, Some(result.observation));
                let collected = owner.storage().collect(CollectInput {
                    snapshot: old.snapshot,
                    protected_outputs: vec![],
                });
                assert_eq!(
                    collected.outputs_dropped,
                    if case == "collected" { 0 } else { 11 }
                );
                assert_eq!(
                    collected.abandoned_slots_dropped,
                    usize::from(case == "denied")
                );
            }
            let mut identities = Vec::new();
            for case in 0..11 {
                let owner = publisher(old.snapshot, 0, &old.work_unit);
                owner.allow_work_unit(AllowedWorkUnit::new(
                    IrPipelinePhase::new("AtpSolve"),
                    OutputKind::new("SourceTheoremCandidateInputs"),
                    unit.clone(),
                ));
                let submission = prerequisites(owner.clone());
                let (inputs, _) = native_disk_lookup_context(&submission, ids);
                let refs = outputs(&submission);
                let key = if case == 1 { "changed" } else { "α\n\0" };
                let max_steps = if case == 2 { u32::MAX } else { 0 };
                let name = if case == 3 {
                    "changed"
                } else {
                    "caller λ\n\0"
                };
                let policy = DischargePolicy {
                    computation_limit: ComputationLimit {
                        policy: PolicyKey::new(key),
                        max_steps,
                    },
                };
                let tags = [
                    u8::from(case == 4),
                    u8::from(case != 5),
                    u8::from(case != 6),
                    if case == 7 { 1 } else { 3 },
                    u8::from(case != 8),
                    if case == 9 { 1 } else { 3 },
                    u8::from(case == 10),
                ];
                let profile = LogicProfile::try_new(
                    name,
                    [Fof, TffLike][usize::from(case == 4)],
                    [Supported, Unsupported][usize::from(case == 5)],
                    [FirstOrder, PropositionalOnly][usize::from(case == 6)],
                    [ScopedGuardPredicates, GuardPredicates][usize::from(case == 7)],
                    [NativeSupported, NativeUnsupported][usize::from(case == 8)],
                    [Tptp, SmtLib]
                        .into_iter()
                        .filter(|format| case != 9 || format == &Tptp)
                        .collect(),
                )
                .unwrap();
                let format = [Tptp, SmtLib][usize::from(case == 10)];
                let module_input = input(old.work_unit.clone(), &owner, &refs);
                let expected = inputs.native_theorem_candidate_with_core_vc_discharge_parents(
                    &module_input,
                    &owner,
                    selected,
                    &policy,
                    profile.clone(),
                    format,
                );
                let mut builder = PhaseRegistryBuilder::new();
                builder
                    .register_source_theorem_atp_solve(policy, profile, format)
                    .unwrap();
                let registry = builder.build().unwrap();
                let descriptor = registry
                    .descriptor_for_phase(PipelinePhase::AtpSolve)
                    .unwrap();
                let [f, e, q, s, n, a, c] = tags;
                let key_len = key.len();
                let name_len = name.len();
                let schema = format!(
                    "mizar-atp/source-theorem-candidate-publication/v1;computation_policy={key_len}:{key};max_steps={max_steps};profile={name_len}:{name};fragment={f};equality={e};quantifiers={q};soft_types={s};native_properties={n};formats={a};format={c}"
                );
                assert_eq!(descriptor.schema_version, schema);
                let query = input(
                    WorkUnit::Vc {
                        module: module.clone(),
                        descriptor: VcTaskDescriptorId::new("opaque-config-query"),
                    },
                    &owner,
                    &refs,
                );
                let cache = registry
                    .cache_key_for_phase(PipelinePhase::AtpSolve, &query)
                    .unwrap();
                assert!(matches!(cache.intent, PhaseCacheIntent::NoKey { .. }));
                assert!(
                    identities
                        .iter()
                        .all(|(signature, observation)| signature != &schema
                            && observation != &cache.observation)
                );
                identities.push((schema, cache.observation));
                let result = registry
                    .execute_phase_with_resources(
                        PipelinePhase::AtpSolve,
                        query,
                        PhaseExecutionResources {
                            output_publisher: Some(owner.clone()),
                            source_load: Some(inputs),
                            ..PhaseExecutionResources::default()
                        },
                    )
                    .unwrap();
                let complete = match expected {
                    Ok(Some(expected)) => {
                        assert_eq!(result.result.status, PhaseStatus::Complete);
                        let [output] = result.result.output_refs.as_slice() else {
                            panic!()
                        };
                        let typed = owner
                            .storage()
                            .typed_handle::<(EncodedBackendProblem, BackendCandidateEvidence)>(
                                output,
                                output.output_kind(),
                            )
                            .unwrap();
                        assert_eq!(*owner.storage().get(&typed).unwrap(), expected);
                        true
                    }
                    Err(_) => {
                        assert_eq!(result.result.status, PhaseStatus::Blocking);
                        assert!(result.result.output_refs.is_empty());
                        false
                    }
                    Ok(None) => panic!("genuine source unexpectedly withheld candidate"),
                };
                assert!(result.result.diagnostics.is_empty());
                let collected = owner.storage().collect(CollectInput {
                    snapshot: old.snapshot,
                    protected_outputs: vec![],
                });
                assert_eq!(collected.outputs_dropped, 11 + usize::from(complete));
                assert_eq!(collected.abandoned_slots_dropped, 0);
            }
            let collected = baseline.storage().collect(CollectInput {
                snapshot: old.snapshot,
                protected_outputs: vec![],
            });
            assert_eq!(collected.outputs_dropped, 12);
            assert_eq!(collected.abandoned_slots_dropped, 0);
        },
    );
}
