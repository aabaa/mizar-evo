use super::*;

#[test]
fn step5c7_proof_corpus() {
    let config = config();
    let root = &config.workspace_root;
    let plan = crate::harness::build_test_plan(&config).unwrap();
    let cases = plan
        .cases
        .iter()
        .filter(|case| step5c7_proof_candidate(case))
        .collect::<Vec<_>>();
    assert_eq!(cases.len(), 2);
    for case in cases {
        let result = run_proof_verification_case(root, &root.join("tests"), case, 1);
        assert_eq!(
            result.status,
            ProofVerificationCaseStatus::Passed,
            "{:?}",
            result.failure
        );
    }
}

#[test]
fn step5c7_membership_proof_mutations_are_not_credit() {
    let config = config();
    let root = &config.workspace_root;
    let plan = crate::harness::build_test_plan(&config).unwrap();
    for (id, path) in STEP5C7_PROOF_CASES {
        let case = plan.cases.iter().find(|case| case.id.0 == id).unwrap();
        let source = fs::read_to_string(root.join(path)).unwrap();
        let check = |text: &str| {
            let ast = parse(text);
            let symbols = resolver_symbol_collection(root, case, &ast);
            normalize_term_ast(&ast, &symbols.module, &symbols.env)
        };
        // Renaming is semantically irrelevant, including proof-local alpha names.
        let renamed = source
            .replace("A1", "Premise")
            .replace("CompMember1", "Guarded")
            .replace("EnumMember1", "Listed");
        assert_eq!(check(&renamed), Ok(2));
        if id.contains("comprehension") {
            for (label, changed) in [
                ("missing citation", source.replace(" by A1", "")),
                ("wrong citation", source.replace("by A1", "by Missing")),
                ("missing guard", source.replace(" : z in A", "")),
                (
                    "non-set bound",
                    source.replace("reserve A for set", "reserve A for object"),
                ),
                (
                    "shadowed set reservation",
                    source.replace(
                        "reserve A for set;",
                        "reserve A for set;\nreserve A for object;",
                    ),
                ),
                ("dependent bound", source.replace("z in A", "z in z")),
                ("wrong mapper", source.replace("{ z where", "{ x where")),
                (
                    "wrong proof premise",
                    source.replace("A1: x in A", "A1: A in A"),
                ),
                (
                    "altered conclusion",
                    source.replace("thus x in", "thus A in"),
                ),
                ("unjustified premise", source.replace(" st x in A", "")),
            ] {
                assert!(check(&changed).is_err(), "{label} was accepted");
            }
            let alpha = source
                .replace("let x be", "let u be")
                .replace("A1: x in", "A1: u in")
                .replace("thus x in", "thus u in");
            assert_eq!(check(&alpha), Ok(2));
        } else {
            for changed in [
                source.replace("thus x in", "thus y in"),
                source.replace("{x, y}", "{y}"),
                source.replace("thus x in {x, y}", "thus x in {x}"),
                source.replace("let x, y be object", "let x, y be set"),
            ] {
                assert!(
                    check(&changed).is_err(),
                    "altered enumeration proof accepted: {changed}"
                );
            }
        }
    }
}

