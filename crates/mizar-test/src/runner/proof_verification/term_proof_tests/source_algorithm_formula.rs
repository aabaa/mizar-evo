use super::*;

#[test]
fn source_algorithm_formula_payloads_preserve_actual_content() {
    use mizar_checker::type_checker::check_source_algorithm_types;
    use mizar_core::{
        binder_normalization::{
            BinderContext, CanonicalFormulaKind as F, CanonicalTermKind as T, CanonicalVar,
            NormalizedVarClass, NormalizedVarSort, canonical_formula, normalize_core_formula,
        },
        control_flow::{build_control_flow_ir, build_obligation_seed_handoff},
        core_ir::*,
        elaborator::{lower_source_algorithms, source_algorithm_formula_payloads},
    };
    use std::collections::{BTreeMap, BTreeSet};
    let config = config();
    let plan = crate::harness::build_test_plan(&config).unwrap();
    for (profile, id, owner) in [
        (
            "while",
            "pass_proof_verification_algorithm_while_invariant_001",
            "loopalgo",
        ),
        (
            "state",
            "pass_proof_verification_algorithm_var_const_assert_001",
            "swapalgo",
        ),
        (
            "ghost",
            "pass_proof_verification_algorithm_ghost_snapshot_001",
            "ghostalgo",
        ),
        (
            "choice",
            "fail_proof_verification_algorithm_ensures_unprovable_001",
            "badalgo",
        ),
    ] {
        let case = plan.cases.iter().find(|case| case.id.0 == id).unwrap();
        let original = fs::read_to_string(&case.source_path).unwrap();
        let mut baseline = None;
        for edit in 0..if profile == "state" { 7 } else { 4 } {
            let text = match edit {
                0 => original.clone(),
                1 => original
                    .replace(" a ", " param ")
                    .replace("(a)", "(param)")
                    .replace(" a;", " param;")
                    .replace("= a\n", "= param\n")
                    .replace(" x", " local"),
                2 => format!(":: neutral algorithm trivia\n\n{original}"),
                3 => original.replace(owner, "renamed_algorithm"),
                4 => original.replace("assert x = a", "assert a = x"),
                5 => original.replace("assert x = a", "assert x = x"),
                6 => original.replace("return x;", "return c;"),
                _ => unreachable!(),
            };
            let frontend = super::super::super::formula_statement::step5c8_test_frontend(&text);
            let (source, typed, symbols) = super::super::super::source_registration_inputs(
                &config.workspace_root,
                case,
                frontend,
            )
            .unwrap();
            let check = check_source_algorithm_types(&source, &typed, &symbols).unwrap();
            let core = lower_source_algorithms(&check).unwrap();
            assert_eq!(core.items().len(), 1);
            assert_eq!(core.algorithms().len(), 1);
            assert!(
                core.definitions().is_empty()
                    && core.proofs().is_empty()
                    && core.proof_nodes().is_empty()
                    && core.generated().is_empty()
                    && core.diagnostics().is_empty()
            );
            let (item_id, item) = core.items().iter().next().unwrap();
            let (algorithm_id, algorithm) = core.algorithms().iter().next().unwrap();
            assert_eq!(item.kind, CoreItemKind::Algorithm);
            assert_eq!(item.status, CoreItemStatus::Valid);
            assert_eq!(algorithm.item, item_id);
            assert_eq!(&algorithm.symbol, &check.algorithm().1);
            assert_eq!(algorithm.symbol, item.symbol);
            assert!(
                item.dependencies.is_empty()
                    && item.diagnostics.is_empty()
                    && algorithm.diagnostics.is_empty()
                    && algorithm.ghost_effects.is_empty()
            );
            assert!(
                algorithm.contracts.requires.is_empty()
                    && algorithm.contracts.invariants.is_empty()
                    && algorithm.contracts.assertions.is_empty()
                    && algorithm.contracts.decreasing.is_empty()
            );
            assert_eq!(algorithm.params.len(), 1);
            assert_eq!(algorithm.contracts.ensures.len(), 1);
            let mut binders = vec![&algorithm.params[0], algorithm.result.as_ref().unwrap()];
            let roles = match profile {
                "state" => vec!["parameter", "result", "local:var", "local:const"],
                "choice" => vec!["parameter", "result", "pick"],
                _ => vec!["parameter", "result", "local:var"],
            };
            for (_, statement) in core.algorithm_statements().iter() {
                assert_eq!(statement.owner, algorithm_id);
                assert!(statement.diagnostics.is_empty());
                match &statement.kind {
                    CoreAlgorithmStmtKind::Let { binder, .. }
                    | CoreAlgorithmStmtKind::Pick { binder, .. } => binders.push(binder),
                    _ => {}
                }
            }
            assert_eq!(binders.len(), roles.len());
            assert_eq!(
                binders.iter().map(|b| b.var).collect::<BTreeSet<_>>().len(),
                binders.len()
            );
            let mut context = BinderContext::new();
            let mut trees = BTreeMap::new();
            for (slot, (binder, role)) in binders.iter().zip(&roles).enumerate() {
                assert_eq!(binder.role.as_str(), *role);
                let generated = slot == 1 || *role == "pick";
                context.declare_variable(
                    binder.var,
                    if generated {
                        NormalizedVarClass::Generated
                    } else {
                        NormalizedVarClass::Free
                    },
                    binder.role.clone(),
                    NormalizedVarSort::Term,
                );
                let guard = binder.ty_guard.unwrap();
                let CoreFormulaKind::TypePred { subject, ty } =
                    &core.formulas().get(guard).unwrap().kind
                else {
                    panic!()
                };
                assert_eq!(
                    core.terms().get(*subject).unwrap().kind,
                    CoreTermKind::Var(binder.var)
                );
                assert_eq!(ty.as_str(), if *role == "pick" { "set" } else { "object" });
                let expected = match slot {
                    0 => b"\x03\0\0\0\x06object\x02\0\0\0\0".to_vec(),
                    1 => b"\x03\0\0\0\x06object\x04\0\0\0\x01".to_vec(),
                    2 if *role == "pick" => b"\x03\0\0\0\x03set\x04\0\0\0\x02".to_vec(),
                    2 => b"\x03\0\0\0\x06object\x02\0\0\0\x02".to_vec(),
                    3 => b"\x03\0\0\0\x06object\x02\0\0\0\x03".to_vec(),
                    _ => unreachable!(),
                };
                assert!(trees.insert(guard, expected).is_none());
            }
            let ensures = algorithm.contracts.ensures[0];
            let CoreFormulaKind::Equals { left, right } =
                &core.formulas().get(ensures).unwrap().kind
            else {
                panic!()
            };
            assert_eq!(
                core.terms().get(*left).unwrap().kind,
                CoreTermKind::Var(binders[1].var)
            );
            assert_eq!(
                core.terms().get(*right).unwrap().kind,
                CoreTermKind::Var(binders[0].var)
            );
            trees.insert(ensures, b"\x02\x04\0\0\0\x01\x02\0\0\0\0".to_vec());
            let mut assertion = None;
            let mut internal = BTreeSet::new();
            for (_, statement) in core.algorithm_statements().iter() {
                match &statement.kind {
                    CoreAlgorithmStmtKind::Assert { formula } => {
                        assertion = Some(*formula);
                        let CoreFormulaKind::Equals { left, right } =
                            &core.formulas().get(*formula).unwrap().kind
                        else {
                            panic!()
                        };
                        let (l, r, expected) = match edit {
                            4 => (0, 2, b"\x02\x02\0\0\0\0\x02\0\0\0\x02".as_slice()),
                            5 => (2, 2, b"\x02\x02\0\0\0\x02\x02\0\0\0\x02".as_slice()),
                            _ => (2, 0, b"\x02\x02\0\0\0\x02\x02\0\0\0\0".as_slice()),
                        };
                        assert_eq!(
                            core.terms().get(*left).unwrap().kind,
                            CoreTermKind::Var(binders[l].var)
                        );
                        assert_eq!(
                            core.terms().get(*right).unwrap().kind,
                            CoreTermKind::Var(binders[r].var)
                        );
                        trees.insert(*formula, expected.to_vec());
                    }
                    CoreAlgorithmStmtKind::While {
                        condition,
                        invariants,
                        decreasing,
                        body,
                    } => {
                        assert!(decreasing.is_empty());
                        assert_eq!(invariants.len(), 1);
                        assert_eq!(body.len(), 1);
                        assert!(matches!(
                            core.algorithm_statements().get(body[0]).unwrap().kind,
                            CoreAlgorithmStmtKind::AssignLocal { .. }
                        ));
                        let CoreFormulaKind::Not(child) =
                            core.formulas().get(*condition).unwrap().kind
                        else {
                            panic!()
                        };
                        internal.insert(child);
                        let CoreFormulaKind::Equals { left, right } =
                            core.formulas().get(child).unwrap().kind
                        else {
                            panic!()
                        };
                        assert_eq!(
                            core.terms().get(left).unwrap().kind,
                            CoreTermKind::Var(binders[2].var)
                        );
                        assert_eq!(
                            core.terms().get(right).unwrap().kind,
                            CoreTermKind::Var(binders[0].var)
                        );
                        let CoreFormulaKind::Equals { left, right } =
                            core.formulas().get(invariants[0]).unwrap().kind
                        else {
                            panic!()
                        };
                        for term in [left, right] {
                            assert_eq!(
                                core.terms().get(term).unwrap().kind,
                                CoreTermKind::Var(binders[2].var)
                            );
                        }
                        trees.insert(*condition, b"\x04\x02\x02\0\0\0\x02\x02\0\0\0\0".to_vec());
                        trees.insert(invariants[0], b"\x02\x02\0\0\0\x02\x02\0\0\0\x02".to_vec());
                    }
                    CoreAlgorithmStmtKind::Pick {
                        binder,
                        witness_ty,
                        ghost,
                    } => {
                        assert!(!ghost);
                        assert_eq!(*witness_ty, binder.ty_guard);
                    }
                    CoreAlgorithmStmtKind::Snapshot { captures, .. } => {
                        assert_eq!(profile, "ghost");
                        assert_eq!(captures, &vec![binders[0].var, binders[2].var]);
                    }
                    _ => {}
                }
            }
            assert_eq!(
                core.obligation_seeds().len(),
                usize::from(profile == "choice")
            );
            for (_, seed) in core.obligation_seeds().iter() {
                assert_eq!(seed.owner, item_id);
                assert_eq!(seed.kind, ObligationSeedKind::GeneratedNonEmptiness);
                assert_eq!(seed.status, ObligationSeedStatus::Active);
                assert!(seed.context.is_empty() && seed.diagnostics.is_empty());
                let goal = seed.goal.unwrap();
                let CoreFormulaKind::Exists {
                    binders: logical,
                    body,
                } = &core.formulas().get(goal).unwrap().kind
                else {
                    panic!()
                };
                assert_eq!(logical.len(), 1);
                assert_eq!(logical[0].role.as_str(), "term-binder");
                assert!(logical[0].ty_guard.is_none());
                assert!(binders.iter().all(|b| b.var != logical[0].var));
                internal.insert(*body);
                let row = core.formulas().get(goal).unwrap();
                let normalized =
                    normalize_core_formula(&core, goal, &context, &row.source).unwrap();
                assert!(normalized.free_variables.is_empty());
                let canonical = canonical_formula(&normalized, &context, &row.source).unwrap();
                let F::Exists {
                    binders: entries,
                    body,
                } = canonical.kind()
                else {
                    panic!()
                };
                assert_eq!(entries.len(), 1);
                let F::TypePred { subject, ty } = body.kind() else {
                    panic!()
                };
                assert_eq!(ty.as_str(), "set");
                assert_eq!(subject.kind(), &T::Var(CanonicalVar::Bound(0)));
                trees.insert(
                    goal,
                    b"\x06\0\0\0\x01\0\0\0\x0bterm-binder\0\x03\0\0\0\x03set\x01\0\0\0\0".to_vec(),
                );
            }
            assert_eq!(
                trees.len(),
                match profile {
                    "state" | "while" => 6,
                    "ghost" => 4,
                    _ => 5,
                }
            );
            assert_eq!(core.formulas().len(), trees.len() + internal.len());
            assert!(internal.iter().all(|id| !trees.contains_key(id)));
            let flow = build_control_flow_ir(&core);
            assert_eq!(flow.flows.len(), 1);
            let (_, cfg) = flow.flows.iter().next().unwrap();
            assert!(!cfg.blocks.is_empty() && cfg.diagnostics.is_empty());
            let handoff = build_obligation_seed_handoff(&core, &flow);
            assert!(!handoff.entries.is_empty());
            assert!(handoff.entries.iter().any(|(_, row)| row.seed.kind
                == ObligationSeedKind::AlgorithmTermination
                && row.seed.status == ObligationSeedStatus::Deferred));
            for (_, row) in handoff.entries.iter() {
                if row.seed.kind == ObligationSeedKind::GeneratedNonEmptiness {
                    assert_eq!(row.seed.status, ObligationSeedStatus::Active);
                } else {
                    assert_eq!(row.seed.status, ObligationSeedStatus::Deferred);
                }
            }
            // These literal tree and slot vectors are independent of the production writer.
            let mut prefix = b"mizar-core/source-algorithm-formula/v1\0".to_vec();
            for field in [
                core.module_id().package().as_str(),
                core.module_id().path().as_str(),
                item.symbol.module().package().as_str(),
                item.symbol.module().path().as_str(),
                item.symbol.local().as_str(),
                item.symbol.fqn().as_str(),
                "public",
            ] {
                prefix.extend_from_slice(&u32::try_from(field.len()).unwrap().to_be_bytes());
                prefix.extend_from_slice(field.as_bytes());
            }
            prefix.extend_from_slice(match profile {
                "state" => b"\0\0\0\x04\x01\0\0\0\x09parameter\x02\0\0\0\x06result\x01\0\0\0\x09local:var\x01\0\0\0\x0blocal:const",
                "choice" => b"\0\0\0\x03\x01\0\0\0\x09parameter\x02\0\0\0\x06result\x02\0\0\0\x04pick",
                _ => b"\0\0\0\x03\x01\0\0\0\x09parameter\x02\0\0\0\x06result\x01\0\0\0\x09local:var",
            });
            let expected = trees
                .iter()
                .map(|(id, tree)| {
                    let mut value = prefix.clone();
                    value.extend_from_slice(tree);
                    (*id, value)
                })
                .collect::<BTreeMap<_, _>>();
            let roots = trees.keys().copied().collect::<Vec<_>>();
            let before = (
                core.clone(),
                typed.clone(),
                source.clone(),
                symbols.clone(),
                check.bindings().clone(),
                check.inference().clone(),
                check.snapshots().clone(),
                flow.clone(),
                handoff.clone(),
            );
            let payloads = source_algorithm_formula_payloads(&check, &core, &roots).unwrap();
            assert_eq!(payloads, expected);
            assert_eq!(
                source_algorithm_formula_payloads(&check, &core, &roots).unwrap(),
                expected
            );
            let reversed = roots.iter().rev().copied().collect::<Vec<_>>();
            assert_eq!(
                source_algorithm_formula_payloads(&check, &core, &reversed).unwrap(),
                expected
            );
            for root in &roots {
                assert_eq!(
                    source_algorithm_formula_payloads(&check, &core, &[*root]).unwrap(),
                    [(*root, expected[root].clone())].into_iter().collect()
                );
            }
            assert!(
                source_algorithm_formula_payloads(&check, &core, &[])
                    .unwrap()
                    .is_empty()
            );
            assert_eq!(
                before,
                (
                    core.clone(),
                    typed.clone(),
                    source.clone(),
                    symbols.clone(),
                    check.bindings().clone(),
                    check.inference().clone(),
                    check.snapshots().clone(),
                    build_control_flow_ir(&core),
                    build_obligation_seed_handoff(&core, &flow)
                )
            );
            if let Some((old_core, old_payloads)) = &baseline {
                assert_ne!(&core, old_core);
                if matches!(edit, 1 | 2 | 6) {
                    assert_eq!(&payloads, old_payloads);
                } else if edit == 3 {
                    for root in &roots {
                        assert_ne!(payloads[root], old_payloads[root]);
                    }
                } else {
                    for root in &roots {
                        if Some(*root) == assertion {
                            assert_ne!(payloads[root], old_payloads[root]);
                        } else {
                            assert_eq!(payloads[root], old_payloads[root]);
                        }
                    }
                }
            } else {
                baseline = Some((core, payloads));
            }
        }
    }
}

