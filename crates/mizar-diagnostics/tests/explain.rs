use std::str::FromStr;

use mizar_diagnostics::{
    explain::{
        ExplanationError, ExplanationHandle, ExplanationHandleId, ExplanationHandleInput,
        ExplanationKind, ExplanationMissingReason, ExplanationPayload, ExplanationPreview,
        ExplanationPreviewFormat, ExplanationResolution, ExplanationSourceRef, ExplanationStore,
        ExplanationSubject,
    },
    failure_record::PipelinePhase,
    registry::DiagnosticCode,
};
use mizar_session::{
    BuildSnapshotId, Hash, InMemorySessionIdAllocator, SessionIdAllocator, SourceId, SourceRange,
};

#[test]
fn explanation_handle_round_trip_and_debug_snapshot_are_deterministic() {
    let snapshot = snapshot_id(1);
    let summary_hash = hash(9);
    let handle = ExplanationHandle::new(ExplanationHandleInput {
        id: ExplanationHandleId::new("explain.type_context").expect("valid handle id"),
        kind: ExplanationKind::TypeInference,
        subject: ExplanationSubject::Diagnostic {
            code: DiagnosticCode::from_str("E0001").expect("allocated code"),
            stable_detail_key: "syntax.unexpected_token".to_owned(),
        },
        source: ExplanationSourceRef::QueryService {
            service_key: "type.explain".to_owned(),
            query_key: "expr.main".to_owned(),
        },
        required_snapshot: Some(snapshot),
        required_artifact_hash: Some(hash(8)),
        summary_hash: Some(summary_hash),
        preview: Some(ExplanationPreview::new(
            ExplanationPreviewFormat::PlainText,
            "inferred type context",
        )),
    })
    .expect("valid explanation handle");

    assert_eq!(handle.id().identity(), "explain.type_context");
    assert_eq!(handle.kind(), ExplanationKind::TypeInference);
    assert_eq!(handle.required_snapshot(), Some(snapshot));
    assert_eq!(handle.required_artifact_hash(), Some(hash(8)));
    assert_eq!(handle.summary_hash(), Some(summary_hash));
    assert_eq!(
        handle.preview().map(ExplanationPreview::text),
        Some("inferred type context")
    );
    assert_eq!(
        handle.debug_snapshot(),
        concat!(
            "kind=explanation\n",
            "id=\"explain.type_context\"\n",
            "diagnostic=unpublished\n",
            "explanation_kind=type_inference\n",
            "subject=diagnostic(code=E0001,stable_detail_key=\"syntax.unexpected_token\")\n",
            "source=query_service(service_key=\"type.explain\",query_key=\"expr.main\")\n",
            "required_snapshot=mizar-session-build-snapshot-v1:",
            "0101010101010101010101010101010101010101010101010101010101010101\n",
            "required_artifact_hash=0808080808080808080808080808080808080808080808080808080808080808\n",
            "summary_hash=0909090909090909090909090909090909090909090909090909090909090909\n",
            "preview={format=plain_text,text=\"inferred type context\",truncated=false,byte_len=21,line_count=1}\n",
        )
    );
}

#[test]
fn previews_are_bounded_by_bytes_and_lines() {
    let byte_bounded =
        ExplanationPreview::with_bounds(ExplanationPreviewFormat::PlainText, "αβγδεζηθι", 16, 10)
            .expect("valid bounds");
    assert_eq!(byte_bounded.text(), "αβ [truncated]");
    assert!(byte_bounded.truncated());
    assert_eq!(byte_bounded.byte_len(), 16);
    assert_eq!(byte_bounded.line_count(), 1);

    let line_bounded = ExplanationPreview::with_bounds(
        ExplanationPreviewFormat::Markdown,
        "one\ntwo\nthree",
        64,
        2,
    )
    .expect("valid line bounds");
    assert_eq!(line_bounded.text(), "one\ntwo [truncated]");
    assert!(line_bounded.truncated());
    assert_eq!(line_bounded.line_count(), 2);

    assert!(matches!(
        ExplanationPreview::with_bounds(ExplanationPreviewFormat::PlainText, "text", 4, 1),
        Err(ExplanationError::InvalidPreviewByteBound { max_bytes: 4 })
    ));
    assert!(matches!(
        ExplanationPreview::with_bounds(ExplanationPreviewFormat::PlainText, "text", 64, 0),
        Err(ExplanationError::InvalidPreviewLineBound)
    ));
}

