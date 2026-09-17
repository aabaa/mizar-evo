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
    for mutation in 0..13 {
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
            assert!(!matches!(mutation, 0 | 7..=12), "valid modified Core must generate VCs");
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
    let (original, _) = step5c11_false_coherence_case();
    let config = step5c11_config();
    assert!(super::is_active_proof_verification(&original));
    for mutation in 0..13 {
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
            _ => unreachable!(),
        }
        assert!(
            !super::proof_verification::step5c11_proof_admitted(
                Some(&config.workspace_root),
                &case
            ),
            "mutation {mutation}"
        );
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
        assert!(!super::formula_statement::is_active_formula_statement(&config.workspace_root, &case));
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
    let temporary = std::process::Command::new("mktemp").arg("-d").output().unwrap();
    assert!(temporary.status.success());
    let root = PathBuf::from(String::from_utf8(temporary.stdout).unwrap().trim());
    let mut absent = plan.clone();
    absent.cases.clear();
    assert!(super::proof_verification::validate_active_proof_verification_tags(&root, &absent).is_empty());
    std::fs::create_dir_all(root.join("tests/coverage")).unwrap();
    std::fs::copy(config.workspace_root.join("tests/coverage/step5_activation_map.tsv"), root.join("tests/coverage/step5_activation_map.tsv")).unwrap();
    assert!(super::proof_verification::validate_active_proof_verification_tags(&root, &absent).iter().any(|diagnostic| diagnostic.detail_key == "proof_verification.step5c11_inventory"));
    std::fs::remove_dir_all(root).unwrap();

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
        ("var x := g", "const x := g"),
        ("return x;", "x := a;\n return x;"),
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
        text.replace("return a;", "var x := a; return x;"),
        text.replace("return a;", "return a; return a;"),
        text.replace("return a;", "assert a = a; return a;"),
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
    let original = step5c14_return_case();
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
            3 => case.expectation_path = case.expectation_path.with_file_name("wrong.expect.toml"),
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
