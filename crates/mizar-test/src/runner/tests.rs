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
        ("existential", "means X = X", "means not X = Y"),
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
    assert_eq!(report.results.len(), 5);
    assert_eq!(report.passed_count(), 5);
}

#[test]
fn step5c11_intake_authenticates_complete_typed_projection_and_symbol_environment() {
    use mizar_checker::registration_resolution::check_source_registration_intake;
    use mizar_checker::typed_ast::{NodeRecoveryState, TypedArena, TypingState};
    use mizar_resolve::resolved_ast::SurfaceResolvedArena;
    let (case, text) = step5c11_source_case("functorial");
    let frontend = super::formula_statement::step5c8_test_frontend(&text);
    let ast = frontend.ast.clone().unwrap();
    let (source, typed, symbols) =
        super::source_registration_inputs(&step5c11_config().workspace_root, &case, frontend)
            .unwrap();
    let raw_nodes = typed
        .iter()
        .map(|(_, node)| node.clone())
        .collect::<Vec<_>>();
    let root = typed.root();
    check_source_registration_intake(&source, &typed, &symbols).unwrap();
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
            check_source_registration_intake(&source, &corrupt, &symbols).is_err(),
            "accepted projection mutation {mutation}"
        );
    }
    let foreign_module =
        ResolverModuleId::new(PackageId::new("foreign"), ModulePath::new("foreign"));
    let foreign_source = SurfaceResolvedArena::lower(&ast, &foreign_module).unwrap();
    assert!(check_source_registration_intake(&foreign_source, &typed, &symbols).is_err());
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

fn step5c11_false_coherence_case() -> (crate::harness::TestCase, String) {
    let plan = build_test_plan(&step5c11_config()).unwrap();
    let case = plan
        .cases
        .into_iter()
        .find(|case| case.id.0 == "fail_proof_verification_functorial_false_coherence_001")
        .unwrap();
    let text = std::fs::read_to_string(&case.source_path).unwrap();
    (case, text)
}

fn step5c11_functorial_core(
    case: &crate::harness::TestCase,
    text: &str,
) -> Result<mizar_core::core_ir::CoreIr, String> {
    let (source, nodes, symbols) = super::source_registration_inputs(
        &step5c11_config().workspace_root,
        case,
        super::formula_statement::step5c8_test_frontend(text),
    )?;
    let checked = mizar_checker::registration_resolution::check_source_registration_intake(&source, &nodes, &symbols)?;
    mizar_core::elaborator::lower_source_functorial_registration(&checked)
}

#[test]
fn step5c11_false_coherence_executes_real_guarded_goal_and_definition_polarity() {
    use mizar_core::core_ir::{CoreFormulaKind as F, DefinitionBody};
    use mizar_vc::discharge::failed_functorial_coherence;
    let (case, text) = step5c11_false_coherence_case();
    for source in [
        text.clone(),
        text.replace("C4Def", "OtherAttribute")
            .replace("CBox3Def", "OtherFunction")
            .replace("CReg4", "OtherRegistration")
            .replace("cbox3", "renamed_box")
            .replace("f4marked", "renamed_mark")
            .replace('X', "Y"),
    ] {
        let core = step5c11_functorial_core(&case, &source).unwrap();
        assert_eq!(core, step5c11_functorial_core(&case, &source).unwrap());
        assert_eq!(core.definitions().len(), 2);
        let goal = core
            .obligation_seeds()
            .iter()
            .next()
            .unwrap()
            .1
            .goal
            .unwrap();
        let F::Forall { binders, body } = &core.formulas().get(goal).unwrap().kind else {
            panic!("full quantifier missing")
        };
        assert_eq!(binders.len(), 1);
        let F::Implies {
            premise,
            conclusion,
        } = core.formulas().get(*body).unwrap().kind
        else {
            panic!("guard implication missing")
        };
        let F::And(ref guards) = core.formulas().get(premise).unwrap().kind else {
            panic!("both guards missing")
        };
        assert_eq!(guards.len(), 2);
        assert!(guards.iter().all(|id| matches!(&core.formulas().get(*id).unwrap().kind, F::TypePred { ty, .. } if ty.as_str() == "set")));
        let F::TypePred {
            subject: parameter, ..
        } = core.formulas().get(guards[0]).unwrap().kind
        else {
            unreachable!()
        };
        let F::TypePred {
            subject: application,
            ..
        } = core.formulas().get(guards[1]).unwrap().kind
        else {
            unreachable!()
        };
        assert_eq!(
            core.terms().get(parameter).unwrap().kind,
            mizar_core::core_ir::CoreTermKind::Var(binders[0].var)
        );
        let mizar_core::core_ir::CoreTermKind::Apply { ref args, .. } =
            core.terms().get(application).unwrap().kind
        else {
            panic!("result guard lost application")
        };
        assert_eq!(args, &[parameter]);
        let F::Atom { ref args, .. } = core.formulas().get(conclusion).unwrap().kind else {
            panic!("consequent lost attribute")
        };
        assert_eq!(args, &[application]);
        assert!(core.definitions().iter().any(|(_, def)| matches!(def.body, DefinitionBody::Formula(id) if matches!(core.formulas().get(id).unwrap().kind, F::Not(_)))));
        let vcs =
            super::proof_verification::generate_core_vcs(&core, super::shared::snapshot_id(5800))
                .unwrap();
        assert_eq!(
            vcs,
            super::proof_verification::generate_core_vcs(&core, super::shared::snapshot_id(5800))
                .unwrap()
        );
        assert!(core.obligation_seeds().iter().all(|(_, seed)| seed.label.is_none()));
        assert!(vcs.vcs()[0].premises.is_empty());
        assert!(vcs.vcs()[0].proof_hint.is_none());
        assert_eq!(
            failed_functorial_coherence(&core, &vcs).unwrap(),
            Some(vcs.vcs()[0].id)
        );
        let (database, inference) = step5c11_check_source(&case, &source).unwrap();
        assert!(database.activated().is_empty() && database.rejected().is_empty());
        assert!(
            database
                .pending()
                .iter()
                .all(|row| !row.may_contribute_to_inference())
        );
        assert!(
            inference
                .formulas()
                .iter()
                .any(|(_, formula)| formula.kind
                    == mizar_checker::type_checker::FormulaKind::Negation)
        );
    }
    let positive = step5c11_functorial_core(&case, &text.replace("not X = X", "X = X")).unwrap();
    let vcs =
        super::proof_verification::generate_core_vcs(&positive, super::shared::snapshot_id(5800))
            .unwrap();
    assert_eq!(failed_functorial_coherence(&positive, &vcs).unwrap(), None);
    let report = super::run_proof_verification_corpus(&step5c11_config()).unwrap();
    assert_eq!(
        report.failed_count() + report.error_count(),
        0,
        "{report:?}"
    );
    assert!(report.results.iter().any(|row| row.id == case.id));
}

#[test]
fn step5c11_false_coherence_rejects_unsupported_source_proofs_and_operands() {
    let (case, text) = step5c11_false_coherence_case();
    for (before, after) in [
        ("thus thesis;", "thus contradiction;"),
        ("thus thesis;", "hence thesis;"),
        ("thus thesis;", "assume X = X; thus thesis;"),
        ("thus thesis;", "thus thesis by Missing;"),
        ("coherence;", "coherence proof thus thesis; end;"),
        ("CReg4: cbox3 X", "CReg4: cbox3 Y"),
        ("not X = X", "not X = Y"),
        ("not X = X", "not not X = X"),
        ("for set", "for object"),
        ("equals X", "equals {X}"),
        ("cbox3 X -> set", "cbox3 X -> object"),
    ] {
        assert!(text.contains(before));
        assert!(
            step5c11_functorial_core(&case, &text.replace(before, after)).is_err(),
            "ignored {before} -> {after}"
        );
    }
}

#[test]
fn step5c11_false_coherence_consumer_rejects_core_and_vc_corruption() {
    use mizar_core::core_ir::*;
    use mizar_vc::discharge::failed_functorial_coherence;
    let (case, text) = step5c11_false_coherence_case();
    let core = step5c11_functorial_core(&case, &text).unwrap();
    let original_vcs =
        super::proof_verification::generate_core_vcs(&core, super::shared::snapshot_id(5800))
            .unwrap();
    for mutation in 0..14 {
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
        let (seed_id, seed) = core.obligation_seeds().iter().next().unwrap();
        let goal = seed.goal.unwrap();
        let definition = core
            .definitions()
            .iter()
            .find(|(_, d)| matches!(d.body, DefinitionBody::Formula(_)))
            .unwrap()
            .0;
        match mutation {
            0 => {
                // Change the actual attribute body while preserving a well-formed Core graph.
                let DefinitionBody::Formula(body) = parts.definitions.get(definition).unwrap().body
                else {
                    unreachable!()
                };
                let CoreFormulaKind::Not(inner) = parts.formulas.get(body).unwrap().kind else {
                    unreachable!()
                };
                parts.definitions.get_mut(definition).unwrap().body =
                    DefinitionBody::Formula(inner);
            }
            1 => {
                let id = core
                    .formulas()
                    .iter()
                    .find(|(_, f)| matches!(f.kind, CoreFormulaKind::TypePred { .. }))
                    .unwrap()
                    .0;
                let CoreFormulaKind::TypePred { ref mut ty, .. } =
                    parts.formulas.get_mut(id).unwrap().kind
                else {
                    unreachable!()
                };
                *ty = "object".into();
            }
            2 => parts.formulas.get_mut(goal).unwrap().kind = CoreFormulaKind::False,
            3 => parts
                .obligation_seeds
                .get_mut(seed_id)
                .unwrap()
                .context
                .push(goal),
            4 => {
                let functor = core
                    .definitions()
                    .iter()
                    .find(|(_, d)| matches!(d.body, DefinitionBody::Term(_)))
                    .unwrap()
                    .1;
                parts.definitions.get_mut(definition).unwrap().symbol = functor.symbol.clone();
            }
            5 => {
                let id = core
                    .terms()
                    .iter()
                    .find(|(_, t)| matches!(t.kind, CoreTermKind::Apply { .. }))
                    .unwrap()
                    .0;
                let CoreTermKind::Apply { ref mut args, .. } =
                    parts.terms.get_mut(id).unwrap().kind
                else {
                    unreachable!()
                };
                args.clear();
            }
            6 => parts
                .obligation_seeds
                .get_mut(seed_id)
                .unwrap()
                .provenance
                .clear(),
            7 => {
                let CoreFormulaKind::Forall { body, .. } = parts.formulas.get(goal).unwrap().kind
                else {
                    unreachable!()
                };
                let CoreFormulaKind::Implies { premise, .. } =
                    parts.formulas.get(body).unwrap().kind
                else {
                    unreachable!()
                };
                let CoreFormulaKind::And(ref guards) = parts.formulas.get(premise).unwrap().kind
                else {
                    unreachable!()
                };
                let result = guards[1];
                let CoreFormulaKind::TypePred { ref mut ty, .. } =
                    parts.formulas.get_mut(result).unwrap().kind
                else {
                    unreachable!()
                };
                *ty = "object".into();
            }
            8 => {
                let id = core.proofs().iter().next().unwrap().0;
                parts.proofs.get_mut(id).unwrap().item = seed.owner;
            }
            9 => {
                let id = core.proofs().iter().next().unwrap().0;
                parts.proofs.get_mut(id).unwrap().proposition = goal;
            }
            10 => {
                let (id, proof) = core.proofs().iter().next().unwrap();
                parts.proofs.get_mut(id).unwrap().source = core.items().get(proof.item).unwrap().source.clone();
            }
            11 => {
                let root = core.proofs().iter().next().unwrap().1.root;
                let CoreProofNodeKind::Step { ref mut justification, .. } = parts.proof_nodes.get_mut(root).unwrap().kind else { unreachable!() };
                justification.citations.push(CoreCitation::Label("injected".into()));
            }
            12 => {
                parts.proofs = CoreProofTable::new();
                parts.proof_nodes = CoreProofNodeTable::new();
                parts.source_map.proof_sources.clear();
            }
            13 => {
                parts.obligation_seeds.get_mut(seed_id).unwrap().label =
                    Some(CoreLabelRef::new(core.items().get(seed.owner).unwrap().symbol.fqn().as_str()));
            }
            _ => unreachable!(),
        }
        let changed = CoreIr::try_new(parts).unwrap();
        let generated = super::proof_verification::generate_core_vcs(
            &changed,
            super::shared::snapshot_id(5800),
        );
        if let Ok(vcs) = generated {
            let outcome = failed_functorial_coherence(&changed, &vcs);
            if mutation == 0 {
                assert_eq!(outcome.unwrap(), None);
            } else {
                assert!(outcome.is_err(), "accepted Core mutation {mutation}");
            }
        } else {
            assert!(!matches!(mutation, 0 | 7..=13), "valid modified Core must generate VCs");
        }
        if mutation != 0 {
            assert!(failed_functorial_coherence(&changed, &original_vcs).is_err());
        }
    }
    for mutation in 0..3 {
        use mizar_vc::vc_ir::{SeedOriginRef, VcFormulaRef, VcSet, VcSetParts};
        let mut parts = VcSetParts {
            schema_version: original_vcs.schema_version().clone(),
            snapshot: original_vcs.snapshot(),
            source: original_vcs.source(),
            module: original_vcs.module().clone(),
            generated_formulas: original_vcs.generated_formulas().to_vec(),
            vcs: original_vcs.vcs().to_vec(),
            seed_accounting: original_vcs.seed_accounting().to_vec(),
        };
        match mutation {
            0 => {
                parts.vcs[0].goal = VcFormulaRef::Core(
                    core.formulas()
                        .iter()
                        .find(|(_, f)| matches!(f.kind, CoreFormulaKind::Equals { .. }))
                        .unwrap()
                        .0,
                )
            }
            1 => {
                parts.seed_accounting[0].origin = SeedOriginRef::ExistingCore {
                    seed: ObligationSeedId::new(99),
                }
            }
            2 => {
                parts.vcs[0].source.primary =
                    core.definitions().iter().next().unwrap().1.source.clone()
            }
            _ => unreachable!(),
        }
        let corrupt = VcSet::try_new(parts).unwrap();
        assert!(
            failed_functorial_coherence(&core, &corrupt).is_err(),
            "accepted VC mutation {mutation}"
        );
    }
    let positive = step5c11_functorial_core(&case, &text.replace("not X = X", "X = X")).unwrap();
    let unrelated_vcs =
        super::proof_verification::generate_core_vcs(&positive, super::shared::snapshot_id(5800))
            .unwrap();
    assert!(failed_functorial_coherence(&core, &unrelated_vcs).is_err());
}

