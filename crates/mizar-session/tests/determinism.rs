use mizar_session::{
    BuildSnapshot, BuildSnapshotId, CommentKind, DependencyArtifactRef, DiskSourceLoader, Edition,
    Hash, InMemorySessionIdAllocator, LineColumn, LineMap, LoadedSource, LoadingMap,
    LoadingMapSegment, LoadingOrigin, MappedSourceRange, MappedSourceRangeKind, ModulePath,
    PackageId, PreprocessMap, PreprocessSegment, RetainedSourceMapService, SessionIdAllocator,
    SnapshotInput, SnapshotRegistry, SourceAnchor, SourceId, SourceInput, SourceLoader,
    SourceMapService, SourceOriginInput, SourceRange, SourceVersion, TextRange, ToolchainInfo,
    WorkspaceRoot, normalize_path,
};
use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT_FIXTURE_ID: AtomicUsize = AtomicUsize::new(0);

#[test]
fn build_snapshot_id_is_deterministic_for_public_snapshot_construction_orders() {
    let package = PackageFixture::new();
    for source in SOURCE_FIXTURES {
        package.write(source.relative_path, source.text);
    }

    let baseline_snapshot = create_snapshot(snapshot_input_for_order(
        package.root(),
        load_source_versions(&package, [0, 1, 2]),
        dependency_artifacts([0, 1]),
    ));
    let baseline_id = baseline_snapshot.id;
    let baseline_serialized = baseline_id
        .to_published_schema_string()
        .expect("build snapshot ids serialize for published schemas");

    for source_order in SOURCE_ORDERS {
        for dependency_order in DEPENDENCY_ORDERS {
            let snapshot = create_snapshot(snapshot_input_for_order(
                package.root(),
                load_source_versions(&package, source_order),
                dependency_artifacts(dependency_order),
            ));

            assert_eq!(
                snapshot.id, baseline_id,
                "{source_order:?} {dependency_order:?}"
            );
            assert_eq!(
                snapshot
                    .id
                    .to_published_schema_string()
                    .expect("build snapshot ids serialize for published schemas"),
                baseline_serialized,
                "{source_order:?} {dependency_order:?}",
            );
            assert_eq!(
                source_identity_summary(&snapshot),
                vec![
                    (
                        "mml".to_owned(),
                        "alpha".to_owned(),
                        "src/alpha.miz".to_owned()
                    ),
                    (
                        "mml".to_owned(),
                        "beta".to_owned(),
                        "src/beta.miz".to_owned()
                    ),
                    (
                        "mml".to_owned(),
                        "gamma.delta".to_owned(),
                        "src/gamma/delta.miz".to_owned(),
                    ),
                ],
                "{source_order:?} {dependency_order:?}",
            );
        }
    }

    let reversed_snapshot = create_snapshot(snapshot_input_for_order(
        package.root(),
        load_source_versions(&package, [2, 1, 0]),
        dependency_artifacts([1, 0]),
    ));
    assert_eq!(reversed_snapshot.id, baseline_id);
    assert_ne!(
        source_id_for_module(&baseline_snapshot, "alpha"),
        source_id_for_module(&reversed_snapshot, "alpha"),
        "the test must exercise a different source-id allocation schedule",
    );
}

#[test]
fn retained_source_map_conversions_are_deterministic_for_equivalent_retained_maps() {
    let snapshot = snapshot_id(0x7a);
    let ids = InMemorySessionIdAllocator::new();
    let primary_source = ids
        .next_source_id(snapshot)
        .expect("test source id should allocate");
    let secondary_source = ids
        .next_source_id(snapshot)
        .expect("test source id should allocate");
    let primary_maps = retained_maps(primary_source);
    let secondary_maps = retained_maps(secondary_source);

    let service_a = service_with_maps_in_source_order(&primary_maps, &secondary_maps);
    let service_b = service_with_maps_in_interleaved_order(&primary_maps, &secondary_maps);

    let line_range = SourceRange {
        source_id: primary_source,
        start: 4,
        end: 7,
    };
    assert_eq!(
        service_a.line_column(line_range),
        service_b.line_column(line_range)
    );
    assert_eq!(
        service_a.line_column(line_range),
        Ok((
            LineColumn { line: 1, column: 5 },
            LineColumn { line: 2, column: 2 },
        )),
    );

    let loaded_range = TextRange { start: 4, end: 7 };
    assert_eq!(
        service_a.original_range_for_loaded(primary_source, loaded_range),
        service_b.original_range_for_loaded(primary_source, loaded_range),
    );
    assert_eq!(
        service_a.original_range_for_loaded(primary_source, loaded_range),
        Ok(MappedSourceRange {
            primary: SourceRange {
                source_id: primary_source,
                start: 4,
                end: 7,
            },
            secondary: Vec::new(),
            original_input: Some(TextRange { start: 4, end: 8 }),
            kind: MappedSourceRangeKind::Degraded,
        }),
    );

    let lexical_range = TextRange { start: 0, end: 11 };
    assert_eq!(
        service_a.source_range_for_lexical(primary_source, lexical_range),
        service_b.source_range_for_lexical(primary_source, lexical_range),
    );
    assert_eq!(
        service_a.source_range_for_lexical(primary_source, lexical_range),
        Ok(MappedSourceRange {
            primary: SourceRange {
                source_id: primary_source,
                start: 0,
                end: 25,
            },
            secondary: vec![
                SourceAnchor::Range(SourceRange {
                    source_id: primary_source,
                    start: 0,
                    end: 6,
                }),
                SourceAnchor::Range(SourceRange {
                    source_id: primary_source,
                    start: 6,
                    end: 20,
                }),
                SourceAnchor::Range(SourceRange {
                    source_id: primary_source,
                    start: 20,
                    end: 25,
                }),
            ],
            original_input: None,
            kind: MappedSourceRangeKind::Composite,
        }),
    );

    let boundary = TextRange { start: 6, end: 6 };
    assert_eq!(
        service_a.source_range_for_lexical(primary_source, boundary),
        service_b.source_range_for_lexical(primary_source, boundary),
    );
    assert_eq!(
        service_a.source_range_for_lexical(primary_source, boundary),
        Ok(MappedSourceRange {
            primary: SourceRange {
                source_id: primary_source,
                start: 6,
                end: 6,
            },
            secondary: vec![
                SourceAnchor::Range(SourceRange {
                    source_id: primary_source,
                    start: 6,
                    end: 20,
                }),
                SourceAnchor::Point {
                    source_id: primary_source,
                    offset: 20,
                },
            ],
            original_input: None,
            kind: MappedSourceRangeKind::Composite,
        }),
    );
}

