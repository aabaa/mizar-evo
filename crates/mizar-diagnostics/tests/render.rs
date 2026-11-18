use std::{collections::HashMap, fs, str::FromStr};

use mizar_diagnostics::{
    aggregator::BuildDiagnosticIndex,
    explain::{
        ExplanationHandle, ExplanationHandleId, ExplanationHandleInput, ExplanationKind,
        ExplanationPreview, ExplanationPreviewFormat, ExplanationSourceRef, ExplanationSubject,
    },
    failure_record::{
        DiagnosticDetailValue, DiagnosticDetails, DiagnosticDraft, DiagnosticDraftInput,
        DiagnosticFreshness, DiagnosticHandle, DiagnosticId, DiagnosticNote, DiagnosticNoteKind,
        DiagnosticPrimaryLocation, DiagnosticSpan, DiagnosticSpanRole, FailureCategory,
        PipelinePhase, SpanFreshness, ZeroWidthSpanIntent,
    },
    fix::{
        FixApplicability, FixCommandRef, FixEdit, FixSafety, FixSuggestion, FixSuggestionId,
        FixSuggestionInput,
    },
    registry::DiagnosticCode,
    render::{
        DiagnosticRenderInput, DiagnosticSourceContext, RenderOptions, RenderStyle,
        render_diagnostics,
    },
    sink::{DiagnosticProducerScope, DiagnosticSink},
};
use mizar_session::{
    BuildSnapshotId, Hash, InMemorySessionIdAllocator, LineColumn, LineColumnRange, NormalizedPath,
    PackageId, SessionIdAllocator, SourceAnchor, SourceId, SourceRange, normalize_source_path,
};

#[test]
fn plain_rendering_is_byte_stable_for_primary_secondary_notes_and_refs() {
    let snapshot = snapshot_id(1);
    let source_id = source_id(snapshot);
    let context = TestSourceContext::new().with_source(
        source_id,
        "src/lib.miz",
        "Src(1)",
        "theorem Main:\n  a + b\nend\n",
    );
    let secondary = DiagnosticSpan::secondary(
        SourceRange {
            source_id,
            start: 0,
            end: 7,
        },
        Some("declaration starts here".to_owned()),
    )
    .expect("valid secondary span");
    let note_span = DiagnosticSpan::secondary(
        SourceRange {
            source_id,
            start: 22,
            end: 25,
        },
        Some("closing token".to_owned()),
    )
    .expect("valid note span");
    let record = record(
        RecordFixture::new(snapshot, source_id, 16, 21, "ambiguous addition")
            .secondary_spans(vec![secondary])
            .notes(vec![
                DiagnosticNote::new(DiagnosticNoteKind::Note, "two candidates remain", None),
                DiagnosticNote::new(
                    DiagnosticNoteKind::Help,
                    "qualify one candidate",
                    Some(note_span),
                ),
            ])
            .fixes(vec![informational_fix(
                "render.qualify",
                "qualify one candidate",
            )])
            .explanation(explanation("render.explain", "show overload candidates")),
    );

    assert_eq!(
        render_diagnostics(DiagnosticRenderInput::new(
            std::slice::from_ref(&record),
            &context,
            RenderOptions::plain(),
        )),
        concat!(
            "error[E0001]: ambiguous addition (syntax.unexpected_token)\n",
            "  --> src/lib.miz:2:3\n",
            "   |\n",
            " 2 |   a + b\n",
            "   |   ^^^^^ ambiguous addition\n",
            "  --> src/lib.miz:1:1\n",
            "   |\n",
            " 1 | theorem Main:\n",
            "   | ------- declaration starts here\n",
            "   = note: two candidates remain\n",
            "  --> src/lib.miz:3:1\n",
            "   |\n",
            " 3 | end\n",
            "   | --- closing token\n",
            "   = help: qualify one candidate\n",
            "   = help: fix suggestion `render.qualify`: qualify one candidate\n",
            "   = explain: `render.explain`: show overload candidates",
        )
    );
}

#[test]
fn source_load_rendering_uses_request_path_without_source_context() {
    let snapshot = snapshot_id(21);
    let path = normalized_path("src/missing.miz");
    let draft = DiagnosticDraft::new(DiagnosticDraftInput {
        source_snapshot: snapshot,
        code: DiagnosticCode::from_str("E0602").expect("allocated code"),
        phase: PipelinePhase::SourceLoad,
        category: FailureCategory::SourceLoadError,
        stable_detail_key: "source.unreadable_file".to_owned(),
        message: "cannot read source".to_owned(),
        primary_location: DiagnosticPrimaryLocation::SourceLoad {
            package_id: PackageId::new("demo"),
            path,
        },
        secondary_spans: vec![],
        notes: vec![],
        details: DiagnosticDetails::new(),
        fixes: vec![],
        explanation: None,
    })
    .expect("valid source-load draft");
    let mut sink = DiagnosticSink::new(DiagnosticProducerScope::new(
        PipelinePhase::SourceLoad,
        snapshot,
        "source-loader",
    ));
    sink.emit(draft).expect("source-load draft enters sink");
    let index = BuildDiagnosticIndex::from_batches(snapshot, vec![sink.into_batch()])
        .expect("source-load batch aggregates");

    let rendered = render_diagnostics(DiagnosticRenderInput::new(
        index.records(),
        &PanicSourceContext,
        RenderOptions::plain(),
    ));
    assert_eq!(
        rendered,
        concat!(
            "error[E0602]: cannot read source (source.unreadable_file)\n",
            "  --> src/missing.miz"
        )
    );
}

#[test]
fn source_load_primary_keeps_real_secondary_span_rendering() {
    let snapshot = snapshot_id(22);
    let secondary_source = source_id(snapshot);
    let context = TestSourceContext::new().with_source(
        secondary_source,
        "src/related.miz",
        "related-source",
        "abc\n",
    );
    let draft = DiagnosticDraft::new(DiagnosticDraftInput {
        source_snapshot: snapshot,
        code: DiagnosticCode::from_str("E0600").expect("allocated code"),
        phase: PipelinePhase::SourceLoad,
        category: FailureCategory::SourceLoadError,
        stable_detail_key: "source.unmapped_buffer".to_owned(),
        message: "source request failed".to_owned(),
        primary_location: DiagnosticPrimaryLocation::SourceLoad {
            package_id: PackageId::new("demo"),
            path: normalized_path("src/requested.miz"),
        },
        secondary_spans: vec![
            DiagnosticSpan::secondary(
                SourceRange {
                    source_id: secondary_source,
                    start: 0,
                    end: 3,
                },
                Some("related source".to_owned()),
            )
            .expect("valid secondary span"),
        ],
        notes: vec![],
        details: DiagnosticDetails::new(),
        fixes: vec![],
        explanation: None,
    })
    .expect("valid source-load draft");
    let mut sink = DiagnosticSink::new(DiagnosticProducerScope::new(
        PipelinePhase::SourceLoad,
        snapshot,
        "source-loader",
    ));
    sink.emit(draft).expect("source-load draft enters sink");
    let index = BuildDiagnosticIndex::from_batches(snapshot, vec![sink.into_batch()])
        .expect("source-load batch aggregates");

    let rendered = render_diagnostics(DiagnosticRenderInput::new(
        index.records(),
        &context,
        RenderOptions::plain(),
    ));
    assert_eq!(
        rendered,
        concat!(
            "error[E0600]: source request failed (source.load_failed)\n",
            "  --> src/requested.miz\n",
            "  --> src/related.miz:1:1\n",
            "   |\n",
            " 1 | abc\n",
            "   | --- related source"
        )
    );
}