#[test]
fn store_resolves_preview_payloads_and_degrades_missing_stale_or_mismatched_data() {
    let snapshot = snapshot_id(2);
    let current = snapshot_id(3);
    let preview_handle = ExplanationHandle::preview_only(
        ExplanationHandleId::new("explain.preview").expect("valid handle id"),
        DiagnosticCode::from_str("E0001").expect("allocated code"),
        "syntax.unexpected_token",
        Some(ExplanationPreview::new(
            ExplanationPreviewFormat::PlainText,
            "preview only",
        )),
    )
    .expect("valid preview-only handle");
    let store = ExplanationStore::new();
    assert!(store.is_empty());
    assert_eq!(
        store.resolve(&preview_handle, None),
        ExplanationResolution::Available(ExplanationPayload::from_preview(
            ExplanationPreview::new(ExplanationPreviewFormat::PlainText, "preview only"),
            None,
        ))
    );

    let missing_preview = ExplanationHandle::preview_only(
        ExplanationHandleId::new("explain.missing_preview").expect("valid handle id"),
        DiagnosticCode::from_str("E0001").expect("allocated code"),
        "syntax.unexpected_token",
        None,
    )
    .expect("valid preview-only handle");
    assert_eq!(
        store.resolve(&missing_preview, None),
        ExplanationResolution::Missing {
            reason: ExplanationMissingReason::PreviewUnavailable,
        }
    );

    let artifact = artifact_handle(snapshot, Some(hash(4)));
    assert_eq!(
        store.resolve(&artifact, Some(snapshot)),
        ExplanationResolution::Missing {
            reason: ExplanationMissingReason::BackingDataMissing,
        }
    );
    assert_eq!(
        store.resolve(&artifact, Some(current)),
        ExplanationResolution::Stale {
            source_snapshot: snapshot,
            current_snapshot: current,
        }
    );

    let mut store = ExplanationStore::new();
    store.insert_payload(
        &artifact,
        ExplanationPayload::with_bounds(
            ExplanationPreviewFormat::StructuredText,
            "artifact payload",
            Some(hash(4)),
            64,
            4,
        )
        .expect("valid payload"),
    );
    assert_eq!(store.len(), 1);
    assert!(matches!(
        store.resolve(&artifact, Some(snapshot)),
        ExplanationResolution::Available(payload)
            if payload.text() == "artifact payload"
                && payload.summary_hash() == Some(hash(4))
    ));

    store.insert_payload(
        &artifact,
        ExplanationPayload::with_bounds(
            ExplanationPreviewFormat::StructuredText,
            "wrong payload",
            Some(hash(5)),
            64,
            4,
        )
        .expect("valid payload"),
    );
    assert_eq!(
        store.resolve(&artifact, Some(snapshot)),
        ExplanationResolution::Unavailable {
            reason: "summary_hash_mismatch".to_owned(),
        }
    );
}

