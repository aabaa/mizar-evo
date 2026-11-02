use super::*;

#[test]
fn source_algorithm_core_semantic_payload_preserves_actual_program() {
    use mizar_checker::type_checker::check_source_algorithm_types;
    use mizar_core::{
        binder_normalization::{
            BinderContext, CanonicalFormulaKind as F, CanonicalTermKind as T, CanonicalVar,
            NormalizedVarClass, NormalizedVarSort, canonical_formula, normalize_core_formula,
        },
        control_flow::{build_control_flow_ir, build_obligation_seed_handoff},
        core_ir::*,
        elaborator::{
            lower_source_algorithms, source_algorithm_core_semantic_payload,
            source_algorithm_formula_payloads,
        },
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
        for edit in 0..match profile {
            "state" => 8,
            "ghost" => 6,
            _ => 4,
        } {
            let text = match edit {
                0 => original.clone(),
                1 => original
                    .replace(" a ", " param ")
                    .replace("(a)", "(param)")
                    .replace(" a;", " param;")
                    .replace("= a\n", "= param\n")
                    .replace(" x", " local")
                    .replace(" g :=", " hidden :="),
                2 => format!(":: neutral algorithm trivia\n\n{original}"),
                3 => original.replace(owner, "renamed_algorithm"),
                4 if profile == "ghost" => original.replace("snapshot s0", "snapshot renamed"),
                5 if profile == "ghost" => original.replace("ghost var", "var"),
                4 => original.replace("assert x = a", "assert x = x"),
                5 => original.replace("return x;", "return c;"),
                6 => original.replace("x := c;", "x := a;"),
                7 => original.replace("x := c;\n    assert x = a;", "assert x = a;\n    x := c;"),
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
            for (_, statement) in core.algorithm_statements().iter() {
                match &statement.kind {
                    CoreAlgorithmStmtKind::Assert { formula } => {
                        let CoreFormulaKind::Equals { left, right } =
                            &core.formulas().get(*formula).unwrap().kind
                        else {
                            panic!()
                        };
                        let (l, r, expected) = match edit {
                            4 => (2, 2, b"\x02\x02\0\0\0\x02\x02\0\0\0\x02".as_slice()),
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
                assert!(
                    seed.context.is_empty() && seed.diagnostics.is_empty() && seed.label.is_none()
                );
                let pick = algorithm.statements[0];
                assert!(matches!(
                    core.algorithm_statements().get(pick).unwrap().kind,
                    CoreAlgorithmStmtKind::Pick { .. }
                ));
                let refs = vec![
                    CoreNodeRef::Item(item_id),
                    CoreNodeRef::Algorithm(algorithm_id),
                    CoreNodeRef::AlgorithmStmt(pick),
                    CoreNodeRef::Formula(seed.goal.unwrap()),
                ];
                assert_eq!(seed.core_refs, refs);
                assert_eq!(seed.local_path.as_str(), "algorithm/pick-nonempty");
                assert_eq!(seed.semantic_origin.as_str(), "algorithm/runtime-pick");
                let goal = seed.goal.unwrap();
                let CoreFormulaKind::Exists {
                    binders: logical, ..
                } = &core.formulas().get(goal).unwrap().kind
                else {
                    panic!()
                };
                assert_eq!(logical.len(), 1);
                assert_eq!(logical[0].role.as_str(), "term-binder");
                assert!(logical[0].ty_guard.is_none());
                assert!(binders.iter().all(|b| b.var != logical[0].var));
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
            let frame = |bytes: &[u8]| {
                let mut out = u32::try_from(bytes.len()).unwrap().to_be_bytes().to_vec();
                out.extend_from_slice(bytes);
                out
            };
            let mut expected_whole = b"mizar-core/source-algorithm-core-semantics/v1\0".to_vec();
            expected_whole.extend_from_slice(b"\0\0\0\x01");
            expected_whole.extend_from_slice(&frame(&expected[&binders[0].ty_guard.unwrap()]));
            expected_whole.extend_from_slice(&frame(&expected[&binders[1].ty_guard.unwrap()]));
            expected_whole.extend_from_slice(b"\0\0\0\x01");
            expected_whole.extend_from_slice(&frame(&expected[&ensures]));
            // These complete fixture programs are literal grammar expectations.
            let let_bytes = |slot: u8, ghost: bool| {
                let mut out = vec![2, 0, 0, 0, slot];
                out.extend_from_slice(&frame(
                    &expected[&binders[usize::from(slot)].ty_guard.unwrap()],
                ));
                out.extend_from_slice(&[u8::from(ghost), 1, 2, 0, 0, 0, 0]);
                out
            };
            match profile {
                "while" => {
                    expected_whole.extend_from_slice(b"\0\0\0\x03");
                    expected_whole.extend_from_slice(&let_bytes(2, false));
                    let CoreAlgorithmStmtKind::While {
                        condition,
                        invariants,
                        ..
                    } = &core
                        .algorithm_statements()
                        .get(algorithm.statements[1])
                        .unwrap()
                        .kind
                    else {
                        panic!()
                    };
                    expected_whole.push(5);
                    expected_whole.extend_from_slice(&frame(&expected[condition]));
                    expected_whole.extend_from_slice(b"\0\0\0\x01");
                    expected_whole.extend_from_slice(&frame(&expected[&invariants[0]]));
                    expected_whole.extend_from_slice(
                        b"\0\0\0\x01\x03\0\0\0\x02\x02\0\0\0\0\x06\x01\x02\0\0\0\x02",
                    );
                }
                "state" => {
                    expected_whole.extend_from_slice(b"\0\0\0\x05");
                    expected_whole.extend_from_slice(&let_bytes(2, false));
                    expected_whole.extend_from_slice(&let_bytes(3, false));
                    let assertion = core
                        .algorithm_statements()
                        .iter()
                        .find_map(|(_, row)| {
                            if let CoreAlgorithmStmtKind::Assert { formula } = row.kind {
                                Some(formula)
                            } else {
                                None
                            }
                        })
                        .unwrap();
                    let mut assert_bytes = vec![4];
                    assert_bytes.extend_from_slice(&frame(&expected[&assertion]));
                    let assignment = [3, 0, 0, 0, 2, 2, 0, 0, 0, if edit == 6 { 0 } else { 3 }];
                    if edit == 7 {
                        expected_whole.extend_from_slice(&assert_bytes);
                        expected_whole.extend_from_slice(&assignment);
                    } else {
                        expected_whole.extend_from_slice(&assignment);
                        expected_whole.extend_from_slice(&assert_bytes);
                    }
                    expected_whole.extend_from_slice(&[
                        6,
                        1,
                        2,
                        0,
                        0,
                        0,
                        if edit == 5 { 3 } else { 2 },
                    ]);
                }
                "ghost" => {
                    expected_whole.extend_from_slice(b"\0\0\0\x03");
                    expected_whole.extend_from_slice(&let_bytes(2, edit != 5));
                    expected_whole.push(1);
                    expected_whole.extend_from_slice(&frame(if edit == 4 {
                        b"renamed"
                    } else {
                        b"s0"
                    }));
                    expected_whole
                        .extend_from_slice(b"\0\0\0\x02\0\0\0\0\0\0\0\x02\x06\x01\x02\0\0\0\0");
                }
                "choice" => {
                    expected_whole.extend_from_slice(b"\0\0\0\x02\x08\0\0\0\x02");
                    let guard = binders[2].ty_guard.unwrap();
                    expected_whole.extend_from_slice(&frame(&expected[&guard]));
                    expected_whole.push(1);
                    expected_whole.extend_from_slice(&frame(&expected[&guard]));
                    expected_whole.extend_from_slice(b"\0\x06\x01\x04\0\0\0\x02");
                }
                _ => unreachable!(),
            }
            if profile == "choice" {
                let (_, seed) = core.obligation_seeds().iter().next().unwrap();
                expected_whole.extend_from_slice(b"\0\0\0\x01\0\0\0\0");
                expected_whole.extend_from_slice(&frame(&expected[&seed.goal.unwrap()]));
                expected_whole.extend_from_slice(&frame(b"algorithm/pick-nonempty"));
                expected_whole.extend_from_slice(&frame(b"algorithm/runtime-pick"));
            } else {
                expected_whole.extend_from_slice(b"\0\0\0\0");
            }
            let mut visited = Vec::new();
            let mut pending = algorithm
                .statements
                .iter()
                .rev()
                .copied()
                .collect::<Vec<_>>();
            let mut used_terms = BTreeSet::new();
            while let Some(id) = pending.pop() {
                assert!(!visited.contains(&id));
                visited.push(id);
                let row = core.algorithm_statements().get(id).unwrap();
                let position = visited.len() - 1;
                let term_slot = |id| {
                    let CoreTermKind::Var(var) = core.terms().get(id).unwrap().kind else {
                        panic!()
                    };
                    binders.iter().position(|binder| binder.var == var).unwrap()
                };
                match &row.kind {
                    CoreAlgorithmStmtKind::Let {
                        binder,
                        value,
                        ghost,
                    } => {
                        assert_eq!(binder.var, binders[if position == 1 { 3 } else { 2 }].var);
                        assert_eq!(*ghost, profile == "ghost" && edit != 5);
                        let id = value.unwrap();
                        assert_eq!(term_slot(id), 0);
                        used_terms.insert(id);
                    }
                    CoreAlgorithmStmtKind::AssignLocal { target, value } => {
                        assert_eq!(*target, binders[2].var);
                        assert_eq!(
                            term_slot(*value),
                            if profile == "state" && edit != 6 {
                                3
                            } else {
                                0
                            }
                        );
                        used_terms.insert(*value);
                        assert_eq!(
                            position,
                            if profile == "state" && edit == 7 {
                                3
                            } else {
                                2
                            }
                        );
                    }
                    CoreAlgorithmStmtKind::While { body, .. } => {
                        assert_eq!(position, 1);
                        pending.extend(body.iter().rev().copied());
                    }
                    CoreAlgorithmStmtKind::Return(value) => {
                        let id = value.unwrap();
                        let slot = match profile {
                            "ghost" => 0,
                            "state" if edit == 5 => 3,
                            _ => 2,
                        };
                        assert_eq!(term_slot(id), slot);
                        used_terms.insert(id);
                        assert_eq!(
                            position,
                            match profile {
                                "state" => 4,
                                "while" => 3,
                                "ghost" => 2,
                                _ => 1,
                            }
                        );
                    }
                    CoreAlgorithmStmtKind::Snapshot { name, .. } => {
                        assert_eq!(position, 1);
                        assert_eq!(name, if edit == 4 { "renamed" } else { "s0" });
                    }
                    CoreAlgorithmStmtKind::Pick { .. } => assert_eq!(position, 0),
                    CoreAlgorithmStmtKind::Assert { .. } => {
                        assert_eq!(position, if edit == 7 { 2 } else { 3 })
                    }
                    _ => panic!(),
                }
            }
            assert_eq!(visited.len(), core.algorithm_statements().len());
            assert_eq!(
                algorithm.statements.len(),
                match profile {
                    "state" => 5,
                    "choice" => 2,
                    _ => 3,
                }
            );
            assert_eq!(
                visited.len(),
                if profile == "while" {
                    4
                } else {
                    algorithm.statements.len()
                }
            );
            let mut all_formulas = BTreeSet::new();
            let mut pending = roots.clone();
            while let Some(id) = pending.pop() {
                if !all_formulas.insert(id) {
                    continue;
                }
                match &core.formulas().get(id).unwrap().kind {
                    CoreFormulaKind::Equals { left, right } => used_terms.extend([*left, *right]),
                    CoreFormulaKind::TypePred { subject, .. } => {
                        used_terms.insert(*subject);
                    }
                    CoreFormulaKind::Not(child) => pending.push(*child),
                    CoreFormulaKind::Exists { body, .. } => pending.push(*body),
                    _ => panic!(),
                }
            }
            assert_eq!(
                all_formulas,
                core.formulas().iter().map(|(id, _)| id).collect()
            );
            let unused = core
                .terms()
                .iter()
                .filter(|(id, _)| !used_terms.contains(id))
                .collect::<Vec<_>>();
            assert_eq!(unused.len(), 1);
            assert_eq!(unused[0].1.kind, CoreTermKind::Var(binders[0].var));
            assert_eq!(used_terms.len() + 1, core.terms().len());
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
            let bytes = source_algorithm_core_semantic_payload(&check, &core).unwrap();
            assert_eq!(bytes, expected_whole, "{profile} edit {edit}");
            assert_eq!(
                source_algorithm_core_semantic_payload(&check, &core).unwrap(),
                bytes
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
            if let Some((old_core, old_payloads, old_bytes)) = &baseline {
                assert_ne!(&core, old_core);
                if matches!(edit, 1 | 2) {
                    assert_eq!(&payloads, old_payloads);
                    assert_eq!(&bytes, old_bytes);
                } else {
                    assert_ne!(&bytes, old_bytes);
                    if (profile == "state" && matches!(edit, 5 | 6))
                        || profile == "ghost" && edit == 5
                    {
                        assert_eq!(&payloads, old_payloads);
                    }
                }
            } else {
                baseline = Some((core, payloads, bytes));
            }
        }
    }
}

#[test]
fn source_algorithm_core_semantic_payload_preserves_refusals() {
    use mizar_checker::type_checker::check_source_algorithm_types;
    use mizar_core::elaborator::{
        lower_source_algorithms, source_algorithm_core_semantic_payload,
        source_algorithm_formula_payloads,
    };
    let config = config();
    let plan = crate::harness::build_test_plan(&config).unwrap();
    let mut genuine_core = None;
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
        if id == "pass_proof_verification_algorithm_while_invariant_001" {
            genuine_core = Some(lower_source_algorithms(&check).unwrap());
            continue;
        }
        let other = genuine_core.as_ref().unwrap();
        let before = (
            other.clone(),
            source.clone(),
            typed.clone(),
            symbols.clone(),
            check.bindings().clone(),
            check.inference().clone(),
            check.snapshots().clone(),
        );
        let expected = if id == "pass_proof_verification_claim_block_theorem_001" {
            assert!(check.bindings().bindings().is_empty());
            assert!(
                check.inference().terms().is_empty() && check.inference().formulas().is_empty()
            );
            let old_error = lower_source_algorithms(&check).unwrap_err();
            assert_eq!(old_error, "algorithms.core.unsupported_source");
            old_error
        } else {
            let actual = lower_source_algorithms(&check).unwrap();
            assert_ne!(&actual, other);
            assert_ne!(
                actual.items().iter().next().unwrap().1.symbol,
                other.items().iter().next().unwrap().1.symbol
            );
            "unsupported source algorithm formula payload".to_owned()
        };
        assert_eq!(
            source_algorithm_formula_payloads(&check, other, &[]).unwrap_err(),
            expected
        );
        assert_eq!(
            source_algorithm_core_semantic_payload(&check, other).unwrap_err(),
            expected
        );
        assert_eq!(
            before,
            (
                other.clone(),
                source.clone(),
                typed.clone(),
                symbols.clone(),
                check.bindings().clone(),
                check.inference().clone(),
                check.snapshots().clone()
            )
        );
    }
}
