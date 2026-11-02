use super::*;

#[test]
fn source_algorithm_syntax_payloads_preserve_exact_declarations() {
    use mizar_resolve::{
        declarations::{DeclarationShellCollector, DeclarationShellKind},
        env::SymbolKind,
        symbols::{
            source_algorithm_syntax_payloads_from_arena as syntax, validate_source_symbol_env,
        },
    };
    let original = include_str!(
        "../../../../../tests/miz/pass/algorithms/pass_proof_verification_algorithm_ensures_return_001.miz"
    );
    // Entire 425-byte vector was frozen from the old parser and written grammar before this API.
    let hex = concat!(
        "6d697a61722d7265736f6c76652f736f757263652d616c676f726974686d2d73796e7461782f76310000970000",
        "000900ab00000001000200650000000b7465726d696e6174696e67000000000002006500000009616c676f7269",
        "74686d0000000000020064000000066964616c676f000000000098000000030002006600000001280000000000",
        "0200640000000161000000000002006600000001290000000000020066000000022d3e00000000001500000001",
        "00190000000100020065000000066f626a6563740000000000ad000000020002006500000007656e7375726573",
        "00000000002c00000001002d00000003001c00000001001d000000010002006400000006726573756c74000000",
        "0000020066000000013d00000000001c00000001001d0000000100020064000000016100000000009900000003",
        "0002006500000002646f00000000009a0000000100a000000003000200650000000672657475726e0000000000",
        "1c00000001001d000000010002006400000001610000000000020066000000013b000000000002006500000003",
        "656e640000000000020066000000013b00000000",
    );
    let expected = hex
        .as_bytes()
        .as_chunks::<2>()
        .0
        .iter()
        .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
        .collect::<Vec<_>>();
    assert_eq!(expected.len(), 425);
    let second = original.replace("idalgo", "otheralgo");
    for (text, count) in [
        (original.to_owned(), 1),
        (format!("{original}\n{second}"), 2),
    ] {
        let (frontend, source, collected) =
            source_registration_owner_inputs(&text, "owner-test", "owners", 0);
        assert!(frontend.diagnostics.is_empty());
        assert!(collected.diagnostics().is_empty());
        validate_source_symbol_env(&source, collected.env()).unwrap();
        let ast = frontend.ast.as_ref().unwrap();
        let shells = DeclarationShellCollector::new(ast, source.module()).collect();
        let algorithms = shells
            .declarations()
            .iter()
            .filter(|s| s.kind() == DeclarationShellKind::AlgorithmDefinition)
            .collect::<Vec<_>>();
        assert_eq!(algorithms.len(), count);
        for shell in algorithms {
            let parent = shells.declaration(shell.parent().unwrap()).unwrap();
            assert_eq!(parent.kind(), DeclarationShellKind::DefinitionBlock);
            assert!(parent.parent().is_none());
        }
        let entries = collected
            .env()
            .symbols()
            .iter()
            .filter(|e| e.kind() == SymbolKind::Algorithm)
            .collect::<Vec<_>>();
        assert_eq!(entries.len(), count);
        let before = (source.clone(), collected.env().clone());
        let values = syntax(&source, collected.env()).unwrap();
        assert_eq!(values.len(), count);
        assert_eq!(
            values.keys().collect::<std::collections::BTreeSet<_>>(),
            entries
                .iter()
                .map(|e| e.symbol())
                .collect::<std::collections::BTreeSet<_>>()
        );
        let original_entry = entries
            .iter()
            .find(|e| e.primary_spelling() == "idalgo")
            .unwrap();
        assert_eq!(values.get(original_entry.symbol()), Some(&expected));
        assert_eq!(syntax(&source, collected.env()).unwrap(), values);
        assert_eq!((source, collected.env().clone()), before);
    }
    for text in ["", "theorem T: for X being set holds X = X;"] {
        let (frontend, source, collected) =
            source_registration_owner_inputs(text, "owner-test", "owners", 0);
        assert!(frontend.diagnostics.is_empty());
        assert!(collected.diagnostics().is_empty());
        validate_source_symbol_env(&source, collected.env()).unwrap();
        assert!(
            !collected
                .env()
                .symbols()
                .iter()
                .any(|e| e.kind() == SymbolKind::Algorithm)
        );
        assert!(syntax(&source, collected.env()).unwrap().is_empty());
    }
}

#[test]
fn source_algorithm_syntax_payloads_preserve_source_associations_and_locality() {
    use mizar_resolve::{
        declarations::{DeclarationShellCollector, DeclarationShellKind},
        env::{SymbolKind, Visibility},
        symbols::{
            source_algorithm_syntax_payloads_from_arena as syntax, validate_source_symbol_env,
        },
    };
    let original = include_str!(
        "../../../../../tests/miz/pass/algorithms/pass_proof_verification_algorithm_ensures_return_001.miz"
    );
    let second = original.replace("idalgo", "otheralgo");
    let text = format!("{original}\n{second}");
    let (_, source, collected) = source_registration_owner_inputs(&text, "owner-test", "owners", 0);
    validate_source_symbol_env(&source, collected.env()).unwrap();
    let prior = syntax(&source, collected.env()).unwrap();
    let baseline = collected
        .env()
        .symbols()
        .iter()
        .filter(|e| e.kind() == SymbolKind::Algorithm)
        .map(|e| (e.primary_spelling().to_owned(), prior[e.symbol()].clone()))
        .collect::<std::collections::BTreeMap<_, _>>();
    assert_eq!(baseline.len(), 2);
    let before = (source.clone(), collected.env().clone(), prior.clone());
    let trivia = format!(":: λ 日本語\n{}", text.replace(' ', "  "));
    let reordered = format!("{second}\n{original}");
    let private = format!(
        "{}\n{second}",
        original.replace("terminating algorithm", "private terminating algorithm")
    );
    let unterminated = format!(
        "{}\n{second}",
        original.replace("terminating algorithm", "algorithm")
    );
    let renamed = format!("{}\n{second}", original.replace("idalgo", "newalgo"));
    let parameter = format!("{}\n{second}", original.replace("idalgo(a)", "idalgo(b)"));
    let returned = format!(
        "{}\n{second}",
        original.replace("return a;", "return result;")
    );
    let outer = format!(
        "{}\n{second}",
        original.replace("let a be object", "let z be object")
    );
    for (index, edited, package, module, skip) in [
        (0, trivia, "owner-test", "owners", 0),
        (1, text.clone(), "owner-test", "owners", 7),
        (2, reordered, "owner-test", "owners", 0),
        (3, text.clone(), "other-package", "owners", 0),
        (4, text.clone(), "owner-test", "other_module", 0),
        (5, private, "owner-test", "owners", 0),
        (6, unterminated, "owner-test", "owners", 0),
        (7, renamed, "owner-test", "owners", 0),
        (8, parameter, "owner-test", "owners", 0),
        (9, returned, "owner-test", "owners", 0),
        (10, outer, "owner-test", "owners", 0),
    ] {
        let (frontend, current, entries) =
            source_registration_owner_inputs(&edited, package, module, skip);
        assert!(frontend.diagnostics.is_empty(), "edit {index}");
        assert!(entries.diagnostics().is_empty(), "edit {index}");
        validate_source_symbol_env(&current, entries.env()).unwrap();
        let shells =
            DeclarationShellCollector::new(frontend.ast.as_ref().unwrap(), current.module())
                .collect();
        assert_eq!(
            shells
                .declarations()
                .iter()
                .filter(|s| s.kind() == DeclarationShellKind::AlgorithmDefinition
                    && shells
                        .declaration(s.parent().unwrap())
                        .is_some_and(|p| p.kind() == DeclarationShellKind::DefinitionBlock
                            && p.parent().is_none()))
                .count(),
            2
        );
        let input_before = (current.clone(), entries.env().clone());
        let values = syntax(&current, entries.env()).unwrap();
        let actual = entries
            .env()
            .symbols()
            .iter()
            .filter(|e| e.kind() == SymbolKind::Algorithm)
            .collect::<Vec<_>>();
        assert_eq!(actual.len(), 2);
        assert_eq!(
            values.keys().collect::<std::collections::BTreeSet<_>>(),
            actual
                .iter()
                .map(|e| e.symbol())
                .collect::<std::collections::BTreeSet<_>>()
        );
        for entry in actual {
            let name = if entry.primary_spelling() == "newalgo" {
                "idalgo"
            } else {
                entry.primary_spelling()
            };
            let value = &values[entry.symbol()];
            if matches!(index, 6..=9) && name == "idalgo" {
                assert_ne!(value, &baseline[name], "edit {index}");
            } else {
                assert_eq!(value, &baseline[name], "edit {index}");
            }
            assert_eq!(entry.symbol().module(), current.module());
            assert_eq!(
                entry.visibility(),
                if index == 5 && name == "idalgo" {
                    Visibility::Private
                } else {
                    Visibility::Public
                }
            );
        }
        if index == 1 {
            assert_ne!(current.source_id(), source.source_id());
        }
        if matches!(index, 2..=4) {
            assert_ne!(
                values.keys().collect::<Vec<_>>(),
                prior.keys().collect::<Vec<_>>()
            );
        }
        assert_eq!((current, entries.env().clone()), input_before);
    }
    assert_eq!((source, collected.env().clone(), prior), before);
}

#[test]
fn source_algorithm_syntax_payloads_preserve_upstream_and_atomic_refusals() {
    use mizar_resolve::{
        env::SymbolKind,
        symbols::{
            source_algorithm_syntax_payloads_from_arena as syntax, validate_source_symbol_env,
        },
    };
    use mizar_syntax::SurfaceNodeKind;
    let original = include_str!(
        "../../../../../tests/miz/pass/algorithms/pass_proof_verification_algorithm_ensures_return_001.miz"
    );
    let (_, source, collected) =
        source_registration_owner_inputs(original, "owner-test", "owners", 0);
    validate_source_symbol_env(&source, collected.env()).unwrap();
    let prior = syntax(&source, collected.env()).unwrap();
    let before = (source.clone(), collected.env().clone(), prior.clone());
    for edited in [
        original.replace("idalgo", "foreignalgo"),
        format!("{original}\n{}", original.replace("idalgo", "otheralgo")),
    ] {
        let (frontend, current, entries) =
            source_registration_owner_inputs(&edited, "owner-test", "owners", 0);
        assert!(frontend.diagnostics.is_empty());
        assert!(entries.diagnostics().is_empty());
        let mismatch = validate_source_symbol_env(&source, entries.env()).unwrap_err();
        assert_eq!(syntax(&source, entries.env()).unwrap_err(), mismatch);
        let mismatch = validate_source_symbol_env(&current, collected.env()).unwrap_err();
        assert_eq!(syntax(&current, collected.env()).unwrap_err(), mismatch);
    }
    let while_source = include_str!(
        "../../../../../tests/miz/pass/algorithms/pass_proof_verification_algorithm_while_invariant_001.miz"
    );
    for edited in [
        while_source.to_owned(),
        format!("{original}\n{while_source}"),
    ] {
        let (frontend, current, entries) =
            source_registration_owner_inputs(&edited, "owner-test", "owners", 0);
        assert!(frontend.diagnostics.is_empty());
        assert!(entries.diagnostics().is_empty());
        assert!(
            frontend
                .ast
                .as_ref()
                .unwrap()
                .node_views()
                .any(|n| matches!(n.kind(), SurfaceNodeKind::WhileStatement))
        );
        assert_eq!(
            entries
                .env()
                .symbols()
                .iter()
                .filter(|e| e.kind() == SymbolKind::Algorithm)
                .count(),
            if edited == while_source { 1 } else { 2 }
        );
        let input_before = (current.clone(), entries.env().clone());
        validate_source_symbol_env(&current, entries.env()).unwrap();
        assert_eq!(
            syntax(&current, entries.env()).unwrap_err(),
            "unsupported source algorithm syntax payload"
        );
        assert_eq!((current, entries.env().clone()), input_before);
    }
    assert_eq!(syntax(&source, collected.env()).unwrap(), prior);
    assert_eq!((source, collected.env().clone(), prior), before);
}

