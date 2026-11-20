use super::*;

#[test]
fn source_algorithm_return_canonical_components_preserve_genuine_content() {
    use mizar_checker::type_checker::check_source_algorithm_types;
    use mizar_core::elaborator::{lower_source_algorithms, source_algorithm_formula_payloads};
    use mizar_session::Hash;
    use mizar_vc::generator::generate_source_algorithm_postconditions;
    use mizar_vc::vc_ir::{
        AnchorCompleteness, AnchorIngredient, ContextEntryKind, HashMarker, PremiseRef,
        SeedNoVcReason, SeedVcMapping, VcFormulaRef, VcKind, VcStatus,
    };

    // This is the pre-existing fingerprint primitive, independent of the new method.
    let old_hash = |domain: &str, bytes: &[u8]| {
        let mut lanes = [
            0x6d_69_7a_61_72_2d_76_63_u64,
            0x70_72_6f_6f_66_2d_69_64_u64,
            0x74_61_73_6b_32_30_2d_76_u64,
            0x66_69_6e_67_65_72_2d_31_u64,
        ];
        for (index, byte) in domain
            .as_bytes()
            .iter()
            .copied()
            .chain([0])
            .chain(bytes.iter().copied())
            .enumerate()
        {
            let lane = index % 4;
            lanes[lane] ^= u64::from(byte)
                .wrapping_add(0x9e37_79b9_7f4a_7c15)
                .wrapping_add((index as u64).rotate_left((lane as u32) + 1));
            lanes[lane] = lanes[lane]
                .rotate_left(11 + lane as u32)
                .wrapping_mul(0x1000_0000_01b3);
        }
        lanes[0] ^= bytes.len() as u64;
        lanes[1] ^= (domain.len() as u64).rotate_left(17);
        lanes[2] ^= lanes[0].rotate_left(7);
        lanes[3] ^= lanes[1].rotate_left(13);
        let mut output = [0; Hash::BYTE_LEN];
        for (chunk, lane) in output.as_chunks_mut::<8>().0.iter_mut().zip(lanes) {
            chunk.copy_from_slice(&lane.to_be_bytes());
        }
        Hash::from_bytes(output)
    };
    let config = config();
    let plan = crate::harness::build_test_plan(&config).unwrap();
    let case = plan
        .cases
        .iter()
        .find(|case| case.id.0 == "pass_proof_verification_algorithm_ensures_return_001")
        .unwrap();
    let original = fs::read_to_string(&case.source_path).unwrap();
    let mut baseline: Option<(
        (
            mizar_vc::vc_ir::CanonicalVcFingerprint,
            Hash,
            mizar_vc::vc_ir::LocalContextFingerprint,
        ),
        Vec<u8>,
    )> = None;
    for (edit, text) in [
        original.clone(),
        original
            .replace(" a ", " param ")
            .replace("(a)", "(param)")
            .replace(" a;", " param;")
            .replace("= a\n", "= param\n"),
        format!(":: neutral algorithm trivia\n\n{original}"),
        original.replace("idalgo", "renamed_algorithm"),
    ]
    .into_iter()
    .enumerate()
    {
        let (source, typed, symbols) = super::super::super::source_registration_inputs(
            &config.workspace_root,
            case,
            super::super::super::formula_statement::step5c8_test_frontend(&text),
        )
        .unwrap();
        let check = check_source_algorithm_types(&source, &typed, &symbols).unwrap();
        let core = lower_source_algorithms(&check).unwrap();
        let algorithm = &core.algorithms().iter().next().unwrap().1;
        assert_eq!(algorithm.statements.len(), 1);
        assert_eq!(algorithm.contracts.ensures.len(), 1);
        let guard = algorithm.params[0].ty_guard.unwrap();
        let formulas = source_algorithm_formula_payloads(&check, &core, &[guard]).unwrap();
        assert_eq!(formulas.len(), 1);
        let before = (core.clone(), formulas.clone());
        for (schema_index, (vc_schema, generation_schema, literal_goal)) in [
            (
                "mizar-vc-vcset-step5c14-return-v1",
                "mizar-vc-generation-step5c14-return-v1",
                "0a4a4e2f40a14a6fb930893deec52bfa09233a485303d7e56a2bb866f8290045",
            ),
            (
                "mizar-vc-vcset-step5c14-return-v1-changed",
                "mizar-vc-generation-step5c14-return-v1",
                "8f47d1004f1326a7bb2c42c6180bde668a3027f55cf7155555e8bddc4a60aa6a",
            ),
            (
                "mizar-vc-vcset-step5c14-return-v1",
                "mizar-vc-generation-step5c14-return-v1-changed",
                "2b5298d40777953f5fc75e7cf0456cf879b50593ca97cff45f286955047d3974",
            ),
            (
                "",
                " \n世代\t\0",
                "4cf1ca2e967115d59067d075cb3327513ab8419139c3d1293426f468560c7cd7",
            ),
        ]
        .into_iter()
        .take(if edit == 0 { 4 } else { 1 })
        .enumerate()
        {
            let generation_schema = GenerationSchemaVersion::new(generation_schema);
            let snapshot = snapshot_id(usize::from(edit == 2));
            let vcs = generate_source_algorithm_postconditions(
                &core,
                snapshot,
                &generation_schema,
                &VcSchemaVersion::new(vc_schema),
            )
            .unwrap();
            assert_eq!(vcs.vcs().len(), 1);
            assert_eq!(vcs.generated_formulas().len(), 1);
            let vc = &vcs.vcs()[0];
            assert_eq!(vc.kind, VcKind::AlgorithmPostcondition);
            assert_eq!(vc.status, VcStatus::Open);
            assert_eq!(
                vc.goal,
                VcFormulaRef::Generated(vcs.generated_formulas()[0].id)
            );
            assert_eq!(
                vc.premises,
                [PremiseRef::LocalContext(vc.local_context.entries()[0].id)]
            );
            assert_eq!(
                vc.local_context.entries()[0].kind,
                ContextEntryKind::CheckerFact
            );
            assert_eq!(vc.local_context.policy_inputs().len(), 6);
            assert!(vc.proof_hint.is_none());
            assert!(vcs.canonical_vc_fingerprint(vc.id).is_none());
            if edit == 0 && schema_index == 0 {
                assert!(vcs.local_context_fingerprint(vc.id).is_none());
                assert!(matches!(
                    &vc.anchor.completeness,
                    AnchorCompleteness::Incomplete { missing }
                        if missing == &[AnchorIngredient::CanonicalGoalHash, AnchorIngredient::CanonicalContextHash]
                ));
                assert!(matches!(
                    &vc.anchor.canonical_goal_hash,
                    HashMarker::ConservativeUnknown { .. }
                ));
                assert!(matches!(
                    &vc.anchor.canonical_context_hash,
                    HashMarker::ConservativeUnknown { .. }
                ));
                let [termination, contract] = vcs.seed_accounting() else {
                    panic!("expected return accounting")
                };
                assert_eq!(
                    termination.seed_status,
                    mizar_core::core_ir::ObligationSeedStatus::Deferred
                );
                assert!(matches!(&termination.mapping, SeedVcMapping::NoConcreteVc {
                    reason: SeedNoVcReason::DeferredExternal(reason),
                } if reason.as_str() == "task 7 requires explicit goal formula for AlgorithmTermination site PartialTermination"));
                assert_eq!(
                    contract.seed_status,
                    mizar_core::core_ir::ObligationSeedStatus::Deferred
                );
                assert!(matches!(&contract.mapping, SeedVcMapping::One { vc: id } if *id == vc.id));
            }

            let mut entry = 0u32.to_be_bytes().to_vec();
            entry.push(1);
            entry.extend(u32::try_from(formulas[&guard].len()).unwrap().to_be_bytes());
            entry.extend(&formulas[&guard]);
            entry.extend(1u32.to_be_bytes());
            entry.push(1);
            entry.extend(24u32.to_be_bytes());
            entry.extend(b"algorithm-parameter-type");
            entry.extend([1, 1]);
            entry.extend(38u32.to_be_bytes());
            entry.extend(b"source-algorithm-return/parameter-type");
            let mut policy = 6u32.to_be_bytes().to_vec();
            for (index, (name, value)) in [
                ("flow", "0"),
                ("site-kind", "Ensures"),
                ("ordinal", "0"),
                ("statement", "0"),
                ("block", "0"),
                ("exit", "0"),
            ]
            .into_iter()
            .enumerate()
            {
                for field in [
                    format!("{index:04}-algorithm-site-{name}"),
                    format!("algorithm-site-{name}"),
                    value.to_owned(),
                ] {
                    policy.extend(u32::try_from(field.len()).unwrap().to_be_bytes());
                    policy.extend(field.as_bytes());
                }
            }
            let goal = [1, 2, 0, 0, 0, 0, 2, 0, 0, 0, 0];
            let mut vc_payload = b"mizar-vc/source-algorithm-return-canonical-vc/v1\0".to_vec();
            let mut goal_payload = b"mizar-vc/source-algorithm-return-canonical-goal/v1\0".to_vec();
            let mut context_payload =
                b"mizar-vc/source-algorithm-return-canonical-context/v1\0".to_vec();
            for payload in [&mut vc_payload, &mut goal_payload, &mut context_payload] {
                for field in [vc_schema, generation_schema.as_str()] {
                    payload.extend(u32::try_from(field.len()).unwrap().to_be_bytes());
                    payload.extend(field.as_bytes());
                }
            }
            goal_payload.extend(11u32.to_be_bytes());
            goal_payload.extend(goal);
            context_payload.extend(1u32.to_be_bytes());
            context_payload.extend(u32::try_from(entry.len()).unwrap().to_be_bytes());
            context_payload.extend(&entry);
            context_payload.extend(&policy);
            vc_payload.push(1);
            vc_payload.extend(11u32.to_be_bytes());
            vc_payload.extend(goal);
            vc_payload.extend(1u32.to_be_bytes());
            vc_payload.push(1);
            vc_payload.extend(u32::try_from(entry.len()).unwrap().to_be_bytes());
            vc_payload.extend(&entry);
            vc_payload.extend(&policy);
            vc_payload.push(0);
            let expected_vc =
                old_hash("mizar-vc-source-algorithm-return-canonical-vc", &vc_payload);
            let expected_goal = old_hash(
                "mizar-vc-source-algorithm-return-canonical-goal",
                &goal_payload,
            );
            let expected_context = old_hash(
                "mizar-vc-source-algorithm-return-canonical-context",
                &context_payload,
            );
            assert_eq!(
                expected_goal
                    .as_bytes()
                    .iter()
                    .map(|byte| format!("{byte:02x}"))
                    .collect::<String>(),
                literal_goal
            );
            let graph_before = vcs.clone();
            let actual = vcs
                .source_algorithm_return_canonical_components(
                    vc.id,
                    &core,
                    &generation_schema,
                    &formulas,
                )
                .unwrap();
            assert_eq!(
                (actual.0.hash(), actual.1, actual.2.hash()),
                (expected_vc, expected_goal, expected_context)
            );
            assert_eq!(
                actual,
                vcs.source_algorithm_return_canonical_components(
                    vc.id,
                    &core,
                    &generation_schema,
                    &formulas
                )
                .unwrap()
            );
            assert_eq!(vcs, graph_before);
            if schema_index == 0 {
                if let Some((old, old_guard)) = &baseline {
                    assert_eq!(actual.1, old.1);
                    if edit == 3 {
                        assert_ne!(actual.0, old.0);
                        assert_ne!(actual.2, old.2);
                        assert_ne!(&formulas[&guard], old_guard);
                    } else {
                        assert_eq!(actual, *old);
                        assert_eq!(&formulas[&guard], old_guard);
                    }
                } else {
                    baseline = Some((actual, formulas[&guard].clone()));
                }
            }
        }
        assert_eq!(before, (core.clone(), formulas.clone()));
    }
}

#[test]
fn source_algorithm_return_canonical_components_preserve_refusals() {
    use mizar_checker::type_checker::check_source_algorithm_types;
    use mizar_core::elaborator::{lower_source_algorithms, source_algorithm_formula_payloads};
    use mizar_vc::generator::generate_source_algorithm_postconditions;
    use mizar_vc::vc_ir::{GenerationSchemaVersion, VcId, VcSchemaVersion};
    let config = config();
    let plan = crate::harness::build_test_plan(&config).unwrap();
    let case = plan
        .cases
        .iter()
        .find(|case| case.id.0 == "pass_proof_verification_algorithm_ensures_return_001")
        .unwrap();
    let text = fs::read_to_string(&case.source_path).unwrap();
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
    let generation = GenerationSchemaVersion::new("mizar-vc-generation-step5c14-return-v1");
    let schema = VcSchemaVersion::new("mizar-vc-vcset-step5c14-return-v1");
    let vcs = generate_source_algorithm_postconditions(&core, snapshot_id(0), &generation, &schema)
        .unwrap();
    let invalid = "unsupported source algorithm return canonical components";
    assert_eq!(
        vcs.source_algorithm_return_canonical_components(
            VcId::new(1),
            &core,
            &generation,
            &formulas
        )
        .unwrap_err(),
        invalid
    );
    let empty = std::collections::BTreeMap::new();
    assert_eq!(
        vcs.source_algorithm_return_canonical_components(VcId::new(0), &core, &generation, &empty)
            .unwrap_err(),
        invalid
    );
    let mut empty_value = formulas.clone();
    empty_value.insert(guard, Vec::new());
    assert_eq!(
        vcs.source_algorithm_return_canonical_components(
            VcId::new(0),
            &core,
            &generation,
            &empty_value
        )
        .unwrap_err(),
        invalid
    );
    let mut extra = formulas.clone();
    extra.insert(
        core.algorithms().iter().next().unwrap().1.contracts.ensures[0],
        b"extra".to_vec(),
    );
    assert_eq!(
        vcs.source_algorithm_return_canonical_components(VcId::new(0), &core, &generation, &extra)
            .unwrap_err(),
        invalid
    );
    let mut wrong_root = empty.clone();
    wrong_root.insert(
        core.algorithms().iter().next().unwrap().1.contracts.ensures[0],
        formulas[&guard].clone(),
    );
    assert_eq!(
        vcs.source_algorithm_return_canonical_components(
            VcId::new(0),
            &core,
            &generation,
            &wrong_root
        )
        .unwrap_err(),
        invalid
    );
    let mut parts = mizar_vc::vc_ir::VcSetParts {
        schema_version: vcs.schema_version().clone(),
        snapshot: vcs.snapshot(),
        source: vcs.source(),
        module: vcs.module().clone(),
        generated_formulas: vcs.generated_formulas().to_vec(),
        vcs: vcs.vcs().to_vec(),
        seed_accounting: vcs.seed_accounting().to_vec(),
    };
    let mut entry = parts.vcs[0].local_context.entries()[0].clone();
    entry.provenance[0].key = "different".into();
    parts.vcs[0].local_context = mizar_vc::vc_ir::LocalContext::try_new(
        vec![entry],
        parts.vcs[0].local_context.policy_inputs().to_vec(),
    )
    .unwrap();
    let drift = mizar_vc::vc_ir::VcSet::try_new(parts).unwrap();
    assert_eq!(
        drift
            .source_algorithm_return_canonical_components(
                VcId::new(0),
                &core,
                &generation,
                &formulas
            )
            .unwrap_err(),
        invalid
    );
    assert_eq!(
        vcs.source_algorithm_return_canonical_components(
            VcId::new(0),
            &core,
            &GenerationSchemaVersion::new("different"),
            &formulas
        )
        .unwrap_err(),
        invalid
    );
    for (id, edit) in [
        (
            "pass_proof_verification_algorithm_ensures_return_001",
            "owner",
        ),
        (
            "pass_proof_verification_algorithm_var_const_assert_001",
            "broader",
        ),
        (
            "pass_proof_verification_algorithm_ensures_return_001",
            "zero",
        ),
    ] {
        let case = plan.cases.iter().find(|case| case.id.0 == id).unwrap();
        let original = fs::read_to_string(&case.source_path).unwrap();
        let text = match edit {
            "owner" => original.replace("idalgo", "foreign_algorithm"),
            "zero" => original.replace("    ensures result = a\n", ""),
            _ => original,
        };
        let (source, typed, symbols) = super::super::super::source_registration_inputs(
            &config.workspace_root,
            case,
            super::super::super::formula_statement::step5c8_test_frontend(&text),
        )
        .unwrap();
        let check = check_source_algorithm_types(&source, &typed, &symbols).unwrap();
        let other_core = lower_source_algorithms(&check).unwrap();
        let other_graph = generate_source_algorithm_postconditions(
            &other_core,
            snapshot_id(0),
            &generation,
            &schema,
        )
        .unwrap();
        assert_ne!(other_graph, vcs);
        if edit == "zero" {
            assert!(other_graph.vcs().is_empty());
            assert_eq!(
                other_graph
                    .source_algorithm_return_canonical_components(
                        VcId::new(0),
                        &other_core,
                        &generation,
                        &empty,
                    )
                    .unwrap_err(),
                invalid
            );
        } else {
            assert!(!other_graph.vcs().is_empty());
            let other_guard = other_core.algorithms().iter().next().unwrap().1.params[0]
                .ty_guard
                .unwrap();
            let other_formulas =
                source_algorithm_formula_payloads(&check, &other_core, &[other_guard]).unwrap();
            assert_eq!(other_formulas.len(), 1);
            assert!(!other_formulas[&other_guard].is_empty());
            if edit == "owner" {
                assert_eq!(
                    vcs.source_algorithm_return_canonical_components(
                        VcId::new(0),
                        &other_core,
                        &generation,
                        &other_formulas,
                    )
                    .unwrap_err(),
                    invalid
                );
            } else {
                assert_eq!(
                    other_graph
                        .source_algorithm_return_canonical_components(
                            VcId::new(0),
                            &other_core,
                            &generation,
                            &other_formulas,
                        )
                        .unwrap_err(),
                    invalid
                );
            }
        }
    }
    let two_returns = text
        .replace("    ensures result = a\n", "")
        .replace("return a;", "return a;\n    return a;");
    let (source, typed, symbols) = super::super::super::source_registration_inputs(
        &config.workspace_root,
        case,
        super::super::super::formula_statement::step5c8_test_frontend(&two_returns),
    )
    .unwrap();
    let check = check_source_algorithm_types(&source, &typed, &symbols).unwrap();
    let two_core = lower_source_algorithms(&check).unwrap();
    assert!(
        two_core
            .algorithms()
            .iter()
            .next()
            .unwrap()
            .1
            .contracts
            .ensures
            .is_empty()
    );
    assert_eq!(
        two_core
            .algorithms()
            .iter()
            .next()
            .unwrap()
            .1
            .statements
            .len(),
        2
    );
    let upstream =
        generate_source_algorithm_postconditions(&two_core, snapshot_id(0), &generation, &schema)
            .unwrap_err();
    assert_eq!(upstream, "algorithms.postcondition.unsupported_core");
    assert_eq!(
        vcs.source_algorithm_return_canonical_components(
            VcId::new(99),
            &two_core,
            &generation,
            &empty,
        )
        .unwrap_err(),
        upstream
    );
}

