use std::str::FromStr;

use mizar_diagnostics::{
    failure_record::{
        DiagnosticDetailValue, DiagnosticDetails, DiagnosticDraft, DiagnosticDraftInput,
        DiagnosticFreshness, DiagnosticHandle, DiagnosticId, DiagnosticNote, DiagnosticNoteKind,
        DiagnosticRecord, DiagnosticRecordError, DiagnosticSpan, DiagnosticSpanRole,
        FailureCategory, PipelinePhase, SpanFreshness, StaleDiagnosticReason, ZeroWidthSpanIntent,
        is_valid_detail_key,
    },
    fix::{FixSuggestion, FixSuggestionError, FixSuggestionId},
    registry::{
        BUILTIN_DESCRIPTORS, DiagnosticCode, DiagnosticDescriptor, DiagnosticRegistry,
        DiagnosticStatus,
    },
};
use mizar_session::{
    BuildSnapshotId, InMemorySessionIdAllocator, SessionIdAllocator, SourceId, SourceRange,
};

#[test]
fn draft_and_record_round_trip_through_descriptor_metadata() {
    let snapshot = snapshot_id(1);
    let source_id = source_id(snapshot);
    let code = DiagnosticCode::from_str("E0001").expect("allocated code");
    let descriptor = DiagnosticRegistry::builtin()
        .lookup(code)
        .expect("descriptor exists");
    let primary_span = DiagnosticSpan::primary(
        SourceRange {
            source_id,
            start: 4,
            end: 9,
        },
        Some("unexpected token".to_owned()),
    )
    .expect("valid primary span");
    let secondary_span = DiagnosticSpan::secondary(
        SourceRange {
            source_id,
            start: 0,
            end: 3,
        },
        Some("open block".to_owned()),
    )
    .expect("valid secondary span");
    let note = DiagnosticNote::new(
        DiagnosticNoteKind::Help,
        "insert a matching keyword",
        Some(secondary_span.clone()),
    );
    let details = DiagnosticDetails::from_entries([
        ("parse.expected_count", DiagnosticDetailValue::Integer(2)),
        ("parse.recovered", DiagnosticDetailValue::Boolean(true)),
    ])
    .expect("valid details");
    let fix = informational_fix("syntax.insert_end", "insert a matching keyword");
    let explanation =
        mizar_diagnostics::failure_record::ExplanationRef::new("syntax.unexpected_token.context")
            .expect("valid explanation ref");

    let draft = DiagnosticDraft::new(DiagnosticDraftInput {
        source_snapshot: snapshot,
        code,
        phase: PipelinePhase::Parser,
        category: FailureCategory::ParseError,
        stable_detail_key: "syntax.unexpected_token".to_owned(),
        message: "unexpected token".to_owned(),
        primary_span: primary_span.clone(),
        secondary_spans: vec![secondary_span.clone()],
        notes: vec![note.clone()],
        details: details.clone(),
        fixes: vec![fix.clone()],
        explanation: Some(explanation.clone()),
    })
    .expect("valid draft");

    let handle = DiagnosticHandle::new(snapshot, DiagnosticId::new(7));
    let related = vec![DiagnosticHandle::new(snapshot, DiagnosticId::new(8))];
    let record = DiagnosticRecord::from_draft(
        draft.clone(),
        handle,
        DiagnosticFreshness::Current {
            source_snapshot: snapshot,
        },
        related.clone(),
    )
    .expect("valid record");

    assert_eq!(record.handle(), handle);
    assert_eq!(record.code(), draft.code());
    assert_eq!(record.semantic_name(), descriptor.semantic_name);
    assert_eq!(record.severity(), descriptor.default_severity);
    assert_eq!(record.phase(), draft.phase());
    assert_eq!(record.category(), draft.category());
    assert_eq!(record.stable_detail_key(), draft.stable_detail_key());
    assert_eq!(record.message(), draft.message());
    assert_eq!(record.primary_span(), draft.primary_span());
    assert_eq!(record.secondary_spans(), draft.secondary_spans());
    assert_eq!(record.notes(), draft.notes());
    assert_eq!(record.details(), draft.details());
    assert_eq!(record.fixes(), draft.fixes());
    assert_eq!(record.explanation(), draft.explanation());
    assert_eq!(record.related(), related);
    assert_eq!(record.freshness().source_snapshot(), snapshot);
    assert!(record.freshness().is_current());
}