#[test]
fn source_algorithm_state_handoff_projects_real_versions_and_normal_evidence() {
    use mizar_core::core_ir::{CoreAlgorithmStmtKind as S, CoreTermKind as T};
    use mizar_kernel::checker::KernelContextIdentitySource as KernelSource;
    use mizar_kernel::checker::{KernelContextIdentityEntry, KernelContextIdentityPayload};
    use mizar_kernel::{
        certificate_parser::*, checker::*, clause::*, formula_evidence::*, rejection::*,
    };
    use mizar_vc::kernel_evidence_handoff::KernelContextIdentitySource as VcSource;
    use mizar_vc::vc_ir::{
        VcGeneratedFormulaKind as K, VcGeneratedFormulaShape as F, VcProgramValue as V,
    };
    use mizar_vc::{kernel_evidence_handoff::*, vc_ir::*};
    let case = step5c14_state_case();
    let original = std::fs::read_to_string(&case.source_path).unwrap();
    assert_eq!(original.matches("    assert x = a;\n").count(), 1);
    let source = original.replacen("    assert x = a;\n", "", 1);
    let core = step5c14_static_core(&case, &source).unwrap();
    let (_, item) = core.items().iter().next().unwrap();
    let (_, algorithm) = core.algorithms().iter().next().unwrap();
    let [xd, cd, assignment, returned] = algorithm.statements.as_slice() else {
        panic!("actual flat writes")
    };
    let [(x, x_rhs), (c, c_rhs)] =
        [xd, cd].map(
            |id| match &core.algorithm_statements().get(*id).unwrap().kind {
                S::Let {
                    binder,
                    value: Some(rhs),
                    ghost: false,
                } => (binder.var, *rhs),
                _ => panic!(),
            },
        );
    let assigned_rhs = match core.algorithm_statements().get(*assignment).unwrap().kind {
        S::AssignLocal { value, .. } => value,
        _ => panic!(),
    };
    let S::Return(Some(return_rhs)) = core.algorithm_statements().get(*returned).unwrap().kind
    else {
        panic!()
    };
    for (term, var) in [
        (x_rhs, algorithm.params[0].var),
        (c_rhs, algorithm.params[0].var),
        (assigned_rhs, c),
        (return_rhs, x),
    ] {
        assert_eq!(core.terms().get(term).unwrap().kind, T::Var(var));
    }
    let value = |var, definition| V { var, definition };
    let values = [
        value(algorithm.params[0].var, None),
        value(x, Some(*xd)),
        value(c, Some(*cd)),
        value(x, Some(*assignment)),
    ];
    assert_ne!(values[1], values[3]);
    let vcs = step5c14_state_vcs(&core).unwrap();
    let [vc] = vcs.vcs() else {
        panic!("one return VC")
    };
    assert_eq!(
        (vc.kind.clone(), vc.status.clone()),
        (VcKind::AlgorithmPostcondition, VcStatus::Open)
    );
    assert!(!vc.anchor.is_complete());
    let [termination, contract] = vcs.seed_accounting() else {
        panic!()
    };
    assert!(matches!(
        termination.mapping,
        SeedVcMapping::NoConcreteVc {
            reason: SeedNoVcReason::DeferredExternal(_)
        }
    ));
    assert!(matches!(contract.mapping, SeedVcMapping::One { vc: id } if id == vc.id));
    let pairs = [
        (values[1], values[0]),
        (values[2], values[0]),
        (values[3], values[2]),
        (values[3], values[0]),
    ];
    assert_eq!(vcs.generated_formulas().len(), 7);
    for (index, generated) in vcs.generated_formulas().iter().enumerate() {
        let shape = if index == 6 || index % 2 == 0 {
            let (left, right) = pairs[if index == 6 { 3 } else { index / 2 }];
            F::ProgramEquals { left, right }
        } else {
            F::ProgramTypePredicate {
                subject: values[index / 2 + 1],
                ty: mizar_core::core_ir::CoreTypePredicate::new("object"),
            }
        };
        assert_eq!(generated.shape, shape);
        assert_eq!(
            generated.kind,
            if index == 6 {
                K::AlgorithmPostcondition
            } else {
                K::AlgorithmStateFact
            }
        );
        if index < 6 {
            let entry = &vc.local_context.entries()[index + 1];
            assert_eq!(entry.kind, ContextEntryKind::GeneratedFact);
            assert_eq!(entry.formula, Some(VcFormulaRef::Generated(generated.id)));
            assert_eq!(entry.provenance, generated.provenance);
        }
    }
    assert_eq!(
        vc.local_context.entries()[0].formula,
        Some(VcFormulaRef::Core(algorithm.params[0].ty_guard.unwrap()))
    );
    assert_eq!(
        vc.goal,
        VcFormulaRef::Generated(vcs.generated_formulas()[6].id)
    );
    let premises = vc
        .local_context
        .entries()
        .iter()
        .map(|row| PremiseRef::LocalContext(row.id))
        .collect::<Vec<_>>();
    assert_eq!(vc.premises, premises);
    assert_eq!(vc.local_context.policy_inputs().len(), 6);
    let generation = GenerationSchemaVersion::new("mizar-vc-generation-step5c14-state-v1");
    let before = (core.clone(), vcs.clone());
    let handoff =
        build_source_algorithm_state_kernel_handoff(&core, &vcs, vc.id, &generation).unwrap();
    assert_eq!((core.clone(), vcs.clone()), before);
    let bytes = handoff.encode_local_evidence(None).unwrap();
    let hex_bytes = |text: &str| -> Vec<u8> {
        text.as_bytes()
            .as_chunks::<2>()
            .0
            .iter()
            .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
            .collect()
    };
    let wires = "
        01040000000100000001000000010100000000
        010300000000000000020000000201000000010100000000
        01040000000100000001000000010100000001
        010300000000000000020000000201000000020100000000
        01040000000100000001000000010100000002
        010300000000000000020000000201000000030100000002
        01040000000100000001000000010100000003
        010300000000000000020000000201000000030100000000";
    let fingerprints = "
        4d495a41525f4b45524e454c5f464f524d554c41000b010000001d0204000000170400000001000000010000000101010000000400000000
        4d495a41525f4b45524e454c5f464f524d554c41000b0100000027020300000021030000000000000002000000020101000000040000000101010000000400000000
        4d495a41525f4b45524e454c5f464f524d554c41000b010000001d0204000000170400000001000000010000000101010000000400000001
        4d495a41525f4b45524e454c5f464f524d554c41000b0100000027020300000021030000000000000002000000020101000000040000000201010000000400000000
        4d495a41525f4b45524e454c5f464f524d554c41000b010000001d0204000000170400000001000000010000000101010000000400000002
        4d495a41525f4b45524e454c5f464f524d554c41000b0100000027020300000021030000000000000002000000020101000000040000000301010000000400000002
        4d495a41525f4b45524e454c5f464f524d554c41000b010000001d0204000000170400000001000000010000000101010000000400000003
        4d495a41525f4b45524e454c5f464f524d554c41000b0100000027020300000021030000000000000002000000020101000000040000000301010000000400000000";
    let literal = wires
        .split_whitespace()
        .zip(fingerprints.split_whitespace());
    let projected = handoff.canonical_evidence();
    assert_eq!(
        projected
            .symbol_manifest()
            .iter()
            .map(|row| (row.entry_id, row.payload.as_slice()))
            .collect::<Vec<_>>(),
        [(0, &[3, 0, 0, 0, 0][..]), (1, &[4, 0, 0, 0, 1][..])]
    );
    let variables = projected.variable_manifest();
    assert_eq!(variables.len(), 4);
    for (index, row) in variables.iter().enumerate() {
        assert_eq!(row.entry_id, index as u32);
        assert_eq!(row.payload, (index as u32).to_be_bytes());
    }
    let provenance = format!("source-algorithm-state-local-handoff-v1;source={:?};module={:?};item={:?};algorithm={:?};statements={:?};accounting={:?}",
        vcs.source(), core.module_id(), item, algorithm,
        algorithm.statements.iter().map(|id| core.algorithm_statements().get(*id).unwrap()).collect::<Vec<_>>(), vcs.seed_accounting()).into_bytes();
    let references = vc
        .local_context
        .entries()
        .iter()
        .map(|row| row.formula.unwrap())
        .chain(std::iter::once(vc.goal))
        .collect::<Vec<_>>();
    let coords: [&[u32]; 8] = [&[0], &[1, 0], &[1], &[2, 0], &[2], &[3, 2], &[3], &[3, 0]];
    let manual = references
        .iter()
        .zip(literal)
        .zip(coords)
        .map(|((reference, (wire, digest)), coords)| {
            let kind = if coords.len() == 2 { 3 } else { 4 };
            let pure = Formula::Atom(Atom::new(
                if kind == 3 {
                    SymbolKey::new(SymbolKind::Equality, 0)
                } else {
                    SymbolKey::new(SymbolKind::BuiltinRelation, 1)
                },
                coords
                    .iter()
                    .copied()
                    .map(|i| Term::Variable(VariableId(i)))
                    .collect(),
            ))
            .canonical_hash_input()
            .unwrap();
            assert_eq!(pure, hex_bytes(digest));
            KernelFormulaPayload {
                formula_ref: *reference,
                projection: KernelFormulaProjection {
                    formula_fingerprint: KernelEvidenceFingerprint {
                        algorithm_id: 2,
                        digest: pure,
                    },
                    formula_bytes: hex_bytes(wire),
                    provenance_payload: provenance.clone(),
                },
            }
        })
        .collect::<Vec<_>>();
    let expected = build_kernel_evidence_handoff(KernelEvidenceHandoffInput {
        vc_set: &vcs,
        vc: vc.id,
        goal_polarity: KernelGoalPolarity::AssertFalseForRefutation,
        kernel_profile: KernelEvidenceProfile::v1(1, KernelClauseTautologyPolicy::Reject),
        symbol_manifest: projected.symbol_manifest(),
        variable_manifest: projected.variable_manifest(),
        formula_payloads: &manual,
        imported_formula_payloads: &[],
        substitutions: &[],
        formula_context: None,
        discharge_output: None,
    })
    .unwrap();
    assert_eq!(handoff, expected);
    assert!(handoff.targets_vc(&vcs, vc.id).unwrap());
    let fingerprint = projected.target_vc();
    let target = TargetVcFingerprint::new(fingerprint.algorithm_id, fingerprint.digest.clone());
    let parsed = parse_formula_evidence(
        &bytes,
        &FormulaEvidenceParseContext::v1(
            Fingerprint::new(fingerprint.algorithm_id, fingerprint.digest.clone()),
            KernelProfileRecord::v1(1, ClauseTautologyPolicy::Reject),
        ),
    )
    .unwrap();
    assert!(parsed.equality_instances().is_empty());
    let entries = handoff
        .context_identity()
        .entries()
        .iter()
        .map(|row| {
            let VcSource::LocalHypothesis { local_context_id } = row.source() else {
                panic!()
            };
            let source = KernelSource::LocalHypothesis { local_context_id };
            let producer = match row.producer_formula_ref() {
                VcFormulaRef::Core(id) => KernelFormulaProducerRef::Core(id),
                VcFormulaRef::Generated(id) => {
                    KernelFormulaProducerRef::Generated(KernelVcGeneratedFormulaId::new(id.index()))
                }
                _ => panic!(),
            };
            let fingerprint = row.formula_fingerprint();
            KernelContextIdentityEntry::new(
                source,
                row.formula_id(),
                Fingerprint::new(fingerprint.algorithm_id, fingerprint.digest.clone()),
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
    let checked = check_kernel_evidence(KernelEvidenceCheckInput {
        target_vc_fingerprint: &target,
        evidence: &parsed,
        formula_context: Some(&context),
        check_kind: KernelEvidenceCheckKind::ProofObligation,
        policy: KernelCheckPolicy::default(),
        limits: KernelEvidenceCheckLimits::default(),
    });
    assert_eq!(checked.status(), KernelCheckStatus::Rejected);
    assert_eq!(
        checked.rejections()[0].detail(),
        RejectionDetail::InvalidSatRefutation
    );
    assert!(!checked.policy_taint());
}

#[test]
fn source_algorithm_state_handoff_preserves_distinct_values_and_contexts() {
    use mizar_core::core_ir::{CoreAlgorithmStmtKind as S, CoreTypePredicate};
    use mizar_vc::{
        generator::generate_source_algorithm_postconditions,
        kernel_evidence_handoff::build_source_algorithm_state_kernel_handoff,
        vc_ir::{
            GenerationSchemaVersion, VcFormulaRef, VcGeneratedFormulaShape as F,
            VcProgramValue as V, VcSchemaVersion,
        },
    };
    let case = step5c14_state_case();
    let source = std::fs::read_to_string(&case.source_path)
        .unwrap()
        .replacen("    assert x = a;\n", "", 1)
        .replace("    x := c;", "    const d := a;\n    x := c;");
    let texts = [
        source.clone(),
        source.replace("swapalgo", "copyalgo"),
        format!(":: neutral comment\n\n{source}"),
        source.replace("ensures result = a", "ensures a = result"),
        source.replace("ensures result = a", "ensures a = a"),
        source.replace("ensures result = a", "ensures result = result"),
    ];
    for (case_index, snapshot, changed_generation, changed_schema) in [
        (0, 0, false, false),
        (1, 0, false, false),
        (2, 0, false, false),
        (0, 1, false, false),
        (0, 0, true, false),
        (0, 0, false, true),
        (3, 0, false, false),
        (4, 0, false, false),
        (5, 0, false, false),
    ] {
        let core = step5c14_static_core(&case, &texts[case_index]).unwrap();
        let generation = GenerationSchemaVersion::new(if changed_generation {
            "mizar-vc-generation-step5c14-state-v1-changed"
        } else {
            "mizar-vc-generation-step5c14-state-v1"
        });
        let schema = VcSchemaVersion::new(if changed_schema {
            "mizar-vc-vcset-step5c14-state-v1-changed"
        } else {
            "mizar-vc-vcset-step5c14-state-v1"
        });
        let vcs = generate_source_algorithm_postconditions(
            &core,
            super::super::shared::snapshot_id(snapshot),
            &generation,
            &schema,
        )
        .unwrap();
        let [vc] = vcs.vcs() else { panic!() };
        let before = (core.clone(), vcs.clone());
        let (_, algorithm) = core.algorithms().iter().next().unwrap();
        let [xd, cd, dd, assignment, _] = algorithm.statements.as_slice() else {
            panic!()
        };
        let vars =
            [xd, cd, dd].map(
                |id| match &core.algorithm_statements().get(*id).unwrap().kind {
                    S::Let {
                        binder,
                        value: Some(_),
                        ghost: false,
                    } => binder.var,
                    _ => panic!(),
                },
            );
        let v = |var, definition| V { var, definition };
        let values = [
            v(algorithm.params[0].var, None),
            v(vars[0], Some(*xd)),
            v(vars[1], Some(*cd)),
            v(vars[2], Some(*dd)),
            v(vars[0], Some(*assignment)),
        ];
        if case_index >= 3 {
            let (left, right) = [
                (values[0], values[4]),
                (values[0], values[0]),
                (values[4], values[4]),
            ][case_index - 3];
            let goal = &vcs.generated_formulas()[8];
            assert_eq!(goal.shape, F::ProgramEquals { left, right });
            assert_eq!(vc.goal, VcFormulaRef::Generated(goal.id));
        }
        if case_index == 0 && snapshot == 0 && !changed_generation && !changed_schema {
            for (index, value) in values.iter().enumerate() {
                assert!(!values[..index].contains(value));
            }
            let facts = vcs.generated_formulas();
            assert_eq!((facts.len(), vc.local_context.entries().len()), (9, 9));
            for (index, left, right) in [
                (4, values[3], values[0]),
                (6, values[4], values[2]),
                (8, values[4], values[0]),
            ] {
                assert_eq!(facts[index].shape, F::ProgramEquals { left, right });
            }
            for (index, subject) in [(5, values[3]), (7, values[4])] {
                assert_eq!(
                    facts[index].shape,
                    F::ProgramTypePredicate {
                        subject,
                        ty: CoreTypePredicate::new("object")
                    }
                );
            }
            for index in [4, 6, 7] {
                let entry = &vc.local_context.entries()[index + 1];
                assert_eq!(
                    entry.formula,
                    Some(VcFormulaRef::Generated(facts[index].id))
                );
                assert_eq!(entry.provenance, facts[index].provenance);
            }
        }
        let first =
            build_source_algorithm_state_kernel_handoff(&core, &vcs, vc.id, &generation).unwrap();
        assert_eq!(
            first,
            build_source_algorithm_state_kernel_handoff(&core, &vcs, vc.id, &generation).unwrap()
        );
        assert_eq!((core.clone(), vcs.clone()), before);
        let bytes = |hex: &str| {
            hex.as_bytes()
                .as_chunks::<2>()
                .0
                .iter()
                .map(|p| u8::from_str_radix(std::str::from_utf8(p).unwrap(), 16).unwrap())
                .collect::<Vec<_>>()
        };
        if case_index >= 3 {
            let wires = "010300000000000000020000000201000000000100000004 010300000000000000020000000201000000000100000000 010300000000000000020000000201000000040100000004";
            let hashes = "4d495a41525f4b45524e454c5f464f524d554c41000b0100000027020300000021030000000000000002000000020101000000040000000001010000000400000004 4d495a41525f4b45524e454c5f464f524d554c41000b0100000027020300000021030000000000000002000000020101000000040000000001010000000400000000 4d495a41525f4b45524e454c5f464f524d554c41000b0100000027020300000021030000000000000002000000020101000000040000000401010000000400000004";
            let goal = first.canonical_evidence().final_goal();
            let index = case_index - 3;
            assert_eq!(goal.producer_formula_ref, vc.goal);
            assert_eq!(
                goal.formula_bytes,
                bytes(wires.split_whitespace().nth(index).unwrap())
            );
            assert_eq!(goal.formula_fingerprint.algorithm_id, 2);
            assert_eq!(
                goal.formula_fingerprint.digest,
                bytes(hashes.split_whitespace().nth(index).unwrap())
            );
        }
        if case_index == 0 && snapshot == 0 && !changed_generation && !changed_schema {
            let envelope = first.canonical_evidence();
            assert_eq!(envelope.variable_manifest().len(), 5);
            assert_eq!(envelope.variable_manifest()[4].payload, [0, 0, 0, 4]);
            assert_eq!(first.context_identity().entries().len(), 9);
            let wires = "010300000000000000020000000201000000040100000002 01040000000100000001000000010100000004 010300000000000000020000000201000000040100000000";
            let hashes = "4d495a41525f4b45524e454c5f464f524d554c41000b0100000027020300000021030000000000000002000000020101000000040000000401010000000400000002 4d495a41525f4b45524e454c5f464f524d554c41000b010000001d0204000000170400000001000000010000000101010000000400000004 4d495a41525f4b45524e454c5f464f524d554c41000b0100000027020300000021030000000000000002000000020101000000040000000401010000000400000000";
            for (id, (wire, hash)) in
                (6..9).zip(wires.split_whitespace().zip(hashes.split_whitespace()))
            {
                let reference = VcFormulaRef::Generated(vcs.generated_formulas()[id].id);
                let (actual_bytes, fingerprint) = if id == 8 {
                    let goal = envelope.final_goal();
                    assert_eq!(goal.producer_formula_ref, reference);
                    (goal.formula_bytes.as_slice(), &goal.formula_fingerprint)
                } else {
                    let row = envelope
                        .formula_evidence()
                        .iter()
                        .find(|r| r.producer_formula_ref() == Some(reference))
                        .unwrap();
                    (row.formula_bytes(), row.formula_fingerprint())
                };
                assert_eq!(actual_bytes, bytes(wire));
                assert_eq!(fingerprint.algorithm_id, 2);
                assert_eq!(fingerprint.digest, bytes(hash));
            }
        }
    }
}

#[test]
fn source_algorithm_state_handoff_preserves_refusals_and_upstream_order() {
    use mizar_vc::{
        generator::generate_source_algorithm_postconditions,
        kernel_evidence_handoff::build_source_algorithm_state_kernel_handoff,
        vc_ir::{GenerationSchemaVersion, VcId, VcKind, VcSet, VcSetParts, VcStatus},
    };
    let case = step5c14_state_case();
    let original = std::fs::read_to_string(&case.source_path).unwrap();
    let source = original.replacen("    assert x = a;\n", "", 1);
    let core = step5c14_static_core(&case, &source).unwrap();
    let generation = GenerationSchemaVersion::new("mizar-vc-generation-step5c14-state-v1");
    let vcs = step5c14_state_vcs(&core).unwrap();
    let [vc] = vcs.vcs() else { panic!() };
    let invalid = "algorithm.state.handoff.invalid_projection";
    let wrong_generation = GenerationSchemaVersion::new("different");
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
            entries[1].sort_key = "algorithm-state-00000000a".into();
            entries
        },
        parts.vcs[0].local_context.policy_inputs().to_vec(),
    )
    .unwrap();
    let drift = VcSet::try_new(parts).unwrap();
    let original_core = step5c14_static_core(&case, &original).unwrap();
    let original_vcs = step5c14_state_vcs(&original_core).unwrap();
    assert_eq!(original_vcs.vcs().len(), 2);
    let all_open = original_vcs
        .vcs()
        .iter()
        .all(|vc| vc.status == VcStatus::Open);
    assert!(all_open);
    let return_vc = original_vcs
        .vcs()
        .iter()
        .find(|vc| vc.kind == VcKind::AlgorithmPostcondition)
        .unwrap();
    assert!(return_vc.local_context.entries().iter().any(|row| matches!(
        row.kind,
        mizar_vc::vc_ir::ContextEntryKind::PendingAlgorithmAssertion { .. }
    )));
    let foreign =
        step5c14_static_core(&case, &source.replace("swapalgo", "foreign_algorithm")).unwrap();
    let foreign_vcs = step5c14_state_vcs(&foreign).unwrap();
    assert_ne!(foreign_vcs, vcs);
    let zero_core =
        step5c14_static_core(&case, &source.replace("    ensures result = a\n", "")).unwrap();
    let zero_vcs = step5c14_state_vcs(&zero_core).unwrap();
    assert!(zero_vcs.vcs().is_empty());
    let broader = step5c14_static_core(
        &case,
        &source.replace("    x := c;", "    assert x = a;\n    x := c;"),
    )
    .unwrap();
    let broader_vcs = step5c14_state_vcs(&broader).unwrap();
    assert!(broader_vcs.vcs().len() > 1);
    let broad_return = broader_vcs
        .vcs()
        .iter()
        .find(|vc| vc.kind == VcKind::AlgorithmPostcondition)
        .unwrap();
    let two_returns = source
        .replace("    ensures result = a\n", "")
        .replace("    return x;", "    return x;\n    return x;");
    let two_core = step5c14_static_core(&case, &two_returns).unwrap();
    let upstream = generate_source_algorithm_postconditions(
        &two_core,
        vcs.snapshot(),
        &generation,
        vcs.schema_version(),
    )
    .unwrap_err();
    assert_eq!(upstream, "algorithms.postcondition.unsupported_core");
    for (index, (checked_core, checked_set, target, checked_generation)) in [
        (&core, &vcs, VcId::new(99), &generation),
        (&core, &vcs, vc.id, &wrong_generation),
        (&core, &drift, vc.id, &generation),
        (&original_core, &original_vcs, return_vc.id, &generation),
        (&foreign, &vcs, vc.id, &generation),
        (&zero_core, &zero_vcs, VcId::new(0), &generation),
        (&broader, &broader_vcs, broad_return.id, &generation),
        (&two_core, &vcs, VcId::new(99), &generation),
    ]
    .into_iter()
    .enumerate()
    {
        let error = build_source_algorithm_state_kernel_handoff(
            checked_core,
            checked_set,
            target,
            checked_generation,
        )
        .unwrap_err();
        assert_eq!(
            error,
            if index == 7 {
                upstream.as_str()
            } else {
                invalid
            }
        );
    }
}

#[test]
fn source_algorithm_state_transitivity_names_reach_normal_kernel() {
    use mizar_atp::backend::encode_local_variable_transitivity_names;
    use mizar_core::core_ir::CoreAlgorithmStmtKind as S;
    use mizar_kernel::checker::KernelContextIdentitySource as KernelSource;
    use mizar_kernel::{
        certificate_parser::*, checker::*, clause::*, formula_evidence::*, rejection::*,
    };
    use mizar_vc::kernel_evidence_handoff::KernelContextIdentitySource as VcSource;
    use mizar_vc::{kernel_evidence_handoff::*, vc_ir::*};
    let case = step5c14_state_case();
    let original = std::fs::read_to_string(&case.source_path).unwrap();
    assert_eq!(original.matches("    assert x = a;\n").count(), 1);
    let base = original.replacen("    assert x = a;\n", "", 1);
    for (index, source) in [
        base.clone(),
        base.replace("swapalgo", "copyalgo"),
        base.replace("    x := c;", "    const d := a;\n    x := c;"),
    ]
    .into_iter()
    .enumerate()
    {
        let terminal = if index == 2 { 4 } else { 3 };
        let literal = if index == 2 {
            "000000010300000000010000000401000000020100000000"
        } else {
            "000000010300000000010000000301000000020100000000"
        };
        let core = step5c14_static_core(&case, &source).unwrap();
        let (_, algorithm) = core.algorithms().iter().next().unwrap();
        let statements = &algorithm.statements;
        let (S::Let { binder: x, .. }, S::Let { binder: c, .. }) = (
            &core.algorithm_statements().get(statements[0]).unwrap().kind,
            &core.algorithm_statements().get(statements[1]).unwrap().kind,
        ) else {
            panic!("actual checked declarations")
        };
        let assignment = statements[statements.len() - 2];
        assert!(
            matches!(core.algorithm_statements().get(assignment).unwrap().kind,
            S::AssignLocal { target, .. } if target == x.var)
        );
        let value = |var, definition| VcProgramValue { var, definition };
        let a = value(algorithm.params[0].var, None);
        let c0 = value(c.var, Some(statements[1]));
        let x1 = value(x.var, Some(assignment));
        let vcs = step5c14_state_vcs(&core).unwrap();
        for (left, right) in [(c0, a), (x1, c0)] {
            assert!(vcs.generated_formulas().iter().any(|formula| formula.kind
                == VcGeneratedFormulaKind::AlgorithmStateFact
                && formula.shape == VcGeneratedFormulaShape::ProgramEquals { left, right }));
        }
        let [vc] = vcs.vcs() else {
            panic!("one actual return VC")
        };
        assert_eq!(
            (vc.kind.clone(), vc.status.clone()),
            (VcKind::AlgorithmPostcondition, VcStatus::Open)
        );
        assert!(!vc.anchor.is_complete());
        assert!(matches!(
            vcs.seed_accounting()[0].mapping,
            SeedVcMapping::NoConcreteVc {
                reason: SeedNoVcReason::DeferredExternal(_)
            }
        ));
        let VcFormulaRef::Generated(goal_id) = vc.goal else {
            panic!()
        };
        assert_eq!(
            vcs.generated_formulas()[goal_id.index()].shape,
            VcGeneratedFormulaShape::ProgramEquals { left: x1, right: a }
        );
        let before = (core.clone(), vcs.clone());
        let generation = GenerationSchemaVersion::new("mizar-vc-generation-step5c14-state-v1");
        let handoff =
            build_source_algorithm_state_kernel_handoff(&core, &vcs, vc.id, &generation).unwrap();
        let fingerprint = handoff.canonical_evidence().target_vc();
        let target = TargetVcFingerprint::new(fingerprint.algorithm_id, fingerprint.digest.clone());
        let mut parse_context = FormulaEvidenceParseContext::v1(
            Fingerprint::new(fingerprint.algorithm_id, fingerprint.digest.clone()),
            KernelProfileRecord::v1(1, ClauseTautologyPolicy::Reject),
        );
        let v1 = handoff.encode_local_evidence(None).unwrap();
        let parsed = parse_formula_evidence(&v1, &parse_context).unwrap();
        let Formula::Atom(goal) = &parsed.final_goal().formula else {
            panic!()
        };
        assert_eq!(goal.symbol, SymbolKey::new(SymbolKind::Equality, 0));
        assert_eq!(
            goal.arguments,
            [
                Term::Variable(VariableId(terminal)),
                Term::Variable(VariableId(0))
            ]
        );
        for (left, right) in [(terminal, 2), (2, 0)] {
            assert!(parsed.formulas().iter().any(|row| matches!(&row.formula,
                Formula::Atom(atom) if atom.symbol == goal.symbol &&
                atom.arguments == [Term::Variable(VariableId(left)), Term::Variable(VariableId(right))])));
        }
        let pristine = parsed.clone();
        let names = encode_local_variable_transitivity_names(&parsed, VariableId(2)).unwrap();
        assert_eq!(
            names,
            [literal
                .as_bytes()
                .as_chunks::<2>()
                .0
                .iter()
                .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
                .collect::<Vec<_>>()]
        );
        assert_eq!(
            encode_local_variable_transitivity_names(&parsed, VariableId(2)).unwrap(),
            names
        );
        assert_eq!(parsed, pristine);
        let context_for = |source: &VcKernelEvidenceHandoff| {
            let fp = source.canonical_evidence().target_vc();
            let target = TargetVcFingerprint::new(fp.algorithm_id, fp.digest.clone());
            let entries = source
                .context_identity()
                .entries()
                .iter()
                .map(|row| {
                    let VcSource::LocalHypothesis { local_context_id } = row.source() else {
                        panic!()
                    };
                    let producer = match row.producer_formula_ref() {
                        VcFormulaRef::Core(id) => KernelFormulaProducerRef::Core(id),
                        VcFormulaRef::Generated(id) => KernelFormulaProducerRef::Generated(
                            KernelVcGeneratedFormulaId::new(id.index()),
                        ),
                        _ => panic!(),
                    };
                    let fp = row.formula_fingerprint();
                    mizar_kernel::checker::KernelContextIdentityEntry::new(
                        KernelSource::LocalHypothesis { local_context_id },
                        row.formula_id(),
                        Fingerprint::new(fp.algorithm_id, fp.digest.clone()),
                        producer,
                    )
                })
                .collect();
            FormulaEvidenceContext::with_context_identity(
                Some(source.context_identity_hash().as_bytes().to_vec()),
                vec![],
                vec![],
                Some(mizar_kernel::checker::KernelContextIdentityPayload::new(
                    target,
                    source.canonical_hash(),
                    source.context_identity_hash(),
                    entries,
                )),
                ImportedFactContextLimits::default(),
            )
            .unwrap()
        };
        let context = context_for(&handoff);
        let check = |evidence: &ParsedKernelEvidence,
                     target: &TargetVcFingerprint,
                     context: &FormulaEvidenceContext| {
            check_kernel_evidence(KernelEvidenceCheckInput {
                target_vc_fingerprint: target,
                evidence,
                formula_context: Some(context),
                check_kind: KernelEvidenceCheckKind::ProofObligation,
                policy: KernelCheckPolicy::default(),
                limits: KernelEvidenceCheckLimits::default(),
            })
        };
        let no_name = check(&parsed, &target, &context);
        assert_eq!(no_name.status(), KernelCheckStatus::Rejected);
        parse_context.accepted_schema_versions.insert(2);
        let wire = handoff.encode_local_evidence(Some(&names)).unwrap();
        let named = parse_formula_evidence(&wire, &parse_context).unwrap();
        let checked = check(&named, &target, &context);
        assert_eq!(checked.status(), KernelCheckStatus::Accepted);
        assert_eq!(
            checked.evidence_check_kind(),
            Some(KernelEvidenceCheckKind::ProofObligation)
        );
        assert!(!checked.policy_taint());
        assert!(checked.used_axioms().is_empty());
        let mut wrong = names[0].clone();
        wrong[18] = 1; // An existing but non-chain middle, still a coherent name.
        let wrong_wire = handoff.encode_local_evidence(Some(&[wrong])).unwrap();
        let wrong_parsed = parse_formula_evidence(&wrong_wire, &parse_context).unwrap();
        let rejected = check(&wrong_parsed, &target, &context);
        assert_eq!(rejected.status(), KernelCheckStatus::Rejected);
        if index == 0 {
            let foreign_core =
                step5c14_static_core(&case, &source.replace("return x;", "return c;")).unwrap();
            let foreign_vcs = step5c14_state_vcs(&foreign_core).unwrap();
            let [foreign_vc] = foreign_vcs.vcs() else {
                panic!()
            };
            let foreign = build_source_algorithm_state_kernel_handoff(
                &foreign_core,
                &foreign_vcs,
                foreign_vc.id,
                &generation,
            )
            .unwrap();
            let fp = foreign.canonical_evidence().target_vc();
            let foreign_target = TargetVcFingerprint::new(fp.algorithm_id, fp.digest.clone());
            assert_ne!(foreign_target, target);
            assert_eq!(
                check(&named, &foreign_target, &context).status(),
                KernelCheckStatus::Rejected
            );
            let foreign_context = context_for(&foreign);
            assert_eq!(
                check(&named, &target, &foreign_context).status(),
                KernelCheckStatus::Rejected
            );
        }
        assert_eq!((core, vcs), before);
        assert_eq!(handoff.encode_local_evidence(None).unwrap(), v1);
    }
}

#[test]
fn source_algorithm_state_transitivity_names_reject_untrusted_profiles() {
    use mizar_atp::backend::encode_local_variable_transitivity_names;
    use mizar_kernel::{
        certificate_parser::{ClauseTautologyPolicy, Fingerprint, KernelProfileRecord},
        clause::{Atom, SymbolKey, SymbolKind, Term, VariableId},
        formula_evidence::{
            Formula, FormulaEvidenceParseContext, GoalPolarity, parse_formula_evidence,
        },
    };
    use mizar_vc::kernel_evidence_handoff::*;
    let case = step5c14_state_case();
    let source = std::fs::read_to_string(&case.source_path)
        .unwrap()
        .replacen("    assert x = a;\n", "", 1);
    let core = step5c14_static_core(&case, &source).unwrap();
    let vcs = step5c14_state_vcs(&core).unwrap();
    let [vc] = vcs.vcs() else { panic!() };
    let eq = SymbolKey::new(SymbolKind::Equality, 7);
    let v = |id| Term::Variable(VariableId(id));
    // Raw generic projections here are untrusted input data, with no source acceptance claim.
    let atom = |symbol: SymbolKey, arguments: Vec<Term>| {
        let kind = match symbol.kind {
            SymbolKind::Equality => 3,
            SymbolKind::Predicate => 1,
            _ => panic!(),
        };
        let mut wire = vec![1, kind];
        wire.extend(symbol.id.0.to_be_bytes());
        wire.extend((arguments.len() as u32).to_be_bytes());
        wire.extend((arguments.len() as u32).to_be_bytes());
        for term in &arguments {
            match term {
                Term::Variable(id) => {
                    wire.push(1);
                    wire.extend(id.0.to_be_bytes());
                }
                Term::Application { symbol, arguments } if arguments.is_empty() => {
                    wire.push(2);
                    wire.push(2);
                    wire.extend(symbol.id.0.to_be_bytes());
                    wire.extend(0u32.to_be_bytes());
                }
                _ => panic!(),
            }
        }
        (Formula::Atom(Atom::new(symbol, arguments)), wire)
    };
    for profile in 0..16 {
        let mut first = atom(eq, vec![v(11), v(12)]);
        let mut second = atom(eq, vec![v(12), v(13)]);
        let mut goal = atom(eq, vec![v(11), v(13)]);
        let mut middle = VariableId(12);
        match profile {
            1 => first = atom(eq, vec![v(11), v(12), v(14)]),
            2 => second = atom(eq, vec![v(12), v(13), v(14)]),
            3 => first = atom(eq, vec![v(12), v(11)]),
            4 => second = atom(eq, vec![v(13), v(12)]),
            5 => first = atom(SymbolKey::new(SymbolKind::Equality, 10), vec![v(11), v(12)]),
            6 => second = atom(SymbolKey::new(SymbolKind::Equality, 10), vec![v(12), v(13)]),
            7 => middle = VariableId(11),
            8 => middle = VariableId(15),
            9 => goal = atom(SymbolKey::new(SymbolKind::Predicate, 8), vec![v(11), v(13)]),
            10 => {
                goal.0 = Formula::Not(Box::new(goal.0));
                goal.1.insert(0, 2);
            }
            11 => {
                goal = atom(
                    eq,
                    vec![
                        v(11),
                        Term::Application {
                            symbol: SymbolKey::new(SymbolKind::FunctorPredicate, 9),
                            arguments: vec![],
                        },
                    ],
                )
            }
            12 => goal = atom(eq, vec![v(11), v(13), v(14)]),
            _ => {}
        }
        let unrelated = atom(SymbolKey::new(SymbolKind::Predicate, 8), vec![v(14)]);
        let formulas = vc
            .local_context
            .entries()
            .iter()
            .enumerate()
            .map(|(index, row)| {
                let (formula, wire) = match index {
                    0 => &first,
                    1 => &second,
                    _ => &unrelated,
                };
                KernelFormulaPayload {
                    formula_ref: row.formula.unwrap(),
                    projection: KernelFormulaProjection {
                        formula_fingerprint: KernelEvidenceFingerprint {
                            algorithm_id: 2,
                            digest: formula.canonical_hash_input().unwrap(),
                        },
                        formula_bytes: wire.clone(),
                        provenance_payload: b"untrusted-transitivity-input".to_vec(),
                    },
                }
            })
            .chain(std::iter::once(KernelFormulaPayload {
                formula_ref: vc.goal,
                projection: KernelFormulaProjection {
                    formula_fingerprint: KernelEvidenceFingerprint {
                        algorithm_id: 2,
                        digest: goal.0.canonical_hash_input().unwrap(),
                    },
                    formula_bytes: goal.1.clone(),
                    provenance_payload: b"untrusted-transitivity-goal".to_vec(),
                },
            }))
            .collect::<Vec<_>>();
        let substitutions = if profile == 13 {
            vec![KernelSubstitutionPayload {
                substitution_id: 1,
                source_formula: formulas[0].formula_ref,
                binder_context_encoding: vec![
                    0, 1, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 11, 0, 0, 0, 0,
                ],
                payload: vec![
                    0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 11, 1, 0, 0, 0, 11, 1,
                ],
                freshness_witnesses: vec![],
                free_variable_constraints: vec![],
                provenance_payload: b"untrusted-substitution".to_vec(),
            }]
        } else {
            vec![]
        };
        let symbols = [(8, 1), (9, 2), (7, 3), (10, 3)]
            .into_iter()
            .enumerate()
            .map(|(index, (id, kind))| KernelManifestEntry {
                entry_id: index as u32,
                payload: vec![kind, 0, 0, 0, id],
            })
            .collect::<Vec<_>>();
        let variables = [11u32, 12, 13, 14].map(|id| KernelManifestEntry {
            entry_id: id,
            payload: id.to_be_bytes().to_vec(),
        });
        assert!(profile != 8 || variables.iter().all(|entry| entry.entry_id != middle.0));
        let handoff = build_kernel_evidence_handoff(KernelEvidenceHandoffInput {
            vc_set: &vcs,
            vc: vc.id,
            goal_polarity: KernelGoalPolarity::AssertFalseForRefutation,
            kernel_profile: KernelEvidenceProfile::v1(1, KernelClauseTautologyPolicy::Reject),
            symbol_manifest: &symbols,
            variable_manifest: &variables,
            formula_payloads: &formulas,
            imported_formula_payloads: &[],
            substitutions: &substitutions,
            formula_context: None,
            discharge_output: None,
        })
        .unwrap();
        let mut wire = handoff
            .encode_local_evidence(if profile == 14 { Some(&[]) } else { None })
            .unwrap();
        if profile == 15 {
            let goal_size =
                1 + 1 + 4 + goal.0.canonical_hash_input().unwrap().len() + 4 + goal.1.len();
            let offset = wire.len() - goal_size;
            assert_eq!(wire[offset], 1);
            wire[offset] = 2;
        }
        let target = handoff.canonical_evidence().target_vc();
        let mut parse_context = FormulaEvidenceParseContext::v1(
            Fingerprint::new(target.algorithm_id, target.digest.clone()),
            KernelProfileRecord::v1(1, ClauseTautologyPolicy::Reject),
        );
        parse_context.accepted_schema_versions.insert(2);
        let parsed = parse_formula_evidence(&wire, &parse_context).unwrap();
        let pristine = parsed.clone();
        let result = encode_local_variable_transitivity_names(&parsed, middle);
        assert_eq!(
            encode_local_variable_transitivity_names(&parsed, middle),
            result
        );
        if profile == 0 {
            assert_eq!(
                result.unwrap(),
                [vec![
                    0, 0, 0, 1, 3, 0, 0, 0, 7, 1, 0, 0, 0, 11, 1, 0, 0, 0, 12, 1, 0, 0, 0, 13
                ]]
            );
        } else {
            assert_eq!(
                result.unwrap_err(),
                "unsupported local-variable transitivity input",
                "profile {profile}"
            );
        }
        assert_eq!(parsed, pristine);
        if profile == 12 {
            let Formula::Atom(atom) = &parsed.final_goal().formula else {
                panic!()
            };
            assert_eq!(atom.arguments.len(), 3);
        }
        assert_eq!(parsed.substitutions().len(), usize::from(profile == 13));
        assert_eq!(parsed.schema_version(), if profile == 14 { 2 } else { 1 });
        if profile == 15 {
            assert_eq!(
                parsed.final_goal().polarity,
                GoalPolarity::AssertTrueForConsistency
            );
        }
    }
}
#[test]
fn source_algorithm_state_syntax_payloads_preserve_exact_declarations() {
    use mizar_resolve::{
        declarations::{DeclarationShellCollector, DeclarationShellKind},
        env::SymbolKind,
        symbols::{
            source_algorithm_state_syntax_payloads_from_arena as syntax,
            source_algorithm_syntax_payloads_from_arena as old_syntax, validate_source_symbol_env,
        },
    };
    let fixture = include_str!(
        "../../../../../tests/miz/pass/algorithms/pass_proof_verification_algorithm_var_const_assert_001.miz"
    );
    assert_eq!(fixture.matches("    assert x = a;\n").count(), 1);
    let original = fixture.replacen("    assert x = a;\n", "", 1);
    let renamed = original
        .replace("swapalgo", "copyalgo")
        .replace("let a be", "let input be")
        .replace("copyalgo(a)", "copyalgo(input)")
        .replace("result = a", "result = input")
        .replace("var x := a", "var local := input")
        .replace("const c := a", "const saved := input")
        .replace("x := c", "local := saved")
        .replace("return x", "return local");
    let independent = original.replacen("    x := c;\n", "    const d := a;\n    x := c;\n", 1);
    let postcondition_hex = concat!(
        "6d697a61722d7265736f6c76652f736f757263652d616c676f726974686d2d73796e7461782f76310000970000000900ab00",
        "000001000200650000000b7465726d696e6174696e67000000000002006500000009616c676f726974686d00000000000200",
        "640000000873776170616c676f00000000009800000003000200660000000128000000000002006400000001610000000000",
        "02006600000001290000000000020066000000022d3e0000000000150000000100190000000100020065000000066f626a65",
        "63740000000000ad000000020002006500000007656e737572657300000000002c00000001002d00000003001c0000000100",
        "1d000000010002006400000006726573756c740000000000020066000000013d00000000001c00000001001d000000010002",
        "00640000000161000000000099000000030002006500000002646f00000000009a00000004009b0000000300020065000000",
        "0376617200000000009c000000030002006400000001780000000000020066000000023a3d00000000001c00000001001d00",
        "0000010002006400000001610000000000020066000000013b00000000009b000000030002006500000005636f6e73740000",
        "0000009c000000030002006400000001630000000000020066000000023a3d00000000001c00000001001d00000001000200",
        "6400000001610000000000020066000000013b00000000009d00000004009e00000001000200640000000178000000000002",
        "0066000000023a3d00000000001c00000001001d000000010002006400000001630000000000020066000000013b00000000",
        "00a000000003000200650000000672657475726e00000000001c00000001001d000000010002006400000001780000000000",
        "020066000000013b000000000002006500000003656e640000000000020066000000013b00000000",
    );
    let renamed_hex = concat!(
        "6d697a61722d7265736f6c76652f736f757263652d616c676f726974686d2d73796e7461782f76310000970000000900ab00",
        "000001000200650000000b7465726d696e6174696e67000000000002006500000009616c676f726974686d00000000000200",
        "6400000008636f7079616c676f00000000009800000003000200660000000128000000000002006400000005696e70757400",
        "0000000002006600000001290000000000020066000000022d3e000000000015000000010019000000010002006500000006",
        "6f626a6563740000000000ad000000020002006500000007656e737572657300000000002c00000001002d00000003001c00",
        "000001001d000000010002006400000006726573756c740000000000020066000000013d00000000001c00000001001d0000",
        "00010002006400000005696e707574000000000099000000030002006500000002646f00000000009a00000004009b000000",
        "03000200650000000376617200000000009c0000000300020064000000056c6f63616c0000000000020066000000023a3d00",
        "000000001c00000001001d000000010002006400000005696e7075740000000000020066000000013b00000000009b000000",
        "030002006500000005636f6e737400000000009c00000003000200640000000573617665640000000000020066000000023a",
        "3d00000000001c00000001001d000000010002006400000005696e7075740000000000020066000000013b00000000009d00",
        "000004009e0000000100020064000000056c6f63616c0000000000020066000000023a3d00000000001c00000001001d0000",
        "0001000200640000000573617665640000000000020066000000013b0000000000a000000003000200650000000672657475",
        "726e00000000001c00000001001d0000000100020064000000056c6f63616c0000000000020066000000013b000000000002",
        "006500000003656e640000000000020066000000013b00000000",
    );
    let independent_hex = concat!(
        "6d697a61722d7265736f6c76652f736f757263652d616c676f726974686d2d73796e7461782f76310000970000000900ab00",
        "000001000200650000000b7465726d696e6174696e67000000000002006500000009616c676f726974686d00000000000200",
        "640000000873776170616c676f00000000009800000003000200660000000128000000000002006400000001610000000000",
        "02006600000001290000000000020066000000022d3e0000000000150000000100190000000100020065000000066f626a65",
        "63740000000000ad000000020002006500000007656e737572657300000000002c00000001002d00000003001c0000000100",
        "1d000000010002006400000006726573756c740000000000020066000000013d00000000001c00000001001d000000010002",
        "00640000000161000000000099000000030002006500000002646f00000000009a00000005009b0000000300020065000000",
        "0376617200000000009c000000030002006400000001780000000000020066000000023a3d00000000001c00000001001d00",
        "0000010002006400000001610000000000020066000000013b00000000009b000000030002006500000005636f6e73740000",
        "0000009c000000030002006400000001630000000000020066000000023a3d00000000001c00000001001d00000001000200",
        "6400000001610000000000020066000000013b00000000009b000000030002006500000005636f6e737400000000009c0000",
        "00030002006400000001640000000000020066000000023a3d00000000001c00000001001d00000001000200640000000161",
        "0000000000020066000000013b00000000009d00000004009e00000001000200640000000178000000000002006600000002",
        "3a3d00000000001c00000001001d000000010002006400000001630000000000020066000000013b0000000000a000000003",
        "000200650000000672657475726e00000000001c00000001001d000000010002006400000001780000000000020066000000",
        "013b000000000002006500000003656e640000000000020066000000013b00000000",
    );
    for (text, name, hex, length) in [
        (&original, "swapalgo", postcondition_hex, 690),
        (&renamed, "copyalgo", renamed_hex, 726),
        (&independent, "swapalgo", independent_hex, 784),
    ] {
        let expected = hex
            .as_bytes()
            .as_chunks::<2>()
            .0
            .iter()
            .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
            .collect::<Vec<_>>();
        assert_eq!(expected.len(), length);
        let (frontend, source, collected) =
            source_registration_owner_inputs(text, "owner-test", "owners", 0);
        assert!(frontend.diagnostics.is_empty());
        assert!(collected.diagnostics().is_empty());
        validate_source_symbol_env(&source, collected.env()).unwrap();
        let shells =
            DeclarationShellCollector::new(frontend.ast.as_ref().unwrap(), source.module())
                .collect();
        let algorithms = shells
            .declarations()
            .iter()
            .filter(|shell| shell.kind() == DeclarationShellKind::AlgorithmDefinition)
            .collect::<Vec<_>>();
        assert_eq!(algorithms.len(), 1);
        let parent = shells.declaration(algorithms[0].parent().unwrap()).unwrap();
        assert_eq!(parent.kind(), DeclarationShellKind::DefinitionBlock);
        assert!(parent.parent().is_none());
        let entries = collected
            .env()
            .symbols()
            .iter()
            .filter(|entry| entry.kind() == SymbolKind::Algorithm)
            .collect::<Vec<_>>();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].primary_spelling(), name);
        let before = (source.clone(), collected.env().clone());
        let values = syntax(&source, collected.env()).unwrap();
        assert_eq!(values.len(), 1);
        assert_eq!(values.get(entries[0].symbol()), Some(&expected));
        assert_eq!(syntax(&source, collected.env()).unwrap(), values);
        assert_eq!((source, collected.env().clone()), before);
    }
    let combined = format!("{original}\n{renamed}");
    let (frontend, source, collected) =
        source_registration_owner_inputs(&combined, "owner-test", "owners", 0);
    assert!(frontend.diagnostics.is_empty());
    validate_source_symbol_env(&source, collected.env()).unwrap();
    let entries = collected
        .env()
        .symbols()
        .iter()
        .filter(|entry| entry.kind() == SymbolKind::Algorithm)
        .collect::<Vec<_>>();
    let before = (source.clone(), collected.env().clone());
    let values = syntax(&source, collected.env()).unwrap();
    assert_eq!(values.len(), 2);
    assert_eq!(
        values.keys().collect::<std::collections::BTreeSet<_>>(),
        entries
            .iter()
            .map(|entry| entry.symbol())
            .collect::<std::collections::BTreeSet<_>>()
    );
    for (name, hex) in [("swapalgo", postcondition_hex), ("copyalgo", renamed_hex)] {
        let entry = entries
            .iter()
            .find(|entry| entry.primary_spelling() == name)
            .unwrap();
        let expected = hex
            .as_bytes()
            .as_chunks::<2>()
            .0
            .iter()
            .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
            .collect::<Vec<_>>();
        assert_eq!(values.get(entry.symbol()), Some(&expected));
    }
    assert_eq!((source, collected.env().clone()), before);
    for text in ["", "theorem T: for X being set holds X = X;"] {
        let (frontend, source, collected) =
            source_registration_owner_inputs(text, "owner-test", "owners", 0);
        assert!(frontend.diagnostics.is_empty());
        validate_source_symbol_env(&source, collected.env()).unwrap();
        assert!(syntax(&source, collected.env()).unwrap().is_empty());
    }
    let return_only = include_str!(
        "../../../../../tests/miz/pass/algorithms/pass_proof_verification_algorithm_ensures_return_001.miz"
    );
    let (_, source, collected) =
        source_registration_owner_inputs(return_only, "owner-test", "owners", 0);
    validate_source_symbol_env(&source, collected.env()).unwrap();
    assert_eq!(
        syntax(&source, collected.env()).unwrap(),
        old_syntax(&source, collected.env()).unwrap()
    );
}

