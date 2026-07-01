use std::str::FromStr;

use mizar_diagnostics::{
    aggregator::{BuildDiagnosticIndex, DiagnosticAggregationInput},
    explain::{
        ExplanationHandle, ExplanationHandleId, ExplanationHandleInput, ExplanationKind,
        ExplanationPreview, ExplanationPreviewFormat, ExplanationSourceRef, ExplanationSubject,
    },
    failure_record::{
        DiagnosticDetailValue, DiagnosticDetails, DiagnosticDraft, DiagnosticDraftInput,
        DiagnosticFreshness, DiagnosticSpan, FailureCategory, PipelinePhase,
    },
    fix::{
        FixApplicability, FixCommandRef, FixEdit, FixSafety, FixSuggestion, FixSuggestionId,
        FixSuggestionInput,
    },
    registry::DiagnosticCode,
    sink::{DiagnosticBatch, DiagnosticProducerScope, DiagnosticSink},
};
use mizar_session::{
    BuildSnapshotId, Hash, InMemorySessionIdAllocator, SessionIdAllocator, SourceId, SourceRange,
};

#[test]
fn aggregation_is_independent_of_batch_and_production_order() {
    let snapshot = snapshot_id(1);
    let (source_a, source_b) = source_pair(snapshot);
    let parser = draft(
        DraftFixture::new(snapshot, source_b, PipelinePhase::Parser, 8, 9, "parser")
            .detail("agg.case", DiagnosticDetailValue::Integer(2)),
    );
    let resolver = draft(
        DraftFixture::new(
            snapshot,
            source_a,
            PipelinePhase::Resolver,
            2,
            3,
            "resolver",
        )
        .code("E0201")
        .category(FailureCategory::ResolveError)
        .stable_detail_key("resolve.ambiguous_symbol")
        .detail("agg.case", DiagnosticDetailValue::Integer(1)),
    );
    let parser_batch = batch(
        snapshot,
        PipelinePhase::Parser,
        "parser.recovery",
        vec![parser.clone()],
    );
    let resolver_batch = batch(
        snapshot,
        PipelinePhase::Resolver,
        "resolver.names",
        vec![resolver.clone()],
    );

    let forward = BuildDiagnosticIndex::from_batches(
        snapshot,
        vec![parser_batch.clone(), resolver_batch.clone()],
    )
    .expect("forward aggregation succeeds");
    let reversed = BuildDiagnosticIndex::from_batches(snapshot, vec![resolver_batch, parser_batch])
        .expect("reversed aggregation succeeds");

    assert_eq!(forward.debug_snapshot(), reversed.debug_snapshot());
    assert_eq!(forward.records()[0].message(), "resolver");
    assert_eq!(forward.records()[0].handle().id.get(), 0);
    assert_eq!(forward.records()[1].message(), "parser");
    assert_eq!(forward.records()[1].handle().id.get(), 1);
    assert_eq!(
        forward.handles_for_source(source_a),
        Some(&[forward.records()[0].handle()][..])
    );
    assert_eq!(
        forward.record_by_id(forward.records()[1].handle().id),
        Some(&forward.records()[1])
    );

    let parser_left = draft(
        DraftFixture::new(
            snapshot,
            source_a,
            PipelinePhase::Parser,
            2,
            3,
            "parser-left",
        )
        .detail("agg.case", DiagnosticDetailValue::Integer(1)),
    );
    let parser_right = draft(
        DraftFixture::new(
            snapshot,
            source_b,
            PipelinePhase::Parser,
            8,
            9,
            "parser-right",
        )
        .detail("agg.case", DiagnosticDetailValue::Integer(2)),
    );
    let multi_forward = BuildDiagnosticIndex::from_batches(
        snapshot,
        vec![batch(
            snapshot,
            PipelinePhase::Parser,
            "parser.recovery",
            vec![parser_right.clone(), parser_left.clone()],
        )],
    )
    .expect("multi-draft forward aggregation succeeds");
    let multi_reversed = BuildDiagnosticIndex::from_batches(
        snapshot,
        vec![batch(
            snapshot,
            PipelinePhase::Parser,
            "parser.recovery",
            vec![parser_left, parser_right],
        )],
    )
    .expect("multi-draft reversed aggregation succeeds");
    assert_eq!(
        multi_forward.debug_snapshot(),
        multi_reversed.debug_snapshot()
    );
}

#[test]
fn deduplication_ignores_message_text_and_uses_deterministic_representative() {
    let snapshot = snapshot_id(2);
    let source_id = source_id(snapshot);
    let later_message = draft(
        DraftFixture::new(
            snapshot,
            source_id,
            PipelinePhase::Parser,
            0,
            1,
            "zeta wording",
        )
        .detail("agg.case", DiagnosticDetailValue::Integer(1)),
    );
    let earlier_message = draft(
        DraftFixture::new(
            snapshot,
            source_id,
            PipelinePhase::Parser,
            0,
            1,
            "alpha wording",
        )
        .detail("agg.case", DiagnosticDetailValue::Integer(1)),
    );

    let index = BuildDiagnosticIndex::from_batches(
        snapshot,
        vec![batch(
            snapshot,
            PipelinePhase::Parser,
            "parser.recovery",
            vec![later_message, earlier_message],
        )],
    )
    .expect("aggregation succeeds");

    assert_eq!(index.len(), 1);
    assert_eq!(
        index.records()[0].code(),
        DiagnosticCode::from_str("E0001").unwrap()
    );
    assert_eq!(
        index.records()[0].stable_detail_key(),
        "syntax.unexpected_token"
    );
    assert_eq!(index.records()[0].message(), "alpha wording");
}

