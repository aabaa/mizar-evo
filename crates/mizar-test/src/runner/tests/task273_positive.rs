use super::*;

fn task273_handoff(
    family: &str,
    text: &str,
) -> (
    mizar_checker::source_registration::SourceRegistrationHandoff,
    mizar_checker::typed_ast::TypedAst,
    mizar_checker::resolved_typed_ast::ResolvedTypedAst,
    mizar_resolve::env::SymbolEnv,
    mizar_resolve::resolved_ast::SurfaceResolvedArena,
) {
    let (case, _) = step5c11_source_case(family);
    let frontend = super::super::formula_statement::step5c8_test_frontend(text);
    let (native, nodes, symbols) = super::super::source_registration_inputs(
        &step5c11_config().workspace_root,
        &case,
        frontend,
    )
    .unwrap_or_else(|error| panic!("{family}: {error}"));
    let original = std::sync::Arc::new(
        super::super::source_registration_receiver(&native, nodes)
            .unwrap_or_else(|error| panic!("{family}: {error}")),
    );
    let handoff = mizar_checker::source_registration::SourceRegistrationHandoff::build(
        std::sync::Arc::clone(&original),
        &native,
        &symbols,
    )
    .unwrap_or_else(|error| panic!("{family}: {error}"));
    let installed = original
        .as_ref()
        .clone()
        .with_source_registration(handoff.clone())
        .unwrap_or_else(|error| panic!("{family}: {error}"));
    let resolved =
        super::super::type_elaboration::assemble_empty_resolved_typed_ast(&installed, Vec::new())
            .unwrap_or_else(|error| panic!("{family}: {error}"));
    (handoff, installed, resolved, symbols, native)
}

fn fixture_text(family: &str) -> String {
    step5c11_source_case(family).1
}

fn attribute_symbol_key(symbols: &mizar_resolve::env::SymbolEnv, local: &str) -> String {
    let symbol = symbols
        .symbols()
        .iter()
        .find(|entry| entry.primary_spelling() == local)
        .unwrap_or_else(|| panic!("missing source attribute {local}"));
    format!("{:?}", symbol.symbol())
}

