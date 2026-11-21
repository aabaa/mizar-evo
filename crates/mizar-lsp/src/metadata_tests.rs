use super::*;
use mizar_artifact::{
    module_summary::ModuleSummaryIdentity,
    registration_summary::{ArtifactHashClass, ArtifactHashRef},
    store::SchemaVersion,
    verified_artifact::{
        BuildProvenance, ExpressionMetadata, OverloadMetadata, VerifiedArtifact,
        VerifiedArtifactReadOptions, current_schema_version, read_verified_artifact,
        verified_artifact_json,
    },
};
use mizar_driver::request::{
    BatchInvocation, BatchRequest, BuildLaneId, BuildProfile, BuildRequestDraft,
    BuildRequestGeneration, BuildRequestOrigin, BuildTargets, DependencyInputSet, SourceInputSet,
    VerifierConfigInput,
};
use mizar_session::{
    DiskSourceLoader, Edition, Hash, InMemorySessionIdAllocator, ModulePath, PackageId,
    SnapshotRegistry, SourceInput, SourceOriginInput, ToolchainInfo, WorkspaceRoot,
    normalize_source_path,
};
use serde_json::Value;
use std::{
    fs,
    path::PathBuf,
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

struct MetadataFixture {
    root: PathBuf,
    captured: mizar_driver::request::CapturedSnapshot,
    uri: String,
    source_file: String,
    source_hash: Hash,
    loaded_text: String,
}

impl MetadataFixture {
    fn new(text: &str) -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "mizar-lsp-metadata-reader-{}-{nonce}",
            std::process::id()
        ));
        let package_root = root.join("alpha");
        fs::create_dir_all(package_root.join("src")).unwrap();
        let path = PathBuf::from("src/a.miz");
        fs::write(package_root.join(&path), text).unwrap();
        let uri = format!("file://{}", package_root.join(&path).display());
        let loader = DiskSourceLoader::new(&package_root);
        let source = SourceInput {
            package_id: PackageId::new("alpha"),
            module_path: ModulePath::new("a"),
            normalized_path: normalize_source_path(&package_root, &path).unwrap(),
            edition: Edition::new("metadata-test"),
            origin: SourceOriginInput::OpenBuffer {
                uri: uri.clone(),
                expected_version: 1,
                actual_version: 1,
                text: Arc::<str>::from(text),
            },
        };
        let ids = InMemorySessionIdAllocator::new();
        let registry = SnapshotRegistry::new();
        let draft = BuildRequestDraft {
            lane: BuildLaneId::new(1),
            origin: BuildRequestOrigin::Batch(BatchRequest {
                invocation: BatchInvocation::default(),
            }),
            generation: BuildRequestGeneration::new(1),
            workspace_root: WorkspaceRoot::new(root.to_string_lossy().into_owned()),
            profile: BuildProfile::new("metadata-test"),
            targets: BuildTargets::default(),
            source_inputs: SourceInputSet::default(),
            dependency_inputs: DependencyInputSet::new(
                vec![],
                Hash::from_bytes([2; Hash::BYTE_LEN]),
                ToolchainInfo::new("metadata-test"),
            ),
            verifier_config: VerifierConfigInput::new(Hash::from_bytes([1; Hash::BYTE_LEN])),
        };
        let session = draft
            .capture_sources(&ids, &registry, vec![(loader, source)])
            .unwrap();
        let captured = session.captured().clone();
        let loaded = &captured.native_sources().unwrap()[0].1;
        let source_file = loaded.normalized_path.as_str().to_owned();
        let source_hash = loaded.source_hash;
        let loaded_text = loaded.text.to_string();
        Self {
            root,
            captured,
            uri,
            source_file,
            source_hash,
            loaded_text,
        }
    }

    fn artifact(&self, expressions: Vec<ExpressionMetadata>) -> Arc<VerifiedArtifact> {
        let schema_version = current_schema_version();
        let reference = |class, family: &str, seed: u8| {
            ArtifactHashRef::new(
                class,
                family,
                SchemaVersion::new(1, 0),
                Hash::from_bytes([seed; Hash::BYTE_LEN]),
            )
        };
        let mut artifact = VerifiedArtifact {
            schema_version,
            module: ModuleSummaryIdentity {
                package_id: "alpha".into(),
                package_version: None,
                lockfile_identity: None,
                module_path: "a".into(),
                language_edition: "metadata-test".into(),
            },
            source_file: self.source_file.clone(),
            source_hash: self.source_hash,
            verified_at: None,
            interface_hash: Hash::from_bytes([0; Hash::BYTE_LEN]),
            implementation_hash: Hash::from_bytes([0; Hash::BYTE_LEN]),
            exports: vec![],
            expressions,
            statements: vec![],
            obligations: vec![],
            proof_witnesses: vec![],
            diagnostics: vec![],
            provenance: BuildProvenance {
                toolchain: "metadata-test".into(),
                language_edition: "metadata-test".into(),
                lockfile_hash: reference(ArtifactHashClass::Artifact, "mizar-build/lockfile", 1),
                verifier_config_hash: reference(
                    ArtifactHashClass::Interface,
                    "mizar-build/config",
                    2,
                ),
                dependency_artifact_hashes: vec![],
                cache_key: None,
                captured_source: Some({
                    let version = &self.captured.snapshot().source_versions[0];
                    (
                        version.source_id,
                        version.origin.clone(),
                        version.source_hash,
                    )
                }),
            },
        };
        artifact.refresh_hashes().unwrap();
        // Exercise the public validating reader used for published artifact inputs.
        let encoded = verified_artifact_json(&artifact).unwrap();
        Arc::new(read_verified_artifact(&encoded, VerifiedArtifactReadOptions::default()).unwrap())
    }

    fn expression(id: &str, start: usize, end: usize, surface: &str) -> ExpressionMetadata {
        ExpressionMetadata {
            expression_id: id.into(),
            source_range: mizar_artifact::module_summary::SourceRangeSummary {
                start_byte: start as u64,
                end_byte: end as u64,
            },
            expression_kind: "term".into(),
            rendered_surface: surface.into(),
            inferred_type: None,
            resolved_symbol: None,
            inserted_coercions: vec![],
            active_thesis: None,
            overload_resolution: None,
        }
    }

    fn hover(
        &self,
        expressions: Vec<ExpressionMetadata>,
        position: LspPosition,
    ) -> Result<Option<Value>, String> {
        artifact_hover(
            &[self.artifact(expressions)],
            &self.captured,
            &self.uri,
            position,
        )
    }
}