#[test]
fn source_algorithm_state_syntax_payloads_preserve_source_associations_and_locality() {
    use mizar_resolve::{
        env::{SymbolKind, Visibility},
        symbols::{
            source_algorithm_state_syntax_payloads_from_arena as syntax, validate_source_symbol_env,
        },
    };
    let fixture = include_str!(
        "../../../../../tests/miz/pass/algorithms/pass_proof_verification_algorithm_var_const_assert_001.miz"
    );
    let original = fixture.replacen("    assert x = a;\n", "", 1);
    let second = original.replace("swapalgo", "otheralgo");
    let text = format!("{original}\n{second}");
    let (_, source, collected) = source_registration_owner_inputs(&text, "owner-test", "owners", 0);
    validate_source_symbol_env(&source, collected.env()).unwrap();
    let prior = syntax(&source, collected.env()).unwrap();
    let baseline = collected
        .env()
        .symbols()
        .iter()
        .filter(|entry| entry.kind() == SymbolKind::Algorithm)
        .map(|entry| {
            (
                entry.primary_spelling().to_owned(),
                prior[entry.symbol()].clone(),
            )
        })
        .collect::<std::collections::BTreeMap<_, _>>();
    assert_eq!(baseline.len(), 2);
    let before = (source.clone(), collected.env().clone(), prior.clone());
    let trivia = format!(":: λ 日本語\n{}", text.replace(' ', "  "));
    let reordered = format!("{second}\n{original}");
    let private = format!(
        "{}\n{second}",
        original.replace("terminating algorithm", "private terminating algorithm")
    );
    let unterminated = format!(
        "{}\n{second}",
        original.replace("terminating algorithm", "algorithm")
    );
    let renamed = format!("{}\n{second}", original.replace("swapalgo", "newalgo"));
    let binder = format!(
        "{}\n{second}",
        original
            .replace("var x", "var local")
            .replace("x := c", "local := c")
            .replace("return x", "return local")
    );
    let assigned = format!("{}\n{second}", original.replace("x := c", "x := a"));
    let returned = format!("{}\n{second}", original.replace("return x", "return c"));
    let outer = format!(
        "{}\n{second}",
        original.replace("let a be object", "let z be object")
    );
    let header = format!(
        "{}\n{second}",
        original.replace("swapalgo(a)", "swapalgo(b)")
    );
    for (index, edited, package, module, skip) in [
        (0, trivia, "owner-test", "owners", 0),
        (1, text.clone(), "owner-test", "owners", 7),
        (2, reordered, "owner-test", "owners", 0),
        (3, text.clone(), "other-package", "owners", 0),
        (4, text.clone(), "owner-test", "other_module", 0),
        (5, private, "owner-test", "owners", 0),
        (6, unterminated, "owner-test", "owners", 0),
        (7, renamed, "owner-test", "owners", 0),
        (8, binder, "owner-test", "owners", 0),
        (9, assigned, "owner-test", "owners", 0),
        (10, returned, "owner-test", "owners", 0),
        (11, outer, "owner-test", "owners", 0),
        (12, header, "owner-test", "owners", 0),
    ] {
        let (frontend, current, entries) =
            source_registration_owner_inputs(&edited, package, module, skip);
        assert!(frontend.diagnostics.is_empty(), "edit {index}");
        assert!(entries.diagnostics().is_empty(), "edit {index}");
        validate_source_symbol_env(&current, entries.env()).unwrap();
        let input_before = (current.clone(), entries.env().clone());
        let values = syntax(&current, entries.env()).unwrap();
        let actual = entries
            .env()
            .symbols()
            .iter()
            .filter(|entry| entry.kind() == SymbolKind::Algorithm)
            .collect::<Vec<_>>();
        assert_eq!(actual.len(), 2);
        assert_eq!(
            values.keys().collect::<std::collections::BTreeSet<_>>(),
            actual
                .iter()
                .map(|entry| entry.symbol())
                .collect::<std::collections::BTreeSet<_>>()
        );
        for entry in actual {
            let name = if entry.primary_spelling() == "newalgo" {
                "swapalgo"
            } else {
                entry.primary_spelling()
            };
            if matches!(index, 6..=10 | 12) && name == "swapalgo" {
                assert_ne!(&values[entry.symbol()], &baseline[name], "edit {index}");
            } else {
                assert_eq!(&values[entry.symbol()], &baseline[name], "edit {index}");
            }
            assert_eq!(entry.symbol().module(), current.module());
            assert_eq!(
                entry.visibility(),
                if index == 5 && name == "swapalgo" {
                    Visibility::Private
                } else {
                    Visibility::Public
                }
            );
        }
        if index == 1 {
            assert_ne!(current.source_id(), source.source_id());
        }
        if matches!(index, 2..=4) {
            assert_ne!(
                values.keys().collect::<Vec<_>>(),
                prior.keys().collect::<Vec<_>>()
            );
        }
        assert_eq!((current, entries.env().clone()), input_before);
    }
    assert_eq!((source, collected.env().clone(), prior), before);
}