#[test]
fn source_algorithm_return_local_handoff_preserves_genuine_projection() {
    use mizar_checker::type_checker::check_source_algorithm_types;
    use mizar_core::{
        binder_normalization::{
            BinderContext, CanonicalTermKind, CanonicalVar, NormalizedVarClass, NormalizedVarSort,
            canonical_term, normalize_core_term,
        },
        core_ir::{CoreFormulaKind, CoreProvenancePhase},
        elaborator::lower_source_algorithms,
    };
    use mizar_kernel::{
        certificate_parser::{ClauseTautologyPolicy, Fingerprint, KernelProfileRecord},
        checker::{
            FormulaEvidenceContext, ImportedFactContextLimits, KernelCheckPolicy,
            KernelCheckStatus, KernelContextIdentityEntry, KernelContextIdentityPayload,
            KernelContextIdentitySource, KernelEvidenceCheckInput, KernelEvidenceCheckKind,
            KernelEvidenceCheckLimits, KernelFormulaProducerRef, check_kernel_evidence,
        },
        clause::{Atom, SymbolKey, SymbolKind, Term, VariableId},
        formula_evidence::{Formula, FormulaEvidenceParseContext, parse_formula_evidence},
        rejection::{RejectionDetail, TargetVcFingerprint},
    };
    use mizar_vc::kernel_evidence_handoff::KernelContextIdentitySource as SourceContext;
    use mizar_vc::{
        generator::generate_source_algorithm_postconditions,
        kernel_evidence_handoff::*,
        vc_ir::{
            AnchorCompleteness, AnchorIngredient, ContextEntryKind, GenerationSchemaVersion,
            HashMarker, PremiseRef, SeedNoVcReason, SeedVcMapping, VcFormulaRef,
            VcGeneratedFormulaKind, VcGeneratedFormulaShape, VcKind, VcProvenancePhase,
            VcSchemaVersion, VcStatus,
        },
    };
    let config = config();
    let plan = crate::harness::build_test_plan(&config).unwrap();
    let case = plan
        .cases
        .iter()
        .find(|case| case.id.0 == "pass_proof_verification_algorithm_ensures_return_001")
        .unwrap();
    let original = fs::read_to_string(&case.source_path).unwrap();
    let guard_wire = [1, 4, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 1, 1, 0, 0, 0, 0];
    let goal_wire = [
        1, 3, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 2, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0,
    ];
    let guard_digest = "4d495a41525f4b45524e454c5f464f524d554c41000b010000001d0204000000170400000001000000010000000101010000000400000000";
    let goal_digest = "4d495a41525f4b45524e454c5f464f524d554c41000b0100000027020300000021030000000000000002000000020101000000040000000001010000000400000000";
    let pure_guard = Formula::Atom(Atom::new(
        SymbolKey::new(SymbolKind::BuiltinRelation, 1),
        vec![Term::Variable(VariableId(0))],
    ))
    .canonical_hash_input()
    .unwrap();
    let pure_goal = Formula::Atom(Atom::new(
        SymbolKey::new(SymbolKind::Equality, 0),
        vec![Term::Variable(VariableId(0)), Term::Variable(VariableId(0))],
    ))
    .canonical_hash_input()
    .unwrap();
    assert_eq!(
        pure_guard
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>(),
        guard_digest
    );
    assert_eq!(
        pure_goal
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>(),
        goal_digest
    );
    let symbols_manifest = [
        KernelManifestEntry {
            entry_id: 0,
            payload: vec![3, 0, 0, 0, 0],
        },
        KernelManifestEntry {
            entry_id: 1,
            payload: vec![4, 0, 0, 0, 1],
        },
    ];
    let variables_manifest = [KernelManifestEntry {
        entry_id: 0,
        payload: vec![0, 0, 0, 0],
    }];
    for (edit, text) in [
        original.clone(),
        original
            .replace(" a ", " param ")
            .replace("(a)", "(param)")
            .replace(" a;", " param;")
            .replace("= a\n", "= param\n"),
        format!(":: neutral algorithm trivia\n\n{original}"),
        original.replace("idalgo", "renamed_algorithm"),
    ]
    .into_iter()
    .enumerate()
    {
        let (source, typed, names) = super::super::super::source_registration_inputs(
            &config.workspace_root,
            case,
            super::super::super::formula_statement::step5c8_test_frontend(&text),
        )
        .unwrap();
        let check = check_source_algorithm_types(&source, &typed, &names).unwrap();
        let core = lower_source_algorithms(&check).unwrap();
        let (_, item) = core.items().iter().next().unwrap();
        let (_, algorithm) = core.algorithms().iter().next().unwrap();
        let [parameter] = algorithm.params.as_slice() else {
            panic!()
        };
        let result = algorithm.result.as_ref().unwrap();
        let [statement_id] = algorithm.statements.as_slice() else {
            panic!()
        };
        let statement = core.algorithm_statements().get(*statement_id).unwrap();
        assert_eq!(algorithm.contracts.ensures.len(), 1);
        assert_eq!(parameter.role.as_str(), "parameter");
        assert_eq!(result.role.as_str(), "result");
        let guard = parameter.ty_guard.unwrap();
        let CoreFormulaKind::TypePred { subject, ty } = &core.formulas().get(guard).unwrap().kind
        else {
            panic!()
        };
        assert_eq!(ty.as_str(), "object");
        let guard_source = &core.formulas().get(guard).unwrap().source;
        assert_eq!(guard_source.provenance.len(), 1);
        assert_eq!(
            guard_source.provenance[0].phase,
            CoreProvenancePhase::Checker
        );
        let before_core = core.clone();
        for (variant, (schema_text, generation_text, snapshot)) in [
            (
                "mizar-vc-vcset-step5c14-return-v1",
                "mizar-vc-generation-step5c14-return-v1",
                0,
            ),
            (
                "mizar-vc-vcset-step5c14-return-v1-changed",
                "mizar-vc-generation-step5c14-return-v1",
                0,
            ),
            (
                "mizar-vc-vcset-step5c14-return-v1",
                "mizar-vc-generation-step5c14-return-v1-changed",
                0,
            ),
            (
                "mizar-vc-vcset-step5c14-return-v1",
                "mizar-vc-generation-step5c14-return-v1",
                1,
            ),
        ]
        .into_iter()
        .take(if edit == 0 { 4 } else { 1 })
        .enumerate()
        {
            let generation = GenerationSchemaVersion::new(generation_text);
            let vcs = generate_source_algorithm_postconditions(
                &core,
                snapshot_id(snapshot),
                &generation,
                &VcSchemaVersion::new(schema_text),
            )
            .unwrap();
            let before_vcs = vcs.clone();
            let [vc] = vcs.vcs() else { panic!() };
            let [generated] = vcs.generated_formulas() else {
                panic!()
            };
            assert_eq!(vc.kind, VcKind::AlgorithmPostcondition);
            assert_eq!(vc.status, VcStatus::Open);
            assert_eq!(
                generated.kind,
                VcGeneratedFormulaKind::AlgorithmPostcondition
            );
            assert_eq!(vc.goal, VcFormulaRef::Generated(generated.id));
            let VcGeneratedFormulaShape::Equals { left, right } = generated.shape else {
                panic!()
            };
            let [entry] = vc.local_context.entries() else {
                panic!()
            };
            assert_eq!(entry.kind, ContextEntryKind::CheckerFact);
            assert_eq!(entry.formula, Some(VcFormulaRef::Core(guard)));
            assert_eq!(entry.sort_key.as_str(), "algorithm-parameter-type-0000");
            assert_eq!(vc.premises, [PremiseRef::LocalContext(entry.id)]);
            assert!(vc.proof_hint.is_none());
            assert_eq!(entry.provenance.len(), 1);
            assert_eq!(entry.provenance[0].phase, VcProvenancePhase::CoreHandoff);
            assert_eq!(entry.provenance[0].key.as_str(), "algorithm-parameter-type");
            assert_eq!(
                entry.provenance[0].core.as_ref(),
                Some(&guard_source.provenance[0])
            );
            assert_eq!(vc.local_context.policy_inputs().len(), 6);
            for (index, (name, value)) in [
                ("flow", "0"),
                ("site-kind", "Ensures"),
                ("ordinal", "0"),
                ("statement", "0"),
                ("block", "0"),
                ("exit", "0"),
            ]
            .into_iter()
            .enumerate()
            {
                let policy = &vc.local_context.policy_inputs()[index];
                assert_eq!(
                    policy.sort_key.as_str(),
                    format!("{index:04}-algorithm-site-{name}")
                );
                assert_eq!(policy.key.as_str(), format!("algorithm-site-{name}"));
                assert_eq!(policy.value.as_str(), value);
            }
            let mut binders = BinderContext::new();
            binders.declare_variable(
                parameter.var,
                NormalizedVarClass::Free,
                parameter.role.clone(),
                NormalizedVarSort::Term,
            );
            binders.declare_variable(
                result.var,
                NormalizedVarClass::Generated,
                result.role.clone(),
                NormalizedVarSort::Term,
            );
            for term_id in [*subject, left, right] {
                let term_source = &core.terms().get(term_id).unwrap().source;
                let normalized =
                    normalize_core_term(&core, term_id, &binders, term_source).unwrap();
                let canonical = canonical_term(&normalized, &binders, term_source).unwrap();
                assert_eq!(
                    canonical.kind(),
                    &CanonicalTermKind::Var(CanonicalVar::Free(parameter.var))
                );
            }
            let provenance = format!(
                "source-algorithm-return-local-handoff-v1;source={:?};module={:?};item={:?};algorithm={:?};statement={:?};accounting={:?}",
                core.source_id(), core.module_id(), item, algorithm, statement, vcs.seed_accounting(),
            ).into_bytes();
            let formulas = [
                KernelFormulaPayload {
                    formula_ref: VcFormulaRef::Core(guard),
                    projection: KernelFormulaProjection {
                        formula_fingerprint: KernelEvidenceFingerprint {
                            algorithm_id: 2,
                            digest: pure_guard.clone(),
                        },
                        formula_bytes: guard_wire.to_vec(),
                        provenance_payload: provenance.clone(),
                    },
                },
                KernelFormulaPayload {
                    formula_ref: vc.goal,
                    projection: KernelFormulaProjection {
                        formula_fingerprint: KernelEvidenceFingerprint {
                            algorithm_id: 2,
                            digest: pure_goal.clone(),
                        },
                        formula_bytes: goal_wire.to_vec(),
                        provenance_payload: provenance.clone(),
                    },
                },
            ];
            let expected = build_kernel_evidence_handoff(KernelEvidenceHandoffInput {
                vc_set: &vcs,
                vc: vc.id,
                goal_polarity: KernelGoalPolarity::AssertFalseForRefutation,
                kernel_profile: KernelEvidenceProfile::v1(1, KernelClauseTautologyPolicy::Reject),
                symbol_manifest: &symbols_manifest,
                variable_manifest: &variables_manifest,
                formula_payloads: &formulas,
                imported_formula_payloads: &[],
                substitutions: &[],
                formula_context: None,
                discharge_output: None,
            })
            .unwrap();
            let handoff =
                build_source_algorithm_return_kernel_handoff(&core, &vcs, vc.id, &generation)
                    .unwrap();
            assert_eq!(handoff, expected);
            assert_eq!(
                handoff,
                build_source_algorithm_return_kernel_handoff(&core, &vcs, vc.id, &generation)
                    .unwrap()
            );
            assert_eq!(
                handoff.canonical_evidence().symbol_manifest(),
                symbols_manifest
            );
            assert_eq!(
                handoff.canonical_evidence().variable_manifest(),
                variables_manifest
            );
            assert_eq!(handoff.canonical_evidence().formula_evidence().len(), 1);
            assert_eq!(
                handoff.canonical_evidence().formula_evidence()[0].producer_formula_ref(),
                Some(VcFormulaRef::Core(guard))
            );
            assert_eq!(
                handoff.canonical_evidence().formula_evidence()[0].formula_bytes(),
                guard_wire
            );
            assert_eq!(
                handoff.canonical_evidence().final_goal().formula_bytes,
                goal_wire
            );
            assert_eq!(
                handoff.canonical_evidence().final_goal().polarity,
                KernelGoalPolarity::AssertFalseForRefutation
            );
            assert!(handoff.canonical_evidence().substitutions().is_empty());
            assert!(handoff.formula_context_requirements().is_none());
            assert!(handoff.context_identity_is_consistent());
            assert!(handoff.targets_vc(&vcs, vc.id).unwrap());
            for row in handoff.canonical_evidence().provenance() {
                assert_eq!(row.payload, provenance);
            }
            assert_eq!(core, before_core);
            assert_eq!(vcs, before_vcs);
            if edit == 0 && variant == 0 {
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
                    panic!()
                };
                assert_eq!(
                    termination.seed_status,
                    mizar_core::core_ir::ObligationSeedStatus::Deferred
                );
                assert!(matches!(
                    &termination.mapping,
                    SeedVcMapping::NoConcreteVc {
                        reason: SeedNoVcReason::DeferredExternal(_),
                    }
                ));
                assert_eq!(
                    contract.seed_status,
                    mizar_core::core_ir::ObligationSeedStatus::Deferred
                );
                assert!(matches!(&contract.mapping, SeedVcMapping::One { vc: id } if *id == vc.id));
                let bytes = handoff.encode_local_evidence(None).unwrap();
                let target_fp = handoff.canonical_evidence().target_vc();
                let parsed = parse_formula_evidence(
                    &bytes,
                    &FormulaEvidenceParseContext::v1(
                        Fingerprint::new(target_fp.algorithm_id, target_fp.digest.clone()),
                        KernelProfileRecord::v1(1, ClauseTautologyPolicy::Reject),
                    ),
                )
                .unwrap();
                assert!(parsed.equality_instances().is_empty());
                let target =
                    TargetVcFingerprint::new(target_fp.algorithm_id, target_fp.digest.clone());
                let entries = handoff
                    .context_identity()
                    .entries()
                    .iter()
                    .map(|entry| {
                        let VcFormulaRef::Core(formula) = entry.producer_formula_ref() else {
                            panic!()
                        };
                        let SourceContext::LocalHypothesis { local_context_id } = entry.source()
                        else {
                            panic!()
                        };
                        KernelContextIdentityEntry::new(
                            KernelContextIdentitySource::LocalHypothesis { local_context_id },
                            entry.formula_id(),
                            Fingerprint::new(
                                entry.formula_fingerprint().algorithm_id,
                                entry.formula_fingerprint().digest.clone(),
                            ),
                            KernelFormulaProducerRef::Core(formula),
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
                let checked = check_kernel_evidence(KernelEvidenceCheckInput {
                    target_vc_fingerprint: &target,
                    evidence: &parsed,
                    formula_context: Some(&context),
                    check_kind: KernelEvidenceCheckKind::ProofObligation,
                    policy: KernelCheckPolicy::default(),
                    limits: KernelEvidenceCheckLimits::default(),
                });
                assert_eq!(checked.status(), KernelCheckStatus::Rejected);
                assert_eq!(checked.rejections().len(), 1);
                assert_eq!(
                    checked.rejections()[0].detail(),
                    RejectionDetail::InvalidSatRefutation
                );
                assert!(!checked.policy_taint());
            }
        }
    }
}

#[test]
fn source_algorithm_return_local_handoff_preserves_refusals() {
    use mizar_checker::type_checker::check_source_algorithm_types;
    use mizar_core::elaborator::lower_source_algorithms;
    use mizar_vc::{
        generator::generate_source_algorithm_postconditions,
        kernel_evidence_handoff::build_source_algorithm_return_kernel_handoff,
        vc_ir::{GenerationSchemaVersion, VcId, VcSchemaVersion, VcSet, VcSetParts},
    };
    let config = config();
    let plan = crate::harness::build_test_plan(&config).unwrap();
    let case = plan
        .cases
        .iter()
        .find(|case| case.id.0 == "pass_proof_verification_algorithm_ensures_return_001")
        .unwrap();
    let text = fs::read_to_string(&case.source_path).unwrap();
    let (source, typed, names) = super::super::super::source_registration_inputs(
        &config.workspace_root,
        case,
        super::super::super::formula_statement::step5c8_test_frontend(&text),
    )
    .unwrap();
    let check = check_source_algorithm_types(&source, &typed, &names).unwrap();
    let core = lower_source_algorithms(&check).unwrap();
    let generation = GenerationSchemaVersion::new("mizar-vc-generation-step5c14-return-v1");
    let schema = VcSchemaVersion::new("mizar-vc-vcset-step5c14-return-v1");
    let vcs = generate_source_algorithm_postconditions(&core, snapshot_id(0), &generation, &schema)
        .unwrap();
    let invalid = "algorithm.return.handoff.invalid_projection";
    assert_eq!(
        build_source_algorithm_return_kernel_handoff(&core, &vcs, VcId::new(99), &generation)
            .unwrap_err(),
        invalid
    );
    assert_eq!(
        build_source_algorithm_return_kernel_handoff(
            &core,
            &vcs,
            VcId::new(0),
            &GenerationSchemaVersion::new("different")
        )
        .unwrap_err(),
        invalid
    );
    let mut parts = VcSetParts {
        schema_version: vcs.schema_version().clone(),
        snapshot: vcs.snapshot(),
        source: vcs.source(),
        module: vcs.module().clone(),
        generated_formulas: vcs.generated_formulas().to_vec(),
        vcs: vcs.vcs().to_vec(),
        seed_accounting: vcs.seed_accounting().to_vec(),
    };
    parts.vcs[0].local_context = mizar_vc::vc_ir::LocalContext::try_new(
        {
            let mut entries = parts.vcs[0].local_context.entries().to_vec();
            entries[0].provenance[0].key = "drift".into();
            entries
        },
        parts.vcs[0].local_context.policy_inputs().to_vec(),
    )
    .unwrap();
    let drift = VcSet::try_new(parts).unwrap();
    assert_eq!(
        build_source_algorithm_return_kernel_handoff(&core, &drift, VcId::new(0), &generation)
            .unwrap_err(),
        invalid
    );
    for (kind, case_id, changed) in [
        (
            "foreign",
            "pass_proof_verification_algorithm_ensures_return_001",
            text.replace("idalgo", "foreign_algorithm"),
        ),
        (
            "zero",
            "pass_proof_verification_algorithm_ensures_return_001",
            text.replace("    ensures result = a\n", ""),
        ),
        (
            "broader",
            "pass_proof_verification_algorithm_var_const_assert_001",
            String::new(),
        ),
    ] {
        let case = plan.cases.iter().find(|case| case.id.0 == case_id).unwrap();
        let changed = if kind == "broader" {
            fs::read_to_string(&case.source_path).unwrap()
        } else {
            changed
        };
        let (source, typed, names) = super::super::super::source_registration_inputs(
            &config.workspace_root,
            case,
            super::super::super::formula_statement::step5c8_test_frontend(&changed),
        )
        .unwrap();
        let check = check_source_algorithm_types(&source, &typed, &names).unwrap();
        let other_core = lower_source_algorithms(&check).unwrap();
        let other_vcs = generate_source_algorithm_postconditions(
            &other_core,
            snapshot_id(0),
            &generation,
            &schema,
        )
        .unwrap();
        assert_ne!(other_vcs, vcs);
        if kind == "foreign" {
            assert_eq!(
                build_source_algorithm_return_kernel_handoff(
                    &other_core,
                    &vcs,
                    VcId::new(0),
                    &generation
                )
                .unwrap_err(),
                invalid
            );
        } else {
            assert_eq!(other_vcs.vcs().is_empty(), kind == "zero");
            assert_eq!(
                build_source_algorithm_return_kernel_handoff(
                    &other_core,
                    &other_vcs,
                    VcId::new(0),
                    &generation
                )
                .unwrap_err(),
                invalid
            );
        }
    }
    let two_returns = text
        .replace("    ensures result = a\n", "")
        .replace("return a;", "return a;\n    return a;");
    let (source, typed, names) = super::super::super::source_registration_inputs(
        &config.workspace_root,
        case,
        super::super::super::formula_statement::step5c8_test_frontend(&two_returns),
    )
    .unwrap();
    let check = check_source_algorithm_types(&source, &typed, &names).unwrap();
    let two_core = lower_source_algorithms(&check).unwrap();
    assert!(
        two_core
            .algorithms()
            .iter()
            .next()
            .unwrap()
            .1
            .contracts
            .ensures
            .is_empty()
    );
    assert_eq!(
        two_core
            .algorithms()
            .iter()
            .next()
            .unwrap()
            .1
            .statements
            .len(),
        2
    );
    let upstream =
        generate_source_algorithm_postconditions(&two_core, snapshot_id(0), &generation, &schema)
            .unwrap_err();
    assert_eq!(upstream, "algorithms.postcondition.unsupported_core");
    assert_eq!(
        build_source_algorithm_return_kernel_handoff(&two_core, &vcs, VcId::new(99), &generation)
            .unwrap_err(),
        upstream
    );
}

#[test]
fn source_algorithm_return_reflexivity_checks_genuine_targets() {
    use mizar_checker::type_checker::check_source_algorithm_types;
    use mizar_core::elaborator::lower_source_algorithms;
    use mizar_kernel::{
        certificate_parser::{ClauseTautologyPolicy, Fingerprint, KernelProfileRecord},
        checker::{
            FormulaEvidenceContext, ImportedFactContextLimits, KernelCheckPolicy,
            KernelCheckStatus, KernelContextIdentityEntry, KernelContextIdentityPayload,
            KernelContextIdentitySource, KernelEvidenceCheckInput, KernelEvidenceCheckKind,
            KernelEvidenceCheckLimits, KernelFormulaProducerRef, check_kernel_evidence,
        },
        formula_evidence::{FormulaEvidenceParseContext, parse_formula_evidence},
        rejection::TargetVcFingerprint,
    };
    use mizar_proof::{
        policy::{
            AcceptedGoalPolarity, CandidatePolicyClass, KernelEvidenceOrigin, KernelPolicyInput,
            PolicyCandidate, ProofPolicyEvaluator, VerifierPolicy,
        },
        status::check_source_algorithm_return_reflexivity as verify,
    };
    use mizar_vc::{
        generator::generate_source_algorithm_postconditions,
        kernel_evidence_handoff::{
            KernelClauseTautologyPolicy, KernelContextIdentitySource as SourceContext,
            build_source_algorithm_return_kernel_handoff,
        },
        vc_ir::{
            AnchorCompleteness, AnchorIngredient, GenerationSchemaVersion, HashMarker,
            SeedNoVcReason, SeedVcMapping, VcFormulaRef, VcSchemaVersion, VcStatus,
        },
    };
    let config = config();
    let plan = crate::harness::build_test_plan(&config).unwrap();
    let case = plan
        .cases
        .iter()
        .find(|case| case.id.0 == "pass_proof_verification_algorithm_ensures_return_001")
        .unwrap();
    let original = fs::read_to_string(&case.source_path).unwrap();
    let variants = [
        original.clone(),
        original
            .replace(" a ", " param ")
            .replace("(a)", "(param)")
            .replace(" a;", " param;")
            .replace("= a\n", "= param\n"),
        format!(":: 日本語 λ\n\n{original}"),
        original.replace("idalgo", "renamed_algorithm"),
    ];
    let generation = GenerationSchemaVersion::new("mizar-vc-generation-step5c14-return-v1");
    let schema = VcSchemaVersion::new("mizar-vc-vcset-step5c14-return-v1");
    for (variant, text) in variants.into_iter().enumerate() {
        let (source, nodes, symbols) = super::super::super::source_registration_inputs(
            &config.workspace_root,
            case,
            super::super::super::formula_statement::step5c8_test_frontend(&text),
        )
        .unwrap();
        let immutable = (source.clone(), nodes.clone(), symbols.clone());
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
            let [vc] = vcs.vcs() else { panic!() };
            assert_eq!(vc.status, VcStatus::Open);
            assert!(matches!(
                &vc.anchor.completeness,
                AnchorCompleteness::Incomplete { missing }
                if missing == &[AnchorIngredient::CanonicalGoalHash, AnchorIngredient::CanonicalContextHash]
            ));
            assert!(matches!(
                &vc.anchor.canonical_goal_hash,
                HashMarker::ConservativeUnknown { .. }
            ));
            assert!(matches!(
                &vc.anchor.canonical_context_hash,
                HashMarker::ConservativeUnknown { .. }
            ));
            let [termination, contract] = vcs.seed_accounting() else {
                panic!()
            };
            assert_eq!(
                termination.seed_status,
                mizar_core::core_ir::ObligationSeedStatus::Deferred
            );
            assert!(matches!(
                &termination.mapping,
                SeedVcMapping::NoConcreteVc {
                    reason: SeedNoVcReason::DeferredExternal(_),
                }
            ));
            assert_eq!(
                contract.seed_status,
                mizar_core::core_ir::ObligationSeedStatus::Deferred
            );
            assert!(matches!(&contract.mapping, SeedVcMapping::One { vc: id } if *id == vc.id));
            let handoff =
                build_source_algorithm_return_kernel_handoff(&core, &vcs, vc.id, &generation)
                    .unwrap();
            let envelope = handoff.canonical_evidence();
            let target = envelope.target_vc();
            let profile = envelope.kernel_profile();
            let tautology = match profile.clause_tautology_policy {
                KernelClauseTautologyPolicy::Reject => ClauseTautologyPolicy::Reject,
                KernelClauseTautologyPolicy::Marker => ClauseTautologyPolicy::Marker,
                _ => panic!(),
            };
            let mut context = FormulaEvidenceParseContext::v1(
                Fingerprint::new(target.algorithm_id, target.digest.clone()),
                KernelProfileRecord::v1(profile.profile_id, tautology),
            );
            let parsed_v1 =
                parse_formula_evidence(&handoff.encode_local_evidence(None).unwrap(), &context)
                    .unwrap();
            let names =
                mizar_atp::backend::encode_local_variable_reflexivity_names(&parsed_v1).unwrap();
            assert_eq!(names, vec![vec![0, 0, 0, 1, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0]]);
            context.accepted_schema_versions.insert(2);
            let parsed = parse_formula_evidence(
                &handoff.encode_local_evidence(Some(&names)).unwrap(),
                &context,
            )
            .unwrap();
            let target = TargetVcFingerprint::new(target.algorithm_id, target.digest.clone());
            let entries = handoff
                .context_identity()
                .entries()
                .iter()
                .map(|entry| {
                    let SourceContext::LocalHypothesis { local_context_id } = entry.source() else {
                        panic!()
                    };
                    let VcFormulaRef::Core(formula) = entry.producer_formula_ref() else {
                        panic!()
                    };
                    KernelContextIdentityEntry::new(
                        KernelContextIdentitySource::LocalHypothesis { local_context_id },
                        entry.formula_id(),
                        Fingerprint::new(
                            entry.formula_fingerprint().algorithm_id,
                            entry.formula_fingerprint().digest.clone(),
                        ),
                        KernelFormulaProducerRef::Core(formula),
                    )
                })
                .collect();
            let formula_context = FormulaEvidenceContext::with_context_identity(
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
                    has_stable_kernel_representation: true,
                }
            ));
            let expected_result = check_kernel_evidence(KernelEvidenceCheckInput {
                target_vc_fingerprint: &target,
                evidence: &parsed,
                formula_context: Some(&formula_context),
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
            let actual = verify(&source, &nodes, &symbols, snapshot, vc.id, &policy).unwrap();
            assert_eq!(actual, (expected_result, expected_decision));
            assert_eq!(actual.0.status(), KernelCheckStatus::Accepted);
            assert_eq!(actual.0.target_vc_fingerprint(), &target);
            assert_eq!(
                actual.0.evidence_check_kind(),
                Some(KernelEvidenceCheckKind::ProofObligation)
            );
            assert!(!actual.0.policy_taint());
            assert!(actual.0.used_axioms().is_empty());
            assert!(input.accepted_evidence_hash().is_some());
            assert_eq!(
                input.accepted_goal_polarity(),
                Some(AcceptedGoalPolarity::AssertFalseForRefutation)
            );
            assert_eq!(actual.1.class, CandidatePolicyClass::DischargedBuiltin);
            assert_eq!(
                actual,
                verify(&source, &nodes, &symbols, snapshot, vc.id, &policy).unwrap()
            );
            assert_eq!((&core, &vcs, &policy), (&prior.0, &prior.1, &prior.2));
        }
        assert_eq!(
            (&source, &nodes, &symbols),
            (&immutable.0, &immutable.1, &immutable.2)
        );
    }
}

#[test]
fn source_algorithm_return_reflexivity_preserves_upstream_and_policy_errors() {
    use mizar_checker::type_checker::check_source_algorithm_types;
    use mizar_core::elaborator::lower_source_algorithms;
    use mizar_proof::{
        policy::{KernelEvidenceFormat, VerifierPolicy},
        status::check_source_algorithm_return_reflexivity as verify,
    };
    use mizar_vc::{
        generator::generate_source_algorithm_postconditions,
        kernel_evidence_handoff::build_source_algorithm_return_kernel_handoff,
        vc_ir::{GenerationSchemaVersion, VcId, VcSchemaVersion},
    };
    let config = config();
    let plan = crate::harness::build_test_plan(&config).unwrap();
    let case = plan
        .cases
        .iter()
        .find(|case| case.id.0 == "pass_proof_verification_algorithm_ensures_return_001")
        .unwrap();
    let original = fs::read_to_string(&case.source_path).unwrap();
    let generation = GenerationSchemaVersion::new("mizar-vc-generation-step5c14-return-v1");
    let schema = VcSchemaVersion::new("mizar-vc-vcset-step5c14-return-v1");
    let disabled = VerifierPolicy::default().with_kernel_evidence_formats([]);
    let (source, nodes, symbols) = super::super::super::source_registration_inputs(
        &config.workspace_root,
        case,
        super::super::super::formula_statement::step5c8_test_frontend(&original),
    )
    .unwrap();
    let check = check_source_algorithm_types(&source, &nodes, &symbols).unwrap();
    let core = lower_source_algorithms(&check).unwrap();
    let vcs = generate_source_algorithm_postconditions(&core, snapshot_id(0), &generation, &schema)
        .unwrap();
    let absent = VcId::new(99);
    let upstream =
        build_source_algorithm_return_kernel_handoff(&core, &vcs, absent, &generation).unwrap_err();
    assert_eq!(
        verify(&source, &nodes, &symbols, snapshot_id(0), absent, &disabled).unwrap_err(),
        upstream
    );
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
                VcId::new(0),
                &policy
            )
            .unwrap_err(),
            "algorithm.proof.kernel_policy_rejected"
        );
    }
    for (text, case) in [
        (original.replace("    ensures result = a\n", ""), case),
        (
            String::new(),
            plan.cases
                .iter()
                .find(|case| case.id.0 == "pass_proof_verification_algorithm_var_const_assert_001")
                .unwrap(),
        ),
    ] {
        let text = if text.is_empty() {
            fs::read_to_string(&case.source_path).unwrap()
        } else {
            text
        };
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
            build_source_algorithm_return_kernel_handoff(&core, &vcs, VcId::new(0), &generation)
                .unwrap_err();
        assert_eq!(
            verify(
                &source,
                &nodes,
                &symbols,
                snapshot_id(0),
                VcId::new(0),
                &VerifierPolicy::default()
            )
            .unwrap_err(),
            expected
        );
    }
    let two_returns = original
        .replace("    ensures result = a\n", "")
        .replace("return a;", "return a;\n    return a;");
    let (source, nodes, symbols) = super::super::super::source_registration_inputs(
        &config.workspace_root,
        case,
        super::super::super::formula_statement::step5c8_test_frontend(&two_returns),
    )
    .unwrap();
    let check = check_source_algorithm_types(&source, &nodes, &symbols).unwrap();
    let core = lower_source_algorithms(&check).unwrap();
    let expected =
        generate_source_algorithm_postconditions(&core, snapshot_id(0), &generation, &schema)
            .unwrap_err();
    assert_eq!(expected, "algorithms.postcondition.unsupported_core");
    assert_eq!(
        verify(
            &source,
            &nodes,
            &symbols,
            snapshot_id(0),
            absent,
            &VerifierPolicy::default()
        )
        .unwrap_err(),
        expected
    );
    let bare_return = original.replace("return a;", "return;");
    let (source, nodes, symbols) = super::super::super::source_registration_inputs(
        &config.workspace_root,
        case,
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
            absent,
            &VerifierPolicy::default().with_kernel_evidence_formats([])
        )
        .unwrap_err(),
        expected
    );
}

#[test]
fn source_algorithm_return_canonical_inputs_preserve_components_and_slice() {
    use mizar_checker::type_checker::check_source_algorithm_types;
    use mizar_core::elaborator::{lower_source_algorithms, source_algorithm_formula_payloads};
    use mizar_proof::status::source_algorithm_return_canonical_inputs;
    use mizar_vc::{
        dependency_slice::{
            DependencySliceCompleteness, DependencySliceInput, try_compute_dependency_slices,
        },
        generator::generate_source_algorithm_postconditions,
        vc_ir::{
            AnchorCompleteness, AnchorIngredient, GenerationSchemaVersion, HashMarker,
            SeedNoVcReason, SeedVcMapping, VcKind, VcSchemaVersion, VcStatus,
        },
    };

    let config = config();
    let plan = crate::harness::build_test_plan(&config).unwrap();
    let case = plan
        .cases
        .iter()
        .find(|case| case.id.0 == "pass_proof_verification_algorithm_ensures_return_001")
        .unwrap();
    let original = fs::read_to_string(&case.source_path).unwrap();
    for (edit, text) in [
        original.clone(),
        original
            .replace(" a ", " param ")
            .replace("(a)", "(param)")
            .replace(" a;", " param;")
            .replace("= a\n", "= param\n"),
        format!(":: 世代 neutral trivia\n\n{original}"),
        original.replace("idalgo", "renamed_algorithm"),
    ]
    .into_iter()
    .enumerate()
    {
        let (source, nodes, symbols) = super::super::super::source_registration_inputs(
            &config.workspace_root,
            case,
            super::super::super::formula_statement::step5c8_test_frontend(&text),
        )
        .unwrap();
        let check = check_source_algorithm_types(&source, &nodes, &symbols).unwrap();
        let before_inputs = format!("{source:?}{nodes:?}{symbols:?}{check:?}");
        for snapshot_ordinal in 0..if edit == 0 { 2 } else { 1 } {
            let snapshot = snapshot_id(snapshot_ordinal);
            let core = lower_source_algorithms(&check).unwrap();
            let generation = GenerationSchemaVersion::new("mizar-vc-generation-step5c14-return-v1");
            let schema = VcSchemaVersion::new("mizar-vc-vcset-step5c14-return-v1");
            let vcs =
                generate_source_algorithm_postconditions(&core, snapshot, &generation, &schema)
                    .unwrap();
            let [vc] = vcs.vcs() else {
                panic!("one genuine return VC")
            };
            let [parameter] = core.algorithms().iter().next().unwrap().1.params.as_slice() else {
                panic!("one genuine parameter")
            };
            let guard = parameter.ty_guard.unwrap();
            let formulas = source_algorithm_formula_payloads(&check, &core, &[guard]).unwrap();
            let components = vcs
                .source_algorithm_return_canonical_components(vc.id, &core, &generation, &formulas)
                .unwrap();
            let slices = try_compute_dependency_slices(DependencySliceInput {
                vc_set: &vcs,
                discharge_output: None,
            })
            .unwrap();
            let original_slice = slices.slice_for(vc.id).unwrap();
            let before_lower = (
                core.clone(),
                vcs.clone(),
                formulas.clone(),
                components,
                original_slice.clone(),
            );

            assert_eq!(vc.kind, VcKind::AlgorithmPostcondition);
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
            assert!(vcs.canonical_vc_fingerprint(vc.id).is_none());
            assert!(vcs.local_context_fingerprint(vc.id).is_none());
            let [termination, contract] = vcs.seed_accounting() else {
                panic!("return accounting")
            };
            assert_eq!(
                termination.seed_status,
                mizar_core::core_ir::ObligationSeedStatus::Deferred
            );
            assert!(matches!(
                &termination.mapping,
                SeedVcMapping::NoConcreteVc {
                    reason: SeedNoVcReason::DeferredExternal(_),
                }
            ));
            assert!(matches!(&contract.mapping, SeedVcMapping::One { vc: id } if *id == vc.id));

            let actual = source_algorithm_return_canonical_inputs(&check, snapshot, vc.id).unwrap();
            assert_eq!((actual.0, actual.1, actual.2), components);
            assert_eq!(&actual.3, original_slice);
            assert_eq!(actual.3.vc(), vc.id);
            assert_eq!(actual.3.kind(), &VcKind::AlgorithmPostcondition);
            assert_eq!(actual.3.status(), &VcStatus::Open);
            assert!(!actual.3.entries().is_empty());
            assert!(!actual.3.unknowns().is_empty());
            assert_eq!(
                actual.3.completeness(),
                DependencySliceCompleteness::IncompleteUncacheable
            );
            assert!(actual.3.requires_cache_miss());
            assert_eq!(
                actual,
                source_algorithm_return_canonical_inputs(&check, snapshot, vc.id).unwrap()
            );
            assert_eq!(
                (core, vcs, formulas, components, original_slice.clone()),
                before_lower
            );
            assert_eq!(
                format!("{source:?}{nodes:?}{symbols:?}{check:?}"),
                before_inputs
            );
        }
    }
}

#[test]
fn source_algorithm_return_canonical_inputs_preserve_upstream_order() {
    use mizar_checker::type_checker::check_source_algorithm_types;
    use mizar_core::elaborator::{lower_source_algorithms, source_algorithm_formula_payloads};
    use mizar_proof::status::source_algorithm_return_canonical_inputs;
    use mizar_vc::{
        generator::generate_source_algorithm_postconditions,
        vc_ir::{GenerationSchemaVersion, VcId, VcSchemaVersion},
    };

    let config = config();
    let plan = crate::harness::build_test_plan(&config).unwrap();
    let case = plan
        .cases
        .iter()
        .find(|case| case.id.0 == "pass_proof_verification_algorithm_ensures_return_001")
        .unwrap();
    let original = fs::read_to_string(&case.source_path).unwrap();
    let generation = GenerationSchemaVersion::new("mizar-vc-generation-step5c14-return-v1");
    let schema = VcSchemaVersion::new("mizar-vc-vcset-step5c14-return-v1");
    let absent = VcId::new(usize::MAX);
    for (kind, case, text) in [
        ("absent", case, original.clone()),
        (
            "zero",
            case,
            original.replace("    ensures result = a\n", ""),
        ),
        (
            "broader",
            plan.cases
                .iter()
                .find(|case| case.id.0 == "pass_proof_verification_algorithm_var_const_assert_001")
                .unwrap(),
            String::new(),
        ),
        (
            "two_returns",
            case,
            original
                .replace("    ensures result = a\n", "")
                .replace("return a;", "return a;\n    return a;"),
        ),
    ] {
        let text = if kind == "broader" {
            fs::read_to_string(&case.source_path).unwrap()
        } else {
            text
        };
        let (source, nodes, symbols) = super::super::super::source_registration_inputs(
            &config.workspace_root,
            case,
            super::super::super::formula_statement::step5c8_test_frontend(&text),
        )
        .unwrap();
        let check = check_source_algorithm_types(&source, &nodes, &symbols).unwrap();
        let before = format!("{source:?}{nodes:?}{symbols:?}{check:?}");
        let core = lower_source_algorithms(&check).unwrap();
        let generated =
            generate_source_algorithm_postconditions(&core, snapshot_id(0), &generation, &schema);
        let (selected, expected) = if kind == "two_returns" {
            (absent, generated.unwrap_err())
        } else {
            let vcs = generated.unwrap();
            let roots = core
                .algorithms()
                .iter()
                .flat_map(|(_, algorithm)| {
                    algorithm
                        .params
                        .iter()
                        .filter_map(|parameter| parameter.ty_guard)
                })
                .collect::<Vec<_>>();
            let formulas = source_algorithm_formula_payloads(&check, &core, &roots).unwrap();
            if kind == "zero" {
                assert!(vcs.vcs().is_empty());
            }
            let selected = if kind == "broader" {
                let selected = vcs.vcs().first().expect("broader raw VC").id;
                assert!(vcs.vc(selected).is_some());
                selected
            } else {
                absent
            };
            (
                selected,
                vcs.source_algorithm_return_canonical_components(
                    selected,
                    &core,
                    &generation,
                    &formulas,
                )
                .unwrap_err(),
            )
        };
        assert_eq!(
            source_algorithm_return_canonical_inputs(&check, snapshot_id(0), selected).unwrap_err(),
            expected
        );
        assert_eq!(format!("{source:?}{nodes:?}{symbols:?}{check:?}"), before);
    }
}

#[test]
fn source_algorithm_return_obligation_identity_preserves_genuine_fields_and_slice() {
    use mizar_artifact::store::{CanonicalJson, canonical_json_bytes};
    use mizar_proof::status::{
        source_algorithm_return_canonical_inputs, source_algorithm_return_obligation_identity,
    };
    use mizar_vc::vc_ir::source_algorithm_return_candidate_anchor_hash;

    let original = fs::read_to_string(config().workspace_root.join(
        "tests/miz/pass/algorithms/pass_proof_verification_algorithm_ensures_return_001.miz",
    ))
    .unwrap();
    for (text, candidate_hex, final_hex) in [
        (
            original.clone(),
            "b8754196e63d0faf7227278f18ffe36efe43e368355f83e57e0fd71e3efcb92e",
            "75a724cd673ed59039baa1873159384ad6ba0c1848b9265264f73af0850e0a47",
        ),
        (
            original.replace("terminating algorithm", "algorithm"),
            "d466807aeac30ceca7fcb082fb0cfa230031d52413dff814fd25803cef810a1a",
            "67edf90e23328b9b98f91493286e20c4054999675d53d5b4201f9374116c7f7c",
        ),
    ] {
        with_source_algorithm_identity_inputs(
            &text,
            "owner-test",
            "owners",
            0,
            |source, typed, symbols, check| {
                let before = format!("{source:?}{typed:?}{symbols:?}{check:?}");
                let snapshot = snapshot_id(0);
                let core = mizar_core::elaborator::lower_source_algorithms(check).unwrap();
                let vcs = mizar_vc::generator::generate_source_algorithm_postconditions(
                    &core,
                    snapshot,
                    &GenerationSchemaVersion::new("mizar-vc-generation-step5c14-return-v1"),
                    &VcSchemaVersion::new("mizar-vc-vcset-step5c14-return-v1"),
                )
                .unwrap();
                let [target] = vcs.vcs() else {
                    panic!("one genuine return VC")
                };
                let old =
                    source_algorithm_return_canonical_inputs(check, snapshot, target.id).unwrap();
                assert!(!old.3.entries().is_empty());
                assert!(!old.3.unknowns().is_empty());
                assert!(old.3.requires_cache_miss());
                let syntax = mizar_resolve::symbols::source_algorithm_syntax_payloads_from_arena(
                    source, symbols,
                )
                .unwrap();
                assert_eq!(syntax.len(), 1);
                let semantic =
                    mizar_core::elaborator::source_algorithm_core_semantic_payload(check, &core)
                        .unwrap();
                let mut entries = symbols.symbols().iter().filter(|entry| {
                    entry.symbol() == &check.algorithm().1
                        && entry.kind() == mizar_resolve::env::SymbolKind::Algorithm
                });
                let entry = entries.next().unwrap();
                assert!(entries.next().is_none());
                let module = entry.symbol().module();
                let owner = CanonicalJson::object([
                    (
                        "package_id",
                        CanonicalJson::string(module.package().as_str()),
                    ),
                    ("module_path", CanonicalJson::string(module.path().as_str())),
                    (
                        "namespace",
                        CanonicalJson::string(entry.namespace().as_str()),
                    ),
                    ("spelling", CanonicalJson::string(entry.primary_spelling())),
                    ("kind", CanonicalJson::string("algorithm")),
                    ("visibility", CanonicalJson::string("public")),
                ])
                .unwrap();
                let owner = canonical_json_bytes(&owner);
                let syntax = syntax.get(&check.algorithm().1).unwrap();
                let hash = source_algorithm_return_candidate_anchor_hash(
                    &owner, syntax, &semantic, old.0, old.1, old.2,
                )
                .unwrap();
                let hex = |bytes: &[u8]| -> String {
                    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
                };
                assert_eq!(hex(hash.as_bytes()), candidate_hex);
                assert_eq!(
                    hash,
                    source_algorithm_return_candidate_anchor_hash(
                        &owner, syntax, &semantic, old.0, old.1, old.2,
                    )
                    .unwrap()
                );
                for (owner, syntax, semantic) in [
                    (&[][..], syntax.as_slice(), semantic.as_slice()),
                    (owner.as_slice(), &[][..], semantic.as_slice()),
                    (owner.as_slice(), syntax.as_slice(), &[][..]),
                ] {
                    assert_eq!(
                        source_algorithm_return_candidate_anchor_hash(
                            owner, syntax, semantic, old.0, old.1, old.2,
                        )
                        .unwrap_err(),
                        "unsupported source algorithm return candidate anchor"
                    );
                }
                let (identity, slice) = source_algorithm_return_obligation_identity(
                    source, check, symbols, snapshot, target.id,
                )
                .unwrap();
                let anchor = format!("source-algorithm-return-candidate-anchor-v1:{candidate_hex}");
                assert_eq!(identity.obligation_id(), anchor);
                assert_eq!(identity.obligation_anchor().as_str(), anchor);
                assert_eq!(hex(identity.obligation_fingerprint().as_bytes()), final_hex);
                assert_eq!(identity.vc_fingerprint(), old.0.hash());
                assert_eq!(identity.local_context_fingerprint(), old.2.hash());
                assert_eq!(
                    identity.dependency_slice_fingerprint(),
                    old.3.fingerprint().hash()
                );
                assert_eq!(slice, old.3);
                assert_eq!(
                    (identity.clone(), slice.clone()),
                    source_algorithm_return_obligation_identity(
                        source, check, symbols, snapshot, target.id,
                    )
                    .unwrap()
                );
                assert_eq!(
                    old,
                    source_algorithm_return_canonical_inputs(check, snapshot, target.id).unwrap()
                );
                assert_eq!(format!("{source:?}{typed:?}{symbols:?}{check:?}"), before);
            },
        );
    }
}

