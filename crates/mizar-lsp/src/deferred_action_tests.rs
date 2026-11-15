use super::full_snapshot_registry_a::RegistryFixture;
use crate::diagnostics::{
    DeferredActionContext, DeferredArtifactDependency, DeferredFixRegistration,
    RegisteredDeferredFix, action_backing_hash, convert_code_actions_with_deferred,
    execute_deferred_command, validate_deferred_fix,
};
use mizar_diagnostics::{
    aggregator::{BuildDiagnosticIndex, DiagnosticAggregationInput},
    explain::{
        ExplanationHandle, ExplanationHandleId, ExplanationHandleInput, ExplanationKind,
        ExplanationPayload, ExplanationPreviewFormat, ExplanationSourceRef, ExplanationStore,
        ExplanationSubject,
    },
    failure_record::{
        DiagnosticDetails, DiagnosticDraft, DiagnosticDraftInput, DiagnosticPrimaryLocation,
        DiagnosticSpan, DiagnosticSpanRole, FailureCategory, PipelinePhase, SpanFreshness,
    },
    fix::{
        FixApplicability, FixCommandRef, FixEdit, FixSafety, FixSuggestion, FixSuggestionId,
        FixSuggestionInput,
    },
    registry::{DiagnosticCode, DiagnosticSeverity},
    sink::{DiagnosticProducerScope, DiagnosticSink},
};
use mizar_session::{Hash, SourceRange};
use serde_json::{Value, json};
use std::sync::Arc;

const TEXT_HASH_DOMAIN: &str = "mizar-session/source-text/v1";
const PUBLICATION: &str = "task38-test-publication";

#[derive(Default)]
struct TestProducer {
    accepted: Vec<(String, String, Vec<u8>)>,
    expected_payload: Option<String>,
    expected_handle: Option<String>,
}

impl crate::DeferredActionProducer for TestProducer {
    fn validates_encoding(
        &self,
        role: &str,
        encoding: &str,
        bytes: &[u8],
        payload: &ExplanationPayload,
        handle: &ExplanationHandle,
    ) -> bool {
        self.expected_payload
            .as_deref()
            .is_some_and(|expected| payload.text() == expected)
            && self
                .expected_handle
                .as_deref()
                .is_some_and(|expected| handle.id().identity() == expected)
            && self
                .accepted
                .iter()
                .any(|(expected_role, expected_encoding, expected_bytes)| {
                    expected_role == role
                        && expected_encoding == encoding
                        && expected_bytes == bytes
                })
    }

    fn validates_missing_facts(
        &self,
        _backing: &mizar_artifact::store::CanonicalJson,
        _dependency: &DeferredArtifactDependency,
        _diagnostic: &mizar_diagnostics::failure_record::DiagnosticRecord,
        _fix: &FixSuggestion,
        _obligation: &str,
        _entry: &str,
    ) -> bool {
        // The genuine fixture artifacts carry no registered missing-facts schema.
        false
    }
}

fn text_sources(fixture: &RegistryFixture) -> Vec<(String, mizar_session::SourceId, String)> {
    let (selected, _) = fixture
        .server
        .lsp_snapshot(&fixture.driver, &fixture.snapshots)
        .expect("published snapshot");
    selected
        .native_open_buffers()
        .captured()
        .native_sources()
        .unwrap()
        .iter()
        .map(|(_, loaded, input)| {
            let mizar_session::SourceOriginInput::OpenBuffer { uri, .. } = &input.origin else {
                panic!("expected open buffer")
            };
            (uri.clone(), loaded.source_id, loaded.text.to_string())
        })
        .collect()
}

fn edit_for(
    source_id: mizar_session::SourceId,
    text: &str,
    needle: &str,
    replacement: &str,
) -> FixEdit {
    let start = text.find(needle).expect("edit text present");
    FixEdit::new(
        SourceRange {
            source_id,
            start,
            end: start + needle.len(),
        },
        replacement,
        Some(needle.to_owned()),
    )
    .unwrap()
}

fn fix(
    id: &str,
    safety: FixSafety,
    edits: Vec<FixEdit>,
    required_text_hash: Option<Hash>,
    command: Option<FixCommandRef>,
) -> FixSuggestion {
    FixSuggestion::new(FixSuggestionInput {
        id: FixSuggestionId::new(id).unwrap(),
        producer_key: Some("task38_test_producer".to_owned()),
        title: format!("Apply {id}"),
        applicability: FixApplicability::MachineApplicable,
        safety,
        edits,
        command,
        required_snapshot: None,
        required_text_hash,
    })
    .unwrap()
}

fn explanation_handle(
    id: &str,
    stable_key: &str,
    source: ExplanationSourceRef,
    summary_hash: Option<Hash>,
    snapshot: Option<mizar_session::BuildSnapshotId>,
) -> ExplanationHandle {
    ExplanationHandle::new(ExplanationHandleInput {
        id: ExplanationHandleId::new(id).unwrap(),
        kind: ExplanationKind::DiagnosticContext,
        subject: ExplanationSubject::Diagnostic {
            code: DiagnosticCode::from_parts(DiagnosticSeverity::Error, 1).unwrap(),
            stable_detail_key: stable_key.replace('-', "_"),
        },
        source,
        required_snapshot: snapshot,
        required_artifact_hash: None,
        summary_hash,
        preview: None,
    })
    .unwrap()
}

