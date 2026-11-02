use super::*;

#[test]
fn step5c14_static_real_sources_preserve_checked_bindings_and_actual_cfg_errors() {
    use mizar_checker::type_checker::{
        TermReference, TermStatus, TypeHeadRef, check_source_algorithm_types,
    };
    use mizar_checker::typed_ast::TypeEntryActual;
    use mizar_core::control_flow::{
        ControlFlowDiagnosticKind as D, LocalMutability, build_control_flow_ir,
    };
    use mizar_core::core_ir::{
        CoreAlgorithmStmtKind as S, CoreFormulaKind, CoreSourceAnchor, CoreTermKind,
    };
    let config = step5c11_config();
    for name in ["break_outside_loop", "ghost_isolation"] {
        let case = step5c14_static_case(name);
        let text = std::fs::read_to_string(&case.source_path).unwrap();
        let renamed = text
            .replace("badbreak", "other_break")
            .replace("ghostleak", "other_ghost")
            .replace(" a ", " param ")
            .replace("(a)", "(param)")
            .replace(" a;", " param;")
            .replace(" g ", " hidden ")
            .replace(" g;", " hidden;")
            .replace(" x ", " visible ")
            .replace(" x;", " visible;");
        for source_text in [&text, &renamed] {
            let (source, typed, symbols) = super::super::source_registration_inputs(
                &config.workspace_root,
                &case,
                super::super::formula_statement::step5c8_test_frontend(source_text),
            )
            .unwrap();
            let checked = check_source_algorithm_types(&source, &typed, &symbols).unwrap();
            let inference = checked.inference();
            assert!(
                inference.diagnostics().is_empty()
                    && inference.facts().is_empty()
                    && inference.candidate_sets().is_empty()
            );
            assert_eq!(
                checked.bindings().bindings().len(),
                if name == "ghost_isolation" { 3 } else { 1 }
            );
            assert_eq!(
                inference.terms().len(),
                if name == "ghost_isolation" { 4 } else { 2 }
            );
            for (_, term) in inference.terms().iter() {
                assert_eq!(term.status, TermStatus::Inferred);
                let Some(TermReference::Binding(binding)) = term.reference else {
                    panic!("missing actual binding")
                };
                let entry = checked.bindings().bindings().get(binding).unwrap();
                let mizar_session::SourceAnchor::Range(use_range) =
                    typed.node(term.site.node()).unwrap().anchor
                else {
                    panic!("missing use range")
                };
                assert!(entry.declaration_range.end <= use_range.start);
                let TypeEntryActual::Known(ty) = inference
                    .type_entries()
                    .get(term.type_entry)
                    .unwrap()
                    .actual
                else {
                    panic!("unknown type")
                };
                assert_eq!(
                    inference.normalized_types().get(ty).unwrap().head,
                    TypeHeadRef::BuiltinObject
                );
            }
            let core = mizar_core::elaborator::lower_source_algorithms(&checked).unwrap();
            assert_eq!(core, step5c14_static_core(&case, source_text).unwrap());
            assert!(
                core.obligation_seeds().is_empty()
                    && core.diagnostics().is_empty()
                    && core.proofs().is_empty()
            );
            let (algorithm_id, algorithm) = core.algorithms().iter().next().unwrap();
            assert_eq!(core.algorithms().len(), 1);
            assert_eq!(&algorithm.symbol, &checked.algorithm().1);
            assert_eq!(algorithm.params.len(), 1);
            let result = algorithm
                .result
                .as_ref()
                .expect("preserve written return type");
            assert_ne!(result.var, algorithm.params[0].var);
            let CoreFormulaKind::TypePred { subject, ty } =
                &core.formulas().get(result.ty_guard.unwrap()).unwrap().kind
            else {
                panic!("missing result type guard")
            };
            assert_eq!(ty.as_str(), "object");
            assert_eq!(
                core.terms().get(*subject).unwrap().kind,
                CoreTermKind::Var(result.var)
            );
            let algorithm_source = source
                .arena()
                .node(
                    typed
                        .node(checked.algorithm().0)
                        .unwrap()
                        .resolved_node
                        .unwrap(),
                )
                .unwrap();
            let return_type = algorithm_source
                .children()
                .iter()
                .copied()
                .find(|id| {
                    source.arena().node(*id).unwrap().kind()
                        == &mizar_syntax::ast::SurfaceNodeKind::TypeExpression
                })
                .unwrap();
            let mizar_session::SourceAnchor::Range(return_range) =
                source.arena().node(return_type).unwrap().origin().anchor()
            else {
                panic!("missing written return range")
            };
            assert_eq!(
                result.source.anchor,
                CoreSourceAnchor::SourceRange(*return_range)
            );
            assert!(result.source.provenance.iter().any(|p| p.phase
                == mizar_core::core_ir::CoreProvenancePhase::Checker
                && p.key.as_str() == format!("algorithm/source-node#{}", return_type.index())));
            assert_eq!(
                algorithm.contracts,
                mizar_core::core_ir::CoreContractSet::default()
            );
            let statements = algorithm
                .statements
                .iter()
                .map(|id| (*id, core.algorithm_statements().get(*id).unwrap()))
                .collect::<Vec<_>>();
            let source_statements = source
                .arena()
                .iter()
                .find(|(_, node)| {
                    node.kind() == &mizar_syntax::ast::SurfaceNodeKind::AlgorithmStatementList
                })
                .unwrap()
                .1
                .children();
            assert_eq!(statements.len(), source_statements.len());
            for (_, statement) in &statements {
                if let S::Let { binder, .. } = &statement.kind {
                    assert_ne!(result.var, binder.var);
                }
            }
            for ((_, statement), source_id) in statements.iter().zip(source_statements) {
                assert_eq!(statement.owner, algorithm_id);
                let mizar_session::SourceAnchor::Range(range) =
                    source.arena().node(*source_id).unwrap().origin().anchor()
                else {
                    panic!("missing statement range")
                };
                assert_eq!(
                    statement.source.anchor,
                    CoreSourceAnchor::SourceRange(*range)
                );
            }
            if name == "ghost_isolation" {
                let S::Let {
                    binder: g,
                    value: Some(initial),
                    ghost: true,
                } = &statements[0].1.kind
                else {
                    panic!("missing ghost initialization")
                };
                let S::Let {
                    binder: x,
                    value: Some(read_g),
                    ghost: false,
                } = &statements[1].1.kind
                else {
                    panic!("missing runtime initialization")
                };
                assert_ne!(g.var, x.var);
                assert_eq!(
                    core.terms().get(*initial).unwrap().kind,
                    CoreTermKind::Var(algorithm.params[0].var)
                );
                assert_eq!(
                    core.terms().get(*read_g).unwrap().kind,
                    CoreTermKind::Var(g.var)
                );
                let S::Return(Some(ret)) = statements[2].1.kind else {
                    panic!("missing return")
                };
                assert_eq!(
                    core.terms().get(ret).unwrap().kind,
                    CoreTermKind::Var(x.var)
                );
            } else {
                assert!(matches!(statements[0].1.kind, S::Break));
                assert!(matches!(statements[1].1.kind, S::Return(_)));
            }
            let flow = build_control_flow_ir(&core);
            assert_eq!(flow, build_control_flow_ir(&core));
            assert_eq!(flow.flows.len(), 1);
            let (_, flow) = flow.flows.iter().next().unwrap();
            assert_eq!(flow.algorithm, algorithm_id);
            assert!(
                flow.locals
                    .iter()
                    .any(|(_, local)| local.binder.var == algorithm.params[0].var
                        && local.mutability == LocalMutability::Immutable
                        && !local.ghost)
            );
            if name == "ghost_isolation" {
                let diagnostics = flow.diagnostics.iter().map(|(_, d)| d).collect::<Vec<_>>();
                assert_eq!(diagnostics.len(), 1);
                let D::GhostIsolationViolation { local, var } = diagnostics[0].kind else {
                    panic!("wrong static error")
                };
                assert!(flow.locals.get(local).unwrap().ghost);
                assert_eq!(var, flow.locals.get(local).unwrap().binder.var);
                assert_eq!(diagnostics[0].statement, Some(statements[1].0));
                let S::Let {
                    value: Some(read), ..
                } = statements[1].1.kind
                else {
                    unreachable!()
                };
                assert_eq!(
                    diagnostics[0].source,
                    core.terms().get(read).unwrap().source
                );
            } else {
                assert!(
                    flow.diagnostics
                        .iter()
                        .any(|(_, d)| d.kind == D::IllegalBreak
                            && d.statement == Some(statements[0].0)
                            && d.source == statements[0].1.source)
                );
                assert!(flow.diagnostics.iter().any(|(_, d)| matches!(
                    d.kind,
                    D::UnreachableStatement { .. }
                ) && d.statement
                    == Some(statements[1].0)));
            }
            let keys = super::super::type_elaboration_detail_keys(
                &config.workspace_root,
                &case,
                super::super::formula_statement::step5c8_test_frontend(source_text),
                &mut None,
            );
            assert_eq!(
                keys,
                vec![case.expectation.stable_detail_key.clone().unwrap()]
            );
        }
    }
}

#[test]
fn step5c14_static_ghost_checks_actual_sinks_and_preserves_shadowing() {
    use mizar_core::control_flow::{ControlFlowDiagnosticKind as D, build_control_flow_ir};
    use mizar_core::core_ir::{CoreAlgorithmStmtKind as S, CoreTermKind};
    let case = step5c14_static_case("ghost_isolation");
    let text = std::fs::read_to_string(&case.source_path).unwrap();
    for source in [
        text.replace("var x := g", "var x := a"),
        text.replace("ghost var", "var"),
        text.replace(
            "ghost var g := a;\n    var x := g;\n    return x;",
            "var a := a;\n    return a;",
        ),
    ] {
        let core = step5c14_static_core(&case, &source).unwrap();
        let flow = build_control_flow_ir(&core);
        assert!(flow.flows.iter().all(|(_, f)| f.diagnostics.is_empty()));
        assert!(core.obligation_seeds().is_empty());
        let keys = super::super::type_elaboration_detail_keys(
            &step5c11_config().workspace_root,
            &case,
            super::super::formula_statement::step5c8_test_frontend(&source),
            &mut None,
        );
        assert!(keys.is_empty(), "safe source observations: {keys:?}");
        if source.contains("var a := a") {
            let (_, algorithm) = core.algorithms().iter().next().unwrap();
            let S::Let {
                binder,
                value: Some(initial),
                ..
            } = &core
                .algorithm_statements()
                .get(algorithm.statements[0])
                .unwrap()
                .kind
            else {
                panic!("missing shadow")
            };
            assert_ne!(binder.var, algorithm.params[0].var);
            assert_eq!(
                core.terms().get(*initial).unwrap().kind,
                CoreTermKind::Var(algorithm.params[0].var)
            );
            let S::Return(Some(ret)) = core
                .algorithm_statements()
                .get(algorithm.statements[1])
                .unwrap()
                .kind
            else {
                panic!("missing shadow return")
            };
            assert_eq!(
                core.terms().get(ret).unwrap().kind,
                CoreTermKind::Var(binder.var)
            );
        }
    }
    let repaired_case = step5c14_static_case("break_outside_loop");
    let repaired = std::fs::read_to_string(&repaired_case.source_path)
        .unwrap()
        .replace("    break;\n", "");
    assert!(
        super::super::type_elaboration_detail_keys(
            &step5c11_config().workspace_root,
            &repaired_case,
            super::super::formula_statement::step5c8_test_frontend(&repaired),
            &mut None
        )
        .is_empty()
    );
    for source in [
        text.replace("var x := g;\n    return x;", "return g;"),
        text.replace("var x := g;", "return a;\n    var x := g;"),
    ] {
        let core = step5c14_static_core(&case, &source).unwrap();
        assert!(build_control_flow_ir(&core).flows.iter().any(|(_, flow)| {
            flow.diagnostics
                .iter()
                .any(|(_, d)| matches!(d.kind, D::GhostIsolationViolation { .. }))
        }));
    }
    for (from, to) in [
        ("ghostleak(a)", "ghostleak(g)"),
        ("var g := a", "var g := g"),
        ("var x := g", "var x := x"),
        ("return x", "return absent"),
        ("var x := g", "const x, y := g"),
        ("return x;", "x.field := a;\n return x;"),
        ("return x;", "while a = a do break; end; return x;"),
        ("let a be object", "let a be set"),
        ("-> object", "-> set"),
    ] {
        assert!(
            step5c14_static_core(&case, &text.replace(from, to)).is_err(),
            "{from}->{to}"
        );
        let keys = super::super::type_elaboration_detail_keys(
            &step5c11_config().workspace_root,
            &case,
            super::super::formula_statement::step5c8_test_frontend(&text.replace(from, to)),
            &mut None,
        );
        assert!(
            !keys
                .iter()
                .any(|key| key == "algorithms.ghost.isolation_violation"
                    || key == "algorithms.control_flow.break_outside_loop")
        );
    }
}

#[test]
fn step5c14_static_seal_rejects_projection_environment_and_owner_corruption() {
    use mizar_checker::type_checker::check_source_algorithm_types;
    use mizar_checker::typed_ast::{NodeRecoveryState, TypedArena, TypingState};
    use mizar_resolve::resolved_ast::SurfaceResolvedArena;
    let case = step5c14_static_case("ghost_isolation");
    let text = std::fs::read_to_string(&case.source_path).unwrap();
    for text in [text, "definition let a be object; terminating algorithm ghostalgo(a) -> object ensures result = a do ghost var g := a; snapshot s0; return a; end; end;".to_owned()] {
    let frontend = super::super::formula_statement::step5c8_test_frontend(&text);
    let ast = frontend.ast.clone().unwrap();
    let (source, typed, symbols) =
        super::super::source_registration_inputs(&step5c11_config().workspace_root, &case, frontend)
            .unwrap();
    check_source_algorithm_types(&source, &typed, &symbols).unwrap();
    let raw = typed
        .iter()
        .map(|(_, node)| node.clone())
        .collect::<Vec<_>>();
    let target = raw.iter().position(|node| node.kind.as_str() == "SnapshotStatement").unwrap_or(0);
    for mutation in 0..7 {
        let mut nodes = raw.clone();
        let mut root = typed.root();
        match mutation {
            0 => nodes[target].kind = "Unrelated".into(),
            1 => nodes[target].resolved_node = nodes[1].resolved_node,
            2 => nodes[target].anchor = nodes[1].anchor.clone(),
            3 => nodes[target].recovery = NodeRecoveryState::Recovered,
            4 => nodes[target].typing = TypingState::Successful,
            5 => nodes.last_mut().unwrap().children.clear(),
            6 => root = None,
            _ => unreachable!(),
        }
        let changed = TypedArena::try_new(root, nodes).unwrap();
        assert!(
            check_source_algorithm_types(&source, &changed, &symbols).is_err(),
            "mutation {mutation}"
        );
    }
    let foreign = ResolverModuleId::new(PackageId::new("foreign"), ModulePath::new("foreign"));
    assert!(
        check_source_algorithm_types(
            &SurfaceResolvedArena::lower(&ast, &foreign).unwrap(),
            &typed,
            &symbols
        )
        .is_err()
    );
    let changed = super::super::formula_statement::step5c8_test_frontend(
        &text.replace("ghostleak", "other_algorithm").replace("ghostalgo", "other_algorithm"),
    )
    .ast
    .unwrap();
    let wrong =
        super::super::resolver_symbol_collection(&step5c11_config().workspace_root, &case, &changed);
    assert!(check_source_algorithm_types(&source, &typed, &wrong.env).is_err());
    assert!(step5c14_static_core(&case, &format!("{text}\n{text}")).is_err());
    assert!(step5c14_static_core(&case, &format!("{text} snapshot outside;")).is_err());
    }
}

#[test]
fn step5c14_static_admission_is_exact_and_has_no_other_stage_credit() {
    let config = step5c11_config();
    let plan = build_test_plan(&config).unwrap();
    for name in ["break_outside_loop", "ghost_isolation"] {
        let original = step5c14_static_case(name);
        assert!(super::super::step5c14_static_admitted(
            Some(&config.workspace_root),
            &original
        ));
        for mutation in 0..15 {
            let mut case = original.clone();
            match mutation {
                0 => case.id.0.push_str("_extra"),
                1 => case.expectation.id.0.push_str("_extra"),
                2 => case.source_path = case.source_path.with_file_name("wrong.miz"),
                3 => {
                    case.expectation_path =
                        case.expectation_path.with_file_name("wrong.expect.toml")
                }
                4 => case.expectation.source = PathBuf::from("wrong.miz"),
                5 => case.expectation.stage = crate::staged_model::Stage::ProofVerification,
                6 => {
                    case.expectation.expected_phase =
                        Some(crate::expectation::PipelinePhase::TypeCheck)
                }
                7 => case.expectation.expected_outcome = crate::expectation::ExpectedOutcome::Pass,
                8 => case.expectation.tags.clear(),
                9 => case.expectation.tags.push("extra".into()),
                10 => case.expectation.diagnostic_codes.push("E-UNRELATED".into()),
                11 => case.expectation.stable_detail_key = Some("wrong.key".into()),
                12 => case.expectation.kind = crate::expectation::TestKind::Pass,
                13 => case.expectation.rejection_reason = Some("wrong.reason".into()),
                14 => case.expectation.failure_category = None,
                _ => unreachable!(),
            }
            assert!(
                !super::super::step5c14_static_admitted(Some(&config.workspace_root), &case),
                "mutation {mutation}"
            );
        }
        for (stage, phase, tag) in [
            (
                crate::staged_model::Stage::ParseOnly,
                crate::expectation::PipelinePhase::Parse,
                "active_parse_only",
            ),
            (
                crate::staged_model::Stage::DeclarationSymbol,
                crate::expectation::PipelinePhase::Resolve,
                "active_declaration_symbol",
            ),
            (
                crate::staged_model::Stage::FormulaStatement,
                crate::expectation::PipelinePhase::StatementCheck,
                "active_formula_statement",
            ),
            (
                crate::staged_model::Stage::AdvancedSemantics,
                crate::expectation::PipelinePhase::ClusterResolution,
                "active_advanced_semantics",
            ),
            (
                crate::staged_model::Stage::ProofVerification,
                crate::expectation::PipelinePhase::VcGeneration,
                "active_proof_verification",
            ),
        ] {
            for alias in [false, true] {
                let mut case = original.clone();
                case.expectation.stage = stage;
                case.expectation.expected_phase = Some(phase);
                case.expectation.tags = vec![tag.into()];
                if alias {
                    case.id.0 = "unrelated".into();
                    case.source_path = case.source_path.with_file_name("unrelated.miz");
                    case.expectation_path = case
                        .expectation_path
                        .with_file_name("unrelated.expect.toml");
                }
                assert!(!super::super::is_active_parse_only(&case));
                assert!(!super::super::is_active_declaration_symbol(&case));
                assert!(!super::super::is_active_type_elaboration(&case));
                assert!(!super::super::formula_statement::is_active_formula_statement(
                    &config.workspace_root,
                    &case
                ));
                assert!(!super::super::is_active_proof_verification(&case));
                assert!(!super::super::step5c11_registration_admitted(
                    &config.workspace_root,
                    &case
                ));
                assert!(!super::super::step5c13_overload_admitted(
                    &config.workspace_root,
                    &case
                ));
            }
        }
        let mut missing = plan.clone();
        missing.cases.retain(|case| case.id != original.id);
        let mut duplicate = plan.clone();
        duplicate.cases.push(original);
        for invalid in [missing, duplicate] {
            assert!(
                super::super::validate_active_type_elaboration_tags(&config.workspace_root, &invalid)
                    .iter()
                    .any(|d| d.code.0 == "E-TYPE-ELABORATION-STEP5C14-INVENTORY")
            );
        }
    }
    assert!(super::super::validate_active_type_elaboration_tags(&config.workspace_root, &plan).is_empty());
}