#[test]
fn duplicate_representative_choice_is_independent_of_duplicate_order() {
    let snapshot = snapshot_id(8);
    let source_id = source_id(snapshot);
    let alpha = draft(
        DraftFixture::new(
            snapshot,
            source_id,
            PipelinePhase::Parser,
            0,
            1,
            "alpha wording",
        )
        .detail("agg.case", DiagnosticDetailValue::Integer(1)),
    );
    let zeta = draft(
        DraftFixture::new(
            snapshot,
            source_id,
            PipelinePhase::Parser,
            0,
            1,
            "zeta wording",
        )
        .detail("agg.case", DiagnosticDetailValue::Integer(1)),
    );

    let forward = BuildDiagnosticIndex::from_batches(
        snapshot,
        vec![batch(
            snapshot,
            PipelinePhase::Parser,
            "parser.recovery",
            vec![zeta.clone(), alpha.clone()],
        )],
    )
    .expect("forward duplicate aggregation succeeds");
    let reversed = BuildDiagnosticIndex::from_batches(
        snapshot,
        vec![batch(
            snapshot,
            PipelinePhase::Parser,
            "parser.recovery",
            vec![alpha, zeta],
        )],
    )
    .expect("reversed duplicate aggregation succeeds");

    assert_eq!(forward.debug_snapshot(), reversed.debug_snapshot());
    assert_eq!(forward.len(), 1);
    assert_eq!(forward.records()[0].message(), "alpha wording");
}

#[test]
fn deduplication_keeps_diagnostic_code_identity_differences() {
    let snapshot = snapshot_id(9);
    let source_id = source_id(snapshot);
    let first = draft(
        DraftFixture::new(snapshot, source_id, PipelinePhase::Parser, 0, 1, "same")
            .detail("agg.case", DiagnosticDetailValue::Integer(1)),
    );
    let second = draft(
        DraftFixture::new(snapshot, source_id, PipelinePhase::Parser, 0, 1, "same")
            .code("E0002")
            .detail("agg.case", DiagnosticDetailValue::Integer(1)),
    );

    let index = BuildDiagnosticIndex::from_batches(
        snapshot,
        vec![batch(
            snapshot,
            PipelinePhase::Parser,
            "parser.recovery",
            vec![second, first],
        )],
    )
    .expect("aggregation succeeds");

    assert_eq!(index.len(), 2);
    assert_eq!(
        index
            .records()
            .iter()
            .map(|record| record.code())
            .collect::<Vec<_>>(),
        vec![
            DiagnosticCode::from_str("E0001").unwrap(),
            DiagnosticCode::from_str("E0002").unwrap(),
        ]
    );
}

#[test]
fn deduplication_keeps_each_core_identity_field_difference() {
    let snapshot = snapshot_id(16);
    let (source_id, other_source_id) = source_pair(snapshot);
    let detail = ("agg.case", DiagnosticDetailValue::Integer(1));

    assert_non_deduplicated(
        snapshot,
        vec![batch(
            snapshot,
            PipelinePhase::Parser,
            "parser.recovery",
            vec![
                draft(
                    DraftFixture::new(snapshot, source_id, PipelinePhase::Parser, 0, 1, "same")
                        .detail(detail.0, detail.1.clone()),
                ),
                draft(
                    DraftFixture::new(
                        snapshot,
                        other_source_id,
                        PipelinePhase::Parser,
                        0,
                        1,
                        "same",
                    )
                    .detail(detail.0, detail.1.clone()),
                ),
            ],
        )],
    );

    assert_non_deduplicated(
        snapshot,
        vec![batch(
            snapshot,
            PipelinePhase::Parser,
            "parser.recovery",
            vec![
                draft(
                    DraftFixture::new(snapshot, source_id, PipelinePhase::Parser, 0, 1, "same")
                        .detail(detail.0, detail.1.clone()),
                ),
                draft(
                    DraftFixture::new(snapshot, source_id, PipelinePhase::Parser, 2, 3, "same")
                        .detail(detail.0, detail.1.clone()),
                ),
            ],
        )],
    );

    assert_non_deduplicated(
        snapshot,
        vec![
            batch(
                snapshot,
                PipelinePhase::Parser,
                "parser.recovery",
                vec![draft(
                    DraftFixture::new(snapshot, source_id, PipelinePhase::Parser, 0, 1, "same")
                        .detail(detail.0, detail.1.clone()),
                )],
            ),
            batch(
                snapshot,
                PipelinePhase::Resolver,
                "resolver.names",
                vec![draft(
                    DraftFixture::new(snapshot, source_id, PipelinePhase::Resolver, 0, 1, "same")
                        .detail(detail.0, detail.1.clone()),
                )],
            ),
        ],
    );

    assert_non_deduplicated(
        snapshot,
        vec![batch(
            snapshot,
            PipelinePhase::Parser,
            "parser.recovery",
            vec![
                draft(
                    DraftFixture::new(snapshot, source_id, PipelinePhase::Parser, 0, 1, "same")
                        .detail(detail.0, detail.1.clone()),
                ),
                draft(
                    DraftFixture::new(snapshot, source_id, PipelinePhase::Parser, 0, 1, "same")
                        .category(FailureCategory::ResolveError)
                        .detail(detail.0, detail.1.clone()),
                ),
            ],
        )],
    );

    assert_non_deduplicated(
        snapshot,
        vec![batch(
            snapshot,
            PipelinePhase::Parser,
            "parser.recovery",
            vec![
                draft(
                    DraftFixture::new(snapshot, source_id, PipelinePhase::Parser, 0, 1, "same")
                        .detail(detail.0, detail.1.clone()),
                ),
                draft(
                    DraftFixture::new(snapshot, source_id, PipelinePhase::Parser, 0, 1, "same")
                        .stable_detail_key("syntax.alternate")
                        .detail(detail.0, detail.1),
                ),
            ],
        )],
    );
}

#[test]
fn deduplication_keeps_structured_identity_differences() {
    let snapshot = snapshot_id(3);
    let source_id = source_id(snapshot);
    let base = DraftFixture::new(snapshot, source_id, PipelinePhase::Parser, 0, 1, "same")
        .detail("agg.case", DiagnosticDetailValue::Integer(1));
    let detail_variant =
        DraftFixture::new(snapshot, source_id, PipelinePhase::Parser, 0, 1, "same")
            .detail("agg.case", DiagnosticDetailValue::Integer(2));
    let fix_variant = DraftFixture::new(snapshot, source_id, PipelinePhase::Parser, 0, 1, "same")
        .detail("agg.case", DiagnosticDetailValue::Integer(1))
        .fix("agg.insert_token");
    let explanation_variant =
        DraftFixture::new(snapshot, source_id, PipelinePhase::Parser, 0, 1, "same")
            .detail("agg.case", DiagnosticDetailValue::Integer(1))
            .explanation("agg.explain_token");

    let index = BuildDiagnosticIndex::from_batches(
        snapshot,
        vec![batch(
            snapshot,
            PipelinePhase::Parser,
            "parser.recovery",
            vec![
                draft(explanation_variant),
                draft(fix_variant),
                draft(detail_variant),
                draft(base),
            ],
        )],
    )
    .expect("aggregation succeeds");

    assert_eq!(index.len(), 4);
    assert_eq!(
        index
            .records()
            .iter()
            .map(|record| record.handle().id.get())
            .collect::<Vec<_>>(),
        vec![0, 1, 2, 3]
    );
    assert!(
        index
            .records()
            .iter()
            .any(|record| !record.fixes().is_empty())
    );
    assert!(
        index
            .records()
            .iter()
            .any(|record| record.explanation().is_some())
    );
}