impl Drop for MetadataFixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn hover_text(hover: &Value) -> &str {
    hover["contents"]["value"]
        .as_str()
        .expect("plaintext hover")
}

#[test]
fn artifact_expression_hover_preserves_published_semantics_and_identity() {
    let fixture = MetadataFixture::new("x + 1");
    let mut expression = MetadataFixture::expression("producer-expr-17", 0, 5, "x + 1");
    expression.inferred_type = Some("real number".into());
    expression.resolved_symbol = Some("Arithmetic.add_real".into());
    expression.inserted_coercions = vec!["natural-to-real".into()];
    expression.active_thesis = Some("x is real".into());
    expression.overload_resolution = Some(OverloadMetadata {
        root_symbol: "+".into(),
        selected_candidate: "Arithmetic.add_real".into(),
        active_refinements: vec!["left operand: real".into(), "right operand: real".into()],
        coercion_summary: Some("natural-to-real".into()),
    });
    let hover = fixture
        .hover(
            vec![expression],
            LspPosition {
                line: 0,
                character: 2,
            },
        )
        .unwrap()
        .unwrap();
    let text = hover_text(&hover);
    for expected in [
        "producer-expr-17",
        "x + 1",
        "real number",
        "Arithmetic.add_real",
        "natural-to-real",
        "x is real",
        "left operand: real",
        "right operand: real",
    ] {
        assert!(
            text.contains(expected),
            "hover omitted {expected:?}: {text}"
        );
    }
    assert_eq!(hover["range"]["start"]["character"], 0);
    assert_eq!(hover["range"]["end"]["character"], 5);
}