fn index_with_fix(
    fixture: &RegistryFixture,
    primary_source: mizar_session::SourceId,
    primary_text: &str,
    fix: FixSuggestion,
    explanation: Option<ExplanationHandle>,
) -> BuildDiagnosticIndex {
    let (selected, _) = fixture
        .server
        .lsp_snapshot(&fixture.driver, &fixture.snapshots)
        .expect("published snapshot");
    let snapshot = selected.publication().captured().snapshot().id;
    let start = primary_text.find("theorem").unwrap_or(0);
    let span = DiagnosticSpan::new(
        SourceRange {
            source_id: primary_source,
            start,
            end: start + "theorem".len(),
        },
        DiagnosticSpanRole::Primary,
        None,
        SpanFreshness::Current,
        None,
    )
    .unwrap();
    let draft = DiagnosticDraft::new(DiagnosticDraftInput {
        source_snapshot: snapshot,
        code: DiagnosticCode::from_parts(DiagnosticSeverity::Error, 1).unwrap(),
        phase: PipelinePhase::Parser,
        category: FailureCategory::ParseError,
        stable_detail_key: "task38_deferred_action".to_owned(),
        message: "test diagnostic".to_owned(),
        primary_location: DiagnosticPrimaryLocation::Span(span),
        secondary_spans: vec![],
        notes: vec![],
        details: DiagnosticDetails::new(),
        fixes: vec![fix],
        explanation,
    })
    .unwrap();
    let mut sink = DiagnosticSink::new(DiagnosticProducerScope::new(
        PipelinePhase::Parser,
        snapshot,
        "task38-typed-consumer-fixture",
    ));
    sink.emit(draft).unwrap();
    BuildDiagnosticIndex::aggregate(DiagnosticAggregationInput::new(
        snapshot,
        vec![sink.into_batch()],
    ))
    .unwrap()
}

fn registration(producer: Arc<dyn crate::DeferredActionProducer>) -> DeferredFixRegistration {
    DeferredFixRegistration {
        text_hash_domain: None,
        artifacts: vec![],
        explanation: None,
        content: None,
        summary: None,
        missing_facts: None,
        command: None,
        store: None,
        producer,
    }
}

fn dependency_copy(dependency: &DeferredArtifactDependency) -> DeferredArtifactDependency {
    DeferredArtifactDependency {
        source: dependency.source.clone(),
        path: dependency.path.clone(),
        hash: dependency.hash.clone(),
    }
}

fn sole_source(fixture: &RegistryFixture) -> (String, mizar_session::SourceId, String) {
    text_sources(fixture).remove(0)
}

#[test]
fn loaded_hash_and_utf16_conversion_keep_raw_incarnation_guards() {
    let mut fixture = RegistryFixture::new();
    let uri = fixture.sources[0].2.clone();
    let original = fixture.server.document(&uri).unwrap().1.to_owned();
    fixture.did_change_text(&uri, 2, format!("\u{feff}:: café\r\n{original}"));
    assert!(fixture.build().1);
    let (selected, _) = fixture
        .server
        .lsp_snapshot(&fixture.driver, &fixture.snapshots)
        .unwrap();
    let captured = selected.native_open_buffers().captured();
    let (uri, source_id, loaded_text) = sole_source(&fixture);
    assert!(!loaded_text.starts_with('\u{feff}'));
    assert!(loaded_text.contains("café\n"));
    let required = mizar_session::hash_text(&loaded_text);
    let suggestion = fix(
        "loaded_hash",
        FixSafety::LocalTextEdit,
        vec![edit_for(source_id, &loaded_text, "theorem", "theorem")],
        Some(required),
        None,
    );
    let index = index_with_fix(&fixture, source_id, &loaded_text, suggestion.clone(), None);
    let record = &index.records()[0];
    let mut reg = registration(Arc::new(TestProducer::default()));
    reg.text_hash_domain = Some(TEXT_HASH_DOMAIN.to_owned());
    let context = DeferredActionContext {
        publication: selected.publication(),
        publication_identity: PUBLICATION,
        registrations: &[RegisteredDeferredFix {
            publication: PUBLICATION.to_owned(),
            diagnostic: record.handle(),
            fix: suggestion.clone(),
            registration: reg,
        }],
        supports_edits: true,
        commands: false,
    };
    assert!(validate_deferred_fix(
        &context,
        record,
        &suggestion,
        captured
    ));
    let selected_range = json!({"start":{"line":1,"character":0},"end":{"line":1,"character":7}});
    let offered = convert_code_actions_with_deferred(
        &index,
        captured,
        PUBLICATION,
        &uri,
        Some(&selected_range),
        None,
        Some(&context),
    )
    .unwrap();
    assert_eq!(offered.len(), 1);
    assert!(offered[0].get("edit").is_none());
    let resolve = json!({"data":offered[0]["data"]});
    let actions = convert_code_actions_with_deferred(
        &index,
        captured,
        PUBLICATION,
        &uri,
        None,
        Some(&resolve),
        Some(&context),
    )
    .unwrap();
    assert_eq!(actions.len(), 1);
    assert_eq!(
        actions[0]["edit"]["documentChanges"][0]["textDocument"]["version"],
        2
    );
    assert_eq!(
        actions[0]["edit"]["documentChanges"][0]["edits"][0]["range"]["start"],
        json!({"line":1,"character":0})
    );

    fixture.did_change_version_only(&uri, 3);
    assert_eq!(
        fixture
            .server
            .lsp_snapshot(&fixture.driver, &fixture.snapshots)
            .unwrap()
            .1,
        crate::server::LspSnapshotFreshness::StaleArtifact
    );
    fixture.did_close_and_reopen(&uri, 3);
    assert_eq!(
        fixture
            .server
            .lsp_snapshot(&fixture.driver, &fixture.snapshots)
            .unwrap()
            .1,
        crate::server::LspSnapshotFreshness::StaleArtifact
    );
    let retained = fixture
        .server
        .lsp_snapshot(&fixture.driver, &fixture.snapshots)
        .unwrap()
        .0;
    assert_eq!(
        retained.publication().captured().snapshot().id,
        selected.publication().captured().snapshot().id
    );
    fixture.did_change_text(
        &uri,
        4,
        format!("{} ", fixture.server.document(&uri).unwrap().1),
    );
    assert_eq!(
        fixture
            .server
            .lsp_snapshot(&fixture.driver, &fixture.snapshots)
            .unwrap()
            .1,
        crate::server::LspSnapshotFreshness::StaleArtifact
    );
}

