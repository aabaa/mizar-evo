use super::{
    DiskSourceLoader, LoadedSource, NormalizedPath, SourceInput, SourceLoadError, SourceLoader,
    SourceOriginInput, SourceOriginKind, SourcePathError, file_path_from_document_uri, hash_text,
    normalize_path, normalize_source_path, reject_non_canonical_alias,
};
use crate::{
    BuildRequestId, BuildSessionId, BuildSnapshotId, Edition, GeneratedSourceKind, IdError,
    InMemorySessionIdAllocator, LineMap, LoadingMapSegment, LoadingOrigin, ModulePath, PackageId,
    SessionIdAllocator, SnapshotLeaseId, SourceAnchor, SourceId, SourceMapId, SourceOrigin,
    SourceRange, TextRange,
};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

static NEXT_ID: AtomicUsize = AtomicUsize::new(0);

#[test]
fn source_hash_is_text_only_and_excludes_path_or_document_version_metadata() {
    let text = "environ\nbegin\n";
    let snapshot = snapshot_id(1);
    let ids = InMemorySessionIdAllocator::new();
    let loader = TextOnlyLoader::with_disk_text(text);
    let disk = loader
        .load(
            snapshot,
            source_input(SourceOriginInput::Disk {
                path: PathBuf::from("/absolute/package/src/basic.miz"),
            }),
            &ids,
        )
        .unwrap();
    let same_text_different_path = loader
        .load(
            snapshot,
            source_input(SourceOriginInput::Disk {
                path: PathBuf::from("/other/root/src/basic.miz"),
            }),
            &ids,
        )
        .unwrap();
    let open_v1 = loader
        .load(
            snapshot,
            source_input(SourceOriginInput::OpenBuffer {
                uri: "file:///absolute/package/src/basic.miz".to_owned(),
                expected_version: 1,
                actual_version: 1,
                text: Arc::from(text),
            }),
            &ids,
        )
        .unwrap();
    let open_v2 = loader
        .load(
            snapshot,
            source_input(SourceOriginInput::OpenBuffer {
                uri: "file:///absolute/package/src/basic.miz".to_owned(),
                expected_version: 2,
                actual_version: 2,
                text: Arc::from(text),
            }),
            &ids,
        )
        .unwrap();

    assert_eq!(disk.source_hash, hash_text(text));
    assert_eq!(disk.source_hash, same_text_different_path.source_hash);
    assert_eq!(disk.source_hash, open_v1.source_hash);
    assert_eq!(open_v1.source_hash, open_v2.source_hash);
    assert_eq!(hash_text(text), disk.line_map.text_hash());
}

#[test]
fn identical_text_hashes_match_across_disk_open_buffer_and_generated_origins() {
    let text = "theorem Th1;\n";
    let snapshot = snapshot_id(2);
    let ids = InMemorySessionIdAllocator::new();
    let loader = TextOnlyLoader::with_disk_text(text);
    let disk = loader
        .load(
            snapshot,
            source_input(SourceOriginInput::Disk {
                path: PathBuf::from("/absolute/package/src/basic.miz"),
            }),
            &ids,
        )
        .unwrap();
    let open_buffer = loader
        .load(
            snapshot,
            source_input(SourceOriginInput::OpenBuffer {
                uri: "file:///absolute/package/src/basic.miz".to_owned(),
                expected_version: 4,
                actual_version: 4,
                text: Arc::from(text),
            }),
            &ids,
        )
        .unwrap();
    let generated = loader
        .load(
            snapshot,
            source_input(SourceOriginInput::Generated {
                generator: GeneratedSourceKind::new("test-generator"),
                text: Arc::from(text),
                anchor: None,
            }),
            &ids,
        )
        .unwrap();

    assert_eq!(disk.source_hash, open_buffer.source_hash);
    assert_eq!(open_buffer.source_hash, generated.source_hash);
    assert!(matches!(disk.origin, SourceOrigin::Disk));
    assert!(matches!(
        open_buffer.origin,
        SourceOrigin::OpenBuffer { version: 4 }
    ));
    assert!(matches!(
        generated.origin,
        SourceOrigin::Generated { ref generator } if generator.as_str() == "test-generator"
    ));
}

#[test]
fn source_loader_load_uses_snapshot_scoped_source_id_allocation() {
    let snapshot = snapshot_id(3);
    let allocator = RecordingAllocator::new();
    let loader = TextOnlyLoader::default();
    let input = source_input(SourceOriginInput::OpenBuffer {
        uri: "file:///package/src/basic.miz".to_owned(),
        expected_version: 7,
        actual_version: 7,
        text: Arc::from("environ\nbegin\n"),
    });

    let loaded = loader.load(snapshot, input, &allocator).unwrap();

    assert_eq!(allocator.source_snapshots(), vec![snapshot]);
    assert_eq!(loaded.line_map.source_id(), loaded.source_id);
    assert_eq!(loaded.line_map.text_hash(), loaded.source_hash);
    assert!(matches!(
        loaded.origin,
        SourceOrigin::OpenBuffer { version: 7 }
    ));
}

#[test]
fn loaded_source_origin_reuses_snapshot_source_origin() {
    let loaded = loaded_source(
        InMemorySessionIdAllocator::new()
            .next_source_id(snapshot_id(4))
            .unwrap(),
        SourceOrigin::Generated {
            generator: GeneratedSourceKind::new("doc-extract"),
        },
        "definition X;\n",
    );

    let origin: SourceOrigin = loaded.origin.clone();

    assert!(matches!(
        origin,
        SourceOrigin::Generated { generator } if generator.as_str() == "doc-extract"
    ));
}