#[test]
fn task273_each_registration_kind_carries_source_operands_and_pending_intake() {
    use mizar_checker::registration_resolution::{
        PendingRegistrationStatus, RegistrationPatternStatus, RegistrationValidationKind,
        RegistrationValidationPattern,
    };
    use mizar_checker::typed_ast::{InitialObligationKind, InitialObligationStatus};

    for (family, expected) in [
        ("existential", RegistrationValidationKind::Existential),
        ("conditional", RegistrationValidationKind::Conditional),
        ("functorial", RegistrationValidationKind::Functorial),
        ("reduce", RegistrationValidationKind::Reduction),
    ] {
        let text = fixture_text(family);
        let (handoff, typed, resolved, symbols, native) = task273_handoff(family, &text);
        assert_eq!(handoff.validations().len(), 1, "{family}");
        let validation = &handoff.validations()[0];
        assert_eq!(validation.pattern().kind(), expected, "{family}");
        match validation.pattern() {
            RegistrationValidationPattern::Existential {
                type_head,
                attributes,
            } => {
                assert_eq!(type_head.as_str(), "builtin.set");
                assert_eq!(attributes.len(), 1);
                assert_eq!(
                    attributes[0].as_str(),
                    attribute_symbol_key(&symbols, "cmarked")
                );
            }
            RegistrationValidationPattern::Conditional {
                type_head,
                antecedent,
                consequent,
            } => {
                assert_eq!(type_head.as_str(), "builtin.set");
                assert_eq!(antecedent.len(), 1);
                assert_eq!(consequent.len(), 1);
                assert_eq!(
                    antecedent[0].as_str(),
                    attribute_symbol_key(&symbols, "c2marked")
                );
                assert_eq!(
                    consequent[0].as_str(),
                    attribute_symbol_key(&symbols, "d2marked")
                );
            }
            RegistrationValidationPattern::Functorial {
                functor,
                result_type,
                consequent,
            } => {
                let functors = symbols
                    .symbols()
                    .iter()
                    .filter(|entry| entry.kind() == mizar_resolve::env::SymbolKind::Functor)
                    .map(|entry| entry.symbol())
                    .collect::<Vec<_>>();
                let [symbol] = functors.as_slice() else {
                    panic!("one source functor declaration");
                };
                let symbol = *symbol;
                let (registration_id, registration) = native
                    .arena()
                    .iter()
                    .find(|(_, node)| {
                        node.kind() == &mizar_syntax::ast::SurfaceNodeKind::FunctorialRegistration
                    })
                    .expect("functorial source registration");
                let term_expression = registration
                    .children()
                    .iter()
                    .map(|id| native.arena().node(*id).unwrap())
                    .find(|node| node.kind() == &mizar_syntax::ast::SurfaceNodeKind::TermExpression)
                    .expect("functorial term operand");
                let application_id = term_expression
                    .children()
                    .iter()
                    .copied()
                    .find(|id| {
                        matches!(
                            native.arena().node(*id).unwrap().kind(),
                            mizar_syntax::ast::SurfaceNodeKind::PrefixExpression(_)
                        )
                    })
                    .expect("functor application source site");
                let application = native.arena().node(application_id).unwrap();
                let source_range = match application.origin().anchor() {
                    mizar_session::SourceAnchor::Range(range) => *range,
                    _ => panic!("functor application must have an original source range"),
                };
                let application_site = mizar_checker::typed_ast::TypedSiteRef::Node(
                    mizar_checker::typed_ast::TypedNodeId::new(application_id.index()),
                );
                assert_eq!(functor.as_str(), format!("{symbol:?}:{application_site:?}"));
                assert_eq!(&text[source_range.start..source_range.end], "cbox X");
                assert_eq!(validation.parameters().len(), 1);
                assert!(validation.parameters()[0].is_typed());
                assert!(validation.parameters()[0].facts_are_visible());
                let application_term = handoff
                    .inference()
                    .terms()
                    .iter()
                    .map(|(_, term)| term)
                    .find(|term| term.site == application_site)
                    .expect("checked functor application operand");
                assert_eq!(
                    application_term.kind,
                    mizar_checker::type_checker::TermKind::FunctorApplication
                );
                assert_eq!(
                    application_term.reference,
                    Some(mizar_checker::type_checker::TermReference::Symbol(
                        symbol.clone()
                    ))
                );
                assert!(application_term.expected_type.is_some());
                assert_eq!(registration_id.index(), validation.owner().node().index());
                let argument_id = native
                    .arena()
                    .iter()
                    .find_map(|(id, node)| {
                        if node.kind() != &mizar_syntax::ast::SurfaceNodeKind::TermReference {
                            return None;
                        }
                        match node.origin().anchor() {
                            mizar_session::SourceAnchor::Range(range)
                                if range.start >= source_range.start
                                    && range.end <= source_range.end =>
                            {
                                Some(id)
                            }
                            _ => None,
                        }
                    })
                    .expect("source argument binding reference");
                let argument_node = native.arena().node(argument_id).unwrap();
                let argument_range = match argument_node.origin().anchor() {
                    mizar_session::SourceAnchor::Range(range) => *range,
                    _ => panic!("argument binding must have a source range"),
                };
                assert_eq!(&text[argument_range.start..argument_range.end], "X");
                let argument_site = mizar_checker::typed_ast::TypedSiteRef::Node(
                    mizar_checker::typed_ast::TypedNodeId::new(argument_id.index()),
                );
                let argument_term = handoff
                    .inference()
                    .terms()
                    .iter()
                    .map(|(_, term)| term)
                    .find(|term| term.site == argument_site)
                    .expect("checked registration argument binding");
                assert_eq!(
                    argument_term.kind,
                    mizar_checker::type_checker::TermKind::Variable
                );
                assert!(matches!(
                    argument_term.reference,
                    Some(mizar_checker::type_checker::TermReference::Binding(_))
                ));
                assert_eq!(result_type.as_str(), "builtin.set");
                assert_eq!(consequent.len(), 1);
                assert_eq!(
                    consequent[0].as_str(),
                    attribute_symbol_key(&symbols, "e3marked")
                );
            }
            RegistrationValidationPattern::Reduction { lhs, rhs } => {
                assert!(!lhs.fingerprint().as_str().is_empty());
                assert!(!rhs.fingerprint().as_str().is_empty());
                assert_ne!(lhs.fingerprint(), rhs.fingerprint());
                let lhs_range = lhs.source_range().unwrap();
                let rhs_range = rhs.source_range().unwrap();
                assert_eq!(&text[lhs_range.start..lhs_range.end], "cbox2 X");
                assert_eq!(validation.parameters().len(), 1);
                assert!(validation.parameters()[0].is_typed());
                assert!(validation.parameters()[0].facts_are_visible());
                assert_eq!(&text[rhs_range.start..rhs_range.end], "X");
            }
            _ => panic!("unexpected registration validation pattern for {family}"),
        }
        let database = handoff.database();
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
        let (correctness_site, associated_id, correctness_anchor) = handoff
            .correctness()
            .next()
            .unwrap_or_else(|| panic!("correctness association missing for {family}"));
        assert_eq!(associated_id, obligation.id);
        let correctness_node = typed
            .source_node_entries()
            .unwrap()
            .find(|source_node| {
                source_node.site().index() == correctness_site.node().index()
                    && source_node.kind()
                        == &mizar_syntax::ast::SurfaceNodeKind::CorrectnessCondition
            })
            .unwrap_or_else(|| panic!("correctness source identity missing for {family}"));
        assert_eq!(
            correctness_node.range(),
            match correctness_anchor {
                mizar_session::SourceAnchor::Range(range) => *range,
                _ => panic!("expected original source range for {family}"),
            }
        );
        assert!(handoff.inference().diagnostics().is_empty());
        assert_eq!(typed.source_registration(), Some(&handoff));
        assert_eq!(resolved.source_registration(), Some(&handoff));
        assert_eq!(typed.initial_obligations(), database.initial_obligations());
    }
}

