#[path = "support/source_overload.rs"]
mod support;

use mizar_checker::{
    overload_resolution::{
        ArgumentViabilityEvidence, CandidateDeclarationKind, InsertedViewInput, InsertedViewKind,
        InsertedViewReasonKey, InsertedViewStatus, OverloadCandidateId, OverloadResultId,
        OverloadResultStatus, SpecificityComparisonInput, SpecificityComparisonStatus,
        SpecificityGraphId, SpecificityReasonKey, ViabilityFactStatus,
    },
    source_overload::{SourceOverloadHandoff, SourceOverloadInput},
    typed_ast::TypedSiteRef,
};
use support::{SourceFixture, ordinary_overloads, single_ordinary, source_view, template_overload};

fn contest(input: &mut SourceOverloadInput, left: usize, right: usize, reason: &str) {
    input.comparisons.push(SpecificityComparisonInput {
        left: OverloadCandidateId::new(left),
        right: OverloadCandidateId::new(right),
        status: SpecificityComparisonStatus::LeftAtLeastRight,
        reasons: vec![SpecificityReasonKey::new(reason)],
    });
}

fn resolved_root(handoff: &SourceOverloadHandoff) -> OverloadCandidateId {
    let row = handoff
        .selection()
        .results()
        .get(OverloadResultId::new(0))
        .expect("one selected overload result");
    match row.status {
        OverloadResultStatus::Resolved { root, .. } => root,
        ref status => panic!("expected an ordinary root, found {status:?}"),
    }
}

fn add_object_widening_evidence(
    input: &mut SourceOverloadInput,
    fact: mizar_checker::typed_ast::TypeFactId,
) {
    let ArgumentViabilityEvidence::Exact { actual } = &input.viability[1].arguments[0] else {
        panic!("source intake starts with exact argument evidence");
    };
    input.viability[1].arguments[0] = ArgumentViabilityEvidence::FactWidening {
        actual: *actual,
        target: input.candidates[1].parameters[0],
        facts: vec![fact],
        status: ViabilityFactStatus::Consumable,
    };
}

fn source_anchored_overloads() -> (
    SourceFixture,
    mizar_checker::typed_ast::TypedAst,
    mizar_checker::typed_ast::TypeFactId,
) {
    let fixture = ordinary_overloads();
    let (initial, _) =
        SourceOverloadInput::from_source(&fixture.native, &fixture.symbols, &fixture.typed)
            .expect("derive source call and argument");
    let (typed, fact) = fixture.with_known_fact(initial.sites[0].arguments[0].clone(), "object");
    (fixture, typed, fact)
}

#[test]
fn ordinary_multicandidate_site_uses_explicit_evidence_through_all_stages() {
    let (fixture, typed, fact) = source_anchored_overloads();
    let (mut input, normalization) =
        SourceOverloadInput::from_source(&fixture.native, &fixture.symbols, &typed)
            .expect("derive authenticated source overload rows");
    assert_eq!(input.sites.len(), 1);
    assert_eq!(input.candidates.len(), 2);
    add_object_widening_evidence(&mut input, fact);
    contest(&mut input, 0, 1, "source-specificity-set-over-object");

    let handoff = SourceOverloadHandoff::build(
        input.clone(),
        &fixture.native,
        &fixture.symbols,
        &typed,
        &normalization,
    )
    .expect("build source-authenticated selection pipeline");

    assert_eq!(handoff.source_id(), fixture.native.source_id());
    assert_eq!(handoff.module_id(), fixture.native.module());
    assert_eq!(handoff.collection().sites().len(), 1);
    assert_eq!(handoff.collection().candidates().len(), 2);
    assert_eq!(handoff.expansion().candidates().len(), 2);
    assert_eq!(handoff.viability().decisions().len(), 2);
    assert_eq!(handoff.specificity().graphs().len(), 1);
    assert_eq!(
        handoff
            .specificity()
            .graphs()
            .get(SpecificityGraphId::new(0))
            .unwrap()
            .comparisons
            .len(),
        1
    );
    let root = resolved_root(&handoff);
    assert_eq!(root, OverloadCandidateId::new(0));
    let selected = handoff.collection().candidates().get(root).unwrap();
    assert_eq!(selected.symbol, input.candidates[root.index()].symbol);
    assert_eq!(selected.ordinary_root, selected.symbol);
    assert!(input.candidates.iter().all(|candidate| {
        candidate
            .provenance
            .source_range
            .is_some_and(|range| range.source_id == fixture.native.source_id())
    }));

    let installed = typed
        .clone()
        .with_source_overload(handoff.clone())
        .expect("install immutable handoff in TypedAst");
    assert_eq!(installed.source_overload(), Some(&handoff));
    assert_eq!(
        handoff.debug_text(),
        SourceOverloadHandoff::build(
            input,
            &fixture.native,
            &fixture.symbols,
            &typed,
            &normalization,
        )
        .unwrap()
        .debug_text()
    );
}

#[test]
fn template_site_uses_native_direct_template_roles_and_selects_template_root() {
    let fixture = template_overload();
    let (input, normalization) =
        SourceOverloadInput::from_source(&fixture.native, &fixture.symbols, &fixture.typed)
            .expect("derive template candidate from direct source roles");
    assert_eq!(input.sites.len(), 1);
    assert_eq!(input.candidates.len(), 1);
    let candidate = &input.candidates[0];
    assert!(candidate.template.is_some());
    assert_eq!(
        candidate.declaration_kind,
        CandidateDeclarationKind::Template
    );
    let template_argument = &candidate.template.as_ref().unwrap().arguments;
    assert_eq!(template_argument.len(), 1);
    assert!(matches!(
        template_argument[0],
        mizar_checker::overload_resolution::TemplateArgument::Explicit(_)
    ));

    let handoff = SourceOverloadHandoff::build(
        input,
        &fixture.native,
        &fixture.symbols,
        &fixture.typed,
        &normalization,
    )
    .expect("build template overload pipeline");
    assert_eq!(handoff.expansion().expansions().len(), 1);
    assert_eq!(handoff.viability().decisions().len(), 1);
    assert_eq!(resolved_root(&handoff), OverloadCandidateId::new(0));
    assert!(
        handoff
            .collection()
            .candidates()
            .get(OverloadCandidateId::new(0))
            .unwrap()
            .template
            .is_some()
    );
}