#[test]
fn step5c14_while_real_source_havoc_preservation_exit_and_controls() {
    use mizar_core::control_flow::{build_control_flow_ir, build_obligation_seed_handoff, ControlFlowTerminator, LoopInvariantPlacement};
    use mizar_core::core_ir::{CoreAlgorithmStmtKind as S, CoreFormulaKind as F, CoreTermKind, ObligationSeedKind, ObligationSeedStatus};
    use mizar_vc::vc_ir::{ContextEntryKind, LoopInvariantPhase, PremiseRef, SeedVcMapping, VcFormulaRef, VcGeneratedFormulaShape as G, VcKind, VcProgramValue as V, VcStatus};
    let config = step5c11_config();
    let case = build_test_plan(&config).unwrap().cases.into_iter().find(|case| case.id.0 == "pass_proof_verification_algorithm_while_invariant_001").unwrap();
    let original = std::fs::read_to_string(&case.source_path).unwrap();
    let renamed = original.replace("loopalgo", "renamed").replace("let a be", "let input be").replace("(a)", "(input)").replace("= a", "= input").replace("x", "saved");
    for (text, invariant_parameter, self_write, return_parameter) in [
        (original.clone(), false, false, false),
        (renamed, false, false, false),
        (original.replace("invariant x = x", "invariant x = a"), true, false, false),
        (original.replace("      x := a", "      x := x"), false, true, false),
        (original.replace("return x", "return a"), false, false, true),
    ] {
        let core = step5c14_static_core(&case, &text).unwrap();
        assert_eq!(core, step5c14_static_core(&case, &text).unwrap());
        let (algorithm_id, algorithm) = core.algorithms().iter().next().unwrap();
        let [declaration, loop_statement, returned] = algorithm.statements.as_slice() else { panic!("source statement order") };
        let S::Let { binder, value: Some(initializer), ghost: false } = &core.algorithm_statements().get(*declaration).unwrap().kind else { panic!("initializer") };
        let S::While { condition, invariants, decreasing, body } = &core.algorithm_statements().get(*loop_statement).unwrap().kind else { panic!("while") };
        assert!(decreasing.is_empty());
        let [invariant] = invariants.as_slice() else { panic!("invariant") };
        let [assignment] = body.as_slice() else { panic!("body") };
        assert_eq!(core.algorithm_statements().len(), 4);
        for (_, statement) in core.algorithm_statements().iter() { assert_eq!(statement.owner, algorithm_id); }
        let parameter = V { var: algorithm.params[0].var, definition: None };
        let initial = V { var: binder.var, definition: Some(*declaration) };
        let head = V { var: binder.var, definition: Some(*loop_statement) };
        let written = V { var: binder.var, definition: Some(*assignment) };
        assert_eq!(core.terms().get(*initializer).unwrap().kind, CoreTermKind::Var(parameter.var));
        let S::AssignLocal { target, value } = core.algorithm_statements().get(*assignment).unwrap().kind else { panic!("assignment") };
        assert_eq!(target, binder.var);
        assert_eq!(core.terms().get(value).unwrap().kind, CoreTermKind::Var(if self_write { binder.var } else { parameter.var }));
        let F::Not(guard) = core.formulas().get(*condition).unwrap().kind else { panic!("guard polarity") };
        assert!(matches!(core.formulas().get(guard).unwrap().kind, F::Equals { .. }));
        assert!(matches!(core.formulas().get(*invariant).unwrap().kind, F::Equals { .. }));
        assert!(matches!(core.algorithm_statements().get(*returned).unwrap().kind, S::Return(Some(_))));
        let flow = build_control_flow_ir(&core);
        assert_eq!(flow, build_control_flow_ir(&core));
        let (_, graph) = flow.flows.iter().next().unwrap();
        assert!(graph.diagnostics.is_empty());
        assert_eq!(graph.blocks.len(), 4);
        assert_eq!(graph.loops.len(), 1);
        assert_eq!(graph.assignment_effects.len(), 2);
        assert_eq!(graph.termination.partial_sites.len(), 2);
        let (_, loop_row) = graph.loops.iter().next().unwrap();
        assert_eq!(graph.blocks.get(graph.entry).unwrap().terminator, ControlFlowTerminator::Goto(loop_row.header));
        assert_eq!(graph.blocks.get(loop_row.header).unwrap().terminator, ControlFlowTerminator::Branch { condition: *condition, then_block: loop_row.body, else_block: loop_row.exit });
        assert_eq!(graph.blocks.get(loop_row.body).unwrap().terminator, ControlFlowTerminator::Goto(loop_row.header));
        assert!(matches!(graph.contracts.loop_invariants[0].placement, LoopInvariantPlacement::Header { .. }));
        assert!(matches!(graph.contracts.loop_invariants[1].placement, LoopInvariantPlacement::NormalBackedge { .. }));
        let handoff = build_obligation_seed_handoff(&core, &flow);
        assert_eq!(handoff.entries.len(), 5);
        assert_eq!(handoff.entries.iter().filter(|(_, row)| row.seed.kind == ObligationSeedKind::AlgorithmTermination && row.seed.status == ObligationSeedStatus::Deferred && row.seed.goal.is_none()).count(), 2);
        let vcs = step5c14_return_vcs(&core).unwrap();
        assert_eq!(vcs, step5c14_return_vcs(&core).unwrap());
        assert_eq!(vcs.vcs().len(), 3);
        assert_eq!(vcs.seed_accounting().len(), 5);
        assert_eq!(vcs.seed_accounting().iter().filter(|row| matches!(row.mapping, SeedVcMapping::NoConcreteVc { .. })).count(), 2);
        let shape = |reference| {
            let VcFormulaRef::Generated(id) = reference else { panic!("generated state") };
            &vcs.generated_formula(id).unwrap().shape
        };
        let entry = vcs.vcs().iter().find(|vc| vc.kind == VcKind::LoopInvariant { phase: LoopInvariantPhase::Entry }).unwrap();
        let preservation = vcs.vcs().iter().find(|vc| vc.kind == VcKind::LoopInvariant { phase: LoopInvariantPhase::Preservation }).unwrap();
        let exit = vcs.vcs().iter().find(|vc| vc.kind == VcKind::AlgorithmPostcondition).unwrap();
        assert_eq!(shape(entry.goal), &G::ProgramEquals { left: initial, right: if invariant_parameter { parameter } else { initial } });
        assert_eq!(shape(preservation.goal), &G::ProgramEquals { left: written, right: if invariant_parameter { parameter } else { written } });
        assert_eq!(shape(exit.goal), &G::ProgramEquals { left: if return_parameter { parameter } else { head }, right: parameter });
        assert_eq!(entry.local_context.entries().len(), 3);
        assert_eq!(preservation.local_context.entries().len(), 6);
        assert_eq!(exit.local_context.entries().len(), 4);
        assert!(!entry.local_context.entries().iter().any(|row| row.kind == ContextEntryKind::LoopInvariantAvailable));
        for vc in [preservation, exit] {
            let summary = vc.local_context.entries().iter().find(|row| row.kind == ContextEntryKind::LoopInvariantAvailable).unwrap();
            assert_eq!(shape(summary.formula.unwrap()), &G::ProgramEquals { left: head, right: if invariant_parameter { parameter } else { head } });
        }
        assert!(preservation.local_context.entries().iter().any(|row| row.formula.is_some_and(|formula| matches!(formula, VcFormulaRef::Generated(_)) && shape(formula) == &G::ProgramEquals { left: written, right: if self_write { head } else { parameter } })));
        let guard = preservation.local_context.entries().iter().find(|row| row.kind == ContextEntryKind::AlgorithmPathCondition).unwrap().formula.unwrap();
        let G::Not(equality) = shape(guard) else { panic!("negated head guard") };
        assert_eq!(shape(*equality), &G::ProgramEquals { left: head, right: parameter });
        let exit_guard = exit.local_context.entries().iter().find(|row| row.kind == ContextEntryKind::AlgorithmPathCondition).unwrap().formula.unwrap();
        assert_eq!(shape(exit_guard), &G::Not(guard));
        for row in preservation.local_context.entries().iter().chain(exit.local_context.entries()) {
            if let Some(VcFormulaRef::Generated(id)) = row.formula {
                match &vcs.generated_formula(id).unwrap().shape {
                    G::ProgramEquals { left, right } => { assert_ne!(*left, initial); assert_ne!(*right, initial); }
                    G::ProgramTypePredicate { subject, .. } => assert_ne!(*subject, initial),
                    _ => {}
                }
            }
        }
        for row in exit.local_context.entries() {
            if let Some(VcFormulaRef::Generated(id)) = row.formula
                && let G::ProgramEquals { left, right } = vcs.generated_formula(id).unwrap().shape {
                assert_ne!(left, written); assert_ne!(right, written);
            }
        }
        for vc in vcs.vcs() {
            assert_eq!(vc.status, VcStatus::Open);
            assert!(vc.premises.iter().any(|premise| matches!(premise, PremiseRef::ConservativeUnknown { .. })));
            assert!(vcs.canonical_vc_fingerprint(vc.id).is_none());
        }
        let slices = mizar_vc::dependency_slice::try_compute_dependency_slices(mizar_vc::dependency_slice::DependencySliceInput { vc_set: &vcs, discharge_output: None }).unwrap();
        assert!(slices.slices().iter().all(|slice| !slice.unknowns().is_empty()));
    }
    for text in [
        original.replace("not x = a", "x = a"),
        original.replace("invariant x = x;", "invariant not x = x;"),
        original.replace("invariant x = x;", "invariant x = x; invariant x = a;"),
        original.replace("invariant x = x;", "invariant x = x; decreasing x;"),
        original.replace("      x := a;", "      break;"),
        original.replace("      x := a;", "      continue;"),
        original.replace("      x := a;", "      x := a; x := x;"),
        original.replace("      x := a;", "      while not x = a do invariant x = x; x := a; end;"),
        original.replace("      x := a;", "      x := result;"),
        original.replace("      x := a;", "      x := missing;"),
        original.replace("var x", "ghost var x"),
        original.replace("var x", "const x"),
    ] { assert!(step5c14_static_core(&case, &text).is_err(), "{text}"); }
    let parameter_write = original.replace("      x := a;", "      a := x;");
    let parameter_write = step5c14_static_core(&case, &parameter_write).unwrap();
    assert!(!build_control_flow_ir(&parameter_write).flows.iter().next().unwrap().1.diagnostics.is_empty());
    assert!(step5c14_return_vcs(&parameter_write).is_err());
}

#[test]
fn step5c14_while_rejects_reachable_core_and_seal_corruption() {
    use mizar_core::core_ir::*;
    use mizar_checker::typed_ast::{TypedArena, TypedNodeId};
    let config = step5c11_config();
    let case = step5c14_return_case();
    let text = "definition let a be object; algorithm loopalgo(a) -> object ensures result = a do var x := a; while not x = a do invariant x = x; x := a; end; return x; end; end;";
    let (source, typed, symbols) = super::super::source_registration_inputs(&config.workspace_root, &case, super::super::formula_statement::step5c8_test_frontend(text)).unwrap();
    for kind in ["WhileStatement", "LoopInvariantClause", "AssignmentStatement", "PrefixFormula(Not)"] {
        let mut nodes = typed.iter().map(|(_, node)| node.clone()).collect::<Vec<_>>();
        let index = nodes.iter().position(|node| node.kind.as_str() == kind).unwrap();
        nodes[index].children.reverse();
        let changed = TypedArena::try_new(typed.root(), nodes).unwrap();
        assert!(mizar_checker::type_checker::check_source_algorithm_types(&source, &changed, &symbols).is_err(), "{kind}");
    }
    let mut nodes = typed.iter().map(|(_, node)| node.clone()).collect::<Vec<_>>();
    let index = nodes.iter().position(|node| node.kind.as_str() == "WhileStatement").unwrap();
    nodes[index].resolved_node = Some(source.arena().root());
    let changed = TypedArena::try_new(Some(TypedNodeId::new(source.arena().root().index())), nodes).unwrap();
    assert!(mizar_checker::type_checker::check_source_algorithm_types(&source, &changed, &symbols).is_err());
    let core = step5c14_static_core(&case, text).unwrap();
    let (_, algorithm) = core.algorithms().iter().next().unwrap();
    let loop_statement = algorithm.statements[1];
    let CoreAlgorithmStmtKind::While { condition, invariants, body, .. } = &core.algorithm_statements().get(loop_statement).unwrap().kind else { panic!() };
    let assignment = body[0];
    for mutation in 0..6 {
        let mut parts = CoreIrParts { source_id: core.source_id(), module_id: core.module_id().clone(), items: core.items().clone(), terms: core.terms().clone(), formulas: core.formulas().clone(), definitions: core.definitions().clone(), proofs: core.proofs().clone(), proof_nodes: core.proof_nodes().clone(), algorithms: core.algorithms().clone(), algorithm_statements: core.algorithm_statements().clone(), generated: core.generated().clone(), obligation_seeds: core.obligation_seeds().clone(), source_map: core.source_map().clone(), diagnostics: core.diagnostics().clone() };
        match mutation {
            0 => parts.algorithm_statements.get_mut(assignment).unwrap().owner = CoreAlgorithmId::new(999),
            1 => {
                let row = parts.algorithm_statements.get_mut(assignment).unwrap();
                row.source.provenance.clear();
                parts.source_map.algorithm_sources.insert(assignment, row.source.clone());
            }
            2 => {
                let row = parts.formulas.get_mut(invariants[0]).unwrap();
                row.source = core.formulas().get(*condition).unwrap().source.clone();
                parts.source_map.formula_sources.insert(invariants[0], row.source.clone());
            }
            3 => {
                let CoreFormulaKind::Not(child) = core.formulas().get(*condition).unwrap().kind else { panic!() };
                let row = parts.formulas.get_mut(*condition).unwrap();
                row.kind = core.formulas().get(child).unwrap().kind.clone();
            }
            4 => {
                let CoreAlgorithmStmtKind::While { body, .. } = &mut parts.algorithm_statements.get_mut(loop_statement).unwrap().kind else { panic!() };
                body.push(assignment);
            }
            5 => {
                let CoreAlgorithmStmtKind::AssignLocal { target, .. } = &mut parts.algorithm_statements.get_mut(assignment).unwrap().kind else { panic!() };
                *target = algorithm.params[0].var;
            }
            _ => unreachable!(),
        }
        let changed = CoreIr::try_new(parts);
        if mutation == 0 { assert!(changed.is_err(), "Core ownership mutation {mutation}"); }
        else { let changed = changed.unwrap_or_else(|error| panic!("structural probe {mutation}: {error}")); assert!(step5c14_return_vcs(&changed).is_err(), "source graph mutation {mutation}"); }
    }
}

#[test]
fn step5c14_snapshot_preserves_visible_storage_and_capture_point() {
    use mizar_core::control_flow::{ControlFlowStatementPlacement as P, build_control_flow_ir};
    use mizar_core::core_ir::{
        CoreAlgorithmStmtKind as S, CoreFormulaKind as F, CoreTermKind as T, ObligationSeedKind,
        ObligationSeedStatus,
    };
    let config = step5c11_config();
    let case = step5c14_return_case();
    for declaration in ["var", "const", "ghost var", "ghost const"] {
        for shadow in [false, true] {
            let local = if shadow { "a" } else { "g" };
            let writes = if declaration.ends_with("var") {
                format!("{local} := a;")
            } else {
                String::new()
            };
            let text = format!(
                "definition let a be object; terminating algorithm renamed(a) -> object ensures result = a do {declaration} {local} := a; {writes} snapshot before; var later := a; {writes} snapshot after; return a; end; end;"
            );
            let (source, typed, symbols) = super::super::source_registration_inputs(
                &config.workspace_root,
                &case,
                super::super::formula_statement::step5c8_test_frontend(&text),
            )
            .unwrap();
            let checked = mizar_checker::type_checker::check_source_algorithm_types(
                &source, &typed, &symbols,
            )
            .unwrap();
            assert_eq!(checked.snapshots().len(), 2);
            let core = mizar_core::elaborator::lower_source_algorithms(&checked).unwrap();
            assert_eq!(core, step5c14_static_core(&case, &text).unwrap());
            let output = build_control_flow_ir(&core);
            assert_eq!(output, build_control_flow_ir(&core));
            let (_, flow) = output.flows.iter().next().unwrap();
            let mut snapshots = Vec::new();
            for (id, stmt) in core.algorithm_statements().iter() {
                if let S::Snapshot { name, captures } = &stmt.kind {
                    let (site, (_, bindings)) = checked
                        .snapshots()
                        .iter()
                        .find(|(_, (n, _))| n == name)
                        .unwrap();
                    assert_eq!(
                        stmt.source.anchor,
                        mizar_core::core_ir::CoreSourceAnchor::SourceRange(
                            match typed.node(*site).unwrap().anchor {
                                mizar_session::SourceAnchor::Range(range) => range,
                                _ => panic!(),
                            }
                        )
                    );
                    assert_eq!(captures.len(), bindings.len());
                    for (var, binding) in captures.iter().zip(bindings) {
                        let declaration = &checked
                            .bindings()
                            .bindings()
                            .get(*binding)
                            .unwrap()
                            .declaration_range;
                        let local = flow
                            .locals
                            .iter()
                            .find(|(_, local)| local.binder.var == *var)
                            .unwrap()
                            .1;
                        assert_eq!(
                            local.binder.source.anchor,
                            mizar_core::core_ir::CoreSourceAnchor::SourceRange(*declaration)
                        );
                    }
                    let P::Snapshot {
                        block,
                        context,
                        captures: locals,
                    } = &flow.source_map.statement_placements[&id]
                    else {
                        panic!()
                    };
                    assert!(flow.blocks.get(*block).unwrap().statements.contains(&id));
                    assert_eq!(
                        *captures,
                        locals
                            .iter()
                            .map(|id| flow.locals.get(*id).unwrap().binder.var)
                            .collect::<Vec<_>>()
                    );
                    let names = locals
                        .iter()
                        .map(|id| {
                            flow.locals
                                .get(*id)
                                .unwrap()
                                .binder
                                .source_name
                                .as_ref()
                                .unwrap()
                                .as_str()
                        })
                        .collect::<Vec<_>>();
                    let expected = match (name.as_str(), shadow) {
                        ("before", false) => vec!["a", "g"],
                        ("before", true) => vec!["a"],
                        ("after", false) => vec!["a", "g", "later"],
                        ("after", true) => vec!["a", "later"],
                        _ => panic!(),
                    };
                    assert_eq!(names, expected);
                    let (_, algorithm) = core.algorithms().iter().next().unwrap();
                    let parameter = algorithm.params[0].var;
                    let F::Equals { right, .. } = core
                        .formulas()
                        .get(algorithm.contracts.ensures[0])
                        .unwrap()
                        .kind
                    else {
                        panic!()
                    };
                    assert_eq!(core.terms().get(right).unwrap().kind, T::Var(parameter));
                    if shadow {
                        let S::Let { binder, .. } = &core
                            .algorithm_statements()
                            .get(algorithm.statements[0])
                            .unwrap()
                            .kind
                        else {
                            panic!()
                        };
                        assert_ne!(binder.var, parameter);
                        assert_eq!(captures[0], binder.var);
                        assert!(!captures.contains(&parameter));
                        assert!(
                            flow.locals
                                .iter()
                                .any(|(_, local)| local.binder.var == parameter)
                        );
                    }

                    let saved = flow.contexts.get(*context).unwrap();
                    assert!(
                        locals
                            .iter()
                            .all(|local| saved.definitely_initialized.contains(local))
                    );
                    snapshots.push((
                        *context,
                        saved.assignment_effects.clone(),
                        saved.available_facts.clone(),
                    ));
                }
            }
            assert_eq!(snapshots.len(), 2);
            assert_ne!(snapshots[0].0, snapshots[1].0);
            let before_count = 1 + usize::from(!writes.is_empty());
            assert_eq!(snapshots[0].1.len(), before_count);
            assert_eq!(
                snapshots[1].1.len(),
                before_count + 1 + usize::from(!writes.is_empty())
            );
            assert_eq!(snapshots[1].1[..before_count], snapshots[0].1);
            if shadow && declaration.starts_with("ghost") {
                assert!(!flow.diagnostics.is_empty());
                assert!(step5c14_return_vcs(&core).is_err());
            } else {
                assert!(flow.diagnostics.is_empty());
                let vcs = step5c14_return_vcs(&core).unwrap();
                assert_eq!(vcs, step5c14_return_vcs(&core).unwrap());
            }
        }
    }
    let original = "definition let a be object; terminating algorithm ghostalgo(a) -> object ensures result = a do ghost var g := a; snapshot s0; return a; end; end;";
    let captured = step5c14_static_core(&case, original).unwrap();
    let uncaptured = step5c14_static_core(&case, &original.replace("snapshot s0;", "")).unwrap();
    assert_eq!(captured.terms().len(), uncaptured.terms().len());
    assert_eq!(captured.formulas().len(), uncaptured.formulas().len());
    let vcs = step5c14_return_vcs(&captured).unwrap();
    let without_snapshot = step5c14_return_vcs(&uncaptured).unwrap();
    assert_eq!(vcs.vcs().len(), 1);
    assert_eq!(vcs.vcs()[0].status, mizar_vc::vc_ir::VcStatus::Open);
    assert_eq!(vcs.seed_accounting().len(), 3);
    let output = build_control_flow_ir(&captured);
    let handoff = mizar_core::control_flow::build_obligation_seed_handoff(&captured, &output);
    let (_, algorithm) = captured.algorithms().iter().next().unwrap();
    let initializer = algorithm.statements[0];
    for row in vcs.seed_accounting() {
        let entry = handoff.entries.get(row.handoff).unwrap();
        assert_eq!(entry.seed.owner, algorithm.item);
        assert_eq!(entry.seed.status, ObligationSeedStatus::Deferred);
        assert_eq!(row.seed_status, entry.seed.status);
        match entry.flow_site.as_ref().unwrap().kind {
            mizar_core::control_flow::ControlFlowObligationSiteKind::GhostAssignment => {
                assert_eq!(entry.seed.kind, ObligationSeedKind::GhostErasure);
                assert_eq!(
                    entry.flow_site.as_ref().unwrap().statement,
                    Some(initializer)
                );
                let source = &captured
                    .algorithm_statements()
                    .get(initializer)
                    .unwrap()
                    .source;
                let mut provenance = source.provenance.clone();
                provenance.push(mizar_core::core_ir::CoreProvenance::new(
                    mizar_core::core_ir::CoreProvenancePhase::Generated,
                    "flow-handoff:ghost-assignment:0",
                ));
                assert_eq!(
                    entry.seed.source,
                    source.clone().with_provenance(provenance)
                );
                assert!(matches!(
                    row.mapping,
                    mizar_vc::vc_ir::SeedVcMapping::NoConcreteVc { .. }
                ));
            }
            mizar_core::control_flow::ControlFlowObligationSiteKind::PartialTermination => {
                assert_eq!(entry.seed.kind, ObligationSeedKind::AlgorithmTermination);
                assert!(matches!(
                    row.mapping,
                    mizar_vc::vc_ir::SeedVcMapping::NoConcreteVc { .. }
                ));
            }
            mizar_core::control_flow::ControlFlowObligationSiteKind::Ensures => {
                assert_eq!(entry.seed.kind, ObligationSeedKind::AlgorithmContract)
            }
            _ => panic!("invented snapshot obligation"),
        }
    }

    assert_eq!(
        vcs.seed_accounting()
            .iter()
            .filter(|row| matches!(
                row.mapping,
                mizar_vc::vc_ir::SeedVcMapping::NoConcreteVc { .. }
            ))
            .count(),
        2
    );
    assert_eq!(
        vcs.generated_formulas().len(),
        without_snapshot.generated_formulas().len()
    );
    assert_eq!(
        vcs.vcs()[0].local_context.entries().len(),
        without_snapshot.vcs()[0].local_context.entries().len()
    );
    for text in [
        original.replace("ghostalgo", "renamed"),
        original
            .replace("let a be", "let parameter be")
            .replace("(a)", "(parameter)")
            .replace("= a", "= parameter")
            .replace("return a;", "return parameter;"),
        original.replace("g :=", "hidden :="),
        original.replace("s0", "state"),
    ] {
        let core = step5c14_static_core(&case, &text).unwrap();
        assert!(step5c14_return_vcs(&core).is_ok());
    }
    for (old, new) in [
        ("snapshot s0;", "snapshot s0; snapshot s0;"),
        ("snapshot s0;", "snapshot ;"),
        ("snapshot s0;", "snapshot s0"),
        ("snapshot s0;", "while a = a do snapshot s0; end;"),
        ("return a;", "return s0.a;"),
    ] {
        assert!(
            step5c14_static_core(&case, &original.replace(old, new)).is_err(),
            "{new}"
        );
    }
}