#[test]
fn source_loader_normalize_path_reuses_source_path_normalization() {
    let package = PackageFixture::new();
    package.write("src/groups/basic.miz", "");
    package.write("src/groups/basic.txt", "");
    package.write("other/basic.miz", "");
    let loader = TextOnlyLoader::default();

    let normalized = loader
        .normalize_path(package.root(), Path::new("src/groups/basic.miz"))
        .unwrap();
    let unsupported = normalize_path(package.root(), Path::new("src/groups/basic.txt"))
        .expect_err("non-miz source should be rejected");
    let missing_source_root = normalize_path(package.root(), Path::new("other/basic.miz"))
        .expect_err("package file outside src should be rejected");

    assert_eq!(normalized, path("src/groups/basic.miz"));
    assert!(matches!(
        unsupported,
        SourceLoadError::UnsupportedFileExtension { .. }
    ));
    assert_missing_source_root_error(&missing_source_root, "other/basic.miz");
}

#[test]
fn disk_source_loader_reads_disk_text_and_builds_loaded_source_metadata() {
    let package = PackageFixture::new();
    package.write("src/groups/basic.miz", "environ\nbegin\n");
    let loader = DiskSourceLoader::new(package.root());
    let ids = InMemorySessionIdAllocator::new();

    let loaded = loader
        .load(
            snapshot_id(20),
            disk_source_input("src/groups/basic.miz"),
            &ids,
        )
        .unwrap();

    assert_eq!(loaded.text.as_ref(), "environ\nbegin\n");
    assert_eq!(loaded.normalized_path, path("src/groups/basic.miz"));
    assert_eq!(loaded.source_hash, hash_text("environ\nbegin\n"));
    assert_eq!(loaded.line_map.source(), "environ\nbegin\n");
    assert_eq!(loaded.line_map.line_starts(), &[0, 8, 14]);
    assert_eq!(loaded.line_map.text_hash(), loaded.source_hash);
    assert!(matches!(loaded.origin, SourceOrigin::Disk));
    assert!(loaded.loading_map.is_none());
}

#[test]
fn disk_source_loader_rejects_invalid_utf8_before_source_id_allocation() {
    let package = PackageFixture::new();
    package.write_bytes("src/groups/invalid.miz", &[0xff]);
    let loader = DiskSourceLoader::new(package.root());
    let allocator = RecordingAllocator::new();

    let error = loader
        .load(
            snapshot_id(21),
            disk_source_input("src/groups/invalid.miz"),
            &allocator,
        )
        .expect_err("invalid UTF-8 should be rejected");

    assert!(matches!(
        error,
        SourceLoadError::InvalidUtf8 { path: Some(path) }
            if path.ends_with("src/groups/invalid.miz")
    ));
    assert!(allocator.source_snapshots().is_empty());
}

#[test]
fn disk_source_loader_rejects_unsupported_extension_before_decoding() {
    let package = PackageFixture::new();
    package.write_bytes("src/groups/basic.txt", &[0xff]);
    let loader = DiskSourceLoader::new(package.root());

    let error = loader
        .load(
            snapshot_id(22),
            disk_source_input("src/groups/basic.txt"),
            &InMemorySessionIdAllocator::new(),
        )
        .expect_err("non-miz extension should be rejected");

    assert!(matches!(
        error,
        SourceLoadError::UnsupportedFileExtension { path }
            if path.ends_with("src/groups/basic.txt")
    ));
}

#[test]
fn disk_source_loader_strips_leading_bom_and_maps_loaded_zero_to_original_three() {
    let package = PackageFixture::new();
    package.write_bytes("src/groups/bom.miz", b"\xef\xbb\xbfalpha");
    let loader = DiskSourceLoader::new(package.root());
    let loaded = loader
        .load(
            snapshot_id(23),
            disk_source_input("src/groups/bom.miz"),
            &InMemorySessionIdAllocator::new(),
        )
        .unwrap();

    assert_eq!(loaded.text.as_ref(), "alpha");
    assert_eq!(loaded.source_hash, hash_text("alpha"));
    assert_ne!(loaded.source_hash, hash_text("\u{feff}alpha"));
    let loading_map = loaded
        .loading_map
        .as_ref()
        .expect("leading BOM should emit a loading map");
    assert_eq!(
        loading_map.origin,
        LoadingOrigin::DiskBytes {
            normalized_path: path("src/groups/bom.miz")
        }
    );
    assert_eq!(
        loading_map.segments,
        vec![
            LoadingMapSegment::RemovedLeadingBom {
                original: TextRange { start: 0, end: 3 },
            },
            LoadingMapSegment::Original {
                loaded: TextRange { start: 0, end: 5 },
                original: TextRange { start: 3, end: 8 },
            },
        ]
    );
    assert_eq!(
        loading_map.original_offset_for_loaded(loaded.source_id, 0),
        Ok(3)
    );
}

#[test]
fn disk_source_loader_maps_file_that_contains_only_bom_to_original_eof() {
    let package = PackageFixture::new();
    package.write_bytes("src/groups/bom_only.miz", b"\xef\xbb\xbf");
    let loader = DiskSourceLoader::new(package.root());

    let loaded = loader
        .load(
            snapshot_id(24),
            disk_source_input("src/groups/bom_only.miz"),
            &InMemorySessionIdAllocator::new(),
        )
        .unwrap();

    assert_eq!(loaded.text.as_ref(), "");
    assert_eq!(loaded.source_hash, hash_text(""));
    let loading_map = loaded
        .loading_map
        .as_ref()
        .expect("leading BOM should emit a loading map");
    assert_eq!(
        loading_map.segments,
        vec![
            LoadingMapSegment::RemovedLeadingBom {
                original: TextRange { start: 0, end: 3 },
            },
            LoadingMapSegment::Original {
                loaded: TextRange { start: 0, end: 0 },
                original: TextRange { start: 3, end: 3 },
            },
        ]
    );
    assert_eq!(
        loading_map.original_offset_for_loaded(loaded.source_id, 0),
        Ok(3)
    );
}

