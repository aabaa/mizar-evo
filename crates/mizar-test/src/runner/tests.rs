include!("tests/support.rs");

include!("tests/declaration_symbol.rs");

include!("tests/proof_verification.rs");

include!("tests/parse_only.rs");

include!("tests/syntax_smoke.rs");

include!("tests/type_elaboration/source_extraction.rs");

include!("tests/type_elaboration/reserved_binary.rs");

include!("tests/type_elaboration/mode_chain.rs");

include!("tests/type_elaboration/reserved_direct.rs");

include!("tests/type_elaboration/asserted_head_base.rs");

include!("tests/type_elaboration/asserted_head_four_edge_radix.rs");

include!("tests/type_elaboration/asserted_head_three_edge_object_radix.rs");

include!("tests/type_elaboration/asserted_head_two_edge_object_radix.rs");

include!("tests/type_elaboration/asserted_head_type_assertion.rs");

include!("tests/type_elaboration/binary_route_fixtures.rs");

include!("tests/type_elaboration/reserve_object_fixtures.rs");

include!("tests/type_elaboration/formula_constant_fixture.rs");

include!("tests/type_elaboration/reserve_fixtures.rs");

include!("tests/type_elaboration/mode_chain_fixtures.rs");

include!("tests/type_elaboration/asserted_head_fixtures.rs");

include!("tests/type_elaboration/source_gap_and_equality.rs");
include!("tests/type_elaboration/source_context.rs");
include!("tests/type_elaboration/source_type.rs");
include!("tests/type_elaboration/source_attribute.rs");
include!("tests/type_elaboration/source_evidence.rs");
include!("tests/type_elaboration/source_term.rs");
include!("tests/type_elaboration/source_application.rs");
include!("tests/type_elaboration/source_set_term.rs");
include!("tests/type_elaboration/source_structure.rs");
include!("tests/type_elaboration/source_atomic_formula.rs");
include!("tests/type_elaboration/source_composite_formula.rs");
include!("tests/type_elaboration/source_formula_composition.rs");
include!("tests/type_elaboration/source_statement.rs");
include!("tests/type_elaboration/source_attribute_definition.rs");
include!("tests/type_elaboration/source_functor_definition.rs");
include!("tests/type_elaboration/source_mode_definition.rs");
include!("tests/type_elaboration/source_structure_definition.rs");
include!("tests/type_elaboration/source_template.rs");
include!("tests/type_elaboration/template_parameter_identity.rs");
include!("tests/type_elaboration/fraenkel_generator_variable_identity.rs");
include!("tests/type_elaboration/fraenkel_nested_capture_identity.rs");
include!("tests/type_elaboration/template_type_parameter_association.rs");
include!("tests/type_elaboration/template_fraenkel_structural_composition.rs");
include!("tests/type_elaboration/template_fraenkel_generator_binding_context.rs");
include!("tests/type_elaboration/template_fraenkel_generator_bound_use.rs");
include!("tests/type_elaboration/source_predicate_definition.rs");
include!("tests/type_elaboration/source_proof_local_declaration.rs");
include!("tests/type_elaboration/source_property_implementation.rs");
include!("tests/type_elaboration/long_chain.rs");
include!("tests/type_elaboration/remaining_bridges_and_nested_isolation.rs");

fn step5c11_config() -> DiscoveryConfig {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf();
    DiscoveryConfig {
        workspace_root: root,
        tests_root: PathBuf::from("tests"),
        manifest_path: PathBuf::from("tests/coverage/spec_trace.toml"),
        profile: TestProfile::Fast,
        validation_mode: ValidationMode::Metadata,
    }
}

fn step5c11_source_case(family: &str) -> (crate::harness::TestCase, String) {
    let plan = build_test_plan(&step5c11_config()).unwrap();
    let id = format!("pass_advanced_semantics_{family}_registration_001");
    let case = plan.cases.into_iter().find(|case| case.id.0 == id).unwrap();
    let source = std::fs::read_to_string(&case.source_path).unwrap();
    (case, source)
}