#[test]
fn documentation_draft_to_cli_renders_provenance_without_primary_source_lookup() {
    let snapshot = snapshot_id(40);
    let input = DiagnosticDraftInput {
        source_snapshot: snapshot,
        code: DiagnosticCode::from_str("W0411").unwrap(),
        phase: PipelinePhase::Documentation,
        category: FailureCategory::DocumentationWarning,
        stable_detail_key: "documentation.markdown.conversion_failure".into(),
        message: "source map conversion failed".into(),
        primary_location: DiagnosticPrimaryLocation::Documentation {
            publication_hash: Hash::from_bytes([0xab; Hash::BYTE_LEN]),
            package_id: "pkg".into(),
            package_version: Some(String::new()),
            lockfile_identity: None,
            module_path: "src/doc.miz".into(),
            language_edition: "2026".into(),
            block_ordinal: 12,
        },
        secondary_spans: vec![],
        notes: vec![],
        details: DiagnosticDetails::new(),
        fixes: vec![],
        explanation: None,
    };
    let publish = |input| {
        let mut sink = DiagnosticSink::new(DiagnosticProducerScope::new(
            PipelinePhase::Documentation,
            snapshot,
            "documentation",
        ));
        sink.emit(DiagnosticDraft::new(input).unwrap()).unwrap();
        BuildDiagnosticIndex::from_batches(snapshot, vec![sink.into_batch()]).unwrap()
    };
    let index = publish(input.clone());
    let expected = format!(
        "warning[W0411]: source map conversion failed (documentation.markdown.conversion_failure)\n  --> documentation publication={} package=\"pkg\" version=Some(\"\") lockfile=None module=\"src/doc.miz\" edition=\"2026\" block=12",
        "ab".repeat(32)
    );
    let rendered = render_diagnostics(DiagnosticRenderInput::new(
        index.records(),
        &PanicSourceContext,
        RenderOptions::plain(),
    ));
    assert_eq!(rendered, expected);
    let source = source_id(snapshot);
    let context = TestSourceContext::new().with_source(source, "related.miz", "candidate", "abc\n");
    let mut related = input;
    related.secondary_spans = vec![
        DiagnosticSpan::secondary(
            SourceRange {
                source_id: source,
                start: 0,
                end: 3,
            },
            Some("candidate".into()),
        )
        .unwrap(),
    ];
    let index = publish(related);
    let rendered = render_diagnostics(DiagnosticRenderInput::new(
        index.records(),
        &context,
        RenderOptions::plain(),
    ));
    assert_eq!(
        rendered,
        format!("{expected}\n  --> related.miz:1:1\n   |\n 1 | abc\n   | --- candidate")
    );
}

fn link_input(snapshot: BuildSnapshotId, scope: &str) -> DiagnosticDraftInput {
    use DiagnosticDetailValue::{List, String as Text};
    let empty = List(vec![]);
    let module = List(vec![
        Text("pkg".into()),
        empty.clone(),
        empty.clone(),
        Text("doc.miz".into()),
        Text("2026".into()),
    ]);
    let target = match scope {
        "module_page" => List(vec![Text("module".into())]),
        "section_index" => empty.clone(),
        _ => List(vec![
            Text("item".into()),
            Text("origin-1".into()),
            Text("pkg.T".into()),
        ]),
    };
    let occurrence = if scope == "reference" {
        List(vec![
            Text("0".into()),
            Text("3".into()),
            Text("8".into()),
            Text("code".into()),
        ])
    } else {
        empty.clone()
    };
    let subject = List(vec![
        Text("ab".repeat(32)),
        module,
        target,
        Text(scope.into()),
        occurrence,
    ]);
    let primary_location = if scope == "reference" {
        DiagnosticPrimaryLocation::Documentation {
            publication_hash: Hash::from_bytes([0xab; Hash::BYTE_LEN]),
            package_id: "pkg".into(),
            package_version: None,
            lockfile_identity: None,
            module_path: "doc.miz".into(),
            language_edition: "2026".into(),
            block_ordinal: 0,
        }
    } else {
        DiagnosticPrimaryLocation::DocumentationScope {
            publication_hash: Hash::from_bytes([0xab; Hash::BYTE_LEN]),
            package_id: "pkg".into(),
            package_version: None,
            lockfile_identity: None,
            module_path: "doc.miz".into(),
            language_edition: "2026".into(),
        }
    };
    DiagnosticDraftInput {
        source_snapshot: snapshot,
        code: DiagnosticCode::from_str("W0414").unwrap(),
        phase: PipelinePhase::Documentation,
        category: FailureCategory::DocumentationWarning,
        stable_detail_key: "documentation.link.incomplete_input".into(),
        message: "link input".into(),
        primary_location,
        secondary_spans: vec![],
        notes: vec![],
        fixes: vec![],
        explanation: None,
        details: DiagnosticDetails::from_entries([
            ("documentation.link.subject", subject),
            ("documentation.link.reference", empty.clone()),
            ("documentation.link.section_version", empty.clone()),
            ("documentation.link.candidates", empty.clone()),
            (
                "documentation.link.causes",
                List(vec![List(vec![
                    Text("page_unavailable".into()),
                    empty.clone(),
                ])]),
            ),
            ("documentation.link.routes", empty.clone()),
            ("documentation.link.citation_coverage", empty),
        ])
        .unwrap(),
    }
}

fn link_index(
    snapshot: BuildSnapshotId,
    inputs: Vec<DiagnosticDraftInput>,
) -> BuildDiagnosticIndex {
    let batches = inputs
        .into_iter()
        .map(|input| {
            let mut sink = DiagnosticSink::new(DiagnosticProducerScope::new(
                PipelinePhase::Documentation,
                input.source_snapshot,
                "shared-links",
            ));
            sink.emit(DiagnosticDraft::new(input).unwrap()).unwrap();
            sink.into_batch()
        })
        .collect();
    BuildDiagnosticIndex::from_batches(snapshot, batches).unwrap()
}

