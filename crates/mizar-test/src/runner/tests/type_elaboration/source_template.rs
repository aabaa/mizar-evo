use super::type_elaboration::{
    SOURCE_TEMPLATE_TEXT, source_template_output, source_template_output_with_mutation,
};
use mizar_checker::source_template::{
    SourceTemplateArgumentId, SourceTemplateArgumentsId, SourceTemplateLociId,
    SourceTemplateParameterKind, SourceTemplateParentKind, SourceTemplateRecovery,
};

fn task277a_fixture(
    ordinal: usize,
) -> (
    mizar_syntax::SurfaceAst,
    mizar_resolve::resolved_ast::ModuleId,
) {
    let (ast, module, _, _, diagnostics) =
        task253_ast_from_source_text_with_diagnostic_count(SOURCE_TEMPLATE_TEXT, 277_000 + ordinal);
    assert_eq!(diagnostics, 0, "Task277A fixture parser diagnostics");
    (ast, module)
}

#[test]
fn task277a_runner_extracts_exact_real_surface_profile() {
    assert_eq!(SOURCE_TEMPLATE_TEXT.len(), 207);
    assert!(SOURCE_TEMPLATE_TEXT.ends_with('\n'));
    let (ast, module) = task277a_fixture(0);
    let output = source_template_output(&ast, module, SOURCE_TEMPLATE_TEXT)
        .expect("Task277A must select the frozen parser fixture")
        .expect("Task277A exact parser transport must build");
    let handoff = &output.handoff;
    assert_eq!(output.typed_ast.nodes().len(), 116);
    assert_eq!(
        output.typed_ast.nodes().root(),
        Some(mizar_checker::typed_ast::TypedNodeId::new(115))
    );
    for (index, surface) in ast.nodes().iter().enumerate() {
        let typed = output
            .typed_ast
            .nodes()
            .node(mizar_checker::typed_ast::TypedNodeId::new(index))
            .expect("Task277A dense all-surface typed row");
        let expected_kind = match index {
            60 => "AbstractTypeSyntax".to_owned(),
            63 => "TypedValueSyntax".to_owned(),
            _ => format!("{:?}", surface.kind),
        };
        assert_eq!(typed.kind.as_str(), expected_kind);
        assert_eq!(typed.anchor, mizar_session::SourceAnchor::Range(surface.range));
        assert_eq!(
            typed.children,
            surface
                .children
                .iter()
                .map(|child| mizar_checker::typed_ast::TypedNodeId::new(child.index()))
                .collect::<Vec<_>>()
        );
        assert_eq!(
            typed.recovery,
            mizar_checker::typed_ast::NodeRecoveryState::Normal
        );
        assert_eq!(typed.typing, mizar_checker::typed_ast::TypingState::Unknown);
        assert_eq!(typed.resolved_node, None);
        assert_eq!(
            typed.links,
            mizar_checker::typed_ast::TypedNodeLinks::default()
        );
    }
    assert_eq!(handoff.parameters().len(), 2);
    assert_eq!(handoff.loci_groups().len(), 2);
    assert_eq!(handoff.loci().len(), 2);
    assert_eq!(handoff.argument_groups().len(), 2);
    assert_eq!(handoff.arguments().len(), 2);

    let parameters = handoff.parameters().iter().collect::<Vec<_>>();
    assert_eq!(
        parameters[0].1.site(),
        mizar_checker::typed_ast::TypedNodeId::new(60)
    );
    assert_eq!(
        parameters[0].1.parent(),
        mizar_checker::typed_ast::TypedNodeId::new(112)
    );
    assert_eq!(
        parameters[0].1.parent_kind(),
        SourceTemplateParentKind::DefinitionBlockItem
    );
    assert_eq!(
        parameters[0].1.kind(),
        SourceTemplateParameterKind::AbstractTypeSyntax
    );
    assert_eq!(parameters[0].1.source_range(), ast.nodes()[60].range);
    assert_eq!(parameters[0].1.source_ordinal(), 0);
    assert_eq!(parameters[0].1.recovery(), SourceTemplateRecovery::Normal);
    assert_eq!(
        parameters[1].1.site(),
        mizar_checker::typed_ast::TypedNodeId::new(63)
    );
    assert_eq!(
        parameters[1].1.parent(),
        mizar_checker::typed_ast::TypedNodeId::new(112)
    );
    assert_eq!(
        parameters[1].1.parent_kind(),
        SourceTemplateParentKind::DefinitionBlockItem
    );
    assert_eq!(
        parameters[1].1.kind(),
        SourceTemplateParameterKind::TypedValueSyntax
    );
    assert_eq!(parameters[1].1.source_range(), ast.nodes()[63].range);
    assert_eq!(parameters[1].1.source_ordinal(), 1);
    assert_eq!(parameters[1].1.recovery(), SourceTemplateRecovery::Normal);

    let loci = handoff.loci_groups().iter().collect::<Vec<_>>();
    assert_eq!(
        loci[0].1.site(),
        mizar_checker::typed_ast::TypedNodeId::new(65)
    );
    assert_eq!(
        loci[0].1.parent(),
        mizar_checker::typed_ast::TypedNodeId::new(66)
    );
    assert_eq!(
        loci[0].1.parent_kind(),
        SourceTemplateParentKind::PredicatePattern
    );
    assert_eq!(loci[0].1.source_range(), ast.nodes()[65].range);
    assert_eq!(loci[0].1.source_ordinal(), 0);
    assert_eq!(loci[0].1.recovery(), SourceTemplateRecovery::Normal);
    assert_eq!(
        loci[1].1.site(),
        mizar_checker::typed_ast::TypedNodeId::new(76)
    );
    assert_eq!(
        loci[1].1.parent(),
        mizar_checker::typed_ast::TypedNodeId::new(77)
    );
    assert_eq!(
        loci[1].1.parent_kind(),
        SourceTemplateParentKind::FunctorPattern
    );
    assert_eq!(loci[1].1.source_range(), ast.nodes()[76].range);
    assert_eq!(loci[1].1.source_ordinal(), 1);
    assert_eq!(loci[1].1.recovery(), SourceTemplateRecovery::Normal);
    let locus = handoff.loci().iter().collect::<Vec<_>>();
    assert_eq!(
        locus[0].0,
        mizar_checker::source_template::SourceTemplateLocusId::new(0)
    );
    assert_eq!(
        locus[0].1.site(),
        mizar_checker::typed_ast::TypedNodeId::new(64)
    );
    assert_eq!(locus[0].1.loci(), SourceTemplateLociId::new(0));
    assert_eq!(locus[0].1.ordinal(), 0);
    assert_eq!(locus[0].1.source_range(), ast.nodes()[64].range);
    assert_eq!(locus[0].1.source_ordinal(), 0);
    assert_eq!(locus[0].1.recovery(), SourceTemplateRecovery::Normal);
    assert_eq!(
        locus[1].1.site(),
        mizar_checker::typed_ast::TypedNodeId::new(75)
    );
    assert_eq!(locus[1].1.loci(), SourceTemplateLociId::new(1));
    assert_eq!(locus[1].1.ordinal(), 0);
    assert_eq!(locus[1].1.source_range(), ast.nodes()[75].range);
    assert_eq!(locus[1].1.source_ordinal(), 1);
    assert_eq!(locus[1].1.recovery(), SourceTemplateRecovery::Normal);

    let arguments = handoff.argument_groups().iter().collect::<Vec<_>>();
    assert_eq!(
        arguments[0].1.site(),
        mizar_checker::typed_ast::TypedNodeId::new(91)
    );
    assert_eq!(
        arguments[0].1.parent(),
        mizar_checker::typed_ast::TypedNodeId::new(92)
    );
    assert_eq!(
        arguments[0].1.parent_kind(),
        SourceTemplateParentKind::PredicateHead
    );
    assert_eq!(arguments[0].1.source_range(), ast.nodes()[91].range);
    assert_eq!(arguments[0].1.source_ordinal(), 0);
    assert_eq!(arguments[0].1.recovery(), SourceTemplateRecovery::Normal);
    assert_eq!(
        arguments[1].1.site(),
        mizar_checker::typed_ast::TypedNodeId::new(100)
    );
    assert_eq!(
        arguments[1].1.parent(),
        mizar_checker::typed_ast::TypedNodeId::new(101)
    );
    assert_eq!(
        arguments[1].1.parent_kind(),
        SourceTemplateParentKind::TermReference
    );
    assert_eq!(arguments[1].1.source_range(), ast.nodes()[100].range);
    assert_eq!(arguments[1].1.source_ordinal(), 1);
    assert_eq!(arguments[1].1.recovery(), SourceTemplateRecovery::Normal);
    let argument = handoff.arguments().iter().collect::<Vec<_>>();
    assert_eq!(argument[0].0, SourceTemplateArgumentId::new(0));
    assert_eq!(
        argument[0].1.site(),
        mizar_checker::typed_ast::TypedNodeId::new(90)
    );
    assert_eq!(argument[0].1.arguments(), SourceTemplateArgumentsId::new(0));
    assert_eq!(argument[0].1.ordinal(), 0);
    assert_eq!(argument[0].1.source_range(), ast.nodes()[90].range);
    assert_eq!(argument[0].1.source_ordinal(), 0);
    assert_eq!(argument[0].1.recovery(), SourceTemplateRecovery::Normal);
    assert_eq!(
        argument[1].1.site(),
        mizar_checker::typed_ast::TypedNodeId::new(99)
    );
    assert_eq!(argument[1].1.arguments(), SourceTemplateArgumentsId::new(1));
    assert_eq!(argument[1].1.ordinal(), 0);
    assert_eq!(argument[1].1.source_range(), ast.nodes()[99].range);
    assert_eq!(argument[1].1.source_ordinal(), 1);
    assert_eq!(argument[1].1.recovery(), SourceTemplateRecovery::Normal);
}