fn step5c11_check_source(
    case: &crate::harness::TestCase,
    source: &str,
) -> Result<
    (
        mizar_checker::registration_resolution::RegistrationDatabase,
        mizar_checker::type_checker::TermFormulaInferenceOutput,
    ),
    String,
> {
    super::source_registration_intake(
        &step5c11_config().workspace_root,
        case,
        super::formula_statement::step5c8_test_frontend(source),
    )
}

#[test]
fn step5c11_four_real_sources_create_only_checked_pending_registrations() {
    use mizar_checker::registration_resolution::{
        PendingRegistrationStatus, RegistrationPatternStatus,
    };
    use mizar_checker::typed_ast::{InitialObligationKind, InitialObligationStatus};
    for family in ["existential", "conditional", "functorial", "reduce"] {
        let (case, source) = step5c11_source_case(family);
        for text in [
            source.clone(),
            format!(
                ":: renamed source\n{}",
                source
                    .replace('X', "Z")
                    .replace("marked", "marked_new")
                    .replace("cbox", "box_new")
            ),
        ] {
            let (database, inferred) = step5c11_check_source(&case, &text)
                .unwrap_or_else(|error| panic!("{family}: {error}"));
            assert!(inferred.diagnostics().is_empty(), "{family}");
            assert!(!inferred.terms().is_empty(), "{family}");
            assert!(!inferred.normalized_types().is_empty(), "{family}");
            assert_eq!(database.pending().len(), 1, "{family}");
            assert!(database.activated().is_empty());
            assert!(database.rejected().is_empty());
            assert!(database.diagnostics().is_empty());
            let pending = database.pending().iter().next().unwrap();
            assert!(matches!(
                pending.pattern_status(),
                RegistrationPatternStatus::Validated(_)
            ));
            assert_eq!(
                pending.status(),
                PendingRegistrationStatus::AwaitingVerifierAcceptance
            );
            assert!(!pending.may_contribute_to_inference());
            assert_eq!(pending.obligations().len(), 1);
            assert_eq!(database.initial_obligations().len(), 1);
            let obligation = database
                .initial_obligations()
                .get(pending.obligations()[0])
                .unwrap();
            assert_eq!(
                obligation.kind,
                InitialObligationKind::RegistrationCorrectness
            );
            assert_eq!(obligation.status, InitialObligationStatus::Pending);
            assert!(!obligation.goal.as_str().is_empty());
        }
    }
}

#[test]
fn step5c11_source_intake_rejects_wrong_bindings_types_guards_and_correctness() {
    let mutations = [
        ("existential", "means X = X", "means not X = X"),

        (
            "existential",
            "CReg1: cmarked set",
            "CReg1: non cmarked set",
        ),
        (
            "conditional",
            "CReg2: c2marked ->",
            "CReg2: non c2marked ->",
        ),
        ("conditional", "-> d2marked for", "-> non d2marked for"),
        ("functorial", "-> e3marked for", "-> non e3marked for"),
        ("existential", "CMDef: X is", "CMDef: Y is"),
        ("existential", "means X = X", "means X = Y"),
        ("existential", "let X be set", "let X be object"),
        (
            "existential",
            "cluster CReg1: cmarked set",
            "cluster CReg1: missing set",
        ),
        (
            "existential",
            "cluster CReg1: cmarked set",
            "cluster CReg1: cmarked object",
        ),
        ("existential", "existence", "coherence"),
        ("existential", "means X = X", "means contradiction"),
        ("conditional", "c2marked ->", "missing ->"),
        ("conditional", "-> d2marked", "-> missing"),
        (
            "functorial",
            "registration\n  let X be set;",
            "registration",
        ),
        ("functorial", "cbox X -> set", "cbox X -> object"),
        ("functorial", "CReg3: cbox X", "CReg3: cbox Y"),
        ("functorial", "for set", "for object"),
        ("functorial", "coherence;", "existence;"),
        ("functorial", "equals X", "equals Y"),
        ("reduce", "to X;", "to cbox2 X;"),
        ("reduce", "to X;", "to Y;"),
        ("reduce", "to X;", "to {X};"),
        ("reduce", "reducibility", "coherence"),
        (
            "reduce",
            "registration\n  let X be set;",
            "registration\n  let Y be set;",
        ),
    ];
    for (family, before, after) in mutations {
        let (case, source) = step5c11_source_case(family);
        assert!(source.contains(before), "mutation missing: {before}");
        let mutated = source.replace(before, after);
        assert!(
            step5c11_check_source(&case, &mutated).is_err(),
            "accepted {family}: {before} -> {after}"
        );
    }
}