#[test]
fn link_reference_drafts_preserve_explicit_outcomes_routes_and_discontiguous_mapping() {
    use DiagnosticDetailValue::{List, String as Text};
    let snapshot = snapshot_id(42);
    let source = source_id(snapshot);
    let source_text = "::: Λ&\n::: gap\n::: tail\n::: last";
    let range = |marker: &str| {
        let start = source_text.find(marker).unwrap();
        SourceRange {
            source_id: source,
            start,
            end: start + marker.len(),
        }
    };
    let base = link_input(snapshot, "reference");
    let List(subject) = base.details.entries()["documentation.link.subject"].clone() else {
        unreachable!()
    };
    let candidate = List(vec![
        subject[0].clone(),
        subject[1].clone(),
        Text("origin-1".into()),
        Text("pkg.T".into()),
    ]);
    let other = List(vec![
        Text("cd".repeat(32)),
        subject[1].clone(),
        Text("origin-2".into()),
        Text("pkg.T".into()),
    ]);
    let endpoint = List(vec![subject[0].clone(), subject[1].clone()]);
    for (code, reason, form) in [
        ("W0412", "target_absent", "code"),
        ("W0412", "section_absent", "explicit"),
        ("W0412", "classification_unavailable", "see"),
        ("W0413", "multiple_targets", "explicit"),
        ("W0414", "imports_partial", "code"),
        ("W0414", "imports_unavailable", "see"),
    ] {
        for mapped in [false, true] {
            let mut input = base.clone();
            input.code = DiagnosticCode::from_str(code).unwrap();
            input.stable_detail_key = match code {
                "W0412" => "documentation.link.unresolved",
                "W0413" => "documentation.link.ambiguous",
                _ => "documentation.link.incomplete_input",
            }
            .into();
            let mut subject = subject.clone();
            subject[4] = List(vec![
                Text("0".into()),
                Text("3".into()),
                Text("8".into()),
                Text(form.into()),
            ]);
            let reference = List(vec![
                Text(" Λ&amp; ".into()),
                List(if reason == "classification_unavailable" {
                    vec![]
                } else {
                    vec![Text(" Λ& ".into())]
                }),
            ]);
            let candidates = if code == "W0412" {
                vec![]
            } else if code == "W0413" {
                vec![candidate.clone(), other.clone()]
            } else {
                vec![candidate.clone()]
            };
            let route = |kind: &str, position| {
                List(vec![
                    candidate.clone(),
                    Text(kind.into()),
                    position,
                    endpoint.clone(),
                ])
            };
            let routes = if candidates.is_empty() {
                vec![]
            } else {
                let mut routes = vec![
                    route("import", List(vec![Text("2".into())])),
                    route("import", List(vec![Text("12".into())])),
                    route("closure", List(vec![])),
                ];
                if code == "W0413" {
                    routes.insert(0, route("current", List(vec![])));
                }
                routes
            };
            let causes = List(vec![List(vec![Text(reason.into()), List(vec![])])]);
            input.details = DiagnosticDetails::from_entries([
                ("documentation.link.subject", List(subject)),
                ("documentation.link.reference", reference),
                (
                    "documentation.link.section_version",
                    List(vec![Text("2026-draft.1".into())]),
                ),
                ("documentation.link.candidates", List(candidates)),
                ("documentation.link.causes", causes),
                ("documentation.link.routes", List(routes)),
                ("documentation.link.citation_coverage", List(vec![])),
            ])
            .unwrap();
            if mapped {
                input.primary_location = DiagnosticPrimaryLocation::Span(
                    DiagnosticSpan::primary(range("Λ&"), None).unwrap(),
                );
                input.secondary_spans = vec![
                    DiagnosticSpan::secondary(range("tail"), None).unwrap(),
                    DiagnosticSpan::secondary(range("last"), None).unwrap(),
                ];
            }
            let mut distinct = input.clone();
            let List(mut shifted) =
                distinct.details.entries()["documentation.link.subject"].clone()
            else {
                unreachable!()
            };
            shifted[4] = List(vec![
                Text("0".into()),
                Text("4".into()),
                Text("9".into()),
                Text(form.into()),
            ]);
            distinct
                .details
                .insert("documentation.link.subject", List(shifted))
                .unwrap();
            let copies = vec![input.clone(), distinct, input.clone()];
            let index = link_index(snapshot, copies.clone());
            assert_eq!(index.len(), 2);
            assert_eq!(
                index.debug_snapshot(),
                link_index(snapshot, copies.into_iter().rev().collect()).debug_snapshot()
            );
            let record = index
                .records()
                .iter()
                .find(|r| r.details() == &input.details)
                .unwrap();
            assert_eq!(
                record.severity(),
                mizar_diagnostics::registry::DiagnosticSeverity::Warning
            );
            assert_eq!(record.details(), &input.details);
            assert_eq!(record.primary_location(), &input.primary_location);
            assert_eq!(record.secondary_spans(), input.secondary_spans);
            for key in input.details.entries().keys() {
                assert!(record.debug_snapshot().contains(key));
            }
        }
    }
}