#[test]
fn deduplication_keeps_distinct_fix_and_explanation_identities() {
    let snapshot = snapshot_id(10);
    let source_id = source_id(snapshot);
    let fix_a = draft(
        DraftFixture::new(snapshot, source_id, PipelinePhase::Parser, 0, 1, "same")
            .detail("agg.case", DiagnosticDetailValue::Integer(1))
            .fix("agg.fix_a"),
    );
    let fix_b = draft(
        DraftFixture::new(snapshot, source_id, PipelinePhase::Parser, 0, 1, "same")
            .detail("agg.case", DiagnosticDetailValue::Integer(1))
            .fix("agg.fix_b"),
    );
    let exp_a = draft(
        DraftFixture::new(snapshot, source_id, PipelinePhase::Parser, 2, 3, "same")
            .detail("agg.case", DiagnosticDetailValue::Integer(1))
            .explanation("agg.exp_a"),
    );
    let exp_b = draft(
        DraftFixture::new(snapshot, source_id, PipelinePhase::Parser, 2, 3, "same")
            .detail("agg.case", DiagnosticDetailValue::Integer(1))
            .explanation("agg.exp_b"),
    );

    let index = BuildDiagnosticIndex::from_batches(
        snapshot,
        vec![batch(
            snapshot,
            PipelinePhase::Parser,
            "parser.recovery",
            vec![fix_b, exp_b, fix_a, exp_a],
        )],
    )
    .expect("aggregation succeeds");

    assert_eq!(index.len(), 4);
    assert_eq!(
        index
            .records()
            .iter()
            .filter(|record| !record.fixes().is_empty())
            .map(|record| record.fixes()[0].id().identity())
            .collect::<Vec<_>>(),
        vec!["agg.fix_a", "agg.fix_b"]
    );
    assert_eq!(
        index
            .records()
            .iter()
            .filter_map(|record| record.explanation())
            .map(|explanation| explanation.id().identity())
            .collect::<Vec<_>>(),
        vec!["agg.exp_a", "agg.exp_b"]
    );
}

#[test]
fn deduplication_uses_structured_fix_payload_but_not_title_text() {
    let snapshot = snapshot_id(17);
    let source_id = source_id(snapshot);
    let alpha_title = draft(
        DraftFixture::new(snapshot, source_id, PipelinePhase::Parser, 0, 1, "same")
            .detail("agg.case", DiagnosticDetailValue::Integer(1))
            .structured_fix(text_fix(
                source_id,
                "agg.structured_fix",
                "alpha title",
                FixSafety::LocalTextEdit,
                None,
            )),
    );
    let zeta_title = draft(
        DraftFixture::new(snapshot, source_id, PipelinePhase::Parser, 0, 1, "same")
            .detail("agg.case", DiagnosticDetailValue::Integer(1))
            .structured_fix(text_fix(
                source_id,
                "agg.structured_fix",
                "zeta title",
                FixSafety::LocalTextEdit,
                None,
            )),
    );

    let title_only_index = BuildDiagnosticIndex::from_batches(
        snapshot,
        vec![batch(
            snapshot,
            PipelinePhase::Parser,
            "parser.recovery",
            vec![zeta_title, alpha_title],
        )],
    )
    .expect("aggregation succeeds");
    assert_eq!(title_only_index.len(), 1);
    assert_eq!(
        title_only_index.records()[0].fixes()[0].title(),
        "alpha title"
    );

    let local = draft(
        DraftFixture::new(snapshot, source_id, PipelinePhase::Parser, 0, 1, "same")
            .detail("agg.case", DiagnosticDetailValue::Integer(1))
            .structured_fix(text_fix(
                source_id,
                "agg.structured_fix",
                "same title",
                FixSafety::LocalTextEdit,
                None,
            )),
    );
    let snapshot_bound = draft(
        DraftFixture::new(snapshot, source_id, PipelinePhase::Parser, 0, 1, "same")
            .detail("agg.case", DiagnosticDetailValue::Integer(1))
            .structured_fix(text_fix(
                source_id,
                "agg.structured_fix",
                "same title",
                FixSafety::SnapshotBound,
                Some(snapshot),
            )),
    );

    let payload_index = BuildDiagnosticIndex::from_batches(
        snapshot,
        vec![batch(
            snapshot,
            PipelinePhase::Parser,
            "parser.recovery",
            vec![snapshot_bound, local],
        )],
    )
    .expect("aggregation succeeds");
    assert_eq!(payload_index.len(), 2);
    assert_eq!(
        payload_index
            .records()
            .iter()
            .map(|record| record.fixes()[0].safety())
            .collect::<Vec<_>>(),
        vec![FixSafety::LocalTextEdit, FixSafety::SnapshotBound]
    );
}