#[test]
fn task277a_runner_replays_deterministically_through_typed_and_resolved() {
    let (ast, module) = task277a_fixture(1);
    let first = source_template_output(&ast, module.clone(), SOURCE_TEMPLATE_TEXT)
        .expect("Task277A must select")
        .expect("first Task277A transport");
    let second = source_template_output(&ast, module, SOURCE_TEMPLATE_TEXT)
        .expect("Task277A must select on replay")
        .expect("second Task277A transport");
    assert_eq!(first.handoff, second.handoff);
    assert_eq!(first.handoff.debug_text(), second.handoff.debug_text());
    assert_eq!(first.typed_ast.debug_text(), second.typed_ast.debug_text());
    assert_eq!(first.resolved.debug_text(), second.resolved.debug_text());
    assert_eq!(first.typed_ast.source_template(), Some(&first.handoff));
    assert_eq!(first.resolved.source_template(), Some(&first.handoff));
    assert!(
        first
            .typed_ast
            .clone()
            .with_source_template(first.handoff.clone())
            .is_err()
    );
}

#[test]
fn task277a_runner_rejects_corrupt_template_edges_and_order() {
    let (ast, module) = task277a_fixture(2);
    let invalid_parent_kind =
        source_template_output_with_mutation(&ast, module.clone(), SOURCE_TEMPLATE_TEXT, |input| {
            input.loci_groups[0].parent_kind = SourceTemplateParentKind::FunctorPattern
        })
        .expect("Task277A source selection")
        .expect_err("wrong loci parent kind must fail");
    assert_eq!(
        invalid_parent_kind,
        "source template loci group 0 is invalid"
    );
    let invalid_parent_edge =
        source_template_output_with_mutation(&ast, module.clone(), SOURCE_TEMPLATE_TEXT, |input| {
            input.loci_groups[0].parent = mizar_checker::typed_ast::TypedNodeId::new(0)
        })
        .expect("Task277A source selection")
        .expect_err("non-direct loci edge must fail");
    assert_eq!(
        invalid_parent_edge,
        "source template loci group 0 is invalid"
    );
    let invalid_locus_group =
        source_template_output_with_mutation(&ast, module.clone(), SOURCE_TEMPLATE_TEXT, |input| {
            input.loci[0].loci = SourceTemplateLociId::new(1)
        })
        .expect("Task277A source selection")
        .expect_err("wrong locus group must fail");
    assert_eq!(invalid_locus_group, "source template locus 0 is invalid");
    let invalid_argument_group =
        source_template_output_with_mutation(&ast, module.clone(), SOURCE_TEMPLATE_TEXT, |input| {
            input.arguments[0].arguments = SourceTemplateArgumentsId::new(1)
        })
        .expect("Task277A source selection")
        .expect_err("wrong argument group must fail");
    assert_eq!(
        invalid_argument_group,
        "source template argument 0 is invalid"
    );
    let reordered =
        source_template_output_with_mutation(&ast, module, SOURCE_TEMPLATE_TEXT, |input| {
            input.argument_groups[1].source_ordinal = 0
        })
        .expect("Task277A source selection")
        .expect_err("reordered argument groups must fail");
    assert_eq!(
        reordered,
        "source template arguments group 1 is out of source order"
    );
}

