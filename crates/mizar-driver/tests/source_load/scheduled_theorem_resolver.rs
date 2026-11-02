use super::*;

#[test]
fn scheduled_theorem_resolver_matches_genuine_publication_and_worker_replay() {
    let private = include_str!(
        "../../../../tests/miz/pass/resolve/pass_declaration_symbol_private_theorem_visibility_001.miz"
    );
    let lemma = include_str!(
        "../../../../tests/miz/pass/theorems/pass_proof_verification_lemma_reference_001.miz"
    );
    let open_assumed = include_str!(
        "../../../../tests/miz/pass/theorems/pass_proof_verification_theorem_status_open_assumed_001.miz"
    );
    for (case, (text, open, threshold)) in [
        (private, false, usize::MAX),
        (private, true, 0),
        (lemma, false, 0),
        (open_assumed, true, usize::MAX),
    ]
    .into_iter()
    .enumerate()
    {
        let fixture = Fixture::new(text.as_bytes());
        let ids = InMemorySessionIdAllocator::new();
        let snapshots = SnapshotRegistry::new();
        let (mut draft, submit) = fixture.submit_request(vec![], vec![]);
        draft.source_inputs.versions.clear();
        let descriptor = SourceInput {
            package_id: fixture.version.package_id.clone(),
            module_path: fixture.version.module_path.clone(),
            normalized_path: fixture.version.normalized_path.clone(),
            edition: fixture.version.edition.clone(),
            origin: if open {
                SourceOriginInput::OpenBuffer {
                    uri: format!(
                        "file://{}",
                        fixture.root.join("alpha/src/main.miz").display()
                    ),
                    expected_version: 7,
                    actual_version: 7,
                    text: std::sync::Arc::from(text),
                }
            } else {
                SourceOriginInput::Disk {
                    path: "src/main.miz".into(),
                }
            },
        };
        let session = draft
            .capture_sources(
                &ids,
                &snapshots,
                vec![(
                    DiskSourceLoader::new(fixture.root.join("alpha")),
                    descriptor,
                )],
            )
            .unwrap();
        let captured = session.captured().clone();
        let snapshot = captured.snapshot().id;
        let unit = WorkUnit::Module {
            module: mizar_build::module_index::ModuleId::new(
                PackageId::new("alpha"),
                ModulePath::new("main"),
            ),
        };
        let baseline_owner = publisher(snapshot, threshold, &unit);
        allow_frontend(&baseline_owner, &unit);
        baseline_owner.allow_work_unit(AllowedWorkUnit::new(
            IrPipelinePhase::new("ModuleResolve"),
            OutputKind::new("SourceTheoremModuleCandidates"),
            IrWorkUnit::new("\"alpha\":\"main\""),
        ));
        let mut builder = PhaseRegistryBuilder::new();
        builder.register_source_load().register_frontend(vec![]);
        let baseline = CompilerDriver::new(builder.build().unwrap())
            .with_output_publisher(baseline_owner.clone())
            .submit_captured(session.clone(), &ids, &snapshots, submit)
            .unwrap();
        let (inputs, source_input) = native_disk_lookup_context(&baseline, &ids);
        let parent = baseline
            .scheduler_run
            .as_ref()
            .unwrap()
            .phase_results
            .values()
            .flatten()
            .flat_map(|result| &result.output_refs)
            .find(|parent| parent.output_kind() == &OutputKind::new("FrontendOutput"))
            .unwrap();
        let phase_input = PhaseInput::new(
            unit.clone(),
            PhaseDispatchInputBundle::new(
                snapshot,
                source_input.identities().input_hash(),
                vec![],
                vec![
                    SealedParentOutputHandle::from_published_output(
                        &baseline_owner,
                        snapshot,
                        parent.clone(),
                    )
                    .unwrap(),
                ],
            )
            .unwrap(),
        );
        let expected = inputs
            .publish_native_theorem_module_candidates(&phase_input, &baseline_owner)
            .unwrap()
            .unwrap();
        let expected_pair = baseline_owner.storage().get(&expected).unwrap();
        assert_eq!(
            expected_pair.0.source_id(),
            captured.snapshot().source_versions[0].source_id
        );
        assert_eq!(
            expected_pair.0.module_id(),
            &ResolverModuleId::new(PackageId::new("alpha"), ModulePath::new("main"))
        );
        let mut replay = None;
        for &workers in if case == 0 { &[1, 4][..] } else { &[1][..] } {
            if open {
                fixture.write(&[0xff]);
            }
            let (_, mut submit) = fixture.submit_request(vec![], vec![]);
            submit.worker_count = workers;
            submit.completion_order = if workers == 1 {
                CompletionOrder::Canonical
            } else {
                CompletionOrder::Reverse
            };
            let owner = publisher(snapshot, threshold, &unit);
            allow_frontend(&owner, &unit);
            owner.allow_work_unit(AllowedWorkUnit::new(
                IrPipelinePhase::new("ModuleResolve"),
                OutputKind::new("SourceTheoremModuleCandidates"),
                IrWorkUnit::new("\"alpha\":\"main\""),
            ));
            let mut builder = PhaseRegistryBuilder::new();
            builder
                .register_source_load()
                .register_frontend(vec![])
                .register_source_theorem_module_resolver();
            let mut driver =
                CompilerDriver::new(builder.build().unwrap()).with_output_publisher(owner.clone());
            let submission = driver
                .submit_captured(session.clone(), &ids, &snapshots, submit)
                .unwrap();
            assert_eq!(
                submission.status,
                DriverSubmissionStatus::BlockedByMissingPhaseServices
            );
            assert_eq!(
                submission.session.state(),
                BuildSessionState::Finished(BuildSessionOutcome::Blocked)
            );
            assert!(
                submission
                    .missing_services
                    .iter()
                    .any(|gap| gap.phase == PipelinePhase::SignatureCollection)
            );
            assert!(
                !submission
                    .missing_services
                    .iter()
                    .any(|gap| gap.phase == PipelinePhase::ModuleResolve)
            );
            let task = submission
                .task_graph
                .as_ref()
                .unwrap()
                .tasks()
                .iter()
                .find(|task| task.kind == TaskKind::ModuleResolve && task.unit == unit)
                .unwrap();
            let run = submission.scheduler_run.as_ref().unwrap();
            assert_eq!(
                run.task_states
                    .iter()
                    .find(|record| record.task_id == task.id)
                    .unwrap()
                    .state,
                TaskState::Completed
            );
            let [result] = run.phase_results[&task.id].as_slice() else {
                panic!("one resolver result")
            };
            assert_eq!(result.status, PhaseStatus::Complete);
            assert!(result.diagnostics.is_empty());
            let [actual] = result.output_refs.as_slice() else {
                panic!("one genuine resolver output")
            };
            assert_eq!(actual, &expected.erase());
            assert_eq!(
                actual.placement() == &StoragePlacement::Resident,
                threshold != 0
            );
            let typed = owner
                .storage()
                .typed_handle::<(
                    mizar_resolve::resolved_ast::ResolvedAst,
                    mizar_resolve::env::SymbolEnv,
                )>(actual, &OutputKind::new("SourceTheoremModuleCandidates"))
                .unwrap();
            assert_eq!(*owner.storage().get(&typed).unwrap(), *expected_pair);
            assert_eq!(
                owner.storage().side_tables(&typed).unwrap(),
                baseline_owner.storage().side_tables(&expected).unwrap()
            );
            let cache = driver
                .registry()
                .cache_key_for_phase(PipelinePhase::ModuleResolve, &phase_input)
                .unwrap();
            assert!(
                matches!(cache.intent, PhaseCacheIntent::NoKey { reason } if !reason.is_empty())
            );
            assert!(result.cache_observation.is_some());
            if let Some(previous) = &replay {
                assert_eq!(result, previous);
            }
            replay = Some(result.clone());
            assert_eq!(submission.session.captured(), &captured);
        }
        assert_eq!(session.captured(), &captured);
    }
}

#[test]
fn scheduled_theorem_resolver_preserves_genuine_refusals() {
    let private = include_str!(
        "../../../../tests/miz/pass/resolve/pass_declaration_symbol_private_theorem_visibility_001.miz"
    );
    for (text, error) in [
        (format!("{private}\nreserve Y for set;\n"), false),
        (private.replace("by PrivT1", "by Missing"), false),
        (private.replace("thus X = X;", "thus Y = Y;"), true),
    ] {
        let fixture = Fixture::new(text.as_bytes());
        let ids = InMemorySessionIdAllocator::new();
        let snapshots = SnapshotRegistry::new();
        let (_, baseline, adapter, parent, _) =
            native_disk_frontend_record_submission(&fixture, &ids, &snapshots, 0);
        let owner = adapter.publisher();
        let (inputs, source_input) = native_disk_lookup_context(&baseline, &ids);
        let snapshot = source_input.snapshot;
        owner.allow_work_unit(AllowedWorkUnit::new(
            IrPipelinePhase::new("ModuleResolve"),
            OutputKind::new("SourceTheoremModuleCandidates"),
            parent.work_unit().clone(),
        ));
        let input = PhaseInput::new(
            source_input.work_unit.clone(),
            PhaseDispatchInputBundle::new(
                snapshot,
                source_input.identities().input_hash(),
                vec![],
                vec![
                    SealedParentOutputHandle::from_published_output(
                        owner,
                        snapshot,
                        parent.erase(),
                    )
                    .unwrap(),
                ],
            )
            .unwrap(),
        );
        let old = inputs.publish_native_theorem_module_candidates(&input, owner);
        if error {
            assert!(old.is_err());
        } else {
            assert!(matches!(old, Ok(None)));
        }
        let mut builder = PhaseRegistryBuilder::new();
        builder.register_source_theorem_module_resolver();
        let registry = builder.build().unwrap();
        let result = registry
            .execute_phase_with_resources(
                PipelinePhase::ModuleResolve,
                input,
                PhaseExecutionResources {
                    output_publisher: Some(owner.clone()),
                    source_load: Some(inputs),
                    ..PhaseExecutionResources::default()
                },
            )
            .unwrap()
            .result;
        assert_eq!(result.status, PhaseStatus::Blocking);
        assert!(result.output_refs.is_empty());
        assert!(result.diagnostics.is_empty());
        assert!(result.cache_observation.is_some());
        let collected = owner.storage().collect(mizar_ir::storage::CollectInput {
            snapshot,
            protected_outputs: vec![],
        });
        assert_eq!(
            collected.outputs_dropped,
            baseline
                .scheduler_run
                .as_ref()
                .unwrap()
                .phase_results
                .values()
                .flatten()
                .map(|result| result.output_refs.len())
                .sum::<usize>()
        );
        assert_eq!(collected.abandoned_slots_dropped, 0);
    }
}

#[test]
fn scheduled_theorem_resolver_requires_current_resources_and_honors_cancellation() {
    let text = include_str!(
        "../../../../tests/miz/pass/theorems/pass_proof_verification_lemma_reference_001.miz"
    );
    for case in [
        "valid",
        "cancel",
        "foreign-token",
        "no-publisher",
        "no-inputs",
        "no-capture",
        "foreign-capture",
        "no-parent",
        "collected-parent",
        "obsolete",
        "denied",
    ] {
        let fixture = Fixture::new(text.as_bytes());
        let ids = InMemorySessionIdAllocator::new();
        let snapshots = SnapshotRegistry::new();
        let (_, baseline, adapter, parent, _) =
            native_disk_frontend_record_submission(&fixture, &ids, &snapshots, 0);
        let owner = adapter.publisher();
        let (mut inputs, source_input) = native_disk_lookup_context(&baseline, &ids);
        let snapshot = source_input.snapshot;
        if case != "denied" {
            owner.allow_work_unit(AllowedWorkUnit::new(
                IrPipelinePhase::new("ModuleResolve"),
                OutputKind::new("SourceTheoremModuleCandidates"),
                parent.work_unit().clone(),
            ));
        }
        let mut parents = vec![
            SealedParentOutputHandle::from_published_output(owner, snapshot, parent.erase())
                .unwrap(),
        ];
        let foreign = fixture.submit(&ids, &snapshots);
        if case == "no-capture" {
            inputs.captured = None;
        }
        if case == "foreign-capture" {
            inputs.captured = Some(foreign.session.captured());
        }
        if case == "no-parent" {
            parents.clear();
        }
        let input = PhaseInput::new(
            source_input.work_unit.clone(),
            PhaseDispatchInputBundle::new(
                snapshot,
                source_input.identities().input_hash(),
                vec![],
                parents,
            )
            .unwrap(),
        );
        if case == "collected-parent" {
            owner.storage().collect(mizar_ir::storage::CollectInput {
                snapshot,
                protected_outputs: vec![],
            });
        }
        if case == "obsolete" {
            owner.mark_obsolete(snapshot).unwrap();
        }
        let old = inputs.native_theorem_module_candidates_with_frontend_parent(&input, owner);
        if matches!(
            case,
            "no-capture" | "foreign-capture" | "no-parent" | "collected-parent" | "obsolete"
        ) {
            assert!(old.is_err(), "{case}");
        } else {
            assert!(old.unwrap().is_some(), "{case}");
        }
        let mut builder = PhaseRegistryBuilder::new();
        builder.register_source_theorem_module_resolver();
        let result = builder
            .build()
            .unwrap()
            .execute_phase_with_resources(
                PipelinePhase::ModuleResolve,
                input,
                PhaseExecutionResources {
                    cancellation: matches!(case, "cancel" | "foreign-token").then_some(
                        CancellationToken {
                            snapshot: if case == "foreign-token" {
                                snapshot_id(0xf4)
                            } else {
                                snapshot
                            },
                            generation: CancellationGeneration::new(1),
                            reason: CancellationReason::ExplicitRequest,
                        },
                    ),
                    output_publisher: (case != "no-publisher").then(|| owner.clone()),
                    source_load: (case != "no-inputs").then_some(inputs),
                    ..PhaseExecutionResources::default()
                },
            )
            .unwrap()
            .result;
        assert_eq!(
            result.status,
            match case {
                "valid" => PhaseStatus::Complete,
                "cancel" => PhaseStatus::Cancelled,
                _ => PhaseStatus::Blocking,
            },
            "{case}"
        );
        assert!(result.diagnostics.is_empty());
        if case == "valid" {
            let [actual] = result.output_refs.as_slice() else {
                panic!("one producer output")
            };
            assert_eq!(actual.phase(), &IrPipelinePhase::new("ModuleResolve"));
            owner.validate_published_output(snapshot, actual).unwrap();
            owner.storage().validate_handle(actual).unwrap();
        } else {
            assert!(result.output_refs.is_empty(), "{case}");
        }
    }
}