#[test]
fn rejected_url_explicit_drafts_preserve_occurrences_locations_and_cli_warning() {
    use DiagnosticDetailValue::{List, String as Text};
    let snapshot = snapshot_id(44);
    let source = source_id(snapshot);
    let raw = "jav&#x61;script:one";
    let interpreted = "javascript:one";
    let source_text = "::: jav&#x61;script:one\n::: jav&#x61;\n::: gap\n::: script:one";
    let range = |start, len| SourceRange {
        source_id: source,
        start,
        end: start + len,
    };
    for kind in ["link", "image"] {
        for segments in 0..=2 {
            let mut input = link_input(snapshot, "reference");
            input.code = DiagnosticCode::from_str("W0415").unwrap();
            input.stable_detail_key = "documentation.render.rejected_url".into();
            input.message = "Documentation URL is rejected by output policy".into();
            if let DiagnosticPrimaryLocation::Documentation { block_ordinal, .. } =
                &mut input.primary_location
            {
                *block_ordinal = 12;
            }
            let occurrence = vec![
                Text("ab".repeat(32)),
                List(vec![
                    Text("pkg".into()),
                    List(vec![]),
                    List(vec![]),
                    Text("doc.miz".into()),
                    Text("2026".into()),
                ]),
                List(vec![
                    Text("item".into()),
                    Text("origin-1".into()),
                    Text("pkg.T".into()),
                ]),
                Text("12".into()),
                Text(input.stable_detail_key.clone()),
                List(vec![Text("3".into()), Text((3 + raw.len()).to_string())]),
            ];
            let url = vec![
                Text(kind.into()),
                Text("7".into()),
                Text(raw.into()),
                Text(interpreted.into()),
            ];
            input.details = DiagnosticDetails::from_entries([
                ("documentation.occurrence", List(occurrence.clone())),
                ("documentation.render.url", List(url.clone())),
            ])
            .unwrap();
            if segments > 0 {
                let primary = if segments == 1 {
                    range(source_text.find(raw).unwrap(), raw.len())
                } else {
                    range(source_text.rfind("jav&#x61;").unwrap(), "jav&#x61;".len())
                };
                input.primary_location = DiagnosticPrimaryLocation::Span(
                    DiagnosticSpan::primary(primary, None).unwrap(),
                );
                if segments == 2 {
                    input.secondary_spans = vec![
                        DiagnosticSpan::secondary(
                            range(source_text.rfind("script:one").unwrap(), "script:one".len()),
                            None,
                        )
                        .unwrap(),
                    ];
                }
            }
            let mut ordinal = input.clone();
            let mut next_url = url;
            next_url[1] = Text("8".into());
            ordinal
                .details
                .insert("documentation.render.url", List(next_url))
                .unwrap();
            let mut offsets = input.clone();
            let mut next_occurrence = occurrence.clone();
            next_occurrence[5] = List(vec![Text("4".into()), Text((4 + raw.len()).to_string())]);
            offsets
                .details
                .insert("documentation.occurrence", List(next_occurrence))
                .unwrap();
            let mut duplicate = input.clone();
            duplicate.message = "another presentation of the same occurrence".into();
            let mut copies = vec![input.clone(), ordinal, offsets, duplicate];
            if segments == 1 {
                for (part, member) in [
                    (0, None),
                    (1, Some(0)),
                    (1, Some(1)),
                    (1, Some(2)),
                    (1, Some(3)),
                    (1, Some(4)),
                    (2, Some(1)),
                    (2, Some(2)),
                    (3, None),
                ] {
                    let mut distinct = input.clone();
                    let mut identity = occurrence.clone();
                    let replacement = match (part, member) {
                        (0, _) => Text("cd".repeat(32)),
                        (1, Some(1 | 2)) => List(vec![Text(String::new())]),
                        (3, _) => Text("13".into()),
                        _ => Text("other".into()),
                    };
                    if let Some(member) = member {
                        let List(tuple) = &mut identity[part] else {
                            unreachable!()
                        };
                        tuple[member] = replacement;
                    } else {
                        identity[part] = replacement;
                    }
                    distinct
                        .details
                        .insert("documentation.occurrence", List(identity))
                        .unwrap();
                    copies.push(distinct);
                }
            }
            let index = link_index(snapshot, copies.clone());
            assert_eq!(index.len(), if segments == 1 { 12 } else { 3 });
            assert_eq!(
                index.debug_snapshot(),
                link_index(snapshot, copies.into_iter().rev().collect()).debug_snapshot()
            );
            let record = index
                .records()
                .iter()
                .find(|record| record.details() == &input.details)
                .unwrap();
            assert_eq!(record.code(), input.code);
            assert_eq!(
                record.severity(),
                mizar_diagnostics::registry::DiagnosticSeverity::Warning
            );
            assert_eq!(record.phase(), PipelinePhase::Documentation);
            assert_eq!(record.category(), FailureCategory::DocumentationWarning);
            assert_eq!(record.primary_location(), &input.primary_location);
            assert_eq!(record.secondary_spans(), input.secondary_spans);
            assert_eq!(
                record.freshness(),
                &DiagnosticFreshness::Current {
                    source_snapshot: snapshot
                }
            );
            let rendered = if segments == 0 {
                render_diagnostics(DiagnosticRenderInput::new(
                    std::slice::from_ref(record),
                    &PanicSourceContext,
                    RenderOptions::plain(),
                ))
            } else {
                let context = TestSourceContext::new().with_source(
                    source,
                    "doc.miz",
                    "url-source",
                    source_text,
                );
                render_diagnostics(DiagnosticRenderInput::new(
                    std::slice::from_ref(record),
                    &context,
                    RenderOptions::plain(),
                ))
            };
            assert!(rendered.starts_with("warning[W0415]: Documentation URL is rejected by output policy (documentation.render.rejected_url)\n"));
            if segments == 0 {
                assert!(rendered.ends_with(&format!("  --> documentation publication={} package=\"pkg\" version=None lockfile=None module=\"doc.miz\" edition=\"2026\" block=12", "ab".repeat(32))));
            }
            let obsolete = link_index(snapshot_id(45), vec![input]);
            assert!(obsolete.records().is_empty());
            assert_eq!(obsolete.obsolete_drafts().len(), 1);
        }
    }
}

