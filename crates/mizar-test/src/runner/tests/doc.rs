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

fn doc_q1_publish(
    root: &Path,
    manifest: &mut mizar_artifact::manifest::ArtifactManifest,
    artifacts: &mut [mizar_artifact::verified_artifact::VerifiedArtifact],
    index: usize,
    text: Option<&str>,
) -> (mizar_artifact::store::CanonicalJson, mizar_session::Hash) {
    use mizar_artifact::{
        comment_projection::*, manifest::write_manifest_file, module_summary::SourceRangeSummary,
        store::*,
    };
    let artifact = &mut artifacts[index];
    artifact.schema_version = SchemaVersion::new(1, 2);
    artifact.source_hash = mizar_session::hash_text(text.unwrap_or("unavailable comments"));
    artifact.provenance.captured_source = Some((
        mizar_session::SourceId::from_nonportable_provenance_str(
            "mizar-session-source-nonportable-v1:0000000000000001",
        )
        .unwrap(),
        mizar_session::SourceOrigin::Disk,
        artifact.source_hash,
    ));
    let zero = SourceRangeSummary {
        start_byte: 0,
        end_byte: 0,
    };
    for export in &mut artifact.exports {
        export.source_range = zero;
    }
    for expression in &mut artifact.expressions {
        expression.source_range = zero;
    }
    for diagnostic in &mut artifact.diagnostics {
        diagnostic.primary_range = Some(zero);
        for related in &mut diagnostic.related {
            related.source_range = zero;
        }
    }
    for obligation in &mut artifact.obligations {
        obligation.source_range = zero;
    }
    artifact.refresh_hashes().unwrap();
    let comments = text.map_or(CanonicalJson::Null, |text| {
        let whole = CanonicalJson::array([
            CanonicalJson::integer(0),
            CanonicalJson::integer(text.len() as i64),
        ]);
        CanonicalJson::object([
            ("original", CanonicalJson::string(text)),
            ("loaded", CanonicalJson::string(text)),
            ("lexical", CanonicalJson::string("")),
            (
                "loading",
                CanonicalJson::array([CanonicalJson::array([
                    CanonicalJson::string("original"),
                    whole.clone(),
                    whole.clone(),
                ])]),
            ),
            (
                "preprocess",
                CanonicalJson::array(if text.is_empty() {
                    vec![]
                } else {
                    vec![CanonicalJson::array([
                        CanonicalJson::string("removed"),
                        CanonicalJson::Null,
                        whole.clone(),
                        CanonicalJson::string("documentation"),
                    ])]
                }),
            ),
            (
                "comments",
                CanonicalJson::array(if text.is_empty() {
                    vec![]
                } else {
                    vec![CanonicalJson::array([
                        CanonicalJson::string("documentation"),
                        whole,
                        CanonicalJson::string(text),
                    ])]
                }),
            ),
            ("whitespace", CanonicalJson::array([])),
            ("landmarks", CanonicalJson::array([])),
        ])
        .unwrap()
    });
    let value = CommentedArtifact {
        artifact: artifact.clone(),
        comments: comments.clone(),
    };
    let entry = &mut manifest.modules[index];
    let write = write_published_artifact(
        root,
        &PublishedArtifactPath::new(&entry.artifact_file).unwrap(),
        &commented_artifact_json(&value).unwrap(),
        &artifact_hash_domain(
            COMMENTED_ARTIFACT_SCHEMA_FAMILY,
            COMMENTED_ARTIFACT_SCHEMA_VERSION,
        ),
        &artifact_hash_excluded_paths(),
    )
    .unwrap();
    entry.module = artifact.module.clone();
    entry.source_file = artifact.source_file.clone();
    entry.source_hash = artifact.source_hash;
    for (reference, digest) in [
        (&mut entry.artifact_hash, write.artifact_hash),
        (
            &mut entry.interface_hash,
            value.compute_interface_hash().unwrap(),
        ),
        (
            &mut entry.implementation_hash,
            value.compute_implementation_hash().unwrap(),
        ),
    ] {
        reference.schema_family = COMMENTED_ARTIFACT_SCHEMA_FAMILY.into();
        reference.schema_version = COMMENTED_ARTIFACT_SCHEMA_VERSION;
        reference.digest = digest;
    }
    (
        comments,
        write_manifest_file(root, manifest).unwrap().artifact_hash,
    )
}

fn doc_q1_request(
    root: &Path,
    manifest: &mizar_artifact::manifest::ArtifactManifest,
    hash: mizar_session::Hash,
) -> mizar_doc::doc_build::DocBuildRequest {
    use mizar_doc::doc_build::*;
    DocBuildRequest {
        profile: Phase16Profile {
            render_docs: true,
            extract_code: false,
            fail_on_doc_warnings: false,
            fail_on_extraction_warnings: false,
            include_private_docs: false,
        },
        publications: vec![DocPublicationRequest {
            package: manifest.package.clone(),
            artifact_root: root.into(),
            expected_manifest_hash: hash,
            output_dir: root.join("out"),
            source_bundle: Some(root.join("absent-native-source")),
            dependency_indexes: None,
            import_order: BTreeMap::new(),
        }],
    }
}