#[test]
fn optional_fields_and_unavailable_artifacts_remain_absent() {
    let fixture = MetadataFixture::new("x");
    let hover = fixture
        .hover(
            vec![MetadataFixture::expression("bare-expr", 0, 1, "x")],
            LspPosition {
                line: 0,
                character: 0,
            },
        )
        .unwrap()
        .unwrap();
    let text = hover_text(&hover);
    assert_eq!(
        text,
        format!(
            "Source: {} (version 1)\nExpression: bare-expr\nSurface: x",
            fixture.uri
        )
    );
    assert_eq!(
        artifact_hover(
            &[],
            &fixture.captured,
            &fixture.uri,
            LspPosition {
                line: 0,
                character: 0
            }
        )
        .unwrap(),
        None
    );
    assert_eq!(
        artifact_hover(
            &[fixture.artifact(vec![])],
            &fixture.captured,
            &fixture.uri,
            LspPosition {
                line: 0,
                character: 0
            }
        )
        .unwrap(),
        None
    );
    assert_eq!(
        artifact_hover(
            &[fixture.artifact(vec![MetadataFixture::expression("bare-expr", 0, 1, "x")])],
            &fixture.captured,
            "file:///unknown.miz",
            LspPosition {
                line: 0,
                character: 0
            }
        )
        .unwrap(),
        None
    );
}

#[test]
fn nested_and_tied_ranges_use_smallest_enclosure_and_half_open_boundaries() {
    let fixture = MetadataFixture::new("abcdef");
    let rows = vec![
        MetadataFixture::expression("outer", 0, 6, "abcdef"),
        MetadataFixture::expression("z-tied", 1, 4, "bcd"),
        MetadataFixture::expression("a-tied", 1, 4, "bcd"),
        MetadataFixture::expression("disjoint", 5, 6, "f"),
    ];
    let hover = fixture
        .hover(
            rows.clone(),
            LspPosition {
                line: 0,
                character: 2,
            },
        )
        .unwrap()
        .unwrap();
    assert!(hover_text(&hover).contains("a-tied"));
    assert!(!hover_text(&hover).contains("z-tied"));
    // At the inner range's exclusive end, the enclosing expression still applies.
    let end = fixture
        .hover(
            rows.clone(),
            LspPosition {
                line: 0,
                character: 4,
            },
        )
        .unwrap()
        .unwrap();
    assert!(hover_text(&end).contains("outer"));
    let disjoint = fixture
        .hover(
            rows.clone(),
            LspPosition {
                line: 0,
                character: 5,
            },
        )
        .unwrap()
        .unwrap();
    assert!(hover_text(&disjoint).contains("disjoint"));
    assert_eq!(
        fixture
            .hover(
                rows,
                LspPosition {
                    line: 0,
                    character: 6
                }
            )
            .unwrap(),
        None
    );
}

#[test]
fn original_open_buffer_mapping_keeps_bom_crlf_and_utf16_coordinates() {
    let fixture = MetadataFixture::new("\u{feff}a\r\n😀e\u{301}z");
    let start = fixture.loaded_text.find("e\u{301}").unwrap();
    let end = start + "e\u{301}".len();
    let hover = fixture
        .hover(
            vec![MetadataFixture::expression(
                "unicode-expr",
                start,
                end,
                "e\u{301}",
            )],
            LspPosition {
                line: 1,
                character: 3,
            },
        )
        .unwrap()
        .unwrap();
    assert!(hover_text(&hover).contains("unicode-expr"));
    assert_eq!(hover["range"]["start"]["line"], 1);
    assert_eq!(hover["range"]["start"]["character"], 2);
    assert_eq!(hover["range"]["end"]["character"], 4);
    for invalid in [
        LspPosition {
            line: 2,
            character: 0,
        },
        LspPosition {
            line: 1,
            character: 1,
        },
    ] {
        assert!(matches!(
            fixture.hover(
                vec![MetadataFixture::expression(
                    "unicode-expr",
                    start,
                    end,
                    "e\u{301}"
                )],
                invalid
            ),
            Ok(None)
        ));
    }
}