#[test]
fn disk_source_loader_preserves_non_leading_bom() {
    let package = PackageFixture::new();
    package.write("src/groups/non_leading_bom.miz", "alpha\u{feff}beta");
    let loader = DiskSourceLoader::new(package.root());

    let loaded = loader
        .load(
            snapshot_id(24),
            disk_source_input("src/groups/non_leading_bom.miz"),
            &InMemorySessionIdAllocator::new(),
        )
        .unwrap();

    assert_eq!(loaded.text.as_ref(), "alpha\u{feff}beta");
    assert!(loaded.loading_map.is_none());
}

#[test]
fn disk_source_loader_normalizes_crlf_but_preserves_lone_cr() {
    let package = PackageFixture::new();
    package.write("src/groups/newlines.miz", "alpha\r\nbeta\rgamma\r\n");
    let loader = DiskSourceLoader::new(package.root());

    let loaded = loader
        .load(
            snapshot_id(25),
            disk_source_input("src/groups/newlines.miz"),
            &InMemorySessionIdAllocator::new(),
        )
        .unwrap();

    assert_eq!(loaded.text.as_ref(), "alpha\nbeta\rgamma\n");
    assert_eq!(loaded.source_hash, hash_text("alpha\nbeta\rgamma\n"));
    assert_ne!(loaded.source_hash, hash_text("alpha\r\nbeta\rgamma\r\n"));
    assert_eq!(loaded.line_map.line_starts(), &[0, 6, 17]);
    assert_eq!(
        loaded.loading_map.as_ref().map(|map| &map.segments),
        Some(&vec![
            LoadingMapSegment::Original {
                loaded: TextRange { start: 0, end: 5 },
                original: TextRange { start: 0, end: 5 },
            },
            LoadingMapSegment::NormalizedNewline {
                loaded: TextRange { start: 5, end: 6 },
                original: TextRange { start: 5, end: 7 },
            },
            LoadingMapSegment::Original {
                loaded: TextRange { start: 6, end: 16 },
                original: TextRange { start: 7, end: 17 },
            },
            LoadingMapSegment::NormalizedNewline {
                loaded: TextRange { start: 16, end: 17 },
                original: TextRange { start: 17, end: 19 },
            },
        ])
    );
}

#[test]
fn disk_source_loader_combines_leading_bom_and_crlf_loading_map_offsets() {
    let package = PackageFixture::new();
    package.write_bytes("src/groups/bom_crlf.miz", b"\xef\xbb\xbfalpha\r\nbeta");
    let loader = DiskSourceLoader::new(package.root());

    let loaded = loader
        .load(
            snapshot_id(26),
            disk_source_input("src/groups/bom_crlf.miz"),
            &InMemorySessionIdAllocator::new(),
        )
        .unwrap();

    assert_eq!(loaded.text.as_ref(), "alpha\nbeta");
    assert_eq!(loaded.source_hash, hash_text("alpha\nbeta"));
    assert_ne!(loaded.source_hash, hash_text("\u{feff}alpha\r\nbeta"));
    let loading_map = loaded
        .loading_map
        .as_ref()
        .expect("combined normalization should emit a loading map");
    assert_eq!(
        loading_map.segments,
        vec![
            LoadingMapSegment::RemovedLeadingBom {
                original: TextRange { start: 0, end: 3 },
            },
            LoadingMapSegment::Original {
                loaded: TextRange { start: 0, end: 5 },
                original: TextRange { start: 3, end: 8 },
            },
            LoadingMapSegment::NormalizedNewline {
                loaded: TextRange { start: 5, end: 6 },
                original: TextRange { start: 8, end: 10 },
            },
            LoadingMapSegment::Original {
                loaded: TextRange { start: 6, end: 10 },
                original: TextRange { start: 10, end: 14 },
            },
        ]
    );
    assert_eq!(
        loading_map.original_offset_for_loaded(loaded.source_id, 0),
        Ok(3)
    );
    assert_eq!(
        loading_map.original_offset_for_loaded(loaded.source_id, 5),
        Ok(8)
    );
    assert_eq!(
        loading_map.original_offset_for_loaded(loaded.source_id, 6),
        Ok(10)
    );
}

#[test]
fn disk_source_loader_reads_through_normalized_dot_component_path() {
    let package = PackageFixture::new();
    package.write("src/groups/dotted.miz", "environ\n");
    let loader = DiskSourceLoader::new(package.root());

    let loaded = loader
        .load(
            snapshot_id(27),
            disk_source_input("src/./groups/../groups/dotted.miz"),
            &InMemorySessionIdAllocator::new(),
        )
        .unwrap();

    assert_eq!(loaded.normalized_path, path("src/groups/dotted.miz"));
    assert_eq!(loaded.text.as_ref(), "environ\n");
}

#[test]
fn disk_source_loader_rejects_paths_outside_package_root() {
    let package = PackageFixture::new();
    package.write_outside("outside.miz", "environ\n");
    let loader = DiskSourceLoader::new(package.root());

    let error = loader
        .load(
            snapshot_id(28),
            source_input(SourceOriginInput::Disk {
                path: package.outside_path("outside.miz"),
            }),
            &InMemorySessionIdAllocator::new(),
        )
        .expect_err("outside package path should be rejected");

    assert!(matches!(
        &error,
        SourceLoadError::SourcePathOutsidePackageRoot { .. }
    ));
    assert!(
        error.to_string().contains("must stay inside package root"),
        "outside-root error should keep the package-root boundary message: {error}"
    );
}

#[test]
fn disk_source_loader_reports_missing_source_root_as_invalid_source_path() {
    let package = PackageFixture::new();
    package.write("other/basic.miz", "environ\n");
    let loader = DiskSourceLoader::new(package.root());
    let allocator = RecordingAllocator::new();

    let error = loader
        .load(
            snapshot_id(29),
            disk_source_input("other/basic.miz"),
            &allocator,
        )
        .expect_err("package file outside src should be rejected");

    assert_missing_source_root_error(&error, "other/basic.miz");
    assert!(allocator.source_snapshots().is_empty());
}

