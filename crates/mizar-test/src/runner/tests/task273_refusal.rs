use super::*;

fn registration_inputs(
    family: &str,
    source_text: &str,
) -> (
    mizar_resolve::resolved_ast::SurfaceResolvedArena,
    mizar_checker::typed_ast::TypedArena,
    mizar_resolve::env::SymbolEnv,
) {
    let (case, _) = step5c11_source_case(family);
    let frontend = super::super::formula_statement::step5c8_test_frontend(source_text);
    super::super::source_registration_inputs(&step5c11_config().workspace_root, &case, frontend)
        .unwrap_or_else(|error| panic!("valid {family} input rejected: {error}"))
}

fn refuses_source(family: &str, source_text: &str) {
    use mizar_checker::source_registration::SourceRegistrationHandoff;
    use std::sync::Arc;

    assert_ne!(source_text, step5c11_source_case(family).1);
    let (source, nodes, symbols) = match super::super::source_registration_inputs(
        &step5c11_config().workspace_root,
        &step5c11_source_case(family).0,
        super::super::formula_statement::step5c8_test_frontend(source_text),
    ) {
        Ok(inputs) => inputs,
        Err(error) => {
            assert!(!error.is_empty(), "mutation must be rejected with a reason");
            return;
        }
    };
    let receiver = super::super::source_registration_receiver(&source, nodes)
        .unwrap_or_else(|error| panic!("valid {family} receiver rejected: {error}"));
    assert!(
        SourceRegistrationHandoff::build(Arc::new(receiver), &source, &symbols).is_err(),
        "accepted invalid {family} registration source"
    );
}

#[test]
fn task273_refuses_missing_kind_applicable_operands() {
    use mizar_checker::source_registration::SourceRegistrationHandoff;
    use std::sync::Arc;

    // Establish that each source mutation starts from an authentic, accepted source.
    for family in ["existential", "conditional", "functorial", "reduce"] {
        let (_, original) = step5c11_source_case(family);
        let (source, nodes, symbols) = registration_inputs(family, &original);
        let receiver = super::super::source_registration_receiver(&source, nodes).unwrap();
        assert!(
            SourceRegistrationHandoff::build(Arc::new(receiver), &source, &symbols).is_ok(),
            "baseline {family} registration was not accepted"
        );
    }

    let (_, existential) = step5c11_source_case("existential");
    refuses_source(
        "existential",
        &existential.replace("cluster CReg1: cmarked set;", "cluster CReg1: set;"),
    );
    refuses_source(
        "existential",
        &existential.replace(
            "cluster CReg1: cmarked set;",
            "cluster CReg1: cmarked set;\n  cluster CReg1: cmarked set;",
        ),
    );
    refuses_source("existential", &existential.replace("  existence\n", ""));

    let (_, conditional) = step5c11_source_case("conditional");
    refuses_source(
        "conditional",
        &conditional.replace(
            "cluster CReg2: c2marked -> d2marked for set;",
            "cluster CReg2: d2marked for set;",
        ),
    );
    refuses_source(
        "conditional",
        &conditional.replace(
            "cluster CReg2: c2marked -> d2marked for set;",
            "cluster CReg2: c2marked -> for set;",
        ),
    );
    refuses_source("conditional", &conditional.replace("  coherence\n", ""));

    let (_, functorial) = step5c11_source_case("functorial");
    refuses_source(
        "functorial",
        &functorial.replace(
            "cluster CReg3: cbox X -> e3marked for set;",
            "cluster CReg3: cbox X -> for set;",
        ),
    );

    let (_, reduce) = step5c11_source_case("reduce");
    refuses_source(
        "reduce",
        &reduce.replace("reduce CRed1: cbox2 X to X;", "reduce CRed1: cbox2 X;"),
    );
    refuses_source(
        "reduce",
        &reduce.replace("reduce CRed1: cbox2 X to X;", "reduce CRed1: to X;"),
    );
}