#[test]
fn invalid_expression_ranges_are_explicit_and_cannot_cross_source_associations() {
    let fixture = MetadataFixture::new("abc");
    assert!(
        fixture
            .hover(
                vec![MetadataFixture::expression("out-of-bounds", 2, 8, "c")],
                LspPosition {
                    line: 0,
                    character: 2
                },
            )
            .is_err()
    );
    let unicode = MetadataFixture::new("a😀b");
    assert!(
        unicode
            .hover(
                vec![MetadataFixture::expression("utf8-interior", 2, 3, "x")],
                LspPosition {
                    line: 0,
                    character: 0
                },
            )
            .is_err()
    );
    let mut other = fixture.artifact(vec![MetadataFixture::expression("foreign", 0, 1, "a")]);
    Arc::make_mut(&mut other).source_file = "src/other.miz".into();
    // Revalidate after changing the producer-owned source association and hashes.
    Arc::make_mut(&mut other).refresh_hashes().unwrap();
    let encoded = verified_artifact_json(&other).unwrap();
    let other =
        Arc::new(read_verified_artifact(&encoded, VerifiedArtifactReadOptions::default()).unwrap());
    assert_eq!(
        artifact_hover(
            &[other],
            &fixture.captured,
            &fixture.uri,
            LspPosition {
                line: 0,
                character: 0
            }
        )
        .unwrap(),
        None
    );
    let mut same_path_other_package = fixture.artifact(vec![MetadataFixture::expression(
        "foreign-package",
        0,
        1,
        "a",
    )]);
    Arc::make_mut(&mut same_path_other_package)
        .module
        .package_id = "other-package".into();
    Arc::make_mut(&mut same_path_other_package)
        .refresh_hashes()
        .unwrap();
    let encoded = verified_artifact_json(&same_path_other_package).unwrap();
    let same_path_other_package =
        Arc::new(read_verified_artifact(&encoded, VerifiedArtifactReadOptions::default()).unwrap());
    assert_eq!(
        artifact_hover(
            &[same_path_other_package],
            &fixture.captured,
            &fixture.uri,
            LspPosition {
                line: 0,
                character: 0
            }
        )
        .unwrap(),
        None
    );
}

fn definition_fixture(
    text: &str,
    expect_publication: bool,
) -> crate::server::full_snapshot_registry_a::RegistryFixture {
    use mizar_driver::facade::{DischargePolicy, PhaseRegistryBuilder, VerifierPolicy};
    use mizar_ir::{
        identity::{OutputKind, PipelinePhase, WorkUnit},
        publisher::AllowedWorkUnit,
    };
    let mut fixture = crate::server::full_snapshot_registry_a::RegistryFixture::new();
    let uri = fixture.sources[0].2.clone();
    fixture.sources.truncate(1);
    fixture.input_template.source_layout =
        mizar_build::module_index::StaticSourceLayout::new(vec![
            mizar_build::module_index::WorkspaceSourcePackage {
                package_id: PackageId::new("alpha"),
                files: vec![mizar_build::module_index::WorkspaceSourceFile::new(
                    "src/main.miz",
                    "main.miz",
                )],
            },
        ]);
    fixture.input_template.vc_descriptors.truncate(1);
    fixture.did_change_text(&uri, 2, text.into());
    let mut builder = PhaseRegistryBuilder::new();
    builder
        .register_source_load()
        .register_frontend(vec![])
        .register_source_theorem_module_resolver()
        .register_source_theorem_signature_collection()
        .register_source_theorem_type_checking()
        .register_source_theorem_registration_resolution()
        .register_source_theorem_overload_resolution()
        .register_source_theorem_elaboration()
        .register_source_theorem_algorithm_preparation()
        .register_source_theorem_vc_generation("source-theorem-local-v1".into(), "vc-v1".into())
        .register_source_theorem_reflexivity_kernel_check(VerifierPolicy::release())
        .register_source_theorem_vc_discharge(DischargePolicy::default())
        .register_source_theorem_reflexivity_artifact_commit(
            VerifierPolicy::release(),
            (
                "proof-witnesses".into(),
                "main.mizir.json".into(),
                "artifacts".into(),
            ),
        )
        .unwrap();
    fixture.driver = CompilerDriver::new(builder.build().unwrap());
    let capture = fixture.capture();
    fixture.publisher.allow_work_unit(AllowedWorkUnit::new(
        PipelinePhase::new("KernelCheck"),
        OutputKind::new("SourceTheoremReflexivityKernelStatus"),
        WorkUnit::new(mizar_session::module_work_unit_label("alpha", "main")),
    ));
    let input = fixture.driver_input();
    let (submission, activated) = fixture
        .server
        .submit_originating_snapshot(
            &mut fixture.driver,
            capture,
            &fixture.ids,
            &fixture.snapshots,
            &fixture.publisher,
            input,
        )
        .unwrap();
    assert_eq!(
        activated, expect_publication,
        "genuine publication activation: {:?}",
        submission.status
    );
    fixture
}