#[test]
fn step5c7_citation_results_cannot_be_substituted() {
    use mizar_checker::{
        source_set_term::SourceSetTermProducer, type_checker::SourceVariableSemanticsChecker,
    };
    use mizar_resolve::{
        labels::{
            LabelReferenceCandidate, LabelReferenceScope, LabelResolver, ProofLabelSourceCollector,
        },
        names::{SourceVariableScopeInput, SourceVariableScopeResolver},
        resolved_ast::{ReferenceSite, SurfaceResolvedArena},
    };
    let config = config();
    let root = &config.workspace_root;
    let plan = crate::harness::build_test_plan(&config).unwrap();
    let case = plan
        .cases
        .iter()
        .find(|case| case.id.0 == STEP5C7_PROOF_CASES[0].0)
        .unwrap();
    let source = fs::read_to_string(root.join(STEP5C7_PROOF_CASES[0].1)).unwrap();
    for citation in ["A1", "Missing"] {
        let ast = parse(&source.replace("by A1", &format!("by {citation}")));
        let symbols = resolver_symbol_collection(root, case, &ast);
        let module = &symbols.module;
        let scope = SourceVariableScopeResolver::resolve_occurrences(
            SourceVariableScopeInput::new(&ast, module, &symbols.env),
        )
        .unwrap();
        let bindings = SourceVariableSemanticsChecker::occurrence_binding_env(&scope);
        let typed = super::super::super::type_elaboration::step5c7_membership_typed_ast(
            &ast,
            module,
            &symbols.env,
            &scope,
            &bindings,
        )
        .unwrap();
        let arena = SurfaceResolvedArena::lower(&ast, module).unwrap();
        let owner = symbols
            .env
            .symbols()
            .iter()
            .find(|entry| entry.kind() == mizar_resolve::env::SymbolKind::Theorem)
            .unwrap();
        let namespace = mizar_resolve::env::NamespacePath::new(module.path().as_str());
        let labels = ProofLabelSourceCollector::new(
            &ast,
            module,
            namespace.clone(),
            owner.contribution(),
            &arena,
        )
        .unwrap()
        .collect_with_let_conditions()
        .unwrap();
        let resolve = |references: &[LabelReferenceCandidate]| {
            LabelResolver::new(labels.projections()).resolve(module, &namespace, references)
        };
        let check = |resolved| {
            SourceSetTermProducer::check_membership_proof(
                &typed,
                &scope,
                &bindings,
                &labels,
                resolved,
                &symbols.env,
            )
            .is_ok()
        };
        let original = resolve(labels.references());
        assert_eq!(check(&original), citation == "A1");
        let reference = &labels.references()[0];
        let LabelReferenceScope::Unqualified { proof_scope } = reference.scope() else {
            panic!("local citation")
        };
        let forged = LabelReferenceCandidate::unqualified_citation(
            ReferenceSite::new(reference.site().node(), reference.site().range(), "A1"),
            if citation == "A1" {
                labels.projections()[0].origin().clone()
            } else {
                reference.origin().clone()
            },
            reference.ordinal(),
            proof_scope.clone(),
        );
        let substituted = resolve(&[forged]);
        assert!(!substituted.has_unresolved());
        assert!(
            !check(&substituted),
            "substituted site/origin for {citation}"
        );
    }
}

