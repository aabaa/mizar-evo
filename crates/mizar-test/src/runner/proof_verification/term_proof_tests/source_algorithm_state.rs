use super::*;

#[test]
fn source_algorithm_state_transitivity_checks_genuine_targets() {
    use mizar_checker::type_checker::check_source_algorithm_types;
    use mizar_core::elaborator::lower_source_algorithms;
    use mizar_kernel::{
        certificate_parser::{ClauseTautologyPolicy, Fingerprint, KernelProfileRecord},
        checker::{
            FormulaEvidenceContext, ImportedFactContextLimits, KernelCheckPolicy,
            KernelCheckStatus, KernelContextIdentityEntry, KernelContextIdentityPayload,
            KernelContextIdentitySource, KernelEvidenceCheckInput, KernelEvidenceCheckKind,
            KernelEvidenceCheckLimits, KernelFormulaProducerRef, KernelVcGeneratedFormulaId,
            check_kernel_evidence,
        },
        clause::VariableId,
        formula_evidence::{FormulaEvidenceParseContext, parse_formula_evidence},
        rejection::TargetVcFingerprint,
    };
    use mizar_proof::{
        policy::{
            AcceptedGoalPolarity, KernelEvidenceOrigin, KernelPolicyInput, PolicyCandidate,
            ProofPolicyEvaluator, VerifierPolicy,
        },
        status::check_source_algorithm_state_transitivity as verify,
    };
    use mizar_vc::{
        generator::generate_source_algorithm_postconditions,
        kernel_evidence_handoff::{
            KernelClauseTautologyPolicy, KernelContextIdentitySource as SourceContext,
            build_source_algorithm_state_kernel_handoff,
        },
        vc_ir::{
            AnchorCompleteness, AnchorIngredient, GenerationSchemaVersion, HashMarker,
            SeedVcMapping, VcFormulaRef, VcSchemaVersion, VcStatus,
        },
    };

    let config = config();
    let plan = crate::harness::build_test_plan(&config).unwrap();
    let case = plan
        .cases
        .iter()
        .find(|case| case.id.0 == "pass_proof_verification_algorithm_var_const_assert_001")
        .unwrap();
    let original = fs::read_to_string(&case.source_path).unwrap();
    assert_eq!(original.matches("    assert x = a;\n").count(), 1);
    let flat = original.replacen("    assert x = a;\n", "", 1);
    let renamed = flat
        .replace("let a be", "let input be")
        .replace("swapalgo(a)", "copyalgo(input)")
        .replace("var x :=", "var local :=")
        .replace("const c :=", "const saved :=")
        .replace("x := c;", "local := saved;")
        .replace("return x;", "return local;")
        .replace("result = a", "result = input")
        .replace(":= a;", ":= input;");
    let variants = [
        (&flat, "000000010300000000010000000301000000020100000000"),
        (&renamed, "000000010300000000010000000301000000020100000000"),
        (
            &flat.replace("    x := c;", "    const d := a;\n    x := c;"),
            "000000010300000000010000000401000000020100000000",
        ),
    ];
    let generation = GenerationSchemaVersion::new("mizar-vc-generation-step5c14-state-v1");
    let schema = VcSchemaVersion::new("mizar-vc-vcset-step5c14-state-v1");
    for (variant, (text, literal_name)) in variants.iter().enumerate() {
        let (source, nodes, symbols) = super::super::super::source_registration_inputs(
            &config.workspace_root,
            case,
            super::super::super::formula_statement::step5c8_test_frontend(text),
        )
        .unwrap();
        let prior_inputs = (source.clone(), nodes.clone(), symbols.clone());
        let check = check_source_algorithm_types(&source, &nodes, &symbols).unwrap();
        let core = lower_source_algorithms(&check).unwrap();
        let cases = if variant == 0 {
            vec![
                (0, VerifierPolicy::default()),
                (1, VerifierPolicy::default()),
                (0, VerifierPolicy::development()),
                (0, VerifierPolicy::interactive()),
            ]
        } else {
            vec![(0, VerifierPolicy::default())]
        };
        for (snapshot, policy) in cases {
            let snapshot = snapshot_id(snapshot);
            let vcs =
                generate_source_algorithm_postconditions(&core, snapshot, &generation, &schema)
                    .unwrap();
            let prior = (core.clone(), vcs.clone(), policy.clone());
            let [vc] = vcs.vcs() else {
                panic!("one genuine state VC")
            };
            assert_eq!(vc.status, VcStatus::Open);
            assert!(
                matches!(&vc.anchor.completeness, AnchorCompleteness::Incomplete { missing }
                if missing == &[AnchorIngredient::CanonicalGoalHash, AnchorIngredient::CanonicalContextHash])
            );
            assert!(matches!(
                &vc.anchor.canonical_goal_hash,
                HashMarker::ConservativeUnknown { .. }
            ));
            assert!(matches!(
                &vc.anchor.canonical_context_hash,
                HashMarker::ConservativeUnknown { .. }
            ));
            let [termination, contract] = vcs.seed_accounting() else {
                panic!("state accounting")
            };
            assert_eq!(
                termination.seed_status,
                mizar_core::core_ir::ObligationSeedStatus::Deferred
            );
            assert!(matches!(
                &termination.mapping,
                SeedVcMapping::NoConcreteVc { .. }
            ));
            assert!(matches!(&contract.mapping, SeedVcMapping::One { vc: id } if *id == vc.id));
            let handoff =
                build_source_algorithm_state_kernel_handoff(&core, &vcs, vc.id, &generation)
                    .unwrap();
            let envelope = handoff.canonical_evidence();
            let target = envelope.target_vc();
            let profile = envelope.kernel_profile();
            let tautology = match profile.clause_tautology_policy {
                KernelClauseTautologyPolicy::Reject => ClauseTautologyPolicy::Reject,
                KernelClauseTautologyPolicy::Marker => ClauseTautologyPolicy::Marker,
                _ => panic!("unsupported actual profile"),
            };
            let mut parse_context = FormulaEvidenceParseContext::v1(
                Fingerprint::new(target.algorithm_id, target.digest.clone()),
                KernelProfileRecord::v1(profile.profile_id, tautology),
            );
            let parsed_v1 = parse_formula_evidence(
                &handoff.encode_local_evidence(None).unwrap(),
                &parse_context,
            )
            .unwrap();
            let names = mizar_atp::backend::encode_local_variable_transitivity_names(
                &parsed_v1,
                VariableId(2),
            )
            .unwrap();
            assert_eq!(
                names[0]
                    .iter()
                    .map(|byte| format!("{byte:02x}"))
                    .collect::<String>(),
                *literal_name
            );
            parse_context.accepted_schema_versions.insert(2);
            let parsed = parse_formula_evidence(
                &handoff.encode_local_evidence(Some(&names)).unwrap(),
                &parse_context,
            )
            .unwrap();
            let target = TargetVcFingerprint::new(target.algorithm_id, target.digest.clone());
            let entries = handoff
                .context_identity()
                .entries()
                .iter()
                .map(|entry| {
                    let SourceContext::LocalHypothesis { local_context_id } = entry.source() else {
                        panic!("actual local state")
                    };
                    let producer = match entry.producer_formula_ref() {
                        VcFormulaRef::Core(id) => KernelFormulaProducerRef::Core(id),
                        VcFormulaRef::Generated(id) => KernelFormulaProducerRef::Generated(
                            KernelVcGeneratedFormulaId::new(id.index()),
                        ),
                        _ => panic!("actual state producer"),
                    };
                    KernelContextIdentityEntry::new(
                        KernelContextIdentitySource::LocalHypothesis { local_context_id },
                        entry.formula_id(),
                        Fingerprint::new(
                            entry.formula_fingerprint().algorithm_id,
                            entry.formula_fingerprint().digest.clone(),
                        ),
                        producer,
                    )
                })
                .collect();
            let context = FormulaEvidenceContext::with_context_identity(
                Some(handoff.context_identity_hash().as_bytes().to_vec()),
                vec![],
                vec![],
                Some(KernelContextIdentityPayload::new(
                    target.clone(),
                    handoff.canonical_hash(),
                    handoff.context_identity_hash(),
                    entries,
                )),
                ImportedFactContextLimits::default(),
            )
            .unwrap();
            let evaluator = ProofPolicyEvaluator::new(policy.clone());
            assert!(evaluator.can_schedule_kernel_check(
                &PolicyCandidate::UncheckedBuiltinDischarge {
                    has_stable_kernel_representation: true
                },
            ));
            let expected_result = check_kernel_evidence(KernelEvidenceCheckInput {
                target_vc_fingerprint: &target,
                evidence: &parsed,
                formula_context: Some(&context),
                check_kind: KernelEvidenceCheckKind::ProofObligation,
                policy: KernelCheckPolicy::default(),
                limits: KernelEvidenceCheckLimits::default(),
            });
            let input = KernelPolicyInput::from_kernel_result(
                &expected_result,
                KernelEvidenceOrigin::BuiltinDischarge,
            );
            let expected_decision =
                evaluator.evaluate_candidate(&PolicyCandidate::KernelResult(input.clone()));
            let expected = (expected_result, expected_decision);
            assert_eq!(expected.0.status(), KernelCheckStatus::Accepted);
            assert_eq!(expected.0.target_vc_fingerprint(), &target);
            assert_eq!(
                expected.0.evidence_check_kind(),
                Some(KernelEvidenceCheckKind::ProofObligation)
            );
            assert!(!expected.0.policy_taint());
            assert!(expected.0.used_axioms().is_empty());
            assert!(input.accepted_evidence_hash().is_some());
            assert_eq!(
                input.accepted_goal_polarity(),
                Some(AcceptedGoalPolarity::AssertFalseForRefutation)
            );
            let actual = verify(
                &source,
                &nodes,
                &symbols,
                snapshot,
                vc.id,
                VariableId(2),
                &policy,
            )
            .unwrap();
            assert_eq!(actual, expected);
            assert_eq!(
                actual,
                verify(
                    &source,
                    &nodes,
                    &symbols,
                    snapshot,
                    vc.id,
                    VariableId(2),
                    &policy
                )
                .unwrap()
            );
            assert_eq!((&core, &vcs, &policy), (&prior.0, &prior.1, &prior.2));
        }
        assert_eq!(
            (&source, &nodes, &symbols),
            (&prior_inputs.0, &prior_inputs.1, &prior_inputs.2)
        );
    }
}

#[test]
fn source_algorithm_state_transitivity_preserves_upstream_and_policy_errors() {
    use mizar_checker::type_checker::check_source_algorithm_types;
    use mizar_core::elaborator::lower_source_algorithms;
    use mizar_kernel::clause::VariableId;
    use mizar_proof::{
        policy::{KernelEvidenceFormat, VerifierPolicy},
        status::check_source_algorithm_state_transitivity as verify,
    };
    use mizar_vc::{
        generator::generate_source_algorithm_postconditions,
        kernel_evidence_handoff::build_source_algorithm_state_kernel_handoff,
        vc_ir::{GenerationSchemaVersion, VcId, VcSchemaVersion},
    };
    let config = config();
    let plan = crate::harness::build_test_plan(&config).unwrap();
    let case = plan
        .cases
        .iter()
        .find(|case| case.id.0 == "pass_proof_verification_algorithm_var_const_assert_001")
        .unwrap();
    let original = fs::read_to_string(&case.source_path).unwrap();
    let flat = original.replacen("    assert x = a;\n", "", 1);
    let generation = GenerationSchemaVersion::new("mizar-vc-generation-step5c14-state-v1");
    let schema = VcSchemaVersion::new("mizar-vc-vcset-step5c14-state-v1");
    let disabled = VerifierPolicy::default().with_kernel_evidence_formats([]);
    for (text, id) in [
        (flat.clone(), VcId::new(99)),
        (flat.replace("    ensures result = a\n", ""), VcId::new(0)),
        (original, VcId::new(0)),
    ] {
        let (source, nodes, symbols) = super::super::super::source_registration_inputs(
            &config.workspace_root,
            case,
            super::super::super::formula_statement::step5c8_test_frontend(&text),
        )
        .unwrap();
        let check = check_source_algorithm_types(&source, &nodes, &symbols).unwrap();
        let core = lower_source_algorithms(&check).unwrap();
        let vcs =
            generate_source_algorithm_postconditions(&core, snapshot_id(0), &generation, &schema)
                .unwrap();
        let expected =
            build_source_algorithm_state_kernel_handoff(&core, &vcs, id, &generation).unwrap_err();
        assert_eq!(
            verify(
                &source,
                &nodes,
                &symbols,
                snapshot_id(0),
                id,
                VariableId(2),
                &disabled
            )
            .unwrap_err(),
            expected
        );
    }
    let return_case = plan
        .cases
        .iter()
        .find(|case| case.id.0 == "pass_proof_verification_algorithm_ensures_return_001")
        .unwrap();
    let return_source = fs::read_to_string(&return_case.source_path).unwrap();
    let two_returns = return_source
        .replace("    ensures result = a\n", "")
        .replace("return a;", "return a;\n    return a;");
    let (source, nodes, symbols) = super::super::super::source_registration_inputs(
        &config.workspace_root,
        return_case,
        super::super::super::formula_statement::step5c8_test_frontend(&two_returns),
    )
    .unwrap();
    let check = check_source_algorithm_types(&source, &nodes, &symbols).unwrap();
    let core = lower_source_algorithms(&check).unwrap();
    let expected =
        generate_source_algorithm_postconditions(&core, snapshot_id(0), &generation, &schema)
            .unwrap_err();
    assert_eq!(
        verify(
            &source,
            &nodes,
            &symbols,
            snapshot_id(0),
            VcId::new(99),
            VariableId(2),
            &disabled
        )
        .unwrap_err(),
        expected
    );
    let bare_return = return_source.replace("return a;", "return;");
    let (source, nodes, symbols) = super::super::super::source_registration_inputs(
        &config.workspace_root,
        return_case,
        super::super::super::formula_statement::step5c8_test_frontend(&bare_return),
    )
    .unwrap();
    let expected = check_source_algorithm_types(&source, &nodes, &symbols).unwrap_err();
    assert_eq!(
        verify(
            &source,
            &nodes,
            &symbols,
            snapshot_id(0),
            VcId::new(99),
            VariableId(2),
            &disabled
        )
        .unwrap_err(),
        expected
    );
    let (source, nodes, symbols) = super::super::super::source_registration_inputs(
        &config.workspace_root,
        case,
        super::super::super::formula_statement::step5c8_test_frontend(&flat),
    )
    .unwrap();
    let check = check_source_algorithm_types(&source, &nodes, &symbols).unwrap();
    let core = lower_source_algorithms(&check).unwrap();
    let vcs = generate_source_algorithm_postconditions(&core, snapshot_id(0), &generation, &schema)
        .unwrap();
    let handoff =
        build_source_algorithm_state_kernel_handoff(&core, &vcs, vcs.vcs()[0].id, &generation)
            .unwrap();
    let target = handoff.canonical_evidence().target_vc();
    let profile = handoff.canonical_evidence().kernel_profile();
    let parsed = mizar_kernel::formula_evidence::parse_formula_evidence(
        &handoff.encode_local_evidence(None).unwrap(),
        &mizar_kernel::formula_evidence::FormulaEvidenceParseContext::v1(
            mizar_kernel::certificate_parser::Fingerprint::new(
                target.algorithm_id,
                target.digest.clone(),
            ),
            mizar_kernel::certificate_parser::KernelProfileRecord::v1(
                profile.profile_id,
                mizar_kernel::certificate_parser::ClauseTautologyPolicy::Reject,
            ),
        ),
    )
    .unwrap();
    for middle in [VariableId(1), VariableId(99)] {
        let expected =
            mizar_atp::backend::encode_local_variable_transitivity_names(&parsed, middle)
                .unwrap_err();
        assert_eq!(
            verify(
                &source,
                &nodes,
                &symbols,
                snapshot_id(0),
                vcs.vcs()[0].id,
                middle,
                &disabled
            )
            .unwrap_err(),
            expected
        );
    }
    for policy in [
        disabled,
        VerifierPolicy::default()
            .with_kernel_evidence_formats([KernelEvidenceFormat::FormulaSubstitution]),
    ] {
        assert_eq!(
            verify(
                &source,
                &nodes,
                &symbols,
                snapshot_id(0),
                vcs.vcs()[0].id,
                VariableId(2),
                &policy
            )
            .unwrap_err(),
            "algorithm.proof.kernel_policy_rejected"
        );
    }
}
#[test]
fn source_algorithm_state_canonical_components_preserve_genuine_content() {
    use mizar_checker::type_checker::check_source_algorithm_types;
    use mizar_core::{
        core_ir::ObligationSeedStatus,
        elaborator::{lower_source_algorithms, source_algorithm_formula_payloads},
    };
    use mizar_vc::{
        generator::generate_source_algorithm_postconditions,
        kernel_evidence_handoff::build_source_algorithm_state_kernel_handoff,
        vc_ir::{
            AnchorCompleteness, AnchorIngredient, GenerationSchemaVersion, HashMarker,
            SeedVcMapping, VcGeneratedFormulaShape, VcSchemaVersion, VcStatus,
        },
    };
    let config = config();
    let plan = crate::harness::build_test_plan(&config).unwrap();
    let case = plan
        .cases
        .iter()
        .find(|case| case.id.0 == "pass_proof_verification_algorithm_var_const_assert_001")
        .unwrap();
    let original = fs::read_to_string(&case.source_path).unwrap();
    assert_eq!(original.matches("    assert x = a;\n").count(), 1);
    let flat = original.replacen("    assert x = a;\n", "", 1);
    let renamed = flat
        .replace("let a be", "let input be")
        .replace("swapalgo(a)", "copyalgo(input)")
        .replace("var x :=", "var local :=")
        .replace("const c :=", "const saved :=")
        .replace("x := c;", "local := saved;")
        .replace("return x;", "return local;")
        .replace("result = a", "result = input")
        .replace(":= a;", ":= input;");
    let independent = flat.replace("    x := c;", "    const d := a;\n    x := c;");
    for (variant, text) in [flat, renamed, independent].into_iter().enumerate() {
        let (source, typed, symbols) = super::super::super::source_registration_inputs(
            &config.workspace_root,
            case,
            super::super::super::formula_statement::step5c8_test_frontend(&text),
        )
        .unwrap();
        let before_inputs = (source.clone(), typed.clone(), symbols.clone());
        let check = check_source_algorithm_types(&source, &typed, &symbols).unwrap();
        let core = lower_source_algorithms(&check).unwrap();
        let algorithm = &core.algorithms().iter().next().unwrap().1;
        assert_eq!(algorithm.statements.len(), if variant == 2 { 5 } else { 4 });
        let guard = algorithm.params[0].ty_guard.unwrap();
        let formulas = source_algorithm_formula_payloads(&check, &core, &[guard]).unwrap();
        assert_eq!(formulas.len(), 1);
        assert_eq!(
            formulas[&guard].len(),
            if variant == 2 { 1107 } else { 1091 }
        );
        let vc_base = "mizar-vc-vcset-step5c14-state-v1";
        let generation_base = "mizar-vc-generation-step5c14-state-v1";
        let schemas = [
            (vc_base, generation_base),
            ("mizar-vc-vcset-step5c14-state-v1-changed", generation_base),
            (vc_base, "mizar-vc-generation-step5c14-state-v1-changed"),
            ("", " \n世代\t\0"),
        ];
        for (schema_index, (vc_schema, generation_schema)) in schemas
            .into_iter()
            .take(if variant == 0 { 4 } else { 1 })
            .enumerate()
        {
            let generation = GenerationSchemaVersion::new(generation_schema);
            let schema = VcSchemaVersion::new(vc_schema);
            let snapshot = snapshot_id(usize::from(variant == 0 && schema_index == 1));
            let vcs =
                generate_source_algorithm_postconditions(&core, snapshot, &generation, &schema)
                    .unwrap();
            let [vc] = vcs.vcs() else {
                panic!("one actual state VC")
            };
            assert_eq!(vc.status, VcStatus::Open);
            let expected_length = if variant == 2 { 9 } else { 7 };
            assert_eq!(vcs.generated_formulas().len(), expected_length);
            assert_eq!(
                vc.local_context.entries().len(),
                vcs.generated_formulas().len()
            );
            let VcGeneratedFormulaShape::ProgramEquals { left, .. } =
                vcs.generated_formulas()[0].shape
            else {
                panic!("first write fact")
            };
            assert_eq!(left.definition, Some(algorithm.statements[0]));
            assert_eq!(vc.local_context.policy_inputs().len(), 6);
            assert_eq!(
                vc.local_context.policy_inputs()[3].value.as_str(),
                (algorithm.statements.len() - 1).to_string()
            );
            build_source_algorithm_state_kernel_handoff(&core, &vcs, vc.id, &generation).unwrap();
            assert!(vcs.canonical_vc_fingerprint(vc.id).is_none());
            assert!(vcs.local_context_fingerprint(vc.id).is_none());
            assert!(
                matches!(&vc.anchor.completeness, AnchorCompleteness::Incomplete { missing }
                if missing == &[AnchorIngredient::CanonicalGoalHash, AnchorIngredient::CanonicalContextHash])
            );
            assert!(matches!(
                (
                    &vc.anchor.canonical_goal_hash,
                    &vc.anchor.canonical_context_hash
                ),
                (
                    HashMarker::ConservativeUnknown { .. },
                    HashMarker::ConservativeUnknown { .. }
                )
            ));
            let [termination, contract] = vcs.seed_accounting() else {
                panic!("state accounting")
            };
            assert_eq!(termination.seed_status, ObligationSeedStatus::Deferred);
            assert!(matches!(
                &termination.mapping,
                SeedVcMapping::NoConcreteVc { .. }
            ));
            assert!(matches!(&contract.mapping, SeedVcMapping::One { vc: id } if *id == vc.id));
            let before = (core.clone(), vcs.clone(), formulas.clone());
            let actual = vcs
                .source_algorithm_state_canonical_components(vc.id, &core, &generation, &formulas)
                .unwrap();
            let digest = |bytes: &[u8]| {
                bytes
                    .iter()
                    .map(|byte| format!("{byte:02x}"))
                    .collect::<String>()
            };
            let actual_hex = [
                digest(actual.0.hash().as_bytes()),
                digest(actual.1.as_bytes()),
                digest(actual.2.hash().as_bytes()),
            ];
            let row = if variant == 0 {
                schema_index
            } else {
                variant + 3
            };
            let expected = [
                "a99236a729a9774bb09d0511ecba2563c6c916631e8b95414115cca16a2ea926|23be8c3fce29acaf31bdbe99c60680cd6ed0f2b97bfac00fcfa89e4ee70cd909|39def0868c5e0a2e393bf81946180d64234b563418e2ac8f67d3b82afa132ec6",
                "6a530e6729c50498509d1bde6b934706c035434eee638e8763730f9c15be0ed7|048529bae147158240f3ea9ce721c2c02df411ef5a6565bc78eed2760a65c23d|a01a7e7df8b0b18fc3b637990949521e67b14611426cd9d646400613d9490687",
                "4a691633104b8efbd036fabf0a4b031c70e00632af58bb8a992f0f9b2619d725|7155e83ae45739e770e25039637ae0b5fc2ddc05dc3147706ebe87e0476c606f|736e356e7ea27be3c3b619f34ce2f892faf1ce38f5181b4d102338c62a7415ce",
                "e57a4b6923335a3bae2e25a0baf4da893ac4fce488fa8ec7ba7d708b7728c71f|630237a2859bdcee516fd9c095632355c81d1bdf04a7c98e529a6c3d3b13bdf9|9b3f1d0359c93bcf5e2d6684b5370d0f69a4105386177a12691e7385578b327f",
                "9f40a766072eac6f25c0264396192061edd3d25fbf94e091466272b8e2ce2aa3|23be8c3fce29acaf31bdbe99c60680cd6ed0f2b97bfac00fcfa89e4ee70cd909|58e887ea695f260f67a0459a4975e145fb1be61fa65ede5b04685d9fb4648efb",
                "64771f5a91234d8613b892540592e00a0ae7421b97e7c5a9a0b3c9eec819b475|23be8c3fce29acaf31bdbe99c60680cd6edce6827ffb53cfcfa89e4ee70cd909|72e349ff495082ee367259b7a3874b14e521d60d9db04b17fe3164ee443a5711",
            ][row];
            assert_eq!(actual_hex.join("|"), expected);
            assert_eq!(
                actual,
                vcs.source_algorithm_state_canonical_components(
                    vc.id,
                    &core,
                    &generation,
                    &formulas
                )
                .unwrap()
            );
            assert_eq!(before, (core.clone(), vcs.clone(), formulas.clone()));
        }
        assert_eq!(before_inputs, (source, typed, symbols));
    }
}