#[test]
fn required_text_hash_is_sole_source_only_and_unhashed_multisource_keeps_text_checks() {
    let mut fixture = RegistryFixture::new();
    assert!(fixture.build().1);
    let (selected, _) = fixture
        .server
        .lsp_snapshot(&fixture.driver, &fixture.snapshots)
        .unwrap();
    let captured = selected.native_open_buffers().captured();
    let sources = text_sources(&fixture);
    let (uri, source_id, text) = sources[0].clone();
    let required = mizar_session::hash_text(&text);
    let one = fix(
        "single_source_hash",
        FixSafety::LocalTextEdit,
        vec![edit_for(source_id, &text, "theorem", "theorem")],
        Some(required),
        None,
    );
    let index = index_with_fix(&fixture, source_id, &text, one.clone(), None);
    let record = &index.records()[0];
    let mut correct = registration(Arc::new(TestProducer::default()));
    correct.text_hash_domain = Some(TEXT_HASH_DOMAIN.to_owned());
    let make_registered = |registration| RegisteredDeferredFix {
        publication: PUBLICATION.to_owned(),
        diagnostic: record.handle(),
        fix: one.clone(),
        registration,
    };
    let correct_registrations = vec![make_registered(correct)];
    let correct_context = DeferredActionContext {
        publication: selected.publication(),
        publication_identity: PUBLICATION,
        registrations: &correct_registrations,
        supports_edits: true,
        commands: false,
    };
    assert!(validate_deferred_fix(
        &correct_context,
        record,
        &one,
        captured
    ));
    let mut mismatch = registration(Arc::new(TestProducer::default()));
    mismatch.text_hash_domain = Some("unknown-domain".to_owned());
    let mismatch_registrations = vec![make_registered(mismatch)];
    let mismatch_context = DeferredActionContext {
        publication: selected.publication(),
        publication_identity: PUBLICATION,
        registrations: &mismatch_registrations,
        supports_edits: true,
        commands: false,
    };
    assert!(!validate_deferred_fix(
        &mismatch_context,
        record,
        &one,
        captured
    ));
    let missing_registrations = vec![make_registered(registration(Arc::new(
        TestProducer::default(),
    )))];
    let missing_context = DeferredActionContext {
        publication: selected.publication(),
        publication_identity: PUBLICATION,
        registrations: &missing_registrations,
        supports_edits: true,
        commands: false,
    };
    assert!(!validate_deferred_fix(
        &missing_context,
        record,
        &one,
        captured
    ));
    let wrong_digest = fix(
        "wrong_source_digest",
        FixSafety::LocalTextEdit,
        vec![edit_for(source_id, &text, "theorem", "theorem")],
        Some(Hash::from_bytes([7; 32])),
        None,
    );
    let wrong_digest_index = index_with_fix(&fixture, source_id, &text, wrong_digest.clone(), None);
    let wrong_digest_record = &wrong_digest_index.records()[0];
    let mut wrong_digest_registration = registration(Arc::new(TestProducer::default()));
    wrong_digest_registration.text_hash_domain = Some(TEXT_HASH_DOMAIN.to_owned());
    let wrong_digest_registrations = vec![RegisteredDeferredFix {
        publication: PUBLICATION.to_owned(),
        diagnostic: wrong_digest_record.handle(),
        fix: wrong_digest,
        registration: wrong_digest_registration,
    }];
    let wrong_digest_context = DeferredActionContext {
        publication: selected.publication(),
        publication_identity: PUBLICATION,
        registrations: &wrong_digest_registrations,
        supports_edits: true,
        commands: false,
    };
    assert!(!validate_deferred_fix(
        &wrong_digest_context,
        wrong_digest_record,
        &wrong_digest_registrations[0].fix,
        captured
    ));

    let second = &sources[1];
    let multi = fix(
        "hashed_multisource",
        FixSafety::LocalTextEdit,
        vec![
            edit_for(source_id, &text, "theorem", "theorem"),
            edit_for(second.1, &second.2, "theorem", "theorem"),
        ],
        Some(required),
        None,
    );
    let multi_index = index_with_fix(&fixture, source_id, &text, multi.clone(), None);
    let multi_record = &multi_index.records()[0];
    let mut declared = registration(Arc::new(TestProducer::default()));
    declared.text_hash_domain = Some(TEXT_HASH_DOMAIN.to_owned());
    let multi_context = DeferredActionContext {
        publication: selected.publication(),
        publication_identity: PUBLICATION,
        registrations: &[RegisteredDeferredFix {
            publication: PUBLICATION.to_owned(),
            diagnostic: multi_record.handle(),
            fix: multi.clone(),
            registration: declared,
        }],
        supports_edits: true,
        commands: false,
    };
    assert!(!validate_deferred_fix(
        &multi_context,
        multi_record,
        &multi,
        captured
    ));

    let unhashed = fix(
        "unhashed_multisource",
        FixSafety::LocalTextEdit,
        vec![
            edit_for(source_id, &text, "theorem", "theorem"),
            edit_for(second.1, &second.2, "theorem", "theorem"),
        ],
        None,
        None,
    );
    let unhashed_index = index_with_fix(&fixture, source_id, &text, unhashed, None);
    let actions = convert_code_actions_with_deferred(
        &unhashed_index,
        captured,
        PUBLICATION,
        &uri,
        None,
        None,
        None,
    )
    .unwrap();
    assert_eq!(actions.len(), 1);
    let expected_start = text.find("theorem").unwrap();
    let wrong_expected = fix(
        "wrong_expected_text",
        FixSafety::LocalTextEdit,
        vec![
            FixEdit::new(
                SourceRange {
                    source_id,
                    start: expected_start,
                    end: expected_start + "theorem".len(),
                },
                "x",
                Some("z".to_owned()),
            )
            .unwrap(),
        ],
        None,
        None,
    );
    let wrong_index = index_with_fix(&fixture, source_id, &text, wrong_expected, None);
    assert!(
        convert_code_actions_with_deferred(
            &wrong_index,
            captured,
            PUBLICATION,
            &uri,
            None,
            None,
            None
        )
        .unwrap()
        .is_empty()
    );
}