#[test]
fn step5c7_native_condition_composition_replays_source_membership() {
    use mizar_checker::{
        source_application::{
            SourceFunctorApplicationHandoffInput, SourceFunctorApplicationProducer,
        },
        source_atomic_formula::{SourceAtomicFormulaKind, SourceAtomicRequestKind},
        source_formula_composition::{
            SourceConditionFormulaCompositionHandoffInput,
            SourceConditionFormulaCompositionProducer, SourceConditionFormulaEdgeInput,
        },
        source_set_term::SourceSetConditionId,
        type_checker::SourceVariableSemanticsChecker,
    };
    use mizar_resolve::names::{SourceVariableScopeInput, SourceVariableScopeResolver};
    use mizar_syntax::SurfaceNodeKind;

    let config = config();
    let root = &config.workspace_root;
    let plan = crate::harness::build_test_plan(&config).unwrap();
    let case = plan
        .cases
        .iter()
        .find(|case| case.id.0 == STEP5C7_PROOF_CASES[0].0)
        .unwrap();
    let original = fs::read_to_string(root.join(STEP5C7_PROOF_CASES[0].1)).unwrap();
    let one = original.replacen(" : z in A", "", 1);
    let mut header_only = original.clone();
    let last_condition = header_only.rfind(" : z in A").unwrap();
    header_only.replace_range(last_condition..last_condition + " : z in A".len(), "");
    let zero = original.replace(" : z in A", "");
    let three = format!(
        "{original}\ntheorem CompMemberExtra: for A for x being object st x in A holds x in {{ z where z is object : z in A }};\n"
    );
    let alpha = original.replace("z", "q");
    let layout = original.replace(": z in A", ":  z in A");
    let set_heads = original.replace("z is object", "z is set");
    let equality = original.replace(": z in A", ": z = z");
    let header_equality = original.replacen(": z in A", ": z = z", 1);
    let mut proof_equality = original.clone();
    let proof_condition = proof_equality.rfind(": z in A").unwrap();
    proof_equality.replace_range(
        proof_condition..proof_condition + ": z in A".len(),
        ": z = z",
    );
    let noncondition_equality = original.replacen("st x in A holds", "st x = x holds", 1);
    let equality_layout = original.replace(": z in A", ":  z = z");
    let cases = [
        ("one", one, 1, &["z in A"][..]),
        ("header only", header_only, 1, &["z in A"][..]),
        ("original", original, 2, &["z in A"][..]),
        ("three", three, 3, &["z in A"][..]),
        ("alpha", alpha, 2, &["q in A"][..]),
        ("layout", layout, 2, &["z in A"][..]),
        ("set heads", set_heads, 2, &["z in A"][..]),
        ("equality", equality.clone(), 2, &["z = z"][..]),
        (
            "header equality",
            header_equality,
            2,
            &["z = z", "z in A"][..],
        ),
        (
            "proof equality",
            proof_equality,
            2,
            &["z in A", "z = z"][..],
        ),
        (
            "equality alpha",
            equality.replace('z', "q"),
            2,
            &["q = q"][..],
        ),
        ("equality layout", equality_layout, 2, &["z = z"][..]),
        (
            "noncondition equality",
            noncondition_equality,
            2,
            &["z in A"][..],
        ),
        ("zero", zero, 0, &["z in A"][..]),
    ];
    let mut foreign_lower = None;
    let mut foreign_composition: Option<
        mizar_checker::source_formula_composition::SourceConditionFormulaCompositionHandoff,
    > = None;
    for (label, source, count, condition_texts) in cases {
        let ast = parse(&source);
        let symbols = resolver_symbol_collection(root, case, &ast);
        assert!(
            symbols.detail_keys.is_empty(),
            "{label}: {:?}",
            symbols.detail_keys
        );
        let module = &symbols.module;
        let scope = SourceVariableScopeResolver::resolve_occurrences(
            SourceVariableScopeInput::new(&ast, module, &symbols.env),
        )
        .unwrap();
        let bindings = SourceVariableSemanticsChecker::occurrence_binding_env(&scope);
        let typed = super::super::super::type_elaboration::step5c7_membership_typed_ast(
            &ast,
            module,
            &symbols.env,
            &scope,
            &bindings,
        )
        .unwrap();
        let primary = typed.source_term().unwrap();
        let sets = typed.source_set_term().unwrap();
        let atomic = typed.source_atomic_formula().unwrap();
        if label == "noncondition equality" {
            let outside = ast
                .node_views()
                .find(|view| {
                    let node = ast.node(view.id()).unwrap();
                    matches!(node.kind, SurfaceNodeKind::BuiltinPredicateApplication)
                        && node
                            .children
                            .iter()
                            .filter_map(|child| ast.node(*child).unwrap().token_text())
                            .collect::<Vec<_>>()
                            == ["="]
                })
                .expect("actual noncondition equality");
            assert!(
                atomic
                    .formulas()
                    .iter()
                    .all(|(_, row)| row.site().node().index() != outside.id().index())
            );
            assert_ne!(
                typed
                    .nodes()
                    .node(mizar_checker::typed_ast::TypedNodeId::new(
                        outside.id().index()
                    ))
                    .unwrap()
                    .kind
                    .as_str(),
                "source.formula.atomic.equality"
            );
        }
        let application = SourceFunctorApplicationProducer::build(
            SourceFunctorApplicationHandoffInput {
                source_id: ast.source_id,
                module_id: module.clone(),
                applications: vec![],
                wrappers: vec![],
                candidates: vec![],
                arguments: vec![],
                type_requests: vec![],
            },
            &symbols.env,
            &bindings,
            primary,
            typed.nodes(),
        )
        .unwrap();
        assert!(application.applications().is_empty());
        let typed_with_application = typed
            .clone()
            .with_source_application(application.clone())
            .unwrap();
        let baseline = super::super::super::type_elaboration::assemble_empty_resolved_typed_ast(
            &typed_with_application,
            vec![],
        )
        .unwrap();
        assert!(baseline.source_condition_formula_composition().is_none());
        assert!(baseline.checked_formulas().is_empty());
        assert!(baseline.statement_semantics().is_empty());
        assert!(baseline.checked_proofs().is_empty());

        // The source literal fixes the condition census. Resolve every literal
        // to its actual wrapper and direct atomic child before looking up
        // lower IDs; no expected association comes from C2 output.
        let mut literal_positions = condition_texts
            .iter()
            .flat_map(|condition_text| {
                source
                    .match_indices(*condition_text)
                    .filter_map(|(start, _)| {
                        source[..start].trim_end().ends_with(':').then_some((
                            start,
                            start + condition_text.len(),
                            *condition_text,
                        ))
                    })
            })
            .collect::<Vec<_>>();
        literal_positions.sort_by_key(|(start, _, _)| *start);
        assert_eq!(literal_positions.len(), count, "{label}");
        assert_eq!(sets.conditions().len(), count, "{label}");
        let mut edges = Vec::new();
        for (start, end, condition_text) in literal_positions {
            let (wrapper_id, wrapper) = ast
                .node_views()
                .find_map(|view| {
                    let node = ast.node(view.id())?;
                    (matches!(node.kind, SurfaceNodeKind::FormulaExpression)
                        && node.range.start == start
                        && node.range.end == end)
                        .then_some((view.id(), node))
                })
                .expect("literal condition wrapper");
            let children = wrapper
                .children
                .iter()
                .copied()
                .filter(|id| !matches!(ast.node(*id).unwrap().kind, SurfaceNodeKind::Token(_)))
                .collect::<Vec<_>>();
            let [formula_id] = children.as_slice() else {
                panic!("{label}: condition is not a direct atomic child")
            };
            let formula = ast.node(*formula_id).unwrap();
            assert!(matches!(
                formula.kind,
                SurfaceNodeKind::BuiltinPredicateApplication
            ));
            assert_eq!(formula.range, wrapper.range);
            let expected_kind = if condition_text.contains(" = ") {
                SourceAtomicFormulaKind::Equality
            } else {
                SourceAtomicFormulaKind::Membership
            };
            let written_operator = formula
                .children
                .iter()
                .filter_map(|child| ast.node(*child).unwrap().token_text())
                .collect::<Vec<_>>();
            assert_eq!(
                written_operator,
                if expected_kind == SourceAtomicFormulaKind::Equality {
                    ["="]
                } else {
                    ["in"]
                }
            );
            let (condition_id, condition) = sets
                .conditions()
                .iter()
                .find(|(_, row)| row.condition_site().node().index() == wrapper_id.index())
                .expect("actual lower condition wrapper");
            let set_parent = ast
                .node_views()
                .find_map(|view| {
                    let node = ast.node(view.id())?;
                    (matches!(node.kind, SurfaceNodeKind::SetComprehension)
                        && node.children.contains(&wrapper_id))
                    .then_some(view.id())
                })
                .expect("actual comprehension parent");
            let set_id = sets
                .terms()
                .iter()
                .find(|(_, row)| row.site().node().index() == set_parent.index())
                .map(|(id, _)| id)
                .expect("actual lower set owner");
            let (atomic_id, atomic_row) = atomic
                .formulas()
                .iter()
                .find(|(_, row)| row.site().node().index() == formula_id.index())
                .expect("actual lower direct condition atom");
            assert_eq!(condition.source_range(), wrapper.range);
            assert_eq!(atomic_row.source_range(), formula.range);
            assert_eq!(condition.spelling(), condition_text);
            assert_eq!(atomic_row.spelling(), condition_text);
            assert_eq!(atomic_row.kind(), expected_kind);
            assert_eq!(condition.term(), set_id);
            assert_eq!(condition.ordinal(), 0);
            assert_eq!(
                atomic_row.context(),
                sets.terms().get(condition.term()).unwrap().context()
            );
            let formula_edges = atomic
                .edges()
                .iter()
                .filter(|(_, row)| row.formula() == atomic_id)
                .map(|(id, _)| id)
                .collect::<Vec<_>>();
            assert_eq!(formula_edges.len(), 2);
            let formula_requests = atomic
                .requests()
                .iter()
                .filter(|(_, row)| row.formula() == atomic_id)
                .map(|(_, row)| row)
                .collect::<Vec<_>>();
            let expected_request_edges = if expected_kind == SourceAtomicFormulaKind::Equality {
                formula_edges.as_slice()
            } else {
                &formula_edges[1..]
            };
            assert_eq!(formula_requests.len(), expected_request_edges.len());
            for (ordinal, (request, edge)) in formula_requests
                .into_iter()
                .zip(expected_request_edges)
                .enumerate()
            {
                assert_eq!(request.ordinal(), ordinal);
                assert_eq!(request.kind(), SourceAtomicRequestKind::OperandExpectedType);
                assert_eq!(request.edge(), Some(*edge));
                assert!(request.candidate().is_none());
                assert!(request.type_site().is_none());
                assert!(request.attribute().is_none());
            }
            edges.push(SourceConditionFormulaEdgeInput {
                condition: condition_id,
                ordinal: 0,
                formula: atomic_id,
            });
        }
        let input = SourceConditionFormulaCompositionHandoffInput {
            source_id: ast.source_id,
            module_id: module.clone(),
            edges,
        };
        let build = |candidate| {
            SourceConditionFormulaCompositionProducer::build(
                candidate,
                primary,
                &application,
                sets,
                atomic,
                typed.nodes(),
            )
        };
        if count == 0 {
            assert!(build(input).is_err(), "zero-condition C2 admitted");
            continue;
        }
        let composition = build(input.clone()).expect("RED native C2 associations");
        assert_eq!(composition.edges().len(), count);
        for (index, (id, row)) in composition.edges().iter().enumerate() {
            assert_eq!(id.index(), index);
            assert_eq!(row.condition(), input.edges[index].condition);
            assert_eq!(row.ordinal(), input.edges[index].ordinal);
            assert_eq!(row.formula(), input.edges[index].formula);
        }
        assert_eq!(composition.primary_term_fingerprint(), primary.debug_text());
        assert_eq!(
            composition.application_fingerprint(),
            application.debug_text()
        );
        assert_eq!(composition.set_term_fingerprint(), sets.debug_text());
        assert_eq!(
            composition.atomic_formula_fingerprint(),
            atomic.debug_text()
        );
        let installed = typed_with_application
            .clone()
            .with_source_condition_formula_composition(composition.clone())
            .unwrap();
        assert_eq!(
            installed.source_term(),
            typed_with_application.source_term()
        );
        assert_eq!(
            installed.source_application(),
            typed_with_application.source_application()
        );
        assert_eq!(
            installed.source_set_term(),
            typed_with_application.source_set_term()
        );
        assert_eq!(
            installed.source_atomic_formula(),
            typed_with_application.source_atomic_formula()
        );
        let final_ast = super::super::super::type_elaboration::assemble_empty_resolved_typed_ast(
            &installed,
            vec![],
        )
        .unwrap();
        assert_eq!(
            final_ast.source_condition_formula_composition(),
            Some(&composition)
        );
        assert_eq!(final_ast.source_term(), baseline.source_term());
        assert_eq!(
            final_ast.source_application(),
            baseline.source_application()
        );
        assert_eq!(final_ast.source_set_term(), baseline.source_set_term());
        assert_eq!(
            final_ast.source_atomic_formula(),
            baseline.source_atomic_formula()
        );
        assert!(final_ast.checked_formulas().is_empty());
        assert!(final_ast.statement_semantics().is_empty());
        assert!(final_ast.checked_proofs().is_empty());
        if matches!(label, "original" | "equality") {
            use mizar_checker::source_set_term::SourceSetTermProducer;
            use mizar_resolve::{
                labels::{LabelResolver, ProofLabelSourceCollector},
                resolved_ast::SurfaceResolvedArena,
            };
            let source_arena = SurfaceResolvedArena::lower(&ast, module).unwrap();
            let owner = symbols
                .env
                .symbols()
                .iter()
                .find(|entry| entry.kind() == mizar_resolve::env::SymbolKind::Theorem)
                .unwrap();
            let namespace = mizar_resolve::env::NamespacePath::new(module.path().as_str());
            let labels = ProofLabelSourceCollector::new(
                &ast,
                module,
                namespace.clone(),
                owner.contribution(),
                &source_arena,
            )
            .unwrap()
            .collect_with_let_conditions()
            .unwrap();
            let resolved = LabelResolver::new(labels.projections()).resolve(
                module,
                &namespace,
                labels.references(),
            );
            assert!(!resolved.has_unresolved());
            assert_eq!(
                SourceSetTermProducer::check_membership_proof(
                    &installed,
                    &scope,
                    &bindings,
                    &labels,
                    &resolved,
                    &symbols.env,
                )
                .is_ok(),
                label == "original",
                "{label}: wrong proof credit"
            );
        }
        assert!(
            installed
                .clone()
                .with_source_condition_formula_composition(composition.clone())
                .is_err()
        );
        if let Some(stale) = &foreign_composition {
            assert!(
                typed_with_application
                    .clone()
                    .with_source_condition_formula_composition(stale.clone())
                    .is_err(),
                "{label}: stale C2 installation"
            );
            assert!(
                typed_with_application
                    .source_condition_formula_composition()
                    .is_none(),
                "{label}: failed installation changed its input"
            );
        }

        let mut missing = input.clone();
        missing.edges.pop();
        assert!(build(missing).is_err(), "{label}: missing condition");
        let mut duplicated = input.clone();
        duplicated.edges.push(input.edges[0].clone());
        assert!(build(duplicated).is_err(), "{label}: duplicate condition");
        let mut extra = input.clone();
        extra.edges.push(SourceConditionFormulaEdgeInput {
            condition: SourceSetConditionId::new(count),
            ordinal: 0,
            formula: input.edges[0].formula,
        });
        assert!(build(extra).is_err(), "{label}: extra condition");
        let mut wrong_ordinal = input.clone();
        wrong_ordinal.edges[0].ordinal = 1;
        assert!(build(wrong_ordinal).is_err(), "{label}: wrong ordinal");
        let unrelated = atomic
            .formulas()
            .iter()
            .find(|(id, _)| *id != input.edges[0].formula)
            .map(|(id, _)| id)
            .unwrap();
        let mut nondirect = input.clone();
        nondirect.edges[0].formula = unrelated;
        assert!(build(nondirect).is_err(), "{label}: nondirect formula");
        if count > 1 {
            let mut reordered = input.clone();
            reordered.edges.swap(0, 1);
            assert!(build(reordered).is_err(), "{label}: reordered conditions");
            let mut wrong_condition = input.clone();
            wrong_condition.edges[0].condition = input.edges[1].condition;
            assert!(build(wrong_condition).is_err(), "{label}: wrong condition");
            let mut cross_owner = input.clone();
            cross_owner.edges[0].formula = input.edges[1].formula;
            assert!(build(cross_owner).is_err(), "{label}: cross-owner formula");
        }
        let mut foreign_source = input.clone();
        use mizar_session::SessionIdAllocator;
        let ids = mizar_session::InMemorySessionIdAllocator::new();
        ids.next_source_id(snapshot_id(778)).unwrap();
        foreign_source.source_id = ids.next_source_id(snapshot_id(778)).unwrap();
        assert_ne!(foreign_source.source_id, input.source_id);
        assert!(build(foreign_source).is_err(), "{label}: foreign source");
        let mut foreign_module = input.clone();
        foreign_module.module_id = mizar_resolve::resolved_ast::ModuleId::new(
            module.package().clone(),
            mizar_session::ModulePath::new("foreign.condition-composition"),
        );
        assert!(build(foreign_module).is_err(), "{label}: foreign module");
        if let Some((other_primary, other_arena)) = &foreign_lower {
            assert!(
                SourceConditionFormulaCompositionProducer::build(
                    input.clone(),
                    other_primary,
                    &application,
                    sets,
                    atomic,
                    typed.nodes(),
                )
                .is_err(),
                "{label}: foreign primary"
            );
            assert!(
                SourceConditionFormulaCompositionProducer::build(
                    input.clone(),
                    primary,
                    &application,
                    sets,
                    atomic,
                    other_arena,
                )
                .is_err(),
                "{label}: foreign arena"
            );
        }
        assert_eq!(
            build(input).unwrap(),
            composition,
            "{label}: pristine replay"
        );
        foreign_lower = Some((primary.clone(), typed.nodes().clone()));
        foreign_composition = Some(composition);
    }
}