#[test]
fn source_algorithm_state_canonical_components_preserve_source_and_schema_participants() {
    use mizar_checker::type_checker::check_source_algorithm_types;
    use mizar_core::elaborator::{lower_source_algorithms, source_algorithm_formula_payloads};
    use mizar_vc::{
        generator::generate_source_algorithm_postconditions,
        kernel_evidence_handoff::build_source_algorithm_state_kernel_handoff,
        vc_ir::{GenerationSchemaVersion, VcSchemaVersion},
    };
    let config = config();
    let plan = crate::harness::build_test_plan(&config).unwrap();
    let case = plan
        .cases
        .iter()
        .find(|case| case.id.0 == "pass_proof_verification_algorithm_var_const_assert_001")
        .unwrap();
    let original = fs::read_to_string(&case.source_path).unwrap();
    let flat = original.replacen("    assert x = a;\n", "", 1);
    let binder = flat
        .replace(" a ", " input ")
        .replace("(a)", "(input)")
        .replace(" a;", " input;")
        .replace("= a\n", "= input\n");
    let inputs = [
        flat.clone(),
        format!(":: neutral source trivia\n\n{flat}"),
        binder,
        flat.replace("    x := c;", "    x := a;"),
        flat.replace("return x;", "return c;"),
    ];
    let generation = GenerationSchemaVersion::new("mizar-vc-generation-step5c14-state-v1");
    let schema = VcSchemaVersion::new("mizar-vc-vcset-step5c14-state-v1");
    let mut baseline = None;
    for (index, text) in inputs.into_iter().enumerate() {
        let (source, typed, symbols) = super::super::super::source_registration_inputs(
            &config.workspace_root,
            case,
            super::super::super::formula_statement::step5c8_test_frontend(&text),
        )
        .unwrap();
        let check = check_source_algorithm_types(&source, &typed, &symbols).unwrap();
        let core = lower_source_algorithms(&check).unwrap();
        let guard = core.algorithms().iter().next().unwrap().1.params[0]
            .ty_guard
            .unwrap();
        let formulas = source_algorithm_formula_payloads(&check, &core, &[guard]).unwrap();
        let vcs = generate_source_algorithm_postconditions(
            &core,
            snapshot_id(index),
            &generation,
            &schema,
        )
        .unwrap();
        build_source_algorithm_state_kernel_handoff(&core, &vcs, vcs.vcs()[0].id, &generation)
            .unwrap();
        let actual = vcs
            .source_algorithm_state_canonical_components(
                vcs.vcs()[0].id,
                &core,
                &generation,
                &formulas,
            )
            .unwrap();
        if index == 0 {
            baseline = Some(actual);
        } else if index <= 2 {
            assert_eq!(actual, baseline.unwrap());
        } else if index == 3 {
            assert_ne!(actual.0, baseline.unwrap().0);
            assert_eq!(actual.1, baseline.unwrap().1);
            assert_ne!(actual.2, baseline.unwrap().2);
        } else {
            assert_ne!(actual.0, baseline.unwrap().0);
            assert_ne!(actual.1, baseline.unwrap().1);
        }
    }
}

#[test]
fn source_algorithm_state_canonical_components_preserve_upstream_and_atomic_refusals() {
    use mizar_checker::type_checker::check_source_algorithm_types;
    use mizar_core::elaborator::{lower_source_algorithms, source_algorithm_formula_payloads};
    use mizar_vc::{
        generator::generate_source_algorithm_postconditions,
        kernel_evidence_handoff::build_source_algorithm_state_kernel_handoff,
        vc_ir::{GenerationSchemaVersion, LocalContext, VcId, VcSchemaVersion, VcSet, VcSetParts},
    };
    let config = config();
    let plan = crate::harness::build_test_plan(&config).unwrap();
    let case = plan
        .cases
        .iter()
        .find(|case| case.id.0 == "pass_proof_verification_algorithm_var_const_assert_001")
        .unwrap();
    let original = fs::read_to_string(&case.source_path).unwrap();
    let flat = original.replacen("    assert x = a;\n", "", 1);
    let (source, typed, symbols) = super::super::super::source_registration_inputs(
        &config.workspace_root,
        case,
        super::super::super::formula_statement::step5c8_test_frontend(&flat),
    )
    .unwrap();
    let check = check_source_algorithm_types(&source, &typed, &symbols).unwrap();
    let core = lower_source_algorithms(&check).unwrap();
    let algorithm = &core.algorithms().iter().next().unwrap().1;
    let guard = algorithm.params[0].ty_guard.unwrap();
    let formulas = source_algorithm_formula_payloads(&check, &core, &[guard]).unwrap();
    let generation = GenerationSchemaVersion::new("mizar-vc-generation-step5c14-state-v1");
    let schema = VcSchemaVersion::new("mizar-vc-vcset-step5c14-state-v1");
    let vcs = generate_source_algorithm_postconditions(&core, snapshot_id(0), &generation, &schema)
        .unwrap();
    let id = vcs.vcs()[0].id;
    build_source_algorithm_state_kernel_handoff(&core, &vcs, id, &generation).unwrap();
    let empty = std::collections::BTreeMap::new();
    let invalid = "unsupported source algorithm state canonical components";
    for (target, map) in [(VcId::new(99), &formulas), (id, &empty)] {
        let expected = if target == id {
            invalid.to_owned()
        } else {
            build_source_algorithm_state_kernel_handoff(&core, &vcs, target, &generation)
                .unwrap_err()
        };
        assert_eq!(
            vcs.source_algorithm_state_canonical_components(target, &core, &generation, map)
                .unwrap_err(),
            expected
        );
    }
    let mut empty_value = formulas.clone();
    empty_value.insert(guard, Vec::new());
    let mut extra = formulas.clone();
    extra.insert(algorithm.contracts.ensures[0], b"extra".to_vec());
    let mut wrong_root = empty.clone();
    wrong_root.insert(algorithm.contracts.ensures[0], formulas[&guard].clone());
    for map in [&empty_value, &extra, &wrong_root] {
        assert_eq!(
            vcs.source_algorithm_state_canonical_components(id, &core, &generation, map)
                .unwrap_err(),
            invalid
        );
    }
    for drift_kind in 0..3 {
        let mut parts = VcSetParts {
            schema_version: vcs.schema_version().clone(),
            snapshot: vcs.snapshot(),
            source: vcs.source(),
            module: vcs.module().clone(),
            generated_formulas: vcs.generated_formulas().to_vec(),
            vcs: vcs.vcs().to_vec(),
            seed_accounting: vcs.seed_accounting().to_vec(),
        };
        let mut entries = parts.vcs[0].local_context.entries().to_vec();
        let mut policy = parts.vcs[0].local_context.policy_inputs().to_vec();
        match drift_kind {
            0 => entries[1].sort_key = "algorithm-state-00000001-drift".into(),
            1 => entries[1].provenance[0].key = "different".into(),
            _ => policy[3].value = "999".into(),
        }
        parts.vcs[0].local_context = LocalContext::try_new(entries, policy).unwrap();
        let drift = VcSet::try_new(parts).unwrap();
        let expected = build_source_algorithm_state_kernel_handoff(&core, &drift, id, &generation)
            .unwrap_err();
        assert_eq!(
            drift
                .source_algorithm_state_canonical_components(id, &core, &generation, &empty)
                .unwrap_err(),
            expected
        );
    }
    let foreign = flat.replace("swapalgo", "foreign_algorithm");
    let (foreign_source, foreign_typed, foreign_symbols) =
        super::super::super::source_registration_inputs(
            &config.workspace_root,
            case,
            super::super::super::formula_statement::step5c8_test_frontend(&foreign),
        )
        .unwrap();
    let foreign_check =
        check_source_algorithm_types(&foreign_source, &foreign_typed, &foreign_symbols).unwrap();
    let foreign_core = lower_source_algorithms(&foreign_check).unwrap();
    let foreign_vcs = generate_source_algorithm_postconditions(
        &foreign_core,
        snapshot_id(0),
        &generation,
        &schema,
    )
    .unwrap();
    build_source_algorithm_state_kernel_handoff(
        &foreign_core,
        &foreign_vcs,
        foreign_vcs.vcs()[0].id,
        &generation,
    )
    .unwrap();
    let expected =
        build_source_algorithm_state_kernel_handoff(&foreign_core, &vcs, id, &generation)
            .unwrap_err();
    assert_eq!(
        vcs.source_algorithm_state_canonical_components(id, &foreign_core, &generation, &empty)
            .unwrap_err(),
        expected
    );
    for text in [flat.replace("    ensures result = a\n", ""), original] {
        let (source, typed, symbols) = super::super::super::source_registration_inputs(
            &config.workspace_root,
            case,
            super::super::super::formula_statement::step5c8_test_frontend(&text),
        )
        .unwrap();
        let check = check_source_algorithm_types(&source, &typed, &symbols).unwrap();
        let other_core = lower_source_algorithms(&check).unwrap();
        let other_vcs = generate_source_algorithm_postconditions(
            &other_core,
            snapshot_id(0),
            &generation,
            &schema,
        )
        .unwrap();
        let expected = build_source_algorithm_state_kernel_handoff(
            &other_core,
            &other_vcs,
            VcId::new(0),
            &generation,
        )
        .unwrap_err();
        assert_eq!(
            other_vcs
                .source_algorithm_state_canonical_components(
                    VcId::new(0),
                    &other_core,
                    &generation,
                    &empty
                )
                .unwrap_err(),
            expected
        );
    }
    let return_case = plan
        .cases
        .iter()
        .find(|case| case.id.0 == "pass_proof_verification_algorithm_ensures_return_001")
        .unwrap();
    let return_source = fs::read_to_string(&return_case.source_path).unwrap();
    let two_returns = return_source
        .replace("    ensures result = a\n", "")
        .replace("return a;", "return a;\n    return a;");
    let (source, typed, symbols) = super::super::super::source_registration_inputs(
        &config.workspace_root,
        return_case,
        super::super::super::formula_statement::step5c8_test_frontend(&two_returns),
    )
    .unwrap();
    let check = check_source_algorithm_types(&source, &typed, &symbols).unwrap();
    let two_core = lower_source_algorithms(&check).unwrap();
    let expected =
        generate_source_algorithm_postconditions(&two_core, snapshot_id(0), &generation, &schema)
            .unwrap_err();
    assert_eq!(expected, "algorithms.postcondition.unsupported_core");
    assert_eq!(
        vcs.source_algorithm_state_canonical_components(
            VcId::new(99),
            &two_core,
            &generation,
            &empty
        )
        .unwrap_err(),
        expected
    );
}

#[test]
fn source_algorithm_state_obligation_identity_preserves_genuine_fields_and_slice() {
    use mizar_artifact::store::{CanonicalJson as Json, canonical_json_bytes};
    use mizar_checker::type_checker::check_source_algorithm_types;
    use mizar_core::elaborator::{
        lower_source_algorithms as lower, source_algorithm_core_semantic_payload as semantic,
        source_algorithm_formula_payloads as guards,
    };
    use mizar_proof::status::source_algorithm_state_obligation_identity as identify;
    use mizar_resolve::symbols::source_algorithm_state_syntax_payloads_from_arena as syntax_from;
    use mizar_vc::{
        dependency_slice::{DependencySliceInput, try_compute_dependency_slices},
        generator::generate_source_algorithm_postconditions as generate,
        kernel_evidence_handoff::build_source_algorithm_state_kernel_handoff as handoff,
        vc_ir::source_algorithm_state_candidate_anchor_hash as candidate_hash,
    };
    let original = fs::read_to_string(config().workspace_root.join(
        "tests/miz/pass/algorithms/pass_proof_verification_algorithm_var_const_assert_001.miz",
    ))
    .unwrap();
    assert_eq!(original.matches("    assert x = a;\n").count(), 1);
    let flat = original.replacen("    assert x = a;\n", "", 1);
    let renamed = flat
        .replace("let a be", "let input be")
        .replace("swapalgo(a)", "copyalgo(input)")
        .replace("var x :=", "var local :=")
        .replace("const c :=", "const saved :=")
        .replace("x := c;", "local := saved;")
        .replace("return x;", "return local;")
        .replace("result = a", "result = input")
        .replace(":= a;", ":= input;");
    let schema = VcSchemaVersion::new("mizar-vc-vcset-step5c14-state-v1");
    for (text, candidate_hex, final_hex) in [
        (
            flat.clone(),
            "e5ea8aded6a61a092f8614873d1f0efebef85ac9968dc9b2e13bd3363c1a3544",
            "abbcc873eb36ff245f37d1325d6fbdb65f9e033e65a9d3ba2c14950c2f1282b9",
        ),
        (
            renamed,
            "22254f008c24618adfb0934803dcd8ae53f90da81ef95a55cfa532da81c17d18",
            "fcf65b9cba542e5af3f8ff77b45022e79f5c2fc5310e4d496b01553a7b4f404a",
        ),
        (
            flat.replace("    x := c;", "    const d := a;\n    x := c;"),
            "ea9790dc8ad7a28a936ca7929daf31fe74ce21939b6dc694df446329093461b0",
            "2f833d6e863cb9e05144d3f79190aa2dc82898fec348c023abf33b2064fae7dc",
        ),
        (
            flat.replace("terminating algorithm", "algorithm"),
            "211e70aa92795c7d3ea9e97ab48bb03d8a2e99c39bbd0fbfb3ede6509c109fff",
            "c6ccea721c8458d54b3f9831a5d54b175195601e686eda721783f5685700c697",
        ),
    ] {
        with_source_algorithm_identity_inputs(
            &text,
            "mizar-test-corpus",
            "tests.miz.pass.algorithms.pass_proof_verification_algorithm_var_const_assert_001",
            0,
            |source, typed, symbols, check| {
                let before = format!("{source:?}{typed:?}{symbols:?}{check:?}");
                let fresh = check_source_algorithm_types(source, check.typed(), symbols).unwrap();
                let core = lower(&fresh).unwrap();
                let generation = "mizar-vc-generation-step5c14-state-v1".into();
                let vcs = generate(&core, snapshot_id(0), &generation, &schema).unwrap();
                let [target] = vcs.vcs() else {
                    panic!("one genuine state VC")
                };
                assert_eq!(target.status, mizar_vc::vc_ir::VcStatus::Open);
                assert_eq!(
                    vcs.seed_accounting()[0].seed_status,
                    mizar_core::core_ir::ObligationSeedStatus::Deferred
                );
                let roots = [core.algorithms().iter().next().unwrap().1.params[0]
                    .ty_guard
                    .unwrap()];
                let formulas = guards(&fresh, &core, &roots).unwrap();
                handoff(&core, &vcs, target.id, &generation).unwrap();
                let old = vcs
                    .source_algorithm_state_canonical_components(
                        target.id,
                        &core,
                        &generation,
                        &formulas,
                    )
                    .unwrap();
                let slices = try_compute_dependency_slices(DependencySliceInput {
                    vc_set: &vcs,
                    discharge_output: None,
                })
                .unwrap();
                let expected_slice = slices.slice_for(target.id).unwrap().clone();
                assert!(!expected_slice.unknowns().is_empty());
                assert!(expected_slice.requires_cache_miss());
                let syntax = syntax_from(source, symbols).unwrap();
                assert_eq!(syntax.len(), 1);
                let semantic = semantic(&fresh, &core).unwrap();
                let mut entries = symbols.symbols().iter().filter(|entry| {
                    entry.symbol() == &fresh.algorithm().1
                        && entry.kind() == mizar_resolve::env::SymbolKind::Algorithm
                });
                let entry = entries.next().unwrap();
                assert!(entries.next().is_none());
                assert_eq!(entry.visibility(), mizar_resolve::env::Visibility::Public);
                let module = entry.symbol().module();
                let owner = Json::object([
                    ("package_id", Json::string(module.package().as_str())),
                    ("module_path", Json::string(module.path().as_str())),
                    ("namespace", Json::string(entry.namespace().as_str())),
                    ("spelling", Json::string(entry.primary_spelling())),
                    ("kind", Json::string("algorithm")),
                    ("visibility", Json::string("public")),
                ])
                .unwrap();
                let owner = canonical_json_bytes(&owner);
                let syntax = &syntax[&fresh.algorithm().1];
                let candidate =
                    candidate_hash(&owner, syntax, &semantic, old.0, old.1, old.2).unwrap();
                let hex =
                    |bytes: &[u8]| bytes.iter().map(|b| format!("{b:02x}")).collect::<String>();
                assert_eq!(hex(candidate.as_bytes()), candidate_hex);
                assert_eq!(
                    candidate,
                    candidate_hash(&owner, syntax, &semantic, old.0, old.1, old.2,).unwrap()
                );
                if candidate_hex
                    == "e5ea8aded6a61a092f8614873d1f0efebef85ac9968dc9b2e13bd3363c1a3544"
                {
                    for (owner, syntax, semantic) in [
                        (&[][..], syntax.as_slice(), semantic.as_slice()),
                        (owner.as_slice(), &[][..], semantic.as_slice()),
                        (owner.as_slice(), syntax.as_slice(), &[][..]),
                    ] {
                        assert_eq!(
                            candidate_hash(owner, syntax, semantic, old.0, old.1, old.2,)
                                .unwrap_err(),
                            "unsupported source algorithm state candidate anchor"
                        );
                    }
                }
                let (identity, slice) =
                    identify(source, check, symbols, snapshot_id(0), target.id).unwrap();
                let anchor = format!("source-algorithm-state-candidate-anchor-v1:{candidate_hex}");
                assert_eq!(identity.obligation_id(), anchor);
                assert_eq!(identity.obligation_anchor().as_str(), anchor);
                assert_eq!(hex(identity.obligation_fingerprint().as_bytes()), final_hex);
                assert_eq!(identity.vc_fingerprint(), old.0.hash());
                assert_eq!(identity.local_context_fingerprint(), old.2.hash());
                assert_eq!(
                    identity.dependency_slice_fingerprint(),
                    expected_slice.fingerprint().hash()
                );
                assert_eq!(slice, expected_slice);
                assert_eq!(
                    (identity.clone(), slice.clone()),
                    identify(source, check, symbols, snapshot_id(0), target.id,).unwrap()
                );
                assert_eq!(format!("{source:?}{typed:?}{symbols:?}{check:?}"), before);
            },
        );
    }
}