#[test]
fn link_scope_coverage_location_validation_and_source_free_cli_are_independent() {
    use DiagnosticDetailValue::{List, String as Text};
    let snapshot = snapshot_id(43);
    for scope in ["module_page", "section_index", "used_by"] {
        let mut input = link_input(snapshot, scope);
        let List(subject) = input.details.entries()["documentation.link.subject"].clone() else {
            unreachable!()
        };
        let affected = List(subject[..3].to_vec());
        let states: &[(&str, &str)] = match scope {
            "module_page" => &[("unavailable", "complete")],
            "section_index" => &[("complete", "unavailable"), ("complete", "incomplete")],
            _ => &[
                ("unavailable", "complete"),
                ("incomplete", "unavailable"),
                ("complete", "incomplete"),
                ("complete", "complete"),
            ],
        };
        for &(closure, coverage) in states {
            let reasons: Vec<&str> = match scope {
                "module_page" => vec![
                    "anchor_collision",
                    "label_unavailable",
                    "page_collision",
                    "page_unavailable",
                ],
                "section_index" => vec![if coverage == "incomplete" {
                    "sections_partial"
                } else {
                    "sections_unavailable"
                }],
                _ => [
                    Some("page_unavailable"),
                    (coverage == "incomplete").then_some("citation_endpoint_unmapped"),
                    match closure {
                        "unavailable" => Some("closure_unavailable"),
                        "incomplete" => Some("closure_partial"),
                        _ => None,
                    },
                    match coverage {
                        "unavailable" => Some("citations_unavailable"),
                        "incomplete" => Some("citations_partial"),
                        _ => None,
                    },
                ]
                .into_iter()
                .flatten()
                .collect(),
            };
            let mut reasons = reasons;
            reasons.sort();
            input
                .details
                .insert(
                    "documentation.link.causes",
                    List(
                        reasons
                            .iter()
                            .map(|reason| List(vec![Text((*reason).into()), affected.clone()]))
                            .collect(),
                    ),
                )
                .unwrap();
            if scope == "used_by" {
                let inventory = if closure == "incomplete" || coverage == "incomplete" {
                    vec![List(subject[..2].to_vec())]
                } else {
                    vec![]
                };
                let examined = if coverage == "incomplete" {
                    vec![List(vec![
                        subject[0].clone(),
                        subject[1].clone(),
                        Text("origin-1".into()),
                        Text("pkg.T".into()),
                        List(vec![Text("body".into()), Text("proof".into())]),
                    ])]
                } else {
                    vec![]
                };
                input
                    .details
                    .insert(
                        "documentation.link.citation_coverage",
                        List(vec![
                            Text(closure.into()),
                            List(inventory),
                            Text(coverage.into()),
                            List(examined),
                        ]),
                    )
                    .unwrap();
            }
            let index = link_index(snapshot, vec![input.clone(), input.clone()]);
            assert_eq!(index.len(), 1);
            assert_eq!(index.records()[0].details(), &input.details);
            assert!(
                index.records()[0]
                    .debug_snapshot()
                    .contains("documentation_scope(")
            );
            let rendered = render_diagnostics(DiagnosticRenderInput::new(
                index.records(),
                &PanicSourceContext,
                RenderOptions::plain(),
            ));
            let suffix = if scope == "used_by" {
                " item_origin=\"origin-1\" item_fqn=\"pkg.T\""
            } else {
                ""
            };
            assert_eq!(
                rendered,
                format!(
                    "warning[W0414]: link input (documentation.link.incomplete_input)\n  --> documentation publication={} package=\"pkg\" version=None lockfile=None module=\"doc.miz\" edition=\"2026\" scope={scope:?}{suffix}",
                    "ab".repeat(32)
                )
            );
        }
    }
    let input = link_input(snapshot, "module_page");
    for field in 0..6 {
        let mut invalid = input.clone();
        if let DiagnosticPrimaryLocation::DocumentationScope {
            package_id,
            module_path,
            language_edition,
            ..
        } = &mut invalid.primary_location
        {
            match field {
                0 => *package_id = " ".into(),
                1 => *module_path = String::new(),
                2 => *language_edition = "\t".into(),
                3 => invalid.phase = PipelinePhase::Parser,
                4 => invalid.category = FailureCategory::ParseError,
                _ => {
                    invalid.code = DiagnosticCode::from_str("E0001").unwrap();
                    invalid.phase = PipelinePhase::Parser;
                    invalid.category = FailureCategory::ParseError;
                }
            }
        }
        assert!(DiagnosticDraft::new(invalid).is_err());
    }
}

#[test]
fn fix_rendering_projects_edit_and_command_metadata_without_application_payloads() {
    let snapshot = snapshot_id(7);
    let source_id = source_id(snapshot);
    let context = TestSourceContext::new().with_source(source_id, "fix.miz", "Src(7)", "abc\n");
    let record = record(
        RecordFixture::new(snapshot, source_id, 0, 1, "fixes")
            .fixes(vec![command_fix(), text_edit_fix(source_id)]),
    );
    let rendered = render_diagnostics(DiagnosticRenderInput::new(
        std::slice::from_ref(&record),
        &context,
        RenderOptions::plain(),
    ));

    assert_eq!(
        rendered,
        concat!(
            "error[E0001]: fixes (syntax.unexpected_token)\n",
            "  --> fix.miz:1:1\n",
            "   |\n",
            " 1 | abc\n",
            "   | ^ fixes\n",
            "   = help: fix suggestion `render.command`: open command (command_only command `render.open_command`)\n",
            "   = help: fix suggestion `render.insert`: insert token (machine_applicable local_text_edit, 1 edit)",
        )
    );
    assert!(!rendered.contains("WorkspaceEdit"));
    assert!(!rendered.contains("replacement"));
    assert!(!rendered.contains("expected_text"));
}

#[test]
fn explanation_rendering_projects_only_handle_id_and_bounded_preview_without_resolution_payloads() {
    let snapshot = snapshot_id(8);
    let source_id = source_id(snapshot);
    let context =
        TestSourceContext::new().with_source(source_id, "explain.miz", "Src(8)", "abc\ndef\n");
    let artifact_record = record(
        RecordFixture::new(snapshot, source_id, 0, 1, "artifact explanation")
            .explanation(backing_explanation(snapshot, true)),
    );
    let query_record = record(
        RecordFixture::new(snapshot, source_id, 4, 7, "query explanation")
            .explanation(query_explanation()),
    );
    let rendered = render_diagnostics(DiagnosticRenderInput::new(
        &[artifact_record, query_record],
        &context,
        RenderOptions::plain(),
    ));

    assert_eq!(
        rendered,
        concat!(
            "error[E0001]: artifact explanation (syntax.unexpected_token)\n",
            "  --> explain.miz:1:1\n",
            "   |\n",
            " 1 | abc\n",
            "   | ^ artifact explanation\n",
            "   = explain: `render.artifact_explain`: bounded preview\n",
            "\n",
            "error[E0001]: query explanation (syntax.unexpected_token)\n",
            "  --> explain.miz:2:1\n",
            "   |\n",
            " 2 | def\n",
            "   | ^^^ query explanation\n",
            "   = explain: `render.query_explain`",
        )
    );
    for forbidden in [
        "explain/proof.json",
        "0101010101010101010101010101010101010101010101010101010101010101",
        "0202020202020202020202020202020202020202020202020202020202020202",
        "required_snapshot",
        "artifact payload",
        "WorkspaceEdit",
        "jsonrpc",
        "mizar/explain",
        "replacement",
        "expected_text",
    ] {
        assert!(
            !rendered.contains(forbidden),
            "rendered explanation must not leak `{forbidden}`"
        );
    }
}

#[test]
fn missing_source_context_uses_deterministic_fallback() {
    let snapshot = snapshot_id(2);
    let source_id = source_id(snapshot);
    let context = TestSourceContext::new().with_source_key(source_id, "source-key");
    let record = record(RecordFixture::new(snapshot, source_id, 1, 4, "fallback"));

    assert_eq!(
        render_diagnostics(DiagnosticRenderInput::new(
            std::slice::from_ref(&record),
            &context,
            RenderOptions::plain(),
        )),
        "error[E0001]: fallback (syntax.unexpected_token)\n  --> source source-key:1..4 \
         (primary, current): fallback"
    );
}