#[test]
fn step5c7_proof_admission_is_exact() {
    let config = config();
    let root = &config.workspace_root;
    let mut plan = crate::harness::build_test_plan(&config).unwrap();
    let diagnostics = validate_active_proof_verification_tags(root, &plan);
    assert!(diagnostics.is_empty(), "{diagnostics:?}");
    let original = plan
        .cases
        .iter()
        .find(|case| case.id.0 == STEP5C7_PROOF_CASES[0].0)
        .unwrap()
        .clone();
    for mutate in [
        |case: &mut TestCase| case.expectation.tags.clear(),
        |case: &mut TestCase| case.expectation.tags.push("extra".into()),
        |case: &mut TestCase| case.expectation.expected_phase = Some(PipelinePhase::TypeCheck),
        |case: &mut TestCase| case.expectation.expected_outcome = ExpectedOutcome::Fail,
        |case: &mut TestCase| case.expectation.stable_detail_key = Some("forged".into()),
        |case: &mut TestCase| case.expectation.source = "wrong.miz".into(),
        |case: &mut TestCase| case.id.0 = "unlisted".into(),
        |case: &mut TestCase| case.expectation.id.0 = "forged".into(),
        |case: &mut TestCase| case.expectation_path.set_file_name("wrong.expect.toml"),
    ] {
        let mut altered = original.clone();
        mutate(&mut altered);
        assert!(!step5c7_proof_workspace_member(root, &altered));
        assert_eq!(
            run_proof_verification_case(root, &root.join("tests"), &altered, 0).status,
            ProofVerificationCaseStatus::Failed
        );
    }
    let mut wrong_root = original.clone();
    wrong_root.source_path = root.join("alias").join(STEP5C7_PROOF_CASES[0].1);
    assert!(!step5c7_proof_workspace_member(root, &wrong_root));
    plan.cases.push(original);
    assert!(!validate_active_proof_verification_tags(root, &plan).is_empty());
    plan.cases
        .retain(|case| case.id.0 != STEP5C7_PROOF_CASES[1].0);
    assert!(
        validate_active_proof_verification_tags(root, &plan)
            .iter()
            .any(|diagnostic| diagnostic.detail_key.ends_with(STEP5C7_PROOF_CASES[1].0))
    );
}

