use super::*;

#[test]
fn step5c14_computation_rejects_stale_and_mutated_receipts() -> Result<(), String> {
    use mizar_checker::{type_checker::SourceVariableSemanticsChecker, typed_ast::*};
    use mizar_resolve::{
        labels::{LabelResolver, ProofLabelSourceCollector},
        names::{SourceVariableScopeInput, SourceVariableScopeResolver},
        resolved_ast::SurfaceResolvedArena,
    };
    let text = "theorem T: 0 = 0 by computation(steps: 8);";
    let config = config();
    let plan = crate::harness::build_test_plan(&config).unwrap();
    let case = plan
        .cases
        .iter()
        .find(|case| case.id.0 == super::super::super::formula_statement::STEP5C10_CASES[0].0)
        .unwrap();
    let ast = parse(text);
    let resolver = resolver_symbol_collection(&config.workspace_root, case, &ast);
    if !resolver.detail_keys.is_empty() {
        return Err(format!("{:?}", resolver.detail_keys));
    }
    let source = SurfaceResolvedArena::lower(&ast, &resolver.module).map_err(|e| e.to_string())?;
    let scope = SourceVariableScopeResolver::resolve_proof_occurrences(
        SourceVariableScopeInput::new(&ast, &resolver.module, &resolver.env),
    )
    .map_err(|e| format!("{e:?}"))?;
    let bindings = SourceVariableSemanticsChecker::occurrence_binding_env(&scope);
    let typed = super::super::super::type_elaboration::step5c8_formula_typed_ast(
        &ast,
        &resolver.module,
        &resolver.env,
        &scope,
        &bindings,
        true,
    )?;
    let owner = resolver
        .env
        .symbols()
        .iter()
        .find(|entry| entry.kind() == mizar_resolve::env::SymbolKind::Theorem)
        .ok_or("no theorem")?;
    let namespace = mizar_resolve::env::NamespacePath::new(resolver.module.path().as_str());
    let labels = ProofLabelSourceCollector::new(
        &ast,
        &resolver.module,
        namespace.clone(),
        owner.contribution(),
        &source,
    )
    .and_then(|collector| collector.collect_with_theorem_owners(&resolver.env))
    .map_err(|e| e.to_string())?;
    let resolved = LabelResolver::new(labels.projections()).resolve(
        &resolver.module,
        &namespace,
        labels.references(),
    );

    for changed in [
        text.replace("steps: 8", "steps: 9"),
        text.replace("T:", "U:"),
        text.replace("0 = 0", "0 = 1"),
    ] {
        let changed_ast = parse(&changed);
        let changed_source = SurfaceResolvedArena::lower(&changed_ast, &resolver.module).unwrap();
        assert!(
            SourceVariableSemanticsChecker::check_theorem_skeletons(
                &changed_source,
                &typed,
                &scope,
                &resolver.env,
                &labels,
                &resolved,
                None
            )
            .is_err()
        );
    }
    let option = typed
        .nodes()
        .iter()
        .find_map(|(id, node)| (node.kind.as_str() == "ComputationOption").then_some(id))
        .unwrap();
    for mutation in 0..5 {
        let mut nodes = typed
            .nodes()
            .iter()
            .map(|(_, node)| node.clone())
            .collect::<Vec<_>>();
        match mutation {
            0 => nodes[option.index()].children.clear(),
            1 => nodes[option.index()].recovery = NodeRecoveryState::Recovered,
            2 => nodes[option.index()].kind = "TermReference".into(),
            3 => {
                nodes[option.index()].anchor =
                    mizar_session::SourceAnchor::Range(mizar_session::SourceRange {
                        source_id: typed.source_id(),
                        start: 0,
                        end: 1,
                    })
            }
            4 => nodes[option.index()].resolved_node = Some(source.arena().root()),
            _ => unreachable!(),
        }
        let nodes = TypedArena::try_new(typed.nodes().root(), nodes).unwrap();
        let changed = {
            TypedAst::try_new(TypedAstParts {
                source_id: typed.source_id(),
                module_id: typed.module_id().clone(),
                resolved_root: None,
                source_context: None,
                source_type: None,
                source_attribute: None,
                nodes,
                contexts: LocalTypeContextTable::new(),
                types: TypeTable::new(),
                facts: TypeFactTable::new(),
                coercions: CoercionTable::new(),
                initial_obligations: InitialObligationTable::new(),
                diagnostics: TypeDiagnosticTable::new(),
            })
        }
        .and_then(|typed_ast| typed_ast.with_source_term(typed.source_term().unwrap().clone()))
        .and_then(|typed_ast| {
            typed_ast.with_source_atomic_formula(typed.source_atomic_formula().unwrap().clone())
        })
        .unwrap_or_else(|error| {
            panic!("mutation {mutation} must construct a typed receipt: {error}")
        });
        {
            assert!(
                SourceVariableSemanticsChecker::check_theorem_skeletons(
                    &source,
                    &changed,
                    &scope,
                    &resolver.env,
                    &labels,
                    &resolved,
                    None
                )
                .is_err(),
                "mutation {mutation}"
            );
        }
    }
    Ok(())
}