#[test]
fn source_algorithm_state_obligation_identity_preserves_source_participants() {
    use mizar_artifact::store::{CanonicalJson as Json, canonical_json_bytes};
    use mizar_core::elaborator::{
        lower_source_algorithms as lower, source_algorithm_core_semantic_payload as semantic,
        source_algorithm_formula_payloads as guards,
    };
    use mizar_proof::status::source_algorithm_state_obligation_identity as identify;
    use mizar_resolve::{
        env::{SymbolKind, Visibility},
        symbols::source_algorithm_state_syntax_payloads_from_arena as syntax,
    };
    use mizar_vc::{
        dependency_slice::{DependencySliceInput, try_compute_dependency_slices as slices},
        generator::generate_source_algorithm_postconditions as generate,
    };
    let original = fs::read_to_string(config().workspace_root.join(
        "tests/miz/pass/algorithms/pass_proof_verification_algorithm_var_const_assert_001.miz",
    ))
    .unwrap();
    let flat = original.replacen("    assert x = a;\n", "", 1);
    let binder = flat
        .replace(" a ", " input ")
        .replace("(a)", "(input)")
        .replace(" a;", " input;")
        .replace("= a\n", "= input\n");
    let package = "mizar-test-corpus";
    let module = "tests.miz.pass.algorithms.pass_proof_verification_algorithm_var_const_assert_001";
    let generation = "mizar-vc-generation-step5c14-state-v1".into();
    let schema = VcSchemaVersion::new("mizar-vc-vcset-step5c14-state-v1");
    let mut baseline = None;
    for (label, text) in [
        ("base", flat.clone()),
        ("trivia", format!(":: 世代 neutral trivia\n\n{flat}")),
        ("relocated", flat.clone()),
        ("package", flat.clone()),
        ("module", flat.clone()),
        ("binder", binder),
        ("owner", flat.replace("swapalgo", "otheralgo")),
        ("assignment", flat.replace("    x := c;", "    x := a;")),
        ("return", flat.replace("return x;", "return c;")),
    ] {
        let package = if label == "package" {
            "other-package"
        } else {
            package
        };
        let module = if label == "module" {
            "nested.owners"
        } else {
            module
        };
        let skip = if label == "relocated" { 7 } else { 0 };
        with_source_algorithm_identity_inputs(
            &text,
            package,
            module,
            skip,
            |source, _, symbols, check| {
                let core = lower(check).unwrap();
                let vcs = generate(&core, snapshot_id(0), &generation, &schema).unwrap();
                let id = vcs.vcs()[0].id;
                let guard = core.algorithms().iter().next().unwrap().1.params[0]
                    .ty_guard
                    .unwrap();
                let formulas = guards(check, &core, &[guard]).unwrap();
                let triple = vcs
                    .source_algorithm_state_canonical_components(id, &core, &generation, &formulas)
                    .unwrap();
                let old_slices = slices(DependencySliceInput {
                    vc_set: &vcs,
                    discharge_output: None,
                })
                .unwrap();
                let old_slice = old_slices.slice_for(id).unwrap();
                let syntax = syntax(source, symbols).unwrap();
                let semantic = semantic(check, &core).unwrap();
                let mut entries = symbols.symbols().iter().filter(|entry| {
                    entry.symbol() == &check.algorithm().1 && entry.kind() == SymbolKind::Algorithm
                });
                let entry = entries.next().unwrap();
                assert!(entries.next().is_none());
                assert_eq!(entry.visibility(), Visibility::Public);
                let owner_module = entry.symbol().module();
                let owner = Json::object([
                    ("package_id", Json::string(owner_module.package().as_str())),
                    ("module_path", Json::string(owner_module.path().as_str())),
                    ("namespace", Json::string(entry.namespace().as_str())),
                    ("spelling", Json::string(entry.primary_spelling())),
                    ("kind", Json::string("algorithm")),
                    ("visibility", Json::string("public")),
                ])
                .unwrap();
                let operands = (
                    canonical_json_bytes(&owner),
                    syntax[&check.algorithm().1].clone(),
                    semantic,
                    triple.0.hash(),
                    triple.1,
                    triple.2.hash(),
                );
                let (identity, slice) =
                    identify(source, check, symbols, snapshot_id(0), id).unwrap();
                assert_eq!(slice, *old_slice);
                let _ = baseline.get_or_insert((
                    source.source_id(),
                    operands.clone(),
                    identity.obligation_anchor().as_str().to_owned(),
                ));
                let (source_id, baseline_operands, baseline_anchor) = baseline.as_ref().unwrap();
                if label != "base" {
                    let changed = [
                        operands.0 != baseline_operands.0,
                        operands.1 != baseline_operands.1,
                        operands.2 != baseline_operands.2,
                        operands.3 != baseline_operands.3,
                        operands.4 != baseline_operands.4,
                        operands.5 != baseline_operands.5,
                    ];
                    let expected = match label {
                        "trivia" | "relocated" => [false, false, false, false, false, false],
                        "package" | "module" => [true, false, true, true, false, true],
                        "binder" => [false, true, false, false, false, false],
                        "owner" => [true, true, true, true, false, true],
                        "assignment" => [false, true, true, true, false, true],
                        "return" => [false, true, true, true, true, false],
                        _ => panic!("unexpected participant"),
                    };
                    assert_eq!(changed, expected, "{label} old operands");
                    assert_eq!(
                        identity.obligation_anchor().as_str() == baseline_anchor,
                        label == "trivia" || label == "relocated"
                    );
                    if label == "relocated" {
                        assert_ne!(source.source_id(), *source_id);
                    }
                } else {
                    let other_vcs = generate(&core, snapshot_id(1), &generation, &schema).unwrap();
                    let other_slices = slices(DependencySliceInput {
                        vc_set: &other_vcs,
                        discharge_output: None,
                    })
                    .unwrap();
                    let other = identify(source, check, symbols, snapshot_id(1), id).unwrap();
                    assert_eq!(other.1, *other_slices.slice_for(id).unwrap());
                }
            },
        );
    }
}

#[test]
fn source_algorithm_state_obligation_identity_preserves_upstream_errors_and_inputs() {
    use mizar_checker::type_checker::check_source_algorithm_types;
    use mizar_core::elaborator::{
        lower_source_algorithms as lower, source_algorithm_formula_payloads as project,
    };
    use mizar_proof::status::source_algorithm_state_obligation_identity as identify;
    use mizar_vc::{
        generator::generate_source_algorithm_postconditions as generate,
        kernel_evidence_handoff::build_source_algorithm_state_kernel_handoff as handoff,
        vc_ir::VcId,
    };
    let root = config().workspace_root;
    let original = fs::read_to_string(root.join(
        "tests/miz/pass/algorithms/pass_proof_verification_algorithm_var_const_assert_001.miz",
    ))
    .unwrap();
    let flat = original.replacen("    assert x = a;\n", "", 1);
    let generation = "mizar-vc-generation-step5c14-state-v1".into();
    let schema = VcSchemaVersion::new("mizar-vc-vcset-step5c14-state-v1");
    for (kind, text) in [
        ("absent", flat.clone()),
        ("zero", flat.replace("    ensures result = a\n", "")),
        ("assertion", original.clone()),
    ] {
        with_source_algorithm_identity_inputs(
            &text,
            "mizar-test-corpus",
            "tests.miz.pass.algorithms.pass_proof_verification_algorithm_var_const_assert_001",
            0,
            |source, typed, symbols, check| {
                let before = format!("{source:?}{typed:?}{symbols:?}{check:?}");
                let fresh = check_source_algorithm_types(source, check.typed(), symbols).unwrap();
                let core = lower(&fresh).unwrap();
                let vcs = generate(&core, snapshot_id(0), &generation, &schema).unwrap();
                let guard = core.algorithms().iter().next().unwrap().1.params[0]
                    .ty_guard
                    .unwrap();
                let formulas = project(&fresh, &core, &[guard]).unwrap();
                if kind == "absent" {
                    handoff(&core, &vcs, vcs.vcs()[0].id, &generation).unwrap();
                }
                let selected = if kind == "assertion" {
                    vcs.vcs()[0].id
                } else {
                    VcId::new(99)
                };
                if kind == "zero" {
                    assert!(vcs.vcs().is_empty());
                }
                let expected = vcs
                    .source_algorithm_state_canonical_components(
                        selected,
                        &core,
                        &generation,
                        &formulas,
                    )
                    .unwrap_err();
                assert_eq!(
                    identify(source, check, symbols, snapshot_id(0), selected,).unwrap_err(),
                    expected
                );
                if kind == "absent" {
                    let foreign = with_source_algorithm_identity_inputs(
                        &flat.replace("swapalgo", "foreign_algorithm"),
                        "mizar-test-corpus",
                        "tests.miz.pass.algorithms.pass_proof_verification_algorithm_var_const_assert_001",
                        0,
                        |_, _, own_symbols, own_check| {
                            let own_core = lower(own_check).unwrap();
                            let own_vcs =
                                generate(&own_core, snapshot_id(0), &generation, &schema).unwrap();
                            handoff(&own_core, &own_vcs, own_vcs.vcs()[0].id, &generation).unwrap();
                            own_symbols.clone()
                        },
                    );
                    let (_, _, augmented) =
                        super::super::super::tests::source_registration_owner_inputs(
                            &format!("{flat}\n{flat}"),
                            "mizar-test-corpus",
                            "tests.miz.pass.algorithms.pass_proof_verification_algorithm_var_const_assert_001",
                            0,
                        );
                    for foreign in [&foreign, augmented.env()] {
                        let expected = check_source_algorithm_types(source, check.typed(), foreign)
                            .unwrap_err();
                        assert_eq!(
                            identify(source, check, foreign, snapshot_id(0), selected,)
                                .unwrap_err(),
                            expected
                        );
                    }
                }
                assert_eq!(format!("{source:?}{typed:?}{symbols:?}{check:?}"), before);
            },
        );
    }
    let return_source = fs::read_to_string(root.join(
        "tests/miz/pass/algorithms/pass_proof_verification_algorithm_ensures_return_001.miz",
    ))
    .unwrap();
    let two_returns = return_source
        .replace("    ensures result = a\n", "")
        .replace("return a;", "return a;\n    return a;");
    with_source_algorithm_identity_inputs(
        &two_returns,
        "mizar-test-corpus",
        "tests.miz.pass.algorithms.pass_proof_verification_algorithm_var_const_assert_001",
        0,
        |source, _, symbols, check| {
            let core = lower(check).unwrap();
            let expected = generate(&core, snapshot_id(0), &generation, &schema).unwrap_err();
            assert_eq!(
                identify(source, check, symbols, snapshot_id(0), VcId::new(99),).unwrap_err(),
                expected
            );
        },
    );
}
#[test]
fn source_algorithm_state_status_matches_genuine_projection() {
    use mizar_kernel::{checker::KernelCheckStatus, clause::VariableId};
    use mizar_proof::{
        policy::{CandidatePolicyClass, KernelEvidenceOrigin, KernelPolicyInput, VerifierPolicy},
        selection::{
            CandidateSourceId, ProofEvidenceCandidate, ProofEvidenceSet, ProofWinnerClass,
            VcProofSelection, merge_artifact_proof_selections, select_winner,
        },
        status::{
            ArtifactPublicationGap, ProofStatusProjectionInput,
            check_source_algorithm_state_transitivity as normal,
            project_source_algorithm_state_transitivity_status as status, project_status,
            source_algorithm_state_obligation_identity as identify,
        },
    };
    use mizar_vc::{
        dependency_slice::{
            DependencySliceCompleteness, DependencySliceInput, try_compute_dependency_slices,
        },
        generator::generate_source_algorithm_postconditions as generate,
        vc_ir::{GenerationSchemaVersion, VcSchemaVersion, VcStatus},
    };

    let original = fs::read_to_string(config().workspace_root.join(
        "tests/miz/pass/algorithms/pass_proof_verification_algorithm_var_const_assert_001.miz",
    ))
    .unwrap();
    assert_eq!(original.matches("    assert x = a;\n").count(), 1);
    let flat = original.replacen("    assert x = a;\n", "", 1);
    let renamed = flat
        .replace("let a be", "let input be")
        .replace("swapalgo(a)", "copyalgo(input)")
        .replace("var x :=", "var local :=")
        .replace("const c :=", "const saved :=")
        .replace("x := c;", "local := saved;")
        .replace("return x;", "return local;")
        .replace("result = a", "result = input")
        .replace(":= a;", ":= input;");
    let variants = [
        (flat.clone(), 0, 4),
        (renamed, 0, 1),
        (
            flat.replacen("    x := c;", "    const d := a;\n    x := c;", 1),
            0,
            1,
        ),
        (flat.replacen("terminating algorithm", "algorithm", 1), 0, 1),
        (format!(":: 世代 neutral trivia\n\n{flat}"), 0, 1),
        (flat, 7, 1),
    ];
    let generation = GenerationSchemaVersion::new("mizar-vc-generation-step5c14-state-v1");
    let schema = VcSchemaVersion::new("mizar-vc-vcset-step5c14-state-v1");
    for (variant, (text, skip, cases)) in variants.iter().enumerate() {
        with_source_algorithm_identity_inputs(
            text,
            "mizar-test-corpus",
            "tests.miz.pass.algorithms.pass_proof_verification_algorithm_var_const_assert_001",
            *skip,
            |source, typed, symbols, check| {
                let before = format!("{source:?}{typed:?}{symbols:?}{check:?}");
                if variant == 5 {
                    assert_eq!(format!("{:?}", source.source_id()), "SourceId(OpaqueId(8))");
                }
                let core = mizar_core::elaborator::lower_source_algorithms(check).unwrap();
                let policies = [
                    (0, VerifierPolicy::default()),
                    (1, VerifierPolicy::default()),
                    (0, VerifierPolicy::development()),
                    (0, VerifierPolicy::interactive()),
                ];
                for (ordinal, policy) in policies.into_iter().take(*cases) {
                    let snapshot = snapshot_id(ordinal);
                    let vcs = generate(&core, snapshot, &generation, &schema).unwrap();
                    let [vc] = vcs.vcs() else {
                        panic!("one genuine state VC")
                    };
                    assert_eq!(vc.status, VcStatus::Open);
                    assert!(vcs.seed_accounting().iter().any(|seed| seed.seed_status
                        == mizar_core::core_ir::ObligationSeedStatus::Deferred));
                    let slices = try_compute_dependency_slices(DependencySliceInput {
                        vc_set: &vcs,
                        discharge_output: None,
                    })
                    .unwrap();
                    let whole = slices.slice_for(vc.id).unwrap().clone();
                    assert_eq!(
                        whole.completeness(),
                        DependencySliceCompleteness::IncompleteUncacheable
                    );
                    assert!(whole.requires_cache_miss());
                    let (identity, old_slice) =
                        identify(source, check, symbols, snapshot, vc.id).unwrap();
                    assert_eq!(old_slice, whole);
                    let normal_check = || {
                        normal(
                            source,
                            check.typed(),
                            symbols,
                            snapshot,
                            vc.id,
                            VariableId(2),
                            &policy,
                        )
                    };
                    let (result, decision) = normal_check().unwrap();
                    assert_eq!(result.status(), KernelCheckStatus::Accepted);
                    assert!(!result.policy_taint());
                    assert_eq!(decision.class, CandidatePolicyClass::DischargedBuiltin);
                    let target = result.target_vc_fingerprint().sort_bytes();
                    let id_text = format!(
                        "source-algorithm-state-transitivity-v1:{}",
                        target
                            .iter()
                            .map(|byte| format!("{byte:02x}"))
                            .collect::<String>()
                    );
                    if variant == 0 && ordinal == 0 {
                        assert_eq!(
                            id_text,
                            "source-algorithm-state-transitivity-v1:1200000020c5b0541068d905693a7b23e1c38b97b88593f8c033353d51d92e377d70142624"
                        );
                    }
                    assert_ne!(id_text, identity.obligation_id());
                    let id = CandidateSourceId::new(id_text.clone()).unwrap();
                    let candidate = ProofEvidenceCandidate::from_trusted_kernel_input(
                        id,
                        &KernelPolicyInput::from_kernel_result(
                            &result,
                            KernelEvidenceOrigin::BuiltinDischarge,
                        ),
                    )
                    .expect("actual accepted untainted Kernel input");
                    let selected = select_winner(
                        &ProofEvidenceSet::new(target, None, policy.clone())
                            .with_candidates([candidate]),
                    );
                    let [row] = <[_; 1]>::try_from(
                        merge_artifact_proof_selections(
                            [],
                            [VcProofSelection::new(vc.id, selected)],
                        )
                        .unwrap(),
                    )
                    .unwrap();
                    let expected = project_status(ProofStatusProjectionInput::new(
                        row,
                        policy.clone(),
                        identity.clone(),
                    ))
                    .unwrap();
                    let facade = |policy: &VerifierPolicy| {
                        status(
                            source,
                            check,
                            symbols,
                            snapshot,
                            vc.id,
                            VariableId(2),
                            policy,
                        )
                    };
                    let actual = facade(&policy).unwrap();
                    assert_eq!(actual, (expected, whole.clone()));
                    assert_eq!(actual, facade(&policy).unwrap());
                    assert_eq!(
                        actual.0.selected_class(),
                        ProofWinnerClass::DischargedBuiltin
                    );
                    assert_eq!(
                        actual.0.artifact_publication().gap(),
                        Some(ArtifactPublicationGap::DischargedBuiltinWitnessUnsupported)
                    );
                    assert!(actual.0.accepted_witness_obligation_id().is_none());
                    assert!(actual.0.trusted_used_axioms().is_none());
                    assert!(actual.0.explanation_ref().is_none());
                    let reuse = actual.0.reuse_metadata();
                    assert_eq!(reuse.selected_candidate_id().unwrap().as_str(), id_text);
                    assert_eq!(reuse.obligation_anchor(), identity.obligation_anchor());
                    assert_eq!(
                        reuse.obligation_fingerprint(),
                        identity.obligation_fingerprint()
                    );
                    assert_eq!(reuse.vc_fingerprint(), identity.vc_fingerprint());
                    assert_eq!(
                        reuse.local_context_fingerprint(),
                        identity.local_context_fingerprint()
                    );
                    assert_eq!(
                        reuse.dependency_slice_fingerprint(),
                        identity.dependency_slice_fingerprint()
                    );
                    assert!(reuse.trusted_used_axioms_hash().is_none());
                    assert!(reuse.selected_proof_witness_hash().is_none());
                    assert!(reuse.dependency_compatibility().is_none());
                    assert!(!reuse.cache_reuse_predicate_complete());
                    assert_eq!(
                        (identity, old_slice),
                        identify(source, check, symbols, snapshot, vc.id).unwrap()
                    );
                    assert_eq!((result, decision), normal_check().unwrap());
                }
                assert_eq!(format!("{source:?}{typed:?}{symbols:?}{check:?}"), before);
            },
        );
    }
}

#[test]
fn source_algorithm_state_status_preserves_upstream_errors_and_inputs() {
    use mizar_checker::type_checker::check_source_algorithm_types;
    use mizar_kernel::clause::VariableId;
    use mizar_proof::{
        policy::VerifierPolicy,
        status::{
            check_source_algorithm_state_transitivity as normal,
            project_source_algorithm_state_transitivity_status as status,
            source_algorithm_state_obligation_identity as identify,
        },
    };
    use mizar_vc::{
        generator::generate_source_algorithm_postconditions as generate,
        vc_ir::{GenerationSchemaVersion, VcId, VcSchemaVersion},
    };
    let root = config().workspace_root;
    let original = fs::read_to_string(root.join(
        "tests/miz/pass/algorithms/pass_proof_verification_algorithm_var_const_assert_001.miz",
    ))
    .unwrap();
    assert_eq!(original.matches("    assert x = a;\n").count(), 1);
    let flat = original.replacen("    assert x = a;\n", "", 1);
    let disabled = VerifierPolicy::default().with_kernel_evidence_formats([]);
    let generation = GenerationSchemaVersion::new("mizar-vc-generation-step5c14-state-v1");
    let schema = VcSchemaVersion::new("mizar-vc-vcset-step5c14-state-v1");
    for (kind, text) in [
        ("absent", flat.clone()),
        ("zero", flat.replace("    ensures result = a\n", "")),
        ("assertion", original),
    ] {
        with_source_algorithm_identity_inputs(
            &text,
            "mizar-test-corpus",
            "tests.miz.pass.algorithms.pass_proof_verification_algorithm_var_const_assert_001",
            0,
            |source, typed, symbols, check| {
                let before = format!("{source:?}{typed:?}{symbols:?}{check:?}");
                let core = mizar_core::elaborator::lower_source_algorithms(check).unwrap();
                let vcs = generate(&core, snapshot_id(0), &generation, &schema).unwrap();
                let selected = if kind == "assertion" {
                    vcs.vcs()[0].id
                } else {
                    VcId::new(99)
                };
                if kind == "zero" {
                    assert!(vcs.vcs().is_empty());
                }
                let expected =
                    identify(source, check, symbols, snapshot_id(0), selected).unwrap_err();
                let facade = |middle| {
                    status(
                        source,
                        check,
                        symbols,
                        snapshot_id(0),
                        selected,
                        middle,
                        &disabled,
                    )
                };
                for middle in [VariableId(2), VariableId(99)] {
                    assert_eq!(facade(middle).unwrap_err(), expected);
                }
                if kind == "absent" {
                    let foreign = with_source_algorithm_identity_inputs(
                        &flat.replace("swapalgo", "foreign_algorithm"),
                        "mizar-test-corpus",
                        "tests.miz.pass.algorithms.pass_proof_verification_algorithm_var_const_assert_001",
                        0,
                        |own_source, _, own_symbols, own_check| {
                            let own_core =
                                mizar_core::elaborator::lower_source_algorithms(own_check).unwrap();
                            let own_vcs =
                                generate(&own_core, snapshot_id(0), &generation, &schema).unwrap();
                            let [own_vc] = own_vcs.vcs() else {
                                panic!("one renamed source VC")
                            };
                            identify(
                                own_source,
                                own_check,
                                own_symbols,
                                snapshot_id(0),
                                own_vc.id,
                            )
                            .unwrap();
                            normal(
                                own_source,
                                own_check.typed(),
                                own_symbols,
                                snapshot_id(0),
                                own_vc.id,
                                VariableId(2),
                                &VerifierPolicy::default(),
                            )
                            .unwrap();
                            own_symbols.clone()
                        },
                    );
                    let (_, _, augmented) =
                        super::super::super::tests::source_registration_owner_inputs(
                            &format!("{flat}\n{flat}"),
                            "mizar-test-corpus",
                            "tests.miz.pass.algorithms.pass_proof_verification_algorithm_var_const_assert_001",
                            0,
                        );
                    for other in [&foreign, augmented.env()] {
                        let expected =
                            check_source_algorithm_types(source, check.typed(), other).unwrap_err();
                        assert_eq!(
                            status(
                                source,
                                check,
                                other,
                                snapshot_id(0),
                                selected,
                                VariableId(99),
                                &disabled
                            )
                            .unwrap_err(),
                            expected
                        );
                    }
                }
                assert_eq!(format!("{source:?}{typed:?}{symbols:?}{check:?}"), before);
            },
        );
    }
    let return_source = fs::read_to_string(root.join(
        "tests/miz/pass/algorithms/pass_proof_verification_algorithm_ensures_return_001.miz",
    ))
    .unwrap();
    let two_returns = return_source
        .replace("    ensures result = a\n", "")
        .replace("return a;", "return a;\n    return a;");
    with_source_algorithm_identity_inputs(
        &two_returns,
        "mizar-test-corpus",
        "tests.miz.pass.algorithms.pass_proof_verification_algorithm_var_const_assert_001",
        0,
        |source, typed, symbols, check| {
            let before = format!("{source:?}{typed:?}{symbols:?}{check:?}");
            let core = mizar_core::elaborator::lower_source_algorithms(check).unwrap();
            let expected = generate(&core, snapshot_id(0), &generation, &schema).unwrap_err();
            assert_eq!(
                identify(source, check, symbols, snapshot_id(0), VcId::new(99)).unwrap_err(),
                expected
            );
            assert_eq!(
                status(
                    source,
                    check,
                    symbols,
                    snapshot_id(0),
                    VcId::new(99),
                    VariableId(99),
                    &disabled
                )
                .unwrap_err(),
                expected
            );
            assert_eq!(format!("{source:?}{typed:?}{symbols:?}{check:?}"), before);
        },
    );
    with_source_algorithm_identity_inputs(
        &flat,
        "mizar-test-corpus",
        "tests.miz.pass.algorithms.pass_proof_verification_algorithm_var_const_assert_001",
        0,
        |source, typed, symbols, check| {
            let before = format!("{source:?}{typed:?}{symbols:?}{check:?}");
            let core = mizar_core::elaborator::lower_source_algorithms(check).unwrap();
            let vcs = generate(&core, snapshot_id(0), &generation, &schema).unwrap();
            let [vc] = vcs.vcs() else {
                panic!("one genuine state VC")
            };
            let prior = identify(source, check, symbols, snapshot_id(0), vc.id).unwrap();
            let facade = |middle, policy: &VerifierPolicy| {
                status(
                    source,
                    check,
                    symbols,
                    snapshot_id(0),
                    vc.id,
                    middle,
                    policy,
                )
            };
            let normal_check = |middle, policy: &VerifierPolicy| {
                normal(
                    source,
                    check.typed(),
                    symbols,
                    snapshot_id(0),
                    vc.id,
                    middle,
                    policy,
                )
            };
            let prior_result = normal_check(VariableId(2), &VerifierPolicy::default()).unwrap();
            let success = facade(VariableId(2), &VerifierPolicy::default()).unwrap();
            let expected = normal_check(VariableId(99), &disabled).unwrap_err();
            assert_eq!(facade(VariableId(99), &disabled).unwrap_err(), expected);
            let expected = normal_check(VariableId(2), &disabled).unwrap_err();
            assert_eq!(expected, "algorithm.proof.kernel_policy_rejected");
            assert_eq!(facade(VariableId(2), &disabled).unwrap_err(), expected);
            assert_eq!(
                normal_check(VariableId(2), &VerifierPolicy::default()).unwrap(),
                prior_result
            );
            assert_eq!(
                identify(source, check, symbols, snapshot_id(0), vc.id).unwrap(),
                prior
            );
            assert_eq!(
                facade(VariableId(2), &VerifierPolicy::default()).unwrap(),
                success
            );
            assert_eq!(format!("{source:?}{typed:?}{symbols:?}{check:?}"), before);
        },
    );
}