#[test]
fn drafts_and_records_require_registry_allocated_codes() {
    let snapshot = snapshot_id(9);
    let source_id = source_id(snapshot);
    let mut input = draft_input(
        snapshot,
        source_id,
        DiagnosticSpan::primary(
            SourceRange {
                source_id,
                start: 0,
                end: 1,
            },
            None,
        )
        .expect("valid primary span"),
        vec![],
    );
    input.code = DiagnosticCode::from_str("E0600").expect("well-formed unallocated code");
    assert!(matches!(
        DiagnosticDraft::new(input),
        Err(DiagnosticRecordError::UnknownDiagnosticCode { code })
            if code == DiagnosticCode::from_str("E0600").expect("well-formed unallocated code")
    ));

    let code = DiagnosticCode::from_str("E0001").expect("allocated code");
    let mut retired_descriptors = BUILTIN_DESCRIPTORS.to_vec();
    retire_descriptor(&mut retired_descriptors, code);
    let retired_registry =
        DiagnosticRegistry::new(&retired_descriptors).expect("retired registry is valid");
    let input = draft_input(
        snapshot,
        source_id,
        DiagnosticSpan::primary(
            SourceRange {
                source_id,
                start: 0,
                end: 1,
            },
            None,
        )
        .expect("valid primary span"),
        vec![],
    );
    assert!(matches!(
        DiagnosticDraft::new_with_registry(input, retired_registry),
        Err(DiagnosticRecordError::RetiredDescriptorForDraft { code: retired_code })
            if retired_code == code
    ));
}

#[test]
fn span_constructors_validate_ranges_roles_and_zero_width_intent() {
    let snapshot = snapshot_id(2);
    let source_id = source_id(snapshot);

    assert!(matches!(
        DiagnosticSpan::primary(
            SourceRange {
                source_id,
                start: 10,
                end: 2,
            },
            None,
        ),
        Err(DiagnosticRecordError::InvalidRange { start: 10, end: 2 })
    ));
    assert!(matches!(
        DiagnosticSpan::new(
            SourceRange {
                source_id,
                start: 3,
                end: 3,
            },
            DiagnosticSpanRole::Primary,
            None,
            SpanFreshness::Current,
            None,
        ),
        Err(DiagnosticRecordError::ZeroWidthIntentRequired { offset: 3 })
    ));
    assert!(matches!(
        DiagnosticSpan::new(
            SourceRange {
                source_id,
                start: 3,
                end: 4,
            },
            DiagnosticSpanRole::Primary,
            None,
            SpanFreshness::Current,
            Some(ZeroWidthSpanIntent::Eof),
        ),
        Err(DiagnosticRecordError::ZeroWidthIntentOnNonZeroRange { .. })
    ));

    let secondary_as_primary = DiagnosticSpan::secondary(
        SourceRange {
            source_id,
            start: 1,
            end: 2,
        },
        None,
    )
    .expect("valid secondary span");
    assert!(matches!(
        DiagnosticDraft::new(draft_input(
            snapshot,
            source_id,
            secondary_as_primary,
            vec![]
        )),
        Err(DiagnosticRecordError::PrimarySpanMustUsePrimaryRole {
            actual: DiagnosticSpanRole::Secondary
        })
    ));

    let primary = DiagnosticSpan::primary(
        SourceRange {
            source_id,
            start: 1,
            end: 2,
        },
        None,
    )
    .expect("valid primary span");
    assert!(matches!(
        DiagnosticDraft::new(draft_input(
            snapshot,
            source_id,
            primary.clone(),
            vec![primary],
        )),
        Err(DiagnosticRecordError::SecondarySpanMustNotUsePrimaryRole { index: 0 })
    ));

    let eof = DiagnosticSpan::new(
        SourceRange {
            source_id,
            start: 3,
            end: 3,
        },
        DiagnosticSpanRole::Primary,
        None,
        SpanFreshness::Current,
        Some(ZeroWidthSpanIntent::Eof),
    )
    .expect("valid EOF span");
    assert_eq!(eof.zero_width(), Some(ZeroWidthSpanIntent::Eof));
    let insertion = DiagnosticSpan::new(
        SourceRange {
            source_id,
            start: 4,
            end: 4,
        },
        DiagnosticSpanRole::Secondary,
        None,
        SpanFreshness::Current,
        Some(ZeroWidthSpanIntent::InsertionPoint),
    )
    .expect("valid insertion-point span");
    assert_eq!(
        insertion.zero_width(),
        Some(ZeroWidthSpanIntent::InsertionPoint)
    );
    let definition = DiagnosticSpan::new(
        SourceRange {
            source_id,
            start: 5,
            end: 6,
        },
        DiagnosticSpanRole::DefinitionSite,
        None,
        SpanFreshness::Current,
        None,
    )
    .expect("valid definition-site span");
    let related = DiagnosticSpan::new(
        SourceRange {
            source_id,
            start: 6,
            end: 7,
        },
        DiagnosticSpanRole::Related,
        None,
        SpanFreshness::Current,
        None,
    )
    .expect("valid related span");
    DiagnosticDraft::new(draft_input(
        snapshot,
        source_id,
        DiagnosticSpan::primary(
            SourceRange {
                source_id,
                start: 1,
                end: 2,
            },
            None,
        )
        .expect("valid primary span"),
        vec![insertion, definition, related],
    ))
    .expect("non-primary secondary roles are valid");
}

