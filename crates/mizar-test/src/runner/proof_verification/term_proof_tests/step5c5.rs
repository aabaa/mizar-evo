use super::*;

#[test]
fn step5c5_equals_preserves_rhs_scopes_and_two_open_goals() {
    use mizar_core::core_ir::*;
    fn term(core: &CoreIr, id: CoreTermId) -> String {
        match &core.terms().get(id).unwrap().kind {
            CoreTermKind::Var(var) => format!("v{}", var.index()),
            CoreTermKind::Apply { functor, args } => {
                assert_eq!(functor, &core.definitions().iter().next().unwrap().1.symbol);
                format!(
                    "F({})",
                    args.iter()
                        .map(|id| term(core, *id))
                        .collect::<Vec<_>>()
                        .join(",")
                )
            }
            other => panic!("unexpected term {other:?}"),
        }
    }
    fn formula(core: &CoreIr, id: CoreFormulaId) -> String {
        match &core.formulas().get(id).unwrap().kind {
            CoreFormulaKind::Equals { left, right } => {
                format!("{}={}", term(core, *left), term(core, *right))
            }
            CoreFormulaKind::TypePred { subject, ty } => {
                format!("{}({})", ty.as_str(), term(core, *subject))
            }
            CoreFormulaKind::Forall { binders, body } => format!(
                "forall[{}]({})",
                binders
                    .iter()
                    .map(|b| format!("v{}:{}", b.var.index(), formula(core, b.ty_guard.unwrap())))
                    .collect::<Vec<_>>()
                    .join(","),
                formula(core, *body)
            ),
            other => panic!("unexpected formula {other:?}"),
        }
    }
    let config = config();
    let plan = crate::harness::build_test_plan(&config).unwrap();
    let case = plan
        .cases
        .iter()
        .find(|case| case.id.0 == STEP5C5_EQUALS_ID)
        .unwrap();
    let text = fs::read_to_string(&case.source_path).unwrap();
    let validate = |core: &CoreIr,
                    vcs: &VcSet,
                    source: &str,
                    rhs: usize,
                    swap_args: bool,
                    swap_equality: bool| {
        assert_eq!(
            (
                core.definitions().len(),
                core.proofs().len(),
                core.proof_nodes().len(),
                core.obligation_seeds().len()
            ),
            (1, 1, 3, 2)
        );
        let (definition_id, definition) = core.definitions().iter().next().unwrap();
        assert_eq!(definition.expansion, ExpansionPolicy::Opaque);
        assert_eq!(
            definition
                .params
                .iter()
                .map(|b| b.var.index())
                .collect::<Vec<_>>(),
            [4, 5]
        );
        assert_eq!(
            definition
                .params
                .iter()
                .map(|b| formula(core, b.ty_guard.unwrap()))
                .collect::<Vec<_>>(),
            ["set(v4)", "set(v5)"]
        );
        let DefinitionBody::Term(body) = definition.body else {
            panic!("actual RHS term");
        };
        assert_eq!(term(core, body), format!("v{rhs}"));
        assert_eq!(definition.correctness.len(), 1);
        let seed_id = definition.correctness[0];
        let seed = core.obligation_seeds().get(seed_id).unwrap();
        assert_eq!(seed.kind, ObligationSeedKind::DefinitionCorrectness);
        assert_eq!(seed.status, ObligationSeedStatus::Active);
        assert_eq!(seed.owner, definition.owner.anchor_item());
        assert!(seed.context.is_empty());
        assert_eq!(
            formula(core, seed.goal.unwrap()),
            format!("forall[v4:set(v4),v5:set(v5)](set(v{rhs}))")
        );
        assert!(
            seed.core_refs
                .contains(&CoreNodeRef::Definition(definition_id))
        );
        assert!(seed.core_refs.contains(&CoreNodeRef::Term(body)));
        assert!(
            seed.core_refs
                .contains(&CoreNodeRef::Formula(seed.goal.unwrap()))
        );
        let CoreFormulaKind::Forall {
            binders,
            body: consequent,
        } = &core.formulas().get(seed.goal.unwrap()).unwrap().kind
        else {
            unreachable!()
        };
        for id in binders
            .iter()
            .map(|binder| binder.ty_guard.unwrap())
            .chain(std::iter::once(*consequent))
        {
            assert!(seed.core_refs.contains(&CoreNodeRef::Formula(id)));
            let CoreFormulaKind::TypePred { subject, .. } = core.formulas().get(id).unwrap().kind
            else {
                unreachable!()
            };
            assert!(seed.core_refs.contains(&CoreNodeRef::Term(subject)));
        }
        let CoreSourceAnchor::SourceRange(result_span) =
            core.formulas().get(*consequent).unwrap().source.anchor
        else {
            panic!("declared result source");
        };
        assert_eq!(&source[result_span.start..result_span.end], "set");
        let CoreSourceAnchor::SourceRange(rhs_span) = core.terms().get(body).unwrap().source.anchor
        else {
            panic!("RHS source");
        };
        assert_eq!(
            &source[rhs_span.start..rhs_span.end],
            definition.params[rhs - 4].source_name.as_deref().unwrap()
        );
        let CoreSourceAnchor::SourceRange(span) = seed.source.anchor else {
            panic!("coherence source");
        };
        assert_eq!(&source[span.start..span.end], "coherence;");
        let (_, proof) = core.proofs().iter().next().unwrap();
        assert_eq!(proof.status, CoreProofStatus::PendingAutomaticProof);
        let goal = |first: usize, second: usize| {
            let call = if swap_args {
                format!("F(v{second},v{first})")
            } else {
                format!("F(v{first},v{second})")
            };
            if swap_equality {
                format!("v{first}={call}")
            } else {
                format!("{call}=v{first}")
            }
        };
        assert_eq!(
            formula(core, proof.proposition),
            format!("forall[v0:set(v0),v1:set(v1)]({})", goal(0, 1))
        );
        let mut current = proof.root;
        let mut guards = Vec::new();
        for expected in [2, 3] {
            let CoreProofNodeKind::IntroduceBinder { binder, child } =
                &core.proof_nodes().get(current).unwrap().kind
            else {
                panic!("local binder");
            };
            assert_eq!(binder.var.index(), expected);
            assert_eq!(
                formula(core, binder.ty_guard.unwrap()),
                format!("set(v{expected})")
            );
            guards.push(binder.ty_guard.unwrap());
            current = *child;
        }
        let CoreProofNodeKind::TerminalGoal {
            obligation,
            citations,
        } = &core.proof_nodes().get(current).unwrap().kind
        else {
            panic!("terminal");
        };
        assert!(citations.is_empty());
        let terminal = core.obligation_seeds().get(*obligation).unwrap();
        assert_eq!(terminal.kind, ObligationSeedKind::TheoremProof);
        assert_eq!(terminal.status, ObligationSeedStatus::Active);
        assert_eq!(terminal.owner, proof.item);
        assert_eq!(terminal.context, guards);
        assert_eq!(formula(core, terminal.goal.unwrap()), goal(2, 3));
        assert_ne!(terminal.local_path, seed.local_path);
        assert_eq!(
            core.terms()
                .iter()
                .filter_map(|(_, row)| if let CoreTermKind::Var(var) = row.kind {
                    Some(var.index())
                } else {
                    None
                })
                .collect::<std::collections::BTreeSet<_>>(),
            (0..6).collect()
        );
        for (id, row) in core.terms().iter() {
            assert_eq!(core.source_map().term_sources.get(&id), Some(&row.source));
        }
        for (id, row) in core.formulas().iter() {
            assert_eq!(
                core.source_map().formula_sources.get(&id),
                Some(&row.source)
            );
        }
        assert_eq!(vcs.vcs().len(), 2);
        assert!(
            vcs.vcs()
                .iter()
                .all(|vc| vc.status == mizar_vc::vc_ir::VcStatus::Open)
        );
        for (id, seed) in core.obligation_seeds().iter() {
            assert_eq!(
                core.source_map().obligation_sources.get(&id),
                Some(&seed.source)
            );
            let vc = vcs
                .vcs()
                .iter()
                .find(|vc| vc.goal == mizar_vc::vc_ir::VcFormulaRef::Core(seed.goal.unwrap()))
                .unwrap();
            assert_eq!(
                vc.kind,
                if seed.kind == ObligationSeedKind::DefinitionCorrectness {
                    mizar_vc::vc_ir::VcKind::DefinitionCorrectness
                } else {
                    mizar_vc::vc_ir::VcKind::TheoremProofStep
                }
            );
        }
    };
    let sources = [
        (text.clone(), 4, false, false),
        (text.replace("equals X", "equals Y"), 5, false, false),
        (
            text.replace("FirstOfDef", "DifferentDefinition")
                .replace("FirstOf1", "DifferentTheorem")
                .replace("+", "*"),
            4,
            false,
            false,
        ),
        (
            text.replace("let X, Y be set;\n  func", "let A, B be set;\n  func")
                .replace("X \\+\\ Y -> set equals X", "A \\+\\ B -> set equals A"),
            4,
            false,
            false,
        ),
        (
            text.replace(
                "for X, Y being set holds X \\+\\ Y = X",
                "for C, D being set holds C \\+\\ D = C",
            ),
            4,
            false,
            false,
        ),
        (
            text.replace(
                "proof\n  let X, Y be set;\n  thus X \\+\\ Y = X",
                "proof\n  let E, F be set;\n  thus E \\+\\ F = E",
            ),
            4,
            false,
            false,
        ),
        (
            text.replace("holds X \\+\\ Y = X", "holds Y \\+\\ X = X")
                .replace("thus X \\+\\ Y = X", "thus Y \\+\\ X = X"),
            4,
            true,
            false,
        ),
        (
            text.replace("holds X \\+\\ Y = X", "holds X = X \\+\\ Y")
                .replace("thus X \\+\\ Y = X", "thus X = X \\+\\ Y"),
            4,
            false,
            true,
        ),
    ];
    for (source, rhs, swap_args, swap_equality) in sources {
        let build = || {
            phrase_theorem_output(
                &config.workspace_root,
                case,
                super::super::super::formula_statement::step5c8_test_frontend(&source),
            )
            .unwrap()
        };
        let (core, vcs) = build();
        assert_eq!((core.clone(), vcs.clone()), build());
        validate(&core, &vcs, &source, rhs, swap_args, swap_equality);
        if source == text {
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
                let (definition_id, definition) = core.definitions().iter().next().unwrap();
                let seed = definition.correctness[0];
                let goal = core.obligation_seeds().get(seed).unwrap().goal.unwrap();
                let CoreFormulaKind::Forall {
                    body: consequent, ..
                } = core.formulas().get(goal).unwrap().kind
                else {
                    unreachable!()
                };
                let (call_id, _) = core
                    .terms()
                    .iter()
                    .find(|(_, row)| matches!(row.kind, CoreTermKind::Apply { .. }))
                    .unwrap();
                match mutation {
                    0 => {
                        parts.definitions.get_mut(definition_id).unwrap().expansion =
                            ExpansionPolicy::Transparent
                    }
                    1 => parts
                        .definitions
                        .get_mut(definition_id)
                        .unwrap()
                        .correctness
                        .clear(),
                    2 => parts
                        .definitions
                        .get_mut(definition_id)
                        .unwrap()
                        .correctness
                        .push(seed),
                    3 => {
                        parts.definitions.get_mut(definition_id).unwrap().params[1].ty_guard = None
                    }
                    4 => {
                        let CoreFormulaKind::TypePred { subject, .. } =
                            &mut parts.formulas.get_mut(consequent).unwrap().kind
                        else {
                            unreachable!()
                        };
                        *subject = call_id;
                    }
                    5 => {
                        parts.definitions.get_mut(definition_id).unwrap().params[1].var =
                            definition.params[0].var
                    }
                    6 => {
                        let CoreTermKind::Apply { args, .. } =
                            &mut parts.terms.get_mut(call_id).unwrap().kind
                        else {
                            unreachable!()
                        };
                        args.swap(0, 1);
                    }
                    7 => {
                        let (terminal, _) = core
                            .obligation_seeds()
                            .iter()
                            .find(|(_, row)| row.kind == ObligationSeedKind::TheoremProof)
                            .unwrap();
                        parts
                            .obligation_seeds
                            .get_mut(terminal)
                            .unwrap()
                            .context
                            .push(goal);
                    }
                    _ => unreachable!(),
                }
                // Detached Core validation is structural; source agreement is checked by the graph oracle.
                let hostile = CoreIr::try_new(parts).unwrap_or_else(|error| {
                    panic!("structurally valid graph mutation {mutation}: {error:?}")
                });
                let changed_vcs =
                    generate_core_vcs(&hostile, snapshot_id(0)).unwrap_or_else(|error| {
                        panic!("generatable graph mutation {mutation}: {error}")
                    });
                assert!(
                    std::panic::catch_unwind(|| validate(
                        &hostile,
                        &changed_vcs,
                        &source,
                        rhs,
                        swap_args,
                        swap_equality
                    ))
                    .is_err(),
                    "undetected graph mutation {mutation}"
                );
            }
        }
        if source == text {
            let (_, actual) = phrase_theorem_output(
                &config.workspace_root,
                case,
                run_frontend(&config.workspace_root, case, 0).unwrap(),
            )
            .unwrap();
            assert_eq!(fs::read_to_string(config.workspace_root.join("tests/snapshots/vc/pass_proof_verification_func_equals_infix_operator_001.vc_ir.snap")).unwrap(), actual.debug_text());
        }
    }
}