#[cfg(not(windows))]
#[test]
fn document_uri_parser_maps_posix_local_file_uris_and_rejects_remote_authority() {
    assert_eq!(
        file_path_from_document_uri("file:///tmp/pkg/src/basic%20case.miz"),
        Some(PathBuf::from("/tmp/pkg/src/basic case.miz"))
    );
    assert_eq!(
        file_path_from_document_uri("file://localhost/tmp/pkg/src/basic.miz"),
        Some(PathBuf::from("/tmp/pkg/src/basic.miz"))
    );
    assert_eq!(
        file_path_from_document_uri("file://server/share/pkg/src/basic.miz"),
        None
    );
}

#[cfg(windows)]
#[test]
fn document_uri_parser_maps_windows_drive_and_unc_file_uris() {
    assert_eq!(
        file_path_from_document_uri("file:///C:/pkg/src/basic%20case.miz"),
        Some(PathBuf::from(r"C:\pkg\src\basic case.miz"))
    );
    assert_eq!(
        file_path_from_document_uri("file://localhost/C:/pkg/src/basic.miz"),
        Some(PathBuf::from(r"C:\pkg\src\basic.miz"))
    );
    assert_eq!(
        file_path_from_document_uri("file://server/share/pkg/src/basic.miz"),
        Some(PathBuf::from(r"\\server\share\pkg\src\basic.miz"))
    );
}

#[test]
fn open_buffer_loader_overrides_disk_when_document_version_matches() {
    let package = PackageFixture::new();
    package.write("src/groups/basic.miz", "disk\n");
    let loader = DiskSourceLoader::new(package.root());
    let uri = package.file_uri("src/groups/basic.miz");

    let loaded = loader
        .load(
            snapshot_id(29),
            source_input(SourceOriginInput::OpenBuffer {
                uri: uri.clone(),
                expected_version: 5,
                actual_version: 5,
                text: Arc::from("\u{feff}alpha\r\nbeta\rgamma"),
            }),
            &InMemorySessionIdAllocator::new(),
        )
        .unwrap();

    assert_eq!(loaded.text.as_ref(), "alpha\nbeta\rgamma");
    assert_eq!(loaded.normalized_path, path("src/groups/basic.miz"));
    assert_eq!(loaded.source_hash, hash_text("alpha\nbeta\rgamma"));
    assert_ne!(loaded.source_hash, hash_text("disk\n"));
    assert_eq!(loaded.line_map.line_starts(), &[0, 6]);
    assert!(matches!(
        loaded.origin,
        SourceOrigin::OpenBuffer { version: 5 }
    ));
    assert_eq!(loaded.generated_anchor, None);
    let loading_map = loaded
        .loading_map
        .as_ref()
        .expect("open-buffer BOM/CRLF normalization should emit a loading map");
    assert_eq!(
        loading_map.origin,
        LoadingOrigin::OpenBufferText { uri, version: 5 }
    );
    assert_eq!(
        loading_map.segments,
        vec![
            LoadingMapSegment::RemovedLeadingBom {
                original: TextRange { start: 0, end: 3 },
            },
            LoadingMapSegment::Original {
                loaded: TextRange { start: 0, end: 5 },
                original: TextRange { start: 3, end: 8 },
            },
            LoadingMapSegment::NormalizedNewline {
                loaded: TextRange { start: 5, end: 6 },
                original: TextRange { start: 8, end: 10 },
            },
            LoadingMapSegment::Original {
                loaded: TextRange { start: 6, end: 16 },
                original: TextRange { start: 10, end: 20 },
            },
        ]
    );
    assert_eq!(
        loading_map.original_offset_for_loaded(loaded.source_id, 0),
        Ok(3)
    );
    assert_eq!(
        loading_map.original_offset_for_loaded(loaded.source_id, 5),
        Ok(8)
    );
    assert_eq!(
        loading_map.original_offset_for_loaded(loaded.source_id, 6),
        Ok(10)
    );
}

#[test]
fn open_buffer_loader_rejects_stale_document_version_before_source_id_allocation() {
    let package = PackageFixture::new();
    package.write("src/groups/basic.miz", "disk\n");
    let loader = DiskSourceLoader::new(package.root());
    let allocator = RecordingAllocator::new();

    let error = loader
        .load(
            snapshot_id(30),
            source_input(SourceOriginInput::OpenBuffer {
                uri: package.file_uri("src/groups/basic.miz"),
                expected_version: 8,
                actual_version: 7,
                text: Arc::from("open\n"),
            }),
            &allocator,
        )
        .expect_err("stale open buffer should be rejected");

    assert!(matches!(
        error,
        SourceLoadError::StaleLspDocumentVersion {
            expected: 8,
            actual: 7
        }
    ));
    assert!(allocator.source_snapshots().is_empty());

    let disk = loader
        .load(
            snapshot_id(30),
            disk_source_input("src/groups/basic.miz"),
            &InMemorySessionIdAllocator::new(),
        )
        .unwrap();
    assert_eq!(disk.text.as_ref(), "disk\n");
}

#[test]
fn open_buffer_loader_rejects_negative_expected_version_before_source_id_allocation() {
    let package = PackageFixture::new();
    package.write("src/groups/basic.miz", "disk\n");
    let loader = DiskSourceLoader::new(package.root());
    let allocator = RecordingAllocator::new();

    let error = loader
        .load(
            snapshot_id(31),
            source_input(SourceOriginInput::OpenBuffer {
                uri: package.file_uri("src/groups/basic.miz"),
                expected_version: -1,
                actual_version: -1,
                text: Arc::from("open\n"),
            }),
            &allocator,
        )
        .expect_err("negative open-buffer version should be rejected");

    assert!(matches!(
        error,
        SourceLoadError::StaleLspDocumentVersion {
            expected: 0,
            actual: -1
        }
    ));
    assert!(allocator.source_snapshots().is_empty());
}

