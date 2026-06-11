//! Source loading bridge for frontend phase 1.
//!
//! Canonical behavior is specified in the
//! [source design spec](../../../../doc/design/mizar-frontend/en/source.md).

use crate::span_bridge::{SpanBridge, SpanBridgeError};
use mizar_session::{
    BuildSnapshotId, Edition, Hash, LineMap, LoadedSource, LoadingMap, ModulePath, NormalizedPath,
    PackageId, SessionIdAllocator, SourceAnchor, SourceId, SourceInput, SourceLoadError,
    SourceLoader, SourceOrigin, SourceOriginInput,
};
use std::path::PathBuf;
use std::sync::Arc;

/// Loaded source identity and maps projected from `mizar-session`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceUnit {
    /// Session-owned source identity for this loaded source.
    pub source_id: SourceId,
    /// Package identity that owns the source.
    pub package_id: PackageId,
    /// Logical module path derived during source loading.
    pub module_path: ModulePath,
    /// Normalized source path used for stable identity.
    pub normalized_path: NormalizedPath,
    /// Language edition selected for this source.
    pub edition: Edition,
    /// Diagnostic file path associated with this source.
    pub file_path: PathBuf,
    /// Loaded source text after session-level normalization.
    pub source_text: Arc<str>,
    /// Session-owned hash of the loaded source text.
    pub source_hash: Hash,
    /// Line map for loaded-text coordinates.
    pub line_map: LineMap,
    /// Optional map from loaded text back to the original input.
    pub loading_map: Option<LoadingMap>,
    /// Source origin preserved from the session loader.
    pub origin: SourceOrigin,
    /// Anchor for generated sources, when one exists.
    pub generated_anchor: Option<SourceAnchor>,
}

/// Request to load one source unit in a build snapshot.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceUnitRequest {
    /// Snapshot that owns the requested source load.
    pub snapshot: BuildSnapshotId,
    /// Session source input to load.
    pub input: SourceInput,
}

/// Loads session sources and projects them into frontend `SourceUnit`s.
pub trait SourceUnitLoader {
    /// Loads one source unit using the provided session id allocator.
    fn load_source_unit(
        &self,
        request: SourceUnitRequest,
        ids: &dyn SessionIdAllocator,
    ) -> Result<SourceUnit, SourceLoadError>;
}

/// Adapter from a `mizar-session` source loader to the frontend loader trait.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FrontendSourceLoader<L: SourceLoader> {
    loader: L,
}

impl<L: SourceLoader> FrontendSourceLoader<L> {
    /// Creates a frontend source loader around a session loader.
    pub fn new(loader: L) -> Self {
        Self { loader }
    }
}

impl<L: SourceLoader> SourceUnitLoader for FrontendSourceLoader<L> {
    fn load_source_unit(
        &self,
        request: SourceUnitRequest,
        ids: &dyn SessionIdAllocator,
    ) -> Result<SourceUnit, SourceLoadError> {
        let file_path = diagnostic_file_path(&request.input);
        self.loader
            .load(request.snapshot, request.input, ids)
            .map(|loaded| source_unit_from_loaded(loaded, file_path))
    }
}

/// Projects a `LoadedSource` into a `SourceUnit` without recomputing identity or maps.
pub fn source_unit_from_loaded(loaded: LoadedSource, file_path: PathBuf) -> SourceUnit {
    let LoadedSource {
        source_id,
        package_id,
        module_path,
        normalized_path,
        text,
        source_hash,
        edition,
        origin,
        line_map,
        loading_map,
        generated_anchor,
    } = loaded;

    SourceUnit {
        source_id,
        package_id,
        module_path,
        normalized_path,
        edition,
        file_path,
        source_text: text,
        source_hash,
        line_map,
        loading_map,
        origin,
        generated_anchor,
    }
}

/// Registers a source unit's line and loading maps in the span bridge.
pub fn register_source_unit(
    bridge: &mut SpanBridge,
    source: &SourceUnit,
) -> Result<(), SpanBridgeError> {
    bridge.register_source(
        source.source_id,
        source.line_map.clone(),
        source.loading_map.clone(),
    )
}

