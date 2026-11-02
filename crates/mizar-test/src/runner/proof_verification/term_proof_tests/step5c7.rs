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