#[test]
fn open_buffer_loader_rejects_unmappable_uri_before_source_id_allocation() {
    let package = PackageFixture::new();
    let loader = DiskSourceLoader::new(package.root());
    let allocator = RecordingAllocator::new();

    for uri in ["untitled:basic".to_owned(), "file:///%FF".to_owned()] {
        let error = loader
            .load(
                snapshot_id(31),
                source_input(SourceOriginInput::OpenBuffer {
                    uri: uri.clone(),
                    expected_version: 1,
                    actual_version: 1,
                    text: Arc::from("open\n"),
                }),
                &allocator,
            )
            .expect_err("unmappable open-buffer URI should be rejected");

        assert!(matches!(
            error,
            SourceLoadError::UnmappedOpenBufferUri { uri: error_uri } if error_uri == uri
        ));
    }
    assert!(allocator.source_snapshots().is_empty());
}

#[test]
fn open_buffer_loader_reports_package_source_path_errors_with_disk_categories() {
    let package = PackageFixture::new();
    package.write("other/basic.miz", "outside src\n");
    package.write("src/groups/basic.txt", "not miz\n");
    package.write("src/bad-name.miz", "bad namespace\n");
    let loader = DiskSourceLoader::new(package.root());
    let allocator = RecordingAllocator::new();

    let missing_source_root = loader
        .load(
            snapshot_id(32),
            source_input(SourceOriginInput::OpenBuffer {
                uri: package.file_uri("other/basic.miz"),
                expected_version: 1,
                actual_version: 1,
                text: Arc::from("open\n"),
            }),
            &allocator,
        )
        .expect_err("package file outside src should be rejected");

    assert!(matches!(
        &missing_source_root,
        SourceLoadError::InvalidSourcePath {
            error: SourcePathError::MissingSourceRoot { path }
        } if path.ends_with("other/basic.miz")
    ));
    assert_missing_source_root_error(&missing_source_root, "other/basic.miz");

    let unsupported_extension = loader
        .load(
            snapshot_id(32),
            source_input(SourceOriginInput::OpenBuffer {
                uri: package.file_uri("src/groups/basic.txt"),
                expected_version: 1,
                actual_version: 1,
                text: Arc::from("open\n"),
            }),
            &allocator,
        )
        .expect_err("non-miz package source URI should be rejected");

    assert!(matches!(
        unsupported_extension,
        SourceLoadError::UnsupportedFileExtension { path }
            if path.ends_with("src/groups/basic.txt")
    ));

    let invalid_namespace = loader
        .load(
            snapshot_id(32),
            source_input(SourceOriginInput::OpenBuffer {
                uri: package.file_uri("src/bad-name.miz"),
                expected_version: 1,
                actual_version: 1,
                text: Arc::from("open\n"),
            }),
            &allocator,
        )
        .expect_err("invalid namespace package source URI should be rejected");

    assert!(matches!(
        invalid_namespace,
        SourceLoadError::InvalidSourcePath {
            error: SourcePathError::InvalidNamespaceComponent { component }
        } if component == "bad-name"
    ));
    assert!(allocator.source_snapshots().is_empty());
}

#[test]
fn open_buffer_loader_keeps_file_uri_outside_package_root_unmapped() {
    let package = PackageFixture::new();
    package.write_outside("outside.miz", "outside package\n");
    let loader = DiskSourceLoader::new(package.root());
    let allocator = RecordingAllocator::new();
    let outside_uri = format!("file://{}", package.outside_path("outside.miz").display());

    let error = loader
        .load(
            snapshot_id(32),
            source_input(SourceOriginInput::OpenBuffer {
                uri: outside_uri.clone(),
                expected_version: 1,
                actual_version: 1,
                text: Arc::from("open\n"),
            }),
            &allocator,
        )
        .expect_err("file URI outside package root should be rejected");

    assert!(matches!(
        error,
        SourceLoadError::UnmappedOpenBufferUri { uri } if uri == outside_uri
    ));
    assert!(allocator.source_snapshots().is_empty());
}

#[test]
fn generated_source_loader_preserves_generator_metadata_and_anchor() {
    let package = PackageFixture::new();
    let loader = DiskSourceLoader::new(package.root());
    let ids = InMemorySessionIdAllocator::new();
    let snapshot = snapshot_id(33);
    let anchor_source = ids.next_source_id(snapshot).unwrap();
    let anchor = SourceAnchor::Range(SourceRange {
        source_id: anchor_source,
        start: 2,
        end: 9,
    });

    let loaded = loader
        .load(
            snapshot,
            source_input(SourceOriginInput::Generated {
                generator: GeneratedSourceKind::new("macro-expansion"),
                text: Arc::from("generated\r\ntext"),
                anchor: Some(anchor.clone()),
            }),
            &ids,
        )
        .unwrap();

    assert_eq!(loaded.text.as_ref(), "generated\r\ntext");
    assert_eq!(loaded.source_hash, hash_text("generated\r\ntext"));
    assert_eq!(loaded.loading_map, None);
    assert_eq!(loaded.generated_anchor, Some(anchor));
    assert!(matches!(
        loaded.origin,
        SourceOrigin::Generated { ref generator } if generator.as_str() == "macro-expansion"
    ));
}