#[test]
fn step5c11_pending_goal_preserves_ordered_attribute_operands() {
    let (case, source) = step5c11_source_case("conditional");
    let (original, _) = step5c11_check_source(&case, &source).unwrap();
    let swapped = source.replace("c2marked -> d2marked", "d2marked -> c2marked");
    let (changed, _) = step5c11_check_source(&case, &swapped).unwrap();
    let goal = |database: &mizar_checker::registration_resolution::RegistrationDatabase| {
        database
            .initial_obligations()
            .iter()
            .next()
            .unwrap()
            .1
            .goal
            .as_str()
            .to_owned()
    };
    assert_ne!(goal(&original), goal(&changed));
}

#[test]
fn step5c11_advanced_admission_is_exact_and_rejects_stage_fallback() {
    let config = step5c11_config();
    let (original, _) = step5c11_source_case("functorial");
    assert!(super::step5c11_registration_admitted(
        &config.workspace_root,
        &original
    ));
    for mutation in 0..12 {
        let mut case = original.clone();
        match mutation {
            0 => case.id.0.push_str("_extra"),
            1 => {
                case.source_path = config.workspace_root.join("alias").join(
                    case.source_path
                        .strip_prefix(&config.workspace_root)
                        .unwrap(),
                )
            }
            2 => case.expectation_path = case.expectation_path.with_file_name("wrong.expect.toml"),
            3 => case.expectation.stage = crate::staged_model::Stage::TypeElaboration,
            4 => {
                case.expectation.expected_phase = Some(crate::expectation::PipelinePhase::TypeCheck)
            }
            5 => case.expectation.expected_outcome = crate::expectation::ExpectedOutcome::Fail,
            6 => case.expectation.tags.clear(),
            7 => case.expectation.tags.push("extra".to_owned()),
            8 => case
                .expectation
                .diagnostic_codes
                .push("E-UNRELATED".to_owned()),
            9 => case.expectation.stable_detail_key = Some("wrong.detail".to_owned()),
            10 => case.expectation.tags = vec!["active_type_elaboration".to_owned()],
            11 => case.expectation.tags = vec!["active_proof_verification".to_owned()],
            _ => unreachable!(),
        }
        assert!(
            !super::step5c11_registration_admitted(&config.workspace_root, &case),
            "admitted mutation {mutation}"
        );
        assert!(!super::is_active_parse_only(&case));
        assert!(!super::is_active_declaration_symbol(&case));
        assert!(!super::is_active_type_elaboration(&case));
        assert!(!super::formula_statement::is_active_formula_statement(
            &config.workspace_root,
            &case
        ));
        assert!(!super::is_active_proof_verification(&case));
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
            crate::staged_model::Stage::TypeElaboration,
            crate::expectation::PipelinePhase::TypeCheck,
            "active_type_elaboration",
        ),
        (
            crate::staged_model::Stage::FormulaStatement,
            crate::expectation::PipelinePhase::StatementCheck,
            "active_formula_statement",
        ),
        (
            crate::staged_model::Stage::ProofVerification,
            crate::expectation::PipelinePhase::VcGeneration,
            "active_proof_verification",
        ),
    ] {
        let mut case = original.clone();
        case.expectation.stage = stage;
        case.expectation.expected_phase = Some(phase);
        case.expectation.tags = vec![tag.to_owned()];
        assert!(!super::is_active_parse_only(&case));
        assert!(!super::is_active_declaration_symbol(&case));
        assert!(!super::is_active_type_elaboration(&case));
        assert!(!super::formula_statement::is_active_formula_statement(
            &config.workspace_root,
            &case
        ));
        assert!(!super::is_active_proof_verification(&case));
    }
    let plan = build_test_plan(&config).unwrap();
    assert!(
        super::validate_step5c11_registration_inventory(&config.workspace_root, &plan).is_empty()
    );
    let mut missing = plan.clone();
    missing.cases.retain(|case| case.id != original.id);
    let mut duplicate = plan;
    duplicate.cases.push(original.clone());
    for invalid in [missing, duplicate] {
        assert!(
            super::validate_step5c11_registration_inventory(&config.workspace_root, &invalid)
                .iter()
                .any(|diagnostic| diagnostic.code.0 == "E-ADVANCED-SEMANTICS-INVENTORY")
        );
    }
    let report = super::run_advanced_semantics_corpus(&config).unwrap();
    assert_eq!(report.error_count(), 0, "{:?}", report.diagnostics);
    assert_eq!(report.results.len(), 4);
    assert_eq!(report.passed_count(), 4);
}