#[test]
fn multiple_diagnostics_use_one_blank_line_separator_and_input_order() {
    let snapshot = snapshot_id(3);
    let source_id = source_id(snapshot);
    let context = TestSourceContext::new().with_source(source_id, "a.miz", "Src(3)", "abc\ndef\n");
    let first = record(RecordFixture::new(snapshot, source_id, 0, 1, "first"));
    let second = record(RecordFixture::new(snapshot, source_id, 4, 7, "second"));

    assert_eq!(
        render_diagnostics(DiagnosticRenderInput::new(
            &[first, second],
            &context,
            RenderOptions::plain(),
        )),
        concat!(
            "error[E0001]: first (syntax.unexpected_token)\n",
            "  --> a.miz:1:1\n",
            "   |\n",
            " 1 | abc\n",
            "   | ^ first\n",
            "\n",
            "error[E0001]: second (syntax.unexpected_token)\n",
            "  --> a.miz:2:1\n",
            "   |\n",
            " 2 | def\n",
            "   | ^^^ second",
        )
    );
}

#[test]
fn unicode_scalar_columns_and_multiline_context_are_byte_stable() {
    let snapshot = snapshot_id(4);
    let source_id = source_id(snapshot);
    let unicode_context =
        TestSourceContext::new().with_source(source_id, "unicode.miz", "Src(4)", "αβγ\n");
    let unicode_record = record(RecordFixture::new(
        snapshot,
        source_id,
        2,
        6,
        "unicode span",
    ));

    assert_eq!(
        render_diagnostics(DiagnosticRenderInput::new(
            std::slice::from_ref(&unicode_record),
            &unicode_context,
            RenderOptions::plain(),
        )),
        concat!(
            "error[E0001]: unicode span (syntax.unexpected_token)\n",
            "  --> unicode.miz:1:2\n",
            "   |\n",
            " 1 | αβγ\n",
            "   |  ^^ unicode span",
        )
    );

    let multiline_context = TestSourceContext::new().with_source(
        source_id,
        "multi.miz",
        "Src(4)",
        "one\ntwo\nthree\nfour\n",
    );
    let multiline_record = record(RecordFixture::new(
        snapshot,
        source_id,
        0,
        13,
        "multiline span",
    ));
    assert_eq!(
        render_diagnostics(DiagnosticRenderInput::new(
            std::slice::from_ref(&multiline_record),
            &multiline_context,
            RenderOptions::plain().with_multiline_context_limit(4),
        )),
        concat!(
            "error[E0001]: multiline span (syntax.unexpected_token)\n",
            "  --> multi.miz:1:1\n",
            "   |\n",
            " 1 | one\n",
            "   | ^^^ multiline span\n",
            " 2 | two\n",
            "   | ^^^\n",
            " 3 | three\n",
            "   | ^^^^^",
        )
    );
    assert_eq!(
        render_diagnostics(DiagnosticRenderInput::new(
            std::slice::from_ref(&multiline_record),
            &multiline_context,
            RenderOptions::plain(),
        )),
        concat!(
            "error[E0001]: multiline span (syntax.unexpected_token)\n",
            "  --> multi.miz:1:1\n",
            "   |\n",
            " 1 | one\n",
            "   | ^^^ multiline span\n",
            "   | ...\n",
            " 3 | three\n",
            "   | ^^^^^",
        )
    );
}

#[test]
fn source_key_missing_text_cause_related_and_zero_width_rendering_are_stable() {
    let snapshot = snapshot_id(5);
    let source_id = source_id(snapshot);
    let context =
        TestSourceContext::new().with_source_without_path(source_id, "Src(5)", "abc\ndef\n");
    let path_fallback_record = record(
        RecordFixture::new(snapshot, source_id, 0, 1, "path fallback").notes(vec![
            DiagnosticNote::new(DiagnosticNoteKind::Cause, "because of context", None),
            DiagnosticNote::new(DiagnosticNoteKind::Related, "see also", None),
        ]),
    );
    assert_eq!(
        render_diagnostics(DiagnosticRenderInput::new(
            std::slice::from_ref(&path_fallback_record),
            &context,
            RenderOptions::plain(),
        )),
        concat!(
            "error[E0001]: path fallback (syntax.unexpected_token)\n",
            "  --> Src(5):1:1\n",
            "   |\n",
            " 1 | abc\n",
            "   | ^ path fallback\n",
            "   = cause: because of context\n",
            "   = related: see also",
        )
    );

    let secondary = DiagnosticSpan::secondary(
        SourceRange {
            source_id,
            start: 2,
            end: 4,
        },
        Some("secondary fallback".to_owned()),
    )
    .expect("valid secondary span");
    let note_span = DiagnosticSpan::secondary(
        SourceRange {
            source_id,
            start: 4,
            end: 5,
        },
        Some("note fallback".to_owned()),
    )
    .expect("valid note span");
    let missing_text_context = MissingTextContext { source_id };
    let missing_text_record = record(
        RecordFixture::new(snapshot, source_id, 0, 1, "missing text")
            .secondary_spans(vec![secondary])
            .notes(vec![DiagnosticNote::new(
                DiagnosticNoteKind::Note,
                "anchored note",
                Some(note_span),
            )]),
    );
    assert_eq!(
        render_diagnostics(DiagnosticRenderInput::new(
            std::slice::from_ref(&missing_text_record),
            &missing_text_context,
            RenderOptions::plain(),
        )),
        concat!(
            "error[E0001]: missing text (syntax.unexpected_token)\n",
            "  --> source MissingText:0..1 (primary, current): missing text\n",
            "  --> source MissingText:2..4 (secondary, current): secondary fallback\n",
            "  --> source MissingText:4..5 (secondary, current): note fallback\n",
            "   = note: anchored note",
        )
    );

    let zero_width = DiagnosticSpan::new(
        SourceRange {
            source_id,
            start: 1,
            end: 1,
        },
        DiagnosticSpanRole::Primary,
        Some("insert here".to_owned()),
        SpanFreshness::Current,
        Some(ZeroWidthSpanIntent::InsertionPoint),
    )
    .expect("valid zero-width primary span");
    let zero_width_record =
        record(RecordFixture::new(snapshot, source_id, 0, 1, "unused").primary_span(zero_width));
    assert_eq!(
        render_diagnostics(DiagnosticRenderInput::new(
            std::slice::from_ref(&zero_width_record),
            &context,
            RenderOptions::plain(),
        )),
        concat!(
            "error[E0001]: unused (syntax.unexpected_token)\n",
            "  --> Src(5):1:2\n",
            "   |\n",
            " 1 | abc\n",
            "   |  ^ insert here",
        )
    );
    let point = DiagnosticSpan::from_anchor(
        SourceAnchor::Point {
            source_id,
            offset: 1,
        },
        DiagnosticSpanRole::Primary,
        Some("insert here".to_owned()),
        SpanFreshness::Current,
        None,
    )
    .expect("point with unspecified intent");
    let point_record =
        record(RecordFixture::new(snapshot, source_id, 0, 1, "unused").primary_span(point));
    assert_eq!(
        render_diagnostics(DiagnosticRenderInput::new(
            std::slice::from_ref(&point_record),
            &context,
            RenderOptions::plain(),
        )),
        render_diagnostics(DiagnosticRenderInput::new(
            std::slice::from_ref(&zero_width_record),
            &context,
            RenderOptions::plain(),
        ))
    );
}