#[test]
fn details_validate_key_grammar_and_preserve_sorted_entries() {
    for valid in [
        "proof.rejection_reason",
        "declaration_symbol.symbol.duplicate_declaration",
        "resolve.candidate_count",
        "a0.b1_c2",
    ] {
        assert!(is_valid_detail_key(valid), "{valid}");
    }
    for invalid in [
        "",
        ".leading",
        "trailing.",
        "empty..segment",
        "Bad.case",
        "bad__underscore",
        "bad_",
        "bad-key",
        "bad key",
        "bad.日本語",
    ] {
        assert!(!is_valid_detail_key(invalid), "{invalid}");
    }

    let details = DiagnosticDetails::from_entries([
        (
            "zeta.value",
            DiagnosticDetailValue::String("last".to_owned()),
        ),
        (
            "alpha.value",
            DiagnosticDetailValue::String("first".to_owned()),
        ),
    ])
    .expect("valid details");
    assert_eq!(
        details
            .entries()
            .keys()
            .map(String::as_str)
            .collect::<Vec<_>>(),
        vec!["alpha.value", "zeta.value"]
    );
    assert!(matches!(
        DiagnosticDetails::from_entries([(
            "bad-key",
            DiagnosticDetailValue::String("value".to_owned()),
        )]),
        Err(DiagnosticRecordError::InvalidDetailKey { key }) if key == "bad-key"
    ));

    let snapshot = snapshot_id(3);
    let source_id = source_id(snapshot);
    let mut input = draft_input(
        snapshot,
        source_id,
        DiagnosticSpan::primary(
            SourceRange {
                source_id,
                start: 1,
                end: 2,
            },
            None,
        )
        .expect("valid primary span"),
        vec![],
    );
    input.stable_detail_key = "bad-key".to_owned();
    assert!(matches!(
        DiagnosticDraft::new(input),
        Err(DiagnosticRecordError::InvalidStableDetailKey { key }) if key == "bad-key"
    ));
}

#[test]
fn detail_values_have_canonical_ordering() {
    let snapshot = snapshot_id(10);
    let source_id = source_id(snapshot);
    let code = DiagnosticCode::from_str("E0001").expect("allocated code");
    let source_a = DiagnosticDetailValue::Source(SourceRange {
        source_id,
        start: 1,
        end: 2,
    });
    let source_b = DiagnosticDetailValue::Source(SourceRange {
        source_id,
        start: 2,
        end: 3,
    });
    let mut values = vec![
        DiagnosticDetailValue::List(vec![DiagnosticDetailValue::Boolean(true)]),
        source_b.clone(),
        DiagnosticDetailValue::String("text".to_owned()),
        DiagnosticDetailValue::Boolean(false),
        DiagnosticDetailValue::Code(code),
        source_a.clone(),
        DiagnosticDetailValue::Integer(-1),
    ];

    values.sort();

    assert_eq!(
        values,
        vec![
            DiagnosticDetailValue::Boolean(false),
            DiagnosticDetailValue::Integer(-1),
            DiagnosticDetailValue::String("text".to_owned()),
            DiagnosticDetailValue::Code(code),
            source_a,
            source_b,
            DiagnosticDetailValue::List(vec![DiagnosticDetailValue::Boolean(true)]),
        ]
    );
}

