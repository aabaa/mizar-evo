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
