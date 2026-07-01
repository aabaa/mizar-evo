use std::str::FromStr;

use mizar_diagnostics::{
    aggregator::{BuildDiagnosticIndex, DiagnosticAggregationInput},
    failure_record::{
        DiagnosticDetailValue, DiagnosticDetails, DiagnosticDraft, DiagnosticDraftInput,
        DiagnosticFreshness, DiagnosticSpan, ExplanationRef, FailureCategory, FixSuggestionRef,
        PipelinePhase,
    },
    registry::DiagnosticCode,
    sink::{DiagnosticBatch, DiagnosticProducerScope, DiagnosticSink},
};
use mizar_session::{
    BuildSnapshotId, InMemorySessionIdAllocator, SessionIdAllocator, SourceId, SourceRange,
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
            .map(|record| record.fixes()[0].identity())
            .collect::<Vec<_>>(),
        vec!["agg.fix_a", "agg.fix_b"]
    );
    assert_eq!(
        index
            .records()
            .iter()
            .filter_map(|record| record.explanation())
            .map(|explanation| explanation.identity())
            .collect::<Vec<_>>(),
        vec!["agg.exp_a", "agg.exp_b"]
    );
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
    fixes: Vec<&'static str>,
    explanation: Option<&'static str>,
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
        self.fixes.push(identity);
        self
    }

    fn explanation(mut self, identity: &'static str) -> Self {
        self.explanation = Some(identity);
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
        fixes: fixture
            .fixes
            .into_iter()
            .map(|identity| FixSuggestionRef::new(identity).expect("valid fix"))
            .collect(),
        explanation: fixture
            .explanation
            .map(|identity| ExplanationRef::new(identity).expect("valid explanation")),
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