#[test]
fn step5c14_snapshot_rejects_source_owner_and_hidden_storage_corruption() {
    use mizar_core::core_ir::*;
    let core = step5c14_static_core(&step5c14_return_case(), "definition let a be object; terminating algorithm snap(a) -> object ensures result = a do var a := a; snapshot here; return a; end; end;").unwrap();
    let (_, algorithm) = core.algorithms().iter().next().unwrap();
    let snapshot = algorithm.statements[1];
    assert!(step5c14_return_vcs(&core).is_ok());
    for mutation in 0..4 {
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
        let statement = parts.algorithm_statements.get_mut(snapshot).unwrap();
        match mutation {
            0 => statement.source.provenance.clear(),
            1 => {
                statement.source.anchor = core
                    .algorithm_statements()
                    .get(algorithm.statements[2])
                    .unwrap()
                    .source
                    .anchor
                    .clone()
            }
            2 => statement.owner = CoreAlgorithmId::new(999),
            3 => {
                let CoreAlgorithmStmtKind::Snapshot { captures, .. } = &mut statement.kind else {
                    panic!()
                };
                captures.insert(0, algorithm.params[0].var);
            }
            _ => unreachable!(),
        }
        parts
            .source_map
            .algorithm_sources
            .insert(snapshot, statement.source.clone());
        let changed = CoreIr::try_new(parts);
        if mutation < 2 {
            let changed = changed.unwrap_or_else(|error| {
                panic!("valid enclosing snapshot source probe {mutation}: {error}")
            });
            assert!(
                step5c14_return_vcs(&changed).is_err(),
                "snapshot source mutation {mutation}"
            );
        } else {
            assert!(
                changed.is_err(),
                "snapshot owner/hidden storage mutation {mutation}"
            );
        }
    }
}

#[test]
fn step5c14_return_source_preserves_contract_and_substitutes_actual_return() {
    use mizar_core::core_ir::{
        CoreAlgorithmStmtKind as S, CoreFormulaKind as F, CoreTermKind as T, ObligationSeedStatus,
    };
    use mizar_vc::vc_ir::{
        ContextEntryKind, SeedVcMapping, VcFormulaRef, VcGeneratedFormulaKind,
        VcGeneratedFormulaShape, VcKind, VcStatus,
    };
    let case = step5c14_return_case();
    let text = std::fs::read_to_string(&case.source_path).unwrap();
    for (source, operands) in [
        (text.clone(), ["result", "a"]),
        (
            text.replace("idalgo", "renamed")
                .replace("let a be", "let other be")
                .replace("(a)", "(other)")
                .replace("= a", "= other")
                .replace("return a;", "return other;"),
            ["result", "other"],
        ),
        (text.replace("result = a", "a = result"), ["a", "result"]),
        (
            text.replace("result = a", "result = result"),
            ["result", "result"],
        ),
        (text.replace("result = a", "a = a"), ["a", "a"]),
    ] {
        use mizar_checker::binding_env::{
            BinderIdentity, BindingContextOwner, BindingKind, BindingTypeSite,
        };
        use mizar_session::SourceAnchor;
        let (resolved, typed, symbols) = super::super::source_registration_inputs(
            &step5c11_config().workspace_root,
            &case,
            super::super::formula_statement::step5c8_test_frontend(&source),
        )
        .unwrap();
        let checked =
            mizar_checker::type_checker::check_source_algorithm_types(&resolved, &typed, &symbols)
                .unwrap();
        let (result_id, generated) = checked
            .bindings()
            .bindings()
            .iter()
            .find(|(_, b)| b.kind == BindingKind::Generated)
            .unwrap();
        assert!(
            matches!(generated.identity, BinderIdentity::Generated { context, .. } if context == generated.owner_context)
        );
        let type_start = source.find("-> object").unwrap() + 3;
        assert_eq!(
            (
                generated.declaration_range.start,
                generated.declaration_range.end
            ),
            (type_start, type_start + "object".len())
        );
        assert_eq!(
            generated.type_site,
            BindingTypeSite::Source(generated.declaration_range)
        );
        let context = checked
            .bindings()
            .contexts()
            .get(generated.owner_context)
            .unwrap();
        assert!(
            matches!(context.owner, BindingContextOwner::SourceFormula { source_range } if source_range.start == source.find("ensures ").unwrap())
        );
        assert!(
            context.bindings.contains(&result_id) && context.visible_bindings.contains(&result_id)
        );
        for (_, other) in checked.bindings().contexts().iter() {
            if other.id != generated.owner_context {
                assert!(!other.visible_bindings.contains(&result_id));
            }
        }
        let core = mizar_core::elaborator::lower_source_algorithms(&checked).unwrap();
        let replay = step5c14_static_core(&case, &source).unwrap();
        assert_eq!(core, replay);
        assert!(
            core.diagnostics().is_empty()
                && core.obligation_seeds().is_empty()
                && core.proofs().is_empty()
        );
        let (_, algorithm) = core.algorithms().iter().next().unwrap();
        assert_eq!(core.algorithms().len(), 1);
        assert_eq!(algorithm.params.len(), 1);
        let parameter = &algorithm.params[0];
        let result = algorithm.result.as_ref().unwrap();
        assert_ne!(parameter.var, result.var);
        assert_eq!(algorithm.contracts.ensures.len(), 1);
        let contract = algorithm.contracts.ensures[0];
        let F::Equals { left, right } = core.formulas().get(contract).unwrap().kind else {
            panic!("actual equality missing")
        };
        let [statement] = algorithm.statements.as_slice() else {
            panic!("one return required")
        };
        let S::Return(Some(returned)) = core.algorithm_statements().get(*statement).unwrap().kind
        else {
            panic!("actual return missing")
        };
        assert_eq!(
            core.terms().get(returned).unwrap().kind,
            T::Var(parameter.var)
        );
        let formula_start = source.find("ensures ").unwrap() + "ensures ".len();
        let right_start = formula_start + operands[0].len() + " = ".len();
        for ((term, spelling), offset) in [left, right]
            .into_iter()
            .zip(operands)
            .zip([formula_start, right_start])
        {
            let actual = core.terms().get(term).unwrap();
            assert_eq!(
                actual.kind,
                T::Var(if spelling == "result" {
                    result.var
                } else {
                    parameter.var
                })
            );
            let mizar_core::core_ir::CoreSourceAnchor::SourceRange(range) = actual.source.anchor
            else {
                panic!("operand source missing")
            };
            assert_eq!((range.start, range.end), (offset, offset + spelling.len()));
            let checked_term = checked
                .inference()
                .terms()
                .iter()
                .find(|(_, term)| {
                    typed.node(term.site.node()).unwrap().anchor == SourceAnchor::Range(range)
                })
                .unwrap()
                .1;
            let Some(mizar_checker::type_checker::TermReference::Binding(binding)) =
                checked_term.reference
            else {
                panic!("checked binding missing")
            };
            assert_eq!(
                checked.bindings().bindings().get(binding).unwrap().kind,
                if spelling == "result" {
                    BindingKind::Generated
                } else {
                    BindingKind::DefinitionParameter
                }
            );
        }
        assert_eq!(
            result.source.anchor,
            mizar_core::core_ir::CoreSourceAnchor::SourceRange(generated.declaration_range)
        );
        let (type_node, _) = typed
            .iter()
            .find(|(_, node)| {
                node.kind.as_str() == "TypeExpression"
                    && node.anchor == SourceAnchor::Range(generated.declaration_range)
            })
            .unwrap();
        assert!(result.source.provenance.iter().any(|p| p.phase
            == mizar_core::core_ir::CoreProvenancePhase::Checker
            && p.key.as_str() == format!("algorithm/source-node#{}", type_node.index())));
        let vcs = step5c14_return_vcs(&core).unwrap();
        assert_eq!(vcs, step5c14_return_vcs(&replay).unwrap());
        assert_eq!(
            vcs.debug_text(),
            step5c14_return_vcs(&replay).unwrap().debug_text()
        );
        assert_eq!(vcs.vcs().len(), 1);
        assert_eq!(vcs.generated_formulas().len(), 1);
        let vc = &vcs.vcs()[0];
        assert_eq!(vc.kind, VcKind::AlgorithmPostcondition);
        assert_eq!(vc.status, VcStatus::Open);
        assert!(!vc.anchor.is_complete());
        let VcFormulaRef::Generated(goal) = vc.goal else {
            panic!("concrete VC goal missing")
        };
        let goal = vcs.generated_formula(goal).unwrap();
        assert_eq!(goal.kind, VcGeneratedFormulaKind::AlgorithmPostcondition);
        let expected = |id| {
            if core.terms().get(id).unwrap().kind == T::Var(result.var) {
                returned
            } else {
                id
            }
        };
        assert_eq!(
            goal.shape,
            VcGeneratedFormulaShape::Equals {
                left: expected(left),
                right: expected(right)
            }
        );
        assert!(!goal.provenance.is_empty());
        let [context] = vc.local_context.entries() else {
            panic!("exact parameter context required")
        };
        assert_eq!(context.kind, ContextEntryKind::CheckerFact);
        assert_eq!(
            context.formula,
            Some(VcFormulaRef::Core(parameter.ty_guard.unwrap()))
        );
        assert!(!context.provenance.is_empty());
        let F::TypePred { subject, ref ty } = core
            .formulas()
            .get(parameter.ty_guard.unwrap())
            .unwrap()
            .kind
        else {
            panic!("parameter type missing")
        };
        assert_eq!(ty.as_str(), "object");
        assert_eq!(
            core.terms().get(subject).unwrap().kind,
            T::Var(parameter.var)
        );
        let flow = mizar_core::control_flow::build_control_flow_ir(&core);
        let handoff = mizar_core::control_flow::build_obligation_seed_handoff(&core, &flow);
        assert_eq!(vcs.seed_accounting().len(), 2);
        let mut postconditions = 0;
        let mut metadata = 0;
        for row in vcs.seed_accounting() {
            let entry = handoff.entries.get(row.handoff).unwrap();
            assert_eq!(row.seed_status, entry.seed.status);
            assert_eq!(row.seed_status, ObligationSeedStatus::Deferred);
            match row.mapping {
                SeedVcMapping::One { vc: id } => {
                    postconditions += 1;
                    assert_eq!(id, vc.id);
                    assert_eq!(entry.seed.goal, Some(contract));
                }
                SeedVcMapping::NoConcreteVc { .. } => {
                    metadata += 1;
                    assert!(entry.seed.goal.is_none());
                }
                _ => panic!("unexpected accounting"),
            }
        }
        assert_eq!((postconditions, metadata), (1, 1));
        if source == text {
            assert_eq!(
                std::fs::read_to_string(
                    step5c11_config()
                        .workspace_root
                        .join("tests")
                        .join(case.expectation.snapshots.as_ref().unwrap())
                )
                .unwrap(),
                vcs.debug_text()
            );
        }
    }
}

#[test]
fn step5c14_return_no_contract_is_zero_vc_and_unsupported_source_is_rejected() {
    let case = step5c14_return_case();
    let text = std::fs::read_to_string(&case.source_path).unwrap();
    let without = text.replace("    ensures result = a\n", "");
    assert_ne!(text, without);
    let core = step5c14_static_core(&case, &without).unwrap();
    let vcs = step5c14_return_vcs(&core).unwrap();
    assert_eq!(
        vcs,
        step5c14_return_vcs(&step5c14_static_core(&case, &without).unwrap()).unwrap()
    );
    assert!(vcs.vcs().is_empty() && vcs.generated_formulas().is_empty());
    assert_eq!(vcs.seed_accounting().len(), 1);
    assert!(matches!(
        vcs.seed_accounting()[0].mapping,
        mizar_vc::vc_ir::SeedVcMapping::NoConcreteVc { .. }
    ));
    assert_eq!(
        std::fs::read_to_string(
            step5c11_config()
                .workspace_root
                .join("tests")
                .join("snapshots/vc/step5c14_algorithm_no_ensures.vc_ir.snap")
        )
        .unwrap(),
        vcs.debug_text()
    );
    for source in [
        text.replace("return a;", "return result;"),
        text.replace("result = a", "result = missing"),
        text.replace("result = a", "result in a"),
        text.replace("result = a", "not result = a"),
        text.replace("result = a", "result = a ensures result = a"),
        text.replace("    ensures", "    requires a = a\n    ensures"),
        text.replace("return a;", "var x := missing; return x;"),
        text.replace("return a;", "return a; return a;"),
        text.replace("return a;", "assert a in a; return a;"),
        text.replace("return a;", "return missing;"),
        text.replace("-> object", "-> set"),
    ] {
        assert!(
            step5c14_static_core(&case, &source).is_err(),
            "unsupported source: {source}"
        );
    }
}

#[test]
fn step5c14_return_rejects_core_binding_guard_owner_and_reference_corruption() {
    use mizar_core::core_ir::*;
    let case = step5c14_return_case();
    let core =
        step5c14_static_core(&case, &std::fs::read_to_string(&case.source_path).unwrap()).unwrap();
    let (algorithm_id, algorithm) = core.algorithms().iter().next().unwrap();
    let result = algorithm.result.as_ref().unwrap();
    let parameter = &algorithm.params[0];
    let ensure = algorithm.contracts.ensures[0];
    let statement = algorithm.statements[0];
    let CoreAlgorithmStmtKind::Return(Some(returned)) =
        core.algorithm_statements().get(statement).unwrap().kind
    else {
        panic!()
    };
    for mutation in 0..22 {
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
            0 => {
                parts
                    .algorithms
                    .get_mut(algorithm_id)
                    .unwrap()
                    .result
                    .as_mut()
                    .unwrap()
                    .var = parameter.var
            }
            1 => {
                parts.algorithms.get_mut(algorithm_id).unwrap().params[0].ty_guard = result.ty_guard
            }
            2 => parts
                .algorithms
                .get_mut(algorithm_id)
                .unwrap()
                .contracts
                .ensures
                .push(ensure),
            3 => parts
                .algorithms
                .get_mut(algorithm_id)
                .unwrap()
                .statements
                .push(statement),
            4 => parts.formulas.get_mut(ensure).unwrap().kind = CoreFormulaKind::True,
            5 => parts.terms.get_mut(returned).unwrap().kind = CoreTermKind::Var(result.var),
            6 => {
                parts
                    .formulas
                    .get_mut(parameter.ty_guard.unwrap())
                    .unwrap()
                    .kind = CoreFormulaKind::TypePred {
                    subject: returned,
                    ty: CoreTypePredicate::new("set"),
                }
            }
            7 => parts.algorithms.get_mut(algorithm_id).unwrap().item = CoreItemId::new(999),
            8 => parts
                .algorithm_statements
                .get_mut(statement)
                .unwrap()
                .source
                .provenance
                .clear(),
            9 => parts
                .formulas
                .get_mut(ensure)
                .unwrap()
                .source
                .provenance
                .clear(),
            10 => parts
                .algorithms
                .get_mut(algorithm_id)
                .unwrap()
                .result
                .as_mut()
                .unwrap()
                .source
                .provenance
                .clear(),
            11 => {
                parts
                    .algorithms
                    .get_mut(algorithm_id)
                    .unwrap()
                    .result
                    .as_mut()
                    .unwrap()
                    .ty_guard = parameter.ty_guard
            }
            12 => {
                parts.algorithms.get_mut(algorithm_id).unwrap().params[0].role = "local:var".into()
            }
            13 => {
                parts.source_map.term_sources.remove(&returned);
            }
            14 => {
                parts.source_map.formula_sources.remove(&ensure);
            }
            15 => {
                let CoreFormulaKind::Equals { left, right } =
                    parts.formulas.get(ensure).unwrap().kind
                else {
                    panic!()
                };
                parts.formulas.get_mut(ensure).unwrap().kind = CoreFormulaKind::Equals {
                    left: right,
                    right: left,
                };
            }
            16 => {
                let CoreFormulaKind::Equals { left, right } =
                    parts.formulas.get(ensure).unwrap().kind
                else {
                    panic!()
                };
                parts.terms.get_mut(left).unwrap().source =
                    parts.terms.get(right).unwrap().source.clone();
                parts
                    .source_map
                    .term_sources
                    .insert(left, parts.terms.get(left).unwrap().source.clone());
            }
            17 => {
                let id = parts
                    .terms
                    .insert(core.terms().get(returned).unwrap().clone());
                parts
                    .source_map
                    .term_sources
                    .insert(id, parts.terms.get(id).unwrap().source.clone());
            }
            18 => {
                let id = parts
                    .formulas
                    .insert(core.formulas().get(ensure).unwrap().clone());
                parts
                    .source_map
                    .formula_sources
                    .insert(id, parts.formulas.get(id).unwrap().source.clone());
            }
            19 => {
                let id = parts
                    .algorithm_statements
                    .insert(core.algorithm_statements().get(statement).unwrap().clone());
                parts.source_map.algorithm_sources.insert(
                    id,
                    parts.algorithm_statements.get(id).unwrap().source.clone(),
                );
            }
            20 => {
                parts.diagnostics.insert(CoreDiagnostic::error(
                    CoreDiagnosticClass::AlgorithmShell,
                    "algorithm.test_corruption",
                    algorithm.source.clone(),
                ));
            }
            21 => {
                let CoreFormulaKind::TypePred { subject, .. } =
                    parts.formulas.get(result.ty_guard.unwrap()).unwrap().kind
                else {
                    panic!()
                };
                parts
                    .formulas
                    .get_mut(result.ty_guard.unwrap())
                    .unwrap()
                    .kind = CoreFormulaKind::TypePred {
                    subject,
                    ty: CoreTypePredicate::new("set"),
                };
            }
            _ => unreachable!(),
        }
        if mutation == 8 {
            parts.source_map.algorithm_sources.insert(
                statement,
                parts
                    .algorithm_statements
                    .get(statement)
                    .unwrap()
                    .source
                    .clone(),
            );
        }
        if mutation == 9 {
            parts
                .source_map
                .formula_sources
                .insert(ensure, parts.formulas.get(ensure).unwrap().source.clone());
        }
        let invalid = CoreIr::try_new(parts);
        if matches!(mutation, 7 | 13 | 14) {
            assert!(
                invalid.is_err(),
                "Core must reject invalid owner/map {mutation}"
            );
        } else {
            let invalid =
                invalid.unwrap_or_else(|error| panic!("structural Core probe {mutation}: {error}"));
            assert!(
                step5c14_return_vcs(&invalid).is_err(),
                "VC corruption {mutation}"
            );
        }
    }
}