#[test]
fn step5c14_computation_preserves_source_request_without_execution() {
    let config = config();
    let plan = crate::harness::build_test_plan(&config).unwrap();
    let case = plan
        .cases
        .iter()
        .find(|case| case.id.0 == "pass_proof_verification_computation_justification_001")
        .unwrap();
    let frontend = run_frontend(&config.workspace_root, case, 0).unwrap();
    let ast = frontend.ast.clone().unwrap();
    let (source, _, symbols) =
        super::super::super::source_registration_inputs(&config.workspace_root, case, frontend)
            .unwrap();
    let vcs = theorem_ast_output(
        &ast,
        source.module(),
        &symbols,
        PipelinePhase::VcGeneration,
        snapshot_id(0),
        None,
    )
    .unwrap()
    .1
    .unwrap();
    assert_eq!(
        fs::read_to_string(config.workspace_root.join(
            "tests/snapshots/vc/pass_proof_verification_computation_justification_001.vc_ir.snap"
        ))
        .unwrap(),
        vcs.debug_text()
    );
    assert_eq!(
        vcs,
        theorem_ast_output(
            &ast,
            source.module(),
            &symbols,
            PipelinePhase::VcGeneration,
            snapshot_id(0),
            None
        )
        .unwrap()
        .1
        .unwrap()
    );
    use mizar_core::{binder_normalization::*, control_flow::*, core_ir::*};
    let original = "theorem Comp1: 0 = 0 by computation(steps: 8);";
    let baseline = step5c14_computation_core(original).unwrap();
    for digits in ["8", "9", "0", "0008", "18446744073709551616000000000000000"] {
        let source = original.replace("steps: 8", &format!("steps: {digits}"));
        let core = step5c14_computation_core(&source).unwrap();
        assert_eq!(core, step5c14_computation_core(&source).unwrap());
        assert_eq!(
            core.debug_text(),
            step5c14_computation_core(&source).unwrap().debug_text()
        );
        if digits != "8" {
            assert_ne!(core, baseline);
        }
        let (_, proof) = core.proofs().iter().next().unwrap();
        assert_eq!(proof.status, CoreProofStatus::PendingAutomaticProof);
        let node = core.proof_nodes().get(proof.root).unwrap();
        let CoreProofNodeKind::ComputationGoal { obligation, steps } = &node.kind else {
            panic!("request missing");
        };
        assert_eq!(steps, digits);
        let seed = core.obligation_seeds().get(*obligation).unwrap();
        assert_eq!(seed.status, ObligationSeedStatus::Active);
        assert!(seed.context.is_empty());
        let CoreSourceAnchor::SourceRange(request) = node.source.anchor else {
            panic!();
        };
        assert_eq!(
            &source[request.start..request.end],
            format!("by computation(steps: {digits})")
        );
        let formula = core.formulas().get(proof.proposition).unwrap();
        let CoreFormulaKind::Equals { left, right } = formula.kind else {
            panic!();
        };
        assert_ne!(left, right);
        for term in [left, right] {
            assert_eq!(
                core.terms().get(term).unwrap().kind,
                CoreTermKind::Numeral("0".into())
            );
            assert!(
                normalize_core_term(&core, term, &BinderContext::new(), &formula.source).is_err()
            );
        }
        assert_ne!(
            core.terms().get(left).unwrap().source,
            core.terms().get(right).unwrap().source
        );
        assert!(
            normalize_core_formula(
                &core,
                proof.proposition,
                &BinderContext::new(),
                &formula.source
            )
            .is_err()
        );
        let flow = build_control_flow_ir(&core);
        let handoff = build_obligation_seed_handoff(&core, &flow);
        assert_eq!(
            handoff.entries.iter().next().unwrap().1.seed.status,
            ObligationSeedStatus::Deferred
        );
        assert!(
            generate_core_vcs(&core, snapshot_id(777))
                .unwrap()
                .vcs()
                .is_empty()
        );
        let vcs = check_theorem_source(&source, PipelinePhase::VcGeneration)
            .unwrap()
            .1
            .unwrap();
        assert_eq!(
            vcs,
            check_theorem_source(&source, PipelinePhase::VcGeneration)
                .unwrap()
                .1
                .unwrap()
        );
        let [vc] = vcs.vcs() else {
            panic!("not one VC");
        };
        assert_eq!(vc.status, mizar_vc::vc_ir::VcStatus::Open);
        assert_eq!(
            vc.goal,
            mizar_vc::vc_ir::VcFormulaRef::Core(proof.proposition)
        );
        assert_eq!(
            vc.proof_hint.as_ref().unwrap().computation,
            Some(mizar_vc::vc_ir::ComputationHint::SymbolicRequest(
                mizar_vc::vc_ir::ProofHintKey::new(format!("by-computation(steps:{digits})"))
            ))
        );
        assert_eq!(vc.source.primary, node.source);
        assert_eq!(vc.source.related.len(), 4);
        assert!(vcs.canonical_vc_fingerprint(vc.id).is_none());
        let slices = mizar_vc::dependency_slice::try_compute_dependency_slices(
            mizar_vc::dependency_slice::DependencySliceInput {
                vc_set: &vcs,
                discharge_output: None,
            },
        )
        .unwrap();
        let slice = slices.slice_for(vc.id).unwrap();
        assert!(slice.requires_cache_miss());
        assert!(slice.unknowns().iter().any(|unknown| unknown.family()
            == mizar_vc::dependency_slice::DependencyUnknownFamily::Computation));
        assert!(format!("{vcs:?}").contains(&format!("by-computation(steps:{digits})")));
    }
    let renamed = step5c14_computation_core(&original.replace("Comp1", "Other")).unwrap();
    assert_ne!(baseline.items(), renamed.items());
    assert_ne!(baseline.debug_text(), renamed.debug_text());
}