#[test]
fn genuine_definition_publication_has_a_compiler_bound_citation() {
    let mut fixture = definition_fixture(DEFINITION_SOURCE, true);
    let uri = fixture.sources[0].2.clone();
    // Expected coordinates come from the authored source, not published metadata.
    let expected = json!({"uri": uri, "range": {
        "start": {"line": 0, "character": 0},
        "end": {"line": 0, "character": 78},
    }});
    for character in 76..80 {
        assert_eq!(
            fixture.server.snapshot_definition(
                &fixture.driver,
                &fixture.snapshots,
                &uri,
                LspPosition { line: 1, character },
            ),
            Some(expected.clone()),
            "citation character {character}"
        );
    }
    for character in [75, 80] {
        assert_eq!(
            fixture.server.snapshot_definition(
                &fixture.driver,
                &fixture.snapshots,
                &uri,
                LspPosition { line: 1, character },
            ),
            None,
            "outside half-open citation at {character}"
        );
    }
    let response = fixture
        .server
        .handle_message_with_metadata(
            json!({
                "jsonrpc": "2.0", "id": 26, "method": "textDocument/definition",
                "params": {"textDocument": {"uri": uri}, "position": {"line": 1, "character": 76}},
            }),
            &fixture.driver,
            &fixture.snapshots,
        )
        .unwrap();
    assert_eq!(response["result"], expected);
}

const DEFINITION_SOURCE: &str = concat!(
    "theorem Base: for X being set holds X = X proof let X be set; thus X = X; end;\n",
    "theorem Main: for Y being set holds Y = Y proof let Y be set; thus Y = Y by Base; end;\n",
);

// Raw source oracle: neither compiler ranges nor LSP conversion participate.
fn definition_position(raw: &str, byte: usize) -> LspPosition {
    let mut prefix = raw[..byte].chars().peekable();
    let (mut line, mut character) = (0, 0);
    while let Some(ch) = prefix.next() {
        if ch == '\r' || ch == '\n' {
            if ch == '\r' && prefix.peek() == Some(&'\n') {
                prefix.next();
            }
            line += 1;
            character = 0;
        } else {
            character += ch.len_utf16() as u32;
        }
    }
    LspPosition { line, character }
}

