//! Current full-snapshot diagnostic conversion and guarded delivery.

use std::{
    collections::BTreeMap,
    io::{self, Write},
    sync::Arc,
};

use mizar_diagnostics::{
    aggregator::BuildDiagnosticIndex,
    failure_record::{
        DiagnosticFreshness, DiagnosticPrimaryLocation, DiagnosticSpan, SpanFreshness,
    },
    registry::DiagnosticSeverity,
};
use mizar_driver::{driver::CompilerDriver, request::CapturedSnapshot};
use mizar_session::{
    GeneratedSpanAnchor, SessionIdAllocator, SnapshotRegistry, SourceAnchor, SourceOriginInput,
    SourceRange,
};
use serde_json::{Value, json};

use crate::{Server, server::LspSnapshotFreshness};

impl Server {
    /// Converts the current complete publication using its original retained native maps.
    /// Missing or stale publications return `None`; unrepresentable records fail the batch.
    /// Diagnostic data references the unchanged index, retaining anchor shape and provenance.
    pub fn snapshot_diagnostic_notifications<A: SessionIdAllocator>(
        &self,
        driver: &CompilerDriver,
        snapshots: &SnapshotRegistry<A>,
    ) -> Result<Option<Vec<Value>>, String> {
        let Some((selected, LspSnapshotFreshness::Current)) = self.lsp_snapshot(driver, snapshots)
        else {
            return Ok(None);
        };
        let notifications = convert_index(
            selected.publication().diagnostics(),
            selected.native_open_buffers().captured(),
        )?;
        Ok(self
            .lsp_snapshot(driver, snapshots)
            .filter(|(current, freshness)| {
                *freshness == LspSnapshotFreshness::Current && Arc::ptr_eq(current, &selected)
            })
            .map(|_| notifications))
    }

    /// Validates and serializes the whole batch before writing, then rechecks currentness
    /// before each frame. Returns the number delivered; writer failures propagate unchanged.
    pub fn publish_snapshot_diagnostics_to_writer<A: SessionIdAllocator>(
        &self,
        driver: &CompilerDriver,
        snapshots: &SnapshotRegistry<A>,
        output: &mut impl Write,
    ) -> io::Result<usize> {
        let Some((selected, LspSnapshotFreshness::Current)) = self.lsp_snapshot(driver, snapshots)
        else {
            return Ok(0);
        };
        let frames = convert_index(
            selected.publication().diagnostics(),
            selected.native_open_buffers().captured(),
        )
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?
        .iter()
        .map(serde_json::to_vec)
        .collect::<Result<Vec<_>, _>>()
        .map_err(io::Error::other)?;
        let mut count = 0;
        for body in frames {
            if !self
                .lsp_snapshot(driver, snapshots)
                .is_some_and(|(current, freshness)| {
                    freshness == LspSnapshotFreshness::Current && Arc::ptr_eq(&current, &selected)
                })
            {
                break;
            }
            write!(output, "Content-Length: {}\r\n\r\n", body.len())?;
            output.write_all(&body)?;
            output.flush()?;
            count += 1;
        }
        Ok(count)
    }
}

