use super::*;

#[test]
fn scheduled_theorem_candidate_check_matches_genuine_publication() {
    use mizar_atp::{backend::build_source_theorem_candidate_inputs as build, problem::*};
    use mizar_build::task_graph::{
        EvidenceCandidateId, VcOrderKey, VcTaskDescriptor, VcTaskDescriptorId,
    };
    use mizar_checker::type_checker::{
        SourceVariableSemanticsChecker, check_source_theorem_metadata,
    };
    use mizar_core::core_ir::{CoreIr, CoreProofStatus, ObligationSeedStatus};
    use mizar_ir::storage::{CollectInput, IrSideTables, SchemaVersion, SideTableRecord};
    use mizar_kernel::checker::{KernelCheckResult, KernelCheckStatus, KernelEvidenceCheckKind};
    use mizar_proof::{
        policy::{CandidatePolicyClass, PolicyDecision, VerifierPolicy},
        status::check_source_theorem_candidate as fresh,
    };
    use mizar_resolve::{
        names::{SourceVariableScopeInput, SourceVariableScopeResolver},
        symbols::{
            source_proof_inputs, source_theorem_module_candidates,
            source_theorem_module_original_inputs,
        },
    };
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
                let mut representations = Vec::new();
                for variant in 0..3 {
                    let policy = DischargePolicy {
                        computation_limit: ComputationLimit {
                            policy: PolicyKey::new(if variant == 1 {
                                "changed計算\0"
                            } else {
                                "計算\0"
                            }),
                            max_steps: u32::MAX,
                        },
                    };
                    let verifier = if variant == 2 {
                        VerifierPolicy::release().with_profile_id("current核\0")
                    } else {
                        VerifierPolicy::release()
                    };
                    let baseline = publisher(old.snapshot, threshold, &old.work_unit);
                    let owner = publisher(old.snapshot, threshold, &old.work_unit);
                    let mut submissions = Vec::new();
                    for current in [&baseline, &owner] {
                        allow_frontend(current, &old.work_unit);
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
                            ("KernelCheck", "SourceTheoremCandidateCheck"),
                        ] {
                            current.allow_work_unit(AllowedWorkUnit::new(
                                IrPipelinePhase::new(phase),
                                OutputKind::new(kind),
                                IrWorkUnit::new("\"alpha\":\"main\""),
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
                            .register_source_theorem_vc_generation(
                                generation.clone(),
                                schema.clone(),
                            )
                            .register_source_theorem_vc_discharge(policy.clone());
                        let (_, mut submit) = fixture.submit_request(vec![], vec![]);
                        submit.vc_descriptors = vec![VcTaskDescriptor::new(
                            VcTaskDescriptorId::new("opaque-prerequisite:090"),
                            module.clone(),
                            VcOrderKey::new("order"),
                            vec![],
                            vec![],
                        )];
                        submissions.push(
                            CompilerDriver::new(builder.build().unwrap())
                                .with_output_publisher(current.clone())
                                .submit_captured(session.clone(), ids, snapshots, submit)
                                .unwrap(),
                        );
                    }
                    let baseline_submission = submissions.remove(0);
                    let actual_submission = submissions.remove(0);
                    let (inputs, original) = native_disk_lookup_context(&baseline_submission, ids);
                    let (actual_inputs, actual_original) =
                        native_disk_lookup_context(&actual_submission, ids);
                    let baseline_refs = baseline_submission
                        .scheduler_run
                        .as_ref()
                        .unwrap()
                        .phase_results
                        .values()
                        .flatten()
                        .flat_map(|row| row.output_refs.iter().cloned())
                        .collect::<Vec<_>>();
                    let actual_refs = actual_submission
                        .scheduler_run
                        .as_ref()
                        .unwrap()
                        .phase_results
                        .values()
                        .flatten()
                        .flat_map(|row| row.output_refs.iter().cloned())
                        .collect::<Vec<_>>();
                    assert_eq!((baseline_refs.len(), actual_refs.len()), (11, 11));
                    let parent = |kind: &str| {
                        baseline_refs
                            .iter()
                            .find(|parent| parent.output_kind() == &OutputKind::new(kind))
                            .unwrap()
                    };
                    let core_ref = parent("SourceTheoremCore");
                    let vc_ref = parent("SourceTheoremVC");
                    let discharge_ref = parent("SourceTheoremDischarge");
                    let frontend_ref = parent("FrontendOutput");
                    let core_handle = baseline
                        .storage()
                        .typed_handle::<CoreIr>(core_ref, core_ref.output_kind())
                        .unwrap();
                    let vc_handle = baseline
                        .storage()
                        .typed_handle::<VcSet>(vc_ref, vc_ref.output_kind())
                        .unwrap();
                    let discharge_handle = baseline
                        .storage()
                        .typed_handle::<DischargeOutput>(discharge_ref, discharge_ref.output_kind())
                        .unwrap();
                    let core = baseline.storage().get(&core_handle).unwrap();
                    let vcs = baseline.storage().get(&vc_handle).unwrap();
                    let discharge = baseline.storage().get(&discharge_handle).unwrap();
                    let frontend = baseline.storage().get(&baseline.storage()
                    .typed_handle::<FrontendOutput<<MizarParserSeam as ParserSeam>::Ast>>(frontend_ref, frontend_ref.output_kind()).unwrap()).unwrap();
                    let ast = frontend.ast.as_ref().unwrap();
                    let [vc] = vcs.vcs() else { panic!() };
                    let selected = vc.id;
                    let expected_pair = build(
                        &core,
                        &vcs,
                        selected,
                        &discharge,
                        &policy,
                        profile.clone(),
                        format,
                    )
                    .unwrap();
                    let (arena, env) = source_theorem_module_original_inputs(ast, core.module_id())
                        .unwrap()
                        .unwrap();
                    assert!(
                        source_theorem_module_candidates(ast, &arena, &env)
                            .unwrap()
                            .is_some()
                    );
                    let scope = SourceVariableScopeResolver::resolve_proof_occurrences(
                        SourceVariableScopeInput::new(ast, core.module_id(), &env),
                    )
                    .unwrap();
                    let metadata = check_source_theorem_metadata(&arena, &env, &scope).unwrap();
                    let (scope, labels, resolved) = source_proof_inputs(&arena, &env).unwrap();
                    let check = SourceVariableSemanticsChecker::check_theorem_skeletons(
                        &arena,
                        &metadata.2,
                        &scope,
                        &env,
                        &labels,
                        &resolved,
                        None,
                    )
                    .unwrap();
                    let expected = fresh(
                        &arena,
                        &check,
                        &env,
                        old.snapshot,
                        selected,
                        (&generation, &schema, profile.clone(), format),
                        (&policy, &verifier),
                    )
                    .unwrap();
                    assert_eq!(
                        (
                            &expected.0,
                            &expected.1,
                            &expected.2,
                            &expected.3,
                            &expected.4
                        ),
                        (
                            &*core,
                            &*vcs,
                            &*discharge,
                            &expected_pair.0,
                            &expected_pair.1
                        )
                    );
                    assert_eq!(expected.5.status(), KernelCheckStatus::Accepted);
                    assert_eq!(
                        expected.5.evidence_check_kind(),
                        Some(KernelEvidenceCheckKind::ProofObligation)
                    );
                    assert!(!expected.5.policy_taint());
                    assert_eq!(expected.6.class, CandidatePolicyClass::KernelVerified);
                    assert_eq!(ast.source_id, core.source_id());
                    assert_eq!(vcs.source(), ast.source_id);
                    assert_eq!(vcs.snapshot(), old.snapshot);
                    assert_eq!(vcs.schema_version(), &schema);
                    assert_eq!(vc.anchor.generation_schema_version, generation);
                    assert!(!core.proofs().is_empty() && !vcs.seed_accounting().is_empty());
                    assert!(
                        core.proofs().iter().all(
                            |(_, proof)| proof.status == CoreProofStatus::PendingAutomaticProof
                        )
                    );
                    for seed in vcs.seed_accounting() {
                        let SeedOriginRef::ExistingCore { seed: id } = seed.origin else {
                            panic!()
                        };
                        assert_eq!(seed.seed_status, ObligationSeedStatus::Active);
                        assert_eq!(
                            core.obligation_seeds().get(id).unwrap().status,
                            ObligationSeedStatus::Active
                        );
                    }
                    assert!(vcs.vcs().iter().all(|vc| vc.status == VcStatus::Open));
                    assert!(
                        discharge
                            .vc_set()
                            .vcs()
                            .iter()
                            .all(|vc| vc.status == VcStatus::NeedsAtp)
                    );
                    assert_eq!(
                        expected_pair.1.target_binding(),
                        expected_pair.0.target_binding()
                    );
                    assert_eq!(expected_pair.0.concrete_format(), format);
                    assert_eq!(
                        expected_pair.0.logic_profile_name(),
                        profile.name().as_str()
                    );
                    let baseline_input = PhaseInput::new(
                        original.work_unit.clone(),
                        PhaseDispatchInputBundle::new(
                            old.snapshot,
                            original.identities().input_hash(),
                            original.identities().dependency_hashes().to_vec(),
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
                    let atp = inputs
                        .publish_native_theorem_candidate(
                            &baseline_input,
                            &baseline,
                            selected,
                            &policy,
                            profile.clone(),
                            format,
                        )
                        .unwrap()
                        .unwrap();
                    assert_eq!(*baseline.storage().get(&atp).unwrap(), expected_pair);
                    let mut four_parents = baseline_input.parent_outputs().to_vec();
                    four_parents.push(
                        SealedParentOutputHandle::from_published_output(
                            &baseline,
                            old.snapshot,
                            atp.erase(),
                        )
                        .unwrap(),
                    );
                    let four_input = PhaseInput::new(
                        original.work_unit.clone(),
                        PhaseDispatchInputBundle::new(
                            old.snapshot,
                            original.identities().input_hash(),
                            original.identities().dependency_hashes().to_vec(),
                            four_parents,
                        )
                        .unwrap(),
                    );
                    let published = inputs
                        .publish_native_theorem_candidate_check(
                            &four_input,
                            &baseline,
                            selected,
                            (profile.clone(), format),
                            (&policy, &verifier),
                        )
                        .unwrap()
                        .unwrap();
                    let expected_checked = (expected.5, expected.6);
                    assert_eq!(
                        *baseline.storage().get(&published).unwrap(),
                        expected_checked
                    );
                    let actual_parents = [
                        "SourceTheoremDischarge",
                        "SourceTheoremCore",
                        "SourceTheoremVC",
                    ]
                    .map(|kind| {
                        actual_refs
                            .iter()
                            .find(|parent| parent.output_kind() == &OutputKind::new(kind))
                            .unwrap()
                    });
                    let query = PhaseInput::new(
                        WorkUnit::EvidenceCandidate {
                            module: module.clone(),
                            descriptor: VcTaskDescriptorId::new("opaque-check:090"),
                            evidence_candidate: EvidenceCandidateId::new("no numeric VC id here"),
                        },
                        PhaseDispatchInputBundle::new(
                            old.snapshot,
                            actual_original.identities().input_hash(),
                            actual_original.identities().dependency_hashes().to_vec(),
                            actual_parents
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
                    let mut builder = PhaseRegistryBuilder::new();
                    builder
                        .register_source_theorem_candidate_check(
                            policy.clone(),
                            profile.clone(),
                            format,
                            verifier.clone(),
                        )
                        .unwrap();
                    let registry = builder.build().unwrap();
                    let descriptor = registry
                        .descriptor_for_phase(PipelinePhase::KernelCheck)
                        .unwrap();
                    let key = policy.computation_limit.policy.as_str();
                    let candidate_schema = format!(
                        "mizar-atp/source-theorem-candidate-publication/v1;computation_policy={}:{key};max_steps={};profile={}:caller λ\n\0;fragment={};equality=1;quantifiers=1;soft_types=3;native_properties=1;formats=3;format={}",
                        key.len(),
                        u32::MAX,
                        "caller λ\n\0".len(),
                        if open { 2 } else { 0 },
                        u8::from(open)
                    );
                    let fingerprint = verifier.policy_fingerprint();
                    let hex = fingerprint
                        .as_bytes()
                        .iter()
                        .map(|byte| format!("{byte:02x}"))
                        .collect::<String>();
                    assert_eq!(descriptor.service_name, "SourceTheoremCandidateCheck");
                    assert_eq!(
                        descriptor.owner,
                        mizar_driver::registry::PhaseOwner::MizarKernelProof
                    );
                    assert_eq!(descriptor.phases, [PipelinePhase::KernelCheck]);
                    assert_eq!(descriptor.output_kind, "SourceTheoremCandidateCheck");
                    assert_eq!(
                        descriptor.schema_version,
                        format!(
                            "mizar-proof/source-theorem-candidate-check-publication/v1;candidate={}:{};verifier_policy={hex}",
                            candidate_schema.len(),
                            candidate_schema
                        )
                    );
                    let cache = registry
                        .cache_key_for_phase(PipelinePhase::KernelCheck, &query)
                        .unwrap();
                    assert_eq!(cache.intent, PhaseCacheIntent::NoKey { reason: "source theorem candidate check has no complete dependency footprint".to_owned() });
                    let mut hashes = actual_parents.map(|parent| parent.output().hash());
                    hashes.sort_by(|a, b| a.as_bytes().cmp(b.as_bytes()));
                    assert_eq!(query.identities().parent_output_hashes(), hashes);
                    assert_eq!(
                        query.identities().input_hash(),
                        source_key(&actual_inputs.snapshot.source_versions[0])
                    );
                    assert_eq!(
                        query.identities().dependency_hashes(),
                        actual_original.identities().dependency_hashes()
                    );
                    let probe = registry
                        .execute_phase_with_resources(
                            PipelinePhase::KernelCheck,
                            query.clone(),
                            PhaseExecutionResources::default(),
                        )
                        .unwrap();
                    assert_eq!(probe.result.status, PhaseStatus::Blocking);
                    assert!(
                        probe.result.output_refs.is_empty() && probe.result.diagnostics.is_empty()
                    );
                    let before = (
                        query.clone(),
                        (*core).clone(),
                        (*vcs).clone(),
                        (*discharge).clone(),
                        expected_pair.clone(),
                        policy.clone(),
                        profile.clone(),
                        verifier.clone(),
                        session.captured().clone(),
                    );
                    fixture.write(&[0xff]);
                    if fixture.root.join("alpha/src/main.miz").exists() {
                        fs::remove_file(fixture.root.join("alpha/src/main.miz")).unwrap();
                    }
                    let result = registry
                        .execute_phase_with_resources(
                            PipelinePhase::KernelCheck,
                            query.clone(),
                            PhaseExecutionResources {
                                output_publisher: Some(owner.clone()),
                                source_load: Some(actual_inputs),
                                ..PhaseExecutionResources::default()
                            },
                        )
                        .unwrap();
                    assert_eq!(result.result.status, PhaseStatus::Complete);
                    assert!(result.result.diagnostics.is_empty());
                    assert_eq!(result.observation, probe.observation);
                    assert_eq!(result.result.cache_observation, Some(result.observation));
                    let [actual] = result.result.output_refs.as_slice() else {
                        panic!()
                    };
                    let handle = owner
                        .storage()
                        .typed_handle::<(KernelCheckResult, PolicyDecision)>(
                            actual,
                            actual.output_kind(),
                        )
                        .unwrap();
                    assert_eq!(*owner.storage().get(&handle).unwrap(), expected_checked);
                    assert_eq!(actual.phase(), &IrPipelinePhase::new("KernelCheck"));
                    assert_eq!(
                        actual.output_kind(),
                        &OutputKind::new("SourceTheoremCandidateCheck")
                    );
                    assert_eq!(actual.schema_version(), SchemaVersion::new(1));
                    assert_eq!(actual.snapshot(), old.snapshot);
                    assert_eq!(actual.work_unit(), &IrWorkUnit::new("\"alpha\":\"main\""));
                    assert_eq!(actual.lineage(), published.lineage());
                    assert_eq!(actual.lineage().parents.len(), 4);
                    assert_eq!(query.parent_outputs().len(), 3);
                    for parent in actual_parents {
                        assert!(actual.lineage().parents.contains(&parent.output()));
                    }
                    let hidden = actual
                        .lineage()
                        .parents
                        .iter()
                        .find(|id| !actual_parents.iter().any(|parent| parent.output() == **id))
                        .unwrap();
                    assert_eq!(
                        owner.registry().output_lineage(*hidden).unwrap(),
                        atp.lineage().clone()
                    );
                    assert_eq!(
                        actual.lineage().named_input_hashes,
                        core_ref.lineage().named_input_hashes
                    );
                    assert_eq!(actual.content_hash(), published.content_hash());
                    assert_eq!(actual.side_table_hash(), published.side_table_hash());
                    let tables = owner.storage().side_tables(&handle).unwrap();
                    assert_eq!(
                        *tables,
                        *baseline.storage().side_tables(&published).unwrap()
                    );
                    let [record] = tables.source_maps.as_slice() else {
                        panic!()
                    };
                    assert_eq!(
                        *tables,
                        IrSideTables {
                            source_maps: vec![SideTableRecord::new(
                                "candidate-check-theorem-storage",
                                actual_inputs.snapshot.source_versions[0]
                                    .normalized_path
                                    .as_str(),
                                record.digest,
                            )],
                            ..IrSideTables::default()
                        }
                    );
                    assert_eq!(actual.placement(), published.placement());
                    assert_eq!(
                        matches!(actual.placement(), StoragePlacement::Blob { .. }),
                        open
                    );
                    owner
                        .validate_published_output(old.snapshot, actual)
                        .unwrap();
                    assert_eq!(
                        owner.validate_current_output(old.snapshot, actual).is_ok(),
                        !open
                    );
                    assert_eq!(
                        (
                            query.clone(),
                            (*baseline.storage().get(&core_handle).unwrap()).clone(),
                            (*baseline.storage().get(&vc_handle).unwrap()).clone(),
                            (*baseline.storage().get(&discharge_handle).unwrap()).clone(),
                            (*baseline.storage().get(&atp).unwrap()).clone(),
                            policy.clone(),
                            profile.clone(),
                            verifier.clone(),
                            session.captured().clone()
                        ),
                        before
                    );
                    let protected = owner.storage().collect(CollectInput {
                        snapshot: old.snapshot,
                        protected_outputs: actual_refs
                            .iter()
                            .map(|parent| parent.output())
                            .chain([*hidden, actual.output()])
                            .collect(),
                    });
                    assert_eq!(
                        (
                            protected.outputs_dropped,
                            protected.protected_outputs,
                            protected.abandoned_slots_dropped
                        ),
                        (0, 13, 0)
                    );
                    let repeated = registry
                        .execute_phase_with_resources(
                            PipelinePhase::KernelCheck,
                            query,
                            PhaseExecutionResources {
                                output_publisher: Some(owner.clone()),
                                source_load: Some(actual_inputs),
                                ..PhaseExecutionResources::default()
                            },
                        )
                        .unwrap();
                    assert_eq!(repeated.result.status, PhaseStatus::Blocking);
                    assert!(
                        repeated.result.output_refs.is_empty()
                            && repeated.result.diagnostics.is_empty()
                    );
                    assert_eq!(*owner.storage().get(&handle).unwrap(), expected_checked);
                    representations.push((
                        expected_pair,
                        expected_checked,
                        actual.content_hash(),
                        descriptor.schema_version.clone(),
                        cache.observation,
                    ));
                    for (current, abandoned) in [(&baseline, 0), (&owner, 1)] {
                        let collected = current.storage().collect(CollectInput {
                            snapshot: old.snapshot,
                            protected_outputs: vec![],
                        });
                        assert_eq!(
                            (collected.outputs_dropped, collected.abandoned_slots_dropped),
                            (13, abandoned)
                        );
                    }
                }
                for changed in &representations[1..] {
                    assert_eq!(changed.0, representations[0].0);
                    assert_eq!(changed.1, representations[0].1);
                    assert_ne!(changed.2, representations[0].2);
                    assert_ne!(changed.3, representations[0].3);
                    assert_ne!(changed.4, representations[0].4);
                }
            },
        );
    }
}

#[test]
fn scheduled_theorem_candidate_check_preserves_resource_refusals() {
    use mizar_atp::{backend::build_source_theorem_candidate_inputs as build, problem::*};
    use mizar_build::task_graph::{
        EvidenceCandidateId, VcOrderKey, VcTaskDescriptor, VcTaskDescriptorId,
    };
    use mizar_checker::type_checker::{
        SourceVariableSemanticsChecker, check_source_theorem_metadata,
    };
    use mizar_core::core_ir::CoreIr;
    use mizar_ir::{publisher::PublishError, storage::CollectInput};
    use mizar_proof::{policy::VerifierPolicy, status::check_source_theorem_candidate as fresh};
    use mizar_resolve::{
        names::{SourceVariableScopeInput, SourceVariableScopeResolver},
        symbols::{
            source_proof_inputs, source_theorem_module_candidates,
            source_theorem_module_original_inputs,
        },
    };
    use mizar_vc::{
        discharge::{ComputationLimit, DischargeOutput, DischargePolicy},
        vc_ir::{GenerationSchemaVersion, PolicyKey, VcSchemaVersion, VcSet},
    };
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
                    policy: PolicyKey::new("計算\0"),
                    max_steps: 0,
                },
            };
            let profile = LogicProfile::try_new(
                "caller λ\n\0",
                LogicFragment::Fof,
                EqualitySupport::Supported,
                QuantifierPolicy::FirstOrder,
                SoftTypeStrategy::ScopedGuardPredicates,
                NativePropertySupport::Supported,
                [ConcreteFormat::Tptp, ConcreteFormat::SmtLib]
                    .into_iter()
                    .collect(),
            )
            .unwrap();
            let verifier = VerifierPolicy::release();
            let disabled = verifier.clone().with_kernel_evidence_formats([]);
            let WorkUnit::Module { module } = &old.work_unit else {
                panic!()
            };
            let unit = IrWorkUnit::new("\"alpha\":\"main\"");
            let mut baseline_pair = None;
            for case in [
                "baseline",
                "calibrate-atp-denied",
                "cancel",
                "foreign-token",
                "no-publisher",
                "no-inputs",
                "source-snapshot",
                "foreign-publisher",
                "obsolete",
                "collected",
                "module",
                "vc-unit",
                "wrong-module",
                "missing",
                "surplus",
                "atp-denied",
                "kernel-denied",
                "disabled-policy",
                "unsupported-profile",
            ] {
                let owner = publisher(old.snapshot, 0, &old.work_unit);
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
                if !matches!(case, "atp-denied" | "calibrate-atp-denied") {
                    owner.allow_work_unit(AllowedWorkUnit::new(
                        IrPipelinePhase::new("AtpSolve"),
                        OutputKind::new("SourceTheoremCandidateInputs"),
                        unit.clone(),
                    ));
                }
                if !matches!(case, "baseline" | "kernel-denied") {
                    owner.allow_work_unit(AllowedWorkUnit::new(
                        IrPipelinePhase::new("KernelCheck"),
                        OutputKind::new("SourceTheoremCandidateCheck"),
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
                    VcTaskDescriptorId::new("opaque-prerequisite"),
                    module.clone(),
                    VcOrderKey::new("order"),
                    vec![],
                    vec![],
                )];
                let submission = CompilerDriver::new(builder.build().unwrap())
                    .with_output_publisher(owner.clone())
                    .submit_captured(session.clone(), ids, snapshots, submit)
                    .unwrap();
                let (inputs, original) = native_disk_lookup_context(&submission, ids);
                let refs = submission
                    .scheduler_run
                    .as_ref()
                    .unwrap()
                    .phase_results
                    .values()
                    .flatten()
                    .flat_map(|row| row.output_refs.iter().cloned())
                    .collect::<Vec<_>>();
                assert_eq!(refs.len(), 11);
                let parent = |kind: &str| {
                    refs.iter()
                        .find(|parent| parent.output_kind() == &OutputKind::new(kind))
                        .unwrap()
                };
                let core_ref = parent("SourceTheoremCore");
                let vc_ref = parent("SourceTheoremVC");
                let discharge_ref = parent("SourceTheoremDischarge");
                let vcs = owner
                    .storage()
                    .get(
                        &owner
                            .storage()
                            .typed_handle::<VcSet>(vc_ref, vc_ref.output_kind())
                            .unwrap(),
                    )
                    .unwrap();
                let [vc] = vcs.vcs() else { panic!() };
                let selected = vc.id;
                let module_input = PhaseInput::new(
                    original.work_unit.clone(),
                    PhaseDispatchInputBundle::new(
                        old.snapshot,
                        original.identities().input_hash(),
                        original.identities().dependency_hashes().to_vec(),
                        [core_ref, vc_ref, discharge_ref]
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
                if case == "baseline" {
                    let core = owner
                        .storage()
                        .get(
                            &owner
                                .storage()
                                .typed_handle::<CoreIr>(core_ref, core_ref.output_kind())
                                .unwrap(),
                        )
                        .unwrap();
                    let discharge = owner
                        .storage()
                        .get(
                            &owner
                                .storage()
                                .typed_handle::<DischargeOutput>(
                                    discharge_ref,
                                    discharge_ref.output_kind(),
                                )
                                .unwrap(),
                        )
                        .unwrap();
                    let expected_pair = build(
                        &core,
                        &vcs,
                        selected,
                        &discharge,
                        &policy,
                        profile.clone(),
                        ConcreteFormat::Tptp,
                    )
                    .unwrap();
                    let frontend_ref = parent("FrontendOutput");
                    let frontend = owner.storage().get(&owner.storage().typed_handle::<FrontendOutput<<MizarParserSeam as ParserSeam>::Ast>>(
                    frontend_ref, frontend_ref.output_kind()).unwrap()).unwrap();
                    let ast = frontend.ast.as_ref().unwrap();
                    let (arena, env) = source_theorem_module_original_inputs(ast, core.module_id())
                        .unwrap()
                        .unwrap();
                    assert!(
                        source_theorem_module_candidates(ast, &arena, &env)
                            .unwrap()
                            .is_some()
                    );
                    let scope = SourceVariableScopeResolver::resolve_proof_occurrences(
                        SourceVariableScopeInput::new(ast, core.module_id(), &env),
                    )
                    .unwrap();
                    let metadata = check_source_theorem_metadata(&arena, &env, &scope).unwrap();
                    let (scope, labels, resolved) = source_proof_inputs(&arena, &env).unwrap();
                    let check = SourceVariableSemanticsChecker::check_theorem_skeletons(
                        &arena,
                        &metadata.2,
                        &scope,
                        &env,
                        &labels,
                        &resolved,
                        None,
                    )
                    .unwrap();
                    let expected = fresh(
                        &arena,
                        &check,
                        &env,
                        old.snapshot,
                        selected,
                        (&generation, &schema, profile.clone(), ConcreteFormat::Tptp),
                        (&policy, &verifier),
                    )
                    .unwrap();
                    assert_eq!(
                        (
                            &expected.0,
                            &expected.1,
                            &expected.2,
                            &expected.3,
                            &expected.4
                        ),
                        (
                            &*core,
                            &*vcs,
                            &*discharge,
                            &expected_pair.0,
                            &expected_pair.1
                        )
                    );
                    assert_eq!(
                        inputs
                            .native_theorem_candidate_with_core_vc_discharge_parents(
                                &module_input,
                                &owner,
                                selected,
                                &policy,
                                profile.clone(),
                                ConcreteFormat::Tptp
                            )
                            .unwrap(),
                        Some(expected_pair.clone())
                    );
                    let atp = inputs
                        .publish_native_theorem_candidate(
                            &module_input,
                            &owner,
                            selected,
                            &policy,
                            profile.clone(),
                            ConcreteFormat::Tptp,
                        )
                        .unwrap()
                        .unwrap();
                    assert_eq!(*owner.storage().get(&atp).unwrap(), expected_pair);
                    let mut four_parents = module_input.parent_outputs().to_vec();
                    four_parents.push(
                        SealedParentOutputHandle::from_published_output(
                            &owner,
                            old.snapshot,
                            atp.erase(),
                        )
                        .unwrap(),
                    );
                    let four = PhaseInput::new(
                        original.work_unit.clone(),
                        PhaseDispatchInputBundle::new(
                            old.snapshot,
                            original.identities().input_hash(),
                            original.identities().dependency_hashes().to_vec(),
                            four_parents,
                        )
                        .unwrap(),
                    );
                    let checked = inputs
                        .native_theorem_candidate_check_with_core_vc_discharge_atp_parents(
                            &four,
                            &owner,
                            selected,
                            (profile.clone(), ConcreteFormat::Tptp),
                            (&policy, &verifier),
                        )
                        .unwrap()
                        .unwrap();
                    assert_eq!(checked, (expected.5, expected.6));
                    assert!(
                        mizar_driver::native_theorem_candidate_check_storage_bytes(
                            ast,
                            core.module_id(),
                            &core,
                            (old.snapshot, &generation, &schema, selected),
                            (&policy, &profile, ConcreteFormat::Tptp),
                            &verifier,
                            (&expected_pair, &checked)
                        )
                        .unwrap()
                        .is_some()
                    );
                    assert_eq!(
                        inputs
                            .publish_native_theorem_candidate_check(
                                &four,
                                &owner,
                                selected,
                                (profile.clone(), ConcreteFormat::Tptp),
                                (&policy, &verifier)
                            )
                            .unwrap_err(),
                        PublishError::WorkUnitNotAllowed {
                            phase: IrPipelinePhase::new("KernelCheck"),
                            work_unit: unit.clone(),
                            output_kind: OutputKind::new("SourceTheoremCandidateCheck")
                        }
                        .to_string()
                    );
                    let old_error = fresh(
                        &arena,
                        &check,
                        &env,
                        old.snapshot,
                        selected,
                        (&generation, &schema, profile.clone(), ConcreteFormat::Tptp),
                        (&policy, &disabled),
                    )
                    .unwrap_err();
                    assert_eq!(
                        inputs
                            .native_theorem_candidate_check_with_core_vc_discharge_atp_parents(
                                &four,
                                &owner,
                                selected,
                                (profile.clone(), ConcreteFormat::Tptp),
                                (&policy, &disabled)
                            )
                            .unwrap_err(),
                        old_error
                    );
                    baseline_pair = Some(expected_pair);
                    let collected = owner.storage().collect(CollectInput {
                        snapshot: old.snapshot,
                        protected_outputs: vec![],
                    });
                    assert_eq!(
                        (collected.outputs_dropped, collected.abandoned_slots_dropped),
                        (12, 1)
                    );
                    continue;
                }
                let current_profile = if case == "unsupported-profile" {
                    LogicProfile::try_new(
                        "unsupported",
                        LogicFragment::TffLike,
                        EqualitySupport::Supported,
                        QuantifierPolicy::FirstOrder,
                        SoftTypeStrategy::ScopedGuardPredicates,
                        NativePropertySupport::Supported,
                        [ConcreteFormat::Tptp].into_iter().collect(),
                    )
                    .unwrap()
                } else {
                    profile.clone()
                };
                let raw = inputs.native_theorem_candidate_with_core_vc_discharge_parents(
                    &module_input,
                    &owner,
                    selected,
                    &policy,
                    current_profile.clone(),
                    ConcreteFormat::Tptp,
                );
                if case == "unsupported-profile" {
                    assert!(raw.is_err());
                } else {
                    assert_eq!(raw.unwrap(), baseline_pair.clone());
                }
                if case == "calibrate-atp-denied" {
                    assert_eq!(
                        inputs
                            .publish_native_theorem_candidate(
                                &module_input,
                                &owner,
                                selected,
                                &policy,
                                profile.clone(),
                                ConcreteFormat::Tptp
                            )
                            .unwrap_err(),
                        PublishError::WorkUnitNotAllowed {
                            phase: IrPipelinePhase::new("AtpSolve"),
                            work_unit: unit.clone(),
                            output_kind: OutputKind::new("SourceTheoremCandidateInputs")
                        }
                        .to_string()
                    );
                    let collected = owner.storage().collect(CollectInput {
                        snapshot: old.snapshot,
                        protected_outputs: vec![],
                    });
                    assert_eq!(
                        (collected.outputs_dropped, collected.abandoned_slots_dropped),
                        (11, 1)
                    );
                    continue;
                }
                let work_unit = match case {
                    "module" => original.work_unit.clone(),
                    "vc-unit" => WorkUnit::Vc {
                        module: module.clone(),
                        descriptor: VcTaskDescriptorId::new("opaque"),
                    },
                    _ => WorkUnit::EvidenceCandidate {
                        module: mizar_build::module_index::ModuleId::new(
                            PackageId::new("alpha"),
                            ModulePath::new(if case == "wrong-module" {
                                "other"
                            } else {
                                "main"
                            }),
                        ),
                        descriptor: VcTaskDescriptorId::new("opaque-check:09"),
                        evidence_candidate: EvidenceCandidateId::new("not-an-index"),
                    },
                };
                let mut parents = module_input.parent_outputs().to_vec();
                if case == "missing" {
                    parents.retain(|parent| {
                        parent.as_output_ref().output_kind()
                            != &OutputKind::new("SourceTheoremDischarge")
                    });
                }
                if case == "surplus" {
                    parents.push(
                        SealedParentOutputHandle::from_published_output(
                            &owner,
                            old.snapshot,
                            parent("FrontendOutput").clone(),
                        )
                        .unwrap(),
                    );
                }
                let query = PhaseInput::new(
                    work_unit,
                    PhaseDispatchInputBundle::new(
                        old.snapshot,
                        original.identities().input_hash(),
                        original.identities().dependency_hashes().to_vec(),
                        parents,
                    )
                    .unwrap(),
                );
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
                let current_verifier = if case == "disabled-policy" {
                    disabled.clone()
                } else {
                    verifier.clone()
                };
                let before = (
                    query.clone(),
                    policy.clone(),
                    current_profile.clone(),
                    current_verifier.clone(),
                    session.captured().clone(),
                );
                let mut builder = PhaseRegistryBuilder::new();
                builder
                    .register_source_theorem_candidate_check(
                        policy.clone(),
                        current_profile.clone(),
                        ConcreteFormat::Tptp,
                        current_verifier.clone(),
                    )
                    .unwrap();
                let registry = builder.build().unwrap();
                let cache = registry
                    .cache_key_for_phase(PipelinePhase::KernelCheck, &query)
                    .unwrap();
                assert_eq!(
                    cache.intent,
                    PhaseCacheIntent::NoKey {
                        reason:
                            "source theorem candidate check has no complete dependency footprint"
                                .to_owned()
                    }
                );
                let result = registry
                    .execute_phase_with_resources(
                        PipelinePhase::KernelCheck,
                        query.clone(),
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
                                    alien.clone()
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
                    result.result.output_refs.is_empty() && result.result.diagnostics.is_empty(),
                    "{case}"
                );
                assert_eq!(result.result.cache_observation, Some(result.observation));
                assert_eq!(
                    (
                        query,
                        policy.clone(),
                        current_profile,
                        current_verifier,
                        session.captured().clone()
                    ),
                    before
                );
                let retained = usize::from(matches!(case, "kernel-denied" | "disabled-policy"));
                let collected = owner.storage().collect(CollectInput {
                    snapshot: old.snapshot,
                    protected_outputs: vec![],
                });
                assert_eq!(
                    collected.outputs_dropped,
                    if case == "collected" {
                        0
                    } else {
                        11 + retained
                    },
                    "{case}"
                );
                assert_eq!(
                    collected.abandoned_slots_dropped,
                    usize::from(matches!(case, "atp-denied" | "kernel-denied")),
                    "{case}"
                );
                let empty_foreign = alien.storage().collect(CollectInput {
                    snapshot: old.snapshot,
                    protected_outputs: vec![],
                });
                assert_eq!(
                    (
                        empty_foreign.outputs_dropped,
                        empty_foreign.abandoned_slots_dropped
                    ),
                    (0, 0)
                );
            }
        },
    );
}

#[test]
fn scheduled_theorem_candidate_witness_files_preserve_pair_and_rights() {
    use mizar_atp::problem::*;
    use mizar_build::task_graph::{
        EvidenceCandidateId, VcOrderKey, VcTaskDescriptor, VcTaskDescriptorId,
    };
    use mizar_checker::type_checker::{
        SourceVariableSemanticsChecker, check_source_theorem_metadata,
    };
    use mizar_ir::storage::CollectInput;
    use mizar_proof::{
        policy::{PolicyDecision, VerifierPolicy},
        status::source_theorem_candidate_witness_file_candidate as fresh,
    };
    use mizar_resolve::{
        names::{SourceVariableScopeInput, SourceVariableScopeResolver},
        symbols::{source_proof_inputs, source_theorem_module_original_inputs},
    };
    use mizar_vc::{
        discharge::{ComputationLimit, DischargePolicy},
        vc_ir::{GenerationSchemaVersion, PolicyKey, VcSchemaVersion, VcSet},
    };
    let text = "theorem T: for X being set holds X = X proof let X be set; thus X = X; end;";
    with_native_theorem_witness_kernel(
        text,
        false,
        0,
        |fixture, inputs, four, baseline_owner, baseline_refs, (session, ids, snapshots)| {
            let policy = DischargePolicy {
                computation_limit: ComputationLimit {
                    policy: PolicyKey::new("explicit"),
                    max_steps: 64,
                },
            };
            let verifier = VerifierPolicy::default();
            let profile = LogicProfile::try_new(
                "witness-data",
                LogicFragment::Fof,
                EqualitySupport::Supported,
                QuantifierPolicy::FirstOrder,
                SoftTypeStrategy::ScopedGuardPredicates,
                NativePropertySupport::Supported,
                [ConcreteFormat::Tptp, ConcreteFormat::SmtLib]
                    .into_iter()
                    .collect(),
            )
            .unwrap();
            let kernel_ref = four
                .parent_outputs()
                .iter()
                .find(|row| {
                    row.as_output_ref().output_kind()
                        == &OutputKind::new("SourceTheoremCandidateCheck")
                })
                .unwrap()
                .as_output_ref();
            let kernel = baseline_owner
                .storage()
                .get(
                    &baseline_owner
                        .storage()
                        .typed_handle::<(mizar_kernel::checker::KernelCheckResult, PolicyDecision)>(
                            kernel_ref,
                            kernel_ref.output_kind(),
                        )
                        .unwrap(),
                )
                .unwrap();
            let vc_ref = four
                .parent_outputs()
                .iter()
                .find(|row| {
                    row.as_output_ref().output_kind() == &OutputKind::new("SourceTheoremVC")
                })
                .unwrap()
                .as_output_ref();
            let vcs = baseline_owner
                .storage()
                .get(
                    &baseline_owner
                        .storage()
                        .typed_handle::<VcSet>(vc_ref, vc_ref.output_kind())
                        .unwrap(),
                )
                .unwrap();
            let id = vcs.vcs()[0].id;
            assert!(
                inputs
                    .native_theorem_candidate_selection_with_core_vc_discharge_check_parents(
                        four,
                        baseline_owner,
                        id,
                        (profile.clone(), ConcreteFormat::Tptp),
                        (&policy, &verifier)
                    )
                    .unwrap()
                    .is_some()
            );
            let frontend_ref = baseline_refs
                .iter()
                .find(|row| row.output_kind() == &OutputKind::new("FrontendOutput"))
                .unwrap();
            let frontend = baseline_owner
                .storage()
                .get(
                    &baseline_owner
                        .storage()
                        .typed_handle::<FrontendOutput<<MizarParserSeam as ParserSeam>::Ast>>(
                            frontend_ref,
                            frontend_ref.output_kind(),
                        )
                        .unwrap(),
                )
                .unwrap();
            let ast = frontend.ast.as_ref().unwrap();
            let module = ResolverModuleId::new(PackageId::new("alpha"), ModulePath::new("main"));
            let (arena, env) = source_theorem_module_original_inputs(ast, &module)
                .unwrap()
                .unwrap();
            let variable_scope = SourceVariableScopeResolver::resolve_proof_occurrences(
                SourceVariableScopeInput::new(ast, &module, &env),
            )
            .unwrap();
            let metadata = check_source_theorem_metadata(&arena, &env, &variable_scope).unwrap();
            let (scope, labels, resolved) = source_proof_inputs(&arena, &env).unwrap();
            let check = SourceVariableSemanticsChecker::check_theorem_skeletons(
                &arena,
                &metadata.2,
                &scope,
                &env,
                &labels,
                &resolved,
                None,
            )
            .unwrap();
            let prefix = "proof-witnesses/λ";
            let kernel_hex = kernel_ref
                .content_hash()
                .as_bytes()
                .iter()
                .map(|byte| format!("{byte:02x}"))
                .collect::<String>();
            let path = format!("{prefix}/{kernel_hex}.json");
            let expected = fresh(
                &arena,
                &check,
                &env,
                session.captured().snapshot(),
                id,
                (
                    &vcs.vc(id).unwrap().anchor.generation_schema_version,
                    vcs.schema_version(),
                    profile.clone(),
                    ConcreteFormat::Tptp,
                ),
                (&policy, &verifier),
                &path,
            )
            .unwrap();
            let mut old_builder = PhaseRegistryBuilder::new();
            old_builder
                .register_source_theorem_candidate_check(
                    policy.clone(),
                    profile.clone(),
                    ConcreteFormat::Tptp,
                    verifier.clone(),
                )
                .unwrap();
            let old_registry = old_builder.build().unwrap();
            let old_descriptor = old_registry
                .descriptor_for_phase(PipelinePhase::KernelCheck)
                .unwrap();
            for invalid in [
                "",
                "other",
                "proof-witnesses/",
                "proof-witnesses/..",
                "proof-witnesses/a\\b",
            ] {
                let mut builder = PhaseRegistryBuilder::new();
                assert_eq!(
                    builder
                        .register_source_theorem_candidate_check_with_witness_files(
                            policy.clone(),
                            profile.clone(),
                            ConcreteFormat::Tptp,
                            verifier.clone(),
                            invalid.into()
                        )
                        .err()
                        .unwrap(),
                    "unsupported source theorem witness path prefix"
                );
                assert!(
                    builder
                        .build()
                        .unwrap()
                        .descriptor_for_phase(PipelinePhase::KernelCheck)
                        .is_err()
                );
            }
            for valid in ["proof-witnesses", prefix] {
                let mut builder = PhaseRegistryBuilder::new();
                builder
                    .register_source_theorem_candidate_check_with_witness_files(
                        policy.clone(),
                        profile.clone(),
                        ConcreteFormat::Tptp,
                        verifier.clone(),
                        valid.into(),
                    )
                    .unwrap();
                let registry = builder.build().unwrap();
                let descriptor = registry
                    .descriptor_for_phase(PipelinePhase::KernelCheck)
                    .unwrap();
                assert_eq!(
                    descriptor.schema_version,
                    format!(
                        "{};witness_file_path_prefix={}:{}",
                        old_descriptor.schema_version,
                        valid.len(),
                        valid
                    )
                );
                assert_eq!(descriptor.service_name, old_descriptor.service_name);
                assert_eq!(descriptor.owner, old_descriptor.owner);
                assert_eq!(descriptor.phases, old_descriptor.phases);
                assert_eq!(descriptor.output_kind, old_descriptor.output_kind);
            }
            for case in ["legacy", "allowed", "denied", "cancel", "foreign-token"] {
                let owner = publisher(four.snapshot, 0, &four.work_unit);
                allow_frontend(&owner, &four.work_unit);
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
                    ("KernelCheck", "SourceTheoremCandidateCheck"),
                ] {
                    owner.allow_work_unit(AllowedWorkUnit::new(
                        IrPipelinePhase::new(phase),
                        OutputKind::new(kind),
                        IrWorkUnit::new("\"alpha\":\"main\""),
                    ));
                }
                if case == "allowed" {
                    owner.allow_work_unit(AllowedWorkUnit::new(
                        IrPipelinePhase::new("KernelCheck"),
                        OutputKind::new("SourceTheoremCandidateWitnessFile"),
                        IrWorkUnit::new("\"alpha\":\"main\""),
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
                    .register_source_theorem_vc_generation(
                        GenerationSchemaVersion::new("λ\n\0"),
                        VcSchemaVersion::new(""),
                    )
                    .register_source_theorem_vc_discharge(policy.clone());
                let (_, mut submit) = fixture.submit_request(vec![], vec![]);
                let module = mizar_build::module_index::ModuleId::new(
                    PackageId::new("alpha"),
                    ModulePath::new("main"),
                );
                submit.vc_descriptors = vec![VcTaskDescriptor::new(
                    VcTaskDescriptorId::new("prerequisite"),
                    module.clone(),
                    VcOrderKey::new("order"),
                    vec![],
                    vec![],
                )];
                let submission = CompilerDriver::new(builder.build().unwrap())
                    .with_output_publisher(owner.clone())
                    .submit_captured(session.clone(), ids, snapshots, submit)
                    .unwrap();
                let (source_inputs, original) = native_disk_lookup_context(&submission, ids);
                let refs = submission
                    .scheduler_run
                    .as_ref()
                    .unwrap()
                    .phase_results
                    .values()
                    .flatten()
                    .flat_map(|row| row.output_refs.iter().cloned())
                    .collect::<Vec<_>>();
                assert_eq!(refs.len(), 11);
                let parents = [
                    "SourceTheoremCore",
                    "SourceTheoremVC",
                    "SourceTheoremDischarge",
                ]
                .map(|kind| {
                    refs.iter()
                        .find(|row| row.output_kind() == &OutputKind::new(kind))
                        .unwrap()
                });
                let query = PhaseInput::new(
                    WorkUnit::EvidenceCandidate {
                        module,
                        descriptor: VcTaskDescriptorId::new("opaque-current"),
                        evidence_candidate: EvidenceCandidateId::new("opaque-evidence"),
                    },
                    PhaseDispatchInputBundle::new(
                        four.snapshot,
                        original.identities().input_hash(),
                        original.identities().dependency_hashes().to_vec(),
                        parents
                            .into_iter()
                            .map(|row| {
                                SealedParentOutputHandle::from_published_output(
                                    &owner,
                                    four.snapshot,
                                    row.clone(),
                                )
                                .unwrap()
                            })
                            .collect(),
                    )
                    .unwrap(),
                );
                let mut builder = PhaseRegistryBuilder::new();
                if case == "legacy" {
                    builder
                        .register_source_theorem_candidate_check(
                            policy.clone(),
                            profile.clone(),
                            ConcreteFormat::Tptp,
                            verifier.clone(),
                        )
                        .unwrap();
                } else {
                    builder
                        .register_source_theorem_candidate_check_with_witness_files(
                            policy.clone(),
                            profile.clone(),
                            ConcreteFormat::Tptp,
                            verifier.clone(),
                            prefix.into(),
                        )
                        .unwrap();
                }
                let registry = builder.build().unwrap();
                let cache = registry
                    .cache_key_for_phase(PipelinePhase::KernelCheck, &query)
                    .unwrap();
                assert_eq!(
                    cache.intent,
                    PhaseCacheIntent::NoKey {
                        reason:
                            "source theorem candidate check has no complete dependency footprint"
                                .into()
                    }
                );
                assert_eq!(query.parent_outputs().len(), 3);
                let result = registry
                    .execute_phase_with_resources(
                        PipelinePhase::KernelCheck,
                        query.clone(),
                        PhaseExecutionResources {
                            output_publisher: Some(owner.clone()),
                            source_load: Some(source_inputs),
                            cancellation: matches!(case, "cancel" | "foreign-token").then_some(
                                CancellationToken {
                                    snapshot: if case == "cancel" {
                                        four.snapshot
                                    } else {
                                        snapshot_id(0xd5)
                                    },
                                    generation: CancellationGeneration::new(1),
                                    reason: CancellationReason::ExplicitRequest,
                                },
                            ),
                            ..PhaseExecutionResources::default()
                        },
                    )
                    .unwrap();
                assert_eq!(
                    result.result.status,
                    match case {
                        "legacy" | "allowed" => PhaseStatus::Complete,
                        "cancel" => PhaseStatus::Cancelled,
                        _ => PhaseStatus::Blocking,
                    },
                    "{case}"
                );
                assert!(result.result.diagnostics.is_empty());
                assert_eq!(result.result.cache_observation, Some(result.observation));
                let cancelled = matches!(case, "cancel" | "foreign-token");
                assert_eq!(
                    result.result.output_refs.len(),
                    if cancelled {
                        0
                    } else if case == "allowed" {
                        2
                    } else {
                        1
                    }
                );
                if !cancelled {
                    let actual_kernel = &result.result.output_refs[0];
                    assert_eq!(
                        actual_kernel.output_kind(),
                        &OutputKind::new("SourceTheoremCandidateCheck")
                    );
                    assert_eq!(*owner.storage().get(&owner.storage().typed_handle::<(mizar_kernel::checker::KernelCheckResult, PolicyDecision)>(actual_kernel, actual_kernel.output_kind()).unwrap()).unwrap(), *kernel);
                    assert_eq!(actual_kernel.content_hash(), kernel_ref.content_hash());
                    if case == "allowed" {
                        let file = &result.result.output_refs[1];
                        assert_eq!(
                            file.output_kind(),
                            &OutputKind::new("SourceTheoremCandidateWitnessFile")
                        );
                        assert_eq!(file.lineage().parents, [actual_kernel.output()]);
                        let handle = owner
                            .storage()
                            .typed_handle::<mizar_artifact::store::CanonicalJson>(
                                file,
                                file.output_kind(),
                            )
                            .unwrap();
                        assert_eq!(*owner.storage().get(&handle).unwrap(), expected.1);
                    }
                }
                let collected = owner.storage().collect(CollectInput {
                    snapshot: four.snapshot,
                    protected_outputs: vec![],
                });
                assert_eq!(
                    collected.outputs_dropped,
                    if cancelled {
                        11
                    } else if case == "allowed" {
                        14
                    } else {
                        13
                    },
                    "{case}"
                );
                assert_eq!(
                    collected.abandoned_slots_dropped,
                    usize::from(case == "denied"),
                    "{case}"
                );
            }
            let collected = baseline_owner.storage().collect(CollectInput {
                snapshot: four.snapshot,
                protected_outputs: vec![],
            });
            assert_eq!(
                (collected.outputs_dropped, collected.abandoned_slots_dropped),
                (13, 0)
            );
        },
    );
}