#[test]
fn definition_repeated_self_and_unmapped_citations_do_not_invent_targets() {
    let text = format!(
        "{DEFINITION_SOURCE}{}\n",
        DEFINITION_SOURCE
            .lines()
            .nth(1)
            .unwrap()
            .replace("Main:", "Next:")
    );
    let fixture = definition_fixture(&text, true);
    let uri = &fixture.sources[0].2;
    let (snapshot, _) = fixture
        .server
        .lsp_snapshot(&fixture.driver, &fixture.snapshots)
        .unwrap();
    let projection = snapshot.publication().source_metadata()[0]
        .as_ref()
        .unwrap()
        .direct_by()
        .unwrap();
    assert_eq!(projection.occurrences().len(), 2);
    for (byte, _) in text.match_indices("by Base") {
        let position = definition_position(&text, byte + 3);
        let target = fixture
            .server
            .snapshot_definition(&fixture.driver, &fixture.snapshots, uri, position)
            .unwrap();
        assert_eq!(target["range"]["start"], json!({"line": 0, "character": 0}));
    }
    // Declaration spellings and proof variables carry no direct-by target.
    for byte in [text.find("Base").unwrap(), text.find("X = X").unwrap()] {
        assert_eq!(
            fixture.server.snapshot_definition(
                &fixture.driver,
                &fixture.snapshots,
                uri,
                definition_position(&text, byte)
            ),
            None
        );
    }
    // The finite reflexivity profile does not publish these semantic forms.
    // A genuine failed submission must not invent targets from their spellings.
    for (replacement, spelling) in [
        ("by Base, Base", "Base, Base"),
        ("by Main", "Main; end"),
        ("by Missing", "Missing"),
        ("L: Y = Y; thus Y = Y by L", "L; end"),
    ] {
        let text = if replacement.starts_with("L:") {
            DEFINITION_SOURCE.replace("thus Y = Y by Base", replacement)
        } else {
            DEFINITION_SOURCE.replace("by Base", replacement)
        };
        let failed = definition_fixture(&text, false);
        assert!(
            failed
                .server
                .lsp_snapshot(&failed.driver, &failed.snapshots)
                .is_none()
        );
        assert_eq!(
            failed.server.snapshot_definition(
                &failed.driver,
                &failed.snapshots,
                &failed.sources[0].2,
                definition_position(&text, text.rfind(spelling).unwrap())
            ),
            None
        );
    }
}

#[test]
fn definition_raw_bom_unicode_and_newlines_preserve_utf16_coordinates() {
    let text = format!(
        "\u{feff}:: 😀 α\r continued\n{}\r\n:: 😀 λ\n{}\n",
        DEFINITION_SOURCE.lines().next().unwrap(),
        DEFINITION_SOURCE.lines().nth(1).unwrap()
    );
    let fixture = definition_fixture(&text, true);
    let uri = &fixture.sources[0].2;
    let start = definition_position(&text, text.find("theorem Base").unwrap());
    let end = definition_position(&text, text.find("end;").unwrap() + 4);
    let target_start = text.rfind("Base").unwrap();
    let expected = json!({"uri": uri, "range": {
        "start": {"line": start.line, "character": start.character},
        "end": {"line": end.line, "character": end.character},
    }});
    for byte in target_start..target_start + 4 {
        assert_eq!(
            fixture.server.snapshot_definition(
                &fixture.driver,
                &fixture.snapshots,
                uri,
                definition_position(&text, byte)
            ),
            Some(expected.clone())
        );
    }
    for byte in [target_start - 1, target_start + 4] {
        assert_eq!(
            fixture.server.snapshot_definition(
                &fixture.driver,
                &fixture.snapshots,
                uri,
                definition_position(&text, byte)
            ),
            None
        );
    }
    assert_eq!(
        fixture.server.snapshot_definition(
            &fixture.driver,
            &fixture.snapshots,
            uri,
            LspPosition {
                line: 0,
                character: 5
            }
        ),
        None,
        "inside emoji surrogate pair"
    );
}