#[test]
fn registered_backing_and_selector_require_exact_sources_and_all_dependencies() {
    let mut fixture = RegistryFixture::new();
    assert!(fixture.build().1);
    let (selected, _) = fixture
        .server
        .lsp_snapshot(&fixture.driver, &fixture.snapshots)
        .unwrap();
    let publication = selected.publication();
    let sources = text_sources(&fixture);
    let (_, source_id, text) = &sources[0];
    let source_version = publication
        .captured()
        .snapshot()
        .source_versions
        .iter()
        .find(|version| version.source_id == *source_id)
        .unwrap()
        .clone();
    assert_eq!(source_version.source_hash, mizar_session::hash_text(text));
    let module = publication
        .manifests()
        .iter()
        .flat_map(|manifest| manifest.manifest.modules.iter())
        .find(|module| module.source_hash == source_version.source_hash)
        .unwrap();
    let dependency = DeferredArtifactDependency {
        source: source_version.clone(),
        path: module.artifact_file.clone(),
        hash: module.artifact_hash.clone(),
    };
    let second_version = publication
        .captured()
        .snapshot()
        .source_versions
        .iter()
        .find(|version| version.source_id == sources[1].1)
        .unwrap()
        .clone();
    let second_module = publication
        .manifests()
        .iter()
        .flat_map(|read| read.manifest.modules.iter())
        .find(|module| module.source_hash == second_version.source_hash)
        .unwrap();
    let second_dependency = DeferredArtifactDependency {
        source: second_version,
        path: second_module.artifact_file.clone(),
        hash: second_module.artifact_hash.clone(),
    };
    let content = b"canonical retained content".to_vec();
    let summary = b"canonical bounded summary".to_vec();
    let content_hash = action_backing_hash("content", "owner/v1", &content);
    let summary_hash = action_backing_hash("summary", "owner/v1", &summary);
    let handle = explanation_handle(
        "deferred_backing",
        "task38_deferred_action",
        ExplanationSourceRef::CacheRecord {
            cache_key: "cache_key".to_owned(),
            content_hash: Some(content_hash),
        },
        Some(summary_hash),
        Some(publication.captured().snapshot().id),
    );
    let payload = ExplanationPayload::with_bounds(
        ExplanationPreviewFormat::PlainText,
        "bounded explanation",
        Some(summary_hash),
        512,
        8,
    )
    .unwrap();
    let mut store = ExplanationStore::new();
    store.insert_payload(&handle, payload);
    let producer = Arc::new(TestProducer {
        accepted: vec![
            ("content".to_owned(), "owner/v1".to_owned(), content.clone()),
            ("summary".to_owned(), "owner/v1".to_owned(), summary.clone()),
        ],
        expected_payload: Some("bounded explanation".to_owned()),
        expected_handle: Some("deferred_backing".to_owned()),
    });
    let fix = fix(
        "artifact_fix",
        FixSafety::ArtifactAssisted,
        vec![edit_for(*source_id, text, "theorem", "theorem")],
        Some(source_version.source_hash),
        None,
    );
    let index = index_with_fix(
        &fixture,
        *source_id,
        text,
        fix.clone(),
        Some(handle.clone()),
    );
    let record = &index.records()[0];
    let good = DeferredFixRegistration {
        text_hash_domain: Some(TEXT_HASH_DOMAIN.to_owned()),
        artifacts: vec![
            dependency_copy(&dependency),
            dependency_copy(&second_dependency),
        ],
        explanation: Some(handle.clone()),
        content: Some(("owner/v1".to_owned(), content.clone())),
        summary: Some(("owner/v1".to_owned(), summary.clone())),
        missing_facts: None,
        command: None,
        store: Some(Arc::new(store.clone())),
        producer: producer.clone(),
    };
    let good_context = DeferredActionContext {
        publication,
        publication_identity: PUBLICATION,
        registrations: &[RegisteredDeferredFix {
            publication: PUBLICATION.to_owned(),
            diagnostic: record.handle(),
            fix: fix.clone(),
            registration: good,
        }],
        supports_edits: true,
        commands: true,
    };
    assert!(validate_deferred_fix(
        &good_context,
        record,
        &fix,
        publication.captured()
    ));

    let selector_registration = DeferredFixRegistration {
        text_hash_domain: Some(TEXT_HASH_DOMAIN.to_owned()),
        artifacts: vec![
            dependency_copy(&dependency),
            dependency_copy(&second_dependency),
        ],
        explanation: None,
        content: None,
        summary: None,
        missing_facts: Some((
            0,
            "unowned_obligation".to_owned(),
            "unowned_entry".to_owned(),
        )),
        command: None,
        store: None,
        producer: producer.clone(),
    };
    let selector_registrations = vec![RegisteredDeferredFix {
        publication: PUBLICATION.to_owned(),
        diagnostic: record.handle(),
        fix: fix.clone(),
        registration: selector_registration,
    }];
    let selector_context = DeferredActionContext {
        publication,
        publication_identity: PUBLICATION,
        registrations: &selector_registrations,
        supports_edits: true,
        commands: true,
    };
    assert!(!validate_deferred_fix(
        &selector_context,
        record,
        &fix,
        publication.captured()
    ));
    let bad_selector = DeferredFixRegistration {
        text_hash_domain: Some(TEXT_HASH_DOMAIN.to_owned()),
        artifacts: vec![
            dependency_copy(&dependency),
            dependency_copy(&second_dependency),
        ],
        explanation: None,
        content: None,
        summary: None,
        missing_facts: Some((9, "obligation-7".to_owned(), "fact-2".to_owned())),
        command: None,
        store: None,
        producer: producer.clone(),
    };
    let bad_selector_registrations = vec![RegisteredDeferredFix {
        publication: PUBLICATION.to_owned(),
        diagnostic: record.handle(),
        fix: fix.clone(),
        registration: bad_selector,
    }];
    let bad_selector_context = DeferredActionContext {
        publication,
        publication_identity: PUBLICATION,
        registrations: &bad_selector_registrations,
        supports_edits: true,
        commands: true,
    };
    assert!(!validate_deferred_fix(
        &bad_selector_context,
        record,
        &fix,
        publication.captured()
    ));

    let duplicate_registrations = vec![
        RegisteredDeferredFix {
            publication: PUBLICATION.to_owned(),
            diagnostic: record.handle(),
            fix: fix.clone(),
            registration: DeferredFixRegistration {
                text_hash_domain: Some(TEXT_HASH_DOMAIN.to_owned()),
                artifacts: vec![
                    dependency_copy(&dependency),
                    dependency_copy(&second_dependency),
                ],
                explanation: Some(handle.clone()),
                content: Some(("owner/v1".to_owned(), content.clone())),
                summary: Some(("owner/v1".to_owned(), summary.clone())),
                missing_facts: None,
                command: None,
                store: Some(Arc::new(store.clone())),
                producer: producer.clone(),
            },
        },
        RegisteredDeferredFix {
            publication: PUBLICATION.to_owned(),
            diagnostic: record.handle(),
            fix: fix.clone(),
            registration: DeferredFixRegistration {
                text_hash_domain: Some(TEXT_HASH_DOMAIN.to_owned()),
                artifacts: vec![
                    dependency_copy(&dependency),
                    dependency_copy(&second_dependency),
                ],
                explanation: Some(handle.clone()),
                content: Some(("owner/v1".to_owned(), content.clone())),
                summary: Some(("owner/v1".to_owned(), summary.clone())),
                missing_facts: None,
                command: None,
                store: Some(Arc::new(store.clone())),
                producer: producer.clone(),
            },
        },
    ];
    let duplicate_context = DeferredActionContext {
        publication,
        publication_identity: PUBLICATION,
        registrations: &duplicate_registrations,
        supports_edits: true,
        commands: true,
    };
    assert!(!validate_deferred_fix(
        &duplicate_context,
        record,
        &fix,
        publication.captured()
    ));
    let wrong_handle = explanation_handle(
        "other_handle",
        "task38_deferred_action",
        handle.source().clone(),
        Some(summary_hash),
        Some(publication.captured().snapshot().id),
    );
    let wrong_handle_context = DeferredActionContext {
        publication,
        publication_identity: PUBLICATION,
        registrations: &[RegisteredDeferredFix {
            publication: PUBLICATION.to_owned(),
            diagnostic: record.handle(),
            fix: fix.clone(),
            registration: DeferredFixRegistration {
                text_hash_domain: Some(TEXT_HASH_DOMAIN.to_owned()),
                artifacts: vec![
                    dependency_copy(&dependency),
                    dependency_copy(&second_dependency),
                ],
                explanation: Some(wrong_handle),
                content: Some(("owner/v1".to_owned(), content.clone())),
                summary: Some(("owner/v1".to_owned(), summary.clone())),
                missing_facts: None,
                command: None,
                store: Some(Arc::new(store.clone())),
                producer: producer.clone(),
            },
        }],
        supports_edits: true,
        commands: true,
    };
    assert!(!validate_deferred_fix(
        &wrong_handle_context,
        record,
        &fix,
        publication.captured()
    ));

    for (wrong_source, wrong_path, wrong_hash) in [
        (true, false, false),
        (false, true, false),
        (false, false, true),
    ] {
        let mut wrong = DeferredFixRegistration {
            text_hash_domain: Some(TEXT_HASH_DOMAIN.to_owned()),
            artifacts: vec![
                dependency_copy(&dependency),
                dependency_copy(&second_dependency),
            ],
            explanation: Some(handle.clone()),
            content: Some(("owner/v1".to_owned(), content.clone())),
            summary: Some(("owner/v1".to_owned(), summary.clone())),
            missing_facts: None,
            command: None,
            store: Some(Arc::new(store.clone())),
            producer: producer.clone(),
        };
        if wrong_source {
            wrong.artifacts[0].source.source_id = sources[1].1;
        }
        if wrong_path {
            wrong.artifacts[0].path.push_str("-wrong");
        }
        if wrong_hash {
            wrong.artifacts[1].hash = second_module.interface_hash.clone();
        }
        let wrong_context = DeferredActionContext {
            publication,
            publication_identity: PUBLICATION,
            registrations: &[RegisteredDeferredFix {
                publication: PUBLICATION.to_owned(),
                diagnostic: record.handle(),
                fix: fix.clone(),
                registration: wrong,
            }],
            supports_edits: true,
            commands: true,
        };
        assert!(!validate_deferred_fix(
            &wrong_context,
            record,
            &fix,
            publication.captured()
        ));
    }
    let wrong_content = DeferredFixRegistration {
        text_hash_domain: Some(TEXT_HASH_DOMAIN.to_owned()),
        artifacts: vec![
            dependency_copy(&dependency),
            dependency_copy(&second_dependency),
        ],
        explanation: Some(handle.clone()),
        content: Some(("owner/v1".to_owned(), b"changed".to_vec())),
        summary: Some(("owner/v1".to_owned(), summary.clone())),
        missing_facts: None,
        command: None,
        store: Some(Arc::new(store.clone())),
        producer: producer.clone(),
    };
    let wrong_content_context = DeferredActionContext {
        publication,
        publication_identity: PUBLICATION,
        registrations: &[RegisteredDeferredFix {
            publication: PUBLICATION.to_owned(),
            diagnostic: record.handle(),
            fix: fix.clone(),
            registration: wrong_content,
        }],
        supports_edits: true,
        commands: true,
    };
    assert!(!validate_deferred_fix(
        &wrong_content_context,
        record,
        &fix,
        publication.captured()
    ));

    let mut wrong_schema = DeferredFixRegistration {
        text_hash_domain: Some(TEXT_HASH_DOMAIN.to_owned()),
        artifacts: vec![
            dependency_copy(&dependency),
            dependency_copy(&second_dependency),
        ],
        explanation: Some(handle.clone()),
        content: Some(("owner/v1".to_owned(), content.clone())),
        summary: Some(("owner/v1".to_owned(), summary.clone())),
        missing_facts: None,
        command: None,
        store: Some(Arc::new(store.clone())),
        producer: producer.clone(),
    };
    wrong_schema.artifacts[1]
        .hash
        .schema_family
        .push_str("-wrong");
    let wrong_schema_context = DeferredActionContext {
        publication,
        publication_identity: PUBLICATION,
        registrations: &[RegisteredDeferredFix {
            publication: PUBLICATION.to_owned(),
            diagnostic: record.handle(),
            fix: fix.clone(),
            registration: wrong_schema,
        }],
        supports_edits: true,
        commands: true,
    };
    assert!(!validate_deferred_fix(
        &wrong_schema_context,
        record,
        &fix,
        publication.captured()
    ));

    let wrong_summary = DeferredFixRegistration {
        text_hash_domain: Some(TEXT_HASH_DOMAIN.to_owned()),
        artifacts: vec![
            dependency_copy(&dependency),
            dependency_copy(&second_dependency),
        ],
        explanation: Some(handle.clone()),
        content: Some(("owner/v1".to_owned(), content.clone())),
        summary: Some(("owner/v1".to_owned(), b"different summary".to_vec())),
        missing_facts: None,
        command: None,
        store: Some(Arc::new(store.clone())),
        producer: producer.clone(),
    };
    let wrong_summary_context = DeferredActionContext {
        publication,
        publication_identity: PUBLICATION,
        registrations: &[RegisteredDeferredFix {
            publication: PUBLICATION.to_owned(),
            diagnostic: record.handle(),
            fix: fix.clone(),
            registration: wrong_summary,
        }],
        supports_edits: true,
        commands: true,
    };
    assert!(!validate_deferred_fix(
        &wrong_summary_context,
        record,
        &fix,
        publication.captured()
    ));
}