#[test]
fn store_resolves_backing_source_variants_by_canonical_handle_key() {
    let snapshot = snapshot_id(5);
    let artifact = artifact_handle_with_content(snapshot, hash(6), Some(hash(4)));
    let artifact_other_content = artifact_handle_with_content(snapshot, hash(7), Some(hash(4)));
    let mut store = ExplanationStore::new();
    store.insert_payload(
        &artifact,
        ExplanationPayload::with_bounds(
            ExplanationPreviewFormat::StructuredText,
            "artifact payload",
            Some(hash(4)),
            64,
            4,
        )
        .expect("valid artifact payload"),
    );
    assert_eq!(
        store.resolve(&artifact_other_content, Some(snapshot)),
        ExplanationResolution::Missing {
            reason: ExplanationMissingReason::BackingDataMissing,
        }
    );

    let cache = cache_handle(Some(hash(8)));
    assert_eq!(
        store.resolve(&cache, None),
        ExplanationResolution::Missing {
            reason: ExplanationMissingReason::BackingDataMissing,
        }
    );
    store.insert_payload(
        &cache,
        ExplanationPayload::with_bounds(
            ExplanationPreviewFormat::PlainText,
            "cache payload",
            Some(hash(8)),
            64,
            4,
        )
        .expect("valid cache payload"),
    );
    assert!(matches!(
        store.resolve(&cache, None),
        ExplanationResolution::Available(payload)
            if payload.text() == "cache payload"
                && payload.summary_hash() == Some(hash(8))
    ));

    let query = query_handle();
    store.insert_payload(
        &query,
        ExplanationPayload::with_bounds(
            ExplanationPreviewFormat::Markdown,
            "query payload",
            None,
            64,
            4,
        )
        .expect("valid query payload"),
    );
    assert!(matches!(
        store.resolve(&query, None),
        ExplanationResolution::Available(payload)
            if payload.format() == ExplanationPreviewFormat::Markdown
                && payload.text() == "query payload"
                && payload.summary_hash().is_none()
    ));
}

#[test]
fn handle_validation_rejects_invalid_subjects_sources_and_ranges() {
    assert!(matches!(
        ExplanationHandleId::new("bad id"),
        Err(ExplanationError::InvalidHandleId { identity }) if identity == "bad id"
    ));
    assert!(matches!(
        ExplanationHandle::new(ExplanationHandleInput {
            id: ExplanationHandleId::new("explain.bad_subject").expect("valid handle id"),
            kind: ExplanationKind::DiagnosticContext,
            subject: ExplanationSubject::Expression("bad key".to_owned()),
            source: ExplanationSourceRef::PreviewOnly,
            required_snapshot: None,
            required_artifact_hash: None,
            summary_hash: None,
            preview: None,
        }),
        Err(ExplanationError::InvalidSubjectKey { key }) if key == "bad key"
    ));

    let snapshot = snapshot_id(4);
    let source_id = source_id(snapshot);
    assert!(matches!(
        ExplanationHandle::new(ExplanationHandleInput {
            id: ExplanationHandleId::new("explain.bad_range").expect("valid handle id"),
            kind: ExplanationKind::DiagnosticContext,
            subject: ExplanationSubject::SourceRange(SourceRange {
                source_id,
                start: 9,
                end: 2,
            }),
            source: ExplanationSourceRef::PreviewOnly,
            required_snapshot: None,
            required_artifact_hash: None,
            summary_hash: None,
            preview: None,
        }),
        Err(ExplanationError::InvalidRange { start: 9, end: 2 })
    ));
    assert!(matches!(
        ExplanationHandle::new(ExplanationHandleInput {
            id: ExplanationHandleId::new("explain.empty_artifact").expect("valid handle id"),
            kind: ExplanationKind::DiagnosticContext,
            subject: ExplanationSubject::PhaseLocal {
                phase: PipelinePhase::Parser,
                key: "parse.explain".to_owned(),
            },
            source: ExplanationSourceRef::Artifact {
                path: String::new(),
                content_hash: hash(1),
            },
            required_snapshot: None,
            required_artifact_hash: None,
            summary_hash: None,
            preview: None,
        }),
        Err(ExplanationError::EmptyArtifactPath)
    ));
    assert!(matches!(
        ExplanationHandle::new(ExplanationHandleInput {
            id: ExplanationHandleId::new("explain.bad_query").expect("valid handle id"),
            kind: ExplanationKind::DiagnosticContext,
            subject: ExplanationSubject::PhaseLocal {
                phase: PipelinePhase::Parser,
                key: "parse.explain".to_owned(),
            },
            source: ExplanationSourceRef::QueryService {
                service_key: "bad service".to_owned(),
                query_key: "parse.query".to_owned(),
            },
            required_snapshot: None,
            required_artifact_hash: None,
            summary_hash: None,
            preview: None,
        }),
        Err(ExplanationError::InvalidServiceKey { key }) if key == "bad service"
    ));
    assert!(matches!(
        ExplanationHandle::new(ExplanationHandleInput {
            id: ExplanationHandleId::new("explain.bad_cache").expect("valid handle id"),
            kind: ExplanationKind::DiagnosticContext,
            subject: ExplanationSubject::PhaseLocal {
                phase: PipelinePhase::Parser,
                key: "parse.explain".to_owned(),
            },
            source: ExplanationSourceRef::CacheRecord {
                cache_key: "bad cache".to_owned(),
                content_hash: None,
            },
            required_snapshot: None,
            required_artifact_hash: None,
            summary_hash: None,
            preview: None,
        }),
        Err(ExplanationError::InvalidCacheKey { key }) if key == "bad cache"
    ));
    assert!(matches!(
        ExplanationHandle::new(ExplanationHandleInput {
            id: ExplanationHandleId::new("explain.bad_query_key").expect("valid handle id"),
            kind: ExplanationKind::DiagnosticContext,
            subject: ExplanationSubject::PhaseLocal {
                phase: PipelinePhase::Parser,
                key: "parse.explain".to_owned(),
            },
            source: ExplanationSourceRef::QueryService {
                service_key: "parse.service".to_owned(),
                query_key: "bad query".to_owned(),
            },
            required_snapshot: None,
            required_artifact_hash: None,
            summary_hash: None,
            preview: None,
        }),
        Err(ExplanationError::InvalidQueryKey { key }) if key == "bad query"
    ));
}