#[test]
fn definition_authenticates_complete_endpoints_summary_and_import_visibility() {
    use mizar_artifact::direct_by::DirectByTarget;
    let mut fixture = definition_fixture(DEFINITION_SOURCE, true);
    let (snapshot, _) = fixture
        .server
        .lsp_snapshot(&fixture.driver, &fixture.snapshots)
        .unwrap();
    let publication = snapshot.publication();
    let artifact = publication.artifacts()[0].as_ref();
    let metadata = publication.source_metadata()[0].as_ref().unwrap();
    let DirectByTarget::Theorem(endpoint) = &metadata.direct_by().unwrap().occurrences()[0].target
    else {
        panic!("compiler theorem binding")
    };
    let summary = metadata.summary();
    let hash = endpoint.publication_hash;
    assert!(definition_export(artifact, summary, hash, endpoint, None).is_some());
    for field in [
        "package", "module", "version", "lock", "edition", "hash", "export",
    ] {
        let mut changed = endpoint.as_ref().clone();
        match field {
            "package" => changed.module.package_id.push_str("_other"),
            "module" => changed.module.module_path.push_str("_other"),
            "version" => changed.module.package_version = Some("9.0.0".into()),
            "lock" => changed.module.lockfile_identity = Some("other".into()),
            "edition" => changed.module.language_edition.push_str("_other"),
            "hash" => changed.publication_hash = Hash::from_bytes([91; Hash::BYTE_LEN]),
            "export" => {
                changed.export.documentation_ref =
                    Some(changed.export.interface_fingerprint.clone())
            }
            _ => unreachable!(),
        }
        assert!(
            definition_export(artifact, summary, hash, &changed, None).is_none(),
            "{field}"
        );
    }
    for field in [
        "source",
        "range",
        "signature",
        "fingerprint",
        "ambiguous",
        "unavailable",
    ] {
        let mut changed = summary.clone();
        let row = changed
            .exported_symbols
            .iter_mut()
            .find(|row| row.origin_id == endpoint.export.origin_id)
            .unwrap();
        match field {
            "source" => changed.source_hash = Hash::from_bytes([92; Hash::BYTE_LEN]),
            "range" => row.source_range.end_byte += 1,
            "signature" => row.rendered_signature.push('!'),
            "fingerprint" => row.interface_fingerprint = Hash::from_bytes([93; Hash::BYTE_LEN]),
            "ambiguous" => {
                let duplicate = row.clone();
                changed.exported_symbols.push(duplicate);
            }
            "unavailable" => changed.exported_symbols.clear(),
            _ => unreachable!(),
        }
        assert!(
            definition_export(artifact, &changed, hash, endpoint, None).is_none(),
            "{field}"
        );
    }
    let mut duplicate = artifact.clone();
    duplicate.exports.push(endpoint.export.clone());
    assert!(definition_export(&duplicate, summary, hash, endpoint, None).is_none());
    let unrelated = MetadataFixture::new(DEFINITION_SOURCE);
    assert_eq!(
        artifact_definition(
            publication,
            &unrelated.captured,
            &unrelated.uri,
            LspPosition {
                line: 1,
                character: 76
            }
        ),
        None
    );
    let uri = fixture.sources[0].2.clone();
    fixture.did_change_text(&uri, 3, format!(":: changed\n{DEFINITION_SOURCE}"));
    let changed_source = fixture.capture();
    assert_eq!(
        artifact_definition(
            publication,
            changed_source.snapshot.captured(),
            &uri,
            LspPosition {
                line: 2,
                character: 76
            }
        ),
        None,
        "same module with a different original source/hash"
    );

    // Resolver products exercise foreign visibility at the consumer helper boundary;
    // today's direct-by producer binds local theorem citations only.
    let mut imported = summary.clone();
    let mut target = artifact.clone();
    let mut bound = endpoint.as_ref().clone();
    for row in &mut imported.exported_symbols {
        row.fully_qualified_name = format!("alpha::main::T{}", row.source_range.start_byte);
    }
    for label in &mut imported.exported_labels {
        let original = summary
            .exported_symbols
            .iter()
            .find(|row| row.fully_qualified_name == label.owner_fully_qualified_name)
            .unwrap();
        label.owner_fully_qualified_name = imported
            .exported_symbols
            .iter()
            .find(|row| row.origin_id == original.origin_id)
            .unwrap()
            .fully_qualified_name
            .clone();
    }
    imported.refresh_interface_hash().unwrap();
    for export in &mut target.exports {
        export.fully_qualified_name = imported
            .exported_symbols
            .iter()
            .find(|row| row.origin_id == export.origin_id)
            .unwrap()
            .fully_qualified_name
            .clone();
    }
    bound.export = target
        .exports
        .iter()
        .find(|export| export.origin_id == endpoint.export.origin_id)
        .unwrap()
        .clone();
    let (_, visible) = crate::server::completion::tests::import_fixture(
        "import alpha.main;\ndefinition\nend;",
        "client",
        std::slice::from_ref(&imported),
    );
    let visible = visible.expect("resolver visibility product");
    assert!(definition_export(&target, &imported, hash, &bound, Some(&visible)).is_some());
    let (_, hidden) = crate::server::completion::tests::import_fixture(
        "definition\nend;",
        "client",
        std::slice::from_ref(&imported),
    );
    let hidden = hidden.expect("complete empty import surface");
    assert!(definition_export(&target, &imported, hash, &bound, Some(&hidden)).is_none());
    let mut changed = imported.clone();
    changed
        .exported_symbols
        .iter_mut()
        .find(|row| row.origin_id == bound.export.origin_id)
        .unwrap()
        .rendered_signature
        .push('!');
    let mut changed_target = target.clone();
    changed_target
        .exports
        .iter_mut()
        .find(|export| export.origin_id == bound.export.origin_id)
        .unwrap()
        .rendered_signature
        .push('!');
    let mut changed_bound = bound.clone();
    changed_bound.export.rendered_signature.push('!');
    assert!(definition_export(&changed_target, &changed, hash, &changed_bound, None).is_some());
    assert!(
        definition_export(
            &changed_target,
            &changed,
            hash,
            &changed_bound,
            Some(&visible)
        )
        .is_none()
    );
}