#[test]
fn source_algorithm_state_witness_draft_matches_genuine_checked_wire_and_fields() {
    use mizar_artifact::{
        proof_witness::KernelAcceptanceMetadata,
        registration_summary::{ArtifactHashClass, ArtifactHashRef},
        store::{CanonicalJson, SchemaVersion, artifact_hash_domain, canonical_json_bytes},
    };
    use mizar_checker::type_checker::check_source_algorithm_types;
    use mizar_kernel::{
        certificate_parser::{ClauseTautologyPolicy, Fingerprint, KernelProfileRecord},
        checker::{
            FormulaEvidenceContext, ImportedFactContextLimits, KernelCheckPolicy,
            KernelCheckStatus, KernelContextIdentityEntry, KernelContextIdentityPayload,
            KernelContextIdentitySource, KernelEvidenceCheckInput, KernelEvidenceCheckKind,
            KernelEvidenceCheckLimits, KernelFormulaProducerRef, KernelVcGeneratedFormulaId,
            check_kernel_evidence,
        },
        clause::VariableId,
        formula_evidence::{FormulaEvidenceParseContext, parse_formula_evidence},
        rejection::TargetVcFingerprint,
    };
    use mizar_proof::{
        policy::{
            AcceptedGoalPolarity, CandidatePolicyClass, KernelEvidenceOrigin, KernelPolicyInput,
            PolicyCandidate, ProofPolicyEvaluator, VerifierPolicy,
        },
        selection::{
            CandidateSourceId, ProofEvidenceCandidate, ProofEvidenceSet, VcProofSelection,
            merge_artifact_proof_selections, select_winner,
        },
        status::{
            ArtifactPublicationGap, ProofStatusProjectionInput,
            check_source_algorithm_state_transitivity, project_status,
            source_algorithm_state_obligation_identity,
            source_algorithm_state_transitivity_witness_draft,
        },
        witness_store::{
            ProofWitnessPayloadSchema, ProofWitnessProvenance, ProofWitnessStoreError, stage,
        },
    };
    use mizar_vc::{
        generator::generate_source_algorithm_postconditions,
        kernel_evidence_handoff::{
            KernelClauseTautologyPolicy, KernelContextIdentitySource as SourceContext,
            build_source_algorithm_state_kernel_handoff,
        },
        vc_ir::{GenerationSchemaVersion, VcFormulaRef, VcSchemaVersion, VcStatus},
    };
    use std::collections::BTreeMap;

    let original = fs::read_to_string(config().workspace_root.join(
        "tests/miz/pass/algorithms/pass_proof_verification_algorithm_var_const_assert_001.miz",
    ))
    .unwrap();
    let flat = original.replacen("    assert x = a;\n", "", 1);
    let renamed = flat
        .replace("let a be", "let input be")
        .replace("swapalgo(a)", "copyalgo(input)")
        .replace("var x :=", "var local :=")
        .replace("const c :=", "const saved :=")
        .replace("x := c;", "local := saved;")
        .replace("return x;", "return local;")
        .replace("result = a", "result = input")
        .replace(":= a;", ":= input;");
    let variants = [
        (flat.clone(), VerifierPolicy::default()),
        (flat.clone(), VerifierPolicy::development()),
        (renamed, VerifierPolicy::default()),
        (
            flat.replacen("    x := c;", "    const d := a;\n    x := c;", 1),
            VerifierPolicy::default(),
        ),
        (
            flat.replacen("terminating algorithm", "algorithm", 1),
            VerifierPolicy::default(),
        ),
        (
            format!(":: 世代 neutral trivia\n\n{flat}"),
            VerifierPolicy::default(),
        ),
    ];
    for (case, (text, policy)) in variants.into_iter().enumerate() {
        with_source_algorithm_state_draft_inputs(
            &text,
            &policy,
            |source, check, symbols, snapshot| {
                let inputs_before = format!("{source:?}{check:?}{symbols:?}{snapshot:?}{policy:?}");
                let generation =
                    GenerationSchemaVersion::new("mizar-vc-generation-step5c14-state-v1");
                let core = mizar_core::elaborator::lower_source_algorithms(check).unwrap();
                let vcs = generate_source_algorithm_postconditions(
                    &core,
                    snapshot.id,
                    &generation,
                    &VcSchemaVersion::new("mizar-vc-vcset-step5c14-state-v1"),
                )
                .unwrap();
                let [vc] = vcs.vcs() else {
                    panic!("one genuine state VC")
                };
                assert_eq!(vc.status, VcStatus::Open);
                let identify = || {
                    source_algorithm_state_obligation_identity(
                        source,
                        check,
                        symbols,
                        snapshot.id,
                        vc.id,
                    )
                };
                let (identity, slice) = identify().unwrap();
                let handoff =
                    build_source_algorithm_state_kernel_handoff(&core, &vcs, vc.id, &generation)
                        .unwrap();
                let envelope = handoff.canonical_evidence();
                let target = envelope.target_vc();
                let profile = envelope.kernel_profile();
                let tautology = match profile.clause_tautology_policy {
                    KernelClauseTautologyPolicy::Reject => ClauseTautologyPolicy::Reject,
                    KernelClauseTautologyPolicy::Marker => ClauseTautologyPolicy::Marker,
                    _ => panic!("unexpected profile"),
                };
                let mut context = FormulaEvidenceParseContext::v1(
                    Fingerprint::new(target.algorithm_id, target.digest.clone()),
                    KernelProfileRecord::v1(profile.profile_id, tautology),
                );
                let parsed_v1 =
                    parse_formula_evidence(&handoff.encode_local_evidence(None).unwrap(), &context)
                        .unwrap();
                let names = mizar_atp::backend::encode_local_variable_transitivity_names(
                    &parsed_v1,
                    VariableId(2),
                )
                .unwrap();
                let wire = handoff.encode_local_evidence(Some(&names)).unwrap();
                context.accepted_schema_versions.insert(2);
                let parsed = parse_formula_evidence(&wire, &context).unwrap();
                let target = TargetVcFingerprint::new(target.algorithm_id, target.digest.clone());
                let entries = handoff
                    .context_identity()
                    .entries()
                    .iter()
                    .map(|entry| {
                        let SourceContext::LocalHypothesis { local_context_id } = entry.source()
                        else {
                            panic!("actual state context is local")
                        };
                        let producer = match entry.producer_formula_ref() {
                            VcFormulaRef::Core(id) => KernelFormulaProducerRef::Core(id),
                            VcFormulaRef::Generated(id) => KernelFormulaProducerRef::Generated(
                                KernelVcGeneratedFormulaId::new(id.index()),
                            ),
                            _ => panic!("actual state producer is supported"),
                        };
                        KernelContextIdentityEntry::new(
                            KernelContextIdentitySource::LocalHypothesis { local_context_id },
                            entry.formula_id(),
                            Fingerprint::new(
                                entry.formula_fingerprint().algorithm_id,
                                entry.formula_fingerprint().digest.clone(),
                            ),
                            producer,
                        )
                    })
                    .collect();
                let requirements = handoff.formula_context_requirements();
                assert!(
                    requirements.is_none_or(
                        |c| c.imported_axioms.is_empty() && c.imported_theorems.is_empty()
                    )
                );
                let kernel_context = FormulaEvidenceContext::with_context_identity(
                    Some(handoff.context_identity_hash().as_bytes().to_vec()),
                    vec![],
                    vec![],
                    Some(KernelContextIdentityPayload::new(
                        target.clone(),
                        handoff.canonical_hash(),
                        handoff.context_identity_hash(),
                        entries,
                    )),
                    ImportedFactContextLimits::default(),
                )
                .unwrap();
                let public_result = check_kernel_evidence(KernelEvidenceCheckInput {
                    target_vc_fingerprint: &target,
                    evidence: &parsed,
                    formula_context: Some(&kernel_context),
                    check_kind: KernelEvidenceCheckKind::ProofObligation,
                    policy: KernelCheckPolicy::default(),
                    limits: KernelEvidenceCheckLimits::default(),
                });
                let public_input = KernelPolicyInput::from_kernel_result(
                    &public_result,
                    KernelEvidenceOrigin::BuiltinDischarge,
                );
                let public_decision = ProofPolicyEvaluator::new(policy.clone())
                    .evaluate_candidate(&PolicyCandidate::KernelResult(public_input));
                let normal = || {
                    check_source_algorithm_state_transitivity(
                        source,
                        check.typed(),
                        symbols,
                        snapshot.id,
                        vc.id,
                        VariableId(2),
                        &policy,
                    )
                };
                let (result, decision) = normal().unwrap();
                assert_eq!(
                    (result.clone(), decision.clone()),
                    (public_result, public_decision)
                );
                assert_eq!(result.status(), KernelCheckStatus::Accepted);
                assert!(!result.policy_taint());
                assert_eq!(
                    result.evidence_check_kind(),
                    Some(KernelEvidenceCheckKind::ProofObligation)
                );
                assert_eq!(decision.class, CandidatePolicyClass::DischargedBuiltin);
                let input = KernelPolicyInput::from_kernel_result(
                    &result,
                    KernelEvidenceOrigin::BuiltinDischarge,
                );
                assert_eq!(
                    input.accepted_goal_polarity(),
                    Some(AcceptedGoalPolarity::AssertFalseForRefutation)
                );
                let accepted = input.accepted_evidence_hash().unwrap();
                let components = handoff.local_evidence_component_hashes().unwrap();
                let reference = |family: &str, version, digest| {
                    ArtifactHashRef::new(ArtifactHashClass::Interface, family, version, digest)
                };
                let component = |kind: &str, digest| {
                    reference(
                        &format!("mizar-vc/local-evidence-component/{kind}"),
                        SchemaVersion::new(1, 0),
                        digest,
                    )
                };
                let metadata = KernelAcceptanceMetadata {
                    kernel_profile_fingerprint: component("profile", components[0]),
                    verifier_policy_fingerprint: reference(
                        "mizar-proof/verifier-policy",
                        SchemaVersion::new(policy.schema_version(), 0),
                        policy.policy_fingerprint().hash(),
                    ),
                    checker_schema_version: SchemaVersion::new(policy.checker_schema_version(), 0),
                    evidence_schema_version: SchemaVersion::new(parsed.schema_version(), 0),
                    target_binding_hash: reference(
                        "mizar-vc/kernel-evidence-handoff-canonical",
                        SchemaVersion::new(1, 0),
                        handoff.canonical_hash(),
                    ),
                    formula_evidence_hash: component("formula", components[1]),
                    substitution_evidence_hash: component("substitutions", components[2]),
                    provenance_hash: component("provenance", components[3]),
                    formula_context_hash: None,
                    accepted_result_hash: reference(
                        "mizar-proof/kernel-accepted-evidence",
                        SchemaVersion::new(1, 0),
                        accepted,
                    ),
                };
                let target = result.target_vc_fingerprint().sort_bytes();
                let candidate_text = format!(
                    "source-algorithm-state-transitivity-v1:{}",
                    target
                        .iter()
                        .map(|byte| format!("{byte:02x}"))
                        .collect::<String>()
                );
                let candidate_id = CandidateSourceId::new(candidate_text).unwrap();
                let candidate =
                    ProofEvidenceCandidate::from_trusted_kernel_input(candidate_id.clone(), &input)
                        .unwrap();
                let selection = select_winner(
                    &ProofEvidenceSet::new(target, None, policy.clone())
                        .with_candidates([candidate]),
                );
                let [row] = <[_; 1]>::try_from(
                    merge_artifact_proof_selections([], [VcProofSelection::new(vc.id, selection)])
                        .unwrap(),
                )
                .ok()
                .unwrap();
                let expected = project_status(ProofStatusProjectionInput::new(
                    row,
                    policy.clone(),
                    identity.clone(),
                ))
                .unwrap();
                let original_ref = reference(
                    "mizar-proof/source-algorithm-state-original-obligation",
                    SchemaVersion::new(1, 0),
                    identity.obligation_fingerprint(),
                );
                let hex = |bytes: &[u8]| {
                    bytes
                        .iter()
                        .map(|byte| format!("{byte:02x}"))
                        .collect::<String>()
                };
                let schema = ProofWitnessPayloadSchema::with_canonical_bytes_required(
                    "mizar-proof/source-algorithm-state-transitivity-checked-wire",
                    metadata.evidence_schema_version,
                    true,
                )
                .unwrap();
                let payload_value = CanonicalJson::Object(BTreeMap::from([
                    (
                        "hash_domain".into(),
                        CanonicalJson::String("mizar-proof/witness-payload/v1".into()),
                    ),
                    (
                        "obligation_fingerprint".into(),
                        CanonicalJson::String(original_ref.to_artifact_hash_string()),
                    ),
                    (
                        "payload_bytes_hex".into(),
                        CanonicalJson::String(hex(&wire)),
                    ),
                    (
                        "payload_schema_family".into(),
                        CanonicalJson::String(schema.family().into()),
                    ),
                    (
                        "payload_schema_version".into(),
                        CanonicalJson::String(schema.version().to_string()),
                    ),
                    (
                        "selected_evidence_hash".into(),
                        CanonicalJson::String(hex(accepted.as_bytes())),
                    ),
                    (
                        "verifier_policy_fingerprint".into(),
                        CanonicalJson::String(hex(policy.policy_fingerprint().hash().as_bytes())),
                    ),
                ]));
                assert_eq!(canonical_json_bytes(&payload_value).last(), Some(&b'\n'));
                let payload_hash = artifact_hash_domain(schema.family(), schema.version())
                    .hash(&payload_value, &[]);
                let path = "proof-witnesses/state-transitivity.json";
                let acquire = || {
                    source_algorithm_state_transitivity_witness_draft(
                        source,
                        check,
                        symbols,
                        snapshot,
                        VariableId(2),
                        &policy,
                        path,
                    )
                };
                let product = acquire().unwrap();
                assert_eq!((product.0.clone(), product.1.clone()), (expected, slice));
                assert_eq!(product.1.status(), &VcStatus::Open);
                assert_eq!(
                    product.1.completeness(),
                    mizar_vc::dependency_slice::DependencySliceCompleteness::IncompleteUncacheable
                );
                assert!(product.1.requires_cache_miss());
                assert_eq!(
                    product.0.artifact_publication().gap(),
                    Some(ArtifactPublicationGap::DischargedBuiltinWitnessUnsupported)
                );
                assert!(!product.0.reuse_metadata().cache_reuse_predicate_complete());
                assert!(
                    product
                        .0
                        .reuse_metadata()
                        .selected_proof_witness_hash()
                        .is_none()
                );
                assert_eq!(product, acquire().unwrap());
                let draft = &product.2;
                assert_eq!(draft.obligation_id(), identity.obligation_id());
                assert_eq!(draft.obligation_anchor(), identity.obligation_anchor());
                assert_eq!(draft.obligation_fingerprint(), &original_ref);
                assert_eq!(
                    draft.selected_class(),
                    mizar_proof::selection::ProofWinnerClass::DischargedBuiltin
                );
                assert_eq!(draft.selected_evidence_hash(), accepted);
                assert_eq!(draft.kernel_acceptance(), &metadata);
                assert_eq!(draft.payload_bytes(), wire);
                assert_eq!(draft.witness_path(), path);
                assert_eq!(draft.payload_schema(), &schema);
                assert_eq!(draft.payload_hash_value(), payload_value);
                let staged = stage(draft.clone()).unwrap();
                assert_eq!(staged.staged_payload_hash().digest, payload_hash);
                assert!(staged.artifact_reference_candidate().is_none());
                let provenance = ProofWitnessProvenance::new(
                    snapshot.id.fingerprint(),
                    "mizar-proof/source-algorithm-state-transitivity-witness-draft/v1",
                    candidate_id,
                    KernelEvidenceOrigin::BuiltinDischarge,
                    identity.vc_fingerprint(),
                    identity.dependency_slice_fingerprint(),
                    snapshot.dependency_artifact_fingerprint(),
                    policy.policy_fingerprint().hash(),
                    metadata.checker_schema_version,
                    metadata.evidence_schema_version,
                )
                .unwrap()
                .with_accepted_result_hash(accepted);
                assert_eq!(draft.provenance(), &provenance);
                if case == 0 {
                    let disabled = policy.clone().with_kernel_evidence_formats([]);
                    let control = |middle, policy: &VerifierPolicy, path| {
                        source_algorithm_state_transitivity_witness_draft(
                            source, check, symbols, snapshot, middle, policy, path,
                        )
                    };
                    let bad_middle = check_source_algorithm_state_transitivity(
                        source,
                        check.typed(),
                        symbols,
                        snapshot.id,
                        vc.id,
                        VariableId(99),
                        &policy,
                    )
                    .unwrap_err();
                    assert_eq!(
                        control(VariableId(99), &policy, path).unwrap_err(),
                        bad_middle
                    );
                    assert_eq!(
                        control(VariableId(99), &disabled, "../invalid").unwrap_err(),
                        bad_middle
                    );
                    let rejected = check_source_algorithm_state_transitivity(
                        source,
                        check.typed(),
                        symbols,
                        snapshot.id,
                        vc.id,
                        VariableId(2),
                        &disabled,
                    )
                    .unwrap_err();
                    assert_eq!(
                        control(VariableId(2), &disabled, path).unwrap_err(),
                        rejected
                    );
                    assert_eq!(
                        control(VariableId(2), &disabled, "../invalid").unwrap_err(),
                        rejected
                    );
                    let invalid_path = ProofWitnessStoreError::InvalidWitnessPath {
                        reason: "path must start with proof-witnesses/".to_owned(),
                    }
                    .to_string();
                    assert_eq!(
                        control(VariableId(2), &policy, "../invalid").unwrap_err(),
                        invalid_path
                    );
                    let foreign = with_source_algorithm_state_draft_inputs(
                        &flat.replace("swapalgo", "foreign_algorithm"),
                        &policy,
                        |_, _, other_symbols, _| other_symbols.clone(),
                    );
                    let (_, _, augmented) =
                        super::super::super::tests::source_registration_owner_inputs(
                            &format!("{flat}\n{flat}"),
                            "mizar-test-corpus",
                            "tests.miz.pass.algorithms.pass_proof_verification_algorithm_var_const_assert_001",
                            0,
                        );
                    for other in [&foreign, augmented.env()] {
                        let expected =
                            check_source_algorithm_types(source, check.typed(), other).unwrap_err();
                        assert_eq!(
                            source_algorithm_state_transitivity_witness_draft(
                                source,
                                check,
                                other,
                                snapshot,
                                VariableId(99),
                                &disabled,
                                "../invalid",
                            )
                            .unwrap_err(),
                            expected
                        );
                    }
                }
                assert_eq!(normal().unwrap(), (result, decision));
                assert_eq!(identify().unwrap(), (identity, product.1.clone()));
                assert_eq!(
                    format!("{source:?}{check:?}{symbols:?}{snapshot:?}{policy:?}"),
                    inputs_before
                );
            },
        );
    }
}