#[test]
fn deduplication_includes_each_canonical_fix_payload_field_and_normalizes_fix_order() {
    let snapshot = snapshot_id(18);
    let other_snapshot = snapshot_id(19);
    let (source_id, alternate_fix_source_id) = source_pair(snapshot);
    let cases = [
        (
            "id",
            structured_text_fix(StructuredFixFixture::new(source_id).identity("agg.fix_id_a")),
            structured_text_fix(StructuredFixFixture::new(source_id).identity("agg.fix_id_b")),
        ),
        (
            "producer key",
            structured_text_fix(
                StructuredFixFixture::new(source_id).producer_key(Some("agg.producer_a")),
            ),
            structured_text_fix(
                StructuredFixFixture::new(source_id).producer_key(Some("agg.producer_b")),
            ),
        ),
        (
            "applicability",
            structured_text_fix(
                StructuredFixFixture::new(source_id)
                    .applicability(FixApplicability::MachineApplicable),
            ),
            structured_text_fix(
                StructuredFixFixture::new(source_id)
                    .applicability(FixApplicability::MaybeIncorrect),
            ),
        ),
        (
            "safety",
            structured_text_fix(
                StructuredFixFixture::new(source_id)
                    .safety(FixSafety::LocalTextEdit)
                    .required_snapshot(Some(snapshot)),
            ),
            structured_text_fix(
                StructuredFixFixture::new(source_id)
                    .safety(FixSafety::SnapshotBound)
                    .required_snapshot(Some(snapshot)),
            ),
        ),
        (
            "edit source",
            structured_text_fix(StructuredFixFixture::new(source_id)),
            structured_text_fix(StructuredFixFixture::new(alternate_fix_source_id)),
        ),
        (
            "edit range",
            structured_text_fix(StructuredFixFixture::new(source_id).range(1, 1)),
            structured_text_fix(StructuredFixFixture::new(source_id).range(2, 2)),
        ),
        (
            "replacement",
            structured_text_fix(StructuredFixFixture::new(source_id).replacement(" inserted")),
            structured_text_fix(StructuredFixFixture::new(source_id).replacement(" replacement")),
        ),
        (
            "expected text",
            structured_text_fix(StructuredFixFixture::new(source_id).expected_text(Some(""))),
            structured_text_fix(StructuredFixFixture::new(source_id).expected_text(Some("old"))),
        ),
        (
            "snapshot precondition",
            structured_text_fix(
                StructuredFixFixture::new(source_id)
                    .safety(FixSafety::SnapshotBound)
                    .required_snapshot(Some(snapshot)),
            ),
            structured_text_fix(
                StructuredFixFixture::new(source_id)
                    .safety(FixSafety::SnapshotBound)
                    .required_snapshot(Some(other_snapshot)),
            ),
        ),
        (
            "hash precondition",
            structured_text_fix(
                StructuredFixFixture::new(source_id)
                    .safety(FixSafety::ArtifactAssisted)
                    .required_text_hash(Some(Hash::from_bytes([1; Hash::BYTE_LEN]))),
            ),
            structured_text_fix(
                StructuredFixFixture::new(source_id)
                    .safety(FixSafety::ArtifactAssisted)
                    .required_text_hash(Some(Hash::from_bytes([2; Hash::BYTE_LEN]))),
            ),
        ),
        (
            "command reference",
            command_fix("agg.command_fix", "agg.command_a"),
            command_fix("agg.command_fix", "agg.command_b"),
        ),
    ];

    for (label, left_fix, right_fix) in cases {
        let left = draft(
            DraftFixture::new(snapshot, source_id, PipelinePhase::Parser, 0, 1, "same")
                .detail("agg.case", DiagnosticDetailValue::Integer(1))
                .structured_fix(left_fix),
        );
        let right = draft(
            DraftFixture::new(snapshot, source_id, PipelinePhase::Parser, 0, 1, "same")
                .detail("agg.case", DiagnosticDetailValue::Integer(1))
                .structured_fix(right_fix),
        );
        let index = BuildDiagnosticIndex::from_batches(
            snapshot,
            vec![batch(
                snapshot,
                PipelinePhase::Parser,
                "parser.recovery",
                vec![right, left],
            )],
        )
        .expect("aggregation succeeds");
        assert_eq!(index.len(), 2, "{label}");
    }

    let fix_a = structured_text_fix(StructuredFixFixture::new(source_id).identity("agg.order_a"));
    let fix_b = structured_text_fix(
        StructuredFixFixture::new(source_id)
            .identity("agg.order_b")
            .range(2, 2),
    );
    let forward = draft(
        DraftFixture::new(snapshot, source_id, PipelinePhase::Parser, 4, 5, "same")
            .detail("agg.case", DiagnosticDetailValue::Integer(2))
            .structured_fix(fix_b.clone())
            .structured_fix(fix_a.clone()),
    );
    let reversed = draft(
        DraftFixture::new(snapshot, source_id, PipelinePhase::Parser, 4, 5, "same")
            .detail("agg.case", DiagnosticDetailValue::Integer(2))
            .structured_fix(fix_a)
            .structured_fix(fix_b),
    );
    let index = BuildDiagnosticIndex::from_batches(
        snapshot,
        vec![batch(
            snapshot,
            PipelinePhase::Parser,
            "parser.recovery",
            vec![forward, reversed],
        )],
    )
    .expect("aggregation succeeds");
    assert_eq!(index.len(), 1);
    assert_eq!(
        index.records()[0]
            .fixes()
            .iter()
            .map(|fix| fix.id().identity())
            .collect::<Vec<_>>(),
        vec!["agg.order_a", "agg.order_b"]
    );
}