#[test]
fn doc_q1_mixed_publication_preserves_complete_inputs_and_readers() {
    use mizar_artifact::{manifest::*, store::CanonicalJson};
    use mizar_doc::{artifact_reader::*, doc_build::*};
    let (root, mut manifest, mut artifacts, summaries, _) =
        doc_build_summary_fixture("doc-q1-mixed", false, false);
    let mut commented = artifacts[0].clone();
    commented.module.module_path = "module_2".into();
    artifacts.push(commented);
    let mut entry = manifest.modules[0].clone();
    entry.artifact_file = "modules/commented.json".into();
    entry.module_summary_file = None;
    entry.module_summary_hash = None;
    entry.module_summary_interface_hash = None;
    manifest.modules.push(entry);
    let (projection, hash) = doc_q1_publish(&root, &mut manifest, &mut artifacts, 2, Some("::: λ"));
    let paths = std::iter::once("artifact-manifest.json".into())
        .chain(
            manifest
                .modules
                .iter()
                .map(|entry| entry.artifact_file.clone()),
        )
        .chain(
            manifest
                .modules
                .iter()
                .filter_map(|entry| entry.module_summary_file.clone()),
        )
        .chain(
            manifest
                .modules
                .iter()
                .flat_map(|entry| entry.proof_witnesses.iter().map(|w| w.witness_path.clone())),
        )
        .collect::<Vec<String>>();
    let before = paths
        .iter()
        .map(|path| std::fs::read(root.join(path)).unwrap())
        .collect::<Vec<_>>();
    let expected = read_manifest_file(&root, Default::default()).unwrap();
    let (actual, loaded) = read_published_artifacts_with_comments(&root, hash).unwrap();
    assert_eq!(actual, expected);
    assert_eq!(actual.artifact_hash, hash);
    assert_eq!(actual.manifest, manifest);
    assert_eq!(
        loaded,
        vec![
            (artifacts[0].clone(), CanonicalJson::Null),
            (artifacts[1].clone(), CanonicalJson::Null),
            (artifacts[2].clone(), projection)
        ]
    );
    assert_eq!(
        read_published_artifacts(&root, hash).unwrap(),
        (expected.clone(), artifacts.clone())
    );
    let (export_manifest, exports) = read_published_export_index(&root, hash).unwrap();
    assert_eq!(export_manifest, expected);
    for artifact in &artifacts {
        assert_eq!(exports[&artifact.module], artifact.exports);
    }
    let (summary_manifest, indexed) = read_published_module_summary_index(&root, hash).unwrap();
    assert_eq!(summary_manifest, expected);
    for summary in &summaries {
        assert_eq!(indexed[&summary.module], Some(summary.clone()));
    }
    assert_eq!(indexed[&artifacts[2].module], None);
    let label = &summaries[0].exported_labels[0];
    let (label_manifest, candidates) =
        read_published_label_export_candidates(&root, hash, &summaries[0].module, label).unwrap();
    assert_eq!(label_manifest, expected);
    assert_eq!(
        candidates,
        Some(
            artifacts[0]
                .exports
                .iter()
                .filter(|export| export.fully_qualified_name == label.owner_fully_qualified_name)
                .cloned()
                .collect()
        )
    );
    assert_eq!(
        paths
            .iter()
            .map(|path| std::fs::read(root.join(path)).unwrap())
            .collect::<Vec<_>>(),
        before
    );
    assert!(!root.join("out").exists());
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn doc_q1_availability_and_empty_doc_lines_remain_distinct_without_source() {
    use mizar_artifact::store::CanonicalJson;
    use mizar_doc::artifact_reader::read_published_artifacts_with_comments;
    let mut projections = Vec::new();
    for text in [None, Some(""), Some(":::")] {
        let (root, mut manifest, mut artifacts) = doc_published_fixture();
        let (projection, hash) = doc_q1_publish(&root, &mut manifest, &mut artifacts, 0, text);
        assert!(!root.join(&manifest.modules[0].source_file).exists());
        let (loaded_manifest, loaded) =
            read_published_artifacts_with_comments(&root, hash).unwrap();
        assert_eq!(loaded_manifest.manifest, manifest);
        assert_eq!(loaded[0], (artifacts[0].clone(), projection.clone()));
        assert_eq!(loaded[1].1, CanonicalJson::Null);
        projections.push(projection);
        std::fs::remove_dir_all(root).unwrap();
    }
    assert_eq!(projections[0], CanonicalJson::Null);
    assert_ne!(projections[0], projections[1]);
    assert_ne!(projections[1], projections[2]);
    let CanonicalJson::Object(empty) = &projections[1] else {
        panic!("available-empty projection")
    };
    assert_eq!(empty["comments"], CanonicalJson::array([]));
    let CanonicalJson::Object(line) = &projections[2] else {
        panic!("present empty documentation line")
    };
    assert_eq!(
        line["comments"],
        CanonicalJson::array([CanonicalJson::array([
            CanonicalJson::string("documentation"),
            CanonicalJson::array([CanonicalJson::integer(0), CanonicalJson::integer(3)]),
            CanonicalJson::string(":::"),
        ])])
    );
}

#[test]
fn doc_q1_bound_plans_keep_projections_full_identity_empty_modules_and_replay() {
    use mizar_artifact::store::CanonicalJson;
    use mizar_doc::doc_build::*;
    let (left, mut lm, mut la, _) = doc_build_case_fixture("doc-q1-left");
    let (right, mut rm, mut ra, _) = doc_build_case_fixture("doc-q1-right");
    la[0].exports.clear();
    let (lp, lh) = doc_q1_publish(&left, &mut lm, &mut la, 0, Some("::: left"));
    let (rp, rh) = doc_q1_publish(&right, &mut rm, &mut ra, 0, Some("::: right"));
    let (registry, request_id, snapshot, workspace) =
        doc_build_registry(&[(&left, &lm, &la), (&right, &rm, &ra)]);
    let mut request = doc_q1_request(&left, &lm, lh);
    request
        .publications
        .extend(doc_q1_request(&right, &rm, rh).publications);
    let plans = build_doc_plans(&registry, request_id, snapshot.id, &workspace, &request).unwrap();
    let mut reversed = request.clone();
    reversed.publications.reverse();
    assert_eq!(
        plans,
        build_doc_plans(&registry, request_id, snapshot.id, &workspace, &reversed).unwrap()
    );
    for (plan, manifest, artifacts, projection, hash) in [
        (&plans[0], &lm, &la, &lp, lh),
        (&plans[1], &rm, &ra, &rp, rh),
    ] {
        assert_eq!(plan.manifest.manifest, *manifest);
        assert_eq!(plan.manifest.artifact_hash, hash);
        assert_eq!(plan.request.expected_manifest_hash, hash);
        assert_eq!(plan.snapshot, snapshot.id);
        assert_eq!(plan.build_request, request_id);
        assert_eq!(plan.index.modules.len(), 2);
        for (index, entry) in manifest.modules.iter().enumerate() {
            let module = &plan.index.modules[&entry.module];
            assert_eq!(&module.artifact, entry);
            assert_eq!(module.exports, artifacts[index].exports);
            assert_eq!(
                &module.comment_projection,
                if index == 0 {
                    projection
                } else {
                    &CanonicalJson::Null
                }
            );
        }
    }
    assert_eq!(la[0].module.module_path, ra[0].module.module_path);
    assert_ne!(la[0].module, ra[0].module);
    assert!(plans[0].index.modules[&la[0].module].exports.is_empty());
    assert!(!plans[0].index.modules.contains_key(&ra[0].module));
    std::fs::remove_dir_all(left).unwrap();
    std::fs::remove_dir_all(right).unwrap();
}

#[test]
fn doc_q1_malformed_maps_and_declared_profiles_keep_lower_refusals() {
    use mizar_artifact::{
        comment_projection::*, manifest::*, store::*, verified_artifact::VerifiedArtifactError,
    };
    use mizar_doc::artifact_reader::read_published_artifacts_with_comments;
    let (root, mut manifest, mut artifacts) = doc_published_fixture();
    doc_q1_publish(&root, &mut manifest, &mut artifacts, 0, Some("::: valid"));
    let path = PublishedArtifactPath::new(&manifest.modules[0].artifact_file).unwrap();
    let domain = artifact_hash_domain(
        COMMENTED_ARTIFACT_SCHEMA_FAMILY,
        COMMENTED_ARTIFACT_SCHEMA_VERSION,
    );
    let original = read_published_artifact(&root, &path, Default::default())
        .unwrap()
        .value;
    for case in 0..4 {
        let mut bad = manifest.clone();
        let mut json = original.clone();
        if case < 2 {
            let CanonicalJson::Object(fields) = &mut json else {
                unreachable!()
            };
            if case == 0 {
                fields.insert("comments".into(), CanonicalJson::array([]));
            } else {
                let CanonicalJson::Object(comments) = fields.get_mut("comments").unwrap() else {
                    unreachable!()
                };
                comments.insert("loading".into(), CanonicalJson::array([]));
            }
        } else if case == 2 {
            bad.modules[0].artifact_hash.schema_version = SchemaVersion::new(9, 0);
            bad.modules[0].interface_hash.schema_version = SchemaVersion::new(9, 0);
            bad.modules[0].implementation_hash.schema_version = SchemaVersion::new(9, 0);
        } else {
            bad.modules[0].interface_hash.schema_family = "mizar/verified-artifact".into();
        }
        bad.modules[0].artifact_hash.digest = write_published_artifact(
            &root,
            &path,
            &json,
            &domain,
            &artifact_hash_excluded_paths(),
        )
        .unwrap()
        .artifact_hash;
        let mut manifest_json = artifact_manifest_json(&manifest).unwrap();
        let CanonicalJson::Object(fields) = &mut manifest_json else {
            unreachable!()
        };
        let CanonicalJson::Array(modules) = fields.get_mut("modules").unwrap() else {
            unreachable!()
        };
        let CanonicalJson::Object(entry) = &mut modules[0] else {
            unreachable!()
        };
        entry.insert(
            "artifact_hash".into(),
            CanonicalJson::string(bad.modules[0].artifact_hash.to_artifact_hash_string()),
        );
        entry.insert(
            "interface_hash".into(),
            CanonicalJson::string(bad.modules[0].interface_hash.to_artifact_hash_string()),
        );
        entry.insert(
            "implementation_hash".into(),
            CanonicalJson::string(bad.modules[0].implementation_hash.to_artifact_hash_string()),
        );
        let hash = write_published_artifact(
            &root,
            &PublishedArtifactPath::new("artifact-manifest.json").unwrap(),
            &manifest_json,
            &manifest_hash_domain(manifest.schema_version),
            &[],
        )
        .unwrap()
        .artifact_hash;
        let lower = read_manifest_file(
            &root,
            ManifestFileReadOptions {
                expected_manifest_hash: Some(hash),
                validate_references: true,
                validate_witness_files: true,
                ..Default::default()
            },
        )
        .unwrap_err();
        let error = read_published_artifacts_with_comments(&root, hash).unwrap_err();
        assert_eq!(error, lower, "case {case}");
        let expected_layer = match &error {
            ManifestError::VerifiedArtifact {
                path: failed,
                error,
            } if failed == path.as_str() => match (case, error.as_ref()) {
                (0, VerifiedArtifactError::UnexpectedType { path, .. }) => path == "$.comments",
                (1, VerifiedArtifactError::InvalidField { path, .. }) => {
                    path == "$.comments.loading"
                }
                _ => false,
            },
            ManifestError::InvalidHash { path, .. } => {
                path == if case == 2 {
                    "$.modules[0].artifact_hash"
                } else {
                    "$.modules[0].interface_hash"
                } && case >= 2
            }
            _ => false,
        };
        assert!(expected_layer, "case {case}: {error:?}");
    }
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn doc_q1_bound_plans_refuse_hash_source_package_provenance_request_and_snapshot() {
    use mizar_artifact::manifest::write_manifest_file;
    use mizar_doc::doc_build::*;
    let (root, mut manifest, mut artifacts, _) = doc_build_case_fixture("doc-q1-binding");
    let (_, hash) = doc_q1_publish(&root, &mut manifest, &mut artifacts, 0, Some("::: bound"));
    let (registry, request_id, snapshot, workspace) =
        doc_build_registry(&[(&root, &manifest, &artifacts)]);
    let request = doc_q1_request(&root, &manifest, hash);
    assert!(build_doc_plans(&registry, request_id, snapshot.id, &workspace, &request).is_ok());
    let mut wrong = request.clone();
    wrong.publications[0].expected_manifest_hash = mizar_session::hash_text("wrong selection");
    assert!(matches!(
        build_doc_plans(&registry, request_id, snapshot.id, &workspace, &wrong),
        Err(DocBuildError::Publication(_))
    ));
    wrong = request.clone();
    wrong.publications[0].package.package_id = "wrong package".into();
    assert!(matches!(
        build_doc_plans(&registry, request_id, snapshot.id, &workspace, &wrong),
        Err(DocBuildError::Binding(_))
    ));
    let other_id = mizar_session::InMemorySessionIdAllocator::new();
    other_id.next_request_id().unwrap();
    assert!(matches!(
        build_doc_plans(
            &registry,
            other_id.next_request_id().unwrap(),
            snapshot.id,
            &workspace,
            &request
        ),
        Err(DocBuildError::Binding(_))
    ));
    let (bad_root, mut bad_manifest, mut bad_artifacts, _) =
        doc_build_case_fixture("doc-q1-binding");
    let (_, bad_hash) = doc_q1_publish(
        &bad_root,
        &mut bad_manifest,
        &mut bad_artifacts,
        0,
        Some("::: changed source"),
    );
    let mut later_bad = request.clone();
    later_bad
        .publications
        .extend(doc_q1_request(&bad_root, &bad_manifest, bad_hash).publications);
    assert!(matches!(
        build_doc_plans(&registry, request_id, snapshot.id, &workspace, &later_bad),
        Err(DocBuildError::Binding(_))
    ));
    let mut provenance = manifest.clone();
    provenance.toolchain.push_str("-wrong");
    let provenance_hash = write_manifest_file(&root, &provenance)
        .unwrap()
        .artifact_hash;
    assert!(matches!(
        build_doc_plans(
            &registry,
            request_id,
            snapshot.id,
            &workspace,
            &doc_q1_request(&root, &provenance, provenance_hash)
        ),
        Err(DocBuildError::Binding(_))
    ));
    write_manifest_file(&root, &manifest).unwrap();
    registry
        .create_snapshot(
            request_id,
            mizar_session::SnapshotInput {
                workspace_root: workspace.clone(),
                source_versions: snapshot.source_versions.clone(),
                dependency_artifacts: snapshot.dependency_artifacts.clone(),
                lockfile_hash: snapshot.lockfile_hash,
                toolchain: snapshot.toolchain.clone(),
                verifier_config_hash: mizar_session::hash_text("new snapshot"),
            },
        )
        .unwrap();
    assert!(matches!(
        build_doc_plans(&registry, request_id, snapshot.id, &workspace, &request),
        Err(DocBuildError::Binding(_))
    ));
    std::fs::remove_dir_all(root).unwrap();
    std::fs::remove_dir_all(bad_root).unwrap();
}

#[test]
fn doc_q1_comment_only_republication_preserves_old_plans_and_refuses_old_bindings() {
    use mizar_doc::{artifact_reader::read_published_artifacts_with_comments, doc_build::*};
    let (root, mut manifest, mut artifacts, _) = doc_build_case_fixture("doc-q1-change");
    let (old_projection, old_hash) =
        doc_q1_publish(&root, &mut manifest, &mut artifacts, 0, Some("::: old"));
    let old_manifest = manifest.clone();
    let old_artifacts = artifacts.clone();
    let (registry, request_id, old_snapshot, workspace) =
        doc_build_registry(&[(&root, &manifest, &artifacts)]);
    let old_request = doc_q1_request(&root, &manifest, old_hash);
    let old_plan = build_doc_plans(
        &registry,
        request_id,
        old_snapshot.id,
        &workspace,
        &old_request,
    )
    .unwrap();
    let saved = old_plan.clone();
    let (projection, hash) =
        doc_q1_publish(&root, &mut manifest, &mut artifacts, 0, Some("::: new"));
    assert_ne!(old_hash, hash);
    assert_ne!(old_artifacts[0].source_hash, artifacts[0].source_hash);
    assert_ne!(old_projection, projection);
    assert_eq!(
        old_manifest.modules[0].interface_hash,
        manifest.modules[0].interface_hash
    );
    assert!(read_published_artifacts_with_comments(&root, old_hash).is_err());
    let request = doc_q1_request(&root, &manifest, hash);
    assert!(matches!(
        build_doc_plans(&registry, request_id, old_snapshot.id, &workspace, &request),
        Err(DocBuildError::Binding(_))
    ));
    let mut input = mizar_session::SnapshotInput {
        workspace_root: workspace.clone(),
        source_versions: old_snapshot.source_versions.clone(),
        dependency_artifacts: old_snapshot.dependency_artifacts.clone(),
        lockfile_hash: old_snapshot.lockfile_hash,
        toolchain: old_snapshot.toolchain.clone(),
        verifier_config_hash: old_snapshot.verifier_config_hash,
    };
    input.source_versions[0].source_hash = artifacts[0].source_hash;
    let (current, _) = registry.create_snapshot(request_id, input).unwrap();
    let plans = build_doc_plans(&registry, request_id, current.id, &workspace, &request).unwrap();
    assert_eq!(plans[0].manifest.artifact_hash, hash);
    assert_eq!(
        plans[0].index.modules[&artifacts[0].module].comment_projection,
        projection
    );
    assert_eq!(
        plans[0].index.modules[&artifacts[0].module]
            .artifact
            .source_hash,
        artifacts[0].source_hash
    );
    assert!(build_doc_plans(&registry, request_id, old_snapshot.id, &workspace, &request).is_err());
    assert_eq!(old_plan, saved);
    assert_eq!(
        old_plan[0].index.modules[&old_artifacts[0].module].comment_projection,
        old_projection
    );
    std::fs::remove_dir_all(root).unwrap();
}

fn doc7_range(start: usize, end: usize) -> mizar_artifact::store::CanonicalJson {
    use mizar_artifact::store::CanonicalJson as J;
    J::array([J::integer(start as i64), J::integer(end as i64)])
}

fn doc7_landmark(
    kind: &str,
    start: usize,
    end: usize,
    parent: Option<usize>,
    export: Option<&mizar_artifact::verified_artifact::VerifiedExport>,
) -> mizar_artifact::store::CanonicalJson {
    use mizar_artifact::store::CanonicalJson as J;
    J::array([
        J::string(kind),
        doc7_range(start, end),
        parent.map_or(J::Null, |p| J::integer(p as i64)),
        J::Bool(false),
        export.map_or(J::Null, |e| {
            J::array([J::string(&e.origin_id), J::string(&e.fully_qualified_name)])
        }),
    ])
}

fn doc7_projection(
    original: &str,
    landmarks: Vec<mizar_artifact::store::CanonicalJson>,
) -> mizar_artifact::store::CanonicalJson {
    use mizar_artifact::store::CanonicalJson as J;
    use mizar_lexer::{CommentKind, SourcePreprocessMapSegment as P};
    use mizar_session::LoadingMapSegment as L;
    let (loaded, loading) = mizar_session::normalize_source_text(original);
    let lexical = mizar_lexer::preprocess_source_for_lexing(&loaded);
    assert!(lexical.diagnostics.is_empty());
    let kind = |kind| match kind {
        CommentKind::SingleLine => "single_line",
        CommentKind::MultiLine => "multi_line",
        CommentKind::Documentation => "documentation",
        _ => unreachable!(),
    };
    let loading = loading.unwrap_or_else(|| {
        vec![L::Original {
            loaded: mizar_session::TextRange::new(0, loaded.len()),
            original: mizar_session::TextRange::new(0, original.len()),
        }]
    });
    J::object([
        ("original", J::string(original)),
        ("loaded", J::string(&loaded)),
        ("lexical", J::string(&lexical.lexical_text)),
        (
            "loading",
            J::array(loading.into_iter().map(|segment| {
                let (kind, output, input) = match segment {
                    L::Original { loaded, original } => ("original", Some(loaded), original),
                    L::NormalizedNewline { loaded, original } => {
                        ("newline", Some(loaded), original)
                    }
                    L::RemovedLeadingBom { original } => ("bom", None, original),
                    _ => unreachable!(),
                };
                J::array([
                    J::string(kind),
                    output.map_or(J::Null, |r| doc7_range(r.start, r.end)),
                    doc7_range(input.start, input.end),
                ])
            })),
        ),
        (
            "preprocess",
            J::array(lexical.preprocess_map.segments.iter().map(|segment| {
                let (tag, output, input, comment) = match *segment {
                    P::Original { lexical, source } => ("original", Some(lexical), source, J::Null),
                    P::RemovedComment { source, kind: k } => {
                        ("removed", None, source, J::string(kind(k)))
                    }
                    P::SyntheticWhitespace { lexical, anchor } => {
                        ("synthetic", Some(lexical), anchor, J::Null)
                    }
                    _ => unreachable!(),
                };
                J::array([
                    J::string(tag),
                    output.map_or(J::Null, |r| doc7_range(r.start, r.end)),
                    doc7_range(input.start, input.end),
                    comment,
                ])
            })),
        ),
        (
            "comments",
            J::array(lexical.comments.iter().map(|c| {
                J::array([
                    J::string(kind(c.kind)),
                    doc7_range(c.span.start, c.span.end),
                    J::string(&c.lexeme),
                ])
            })),
        ),
        (
            "whitespace",
            J::array(
                loaded
                    .char_indices()
                    .filter(|&(at, ch)| {
                        ch.is_whitespace()
                            && !lexical
                                .comments
                                .iter()
                                .any(|c| c.span.start <= at && at < c.span.end)
                    })
                    .map(|(at, ch)| doc7_range(at, at + ch.len_utf8())),
            ),
        ),
        ("landmarks", J::array(landmarks)),
    ])
    .unwrap()
}

// Publish real Session normalization and Lexer preprocessing through Artifact's
// codec and Doc's bound-plan reader. Structural verifier metadata permits cases
// (missing labels, recovery, competing targets) a successful producer cannot emit.
fn doc7_fixture(
    package: &str,
    original: &str,
    target: Option<(&str, &str, &str)>,
    adjust: impl FnOnce(
        &mut mizar_artifact::verified_artifact::VerifiedArtifact,
        &mut mizar_artifact::module_summary::ModuleSummary,
        &mut Vec<mizar_artifact::store::CanonicalJson>,
    ),
) -> (
    PathBuf,
    mizar_doc::doc_build::DocBuildPlan,
    mizar_session::LoadedSource,
) {
    use mizar_artifact::{comment_projection::*, manifest::*, module_summary::*, store::*};
    use mizar_session::{DiskSourceLoader, SourceLoader};
    let (root, mut manifest, mut artifacts, mut summaries, _) =
        doc_build_summary_fixture(package, true, false);
    let path = root.join("src/source.miz");
    std::fs::write(&path, original).unwrap();
    let loader = DiskSourceLoader::new(&root);
    let source = loader
        .load(
            super::super::shared::snapshot_id(40_007),
            mizar_session::SourceInput {
                package_id: mizar_session::PackageId::new(package),
                module_path: mizar_session::ModulePath::new("module_0"),
                normalized_path: loader.normalize_path(&root, &path).unwrap(),
                edition: mizar_session::Edition::new(artifacts[0].module.language_edition.clone()),
                origin: mizar_session::SourceOriginInput::Disk { path },
            },
            &mizar_session::InMemorySessionIdAllocator::new(),
        )
        .unwrap();
    doc_q1_publish(&root, &mut manifest, &mut artifacts, 0, None);
    let artifact = &mut artifacts[0];
    artifact.source_hash = source.source_hash;
    manifest.modules[0].source_hash = source.source_hash;
    artifact.provenance.captured_source =
        Some((source.source_id, source.origin.clone(), source.source_hash));
    let summary = &mut summaries[0];
    summary.source_hash = source.source_hash;
    let mut landmarks = Vec::new();
    if let Some((kind, syntax, body)) = target {
        let start = source.text.find(body).unwrap();
        let lexical = mizar_lexer::preprocess_source_for_lexing(&source.text).lexical_text;
        let lexical_start = lexical.find(body).unwrap();
        let export = &mut artifact.exports[0];
        export.export_kind = kind.into();
        export.proof_status = Some(
            if matches!(kind, "definition" | "functor" | "redefinition") {
                mizar_artifact::verified_artifact::ExportProofStatus::NotRequired
            } else {
                mizar_artifact::verified_artifact::ExportProofStatus::Accepted
            },
        );
        export.source_range = SourceRangeSummary {
            start_byte: lexical_start as u64,
            end_byte: (lexical_start + body.len()) as u64,
        };
        summary.exported_labels.push(ExportedLabelSummary {
            origin_id: "stored-label-origin".into(),
            label: "StoredLabel".into(),
            owner_fully_qualified_name: export.fully_qualified_name.clone(),
            visibility: "public".into(),
            source_range: export.source_range,
            target_kind: match kind {
                "functor" | "redefinition" => "definition",
                "lemma" => "theorem",
                _ => kind,
            }
            .into(),
        });
        landmarks.push(doc7_landmark(
            syntax,
            start,
            start + body.len(),
            None,
            Some(export),
        ));
    }
    adjust(artifact, summary, &mut landmarks);
    artifact.refresh_hashes().unwrap();
    summary.refresh_interface_hash().unwrap();
    let summary_path =
        PublishedArtifactPath::new(manifest.modules[0].module_summary_file.as_ref().unwrap())
            .unwrap();
    let summary_write = write_published_artifact(
        &root,
        &summary_path,
        &module_summary_json(summary).unwrap(),
        &artifact_hash_domain(MODULE_SUMMARY_SCHEMA_FAMILY, summary.schema_version),
        &[],
    )
    .unwrap();
    let entry = &mut manifest.modules[0];
    entry.module_summary_hash.as_mut().unwrap().digest = summary_write.artifact_hash;
    entry.module_summary_interface_hash.as_mut().unwrap().digest = summary.interface_hash;
    let commented = CommentedArtifact {
        artifact: artifact.clone(),
        comments: doc7_projection(original, landmarks),
    };
    let write = write_published_artifact(
        &root,
        &PublishedArtifactPath::new(&entry.artifact_file).unwrap(),
        &commented_artifact_json(&commented).unwrap(),
        &artifact_hash_domain(
            COMMENTED_ARTIFACT_SCHEMA_FAMILY,
            COMMENTED_ARTIFACT_SCHEMA_VERSION,
        ),
        &artifact_hash_excluded_paths(),
    )
    .unwrap();
    entry.artifact_hash.digest = write.artifact_hash;
    entry.interface_hash.digest = commented.compute_interface_hash().unwrap();
    entry.implementation_hash.digest = commented.compute_implementation_hash().unwrap();
    let hash = write_manifest_file(&root, &manifest).unwrap().artifact_hash;
    let (registry, request, snapshot, workspace) =
        doc_build_registry(&[(&root, &manifest, &artifacts)]);
    let mut plans = mizar_doc::doc_build::build_doc_plans(
        &registry,
        request,
        snapshot.id,
        &workspace,
        &doc_q1_request(&root, &manifest, hash),
    )
    .unwrap();
    (root, plans.remove(0), source)
}

fn doc7_attach(
    plan: &mizar_doc::doc_build::DocBuildPlan,
) -> mizar_doc::comments::CommentAttachments<'_> {
    let module = plan
        .index
        .modules
        .keys()
        .find(|m| m.module_path == "module_0")
        .unwrap();
    mizar_doc::comments::attach_comments(plan, module, None).unwrap()
}

fn doc7_codes(result: &mizar_doc::comments::CommentAttachments<'_>) -> Vec<String> {
    result
        .warnings
        .iter()
        .map(|w| w.code().to_string())
        .collect()
}

#[test]
fn doc7_module_prefix_keeps_separated_raw_blocks_module_only() {
    use mizar_doc::comments::CommentTarget;
    let text = " \t\n:: preface\n::: first λ\n  :::\n\t::: third\n\n::: second block\n::= separator =::\n::: final block\ntheorem T: true;\n";
    let (root, plan, _) = doc7_fixture(
        "doc7",
        text,
        Some(("theorem", "TheoremItem", "theorem T: true;")),
        |_, _, _| {},
    );
    let result = doc7_attach(&plan);
    assert!(result.available());
    assert!(result.warnings.is_empty());
    assert_eq!(result.blocks.len(), 3);
    assert_eq!(
        result
            .blocks
            .iter()
            .map(|b| b.lines.len())
            .collect::<Vec<_>>(),
        [3, 1, 1]
    );
    for block in &result.blocks {
        assert!(matches!(block.target, Some(CommentTarget::Module)));
        assert!(block.candidates.is_empty());
        let raw = &text[block.range.start..block.range.end];
        assert!(raw.starts_with(":::"));
        assert!(!raw.contains("separator"));
    }
    assert!(
        result
            .blocks
            .windows(2)
            .all(|b| b[0].range.end < b[1].range.start)
    );
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn doc7_exact_named_targets_use_original_export_and_labels_with_attached_annotation() {
    use mizar_doc::comments::CommentTarget;
    for (kind, syntax, source_body, labels) in [
        (
            "functor",
            "FunctorDefinition",
            "func f -> object equals x;",
            true,
        ),
        (
            "redefinition",
            "FunctorRedefinition",
            "redefine func f -> object equals x;",
            true,
        ),
        ("theorem", "TheoremItem", "theorem T: true;", true),
        ("theorem", "TheoremItem", "public\ntheorem T: true;", true),
        ("lemma", "LemmaItem", "lemma T: true;", true),
        (
            "registration",
            "ExistentialRegistration",
            "cluster inhabited for set;",
            true,
        ),
        ("algorithm", "AlgorithmDefinition", "algorithm A;", true),
        ("algorithm", "AlgorithmDefinition", "algorithm A;", false),
    ] {
        let body = source_body.strip_prefix("public\n").unwrap_or(source_body);
        let text = format!("import X;\n::: item\n@proof_hint(\"trivial\")\n{source_body}\n");
        let (root, plan, _) = doc7_fixture(
            "doc7",
            &text,
            Some((kind, syntax, body)),
            |artifact, summary, landmarks| {
                if !labels {
                    summary.exported_labels.clear();
                }
                let start = text.find('@').unwrap();
                let item_start = text.find(body).unwrap();
                let item_end = item_start + body.len();
                let item = landmarks.pop().unwrap();
                landmarks.push(doc7_landmark(
                    "ImportItem",
                    0,
                    "import X;".len(),
                    None,
                    None,
                ));
                let mut item = item;
                if source_body.starts_with("public") {
                    // parse_visible_item retains these three direct children.
                    landmarks.push(doc7_landmark("VisibleItem", start, item_end, None, None));
                    landmarks.push(doc7_landmark(
                        "Annotation",
                        start,
                        start + "@proof_hint(\"trivial\")".len(),
                        Some(1),
                        None,
                    ));
                    let marker = text.find("public").unwrap();
                    landmarks.push(doc7_landmark(
                        "VisibilityMarker",
                        marker,
                        marker + "public".len(),
                        Some(1),
                        None,
                    ));
                    if let mizar_artifact::store::CanonicalJson::Array(row) = &mut item {
                        row[2] = mizar_artifact::store::CanonicalJson::integer(1);
                    }
                    landmarks.push(item);
                } else if matches!(syntax, "TheoremItem" | "LemmaItem") {
                    // parser/module.rs::parse_theorem_item owns leading Annotation
                    // children; its covering_token_range begins at the annotation.
                    if let mizar_artifact::store::CanonicalJson::Array(row) = &mut item {
                        row[1] = doc7_range(start, item_end);
                    }
                    landmarks.push(item);
                    landmarks.push(doc7_landmark(
                        "Annotation",
                        start,
                        item_start - 1,
                        Some(1),
                        None,
                    ));
                } else {
                    landmarks.push(doc7_landmark(
                        "AnnotatedStatement",
                        start,
                        item_end,
                        None,
                        None,
                    ));
                    landmarks.push(doc7_landmark(
                        "Annotation",
                        start,
                        item_start - 1,
                        Some(1),
                        None,
                    ));
                    if let mizar_artifact::store::CanonicalJson::Array(row) = &mut item {
                        row[2] = mizar_artifact::store::CanonicalJson::integer(1);
                    }
                    landmarks.push(item);
                }
                assert_eq!(artifact.exports[0].fully_qualified_name, "fixture::export");
            },
        );
        let result = doc7_attach(&plan);
        assert!(
            result.warnings.is_empty(),
            "{source_body}, labels={labels}: {:?}",
            result.warnings
        );
        assert_eq!(result.blocks.len(), 1);
        let Some(CommentTarget::Item(export)) = result.blocks[0].target else {
            panic!("{kind} did not attach")
        };
        assert!(std::ptr::eq(export, &result.input.exports[0]));
        assert_eq!(export.origin_id, "structural-export");
        assert_eq!(export.fully_qualified_name, "fixture::export");
        assert_eq!(result.blocks[0].lines.len(), 1);
        assert!(!result.blocks[0].candidates.is_empty());
        std::fs::remove_dir_all(root).unwrap();
    }
}

#[test]
fn doc7_interruptions_never_skip_to_later_target_and_retain_orphan_text() {
    for interruption in [
        "\n",
        ":: ordinary\n",
        "import Y;\n",
        "reserve x for set;\n",
        "@standalone\n",
        "@proof_hint(\"trivial\")\n\n",
        "@proof_hint(\"trivial\")\n:: interruption\n",
        "@proof_hint(\"trivial\")\n\npublic\n",
        "@proof_hint(\"trivial\")\n:: interruption\npublic\n",
        "@proof_hint(\"trivial\")\npublic\n\n",
        "@proof_hint(\"trivial\")\npublic\n:: interruption\n",
        "",
    ] {
        let trailing = interruption.is_empty();
        let body = "theorem T: true;";
        let text = if trailing {
            "import X;\n::: retained trailing\n".into()
        } else {
            format!("import X;\n::: retained\n{interruption}{body}\n")
        };
        let (root, plan, _) = doc7_fixture(
            "doc7",
            &text,
            (!trailing).then_some(("theorem", "TheoremItem", body)),
            |_, _, landmarks| {
                let mut target = landmarks.pop();
                landmarks.push(doc7_landmark(
                    "ImportItem",
                    0,
                    "import X;".len(),
                    None,
                    None,
                ));
                if interruption.starts_with("@proof_hint") {
                    let start = text.find(interruption).unwrap();
                    let end = text.find(body).unwrap() + body.len();
                    let visible = interruption.contains("public");
                    // Same direct-child shape emitted by parse_theorem_item.
                    let Some(mizar_artifact::store::CanonicalJson::Array(row)) = &mut target else {
                        unreachable!()
                    };
                    if visible {
                        row[2] = mizar_artifact::store::CanonicalJson::integer(1);
                        landmarks.push(doc7_landmark("VisibleItem", start, end, None, None));
                    } else {
                        row[1] = doc7_range(start, end);
                        landmarks.push(target.take().unwrap());
                    }
                    landmarks.push(doc7_landmark(
                        "Annotation",
                        start,
                        start + "@proof_hint(\"trivial\")".len(),
                        Some(1),
                        None,
                    ));
                    if visible {
                        let marker = text.find("public").unwrap();
                        landmarks.push(doc7_landmark(
                            "VisibilityMarker",
                            marker,
                            marker + "public".len(),
                            Some(1),
                            None,
                        ));
                    }
                } else if !trailing && !interruption.starts_with(['\n', ':']) {
                    let start = text.find(interruption).unwrap();
                    let kind = if interruption.starts_with('@') {
                        "StandaloneDiagnosticAnnotation"
                    } else if interruption.starts_with("reserve") {
                        "ReserveItem"
                    } else {
                        "ImportItem"
                    };
                    landmarks.push(doc7_landmark(
                        kind,
                        start,
                        start + interruption.len() - 1,
                        None,
                        None,
                    ));
                }
                landmarks.extend(target);
            },
        );
        let result = doc7_attach(&plan);
        assert_eq!(doc7_codes(&result), ["W0400"], "{interruption:?}");
        assert!(result.blocks[0].target.is_none());
        assert!(
            text[result.blocks[0].range.start..result.blocks[0].range.end].contains("retained")
        );
        std::fs::remove_dir_all(root).unwrap();
    }
}

#[test]
fn doc7_missing_ambiguous_unsupported_and_unverified_targets_choose_none() {
    use mizar_artifact::{store::CanonicalJson as J, verified_artifact::ExportProofStatus};
    for (case, code) in [
        ("identity", "W0401"),
        ("label", "W0401"),
        ("unsupported", "W0401"),
        ("nested", "W0401"),
        ("recovered", "W0403"),
        ("unverified", "W0403"),
        ("absent-status", "W0403"),
        ("ambiguous", "W0402"),
        ("range", "W0402"),
    ] {
        let text = if case == "nested" {
            "import X;\ntheorem Outer: true;\n::: retained\ntheorem T: true;\n"
        } else {
            "import X;\n::: retained\ntheorem T: true;\n"
        };
        let (root, plan, _) = doc7_fixture(
            "doc7",
            text,
            Some(("theorem", "TheoremItem", "theorem T: true;")),
            |artifact, summary, landmarks| {
                let J::Array(row) = &mut landmarks[0] else {
                    unreachable!()
                };
                match case {
                    "identity" => row[4] = J::Null,
                    "label" => summary.exported_labels.clear(),
                    "unsupported" => row[0] = J::string("StatementItem"),
                    "nested" => {
                        row[2] = J::integer(1);
                    }
                    "recovered" => row[3] = J::Bool(true),
                    "unverified" => {
                        artifact.exports[0].proof_status = Some(ExportProofStatus::NotAccepted)
                    }
                    "absent-status" => artifact.exports[0].proof_status = None,
                    "ambiguous" => {
                        let mut other = artifact.exports[0].clone();
                        other.origin_id.push_str("-other");
                        other.fully_qualified_name.push_str("Other");
                        let mut label = summary.exported_labels[0].clone();
                        label.origin_id.push_str("-other");
                        label.owner_fully_qualified_name = other.fully_qualified_name.clone();
                        summary.exported_labels.push(label);
                        let start = text.find("theorem").unwrap();
                        landmarks.push(doc7_landmark(
                            "FunctorDefinition",
                            start,
                            start + "theorem T: true;".len(),
                            None,
                            Some(&other),
                        ));
                        artifact.exports.push(other);
                    }
                    "range" => {
                        artifact.exports[0].source_range.start_byte = 0;
                        artifact.exports[0].source_range.end_byte = 1;
                    }
                    _ => unreachable!(),
                }
                let target = landmarks.remove(0);
                landmarks.insert(
                    0,
                    doc7_landmark("ImportItem", 0, "import X;".len(), None, None),
                );
                if case == "nested" {
                    let start = text.find("theorem Outer").unwrap();
                    landmarks.push(doc7_landmark("TheoremItem", start, text.len(), None, None));
                    landmarks.push(target);
                } else {
                    landmarks.insert(1, target);
                }
            },
        );
        let result = doc7_attach(&plan);
        assert_eq!(doc7_codes(&result), [code], "{case}");
        assert!(result.blocks[0].target.is_none(), "{case}");
        assert_eq!(result.blocks[0].lines.len(), 1);
        assert!(!result.blocks[0].candidates.is_empty());
        std::fs::remove_dir_all(root).unwrap();
    }
}

#[test]
fn doc7_composes_unicode_bom_newlines_and_preprocessing_or_warns_with_provenance() {
    use mizar_diagnostics::failure_record::DiagnosticPrimaryLocation;
    use mizar_doc::comments::{CommentTarget, attach_comments};
    let text = "\u{feff}import X;\r\n:: λ preface\r\n::: λ docs\r\ntheorem T: true;\r\n";
    let (root, plan, source) = doc7_fixture(
        "doc7",
        text,
        Some(("theorem", "TheoremItem", "theorem T: true;")),
        |_, _, landmarks| {
            landmarks.insert(
                0,
                doc7_landmark("ImportItem", 0, "import X;".len(), None, None),
            );
        },
    );
    let result = doc7_attach(&plan);
    assert!(result.warnings.is_empty());
    assert!(matches!(
        result.blocks[0].target,
        Some(CommentTarget::Item(_))
    ));
    assert_eq!(
        &source.text[result.blocks[0].range.start..result.blocks[0].range.end],
        "::: λ docs\n"
    );
    let saved = result.input.comment_projection.clone();
    assert_eq!(
        saved,
        doc7_projection(
            text,
            match &saved {
                mizar_artifact::store::CanonicalJson::Object(o) => match &o["landmarks"] {
                    mizar_artifact::store::CanonicalJson::Array(a) => a.clone(),
                    _ => unreachable!(),
                },
                _ => unreachable!(),
            }
        )
    );
    let foreign_source = source;
    std::fs::remove_dir_all(root).unwrap();
    let (root, plan, source) = doc7_fixture(
        "doc7",
        "import X;\n::: kept\ntheorem T: true;\n",
        Some(("theorem", "TheoremItem", "theorem T: true;")),
        |artifact, _, landmarks| {
            artifact.exports[0].source_range.start_byte = 10_000;
            artifact.exports[0].source_range.end_byte = 10_001;
            landmarks.insert(
                0,
                doc7_landmark("ImportItem", 0, "import X;".len(), None, None),
            );
        },
    );
    let result = doc7_attach(&plan);
    assert_eq!(doc7_codes(&result), ["W0404"]);
    assert!(result.blocks[0].target.is_none());
    assert!(
        matches!(result.warnings[0].primary_location(), DiagnosticPrimaryLocation::Documentation { publication_hash, package_id, package_version, lockfile_identity, module_path, language_edition, block_ordinal: 0 } if *publication_hash == plan.manifest.artifact_hash && package_id == &result.module.package_id && package_version == &result.module.package_version && lockfile_identity == &result.module.lockfile_identity && module_path == &result.module.module_path && language_edition == &result.module.language_edition)
    );
    let bound = attach_comments(&plan, result.module, Some(&source)).unwrap();
    assert!(
        matches!(bound.warnings[0].primary_location(), DiagnosticPrimaryLocation::Span(span) if span.range() == mizar_session::SourceRange { source_id: source.source_id, start: bound.blocks[0].range.start, end: bound.blocks[0].range.end })
    );
    let incompatible = attach_comments(&plan, result.module, Some(&foreign_source)).unwrap();
    assert_eq!(incompatible.warnings, result.warnings);
    assert!(incompatible.blocks[0].source_range.is_none());
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn doc7_replay_availability_private_option_and_warning_promotion_preserve_inputs() {
    let (root, mut plan, _) = doc7_fixture(
        "doc7",
        "import X;\n::: retained\n",
        None,
        |_, _, landmarks| {
            landmarks.push(doc7_landmark(
                "ImportItem",
                0,
                "import X;".len(),
                None,
                None,
            ));
        },
    );
    let saved = plan.clone();
    let paths = std::iter::once("artifact-manifest.json".to_owned())
        .chain(std::iter::once("src/source.miz".to_owned()))
        .chain(
            plan.manifest
                .manifest
                .modules
                .iter()
                .flat_map(|m| [Some(m.artifact_file.clone()), m.module_summary_file.clone()])
                .flatten(),
        )
        .chain(
            plan.manifest
                .manifest
                .modules
                .iter()
                .flat_map(|m| m.proof_witnesses.iter().map(|w| w.witness_path.clone())),
        )
        .collect::<Vec<_>>();
    let before = paths
        .iter()
        .map(|p| std::fs::read(root.join(p)).unwrap())
        .collect::<Vec<_>>();
    let mut initial = doc7_attach(&plan);
    assert_eq!(doc7_codes(&initial), ["W0400"]);
    assert!(!initial.documentation_failed());
    let blocks = format!("{:?}", initial.blocks);
    let warnings = initial.warnings.clone();
    assert_eq!(blocks, format!("{:?}", doc7_attach(&plan).blocks));
    assert_eq!(initial.warnings, doc7_attach(&plan).warnings);
    // New Markdown/tag drafts are supplied at the consumer seam; attachment still emits W0400 only.
    use mizar_diagnostics::{
        failure_record::{DiagnosticDraft, DiagnosticDraftInput},
        registry::{DiagnosticCode, DiagnosticRegistry, DiagnosticSeverity},
    };
    let template = &warnings[0];
    let supplied = (405..=411)
        .map(|number| {
            let code = DiagnosticCode::from_parts(DiagnosticSeverity::Warning, number).unwrap();
            let descriptor = DiagnosticRegistry::builtin().lookup(code).unwrap();
            DiagnosticDraft::new(DiagnosticDraftInput {
                source_snapshot: template.source_snapshot(),
                code,
                phase: template.phase(),
                category: template.category(),
                stable_detail_key: descriptor.semantic_name.into(),
                message: "supplied Doc8 warning".into(),
                primary_location: template.primary_location().clone(),
                secondary_spans: template.secondary_spans().to_vec(),
                notes: template.notes().to_vec(),
                details: template.details().clone(),
                fixes: vec![],
                explanation: None,
            })
            .unwrap()
        })
        .collect::<Vec<_>>();
    initial.warnings = supplied.clone();
    assert!(!initial.documentation_failed());
    drop(initial);
    plan.profile.include_private_docs = true;
    assert_eq!(blocks, format!("{:?}", doc7_attach(&plan).blocks));
    assert_eq!(warnings, doc7_attach(&plan).warnings);
    plan.profile.fail_on_doc_warnings = true;
    assert!(doc7_attach(&plan).documentation_failed());
    let mut promoted = doc7_attach(&plan);
    promoted.warnings = supplied.clone();
    assert!(promoted.documentation_failed());
    assert_eq!(promoted.warnings, supplied);
    assert_eq!(blocks, format!("{:?}", promoted.blocks));
    for warning in &promoted.warnings {
        let descriptor = DiagnosticRegistry::builtin()
            .lookup(warning.code())
            .unwrap();
        assert_eq!(descriptor.default_severity, DiagnosticSeverity::Warning);
    }
    drop(promoted);
    assert_eq!(plan.index, saved.index);
    assert_eq!(
        before,
        paths
            .iter()
            .map(|p| std::fs::read(root.join(p)).unwrap())
            .collect::<Vec<_>>()
    );
    let absent = plan
        .index
        .modules
        .keys()
        .find(|m| m.module_path == "module_1")
        .unwrap();
    let unavailable = mizar_doc::comments::attach_comments(&plan, absent, None).unwrap();
    assert!(!unavailable.available());
    assert!(unavailable.blocks.is_empty());
    std::fs::remove_dir_all(root).unwrap();
    let (root, plan, _) = doc7_fixture("doc7", "", None, |_, _, _| {});
    let empty = doc7_attach(&plan);
    assert!(empty.available());
    assert!(empty.blocks.is_empty());
    assert!(empty.warnings.is_empty());
    std::fs::remove_dir_all(root).unwrap();

    // Identical origin/FQN/label text in distinct owning packages is not one item.
    let text = "import X;\n::: same label\ntheorem T: true;\n";
    let target = Some(("theorem", "TheoremItem", "theorem T: true;"));
    let (left, lp, _) = doc7_fixture("doc7-left", text, target, |_, _, rows| {
        rows.insert(
            0,
            doc7_landmark("ImportItem", 0, "import X;".len(), None, None),
        );
    });
    let (right, rp, _) = doc7_fixture("doc7-right", text, target, |_, _, rows| {
        rows.insert(
            0,
            doc7_landmark("ImportItem", 0, "import X;".len(), None, None),
        );
    });
    let la = mizar_doc::artifact_reader::read_published_artifacts(&left, lp.manifest.artifact_hash)
        .unwrap()
        .1;
    let ra =
        mizar_doc::artifact_reader::read_published_artifacts(&right, rp.manifest.artifact_hash)
            .unwrap()
            .1;
    let (registry, request_id, snapshot, workspace) = doc_build_registry(&[
        (&left, &lp.manifest.manifest, &la),
        (&right, &rp.manifest.manifest, &ra),
    ]);
    let mut request = mizar_doc::doc_build::DocBuildRequest {
        profile: lp.profile,
        publications: vec![lp.request.clone(), rp.request.clone()],
    };
    let plans = mizar_doc::doc_build::build_doc_plans(
        &registry,
        request_id,
        snapshot.id,
        &workspace,
        &request,
    )
    .unwrap();
    request.publications.reverse();
    let reversed = mizar_doc::doc_build::build_doc_plans(
        &registry,
        request_id,
        snapshot.id,
        &workspace,
        &request,
    )
    .unwrap();
    assert_eq!(plans, reversed);
    let (first, second) = (doc7_attach(&plans[0]), doc7_attach(&plans[1]));
    assert_ne!(first.module, second.module);
    assert_eq!(
        first.input.exports[0].origin_id,
        second.input.exports[0].origin_id
    );
    assert_eq!(
        first.input.exports[0].fully_qualified_name,
        second.input.exports[0].fully_qualified_name
    );
    for (plan, replay) in plans.iter().zip(&reversed) {
        let result = doc7_attach(plan);
        assert!(
            matches!(result.blocks[0].target, Some(mizar_doc::comments::CommentTarget::Item(export)) if std::ptr::eq(export, &result.input.exports[0]))
        );
        assert_eq!(
            format!("{:?}", result.blocks),
            format!("{:?}", doc7_attach(replay).blocks)
        );
        assert_eq!(result.warnings, doc7_attach(replay).warnings);
    }
    std::fs::remove_dir_all(left).unwrap();
    std::fs::remove_dir_all(right).unwrap();
}

fn doc8_fixture(
    markdown: &str,
) -> (
    PathBuf,
    mizar_doc::doc_build::DocBuildPlan,
    mizar_session::LoadedSource,
) {
    let source = markdown
        .lines()
        .map(|line| format!("::: {line}\n"))
        .collect::<String>();
    doc7_fixture("doc8", &source, None, |_, _, _| {})
}

fn doc8_codes(warnings: &[mizar_diagnostics::failure_record::DiagnosticDraft]) -> Vec<String> {
    warnings
        .iter()
        .map(|warning| warning.code().to_string())
        .collect()
}

#[test]
fn doc8_paragraph_frontiers_and_literal_markers() {
    use mizar_doc::markdown::parse_comments;
    let markdown = "@param a\n@param b\n@returns\n@requires\n@see\n\n- @param tight\n  - @see nested\n\n- @since loose\n\n  @ensures prose\n\n> @deprecated v message\n>\n> @returns **\n\n# @param heading\n\n    @param indented\n\n```\n@param fenced\n```\n\n<div>\n@param html\n</div>\n\n[@param]: /url\n\n\\@param escape\n\n&#64;param entity\n\n`@param` code\n\n*@param* emphasis\n\n[@param](/url) link\n\n@PARAM uppercase\n\n@param, punctuation\n\n@param\u{a0}unicode\n\n@unknown untouched\n";
    let (root, plan, _) = doc8_fixture(markdown);
    let attachments = doc7_attach(&plan);
    let parsed = parse_comments(&attachments, &[]);
    assert!(parsed.warnings.is_empty(), "{:?}", parsed.warnings);
    let block = &parsed.blocks[0];
    assert_eq!(block.text, markdown);
    assert_eq!(
        block
            .fields
            .iter()
            .map(|field| &block.text[field.marker.clone()])
            .collect::<Vec<_>>(),
        [
            "@param",
            "@param",
            "@see",
            "@since",
            "@ensures",
            "@deprecated",
            "@returns"
        ]
    );
    let first = &block.fields[0];
    assert_eq!(&block.text[first.slot.clone().unwrap()], "a");
    assert_eq!(
        &block.text[first.payload.clone()],
        " a\n@param b\n@returns\n@requires\n@see"
    );
    assert_eq!(
        first
            .body
            .iter()
            .map(|r| &block.text[r.clone()])
            .collect::<String>(),
        "@param b\n@returns\n@requires\n@see"
    );
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn doc8_raw_slots_breaks_and_required_bodies() {
    use mizar_doc::markdown::parse_comments;
    let markdown = "@param\n`a&amp;`  explanation  \ncontinued\n\n@see\n[opaque](ref)\n\n@since \"v raw\n\n@deprecated v\nmessage\n\n@returns \u{a0}\n\n@requires **\n\n@ensures &amp;\n\n@param x\u{a0}y\n";
    let (root, plan, _) = doc8_fixture(markdown);
    let attachments = doc7_attach(&plan);
    let parsed = parse_comments(&attachments, &[]);
    assert!(parsed.warnings.is_empty(), "{:?}", parsed.warnings);
    let block = &parsed.blocks[0];
    assert_eq!(
        block
            .fields
            .iter()
            .map(|f| f.slot.as_ref().map(|r| &block.text[r.clone()]))
            .collect::<Vec<_>>(),
        [
            Some("`a&amp;`"),
            Some("[opaque](ref)"),
            Some("\"v"),
            Some("v"),
            None,
            None,
            None,
            Some("x\u{a0}y")
        ]
    );
    assert_eq!(
        block.fields[0]
            .body
            .iter()
            .map(|r| &block.text[r.clone()])
            .collect::<String>(),
        "explanation  \ncontinued"
    );
    assert_eq!(
        &block.text[block.fields[0].payload.clone()],
        "\n`a&amp;`  explanation  \ncontinued"
    );
    assert_eq!(
        block.fields[3]
            .body
            .iter()
            .map(|r| &block.text[r.clone()])
            .collect::<String>(),
        "message"
    );
    std::fs::remove_dir_all(root).unwrap();

    let (root, plan, _) = doc8_fixture(
        "@param\n\n@see\n\n@since\n\n@deprecated\n\n@returns \t\n\n@requires\n\n@ensures\n",
    );
    let attachments = doc7_attach(&plan);
    let parsed = parse_comments(&attachments, &[]);
    assert_eq!(
        doc8_codes(&parsed.warnings),
        [
            "W0405", "W0405", "W0405", "W0405", "W0406", "W0406", "W0406", "W0406"
        ]
    );
    use mizar_diagnostics::failure_record::DiagnosticDetailValue as V;
    assert_eq!(
        parsed
            .warnings
            .iter()
            .map(
                |warning| match &warning.details().entries()["documentation.subreason"] {
                    V::String(s) => s.as_str(),
                    _ => panic!("subreason must be a string"),
                }
            )
            .collect::<Vec<_>>(),
        [
            "name",
            "reference",
            "version",
            "version",
            "deprecated",
            "returns",
            "requires",
            "ensures"
        ]
    );
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn doc8_exact_duplicates_span_blocks_and_keep_all_fields() {
    use mizar_doc::markdown::parse_comments;
    let markdown = "@param a first\n\n@param a second\n\n@param A distinct\n\n@param a&amp; distinct\n\n@param a& distinct\n\n@returns first\n\n@returns second\n\n@since v1\n\n@since v2\n\n@deprecated v1 first\n\n@deprecated v2 second\n\n@requires same\n\n@requires same\n\n@requires  same\n\n@ensures same\n\n@ensures same \n\n@ensures same\n\n@see ref\n\n@see ref\n\n@see REF\n";
    let (root, plan, _) = doc8_fixture(markdown);
    let attachments = doc7_attach(&plan);
    let parsed = parse_comments(&attachments, &[]);
    assert_eq!(parsed.blocks[0].fields.len(), 20);
    assert_eq!(doc8_codes(&parsed.warnings), vec!["W0407"; 7]);
    assert_eq!(parsed.blocks[0].text, markdown);
    std::fs::remove_dir_all(root).unwrap();

    let text = "::: @param a first\n\n::: @param a second\nimport X;\n::: @param a item\ntheorem T: true;\n";
    let (root, plan, _) = doc7_fixture(
        "doc8",
        text,
        Some(("theorem", "TheoremItem", "theorem T: true;")),
        |_, _, landmarks| {
            let start = text.find("import X;").unwrap();
            landmarks.insert(
                0,
                doc7_landmark("ImportItem", start, start + "import X;".len(), None, None),
            );
        },
    );
    let attachments = doc7_attach(&plan);
    assert!(attachments.warnings.is_empty());
    let parsed = parse_comments(&attachments, &[]);
    assert_eq!(parsed.blocks.len(), 3);
    assert_eq!(doc8_codes(&parsed.warnings), ["W0407"]);
    assert_ne!(attachments.blocks[1].target, attachments.blocks[2].target);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn doc8_metadata_is_authoritative_complete_and_target_specific() {
    use mizar_doc::{
        comments::CommentTarget,
        markdown::{TargetMetadata, parse_comments},
    };
    let text = "import X;\n::: @param absentλ body\n:::\n::: @returns value\ntheorem T: true;\n";
    let (root, plan, source) = doc7_fixture(
        "doc8",
        text,
        Some(("theorem", "TheoremItem", "theorem T: true;")),
        |artifact, _, landmarks| {
            artifact.exports[0].rendered_signature = "signature(absentλ) returns result".into();
            landmarks.insert(
                0,
                doc7_landmark("ImportItem", 0, "import X;".len(), None, None),
            );
        },
    );
    let unbound = doc7_attach(&plan);
    let attachments =
        mizar_doc::comments::attach_comments(&plan, unbound.module, Some(&source)).unwrap();
    let item = attachments.blocks[0].target.unwrap();
    assert!(matches!(item, CommentTarget::Item(_)));
    assert!(parse_comments(&attachments, &[]).warnings.is_empty());
    for (parameters, complete, returns, target, expected) in [
        (None, true, None, item, 0),
        (Some(&[][..]), false, None, item, 0),
        (Some(&["present"][..]), false, Some(true), item, 0),
        (Some(&["absentλ"][..]), true, Some(true), item, 0),
        (Some(&["Absentλ"][..]), true, None, item, 1),
        (Some(&[][..]), true, Some(false), item, 2),
        (Some(&[][..]), true, Some(false), CommentTarget::Module, 0),
    ] {
        let metadata = [TargetMetadata {
            target,
            parameters,
            parameters_complete: complete,
            has_return: returns,
        }];
        let parsed = parse_comments(&attachments, &metadata);
        assert_eq!(doc8_codes(&parsed.warnings), vec!["W0408"; expected]);
        if expected > 0 {
            use mizar_diagnostics::failure_record::{
                DiagnosticDetailValue as V, DiagnosticPrimaryLocation,
            };
            let start = text.find("absentλ").unwrap();
            assert!(
                matches!(parsed.warnings[0].primary_location(), DiagnosticPrimaryLocation::Span(span) if span.range() == mizar_session::SourceRange { source_id: source.source_id, start, end: start + "absentλ".len() })
            );
            assert_eq!(
                parsed.warnings[0].details().entries()["documentation.subreason"],
                V::String("parameter_absent".into())
            );
            if expected == 2 {
                let start = text.find("@returns").unwrap();
                assert!(
                    matches!(parsed.warnings[1].primary_location(), DiagnosticPrimaryLocation::Span(span) if span.range() == mizar_session::SourceRange { source_id: source.source_id, start, end: start + "@returns".len() })
                );
                assert_eq!(
                    parsed.warnings[1].details().entries()["documentation.subreason"],
                    V::String("no_return".into())
                );
            }
        }
    }
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn doc8_real_parser_diagnostics_replay_cli_and_profile_leave_inputs_unchanged() {
    use mizar_diagnostics::{
        aggregator::BuildDiagnosticIndex,
        failure_record::PipelinePhase,
        render::{
            DiagnosticRenderInput, DiagnosticSourceContext, RenderOptions, render_diagnostics,
        },
        sink::{DiagnosticProducerScope, DiagnosticSink},
    };
    use mizar_doc::markdown::parse_comments;
    struct PortableContext;
    impl DiagnosticSourceContext for PortableContext {
        fn path_for(&self, _: mizar_session::SourceId) -> Option<&str> {
            None
        }
        fn source_key_for(&self, _: mizar_session::SourceId) -> String {
            "portable".into()
        }
        fn line_text(&self, _: mizar_session::SourceId, _: u32) -> Option<&str> {
            None
        }
        fn line_column(
            &self,
            _: mizar_session::SourceRange,
        ) -> Option<mizar_session::LineColumnRange> {
            None
        }
    }
    let (root, mut plan, _) = doc8_fixture("@param\n\n@returns\n\n$unclosed\n");
    let saved = plan.clone();
    let paths = std::iter::once("artifact-manifest.json".to_owned())
        .chain(std::iter::once("src/source.miz".to_owned()))
        .chain(
            plan.manifest
                .manifest
                .modules
                .iter()
                .flat_map(|m| [Some(m.artifact_file.clone()), m.module_summary_file.clone()])
                .flatten(),
        )
        .collect::<Vec<_>>();
    let bytes = paths
        .iter()
        .map(|p| std::fs::read(root.join(p)).unwrap())
        .collect::<Vec<_>>();
    let attachments = doc7_attach(&plan);
    let raw = format!("{:?}", attachments.blocks);
    let parsed = parse_comments(&attachments, &[]);
    assert_eq!(doc8_codes(&parsed.warnings), ["W0405", "W0406", "W0409"]);
    assert!(!parsed.documentation_failed());
    use mizar_diagnostics::failure_record::DiagnosticDetailValue as V;
    for (warning, (start, end, subreason)) in
        parsed
            .warnings
            .iter()
            .zip([(0, 6, "name"), (8, 16, "returns"), (18, 19, "inline")])
    {
        let details = warning.details().entries();
        let V::List(occurrence) = &details["documentation.occurrence"] else {
            panic!("occurrence must be a list")
        };
        assert_eq!(occurrence.len(), 6);
        assert_eq!(occurrence[2], V::List(vec![V::String("module".into())]));
        assert_eq!(occurrence[3], V::String("0".into()));
        assert_eq!(occurrence[4], V::String(warning.stable_detail_key().into()));
        assert_eq!(
            occurrence[5],
            V::List(vec![
                V::String(start.to_string()),
                V::String(end.to_string())
            ])
        );
        assert_eq!(
            details["documentation.subreason"],
            V::String(subreason.into())
        );
    }
    assert_eq!(
        parsed.warnings[0].details().entries()["documentation.raw"],
        V::List(vec![
            V::String("@param".into()),
            V::String(String::new()),
            V::List(vec![])
        ])
    );
    assert_eq!(
        parsed.warnings[2].details().entries()["documentation.raw"],
        V::List(vec![])
    );
    assert_eq!(parsed.warnings, parse_comments(&attachments, &[]).warnings);
    let drafts = parsed.warnings.clone();
    let snapshot = drafts[0].source_snapshot();
    let mut sink = DiagnosticSink::new(DiagnosticProducerScope::new(
        PipelinePhase::Documentation,
        snapshot,
        "doc8-parser",
    ));
    for warning in drafts.iter().cloned() {
        sink.emit(warning).unwrap();
    }
    let index = BuildDiagnosticIndex::from_batches(snapshot, vec![sink.into_batch()]).unwrap();
    let mut reversed = DiagnosticSink::new(DiagnosticProducerScope::new(
        PipelinePhase::Documentation,
        snapshot,
        "doc8-parser",
    ));
    for warning in drafts.iter().rev().cloned() {
        reversed.emit(warning).unwrap();
    }
    let replay = BuildDiagnosticIndex::from_batches(snapshot, vec![reversed.into_batch()]).unwrap();
    assert_eq!(index.records(), replay.records());
    assert_eq!(index.len(), 3);
    let output = render_diagnostics(DiagnosticRenderInput::new(
        index.records(),
        &PortableContext,
        RenderOptions::plain(),
    ));
    for code in ["W0405", "W0406", "W0409"] {
        assert!(output.contains(code), "{output}");
    }
    assert!(output.contains("doc8"), "{output}");
    drop(parsed);
    drop(attachments);
    plan.profile.include_private_docs = true;
    plan.profile.fail_on_doc_warnings = true;
    let promoted = doc7_attach(&plan);
    let parsed = parse_comments(&promoted, &[]);
    assert!(parsed.documentation_failed());
    assert_eq!(parsed.warnings, drafts);
    assert_eq!(format!("{:?}", promoted.blocks), raw);
    assert_eq!(plan.index, saved.index);
    assert_eq!(
        bytes,
        paths
            .iter()
            .map(|p| std::fs::read(root.join(p)).unwrap())
            .collect::<Vec<_>>()
    );
    let absent = plan
        .index
        .modules
        .keys()
        .find(|m| m.module_path == "module_1")
        .unwrap();
    let unavailable = mizar_doc::comments::attach_comments(&plan, absent, None).unwrap();
    let unavailable = parse_comments(&unavailable, &[]);
    assert!(!unavailable.available());
    assert!(unavailable.blocks.is_empty());
    assert!(unavailable.warnings.is_empty());
    std::fs::remove_dir_all(root).unwrap();
    let (root, plan, _) = doc8_fixture("");
    let empty = doc7_attach(&plan);
    let empty = parse_comments(&empty, &[]);
    assert!(empty.available());
    assert!(empty.blocks.is_empty());
    assert!(empty.warnings.is_empty());
    std::fs::remove_dir_all(root).unwrap();
}

#[path = "doc8_math.rs"]
mod doc8_math;

#[test]
fn doc8_production_parser_disables_optional_commonmark_extensions() {
    let (root, plan, _) = doc8_fixture(
        "~~literal~~\n\n| a | b |\n| - | - |\n| x | y |\n\n- [x] literal\n\n[^note]: ordinary\n\n[^note]\n\n# heading {#id}\n\n---\ntitle: ordinary\n---\n",
    );
    let attachments = doc7_attach(&plan);
    let parsed = mizar_doc::markdown::parse_comments(&attachments, &[]);
    let events = format!("{:?}", parsed.blocks[0].events);
    let block = &parsed.blocks[0];
    let text = block
        .events
        .iter()
        .filter(|(event, _)| format!("{event:?}").starts_with("Text("))
        .map(|(_, range)| &block.text[range.clone()])
        .collect::<String>();
    assert!(text.contains("~~literal~~"));
    assert!(!events.contains("Strikethrough"));
    assert!(!events.contains("Table"));
    assert!(!events.contains("TaskListMarker"));
    assert!(!events.contains("FootnoteDefinition"));
    assert!(!events.contains("FootnoteReference"));
    assert!(!events.contains("MetadataBlock"));
    assert!(!events.contains("Some(\"id\")"));
    assert!(parsed.warnings.is_empty());
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn doc8_tight_paragraphs_preserve_raw_trailing_whitespace_duplicate_keys() {
    let (root, plan, _) = doc8_fixture(
        "- @requires x  \n- @requires x\n- @requires x  \n- @see ref\n  continued\t \n- @see ref\n  continued\n",
    );
    let attachments = doc7_attach(&plan);
    let parsed = mizar_doc::markdown::parse_comments(&attachments, &[]);
    let block = &parsed.blocks[0];
    assert_eq!(
        block
            .fields
            .iter()
            .map(|field| &block.text[field.payload.clone()])
            .collect::<Vec<_>>(),
        [
            " x  ",
            " x",
            " x  ",
            " ref\n  continued\t ",
            " ref\n  continued"
        ]
    );
    assert_eq!(doc8_codes(&parsed.warnings), ["W0407"]);
    assert_eq!(
        block.fields[0]
            .body
            .iter()
            .map(|range| &block.text[range.clone()])
            .collect::<String>(),
        "x  "
    );
    std::fs::remove_dir_all(root).unwrap();
}