#[test]
fn source_algorithm_state_witness_draft_stages_generic_hash_without_reference() {
    use mizar_kernel::clause::VariableId;
    use mizar_proof::{
        policy::VerifierPolicy, status::source_algorithm_state_transitivity_witness_draft as draft,
        witness_store::stage,
    };
    let original = fs::read_to_string(config().workspace_root.join(
        "tests/miz/pass/algorithms/pass_proof_verification_algorithm_var_const_assert_001.miz",
    ))
    .unwrap();
    let flat = original.replacen("    assert x = a;\n", "", 1);
    let policy = VerifierPolicy::default();
    with_source_algorithm_state_draft_inputs(&flat, &policy, |source, check, symbols, snapshot| {
        let product = draft(
            source,
            check,
            symbols,
            snapshot,
            VariableId(2),
            &policy,
            "proof-witnesses/state-transitivity.json",
        )
        .unwrap();
        let staged = stage(product.2).unwrap();
        assert!(staged.artifact_reference_candidate().is_none());
    });
}

#[test]
fn source_algorithm_state_witness_draft_preserves_upstream_errors_and_inputs() {
    use mizar_kernel::clause::VariableId;
    use mizar_proof::{
        policy::VerifierPolicy, status::source_algorithm_state_transitivity_witness_draft as draft,
    };
    use mizar_vc::{
        generator::generate_source_algorithm_postconditions as generate,
        vc_ir::{GenerationSchemaVersion, VcSchemaVersion},
    };
    let original = fs::read_to_string(config().workspace_root.join(
        "tests/miz/pass/algorithms/pass_proof_verification_algorithm_var_const_assert_001.miz",
    ))
    .unwrap();
    let flat = original.replacen("    assert x = a;\n", "", 1);
    let policy = VerifierPolicy::default();
    let disabled = policy.clone().with_kernel_evidence_formats([]);
    let generation = GenerationSchemaVersion::new("mizar-vc-generation-step5c14-state-v1");
    let schema = VcSchemaVersion::new("mizar-vc-vcset-step5c14-state-v1");
    let assertion_only = original.replace("    ensures result = a\n", "");
    let return_source = fs::read_to_string(config().workspace_root.join(
        "tests/miz/pass/algorithms/pass_proof_verification_algorithm_ensures_return_001.miz",
    ))
    .unwrap();
    let two_returns = return_source
        .replace("    ensures result = a\n", "")
        .replace("return a;", "return a;\n    return a;");
    for (text, count) in [
        (flat.replace("    ensures result = a\n", ""), Some(0)),
        (original, Some(2)),
        (assertion_only, Some(1)),
        (two_returns, None),
    ] {
        with_source_algorithm_state_draft_inputs(
            &text,
            &policy,
            |source, check, symbols, snapshot| {
                let before = format!("{source:?}{check:?}{symbols:?}{snapshot:?}{policy:?}");
                let core = mizar_core::elaborator::lower_source_algorithms(check).unwrap();
                let vcs = generate(&core, snapshot.id, &generation, &schema);
                let expected = match count {
                    Some(count) => {
                        let vcs = vcs.unwrap();
                        assert_eq!(vcs.vcs().len(), count);
                        match vcs.vcs() {
                            [vc] => {
                                mizar_proof::status::source_algorithm_state_obligation_identity(
                                    source,
                                    check,
                                    symbols,
                                    snapshot.id,
                                    vc.id,
                                )
                                .unwrap_err()
                            }
                            _ => "algorithm.proof.invalid_source_state_witness_draft".to_owned(),
                        }
                    }
                    None => vcs.unwrap_err(),
                };
                assert_eq!(
                    draft(
                        source,
                        check,
                        symbols,
                        snapshot,
                        VariableId(99),
                        &disabled,
                        "../invalid"
                    )
                    .unwrap_err(),
                    expected
                );
                assert_eq!(
                    format!("{source:?}{check:?}{symbols:?}{snapshot:?}{policy:?}"),
                    before
                );
            },
        );
    }
}

#[test]
fn source_algorithm_state_transitivity_search_matches_fresh_explicit_check() {
    use super::super::super::{
        formula_statement::step5c8_test_frontend as frontend, source_registration_inputs as inputs,
    };
    use mizar_atp::backend::{
        encode_local_variable_transitivity_names, select_local_variable_transitivity_names,
    };
    use mizar_checker::type_checker::check_source_algorithm_types as check_source;
    use mizar_core::elaborator::lower_source_algorithms as lower;
    use mizar_kernel::{
        certificate_parser::{ClauseTautologyPolicy, Fingerprint, KernelProfileRecord},
        checker::{KernelCheckStatus, KernelEvidenceCheckKind},
        clause::VariableId,
        formula_evidence::{FormulaEvidenceParseContext, parse_formula_evidence},
    };
    use mizar_proof::status::try_check_source_algorithm_state_transitivity as search;
    use mizar_proof::{
        policy::{
            AcceptedGoalPolarity, CandidatePolicyClass, KernelEvidenceOrigin, KernelPolicyInput,
            VerifierPolicy,
        },
        status::check_source_algorithm_state_transitivity as explicit,
    };
    use mizar_vc::{
        generator::generate_source_algorithm_postconditions as generate,
        kernel_evidence_handoff::{
            KernelClauseTautologyPolicy, build_source_algorithm_state_kernel_handoff as build,
        },
        vc_ir::{
            AnchorCompleteness, AnchorIngredient, GenerationSchemaVersion, SeedVcMapping,
            VcSchemaVersion, VcStatus,
        },
    };
    let config = config();
    let plan = crate::harness::build_test_plan(&config).unwrap();
    let case = plan
        .cases
        .iter()
        .find(|case| case.id.0 == "pass_proof_verification_algorithm_var_const_assert_001")
        .unwrap();
    let original = fs::read_to_string(&case.source_path).unwrap();
    assert_eq!(original.matches("    assert x = a;\n").count(), 1);
    let flat = original.replacen("    assert x = a;\n", "", 1);
    let renamed = flat
        .replace("let a be", "let input be")
        .replace("swapalgo(a)", "copyalgo(input)")
        .replace("var x :=", "var local :=")
        .replace("const c :=", "const saved :=")
        .replace("x := c;", "local := saved;")
        .replace("return x;", "return local;")
        .replace("result = a", "result = input")
        .replace(":= a;", ":= input;");
    let independent = flat.replace("    x := c;", "    const d := a;\n    x := c;");
    let generation = GenerationSchemaVersion::new("mizar-vc-generation-step5c14-state-v1");
    let schema = VcSchemaVersion::new("mizar-vc-vcset-step5c14-state-v1");
    for (variant, text) in [flat, renamed, independent].iter().enumerate() {
        let (s, n, e) = inputs(&config.workspace_root, case, frontend(text)).unwrap();
        let before_inputs = (s.clone(), n.clone(), e.clone());
        let check = check_source(&s, &n, &e).unwrap();
        let core = lower(&check).unwrap();
        let cases = [
            (0, VerifierPolicy::default()),
            (1, VerifierPolicy::default()),
            (0, VerifierPolicy::development()),
            (0, VerifierPolicy::interactive()),
        ];
        for (snapshot, policy) in cases.into_iter().take(if variant == 0 { 4 } else { 1 }) {
            let snapshot = snapshot_id(snapshot);
            let policy = policy.with_instance_search_limits(Some([0, 0, 3, 0]));
            let vcs = generate(&core, snapshot, &generation, &schema).unwrap();
            let [vc] = vcs.vcs() else {
                panic!("one genuine state VC")
            };
            assert_eq!(vc.status, VcStatus::Open);
            assert!(
                matches!(&vc.anchor.completeness, AnchorCompleteness::Incomplete { missing }
            if missing == &[AnchorIngredient::CanonicalGoalHash, AnchorIngredient::CanonicalContextHash])
            );
            let [termination, contract] = vcs.seed_accounting() else {
                panic!("actual state accounting")
            };
            assert_eq!(
                termination.seed_status,
                mizar_core::core_ir::ObligationSeedStatus::Deferred
            );
            assert!(matches!(
                &termination.mapping,
                SeedVcMapping::NoConcreteVc { .. }
            ));
            assert!(matches!(&contract.mapping, SeedVcMapping::One { vc: id } if *id == vc.id));
            let handoff = build(&core, &vcs, vc.id, &generation).unwrap();
            let envelope = handoff.canonical_evidence();
            let target = envelope.target_vc();
            let profile = envelope.kernel_profile();
            assert!(matches!(
                profile.clause_tautology_policy,
                KernelClauseTautologyPolicy::Reject
            ));
            let bytes = handoff.encode_local_evidence(None).unwrap();
            let parsed = parse_formula_evidence(
                &bytes,
                &FormulaEvidenceParseContext::v1(
                    Fingerprint::new(target.algorithm_id, target.digest.clone()),
                    KernelProfileRecord::v1(profile.profile_id, ClauseTautologyPolicy::Reject),
                ),
            )
            .unwrap();
            let before = (core.clone(), vcs.clone(), parsed.clone(), policy.clone());
            let names = encode_local_variable_transitivity_names(&parsed, VariableId(2)).unwrap();
            assert_eq!(names.len(), 1);
            assert_eq!(
                names[0]
                    .iter()
                    .map(|byte| format!("{byte:02x}"))
                    .collect::<String>(),
                if variant == 2 {
                    "000000010300000000010000000401000000020100000000"
                } else {
                    "000000010300000000010000000301000000020100000000"
                }
            );
            for limit in [None, Some(0), Some(1), Some(2)] {
                assert_eq!(
                    select_local_variable_transitivity_names(&parsed, limit),
                    None
                );
                let bounded = policy
                    .clone()
                    .with_instance_search_limits(limit.map(|limit| [0, 0, limit, 0]));
                assert_eq!(search(&s, &n, &e, snapshot, vc.id, &bounded).unwrap(), None);
            }
            for limit in [3, u64::MAX] {
                assert_eq!(
                    select_local_variable_transitivity_names(&parsed, Some(limit)),
                    Some(names.clone())
                );
                let bounded = policy
                    .clone()
                    .with_instance_search_limits(Some([0, 0, limit, 0]));
                let expected =
                    explicit(&s, &n, &e, snapshot, vc.id, VariableId(2), &bounded).unwrap();
                let actual = search(&s, &n, &e, snapshot, vc.id, &bounded).unwrap();
                let before_results = (expected.clone(), actual.clone());
                assert_eq!(actual, Some(expected.clone()));
                assert_eq!(
                    search(&s, &n, &e, snapshot, vc.id, &bounded).unwrap(),
                    actual
                );
                assert_eq!((expected, actual), before_results);
            }
            let expected = explicit(&s, &n, &e, snapshot, vc.id, VariableId(2), &policy).unwrap();
            assert_eq!(expected.0.status(), KernelCheckStatus::Accepted);
            assert_eq!(expected.1.class, CandidatePolicyClass::DischargedBuiltin);
            assert_eq!(
                expected.0.target_vc_fingerprint(),
                &mizar_kernel::rejection::TargetVcFingerprint::new(
                    target.algorithm_id,
                    target.digest.clone()
                )
            );
            assert_eq!(
                expected.0.evidence_check_kind(),
                Some(KernelEvidenceCheckKind::ProofObligation)
            );
            assert!(!expected.0.policy_taint());
            assert!(expected.0.used_axioms().is_empty());
            let trusted = KernelPolicyInput::from_kernel_result(
                &expected.0,
                KernelEvidenceOrigin::BuiltinDischarge,
            );
            assert!(trusted.accepted_evidence_hash().is_some());
            assert_eq!(
                trusted.accepted_goal_polarity(),
                Some(AcceptedGoalPolarity::AssertFalseForRefutation)
            );
            assert_eq!(
                (&core, &vcs, &parsed, &policy),
                (&before.0, &before.1, &before.2, &before.3)
            );
            assert_eq!(handoff.encode_local_evidence(None).unwrap(), bytes);
        }
        assert_eq!(
            (&s, &n, &e),
            (&before_inputs.0, &before_inputs.1, &before_inputs.2)
        );
    }
}

#[test]
fn source_algorithm_state_transitivity_search_preserves_errors_and_misses() {
    use super::super::super::{
        formula_statement::step5c8_test_frontend as frontend, source_registration_inputs as inputs,
    };
    use mizar_atp::backend::{
        encode_local_variable_transitivity_names, select_local_variable_transitivity_names,
    };
    use mizar_checker::type_checker::check_source_algorithm_types as check_source;
    use mizar_core::elaborator::lower_source_algorithms as lower;
    use mizar_kernel::{
        certificate_parser::{ClauseTautologyPolicy, Fingerprint, KernelProfileRecord},
        clause::VariableId,
        formula_evidence::{FormulaEvidenceParseContext, parse_formula_evidence},
    };
    use mizar_proof::status::try_check_source_algorithm_state_transitivity as search;
    use mizar_proof::{
        policy::{KernelEvidenceFormat, VerifierPolicy},
        status::check_source_algorithm_state_transitivity as explicit,
    };
    use mizar_vc::{
        generator::generate_source_algorithm_postconditions as generate,
        kernel_evidence_handoff::{
            KernelClauseTautologyPolicy, build_source_algorithm_state_kernel_handoff as build,
        },
        vc_ir::{GenerationSchemaVersion, VcId, VcSchemaVersion},
    };
    let config = config();
    let plan = crate::harness::build_test_plan(&config).unwrap();
    let case = plan
        .cases
        .iter()
        .find(|case| case.id.0 == "pass_proof_verification_algorithm_var_const_assert_001")
        .unwrap();
    let original = fs::read_to_string(&case.source_path).unwrap();
    let flat = original.replacen("    assert x = a;\n", "", 1);
    let return_case = plan
        .cases
        .iter()
        .find(|case| case.id.0 == "pass_proof_verification_algorithm_ensures_return_001")
        .unwrap();
    let return_source = fs::read_to_string(&return_case.source_path).unwrap();
    let generation = GenerationSchemaVersion::new("mizar-vc-generation-step5c14-state-v1");
    let schema = VcSchemaVersion::new("mizar-vc-vcset-step5c14-state-v1");
    let disabled = VerifierPolicy::default()
        .with_instance_search_limits(Some([0, 0, 3, 0]))
        .with_kernel_evidence_formats([]);
    let no_ensures = flat.replace("    ensures result = a\n", "");
    let two_returns = return_source
        .replace("    ensures result = a\n", "")
        .replace("return a;", "return a;\n    return a;");
    let bare_return = return_source.replace("return a;", "return;");
    for (fixture, text, id, stage) in [
        (case, flat.clone(), VcId::new(99), 2),
        (case, no_ensures, VcId::new(0), 2),
        (case, original, VcId::new(0), 2),
        (return_case, two_returns, VcId::new(99), 1),
        (return_case, bare_return, VcId::new(99), 0),
    ] {
        let (s, n, e) = inputs(&config.workspace_root, fixture, frontend(&text)).unwrap();
        let before = (s.clone(), n.clone(), e.clone(), disabled.clone());
        let expected = match check_source(&s, &n, &e) {
            Err(error) => {
                assert_eq!(stage, 0);
                error
            }
            Ok(check) => {
                let core = lower(&check).unwrap();
                match generate(&core, snapshot_id(0), &generation, &schema) {
                    Err(error) => {
                        assert_eq!(stage, 1);
                        error
                    }
                    Ok(vcs) => {
                        assert_eq!(stage, 2);
                        build(&core, &vcs, id, &generation).unwrap_err()
                    }
                }
            }
        };
        assert_eq!(
            explicit(&s, &n, &e, snapshot_id(0), id, VariableId(2), &disabled).unwrap_err(),
            expected
        );
        for limits in [None, Some([0; 4])] {
            let policy = disabled.clone().with_instance_search_limits(limits);
            assert_eq!(
                search(&s, &n, &e, snapshot_id(0), id, &policy).unwrap_err(),
                expected
            );
        }
        assert_eq!((s, n, e, disabled.clone()), before);
    }
    let no_chain = flat.replace("    x := c;", "    x := a;");
    for (text, miss) in [(&flat, false), (&no_chain, true)] {
        let (s, n, e) = inputs(&config.workspace_root, case, frontend(text)).unwrap();
        let check = check_source(&s, &n, &e).unwrap();
        let core = lower(&check).unwrap();
        let vcs = generate(&core, snapshot_id(0), &generation, &schema).unwrap();
        let id = vcs.vcs()[0].id;
        let handoff = build(&core, &vcs, id, &generation).unwrap();
        let envelope = handoff.canonical_evidence();
        let target = envelope.target_vc();
        let profile = envelope.kernel_profile();
        assert!(matches!(
            profile.clause_tautology_policy,
            KernelClauseTautologyPolicy::Reject
        ));
        let bytes = handoff.encode_local_evidence(None).unwrap();
        let parsed = parse_formula_evidence(
            &bytes,
            &FormulaEvidenceParseContext::v1(
                Fingerprint::new(target.algorithm_id, target.digest.clone()),
                KernelProfileRecord::v1(profile.profile_id, ClauseTautologyPolicy::Reject),
            ),
        )
        .unwrap();
        let before = (
            s.clone(),
            n.clone(),
            e.clone(),
            core.clone(),
            vcs.clone(),
            parsed.clone(),
        );
        assert_eq!(
            select_local_variable_transitivity_names(&parsed, Some(u64::MAX)).is_none(),
            miss
        );
        let expected_error = if miss {
            encode_local_variable_transitivity_names(&parsed, VariableId(2)).unwrap_err()
        } else {
            "algorithm.proof.kernel_policy_rejected".to_owned()
        };
        for policy in [
            disabled.clone(),
            disabled
                .clone()
                .with_kernel_evidence_formats([KernelEvidenceFormat::FormulaSubstitution]),
        ] {
            assert_eq!(
                explicit(&s, &n, &e, snapshot_id(0), id, VariableId(2), &policy).unwrap_err(),
                expected_error
            );
            for limit in [None, Some(0), Some(2), Some(3), Some(u64::MAX)] {
                let bounded = policy
                    .clone()
                    .with_instance_search_limits(limit.map(|limit| [0, 0, limit, 0]));
                let actual = search(&s, &n, &e, snapshot_id(0), id, &bounded);
                assert_eq!(search(&s, &n, &e, snapshot_id(0), id, &bounded), actual);
                if miss || limit.unwrap_or(0) < 3 {
                    assert_eq!(actual.unwrap(), None);
                } else {
                    assert_eq!(actual.unwrap_err(), expected_error);
                }
            }
        }
        assert_eq!((s, n, e, core, vcs, parsed), before);
        assert_eq!(handoff.encode_local_evidence(None).unwrap(), bytes);
    }
}