#[test]
fn source_algorithm_return_obligation_identity_preserves_source_associations() {
    use mizar_artifact::store::{CanonicalJson, canonical_json_bytes};
    use mizar_proof::status::{
        source_algorithm_return_canonical_inputs, source_algorithm_return_obligation_identity,
    };
    let original = fs::read_to_string(config().workspace_root.join(
        "tests/miz/pass/algorithms/pass_proof_verification_algorithm_ensures_return_001.miz",
    ))
    .unwrap();
    let parameter = original
        .replace(" a ", " param ")
        .replace("(a)", "(param)")
        .replace(" a;", " param;")
        .replace("= a\n", "= param\n");
    let mut observed = Vec::new();
    for (label, text, package, module, skip, expected) in [
        (
            "original",
            original.clone(),
            "owner-test",
            "owners",
            0,
            "b8754196e63d0faf7227278f18ffe36efe43e368355f83e57e0fd71e3efcb92e",
        ),
        (
            "without-terminating",
            original.replace("terminating algorithm", "algorithm"),
            "owner-test",
            "owners",
            0,
            "d466807aeac30ceca7fcb082fb0cfa230031d52413dff814fd25803cef810a1a",
        ),
        (
            "trivia",
            format!(":: 世代 neutral trivia\n\n{original}"),
            "owner-test",
            "owners",
            0,
            "b8754196e63d0faf7227278f18ffe36efe43e368355f83e57e0fd71e3efcb92e",
        ),
        (
            "source-relocation",
            original.clone(),
            "owner-test",
            "owners",
            7,
            "b8754196e63d0faf7227278f18ffe36efe43e368355f83e57e0fd71e3efcb92e",
        ),
        (
            "package",
            original.clone(),
            "other-package",
            "owners",
            0,
            "e137ff9b58ed577c5da787e28f09f0298e5b14bfa6af6a8aeca92f8a29aa1379",
        ),
        (
            "module",
            original.clone(),
            "owner-test",
            "nested.owners",
            0,
            "c0abe548fcd249b548741207fb38271ad0d58b85d3085cdd796ca6920711e995",
        ),
        (
            "owner",
            original.replace("idalgo", "renamed_algorithm"),
            "owner-test",
            "owners",
            0,
            "f3b43e9245ce0a81e62ec093264dbaa6c644b5fda58aa198b5dafb098f4484a8",
        ),
        (
            "parameter",
            parameter,
            "owner-test",
            "owners",
            0,
            "90dc7d7b6880a43db46c7da2b1215ff3bcaca8785191b48172a115341d871286",
        ),
    ] {
        let operands = with_source_algorithm_identity_inputs(
            &text,
            package,
            module,
            skip,
            |source, _, symbols, check| {
                let snapshot = snapshot_id(0);
                let core = mizar_core::elaborator::lower_source_algorithms(check).unwrap();
                let vcs = mizar_vc::generator::generate_source_algorithm_postconditions(
                    &core,
                    snapshot,
                    &GenerationSchemaVersion::new("mizar-vc-generation-step5c14-return-v1"),
                    &VcSchemaVersion::new("mizar-vc-vcset-step5c14-return-v1"),
                )
                .unwrap();
                let [target] = vcs.vcs() else {
                    panic!("one genuine return VC")
                };
                let old =
                    source_algorithm_return_canonical_inputs(check, snapshot, target.id).unwrap();
                assert!(!old.3.unknowns().is_empty());
                assert!(old.3.requires_cache_miss());
                let syntax = mizar_resolve::symbols::source_algorithm_syntax_payloads_from_arena(
                    source, symbols,
                )
                .unwrap();
                assert_eq!(syntax.len(), 1);
                let semantic =
                    mizar_core::elaborator::source_algorithm_core_semantic_payload(check, &core)
                        .unwrap();
                let entry = symbols
                    .symbols()
                    .iter()
                    .find(|entry| {
                        entry.symbol() == &check.algorithm().1
                            && entry.kind() == mizar_resolve::env::SymbolKind::Algorithm
                    })
                    .unwrap();
                let owner = CanonicalJson::object([
                    (
                        "package_id",
                        CanonicalJson::string(entry.symbol().module().package().as_str()),
                    ),
                    (
                        "module_path",
                        CanonicalJson::string(entry.symbol().module().path().as_str()),
                    ),
                    (
                        "namespace",
                        CanonicalJson::string(entry.namespace().as_str()),
                    ),
                    ("spelling", CanonicalJson::string(entry.primary_spelling())),
                    ("kind", CanonicalJson::string("algorithm")),
                    ("visibility", CanonicalJson::string("public")),
                ])
                .unwrap();
                let operands = (
                    canonical_json_bytes(&owner),
                    syntax[&check.algorithm().1].clone(),
                    semantic,
                    old.0,
                    old.1,
                    old.2,
                );
                if let Some((baseline_source_id, baseline)) = observed.first() {
                    match label {
                        "trivia" | "source-relocation" => {
                            assert_eq!(&operands, baseline);
                            if label == "source-relocation" {
                                assert_ne!(source.source_id(), *baseline_source_id);
                            }
                        }
                        "without-terminating" | "parameter" => {
                            assert_eq!(operands.0, baseline.0);
                            assert_ne!(operands.1, baseline.1);
                            assert_eq!(operands.2, baseline.2);
                            assert_eq!(
                                (operands.3, operands.4, operands.5),
                                (baseline.3, baseline.4, baseline.5)
                            );
                        }
                        "package" | "module" | "owner" => {
                            assert_ne!(operands.0, baseline.0);
                        }
                        _ => {}
                    }
                }
                let (identity, slice) = source_algorithm_return_obligation_identity(
                    source, check, symbols, snapshot, target.id,
                )
                .unwrap();
                assert_eq!(slice, old.3);
                assert_eq!(
                    identity.obligation_anchor().as_str(),
                    format!("source-algorithm-return-candidate-anchor-v1:{expected}")
                );
                if label == "original" {
                    let old_other =
                        source_algorithm_return_canonical_inputs(check, snapshot_id(1), target.id)
                            .unwrap();
                    let other = source_algorithm_return_obligation_identity(
                        source,
                        check,
                        symbols,
                        snapshot_id(1),
                        target.id,
                    )
                    .unwrap();
                    assert_eq!(other.1, old_other.3);
                }
                (source.source_id(), operands)
            },
        );
        observed.push(operands);
    }
}

#[test]
fn source_algorithm_return_obligation_identity_preserves_upstream_errors_and_inputs() {
    use mizar_checker::type_checker::check_source_algorithm_types;
    use mizar_proof::status::{
        source_algorithm_return_canonical_inputs, source_algorithm_return_obligation_identity,
    };
    use mizar_vc::vc_ir::VcId;

    let root = config().workspace_root;
    let original = fs::read_to_string(root.join(
        "tests/miz/pass/algorithms/pass_proof_verification_algorithm_ensures_return_001.miz",
    ))
    .unwrap();
    let broader = fs::read_to_string(root.join(
        "tests/miz/pass/algorithms/pass_proof_verification_algorithm_var_const_assert_001.miz",
    ))
    .unwrap();
    for (kind, text) in [
        ("absent", original.clone()),
        ("zero", original.replace("    ensures result = a\n", "")),
        ("broader", broader),
        (
            "two_returns",
            original
                .replace("    ensures result = a\n", "")
                .replace("return a;", "return a;\n    return a;"),
        ),
    ] {
        with_source_algorithm_identity_inputs(
            &text,
            "owner-test",
            "owners",
            0,
            |source, typed, symbols, check| {
                let before = format!("{source:?}{typed:?}{symbols:?}{check:?}");
                let snapshot = snapshot_id(0);
                let fresh = check_source_algorithm_types(source, check.typed(), symbols).unwrap();
                assert_eq!(fresh.algorithm(), check.algorithm());
                let core = mizar_core::elaborator::lower_source_algorithms(&fresh).unwrap();
                let generated = mizar_vc::generator::generate_source_algorithm_postconditions(
                    &core,
                    snapshot,
                    &GenerationSchemaVersion::new("mizar-vc-generation-step5c14-return-v1"),
                    &VcSchemaVersion::new("mizar-vc-vcset-step5c14-return-v1"),
                );
                let absent = VcId::new(usize::MAX);
                let selected = if kind == "broader" {
                    generated.as_ref().unwrap().vcs().first().unwrap().id
                } else {
                    absent
                };
                if kind == "zero" {
                    assert!(generated.as_ref().unwrap().vcs().is_empty());
                }
                if kind == "two_returns" {
                    assert!(generated.is_err());
                }
                let expected = source_algorithm_return_canonical_inputs(&fresh, snapshot, selected)
                    .unwrap_err();
                if kind == "two_returns" {
                    assert_eq!(expected, generated.unwrap_err());
                }
                assert_eq!(
                    source_algorithm_return_obligation_identity(
                        source, check, symbols, snapshot, selected,
                    )
                    .unwrap_err(),
                    expected
                );
                if kind == "absent" {
                    for changed in [
                        original.replace("idalgo", "foreign_algorithm"),
                        format!("{original}\n{original}"),
                    ] {
                        let (_, _, foreign) =
                            super::super::super::tests::source_registration_owner_inputs(
                                &changed,
                                "owner-test",
                                "owners",
                                0,
                            );
                        let actual =
                            check_source_algorithm_types(source, check.typed(), foreign.env())
                                .unwrap_err();
                        assert_eq!(
                            source_algorithm_return_obligation_identity(
                                source,
                                check,
                                foreign.env(),
                                snapshot,
                                absent,
                            )
                            .unwrap_err(),
                            actual
                        );
                    }
                }
                assert_eq!(format!("{source:?}{typed:?}{symbols:?}{check:?}"), before);
            },
        );
    }
}

#[test]
fn source_algorithm_return_status_matches_genuine_projection() {
    use mizar_proof::{
        policy::{CandidatePolicyClass, KernelEvidenceOrigin, KernelPolicyInput, VerifierPolicy},
        selection::{
            CandidateSourceId, ProofEvidenceCandidate, ProofEvidenceSet, ProofWinnerClass,
            VcProofSelection, merge_artifact_proof_selections, select_winner,
        },
        status::{
            ArtifactPublicationGap, ArtifactStatusPublication, ProjectedProofStatus,
            ProofStatusProjectionInput as Input,
            check_source_algorithm_return_reflexivity as normal,
            project_source_algorithm_return_reflexivity_status as project, project_status,
            source_algorithm_return_obligation_identity as identity,
        },
    };
    use mizar_vc::{dependency_slice::DependencySliceCompleteness, vc_ir::VcStatus};

    let original = fs::read_to_string(config().workspace_root.join(
        "tests/miz/pass/algorithms/pass_proof_verification_algorithm_ensures_return_001.miz",
    ))
    .unwrap();
    let mut baseline_source = None;
    for (index, text, skip) in [
        (0, original.clone(), 0),
        (1, original.replace("terminating algorithm", "algorithm"), 0),
        (2, format!(":: 世代 neutral trivia\n\n{original}"), 0),
        (3, original.clone(), 7),
    ] {
        with_source_algorithm_identity_inputs(
            &text,
            "owner-test",
            "owners",
            skip,
            |source, typed, symbols, check| {
                if index == 0 {
                    baseline_source = Some(source.source_id());
                } else if index == 3 {
                    assert_ne!(Some(source.source_id()), baseline_source);
                }
                let before = format!("{source:?}{typed:?}{symbols:?}{check:?}");
                let core = mizar_core::elaborator::lower_source_algorithms(check).unwrap();
                for (policy_case, (snapshot, policy)) in [
                    (snapshot_id(0), VerifierPolicy::default()),
                    (snapshot_id(1), VerifierPolicy::default()),
                    (snapshot_id(0), VerifierPolicy::development()),
                ]
                .into_iter()
                .take(if index == 0 { 3 } else { 1 })
                .enumerate()
                {
                    let vcs = mizar_vc::generator::generate_source_algorithm_postconditions(
                        &core,
                        snapshot,
                        &GenerationSchemaVersion::new("mizar-vc-generation-step5c14-return-v1"),
                        &VcSchemaVersion::new("mizar-vc-vcset-step5c14-return-v1"),
                    )
                    .unwrap();
                    let [vc] = vcs.vcs() else {
                        panic!("one genuine return VC")
                    };
                    let old = identity(source, check, symbols, snapshot, vc.id).unwrap();
                    let checked =
                        normal(source, check.typed(), symbols, snapshot, vc.id, &policy).unwrap();
                    assert_eq!(checked.1.class, CandidatePolicyClass::DischargedBuiltin);
                    let target = checked.0.target_vc_fingerprint().sort_bytes();
                    let id_text = format!(
                        "source-algorithm-return-reflexivity-v1:{}",
                        target
                            .iter()
                            .map(|byte| format!("{byte:02x}"))
                            .collect::<String>()
                    );
                    if index == 0 && policy_case == 0 {
                        assert_eq!(
                            id_text,
                            "source-algorithm-return-reflexivity-v1:1200000020d70222e20484e59bc7ee4ef3e1eb0123df719e794ddd40c7a5d09674c21b8ded"
                        );
                    }
                    let input = KernelPolicyInput::from_kernel_result(
                        &checked.0,
                        KernelEvidenceOrigin::BuiltinDischarge,
                    );
                    let candidate = ProofEvidenceCandidate::from_trusted_kernel_input(
                        CandidateSourceId::new(id_text.clone()).unwrap(),
                        &input,
                    )
                    .unwrap();
                    let selected = select_winner(
                        &ProofEvidenceSet::new(target, None, policy.clone())
                            .with_candidates([candidate]),
                    );
                    assert_eq!(
                        selected.selected_class(),
                        ProofWinnerClass::DischargedBuiltin
                    );
                    let [row] = <[_; 1]>::try_from(
                        merge_artifact_proof_selections(
                            [],
                            [VcProofSelection::new(vc.id, selected.clone())],
                        )
                        .unwrap(),
                    )
                    .unwrap();
                    let expected =
                        project_status(Input::new(row, policy.clone(), old.0.clone())).unwrap();
                    let prior = (checked.clone(), policy.clone());
                    let actual = project(source, check, symbols, snapshot, vc.id, &policy).unwrap();
                    assert_eq!(actual, (expected, old.1.clone()));
                    assert_eq!(
                        actual.0.selected_class(),
                        ProofWinnerClass::DischargedBuiltin
                    );
                    assert_eq!(actual.0.projected_status(), ProjectedProofStatus::Accepted);
                    assert_eq!(
                        actual.0.artifact_publication(),
                        ArtifactStatusPublication::ExternalDependencyGap(
                            ArtifactPublicationGap::DischargedBuiltinWitnessUnsupported
                        )
                    );
                    let reuse = actual.0.reuse_metadata();
                    assert_eq!(reuse.selected_candidate_id().unwrap().as_str(), id_text);
                    assert!(reuse.selected_evidence_hash().is_some());
                    assert_eq!(reuse.selected_proof_witness_hash(), None);
                    assert_eq!(reuse.deterministic_discharge_hash(), None);
                    assert_eq!(reuse.selected_candidate_provenance_hash(), None);
                    assert_eq!(reuse.dependency_compatibility(), None);
                    assert_eq!(reuse.trusted_used_axioms_hash(), None);
                    assert_eq!(reuse.explanation_ref(), None);
                    assert!(!reuse.cache_reuse_predicate_complete());
                    assert_eq!(actual.1.status(), &VcStatus::Open);
                    assert!(!actual.1.unknowns().is_empty());
                    assert_eq!(
                        actual.1.completeness(),
                        DependencySliceCompleteness::IncompleteUncacheable
                    );
                    assert!(actual.1.requires_cache_miss());
                    assert_eq!(
                        actual,
                        project(source, check, symbols, snapshot, vc.id, &policy).unwrap()
                    );
                    assert_eq!((checked, policy), prior);
                }
                assert_eq!(format!("{source:?}{typed:?}{symbols:?}{check:?}"), before);
            },
        );
    }
}

#[test]
fn source_algorithm_return_status_preserves_upstream_errors_and_inputs() {
    use mizar_checker::type_checker::check_source_algorithm_types;
    use mizar_proof::{
        policy::VerifierPolicy,
        status::{
            check_source_algorithm_return_reflexivity as normal,
            project_source_algorithm_return_reflexivity_status as project,
            source_algorithm_return_canonical_inputs as canonical,
            source_algorithm_return_obligation_identity as identity,
        },
    };
    use mizar_vc::vc_ir::VcId;

    let root = config().workspace_root;
    let original = fs::read_to_string(root.join(
        "tests/miz/pass/algorithms/pass_proof_verification_algorithm_ensures_return_001.miz",
    ))
    .unwrap();
    let broader = fs::read_to_string(root.join(
        "tests/miz/pass/algorithms/pass_proof_verification_algorithm_var_const_assert_001.miz",
    ))
    .unwrap();
    let disabled = VerifierPolicy::default().with_kernel_evidence_formats([]);
    for (kind, text) in [
        ("absent", original.clone()),
        ("zero", original.replace("    ensures result = a\n", "")),
        ("broader", broader),
        (
            "two_returns",
            original
                .replace("    ensures result = a\n", "")
                .replace("return a;", "return a;\n    return a;"),
        ),
    ] {
        with_source_algorithm_identity_inputs(
            &text,
            "owner-test",
            "owners",
            0,
            |source, typed, symbols, check| {
                let before = format!("{source:?}{typed:?}{symbols:?}{check:?}");
                let snapshot = snapshot_id(0);
                let fresh = check_source_algorithm_types(source, check.typed(), symbols).unwrap();
                assert_eq!(fresh.algorithm(), check.algorithm());
                let core = mizar_core::elaborator::lower_source_algorithms(&fresh).unwrap();
                let generated = mizar_vc::generator::generate_source_algorithm_postconditions(
                    &core,
                    snapshot,
                    &GenerationSchemaVersion::new("mizar-vc-generation-step5c14-return-v1"),
                    &VcSchemaVersion::new("mizar-vc-vcset-step5c14-return-v1"),
                );
                let selected = if kind == "broader" {
                    generated.as_ref().unwrap().vcs().first().unwrap().id
                } else {
                    VcId::new(usize::MAX)
                };
                if kind == "zero" {
                    assert!(generated.as_ref().unwrap().vcs().is_empty());
                }
                if kind == "two_returns" {
                    assert!(generated.is_err());
                }
                let expected = canonical(&fresh, snapshot, selected).unwrap_err();
                if kind == "two_returns" {
                    assert_eq!(expected, generated.unwrap_err());
                }
                assert_eq!(
                    identity(source, check, symbols, snapshot, selected).unwrap_err(),
                    expected
                );
                assert_eq!(
                    project(source, check, symbols, snapshot, selected, &disabled).unwrap_err(),
                    expected
                );
                if kind == "absent" {
                    for changed in [
                        original.replace("idalgo", "foreign_algorithm"),
                        format!("{original}\n{original}"),
                    ] {
                        let (_, _, foreign) =
                            super::super::super::tests::source_registration_owner_inputs(
                                &changed,
                                "owner-test",
                                "owners",
                                0,
                            );
                        let actual =
                            check_source_algorithm_types(source, check.typed(), foreign.env())
                                .unwrap_err();
                        assert_eq!(
                            identity(source, check, foreign.env(), snapshot, selected).unwrap_err(),
                            actual
                        );
                        assert_eq!(
                            project(source, check, foreign.env(), snapshot, selected, &disabled)
                                .unwrap_err(),
                            actual
                        );
                    }
                }
                assert_eq!(format!("{source:?}{typed:?}{symbols:?}{check:?}"), before);
            },
        );
    }
    with_source_algorithm_identity_inputs(
        &original,
        "owner-test",
        "owners",
        0,
        |source, typed, symbols, check| {
            let before = format!("{source:?}{typed:?}{symbols:?}{check:?}");
            let snapshot = snapshot_id(0);
            let core = mizar_core::elaborator::lower_source_algorithms(check).unwrap();
            let vcs = mizar_vc::generator::generate_source_algorithm_postconditions(
                &core,
                snapshot,
                &GenerationSchemaVersion::new("mizar-vc-generation-step5c14-return-v1"),
                &VcSchemaVersion::new("mizar-vc-vcset-step5c14-return-v1"),
            )
            .unwrap();
            let [target] = vcs.vcs() else {
                panic!("one genuine return VC")
            };
            let old = identity(source, check, symbols, snapshot, target.id).unwrap();
            let expected = normal(
                source,
                check.typed(),
                symbols,
                snapshot,
                target.id,
                &disabled,
            )
            .unwrap_err();
            assert_eq!(
                project(source, check, symbols, snapshot, target.id, &disabled).unwrap_err(),
                expected
            );
            assert_eq!(
                old,
                identity(source, check, symbols, snapshot, target.id).unwrap()
            );
            assert_eq!(format!("{source:?}{typed:?}{symbols:?}{check:?}"), before);
        },
    );
}

fn with_direct_return_request_fixture(
    f: impl FnOnce(
        &mizar_resolve::resolved_ast::SurfaceResolvedArena,
        &mizar_checker::type_checker::SourceAlgorithmCheck<'_>,
        &mizar_resolve::env::SymbolEnv,
        &mizar_core::core_ir::CoreIr,
        &mizar_vc::vc_ir::VcSet,
    ),
) {
    let config = config();
    let plan = crate::harness::build_test_plan(&config).unwrap();
    let case = plan
        .cases
        .iter()
        .find(|case| case.id.0 == "pass_proof_verification_algorithm_ensures_return_001")
        .unwrap();
    let text = fs::read_to_string(&case.source_path).unwrap();
    let (source, typed, symbols) = super::super::super::source_registration_inputs(
        &config.workspace_root,
        case,
        super::super::super::formula_statement::step5c8_test_frontend(&text),
    )
    .unwrap();
    let check =
        mizar_checker::type_checker::check_source_algorithm_types(&source, &typed, &symbols)
            .unwrap();
    let core = mizar_core::elaborator::lower_source_algorithms(&check).unwrap();
    let vcs = mizar_vc::generator::generate_source_algorithm_direct_return_requests(
        &source,
        &check,
        &symbols,
        &core,
        snapshot_id(0),
        &GenerationSchemaVersion::new("direct-return-test-generation"),
        &VcSchemaVersion::new("direct-return-test-vc"),
    )
    .unwrap();
    f(&source, &check, &symbols, &core, &vcs);
}

fn direct_return_request_shape(
    vcs: &mizar_vc::vc_ir::VcSet,
    reference: mizar_vc::vc_ir::VcFormulaRef,
) -> &mizar_vc::vc_ir::VcGeneratedFormulaShape {
    let mizar_vc::vc_ir::VcFormulaRef::Generated(id) = reference else {
        panic!("request formula must be generated")
    };
    &vcs.generated_formula(id).unwrap().shape
}

#[test]
fn source_algorithm_direct_return_requests_preserve_authentic_output() {
    use mizar_core::control_flow::{build_control_flow_ir, build_obligation_seed_handoff};
    use mizar_vc::{
        dependency_slice::{DependencySliceInput, try_compute_dependency_slices},
        discharge::{DischargeInput, DischargePolicy, try_discharge},
        generator::{
            generate_source_algorithm_direct_return_requests,
            generate_source_algorithm_postconditions,
        },
        vc_ir::{
            AnchorOwner, ContextEntry, ContextEntryId, ContextEntryKind, ExpandedVcRef,
            ExpansionSchemaVersion, LocalContext, PremiseRef, SeedVcMapping, VcKind, VcProvenance,
            VcProvenancePhase, VcStatus,
        },
    };
    with_direct_return_request_fixture(|source, check, symbols, core, vcs| {
        let generation = &vcs.vcs()[0].anchor.generation_schema_version;
        let old = generate_source_algorithm_postconditions(
            core,
            vcs.snapshot(),
            generation,
            vcs.schema_version(),
        )
        .unwrap();
        assert_eq!(vcs.vcs().len(), 5);
        assert_eq!(vcs.vcs()[0], old.vcs()[0]);
        assert_eq!(vcs.generated_formulas()[0], old.generated_formulas()[0]);
        assert_eq!(
            (
                vcs.source(),
                vcs.module(),
                vcs.snapshot(),
                vcs.schema_version()
            ),
            (
                core.source_id(),
                old.module(),
                old.snapshot(),
                old.schema_version()
            )
        );
        let (algorithm_id, algorithm) = core.algorithms().iter().next().unwrap();
        let parameter = &algorithm.params[0];
        let result = algorithm.result.as_ref().unwrap();
        let statement = core
            .algorithm_statements()
            .get(algorithm.statements[0])
            .unwrap();
        let flow = build_control_flow_ir(core);
        let handoff = build_obligation_seed_handoff(core, &flow);
        let seed = &handoff.entries.get(old.vcs()[0].seed.handoff).unwrap().seed;
        let route = VcProvenance {
            phase: VcProvenancePhase::Generator,
            key: format!("source-algorithm-direct-return-v1:{}", generation.as_str()).into(),
            core: None,
        };
        let fresh_start = core
            .terms()
            .iter()
            .filter_map(|(_, term)| {
                if let mizar_core::core_ir::CoreTermKind::Var(var) = term.kind {
                    Some(var.index())
                } else {
                    None
                }
            })
            .chain([parameter.var.index(), result.var.index()])
            .max()
            .unwrap()
            + 1;
        let expected_kinds = [
            VcKind::AlgorithmE1Typing,
            VcKind::AlgorithmE1SingleValuedness,
            VcKind::AlgorithmE1ResultInhabitation,
            VcKind::AlgorithmE4NormalReturn,
        ];
        let slices = try_compute_dependency_slices(DependencySliceInput {
            vc_set: vcs,
            discharge_output: None,
        })
        .unwrap();
        let discharged = try_discharge(DischargeInput {
            vc_set: vcs,
            policy: &DischargePolicy::default(),
        })
        .unwrap();
        for (vc, kind) in vcs.vcs()[1..].iter().zip(expected_kinds) {
            assert_eq!(vc.kind, kind);
            let mut entries = old.vcs()[0].local_context.entries().to_vec();
            for index in 0..if kind == VcKind::AlgorithmE1SingleValuedness {
                2
            } else {
                1
            } {
                let id = ContextEntryId::new(entries.len());
                entries.push(ContextEntry {
                    id,
                    sort_key: format!("direct-return-result-{:08}", id.index()).into(),
                    kind: ContextEntryKind::BinderDeclaration {
                        var: mizar_core::core_ir::CoreVarId::new(fresh_start + index),
                        role: "result".into(),
                    },
                    formula: None,
                    provenance: vec![route.clone()],
                });
            }
            assert_eq!(
                vc.local_context,
                LocalContext::try_new(entries, old.vcs()[0].local_context.policy_inputs().to_vec())
                    .unwrap()
            );
            assert_eq!(
                vc.premises,
                vec![PremiseRef::LocalContext(ContextEntryId::new(0))]
            );
            assert_eq!(vc.seed, old.vcs()[0].seed);
            let mut source = old.vcs()[0].source.clone();
            source.related.extend([
                algorithm.source.clone(),
                parameter.source.clone(),
                result.source.clone(),
                statement.source.clone(),
                core.formulas()
                    .get(parameter.ty_guard.unwrap())
                    .unwrap()
                    .source
                    .clone(),
                core.formulas()
                    .get(result.ty_guard.unwrap())
                    .unwrap()
                    .source
                    .clone(),
            ]);
            assert_eq!(vc.source, source);
            let mut provenance = old.vcs()[0].provenance.clone();
            provenance.push(route.clone());
            assert_eq!(vc.provenance, provenance);
            assert_eq!(vc.proof_hint, old.vcs()[0].proof_hint);
            assert_eq!(vc.anchor.owner, AnchorOwner::Algorithm(algorithm_id));
            assert_eq!(vc.anchor.kind, kind);
            assert_eq!(vc.anchor.local_path, seed.local_path);
            assert_eq!(vc.anchor.semantic_origin, seed.semantic_origin);
            assert_eq!(vc.anchor.source_range, old.vcs()[0].anchor.source_range);
            assert_eq!(vc.anchor.label, old.vcs()[0].anchor.label);
            assert_eq!(vc.anchor.provenance, old.vcs()[0].anchor.provenance);
            assert_eq!(
                vc.anchor.canonical_goal_hash,
                old.vcs()[0].anchor.canonical_goal_hash
            );
            assert_eq!(
                vc.anchor.canonical_context_hash,
                old.vcs()[0].anchor.canonical_context_hash
            );
            assert_eq!(vc.anchor.completeness, old.vcs()[0].anchor.completeness);
            assert_eq!(vc.anchor.generation_schema_version, *generation);
            assert!(!vc.anchor.is_complete());
            assert_ne!(
                vc.anchor.source_shape_hash,
                old.vcs()[0].anchor.source_shape_hash
            );
            assert_eq!(vc.status, VcStatus::Open);
            assert_eq!(
                discharged.vc_set().vc(vc.id).unwrap().status,
                VcStatus::NeedsAtp
            );
            let slice = slices.slice_for(vc.id).unwrap();
            assert!(slice.requires_cache_miss());
            assert!(!slice.unknowns().is_empty());
            assert!(slices.proof_reuse_key_for(&discharged, vc.id).is_none());
        }
        assert_eq!(vcs.generated_formulas().len(), 12);
        for formula in &vcs.generated_formulas()[1..] {
            assert_eq!(
                formula.kind,
                mizar_vc::vc_ir::VcGeneratedFormulaKind::SplitGoal
            );
            assert_eq!(formula.provenance, vec![route.clone()]);
        }
        let mut accounting = old.seed_accounting().to_vec();
        accounting
            .iter_mut()
            .find(|row| row.handoff == old.vcs()[0].seed.handoff)
            .unwrap()
            .mapping = SeedVcMapping::Expanded {
            vcs: vcs
                .vcs()
                .iter()
                .enumerate()
                .map(|(expansion_index, vc)| ExpandedVcRef {
                    expansion_index,
                    vc: vc.id,
                })
                .collect(),
            expansion_schema: ExpansionSchemaVersion::new("source-algorithm-direct-return-v1"),
        };
        assert_eq!(vcs.seed_accounting(), accounting);
        let termination = vcs
            .seed_accounting()
            .iter()
            .find(|row| row.handoff != old.vcs()[0].seed.handoff)
            .unwrap();
        assert_eq!(
            termination.seed_status,
            mizar_core::core_ir::ObligationSeedStatus::Deferred
        );
        assert!(matches!(
            termination.mapping,
            SeedVcMapping::NoConcreteVc { .. }
        ));
        assert_eq!(
            *vcs,
            generate_source_algorithm_direct_return_requests(
                source,
                check,
                symbols,
                core,
                vcs.snapshot(),
                generation,
                vcs.schema_version(),
            )
            .unwrap()
        );
    });
}