#[test]
fn step5c11_intake_authenticates_complete_typed_projection_and_symbol_environment() {
    use mizar_checker::registration_resolution::check_source_registration_intake;
    use mizar_checker::typed_ast::{
        NodeRecoveryState, TypedArena, TypedNode, TypedNodeId, TypingState,
    };
    use mizar_resolve::resolved_ast::SurfaceResolvedArena;
    let (case, text) = step5c11_source_case("functorial");
    let ast = super::formula_statement::step5c8_test_frontend(&text)
        .ast
        .unwrap();
    let collected =
        super::resolver_symbol_collection(&step5c11_config().workspace_root, &case, &ast);
    let source = SurfaceResolvedArena::lower(&ast, &collected.module).unwrap();
    let raw_nodes = source
        .arena()
        .iter()
        .map(|(id, node)| {
            TypedNode::new(format!("{:?}", node.kind()), node.origin().anchor().clone())
                .with_resolved_node(id)
                .with_children(
                    node.children()
                        .iter()
                        .map(|id| TypedNodeId::new(id.index()))
                        .collect(),
                )
        })
        .collect::<Vec<_>>();
    let root = Some(TypedNodeId::new(source.arena().root().index()));
    let typed = TypedArena::try_new(root, raw_nodes.clone()).unwrap();
    check_source_registration_intake(&source, &typed, &collected.env).unwrap();
    for mutation in 0..7 {
        let mut nodes = raw_nodes.clone();
        let mut actual_root = root;
        match mutation {
            0 => nodes[0].kind = "Unrelated".into(),
            1 => nodes[0].resolved_node = nodes[1].resolved_node,
            2 => nodes[0].anchor = nodes[1].anchor.clone(),
            3 => nodes[0].recovery = NodeRecoveryState::Recovered,
            4 => nodes[0].typing = TypingState::Successful,
            5 => nodes.last_mut().unwrap().children.clear(),
            6 => actual_root = None,
            _ => unreachable!(),
        }
        let corrupt = TypedArena::try_new(actual_root, nodes).unwrap();
        assert!(
            check_source_registration_intake(&source, &corrupt, &collected.env).is_err(),
            "accepted projection mutation {mutation}"
        );
    }
    let foreign_module =
        ResolverModuleId::new(PackageId::new("foreign"), ModulePath::new("foreign"));
    let foreign_source = SurfaceResolvedArena::lower(&ast, &foreign_module).unwrap();
    assert!(check_source_registration_intake(&foreign_source, &typed, &collected.env).is_err());
    let changed_ast =
        super::formula_statement::step5c8_test_frontend(&text.replace("cbox", "fbox"))
            .ast
            .unwrap();
    let changed =
        super::resolver_symbol_collection(&step5c11_config().workspace_root, &case, &changed_ast);
    assert!(check_source_registration_intake(&source, &typed, &changed.env).is_err());
}