fn convert_index(
    index: &BuildDiagnosticIndex,
    captured: &CapturedSnapshot,
) -> Result<Vec<Value>, String> {
    if index.snapshot() != captured.snapshot().id {
        return Err("diagnostic index snapshot mismatch".into());
    }
    let sources = captured
        .native_sources()
        .filter(|sources| !sources.is_empty())
        .ok_or("diagnostic publication has no native sources")?;
    let mut by_source = std::collections::HashMap::new();
    let mut by_uri = BTreeMap::new();
    for (_, loaded, input) in sources {
        let SourceOriginInput::OpenBuffer {
            uri,
            actual_version,
            ..
        } = &input.origin
        else {
            return Err("diagnostic source is not an OpenBuffer".into());
        };
        let version = i32::try_from(*actual_version).map_err(|_| "diagnostic version overflow")?;
        if by_source
            .insert(loaded.source_id, (loaded, input, uri))
            .is_some()
            || by_uri.insert(uri, (version, Vec::new())).is_some()
        {
            return Err("duplicate diagnostic source or URI".into());
        }
        let eof = loaded.line_map.source().len();
        crate::lsp_range_from_open_buffer(
            &loaded.line_map,
            loaded.loading_map.as_ref(),
            &input.origin,
            SourceRange {
                source_id: loaded.source_id,
                start: eof,
                end: eof,
            },
        )
        .map_err(|error| format!("invalid diagnostic source map: {error:?}"))?;
    }
    let convert_span = |span: &DiagnosticSpan| -> Result<(&String, Value), String> {
        if span.freshness() != SpanFreshness::Current {
            return Err("diagnostic span is not current".into());
        }
        let range = match span.anchor() {
            SourceAnchor::Range(range) => *range,
            SourceAnchor::Point { source_id, offset } => SourceRange {
                source_id: *source_id,
                start: *offset,
                end: *offset,
            },
            SourceAnchor::Generated(origin) => match origin.anchor() {
                GeneratedSpanAnchor::Range(range) => range,
                GeneratedSpanAnchor::Point { source_id, offset } => SourceRange {
                    source_id,
                    start: offset,
                    end: offset,
                },
                _ => return Err("unsupported generated diagnostic anchor".into()),
            },
            _ => return Err("unsupported diagnostic anchor".into()),
        };
        let (loaded, input, uri) = by_source
            .get(&range.source_id)
            .ok_or("unknown diagnostic source")?;
        let range = crate::lsp_range_from_open_buffer(
            &loaded.line_map,
            loaded.loading_map.as_ref(),
            &input.origin,
            range,
        )
        .map_err(|error| format!("invalid diagnostic span: {error:?}"))?;
        Ok((
            uri,
            json!({"start": {"line": range.start.line, "character": range.start.character},
            "end": {"line": range.end.line, "character": range.end.character}}),
        ))
    };
    for record in index.records() {
        if record.handle().snapshot != index.snapshot()
            || record.freshness()
                != &(DiagnosticFreshness::Current {
                    source_snapshot: index.snapshot(),
                })
        {
            return Err("diagnostic record is not current for its index".into());
        }
        let DiagnosticPrimaryLocation::Span(primary) = record.primary_location() else {
            return Err("diagnostic primary location has no supported range encoding".into());
        };
        let (uri, range) = convert_span(primary)?;
        let severity = match record.severity() {
            DiagnosticSeverity::Error => 1,
            DiagnosticSeverity::Warning => 2,
            DiagnosticSeverity::Info => 3,
            _ => return Err("unsupported diagnostic severity".into()),
        };
        let mut related = Vec::new();
        for span in record.secondary_spans() {
            let (uri, range) = convert_span(span)?;
            related.push(json!({"location": {"uri": uri, "range": range}, "message": span.label().unwrap_or(record.message())}));
        }
        for span in record.notes().iter().filter_map(|note| note.span()) {
            convert_span(span)?;
        }
        let snapshot = record
            .handle()
            .snapshot
            .to_published_schema_string()
            .map_err(|error| format!("invalid diagnostic snapshot identity: {error:?}"))?;
        by_uri.get_mut(uri).ok_or("unknown primary diagnostic URI")?.1.push(json!({
            "range": range, "code": record.code().to_string(), "message": record.message(), "severity": severity,
            "relatedInformation": related, "data": {"snapshot": snapshot, "id": record.handle().id.get()}
        }));
    }
    Ok(by_uri
        .into_iter()
        .map(|(uri, (version, diagnostics))| {
            json!({
                "jsonrpc": "2.0", "method": "textDocument/publishDiagnostics",
                "params": {"uri": uri, "version": version, "diagnostics": diagnostics}
            })
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use mizar_diagnostics::{
        aggregator::{BuildDiagnosticIndex, DiagnosticAggregationInput},
        failure_record::{
            DiagnosticDetails, DiagnosticDraft, DiagnosticDraftInput, DiagnosticNote,
            DiagnosticNoteKind, DiagnosticPrimaryLocation, DiagnosticSpan, DiagnosticSpanRole,
            FailureCategory, PipelinePhase, SpanFreshness, ZeroWidthSpanIntent,
        },
        registry::{DiagnosticCode, DiagnosticSeverity},
        sink::{DiagnosticProducerScope, DiagnosticSink},
    };
    use mizar_driver::request::{
        BatchInvocation, BatchRequest, BuildLaneId, BuildProfile, BuildRequestDraft,
        BuildRequestGeneration, BuildRequestOrigin, BuildTargets, CapturedSnapshot,
        DependencyInputSet, SourceInputSet, VerifierConfigInput,
    };
    use mizar_session::{
        BuildSnapshotId, DiskSourceLoader, Edition, GeneratedSpanAnchor, GeneratedSpanOrigin, Hash,
        InMemorySessionIdAllocator, ModulePath, PackageId, SessionIdAllocator, SnapshotRegistry,
        SourceAnchor, SourceInput, SourceOriginInput, SourceRange, ToolchainInfo, WorkspaceRoot,
        normalize_source_path,
    };
    use std::{
        fs,
        path::PathBuf,
        sync::Arc,
        time::{SystemTime, UNIX_EPOCH},
    };

    struct CapturedFixture {
        _root: PathBuf,
        captured: CapturedSnapshot,
        source_ids: Vec<mizar_session::SourceId>,
        ids: InMemorySessionIdAllocator,
    }

    impl CapturedFixture {
        fn new(sources: &[(&str, &str, i64)]) -> Self {
            let nonce = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            let root = std::env::temp_dir().join(format!(
                "mizar-lsp-task9-converter-{}-{nonce}",
                std::process::id()
            ));
            let package_root = root.join("alpha");
            fs::create_dir_all(package_root.join("src")).unwrap();
            let loader = DiskSourceLoader::new(&package_root);
            let source_inputs = sources
                .iter()
                .map(|(uri, text, version)| {
                    let name = uri.rsplit('/').next().unwrap();
                    let path = PathBuf::from("src").join(name);
                    fs::write(package_root.join(&path), text).unwrap();
                    SourceInput {
                        package_id: PackageId::new("alpha"),
                        module_path: ModulePath::new(name.strip_suffix(".miz").unwrap()),
                        normalized_path: normalize_source_path(&package_root, &path).unwrap(),
                        edition: Edition::new("task9-test"),
                        origin: SourceOriginInput::OpenBuffer {
                            uri: format!("file://{}", package_root.join(&path).display()),
                            expected_version: *version,
                            actual_version: *version,
                            text: Arc::<str>::from(*text),
                        },
                    }
                })
                .map(|source| (loader.clone(), source))
                .collect();
            let ids = InMemorySessionIdAllocator::new();
            let registry = SnapshotRegistry::new();
            let draft = BuildRequestDraft {
                lane: BuildLaneId::new(1),
                origin: BuildRequestOrigin::Batch(BatchRequest {
                    invocation: BatchInvocation::default(),
                }),
                generation: BuildRequestGeneration::new(1),
                workspace_root: WorkspaceRoot::new(root.to_string_lossy().into_owned()),
                profile: BuildProfile::new("test"),
                targets: BuildTargets::default(),
                source_inputs: SourceInputSet::default(),
                dependency_inputs: DependencyInputSet::new(
                    vec![],
                    Hash::from_bytes([2; Hash::BYTE_LEN]),
                    ToolchainInfo::new("task9-test"),
                ),
                verifier_config: VerifierConfigInput::new(Hash::from_bytes([1; Hash::BYTE_LEN])),
            };
            let session = draft
                .capture_sources(&ids, &registry, source_inputs)
                .unwrap();
            let captured = session.captured().clone();
            let source_ids = captured
                .native_sources()
                .unwrap()
                .iter()
                .map(|(_, loaded, _)| loaded.source_id)
                .collect();
            Self {
                _root: root,
                captured,
                source_ids,
                ids,
            }
        }

        fn snapshot(&self) -> BuildSnapshotId {
            self.captured.snapshot().id
        }

        fn uri(&self, source: usize) -> &str {
            let SourceOriginInput::OpenBuffer { uri, .. } =
                &self.captured.native_sources().unwrap()[source].2.origin
            else {
                unreachable!()
            };
            uri
        }
    }

    impl Drop for CapturedFixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self._root);
        }
    }

    fn span(
        anchor: SourceAnchor,
        role: DiagnosticSpanRole,
        label: Option<&str>,
        zero_width: Option<ZeroWidthSpanIntent>,
    ) -> DiagnosticSpan {
        DiagnosticSpan::from_anchor(
            anchor,
            role,
            label.map(str::to_owned),
            SpanFreshness::Current,
            zero_width,
        )
        .unwrap()
    }

    fn syntax_draft(
        snapshot: BuildSnapshotId,
        code_number: u16,
        stable_key: &str,
        message: &str,
        primary: DiagnosticSpan,
        secondary_spans: Vec<DiagnosticSpan>,
        notes: Vec<DiagnosticNote>,
    ) -> DiagnosticDraft {
        DiagnosticDraft::new(DiagnosticDraftInput {
            source_snapshot: snapshot,
            code: DiagnosticCode::from_parts(DiagnosticSeverity::Error, code_number).unwrap(),
            phase: PipelinePhase::Parser,
            category: FailureCategory::ParseError,
            stable_detail_key: stable_key.to_owned(),
            message: message.to_owned(),
            primary_location: DiagnosticPrimaryLocation::Span(primary),
            secondary_spans,
            notes,
            details: DiagnosticDetails::new(),
            fixes: vec![],
            explanation: None,
        })
        .unwrap()
    }

    fn index(snapshot: BuildSnapshotId, drafts: Vec<DiagnosticDraft>) -> BuildDiagnosticIndex {
        let mut sink = DiagnosticSink::new(DiagnosticProducerScope::new(
            PipelinePhase::Parser,
            snapshot,
            "lsp-task9-test",
        ));
        for draft in drafts {
            sink.emit(draft).unwrap();
        }
        BuildDiagnosticIndex::aggregate(DiagnosticAggregationInput::new(
            snapshot,
            vec![sink.into_batch()],
        ))
        .unwrap()
    }

    fn ordinary_range(
        source_id: mizar_session::SourceId,
        start: usize,
        end: usize,
    ) -> SourceAnchor {
        SourceAnchor::Range(SourceRange {
            source_id,
            start,
            end,
        })
    }

    fn loaded_text(fixture: &CapturedFixture, source: usize) -> &str {
        &fixture.captured.native_sources().unwrap()[source].1.text
    }

    fn loaded_range(fixture: &CapturedFixture, source: usize, needle: &str) -> SourceRange {
        let text = loaded_text(fixture, source);
        let start = text.find(needle).unwrap();
        SourceRange {
            source_id: fixture.source_ids[source],
            start,
            end: start + needle.len(),
        }
    }

    #[test]
    fn converts_all_primary_anchor_shapes_and_preserves_the_typed_index() {
        let fixture = CapturedFixture::new(&[("file:///alpha.miz", "\u{feff}a\r\n😀e\u{301}z", 7)]);
        let sid = fixture.source_ids[0];
        // Use normalized loaded-source coordinates; the range mapper recovers raw
        // BOM/CRLF positions before producing editor UTF-16 coordinates.
        let loaded = loaded_text(&fixture, 0);
        let a = loaded_range(&fixture, 0, "a");
        let after_newline = loaded_range(&fixture, 0, "a\n").end;
        let emoji = loaded_range(&fixture, 0, "😀");
        let combining = loaded_range(&fixture, 0, "e\u{301}");
        let eof = loaded.len();
        let generated_range = GeneratedSpanOrigin::new(
            GeneratedSpanAnchor::Range(combining),
            "generated from the exact source expression",
        )
        .unwrap();
        let generated_point = GeneratedSpanOrigin::new(
            GeneratedSpanAnchor::Point {
                source_id: sid,
                offset: eof,
            },
            "generated insertion anchor",
        )
        .unwrap();
        let primary_anchors = [
            SourceAnchor::Range(a),
            SourceAnchor::Range(SourceRange {
                source_id: sid,
                start: after_newline,
                end: after_newline,
            }),
            SourceAnchor::Point {
                source_id: sid,
                offset: emoji.end,
            },
            SourceAnchor::Generated(generated_range),
            SourceAnchor::Generated(generated_point),
        ];
        let drafts = primary_anchors
            .into_iter()
            .enumerate()
            .map(|(ordinal, anchor)| {
                let zero_width =
                    matches!(ordinal, 1 | 2 | 4).then_some(ZeroWidthSpanIntent::InsertionPoint);
                syntax_draft(
                    fixture.snapshot(),
                    1,
                    &format!("anchor_{ordinal}"),
                    &format!("misleading message {}", 5 - ordinal),
                    span(anchor, DiagnosticSpanRole::Primary, None, zero_width),
                    vec![],
                    vec![],
                )
            })
            .collect();
        let index = index(fixture.snapshot(), drafts);
        let before = index.records().to_vec();
        let notifications = convert_index(&index, &fixture.captured).unwrap();
        assert_eq!(index.records(), before);
        assert_eq!(notifications.len(), 1);
        let diagnostics = notifications[0]["params"]["diagnostics"]
            .as_array()
            .unwrap();
        assert_eq!(diagnostics.len(), 5);
        for (original, diagnostic) in before.iter().zip(diagnostics) {
            assert_eq!(
                BuildSnapshotId::from_published_schema_str(
                    diagnostic["data"]["snapshot"].as_str().unwrap()
                )
                .unwrap(),
                fixture.snapshot()
            );
            let record = index
                .record_by_id(mizar_diagnostics::failure_record::DiagnosticId::new(
                    diagnostic["data"]["id"].as_u64().unwrap(),
                ))
                .unwrap();
            assert_eq!(record, original);
            let DiagnosticPrimaryLocation::Span(primary) = record.primary_location() else {
                panic!("typed test primary")
            };
            let expected = match primary.anchor() {
                SourceAnchor::Range(range) if *range == a => {
                    json!({"start":{"line":0,"character":1},"end":{"line":0,"character":2}})
                }
                SourceAnchor::Range(range)
                    if range.start == after_newline && range.end == after_newline =>
                {
                    json!({"start":{"line":1,"character":0},"end":{"line":1,"character":0}})
                }
                SourceAnchor::Point { offset, .. } if *offset == emoji.end => {
                    json!({"start":{"line":1,"character":2},"end":{"line":1,"character":2}})
                }
                SourceAnchor::Generated(origin)
                    if origin.reason() == "generated from the exact source expression" =>
                {
                    assert!(
                        matches!(origin.anchor(), GeneratedSpanAnchor::Range(range) if range == combining)
                    );
                    json!({"start":{"line":1,"character":2},"end":{"line":1,"character":4}})
                }
                SourceAnchor::Generated(origin)
                    if origin.reason() == "generated insertion anchor" =>
                {
                    assert!(
                        matches!(origin.anchor(), GeneratedSpanAnchor::Point { offset, .. } if offset == eof)
                    );
                    json!({"start":{"line":1,"character":5},"end":{"line":1,"character":5}})
                }
                other => panic!("unexpected anchor: {other:?}"),
            };
            assert_eq!(diagnostic["range"], expected);
        }
    }

    #[test]
    fn retains_secondary_anchor_order_multiplicity_and_note_text() {
        let fixture = CapturedFixture::new(&[
            ("file:///z.miz", "abcdef", 2),
            ("file:///a.miz", "uvwxyz", 5),
        ]);
        let first_range = loaded_range(&fixture, 0, "bc");
        let anchor_a = SourceAnchor::Range(first_range);
        let point_range = loaded_range(&fixture, 1, "uvwx");
        let anchor_b = SourceAnchor::Point {
            source_id: point_range.source_id,
            offset: point_range.end,
        };
        let secondary = vec![
            span(
                anchor_a.clone(),
                DiagnosticSpanRole::Secondary,
                Some("first"),
                None,
            ),
            span(
                anchor_b,
                DiagnosticSpanRole::Related,
                Some("cross source"),
                Some(ZeroWidthSpanIntent::Eof),
            ),
            span(
                anchor_a.clone(),
                DiagnosticSpanRole::Secondary,
                Some("duplicate"),
                None,
            ),
        ];
        let note_span = span(
            SourceAnchor::Generated(
                GeneratedSpanOrigin::new(
                    GeneratedSpanAnchor::Range(loaded_range(&fixture, 1, "vwx")),
                    "note generated reason",
                )
                .unwrap(),
            ),
            DiagnosticSpanRole::Related,
            None,
            None,
        );
        let draft = syntax_draft(
            fixture.snapshot(),
            1,
            "related_order",
            "same deliberately misleading message",
            span(
                SourceAnchor::Range(loaded_range(&fixture, 0, "a")),
                DiagnosticSpanRole::Primary,
                None,
                None,
            ),
            secondary,
            vec![
                DiagnosticNote::new(
                    DiagnosticNoteKind::Note,
                    "recovery note exact text",
                    Some(note_span),
                ),
                DiagnosticNote::new(DiagnosticNoteKind::Help, "", None),
            ],
        );
        let index = index(fixture.snapshot(), vec![draft]);
        let before = index.records().to_vec();
        let notifications = convert_index(&index, &fixture.captured).unwrap();
        assert_eq!(index.records(), before);
        let diagnostic = &notifications
            .iter()
            .find(|n| n["params"]["uri"] == fixture.uri(0))
            .unwrap()["params"]["diagnostics"][0];
        assert_eq!(
            BuildSnapshotId::from_published_schema_str(
                diagnostic["data"]["snapshot"].as_str().unwrap()
            )
            .unwrap(),
            fixture.snapshot()
        );
        let record = index
            .record_by_id(mizar_diagnostics::failure_record::DiagnosticId::new(
                diagnostic["data"]["id"].as_u64().unwrap(),
            ))
            .unwrap();
        assert_eq!(record, &before[0]);
        assert_eq!(
            record
                .secondary_spans()
                .iter()
                .map(|s| s.label())
                .collect::<Vec<_>>(),
            vec![Some("first"), Some("cross source"), Some("duplicate")]
        );
        assert_eq!(record.notes()[0].message(), "recovery note exact text");
        assert!(record.notes()[0].span().is_some());
        assert_eq!(record.notes()[1].message(), "");
        assert!(record.notes()[1].span().is_none());
        let related = diagnostic["relatedInformation"].as_array().unwrap();
        assert_eq!(related.len(), 3);
        assert_eq!(related[0]["location"]["uri"], fixture.uri(0));
        assert_eq!(related[1]["location"]["uri"], fixture.uri(1));
        assert_eq!(related[2]["location"]["uri"], fixture.uri(0));
        assert_eq!(
            related[0]["location"]["range"],
            json!({"start":{"line":0,"character":1},"end":{"line":0,"character":3}})
        );
        assert_eq!(
            related[1]["location"]["range"],
            json!({"start":{"line":0,"character":4},"end":{"line":0,"character":4}})
        );
        assert_eq!(
            related[2]["location"]["range"],
            related[0]["location"]["range"]
        );
        assert_eq!(
            related
                .iter()
                .map(|r| r["message"].as_str().unwrap())
                .collect::<Vec<_>>(),
            vec!["first", "cross source", "duplicate"]
        );
    }

    #[test]
    fn uri_groups_are_sorted_but_records_follow_the_canonical_index() {
        let fixture = CapturedFixture::new(&[
            ("file:///z.miz", "abcdefgh", 12),
            ("file:///a.miz", "abcdefgh", 3),
            ("file:///empty.miz", "clean", 9),
        ]);
        let z = fixture.source_ids[0];
        let mut drafts = vec![
            syntax_draft(
                fixture.snapshot(),
                1,
                "z_later",
                "aaa text",
                span(
                    SourceAnchor::Range(loaded_range(&fixture, 0, "e")),
                    DiagnosticSpanRole::Primary,
                    None,
                    None,
                ),
                vec![],
                vec![],
            ),
            syntax_draft(
                fixture.snapshot(),
                1,
                "a_first",
                "zzz text",
                span(
                    SourceAnchor::Range(loaded_range(&fixture, 1, "a")),
                    DiagnosticSpanRole::Primary,
                    None,
                    None,
                ),
                vec![],
                vec![],
            ),
            syntax_draft(
                fixture.snapshot(),
                1,
                "z_first",
                "zzz text",
                span(
                    SourceAnchor::Range(loaded_range(&fixture, 0, "a")),
                    DiagnosticSpanRole::Primary,
                    None,
                    None,
                ),
                vec![],
                vec![],
            ),
        ];
        drafts.reverse();
        let index = index(fixture.snapshot(), drafts);
        let expected_z = index.records().iter()
            .filter(|record| matches!(record.primary_location(), DiagnosticPrimaryLocation::Span(s) if s.range().source_id == z))
            .map(|record| record.message()).collect::<Vec<_>>();
        let notifications = convert_index(&index, &fixture.captured).unwrap();
        let uris = notifications
            .iter()
            .map(|event| event["params"]["uri"].as_str().unwrap())
            .collect::<Vec<_>>();
        assert_eq!(uris, vec![fixture.uri(1), fixture.uri(2), fixture.uri(0)]);
        assert_eq!(notifications[0]["params"]["version"], 3);
        assert_eq!(notifications[1]["params"]["version"], 9);
        assert_eq!(
            notifications[1]["params"]["diagnostics"]
                .as_array()
                .unwrap()
                .len(),
            0
        );
        assert_eq!(notifications[2]["params"]["version"], 12);
        assert_eq!(
            notifications[2]["params"]["diagnostics"]
                .as_array()
                .unwrap()
                .iter()
                .map(|d| d["message"].as_str().unwrap())
                .collect::<Vec<_>>(),
            expected_z
        );
        assert_eq!(index.records().len(), 3);
    }

    #[test]
    fn conversion_fails_the_whole_batch_for_source_load_unknown_and_invalid_spans() {
        let fixture = CapturedFixture::new(&[("file:///source.miz", "ab😀c", 1)]);
        let sid = fixture.source_ids[0];
        let good = syntax_draft(
            fixture.snapshot(),
            1,
            "valid",
            "valid",
            span(
                SourceAnchor::Range(loaded_range(&fixture, 0, "a")),
                DiagnosticSpanRole::Primary,
                None,
                None,
            ),
            vec![],
            vec![],
        );
        let unknown_id = loop {
            let candidate = fixture.ids.next_source_id(fixture.snapshot()).unwrap();
            if !fixture.source_ids.contains(&candidate) {
                break candidate;
            }
        };
        let source_load = DiagnosticDraft::new(DiagnosticDraftInput {
            source_snapshot: fixture.snapshot(),
            code: DiagnosticCode::from_parts(DiagnosticSeverity::Error, 600).unwrap(),
            phase: PipelinePhase::SourceLoad,
            category: FailureCategory::SourceLoadError,
            stable_detail_key: "source_load".into(),
            message: "unmappable source load".into(),
            primary_location: DiagnosticPrimaryLocation::SourceLoad {
                package_id: PackageId::new("alpha"),
                path: normalize_source_path(
                    &fixture._root.join("alpha"),
                    std::path::Path::new("src/source.miz"),
                )
                .unwrap(),
            },
            secondary_spans: vec![],
            notes: vec![],
            details: DiagnosticDetails::new(),
            fixes: vec![],
            explanation: None,
        })
        .unwrap();
        let mut parser_sink = DiagnosticSink::new(DiagnosticProducerScope::new(
            PipelinePhase::Parser,
            fixture.snapshot(),
            "parser",
        ));
        parser_sink.emit(good.clone()).unwrap();
        let mut source_load_sink = DiagnosticSink::new(DiagnosticProducerScope::new(
            PipelinePhase::SourceLoad,
            fixture.snapshot(),
            "source_load",
        ));
        source_load_sink.emit(source_load).unwrap();
        let non_range = BuildDiagnosticIndex::aggregate(DiagnosticAggregationInput::new(
            fixture.snapshot(),
            vec![parser_sink.into_batch(), source_load_sink.into_batch()],
        ))
        .unwrap();
        assert!(convert_index(&non_range, &fixture.captured).is_err());

        let unknown = syntax_draft(
            fixture.snapshot(),
            1,
            "unknown",
            "unknown source",
            span(
                SourceAnchor::Range(SourceRange {
                    source_id: unknown_id,
                    start: loaded_range(&fixture, 0, "a").start,
                    end: loaded_range(&fixture, 0, "a").end,
                }),
                DiagnosticSpanRole::Primary,
                None,
                None,
            ),
            vec![],
            vec![],
        );
        assert!(
            convert_index(
                &index(fixture.snapshot(), vec![good.clone(), unknown]),
                &fixture.captured
            )
            .is_err()
        );

        // Byte offset 3 is inside the four-byte supplementary scalar. The typed
        // span constructor permits this; the captured-source mapper must reject it.
        let emoji_range = loaded_range(&fixture, 0, "😀");
        let invalid_interior = emoji_range.start + '😀'.len_utf8() / 2;
        let invalid_primary = syntax_draft(
            fixture.snapshot(),
            1,
            "invalid_primary",
            "bad utf8",
            span(
                ordinary_range(sid, invalid_interior, invalid_interior + 1),
                DiagnosticSpanRole::Primary,
                None,
                None,
            ),
            vec![],
            vec![],
        );
        assert!(
            convert_index(
                &index(fixture.snapshot(), vec![good.clone(), invalid_primary]),
                &fixture.captured
            )
            .is_err()
        );

        let eof = loaded_text(&fixture, 0).len();
        let outside = syntax_draft(
            fixture.snapshot(),
            1,
            "outside",
            "outside source",
            span(
                ordinary_range(sid, eof, eof + "past".len()),
                DiagnosticSpanRole::Primary,
                None,
                None,
            ),
            vec![],
            vec![],
        );
        assert!(
            convert_index(
                &index(fixture.snapshot(), vec![good.clone(), outside]),
                &fixture.captured
            )
            .is_err()
        );

        let invalid_secondary = syntax_draft(
            fixture.snapshot(),
            1,
            "invalid_secondary",
            "bad related",
            span(
                SourceAnchor::Range(loaded_range(&fixture, 0, "a")),
                DiagnosticSpanRole::Primary,
                None,
                None,
            ),
            vec![span(
                SourceAnchor::Range(SourceRange {
                    source_id: unknown_id,
                    start: loaded_range(&fixture, 0, "a").start,
                    end: loaded_range(&fixture, 0, "a").end,
                }),
                DiagnosticSpanRole::Secondary,
                None,
                None,
            )],
            vec![],
        );
        assert!(
            convert_index(
                &index(fixture.snapshot(), vec![good.clone(), invalid_secondary]),
                &fixture.captured
            )
            .is_err()
        );

        let invalid_note = syntax_draft(
            fixture.snapshot(),
            1,
            "invalid_note",
            "bad note",
            span(
                SourceAnchor::Range(loaded_range(&fixture, 0, "a")),
                DiagnosticSpanRole::Primary,
                None,
                None,
            ),
            vec![],
            vec![DiagnosticNote::new(
                DiagnosticNoteKind::Note,
                "bad note span",
                Some(span(
                    ordinary_range(sid, invalid_interior, invalid_interior + 1),
                    DiagnosticSpanRole::Related,
                    None,
                    None,
                )),
            )],
        );
        assert!(
            convert_index(
                &index(fixture.snapshot(), vec![good, invalid_note]),
                &fixture.captured
            )
            .is_err()
        );
    }

    #[test]
    fn rejects_mismatched_capture_and_ignores_aggregator_withheld_obsolete_drafts() {
        let current = CapturedFixture::new(&[("file:///current.miz", "abc", 1)]);
        let other = CapturedFixture::new(&[("file:///other.miz", "abc", 1)]);
        let current_draft = syntax_draft(
            current.snapshot(),
            1,
            "current",
            "current message",
            span(
                SourceAnchor::Range(loaded_range(&current, 0, "a")),
                DiagnosticSpanRole::Primary,
                None,
                None,
            ),
            vec![],
            vec![],
        );
        let stale_draft = syntax_draft(
            other.snapshot(),
            1,
            "obsolete",
            "obsolete message",
            span(
                SourceAnchor::Range(loaded_range(&other, 0, "b")),
                DiagnosticSpanRole::Primary,
                None,
                None,
            ),
            vec![],
            vec![],
        );
        let mut current_sink = DiagnosticSink::new(DiagnosticProducerScope::new(
            PipelinePhase::Parser,
            current.snapshot(),
            "current",
        ));
        current_sink.emit(current_draft).unwrap();
        let mut old_sink = DiagnosticSink::new(DiagnosticProducerScope::new(
            PipelinePhase::Parser,
            other.snapshot(),
            "old",
        ));
        old_sink.emit(stale_draft).unwrap();
        let aggregate = BuildDiagnosticIndex::aggregate(DiagnosticAggregationInput::new(
            current.snapshot(),
            vec![old_sink.into_batch(), current_sink.into_batch()],
        ))
        .unwrap();
        assert_eq!(aggregate.obsolete_drafts().len(), 1);
        let notifications = convert_index(&aggregate, &current.captured).unwrap();
        assert_eq!(aggregate.records().len(), 1);
        assert_eq!(
            notifications[0]["params"]["diagnostics"]
                .as_array()
                .unwrap()
                .len(),
            1
        );
        assert_eq!(
            notifications[0]["params"]["diagnostics"][0]["message"],
            "current message"
        );

        let wrong_index = index(
            other.snapshot(),
            vec![syntax_draft(
                other.snapshot(),
                1,
                "wrong_snapshot",
                "wrong",
                span(
                    SourceAnchor::Range(loaded_range(&other, 0, "a")),
                    DiagnosticSpanRole::Primary,
                    None,
                    None,
                ),
                vec![],
                vec![],
            )],
        );
        assert!(convert_index(&wrong_index, &current.captured).is_err());
    }
}