#[test]
fn source_algorithm_direct_return_typing_preserves_guards_and_refusals() {
    use mizar_core::core_ir::{CoreAlgorithmStmtKind, CoreFormulaKind, CoreIr, CoreIrParts};
    use mizar_vc::{
        generator::generate_source_algorithm_direct_return_requests,
        vc_ir::{
            ContextEntryKind, QuantifierKind, VcGeneratedFormulaShape as Shape, VcKind,
            VcProgramValue,
        },
    };
    with_direct_return_request_fixture(|source, check, symbols, core, vcs| {
        let (algorithm_id, algorithm) = core.algorithms().iter().next().unwrap();
        let parameter = &algorithm.params[0];
        let result = algorithm.result.as_ref().unwrap();
        let request = vcs
            .vcs()
            .iter()
            .find(|vc| vc.kind == VcKind::AlgorithmE1Typing)
            .unwrap();
        let Shape::Quantified {
            kind: QuantifierKind::Forall,
            binders,
            body,
        } = direct_return_request_shape(vcs, request.goal)
        else {
            panic!("universal typing")
        };
        let [binder] = binders.as_slice() else {
            panic!("one result binder")
        };
        let ContextEntryKind::BinderDeclaration { var, .. } =
            request.local_context.entries()[binder.index()].kind
        else {
            panic!("result declaration")
        };
        let candidate = VcProgramValue {
            var,
            definition: None,
        };
        let Shape::Implies {
            premise,
            conclusion,
        } = direct_return_request_shape(vcs, *body)
        else {
            panic!("return implies type")
        };
        assert_eq!(
            direct_return_request_shape(vcs, *conclusion),
            &Shape::ProgramTypePredicate {
                subject: candidate,
                ty: "object".into(),
            }
        );
        assert_eq!(
            direct_return_request_shape(vcs, *premise),
            &Shape::AlgorithmReturns {
                algorithm: algorithm_id,
                flow: mizar_core::control_flow::build_control_flow_ir(core).flow_map[&algorithm_id],
                statement: algorithm.statements[0],
                input: VcProgramValue {
                    var: parameter.var,
                    definition: None
                },
                result: candidate,
            }
        );
        for mutation in 0..8 {
            let mut parts = CoreIrParts {
                source_id: core.source_id(),
                module_id: core.module_id().clone(),
                items: core.items().clone(),
                terms: core.terms().clone(),
                formulas: core.formulas().clone(),
                definitions: core.definitions().clone(),
                proofs: core.proofs().clone(),
                proof_nodes: core.proof_nodes().clone(),
                algorithms: core.algorithms().clone(),
                algorithm_statements: core.algorithm_statements().clone(),
                generated: core.generated().clone(),
                obligation_seeds: core.obligation_seeds().clone(),
                source_map: core.source_map().clone(),
                diagnostics: core.diagnostics().clone(),
            };
            match mutation {
                0 => parts.algorithms.get_mut(algorithm_id).unwrap().params[0].ty_guard = None,
                1 => {
                    parts
                        .algorithms
                        .get_mut(algorithm_id)
                        .unwrap()
                        .result
                        .as_mut()
                        .unwrap()
                        .ty_guard = None
                }
                2 => {
                    parts.algorithms.get_mut(algorithm_id).unwrap().params[0].ty_guard =
                        result.ty_guard
                }
                3 => {
                    parts
                        .algorithms
                        .get_mut(algorithm_id)
                        .unwrap()
                        .result
                        .as_mut()
                        .unwrap()
                        .ty_guard = parameter.ty_guard
                }
                4 => {
                    let guard = parts.formulas.get_mut(parameter.ty_guard.unwrap()).unwrap();
                    guard.source = algorithm.source.clone();
                    parts
                        .source_map
                        .formula_sources
                        .insert(parameter.ty_guard.unwrap(), algorithm.source.clone());
                }
                5 => {
                    parts
                        .formulas
                        .get_mut(result.ty_guard.unwrap())
                        .unwrap()
                        .kind = CoreFormulaKind::True
                }
                6 => {
                    parts
                        .algorithm_statements
                        .get_mut(algorithm.statements[0])
                        .unwrap()
                        .kind = CoreAlgorithmStmtKind::Return(None)
                }
                _ => {
                    let term = core.terms().iter().find(|(_, term)| {
                        matches!(term.kind, mizar_core::core_ir::CoreTermKind::Var(var) if var == result.var)
                    }).unwrap().0;
                    parts
                        .algorithm_statements
                        .get_mut(algorithm.statements[0])
                        .unwrap()
                        .kind = CoreAlgorithmStmtKind::Return(Some(term));
                }
            }
            let changed = CoreIr::try_new(parts).unwrap();
            assert!(
                generate_source_algorithm_direct_return_requests(
                    source,
                    check,
                    symbols,
                    &changed,
                    vcs.snapshot(),
                    &request.anchor.generation_schema_version,
                    vcs.schema_version(),
                )
                .is_err(),
                "Core mutation {mutation}"
            );
        }
    });
}

#[test]
fn source_algorithm_direct_return_uniqueness_binds_distinct_results() {
    use mizar_vc::vc_ir::{
        ContextEntryKind, QuantifierKind, VcGeneratedFormulaShape as Shape, VcKind, VcProgramValue,
    };
    with_direct_return_request_fixture(|_, _, _, core, vcs| {
        let request = vcs
            .vcs()
            .iter()
            .find(|vc| vc.kind == VcKind::AlgorithmE1SingleValuedness)
            .unwrap();
        let Shape::Quantified {
            kind: QuantifierKind::Forall,
            binders,
            body,
        } = direct_return_request_shape(vcs, request.goal)
        else {
            panic!("universal uniqueness")
        };
        let candidates = binders.iter().map(|id| {
            let entry = &request.local_context.entries()[id.index()];
            assert_eq!(entry.formula, None);
            let ContextEntryKind::BinderDeclaration { var, .. } = entry.kind else { panic!("declaration") };
            assert!(core.terms().iter().all(|(_, term)| {
                !matches!(term.kind, mizar_core::core_ir::CoreTermKind::Var(original) if original == var)
            }));
            VcProgramValue { var, definition: None }
        }).collect::<Vec<_>>();
        let [r, s] = candidates.as_slice() else {
            panic!("two result binders")
        };
        assert_ne!(r, s);
        let Shape::Implies {
            premise,
            conclusion,
        } = direct_return_request_shape(vcs, *body)
        else {
            panic!("two returns imply equality")
        };
        assert_eq!(
            direct_return_request_shape(vcs, *conclusion),
            &Shape::ProgramEquals {
                left: *r,
                right: *s
            }
        );
        let Shape::And(relations) = direct_return_request_shape(vcs, *premise) else {
            panic!("two executions")
        };
        let [first, second] = relations.as_slice() else {
            panic!("only execution premises")
        };
        let Shape::AlgorithmReturns {
            algorithm,
            flow,
            statement,
            input,
            result,
        } = direct_return_request_shape(vcs, *first)
        else {
            panic!("source relation")
        };
        assert_eq!(*result, *r);
        let (expected_id, expected) = core.algorithms().iter().next().unwrap();
        assert_eq!(
            (*algorithm, *flow, *statement, *input),
            (
                expected_id,
                mizar_core::control_flow::build_control_flow_ir(core).flow_map[&expected_id],
                expected.statements[0],
                VcProgramValue {
                    var: expected.params[0].var,
                    definition: None
                },
            )
        );
        assert_eq!(
            direct_return_request_shape(vcs, *second),
            &Shape::AlgorithmReturns {
                algorithm: *algorithm,
                flow: *flow,
                statement: *statement,
                input: *input,
                result: *s,
            }
        );
        assert_eq!(request.premises, vcs.vcs()[0].premises);
        assert_eq!(request.proof_hint, None);
    });
}

#[test]
fn source_algorithm_direct_return_inhabitation_has_only_typed_input() {
    use mizar_vc::vc_ir::{
        ContextEntryKind, QuantifierKind, VcFormulaRef, VcGeneratedFormulaShape as Shape, VcKind,
        VcProgramValue,
    };
    with_direct_return_request_fixture(|_, _, _, core, vcs| {
        let algorithm = core.algorithms().iter().next().unwrap().1;
        assert!(algorithm.contracts.requires.is_empty());
        let request = vcs
            .vcs()
            .iter()
            .find(|vc| vc.kind == VcKind::AlgorithmE1ResultInhabitation)
            .unwrap();
        let Shape::Quantified {
            kind: QuantifierKind::Exists,
            binders,
            body,
        } = direct_return_request_shape(vcs, request.goal)
        else {
            panic!("all-input inhabitation")
        };
        let [binder] = binders.as_slice() else {
            panic!("one existential result")
        };
        let ContextEntryKind::BinderDeclaration { var, .. } =
            request.local_context.entries()[binder.index()].kind
        else {
            panic!("result declaration")
        };
        assert_eq!(
            direct_return_request_shape(vcs, *body),
            &Shape::ProgramTypePredicate {
                subject: VcProgramValue {
                    var,
                    definition: None
                },
                ty: "object".into(),
            }
        );
        assert_ne!(var, algorithm.params[0].var);
        assert_eq!(
            request.local_context.entries()[0].formula,
            Some(VcFormulaRef::Core(algorithm.params[0].ty_guard.unwrap()))
        );
        assert_eq!(request.local_context.entries().len(), 2);
        assert_eq!(request.premises, vcs.vcs()[0].premises);
        assert_eq!(request.proof_hint, None);
        assert_eq!(request.status, mizar_vc::vc_ir::VcStatus::Open);
    });
}

#[test]
fn source_algorithm_direct_return_e4_is_independent_and_replays_inputs() {
    use mizar_vc::{
        generator::generate_source_algorithm_direct_return_requests,
        vc_ir::{QuantifierKind, VcGeneratedFormulaShape as Shape, VcKind, VcStatus},
    };
    with_direct_return_request_fixture(|source, check, symbols, core, vcs| {
        let request = vcs
            .vcs()
            .iter()
            .find(|vc| vc.kind == VcKind::AlgorithmE4NormalReturn)
            .unwrap();
        let Shape::Quantified {
            kind: QuantifierKind::Exists,
            binders,
            body,
        } = direct_return_request_shape(vcs, request.goal)
        else {
            panic!("independent return existential")
        };
        let [binder] = binders.as_slice() else {
            panic!("one normal-return result")
        };
        let mizar_vc::vc_ir::ContextEntryKind::BinderDeclaration { var, .. } =
            request.local_context.entries()[binder.index()].kind
        else {
            panic!("result declaration")
        };
        let (algorithm_id, algorithm) = core.algorithms().iter().next().unwrap();
        assert_eq!(
            direct_return_request_shape(vcs, *body),
            &Shape::AlgorithmReturns {
                algorithm: algorithm_id,
                flow: mizar_core::control_flow::build_control_flow_ir(core).flow_map[&algorithm_id],
                statement: algorithm.statements[0],
                input: mizar_vc::vc_ir::VcProgramValue {
                    var: algorithm.params[0].var,
                    definition: None
                },
                result: mizar_vc::vc_ir::VcProgramValue {
                    var,
                    definition: None
                },
            }
        );
        assert_eq!(request.premises, vcs.vcs()[0].premises);
        assert_eq!(request.proof_hint, None);
        for (snapshot, generation, schema) in [
            (
                snapshot_id(1),
                request.anchor.generation_schema_version.clone(),
                vcs.schema_version().clone(),
            ),
            (
                vcs.snapshot(),
                GenerationSchemaVersion::new("changed-generation"),
                vcs.schema_version().clone(),
            ),
            (
                vcs.snapshot(),
                request.anchor.generation_schema_version.clone(),
                VcSchemaVersion::new("changed-vc"),
            ),
        ] {
            let changed = generate_source_algorithm_direct_return_requests(
                source,
                check,
                symbols,
                core,
                snapshot,
                &generation,
                &schema,
            )
            .unwrap();
            assert_ne!(*vcs, changed);
            assert!(changed.vcs().iter().all(|vc| vc.status == VcStatus::Open));
        }
        let original = fs::read_to_string(config().workspace_root.join(
            "tests/miz/pass/algorithms/pass_proof_verification_algorithm_ensures_return_001.miz",
        ))
        .unwrap();
        for text in [
            format!(":: stale source placement\n{original}"),
            original.replace("idalgo", "foreign_algorithm"),
            original.replace("terminating algorithm", "algorithm"),
            original.replace("return a;", "const copy := a;\n    return copy;"),
        ] {
            with_source_algorithm_identity_inputs(
                &text,
                "owner-test",
                "owners",
                0,
                |foreign_source, _, foreign_symbols, foreign_check| {
                    let foreign_core =
                        mizar_core::elaborator::lower_source_algorithms(foreign_check).unwrap();
                    for (source, check, symbols, core) in [
                        (foreign_source, check, symbols, core),
                        (source, foreign_check, symbols, core),
                        (source, check, foreign_symbols, core),
                        (source, check, symbols, &foreign_core),
                    ] {
                        assert!(
                            generate_source_algorithm_direct_return_requests(
                                source,
                                check,
                                symbols,
                                core,
                                vcs.snapshot(),
                                &request.anchor.generation_schema_version,
                                vcs.schema_version(),
                            )
                            .is_err()
                        );
                    }
                    if !foreign_check.declared_terminating() || text.contains("const copy") {
                        assert!(
                            generate_source_algorithm_direct_return_requests(
                                foreign_source,
                                foreign_check,
                                foreign_symbols,
                                &foreign_core,
                                vcs.snapshot(),
                                &request.anchor.generation_schema_version,
                                vcs.schema_version(),
                            )
                            .is_err()
                        );
                    }
                },
            );
        }
    });
}

// Extend the independently collected structural remainder, then replay its hash frame.
// Debug exposes existing normalized payloads without widening the production API.
fn source_algorithm_direct_return_slice_oracle(
    source: &mizar_resolve::resolved_ast::SurfaceResolvedArena,
    check: &mizar_checker::type_checker::SourceAlgorithmCheck<'_>,
    symbols: &mizar_resolve::env::SymbolEnv,
    core: &mizar_core::core_ir::CoreIr,
    vcs: &mizar_vc::vc_ir::VcSet,
    generation: &GenerationSchemaVersion,
) -> mizar_vc::dependency_slice::DependencySliceSet {
    use mizar_vc::{
        dependency_slice::{
            DependencyEntryClass, DependencySliceInput, try_compute_dependency_slices,
            try_compute_source_algorithm_direct_return_dependency_slices,
        },
        vc_ir::{ContextEntryKind, VcGeneratedFormulaShape},
    };
    use std::fmt::Write as _;
    let plain = try_compute_dependency_slices(DependencySliceInput {
        vc_set: vcs,
        discharge_output: None,
    })
    .unwrap();
    let actual = try_compute_source_algorithm_direct_return_dependency_slices(
        source, check, symbols, core, vcs, generation,
    )
    .unwrap();
    let content = mizar_session::lower_hex(
        &mizar_core::elaborator::source_algorithm_core_semantic_payload(check, core).unwrap(),
    );
    for vc in vcs.vcs() {
        let old = plain.slice_for(vc.id).unwrap();
        let new = actual.slice_for(vc.id).unwrap();
        let binders = vc
            .local_context
            .entries()
            .iter()
            .filter_map(|entry| match &entry.kind {
                ContextEntryKind::BinderDeclaration { var, role } if role.as_str() == "result" => {
                    Some(*var)
                }
                _ => None,
            })
            .collect::<Vec<_>>();
        let mut additions = Vec::new();
        for formula in vcs.generated_formulas() {
            if !old.entries().iter().any(|entry| {
                entry.local_key() == format!("generated-formula:{}:payload", formula.id.index())
            }) {
                continue;
            }
            if let VcGeneratedFormulaShape::AlgorithmReturns { result, .. } = &formula.shape {
                let ordinal = binders.iter().position(|var| *var == result.var).unwrap();
                additions.push((
                    format!("source-direct-return-core:{}", formula.id.index()),
                    format!("source-direct-return-core-v1; request={:?}; input=parameter; result-binder={ordinal}; content={content}", vc.kind),
                ));
            }
        }
        let remainder = new
            .entries()
            .iter()
            .filter(|entry| !entry.local_key().starts_with("source-direct-return-core:"))
            .collect::<Vec<_>>();
        assert_eq!(remainder, old.entries().iter().collect::<Vec<_>>());
        assert_eq!(new.unknowns(), old.unknowns());
        assert_eq!(
            (new.vc(), new.kind(), new.status(), new.completeness()),
            (old.vc(), old.kind(), old.status(), old.completeness())
        );
        let mut observed = new
            .entries()
            .iter()
            .filter(|entry| entry.local_key().starts_with("source-direct-return-core:"))
            .map(|entry| {
                assert_eq!(entry.class(), DependencyEntryClass::GeneratedFormula);
                (entry.local_key().to_owned(), entry.payload().to_owned())
            })
            .collect::<Vec<_>>();
        observed.sort();
        additions.sort();
        assert_eq!(observed, additions);
        let mut payloads = old
            .entries()
            .iter()
            .map(|entry| {
                let debug = format!("{entry:#?}");
                let encoded = debug
                    .lines()
                    .find_map(|line| line.trim().strip_prefix("fingerprint_payload: "))
                    .unwrap()
                    .trim_end_matches(',');
                (
                    entry.class(),
                    serde_json::from_str::<String>(encoded).unwrap(),
                )
            })
            .collect::<Vec<_>>();
        payloads.extend(
            additions
                .into_iter()
                .map(|(_, payload)| (DependencyEntryClass::GeneratedFormula, payload)),
        );
        payloads.sort();
        let mut frame = format!(
            "dependency-slice-fingerprint-v1\nschema-version: mizar-vc-dependency-slice-v1\nkind: {:?}\nstatus: Open\ncompleteness: {:?}\n[entries]\n",
            vc.kind,
            old.completeness()
        );
        for (class, payload) in payloads {
            writeln!(frame, "{class:?}\t{payload}").unwrap();
        }
        frame.push_str("[unknowns]\n");
        let mut unknowns = old
            .unknowns()
            .iter()
            .map(|unknown| (unknown.family(), unknown.reason()))
            .collect::<Vec<_>>();
        unknowns.sort();
        for (family, reason) in unknowns {
            writeln!(frame, "{family:?}\t{reason}").unwrap();
        }
        let mut lanes = [
            0x6d697a61722d7663_u64,
            0x6465702d736c6963,
            0x652d66696e676572,
            0x7072696e742d7631,
        ];
        for (index, byte) in frame.bytes().enumerate() {
            let lane = index % 4;
            lanes[lane] ^= u64::from(byte)
                .wrapping_add(0x9e3779b97f4a7c15)
                .wrapping_add((index as u64).rotate_left(lane as u32 + 1));
            lanes[lane] = lanes[lane]
                .rotate_left(11 + lane as u32)
                .wrapping_mul(0x1000000001b3);
        }
        lanes[0] ^= frame.len() as u64;
        lanes[1] ^= (frame.len() as u64).rotate_left(17);
        lanes[2] ^= lanes[0].rotate_left(7);
        lanes[3] ^= lanes[1].rotate_left(13);
        assert_eq!(
            mizar_session::lower_hex(new.fingerprint().hash().as_bytes()),
            lanes
                .iter()
                .map(|lane| format!("{lane:016x}"))
                .collect::<String>()
        );
    }
    actual
}

// Independent oracle for status.md#source-algorithm-direct-return-request-identity.
use mizar_vc::vc_ir::VcId;

fn source_algorithm_direct_return_reference(
    source: &mizar_resolve::resolved_ast::SurfaceResolvedArena,
    check: &mizar_checker::type_checker::SourceAlgorithmCheck<'_>,
    symbols: &mizar_resolve::env::SymbolEnv,
    snapshot: mizar_session::BuildSnapshotId,
    request: usize,
) -> (
    mizar_proof::status::ProofObligationIdentity,
    mizar_session::Hash,
    mizar_session::Hash,
    mizar_session::Hash,
    mizar_vc::dependency_slice::DependencySlice,
) {
    use mizar_artifact::store::{
        CanonicalHashDomain, CanonicalJson as J, HashClass, SchemaVersion,
    };
    use mizar_session::lower_hex;
    let generation = "mizar-vc-generation-vc53-direct-return-v1";
    let schema = "mizar-vc-vcset-vc53-direct-return-v1";
    let core = mizar_core::elaborator::lower_source_algorithms(check).unwrap();
    let vcs = mizar_vc::generator::generate_source_algorithm_direct_return_requests(
        source,
        check,
        symbols,
        &core,
        snapshot,
        &GenerationSchemaVersion::new(generation),
        &VcSchemaVersion::new(schema),
    )
    .unwrap();
    let vc = &vcs.vcs()[request + 1];
    let role = [
        "e1-typing",
        "e1-single-valuedness",
        "e1-inhabitation",
        "e4-normal-return",
    ][request];
    let semantic =
        mizar_core::elaborator::source_algorithm_core_semantic_payload(check, &core).unwrap();
    let parameter = &core.algorithms().iter().next().unwrap().1.params[0];
    let guard = parameter.ty_guard.unwrap();
    let formulas =
        mizar_core::elaborator::source_algorithm_formula_payloads(check, &core, &[guard]).unwrap();
    let hash = |domain: &str, object: J| {
        CanonicalHashDomain::new(
            HashClass::Interface,
            format!("mizar-proof/source-algorithm-direct-return/{domain}"),
            SchemaVersion::new(1, 0),
        )
        .hash(&object, &[])
    };
    let goal = hash(
        "goal",
        J::object([
            ("generation_schema", J::string(generation)),
            ("vc_schema", J::string(schema)),
            ("role", J::string(role)),
            ("core_semantics", J::string(lower_hex(&semantic))),
            ("terminating", J::string("true")),
        ])
        .unwrap(),
    );
    let context = hash(
        "context",
        J::object([
            ("generation_schema", J::string(generation)),
            ("vc_schema", J::string(schema)),
            ("guard", J::string(lower_hex(&formulas[&guard]))),
            ("binders", J::string(if request == 1 { "r,s" } else { "r" })),
            (
                "policy_inputs",
                J::array(vc.local_context.policy_inputs().iter().map(|input| {
                    J::object([
                        ("sort_key", J::string(input.sort_key.as_str())),
                        ("key", J::string(input.key.as_str())),
                        ("value", J::string(input.value.as_str())),
                    ])
                    .unwrap()
                })),
            ),
        ])
        .unwrap(),
    );
    let fingerprint = hash(
        "vc",
        J::object([
            ("role", J::string(role)),
            ("goal", J::string(lower_hex(goal.as_bytes()))),
            ("context", J::string(lower_hex(context.as_bytes()))),
        ])
        .unwrap(),
    );
    let slices = source_algorithm_direct_return_slice_oracle(
        source,
        check,
        symbols,
        &core,
        &vcs,
        &GenerationSchemaVersion::new(generation),
    );
    let slice = slices.slice_for(vc.id).unwrap().clone();
    let syntax =
        mizar_resolve::symbols::source_algorithm_syntax_payloads_from_arena(source, symbols)
            .unwrap();
    let entry = symbols
        .symbols()
        .iter()
        .find(|entry| entry.symbol() == &check.algorithm().1)
        .unwrap();
    let owner = J::object([
        (
            "package_id",
            J::string(entry.symbol().module().package().as_str()),
        ),
        (
            "module_path",
            J::string(entry.symbol().module().path().as_str()),
        ),
        ("namespace", J::string(entry.namespace().as_str())),
        ("spelling", J::string(entry.primary_spelling())),
        ("kind", J::string("algorithm")),
        ("visibility", J::string("public")),
    ])
    .unwrap();
    let candidate = hash(
        "candidate",
        J::object([
            ("owner", owner),
            (
                "syntax",
                J::string(lower_hex(&syntax[&check.algorithm().1])),
            ),
            ("core_semantics", J::string(lower_hex(&semantic))),
            ("vc", J::string(lower_hex(fingerprint.as_bytes()))),
            ("goal", J::string(lower_hex(goal.as_bytes()))),
            ("context", J::string(lower_hex(context.as_bytes()))),
        ])
        .unwrap(),
    );
    let anchor = format!(
        "source-algorithm-direct-return-candidate-anchor-v1:{}",
        lower_hex(candidate.as_bytes())
    );
    let obligation = hash(
        "obligation",
        J::object([
            ("obligation_anchor", J::string(&anchor)),
            (
                "vc_fingerprint",
                J::string(lower_hex(fingerprint.as_bytes())),
            ),
            ("canonical_goal_hash", J::string(lower_hex(goal.as_bytes()))),
            (
                "local_context_fingerprint",
                J::string(lower_hex(context.as_bytes())),
            ),
            (
                "vc_local_dependency_slice_fingerprint",
                J::string(lower_hex(slice.fingerprint().hash().as_bytes())),
            ),
        ])
        .unwrap(),
    );
    let identity = mizar_proof::status::ProofObligationIdentity::new(
        &anchor,
        mizar_proof::status::ObligationAnchor::new(&anchor).unwrap(),
        obligation,
        fingerprint,
        context,
        slice.fingerprint().hash(),
    )
    .unwrap();
    (identity, fingerprint, goal, context, slice)
}

#[test]
fn source_algorithm_direct_return_canonical_inputs_match_four_role_oracle() {
    use mizar_proof::status::source_algorithm_direct_return_canonical_inputs as canonical;
    use mizar_vc::{dependency_slice::DependencySliceCompleteness, vc_ir::VcStatus};
    with_direct_return_request_fixture(|source, check, symbols, core, vcs| {
        let before = format!("{source:?}{check:?}{symbols:?}{core:?}{vcs:?}");
        assert!(
            vcs.seed_accounting()
                .iter()
                .any(|entry| entry.seed_status
                    == mizar_core::core_ir::ObligationSeedStatus::Deferred)
        );
        for request in 0..4 {
            let (_, vc, goal, context, slice) = source_algorithm_direct_return_reference(
                source,
                check,
                symbols,
                snapshot_id(0),
                request,
            );
            let selected = vcs.vcs()[request + 1].id;
            let expected = (vc, goal, context, slice);
            assert_eq!(vcs.vcs()[request + 1].status, VcStatus::Open);
            assert_eq!(expected.3.status(), &VcStatus::Open);
            assert_eq!(
                expected.3.completeness(),
                DependencySliceCompleteness::IncompleteUncacheable
            );
            assert!(!expected.3.entries().is_empty());
            assert!(!expected.3.unknowns().is_empty());
            assert!(expected.3.requires_cache_miss());
            assert_eq!(
                canonical(source, check, symbols, snapshot_id(0), selected).unwrap(),
                expected
            );
            assert_eq!(
                canonical(source, check, symbols, snapshot_id(0), selected).unwrap(),
                expected
            );
        }
        assert_eq!(
            format!("{source:?}{check:?}{symbols:?}{core:?}{vcs:?}"),
            before
        );
    });
}

#[test]
fn source_algorithm_direct_return_identity_matches_independent_vectors() {
    use mizar_proof::status::source_algorithm_direct_return_obligation_identity as identity;
    let text = fs::read_to_string(config().workspace_root.join(
        "tests/miz/pass/algorithms/pass_proof_verification_algorithm_ensures_return_001.miz",
    ))
    .unwrap();
    // Independently replayed from complete structural frames and existing Core bytes;
    // original relation roles bind semantic content. Candidates/inhabitation are unchanged.
    let vectors = [
        (
            "1abb18aa8b523d60011cfce3b5ac46127368313ee8f653e65f38fa8456c77421",
            "c00daa8bf9aa04b1bc5a7e25d3a0edd66f0d4ecb0e433ee20adcdcff99729623",
        ),
        (
            "278b32210a82cf79c4cb1d076fdfa0c6094aa9e8e3498afe4418f462be74854d",
            "2ce71f491bd4c5f407991749f4b0f729489a255e4581c3debbe0d9c6850564cc",
        ),
        (
            "2bf9cf51be1f984735819dc975b146e73d66e967d4fdeb71a2382769f745b61b",
            "c6c4017ec23460039b504e324d5720a940d958512da5a15e2e87543c3177aa28",
        ),
        (
            "5f0f0d3bde16a1a539b3acda9f5997fc6c9bda946806de691e79f3ceb4c956b4",
            "d55a8f13acd255de8fd10ffd8e59062dd1ff53ee37460f6846fc1e8ebbc63f92",
        ),
    ];
    with_source_algorithm_identity_inputs(
        &text,
        "owner-test",
        "owners",
        0,
        |source, typed, symbols, check| {
            let before = format!("{source:?}{typed:?}{symbols:?}{check:?}");
            for (request, (candidate_hex, obligation_hex)) in vectors.into_iter().enumerate() {
                let (expected, _, _, _, slice) = source_algorithm_direct_return_reference(
                    source,
                    check,
                    symbols,
                    snapshot_id(0),
                    request,
                );
                assert_eq!(
                    expected.obligation_anchor().as_str(),
                    format!("source-algorithm-direct-return-candidate-anchor-v1:{candidate_hex}")
                );
                assert_eq!(
                    mizar_session::lower_hex(expected.obligation_fingerprint().as_bytes()),
                    obligation_hex
                );
                let selected = VcId::new(request + 1);
                let actual = identity(source, check, symbols, snapshot_id(0), selected).unwrap();
                assert_eq!(actual, (expected.clone(), slice.clone()));
                assert_eq!(
                    identity(source, check, symbols, snapshot_id(0), selected).unwrap(),
                    actual
                );
            }
            assert_eq!(format!("{source:?}{typed:?}{symbols:?}{check:?}"), before);
        },
    );
}

#[test]
fn source_algorithm_direct_return_identity_preserves_rebuilt_associations() {
    use mizar_proof::status::{
        source_algorithm_direct_return_canonical_inputs as canonical,
        source_algorithm_direct_return_obligation_identity as identity,
    };
    let original = fs::read_to_string(config().workspace_root.join(
        "tests/miz/pass/algorithms/pass_proof_verification_algorithm_ensures_return_001.miz",
    ))
    .unwrap();
    let parameter = original
        .replace(" a ", " param ")
        .replace("(a)", "(param)")
        .replace(" a;", " param;")
        .replace("= a\n", "= param\n");
    let mut baseline = None;
    let mut identities = Vec::new();
    for (label, text, package, module, skip) in [
        ("original", original.clone(), "owner-test", "owners", 0),
        (
            "trivia",
            format!(":: neutral trivia\n\n{original}"),
            "owner-test",
            "owners",
            0,
        ),
        ("relocation", original.clone(), "owner-test", "owners", 7),
        ("package", original.clone(), "other-package", "owners", 0),
        ("module", original.clone(), "owner-test", "nested.owners", 0),
        (
            "owner",
            original.replace("idalgo", "renamed_algorithm"),
            "owner-test",
            "owners",
            0,
        ),
        ("parameter", parameter, "owner-test", "owners", 0),
        (
            "semantics",
            original.replace("ensures result = a", "ensures a = result"),
            "owner-test",
            "owners",
            0,
        ),
    ] {
        with_source_algorithm_identity_inputs(
            &text,
            package,
            module,
            skip,
            |source, typed, symbols, check| {
                let before = format!("{source:?}{typed:?}{symbols:?}{check:?}");
                if label == "relocation" {
                    assert_ne!(source.source_id(), baseline.unwrap());
                }
                if label == "original" {
                    baseline = Some(source.source_id());
                }
                for snapshot in [snapshot_id(0), snapshot_id(1)] {
                    for request in 0..4 {
                        let (expected, vc, goal, context, slice) =
                            source_algorithm_direct_return_reference(
                                source, check, symbols, snapshot, request,
                            );
                        let selected = VcId::new(request + 1);
                        if label == "original" && snapshot == snapshot_id(0) {
                            identities.push(expected.clone());
                        } else if matches!(label, "original" | "trivia" | "relocation") {
                            assert_eq!(expected, identities[request], "{label}");
                        } else {
                            assert_ne!(expected, identities[request], "{label}");
                            if label == "parameter" {
                                assert_eq!(
                                    slice.fingerprint().hash(),
                                    identities[request].dependency_slice_fingerprint()
                                );
                                assert_eq!(
                                    expected.vc_fingerprint(),
                                    identities[request].vc_fingerprint()
                                );
                                assert_eq!(
                                    expected.local_context_fingerprint(),
                                    identities[request].local_context_fingerprint()
                                );
                            }
                            if label == "semantics" {
                                if request == 2 {
                                    assert_eq!(
                                        slice.fingerprint().hash(),
                                        identities[request].dependency_slice_fingerprint()
                                    );
                                } else {
                                    assert_ne!(
                                        slice.fingerprint().hash(),
                                        identities[request].dependency_slice_fingerprint()
                                    );
                                }
                            }
                        }
                        assert_eq!(
                            canonical(source, check, symbols, snapshot, selected).unwrap(),
                            (vc, goal, context, slice.clone()),
                            "{label}"
                        );
                        assert_eq!(
                            identity(source, check, symbols, snapshot, selected).unwrap(),
                            (expected, slice),
                            "{label}"
                        );
                    }
                }
                assert_eq!(format!("{source:?}{typed:?}{symbols:?}{check:?}"), before);
            },
        );
    }
}