#[test]
fn step5c14_return_admission_requires_exact_snapshot_trace_and_stage() {
    use crate::expectation::{ExpectedOutcome, PipelinePhase, TestKind};
    use crate::staged_model::Stage;
    let config = step5c11_config();
    let plan = build_test_plan(&config).unwrap();
    for original in [
        step5c14_return_case(),
        step5c14_state_case(),
        step5c14_claim_case(),
        step5c14_assert_failure_case(),
        plan.cases.iter().find(|case| case.id.0 == "fail_proof_verification_algorithm_ensures_unprovable_001").unwrap().clone(),
        plan.cases
            .iter()
            .find(|case| case.id.0 == "pass_proof_verification_algorithm_ghost_snapshot_001")
            .unwrap()
            .clone(),
        plan.cases
            .iter()
            .find(|case| case.id.0 == "pass_proof_verification_algorithm_while_invariant_001")
            .unwrap()
            .clone(),
        plan.cases
            .iter()
            .find(|case| case.id.0 == "pass_proof_verification_computation_justification_001")
            .unwrap()
            .clone(),
    ] {
        assert!(super::super::proof_verification::step5c14_return_admitted(
            Some(&config.workspace_root),
            &original
        ));
        assert!(
            crate::expectation::validate_expectation_path(
                &original.expectation_path,
                &original.expectation,
                &config.workspace_root.join("tests")
            )
            .is_empty()
        );
        for mutation in 0..21 {
            let mut case = original.clone();
            match mutation {
                0 => case.id.0.push_str("_extra"),
                1 => case.expectation.id.0.push_str("_extra"),
                2 => case.source_path = case.source_path.with_file_name("wrong.miz"),
                3 => {
                    case.expectation_path =
                        case.expectation_path.with_file_name("wrong.expect.toml")
                }
                4 => case.expectation.source = PathBuf::from("wrong.miz"),
                5 => case.expectation.stage = Stage::TypeElaboration,
                6 => {
                    case.expectation.expected_phase = Some(
                        if original.expectation.expected_phase == Some(PipelinePhase::Verification)
                        {
                            PipelinePhase::VcGeneration
                        } else {
                            PipelinePhase::Verification
                        },
                    )
                }
                7 => {
                    case.expectation.expected_outcome =
                        if original.expectation.expected_outcome == ExpectedOutcome::Fail {
                            ExpectedOutcome::Pass
                        } else {
                            ExpectedOutcome::Fail
                        }
                }
                8 => case.expectation.tags.clear(),
                9 => case.expectation.tags.push("extra".into()),
                10 => case.expectation.diagnostic_codes.push("E-UNRELATED".into()),
                11 => case.expectation.stable_detail_key = Some("wrong.key".into()),
                12 => {
                    case.expectation.kind = if original.expectation.kind == TestKind::Fail {
                        TestKind::Pass
                    } else {
                        TestKind::Fail
                    }
                }
                13 => case.expectation.rejection_reason = Some("wrong.reason".into()),
                14 => case.expectation.failure_category = Some("wrong_category".into()),
                15 => case.expectation.snapshots = None,
                16 => case.expectation.snapshots = Some(PathBuf::from("snapshots/wrong.snap")),
                17 => {
                    case.expectation.spec_refs.pop();
                }
                18 => case.expectation.spec_refs[1].0.push_str("_wrong"),
                19 => case.expectation.spec_refs.reverse(),
                20 => case.expectation.domain = "wrong.domain".into(),
                _ => unreachable!(),
            }
            assert!(
                !super::super::proof_verification::step5c14_return_admitted(
                    Some(&config.workspace_root),
                    &case
                ),
                "mutation {mutation}"
            );
            if !matches!(mutation, 0 | 2 | 15) {
                assert!(
                    crate::expectation::validate_expectation_path(
                        &case.expectation_path,
                        &case.expectation,
                        &config.workspace_root.join("tests")
                    )
                    .iter()
                    .any(|diagnostic| diagnostic.code.0 == "E-EXPECT-SNAPSHOT-SCOPE"),
                    "snapshot scope mutation {mutation}"
                );
            }
        }
        for (stage, phase, tag) in [
            (Stage::ParseOnly, PipelinePhase::Parse, "active_parse_only"),
            (
                Stage::DeclarationSymbol,
                PipelinePhase::Resolve,
                "active_declaration_symbol",
            ),
            (
                Stage::TypeElaboration,
                PipelinePhase::TypeCheck,
                "active_type_elaboration",
            ),
            (
                Stage::FormulaStatement,
                PipelinePhase::StatementCheck,
                "active_formula_statement",
            ),
            (
                Stage::AdvancedSemantics,
                PipelinePhase::ClusterResolution,
                "active_advanced_semantics",
            ),
        ] {
            for alias in [false, true] {
                let mut case = original.clone();
                case.expectation.stage = stage;
                case.expectation.expected_phase = Some(phase);
                case.expectation.tags = vec![tag.into()];
                if alias {
                    case.id.0 = "unrelated".into();
                    case.source_path = case.source_path.with_file_name("unrelated.miz");
                    case.expectation_path = case
                        .expectation_path
                        .with_file_name("unrelated.expect.toml");
                }
                assert!(!super::super::is_active_parse_only(&case));
                assert!(!super::super::is_active_declaration_symbol(&case));
                assert!(!super::super::is_active_type_elaboration(&case));
                assert!(!super::super::formula_statement::is_active_formula_statement(
                    &config.workspace_root,
                    &case
                ));
                assert!(!super::super::is_active_proof_verification(&case));
                assert!(!super::super::step5c11_registration_admitted(
                    &config.workspace_root,
                    &case
                ));
                assert!(!super::super::step5c13_overload_admitted(
                    &config.workspace_root,
                    &case
                ));
            }
        }
        let run = super::super::proof_verification::run_proof_verification_case(
            &config.workspace_root,
            &config.workspace_root.join(&config.tests_root),
            &original,
            999,
        );
        assert_eq!(
            run.status,
            super::super::ProofVerificationCaseStatus::Passed,
            "{:?}",
            run.failure
        );
        let temporary = std::process::Command::new("mktemp")
            .arg("-d")
            .output()
            .unwrap();
        assert!(temporary.status.success());
        let root = PathBuf::from(String::from_utf8(temporary.stdout).unwrap().trim());
        let missing_snapshot = super::super::proof_verification::run_proof_verification_case(
            &config.workspace_root,
            &root,
            &original,
            0,
        );
        assert_eq!(
            missing_snapshot.status,
            super::super::ProofVerificationCaseStatus::Failed
        );
        assert!(
            missing_snapshot
                .failure
                .unwrap()
                .contains("could not be read")
        );
        let snapshot = root.join(original.expectation.snapshots.as_ref().unwrap());
        std::fs::create_dir_all(snapshot.parent().unwrap()).unwrap();
        std::fs::write(&snapshot, "incorrect snapshot").unwrap();
        let wrong_snapshot = super::super::proof_verification::run_proof_verification_case(
            &config.workspace_root,
            &root,
            &original,
            0,
        );
        assert_eq!(
            wrong_snapshot.status,
            super::super::ProofVerificationCaseStatus::Failed
        );
        assert!(
            wrong_snapshot
                .failure
                .unwrap()
                .contains("snapshot differed")
        );
        std::fs::remove_dir_all(&root).unwrap();
        let mut missing = plan.clone();
        missing.cases.retain(|case| case.id != original.id);
        let mut duplicate = plan.clone();
        duplicate.cases.push(original);
        for invalid in [missing, duplicate] {
            assert!(
                super::super::validate_active_proof_verification_tags(&config.workspace_root, &invalid)
                    .iter()
                    .any(|d| d.code.0 == "E-PROOF-VERIFICATION-STEP5C14-INVENTORY")
            );
        }
    }
}

