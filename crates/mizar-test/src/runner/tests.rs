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
    assert_eq!(report.results.len(), 4);
    assert_eq!(report.passed_count(), 4);
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