#[test]
fn step5c14_computation_core_mutations_fail_closed() {
    use mizar_core::core_ir::*;
    let core = step5c14_computation_core("theorem T: 0 = 0 by computation(steps: 8);").unwrap();
    let (proof_id, proof) = core.proofs().iter().next().unwrap();
    let formula = proof.proposition;
    let node = proof.root;
    let CoreProofNodeKind::ComputationGoal { obligation, .. } =
        core.proof_nodes().get(node).unwrap().kind
    else {
        panic!();
    };
    let CoreFormulaKind::Equals { left, right } = core.formulas().get(formula).unwrap().kind else {
        panic!();
    };
    let generate = |core: &CoreIr| {
        mizar_vc::generator::generate_source_computation_request(
            core,
            snapshot_id(777),
            &GenerationSchemaVersion::new("test"),
            &VcSchemaVersion::new("test"),
        )
    };
    for mutation in 0..20 {
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
                parts.proof_nodes.get_mut(node).unwrap().kind = CoreProofNodeKind::TerminalGoal {
                    obligation,
                    citations: Vec::new(),
                }
            }
            1 => {
                parts.formulas.get_mut(formula).unwrap().kind =
                    CoreFormulaKind::Equals { left, right: left }
            }
            2 => parts.terms.get_mut(right).unwrap().kind = CoreTermKind::Numeral("1".into()),
            3 => parts
                .obligation_seeds
                .get_mut(obligation)
                .unwrap()
                .context
                .push(formula),
            4 => {
                parts.obligation_seeds.get_mut(obligation).unwrap().label =
                    Some(CoreLabelRef::new("fake"))
            }
            5 => parts
                .items
                .get_mut(proof.item)
                .unwrap()
                .dependencies
                .push(proof.item),
            6 => parts.proofs.get_mut(proof_id).unwrap().status = CoreProofStatus::Open,
            7 => parts.items.get_mut(proof.item).unwrap().kind = CoreItemKind::Lemma,
            8 => parts
                .obligation_seeds
                .get_mut(obligation)
                .unwrap()
                .core_refs
                .push(CoreNodeRef::Term(left)),
            9 => parts
                .obligation_seeds
                .get_mut(obligation)
                .unwrap()
                .core_refs
                .retain(|reference| *reference != CoreNodeRef::Proof(proof_id)),
            10 => {
                parts
                    .obligation_seeds
                    .get_mut(obligation)
                    .unwrap()
                    .semantic_origin = NormalizedSemanticOrigin::new("fake")
            }
            11 => {
                parts
                    .obligation_seeds
                    .get_mut(obligation)
                    .unwrap()
                    .local_path = LocalProofOrProgramPath::new("fake")
            }
            12 => {
                parts.terms.get_mut(left).unwrap().source =
                    parts.terms.get(right).unwrap().source.clone();
                parts
                    .source_map
                    .term_sources
                    .insert(left, parts.terms.get(left).unwrap().source.clone());
            }
            13 => {
                parts
                    .formulas
                    .get_mut(formula)
                    .unwrap()
                    .source
                    .provenance
                    .clear();
                parts
                    .source_map
                    .formula_sources
                    .insert(formula, parts.formulas.get(formula).unwrap().source.clone());
            }
            14 => {
                let term = parts.terms.get(left).unwrap().clone();
                let id = parts.terms.insert(term.clone());
                parts.source_map.term_sources.insert(id, term.source);
            }
            15 => parts
                .obligation_seeds
                .get_mut(obligation)
                .unwrap()
                .provenance
                .clear(),
            16 => parts
                .proofs
                .get_mut(proof_id)
                .unwrap()
                .source
                .provenance
                .clear(),
            17 => {
                parts.proof_nodes.get_mut(node).unwrap().kind = CoreProofNodeKind::Sequence {
                    children: Vec::new(),
                }
            }
            18 => parts.obligation_seeds.get_mut(obligation).unwrap().goal = None,
            19 => {
                parts.proof_nodes.get_mut(node).unwrap().kind = CoreProofNodeKind::ComputationGoal {
                    obligation,
                    steps: "9".into(),
                }
            }
            _ => unreachable!(),
        }
        match CoreIr::try_new(parts) {
            Err(_) => assert_ne!(mutation, 19),
            Ok(mutated) if mutation == 19 => {
                let vcs = generate(&mutated).unwrap();
                assert_ne!(vcs, generate(&core).unwrap());
                assert!(vcs.debug_text().contains("by-computation(steps:9)"));
            }
            Ok(mutated) => assert!(generate(&mutated).is_err(), "accepted mutation {mutation}"),
        }
    }
}