fn source_algorithm_direct_return_assert_error(
    source: &mizar_resolve::resolved_ast::SurfaceResolvedArena,
    check: &mizar_checker::type_checker::SourceAlgorithmCheck<'_>,
    symbols: &mizar_resolve::env::SymbolEnv,
    selected: VcId,
    expected: &str,
) {
    use mizar_proof::status::{
        source_algorithm_direct_return_canonical_inputs as canonical,
        source_algorithm_direct_return_obligation_identity as identity,
    };
    assert_eq!(
        canonical(source, check, symbols, snapshot_id(0), selected).unwrap_err(),
        expected
    );
    assert_eq!(
        identity(source, check, symbols, snapshot_id(0), selected).unwrap_err(),
        expected
    );
}

#[test]
fn source_algorithm_direct_return_identity_preserves_upstream_order_and_inputs() {
    use mizar_checker::type_checker::check_source_algorithm_types as authenticate;
    use mizar_proof::status::source_algorithm_direct_return_obligation_identity as identity;
    with_direct_return_request_fixture(|source, check, symbols, _, _| {
        let before = format!("{source:?}{check:?}{symbols:?}");
        let prior = identity(source, check, symbols, snapshot_id(0), VcId::new(1)).unwrap();
        let original = fs::read_to_string(config().workspace_root.join(
            "tests/miz/pass/algorithms/pass_proof_verification_algorithm_ensures_return_001.miz",
        ))
        .unwrap();
        for text in [
            original.replace("idalgo", "foreign_algorithm"),
            format!("{original}\n{original}"),
        ] {
            let (_, _, foreign) = super::super::super::tests::source_registration_owner_inputs(
                &text,
                "owner-test",
                "owners",
                0,
            );
            let expected = authenticate(source, check.typed(), foreign.env()).unwrap_err();
            for selected in [VcId::new(1), VcId::new(usize::MAX)] {
                source_algorithm_direct_return_assert_error(
                    source,
                    check,
                    foreign.env(),
                    selected,
                    &expected,
                );
            }
        }
        with_source_algorithm_identity_inputs(
            &original.replace("idalgo", "foreign_algorithm"),
            "owner-test",
            "owners",
            7,
            |other_source, _, other_symbols, other_check| {
                for (source, check, symbols) in [
                    (other_source, check, symbols),
                    (source, other_check, symbols),
                    (source, other_check, other_symbols),
                ] {
                    let expected = authenticate(source, check.typed(), symbols).unwrap_err();
                    source_algorithm_direct_return_assert_error(
                        source,
                        check,
                        symbols,
                        VcId::new(usize::MAX),
                        &expected,
                    );
                }
            },
        );
        assert_eq!(
            identity(source, check, symbols, snapshot_id(0), VcId::new(1)).unwrap(),
            prior
        );
        assert_eq!(format!("{source:?}{check:?}{symbols:?}"), before);
    });
    let text = fs::read_to_string(
        config()
            .workspace_root
            .join("tests/miz/pass/algorithms/pass_proof_verification_claim_block_theorem_001.miz"),
    )
    .unwrap();
    with_source_algorithm_identity_inputs(
        &text,
        "owner-test",
        "owners",
        0,
        |source, typed, symbols, check| {
            let before = format!("{source:?}{typed:?}{symbols:?}{check:?}");
            let fresh = authenticate(source, check.typed(), symbols).unwrap();
            let expected = mizar_core::elaborator::lower_source_algorithms(&fresh).unwrap_err();
            assert_eq!(expected, "algorithms.core.unsupported_source");
            source_algorithm_direct_return_assert_error(
                source,
                check,
                symbols,
                VcId::new(usize::MAX),
                &expected,
            );
            assert_eq!(format!("{source:?}{typed:?}{symbols:?}{check:?}"), before);
        },
    );
}

#[test]
fn source_algorithm_direct_return_identity_refuses_old_absent_and_broader_profiles() {
    with_direct_return_request_fixture(|source, check, symbols, _, vcs| {
        for selected in [vcs.vcs()[0].id, VcId::new(usize::MAX)] {
            source_algorithm_direct_return_assert_error(
                source,
                check,
                symbols,
                selected,
                "algorithm.proof.invalid_direct_return_request",
            );
        }
    });
    let root = config().workspace_root;
    let original = fs::read_to_string(root.join(
        "tests/miz/pass/algorithms/pass_proof_verification_algorithm_ensures_return_001.miz",
    ))
    .unwrap();
    let broader = fs::read_to_string(root.join(
        "tests/miz/pass/algorithms/pass_proof_verification_algorithm_var_const_assert_001.miz",
    ))
    .unwrap();
    for text in [
        original.replace("terminating algorithm", "algorithm"),
        broader,
        original
            .replace("    ensures result = a\n", "")
            .replace("return a;", "return a;\n    return a;"),
    ] {
        with_source_algorithm_identity_inputs(
            &text,
            "owner-test",
            "owners",
            0,
            |source, typed, symbols, check| {
                let before = format!("{source:?}{typed:?}{symbols:?}{check:?}");
                let core = mizar_core::elaborator::lower_source_algorithms(check).unwrap();
                let expected =
                    mizar_vc::generator::generate_source_algorithm_direct_return_requests(
                        source,
                        check,
                        symbols,
                        &core,
                        snapshot_id(0),
                        &GenerationSchemaVersion::new("mizar-vc-generation-vc53-direct-return-v1"),
                        &VcSchemaVersion::new("mizar-vc-vcset-vc53-direct-return-v1"),
                    )
                    .unwrap_err();
                source_algorithm_direct_return_assert_error(
                    source,
                    check,
                    symbols,
                    VcId::new(usize::MAX),
                    &expected,
                );
                assert_eq!(format!("{source:?}{typed:?}{symbols:?}{check:?}"), before);
            },
        );
    }
}

#[test]
fn source_algorithm_inhabitation_witness_preserves_whole_parent_and_identity() {
    use mizar_core::core_ir::{CoreAlgorithmStmtKind, CoreFormulaKind};
    use mizar_vc::{
        generator::{
            check_source_algorithm_e1_inhabitation_result as native,
            check_source_algorithm_e1_inhabitation_witness as checked,
            generate_source_algorithm_direct_return_requests as original,
            generate_source_algorithm_e1_inhabitation_witness_request as witness,
        },
        vc_ir::{
            ContextEntryId, ContextEntryKind, PremiseRef, QuantifierKind, VcFormulaRef,
            VcGeneratedFormulaShape as Shape, VcProgramValue,
        },
    };
    let text = fs::read_to_string(config().workspace_root.join(
        "tests/miz/pass/algorithms/pass_proof_verification_algorithm_ensures_return_001.miz",
    ))
    .unwrap();
    with_source_algorithm_identity_inputs(
        &text,
        "owner-test",
        "owners",
        0,
        |source, _, symbols, check| {
            let core = mizar_core::elaborator::lower_source_algorithms(check).unwrap();
            let generation =
                GenerationSchemaVersion::new("mizar-vc-generation-vc53-direct-return-v1");
            let schema = VcSchemaVersion::new("mizar-vc-vcset-vc53-direct-return-v1");
            let parent = original(
                source,
                check,
                symbols,
                &core,
                snapshot_id(0),
                &generation,
                &schema,
            )
            .unwrap();
            let (identity, _, _, _, slice) =
                source_algorithm_direct_return_reference(source, check, symbols, snapshot_id(0), 2);
            let algorithm = core.algorithms().iter().next().unwrap().1;
            let CoreAlgorithmStmtKind::Return(Some(term)) = core
                .algorithm_statements()
                .get(algorithm.statements[0])
                .unwrap()
                .kind
            else {
                panic!("actual return")
            };
            let guard = algorithm.params[0].ty_guard.unwrap();
            assert!(matches!(&core.formulas().get(guard).unwrap().kind,
                CoreFormulaKind::TypePred { subject, ty } if ty.as_str() == "object"
                    && core.terms().get(*subject).unwrap().kind
                        == mizar_core::core_ir::CoreTermKind::Var(algorithm.params[0].var)));
            let request = &parent.vcs()[3];
            let Shape::Quantified {
                kind: QuantifierKind::Exists,
                binders,
                body,
            } = direct_return_request_shape(&parent, request.goal)
            else {
                panic!("original existential")
            };
            assert_eq!(binders, &[ContextEntryId::new(1)]);
            let entries = request.local_context.entries();
            assert_eq!(entries.len(), 2);
            assert_eq!(entries[0].kind, ContextEntryKind::CheckerFact);
            assert_eq!(entries[0].formula, Some(VcFormulaRef::Core(guard)));
            assert_eq!(request.premises, [PremiseRef::LocalContext(entries[0].id)]);
            let ContextEntryKind::BinderDeclaration { var, ref role } = entries[1].kind else {
                panic!("original result binder")
            };
            assert_eq!(role.as_str(), "result");
            assert_eq!(entries[1].formula, None);
            assert_eq!(
                direct_return_request_shape(&parent, *body),
                &Shape::ProgramTypePredicate {
                    subject: VcProgramValue {
                        var,
                        definition: None
                    },
                    ty: "object".into(),
                }
            );
            assert!(mizar_kernel::checker::check_object_inhabitation_witness(
                &core,
                (var, var, "object"),
                term,
                guard,
                &[guard],
            ));
            let expected = (
                parent,
                term,
                ContextEntryId::new(1),
                VcFormulaRef::Core(guard),
                slice.clone(),
            );
            let (descriptor, result) = {
                let generation = generation.clone();
                let schema = schema.clone();
                native(
                    source,
                    check,
                    symbols,
                    &core,
                    snapshot_id(0),
                    &generation,
                    &schema,
                    VcId::new(3),
                )
                .unwrap()
            };
            assert_eq!(descriptor, expected);
            let result: mizar_kernel::checker::ObjectInhabitationCheck<'_> = result;
            assert!(std::ptr::eq(result.core(), &core));
            assert_eq!(result.binder(), var);
            assert_eq!(result.context(), &[guard]);
            let policy = mizar_proof::policy::VerifierPolicy::default();
            let consumed = mizar_proof::status::check_source_algorithm_e1_inhabitation(
                source,
                check,
                symbols,
                &core,
                snapshot_id(0),
                VcId::new(3),
                &policy,
            )
            .unwrap();
            assert_eq!((&consumed.0, &consumed.1), (&identity, &slice));
            assert!(std::ptr::eq(consumed.2.core(), result.core()));
            assert_eq!(consumed.2.binder(), result.binder());
            assert_eq!(consumed.2.context(), result.context());
            assert_eq!(consumed.2.checker_schema_version(), 1);
            assert_eq!(consumed.3, policy.policy_fingerprint());
            let capability = mizar_proof::status::NativeE1Inhabitation::from_source(
                source,
                check,
                symbols,
                &core,
                snapshot_id(0),
                VcId::new(3),
                &policy,
            )
            .unwrap();
            assert!(
                capability
                    .adopt_original_request(
                        source,
                        check,
                        symbols,
                        &core,
                        snapshot_id(0),
                        VcId::new(3),
                        &policy,
                    )
                    .unwrap()
            );
            let (_, repeated) = native(
                source,
                check,
                symbols,
                &core,
                snapshot_id(0),
                &generation,
                &schema,
                VcId::new(3),
            )
            .unwrap();
            assert!(std::ptr::eq(repeated.core(), result.core()));
            assert_eq!(repeated.binder(), result.binder());
            assert_eq!(repeated.context(), result.context());
            for witness in [witness, checked] {
                let actual = witness(
                    source,
                    check,
                    symbols,
                    &core,
                    snapshot_id(0),
                    &generation,
                    &schema,
                    VcId::new(3),
                )
                .unwrap();
                assert_eq!(actual, expected);
                assert_eq!(
                    mizar_proof::status::source_algorithm_direct_return_obligation_identity(
                        source,
                        check,
                        symbols,
                        actual.0.snapshot(),
                        actual.0.vcs()[3].id,
                    )
                    .unwrap(),
                    (identity.clone(), slice.clone())
                );
            }
        },
    );
    let renamed = text
        .replace(" a ", " param ")
        .replace("(a)", "(param)")
        .replace(" a;", " param;")
        .replace("= a\n", "= param\n");
    with_source_algorithm_identity_inputs(
        &renamed,
        "owner-test",
        "nested.owners",
        7,
        |source, _, symbols, check| {
            let core = mizar_core::elaborator::lower_source_algorithms(check).unwrap();
            let policy = mizar_proof::policy::VerifierPolicy::default();
            let capability = mizar_proof::status::NativeE1Inhabitation::from_source(
                source,
                check,
                symbols,
                &core,
                snapshot_id(0),
                VcId::new(3),
                &policy,
            )
            .unwrap();
            assert!(
                capability
                    .adopt_original_request(
                        source,
                        check,
                        symbols,
                        &core,
                        snapshot_id(0),
                        VcId::new(3),
                        &policy,
                    )
                    .unwrap()
            );
        },
    );
}

#[test]
fn source_algorithm_inhabitation_witness_uses_renamed_relocated_return_and_fresh_binder() {
    use mizar_core::core_ir::{CoreAlgorithmStmtKind, CoreTermKind};
    use mizar_vc::{
        dependency_slice::{DependencySliceInput, try_compute_dependency_slices},
        generator::{
            check_source_algorithm_e1_inhabitation_result as native,
            check_source_algorithm_e1_inhabitation_witness as checked,
            generate_source_algorithm_direct_return_requests as original,
            generate_source_algorithm_e1_inhabitation_witness_request as witness,
        },
        vc_ir::{
            ContextEntryId, ContextEntryKind, QuantifierKind, VcFormulaRef,
            VcGeneratedFormulaShape as Shape, VcProgramValue,
        },
    };
    let text = fs::read_to_string(config().workspace_root.join(
        "tests/miz/pass/algorithms/pass_proof_verification_algorithm_ensures_return_001.miz",
    ))
    .unwrap()
    .replace(" a ", " param ")
    .replace("(a)", "(param)")
    .replace(" a;", " param;")
    .replace("= a\n", "= param\n");
    with_source_algorithm_identity_inputs(
        &text,
        "owner-test",
        "nested.owners",
        7,
        |source, _, symbols, check| {
            let core = mizar_core::elaborator::lower_source_algorithms(check).unwrap();
            let generation = GenerationSchemaVersion::new("renamed-generation");
            let schema = VcSchemaVersion::new("renamed-vc");
            let parent = original(
                source,
                check,
                symbols,
                &core,
                snapshot_id(0),
                &generation,
                &schema,
            )
            .unwrap();
            let algorithm = core.algorithms().iter().next().unwrap().1;
            let CoreAlgorithmStmtKind::Return(Some(term)) = core
                .algorithm_statements()
                .get(algorithm.statements[0])
                .unwrap()
                .kind
            else {
                panic!("actual return")
            };
            assert!(matches!(core.terms().get(term).unwrap().kind,
                CoreTermKind::Var(var) if var == algorithm.params[0].var));
            let request = &parent.vcs()[3];
            let binder = ContextEntryId::new(1);
            let ContextEntryKind::BinderDeclaration { var, .. } =
                request.local_context.entries()[1].kind
            else {
                panic!("existential declaration")
            };
            assert_ne!(var, algorithm.params[0].var);
            assert_ne!(var, algorithm.result.as_ref().unwrap().var);
            assert!(
                core.terms()
                    .iter()
                    .all(|(_, term)| !matches!(term.kind, CoreTermKind::Var(old) if old == var))
            );
            let Shape::Quantified {
                kind: QuantifierKind::Exists,
                binders,
                body,
            } = direct_return_request_shape(&parent, request.goal)
            else {
                panic!("existential parent")
            };
            assert_eq!(binders, &[binder]);
            assert_eq!(
                direct_return_request_shape(&parent, *body),
                &Shape::ProgramTypePredicate {
                    subject: VcProgramValue {
                        var,
                        definition: None
                    },
                    ty: "object".into(),
                }
            );
            let slice = try_compute_dependency_slices(DependencySliceInput {
                vc_set: &parent,
                discharge_output: None,
            })
            .unwrap()
            .slice_for(request.id)
            .unwrap()
            .clone();
            let expected = (
                parent,
                term,
                binder,
                VcFormulaRef::Core(algorithm.params[0].ty_guard.unwrap()),
                slice,
            );
            let (descriptor, result) = native(
                source,
                check,
                symbols,
                &core,
                snapshot_id(0),
                &generation,
                &schema,
                VcId::new(3),
            )
            .unwrap();
            assert_eq!(descriptor, expected);
            assert!(std::ptr::eq(result.core(), &core));
            assert_eq!(result.binder(), var);
            assert_eq!(result.context(), &[algorithm.params[0].ty_guard.unwrap()]);
            let consumed = mizar_proof::status::check_source_algorithm_e1_inhabitation(
                source,
                check,
                symbols,
                &core,
                snapshot_id(0),
                VcId::new(3),
                &mizar_proof::policy::VerifierPolicy::default(),
            )
            .unwrap();
            let (identity, _, _, _, slice) =
                source_algorithm_direct_return_reference(source, check, symbols, snapshot_id(0), 2);
            assert_eq!((consumed.0, consumed.1), (identity, slice));
            assert!(std::ptr::eq(consumed.2.core(), result.core()));
            assert_eq!(consumed.2.binder(), var);
            assert_eq!(consumed.2.context(), result.context());
            for witness in [witness, checked] {
                assert_eq!(
                    witness(
                        source,
                        check,
                        symbols,
                        &core,
                        snapshot_id(0),
                        &generation,
                        &schema,
                        VcId::new(3)
                    )
                    .unwrap(),
                    expected
                );
            }
        },
    );
}

#[test]
fn source_algorithm_inhabitation_witness_refuses_every_other_role_and_absent_id() {
    use mizar_vc::generator::{
        check_source_algorithm_e1_inhabitation_result as native,
        check_source_algorithm_e1_inhabitation_witness as checked,
        generate_source_algorithm_e1_inhabitation_witness_request as witness,
    };
    with_direct_return_request_fixture(|source, check, symbols, core, vcs| {
        for selected in [
            VcId::new(0),
            VcId::new(1),
            VcId::new(2),
            VcId::new(4),
            VcId::new(usize::MAX),
        ] {
            assert_eq!(
                native(
                    source,
                    check,
                    symbols,
                    core,
                    vcs.snapshot(),
                    &vcs.vcs()[0].anchor.generation_schema_version,
                    vcs.schema_version(),
                    selected,
                )
                .map(|_| ())
                .unwrap_err(),
                "algorithms.direct_return.invalid_inhabitation_witness_request"
            );
            for witness in [witness, checked] {
                assert_eq!(
                    witness(
                        source,
                        check,
                        symbols,
                        core,
                        vcs.snapshot(),
                        &vcs.vcs()[0].anchor.generation_schema_version,
                        vcs.schema_version(),
                        selected
                    )
                    .unwrap_err(),
                    "algorithms.direct_return.invalid_inhabitation_witness_request"
                );
            }
        }
    });
}

#[test]
fn source_algorithm_inhabitation_witness_preserves_upstream_errors_before_selection() {
    use mizar_vc::generator::{
        check_source_algorithm_e1_inhabitation_result as native,
        check_source_algorithm_e1_inhabitation_witness as checked,
        generate_source_algorithm_direct_return_requests as original,
        generate_source_algorithm_e1_inhabitation_witness_request as witness,
    };
    with_direct_return_request_fixture(|source, check, symbols, core, vcs| {
        let policy = mizar_proof::policy::VerifierPolicy::default();
        let capability = mizar_proof::status::NativeE1Inhabitation::from_source(
            source,
            check,
            symbols,
            core,
            vcs.snapshot(),
            VcId::new(3),
            &policy,
        )
        .unwrap();
        let text = fs::read_to_string(config().workspace_root.join(
            "tests/miz/pass/algorithms/pass_proof_verification_algorithm_ensures_return_001.miz",
        ))
        .unwrap();
        for foreign in [
            format!(":: stale placement\n{text}"),
            text.replace("idalgo", "foreign_algorithm"),
            text.replace("terminating algorithm", "algorithm"),
        ] {
            with_source_algorithm_identity_inputs(
                &foreign,
                "owner-test",
                "owners",
                7,
                |other_source, _, other_symbols, other_check| {
                    let other_core =
                        mizar_core::elaborator::lower_source_algorithms(other_check).unwrap();
                    if other_check.declared_terminating() {
                        assert!(
                            !capability
                                .adopt_original_request(
                                    other_source,
                                    other_check,
                                    other_symbols,
                                    &other_core,
                                    vcs.snapshot(),
                                    VcId::new(3),
                                    &policy,
                                )
                                .unwrap()
                        );
                    }
                    for (source, check, symbols, core) in [
                        (other_source, check, symbols, core),
                        (source, other_check, symbols, core),
                        (source, check, other_symbols, core),
                        (source, check, symbols, &other_core),
                    ]
                    .into_iter()
                    .chain((!other_check.declared_terminating()).then_some((
                        other_source,
                        other_check,
                        other_symbols,
                        &other_core,
                    ))) {
                        let before = format!("{source:?}{check:?}{symbols:?}{core:?}");
                        let generation = &vcs.vcs()[0].anchor.generation_schema_version;
                        let expected = original(
                            source,
                            check,
                            symbols,
                            core,
                            vcs.snapshot(),
                            generation,
                            vcs.schema_version(),
                        )
                        .unwrap_err();
                        assert_eq!(
                            mizar_vc::dependency_slice::try_compute_source_algorithm_direct_return_dependency_slices(
                                source, check, symbols, core, vcs, generation,
                            ).unwrap_err(),
                            expected,
                        );
                        for selected in [VcId::new(3), VcId::new(usize::MAX)] {
                            for policy in [
                                mizar_proof::policy::VerifierPolicy::default(),
                                mizar_proof::policy::VerifierPolicy::default()
                                    .with_kernel_evidence_formats([]),
                                mizar_proof::policy::VerifierPolicy::default()
                                    .with_checker_schema_version(2),
                            ] {
                                assert_eq!(
                                    mizar_proof::status::NativeE1Inhabitation::from_source(
                                        source,
                                        check,
                                        symbols,
                                        core,
                                        vcs.snapshot(),
                                        selected,
                                        &policy,
                                    )
                                    .err()
                                    .unwrap(),
                                    expected
                                );
                                let adopted = capability.adopt_original_request(
                                    source,
                                    check,
                                    symbols,
                                    core,
                                    vcs.snapshot(),
                                    selected,
                                    &policy,
                                );
                                match mizar_proof::status::source_algorithm_direct_return_obligation_identity(
                                    source, check, symbols, vcs.snapshot(), selected,
                                ) {
                                    Err(error) => assert_eq!(adopted.unwrap_err(), error),
                                    Ok(_) => assert!(!adopted.unwrap()),
                                }
                                assert_eq!(
                                    mizar_proof::status::check_source_algorithm_e1_inhabitation(
                                        source,
                                        check,
                                        symbols,
                                        core,
                                        vcs.snapshot(),
                                        selected,
                                        &policy,
                                    )
                                    .err()
                                    .unwrap(),
                                    expected
                                );
                            }
                            assert_eq!(
                                native(
                                    source,
                                    check,
                                    symbols,
                                    core,
                                    vcs.snapshot(),
                                    generation,
                                    vcs.schema_version(),
                                    selected,
                                )
                                .map(|_| ())
                                .unwrap_err(),
                                expected
                            );
                            for witness in [witness, checked] {
                                assert_eq!(
                                    witness(
                                        source,
                                        check,
                                        symbols,
                                        core,
                                        vcs.snapshot(),
                                        generation,
                                        vcs.schema_version(),
                                        selected
                                    )
                                    .unwrap_err(),
                                    expected
                                );
                            }
                        }
                        assert_eq!(format!("{source:?}{check:?}{symbols:?}{core:?}"), before);
                    }
                },
            );
        }
    });
}

#[test]
fn source_algorithm_inhabitation_witness_isolates_schemas_snapshot_and_preserves_open_purity() {
    use mizar_vc::{
        dependency_slice::{
            DependencySliceCompleteness, DependencySliceInput, try_compute_dependency_slices,
        },
        generator::{
            check_source_algorithm_e1_inhabitation_result as native,
            check_source_algorithm_e1_inhabitation_witness as checked,
            generate_source_algorithm_direct_return_requests as original,
            generate_source_algorithm_e1_inhabitation_witness_request as witness,
        },
        vc_ir::{ContextEntryId, VcFormulaRef, VcStatus},
    };
    with_direct_return_request_fixture(|source, check, symbols, core, vcs| {
        let before = format!("{source:?}{check:?}{symbols:?}{core:?}{vcs:?}");
        let generation = &vcs.vcs()[0].anchor.generation_schema_version;
        for (snapshot, generation, schema) in [
            (
                vcs.snapshot(),
                generation.clone(),
                vcs.schema_version().clone(),
            ),
            (
                snapshot_id(1),
                generation.clone(),
                vcs.schema_version().clone(),
            ),
            (
                vcs.snapshot(),
                GenerationSchemaVersion::new("isolated-generation"),
                vcs.schema_version().clone(),
            ),
            (
                vcs.snapshot(),
                generation.clone(),
                VcSchemaVersion::new("isolated-vc"),
            ),
        ] {
            let parent =
                original(source, check, symbols, core, snapshot, &generation, &schema).unwrap();
            let algorithm = core.algorithms().iter().next().unwrap().1;
            let mizar_core::core_ir::CoreAlgorithmStmtKind::Return(Some(term)) = core
                .algorithm_statements()
                .get(algorithm.statements[0])
                .unwrap()
                .kind
            else {
                panic!("actual return")
            };
            let slice = try_compute_dependency_slices(DependencySliceInput {
                vc_set: &parent,
                discharge_output: None,
            })
            .unwrap()
            .slice_for(VcId::new(3))
            .unwrap()
            .clone();
            assert_eq!(slice.status(), &VcStatus::Open);
            assert_eq!(
                slice.completeness(),
                DependencySliceCompleteness::IncompleteUncacheable
            );
            assert!(slice.requires_cache_miss());
            assert!(parent.vcs().iter().all(|vc| vc.status == VcStatus::Open));
            assert_eq!(parent.seed_accounting(), vcs.seed_accounting());
            assert!(
                parent
                    .seed_accounting()
                    .iter()
                    .any(|entry| entry.seed_status
                        == mizar_core::core_ir::ObligationSeedStatus::Deferred)
            );
            if snapshot != vcs.snapshot()
                || generation != vcs.vcs()[0].anchor.generation_schema_version
                || &schema != vcs.schema_version()
            {
                assert_ne!(&parent, vcs);
            }
            let expected = (
                parent,
                term,
                ContextEntryId::new(1),
                VcFormulaRef::Core(algorithm.params[0].ty_guard.unwrap()),
                slice,
            );
            assert!(!expected.4.unknowns().is_empty());
            let (descriptor, result) = native(
                source,
                check,
                symbols,
                core,
                snapshot,
                &generation,
                &schema,
                VcId::new(3),
            )
            .unwrap();
            assert_eq!(descriptor, expected);
            let mizar_vc::vc_ir::ContextEntryKind::BinderDeclaration { var, .. } =
                expected.0.vcs()[3].local_context.entries()[1].kind
            else {
                panic!("original existential binder")
            };
            assert!(std::ptr::eq(result.core(), core));
            assert_eq!(result.binder(), var);
            assert_eq!(result.context(), &[algorithm.params[0].ty_guard.unwrap()]);
            let policy = mizar_proof::policy::VerifierPolicy::default();
            let consumed = mizar_proof::status::check_source_algorithm_e1_inhabitation(
                source,
                check,
                symbols,
                core,
                snapshot,
                VcId::new(3),
                &policy,
            )
            .unwrap();
            let (identity, _, _, _, whole_slice) =
                source_algorithm_direct_return_reference(source, check, symbols, snapshot, 2);
            assert_eq!((consumed.0, consumed.1.clone()), (identity, whole_slice));
            assert!(!consumed.1.entries().is_empty());
            assert!(!consumed.1.unknowns().is_empty());
            assert_eq!(
                consumed.1.completeness(),
                DependencySliceCompleteness::IncompleteUncacheable
            );
            assert!(consumed.1.requires_cache_miss());
            assert_eq!(consumed.1.status(), &VcStatus::Open);
            assert_eq!(consumed.2.binder(), var);
            assert_eq!(consumed.2.context(), result.context());
            let capability = mizar_proof::status::NativeE1Inhabitation::from_source(
                source,
                check,
                symbols,
                core,
                snapshot,
                VcId::new(3),
                &policy,
            )
            .unwrap();
            let rebuilt_core = mizar_core::elaborator::lower_source_algorithms(check).unwrap();
            for (candidate_core, candidate_snapshot, expected) in [
                (core, snapshot, true),
                (&rebuilt_core, snapshot, false),
                (
                    core,
                    snapshot_id(if snapshot == snapshot_id(0) { 1 } else { 0 }),
                    false,
                ),
                (core, snapshot, true),
            ] {
                assert_eq!(
                    capability
                        .adopt_original_request(
                            source,
                            check,
                            symbols,
                            candidate_core,
                            candidate_snapshot,
                            VcId::new(3),
                            &policy,
                        )
                        .unwrap(),
                    expected
                );
            }
            for witness in [witness, checked] {
                assert_eq!(
                    witness(
                        source,
                        check,
                        symbols,
                        core,
                        snapshot,
                        &generation,
                        &schema,
                        VcId::new(3)
                    )
                    .unwrap(),
                    expected
                );
            }
        }
        assert_eq!(
            format!("{source:?}{check:?}{symbols:?}{core:?}{vcs:?}"),
            before
        );
    });
}

#[test]
fn source_algorithm_native_e1_refuses_wrong_roles_before_policy() {
    use mizar_proof::{
        policy::VerifierPolicy, status::check_source_algorithm_e1_inhabitation as consume,
    };
    with_direct_return_request_fixture(|source, check, symbols, core, vcs| {
        let capability = mizar_proof::status::NativeE1Inhabitation::from_source(
            source,
            check,
            symbols,
            core,
            vcs.snapshot(),
            VcId::new(3),
            &VerifierPolicy::default(),
        )
        .unwrap();
        for selected in [
            VcId::new(0),
            VcId::new(1),
            VcId::new(2),
            VcId::new(4),
            VcId::new(usize::MAX),
        ] {
            for policy in [
                VerifierPolicy::default(),
                VerifierPolicy::default().with_kernel_evidence_formats([]),
                VerifierPolicy::default().with_checker_schema_version(2),
            ] {
                assert_eq!(
                    mizar_proof::status::NativeE1Inhabitation::from_source(
                        source,
                        check,
                        symbols,
                        core,
                        vcs.snapshot(),
                        selected,
                        &policy,
                    )
                    .err()
                    .unwrap(),
                    "algorithms.direct_return.invalid_inhabitation_witness_request"
                );
                let adopted = capability.adopt_original_request(
                    source,
                    check,
                    symbols,
                    core,
                    vcs.snapshot(),
                    selected,
                    &policy,
                );
                if selected == VcId::new(0) || selected == VcId::new(usize::MAX) {
                    assert_eq!(
                        adopted.unwrap_err(),
                        "algorithm.proof.invalid_direct_return_request"
                    );
                } else {
                    assert!(!adopted.unwrap());
                }
                assert_eq!(
                    consume(
                        source,
                        check,
                        symbols,
                        core,
                        vcs.snapshot(),
                        selected,
                        &policy
                    )
                    .err()
                    .unwrap(),
                    "algorithms.direct_return.invalid_inhabitation_witness_request"
                );
            }
        }
    });
}