#[test]
fn deduplication_includes_canonical_explanation_fields_but_not_preview_text() {
    let snapshot = snapshot_id(20);
    let source_id = source_id(snapshot);
    let same_identity_alpha = draft(
        DraftFixture::new(snapshot, source_id, PipelinePhase::Parser, 0, 1, "same")
            .detail("agg.case", DiagnosticDetailValue::Integer(1))
            .structured_explanation(structured_explanation(
                StructuredExplanationFixture::new("agg.explain_canonical").preview("alpha"),
            )),
    );
    let same_identity_zeta = draft(
        DraftFixture::new(snapshot, source_id, PipelinePhase::Parser, 0, 1, "same")
            .detail("agg.case", DiagnosticDetailValue::Integer(1))
            .structured_explanation(structured_explanation(
                StructuredExplanationFixture::new("agg.explain_canonical").preview("zeta"),
            )),
    );

    let preview_index = BuildDiagnosticIndex::from_batches(
        snapshot,
        vec![batch(
            snapshot,
            PipelinePhase::Parser,
            "parser.recovery",
            vec![same_identity_zeta, same_identity_alpha],
        )],
    )
    .expect("aggregation succeeds");
    assert_eq!(preview_index.len(), 1);
    assert_eq!(
        preview_index.records()[0]
            .explanation()
            .and_then(ExplanationHandle::preview)
            .map(ExplanationPreview::text),
        Some("alpha")
    );

    let cases = vec![
        (
            "id",
            structured_explanation(StructuredExplanationFixture::new("agg.explain_id_a")),
            structured_explanation(StructuredExplanationFixture::new("agg.explain_id_b")),
        ),
        (
            "kind",
            structured_explanation(
                StructuredExplanationFixture::new("agg.explain_canonical")
                    .kind(ExplanationKind::DiagnosticContext),
            ),
            structured_explanation(
                StructuredExplanationFixture::new("agg.explain_canonical")
                    .kind(ExplanationKind::ProofFailure),
            ),
        ),
        (
            "subject",
            structured_explanation(
                StructuredExplanationFixture::new("agg.explain_canonical")
                    .subject(ExplanationSubject::Expression("expr.left".to_owned())),
            ),
            structured_explanation(
                StructuredExplanationFixture::new("agg.explain_canonical")
                    .subject(ExplanationSubject::Expression("expr.right".to_owned())),
            ),
        ),
        (
            "source",
            structured_explanation(
                StructuredExplanationFixture::new("agg.explain_canonical")
                    .source(ExplanationSourceRef::PreviewOnly),
            ),
            structured_explanation(
                StructuredExplanationFixture::new("agg.explain_canonical").source(
                    ExplanationSourceRef::QueryService {
                        service_key: "agg.explain_service".to_owned(),
                        query_key: "agg.query_key".to_owned(),
                    },
                ),
            ),
        ),
        (
            "snapshot precondition",
            structured_explanation(StructuredExplanationFixture::new("agg.explain_canonical")),
            structured_explanation(
                StructuredExplanationFixture::new("agg.explain_canonical")
                    .required_snapshot(Some(snapshot)),
            ),
        ),
        (
            "artifact hash precondition",
            structured_explanation(
                StructuredExplanationFixture::new("agg.explain_canonical")
                    .required_artifact_hash(Some(Hash::from_bytes([1; Hash::BYTE_LEN]))),
            ),
            structured_explanation(
                StructuredExplanationFixture::new("agg.explain_canonical")
                    .required_artifact_hash(Some(Hash::from_bytes([2; Hash::BYTE_LEN]))),
            ),
        ),
        (
            "summary hash",
            structured_explanation(
                StructuredExplanationFixture::new("agg.explain_canonical")
                    .summary_hash(Some(Hash::from_bytes([3; Hash::BYTE_LEN]))),
            ),
            structured_explanation(
                StructuredExplanationFixture::new("agg.explain_canonical")
                    .summary_hash(Some(Hash::from_bytes([4; Hash::BYTE_LEN]))),
            ),
        ),
    ];

    for (label, left_explanation, right_explanation) in cases {
        let left = draft(
            DraftFixture::new(snapshot, source_id, PipelinePhase::Parser, 2, 3, "same")
                .detail("agg.case", DiagnosticDetailValue::Integer(2))
                .structured_explanation(left_explanation),
        );
        let right = draft(
            DraftFixture::new(snapshot, source_id, PipelinePhase::Parser, 2, 3, "same")
                .detail("agg.case", DiagnosticDetailValue::Integer(2))
                .structured_explanation(right_explanation),
        );
        let index = BuildDiagnosticIndex::from_batches(
            snapshot,
            vec![batch(
                snapshot,
                PipelinePhase::Parser,
                "parser.recovery",
                vec![right, left],
            )],
        )
        .expect("aggregation succeeds");
        assert_eq!(index.len(), 2, "{label}");
    }
}

#[test]
fn obsolete_snapshot_drafts_are_withheld_from_current_publication() {
    let current = snapshot_id(4);
    let stale = snapshot_id(5);
    let current_source = source_id(current);
    let stale_source = source_id(stale);
    let current_draft = draft(
        DraftFixture::new(
            current,
            current_source,
            PipelinePhase::Parser,
            0,
            1,
            "current",
        )
        .detail("agg.case", DiagnosticDetailValue::Integer(0)),
    );
    let stale_draft = draft(
        DraftFixture::new(stale, stale_source, PipelinePhase::Parser, 0, 1, "stale")
            .detail("agg.case", DiagnosticDetailValue::Integer(0)),
    );

    let index = BuildDiagnosticIndex::from_batches(
        current,
        vec![
            batch(
                stale,
                PipelinePhase::Parser,
                "parser.stale",
                vec![stale_draft],
            ),
            batch(
                current,
                PipelinePhase::Parser,
                "parser.current",
                vec![current_draft],
            ),
        ],
    )
    .expect("aggregation succeeds");

    assert_eq!(index.len(), 1);
    assert_eq!(index.obsolete_drafts().len(), 1);
    assert_eq!(index.obsolete_drafts()[0].source_snapshot(), stale);
    assert_eq!(index.obsolete_drafts()[0].producer_name(), "parser.stale");
    assert!(matches!(
        index.records()[0].freshness(),
        DiagnosticFreshness::Current { source_snapshot } if *source_snapshot == current
    ));
}

#[test]
fn obsolete_accounting_is_independent_of_stale_input_order() {
    let current = snapshot_id(11);
    let stale_a = snapshot_id(12);
    let stale_b = snapshot_id(13);
    let stale_a_source = source_id(stale_a);
    let stale_b_source = source_id(stale_b);
    let stale_a_draft = draft(
        DraftFixture::new(
            stale_a,
            stale_a_source,
            PipelinePhase::Parser,
            4,
            5,
            "stale-a",
        )
        .detail("agg.case", DiagnosticDetailValue::Integer(2)),
    );
    let stale_b_draft = draft(
        DraftFixture::new(
            stale_b,
            stale_b_source,
            PipelinePhase::Parser,
            0,
            1,
            "stale-b",
        )
        .detail("agg.case", DiagnosticDetailValue::Integer(1)),
    );

    let forward = BuildDiagnosticIndex::from_batches(
        current,
        vec![
            batch(
                stale_b,
                PipelinePhase::Parser,
                "parser.z",
                vec![stale_b_draft.clone()],
            ),
            batch(
                stale_a,
                PipelinePhase::Parser,
                "parser.a",
                vec![stale_a_draft.clone()],
            ),
        ],
    )
    .expect("forward aggregation succeeds");
    let reversed = BuildDiagnosticIndex::from_batches(
        current,
        vec![
            batch(
                stale_a,
                PipelinePhase::Parser,
                "parser.a",
                vec![stale_a_draft],
            ),
            batch(
                stale_b,
                PipelinePhase::Parser,
                "parser.z",
                vec![stale_b_draft],
            ),
        ],
    )
    .expect("reversed aggregation succeeds");

    assert_eq!(forward.records(), &[]);
    assert_eq!(forward.obsolete_drafts().len(), 2);
    assert_eq!(forward.debug_snapshot(), reversed.debug_snapshot());
    assert_eq!(forward.obsolete_drafts()[0].source_snapshot(), stale_a);
    assert_eq!(forward.obsolete_drafts()[1].source_snapshot(), stale_b);
}