fn artifact_handle(snapshot: BuildSnapshotId, summary_hash: Option<Hash>) -> ExplanationHandle {
    artifact_handle_with_content(snapshot, hash(6), summary_hash)
}

fn artifact_handle_with_content(
    snapshot: BuildSnapshotId,
    content_hash: Hash,
    summary_hash: Option<Hash>,
) -> ExplanationHandle {
    ExplanationHandle::new(ExplanationHandleInput {
        id: ExplanationHandleId::new("explain.artifact").expect("valid handle id"),
        kind: ExplanationKind::ProofFailure,
        subject: ExplanationSubject::VerificationCondition("vc.goal".to_owned()),
        source: ExplanationSourceRef::Artifact {
            path: "explain/proof.json".to_owned(),
            content_hash,
        },
        required_snapshot: Some(snapshot),
        required_artifact_hash: Some(content_hash),
        summary_hash,
        preview: None,
    })
    .expect("valid artifact handle")
}

fn cache_handle(summary_hash: Option<Hash>) -> ExplanationHandle {
    ExplanationHandle::new(ExplanationHandleInput {
        id: ExplanationHandleId::new("explain.cache").expect("valid handle id"),
        kind: ExplanationKind::ClusterResolution,
        subject: ExplanationSubject::Expression("expr.cluster".to_owned()),
        source: ExplanationSourceRef::CacheRecord {
            cache_key: "cluster.lookup".to_owned(),
            content_hash: Some(hash(7)),
        },
        required_snapshot: None,
        required_artifact_hash: None,
        summary_hash,
        preview: None,
    })
    .expect("valid cache handle")
}

fn query_handle() -> ExplanationHandle {
    ExplanationHandle::new(ExplanationHandleInput {
        id: ExplanationHandleId::new("explain.query").expect("valid handle id"),
        kind: ExplanationKind::OverloadResolution,
        subject: ExplanationSubject::PhaseLocal {
            phase: PipelinePhase::Resolver,
            key: "resolve.overload".to_owned(),
        },
        source: ExplanationSourceRef::QueryService {
            service_key: "resolve.explain".to_owned(),
            query_key: "resolve.candidates".to_owned(),
        },
        required_snapshot: None,
        required_artifact_hash: None,
        summary_hash: None,
        preview: None,
    })
    .expect("valid query handle")
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

fn hash(byte: u8) -> Hash {
    Hash::from_bytes([byte; Hash::BYTE_LEN])
}