#[test]
fn source_algorithm_native_e1_uses_current_policy_and_actual_checker_schema() {
    use mizar_proof::{
        policy::{KernelEvidenceFormat, VerifierPolicy},
        status::check_source_algorithm_e1_inhabitation as consume,
    };
    with_direct_return_request_fixture(|source, check, symbols, core, vcs| {
        let default = VerifierPolicy::default();
        let capability = mizar_proof::status::NativeE1Inhabitation::from_source(
            source,
            check,
            symbols,
            core,
            vcs.snapshot(),
            VcId::new(3),
            &default,
        )
        .unwrap();
        let prior = consume(
            source,
            check,
            symbols,
            core,
            vcs.snapshot(),
            VcId::new(3),
            &default,
        )
        .unwrap();
        for policy in [
            default.clone(),
            VerifierPolicy::development(),
            VerifierPolicy::interactive(),
            default.clone().with_profile_id("changed-native-policy"),
        ] {
            let actual = consume(
                source,
                check,
                symbols,
                core,
                vcs.snapshot(),
                VcId::new(3),
                &policy,
            )
            .unwrap();
            let current = mizar_proof::status::NativeE1Inhabitation::from_source(
                source,
                check,
                symbols,
                core,
                vcs.snapshot(),
                VcId::new(3),
                &policy,
            )
            .unwrap();
            for native in [&capability, &current] {
                assert!(
                    native
                        .adopt_original_request(
                            source,
                            check,
                            symbols,
                            core,
                            vcs.snapshot(),
                            VcId::new(3),
                            &policy,
                        )
                        .unwrap()
                );
            }
            assert_eq!((&actual.0, &actual.1), (&prior.0, &prior.1));
            assert_eq!(actual.3, policy.policy_fingerprint());
            if policy != default {
                assert_ne!(actual.3, prior.3);
            }
            assert_eq!(actual.2.checker_schema_version(), 1);
        }
        for policy in [
            default.clone().with_kernel_evidence_formats([]),
            default
                .clone()
                .with_kernel_evidence_formats([KernelEvidenceFormat::FormulaSubstitution]),
            default.clone().with_checker_schema_version(0),
            default.clone().with_checker_schema_version(2),
        ] {
            let before = policy.clone();
            assert_eq!(
                mizar_proof::status::NativeE1Inhabitation::from_source(
                    source,
                    check,
                    symbols,
                    core,
                    vcs.snapshot(),
                    VcId::new(3),
                    &policy,
                )
                .err()
                .unwrap(),
                "algorithm.proof.native_e1_policy_rejected"
            );
            assert_eq!(
                capability
                    .adopt_original_request(
                        source,
                        check,
                        symbols,
                        core,
                        vcs.snapshot(),
                        VcId::new(3),
                        &policy,
                    )
                    .unwrap_err(),
                "algorithm.proof.native_e1_policy_rejected"
            );
            assert_eq!(
                consume(
                    source,
                    check,
                    symbols,
                    core,
                    vcs.snapshot(),
                    VcId::new(3),
                    &policy
                )
                .err()
                .unwrap(),
                "algorithm.proof.native_e1_policy_rejected"
            );
            assert_eq!(policy, before);
        }
        assert!(
            capability
                .adopt_original_request(
                    source,
                    check,
                    symbols,
                    core,
                    vcs.snapshot(),
                    VcId::new(3),
                    &default,
                )
                .unwrap()
        );
        assert_eq!(
            consume(
                source,
                check,
                symbols,
                core,
                vcs.snapshot(),
                VcId::new(3),
                &default
            )
            .unwrap()
            .3,
            prior.3
        );
    });
}

#[test]
fn source_algorithm_e4_reduction_preserves_whole_parent_descriptor_and_identity() {
    use mizar_core::core_ir::CoreAlgorithmStmtKind;
    use mizar_vc::{
        generator::{
            generate_source_algorithm_direct_return_requests as original,
            generate_source_algorithm_e4_operational_reduction_request as reduction,
        },
        vc_ir::{ContextEntryId, VcGeneratedFormulaShape as Shape, VcProgramValue},
    };
    with_direct_return_request_fixture(|source, check, symbols, core, _| {
        let generation = GenerationSchemaVersion::new("mizar-vc-generation-vc53-direct-return-v1");
        let schema = VcSchemaVersion::new("mizar-vc-vcset-vc53-direct-return-v1");
        let parent = original(
            source,
            check,
            symbols,
            core,
            snapshot_id(0),
            &generation,
            &schema,
        )
        .unwrap();
        let (identity, _, _, _, slice) =
            source_algorithm_direct_return_reference(source, check, symbols, snapshot_id(0), 3);
        let (algorithm_id, algorithm) = core.algorithms().iter().next().unwrap();
        let CoreAlgorithmStmtKind::Return(Some(term)) = core
            .algorithm_statements()
            .get(algorithm.statements[0])
            .unwrap()
            .kind
        else {
            panic!("actual return")
        };
        let input = VcProgramValue {
            var: algorithm.params[0].var,
            definition: None,
        };
        let expected = (
            parent,
            term,
            ContextEntryId::new(1),
            Shape::AlgorithmReturns {
                algorithm: algorithm_id,
                flow: mizar_core::control_flow::build_control_flow_ir(core).flow_map[&algorithm_id],
                statement: algorithm.statements[0],
                input,
                result: input,
            },
            Shape::True,
            slice.clone(),
        );
        let actual = reduction(
            source,
            check,
            symbols,
            core,
            snapshot_id(0),
            &generation,
            &schema,
            VcId::new(4),
        )
        .unwrap();
        assert_eq!(actual, expected);
        assert_eq!(
            mizar_proof::status::source_algorithm_direct_return_obligation_identity(
                source,
                check,
                symbols,
                actual.0.snapshot(),
                actual.0.vcs()[4].id,
            )
            .unwrap(),
            (identity, slice)
        );
    });
}

#[test]
fn source_algorithm_e4_reduction_uses_renamed_relocated_return_and_original_fresh_binder() {
    use mizar_core::core_ir::{CoreAlgorithmStmtKind, CoreTermKind};
    use mizar_vc::{
        generator::{
            generate_source_algorithm_direct_return_requests as original,
            generate_source_algorithm_e4_operational_reduction_request as reduction,
        },
        vc_ir::{
            ContextEntryKind, QuantifierKind, VcGeneratedFormulaShape as Shape, VcProgramValue,
        },
    };
    let text = fs::read_to_string(config().workspace_root.join(
        "tests/miz/pass/algorithms/pass_proof_verification_algorithm_ensures_return_001.miz",
    ))
    .unwrap()
    .replace(" a ", " param ")
    .replace("(a)", "(param)")
    .replace(" a;", " param;")
    .replace("= a\n", "= param\n");
    with_source_algorithm_identity_inputs(
        &text,
        "owner-test",
        "nested.owners",
        7,
        |source, _, symbols, check| {
            let core = mizar_core::elaborator::lower_source_algorithms(check).unwrap();
            let generation = GenerationSchemaVersion::new("relocated-generation");
            let schema = VcSchemaVersion::new("relocated-vc");
            let parent = original(
                source,
                check,
                symbols,
                &core,
                snapshot_id(0),
                &generation,
                &schema,
            )
            .unwrap();
            let (algorithm_id, algorithm) = core.algorithms().iter().next().unwrap();
            let CoreAlgorithmStmtKind::Return(Some(term)) = core
                .algorithm_statements()
                .get(algorithm.statements[0])
                .unwrap()
                .kind
            else {
                panic!("actual return")
            };
            assert_eq!(
                core.terms().get(term).unwrap().kind,
                CoreTermKind::Var(algorithm.params[0].var)
            );
            let request = &parent.vcs()[4];
            let Shape::Quantified {
                kind: QuantifierKind::Exists,
                binders,
                body,
            } = direct_return_request_shape(&parent, request.goal)
            else {
                panic!("E4 existential")
            };
            let [binder] = binders.as_slice() else {
                panic!("one E4 binder")
            };
            let ContextEntryKind::BinderDeclaration { var, .. } =
                request.local_context.entries()[binder.index()].kind
            else {
                panic!("original declaration")
            };
            assert_ne!(var, algorithm.params[0].var);
            assert_ne!(var, algorithm.result.as_ref().unwrap().var);
            assert!(
                core.terms()
                    .iter()
                    .all(|(_, term)| !matches!(term.kind, CoreTermKind::Var(old) if old == var))
            );
            let input = VcProgramValue {
                var: algorithm.params[0].var,
                definition: None,
            };
            let flow =
                mizar_core::control_flow::build_control_flow_ir(&core).flow_map[&algorithm_id];
            assert_eq!(
                direct_return_request_shape(&parent, *body),
                &Shape::AlgorithmReturns {
                    algorithm: algorithm_id,
                    flow,
                    statement: algorithm.statements[0],
                    input,
                    result: VcProgramValue {
                        var,
                        definition: None
                    },
                }
            );
            let slice = source_algorithm_direct_return_slice_oracle(
                source,
                check,
                symbols,
                &core,
                &parent,
                &generation,
            )
            .slice_for(request.id)
            .unwrap()
            .clone();
            let expected = (
                parent.clone(),
                term,
                *binder,
                Shape::AlgorithmReturns {
                    algorithm: algorithm_id,
                    flow,
                    statement: algorithm.statements[0],
                    input,
                    result: input,
                },
                Shape::True,
                slice,
            );
            assert_eq!(
                reduction(
                    source,
                    check,
                    symbols,
                    &core,
                    snapshot_id(0),
                    &generation,
                    &schema,
                    request.id,
                )
                .unwrap(),
                expected
            );
        },
    );
}

#[test]
fn source_algorithm_e4_reduction_refuses_every_other_role_and_absent_id() {
    use mizar_vc::generator::generate_source_algorithm_e4_operational_reduction_request as reduction;
    with_direct_return_request_fixture(|source, check, symbols, core, vcs| {
        for selected in [
            VcId::new(0),
            VcId::new(1),
            VcId::new(2),
            VcId::new(3),
            VcId::new(usize::MAX),
        ] {
            assert_eq!(
                reduction(
                    source,
                    check,
                    symbols,
                    core,
                    vcs.snapshot(),
                    &vcs.vcs()[0].anchor.generation_schema_version,
                    vcs.schema_version(),
                    selected,
                )
                .unwrap_err(),
                "algorithms.direct_return.invalid_e4_operational_reduction_request"
            );
        }
    });
}

#[test]
fn source_algorithm_e4_reduction_preserves_upstream_errors_before_selection() {
    use mizar_vc::generator::{
        generate_source_algorithm_direct_return_requests as original,
        generate_source_algorithm_e4_operational_reduction_request as reduction,
    };
    with_direct_return_request_fixture(|source, check, symbols, core, vcs| {
        let text = fs::read_to_string(config().workspace_root.join(
            "tests/miz/pass/algorithms/pass_proof_verification_algorithm_ensures_return_001.miz",
        ))
        .unwrap();
        for foreign in [
            format!(":: stale placement\n{text}"),
            text.replace("idalgo", "foreign_algorithm"),
            text.replace("terminating algorithm", "algorithm"),
        ] {
            with_source_algorithm_identity_inputs(
                &foreign,
                "owner-test",
                "owners",
                7,
                |other_source, _, other_symbols, other_check| {
                    let other_core =
                        mizar_core::elaborator::lower_source_algorithms(other_check).unwrap();
                    for (source, check, symbols, core) in [
                        (other_source, check, symbols, core),
                        (source, other_check, symbols, core),
                        (source, check, other_symbols, core),
                        (source, check, symbols, &other_core),
                    ]
                    .into_iter()
                    .chain((!other_check.declared_terminating()).then_some((
                        other_source,
                        other_check,
                        other_symbols,
                        &other_core,
                    ))) {
                        let before = format!("{source:?}{check:?}{symbols:?}{core:?}");
                        let generation = &vcs.vcs()[0].anchor.generation_schema_version;
                        let expected = original(
                            source,
                            check,
                            symbols,
                            core,
                            vcs.snapshot(),
                            generation,
                            vcs.schema_version(),
                        )
                        .unwrap_err();
                        for selected in [VcId::new(4), VcId::new(usize::MAX)] {
                            assert_eq!(
                                reduction(
                                    source,
                                    check,
                                    symbols,
                                    core,
                                    vcs.snapshot(),
                                    generation,
                                    vcs.schema_version(),
                                    selected,
                                )
                                .unwrap_err(),
                                expected
                            );
                        }
                        assert_eq!(format!("{source:?}{check:?}{symbols:?}{core:?}"), before);
                    }
                },
            );
        }
    });
}

#[test]
fn source_algorithm_e4_reduction_isolates_schemas_snapshot_and_preserves_open_purity() {
    use mizar_vc::{
        dependency_slice::DependencySliceCompleteness,
        generator::{
            generate_source_algorithm_direct_return_requests as original,
            generate_source_algorithm_e4_operational_reduction_request as reduction,
        },
        vc_ir::{ContextEntryId, VcGeneratedFormulaShape as Shape, VcProgramValue, VcStatus},
    };
    with_direct_return_request_fixture(|source, check, symbols, core, vcs| {
        let before = format!("{source:?}{check:?}{symbols:?}{core:?}{vcs:?}");
        let generation = &vcs.vcs()[0].anchor.generation_schema_version;
        for (snapshot, generation, schema) in [
            (
                vcs.snapshot(),
                generation.clone(),
                vcs.schema_version().clone(),
            ),
            (
                snapshot_id(1),
                generation.clone(),
                vcs.schema_version().clone(),
            ),
            (
                vcs.snapshot(),
                GenerationSchemaVersion::new("isolated-generation"),
                vcs.schema_version().clone(),
            ),
            (
                vcs.snapshot(),
                generation.clone(),
                VcSchemaVersion::new("isolated-vc"),
            ),
        ] {
            let parent =
                original(source, check, symbols, core, snapshot, &generation, &schema).unwrap();
            let (algorithm_id, algorithm) = core.algorithms().iter().next().unwrap();
            let mizar_core::core_ir::CoreAlgorithmStmtKind::Return(Some(term)) = core
                .algorithm_statements()
                .get(algorithm.statements[0])
                .unwrap()
                .kind
            else {
                panic!("actual return")
            };
            let slice = source_algorithm_direct_return_slice_oracle(
                source,
                check,
                symbols,
                core,
                &parent,
                &generation,
            )
            .slice_for(VcId::new(4))
            .unwrap()
            .clone();
            assert_eq!(slice.status(), &VcStatus::Open);
            assert_eq!(
                slice.completeness(),
                DependencySliceCompleteness::IncompleteUncacheable
            );
            assert!(slice.requires_cache_miss());
            assert!(parent.vcs().iter().all(|vc| vc.status == VcStatus::Open));
            assert_eq!(parent.seed_accounting(), vcs.seed_accounting());
            assert!(
                parent
                    .seed_accounting()
                    .iter()
                    .any(|entry| entry.seed_status
                        == mizar_core::core_ir::ObligationSeedStatus::Deferred)
            );
            if snapshot != vcs.snapshot()
                || generation != vcs.vcs()[0].anchor.generation_schema_version
                || &schema != vcs.schema_version()
            {
                assert_ne!(&parent, vcs);
            }
            let input = VcProgramValue {
                var: algorithm.params[0].var,
                definition: None,
            };
            let expected = (
                parent,
                term,
                ContextEntryId::new(1),
                Shape::AlgorithmReturns {
                    algorithm: algorithm_id,
                    flow: mizar_core::control_flow::build_control_flow_ir(core).flow_map
                        [&algorithm_id],
                    statement: algorithm.statements[0],
                    input,
                    result: input,
                },
                Shape::True,
                slice,
            );
            for _ in 0..2 {
                assert_eq!(
                    reduction(
                        source,
                        check,
                        symbols,
                        core,
                        snapshot,
                        &generation,
                        &schema,
                        VcId::new(4),
                    )
                    .unwrap(),
                    expected
                );
            }
        }
        assert_eq!(
            format!("{source:?}{check:?}{symbols:?}{core:?}{vcs:?}"),
            before
        );
    });
}

fn sole_return_core_parts(core: &mizar_core::core_ir::CoreIr) -> mizar_core::core_ir::CoreIrParts {
    mizar_core::core_ir::CoreIrParts {
        source_id: core.source_id(),
        module_id: core.module_id().clone(),
        items: core.items().clone(),
        terms: core.terms().clone(),
        formulas: core.formulas().clone(),
        definitions: core.definitions().clone(),
        proofs: core.proofs().clone(),
        proof_nodes: core.proof_nodes().clone(),
        algorithms: core.algorithms().clone(),
        algorithm_statements: core.algorithm_statements().clone(),
        generated: core.generated().clone(),
        obligation_seeds: core.obligation_seeds().clone(),
        source_map: core.source_map().clone(),
        diagnostics: core.diagnostics().clone(),
    }
}

#[test]
fn source_algorithm_sole_return_var_borrows_arbitrary_opaque_inputs_and_relocated_binding() {
    use mizar_core::core_ir::{CoreAlgorithmStmtKind, CoreFormulaKind, CoreIr};
    use mizar_kernel::checker::check_sole_return_var;
    // No Clone bound or runtime-value grammar; typing and initialization are caller premises.
    #[derive(Debug, PartialEq, Eq)]
    struct Opaque(Vec<u8>, String);
    let check_values = |core: &CoreIr| {
        let (id, algorithm) = core.algorithms().iter().next().unwrap();
        let CoreAlgorithmStmtKind::Return(Some(term)) = core
            .algorithm_statements()
            .get(algorithm.statements[0])
            .unwrap()
            .kind
        else {
            panic!("sole return")
        };
        for value in [
            Opaque(vec![], "empty".into()),
            Opaque(vec![255, 0, 7], "opaque".into()),
        ] {
            let environment = [(
                algorithm.params[0].var,
                algorithm.params[0].ty_guard.unwrap(),
                value,
            )];
            let actual = check_sole_return_var(core, id, &environment).unwrap();
            assert_eq!(actual, (algorithm.statements[0], term, &environment[0].2));
            assert!(std::ptr::eq(actual.2, &environment[0].2));
        }
    };
    with_direct_return_request_fixture(|_, _, _, core, _| {
        check_values(core);
        // A resolved result guard is an association, not a check of the opaque value's type.
        let (_, algorithm) = core.algorithms().iter().next().unwrap();
        let mut parts = sole_return_core_parts(core);
        parts
            .formulas
            .get_mut(algorithm.result.as_ref().unwrap().ty_guard.unwrap())
            .unwrap()
            .kind = CoreFormulaKind::False;
        check_values(&CoreIr::try_new(parts).unwrap());
    });
    let text = fs::read_to_string(config().workspace_root.join(
        "tests/miz/pass/algorithms/pass_proof_verification_algorithm_ensures_return_001.miz",
    ))
    .unwrap()
    .replace(" a ", " input ")
    .replace("(a)", "(input)")
    .replace(" a;", " input;")
    .replace("= a\n", "= input\n");
    with_source_algorithm_identity_inputs(
        &text,
        "local-evaluation",
        "relocated.binding",
        7,
        |_, _, _, check| {
            check_values(&mizar_core::elaborator::lower_source_algorithms(check).unwrap())
        },
    );
}

#[test]
fn source_algorithm_sole_return_var_refuses_environment_mismatch_without_output() {
    use mizar_kernel::checker::check_sole_return_var;
    with_direct_return_request_fixture(|_, _, _, core, _| {
        let (id, algorithm) = core.algorithms().iter().next().unwrap();
        let parameter = &algorithm.params[0];
        let result = algorithm.result.as_ref().unwrap();
        let input = (parameter.var, parameter.ty_guard.unwrap(), 17);
        let other = (result.var, result.ty_guard.unwrap(), 23);
        for environment in [
            vec![],
            vec![input, input],
            vec![input, other],
            vec![other],
            vec![(result.var, parameter.ty_guard.unwrap(), 17)],
            vec![(parameter.var, result.ty_guard.unwrap(), 17)],
        ] {
            assert_eq!(check_sole_return_var(core, id, &environment), None);
        }
    });
}

#[test]
fn source_algorithm_sole_return_var_refuses_other_roles_and_executable_bodies() {
    use mizar_core::core_ir::{CoreAlgorithmStmtKind as Stmt, CoreIr, CoreTermKind};
    use mizar_kernel::checker::check_sole_return_var;
    with_direct_return_request_fixture(|_, _, _, core, _| {
        let (id, algorithm) = core.algorithms().iter().next().unwrap();
        let statement = algorithm.statements[0];
        let Stmt::Return(Some(term)) = core.algorithm_statements().get(statement).unwrap().kind
        else {
            panic!("sole return")
        };
        let environment = [(
            algorithm.params[0].var,
            algorithm.params[0].ty_guard.unwrap(),
            (),
        )];
        let flow = mizar_core::control_flow::build_control_flow_ir(core).flow_map[&id];
        let value = (algorithm.params[0].var, None);
        let target = (id, flow, statement, value, value);
        let context = [algorithm.params[0].ty_guard.unwrap()];
        for mutation in 0..18 {
            let mut parts = sole_return_core_parts(core);
            match mutation {
                0..=3 => {
                    parts.algorithms.get_mut(id).unwrap().params[0].role =
                        ["local:var", "local:const", "result", "ghost"][mutation].into()
                }
                4 => {
                    parts
                        .algorithms
                        .get_mut(id)
                        .unwrap()
                        .result
                        .as_mut()
                        .unwrap()
                        .role = "parameter".into()
                }
                5 => parts.terms.get_mut(term).unwrap().kind = CoreTermKind::Numeral("1".into()),
                6 => {
                    parts.terms.get_mut(term).unwrap().kind =
                        CoreTermKind::Var(algorithm.result.as_ref().unwrap().var)
                }
                7 => {
                    parts.algorithm_statements.get_mut(statement).unwrap().kind = Stmt::Return(None)
                }
                8 => {
                    let mut extra = core.algorithm_statements().get(statement).unwrap().clone();
                    extra.kind = Stmt::Assert {
                        formula: algorithm.params[0].ty_guard.unwrap(),
                    };
                    let extra_id = parts.algorithm_statements.insert(extra.clone());
                    parts
                        .source_map
                        .algorithm_sources
                        .insert(extra_id, extra.source);
                    parts
                        .algorithms
                        .get_mut(id)
                        .unwrap()
                        .statements
                        .insert(0, extra_id);
                }
                9 => parts.algorithms.get_mut(id).unwrap().params.clear(),
                10 => parts
                    .algorithms
                    .get_mut(id)
                    .unwrap()
                    .params
                    .push(algorithm.params[0].clone()),
                11 => parts.algorithms.get_mut(id).unwrap().result = None,
                12 => parts.algorithms.get_mut(id).unwrap().statements.clear(),
                13 => parts
                    .algorithms
                    .get_mut(id)
                    .unwrap()
                    .ghost_effects
                    .push("ghost-write".into()),
                _ => {
                    let mut binder = algorithm.result.as_ref().unwrap().clone();
                    binder.role = "local:var".into();
                    let mut local_term = core.terms().get(term).unwrap().clone();
                    local_term.kind = CoreTermKind::Var(binder.var);
                    let local_return = parts.terms.insert(local_term.clone());
                    parts
                        .source_map
                        .term_sources
                        .insert(local_return, local_term.source);
                    parts.algorithm_statements.get_mut(statement).unwrap().kind =
                        Stmt::Return(Some(local_return));
                    let mut declaration =
                        core.algorithm_statements().get(statement).unwrap().clone();
                    declaration.kind = Stmt::Let {
                        binder,
                        value: (mutation % 2 == 0).then_some(term),
                        ghost: mutation >= 16,
                    };
                    let declaration_id = parts.algorithm_statements.insert(declaration.clone());
                    parts
                        .source_map
                        .algorithm_sources
                        .insert(declaration_id, declaration.source);
                    parts
                        .algorithms
                        .get_mut(id)
                        .unwrap()
                        .statements
                        .insert(0, declaration_id);
                }
            }
            let changed = CoreIr::try_new(parts).unwrap();
            assert_eq!(
                check_sole_return_var(&changed, id, &environment),
                None,
                "mutation {mutation}"
            );
            assert!(
                !mizar_kernel::checker::check_instantiated_algorithm_returns(
                    &changed, target, term, &context,
                ),
                "instantiated mutation {mutation}"
            );
        }
    });
}

#[test]
fn source_algorithm_sole_return_var_refuses_corrupt_core_and_absent_algorithm() {
    use mizar_core::core_ir::{
        CoreAlgorithmId, CoreAlgorithmStmtKind as Stmt, CoreDiagnostic, CoreDiagnosticClass,
        CoreFormulaId, CoreFormulaKind, CoreIr, CoreIrError, CoreTermKind,
    };
    use mizar_kernel::checker::check_sole_return_var;
    with_direct_return_request_fixture(|_, _, _, core, _| {
        let (id, algorithm) = core.algorithms().iter().next().unwrap();
        let statement = algorithm.statements[0];
        let Stmt::Return(Some(term)) = core.algorithm_statements().get(statement).unwrap().kind
        else {
            panic!("sole return")
        };
        let environment = [(
            algorithm.params[0].var,
            algorithm.params[0].ty_guard.unwrap(),
            (),
        )];
        let flow = mizar_core::control_flow::build_control_flow_ir(core).flow_map[&id];
        let value = (algorithm.params[0].var, None);
        let target = (id, flow, statement, value, value);
        let context = [algorithm.params[0].ty_guard.unwrap()];
        assert_eq!(
            check_sole_return_var(
                core,
                CoreAlgorithmId::new(core.algorithms().len()),
                &environment
            ),
            None
        );
        for mutation in 0..12 {
            let mut parts = sole_return_core_parts(core);
            match mutation {
                0 => parts.algorithms.get_mut(id).unwrap().params[0].ty_guard = None,
                1 => {
                    parts
                        .algorithms
                        .get_mut(id)
                        .unwrap()
                        .result
                        .as_mut()
                        .unwrap()
                        .ty_guard = None
                }
                2 => {
                    parts
                        .algorithms
                        .get_mut(id)
                        .unwrap()
                        .result
                        .as_mut()
                        .unwrap()
                        .var = algorithm.params[0].var
                }
                3 => {
                    parts.algorithm_statements.get_mut(statement).unwrap().owner =
                        CoreAlgorithmId::new(core.algorithms().len())
                }
                4 => {
                    parts.algorithms.get_mut(id).unwrap().params[0].ty_guard =
                        Some(CoreFormulaId::new(core.formulas().len()))
                }
                5 => {
                    parts
                        .algorithms
                        .get_mut(id)
                        .unwrap()
                        .result
                        .as_mut()
                        .unwrap()
                        .ty_guard = Some(CoreFormulaId::new(core.formulas().len()))
                }
                _ => {
                    let source = match mutation {
                        9 => core.terms().get(term).unwrap().source.clone(),
                        10 => core
                            .formulas()
                            .get(algorithm.params[0].ty_guard.unwrap())
                            .unwrap()
                            .source
                            .clone(),
                        _ => core
                            .algorithm_statements()
                            .get(statement)
                            .unwrap()
                            .source
                            .clone(),
                    };
                    let diagnostic = parts.diagnostics.insert(CoreDiagnostic::error(
                        CoreDiagnosticClass::AlgorithmShell,
                        "local-return-test",
                        source,
                    ));
                    match mutation {
                        6 => {}
                        7 => parts
                            .algorithms
                            .get_mut(id)
                            .unwrap()
                            .diagnostics
                            .push(diagnostic),
                        8 => parts
                            .algorithm_statements
                            .get_mut(statement)
                            .unwrap()
                            .diagnostics
                            .push(diagnostic),
                        9 => {
                            parts.terms.get_mut(term).unwrap().kind =
                                CoreTermKind::Error(diagnostic)
                        }
                        10 => {
                            parts
                                .formulas
                                .get_mut(algorithm.params[0].ty_guard.unwrap())
                                .unwrap()
                                .kind = CoreFormulaKind::Error(diagnostic)
                        }
                        _ => {
                            parts.algorithm_statements.get_mut(statement).unwrap().kind =
                                Stmt::Error(diagnostic)
                        }
                    }
                }
            }
            let changed = CoreIr::try_new(parts);
            match mutation {
                3 => assert!(matches!(
                    changed,
                    Err(CoreIrError::StatementOwnerMismatch { .. })
                )),
                4 | 5 => assert!(matches!(
                    changed,
                    Err(CoreIrError::InvalidReference {
                        table: "formula",
                        ..
                    })
                )),
                _ => {
                    let changed = changed.unwrap();
                    assert_eq!(
                        check_sole_return_var(&changed, id, &environment),
                        None,
                        "mutation {mutation}"
                    );
                    assert!(
                        !mizar_kernel::checker::check_instantiated_algorithm_returns(
                            &changed, target, term, &context,
                        ),
                        "instantiated mutation {mutation}"
                    );
                }
            }
        }
    });
}

#[test]
fn source_algorithm_sole_return_var_preserves_authentic_e4_identity_slice_and_open_status() {
    use mizar_kernel::checker::check_sole_return_var;
    use mizar_vc::{
        generator::{
            generate_source_algorithm_direct_return_requests as original,
            generate_source_algorithm_e4_operational_reduction_request as reduction,
        },
        vc_ir::{VcKind, VcStatus},
    };
    with_direct_return_request_fixture(|source, check, symbols, core, _| {
        let generation = GenerationSchemaVersion::new("mizar-vc-generation-vc53-direct-return-v1");
        let schema = VcSchemaVersion::new("mizar-vc-vcset-vc53-direct-return-v1");
        let parent = original(
            source,
            check,
            symbols,
            core,
            snapshot_id(0),
            &generation,
            &schema,
        )
        .unwrap();
        let vcs = &parent;
        let before = format!("{source:?}{check:?}{symbols:?}{core:?}{vcs:?}");
        let generation = &vcs.vcs()[0].anchor.generation_schema_version;
        let descriptor = || {
            reduction(
                source,
                check,
                symbols,
                core,
                vcs.snapshot(),
                generation,
                vcs.schema_version(),
                VcId::new(4),
            )
            .unwrap()
        };
        let identity = || {
            mizar_proof::status::source_algorithm_direct_return_obligation_identity(
                source,
                check,
                symbols,
                vcs.snapshot(),
                VcId::new(4),
            )
            .unwrap()
        };
        let descriptor_before = descriptor();
        let identity_before = identity();
        let raw_slice = source_algorithm_direct_return_slice_oracle(
            source, check, symbols, core, vcs, generation,
        )
        .slice_for(VcId::new(4))
        .unwrap()
        .clone();
        assert_eq!(descriptor_before.0, *vcs);
        assert_eq!(descriptor_before.5, raw_slice);
        assert_eq!(identity_before.1, raw_slice);
        let (id, algorithm) = core.algorithms().iter().next().unwrap();
        let environment = [(
            algorithm.params[0].var,
            algorithm.params[0].ty_guard.unwrap(),
            vec![1, 4, 9],
        )];
        let actual = check_sole_return_var(core, id, &environment).unwrap();
        assert_eq!(
            actual,
            (
                algorithm.statements[0],
                descriptor_before.1,
                &environment[0].2
            )
        );
        assert!(std::ptr::eq(actual.2, &environment[0].2));
        assert_eq!(descriptor(), descriptor_before);
        assert_eq!(identity(), identity_before);
        assert_eq!(
            vcs.vcs()
                .iter()
                .filter(|vc| matches!(
                    vc.kind,
                    VcKind::AlgorithmE1Typing
                        | VcKind::AlgorithmE1SingleValuedness
                        | VcKind::AlgorithmE1ResultInhabitation
                        | VcKind::AlgorithmE4NormalReturn
                ))
                .count(),
            4
        );
        assert!(
            vcs.vcs()
                .iter()
                .all(|vc| vc.status == VcStatus::Open && vc.proof_hint.is_none())
        );
        assert!(
            vcs.seed_accounting()
                .iter()
                .any(|entry| entry.seed_status
                    == mizar_core::core_ir::ObligationSeedStatus::Deferred)
        );
        assert_eq!(
            format!("{source:?}{check:?}{symbols:?}{core:?}{vcs:?}"),
            before
        );
    });
}