#[test]
fn step5c14_state_source_preserves_write_values_and_pending_assertion() {
    use mizar_core::core_ir::{
        CoreAlgorithmStmtKind as S, CoreTermKind as T, ObligationSeedStatus,
    };
    use mizar_vc::vc_ir::{
        ContextEntryKind as C, SeedVcMapping, VcFormulaRef, VcGeneratedFormulaKind as K,
        VcGeneratedFormulaShape as F, VcKind, VcProgramValue as V, VcStatus,
    };
    let case = step5c14_state_case();
    let text = std::fs::read_to_string(&case.source_path).unwrap();
    let renamed = text
        .replace("swapalgo", "copyalgo")
        .replace("let a be", "let input be")
        .replace("(a)", "(input)")
        .replace("= a", "= input")
        .replace(":= a", ":= input")
        .replace(" x", " local")
        .replace("const c", "const saved")
        .replace(":= c;", ":= saved;");
    for source in [&text, &renamed] {
        let core = step5c14_static_core(&case, source).unwrap();
        let replay = step5c14_static_core(&case, source).unwrap();
        assert_eq!(core, replay);
        assert!(core.diagnostics().is_empty() && core.proofs().is_empty());
        let (_, algorithm) = core.algorithms().iter().next().unwrap();
        let [x_decl, c_decl, assignment, assertion, returned] = algorithm.statements.as_slice()
        else {
            panic!("five actual statements")
        };
        let param = V {
            var: algorithm.params[0].var,
            definition: None,
        };
        let S::Let {
            binder: x,
            value: Some(x_initial),
            ghost: false,
        } = &core.algorithm_statements().get(*x_decl).unwrap().kind
        else {
            panic!("var declaration")
        };
        let S::Let {
            binder: c,
            value: Some(c_initial),
            ghost: false,
        } = &core.algorithm_statements().get(*c_decl).unwrap().kind
        else {
            panic!("const declaration")
        };
        assert_eq!(x.role.as_str(), "local:var");
        assert_eq!(c.role.as_str(), "local:const");
        assert_ne!(x.var, c.var);
        assert_ne!(x.var, param.var);
        assert_ne!(c.var, param.var);
        for initial in [x_initial, c_initial] {
            assert_eq!(core.terms().get(*initial).unwrap().kind, T::Var(param.var));
        }
        let S::AssignLocal { target, value } =
            core.algorithm_statements().get(*assignment).unwrap().kind
        else {
            panic!("typed assignment")
        };
        assert_eq!(target, x.var);
        assert_eq!(core.terms().get(value).unwrap().kind, T::Var(c.var));
        let S::Assert { formula: asserted } =
            core.algorithm_statements().get(*assertion).unwrap().kind
        else {
            panic!("actual assertion")
        };
        assert!(!algorithm.contracts.ensures.contains(&asserted));
        assert_eq!(algorithm.contracts.ensures.len(), 1);
        let S::Return(Some(value)) = core.algorithm_statements().get(*returned).unwrap().kind
        else {
            panic!("return")
        };
        assert_eq!(core.terms().get(value).unwrap().kind, T::Var(x.var));
        let flow = mizar_core::control_flow::build_control_flow_ir(&core);
        assert!(
            flow.flows
                .iter()
                .all(|(_, flow)| flow.diagnostics.is_empty())
        );
        let handoff = mizar_core::control_flow::build_obligation_seed_handoff(&core, &flow);
        let x0 = V {
            var: x.var,
            definition: Some(*x_decl),
        };
        let c0 = V {
            var: c.var,
            definition: Some(*c_decl),
        };
        let x1 = V {
            var: x.var,
            definition: Some(*assignment),
        };
        let vcs = step5c14_state_vcs(&core).unwrap();
        assert_eq!(vcs, step5c14_state_vcs(&replay).unwrap());
        assert_eq!(
            vcs.debug_text(),
            step5c14_state_vcs(&replay).unwrap().debug_text()
        );
        let actual_equations = vcs
            .generated_formulas()
            .iter()
            .filter_map(|formula| {
                if formula.kind != K::AlgorithmStateFact {
                    return None;
                }
                match formula.shape {
                    F::ProgramEquals { left, right } => Some((left, right)),
                    _ => None,
                }
            })
            .collect::<std::collections::BTreeSet<_>>();
        assert_eq!(
            actual_equations,
            std::collections::BTreeSet::from([(x0, param), (c0, param), (x1, c0)])
        );
        assert_eq!(vcs.vcs().len(), 2);
        let assertion_vc = vcs
            .vcs()
            .iter()
            .find(|vc| vc.kind == VcKind::AlgorithmAssertion)
            .unwrap();
        let return_vc = vcs
            .vcs()
            .iter()
            .find(|vc| vc.kind == VcKind::AlgorithmPostcondition)
            .unwrap();
        for vc in [assertion_vc, return_vc] {
            assert_eq!(vc.status, VcStatus::Open);
            assert!(!vc.anchor.is_complete());
            let VcFormulaRef::Generated(id) = vc.goal else {
                panic!("generated state goal")
            };
            assert_eq!(
                vcs.generated_formula(id).unwrap().shape,
                F::ProgramEquals {
                    left: x1,
                    right: param
                }
            );
            for fact in &actual_equations {
                assert!(vc.local_context.entries().iter().any(|entry| {
                    let Some(VcFormulaRef::Generated(id)) = entry.formula else {
                        return false;
                    };
                    vcs.generated_formula(id).is_some_and(|f| {
                        f.kind == K::AlgorithmStateFact
                            && f.shape
                                == F::ProgramEquals {
                                    left: fact.0,
                                    right: fact.1,
                                }
                    })
                }));
            }
        }
        assert!(
            assertion_vc
                .local_context
                .entries()
                .iter()
                .all(|entry| entry.formula != Some(assertion_vc.goal)
                    && !matches!(entry.kind, C::PendingAlgorithmAssertion { .. }))
        );
        let pending = return_vc
            .local_context
            .entries()
            .iter()
            .filter_map(|entry| {
                if let C::PendingAlgorithmAssertion { handoff } = entry.kind {
                    Some((handoff, entry.formula))
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        let [(assertion_handoff, assertion_formula)] = pending.as_slice() else {
            panic!("one pending assertion dependency")
        };
        assert_eq!(*assertion_formula, Some(assertion_vc.goal));
        assert!(
            matches!(vcs.seed_accounting_for(*assertion_handoff).unwrap().mapping, SeedVcMapping::One { vc } if vc == assertion_vc.id)
        );
        assert_eq!(
            handoff.entries.get(*assertion_handoff).unwrap().seed.goal,
            Some(asserted)
        );
        assert_eq!(vcs.seed_accounting().len(), 3);
        assert_eq!(
            vcs.seed_accounting()
                .iter()
                .filter(|r| matches!(r.mapping, SeedVcMapping::One { .. }))
                .count(),
            2
        );
        assert_eq!(
            vcs.seed_accounting()
                .iter()
                .filter(|r| matches!(r.mapping, SeedVcMapping::NoConcreteVc { .. }))
                .count(),
            1
        );
        assert!(
            vcs.seed_accounting()
                .iter()
                .all(|r| r.seed_status == ObligationSeedStatus::Deferred)
        );
        if source == &text {
            assert_eq!(
                vcs.debug_text(),
                std::fs::read_to_string(
                    step5c11_config()
                        .workspace_root
                        .join("tests")
                        .join(case.expectation.snapshots.as_ref().unwrap())
                )
                .unwrap()
            );
        }
    }
}

#[test]
fn step5c14_state_near_misses_keep_real_goals_and_zero_vc_accounting() {
    use mizar_vc::vc_ir::{SeedVcMapping, VcKind};
    let case = step5c14_state_case();
    let text = std::fs::read_to_string(&case.source_path).unwrap();
    for (source, expected) in [
        (
            text.replace("    assert x = a;\n", ""),
            vec![VcKind::AlgorithmPostcondition],
        ),
        (
            text.replace("    ensures result = a\n", ""),
            vec![VcKind::AlgorithmAssertion],
        ),
        (
            text.replace("    assert x = a;\n", "")
                .replace("    ensures result = a\n", ""),
            vec![],
        ),
    ] {
        let core = step5c14_static_core(&case, &source).unwrap();
        let vcs = step5c14_state_vcs(&core).unwrap();
        assert_eq!(
            vcs,
            step5c14_state_vcs(&step5c14_static_core(&case, &source).unwrap()).unwrap()
        );
        assert_eq!(
            vcs.vcs()
                .iter()
                .map(|vc| vc.kind.clone())
                .collect::<Vec<_>>(),
            expected
        );
        assert_eq!(vcs.seed_accounting().len(), expected.len() + 1);
        assert_eq!(
            vcs.seed_accounting()
                .iter()
                .filter(|r| matches!(r.mapping, SeedVcMapping::NoConcreteVc { .. }))
                .count(),
            1
        );
        if expected.is_empty() {
            assert!(vcs.generated_formulas().is_empty());
            assert_eq!(
                vcs.debug_text(),
                std::fs::read_to_string(
                    step5c11_config()
                        .workspace_root
                        .join("tests/snapshots/vc/step5c14_algorithm_state_no_contract.vc_ir.snap")
                )
                .unwrap()
            );
        }
    }
    for (from, to) in [
        ("x := c;", "missing := c;"),
        ("x := c;", "x := missing;"),
        ("x := c;", "x.field := c;"),
        ("var x := a;", "var x := x;"),
        ("var x := a;", "var x := c;"),
        ("const c := a;", "const x := a;"),
        ("assert x = a;", "assert x in a;"),
        ("assert x = a;", "assert result = a;"),
        ("assert x = a;", "assert x = a by Missing;"),
        ("return x;", "return result;"),
        ("return x;", "return x; x := a;"),
    ] {
        let changed = text.replace(from, to);
        assert_ne!(changed, text);
        assert!(
            step5c14_static_core(&case, &changed).is_err(),
            "{from} -> {to}"
        );
    }
}

#[test]
fn step5c14_state_rejects_actual_immutable_and_ghost_writes() {
    use mizar_core::{
        control_flow::{ControlFlowDiagnosticKind as D, build_control_flow_ir},
        core_ir::CoreAlgorithmStmtKind as S,
    };
    let case = step5c14_state_case();
    let text = std::fs::read_to_string(&case.source_path).unwrap();
    for target in ["c", "a"] {
        let source = text.replace("x := c;", &format!("{target} := x;"));
        let core = step5c14_static_core(&case, &source).unwrap();
        let (_, algorithm) = core.algorithms().iter().next().unwrap();
        let S::AssignLocal { target: actual, .. } = core
            .algorithm_statements()
            .get(algorithm.statements[2])
            .unwrap()
            .kind
        else {
            panic!("typed destination")
        };
        let flows = build_control_flow_ir(&core);
        assert!(flows.flows.iter().any(|(_, flow)| {
            flow.diagnostics
                .iter()
                .any(|(_, d)| matches!(d.kind, D::ImmutableAssignment { var, .. } if var == actual))
        }));
        assert!(step5c14_state_vcs(&core).is_err());
    }
    let diagnostic_only = text
        .replace("    ensures result = a\n", "")
        .replace("    assert x = a;\n", "");
    for unreachable in [false, true] {
        let source = diagnostic_only
            .replace("const c := a;", "ghost var c := a;")
            .replace(
                "x := c;",
                if unreachable {
                    "return a; x := c;"
                } else {
                    "x := c;"
                },
            );
        let core = step5c14_static_core(&case, &source).unwrap();
        assert!(build_control_flow_ir(&core).flows.iter().any(|(_, f)| {
            f.diagnostics
                .iter()
                .any(|(_, d)| matches!(d.kind, D::GhostIsolationViolation { .. }))
        }));
        assert!(step5c14_state_vcs(&core).is_err());
    }
    let shadowed = text
        .replace("var x := a;", "var a := a;")
        .replace("x := c;", "a := c;")
        .replace("assert x = a;", "assert a = c;")
        .replace("return x;", "return a;");
    let core = step5c14_static_core(&case, &shadowed).unwrap();
    let (_, algorithm) = core.algorithms().iter().next().unwrap();
    let S::Let {
        binder,
        value: Some(value),
        ..
    } = &core
        .algorithm_statements()
        .get(algorithm.statements[0])
        .unwrap()
        .kind
    else {
        panic!("shadow declaration")
    };
    assert_ne!(binder.var, algorithm.params[0].var);
    assert_eq!(
        core.terms().get(*value).unwrap().kind,
        mizar_core::core_ir::CoreTermKind::Var(algorithm.params[0].var)
    );
    assert_eq!(step5c14_state_vcs(&core).unwrap().vcs().len(), 2);
}

#[test]
fn step5c14_state_changed_source_references_reach_each_generated_formula() {
    use mizar_core::core_ir::CoreAlgorithmStmtKind as S;
    use mizar_vc::vc_ir::{
        VcFormulaRef, VcGeneratedFormulaKind as K, VcGeneratedFormulaShape as F, VcKind,
        VcProgramValue as V,
    };
    let case = step5c14_state_case();
    let text = std::fs::read_to_string(&case.source_path).unwrap();
    for (index, source) in [
        text.replace("x := c;", "x := a;"),
        text.replace("assert x = a;", "assert c = x;"),
        text.replace("return x;", "return c;"),
        text.replace("x := c;", "x := x;"),
    ]
    .into_iter()
    .enumerate()
    {
        assert_ne!(source, text);
        let core = step5c14_static_core(&case, &source).unwrap();
        let (_, algorithm) = core.algorithms().iter().next().unwrap();
        let S::Let { binder: x, .. } = &core
            .algorithm_statements()
            .get(algorithm.statements[0])
            .unwrap()
            .kind
        else {
            panic!("x")
        };
        let S::Let { binder: c, .. } = &core
            .algorithm_statements()
            .get(algorithm.statements[1])
            .unwrap()
            .kind
        else {
            panic!("c")
        };
        let param = V {
            var: algorithm.params[0].var,
            definition: None,
        };
        let x0 = V {
            var: x.var,
            definition: Some(algorithm.statements[0]),
        };
        let c0 = V {
            var: c.var,
            definition: Some(algorithm.statements[1]),
        };
        let x1 = V {
            var: x.var,
            definition: Some(algorithm.statements[2]),
        };
        let rhs = match index {
            0 => param,
            3 => x0,
            _ => c0,
        };
        let vcs = step5c14_state_vcs(&core).unwrap();
        assert!(vcs.generated_formulas().iter().any(|formula| formula.kind
            == K::AlgorithmStateFact
            && formula.shape
                == F::ProgramEquals {
                    left: x1,
                    right: rhs
                }));
        for (kind, operands) in [
            (
                VcKind::AlgorithmAssertion,
                if index == 1 { (c0, x1) } else { (x1, param) },
            ),
            (
                VcKind::AlgorithmPostcondition,
                (if index == 2 { c0 } else { x1 }, param),
            ),
        ] {
            let vc = vcs.vcs().iter().find(|vc| vc.kind == kind).unwrap();
            let VcFormulaRef::Generated(id) = vc.goal else {
                panic!("state goal")
            };
            assert_eq!(
                vcs.generated_formula(id).unwrap().shape,
                F::ProgramEquals {
                    left: operands.0,
                    right: operands.1
                },
                "mutation {index}"
            );
        }
    }
}

#[test]
fn step5c14_state_earlier_assertion_excludes_later_writes_and_keeps_its_goal() {
    use mizar_core::core_ir::CoreAlgorithmStmtKind as S;
    use mizar_vc::vc_ir::{
        ContextEntryKind as C, VcFormulaRef, VcGeneratedFormulaKind as K,
        VcGeneratedFormulaShape as F, VcKind, VcProgramValue as V,
    };
    let case = step5c14_state_case();
    let text = std::fs::read_to_string(&case.source_path).unwrap();
    let source = text
        .replace("const c := a;", "const c := x;\n    assert x = a;")
        .replace(
            "    assert x = a;\n    return x;",
            "    assert x = c;\n    return x;",
        );
    let core = step5c14_static_core(&case, &source).unwrap();
    let (_, algorithm) = core.algorithms().iter().next().unwrap();
    let [xd, cd, first_assert, assignment, second_assert, _] = algorithm.statements.as_slice()
    else {
        panic!("two assertions")
    };
    let S::Let { binder: x, .. } = &core.algorithm_statements().get(*xd).unwrap().kind else {
        panic!("x")
    };
    let S::Let { binder: c, .. } = &core.algorithm_statements().get(*cd).unwrap().kind else {
        panic!("c")
    };
    let x0 = V {
        var: x.var,
        definition: Some(*xd),
    };
    let c0 = V {
        var: c.var,
        definition: Some(*cd),
    };
    let x1 = V {
        var: x.var,
        definition: Some(*assignment),
    };
    let param = V {
        var: algorithm.params[0].var,
        definition: None,
    };
    let vcs = step5c14_state_vcs(&core).unwrap();
    assert_eq!(
        vcs,
        step5c14_state_vcs(&step5c14_static_core(&case, &source).unwrap()).unwrap()
    );
    assert_eq!(vcs.vcs().len(), 3);
    let shape = |reference| {
        let VcFormulaRef::Generated(id) = reference else {
            panic!("generated formula")
        };
        &vcs.generated_formula(id).unwrap().shape
    };
    let first = vcs
        .vcs()
        .iter()
        .find(|vc| {
            vc.kind == VcKind::AlgorithmAssertion
                && shape(vc.goal)
                    == &F::ProgramEquals {
                        left: x0,
                        right: param,
                    }
        })
        .unwrap();
    let second = vcs
        .vcs()
        .iter()
        .find(|vc| {
            vc.kind == VcKind::AlgorithmAssertion
                && shape(vc.goal)
                    == &F::ProgramEquals {
                        left: x1,
                        right: c0,
                    }
        })
        .unwrap();
    let returned = vcs
        .vcs()
        .iter()
        .find(|vc| vc.kind == VcKind::AlgorithmPostcondition)
        .unwrap();
    assert_eq!(
        shape(returned.goal),
        &F::ProgramEquals {
            left: x1,
            right: param
        }
    );
    assert!(
        first
            .local_context
            .entries()
            .iter()
            .all(|entry| !matches!(entry.kind, C::PendingAlgorithmAssertion { .. }))
    );
    for entry in first.local_context.entries() {
        if let Some(VcFormulaRef::Generated(id)) = entry.formula {
            match &vcs.generated_formula(id).unwrap().shape {
                F::ProgramEquals { left, right } => {
                    assert_ne!(left.definition, Some(*assignment));
                    assert_ne!(right.definition, Some(*assignment));
                }
                F::ProgramTypePredicate { subject, .. } => {
                    assert_ne!(subject.definition, Some(*assignment))
                }
                other => panic!("unexpected state premise {other:?}"),
            }
        }
        assert_ne!(entry.formula, Some(first.goal));
    }
    for vc in [first, second, returned] {
        assert!(vc.local_context.entries().iter().any(|entry| {
            let Some(VcFormulaRef::Generated(id)) = entry.formula else {
                return false;
            };
            let formula = vcs.generated_formula(id).unwrap();
            formula.kind == K::AlgorithmStateFact
                && formula.shape
                    == F::ProgramEquals {
                        left: c0,
                        right: x0,
                    }
        }));
    }
    let handoff = mizar_core::control_flow::build_obligation_seed_handoff(
        &core,
        &mizar_core::control_flow::build_control_flow_ir(&core),
    );
    for (vc, count) in [(second, 1), (returned, 2)] {
        let pending = vc
            .local_context
            .entries()
            .iter()
            .filter(|entry| matches!(entry.kind, C::PendingAlgorithmAssertion { .. }))
            .collect::<Vec<_>>();
        assert_eq!(pending.len(), count);
        let entry = pending
            .iter()
            .find(|entry| entry.formula == Some(first.goal))
            .unwrap();
        let C::PendingAlgorithmAssertion { handoff: id } = entry.kind else {
            unreachable!()
        };
        let S::Assert { formula } = core.algorithm_statements().get(*first_assert).unwrap().kind
        else {
            unreachable!()
        };
        assert_eq!(handoff.entries.get(id).unwrap().seed.goal, Some(formula));
        assert_eq!(
            shape(entry.formula.unwrap()),
            &F::ProgramEquals {
                left: x0,
                right: param
            }
        );
        if count == 2 {
            let entry = pending
                .iter()
                .find(|entry| entry.formula == Some(second.goal))
                .unwrap();
            let C::PendingAlgorithmAssertion { handoff: id } = entry.kind else {
                unreachable!()
            };
            let S::Assert { formula } = core
                .algorithm_statements()
                .get(*second_assert)
                .unwrap()
                .kind
            else {
                unreachable!()
            };
            assert_eq!(handoff.entries.get(id).unwrap().seed.goal, Some(formula));
        }
    }
}

#[test]
fn step5c14_state_rejects_corrupt_core_write_formula_and_source_ownership() {
    use mizar_core::core_ir::*;
    let case = step5c14_state_case();
    let core =
        step5c14_static_core(&case, &std::fs::read_to_string(&case.source_path).unwrap()).unwrap();
    let (algorithm_id, algorithm) = core.algorithms().iter().next().unwrap();
    let [xd, cd, assignment, assertion, returned] = algorithm.statements.as_slice() else {
        panic!("state statements")
    };
    let CoreAlgorithmStmtKind::Let {
        value: Some(initial),
        ..
    } = core.algorithm_statements().get(*xd).unwrap().kind
    else {
        panic!("initial")
    };
    let CoreAlgorithmStmtKind::Let { binder: c, .. } =
        &core.algorithm_statements().get(*cd).unwrap().kind
    else {
        panic!("const")
    };
    let CoreAlgorithmStmtKind::Assert { formula: asserted } =
        core.algorithm_statements().get(*assertion).unwrap().kind
    else {
        panic!("assert")
    };
    let CoreAlgorithmStmtKind::AssignLocal { target, value } =
        core.algorithm_statements().get(*assignment).unwrap().kind
    else {
        panic!("write")
    };
    let ensure = algorithm.contracts.ensures[0];
    for mutation in 0..12 {
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
            0 => {
                parts
                    .algorithm_statements
                    .get_mut(*assignment)
                    .unwrap()
                    .kind = CoreAlgorithmStmtKind::AssignLocal {
                    target,
                    value: initial,
                }
            }
            1 => {
                parts
                    .algorithm_statements
                    .get_mut(*assignment)
                    .unwrap()
                    .kind = CoreAlgorithmStmtKind::AssignLocal {
                    target: c.var,
                    value,
                }
            }
            2 => {
                parts.algorithm_statements.get_mut(*assertion).unwrap().kind =
                    CoreAlgorithmStmtKind::Assert { formula: ensure }
            }
            3 => {
                parts
                    .algorithms
                    .get_mut(algorithm_id)
                    .unwrap()
                    .contracts
                    .ensures[0] = asserted
            }
            4 => {
                let CoreAlgorithmStmtKind::Let { binder, .. } =
                    &mut parts.algorithm_statements.get_mut(*cd).unwrap().kind
                else {
                    unreachable!()
                };
                binder.role = "unknown-local".into();
            }
            5 => {
                let CoreAlgorithmStmtKind::Let { binder, .. } =
                    &mut parts.algorithm_statements.get_mut(*xd).unwrap().kind
                else {
                    unreachable!()
                };
                binder.ty_guard = algorithm.params[0].ty_guard;
            }
            6 => parts
                .algorithm_statements
                .get_mut(*assignment)
                .unwrap()
                .source
                .provenance
                .pop()
                .map(|_| ())
                .unwrap(),
            7 => {
                parts
                    .algorithm_statements
                    .get_mut(*assignment)
                    .unwrap()
                    .source
                    .anchor = core
                    .algorithm_statements()
                    .get(*assertion)
                    .unwrap()
                    .source
                    .anchor
                    .clone()
            }
            8 => parts
                .algorithms
                .get_mut(algorithm_id)
                .unwrap()
                .statements
                .swap(2, 3),
            9 => {
                parts.algorithm_statements.get_mut(*returned).unwrap().kind =
                    CoreAlgorithmStmtKind::Return(Some(initial))
            }
            10 => {
                let id = parts
                    .formulas
                    .insert(core.formulas().get(asserted).unwrap().clone());
                parts
                    .source_map
                    .formula_sources
                    .insert(id, parts.formulas.get(id).unwrap().source.clone());
            }
            11 => {
                let id = parts.terms.insert(core.terms().get(value).unwrap().clone());
                parts
                    .source_map
                    .term_sources
                    .insert(id, parts.terms.get(id).unwrap().source.clone());
            }
            _ => unreachable!(),
        }
        if matches!(mutation, 6 | 7) {
            parts.source_map.algorithm_sources.insert(
                *assignment,
                parts
                    .algorithm_statements
                    .get(*assignment)
                    .unwrap()
                    .source
                    .clone(),
            );
        }
        let corrupt = CoreIr::try_new(parts)
            .unwrap_or_else(|error| panic!("structural state probe {mutation}: {error}"));
        assert!(
            step5c14_state_vcs(&corrupt).is_err(),
            "state corruption {mutation}"
        );
    }
}

#[test]
fn step5c14_claim_retains_target_quantification_local_goal_and_handoff() {
    use mizar_core::control_flow::{
        ControlFlowObligationSiteKind, ObligationHandoffOrigin, build_control_flow_ir,
        build_obligation_seed_handoff,
    };
    use mizar_core::core_ir::{
        CoreFormulaKind as F, CoreNodeRef as R, CoreProofNodeKind as P, CoreTermKind as T,
    };
    use mizar_vc::vc_ir::{ContextEntryKind, SeedVcMapping, VcFormulaRef, VcStatus};
    let case = step5c14_claim_case();
    let text = std::fs::read_to_string(&case.source_path).unwrap();
    for (source, expected_algorithm) in [
        (text.clone(), "Cl1"),
        (
            text.replace("Cl1", "EmptyRun")
                .replace("CT1", "ReflexiveClaim")
                .replace("X", "Y"),
            "EmptyRun",
        ),
        (
            text.replace(
                "let X be set;\n    thus X = X",
                "let Z be set;\n    thus Z = Z",
            ),
            "Cl1",
        ),
    ] {
        let core = step5c14_claim_core(&case, &source).unwrap();
        let replay = step5c14_claim_core(&case, &source).unwrap();
        assert_eq!(core, replay);
        let (algorithm_id, algorithm) = core.algorithms().iter().next().unwrap();
        let (_, proof) = core.proofs().iter().next().unwrap();
        let item = core.items().get(proof.item).unwrap();
        let ast = super::super::formula_statement::step5c8_test_frontend(&source)
            .ast
            .unwrap();
        let resolver =
            super::super::resolver_symbol_collection(&step5c11_config().workspace_root, &case, &ast);
        let expected_symbol = resolver
            .env
            .symbols()
            .iter()
            .find(|entry| {
                entry.kind() == mizar_resolve::env::SymbolKind::Algorithm
                    && entry.primary_spelling() == expected_algorithm
            })
            .unwrap();
        assert_eq!(&algorithm.symbol, expected_symbol.symbol());
        assert_eq!(item.dependencies, [algorithm.item]);
        let F::Forall { binders, body } = &core.formulas().get(proof.proposition).unwrap().kind
        else {
            panic!("actual universal theorem")
        };
        let P::IntroduceBinder { binder, child } =
            &core.proof_nodes().get(proof.root).unwrap().kind
        else {
            panic!("actual proof let")
        };
        assert_eq!(binders.len(), 1);
        assert_ne!(binders[0].var, binder.var);
        let P::TerminalGoal {
            obligation,
            citations,
        } = &core.proof_nodes().get(*child).unwrap().kind
        else {
            panic!("actual terminal")
        };
        assert!(citations.is_empty());
        let seed = core.obligation_seeds().get(*obligation).unwrap();
        assert!(seed.core_refs.contains(&R::Item(algorithm.item)));
        assert!(seed.core_refs.contains(&R::Algorithm(algorithm_id)));
        assert_eq!(seed.context, [binder.ty_guard.unwrap()]);
        assert!(seed.label.is_none());
        for (formula, variable, guard) in [
            (*body, binders[0].var, binders[0].ty_guard.unwrap()),
            (seed.goal.unwrap(), binder.var, binder.ty_guard.unwrap()),
        ] {
            let F::Equals { left, right } = core.formulas().get(formula).unwrap().kind else {
                panic!("written equality")
            };
            assert_ne!(left, right);
            for term in [left, right] {
                assert_eq!(core.terms().get(term).unwrap().kind, T::Var(variable));
            }
            let F::TypePred { subject, ref ty } = core.formulas().get(guard).unwrap().kind else {
                panic!("set guard")
            };
            assert_eq!(ty.as_str(), "set");
            assert_eq!(core.terms().get(subject).unwrap().kind, T::Var(variable));
        }
        let mizar_core::core_ir::CoreSourceAnchor::SourceRange(claim_range) = seed.source.anchor
        else {
            panic!("source range")
        };
        assert!(source[claim_range.start..claim_range.end].starts_with("claim "));
        let flow = build_control_flow_ir(&core);
        assert_eq!(flow, build_control_flow_ir(&replay));
        let handoff = build_obligation_seed_handoff(&core, &flow);
        let vcs = step5c14_claim_vcs(&core).unwrap();
        assert_eq!(vcs, step5c14_claim_vcs(&replay).unwrap());
        assert_eq!(
            vcs.debug_text(),
            step5c14_claim_vcs(&replay).unwrap().debug_text()
        );
        assert_eq!(vcs.vcs().len(), 1);
        assert_eq!(vcs.seed_accounting().len(), 2);
        assert!(vcs.generated_formulas().is_empty());
        let vc = &vcs.vcs()[0];
        assert_eq!(vc.status, VcStatus::Open);
        assert_eq!(vc.goal, VcFormulaRef::Core(seed.goal.unwrap()));
        assert_eq!(vc.source.primary, seed.source);
        assert_eq!(
            vc.source.related,
            [
                algorithm.source.clone(),
                item.source.clone(),
                core.proof_nodes().get(*child).unwrap().source.clone()
            ]
        );
        assert_eq!(vc.local_context.entries().len(), 1);
        assert_eq!(
            vc.local_context.entries()[0].kind,
            ContextEntryKind::ProofAssumption
        );
        assert_eq!(
            vc.local_context.entries()[0].formula,
            Some(VcFormulaRef::Core(binder.ty_guard.unwrap()))
        );
        assert_eq!(
            vc.premises,
            [mizar_vc::vc_ir::PremiseRef::LocalContext(
                vc.local_context.entries()[0].id
            )]
        );
        assert!(vc.proof_hint.is_none());
        for row in vcs.seed_accounting() {
            let entry = handoff.entries.get(row.handoff).unwrap();
            match entry.origin {
                ObligationHandoffOrigin::ExistingCore { seed: id } => {
                    assert_eq!(id, *obligation);
                    assert_eq!(row.mapping, SeedVcMapping::One { vc: vc.id });
                    assert_eq!(
                        row.seed_status,
                        mizar_core::core_ir::ObligationSeedStatus::Active
                    );
                }
                ObligationHandoffOrigin::FlowDerived { algorithm, .. } => {
                    assert_eq!(algorithm, algorithm_id);
                    assert_eq!(
                        entry.flow_site.as_ref().unwrap().kind,
                        ControlFlowObligationSiteKind::PartialTermination
                    );
                    assert_eq!(
                        row.seed_status,
                        mizar_core::core_ir::ObligationSeedStatus::Deferred
                    );
                    assert!(matches!(row.mapping, SeedVcMapping::NoConcreteVc { .. }));
                }
                _ => panic!("unexpected origin"),
            }
        }
    }
}

#[test]
fn step5c14_claim_rejects_unsupported_source_without_partial_output() {
    let case = step5c14_claim_case();
    let source = std::fs::read_to_string(&case.source_path).unwrap();
    for (from, to) in [
        ("claim Cl1", "claim Missing"),
        ("claim Cl1", "claim CT1"),
        ("algorithm Cl1()", "algorithm Cl1(X)"),
        ("algorithm Cl1()", "algorithm Cl1() -> set"),
        ("  do\n", "  requires contradiction\n  do\n"),
        ("return;", "var X as object; return;"),
        ("return;", "snapshot S; return;"),
        ("return;", "return Cl1();"),
        ("return;", "return 0;"),
        ("theorem CT1", "open theorem CT1"),
        ("theorem CT1", "assumed theorem CT1"),
        ("thus X = X;", "thus X = X by CT1;"),
        (
            "  theorem CT1",
            "  theorem Extra: for Z being set holds Z = Z proof let Z be set; thus Z = Z; end;\n  theorem CT1",
        ),
        ("    thus X = X;", ""),
        ("for X being set", "for X being object"),
        ("    let X be set;", "    let X be object;"),
    ] {
        let changed = source.replacen(from, to, 1);
        assert_ne!(changed, source);
        let frontend = super::super::formula_statement::step5c8_test_frontend(&changed);
        assert!(
            frontend.diagnostics.is_empty(),
            "semantic control {from} -> {to}: {:?}",
            frontend.diagnostics
        );
        assert!(
            step5c14_claim_core(&case, &changed).is_err(),
            "accepted {from} -> {to}"
        );
    }
    let (algorithm, claim) = source.split_once("claim Cl1").unwrap();
    for changed in [
        format!("claim Cl1{claim}\n{algorithm}"),
        algorithm.to_owned(),
        format!("{algorithm}\nclaim Cl1 do end;"),
        format!("{source}\n theorem Extra: contradiction;"),
        source.replace("  proof\n    let X be set;\n    thus X = X;\n  end;", ";"),
    ] {
        let frontend = super::super::formula_statement::step5c8_test_frontend(&changed);
        assert!(
            frontend.diagnostics.is_empty(),
            "semantic control {changed}: {:?}",
            frontend.diagnostics
        );
        assert!(
            step5c14_claim_core(&case, &changed).is_err(),
            "accepted {changed}"
        );
    }
    let malformed = source.replace("return;", "return @;");
    let frontend = super::super::formula_statement::step5c8_test_frontend(&malformed);
    assert!(!frontend.diagnostics.is_empty());
    let error = step5c14_claim_core(&case, &malformed).unwrap_err();
    assert!(
        error == "registration.frontend_diagnostics" || error == "missing AST",
        "{error}"
    );
    let duplicate = format!("{algorithm}\n{source}");
    assert!(
        super::super::formula_statement::step5c8_test_frontend(&duplicate)
            .diagnostics
            .is_empty()
    );
    assert_eq!(
        step5c14_claim_core(&case, &duplicate).unwrap_err(),
        "registration.resolver_diagnostics"
    );
}

#[test]
fn step5c14_claim_rejects_coherent_core_corruption_and_unused_rows() {
    use mizar_core::core_ir::{
        CoreAlgorithmStmtKind as S, CoreFormulaKind as F, CoreIr, CoreIrParts, CoreNodeRef as R,
        CoreProofNodeKind as P, CoreSourceAnchor, CoreTermKind as T,
    };
    let case = step5c14_claim_case();
    let text = std::fs::read_to_string(&case.source_path).unwrap();
    let core = step5c14_claim_core(&case, &text).unwrap();
    let (algorithm_id, algorithm) = core.algorithms().iter().next().unwrap();
    let (proof_id, proof) = core.proofs().iter().next().unwrap();
    let P::IntroduceBinder { binder, child } = &core.proof_nodes().get(proof.root).unwrap().kind
    else {
        panic!("proof let")
    };
    let P::TerminalGoal { obligation, .. } = &core.proof_nodes().get(*child).unwrap().kind else {
        panic!("terminal")
    };
    let seed = core.obligation_seeds().get(*obligation).unwrap();
    let F::Forall { binders, body } = &core.formulas().get(proof.proposition).unwrap().kind else {
        panic!("forall")
    };
    let F::Equals { left, .. } = core.formulas().get(*body).unwrap().kind else {
        panic!("equality")
    };
    for mutation in 0..33 {
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
            0 => parts
                .items
                .get_mut(proof.item)
                .unwrap()
                .dependencies
                .clear(),
            1 => {
                parts.algorithms.get_mut(algorithm_id).unwrap().symbol =
                    core.items().get(proof.item).unwrap().symbol.clone()
            }
            2 => parts.algorithms.get_mut(algorithm_id).unwrap().item = proof.item,
            3 => {
                parts
                    .obligation_seeds
                    .get_mut(*obligation)
                    .unwrap()
                    .source
                    .anchor = proof.source.anchor.clone()
            }
            4 => {
                parts
                    .obligation_seeds
                    .get_mut(*obligation)
                    .unwrap()
                    .source
                    .anchor = algorithm.source.anchor.clone()
            }
            5 => parts.obligation_seeds.get_mut(*obligation).unwrap().owner = algorithm.item,
            6 => parts
                .obligation_seeds
                .get_mut(*obligation)
                .unwrap()
                .core_refs
                .retain(|reference| reference != &R::Algorithm(algorithm_id)),
            7 => {
                parts.obligation_seeds.get_mut(*obligation).unwrap().context =
                    vec![binders[0].ty_guard.unwrap()]
            }
            8 => {
                parts.obligation_seeds.get_mut(*obligation).unwrap().goal = Some(proof.proposition)
            }
            9 => {
                parts.proofs.get_mut(proof_id).unwrap().status =
                    mizar_core::core_ir::CoreProofStatus::Open
            }
            10 => {
                parts.obligation_seeds.get_mut(*obligation).unwrap().status =
                    mizar_core::core_ir::ObligationSeedStatus::Skipped
            }
            11 => {
                let F::Forall { binders, .. } =
                    &mut parts.formulas.get_mut(proof.proposition).unwrap().kind
                else {
                    unreachable!()
                };
                binders[0].var = binder.var;
            }
            12 => {
                let P::IntroduceBinder { binder: local, .. } =
                    &mut parts.proof_nodes.get_mut(proof.root).unwrap().kind
                else {
                    unreachable!()
                };
                local.var = binders[0].var;
            }
            13 => {
                let F::TypePred { ty, .. } = &mut parts
                    .formulas
                    .get_mut(binder.ty_guard.unwrap())
                    .unwrap()
                    .kind
                else {
                    unreachable!()
                };
                *ty = mizar_core::core_ir::CoreTypePredicate::new("object");
            }
            14 => {
                let P::TerminalGoal { citations, .. } =
                    &mut parts.proof_nodes.get_mut(*child).unwrap().kind
                else {
                    unreachable!()
                };
                citations.push(mizar_core::core_ir::CoreCitation::Symbol(
                    core.items().get(proof.item).unwrap().symbol.clone(),
                ));
            }
            15 => {
                parts.terms.insert(core.terms().get(left).unwrap().clone());
            }
            16 => {
                parts
                    .formulas
                    .insert(core.formulas().get(*body).unwrap().clone());
            }
            17 => {
                parts
                    .algorithm_statements
                    .get_mut(algorithm.statements[0])
                    .unwrap()
                    .kind = S::Return(Some(left))
            }
            18 => {
                let other = parts.algorithms.insert(algorithm.clone());
                parts
                    .algorithm_statements
                    .get_mut(algorithm.statements[0])
                    .unwrap()
                    .owner = other;
            }
            19 => {
                parts.proofs.get_mut(proof_id).unwrap().source.anchor =
                    algorithm.source.anchor.clone()
            }
            20 => {
                parts.proof_nodes.get_mut(*child).unwrap().source.anchor =
                    proof.source.anchor.clone()
            }
            21 => {
                parts.proof_nodes.get_mut(proof.root).unwrap().source.anchor =
                    proof.source.anchor.clone()
            }
            22 => parts.terms.get_mut(left).unwrap().kind = T::Var(binder.var),
            23 => {
                parts.obligation_seeds.get_mut(*obligation).unwrap().label =
                    Some(mizar_core::core_ir::CoreLabelRef::new("CT1"))
            }
            24 => parts
                .obligation_seeds
                .get_mut(*obligation)
                .unwrap()
                .core_refs
                .push(R::Formula(proof.proposition)),
            25 => parts
                .obligation_seeds
                .get_mut(*obligation)
                .unwrap()
                .source
                .provenance
                .clear(),
            26 => parts
                .algorithms
                .get_mut(algorithm_id)
                .unwrap()
                .contracts
                .requires
                .push(*body),
            27 => {
                parts
                    .algorithm_statements
                    .get_mut(algorithm.statements[0])
                    .unwrap()
                    .source
                    .anchor = proof.source.anchor.clone()
            }
            28 => {
                let CoreSourceAnchor::SourceRange(mut source) =
                    parts.terms.get(left).unwrap().source.anchor
                else {
                    unreachable!()
                };
                source.start = 0;
                parts.terms.get_mut(left).unwrap().source.anchor =
                    CoreSourceAnchor::SourceRange(source);
            }
            29 => parts
                .items
                .get_mut(proof.item)
                .unwrap()
                .source
                .provenance
                .clear(),
            30 => {
                parts
                    .algorithm_statements
                    .get_mut(algorithm.statements[0])
                    .unwrap()
                    .source
                    .anchor = algorithm.source.anchor.clone()
            }
            31 => {
                parts
                    .formulas
                    .get_mut(seed.goal.unwrap())
                    .unwrap()
                    .source
                    .anchor = core
                    .proof_nodes()
                    .get(*child)
                    .unwrap()
                    .source
                    .anchor
                    .clone()
            }
            32 => {
                let guard = binders[0].ty_guard.unwrap();
                let F::TypePred { subject, .. } = parts.formulas.get(guard).unwrap().kind else {
                    unreachable!()
                };
                parts.formulas.get_mut(guard).unwrap().source.anchor =
                    binders[0].source.anchor.clone();
                parts.terms.get_mut(subject).unwrap().source.anchor =
                    binders[0].source.anchor.clone();
            }
            _ => unreachable!(),
        }
        // Keep source maps coherent so these controls reach the semantic owner.
        parts.source_map.item_sources = parts
            .items
            .iter()
            .map(|(id, row)| (id, row.source.clone()))
            .collect();
        parts.source_map.term_sources = parts
            .terms
            .iter()
            .map(|(id, row)| (id, row.source.clone()))
            .collect();
        parts.source_map.formula_sources = parts
            .formulas
            .iter()
            .map(|(id, row)| (id, row.source.clone()))
            .collect();
        parts.source_map.proof_sources = parts
            .proof_nodes
            .iter()
            .map(|(id, row)| (id, row.source.clone()))
            .collect();
        parts.source_map.algorithm_sources = parts
            .algorithm_statements
            .iter()
            .map(|(id, row)| (id, row.source.clone()))
            .collect();
        parts.source_map.obligation_sources = parts
            .obligation_seeds
            .iter()
            .map(|(id, row)| (id, row.source.clone()))
            .collect();
        if mutation == 18 {
            assert!(matches!(
                CoreIr::try_new(parts),
                Err(mizar_core::core_ir::CoreIrError::StatementOwnerMismatch { .. })
            ));
            continue;
        }
        let changed =
            CoreIr::try_new(parts).unwrap_or_else(|error| panic!("mutation {mutation}: {error}"));
        assert!(
            step5c14_claim_vcs(&changed).is_err(),
            "accepted mutation {mutation}; original goal {:?}",
            seed.goal
        );
    }
}

#[test]
fn step5c14_claim_admission_rejects_auxiliary_payload_and_identity_hijacking() {
    let config = step5c11_config();
    let plan = build_test_plan(&config).unwrap();
    for original in [
        step5c14_claim_case(),
        step5c14_assert_failure_case(),
        plan.cases
            .iter()
            .find(|case| case.id.0 == "pass_proof_verification_computation_justification_001")
            .unwrap()
            .clone(),
    ] {
        for mutation in 0..11 {
            let mut case = original.clone();
            let expected = &mut case.expectation;
            match mutation {
                0 => expected.schema_version = 2,
                1 => expected.profiles.push("stress".into()),
                2 => expected.ast_profile = Some("unrelated".into()),
                3 => expected.snapshot_profiles.push("unrelated".into()),
                4 => {
                    expected.tokens = plan
                        .cases
                        .iter()
                        .find(|case| !case.expectation.tokens.is_empty())
                        .unwrap()
                        .expectation
                        .tokens
                        .clone()
                }
                5 => {
                    expected.origin = Some(crate::expectation::OriginMetadata {
                        schema_version: 1,
                        kind: expected.kind,
                        generator: "test".into(),
                        generator_version: "1".into(),
                        seed: "0".into(),
                        profile: "fast".into(),
                        expected_outcome: expected.expected_outcome,
                        minimized: false,
                        original_failure_category: None,
                    })
                }
                6 => {
                    expected.architecture22 = Some(crate::expectation::Architecture22Metadata {
                        scenarios: vec!["cache_hit_miss_timing".into()],
                        equivalence_class: Some("observable_outputs_equal".into()),
                        gate: crate::expectation::Architecture22Gate::Planned,
                    })
                }
                7 => expected.diagnostic_payloads.push("unrelated".into()),
                8 => expected
                    .declaration_symbol_payloads
                    .push("unrelated".into()),
                9 => expected.spec_refs.push(expected.spec_refs[1].clone()),
                10 => {
                    expected.spec_refs.remove(0);
                }
                _ => unreachable!(),
            }
            assert!(
                !super::super::proof_verification::step5c14_return_admitted(
                    Some(&config.workspace_root),
                    &case
                ),
                "mutation {mutation}"
            );
            assert!(!super::super::is_active_proof_verification(&case));
            assert!(
                crate::expectation::validate_expectation_path(
                    &case.expectation_path,
                    &case.expectation,
                    &config.workspace_root.join("tests")
                )
                .iter()
                .any(|diagnostic| diagnostic.code.0 == "E-EXPECT-SNAPSHOT-SCOPE"),
                "snapshot mutation {mutation}"
            );
        }
        for donor in [
            "pass_proof_verification_algorithm_ensures_return_001",
            "pass_proof_verification_contradiction_formula_constant_001",
        ] {
            for alias in 0..4 {
                let mut case = plan
                    .cases
                    .iter()
                    .find(|case| case.id.0 == donor)
                    .unwrap()
                    .clone();
                match alias {
                    0 => case.id = original.id.clone(),
                    1 => case.expectation.id = original.id.clone(),
                    2 => case.source_path = original.source_path.clone(),
                    3 => case.expectation_path = original.expectation_path.clone(),
                    _ => unreachable!(),
                }
                assert!(
                    !super::super::is_active_proof_verification(&case),
                    "{donor} alias {alias}"
                );
                let run = super::super::proof_verification::run_proof_verification_case(
                    &config.workspace_root,
                    &config.workspace_root.join("tests"),
                    &case,
                    0,
                );
                assert_eq!(run.status, super::super::ProofVerificationCaseStatus::Failed);
            }
        }
    }
}
#[test]
fn step5c14_claim_rejects_genuine_foreign_and_stale_algorithm_seals() {
    use mizar_checker::type_checker::{
        SourceVariableSemanticsChecker, check_source_algorithm_types,
    };
    use mizar_resolve::{
        labels::{LabelResolver, ProofLabelSourceCollector},
        names::{SourceVariableScopeInput, SourceVariableScopeResolver},
    };
    let root = step5c11_config().workspace_root;
    let case = step5c14_claim_case();
    let text = std::fs::read_to_string(&case.source_path).unwrap();
    let frontend = super::super::formula_statement::step5c8_test_frontend(&text);
    let ast = frontend.ast.clone().unwrap();
    let (source, nodes, symbols) =
        super::super::source_registration_inputs(&root, &case, frontend).unwrap();
    let algorithm = check_source_algorithm_types(&source, &nodes, &symbols).unwrap();
    let scope = SourceVariableScopeResolver::resolve_proof_occurrences(
        SourceVariableScopeInput::new(&ast, source.module(), &symbols),
    )
    .unwrap();
    let bindings = SourceVariableSemanticsChecker::occurrence_binding_env(&scope);
    let typed = super::super::type_elaboration::step5c8_formula_typed_ast(
        &ast,
        source.module(),
        &symbols,
        &scope,
        &bindings,
        true,
    )
    .unwrap();
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
    let checked = SourceVariableSemanticsChecker::check_theorem_skeletons(
        &source,
        &typed,
        &scope,
        &symbols,
        &labels,
        &resolved,
        Some(&algorithm),
    )
    .unwrap();
    mizar_core::elaborator::lower_source_theorem_skeletons(&checked, Some(&algorithm)).unwrap();
    for variant in ["module", "source", "owner_tree"] {
        let mut alternate_case = case.clone();
        if variant == "module" {
            alternate_case.source_path =
                root.join("tests/miz/pass/algorithms/foreign_claim_module.miz");
        }
        let alternate_text = if variant == "owner_tree" {
            text.replace("Cl1", "Cl2")
        } else {
            text.clone()
        };
        let frontend = if variant == "source" {
            use mizar_frontend::{
                orchestration::Frontend,
                parsing::MizarParserSeam,
                source::{FrontendSourceLoader, SourceUnitRequest},
            };
            use mizar_session::{
                DiskSourceLoader, Edition, InMemorySessionIdAllocator, ModulePath, PackageId,
                SessionIdAllocator, SourceInput, SourceOriginInput,
            };
            let temporary = std::process::Command::new("mktemp")
                .arg("-d")
                .output()
                .unwrap();
            assert!(temporary.status.success());
            let directory =
                std::path::PathBuf::from(String::from_utf8(temporary.stdout).unwrap().trim());
            std::fs::create_dir(directory.join("src")).unwrap();
            let path = directory.join("src/foreign_source.miz");
            std::fs::write(&path, &alternate_text).unwrap();
            let ids = InMemorySessionIdAllocator::new();
            let snapshot = super::super::shared::snapshot_id(5814);
            ids.next_source_id(snapshot).unwrap();
            let output = Frontend::new(
                FrontendSourceLoader::new(DiskSourceLoader::new(&directory)),
                super::super::ParseOnlyImportProvider,
                MizarParserSeam,
            )
            .run(
                SourceUnitRequest {
                    snapshot,
                    input: SourceInput {
                        package_id: PackageId::new("claim-seal"),
                        module_path: ModulePath::new("foreign_source"),
                        normalized_path: mizar_session::normalize_path(&directory, &path).unwrap(),
                        edition: Edition::new("2026"),
                        origin: SourceOriginInput::Disk { path: path.clone() },
                    },
                },
                &ids,
            )
            .unwrap();
            std::fs::remove_dir_all(directory).unwrap();
            super::super::shared::FrontendRun {
                source_text: alternate_text.clone().into(),
                ast: output.ast,
                ast_snapshot: None,
                diagnostics: output.diagnostics,
            }
        } else {
            super::super::formula_statement::step5c8_test_frontend(&alternate_text)
        };
        let alternate_ast = frontend.ast.clone().unwrap();
        let (alternate_source, alternate_nodes, alternate_symbols) =
            super::super::source_registration_inputs(&root, &alternate_case, frontend).unwrap();
        match variant {
            "module" => {
                assert_ne!(alternate_source.module(), source.module());
                assert_eq!(alternate_source.source_id(), source.source_id());
            }
            "source" => {
                assert_eq!(alternate_source.module(), source.module());
                assert_ne!(alternate_source.source_id(), source.source_id());
            }
            "owner_tree" => {
                assert_eq!(alternate_source.module(), source.module());
                assert_eq!(alternate_source.source_id(), source.source_id());
                assert_eq!(alternate_nodes.len(), nodes.len());
                assert_ne!(alternate_nodes, nodes);
            }
            _ => unreachable!(),
        }
        let alternate_algorithm =
            check_source_algorithm_types(&alternate_source, &alternate_nodes, &alternate_symbols)
                .unwrap();
        let alternate_scope =
            SourceVariableScopeResolver::resolve_proof_occurrences(SourceVariableScopeInput::new(
                &alternate_ast,
                alternate_source.module(),
                &alternate_symbols,
            ))
            .unwrap();
        let alternate_bindings =
            SourceVariableSemanticsChecker::occurrence_binding_env(&alternate_scope);
        let alternate_typed = super::super::type_elaboration::step5c8_formula_typed_ast(
            &alternate_ast,
            alternate_source.module(),
            &alternate_symbols,
            &alternate_scope,
            &alternate_bindings,
            true,
        )
        .unwrap();
        let alternate_owner = alternate_symbols
            .symbols()
            .iter()
            .find(|entry| entry.kind() == mizar_resolve::env::SymbolKind::Theorem)
            .unwrap();
        let alternate_namespace =
            mizar_resolve::env::NamespacePath::new(alternate_source.module().path().as_str());
        let alternate_labels = ProofLabelSourceCollector::new(
            &alternate_ast,
            alternate_source.module(),
            alternate_namespace.clone(),
            alternate_owner.contribution(),
            &alternate_source,
        )
        .unwrap()
        .collect_with_theorem_owners(&alternate_symbols)
        .unwrap();
        let alternate_resolved = LabelResolver::new(alternate_labels.projections()).resolve(
            alternate_source.module(),
            &alternate_namespace,
            alternate_labels.references(),
        );
        let alternate_checked = SourceVariableSemanticsChecker::check_theorem_skeletons(
            &alternate_source,
            &alternate_typed,
            &alternate_scope,
            &alternate_symbols,
            &alternate_labels,
            &alternate_resolved,
            Some(&alternate_algorithm),
        )
        .unwrap();
        mizar_core::elaborator::lower_source_theorem_skeletons(
            &alternate_checked,
            Some(&alternate_algorithm),
        )
        .unwrap();
        assert!(
            SourceVariableSemanticsChecker::check_theorem_skeletons(
                &source,
                &typed,
                &scope,
                &symbols,
                &labels,
                &resolved,
                Some(&alternate_algorithm)
            )
            .is_err(),
            "checker accepted {variant} seal"
        );
        assert!(
            mizar_core::elaborator::lower_source_theorem_skeletons(
                &checked,
                Some(&alternate_algorithm)
            )
            .is_err(),
            "Core accepted {variant} seal"
        );
    }
}

#[test]
fn step5c14_assert_failure_preserves_real_negation_and_open_vc() {
    use mizar_checker::type_checker::{FormulaKind, TermReference, check_source_algorithm_types};
    use mizar_core::core_ir::{
        CoreAlgorithmStmtKind as S, CoreFormulaKind as F, CoreTermKind as T, ObligationSeedStatus,
    };
    use mizar_vc::{
        discharge::failed_source_algorithm_assertion,
        vc_ir::{
            ContextEntryKind, PremiseRef, SeedVcMapping, VcFormulaRef,
            VcGeneratedFormulaShape as G, VcKind, VcProgramValue, VcStatus,
        },
    };
    let case = step5c14_assert_failure_case();
    let text = std::fs::read_to_string(&case.source_path).unwrap();
    for (text, negative, baseline) in [
        (
            text.clone(),
            true,
            Some("fail_proof_verification_algorithm_assert_unprovable_001.vc_ir.snap"),
        ),
        (
            text.replace("not a = a", "a = a"),
            false,
            Some("step5c14_algorithm_assert_reflexive.vc_ir.snap"),
        ),
        (text.replace("badassert", "renamed"), true, None),
        (
            text.replace("let a be", "let input be")
                .replace("(a)", "(input)")
                .replace("a = a", "input = input")
                .replace("return a", "return input"),
            true,
            None,
        ),
        (
            text.replace("terminating algorithm", "algorithm"),
            true,
            None,
        ),
    ] {
        let frontend = super::super::formula_statement::step5c8_test_frontend(&text);
        assert!(frontend.diagnostics.is_empty());
        let (source, typed, symbols) =
            super::super::source_registration_inputs(&step5c11_config().workspace_root, &case, frontend)
                .unwrap();
        let checked = check_source_algorithm_types(&source, &typed, &symbols).unwrap();
        assert_eq!(
            checked.inference().formulas().len(),
            if negative { 2 } else { 1 }
        );
        let equality = checked
            .inference()
            .formulas()
            .iter()
            .find(|(_, formula)| formula.kind == FormulaKind::Equality)
            .unwrap()
            .1;
        let operand_bindings = equality
            .terms
            .iter()
            .map(|site| {
                checked
                    .inference()
                    .terms()
                    .iter()
                    .find(|(_, term)| term.site == *site)
                    .unwrap()
                    .1
                    .reference
                    .clone()
            })
            .collect::<Vec<_>>();
        assert_eq!(operand_bindings[0], operand_bindings[1]);
        assert!(matches!(
            operand_bindings[0],
            Some(TermReference::Binding(_))
        ));
        let core = mizar_core::elaborator::lower_source_algorithms(&checked).unwrap();
        assert_eq!(core, step5c14_static_core(&case, &text).unwrap());
        assert_eq!(
            (
                core.items().len(),
                core.algorithms().len(),
                core.algorithm_statements().len(),
                core.terms().len(),
                core.formulas().len()
            ),
            (1, 1, 2, 6, if negative { 4 } else { 3 })
        );
        let (_, algorithm) = core.algorithms().iter().next().unwrap();
        assert_ne!(
            algorithm.params[0].var,
            algorithm.result.as_ref().unwrap().var
        );
        assert!(algorithm.contracts.ensures.is_empty());
        let S::Assert { formula } = core
            .algorithm_statements()
            .get(algorithm.statements[0])
            .unwrap()
            .kind
        else {
            panic!("assertion")
        };
        let inner = if negative {
            let F::Not(inner) = core.formulas().get(formula).unwrap().kind else {
                panic!("actual not")
            };
            inner
        } else {
            formula
        };
        let F::Equals { left, right } = core.formulas().get(inner).unwrap().kind else {
            panic!("actual equality")
        };
        assert_ne!(left, right);
        for operand in [left, right] {
            assert_eq!(
                core.terms().get(operand).unwrap().kind,
                T::Var(algorithm.params[0].var)
            );
        }
        assert_ne!(
            core.terms().get(left).unwrap().source,
            core.terms().get(right).unwrap().source
        );
        let vcs = step5c14_assert_failure_vcs(&core).unwrap();
        assert_eq!(vcs, step5c14_assert_failure_vcs(&core).unwrap());
        assert_eq!(
            (
                vcs.vcs().len(),
                vcs.generated_formulas().len(),
                vcs.seed_accounting().len()
            ),
            (1, if negative { 2 } else { 1 }, 2)
        );
        let vc = &vcs.vcs()[0];
        assert_eq!(vc.kind, VcKind::AlgorithmAssertion);
        assert_eq!(vc.status, VcStatus::Open);
        assert!(vc.proof_hint.is_none());
        assert_eq!(vc.local_context.entries().len(), 1);
        let guard = &vc.local_context.entries()[0];
        assert_eq!(guard.kind, ContextEntryKind::CheckerFact);
        assert_eq!(
            guard.formula,
            algorithm.params[0].ty_guard.map(VcFormulaRef::Core)
        );
        assert_eq!(vc.premises, [PremiseRef::LocalContext(guard.id)]);
        let value = VcProgramValue {
            var: algorithm.params[0].var,
            definition: None,
        };
        assert_eq!(
            vcs.generated_formulas()[0].shape,
            G::ProgramEquals {
                left: value,
                right: value
            }
        );
        let goal = if negative {
            assert_eq!(
                vcs.generated_formulas()[1].shape,
                G::Not(VcFormulaRef::Generated(vcs.generated_formulas()[0].id))
            );
            vcs.generated_formulas()[1].id
        } else {
            vcs.generated_formulas()[0].id
        };
        assert_eq!(vc.goal, VcFormulaRef::Generated(goal));
        assert_eq!(
            vcs.seed_accounting()
                .iter()
                .filter(|row| row.mapping == SeedVcMapping::One { vc: vc.id })
                .count(),
            1
        );
        assert_eq!(
            vcs.seed_accounting()
                .iter()
                .filter(|row| row.seed_status == ObligationSeedStatus::Deferred
                    && matches!(row.mapping, SeedVcMapping::NoConcreteVc { .. }))
                .count(),
            1
        );
        assert_eq!(
            failed_source_algorithm_assertion(&core, &vcs).unwrap(),
            negative.then_some(vc.id)
        );
        let ordinary = mizar_vc::discharge::try_discharge(mizar_vc::discharge::DischargeInput {
            vc_set: &vcs,
            policy: &mizar_vc::discharge::DischargePolicy::default(),
        })
        .unwrap();
        assert!(ordinary.evidence_records().is_empty());
        assert_eq!(ordinary.vc_set().vcs()[0].status, VcStatus::NeedsAtp);
        assert!(failed_source_algorithm_assertion(&core, ordinary.vc_set()).is_err());
        if let Some(baseline) = baseline {
            assert_eq!(
                vcs.debug_text(),
                std::fs::read_to_string(
                    step5c11_config()
                        .workspace_root
                        .join("tests/snapshots/vc")
                        .join(baseline)
                )
                .unwrap()
            );
        }
    }
}

#[test]
fn step5c14_assert_failure_source_and_seal_controls() {
    use mizar_checker::{
        type_checker::check_source_algorithm_types,
        typed_ast::{NodeRecoveryState, TypedArena},
    };
    let case = step5c14_assert_failure_case();
    let text = std::fs::read_to_string(&case.source_path).unwrap();
    for (from, to) in [
        ("not a = a", "not missing = a"),
        ("not a = a", "not a = missing"),
        ("let a be object", "let a be set"),
        ("-> object", "-> set"),
        ("return a", "return missing"),
        ("not a = a", "not not a = a"),
        ("not a = a", "a <> a"),
    ] {
        let changed = text.replace(from, to);
        assert_ne!(changed, text);
        assert!(
            super::super::formula_statement::step5c8_test_frontend(&changed)
                .diagnostics
                .is_empty(),
            "{from} -> {to}"
        );
        assert!(
            step5c14_static_core(&case, &changed).is_err(),
            "{from} -> {to}"
        );
    }
    let malformed = text.replace("assert not a = a;", "assert not ;");
    assert!(
        !super::super::formula_statement::step5c8_test_frontend(&malformed)
            .diagnostics
            .is_empty()
    );
    assert!(step5c14_static_core(&case, &malformed).is_err());
    let duplicate = format!("{text}\n{text}");
    assert!(
        super::super::formula_statement::step5c8_test_frontend(&duplicate)
            .diagnostics
            .is_empty()
    );
    assert_eq!(
        step5c14_static_core(&case, &duplicate).unwrap_err(),
        "registration.resolver_diagnostics"
    );
    let (source, typed, symbols) = super::super::source_registration_inputs(
        &step5c11_config().workspace_root,
        &case,
        super::super::formula_statement::step5c8_test_frontend(&text),
    )
    .unwrap();
    let (_, _, foreign_symbols) = super::super::source_registration_inputs(
        &step5c11_config().workspace_root,
        &case,
        super::super::formula_statement::step5c8_test_frontend(&text.replace("badassert", "foreign")),
    )
    .unwrap();
    assert!(check_source_algorithm_types(&source, &typed, &foreign_symbols).is_err());
    let negation = typed
        .iter()
        .find(|(_, node)| node.kind.as_str() == "PrefixFormula(Not)")
        .unwrap()
        .0;
    for mutation in 0..4 {
        let mut rows = typed
            .iter()
            .map(|(_, node)| node.clone())
            .collect::<Vec<_>>();
        match mutation {
            0 => rows[negation.index()].recovery = NodeRecoveryState::Recovered,
            1 => rows[negation.index()].children.reverse(),
            2 => rows[negation.index()].kind = "FormulaConstant(Contradiction)".into(),
            3 => rows[negation.index()].anchor = rows[0].anchor.clone(),
            _ => unreachable!(),
        }
        let changed = TypedArena::try_new(typed.root(), rows).unwrap();
        assert!(check_source_algorithm_types(&source, &changed, &symbols).is_err());
    }
    for changed in [
        text.replace("assert not a = a;", "var x := a; assert not x = a;"),
        text.replace("assert not a = a;", "var x := a; x := a; assert not x = x;"),
        text.replace("assert not a = a;", "assert a = a; assert not a = a;"),
    ] {
        assert!(
            super::super::formula_statement::step5c8_test_frontend(&changed)
                .diagnostics
                .is_empty()
        );
        let core = step5c14_static_core(&case, &changed).unwrap();
        let vcs = step5c14_assert_failure_vcs(&core).unwrap();
        assert_eq!(
            mizar_vc::discharge::failed_source_algorithm_assertion(&core, &vcs).unwrap(),
            None
        );
        assert!(
            vcs.vcs()
                .iter()
                .all(|vc| vc.status == mizar_vc::vc_ir::VcStatus::Open)
        );
    }
}

#[test]
fn step5c14_assert_failure_rejects_coherent_core_corruption() {
    use mizar_core::core_ir::*;
    let case = step5c14_assert_failure_case();
    let core =
        step5c14_static_core(&case, &std::fs::read_to_string(&case.source_path).unwrap()).unwrap();
    let original = step5c14_assert_failure_vcs(&core).unwrap();
    let (algorithm_id, algorithm) = core.algorithms().iter().next().unwrap();
    let assertion = algorithm.statements[0];
    let CoreAlgorithmStmtKind::Assert { formula } =
        core.algorithm_statements().get(assertion).unwrap().kind
    else {
        unreachable!()
    };
    let CoreFormulaKind::Not(inner) = core.formulas().get(formula).unwrap().kind else {
        unreachable!()
    };
    let CoreFormulaKind::Equals { left, right } = core.formulas().get(inner).unwrap().kind else {
        unreachable!()
    };
    let parameter = &algorithm.params[0];
    let result = algorithm.result.as_ref().unwrap();
    for mutation in 0..18 {
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
            0 => parts.formulas.get_mut(formula).unwrap().kind = CoreFormulaKind::False,
            1 => {
                parts.formulas.get_mut(formula).unwrap().kind =
                    CoreFormulaKind::Not(parameter.ty_guard.unwrap())
            }
            2 => {
                parts.formulas.get_mut(inner).unwrap().kind =
                    CoreFormulaKind::Equals { left, right: left }
            }
            3 => parts.terms.get_mut(left).unwrap().kind = CoreTermKind::Var(result.var),
            4 => parts.terms.get_mut(right).unwrap().kind = CoreTermKind::Var(result.var),
            5 => {
                parts
                    .formulas
                    .get_mut(parameter.ty_guard.unwrap())
                    .unwrap()
                    .kind = CoreFormulaKind::True
            }
            6 => parts
                .formulas
                .get_mut(formula)
                .unwrap()
                .source
                .provenance
                .clear(),
            7 => {
                parts.formulas.get_mut(inner).unwrap().source.anchor =
                    core.formulas().get(formula).unwrap().source.anchor.clone()
            }
            8 => {
                parts.formulas.get_mut(formula).unwrap().source.anchor =
                    core.formulas().get(inner).unwrap().source.anchor.clone()
            }
            9 => {
                parts.terms.get_mut(left).unwrap().source.anchor =
                    core.terms().get(right).unwrap().source.anchor.clone()
            }
            10 => parts
                .algorithms
                .get_mut(algorithm_id)
                .unwrap()
                .statements
                .reverse(),
            11 => parts.algorithms.get_mut(algorithm_id).unwrap().params[0].var = result.var,
            12 => {
                parts.algorithms.get_mut(algorithm_id).unwrap().params[0].ty_guard = result.ty_guard
            }
            13 => {
                parts
                    .formulas
                    .insert(core.formulas().get(inner).unwrap().clone());
            }
            14 => {
                parts.terms.insert(core.terms().get(left).unwrap().clone());
            }
            15 => parts.items.get_mut(algorithm.item).unwrap().visibility = "private".into(),
            16 => {
                parts
                    .algorithm_statements
                    .get_mut(assertion)
                    .unwrap()
                    .source
                    .anchor = algorithm.source.anchor.clone()
            }
            17 => {
                parts.algorithms.get_mut(algorithm_id).unwrap().params[0].role = "local:var".into()
            }
            _ => unreachable!(),
        }
        parts.source_map.item_sources = parts
            .items
            .iter()
            .map(|(id, row)| (id, row.source.clone()))
            .collect();
        parts.source_map.term_sources = parts
            .terms
            .iter()
            .map(|(id, row)| (id, row.source.clone()))
            .collect();
        parts.source_map.formula_sources = parts
            .formulas
            .iter()
            .map(|(id, row)| (id, row.source.clone()))
            .collect();
        parts.source_map.algorithm_sources = parts
            .algorithm_statements
            .iter()
            .map(|(id, row)| (id, row.source.clone()))
            .collect();
        let changed =
            CoreIr::try_new(parts).unwrap_or_else(|error| panic!("mutation {mutation}: {error}"));
        assert!(
            step5c14_assert_failure_vcs(&changed).is_err(),
            "mutation {mutation}"
        );
        assert!(
            mizar_vc::discharge::failed_source_algorithm_assertion(&changed, &original).is_err(),
            "stale result {mutation}"
        );
    }
}