#[test]
fn source_algorithm_state_syntax_payloads_preserve_upstream_and_atomic_refusals() {
    use mizar_resolve::{
        env::SymbolKind,
        symbols::{
            source_algorithm_state_syntax_payloads_from_arena as syntax, validate_source_symbol_env,
        },
    };
    use mizar_syntax::{SurfaceNodeKind, SurfaceTokenKind};
    let original = include_str!(
        "../../../../../tests/miz/pass/algorithms/pass_proof_verification_algorithm_var_const_assert_001.miz"
    );
    let supported = original.replacen("    assert x = a;\n", "", 1);
    let (_, source, collected) =
        source_registration_owner_inputs(&supported, "owner-test", "owners", 0);
    validate_source_symbol_env(&source, collected.env()).unwrap();
    let prior = syntax(&source, collected.env()).unwrap();
    let before = (source.clone(), collected.env().clone(), prior.clone());
    for edited in [
        supported.replace("swapalgo", "foreignalgo"),
        format!(
            "{supported}\n{}",
            supported.replace("swapalgo", "otheralgo")
        ),
    ] {
        let (frontend, current, entries) =
            source_registration_owner_inputs(&edited, "owner-test", "owners", 0);
        assert!(frontend.diagnostics.is_empty());
        assert!(entries.diagnostics().is_empty());
        validate_source_symbol_env(&current, entries.env()).unwrap();
        assert!(syntax(&current, entries.env()).is_ok());
        let mismatch = validate_source_symbol_env(&source, entries.env()).unwrap_err();
        assert_eq!(syntax(&source, entries.env()).unwrap_err(), mismatch);
        let mismatch = validate_source_symbol_env(&current, collected.env()).unwrap_err();
        assert_eq!(syntax(&current, collected.env()).unwrap_err(), mismatch);
    }
    let while_source = include_str!(
        "../../../../../tests/miz/pass/algorithms/pass_proof_verification_algorithm_while_invariant_001.miz"
    );
    for (edited, ghost) in [
        (original.to_owned(), false),
        (while_source.to_owned(), false),
        (format!("{supported}\n{while_source}"), false),
        (
            supported.replacen("    var x := a;", "    ghost var x := a;", 1),
            true,
        ),
        (
            supported.replacen("    const c := a;", "    ghost const c := a;", 1),
            true,
        ),
        (
            supported.replacen("    x := c;", "    ghost x := c;", 1),
            true,
        ),
    ] {
        let (frontend, current, entries) =
            source_registration_owner_inputs(&edited, "owner-test", "owners", 0);
        assert!(frontend.diagnostics.is_empty());
        assert!(entries.diagnostics().is_empty());
        let ast = frontend.ast.as_ref().unwrap();
        assert!(ast.node_views().any(|node| match node.kind() {
            SurfaceNodeKind::Token(token) if ghost =>
                token.kind == SurfaceTokenKind::ReservedWord && token.text.as_ref() == "ghost",
            SurfaceNodeKind::AssertStatement | SurfaceNodeKind::WhileStatement => !ghost,
            _ => false,
        }));
        assert!(
            entries
                .env()
                .symbols()
                .iter()
                .any(|entry| entry.kind() == SymbolKind::Algorithm)
        );
        validate_source_symbol_env(&current, entries.env()).unwrap();
        let input_before = (current.clone(), entries.env().clone());
        assert_eq!(
            syntax(&current, entries.env()).unwrap_err(),
            "unsupported source algorithm state syntax payload"
        );
        assert_eq!((current, entries.env().clone()), input_before);
    }
    assert_eq!(syntax(&source, collected.env()).unwrap(), prior);
    assert_eq!((source, collected.env().clone(), prior), before);
}