#[test]
fn task273_mixed_registration_items_keep_source_order_and_obligation_association() {
    use mizar_checker::typed_ast::{InitialObligationKind, InitialObligationStatus};

    let text = ["existential", "conditional", "functorial", "reduce"]
        .into_iter()
        .map(fixture_text)
        .collect::<Vec<_>>()
        .join("\n");
    let (handoff, typed, resolved, _, _) = task273_handoff("existential", &text);
    let validations = handoff.validations();
    assert_eq!(validations.len(), 4);
    assert!(
        validations
            .windows(2)
            .all(|pair| pair[0].source_range().start < pair[1].source_range().start)
    );
    assert_eq!(
        validations
            .iter()
            .map(|row| row.pattern().kind())
            .collect::<Vec<_>>(),
        vec![
            mizar_checker::registration_resolution::RegistrationValidationKind::Existential,
            mizar_checker::registration_resolution::RegistrationValidationKind::Conditional,
            mizar_checker::registration_resolution::RegistrationValidationKind::Functorial,
            mizar_checker::registration_resolution::RegistrationValidationKind::Reduction,
        ]
    );

    let pending = handoff.database().pending().iter().collect::<Vec<_>>();
    assert_eq!(pending.len(), 4);
    let obligations = handoff
        .database()
        .initial_obligations()
        .iter()
        .collect::<Vec<_>>();
    assert_eq!(obligations.len(), 4);
    for (index, ((row, registration), (id, obligation))) in
        validations.iter().zip(pending).zip(obligations).enumerate()
    {
        assert_eq!(
            row.resolver_registration(),
            registration.resolver_registration()
        );
        assert_eq!(registration.obligations(), &[id]);
        assert_eq!(obligation.id, id);
        assert_eq!(
            obligation.kind,
            InitialObligationKind::RegistrationCorrectness
        );
        assert_eq!(obligation.status, InitialObligationStatus::Pending);
        assert_eq!(obligation.source_range, row.source_range());
        assert_eq!(
            registration.source().origin().anchor(),
            &mizar_session::SourceAnchor::Range(row.source_range())
        );
        assert_eq!(obligation.owner, *row.owner());
        let (site, _, anchor) = handoff
            .correctness()
            .find(|(_, associated, _)| *associated == id)
            .unwrap_or_else(|| panic!("correctness identity missing for registration {index}"));
        assert_ne!(*site, *row.owner());
        let source_node = typed
            .source_node_entries()
            .unwrap()
            .find(|source_node| source_node.site().index() == site.node().index())
            .expect("correctness site refers to an original source node");
        assert_eq!(
            source_node.kind(),
            &mizar_syntax::ast::SurfaceNodeKind::CorrectnessCondition
        );
        assert_eq!(
            source_node.range(),
            match anchor {
                mizar_session::SourceAnchor::Range(range) => *range,
                _ => panic!("correctness anchor must be an original source range"),
            }
        );
    }
    assert_eq!(
        typed.initial_obligations(),
        handoff.database().initial_obligations()
    );
    assert_eq!(typed.source_registration(), Some(&handoff));
    assert_eq!(resolved.source_registration(), Some(&handoff));
}