const SOURCE_ORDERS: [[usize; 3]; 6] = [
    [0, 1, 2],
    [0, 2, 1],
    [1, 0, 2],
    [1, 2, 0],
    [2, 0, 1],
    [2, 1, 0],
];

const DEPENDENCY_ORDERS: [[usize; 2]; 2] = [[0, 1], [1, 0]];

const SOURCE_FIXTURES: [SourceFixture; 3] = [
    SourceFixture {
        module_path: "alpha",
        relative_path: "src/alpha.miz",
        text: "theorem Alpha;\n",
    },
    SourceFixture {
        module_path: "beta",
        relative_path: "src/beta.miz",
        text: "\u{feff}theorem Beta;\r\n",
    },
    SourceFixture {
        module_path: "gamma.delta",
        relative_path: "src/gamma/delta.miz",
        text: "theorem Gamma;\n",
    },
];

#[derive(Debug, Clone, Copy)]
struct SourceFixture {
    module_path: &'static str,
    relative_path: &'static str,
    text: &'static str,
}

#[derive(Debug)]
struct PackageFixture {
    root: PathBuf,
}

impl PackageFixture {
    fn new() -> Self {
        let fixture_id = NEXT_FIXTURE_ID.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "mizar-session-determinism-{}-{fixture_id}",
            std::process::id(),
        ));
        fs::create_dir_all(root.join("src")).expect("fixture root should be created");
        Self { root }
    }

    fn root(&self) -> &Path {
        &self.root
    }

    fn write(&self, relative_path: &str, text: &str) {
        let path = self.root.join(relative_path);
        fs::create_dir_all(path.parent().expect("fixture path should have a parent"))
            .expect("fixture parent directory should be created");
        fs::write(path, text).expect("fixture source should be written");
    }
}

impl Drop for PackageFixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn load_source_versions(package: &PackageFixture, order: [usize; 3]) -> Vec<SourceVersion> {
    let loader = DiskSourceLoader::new(package.root().to_owned());
    let ids = InMemorySessionIdAllocator::new();
    let allocation_snapshot = snapshot_id(0x51);

    order
        .into_iter()
        .map(|index| {
            let fixture = SOURCE_FIXTURES[index];
            let loaded = loader
                .load(
                    allocation_snapshot,
                    source_input(package.root(), fixture),
                    &ids,
                )
                .expect("fixture source should load");
            source_version_from_loaded(loaded)
        })
        .collect()
}

fn source_input(package_root: &Path, fixture: SourceFixture) -> SourceInput {
    let path = PathBuf::from(fixture.relative_path);
    SourceInput {
        package_id: PackageId::new("mml"),
        module_path: ModulePath::new(fixture.module_path),
        normalized_path: normalize_path(package_root, &path)
            .expect("fixture path should normalize before loading"),
        edition: Edition::new("2026"),
        origin: SourceOriginInput::Disk { path },
    }
}

fn source_version_from_loaded(loaded: LoadedSource) -> SourceVersion {
    SourceVersion {
        source_id: loaded.source_id,
        package_id: loaded.package_id,
        module_path: loaded.module_path,
        normalized_path: loaded.normalized_path,
        source_hash: loaded.source_hash,
        edition: loaded.edition,
        origin: loaded.origin,
    }
}

fn snapshot_input_for_order(
    package_root: &Path,
    source_versions: Vec<SourceVersion>,
    dependency_artifacts: Vec<DependencyArtifactRef>,
) -> SnapshotInput {
    SnapshotInput {
        workspace_root: WorkspaceRoot::new(package_root.to_string_lossy().into_owned()),
        source_versions,
        dependency_artifacts,
        lockfile_hash: hash(0x41),
        toolchain: ToolchainInfo::new("mizar-evo-test/2026"),
        verifier_config_hash: hash(0x42),
    }
}