#[test]
fn source_algorithm_state_transitivity_selection_reaches_normal_kernel() {
    use mizar_atp::backend::{
        encode_local_variable_transitivity_names, select_local_variable_transitivity_names,
    };
    use mizar_core::core_ir::CoreAlgorithmStmtKind as S;
    use mizar_kernel::checker::KernelContextIdentitySource as KernelSource;
    use mizar_kernel::{certificate_parser::*, checker::*, clause::*, formula_evidence::*};
    use mizar_vc::kernel_evidence_handoff::KernelContextIdentitySource as VcSource;
    use mizar_vc::{kernel_evidence_handoff::*, vc_ir::*};
    let case = step5c14_state_case();
    let original = std::fs::read_to_string(&case.source_path).unwrap();
    assert_eq!(original.matches("    assert x = a;\n").count(), 1);
    let base = original.replacen("    assert x = a;\n", "", 1);
    for (index, source) in [
        base.clone(),
        base.replace("swapalgo", "copyalgo"),
        base.replace("    x := c;", "    const d := a;\n    x := c;"),
    ]
    .into_iter()
    .enumerate()
    {
        let terminal = if index == 2 { 4 } else { 3 };
        let literal = if index == 2 {
            "000000010300000000010000000401000000020100000000"
        } else {
            "000000010300000000010000000301000000020100000000"
        };
        let core = step5c14_static_core(&case, &source).unwrap();
        let (_, algorithm) = core.algorithms().iter().next().unwrap();
        let statements = &algorithm.statements;
        let (S::Let { binder: x, .. }, S::Let { binder: c, .. }) = (
            &core.algorithm_statements().get(statements[0]).unwrap().kind,
            &core.algorithm_statements().get(statements[1]).unwrap().kind,
        ) else {
            panic!("actual checked declarations")
        };
        let assignment = statements[statements.len() - 2];
        let S::AssignLocal { target, value: rhs } =
            core.algorithm_statements().get(assignment).unwrap().kind
        else {
            panic!("actual checked write")
        };
        assert_eq!(target, x.var);
        assert_eq!(
            core.terms().get(rhs).unwrap().kind,
            mizar_core::core_ir::CoreTermKind::Var(c.var)
        );
        let value = |var, definition| VcProgramValue { var, definition };
        let a = value(algorithm.params[0].var, None);
        let c0 = value(c.var, Some(statements[1]));
        let x1 = value(x.var, Some(assignment));
        let vcs = step5c14_state_vcs(&core).unwrap();
        for (left, right) in [(c0, a), (x1, c0)] {
            assert!(vcs.generated_formulas().iter().any(|formula| formula.kind
                == VcGeneratedFormulaKind::AlgorithmStateFact
                && formula.shape == VcGeneratedFormulaShape::ProgramEquals { left, right }));
        }
        let [vc] = vcs.vcs() else {
            panic!("one actual return VC")
        };
        assert_eq!(
            (vc.kind.clone(), vc.status.clone()),
            (VcKind::AlgorithmPostcondition, VcStatus::Open)
        );
        assert!(!vc.anchor.is_complete());
        assert!(matches!(
            vcs.seed_accounting()[0].mapping,
            SeedVcMapping::NoConcreteVc {
                reason: SeedNoVcReason::DeferredExternal(_)
            }
        ));
        let VcFormulaRef::Generated(goal_id) = vc.goal else {
            panic!()
        };
        assert_eq!(
            vcs.generated_formulas()[goal_id.index()].shape,
            VcGeneratedFormulaShape::ProgramEquals { left: x1, right: a }
        );
        let before = (core.clone(), vcs.clone());
        let generation = GenerationSchemaVersion::new("mizar-vc-generation-step5c14-state-v1");
        let handoff =
            build_source_algorithm_state_kernel_handoff(&core, &vcs, vc.id, &generation).unwrap();
        let fingerprint = handoff.canonical_evidence().target_vc();
        let target = mizar_kernel::rejection::TargetVcFingerprint::new(
            fingerprint.algorithm_id,
            fingerprint.digest.clone(),
        );
        let mut parse_context = FormulaEvidenceParseContext::v1(
            Fingerprint::new(fingerprint.algorithm_id, fingerprint.digest.clone()),
            KernelProfileRecord::v1(1, ClauseTautologyPolicy::Reject),
        );
        let v1 = handoff.encode_local_evidence(None).unwrap();
        let parsed = parse_formula_evidence(&v1, &parse_context).unwrap();
        let Formula::Atom(goal) = &parsed.final_goal().formula else {
            panic!()
        };
        assert_eq!(goal.symbol, SymbolKey::new(SymbolKind::Equality, 0));
        assert_eq!(
            goal.arguments,
            [
                Term::Variable(VariableId(terminal)),
                Term::Variable(VariableId(0))
            ]
        );
        for (left, right) in [(terminal, 2), (2, 0)] {
            assert!(parsed.formulas().iter().any(|row| matches!(&row.formula,
                Formula::Atom(atom) if atom.symbol == goal.symbol &&
                atom.arguments == [Term::Variable(VariableId(left)), Term::Variable(VariableId(right))])));
        }
        let pristine = parsed.clone();
        let names = encode_local_variable_transitivity_names(&parsed, VariableId(2)).unwrap();
        assert_eq!(
            names,
            [literal
                .as_bytes()
                .as_chunks::<2>()
                .0
                .iter()
                .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
                .collect::<Vec<_>>()]
        );
        for limit in [None, Some(0), Some(1), Some(2)] {
            assert_eq!(
                select_local_variable_transitivity_names(&parsed, limit),
                None
            );
        }
        let automatic = select_local_variable_transitivity_names(&parsed, Some(3)).unwrap();
        assert_eq!(automatic, names);
        assert_eq!(
            select_local_variable_transitivity_names(&parsed, Some(u64::MAX)),
            Some(names.clone())
        );
        let entries = handoff
            .context_identity()
            .entries()
            .iter()
            .map(|row| {
                let VcSource::LocalHypothesis { local_context_id } = row.source() else {
                    panic!()
                };
                let producer = match row.producer_formula_ref() {
                    VcFormulaRef::Core(id) => KernelFormulaProducerRef::Core(id),
                    VcFormulaRef::Generated(id) => KernelFormulaProducerRef::Generated(
                        KernelVcGeneratedFormulaId::new(id.index()),
                    ),
                    _ => panic!(),
                };
                let fp = row.formula_fingerprint();
                mizar_kernel::checker::KernelContextIdentityEntry::new(
                    KernelSource::LocalHypothesis { local_context_id },
                    row.formula_id(),
                    Fingerprint::new(fp.algorithm_id, fp.digest.clone()),
                    producer,
                )
            })
            .collect();
        let context = FormulaEvidenceContext::with_context_identity(
            Some(handoff.context_identity_hash().as_bytes().to_vec()),
            vec![],
            vec![],
            Some(mizar_kernel::checker::KernelContextIdentityPayload::new(
                target.clone(),
                handoff.canonical_hash(),
                handoff.context_identity_hash(),
                entries,
            )),
            ImportedFactContextLimits::default(),
        )
        .unwrap();
        parse_context.accepted_schema_versions.insert(2);
        let v2 = handoff.encode_local_evidence(Some(&automatic)).unwrap();
        for (wire, status) in [
            (&v1, KernelCheckStatus::Rejected),
            (&v2, KernelCheckStatus::Accepted),
        ] {
            let evidence = parse_formula_evidence(wire, &parse_context).unwrap();
            let checked = check_kernel_evidence(KernelEvidenceCheckInput {
                target_vc_fingerprint: &target,
                evidence: &evidence,
                formula_context: Some(&context),
                check_kind: KernelEvidenceCheckKind::ProofObligation,
                policy: KernelCheckPolicy::default(),
                limits: KernelEvidenceCheckLimits::default(),
            });
            assert_eq!(checked.status(), status);
            assert_eq!(
                checked.evidence_check_kind(),
                Some(KernelEvidenceCheckKind::ProofObligation)
            );
            assert!(!checked.policy_taint());
            assert!(checked.used_axioms().is_empty());
        }
        assert_eq!(
            select_local_variable_transitivity_names(&parsed, Some(3)),
            Some(automatic)
        );
        assert_eq!(parsed, pristine);
        assert_eq!((core, vcs), before);
        assert_eq!(handoff.encode_local_evidence(None).unwrap(), v1);
    }
}