#[test]
fn obsolete_accounting_uses_same_snapshot_tie_breakers() {
    let current = snapshot_id(14);
    let stale = snapshot_id(15);
    let stale_source = source_id(stale);
    let producer_z = draft(
        DraftFixture::new(
            stale,
            stale_source,
            PipelinePhase::Parser,
            0,
            1,
            "producer-z",
        )
        .detail("agg.case", DiagnosticDetailValue::Integer(1)),
    );
    let producer_a = draft(
        DraftFixture::new(
            stale,
            stale_source,
            PipelinePhase::Parser,
            0,
            1,
            "producer-a",
        )
        .detail("agg.case", DiagnosticDetailValue::Integer(1)),
    );
    let local_z = draft(
        DraftFixture::new(stale, stale_source, PipelinePhase::Parser, 0, 1, "z-local")
            .detail("agg.case", DiagnosticDetailValue::Integer(1)),
    );
    let local_a = draft(
        DraftFixture::new(stale, stale_source, PipelinePhase::Parser, 0, 1, "a-local")
            .detail("agg.case", DiagnosticDetailValue::Integer(1)),
    );
    let tie_z = draft(
        DraftFixture::new(stale, stale_source, PipelinePhase::Parser, 0, 1, "z-tie")
            .detail("agg.case", DiagnosticDetailValue::Integer(1)),
    );
    let tie_a = draft(
        DraftFixture::new(stale, stale_source, PipelinePhase::Parser, 0, 1, "a-tie")
            .detail("agg.case", DiagnosticDetailValue::Integer(1)),
    );

    let index = BuildDiagnosticIndex::from_batches(
        current,
        vec![
            batch(stale, PipelinePhase::Parser, "parser.z", vec![producer_z]),
            batch(stale, PipelinePhase::Parser, "parser.tie", vec![tie_z]),
            batch(
                stale,
                PipelinePhase::Parser,
                "parser.m",
                vec![local_z, local_a],
            ),
            batch(stale, PipelinePhase::Parser, "parser.tie", vec![tie_a]),
            batch(stale, PipelinePhase::Parser, "parser.a", vec![producer_a]),
        ],
    )
    .expect("aggregation succeeds");

    assert_eq!(
        index
            .obsolete_drafts()
            .iter()
            .map(|obsolete| {
                (
                    obsolete.producer_name(),
                    obsolete.local_ordinal(),
                    obsolete.draft().message(),
                )
            })
            .collect::<Vec<_>>(),
        vec![
            ("parser.a", 0, "producer-a"),
            ("parser.m", 0, "z-local"),
            ("parser.m", 1, "a-local"),
            ("parser.tie", 0, "a-tie"),
            ("parser.tie", 0, "z-tie"),
            ("parser.z", 0, "producer-z"),
        ]
    );
}

#[test]
fn index_debug_snapshot_is_byte_stable() {
    let current = snapshot_id(6);
    let stale = snapshot_id(7);
    let current_source = source_id(current);
    let stale_source = source_id(stale);
    let current_draft = draft(
        DraftFixture::new(
            current,
            current_source,
            PipelinePhase::Parser,
            0,
            1,
            "current",
        )
        .detail("agg.case", DiagnosticDetailValue::Integer(0)),
    );
    let stale_draft = draft(
        DraftFixture::new(stale, stale_source, PipelinePhase::Parser, 0, 1, "stale")
            .detail("agg.case", DiagnosticDetailValue::Integer(0)),
    );
    let index = BuildDiagnosticIndex::aggregate(DiagnosticAggregationInput::new(
        current,
        vec![
            batch(
                stale,
                PipelinePhase::Parser,
                "parser.stale",
                vec![stale_draft],
            ),
            batch(
                current,
                PipelinePhase::Parser,
                "parser.current",
                vec![current_draft],
            ),
        ],
    ))
    .expect("aggregation succeeds");

    assert_eq!(
        index.debug_snapshot(),
        concat!(
            "kind=index\n",
            "snapshot=mizar-session-build-snapshot-v1:",
            "0606060606060606060606060606060606060606060606060606060606060606\n",
            "record_count=1\n",
            "obsolete_count=1\n",
            "record[0]=kind=record\\nhandle=mizar-session-build-snapshot-v1:",
            "0606060606060606060606060606060606060606060606060606060606060606#0\\n",
            "code=E0001\\nsemantic_name=\"syntax.unexpected_token\"\\nseverity=error\\n",
            "phase=parser\\ncategory=parse_error\\n",
            "stable_detail_key=\"syntax.unexpected_token\"\\nmessage=\"current\"\\n",
            "source_snapshot=mizar-session-build-snapshot-v1:",
            "0606060606060606060606060606060606060606060606060606060606060606\\n",
            "freshness=current(source_snapshot=mizar-session-build-snapshot-v1:",
            "0606060606060606060606060606060606060606060606060606060606060606)\\n",
            "primary=SourceId(OpaqueId(1)):0..1:primary:current:none:none\\n",
            "secondary=[]\\nnotes=[]\\ndetails={agg.case=int:0}\\nfixes=[]\\n",
            "explanation=none\\nrelated=[]\n",
            "obsolete[0]=source_snapshot=mizar-session-build-snapshot-v1:",
            "0707070707070707070707070707070707070707070707070707070707070707;",
            "producer_name=\"parser.stale\";local_ordinal=0;draft=kind=draft\\n",
            "handle=none\\ncode=E0001\\nsemantic_name=none\\nseverity=none\\n",
            "phase=parser\\ncategory=parse_error\\n",
            "stable_detail_key=\"syntax.unexpected_token\"\\nmessage=\"stale\"\\n",
            "source_snapshot=mizar-session-build-snapshot-v1:",
            "0707070707070707070707070707070707070707070707070707070707070707\\n",
            "freshness=draft\\nprimary=SourceId(OpaqueId(1)):0..1:primary:current:none:none\\n",
            "secondary=[]\\nnotes=[]\\ndetails={agg.case=int:0}\\nfixes=[]\\n",
            "explanation=none\\nrelated=[]\n",
        )
    );
}