#[test]
fn ansi_style_wraps_header_only_and_uses_severity_colors() {
    let snapshot = snapshot_id(6);
    let source_id = source_id(snapshot);
    let context = TestSourceContext::new().with_source(source_id, "a.miz", "Src(6)", "abc\n");
    let error_record = record(RecordFixture::new(
        snapshot,
        source_id,
        0,
        1,
        "error styled",
    ));
    let warning_record = record(
        RecordFixture::new(snapshot, source_id, 0, 1, "warning styled")
            .code("W0001")
            .stable_detail_key("warn.unused_variable")
            .category(FailureCategory::CompatibilityWarning),
    );

    assert!(matches!(RenderOptions::ansi().style(), RenderStyle::Ansi));
    assert_eq!(
        render_diagnostics(DiagnosticRenderInput::new(
            std::slice::from_ref(&error_record),
            &context,
            RenderOptions::ansi(),
        )),
        concat!(
            "\u{1b}[31merror[E0001]: error styled (syntax.unexpected_token)\u{1b}[0m\n",
            "  --> a.miz:1:1\n",
            "   |\n",
            " 1 | abc\n",
            "   | ^ error styled",
        )
    );
    assert_eq!(
        render_diagnostics(DiagnosticRenderInput::new(
            std::slice::from_ref(&warning_record),
            &context,
            RenderOptions::ansi(),
        )),
        concat!(
            "\u{1b}[33mwarning[W0001]: warning styled (warn.unused_variable)\u{1b}[0m\n",
            "  --> a.miz:1:1\n",
            "   |\n",
            " 1 | abc\n",
            "   | ^ warning styled",
        )
    );
}

#[derive(Clone, Copy)]
struct MissingTextContext {
    source_id: SourceId,
}

impl DiagnosticSourceContext for MissingTextContext {
    fn path_for(&self, _source: SourceId) -> Option<&str> {
        None
    }

    fn source_key_for(&self, _source: SourceId) -> String {
        "MissingText".to_owned()
    }

    fn line_text(&self, _source: SourceId, _line: u32) -> Option<&str> {
        None
    }

    fn line_column(&self, range: SourceRange) -> Option<LineColumnRange> {
        if range.source_id != self.source_id {
            return None;
        }
        Some(LineColumnRange {
            start: LineColumn { line: 1, column: 1 },
            end: LineColumn { line: 1, column: 2 },
        })
    }
}

#[derive(Default)]
struct TestSourceContext {
    sources: HashMap<SourceId, TestSource>,
}

struct PanicSourceContext;

impl DiagnosticSourceContext for PanicSourceContext {
    fn path_for(&self, _: SourceId) -> Option<&str> {
        panic!("source-load rendering must not query source paths")
    }

    fn source_key_for(&self, _: SourceId) -> String {
        panic!("source-load rendering must not query source keys")
    }

    fn line_text(&self, _: SourceId, _: u32) -> Option<&str> {
        panic!("source-load rendering must not query source text")
    }

    fn line_column(&self, _: SourceRange) -> Option<LineColumnRange> {
        panic!("source-load rendering must not query source coordinates")
    }
}

impl TestSourceContext {
    fn new() -> Self {
        Self::default()
    }

    fn with_source(
        mut self,
        source_id: SourceId,
        path: &'static str,
        source_key: &'static str,
        text: &'static str,
    ) -> Self {
        self.sources.insert(
            source_id,
            TestSource {
                path: Some(path),
                source_key,
                text: Some(text),
                line_starts: line_starts(text),
            },
        );
        self
    }

    fn with_source_without_path(
        mut self,
        source_id: SourceId,
        source_key: &'static str,
        text: &'static str,
    ) -> Self {
        self.sources.insert(
            source_id,
            TestSource {
                path: None,
                source_key,
                text: Some(text),
                line_starts: line_starts(text),
            },
        );
        self
    }

    fn with_source_key(mut self, source_id: SourceId, source_key: &'static str) -> Self {
        self.sources.insert(
            source_id,
            TestSource {
                path: None,
                source_key,
                text: None,
                line_starts: vec![],
            },
        );
        self
    }
}

impl DiagnosticSourceContext for TestSourceContext {
    fn path_for(&self, source: SourceId) -> Option<&str> {
        self.sources.get(&source).and_then(|source| source.path)
    }

    fn source_key_for(&self, source: SourceId) -> String {
        self.sources.get(&source).map_or_else(
            || format!("{source:?}"),
            |source| source.source_key.to_owned(),
        )
    }

    fn line_text(&self, source: SourceId, line: u32) -> Option<&str> {
        let source = self.sources.get(&source)?;
        let text = source.text?;
        let line_index = usize::try_from(line.checked_sub(1)?).ok()?;
        let start = *source.line_starts.get(line_index)?;
        let end = source
            .line_starts
            .get(line_index + 1)
            .copied()
            .unwrap_or(text.len());
        Some(text[start..end].trim_end_matches('\n'))
    }

    fn line_column(&self, range: SourceRange) -> Option<LineColumnRange> {
        let source = self.sources.get(&range.source_id)?;
        let text = source.text?;
        if range.end > text.len() || range.start > range.end {
            return None;
        }
        Some(LineColumnRange {
            start: line_column(text, &source.line_starts, range.start)?,
            end: line_column(text, &source.line_starts, range.end)?,
        })
    }
}

struct TestSource {
    path: Option<&'static str>,
    source_key: &'static str,
    text: Option<&'static str>,
    line_starts: Vec<usize>,
}

struct RecordFixture {
    snapshot: BuildSnapshotId,
    source_id: SourceId,
    start: usize,
    end: usize,
    message: &'static str,
    code: &'static str,
    category: FailureCategory,
    stable_detail_key: &'static str,
    primary_span: Option<DiagnosticSpan>,
    secondary_spans: Vec<DiagnosticSpan>,
    notes: Vec<DiagnosticNote>,
    fixes: Vec<FixSuggestion>,
    explanation: Option<ExplanationHandle>,
}