#[test]
fn one_source_candidate_keeps_its_full_payload_and_root() {
    let fixture = single_ordinary();
    let (input, normalization) =
        SourceOverloadInput::from_source(&fixture.native, &fixture.symbols, &fixture.typed)
            .expect("derive source overload rows");
    assert_eq!(input.candidates.len(), 1);
    let source_symbol = input.candidates[0].symbol.clone();
    let source_result = input.candidates[0].result;
    let source_parameters = input.candidates[0].parameters.clone();
    let handoff = SourceOverloadHandoff::build(
        input,
        &fixture.native,
        &fixture.symbols,
        &fixture.typed,
        &normalization,
    )
    .expect("build single-candidate selection pipeline");

    assert_eq!(resolved_root(&handoff), OverloadCandidateId::new(0));
    let candidate = handoff
        .collection()
        .candidates()
        .get(OverloadCandidateId::new(0))
        .unwrap();
    assert_eq!(candidate.symbol, source_symbol);
    assert_eq!(candidate.result, source_result);
    assert_eq!(candidate.parameters, source_parameters);
}

#[test]
fn explicit_source_qua_is_consumed_by_inserted_view_selection() {
    let fixture = source_view();
    let (initial, _) =
        SourceOverloadInput::from_source(&fixture.native, &fixture.symbols, &fixture.typed)
            .expect("derive source call and argument");
    let (typed, fact) = fixture.with_known_fact(initial.sites[0].arguments[0].clone(), "object");
    let (mut input, normalization) =
        SourceOverloadInput::from_source(&fixture.native, &fixture.symbols, &typed)
            .expect("derive authenticated source view");
    let ArgumentViabilityEvidence::Exact { actual } = &input.viability[0].arguments[0] else {
        panic!("source intake starts with exact argument evidence");
    };
    input.viability[0].arguments[0] = ArgumentViabilityEvidence::FactWidening {
        actual: *actual,
        target: input.candidates[0].parameters[0],
        facts: vec![fact],
        status: ViabilityFactStatus::Consumable,
    };
    let source_qua = input.sites[0]
        .source_qua
        .first()
        .expect("written qua basis")
        .clone();
    let resolution = input.resolution.first_mut().expect("site resolution");
    resolution.inserted_views.push(InsertedViewInput {
        argument: source_qua.argument.clone(),
        target: source_qua.target,
        selected_candidate: OverloadCandidateId::new(0),
        kind: InsertedViewKind::SourceQua,
        status: InsertedViewStatus::Accepted,
        reason: InsertedViewReasonKey::new("written-source-qua"),
        evidence_facts: source_qua.evidence_facts.clone(),
        path: source_qua.path.clone(),
    });
    let handoff = SourceOverloadHandoff::build(
        input,
        &fixture.native,
        &fixture.symbols,
        &typed,
        &normalization,
    )
    .expect("build explicit source-view selection");

    let result = handoff
        .selection()
        .results()
        .get(OverloadResultId::new(0))
        .unwrap();
    let OverloadResultStatus::Resolved {
        root,
        inserted_views,
        ..
    } = &result.status
    else {
        panic!(
            "expected source-view candidate to resolve: {:?}",
            result.status
        );
    };
    assert_eq!(*root, OverloadCandidateId::new(0));
    assert_eq!(inserted_views.len(), 1);
    let view = handoff
        .selection()
        .inserted_views()
        .get(inserted_views[0])
        .expect("selected inserted view");
    assert_eq!(view.argument, source_qua.argument);
    assert_eq!(view.target, source_qua.target);
    assert_eq!(view.kind, InsertedViewKind::SourceQua);
    assert_eq!(view.reason.as_str(), "written-source-qua");
    assert!(matches!(source_qua.argument, TypedSiteRef::Node(_)));
}

#[test]
fn contested_candidates_keep_source_order_and_deterministic_winner() {
    let (fixture, typed, fact) = source_anchored_overloads();
    let (mut input, normalization) =
        SourceOverloadInput::from_source(&fixture.native, &fixture.symbols, &typed)
            .expect("derive authenticated source overload rows");
    let source_order: Vec<_> = input
        .candidates
        .iter()
        .map(|candidate| candidate.provenance.source_range.unwrap().start)
        .collect();
    assert!(source_order.windows(2).all(|pair| pair[0] < pair[1]));
    add_object_widening_evidence(&mut input, fact);
    contest(&mut input, 0, 1, "source-order-contest");

    let first = SourceOverloadHandoff::build(
        input.clone(),
        &fixture.native,
        &fixture.symbols,
        &typed,
        &normalization,
    )
    .expect("build first contested resolution");
    let second = SourceOverloadHandoff::build(
        input,
        &fixture.native,
        &fixture.symbols,
        &typed,
        &normalization,
    )
    .expect("build deterministic rerun");
    assert_eq!(resolved_root(&first), OverloadCandidateId::new(0));
    assert_eq!(first.selection(), second.selection());
    assert_eq!(first.debug_text(), second.debug_text());
}