#[test]
fn source_algorithm_state_transitivity_search_status_matches_original_projection() {
    use super::with_source_algorithm_identity_inputs as with_inputs;
    use mizar_kernel::{checker::KernelCheckStatus, clause::VariableId};
    use mizar_proof::status::try_project_source_algorithm_state_transitivity_status as optional;
    use mizar_proof::{
        policy::{
            CandidatePolicyClass as Class, KernelEvidenceOrigin, KernelPolicyInput, VerifierPolicy,
        },
        selection::{
            CandidateSourceId, ProofEvidenceCandidate, ProofEvidenceSet, ProofWinnerClass,
            VcProofSelection, merge_artifact_proof_selections, select_winner,
        },
        status::{
            ArtifactPublicationGap, ProofStatusProjectionInput,
            project_source_algorithm_state_transitivity_status as explicit, project_status,
            source_algorithm_state_obligation_identity as identify,
            try_check_source_algorithm_state_transitivity as normal,
        },
    };
    use mizar_vc::{
        dependency_slice::{
            DependencySliceCompleteness, DependencySliceInput, try_compute_dependency_slices,
        },
        generator::generate_source_algorithm_postconditions as generate,
        vc_ir::{AnchorCompleteness, GenerationSchemaVersion, VcSchemaVersion, VcStatus},
    };
    let package = "mizar-test-corpus";
    let module = "tests.miz.pass.algorithms.pass_proof_verification_algorithm_var_const_assert_001";
    let original = fs::read_to_string(config().workspace_root.join(
        "tests/miz/pass/algorithms/pass_proof_verification_algorithm_var_const_assert_001.miz",
    ))
    .unwrap();
    assert_eq!(original.matches("    assert x = a;\n").count(), 1);
    let flat = original.replacen("    assert x = a;\n", "", 1);
    let renamed = flat
        .replace("let a be", "let input be")
        .replace("swapalgo(a)", "copyalgo(input)")
        .replace("var x :=", "var local :=")
        .replace("const c :=", "const saved :=")
        .replace("x := c;", "local := saved;")
        .replace("return x;", "return local;")
        .replace("result = a", "result = input")
        .replace(":= a;", ":= input;");
    let independent = flat.replace("    x := c;", "    const d := a;\n    x := c;");
    let generation = GenerationSchemaVersion::new("mizar-vc-generation-step5c14-state-v1");
    let schema = VcSchemaVersion::new("mizar-vc-vcset-step5c14-state-v1");
    for (variant, text) in [flat, renamed, independent].iter().enumerate() {
        with_inputs(text, package, module, 0, |s, n, e, c| {
            let before = format!("{s:?}{n:?}{e:?}{c:?}");
            let core = mizar_core::elaborator::lower_source_algorithms(c).unwrap();
            let cases = [
                (0, VerifierPolicy::default()),
                (1, VerifierPolicy::default()),
                (0, VerifierPolicy::development()),
                (0, VerifierPolicy::interactive()),
            ];
            for (snapshot, p) in cases.into_iter().take(if variant == 0 { 4 } else { 1 }) {
                let snapshot = snapshot_id(snapshot);
                let p = p.with_instance_search_limits(Some([0, 0, 3, 0]));
                let vcs = generate(&core, snapshot, &generation, &schema).unwrap();
                let [vc] = vcs.vcs() else {
                    panic!("one original raw state VC")
                };
                assert_eq!(vc.status, VcStatus::Open);
                assert!(matches!(
                    &vc.anchor.completeness,
                    AnchorCompleteness::Incomplete { .. }
                ));
                assert!(
                    vcs.seed_accounting().iter().any(|seed| seed.seed_status
                        == mizar_core::core_ir::ObligationSeedStatus::Deferred)
                );
                let slices = try_compute_dependency_slices(DependencySliceInput {
                    vc_set: &vcs,
                    discharge_output: None,
                })
                .unwrap();
                let whole = slices.slice_for(vc.id).unwrap().clone();
                let (identity, slice) = identify(s, c, e, snapshot, vc.id).unwrap();
                assert_eq!(slice, whole);
                assert_eq!(
                    slice.completeness(),
                    DependencySliceCompleteness::IncompleteUncacheable
                );
                assert!(slice.requires_cache_miss());
                let graph_before = (core.clone(), vcs.clone(), p.clone());
                let checked = normal(s, n, e, snapshot, vc.id, &p).unwrap().unwrap();
                let (result, decision) = &checked;
                assert_eq!(result.status(), KernelCheckStatus::Accepted);
                assert!(!result.policy_taint());
                assert_eq!(decision.class, Class::DischargedBuiltin);
                let target = result.target_vc_fingerprint().sort_bytes();
                let id_text = format!(
                    "source-algorithm-state-transitivity-v1:{}",
                    target
                        .iter()
                        .map(|byte| format!("{byte:02x}"))
                        .collect::<String>()
                );
                assert_ne!(id_text, identity.obligation_id());
                let id = CandidateSourceId::new(id_text.clone()).unwrap();
                let candidate = if decision.class == Class::DischargedBuiltin {
                    ProofEvidenceCandidate::from_trusted_kernel_input(
                        id,
                        &KernelPolicyInput::from_kernel_result(
                            result,
                            KernelEvidenceOrigin::BuiltinDischarge,
                        ),
                    )
                    .unwrap()
                } else {
                    ProofEvidenceCandidate::new(id, decision.clone())
                };
                let selected = select_winner(
                    &ProofEvidenceSet::new(target, None, p.clone()).with_candidates([candidate]),
                );
                let [row] = <[_; 1]>::try_from(
                    merge_artifact_proof_selections([], [VcProofSelection::new(vc.id, selected)])
                        .unwrap(),
                )
                .unwrap();
                let projected = project_status(ProofStatusProjectionInput::new(
                    row,
                    p.clone(),
                    identity.clone(),
                ))
                .unwrap();
                let expected = (projected, whole);
                let actual = explicit(s, c, e, snapshot, vc.id, VariableId(2), &p).unwrap();
                assert_eq!(actual, expected);
                for bound in [None, Some(0), Some(1), Some(2)] {
                    let bounded = p
                        .clone()
                        .with_instance_search_limits(bound.map(|bound| [0, 0, bound, 0]));
                    assert_eq!(optional(s, c, e, snapshot, vc.id, &bounded).unwrap(), None);
                }
                for bound in [3, u64::MAX] {
                    let bounded = p
                        .clone()
                        .with_instance_search_limits(Some([0, 0, bound, 0]));
                    let expected =
                        explicit(s, c, e, snapshot, vc.id, VariableId(2), &bounded).unwrap();
                    let before_result = expected.clone();
                    let projected = optional(s, c, e, snapshot, vc.id, &bounded).unwrap();
                    assert_eq!(projected, Some(expected.clone()));
                    assert_eq!(
                        optional(s, c, e, snapshot, vc.id, &bounded).unwrap(),
                        projected
                    );
                    assert_eq!(expected, before_result);
                }
                assert_eq!(
                    actual,
                    explicit(s, c, e, snapshot, vc.id, VariableId(2), &p).unwrap()
                );
                assert_eq!(actual.0.vc(), vc.id);
                assert_eq!(actual.0.obligation_id(), identity.obligation_id());
                assert_eq!(
                    actual.0.selected_class(),
                    ProofWinnerClass::DischargedBuiltin
                );
                assert_eq!(
                    actual.0.artifact_publication().gap(),
                    Some(ArtifactPublicationGap::DischargedBuiltinWitnessUnsupported)
                );
                assert!(actual.0.accepted_witness_obligation_id().is_none());
                assert!(actual.0.trusted_used_axioms().is_none());
                assert!(actual.0.explanation_ref().is_none());
                let reuse = actual.0.reuse_metadata();
                assert_eq!(reuse.selected_candidate_id().unwrap().as_str(), id_text);
                assert!(reuse.trusted_used_axioms_hash().is_none());
                assert!(reuse.selected_proof_witness_hash().is_none());
                assert!(reuse.dependency_compatibility().is_none());
                assert!(!reuse.cache_reuse_predicate_complete());
                assert_eq!(
                    identify(s, c, e, snapshot, vc.id).unwrap(),
                    (identity, slice)
                );
                assert_eq!(normal(s, n, e, snapshot, vc.id, &p).unwrap(), Some(checked));
                assert_eq!(
                    (&core, &vcs, &p),
                    (&graph_before.0, &graph_before.1, &graph_before.2)
                );
            }
            assert_eq!(format!("{s:?}{n:?}{e:?}{c:?}"), before);
        });
    }
}

#[test]
fn source_algorithm_state_transitivity_search_status_preserves_errors_and_absence() {
    use super::super::super::tests::source_registration_owner_inputs as owners;
    use super::with_source_algorithm_identity_inputs as with_inputs;
    use mizar_checker::type_checker::check_source_algorithm_types as check_source;
    use mizar_kernel::clause::VariableId;
    use mizar_proof::status::try_project_source_algorithm_state_transitivity_status as optional;
    use mizar_proof::{
        policy::VerifierPolicy,
        status::{
            project_source_algorithm_state_transitivity_status as explicit,
            source_algorithm_state_obligation_identity as identify,
            try_check_source_algorithm_state_transitivity as normal,
        },
    };
    use mizar_vc::{
        dependency_slice::{DependencySliceInput, try_compute_dependency_slices},
        generator::generate_source_algorithm_postconditions as generate,
        vc_ir::{GenerationSchemaVersion, VcId, VcSchemaVersion},
    };
    let package = "mizar-test-corpus";
    let module = "tests.miz.pass.algorithms.pass_proof_verification_algorithm_var_const_assert_001";
    let root = config().workspace_root;
    let original = fs::read_to_string(root.join(
        "tests/miz/pass/algorithms/pass_proof_verification_algorithm_var_const_assert_001.miz",
    ))
    .unwrap();
    let flat = original.replacen("    assert x = a;\n", "", 1);
    let return_source = fs::read_to_string(root.join(
        "tests/miz/pass/algorithms/pass_proof_verification_algorithm_ensures_return_001.miz",
    ))
    .unwrap();
    let two_returns = return_source
        .replace("    ensures result = a\n", "")
        .replace("return a;", "return a;\n    return a;");
    let (_, _, foreign) = owners(
        &flat.replace("swapalgo", "foreign_algorithm"),
        package,
        module,
        0,
    );
    let (_, _, augmented) = owners(&format!("{flat}\n{flat}"), package, module, 0);
    let generation = GenerationSchemaVersion::new("mizar-vc-generation-step5c14-state-v1");
    let schema = VcSchemaVersion::new("mizar-vc-vcset-step5c14-state-v1");
    let disabled = VerifierPolicy::default()
        .with_instance_search_limits(Some([0, 0, u64::MAX, 0]))
        .with_kernel_evidence_formats([]);
    for (kind, text) in [
        ("absent", flat.clone()),
        ("zero", flat.replace("    ensures result = a\n", "")),
        ("assertion", original),
        ("two", two_returns),
        ("foreign", flat.clone()),
        ("augmented", flat.clone()),
    ] {
        with_inputs(&text, package, module, 0, |s, n, e, c| {
            let e = match kind {
                "foreign" => foreign.env(),
                "augmented" => augmented.env(),
                _ => e,
            };
            let before = format!("{s:?}{n:?}{e:?}{c:?}");
            let id = VcId::new(if matches!(kind, "absent" | "two") {
                99
            } else {
                0
            });
            let expected = identify(s, c, e, snapshot_id(0), id).unwrap_err();
            if matches!(kind, "foreign" | "augmented") {
                assert_eq!(check_source(s, n, e).unwrap_err(), expected);
            } else if kind == "two" {
                let core = mizar_core::elaborator::lower_source_algorithms(c).unwrap();
                assert_eq!(
                    generate(&core, snapshot_id(0), &generation, &schema).unwrap_err(),
                    expected
                );
            }
            assert_eq!(
                explicit(s, c, e, snapshot_id(0), id, VariableId(99), &disabled).unwrap_err(),
                expected
            );
            for limits in [None, Some([0; 4])] {
                let bounded = disabled.clone().with_instance_search_limits(limits);
                assert_eq!(
                    optional(s, c, e, snapshot_id(0), id, &bounded).unwrap_err(),
                    expected
                );
            }
            assert_eq!(format!("{s:?}{n:?}{e:?}{c:?}"), before);
        });
    }
    let no_chain = flat.replace("    x := c;", "    x := a;");
    assert_ne!(flat, no_chain);
    for (text, miss) in [(&flat, false), (&no_chain, true)] {
        with_inputs(text, package, module, 0, |s, n, e, c| {
            let before = format!("{s:?}{n:?}{e:?}{c:?}");
            let core = mizar_core::elaborator::lower_source_algorithms(c).unwrap();
            let vcs = generate(&core, snapshot_id(0), &generation, &schema).unwrap();
            let id = vcs.vcs()[0].id;
            let original = identify(s, c, e, snapshot_id(0), id).unwrap();
            let slices = try_compute_dependency_slices(DependencySliceInput {
                vc_set: &vcs,
                discharge_output: None,
            })
            .unwrap();
            assert_eq!(&original.1, slices.slice_for(id).unwrap());
            let checked = normal(s, n, e, snapshot_id(0), id, &disabled);
            if miss {
                assert_eq!(checked.unwrap(), None);
            } else {
                assert_eq!(
                    checked.unwrap_err(),
                    "algorithm.proof.kernel_policy_rejected"
                );
                assert_eq!(
                    explicit(s, c, e, snapshot_id(0), id, VariableId(2), &disabled).unwrap_err(),
                    "algorithm.proof.kernel_policy_rejected"
                );
            }
            for bound in [None, Some(0), Some(2), Some(3), Some(u64::MAX)] {
                let bounded = disabled
                    .clone()
                    .with_instance_search_limits(bound.map(|bound| [0, 0, bound, 0]));
                let checked = normal(s, n, e, snapshot_id(0), id, &bounded);
                let projected = optional(s, c, e, snapshot_id(0), id, &bounded);
                assert_eq!(optional(s, c, e, snapshot_id(0), id, &bounded), projected);
                if miss || bound.unwrap_or(0) < 3 {
                    assert_eq!(checked.unwrap(), None);
                    assert_eq!(projected.unwrap(), None);
                } else {
                    assert_eq!(projected.unwrap_err(), checked.unwrap_err());
                }
            }
            assert_eq!(identify(s, c, e, snapshot_id(0), id).unwrap(), original);
            assert_eq!(format!("{s:?}{n:?}{e:?}{c:?}"), before);
        });
    }
}

#[test]
fn source_algorithm_state_transitivity_search_draft_matches_checked_wire() {
    use super::with_source_algorithm_state_draft_inputs as with_inputs;
    use mizar_artifact::{
        registration_summary::{ArtifactHashClass as Class, ArtifactHashRef as Ref},
        store::{
            CanonicalJson as J, SchemaVersion as V, artifact_hash_domain, canonical_json_bytes,
        },
    };
    use mizar_atp::backend::{
        encode_local_variable_transitivity_names as names,
        select_local_variable_transitivity_names as select,
    };
    use mizar_kernel::{
        certificate_parser::{ClauseTautologyPolicy, Fingerprint, KernelProfileRecord},
        clause::VariableId,
        formula_evidence::{FormulaEvidenceParseContext, parse_formula_evidence as parse},
    };
    use mizar_proof::status::try_source_algorithm_state_transitivity_witness_draft as optional;
    use mizar_proof::{
        policy::{KernelEvidenceOrigin, KernelPolicyInput, VerifierPolicy},
        selection::CandidateSourceId,
        status::{
            ArtifactPublicationGap, check_source_algorithm_state_transitivity as normal,
            source_algorithm_state_obligation_identity as identify,
            source_algorithm_state_transitivity_witness_draft as explicit,
        },
        witness_store::{ProofWitnessPayloadSchema, ProofWitnessProvenance, stage},
    };
    use mizar_vc::{
        dependency_slice::DependencySliceCompleteness,
        generator::generate_source_algorithm_postconditions as generate,
        kernel_evidence_handoff::{
            KernelClauseTautologyPolicy::Reject as LocalReject,
            build_source_algorithm_state_kernel_handoff as build,
        },
        vc_ir::{GenerationSchemaVersion, VcSchemaVersion, VcStatus},
    };
    let root = config().workspace_root.join("tests/miz/pass/algorithms");
    let original =
        fs::read_to_string(root.join("pass_proof_verification_algorithm_var_const_assert_001.miz"))
            .unwrap();
    assert_eq!(original.matches("    assert x = a;\n").count(), 1);
    let flat = original.replacen("    assert x = a;\n", "", 1);
    let renamed = flat
        .replace("let a be", "let input be")
        .replace("swapalgo(a)", "copyalgo(input)")
        .replace("var x :=", "var local :=")
        .replace("const c :=", "const saved :=")
        .replace("x := c;", "local := saved;")
        .replace("return x;", "return local;")
        .replace("result = a", "result = input")
        .replace(":= a;", ":= input;");
    let independent = flat.replace("    x := c;", "    const d := a;\n    x := c;");
    let p = VerifierPolicy::default().with_instance_search_limits(Some([0, 0, 3, 0]));
    let generation = GenerationSchemaVersion::new("mizar-vc-generation-step5c14-state-v1");
    let schema = VcSchemaVersion::new("mizar-vc-vcset-step5c14-state-v1");
    for text in [flat, renamed, independent] {
        with_inputs(&text, &p, |s, c, e, b| {
            let before = format!("{s:?}{c:?}{e:?}{b:?}{p:?}");
            let core = mizar_core::elaborator::lower_source_algorithms(c).unwrap();
            let vcs = generate(&core, b.id, &generation, &schema).unwrap();
            let [vc] = vcs.vcs() else {
                panic!("native singleton discovery")
            };
            assert_eq!(vc.status, VcStatus::Open);
            assert!(vcs.seed_accounting().iter().any(
                |seed| seed.seed_status == mizar_core::core_ir::ObligationSeedStatus::Deferred
            ));
            let (identity, slice) = identify(s, c, e, b.id, vc.id).unwrap();
            assert_eq!(
                slice.completeness(),
                DependencySliceCompleteness::IncompleteUncacheable
            );
            assert!(slice.requires_cache_miss());
            let handoff = build(&core, &vcs, vc.id, &generation).unwrap();
            let envelope = handoff.canonical_evidence();
            let target = envelope.target_vc();
            let profile = envelope.kernel_profile();
            assert!(matches!(profile.clause_tautology_policy, LocalReject));
            let mut context = FormulaEvidenceParseContext::v1(
                Fingerprint::new(target.algorithm_id, target.digest.clone()),
                KernelProfileRecord::v1(profile.profile_id, ClauseTautologyPolicy::Reject),
            );
            let parsed = parse(&handoff.encode_local_evidence(None).unwrap(), &context).unwrap();
            let names = names(&parsed, VariableId(2)).unwrap();
            assert_eq!(select(&parsed, Some(3)), Some(names.clone()));
            assert_eq!(select(&parsed, Some(u64::MAX)), Some(names.clone()));
            let wire = handoff.encode_local_evidence(Some(&names)).unwrap();
            context.accepted_schema_versions.insert(2);
            let parsed = parse(&wire, &context).unwrap();
            let checked = normal(s, c.typed(), e, b.id, vc.id, VariableId(2), &p).unwrap();
            let input = KernelPolicyInput::from_kernel_result(
                &checked.0,
                KernelEvidenceOrigin::BuiltinDischarge,
            );
            let accepted = input.accepted_evidence_hash().unwrap();
            let path = "proof-witnesses/state-search.json";
            let product = explicit(s, c, e, b, VariableId(2), &p, path).unwrap();
            assert_eq!(product.1, slice);
            let draft = &product.2;
            assert_eq!(draft.payload_bytes(), wire);
            let original_hash = Ref::new(
                Class::Interface,
                "mizar-proof/source-algorithm-state-original-obligation",
                V::new(1, 0),
                identity.obligation_fingerprint(),
            );
            let h = handoff.local_evidence_component_hashes().unwrap();
            let m = draft.kernel_acceptance();
            let policy_hash = p.policy_fingerprint().hash();
            let version = V::new(parsed.schema_version(), 0);
            assert_eq!(
                [
                    m.kernel_profile_fingerprint.digest,
                    m.formula_evidence_hash.digest,
                    m.substitution_evidence_hash.digest,
                    m.provenance_hash.digest
                ],
                h
            );
            assert_eq!(m.target_binding_hash.digest, handoff.canonical_hash());
            assert_eq!(m.accepted_result_hash.digest, accepted);
            assert_eq!(m.verifier_policy_fingerprint.digest, policy_hash);
            assert_eq!(m.evidence_schema_version, version);
            assert!(m.formula_context_hash.is_none());
            let hex = |bytes: &[u8]| {
                bytes
                    .iter()
                    .map(|byte| format!("{byte:02x}"))
                    .collect::<String>()
            };
            let candidate = CandidateSourceId::new(format!(
                "source-algorithm-state-transitivity-v1:{}",
                hex(&checked.0.target_vc_fingerprint().sort_bytes())
            ))
            .unwrap();
            let provenance = ProofWitnessProvenance::new(
                b.id.fingerprint(),
                "mizar-proof/source-algorithm-state-transitivity-witness-draft/v1",
                candidate,
                KernelEvidenceOrigin::BuiltinDischarge,
                identity.vc_fingerprint(),
                identity.dependency_slice_fingerprint(),
                b.dependency_artifact_fingerprint(),
                policy_hash,
                V::new(p.checker_schema_version(), 0),
                version,
            )
            .unwrap()
            .with_accepted_result_hash(accepted);
            assert_eq!(draft.provenance(), &provenance);
            let ps = ProofWitnessPayloadSchema::with_canonical_bytes_required(
                "mizar-proof/source-algorithm-state-transitivity-checked-wire",
                version,
                true,
            )
            .unwrap();
            assert_eq!(draft.payload_schema(), &ps);
            let value = J::object(
                [
                    ("hash_domain", "mizar-proof/witness-payload/v1".to_owned()),
                    (
                        "obligation_fingerprint",
                        original_hash.to_artifact_hash_string(),
                    ),
                    ("payload_bytes_hex", hex(&wire)),
                    ("payload_schema_family", ps.family().to_owned()),
                    ("payload_schema_version", ps.version().to_string()),
                    ("selected_evidence_hash", hex(accepted.as_bytes())),
                    ("verifier_policy_fingerprint", hex(policy_hash.as_bytes())),
                ]
                .map(|(key, value)| (key, J::string(value))),
            )
            .unwrap();
            assert_eq!(canonical_json_bytes(&value).last(), Some(&b'\n'));
            assert_eq!(draft.payload_hash_value(), value);
            let hash = artifact_hash_domain(ps.family(), ps.version()).hash(&value, &[]);
            let staged = stage(draft.clone()).unwrap();
            assert_eq!(staged.staged_payload_hash().digest, hash);
            assert!(staged.artifact_reference_candidate().is_none());
            for bound in [None, Some(0), Some(1), Some(2), Some(3), Some(u64::MAX)] {
                let bounded = p
                    .clone()
                    .with_instance_search_limits(bound.map(|bound| [0, 0, bound, 0]));
                let expected = if bound.unwrap_or(0) >= 3 {
                    Some(explicit(s, c, e, b, VariableId(2), &bounded, path).unwrap())
                } else {
                    None
                };
                let before_result = expected.clone();
                let actual = optional(s, c, e, b, &bounded, path).unwrap();
                assert_eq!(actual, expected);
                assert_eq!(optional(s, c, e, b, &bounded, path).unwrap(), actual);
                assert_eq!(expected, before_result);
            }
            assert_eq!(
                product.0.artifact_publication().gap(),
                Some(ArtifactPublicationGap::DischargedBuiltinWitnessUnsupported)
            );
            assert!(!product.0.reuse_metadata().cache_reuse_predicate_complete());
            let reuse = product.0.reuse_metadata();
            assert!(reuse.selected_proof_witness_hash().is_none());
            assert_eq!(
                explicit(s, c, e, b, VariableId(2), &p, path).unwrap(),
                product
            );
            assert_eq!(identify(s, c, e, b.id, vc.id).unwrap(), (identity, slice));
            assert_eq!(format!("{s:?}{c:?}{e:?}{b:?}{p:?}"), before);
        });
    }
}

