use super::*;
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

#[test]
fn doc_published_profiles_order_and_typed_preservation() {
    use mizar_artifact::{manifest::*, store::SchemaVersion};
    use mizar_doc::artifact_reader::read_published_artifacts;
    let (root, manifest, artifacts) = doc_published_fixture();
    let paths = std::iter::once("artifact-manifest.json".to_owned())
        .chain(
            manifest
                .modules
                .iter()
                .map(|entry| entry.artifact_file.clone()),
        )
        .chain(
            manifest.modules[1]
                .proof_witnesses
                .iter()
                .map(|w| w.witness_path.clone()),
        )
        .collect::<Vec<_>>();
    let bytes_before = paths
        .iter()
        .map(|path| std::fs::read(root.join(path)).unwrap())
        .collect::<Vec<_>>();
    let expected = read_manifest_file(&root, ManifestFileReadOptions::default()).unwrap();
    let actual = read_published_artifacts(&root, expected.artifact_hash).unwrap();
    assert_eq!(actual, (expected, artifacts.clone()));
    assert_eq!(
        actual
            .1
            .iter()
            .map(|a| a.schema_version)
            .collect::<Vec<_>>(),
        vec![SchemaVersion::new(1, 0), SchemaVersion::new(2, 0)]
    );
    assert_eq!(
        actual
            .1
            .iter()
            .map(|a| a.module.module_path.as_str())
            .collect::<Vec<_>>(),
        vec!["module_0", "module_1"]
    );
    assert_eq!(
        paths
            .iter()
            .map(|path| std::fs::read(root.join(path)).unwrap())
            .collect::<Vec<_>>(),
        bytes_before
    );
    assert!(!root.join("development/absent.json").exists());
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn doc_published_rejects_bound_hashes_versions_and_coverage() {
    use mizar_artifact::{
        manifest::*,
        store::{
            CanonicalJson, SchemaVersion, StoreIoError, artifact_hash_domain, canonical_json_bytes,
        },
        verified_artifact::*,
    };
    use mizar_doc::artifact_reader::read_published_artifacts;
    let (root, manifest, artifacts) = doc_published_fixture();
    let initial = read_manifest_file(&root, Default::default()).unwrap();
    assert!(
        matches!(read_published_artifacts(&root, mizar_session::hash_text("wrong publication")),
        Err(ManifestError::StoreIo(StoreIoError::ArtifactHashMismatch { path, .. })) if path == "artifact-manifest.json")
    );
    // Every mutated manifest gets its actual public store hash; stale outer hashes
    // cannot mask the selected reference guard. Module 0 remains valid throughout.
    for index in 0..11 {
        let mut bad = manifest.clone();
        let entry = &mut bad.modules[1];
        match index {
            0 => entry.artifact_hash.digest = mizar_session::hash_text("wrong store hash"),
            1 => entry.source_file = "src/other.miz".into(),
            2 => entry.source_hash = mizar_session::hash_text("wrong source hash"),
            3 => entry.module.module_path = "module_2".into(),
            4 => entry.interface_hash.digest = mizar_session::hash_text("wrong interface"),
            5 => {
                entry.implementation_hash.digest = mizar_session::hash_text("wrong implementation")
            }
            6 => {
                entry.proof_witnesses.pop();
            }
            7 => entry.proof_witnesses[0].witness_path = "proof-witnesses/other.bin".into(),
            _ => {
                let version = match index {
                    8 => SchemaVersion::new(1, 0),
                    9 => SchemaVersion::new(2, 1),
                    _ => SchemaVersion::new(9, 0),
                };
                entry.artifact_hash.schema_version = version;
                entry.interface_hash.schema_version = version;
                entry.implementation_hash.schema_version = version;
                entry.artifact_hash.digest =
                    artifact_hash_domain(VERIFIED_ARTIFACT_SCHEMA_FAMILY, version).hash(
                        &builtin_verified_artifact_json(&artifacts[1]).unwrap(),
                        &artifact_hash_excluded_paths(),
                    );
            }
        }
        let write = write_manifest_file(&root, &bad).unwrap();
        let before = std::fs::read(root.join("artifact-manifest.json")).unwrap();
        let error = read_published_artifacts(&root, write.artifact_hash).unwrap_err();
        let valid = match (index, &error) {
            (0, ManifestError::StoreIo(StoreIoError::ArtifactHashMismatch { path, .. })) => {
                path == "modules/module_1.json"
            }
            (1 | 2, ManifestError::ReferencedArtifactMismatch { path, field, .. }) => {
                path == "modules/module_1.json"
                    && field
                        == if index == 1 {
                            "source_file"
                        } else {
                            "source_hash"
                        }
            }
            (3..=5 | 8..=10, ManifestError::VerifiedArtifact { path, error }) => {
                path == "modules/module_1.json"
                    && matches!(
                        (index, error.as_ref()),
                        (3, VerifiedArtifactError::ModuleIdentityMismatch { .. })
                            | (
                                4,
                                VerifiedArtifactError::ExpectedInterfaceHashMismatch { .. }
                            )
                            | (
                                5,
                                VerifiedArtifactError::ExpectedImplementationHashMismatch { .. }
                            )
                            | (8..=10, VerifiedArtifactError::SchemaVersion(_))
                    )
            }
            (6 | 7, ManifestError::WitnessCoverageMismatch { module, .. }) => {
                module.contains("module_1")
            }
            _ => false,
        };
        assert!(valid, "case {index}: {error:?}");
        assert_eq!(
            std::fs::read(root.join("artifact-manifest.json")).unwrap(),
            before
        );
    }
    // Bind the actual malformed inner value to its actual store hash, isolating
    // public codec schema validation from publication-integrity checks.
    let mut json = builtin_verified_artifact_json(&artifacts[1]).unwrap();
    let CanonicalJson::Object(fields) = &mut json else {
        unreachable!()
    };
    fields.insert("schema_version".into(), CanonicalJson::string("2.1"));
    let mut bad = manifest.clone();
    bad.modules[1].artifact_hash.digest =
        artifact_hash_domain(VERIFIED_ARTIFACT_SCHEMA_FAMILY, SchemaVersion::new(2, 0))
            .hash(&json, &artifact_hash_excluded_paths());
    std::fs::write(
        root.join(&bad.modules[1].artifact_file),
        canonical_json_bytes(&json),
    )
    .unwrap();
    let write = write_manifest_file(&root, &bad).unwrap();
    assert!(
        matches!(read_published_artifacts(&root, write.artifact_hash), Err(ManifestError::VerifiedArtifact { path, error })
        if path == "modules/module_1.json" && matches!(*error, VerifiedArtifactError::SchemaVersion(_)))
    );
    let mut json = artifact_manifest_json(&manifest).unwrap();
    let CanonicalJson::Object(fields) = &mut json else {
        unreachable!()
    };
    fields.insert("schema_version".into(), CanonicalJson::string("2.0"));
    std::fs::write(
        root.join("artifact-manifest.json"),
        canonical_json_bytes(&json),
    )
    .unwrap();
    assert!(matches!(
        read_published_artifacts(&root, initial.artifact_hash),
        Err(ManifestError::SchemaVersion(_))
    ));
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn doc_published_missing_malformed_unsafe_and_witness_files() {
    use mizar_artifact::{
        manifest::*,
        store::{CanonicalJson, PublishedPathError, StoreIoError, canonical_json_bytes},
    };
    use mizar_doc::artifact_reader::read_published_artifacts;
    let (root, manifest, _) = doc_published_fixture();
    let hash = read_manifest_file(&root, Default::default())
        .unwrap()
        .artifact_hash;
    let targets = [
        "artifact-manifest.json",
        "modules/module_1.json",
        manifest.modules[1].proof_witnesses[0].witness_path.as_str(),
    ];
    for (index, relative) in targets.iter().enumerate() {
        let path = root.join(relative);
        let bytes = std::fs::read(&path).unwrap();
        std::fs::remove_file(&path).unwrap();
        assert!(
            matches!(
                read_published_artifacts(&root, hash),
                Err(ManifestError::StoreIo(StoreIoError::Io {
                    kind: std::io::ErrorKind::NotFound,
                    ..
                }))
            ),
            "missing {relative}"
        );
        std::fs::write(&path, &bytes).unwrap();
        if index < 2 {
            for (bad, utf8) in [(b"{".as_slice(), false), (b"\xff".as_slice(), true)] {
                std::fs::write(&path, bad).unwrap();
                let error = read_published_artifacts(&root, hash).unwrap_err();
                assert!(
                    matches!((&error, utf8),
                    (ManifestError::StoreIo(StoreIoError::InvalidUtf8 { path, .. }), true) |
                    (ManifestError::StoreIo(StoreIoError::CorruptCanonicalJson { path, .. }), false) if path == relative),
                    "{error:?}"
                );
                assert_eq!(std::fs::read(&path).unwrap(), bad);
            }
            std::fs::write(&path, &bytes).unwrap();
        }
        #[cfg(unix)]
        {
            let outside = root.with_extension(format!("outside-{index}"));
            std::fs::write(&outside, &bytes).unwrap();
            std::fs::remove_file(&path).unwrap();
            std::os::unix::fs::symlink(&outside, &path).unwrap();
            assert!(matches!(
                read_published_artifacts(&root, hash),
                Err(ManifestError::StoreIo(StoreIoError::Path { .. }))
            ));
            assert_eq!(std::fs::read(&outside).unwrap(), bytes);
            std::fs::remove_file(&path).unwrap();
            std::fs::remove_file(outside).unwrap();
            std::fs::write(&path, &bytes).unwrap();
        }
    }
    // Raw canonical JSON is intentional: the public writer rejects unsafe paths.
    for (field, unsafe_path) in [
        ("artifact_file", "../escape.json"),
        ("artifact_file", "/absolute.json"),
        ("witness_path", "../escape.bin"),
    ] {
        let mut json = artifact_manifest_json(&manifest).unwrap();
        let CanonicalJson::Object(fields) = &mut json else {
            unreachable!()
        };
        let CanonicalJson::Array(modules) = fields.get_mut("modules").unwrap() else {
            unreachable!()
        };
        let CanonicalJson::Object(entry) = &mut modules[1] else {
            unreachable!()
        };
        if field == "witness_path" {
            let CanonicalJson::Array(witnesses) = entry.get_mut("proof_witnesses").unwrap() else {
                unreachable!()
            };
            let CanonicalJson::Object(witness) = &mut witnesses[0] else {
                unreachable!()
            };
            witness.insert(field.into(), CanonicalJson::string(unsafe_path));
        } else {
            entry.insert(field.into(), CanonicalJson::string(unsafe_path));
        }
        let bytes = canonical_json_bytes(&json);
        std::fs::write(root.join("artifact-manifest.json"), &bytes).unwrap();
        let error = read_published_artifacts(&root, hash).unwrap_err();
        assert!(
            matches!(&error, ManifestError::InvalidField { path, reason }
                if path == if field == "witness_path" { "$.modules[1].proof_witnesses[0].witness_path" } else { "$.modules[1].artifact_file" }
                && reason == &if unsafe_path.starts_with('/') { PublishedPathError::Absolute } else { PublishedPathError::ParentSegment }.to_string()),
            "{error:?}"
        );
        assert_eq!(
            std::fs::read(root.join("artifact-manifest.json")).unwrap(),
            bytes
        );
    }
    write_manifest_file(&root, &manifest).unwrap();
    assert!(read_published_artifacts(&root, hash).is_ok());
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn doc_export_index_profiles_fqn_order_and_complete_preservation() {
    use mizar_artifact::manifest::{ManifestFileReadOptions, read_manifest_file};
    use mizar_doc::doc_build::read_published_export_index;
    let (root, mut manifest, mut artifacts) = doc_published_fixture();
    // Genuine witness payloads come from the old fixture; export metadata is caller-owned.
    let mut expected = std::collections::BTreeMap::new();
    for artifact in &mut artifacts {
        let mut zulu = artifact.exports[0].clone();
        zulu.origin_id = "a-origin".into();
        zulu.fully_qualified_name = "fixture::Zulu".into();
        let mut alpha = zulu.clone();
        alpha.origin_id = "z-origin".into();
        alpha.fully_qualified_name = "fixture::Alpha".into();
        alpha.namespace_path = vec!["caller".into(), "nested".into()];
        alpha.export_kind = "caller-kind".into();
        alpha.rendered_signature = "caller signature with uninterpreted text".into();
        artifact.exports = vec![zulu.clone(), alpha.clone()];
        expected.insert(artifact.module.clone(), vec![alpha, zulu]);
    }
    let hash = doc_export_index_publish(&root, &mut manifest, &mut artifacts);
    let paths = std::iter::once("artifact-manifest.json".to_owned())
        .chain(manifest.modules.iter().map(|m| m.artifact_file.clone()))
        .chain(
            manifest.modules[1]
                .proof_witnesses
                .iter()
                .map(|w| w.witness_path.clone()),
        )
        .collect::<Vec<_>>();
    let before = paths
        .iter()
        .map(|path| std::fs::read(root.join(path)).unwrap())
        .collect::<Vec<_>>();
    let expected_manifest = read_manifest_file(&root, ManifestFileReadOptions::default()).unwrap();
    assert_eq!(
        read_published_export_index(&root, hash).unwrap(),
        (expected_manifest, expected)
    );
    assert_eq!(
        paths
            .iter()
            .map(|path| std::fs::read(root.join(path)).unwrap())
            .collect::<Vec<_>>(),
        before
    );
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn doc_export_index_full_identity_duplicates_and_absent_metadata() {
    use mizar_artifact::{
        module_summary::ModuleSummaryIdentity, verified_artifact::ExportVisibility,
    };
    use mizar_doc::doc_build::read_published_export_index;
    let (root, mut manifest, originals) = doc_published_fixture();
    manifest.package.package_id = "outer-publication".into();
    // These caller-owned identity constants state existing tuple/string order,
    // including lexical 1.10 before 1.2; every module has the same path.
    let identities = [
        ("package-a", None, None, "2026"),
        ("package-a", None, None, "2027"),
        ("package-a", None, Some("lock-a"), "2026"),
        ("package-a", None, Some("lock-b"), "2026"),
        ("package-a", Some("1.10"), None, "2026"),
        ("package-a", Some("1.2"), None, "2026"),
        ("package-b", None, None, "2026"),
    ]
    .map(|(package, version, lock, edition)| ModuleSummaryIdentity {
        package_id: package.into(),
        package_version: version.map(str::to_owned),
        lockfile_identity: lock.map(str::to_owned),
        module_path: "same_path".into(),
        language_edition: edition.into(),
    });
    let mut first = originals[0].exports[0].clone();
    first.origin_id = "a-origin".into();
    first.fully_qualified_name = "fixture::duplicate".into();
    first.visibility = ExportVisibility::Reexported;
    first.proof_status = None;
    first.documentation_ref = None;
    let mut last = first.clone();
    last.origin_id = "z-origin".into();
    last.visibility = ExportVisibility::Public;
    last.documentation_ref = originals[0].exports[0].documentation_ref.clone();
    let mut artifacts = identities
        .iter()
        .rev()
        .map(|identity| {
            let mut artifact = originals[0].clone();
            artifact.module = identity.clone();
            artifact.exports = vec![first.clone(), last.clone()];
            artifact
        })
        .collect::<Vec<_>>();
    let hash = doc_export_index_publish(&root, &mut manifest, &mut artifacts);
    let (read, index) = read_published_export_index(&root, hash).unwrap();
    assert_eq!(read.manifest.package, manifest.package);
    assert_eq!(index.keys().cloned().collect::<Vec<_>>(), identities);
    for identity in &identities {
        assert_ne!(identity.package_id, read.manifest.package.package_id);
        assert_eq!(index[identity], vec![first.clone(), last.clone()]);
    }
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn doc_export_index_empty_inputs_delegated_errors_and_file_preservation() {
    use mizar_artifact::{manifest::*, store::StoreIoError};
    use mizar_doc::doc_build::read_published_export_index;
    let (root, mut manifest, mut artifacts) = doc_published_fixture();
    for artifact in &mut artifacts {
        artifact.exports.clear();
    }
    let hash = doc_export_index_publish(&root, &mut manifest, &mut artifacts);
    let paths = std::iter::once("artifact-manifest.json".to_owned())
        .chain(manifest.modules.iter().map(|m| m.artifact_file.clone()))
        .chain(
            manifest.modules[1]
                .proof_witnesses
                .iter()
                .map(|w| w.witness_path.clone()),
        )
        .collect::<Vec<_>>();
    let before = paths
        .iter()
        .map(|path| std::fs::read(root.join(path)).unwrap())
        .collect::<Vec<_>>();
    let (read, index) = read_published_export_index(&root, hash).unwrap();
    assert_eq!(read.manifest, manifest);
    assert_eq!(index.len(), artifacts.len());
    for artifact in &artifacts {
        assert_eq!(index.get(&artifact.module), Some(&Vec::new()));
    }
    assert!(matches!(
        read_published_export_index(&root, mizar_session::hash_text("wrong publication")),
        Err(ManifestError::StoreIo(StoreIoError::ArtifactHashMismatch { path, .. }))
            if path == "artifact-manifest.json"
    ));
    let missing = root.join(&manifest.modules[1].artifact_file);
    let nested_bytes = std::fs::read(&missing).unwrap();
    std::fs::remove_file(&missing).unwrap();
    assert!(matches!(
        read_published_export_index(&root, hash),
        Err(ManifestError::StoreIo(StoreIoError::Io {
            kind: std::io::ErrorKind::NotFound,
            ..
        }))
    ));
    assert!(!missing.exists());
    std::fs::write(&missing, nested_bytes).unwrap();
    assert_eq!(
        paths
            .iter()
            .map(|path| std::fs::read(root.join(path)).unwrap())
            .collect::<Vec<_>>(),
        before
    );
    manifest.modules.clear();
    let hash = write_manifest_file(&root, &manifest).unwrap().artifact_hash;
    let before = std::fs::read(root.join("artifact-manifest.json")).unwrap();
    let (read, index) = read_published_export_index(&root, hash).unwrap();
    assert_eq!(read.manifest, manifest);
    assert!(index.is_empty());
    assert_eq!(
        std::fs::read(root.join("artifact-manifest.json")).unwrap(),
        before
    );
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn doc_native_source_genuine_artifact_hash_and_preserved_bytes() {
    use mizar_doc::artifact_reader::read_native_disk_source;
    let (root, manifest, artifacts) = doc_published_fixture();
    let original = std::fs::read(step5c13_registration_case().source_path).unwrap();
    let expected_text = std::str::from_utf8(&original).unwrap().to_owned();
    let path = root.join("src/source.miz");
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(&path, &original).unwrap();
    let publication_paths = std::iter::once("artifact-manifest.json".to_owned())
        .chain(manifest.modules.iter().map(|m| m.artifact_file.clone()))
        .chain(
            manifest.modules[1]
                .proof_witnesses
                .iter()
                .map(|w| w.witness_path.clone()),
        )
        .collect::<Vec<_>>();
    let publication_before = publication_paths
        .iter()
        .map(|p| std::fs::read(root.join(p)).unwrap())
        .collect::<Vec<_>>();
    for (artifact, requested) in artifacts
        .iter()
        .zip([PathBuf::from("src/source.miz"), path.clone()])
    {
        let expected_hash = artifact.source_hash;
        assert_eq!(
            read_native_disk_source(&root, requested, expected_hash).unwrap(),
            Some((expected_text.clone(), None))
        );
        assert_eq!(std::fs::read(&path).unwrap(), original);
    }
    assert_eq!(
        publication_paths
            .iter()
            .map(|p| std::fs::read(root.join(p)).unwrap())
            .collect::<Vec<_>>(),
        publication_before
    );
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn doc_native_source_literal_normalization_and_complete_loading_maps() {
    use mizar_doc::artifact_reader::read_native_disk_source;
    use mizar_session::{
        hash_text,
        source_map::{LoadingMapSegment::*, TextRange},
    };
    let (root, _, _) = doc_published_fixture();
    let path = root.join("src/source.miz");
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    let cases = [
        ("", "", None),
        ("λ\r\u{feff}z\n", "λ\r\u{feff}z\n", None),
        (
            "\u{feff}",
            "",
            Some(vec![
                RemovedLeadingBom {
                    original: TextRange { start: 0, end: 3 },
                },
                Original {
                    loaded: TextRange { start: 0, end: 0 },
                    original: TextRange { start: 3, end: 3 },
                },
            ]),
        ),
        (
            "\u{feff}\u{feff}λ",
            "\u{feff}λ",
            Some(vec![
                RemovedLeadingBom {
                    original: TextRange { start: 0, end: 3 },
                },
                Original {
                    loaded: TextRange { start: 0, end: 5 },
                    original: TextRange { start: 3, end: 8 },
                },
            ]),
        ),
        (
            "\r\n\r\n",
            "\n\n",
            Some(vec![
                NormalizedNewline {
                    loaded: TextRange { start: 0, end: 1 },
                    original: TextRange { start: 0, end: 2 },
                },
                NormalizedNewline {
                    loaded: TextRange { start: 1, end: 2 },
                    original: TextRange { start: 2, end: 4 },
                },
            ]),
        ),
        (
            "\r\nλ",
            "\nλ",
            Some(vec![
                NormalizedNewline {
                    loaded: TextRange { start: 0, end: 1 },
                    original: TextRange { start: 0, end: 2 },
                },
                Original {
                    loaded: TextRange { start: 1, end: 3 },
                    original: TextRange { start: 2, end: 4 },
                },
            ]),
        ),
        (
            "\u{feff}λ\r\nz\r\u{feff}\r\n",
            "λ\nz\r\u{feff}\n",
            Some(vec![
                RemovedLeadingBom {
                    original: TextRange { start: 0, end: 3 },
                },
                Original {
                    loaded: TextRange { start: 0, end: 2 },
                    original: TextRange { start: 3, end: 5 },
                },
                NormalizedNewline {
                    loaded: TextRange { start: 2, end: 3 },
                    original: TextRange { start: 5, end: 7 },
                },
                Original {
                    loaded: TextRange { start: 3, end: 8 },
                    original: TextRange { start: 7, end: 12 },
                },
                NormalizedNewline {
                    loaded: TextRange { start: 8, end: 9 },
                    original: TextRange { start: 12, end: 14 },
                },
            ]),
        ),
    ];
    for (raw, expected_text, expected_map) in cases {
        let expected_hash = hash_text(expected_text);
        std::fs::write(&path, raw).unwrap();
        assert_eq!(
            read_native_disk_source(&root, "src/source.miz", expected_hash).unwrap(),
            Some((expected_text.to_owned(), expected_map)),
            "{raw:?}"
        );
        assert_eq!(std::fs::read(&path).unwrap(), raw.as_bytes());
    }
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn doc_native_source_absence_typed_errors_and_edit_restore() {
    use mizar_doc::artifact_reader::read_native_disk_source;
    use mizar_session::source::SourcePathError;
    use std::io::ErrorKind;
    let (root, _, artifacts) = doc_published_fixture();
    let original_path = step5c13_registration_case().source_path;
    let original = std::fs::read(&original_path).unwrap();
    let expected_hash = artifacts[1].source_hash;
    let path = root.join("src/source.miz");
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    assert_eq!(
        read_native_disk_source(&root, "src/source.miz", expected_hash).unwrap(),
        None
    );
    assert!(!path.exists());
    let missing_root = root.join("absent");
    let error =
        read_native_disk_source(&missing_root, "src/source.miz", expected_hash).unwrap_err();
    assert_eq!(error.kind(), ErrorKind::NotFound);
    assert_eq!(
        error.get_ref().unwrap().downcast_ref::<SourcePathError>(),
        Some(&SourcePathError::PackageRootUnavailable {
            path: missing_root,
            kind: ErrorKind::NotFound
        })
    );
    let canonical_root = std::fs::canonicalize(&root).unwrap();
    let bad_extension = root.join("src/source.txt");
    std::fs::write(&bad_extension, &original).unwrap();
    let error = read_native_disk_source(&root, "src/source.txt", expected_hash).unwrap_err();
    assert_eq!(error.kind(), ErrorKind::InvalidInput);
    assert_eq!(
        error.get_ref().unwrap().downcast_ref::<SourcePathError>(),
        Some(&SourcePathError::UnsupportedExtension {
            path: canonical_root.join("src/source.txt")
        })
    );
    let error = read_native_disk_source(&root, &original_path, expected_hash).unwrap_err();
    assert_eq!(error.kind(), ErrorKind::InvalidInput);
    assert_eq!(
        error.get_ref().unwrap().downcast_ref::<SourcePathError>(),
        Some(&SourcePathError::OutsidePackageRoot {
            package_root: canonical_root.clone(),
            path: std::fs::canonicalize(&original_path).unwrap()
        })
    );
    std::fs::write(&path, &original).unwrap();
    let blocked = "src/source.miz/child.miz";
    let kind = std::fs::canonicalize(root.join(blocked))
        .unwrap_err()
        .kind();
    assert_ne!(kind, ErrorKind::NotFound);
    let error = read_native_disk_source(&root, blocked, expected_hash).unwrap_err();
    assert_eq!(error.kind(), kind);
    assert_eq!(
        error.get_ref().unwrap().downcast_ref::<SourcePathError>(),
        Some(&SourcePathError::SourcePathUnavailable {
            path: canonical_root.join(blocked),
            kind
        })
    );
    let directory = root.join("src/directory.miz");
    std::fs::create_dir(&directory).unwrap();
    let expected_error = std::fs::read(&directory).unwrap_err();
    let error = read_native_disk_source(&root, "src/directory.miz", expected_hash).unwrap_err();
    assert_eq!(error.kind(), expected_error.kind());
    assert_eq!(error.raw_os_error(), expected_error.raw_os_error());
    assert!(error.get_ref().is_none());
    std::fs::write(&path, [0xff]).unwrap();
    let error = read_native_disk_source(&root, "src/source.miz", expected_hash).unwrap_err();
    assert_eq!(error.kind(), ErrorKind::InvalidData);
    let utf8 = error
        .get_ref()
        .unwrap()
        .downcast_ref::<std::str::Utf8Error>()
        .unwrap();
    assert_eq!(utf8.valid_up_to(), 0);
    assert_eq!(utf8.error_len(), Some(1));
    assert_eq!(std::fs::read(&path).unwrap(), [0xff]);
    let mut edited = original.clone();
    edited.extend_from_slice(b"\n:: edited source\n");
    std::fs::write(&path, &edited).unwrap();
    assert_eq!(
        read_native_disk_source(&root, "src/source.miz", expected_hash)
            .unwrap_err()
            .kind(),
        ErrorKind::InvalidData
    );
    assert_eq!(std::fs::read(&path).unwrap(), edited);
    std::fs::write(&path, &original).unwrap();
    assert_eq!(
        read_native_disk_source(&root, "src/source.miz", expected_hash).unwrap(),
        Some((String::from_utf8(original.clone()).unwrap(), None))
    );
    assert_eq!(std::fs::read(&path).unwrap(), original);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn doc_module_summaries_complete_fields_vector_order_replay_and_purity() {
    use mizar_artifact::{manifest::*, module_summary::*, store::*};
    use mizar_doc::{
        artifact_reader::read_published_artifacts, doc_build::read_published_module_summary_index,
    };
    let (root, manifest, summaries) = doc_module_summaries_fixture();
    let selected = read_manifest_file(&root, ManifestFileReadOptions::default()).unwrap();
    let hash = selected.artifact_hash;
    let paths = std::iter::once("artifact-manifest.json".to_owned())
        .chain(manifest.modules.iter().flat_map(|entry| {
            std::iter::once(entry.artifact_file.clone())
                .chain(entry.module_summary_file.clone())
                .chain(entry.proof_witnesses.iter().map(|w| w.witness_path.clone()))
        }))
        .collect::<Vec<_>>();
    let before = paths
        .iter()
        .map(|p| std::fs::read(root.join(p)).unwrap())
        .collect::<Vec<_>>();
    assert_eq!(read_published_artifacts(&root, hash).unwrap().0, selected);
    let mut expected = std::collections::BTreeMap::new();
    for (entry, summary) in manifest.modules.iter().zip(summaries) {
        let path = PublishedArtifactPath::new(entry.module_summary_file.as_ref().unwrap()).unwrap();
        let domain = artifact_hash_domain(MODULE_SUMMARY_SCHEMA_FAMILY, summary.schema_version);
        let original = read_published_artifact(
            &root,
            &path,
            PublishedArtifactReadOptions {
                artifact_hash_domain: Some(&domain),
                hash_excluded_paths: &[],
                expected_artifact_hash: Some(entry.module_summary_hash.as_ref().unwrap().digest),
            },
        )
        .unwrap();
        assert_eq!(
            read_module_summary(&original.value, ModuleSummaryReadOptions::default()).unwrap(),
            summary
        );
        assert_eq!(summary.exported_labels.len(), 2);
        assert_eq!(
            summary.exported_labels[0].label,
            summary.exported_labels[1].label
        );
        assert_ne!(
            summary.exported_labels[0].origin_id,
            summary.exported_labels[1].origin_id
        );
        expected.insert(summary.module.clone(), Some(summary));
    }
    for _ in 0..2 {
        assert_eq!(
            read_published_module_summary_index(&root, hash).unwrap(),
            (selected.clone(), expected.clone())
        );
    }
    assert_eq!(
        paths
            .iter()
            .map(|p| std::fs::read(root.join(p)).unwrap())
            .collect::<Vec<_>>(),
        before
    );
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn doc_module_summaries_full_identity_absence_empty_labels_modules_and_publication() {
    use mizar_artifact::{manifest::*, module_summary::*, store::*};
    use mizar_doc::{
        artifact_reader::read_published_artifacts, doc_build::read_published_module_summary_index,
    };
    let (root, mut manifest, mut summaries) = doc_module_summaries_fixture();
    let selected = read_manifest_file(&root, ManifestFileReadOptions::default()).unwrap();
    let originals = read_published_artifacts(&root, selected.artifact_hash)
        .unwrap()
        .1;
    let identities = [
        ("package-a", None, None, "2026"),
        ("package-a", None, None, "2027"),
        ("package-a", None, Some("lock-a"), "2026"),
        ("package-a", None, Some("lock-b"), "2026"),
        ("package-a", Some("1.10"), None, "2026"),
        ("package-a", Some("1.2"), None, "2026"),
        ("package-b", None, None, "2026"),
    ]
    .map(|(package, version, lock, edition)| ModuleSummaryIdentity {
        package_id: package.into(),
        package_version: version.map(str::to_owned),
        lockfile_identity: lock.map(str::to_owned),
        module_path: "same_path".into(),
        language_edition: edition.into(),
    });
    for entry in &mut manifest.modules {
        entry.module_summary_file = None;
        entry.module_summary_hash = None;
        entry.module_summary_interface_hash = None;
    }
    let mut artifacts = identities
        .iter()
        .rev()
        .map(|identity| {
            let mut artifact = originals[0].clone();
            artifact.module = identity.clone();
            artifact.exports.clear();
            artifact
        })
        .collect::<Vec<_>>();
    doc_export_index_publish(&root, &mut manifest, &mut artifacts);
    let empty = &mut summaries[0];
    empty.module = identities[0].clone();
    empty.exported_symbols.clear();
    empty.exported_labels.clear();
    empty.lexical_summary.fingerprint = None;
    empty.lexical_summary.contributions.clear();
    empty.reexports.clear();
    empty.dependency_interfaces.clear();
    empty.refresh_interface_hash().unwrap();
    let path = PublishedArtifactPath::new("summaries/empty.json").unwrap();
    let write = write_published_artifact(
        &root,
        &path,
        &module_summary_json(empty).unwrap(),
        &artifact_hash_domain(MODULE_SUMMARY_SCHEMA_FAMILY, empty.schema_version),
        &[],
    )
    .unwrap();
    let entry = manifest
        .modules
        .iter_mut()
        .find(|e| e.module == identities[0])
        .unwrap();
    entry.module_summary_file = Some(path.as_str().into());
    entry.module_summary_hash = selected.manifest.modules[0].module_summary_hash.clone();
    entry.module_summary_hash.as_mut().unwrap().digest = write.artifact_hash;
    entry.module_summary_interface_hash = selected.manifest.modules[0]
        .module_summary_interface_hash
        .clone();
    entry.module_summary_interface_hash.as_mut().unwrap().digest = empty.interface_hash;
    manifest.package.package_id = "outer-publication".into();
    let hash = write_manifest_file(&root, &manifest).unwrap().artifact_hash;
    let (read, index) = read_published_module_summary_index(&root, hash).unwrap();
    assert_eq!(
        read,
        read_manifest_file(&root, ManifestFileReadOptions::default()).unwrap()
    );
    assert_eq!(index.keys().cloned().collect::<Vec<_>>(), identities);
    assert_eq!(index[&identities[0]], Some(empty.clone()));
    for identity in &identities[1..] {
        assert_eq!(index.get(identity), Some(&None));
    }
    manifest.modules.clear();
    let hash = write_manifest_file(&root, &manifest).unwrap().artifact_hash;
    let (read, index) = read_published_module_summary_index(&root, hash).unwrap();
    assert_eq!(
        read,
        read_manifest_file(&root, ManifestFileReadOptions::default()).unwrap()
    );
    assert!(index.is_empty());
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn doc_module_summaries_delegated_refusals_preserve_errors_and_bytes() {
    use mizar_artifact::{manifest::*, module_summary::*, store::*};
    use mizar_doc::{
        artifact_reader::read_published_artifacts, doc_build::read_published_module_summary_index,
    };
    let (root, manifest, summaries) = doc_module_summaries_fixture();
    let path =
        PublishedArtifactPath::new(manifest.modules[0].module_summary_file.as_ref().unwrap())
            .unwrap();
    let paths = std::iter::once("artifact-manifest.json".to_owned())
        .chain(manifest.modules.iter().flat_map(|entry| {
            std::iter::once(entry.artifact_file.clone())
                .chain(entry.module_summary_file.clone())
                .chain(entry.proof_witnesses.iter().map(|w| w.witness_path.clone()))
        }))
        .collect::<Vec<_>>();
    for case in 0..6 {
        let mut changed = manifest.clone();
        let mut summary = summaries[0].clone();
        let wrong = mizar_session::hash_text("caller-selected mismatch");
        match case {
            2 => changed.modules[0].module_summary_file = Some("summaries/missing.json".into()),
            3 => summary.module.module_path = "other_module".into(),
            4 => {
                changed.modules[0]
                    .module_summary_interface_hash
                    .as_mut()
                    .unwrap()
                    .digest = wrong
            }
            5 => summary.source_hash = wrong,
            _ => {}
        }
        summary.refresh_interface_hash().unwrap();
        let write = write_published_artifact(
            &root,
            &path,
            &module_summary_json(&summary).unwrap(),
            &artifact_hash_domain(MODULE_SUMMARY_SCHEMA_FAMILY, summary.schema_version),
            &[],
        )
        .unwrap();
        changed.modules[0]
            .module_summary_hash
            .as_mut()
            .unwrap()
            .digest = if case == 1 {
            wrong
        } else {
            write.artifact_hash
        };
        let published = write_manifest_file(&root, &changed).unwrap().artifact_hash;
        let hash = if case == 0 { wrong } else { published };
        let before = paths
            .iter()
            .map(|p| std::fs::read(root.join(p)).unwrap())
            .collect::<Vec<_>>();
        let original = read_published_artifacts(&root, hash).unwrap_err();
        match (case, &original) {
            (0 | 1, ManifestError::StoreIo(StoreIoError::ArtifactHashMismatch { .. })) => {}
            (
                2,
                ManifestError::StoreIo(StoreIoError::Io {
                    kind: std::io::ErrorKind::NotFound,
                    ..
                }),
            ) => {}
            (3, ManifestError::ModuleSummary { error, .. }) => assert!(matches!(
                error.as_ref(),
                ModuleSummaryError::ModuleIdentityMismatch { .. }
            )),
            (4, ManifestError::ModuleSummary { error, .. }) => assert!(matches!(
                error.as_ref(),
                ModuleSummaryError::ExpectedInterfaceHashMismatch { .. }
            )),
            (5, ManifestError::ReferencedArtifactMismatch { field, .. }) => {
                assert_eq!(field, "source_hash")
            }
            _ => panic!("unexpected delegated refusal for case {case}: {original:?}"),
        }
        assert_eq!(
            read_published_module_summary_index(&root, hash).unwrap_err(),
            original
        );
        assert_eq!(
            paths
                .iter()
                .map(|p| std::fs::read(root.join(p)).unwrap())
                .collect::<Vec<_>>(),
            before
        );
        assert!(!root.join("summaries/missing.json").exists());
    }
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn doc_label_export_candidates_exact_records_all_owners_identity_and_purity() {
    use mizar_artifact::{manifest::*, module_summary::*, store::*, verified_artifact::*};
    use mizar_doc::{
        artifact_reader::read_published_artifacts,
        doc_build::read_published_label_export_candidates as read,
    };
    let (root, mut manifest, mut summaries) = doc_module_summaries_fixture();
    let selected = read_manifest_file(&root, ManifestFileReadOptions::default()).unwrap();
    let mut artifacts = read_published_artifacts(&root, selected.artifact_hash)
        .unwrap()
        .1;
    let mut unique = artifacts[0].exports[0].clone();
    unique.fully_qualified_name = "caller::unique".into();
    let mut last = unique.clone();
    last.origin_id = "z-owner".into();
    last.fully_qualified_name = "caller::multiple".into();
    last.namespace_path = vec!["uninterpreted".into()];
    last.visibility = ExportVisibility::Reexported;
    last.export_kind = "different-kind".into();
    last.source_range = SourceRangeSummary {
        start_byte: 20,
        end_byte: 30,
    };
    last.rendered_signature = "uninterpreted signature".into();
    last.interface_fingerprint.digest = mizar_session::hash_text("different interface");
    last.proof_status = None;
    last.documentation_ref = None;
    let mut first = last.clone();
    first.origin_id = "a-owner".into();
    first.proof_status = unique.proof_status;
    first.documentation_ref = unique.documentation_ref.clone();
    let mut unrelated = unique.clone();
    unrelated.origin_id = "unrelated".into();
    unrelated.fully_qualified_name = "caller::multiple_extra".into();
    artifacts[0].exports = vec![last.clone(), unique.clone(), unrelated, first.clone()];
    let mut foreign = unique.clone();
    foreign.origin_id = "foreign-owner".into();
    artifacts[1].exports = vec![foreign.clone()];
    summaries[0].exported_labels[0].owner_fully_qualified_name =
        unique.fully_qualified_name.clone();
    summaries[0].exported_labels[1].owner_fully_qualified_name = first.fully_qualified_name.clone();
    let mut unmatched = summaries[0].exported_labels[0].clone();
    unmatched.origin_id = "unmatched-label".into();
    unmatched.owner_fully_qualified_name = "caller::unmatched".into();
    summaries[0].exported_labels.push(unmatched);
    summaries[1].exported_labels = summaries[0].exported_labels.clone();
    for (index, ((entry, artifact), summary)) in manifest
        .modules
        .iter_mut()
        .zip(&mut artifacts)
        .zip(&mut summaries)
        .enumerate()
    {
        artifact.module.module_path = "same_path".into();
        artifact.module.package_version = Some(format!("1.{index}"));
        summary.module = artifact.module.clone();
        entry.module = artifact.module.clone();
        artifact.refresh_hashes().unwrap();
        let json = if index == 0 {
            verified_artifact_json(artifact)
        } else {
            builtin_verified_artifact_json(artifact)
        }
        .unwrap();
        let write = write_published_artifact(
            &root,
            &PublishedArtifactPath::new(&entry.artifact_file).unwrap(),
            &json,
            &artifact_hash_domain(VERIFIED_ARTIFACT_SCHEMA_FAMILY, artifact.schema_version),
            &artifact_hash_excluded_paths(),
        )
        .unwrap();
        entry.artifact_hash.digest = write.artifact_hash;
        entry.interface_hash.digest = artifact.interface_hash;
        entry.implementation_hash.digest = artifact.implementation_hash;
        summary.refresh_interface_hash().unwrap();
        let write = write_published_artifact(
            &root,
            &PublishedArtifactPath::new(entry.module_summary_file.as_ref().unwrap()).unwrap(),
            &module_summary_json(summary).unwrap(),
            &artifact_hash_domain(MODULE_SUMMARY_SCHEMA_FAMILY, summary.schema_version),
            &[],
        )
        .unwrap();
        entry.module_summary_hash.as_mut().unwrap().digest = write.artifact_hash;
        entry.module_summary_interface_hash.as_mut().unwrap().digest = summary.interface_hash;
    }
    let hash = write_manifest_file(&root, &manifest).unwrap().artifact_hash;
    let selected = read_manifest_file(&root, ManifestFileReadOptions::default()).unwrap();
    let paths = std::iter::once("artifact-manifest.json".to_owned())
        .chain(manifest.modules.iter().flat_map(|entry| {
            std::iter::once(entry.artifact_file.clone())
                .chain(entry.module_summary_file.clone())
                .chain(entry.proof_witnesses.iter().map(|w| w.witness_path.clone()))
        }))
        .collect::<Vec<_>>();
    let before = paths
        .iter()
        .map(|p| std::fs::read(root.join(p)).unwrap())
        .collect::<Vec<_>>();
    assert_eq!(
        summaries[0].module.module_path,
        summaries[1].module.module_path
    );
    assert_ne!(summaries[0].module, summaries[1].module);
    assert!(
        summaries[0]
            .exported_labels
            .iter()
            .all(|label| label.label == "same-label")
    );
    for _ in 0..2 {
        for (module, label, expected) in [
            (
                &summaries[0].module,
                &summaries[0].exported_labels[0],
                vec![unique.clone()],
            ),
            (
                &summaries[0].module,
                &summaries[0].exported_labels[1],
                vec![first.clone(), last.clone()],
            ),
            (
                &summaries[0].module,
                &summaries[0].exported_labels[2],
                vec![],
            ),
            (
                &summaries[1].module,
                &summaries[1].exported_labels[0],
                vec![foreign.clone()],
            ),
            (
                &summaries[1].module,
                &summaries[1].exported_labels[1],
                vec![],
            ),
        ] {
            assert_eq!(
                read(&root, hash, module, label).unwrap(),
                (selected.clone(), Some(expected))
            );
        }
    }
    for field in 0..6 {
        let mut forged = summaries[0].exported_labels[0].clone();
        match field {
            0 => forged.origin_id.push_str("-forged"),
            1 => forged.label.push_str("-forged"),
            2 => forged.owner_fully_qualified_name = first.fully_qualified_name.clone(),
            3 => forged.visibility = "private".into(),
            4 => forged.source_range.end_byte += 1,
            5 => forged.target_kind.push_str("-forged"),
            _ => unreachable!(),
        }
        assert_eq!(
            read(&root, hash, &summaries[0].module, &forged).unwrap(),
            (selected.clone(), None)
        );
    }
    assert_eq!(
        paths
            .iter()
            .map(|p| std::fs::read(root.join(p)).unwrap())
            .collect::<Vec<_>>(),
        before
    );
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn doc_label_export_candidates_absences_and_original_errors_before_lookup() {
    use mizar_artifact::{manifest::*, module_summary::*, store::*};
    use mizar_doc::doc_build::{
        read_published_export_index, read_published_label_export_candidates as read,
    };
    let (root, manifest, summaries) = doc_module_summaries_fixture();
    let label = &summaries[0].exported_labels[0];
    let module = &summaries[0].module;
    let mut missing = module.clone();
    missing.package_version = Some("missing-coordinate".into());
    let mut forged = label.clone();
    forged.origin_id.push_str("-forged");
    let paths = std::iter::once("artifact-manifest.json".to_owned())
        .chain(manifest.modules.iter().flat_map(|entry| {
            std::iter::once(entry.artifact_file.clone())
                .chain(entry.module_summary_file.clone())
                .chain(entry.proof_witnesses.iter().map(|w| w.witness_path.clone()))
        }))
        .collect::<Vec<_>>();
    for case in 0..7 {
        let mut changed = manifest.clone();
        let mut summary = summaries[0].clone();
        let wrong = mizar_session::hash_text("caller-selected mismatch");
        match case {
            2 => summary.module.module_path = "other_module".into(),
            3 => summary.source_hash = wrong,
            4 => summary.exported_labels.clear(),
            5 => {
                changed.modules[0].module_summary_file = None;
                changed.modules[0].module_summary_hash = None;
                changed.modules[0].module_summary_interface_hash = None;
            }
            6 => changed.modules.clear(),
            _ => {}
        }
        summary.refresh_interface_hash().unwrap();
        let path =
            PublishedArtifactPath::new(manifest.modules[0].module_summary_file.as_ref().unwrap())
                .unwrap();
        let write = write_published_artifact(
            &root,
            &path,
            &module_summary_json(&summary).unwrap(),
            &artifact_hash_domain(MODULE_SUMMARY_SCHEMA_FAMILY, summary.schema_version),
            &[],
        )
        .unwrap();
        if case != 5 && case != 6 {
            changed.modules[0]
                .module_summary_hash
                .as_mut()
                .unwrap()
                .digest = write.artifact_hash;
            changed.modules[0]
                .module_summary_interface_hash
                .as_mut()
                .unwrap()
                .digest = summary.interface_hash;
        }
        let published = write_manifest_file(&root, &changed).unwrap().artifact_hash;
        let hash = if case == 1 { wrong } else { published };
        let before = paths
            .iter()
            .map(|p| std::fs::read(root.join(p)).unwrap())
            .collect::<Vec<_>>();
        if (1..=3).contains(&case) {
            let original = read_published_export_index(&root, hash).unwrap_err();
            match (case, &original) {
                (1, ManifestError::StoreIo(StoreIoError::ArtifactHashMismatch { .. })) => {}
                (2, ManifestError::ModuleSummary { error, .. }) => assert!(matches!(
                    error.as_ref(),
                    ModuleSummaryError::ModuleIdentityMismatch { .. }
                )),
                (3, ManifestError::ReferencedArtifactMismatch { field, .. }) => {
                    assert_eq!(field, "source_hash")
                }
                _ => panic!("unexpected original error for case {case}: {original:?}"),
            }
            for (module, label) in [(&missing, label), (module, &forged)] {
                assert_eq!(read(&root, hash, module, label).unwrap_err(), original);
            }
        } else {
            let selected = read_manifest_file(&root, ManifestFileReadOptions::default()).unwrap();
            assert_eq!(
                read(&root, hash, module, label).unwrap(),
                (
                    selected.clone(),
                    if case == 0 { Some(vec![]) } else { None }
                )
            );
            for (module, label) in [(&missing, label), (module, &forged)] {
                assert_eq!(
                    read(&root, hash, module, label).unwrap(),
                    (selected.clone(), None)
                );
            }
        }
        assert_eq!(
            paths
                .iter()
                .map(|p| std::fs::read(root.join(p)).unwrap())
                .collect::<Vec<_>>(),
            before
        );
    }
    std::fs::remove_dir_all(root).unwrap();
}

fn doc_build_case_fixture(
    package_id: &str,
) -> (
    PathBuf,
    mizar_artifact::manifest::ArtifactManifest,
    Vec<mizar_artifact::verified_artifact::VerifiedArtifact>,
    mizar_session::Hash,
) {
    let (root, mut manifest, mut artifacts) = doc_published_fixture();
    std::fs::create_dir_all(root.join("src")).unwrap();
    std::fs::write(
        root.join("src/source.miz"),
        ":: stored source path fixture\n",
    )
    .unwrap();
    manifest.package.package_id = package_id.into();
    manifest.package.package_version = Some("1.0.0-test".into());
    for artifact in &mut artifacts {
        artifact.module.package_id = package_id.into();
        artifact.module.package_version = Some("1.0.0-test".into());
        artifact.source_file = "src/source.miz".into();
    }
    let hash = doc_export_index_publish(&root, &mut manifest, &mut artifacts);
    (root, manifest, artifacts, hash)
}

fn doc_build_summary_fixture(
    package_id: &str,
    empty_summary: bool,
    duplicate_export: bool,
) -> (
    PathBuf,
    mizar_artifact::manifest::ArtifactManifest,
    Vec<mizar_artifact::verified_artifact::VerifiedArtifact>,
    Vec<mizar_artifact::module_summary::ModuleSummary>,
    mizar_session::Hash,
) {
    use mizar_artifact::{
        manifest::*,
        module_summary::*,
        registration_summary::{ArtifactHashClass, ArtifactHashRef},
        store::*,
    };
    let (root, mut manifest, mut summaries) = doc_module_summaries_fixture();
    std::fs::create_dir_all(root.join("src")).unwrap();
    std::fs::write(
        root.join("src/source.miz"),
        ":: stored source path fixture\n",
    )
    .unwrap();
    let old_hash = read_manifest_file(&root, Default::default())
        .unwrap()
        .artifact_hash;
    let (_, mut artifacts) =
        mizar_doc::artifact_reader::read_published_artifacts(&root, old_hash).unwrap();
    manifest.package.package_id = package_id.into();
    manifest.package.package_version = Some("1.0.0-test".into());
    for (artifact, summary) in artifacts.iter_mut().zip(&mut summaries) {
        artifact.module.package_id = package_id.into();
        artifact.module.package_version = Some("1.0.0-test".into());
        summary.module = artifact.module.clone();
        summary.source_hash = artifact.source_hash;
        if empty_summary {
            summary.exported_symbols.clear();
            summary.exported_labels.clear();
            summary.lexical_summary.contributions.clear();
            summary.reexports.clear();
            summary.dependency_interfaces.clear();
        } else if let Some(label) = summary.exported_labels.first_mut() {
            label.owner_fully_qualified_name = artifact.exports[0].fully_qualified_name.clone();
        }
        if duplicate_export {
            let mut duplicate = artifact.exports[0].clone();
            duplicate.origin_id.push_str("-second");
            artifact.exports.push(duplicate);
        }
        summary.refresh_interface_hash().unwrap();
        *summary = read_module_summary(&module_summary_json(summary).unwrap(), Default::default())
            .unwrap();
    }
    doc_export_index_publish(&root, &mut manifest, &mut artifacts);
    for (index, (entry, summary)) in manifest.modules.iter_mut().zip(&summaries).enumerate() {
        let path = PublishedArtifactPath::new(format!("summaries/doc_build_{index}.json")).unwrap();
        let write = write_published_artifact(
            &root,
            &path,
            &module_summary_json(summary).unwrap(),
            &artifact_hash_domain(MODULE_SUMMARY_SCHEMA_FAMILY, summary.schema_version),
            &[],
        )
        .unwrap();
        entry.module_summary_file = Some(path.as_str().into());
        entry.module_summary_hash = Some(ArtifactHashRef::new(
            ArtifactHashClass::Artifact,
            MODULE_SUMMARY_SCHEMA_FAMILY,
            summary.schema_version,
            write.artifact_hash,
        ));
        entry.module_summary_interface_hash = Some(ArtifactHashRef::new(
            ArtifactHashClass::Interface,
            MODULE_SUMMARY_SCHEMA_FAMILY,
            summary.schema_version,
            summary.interface_hash,
        ));
    }
    let hash = write_manifest_file(&root, &manifest).unwrap().artifact_hash;
    (root, manifest, artifacts, summaries, hash)
}

fn doc_build_registry(
    publications: &[(
        &Path,
        &mizar_artifact::manifest::ArtifactManifest,
        &[mizar_artifact::verified_artifact::VerifiedArtifact],
    )],
) -> (
    mizar_session::SnapshotRegistry,
    mizar_session::BuildRequestId,
    mizar_session::BuildSnapshot,
    mizar_session::WorkspaceRoot,
) {
    use mizar_session::{
        BuildRequestId, DependencyArtifactRef, Edition, InMemorySessionIdAllocator, ModulePath,
        PackageId, SnapshotInput, SourceOrigin, SourceVersion, ToolchainInfo, WorkspaceRoot,
    };
    let ids = InMemorySessionIdAllocator::new();
    let registry = mizar_session::SnapshotRegistry::new();
    let request: BuildRequestId = ids.next_request_id().unwrap();
    let workspace = WorkspaceRoot::new("doc-build-workspace");
    let first_manifest = &publications[0].1;
    let initial_input = SnapshotInput {
        workspace_root: workspace.clone(),
        source_versions: Vec::new(),
        dependency_artifacts: Vec::<DependencyArtifactRef>::new(),
        lockfile_hash: first_manifest.lockfile_hash.digest,
        toolchain: ToolchainInfo::new(first_manifest.toolchain.clone()),
        verifier_config_hash: first_manifest.verifier_config_hash.digest,
    };
    let (initial, _) = registry
        .create_snapshot(request, initial_input.clone())
        .unwrap();
    let mut sources = Vec::new();
    for (root, manifest, artifacts) in publications {
        for (entry, artifact) in manifest.modules.iter().zip(*artifacts) {
            sources.push(SourceVersion {
                source_id: ids.next_source_id(initial.id).unwrap(),
                package_id: PackageId::new(artifact.module.package_id.clone()),
                module_path: ModulePath::new(artifact.module.module_path.clone()),
                normalized_path: mizar_session::normalize_path(
                    root,
                    &root.join(&entry.source_file),
                )
                .unwrap(),
                source_hash: entry.source_hash,
                edition: Edition::new(artifact.module.language_edition.clone()),
                origin: SourceOrigin::Disk,
            });
        }
    }
    let input = SnapshotInput {
        workspace_root: workspace.clone(),
        source_versions: sources,
        dependency_artifacts: Vec::<DependencyArtifactRef>::new(),
        lockfile_hash: first_manifest.lockfile_hash.digest,
        toolchain: ToolchainInfo::new(first_manifest.toolchain.clone()),
        verifier_config_hash: first_manifest.verifier_config_hash.digest,
    };
    let (snapshot, _) = registry.create_snapshot(request, input).unwrap();
    (registry, request, snapshot, workspace)
}

#[test]
fn doc_build_switches_preserve_independent_policies_and_publication_bytes() {
    use mizar_doc::doc_build::*;
    let (root, manifest, artifacts, hash) = doc_build_case_fixture("doc-switches");
    let paths = std::iter::once("artifact-manifest.json".to_owned())
        .chain(
            manifest
                .modules
                .iter()
                .map(|entry| entry.artifact_file.clone()),
        )
        .collect::<Vec<_>>();
    let before = paths
        .iter()
        .map(|path| std::fs::read(root.join(path)).unwrap())
        .collect::<Vec<_>>();
    let publications = [(root.as_path(), &manifest, artifacts.as_slice())];
    let (registry, build_request, snapshot, workspace) = doc_build_registry(&publications);
    let selected = DocPublicationRequest {
        package: manifest.package.clone(),
        artifact_root: root.clone(),
        expected_manifest_hash: hash,
        output_dir: root.join("output"),
        source_bundle: None,
        dependency_indexes: None,
        import_order: BTreeMap::new(),
    };
    let mut plans = Vec::new();
    for flags in [
        [false, false, false, false, false],
        [true, false, false, false, false],
        [true, true, false, false, false],
        [true, false, true, false, false],
        [true, false, false, true, false],
        [true, false, false, false, true],
        [false, true, true, true, true],
    ] {
        let profile = Phase16Profile {
            render_docs: flags[0],
            extract_code: flags[1],
            fail_on_doc_warnings: flags[2],
            fail_on_extraction_warnings: flags[3],
            include_private_docs: flags[4],
        };
        let selected = if profile.render_docs {
            selected.clone()
        } else {
            DocPublicationRequest {
                artifact_root: root.join("missing-publication"),
                expected_manifest_hash: mizar_session::hash_text("missing"),
                ..selected.clone()
            }
        };
        let request = DocBuildRequest {
            profile: profile.clone(),
            publications: vec![selected],
        };
        let result = build_doc_plans(&registry, build_request, snapshot.id, &workspace, &request);
        if profile.render_docs {
            let plan = result.unwrap().pop().unwrap();
            assert_eq!(plan.profile, profile);
            plans.push(plan);
        } else {
            assert!(result.unwrap().is_empty());
        }
    }
    for plan in &plans[1..] {
        assert_eq!(plans[0].request, plan.request);
        assert_eq!(plans[0].index, plan.index);
        assert_eq!(plans[0].manifest, plan.manifest);
    }
    assert_eq!(
        std::iter::once("artifact-manifest.json".to_owned())
            .chain(
                manifest
                    .modules
                    .iter()
                    .map(|entry| entry.artifact_file.clone())
            )
            .map(|path| std::fs::read(root.join(path)).unwrap())
            .collect::<Vec<_>>(),
        before
    );
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn doc_build_keeps_package_module_and_empty_module_identities() {
    use mizar_artifact::manifest::PackageIdentity;
    use mizar_doc::doc_build::*;
    let (left_root, mut left_manifest, mut left_artifacts, _) = doc_build_case_fixture("doc-left");
    let (right_root, right_manifest, right_artifacts, right_summaries, right_hash) =
        doc_build_summary_fixture("doc-right", false, false);
    left_artifacts[0].exports.clear();
    let left_hash = doc_export_index_publish(&left_root, &mut left_manifest, &mut left_artifacts);
    let publications = [
        (
            left_root.as_path(),
            &left_manifest,
            left_artifacts.as_slice(),
        ),
        (
            right_root.as_path(),
            &right_manifest,
            right_artifacts.as_slice(),
        ),
    ];
    let (registry, build_request, snapshot, workspace) = doc_build_registry(&publications);
    let dependency = DependencyDocIndexRef {
        package: right_manifest.package.clone(),
        publication_hash: right_hash,
        path: right_root.join("dependency-index.json"),
    };
    let selected =
        |package: PackageIdentity, root: &Path, hash, dependency_indexes| DocPublicationRequest {
            package,
            artifact_root: root.into(),
            expected_manifest_hash: hash,
            output_dir: root.join("out"),
            source_bundle: Some(root.join("unavailable-source-bundle")),
            dependency_indexes,
            import_order: BTreeMap::new(),
        };
    let request = DocBuildRequest {
        profile: Phase16Profile {
            render_docs: true,
            extract_code: true,
            fail_on_doc_warnings: false,
            fail_on_extraction_warnings: false,
            include_private_docs: true,
        },
        publications: vec![
            selected(
                left_manifest.package.clone(),
                &left_root,
                left_hash,
                Some(vec![dependency.clone()]),
            ),
            selected(
                right_manifest.package.clone(),
                &right_root,
                right_hash,
                None,
            ),
        ],
    };
    let plans =
        build_doc_plans(&registry, build_request, snapshot.id, &workspace, &request).unwrap();
    assert_eq!(plans.len(), 2);
    for (plan, selected) in plans.iter().zip(&request.publications) {
        assert_eq!(&plan.request, selected);
        assert_eq!(plan.profile, request.profile);
        assert_eq!(plan.snapshot, snapshot.id);
        assert_eq!(plan.build_request, build_request);
        assert_eq!(
            plan.manifest,
            mizar_artifact::manifest::read_manifest_file(
                &selected.artifact_root,
                Default::default()
            )
            .unwrap()
        );
        assert_eq!(
            Some(&plan.index.package_version),
            plan.manifest.manifest.package.package_version.as_ref()
        );
    }
    for (plan, entries, source_artifacts) in [
        (&plans[0], &left_manifest.modules, &left_artifacts),
        (&plans[1], &right_manifest.modules, &right_artifacts),
    ] {
        assert_eq!(plan.index.modules.len(), 2);
        assert!(
            plan.index
                .modules
                .keys()
                .all(|identity| identity.package_id == plan.request.package.package_id)
        );
        let module_paths = plan
            .index
            .modules
            .keys()
            .map(|identity| identity.module_path.as_str())
            .collect::<Vec<_>>();
        assert_eq!(module_paths, vec!["module_0", "module_1"]);
        for entry in entries {
            let module = &plan.index.modules[&entry.module];
            assert_eq!(&module.artifact, entry);
            assert_eq!(
                module.summary.as_ref(),
                right_summaries
                    .iter()
                    .find(|summary| summary.module == entry.module)
            );
            let mut expected = source_artifacts
                .iter()
                .find(|artifact| artifact.module == entry.module)
                .unwrap()
                .exports
                .clone();
            expected.sort_by(|a, b| {
                a.fully_qualified_name
                    .cmp(&b.fully_qualified_name)
                    .then_with(|| a.origin_id.cmp(&b.origin_id))
            });
            assert_eq!(module.exports, expected);
        }
    }
    assert!(
        plans[0]
            .index
            .modules
            .values()
            .any(|module| module.exports.is_empty())
    );
    std::fs::remove_dir_all(left_root).unwrap();
    std::fs::remove_dir_all(right_root).unwrap();
}

#[test]
fn doc_build_canonicalizes_enumeration_and_preserves_import_order() {
    use mizar_doc::doc_build::*;
    let (one_root, one_manifest, one_artifacts, one_hash) = doc_build_case_fixture("doc-order");
    let (two_root, mut two_manifest, mut two_artifacts, _) = doc_build_case_fixture("doc-order");
    two_manifest.package.package_version = Some("2.0.0-test".into());
    for artifact in &mut two_artifacts {
        artifact.module.package_version = Some("2.0.0-test".into());
    }
    doc_export_index_publish(&two_root, &mut two_manifest, &mut two_artifacts);
    two_manifest.modules.reverse();
    two_artifacts.reverse();
    let two_hash = mizar_artifact::manifest::write_manifest_file(&two_root, &two_manifest)
        .unwrap()
        .artifact_hash;
    let publications = [(one_root.as_path(), &one_manifest, one_artifacts.as_slice())];
    let (registry, build_request, snapshot, workspace) = doc_build_registry(&publications);
    let module_ids = one_manifest
        .modules
        .iter()
        .map(|entry| entry.module.clone())
        .collect::<Vec<_>>();
    let imports = vec![module_ids[1].clone(), module_ids[0].clone()];
    let dependency = |package: &mizar_artifact::manifest::ArtifactManifest, hash, root: &Path| {
        DependencyDocIndexRef {
            package: package.package.clone(),
            publication_hash: hash,
            path: root.join("dep.json"),
        }
    };
    let mut dependencies = vec![
        dependency(&one_manifest, one_hash, &one_root),
        dependency(&two_manifest, two_hash, &two_root),
    ];
    let mut other_path = dependencies[0].clone();
    other_path.path = one_root.join("another-index.json");
    dependencies.push(other_path);
    let mut other_publication = dependencies[0].clone();
    other_publication.publication_hash = mizar_session::Hash::from_bytes([0; 32]);
    other_publication.path = one_root.join("zz-index.json");
    dependencies.push(other_publication.clone());
    let mut other_lock = other_publication.clone();
    other_lock.package.lockfile_identity = Some("opaque-lock".into());
    dependencies.push(other_lock.clone());
    let selected = |manifest: &mizar_artifact::manifest::ArtifactManifest,
                    hash,
                    root: &Path,
                    deps| DocPublicationRequest {
        package: manifest.package.clone(),
        artifact_root: root.into(),
        expected_manifest_hash: hash,
        output_dir: root.join("out"),
        source_bundle: None,
        dependency_indexes: Some(deps),
        import_order: BTreeMap::from([(module_ids[0].clone(), Some(imports.clone()))]),
    };
    let profile = Phase16Profile {
        render_docs: true,
        extract_code: true,
        fail_on_doc_warnings: true,
        fail_on_extraction_warnings: false,
        include_private_docs: false,
    };
    let forward = DocBuildRequest {
        profile: profile.clone(),
        publications: vec![
            selected(&one_manifest, one_hash, &one_root, dependencies.clone()),
            selected(&two_manifest, two_hash, &two_root, dependencies.clone()),
        ],
    };
    let mut reverse_deps = dependencies.clone();
    reverse_deps.reverse();
    let reverse = DocBuildRequest {
        profile,
        publications: vec![
            selected(&two_manifest, two_hash, &two_root, reverse_deps.clone()),
            selected(&one_manifest, one_hash, &one_root, reverse_deps),
        ],
    };
    let first =
        build_doc_plans(&registry, build_request, snapshot.id, &workspace, &forward).unwrap();
    let registry = std::sync::Arc::new(registry);
    let second = std::thread::spawn({
        let registry = std::sync::Arc::clone(&registry);
        let reverse = reverse.clone();
        let workspace = workspace.clone();
        move || {
            build_doc_plans(&registry, build_request, snapshot.id, &workspace, &reverse).unwrap()
        }
    })
    .join()
    .unwrap();
    assert_eq!(first, second);
    assert_eq!(second[0].index.package_version, "1.0.0-test");
    assert_eq!(second[1].index.package_version, "2.0.0-test");
    for plan in &second {
        assert_eq!(
            plan.request.import_order.get(&module_ids[0]),
            Some(&Some(imports.clone()))
        );
        let refs = plan.request.dependency_indexes.as_ref().unwrap();
        assert_eq!(refs[0], other_publication);
        assert_eq!(refs[1].path, one_root.join("another-index.json"));
        assert_eq!(refs[2].path, one_root.join("dep.json"));
        assert_eq!(refs[3], other_lock);
        assert_eq!(refs[4].package, two_manifest.package);
        let module_paths = plan
            .index
            .modules
            .keys()
            .map(|identity| identity.module_path.as_str())
            .collect::<Vec<_>>();
        assert_eq!(module_paths, vec!["module_0", "module_1"]);
    }
    std::fs::remove_dir_all(one_root).unwrap();
    std::fs::remove_dir_all(two_root).unwrap();
}

#[test]
fn doc_build_refuses_missing_required_metadata_without_defaults() {
    use mizar_artifact::{manifest::*, store::*, verified_artifact::*};
    use mizar_doc::doc_build::*;
    for case in 0..4 {
        let (root, mut manifest, mut artifacts, _) = doc_build_case_fixture("doc-required");
        let source_publications = [(root.as_path(), &manifest, artifacts.as_slice())];
        let (registry, build_request, snapshot, workspace) =
            doc_build_registry(&source_publications);
        match case {
            0 => manifest.package.package_version = None,
            1 => {
                artifacts[0].module.package_version = None;
                doc_export_index_publish(&root, &mut manifest, &mut artifacts);
            }
            _ => {
                let entry = &mut manifest.modules[0];
                let path = PublishedArtifactPath::new(&entry.artifact_file).unwrap();
                let mut json = parse_canonical_json_bytes(
                    &std::fs::read(root.join(path.as_str())).unwrap(),
                    path.as_str(),
                )
                .unwrap();
                let CanonicalJson::Object(fields) = &mut json else {
                    unreachable!()
                };
                if case == 2 {
                    fields.remove("source_file");
                } else {
                    let CanonicalJson::Array(exports) = fields.get_mut("exports").unwrap() else {
                        unreachable!()
                    };
                    let CanonicalJson::Object(export) = &mut exports[0] else {
                        unreachable!()
                    };
                    export.remove("source_range");
                }
                entry.artifact_hash.digest = write_published_artifact(
                    &root,
                    &path,
                    &json,
                    &artifact_hash_domain(
                        VERIFIED_ARTIFACT_SCHEMA_FAMILY,
                        entry.artifact_hash.schema_version,
                    ),
                    &artifact_hash_excluded_paths(),
                )
                .unwrap()
                .artifact_hash;
            }
        }
        let hash = write_manifest_file(&root, &manifest).unwrap().artifact_hash;
        let package = manifest.package.clone();
        let request = DocBuildRequest {
            profile: Phase16Profile {
                render_docs: true,
                extract_code: false,
                fail_on_doc_warnings: false,
                fail_on_extraction_warnings: false,
                include_private_docs: false,
            },
            publications: vec![DocPublicationRequest {
                package,
                artifact_root: root.clone(),
                expected_manifest_hash: hash,
                output_dir: root.join("output"),
                source_bundle: None,
                dependency_indexes: None,
                import_order: BTreeMap::new(),
            }],
        };
        assert!(
            build_doc_plans(&registry, build_request, snapshot.id, &workspace, &request).is_err(),
            "case {case}"
        );
        std::fs::remove_dir_all(root).unwrap();
    }
    let (plain_root, plain_manifest, plain_artifacts, plain_hash) =
        doc_build_case_fixture("doc-summary-states");
    let (empty_root, empty_manifest, empty_artifacts, _, empty_hash) =
        doc_build_summary_fixture("doc-summary-states", true, false);
    for (root, manifest, hash) in [
        (&plain_root, &plain_manifest, plain_hash),
        (&empty_root, &empty_manifest, empty_hash),
    ] {
        let publications = [(
            root.as_path(),
            manifest,
            if root == &plain_root {
                plain_artifacts.as_slice()
            } else {
                empty_artifacts.as_slice()
            },
        )];
        let (registry, build_request, snapshot, workspace) = doc_build_registry(&publications);
        let request = DocBuildRequest {
            profile: Phase16Profile {
                render_docs: true,
                extract_code: false,
                fail_on_doc_warnings: false,
                fail_on_extraction_warnings: false,
                include_private_docs: false,
            },
            publications: vec![DocPublicationRequest {
                package: manifest.package.clone(),
                artifact_root: root.clone(),
                expected_manifest_hash: hash,
                output_dir: root.join("out"),
                source_bundle: None,
                dependency_indexes: None,
                import_order: BTreeMap::new(),
            }],
        };
        let plan = build_doc_plans(&registry, build_request, snapshot.id, &workspace, &request)
            .unwrap()
            .remove(0);
        if root == &plain_root {
            assert!(
                plan.index
                    .modules
                    .values()
                    .all(|module| module.summary.is_none())
            );
        } else {
            assert!(plan.index.modules.values().all(|module| {
                module
                    .summary
                    .as_ref()
                    .is_some_and(|summary| summary.exported_labels.is_empty())
            }));
        }
    }
    std::fs::remove_dir_all(plain_root).unwrap();
    std::fs::remove_dir_all(empty_root).unwrap();
}

#[test]
fn doc_build_refuses_tampering_mismatches_and_partial_plans() {
    use mizar_artifact::manifest::*;
    use mizar_doc::doc_build::*;
    let (root, manifest, artifacts, hash) = doc_build_case_fixture("doc-refusal");
    let publications = [(root.as_path(), &manifest, artifacts.as_slice())];
    let (registry, build_request, snapshot, workspace) = doc_build_registry(&publications);
    let selected = DocPublicationRequest {
        package: manifest.package.clone(),
        artifact_root: root.clone(),
        expected_manifest_hash: hash,
        output_dir: root.join("out"),
        source_bundle: None,
        dependency_indexes: None,
        import_order: BTreeMap::new(),
    };
    let request = |publications| DocBuildRequest {
        profile: Phase16Profile {
            render_docs: true,
            extract_code: false,
            fail_on_doc_warnings: false,
            fail_on_extraction_warnings: false,
            include_private_docs: false,
        },
        publications,
    };
    assert!(matches!(
        build_doc_plans(
            &registry,
            build_request,
            snapshot.id,
            &workspace,
            &request(vec![DocPublicationRequest {
                expected_manifest_hash: mizar_session::hash_text("wrong"),
                ..selected.clone()
            }])
        ),
        Err(DocBuildError::Publication(_))
    ));
    assert!(matches!(
        build_doc_plans(
            &registry,
            build_request,
            snapshot.id,
            &mizar_session::WorkspaceRoot::new("other-workspace"),
            &request(vec![selected.clone()])
        ),
        Err(DocBuildError::Binding(_))
    ));
    assert!(matches!(
        build_doc_plans(
            &registry,
            build_request,
            snapshot.id,
            &workspace,
            &request(vec![DocPublicationRequest {
                package: PackageIdentity {
                    package_id: "wrong-package".into(),
                    ..manifest.package.clone()
                },
                ..selected.clone()
            }])
        ),
        Err(DocBuildError::Binding(_))
    ));
    let (bad_root, mut bad_manifest, mut bad_artifacts, _) = doc_build_case_fixture("doc-refusal");
    bad_artifacts[0].source_hash = mizar_session::hash_text("different source");
    bad_artifacts[0].refresh_hashes().unwrap();
    doc_export_index_publish(&bad_root, &mut bad_manifest, &mut bad_artifacts);
    bad_manifest.modules[0].source_hash = bad_artifacts[0].source_hash;
    let bad_hash = write_manifest_file(&bad_root, &bad_manifest)
        .unwrap()
        .artifact_hash;
    let bad = DocPublicationRequest {
        package: bad_manifest.package.clone(),
        artifact_root: bad_root.clone(),
        expected_manifest_hash: bad_hash,
        output_dir: bad_root.join("out"),
        source_bundle: None,
        dependency_indexes: None,
        import_order: BTreeMap::new(),
    };
    assert!(matches!(
        build_doc_plans(
            &registry,
            build_request,
            snapshot.id,
            &workspace,
            &request(vec![selected.clone(), bad])
        ),
        Err(DocBuildError::Binding(_))
    ));

    let (provenance_root, mut provenance_manifest, provenance_artifacts, _) =
        doc_build_case_fixture("doc-refusal");
    provenance_manifest.toolchain = "different-toolchain".into();
    let provenance_hash = write_manifest_file(&provenance_root, &provenance_manifest)
        .unwrap()
        .artifact_hash;
    let provenance = DocPublicationRequest {
        package: provenance_manifest.package.clone(),
        artifact_root: provenance_root.clone(),
        expected_manifest_hash: provenance_hash,
        output_dir: provenance_root.join("out"),
        source_bundle: None,
        dependency_indexes: None,
        import_order: BTreeMap::new(),
    };
    let provenance_publications = [(
        provenance_root.as_path(),
        &provenance_manifest,
        provenance_artifacts.as_slice(),
    )];
    let (provenance_registry, provenance_request, provenance_snapshot, provenance_workspace) =
        doc_build_registry(&provenance_publications);
    assert!(matches!(
        build_doc_plans(
            &provenance_registry,
            provenance_request,
            provenance_snapshot.id,
            &provenance_workspace,
            &request(vec![provenance])
        ),
        Err(DocBuildError::Binding(_))
    ));

    let stale_input = mizar_session::SnapshotInput {
        workspace_root: workspace.clone(),
        source_versions: snapshot.source_versions.clone(),
        dependency_artifacts: snapshot.dependency_artifacts.clone(),
        lockfile_hash: snapshot.lockfile_hash,
        toolchain: snapshot.toolchain.clone(),
        verifier_config_hash: mizar_session::hash_text("changed config"),
    };
    registry
        .create_snapshot(build_request, stale_input)
        .unwrap();
    assert!(matches!(
        build_doc_plans(
            &registry,
            build_request,
            snapshot.id,
            &workspace,
            &request(vec![selected.clone()])
        ),
        Err(DocBuildError::Binding(_))
    ));

    let (empty_root, mut empty_manifest, _, _) = doc_build_case_fixture("doc-empty-package");
    empty_manifest.modules.clear();
    let empty_hash = write_manifest_file(&empty_root, &empty_manifest)
        .unwrap()
        .artifact_hash;
    let no_sources = doc_build_registry(&[(empty_root.as_path(), &empty_manifest, &[])]);
    let empty_request = DocBuildRequest {
        profile: Phase16Profile {
            render_docs: true,
            extract_code: false,
            fail_on_doc_warnings: false,
            fail_on_extraction_warnings: false,
            include_private_docs: false,
        },
        publications: vec![DocPublicationRequest {
            package: empty_manifest.package.clone(),
            artifact_root: empty_root.clone(),
            expected_manifest_hash: empty_hash,
            output_dir: empty_root.join("out"),
            source_bundle: None,
            dependency_indexes: None,
            import_order: BTreeMap::new(),
        }],
    };
    assert!(matches!(
        build_doc_plans(
            &no_sources.0,
            no_sources.1,
            no_sources.2.id,
            &no_sources.3,
            &empty_request
        ),
        Err(DocBuildError::Binding(_))
    ));
    std::fs::remove_dir_all(empty_root).unwrap();
    std::fs::remove_dir_all(bad_root).unwrap();
    std::fs::remove_dir_all(provenance_root).unwrap();
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn doc_build_label_candidates_keep_absence_and_all_exact_matches() {
    use mizar_doc::doc_build::*;
    let (root, manifest, artifacts, summaries, hash) =
        doc_build_summary_fixture("doc-labels", false, true);
    let publications = [(root.as_path(), &manifest, artifacts.as_slice())];
    let (registry, build_request, snapshot, workspace) = doc_build_registry(&publications);
    let module = &manifest.modules[0].module;
    let import_order = BTreeMap::from([
        (manifest.modules[0].module.clone(), None),
        (manifest.modules[1].module.clone(), Some(Vec::new())),
    ]);
    let request = DocBuildRequest {
        profile: Phase16Profile {
            render_docs: true,
            extract_code: false,
            fail_on_doc_warnings: false,
            fail_on_extraction_warnings: false,
            include_private_docs: false,
        },
        publications: vec![DocPublicationRequest {
            package: manifest.package.clone(),
            artifact_root: root.clone(),
            expected_manifest_hash: hash,
            output_dir: root.join("out"),
            source_bundle: None,
            dependency_indexes: None,
            import_order: import_order.clone(),
        }],
    };
    let plan = build_doc_plans(&registry, build_request, snapshot.id, &workspace, &request)
        .unwrap()
        .remove(0);
    assert_eq!(plan.request.dependency_indexes, None);
    assert_eq!(plan.request.import_order, import_order);
    assert_eq!(plan.manifest.manifest.modules, manifest.modules);
    for (entry, original_summary) in manifest.modules.iter().zip(&summaries) {
        let retained = &plan.index.modules[&entry.module];
        assert_eq!(&retained.artifact, entry);
        assert_eq!(retained.summary.as_ref(), Some(original_summary));
    }
    let summary = plan.index.modules[module].summary.as_ref().unwrap();
    let label = summaries[0]
        .exported_labels
        .iter()
        .find(|label| {
            label.owner_fully_qualified_name == artifacts[0].exports[0].fully_qualified_name
        })
        .unwrap();
    let mut expected_candidates = artifacts[0]
        .exports
        .iter()
        .filter(|export| export.fully_qualified_name == label.owner_fully_qualified_name)
        .collect::<Vec<_>>();
    expected_candidates.sort_by(|a, b| {
        a.fully_qualified_name
            .cmp(&b.fully_qualified_name)
            .then_with(|| a.origin_id.cmp(&b.origin_id))
    });
    assert_eq!(
        plan.index.label_export_candidates(module, label),
        Some(expected_candidates)
    );
    let mut forged = label.clone();
    forged.origin_id.push_str("-forged");
    assert_eq!(plan.index.label_export_candidates(module, &forged), None);
    let zero = summaries[0]
        .exported_labels
        .iter()
        .find(|candidate| *candidate != label)
        .unwrap();
    assert_eq!(
        plan.index.label_export_candidates(module, zero),
        Some(Vec::new())
    );
    let mut absent_module = module.clone();
    absent_module.module_path = "absent-module".into();
    assert_eq!(
        plan.index.label_export_candidates(&absent_module, label),
        None
    );
    assert_eq!(summary.exported_labels, summaries[0].exported_labels);
    assert!(
        plan.index
            .modules
            .values()
            .all(|module| module.summary.is_some())
    );
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn doc_native_lexical_literal_trivia_and_annotation_strings() {
    use mizar_doc::artifact_reader::read_native_lexical_source;
    use mizar_lexer::{
        CommentKind::*, CommentTrivia, PreprocessedLexicalSource, SourcePreprocessMap,
        SourcePreprocessMapSegment::*, SourceSpan as Span,
    };
    let (root, manifest, _) = doc_published_fixture();
    let publication_paths = std::iter::once("artifact-manifest.json".to_owned())
        .chain(manifest.modules.iter().map(|m| m.artifact_file.clone()))
        .chain(
            manifest.modules[1]
                .proof_witnesses
                .iter()
                .map(|w| w.witness_path.clone()),
        )
        .collect::<Vec<_>>();
    let before = publication_paths
        .iter()
        .map(|p| std::fs::read(root.join(p)).unwrap())
        .collect::<Vec<_>>();
    let text = "::: λ\n:: n\nx::=b=::y\n@latex(\":::λ\", '::=x=::')\n";
    let path = root.join("src/source.miz");
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(&path, text).unwrap();
    assert_eq!(
        read_native_lexical_source(&root, "src/source.miz", mizar_session::hash_text(text))
            .unwrap(),
        Some((
            text.to_owned(),
            None,
            PreprocessedLexicalSource {
                lexical_text: "\n\nx y\n@latex(\":::λ\", '::=x=::')\n".into(),
                comments: vec![
                    CommentTrivia {
                        kind: Documentation,
                        lexeme: "::: λ\n".into(),
                        span: Span::new(0, 7)
                    },
                    CommentTrivia {
                        kind: SingleLine,
                        lexeme: ":: n\n".into(),
                        span: Span::new(7, 12)
                    },
                    CommentTrivia {
                        kind: MultiLine,
                        lexeme: "::=b=::".into(),
                        span: Span::new(13, 20)
                    },
                ],
                diagnostics: vec![],
                preprocess_map: SourcePreprocessMap {
                    segments: vec![
                        RemovedComment {
                            source: Span::new(0, 7),
                            kind: Documentation
                        },
                        SyntheticWhitespace {
                            lexical: Span::new(0, 1),
                            anchor: Span::new(6, 7)
                        },
                        RemovedComment {
                            source: Span::new(7, 12),
                            kind: SingleLine
                        },
                        SyntheticWhitespace {
                            lexical: Span::new(1, 2),
                            anchor: Span::new(11, 12)
                        },
                        Original {
                            lexical: Span::new(2, 3),
                            source: Span::new(12, 13)
                        },
                        RemovedComment {
                            source: Span::new(13, 20),
                            kind: MultiLine
                        },
                        SyntheticWhitespace {
                            lexical: Span::new(3, 4),
                            anchor: Span::new(13, 20)
                        },
                        Original {
                            lexical: Span::new(4, 33),
                            source: Span::new(20, 49)
                        },
                    ]
                },
            }
        ))
    );
    assert_eq!(std::fs::read(&path).unwrap(), text.as_bytes());
    assert_eq!(
        publication_paths
            .iter()
            .map(|p| std::fs::read(root.join(p)).unwrap())
            .collect::<Vec<_>>(),
        before
    );
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn doc_native_lexical_normalized_unicode_and_both_map_frames() {
    use mizar_doc::artifact_reader::read_native_lexical_source;
    use mizar_lexer::{
        CommentKind::*, CommentTrivia, PreprocessedLexicalSource, SourcePreprocessDiagnostic,
        SourcePreprocessDiagnosticCode, SourcePreprocessDiagnosticPayload, SourcePreprocessMap,
        SourcePreprocessMapSegment::*, SourceSpan as Span,
    };
    use mizar_session::source_map::{LoadingMapSegment, TextRange};
    let (root, _, _) = doc_published_fixture();
    let raw = "\u{feff}\u{feff}::: λ\r\nx::=μ=::y\r\n";
    let normalized = "\u{feff}::: λ\nx::=μ=::y\n";
    let path = root.join("src/source.miz");
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(&path, raw).unwrap();
    assert_eq!(
        read_native_lexical_source(&root, &path, mizar_session::hash_text(normalized)).unwrap(),
        Some((
            normalized.to_owned(),
            Some(vec![
                LoadingMapSegment::RemovedLeadingBom {
                    original: TextRange { start: 0, end: 3 }
                },
                LoadingMapSegment::Original {
                    loaded: TextRange { start: 0, end: 9 },
                    original: TextRange { start: 3, end: 12 }
                },
                LoadingMapSegment::NormalizedNewline {
                    loaded: TextRange { start: 9, end: 10 },
                    original: TextRange { start: 12, end: 14 }
                },
                LoadingMapSegment::Original {
                    loaded: TextRange { start: 10, end: 20 },
                    original: TextRange { start: 14, end: 24 }
                },
                LoadingMapSegment::NormalizedNewline {
                    loaded: TextRange { start: 20, end: 21 },
                    original: TextRange { start: 24, end: 26 }
                },
            ]),
            PreprocessedLexicalSource {
                lexical_text: "\u{feff}\nx y\n".into(),
                comments: vec![
                    CommentTrivia {
                        kind: Documentation,
                        lexeme: "::: λ\n".into(),
                        span: Span::new(3, 10)
                    },
                    CommentTrivia {
                        kind: MultiLine,
                        lexeme: "::=μ=::".into(),
                        span: Span::new(11, 19)
                    },
                ],
                diagnostics: vec![SourcePreprocessDiagnostic {
                    code: SourcePreprocessDiagnosticCode::NonAsciiCode,
                    message: "code regions must be ASCII before lexing".into(),
                    span: Span::new(0, 3),
                    payload: SourcePreprocessDiagnosticPayload::NonAsciiCode {
                        character: '\u{feff}',
                        utf8_len: 3,
                    },
                }],
                preprocess_map: SourcePreprocessMap {
                    segments: vec![
                        Original {
                            lexical: Span::new(0, 3),
                            source: Span::new(0, 3)
                        },
                        RemovedComment {
                            source: Span::new(3, 10),
                            kind: Documentation
                        },
                        SyntheticWhitespace {
                            lexical: Span::new(3, 4),
                            anchor: Span::new(9, 10)
                        },
                        Original {
                            lexical: Span::new(4, 5),
                            source: Span::new(10, 11)
                        },
                        RemovedComment {
                            source: Span::new(11, 19),
                            kind: MultiLine
                        },
                        SyntheticWhitespace {
                            lexical: Span::new(5, 6),
                            anchor: Span::new(11, 19)
                        },
                        Original {
                            lexical: Span::new(6, 8),
                            source: Span::new(19, 21)
                        },
                    ]
                },
            }
        ))
    );
    assert_eq!(std::fs::read(&path).unwrap(), raw.as_bytes());
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn doc_native_lexical_delegated_errors_edit_restore_and_complete_diagnostics() {
    use mizar_doc::artifact_reader::read_native_lexical_source;
    use mizar_lexer::{
        CommentKind, CommentTrivia, PreprocessedLexicalSource, SourcePreprocessDiagnostic,
        SourcePreprocessDiagnosticCode as Code, SourcePreprocessDiagnosticPayload as Payload,
        SourcePreprocessMap, SourcePreprocessMapSegment::*,
        SourcePreprocessRecoveryHint as Recovery, SourceSpan as Span,
    };
    use mizar_session::{hash_text, source::SourcePathError};
    use std::io::ErrorKind;
    let (root, _, _) = doc_published_fixture();
    let text = "λ\r::=μ\n";
    let expected_hash = hash_text(text);
    let path = root.join("src/source.miz");
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    assert_eq!(
        read_native_lexical_source(&root, &path, expected_hash).unwrap(),
        None
    );
    assert!(!path.exists());
    let missing_root = root.join("absent");
    let error =
        read_native_lexical_source(&missing_root, "src/source.miz", expected_hash).unwrap_err();
    assert_eq!(error.kind(), ErrorKind::NotFound);
    assert_eq!(
        error.get_ref().unwrap().downcast_ref::<SourcePathError>(),
        Some(&SourcePathError::PackageRootUnavailable {
            path: missing_root,
            kind: ErrorKind::NotFound
        })
    );
    std::fs::write(&path, [0xff]).unwrap();
    let error = read_native_lexical_source(&root, &path, expected_hash).unwrap_err();
    assert_eq!(error.kind(), ErrorKind::InvalidData);
    let utf8 = error
        .get_ref()
        .unwrap()
        .downcast_ref::<std::str::Utf8Error>()
        .unwrap();
    assert_eq!((utf8.valid_up_to(), utf8.error_len()), (0, Some(1)));
    assert_eq!(std::fs::read(&path).unwrap(), [0xff]);
    let edited = "λ\r::=edited\n";
    std::fs::write(&path, edited).unwrap();
    let error = read_native_lexical_source(&root, &path, expected_hash).unwrap_err();
    assert_eq!(error.kind(), ErrorKind::InvalidData);
    assert_eq!(error.to_string(), "native source hash mismatch");
    assert_eq!(std::fs::read(&path).unwrap(), edited.as_bytes());
    std::fs::write(&path, text).unwrap();
    assert_eq!(
        read_native_lexical_source(&root, &path, expected_hash).unwrap(),
        Some((
            text.into(),
            None,
            PreprocessedLexicalSource {
                lexical_text: "λ\r\n".into(),
                comments: vec![CommentTrivia {
                    kind: CommentKind::MultiLine,
                    lexeme: "::=μ\n".into(),
                    span: Span::new(3, 9)
                }],
                diagnostics: vec![
                    SourcePreprocessDiagnostic {
                        code: Code::NonAsciiCode,
                        message: "code regions must be ASCII before lexing".into(),
                        span: Span::new(0, 2),
                        payload: Payload::NonAsciiCode {
                            character: 'λ',
                            utf8_len: 2
                        }
                    },
                    SourcePreprocessDiagnostic {
                        code: Code::CarriageReturn,
                        message: "source text must be LF-only before lexing".into(),
                        span: Span::new(2, 3),
                        payload: Payload::CarriageReturn {
                            recovery: Recovery::NormalizeCrLfBeforeLexerEntry
                        }
                    },
                    SourcePreprocessDiagnostic {
                        code: Code::UnterminatedMultiLineComment,
                        message: "unterminated multi-line comment".into(),
                        span: Span::new(3, 9),
                        payload: Payload::UnterminatedMultiLineComment {
                            opener: Span::new(3, 6),
                            recovery: Recovery::PreserveNewlinesAndDropCommentText
                        }
                    },
                ],
                preprocess_map: SourcePreprocessMap {
                    segments: vec![
                        Original {
                            lexical: Span::new(0, 3),
                            source: Span::new(0, 3)
                        },
                        RemovedComment {
                            source: Span::new(3, 9),
                            kind: CommentKind::MultiLine
                        },
                        SyntheticWhitespace {
                            lexical: Span::new(3, 4),
                            anchor: Span::new(8, 9)
                        },
                    ]
                },
            }
        ))
    );
    assert_eq!(std::fs::read(&path).unwrap(), text.as_bytes());
    std::fs::remove_dir_all(root).unwrap();
}