#[test]
fn source_algorithm_state_transitivity_selection_respects_order_and_limits() {
    use mizar_atp::backend::{
        encode_local_variable_transitivity_names, select_local_variable_transitivity_names,
    };
    use mizar_kernel::{
        certificate_parser::{ClauseTautologyPolicy, Fingerprint, KernelProfileRecord},
        clause::{Atom, SymbolKey, SymbolKind, Term, VariableId},
        formula_evidence::{Formula, FormulaEvidenceParseContext, parse_formula_evidence},
    };
    use mizar_vc::kernel_evidence_handoff::*;
    let case = step5c14_state_case();
    let source = std::fs::read_to_string(&case.source_path)
        .unwrap()
        .replacen("    assert x = a;\n", "", 1);
    let core = step5c14_static_core(&case, &source).unwrap();
    let vcs = step5c14_state_vcs(&core).unwrap();
    let [vc] = vcs.vcs() else { panic!() };
    assert!(vc.local_context.entries().len() >= 4);
    let before = (core.clone(), vcs.clone());
    let equality = SymbolKey::new(SymbolKind::Equality, 7);
    // These generic projections are untrusted data, never source or Kernel acceptance evidence.
    let atom = |left: u32, right: u32| {
        let formula = Formula::Atom(Atom::new(
            equality,
            vec![
                Term::Variable(VariableId(left)),
                Term::Variable(VariableId(right)),
            ],
        ));
        let mut wire = vec![1, 3];
        wire.extend(7u32.to_be_bytes());
        wire.extend(2u32.to_be_bytes());
        wire.extend(2u32.to_be_bytes());
        for id in [left, right] {
            wire.push(1);
            wire.extend(id.to_be_bytes());
        }
        KernelFormulaProjection {
            formula_fingerprint: KernelEvidenceFingerprint {
                algorithm_id: 2,
                digest: formula.canonical_hash_input().unwrap(),
            },
            formula_bytes: wire,
            provenance_payload: b"untrusted-selector-input".to_vec(),
        }
    };
    let variables = [5u32, 11, 14, 27, 40].map(|id| KernelManifestEntry {
        entry_id: id,
        payload: id.to_be_bytes().to_vec(),
    });
    let symbols = [KernelManifestEntry {
        entry_id: 7,
        payload: vec![3, 0, 0, 0, 7],
    }];
    for profile in 0..4 {
        let mut pairs = [(40, 27), (27, 5), (40, 14), (14, 5)];
        if profile == 1 {
            pairs.swap(0, 2);
            pairs.swap(1, 3);
        }
        if profile == 2 {
            pairs[1] = (5, 27);
            pairs[3] = (5, 14);
        }
        let payloads = vc
            .local_context
            .entries()
            .iter()
            .enumerate()
            .map(|(index, row)| {
                let (left, right) = pairs.get(index).copied().unwrap_or((11, 11));
                KernelFormulaPayload {
                    formula_ref: row.formula.unwrap(),
                    projection: atom(left, right),
                }
            })
            .chain(std::iter::once(KernelFormulaPayload {
                formula_ref: vc.goal,
                projection: atom(40, 5),
            }))
            .collect::<Vec<_>>();
        let handoff = build_kernel_evidence_handoff(KernelEvidenceHandoffInput {
            vc_set: &vcs,
            vc: vc.id,
            goal_polarity: KernelGoalPolarity::AssertFalseForRefutation,
            kernel_profile: KernelEvidenceProfile::v1(1, KernelClauseTautologyPolicy::Reject),
            symbol_manifest: &symbols,
            variable_manifest: &variables,
            formula_payloads: &payloads,
            imported_formula_payloads: &[],
            substitutions: &[],
            formula_context: None,
            discharge_output: None,
        })
        .unwrap();
        let wire = handoff
            .encode_local_evidence(if profile == 3 { Some(&[]) } else { None })
            .unwrap();
        let fingerprint = handoff.canonical_evidence().target_vc();
        let mut context = FormulaEvidenceParseContext::v1(
            Fingerprint::new(fingerprint.algorithm_id, fingerprint.digest.clone()),
            KernelProfileRecord::v1(1, ClauseTautologyPolicy::Reject),
        );
        context.accepted_schema_versions.insert(2);
        let parsed = parse_formula_evidence(&wire, &context).unwrap();
        let pristine = parsed.clone();
        assert_eq!(parsed.schema_version(), if profile == 3 { 2 } else { 1 });
        assert_eq!(
            parsed
                .variable_manifest()
                .iter()
                .map(|row| row.variable_id.0)
                .collect::<Vec<_>>(),
            [5, 11, 14, 27, 40]
        );
        for limit in [None, Some(0), Some(1), Some(2)] {
            assert_eq!(
                select_local_variable_transitivity_names(&parsed, limit),
                None
            );
        }
        let expected = if profile < 2 {
            let positions = [27, 14].map(|middle| parsed.formulas().iter().position(|row| matches!(&row.formula,
                Formula::Atom(atom) if atom.symbol == equality &&
                atom.arguments == [Term::Variable(VariableId(40)), Term::Variable(VariableId(middle))])).unwrap());
            assert_eq!(positions[0] < positions[1], profile == 0);
            for middle in [5, 11] {
                assert!(
                    encode_local_variable_transitivity_names(&parsed, VariableId(middle)).is_err()
                );
            }
            let names = encode_local_variable_transitivity_names(&parsed, VariableId(14)).unwrap();
            assert_eq!(
                names,
                [vec![
                    0, 0, 0, 1, 3, 0, 0, 0, 7, 1, 0, 0, 0, 40, 1, 0, 0, 0, 14, 1, 0, 0, 0, 5
                ]]
            );
            assert_ne!(
                encode_local_variable_transitivity_names(&parsed, VariableId(27)).unwrap(),
                names
            );
            Some(names)
        } else {
            None
        };
        assert_eq!(
            select_local_variable_transitivity_names(&parsed, Some(3)),
            expected
        );
        assert_eq!(
            select_local_variable_transitivity_names(&parsed, Some(u64::MAX)),
            expected
        );
        assert_eq!(
            select_local_variable_transitivity_names(&parsed, Some(3)),
            expected
        );
        assert_eq!(parsed, pristine);
    }
    assert_eq!((core, vcs), before);
}