#[test]
fn generated_source_loader_preserves_bom_and_crlf_without_loading_map() {
    let package = PackageFixture::new();
    let loader = DiskSourceLoader::new(package.root());
    let text = "\u{feff}generated\r\ntext";

    let loaded = loader
        .load(
            snapshot_id(34),
            source_input(SourceOriginInput::Generated {
                generator: GeneratedSourceKind::new("macro-expansion"),
                text: Arc::from(text),
                anchor: None,
            }),
            &InMemorySessionIdAllocator::new(),
        )
        .unwrap();

    assert_eq!(loaded.text.as_ref(), text);
    assert_eq!(loaded.source_hash, hash_text(text));
    assert_ne!(loaded.source_hash, hash_text("generated\ntext"));
    assert_eq!(loaded.line_map.source(), text);
    assert_eq!(loaded.line_map.text_hash(), loaded.source_hash);
    assert_eq!(
        loaded.line_map.line_starts(),
        &[0, "\u{feff}generated\r\n".len()]
    );
    assert_eq!(loaded.loading_map, None);
    assert_eq!(loaded.generated_anchor, None);
}

#[test]
fn generated_source_loader_accepts_missing_anchor_with_generator_metadata() {
    let package = PackageFixture::new();
    let loader = DiskSourceLoader::new(package.root());

    let loaded = loader
        .load(
            snapshot_id(34),
            source_input(SourceOriginInput::Generated {
                generator: GeneratedSourceKind::new("doc-extract"),
                text: Arc::from("generated\n"),
                anchor: None,
            }),
            &InMemorySessionIdAllocator::new(),
        )
        .unwrap();

    assert_eq!(loaded.text.as_ref(), "generated\n");
    assert_eq!(loaded.generated_anchor, None);
    assert!(matches!(
        loaded.origin,
        SourceOrigin::Generated { ref generator } if generator.as_str() == "doc-extract"
    ));
}

#[test]
fn generated_source_loader_rejects_blank_generator_metadata_before_source_id_allocation() {
    let package = PackageFixture::new();
    let loader = DiskSourceLoader::new(package.root());
    let allocator = RecordingAllocator::new();

    let error = loader
        .load(
            snapshot_id(35),
            source_input(SourceOriginInput::Generated {
                generator: GeneratedSourceKind::new("  "),
                text: Arc::from("generated\n"),
                anchor: None,
            }),
            &allocator,
        )
        .expect_err("generated source without metadata should be rejected");

    assert!(matches!(
        error,
        SourceLoadError::GeneratedSourceWithoutMetadata { module_path }
            if module_path.as_str() == "groups.basic"
    ));
    assert!(allocator.source_snapshots().is_empty());
}

#[test]
fn source_load_error_variants_and_error_sources_are_available() {
    let allocation = SourceLoadError::SourceIdAllocation {
        error: IdError::AllocatorOverflow,
    };
    let invalid_path = SourceLoadError::InvalidSourcePath {
        error: SourcePathError::InvalidNamespaceComponent {
            component: "bad-name".to_owned(),
        },
    };
    let duplicate = SourceLoadError::DuplicateModulePath {
        package_id: PackageId::new("mml"),
        module_path: ModulePath::new("groups.basic"),
    };
    let stale = SourceLoadError::StaleLspDocumentVersion {
        expected: 12,
        actual: 11,
    };
    let unmapped = SourceLoadError::UnmappedOpenBufferUri {
        uri: "untitled:basic".to_owned(),
    };
    let missing_generator = SourceLoadError::GeneratedSourceWithoutMetadata {
        module_path: ModulePath::new("generated.basic"),
    };
    let invalid_utf8 = SourceLoadError::InvalidUtf8 { path: None };
    let unreadable = SourceLoadError::UnreadableSourceFile {
        path: PathBuf::from("src/missing.miz"),
        kind: io::ErrorKind::NotFound,
    };
    let unsupported_origin = SourceLoadError::UnsupportedSourceOrigin {
        origin: SourceOriginKind::Generated,
    };

    assert!(std::error::Error::source(&allocation).is_some());
    assert!(std::error::Error::source(&invalid_path).is_some());
    assert!(matches!(
        duplicate,
        SourceLoadError::DuplicateModulePath {
            package_id,
            module_path,
        } if package_id.as_str() == "mml" && module_path.as_str() == "groups.basic"
    ));
    assert!(matches!(
        stale,
        SourceLoadError::StaleLspDocumentVersion {
            expected: 12,
            actual: 11
        }
    ));
    assert!(matches!(
        unmapped,
        SourceLoadError::UnmappedOpenBufferUri { uri } if uri == "untitled:basic"
    ));
    assert!(matches!(
        missing_generator,
        SourceLoadError::GeneratedSourceWithoutMetadata { module_path }
            if module_path.as_str() == "generated.basic"
    ));
    assert!(matches!(
        invalid_utf8,
        SourceLoadError::InvalidUtf8 { path: None }
    ));
    assert!(matches!(
        unreadable,
        SourceLoadError::UnreadableSourceFile {
            kind: io::ErrorKind::NotFound,
            ..
        }
    ));
    assert!(matches!(
        unsupported_origin,
        SourceLoadError::UnsupportedSourceOrigin {
            origin: SourceOriginKind::Generated
        }
    ));
}

#[test]
fn source_path_normalization_removes_dot_components() {
    let package = PackageFixture::new();
    package.write("src/groups/basic.miz", "");

    let normalized = normalize_source_path(
        package.root(),
        &package.root().join("./src/./groups/../groups/basic.miz"),
    );

    assert_eq!(normalized, Ok(path("src/groups/basic.miz")));
}

#[test]
fn source_path_normalization_rejects_package_root_escape_attempts() {
    let package = PackageFixture::new();
    package.write("src/main.miz", "");
    package.write_outside("outside.miz", "");

    let normalized = normalize_source_path(package.root(), Path::new("../outside.miz"));

    assert!(matches!(
        normalized,
        Err(SourcePathError::OutsidePackageRoot { .. })
    ));
}

#[test]
fn source_path_normalization_rejects_sources_outside_src() {
    let package = PackageFixture::new();
    package.write("other/main.miz", "");

    let normalized = normalize_source_path(package.root(), Path::new("other/main.miz"));

    assert!(matches!(
        normalized,
        Err(SourcePathError::MissingSourceRoot { .. })
    ));
}