#[test]
fn backing_hash_domain_uses_role_encoding_and_length_framed_bytes() {
    let role = b"content";
    let encoding = b"owner/v1";
    let bytes = b"abc";
    let mut framed = b"mizar-lsp/action-backing/v1".to_vec();
    for part in [role.as_slice(), encoding.as_slice(), bytes.as_slice()] {
        framed.extend_from_slice(&(part.len() as u64).to_le_bytes());
        framed.extend_from_slice(part);
    }
    let independent_oracle = Hash::from_bytes(*blake3::hash(&framed).as_bytes());
    assert_eq!(
        action_backing_hash("content", "owner/v1", b"abc"),
        independent_oracle
    );
    assert_ne!(
        independent_oracle,
        action_backing_hash("summary", "owner/v1", b"abc")
    );
    assert_ne!(
        independent_oracle,
        action_backing_hash("content", "owner/v2", b"abc")
    );
    assert_ne!(
        independent_oracle,
        action_backing_hash("content", "owner/v1", b"ab")
    );
}

#[test]
fn bindings_commands_and_lifecycle_are_revalidated_without_effectful_owner() {
    let mut fixture = RegistryFixture::new();
    assert!(fixture.build().1);
    let (selected, _) = fixture
        .server
        .lsp_snapshot(&fixture.driver, &fixture.snapshots)
        .unwrap();
    let publication = selected.publication();
    let (uri, source_id, text) = sole_source(&fixture);
    let content = b"registered command content".to_vec();
    let summary = b"registered command summary".to_vec();
    let content_hash = action_backing_hash("content", "command/v1", &content);
    let summary_hash = action_backing_hash("summary", "command/v1", &summary);
    let stable = "task38_deferred_action";
    let handle = explanation_handle(
        "command_explanation",
        stable,
        ExplanationSourceRef::CacheRecord {
            cache_key: "command_cache".to_owned(),
            content_hash: Some(content_hash),
        },
        Some(summary_hash),
        Some(publication.captured().snapshot().id),
    );
    let payload = ExplanationPayload::with_bounds(
        ExplanationPreviewFormat::PlainText,
        "validated command explanation",
        Some(summary_hash),
        512,
        8,
    )
    .unwrap();
    let mut store = ExplanationStore::new();
    store.insert_payload(&handle, payload);
    let cmd = FixCommandRef::new("explain_current").unwrap();
    let command_fix = fix(
        "explain",
        FixSafety::CommandOnly,
        vec![],
        None,
        Some(cmd.clone()),
    );
    let index = index_with_fix(
        &fixture,
        source_id,
        &text,
        command_fix.clone(),
        Some(handle.clone()),
    );
    let record = &index.records()[0];
    let producer = Arc::new(TestProducer {
        accepted: vec![
            (
                "content".to_owned(),
                "command/v1".to_owned(),
                content.clone(),
            ),
            (
                "summary".to_owned(),
                "command/v1".to_owned(),
                summary.clone(),
            ),
        ],
        expected_payload: Some("validated command explanation".to_owned()),
        expected_handle: Some("command_explanation".to_owned()),
    });
    let command_registration = DeferredFixRegistration {
        text_hash_domain: None,
        artifacts: vec![],
        explanation: Some(handle.clone()),
        content: Some(("command/v1".to_owned(), content)),
        summary: Some(("command/v1".to_owned(), summary)),
        missing_facts: None,
        command: Some(cmd),
        store: Some(Arc::new(store)),
        producer,
    };
    let registrations = vec![RegisteredDeferredFix {
        publication: PUBLICATION.to_owned(),
        diagnostic: record.handle(),
        fix: command_fix.clone(),
        registration: command_registration,
    }];
    let context = DeferredActionContext {
        publication,
        publication_identity: PUBLICATION,
        registrations: &registrations,
        supports_edits: true,
        commands: true,
    };
    let selection = json!({"start":{"line":0,"character":0},"end":{"line":0,"character":7}});
    let offered = convert_code_actions_with_deferred(
        &index,
        publication.captured(),
        PUBLICATION,
        &uri,
        Some(&selection),
        None,
        Some(&context),
    )
    .unwrap();
    assert_eq!(offered.len(), 1);
    assert_eq!(offered[0]["title"], command_fix.title());
    assert_eq!(offered[0]["kind"], "quickfix");
    assert!(offered[0].get("command").is_none());
    assert!(offered[0].get("edit").is_none());
    assert_eq!(offered[0].as_object().unwrap().len(), 3);
    let resolve_params = json!({"data":offered[0]["data"]});
    let actions = convert_code_actions_with_deferred(
        &index,
        publication.captured(),
        PUBLICATION,
        &uri,
        None,
        Some(&resolve_params),
        Some(&context),
    )
    .unwrap();
    assert_eq!(actions.len(), 1);
    assert_eq!(
        actions[0]["command"]["command"],
        "mizar.executeDiagnosticCommand"
    );
    assert_eq!(
        actions[0]["command"]["arguments"].as_array().unwrap().len(),
        1
    );
    let data = actions[0]["command"]["arguments"][0].clone();
    let resolved =
        execute_deferred_command(&index, publication.captured(), PUBLICATION, &context, &data)
            .unwrap();
    assert_eq!(resolved.unwrap()["status"], "available");

    let mut forged = data.clone();
    forged["publication"] = json!("other-publication");
    assert!(
        execute_deferred_command(
            &index,
            publication.captured(),
            PUBLICATION,
            &context,
            &forged
        )
        .unwrap()
        .is_none()
    );
    let no_command_support = DeferredActionContext {
        publication,
        publication_identity: PUBLICATION,
        registrations: &registrations,
        supports_edits: false,
        commands: false,
    };
    assert!(
        convert_code_actions_with_deferred(
            &index,
            publication.captured(),
            PUBLICATION,
            &uri,
            Some(&selection),
            None,
            Some(&no_command_support),
        )
        .unwrap()
        .is_empty()
    );
    let no_owner = DeferredActionContext {
        publication,
        publication_identity: PUBLICATION,
        registrations: &[],
        supports_edits: true,
        commands: true,
    };
    assert!(
        convert_code_actions_with_deferred(
            &index,
            publication.captured(),
            PUBLICATION,
            &uri,
            Some(&selection),
            None,
            Some(&no_owner),
        )
        .unwrap()
        .is_empty()
    );

    let mut wire_server = crate::Server::default();
    assert!(wire_server.enable_diagnostic_commands());
    let initialized = wire_server
        .handle_message(json!({
            "jsonrpc":"2.0","id":1,"method":"initialize",
            "params":{"processId":null,"rootUri":null,"capabilities":{}}
        }))
        .unwrap();
    assert_eq!(
        initialized["result"]["capabilities"]["executeCommandProvider"]["commands"][0],
        "mizar.executeDiagnosticCommand"
    );
    let mut wire_request = |command: &str, arguments: Value| {
        wire_server
            .handle_message(json!({
                "jsonrpc":"2.0","id":2,"method":"workspace/executeCommand",
                "params":{"command":command,"arguments":arguments}
            }))
            .unwrap()
    };
    assert_eq!(
        wire_request("unknown.command", json!([data]))["error"]["code"],
        -32601
    );
    assert_eq!(
        wire_request(
            "mizar.executeDiagnosticCommand",
            json!([data.clone(), data.clone()])
        )["error"]["code"],
        -32602
    );
    drop(wire_request);
    let request = json!({
        "jsonrpc":"2.0","id":3,"method":"workspace/executeCommand",
        "params":{"command":"mizar.executeDiagnosticCommand","arguments":[data]}
    });
    let cancelled = wire_server
        .handle_message_with_diagnostic_commands(
            request.clone(),
            &fixture.driver,
            &fixture.snapshots,
            &|| true,
        )
        .unwrap();
    assert_eq!(cancelled["error"]["code"], -32800);
    let stale = wire_server
        .handle_message_with_diagnostic_commands(
            request,
            &fixture.driver,
            &fixture.snapshots,
            &|| false,
        )
        .unwrap();
    assert_eq!(stale["error"]["code"], -32801);

    // Exercise the server wrapper with the fixture's genuine current publication.
    // The fixture has no producer-owned fix, so execution must remain unavailable.
    fixture.server.diagnostic_commands_enabled = true;
    let current_request = json!({
        "jsonrpc":"2.0","id":4,"method":"workspace/executeCommand",
        "params":{"command":"mizar.executeDiagnosticCommand","arguments":[{
            "publication":"opaque-publication","snapshot":"opaque-snapshot",
            "fix":"unowned_fix","uri":uri.clone(),"version":1,"id":0
        }]}
    });
    let cancelled_current = fixture
        .server
        .handle_message_with_diagnostic_commands(
            current_request.clone(),
            &fixture.driver,
            &fixture.snapshots,
            &|| true,
        )
        .unwrap();
    assert_eq!(cancelled_current["error"]["code"], -32800);
    let unavailable_current = fixture
        .server
        .handle_message_with_diagnostic_commands(
            current_request.clone(),
            &fixture.driver,
            &fixture.snapshots,
            &|| false,
        )
        .unwrap();
    assert_eq!(unavailable_current["error"]["code"], -32801);

    fixture.did_change_version_only(&uri, 2);
    assert_eq!(
        fixture
            .server
            .lsp_snapshot(&fixture.driver, &fixture.snapshots)
            .unwrap()
            .1,
        crate::server::LspSnapshotFreshness::StaleArtifact
    );
    let unavailable_stale = fixture
        .server
        .handle_message_with_diagnostic_commands(
            current_request,
            &fixture.driver,
            &fixture.snapshots,
            &|| false,
        )
        .unwrap();
    assert_eq!(unavailable_stale["error"]["code"], -32801);
}