#[test]
fn step5c5_equals_authenticates_infix_association_payload() {
    use mizar_checker::type_checker::TermFormulaChecker;
    use mizar_syntax::ast::{
        SurfaceAstBuilder, SurfaceNodeKind as K, SurfaceOperatorAssociativity,
    };
    let config = config();
    let plan = crate::harness::build_test_plan(&config).unwrap();
    let case = plan
        .cases
        .iter()
        .find(|case| case.id.0 == STEP5C5_EQUALS_ID)
        .unwrap();
    let text = fs::read_to_string(&case.source_path).unwrap();
    let ast = super::super::super::formula_statement::step5c8_test_frontend(&text)
        .ast
        .unwrap();
    let targets = ast
        .nodes()
        .iter()
        .enumerate()
        .filter_map(|(index, node)| matches!(node.kind, K::InfixExpression(_)).then_some(index))
        .collect::<Vec<_>>();
    assert_eq!(targets.len(), 2);
    for target in targets {
        for association in [
            SurfaceOperatorAssociativity::Right,
            SurfaceOperatorAssociativity::NonAssociative,
        ] {
            let mut builder = SurfaceAstBuilder::new(ast.source_id);
            let mut rebuilt = Vec::new();
            for (index, node) in ast.nodes().iter().enumerate() {
                let children = node.children.iter().map(|id| rebuilt[id.index()]).collect();
                let mut kind = node.kind.clone();
                if index == target {
                    let K::InfixExpression(operator) = &mut kind else {
                        unreachable!()
                    };
                    assert_eq!(operator.associativity, SurfaceOperatorAssociativity::Left);
                    operator.associativity = association;
                }
                rebuilt.push(match kind {
                    K::Token(token) => builder.add_token(token.kind, token.text, node.range),
                    kind => builder.add_node(kind, node.range, children),
                });
            }
            let changed = builder.finish(Some(rebuilt[ast.root().unwrap().index()]), None);
            let mut frontend = super::super::super::formula_statement::step5c8_test_frontend(&text);
            assert!(frontend.diagnostics.is_empty());
            frontend.ast = Some(changed);
            // Both source and neutral receipts share the hostile payload, so equality cannot reject it.
            let (source, nodes, symbols) = super::super::super::source_registration_inputs(
                &config.workspace_root,
                case,
                frontend,
            )
            .unwrap();
            mizar_resolve::symbols::validate_source_symbol_env(&source, &symbols).unwrap();
            assert_eq!(
                TermFormulaChecker::check_source_predicate_statements(&source, &symbols, &nodes)
                    .unwrap_err(),
                "predicates.statement.unsupported_source_types"
            );
        }
    }
}