fn diagnostic_file_path(input: &SourceInput) -> PathBuf {
    match &input.origin {
        SourceOriginInput::Disk { path } => path.clone(),
        SourceOriginInput::OpenBuffer { uri, .. } => file_path_from_document_uri(uri)
            .unwrap_or_else(|| PathBuf::from(input.normalized_path.as_str())),
        SourceOriginInput::Generated { .. } => PathBuf::from(input.normalized_path.as_str()),
        _ => PathBuf::from(input.normalized_path.as_str()),
    }
}

fn file_path_from_document_uri(uri: &str) -> Option<PathBuf> {
    let rest = uri.strip_prefix("file://")?;
    let (authority, path) = if rest.starts_with('/') {
        ("", rest)
    } else {
        let slash = rest.find('/')?;
        rest.split_at(slash)
    };
    if authority.contains(['?', '#']) || path.contains(['?', '#']) {
        return None;
    }
    platform_file_path_from_uri_parts(authority, &percent_decode_uri_path(path)?)
}

#[cfg(windows)]
fn platform_file_path_from_uri_parts(authority: &str, path: &str) -> Option<PathBuf> {
    if authority.is_empty() || authority.eq_ignore_ascii_case("localhost") {
        return Some(PathBuf::from(windows_drive_path_from_uri_path(path)?));
    }

    let path = path.strip_prefix('/')?;
    if path.is_empty() {
        return None;
    }
    let mut unc_path = String::from(r"\\");
    unc_path.push_str(authority);
    unc_path.push('\\');
    unc_path.push_str(&path.replace('/', "\\"));
    Some(PathBuf::from(unc_path))
}

#[cfg(windows)]
fn windows_drive_path_from_uri_path(path: &str) -> Option<String> {
    let path = path.strip_prefix('/')?;
    let bytes = path.as_bytes();
    if bytes.len() < 2 || bytes[1] != b':' || !bytes[0].is_ascii_alphabetic() {
        return None;
    }
    Some(path.replace('/', "\\"))
}

#[cfg(not(windows))]
fn platform_file_path_from_uri_parts(authority: &str, path: &str) -> Option<PathBuf> {
    if !authority.is_empty() && !authority.eq_ignore_ascii_case("localhost") {
        return None;
    }
    Some(PathBuf::from(path))
}

fn percent_decode_uri_path(path: &str) -> Option<String> {
    let mut decoded = Vec::with_capacity(path.len());
    let mut bytes = path.bytes();
    while let Some(byte) = bytes.next() {
        if byte == b'%' {
            let high = bytes.next()?;
            let low = bytes.next()?;
            decoded.push(hex_value(high)? << 4 | hex_value(low)?);
        } else {
            decoded.push(byte);
        }
    }
    String::from_utf8(decoded).ok()
}