#[test]
fn task277a_runner_stays_private_targetless_and_semantic_free() {
    let (ast, module) = task277a_fixture(3);
    let output = source_template_output(&ast, module.clone(), SOURCE_TEMPLATE_TEXT)
        .expect("Task277A must select")
        .expect("Task277A transport");
    assert_eq!(output.typed_ast.types().len(), 0);
    assert_eq!(output.typed_ast.facts().len(), 0);
    assert_eq!(output.typed_ast.coercions().len(), 0);
    assert_eq!(output.typed_ast.initial_obligations().len(), 0);
    assert_eq!(output.typed_ast.diagnostics().len(), 0);
    let changed = SOURCE_TEMPLATE_TEXT.replacen("TemplateUse", "TemplateUses", 1);
    let (changed_ast, changed_module, _, _, _) =
        task253_ast_from_source_text_with_diagnostic_count(&changed, 277_999);
    assert!(source_template_output(&changed_ast, changed_module, &changed).is_none());
    assert!(source_template_output(&ast, module, &changed).is_none());
}
fn step5c12_case(id: &str) -> (crate::harness::TestCase, String) {
    let case = build_test_plan(&step5c11_config())
        .unwrap()
        .cases
        .into_iter()
        .find(|case| case.id.0 == id)
        .unwrap();
    let source = std::fs::read_to_string(&case.source_path).unwrap();
    (case, source)
}

fn step5c12_inputs(
    case: &crate::harness::TestCase,
    text: &str,
) -> (
    mizar_resolve::resolved_ast::SurfaceResolvedArena,
    mizar_resolve::env::SymbolEnv,
) {
    let frontend = super::formula_statement::step5c8_test_frontend(text);
    assert!(
        frontend.diagnostics.is_empty(),
        "{text}\n{:?}",
        frontend.diagnostics
    );
    let ast = frontend.ast.unwrap();
    let symbols = super::resolver_symbol_collection(&step5c11_config().workspace_root, case, &ast);
    assert!(
        symbols.detail_keys.is_empty(),
        "{text}\n{:?}",
        symbols.detail_keys
    );
    (
        mizar_resolve::resolved_ast::SurfaceResolvedArena::lower(&ast, &symbols.module).unwrap(),
        symbols.env,
    )
}

fn step5c12_check(case: &crate::harness::TestCase, text: &str) -> Result<(), String> {
    let (source, symbols) = step5c12_inputs(case, text);
    mizar_checker::type_checker::check_source_unbounded_template_types(&source, &symbols)
}