#[test]
fn source_algorithm_state_transitivity_search_draft_preserves_errors_and_absence() {
    use super::super::super::tests::source_registration_owner_inputs as owners;
    use super::with_source_algorithm_state_draft_inputs as with_inputs;
    use mizar_atp::backend::select_local_variable_transitivity_names as select;
    use mizar_checker::type_checker::check_source_algorithm_types as check_source;
    use mizar_kernel::{
        certificate_parser::{ClauseTautologyPolicy, Fingerprint, KernelProfileRecord},
        clause::VariableId,
        formula_evidence::{FormulaEvidenceParseContext, parse_formula_evidence as parse},
    };
    use mizar_proof::status::try_source_algorithm_state_transitivity_witness_draft as optional;
    use mizar_proof::{
        policy::VerifierPolicy,
        status::{
            source_algorithm_state_obligation_identity as identify,
            source_algorithm_state_transitivity_witness_draft as explicit,
            try_check_source_algorithm_state_transitivity as normal,
        },
    };
    use mizar_vc::{
        generator::generate_source_algorithm_postconditions as generate,
        kernel_evidence_handoff::{
            KernelClauseTautologyPolicy::Reject as LocalReject,
            build_source_algorithm_state_kernel_handoff as build,
        },
        vc_ir::{GenerationSchemaVersion, VcSchemaVersion},
    };
    let root = config().workspace_root.join("tests/miz/pass/algorithms");
    let original =
        fs::read_to_string(root.join("pass_proof_verification_algorithm_var_const_assert_001.miz"))
            .unwrap();
    let flat = original.replacen("    assert x = a;\n", "", 1);
    let return_source =
        fs::read_to_string(root.join("pass_proof_verification_algorithm_ensures_return_001.miz"))
            .unwrap();
    let two_returns = return_source
        .replace("    ensures result = a\n", "")
        .replace("return a;", "return a;\n    return a;");
    let p = VerifierPolicy::default().with_instance_search_limits(Some([0, 0, 3, 0]));
    let disabled = p.clone().with_kernel_evidence_formats([]);
    let foreign = with_inputs(
        &flat.replace("swapalgo", "foreign_algorithm"),
        &p,
        |_, _, e, _| e.clone(),
    );
    let (_, _, augmented) = owners(
        &format!("{flat}\n{flat}"),
        "mizar-test-corpus",
        "tests.miz.pass.algorithms.pass_proof_verification_algorithm_var_const_assert_001",
        0,
    );
    let generation = GenerationSchemaVersion::new("mizar-vc-generation-step5c14-state-v1");
    let schema = VcSchemaVersion::new("mizar-vc-vcset-step5c14-state-v1");
    let zero = flat.replace("    ensures result = a\n", "");
    let assertion = original.replace("    ensures result = a\n", "");
    for (kind, text, count) in [
        ("zero", zero, Some(0)),
        ("multi", original, Some(2)),
        ("assertion", assertion, Some(1)),
        ("two", two_returns, None),
        ("foreign", flat.clone(), Some(1)),
        ("augmented", flat.clone(), Some(1)),
    ] {
        with_inputs(&text, &p, |s, c, e, b| {
            let e = match kind {
                "foreign" => &foreign,
                "augmented" => augmented.env(),
                _ => e,
            };
            let before = format!("{s:?}{c:?}{e:?}{b:?}{p:?}");
            let expected = if matches!(kind, "foreign" | "augmented") {
                check_source(s, c.typed(), e).unwrap_err()
            } else {
                let core = mizar_core::elaborator::lower_source_algorithms(c).unwrap();
                let vcs = generate(&core, b.id, &generation, &schema);
                match count {
                    Some(count) => {
                        let vcs = vcs.unwrap();
                        assert_eq!(vcs.vcs().len(), count);
                        match vcs.vcs() {
                            [vc] => identify(s, c, e, b.id, vc.id).unwrap_err(),
                            _ => "algorithm.proof.invalid_source_state_witness_draft".to_owned(),
                        }
                    }
                    None => vcs.unwrap_err(),
                }
            };
            assert_eq!(
                explicit(s, c, e, b, VariableId(99), &disabled, "../invalid").unwrap_err(),
                expected
            );
            for limits in [None, Some([0; 4])] {
                let bounded = disabled.clone().with_instance_search_limits(limits);
                assert_eq!(
                    optional(s, c, e, b, &bounded, "../invalid").unwrap_err(),
                    expected
                );
            }
            assert_eq!(format!("{s:?}{c:?}{e:?}{b:?}{p:?}"), before);
        });
    }
    let no_chain = flat.replace("    x := c;", "    x := a;");
    assert_ne!(no_chain, flat);
    for (text, miss) in [(&flat, false), (&no_chain, true)] {
        with_inputs(text, &p, |s, c, e, b| {
            let before = format!("{s:?}{c:?}{e:?}{b:?}{p:?}");
            let core = mizar_core::elaborator::lower_source_algorithms(c).unwrap();
            let vcs = generate(&core, b.id, &generation, &schema).unwrap();
            let [vc] = vcs.vcs() else {
                panic!("native singleton before no-chain observation")
            };
            let original = identify(s, c, e, b.id, vc.id).unwrap();
            assert!(original.1.requires_cache_miss());
            let handoff = build(&core, &vcs, vc.id, &generation).unwrap();
            let profile = handoff.canonical_evidence().kernel_profile();
            let target = handoff.canonical_evidence().target_vc();
            assert!(matches!(profile.clause_tautology_policy, LocalReject));
            let parsed = parse(
                &handoff.encode_local_evidence(None).unwrap(),
                &FormulaEvidenceParseContext::v1(
                    Fingerprint::new(target.algorithm_id, target.digest.clone()),
                    KernelProfileRecord::v1(profile.profile_id, ClauseTautologyPolicy::Reject),
                ),
            )
            .unwrap();
            let max = p
                .clone()
                .with_instance_search_limits(Some([0, 0, u64::MAX, 0]));
            assert_eq!(select(&parsed, Some(u64::MAX)).is_none(), miss);
            for bound in [None, Some(0), Some(1), Some(2)] {
                let bounded = disabled
                    .clone()
                    .with_instance_search_limits(bound.map(|bound| [0, 0, bound, 0]));
                assert_eq!(optional(s, c, e, b, &bounded, "../invalid").unwrap(), None);
            }
            if miss {
                assert_eq!(normal(s, c.typed(), e, b.id, vc.id, &max).unwrap(), None);
                let bounded = max.clone().with_kernel_evidence_formats([]);
                assert_eq!(optional(s, c, e, b, &bounded, "../invalid").unwrap(), None);
            } else {
                let expected = normal(s, c.typed(), e, b.id, vc.id, &disabled).unwrap_err();
                assert_eq!(expected, "algorithm.proof.kernel_policy_rejected");
                assert_eq!(
                    optional(s, c, e, b, &disabled, "../invalid").unwrap_err(),
                    expected
                );
                assert_eq!(
                    explicit(s, c, e, b, VariableId(2), &disabled, "../invalid").unwrap_err(),
                    expected
                );
                let invalid = explicit(s, c, e, b, VariableId(2), &p, "../invalid").unwrap_err();
                assert_eq!(
                    invalid,
                    "invalid proof witness path: path must start with proof-witnesses/"
                );
                assert_eq!(optional(s, c, e, b, &p, "../invalid").unwrap_err(), invalid);
            }
            assert_eq!(identify(s, c, e, b.id, vc.id).unwrap(), original);
            assert_eq!(format!("{s:?}{c:?}{e:?}{b:?}{p:?}"), before);
        });
    }
}

#[test]
fn source_algorithm_state_witness_file_matches_genuine_transport() {
    use super::with_source_algorithm_state_draft_inputs as with_inputs;
    use mizar_artifact::{
        proof_witness::{
            EvidenceKind, ProofStatus, ProofWitnessReadOptions, ProofWitnessRef,
            builtin_proof_witness_ref_json, read_builtin_proof_witness_ref,
        },
        registration_summary::{ArtifactHashClass as Class, ArtifactHashRef as Ref},
        store::{
            CanonicalJson as J, PublishedArtifactPath, PublishedArtifactReadOptions,
            SchemaVersion as V, artifact_hash_domain, canonical_json_bytes,
            read_published_artifact, write_published_artifact,
        },
    };
    use mizar_proof::{
        policy::VerifierPolicy,
        status::{
            ArtifactPublicationGap,
            try_source_algorithm_state_transitivity_witness_draft as acquire,
            try_source_algorithm_state_transitivity_witness_file_candidate as candidate,
        },
        witness_store::{read_source_theorem_witness_file as recover, stage},
    };
    use mizar_vc::{dependency_slice::DependencySliceCompleteness, vc_ir::VcStatus};
    let original = fs::read_to_string(config().workspace_root.join(
        "tests/miz/pass/algorithms/pass_proof_verification_algorithm_var_const_assert_001.miz",
    ))
    .unwrap();
    let flat = original.replacen("    assert x = a;\n", "", 1);
    let renamed = flat
        .replace("let a be", "let input be")
        .replace("swapalgo(a)", "copyalgo(input)")
        .replace("var x :=", "var local :=")
        .replace("const c :=", "const saved :=")
        .replace("x := c;", "local := saved;")
        .replace("return x;", "return local;")
        .replace("result = a", "result = input")
        .replace(":= a;", ":= input;");
    let independent = flat.replace("    x := c;", "    const d := a;\n    x := c;");
    let temporary = std::process::Command::new("mktemp")
        .arg("-d")
        .output()
        .unwrap();
    assert!(temporary.status.success());
    let transport = std::path::PathBuf::from(String::from_utf8(temporary.stdout).unwrap().trim());
    let p = VerifierPolicy::default().with_instance_search_limits(Some([0, 0, 3, 0]));
    for text in [flat, renamed, independent] {
        with_inputs(&text, &p, |s, c, e, b| {
            let before = format!("{s:?}{c:?}{e:?}{b:?}{p:?}");
            let path = "proof-witnesses/state-file/checked.json";
            let baseline = acquire(s, c, e, b, &p, path).unwrap().unwrap();
            let slice = &baseline.1;
            assert_eq!(slice.status(), &VcStatus::Open);
            assert_eq!(
                slice.completeness(),
                DependencySliceCompleteness::IncompleteUncacheable
            );
            assert!(slice.requires_cache_miss());
            let d = &baseline.2;
            let m = d.kernel_acceptance();
            let reuse = baseline.0.reuse_metadata();
            let mut metadata = std::collections::BTreeMap::new();
            for (key, hash) in [
                ("kernel_profile_fingerprint", &m.kernel_profile_fingerprint),
                (
                    "verifier_policy_fingerprint",
                    &m.verifier_policy_fingerprint,
                ),
                ("target_binding_hash", &m.target_binding_hash),
                ("formula_evidence_hash", &m.formula_evidence_hash),
                ("substitution_evidence_hash", &m.substitution_evidence_hash),
                ("provenance_hash", &m.provenance_hash),
                ("accepted_result_hash", &m.accepted_result_hash),
            ] {
                metadata.insert(key.to_owned(), J::string(hash.to_artifact_hash_string()));
            }
            metadata.insert(
                "checker_schema_version".into(),
                J::string(m.checker_schema_version.to_string()),
            );
            metadata.insert(
                "evidence_schema_version".into(),
                J::string(m.evidence_schema_version.to_string()),
            );
            metadata.insert(
                "formula_context_hash".into(),
                m.formula_context_hash
                    .as_ref()
                    .map_or(J::Null, |hash| J::string(hash.to_artifact_hash_string())),
            );
            let wire_hex = d
                .payload_bytes()
                .iter()
                .map(|byte| format!("{byte:02x}"))
                .collect::<String>();
            let family = "mizar-proof/source-algorithm-state-witness-file";
            let strings = [
                ("schema_version", "1.0".to_owned()),
                ("schema_family", family.to_owned()),
                ("obligation_id", d.obligation_id().to_owned()),
                (
                    "obligation_fingerprint",
                    d.obligation_fingerprint().to_artifact_hash_string(),
                ),
                (
                    "payload_schema_family",
                    d.payload_schema().family().to_owned(),
                ),
                (
                    "payload_schema_version",
                    d.payload_schema().version().to_string(),
                ),
                ("payload_bytes_hex", wire_hex),
            ]
            .map(|(key, value)| (key, J::string(value)));
            let file = J::object(
                strings
                    .into_iter()
                    .chain([("kernel_acceptance", J::Object(metadata))]),
            )
            .unwrap();
            let domain = artifact_hash_domain(family, V::new(1, 0));
            let digest = domain.hash(&file, &[]);
            let hash = Ref::new(Class::Artifact, family, V::new(1, 0), digest);
            let reference = ProofWitnessRef {
                schema_version: V::new(3, 0),
                obligation_id: d.obligation_id().into(),
                obligation_fingerprint: d.obligation_fingerprint().clone(),
                proof_status: ProofStatus::DischargedBuiltin,
                evidence_kind: EvidenceKind::FormulaSubstitutionKernelEvidence,
                witness_path: path.into(),
                witness_artifact_hash: hash.clone(),
                kernel_acceptance: m.clone(),
            };
            let encoded = builtin_proof_witness_ref_json(&reference).unwrap();
            let options = ProofWitnessReadOptions {
                expected_witness_artifact_hash: Some(&hash),
                ..ProofWitnessReadOptions::default()
            };
            assert_eq!(
                read_builtin_proof_witness_ref(&encoded, options).unwrap(),
                reference
            );
            let staged = stage(d.clone()).unwrap();
            let generic_value = d.payload_hash_value();
            let J::Object(keys) = &generic_value else {
                panic!("generic object")
            };
            assert_eq!(keys.len(), 7);
            assert_eq!(
                staged.staged_payload_hash().digest,
                artifact_hash_domain(d.payload_schema().family(), d.payload_schema().version())
                    .hash(&generic_value, &[])
            );
            assert_ne!(staged.staged_payload_hash(), &hash);
            assert!(staged.artifact_reference_candidate().is_none());
            assert_eq!(
                baseline.0.artifact_publication().gap(),
                Some(ArtifactPublicationGap::DischargedBuiltinWitnessUnsupported)
            );
            assert!(!reuse.cache_reuse_predicate_complete());
            assert!(reuse.selected_proof_witness_hash().is_none());
            assert!(baseline.0.trusted_used_axioms().is_none());
            assert!(d.provenance().trusted_used_axioms_ref().is_none());
            assert!(d.provenance().advisory_backend_ref().is_none());
            let published_path = PublishedArtifactPath::new(path).unwrap();
            let bytes = canonical_json_bytes(&file);
            assert_eq!(bytes.last(), Some(&b'\n'));
            let written =
                write_published_artifact(&transport, &published_path, &file, &domain, &[]).unwrap();
            assert_eq!(written.artifact_hash, hash.digest);
            let read = read_published_artifact(
                &transport,
                &published_path,
                PublishedArtifactReadOptions {
                    artifact_hash_domain: Some(&domain),
                    expected_artifact_hash: Some(hash.digest),
                    ..PublishedArtifactReadOptions::default()
                },
            )
            .unwrap();
            assert_eq!(read.value, file);
            assert_eq!(read.artifact_hash, Some(hash.digest));
            assert_eq!(fs::read(transport.join(path)).unwrap(), bytes);
            let refusal = recover(&read, &reference).unwrap_err();
            assert!(refusal.starts_with("incompatible schema family for"));
            let refused = d.clone().stage_source_theorem_file(&read, &reference);
            assert_eq!(refused.unwrap_err(), refusal);
            let actual = candidate(s, c, e, b, &p, path).unwrap();
            let expected = Some((baseline.clone(), file.clone(), reference.clone()));
            assert_eq!(actual, expected);
            assert_eq!(candidate(s, c, e, b, &p, path).unwrap(), actual);
            assert_eq!(format!("{s:?}{c:?}{e:?}{b:?}{p:?}"), before);
        });
    }
    fs::remove_dir_all(transport).unwrap();
}

#[test]
fn source_algorithm_state_witness_file_preserves_absence_and_errors() {
    use super::with_source_algorithm_state_draft_inputs as with_inputs;
    use mizar_checker::type_checker::check_source_algorithm_types as check_source;
    use mizar_kernel::clause::VariableId;
    use mizar_proof::{
        policy::VerifierPolicy,
        status::{
            source_algorithm_state_transitivity_witness_draft as explicit,
            try_source_algorithm_state_transitivity_witness_draft as acquire,
            try_source_algorithm_state_transitivity_witness_file_candidate as candidate,
        },
    };
    let original = fs::read_to_string(config().workspace_root.join(
        "tests/miz/pass/algorithms/pass_proof_verification_algorithm_var_const_assert_001.miz",
    ))
    .unwrap();
    let flat = original.replacen("    assert x = a;\n", "", 1);
    let p = VerifierPolicy::default().with_instance_search_limits(Some([0, 0, 3, 0]));
    let disabled = p.clone().with_kernel_evidence_formats([]);
    let foreign = with_inputs(
        &flat.replace("swapalgo", "foreign_algorithm"),
        &p,
        |_, _, e, _| e.clone(),
    );
    for (text, identity_error) in [(original, false), (flat.clone(), true)] {
        with_inputs(&text, &p, |s, c, e, b| {
            let e = if identity_error { &foreign } else { e };
            let before = format!("{s:?}{c:?}{e:?}{b:?}{p:?}");
            let expected = if identity_error {
                check_source(s, c.typed(), e).unwrap_err()
            } else {
                explicit(s, c, e, b, VariableId(99), &disabled, "../invalid").unwrap_err()
            };
            for limits in [None, Some([0; 4])] {
                let bounded = disabled.clone().with_instance_search_limits(limits);
                assert_eq!(
                    acquire(s, c, e, b, &bounded, "../invalid").unwrap_err(),
                    expected
                );
                assert_eq!(
                    candidate(s, c, e, b, &bounded, "../invalid").unwrap_err(),
                    expected
                );
            }
            assert_eq!(format!("{s:?}{c:?}{e:?}{b:?}{p:?}"), before);
        });
    }
    let no_chain = flat.replace("    x := c;", "    x := a;");
    for (text, miss) in [(&flat, false), (&no_chain, true)] {
        with_inputs(text, &p, |s, c, e, b| {
            let before = format!("{s:?}{c:?}{e:?}{b:?}{p:?}");
            for bound in [None, Some(0), Some(2)] {
                let bounded = disabled
                    .clone()
                    .with_instance_search_limits(bound.map(|bound| [0, 0, bound, 0]));
                assert_eq!(acquire(s, c, e, b, &bounded, "../invalid").unwrap(), None);
                assert_eq!(candidate(s, c, e, b, &bounded, "../invalid").unwrap(), None);
            }
            let max = p
                .clone()
                .with_instance_search_limits(Some([0, 0, u64::MAX, 0]));
            if miss {
                let denied = max.with_kernel_evidence_formats([]);
                assert_eq!(acquire(s, c, e, b, &denied, "../invalid").unwrap(), None);
                assert_eq!(candidate(s, c, e, b, &denied, "../invalid").unwrap(), None);
            } else {
                let selected = acquire(s, c, e, b, &max, "proof-witnesses/max.json").unwrap();
                assert!(selected.is_some());
                let produced = candidate(s, c, e, b, &max, "proof-witnesses/max.json")
                    .unwrap()
                    .map(|(triple, _, _)| triple);
                assert_eq!(produced, selected);
                let scheduling =
                    explicit(s, c, e, b, VariableId(2), &disabled, "../invalid").unwrap_err();
                assert_eq!(scheduling, "algorithm.proof.kernel_policy_rejected");
                assert_eq!(
                    acquire(s, c, e, b, &disabled, "../invalid").unwrap_err(),
                    scheduling
                );
                assert_eq!(
                    candidate(s, c, e, b, &disabled, "../invalid").unwrap_err(),
                    scheduling
                );
                let path = explicit(s, c, e, b, VariableId(2), &p, "../invalid").unwrap_err();
                assert!(path.starts_with("invalid proof witness path:"));
                assert_eq!(acquire(s, c, e, b, &p, "../invalid").unwrap_err(), path);
                assert_eq!(candidate(s, c, e, b, &p, "../invalid").unwrap_err(), path);
            }
            assert_eq!(format!("{s:?}{c:?}{e:?}{b:?}{p:?}"), before);
        });
    }
}