#[derive(Clone)]
struct DraftFixture {
    snapshot: BuildSnapshotId,
    source_id: SourceId,
    phase: PipelinePhase,
    start: usize,
    end: usize,
    message: &'static str,
    code: &'static str,
    category: FailureCategory,
    stable_detail_key: &'static str,
    details: Vec<(&'static str, DiagnosticDetailValue)>,
    fixes: Vec<FixSuggestion>,
    explanation: Option<ExplanationHandle>,
}

impl DraftFixture {
    fn new(
        snapshot: BuildSnapshotId,
        source_id: SourceId,
        phase: PipelinePhase,
        start: usize,
        end: usize,
        message: &'static str,
    ) -> Self {
        Self {
            snapshot,
            source_id,
            phase,
            start,
            end,
            message,
            code: "E0001",
            category: FailureCategory::ParseError,
            stable_detail_key: "syntax.unexpected_token",
            details: Vec::new(),
            fixes: Vec::new(),
            explanation: None,
        }
    }

    fn code(mut self, code: &'static str) -> Self {
        self.code = code;
        self
    }

    fn category(mut self, category: FailureCategory) -> Self {
        self.category = category;
        self
    }

    fn stable_detail_key(mut self, stable_detail_key: &'static str) -> Self {
        self.stable_detail_key = stable_detail_key;
        self
    }

    fn detail(mut self, key: &'static str, value: DiagnosticDetailValue) -> Self {
        self.details.push((key, value));
        self
    }

    fn fix(mut self, identity: &'static str) -> Self {
        self.fixes.push(informational_fix(identity, identity));
        self
    }

    fn structured_fix(mut self, fix: FixSuggestion) -> Self {
        self.fixes.push(fix);
        self
    }

    fn explanation(mut self, identity: &'static str) -> Self {
        self.explanation = Some(explanation(identity, identity));
        self
    }

    fn structured_explanation(mut self, explanation: ExplanationHandle) -> Self {
        self.explanation = Some(explanation);
        self
    }
}

fn draft(fixture: DraftFixture) -> DiagnosticDraft {
    DiagnosticDraft::new(DiagnosticDraftInput {
        source_snapshot: fixture.snapshot,
        code: DiagnosticCode::from_str(fixture.code).expect("allocated code"),
        phase: fixture.phase,
        category: fixture.category,
        stable_detail_key: fixture.stable_detail_key.to_owned(),
        message: fixture.message.to_owned(),
        primary_span: DiagnosticSpan::primary(
            SourceRange {
                source_id: fixture.source_id,
                start: fixture.start,
                end: fixture.end,
            },
            None,
        )
        .expect("valid primary span"),
        secondary_spans: vec![],
        notes: vec![],
        details: DiagnosticDetails::from_entries(fixture.details).expect("valid details"),
        fixes: fixture.fixes,
        explanation: fixture.explanation,
    })
    .expect("valid draft")
}

fn batch(
    snapshot: BuildSnapshotId,
    phase: PipelinePhase,
    producer_name: &'static str,
    drafts: Vec<DiagnosticDraft>,
) -> DiagnosticBatch {
    let mut sink =
        DiagnosticSink::new(DiagnosticProducerScope::new(phase, snapshot, producer_name));
    for draft in drafts {
        sink.emit(draft).expect("fixture draft matches sink scope");
    }
    sink.into_batch()
}

fn assert_non_deduplicated(snapshot: BuildSnapshotId, batches: Vec<DiagnosticBatch>) {
    let index =
        BuildDiagnosticIndex::from_batches(snapshot, batches).expect("aggregation succeeds");
    assert_eq!(index.len(), 2);
}

fn informational_fix(identity: &str, title: &str) -> FixSuggestion {
    FixSuggestion::informational(
        FixSuggestionId::new(identity).expect("valid fix identity"),
        title,
    )
    .expect("valid informational fix")
}

fn text_fix(
    source_id: SourceId,
    identity: &str,
    title: &str,
    safety: FixSafety,
    required_snapshot: Option<BuildSnapshotId>,
) -> FixSuggestion {
    FixSuggestion::new(FixSuggestionInput {
        id: FixSuggestionId::new(identity).expect("valid fix identity"),
        producer_key: Some("agg.producer_fix".to_owned()),
        title: title.to_owned(),
        applicability: FixApplicability::MachineApplicable,
        safety,
        edits: vec![
            FixEdit::new(
                SourceRange {
                    source_id,
                    start: 1,
                    end: 1,
                },
                " inserted",
                Some(String::new()),
            )
            .expect("valid edit"),
        ],
        command: None,
        required_snapshot,
        required_text_hash: None,
    })
    .expect("valid text fix")
}

#[derive(Clone)]
struct StructuredFixFixture {
    source_id: SourceId,
    identity: &'static str,
    producer_key: Option<&'static str>,
    title: &'static str,
    applicability: FixApplicability,
    safety: FixSafety,
    start: usize,
    end: usize,
    replacement: &'static str,
    expected_text: Option<&'static str>,
    required_snapshot: Option<BuildSnapshotId>,
    required_text_hash: Option<Hash>,
}

impl StructuredFixFixture {
    fn new(source_id: SourceId) -> Self {
        Self {
            source_id,
            identity: "agg.structured_fix",
            producer_key: Some("agg.producer_fix"),
            title: "structured fix",
            applicability: FixApplicability::MachineApplicable,
            safety: FixSafety::LocalTextEdit,
            start: 1,
            end: 1,
            replacement: " inserted",
            expected_text: Some(""),
            required_snapshot: None,
            required_text_hash: None,
        }
    }

    fn identity(mut self, identity: &'static str) -> Self {
        self.identity = identity;
        self
    }