#[test]
fn source_algorithm_formula_payloads_preserve_refusals() {
    use mizar_checker::type_checker::check_source_algorithm_types;
    use mizar_core::{
        core_ir::*,
        elaborator::{lower_source_algorithms, source_algorithm_formula_payloads},
    };
    let config = config();
    let plan = crate::harness::build_test_plan(&config).unwrap();
    let mut genuine_cores = Vec::new();
    for id in [
        "pass_proof_verification_algorithm_while_invariant_001",
        "fail_proof_verification_algorithm_ensures_unprovable_001",
        "pass_proof_verification_claim_block_theorem_001",
    ] {
        let case = plan.cases.iter().find(|case| case.id.0 == id).unwrap();
        let text = fs::read_to_string(&case.source_path).unwrap();
        let (source, typed, symbols) = super::super::super::source_registration_inputs(
            &config.workspace_root,
            case,
            super::super::super::formula_statement::step5c8_test_frontend(&text),
        )
        .unwrap();
        let check = check_source_algorithm_types(&source, &typed, &symbols).unwrap();
        if id == "pass_proof_verification_claim_block_theorem_001" {
            assert!(
                check.inference().terms().is_empty() && check.inference().formulas().is_empty()
            );
            assert!(check.bindings().bindings().is_empty());
            let old_error = lower_source_algorithms(&check).unwrap_err();
            assert_eq!(old_error, "algorithms.core.unsupported_source");
            for other in &genuine_cores {
                assert_eq!(
                    source_algorithm_formula_payloads(&check, other, &[]).unwrap_err(),
                    old_error
                );
                assert_eq!(
                    source_algorithm_formula_payloads(
                        &check,
                        other,
                        &[CoreFormulaId::new(usize::MAX)]
                    )
                    .unwrap_err(),
                    old_error
                );
            }
            continue;
        }
        let core = lower_source_algorithms(&check).unwrap();
        let (_, algorithm) = core.algorithms().iter().next().unwrap();
        let guard = algorithm.params[0].ty_guard.unwrap();
        let invalid = "unsupported source algorithm formula payload";
        assert!(core.formulas().get(guard).is_some());
        let mut internal = Vec::new();
        for (_, statement) in core.algorithm_statements().iter() {
            if let CoreAlgorithmStmtKind::While { condition, .. } = statement.kind {
                let CoreFormulaKind::Not(child) = core.formulas().get(condition).unwrap().kind
                else {
                    panic!()
                };
                internal.push(child);
            }
        }
        for (_, seed) in core.obligation_seeds().iter() {
            let CoreFormulaKind::Exists { body, .. } =
                core.formulas().get(seed.goal.unwrap()).unwrap().kind
            else {
                panic!()
            };
            internal.push(body);
        }
        assert_eq!(internal.len(), 1);
        let before = (
            core.clone(),
            typed.clone(),
            source.clone(),
            symbols.clone(),
            check.bindings().clone(),
            check.inference().clone(),
            check.snapshots().clone(),
        );
        for roots in [
            vec![guard, guard],
            vec![CoreFormulaId::new(usize::MAX)],
            internal,
        ] {
            assert_eq!(
                source_algorithm_formula_payloads(&check, &core, &roots).unwrap_err(),
                invalid
            );
        }
        for other in &genuine_cores {
            assert_ne!(other, &core);
            assert_eq!(
                source_algorithm_formula_payloads(&check, other, &[guard]).unwrap_err(),
                invalid
            );
            assert_eq!(
                source_algorithm_formula_payloads(&check, other, &[]).unwrap_err(),
                invalid
            );
        }
        assert_eq!(
            before,
            (
                core.clone(),
                typed.clone(),
                source.clone(),
                symbols.clone(),
                check.bindings().clone(),
                check.inference().clone(),
                check.snapshots().clone()
            )
        );
        genuine_cores.push(core);
    }
}