#[test]
fn step5c11_false_coherence_admission_is_exact_and_cannot_fall_back() {
    use crate::{
        expectation::{ExpectedOutcome, PipelinePhase},
        staged_model::Stage,
    };
    for (original, _) in [step5c11_false_coherence_case(), step5c11_reduce_case()] {
        let config = step5c11_config();
        assert!(super::is_active_proof_verification(&original));
        for mutation in 0..27 {
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
                2 => {
                    case.expectation_path =
                        case.expectation_path.with_file_name("wrong.expect.toml")
                }
                3 => case.expectation.expected_phase = Some(PipelinePhase::VcGeneration),
                4 => case.expectation.expected_outcome = ExpectedOutcome::Pass,
                5 => case.expectation.tags.clear(),
                6 => case.expectation.tags.push("extra".into()),
                7 => case.expectation.stable_detail_key = Some("wrong".into()),
                8 => case.expectation.failure_category = None,
                9 => case.expectation.diagnostic_codes.push("E-UNRELATED".into()),
                10 => case.expectation.rejection_reason = Some("wrong".into()),
                11 => case.expectation.snapshots = Some("wrong".into()),
                12 => case
                    .expectation
                    .declaration_symbol_payloads
                    .push("wrong".into()),
                13 => case.expectation.id.0.push_str("_alias"),
                14 => case.expectation.domain.push_str("_wrong"),
                15 => case.expectation.spec_refs[0].0.push_str("_wrong"),
                16 => case
                    .expectation
                    .spec_refs
                    .push(case.expectation.spec_refs[0].clone()),
                17 => {
                    case.expectation.spec_refs.pop();
                }
                18 => case.expectation.snapshots = None,
                19 => case.expectation.kind = crate::expectation::TestKind::Pass,
                20 => case.expectation.source = "wrong.miz".into(),
                21 => case.expectation.schema_version = 2,
                22 => case.expectation.profiles.push("extra".into()),
                23 => case.expectation.ast_profile = Some("wrong".into()),
                24 => case.expectation.snapshot_profiles.push("wrong".into()),
                25 => case.expectation.diagnostic_payloads.push("wrong".into()),
                26 => {
                    if case.expectation.snapshots.is_none() {
                        continue;
                    }
                    case.expectation.spec_refs.reverse();
                }
                _ => unreachable!(),
            }
            if mutation == 18 && original.expectation.snapshots.is_none() {
                continue;
            }
            assert!(
                !super::proof_verification::step5c11_proof_admitted(
                    Some(&config.workspace_root),
                    &case
                ),
                "mutation {mutation}"
            );
            if original.expectation.snapshots.is_some() && !matches!(mutation, 0 | 1 | 18 | 21..=24)
            {
                assert!(
                    crate::expectation::validate_expectation_path(
                        &case.expectation_path,
                        &case.expectation,
                        &config.workspace_root.join("tests")
                    )
                    .iter()
                    .any(|diagnostic| diagnostic.code.0 == "E-EXPECT-SNAPSHOT-SCOPE"),
                    "reduction snapshot scope mutation {mutation}"
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
            let mut case = original.clone();
            case.expectation.stage = stage;
            case.expectation.expected_phase = Some(phase);
            case.expectation.tags = vec![tag.into()];
            assert!(!super::is_active_parse_only(&case));
            assert!(!super::is_active_declaration_symbol(&case));
            assert!(!super::is_active_type_elaboration(&case));
            assert!(!super::formula_statement::is_active_formula_statement(
                &config.workspace_root,
                &case
            ));
            assert!(!super::is_active_proof_verification(&case));
            assert!(!super::step5c11_registration_admitted(
                &config.workspace_root,
                &case
            ));
            case.id.0 = "corrupt_identity".into();
            case.source_path = config.workspace_root.join("alias.miz");
            case.expectation_path = config.workspace_root.join("alias.expect.toml");
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
            super::proof_verification::validate_active_proof_verification_tags(
                &config.workspace_root,
                &plan
            )
            .is_empty()
        );
        for duplicate in [false, true] {
            let mut changed = plan.clone();
            if duplicate {
                changed.cases.push(original.clone());
            } else {
                changed.cases.retain(|case| case.id != original.id);
            }
            assert!(
                !super::proof_verification::validate_active_proof_verification_tags(
                    &config.workspace_root,
                    &changed
                )
                .is_empty()
            );
        }
        let temporary = std::process::Command::new("mktemp")
            .arg("-d")
            .output()
            .unwrap();
        assert!(temporary.status.success());
        let root = PathBuf::from(String::from_utf8(temporary.stdout).unwrap().trim());
        let mut absent = plan.clone();
        absent.cases.clear();
        assert!(
            super::proof_verification::validate_active_proof_verification_tags(&root, &absent)
                .is_empty()
        );
        std::fs::create_dir_all(root.join("tests/coverage")).unwrap();
        std::fs::copy(
            config
                .workspace_root
                .join("tests/coverage/step5_activation_map.tsv"),
            root.join("tests/coverage/step5_activation_map.tsv"),
        )
        .unwrap();
        assert!(
            super::proof_verification::validate_active_proof_verification_tags(&root, &absent)
                .iter()
                .any(|diagnostic| diagnostic.detail_key == "proof_verification.step5c11_inventory")
        );
        std::fs::remove_dir_all(root).unwrap();
    }
}

fn step5c13_case() -> crate::harness::TestCase {
    build_test_plan(&step5c11_config())
        .unwrap()
        .cases
        .into_iter()
        .find(|case| case.id.0 == super::STEP5C13_OVERLOAD_IDS[0])
        .unwrap()
}

fn step5c13_inputs(
    text: &str,
) -> Result<
    (
        mizar_resolve::resolved_ast::SurfaceResolvedArena,
        mizar_checker::typed_ast::TypedArena,
        mizar_resolve::env::SymbolEnv,
    ),
    String,
> {
    super::source_registration_inputs(
        &step5c11_config().workspace_root,
        &step5c13_case(),
        super::formula_statement::step5c8_test_frontend(text),
    )
}

#[test]
fn step5c13_real_source_selects_from_both_roots_at_both_sites_by_actual_type() {
    use mizar_checker::overload_resolution::{
        CandidateViabilityStatus, ExposedResultSource, OverloadResultStatus,
    };
    use mizar_checker::type_checker::{TypeHeadRef, check_source_distinct_loci_overloads};
    let case = step5c13_case();
    let text = std::fs::read_to_string(&case.source_path).unwrap();
    for (source_text, structure) in [
        (text.clone(), false),
        (
            text.replace("ovbox", "other_box")
                .replace("OvBox", "OtherBox")
                .replace("X", "A")
                .replace("B be", "C be")
                .replace("other_box B", "other_box C")
                .replace("B.d", "C.d")
                .replace(" d ", " value ")
                .replace(".d", ".value"),
            false,
        ),
        (
            text.replace("for X being set", "for X being OvBox")
                .replace("  let X be set;\n  thus", "  let X be OvBox;\n  thus")
                .replace("ovbox X = X", "ovbox X = X.d"),
            true,
        ),
    ] {
        let (source, typed, symbols) = step5c13_inputs(&source_text).unwrap();
        let outputs =
            check_source_distinct_loci_overloads(&source, &symbols, &typed, false).unwrap();
        assert_eq!(
            outputs,
            check_source_distinct_loci_overloads(&source, &symbols, &typed, false).unwrap()
        );
        let (normalization, collection, expansion, viability, graphs, selection) = outputs;
        assert_eq!(collection.sites().len(), 2);
        assert_eq!(collection.candidates().len(), 4);
        assert_eq!(expansion.candidates().len(), 4);
        assert_eq!(viability.decisions().len(), 4);
        assert_eq!(
            viability
                .decisions()
                .iter()
                .filter(|(_, row)| matches!(row.status, CandidateViabilityStatus::Viable { .. }))
                .count(),
            2
        );
        assert_eq!(
            viability
                .decisions()
                .iter()
                .filter(|(_, row)| matches!(row.status, CandidateViabilityStatus::Rejected { .. }))
                .count(),
            2
        );
        assert_eq!(graphs.graphs().len(), 2);
        assert_eq!(selection.results().len(), 2);
        assert!(selection.inserted_views().is_empty());
        let mut declarations = symbols
            .symbols()
            .iter()
            .filter(|entry| entry.kind() == mizar_resolve::env::SymbolKind::Functor)
            .collect::<Vec<_>>();
        declarations.sort_by_key(|entry| match entry.origin().anchor() {
            mizar_session::SourceAnchor::Range(r) => r.start,
            _ => panic!("missing range"),
        });
        assert_eq!(declarations.len(), 2);
        let structure_symbol = symbols
            .symbols()
            .iter()
            .find(|entry| entry.kind() == mizar_resolve::env::SymbolKind::Structure)
            .unwrap()
            .symbol();
        let applications = source
            .arena()
            .iter()
            .filter(|(_, node)| {
                matches!(
                    node.kind(),
                    mizar_syntax::ast::SurfaceNodeKind::PrefixExpression(_)
                )
            })
            .collect::<Vec<_>>();
        assert_eq!(applications.len(), 2);
        for (index, (_, output)) in collection.sites().iter().enumerate() {
            let (id, application) = applications[index];
            assert_eq!(
                output.owner,
                mizar_checker::typed_ast::TypedSiteRef::Node(
                    mizar_checker::typed_ast::TypedNodeId::new(id.index())
                )
            );
            assert_eq!(
                mizar_session::SourceAnchor::Range(output.source_range),
                *application.origin().anchor()
            );
            assert_eq!(
                output.arguments,
                vec![mizar_checker::typed_ast::TypedSiteRef::Node(
                    mizar_checker::typed_ast::TypedNodeId::new(application.children()[1].index())
                )]
            );
        }
        for (id, site) in collection.sites().iter() {
            assert!(site.expected.is_none());
            let candidates = collection
                .candidates()
                .iter()
                .filter(|(_, row)| row.site == id)
                .map(|(_, row)| row)
                .collect::<Vec<_>>();
            assert_eq!(candidates.len(), 2);
            assert_ne!(candidates[0].ordinary_root, candidates[1].ordinary_root);
            assert_ne!(candidates[0].parameters, candidates[1].parameters);
            for candidate in candidates {
                assert_eq!(candidate.symbol, candidate.ordinary_root);
                assert!(candidate.coherence.is_none());
                assert!(candidate.template.is_none());
                let declaration = declarations[candidate.provenance.declaration_order];
                assert_eq!(&candidate.symbol, declaration.symbol());
                assert_eq!(
                    mizar_session::SourceAnchor::Range(candidate.provenance.source_range.unwrap()),
                    *declaration.origin().anchor()
                );
                assert_eq!(candidate.parameters.len(), 1);
                let expected = if candidate.provenance.declaration_order == 0 {
                    TypeHeadRef::BuiltinSet
                } else {
                    TypeHeadRef::Structure(structure_symbol.clone())
                };
                assert_eq!(
                    normalization
                        .normalized_types()
                        .get(candidate.parameters[0])
                        .unwrap()
                        .head,
                    expected
                );
                assert_eq!(
                    normalization
                        .normalized_types()
                        .get(candidate.result.unwrap())
                        .unwrap()
                        .head,
                    TypeHeadRef::BuiltinSet
                );
                assert!(candidate.provenance.source_range.unwrap().end <= site.source_range.start);
            }
        }
        for (_, result) in selection.results().iter() {
            let OverloadResultStatus::Resolved {
                root,
                exposed_result: Some(exposed),
                refinements,
                inserted_views,
            } = &result.status
            else {
                panic!("{:?}", result.status)
            };
            let selected = graphs.candidates().get(*root).unwrap();
            assert_eq!(
                &selected.symbol,
                declarations[usize::from(structure)].symbol()
            );
            assert!(
                refinements.is_empty() && inserted_views.is_empty() && exposed.evidence.is_empty()
            );
            assert_eq!(exposed.source, ExposedResultSource::SelectedRoot);
            assert_eq!(exposed.result, selected.result);
            assert_eq!(
                normalization
                    .normalized_types()
                    .get(exposed.result.unwrap())
                    .unwrap()
                    .head,
                TypeHeadRef::BuiltinSet
            );
            let parameter = &normalization
                .normalized_types()
                .get(selected.parameters[0])
                .unwrap()
                .head;
            if structure {
                assert!(matches!(parameter, TypeHeadRef::Structure(_)));
            } else {
                assert_eq!(parameter, &TypeHeadRef::BuiltinSet);
            }
        }
    }
    let report = super::run_advanced_semantics_corpus(&step5c11_config()).unwrap();
    assert_eq!(report.error_count(), 0, "{:?}", report.diagnostics);
    assert_eq!(report.results.len(), 5);
    assert_eq!(report.passed_count(), 5);
}

#[test]
fn step5c13_rejects_source_signature_selector_binding_and_order_corruption() {
    use mizar_checker::type_checker::check_source_distinct_loci_overloads;
    let text = std::fs::read_to_string(step5c13_case().source_path).unwrap();
    for (before, after) in [
        (
            "func Ov1Def: ovbox X -> set equals X",
            "func Ov1Def: ovbox X -> OvBox equals X",
        ),
        ("let B be OvBox", "let B be set"),
        ("field d -> set", "field d -> object"),
        ("equals B.d", "equals B.missing"),
        ("equals B.d", "equals X.d"),
        ("equals X;", "equals Y;"),
        ("ovbox B -> set", "ovbox X -> set"),
        ("let X be set;\n  thus", "let Y be set;\n  thus"),
        ("thus ovbox X = X", "thus ovbox Y = X"),
        ("holds ovbox X = X", "holds ovbox Y = X"),
        ("thus ovbox X = X", "thus X = X"),
        ("coherence;", "existence;"),
        ("func Ov2Def:", "redefine func Ov2Def:"),
    ] {
        assert!(text.contains(before));
        let input = step5c13_inputs(&text.replace(before, after));
        assert!(
            input
                .and_then(
                    |(source, typed, symbols)| check_source_distinct_loci_overloads(
                        &source, &symbols, &typed, false
                    )
                )
                .is_err(),
            "{before} -> {after}"
        );
    }
    let split = text.find("theorem OvUse1:").unwrap();
    let first_end = text.find("end;").unwrap() + 4;
    for changed in [
        format!("{}\n{}", &text[split..], &text[..split]),
        format!("{}\n{}", &text[..first_end], text),
        text[first_end..].to_owned(),
    ] {
        assert!(
            step5c13_inputs(&changed)
                .and_then(|(s, t, e)| check_source_distinct_loci_overloads(&s, &e, &t, false))
                .is_err()
        );
    }
}

#[test]
fn step5c13_authenticates_complete_source_environment_and_neutral_projection() {
    use mizar_checker::type_checker::check_source_distinct_loci_overloads;
    use mizar_checker::typed_ast::{NodeRecoveryState, TypedArena, TypingState};
    use mizar_resolve::resolved_ast::SurfaceResolvedArena;
    let case = step5c13_case();
    let text = std::fs::read_to_string(&case.source_path).unwrap();
    let frontend = super::formula_statement::step5c8_test_frontend(&text);
    let ast = frontend.ast.clone().unwrap();
    let (source, typed, symbols) =
        super::source_registration_inputs(&step5c11_config().workspace_root, &case, frontend)
            .unwrap();
    check_source_distinct_loci_overloads(&source, &symbols, &typed, false).unwrap();
    let raw = typed
        .iter()
        .map(|(_, node)| node.clone())
        .collect::<Vec<_>>();
    for mutation in 0..7 {
        let mut nodes = raw.clone();
        let mut root = typed.root();
        match mutation {
            0 => nodes[0].kind = "Unrelated".into(),
            1 => nodes[0].resolved_node = nodes[1].resolved_node,
            2 => nodes[0].anchor = nodes[1].anchor.clone(),
            3 => nodes[0].recovery = NodeRecoveryState::Recovered,
            4 => nodes[0].typing = TypingState::Successful,
            5 => nodes.last_mut().unwrap().children.clear(),
            6 => root = None,
            _ => unreachable!(),
        }
        let changed = TypedArena::try_new(root, nodes).unwrap();
        assert!(
            check_source_distinct_loci_overloads(&source, &symbols, &changed, false).is_err(),
            "mutation {mutation}"
        );
    }
    let foreign = ResolverModuleId::new(PackageId::new("other"), ModulePath::new("other"));
    assert!(
        check_source_distinct_loci_overloads(
            &SurfaceResolvedArena::lower(&ast, &foreign).unwrap(),
            &symbols,
            &typed,
            false
        )
        .is_err()
    );
    let (_, _, changed_env) = step5c13_inputs(&text.replace("ovbox", "otherbox")).unwrap();
    assert!(check_source_distinct_loci_overloads(&source, &changed_env, &typed, false).is_err());
}

#[test]
fn step5c13_admission_reserves_both_rows_and_all_stage_aliases() {
    let config = step5c11_config();
    let original = step5c13_case();
    assert!(super::step5c13_overload_admitted(
        &config.workspace_root,
        &original
    ));
    for mutation in 0..14 {
        let mut case = original.clone();
        match mutation {
            0 => case.id.0.push_str("_extra"),
            1 => case.expectation.id.0.push_str("_extra"),
            2 => case.source_path = case.source_path.with_file_name("wrong.miz"),
            3 => case.expectation_path = case.expectation_path.with_file_name("wrong.expect.toml"),
            4 => case.expectation.source = PathBuf::from("wrong.miz"),
            5 => case.expectation.stage = crate::staged_model::Stage::TypeElaboration,
            6 => {
                case.expectation.expected_phase = Some(crate::expectation::PipelinePhase::TypeCheck)
            }
            7 => case.expectation.expected_outcome = crate::expectation::ExpectedOutcome::Fail,
            8 => case.expectation.tags.clear(),
            9 => case.expectation.tags.push("extra".into()),
            10 => case.expectation.diagnostic_codes.push("E-UNRELATED".into()),
            11 => case.expectation.stable_detail_key = Some("wrong.detail".into()),
            12 => case.expectation.kind = crate::expectation::TestKind::Fail,
            13 => case.expectation.rejection_reason = Some("wrong.reason".into()),
            _ => unreachable!(),
        }
        assert!(
            !super::step5c13_overload_admitted(&config.workspace_root, &case),
            "mutation {mutation}"
        );
    }
    let plan = build_test_plan(&config).unwrap();
    let negative = plan
        .cases
        .iter()
        .find(|case| case.id.0 == super::STEP5C13_OVERLOAD_IDS[1])
        .unwrap();
    assert!(negative.expectation.tags.is_empty());
    for source_case in [&original, negative] {
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
            for alias in [false, true] {
                let mut case = source_case.clone();
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
                assert!(!super::is_active_parse_only(&case));
                assert!(!super::is_active_declaration_symbol(&case));
                assert!(!super::is_active_type_elaboration(&case));
                assert!(!super::formula_statement::is_active_formula_statement(
                    &config.workspace_root,
                    &case
                ));
                assert!(!super::is_active_proof_verification(&case));
                assert!(!super::step5c13_overload_admitted(
                    &config.workspace_root,
                    &case
                ));
            }
        }
    }
    assert!(
        super::validate_step5c11_registration_inventory(&config.workspace_root, &plan).is_empty()
    );
    let mut missing = plan.clone();
    missing.cases.retain(|case| case.id != original.id);
    let mut duplicate = plan;
    duplicate.cases.push(original);
    for invalid in [missing, duplicate] {
        assert!(
            super::validate_step5c11_registration_inventory(&config.workspace_root, &invalid)
                .iter()
                .any(|d| d.code.0 == "E-ADVANCED-SEMANTICS-INVENTORY")
        );
    }
}

#[test]
fn step5c13_checks_each_coherently_corrupted_source_callee() {
    use mizar_resolve::resolved_ast::SurfaceResolvedArena;
    use mizar_syntax::ast::{SurfaceAstBuilder, SurfaceNodeKind as K};
    let case = step5c13_case();
    let text = std::fs::read_to_string(&case.source_path).unwrap();
    let ast = super::formula_statement::step5c8_test_frontend(&text)
        .ast
        .unwrap();
    for target in [2, 3] {
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
                    let changed = token.text.as_ref() == "ovbox" && {
                        occurrence += 1;
                        occurrence - 1 == target
                    };
                    if changed {
                        changed_callee = Some(index);
                    }
                    builder.add_token(
                        token.kind,
                        if changed {
                            "wrong".into()
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
                    operator.spelling = "wrong".into();
                    builder.add_node(K::PrefixExpression(operator), node.range, children)
                }
                kind => builder.add_node(kind.clone(), node.range, children),
            };
            rebuilt.push(id);
        }
        assert_eq!(occurrence, 4);
        let changed = builder.finish(Some(rebuilt[ast.root().unwrap().index()]), None);
        let env =
            super::resolver_symbol_collection(&step5c11_config().workspace_root, &case, &changed);
        assert!(env.detail_keys.is_empty());
        let resolved = SurfaceResolvedArena::lower(&changed, &env.module).unwrap();
        mizar_resolve::symbols::validate_source_symbol_env(&resolved, &env.env).unwrap();
        let typed = mizar_checker::typed_ast::TypedArena::try_new(
            Some(mizar_checker::typed_ast::TypedNodeId::new(
                resolved.arena().root().index(),
            )),
            resolved
                .arena()
                .iter()
                .map(|(id, node)| {
                    mizar_checker::typed_ast::TypedNode::new(
                        format!("{:?}", node.kind()),
                        node.origin().anchor().clone(),
                    )
                    .with_resolved_node(id)
                    .with_children(
                        node.children()
                            .iter()
                            .map(|child| mizar_checker::typed_ast::TypedNodeId::new(child.index()))
                            .collect(),
                    )
                })
                .collect(),
        )
        .unwrap();
        assert!(
            mizar_checker::type_checker::check_source_distinct_loci_overloads(
                &resolved, &env.env, &typed, false
            )
            .is_err(),
            "callee {target}"
        );
    }
}

fn step5c14_static_case(name: &str) -> crate::harness::TestCase {
    let id = format!("fail_type_elaboration_algorithm_{name}_001");
    build_test_plan(&step5c11_config())
        .unwrap()
        .cases
        .into_iter()
        .find(|case| case.id.0 == id)
        .unwrap()
}