    fn producer_key(mut self, producer_key: Option<&'static str>) -> Self {
        self.producer_key = producer_key;
        self
    }

    fn applicability(mut self, applicability: FixApplicability) -> Self {
        self.applicability = applicability;
        self
    }

    fn safety(mut self, safety: FixSafety) -> Self {
        self.safety = safety;
        self
    }

    fn range(mut self, start: usize, end: usize) -> Self {
        self.start = start;
        self.end = end;
        self
    }

    fn replacement(mut self, replacement: &'static str) -> Self {
        self.replacement = replacement;
        self
    }

    fn expected_text(mut self, expected_text: Option<&'static str>) -> Self {
        self.expected_text = expected_text;
        self
    }

    fn required_snapshot(mut self, required_snapshot: Option<BuildSnapshotId>) -> Self {
        self.required_snapshot = required_snapshot;
        self
    }

    fn required_text_hash(mut self, required_text_hash: Option<Hash>) -> Self {
        self.required_text_hash = required_text_hash;
        self
    }
}

fn structured_text_fix(fixture: StructuredFixFixture) -> FixSuggestion {
    FixSuggestion::new(FixSuggestionInput {
        id: FixSuggestionId::new(fixture.identity).expect("valid fix identity"),
        producer_key: fixture.producer_key.map(str::to_owned),
        title: fixture.title.to_owned(),
        applicability: fixture.applicability,
        safety: fixture.safety,
        edits: vec![
            FixEdit::new(
                SourceRange {
                    source_id: fixture.source_id,
                    start: fixture.start,
                    end: fixture.end,
                },
                fixture.replacement,
                fixture.expected_text.map(str::to_owned),
            )
            .expect("valid edit"),
        ],
        command: None,
        required_snapshot: fixture.required_snapshot,
        required_text_hash: fixture.required_text_hash,
    })
    .expect("valid structured text fix")
}

fn command_fix(identity: &str, command: &str) -> FixSuggestion {
    FixSuggestion::new(FixSuggestionInput {
        id: FixSuggestionId::new(identity).expect("valid fix identity"),
        producer_key: Some("agg.command_producer".to_owned()),
        title: "command fix".to_owned(),
        applicability: FixApplicability::MaybeIncorrect,
        safety: FixSafety::CommandOnly,
        edits: Vec::new(),
        command: Some(FixCommandRef::new(command).expect("valid command ref")),
        required_snapshot: None,
        required_text_hash: None,
    })
    .expect("valid command fix")
}

fn explanation(identity: &str, preview: &str) -> ExplanationHandle {
    ExplanationHandle::preview_only(
        ExplanationHandleId::new(identity).expect("valid explanation identity"),
        DiagnosticCode::from_str("E0001").expect("allocated code"),
        "syntax.unexpected_token",
        Some(ExplanationPreview::new(
            ExplanationPreviewFormat::PlainText,
            preview,
        )),
    )
    .expect("valid explanation handle")
}

#[derive(Clone)]
struct StructuredExplanationFixture {
    identity: &'static str,
    kind: ExplanationKind,
    subject: ExplanationSubject,
    source: ExplanationSourceRef,
    required_snapshot: Option<BuildSnapshotId>,
    required_artifact_hash: Option<Hash>,
    summary_hash: Option<Hash>,
    preview: &'static str,
}

impl StructuredExplanationFixture {
    fn new(identity: &'static str) -> Self {
        Self {
            identity,
            kind: ExplanationKind::DiagnosticContext,
            subject: ExplanationSubject::Diagnostic {
                code: DiagnosticCode::from_str("E0001").expect("allocated code"),
                stable_detail_key: "syntax.unexpected_token".to_owned(),
            },
            source: ExplanationSourceRef::PreviewOnly,
            required_snapshot: None,
            required_artifact_hash: None,
            summary_hash: None,
            preview: "preview",
        }
    }

    fn kind(mut self, kind: ExplanationKind) -> Self {
        self.kind = kind;
        self
    }

    fn subject(mut self, subject: ExplanationSubject) -> Self {
        self.subject = subject;
        self
    }

    fn source(mut self, source: ExplanationSourceRef) -> Self {
        self.source = source;
        self
    }

    fn required_snapshot(mut self, required_snapshot: Option<BuildSnapshotId>) -> Self {
        self.required_snapshot = required_snapshot;
        self
    }

    fn required_artifact_hash(mut self, required_artifact_hash: Option<Hash>) -> Self {
        self.required_artifact_hash = required_artifact_hash;
        self
    }

    fn summary_hash(mut self, summary_hash: Option<Hash>) -> Self {
        self.summary_hash = summary_hash;
        self
    }

    fn preview(mut self, preview: &'static str) -> Self {
        self.preview = preview;
        self
    }
}

fn structured_explanation(fixture: StructuredExplanationFixture) -> ExplanationHandle {
    ExplanationHandle::new(ExplanationHandleInput {
        id: ExplanationHandleId::new(fixture.identity).expect("valid explanation identity"),
        kind: fixture.kind,
        subject: fixture.subject,
        source: fixture.source,
        required_snapshot: fixture.required_snapshot,
        required_artifact_hash: fixture.required_artifact_hash,
        summary_hash: fixture.summary_hash,
        preview: Some(ExplanationPreview::new(
            ExplanationPreviewFormat::PlainText,
            fixture.preview,
        )),
    })
    .expect("valid structured explanation handle")
}

fn snapshot_id(byte: u8) -> BuildSnapshotId {
    let hex = format!("{byte:02x}");
    BuildSnapshotId::from_published_schema_str(&format!(
        "mizar-session-build-snapshot-v1:{}",
        hex.repeat(32)
    ))
    .expect("test snapshot id is valid")
}

fn source_id(snapshot: BuildSnapshotId) -> SourceId {
    InMemorySessionIdAllocator::new()
        .next_source_id(snapshot)
        .expect("source id allocation succeeds")
}

fn source_pair(snapshot: BuildSnapshotId) -> (SourceId, SourceId) {
    let allocator = InMemorySessionIdAllocator::new();
    (
        allocator
            .next_source_id(snapshot)
            .expect("first source id allocation succeeds"),
        allocator
            .next_source_id(snapshot)
            .expect("second source id allocation succeeds"),
    )
}