fn hex_value(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        FrontendSourceLoader, SourceUnitLoader, SourceUnitRequest, register_source_unit,
        source_unit_from_loaded,
    };
    use crate::span_bridge::{LexerByteSpan, SpanBridge, SpanBridgeError};
    use mizar_session::{
        BuildSnapshotId, DiskSourceLoader, Edition, GeneratedSourceKind, Hash,
        InMemorySessionIdAllocator, LineMap, LoadedSource, LoadingMap, LoadingMapSegment,
        LoadingOrigin, MappedSourceRange, MappedSourceRangeKind, ModulePath, PackageId,
        SessionIdAllocator, SourceAnchor, SourceInput, SourceLoadError, SourceLoader, SourceOrigin,
        SourceOriginInput, SourceRange, TextRange, hash_text, normalize_path,
    };
    use std::fs;
    use std::io;
    use std::path::{Path, PathBuf};
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static NEXT_FIXTURE_ID: AtomicUsize = AtomicUsize::new(0);

    #[test]
    fn disk_loaded_source_projects_without_recomputing_identity_or_maps() {
        let package = PackageFixture::new();
        package.write("src/groups/basic.miz", "environ\nbegin\n");
        let loader = DiskSourceLoader::new(package.root());
        let loaded = loader
            .load(
                snapshot_id(1),
                disk_source_input(package.root(), package.path("src/groups/basic.miz")),
                &InMemorySessionIdAllocator::new(),
            )
            .unwrap();
        let file_path = package.path("src/groups/basic.miz");

        let source = source_unit_from_loaded(loaded.clone(), file_path.clone());

        assert_eq!(source.source_id, loaded.source_id);
        assert_eq!(source.package_id, loaded.package_id);
        assert_eq!(source.module_path, loaded.module_path);
        assert_eq!(source.normalized_path, loaded.normalized_path);
        assert_eq!(source.edition, loaded.edition);
        assert_eq!(source.file_path, file_path);
        assert_eq!(source.source_text, loaded.text);
        assert_eq!(source.source_hash, loaded.source_hash);
        assert_eq!(source.line_map, loaded.line_map);
        assert_eq!(source.loading_map, loaded.loading_map);
        assert_eq!(source.origin, loaded.origin);
        assert_eq!(source.generated_anchor, loaded.generated_anchor);
    }

    #[test]
    fn source_unit_from_loaded_preserves_sentinel_session_fields_without_recomputing() {
        let package = PackageFixture::new();
        package.write("src/groups/sentinel.miz", "");
        let snapshot = snapshot_id(8);
        let source_id = InMemorySessionIdAllocator::new()
            .next_source_id(snapshot)
            .unwrap();
        let anchor = SourceAnchor::Range(SourceRange {
            source_id,
            start: 1,
            end: 4,
        });
        let sentinel_hash = Hash::from_bytes([0xa5; Hash::BYTE_LEN]);
        let sentinel_line_map = LineMap::new(source_id, "line-map-sentinel\n");
        let sentinel_loading_map =
            LoadingMap::identity(source_id, "loading-map-sentinel", LoadingOrigin::Generated);
        let loaded_text: Arc<str> = Arc::from("loaded text\n");
        let loaded = LoadedSource {
            source_id,
            package_id: PackageId::new("mml"),
            module_path: ModulePath::new("groups.sentinel"),
            normalized_path: normalize_path(
                package.root(),
                &package.path("src/groups/sentinel.miz"),
            )
            .unwrap(),
            text: loaded_text.clone(),
            source_hash: sentinel_hash,
            edition: Edition::new("sentinel-edition"),
            origin: SourceOrigin::Generated {
                generator: GeneratedSourceKind::new("sentinel-generator"),
            },
            line_map: sentinel_line_map.clone(),
            loading_map: Some(sentinel_loading_map.clone()),
            generated_anchor: Some(anchor.clone()),
        };

        let source = source_unit_from_loaded(loaded, PathBuf::from("sentinel-display.miz"));

        assert_eq!(source.source_text, loaded_text);
        assert_eq!(source.source_hash, sentinel_hash);
        assert_ne!(source.source_hash, hash_text(source.source_text.as_ref()));
        assert_eq!(source.line_map, sentinel_line_map);
        assert_ne!(source.line_map.source(), source.source_text.as_ref());
        assert_eq!(source.loading_map, Some(sentinel_loading_map));
        assert_eq!(source.edition, Edition::new("sentinel-edition"));
        assert_eq!(source.generated_anchor, Some(anchor));
    }

    #[test]
    fn frontend_loader_forwards_request_and_allocator_to_wrapped_loader() {
        let package = PackageFixture::new();
        package.write("src/groups/basic.miz", "disk\n");
        let snapshot = snapshot_id(9);
        let input = disk_source_input(package.root(), package.path("src/groups/basic.miz"));
        let loader = FrontendSourceLoader::new(SpyLoader {
            expected_snapshot: snapshot,
            expected_input: input.clone(),
        });

        let source = loader
            .load_source_unit(
                SourceUnitRequest {
                    snapshot,
                    input: input.clone(),
                },
                &InMemorySessionIdAllocator::new(),
            )
            .unwrap();

        assert_eq!(source.package_id, input.package_id);
        assert_eq!(source.module_path, input.module_path);
        assert_eq!(source.normalized_path, input.normalized_path);
        assert_eq!(source.source_text.as_ref(), "spy text\n");
        assert_eq!(source.source_hash, hash_text("spy text\n"));
        assert_eq!(source.file_path, package.path("src/groups/basic.miz"));
    }

    #[test]
    fn frontend_loader_preserves_bom_and_crlf_loading_map() {
        let package = PackageFixture::new();
        package.write_bytes("src/groups/bom_crlf.miz", b"\xef\xbb\xbfalpha\r\nbeta");
        let loader = FrontendSourceLoader::new(DiskSourceLoader::new(package.root()));
        let path = package.path("src/groups/bom_crlf.miz");

        let source = loader
            .load_source_unit(
                SourceUnitRequest {
                    snapshot: snapshot_id(2),
                    input: disk_source_input(package.root(), path.clone()),
                },
                &InMemorySessionIdAllocator::new(),
            )
            .unwrap();

        assert_eq!(source.file_path, path);
        assert_eq!(source.source_text.as_ref(), "alpha\nbeta");
        let loading_map = source
            .loading_map
            .as_ref()
            .expect("BOM/CRLF normalization should keep the session loading map");
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
    }

    #[test]
    fn frontend_loader_preserves_identity_load_without_loading_map() {
        let package = PackageFixture::new();
        package.write("src/groups/basic.miz", "alpha\n");
        let loader = FrontendSourceLoader::new(DiskSourceLoader::new(package.root()));

        let source = loader
            .load_source_unit(
                SourceUnitRequest {
                    snapshot: snapshot_id(3),
                    input: disk_source_input(package.root(), package.path("src/groups/basic.miz")),
                },
                &InMemorySessionIdAllocator::new(),
            )
            .unwrap();

        assert_eq!(source.source_text.as_ref(), "alpha\n");
        assert_eq!(source.loading_map, None);
        assert!(matches!(source.origin, SourceOrigin::Disk));
    }

    #[test]
    fn frontend_loader_preserves_open_buffer_origin_version_and_file_path() {
        let package = PackageFixture::new();
        package.write("src/groups/basic.miz", "disk\n");
        let loader = FrontendSourceLoader::new(DiskSourceLoader::new(package.root()));
        let uri = package.file_uri("src/groups/basic.miz");

        let source = loader
            .load_source_unit(
                SourceUnitRequest {
                    snapshot: snapshot_id(4),
                    input: source_input(
                        package.root(),
                        SourceOriginInput::OpenBuffer {
                            uri: uri.clone(),
                            expected_version: 7,
                            actual_version: 7,
                            text: Arc::from("\u{feff}open\r\n"),
                        },
                    ),
                },
                &InMemorySessionIdAllocator::new(),
            )
            .unwrap();

        assert_eq!(source.file_path, package.path("src/groups/basic.miz"));
        assert_eq!(source.source_text.as_ref(), "open\n");
        assert!(matches!(
            source.origin,
            SourceOrigin::OpenBuffer { version: 7 }
        ));
        assert_eq!(
            source.loading_map.as_ref().map(|map| &map.origin),
            Some(&LoadingOrigin::OpenBufferText { uri, version: 7 })
        );
    }

    #[test]
    fn frontend_loader_decodes_open_buffer_file_uri_path() {
        let package = PackageFixture::new();
        package.write("src/groups/basic.miz", "disk\n");
        let snapshot = snapshot_id(4);
        let encoded_path = package
            .path("src/groups/space name.miz")
            .display()
            .to_string()
            .replace("space name", "space%20name");
        let uri = format!("file://{encoded_path}");
        let input = source_input(
            package.root(),
            SourceOriginInput::OpenBuffer {
                uri,
                expected_version: 7,
                actual_version: 7,
                text: Arc::from("open\n"),
            },
        );
        let loader = FrontendSourceLoader::new(SpyLoader {
            expected_snapshot: snapshot,
            expected_input: input.clone(),
        });

        let source = loader
            .load_source_unit(
                SourceUnitRequest { snapshot, input },
                &InMemorySessionIdAllocator::new(),
            )
            .unwrap();

        assert_eq!(source.file_path, package.path("src/groups/space name.miz"));
    }

    #[test]
    fn frontend_loader_falls_back_to_normalized_path_for_invalid_open_buffer_file_uri() {
        let package = PackageFixture::new();
        package.write("src/groups/basic.miz", "disk\n");
        let snapshot = snapshot_id(4);
        let input = source_input(
            package.root(),
            SourceOriginInput::OpenBuffer {
                uri: "file:///not%zzvalid.miz".to_owned(),
                expected_version: 7,
                actual_version: 7,
                text: Arc::from("open\n"),
            },
        );
        let expected_path = PathBuf::from(input.normalized_path.as_str());
        let loader = FrontendSourceLoader::new(SpyLoader {
            expected_snapshot: snapshot,
            expected_input: input.clone(),
        });

        let source = loader
            .load_source_unit(
                SourceUnitRequest { snapshot, input },
                &InMemorySessionIdAllocator::new(),
            )
            .unwrap();

        assert_eq!(source.file_path, expected_path);
    }

    #[test]
    fn frontend_loader_preserves_generated_origin_and_anchor() {
        let package = PackageFixture::new();
        package.write("src/generated/basic.miz", "");
        let snapshot = snapshot_id(5);
        let ids = InMemorySessionIdAllocator::new();
        let anchor_source = ids.next_source_id(snapshot).unwrap();
        let anchor = SourceAnchor::Range(SourceRange {
            source_id: anchor_source,
            start: 2,
            end: 9,
        });
        let loader = FrontendSourceLoader::new(DiskSourceLoader::new(package.root()));

        let source = loader
            .load_source_unit(
                SourceUnitRequest {
                    snapshot,
                    input: source_input(
                        package.root(),
                        SourceOriginInput::Generated {
                            generator: GeneratedSourceKind::new("macro-expansion"),
                            text: Arc::from("generated\r\ntext"),
                            anchor: Some(anchor.clone()),
                        },
                    ),
                },
                &ids,
            )
            .unwrap();

        assert_eq!(source.file_path, PathBuf::from("src/generated/basic.miz"));
        assert_eq!(source.source_text.as_ref(), "generated\r\ntext");
        assert_eq!(source.loading_map, None);
        assert_eq!(source.generated_anchor, Some(anchor));
        assert!(matches!(
            source.origin,
            SourceOrigin::Generated { ref generator } if generator.as_str() == "macro-expansion"
        ));
    }

    #[test]
    fn register_source_unit_records_line_and_loading_maps_and_reports_conflicts() {
        let package = PackageFixture::new();
        package.write_bytes("src/groups/bom_crlf.miz", b"\xef\xbb\xbfalpha\r\nbeta");
        let loader = FrontendSourceLoader::new(DiskSourceLoader::new(package.root()));
        let mut source = loader
            .load_source_unit(
                SourceUnitRequest {
                    snapshot: snapshot_id(6),
                    input: disk_source_input(
                        package.root(),
                        package.path("src/groups/bom_crlf.miz"),
                    ),
                },
                &InMemorySessionIdAllocator::new(),
            )
            .unwrap();
        let mut bridge = SpanBridge::new();

        register_source_unit(&mut bridge, &source).unwrap();
        register_source_unit(&mut bridge, &source).unwrap();
        assert_eq!(
            bridge.loaded_mapping(source.source_id, LexerByteSpan { start: 5, end: 6 }),
            Ok(MappedSourceRange {
                primary: SourceRange {
                    source_id: source.source_id,
                    start: 5,
                    end: 6,
                },
                secondary: Vec::new(),
                original_input: Some(TextRange { start: 8, end: 10 }),
                kind: MappedSourceRangeKind::Degraded,
            })
        );

        source.line_map = LineMap::new(source.source_id, "different text");
        assert_eq!(
            register_source_unit(&mut bridge, &source),
            Err(SpanBridgeError::ConflictingSourceRegistration {
                source_id: source.source_id,
            })
        );
    }

    #[test]
    fn frontend_loader_propagates_session_load_errors_unchanged() {
        let package = PackageFixture::new();
        package.write("src/groups/basic.miz", "alpha\n");
        let expected = SourceLoadError::StaleLspDocumentVersion {
            expected: 2,
            actual: 1,
        };
        let loader = FrontendSourceLoader::new(ErrorLoader {
            error: expected.clone(),
        });

        let error = loader
            .load_source_unit(
                SourceUnitRequest {
                    snapshot: snapshot_id(7),
                    input: disk_source_input(package.root(), package.path("src/groups/basic.miz")),
                },
                &InMemorySessionIdAllocator::new(),
            )
            .expect_err("session loader error should be returned unchanged");

        assert_eq!(error, expected);
    }

    fn disk_source_input(package_root: &Path, path: PathBuf) -> SourceInput {
        let mut input = source_input(package_root, SourceOriginInput::Disk { path: path.clone() });
        input.normalized_path = normalize_path(package_root, &path).unwrap();
        input
    }

    fn source_input(package_root: &Path, origin: SourceOriginInput) -> SourceInput {
        let normalized_path = match &origin {
            SourceOriginInput::Disk { path } => normalize_path(package_root, path),
            SourceOriginInput::OpenBuffer { .. } => {
                normalize_path(package_root, &package_root.join("src/groups/basic.miz"))
            }
            SourceOriginInput::Generated { .. } => {
                normalize_path(package_root, &package_root.join("src/generated/basic.miz"))
            }
            _ => normalize_path(package_root, &package_root.join("src/groups/basic.miz")),
        }
        .unwrap();

        SourceInput {
            package_id: PackageId::new("mml"),
            module_path: ModulePath::new("groups.basic"),
            normalized_path,
            edition: Edition::new("2026"),
            origin,
        }
    }

    fn snapshot_id(byte: u8) -> BuildSnapshotId {
        let hex = format!("{byte:02x}").repeat(32);
        BuildSnapshotId::from_published_schema_str(&format!(
            "mizar-session-build-snapshot-v1:{hex}"
        ))
        .unwrap()
    }

    #[derive(Debug)]
    struct ErrorLoader {
        error: SourceLoadError,
    }

    impl SourceLoader for ErrorLoader {
        fn load(
            &self,
            _snapshot: BuildSnapshotId,
            _input: SourceInput,
            _ids: &dyn SessionIdAllocator,
        ) -> Result<mizar_session::LoadedSource, SourceLoadError> {
            Err(self.error.clone())
        }
    }

    #[derive(Debug)]
    struct SpyLoader {
        expected_snapshot: BuildSnapshotId,
        expected_input: SourceInput,
    }

    impl SourceLoader for SpyLoader {
        fn load(
            &self,
            snapshot: BuildSnapshotId,
            input: SourceInput,
            ids: &dyn SessionIdAllocator,
        ) -> Result<LoadedSource, SourceLoadError> {
            assert_eq!(snapshot, self.expected_snapshot);
            assert_eq!(input, self.expected_input);

            let source_id = ids
                .next_source_id(snapshot)
                .map_err(|error| SourceLoadError::SourceIdAllocation { error })?;
            let text: Arc<str> = Arc::from("spy text\n");
            Ok(LoadedSource {
                source_id,
                package_id: input.package_id,
                module_path: input.module_path,
                normalized_path: input.normalized_path,
                text: text.clone(),
                source_hash: self.hash_text(&text),
                edition: input.edition,
                origin: SourceOrigin::Disk,
                line_map: LineMap::new(source_id, &text),
                loading_map: None,
                generated_anchor: None,
            })
        }
    }

    struct PackageFixture {
        root: PathBuf,
    }

    impl PackageFixture {
        fn new() -> Self {
            let id = NEXT_FIXTURE_ID.fetch_add(1, Ordering::Relaxed);
            let root = std::env::temp_dir().join(format!(
                "mizar-frontend-source-test-{}-{id}",
                std::process::id()
            ));
            fs::create_dir_all(&root).unwrap();
            Self { root }
        }

        fn root(&self) -> &Path {
            &self.root
        }

        fn path(&self, relative: &str) -> PathBuf {
            self.root.join(relative)
        }

        fn write(&self, relative: &str, text: &str) {
            self.write_bytes(relative, text.as_bytes());
        }

        fn write_bytes(&self, relative: &str, bytes: &[u8]) {
            let path = self.path(relative);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, bytes).unwrap();
        }

        fn file_uri(&self, relative: &str) -> String {
            format!("file://{}", self.path(relative).display())
        }
    }

    impl Drop for PackageFixture {
        fn drop(&mut self) {
            match fs::remove_dir_all(&self.root) {
                Ok(()) => {}
                Err(error) if error.kind() == io::ErrorKind::NotFound => {}
                Err(error) => panic!(
                    "failed to remove temporary package `{}`: {error}",
                    self.root.display()
                ),
            }
        }
    }
}
