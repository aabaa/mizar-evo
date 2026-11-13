#[path = "support/source_overload.rs"]
mod support;

use mizar_checker::{
    overload_resolution::{
        ArgumentViabilityEvidence, CandidateDeclarationKind, CandidateOrigin, InsertedViewInput,
        InsertedViewKind, InsertedViewReasonKey, InsertedViewStatus, OverloadCandidateId,
        SpecificityComparisonInput, SpecificityComparisonStatus, SpecificityReasonKey,
        ViabilityFactStatus,
    },
    source_overload::{SourceOverloadHandoff, SourceOverloadInput},
    typed_ast::TypedAstError,
};
use support::{SourceFixture, ordinary_overloads, single_ordinary, source_view};

fn source_input(
    fixture: &SourceFixture,
) -> (
    SourceOverloadInput,
    mizar_checker::type_checker::TypeNormalizationOutput,
) {
    SourceOverloadInput::from_source(&fixture.native, &fixture.symbols, &fixture.typed)
        .expect("derive overload rows from parsed and resolved source")
}

fn build(
    fixture: &SourceFixture,
    input: SourceOverloadInput,
    normalization: &mizar_checker::type_checker::TypeNormalizationOutput,
) -> Result<SourceOverloadHandoff, String> {
    SourceOverloadHandoff::build(
        input,
        &fixture.native,
        &fixture.symbols,
        &fixture.typed,
        normalization,
    )
}

#[test]
fn missing_candidate_payload_and_foreign_source_anchors_refuse_transactionally() {
    let fixture = ordinary_overloads();
    let (canonical, normalization) = source_input(&fixture);
    assert!(canonical.candidates.len() >= 2);
    assert!(build(&fixture, canonical.clone(), &normalization).is_ok());

    let mut missing_site = canonical.clone();
    missing_site.sites.clear();
    assert!(build(&fixture, missing_site, &normalization).is_err());

    let mut missing_anchor = canonical.clone();
    missing_anchor.candidates[0].provenance.source_range = None;
    assert!(build(&fixture, missing_anchor, &normalization).is_err());

    let mut recovered_site = canonical.clone();
    recovered_site.sites[0].recovery =
        mizar_checker::overload_resolution::OverloadSiteRecovery::Degraded {
            message_key: mizar_checker::overload_resolution::OverloadDiagnosticMessageKey::new(
                "recovered-site",
            ),
        };
    assert!(build(&fixture, recovered_site, &normalization).is_err());

    let mut missing_candidate = canonical.clone();
    missing_candidate.candidates.pop();
    assert!(build(&fixture, missing_candidate, &normalization).is_err());

    let foreign = single_ordinary();
    let (foreign_input, _) = source_input(&foreign);

    let mut foreign_identity = canonical.clone();
    foreign_identity.source_id = foreign_input.source_id;
    assert!(build(&fixture, foreign_identity, &normalization).is_err());

    let mut foreign_site = canonical.clone();
    foreign_site.sites[0].owner = foreign_input.sites[0].owner.clone();
    assert!(build(&fixture, foreign_site, &normalization).is_err());

    let mut foreign_anchor = canonical.clone();
    foreign_anchor.sites[0].source_range = foreign_input.sites[0].source_range;
    assert!(build(&fixture, foreign_anchor, &normalization).is_err());

    let mut foreign_candidate_anchor = canonical.clone();
    foreign_candidate_anchor.candidates[0]
        .provenance
        .source_range = foreign_input.candidates[0].provenance.source_range;
    assert!(build(&fixture, foreign_candidate_anchor, &normalization).is_err());
}

#[test]
fn contested_site_without_comparison_evidence_does_not_select_a_winner() {
    let mut fixture = ordinary_overloads();
    let (mut input, normalization) = source_input(&fixture);
    assert!(input.candidates.len() >= 2);

    let actual = match &input.viability[1].arguments[0] {
        ArgumentViabilityEvidence::Exact { actual } => *actual,
        _ => panic!("source extraction records the argument's exact type"),
    };
    let (typed, fact_id) = fixture.with_known_fact(input.sites[0].arguments[0].clone(), "object");
    fixture.typed = typed;
    input.viability[1].arguments[0] = ArgumentViabilityEvidence::FactWidening {
        actual,
        target: input.candidates[1].parameters[0],
        facts: vec![fact_id],
        status: ViabilityFactStatus::Consumable,
    };

    input.comparisons.push(SpecificityComparisonInput {
        left: OverloadCandidateId::new(0),
        right: OverloadCandidateId::new(1),
        status: SpecificityComparisonStatus::LeftAtLeastRight,
        reasons: vec![SpecificityReasonKey::new("test-recorded-order")],
    });
    assert!(build(&fixture, input.clone(), &normalization).is_ok());

    input.comparisons.clear();
    assert!(build(&fixture, input, &normalization).is_err());
}

#[test]
fn equal_parameter_signatures_with_different_results_are_refused() {
    let baseline = single_ordinary();
    let (baseline_input, baseline_normalization) = source_input(&baseline);
    assert!(build(&baseline, baseline_input, &baseline_normalization).is_ok());

    let fixture = SourceFixture::parse(
        "definition let x be set; func OvSameSet: ovsame(x) -> set equals x; coherence; end;\n\
         definition let x be set; func OvSameObject: ovsame(x) -> object equals x; coherence; end;\n\
         reserve x for set; theorem OvSameUse: ovsame(x) = x;\n",
        "same_parameter_different_result",
    );

    // The source declarations carry identical argument signatures and distinct
    // result contracts. They cannot become ordinary overload alternatives.
    assert!(
        SourceOverloadInput::from_source(&fixture.native, &fixture.symbols, &fixture.typed)
            .is_err()
    );
}