#[test]
fn step5c7_qua_checks_both_source_scopes_and_rejects_unrelated_errors() {
    let config = config();
    let root = &config.workspace_root;
    let plan = crate::harness::build_test_plan(&config).unwrap();
    let case = plan
        .cases
        .iter()
        .find(|case| case.id.0 == "fail_type_elaboration_term_qua_invalid_narrowing_001")
        .unwrap();
    let source = fs::read_to_string(&case.source_path).unwrap();
    let keys = |source: &str| {
        let ast = parse(source);
        let symbols = resolver_symbol_collection(root, case, &ast);
        if !symbols.detail_keys.is_empty() {
            return symbols.detail_keys;
        }
        super::super::super::type_elaboration::step5c7_term_detail_keys(
            &ast,
            &symbols.module,
            &symbols.env,
        )
    };
    let invalid = vec!["terms.qua.invalid_narrowing".to_owned()];
    assert_eq!(keys(&source), invalid);
    let legal = source.replace("qua QuaBox", "qua object");
    assert!(keys(&legal).is_empty());
    assert!(keys(&source.replace("qua QuaBox", "qua set")).is_empty());
    assert_eq!(
        keys(&source.replacen("qua QuaBox", "qua object", 1)),
        invalid
    );
    assert_eq!(
        keys(&legal.replacen("qua object", "qua QuaBox", 1)),
        invalid
    );
    let renamed = source
        .replace("QuaBox", "Crate")
        .replace("field d", "field payload")
        .replace("X", "Y");
    assert_eq!(keys(&renamed), invalid);
    assert!(keys(&renamed.replace("qua Crate", "qua object")).is_empty());
    let (_, theorem) = source.split_once("theorem").unwrap();
    let definition = source.split_once("theorem").unwrap().0;
    for changed in [
        source.replace("qua QuaBox", "qua Unknown"),
        source.replacen("X qua", "Z qua", 1),
        source.replace("being set", "being object"),
        source.replace("be set", "be object"),
        source.replace("field d -> set;", "field d -> object;"),
        source.replace("field d -> set;", "field d -> set; field d -> set;"),
        source.replace("field d -> set;", "field d -> ;"),
        source.replace("struct QuaBox where", "struct QuaBox -> Missing where"),
        source.replace("qua QuaBox", "qua empty QuaBox"),
        source.replace(
            "struct QuaBox where\n    field d -> set;\n  end;",
            "mode QuaBox is set;",
        ),
        format!("theorem{theorem}{definition}"),
        source.replace("thus (X qua QuaBox)", "thus (Z qua object)"),
    ] {
        assert_ne!(keys(&changed), invalid, "{changed}");
    }
    let ast = parse(&source);
    let symbols = resolver_symbol_collection(root, case, &ast);
    let mut pending = vec![ast.root().unwrap()];
    let mut reachable = std::collections::BTreeSet::new();
    while let Some(id) = pending.pop() {
        assert!(
            reachable.insert(id),
            "duplicate structural parent for {id:?}"
        );
        let node = ast.node(id).unwrap();
        pending.extend(node.children.iter().copied().filter(|child| {
            node.kind != mizar_syntax::SurfaceNodeKind::Root
                || !matches!(
                    ast.node(*child).unwrap().kind,
                    mizar_syntax::SurfaceNodeKind::Token(_)
                )
        }));
    }
    assert_eq!(reachable.len(), ast.nodes().len(), "orphan source nodes");
    let scope = mizar_resolve::names::SourceVariableScopeResolver::resolve_occurrences(
        mizar_resolve::names::SourceVariableScopeInput::new(&ast, &symbols.module, &symbols.env),
    )
    .unwrap();
    let bindings = scope
        .references()
        .iter()
        .map(|reference| reference.binding())
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(bindings.len(), 2);
    let qua_sites = ast
        .nodes()
        .iter()
        .filter(|node| node.kind == mizar_syntax::SurfaceNodeKind::QuaExpression)
        .map(|node| node.range)
        .collect::<Vec<_>>();
    assert_eq!(qua_sites.len(), 2);
    assert_ne!(qua_sites[0], qua_sites[1]);
    let renamed_ast = parse(&renamed);
    let stale = super::super::super::type_elaboration::step5c7_term_detail_keys(
        &renamed_ast,
        &symbols.module,
        &symbols.env,
    );
    assert_ne!(stale, invalid);
    let mut foreign_ast = ast.clone();
    use mizar_session::SessionIdAllocator;
    let ids = mizar_session::InMemorySessionIdAllocator::new();
    ids.next_source_id(snapshot_id(777)).unwrap();
    foreign_ast.source_id = ids.next_source_id(snapshot_id(777)).unwrap();
    assert_ne!(
        super::super::super::type_elaboration::step5c7_term_detail_keys(
            &foreign_ast,
            &symbols.module,
            &symbols.env
        ),
        invalid
    );
}