fn dependency_artifacts(order: [usize; 2]) -> Vec<DependencyArtifactRef> {
    let dependencies = [
        DependencyArtifactRef::new("kernel/base.vo", hash(0x31)),
        DependencyArtifactRef::new("kernel/order.vo", hash(0x32)),
    ];
    order
        .into_iter()
        .map(|index| dependencies[index].clone())
        .collect()
}

fn create_snapshot(input: SnapshotInput) -> BuildSnapshot {
    let request_ids = InMemorySessionIdAllocator::new();
    let request = request_ids
        .next_request_id()
        .expect("request id should allocate");
    let registry = SnapshotRegistry::new();
    let (snapshot, _lease) = registry
        .create_snapshot(request, input)
        .expect("valid snapshot input should create a registry snapshot");
    snapshot
}

fn source_identity_summary(snapshot: &BuildSnapshot) -> Vec<(String, String, String)> {
    snapshot
        .source_versions
        .iter()
        .map(|version| {
            (
                version.package_id.as_str().to_owned(),
                version.module_path.as_str().to_owned(),
                version.normalized_path.as_str().to_owned(),
            )
        })
        .collect()
}

fn source_id_for_module(snapshot: &BuildSnapshot, module_path: &str) -> SourceId {
    snapshot
        .source_versions
        .iter()
        .find(|version| version.module_path.as_str() == module_path)
        .expect("snapshot should contain the requested module")
        .source_id
}

#[derive(Debug, Clone)]
struct SourceMaps {
    line: LineMap,
    loading: LoadingMap,
    preprocess: PreprocessMap,
}

fn retained_maps(source_id: SourceId) -> SourceMaps {
    let source_text = "abcde\nfghijklmnopqrstuvwxyz";
    SourceMaps {
        line: LineMap::with_source(source_id, source_text),
        loading: LoadingMap::new(
            source_id,
            source_text,
            LoadingOrigin::OpenBufferText {
                uri: "file:///workspace/src/determinism.miz".to_owned(),
                version: 7,
            },
            vec![
                LoadingMapSegment::Original {
                    loaded: TextRange { start: 0, end: 5 },
                    original: TextRange { start: 0, end: 5 },
                },
                LoadingMapSegment::NormalizedNewline {
                    loaded: TextRange { start: 5, end: 6 },
                    original: TextRange { start: 5, end: 7 },
                },
                LoadingMapSegment::Original {
                    loaded: TextRange {
                        start: 6,
                        end: source_text.len(),
                    },
                    original: TextRange {
                        start: 7,
                        end: source_text.len() + 1,
                    },
                },
            ],
        ),
        preprocess: PreprocessMap::new(
            source_id,
            "alpha  beta",
            vec![
                PreprocessSegment::Original {
                    lexical: TextRange { start: 0, end: 6 },
                    source: SourceRange {
                        source_id,
                        start: 0,
                        end: 6,
                    },
                },
                PreprocessSegment::RemovedComment {
                    source: SourceRange {
                        source_id,
                        start: 6,
                        end: 20,
                    },
                    kind: CommentKind::MultiLine,
                },
                PreprocessSegment::Original {
                    lexical: TextRange { start: 6, end: 11 },
                    source: SourceRange {
                        source_id,
                        start: 20,
                        end: 25,
                    },
                },
            ],
        ),
    }
}

fn service_with_maps_in_source_order(
    primary: &SourceMaps,
    secondary: &SourceMaps,
) -> RetainedSourceMapService {
    let mut service = RetainedSourceMapService::new();
    service.insert_line_map(primary.line.clone());
    service.insert_loading_map(primary.loading.clone());
    service.insert_preprocess_map(primary.preprocess.clone());
    service.insert_line_map(secondary.line.clone());
    service.insert_loading_map(secondary.loading.clone());
    service.insert_preprocess_map(secondary.preprocess.clone());
    service
}

fn service_with_maps_in_interleaved_order(
    primary: &SourceMaps,
    secondary: &SourceMaps,
) -> RetainedSourceMapService {
    let mut service = RetainedSourceMapService::new();
    service.insert_preprocess_map(secondary.preprocess.clone());
    service.insert_preprocess_map(primary.preprocess.clone());
    service.insert_loading_map(secondary.loading.clone());
    service.insert_loading_map(primary.loading.clone());
    service.insert_line_map(secondary.line.clone());
    service.insert_line_map(primary.line.clone());
    service
}

fn snapshot_id(seed: u8) -> BuildSnapshotId {
    let mut serialized = String::from("mizar-session-build-snapshot-v1:");
    for _ in 0..Hash::BYTE_LEN {
        write!(&mut serialized, "{seed:02x}").expect("writing to a string cannot fail");
    }
    BuildSnapshotId::from_published_schema_str(&serialized)
        .expect("test snapshot id should be well formed")
}

fn hash(seed: u8) -> Hash {
    Hash::from_bytes([seed; Hash::BYTE_LEN])
}