#[test]
fn step5c14_assert_failure_replays_every_vc_field_before_observation() {
    use mizar_core::core_ir::{CoreAlgorithmId, CoreAlgorithmStmtId, ObligationSeedStatus};
    use mizar_vc::{discharge::failed_source_algorithm_assertion, vc_ir::*};
    let case = step5c14_assert_failure_case();
    let text = std::fs::read_to_string(&case.source_path).unwrap();
    let core = step5c14_static_core(&case, &text).unwrap();
    let original = step5c14_assert_failure_vcs(&core).unwrap();
    let (_, algorithm) = core.algorithms().iter().next().unwrap();
    for mutation in 0..17 {
        let mut parts = VcSetParts {
            schema_version: original.schema_version().clone(),
            snapshot: original.snapshot(),
            source: original.source(),
            module: original.module().clone(),
            generated_formulas: original.generated_formulas().to_vec(),
            vcs: original.vcs().to_vec(),
            seed_accounting: original.seed_accounting().to_vec(),
        };
        match mutation {
            0 => parts.vcs[0].status = VcStatus::NeedsAtp,
            1 => parts.vcs[0].kind = VcKind::AlgorithmPostcondition,
            2 => parts.vcs[0].goal = VcFormulaRef::Generated(parts.generated_formulas[0].id),
            3 => parts.vcs[0].source.primary = algorithm.source.clone(),
            4 => parts.vcs[0].anchor.owner = AnchorOwner::Algorithm(CoreAlgorithmId::new(99)),
            5 => parts.vcs[0].source.related.clear(),
            6 => parts.vcs[0].premises.clear(),
            7 => {
                parts.vcs[0].proof_hint = Some(ProofHint {
                    citations: vec![PremiseRef::LocalContext(ContextEntryId::new(0))],
                    unfold_requests: vec![],
                    premise_restrictions: vec![],
                    solver: None,
                    max_axioms: None,
                    timeout: None,
                    computation: None,
                    provenance: vec![],
                })
            }
            8..=10 => {
                let VcGeneratedFormulaShape::ProgramEquals { left, right } =
                    &mut parts.generated_formulas[0].shape
                else {
                    unreachable!()
                };
                match mutation {
                    8 => left.var = algorithm.result.as_ref().unwrap().var,
                    9 => right.var = algorithm.result.as_ref().unwrap().var,
                    10 => right.definition = Some(CoreAlgorithmStmtId::new(0)),
                    _ => unreachable!(),
                }
            }
            11 => {
                parts.generated_formulas[1].shape = VcGeneratedFormulaShape::Not(
                    VcFormulaRef::Core(algorithm.params[0].ty_guard.unwrap()),
                )
            }
            12 => parts.generated_formulas[0].provenance.clear(),
            13 => {
                let mut extra = parts.generated_formulas[0].clone();
                extra.id = VcGeneratedFormulaId::new(parts.generated_formulas.len());
                parts.generated_formulas.push(extra);
            }
            14 | 15 => {
                let mut entries = parts.vcs[0].local_context.entries().to_vec();
                entries[0].formula = Some(if mutation == 14 {
                    parts.vcs[0].goal
                } else {
                    VcFormulaRef::Core(algorithm.result.as_ref().unwrap().ty_guard.unwrap())
                });
                parts.vcs[0].local_context = LocalContext::try_new(
                    entries,
                    parts.vcs[0].local_context.policy_inputs().to_vec(),
                )
                .unwrap();
            }
            16 => parts.seed_accounting[0].seed_status = ObligationSeedStatus::Active,
            _ => unreachable!(),
        }
        let changed =
            VcSet::try_new(parts).unwrap_or_else(|error| panic!("mutation {mutation}: {error}"));
        assert!(
            failed_source_algorithm_assertion(&core, &changed).is_err(),
            "mutation {mutation}"
        );
    }
    // A pending assertion belongs to an earlier actual VC, never to its own goal.
    let state = text.replace("assert not a = a;", "assert a = a; assert not a = a;");
    let core = step5c14_static_core(&case, &state).unwrap();
    let original = step5c14_assert_failure_vcs(&core).unwrap();
    let mut parts = VcSetParts {
        schema_version: original.schema_version().clone(),
        snapshot: original.snapshot(),
        source: original.source(),
        module: original.module().clone(),
        generated_formulas: original.generated_formulas().to_vec(),
        vcs: original.vcs().to_vec(),
        seed_accounting: original.seed_accounting().to_vec(),
    };
    let vc = parts
        .vcs
        .iter_mut()
        .find(|vc| {
            vc.local_context.entries().iter().any(|entry| {
                matches!(
                    entry.kind,
                    ContextEntryKind::PendingAlgorithmAssertion { .. }
                )
            })
        })
        .expect("second source assertion retains its actual pending predecessor");
    let mut entries = vc.local_context.entries().to_vec();
    let pending = entries
        .iter_mut()
        .find(|entry| {
            matches!(
                entry.kind,
                ContextEntryKind::PendingAlgorithmAssertion { .. }
            )
        })
        .unwrap();
    pending.kind = ContextEntryKind::PendingAlgorithmAssertion {
        handoff: vc.seed.handoff,
    };
    vc.local_context =
        LocalContext::try_new(entries, vc.local_context.policy_inputs().to_vec()).unwrap();
    assert!(matches!(
        VcSet::try_new(parts),
        Err(VcIrError::InvalidPendingAlgorithmAssertion { .. })
    ));
}