#[test]
fn task273_reuses_a_source_attribute_across_two_and_four_registration_items() {
    use mizar_checker::registration_resolution::RegistrationValidationPattern;

    let existential = fixture_text("existential");
    let conditional =
        fixture_text("conditional").replace("c2marked -> d2marked", "cmarked -> d2marked");
    assert_ne!(conditional, fixture_text("conditional"));
    let mut shared = vec![existential, conditional];
    for family in ["functorial", "reduce"] {
        shared.push(fixture_text(family));
    }

    for count in [2, 4] {
        let text = shared[..count].join("\n");
        let (handoff, _, _, symbols, _) = task273_handoff("existential", &text);
        assert_eq!(handoff.validations().len(), count);
        let shared_key = attribute_symbol_key(&symbols, "cmarked");
        let existential = handoff
            .validations()
            .iter()
            .find_map(|row| match row.pattern() {
                RegistrationValidationPattern::Existential { attributes, .. } => {
                    Some(attributes[0].as_str())
                }
                _ => None,
            })
            .expect("existential registration retained");
        let conditional_guard = handoff
            .validations()
            .iter()
            .find_map(|row| match row.pattern() {
                RegistrationValidationPattern::Conditional { antecedent, .. } => {
                    Some(antecedent[0].as_str())
                }
                _ => None,
            })
            .expect("conditional registration retained");
        assert_eq!(existential, shared_key);
        assert_eq!(conditional_guard, shared_key);
    }
}

#[test]
fn task273_handoff_contains_only_pending_registration_and_obligation_intake() {
    let (handoff, typed, resolved, _, _) =
        task273_handoff("existential", &fixture_text("existential"));
    let database = handoff.database();
    assert_eq!(database.pending().len(), 1);
    assert!(database.activated().is_empty());
    assert!(database.rejected().is_empty());
    assert!(database.diagnostics().is_empty());
    assert!(typed.facts().is_empty());
    assert!(typed.coercions().is_empty());
    assert_eq!(typed.initial_obligations(), database.initial_obligations());
    assert!(handoff.inference().diagnostics().is_empty());
    assert!(resolved.cluster_facts().is_empty());
    assert!(resolved.checked_proofs().is_empty());
    assert!(resolved.checked_proof_nodes().is_empty());
    assert!(resolved.checked_terminal_goals().is_empty());
    assert_eq!(typed.source_registration(), Some(&handoff));
    assert_eq!(resolved.source_registration(), Some(&handoff));
}