#[test]
fn source_path_normalization_rejects_non_miz_files() {
    let package = PackageFixture::new();
    package.write("src/main.txt", "");

    let normalized = normalize_source_path(package.root(), Path::new("src/main.txt"));

    assert!(matches!(
        normalized,
        Err(SourcePathError::UnsupportedExtension { .. })
    ));
}

#[test]
fn source_path_normalization_uses_canonical_case_spelling() {
    let package = PackageFixture::new();
    package.write("src/MixedCase.miz", "");

    let normalized = normalize_source_path(package.root(), Path::new("src/MixedCase.miz"));

    assert_eq!(normalized, Ok(path("src/MixedCase.miz")));
}

#[test]
fn source_path_normalization_rejects_non_canonical_case_variants() {
    let rejected = reject_non_canonical_alias(
        Path::new("src/mixedcase.miz"),
        Path::new("src/MixedCase.miz"),
    );

    assert!(matches!(
        rejected,
        Err(SourcePathError::NonCanonicalPathSpelling { .. })
    ));
}

#[test]
fn source_path_normalization_rejects_symlink_spelling_aliases() {
    let rejected =
        reject_non_canonical_alias(Path::new("src/alias.miz"), Path::new("src/real.miz"));

    assert!(matches!(
        rejected,
        Err(SourcePathError::NonCanonicalPathAlias { .. })
    ));
}

#[test]
fn source_path_normalization_rejects_invalid_namespace_components() {
    for source_path in [
        "src/bad-name.miz",
        "src/groups/bad-name/basic.miz",
        "src/foo.miz/basic.miz",
    ] {
        let package = PackageFixture::new();
        package.write(source_path, "");

        let normalized = normalize_source_path(package.root(), Path::new(source_path));

        assert!(
            matches!(
                normalized,
                Err(SourcePathError::InvalidNamespaceComponent { .. })
            ),
            "{source_path:?}"
        );
    }
}

#[test]
fn source_path_normalization_rejects_reserved_namespace_components() {
    for source_path in ["src/theorem.miz", "src/groups/theorem/basic.miz"] {
        let package = PackageFixture::new();
        package.write(source_path, "");

        let normalized = normalize_source_path(package.root(), Path::new(source_path));

        assert!(
            matches!(
                normalized,
                Err(SourcePathError::InvalidNamespaceComponent { component })
                    if component == "theorem"
            ),
            "{source_path:?}"
        );
    }
}

#[test]
fn source_path_normalization_rejects_non_ascii_namespace_components() {
    let package = PackageFixture::new();
    package.write("src/naive_é.miz", "");

    let normalized = normalize_source_path(package.root(), Path::new("src/naive_é.miz"));

    assert!(matches!(
        normalized,
        Err(SourcePathError::InvalidNamespaceComponent { .. })
    ));
}

#[test]
fn source_path_normalization_accepts_platform_specific_separators() {
    let package = PackageFixture::new();
    package.write("src/groups/basic.miz", "");

    let normalized = normalize_source_path(package.root(), Path::new("src\\groups\\basic.miz"));

    assert_eq!(normalized, Ok(path("src/groups/basic.miz")));
}

#[cfg(unix)]
#[test]
fn source_path_normalization_rejects_symlink_aliases_inside_package() {
    use std::os::unix::fs::symlink;

    let package = PackageFixture::new();
    package.write("src/real.miz", "");
    symlink(
        package.root().join("src/real.miz"),
        package.root().join("src/alias.miz"),
    )
    .expect("symlink should be created");

    let normalized = normalize_source_path(package.root(), Path::new("src/alias.miz"));

    assert!(matches!(
        normalized,
        Err(SourcePathError::NonCanonicalPathAlias { .. })
    ));
}

#[cfg(unix)]
#[test]
fn source_path_normalization_rejects_symlink_escapes() {
    use std::os::unix::fs::symlink;

    let package = PackageFixture::new();
    package.write_outside("outside.miz", "");
    symlink(
        package.outside_path("outside.miz"),
        package.root().join("src/escape.miz"),
    )
    .expect("symlink should be created");

    let normalized = normalize_source_path(package.root(), Path::new("src/escape.miz"));

    assert!(matches!(
        normalized,
        Err(SourcePathError::OutsidePackageRoot { .. })
    ));
}

#[cfg(unix)]
#[test]
fn source_path_normalization_rejects_absolute_symlink_aliases_inside_package() {
    use std::os::unix::fs::symlink;

    let package = PackageFixture::new();
    package.write("src/real.miz", "");
    symlink(
        package.root().join("src/real.miz"),
        package.root().join("src/alias.miz"),
    )
    .expect("symlink should be created");

    let normalized = normalize_source_path(package.root(), &package.root().join("src/alias.miz"));

    assert!(matches!(
        normalized,
        Err(SourcePathError::NonCanonicalPathAlias { .. })
    ));
}

fn path(path: &str) -> NormalizedPath {
    NormalizedPath(path.to_owned())
}

fn source_input(origin: SourceOriginInput) -> SourceInput {
    SourceInput {
        package_id: PackageId::new("mml"),
        module_path: ModulePath::new("groups.basic"),
        normalized_path: path("src/groups/basic.miz"),
        edition: Edition::new("2026"),
        origin,
    }
}

fn disk_source_input(relative_path: &str) -> SourceInput {
    let mut input = source_input(SourceOriginInput::Disk {
        path: PathBuf::from(relative_path),
    });
    input.normalized_path = path(relative_path);
    input
}