#[test]
fn step5c5_equals_requires_exact_source_types_clause_and_operator() {
    let config = config();
    let plan = crate::harness::build_test_plan(&config).unwrap();
    let case = plan
        .cases
        .iter()
        .find(|case| case.id.0 == STEP5C5_EQUALS_ID)
        .unwrap();
    let text = fs::read_to_string(&case.source_path).unwrap();
    for (index, changed) in [
        text.replace("X \\+\\ Y ->", "Y \\+\\ X ->"),
        text.replace("equals X", "equals Unknown"),
        text.replace("equals X", "equals it"),
        text.replace("-> set", "-> object"),
        text.replace("let X, Y be set;\n  func", "let X, Y be object;\n  func"),
        text.replace("for X, Y being set", "for X, Y being object"),
        text.replace("proof\n  let X, Y be set", "proof\n  let X, Y be object"),
        text.replace("holds X \\+\\ Y", "holds Unknown \\+\\ Y"),
        text.replace("thus X \\+\\ Y", "thus X \\+\\ Unknown"),
        text.replace("let X, Y be set;\n  func", "let X, Y, Z be set;\n  func"),
        text.replace("for X, Y being set", "for X, Y, Z being set"),
        text.replace("  coherence;\n", ""),
        text.replace("  coherence;", "  coherence; coherence;"),
        text.replace("  coherence;", "  existence;"),
        text.replace("  coherence;", "  coherence proof thus X = X; end;"),
        text.replace("  coherence;", "  symmetry; coherence;"),
        text.replace("func FirstOfDef", "redefine func FirstOfDef"),
        text.replace(r#""\\+\\""#, r#""\\*\\""#),
        text.replace("left, 80", "right, 80"),
        text.replace("left, 80", "left, 90"),
        text.replace("left, 80", "left, \"80\""),
        text.replace("infix_operator", "prefix_operator"),
        text.lines()
            .filter(|line| !line.starts_with("infix_operator"))
            .collect::<Vec<_>>()
            .join("\n"),
        text.replace(
            "\ntheorem",
            "\ninfix_operator(\"\\\\+\\\\\", left, 80);\ntheorem",
        ),
        text.replace("holds X \\+\\ Y = X", "holds X \\+\\ Y = X & X = X"),
        text.replace("thus X \\+\\ Y = X;", "thus X \\+\\ Y = X by FirstOfDef;"),
    ]
    .into_iter()
    .enumerate()
    {
        assert_ne!(changed, text, "unchanged control {index}");
        assert!(
            phrase_theorem_output(
                &config.workspace_root,
                case,
                super::super::super::formula_statement::step5c8_test_frontend(&changed)
            )
            .is_err(),
            "accepted source mutation {index}"
        );
    }
}

#[test]
fn step5c5_means_preserves_opaque_body_and_three_open_goals() {
    use mizar_core::core_ir::*;
    fn term(core: &CoreIr, id: CoreTermId) -> String {
        match &core.terms().get(id).unwrap().kind {
            CoreTermKind::Var(var) => format!("v{}", var.index()),
            CoreTermKind::Apply { functor, args } => {
                assert_eq!(functor, &core.definitions().iter().next().unwrap().1.symbol);
                format!(
                    "F({})",
                    args.iter()
                        .map(|id| term(core, *id))
                        .collect::<Vec<_>>()
                        .join(",")
                )
            }
            other => panic!("unexpected term {other:?}"),
        }
    }
    fn formula(core: &CoreIr, id: CoreFormulaId) -> String {
        match &core.formulas().get(id).unwrap().kind {
            CoreFormulaKind::Equals { left, right } => {
                format!("{}={}", term(core, *left), term(core, *right))
            }
            CoreFormulaKind::TypePred { subject, ty } => {
                format!("{}({})", ty.as_str(), term(core, *subject))
            }
            CoreFormulaKind::And(items) => format!(
                "and({})",
                items
                    .iter()
                    .map(|id| formula(core, *id))
                    .collect::<Vec<_>>()
                    .join(",")
            ),
            CoreFormulaKind::Implies {
                premise,
                conclusion,
            } => format!(
                "implies({},{})",
                formula(core, *premise),
                formula(core, *conclusion)
            ),
            CoreFormulaKind::Forall { binders, body }
            | CoreFormulaKind::Exists { binders, body } => {
                let quantifier = if matches!(
                    core.formulas().get(id).unwrap().kind,
                    CoreFormulaKind::Forall { .. }
                ) {
                    "forall"
                } else {
                    "exists"
                };
                format!(
                    "{quantifier}[{}]({})",
                    binders
                        .iter()
                        .map(|binder| format!(
                            "v{}:{}",
                            binder.var.index(),
                            formula(core, binder.ty_guard.unwrap())
                        ))
                        .collect::<Vec<_>>()
                        .join(","),
                    formula(core, *body)
                )
            }
            other => panic!("unexpected formula {other:?}"),
        }
    }
    fn backrefs(core: &CoreIr, seed: &ObligationSeed, root: CoreFormulaId) {
        let mut pending = vec![CoreNodeRef::Formula(root)];
        let mut visited = std::collections::BTreeSet::new();
        while let Some(reference) = pending.pop() {
            if !visited.insert(reference.clone()) {
                continue;
            }
            assert!(seed.core_refs.contains(&reference), "missing {reference:?}");
            match reference {
                CoreNodeRef::Term(id) => {
                    if let CoreTermKind::Apply { args, .. } = &core.terms().get(id).unwrap().kind {
                        pending.extend(args.iter().copied().map(CoreNodeRef::Term));
                    }
                }
                CoreNodeRef::Formula(id) => match &core.formulas().get(id).unwrap().kind {
                    CoreFormulaKind::Equals { left, right } => {
                        pending.extend([CoreNodeRef::Term(*left), CoreNodeRef::Term(*right)])
                    }
                    CoreFormulaKind::TypePred { subject, .. } => {
                        pending.push(CoreNodeRef::Term(*subject))
                    }
                    CoreFormulaKind::And(items) => {
                        pending.extend(items.iter().copied().map(CoreNodeRef::Formula))
                    }
                    CoreFormulaKind::Implies {
                        premise,
                        conclusion,
                    } => pending.extend([
                        CoreNodeRef::Formula(*premise),
                        CoreNodeRef::Formula(*conclusion),
                    ]),
                    CoreFormulaKind::Forall { binders, body }
                    | CoreFormulaKind::Exists { binders, body } => {
                        pending.push(CoreNodeRef::Formula(*body));
                        pending.extend(
                            binders
                                .iter()
                                .filter_map(|binder| binder.ty_guard)
                                .map(CoreNodeRef::Formula),
                        );
                    }
                    other => panic!("unexpected {other:?}"),
                },
                _ => unreachable!(),
            }
        }
    }
    let config = config();
    let plan = crate::harness::build_test_plan(&config).unwrap();
    let case = plan
        .cases
        .iter()
        .find(|case| case.id.0 == STEP5C5_MEANS_ID)
        .unwrap();
    let text = fs::read_to_string(&case.source_path).unwrap();
    let validate =
        |core: &CoreIr, vcs: &VcSet, source: &str, swapped_body: bool, swapped_goal: bool| {
            assert_eq!(core.definitions().len(), 1);
            assert_eq!(core.proofs().len(), 1);
            assert_eq!(core.proof_nodes().len(), 2);
            assert_eq!(core.obligation_seeds().len(), 3);
            let (definition_id, definition) = core.definitions().iter().next().unwrap();
            assert_eq!(definition.expansion, ExpansionPolicy::Opaque);
            assert_eq!(definition.params.len(), 1);
            assert_eq!(definition.correctness.len(), 2);
            let DefinitionBody::Formula(body) = definition.body else {
                panic!("formula body")
            };
            assert_eq!(
                formula(core, body),
                if swapped_body { "v2=F(v2)" } else { "F(v2)=v2" }
            );
            assert_eq!(
                formula(core, definition.params[0].ty_guard.unwrap()),
                "set(v2)"
            );
            let relation = |result: usize| {
                if swapped_body {
                    format!("v2=v{result}")
                } else {
                    format!("v{result}=v2")
                }
            };
            let expected = [
                format!("forall[v2:set(v2)](exists[v3:set(v3)]({}))", relation(3)),
                format!(
                    "forall[v2:set(v2)](forall[v4:set(v4),v5:set(v5)](implies(and({},{}),v4=v5)))",
                    relation(4),
                    relation(5)
                ),
            ];
            for (index, seed_id) in definition.correctness.iter().enumerate() {
                let seed = core.obligation_seeds().get(*seed_id).unwrap();
                assert_eq!(seed.kind, ObligationSeedKind::DefinitionCorrectness);
                assert_eq!(seed.status, ObligationSeedStatus::Active);
                assert_eq!(seed.owner, definition.owner.anchor_item());
                assert!(seed.context.is_empty());
                assert_eq!(formula(core, seed.goal.unwrap()), expected[index]);
                assert!(
                    seed.core_refs
                        .contains(&CoreNodeRef::Definition(definition_id))
                );
                assert!(seed.core_refs.contains(&CoreNodeRef::Formula(body)));
                assert!(
                    seed.core_refs
                        .contains(&CoreNodeRef::Formula(seed.goal.unwrap()))
                );
                backrefs(core, seed, body);
                backrefs(core, seed, seed.goal.unwrap());
                assert!(!seed.source.provenance.is_empty());
                let CoreSourceAnchor::SourceRange(span) = seed.source.anchor else {
                    panic!("source clause")
                };
                assert_eq!(
                    &source[span.start..span.end],
                    ["existence;", "uniqueness;"][index]
                );
                assert_eq!(
                    core.source_map().obligation_sources.get(seed_id),
                    Some(&seed.source)
                );
            }
            assert_ne!(
                core.obligation_seeds()
                    .get(definition.correctness[0])
                    .unwrap()
                    .local_path,
                core.obligation_seeds()
                    .get(definition.correctness[1])
                    .unwrap()
                    .local_path
            );
            let (_, proof) = core.proofs().iter().next().unwrap();
            assert_eq!(proof.status, CoreProofStatus::PendingAutomaticProof);
            assert_eq!(
                formula(core, proof.proposition),
                if swapped_goal {
                    "forall[v0:set(v0)](v0=F(v0))"
                } else {
                    "forall[v0:set(v0)](F(v0)=v0)"
                }
            );
            let CoreProofNodeKind::IntroduceBinder { binder, child } =
                &core.proof_nodes().get(proof.root).unwrap().kind
            else {
                panic!("local binder")
            };
            let CoreProofNodeKind::TerminalGoal {
                obligation,
                citations,
            } = &core.proof_nodes().get(*child).unwrap().kind
            else {
                panic!("terminal")
            };
            assert!(citations.is_empty());
            let seed = core.obligation_seeds().get(*obligation).unwrap();
            assert_eq!(seed.kind, ObligationSeedKind::TheoremProof);
            assert_eq!(seed.status, ObligationSeedStatus::Active);
            assert_eq!(seed.owner, proof.item);
            assert_eq!(seed.context, [binder.ty_guard.unwrap()]);
            assert_eq!(formula(core, seed.context[0]), "set(v1)");
            assert_eq!(
                formula(core, seed.goal.unwrap()),
                if swapped_goal { "v1=F(v1)" } else { "F(v1)=v1" }
            );
            let variables = core
                .terms()
                .iter()
                .filter_map(|(_, row)| {
                    if let CoreTermKind::Var(var) = row.kind {
                        Some(var)
                    } else {
                        None
                    }
                })
                .collect::<std::collections::BTreeSet<_>>();
            assert_eq!(variables, (0..6).map(CoreVarId::new).collect());
            for (id, row) in core.terms().iter() {
                assert_eq!(core.source_map().term_sources.get(&id), Some(&row.source));
                if matches!(row.kind, CoreTermKind::Var(var) if var.index() >= 3) {
                    let CoreSourceAnchor::SourceRange(span) = row.source.anchor else {
                        panic!("It origin")
                    };
                    assert_eq!(&source[span.start..span.end], "it");
                    assert!(
                        row.source
                            .provenance
                            .iter()
                            .any(|provenance| provenance.phase == CoreProvenancePhase::Generated)
                    );
                }
            }
            for (id, row) in core.formulas().iter() {
                assert_eq!(
                    core.source_map().formula_sources.get(&id),
                    Some(&row.source)
                );
            }

            assert_eq!(vcs.vcs().len(), 3);
            assert_eq!(
                vcs.vcs()
                    .iter()
                    .filter(|vc| vc.kind == mizar_vc::vc_ir::VcKind::DefinitionCorrectness)
                    .count(),
                2
            );
            assert_eq!(
                vcs.vcs()
                    .iter()
                    .filter(|vc| vc.kind == mizar_vc::vc_ir::VcKind::TheoremProofStep)
                    .count(),
                1
            );
            assert!(
                vcs.vcs()
                    .iter()
                    .all(|vc| vc.status == mizar_vc::vc_ir::VcStatus::Open)
            );
            for (_, seed) in core.obligation_seeds().iter() {
                assert_eq!(
                    vcs.vcs()
                        .iter()
                        .filter(
                            |vc| vc.goal == mizar_vc::vc_ir::VcFormulaRef::Core(seed.goal.unwrap())
                        )
                        .count(),
                    1
                );
            }
        };
    let sources = [
        (text.clone(), false, false),
        (text.replace("means it = X", "means X = it"), true, false),
        (
            text.replace("holds choicebox X = X", "holds X = choicebox X")
                .replace("thus choicebox X = X", "thus X = choicebox X"),
            false,
            true,
        ),
        (
            text.replace("choicebox", "parcel")
                .replace("ChoiceBoxDef", "ParcelDef")
                .replace("ChoiceBox1", "Parcel1"),
            false,
            false,
        ),
        (
            text.replace("let X be set;\n  func", "let Formal be set;\n  func")
                .replace(
                    "choicebox X -> set means it = X",
                    "choicebox Formal -> set means it = Formal",
                ),
            false,
            false,
        ),
        (
            text.replace(
                "for X being set holds choicebox X = X",
                "for Header being set holds choicebox Header = Header",
            ),
            false,
            false,
        ),
        (
            text.replace(
                "proof\n  let X be set;\n  thus choicebox X = X",
                "proof\n  let Local be set;\n  thus choicebox Local = Local",
            ),
            false,
            false,
        ),
    ];
    for (source, swapped_body, swapped_goal) in sources {
        let build = || {
            phrase_theorem_output(
                &config.workspace_root,
                case,
                super::super::super::formula_statement::step5c8_test_frontend(&source),
            )
            .unwrap()
        };
        let (core, vcs) = build();
        assert_eq!((core.clone(), vcs.clone()), build());
        validate(&core, &vcs, &source, swapped_body, swapped_goal);
        if source == text {
            for mutation in 0..7 {
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
                let (definition_id, definition) = core.definitions().iter().next().unwrap();
                let DefinitionBody::Formula(body) = definition.body else {
                    unreachable!()
                };
                match mutation {
                    0 => {
                        parts.definitions.get_mut(definition_id).unwrap().expansion =
                            ExpansionPolicy::Transparent
                    }
                    1 | 2 => {
                        let removed = definition.correctness[mutation - 1];
                        let remap = |id: ObligationSeedId| {
                            ObligationSeedId::new(
                                id.index() - usize::from(id.index() > removed.index()),
                            )
                        };
                        parts.obligation_seeds = ObligationSeedTable::new();
                        for (id, row) in core.obligation_seeds().iter() {
                            if id == removed {
                                continue;
                            }
                            let mut row = row.clone();
                            row.core_refs.retain(|reference| {
                                *reference != CoreNodeRef::ObligationSeed(removed)
                            });
                            for reference in &mut row.core_refs {
                                if let CoreNodeRef::ObligationSeed(id) = reference {
                                    *id = remap(*id);
                                }
                            }
                            parts.obligation_seeds.insert(row);
                        }
                        let owned = &mut parts
                            .definitions
                            .get_mut(definition_id)
                            .unwrap()
                            .correctness;
                        owned.retain(|id| *id != removed);
                        for id in owned {
                            *id = remap(*id);
                        }
                        for (_, row) in parts.proof_nodes.iter_mut() {
                            if let CoreProofNodeKind::TerminalGoal { obligation, .. } =
                                &mut row.kind
                            {
                                *obligation = remap(*obligation);
                            }
                        }
                        parts.source_map.obligation_sources = parts
                            .obligation_seeds
                            .iter()
                            .map(|(id, row)| (id, row.source.clone()))
                            .collect();
                    }
                    3 => {
                        let (_, seed) = parts
                            .obligation_seeds
                            .iter_mut()
                            .find(|(_, row)| row.kind == ObligationSeedKind::TheoremProof)
                            .unwrap();
                        seed.context.push(body);
                        seed.core_refs.push(CoreNodeRef::Formula(body));
                    }
                    4 | 5 => {
                        let from = CoreVarId::new(if mutation == 4 { 3 } else { 1 });
                        let into = CoreVarId::new(2);
                        for (_, row) in parts.terms.iter_mut() {
                            if row.kind == CoreTermKind::Var(from) {
                                row.kind = CoreTermKind::Var(into);
                            }
                        }
                        for (_, row) in parts.formulas.iter_mut() {
                            if let CoreFormulaKind::Forall { binders, .. }
                            | CoreFormulaKind::Exists { binders, .. } = &mut row.kind
                            {
                                for binder in binders {
                                    if binder.var == from {
                                        binder.var = into;
                                    }
                                }
                            }
                        }
                        for (_, row) in parts.proof_nodes.iter_mut() {
                            if let CoreProofNodeKind::IntroduceBinder { binder, .. } = &mut row.kind
                                && binder.var == from
                            {
                                binder.var = into;
                            }
                        }
                    }
                    6 => {
                        let CoreFormulaKind::Equals { left, right } =
                            &mut parts.formulas.get_mut(body).unwrap().kind
                        else {
                            unreachable!()
                        };
                        std::mem::swap(left, right);
                    }
                    _ => unreachable!(),
                }
                let hostile = CoreIr::try_new(parts)
                    .unwrap_or_else(|error| panic!("structural mutation {mutation}: {error}"));
                let hostile_vcs = generate_core_vcs(&hostile, snapshot_id(0)).unwrap();
                // Detached Core validation is structural; this source contract must still detect the forgery.
                assert!(
                    std::panic::catch_unwind(|| validate(
                        &hostile,
                        &hostile_vcs,
                        &source,
                        false,
                        false
                    ))
                    .is_err(),
                    "undetected mutation {mutation}"
                );
            }
        }
        if source == text {
            let (_, actual) = phrase_theorem_output(
                &config.workspace_root,
                case,
                run_frontend(&config.workspace_root, case, 0).unwrap(),
            )
            .unwrap();
            assert_eq!(
                fs::read_to_string(config.workspace_root.join(
                    "tests/snapshots/vc/pass_proof_verification_func_means_prefix_001.vc_ir.snap"
                ))
                .unwrap(),
                actual.debug_text()
            );
        }
    }
}

#[test]
fn step5c5_means_and_equals_authenticate_application_and_clause_receipts() {
    use mizar_checker::{source_application::*, source_term::*, type_checker::*, typed_ast::*};
    use mizar_resolve::{labels::*, names::*};
    let config = config();
    let plan = crate::harness::build_test_plan(&config).unwrap();
    for id in [STEP5C5_MEANS_ID, STEP5C5_EQUALS_ID] {
        let equals = id == STEP5C5_EQUALS_ID;
        let case = plan.cases.iter().find(|case| case.id.0 == id).unwrap();
        let frontend = run_frontend(&config.workspace_root, case, 0).unwrap();
        let ast = frontend.ast.clone().unwrap();
        let (source, neutral, symbols) =
            super::super::super::source_registration_inputs(&config.workspace_root, case, frontend)
                .unwrap();
        let (bindings, _, typed) =
            TermFormulaChecker::check_source_predicate_statements(&source, &symbols, &neutral)
                .unwrap();
        let scope = SourceVariableScopeResolver::resolve_proof_occurrences(
            SourceVariableScopeInput::new(&ast, source.module(), &symbols),
        )
        .unwrap();
        assert_eq!(scope.bindings().len(), if equals { 4 } else { 2 });
        assert_eq!(bindings.bindings().len(), if equals { 6 } else { 3 });
        assert!(typed.initial_obligations().is_empty());
        let owner = symbols
            .symbols()
            .iter()
            .find(|entry| entry.kind() == mizar_resolve::env::SymbolKind::Theorem)
            .unwrap();
        let namespace = mizar_resolve::env::NamespacePath::new(source.module().path().as_str());
        let labels = ProofLabelSourceCollector::new(
            &ast,
            source.module(),
            namespace.clone(),
            owner.contribution(),
            &source,
        )
        .unwrap()
        .collect_with_theorem_owners(&symbols)
        .unwrap();
        let resolved = LabelResolver::new(labels.projections()).resolve(
            source.module(),
            &namespace,
            labels.references(),
        );
        let check = |typed: &TypedAst| {
            SourceVariableSemanticsChecker::check_theorem_skeletons(
                &source, typed, &scope, &symbols, &labels, &resolved, None,
            )
            .map(|_| ())
        };
        assert!(check(&typed).is_ok());
        let primary = typed.source_term().unwrap();
        let application = typed.source_application().unwrap();
        let application_input = SourceFunctorApplicationHandoffInput {
            source_id: typed.source_id(),
            module_id: typed.module_id().clone(),
            wrappers: Vec::new(),
            applications: application
                .applications()
                .iter()
                .map(|(_, row)| SourceFunctorApplicationInput {
                    site: row.site().clone(),
                    source_range: row.source_range(),
                    source_ordinal: row.source_ordinal(),
                    context: row.context(),
                    recovery: row.recovery(),
                    spelling: row.spelling().into(),
                    kind: row.kind(),
                    form: row.form(),
                    head_ordinal: row.head_ordinal(),
                    head: row.head().clone(),
                })
                .collect(),
            candidates: application
                .candidates()
                .iter()
                .map(|(_, row)| SourceFunctorCandidateInput {
                    application: row.application(),
                    ordinal: row.ordinal(),
                    symbol: row.symbol().clone(),
                    contribution: row.contribution(),
                })
                .collect(),
            arguments: application
                .arguments()
                .iter()
                .map(|(_, row)| SourceFunctorArgumentInput {
                    application: row.application(),
                    ordinal: row.ordinal(),
                    target: row.target(),
                })
                .collect(),
            type_requests: application
                .type_requests()
                .iter()
                .map(|(_, row)| SourceFunctorTypeRequestInput {
                    application: row.application(),
                    candidate: row.candidate(),
                    request_ordinal: row.request_ordinal(),
                    kind: row.kind(),
                })
                .collect(),
        };
        assert_eq!(
            SourceFunctorApplicationProducer::build(
                application_input.clone(),
                &symbols,
                &bindings,
                primary,
                typed.nodes()
            )
            .unwrap(),
            *application
        );
        for mutation in 0..5 {
            let mut input = application_input.clone();
            let expected =
                match mutation {
                    0 => {
                        input.candidates[0].symbol = owner.symbol().clone();
                        SourceFunctorApplicationError::InvalidCandidate {
                            candidate: SourceFunctorCandidateId::new(0),
                        }
                    }
                    1 => {
                        input.arguments[0].ordinal = 1;
                        SourceFunctorApplicationError::ReorderedArgument {
                            argument: SourceFunctorArgumentId::new(0),
                        }
                    }
                    2 => {
                        input.type_requests[1].candidate = Some(SourceFunctorCandidateId::new(0));
                        SourceFunctorApplicationError::InvalidTypeRequest {
                            request: SourceFunctorTypeRequestId::new(1),
                        }
                    }
                    3 => {
                        input.applications[0].form = if equals {
                            SourceFunctorApplicationForm::Prefix
                        } else {
                            SourceFunctorApplicationForm::Infix
                        };
                        SourceFunctorApplicationError::InvalidForm {
                            application: SourceFunctorApplicationId::new(0),
                        }
                    }
                    4 => {
                        input.arguments[0].target = SourceFunctorArgumentTarget::Primary(
                            SourcePrimaryTermId::new(if equals { 4 } else { 3 }),
                        );
                        if equals {
                            SourceFunctorApplicationError::DuplicateArgumentTarget {
                                argument: SourceFunctorArgumentId::new(0),
                            }
                        } else {
                            SourceFunctorApplicationError::InvalidArgument {
                                argument: SourceFunctorArgumentId::new(0),
                            }
                        }
                    }
                    _ => unreachable!(),
                };
            assert_eq!(
                SourceFunctorApplicationProducer::build(
                    input,
                    &symbols,
                    &bindings,
                    primary,
                    typed.nodes()
                )
                .unwrap_err(),
                expected
            );
        }
        let mut primary_input = SourcePrimaryTermHandoffInput {
            source_id: typed.source_id(),
            module_id: typed.module_id().clone(),
            numeric_type_requests: Vec::new(),
            terms: primary
                .terms()
                .iter()
                .map(|(_, row)| SourcePrimaryTermInput {
                    site: row.site().clone(),
                    source_range: row.source_range(),
                    source_ordinal: row.source_ordinal(),
                    context: row.context(),
                    recovery: row.recovery(),
                    spelling: row.spelling().into(),
                    kind: row.kind(),
                    role: row.role(),
                    parent: row.parent(),
                })
                .collect(),
            references: primary
                .references()
                .iter()
                .map(|(_, row)| SourcePrimaryTermReferenceInput {
                    term: row.term(),
                    binding: row.binding(),
                    role: row.role(),
                })
                .collect(),
        };
        let mut wrong_formal = primary_input.clone();
        wrong_formal.references[0].binding = bindings.bindings().iter().nth(1).unwrap().0;
        assert_eq!(
            SourcePrimaryTermProducer::build(wrong_formal, &bindings, typed.nodes()).unwrap_err(),
            SourcePrimaryTermError::InvalidReference {
                reference: SourcePrimaryTermReferenceId::new(0)
            }
        );
        primary_input.terms[0].role = if equals {
            SourcePrimaryTermRole::CurrentDefinitionResult
        } else {
            SourcePrimaryTermRole::Value
        };
        assert_eq!(
            SourcePrimaryTermProducer::build(primary_input, &bindings, typed.nodes()).unwrap_err(),
            SourcePrimaryTermError::InvalidTerm {
                term: SourcePrimaryTermId::new(0)
            }
        );
        let clauses = source
            .arena()
            .iter()
            .filter(|(_, node)| {
                matches!(
                    node.kind(),
                    mizar_syntax::SurfaceNodeKind::CorrectnessCondition
                )
            })
            .map(|(id, _)| id)
            .collect::<Vec<_>>();
        for mutation in 0..6 {
            let mut nodes = typed
                .nodes()
                .iter()
                .map(|(_, row)| row.clone())
                .collect::<Vec<_>>();
            let first = clauses[0];
            let second = if equals {
                source
                    .arena()
                    .iter()
                    .find(|(_, node)| {
                        matches!(
                            node.kind(),
                            mizar_syntax::SurfaceNodeKind::DefinitionParameter
                        )
                    })
                    .unwrap()
                    .0
            } else {
                clauses[1]
            };
            match mutation {
                0 if equals => nodes[first.index()].children.reverse(),
                0 => nodes[first.index()].children = nodes[second.index()].children.clone(),
                1 => nodes[first.index()].anchor = nodes[second.index()].anchor.clone(),
                2 => nodes[first.index()].resolved_node = Some(second),
                3 => {
                    nodes[primary
                        .terms()
                        .iter()
                        .next()
                        .unwrap()
                        .1
                        .site()
                        .node()
                        .index()]
                    .resolved_node = Some(source.arena().root())
                }
                4 => {
                    let id = source
                        .arena()
                        .iter()
                        .find(|(_, node)| {
                            matches!(
                                node.kind(),
                                mizar_syntax::SurfaceNodeKind::DefinitionParameter
                            )
                        })
                        .unwrap()
                        .0;
                    nodes[id.index()].children.reverse();
                }
                5 => {
                    let definition = source
                        .arena()
                        .iter()
                        .find(|(_, node)| {
                            matches!(
                                node.kind(),
                                mizar_syntax::SurfaceNodeKind::FunctorDefinition
                            )
                        })
                        .unwrap()
                        .1;
                    let ty = definition.children()[5];
                    let head = source.arena().node(ty).unwrap().children()[0];
                    let token = source.arena().node(head).unwrap().children()[0];
                    nodes[token.index()].kind =
                        r#"Token(SurfaceToken { kind: ReservedWord, text: "object" })"#.into();
                }
                _ => unreachable!(),
            }
            let changed = TypedAst::try_new(TypedAstParts {
                source_id: typed.source_id(),
                module_id: typed.module_id().clone(),
                resolved_root: Some(source.arena().root()),
                source_context: None,
                source_type: None,
                source_attribute: None,
                nodes: TypedArena::try_new(typed.nodes().root(), nodes).unwrap(),
                contexts: LocalTypeContextTable::new(),
                types: TypeTable::new(),
                facts: TypeFactTable::new(),
                coercions: CoercionTable::new(),
                initial_obligations: InitialObligationTable::new(),
                diagnostics: TypeDiagnosticTable::new(),
            })
            .unwrap()
            .with_source_term(primary.clone())
            .unwrap()
            .with_source_application(application.clone())
            .unwrap()
            .with_source_atomic_formula(typed.source_atomic_formula().unwrap().clone())
            .unwrap();
            assert_eq!(check(&changed).unwrap_err(), "theorems.source.invalid");
        }
        let text = fs::read_to_string(&case.source_path).unwrap();
        for foreign in [
            if equals {
                text.replace("+", "*")
            } else {
                text.replace("choicebox", "foreignbox")
            },
            if equals {
                text.replace("equals X", "equals Y")
            } else {
                text.replace("means it = X", "means X = it")
            },
        ] {
            let (other, nodes, env) = super::super::super::source_registration_inputs(
                &config.workspace_root,
                case,
                super::super::super::formula_statement::step5c8_test_frontend(&foreign),
            )
            .unwrap();
            let (_, _, other_typed) =
                TermFormulaChecker::check_source_predicate_statements(&other, &env, &nodes)
                    .unwrap();
            assert!(check(&other_typed).is_err());
        }
    }
}

#[test]
fn step5c5_means_source_types_and_correctness_clauses() {
    use mizar_checker::{source_term::*, type_checker::*};
    let config = config();
    let plan = crate::harness::build_test_plan(&config).unwrap();
    let case = plan
        .cases
        .iter()
        .find(|case| case.id.0 == STEP5C5_MEANS_ID)
        .unwrap();
    let text = fs::read_to_string(&case.source_path).unwrap();
    let check = |text: &str| {
        let (source, neutral, symbols) = super::super::super::source_registration_inputs(
            &config.workspace_root,
            case,
            super::super::super::formula_statement::step5c8_test_frontend(text),
        )?;
        TermFormulaChecker::check_source_predicate_statements(&source, &symbols, &neutral)
    };
    for renamed in [
        text.clone(),
        text.replace("choicebox", "parcel")
            .replace("ChoiceBoxDef", "ParcelDef")
            .replace("ChoiceBox1", "Parcel1"),
        text.replace("let X be set;\n  func", "let Formal be set;\n  func")
            .replace(
                "choicebox X -> set means it = X",
                "choicebox Formal -> set means it = Formal",
            ),
        text.replace(
            "for X being set holds choicebox X = X",
            "for Header being set holds choicebox Header = Header",
        ),
        text.replace(
            "proof\n  let X be set;\n  thus choicebox X = X",
            "proof\n  let Local be set;\n  thus choicebox Local = Local",
        ),
        text.replace("means it = X", "means X = it"),
        text.replace("holds choicebox X = X", "holds X = choicebox X")
            .replace("thus choicebox X = X", "thus X = choicebox X"),
    ] {
        let (bindings, inference, typed) =
            check(&renamed).unwrap_or_else(|error| panic!("{error}: {renamed}"));
        assert_eq!(bindings.bindings().len(), 3);
        assert_eq!(inference.terms().len(), 8);
        assert_eq!(inference.formulas().len(), 3);
        assert!(inference.facts().is_empty());
        assert!(inference.diagnostics().is_empty());
        let primary = typed.source_term().unwrap();
        assert_eq!(primary.terms().len(), 6);
        assert_eq!(primary.references().len(), 5);
        let it = primary
            .terms()
            .iter()
            .find(|(_, term)| term.kind() == SourcePrimaryTermKind::It)
            .unwrap();
        assert_eq!(it.1.role(), SourcePrimaryTermRole::CurrentDefinitionResult);
        assert!(
            !primary
                .references()
                .iter()
                .any(|(_, reference)| reference.term() == it.0)
        );
        let applications = typed.source_application().unwrap();
        assert_eq!(applications.applications().len(), 2);
        assert_eq!(applications.arguments().len(), 2);
        assert_eq!(applications.candidates().len(), 2);
        assert!(typed.initial_obligations().is_empty());
    }
    for changed in [
        text.replace("means it = X", "means X = X"),
        text.replace("means it = X", "means it = Unknown"),
        text.replace("means it = X", "means it = it"),
        text.replace("means it = X", "means not it = X"),
        text.replace("means it = X", "means it = X & X = X"),
        text.replace("-> set", "-> object"),
        text.replace("for X being set", "for X being object"),
        text.replace("thus choicebox X = X", "thus choicebox Missing = X"),
        text.replace("choicebox X ->", "X choicebox X ->"),
        text.replace("  existence;\n", ""),
        text.replace("  uniqueness;\n", ""),
        text.replace("existence;\n  uniqueness;", "uniqueness;\n  existence;"),
        text.replace("existence;", "existence; existence;"),
        text.replace("existence;", "existence by ChoiceBox1;"),
        text.replace("uniqueness;", "uniqueness; commutativity;"),
        text.replace(
            "thus choicebox X = X;",
            "thus choicebox X = X by ChoiceBox1;",
        ),
    ] {
        assert_ne!(changed, text);
        assert!(check(&changed).is_err(), "accepted {changed}");
    }
}

#[test]
fn step5c5_phrase_preserves_guarded_definition_and_open_goal() {
    use mizar_core::core_ir::*;
    let config = config();
    let plan = crate::harness::build_test_plan(&config).unwrap();
    for case in plan
        .cases
        .iter()
        .filter(|case| [STEP5C5_PHRASE_ID, STEP5C5_SYMBOLIC_ID].contains(&case.id.0.as_str()))
    {
        let symbolic = case.id.0 == STEP5C5_SYMBOLIC_ID;
        let build = || {
            phrase_theorem_output(
                &config.workspace_root,
                case,
                run_frontend(&config.workspace_root, case, 0).unwrap(),
            )
            .unwrap()
        };
        let (core, vcs) = build();
        assert_eq!((core.clone(), vcs.clone()), build());
        assert_eq!(core.items().len(), 2);
        assert_eq!(core.definitions().len(), 1);
        assert_eq!(core.proofs().len(), 1);
        assert_eq!(core.proof_nodes().len(), 2);
        assert_eq!(core.obligation_seeds().len(), 1);
        let (_, definition) = core.definitions().iter().next().unwrap();
        assert_eq!(definition.params.len(), 2);
        assert_eq!(definition.expansion, ExpansionPolicy::Transparent);
        assert!(definition.correctness.is_empty());
        let DefinitionBody::Formula(equality) = definition.body else {
            panic!("definition body")
        };
        let variable = |id| match core.terms().get(id).unwrap().kind {
            CoreTermKind::Var(var) => var,
            _ => panic!("variable term"),
        };
        let nested = if symbolic {
            let CoreFormulaKind::Forall { binders, body } =
                &core.formulas().get(equality).unwrap().kind
            else {
                panic!("quantified membership body")
            };
            assert_eq!(binders.len(), 1);
            let nested = &binders[0];
            let CoreFormulaKind::TypePred { subject, ty } =
                &core.formulas().get(nested.ty_guard.unwrap()).unwrap().kind
            else {
                panic!("object guard")
            };
            assert_eq!(ty.as_str(), "object");
            assert_eq!(variable(*subject), nested.var);
            let CoreFormulaKind::Implies {
                premise,
                conclusion,
            } = core.formulas().get(*body).unwrap().kind
            else {
                panic!("source membership implication")
            };
            for (formula, container) in [
                (premise, definition.params[0].var),
                (conclusion, definition.params[1].var),
            ] {
                let CoreFormulaKind::Membership { element, set } =
                    core.formulas().get(formula).unwrap().kind
                else {
                    panic!("primitive membership")
                };
                assert_eq!(variable(element), nested.var);
                assert_eq!(variable(set), container);
                assert_ne!(
                    core.terms().get(element).unwrap().source,
                    core.terms().get(set).unwrap().source
                );
                for source in [
                    &core.formulas().get(formula).unwrap().source,
                    &core.terms().get(element).unwrap().source,
                    &core.terms().get(set).unwrap().source,
                ] {
                    assert!(
                        source
                            .provenance
                            .iter()
                            .any(|row| row.key.as_str().starts_with("checker/predicate/"))
                    );
                }
            }
            Some(nested.var)
        } else {
            let CoreFormulaKind::Equals { left, right } =
                core.formulas().get(equality).unwrap().kind
            else {
                panic!("ordered equality body")
            };
            assert_eq!(
                [variable(left), variable(right)],
                [definition.params[0].var, definition.params[1].var]
            );
            assert_ne!(
                core.terms().get(left).unwrap().source,
                core.terms().get(right).unwrap().source
            );
            for source in [
                &core.formulas().get(equality).unwrap().source,
                &core.terms().get(left).unwrap().source,
                &core.terms().get(right).unwrap().source,
            ] {
                assert!(
                    source
                        .provenance
                        .iter()
                        .any(|row| row.key.as_str().starts_with("checker/predicate/"))
                );
                assert!(
                    !source
                        .provenance
                        .iter()
                        .any(|row| row.key.as_str().starts_with("checker/theorem/"))
                );
            }
            None
        };

        let (_, proof) = core.proofs().iter().next().unwrap();
        assert_eq!(proof.status, CoreProofStatus::PendingAutomaticProof);
        let CoreFormulaKind::Forall {
            binders: header,
            body: header_goal,
        } = &core.formulas().get(proof.proposition).unwrap().kind
        else {
            panic!("universal theorem")
        };
        let CoreProofNodeKind::IntroduceBinder {
            binder: local,
            child,
        } = &core.proof_nodes().get(proof.root).unwrap().kind
        else {
            panic!("local binder")
        };
        let CoreProofNodeKind::TerminalGoal {
            obligation,
            citations,
        } = &core.proof_nodes().get(*child).unwrap().kind
        else {
            panic!("terminal goal")
        };
        assert!(citations.is_empty());
        let seed = core.obligation_seeds().get(*obligation).unwrap();
        assert_eq!(seed.status, ObligationSeedStatus::Active);
        assert_eq!(seed.kind, ObligationSeedKind::TheoremProof);
        assert_eq!(seed.owner, proof.item);
        assert_eq!(seed.context.len(), 2);
        assert_eq!(seed.context[0], local.ty_guard.unwrap());
        assert_eq!(
            [
                definition.params[0].var,
                definition.params[1].var,
                header[0].var,
                local.var
            ]
            .into_iter()
            .collect::<std::collections::BTreeSet<_>>()
            .len(),
            4
        );
        if let Some(nested) = nested {
            assert!(
                ![
                    definition.params[0].var,
                    definition.params[1].var,
                    header[0].var,
                    local.var
                ]
                .contains(&nested)
            );
        }
        for (formula, var) in [
            (*header_goal, header[0].var),
            (seed.goal.unwrap(), local.var),
        ] {
            let CoreFormulaKind::Atom { predicate, args } =
                &core.formulas().get(formula).unwrap().kind
            else {
                panic!("actual predicate goal")
            };
            assert_eq!(predicate, &definition.symbol);
            assert_eq!(
                args.iter().map(|id| variable(*id)).collect::<Vec<_>>(),
                [var, var]
            );
            assert_ne!(
                core.terms().get(args[0]).unwrap().source,
                core.terms().get(args[1]).unwrap().source
            );
        }
        let CoreFormulaKind::Forall {
            binders: formals,
            body: biconditional,
        } = &core.formulas().get(seed.context[1]).unwrap().kind
        else {
            panic!("guarded definition")
        };
        assert_eq!(
            formals.iter().map(|binder| binder.var).collect::<Vec<_>>(),
            definition
                .params
                .iter()
                .map(|binder| binder.var)
                .collect::<Vec<_>>()
        );
        for binder in formals.iter().chain(header).chain([local]) {
            let CoreFormulaKind::TypePred { subject, ty } =
                &core.formulas().get(binder.ty_guard.unwrap()).unwrap().kind
            else {
                panic!("set guard")
            };
            assert_eq!(ty.as_str(), "set");
            assert_eq!(variable(*subject), binder.var);
        }
        let CoreFormulaKind::Iff {
            left: atom,
            right: body,
        } = core.formulas().get(*biconditional).unwrap().kind
        else {
            panic!("biconditional")
        };
        assert_eq!(body, equality);
        let CoreFormulaKind::Atom { predicate, args } = &core.formulas().get(atom).unwrap().kind
        else {
            panic!("definition atom")
        };
        assert_eq!(predicate, &definition.symbol);
        assert_eq!(
            args.iter().map(|id| variable(*id)).collect::<Vec<_>>(),
            formals.iter().map(|binder| binder.var).collect::<Vec<_>>()
        );
        let [vc] = vcs.vcs() else { panic!("one VC") };
        assert_eq!(vc.kind, mizar_vc::vc_ir::VcKind::TheoremProofStep);
        assert_eq!(vc.status, mizar_vc::vc_ir::VcStatus::Open);
        assert_eq!(
            vc.goal,
            mizar_vc::vc_ir::VcFormulaRef::Core(seed.goal.unwrap())
        );
        assert_eq!(
            vc.local_context
                .entries()
                .iter()
                .filter_map(|entry| entry.formula)
                .collect::<Vec<_>>(),
            seed.context
                .iter()
                .copied()
                .map(mizar_vc::vc_ir::VcFormulaRef::Core)
                .collect::<Vec<_>>()
        );
        assert_eq!(
            fs::read_to_string(
                config
                    .workspace_root
                    .join("tests")
                    .join(case.expectation.snapshots.as_ref().unwrap())
            )
            .unwrap(),
            vcs.debug_text()
        );
        let result = run_proof_verification_case(
            &config.workspace_root,
            &config.workspace_root.join("tests"),
            case,
            333,
        );
        assert_eq!(
            result.status,
            ProofVerificationCaseStatus::Passed,
            "{:?}",
            result.failure
        );
    }
}

#[test]
fn step5c5_symbolic_source_operands_and_scopes_are_exact() {
    use mizar_core::core_ir::*;
    let config = config();
    let plan = crate::harness::build_test_plan(&config).unwrap();
    let case = plan
        .cases
        .iter()
        .find(|case| case.id.0 == STEP5C5_SYMBOLIC_ID)
        .unwrap();
    let text = fs::read_to_string(&case.source_path).unwrap();
    let check = |text: &str| {
        phrase_theorem_output(
            &config.workspace_root,
            case,
            super::super::super::formula_statement::step5c8_test_frontend(text),
        )
    };
    for renamed in [
        text.clone(),
        text.replace("SubsetLikeDef", "OtherDefinition"),
        text.replace("SubsetRefl1", "OtherTheorem"),
        text.replace("<<=", ">>="),
        text.replace("X, Y", "First, Y")
            .replace("X <<= Y means", "First <<= Y means")
            .replace("z in X", "z in First"),
        text.replace("X, Y", "X, Second")
            .replace("X <<= Y means", "X <<= Second means")
            .replace("z in Y", "z in Second"),
        text.replace(
            "for z being object st z in X holds z in Y",
            "for Element being object st Element in X holds Element in Y",
        ),
        text.replace(
            "for X being set holds X <<= X",
            "for Header being set holds Header <<= Header",
        ),
        text.replace("let X be set;", "let Local be set;")
            .replace("thus X <<= X", "thus Local <<= Local"),
        text.replace(
            "for X being set holds X <<= X",
            "for z being set holds z <<= z",
        )
        .replace("let X be set;", "let z be set;")
        .replace("thus X <<= X", "thus z <<= z"),
    ] {
        assert!(
            check(&renamed).is_ok(),
            "{renamed}: {:?}",
            check(&renamed).err()
        );
    }
    let baseline = check(&text).unwrap();
    for (changed, indices) in [
        (
            text.replace("z in X holds z in Y", "z in Y holds z in X"),
            [1, 0],
        ),
        (
            text.replace("z in X holds z in Y", "z in X holds z in X"),
            [0, 0],
        ),
        (
            text.replace(
                "for z being object st z in X holds z in Y",
                "for X being object st X in Y holds X in Y",
            ),
            [1, 1],
        ),
    ] {
        let output = check(&changed).unwrap();
        assert_ne!(output.0, baseline.0);
        assert_eq!(output, check(&changed).unwrap());
        let core = &output.0;
        let (_, definition) = core.definitions().iter().next().unwrap();
        assert_eq!(definition.params.len(), 2);
        assert_ne!(definition.params[0].var, definition.params[1].var);
        assert!(
            definition
                .params
                .iter()
                .all(|binder| binder.ty_guard.is_some())
        );
        let DefinitionBody::Formula(body) = definition.body else {
            panic!()
        };
        let CoreFormulaKind::Forall { binders, body } = &core.formulas().get(body).unwrap().kind
        else {
            panic!()
        };
        assert!(binders[0].ty_guard.is_some());
        assert!(
            definition
                .params
                .iter()
                .all(|formal| formal.var != binders[0].var)
        );
        let mut variables = definition
            .params
            .iter()
            .map(|binder| binder.var)
            .collect::<std::collections::BTreeSet<_>>();
        variables.insert(binders[0].var);
        for (_, proof_node) in core.proof_nodes().iter() {
            if let CoreProofNodeKind::IntroduceBinder { binder, .. } = &proof_node.kind {
                variables.insert(binder.var);
            }
        }
        for (_, formula) in core.formulas().iter() {
            if let CoreFormulaKind::Forall { binders, .. } = &formula.kind {
                variables.extend(binders.iter().map(|binder| binder.var));
            }
        }
        assert_eq!(variables.len(), 5);
        let CoreFormulaKind::Implies {
            premise,
            conclusion,
        } = core.formulas().get(*body).unwrap().kind
        else {
            panic!()
        };
        for (formula, index) in [premise, conclusion].into_iter().zip(indices) {
            let CoreFormulaKind::Membership { element, set } =
                core.formulas().get(formula).unwrap().kind
            else {
                panic!()
            };
            assert_eq!(
                core.terms().get(element).unwrap().kind,
                CoreTermKind::Var(binders[0].var)
            );
            assert_eq!(
                core.terms().get(set).unwrap().kind,
                CoreTermKind::Var(definition.params[index].var)
            );
        }
    }
    for changed in [
        text.replace("z in Y", "z in Missing"),
        text.replace("z in Y", "z in z"),
        text.replace(
            "for z being object st z in X holds z in Y",
            "for X being object st X in X holds X in Y",
        ),
        text.replace("for z being object", "for z being set"),
        text.replace("holds z in Y", "holds not z in Y"),
        text.replace("holds z in Y", "holds z in Y & z in X"),
        text.replace("holds z in Y", "holds for w being object holds w in Y"),
        text.replace("thus X <<= X;", "thus X <<= X by SubsetRefl1;"),
        text.replace("thus X <<= X", "thus not X <<= X"),
        text.replace("z in Y;", "z in Y; symmetry;"),
        text.replace("let X be set", "let X be object"),
    ] {
        assert_ne!(changed, text);
        assert!(check(&changed).is_err(), "accepted {changed}");
    }
}

#[test]
fn step5c5_phrase_source_renaming_and_near_misses() {
    let config = config();
    let plan = crate::harness::build_test_plan(&config).unwrap();
    let case = plan
        .cases
        .iter()
        .find(|case| case.id.0 == STEP5C5_PHRASE_ID)
        .unwrap();
    let text = fs::read_to_string(&case.source_path).unwrap();
    let check = |text: &str| {
        phrase_theorem_output(
            &config.workspace_root,
            case,
            super::super::super::formula_statement::step5c8_test_frontend(text),
        )
    };
    for renamed in [
        text.clone(),
        text.replace("LikesDef", "PredicateLabel"),
        text.replace("LikesSelf1", "TheoremLabel"),
        text.replace("likes", "admires"),
        text.replace("X, Y", "First, Y")
            .replace("X likes Y means X = Y", "First likes Y means First = Y"),
        text.replace("X, Y", "X, Second")
            .replace("X likes Y means X = Y", "X likes Second means X = Second"),
        text.replace(
            "for X being set holds X likes X",
            "for Header being set holds Header likes Header",
        ),
        text.replace("let X be set;", "let Local be set;")
            .replace("thus X likes X", "thus Local likes Local"),
    ] {
        assert!(
            check(&renamed).is_ok(),
            "{renamed}: {:?}",
            check(&renamed).err()
        );
    }
    for changed in [
        text.replace("likes", "~"),
        text.replace("means X = Y", "means Y = X"),
        text.replace("means X = Y", "means X = X"),
        text.replace("means X = Y", "means not X = Y"),
        text.replace("thus X likes X", "thus X does not likes X"),
        text.replace("holds X likes X", "holds X does not likes X"),
        text.replace("thus X likes X", "thus X admires X"),
        text.replace("thus X likes X", "thus X likes Y"),
        text.replace("for X being set", "for X being object"),
        text.replace("let X be set", "let X be object"),
        text.replace("let X be set", "let X be non empty set"),
        text.replace("let X be set;", "let X be set; assume A: X = X;"),
        text.replace("thus X likes X;", "thus X likes X by LikesSelf1;"),
        text.replace("means X = Y;", "means X = Y; symmetry;"),
        text.replace("X, Y", "X, X"),
        text.replace(
            "end;\n\ntheorem",
            "end;\ndefinition let A,B be set; pred Extra: A loves B means A = B; end;\ntheorem",
        ),
        text.replace("thus X likes X;", "thus X likes X; thus X likes X;"),
        text.replace("end;", ""),
    ] {
        assert_ne!(changed, text, "ineffective mutation");
        assert!(check(&changed).is_err(), "accepted {changed}");
    }
}

#[test]
fn step5c5_phrase_authenticates_receipts_and_binding_correspondence() {
    use mizar_checker::{
        type_checker::{SourceVariableSemanticsChecker, TermFormulaChecker},
        typed_ast::*,
    };
    use mizar_resolve::{
        labels::{LabelResolver, ProofLabelSourceCollector},
        names::{SourceVariableScopeInput, SourceVariableScopeResolver},
    };
    let config = config();
    let plan = crate::harness::build_test_plan(&config).unwrap();
    for case in plan
        .cases
        .iter()
        .filter(|case| [STEP5C5_PHRASE_ID, STEP5C5_SYMBOLIC_ID].contains(&case.id.0.as_str()))
    {
        let symbolic = case.id.0 == STEP5C5_SYMBOLIC_ID;
        let frontend = run_frontend(&config.workspace_root, case, 0).unwrap();
        let ast = frontend.ast.clone().unwrap();
        let (source, neutral, symbols) =
            super::super::super::source_registration_inputs(&config.workspace_root, case, frontend)
                .unwrap();
        let (bindings, inference, typed) =
            TermFormulaChecker::check_source_predicate_statements(&source, &symbols, &neutral)
                .unwrap();
        assert_eq!(bindings.bindings().len(), if symbolic { 5 } else { 4 });
        assert_eq!(
            bindings
                .bindings()
                .iter()
                .map(|(_, row)| (row.declaration_range.start, row.declaration_range.end))
                .collect::<std::collections::BTreeSet<_>>()
                .len(),
            if symbolic { 5 } else { 4 }
        );
        assert_eq!(inference.terms().len(), if symbolic { 8 } else { 6 });
        assert!(
            inference
                .type_entries()
                .iter()
                .all(|(_, entry)| entry.status == TypeStatus::Known)
        );
        let primary = typed.source_term().unwrap();
        let atomic = typed.source_atomic_formula().unwrap();
        assert_eq!(primary.terms().len(), if symbolic { 8 } else { 6 });
        assert_eq!(
            primary
                .terms()
                .iter()
                .map(|(_, term)| (term.source_range().start, term.source_range().end))
                .collect::<std::collections::BTreeSet<_>>()
                .len(),
            if symbolic { 8 } else { 6 }
        );
        assert_eq!(atomic.candidates().len(), 2);
        for (formula, row) in atomic.formulas().iter() {
            use mizar_checker::source_atomic_formula::{
                SourceAtomicEdgeRole, SourceAtomicFormulaKind, SourceAtomicRequestKind,
            };
            if row.kind() == SourceAtomicFormulaKind::Membership {
                let requests = atomic
                    .requests()
                    .iter()
                    .filter(|(_, request)| request.formula() == formula)
                    .map(|(_, request)| request)
                    .collect::<Vec<_>>();
                assert_eq!(requests.len(), 1);
                assert_eq!(requests[0].ordinal(), 0);
                assert_eq!(
                    requests[0].kind(),
                    SourceAtomicRequestKind::OperandExpectedType
                );
                let edge = atomic.edges().get(requests[0].edge().unwrap()).unwrap();
                assert_eq!(edge.ordinal(), 1);
                assert_eq!(edge.role(), SourceAtomicEdgeRole::BuiltinRightOperand);
            }
        }
        assert_eq!(atomic.predicate_segments().len(), 2);
        let scope = SourceVariableScopeResolver::resolve_proof_occurrences(
            SourceVariableScopeInput::new(&ast, source.module(), &symbols),
        )
        .unwrap();
        assert_eq!(scope.bindings().len(), 2);
        let owner = symbols
            .symbols()
            .iter()
            .find(|entry| entry.kind() == mizar_resolve::env::SymbolKind::Theorem)
            .unwrap();
        let namespace = mizar_resolve::env::NamespacePath::new(source.module().path().as_str());
        let labels = ProofLabelSourceCollector::new(
            &ast,
            source.module(),
            namespace.clone(),
            owner.contribution(),
            &source,
        )
        .unwrap()
        .collect_with_theorem_owners(&symbols)
        .unwrap();
        let resolved = LabelResolver::new(labels.projections()).resolve(
            source.module(),
            &namespace,
            labels.references(),
        );
        let check = |typed: &TypedAst| {
            SourceVariableSemanticsChecker::check_theorem_skeletons(
                &source, typed, &scope, &symbols, &labels, &resolved, None,
            )
            .map(|_| ())
        };
        assert!(check(&typed).is_ok());
        for (_, entry) in inference.type_entries().iter() {
            let TypeEntryActual::Known(ty) = entry.actual else {
                panic!("known set type")
            };
            let head = &inference.normalized_types().get(ty).unwrap().head;
            assert!(
                head == &mizar_checker::type_checker::TypeHeadRef::BuiltinSet
                    || symbolic && head == &mizar_checker::type_checker::TypeHeadRef::BuiltinObject
            );
        }
        let predicate = symbols
            .symbols()
            .iter()
            .find(|entry| entry.kind() == mizar_resolve::env::SymbolKind::Predicate)
            .unwrap();
        assert!(
            atomic
                .candidates()
                .iter()
                .all(|(_, candidate)| candidate.symbol() == predicate.symbol())
        );
        let assemble = |nodes, primary: &mizar_checker::source_term::SourcePrimaryTermHandoff, atomic: &mizar_checker::source_atomic_formula::SourceAtomicFormulaHandoff| {
        TypedAst::try_new(TypedAstParts {
            source_id: typed.source_id(), module_id: typed.module_id().clone(), resolved_root: Some(source.arena().root()),
            source_context: None, source_type: None, source_attribute: None, nodes,
            contexts: LocalTypeContextTable::new(), types: TypeTable::new(), facts: TypeFactTable::new(),
            coercions: CoercionTable::new(), initial_obligations: InitialObligationTable::new(), diagnostics: TypeDiagnosticTable::new(),
        }).and_then(|ast| ast.with_source_term(primary.clone())).and_then(|ast| ast.with_source_atomic_formula(atomic.clone()))
    };

        let target = atomic
            .predicate_heads()
            .iter()
            .next()
            .unwrap()
            .1
            .site()
            .node();
        for mutation in 0..5 {
            let mut nodes = typed
                .nodes()
                .iter()
                .map(|(_, node)| node.clone())
                .collect::<Vec<_>>();
            match mutation {
                0 => nodes[target.index()].kind = "PredicateHead".into(),
                1 => nodes[target.index()].children.clear(),
                2 => nodes[target.index()].recovery = NodeRecoveryState::Recovered,
                3 => nodes[target.index()].anchor = nodes[0].anchor.clone(),
                4 => nodes[target.index()].resolved_node = Some(source.arena().root()),
                _ => unreachable!(),
            }
            let changed = TypedArena::try_new(typed.nodes().root(), nodes).unwrap();
            match assemble(changed, primary, atomic) {
                Ok(changed) => assert!(check(&changed).is_err(), "mutation {mutation}"),
                Err(error) => assert_eq!(
                    error,
                    TypedAstError::InvalidSourceAtomicFormula,
                    "mutation {mutation}"
                ),
            }
        }
        if symbolic {
            let quantifiers = typed
                .nodes()
                .iter()
                .filter(|(_, node)| node.kind.as_str() == "QuantifiedFormula(Universal)")
                .map(|(id, _)| id)
                .collect::<Vec<_>>();
            let nested = quantifiers[0];
            let segment = typed.nodes().node(nested).unwrap().children[1];
            let declaration = typed.nodes().node(segment).unwrap().children[0];
            let parent = typed
                .nodes()
                .iter()
                .find(|(_, node)| node.children.contains(&nested))
                .unwrap()
                .0;
            let membership = atomic
                .formulas()
                .iter()
                .find(|(_, row)| {
                    row.kind()
                        == mizar_checker::source_atomic_formula::SourceAtomicFormulaKind::Membership
                })
                .unwrap()
                .1
                .site()
                .node();
            for mutation in 0..5 {
                let mut nodes = typed
                    .nodes()
                    .iter()
                    .map(|(_, node)| node.clone())
                    .collect::<Vec<_>>();
                match mutation {
                    0 => nodes[nested.index()].children.swap(3, 5),
                    1 => nodes[declaration.index()].anchor = nodes[0].anchor.clone(),
                    2 => nodes[nested.index()].resolved_node = Some(source.arena().root()),
                    3 => {
                        let child = nodes[parent.index()]
                            .children
                            .iter_mut()
                            .find(|id| **id == nested)
                            .unwrap();
                        *child = quantifiers[1];
                    }
                    4 => nodes[membership.index()].children.swap(0, 2),
                    _ => unreachable!(),
                }
                let changed = assemble(
                    TypedArena::try_new(typed.nodes().root(), nodes).unwrap(),
                    primary,
                    atomic,
                )
                .expect("structurally valid hostile graph reaches replay seal");
                assert_eq!(
                    check(&changed),
                    Err("theorems.source.invalid".to_owned()),
                    "nested mutation {mutation}"
                );
            }
        }
        let text = fs::read_to_string(&case.source_path).unwrap();
        // Graft independently valid semantic receipts onto the unchanged original
        // graph: this isolates candidate-symbol and operand-binding authentication.
        let head = if symbolic { "<<=" } else { "likes" };
        let renamed_head = if symbolic { ">>=" } else { "hates" };
        let theorem = if symbolic {
            "SubsetRefl1"
        } else {
            "LikesSelf1"
        };
        let swapped = if symbolic {
            text.replace("z in X holds z in Y", "z in Y holds z in X")
        } else {
            text.replace("means X = Y", "means Y = X")
        };
        for text in [text.replace(head, renamed_head), swapped] {
            let (foreign, nodes, env) = super::super::super::source_registration_inputs(
                &config.workspace_root,
                case,
                super::super::super::formula_statement::step5c8_test_frontend(&text),
            )
            .unwrap();
            let (_, _, foreign_typed) =
                TermFormulaChecker::check_source_predicate_statements(&foreign, &env, &nodes)
                    .unwrap();
            let grafted = assemble(
                typed.nodes().clone(),
                foreign_typed.source_term().unwrap(),
                foreign_typed.source_atomic_formula().unwrap(),
            )
            .unwrap();
            assert!(check(&grafted).is_err());
        }
        for changed in [
            text.replace(head, renamed_head)
                .replace(theorem, "OtherOwner"),
            text.replace(theorem, "OtherOwner"),
            text.replace("let X be set;", "let Z be set;")
                .replace(&format!("thus X {head} X"), &format!("thus Z {head} Z"))
                .replace(theorem, "OtherOwner"),
        ] {
            let frontend = super::super::super::formula_statement::step5c8_test_frontend(&changed);
            let ast = frontend.ast.clone().unwrap();
            let (foreign, nodes, env) = super::super::super::source_registration_inputs(
                &config.workspace_root,
                case,
                frontend,
            )
            .unwrap();
            assert!(
                TermFormulaChecker::check_source_predicate_statements(&source, &env, &neutral)
                    .is_err()
            );
            assert!(
                TermFormulaChecker::check_source_predicate_statements(&source, &symbols, &nodes)
                    .is_err()
            );
            let (_, _, foreign_typed) =
                TermFormulaChecker::check_source_predicate_statements(&foreign, &env, &nodes)
                    .unwrap();
            assert!(check(&foreign_typed).is_err());
            let foreign_scope = SourceVariableScopeResolver::resolve_proof_occurrences(
                SourceVariableScopeInput::new(&ast, foreign.module(), &env),
            )
            .unwrap();
            let owner = env
                .symbols()
                .iter()
                .find(|entry| entry.kind() == mizar_resolve::env::SymbolKind::Theorem)
                .unwrap();
            let foreign_labels = ProofLabelSourceCollector::new(
                &ast,
                foreign.module(),
                namespace.clone(),
                owner.contribution(),
                &foreign,
            )
            .unwrap()
            .collect_with_theorem_owners(&env)
            .unwrap();
            let result = SourceVariableSemanticsChecker::check_theorem_skeletons(
                &source,
                &typed,
                &foreign_scope,
                &symbols,
                &foreign_labels,
                &resolved,
                None,
            );
            assert!(result.is_err(), "foreign ownership admitted: {changed}");
        }
    }
}

#[test]
fn step5c5_phrase_metadata_inventory_and_cross_stage_aliases() {
    let config = config();
    let plan = crate::harness::build_test_plan(&config).unwrap();
    for case in plan.cases.iter().filter(|case| {
        [
            STEP5C5_PHRASE_ID,
            STEP5C5_SYMBOLIC_ID,
            STEP5C5_MEANS_ID,
            STEP5C5_EQUALS_ID,
        ]
        .contains(&case.id.0.as_str())
    }) {
        assert!(step5c5_phrase_admitted(Some(&config.workspace_root), case));
        assert!(
            crate::expectation::validate_expectation_path(
                &case.expectation_path,
                &case.expectation,
                &config.workspace_root.join("tests")
            )
            .is_empty()
        );
        for mutation in 0..28 {
            let mut changed = case.clone();
            match mutation {
                0 => changed.id.0.push('_'),
                1 => changed.expectation.id.0.push('_'),
                2 => changed.source_path = changed.source_path.with_file_name("other.miz"),
                3 => {
                    changed.expectation_path =
                        changed.expectation_path.with_file_name("other.expect.toml")
                }
                4 => changed.expectation.source = "other.miz".into(),
                5 => changed.expectation.domain.push('_'),
                6 => changed.expectation.expected_phase = Some(PipelinePhase::Verification),
                7 => changed.expectation.expected_outcome = ExpectedOutcome::Fail,
                8 => changed.expectation.spec_refs.reverse(),
                9 => changed.expectation.spec_refs.pop().map(|_| ()).unwrap(),
                10 => changed
                    .expectation
                    .tags
                    .push(ACTIVE_PROOF_VERIFICATION_TAG.into()),
                11 => changed.expectation.snapshots = None,
                12 => changed.expectation.snapshots = Some("snapshots/vc/other.snap".into()),
                13 => changed.expectation.stable_detail_key = Some("unrelated".into()),
                14 => changed.expectation.failure_category = Some("proof_failure".into()),
                15 => changed.expectation.schema_version += 1,
                16 => changed.expectation.profiles.push("other".into()),
                17 => changed.expectation.kind = crate::expectation::TestKind::Fail,
                18 => changed.expectation.rejection_reason = Some("other".into()),
                19 => changed
                    .expectation
                    .diagnostic_codes
                    .push("E-UNRELATED".into()),
                20 => changed
                    .expectation
                    .diagnostic_payloads
                    .push("unrelated".into()),
                21 => changed
                    .expectation
                    .declaration_symbol_payloads
                    .push("unrelated".into()),
                22 => changed.expectation.ast_profile = Some("unrelated".into()),
                23 => changed
                    .expectation
                    .snapshot_profiles
                    .push("unrelated".into()),
                24 => {
                    changed.expectation.tokens = plan
                        .cases
                        .iter()
                        .find(|row| !row.expectation.tokens.is_empty())
                        .unwrap()
                        .expectation
                        .tokens
                        .clone()
                }
                25 => {
                    changed.expectation.origin = Some(crate::expectation::OriginMetadata {
                        schema_version: 1,
                        kind: changed.expectation.kind,
                        generator: "test".into(),
                        generator_version: "1".into(),
                        seed: "0".into(),
                        profile: "fast".into(),
                        expected_outcome: changed.expectation.expected_outcome,
                        minimized: false,
                        original_failure_category: None,
                    })
                }
                26 => {
                    changed.expectation.architecture22 =
                        Some(crate::expectation::Architecture22Metadata {
                            scenarios: vec!["cache_hit_miss_timing".into()],
                            equivalence_class: Some("observable_outputs_equal".into()),
                            gate: crate::expectation::Architecture22Gate::Planned,
                        })
                }
                27 => changed
                    .expectation
                    .spec_refs
                    .push(changed.expectation.spec_refs[0].clone()),
                _ => unreachable!(),
            }
            assert!(is_step5c5_phrase_candidate(&changed));
            assert!(
                !step5c5_phrase_admitted(Some(&config.workspace_root), &changed),
                "mutation {mutation}"
            );
            if !matches!(mutation, 0 | 2 | 11) {
                assert!(
                    crate::expectation::validate_expectation_path(
                        &changed.expectation_path,
                        &changed.expectation,
                        &config.workspace_root.join("tests")
                    )
                    .iter()
                    .any(|diagnostic| diagnostic.code.0 == "E-EXPECT-SNAPSHOT-SCOPE"),
                    "snapshot metadata mutation {mutation}"
                );
            }
        }
        for (stage, tag, phase) in [
            (Stage::ParseOnly, "active_parse_only", PipelinePhase::Parse),
            (
                Stage::DeclarationSymbol,
                "active_declaration_symbol",
                PipelinePhase::Resolve,
            ),
            (
                Stage::TypeElaboration,
                "active_type_elaboration",
                PipelinePhase::TypeCheck,
            ),
            (
                Stage::FormulaStatement,
                "active_formula_statement",
                PipelinePhase::StatementCheck,
            ),
            (
                Stage::AdvancedSemantics,
                "active_advanced_semantics",
                PipelinePhase::ClusterResolution,
            ),
        ] {
            for alias in 0..4 {
                let mut changed = case.clone();
                changed.expectation.stage = stage;
                changed.expectation.tags = vec![tag.into()];
                changed.expectation.expected_phase = Some(phase);
                if alias > 0 {
                    changed.id.0 = "alias".into();
                }
                if alias > 1 {
                    changed.expectation.id.0 = "alias".into();
                }
                if alias > 2 {
                    changed.source_path = changed.source_path.with_file_name("alias.miz");
                }
                assert!(is_step5c5_phrase_candidate(&changed));
                assert!(!is_active_proof_verification(&changed));
                assert!(!super::super::super::step5c11_registration_admitted(
                    &config.workspace_root,
                    &changed
                ));
                assert!(!super::super::super::step5c13_overload_admitted(
                    &config.workspace_root,
                    &changed
                ));
                assert!(!super::super::super::is_active_parse_only(&changed));
                assert!(!super::super::super::is_active_declaration_symbol(&changed));
                assert!(!super::super::super::is_active_type_elaboration(&changed));
                assert!(
                    !super::super::super::formula_statement::is_active_formula_statement(
                        &config.workspace_root,
                        &changed
                    )
                );
            }
        }
        for duplicate in [false, true] {
            let mut changed = plan.clone();
            if duplicate {
                changed.cases.push(case.clone());
            } else {
                changed.cases.retain(|row| row.id != case.id);
            }
            assert!(
                validate_active_proof_verification_tags(&config.workspace_root, &changed)
                    .iter()
                    .any(|diagnostic| diagnostic.code.0
                        == "E-PROOF-VERIFICATION-STEP5C5-INVENTORY")
            );
        }
    }
}

#[test]
fn step5c5_phrase_downstream_structural_controls_and_baselines() {
    use mizar_core::core_ir::*;
    let config = config();
    let plan = crate::harness::build_test_plan(&config).unwrap();
    for case in plan
        .cases
        .iter()
        .filter(|case| [STEP5C5_PHRASE_ID, STEP5C5_SYMBOLIC_ID].contains(&case.id.0.as_str()))
    {
        let (core, vcs) = phrase_theorem_output(
            &config.workspace_root,
            case,
            run_frontend(&config.workspace_root, case, 0).unwrap(),
        )
        .unwrap();
        let (seed_id, seed) = core.obligation_seeds().iter().next().unwrap();
        let (proof_id, proof) = core.proofs().iter().next().unwrap();
        let (definition_id, definition) = core.definitions().iter().next().unwrap();
        assert!(
            seed.core_refs
                .contains(&CoreNodeRef::Definition(definition_id))
        );
        assert!(
            seed.core_refs
                .contains(&CoreNodeRef::Item(definition.owner.anchor_item()))
        );
        assert!(
            core.terms()
                .iter()
                .all(|(id, _)| seed.core_refs.contains(&CoreNodeRef::Term(id)))
        );
        assert!(
            core.formulas()
                .iter()
                .all(|(id, _)| seed.core_refs.contains(&CoreNodeRef::Formula(id)))
        );
        for mutation in 0..11 {
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
            let changed = parts.obligation_seeds.get_mut(seed_id).unwrap();
            match mutation {
                0 => {
                    changed.context.remove(0);
                }
                1 => changed.context.push(seed.goal.unwrap()),
                2 => changed.context.reverse(),
                3 => changed.goal = Some(proof.proposition),
                4 => changed.local_path = "".into(),
                5 => changed.status = ObligationSeedStatus::Skipped,
                6 => changed.owner = definition.owner.anchor_item(),
                7 => changed.core_refs.clear(),
                8 => parts.proofs.get_mut(proof_id).unwrap().status = CoreProofStatus::Open,
                9 => {
                    parts.definitions.get_mut(definition_id).unwrap().owner =
                        CoreDefinitionOwner::for_item(proof.item)
                }
                10 => {
                    let CoreProofNodeKind::IntroduceBinder { binder, .. } =
                        &mut parts.proof_nodes.get_mut(proof.root).unwrap().kind
                    else {
                        panic!()
                    };
                    binder.var = definition.params[0].var;
                }
                _ => unreachable!(),
            }
            // Core-only fields such as Pending status need complete Core comparison:
            // generic seed intake does not authenticate a detached graph as source.
            let changed = CoreIr::try_new(parts)
                .unwrap_or_else(|error| panic!("Core mutation {mutation}: {error}"));
            let actual = generate_core_vcs(&changed, snapshot_id(0))
                .unwrap_or_else(|error| panic!("VC mutation {mutation}: {error}"));
            assert_ne!(
                (changed, actual),
                (core.clone(), vcs.clone()),
                "mutation {mutation}"
            );
        }
        let temporary = std::process::Command::new("mktemp")
            .arg("-d")
            .output()
            .unwrap();
        assert!(temporary.status.success());
        let root = std::path::PathBuf::from(String::from_utf8(temporary.stdout).unwrap().trim());
        let path = root.join(case.expectation.snapshots.as_ref().unwrap());
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        for contents in [None, Some("corrupt VC baseline")] {
            if let Some(contents) = contents {
                fs::write(&path, contents).unwrap();
            }
            let run = run_proof_verification_case(&config.workspace_root, &root, case, 0);
            assert_eq!(run.status, ProofVerificationCaseStatus::Failed);
            assert!(run.failure.unwrap().contains("snapshot"));
        }
        fs::remove_dir_all(root).unwrap();
    }
}