#[test]
fn invalid_fix_and_explanation_identities_are_rejected() {
    assert!(matches!(
        FixSuggestionId::new("bad-key"),
        Err(FixSuggestionError::InvalidFixIdentity { identity }) if identity == "bad-key"
    ));
    assert!(matches!(
        mizar_diagnostics::failure_record::ExplanationRef::new("bad key"),
        Err(DiagnosticRecordError::InvalidExplanationIdentity { identity }) if identity == "bad key"
    ));
}

#[test]
fn debug_rendering_is_byte_stable_and_sorts_details() {
    let snapshot = snapshot_id(4);
    let source_id = source_id(snapshot);
    let primary = DiagnosticSpan::primary(
        SourceRange {
            source_id,
            start: 0,
            end: 3,
        },
        Some("token".to_owned()),
    )
    .expect("valid primary span");
    let details = DiagnosticDetails::from_entries([
        ("parse.recovered", DiagnosticDetailValue::Boolean(true)),
        ("parse.expected_count", DiagnosticDetailValue::Integer(2)),
    ])
    .expect("valid details");
    let draft = DiagnosticDraft::new(DiagnosticDraftInput {
        source_snapshot: snapshot,
        code: DiagnosticCode::from_str("E0001").expect("allocated code"),
        phase: PipelinePhase::Parser,
        category: FailureCategory::ParseError,
        stable_detail_key: "syntax.unexpected_token".to_owned(),
        message: "unexpected token".to_owned(),
        primary_span: primary,
        secondary_spans: vec![],
        notes: vec![],
        details,
        fixes: vec![informational_fix(
            "syntax.insert_end",
            "insert a matching keyword",
        )],
        explanation: Some(
            mizar_diagnostics::failure_record::ExplanationRef::new(
                "syntax.unexpected_token.context",
            )
            .expect("valid explanation ref"),
        ),
    })
    .expect("valid draft");
    let record = DiagnosticRecord::from_draft(
        draft.clone(),
        DiagnosticHandle::new(snapshot, DiagnosticId::new(3)),
        DiagnosticFreshness::Current {
            source_snapshot: snapshot,
        },
        vec![DiagnosticHandle::new(snapshot, DiagnosticId::new(4))],
    )
    .expect("valid record");

    assert_eq!(
        draft.debug_snapshot(),
        concat!(
            "kind=draft\n",
            "handle=none\n",
            "code=E0001\n",
            "semantic_name=none\n",
            "severity=none\n",
            "phase=parser\n",
            "category=parse_error\n",
            "stable_detail_key=\"syntax.unexpected_token\"\n",
            "message=\"unexpected token\"\n",
            "source_snapshot=mizar-session-build-snapshot-v1:",
            "0404040404040404040404040404040404040404040404040404040404040404\n",
            "freshness=draft\n",
            "primary=SourceId(OpaqueId(1)):0..3:primary:current:none:\"token\"\n",
            "secondary=[]\n",
            "notes=[]\n",
            "details={parse.expected_count=int:2, parse.recovered=bool:true}\n",
            "fixes=[kind=fix\\nid=\"syntax.insert_end\"\\nproducer_key=none\\n",
            "diagnostic=unpublished\\ntitle=\"insert a matching keyword\"\\n",
            "applicability=informational\\nsafety=command_only\\nedits=[]\\n",
            "command=none\\nrequired_snapshot=none\\nrequired_text_hash=none]\n",
            "explanation=\"syntax.unexpected_token.context\"\n",
            "related=[]\n",
        )
    );
    assert_eq!(
        record.debug_snapshot(),
        concat!(
            "kind=record\n",
            "handle=mizar-session-build-snapshot-v1:",
            "0404040404040404040404040404040404040404040404040404040404040404#3\n",
            "code=E0001\n",
            "semantic_name=\"syntax.unexpected_token\"\n",
            "severity=error\n",
            "phase=parser\n",
            "category=parse_error\n",
            "stable_detail_key=\"syntax.unexpected_token\"\n",
            "message=\"unexpected token\"\n",
            "source_snapshot=mizar-session-build-snapshot-v1:",
            "0404040404040404040404040404040404040404040404040404040404040404\n",
            "freshness=current(source_snapshot=mizar-session-build-snapshot-v1:",
            "0404040404040404040404040404040404040404040404040404040404040404)\n",
            "primary=SourceId(OpaqueId(1)):0..3:primary:current:none:\"token\"\n",
            "secondary=[]\n",
            "notes=[]\n",
            "details={parse.expected_count=int:2, parse.recovered=bool:true}\n",
            "fixes=[kind=fix\\nid=\"syntax.insert_end\"\\nproducer_key=none\\n",
            "diagnostic=mizar-session-build-snapshot-v1:",
            "0404040404040404040404040404040404040404040404040404040404040404#3\\n",
            "title=\"insert a matching keyword\"\\napplicability=informational\\n",
            "safety=command_only\\nedits=[]\\ncommand=none\\n",
            "required_snapshot=none\\nrequired_text_hash=none]\n",
            "explanation=\"syntax.unexpected_token.context\"\n",
            "related=[mizar-session-build-snapshot-v1:",
            "0404040404040404040404040404040404040404040404040404040404040404#4]\n",
        )
    );
}