fn step5c14_static_core(
    case: &crate::harness::TestCase,
    text: &str,
) -> Result<mizar_core::core_ir::CoreIr, String> {
    let (source, typed, symbols) = super::source_registration_inputs(
        &step5c11_config().workspace_root,
        case,
        super::formula_statement::step5c8_test_frontend(text),
    )?;
    let checked =
        mizar_checker::type_checker::check_source_algorithm_types(&source, &typed, &symbols)?;
    mizar_core::elaborator::lower_source_algorithms(&checked)
}

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
            let (source, typed, symbols) = super::source_registration_inputs(
                &config.workspace_root,
                &case,
                super::formula_statement::step5c8_test_frontend(source_text),
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
            let keys = super::type_elaboration_detail_keys(
                &config.workspace_root,
                &case,
                super::formula_statement::step5c8_test_frontend(source_text),
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
        let keys = super::type_elaboration_detail_keys(
            &step5c11_config().workspace_root,
            &case,
            super::formula_statement::step5c8_test_frontend(&source),
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
        super::type_elaboration_detail_keys(
            &step5c11_config().workspace_root,
            &repaired_case,
            super::formula_statement::step5c8_test_frontend(&repaired),
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
        let keys = super::type_elaboration_detail_keys(
            &step5c11_config().workspace_root,
            &case,
            super::formula_statement::step5c8_test_frontend(&text.replace(from, to)),
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
    let frontend = super::formula_statement::step5c8_test_frontend(&text);
    let ast = frontend.ast.clone().unwrap();
    let (source, typed, symbols) =
        super::source_registration_inputs(&step5c11_config().workspace_root, &case, frontend)
            .unwrap();
    check_source_algorithm_types(&source, &typed, &symbols).unwrap();
    let raw = typed
        .iter()
        .map(|(_, node)| node.clone())
        .collect::<Vec<_>>();
    for mutation in 0..7 {
        let mut nodes = raw.clone();
        let mut root = typed.root();
        match mutation {
            0 => nodes[0].kind = "Unrelated".into(),
            1 => nodes[0].resolved_node = nodes[1].resolved_node,
            2 => nodes[0].anchor = nodes[1].anchor.clone(),
            3 => nodes[0].recovery = NodeRecoveryState::Recovered,
            4 => nodes[0].typing = TypingState::Successful,
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
    let changed = super::formula_statement::step5c8_test_frontend(
        &text.replace("ghostleak", "other_algorithm"),
    )
    .ast
    .unwrap();
    let wrong =
        super::resolver_symbol_collection(&step5c11_config().workspace_root, &case, &changed);
    assert!(check_source_algorithm_types(&source, &typed, &wrong.env).is_err());
    assert!(step5c14_static_core(&case, &format!("{text}\n{text}")).is_err());
}

#[test]
fn step5c14_static_admission_is_exact_and_has_no_other_stage_credit() {
    let config = step5c11_config();
    let plan = build_test_plan(&config).unwrap();
    for name in ["break_outside_loop", "ghost_isolation"] {
        let original = step5c14_static_case(name);
        assert!(super::step5c14_static_admitted(
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
                !super::step5c14_static_admitted(Some(&config.workspace_root), &case),
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
                assert!(!super::is_active_parse_only(&case));
                assert!(!super::is_active_declaration_symbol(&case));
                assert!(!super::is_active_type_elaboration(&case));
                assert!(!super::formula_statement::is_active_formula_statement(
                    &config.workspace_root,
                    &case
                ));
                assert!(!super::is_active_proof_verification(&case));
                assert!(!super::step5c11_registration_admitted(
                    &config.workspace_root,
                    &case
                ));
                assert!(!super::step5c13_overload_admitted(
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
                super::validate_active_type_elaboration_tags(&config.workspace_root, &invalid)
                    .iter()
                    .any(|d| d.code.0 == "E-TYPE-ELABORATION-STEP5C14-INVENTORY")
            );
        }
    }
    assert!(super::validate_active_type_elaboration_tags(&config.workspace_root, &plan).is_empty());
}

fn step5c14_return_case() -> crate::harness::TestCase {
    build_test_plan(&step5c11_config())
        .unwrap()
        .cases
        .into_iter()
        .find(|case| case.id.0 == "pass_proof_verification_algorithm_ensures_return_001")
        .unwrap()
}

fn step5c14_return_vcs(
    core: &mizar_core::core_ir::CoreIr,
) -> Result<mizar_vc::vc_ir::VcSet, String> {
    mizar_vc::generator::generate_source_algorithm_postconditions(
        core,
        super::shared::snapshot_id(0),
        &mizar_vc::vc_ir::GenerationSchemaVersion::new("mizar-vc-generation-step5c14-return-v1"),
        &mizar_vc::vc_ir::VcSchemaVersion::new("mizar-vc-vcset-step5c14-return-v1"),
    )
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
        let (resolved, typed, symbols) = super::source_registration_inputs(
            &step5c11_config().workspace_root,
            &case,
            super::formula_statement::step5c8_test_frontend(&source),
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
    for original in [step5c14_return_case(), step5c14_state_case(), step5c14_claim_case()] {
        assert!(super::proof_verification::step5c14_return_admitted(
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
                6 => case.expectation.expected_phase = Some(PipelinePhase::Verification),
                7 => case.expectation.expected_outcome = ExpectedOutcome::Fail,
                8 => case.expectation.tags.clear(),
                9 => case.expectation.tags.push("extra".into()),
                10 => case.expectation.diagnostic_codes.push("E-UNRELATED".into()),
                11 => case.expectation.stable_detail_key = Some("wrong.key".into()),
                12 => case.expectation.kind = TestKind::Fail,
                13 => case.expectation.rejection_reason = Some("wrong.reason".into()),
                14 => case.expectation.failure_category = Some("proof_failure".into()),
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
                !super::proof_verification::step5c14_return_admitted(
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
                assert!(!super::is_active_parse_only(&case));
                assert!(!super::is_active_declaration_symbol(&case));
                assert!(!super::is_active_type_elaboration(&case));
                assert!(!super::formula_statement::is_active_formula_statement(
                    &config.workspace_root,
                    &case
                ));
                assert!(!super::is_active_proof_verification(&case));
                assert!(!super::step5c11_registration_admitted(
                    &config.workspace_root,
                    &case
                ));
                assert!(!super::step5c13_overload_admitted(
                    &config.workspace_root,
                    &case
                ));
            }
        }
        let run = super::proof_verification::run_proof_verification_case(
            &config.workspace_root,
            &config.workspace_root.join(&config.tests_root),
            &original,
            999,
        );
        assert_eq!(
            run.status,
            super::ProofVerificationCaseStatus::Passed,
            "{:?}",
            run.failure
        );
        let temporary = std::process::Command::new("mktemp")
            .arg("-d")
            .output()
            .unwrap();
        assert!(temporary.status.success());
        let root = PathBuf::from(String::from_utf8(temporary.stdout).unwrap().trim());
        let missing_snapshot = super::proof_verification::run_proof_verification_case(
            &config.workspace_root,
            &root,
            &original,
            0,
        );
        assert_eq!(
            missing_snapshot.status,
            super::ProofVerificationCaseStatus::Failed
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
        let wrong_snapshot = super::proof_verification::run_proof_verification_case(
            &config.workspace_root,
            &root,
            &original,
            0,
        );
        assert_eq!(
            wrong_snapshot.status,
            super::ProofVerificationCaseStatus::Failed
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
                super::validate_active_proof_verification_tags(&config.workspace_root, &invalid)
                    .iter()
                    .any(|d| d.code.0 == "E-PROOF-VERIFICATION-STEP5C14-INVENTORY")
            );
        }
    }
}

#[test]
fn step5c4_dependent_mode_preserves_actual_parameter_bindings_and_checked_types() {
    use mizar_checker::binding_env::{
        BinderIdentity, BindingKind, BindingLookupResult, BindingLookupSite, BindingTypeSite,
    };
    use mizar_checker::type_checker::{
        DeclarationStatus, FormulaKind, FormulaStatus, NormalizedTypeStatus, TermReference,
        TermStatus, TypeHeadRef, check_source_dependent_mode_types,
    };
    use mizar_checker::typed_ast::{FactProvenance, FactStatus, TypeEntryActual};
    let config = step5c11_config();
    let plan = build_test_plan(&config).unwrap();
    let case = plan
        .cases
        .iter()
        .find(|case| case.id.0 == "pass_type_elaboration_mode_dependent_of_params_001")
        .unwrap();
    let exact = std::fs::read_to_string(&case.source_path).unwrap();
    let renamed = exact
        .replace("let X be set", "let Formal be set")
        .replace("of X is", "of Formal is")
        .replace(
            "for A for m being MemberKind of A holds m = m",
            "for Header being set for q being MemberKind of Header holds q = q",
        )
        .replace("let A;", "let Local be set;")
        .replace("let m be MemberKind of A", "let n be MemberKind of Local")
        .replace("thus m = m", "thus n = n");
    for (text, names, nonreflexive) in [
        (exact.clone(), ["X", "A", "A", "m", "A", "m"], false),
        (renamed, ["Formal", "A", "Header", "q", "Local", "n"], false),
        (
            exact.replace("m = m", "m = A"),
            ["X", "A", "A", "m", "A", "m"],
            true,
        ),
    ] {
        let inputs = || {
            super::source_registration_inputs(
                &config.workspace_root,
                case,
                super::formula_statement::step5c8_test_frontend(&text),
            )
            .unwrap()
        };
        let (source, typed, symbols) = inputs();
        let output = check_source_dependent_mode_types(&source, &typed, &symbols).unwrap();
        let (replay_source, replay_typed, replay_symbols) = inputs();
        assert_eq!(
            output,
            check_source_dependent_mode_types(&replay_source, &replay_typed, &replay_symbols)
                .unwrap()
        );
        let (bindings, declarations, inferred) = output;
        assert_eq!(bindings.source_id(), source.source_id());
        assert_eq!(bindings.module_id(), source.module());
        assert!(
            bindings.diagnostics().is_empty()
                && declarations.diagnostics().is_empty()
                && inferred.diagnostics().is_empty()
        );
        assert_eq!(bindings.bindings().len(), 6);
        assert_eq!(declarations.declarations().len(), 6);
        assert_eq!(inferred.terms().len(), 6);
        assert_eq!(inferred.formulas().len(), 2);
        assert!(inferred.candidate_sets().is_empty() && inferred.facts().is_empty());
        assert!(declarations.facts().iter().all(|(_, fact)| matches!(
            fact.status,
            FactStatus::Known | FactStatus::Assumed
        ) && matches!(
            fact.provenance,
            FactProvenance::Declared(_) | FactProvenance::Assumed(_)
        )));
        let proof_start = text.find("proof").unwrap();
        let starts = [
            text.find(&format!("let {} be", names[0])).unwrap() + 4,
            text.find(&format!("reserve {} for", names[1])).unwrap() + 8,
            text.find(&format!("for {} ", names[2])).unwrap() + 4,
            text.find(&format!("for {} being", names[3])).unwrap() + 4,
            proof_start
                + text[proof_start..]
                    .find(&format!("let {}", names[4]))
                    .unwrap()
                + 4,
            proof_start
                + text[proof_start..]
                    .find(&format!("let {} be", names[5]))
                    .unwrap()
                + 4,
        ];
        let expected_kinds = [
            BindingKind::DefinitionParameter,
            BindingKind::ReservedVariable,
            BindingKind::QuantifierBinder,
            BindingKind::QuantifierBinder,
            BindingKind::LetBinding,
            BindingKind::LetBinding,
        ];
        let actual = starts.map(|start| {
            bindings
                .bindings()
                .iter()
                .find(|(_, binding)| binding.declaration_range.start == start)
                .unwrap()
                .1
        });
        assert_eq!(
            actual
                .iter()
                .map(|binding| binding.id)
                .collect::<BTreeSet<_>>()
                .len(),
            6
        );
        for (index, binding) in actual.iter().enumerate() {
            assert_eq!(binding.spelling, names[index]);
            assert_eq!(binding.kind, expected_kinds[index]);
            assert_eq!(
                binding.declaration_range.end,
                starts[index] + names[index].len()
            );
            assert!(!matches!(
                binding.identity,
                BinderIdentity::Generated { .. }
            ));
            let declaration = declarations
                .declarations()
                .iter()
                .find(|(_, declaration)| declaration.binding == binding.id)
                .unwrap()
                .1;
            assert_eq!(declaration.status, DeclarationStatus::Checked);
            assert!(declaration.deferred.is_empty());
            assert_eq!(declaration.context, binding.owner_context);
            let TypeEntryActual::Known(ty) = declarations
                .type_entries()
                .get(declaration.type_entry.unwrap())
                .unwrap()
                .actual
            else {
                panic!("known declaration type")
            };
            let ty = declarations.normalized_types().get(ty).unwrap();
            assert_eq!(ty.head, TypeHeadRef::BuiltinSet);
            assert_eq!(ty.status, NormalizedTypeStatus::Known);
            assert!(
                ty.args.is_empty()
                    && ty.attributes.positive().is_empty()
                    && ty.attributes.negative().is_empty()
            );
        }
        assert_ne!(actual[0].owner_context, actual[2].owner_context);
        assert_ne!(actual[2].owner_context, actual[4].owner_context);
        let mut argument_sites = Vec::new();
        for (use_index, (argument_binding, member_binding)) in
            [(actual[2], actual[3]), (actual[4], actual[5])]
                .into_iter()
                .enumerate()
        {
            let spelling = format!("MemberKind of {}", argument_binding.spelling);
            let application_start = if use_index == 0 {
                text.find(&spelling).unwrap()
            } else {
                proof_start + text[proof_start..].find(&spelling).unwrap()
            };
            let argument_start = application_start + "MemberKind of ".len();
            let argument = inferred.terms().iter().find(|(_, term)| matches!(typed.node(term.site.node()).unwrap().anchor, SourceAnchor::Range(range) if range.start == argument_start && range.end == argument_start + argument_binding.spelling.len())).unwrap().1;
            argument_sites.push(argument.site.clone());
            assert_eq!(
                argument.reference,
                Some(TermReference::Binding(argument_binding.id))
            );
            assert_eq!(argument.status, TermStatus::Inferred);
            assert!(argument.deferred.is_empty() && argument.candidate_set.is_none());
            assert_ne!(
                argument.reference,
                Some(TermReference::Binding(actual[0].id))
            );
            assert_ne!(
                argument.reference,
                Some(TermReference::Binding(actual[1].id))
            );
            let context = bindings.contexts().get(argument.context).unwrap();
            assert!(context.visible_bindings.contains(&argument_binding.id));
            assert!(!context.visible_bindings.contains(&actual[0].id));
            assert_eq!(
                bindings
                    .lookup(&BindingLookupSite::new(
                        &argument_binding.spelling,
                        argument.context,
                        context.lexical_scope.clone(),
                        argument_start
                    ))
                    .unwrap(),
                BindingLookupResult::Local(argument_binding.id)
            );
            let TypeEntryActual::Known(actual_type) = inferred
                .type_entries()
                .get(argument.type_entry)
                .unwrap()
                .actual
            else {
                panic!("known actual argument")
            };
            assert_eq!(
                inferred.normalized_types().get(actual_type).unwrap().head,
                TypeHeadRef::BuiltinSet
            );
            assert_eq!(
                inferred
                    .normalized_types()
                    .get(argument.expected_type.unwrap())
                    .unwrap()
                    .head,
                TypeHeadRef::BuiltinSet
            );
            let declaration = declarations
                .declarations()
                .iter()
                .find(|(_, declaration)| declaration.binding == member_binding.id)
                .unwrap()
                .1;
            let application_range = SourceRange {
                source_id: source.source_id(),
                start: application_start,
                end: application_start + spelling.len(),
            };
            assert_eq!(
                typed
                    .node(declaration.type_site.as_ref().unwrap().node())
                    .unwrap()
                    .anchor,
                SourceAnchor::Range(application_range)
            );
            assert_eq!(
                member_binding.type_site,
                BindingTypeSite::Source(application_range)
            );
        }
        assert_ne!(argument_sites[0], argument_sites[1]);
        for (_, formula) in inferred.formulas().iter() {
            assert_eq!(formula.kind, FormulaKind::Equality);
            assert_eq!(formula.status, FormulaStatus::Checked);
            assert!(formula.deferred.is_empty() && formula.facts.is_empty());
            let expected = if formula.source_range.start < proof_start {
                [
                    actual[3].id,
                    if nonreflexive {
                        actual[2].id
                    } else {
                        actual[3].id
                    },
                ]
            } else {
                [
                    actual[5].id,
                    if nonreflexive {
                        actual[4].id
                    } else {
                        actual[5].id
                    },
                ]
            };
            assert_eq!(formula.terms.len(), 2);
            for (site, binding) in formula.terms.iter().zip(expected) {
                let term = inferred
                    .terms()
                    .iter()
                    .find(|(_, term)| term.site == *site)
                    .unwrap()
                    .1;
                assert_eq!(term.reference, Some(TermReference::Binding(binding)));
                assert_eq!(term.status, TermStatus::Inferred);
            }
        }
    }
}

#[test]
fn step5c4_dependent_mode_rejects_each_bad_argument_and_corrupt_source_association() {
    use mizar_checker::type_checker::check_source_dependent_mode_types;
    use mizar_checker::typed_ast::{NodeRecoveryState, TypedArena, TypingState};
    use mizar_resolve::resolved_ast::SurfaceResolvedArena;
    use mizar_syntax::ast::{SurfaceAstBuilder, SurfaceNodeKind};
    let config = step5c11_config();
    let plan = build_test_plan(&config).unwrap();
    let case = plan
        .cases
        .iter()
        .find(|case| case.id.0 == "pass_type_elaboration_mode_dependent_of_params_001")
        .unwrap();
    let text = std::fs::read_to_string(&case.source_path).unwrap();
    let check = |text: &str| {
        let frontend = super::formula_statement::step5c8_test_frontend(text);
        assert!(
            frontend.diagnostics.is_empty(),
            "semantic control must parse cleanly: {text}: {:?}",
            frontend.diagnostics
        );
        let (source, typed, symbols) =
            super::source_registration_inputs(&config.workspace_root, case, frontend)
                .unwrap_or_else(|error| {
                    panic!("semantic control must reach checker: {text}: {error}")
                });
        check_source_dependent_mode_types(&source, &typed, &symbols)
    };
    assert!(check(&text).is_ok());
    let uses = text
        .match_indices("MemberKind of A")
        .map(|(start, _)| start)
        .collect::<Vec<_>>();
    assert_eq!(uses.len(), 2);
    for start in &uses {
        for replacement in [
            "MemberKind of X",
            "MemberKind of Missing",
            "MemberKind of A, A",
            "MemberKind",
            "set",
        ] {
            let mut changed = text.clone();
            changed.replace_range(*start..*start + "MemberKind of A".len(), replacement);
            assert!(check(&changed).is_err(), "{changed}");
        }
    }
    for start in &uses {
        for replacement in ["MemberKind of", "MissingMode of A"] {
            let mut malformed = text.clone();
            malformed.replace_range(*start..*start + "MemberKind of A".len(), replacement);
            let frontend = super::formula_statement::step5c8_test_frontend(&malformed);
            assert!(!frontend.diagnostics.is_empty(), "{malformed}");
            assert!(
                super::source_registration_inputs(&config.workspace_root, case, frontend).is_err()
            );
        }
    }
    for (from, to) in [
        ("for A for m", "for A being object for m"),
        ("let A;", "let A be object;"),
        ("reserve A for set", "reserve A for object"),
    ] {
        let changed = text.replace(from, to);
        assert!(check(&changed).is_err(), "{changed}");
    }
    for (from, to) in [
        ("let X be set", "let X be object"),
        ("of X is set", "of A is set"),
        ("of X is set", "of X is object"),
    ] {
        let changed = text.replace(from, to);
        assert_ne!(changed, text);
        assert!(check(&changed).is_err(), "{changed}");
    }
    for replacement in ["of X is non empty set", "of X is MissingMode"] {
        let malformed = text.replace("of X is set", replacement);
        let frontend = super::formula_statement::step5c8_test_frontend(&malformed);
        assert!(!frontend.diagnostics.is_empty(), "{malformed}");
        assert!(super::source_registration_inputs(&config.workspace_root, case, frontend).is_err());
    }
    let (definition, rest) = text.split_once("\n\nreserve").unwrap();
    let forward =
        super::formula_statement::step5c8_test_frontend(&format!("reserve{rest}\n{definition}"));
    assert!(!forward.diagnostics.is_empty());
    assert!(super::source_registration_inputs(&config.workspace_root, case, forward).is_err());
    let frontend = super::formula_statement::step5c8_test_frontend(&text);
    let ast = frontend.ast.clone().unwrap();
    for start in &uses {
        let mut builder = SurfaceAstBuilder::new(ast.source_id);
        let mut rebuilt = Vec::new();
        let mut changed_tokens = 0;
        for node in ast.nodes() {
            let children = node.children.iter().map(|id| rebuilt[id.index()]).collect();
            let id = match &node.kind {
                SurfaceNodeKind::Token(token) => {
                    let spelling = if node.range.start == *start {
                        assert_eq!(token.text.as_ref(), "MemberKind");
                        assert_eq!(node.range.end - node.range.start, "OtherKindX".len());
                        changed_tokens += 1;
                        "OtherKindX".into()
                    } else {
                        token.text.clone()
                    };
                    builder.add_token(token.kind, spelling, node.range)
                }
                kind => builder.add_node(kind.clone(), node.range, children),
            };
            rebuilt.push(id);
        }
        assert_eq!(changed_tokens, 1);
        let mut changed_frontend = super::formula_statement::step5c8_test_frontend(&text);
        assert!(changed_frontend.diagnostics.is_empty());
        changed_frontend.ast =
            Some(builder.finish(Some(rebuilt[ast.root().unwrap().index()]), None));
        let (changed_source, changed_typed, changed_symbols) =
            super::source_registration_inputs(&config.workspace_root, case, changed_frontend)
                .unwrap();
        mizar_resolve::symbols::validate_source_symbol_env(&changed_source, &changed_symbols)
            .unwrap();
        assert!(
            check_source_dependent_mode_types(&changed_source, &changed_typed, &changed_symbols)
                .is_err()
        );
    }
    let (source, typed, symbols) =
        super::source_registration_inputs(&config.workspace_root, case, frontend).unwrap();
    let argument_nodes = uses.iter().map(|start| typed.iter().find(|(_, node)| node.kind.as_str() == "TermReference" && matches!(node.anchor, SourceAnchor::Range(range) if range.start == start + "MemberKind of ".len())).unwrap().0).collect::<Vec<_>>();
    let raw = typed
        .iter()
        .map(|(_, node)| node.clone())
        .collect::<Vec<_>>();
    for mutation in 0..7 {
        let mut nodes = raw.clone();
        let first = argument_nodes[0].index();
        let second = argument_nodes[1].index();
        match mutation {
            0 => nodes[first].resolved_node = nodes[second].resolved_node,
            1 => nodes[second].anchor = nodes[first].anchor.clone(),
            2 => nodes[first].kind = "TypeExpression".into(),
            3 => nodes[second].recovery = NodeRecoveryState::Recovered,
            4 => nodes[first].typing = TypingState::Successful,
            5 => nodes[first].children.clear(),
            6 => nodes[second].children = nodes[first].children.clone(),
            _ => unreachable!(),
        }
        assert_ne!(nodes, raw);
        let changed = TypedArena::try_new(typed.root(), nodes).unwrap();
        assert!(
            check_source_dependent_mode_types(&source, &changed, &symbols).is_err(),
            "typed mutation {mutation}"
        );
    }
    let foreign_module =
        ResolverModuleId::new(PackageId::new("foreign"), ModulePath::new("dependent"));
    let foreign = SurfaceResolvedArena::lower(&ast, &foreign_module).unwrap();
    assert!(check_source_dependent_mode_types(&foreign, &typed, &symbols).is_err());
    let mut foreign_ast = ast.clone();
    let ids = InMemorySessionIdAllocator::new();
    ids.next_source_id(snapshot_id(0)).unwrap();
    foreign_ast.source_id = ids.next_source_id(snapshot_id(0)).unwrap();
    let foreign_source = SurfaceResolvedArena::lower(&foreign_ast, symbols.module_id()).unwrap();
    assert!(check_source_dependent_mode_types(&foreign_source, &typed, &symbols).is_err());
    let stale =
        super::formula_statement::step5c8_test_frontend(&text.replace("MemberKind", "OtherKind"))
            .ast
            .unwrap();
    let wrong = super::resolver_symbol_collection(&config.workspace_root, case, &stale);
    assert!(check_source_dependent_mode_types(&source, &typed, &wrong.env).is_err());
    let mut indexes = super::import_fixtures::clone_symbol_env_indexes(&symbols);
    indexes.definitions = Default::default();
    assert!(
        check_source_dependent_mode_types(
            &source,
            &typed,
            &SymbolEnv::new(symbols.module_id().clone(), indexes)
        )
        .is_err()
    );
}

#[test]
fn step5c3_functor_argument_mismatch_tracks_both_real_candidates_and_actual_bindings() {
    use super::type_elaboration::step5c3_functor_argument_detail_keys;
    use mizar_checker::overload_resolution::{
        ArgumentViabilityEvidence, CandidateRejectionReason, CandidateViabilityInput,
        CandidateViabilityOutput, CandidateViabilityStatus, OverloadResultStatus,
        OverloadSelectionOutput, SpecificityGraphOutput,
    };
    use mizar_checker::type_checker::{
        NormalizedTypeStatus, TypeHeadRef, check_source_distinct_loci_overloads,
    };
    use mizar_checker::typed_ast::{TypedNodeId, TypedSiteRef};
    use mizar_resolve::{env::SymbolKind, names::resolve_template_formal};
    let config = step5c11_config();
    let plan = build_test_plan(&config).unwrap();
    let case = plan
        .cases
        .iter()
        .find(|case| case.id.0 == "fail_type_elaboration_argument_type_mismatch_functor_001")
        .unwrap();
    let text = std::fs::read_to_string(&case.source_path).unwrap();
    let renamed = text
        .replace("WBox2", "Container")
        .replace("wget", "extract")
        .replace("WGetDef", "ExtractDef")
        .replace("WidenBad1", "UseExtract")
        .replace(" d ", " value ")
        .replace(".d", ".value")
        .replace("B", "Formal")
        .replace("X", "Actual");
    for (variant, mask) in (0..4)
        .map(|mask| (text.clone(), mask))
        .chain([(renamed, 0)])
    {
        let mut variant = variant;
        if mask & 1 != 0 {
            variant = variant.replace(
                "for X being set holds wget X = X",
                "for X being WBox2 holds wget X = X.d",
            );
        }
        if mask & 2 != 0 {
            variant = variant.replace(
                "let X be set;\n  thus wget X = X",
                "let X be WBox2;\n  thus wget X = X.d",
            );
        }
        let frontend = super::formula_statement::step5c8_test_frontend(&variant);
        assert!(
            frontend.diagnostics.is_empty(),
            "{variant}: {:?}",
            frontend.diagnostics
        );
        let (source, typed, symbols) =
            super::source_registration_inputs(&config.workspace_root, case, frontend).unwrap();
        let outputs =
            check_source_distinct_loci_overloads(&source, &symbols, &typed, true).unwrap();
        assert_eq!(
            outputs,
            check_source_distinct_loci_overloads(&source, &symbols, &typed, true).unwrap()
        );
        assert!(check_source_distinct_loci_overloads(&source, &symbols, &typed, false).is_err());
        let keys = step5c3_functor_argument_detail_keys(&source, &symbols, &typed).unwrap();
        assert_eq!(
            keys,
            if mask == 3 {
                Vec::<String>::new()
            } else {
                vec!["types.application.argument_type_mismatch".into()]
            }
        );
        let (normalization, collection, expansion, viability, graphs, selection) = outputs;
        assert!(normalization.diagnostics().is_empty());
        assert!(collection.diagnostics().is_empty());
        assert!(expansion.diagnostics().is_empty());
        assert_eq!(collection.sites().len(), 2);
        assert_eq!(collection.candidates().len(), 2);
        assert_eq!(expansion.candidates().len(), 2);
        assert_eq!(viability.decisions().len(), 2);
        assert_eq!(graphs.graphs().len(), 2);
        assert_eq!(selection.results().len(), 2);
        assert!(selection.inserted_views().is_empty());
        let constructor = symbols
            .symbols()
            .iter()
            .find(|entry| entry.kind() == SymbolKind::Functor)
            .unwrap();
        let structure = symbols
            .symbols()
            .iter()
            .find(|entry| entry.kind() == SymbolKind::Structure)
            .unwrap();
        let binder_offsets = if variant.contains("Actual") {
            [
                variant.find("for Actual").unwrap() + 4,
                variant.find("let Actual").unwrap() + 4,
            ]
        } else {
            [
                variant.find("for X").unwrap() + 4,
                variant.find("let X").unwrap() + 4,
            ]
        };
        let mut binders = Vec::new();
        for (index, (site_id, site)) in collection.sites().iter().enumerate() {
            let application = typed
                .node(site.owner.node())
                .unwrap()
                .resolved_node
                .unwrap();
            let application_node = source.arena().node(application).unwrap();
            assert_eq!(
                SourceAnchor::Range(site.source_range),
                *application_node.origin().anchor()
            );
            assert_eq!(
                site.arguments,
                vec![TypedSiteRef::Node(TypedNodeId::new(
                    application_node.children()[1].index()
                ))]
            );
            let argument = source.arena().node(application_node.children()[1]).unwrap();
            assert_eq!(
                argument.kind(),
                &mizar_syntax::SurfaceNodeKind::TermReference
            );
            let binder = resolve_template_formal(&source, argument.children()[0]).unwrap();
            assert!(
                matches!(source.arena().node(binder).unwrap().origin().anchor(), SourceAnchor::Range(range) if range.start == binder_offsets[index])
            );
            binders.push(binder);
            let (_, candidate) = expansion
                .candidates()
                .iter()
                .find(|(_, row)| row.site == site_id)
                .unwrap();
            assert_eq!(&candidate.symbol, constructor.symbol());
            assert_eq!(candidate.ordinary_root, candidate.symbol);
            assert_eq!(
                SourceAnchor::Range(candidate.provenance.source_range.unwrap()),
                *constructor.origin().anchor()
            );
            assert_eq!(candidate.parameters.len(), 1);
            let parameter = normalization
                .normalized_types()
                .get(candidate.parameters[0])
                .unwrap();
            assert_eq!(
                parameter.head,
                TypeHeadRef::Structure(structure.symbol().clone())
            );
            assert_eq!(parameter.status, NormalizedTypeStatus::Known);
            assert_eq!(
                normalization
                    .normalized_types()
                    .get(candidate.result.unwrap())
                    .unwrap()
                    .head,
                TypeHeadRef::BuiltinSet
            );
            let (_, decision) = viability
                .decisions()
                .iter()
                .find(|(_, row)| row.site == site_id)
                .unwrap();
            assert_eq!(decision.source_candidate, candidate.id);
            let (_, result) = selection
                .results()
                .iter()
                .find(|(_, row)| row.site == site_id)
                .unwrap();
            assert!(result.diagnostics.is_empty());
            if mask & (1 << index) == 0 {
                let CandidateViabilityStatus::Rejected { reasons } = &decision.status else {
                    panic!("{:?}", decision.status)
                };
                let [reason] = reasons.as_slice() else {
                    panic!("{reasons:?}")
                };
                assert_eq!(reason.argument_index, 0);
                assert_eq!(reason.reason, CandidateRejectionReason::MissingEvidence);
                assert_eq!(reason.target, Some(parameter.id));
                let actual = normalization
                    .normalized_types()
                    .get(reason.actual.unwrap())
                    .unwrap();
                assert_eq!(actual.head, TypeHeadRef::BuiltinSet);
                assert_eq!(actual.status, NormalizedTypeStatus::Known);
                assert!(
                    matches!(&result.status, OverloadResultStatus::NoMatch { rejected } if rejected.is_empty())
                );
                assert!(decision.output_candidate.is_none());
            } else {
                let CandidateViabilityStatus::Viable { views } = &decision.status else {
                    panic!("{:?}", decision.status)
                };
                assert_eq!(views.len(), 1);
                assert_eq!(views[0].actual, parameter.id);
                assert_eq!(views[0].target, parameter.id);
                assert!(matches!(
                    result.status,
                    OverloadResultStatus::Resolved { .. }
                ));
            }
        }
        assert_ne!(binders[0], binders[1]);
        if mask == 0 && variant == text {
            // The same coarse pipeline failure also occurs for the reverse pair;
            // its actual/target heads, rather than NoMatch, identify its meaning.
            let mut candidates = collection
                .candidates()
                .iter()
                .map(
                    |(_, row)| mizar_checker::overload_resolution::OverloadCandidateInput {
                        site: row.site_key.clone(),
                        symbol: row.symbol.clone(),
                        ordinary_root: row.ordinary_root.clone(),
                        declaration_kind: row.declaration_kind.clone(),
                        parameters: vec![row.result.unwrap()],
                        result: row.result,
                        origin: row.origin.clone(),
                        template: None,
                        coherence: None,
                        provenance: row.provenance.clone(),
                    },
                )
                .collect::<Vec<_>>();
            let structure_type = expansion.candidates().iter().next().unwrap().1.parameters[0];
            let sites = collection.sites().iter().map(|(_, row)| {
                mizar_checker::overload_resolution::OverloadSiteInput {
                    key: row.key.clone(),
                    owner: row.owner.clone(),
                    source_range: row.source_range,
                    kind: row.kind.clone(),
                    name: row.name.clone(),
                    arguments: row.arguments.clone(),
                    expected: None,
                    source_qua: Vec::new(),
                    recovery: row.recovery.clone(),
                }
            });
            let other = mizar_checker::overload_resolution::OverloadCollectionOutput::collect(
                sites,
                candidates.drain(..),
            );
            let expanded =
                mizar_checker::overload_resolution::TemplateExpansionOutput::expand(&other);
            let filtered = CandidateViabilityOutput::filter(
                &expanded,
                expanded
                    .candidates()
                    .iter()
                    .map(|(candidate, _)| CandidateViabilityInput {
                        candidate,
                        arguments: vec![ArgumentViabilityEvidence::Exact {
                            actual: structure_type,
                        }],
                    }),
            );
            for (_, row) in filtered.decisions().iter() {
                assert!(
                    matches!(&row.status, CandidateViabilityStatus::Rejected { reasons } if reasons[0].reason == CandidateRejectionReason::MissingEvidence && reasons[0].actual == Some(structure_type))
                );
            }
            let selected =
                OverloadSelectionOutput::resolve(&SpecificityGraphOutput::build(&filtered, []), []);
            assert!(
                selected
                    .results()
                    .iter()
                    .all(|(_, row)| matches!(row.status, OverloadResultStatus::NoMatch { .. }))
            );
        }
    }
}

#[test]
fn step5c3_functor_argument_mismatch_never_credits_unsupported_source_or_forged_identity() {
    use super::type_elaboration::step5c3_functor_argument_detail_keys;
    use mizar_checker::typed_ast::{TypedArena, TypingState};
    use mizar_resolve::resolved_ast::SurfaceResolvedArena;
    let config = step5c11_config();
    let plan = build_test_plan(&config).unwrap();
    let case = plan
        .cases
        .iter()
        .find(|case| case.id.0 == "fail_type_elaboration_argument_type_mismatch_functor_001")
        .unwrap();
    let text = std::fs::read_to_string(&case.source_path).unwrap();
    let inputs = |text: &str| {
        let frontend = super::formula_statement::step5c8_test_frontend(text);
        assert!(
            frontend.diagnostics.is_empty(),
            "semantic control must parse cleanly: {text}: {:?}",
            frontend.diagnostics
        );
        super::source_registration_inputs(&config.workspace_root, case, frontend).unwrap()
    };
    for (from, to) in [
        ("equals B.d", "equals B"),
        ("equals B.d", "equals X.d"),
        ("equals B.d", "equals B.missing"),
        ("wget B -> set", "wget B -> object"),
        ("field d -> set", "field d -> object"),
        ("holds wget X = X", "holds wget B = X"),
        ("thus wget X = X", "thus wget B = X"),
        ("holds wget X = X", "holds wget X = B"),
        ("thus wget X = X", "thus wget X = B"),
        ("let B be WBox2", "let B be set"),
        ("coherence;", "existence;"),
    ] {
        let changed = text.replace(from, to);
        assert_ne!(changed, text);
        let (source, typed, symbols) = inputs(&changed);
        assert!(
            step5c3_functor_argument_detail_keys(&source, &symbols, &typed).is_err(),
            "{changed}"
        );
    }
    let legal_widening = text
        .replace("let B be WBox2", "let B be object")
        .replace("wget B -> set equals B.d", "wget B -> object equals B");
    let (source, typed, symbols) = inputs(&legal_widening);
    assert!(step5c3_functor_argument_detail_keys(&source, &symbols, &typed).is_err());
    let (source, typed, symbols) = inputs(&text);
    let applications = typed
        .iter()
        .filter(|(_, node)| node.kind.as_str().starts_with("PrefixExpression("))
        .map(|(id, _)| id)
        .collect::<Vec<_>>();
    assert_eq!(applications.len(), 2);
    let raw = typed
        .iter()
        .map(|(_, node)| node.clone())
        .collect::<Vec<_>>();
    for application in applications {
        let argument = typed.node(application).unwrap().children[1].index();
        for mutation in 0..4 {
            let mut nodes = raw.clone();
            match mutation {
                0 => nodes[argument].anchor = nodes[application.index()].anchor.clone(),
                1 => nodes[argument].resolved_node = nodes[application.index()].resolved_node,
                2 => nodes[argument].typing = TypingState::Successful,
                3 => nodes[argument].children.clear(),
                _ => unreachable!(),
            }
            let changed = TypedArena::try_new(typed.root(), nodes).unwrap();
            assert!(step5c3_functor_argument_detail_keys(&source, &symbols, &changed).is_err());
        }
    }
    let ast = super::formula_statement::step5c8_test_frontend(&text)
        .ast
        .unwrap();
    let call_offsets = text
        .match_indices("wget X")
        .map(|(offset, _)| offset)
        .collect::<Vec<_>>();
    assert_eq!(call_offsets.len(), 2);
    for target in call_offsets.into_iter().map(Some).chain([None]) {
        use mizar_syntax::ast::{SurfaceAstBuilder, SurfaceNodeKind as K};
        let mut builder = SurfaceAstBuilder::new(ast.source_id);
        let mut rebuilt = Vec::new();
        let mut changes = 0;
        for node in ast.nodes() {
            let mut children = node
                .children
                .iter()
                .map(|id| rebuilt[id.index()])
                .collect::<Vec<_>>();
            let id = match &node.kind {
                K::Token(token) => {
                    let spelling = if Some(node.range.start) == target {
                        assert_eq!(token.text.as_ref(), "wget");
                        changes += 1;
                        "oops".into()
                    } else {
                        token.text.clone()
                    };
                    builder.add_token(token.kind, spelling, node.range)
                }
                K::PrefixExpression(operator) if Some(node.range.start) == target => {
                    let mut operator = operator.clone();
                    operator.spelling = "oops".into();
                    builder.add_node(K::PrefixExpression(operator), node.range, children)
                }
                kind => {
                    if target.is_none() && kind == &K::ItemList {
                        assert_eq!(children.len(), 3);
                        children.swap(0, 1);
                        changes += 1;
                    }
                    builder.add_node(kind.clone(), node.range, children)
                }
            };
            rebuilt.push(id);
        }
        assert_eq!(changes, 1);
        let mut frontend = super::formula_statement::step5c8_test_frontend(&text);
        frontend.ast = Some(builder.finish(Some(rebuilt[ast.root().unwrap().index()]), None));
        let (changed_source, changed_typed, changed_symbols) =
            super::source_registration_inputs(&config.workspace_root, case, frontend).unwrap();
        mizar_resolve::symbols::validate_source_symbol_env(&changed_source, &changed_symbols)
            .unwrap();
        assert!(
            step5c3_functor_argument_detail_keys(&changed_source, &changed_symbols, &changed_typed)
                .is_err()
        );
    }
    let foreign_module =
        ResolverModuleId::new(PackageId::new("foreign"), ModulePath::new("argument"));
    let foreign = SurfaceResolvedArena::lower(&ast, &foreign_module).unwrap();
    assert!(step5c3_functor_argument_detail_keys(&foreign, &symbols, &typed).is_err());
    let mut other_ast = ast.clone();
    let ids = InMemorySessionIdAllocator::new();
    ids.next_source_id(snapshot_id(0)).unwrap();
    other_ast.source_id = ids.next_source_id(snapshot_id(0)).unwrap();
    let foreign_source = SurfaceResolvedArena::lower(&other_ast, symbols.module_id()).unwrap();
    assert!(step5c3_functor_argument_detail_keys(&foreign_source, &symbols, &typed).is_err());
    let (_, _, other_symbols) = inputs(&text.replace("WBox2", "Container"));
    assert!(step5c3_functor_argument_detail_keys(&source, &other_symbols, &typed).is_err());
    let mut indexes = super::import_fixtures::clone_symbol_env_indexes(&symbols);
    indexes.definitions = Default::default();
    assert!(
        step5c3_functor_argument_detail_keys(
            &source,
            &SymbolEnv::new(symbols.module_id().clone(), indexes),
            &typed
        )
        .is_err()
    );
    let theorem_start = text.find("theorem WidenBad1:").unwrap();
    let unrelated = format!("{text}\n{}", &text[theorem_start..]);
    let frontend = super::formula_statement::step5c8_test_frontend(&unrelated);
    assert!(frontend.diagnostics.is_empty(), "{:?}", frontend.diagnostics);
    let result = super::resolver_symbol_collection(
        &config.workspace_root,
        case,
        frontend.ast.as_ref().unwrap(),
    );
    assert!(!result.detail_keys.is_empty());
    assert!(super::source_registration_inputs(&config.workspace_root, case, frontend).is_err());
    let malformed = text.replace("thus wget X = X;", "thus wget X = ;");
    let frontend = super::formula_statement::step5c8_test_frontend(&malformed);
    assert!(!frontend.diagnostics.is_empty());
    assert!(super::source_registration_inputs(&config.workspace_root, case, frontend).is_err());
}
#[test]
fn step5c5_predicate_arguments_preserve_order_bindings_polarity_and_real_rejections() {
    use super::type_elaboration::{
        step5c3_functor_argument_detail_keys, step5c5_predicate_argument_detail_keys,
    };
    use mizar_checker::overload_resolution::{
        CandidateDeclarationKind, CandidateRejectionReason, CandidateViabilityStatus,
        OverloadResultStatus, OverloadSiteKind,
    };
    use mizar_checker::type_checker::{
        NormalizedTypeStatus, TypeHeadRef, check_source_distinct_loci_overloads,
    };
    use mizar_resolve::{env::SymbolKind, names::resolve_template_formal};
    use mizar_syntax::SurfaceNodeKind as K;
    let config = step5c11_config();
    let plan = build_test_plan(&config).unwrap();
    let case = plan
        .cases
        .iter()
        .find(|case| case.id.0 == "fail_type_elaboration_pred_argument_type_mismatch_001")
        .unwrap();
    let text = std::fs::read_to_string(&case.source_path).unwrap();
    let valid = text
        .replace("X being set", "X being PredBox")
        .replace("X be set", "X be PredBox");
    let renamed = text
        .replace("PredBox", "Container")
        .replace("MatchesDef", "RelatesDef")
        .replace("BadMatch1", "UseRelates")
        .replace("matches", "relates")
        .replace("let P, Q", "let Left, Right")
        .replace("P relates Q", "Left relates Right")
        .replace("P.d", "Left.value")
        .replace("Q.d", "Right.value")
        .replace("field d", "field value")
        .replace("X", "Actual");
    let mut variants = Vec::new();
    for mask in 0..4 {
        let mut changed = text.clone();
        if mask & 1 != 0 {
            changed = changed.replace("X being set", "X being PredBox");
        }
        if mask & 2 != 0 {
            changed = changed.replace("X be set", "X be PredBox");
        }
        variants.push((
            changed,
            [
                if mask & 1 == 0 { vec![0, 1] } else { vec![] },
                if mask & 2 == 0 { vec![0, 1] } else { vec![] },
            ],
        ));
    }
    variants.extend([
        (renamed, [vec![0, 1], vec![0, 1]]),
        (
            valid.replace("X matches X", "X.d matches X"),
            [vec![0], vec![0]],
        ),
        (
            valid.replace("X matches X", "X matches X.d"),
            [vec![1], vec![1]],
        ),
        (
            text.replace("X matches X", "X does not matches X"),
            [vec![0, 1], vec![0, 1]],
        ),
        (
            valid.replace("X matches X", "X does not matches X"),
            [vec![], vec![]],
        ),
    ]);
    for (variant, rejected_indices) in variants {
        let frontend = super::formula_statement::step5c8_test_frontend(&variant);
        assert!(
            frontend.diagnostics.is_empty(),
            "{variant}: {:?}",
            frontend.diagnostics
        );
        let (source, typed, symbols) =
            super::source_registration_inputs(&config.workspace_root, case, frontend).unwrap();
        let outputs =
            check_source_distinct_loci_overloads(&source, &symbols, &typed, true).unwrap();
        assert_eq!(
            outputs,
            check_source_distinct_loci_overloads(&source, &symbols, &typed, true).unwrap()
        );
        assert!(check_source_distinct_loci_overloads(&source, &symbols, &typed, false).is_err());
        assert!(step5c3_functor_argument_detail_keys(&source, &symbols, &typed).is_err());
        let rejected_sites = rejected_indices
            .iter()
            .filter(|indices| !indices.is_empty())
            .count();
        assert_eq!(
            step5c5_predicate_argument_detail_keys(&source, &symbols, &typed).unwrap(),
            if rejected_sites == 0 {
                vec![]
            } else {
                vec!["predicates.application.argument_type_mismatch".to_owned()]
            }
        );
        let (normalization, collection, expansion, viability, graphs, selection) = outputs;
        assert!(normalization.diagnostics().is_empty());
        assert!(collection.diagnostics().is_empty());
        assert!(expansion.diagnostics().is_empty());
        assert_eq!(collection.sites().len(), 2);
        assert_eq!(collection.candidates().len(), 2);
        assert_eq!(expansion.candidates().len(), 2);
        assert_eq!(viability.decisions().len(), 2);
        assert_eq!(viability.diagnostics().len(), rejected_sites);
        assert_eq!(graphs.graphs().len(), 2);
        assert_eq!(selection.results().len(), 2);
        assert!(selection.inserted_views().is_empty());
        let predicate = symbols
            .symbols()
            .iter()
            .find(|entry| entry.kind() == SymbolKind::Predicate)
            .unwrap();
        let structure = symbols
            .symbols()
            .iter()
            .find(|entry| entry.kind() == SymbolKind::Structure)
            .unwrap();
        let selector = symbols
            .symbols()
            .iter()
            .find(|entry| entry.kind() == SymbolKind::Selector)
            .unwrap();
        assert!(
            matches!(selector.origin().anchor(), SourceAnchor::Range(range) if variant[range.start..range.end].contains("field "))
        );
        let (_, pattern) = source
            .arena()
            .iter()
            .find(|(_, node)| node.kind() == &K::PredicatePattern)
            .unwrap();
        let formals = [
            resolve_template_formal(&source, pattern.children()[0]).unwrap(),
            resolve_template_formal(&source, pattern.children()[2]).unwrap(),
        ];
        assert_ne!(formals[0], formals[1]);
        let renamed = variant.contains("let Left, Right");
        let formal_offsets = if renamed {
            [
                variant.find("let Left").unwrap() + 4,
                variant.find(", Right be").unwrap() + 2,
            ]
        } else {
            [
                variant.find("let P").unwrap() + 4,
                variant.find(", Q be").unwrap() + 2,
            ]
        };
        for (formal, offset) in formals.iter().zip(formal_offsets) {
            assert!(
                matches!(source.arena().node(*formal).unwrap().origin().anchor(), SourceAnchor::Range(range) if range.start == offset)
            );
        }
        let actual_name = if renamed { "Actual" } else { "X" };
        let binder_offsets = [
            variant.find(&format!("for {actual_name}")).unwrap() + 4,
            variant.find(&format!("let {actual_name}")).unwrap() + 4,
        ];
        let mut occurrences = Vec::new();
        let mut binders = Vec::new();
        for (index, (site_id, site)) in collection.sites().iter().enumerate() {
            assert_eq!(site.kind, OverloadSiteKind::PredicateApplication);
            let application = source
                .arena()
                .node(
                    typed
                        .node(site.owner.node())
                        .unwrap()
                        .resolved_node
                        .unwrap(),
                )
                .unwrap();
            assert_eq!(application.kind(), &K::PredicateApplication);
            assert_eq!(
                SourceAnchor::Range(site.source_range),
                *application.origin().anchor()
            );
            if variant.contains(" not ") {
                assert!(variant[site.source_range.start..site.source_range.end].contains(" not "));
            }
            let segment = source.arena().node(application.children()[0]).unwrap();
            assert_eq!(segment.kind(), &K::PredicateSegment);
            let actual_slots = [segment.children()[0], *segment.children().last().unwrap()];
            assert_eq!(
                site.arguments
                    .iter()
                    .map(|site| site.node().index())
                    .collect::<Vec<_>>(),
                actual_slots.map(|id| source.arena().node(id).unwrap().children()[0].index())
            );
            for argument in &site.arguments {
                let node = source
                    .arena()
                    .node(typed.node(argument.node()).unwrap().resolved_node.unwrap())
                    .unwrap();
                let reference = if node.kind() == &K::SelectorAccess {
                    source.arena().node(node.children()[0]).unwrap()
                } else {
                    node
                };
                assert_eq!(reference.kind(), &K::TermReference);
                let binder = resolve_template_formal(&source, reference.children()[0]).unwrap();
                assert!(!formals.contains(&binder));
                assert!(
                    matches!(source.arena().node(binder).unwrap().origin().anchor(), SourceAnchor::Range(range) if range.start == binder_offsets[index])
                );
                binders.push(binder);
                occurrences.push(argument.clone());
            }
            let (_, candidate) = expansion
                .candidates()
                .iter()
                .find(|(_, row)| row.site == site_id)
                .unwrap();
            assert_eq!(
                candidate.declaration_kind,
                CandidateDeclarationKind::Predicate
            );
            assert_eq!(&candidate.symbol, predicate.symbol());
            assert_eq!(candidate.ordinary_root, candidate.symbol);
            assert_eq!(
                SourceAnchor::Range(candidate.provenance.source_range.unwrap()),
                *predicate.origin().anchor()
            );
            assert!(candidate.result.is_none());
            assert_eq!(candidate.parameters.len(), 2);
            for parameter in &candidate.parameters {
                let parameter = normalization.normalized_types().get(*parameter).unwrap();
                assert_eq!(
                    parameter.head,
                    TypeHeadRef::Structure(structure.symbol().clone())
                );
                assert_eq!(parameter.status, NormalizedTypeStatus::Known);
            }
            let (_, decision) = viability
                .decisions()
                .iter()
                .find(|(_, row)| row.site == site_id)
                .unwrap();
            assert_eq!(decision.source_candidate, candidate.id);
            let (_, result) = selection
                .results()
                .iter()
                .find(|(_, row)| row.site == site_id)
                .unwrap();
            assert!(result.diagnostics.is_empty());
            if rejected_indices[index].is_empty() {
                assert!(
                    matches!(&decision.status, CandidateViabilityStatus::Viable { views } if views.len() == 2 && views.iter().all(|view| view.actual == candidate.parameters[view.argument_index] && view.target == view.actual))
                );
                assert!(
                    matches!(&result.status, OverloadResultStatus::Resolved { exposed_result: None, refinements, inserted_views, .. } if refinements.is_empty() && inserted_views.is_empty())
                );
            } else {
                let CandidateViabilityStatus::Rejected { reasons } = &decision.status else {
                    panic!("{:?}", decision.status)
                };
                assert_eq!(
                    reasons
                        .iter()
                        .map(|reason| reason.argument_index)
                        .collect::<Vec<_>>(),
                    rejected_indices[index]
                );
                for reason in reasons {
                    assert_eq!(reason.reason, CandidateRejectionReason::MissingEvidence);
                    assert_eq!(
                        reason.target,
                        Some(candidate.parameters[reason.argument_index])
                    );
                    let actual = normalization
                        .normalized_types()
                        .get(reason.actual.unwrap())
                        .unwrap();
                    assert_eq!(actual.head, TypeHeadRef::BuiltinSet);
                    assert_eq!(actual.status, NormalizedTypeStatus::Known);
                }
                assert_eq!(decision.diagnostics.len(), 1);
                assert!(decision.output_candidate.is_none());
                assert!(
                    matches!(&result.status, OverloadResultStatus::NoMatch { rejected } if rejected.is_empty())
                );
            }
        }
        for (index, occurrence) in occurrences.iter().enumerate() {
            assert!(!occurrences[..index].contains(occurrence));
        }
        assert_eq!(binders[0], binders[1]);
        assert_eq!(binders[2], binders[3]);
        assert_ne!(binders[0], binders[2]);
    }
}

#[test]
fn step5c5_predicate_argument_mismatch_rejects_other_errors_and_forged_inputs() {
    use super::type_elaboration::{
        step5c3_functor_argument_detail_keys, step5c5_predicate_argument_detail_keys,
    };
    use mizar_checker::typed_ast::{TypedArena, TypingState};
    use mizar_resolve::resolved_ast::SurfaceResolvedArena;
    use mizar_syntax::ast::{SurfaceAstBuilder, SurfaceNodeKind as K};
    let config = step5c11_config();
    let plan = build_test_plan(&config).unwrap();
    let case = plan
        .cases
        .iter()
        .find(|case| case.id.0 == "fail_type_elaboration_pred_argument_type_mismatch_001")
        .unwrap();
    let text = std::fs::read_to_string(&case.source_path).unwrap();
    let inputs = |text: &str| {
        let frontend = super::formula_statement::step5c8_test_frontend(text);
        assert!(
            frontend.diagnostics.is_empty(),
            "semantic control must parse cleanly: {text}: {:?}",
            frontend.diagnostics
        );
        super::source_registration_inputs(&config.workspace_root, case, frontend).unwrap()
    };
    for (from, to) in [
        ("P.d = Q.d", "P.d = Q"),
        ("P.d = Q.d", "X.d = Q.d"),
        ("P.d = Q.d", "P.missing = Q.d"),
        ("P.d = Q.d", "P.d = Q.missing"),
        ("P matches Q", "Q matches P"),
        ("P matches Q", "P matches P"),
        ("P, Q be PredBox", "P, Q be set"),
        ("field d -> set", "field d -> object"),
        ("holds X matches X", "holds P matches X"),
        ("thus X matches X", "thus X matches Q"),
        ("X matches X", "X matches X matches X"),
    ] {
        let changed = text.replace(from, to);
        assert_ne!(changed, text);
        let (source, typed, symbols) = inputs(&changed);
        assert!(
            step5c5_predicate_argument_detail_keys(&source, &symbols, &typed).is_err(),
            "{changed}"
        );
    }
    let widening = text
        .replace("P, Q be PredBox", "P, Q be object")
        .replace("P.d = Q.d", "P = Q");
    let (source, typed, symbols) = inputs(&widening);
    assert!(step5c5_predicate_argument_detail_keys(&source, &symbols, &typed).is_err());
    let (source, typed, symbols) = inputs(&text);
    let raw = typed
        .iter()
        .map(|(_, node)| node.clone())
        .collect::<Vec<_>>();
    let argument_ids = typed.iter().filter(|(_, node)| node.kind.as_str() == "TermReference" && matches!(node.anchor, SourceAnchor::Range(range) if range.start > text.find("theorem").unwrap())).map(|(id, _)| id).collect::<Vec<_>>();
    assert_eq!(argument_ids.len(), 4);
    for argument in &argument_ids {
        for mutation in 0..3 {
            let mut nodes = raw.clone();
            let other = argument_ids
                .iter()
                .find(|id| *id != argument)
                .unwrap()
                .index();
            match mutation {
                0 => nodes[argument.index()].resolved_node = nodes[other].resolved_node,
                1 => nodes[argument.index()].anchor = nodes[other].anchor.clone(),
                2 => nodes[argument.index()].typing = TypingState::Successful,
                _ => unreachable!(),
            }
            assert!(
                step5c5_predicate_argument_detail_keys(
                    &source,
                    &symbols,
                    &TypedArena::try_new(typed.root(), nodes).unwrap()
                )
                .is_err()
            );
        }
    }
    let ast = super::formula_statement::step5c8_test_frontend(&text)
        .ast
        .unwrap();
    let offsets = text
        .match_indices("X matches X")
        .map(|(offset, _)| offset + 2)
        .collect::<Vec<_>>();
    assert_eq!(offsets.len(), 2);
    for target in offsets.into_iter().map(Some).chain([None]) {
        let mut builder = SurfaceAstBuilder::new(ast.source_id);
        let mut rebuilt = Vec::new();
        let mut changed = 0;
        for node in ast.nodes() {
            let mut children = node
                .children
                .iter()
                .map(|id| rebuilt[id.index()])
                .collect::<Vec<_>>();
            let id = match &node.kind {
                K::Token(token) => {
                    let spelling = if Some(node.range.start) == target {
                        assert_eq!(token.text.as_ref(), "matches");
                        changed += 1;
                        "unknown".into()
                    } else {
                        token.text.clone()
                    };
                    builder.add_token(token.kind, spelling, node.range)
                }
                kind => {
                    if target.is_none() && kind == &K::ItemList {
                        assert_eq!(children.len(), 3);
                        children.swap(0, 1);
                        changed += 1;
                    }
                    builder.add_node(kind.clone(), node.range, children)
                }
            };
            rebuilt.push(id);
        }
        assert_eq!(changed, 1);
        let mut frontend = super::formula_statement::step5c8_test_frontend(&text);
        frontend.ast = Some(builder.finish(Some(rebuilt[ast.root().unwrap().index()]), None));
        let (changed_source, changed_typed, changed_symbols) =
            super::source_registration_inputs(&config.workspace_root, case, frontend).unwrap();
        mizar_resolve::symbols::validate_source_symbol_env(&changed_source, &changed_symbols)
            .unwrap();
        assert!(
            step5c5_predicate_argument_detail_keys(
                &changed_source,
                &changed_symbols,
                &changed_typed
            )
            .is_err()
        );
    }
    let foreign = ResolverModuleId::new(PackageId::new("foreign"), ModulePath::new("predicate"));
    assert!(
        step5c5_predicate_argument_detail_keys(
            &SurfaceResolvedArena::lower(&ast, &foreign).unwrap(),
            &symbols,
            &typed
        )
        .is_err()
    );
    let mut foreign_ast = ast.clone();
    let ids = InMemorySessionIdAllocator::new();
    ids.next_source_id(snapshot_id(0)).unwrap();
    foreign_ast.source_id = ids.next_source_id(snapshot_id(0)).unwrap();
    assert!(
        step5c5_predicate_argument_detail_keys(
            &SurfaceResolvedArena::lower(&foreign_ast, symbols.module_id()).unwrap(),
            &symbols,
            &typed
        )
        .is_err()
    );
    let (_, _, other_symbols) = inputs(&text.replace("PredBox", "OtherBox"));
    assert!(step5c5_predicate_argument_detail_keys(&source, &other_symbols, &typed).is_err());
    let mut indexes = super::import_fixtures::clone_symbol_env_indexes(&symbols);
    indexes.definitions = Default::default();
    assert!(
        step5c5_predicate_argument_detail_keys(
            &source,
            &SymbolEnv::new(symbols.module_id().clone(), indexes),
            &typed
        )
        .is_err()
    );
    let theorem = text.find("theorem BadMatch1:").unwrap();
    let duplicate = format!("{text}\n{}", &text[theorem..]);
    let frontend = super::formula_statement::step5c8_test_frontend(&duplicate);
    assert!(frontend.diagnostics.is_empty());
    let result = super::resolver_symbol_collection(
        &config.workspace_root,
        case,
        frontend.ast.as_ref().unwrap(),
    );
    assert!(!result.detail_keys.is_empty());
    assert!(super::source_registration_inputs(&config.workspace_root, case, frontend).is_err());
    let frontend = super::formula_statement::step5c8_test_frontend(
        &text.replace("thus X matches X;", "thus X does matches X;"),
    );
    assert!(!frontend.diagnostics.is_empty());
    assert!(super::source_registration_inputs(&config.workspace_root, case, frontend).is_err());
    let functor_case = plan
        .cases
        .iter()
        .find(|case| case.id.0 == "fail_type_elaboration_argument_type_mismatch_functor_001")
        .unwrap();
    let frontend = super::formula_statement::step5c8_test_frontend(
        &std::fs::read_to_string(&functor_case.source_path).unwrap(),
    );
    let (source, typed, symbols) =
        super::source_registration_inputs(&config.workspace_root, functor_case, frontend).unwrap();
    assert_eq!(
        step5c3_functor_argument_detail_keys(&source, &symbols, &typed).unwrap(),
        vec!["types.application.argument_type_mismatch"]
    );
    assert!(step5c5_predicate_argument_detail_keys(&source, &symbols, &typed).is_err());
}
#[test]
fn step5c6_functor_synonym_types_preserve_original_root_locus_order_and_eight_binders() {
    use mizar_checker::overload_resolution::{CandidateViabilityStatus, OverloadSiteKind};
    use mizar_checker::type_checker::{TypeHeadRef, check_source_functor_synonym_types};
    use mizar_checker::typed_ast::{TypeEntryActual, TypedNodeId, TypedSiteRef};
    use mizar_resolve::{
        env::{RelationKind, SymbolKind},
        names::resolve_template_formal,
    };
    use mizar_syntax::SurfaceNodeKind as K;
    let config = step5c11_config();
    let plan = build_test_plan(&config).unwrap();
    let case = plan
        .cases
        .iter()
        .find(|case| case.id.0 == "pass_type_elaboration_synonym_functor_001")
        .unwrap();
    let text = std::fs::read_to_string(&case.source_path).unwrap();
    let blocks = text.split("\n\n").collect::<Vec<_>>();
    let (header, proof) = blocks[2].split_once("proof").unwrap();
    let renamed = format!(
        "{}\n\n{}\n\n{}proof{}",
        blocks[0].replace("X", "BaseL").replace("Y", "BaseR"),
        blocks[1].replace("X", "AliasL").replace("Y", "AliasR"),
        header.replace("X", "HeaderL").replace("Y", "HeaderR"),
        proof.replace("X", "LocalL").replace("Y", "LocalR")
    )
    .replace("synbase", "original")
    .replace("synalt", "alternate")
    .replace("SynBaseDef", "OriginalDef")
    .replace("SynUse1", "UseAlias");
    for (variant, permutation, actual_order) in [
        (text.clone(), [0, 1], [[0, 1], [0, 1]]),
        (renamed.clone(), [0, 1], [[0, 1], [0, 1]]),
        (
            renamed.replace(
                "synonym AliasL alternate AliasR",
                "synonym AliasR alternate AliasL",
            ),
            [1, 0],
            [[0, 1], [0, 1]],
        ),
        (
            text.replace("synonym X synalt Y", "synonym Y synalt X"),
            [1, 0],
            [[0, 1], [0, 1]],
        ),
        (
            text.replace("for X synbase Y", "for Y synbase X"),
            [1, 0],
            [[0, 1], [0, 1]],
        ),
        (
            text.replace("holds X synalt Y", "holds Y synalt X"),
            [0, 1],
            [[1, 0], [0, 1]],
        ),
        (
            text.replace("thus X synalt Y", "thus Y synalt X"),
            [0, 1],
            [[0, 1], [1, 0]],
        ),
        (
            text.replace("holds X synalt Y", "holds X synalt X")
                .replace("thus X synalt Y", "thus X synalt X"),
            [0, 1],
            [[0, 0], [0, 0]],
        ),
        (
            text.replace("equals X;", "equals Y;"),
            [0, 1],
            [[0, 1], [0, 1]],
        ),
        (
            text.replace("= X\nproof", "= Y\nproof")
                .replace("= X by", "= Y by"),
            [0, 1],
            [[0, 1], [0, 1]],
        ),
    ] {
        let frontend = super::formula_statement::step5c8_test_frontend(&variant);
        assert!(
            frontend.diagnostics.is_empty(),
            "{variant}: {:?}",
            frontend.diagnostics
        );
        let (source, typed, symbols) =
            super::source_registration_inputs(&config.workspace_root, case, frontend).unwrap();
        let output = check_source_functor_synonym_types(&source, &symbols, &typed).unwrap();
        assert_eq!(
            output,
            check_source_functor_synonym_types(&source, &symbols, &typed).unwrap()
        );
        let (normalization, collection, viability) = output;
        assert!(normalization.diagnostics().is_empty());
        assert!(collection.diagnostics().is_empty());
        assert!(viability.diagnostics().is_empty());
        assert_eq!(collection.sites().len(), 2);
        assert_eq!(collection.candidates().len(), 2);
        assert_eq!(viability.decisions().len(), 2);
        let original = symbols
            .symbols()
            .iter()
            .find(|entry| entry.kind() == SymbolKind::Functor)
            .unwrap();
        let alias = symbols
            .symbols()
            .iter()
            .find(|entry| entry.kind() == SymbolKind::Synonym)
            .unwrap();
        assert_ne!(original.symbol(), alias.symbol());
        assert_eq!(alias.relations().len(), 1);
        assert_eq!(alias.relations()[0].kind(), RelationKind::Synonym);
        assert_eq!(alias.relations()[0].target(), original.symbol());
        let mut groups = source
            .arena()
            .iter()
            .filter(|(_, node)| {
                matches!(
                    node.kind(),
                    K::QualifiedVariableSegment | K::QuantifierVariableSegment
                )
            })
            .map(|(_, node)| [node.children()[0], node.children()[2]])
            .collect::<Vec<_>>();
        groups.sort_by_key(|group| {
            match source.arena().node(group[0]).unwrap().origin().anchor() {
                SourceAnchor::Range(range) => range.start,
                _ => unreachable!(),
            }
        });
        assert_eq!(groups.len(), 4);
        let identities = groups
            .iter()
            .flatten()
            .copied()
            .collect::<std::collections::BTreeSet<_>>();
        assert_eq!(identities.len(), 8);
        let (_, original_pattern) = source
            .arena()
            .iter()
            .find(|(_, node)| node.kind() == &K::FunctorPattern)
            .unwrap();
        assert_eq!(
            [
                resolve_template_formal(&source, original_pattern.children()[0]).unwrap(),
                resolve_template_formal(&source, original_pattern.children()[2]).unwrap()
            ],
            groups[0]
        );
        for (_, pattern) in source
            .arena()
            .iter()
            .filter(|(_, node)| node.kind() == &K::NotationPattern)
        {
            let binding_pair = [
                resolve_template_formal(&source, pattern.children()[0]).unwrap(),
                resolve_template_formal(&source, pattern.children()[2]).unwrap(),
            ];
            assert_ne!(binding_pair[0], binding_pair[1]);
            assert!(binding_pair.iter().all(|id| groups[1].contains(id)));
        }
        let (_, definition) = source
            .arena()
            .iter()
            .find(|(_, node)| node.kind() == &K::FunctorDefinition)
            .unwrap();
        let return_site = TypedSiteRef::Node(TypedNodeId::new(definition.children()[5].index()));
        let declared_result = normalization
            .type_entries()
            .iter()
            .find(|(_, entry)| entry.owner == return_site)
            .unwrap()
            .1
            .actual;
        let mut occurrences = Vec::new();
        for (index, (site_id, site)) in collection.sites().iter().enumerate() {
            assert_eq!(site.kind, OverloadSiteKind::FunctorApplication);
            let application = source
                .arena()
                .node(
                    typed
                        .node(site.owner.node())
                        .unwrap()
                        .resolved_node
                        .unwrap(),
                )
                .unwrap();
            assert!(matches!(application.kind(), K::InfixExpression(_)));
            assert_eq!(
                SourceAnchor::Range(site.source_range),
                *application.origin().anchor()
            );
            let written = [application.children()[0], application.children()[2]];
            assert_ne!(written[0], written[1]);
            assert_eq!(
                site.arguments,
                permutation.map(|slot| TypedSiteRef::Node(TypedNodeId::new(written[slot].index())))
            );
            for (slot, reference) in written.iter().enumerate() {
                let node = source.arena().node(*reference).unwrap();
                assert_eq!(node.kind(), &K::TermReference);
                assert_eq!(
                    resolve_template_formal(&source, node.children()[0]).unwrap(),
                    groups[index + 2][actual_order[index][slot]]
                );
                occurrences.push(*reference);
            }
            let (_, candidate) = collection
                .candidates()
                .iter()
                .find(|(_, row)| row.site == site_id)
                .unwrap();
            assert_eq!(&candidate.symbol, original.symbol());
            assert_eq!(candidate.ordinary_root, candidate.symbol);
            assert_eq!(
                SourceAnchor::Range(candidate.provenance.source_range.unwrap()),
                *original.origin().anchor()
            );
            assert_eq!(
                declared_result,
                TypeEntryActual::Known(candidate.result.unwrap())
            );
            assert_eq!(candidate.parameters.len(), 2);
            for ty in candidate.parameters.iter().chain(candidate.result.iter()) {
                assert_eq!(
                    normalization.normalized_types().get(*ty).unwrap().head,
                    TypeHeadRef::BuiltinSet
                );
            }
            let (_, decision) = viability
                .decisions()
                .iter()
                .find(|(_, row)| row.site == site_id)
                .unwrap();
            assert_eq!(decision.source_candidate, candidate.id);
            let CandidateViabilityStatus::Viable { views } = &decision.status else {
                panic!("{:?}", decision.status)
            };
            assert_eq!(
                views
                    .iter()
                    .map(|view| view.argument_index)
                    .collect::<Vec<_>>(),
                vec![0, 1]
            );
            assert!(views.iter().all(|view| view.actual
                == candidate.parameters[view.argument_index]
                && view.target == view.actual
                && view.facts.is_empty()
                && view.coercion.is_none()));
        }
        assert_eq!(
            occurrences
                .iter()
                .collect::<std::collections::BTreeSet<_>>()
                .len(),
            4
        );
    }
}

#[test]
fn step5c6_functor_synonym_types_reject_unrelated_failures_and_forged_relations() {
    use mizar_checker::type_checker::check_source_functor_synonym_types;
    use mizar_checker::typed_ast::{TypedArena, TypingState};
    use mizar_resolve::{
        env::{RelationKind, RelationMetadata, SymbolIndex, SymbolKind},
        resolved_ast::SurfaceResolvedArena,
    };
    use mizar_syntax::ast::{SurfaceAstBuilder, SurfaceNodeKind as K};
    let config = step5c11_config();
    let plan = build_test_plan(&config).unwrap();
    let case = plan
        .cases
        .iter()
        .find(|case| case.id.0 == "pass_type_elaboration_synonym_functor_001")
        .unwrap();
    let text = std::fs::read_to_string(&case.source_path).unwrap();
    let inputs = |text: &str| {
        let frontend = super::formula_statement::step5c8_test_frontend(text);
        assert!(
            frontend.diagnostics.is_empty(),
            "semantic control must parse cleanly: {text}: {:?}",
            frontend.diagnostics
        );
        super::source_registration_inputs(&config.workspace_root, case, frontend).unwrap()
    };
    for (from, to) in [
        ("equals X;", "equals Missing;"),
        ("equals X;", "equals (X);"),
        ("-> set", "-> object"),
        ("being set", "being object"),
        ("let X, Y be set;\n  thus", "let X, Y be object;\n  thus"),
        ("holds X synalt Y", "holds Missing synalt Y"),
        ("thus X synalt Y", "thus X synalt Missing"),
        ("holds X synalt Y = X", "holds X synalt Y = Missing"),
        ("= X by", "= Missing by"),
        ("by SynBaseDef", "by SynUse1"),
        ("coherence;", "existence;"),
        (
            "synonym X synalt Y for X synbase Y",
            "synonym X synalt X for X synbase Y",
        ),
    ] {
        let changed = text.replace(from, to);
        assert_ne!(changed, text);
        let (source, typed, symbols) = inputs(&changed);
        assert!(
            check_source_functor_synonym_types(&source, &symbols, &typed).is_err(),
            "{changed}"
        );
    }
    let (source, typed, symbols) = inputs(&text);
    let original = symbols
        .symbols()
        .iter()
        .find(|entry| entry.kind() == SymbolKind::Functor)
        .unwrap();
    let alias = symbols
        .symbols()
        .iter()
        .find(|entry| entry.kind() == SymbolKind::Synonym)
        .unwrap();
    for relations in [
        vec![],
        vec![RelationMetadata::new(
            RelationKind::Antonym,
            original.symbol().clone(),
        )],
        vec![RelationMetadata::new(
            RelationKind::Synonym,
            alias.symbol().clone(),
        )],
    ] {
        let mut indexes = super::import_fixtures::clone_symbol_env_indexes(&symbols);
        indexes.symbols = SymbolIndex::new();
        for entry in symbols.symbols().iter() {
            indexes.symbols.insert(if entry.symbol() == alias.symbol() {
                entry.clone().with_relations(relations.clone())
            } else {
                entry.clone()
            });
        }
        assert!(
            check_source_functor_synonym_types(
                &source,
                &SymbolEnv::new(symbols.module_id().clone(), indexes),
                &typed
            )
            .is_err()
        );
    }
    let raw = typed
        .iter()
        .map(|(_, node)| node.clone())
        .collect::<Vec<_>>();
    let calls = typed
        .iter()
        .filter(|(_, node)| node.kind.as_str().starts_with("InfixExpression("))
        .map(|(id, _)| id)
        .collect::<Vec<_>>();
    assert_eq!(calls.len(), 2);
    for call in &calls {
        for argument in [raw[call.index()].children[0], raw[call.index()].children[2]] {
            for mutation in 0..3 {
                let mut nodes = raw.clone();
                match mutation {
                    0 => nodes[argument.index()].resolved_node = nodes[call.index()].resolved_node,
                    1 => nodes[argument.index()].anchor = nodes[call.index()].anchor.clone(),
                    2 => nodes[argument.index()].typing = TypingState::Successful,
                    _ => unreachable!(),
                }
                assert!(
                    check_source_functor_synonym_types(
                        &source,
                        &symbols,
                        &TypedArena::try_new(typed.root(), nodes).unwrap()
                    )
                    .is_err()
                );
            }
        }
    }
    let ast = super::formula_statement::step5c8_test_frontend(&text)
        .ast
        .unwrap();
    let head_offsets = calls
        .iter()
        .map(|id| match raw[raw[id.index()].children[1].index()].anchor {
            SourceAnchor::Range(range) => range.start,
            _ => unreachable!(),
        })
        .collect::<Vec<_>>();
    for target in head_offsets.into_iter().map(Some).chain([None]) {
        let mut builder = SurfaceAstBuilder::new(ast.source_id);
        let mut rebuilt = Vec::new();
        let mut changed = 0;
        for node in ast.nodes() {
            let mut children = node
                .children
                .iter()
                .map(|id| rebuilt[id.index()])
                .collect::<Vec<_>>();
            let id = match &node.kind {
                K::Token(token) => {
                    let spelling = if Some(node.range.start) == target {
                        assert_eq!(token.text.as_ref(), "synalt");
                        changed += 1;
                        "absent".into()
                    } else {
                        token.text.clone()
                    };
                    builder.add_token(token.kind, spelling, node.range)
                }
                K::InfixExpression(operator)
                    if node
                        .children
                        .get(1)
                        .is_some_and(|id| Some(ast.node(*id).unwrap().range.start) == target) =>
                {
                    let mut operator = operator.clone();
                    operator.spelling = "absent".into();
                    builder.add_node(K::InfixExpression(operator), node.range, children)
                }
                kind => {
                    if target.is_none() && kind == &K::ItemList {
                        assert_eq!(children.len(), 3);
                        children.swap(0, 1);
                        changed += 1;
                    }
                    builder.add_node(kind.clone(), node.range, children)
                }
            };
            rebuilt.push(id);
        }
        assert_eq!(changed, 1);
        let mut frontend = super::formula_statement::step5c8_test_frontend(&text);
        frontend.ast = Some(builder.finish(Some(rebuilt[ast.root().unwrap().index()]), None));
        let (changed_source, changed_typed, changed_symbols) =
            super::source_registration_inputs(&config.workspace_root, case, frontend).unwrap();
        assert!(
            check_source_functor_synonym_types(&changed_source, &changed_symbols, &changed_typed)
                .is_err()
        );
    }
    let foreign = ResolverModuleId::new(PackageId::new("foreign"), ModulePath::new("synonym"));
    assert!(
        check_source_functor_synonym_types(
            &SurfaceResolvedArena::lower(&ast, &foreign).unwrap(),
            &symbols,
            &typed
        )
        .is_err()
    );
    let mut other_ast = ast.clone();
    let ids = InMemorySessionIdAllocator::new();
    ids.next_source_id(snapshot_id(0)).unwrap();
    other_ast.source_id = ids.next_source_id(snapshot_id(0)).unwrap();
    assert!(
        check_source_functor_synonym_types(
            &SurfaceResolvedArena::lower(&other_ast, symbols.module_id()).unwrap(),
            &symbols,
            &typed
        )
        .is_err()
    );
    let (_, _, other_symbols) = inputs(&text.replace("synalt", "otheralias"));
    assert!(check_source_functor_synonym_types(&source, &other_symbols, &typed).is_err());
    let mut indexes = super::import_fixtures::clone_symbol_env_indexes(&symbols);
    indexes.definitions = Default::default();
    assert!(
        check_source_functor_synonym_types(
            &source,
            &SymbolEnv::new(symbols.module_id().clone(), indexes),
            &typed
        )
        .is_err()
    );
    let theorem = text.find("theorem SynUse1:").unwrap();
    let frontend =
        super::formula_statement::step5c8_test_frontend(&format!("{text}\n{}", &text[theorem..]));
    assert!(frontend.diagnostics.is_empty());
    assert!(
        !super::resolver_symbol_collection(
            &config.workspace_root,
            case,
            frontend.ast.as_ref().unwrap()
        )
        .detail_keys
        .is_empty()
    );
    assert!(super::source_registration_inputs(&config.workspace_root, case, frontend).is_err());
    let frontend = super::formula_statement::step5c8_test_frontend(&text.replace("= X by", "= by"));
    assert!(!frontend.diagnostics.is_empty());
    assert!(super::source_registration_inputs(&config.workspace_root, case, frontend).is_err());
}

#[test]
fn step5c6_functor_synonym_relation_selects_actual_target_and_rejects_unsupported_sources() {
    use mizar_checker::type_checker::check_source_functor_synonym_types;
    use mizar_resolve::env::{RelationKind, SymbolKind};
    let config = step5c11_config();
    let plan = build_test_plan(&config).unwrap();
    let case = plan
        .cases
        .iter()
        .find(|case| case.id.0 == "pass_type_elaboration_synonym_functor_001")
        .unwrap();
    let text = std::fs::read_to_string(&case.source_path).unwrap();
    let blocks = text.split("\n\n").collect::<Vec<_>>();
    let second = blocks[0]
        .replace("SynBaseDef", "SecondDef")
        .replace("synbase", "secondbase");
    let two_targets = format!(
        "{}\n\n{}\n\n{}\n\n{}",
        blocks[0],
        second,
        blocks[1].replace("synbase", "secondbase"),
        blocks[2]
    );
    let frontend = super::formula_statement::step5c8_test_frontend(&two_targets);
    assert!(
        frontend.diagnostics.is_empty(),
        "{:?}",
        frontend.diagnostics
    );
    let (source, typed, symbols) =
        super::source_registration_inputs(&config.workspace_root, case, frontend).unwrap();
    let mut originals = symbols
        .symbols()
        .iter()
        .filter(|entry| entry.kind() == SymbolKind::Functor)
        .collect::<Vec<_>>();
    originals.sort_by_key(|entry| match entry.origin().anchor() {
        SourceAnchor::Range(range) => range.start,
        _ => unreachable!(),
    });
    assert_eq!(originals.len(), 2);
    assert_ne!(originals[0].symbol(), originals[1].symbol());
    let alias = symbols
        .symbols()
        .iter()
        .find(|entry| entry.kind() == SymbolKind::Synonym)
        .unwrap();
    assert_eq!(alias.relations().len(), 1);
    assert_eq!(alias.relations()[0].kind(), RelationKind::Synonym);
    assert_eq!(alias.relations()[0].target(), originals[1].symbol());
    assert!(check_source_functor_synonym_types(&source, &symbols, &typed).is_err());
    let predicate =
        "definition\n let X, Y be set;\n pred SynBaseDef: X synbase Y means X = Y;\nend;";
    for variant in [
        format!(
            "{}\n{}\n{}",
            blocks[0],
            blocks[0].replace("SynBaseDef", "DuplicateDef"),
            blocks[1]
        ),
        format!(
            "{}\n{}",
            blocks[0],
            blocks[1].replace("for X synbase Y", "for X missing Y")
        ),
        format!("{}\n{}", blocks[1], blocks[0]),
        format!("{predicate}\n{}", blocks[1]),
        format!(
            "{}\n{}\n{}",
            blocks[0],
            blocks[1],
            blocks[1]
                .replace("synalt", "thirdname")
                .replace("synbase", "synalt")
        ),
        format!(
            "{}\n{}",
            blocks[0],
            blocks[1]
                .replace("X, Y be set", "X, Y, Z be set")
                .replace("for X synbase Y", "for X synbase Z")
        ),
    ] {
        let frontend = super::formula_statement::step5c8_test_frontend(&variant);
        assert!(
            frontend.diagnostics.is_empty(),
            "{variant}: {:?}",
            frontend.diagnostics
        );
        let result = super::resolver_symbol_collection(
            &config.workspace_root,
            case,
            frontend.ast.as_ref().unwrap(),
        );
        let last_alias = result
            .env
            .symbols()
            .iter()
            .filter(|entry| entry.kind() == SymbolKind::Synonym)
            .max_by_key(|entry| match entry.origin().anchor() {
                SourceAnchor::Range(range) => range.start,
                _ => unreachable!(),
            })
            .unwrap();
        assert!(
            last_alias.relations().is_empty(),
            "{variant}: {:?}",
            last_alias.relations()
        );
    }
}


fn step5c14_state_case() -> crate::harness::TestCase {
    build_test_plan(&step5c11_config())
        .unwrap()
        .cases
        .into_iter()
        .find(|case| case.id.0 == "pass_proof_verification_algorithm_var_const_assert_001")
        .unwrap()
}

fn step5c14_state_vcs(
    core: &mizar_core::core_ir::CoreIr,
) -> Result<mizar_vc::vc_ir::VcSet, String> {
    mizar_vc::generator::generate_source_algorithm_postconditions(
        core,
        super::shared::snapshot_id(0),
        &mizar_vc::vc_ir::GenerationSchemaVersion::new("mizar-vc-generation-step5c14-state-v1"),
        &mizar_vc::vc_ir::VcSchemaVersion::new("mizar-vc-vcset-step5c14-state-v1"),
    )
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

fn step5c11_reduce_case() -> (crate::harness::TestCase, String) {
    let case = build_test_plan(&step5c11_config())
        .unwrap()
        .cases
        .into_iter()
        .find(|case| case.id.0 == "fail_proof_verification_reduce_false_reducibility_001")
        .unwrap();
    let text = std::fs::read_to_string(&case.source_path).unwrap();
    (case, text)
}

#[test]
fn step5c11_reduce_retains_the_actual_guarded_goal_proof_and_complete_vc() {
    use mizar_core::core_ir::{
        CoreFormulaKind as F, CoreProofNodeKind, CoreProofStatus, CoreSourceAnchor,
        CoreTermKind as T, DefinitionBody,
    };
    use mizar_vc::{
        discharge::failed_functorial_coherence,
        vc_ir::{RegistrationCorrectnessKind, VcKind, VcStatus},
    };
    let (case, text) = step5c11_reduce_case();
    for (source, singleton, baseline) in [
        (
            text.clone(),
            true,
            Some("fail_proof_verification_reduce_false_reducibility_001.vc_ir.snap"),
        ),
        (
            text.replace("to {X}", "to X"),
            false,
            Some("step5c11_reduce_reflexive_rhs.vc_ir.snap"),
        ),
        (
            text.replace("CBox4Def", "IdentityDefinition")
                .replace("CRed2", "ReductionLabel")
                .replace("cbox4", "identity_fun")
                .replace('X', "Subject"),
            true,
            None,
        ),
        (
            text.replace("cbox4 (cbox4 X) to {X}", "cbox4 ((cbox4 X)) to {(X)}"),
            true,
            None,
        ),
    ] {
        let frontend = super::formula_statement::step5c8_test_frontend(&source);
        assert!(
            frontend.diagnostics.is_empty(),
            "{:?}",
            frontend.diagnostics
        );
        let core = step5c11_functorial_core(&case, &source).unwrap();
        assert_eq!(core, step5c11_functorial_core(&case, &source).unwrap());
        assert_eq!(
            (
                core.items().len(),
                core.definitions().len(),
                core.proofs().len(),
                core.proof_nodes().len(),
                core.obligation_seeds().len()
            ),
            (2, 1, 1, 1, 1)
        );
        let (_, definition) = core.definitions().iter().next().unwrap();
        let (_, seed) = core.obligation_seeds().iter().next().unwrap();
        let goal = seed.goal.unwrap();
        let F::Forall { binders, body } = &core.formulas().get(goal).unwrap().kind else {
            panic!("original universal goal")
        };
        assert_eq!(binders.len(), 1);
        assert_eq!(definition.params.len(), 1);
        assert_ne!(binders[0].var, definition.params[0].var);
        let F::Implies {
            premise,
            conclusion,
        } = core.formulas().get(*body).unwrap().kind
        else {
            panic!("original guard")
        };
        let F::TypePred { subject, ref ty } = core.formulas().get(premise).unwrap().kind else {
            panic!("bare set guard")
        };
        assert_eq!(ty.as_str(), "set");
        assert_eq!(
            core.terms().get(subject).unwrap().kind,
            T::Var(binders[0].var)
        );
        let F::Equals { left, right } = core.formulas().get(conclusion).unwrap().kind else {
            panic!("original equality")
        };
        let T::Apply {
            ref functor,
            ref args,
        } = core.terms().get(left).unwrap().kind
        else {
            panic!("outer application")
        };
        assert_eq!(functor, &definition.symbol);
        assert_eq!(args.len(), 1);
        let T::Apply {
            functor: ref inner_functor,
            args: ref inner_args,
        } = core.terms().get(args[0]).unwrap().kind
        else {
            panic!("inner application")
        };
        assert_eq!(inner_functor, &definition.symbol);
        assert_eq!(inner_args, &[subject]);
        let rhs = if singleton {
            let T::SetEnum(ref elements) = core.terms().get(right).unwrap().kind else {
                panic!("actual singleton")
            };
            assert_eq!(elements.len(), 1);
            elements[0]
        } else {
            right
        };
        assert_eq!(core.terms().get(rhs).unwrap().kind, T::Var(binders[0].var));
        assert_ne!(rhs, subject, "two real source occurrences");
        let DefinitionBody::Term(definiens) = definition.body else {
            panic!("real identity body")
        };
        assert_eq!(
            core.terms().get(definiens).unwrap().kind,
            T::Var(definition.params[0].var)
        );
        let proof = core.proofs().iter().next().unwrap().1;
        assert_eq!(proof.item, seed.owner);
        assert_eq!(proof.proposition, goal);
        assert_eq!(proof.status, CoreProofStatus::PendingAutomaticProof);
        let step = core.proof_nodes().get(proof.root).unwrap();
        let CoreProofNodeKind::Step {
            label,
            formula,
            justification,
        } = &step.kind
        else {
            panic!("real thesis step")
        };
        assert!(label.is_none() && justification.citations.is_empty());
        assert_eq!(*formula, goal);
        for (actual, kind) in [
            (
                &proof.source,
                mizar_syntax::ast::SurfaceNodeKind::ProofBlock,
            ),
            (
                &step.source,
                mizar_syntax::ast::SurfaceNodeKind::ConclusionStatement,
            ),
        ] {
            let expected = frontend
                .ast
                .as_ref()
                .unwrap()
                .nodes()
                .iter()
                .find(|node| node.kind == kind)
                .unwrap()
                .range;
            let CoreSourceAnchor::SourceRange(range) = actual.anchor else {
                panic!("actual source range")
            };
            assert_eq!((range.start, range.end), (expected.start, expected.end));
        }
        let vcs =
            super::proof_verification::generate_core_vcs(&core, super::shared::snapshot_id(0))
                .unwrap();
        let replay =
            super::proof_verification::generate_core_vcs(&core, super::shared::snapshot_id(0))
                .unwrap();
        assert_eq!(vcs, replay);
        assert_eq!(vcs.debug_text(), replay.debug_text());
        assert_eq!((vcs.vcs().len(), vcs.seed_accounting().len()), (1, 1));
        assert_eq!(
            vcs.vcs()[0].kind,
            VcKind::RegistrationStyleCorrectness {
                style: RegistrationCorrectnessKind::Reduction
            }
        );
        assert_eq!(vcs.vcs()[0].status, VcStatus::Open);
        assert!(seed.label.is_none());
        assert!(vcs.vcs()[0].premises.is_empty());
        assert!(vcs.vcs()[0].proof_hint.is_none());
        assert_eq!(
            failed_functorial_coherence(&core, &vcs).unwrap(),
            singleton.then_some(vcs.vcs()[0].id)
        );
        let (database, _) = step5c11_check_source(&case, &source).unwrap();
        assert!(database.activated().is_empty() && database.rejected().is_empty());
        assert_eq!(database.pending().len(), 1);
        assert!(
            database
                .pending()
                .iter()
                .all(|row| !row.may_contribute_to_inference())
        );
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
fn step5c11_reduce_source_controls_cannot_borrow_the_failure_key() {
    let (case, text) = step5c11_reduce_case();
    for (old, new) in [
        ("equals X", "equals {X}"),
        ("let X be set;", "let X be object;"),
        ("to {X}", "to {X, X}"),
        ("to {X}", "to {}"),
        ("to {X}", "to {Y}"),
        ("thus thesis;", "thus thesis by CBox4Def;"),
        ("thus thesis;", "thus L: thesis;"),
        ("thus thesis;", "thus thesis; thus thesis;"),
        (
            "reducibility\n  proof\n    thus thesis;\n  end;",
            "reducibility;",
        ),
        (
            "registration\n",
            "definition\n  let Y be set;\nend;\nregistration\n",
        ),
    ] {
        let changed = text.replace(old, new);
        assert_ne!(changed, text);
        let frontend = super::formula_statement::step5c8_test_frontend(&changed);
        assert!(
            frontend.diagnostics.is_empty(),
            "{old} => {new}: {:?}",
            frontend.diagnostics
        );
        if let Ok(core) = step5c11_functorial_core(&case, &changed) {
            let vcs =
                super::proof_verification::generate_core_vcs(&core, super::shared::snapshot_id(0))
                    .unwrap();
            assert!(
                mizar_vc::discharge::failed_functorial_coherence(&core, &vcs).is_err(),
                "unsupported {old} => {new}"
            );
        }
    }
    let recovered = text.replace("thus thesis;", "assume X = X; thus thesis;");
    assert!(
        !super::formula_statement::step5c8_test_frontend(&recovered)
            .diagnostics
            .is_empty()
    );
    assert!(step5c11_functorial_core(&case, &recovered).is_err());
    let orientation = text.replace("cbox4 (cbox4 X)", "cbox4 X");
    assert!(
        super::formula_statement::step5c8_test_frontend(&orientation)
            .diagnostics
            .is_empty()
    );
    assert!(
        step5c11_check_source(&case, &orientation).is_err(),
        "equal size is not reducibility proof failure"
    );
    let (original, nodes, symbols) = super::source_registration_inputs(
        &step5c11_config().workspace_root,
        &case,
        super::formula_statement::step5c8_test_frontend(&text),
    )
    .unwrap();
    let changed = text.replace("CBox4Def", "ForeignDef");
    let (foreign, _, _) = super::source_registration_inputs(
        &step5c11_config().workspace_root,
        &case,
        super::formula_statement::step5c8_test_frontend(&changed),
    )
    .unwrap();
    assert!(
        mizar_checker::registration_resolution::check_source_registration_intake(
            &foreign, &nodes, &symbols
        )
        .is_err()
    );
    let mut rows = nodes
        .iter()
        .map(|(_, node)| node.clone())
        .collect::<Vec<_>>();
    rows[0].kind = "ForeignToken".into();
    let recovered_nodes =
        mizar_checker::typed_ast::TypedArena::try_new(nodes.root(), rows).unwrap();
    assert!(
        mizar_checker::registration_resolution::check_source_registration_intake(
            &original,
            &recovered_nodes,
            &symbols
        )
        .is_err()
    );
}

#[test]
fn step5c11_reduce_refutation_rejects_foreign_goals_proofs_and_vcs() {
    use mizar_core::core_ir::{
        CoreCitation, CoreFormulaKind as F, CoreIr, CoreIrParts, CoreLabelRef, CoreProofNodeKind,
        CoreProofStatus, CoreTermKind as T, DefinitionBody,
    };
    use mizar_vc::{
        discharge::failed_functorial_coherence,
        vc_ir::{RegistrationCorrectnessKind, VcFormulaRef, VcKind, VcSet, VcSetParts, VcStatus},
    };
    let (case, text) = step5c11_reduce_case();
    let core = step5c11_functorial_core(&case, &text).unwrap();
    let original_vcs =
        super::proof_verification::generate_core_vcs(&core, super::shared::snapshot_id(0)).unwrap();
    let (seed_id, seed) = core.obligation_seeds().iter().next().unwrap();
    let goal = seed.goal.unwrap();
    let F::Forall { body, .. } = core.formulas().get(goal).unwrap().kind else {
        unreachable!()
    };
    let F::Implies {
        premise,
        conclusion,
    } = core.formulas().get(body).unwrap().kind
    else {
        unreachable!()
    };
    let F::Equals { left, right } = core.formulas().get(conclusion).unwrap().kind else {
        unreachable!()
    };
    let T::Apply { ref args, .. } = core.terms().get(left).unwrap().kind else {
        unreachable!()
    };
    let inner = args[0];
    let T::SetEnum(ref elements) = core.terms().get(right).unwrap().kind else {
        unreachable!()
    };
    let rhs = elements[0];
    let (definition_id, definition) = core.definitions().iter().next().unwrap();
    let (proof_id, proof) = core.proofs().iter().next().unwrap();
    for mutation in 0..15 {
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
            0 => parts.formulas.get_mut(premise).unwrap().kind = F::False,
            1 => {
                let F::TypePred { ty, .. } = &mut parts.formulas.get_mut(premise).unwrap().kind
                else {
                    unreachable!()
                };
                *ty = "object".into();
            }
            2 => {
                let F::TypePred { ty, .. } = &mut parts
                    .formulas
                    .get_mut(definition.params[0].ty_guard.unwrap())
                    .unwrap()
                    .kind
                else {
                    unreachable!()
                };
                *ty = "object".into();
            }
            3 => {
                let T::Apply { functor, .. } = &mut parts.terms.get_mut(inner).unwrap().kind else {
                    unreachable!()
                };
                *functor = core.items().get(seed.owner).unwrap().symbol.clone();
            }
            4 => {
                let T::Apply { args, .. } = &mut parts.terms.get_mut(inner).unwrap().kind else {
                    unreachable!()
                };
                args[0] = rhs;
            }
            5 => parts.terms.get_mut(rhs).unwrap().kind = T::Var(definition.params[0].var),
            6 => {
                let F::TypePred { subject, .. } = parts
                    .formulas
                    .get(definition.params[0].ty_guard.unwrap())
                    .unwrap()
                    .kind
                else {
                    unreachable!()
                };
                parts.definitions.get_mut(definition_id).unwrap().body =
                    DefinitionBody::Term(subject);
            }
            7 => parts.proofs.get_mut(proof_id).unwrap().proposition = conclusion,
            8 => parts.proofs.get_mut(proof_id).unwrap().status = CoreProofStatus::Open,
            9 => parts.proofs.get_mut(proof_id).unwrap().item = definition.owner.item().unwrap(),
            10 => {
                let CoreProofNodeKind::Step { justification, .. } =
                    &mut parts.proof_nodes.get_mut(proof.root).unwrap().kind
                else {
                    unreachable!()
                };
                justification
                    .citations
                    .push(CoreCitation::Label("foreign".into()));
            }
            11 => parts
                .obligation_seeds
                .get_mut(seed_id)
                .unwrap()
                .context
                .push(premise),
            12 => {
                parts.proofs.get_mut(proof_id).unwrap().source =
                    core.items().get(seed.owner).unwrap().source.clone()
            }
            13 => {
                let changed = definition.source.clone();
                parts.proof_nodes.get_mut(proof.root).unwrap().source = changed.clone();
                parts.source_map.proof_sources.insert(proof.root, changed);
            }
            14 => {
                parts.obligation_seeds.get_mut(seed_id).unwrap().label = Some(CoreLabelRef::new(
                    core.items().get(seed.owner).unwrap().symbol.fqn().as_str(),
                ));
            }
            _ => unreachable!(),
        }
        let changed = CoreIr::try_new(parts).unwrap();
        let generated =
            super::proof_verification::generate_core_vcs(&changed, super::shared::snapshot_id(0))
                .unwrap();
        assert!(
            failed_functorial_coherence(&changed, &generated).is_err(),
            "Core mutation {mutation}"
        );
        assert!(
            failed_functorial_coherence(&changed, &original_vcs).is_err(),
            "stale VC {mutation}"
        );
    }
    for mutation in 0..4 {
        let mut parts = VcSetParts {
            schema_version: original_vcs.schema_version().clone(),
            snapshot: original_vcs.snapshot(),
            source: original_vcs.source(),
            module: original_vcs.module().clone(),
            generated_formulas: original_vcs.generated_formulas().to_vec(),
            vcs: original_vcs.vcs().to_vec(),
            seed_accounting: original_vcs.seed_accounting().to_vec(),
        };
        match mutation {
            0 => {
                parts.vcs[0].kind = VcKind::RegistrationStyleCorrectness {
                    style: RegistrationCorrectnessKind::Registration,
                }
            }
            1 => parts.vcs[0].status = VcStatus::NeedsAtp,
            2 => parts.vcs[0].goal = VcFormulaRef::Core(conclusion),
            3 => parts.vcs[0].source.primary = definition.source.clone(),
            _ => unreachable!(),
        }
        let changed = VcSet::try_new(parts).unwrap();
        assert!(
            failed_functorial_coherence(&core, &changed).is_err(),
            "VC mutation {mutation}"
        );
    }
}

#[test]
fn step5c11_reduce_runner_requires_the_complete_committed_baseline() {
    let config = step5c11_config();
    let (case, _) = step5c11_reduce_case();
    let run = |tests_root: &Path, ordinal| {
        super::run_proof_verification_case(&config.workspace_root, tests_root, &case, ordinal)
    };
    for ordinal in [0, 73] {
        let result = run(&config.workspace_root.join("tests"), ordinal);
        assert_eq!(
            result.status,
            super::ProofVerificationCaseStatus::Passed,
            "{result:?}"
        );
    }
    let temporary = std::process::Command::new("mktemp")
        .arg("-d")
        .output()
        .unwrap();
    assert!(temporary.status.success());
    let root = PathBuf::from(String::from_utf8(temporary.stdout).unwrap().trim());
    let missing = run(&root, 0);
    assert_eq!(missing.status, super::ProofVerificationCaseStatus::Failed);
    assert!(
        missing
            .failure
            .unwrap()
            .contains("snapshot could not be read")
    );
    let baseline = root.join(case.expectation.snapshots.as_ref().unwrap());
    std::fs::create_dir_all(baseline.parent().unwrap()).unwrap();
    std::fs::write(baseline, "vc-ir-debug-v1\ntruncated\n").unwrap();
    let corrupt = run(&root, 0);
    assert_eq!(corrupt.status, super::ProofVerificationCaseStatus::Failed);
    assert!(corrupt.failure.unwrap().contains("snapshot differed"));
    std::fs::remove_dir_all(root).unwrap();
}

fn step5c14_claim_case() -> crate::harness::TestCase {
    build_test_plan(&step5c11_config())
        .unwrap()
        .cases
        .into_iter()
        .find(|case| case.id.0 == "pass_proof_verification_claim_block_theorem_001")
        .unwrap()
}

fn step5c14_claim_core(
    case: &crate::harness::TestCase,
    text: &str,
) -> Result<mizar_core::core_ir::CoreIr, String> {
    use mizar_checker::type_checker::SourceVariableSemanticsChecker;
    use mizar_resolve::{
        labels::{LabelResolver, ProofLabelSourceCollector},
        names::{SourceVariableScopeInput, SourceVariableScopeResolver},
    };
    let frontend = super::formula_statement::step5c8_test_frontend(text);
    let ast = frontend.ast.clone().ok_or("missing AST")?;
    let (source, nodes, symbols) =
        super::source_registration_inputs(&step5c11_config().workspace_root, case, frontend)?;
    let algorithm =
        mizar_checker::type_checker::check_source_algorithm_types(&source, &nodes, &symbols)?;
    let scope = SourceVariableScopeResolver::resolve_proof_occurrences(
        SourceVariableScopeInput::new(&ast, source.module(), &symbols),
    )
    .map_err(|error| format!("scope: {error:?}"))?;
    let bindings = SourceVariableSemanticsChecker::occurrence_binding_env(&scope);
    let typed = super::type_elaboration::step5c8_formula_typed_ast(
        &ast,
        source.module(),
        &symbols,
        &scope,
        &bindings,
        true,
    )?;
    let owner = symbols
        .symbols()
        .iter()
        .find(|entry| entry.kind() == mizar_resolve::env::SymbolKind::Theorem)
        .ok_or("theorem missing")?;
    let namespace = mizar_resolve::env::NamespacePath::new(source.module().path().as_str());
    let labels = ProofLabelSourceCollector::new(
        &ast,
        source.module(),
        namespace.clone(),
        owner.contribution(),
        &source,
    )
    .and_then(|collector| collector.collect_with_theorem_owners(&symbols))
    .map_err(|error| error.to_string())?;
    let resolved = LabelResolver::new(labels.projections()).resolve(
        source.module(),
        &namespace,
        labels.references(),
    );
    let checked = SourceVariableSemanticsChecker::check_theorem_skeletons(
        &typed,
        &scope,
        &symbols,
        &labels,
        &resolved,
        Some(&algorithm),
    )?;
    assert!(
        SourceVariableSemanticsChecker::check_theorem_skeletons(
            &typed, &scope, &symbols, &labels, &resolved, None
        )
        .is_err()
    );
    assert!(mizar_core::elaborator::lower_source_theorem_skeletons(&checked, None).is_err());
    mizar_core::elaborator::lower_source_theorem_skeletons(&checked, Some(&algorithm))
}

fn step5c14_claim_vcs(
    core: &mizar_core::core_ir::CoreIr,
) -> Result<mizar_vc::vc_ir::VcSet, String> {
    mizar_vc::generator::generate_source_void_claim(
        core,
        super::shared::snapshot_id(0),
        &mizar_vc::vc_ir::GenerationSchemaVersion::new("mizar-vc-generation-step5c14-claim-v1"),
        &mizar_vc::vc_ir::VcSchemaVersion::new("mizar-vc-vcset-step5c14-claim-v1"),
    )
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
        let ast = super::formula_statement::step5c8_test_frontend(&source)
            .ast
            .unwrap();
        let resolver =
            super::resolver_symbol_collection(&step5c11_config().workspace_root, &case, &ast);
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
        let frontend = super::formula_statement::step5c8_test_frontend(&changed);
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
        let frontend = super::formula_statement::step5c8_test_frontend(&changed);
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
    let frontend = super::formula_statement::step5c8_test_frontend(&malformed);
    assert!(!frontend.diagnostics.is_empty());
    let error = step5c14_claim_core(&case, &malformed).unwrap_err();
    assert!(
        error == "registration.frontend_diagnostics" || error == "missing AST",
        "{error}"
    );
    let duplicate = format!("{algorithm}\n{source}");
    assert!(
        super::formula_statement::step5c8_test_frontend(&duplicate)
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
    let original = step5c14_claim_case();
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
            !super::proof_verification::step5c14_return_admitted(
                Some(&config.workspace_root),
                &case
            ),
            "mutation {mutation}"
        );
        assert!(!super::is_active_proof_verification(&case));
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
                !super::is_active_proof_verification(&case),
                "{donor} alias {alias}"
            );
            let run = super::proof_verification::run_proof_verification_case(
                &config.workspace_root,
                &config.workspace_root.join("tests"),
                &case,
                0,
            );
            assert_eq!(run.status, super::ProofVerificationCaseStatus::Failed);
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
    let frontend = super::formula_statement::step5c8_test_frontend(&text);
    let ast = frontend.ast.clone().unwrap();
    let (source, nodes, symbols) =
        super::source_registration_inputs(&root, &case, frontend).unwrap();
    let algorithm = check_source_algorithm_types(&source, &nodes, &symbols).unwrap();
    let scope = SourceVariableScopeResolver::resolve_proof_occurrences(
        SourceVariableScopeInput::new(&ast, source.module(), &symbols),
    )
    .unwrap();
    let bindings = SourceVariableSemanticsChecker::occurrence_binding_env(&scope);
    let typed = super::type_elaboration::step5c8_formula_typed_ast(
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
            let snapshot = super::shared::snapshot_id(5814);
            ids.next_source_id(snapshot).unwrap();
            let output = Frontend::new(
                FrontendSourceLoader::new(DiskSourceLoader::new(&directory)),
                super::ParseOnlyImportProvider,
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
            super::shared::FrontendRun {
                source_text: alternate_text.clone().into(),
                ast: output.ast,
                ast_snapshot: None,
                diagnostics: output.diagnostics,
            }
        } else {
            super::formula_statement::step5c8_test_frontend(&alternate_text)
        };
        let alternate_ast = frontend.ast.clone().unwrap();
        let (alternate_source, alternate_nodes, alternate_symbols) =
            super::source_registration_inputs(&root, &alternate_case, frontend).unwrap();
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
        let alternate_typed = super::type_elaboration::step5c8_formula_typed_ast(
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

fn step5c5_functor_property_case() -> (crate::harness::TestCase, String) {
    let case = build_test_plan(&step5c11_config())
        .unwrap()
        .cases
        .into_iter()
        .find(|case| case.id.0 == "pass_type_elaboration_func_commutativity_property_001")
        .unwrap();
    let text = std::fs::read_to_string(&case.source_path).unwrap();
    (case, text)
}

#[test]
fn step5c5_functor_property_checks_body_bindings_and_retains_only_a_pending_obligation() {
    use mizar_checker::{
        binding_env::BindingKind,
        type_checker::{
            NormalizedTypeStatus, TermFormulaChecker, TermKind, TermReference, TermStatus,
            TypeHeadRef,
        },
        typed_ast::{InitialObligationKind, InitialObligationStatus, TypeEntryActual, TypeStatus},
    };
    let config = step5c11_config();
    let (case, text) = step5c5_functor_property_case();
    for (variant, body_order) in [
        (text.clone(), [0, 1]),
        (text.replace("PairJoinDef", "UnionRule"), [0, 1]),
        (text.replace("X", "Left").replace("Y", "Right"), [0, 1]),
        (text.replace(r"\*\", r"\+\"), [0, 1]),
        (text.replace("{X, Y}", "{Y, X}"), [1, 0]),
        (text.replace(r"X \*\ Y ->", r"Y \*\ X ->"), [0, 1]),
        (text.replace("{X, Y}", "{X, X}"), [0, 0]),
    ] {
        let frontend = super::formula_statement::step5c8_test_frontend(&variant);
        assert!(
            frontend.diagnostics.is_empty(),
            "{variant}: {:?}",
            frontend.diagnostics
        );
        let (source, typed, symbols) =
            super::source_registration_inputs(&config.workspace_root, &case, frontend).unwrap();
        let result =
            TermFormulaChecker::check_source_functor_property(&source, &symbols, &typed).unwrap();
        assert_eq!(
            result,
            TermFormulaChecker::check_source_functor_property(&source, &symbols, &typed).unwrap()
        );
        let (bindings, checked, obligations) = result;
        assert!(bindings.diagnostics().is_empty() && checked.diagnostics().is_empty());
        assert!(
            checked.formulas().is_empty()
                && checked.facts().is_empty()
                && checked.candidate_sets().is_empty()
        );
        let formals = bindings
            .bindings()
            .iter()
            .map(|(_, entry)| entry)
            .collect::<Vec<_>>();
        assert_eq!(formals.len(), 2);
        for formal in &formals {
            assert_eq!(formal.kind, BindingKind::DefinitionParameter);
            assert_eq!(
                &variant[formal.declaration_range.start..formal.declaration_range.end],
                formal.spelling
            );
            assert!(formal.declaration_range.end < variant.find("func ").unwrap());
        }
        assert_ne!(formals[0].id, formals[1].id);
        assert_ne!(formals[0].identity, formals[1].identity);
        let terms = checked
            .terms()
            .iter()
            .map(|(_, term)| term)
            .collect::<Vec<_>>();
        assert_eq!(terms.len(), 3);
        for term in &terms {
            assert_eq!(term.status, TermStatus::Inferred);
            assert!(term.deferred.is_empty() && term.candidate_set.is_none());
            let TypeEntryActual::Known(id) =
                checked.type_entries().get(term.type_entry).unwrap().actual
            else {
                panic!("body type must be known")
            };
            assert_eq!(
                checked.type_entries().get(term.type_entry).unwrap().status,
                TypeStatus::Known
            );
            let normalized = checked.normalized_types().get(id).unwrap();
            assert_eq!(normalized.status, NormalizedTypeStatus::Known);
            assert_eq!(normalized.head, TypeHeadRef::BuiltinSet);
        }
        let elements = terms
            .iter()
            .filter(|term| term.kind == TermKind::Variable)
            .collect::<Vec<_>>();
        assert_eq!(elements.len(), 2);
        assert_ne!(elements[0].site, elements[1].site);
        for (element, formal_index) in elements.iter().zip(body_order) {
            assert_eq!(
                element.reference,
                Some(TermReference::Binding(formals[formal_index].id))
            );
            let SourceAnchor::Range(range) = typed.node(element.site.node()).unwrap().anchor else {
                panic!("real body occurrence")
            };
            assert_eq!(
                &variant[range.start..range.end],
                formals[formal_index].spelling
            );
        }
        let body = terms
            .iter()
            .find(|term| term.kind == TermKind::SetEnumeration)
            .unwrap();
        assert!(body.reference.is_none());
        let body_type = checked.type_entries().get(body.type_entry).unwrap();
        let TypeEntryActual::Known(actual_type) = body_type.actual else {
            unreachable!()
        };
        assert_eq!(body_type.expected, Some(actual_type));
        assert_eq!(body.expected_type, Some(actual_type));
        let SourceAnchor::Range(body_range) = typed.node(body.site.node()).unwrap().anchor else {
            unreachable!()
        };
        assert!(variant[body_range.start..body_range.end].starts_with('{'));
        for element in elements {
            let SourceAnchor::Range(range) = typed.node(element.site.node()).unwrap().anchor else {
                unreachable!()
            };
            assert!(body_range.start < range.start && range.end < body_range.end);
        }
        assert_eq!(obligations.len(), 1);
        let (_, obligation) = obligations.iter().next().unwrap();
        assert_eq!(
            obligation.kind,
            InitialObligationKind::FunctorPropertyCorrectness
        );
        assert_eq!(obligation.status, InitialObligationStatus::Pending);
        assert!(obligation.assumptions.is_empty());
        let (definition_id, definition) = source
            .arena()
            .iter()
            .find(|(_, node)| node.kind() == &mizar_syntax::SurfaceNodeKind::FunctorDefinition)
            .unwrap();
        let functor = symbols
            .symbols()
            .iter()
            .find(|entry| entry.origin().anchor() == definition.origin().anchor())
            .unwrap();
        let property_symbol = symbols
            .symbols()
            .iter()
            .find(|entry| entry.origin().anchor() == &SourceAnchor::Range(obligation.source_range))
            .unwrap();
        let pattern = source
            .arena()
            .iter()
            .find(|(_, node)| node.kind() == &mizar_syntax::SurfaceNodeKind::FunctorPattern)
            .unwrap()
            .1;
        let mut loci = Vec::new();
        for child in pattern.children() {
            let node = source.arena().node(*child).unwrap();
            if let mizar_syntax::SurfaceNodeKind::Token(token) = node.kind()
                && let Some(formal) = formals
                    .iter()
                    .find(|formal| formal.spelling == token.text.as_ref())
            {
                let declaration = source
                    .arena()
                    .iter()
                    .find(|(_, node)| {
                        node.origin().anchor() == &SourceAnchor::Range(formal.declaration_range)
                    })
                    .unwrap()
                    .0;
                loci.push(declaration.index());
            }
        }
        assert_eq!(loci.len(), 2);
        assert_ne!(loci[0], loci[1]);
        let return_type = definition
            .children()
            .iter()
            .find(|id| {
                source.arena().node(**id).unwrap().kind()
                    == &mizar_syntax::SurfaceNodeKind::TypeExpression
            })
            .unwrap();
        assert_eq!(
            obligation.goal.as_str(),
            format!(
                "source.functor.property.request:functor={}:property={}:loci={},{}:body={}:type={}",
                functor.symbol().fqn().as_str(),
                obligation.owner.node().index(),
                loci[0],
                loci[1],
                body.site.node().index(),
                return_type.index()
            )
        );
        assert_eq!(
            obligation.provenance.as_str(),
            format!(
                "source.functor.property:owner={}:node={}",
                property_symbol.symbol().fqn().as_str(),
                obligation.owner.node().index()
            )
        );
        assert_ne!(definition_id.index(), obligation.owner.node().index());
        let property = typed.node(obligation.owner.node()).unwrap();
        assert_eq!(property.kind.as_str(), "PropertyClause");
        assert_eq!(
            property.anchor,
            SourceAnchor::Range(obligation.source_range)
        );
        assert!(
            variant[obligation.source_range.start..obligation.source_range.end]
                .starts_with("commutativity")
        );
    }
}

#[test]
fn step5c5_functor_property_rejects_unsupported_bodies_and_foreign_owners() {
    use mizar_checker::type_checker::TermFormulaChecker;
    let config = step5c11_config();
    let (case, text) = step5c5_functor_property_case();
    for (before, after) in [
        (r#""\\*\\""#, r#""\\+\\""#),
        ("{X, Y}", "{Missing, Y}"),
        ("{X, Y}", "{X, Missing}"),
        ("{X, Y}", "[X, Y]"),
        ("{X, Y}", "{X}"),
        ("X \\*\\ Y ->", "X \\*\\ X ->"),
        ("-> set", "-> object"),
        ("X, Y be set", "X, Y be object"),
        ("commutativity", "idempotence"),
        (
            "commutativity\n  proof\n    thus thesis;\n  end;",
            "commutativity;",
        ),
        ("  commutativity\n  proof\n    thus thesis;\n  end;\n", ""),
    ] {
        let changed = text.replace(before, after);
        assert_ne!(changed, text);
        let frontend = super::formula_statement::step5c8_test_frontend(&changed);
        assert!(
            frontend.diagnostics.is_empty(),
            "{before} => {after}: {:?}",
            frontend.diagnostics
        );
        let outcome = super::source_registration_inputs(&config.workspace_root, &case, frontend)
            .and_then(|(source, typed, symbols)| {
                TermFormulaChecker::check_source_functor_property(&source, &symbols, &typed)
            });
        assert!(outcome.is_err(), "unsupported {before} => {after}");
    }
    let duplicate_binding = text.replace("let X, Y", "let X, X");
    let frontend = super::formula_statement::step5c8_test_frontend(&duplicate_binding);
    assert!(!frontend.diagnostics.is_empty());
    assert!(super::source_registration_inputs(&config.workspace_root, &case, frontend).is_err());
    let duplicate = format!("{text}\n{}", text.split("infix_operator").next().unwrap());
    let frontend = super::formula_statement::step5c8_test_frontend(&duplicate);
    assert!(frontend.diagnostics.is_empty());
    let resolution = super::resolver_symbol_collection(
        &config.workspace_root,
        &case,
        frontend.ast.as_ref().unwrap(),
    );
    assert!(!resolution.detail_keys.is_empty());
    assert!(super::source_registration_inputs(&config.workspace_root, &case, frontend).is_err());
    let malformed = text.replace("{X, Y}", "{X, Y");
    let frontend = super::formula_statement::step5c8_test_frontend(&malformed);
    assert!(!frontend.diagnostics.is_empty());
    assert!(super::source_registration_inputs(&config.workspace_root, &case, frontend).is_err());
}

#[test]
fn step5c5_functor_property_rejects_forged_source_typed_and_symbol_inputs() {
    use mizar_checker::{
        type_checker::TermFormulaChecker,
        typed_ast::{TypedArena, TypingState},
    };
    use mizar_resolve::{env::SymbolEnv, resolved_ast::SurfaceResolvedArena};
    let config = step5c11_config();
    let (case, text) = step5c5_functor_property_case();
    let inputs = |text: &str| {
        let frontend = super::formula_statement::step5c8_test_frontend(text);
        assert!(frontend.diagnostics.is_empty());
        super::source_registration_inputs(&config.workspace_root, &case, frontend).unwrap()
    };
    let (source, typed, symbols) = inputs(&text);
    let (foreign_source, _, foreign_symbols) = inputs(&text.replace("PairJoinDef", "AnotherRule"));
    assert!(
        TermFormulaChecker::check_source_functor_property(&foreign_source, &symbols, &typed)
            .is_err()
    );
    assert!(
        TermFormulaChecker::check_source_functor_property(&source, &foreign_symbols, &typed)
            .is_err()
    );
    let ast = super::formula_statement::step5c8_test_frontend(&text)
        .ast
        .unwrap();
    let foreign = ResolverModuleId::new(PackageId::new("foreign"), ModulePath::new("property"));
    assert!(
        TermFormulaChecker::check_source_functor_property(
            &SurfaceResolvedArena::lower(&ast, &foreign).unwrap(),
            &symbols,
            &typed
        )
        .is_err()
    );
    for mutation in 0..3 {
        let mut indexes = super::import_fixtures::clone_symbol_env_indexes(&symbols);
        match mutation {
            0 => indexes.definitions = Default::default(),
            1 => indexes.symbols = Default::default(),
            2 => indexes.contributions = Default::default(),
            _ => unreachable!(),
        }
        assert!(
            TermFormulaChecker::check_source_functor_property(
                &source,
                &SymbolEnv::new(symbols.module_id().clone(), indexes),
                &typed
            )
            .is_err()
        );
    }
    let original_nodes = typed
        .iter()
        .map(|(_, node)| node.clone())
        .collect::<Vec<_>>();
    let targets = typed
        .iter()
        .filter(|(_, node)| {
            matches!(
                node.kind.as_str(),
                "TermReference" | "PropertyClause" | "FunctorPattern" | "OperatorDeclaration"
            )
        })
        .map(|(id, _)| id)
        .collect::<Vec<_>>();
    assert_eq!(targets.len(), 5);
    for target in targets {
        for mutation in 0..3 {
            let mut nodes = original_nodes.clone();
            match mutation {
                0 => nodes[target.index()].kind = "ForeignOwner".into(),
                1 => nodes[target.index()].resolved_node = None,
                2 => nodes[target.index()].typing = TypingState::Successful,
                _ => unreachable!(),
            }
            let forged = TypedArena::try_new(typed.root(), nodes).unwrap();
            assert!(
                TermFormulaChecker::check_source_functor_property(&source, &symbols, &forged)
                    .is_err(),
                "{target:?}/{mutation}"
            );
        }
    }
}