#[test]
fn inserted_source_qua_view_without_written_basis_is_refused() {
    let fixture = source_view();
    let (mut input, normalization) = source_input(&fixture);
    let written = input.sites[0].source_qua[0].clone();
    let view = InsertedViewInput {
        argument: written.argument,
        target: written.target,
        selected_candidate: OverloadCandidateId::new(0),
        kind: InsertedViewKind::SourceQua,
        status: InsertedViewStatus::Accepted,
        reason: InsertedViewReasonKey::new("written-source-qua"),
        evidence_facts: written.evidence_facts,
        path: written.path,
    };
    input.sites[0].source_qua.clear();
    input.resolution[0].inserted_views.push(view);

    assert!(build(&fixture, input, &normalization).is_err());
}

#[test]
fn redefinition_candidate_is_refused_by_the_ordinary_source_owner() {
    let fixture = single_ordinary();
    let (mut input, normalization) = source_input(&fixture);
    let symbol = input.candidates[0].symbol.clone();
    input.candidates[0].declaration_kind = CandidateDeclarationKind::Redefinition;
    input.candidates[0].origin = CandidateOrigin::Redefinition { refined: symbol };

    assert!(build(&fixture, input, &normalization).is_err());
}

#[test]
fn duplicate_and_out_of_source_order_candidate_identities_are_refused() {
    let fixture = ordinary_overloads();
    let (canonical, normalization) = source_input(&fixture);
    assert!(canonical.candidates.len() >= 2);

    let mut duplicate = canonical.clone();
    duplicate.candidates.push(duplicate.candidates[0].clone());
    assert!(build(&fixture, duplicate, &normalization).is_err());

    let mut reordered = canonical.clone();
    reordered.candidates.reverse();
    assert!(build(&fixture, reordered, &normalization).is_err());
}

#[test]
fn typed_ast_installation_rejects_a_handoff_from_another_source_atomically() {
    let fixture = single_ordinary();
    let (input, normalization) = source_input(&fixture);
    let handoff = build(&fixture, input, &normalization).expect("single candidate handoff");
    let foreign = single_ordinary();
    let before = foreign.typed.clone();

    assert_eq!(
        foreign.typed.clone().with_source_overload(handoff),
        Err(TypedAstError::InvalidSourceOverload)
    );
    assert_eq!(foreign.typed, before);
    assert!(foreign.typed.source_overload().is_none());
}

#[test]
fn final_projection_refuses_each_mismatched_pipeline_stage() {
    use mizar_checker::{
        cluster_trace::ClusterFactTable,
        overload_resolution::{
            CandidateViabilityOutput, OverloadCollectionOutput, OverloadSelectionOutput,
            SpecificityGraphOutput, TemplateExpansionOutput,
        },
        resolved_typed_ast::{ResolvedTypedAst, ResolvedTypedAstError, ResolvedTypedAstInputs},
    };
    let fixture = single_ordinary();
    let (input, normalization) = source_input(&fixture);
    let handoff = build(&fixture, input, &normalization).unwrap();
    let installed = fixture
        .typed
        .clone()
        .with_source_overload(handoff.clone())
        .unwrap();
    let empty_collection = OverloadCollectionOutput::collect([], []);
    let empty_expansion = TemplateExpansionOutput::expand(&empty_collection);
    let empty_viability = CandidateViabilityOutput::filter(&empty_expansion, []);
    let empty_specificity = SpecificityGraphOutput::build(&empty_viability, []);
    let empty_selection = OverloadSelectionOutput::resolve(&empty_specificity, []);
    let cluster_facts = ClusterFactTable::new();
    for stage in 0..=5 {
        let mut inputs = ResolvedTypedAstInputs {
            typed_ast: &installed,
            cluster_facts: &cluster_facts,
            overload_collection: handoff.collection(),
            template_expansion: handoff.expansion(),
            viability: handoff.viability(),
            specificity: handoff.specificity(),
            overload_selection: handoff.selection(),
            expressions: Vec::new(),
            node_hints: Vec::new(),
            statement_semantics: None,
            statement_proofs: None,
        };
        match stage {
            0 => inputs.overload_collection = &empty_collection,
            1 => inputs.template_expansion = &empty_expansion,
            2 => inputs.viability = &empty_viability,
            3 => inputs.specificity = &empty_specificity,
            4 => inputs.overload_selection = &empty_selection,
            _ => {}
        }
        let result = ResolvedTypedAst::assemble(inputs);
        if stage < 5 {
            assert!(
                matches!(result, Err(ResolvedTypedAstError::InvalidSourceOverload)),
                "stage {stage}"
            );
        } else {
            assert_eq!(result.unwrap().source_overload(), Some(&handoff));
        }
    }
}

#[test]
fn template_source_with_untransported_parameters_is_refused() {
    let source = "definition let T be type; let x be T; func OvTemplate: ovtemplate[T](x) -> T equals x; coherence; end; reserve x for set; theorem OvUse: ovtemplate[set](x) = x;";
    for extra in [
        "let U be type;",
        "let U be type; let y be U such that y = y;",
    ] {
        let fixture = SourceFixture::parse(
            &source.replace("let x be T;", &format!("{extra} let x be T;")),
            "extra_template_parameter",
        );
        assert!(
            SourceOverloadInput::from_source(&fixture.native, &fixture.symbols, &fixture.typed)
                .is_err()
        );
    }
}