#[test]
fn freshness_validation_keeps_obsolete_snapshots_out_of_current_records() {
    let old_snapshot = snapshot_id(5);
    let current_snapshot = snapshot_id(6);
    let old_source = source_id(old_snapshot);
    let draft = DiagnosticDraft::new(draft_input(
        old_snapshot,
        old_source,
        DiagnosticSpan::primary(
            SourceRange {
                source_id: old_source,
                start: 0,
                end: 1,
            },
            None,
        )
        .expect("valid primary span"),
        vec![],
    ))
    .expect("valid draft");
    let current_handle = DiagnosticHandle::new(current_snapshot, DiagnosticId::new(1));

    assert!(matches!(
        DiagnosticRecord::from_draft(
            draft.clone(),
            current_handle,
            DiagnosticFreshness::Current {
                source_snapshot: current_snapshot,
            },
            vec![],
        ),
        Err(DiagnosticRecordError::DraftFreshnessSnapshotMismatch { .. })
    ));
    assert!(matches!(
        DiagnosticRecord::from_draft(
            draft.clone(),
            current_handle,
            DiagnosticFreshness::Current {
                source_snapshot: old_snapshot,
            },
            vec![],
        ),
        Err(DiagnosticRecordError::CurrentFreshnessSnapshotMismatch { .. })
    ));
    assert!(matches!(
        DiagnosticRecord::from_draft(
            draft.clone(),
            DiagnosticHandle::new(old_snapshot, DiagnosticId::new(2)),
            DiagnosticFreshness::Stale {
                source_snapshot: old_snapshot,
                current_snapshot: old_snapshot,
                reason: StaleDiagnosticReason::SnapshotSuperseded,
            },
            vec![],
        ),
        Err(DiagnosticRecordError::StaleFreshnessNotStale { .. })
    ));
    assert!(matches!(
        DiagnosticRecord::from_draft(
            draft.clone(),
            current_handle,
            DiagnosticFreshness::Historical {
                source_snapshot: old_snapshot,
                artifact_hash: Some("abc123".to_owned()),
            },
            vec![],
        ),
        Err(DiagnosticRecordError::HistoricalFreshnessSnapshotMismatch { .. })
    ));

    let stale = DiagnosticRecord::from_draft(
        draft.clone(),
        current_handle,
        DiagnosticFreshness::Stale {
            source_snapshot: old_snapshot,
            current_snapshot,
            reason: StaleDiagnosticReason::SnapshotSuperseded,
        },
        vec![],
    )
    .expect("stale record stays explicit");
    assert!(!stale.freshness().is_current());

    let historical = DiagnosticRecord::from_draft(
        draft,
        DiagnosticHandle::new(old_snapshot, DiagnosticId::new(3)),
        DiagnosticFreshness::Historical {
            source_snapshot: old_snapshot,
            artifact_hash: Some("abc123".to_owned()),
        },
        vec![],
    )
    .expect("historical record stays explicit");
    assert!(!historical.freshness().is_current());
}