fn assert_missing_source_root_error(error: &SourceLoadError, expected_suffix: &str) {
    assert!(
        matches!(
            error,
            SourceLoadError::InvalidSourcePath {
                error: SourcePathError::MissingSourceRoot { path }
            } if path.ends_with(expected_suffix)
        ),
        "missing source root should stay observable through InvalidSourcePath: {error:?}"
    );

    let message = error.to_string();
    assert!(
        message.contains("must be under the package `src` root"),
        "missing-source-root message should mention the required src root: {message}"
    );
    assert!(
        !message.contains("must stay inside package root"),
        "missing-source-root message should not look like an outside-package-root error: {message}"
    );
}

fn loaded_source(source_id: SourceId, origin: SourceOrigin, text: &str) -> LoadedSource {
    LoadedSource {
        source_id,
        package_id: PackageId::new("mml"),
        module_path: ModulePath::new("groups.basic"),
        normalized_path: path("src/groups/basic.miz"),
        text: Arc::from(text),
        source_hash: hash_text(text),
        edition: Edition::new("2026"),
        origin,
        line_map: LineMap::new(source_id, text),
        loading_map: None,
        generated_anchor: None,
    }
}

fn snapshot_id(byte: u8) -> BuildSnapshotId {
    let hex = format!("{byte:02x}").repeat(32);
    BuildSnapshotId::from_published_schema_str(&format!("mizar-session-build-snapshot-v1:{hex}"))
        .unwrap()
}

#[derive(Debug, Default)]
struct TextOnlyLoader {
    disk_text: Option<Arc<str>>,
}

impl TextOnlyLoader {
    fn with_disk_text(text: &str) -> Self {
        Self {
            disk_text: Some(Arc::from(text)),
        }
    }
}

impl SourceLoader for TextOnlyLoader {
    fn load(
        &self,
        snapshot: BuildSnapshotId,
        input: SourceInput,
        ids: &dyn SessionIdAllocator,
    ) -> Result<LoadedSource, SourceLoadError> {
        let source_id = ids
            .next_source_id(snapshot)
            .map_err(|error| SourceLoadError::SourceIdAllocation { error })?;
        let SourceInput {
            package_id,
            module_path,
            normalized_path,
            edition,
            origin,
        } = input;
        let (text, origin, generated_anchor) = match origin {
            SourceOriginInput::Disk { path } => (
                self.disk_text.clone().ok_or({
                    SourceLoadError::UnreadableSourceFile {
                        path,
                        kind: io::ErrorKind::Unsupported,
                    }
                })?,
                SourceOrigin::Disk,
                None,
            ),
            SourceOriginInput::OpenBuffer {
                actual_version,
                text,
                ..
            } => (
                text,
                SourceOrigin::OpenBuffer {
                    version: actual_version,
                },
                None,
            ),
            SourceOriginInput::Generated {
                generator,
                text,
                anchor,
            } => (text, SourceOrigin::Generated { generator }, anchor),
        };
        let source_hash = self.hash_text(&text);
        let line_map = LineMap::new(source_id, &text);

        Ok(LoadedSource {
            source_id,
            package_id,
            module_path,
            normalized_path,
            text,
            source_hash,
            edition,
            origin,
            line_map,
            loading_map: None,
            generated_anchor,
        })
    }
}

#[derive(Debug)]
struct RecordingAllocator {
    inner: InMemorySessionIdAllocator,
    source_snapshots: Mutex<Vec<BuildSnapshotId>>,
}

impl RecordingAllocator {
    fn new() -> Self {
        Self {
            inner: InMemorySessionIdAllocator::new(),
            source_snapshots: Mutex::new(Vec::new()),
        }
    }

    fn source_snapshots(&self) -> Vec<BuildSnapshotId> {
        self.source_snapshots
            .lock()
            .expect("recording allocator mutex poisoned")
            .clone()
    }
}

impl SessionIdAllocator for RecordingAllocator {
    fn next_session_id(&self) -> Result<BuildSessionId, IdError> {
        self.inner.next_session_id()
    }

    fn next_request_id(&self) -> Result<BuildRequestId, IdError> {
        self.inner.next_request_id()
    }

    fn next_source_id(&self, snapshot: BuildSnapshotId) -> Result<SourceId, IdError> {
        self.source_snapshots
            .lock()
            .expect("recording allocator mutex poisoned")
            .push(snapshot);
        self.inner.next_source_id(snapshot)
    }

    fn next_source_map_id(&self, snapshot: BuildSnapshotId) -> Result<SourceMapId, IdError> {
        self.inner.next_source_map_id(snapshot)
    }

    fn next_lease_id(&self, snapshot: BuildSnapshotId) -> Result<SnapshotLeaseId, IdError> {
        self.inner.next_lease_id(snapshot)
    }
}

struct PackageFixture {
    base: PathBuf,
    root: PathBuf,
}

impl PackageFixture {
    fn new() -> Self {
        let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "mizar_session_source_path_{}_{}",
            std::process::id(),
            id
        ));
        let package_root = root.join("package");
        fs::create_dir_all(package_root.join("src"))
            .expect("package src directory should be created");
        Self {
            base: root,
            root: package_root,
        }
    }

    fn root(&self) -> &Path {
        &self.root
    }

    fn file_uri(&self, relative: &str) -> String {
        format!("file://{}", self.root.join(relative).display())
    }

    fn write(&self, relative: &str, content: &str) {
        let path = self.root.join(relative);
        self.write_path(&path, content);
    }

    fn write_bytes(&self, relative: &str, content: &[u8]) {
        let path = self.root.join(relative);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("parent directory should be created");
        }
        fs::write(path, content).expect("fixture bytes should be written");
    }

    fn write_outside(&self, relative: &str, content: &str) {
        let path = self.outside_path(relative);
        self.write_path(&path, content);
    }

    fn outside_path(&self, relative: &str) -> PathBuf {
        self.base.join(relative)
    }

    fn write_path(&self, path: &Path, content: &str) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("parent directory should be created");
        }
        fs::write(path, content).expect("fixture file should be written");
    }
}

impl Drop for PackageFixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.base);
    }
}