#[test]
fn step5c7_term_evidence_is_source_derived() {
    let config = config();
    let root = &config.workspace_root;
    let plan = crate::harness::build_test_plan(&config).unwrap();
    let case = plan
        .cases
        .iter()
        .find(|case| case.id.0 == STEP5C7_PROOF_CASES[0].0)
        .unwrap();
    let keys = |source: &str| {
        let ast = parse(source);
        let symbols = resolver_symbol_collection(root, case, &ast);
        super::super::super::type_elaboration::step5c7_term_detail_keys(
            &ast,
            &symbols.module,
            &symbols.env,
        )
    };
    let qua = fs::read_to_string(
        root.join("tests/miz/pass/terms/pass_type_elaboration_term_qua_widening_001.miz"),
    )
    .unwrap();
    assert!(keys(&qua).is_empty());
    assert!(
        !keys(
            &qua.replace("being set", "being object")
                .replace("be set", "be object")
                .replace("qua object", "qua set")
        )
        .is_empty()
    );
    let wrong_base = "theorem T: for x being object for y being set holds (x qua set) = y proof let x be object; let y be set; thus (x qua set) = y; end;";
    assert!(!keys(wrong_base).is_empty());
    let choice = fs::read_to_string(
        root.join("tests/miz/fail/terms/fail_type_elaboration_term_choice_uninhabited_001.miz"),
    )
    .unwrap();
    let mixed = choice.replacen(
        "the emptyish set = the emptyish set",
        "the set = the emptyish set",
        1,
    );
    assert_eq!(keys(&mixed), ["terms.choice.missing_inhabitation"]);
    assert_eq!(
        keys(&choice.replace("emptyish", "barren")),
        ["terms.choice.missing_inhabitation"]
    );
    assert!(!keys("theorem T: 1 = 1 proof thus 1 = 1; end;").is_empty());
    let mut ast = parse(&fs::read_to_string(&case.source_path).unwrap());
    let symbols = resolver_symbol_collection(root, case, &ast);
    use mizar_session::SessionIdAllocator;
    let ids = mizar_session::InMemorySessionIdAllocator::new();
    ids.next_source_id(snapshot_id(777)).unwrap();
    ast.source_id = ids.next_source_id(snapshot_id(777)).unwrap();
    assert!(normalize_term_ast(&ast, &symbols.module, &symbols.env).is_err());
}
