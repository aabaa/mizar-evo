use super::*;

fn run_reflexivity_module(
    text: &str,
    module_name: &str,
) -> (
    Fixture,
    InMemorySessionIdAllocator,
    mizar_driver::driver::BuildSubmission,
    std::sync::Arc<PhaseOutputPublisher>,
) {
    use mizar_build::task_graph::{
        EvidenceCandidateId, VcOrderKey, VcTaskDescriptor, VcTaskDescriptorId,
    };
    let fixture = Fixture::new(text.as_bytes());
    let path = format!("src/{module_name}.miz");
    fs::write(fixture.root.join("alpha").join(&path), text).unwrap();
    let ids = InMemorySessionIdAllocator::new();
    let snapshots = SnapshotRegistry::new();
    let (mut draft, mut submit) = fixture.submit_request(vec![], vec![]);
    draft.source_inputs.versions.clear();
    submit.source_layout = StaticSourceLayout::new(vec![WorkspaceSourcePackage {
        package_id: PackageId::new("alpha"),
        files: vec![WorkspaceSourceFile::new(
            &path,
            format!("{module_name}.miz"),
        )],
    }]);
    let source = SourceInput {
        package_id: PackageId::new("alpha"),
        module_path: ModulePath::new(module_name),
        normalized_path: normalize_path(&fixture.root.join("alpha"), Path::new(&path)).unwrap(),
        edition: Edition::new("2025"),
        origin: SourceOriginInput::Disk { path: path.into() },
    };
    let session = draft
        .capture_sources(
            &ids,
            &snapshots,
            vec![(DiskSourceLoader::new(fixture.root.join("alpha")), source)],
        )
        .unwrap();
    let module = mizar_build::module_index::ModuleId::new(
        PackageId::new("alpha"),
        ModulePath::new(module_name),
    );
    let unit = WorkUnit::Module {
        module: module.clone(),
    };
    let owner = publisher(session.captured().snapshot().id, 0, &unit);
    allow_frontend(&owner, &unit);
    for (phase, kind) in [
        ("ModuleResolve", "SourceTheoremModuleCandidates"),
        ("SignatureCollection", "SourceTheoremModuleSummary"),
        ("TypeChecking", "SourceTheoremTypedAst"),
        ("RegistrationResolution", "SourceTheoremRegistrationClosure"),
        ("OverloadResolution", "SourceTheoremMetadata"),
        ("Elaboration", "SourceTheoremCore"),
        ("AlgorithmPreparation", "SourceTheoremPreparation"),
        ("VcGenerate", "SourceTheoremVC"),
        ("KernelCheck", "SourceTheoremReflexivityKernelStatus"),
    ] {
        owner.allow_work_unit(AllowedWorkUnit::new(
            IrPipelinePhase::new(phase),
            OutputKind::new(kind),
            IrWorkUnit::new(mizar_session::module_work_unit_label("alpha", module_name)),
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
        .register_source_theorem_vc_generation("source-theorem-local-v1".into(), "vc-v1".into())
        .register_source_theorem_reflexivity_kernel_check(
            mizar_proof::policy::VerifierPolicy::release(),
        );
    submit.vc_descriptors = vec![VcTaskDescriptor::new(
        VcTaskDescriptorId::new("module-status"),
        module,
        VcOrderKey::new("opaque"),
        vec![],
        vec![EvidenceCandidateId::new("opaque")],
    )];
    let submission = CompilerDriver::new(builder.build().unwrap())
        .with_output_publisher(owner.clone())
        .submit_captured(session, &ids, &snapshots, submit, Default::default())
        .unwrap();
    (fixture, ids, submission, owner)
}

fn prepare_genuine_reflexivity_status(
    inputs: SourceLoadInputs<'_>,
    old: &PhaseInput,
    owner: &std::sync::Arc<PhaseOutputPublisher>,
    policy: mizar_proof::policy::VerifierPolicy,
    allow_status_output: bool,
    generation_schema: &str,
    vc_schema: &str,
) -> Result<Option<(mizar_driver::registry::PhaseRegistry, PhaseInput)>, String> {
    use mizar_build::task_graph::{EvidenceCandidateId, VcTaskDescriptorId};
    use mizar_vc::vc_ir::{GenerationSchemaVersion, VcSchemaVersion};

    for (phase, kind) in [
        ("SignatureCollection", "SourceTheoremModuleSummary"),
        ("TypeChecking", "SourceTheoremTypedAst"),
        ("RegistrationResolution", "SourceTheoremRegistrationClosure"),
        ("OverloadResolution", "SourceTheoremMetadata"),
        ("Elaboration", "SourceTheoremCore"),
        ("AlgorithmPreparation", "SourceTheoremPreparation"),
        ("VcGenerate", "SourceTheoremVC"),
    ] {
        owner.allow_work_unit(AllowedWorkUnit::new(
            IrPipelinePhase::new(phase),
            OutputKind::new(kind),
            IrWorkUnit::new("\"alpha\":\"main\""),
        ));
    }
    if allow_status_output {
        owner.allow_work_unit(AllowedWorkUnit::new(
            IrPipelinePhase::new("KernelCheck"),
            OutputKind::new("SourceTheoremReflexivityKernelStatus"),
            IrWorkUnit::new("\"alpha\":\"main\""),
        ));
    }
    let child = |parents: Vec<AnyPhaseOutputRef>| {
        PhaseInput::new(
            old.work_unit.clone(),
            PhaseDispatchInputBundle::new(
                old.snapshot,
                old.identities().input_hash(),
                vec![],
                parents
                    .into_iter()
                    .map(|parent| {
                        SealedParentOutputHandle::from_published_output(owner, old.snapshot, parent)
                            .unwrap()
                    })
                    .collect(),
            )
            .unwrap(),
        )
    };
    let Some(signature) = inputs.publish_native_theorem_module_summary(old, owner)? else {
        return Ok(None);
    };
    let Some(typed) =
        inputs.publish_native_theorem_typed_ast(&child(vec![signature.erase()]), owner)?
    else {
        return Ok(None);
    };
    let Some(registration) =
        inputs.publish_native_theorem_registration_closure(&child(vec![typed.erase()]), owner)?
    else {
        return Ok(None);
    };
    let Some(metadata) = inputs.publish_native_theorem_metadata(
        &child(vec![typed.erase(), registration.erase()]),
        owner,
    )?
    else {
        return Ok(None);
    };
    let Some(core) = inputs
        .publish_native_theorem_core_with_metadata_parent(&child(vec![metadata.erase()]), owner)?
    else {
        return Ok(None);
    };
    let Some(preparation) =
        inputs.publish_native_theorem_preparation(&child(vec![core.erase()]), owner)?
    else {
        return Ok(None);
    };
    let Some(vc) = inputs.publish_native_theorem_vcs_with_core_preparation_parents(
        &child(vec![core.erase(), preparation.erase()]),
        owner,
        &GenerationSchemaVersion::new(generation_schema),
        &VcSchemaVersion::new(vc_schema),
    )?
    else {
        return Ok(None);
    };
    let WorkUnit::Module { module } = &old.work_unit else {
        panic!("module input")
    };
    let input = PhaseInput::new(
        WorkUnit::EvidenceCandidate {
            module: module.clone(),
            descriptor: VcTaskDescriptorId::new("native-reflexivity"),
            evidence_candidate: EvidenceCandidateId::new("native-reflexivity"),
        },
        PhaseDispatchInputBundle::new(
            old.snapshot,
            old.identities().input_hash(),
            vec![],
            [core.erase(), preparation.erase(), vc.erase()]
                .into_iter()
                .map(|parent| {
                    SealedParentOutputHandle::from_published_output(owner, old.snapshot, parent)
                        .unwrap()
                })
                .collect(),
        )
        .unwrap(),
    );
    let mut builder = PhaseRegistryBuilder::new();
    let schema = format!(
        "mizar-proof/source-theorem-reflexivity-kernel-status/v1;verifier_policy={}",
        policy.policy_fingerprint().to_lower_hex(),
    );
    builder.register_source_theorem_reflexivity_kernel_check(policy);
    let registry = builder.build().unwrap();
    let descriptor = registry
        .descriptor_for_phase(PipelinePhase::KernelCheck)
        .unwrap();
    assert_eq!(
        descriptor.service_name,
        "SourceTheoremReflexivityKernelCheck"
    );
    assert_eq!(
        descriptor.output_kind,
        "SourceTheoremReflexivityKernelStatus"
    );
    assert_eq!(descriptor.schema_version, schema);
    Ok(Some((registry, input)))
}

fn execute_genuine_reflexivity_status(
    inputs: SourceLoadInputs<'_>,
    old: &PhaseInput,
    owner: &std::sync::Arc<PhaseOutputPublisher>,
    policy: mizar_proof::policy::VerifierPolicy,
) -> (
    mizar_driver::registry::PhaseExecutionQueryResult,
    PhaseInput,
) {
    let (registry, input) = prepare_genuine_reflexivity_status(
        inputs,
        old,
        owner,
        policy,
        true,
        "source-theorem-local-v1",
        "vc-v1",
    )
    .unwrap()
    .expect("supported native reflexivity fixture has genuine Core/Preparation/VC parents");
    let result = registry
        .execute_phase_with_resources(
            PipelinePhase::KernelCheck,
            input.clone(),
            PhaseExecutionResources {
                output_publisher: Some(owner.clone()),
                source_load: Some(inputs),
                ..Default::default()
            },
        )
        .unwrap();
    (result, input)
}

fn independent_reflexivity_status(
    fixture: &Fixture,
    inputs: SourceLoadInputs<'_>,
    old: &PhaseInput,
    owner: &std::sync::Arc<PhaseOutputPublisher>,
    output: &AnyPhaseOutputRef,
    policy: &mizar_proof::policy::VerifierPolicy,
) -> Result<
    (
        mizar_core::core_ir::CoreIr,
        mizar_vc::vc_ir::VcSet,
        mizar_kernel::checker::KernelCheckResult,
        mizar_proof::policy::PolicyDecision,
        mizar_proof::status::ProofStatusProjection,
        mizar_vc::dependency_slice::DependencySlice,
    ),
    String,
> {
    use mizar_frontend::{
        orchestration::Frontend,
        parsing::MizarParserSeam,
        source::{FrontendSourceLoader, source_unit_from_loaded},
    };
    use mizar_resolve::{
        names::{SourceVariableScopeInput, SourceVariableScopeResolver},
        symbols::{source_proof_inputs, source_theorem_module_original_inputs},
    };
    let captured = inputs.captured.unwrap();
    let (_, loaded, _) = &captured.native_sources().unwrap()[0];
    let source = source_unit_from_loaded(
        loaded.clone(),
        std::path::PathBuf::from(loaded.normalized_path.as_str()),
    );
    let package_root = fixture.root.join("alpha");
    let frontend = Frontend::new(
        FrontendSourceLoader::new(DiskSourceLoader::new(&package_root)),
        inputs.dependency_lexical_provider(&[]),
        MizarParserSeam,
    )
    .run_loaded(source)
    .unwrap();
    let ast = frontend.ast.as_ref().unwrap();
    let module = ResolverModuleId::new(PackageId::new("alpha"), ModulePath::new("main"));
    let (arena, env) = source_theorem_module_original_inputs(ast, &module)
        .unwrap()
        .unwrap();
    assert!(
        mizar_resolve::symbols::source_theorem_module_candidates(ast, &arena, &env)
            .unwrap()
            .is_some()
    );
    let scope = SourceVariableScopeResolver::resolve_proof_occurrences(
        SourceVariableScopeInput::new(ast, &module, &env),
    )
    .unwrap();
    let metadata =
        mizar_checker::type_checker::check_source_theorem_metadata(&arena, &env, &scope).unwrap();
    let (scope, labels, resolved) = source_proof_inputs(&arena, &env).unwrap();
    let check =
        mizar_checker::type_checker::SourceVariableSemanticsChecker::check_theorem_skeletons(
            &arena,
            &metadata.2,
            &scope,
            &env,
            &labels,
            &resolved,
            None,
        )
        .unwrap();
    let stored = owner
        .storage()
        .typed_handle::<mizar_vc::vc_ir::VcSet>(output, &OutputKind::new("SourceTheoremVC"))
        .unwrap();
    let vcs = owner.storage().get(&stored).unwrap();
    let vc = vcs
        .vcs()
        .first()
        .expect("genuine producer has an original VC");
    mizar_proof::status::check_source_theorem_reflexivity_with_status(
        &arena,
        &check,
        &env,
        old.snapshot,
        vc.id,
        policy,
    )
}

#[test]
fn native_theorem_reflexivity_kernel_service_matches_same_capture_proof() {
    use mizar_proof::{policy::CandidatePolicyClass, status::ArtifactStatusPublication};
    use mizar_vc::dependency_slice::DependencySliceCompleteness;
    let sources = [
        "theorem T: for X being set holds X = X proof let X be set; thus X = X; end;",
        "theorem Renamed: for Y being set holds Y = Y proof let Y be set; thus Y = Y; end;",
        "\n theorem Renamed: for Y being set holds Y = Y proof let Y be set; thus Y = Y; end; \n",
    ];
    for (open, threshold) in [(false, usize::MAX), (true, 0)] {
        for text in sources {
            let policy = mizar_proof::policy::VerifierPolicy::release();
            with_native_summary_parent(
                text,
                open,
                threshold,
                |fixture, inputs, old, owner, _, _| {
                    let (run, input) =
                        execute_genuine_reflexivity_status(inputs, old, owner, policy.clone());
                    assert_eq!(run.result.status, PhaseStatus::Complete);
                    let [output] = run.result.output_refs.as_slice() else {
                        panic!("one genuine sealed status output")
                    };
                    assert_eq!(
                        output.output_kind(),
                        &OutputKind::new("SourceTheoremReflexivityKernelStatus")
                    );
                    assert_eq!(
                        output.schema_version(),
                        mizar_ir::storage::SchemaVersion::new(1)
                    );
                    assert_eq!(output.snapshot(), old.snapshot);
                    assert_eq!(output.lineage().parents.len(), 3);
                    assert_eq!(output.lineage().named_input_hashes.len(), 1);
                    assert_eq!(output.lineage().named_input_hashes[0].name, "source");
                    let vc_parent = input
                        .parent_outputs()
                        .iter()
                        .find(|parent| {
                            parent.as_output_ref().output_kind()
                                == &OutputKind::new("SourceTheoremVC")
                        })
                        .unwrap();
                    let checked = owner
                        .storage()
                        .typed_handle::<(
                            mizar_core::core_ir::CoreIr,
                            mizar_vc::vc_ir::VcSet,
                            mizar_kernel::checker::KernelCheckResult,
                            mizar_proof::policy::PolicyDecision,
                            mizar_proof::status::ProofStatusProjection,
                            mizar_vc::dependency_slice::DependencySlice,
                        )>(output, output.output_kind())
                        .unwrap();
                    let checked = owner.storage().get(&checked).unwrap();
                    assert_eq!(
                        *checked,
                        independent_reflexivity_status(
                            fixture,
                            inputs,
                            old,
                            owner,
                            vc_parent.as_output_ref(),
                            &policy
                        )
                        .unwrap(),
                    );
                    assert_eq!(checked.3.class, CandidatePolicyClass::DischargedBuiltin);
                    assert!(checked.4.trusted_used_axioms().is_some());
                    assert_eq!(
                    checked.4.artifact_publication(),
                    ArtifactStatusPublication::ExternalDependencyGap(
                        mizar_proof::status::ArtifactPublicationGap::DischargedBuiltinWitnessUnsupported,
                    ),
                );
                    assert_eq!(
                        checked.5.completeness(),
                        DependencySliceCompleteness::IncompleteUncacheable
                    );
                    assert!(checked.5.requires_cache_miss());
                    assert!(checked.0.proofs().iter().all(|(_, proof)| proof.status
                        == mizar_core::core_ir::CoreProofStatus::PendingAutomaticProof));
                    assert!(
                        checked
                            .0
                            .obligation_seeds()
                            .iter()
                            .all(|(_, seed)| seed.status
                                == mizar_core::core_ir::ObligationSeedStatus::Active)
                    );
                    assert!(
                        checked
                            .1
                            .vcs()
                            .iter()
                            .all(|vc| vc.status == mizar_vc::vc_ir::VcStatus::Open)
                    );
                },
            );
        }
    }
}

#[test]
fn native_theorem_reflexivity_status_keeps_successfully_bound_outputs_origin_local() {
    let mut baselines = Vec::new();
    for (text, policy) in [
        (
            "theorem T: for X being set holds X = X proof let X be set; thus X = X; end;",
            mizar_proof::policy::VerifierPolicy::release(),
        ),
        (
            "theorem T: for Y being set holds Y = Y proof let Y be set; thus Y = Y; end;",
            mizar_proof::policy::VerifierPolicy::release().with_require_kernel_certificates(false),
        ),
    ] {
        with_native_summary_parent(text, false, usize::MAX, |_, inputs, old, owner, _, _| {
            let (run, _) = execute_genuine_reflexivity_status(inputs, old, owner, policy.clone());
            assert_eq!(run.result.status, PhaseStatus::Complete);
            let [output] = run.result.output_refs.as_slice() else {
                panic!("genuine baseline status")
            };
            let typed = owner
                .storage()
                .typed_handle::<(
                    mizar_core::core_ir::CoreIr,
                    mizar_vc::vc_ir::VcSet,
                    mizar_kernel::checker::KernelCheckResult,
                    mizar_proof::policy::PolicyDecision,
                    mizar_proof::status::ProofStatusProjection,
                    mizar_vc::dependency_slice::DependencySlice,
                )>(output, output.output_kind())
                .unwrap();
            baselines.push((
                old.snapshot,
                owner.clone(),
                output.clone(),
                owner.storage().get(&typed).unwrap().clone(),
                policy,
            ));
        });
    }
    let [
        (snapshot_a, owner_a, output_a, tuple_a, policy_a),
        (snapshot_b, owner_b, output_b, tuple_b, policy_b),
    ] = baselines.as_slice()
    else {
        panic!("two independently successful source/snapshot baselines")
    };
    assert_ne!(snapshot_a, snapshot_b);
    assert_ne!(output_a.content_hash(), output_b.content_hash());
    assert_ne!(policy_a.policy_fingerprint(), policy_b.policy_fingerprint());
    assert_ne!(
        output_a.lineage().named_input_hashes,
        output_b.lineage().named_input_hashes
    );
    assert_ne!(tuple_a.0, tuple_b.0);
    assert_eq!(tuple_a.0.module_id(), tuple_b.0.module_id());
    assert!(
        SealedParentOutputHandle::from_published_output(owner_a, *snapshot_b, output_a.clone(),)
            .is_err()
    );
    assert!(
        SealedParentOutputHandle::from_published_output(owner_b, *snapshot_a, output_b.clone(),)
            .is_err()
    );
    assert!(
        owner_a
            .validate_output(
                *snapshot_b,
                output_a,
                mizar_ir::publisher::OutputAuthorization::Published,
            )
            .is_err()
    );

    let text = "theorem T: for X being set holds X = X proof let X be set; thus X = X; end;";
    let mut module_outputs = Vec::new();
    for module_name in ["main", "other"] {
        let (_fixture, ids, submission, owner) = run_reflexivity_module(text, module_name);
        let (inputs, old) = native_disk_lookup_context(&submission, &ids);
        let rows = &submission.scheduler_run.as_ref().unwrap().phase_results;
        let kernel = submission
            .task_graph
            .as_ref()
            .unwrap()
            .tasks()
            .iter()
            .find(|task| task.kind == TaskKind::KernelCheck)
            .unwrap();
        let result = &rows[&kernel.id][0];
        assert_eq!(result.status, PhaseStatus::Complete);
        let output = &result.output_refs[0];
        assert_eq!(
            output.work_unit(),
            &IrWorkUnit::new(mizar_session::module_work_unit_label("alpha", module_name))
        );
        let parents = output
            .lineage()
            .parents
            .iter()
            .map(|id| {
                let original = rows
                    .values()
                    .flatten()
                    .flat_map(|row| &row.output_refs)
                    .find(|parent| parent.output() == *id)
                    .unwrap();
                SealedParentOutputHandle::from_published_output(
                    &owner,
                    old.snapshot,
                    original.clone(),
                )
                .unwrap()
            })
            .collect();
        let wrong = PhaseInput::new(
            WorkUnit::EvidenceCandidate {
                module: mizar_build::module_index::ModuleId::new(
                    PackageId::new("alpha"),
                    ModulePath::new(if module_name == "main" {
                        "other"
                    } else {
                        "main"
                    }),
                ),
                descriptor: mizar_build::task_graph::VcTaskDescriptorId::new("wrong-module"),
                evidence_candidate: mizar_build::task_graph::EvidenceCandidateId::new(
                    "wrong-module",
                ),
            },
            PhaseDispatchInputBundle::new(
                old.snapshot,
                old.identities().input_hash(),
                vec![],
                parents,
            )
            .unwrap(),
        );
        let mut builder = PhaseRegistryBuilder::new();
        builder.register_source_theorem_reflexivity_kernel_check(
            mizar_proof::policy::VerifierPolicy::release(),
        );
        let refused = builder
            .build()
            .unwrap()
            .execute_phase_with_resources(
                PipelinePhase::KernelCheck,
                wrong,
                PhaseExecutionResources {
                    output_publisher: Some(owner.clone()),
                    source_load: Some(inputs),
                    ..Default::default()
                },
            )
            .unwrap()
            .result;
        assert_eq!(refused.status, PhaseStatus::Blocking);
        assert!(refused.output_refs.is_empty());
        let produced = rows
            .values()
            .flatten()
            .map(|row| row.output_refs.len())
            .sum::<usize>();
        let collected = owner.storage().collect(mizar_ir::storage::CollectInput {
            snapshot: old.snapshot,
            protected_outputs: vec![],
        });
        assert_eq!(
            (collected.outputs_dropped, collected.abandoned_slots_dropped),
            (produced, 0)
        );
        module_outputs.push(output.clone());
    }
    assert_ne!(module_outputs[0].work_unit(), module_outputs[1].work_unit());
    assert_ne!(
        module_outputs[0].content_hash(),
        module_outputs[1].content_hash()
    );

    #[derive(Clone, Copy)]
    enum Swap {
        Dependency,
        Module,
        Schema,
        Missing,
        Extra,
        WrongRole,
    }
    for swap in [
        Swap::Dependency,
        Swap::Module,
        Swap::Schema,
        Swap::Missing,
        Swap::Extra,
        Swap::WrongRole,
    ] {
        let text = "theorem T: for X being set holds X = X proof let X be set; thus X = X; end;";
        with_native_summary_parent(text, false, 0, |_, inputs, old, owner, _, _| {
            let (registry, input) = prepare_genuine_reflexivity_status(
                inputs,
                old,
                owner,
                mizar_proof::policy::VerifierPolicy::release(),
                true,
                "source-theorem-local-v1",
                if matches!(swap, Swap::Schema) {
                    "foreign-vc-v1"
                } else {
                    "vc-v1"
                },
            )
            .unwrap()
            .expect("supported reflexivity source has genuine producer parents");
            let wrong = match swap {
                Swap::Dependency => PhaseInput::new(
                    input.work_unit.clone(),
                    PhaseDispatchInputBundle::new(
                        input.snapshot,
                        input.identities().input_hash(),
                        vec![Hash::from_bytes([0x51; 32])],
                        input.parent_outputs().to_vec(),
                    )
                    .unwrap(),
                ),
                Swap::Module => PhaseInput::new(
                    WorkUnit::EvidenceCandidate {
                        module: mizar_build::module_index::ModuleId::new(
                            PackageId::new("alpha"),
                            ModulePath::new("other"),
                        ),
                        descriptor: mizar_build::task_graph::VcTaskDescriptorId::new("other"),
                        evidence_candidate: mizar_build::task_graph::EvidenceCandidateId::new(
                            "other",
                        ),
                    },
                    PhaseDispatchInputBundle::new(
                        input.snapshot,
                        input.identities().input_hash(),
                        vec![],
                        input.parent_outputs().to_vec(),
                    )
                    .unwrap(),
                ),
                Swap::Missing | Swap::Extra | Swap::WrongRole => {
                    let baseline = registry
                        .execute_phase_with_resources(
                            PipelinePhase::KernelCheck,
                            input.clone(),
                            PhaseExecutionResources {
                                output_publisher: Some(owner.clone()),
                                source_load: Some(inputs),
                                ..Default::default()
                            },
                        )
                        .unwrap()
                        .result;
                    assert_eq!(baseline.status, PhaseStatus::Complete);
                    let status = SealedParentOutputHandle::from_published_output(
                        owner,
                        input.snapshot,
                        baseline.output_refs[0].clone(),
                    )
                    .unwrap();
                    let mut parents = input.parent_outputs().to_vec();
                    match swap {
                        Swap::Missing => {
                            parents.pop();
                        }
                        Swap::Extra => parents.push(status),
                        Swap::WrongRole => parents[0] = status,
                        _ => unreachable!(),
                    }
                    PhaseInput::new(
                        input.work_unit.clone(),
                        PhaseDispatchInputBundle::new(
                            input.snapshot,
                            input.identities().input_hash(),
                            vec![],
                            parents,
                        )
                        .unwrap(),
                    )
                }
                Swap::Schema => input,
            };
            let result = registry
                .execute_phase_with_resources(
                    PipelinePhase::KernelCheck,
                    wrong,
                    PhaseExecutionResources {
                        output_publisher: Some(owner.clone()),
                        source_load: Some(inputs),
                        ..Default::default()
                    },
                )
                .unwrap()
                .result;
            assert_eq!(result.status, PhaseStatus::Blocking);
            assert!(result.output_refs.is_empty());
            let collected = owner.storage().collect(mizar_ir::storage::CollectInput {
                snapshot: old.snapshot,
                protected_outputs: vec![],
            });
            assert_eq!(
                (collected.outputs_dropped, collected.abandoned_slots_dropped),
                (
                    if matches!(swap, Swap::Missing | Swap::Extra | Swap::WrongRole) {
                        11
                    } else {
                        10
                    },
                    0
                )
            );
            assert!(result.diagnostics.is_empty());
        });
    }
}

#[test]
fn native_theorem_reflexivity_service_preserves_disabled_policy_refusal() {
    use mizar_proof::policy::KernelEvidenceFormat;
    let text = "theorem T: for X being set holds X = X proof let X be set; thus X = X; end;";
    let policy = mizar_proof::policy::VerifierPolicy::release()
        .with_kernel_evidence_formats([] as [KernelEvidenceFormat; 0]);
    with_native_summary_parent(
        text,
        false,
        usize::MAX,
        |fixture, inputs, old, owner, _, _| {
            let (run, input) =
                execute_genuine_reflexivity_status(inputs, old, owner, policy.clone());
            assert_eq!(run.result.status, PhaseStatus::Blocking);
            assert!(run.result.output_refs.is_empty());
            assert!(run.result.diagnostics.is_empty());
            let vc_parent = input
                .parent_outputs()
                .iter()
                .find(|parent| {
                    parent.as_output_ref().output_kind() == &OutputKind::new("SourceTheoremVC")
                })
                .unwrap();
            assert_eq!(
                independent_reflexivity_status(
                    fixture,
                    inputs,
                    &input,
                    owner,
                    vc_parent.as_output_ref(),
                    &policy,
                )
                .unwrap_err(),
                "theorem.proof.kernel_policy_rejected"
            );
            let collected = owner.storage().collect(mizar_ir::storage::CollectInput {
                snapshot: old.snapshot,
                protected_outputs: vec![],
            });
            assert_eq!(
                (collected.outputs_dropped, collected.abandoned_slots_dropped),
                (10, 0)
            );
        },
    );
    for text in [
        "theorem T: for X being set holds X = {} proof let X be set; thus X = {}; end;",
        "theorem A: for X being set holds X = X proof let X be set; thus X = X; end; theorem B: for Y being set holds Y = Y proof let Y be set; thus Y = Y; end;",
        "theorem A: for X being set holds X = X proof let X be set; thus X = X; end; theorem B: for Y being set holds Y = Y proof let Y be set; thus Y = Y by A; end;",
        include_str!(
            "../../../../tests/miz/pass/theorems/pass_proof_verification_theorem_status_open_assumed_001.miz"
        ),
    ] {
        let supported = text.starts_with("theorem A:");
        let (fixture, ids, submission, owner) = run_reflexivity_module(text, "main");
        let (inputs, old) = native_disk_lookup_context(&submission, &ids);
        let module = mizar_build::module_index::ModuleId::new(
            PackageId::new("alpha"),
            ModulePath::new("main"),
        );
        let outputs = submission
            .scheduler_run
            .as_ref()
            .unwrap()
            .phase_results
            .values()
            .flatten()
            .flat_map(|row| &row.output_refs)
            .collect::<Vec<_>>();
        if let Some(vc) = outputs
            .iter()
            .find(|output| output.output_kind() == &OutputKind::new("SourceTheoremVC"))
        {
            let expected = independent_reflexivity_status(
                &fixture,
                inputs,
                &old,
                &owner,
                vc,
                &mizar_proof::policy::VerifierPolicy::release(),
            );
            if supported {
                let expected = expected.unwrap();
                assert_eq!(expected.1.vcs().len(), 2);
                assert_eq!(
                    expected.2.status(),
                    mizar_kernel::checker::KernelCheckStatus::Accepted
                );
                assert_eq!(
                    expected.4.projected_status(),
                    mizar_proof::status::ProjectedProofStatus::Accepted
                );
            } else {
                assert!(expected.is_err());
            }
        } else if let Some(metadata) = outputs
            .iter()
            .find(|output| output.output_kind() == &OutputKind::new("SourceTheoremMetadata"))
        {
            let raw_input = PhaseInput::new(
                old.work_unit.clone(),
                PhaseDispatchInputBundle::new(
                    old.snapshot,
                    old.identities().input_hash(),
                    vec![],
                    vec![
                        SealedParentOutputHandle::from_published_output(
                            &owner,
                            old.snapshot,
                            (*metadata).clone(),
                        )
                        .unwrap(),
                    ],
                )
                .unwrap(),
            );
            assert!(
                outputs
                    .iter()
                    .all(|output| output.output_kind() != &OutputKind::new("SourceTheoremCore"))
            );
            assert!(matches!(
                inputs.publish_native_theorem_core_with_metadata_parent(&raw_input, &owner),
                Ok(None) | Err(_)
            ));
        } else if let Some(signature) = outputs
            .iter()
            .find(|output| output.output_kind() == &OutputKind::new("SourceTheoremModuleSummary"))
        {
            assert!(outputs.iter().all(|output| output.output_kind() != &OutputKind::new("SourceTheoremTypedAst")));
            let raw_input = PhaseInput::new(
                old.work_unit.clone(),
                PhaseDispatchInputBundle::new(
                    old.snapshot,
                    old.identities().input_hash(),
                    vec![],
                    vec![
                        SealedParentOutputHandle::from_published_output(
                            &owner,
                            old.snapshot,
                            (*signature).clone(),
                        )
                        .unwrap(),
                    ],
                )
                .unwrap(),
            );
            assert!(matches!(
                inputs.publish_native_theorem_typed_ast(&raw_input, &owner),
                Ok(None) | Err(_)
            ));
        } else {
            assert!(
                inputs
                    .native_theorem_module_candidates(&module)
                    .unwrap()
                    .is_none()
            );
        }
        assert_eq!(
            outputs.iter().any(|output| output.output_kind()
                == &OutputKind::new("SourceTheoremReflexivityKernelStatus")),
            supported,
        );
        let produced = submission
            .scheduler_run
            .as_ref()
            .unwrap()
            .phase_results
            .values()
            .flatten()
            .map(|row| row.output_refs.len())
            .sum::<usize>();
        assert_eq!(
            owner
                .storage()
                .collect(mizar_ir::storage::CollectInput {
                    snapshot: old.snapshot,
                    protected_outputs: vec![],
                })
                .outputs_dropped,
            produced
        );
    }
}

#[test]
fn native_theorem_reflexivity_service_never_publishes_from_stale_or_unavailable_authority() {
    use mizar_build::cancel::{CancellationGeneration, CancellationReason, CancellationToken};
    #[derive(Clone, Copy)]
    enum Control {
        MatchingCancellation,
        ForeignCancellation,
        MissingPublisher,
        MissingCapture,
        ReplacedSnapshot,
        DeniedPublication,
        CollectedParents,
    }
    for control in [
        Control::MatchingCancellation,
        Control::ForeignCancellation,
        Control::MissingPublisher,
        Control::MissingCapture,
        Control::ReplacedSnapshot,
        Control::DeniedPublication,
        Control::CollectedParents,
    ] {
        let text = "theorem T: for X being set holds X = X proof let X be set; thus X = X; end;";
        with_native_summary_parent(text, false, 0, |_, inputs, old, owner, _, _| {
            let (registry, input) = prepare_genuine_reflexivity_status(
                inputs,
                old,
                owner,
                mizar_proof::policy::VerifierPolicy::release(),
                !matches!(control, Control::DeniedPublication),
                "source-theorem-local-v1",
                "vc-v1",
            )
            .unwrap()
            .expect("lifecycle fixture has genuine producer parents");
            if matches!(control, Control::ReplacedSnapshot) {
                owner
                    .replace_current_snapshot(old.snapshot, snapshot_id(0xc1))
                    .unwrap();
            }
            if matches!(control, Control::CollectedParents) {
                owner.storage().collect(mizar_ir::storage::CollectInput {
                    snapshot: old.snapshot,
                    protected_outputs: vec![],
                });
                assert!(
                    owner
                        .storage()
                        .validate_handle(input.parent_outputs()[0].as_output_ref())
                        .is_err()
                );
            }
            let cancellation = match control {
                Control::MatchingCancellation => Some(CancellationToken {
                    snapshot: input.snapshot,
                    generation: CancellationGeneration::default(),
                    reason: CancellationReason::ExplicitRequest,
                }),
                Control::ForeignCancellation => Some(CancellationToken {
                    snapshot: snapshot_id(0xc2),
                    generation: CancellationGeneration::default(),
                    reason: CancellationReason::ExplicitRequest,
                }),
                _ => None,
            };
            let result = registry
                .execute_phase_with_resources(
                    PipelinePhase::KernelCheck,
                    input,
                    PhaseExecutionResources {
                        cancellation,
                        output_publisher: (!matches!(control, Control::MissingPublisher))
                            .then(|| owner.clone()),
                        source_load: (!matches!(control, Control::MissingCapture))
                            .then_some(inputs),
                        ..Default::default()
                    },
                )
                .unwrap()
                .result;
            match control {
                Control::MatchingCancellation => assert_eq!(result.status, PhaseStatus::Cancelled),
                _ => assert_eq!(result.status, PhaseStatus::Blocking),
            }
            assert!(result.output_refs.is_empty());
            assert!(result.diagnostics.is_empty());
            let collected = owner.storage().collect(mizar_ir::storage::CollectInput {
                snapshot: old.snapshot,
                protected_outputs: vec![],
            });
            assert_eq!(
                (collected.outputs_dropped, collected.abandoned_slots_dropped),
                (
                    if matches!(control, Control::CollectedParents) {
                        0
                    } else {
                        10
                    },
                    usize::from(matches!(control, Control::DeniedPublication)),
                )
            );
        });
    }
    let text = "theorem T: for X being set holds X = X proof let X be set; thus X = X; end;";
    let (_fixture, ids, submission, owner) = run_reflexivity_module(text, "main");
    let (inputs, old) = native_disk_lookup_context(&submission, &ids);
    let rows = &submission.scheduler_run.as_ref().unwrap().phase_results;
    let kernel = submission
        .task_graph
        .as_ref()
        .unwrap()
        .tasks()
        .iter()
        .find(|task| task.kind == TaskKind::KernelCheck)
        .unwrap();
    let row = &rows[&kernel.id][0];
    assert_eq!(row.status, PhaseStatus::Complete);
    let output = &row.output_refs[0];
    let typed = owner
        .storage()
        .typed_handle::<(
            mizar_core::core_ir::CoreIr,
            mizar_vc::vc_ir::VcSet,
            mizar_kernel::checker::KernelCheckResult,
            mizar_proof::policy::PolicyDecision,
            mizar_proof::status::ProofStatusProjection,
            mizar_vc::dependency_slice::DependencySlice,
        )>(output, output.output_kind())
        .unwrap();
    let original = owner.storage().get(&typed).unwrap();
    let parents = output
        .lineage()
        .parents
        .iter()
        .map(|id| {
            let parent = rows
                .values()
                .flatten()
                .flat_map(|row| &row.output_refs)
                .find(|parent| parent.output() == *id)
                .unwrap();
            SealedParentOutputHandle::from_published_output(&owner, old.snapshot, parent.clone())
                .unwrap()
        })
        .collect();
    let input = PhaseInput::new(
        kernel.unit.clone(),
        PhaseDispatchInputBundle::new(old.snapshot, old.identities().input_hash(), vec![], parents)
            .unwrap(),
    );
    let collected = owner.storage().collect(mizar_ir::storage::CollectInput {
        snapshot: old.snapshot,
        protected_outputs: vec![output.output()],
    });
    assert_eq!(
        (collected.outputs_dropped, collected.protected_outputs),
        (10, 1)
    );
    assert_eq!(*owner.storage().get(&typed).unwrap(), *original);
    let mut builder = PhaseRegistryBuilder::new();
    builder.register_source_theorem_reflexivity_kernel_check(
        mizar_proof::policy::VerifierPolicy::release(),
    );
    let refused = builder
        .build()
        .unwrap()
        .execute_phase_with_resources(
            PipelinePhase::KernelCheck,
            input,
            PhaseExecutionResources {
                output_publisher: Some(owner.clone()),
                source_load: Some(inputs),
                ..Default::default()
            },
        )
        .unwrap()
        .result;
    assert_eq!(refused.status, PhaseStatus::Blocking);
    assert!(refused.output_refs.is_empty());
    assert_eq!(*owner.storage().get(&typed).unwrap(), *original);
    let collected = owner.storage().collect(mizar_ir::storage::CollectInput {
        snapshot: old.snapshot,
        protected_outputs: vec![output.output()],
    });
    assert_eq!(
        (
            collected.outputs_dropped,
            collected.protected_outputs,
            collected.abandoned_slots_dropped
        ),
        (0, 1, 0)
    );
}

#[test]
fn native_theorem_kernel_handoff_core_vc_caller_preserves_genuine_outputs() {
    use mizar_core::core_ir::{CoreProofStatus, ObligationSeedStatus};
    use mizar_vc::{
        kernel_evidence_handoff::{
            KernelGoalPolarity, build_source_theorem_kernel_handoff as handoff,
        },
        vc_ir::{GenerationSchemaVersion, SeedOriginRef, VcSchemaVersion, VcStatus},
    };
    for (open, threshold) in [(false, usize::MAX), (true, 0)] {
        let mut positions = std::collections::BTreeSet::new();
        for variant in 0..64 {
            let binder = if variant == 0 { "X" } else { "Renamed" };
            let text = format!(
                "theorem Placement{variant}: for {binder} being set holds {binder} = {binder} proof let {binder} be set; thus {binder} = {binder}; end;"
            );
            with_native_summary_parent(
                &text,
                open,
                threshold,
                |fixture, inputs, old, owner, _, (session, _, _)| {
                    for (phase, kind) in [
                        ("SignatureCollection", "SourceTheoremModuleSummary"),
                        ("TypeChecking", "SourceTheoremTypedAst"),
                        ("RegistrationResolution", "SourceTheoremRegistrationClosure"),
                        ("OverloadResolution", "SourceTheoremMetadata"),
                        ("Elaboration", "SourceTheoremCore"),
                        ("AlgorithmPreparation", "SourceTheoremPreparation"),
                        ("VcGenerate", "SourceTheoremVC"),
                    ] {
                        owner.allow_work_unit(AllowedWorkUnit::new(
                            IrPipelinePhase::new(phase),
                            OutputKind::new(kind),
                            IrWorkUnit::new("\"alpha\":\"main\""),
                        ));
                    }
                    let child = |parents: Vec<AnyPhaseOutputRef>| {
                        PhaseInput::new(
                            old.work_unit.clone(),
                            PhaseDispatchInputBundle::new(
                                old.snapshot,
                                old.identities().input_hash(),
                                vec![],
                                parents
                                    .into_iter()
                                    .map(|parent| {
                                        SealedParentOutputHandle::from_published_output(
                                            owner,
                                            old.snapshot,
                                            parent,
                                        )
                                        .unwrap()
                                    })
                                    .collect(),
                            )
                            .unwrap(),
                        )
                    };
                    let signature = inputs
                        .publish_native_theorem_module_summary(old, owner)
                        .unwrap()
                        .unwrap();
                    let type_input = child(vec![signature.erase()]);
                    let typed = inputs
                        .publish_native_theorem_typed_ast(&type_input, owner)
                        .unwrap()
                        .unwrap();
                    let reg_input = child(vec![typed.erase()]);
                    let registration = inputs
                        .publish_native_theorem_registration_closure(&reg_input, owner)
                        .unwrap()
                        .unwrap();
                    let meta_input = child(vec![typed.erase(), registration.erase()]);
                    let parent = inputs
                        .publish_native_theorem_metadata(&meta_input, owner)
                        .unwrap()
                        .unwrap();
                    let metadata_input = child(vec![parent.erase()]);
                    let core_parent = inputs
                        .publish_native_theorem_core_with_metadata_parent(&metadata_input, owner)
                        .unwrap()
                        .unwrap();
                    let core_input = child(vec![core_parent.erase()]);
                    let preparation = inputs
                        .publish_native_theorem_preparation(&core_input, owner)
                        .unwrap()
                        .unwrap();
                    assert!(
                        inputs
                            .native_theorem_preparation_with_core_parent(&core_input, owner)
                            .unwrap()
                            .is_some()
                    );
                    let generation =
                        GenerationSchemaVersion::new(if variant == 0 { " \n\t" } else { "" });
                    let schema = VcSchemaVersion::new(if variant == 0 { "" } else { "核\0" });
                    let vc_input = child(vec![core_parent.erase(), preparation.erase()]);
                    let vc_parent = inputs
                        .publish_native_theorem_vcs_with_core_preparation_parents(
                            &vc_input,
                            owner,
                            &generation,
                            &schema,
                        )
                        .unwrap()
                        .unwrap();
                    let input = child(vec![vc_parent.erase(), core_parent.erase()]);
                    positions.insert(
                        input
                            .parent_outputs()
                            .iter()
                            .position(|parent| parent.output() == core_parent.output())
                            .unwrap(),
                    );
                    let core = owner.storage().get(&core_parent).unwrap();
                    let vcs = owner.storage().get(&vc_parent).unwrap();
                    let selected = vcs.vcs()[0].id;
                    let vc = vcs.vc(selected).unwrap();
                    let expected = handoff(&core, &vcs, selected).unwrap();
                    assert_eq!(
                        core.source_id(),
                        inputs.snapshot.source_versions[0].source_id
                    );
                    assert_eq!(
                        core.module_id(),
                        &ResolverModuleId::new(PackageId::new("alpha"), ModulePath::new("main"))
                    );
                    assert_eq!(vcs.source(), core.source_id());
                    assert_eq!(vcs.module().as_str(), "package=5:alpha;module=4:main");
                    assert_eq!(vcs.snapshot(), old.snapshot);
                    assert_eq!(vcs.schema_version(), &schema);
                    assert!(
                        !vcs.vcs().is_empty()
                            && !core.proofs().is_empty()
                            && !vcs.seed_accounting().is_empty()
                    );
                    assert!(vcs.vcs().iter().all(|vc| vc.status == VcStatus::Open
                        && vc.anchor.generation_schema_version == generation));
                    assert!(
                        core.proofs().iter().all(
                            |(_, proof)| proof.status == CoreProofStatus::PendingAutomaticProof
                        )
                    );
                    for row in vcs.seed_accounting() {
                        let SeedOriginRef::ExistingCore { seed } = row.origin else {
                            panic!("Core seed")
                        };
                        assert_eq!(row.seed_status, ObligationSeedStatus::Active);
                        assert_eq!(
                            core.obligation_seeds().get(seed).unwrap().status,
                            ObligationSeedStatus::Active
                        );
                    }
                    assert_eq!(expected.diagnostics().vc, selected);
                    assert!(expected.context_identity_is_consistent());
                    let envelope = expected.canonical_evidence();
                    assert_eq!(envelope.symbol_manifest().len(), 2);
                    assert_eq!(envelope.variable_manifest().len(), 1);
                    assert!(
                        envelope
                            .symbol_manifest()
                            .iter()
                            .chain(envelope.variable_manifest())
                            .all(|entry| !entry.payload.is_empty())
                    );
                    let local = vc
                        .local_context
                        .entries()
                        .iter()
                        .filter_map(|entry| entry.formula)
                        .collect::<Vec<_>>();
                    assert!(!local.is_empty());
                    assert_eq!(
                        envelope
                            .formula_evidence()
                            .iter()
                            .map(|entry| entry.producer_formula_ref().unwrap())
                            .collect::<Vec<_>>(),
                        local
                    );
                    assert_eq!(
                        expected
                            .context_identity()
                            .entries()
                            .iter()
                            .map(|entry| entry.producer_formula_ref())
                            .collect::<Vec<_>>(),
                        local
                    );
                    assert_eq!(envelope.final_goal().producer_formula_ref, vc.goal);
                    assert_eq!(
                        envelope.final_goal().polarity,
                        KernelGoalPolarity::AssertFalseForRefutation
                    );
                    assert!(!envelope.final_goal().formula_bytes.is_empty());
                    assert_eq!(
                        core_parent.placement() == &StoragePlacement::Resident,
                        !open
                    );
                    assert_eq!(vc_parent.placement() == &StoragePlacement::Resident, !open);
                    let before = (
                        input.clone(),
                        (*core).clone(),
                        (*vcs).clone(),
                        selected,
                        generation.clone(),
                        schema.clone(),
                        session.captured().clone(),
                    );
                    for stage in 0..3 {
                        if stage == 1 {
                            fixture.write(if open {
                                &[0xff]
                            } else {
                                b"open theorem Changed: 0 = 1;\n"
                            });
                        } else if stage == 2 {
                            fs::remove_file(fixture.root.join("alpha/src/main.miz")).unwrap();
                        }
                        for _ in 0..2 {
                            assert_eq!(
                                inputs
                                    .native_theorem_kernel_handoff_with_core_vc_parents(
                                        &input, owner, selected
                                    )
                                    .unwrap(),
                                Some(expected.clone())
                            );
                        }
                    }
                    assert_eq!(
                        (
                            input,
                            (*owner.storage().get(&core_parent).unwrap()).clone(),
                            (*owner.storage().get(&vc_parent).unwrap()).clone(),
                            selected,
                            generation,
                            schema,
                            session.captured().clone()
                        ),
                        before
                    );
                    let collected = owner.storage().collect(mizar_ir::storage::CollectInput {
                        snapshot: old.snapshot,
                        protected_outputs: vec![],
                    });
                    assert_eq!(collected.outputs_dropped, 10);
                    assert_eq!(collected.abandoned_slots_dropped, 0);
                },
            );
            if positions.len() == 2 {
                break;
            }
        }
        assert_eq!(positions, [0, 1].into_iter().collect());
    }
}

#[test]
fn native_theorem_kernel_handoff_core_vc_caller_preserves_refusals() {
    use mizar_core::control_flow::{build_control_flow_ir, build_obligation_seed_handoff};
    use mizar_driver::{
        native_theorem_core_from_storage_bytes as decode_core,
        native_theorem_core_storage_bytes as encode_core,
        native_theorem_preparation_from_storage_bytes as decode_pair,
        native_theorem_preparation_storage_bytes as encode_pair,
        native_theorem_vc_from_storage_bytes as decode, native_theorem_vc_storage_bytes as encode,
    };
    use mizar_ir::{
        publisher::{OutputOrigin, PublicationTarget, PublishOutputInput},
        storage::{BlobDecodeError, BlobDecoder, CollectInput, IrSideTables, SchemaVersion},
    };
    use mizar_vc::{
        generator::generate_source_theorem_vcs as generate,
        kernel_evidence_handoff::build_source_theorem_kernel_handoff as handoff,
        vc_ir::{GenerationSchemaVersion, VcId, VcSchemaVersion},
    };
    let generation = GenerationSchemaVersion::new(" \n\t");
    let schema = VcSchemaVersion::new("");
    let text = "theorem T: for X being set holds X = X proof let X be set; thus X = X; end;";
    for case in [
        "missing-core",
        "missing-vc",
        "wrong-core",
        "wrong-vc",
        "hash",
        "obsolete",
        "collected-core",
        "collected-vc",
        "dependencies",
        "absent",
        "snapshot",
        "source-none",
        "frontend-vc",
        "ancestors-collected",
    ] {
        with_native_summary_parent(
            text,
            false,
            0,
            |_, inputs, old, owner, outputs, (_, ids, snapshots)| {
                for (phase, kind) in [
                    ("SignatureCollection", "SourceTheoremModuleSummary"),
                    ("TypeChecking", "SourceTheoremTypedAst"),
                    ("RegistrationResolution", "SourceTheoremRegistrationClosure"),
                    ("OverloadResolution", "SourceTheoremMetadata"),
                    ("Elaboration", "SourceTheoremCore"),
                    ("AlgorithmPreparation", "SourceTheoremPreparation"),
                    ("VcGenerate", "SourceTheoremVC"),
                ] {
                    owner.allow_work_unit(AllowedWorkUnit::new(
                        IrPipelinePhase::new(phase),
                        OutputKind::new(kind),
                        IrWorkUnit::new("\"alpha\":\"main\""),
                    ));
                }
                let child = |parents: Vec<AnyPhaseOutputRef>| {
                    PhaseInput::new(
                        old.work_unit.clone(),
                        PhaseDispatchInputBundle::new(
                            old.snapshot,
                            old.identities().input_hash(),
                            vec![],
                            parents
                                .into_iter()
                                .map(|parent| {
                                    SealedParentOutputHandle::from_published_output(
                                        owner,
                                        old.snapshot,
                                        parent,
                                    )
                                    .unwrap()
                                })
                                .collect(),
                        )
                        .unwrap(),
                    )
                };
                let signature = inputs
                    .publish_native_theorem_module_summary(old, owner)
                    .unwrap()
                    .unwrap();
                let type_input = child(vec![signature.erase()]);
                let typed = inputs
                    .publish_native_theorem_typed_ast(&type_input, owner)
                    .unwrap()
                    .unwrap();
                let reg_input = child(vec![typed.erase()]);
                let registration = inputs
                    .publish_native_theorem_registration_closure(&reg_input, owner)
                    .unwrap()
                    .unwrap();
                let meta_input = child(vec![typed.erase(), registration.erase()]);
                let parent = inputs
                    .publish_native_theorem_metadata(&meta_input, owner)
                    .unwrap()
                    .unwrap();
                let frontend = outputs
                    .iter()
                    .find(|output| output.phase() == &IrPipelinePhase::new("Frontend"))
                    .unwrap();
                let metadata_input = child(vec![parent.erase()]);
                let mut other_snapshot = None;
                let (core_parent, preparation, foreign_ast) = if case == "source-none" {
                    let actual_core = inputs
                        .native_theorem_core_with_metadata_parent(&metadata_input, owner)
                        .unwrap()
                        .unwrap();
                    let changed = Fixture::new(b"theorem Different: for Y being set holds Y = Y proof let Y be set; thus Y = Y; end;");
                    let (_, submission, adapter, frontend, _) =
                        native_disk_frontend_record_submission(&changed, ids, snapshots, 0);
                    let (_, other_old) = native_disk_lookup_context(&submission, ids);
                    let other_owner = adapter.publisher();
                    let input = PhaseInput::new(
                        other_old.work_unit.clone(),
                        PhaseDispatchInputBundle::new(
                            other_old.snapshot,
                            other_old.identities().input_hash(),
                            vec![],
                            vec![
                                SealedParentOutputHandle::from_published_output(
                                    other_owner,
                                    other_old.snapshot,
                                    frontend.erase(),
                                )
                                .unwrap(),
                            ],
                        )
                        .unwrap(),
                    );
                    let core =
                        genuine_theorem_core_from_frontend_parent(&input, other_owner).unwrap();
                    let output = other_owner.storage().get(&frontend).unwrap();
                    let ast = output.ast.as_ref().unwrap().clone();
                    assert_ne!(core.source_id(), actual_core.source_id());
                    assert_ne!(core, actual_core);
                    let source_id = ast.source_id;
                    let module = core.module_id().clone();
                    let flow = build_control_flow_ir(&core);
                    let pair = (flow.clone(), build_obligation_seed_handoff(&core, &flow));
                    let core_bytes = encode_core(&ast, &module, &core).unwrap().unwrap();
                    let bytes = encode_pair(&ast, &module, &core, &pair).unwrap().unwrap();
                    let phase = IrPipelinePhase::new("Elaboration");
                    let kind = OutputKind::new("SourceTheoremCore");
                    let core_module = module.clone();
                    let core = owner
                        .publish(PublishOutputInput {
                            slot: owner.allocate(
                                old.snapshot,
                                phase.clone(),
                                parent.work_unit().clone(),
                                kind.clone(),
                                SchemaVersion::new(1),
                            ),
                            snapshot: old.snapshot,
                            phase,
                            work_unit: parent.work_unit().clone(),
                            output_kind: kind,
                            schema_version: SchemaVersion::new(1),
                            payload: core,
                            canonical_payload: Some(core_bytes.clone()),
                            storage_payload: Some(core_bytes),
                            decode: BlobDecoder::new(move |bytes| {
                                decode_core(bytes, source_id, &core_module)
                                    .map_err(BlobDecodeError::new)?
                                    .ok_or_else(|| BlobDecodeError::new("negative Core replay"))
                            }),
                            parents: vec![parent.erase()],
                            named_input_hashes: parent.lineage().named_input_hashes.clone(),
                            side_tables: IrSideTables::default(),
                            origin: OutputOrigin::PackageSource,
                            target: PublicationTarget::CurrentPackage,
                        })
                        .unwrap();
                    let phase = IrPipelinePhase::new("AlgorithmPreparation");
                    let kind = OutputKind::new("SourceTheoremPreparation");
                    let preparation = owner
                        .publish(PublishOutputInput {
                            slot: owner.allocate(
                                old.snapshot,
                                phase.clone(),
                                parent.work_unit().clone(),
                                kind.clone(),
                                SchemaVersion::new(1),
                            ),
                            snapshot: old.snapshot,
                            phase,
                            work_unit: parent.work_unit().clone(),
                            output_kind: kind,
                            schema_version: SchemaVersion::new(1),
                            payload: pair.clone(),
                            canonical_payload: Some(bytes.clone()),
                            storage_payload: Some(bytes),
                            decode: BlobDecoder::new(move |bytes| {
                                decode_pair(bytes, source_id, &module)
                                    .map_err(BlobDecodeError::new)?
                                    .ok_or_else(|| {
                                        BlobDecodeError::new("negative Preparation replay")
                                    })
                            }),
                            parents: vec![core.erase()],
                            named_input_hashes: parent.lineage().named_input_hashes.clone(),
                            side_tables: IrSideTables::default(),
                            origin: OutputOrigin::PackageSource,
                            target: PublicationTarget::CurrentPackage,
                        })
                        .unwrap();
                    assert_eq!(*owner.storage().get(&preparation).unwrap(), pair);
                    (core, preparation, Some(ast))
                } else {
                    let core = inputs
                        .publish_native_theorem_core_with_metadata_parent(&metadata_input, owner)
                        .unwrap()
                        .unwrap();
                    let sole = child(vec![core.erase()]);
                    let preparation = inputs
                        .publish_native_theorem_preparation(&sole, owner)
                        .unwrap()
                        .unwrap();
                    (core, preparation, None)
                };
                let core_input = child(vec![core_parent.erase()]);
                let core = owner.storage().get(&core_parent).unwrap();
                let vc_parent = if matches!(case, "snapshot" | "source-none" | "frontend-vc") {
                    let ast = if case == "snapshot" {
                        let frontend_handle = owner
                            .storage()
                            .typed_handle::<FrontendOutput<<MizarParserSeam as ParserSeam>::Ast>>(
                                frontend,
                                frontend.output_kind(),
                            )
                            .unwrap();
                        let output = owner.storage().get(&frontend_handle).unwrap();
                        output.ast.as_ref().unwrap().clone()
                    } else if case == "source-none" {
                        foreign_ast.unwrap()
                    } else {
                        let frontend_handle = owner
                            .storage()
                            .typed_handle::<FrontendOutput<<MizarParserSeam as ParserSeam>::Ast>>(
                                frontend,
                                &OutputKind::new("FrontendOutput"),
                            )
                            .unwrap();
                        owner
                            .storage()
                            .get(&frontend_handle)
                            .unwrap()
                            .ast
                            .as_ref()
                            .unwrap()
                            .clone()
                    };
                    let snapshot = if case == "snapshot" {
                        let other = Fixture::new(text.as_bytes());
                        let (_, submission, _, _, _) =
                            native_disk_frontend_record_submission(&other, ids, snapshots, 0);
                        let snapshot = submission.session.captured().snapshot().id;
                        assert_ne!(snapshot, old.snapshot);
                        other_snapshot = Some(snapshot);
                        snapshot
                    } else {
                        old.snapshot
                    };
                    let graph = generate(&core, snapshot, &generation, &schema).unwrap();
                    assert!(handoff(&core, &graph, graph.vcs()[0].id).is_ok());
                    let raw = inputs
                        .native_theorem_preparation_with_core_parent(&core_input, owner)
                        .unwrap();
                    assert_eq!(raw.is_some(), matches!(case, "snapshot" | "frontend-vc"));
                    let bytes =
                        encode(&ast, core.module_id(), &core, &graph, snapshot, &generation)
                            .unwrap()
                            .unwrap();
                    let source_id = core.source_id();
                    let module = core.module_id().clone();
                    let phase = IrPipelinePhase::new("VcGenerate");
                    let kind = OutputKind::new("SourceTheoremVC");
                    let parents = if case == "frontend-vc" {
                        vec![frontend.clone()]
                    } else {
                        vec![core_parent.erase(), preparation.erase()]
                    };
                    let negative = owner
                        .publish(PublishOutputInput {
                            slot: owner.allocate(
                                old.snapshot,
                                phase.clone(),
                                parent.work_unit().clone(),
                                kind.clone(),
                                SchemaVersion::new(1),
                            ),
                            snapshot: old.snapshot,
                            phase,
                            work_unit: parent.work_unit().clone(),
                            output_kind: kind,
                            schema_version: SchemaVersion::new(1),
                            payload: graph.clone(),
                            canonical_payload: Some(bytes.clone()),
                            storage_payload: Some(bytes),
                            decode: BlobDecoder::new(move |bytes| {
                                decode(bytes, source_id, &module, snapshot)
                                    .map_err(BlobDecodeError::new)?
                                    .ok_or_else(|| BlobDecodeError::new("negative VC replay"))
                            }),
                            parents,
                            named_input_hashes: parent.lineage().named_input_hashes.clone(),
                            side_tables: IrSideTables::default(),
                            origin: OutputOrigin::PackageSource,
                            target: PublicationTarget::CurrentPackage,
                        })
                        .unwrap();
                    assert_eq!(*owner.storage().get(&negative).unwrap(), graph);
                    negative
                } else {
                    let input = child(vec![core_parent.erase(), preparation.erase()]);
                    inputs
                        .publish_native_theorem_vcs_with_core_preparation_parents(
                            &input,
                            owner,
                            &generation,
                            &schema,
                        )
                        .unwrap()
                        .unwrap()
                };
                let vcs = owner.storage().get(&vc_parent).unwrap();
                let valid = vcs.vcs()[0].id;
                let selected = if matches!(case, "absent" | "source-none" | "dependencies") {
                    VcId::new(vcs.vcs().len())
                } else {
                    valid
                };
                let baseline = handoff(&core, &vcs, selected);
                if matches!(case, "snapshot" | "frontend-vc") {
                    assert!(baseline.is_ok());
                    assert!(
                        inputs
                            .native_theorem_preparation_with_core_parent(&core_input, owner)
                            .unwrap()
                            .is_some()
                    );
                }
                if case == "snapshot" {
                    assert_eq!(vcs.snapshot(), other_snapshot.unwrap());
                }
                let mut input = child(match case {
                    "missing-core" => vec![vc_parent.erase()],
                    "missing-vc" => vec![core_parent.erase()],
                    "wrong-core" => vec![parent.erase(), vc_parent.erase()],
                    "wrong-vc" => vec![core_parent.erase(), parent.erase()],
                    _ => vec![core_parent.erase(), vc_parent.erase()],
                });
                if case == "hash" {
                    input = PhaseInput::new(
                        old.work_unit.clone(),
                        PhaseDispatchInputBundle::new(
                            old.snapshot,
                            hash(0xd0),
                            vec![],
                            input.parent_outputs().to_vec(),
                        )
                        .unwrap(),
                    );
                }
                let upstream = if case == "dependencies" {
                    let [dual, sole] = [input.clone(), core_input].map(|prior| {
                        PhaseInput::new(
                            prior.work_unit.clone(),
                            PhaseDispatchInputBundle::new(
                                prior.snapshot,
                                prior.identities().input_hash(),
                                vec![hash(0xd1)],
                                prior.parent_outputs().to_vec(),
                            )
                            .unwrap(),
                        )
                    });
                    input = dual;
                    Some(
                        inputs
                            .native_theorem_preparation_with_core_parent(&sole, owner)
                            .unwrap_err(),
                    )
                } else {
                    None
                };
                if case == "obsolete" {
                    owner.mark_obsolete(old.snapshot).unwrap();
                } else if matches!(
                    case,
                    "collected-core" | "collected-vc" | "ancestors-collected"
                ) {
                    let protected = match case {
                        "collected-core" => vec![vc_parent.output()],
                        "collected-vc" => vec![core_parent.output()],
                        _ => vec![core_parent.output(), vc_parent.output()],
                    };
                    assert_eq!(
                        owner
                            .storage()
                            .collect(CollectInput {
                                snapshot: old.snapshot,
                                protected_outputs: protected
                            })
                            .outputs_dropped,
                        if case == "ancestors-collected" { 8 } else { 9 }
                    );
                }
                let result = inputs
                    .native_theorem_kernel_handoff_with_core_vc_parents(&input, owner, selected);
                match case {
                    "source-none" => assert_eq!(result.unwrap(), None),
                    "snapshot" => assert_eq!(
                        result.unwrap_err(),
                        "native theorem kernel handoff VC snapshot mismatch"
                    ),
                    "frontend-vc" => assert_eq!(
                        result.unwrap_err(),
                        "native theorem kernel handoff requires Core and Preparation ancestry"
                    ),
                    "dependencies" => assert_eq!(result.unwrap_err(), upstream.unwrap()),
                    "absent" => assert_eq!(result.unwrap_err(), baseline.unwrap_err()),
                    "ancestors-collected" => assert_eq!(result.unwrap(), Some(baseline.unwrap())),
                    _ => assert!(result.is_err(), "{case}"),
                }
                let collected = owner.storage().collect(CollectInput {
                    snapshot: old.snapshot,
                    protected_outputs: vec![],
                });
                assert_eq!(
                    collected.outputs_dropped,
                    if case == "ancestors-collected" {
                        2
                    } else if case.starts_with("collected-") {
                        1
                    } else {
                        10
                    }
                );
                assert_eq!(collected.abandoned_slots_dropped, 0);
            },
        );
    }
}