#[test]
fn step5c12_real_templates_check_symbolic_formals_and_concrete_uses() {
    for (id, expected) in [
        (
            "pass_type_elaboration_template_type_param_functor_001",
            Ok(()),
        ),
        ("pass_type_elaboration_template_pred_param_001", Ok(())),
        ("pass_type_elaboration_template_extends_bound_001", Ok(())),
        (
            "fail_type_elaboration_template_bound_violation_001",
            Err("templates.argument.bound_violation".into()),
        ),
        (
            "fail_type_elaboration_template_arity_mismatch_001",
            Err("templates.argument.arity_mismatch".into()),
        ),
    ] {
        let (case, source) = step5c12_case(id);
        assert_eq!(step5c12_check(&case, &source), expected, "{id}");
        assert_eq!(
            step5c12_check(&case, &source),
            expected,
            "deterministic {id}"
        );
        let renamed = source
            .replace("TId", "RenamedId")
            .replace("THolds", "RenamedHolds")
            .replace("tid", "renamed_id")
            .replace('T', "U")
            .replace('P', "Q")
            .replace('A', "B")
            .replace(" x", " y")
            .replace("(x)", "(y)");
        assert_eq!(step5c12_check(&case, &renamed), expected, "renamed {id}");
        let result = super::run_type_elaboration_case(
            &step5c11_config().workspace_root,
            &step5c11_config().workspace_root.join("tests"),
            &case,
            0,
        );
        assert_eq!(
            result.status,
            super::TypeElaborationCaseStatus::Passed,
            "{result:?}"
        );
    }
}

#[test]
fn step5c12_identity_requires_the_actual_abstract_type_and_value_bindings() {
    let (case, source) = step5c12_case("pass_type_elaboration_template_type_param_functor_001");
    for (old, new) in [
        ("let x be T;", "let x be object;"),
        ("-> T equals", "-> set equals"),
        ("equals x", "equals A"),
        ("tid[T]", "tid[x]"),
        (
            "let T be type;\n  let x be T;",
            "let x be T;\n  let T be type;",
        ),
        ("let T be type;", "let T be type; let U be type;"),
        ("for A being set", "for A being object"),
        ("let A be set", "let A be object"),
        ("tid[set] A = A by", "tid[set] x = A by"),
        ("coherence;", "coherence; coherence;"),
    ] {
        let changed = source.replace(old, new);
        assert_ne!(changed, source);
        let result = step5c12_check(&case, &changed);
        assert!(result.is_err(), "accepted {old} => {new}");
        assert_ne!(result.unwrap_err(), "templates.argument.arity_mismatch");
    }
    let (_, symbols) = step5c12_inputs(&case, &source);
    let (renamed, _) = step5c12_inputs(&case, &source.replace("TIdDef", "ForeignDef"));
    assert!(
        mizar_checker::type_checker::check_source_unbounded_template_types(&renamed, &symbols)
            .is_err()
    );
}

#[test]
fn step5c12_arity_is_derived_from_each_explicit_application() {
    let (case, source) = step5c12_case("fail_type_elaboration_template_arity_mismatch_001");
    let mismatch = Err("templates.argument.arity_mismatch".to_owned());
    assert_eq!(step5c12_check(&case, &source), mismatch);
    assert_eq!(
        step5c12_check(&case, &source.replacen("[set, set]", "[set]", 1)),
        mismatch
    );
    let second_only = source.replace("thus tid4[set, set]", "thus tid4[set]");
    assert_eq!(step5c12_check(&case, &second_only), mismatch);
    let valid = source.replace("[set, set]", "[set]");
    assert_eq!(step5c12_check(&case, &valid), Ok(()));
    for changed in [
        valid.replace("[set]", ""),
        source.replace("equals x", "equals A"),
        source.replace("thus tid4[set, set] A", "thus tid4[set, set] x"),
        source.replace("for A being set", "for A being object"),
        source.replace("[set, set]", "[object, object]"),
    ] {
        let error = step5c12_check(&case, &changed).expect_err("unsupported source");
        assert_ne!(error, "templates.argument.arity_mismatch", "{changed}");
    }
}