#[test]
fn source_algorithm_state_witness_file_reader_matches_genuine_transport() {
    use super::with_source_algorithm_state_draft_inputs as with_inputs;
    use mizar_artifact::{
        proof_witness::{ProofWitnessReadOptions as R, read_builtin_proof_witness_ref as decode},
        store::{
            CanonicalJson as J, PublishedArtifactPath as Path, PublishedArtifactReadOptions as Opt,
            SchemaVersion as V, artifact_hash_domain, canonical_json_bytes,
            read_published_artifact as read, write_published_artifact as write,
        },
    };
    use mizar_proof::{
        policy::VerifierPolicy,
        status::try_source_algorithm_state_transitivity_witness_file_candidate as file,
        witness_store::read_source_algorithm_state_witness_file as recover,
    };
    let text = include_str!("../../../../../../tests/miz/pass/algorithms/pass_proof_verification_algorithm_var_const_assert_001.miz")
        .replacen("    assert x = a;\n", "", 1);
    let temporary = std::process::Command::new("mktemp")
        .arg("-d")
        .output()
        .unwrap();
    assert!(temporary.status.success());
    let root = std::path::PathBuf::from(String::from_utf8(temporary.stdout).unwrap().trim());
    let family = "mizar-proof/source-algorithm-state-witness-file";
    let domain = artifact_hash_domain(family, V::new(1, 0));
    for (case, text) in [text.clone(), text.replace("swapalgo", "copyalgo")]
        .into_iter()
        .enumerate()
    {
        let p = VerifierPolicy::default()
            .with_profile_id(format!("reader-{case}"))
            .with_instance_search_limits(Some([0, 0, 3, 0]));
        with_inputs(&text, &p, |s, c, e, b| {
            let path = Path::new(format!("proof-witnesses/reader-{case}.json")).unwrap();
            let got = file(s, c, e, b, &p, path.as_str()).unwrap().unwrap();
            let hash = &got.2.witness_artifact_hash;
            let written = write(&root, &path, &got.1, &domain, &[]).unwrap();
            let read_options = Opt {
                artifact_hash_domain: Some(&domain),
                expected_artifact_hash: Some(hash.digest),
                ..Opt::default()
            };
            let received = read(&root, &path, read_options).unwrap();
            assert_eq!(received.value, got.1);
            assert_eq!(received.artifact_hash, Some(written.artifact_hash));
            assert_eq!(written.artifact_hash, domain.hash(&got.1, &[]));
            let bytes = canonical_json_bytes(&got.1);
            assert_eq!(bytes.last(), Some(&b'\n'));
            assert_eq!(fs::read(root.join(path.as_str())).unwrap(), bytes);
            let J::Object(fields) = &received.value else {
                panic!("genuine FILE")
            };
            let mut reference_fields = std::collections::BTreeMap::from([
                ("schema_version".into(), J::string("3.0")),
                ("proof_status".into(), J::string("discharged_builtin")),
                (
                    "evidence_kind".into(),
                    J::string("formula_substitution_kernel_evidence"),
                ),
                ("witness_path".into(), J::string(received.path.as_str())),
                (
                    "witness_artifact_hash".into(),
                    J::string(hash.to_artifact_hash_string()),
                ),
            ]);
            for key in [
                "obligation_id",
                "obligation_fingerprint",
                "kernel_acceptance",
            ] {
                reference_fields.insert(key.into(), fields[key].clone());
            }
            let reference_json = J::Object(reference_fields);
            let options = R {
                artifact_path: Some(path.as_str()),
                expected_witness_artifact_hash: Some(hash),
            };
            assert_eq!(decode(&reference_json, options).unwrap(), got.2);
            assert!(!got.0.2.payload_bytes().is_empty());
            let inputs = (received.clone(), got.2.clone());
            let recovered = recover(&received, &got.2).unwrap();
            assert_eq!(recovered, got.0.2.payload_bytes());
            assert_eq!(recover(&received, &got.2).unwrap(), recovered);
            assert_eq!((received, got.2), inputs);
        });
    }
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn source_algorithm_state_witness_file_reader_rejects_profiles_and_associations() {
    use super::with_source_algorithm_state_draft_inputs as with_inputs;
    use mizar_artifact::{
        proof_witness::{
            EvidenceKind, ProofStatus, ProofWitnessReadOptions as R,
            builtin_proof_witness_ref_json as encode, read_builtin_proof_witness_ref as decode,
        },
        registration_summary::ArtifactHashClass as Class,
        store::{
            CanonicalJson as J, MinorVersionPolicy, PublishedArtifactPath as Path,
            PublishedArtifactReadOptions as Opt, SchemaVersion as V, SchemaVersionSupport,
            artifact_hash_domain, read_published_artifact as read,
            write_published_artifact as write,
        },
    };
    use mizar_proof::{
        policy::VerifierPolicy,
        status::try_source_algorithm_state_transitivity_witness_file_candidate as file,
        witness_store::read_source_algorithm_state_witness_file as recover,
    };
    let text = include_str!("../../../../../../tests/miz/pass/algorithms/pass_proof_verification_algorithm_var_const_assert_001.miz")
        .replacen("    assert x = a;\n", "", 1);
    let p = VerifierPolicy::default().with_instance_search_limits(Some([0, 0, 3, 0]));
    let path = Path::new("proof-witnesses/reader-errors.json").unwrap();
    let got = with_inputs(&text, &p, |s, c, e, b| {
        file(s, c, e, b, &p, path.as_str()).unwrap().unwrap()
    });
    let temporary = std::process::Command::new("mktemp")
        .arg("-d")
        .output()
        .unwrap();
    assert!(temporary.status.success());
    let root = std::path::PathBuf::from(String::from_utf8(temporary.stdout).unwrap().trim());
    let family = "mizar-proof/source-algorithm-state-witness-file";
    let domain = artifact_hash_domain(family, V::new(1, 0));
    let options = Opt {
        artifact_hash_domain: Some(&domain),
        ..Opt::default()
    };
    let transport = |path: &Path, value: &J| {
        write(&root, path, value, &domain, &[]).unwrap();
        read(&root, path, options).unwrap()
    };
    let support = SchemaVersionSupport::new(family, 1, 0, MinorVersionPolicy::UpToSupported);
    let payload = "payload_bytes_hex";
    let acceptance = "kernel_acceptance";
    let prefix = "source algorithm state witness file";
    let invalid = format!("invalid {prefix}");
    let J::Object(fields) = &got.1 else {
        panic!("genuine FILE")
    };
    let mut cases = vec![(J::Null, format!("{prefix} must be an object"))];
    for version in ["missing", "null", "bad", "0.0", "2.0", "1.1"] {
        let header = match version {
            "missing" => None,
            "null" => Some(""),
            _ => Some(version),
        };
        let mut value = fields.clone();
        let error = support
            .check_at_path(header, path.as_str())
            .unwrap_err()
            .to_string();
        let replacement = header
            .filter(|text| !text.is_empty())
            .map_or(J::Null, J::string);
        value.insert("schema_version".into(), replacement);
        if header.is_none() {
            value.remove("schema_version");
        }
        value.insert(payload.into(), J::Null);
        value.insert(acceptance.into(), J::Null);
        cases.push((J::Object(value), error));
    }
    let incompatible = format!(
        "incompatible schema family for {family}: actual version 1.0, supported 1.0..=1.0, artifact path {}",
        path.as_str()
    );
    let profile_error = format!("{invalid} payload profile");
    for (key, replacement, error) in [
        ("schema_family", "foreign/file", incompatible),
        ("unknown", "", format!("{invalid} fields")),
        (
            "payload_schema_family",
            "mizar-proof/source-theorem-reflexivity-checked-wire",
            profile_error.clone(),
        ),
        ("payload_schema_version", "1.0", profile_error),
    ] {
        let mut value = fields.clone();
        value.insert(key.into(), J::string(replacement));
        if key == "schema_family" {
            value.insert(acceptance.into(), J::Null);
        }
        cases.push((J::Object(value), error));
    }
    for key in ["obligation_id", acceptance] {
        let mut value = fields.clone();
        value.remove(key);
        cases.push((J::Object(value), format!("{invalid} fields")));
        value = fields.clone();
        value.insert(key.into(), J::Null);
        let suffix = if key == acceptance {
            "an object"
        } else {
            "a string"
        };
        cases.push((J::Object(value), format!("{prefix} {key} must be {suffix}")));
    }
    for hex in ["", "0", "AA", "0g", "00 0", "é", "0é0", "００"] {
        let mut value = fields.clone();
        value.insert(payload.into(), J::string(hex));
        cases.push((J::Object(value), format!("{invalid} payload hex")));
    }
    let J::Object(metadata) = &fields[acceptance] else {
        panic!("actual metadata")
    };
    for (key, replacement) in [
        ("formula_context_hash", None),
        ("accepted_result_hash", Some(J::string("bad-hash"))),
    ] {
        let mut nested = metadata.clone();
        if let Some(replacement) = replacement {
            nested.insert(key.into(), replacement);
        } else {
            nested.remove(key);
        }
        let mut value = fields.clone();
        value.insert(acceptance.into(), J::Object(nested.clone()));
        let J::Object(mut lower) = encode(&got.2).unwrap() else {
            unreachable!()
        };
        lower.insert(acceptance.into(), J::Object(nested));
        let error = decode(&J::Object(lower), R::default())
            .unwrap_err()
            .to_string();
        cases.push((J::Object(value), error));
    }
    for (value, error) in cases {
        let received = transport(&path, &value);
        let mut expected = got.2.clone();
        expected.witness_artifact_hash.digest = received.artifact_hash.unwrap();
        assert_eq!(received.value, value, "{error}; {expected:?}");
        assert_eq!(recover(&received, &expected).unwrap_err(), error);
    }
    let received = transport(&path, &got.1);
    let hash_error = format!("{prefix} read hash mismatch");
    for options in [
        Opt::default(),
        Opt {
            artifact_hash_domain: Some(&artifact_hash_domain("foreign/file", V::new(1, 0))),
            ..Opt::default()
        },
    ] {
        let wrong = read(&root, &path, options).unwrap();
        assert_ne!(wrong.artifact_hash, received.artifact_hash);
        assert_eq!(recover(&wrong, &got.2).unwrap_err(), hash_error);
    }
    let mut stale = received.clone();
    let J::Object(ref mut value) = stale.value else {
        unreachable!()
    };
    value.insert(payload.into(), J::string("00"));
    assert_ne!(domain.hash(&stale.value, &[]), stale.artifact_hash.unwrap());
    assert_eq!(recover(&stale, &got.2).unwrap_err(), hash_error);
    for name in ["proof-witnesses/foreign-path.json", "other/checked.json"] {
        let other = Path::new(name).unwrap();
        let moved = transport(&other, &got.1);
        let mut lower = encode(&got.2).unwrap();
        let J::Object(ref mut values) = lower else {
            unreachable!()
        };
        values.insert("witness_path".into(), J::string(moved.path.as_str()));
        let error = match decode(&lower, R::default()) {
            Ok(decoded) => {
                assert_ne!(decoded, got.2);
                format!("{prefix} reference mismatch")
            }
            Err(error) => error.to_string(),
        };
        assert_ne!(moved.path, path, "{error}");
        assert_eq!(recover(&moved, &got.2).unwrap_err(), error);
    }
    let foreign_hash = mizar_session::hash_text("foreign");
    for index in 1..13 {
        let mut expected = got.2.clone();
        let hash = &mut expected.witness_artifact_hash;
        let metadata = &mut expected.kernel_acceptance;
        match index {
            1 => expected.schema_version = V::new(2, 0),
            2 => expected.obligation_id.push_str("-other"),
            3 => expected.obligation_fingerprint.digest = foreign_hash,
            4 => expected.proof_status = ProofStatus::KernelVerified,
            5 => expected.evidence_kind = EvidenceKind::BuiltinCertificate,
            6 => expected.witness_path = "proof-witnesses/other.json".into(),
            7 => hash.digest = foreign_hash,
            8 => hash.class = Class::Interface,
            9 => hash.schema_family = "foreign/file".into(),
            10 => metadata.verifier_policy_fingerprint.digest = foreign_hash,
            11 => metadata.formula_context_hash = Some(expected.obligation_fingerprint.clone()),
            12 => hash.schema_version = V::new(2, 0),
            _ => unreachable!(),
        }
        let lower = decode(
            &encode(&got.2).unwrap(),
            R {
                expected_witness_artifact_hash: Some(&expected.witness_artifact_hash),
                ..R::default()
            },
        );
        let error = match lower {
            Ok(decoded) => {
                assert_ne!(decoded, expected);
                format!("{prefix} reference mismatch")
            }
            Err(error) => error.to_string(),
        };
        assert_ne!(expected, got.2, "{error}");
        assert_eq!(recover(&received, &expected).unwrap_err(), error);
    }
    let mismatch = format!("{prefix} reference mismatch");
    for kind in ["family", "version", "evidence", "wire"] {
        let mut value = fields.clone();
        let mut expected = got.2.clone();
        match kind {
            "family" => expected.obligation_fingerprint.schema_family = "foreign/original".into(),
            "version" => expected.obligation_fingerprint.schema_version = V::new(2, 0),
            "evidence" => expected.kernel_acceptance.evidence_schema_version = V::new(1, 0),
            _ => {}
        }
        let J::Object(reference) = encode(&expected).unwrap() else {
            unreachable!()
        };
        for key in ["obligation_fingerprint", acceptance] {
            value.insert(key.into(), reference[key].clone());
        }
        if kind == "wire" {
            value.insert(payload.into(), J::string("00"));
        }
        let value = J::Object(value);
        let edited = transport(&path, &value);
        expected.witness_artifact_hash.digest = edited.artifact_hash.unwrap();
        assert_eq!(edited.value, value);
        assert_eq!(
            decode(&encode(&expected).unwrap(), R::default()).unwrap(),
            expected
        );
        if kind == "wire" {
            assert_ne!(got.0.2.payload_bytes(), [0]);
            assert_eq!(recover(&edited, &expected).unwrap(), [0]);
        }
        if kind != "wire" {
            assert_eq!(recover(&edited, &expected).unwrap_err(), mismatch);
        }
    }
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn source_algorithm_state_witness_file_staging_matches_genuine_association() {
    use super::with_source_algorithm_state_draft_inputs as with_inputs;
    use mizar_artifact::store::{
        PublishedArtifactPath as Path, PublishedArtifactReadOptions as Opt, SchemaVersion as V,
        artifact_hash_domain, read_published_artifact as read, write_published_artifact as write,
    };
    use mizar_proof::{
        policy::VerifierPolicy,
        status::try_source_algorithm_state_transitivity_witness_file_candidate as file,
        witness_store::{read_source_algorithm_state_witness_file as recover, stage},
    };
    let text = include_str!("../../../../../../tests/miz/pass/algorithms/pass_proof_verification_algorithm_var_const_assert_001.miz")
        .replacen("    assert x = a;\n", "", 1);
    let temporary = std::process::Command::new("mktemp")
        .arg("-d")
        .output()
        .unwrap();
    assert!(temporary.status.success());
    let root = std::path::PathBuf::from(String::from_utf8(temporary.stdout).unwrap().trim());
    let family = "mizar-proof/source-algorithm-state-witness-file";
    let domain = artifact_hash_domain(family, V::new(1, 0));
    for (case, text) in [text.clone(), text.replace("swapalgo", "copyalgo")]
        .into_iter()
        .enumerate()
    {
        let p = VerifierPolicy::default()
            .with_profile_id(format!("staging-{case}"))
            .with_instance_search_limits(Some([0, 0, 3, 0]));
        with_inputs(&text, &p, |s, c, e, b| {
            let path = Path::new(format!("proof-witnesses/staging-{case}.json")).unwrap();
            let got = file(s, c, e, b, &p, path.as_str()).unwrap().unwrap();
            let d = &got.0.2;
            let written = write(&root, &path, &got.1, &domain, &[]).unwrap();
            let options = Opt {
                artifact_hash_domain: Some(&domain),
                expected_artifact_hash: Some(got.2.witness_artifact_hash.digest),
                ..Opt::default()
            };
            let received = read(&root, &path, options).unwrap();
            assert_eq!(received.value, got.1);
            assert_eq!(received.artifact_hash, Some(written.artifact_hash));
            assert_eq!(recover(&received, &got.2).unwrap(), d.payload_bytes());
            let legacy = stage(d.clone()).unwrap();
            assert_eq!(legacy.obligation_id(), d.obligation_id());
            assert_eq!(legacy.obligation_anchor(), d.obligation_anchor());
            assert_eq!(legacy.obligation_fingerprint(), d.obligation_fingerprint());
            assert_eq!(legacy.selected_class(), d.selected_class());
            assert_eq!(legacy.selected_evidence_hash(), d.selected_evidence_hash());
            assert_eq!(legacy.witness_path(), d.witness_path());
            assert_eq!(legacy.provenance(), d.provenance());
            assert_eq!(legacy.payload_schema(), d.payload_schema());
            assert_ne!(legacy.staged_payload_hash(), &got.2.witness_artifact_hash);
            assert!(legacy.artifact_reference_candidate().is_none());
            let before = (got.clone(), received.clone());
            let staged = d
                .clone()
                .stage_source_algorithm_state_file(&received, &got.2)
                .unwrap();
            assert_eq!(staged.obligation_id(), legacy.obligation_id());
            assert_eq!(staged.obligation_anchor(), legacy.obligation_anchor());
            assert_eq!(
                staged.obligation_fingerprint(),
                legacy.obligation_fingerprint()
            );
            assert_eq!(staged.selected_class(), legacy.selected_class());
            assert_eq!(
                staged.selected_evidence_hash(),
                legacy.selected_evidence_hash()
            );
            assert_eq!(staged.witness_path(), legacy.witness_path());
            assert_eq!(staged.provenance(), legacy.provenance());
            assert_eq!(staged.payload_schema().family(), family);
            assert_eq!(staged.payload_schema().version(), V::new(1, 0));
            assert!(staged.payload_schema().canonical_bytes_required());
            assert_eq!(staged.staged_payload_hash(), &got.2.witness_artifact_hash);
            assert_eq!(staged.artifact_reference_candidate(), Some(&got.2));
            let again = d
                .clone()
                .stage_source_algorithm_state_file(&received, &got.2)
                .unwrap();
            assert_eq!(staged, again);
            assert_eq!((got, received), before);
        });
    }
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn source_algorithm_state_witness_file_staging_preserves_reader_errors_and_rejects_mismatch() {
    use super::{
        with_source_algorithm_state_draft_inputs as with_inputs,
        with_source_theorem_file_staging_inputs as with_theorem,
    };
    use mizar_artifact::{
        proof_witness::builtin_proof_witness_ref_json as encode,
        store::{
            CanonicalJson as J, PublishedArtifactPath as Path, PublishedArtifactReadOptions as Opt,
            SchemaVersion as V, artifact_hash_domain, read_published_artifact as read,
            write_published_artifact as write,
        },
    };
    use mizar_proof::{
        policy::VerifierPolicy,
        status::try_source_algorithm_state_transitivity_witness_file_candidate as file,
        witness_store::{
            read_source_algorithm_state_witness_file as recover,
            read_source_theorem_witness_file as theorem,
        },
    };
    let text = include_str!("../../../../../../tests/miz/pass/algorithms/pass_proof_verification_algorithm_var_const_assert_001.miz")
        .replacen("    assert x = a;\n", "", 1);
    let renamed = text
        .replace("swapalgo", "foreign_algorithm")
        .replace("    x := c;", "    const d := a;\n    x := c;");
    let p = VerifierPolicy::default().with_instance_search_limits(Some([0, 0, 3, 0]));
    let changed = p.clone().with_profile_id("foreign-staging-policy");
    let mut products = Vec::new();
    for (text, policy, path) in [
        (&text, &p, "proof-witnesses/base.json"),
        (&renamed, &p, "proof-witnesses/base.json"),
        (&text, &changed, "proof-witnesses/base.json"),
        (&text, &p, "proof-witnesses/foreign.json"),
    ] {
        products.push(with_inputs(text, policy, |s, c, e, b| {
            file(s, c, e, b, policy, path).unwrap().unwrap()
        }));
    }
    let mut foreign = None;
    with_theorem(|index, got, _, _| {
        if index == 0 {
            foreign = Some(got.0.2.clone());
        }
        Ok(())
    })
    .unwrap();
    let foreign = foreign.unwrap();
    let temporary = std::process::Command::new("mktemp")
        .arg("-d")
        .output()
        .unwrap();
    assert!(temporary.status.success());
    let root = std::path::PathBuf::from(String::from_utf8(temporary.stdout).unwrap().trim());
    let family = "mizar-proof/source-algorithm-state-witness-file";
    let domain = artifact_hash_domain(family, V::new(1, 0));
    let options = Opt {
        artifact_hash_domain: Some(&domain),
        ..Opt::default()
    };
    let transport = |path: &Path, value: &J| {
        write(&root, path, value, &domain, &[]).unwrap();
        read(&root, path, options).unwrap()
    };
    let reads = products
        .iter()
        .map(|got| {
            let path = Path::new(got.2.witness_path.clone()).unwrap();
            let read = transport(&path, &got.1);
            assert_eq!(recover(&read, &got.2).unwrap(), got.0.2.payload_bytes());
            read
        })
        .collect::<Vec<_>>();
    let got = &products[0];
    let d = &got.0.2;
    let received = &reads[0];
    let path = Path::new(got.2.witness_path.clone()).unwrap();
    let mismatch = "source algorithm state witness file staging mismatch";
    for (other, read) in products.iter().zip(&reads).skip(1) {
        let od = &other.0.2;
        assert!(
            d.obligation_id() != od.obligation_id()
                || d.obligation_fingerprint() != od.obligation_fingerprint()
                || d.witness_path() != od.witness_path()
                || d.kernel_acceptance() != od.kernel_acceptance()
                || d.payload_bytes() != od.payload_bytes(),
            "actual foreign read: {read:?}"
        );
        assert_eq!(
            d.clone()
                .stage_source_algorithm_state_file(read, &other.2)
                .unwrap_err(),
            mismatch
        );
        assert_eq!(
            od.clone()
                .stage_source_algorithm_state_file(received, &got.2)
                .unwrap_err(),
            mismatch
        );
    }
    assert_eq!(foreign.selected_class(), d.selected_class());
    assert_ne!(foreign.payload_schema(), d.payload_schema());
    assert_eq!(
        foreign
            .clone()
            .stage_source_algorithm_state_file(received, &got.2)
            .unwrap_err(),
        mismatch
    );
    let theorem_error = theorem(received, &got.2).unwrap_err();
    assert_eq!(
        foreign
            .clone()
            .stage_source_theorem_file(received, &got.2)
            .unwrap_err(),
        theorem_error
    );
    let mut missing_hash = received.clone();
    missing_hash.artifact_hash = None;
    let mut incompatible = got.1.clone();
    let J::Object(ref mut fields) = incompatible else {
        unreachable!()
    };
    fields.insert("schema_version".into(), J::string("2.0"));
    fields.insert("payload_bytes_hex".into(), J::Null);
    let incompatible = transport(&path, &incompatible);
    let mut wrong_reference = got.2.clone();
    wrong_reference.witness_path = "proof-witnesses/wrong.json".into();
    for (read, reference) in [
        (missing_hash, got.2.clone()),
        (incompatible, got.2.clone()),
        (received.clone(), wrong_reference),
    ] {
        let error = recover(&read, &reference).unwrap_err();
        assert_ne!(error, mismatch);
        assert_eq!(
            foreign
                .clone()
                .stage_source_algorithm_state_file(&read, &reference)
                .unwrap_err(),
            error
        );
    }
    let J::Object(fields) = &got.1 else {
        unreachable!()
    };
    let hash = mizar_session::hash_text("untrusted-state-association-edit");
    for kind in ["wire", "id", "fingerprint", "metadata", "path"] {
        let mut value = fields.clone();
        let mut expected = got.2.clone();
        match kind {
            "id" => expected.obligation_id.push_str("-foreign"),
            "fingerprint" => expected.obligation_fingerprint.digest = hash,
            "path" => expected.witness_path = "proof-witnesses/coherent-path.json".into(),
            "metadata" => {
                expected
                    .kernel_acceptance
                    .verifier_policy_fingerprint
                    .digest = hash
            }
            _ => {}
        }
        let J::Object(reference) = encode(&expected).unwrap() else {
            unreachable!()
        };
        for key in [
            "obligation_id",
            "obligation_fingerprint",
            "kernel_acceptance",
        ] {
            value.insert(key.into(), reference[key].clone());
        }
        if kind == "wire" {
            value.insert("payload_bytes_hex".into(), J::string("00"));
        }
        let edited_path = Path::new(expected.witness_path.clone()).unwrap();
        let edited = transport(&edited_path, &J::Object(value));
        expected.witness_artifact_hash.digest = edited.artifact_hash.unwrap();
        let recovered = recover(&edited, &expected).unwrap();
        if kind == "wire" {
            assert_eq!(recovered, [0]);
        } else {
            assert_eq!(recovered, d.payload_bytes());
        }
        assert!(
            recovered != d.payload_bytes()
                || expected.obligation_id != d.obligation_id()
                || expected.witness_path != d.witness_path()
                || &expected.obligation_fingerprint != d.obligation_fingerprint()
                || &expected.kernel_acceptance != d.kernel_acceptance()
        );
        assert_eq!(
            d.clone()
                .stage_source_algorithm_state_file(&edited, &expected)
                .unwrap_err(),
            mismatch
        );
    }
    fs::remove_dir_all(root).unwrap();
}