#[test]
fn step5c11_parameter_lookup_preserves_lexical_identity_and_rejects_declaration_tokens() {
    use mizar_resolve::names::resolve_registration_parameter;
    use mizar_resolve::resolved_ast::SurfaceResolvedArena;
    let (case, text) = step5c11_source_case("functorial");
    let ast = super::formula_statement::step5c8_test_frontend(&text)
        .ast
        .unwrap();
    let collected =
        super::resolver_symbol_collection(&step5c11_config().workspace_root, &case, &ast);
    let source = SurfaceResolvedArena::lower(&ast, &collected.module).unwrap();
    let mut bindings = BTreeSet::new();
    for (_, node) in source
        .arena()
        .iter()
        .filter(|(_, node)| node.kind() == &SurfaceNodeKind::TermReference)
    {
        let [reference] = node.children() else {
            panic!("singleton reference expected")
        };
        let binding = resolve_registration_parameter(&source, *reference).unwrap();
        assert!(binding.index() < reference.index());
        assert!(resolve_registration_parameter(&source, binding).is_err());
        bindings.insert(binding);
    }
    assert_eq!(
        bindings.len(),
        3,
        "attribute, functor and registration have distinct X bindings"
    );
}

#[test]
fn step5c11_source_intake_rejects_forward_declarations_and_unowned_requests() {
    let (case, source) = step5c11_source_case("functorial");
    let (definitions, registration) = source.split_once("registration\n").unwrap();
    let reversed = format!("registration\n{registration}\n{definitions}");
    assert!(step5c11_check_source(&case, &reversed).is_err());
    let late_parameter = source
        .replace("registration\n  let X be set;", "registration")
        .replace("  coherence;", "  coherence;\n  let X be set;");
    assert!(step5c11_check_source(&case, &late_parameter).is_err());
    let duplicate_parameter = source.replace(
        "registration\n  let X be set;",
        "registration\n  let X be set;\n  let X be set;",
    );
    assert!(step5c11_check_source(&case, &duplicate_parameter).is_err());
    let duplicated_registration = format!(
        "{source}\nregistration\n{}",
        registration.replace("CReg3", "DReg3")
    );
    assert!(step5c11_check_source(&case, &duplicated_registration).is_err());
}