#[test]
fn step5c12_predicate_formals_check_each_domain_and_quantified_argument() {
    let (case, source) = step5c12_case("pass_type_elaboration_template_pred_param_001");
    for (old, new) in [
        ("pred(T)", "pred(set)"),
        ("for x being T", "for x being set"),
        ("implies P(x)", "implies P(T)"),
        ("(P(x) implies", "(Q(x) implies"),
        ("implies P(x)", "implies P(x, x)"),
        ("let P be pred(T);", "let P be pred(T); let Q be pred(T);"),
        (
            "theorem THoldsDef: for x being T holds (P(x) implies P(x));",
            "pred THoldsDef: testp x means P(x);",
        ),
    ] {
        assert!(
            step5c12_check(&case, &source.replace(old, new)).is_err(),
            "{old} => {new}"
        );
    }
    let (resolved, _) = step5c12_inputs(&case, &source);
    let uses = resolved
        .arena()
        .iter()
        .filter_map(|(id, node)| match node.kind() {
            mizar_syntax::SurfaceNodeKind::Token(token) if token.text.as_ref() == "x" => Some(id),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(uses.len(), 3);
    for use_id in &uses[1..] {
        assert_eq!(
            mizar_resolve::names::resolve_template_formal(&resolved, *use_id).unwrap(),
            uses[0]
        );
    }
    assert!(mizar_resolve::names::resolve_template_formal(&resolved, uses[0]).is_err());
}

#[test]
fn step5c12_admission_reserves_all_template_rows_and_exact_endpoints() {
    let config = step5c11_config();
    let plan = build_test_plan(&config).unwrap();
    for original in plan
        .cases
        .iter()
        .filter(|case| super::is_step5c12_candidate(case))
    {
        assert!(super::step5c12_admitted(
            Some(&config.workspace_root),
            original
        ));
        for mutation in 0..25 {
            let mut case = (*original).clone();
            match mutation {
                0 => case.id.0.push_str("_alias"),
                1 => case.expectation.id.0.push_str("_alias"),
                2 => {
                    case.source_path = config.workspace_root.join("alias").join(
                        case.source_path
                            .strip_prefix(&config.workspace_root)
                            .unwrap(),
                    )
                }
                3 => {
                    case.expectation_path =
                        case.expectation_path.with_file_name("wrong.expect.toml")
                }
                4 => case.expectation.expected_phase = Some(crate::PipelinePhase::Resolve),
                5 => {
                    case.expectation.expected_outcome =
                        if original.expectation.expected_outcome == crate::ExpectedOutcome::Pass {
                            crate::ExpectedOutcome::Fail
                        } else {
                            crate::ExpectedOutcome::Pass
                        }
                }
                6 => {
                    case.expectation.failure_category =
                        if original.expectation.failure_category.is_some() {
                            None
                        } else {
                            Some("type_error".into())
                        }
                }
                7 => case.expectation.stable_detail_key = Some("wrong".into()),
                8 => case.expectation.tags.clear(),
                9 => case.expectation.tags.push("extra".into()),
                10 => case.expectation.snapshots = Some("wrong".into()),
                11 => case.expectation.rejection_reason = Some("wrong".into()),
                12 => case.expectation.diagnostic_codes.push("E-WRONG".into()),
                13 => case
                    .expectation
                    .declaration_symbol_payloads
                    .push("wrong".into()),
                14 => case.expectation.domain.push_str("_wrong"),
                15 => case.expectation.spec_refs[0].0.push_str("_wrong"),
                16 => case
                    .expectation
                    .spec_refs
                    .push(case.expectation.spec_refs[0].clone()),
                17 => case.expectation.source = "wrong.miz".into(),
                18 => {
                    case.expectation.kind =
                        if original.expectation.kind == crate::expectation::TestKind::Pass {
                            crate::expectation::TestKind::Fail
                        } else {
                            crate::expectation::TestKind::Pass
                        }
                }
                19 => case.expectation.stage = crate::Stage::AdvancedSemantics,
                20 => case.expectation.profiles.push("extra".into()),
                21 => case.expectation.schema_version = 2,
                22 => case.expectation.ast_profile = Some("wrong".into()),
                23 => case.expectation.snapshot_profiles.push("wrong".into()),
                24 => case.expectation.diagnostic_payloads.push("wrong".into()),
                _ => unreachable!(),
            }
            assert!(
                !super::step5c12_admitted(Some(&config.workspace_root), &case),
                "mutation {mutation}"
            );
        }
        for (stage, phase, tag) in [
            (
                crate::Stage::ParseOnly,
                crate::PipelinePhase::Parse,
                "active_parse_only",
            ),
            (
                crate::Stage::DeclarationSymbol,
                crate::PipelinePhase::Resolve,
                "active_declaration_symbol",
            ),
            (
                crate::Stage::FormulaStatement,
                crate::PipelinePhase::StatementCheck,
                "active_formula_statement",
            ),
            (
                crate::Stage::ProofVerification,
                crate::PipelinePhase::Verification,
                "active_proof_verification",
            ),
            (
                crate::Stage::AdvancedSemantics,
                crate::PipelinePhase::OverloadResolution,
                "active_advanced_semantics",
            ),
        ] {
            let mut case = (*original).clone();
            case.expectation.stage = stage;
            case.expectation.expected_phase = Some(phase);
            case.expectation.tags = vec![tag.into()];
            for only_expectation_identity in [false, true] {
                if only_expectation_identity {
                    case.id.0 = "alias".into();
                    case.source_path = config.workspace_root.join("alias.miz");
                    case.expectation_path = config.workspace_root.join("alias.expect.toml");
                }
                assert!(!super::is_active_type_elaboration(&case));
                assert!(!super::is_active_parse_only(&case));
                assert!(!super::is_active_declaration_symbol(&case));
                assert!(!super::is_active_proof_verification(&case));
                assert!(!super::formula_statement::is_active_formula_statement(
                    &config.workspace_root,
                    &case
                ));
                assert!(!super::step5c11_registration_admitted(
                    &config.workspace_root,
                    &case
                ));
            }
        }
        assert!(
            super::validate_active_type_elaboration_tags(&config.workspace_root, &plan).is_empty()
        );
        for duplicate in [false, true] {
            let mut changed = plan.clone();
            if duplicate {
                changed.cases.push((*original).clone());
            } else {
                changed.cases.retain(|case| case.id != original.id);
            }
            assert!(
                super::validate_active_type_elaboration_tags(&config.workspace_root, &changed)
                    .iter()
                    .any(|diagnostic| diagnostic.code.0 == "E-TYPE-ELABORATION-STEP5C12-INVENTORY")
            );
        }
    }
    let mut empty = plan;
    empty.cases.clear();
    assert!(
        super::validate_active_type_elaboration_tags(&config.workspace_root, &empty)
            .iter()
            .any(|diagnostic| diagnostic.code.0 == "E-TYPE-ELABORATION-STEP5C12-INVENTORY")
    );
}

#[test]
fn step5c12_formal_bindings_and_source_environment_are_authenticated() {
    use mizar_resolve::{
        names::resolve_template_formal,
        resolved_ast::{ModuleId, SurfaceResolvedArena},
    };
    let (case, text) = step5c12_case("pass_type_elaboration_template_type_param_functor_001");
    let (source, symbols) = step5c12_inputs(&case, &text);
    let uses = source.arena().iter().filter_map(|(_, node)| {
        if node.kind() != &mizar_syntax::SurfaceNodeKind::TermReference { return None; }
        let [reference] = node.children() else { return None; };
        matches!(source.arena().node(*reference).unwrap().kind(), mizar_syntax::SurfaceNodeKind::Token(token) if token.text.as_ref() == "A").then_some(*reference)
    }).collect::<Vec<_>>();
    assert_eq!(uses.len(), 4);
    let declarations = uses
        .iter()
        .map(|id| resolve_template_formal(&source, *id).unwrap())
        .collect::<Vec<_>>();
    assert_eq!(declarations[0], declarations[1]);
    assert_eq!(declarations[2], declarations[3]);
    assert_ne!(declarations[0], declarations[2]);
    for declaration in declarations {
        assert!(resolve_template_formal(&source, declaration).is_err());
    }
    let ast = super::formula_statement::step5c8_test_frontend(&text)
        .ast
        .unwrap();
    let foreign = ModuleId::new(
        mizar_session::PackageId::new("foreign"),
        mizar_session::ModulePath::new("foreign"),
    );
    let changed = SurfaceResolvedArena::lower(&ast, &foreign).unwrap();
    assert!(
        mizar_checker::type_checker::check_source_unbounded_template_types(&changed, &symbols)
            .is_err()
    );
    let recovered =
        super::formula_statement::step5c8_test_frontend(&text.replace("let x be T;", "let x be ;"));
    assert!(!recovered.diagnostics.is_empty());
    if let Some(ast) = recovered.ast
        && let Ok(recovered) = SurfaceResolvedArena::lower(&ast, source.module())
    {
        assert!(
            mizar_checker::type_checker::check_source_unbounded_template_types(
                &recovered, &symbols
            )
            .is_err()
        );
        assert!(resolve_template_formal(&recovered, uses[0]).is_err());
    }
}

#[test]
fn step5c12_direct_resolution_rejects_illegal_owners_and_application_identities() {
    use mizar_resolve::{names::resolve_template_formal, resolved_ast::SurfaceResolvedArena};
    use mizar_syntax::{SurfaceAstBuilder, SurfaceNodeKind as K};
    let (predicate_case, predicate_text) =
        step5c12_case("pass_type_elaboration_template_pred_param_001");
    let ordinary = predicate_text.replace(
        "theorem THoldsDef: for x being T holds (P(x) implies P(x));",
        "let x be T; pred FormalUseDef: formaluse x means P(x);",
    );
    for (text, valid) in [(&predicate_text, true), (&ordinary, false)] {
        let frontend = super::formula_statement::step5c8_test_frontend(text);
        assert!(
            frontend.diagnostics.is_empty(),
            "{:?}",
            frontend.diagnostics
        );
        let ast = frontend.ast.unwrap();
        let env = super::resolver_symbol_collection(
            &step5c11_config().workspace_root,
            &predicate_case,
            &ast,
        );
        let source = SurfaceResolvedArena::lower(&ast, &env.module).unwrap();
        let uses = source
            .arena()
            .iter()
            .filter(|(_, node)| node.kind() == &K::InlinePredicateApplication)
            .map(|(_, node)| node.children()[0])
            .collect::<Vec<_>>();
        assert!(!uses.is_empty());
        for reference in uses {
            assert_eq!(resolve_template_formal(&source, reference).is_ok(), valid);
        }
    }
    let (case, text) = step5c12_case("fail_type_elaboration_template_arity_mismatch_001");
    for changed in [
        text.replace("let T be type;", "let T be type; let T be type;"),
        text.replace(
            "let T be type;\n  let x be T;",
            "let x be T; let T be type;",
        ),
    ] {
        let ast = super::formula_statement::step5c8_test_frontend(&changed)
            .ast
            .unwrap();
        let env = super::resolver_symbol_collection(&step5c11_config().workspace_root, &case, &ast);
        let source = SurfaceResolvedArena::lower(&ast, &env.module).unwrap();
        let reference = source.arena().iter().find_map(|(_, node)| {
            if node.kind() != &K::TypeHead { return None; }
            let reference = *node.children().first()?;
            matches!(source.arena().node(reference)?.kind(), K::Token(token) if token.text.as_ref() == "T").then_some(reference)
        }).unwrap();
        assert!(resolve_template_formal(&source, reference).is_err());
    }
    for (id, callee, failure_key) in [
        (
            "fail_type_elaboration_template_arity_mismatch_001",
            "tid4",
            "templates.argument.arity_mismatch",
        ),
        (
            "fail_type_elaboration_template_bound_violation_001",
            "tid3",
            "templates.argument.bound_violation",
        ),
    ] {
        let (case, text) = step5c12_case(id);
        let (source, _) = step5c12_inputs(&case, &text);
        let ast = super::formula_statement::step5c8_test_frontend(&text)
            .ast
            .unwrap();
        for target in [1, 2] {
            let mut builder = SurfaceAstBuilder::new(ast.source_id);
            let mut rebuilt = Vec::new();
            let mut occurrence = 0;
            let mut changed_callee = None;
            for (index, node) in ast.nodes().iter().enumerate() {
                let children = node
                    .children
                    .iter()
                    .map(|child| rebuilt[child.index()])
                    .collect();
                let id = match &node.kind {
                    K::Token(token) => {
                        let changed = token.text.as_ref() == callee && {
                            occurrence += 1;
                            occurrence - 1 == target
                        };
                        if changed {
                            changed_callee = Some(index);
                        }
                        builder.add_token(
                            token.kind,
                            if changed {
                                "badc".into()
                            } else {
                                token.text.clone()
                            },
                            node.range,
                        )
                    }
                    K::PrefixExpression(operator)
                        if node.children.first().map(|id| id.index()) == changed_callee =>
                    {
                        let mut operator = operator.clone();
                        operator.spelling = "badc".into();
                        builder.add_node(K::PrefixExpression(operator), node.range, children)
                    }
                    kind => builder.add_node(kind.clone(), node.range, children),
                };
                rebuilt.push(id);
            }
            assert_eq!(occurrence, 3);
            let changed = builder.finish(Some(rebuilt[ast.root().unwrap().index()]), None);
            let changed_env = super::resolver_symbol_collection(
                &step5c11_config().workspace_root,
                &case,
                &changed,
            );
            assert!(changed_env.detail_keys.is_empty());
            let changed = SurfaceResolvedArena::lower(&changed, source.module()).unwrap();
            let symbols = changed_env.env;
            mizar_resolve::symbols::validate_source_symbol_env(&changed, &symbols).unwrap();
            let error = mizar_checker::type_checker::check_source_unbounded_template_types(
                &changed, &symbols,
            )
            .unwrap_err();
            assert_ne!(error, failure_key, "callee {target}");
        }
        let (definition, theorem) = text.split_once("\ntheorem").unwrap();
        let before = format!("theorem{theorem}\n{definition}\n");
        let keys = super::type_elaboration_detail_keys(
            &step5c11_config().workspace_root,
            &case,
            super::formula_statement::step5c8_test_frontend(&before),
            &mut None,
        );
        assert!(!keys.is_empty());
        assert!(!keys.iter().any(|key| key == failure_key));
    }
}

#[test]
fn step5c12_structure_bound_resolves_real_schema_and_member_identities() {
    use mizar_resolve::{env::SymbolKind, names::resolve_template_formal};
    use mizar_syntax::SurfaceNodeKind as K;
    let (case, text) = step5c12_case("pass_type_elaboration_template_extends_bound_001");
    let renamed = text
        .replace("BoundBox", "Container")
        .replace("carrier", "payload")
        .replace("TId2Def", "ExtractDef")
        .replace("tid2", "extract")
        .replace("let T ", "let Schema ")
        .replace("be T;", "be Schema;")
        .replace("[T]", "[Schema]")
        .replace("T.", "Schema.")
        .replace("let x ", "let element ")
        .replace("] x", "] element");
    for (text, ty, value, structure, member) in [
        (&text, "T", "x", "BoundBox", "carrier"),
        (&renamed, "Schema", "element", "Container", "payload"),
    ] {
        assert_eq!(step5c12_check(&case, text), Ok(()));
        let (source, env) = step5c12_inputs(&case, text);
        for (spelling, expected_count) in [(ty, 4), (value, 2)] {
            let tokens = source
                .arena()
                .iter()
                .filter_map(|(id, node)| {
                    matches!(node.kind(), K::Token(token) if token.text.as_ref() == spelling)
                        .then_some(id)
                })
                .collect::<Vec<_>>();
            assert_eq!(tokens.len(), expected_count);
            for reference in &tokens[1..] {
                assert_eq!(
                    resolve_template_formal(&source, *reference).unwrap(),
                    tokens[0]
                );
            }
            assert!(resolve_template_formal(&source, tokens[0]).is_err());
        }
        for (kind, spelling, node_kind) in [
            (SymbolKind::Structure, structure, K::StructureDefinition),
            (SymbolKind::Selector, member, K::StructureField),
        ] {
            let declaration = source
                .arena()
                .iter()
                .find(|(_, node)| node.kind() == &node_kind)
                .unwrap()
                .1;
            let entries = env
                .symbols()
                .iter()
                .filter(|entry| {
                    entry.kind() == kind && entry.origin().anchor() == declaration.origin().anchor()
                })
                .collect::<Vec<_>>();
            assert_eq!(entries.len(), 1);
            assert_eq!(entries[0].symbol().module(), source.module());
            assert!(entries[0].primary_spelling().contains(spelling));
        }
    }
    let collision = text
        .replace("let x ", "let carrier ")
        .replace("] x", "] carrier");
    assert_eq!(step5c12_check(&case, &collision), Ok(()));
    let (source, _) = step5c12_inputs(&case, &collision);
    let declaration_start = collision.find("let carrier").unwrap() + 4;
    let declaration = source
        .arena()
        .iter()
        .find_map(|(id, node)| {
            matches!(node.kind(), K::Token(token) if token.text.as_ref() == "carrier")
                .then_some((id, node.origin().anchor()))
                .and_then(|(id, anchor)| match anchor {
                    mizar_session::SourceAnchor::Range(range)
                        if range.start == declaration_start =>
                    {
                        Some(id)
                    }
                    _ => None,
                })
        })
        .unwrap();
    let pattern = source
        .arena()
        .iter()
        .find(|(_, node)| node.kind() == &K::FunctorPattern)
        .unwrap()
        .1;
    let selector = source
        .arena()
        .iter()
        .find(|(_, node)| node.kind() == &K::SelectorAccess)
        .unwrap()
        .1;
    assert_eq!(
        resolve_template_formal(&source, *pattern.children().last().unwrap()).unwrap(),
        declaration
    );
    assert!(resolve_template_formal(&source, selector.children()[2]).is_err());
    assert_ne!(selector.children()[2], declaration);
    let (original, env) = step5c12_inputs(&case, &text);
    let (changed, _) = step5c12_inputs(&case, &renamed);
    assert!(
        mizar_checker::type_checker::check_source_unbounded_template_types(&changed, &env).is_err()
    );
    let frontend =
        super::formula_statement::step5c8_test_frontend(&text.replace("T.carrier", "T."));
    assert!(!frontend.diagnostics.is_empty());
    if let Some(ast) = frontend.ast {
        let recovered =
            mizar_resolve::resolved_ast::SurfaceResolvedArena::lower(&ast, original.module())
                .unwrap();
        assert!(
            mizar_checker::type_checker::check_source_unbounded_template_types(&recovered, &env)
                .is_err()
        );
    }
}

#[test]
fn step5c12_structure_bound_rejects_definition_and_selector_near_misses() {
    let (case, text) = step5c12_case("pass_type_elaboration_template_extends_bound_001");
    for (old, new) in [
        ("T.carrier", "x"),
        ("T.carrier", "x.carrier"),
        ("T.carrier", "T.unknown"),
        ("field carrier -> set;", "field carrier -> object;"),
        ("-> set equals", "-> object equals"),
        ("tid2[T]", "tid2[x]"),
        ("let x be T;", "let x be object;"),
        ("extends BoundBox", "extends object"),
        ("extends BoundBox", "extends Missing"),
        (
            "field carrier -> set;",
            "field carrier -> set; field extra -> set;",
        ),
        ("let x be T;", "let U be type; let x be T;"),
        ("coherence;", "coherence; coherence;"),
    ] {
        let changed = text.replace(old, new);
        assert_ne!(changed, text);
        let error = step5c12_check(&case, &changed).expect_err("unsupported bounded declaration");
        assert_ne!(
            error, "templates.argument.bound_violation",
            "{old} => {new}"
        );
        assert_ne!(error, "templates.argument.arity_mismatch", "{old} => {new}");
    }
}

#[test]
fn step5c12_bound_violation_requires_both_actual_calls_and_their_bindings() {
    use mizar_resolve::names::resolve_template_formal;
    use mizar_syntax::SurfaceNodeKind as K;
    let (case, text) = step5c12_case("fail_type_elaboration_template_bound_violation_001");
    let failure = "templates.argument.bound_violation";
    for text in [
        text.clone(),
        text.replace("object", "set"),
        text.replace(
            "for a being object holds tid3[object]",
            "for a being set holds tid3[set]",
        ),
    ] {
        assert_eq!(step5c12_check(&case, &text), Err(failure.into()));
        let (source, _) = step5c12_inputs(&case, &text);
        let uses = source.arena().iter().filter_map(|(_, node)| {
            if node.kind() != &K::TermReference { return None; }
            let reference = node.children()[0];
            matches!(source.arena().node(reference).unwrap().kind(), K::Token(token) if token.text.as_ref() == "a").then_some(reference)
        }).collect::<Vec<_>>();
        assert_eq!(uses.len(), 4);
        let binders = uses
            .iter()
            .map(|id| resolve_template_formal(&source, *id).unwrap())
            .collect::<Vec<_>>();
        assert_eq!(binders[0], binders[1]);
        assert_eq!(binders[2], binders[3]);
        assert_ne!(binders[0], binders[2]);
    }
    let call = "tid3[object] a = a";
    let occurrences = text
        .match_indices(call)
        .map(|(offset, _)| offset)
        .collect::<Vec<_>>();
    assert_eq!(occurrences.len(), 2);
    for offset in occurrences {
        for replacement in [
            "tid3[object] T = a",
            "tid3[object] a = b",
            "tid3[set] a = a",
            "tid3[object, object] a = a",
            "tid3[BoundBox] a = a",
            "tid3 a = a",
        ] {
            let mut changed = text.clone();
            changed.replace_range(offset..offset + call.len(), replacement);
            let error = step5c12_check(&case, &changed).expect_err("unsupported independent call");
            assert_ne!(error, failure, "call offset{offset}: {replacement}");
            assert_ne!(error, "templates.argument.arity_mismatch");
        }
    }
    for (old, new) in [
        ("for a being object", "for a being set"),
        ("let a be object", "let a be set"),
        ("by TId3Def", "by MissingDef"),
    ] {
        let changed = text.replace(old, new);
        let error = step5c12_check(&case, &changed).expect_err("unsupported binder/citation");
        assert_ne!(error, failure, "{old} => {new}");
    }
    let matching = text.replace("object", "BoundBox");
    let error =
        step5c12_check(&case, &matching).expect_err("matching bound instantiation is deferred");
    assert_ne!(error, failure);
}