#[test]
fn step5c14_explicit_computation_calls_preserve_terms_and_fail_in_scope() {
    use mizar_resolve::{
        names::{SourceVariableScopeError, SourceVariableScopeInput, SourceVariableScopeResolver},
        resolved_ast::SurfaceResolvedArena,
    };
    use mizar_syntax::SurfaceNodeKind as K;

    let config = config();
    let plan = crate::harness::build_test_plan(&config).unwrap();
    let case = plan
        .cases
        .iter()
        .find(|case| case.id.0 == super::super::super::formula_statement::STEP5C10_CASES[0].0)
        .unwrap();
    let core = step5c14_computation_core("theorem T: 0 = 0 by computation(steps: 8);").unwrap();
    let (_, proof) = core.proofs().iter().next().unwrap();
    let node = core.proof_nodes().get(proof.root).unwrap();
    assert!(
        matches!(&node.kind, mizar_core::core_ir::CoreProofNodeKind::ComputationGoal { steps, .. } if steps == "8")
    );

    for (arguments, direct, call, callee, term_args, qualified, templated) in [
        (
            "calc(0)",
            vec!["computation", "(", "calc(0)", ")"],
            "calc(0)",
            "calc",
            vec!["0"],
            false,
            false,
        ),
        (
            "calc()",
            vec!["computation", "(", "calc()", ")"],
            "calc()",
            "calc",
            vec![],
            false,
            false,
        ),
        (
            "steps: 8, calc(0)",
            vec!["computation", "(", "steps: 8", ",", "calc(0)", ")"],
            "calc(0)",
            "calc",
            vec!["0"],
            false,
            false,
        ),
        (
            "calc(0), steps: 8",
            vec!["computation", "(", "calc(0)", ",", "steps: 8", ")"],
            "calc(0)",
            "calc",
            vec!["0"],
            false,
            false,
        ),
        (
            "pkg.calc(0)",
            vec!["computation", "(", "pkg.calc(0)", ")"],
            "pkg.calc(0)",
            "pkg.calc",
            vec!["0"],
            true,
            false,
        ),
        (
            "calc[set](0)",
            vec!["computation", "(", "calc[set](0)", ")"],
            "calc[set](0)",
            "calc[set]",
            vec!["0"],
            false,
            true,
        ),
    ] {
        let text = format!("theorem T: 0 = 0 by computation({arguments});");
        let frontend = super::super::super::formula_statement::step5c8_test_frontend(&text);
        assert!(
            frontend.diagnostics.is_empty(),
            "{arguments}: {:?}",
            frontend.diagnostics
        );
        let ast = frontend.ast.as_ref().unwrap();
        assert!(
            ast.node_views()
                .all(|node| !node.is_recovered() && node.as_recovery().is_none()),
            "{arguments}"
        );
        let requests = ast
            .node_views()
            .filter(|node| matches!(node.kind(), K::ComputationJustification))
            .collect::<Vec<_>>();
        assert_eq!(requests.len(), 1, "{arguments}");
        let request = requests[0];
        let children = request.child_views().collect::<Vec<_>>();
        assert_eq!(
            children
                .iter()
                .map(|node| &text[node.range().start..node.range().end])
                .collect::<Vec<_>>(),
            direct,
            "{arguments}"
        );
        assert_eq!(
            children
                .iter()
                .filter(|node| matches!(node.kind(), K::ApplicationTerm))
                .count(),
            1,
            "{arguments}"
        );
        let application = children
            .iter()
            .find(|node| matches!(node.kind(), K::ApplicationTerm))
            .unwrap();
        assert_eq!(
            &text[application.range().start..application.range().end],
            call
        );
        let call_children = application.child_views().collect::<Vec<_>>();
        assert!(matches!(call_children[0].kind(), K::TermReference));
        let reference = call_children[0];
        assert_eq!(
            &text[reference.range().start..reference.range().end],
            callee
        );
        assert_eq!(
            reference
                .child_views()
                .any(|node| matches!(node.kind(), K::NamespacePath)),
            qualified
        );
        if qualified {
            let parts = reference.child_views().collect::<Vec<_>>();
            assert!(matches!(parts[0].kind(), K::NamespacePath));
            assert_eq!(parts[1].as_token().unwrap().text.as_ref(), ".");
            assert_eq!(parts[2].as_token().unwrap().text.as_ref(), "calc");
        }
        assert_eq!(
            reference
                .child_views()
                .any(|node| matches!(node.kind(), K::TemplateArguments)),
            templated
        );
        assert_eq!(call_children[1].as_token().unwrap().text.as_ref(), "(");
        assert_eq!(
            call_children
                .last()
                .unwrap()
                .as_token()
                .unwrap()
                .text
                .as_ref(),
            ")"
        );
        assert_eq!(
            call_children[2..call_children.len() - 1]
                .iter()
                .map(|node| {
                    assert!(matches!(node.kind(), K::TermExpression));
                    &text[node.range().start..node.range().end]
                })
                .collect::<Vec<_>>(),
            term_args,
            "{arguments}"
        );

        let resolver = resolver_symbol_collection(&config.workspace_root, case, ast);
        assert!(
            resolver.detail_keys.is_empty(),
            "{arguments}: {:?}",
            resolver.detail_keys
        );
        let source = SurfaceResolvedArena::lower(ast, &resolver.module).unwrap();
        let scope_error = SourceVariableScopeResolver::resolve_proof_occurrences(
            SourceVariableScopeInput::new(ast, source.module(), &resolver.env),
        )
        .unwrap_err();
        assert_eq!(
            scope_error,
            SourceVariableScopeError::InvalidShape,
            "{arguments}"
        );
        assert_eq!(
            step5c14_computation_core(&text).unwrap_err(),
            "InvalidShape",
            "{arguments}"
        );
    }
}