#[test]
fn source_algorithm_e4_validator_handoff_preserves_descriptor_and_borrowed_oracle() {
    use mizar_core::core_ir::CoreAlgorithmStmtKind;
    use mizar_vc::{
        generator::{
            check_source_algorithm_e4_local_request as consume,
            generate_source_algorithm_e4_operational_reduction_request as reduction,
            generate_source_algorithm_e4_validator_handoff_request as handoff,
        },
        vc_ir::{VcGeneratedFormulaShape as Shape, VcProgramValue},
    };
    with_direct_return_request_fixture(|source, check, symbols, core, vcs| {
        let (id, algorithm) = core.algorithms().iter().next().unwrap();
        let statement = algorithm.statements[0];
        let CoreAlgorithmStmtKind::Return(Some(term)) =
            core.algorithm_statements().get(statement).unwrap().kind
        else {
            panic!("actual return")
        };
        // Mutex is neither Clone nor Eq; runtime typing remains a caller premise.
        let environment = [(
            algorithm.params[0].var,
            algorithm.params[0].ty_guard.unwrap(),
            std::sync::Mutex::new(vec![255_u8, 0, 7]),
        )];
        let generation = &vcs.vcs()[0].anchor.generation_schema_version;
        let expected = reduction(
            source,
            check,
            symbols,
            core,
            vcs.snapshot(),
            generation,
            vcs.schema_version(),
            VcId::new(4),
        )
        .unwrap();
        let (descriptor, input, output) = handoff(
            source,
            check,
            symbols,
            core,
            vcs.snapshot(),
            generation,
            vcs.schema_version(),
            VcId::new(4),
            &environment,
        )
        .unwrap();
        assert_eq!(descriptor, expected);
        assert_eq!(descriptor.0, *vcs);
        assert_eq!(descriptor.1, term);
        assert_eq!(descriptor.5.vc(), VcId::new(4));
        let value = VcProgramValue {
            var: environment[0].0,
            definition: None,
        };
        assert_eq!(
            descriptor.3,
            Shape::AlgorithmReturns {
                algorithm: id,
                flow: mizar_core::control_flow::build_control_flow_ir(core).flow_map[&id],
                statement,
                input: value,
                result: value,
            }
        );
        assert_eq!(descriptor.4, Shape::True);
        assert!(std::ptr::eq(input.0, core));
        assert_eq!(input.1, id);
        assert!(std::ptr::eq(input.2, environment.as_slice()));
        assert_eq!((output.0, output.1), (statement, term));
        assert!(std::ptr::eq(output.2, &environment[0].2));
        let checked = mizar_kernel::checker::check_sole_return_var(core, id, &environment).unwrap();
        assert_eq!((checked.0, checked.1), (output.0, output.1));
        assert!(std::ptr::eq(checked.2, output.2));
        assert_eq!(
            consume(
                source,
                check,
                symbols,
                core,
                vcs.snapshot(),
                generation,
                vcs.schema_version(),
                VcId::new(4),
                &environment,
            )
            .unwrap(),
            descriptor
        );
        assert_eq!(*environment[0].2.lock().unwrap(), vec![255_u8, 0, 7]);
    });
}

#[test]
fn source_algorithm_e4_validator_handoff_uses_relocated_renamed_ids_and_fresh_binder() {
    use mizar_core::core_ir::{CoreAlgorithmStmtKind, CoreTermKind};
    use mizar_vc::{
        generator::{
            check_source_algorithm_e4_existential_request as existential,
            check_source_algorithm_e4_existential_result as existential_result,
            check_source_algorithm_e4_instantiated_request as instantiated,
            check_source_algorithm_e4_local_request as consume,
            generate_source_algorithm_e4_validator_handoff_request as handoff,
        },
        vc_ir::{ContextEntryKind, VcGeneratedFormulaShape as Shape},
    };
    let text = fs::read_to_string(config().workspace_root.join(
        "tests/miz/pass/algorithms/pass_proof_verification_algorithm_ensures_return_001.miz",
    ))
    .unwrap()
    .replace(" a ", " param ")
    .replace("(a)", "(param)")
    .replace(" a;", " param;")
    .replace("= a\n", "= param\n");
    with_source_algorithm_identity_inputs(
        &text,
        "owner-test",
        "nested.owners",
        7,
        |source, _, symbols, check| {
            let core = mizar_core::elaborator::lower_source_algorithms(check).unwrap();
            let (id, algorithm) = core.algorithms().iter().next().unwrap();
            let statement = algorithm.statements[0];
            let CoreAlgorithmStmtKind::Return(Some(term)) =
                core.algorithm_statements().get(statement).unwrap().kind
            else {
                panic!("actual return")
            };
            let environment = [(
                algorithm.params[0].var,
                algorithm.params[0].ty_guard.unwrap(),
                (),
            )];
            let (descriptor, input, output) = handoff(
                source,
                check,
                symbols,
                &core,
                snapshot_id(0),
                &GenerationSchemaVersion::new("relocated-generation"),
                &VcSchemaVersion::new("relocated-vc"),
                VcId::new(4),
                &environment,
            )
            .unwrap();
            assert_eq!(
                core.terms().get(term).unwrap().kind,
                CoreTermKind::Var(environment[0].0)
            );
            assert_eq!(
                consume(
                    source,
                    check,
                    symbols,
                    &core,
                    snapshot_id(0),
                    &GenerationSchemaVersion::new("relocated-generation"),
                    &VcSchemaVersion::new("relocated-vc"),
                    VcId::new(4),
                    &environment,
                )
                .unwrap(),
                descriptor
            );
            for checked in [instantiated, existential] {
                assert_eq!(
                    checked(
                        source,
                        check,
                        symbols,
                        &core,
                        snapshot_id(0),
                        &GenerationSchemaVersion::new("relocated-generation"),
                        &VcSchemaVersion::new("relocated-vc"),
                        VcId::new(4)
                    )
                    .unwrap(),
                    descriptor
                );
            }
            assert_eq!(input.1, id);
            assert!(std::ptr::eq(input.0, &core));
            assert!(std::ptr::eq(input.2, environment.as_slice()));
            assert_eq!((output.0, output.1), (statement, term));
            assert!(std::ptr::eq(output.2, &environment[0].2));
            let request = &descriptor.0.vcs()[descriptor.5.vc().index()];
            let Shape::Quantified { binders, .. } =
                direct_return_request_shape(&descriptor.0, request.goal)
            else {
                panic!("E4 existential")
            };
            assert_eq!(binders.as_slice(), &[descriptor.2]);
            let ContextEntryKind::BinderDeclaration { var, .. } =
                request.local_context.entries()[descriptor.2.index()].kind
            else {
                panic!("original fresh declaration")
            };
            let (paired, result) = existential_result(
                source,
                check,
                symbols,
                &core,
                snapshot_id(0),
                &GenerationSchemaVersion::new("relocated-generation"),
                &VcSchemaVersion::new("relocated-vc"),
                VcId::new(4),
            )
            .unwrap();
            assert_eq!(paired, descriptor);
            assert!(std::ptr::eq(result.core(), &core));
            assert_eq!(
                result.target(),
                (
                    id,
                    mizar_core::control_flow::build_control_flow_ir(&core).flow_map[&id],
                    statement,
                    (environment[0].0, None),
                    (var, None),
                )
            );
            assert_eq!(result.witness(), term);
            assert_eq!(result.context(), &[environment[0].1]);
            let (identity, slice, associated) =
                mizar_proof::status::check_source_algorithm_e4_existential(
                    source,
                    check,
                    symbols,
                    &core,
                    snapshot_id(0),
                    VcId::new(4),
                )
                .unwrap();
            assert_eq!(
                (identity, slice),
                mizar_proof::status::source_algorithm_direct_return_obligation_identity(
                    source,
                    check,
                    symbols,
                    snapshot_id(0),
                    VcId::new(4),
                )
                .unwrap()
            );
            assert!(std::ptr::eq(associated.core(), result.core()));
            assert_eq!(associated.target(), result.target());
            assert_eq!(associated.witness(), result.witness());
            assert_eq!(associated.context(), result.context());
            assert_ne!(var, environment[0].0);
            assert_ne!(var, algorithm.result.as_ref().unwrap().var);
            assert!(
                core.terms()
                    .iter()
                    .all(|(_, term)| !matches!(term.kind, CoreTermKind::Var(old) if old == var))
            );
        },
    );
}

#[test]
fn source_algorithm_e4_validator_handoff_refuses_malformed_environment_without_output() {
    use mizar_vc::generator::{
        check_source_algorithm_e4_local_request as consume,
        generate_source_algorithm_e4_validator_handoff_request as handoff,
    };
    with_direct_return_request_fixture(|source, check, symbols, core, vcs| {
        let (_, algorithm) = core.algorithms().iter().next().unwrap();
        let parameter = &algorithm.params[0];
        let result = algorithm.result.as_ref().unwrap();
        let input = (parameter.var, parameter.ty_guard.unwrap(), 17);
        let other = (result.var, result.ty_guard.unwrap(), 23);
        for environment in [
            vec![],
            vec![input, input],
            vec![input, other],
            vec![other],
            vec![(result.var, parameter.ty_guard.unwrap(), 17)],
            vec![(parameter.var, result.ty_guard.unwrap(), 17)],
        ] {
            assert_eq!(
                handoff(
                    source,
                    check,
                    symbols,
                    core,
                    vcs.snapshot(),
                    &vcs.vcs()[0].anchor.generation_schema_version,
                    vcs.schema_version(),
                    VcId::new(4),
                    &environment,
                )
                .unwrap_err(),
                "algorithms.direct_return.invalid_e4_validator_handoff_request"
            );
            assert_eq!(
                consume(
                    source,
                    check,
                    symbols,
                    core,
                    vcs.snapshot(),
                    &vcs.vcs()[0].anchor.generation_schema_version,
                    vcs.schema_version(),
                    VcId::new(4),
                    &environment,
                )
                .unwrap_err(),
                "algorithms.direct_return.invalid_e4_validator_handoff_request"
            );
        }
    });
}

#[test]
fn source_algorithm_e4_validator_handoff_preserves_selection_and_upstream_error_precedence() {
    use mizar_vc::generator::{
        check_source_algorithm_e4_existential_request as existential,
        check_source_algorithm_e4_existential_result as existential_result,
        check_source_algorithm_e4_instantiated_request as instantiated,
        check_source_algorithm_e4_local_request as consume,
        generate_source_algorithm_e4_operational_reduction_request as reduction,
        generate_source_algorithm_e4_validator_handoff_request as handoff,
    };
    with_direct_return_request_fixture(|source, check, symbols, core, vcs| {
        let environment: [(
            mizar_core::core_ir::CoreVarId,
            mizar_core::core_ir::CoreFormulaId,
            (),
        ); 0] = [];
        let generation = &vcs.vcs()[0].anchor.generation_schema_version;
        for selected in [0, 1, 2, 3, usize::MAX].map(VcId::new) {
            let error = handoff(
                source,
                check,
                symbols,
                core,
                vcs.snapshot(),
                generation,
                vcs.schema_version(),
                selected,
                &environment,
            )
            .unwrap_err();
            assert_eq!(
                consume(
                    source,
                    check,
                    symbols,
                    core,
                    vcs.snapshot(),
                    generation,
                    vcs.schema_version(),
                    selected,
                    &environment,
                )
                .unwrap_err(),
                error
            );
            for checked in [
                instantiated,
                existential,
                |source: &mizar_resolve::resolved_ast::SurfaceResolvedArena,
                 check: &mizar_checker::type_checker::SourceAlgorithmCheck<'_>,
                 symbols: &mizar_resolve::env::SymbolEnv,
                 core: &mizar_core::core_ir::CoreIr,
                 snapshot,
                 generation: &GenerationSchemaVersion,
                 schema: &VcSchemaVersion,
                 selected| {
                    existential_result(
                        source, check, symbols, core, snapshot, generation, schema, selected,
                    )
                    .map(|(descriptor, _)| descriptor)
                },
            ] {
                assert_eq!(
                    checked(
                        source,
                        check,
                        symbols,
                        core,
                        vcs.snapshot(),
                        generation,
                        vcs.schema_version(),
                        selected
                    )
                    .unwrap_err(),
                    error
                );
            }
            assert_eq!(
                mizar_proof::status::check_source_algorithm_e4_existential(
                    source,
                    check,
                    symbols,
                    core,
                    vcs.snapshot(),
                    selected,
                )
                .err()
                .unwrap(),
                error
            );
            assert_eq!(
                error,
                reduction(
                    source,
                    check,
                    symbols,
                    core,
                    vcs.snapshot(),
                    generation,
                    vcs.schema_version(),
                    selected,
                )
                .unwrap_err()
            );
        }
        let text = fs::read_to_string(config().workspace_root.join(
            "tests/miz/pass/algorithms/pass_proof_verification_algorithm_ensures_return_001.miz",
        ))
        .unwrap();
        for foreign in [
            format!(":: stale placement\n{text}"),
            text.replace("idalgo", "foreign_algorithm"),
            text.replace("terminating algorithm", "algorithm"),
        ] {
            with_source_algorithm_identity_inputs(
                &foreign,
                "owner-test",
                "owners",
                7,
                |other_source, _, other_symbols, other_check| {
                    let other_core =
                        mizar_core::elaborator::lower_source_algorithms(other_check).unwrap();
                    for (source, check, symbols, core) in [
                        (other_source, check, symbols, core),
                        (source, other_check, symbols, core),
                        (source, check, other_symbols, core),
                        (source, check, symbols, &other_core),
                    ]
                    .into_iter()
                    .chain((!other_check.declared_terminating()).then_some((
                        other_source,
                        other_check,
                        other_symbols,
                        &other_core,
                    ))) {
                        for selected in [VcId::new(4), VcId::new(usize::MAX)] {
                            let error = handoff(
                                source,
                                check,
                                symbols,
                                core,
                                vcs.snapshot(),
                                generation,
                                vcs.schema_version(),
                                selected,
                                &environment,
                            )
                            .unwrap_err();
                            assert_eq!(
                                consume(
                                    source,
                                    check,
                                    symbols,
                                    core,
                                    vcs.snapshot(),
                                    generation,
                                    vcs.schema_version(),
                                    selected,
                                    &environment,
                                )
                                .unwrap_err(),
                                error
                            );
                            for checked in [
                                instantiated,
                                existential,
                                |source: &mizar_resolve::resolved_ast::SurfaceResolvedArena,
                                 check: &mizar_checker::type_checker::SourceAlgorithmCheck<'_>,
                                 symbols: &mizar_resolve::env::SymbolEnv,
                                 core: &mizar_core::core_ir::CoreIr,
                                 snapshot,
                                 generation: &GenerationSchemaVersion,
                                 schema: &VcSchemaVersion,
                                 selected| {
                                    existential_result(
                                        source, check, symbols, core, snapshot, generation, schema,
                                        selected,
                                    )
                                    .map(|(descriptor, _)| descriptor)
                                },
                            ] {
                                assert_eq!(
                                    checked(
                                        source,
                                        check,
                                        symbols,
                                        core,
                                        vcs.snapshot(),
                                        generation,
                                        vcs.schema_version(),
                                        selected
                                    )
                                    .unwrap_err(),
                                    error
                                );
                            }
                            assert_eq!(
                                mizar_proof::status::check_source_algorithm_e4_existential(
                                    source,
                                    check,
                                    symbols,
                                    core,
                                    vcs.snapshot(),
                                    selected,
                                )
                                .err()
                                .unwrap(),
                                error
                            );
                            assert_eq!(
                                error,
                                reduction(
                                    source,
                                    check,
                                    symbols,
                                    core,
                                    vcs.snapshot(),
                                    generation,
                                    vcs.schema_version(),
                                    selected,
                                )
                                .unwrap_err()
                            );
                        }
                    }
                },
            );
        }
    });
}

#[test]
fn source_algorithm_e4_proof_association_preserves_original_result_raw_slice() {
    use mizar_proof::status::{
        check_source_algorithm_e4_existential as associate,
        source_algorithm_direct_return_obligation_identity as identity,
    };
    use mizar_vc::generator::check_source_algorithm_e4_existential_result as producer;
    with_direct_return_request_fixture(|source, check, symbols, core, vcs| {
        let before = format!("{source:?}{check:?}{symbols:?}{core:?}{vcs:?}");
        let generation = GenerationSchemaVersion::new("mizar-vc-generation-vc53-direct-return-v1");
        let schema = VcSchemaVersion::new("mizar-vc-vcset-vc53-direct-return-v1");
        for snapshot in [vcs.snapshot(), snapshot_id(1)] {
            let expected = identity(source, check, symbols, snapshot, VcId::new(4)).unwrap();
            let (descriptor, checked) = producer(
                source,
                check,
                symbols,
                core,
                snapshot,
                &generation,
                &schema,
                VcId::new(4),
            )
            .unwrap();
            let raw = source_algorithm_direct_return_slice_oracle(
                source,
                check,
                symbols,
                core,
                &descriptor.0,
                &generation,
            );
            assert_eq!(&expected.1, raw.slice_for(VcId::new(4)).unwrap());
            for _ in 0..2 {
                let (original, slice, result) =
                    associate(source, check, symbols, core, snapshot, VcId::new(4)).unwrap();
                assert_eq!((&original, &slice), (&expected.0, &expected.1));
                assert_eq!(slice, descriptor.5);
                assert_eq!(result.checker_schema_version(), 1);
                assert!(std::ptr::eq(result.core(), core));
                assert_eq!(result.target(), checked.target());
                assert_eq!(result.witness(), checked.witness());
                assert_eq!(result.context(), checked.context());
            }
        }
        assert_eq!(
            format!("{source:?}{check:?}{symbols:?}{core:?}{vcs:?}"),
            before
        );
    });
}

#[test]
fn source_algorithm_e4_policy_consumer_preserves_association_and_current_fingerprint() {
    use mizar_proof::{
        policy::{KernelEvidenceFormat, VerifierPolicy},
        status::{
            check_source_algorithm_e4_existential as associate,
            consume_source_algorithm_e4_existential as consume,
        },
    };
    with_direct_return_request_fixture(|source, check, symbols, core, vcs| {
        let before = format!("{source:?}{check:?}{symbols:?}{core:?}{vcs:?}");
        let expected =
            associate(source, check, symbols, core, vcs.snapshot(), VcId::new(4)).unwrap();
        let default = VerifierPolicy::default();
        for policy in [
            default.clone(),
            default
                .clone()
                .with_kernel_evidence_formats([KernelEvidenceFormat::BuiltinKernelEvidence]),
            VerifierPolicy::development(),
            VerifierPolicy::interactive(),
            default.clone().with_profile_id("changed-e4-policy"),
            default.clone().with_schema_version(2),
        ] {
            let policy_before = policy.clone();
            for _ in 0..2 {
                let actual = consume(
                    source,
                    check,
                    symbols,
                    core,
                    vcs.snapshot(),
                    VcId::new(4),
                    &policy,
                )
                .unwrap();
                assert_eq!((&actual.0, &actual.1), (&expected.0, &expected.1));
                assert!(std::ptr::eq(actual.2.core(), expected.2.core()));
                assert_eq!(actual.2.target(), expected.2.target());
                assert_eq!(actual.2.witness(), expected.2.witness());
                assert_eq!(actual.2.context(), expected.2.context());
                assert_eq!(
                    actual.2.checker_schema_version(),
                    expected.2.checker_schema_version()
                );
                assert_eq!(actual.3, policy.policy_fingerprint());
                if policy != default {
                    assert_ne!(actual.3, default.policy_fingerprint());
                }
            }
            assert_eq!(policy, policy_before);
        }
        assert_eq!(
            format!("{source:?}{check:?}{symbols:?}{core:?}{vcs:?}"),
            before
        );
    });
}

#[test]
fn source_algorithm_e4_policy_consumer_refuses_policy_after_upstream_errors() {
    use mizar_proof::{
        policy::{KernelEvidenceFormat, VerifierPolicy},
        status::{
            check_source_algorithm_e4_existential as associate,
            consume_source_algorithm_e4_existential as consume,
        },
    };
    with_direct_return_request_fixture(|source, check, symbols, core, vcs| {
        let before = format!("{source:?}{check:?}{symbols:?}{core:?}{vcs:?}");
        let genuine =
            associate(source, check, symbols, core, vcs.snapshot(), VcId::new(4)).unwrap();
        let schema = genuine.2.checker_schema_version();
        let upstream = [VcId::new(3), VcId::new(99)].map(|selected| {
            let error = associate(source, check, symbols, core, vcs.snapshot(), selected)
                .err()
                .unwrap();
            assert_ne!(error, "algorithm.proof.native_e4_policy_rejected");
            (selected, error)
        });
        let default = VerifierPolicy::default();
        for policy in [
            default.clone().with_kernel_evidence_formats([]),
            default
                .clone()
                .with_kernel_evidence_formats([KernelEvidenceFormat::FormulaSubstitution]),
            default.clone().with_checker_schema_version(schema - 1),
            default.with_checker_schema_version(schema + 1),
        ] {
            let policy_before = policy.clone();
            assert_eq!(
                consume(
                    source,
                    check,
                    symbols,
                    core,
                    vcs.snapshot(),
                    VcId::new(4),
                    &policy,
                )
                .err()
                .unwrap(),
                "algorithm.proof.native_e4_policy_rejected"
            );
            for (selected, error) in &upstream {
                assert_eq!(
                    consume(
                        source,
                        check,
                        symbols,
                        core,
                        vcs.snapshot(),
                        *selected,
                        &policy,
                    )
                    .err()
                    .unwrap(),
                    *error
                );
            }
            assert_eq!(policy, policy_before);
        }
        assert_eq!(
            format!("{source:?}{check:?}{symbols:?}{core:?}{vcs:?}"),
            before
        );
    });
}

#[test]
fn source_algorithm_native_e4_preserves_open_requests_and_purity() {
    use mizar_proof::{policy::VerifierPolicy, status::NativeE4NormalReturn};
    use mizar_vc::vc_ir::VcStatus;
    with_direct_return_request_fixture(|source, check, symbols, core, vcs| {
        let policy = VerifierPolicy::default();
        let before = format!("{source:?}{check:?}{symbols:?}{core:?}{vcs:?}{policy:?}");
        for _ in 0..2 {
            let native = NativeE4NormalReturn::from_source(
                source,
                check,
                symbols,
                core,
                vcs.snapshot(),
                VcId::new(4),
                &policy,
            )
            .unwrap();
            for _ in 0..2 {
                assert!(
                    native
                        .adopt_original_request(
                            source,
                            check,
                            symbols,
                            core,
                            vcs.snapshot(),
                            VcId::new(4),
                            &policy,
                        )
                        .unwrap()
                );
            }
        }
        assert_eq!(vcs.vcs()[1..].len(), 4);
        assert!(vcs.vcs()[1..].iter().all(|vc| vc.status == VcStatus::Open));
        assert!(vcs.seed_accounting().iter().any(|seed|
            seed.seed_status == mizar_core::core_ir::ObligationSeedStatus::Deferred
        ));
        assert_eq!(
            format!("{source:?}{check:?}{symbols:?}{core:?}{vcs:?}{policy:?}"),
            before
        );
    });
}

#[test]
fn source_algorithm_native_e4_refuses_other_requests_and_changed_raw_associations() {
    use mizar_proof::{
        policy::VerifierPolicy,
        status::{
            NativeE4NormalReturn, consume_source_algorithm_e4_existential as consume,
            source_algorithm_direct_return_obligation_identity as identity,
        },
    };
    with_direct_return_request_fixture(|source, check, symbols, core, vcs| {
        let policy = VerifierPolicy::default();
        let denied = policy.clone().with_kernel_evidence_formats([]);
        let native = NativeE4NormalReturn::from_source(
            source,
            check,
            symbols,
            core,
            vcs.snapshot(),
            VcId::new(4),
            &policy,
        )
        .unwrap();
        for selected in [0, 1, 2, 3, usize::MAX].map(VcId::new) {
            let expected = consume(
                source,
                check,
                symbols,
                core,
                vcs.snapshot(),
                selected,
                &policy,
            )
            .err()
            .unwrap();
            for policy in [&policy, &denied] {
                assert_eq!(
                    NativeE4NormalReturn::from_source(
                        source,
                        check,
                        symbols,
                        core,
                        vcs.snapshot(),
                        selected,
                        policy,
                    )
                    .err()
                    .unwrap(),
                    expected
                );
                let adopted = native.adopt_original_request(
                    source,
                    check,
                    symbols,
                    core,
                    vcs.snapshot(),
                    selected,
                    policy,
                );
                match identity(source, check, symbols, vcs.snapshot(), selected) {
                    Ok(_) => assert!(!adopted.unwrap()),
                    Err(error) => assert_eq!(adopted.unwrap_err(), error),
                }
            }
        }
        let baseline = identity(source, check, symbols, vcs.snapshot(), VcId::new(4)).unwrap();
        let text = fs::read_to_string(config().workspace_root.join(
            "tests/miz/pass/algorithms/pass_proof_verification_algorithm_ensures_return_001.miz",
        ))
        .unwrap();
        with_source_algorithm_identity_inputs(
            &text.replace("ensures result = a", "ensures a = result"),
            "owner-test",
            "owners",
            0,
            |changed_source, _, changed_symbols, changed_check| {
                let changed = identity(
                    changed_source,
                    changed_check,
                    changed_symbols,
                    vcs.snapshot(),
                    VcId::new(4),
                )
                .unwrap();
                assert_ne!(changed.0, baseline.0);
                assert_ne!(changed.1, baseline.1);
                for policy in [&policy, &denied] {
                    // Retain the original Core so refusal exercises regenerated association.
                    assert!(
                        !native
                            .adopt_original_request(
                                changed_source,
                                changed_check,
                                changed_symbols,
                                core,
                                vcs.snapshot(),
                                VcId::new(4),
                                policy,
                            )
                            .unwrap()
                    );
                }
            },
        );
    });
}

#[test]
fn source_algorithm_native_e4_refuses_snapshot_and_distinct_equal_core() {
    use mizar_proof::{policy::VerifierPolicy, status::NativeE4NormalReturn};
    with_direct_return_request_fixture(|source, check, symbols, core, vcs| {
        let policy = VerifierPolicy::default();
        let denied = policy.clone().with_kernel_evidence_formats([]);
        let native = NativeE4NormalReturn::from_source(
            source,
            check,
            symbols,
            core,
            vcs.snapshot(),
            VcId::new(4),
            &policy,
        )
        .unwrap();
        let rebuilt = mizar_core::elaborator::lower_source_algorithms(check).unwrap();
        let cloned = core.clone();
        assert_eq!(&rebuilt, core);
        assert_eq!(&cloned, core);
        for (candidate_core, snapshot) in [
            (core, snapshot_id(1)),
            (&rebuilt, vcs.snapshot()),
            (&cloned, vcs.snapshot()),
        ] {
            for policy in [&policy, &denied] {
                assert!(
                    !native
                        .adopt_original_request(
                            source,
                            check,
                            symbols,
                            candidate_core,
                            snapshot,
                            VcId::new(4),
                            policy,
                        )
                        .unwrap()
                );
            }
        }
        assert!(
            native
                .adopt_original_request(
                    source,
                    check,
                    symbols,
                    core,
                    vcs.snapshot(),
                    VcId::new(4),
                    &policy,
                )
                .unwrap()
        );
    });
}

#[test]
fn source_algorithm_native_e4_rechecks_current_builtin_policy_and_checker_schema() {
    use mizar_proof::{
        policy::{KernelEvidenceFormat, VerifierPolicy},
        status::NativeE4NormalReturn,
    };
    with_direct_return_request_fixture(|source, check, symbols, core, vcs| {
        let default = VerifierPolicy::default();
        let native = NativeE4NormalReturn::from_source(
            source,
            check,
            symbols,
            core,
            vcs.snapshot(),
            VcId::new(4),
            &default,
        )
        .unwrap();
        for policy in [
            default.clone(),
            default
                .clone()
                .with_kernel_evidence_formats([KernelEvidenceFormat::BuiltinKernelEvidence]),
            VerifierPolicy::development(),
            VerifierPolicy::interactive(),
            default.clone().with_profile_id("changed-native-e4-policy"),
            default.clone().with_schema_version(2),
        ] {
            let before = policy.clone();
            let current = NativeE4NormalReturn::from_source(
                source,
                check,
                symbols,
                core,
                vcs.snapshot(),
                VcId::new(4),
                &policy,
            )
            .unwrap();
            for capability in [&native, &current] {
                assert!(
                    capability
                        .adopt_original_request(
                            source,
                            check,
                            symbols,
                            core,
                            vcs.snapshot(),
                            VcId::new(4),
                            &policy,
                        )
                        .unwrap()
                );
            }
            assert_eq!(policy, before);
        }
        let schema = mizar_proof::status::consume_source_algorithm_e4_existential(
            source,
            check,
            symbols,
            core,
            vcs.snapshot(),
            VcId::new(4),
            &default,
        )
        .unwrap()
        .2
        .checker_schema_version();
        for policy in [
            default.clone().with_kernel_evidence_formats([]),
            default
                .clone()
                .with_kernel_evidence_formats([KernelEvidenceFormat::FormulaSubstitution]),
            default.clone().with_checker_schema_version(schema - 1),
            default.clone().with_checker_schema_version(schema + 1),
        ] {
            let before = policy.clone();
            assert_eq!(
                NativeE4NormalReturn::from_source(
                    source,
                    check,
                    symbols,
                    core,
                    vcs.snapshot(),
                    VcId::new(4),
                    &policy,
                )
                .err()
                .unwrap(),
                "algorithm.proof.native_e4_policy_rejected"
            );
            assert_eq!(
                native
                    .adopt_original_request(
                        source,
                        check,
                        symbols,
                        core,
                        vcs.snapshot(),
                        VcId::new(4),
                        &policy,
                    )
                    .unwrap_err(),
                "algorithm.proof.native_e4_policy_rejected"
            );
            assert_eq!(policy, before);
        }
        assert!(
            native
                .adopt_original_request(
                    source,
                    check,
                    symbols,
                    core,
                    vcs.snapshot(),
                    VcId::new(4),
                    &default,
                )
                .unwrap()
        );
    });
}

#[test]
fn source_algorithm_native_e4_preserves_producer_and_regeneration_errors_before_policy() {
    use mizar_proof::{
        policy::VerifierPolicy,
        status::{
            NativeE4NormalReturn, consume_source_algorithm_e4_existential as consume,
            source_algorithm_direct_return_obligation_identity as identity,
        },
    };
    with_direct_return_request_fixture(|source, check, symbols, core, vcs| {
        let policy = VerifierPolicy::default();
        let denied = policy.clone().with_kernel_evidence_formats([]);
        let native = NativeE4NormalReturn::from_source(
            source,
            check,
            symbols,
            core,
            vcs.snapshot(),
            VcId::new(4),
            &policy,
        )
        .unwrap();
        let text = fs::read_to_string(config().workspace_root.join(
            "tests/miz/pass/algorithms/pass_proof_verification_algorithm_ensures_return_001.miz",
        ))
        .unwrap();
        with_source_algorithm_identity_inputs(
            &text.replace("idalgo", "foreign_algorithm"),
            "owner-test",
            "owners",
            7,
            |other_source, _, other_symbols, other_check| {
                let other_core =
                    mizar_core::elaborator::lower_source_algorithms(other_check).unwrap();
                for (source, check, symbols, core) in [
                    (other_source, check, symbols, core),
                    (source, other_check, other_symbols, core),
                    (source, check, symbols, &other_core),
                ] {
                    let before = format!("{source:?}{check:?}{symbols:?}{core:?}");
                    for selected in [VcId::new(4), VcId::new(usize::MAX)] {
                        let expected = consume(
                            source,
                            check,
                            symbols,
                            core,
                            vcs.snapshot(),
                            selected,
                            &policy,
                        )
                        .err()
                        .unwrap();
                        assert_ne!(expected, "algorithm.proof.native_e4_policy_rejected");
                        for policy in [&policy, &denied] {
                            assert_eq!(
                                NativeE4NormalReturn::from_source(
                                    source,
                                    check,
                                    symbols,
                                    core,
                                    vcs.snapshot(),
                                    selected,
                                    policy,
                                )
                                .err()
                                .unwrap(),
                                expected
                            );
                            let adopted = native.adopt_original_request(
                                source,
                                check,
                                symbols,
                                core,
                                vcs.snapshot(),
                                selected,
                                policy,
                            );
                            match identity(source, check, symbols, vcs.snapshot(), selected) {
                                Err(error) => assert_eq!(adopted.unwrap_err(), error),
                                Ok(_) => assert!(!adopted.unwrap()),
                            }
                        }
                    }
                    assert_eq!(format!("{source:?}{check:?}{symbols:?}{core:?}"), before);
                }
            },
        );
    });
}