#[test]
fn definition_delivery_suppresses_stale_reopened_replaced_and_terminal_snapshots() {
    for action in [
        "text",
        "version",
        "reopen",
        "replacement",
        "shutdown",
        "exit",
    ] {
        let mut fixture = definition_fixture(DEFINITION_SOURCE, true);
        let uri = fixture.sources[0].2.clone();
        let position = LspPosition {
            line: 1,
            character: 76,
        };
        let (retained, _) = fixture
            .server
            .lsp_snapshot(&fixture.driver, &fixture.snapshots)
            .unwrap();
        let expected = fixture
            .server
            .snapshot_definition(&fixture.driver, &fixture.snapshots, &uri, position)
            .unwrap();
        match action {
            "text" => fixture.did_change_text(&uri, 3, format!(":: inserted\n{DEFINITION_SOURCE}")),
            "version" => fixture.did_change_version_only(&uri, 3),
            "reopen" => fixture.did_close_and_reopen(&uri, 2),
            "replacement" => {
                fixture.did_change_text(&uri, 3, format!("\n{DEFINITION_SOURCE}"));
                let capture = fixture.capture();
                fixture
                    .publisher
                    .allow_work_unit(mizar_ir::publisher::AllowedWorkUnit::new(
                        mizar_ir::identity::PipelinePhase::new("KernelCheck"),
                        mizar_ir::identity::OutputKind::new("SourceTheoremReflexivityKernelStatus"),
                        mizar_ir::identity::WorkUnit::new(mizar_session::module_work_unit_label(
                            "alpha", "main",
                        )),
                    ));
                let input = fixture.driver_input();
                assert!(
                    fixture
                        .server
                        .submit_originating_snapshot(
                            &mut fixture.driver,
                            capture,
                            &fixture.ids,
                            &fixture.snapshots,
                            &fixture.publisher,
                            input
                        )
                        .unwrap()
                        .1
                );
                let (now, _) = fixture
                    .server
                    .lsp_snapshot(&fixture.driver, &fixture.snapshots)
                    .unwrap();
                assert!(!Arc::ptr_eq(&retained, &now));
                assert!(
                    fixture
                        .server
                        .snapshot_definition(
                            &fixture.driver,
                            &fixture.snapshots,
                            &uri,
                            LspPosition {
                                line: 2,
                                character: 76
                            }
                        )
                        .is_some()
                );
            }
            "shutdown" => {
                fixture.server.handle_message(
                    json!({"jsonrpc": "2.0", "id": 50, "method": "shutdown", "params": null}),
                );
            }
            "exit" => {
                fixture
                    .server
                    .handle_message(json!({"jsonrpc": "2.0", "method": "exit"}));
            }
            _ => unreachable!(),
        }
        assert_eq!(
            fixture
                .server
                .snapshot_definition(&fixture.driver, &fixture.snapshots, &uri, position),
            None,
            "{action}"
        );
        assert_eq!(
            artifact_definition(
                retained.publication(),
                retained.native_open_buffers().captured(),
                &uri,
                position
            ),
            Some(expected),
            "retained reader {action}"
        );
    }
}