#[test]
fn scheduled_theorem_resolver_edits_match_independent_clean_modules() {
    use mizar_artifact::{
        manifest::{
            ManifestCommitOptions, ManifestFileReadOptions, ManifestTransaction,
            artifact_manifest_json, manifest_hash_domain, read_manifest_file,
        },
        store::{
            PublishedArtifactReadOptions, artifact_hash_domain, canonical_json_bytes,
            read_published_artifact,
        },
        verified_artifact::{
            VERIFIED_ARTIFACT_SCHEMA_FAMILY, VerifiedArtifact, VerifiedArtifactReadOptions,
            artifact_hash_excluded_paths, read_verified_artifact,
        },
    };
    use mizar_atp::{
        backend::{BackendCandidateEvidence, EncodedBackendProblem},
        problem::{
            ConcreteFormat, EqualitySupport, LogicFragment, LogicProfile, NativePropertySupport,
            QuantifierPolicy, SoftTypeStrategy,
        },
    };
    use mizar_build::planner::{lockfile_content_hash, workspace_config_content_hash};
    use mizar_build::task_graph::{
        BackendProfileId, EvidenceCandidateId, VcOrderKey, VcTaskDescriptor, VcTaskDescriptorId,
    };
    use mizar_proof::policy::VerifierPolicy;
    use mizar_vc::{
        discharge::{
            ComputationLimit, DischargeExplanationCategory, DischargeInput, DischargeOutput,
            DischargePolicy, try_discharge,
        },
        vc_ir::{PolicyKey, VcProvenancePhase, VcStatus},
    };
    let verifier = VerifierPolicy::default();
    let policy = DischargePolicy {
        computation_limit: ComputationLimit {
            policy: PolicyKey::new("step6c12-explicit-limit"),
            max_steps: 64,
        },
    };
    let profile = LogicProfile::try_new(
        "step6c99-explicit-atp",
        LogicFragment::Fof,
        EqualitySupport::Supported,
        QuantifierPolicy::FirstOrder,
        SoftTypeStrategy::ScopedGuardPredicates,
        NativePropertySupport::Unsupported,
        [ConcreteFormat::Tptp].into_iter().collect(),
    )
    .unwrap();
    let private = include_str!(
        "../../../../tests/miz/pass/resolve/pass_declaration_symbol_private_theorem_visibility_001.miz"
    );
    let lemma = include_str!(
        "../../../../tests/miz/pass/theorems/pass_proof_verification_lemma_reference_001.miz"
    );
    let renamed = lemma.replace("Lem1", "Lem2");
    let theorem = "theorem T: for X being set holds X = X proof let X be set; thus X = X; end;";
    let renamed_theorem = theorem.replace("theorem T:", "theorem U:");
    let private_theorem = format!("private {theorem}");
    let public_theorem = format!("public {theorem}");
    let generation_schema = mizar_vc::vc_ir::GenerationSchemaVersion::new("step6c10-generation");
    let vc_schema = mizar_vc::vc_ir::VcSchemaVersion::new("step6c10-vc");
    let fixture = Fixture::new(private.as_bytes());
    fs::write(fixture.root.join("alpha/src/leaf.miz"), private).unwrap();
    let ids = InMemorySessionIdAllocator::new();
    let snapshots = SnapshotRegistry::new();
    for parent in ["same-artifact-actual", "same-artifact-clean"] {
        fs::create_dir(fixture.root.join("alpha").join(parent)).unwrap();
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
        .register_source_theorem_vc_generation(generation_schema.clone(), vc_schema.clone())
        .register_source_theorem_vc_discharge(policy.clone())
        .register_source_theorem_atp_solve(policy.clone(), profile.clone(), ConcreteFormat::Tptp)
        .unwrap()
        .register_source_theorem_candidate_check_with_witness_files(
            policy.clone(),
            profile.clone(),
            ConcreteFormat::Tptp,
            verifier.clone(),
            "proof-witnesses/scheduled".into(),
        )
        .unwrap()
        .register_source_theorem_artifact_commit(
            policy.clone(),
            profile.clone(),
            ConcreteFormat::Tptp,
            verifier.clone(),
            (
                "proof-witnesses/scheduled".into(),
                "main.mizir.json".into(),
                "same-artifact-actual".into(),
            ),
        )
        .unwrap();
    let mut driver = CompilerDriver::new(builder.build().unwrap());
    #[expect(
        clippy::type_complexity,
        reason = "retain genuine Core, preparation, VC, and discharge handles with their source session"
    )]
    let mut previous: Option<(
        std::sync::Arc<PhaseOutputPublisher>,
        Vec<AnyPhaseOutputRef>,
        mizar_driver::request::BuildSession,
        [Hash; 2],
    )> = None;
    for (generation, (backend_free, main)) in [false, true]
        .into_iter()
        .flat_map(|backend_free| {
            [
                private,
                lemma,
                renamed.as_str(),
                theorem,
                renamed_theorem.as_str(),
                private_theorem.as_str(),
                public_theorem.as_str(),
            ]
            .into_iter()
            .map(move |main| (backend_free, main))
        })
        .enumerate()
    {
        let stage = generation % 7;
        fixture.write(main.as_bytes());
        let (mut draft, mut input) = fixture.submit_request(vec![], vec![]);
        draft.source_inputs.versions.clear();
        draft.generation = BuildRequestGeneration::new(generation as u64);
        if backend_free && stage == 6 {
            draft.origin = BuildRequestOrigin::Watch(mizar_driver::request::WatchRequest {
                watch_root: draft.workspace_root.clone(),
                changed_paths: vec![fixture.version.normalized_path.clone()],
            });
        }
        input.source_layout = StaticSourceLayout::new(vec![WorkspaceSourcePackage {
            package_id: PackageId::new("alpha"),
            files: vec![
                WorkspaceSourceFile::new("src/main.miz", "main.miz"),
                WorkspaceSourceFile::new("src/leaf.miz", "leaf.miz"),
            ],
        }]);
        input.dependency_overlay = ModuleDependencyOverlay::complete(Vec::new());
        let plan = produce_build_plan(
            input.plan_request.clone(),
            input.workspace_packages.clone(),
            input.lockfile.clone(),
        )
        .unwrap();
        draft.dependency_inputs.lockfile_hash = lockfile_content_hash(&plan.lockfile);
        draft.verifier_config = VerifierConfigInput::new(workspace_config_content_hash(
            &plan.verifier_config,
            &plan.build_config,
        ));
        let descriptors = ["main", "leaf"].map(|module| {
            let path = format!("src/{module}.miz");
            (
                DiskSourceLoader::new(fixture.root.join("alpha")),
                SourceInput {
                    package_id: PackageId::new("alpha"),
                    module_path: ModulePath::new(module),
                    normalized_path: normalize_path(&fixture.root.join("alpha"), Path::new(&path))
                        .unwrap(),
                    edition: Edition::new("2025"),
                    origin: SourceOriginInput::Disk { path: path.into() },
                },
            )
        });
        let session = draft
            .capture_sources(&ids, &snapshots, descriptors.to_vec())
            .unwrap();
        let captured = session.captured().clone();
        let snapshot = captured.snapshot().id;
        let versions = &captured.snapshot().source_versions;
        let hashes = ["main", "leaf"].map(|module| {
            versions
                .iter()
                .find(|v| v.module_path.as_str() == module)
                .unwrap()
                .source_hash
        });
        let units = ["main", "leaf"].map(|module| WorkUnit::Module {
            module: mizar_build::module_index::ModuleId::new(
                PackageId::new("alpha"),
                ModulePath::new(module),
            ),
        });
        let owner = publisher(snapshot, 0, &units[0]);
        let clean_owner = publisher(snapshot, 0, &units[0]);
        let prepass_owner = publisher(snapshot, 0, &units[0]);
        for current in [&owner, &clean_owner, &prepass_owner] {
            for module in ["main", "leaf"] {
                for (phase, kind) in [
                    ("SourceLoad", "SourceUnit"),
                    ("Frontend", "FrontendOutput"),
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
                    ("KernelCheck", "SourceTheoremCandidateCheck"),
                ] {
                    current.allow_work_unit(AllowedWorkUnit::new(
                        IrPipelinePhase::new(phase),
                        OutputKind::new(kind),
                        IrWorkUnit::new(format!("{:?}:{:?}", "alpha", module)),
                    ));
                }
            }
        }
        for current in [&owner, &clean_owner] {
            for module in ["main", "leaf"] {
                current.allow_work_unit(AllowedWorkUnit::new(
                    IrPipelinePhase::new("KernelCheck"),
                    OutputKind::new("SourceTheoremCandidateWitnessFile"),
                    IrWorkUnit::new(format!("{:?}:{:?}", "alpha", module)),
                ));
                current.allow_work_unit(AllowedWorkUnit::new(
                    IrPipelinePhase::new("ArtifactCommit"),
                    OutputKind::new("SourceTheoremArtifactData"),
                    IrWorkUnit::new(format!("{:?}:{:?}", "alpha", module)),
                ));
            }
        }
        driver = driver.with_output_publisher(owner.clone());
        let mut clean_builder = PhaseRegistryBuilder::new();
        clean_builder
            .register_source_load()
            .register_frontend(vec![])
            .register_source_theorem_module_resolver()
            .register_source_theorem_signature_collection()
            .register_source_theorem_type_checking()
            .register_source_theorem_registration_resolution()
            .register_source_theorem_overload_resolution()
            .register_source_theorem_elaboration()
            .register_source_theorem_algorithm_preparation()
            .register_source_theorem_vc_generation(generation_schema.clone(), vc_schema.clone())
            .register_source_theorem_vc_discharge(policy.clone())
            .register_source_theorem_atp_solve(
                policy.clone(),
                profile.clone(),
                ConcreteFormat::Tptp,
            )
            .unwrap()
            .register_source_theorem_candidate_check_with_witness_files(
                policy.clone(),
                profile.clone(),
                ConcreteFormat::Tptp,
                verifier.clone(),
                "proof-witnesses/scheduled".into(),
            )
            .unwrap()
            .register_source_theorem_artifact_commit(
                policy.clone(),
                profile.clone(),
                ConcreteFormat::Tptp,
                verifier.clone(),
                (
                    "proof-witnesses/scheduled".into(),
                    "main.mizir.json".into(),
                    "same-artifact-clean".into(),
                ),
            )
            .unwrap();
        let mut clean_driver = CompilerDriver::new(clean_builder.build().unwrap())
            .with_output_publisher(clean_owner.clone());
        let (_, mut clean_input) = fixture.submit_request(vec![], vec![]);
        clean_input.source_layout = input.source_layout.clone();
        clean_input.dependency_overlay = ModuleDependencyOverlay::complete(Vec::new());
        clean_input.worker_count = 4;
        clean_input.completion_order = CompletionOrder::Reverse;
        input.worker_count = 1;
        input.completion_order = CompletionOrder::Canonical;
        let descriptor_id = VcTaskDescriptorId::new("step6c12-main");
        input.vc_descriptors = Vec::new();
        let baseline_vcs = if stage >= 3 {
            let mut prepass_builder = PhaseRegistryBuilder::new();
            prepass_builder
                .register_source_load()
                .register_frontend(vec![])
                .register_source_theorem_module_resolver()
                .register_source_theorem_signature_collection()
                .register_source_theorem_type_checking()
                .register_source_theorem_registration_resolution()
                .register_source_theorem_overload_resolution()
                .register_source_theorem_elaboration()
                .register_source_theorem_algorithm_preparation()
                .register_source_theorem_vc_generation(
                    generation_schema.clone(),
                    vc_schema.clone(),
                );
            let (_, mut prepass_input) = fixture.submit_request(vec![], vec![]);
            prepass_input.source_layout = input.source_layout.clone();
            prepass_input.dependency_overlay = input.dependency_overlay.clone();
            let prepass = CompilerDriver::new(prepass_builder.build().unwrap())
                .with_output_publisher(prepass_owner.clone())
                .submit_captured(session.clone(), &ids, &snapshots, prepass_input)
                .unwrap();
            assert_eq!(prepass.session.captured(), &captured);
            let task = prepass
                .task_graph
                .as_ref()
                .unwrap()
                .tasks()
                .iter()
                .find(|task| task.kind == TaskKind::VcGenerate && task.unit == units[0])
                .unwrap();
            let WorkUnit::Module { module } = &task.unit else {
                panic!("actual main module")
            };
            assert_eq!(
                prepass
                    .scheduler_run
                    .as_ref()
                    .unwrap()
                    .task_states
                    .iter()
                    .find(|state| state.task_id == task.id)
                    .unwrap()
                    .state,
                TaskState::Completed
            );
            let [row] = prepass.scheduler_run.as_ref().unwrap().phase_results[&task.id].as_slice()
            else {
                panic!("one prepass VC row")
            };
            assert_eq!(row.status, PhaseStatus::Complete);
            assert!(row.diagnostics.is_empty());
            let [output] = row.output_refs.as_slice() else {
                panic!("one prepass VC output")
            };
            assert_eq!(output.output_kind(), &OutputKind::new("SourceTheoremVC"));
            let handle = prepass_owner
                .storage()
                .typed_handle::<mizar_vc::vc_ir::VcSet>(output, output.output_kind())
                .unwrap();
            let vcs = prepass_owner.storage().get(&handle).unwrap();
            let [record] = vcs.vcs() else {
                panic!("actual singleton VC")
            };
            let selected = record.id;
            let order = u64::try_from(selected.index()).unwrap();
            input.vc_descriptors.push(VcTaskDescriptor::new(
                descriptor_id.clone(),
                module.clone(),
                VcOrderKey::new(format!("{order:020}")),
                if backend_free {
                    vec![]
                } else {
                    vec![BackendProfileId::new("step6c99-backend")]
                },
                vec![EvidenceCandidateId::new("step6c99-evidence")],
            ));
            Some((vcs, selected))
        } else {
            None
        };
        clean_input.vc_descriptors = input.vc_descriptors.clone();
        let descriptor_count = input.vc_descriptors.len();
        if backend_free && stage == 3 {
            for case in [
                "spoof-source",
                "spoof-frontend",
                "spoof-artifact",
                "extra",
                "provider-none",
                "provider-error",
                "provider-snapshot",
            ] {
                let parent = format!("same-artifact-admission-{case}");
                let parent_root = fixture.root.join("alpha").join(&parent);
                fs::create_dir(&parent_root).unwrap();
                let negative_owner = publisher(snapshot, 0, &units[0]);
                for module in ["main", "leaf"] {
                    for (phase, kind) in [
                        ("SourceLoad", "SourceUnit"),
                        ("Frontend", "FrontendOutput"),
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
                        ("KernelCheck", "SourceTheoremCandidateCheck"),
                        ("KernelCheck", "SourceTheoremCandidateWitnessFile"),
                        ("ArtifactCommit", "SourceTheoremArtifactData"),
                    ] {
                        negative_owner.allow_work_unit(AllowedWorkUnit::new(
                            IrPipelinePhase::new(phase),
                            OutputKind::new(kind),
                            IrWorkUnit::new(format!("{:?}:{:?}", "alpha", module)),
                        ));
                    }
                }
                let mut negative = PhaseRegistryBuilder::new();
                for phase in [
                    PipelinePhase::SourceLoad,
                    PipelinePhase::Frontend,
                    PipelinePhase::ArtifactCommit,
                ] {
                    let wrapped = matches!(
                        (case, phase),
                        ("spoof-source", PipelinePhase::SourceLoad)
                            | ("spoof-frontend", PipelinePhase::Frontend)
                            | ("spoof-artifact", PipelinePhase::ArtifactCommit)
                    );
                    let mut concrete = PhaseRegistryBuilder::new();
                    let current = if wrapped {
                        &mut concrete
                    } else {
                        &mut negative
                    };
                    match phase {
                        PipelinePhase::SourceLoad => {
                            current.register_source_load();
                        }
                        PipelinePhase::Frontend => {
                            current.register_frontend(vec![]);
                        }
                        PipelinePhase::ArtifactCommit => {
                            current
                                .register_source_theorem_artifact_commit(
                                    policy.clone(),
                                    profile.clone(),
                                    ConcreteFormat::Tptp,
                                    verifier.clone(),
                                    (
                                        "proof-witnesses/scheduled".into(),
                                        "main.mizir.json".into(),
                                        parent.clone(),
                                    ),
                                )
                                .unwrap();
                        }
                        _ => unreachable!(),
                    }
                    if wrapped {
                        let registry = concrete.build().unwrap();
                        negative.register(ScheduledArtifactServiceForwarder {
                            descriptor: registry.descriptor_for_phase(phase).unwrap().clone(),
                            registry: std::sync::Arc::new(std::sync::Mutex::new(registry)),
                            replay: None,
                        });
                    }
                }
                negative
                    .register_source_theorem_module_resolver()
                    .register_source_theorem_signature_collection()
                    .register_source_theorem_type_checking()
                    .register_source_theorem_registration_resolution()
                    .register_source_theorem_overload_resolution()
                    .register_source_theorem_elaboration()
                    .register_source_theorem_algorithm_preparation()
                    .register_source_theorem_vc_generation(
                        generation_schema.clone(),
                        vc_schema.clone(),
                    )
                    .register_source_theorem_vc_discharge(policy.clone())
                    .register_source_theorem_atp_solve(
                        policy.clone(),
                        profile.clone(),
                        ConcreteFormat::Tptp,
                    )
                    .unwrap()
                    .register_source_theorem_candidate_check_with_witness_files(
                        policy.clone(),
                        profile.clone(),
                        ConcreteFormat::Tptp,
                        verifier.clone(),
                        "proof-witnesses/scheduled".into(),
                    )
                    .unwrap();
                if case == "extra" {
                    negative.register(ScheduledArtifactServiceForwarder {
                        descriptor: mizar_driver::registry::PhaseDescriptor::new(
                            "ExtraPlanner",
                            mizar_driver::registry::PhaseOwner::MizarBuild,
                            vec![PipelinePhase::PackageResolve],
                            "fixture-v1",
                            "fixture-output",
                        )
                        .unwrap(),
                        registry: std::sync::Arc::new(std::sync::Mutex::new(
                            PhaseRegistryBuilder::new().build().unwrap(),
                        )),
                        replay: None,
                    });
                }
                let negative = negative.build().unwrap();
                assert_eq!(
                    negative.descriptors().len(),
                    if case == "extra" { 15 } else { 14 }
                );
                let mut negative_driver =
                    CompilerDriver::new(negative).with_output_publisher(negative_owner);
                let (_, mut negative_input) = fixture.submit_request(vec![], vec![]);
                negative_input.plan_request = input.plan_request.clone();
                negative_input.workspace_packages = input.workspace_packages.clone();
                negative_input.lockfile = input.lockfile.clone();
                negative_input.source_layout = input.source_layout.clone();
                negative_input.dependency_overlay = input.dependency_overlay.clone();
                negative_input.vc_descriptors = input.vc_descriptors.clone();
                negative_input.phase_dispatch_inputs = match case {
                    "provider-none" => Some(Box::new(SuppliedInput::None)),
                    "provider-error" => Some(Box::new(SuppliedInput::Error)),
                    "provider-snapshot" => Some(Box::new(SuppliedInput::ForeignSnapshot)),
                    _ => None,
                };
                let negative = negative_driver
                    .submit_captured(session.clone(), &ids, &snapshots, negative_input)
                    .unwrap();
                assert!(negative.missing_services.is_empty(), "{case}");
                let failed = matches!(case, "provider-error" | "provider-snapshot");
                assert_eq!(
                    negative.status,
                    if failed {
                        DriverSubmissionStatus::SchedulerValidated
                    } else {
                        DriverSubmissionStatus::BlockedByPhaseDispatchGap
                    },
                    "{case}"
                );
                assert_eq!(
                    negative.session.state(),
                    BuildSessionState::Finished(if failed {
                        BuildSessionOutcome::Failed
                    } else {
                        BuildSessionOutcome::Blocked
                    }),
                    "{case}"
                );
                let graph = negative.task_graph.as_ref().unwrap();
                let run = negative.scheduler_run.as_ref().unwrap();
                if case.starts_with("provider-") {
                    assert!(run.phase_results.is_empty(), "{case}");
                    assert_eq!(
                        negative
                            .dispatch_gap_phases
                            .contains(&PipelinePhase::SourceLoad),
                        case == "provider-none"
                    );
                }
                for phase in [PipelinePhase::SourceLoad, PipelinePhase::Frontend] {
                    let task = graph
                        .tasks()
                        .iter()
                        .find(|task| task.unit == units[0] && task.phases == [phase])
                        .unwrap();
                    assert_eq!(
                        run.task_states
                            .iter()
                            .find(|state| state.task_id == task.id)
                            .unwrap()
                            .state,
                        if case.starts_with("provider-") {
                            if phase == PipelinePhase::SourceLoad && failed {
                                TaskState::Failed
                            } else {
                                TaskState::Blocked
                            }
                        } else {
                            TaskState::Completed
                        },
                        "{case} {phase:?}"
                    );
                }
                let artifact = graph
                    .tasks()
                    .iter()
                    .find(|task| task.kind == TaskKind::ArtifactCommit && task.unit == units[0])
                    .unwrap();
                assert_eq!(
                    run.task_states
                        .iter()
                        .find(|state| state.task_id == artifact.id)
                        .unwrap()
                        .state,
                    TaskState::Blocked,
                    "{case}"
                );
                assert!(!run.phase_results.contains_key(&artifact.id), "{case}");
                assert!(
                    run.phase_results
                        .values()
                        .flatten()
                        .flat_map(|row| &row.output_refs)
                        .all(|output| output.output_kind()
                            != &OutputKind::new("SourceTheoremArtifactData")),
                    "{case}"
                );
                assert_eq!(fs::read_dir(&parent_root).unwrap().count(), 0, "{case}");
            }
        }
        let replay_session = (backend_free && stage == 3).then(|| session.clone());
        let clean = clean_driver
            .submit_captured(session.clone(), &ids, &snapshots, clean_input)
            .unwrap();
        let actual = driver
            .submit_captured(session, &ids, &snapshots, input)
            .unwrap();
        for submission in [&actual, &clean] {
            assert_eq!(
                submission.status,
                if descriptor_count != 0 && !backend_free {
                    DriverSubmissionStatus::BlockedByMissingPhaseServices
                } else {
                    DriverSubmissionStatus::SchedulerValidated
                }
            );
            assert_eq!(
                submission.session.state(),
                BuildSessionState::Finished(BuildSessionOutcome::Blocked)
            );
            assert_eq!(submission.session.captured(), &captured);
            assert!(
                !submission
                    .missing_services
                    .iter()
                    .any(|gap| gap.phase == PipelinePhase::ArtifactCommit)
            );
            assert!(!submission.missing_services.iter().any(|gap| matches!(
                gap.phase,
                PipelinePhase::ModuleResolve
                    | PipelinePhase::SignatureCollection
                    | PipelinePhase::TypeChecking
                    | PipelinePhase::RegistrationResolution
                    | PipelinePhase::OverloadResolution
                    | PipelinePhase::Elaboration
                    | PipelinePhase::AlgorithmPreparation
                    | PipelinePhase::VcGenerate
                    | PipelinePhase::VcDischarge
                    | PipelinePhase::AtpSolve
                    | PipelinePhase::KernelCheck
            )));
            assert_eq!(
                submission
                    .missing_services
                    .iter()
                    .any(|gap| gap.phase == PipelinePhase::BackendRun),
                descriptor_count != 0 && !backend_free
            );
        }
        assert_eq!(actual.missing_services, clean.missing_services);
        if let Some((old_owner, old_outputs, old_session, old_hashes)) = previous.take() {
            assert_ne!(hashes[0], old_hashes[0]);
            assert_eq!(hashes[1], old_hashes[1]);
            let old_snapshot = old_session.captured().snapshot().id;
            assert_ne!(snapshot, old_snapshot);
            old_owner
                .replace_current_snapshot(old_snapshot, snapshot)
                .unwrap();
            for old_output in &old_outputs {
                for target in [old_snapshot, snapshot] {
                    assert!(
                        old_owner
                            .validate_current_output(target, old_output)
                            .is_err()
                    );
                }
            }
            assert_eq!(
                driver
                    .validate_current_native_diagnostics(old_session.id(), &snapshots)
                    .unwrap_err(),
                "obsolete diagnostic session"
            );
        }
        driver
            .validate_current_native_diagnostics(actual.session.id(), &snapshots)
            .unwrap();
        let graph = actual.task_graph.as_ref().unwrap();
        let run = actual.scheduler_run.as_ref().unwrap();
        let clean_run = clean.scheduler_run.as_ref().unwrap();
        assert_eq!(
            graph
                .tasks()
                .iter()
                .filter(|task| task.kind == TaskKind::VcDischarge)
                .count(),
            descriptor_count
        );
        for kind in [
            TaskKind::AtpSolve,
            TaskKind::BackendRun,
            TaskKind::KernelCheck,
        ] {
            assert_eq!(
                graph
                    .tasks()
                    .iter()
                    .filter(|task| task.kind == kind)
                    .count(),
                if backend_free && kind != TaskKind::KernelCheck {
                    0
                } else {
                    descriptor_count
                }
            );
        }
        assert_eq!(
            run.phase_results.keys().collect::<Vec<_>>(),
            clean_run.phase_results.keys().collect::<Vec<_>>()
        );
        for (task, rows) in &run.phase_results {
            let clean_rows = &clean_run.phase_results[task];
            assert_eq!(rows.len(), clean_rows.len());
            for (row, clean) in rows.iter().zip(clean_rows) {
                assert_eq!(
                    (
                        row.status,
                        &row.diagnostics,
                        &row.output_refs,
                        row.cache_observation
                    ),
                    (
                        clean.status,
                        &clean.diagnostics,
                        &clean.output_refs,
                        clean.cache_observation
                    )
                );
                if row.output_refs.iter().all(|output| {
                    output.output_kind() != &OutputKind::new("SourceTheoremArtifactData")
                }) {
                    assert!(row.artifact_commit.is_none());
                    assert!(clean.artifact_commit.is_none());
                }
            }
        }
        assert_eq!(run.task_states, clean_run.task_states);
        for module in ["main", "leaf"] {
            let version = versions
                .iter()
                .find(|v| v.module_path.as_str() == module)
                .unwrap();
            let mut parent: Option<AnyPhaseOutputRef> = None;
            let mut stale_outputs = Vec::new();
            for phase in [
                PipelinePhase::SourceLoad,
                PipelinePhase::Frontend,
                PipelinePhase::ModuleResolve,
                PipelinePhase::SignatureCollection,
                PipelinePhase::TypeChecking,
                PipelinePhase::RegistrationResolution,
                PipelinePhase::OverloadResolution,
                PipelinePhase::Elaboration,
                PipelinePhase::AlgorithmPreparation,
            ] {
                let task = graph.tasks().iter().find(|task| {
                    task.phases.contains(&phase) && matches!(&task.unit, WorkUnit::Module { module: id } if id.path.as_str() == module)
                }).unwrap();
                assert_eq!(
                    run.task_states
                        .iter()
                        .find(|state| state.task_id == task.id)
                        .unwrap()
                        .state,
                    if matches!(
                        phase,
                        PipelinePhase::SignatureCollection
                            | PipelinePhase::TypeChecking
                            | PipelinePhase::RegistrationResolution
                            | PipelinePhase::OverloadResolution
                            | PipelinePhase::Elaboration
                            | PipelinePhase::AlgorithmPreparation
                    ) {
                        if module == "main" && stage >= 3 {
                            TaskState::Completed
                        } else {
                            TaskState::Blocked
                        }
                    } else {
                        TaskState::Completed
                    }
                );
                let results = &run.phase_results[&task.id];
                assert_eq!(
                    results.len(),
                    if matches!(
                        phase,
                        PipelinePhase::SignatureCollection
                            | PipelinePhase::TypeChecking
                            | PipelinePhase::RegistrationResolution
                            | PipelinePhase::OverloadResolution
                            | PipelinePhase::Elaboration
                            | PipelinePhase::AlgorithmPreparation
                    ) {
                        if module == "main" && stage >= 3 { 6 } else { 5 }
                    } else {
                        1
                    }
                );
                let result_index = match phase {
                    PipelinePhase::TypeChecking => 1,
                    PipelinePhase::RegistrationResolution => 2,
                    PipelinePhase::OverloadResolution => 3,
                    PipelinePhase::Elaboration => 4,
                    PipelinePhase::AlgorithmPreparation => 5,
                    _ => 0,
                };
                let result = &results[result_index];
                let elaboration_refusal =
                    phase == PipelinePhase::Elaboration && (module == "leaf" || stage < 3);
                assert_eq!(
                    result.status,
                    if elaboration_refusal {
                        PhaseStatus::Blocking
                    } else {
                        PhaseStatus::Complete
                    }
                );
                assert!(result.diagnostics.is_empty());
                assert!(result.cache_observation.is_some());
                let parent_refs = if phase == PipelinePhase::OverloadResolution {
                    vec![
                        results[1].output_refs[0].clone(),
                        results[2].output_refs[0].clone(),
                    ]
                } else if phase == PipelinePhase::Elaboration {
                    vec![results[3].output_refs[0].clone()]
                } else if phase == PipelinePhase::AlgorithmPreparation {
                    vec![results[4].output_refs[0].clone()]
                } else {
                    parent.iter().cloned().collect()
                };
                let mut expected_parent_ids = parent_refs
                    .iter()
                    .map(AnyPhaseOutputRef::output)
                    .collect::<Vec<_>>();
                expected_parent_ids
                    .sort_by(|left, right| left.hash().as_bytes().cmp(right.hash().as_bytes()));
                let expected_parent_hashes = expected_parent_ids
                    .iter()
                    .map(|parent| parent.hash())
                    .collect::<Vec<_>>();
                let parents = parent_refs
                    .iter()
                    .map(|p| {
                        SealedParentOutputHandle::from_published_output(&owner, snapshot, p.clone())
                            .unwrap()
                    })
                    .collect();
                let query_input = PhaseInput::new(
                    task.unit.clone(),
                    PhaseDispatchInputBundle::new(snapshot, source_key(version), vec![], parents)
                        .unwrap(),
                );
                let cache = driver
                    .registry()
                    .cache_key_for_phase(phase, &query_input)
                    .unwrap();
                assert!(
                    matches!(&cache.intent, PhaseCacheIntent::NoKey { reason } if !reason.is_empty())
                );
                assert_eq!(
                    cache,
                    clean_driver
                        .registry()
                        .cache_key_for_phase(phase, &query_input)
                        .unwrap()
                );
                if phase == PipelinePhase::TypeChecking {
                    assert_eq!(
                        query_input.parent_outputs()[0].as_output_ref(),
                        &results[0].output_refs[0]
                    );
                } else if phase == PipelinePhase::RegistrationResolution {
                    assert_eq!(
                        query_input.parent_outputs()[0].as_output_ref(),
                        &results[1].output_refs[0]
                    );
                } else if phase == PipelinePhase::OverloadResolution {
                    assert_eq!(
                        query_input
                            .parent_outputs()
                            .iter()
                            .map(SealedParentOutputHandle::output)
                            .collect::<Vec<_>>(),
                        expected_parent_ids
                    );
                    assert_eq!(
                        query_input.identities().parent_output_hashes(),
                        expected_parent_hashes
                    );
                } else if phase == PipelinePhase::Elaboration {
                    assert_eq!(
                        query_input.parent_outputs()[0].as_output_ref(),
                        &results[3].output_refs[0]
                    );
                    assert_eq!(query_input.parent_outputs().len(), 1);
                    assert_eq!(
                        query_input.identities().parent_output_hashes(),
                        expected_parent_hashes
                    );
                } else if phase == PipelinePhase::AlgorithmPreparation {
                    assert_eq!(query_input.parent_outputs().len(), 1);
                    assert_eq!(
                        query_input.parent_outputs()[0].as_output_ref(),
                        &results[4].output_refs[0]
                    );
                    assert_eq!(
                        query_input.identities().parent_output_hashes(),
                        expected_parent_hashes
                    );
                }
                if matches!(
                    phase,
                    PipelinePhase::TypeChecking
                        | PipelinePhase::RegistrationResolution
                        | PipelinePhase::OverloadResolution
                        | PipelinePhase::Elaboration
                        | PipelinePhase::AlgorithmPreparation
                ) {
                    let probe = driver
                        .registry()
                        .execute_phase_with_resources(
                            phase,
                            query_input,
                            PhaseExecutionResources::default(),
                        )
                        .unwrap();
                    assert_eq!(result.cache_observation, Some(probe.observation));
                }
                if elaboration_refusal {
                    assert!(result.output_refs.is_empty());
                    break;
                }
                let [output] = result.output_refs.as_slice() else {
                    panic!("one genuine output")
                };
                let expected = &clean_run.phase_results[&task.id][result_index].output_refs[0];
                assert!(matches!(output.placement(), StoragePlacement::Blob { .. }));
                assert_eq!(output.lineage().parents, expected_parent_ids);
                assert_eq!(
                    owner.storage().side_tables_by_ref(output).unwrap(),
                    clean_owner.storage().side_tables_by_ref(expected).unwrap()
                );
                if phase == PipelinePhase::SourceLoad {
                    let [source, expected] = [&owner, &clean_owner].map(|current| {
                        let typed = current
                            .storage()
                            .typed_handle::<SourceUnit>(output, output.output_kind())
                            .unwrap();
                        current.storage().get(&typed).unwrap()
                    });
                    assert_eq!(source, expected);
                    assert_eq!(source.source_id, version.source_id);
                    assert_eq!(source.source_hash, version.source_hash);
                    assert_eq!(source.module_path, version.module_path);
                } else if phase == PipelinePhase::Frontend {
                    let [frontend, expected] = [&owner, &clean_owner].map(|current| {
                        let typed = current
                            .storage()
                            .typed_handle::<FrontendOutput<<MizarParserSeam as ParserSeam>::Ast>>(
                                output,
                                output.output_kind(),
                            )
                            .unwrap();
                        current.storage().get(&typed).unwrap()
                    });
                    assert_eq!(frontend, expected);
                    assert_eq!(frontend.source.source_id, version.source_id);
                    assert_eq!(frontend.source.module_path, version.module_path);
                } else if phase == PipelinePhase::ModuleResolve {
                    let [pair, expected] = [&owner, &clean_owner].map(|current| {
                        let typed = current
                            .storage()
                            .typed_handle::<(
                                mizar_resolve::resolved_ast::ResolvedAst,
                                mizar_resolve::env::SymbolEnv,
                            )>(output, output.output_kind())
                            .unwrap();
                        current.storage().get(&typed).unwrap()
                    });
                    assert_eq!(pair, expected);
                    assert_eq!(pair.0.source_id(), version.source_id);
                    assert_eq!(
                        pair.0.module_id(),
                        &ResolverModuleId::new(
                            version.package_id.clone(),
                            version.module_path.clone()
                        )
                    );
                } else if phase == PipelinePhase::SignatureCollection {
                    let [summary, expected] = [&owner, &clean_owner].map(|current| {
                        let typed = current
                            .storage()
                            .typed_handle::<mizar_artifact::module_summary::ModuleSummary>(
                                output,
                                output.output_kind(),
                            )
                            .unwrap();
                        current.storage().get(&typed).unwrap()
                    });
                    assert_eq!(summary, expected);
                    assert_eq!(summary.source_hash, version.source_hash);
                    assert_eq!(summary.module.package_id, version.package_id.as_str());
                    assert_eq!(summary.module.module_path, version.module_path.as_str());
                    if module == "main" && stage >= 3 {
                        assert_eq!(summary.exported_symbols.len(), usize::from(stage != 5));
                        assert!(
                            summary
                                .exported_symbols
                                .iter()
                                .all(|row| row.visibility == "public")
                        );
                    }
                } else if phase == PipelinePhase::RegistrationResolution {
                    assert_eq!(
                        output.phase(),
                        &IrPipelinePhase::new("RegistrationResolution")
                    );
                    assert_eq!(
                        output.output_kind(),
                        &OutputKind::new("SourceTheoremRegistrationClosure")
                    );
                    let actual_handle = owner
                        .storage()
                        .typed_handle::<mizar_checker::cluster_trace::ClusterClosureOutput>(
                            output,
                            output.output_kind(),
                        )
                        .unwrap();
                    let expected_handle = clean_owner
                        .storage()
                        .typed_handle::<mizar_checker::cluster_trace::ClusterClosureOutput>(
                            expected,
                            expected.output_kind(),
                        )
                        .unwrap();
                    let closure = owner.storage().get(&actual_handle).unwrap();
                    let clean_closure = clean_owner.storage().get(&expected_handle).unwrap();
                    assert_eq!(*closure, *clean_closure);
                    assert_eq!(
                        closure.status(),
                        mizar_checker::cluster_trace::ClusterClosureStatus::Complete
                    );
                    assert_eq!(closure.trace().source_id(), version.source_id);
                    assert_eq!(
                        closure.trace().module_id(),
                        &ResolverModuleId::new(
                            version.package_id.clone(),
                            version.module_path.clone()
                        )
                    );
                    assert_eq!(
                        closure
                            .trace()
                            .traversal_profile()
                            .cache_key_material()
                            .as_str(),
                        "order=cluster-trace-order-v1;max_depth=128;max_generated=4096"
                    );
                } else if phase == PipelinePhase::OverloadResolution {
                    assert_eq!(output.phase(), &IrPipelinePhase::new("OverloadResolution"));
                    assert_eq!(
                        output.output_kind(),
                        &OutputKind::new("SourceTheoremMetadata")
                    );
                    let actual_handle = owner
                        .storage()
                        .typed_handle::<(
                            mizar_checker::binding_env::BindingEnv,
                            mizar_checker::type_checker::TermFormulaInferenceOutput,
                            mizar_checker::typed_ast::TypedAst,
                            mizar_checker::resolved_typed_ast::ResolvedTypedAst,
                        )>(output, output.output_kind())
                        .unwrap();
                    let expected_handle = clean_owner
                        .storage()
                        .typed_handle::<(
                            mizar_checker::binding_env::BindingEnv,
                            mizar_checker::type_checker::TermFormulaInferenceOutput,
                            mizar_checker::typed_ast::TypedAst,
                            mizar_checker::resolved_typed_ast::ResolvedTypedAst,
                        )>(expected, expected.output_kind())
                        .unwrap();
                    let metadata = owner.storage().get(&actual_handle).unwrap();
                    let clean_metadata = clean_owner.storage().get(&expected_handle).unwrap();
                    assert_eq!(*metadata, *clean_metadata);
                    for source_id in [
                        metadata.0.source_id(),
                        metadata.1.source_id(),
                        metadata.2.source_id(),
                        metadata.3.source_id(),
                    ] {
                        assert_eq!(source_id, version.source_id);
                    }
                    for module_id in [
                        metadata.0.module_id(),
                        metadata.1.module_id(),
                        metadata.2.module_id(),
                        metadata.3.module_id(),
                    ] {
                        assert_eq!(
                            module_id,
                            &ResolverModuleId::new(
                                version.package_id.clone(),
                                version.module_path.clone()
                            )
                        );
                    }
                    let references = metadata
                        .3
                        .nodes()
                        .iter()
                        .filter_map(|(_, node)| node.final_type)
                        .collect::<Vec<_>>();
                    assert!(!references.is_empty());
                    for reference in references {
                        let inferred = metadata.1.normalized_types().get(reference).unwrap();
                        assert_eq!(
                            inferred.head,
                            mizar_checker::type_checker::TypeHeadRef::BuiltinSet
                        );
                        assert_eq!(
                            inferred.status,
                            mizar_checker::type_checker::NormalizedTypeStatus::Known
                        );
                    }
                    let atomic = metadata.2.source_atomic_formula().unwrap();
                    assert!(!atomic.requests().is_empty());
                    assert!(atomic.requests().iter().all(|(_, request)| request.kind()
                        == mizar_checker::source_atomic_formula::SourceAtomicRequestKind::OperandExpectedType));
                    assert_eq!(metadata.3.source_atomic_formula(), Some(atomic));
                    assert!(metadata.3.collection_candidates().is_empty());
                    assert!(metadata.3.resolved_overloads().is_empty());
                } else if phase == PipelinePhase::Elaboration {
                    assert_eq!(output.phase(), &IrPipelinePhase::new("Elaboration"));
                    assert_eq!(output.output_kind(), &OutputKind::new("SourceTheoremCore"));
                    let [core, clean_core] =
                        [(&owner, output), (&clean_owner, expected)].map(|(current, output)| {
                            let handle = current
                                .storage()
                                .typed_handle::<mizar_core::core_ir::CoreIr>(
                                    output,
                                    &OutputKind::new("SourceTheoremCore"),
                                )
                                .unwrap();
                            current.storage().get(&handle).unwrap()
                        });
                    assert_eq!(core, clean_core);
                    assert_eq!(core.items().len(), 1);
                    for (_, item) in core.items().iter() {
                        assert_eq!(
                            item.visibility.as_str(),
                            if stage == 5 { "private" } else { "public" }
                        );
                    }
                    assert_eq!(core.source_id(), version.source_id);
                    assert_eq!(
                        core.module_id(),
                        &ResolverModuleId::new(
                            version.package_id.clone(),
                            version.module_path.clone()
                        )
                    );
                    assert!(!core.proofs().is_empty());
                    assert!(core.proofs().iter().all(|(_, proof)| {
                        proof.status == mizar_core::core_ir::CoreProofStatus::PendingAutomaticProof
                    }));
                    assert!(!core.obligation_seeds().is_empty());
                    assert!(core.obligation_seeds().iter().all(|(_, seed)| {
                        seed.status == mizar_core::core_ir::ObligationSeedStatus::Active
                    }));
                } else if phase == PipelinePhase::AlgorithmPreparation {
                    assert_eq!(
                        output.phase(),
                        &IrPipelinePhase::new("AlgorithmPreparation")
                    );
                    assert_eq!(
                        output.output_kind(),
                        &OutputKind::new("SourceTheoremPreparation")
                    );
                    let [preparation, clean_preparation] =
                        [(&owner, output), (&clean_owner, expected)].map(|(current, output)| {
                            let handle = current
                                .storage()
                                .typed_handle::<(
                                    mizar_core::control_flow::ControlFlowOutput,
                                    mizar_core::control_flow::ObligationSeedHandoff,
                                )>(output, output.output_kind())
                                .unwrap();
                            current.storage().get(&handle).unwrap()
                        });
                    assert_eq!(preparation, clean_preparation);
                    assert!(preparation.0.flows.is_empty());
                    assert!(preparation.0.flow_map.is_empty());
                    let core_output = &results[4].output_refs[0];
                    let core_handle = owner
                        .storage()
                        .typed_handle::<mizar_core::core_ir::CoreIr>(
                            core_output,
                            core_output.output_kind(),
                        )
                        .unwrap();
                    let core = owner.storage().get(&core_handle).unwrap();
                    assert_eq!(preparation.1.entries.len(), core.obligation_seeds().len());
                    assert!(!preparation.1.entries.is_empty());
                    for (id, entry) in preparation.1.entries.iter() {
                        let mizar_core::control_flow::ObligationHandoffOrigin::ExistingCore {
                            seed,
                        } = entry.origin
                        else {
                            panic!("existing Core seed")
                        };
                        assert_eq!(&entry.seed, core.obligation_seeds().get(seed).unwrap());
                        assert_eq!(
                            preparation.1.source_map.get(&id),
                            Some(&core.source_map().obligation_sources[&seed])
                        );
                        assert!(entry.flow_site.is_none());
                    }
                } else {
                    let [typed, expected] = [&owner, &clean_owner].map(|current| {
                        let handle = current
                            .storage()
                            .typed_handle::<mizar_checker::typed_ast::TypedAst>(
                                output,
                                output.output_kind(),
                            )
                            .unwrap();
                        current.storage().get(&handle).unwrap()
                    });
                    assert_eq!(typed, expected);
                    assert_eq!(typed.source_id(), version.source_id);
                    assert_eq!(
                        typed.module_id(),
                        &ResolverModuleId::new(
                            version.package_id.clone(),
                            version.module_path.clone()
                        )
                    );
                    assert!(!typed.types().is_empty());
                    let declarations = typed.source_context().unwrap().declarations();
                    assert!(declarations.iter().next().is_some());
                    for (_, declaration) in declarations.iter() {
                        let node = typed.nodes().node(declaration.site.node()).unwrap();
                        assert_eq!(
                            node.typing,
                            mizar_checker::typed_ast::TypingState::Successful
                        );
                        assert!(node.links.type_entry.is_some());
                    }
                }
                if matches!(
                    phase,
                    PipelinePhase::Elaboration | PipelinePhase::AlgorithmPreparation
                ) {
                    stale_outputs.push(output.clone());
                }
                parent = Some(output.clone());
            }
            if module == "main" && stage >= 3 {
                let check = graph
                    .tasks()
                    .iter()
                    .find(|candidate| {
                        candidate.kind == TaskKind::CheckAndElaborate
                            && matches!(&candidate.unit, WorkUnit::Module { module } if module.path.as_str() == "main")
                    })
                    .unwrap();
                let vc = graph
                    .tasks()
                    .iter()
                    .find(|candidate| {
                        candidate.kind == TaskKind::VcGenerate
                            && matches!(&candidate.unit, WorkUnit::Module { module } if module.path.as_str() == "main")
                    })
                    .unwrap();
                assert!(vc.dependencies.contains(&check.id));
                assert_eq!(
                    run.task_states
                        .iter()
                        .find(|state| state.task_id == vc.id)
                        .unwrap()
                        .state,
                    TaskState::Completed
                );
                let vc_results = &run.phase_results[&vc.id];
                let [vc_result] = vc_results.as_slice() else {
                    panic!("one real separate VC result")
                };
                assert_eq!(vc_result.status, PhaseStatus::Complete);
                assert!(vc_result.diagnostics.is_empty());
                let [vc_output] = vc_result.output_refs.as_slice() else {
                    panic!("one genuine VC output")
                };
                let clean_vc_output = &clean_run.phase_results[&vc.id][0].output_refs[0];
                assert_eq!(vc_output.phase(), &IrPipelinePhase::new("VcGenerate"));
                assert_eq!(vc_output.output_kind(), &OutputKind::new("SourceTheoremVC"));
                assert_eq!(vc_output.snapshot(), snapshot);
                assert_eq!(
                    vc_output.schema_version(),
                    mizar_ir::storage::SchemaVersion::new(1)
                );
                assert_eq!(
                    vc_output.work_unit(),
                    &IrWorkUnit::new(format!("{:?}:{:?}", "alpha", "main"))
                );
                let check_results = &run.phase_results[&check.id];
                let core_output = &check_results[4].output_refs[0];
                let preparation_output = &check_results[5].output_refs[0];
                let mut expected_parent_ids =
                    vec![core_output.output(), preparation_output.output()];
                expected_parent_ids
                    .sort_by(|left, right| left.hash().as_bytes().cmp(right.hash().as_bytes()));
                assert_eq!(vc_output.lineage().parents, expected_parent_ids);
                assert_eq!(
                    owner.storage().side_tables_by_ref(vc_output).unwrap(),
                    clean_owner
                        .storage()
                        .side_tables_by_ref(clean_vc_output)
                        .unwrap()
                );
                assert!(matches!(
                    vc_output.placement(),
                    StoragePlacement::Blob { .. }
                ));
                let [vc_set, clean_vc_set] = [(&owner, vc_output), (&clean_owner, clean_vc_output)]
                    .map(|(current, output)| {
                        let handle = current
                            .storage()
                            .typed_handle::<mizar_vc::vc_ir::VcSet>(output, output.output_kind())
                            .unwrap();
                        current.storage().get(&handle).unwrap()
                    });
                assert_eq!(vc_set, clean_vc_set);
                assert_eq!(vc_set.source(), version.source_id);
                assert_eq!(vc_set.snapshot(), snapshot);
                assert_eq!(vc_set.module().as_str(), "package=5:alpha;module=4:main");
                assert_eq!(vc_set.schema_version(), &vc_schema);
                assert!(!vc_set.vcs().is_empty());
                assert!(vc_set.vcs().iter().all(|vc| {
                    vc.status == mizar_vc::vc_ir::VcStatus::Open
                        && vc.anchor.generation_schema_version == generation_schema
                }));
                let descriptor = driver
                    .registry()
                    .descriptor_for_phase(PipelinePhase::VcGenerate)
                    .unwrap();
                assert_eq!(
                    descriptor.schema_version,
                    format!(
                        "mizar-vc/source-theorem-vc-publication/v1;generation={}:{};vc={}:{}",
                        generation_schema.as_str().len(),
                        generation_schema.as_str(),
                        vc_schema.as_str().len(),
                        vc_schema.as_str()
                    )
                );
                let core_handle = owner
                    .storage()
                    .typed_handle::<mizar_core::core_ir::CoreIr>(
                        core_output,
                        core_output.output_kind(),
                    )
                    .unwrap();
                let core = owner.storage().get(&core_handle).unwrap();
                let preparation_handle = owner
                    .storage()
                    .typed_handle::<(
                        mizar_core::control_flow::ControlFlowOutput,
                        mizar_core::control_flow::ObligationSeedHandoff,
                    )>(preparation_output, preparation_output.output_kind())
                    .unwrap();
                let preparation = owner.storage().get(&preparation_handle).unwrap();
                assert_eq!(vc_set.seed_accounting().len(), preparation.1.entries.len());
                assert!(!preparation.1.entries.is_empty());
                for row in vc_set.seed_accounting() {
                    let mizar_vc::vc_ir::SeedOriginRef::ExistingCore { seed } = row.origin else {
                        panic!("VC accounting uses an existing Core seed")
                    };
                    let entry = preparation.1.entries.get(row.handoff).unwrap();
                    assert_eq!(&entry.seed, core.obligation_seeds().get(seed).unwrap());
                    assert_eq!(
                        row.seed_status,
                        mizar_core::core_ir::ObligationSeedStatus::Active
                    );
                    let mizar_vc::vc_ir::SeedVcMapping::One { vc } = row.mapping else {
                        panic!("one VC for each existing Core seed")
                    };
                    let vc = vc_set.vc(vc).unwrap();
                    assert_eq!(vc.seed.handoff, row.handoff);
                    assert_eq!(vc.source.primary, entry.seed.source);
                }
                let parents = [core_output, preparation_output]
                    .into_iter()
                    .map(|parent| {
                        SealedParentOutputHandle::from_published_output(
                            &owner,
                            snapshot,
                            parent.clone(),
                        )
                        .unwrap()
                    })
                    .collect();
                let vc_input = PhaseInput::new(
                    vc.unit.clone(),
                    PhaseDispatchInputBundle::new(snapshot, source_key(version), vec![], parents)
                        .unwrap(),
                );
                assert_eq!(
                    vc_input
                        .parent_outputs()
                        .iter()
                        .map(SealedParentOutputHandle::output)
                        .collect::<Vec<_>>(),
                    expected_parent_ids
                );
                let expected_query_parent_hashes = expected_parent_ids
                    .iter()
                    .map(|parent| parent.hash())
                    .collect::<Vec<_>>();
                assert_eq!(
                    vc_input.identities().parent_output_hashes(),
                    expected_query_parent_hashes
                );
                let cache = driver
                    .registry()
                    .cache_key_for_phase(PipelinePhase::VcGenerate, &vc_input)
                    .unwrap();
                assert!(matches!(
                    &cache.intent,
                    PhaseCacheIntent::NoKey { reason } if !reason.is_empty()
                ));
                assert_eq!(
                    cache,
                    clean_driver
                        .registry()
                        .cache_key_for_phase(PipelinePhase::VcGenerate, &vc_input)
                        .unwrap()
                );
                let probe = driver
                    .registry()
                    .execute_phase_with_resources(
                        PipelinePhase::VcGenerate,
                        vc_input,
                        PhaseExecutionResources::default(),
                    )
                    .unwrap();
                assert_eq!(vc_result.cache_observation, Some(probe.observation));
                let artifact = graph
                    .tasks()
                    .iter()
                    .find(|candidate| {
                        candidate.kind == TaskKind::ArtifactCommit
                            && matches!(&candidate.unit, WorkUnit::Module { module } if module.path.as_str() == "main")
                    })
                    .unwrap();
                assert!(artifact.dependencies.contains(&vc.id));
                assert_eq!(
                    run.task_states
                        .iter()
                        .find(|state| state.task_id == artifact.id)
                        .unwrap()
                        .state,
                    if backend_free {
                        TaskState::Completed
                    } else {
                        TaskState::Blocked
                    }
                );
                if !backend_free {
                    assert!(!run.phase_results.contains_key(&artifact.id));
                }
                stale_outputs.push(vc_output.clone());
                let (baseline, selected) = baseline_vcs.as_ref().unwrap();
                assert_eq!(&vc_set, baseline);
                let [original] = vc_set.vcs() else {
                    panic!("same actual singleton VC")
                };
                assert_eq!(original.id, *selected);
                let discharge = graph
                    .tasks()
                    .iter()
                    .find(|task| {
                        task.kind == TaskKind::VcDischarge
                            && matches!(&task.unit, WorkUnit::Vc { module, descriptor }
                            if module.path.as_str() == "main" && descriptor == &descriptor_id)
                    })
                    .unwrap();
                assert!(discharge.dependencies.contains(&vc.id));
                assert_eq!(discharge.phases, [PipelinePhase::VcDischarge]);
                assert_eq!(
                    run.task_states
                        .iter()
                        .find(|state| state.task_id == discharge.id)
                        .unwrap()
                        .state,
                    TaskState::Completed
                );
                let [row] = run.phase_results[&discharge.id].as_slice() else {
                    panic!("one separate discharge row")
                };
                assert_eq!(row.status, PhaseStatus::Complete);
                assert!(row.diagnostics.is_empty());
                let [output] = row.output_refs.as_slice() else {
                    panic!("one genuine discharge output")
                };
                let clean_output = &clean_run.phase_results[&discharge.id][0].output_refs[0];
                assert_eq!(output.phase(), &IrPipelinePhase::new("VcDischarge"));
                assert_eq!(
                    output.output_kind(),
                    &OutputKind::new("SourceTheoremDischarge")
                );
                assert_eq!(
                    output.schema_version(),
                    mizar_ir::storage::SchemaVersion::new(1)
                );
                assert_eq!(output.snapshot(), snapshot);
                assert_eq!(output.work_unit(), core_output.work_unit());
                assert!(matches!(output.placement(), StoragePlacement::Blob { .. }));
                assert_eq!(
                    owner.storage().side_tables_by_ref(output).unwrap(),
                    clean_owner
                        .storage()
                        .side_tables_by_ref(clean_output)
                        .unwrap()
                );
                let [value, clean_value] =
                    [(&owner, output), (&clean_owner, clean_output)].map(|(current, output)| {
                        let handle = current
                            .storage()
                            .typed_handle::<DischargeOutput>(output, output.output_kind())
                            .unwrap();
                        current.storage().get(&handle).unwrap()
                    });
                assert_eq!(value, clean_value);
                assert_eq!(
                    *value,
                    try_discharge(DischargeInput {
                        vc_set: &vc_set,
                        policy: &policy
                    })
                    .unwrap()
                );
                let discharged = value.vc_set();
                assert_eq!(discharged.source(), version.source_id);
                assert_eq!(
                    discharged.module().as_str(),
                    "package=5:alpha;module=4:main"
                );
                assert_eq!(discharged.snapshot(), snapshot);
                assert_eq!(discharged.schema_version(), &vc_schema);
                let [record] = discharged.vcs() else {
                    panic!("one actual discharged VC")
                };
                assert_eq!(record.id, *selected);
                assert_eq!(record.status, VcStatus::NeedsAtp);
                assert_eq!(record.anchor.generation_schema_version, generation_schema);
                assert_eq!(
                    record.provenance.last().unwrap().phase,
                    VcProvenancePhase::Discharge
                );
                let [explanation] = value.explanations() else {
                    panic!("one discharge explanation")
                };
                assert_eq!(explanation.vc, *selected);
                assert_eq!(
                    explanation.category,
                    DischargeExplanationCategory::UnsupportedRule
                );
                assert_eq!(explanation.rule, None);
                assert!(value.evidence_records().is_empty());
                assert_eq!(original.status, VcStatus::Open);
                assert!(core.proofs().iter().all(|(_, proof)| proof.status
                    == mizar_core::core_ir::CoreProofStatus::PendingAutomaticProof));
                assert!(
                    core.obligation_seeds().iter().all(|(_, seed)| seed.status
                        == mizar_core::core_ir::ObligationSeedStatus::Active)
                );
                let mut parent_ids = vec![core_output.output(), vc_output.output()];
                parent_ids
                    .sort_by(|left, right| left.hash().as_bytes().cmp(right.hash().as_bytes()));
                assert_eq!(output.lineage().parents, parent_ids);
                let parents = [core_output, vc_output]
                    .into_iter()
                    .map(|parent| {
                        SealedParentOutputHandle::from_published_output(
                            &owner,
                            snapshot,
                            parent.clone(),
                        )
                        .unwrap()
                    })
                    .collect();
                let query = PhaseInput::new(
                    discharge.unit.clone(),
                    PhaseDispatchInputBundle::new(snapshot, source_key(version), vec![], parents)
                        .unwrap(),
                );
                assert_eq!(
                    query
                        .parent_outputs()
                        .iter()
                        .map(SealedParentOutputHandle::output)
                        .collect::<Vec<_>>(),
                    parent_ids
                );
                assert_eq!(
                    query.identities().parent_output_hashes(),
                    parent_ids
                        .iter()
                        .map(|parent| parent.hash())
                        .collect::<Vec<_>>()
                );
                let cache = driver
                    .registry()
                    .cache_key_for_phase(PipelinePhase::VcDischarge, &query)
                    .unwrap();
                assert!(
                    matches!(&cache.intent, PhaseCacheIntent::NoKey { reason } if !reason.is_empty())
                );
                assert_eq!(
                    cache,
                    clean_driver
                        .registry()
                        .cache_key_for_phase(PipelinePhase::VcDischarge, &query)
                        .unwrap()
                );
                let probe = driver
                    .registry()
                    .execute_phase_with_resources(
                        PipelinePhase::VcDischarge,
                        query,
                        PhaseExecutionResources::default(),
                    )
                    .unwrap();
                assert_eq!(probe.result.status, PhaseStatus::Blocking);
                assert!(probe.result.output_refs.is_empty());
                assert_eq!(row.cache_observation, Some(probe.observation));
                assert!(artifact.dependencies.contains(&discharge.id));
                stale_outputs.push(output.clone());
                if backend_free {
                    let kernel = graph
                        .tasks()
                        .iter()
                        .find(|task| task.kind == TaskKind::KernelCheck)
                        .unwrap();
                    assert_eq!(kernel.phases, [PipelinePhase::KernelCheck]);
                    assert!(
                        matches!(&kernel.unit, WorkUnit::EvidenceCandidate { module, descriptor, .. }
                        if module.path.as_str() == "main" && descriptor == &descriptor_id)
                    );
                    let package_resolve = graph
                        .tasks()
                        .iter()
                        .find(|task| {
                            task.kind == TaskKind::PackageResolve
                                && task.unit == WorkUnit::Workspace
                        })
                        .unwrap();
                    let mut expected_dependencies =
                        vec![package_resolve.id.clone(), discharge.id.clone()];
                    expected_dependencies.sort();
                    assert_eq!(kernel.dependencies, expected_dependencies);
                    assert_eq!(
                        run.task_states
                            .iter()
                            .find(|state| state.task_id == kernel.id)
                            .unwrap()
                            .state,
                        TaskState::Completed
                    );
                    assert!(artifact.dependencies.contains(&kernel.id));
                    let [kernel_row] = run.phase_results[&kernel.id].as_slice() else {
                        panic!("one genuine Kernel row")
                    };
                    assert_eq!(kernel_row.status, PhaseStatus::Complete);
                    assert!(kernel_row.diagnostics.is_empty());
                    let [kernel_output, file_output] = kernel_row.output_refs.as_slice() else {
                        panic!("one genuine Kernel and one file output")
                    };
                    let [clean_kernel, clean_file] = clean_run.phase_results[&kernel.id][0]
                        .output_refs
                        .as_slice()
                    else {
                        panic!()
                    };
                    assert_eq!(kernel_output.phase(), &IrPipelinePhase::new("KernelCheck"));
                    assert_eq!(
                        kernel_output.output_kind(),
                        &OutputKind::new("SourceTheoremCandidateCheck")
                    );
                    assert_eq!(
                        kernel_output.schema_version(),
                        mizar_ir::storage::SchemaVersion::new(1)
                    );
                    assert_eq!(kernel_output.snapshot(), snapshot);
                    assert_eq!(kernel_output.work_unit(), core_output.work_unit());
                    assert_eq!(
                        kernel_output.lineage().named_input_hashes,
                        core_output.lineage().named_input_hashes
                    );
                    assert!(matches!(
                        kernel_output.placement(),
                        StoragePlacement::Blob { .. }
                    ));
                    assert_eq!(
                        owner.storage().side_tables_by_ref(kernel_output).unwrap(),
                        clean_owner
                            .storage()
                            .side_tables_by_ref(clean_kernel)
                            .unwrap()
                    );
                    let [checked, clean_checked] =
                        [(&owner, kernel_output), (&clean_owner, clean_kernel)].map(
                            |(current, output)| {
                                let handle = current
                                    .storage()
                                    .typed_handle::<(
                                        mizar_kernel::checker::KernelCheckResult,
                                        mizar_proof::policy::PolicyDecision,
                                    )>(
                                        output, output.output_kind()
                                    )
                                    .unwrap();
                                current.storage().get(&handle).unwrap()
                            },
                        );
                    assert_eq!(checked, clean_checked);
                    let frontend_task = graph
                        .tasks()
                        .iter()
                        .find(|task| task.kind == TaskKind::Frontend && task.unit == units[0])
                        .unwrap();
                    let frontend_output = &run.phase_results[&frontend_task.id][0].output_refs[0];
                    let frontend_handle = owner
                        .storage()
                        .typed_handle::<FrontendOutput<<MizarParserSeam as ParserSeam>::Ast>>(
                            frontend_output,
                            frontend_output.output_kind(),
                        )
                        .unwrap();
                    let frontend = owner.storage().get(&frontend_handle).unwrap();
                    let ast = frontend.ast.as_ref().unwrap();
                    assert_eq!(ast.source_id, version.source_id);
                    let (arena, env) =
                        mizar_resolve::symbols::source_theorem_module_original_inputs(
                            ast,
                            core.module_id(),
                        )
                        .unwrap()
                        .unwrap();
                    let variable_scope = mizar_resolve::names::SourceVariableScopeResolver::resolve_proof_occurrences(
                        mizar_resolve::names::SourceVariableScopeInput::new(ast, core.module_id(), &env)
                    ).unwrap();
                    let metadata = mizar_checker::type_checker::check_source_theorem_metadata(
                        &arena,
                        &env,
                        &variable_scope,
                    )
                    .unwrap();
                    let (scope, labels, resolved) =
                        mizar_resolve::symbols::source_proof_inputs(&arena, &env).unwrap();
                    let fresh_check = mizar_checker::type_checker::SourceVariableSemanticsChecker::check_theorem_skeletons(
                        &arena, &metadata.2, &scope, &env, &labels, &resolved, None
                    ).unwrap();
                    let fresh = mizar_proof::status::check_source_theorem_candidate(
                        &arena,
                        &fresh_check,
                        &env,
                        snapshot,
                        *selected,
                        (
                            &generation_schema,
                            &vc_schema,
                            profile.clone(),
                            ConcreteFormat::Tptp,
                        ),
                        (&policy, &verifier),
                    )
                    .unwrap();
                    let expected_pair = mizar_atp::backend::build_source_theorem_candidate_inputs(
                        &core,
                        &vc_set,
                        *selected,
                        &value,
                        &policy,
                        profile.clone(),
                        ConcreteFormat::Tptp,
                    )
                    .unwrap();
                    assert_eq!(
                        (&fresh.0, &fresh.1, &fresh.2, &fresh.3, &fresh.4),
                        (
                            &*core,
                            &*vc_set,
                            &*value,
                            &expected_pair.0,
                            &expected_pair.1
                        )
                    );
                    assert_eq!((&checked.0, &checked.1), (&fresh.5, &fresh.6));
                    assert_eq!(
                        checked.0.status(),
                        mizar_kernel::checker::KernelCheckStatus::Accepted
                    );
                    assert_eq!(
                        checked.0.evidence_check_kind(),
                        Some(mizar_kernel::checker::KernelEvidenceCheckKind::ProofObligation)
                    );
                    assert!(!checked.0.policy_taint());
                    assert_eq!(
                        checked.1.class,
                        mizar_proof::policy::CandidatePolicyClass::KernelVerified
                    );
                    assert_eq!(
                        expected_pair.1.target_binding(),
                        expected_pair.0.target_binding()
                    );
                    let handoff = mizar_vc::kernel_evidence_handoff::build_source_theorem_discharge_kernel_handoff(
                        &core, &vc_set, *selected, &value, &policy
                    ).unwrap();
                    let target = handoff.canonical_evidence().target_vc();
                    assert_eq!(
                        expected_pair.0.target_binding().fingerprint(),
                        &mizar_atp::problem::AtpFingerprint::new(
                            target.algorithm_id,
                            target.digest.clone()
                        )
                        .unwrap()
                    );
                    assert_eq!(fresh.1.vcs()[0].status, VcStatus::Open);
                    assert_eq!(fresh.2.vc_set().vcs()[0].status, VcStatus::NeedsAtp);
                    assert!(fresh.0.proofs().iter().all(|(_, proof)| proof.status
                        == mizar_core::core_ir::CoreProofStatus::PendingAutomaticProof));
                    assert!(
                        fresh
                            .0
                            .obligation_seeds()
                            .iter()
                            .all(|(_, seed)| seed.status
                                == mizar_core::core_ir::ObligationSeedStatus::Active)
                    );
                    let mut parent_ids =
                        vec![core_output.output(), vc_output.output(), output.output()];
                    parent_ids
                        .sort_by(|left, right| left.hash().as_bytes().cmp(right.hash().as_bytes()));
                    assert_eq!(kernel_output.lineage().parents.len(), 4);
                    let intermediate_ids = kernel_output
                        .lineage()
                        .parents
                        .iter()
                        .filter(|id| !parent_ids.contains(id))
                        .copied()
                        .collect::<Vec<_>>();
                    let [intermediate_id] = intermediate_ids.as_slice() else {
                        panic!("one real intermediate ATP lineage id")
                    };
                    let intermediate = owner.registry().output_lineage(*intermediate_id).unwrap();
                    let clean_intermediate = clean_owner
                        .registry()
                        .output_lineage(*intermediate_id)
                        .unwrap();
                    assert_eq!(intermediate, clean_intermediate);
                    assert_eq!(intermediate.output, *intermediate_id);
                    assert_eq!(intermediate.snapshot, snapshot);
                    assert_eq!(intermediate.phase, IrPipelinePhase::new("AtpSolve"));
                    assert_eq!(
                        intermediate.output_kind,
                        OutputKind::new("SourceTheoremCandidateInputs")
                    );
                    assert_eq!(&intermediate.work_unit, core_output.work_unit());
                    assert_eq!(intermediate.parents, parent_ids);
                    assert_eq!(
                        intermediate.named_input_hashes,
                        core_output.lineage().named_input_hashes
                    );
                    let mut output_parent_ids = parent_ids.clone();
                    output_parent_ids.push(*intermediate_id);
                    output_parent_ids
                        .sort_by(|left, right| left.hash().as_bytes().cmp(right.hash().as_bytes()));
                    assert_eq!(kernel_output.lineage().parents, output_parent_ids);
                    let parents = [core_output, vc_output, output]
                        .into_iter()
                        .map(|parent| {
                            SealedParentOutputHandle::from_published_output(
                                &owner,
                                snapshot,
                                parent.clone(),
                            )
                            .unwrap()
                        })
                        .collect();
                    let query = PhaseInput::new(
                        kernel.unit.clone(),
                        PhaseDispatchInputBundle::new(
                            snapshot,
                            source_key(version),
                            vec![],
                            parents,
                        )
                        .unwrap(),
                    );
                    assert_eq!(query.work_unit, kernel.unit);
                    assert_eq!(query.identities().input_hash(), source_key(version));
                    assert!(query.identities().dependency_hashes().is_empty());
                    assert_eq!(
                        query
                            .parent_outputs()
                            .iter()
                            .map(SealedParentOutputHandle::output)
                            .collect::<Vec<_>>(),
                        parent_ids
                    );
                    assert_eq!(
                        query.identities().parent_output_hashes(),
                        parent_ids.iter().map(|id| id.hash()).collect::<Vec<_>>()
                    );
                    let cache = driver
                        .registry()
                        .cache_key_for_phase(PipelinePhase::KernelCheck, &query)
                        .unwrap();
                    assert!(matches!(&cache.intent, PhaseCacheIntent::NoKey { reason }
                        if reason == "source theorem candidate check has no complete dependency footprint"));
                    assert_eq!(
                        cache,
                        clean_driver
                            .registry()
                            .cache_key_for_phase(PipelinePhase::KernelCheck, &query)
                            .unwrap()
                    );
                    let probe = driver
                        .registry()
                        .execute_phase_with_resources(
                            PipelinePhase::KernelCheck,
                            query,
                            PhaseExecutionResources::default(),
                        )
                        .unwrap();
                    assert_eq!(probe.result.status, PhaseStatus::Blocking);
                    assert!(probe.result.output_refs.is_empty());
                    assert_eq!(kernel_row.cache_observation, Some(probe.observation));
                    let kernel_hex = kernel_output
                        .content_hash()
                        .as_bytes()
                        .iter()
                        .map(|byte| format!("{byte:02x}"))
                        .collect::<String>();
                    let witness_path = format!("proof-witnesses/scheduled/{kernel_hex}.json");
                    let expected_file =
                        mizar_proof::status::source_theorem_candidate_witness_file_candidate(
                            &arena,
                            &fresh_check,
                            &env,
                            captured.snapshot(),
                            *selected,
                            (
                                &generation_schema,
                                &vc_schema,
                                profile.clone(),
                                ConcreteFormat::Tptp,
                            ),
                            (&policy, &verifier),
                            &witness_path,
                        )
                        .unwrap();
                    assert_eq!(expected_file.2.witness_path, witness_path);
                    assert_eq!(
                        expected_file.2.schema_version,
                        mizar_artifact::store::SchemaVersion::new(2, 0)
                    );
                    assert_eq!(
                        expected_file.2.witness_artifact_hash.digest,
                        mizar_artifact::store::artifact_hash_domain(
                            "mizar-proof/source-theorem-witness-file",
                            mizar_artifact::store::SchemaVersion::new(1, 0)
                        )
                        .hash(&expected_file.1, &[])
                    );
                    assert_eq!(
                        file_output.output_kind(),
                        &OutputKind::new("SourceTheoremCandidateWitnessFile")
                    );
                    assert_eq!(file_output.phase(), &IrPipelinePhase::new("KernelCheck"));
                    assert_eq!(
                        file_output.schema_version(),
                        mizar_ir::storage::SchemaVersion::new(1)
                    );
                    assert_eq!(file_output.snapshot(), snapshot);
                    assert_eq!(file_output.work_unit(), kernel_output.work_unit());
                    assert_eq!(file_output.lineage().parents, [kernel_output.output()]);
                    assert_eq!(
                        file_output.lineage().named_input_hashes,
                        kernel_output.lineage().named_input_hashes
                    );
                    let [file, clean_value] = [(&owner, file_output), (&clean_owner, clean_file)]
                        .map(|(current, output)| {
                            let handle = current
                                .storage()
                                .typed_handle::<mizar_artifact::store::CanonicalJson>(
                                    output,
                                    output.output_kind(),
                                )
                                .unwrap();
                            assert_eq!(
                                *current.storage().side_tables(&handle).unwrap(),
                                mizar_ir::storage::IrSideTables::default()
                            );
                            current.storage().get(&handle).unwrap()
                        });
                    assert_eq!(*file, expected_file.1);
                    assert_eq!(file, clean_value);
                    assert_eq!(file_output.lineage(), clean_file.lineage());
                    assert_eq!(file_output.placement(), clean_file.placement());
                    owner
                        .validate_published_output(snapshot, file_output)
                        .unwrap();
                    let [artifact_row] = run.phase_results[&artifact.id].as_slice() else {
                        panic!("one genuine Artifact row")
                    };
                    assert_eq!(artifact_row.status, PhaseStatus::Complete);
                    assert!(artifact_row.diagnostics.is_empty());
                    let [artifact_output] = artifact_row.output_refs.as_slice() else {
                        panic!("one ordinary Artifact DATA output")
                    };
                    let clean_artifact = &clean_run.phase_results[&artifact.id][0].output_refs[0];
                    let [value, clean_value] =
                        [(&owner, artifact_output), (&clean_owner, clean_artifact)].map(
                            |(current, output)| {
                                let handle = current
                                    .storage()
                                    .typed_handle::<VerifiedArtifact>(output, output.output_kind())
                                    .unwrap();
                                current.storage().get(&handle).unwrap()
                            },
                        );
                    assert_eq!(value, clean_value);
                    let summary_output = &check_results[0].output_refs[0];
                    let parents = [
                        summary_output,
                        core_output,
                        vc_output,
                        output,
                        kernel_output,
                        file_output,
                    ];
                    let artifact_input = PhaseInput::new(
                        artifact.unit.clone(),
                        PhaseDispatchInputBundle::new(
                            snapshot,
                            source_key(version),
                            vec![],
                            parents
                                .into_iter()
                                .map(|parent| {
                                    SealedParentOutputHandle::from_published_output(
                                        &owner,
                                        snapshot,
                                        parent.clone(),
                                    )
                                    .unwrap()
                                })
                                .collect(),
                        )
                        .unwrap(),
                    );
                    assert_eq!(
                        artifact_output.output_kind(),
                        &OutputKind::new("SourceTheoremArtifactData")
                    );
                    let mut parent_ids = parents.map(AnyPhaseOutputRef::output);
                    parent_ids
                        .sort_by(|left, right| left.hash().as_bytes().cmp(right.hash().as_bytes()));
                    assert_eq!(artifact_output.lineage().parents, parent_ids);
                    assert_eq!(
                        artifact_output.lineage().named_input_hashes,
                        core_output.lineage().named_input_hashes
                    );
                    assert!(matches!(
                        artifact_output.placement(),
                        StoragePlacement::Blob { .. }
                    ));
                    owner
                        .validate_current_output(snapshot, artifact_output)
                        .unwrap();
                    let cache = driver
                        .registry()
                        .cache_key_for_phase(PipelinePhase::ArtifactCommit, &artifact_input)
                        .unwrap();
                    assert!(
                        matches!(&cache.intent, PhaseCacheIntent::NoKey { reason } if !reason.is_empty())
                    );
                    assert_eq!(
                        cache,
                        clean_driver
                            .registry()
                            .cache_key_for_phase(PipelinePhase::ArtifactCommit, &artifact_input)
                            .unwrap()
                    );
                    let probe = driver
                        .registry()
                        .execute_phase_with_resources(
                            PipelinePhase::ArtifactCommit,
                            artifact_input.clone(),
                            PhaseExecutionResources::default(),
                        )
                        .unwrap();
                    assert_eq!(probe.result.status, PhaseStatus::Blocking);
                    assert!(probe.result.output_refs.is_empty());
                    assert_eq!(artifact_row.cache_observation, Some(probe.observation));
                    let (source_inputs, _) = native_disk_lookup_context(&actual, &ids);
                    let baseline_root = fixture
                        .root
                        .join(format!("alpha/same-artifact-baseline-{generation}"));
                    let (draft, main_write, _) = source_inputs
                        .write_native_theorem_artifact_draft(
                            &artifact_input,
                            &owner,
                            *selected,
                            (profile.clone(), ConcreteFormat::Tptp),
                            (&policy, &verifier),
                            (&witness_path, "main.mizir.json", &baseline_root),
                        )
                        .unwrap()
                        .unwrap();
                    assert_eq!(*value, draft.artifact);
                    assert_eq!(draft.artifact.proof_witnesses, [expected_file.2]);
                    let main_bytes =
                        fs::read(baseline_root.join(main_write.path.as_str())).unwrap();
                    let domain =
                        artifact_hash_domain(VERIFIED_ARTIFACT_SCHEMA_FAMILY, value.schema_version);
                    let excluded = artifact_hash_excluded_paths();
                    let child = format!(
                        "{}-{}",
                        snapshot
                            .fingerprint()
                            .as_bytes()
                            .iter()
                            .map(|byte| format!("{byte:02x}"))
                            .collect::<String>(),
                        source_key(version)
                            .as_bytes()
                            .iter()
                            .map(|byte| format!("{byte:02x}"))
                            .collect::<String>()
                    );
                    let mut manifest_hashes = Vec::new();
                    for (parent, row) in [
                        ("same-artifact-actual", artifact_row),
                        (
                            "same-artifact-clean",
                            &clean_run.phase_results[&artifact.id][0],
                        ),
                    ] {
                        let root = fixture.root.join("alpha").join(parent).join(&child);
                        assert_eq!(
                            fs::read(root.join(main_write.path.as_str())).unwrap(),
                            main_bytes
                        );
                        assert_eq!(
                            fs::read(root.join(&witness_path)).unwrap(),
                            canonical_json_bytes(&file)
                        );
                        let main = read_published_artifact(
                            &root,
                            &main_write.path,
                            PublishedArtifactReadOptions {
                                artifact_hash_domain: Some(&domain),
                                hash_excluded_paths: &excluded,
                                expected_artifact_hash: Some(main_write.artifact_hash),
                            },
                        )
                        .unwrap();
                        assert_eq!(
                            read_verified_artifact(
                                &main.value,
                                VerifiedArtifactReadOptions {
                                    artifact_path: Some(main_write.path.as_str()),
                                    expected_module: Some(&value.module),
                                    expected_interface_hash: Some(value.interface_hash),
                                    expected_implementation_hash: Some(value.implementation_hash),
                                }
                            )
                            .unwrap(),
                            *value
                        );
                        let read = read_manifest_file(
                            &root,
                            ManifestFileReadOptions {
                                validate_references: true,
                                validate_witness_files: true,
                                validate_development_artifacts: true,
                                expected_manifest_hash: None,
                            },
                        )
                        .unwrap();
                        assert_eq!(read.manifest.artifact_root, format!("{parent}/{child}"));
                        let [entry] = read.manifest.modules.as_slice() else {
                            panic!("one genuine main module")
                        };
                        assert_eq!(entry.artifact_hash.digest, main_write.artifact_hash);
                        assert_eq!(entry.module, value.module);
                        let json = artifact_manifest_json(&read.manifest).unwrap();
                        assert_eq!(
                            fs::read(root.join("artifact-manifest.json")).unwrap(),
                            canonical_json_bytes(&json)
                        );
                        assert_eq!(
                            read.artifact_hash,
                            manifest_hash_domain(read.manifest.schema_version).hash(&json, &[])
                        );
                        let commit = row.artifact_commit.as_ref().expect("actual owner commit");
                        assert_eq!(commit.manifest(), &read.manifest);
                        assert_eq!(commit.write().path, read.path);
                        assert_eq!(commit.write().artifact_hash, read.artifact_hash);
                        let mut transaction =
                            ManifestTransaction::begin(&root, read.manifest.clone(), None).unwrap();
                        transaction.stage_module(entry.clone()).unwrap();
                        assert_eq!(
                            commit,
                            &transaction
                                .commit(ManifestCommitOptions::default())
                                .unwrap()
                        );
                        manifest_hashes.push(read.artifact_hash);
                    }
                    assert_ne!(manifest_hashes[0], manifest_hashes[1]);
                    for (producer, submission, row, index) in [
                        (&driver, &actual, artifact_row, 0),
                        (
                            &clean_driver,
                            &clean,
                            &clean_run.phase_results[&artifact.id][0],
                            1,
                        ),
                    ] {
                        let commit = row.artifact_commit.as_ref().unwrap();
                        let identity = mizar_artifact::registration_summary::ArtifactHashRef::new(
                            mizar_artifact::registration_summary::ArtifactHashClass::Artifact,
                            mizar_artifact::manifest::MANIFEST_SCHEMA_FAMILY,
                            commit.manifest().schema_version,
                            manifest_hashes[index],
                        )
                        .to_artifact_hash_string();
                        let stream = producer.events(submission.session.id());
                        let boundaries = stream
                            .events()
                            .iter()
                            .filter(|event| {
                                matches!(event.kind, BuildEventKind::ArtifactBoundary { .. })
                            })
                            .collect::<Vec<_>>();
                        let [event] = boundaries.as_slice() else {
                            panic!("one genuine same-call owner boundary")
                        };
                        assert_eq!(
                            event.kind,
                            BuildEventKind::ArtifactBoundary {
                                committed: mizar_driver::events::OwnerRecordRef::new(
                                    mizar_driver::events::EventOwner::Artifact,
                                    identity.clone()
                                )
                            }
                        );
                        let finished = submission
                            .scheduler_run
                            .as_ref()
                            .unwrap()
                            .events
                            .iter()
                            .find(|event| {
                                event.task_id.as_ref() == Some(&artifact.id)
                                    && event.kind
                                        == mizar_build::scheduler::SchedulerEventKind::TaskFinished
                            })
                            .unwrap();
                        assert_eq!(
                            event.order,
                            mizar_driver::events::BuildEventOrderKey::new(60, 1)
                                .with_scheduler_order(finished.order)
                                .with_phase(PipelinePhase::ArtifactCommit)
                                .with_work_unit(artifact_output.work_unit().as_str())
                                .with_owner_order(identity)
                        );
                        assert_eq!(
                            event.identity,
                            mizar_driver::events::BuildEventIdentity {
                                session: submission.session.id(),
                                lane: submission.session.request().lane,
                                generation: submission.session.request().generation,
                                snapshot: Some(snapshot),
                                publication: mizar_driver::request::PublicationDecision::Current,
                            }
                        );
                        assert_eq!(producer.events(submission.session.id()), stream);
                    }
                    if stage == 3 {
                        let mut concrete = PhaseRegistryBuilder::new();
                        concrete
                            .register_source_load()
                            .register_frontend(vec![])
                            .register_source_theorem_module_resolver()
                            .register_source_theorem_signature_collection()
                            .register_source_theorem_type_checking()
                            .register_source_theorem_registration_resolution()
                            .register_source_theorem_overload_resolution()
                            .register_source_theorem_elaboration()
                            .register_source_theorem_algorithm_preparation()
                            .register_source_theorem_vc_generation(
                                generation_schema.clone(),
                                vc_schema.clone(),
                            )
                            .register_source_theorem_vc_discharge(policy.clone())
                            .register_source_theorem_atp_solve(
                                policy.clone(),
                                profile.clone(),
                                ConcreteFormat::Tptp,
                            )
                            .unwrap()
                            .register_source_theorem_candidate_check_with_witness_files(
                                policy.clone(),
                                profile.clone(),
                                ConcreteFormat::Tptp,
                                verifier.clone(),
                                "proof-witnesses/scheduled".into(),
                            )
                            .unwrap()
                            .register_source_theorem_artifact_commit(
                                policy.clone(),
                                profile.clone(),
                                ConcreteFormat::Tptp,
                                verifier.clone(),
                                (
                                    "proof-witnesses/scheduled".into(),
                                    "main.mizir.json".into(),
                                    "same-artifact-actual".into(),
                                ),
                            )
                            .unwrap();
                        let concrete =
                            std::sync::Arc::new(std::sync::Mutex::new(concrete.build().unwrap()));
                        let genuine = run
                            .phase_results
                            .values()
                            .flatten()
                            .flat_map(|row| row.output_refs.iter())
                            .collect::<Vec<_>>();
                        let routes = graph
                            .tasks()
                            .iter()
                            .filter(|task| task.unit == artifact.unit)
                            .map(|task| {
                                let first = &run.phase_results[&task.id][0].output_refs[0];
                                let parents = first
                                    .lineage()
                                    .parents
                                    .iter()
                                    .map(|id| {
                                        let parent = genuine
                                            .iter()
                                            .find(|parent| parent.output() == *id)
                                            .unwrap();
                                        SealedParentOutputHandle::from_published_output(
                                            &owner,
                                            snapshot,
                                            (**parent).clone(),
                                        )
                                        .unwrap()
                                    })
                                    .collect();
                                (
                                    task.phases[0],
                                    PhaseDispatchInputBundle::new(
                                        snapshot,
                                        source_key(version),
                                        vec![],
                                        parents,
                                    )
                                    .unwrap(),
                                )
                            })
                            .collect::<Vec<_>>();
                        let run_replay_case = |case: &str| {
                            let mut registry = PhaseRegistryBuilder::new();
                            for task in graph
                                .tasks()
                                .iter()
                                .filter(|task| task.unit == artifact.unit)
                            {
                                for (phase, row) in
                                    task.phases.iter().zip(&run.phase_results[&task.id])
                                {
                                    let mut replay = row.clone();
                                    if *phase == PipelinePhase::ArtifactCommit {
                                        if matches!(case, "data-only" | "wrong-phase") {
                                            replay.artifact_commit = None;
                                        }
                                        if case == "no-data" {
                                            replay.output_refs.clear();
                                        }
                                        if case == "blocking" {
                                            replay.status = PhaseStatus::Blocking;
                                        }
                                    } else if *phase == PipelinePhase::VcGenerate
                                        && case == "wrong-phase"
                                    {
                                        replay.artifact_commit =
                                            artifact_row.artifact_commit.clone();
                                    }
                                    let descriptor = concrete
                                        .lock()
                                        .unwrap()
                                        .descriptor_for_phase(*phase)
                                        .unwrap()
                                        .clone();
                                    registry.register(ScheduledArtifactServiceForwarder {
                                        registry: concrete.clone(),
                                        descriptor,
                                        replay: Some(replay),
                                    });
                                }
                            }
                            let mut replay_driver = CompilerDriver::new(registry.build().unwrap())
                                .with_output_publisher(owner.clone());
                            let (_, mut replay_input) = fixture.submit_request(vec![], vec![]);
                            replay_input.dependency_overlay =
                                ModuleDependencyOverlay::complete(vec![]);
                            assert!(replay_input.vc_descriptors.is_empty());
                            replay_input.phase_dispatch_inputs =
                                Some(Box::new(RetainedArtifactInputs {
                                    unit: artifact.unit.clone(),
                                    inputs: routes.clone(),
                                }));
                            if case == "stale" {
                                owner.mark_obsolete(snapshot).unwrap();
                            }
                            let replay = replay_driver
                                .submit_captured(
                                    replay_session.as_ref().unwrap().clone(),
                                    &ids,
                                    &snapshots,
                                    replay_input,
                                )
                                .unwrap();
                            if case == "stale" {
                                assert_eq!(
                                    replay.session.state(),
                                    BuildSessionState::Finished(BuildSessionOutcome::Failed)
                                );
                                assert!(
                                    replay
                                        .scheduler_run
                                        .as_ref()
                                        .unwrap()
                                        .phase_results
                                        .is_empty()
                                );
                                assert!(
                                    replay_driver
                                        .events(replay.session.id())
                                        .events()
                                        .iter()
                                        .all(|event| !matches!(
                                            event.kind,
                                            BuildEventKind::ArtifactBoundary { .. }
                                        ))
                                );
                                owner.register_current_snapshot(snapshot);
                                return;
                            }
                            assert!(replay.missing_services.is_empty());
                            assert!(replay.dispatch_gap_phases.is_empty());
                            assert_eq!(replay.status, DriverSubmissionStatus::SchedulerValidated);
                            assert_eq!(
                                replay.session.state(),
                                BuildSessionState::Finished(if case == "blocking" {
                                    BuildSessionOutcome::Blocked
                                } else {
                                    BuildSessionOutcome::Succeeded
                                })
                            );
                            let replay_graph = replay.task_graph.as_ref().unwrap();
                            assert!(
                                replay_graph
                                    .tasks()
                                    .iter()
                                    .all(|task| task.kind != TaskKind::KernelCheck)
                            );
                            let task = replay_graph
                                .tasks()
                                .iter()
                                .find(|task| task.kind == TaskKind::ArtifactCommit)
                                .unwrap();
                            let run = replay.scheduler_run.as_ref().unwrap();
                            let [row] = run.phase_results[&task.id].as_slice() else {
                                panic!("custom producer reached dispatcher acceptance")
                            };
                            assert_eq!(
                                row.status,
                                if case == "blocking" {
                                    PhaseStatus::Blocking
                                } else {
                                    PhaseStatus::Complete
                                }
                            );
                            assert!(row.diagnostics.is_empty());
                            assert_eq!(row.cache_observation, Some(probe.observation));
                            assert_eq!(
                                row.output_refs,
                                if matches!(case, "no-data" | "blocking") {
                                    vec![]
                                } else {
                                    artifact_row.output_refs.clone()
                                }
                            );
                            assert!(
                                run.phase_results
                                    .values()
                                    .flatten()
                                    .all(|row| row.artifact_commit.is_none()),
                                "{case}"
                            );
                            assert!(
                                replay_driver
                                    .events(replay.session.id())
                                    .events()
                                    .iter()
                                    .all(|event| !matches!(
                                        event.kind,
                                        BuildEventKind::ArtifactBoundary { .. }
                                    )),
                                "{case}"
                            );
                            owner
                                .validate_current_output(snapshot, artifact_output)
                                .unwrap();
                        };
                        run_replay_case("copied");
                        run_replay_case("data-only");
                        run_replay_case("no-data");
                        run_replay_case("blocking");
                        run_replay_case("wrong-phase");
                        run_replay_case("stale");
                    }
                    stale_outputs.push(artifact_output.clone());
                    stale_outputs.push(file_output.clone());
                    stale_outputs.push(kernel_output.clone());
                } else {
                    let atp = graph
                        .tasks()
                        .iter()
                        .find(|task| task.kind == TaskKind::AtpSolve && task.unit == discharge.unit)
                        .unwrap();
                    assert_eq!(atp.phases, [PipelinePhase::AtpSolve]);
                    assert!(atp.dependencies.contains(&discharge.id));
                    assert_eq!(
                        run.task_states
                            .iter()
                            .find(|state| state.task_id == atp.id)
                            .unwrap()
                            .state,
                        TaskState::Completed
                    );
                    let [atp_row] = run.phase_results[&atp.id].as_slice() else {
                        panic!("one actual ATP row")
                    };
                    assert_eq!(atp_row.status, PhaseStatus::Complete);
                    assert!(atp_row.diagnostics.is_empty());
                    let [atp_output] = atp_row.output_refs.as_slice() else {
                        panic!("one genuine ATP output")
                    };
                    let clean_atp = &clean_run.phase_results[&atp.id][0].output_refs[0];
                    assert_eq!(atp_output.phase(), &IrPipelinePhase::new("AtpSolve"));
                    assert_eq!(
                        atp_output.output_kind(),
                        &OutputKind::new("SourceTheoremCandidateInputs")
                    );
                    assert_eq!(
                        atp_output.schema_version(),
                        mizar_ir::storage::SchemaVersion::new(1)
                    );
                    assert_eq!(atp_output.snapshot(), snapshot);
                    assert_eq!(atp_output.work_unit(), core_output.work_unit());
                    assert_eq!(
                        atp_output.lineage().named_input_hashes,
                        core_output.lineage().named_input_hashes
                    );
                    assert!(matches!(
                        atp_output.placement(),
                        StoragePlacement::Blob { .. }
                    ));
                    assert_eq!(
                        owner.storage().side_tables_by_ref(atp_output).unwrap(),
                        clean_owner.storage().side_tables_by_ref(clean_atp).unwrap()
                    );
                    let expected_pair = mizar_atp::backend::build_source_theorem_candidate_inputs(
                        &core,
                        &vc_set,
                        *selected,
                        &value,
                        &policy,
                        profile.clone(),
                        ConcreteFormat::Tptp,
                    )
                    .unwrap();
                    let [pair, clean_pair] = [(&owner, atp_output), (&clean_owner, clean_atp)].map(
                        |(current, output)| {
                            let handle = current
                                .storage()
                                .typed_handle::<(EncodedBackendProblem, BackendCandidateEvidence)>(
                                    output,
                                    output.output_kind(),
                                )
                                .unwrap();
                            current.storage().get(&handle).unwrap()
                        },
                    );
                    assert_eq!(pair, clean_pair);
                    assert_eq!(*pair, expected_pair);
                    assert_eq!(pair.0.logic_profile_name(), "step6c99-explicit-atp");
                    assert_eq!(pair.0.concrete_format(), ConcreteFormat::Tptp);
                    assert_eq!(pair.1.target_binding(), pair.0.target_binding());
                    let handoff = mizar_vc::kernel_evidence_handoff::build_source_theorem_discharge_kernel_handoff(&core, &vc_set, *selected, &value, &policy).unwrap();
                    let target = handoff.canonical_evidence().target_vc();
                    assert_eq!(
                        pair.0.target_binding().fingerprint(),
                        &mizar_atp::problem::AtpFingerprint::new(
                            target.algorithm_id,
                            target.digest.clone()
                        )
                        .unwrap()
                    );
                    let mut parent_ids =
                        vec![core_output.output(), vc_output.output(), output.output()];
                    parent_ids
                        .sort_by(|left, right| left.hash().as_bytes().cmp(right.hash().as_bytes()));
                    assert_eq!(atp_output.lineage().parents, parent_ids);
                    let parents = [core_output, vc_output, output]
                        .into_iter()
                        .map(|parent| {
                            SealedParentOutputHandle::from_published_output(
                                &owner,
                                snapshot,
                                parent.clone(),
                            )
                            .unwrap()
                        })
                        .collect();
                    let query = PhaseInput::new(
                        atp.unit.clone(),
                        PhaseDispatchInputBundle::new(
                            snapshot,
                            source_key(version),
                            vec![],
                            parents,
                        )
                        .unwrap(),
                    );
                    assert_eq!(
                        query
                            .parent_outputs()
                            .iter()
                            .map(SealedParentOutputHandle::output)
                            .collect::<Vec<_>>(),
                        parent_ids
                    );
                    assert_eq!(
                        query.identities().parent_output_hashes(),
                        parent_ids
                            .iter()
                            .map(|parent| parent.hash())
                            .collect::<Vec<_>>()
                    );
                    let cache = driver
                        .registry()
                        .cache_key_for_phase(PipelinePhase::AtpSolve, &query)
                        .unwrap();
                    assert!(
                        matches!(&cache.intent, PhaseCacheIntent::NoKey { reason } if !reason.is_empty())
                    );
                    assert_eq!(
                        cache,
                        clean_driver
                            .registry()
                            .cache_key_for_phase(PipelinePhase::AtpSolve, &query)
                            .unwrap()
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
                    assert_eq!(atp_row.cache_observation, Some(probe.observation));
                    let [backend, kernel] = [TaskKind::BackendRun, TaskKind::KernelCheck]
                        .map(|kind| graph.tasks().iter().find(|task| task.kind == kind).unwrap());
                    assert!(backend.dependencies.contains(&atp.id));
                    assert!(kernel.dependencies.contains(&backend.id));
                    assert!(artifact.dependencies.contains(&atp.id));
                    for task in [backend, kernel] {
                        assert_eq!(
                            run.task_states
                                .iter()
                                .find(|state| state.task_id == task.id)
                                .unwrap()
                                .state,
                            TaskState::Blocked
                        );
                        assert!(!run.phase_results.contains_key(&task.id));
                        assert!(artifact.dependencies.contains(&task.id));
                    }
                    assert!(
                        run.phase_results
                            .values()
                            .flatten()
                            .flat_map(|row| &row.output_refs)
                            .all(|output| ![
                                OutputKind::new("SourceTheoremCandidateWitnessFile"),
                                OutputKind::new("SourceTheoremArtifactData")
                            ]
                            .contains(output.output_kind()))
                    );
                    stale_outputs.push(atp_output.clone());
                }
            } else {
                let vc = graph
                    .tasks()
                    .iter()
                    .find(|candidate| {
                        candidate.kind == TaskKind::VcGenerate
                            && matches!(&candidate.unit, WorkUnit::Module { module: id } if id.path.as_str() == module)
                    })
                    .unwrap();
                assert!(!run.phase_results.contains_key(&vc.id));
                let artifact = graph
                    .tasks()
                    .iter()
                    .find(|task| task.kind == TaskKind::ArtifactCommit && task.unit == vc.unit)
                    .unwrap();
                assert!(!run.phase_results.contains_key(&artifact.id));
            }
            if module == "main" {
                let stale_outputs = if stale_outputs.is_empty() {
                    vec![parent.unwrap()]
                } else {
                    stale_outputs
                };
                previous = Some((owner.clone(), stale_outputs, actual.session.clone(), hashes));
            }
        }
        for (producer, submission) in [(&driver, &actual), (&clean_driver, &clean)] {
            assert_eq!(
                producer
                    .events(submission.session.id())
                    .events()
                    .iter()
                    .filter(|event| matches!(event.kind, BuildEventKind::ArtifactBoundary { .. }))
                    .count(),
                usize::from(backend_free && stage >= 3)
            );
        }
        if backend_free && stage == 6 {
            let recorded = driver.events(actual.session.id());
            let artifact = graph
                .tasks()
                .iter()
                .find(|task| task.kind == TaskKind::ArtifactCommit && task.unit == units[0])
                .unwrap();
            let row = &run.phase_results[&artifact.id][0];
            let commit = row.artifact_commit.as_ref().unwrap();
            let root = fixture
                .root
                .join("alpha")
                .join(&commit.manifest().artifact_root);
            let path = root.join(commit.write().path.as_str());
            let bytes = fs::read(&path).unwrap();
            fs::write(&path, b"{").unwrap();
            assert!(mizar_proof::witness_store::CommittedWitnessPublicationProof::from_source_theorem_manifest_commit(commit, &root, &commit.manifest().modules[0].module, snapshot.fingerprint()).is_err());
            assert_eq!(driver.events(actual.session.id()), recorded);
            fs::write(path, bytes).unwrap();
            owner.mark_obsolete(snapshot).unwrap();
            assert_eq!(driver.events(actual.session.id()), recorded);
            assert!(
                owner
                    .validate_current_output(snapshot, &row.output_refs[0])
                    .is_err()
            );
            let handle = owner
                .storage()
                .typed_handle::<VerifiedArtifact>(
                    &row.output_refs[0],
                    row.output_refs[0].output_kind(),
                )
                .unwrap();
            assert_eq!(
                *owner.storage().get(&handle).unwrap(),
                read_verified_artifact(
                    &read_published_artifact(
                        &root,
                        &mizar_artifact::store::PublishedArtifactPath::new("main.mizir.json")
                            .unwrap(),
                        PublishedArtifactReadOptions::default()
                    )
                    .unwrap()
                    .value,
                    VerifiedArtifactReadOptions::default(),
                )
                .unwrap()
            );
            let (mut next, mut next_input) = fixture.submit_request(vec![], vec![]);
            next.origin = BuildRequestOrigin::Watch(mizar_driver::request::WatchRequest {
                watch_root: next.workspace_root.clone(),
                changed_paths: vec![fixture.version.normalized_path.clone()],
            });
            next.lane = actual.session.request().lane;
            next.generation =
                BuildRequestGeneration::new(actual.session.request().generation.get() + 1);
            next.source_inputs.versions =
                actual.session.captured().snapshot().source_versions.clone();
            next_input.source_layout = StaticSourceLayout::new(vec![WorkspaceSourcePackage {
                package_id: PackageId::new("alpha"),
                files: vec![],
            }]);
            let changed = driver
                .submit_watch_change(
                    next,
                    &ids,
                    &snapshots,
                    next_input,
                    mizar_driver::driver::WatchSubmitControl {
                        previous_session: Some(actual.session.id()),
                        output_publisher: Some(&owner),
                        file_watcher: mizar_driver::driver::WatchOwnerSeam::OwnerProvided,
                        lsp_bridge: mizar_driver::driver::WatchOwnerSeam::OwnerProvided,
                    },
                )
                .unwrap();
            assert_eq!(
                changed.superseded.as_ref().unwrap().session,
                actual.session.id()
            );
            assert_eq!(
                driver.session(actual.session.id()).unwrap().state(),
                BuildSessionState::Finished(BuildSessionOutcome::Superseded)
            );
            let suppressed = driver.events(actual.session.id());
            assert!(
                suppressed
                    .events()
                    .iter()
                    .all(|event| !matches!(event.kind, BuildEventKind::ArtifactBoundary { .. }))
            );
            assert!(
                suppressed
                    .events()
                    .iter()
                    .any(|event| matches!(event.kind, BuildEventKind::PublicationSuppressed))
            );
            assert!(
                recorded
                    .events()
                    .iter()
                    .any(|event| matches!(event.kind, BuildEventKind::ArtifactBoundary { .. }))
            );
            assert_eq!(driver.events(actual.session.id()), suppressed);
        }
    }
}