impl RecordFixture {
    fn new(
        snapshot: BuildSnapshotId,
        source_id: SourceId,
        start: usize,
        end: usize,
        message: &'static str,
    ) -> Self {
        Self {
            snapshot,
            source_id,
            start,
            end,
            message,
            code: "E0001",
            category: FailureCategory::ParseError,
            stable_detail_key: "syntax.unexpected_token",
            primary_span: None,
            secondary_spans: Vec::new(),
            notes: Vec::new(),
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

    fn primary_span(mut self, primary_span: DiagnosticSpan) -> Self {
        self.primary_span = Some(primary_span);
        self
    }

    fn secondary_spans(mut self, secondary_spans: Vec<DiagnosticSpan>) -> Self {
        self.secondary_spans = secondary_spans;
        self
    }

    fn notes(mut self, notes: Vec<DiagnosticNote>) -> Self {
        self.notes = notes;
        self
    }

    fn fixes(mut self, fixes: Vec<FixSuggestion>) -> Self {
        self.fixes = fixes;
        self
    }

    fn explanation(mut self, explanation: ExplanationHandle) -> Self {
        self.explanation = Some(explanation);
        self
    }
}

fn record(fixture: RecordFixture) -> mizar_diagnostics::failure_record::DiagnosticRecord {
    let draft = DiagnosticDraft::new(DiagnosticDraftInput {
        source_snapshot: fixture.snapshot,
        code: DiagnosticCode::from_str(fixture.code).expect("allocated code"),
        phase: PipelinePhase::Parser,
        category: fixture.category,
        stable_detail_key: fixture.stable_detail_key.to_owned(),
        message: fixture.message.to_owned(),
        primary_location: DiagnosticPrimaryLocation::Span(fixture.primary_span.unwrap_or_else(
            || {
                DiagnosticSpan::primary(
                    SourceRange {
                        source_id: fixture.source_id,
                        start: fixture.start,
                        end: fixture.end,
                    },
                    None,
                )
                .expect("valid primary span")
            },
        )),
        secondary_spans: fixture.secondary_spans,
        notes: fixture.notes,
        details: DiagnosticDetails::from_entries([(
            "render.case",
            DiagnosticDetailValue::Integer(1),
        )])
        .expect("valid details"),
        fixes: fixture.fixes,
        explanation: fixture.explanation,
    })
    .expect("valid draft");
    mizar_diagnostics::failure_record::DiagnosticRecord::from_draft(
        draft,
        DiagnosticHandle::new(fixture.snapshot, DiagnosticId::new(0)),
        DiagnosticFreshness::Current {
            source_snapshot: fixture.snapshot,
        },
        vec![],
    )
    .expect("valid record")
}

fn normalized_path(path: &str) -> NormalizedPath {
    static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let root = std::env::temp_dir().join(format!(
        "mizar-diagnostics-source-load-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    ));
    fs::create_dir(&root).expect("create isolated fixture root");
    let full_path = root.join(path);
    fs::create_dir_all(full_path.parent().expect("source path parent")).expect("create source dir");
    fs::write(&full_path, b"source fixture\n").expect("write source fixture");
    let normalized = normalize_source_path(&root, &full_path).expect("normalize source path");
    fs::remove_dir_all(&root).expect("remove fixture root");
    normalized
}

fn informational_fix(identity: &str, title: &str) -> FixSuggestion {
    FixSuggestion::informational(
        FixSuggestionId::new(identity).expect("valid fix identity"),
        title,
    )
    .expect("valid informational fix")
}

fn text_edit_fix(source_id: SourceId) -> FixSuggestion {
    FixSuggestion::local_text_edit(
        FixSuggestionId::new("render.insert").expect("valid fix identity"),
        "insert token",
        FixApplicability::MachineApplicable,
        vec![
            FixEdit::new(
                SourceRange {
                    source_id,
                    start: 1,
                    end: 1,
                },
                "x",
                Some(String::new()),
            )
            .expect("valid edit"),
        ],
    )
    .expect("valid text edit fix")
}

fn command_fix() -> FixSuggestion {
    FixSuggestion::new(FixSuggestionInput {
        id: FixSuggestionId::new("render.command").expect("valid fix identity"),
        producer_key: None,
        title: "open command".to_owned(),
        applicability: FixApplicability::MaybeIncorrect,
        safety: FixSafety::CommandOnly,
        edits: Vec::new(),
        command: Some(FixCommandRef::new("render.open_command").expect("valid command")),
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

fn backing_explanation(snapshot: BuildSnapshotId, with_preview: bool) -> ExplanationHandle {
    ExplanationHandle::new(ExplanationHandleInput {
        id: ExplanationHandleId::new("render.artifact_explain").expect("valid explanation id"),
        kind: ExplanationKind::ProofFailure,
        subject: ExplanationSubject::Diagnostic {
            code: DiagnosticCode::from_str("E0001").expect("allocated code"),
            stable_detail_key: "syntax.unexpected_token".to_owned(),
        },
        source: ExplanationSourceRef::Artifact {
            path: "explain/proof.json".to_owned(),
            content_hash: Hash::from_bytes([1; Hash::BYTE_LEN]),
        },
        required_snapshot: Some(snapshot),
        required_artifact_hash: Some(Hash::from_bytes([1; Hash::BYTE_LEN])),
        summary_hash: Some(Hash::from_bytes([2; Hash::BYTE_LEN])),
        preview: with_preview.then(|| {
            ExplanationPreview::new(ExplanationPreviewFormat::PlainText, "bounded preview")
        }),
    })
    .expect("valid backing explanation handle")
}

fn query_explanation() -> ExplanationHandle {
    ExplanationHandle::new(ExplanationHandleInput {
        id: ExplanationHandleId::new("render.query_explain").expect("valid explanation id"),
        kind: ExplanationKind::OverloadResolution,
        subject: ExplanationSubject::Expression("expr.render".to_owned()),
        source: ExplanationSourceRef::QueryService {
            service_key: "render.explain_service".to_owned(),
            query_key: "render.query".to_owned(),
        },
        required_snapshot: None,
        required_artifact_hash: None,
        summary_hash: None,
        preview: None,
    })
    .expect("valid query explanation handle")
}

fn line_starts(text: &str) -> Vec<usize> {
    let mut starts = vec![0];
    for (index, byte) in text.bytes().enumerate() {
        if byte == b'\n' && index + 1 < text.len() {
            starts.push(index + 1);
        }
    }
    starts
}

fn line_column(text: &str, starts: &[usize], offset: usize) -> Option<LineColumn> {
    let line_index = match starts.binary_search(&offset) {
        Ok(index) => index,
        Err(0) => return None,
        Err(index) => index - 1,
    };
    let line_start = *starts.get(line_index)?;
    Some(LineColumn {
        line: u32::try_from(line_index + 1).ok()?,
        column: u32::try_from(text[line_start..offset].chars().count() + 1).ok()?,
    })
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