#[test]
fn step5c11_correctness_requests_bind_actual_owners_guards_and_checked_operands() {
    use mizar_checker::typed_ast::TypedNodeId;
    use mizar_resolve::resolved_ast::SurfaceResolvedArena;
    for (family, schema) in [
        ("existential", "exists.domain_and_attribute"),
        (
            "conditional",
            "forall.domain_and_antecedent_implies_consequent",
        ),
        (
            "functorial",
            "forall.parameters_and_result_implies_attribute",
        ),
        ("reduce", "forall.parameters_implies_equality"),
    ] {
        let (case, text) = step5c11_source_case(family);
        let ast = super::formula_statement::step5c8_test_frontend(&text)
            .ast
            .unwrap();
        let collected =
            super::resolver_symbol_collection(&step5c11_config().workspace_root, &case, &ast);
        let source = SurfaceResolvedArena::lower(&ast, &collected.module).unwrap();
        let (database, inference) = step5c11_check_source(&case, &text).unwrap();
        let (owner, registration) = source
            .arena()
            .iter()
            .find(|(_, node)| {
                matches!(
                    node.kind(),
                    SurfaceNodeKind::ExistentialRegistration
                        | SurfaceNodeKind::ConditionalRegistration
                        | SurfaceNodeKind::FunctorialRegistration
                        | SurfaceNodeKind::ReductionRegistration
                )
            })
            .unwrap();
        let SourceAnchor::Range(registration_range) = registration.origin().anchor() else {
            panic!("source range")
        };
        let (_, obligation) = database.initial_obligations().iter().next().unwrap();
        assert_eq!(
            obligation.owner,
            TypedSiteRef::Node(TypedNodeId::new(owner.index()))
        );
        assert_eq!(obligation.source_range, *registration_range);
        let goal = obligation.goal.as_str();
        assert!(
            goal.contains(&format!("schema={schema};")),
            "{family}: {goal}"
        );
        assert!(goal.contains(&format!("owner={owner:?};")), "{family}");
        let site = |id: mizar_resolve::resolved_ast::ResolvedNodeId| {
            TypedSiteRef::Node(TypedNodeId::new(id.index()))
        };
        let correctness = registration.children().iter().map(|id| source.arena().node(*id).unwrap())
            .find(|node| node.kind() == &SurfaceNodeKind::CorrectnessCondition).unwrap();
        let SourceAnchor::Range(correctness_range) = correctness.origin().anchor() else { panic!("correctness range") };
        let in_registration = |node: &mizar_resolve::resolved_ast::ResolvedNode| {
            matches!(node.origin().anchor(), SourceAnchor::Range(range)
                if range.start >= registration_range.start && range.end <= correctness_range.start)
        };
        let heads = source
            .arena()
            .iter()
            .filter(|(_, node)| node.kind() == &SurfaceNodeKind::TypeHead && in_registration(node))
            .map(|(id, _)| id)
            .collect::<Vec<_>>();
        for head in &heads {
            assert!(
                inference
                    .type_entries()
                    .iter()
                    .any(|(_, entry)| entry.owner == site(*head)),
                "unchecked result/domain type in {family}"
            );
        }
        if family == "existential" || family == "conditional" {
            assert_eq!(heads.len(), 1);
            assert!(goal.contains(&format!("generated={owner:?}:domain={:?}", heads[0])));
        } else {
            let (_, parameter) = source
                .arena()
                .iter()
                .find(|(_, node)| node.kind() == &SurfaceNodeKind::RegistrationParameter)
                .unwrap();
            let segment = source.arena().node(parameter.children()[1]).unwrap();
            let binder = segment.children()[0];
            let ty = source
                .arena()
                .node(segment.children()[2])
                .unwrap()
                .children()[0];
            assert!(
                goal.contains(&format!("parameter={binder:?}:type={ty:?}")),
                "missing parameter guard: {goal}"
            );
            assert!(
                inference
                    .type_entries()
                    .iter()
                    .any(|(_, entry)| entry.owner == site(ty))
            );
        }
        let operands = registration
            .children()
            .iter()
            .filter_map(|id| {
                let node = source.arena().node(*id).unwrap();
                (node.kind() == &SurfaceNodeKind::TermExpression).then(|| site(node.children()[0]))
            })
            .collect::<Vec<_>>();
        for operand in &operands {
            assert!(
                inference
                    .terms()
                    .iter()
                    .any(|(_, term)| &term.site == operand),
                "unchecked operand in {family}"
            );
        }
        if family == "functorial" {
            assert_eq!(operands.len(), 1);
            assert!(
                goal.contains(&format!("result={:?}:type={:?}", operands[0], heads[0])),
                "missing result guard: {goal}"
            );
            assert!(goal.contains(&format!("application={:?}", operands[0])));
        } else if family == "reduce" {
            assert_eq!(operands.len(), 2);
            let lhs = goal
                .find(&format!("equality_operand={:?}", operands[0]))
                .unwrap();
            let rhs = goal
                .find(&format!("equality_operand={:?}", operands[1]))
                .unwrap();
            assert!(lhs < rhs, "reduction operands reversed: {goal}");
        }
        let attributes = source
            .arena()
            .iter()
            .filter(|(_, node)| {
                node.kind() == &SurfaceNodeKind::AttributeRef && in_registration(node)
            })
            .collect::<Vec<_>>();
        assert_eq!(
            attributes.len(),
            match family {
                "conditional" => 2,
                "reduce" => 0,
                _ => 1,
            }
        );
        for (attribute, _) in attributes {
            assert!(
                goal.contains(&format!("attribute={attribute:?}")),
                "missing attribute operand: {goal}"
            );
        }
        for (_, formula) in inference.formulas().iter() {
            assert!(
                goal.contains(&format!("{:?}", formula.site)),
                "missing checked definition body: {goal}"
            );
        }
    }
}