#[test]
fn step5c14_pick_source_context_and_foundational_failure() {
    use mizar_core::{core_ir::{CoreAlgorithmStmtKind as S, CoreFormulaKind as F, CoreTermKind as T, ObligationSeedKind, ObligationSeedStatus}, control_flow::{build_control_flow_ir, build_obligation_seed_handoff, LocalDeclaration, LocalMutability}};
    use mizar_vc::{discharge::failed_source_algorithm_assertion, vc_ir::{VcKind, VcStatus, VcFormulaRef, VcGeneratedFormulaShape as G, VcProgramValue}};
    let config = step5c11_config();
    let case = build_test_plan(&config).unwrap().cases.into_iter().find(|c| c.id.0 == "fail_proof_verification_algorithm_ensures_unprovable_001").unwrap();
    let text = std::fs::read_to_string(&case.source_path).unwrap();
    for text in [text.clone(), text.replace("badalgo", "selectset").replace("let a be", "let x be").replace("result = a", "result = x").replace("(a)", "(x)")] {
        let (source, typed, symbols) = super::super::source_registration_inputs(&config.workspace_root, &case, super::super::formula_statement::step5c8_test_frontend(&text)).unwrap();
        let checked = mizar_checker::type_checker::check_source_algorithm_types(&source, &typed, &symbols).unwrap();
        let choice = checked.inference().terms().iter().find(|(_, t)| t.kind == mizar_checker::type_checker::TermKind::Choice).unwrap().1;
        assert!(choice.reference.is_none());
        let entry = checked.inference().type_entries().get(choice.type_entry).unwrap();
        let mizar_checker::typed_ast::TypeEntryActual::Known(actual) = entry.actual else { panic!() };
        assert_eq!(checked.inference().normalized_types().get(actual).unwrap().head, mizar_checker::type_checker::TypeHeadRef::BuiltinSet);
        assert_eq!(checked.inference().normalized_types().get(entry.expected.unwrap()).unwrap().head, mizar_checker::type_checker::TypeHeadRef::BuiltinObject);
        for kind in ["ChoiceTerm", "TypeExpression", "ReturnStatement"] {
            let mut nodes = typed.iter().map(|(_, n)| n.clone()).collect::<Vec<_>>();
            let index = nodes.iter().position(|n| n.kind.as_str() == kind).unwrap();
            nodes[index].kind = "TermReference".into();
            let forged = mizar_checker::typed_ast::TypedArena::try_new(typed.root(), nodes).unwrap();
            assert!(mizar_checker::type_checker::check_source_algorithm_types(&source, &forged, &symbols).is_err());
        }
        let core = mizar_core::elaborator::lower_source_algorithms(&checked).unwrap();
        assert_eq!(core, step5c14_static_core(&case, &text).unwrap());
        assert!(core.generated().is_empty());
        let (_, algorithm) = core.algorithms().iter().next().unwrap();
        let [pick, returned] = algorithm.statements.as_slice() else { panic!() };
        let pick_row = core.algorithm_statements().get(*pick).unwrap();
        let S::Pick { binder, witness_ty, ghost: false } = &pick_row.kind else { panic!() };
        assert!(binder.source_name.is_none());
        assert_eq!(binder.ty_guard, *witness_ty);
        let S::Return(Some(value)) = core.algorithm_statements().get(*returned).unwrap().kind else { panic!() };
        assert_eq!(core.terms().get(value).unwrap().kind, T::Var(binder.var));
        assert_eq!(core.terms().get(value).unwrap().source, pick_row.source);
        let seed = core.obligation_seeds().iter().next().unwrap().1;
        assert_eq!(seed.kind, ObligationSeedKind::GeneratedNonEmptiness);
        assert_eq!(seed.status, ObligationSeedStatus::Active);
        assert!(seed.context.is_empty());
        let F::Exists { binders, body } = &core.formulas().get(seed.goal.unwrap()).unwrap().kind else { panic!() };
        assert_eq!(binders.len(), 1);
        let q = &binders[0];
        assert_ne!(q.var, binder.var);
        assert_ne!(q.var, algorithm.params[0].var);
        assert_ne!(q.var, algorithm.result.as_ref().unwrap().var);
        let F::TypePred { subject, ty } = &core.formulas().get(*body).unwrap().kind else { panic!() };
        assert_eq!(ty.as_str(), "set");
        assert_eq!(core.terms().get(*subject).unwrap().kind, T::Var(q.var));
        let flow = build_control_flow_ir(&core);
        assert_eq!(flow, build_control_flow_ir(&core));
        let (_, cfg) = flow.flows.iter().next().unwrap();
        assert_eq!(cfg.blocks.len(), 1);
        assert_eq!(cfg.assignment_effects.len(), 1);
        let local = cfg.locals.iter().find(|(_, l)| l.binder.var == binder.var).unwrap().1;
        assert_eq!(local.declaration, LocalDeclaration::PickRuntime);
        assert_eq!(local.mutability, LocalMutability::Immutable);
        assert!(!local.ghost);
        assert_eq!(local.initialized_at, Some(*pick));
        assert_eq!(build_obligation_seed_handoff(&core, &flow).entries.len(), 3);
        let vcs = step5c14_return_vcs(&core).unwrap();
        assert_eq!(vcs.vcs().len(), 2);
        assert_eq!(vcs.seed_accounting().len(), 3);
        assert!(vcs.vcs().iter().all(|v| v.status == VcStatus::Open));
        let nonempty = vcs.vcs().iter().find(|v| v.goal == VcFormulaRef::Core(seed.goal.unwrap())).unwrap();
        assert!(nonempty.local_context.entries().is_empty());
        assert!(nonempty.premises.is_empty());
        let post = vcs.vcs().iter().find(|v| v.kind == VcKind::AlgorithmPostcondition).unwrap();
        assert_eq!(post.local_context.entries().len(), 2);
        assert_eq!(post.premises.len(), 2);
        assert_eq!(post.local_context.entries()[0].formula, algorithm.params[0].ty_guard.map(VcFormulaRef::Core));
        assert!(matches!(&vcs.generated_formulas()[0].shape, G::ProgramTypePredicate { subject, ty } if subject.var == binder.var && subject.definition == Some(*pick) && ty.as_str() == "set"));
        assert_eq!(vcs.generated_formulas()[1].shape, G::ProgramEquals {
            left: VcProgramValue { var: binder.var, definition: Some(*pick) }, right: VcProgramValue { var: algorithm.params[0].var, definition: None }
        });
        let before = vcs.clone();
        assert_eq!(failed_source_algorithm_assertion(&core, &vcs).unwrap(), Some(post.id));
        assert_eq!(vcs, before);
        assert_eq!(vcs, step5c14_return_vcs(&core).unwrap());
    }
    for control in [text.clone(), text.replace("the set", "a"), text.replace("result = a", "result = result"), text.replace("result = a", "a = a")] {
        use mizar_core::core_ir::*;
        let core = step5c14_static_core(&case, &control).unwrap();
        let observed = failed_source_algorithm_assertion(&core, &step5c14_return_vcs(&core).unwrap()).unwrap();
        assert_eq!(observed.is_some(), control == text);
        let mut p = CoreIrParts { source_id: core.source_id(), module_id: core.module_id().clone(), items: core.items().clone(), terms: core.terms().clone(), formulas: core.formulas().clone(), definitions: core.definitions().clone(), proofs: core.proofs().clone(), proof_nodes: core.proof_nodes().clone(), algorithms: core.algorithms().clone(), algorithm_statements: core.algorithm_statements().clone(), generated: core.generated().clone(), obligation_seeds: core.obligation_seeds().clone(), source_map: core.source_map().clone(), diagnostics: core.diagnostics().clone() };
        let renumber = |var: &mut CoreVarId| *var = CoreVarId::new(var.index() + 100);
        for (id, _) in core.terms().iter() { if let T::Var(var) = &mut p.terms.get_mut(id).unwrap().kind { renumber(var); } }
        for (id, _) in core.algorithms().iter() {
            let algorithm = p.algorithms.get_mut(id).unwrap();
            for binder in &mut algorithm.params { renumber(&mut binder.var); }
            renumber(&mut algorithm.result.as_mut().unwrap().var);
        }
        for (id, _) in core.algorithm_statements().iter() {
            if let S::Pick { binder, .. } = &mut p.algorithm_statements.get_mut(id).unwrap().kind { renumber(&mut binder.var); }
        }
        for (id, _) in core.formulas().iter() {
            if let F::Exists { binders, .. } = &mut p.formulas.get_mut(id).unwrap().kind { for binder in binders { renumber(&mut binder.var); } }
        }
        let renumbered = CoreIr::try_new(p).unwrap();
        assert_eq!(failed_source_algorithm_assertion(&renumbered, &step5c14_return_vcs(&renumbered).unwrap()).unwrap().is_some(), observed.is_some());
    }
    for unsupported in [text.replace("the set", "the object"), text.replace("the set", "the empty set"), text.replace("return the set;", "assert a = a; return the set;"), text.replace("ensures result = a", "requires a = a ensures result = a"), format!("environ vocabularies X; begin {text}"), text.replace("end;\n", "end;\ntheorem a = a;\n")] {
        assert!(step5c14_static_core(&case, &unsupported).is_err(), "{unsupported}");
    }
}

