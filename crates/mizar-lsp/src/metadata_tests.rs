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