#[test]
fn message_text_changes_do_not_change_code_or_detail_identity() {
    let snapshot = snapshot_id(11);
    let source_id = source_id(snapshot);
    let primary = DiagnosticSpan::primary(
        SourceRange {
            source_id,
            start: 0,
            end: 1,
        },
        None,
    )
    .expect("valid primary span");
    let mut first = draft_input(snapshot, source_id, primary.clone(), vec![]);
    first.message = "old wording".to_owned();
    let mut second = draft_input(snapshot, source_id, primary, vec![]);
    second.message = "new wording".to_owned();

    let first = DiagnosticRecord::from_draft(
        DiagnosticDraft::new(first).expect("valid draft"),
        DiagnosticHandle::new(snapshot, DiagnosticId::new(1)),
        DiagnosticFreshness::Current {
            source_snapshot: snapshot,
        },
        vec![],
    )
    .expect("valid first record");
    let second = DiagnosticRecord::from_draft(
        DiagnosticDraft::new(second).expect("valid draft"),
        DiagnosticHandle::new(snapshot, DiagnosticId::new(2)),
        DiagnosticFreshness::Current {
            source_snapshot: snapshot,
        },
        vec![],
    )
    .expect("valid second record");

    assert_eq!(first.code(), second.code());
    assert_eq!(first.semantic_name(), second.semantic_name());
    assert_eq!(first.stable_detail_key(), second.stable_detail_key());
    assert_eq!(first.details(), second.details());
    assert_ne!(first.message(), second.message());
}

#[test]
fn current_records_reject_retired_descriptor_and_related_cross_snapshot_handles() {
    let snapshot = snapshot_id(7);
    let source_id = source_id(snapshot);
    let code = DiagnosticCode::from_str("E0001").expect("allocated code");
    let mut retired_descriptors = BUILTIN_DESCRIPTORS.to_vec();
    retire_descriptor(&mut retired_descriptors, code);
    let draft = DiagnosticDraft::new(draft_input(
        snapshot,
        source_id,
        DiagnosticSpan::primary(
            SourceRange {
                source_id,
                start: 0,
                end: 1,
            },
            None,
        )
        .expect("valid primary span"),
        vec![],
    ))
    .expect("valid draft");
    let handle = DiagnosticHandle::new(snapshot, DiagnosticId::new(1));

    assert!(matches!(
        DiagnosticRecord::from_draft_with_registry(
            draft.clone(),
            DiagnosticRegistry::new(&retired_descriptors).expect("retired registry is valid"),
            handle,
            DiagnosticFreshness::Current {
                source_snapshot: snapshot,
            },
            vec![],
        ),
        Err(DiagnosticRecordError::RetiredDescriptorForCurrentRecord { code: retired_code })
            if retired_code == code
    ));
    assert!(matches!(
        DiagnosticRecord::from_draft(
            draft,
            handle,
            DiagnosticFreshness::Current {
                source_snapshot: snapshot,
            },
            vec![DiagnosticHandle::new(snapshot_id(8), DiagnosticId::new(2))],
        ),
        Err(DiagnosticRecordError::RelatedHandleSnapshotMismatch { index: 0, .. })
    ));
}

fn retire_descriptor(descriptors: &mut [DiagnosticDescriptor], code: DiagnosticCode) {
    let descriptor = descriptors
        .iter_mut()
        .find(|descriptor| descriptor.code == code)
        .expect("descriptor exists");
    *descriptor = DiagnosticDescriptor {
        status: DiagnosticStatus::Retired,
        retired_since: Some("test"),
        ..*descriptor
    };
}

fn draft_input(
    snapshot: BuildSnapshotId,
    source_id: SourceId,
    primary_span: DiagnosticSpan,
    secondary_spans: Vec<DiagnosticSpan>,
) -> DiagnosticDraftInput {
    DiagnosticDraftInput {
        source_snapshot: snapshot,
        code: DiagnosticCode::from_str("E0001").expect("allocated code"),
        phase: PipelinePhase::Parser,
        category: FailureCategory::ParseError,
        stable_detail_key: "syntax.unexpected_token".to_owned(),
        message: "unexpected token".to_owned(),
        primary_span,
        secondary_spans,
        notes: vec![],
        details: DiagnosticDetails::from_entries([(
            "source.id",
            DiagnosticDetailValue::Source(SourceRange {
                source_id,
                start: 0,
                end: 1,
            }),
        )])
        .expect("valid details"),
        fixes: vec![],
        explanation: None,
    }
}

fn informational_fix(identity: &str, title: &str) -> FixSuggestion {
    FixSuggestion::informational(
        FixSuggestionId::new(identity).expect("valid fix identity"),
        title,
    )
    .expect("valid informational fix")
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