#[test]
fn task273_refuses_corrupt_identity_anchor_and_source_order() {
    use mizar_checker::{
        source_registration::SourceRegistrationHandoff,
        typed_ast::{NodeRecoveryState, TypedArena},
    };
    use std::sync::Arc;

    let (case, text) = step5c11_source_case("functorial");
    let frontend = super::super::formula_statement::step5c8_test_frontend(&text);
    let (source, nodes, symbols) = super::super::source_registration_inputs(
        &step5c11_config().workspace_root,
        &case,
        frontend,
    )
    .unwrap();
    let baseline = super::super::source_registration_receiver(&source, nodes.clone()).unwrap();
    assert!(
        SourceRegistrationHandoff::build(Arc::new(baseline), &source, &symbols).is_ok(),
        "authentic source receiver must be accepted"
    );

    let refuse_nodes = |changed: Vec<mizar_checker::typed_ast::TypedNode>| {
        let changed = TypedArena::try_new(nodes.root(), changed).unwrap();
        let receiver = match super::super::source_registration_receiver(&source, changed) {
            Ok(receiver) => receiver,
            Err(error) => {
                assert!(
                    !error.is_empty(),
                    "corrupt receiver must be refused with a reason"
                );
                return;
            }
        };
        assert!(SourceRegistrationHandoff::build(Arc::new(receiver), &source, &symbols).is_err());
    };

    let raw = nodes
        .iter()
        .map(|(_, node)| node.clone())
        .collect::<Vec<_>>();
    let mut recovered = raw.clone();
    recovered[0] = recovered[0]
        .clone()
        .with_recovery(NodeRecoveryState::Recovered);
    refuse_nodes(recovered);

    let mut wrong_anchor = raw.clone();
    wrong_anchor[0].anchor = wrong_anchor[1].anchor.clone();
    refuse_nodes(wrong_anchor);

    let mut wrong_source_identity = raw.clone();
    wrong_source_identity[0].resolved_node = wrong_source_identity[1].resolved_node;
    refuse_nodes(wrong_source_identity);

    let mut foreign_anchor = raw.clone();
    let (_, other_nodes, _) =
        registration_inputs("conditional", &step5c11_source_case("conditional").1);
    foreign_anchor[0].anchor = other_nodes
        .iter()
        .map(|(_, node)| &node.anchor)
        .find(|anchor| **anchor != raw[0].anchor)
        .unwrap()
        .clone();
    assert_ne!(foreign_anchor[0].anchor, raw[0].anchor);
    refuse_nodes(foreign_anchor);

    let mixed = [step5c11_source_case("existential").1, text].join("\n");
    let (source, nodes, symbols) = registration_inputs("existential", &mixed);
    let receiver = super::super::source_registration_receiver(&source, nodes.clone()).unwrap();
    SourceRegistrationHandoff::build(Arc::new(receiver), &source, &symbols).unwrap();
    let registrations = source
        .arena()
        .iter()
        .filter(|(_, node)| {
            matches!(
                node.kind(),
                mizar_syntax::SurfaceNodeKind::ExistentialRegistration
                    | mizar_syntax::SurfaceNodeKind::FunctorialRegistration
            )
        })
        .map(|(id, _)| id.index())
        .collect::<Vec<_>>();
    assert_eq!(registrations.len(), 2);
    let mut reordered = nodes
        .iter()
        .map(|(_, node)| node.clone())
        .collect::<Vec<_>>();
    reordered.swap(registrations[0], registrations[1]);
    let reordered = TypedArena::try_new(nodes.root(), reordered).unwrap();
    assert!(
        super::super::source_registration_receiver(&source, reordered)
            .and_then(|receiver| SourceRegistrationHandoff::build(
                Arc::new(receiver),
                &source,
                &symbols
            ))
            .is_err()
    );
}

#[test]
fn task273_intake_binds_correctness_identity_and_rejects_contaminated_receiver() {
    use mizar_checker::{
        source_registration::SourceRegistrationHandoff,
        typed_ast::{InitialObligationId, TypedAstParts},
    };
    use std::sync::Arc;

    let (case, text) = step5c11_source_case("existential");
    let (source, nodes, symbols) = super::super::source_registration_inputs(
        &step5c11_config().workspace_root,
        &case,
        super::super::formula_statement::step5c8_test_frontend(&text),
    )
    .unwrap();
    let receiver = super::super::source_registration_receiver(&source, nodes).unwrap();
    let handoff =
        SourceRegistrationHandoff::build(Arc::new(receiver.clone()), &source, &symbols).unwrap();
    let obligations = handoff.database().initial_obligations();
    assert_eq!(obligations.len(), 1);
    let (site, obligation_id, anchor) = handoff.correctness().next().unwrap();
    assert_eq!(obligation_id, InitialObligationId::new(0));
    let validation = &handoff.validations()[0];
    let obligation = obligations.get(obligation_id).unwrap();
    assert_eq!(&obligation.owner, validation.owner());
    assert_eq!(obligation.source_range, validation.source_range());
    assert_eq!(anchor, &receiver.nodes().node(site.node()).unwrap().anchor);

    // The handoff has no caller-supplied obligation-ID input. A receiver carrying
    // preexisting obligations fails the original-source equality check.
    let mut parts = TypedAstParts {
        source_id: receiver.source_id(),
        module_id: receiver.module_id().clone(),
        resolved_root: receiver.resolved_root(),
        source_context: None,
        source_type: None,
        source_attribute: None,
        nodes: receiver.nodes().clone(),
        contexts: Default::default(),
        types: Default::default(),
        facts: Default::default(),
        coercions: Default::default(),
        initial_obligations: Default::default(),
        diagnostics: Default::default(),
    };
    // Keep this mutation intentionally malformed: registration intake accepts no
    // preexisting or caller-authored obligation table.
    parts.initial_obligations = (*obligations).clone();
    let mut contaminated = mizar_checker::typed_ast::TypedAst::try_new(parts).unwrap();
    contaminated.install_source_nodes(&source).unwrap();
    assert!(
        contaminated
            .clone()
            .with_source_registration(handoff)
            .is_err()
    );
    assert!(SourceRegistrationHandoff::build(Arc::new(contaminated), &source, &symbols).is_err());
}