#[test]
fn source_algorithm_e4_validator_handoff_preserves_schema_identity_status_and_purity() {
    use mizar_vc::{
        dependency_slice::DependencySliceCompleteness,
        generator::{
            check_source_algorithm_e4_existential_request as existential,
            check_source_algorithm_e4_existential_result as existential_result,
            check_source_algorithm_e4_local_request as consume,
            generate_source_algorithm_e4_validator_handoff_request as handoff,
        },
        vc_ir::{VcKind, VcStatus},
    };
    with_direct_return_request_fixture(|source, check, symbols, core, vcs| {
        let (_, algorithm) = core.algorithms().iter().next().unwrap();
        let environment = [(
            algorithm.params[0].var,
            algorithm.params[0].ty_guard.unwrap(),
            vec![1, 4, 9],
        )];
        let before = format!("{source:?}{check:?}{symbols:?}{core:?}{vcs:?}{environment:?}");
        let generation = &vcs.vcs()[0].anchor.generation_schema_version;
        for (snapshot, generation, schema) in [
            (
                vcs.snapshot(),
                generation.clone(),
                vcs.schema_version().clone(),
            ),
            (
                snapshot_id(1),
                generation.clone(),
                vcs.schema_version().clone(),
            ),
            (
                vcs.snapshot(),
                GenerationSchemaVersion::new("isolated-generation"),
                vcs.schema_version().clone(),
            ),
            (
                vcs.snapshot(),
                generation.clone(),
                VcSchemaVersion::new("isolated-vc"),
            ),
        ] {
            let identity = mizar_proof::status::source_algorithm_direct_return_obligation_identity(
                source,
                check,
                symbols,
                snapshot,
                VcId::new(4),
            )
            .unwrap();
            let request = || {
                handoff(
                    source,
                    check,
                    symbols,
                    core,
                    snapshot,
                    &generation,
                    &schema,
                    VcId::new(4),
                    &environment,
                )
                .unwrap()
            };
            let (original, _, output) = request();
            let checked = || {
                consume(
                    source,
                    check,
                    symbols,
                    core,
                    snapshot,
                    &generation,
                    &schema,
                    VcId::new(4),
                    &environment,
                )
                .unwrap()
            };
            let descriptor = checked();
            assert_eq!(descriptor, original);
            assert_eq!(
                existential(
                    source,
                    check,
                    symbols,
                    core,
                    snapshot,
                    &generation,
                    &schema,
                    VcId::new(4)
                )
                .unwrap(),
                descriptor
            );
            let result = || {
                existential_result(
                    source,
                    check,
                    symbols,
                    core,
                    snapshot,
                    &generation,
                    &schema,
                    VcId::new(4),
                )
                .unwrap()
            };
            for (paired, checked) in [result(), result()] {
                assert_eq!(paired, descriptor);
                assert!(std::ptr::eq(checked.core(), core));
                assert_eq!(checked.witness(), descriptor.1);
                assert_eq!(checked.context(), &[environment[0].1]);
            }
            assert_eq!(checked(), descriptor);
            assert_eq!(request().0, descriptor);
            assert!(std::ptr::eq(output.2, &environment[0].2));
            assert_eq!(descriptor.0.snapshot(), snapshot);
            assert_eq!(descriptor.0.schema_version(), &schema);
            assert!(
                descriptor
                    .0
                    .vcs()
                    .iter()
                    .all(|vc| vc.anchor.generation_schema_version == generation)
            );
            let raw = source_algorithm_direct_return_slice_oracle(
                source,
                check,
                symbols,
                core,
                &descriptor.0,
                &generation,
            );
            assert_eq!(&descriptor.5, raw.slice_for(VcId::new(4)).unwrap());
            // Proof's public identity uses its fixed schemas; the handoff keeps caller schemas.
            assert_eq!(descriptor.5.vc(), identity.1.vc());
            assert_eq!(descriptor.5.status(), &VcStatus::Open);
            assert_eq!(
                descriptor.5.completeness(),
                DependencySliceCompleteness::IncompleteUncacheable
            );
            assert!(descriptor.5.requires_cache_miss());
            assert_eq!(
                descriptor
                    .0
                    .vcs()
                    .iter()
                    .filter(|vc| matches!(
                        vc.kind,
                        VcKind::AlgorithmE1Typing
                            | VcKind::AlgorithmE1SingleValuedness
                            | VcKind::AlgorithmE1ResultInhabitation
                            | VcKind::AlgorithmE4NormalReturn
                    ))
                    .count(),
                4
            );
            assert!(
                descriptor
                    .0
                    .vcs()
                    .iter()
                    .all(|vc| vc.status == VcStatus::Open && vc.proof_hint.is_none())
            );
            assert_eq!(descriptor.0.seed_accounting(), vcs.seed_accounting());
            assert!(descriptor.0.seed_accounting().iter().any(
                |seed| seed.seed_status == mizar_core::core_ir::ObligationSeedStatus::Deferred
            ));
            assert_eq!(
                mizar_proof::status::source_algorithm_direct_return_obligation_identity(
                    source,
                    check,
                    symbols,
                    snapshot,
                    descriptor.5.vc(),
                )
                .unwrap(),
                identity
            );
        }
        assert_eq!(
            format!("{source:?}{check:?}{symbols:?}{core:?}{vcs:?}{environment:?}"),
            before
        );
    });
}

#[test]
fn source_algorithm_direct_return_slices_reject_modified_graph_atomically() {
    use mizar_vc::{
        dependency_slice::try_compute_source_algorithm_direct_return_dependency_slices as collect,
        vc_ir::{
            ContextEntryKind, LocalContext, VcGeneratedFormulaShape as Shape, VcSet, VcSetParts,
        },
    };
    with_direct_return_request_fixture(|source, check, symbols, core, vcs| {
        let before = format!("{source:?}{check:?}{symbols:?}{core:?}{vcs:?}");
        let generation = &vcs.vcs()[0].anchor.generation_schema_version;
        source_algorithm_direct_return_slice_oracle(source, check, symbols, core, vcs, generation);
        for mutation in 0..9 {
            let mut parts = VcSetParts {
                schema_version: vcs.schema_version().clone(),
                snapshot: vcs.snapshot(),
                source: vcs.source(),
                module: vcs.module().clone(),
                generated_formulas: vcs.generated_formulas().to_vec(),
                vcs: vcs.vcs().to_vec(),
                seed_accounting: vcs.seed_accounting().to_vec(),
            };
            let relation = parts
                .generated_formulas
                .iter_mut()
                .find(|formula| matches!(formula.shape, Shape::AlgorithmReturns { .. }))
                .unwrap();
            let Shape::AlgorithmReturns {
                algorithm,
                flow,
                statement,
                input,
                result,
            } = &mut relation.shape
            else {
                panic!("relation")
            };
            match mutation {
                0 => *algorithm = mizar_core::core_ir::CoreAlgorithmId::new(algorithm.index() + 1),
                1 => *flow = mizar_core::control_flow::ControlFlowId::new(flow.index() + 1),
                2 => {
                    *statement =
                        mizar_core::core_ir::CoreAlgorithmStmtId::new(statement.index() + 1)
                }
                3 => input.var = result.var,
                4 => result.var = input.var,
                5 => input.definition = Some(*statement),
                6 => result.definition = Some(*statement),
                7 | 8 => {
                    let context = &parts.vcs[1].local_context;
                    let mut entries = context.entries().to_vec();
                    let declaration = entries.iter_mut().find(|entry| matches!(
                        &entry.kind, ContextEntryKind::BinderDeclaration { role, .. } if role.as_str() == "result"
                    )).unwrap();
                    if mutation == 7 {
                        let ContextEntryKind::BinderDeclaration { role, .. } =
                            &mut declaration.kind
                        else {
                            unreachable!()
                        };
                        *role = "parameter".into();
                    } else {
                        let ContextEntryKind::BinderDeclaration { var, .. } = &mut declaration.kind
                        else {
                            unreachable!()
                        };
                        *var = input.var;
                    }
                    parts.vcs[1].local_context =
                        LocalContext::try_new(entries, context.policy_inputs().to_vec()).unwrap();
                }
                _ => unreachable!(),
            }
            let changed = VcSet::try_new(parts).unwrap();
            assert_eq!(
                collect(source, check, symbols, core, &changed, generation).unwrap_err(),
                "algorithms.direct_return.mismatched_dependency_input",
                "mutation {mutation}"
            );
        }
        assert_eq!(
            collect(
                source,
                check,
                symbols,
                core,
                vcs,
                &GenerationSchemaVersion::new("other-generation")
            )
            .unwrap_err(),
            "algorithms.direct_return.mismatched_dependency_input"
        );
        assert_eq!(
            format!("{source:?}{check:?}{symbols:?}{core:?}{vcs:?}"),
            before
        );
    });
}

#[test]
fn source_algorithm_object_inhabitation_checks_resolved_child_context_and_substitution() {
    use mizar_core::core_ir::{
        CoreAlgorithmStmtKind, CoreFormulaId, CoreFormulaKind, CoreIr, CoreTermId, CoreTermKind,
        CoreVarId,
    };
    use mizar_kernel::checker::{
        check_object_inhabitation_result as checked, check_object_inhabitation_witness as introduce,
    };
    with_direct_return_request_fixture(|_, _, _, core, _| {
        let algorithm = core.algorithms().iter().next().unwrap().1;
        let CoreAlgorithmStmtKind::Return(Some(witness)) = core
            .algorithm_statements()
            .get(algorithm.statements[0])
            .unwrap()
            .kind
        else {
            panic!("actual witness")
        };
        let guard = algorithm.params[0].ty_guard.unwrap();
        let result_guard = algorithm.result.as_ref().unwrap().ty_guard.unwrap();
        let fresh = CoreVarId::new(
            core.terms()
                .iter()
                .filter_map(|(_, term)| match term.kind {
                    CoreTermKind::Var(var) => Some(var.index()),
                    _ => None,
                })
                .max()
                .unwrap()
                + 1,
        );
        let target = (fresh, fresh, "object");
        assert!(introduce(core, target, witness, guard, &[guard]));
        let result = {
            let context = vec![guard];
            checked(core, target, witness, guard, &context).unwrap()
        };
        assert!(std::ptr::eq(result.core(), core));
        assert_eq!(result.binder(), fresh);
        assert_eq!(result.context(), &[guard]);
        for (target, witness, child, context) in [
            (target, witness, guard, vec![]),
            (target, witness, guard, vec![guard, guard]),
            (target, witness, guard, vec![result_guard]),
            (target, witness, result_guard, vec![guard]),
            (
                target,
                witness,
                CoreFormulaId::new(core.formulas().len()),
                vec![guard],
            ),
            (
                target,
                CoreTermId::new(core.terms().len()),
                guard,
                vec![guard],
            ),
            (
                (fresh, algorithm.params[0].var, "object"),
                witness,
                guard,
                vec![guard],
            ),
            ((fresh, fresh, "set"), witness, guard, vec![guard]),
            (
                (algorithm.params[0].var, algorithm.params[0].var, "object"),
                witness,
                guard,
                vec![guard],
            ),
        ] {
            assert!(!introduce(core, target, witness, child, &context));
            assert!(checked(core, target, witness, child, &context).is_none());
        }
        // Equal IDs supply no proof when the resolved assumption/child has changed meaning.
        for kind in [
            CoreFormulaKind::False,
            CoreFormulaKind::True,
            CoreFormulaKind::TypePred {
                subject: witness,
                ty: "set".into(),
            },
        ] {
            let mut parts = sole_return_core_parts(core);
            parts.formulas.get_mut(guard).unwrap().kind = kind;
            let changed = CoreIr::try_new(parts).unwrap();
            assert!(!introduce(&changed, target, witness, guard, &[guard]));
            assert!(checked(&changed, target, witness, guard, &[guard]).is_none());
        }
        let mut parts = sole_return_core_parts(core);
        parts.terms.get_mut(witness).unwrap().kind = CoreTermKind::Numeral("0".into());
        let changed = CoreIr::try_new(parts).unwrap();
        assert!(!introduce(&changed, target, witness, guard, &[guard]));
        assert!(checked(&changed, target, witness, guard, &[guard]).is_none());

        // Different term/formula IDs retain the assumption rule when their meanings agree.
        let mut parts = sole_return_core_parts(core);
        let alias = parts
            .terms
            .insert(core.terms().get(witness).unwrap().clone());
        parts
            .source_map
            .term_sources
            .insert(alias, core.terms().get(witness).unwrap().source.clone());
        let mut child = core.formulas().get(guard).unwrap().clone();
        child.kind = CoreFormulaKind::TypePred {
            subject: alias,
            ty: "object".into(),
        };
        let child = parts.formulas.insert(child);
        parts
            .source_map
            .formula_sources
            .insert(child, core.formulas().get(guard).unwrap().source.clone());
        let aliased = CoreIr::try_new(parts).unwrap();
        assert!(introduce(&aliased, target, alias, child, &[guard]));
        let aliased_result = checked(&aliased, target, alias, child, &[guard]).unwrap();
        assert!(std::ptr::eq(aliased_result.core(), &aliased));
        assert!(!std::ptr::eq(aliased_result.core(), result.core()));
        assert_eq!(aliased_result.binder(), fresh);
        assert_eq!(aliased_result.context(), &[guard]);
        // A term alias with a different variable cannot instantiate the original body.
        let mut parts = sole_return_core_parts(&aliased);
        parts.terms.get_mut(alias).unwrap().kind =
            CoreTermKind::Var(algorithm.result.as_ref().unwrap().var);
        let changed = CoreIr::try_new(parts).unwrap();
        assert!(!introduce(&changed, target, alias, child, &[guard]));
        assert!(checked(&changed, target, alias, child, &[guard]).is_none());
    });
}

#[test]
fn source_algorithm_e4_instantiated_request_preserves_original_and_checks_symbolic_child() {
    use mizar_core::core_ir::{CoreAlgorithmStmtKind, CoreTermKind};
    use mizar_vc::{
        dependency_slice::DependencySliceCompleteness,
        generator::{
            check_source_algorithm_e4_existential_request as existential,
            check_source_algorithm_e4_existential_result as existential_result,
            check_source_algorithm_e4_instantiated_request as checked,
            generate_source_algorithm_e4_operational_reduction_request as reduction,
        },
        vc_ir::{
            ContextEntryKind, PremiseRef, VcFormulaRef, VcGeneratedFormulaShape as Shape, VcStatus,
        },
    };
    with_direct_return_request_fixture(|source, check, symbols, core, vcs| {
        let before = format!("{source:?}{check:?}{symbols:?}{core:?}{vcs:?}");
        let generation = &vcs.vcs()[0].anchor.generation_schema_version;
        let produce = || {
            reduction(
                source,
                check,
                symbols,
                core,
                vcs.snapshot(),
                generation,
                vcs.schema_version(),
                VcId::new(4),
            )
            .unwrap()
        };
        let consume = || {
            checked(
                source,
                check,
                symbols,
                core,
                vcs.snapshot(),
                generation,
                vcs.schema_version(),
                VcId::new(4),
            )
            .unwrap()
        };
        let original = produce();
        let actual = consume();
        let introduce = || {
            existential(
                source,
                check,
                symbols,
                core,
                vcs.snapshot(),
                generation,
                vcs.schema_version(),
                VcId::new(4),
            )
            .unwrap()
        };
        let introduced = introduce();
        assert_eq!(introduced, original);
        assert_eq!(introduce(), introduced);
        assert_eq!(actual, original);
        assert_eq!(consume(), actual);
        assert_eq!(produce(), original);
        let (id, algorithm) = core.algorithms().iter().next().unwrap();
        let statement = algorithm.statements[0];
        let CoreAlgorithmStmtKind::Return(Some(witness)) =
            core.algorithm_statements().get(statement).unwrap().kind
        else {
            panic!("actual return")
        };
        let guard = algorithm.params[0].ty_guard.unwrap();
        let value = (algorithm.params[0].var, None);
        let flow = mizar_core::control_flow::build_control_flow_ir(core).flow_map[&id];
        assert_eq!(actual.1, witness);
        assert_eq!(
            core.terms().get(witness).unwrap().kind,
            CoreTermKind::Var(value.0)
        );
        let Shape::AlgorithmReturns {
            algorithm: child_algorithm,
            flow: child_flow,
            statement: child_statement,
            input,
            result,
        } = actual.3
        else {
            panic!("instantiated relation")
        };
        assert_eq!(
            (
                child_algorithm,
                child_flow,
                child_statement,
                (input.var, input.definition),
                (result.var, result.definition)
            ),
            (id, flow, statement, value, value)
        );
        // The independent oracle uses Core's actual return and CFG, not descriptor fields.
        assert!(mizar_kernel::checker::check_instantiated_algorithm_returns(
            core,
            (id, flow, statement, value, value),
            witness,
            &[guard]
        ));
        let request = &actual.0.vcs()[actual.5.vc().index()];
        let entries = request.local_context.entries();
        assert_eq!(entries[0].kind, ContextEntryKind::CheckerFact);
        assert_eq!(entries[0].formula, Some(VcFormulaRef::Core(guard)));
        assert_eq!(request.premises, [PremiseRef::LocalContext(entries[0].id)]);
        let Shape::Quantified { binders, body, .. } =
            direct_return_request_shape(&actual.0, request.goal)
        else {
            panic!("original existential")
        };
        assert_eq!(binders, &[actual.2]);
        let ContextEntryKind::BinderDeclaration { var, ref role } = entries[actual.2.index()].kind
        else {
            panic!("original result binder")
        };
        assert_eq!(role.as_str(), "result");
        assert_ne!(var, value.0);
        assert_ne!(var, algorithm.result.as_ref().unwrap().var);
        assert!(
            core.terms()
                .iter()
                .all(|(_, term)| !matches!(term.kind, CoreTermKind::Var(old) if old == var))
        );
        let target = (id, flow, statement, value, (var, None));
        assert!(
            matches!(direct_return_request_shape(&actual.0, *body), Shape::AlgorithmReturns { algorithm, flow: original_flow, statement: original_statement, input, result } if (*algorithm, *original_flow, *original_statement, (input.var, input.definition), (result.var, result.definition)) == target)
        );
        assert!(mizar_kernel::checker::check_algorithm_returns_existential(
            core,
            target,
            var,
            witness,
            &[guard]
        ));
        let (paired, result) = existential_result(
            source,
            check,
            symbols,
            core,
            vcs.snapshot(),
            generation,
            vcs.schema_version(),
            VcId::new(4),
        )
        .unwrap();
        assert_eq!(paired, introduced);
        assert_eq!(result.checker_schema_version(), 1);
        assert!(std::ptr::eq(result.core(), core));
        assert_eq!(result.target(), target);
        assert_eq!(result.witness(), witness);
        assert_eq!(result.context(), &[guard]);
        assert_eq!(actual.4, Shape::True);
        let raw = source_algorithm_direct_return_slice_oracle(
            source, check, symbols, core, &actual.0, generation,
        );
        assert_eq!(&actual.5, raw.slice_for(VcId::new(4)).unwrap());
        assert!(!actual.5.unknowns().is_empty());
        assert_eq!(
            actual.5.completeness(),
            DependencySliceCompleteness::IncompleteUncacheable
        );
        assert!(actual.5.requires_cache_miss());
        assert_eq!(actual.0, *vcs);
        assert_eq!(actual.0.vcs().len(), 5);
        assert!(
            actual
                .0
                .vcs()
                .iter()
                .all(|vc| vc.status == VcStatus::Open && vc.proof_hint.is_none())
        );
        assert!(actual.0.seed_accounting().iter().any(|seed| seed.seed_status == mizar_core::core_ir::ObligationSeedStatus::Deferred));
        assert_eq!(
            format!("{source:?}{check:?}{symbols:?}{core:?}{vcs:?}"),
            before
        );
    });
}

#[test]
fn source_algorithm_e4_instantiated_kernel_refuses_wrong_child_and_resolved_guards() {
    use mizar_core::{
        control_flow::{ControlFlowId, build_control_flow_ir},
        core_ir::{
            CoreAlgorithmId, CoreAlgorithmStmtId, CoreAlgorithmStmtKind, CoreFormulaKind, CoreIr,
            CoreTermId, CoreTermKind,
        },
    };
    use mizar_kernel::checker::check_instantiated_algorithm_returns as checked;
    with_direct_return_request_fixture(|_, _, _, core, _| {
        let (id, algorithm) = core.algorithms().iter().next().unwrap();
        let statement = algorithm.statements[0];
        let CoreAlgorithmStmtKind::Return(Some(witness)) =
            core.algorithm_statements().get(statement).unwrap().kind
        else {
            panic!("actual return")
        };
        let flow = build_control_flow_ir(core).flow_map[&id];
        let value = (algorithm.params[0].var, None);
        let other = (algorithm.result.as_ref().unwrap().var, None);
        let target = (id, flow, statement, value, value);
        let guard = algorithm.params[0].ty_guard.unwrap();
        let result_guard = algorithm.result.as_ref().unwrap().ty_guard.unwrap();
        assert!(checked(core, target, witness, &[guard]));
        for target in [
            (
                CoreAlgorithmId::new(core.algorithms().len()),
                flow,
                statement,
                value,
                value,
            ),
            (id, ControlFlowId::new(usize::MAX), statement, value, value),
            (
                id,
                flow,
                CoreAlgorithmStmtId::new(core.algorithm_statements().len()),
                value,
                value,
            ),
            (id, flow, statement, other, value),
            (id, flow, statement, value, other),
            (id, flow, statement, other, other),
            (id, flow, statement, (value.0, Some(statement)), value),
            (id, flow, statement, value, (value.0, Some(statement))),
        ] {
            assert!(
                !checked(core, target, witness, &[guard]),
                "target {target:?}"
            );
        }
        for context in [
            vec![],
            vec![guard, guard],
            vec![result_guard],
            vec![guard, result_guard],
        ] {
            assert!(!checked(core, target, witness, &context));
        }
        assert!(!checked(
            core,
            target,
            CoreTermId::new(core.terms().len()),
            &[guard]
        ));
        let CoreFormulaKind::TypePred { subject, .. } = core.formulas().get(guard).unwrap().kind
        else {
            panic!("typed input")
        };
        let CoreFormulaKind::TypePred {
            subject: wrong_subject,
            ..
        } = core.formulas().get(result_guard).unwrap().kind
        else {
            panic!("other binder")
        };
        for kind in [
            CoreFormulaKind::True,
            CoreFormulaKind::False,
            CoreFormulaKind::TypePred {
                subject,
                ty: "set".into(),
            },
            CoreFormulaKind::TypePred {
                subject: wrong_subject,
                ty: "object".into(),
            },
        ] {
            let mut parts = sole_return_core_parts(core);
            parts.formulas.get_mut(guard).unwrap().kind = kind;
            assert!(!checked(
                &CoreIr::try_new(parts).unwrap(),
                target,
                witness,
                &[guard]
            ));
        }
        // The supplied flow exists but belongs to another algorithm.
        let mut parts = sole_return_core_parts(core);
        let other_id = CoreAlgorithmId::new(parts.algorithms.len());
        let mut other_statement = core.algorithm_statements().get(statement).unwrap().clone();
        other_statement.owner = other_id;
        let other_statement_id = parts.algorithm_statements.insert(other_statement.clone());
        parts
            .source_map
            .algorithm_sources
            .insert(other_statement_id, other_statement.source);
        let mut other_algorithm = algorithm.clone();
        other_algorithm.statements = vec![other_statement_id];
        assert_eq!(parts.algorithms.insert(other_algorithm), other_id);
        let two = CoreIr::try_new(parts).unwrap();
        let other_flow = build_control_flow_ir(&two).flow_map[&other_id];
        assert_ne!(flow, other_flow);
        assert!(checked(&two, target, witness, &[guard]));
        assert!(!checked(
            &two,
            (id, other_flow, statement, value, value),
            witness,
            &[guard]
        ));
        // N1: an unrelated, CoreIr-valid return cycle must refuse before CFG traversal.
        let mut parts = sole_return_core_parts(&two);
        let mut cyclic_term = core.terms().get(witness).unwrap().clone();
        let cyclic_id = CoreTermId::new(parts.terms.len());
        cyclic_term.kind = CoreTermKind::Tuple(vec![cyclic_id]);
        assert_eq!(parts.terms.insert(cyclic_term), cyclic_id);
        parts
            .source_map
            .term_sources
            .insert(cyclic_id, core.terms().get(witness).unwrap().source.clone());
        parts
            .algorithm_statements
            .get_mut(other_statement_id)
            .unwrap()
            .kind = CoreAlgorithmStmtKind::Return(Some(cyclic_id));
        let cyclic = CoreIr::try_new(parts).unwrap();
        assert!(!checked(&cyclic, target, witness, &[guard]));
        let binder = mizar_core::core_ir::CoreVarId::new(usize::MAX);
        assert!(!mizar_kernel::checker::check_algorithm_returns_existential(
            &cyclic,
            (id, flow, statement, value, (binder, None)),
            binder,
            witness,
            &[guard]
        ));
        let mut parts = sole_return_core_parts(core);
        parts.terms.get_mut(witness).unwrap().kind = CoreTermKind::Var(other.0);
        assert!(!checked(
            &CoreIr::try_new(parts).unwrap(),
            target,
            witness,
            &[guard]
        ));
        let mut parts = sole_return_core_parts(core);
        parts.terms.get_mut(witness).unwrap().kind = CoreTermKind::Numeral("0".into());
        assert!(!checked(
            &CoreIr::try_new(parts).unwrap(),
            target,
            witness,
            &[guard]
        ));
    });
}

#[test]
fn source_algorithm_e4_existential_kernel_requires_original_binder_and_rechecked_child() {
    use mizar_core::{
        control_flow::{ControlFlowId, build_control_flow_ir},
        core_ir::{
            CoreAlgorithmId, CoreAlgorithmStmtId, CoreAlgorithmStmtKind, CoreFormulaKind, CoreIr,
            CoreTermId, CoreTermKind, CoreVarId,
        },
    };
    use mizar_kernel::checker::{
        check_algorithm_returns_existential, check_algorithm_returns_existential_result,
    };
    let checked = |core: &CoreIr, target, binder, witness, context: &[_]| {
        let accepted = check_algorithm_returns_existential(core, target, binder, witness, context);
        let result =
            check_algorithm_returns_existential_result(core, target, binder, witness, context);
        assert_eq!(result.is_some(), accepted);
        if let Some(result) = result {
            assert_eq!(result.checker_schema_version(), 1);
            assert!(std::ptr::eq(result.core(), core));
            assert_eq!(result.target(), target);
            assert_eq!(result.witness(), witness);
            assert_eq!(result.context(), context);
        }
        accepted
    };
    with_direct_return_request_fixture(|_, _, _, core, _| {
        let (id, algorithm) = core.algorithms().iter().next().unwrap();
        let statement = algorithm.statements[0];
        let CoreAlgorithmStmtKind::Return(Some(witness)) =
            core.algorithm_statements().get(statement).unwrap().kind
        else {
            panic!("actual return")
        };
        let flow = build_control_flow_ir(core).flow_map[&id];
        let input = (algorithm.params[0].var, None);
        let old_result = (algorithm.result.as_ref().unwrap().var, None);
        let binder = CoreVarId::new(
            core.terms()
                .iter()
                .filter_map(|(_, term)| match term.kind {
                    CoreTermKind::Var(var) => Some(var.index()),
                    _ => None,
                })
                .max()
                .unwrap()
                + 1,
        );
        let result = (binder, None);
        let target = (id, flow, statement, input, result);
        let guard = algorithm.params[0].ty_guard.unwrap();
        let result_guard = algorithm.result.as_ref().unwrap().ty_guard.unwrap();
        assert!(checked(core, target, binder, witness, &[guard]));
        for (target, binder) in [
            (target, CoreVarId::new(binder.index() + 1)),
            ((id, flow, statement, input, input), input.0),
            ((id, flow, statement, input, old_result), binder),
            (
                (id, flow, statement, input, (binder, Some(statement))),
                binder,
            ),
            (
                (id, flow, statement, (input.0, Some(statement)), result),
                binder,
            ),
            ((id, flow, statement, result, input), binder),
            ((id, flow, statement, old_result, result), binder),
            (
                (
                    CoreAlgorithmId::new(core.algorithms().len()),
                    flow,
                    statement,
                    input,
                    result,
                ),
                binder,
            ),
            (
                (id, ControlFlowId::new(usize::MAX), statement, input, result),
                binder,
            ),
            (
                (
                    id,
                    flow,
                    CoreAlgorithmStmtId::new(core.algorithm_statements().len()),
                    input,
                    result,
                ),
                binder,
            ),
        ] {
            assert!(
                !checked(core, target, binder, witness, &[guard]),
                "{target:?}, {binder:?}"
            );
        }
        for context in [
            vec![],
            vec![guard, guard],
            vec![result_guard],
            vec![guard, result_guard],
        ] {
            assert!(!checked(core, target, binder, witness, &context));
        }
        assert!(!checked(
            core,
            target,
            binder,
            CoreTermId::new(core.terms().len()),
            &[guard]
        ));
        for kind in [
            CoreFormulaKind::True,
            CoreFormulaKind::False,
            CoreFormulaKind::TypePred {
                subject: witness,
                ty: "set".into(),
            },
        ] {
            let mut parts = sole_return_core_parts(core);
            parts.formulas.get_mut(guard).unwrap().kind = kind;
            assert!(!checked(
                &CoreIr::try_new(parts).unwrap(),
                target,
                binder,
                witness,
                &[guard]
            ));
        }
        // A guard containing the existential binder cannot be its input premise.
        let mut parts = sole_return_core_parts(core);
        parts.terms.get_mut(witness).unwrap().kind = CoreTermKind::Var(binder);
        assert!(!checked(
            &CoreIr::try_new(parts).unwrap(),
            target,
            binder,
            witness,
            &[guard]
        ));
        // Even a previously successful child check supplies no reusable truth premise.
        assert!(mizar_kernel::checker::check_instantiated_algorithm_returns(
            core,
            (id, flow, statement, input, input),
            witness,
            &[guard]
        ));
        for kind in [
            CoreTermKind::Var(old_result.0),
            CoreTermKind::Numeral("0".into()),
            CoreTermKind::Tuple(vec![]),
        ] {
            let mut parts = sole_return_core_parts(core);
            parts.terms.get_mut(witness).unwrap().kind = kind;
            assert!(!checked(
                &CoreIr::try_new(parts).unwrap(),
                target,
                binder,
                witness,
                &[guard]
            ));
        }
        let mut parts = sole_return_core_parts(core);
        parts.algorithm_statements.get_mut(statement).unwrap().kind =
            CoreAlgorithmStmtKind::Return(None);
        assert!(!checked(
            &CoreIr::try_new(parts).unwrap(),
            target,
            binder,
            witness,
            &[guard]
        ));
    });
}