#[test]
fn step5c14_pick_rejects_core_forgery_and_replays_complete_vcs() {
    use mizar_core::core_ir::*;
    use mizar_vc::{discharge::failed_source_algorithm_assertion, vc_ir::*};
    let case = step5c14_return_case();
    let text = "definition let a be object; terminating algorithm choose(a) -> object ensures result = a do return the set; end; end;";
    let core = step5c14_static_core(&case, text).unwrap();
    let vcs = step5c14_return_vcs(&core).unwrap();
    let (algorithm_id, algorithm) = core.algorithms().iter().next().unwrap();
    let pick = algorithm.statements[0];
    let returned = algorithm.statements[1];
    let CoreAlgorithmStmtKind::Pick { binder, .. } = &core.algorithm_statements().get(pick).unwrap().kind else { panic!() };
    let CoreAlgorithmStmtKind::Return(Some(value)) = core.algorithm_statements().get(returned).unwrap().kind else { panic!() };
    let (seed_id, seed) = core.obligation_seeds().iter().next().unwrap();
    let exists = seed.goal.unwrap();
    let CoreFormulaKind::Exists { body, .. } = core.formulas().get(exists).unwrap().kind else { panic!() };
    let mut constructor_rejections = 0;
    let mut generator_rejections = 0;
    for mutation in 0..19 {
        let mut p = CoreIrParts { source_id: core.source_id(), module_id: core.module_id().clone(), items: core.items().clone(), terms: core.terms().clone(), formulas: core.formulas().clone(), definitions: core.definitions().clone(), proofs: core.proofs().clone(), proof_nodes: core.proof_nodes().clone(), algorithms: core.algorithms().clone(), algorithm_statements: core.algorithm_statements().clone(), generated: core.generated().clone(), obligation_seeds: core.obligation_seeds().clone(), source_map: core.source_map().clone(), diagnostics: core.diagnostics().clone() };
        match mutation {
            0 => p.algorithms.get_mut(algorithm_id).unwrap().statements.remove(0),
            1 => { p.algorithms.get_mut(algorithm_id).unwrap().statements.insert(0, pick); pick },
            2 => { p.algorithm_statements.get_mut(pick).unwrap().owner = CoreAlgorithmId::new(99); pick },
            3 => { let CoreAlgorithmStmtKind::Pick { ghost, .. } = &mut p.algorithm_statements.get_mut(pick).unwrap().kind else { panic!() }; *ghost = true; pick },
            4 => { let CoreAlgorithmStmtKind::Pick { witness_ty, .. } = &mut p.algorithm_statements.get_mut(pick).unwrap().kind else { panic!() }; *witness_ty = algorithm.params[0].ty_guard; pick },
            5 => { p.terms.get_mut(value).unwrap().kind = CoreTermKind::Var(algorithm.params[0].var); pick },
            6 => { let CoreFormulaKind::Exists { binders, .. } = &mut p.formulas.get_mut(exists).unwrap().kind else { panic!() }; binders[0].var = binder.var; pick },
            7 => { p.formulas.get_mut(body).unwrap().kind = CoreFormulaKind::True; pick },
            8 => { let CoreFormulaKind::TypePred { ty, .. } = &mut p.formulas.get_mut(binder.ty_guard.unwrap()).unwrap().kind else { panic!() }; *ty = CoreTypePredicate::new("object"); pick },
            9 => { p.obligation_seeds = ObligationSeedTable::new(); p.source_map.obligation_sources.clear(); pick },
            10 => { let id = p.obligation_seeds.insert(seed.clone()); p.source_map.obligation_sources.insert(id, seed.source.clone()); pick },
            11 => { p.obligation_seeds.get_mut(seed_id).unwrap().core_refs.clear(); pick },
            12 => { p.obligation_seeds.get_mut(seed_id).unwrap().context.push(algorithm.params[0].ty_guard.unwrap()); pick },
            13 => { p.obligation_seeds.get_mut(seed_id).unwrap().status = ObligationSeedStatus::Deferred; pick },
            14 => { p.obligation_seeds.get_mut(seed_id).unwrap().source = algorithm.source.clone(); pick },
            15 => { let row = p.algorithm_statements.get_mut(pick).unwrap(); row.source = core.algorithm_statements().get(returned).unwrap().source.clone(); p.source_map.algorithm_sources.insert(pick, row.source.clone()); pick },
            16 => { let CoreAlgorithmStmtKind::Pick { binder, .. } = &mut p.algorithm_statements.get_mut(pick).unwrap().kind else { panic!() }; binder.var = algorithm.params[0].var; pick },
            17 => { p.source_map.formula_sources.remove(&exists); pick },
            18 => {
                let source = core.terms().get(value).unwrap().source.clone();
                let origin = p.generated.insert(GeneratedOrigin { owner: algorithm.item, kind: GeneratedOriginKind::StableChoice, key: GeneratedOriginKey::new("choice:builtin.set"), functor: Some(algorithm.symbol.clone()), params: vec![], evidence: source.provenance.clone(), source: source.clone() });
                p.source_map.generated_sources.insert(origin, source);
                p.terms.get_mut(value).unwrap().kind = CoreTermKind::Generated { origin, args: vec![] };
                pick
            },
            _ => unreachable!(),
        };
        match CoreIr::try_new(p) {
            Err(_) => constructor_rejections += 1,
            Ok(changed) => {
                assert!(step5c14_return_vcs(&changed).is_err(), "core mutation {mutation}");
                assert!(failed_source_algorithm_assertion(&changed, &vcs).is_err());
                generator_rejections += 1;
            }
        }
    }
    assert!(constructor_rejections > 0 && generator_rejections > 0);
    let post = vcs.vcs().iter().position(|vc| vc.kind == VcKind::AlgorithmPostcondition).unwrap();
    for mutation in 0..17 {
        let mut p = VcSetParts { schema_version: vcs.schema_version().clone(), snapshot: vcs.snapshot(), source: vcs.source(), module: vcs.module().clone(), generated_formulas: vcs.generated_formulas().to_vec(), vcs: vcs.vcs().to_vec(), seed_accounting: vcs.seed_accounting().to_vec() };
        match mutation {
            0 => p.vcs[post].status = VcStatus::NeedsAtp,
            1 => p.vcs[post].premises.clear(),
            2 => p.vcs[post].source.related.clear(),
            3 => p.vcs[post].source.primary = algorithm.source.clone(),
            4 => p.vcs[post].anchor.owner = AnchorOwner::Algorithm(CoreAlgorithmId::new(99)),
            5 => p.vcs[post].goal = VcFormulaRef::Core(algorithm.params[0].ty_guard.unwrap()),
            6 => p.generated_formulas[0].provenance.clear(),
            7 => { let shape @ VcGeneratedFormulaShape::ProgramTypePredicate { .. } = &mut p.generated_formulas[0].shape else { panic!() }; *shape = VcGeneratedFormulaShape::ProgramTypePredicate { subject: VcProgramValue { var: binder.var, definition: None }, ty: CoreTypePredicate::new("set") }; },
            8..=10 => {
                let mut entries = p.vcs[post].local_context.entries().to_vec();
                if mutation == 8 { entries.pop(); } else if mutation == 9 { entries[1].formula = entries[0].formula; } else { let mut extra = entries[1].clone(); extra.id = ContextEntryId::new(2); extra.sort_key = "algorithm-state-00000002".into(); entries.push(extra); }
                p.vcs[post].local_context = LocalContext::try_new(entries, vec![]).unwrap();
                p.vcs[post].premises = p.vcs[post].local_context.entries().iter().map(|e| PremiseRef::LocalContext(e.id)).collect();
            }
            11 => p.seed_accounting[0].seed_status = if p.seed_accounting[0].seed_status == ObligationSeedStatus::Deferred { ObligationSeedStatus::Active } else { ObligationSeedStatus::Deferred },
            12 => p.vcs[post].proof_hint = Some(ProofHint { citations: vec![], unfold_requests: vec![], premise_restrictions: vec![], solver: None, max_axioms: None, timeout: None, computation: None, provenance: vec![] }),
            13 => { let nonempty = 1 - post; p.vcs[nonempty].status = VcStatus::NeedsAtp; },
            14 => {
                let removed = p.vcs.pop().unwrap();
                p.seed_accounting.iter_mut().find(|row| row.handoff == removed.seed.handoff).unwrap().mapping = SeedVcMapping::NoConcreteVc { reason: SeedNoVcReason::DeferredExternal("removed".into()) };
            },
            15 => {
                let mut extra = p.vcs.last().unwrap().clone();
                let original = extra.id;
                extra.id = VcId::new(p.vcs.len());
                p.seed_accounting.iter_mut().find(|row| row.handoff == extra.seed.handoff).unwrap().mapping = SeedVcMapping::Expanded { vcs: vec![ExpandedVcRef { expansion_index: 0, vc: original }, ExpandedVcRef { expansion_index: 1, vc: extra.id }], expansion_schema: ExpansionSchemaVersion::new("test-extra-vc") };
                p.vcs.push(extra);
            },
            16 => p.seed_accounting[0].origin = SeedOriginRef::ExistingCore { seed: ObligationSeedId::new(99) },
            _ => unreachable!(),
        }
        let changed = VcSet::try_new(p).unwrap_or_else(|e| panic!("vc mutation {mutation}: {e}"));
        assert!(failed_source_algorithm_assertion(&core, &changed).is_err(), "vc mutation {mutation}");
    }
}