#[test]
fn step5c14_replay_options_parse_cleanly_but_fail_source_check() {
    use mizar_syntax::SurfaceNodeKind as K;
    for (options, expected, accepted) in [
        ("steps: 8", vec![vec!["steps", ":", "8"]], true),
        ("replay: true", vec![vec!["replay", ":", "true"]], false),
        ("replay: false", vec![vec!["replay", ":", "false"]], false),
        (
            "steps: 8, replay: true",
            vec![vec!["steps", ":", "8"], vec!["replay", ":", "true"]],
            false,
        ),
        (
            "replay: false, steps: 8",
            vec![vec!["replay", ":", "false"], vec!["steps", ":", "8"]],
            false,
        ),
    ] {
        let text = format!("theorem T: 0 = 0 by computation({options});");
        let frontend = super::super::super::formula_statement::step5c8_test_frontend(&text);
        assert!(
            frontend.diagnostics.is_empty(),
            "{options}: {:?}",
            frontend.diagnostics
        );
        let ast = frontend.ast.as_ref().unwrap();
        assert!(
            ast.node_views()
                .all(|node| !node.is_recovered() && node.as_recovery().is_none())
        );
        let actual = ast
            .node_views()
            .filter(|node| matches!(node.kind(), K::ComputationOption))
            .map(|option| {
                option
                    .child_views()
                    .map(|child| child.as_token().unwrap().text.as_ref())
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        assert_eq!(actual, expected, "{options}");
        let core = step5c14_computation_core(&text);
        if accepted {
            assert!(core.is_ok());
        } else {
            assert_eq!(
                core.err().as_deref(),
                Some("theorems.source.invalid"),
                "{options}"
            );
        }
    }
}

#[test]
fn step5c14_computation_rejects_unsupported_source_profiles() {
    for source in [
        "theorem T: 1 = 1 by computation(steps: 8);",
        "theorem T: 0 = 1 by computation(steps: 8);",
        "theorem T: 0 = 0 by computation;",
        "theorem T: 0 = 0 by computation();",
        "theorem T: 0 = 0 by computation(timeout: 8);",
        "theorem T: 0 = 0 by computation(steps: 8, steps: 9);",
        "theorem T: 0 = 0 by computation(steps: 8, nest: 1);",
        "theorem T: 0 = 0 by computation(steps 8);",
        "theorem T: 0 = 0 by (steps: 8);",
        "theorem T: 0 = 0 proof thus 0 = 0 by computation(steps: 8); end;",
        "open theorem T: 0 = 0 by computation(steps: 8);",
        "theorem T: 0 = 0 by computation(steps: 8); theorem U: 0 = 0 by computation(steps: 8);",
    ] {
        assert!(
            step5c14_computation_core(source).is_err(),
            "accepted {source}"
        );
    }
}
