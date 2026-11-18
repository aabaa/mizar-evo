use std::{collections::HashSet, sync::Arc};

use mizar_ir::{
    identity::{
        ExprIdentityInput, ItemIdentityInput, ModuleIdentityInput, NamedInputHash, OutputKind,
        PipelinePhase, SnapshotHandleRegistry, VcIdentityInput, WorkUnit,
    },
    publisher::{
        AllowedWorkUnit, OutputOrigin, PhaseOutputPublisher, PublicationTarget, PublishError,
        PublishOutputInput,
    },
    storage::{
        AnyPhaseOutputRef, BlobDecodeError, BlobDecoder, CollectInput, IrSideTables,
        IrStorageService, PhaseOutputRef, RetainOwner, SchemaVersion, SideTableRecord,
        StorageError,
    },
};
use mizar_session::{BuildSnapshotId, Hash};

fn hash(seed: u8) -> Hash {
    Hash::from_bytes([seed; Hash::BYTE_LEN])
}

fn snapshot(seed: u8) -> BuildSnapshotId {
    let hex = [seed; Hash::BYTE_LEN]
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    BuildSnapshotId::from_published_schema_str(&format!("mizar-session-build-snapshot-v1:{hex}"))
        .expect("test snapshot id is valid")
}

fn phase() -> PipelinePhase {
    PipelinePhase::new("resolve")
}

fn output_kind() -> OutputKind {
    OutputKind::new("ResolvedAst")
}

fn schema() -> SchemaVersion {
    SchemaVersion::new(1)
}

fn work_unit(value: impl Into<String>) -> WorkUnit {
    WorkUnit::new(value)
}

fn named(seed: u8) -> NamedInputHash {
    NamedInputHash {
        name: "source".to_owned(),
        domain: "test".to_owned(),
        digest: hash(seed),
    }
}

fn side_tables(seed: u8) -> IrSideTables {
    IrSideTables {
        source_maps: vec![SideTableRecord::new("source-map", "Main", hash(seed))],
        diagnostics: vec![SideTableRecord::new("diagnostic", "D001", hash(seed + 1))],
        explanation_refs: Vec::new(),
        documentation_attachments: Vec::new(),
    }
}

fn string_decoder() -> BlobDecoder<String> {
    BlobDecoder::new(|bytes| {
        String::from_utf8(bytes.to_vec()).map_err(|error| BlobDecodeError::new(error.to_string()))
    })
}

fn publisher(snapshot: BuildSnapshotId, units: &[String]) -> Arc<PhaseOutputPublisher> {
    let publisher = Arc::new(PhaseOutputPublisher::new(
        Arc::new(IrStorageService::new()),
        Arc::new(SnapshotHandleRegistry::new()),
    ));
    publisher.register_current_snapshot(snapshot);
    for unit in units {
        publisher.allow_work_unit(AllowedWorkUnit::new(
            phase(),
            output_kind(),
            work_unit(unit),
        ));
    }
    publisher
}

fn try_publish_text(
    publisher: &PhaseOutputPublisher,
    snapshot: BuildSnapshotId,
    unit: &str,
    payload: &str,
    parents: Vec<AnyPhaseOutputRef>,
    side_table_seed: u8,
) -> Result<PhaseOutputRef<String>, PublishError> {
    publisher.publish(PublishOutputInput {
        slot: publisher.allocate(snapshot, phase(), work_unit(unit), output_kind(), schema()),
        snapshot,
        phase: phase(),
        work_unit: work_unit(unit),
        output_kind: output_kind(),
        schema_version: schema(),
        payload: payload.to_owned(),
        storage_payload: Some(payload.as_bytes().to_vec()),
        canonical_payload: Some(payload.as_bytes().to_vec()),
        decode: string_decoder(),
        parents,
        named_input_hashes: vec![named(1)],
        side_tables: side_tables(side_table_seed),
        origin: OutputOrigin::PackageSource,
        target: PublicationTarget::CurrentPackage,
    })
}

fn publish_text(
    publisher: &PhaseOutputPublisher,
    snapshot: BuildSnapshotId,
    unit: &str,
    payload: &str,
    parents: Vec<AnyPhaseOutputRef>,
    side_table_seed: u8,
) -> PhaseOutputRef<String> {
    try_publish_text(publisher, snapshot, unit, payload, parents, side_table_seed)
        .expect("test output publishes")
}

fn identity_registry_values(snapshot: BuildSnapshotId) -> (Hash, Hash, Hash, Hash) {
    let registry = SnapshotHandleRegistry::new();
    registry.register_snapshot(snapshot);
    let module = registry
        .module_id(ModuleIdentityInput {
            snapshot,
            package_id: "pkg".to_owned(),
            module_path: "pkg/main".to_owned(),
            source_id: None,
            source_hash: hash(10),
        })
        .expect("module id derives");
    let item = registry
        .item_id(ItemIdentityInput {
            snapshot,
            module,
            item_kind: "theorem".to_owned(),
            origin_key: "Th1".to_owned(),
            declaration_order_key: "0001".to_owned(),
        })
        .expect("item id derives");
    let expr = registry
        .expr_id(ExprIdentityInput {
            snapshot,
            module,
            item: Some(item),
            expression_kind: "term".to_owned(),
            producer_path_key: "body/0".to_owned(),
        })
        .expect("expr id derives");
    let vc = registry
        .vc_id(VcIdentityInput {
            snapshot,
            module,
            item: Some(item),
            obligation_order_key: "goal/0".to_owned(),
            canonical_vc_fingerprint: Some(hash(11)),
        })
        .expect("vc id derives");

    (module.hash(), item.hash(), expr.hash(), vc.hash())
}

#[test]
fn identical_inputs_yield_identical_ids_hashes_and_lineage() {
    let snapshot = snapshot(1);
    assert_eq!(
        identity_registry_values(snapshot),
        identity_registry_values(snapshot)
    );

    let units = ["parent-a", "parent-b", "child"]
        .into_iter()
        .map(str::to_owned)
        .collect::<Vec<_>>();
    let left = publisher(snapshot, &units);
    let right = publisher(snapshot, &units);

    let left_parent_a = publish_text(&left, snapshot, "parent-a", "a", Vec::new(), 20);
    let left_parent_b = publish_text(&left, snapshot, "parent-b", "b", Vec::new(), 21);
    let right_parent_a = publish_text(&right, snapshot, "parent-a", "a", Vec::new(), 20);
    let right_parent_b = publish_text(&right, snapshot, "parent-b", "b", Vec::new(), 21);
    let left_child = publish_text(
        &left,
        snapshot,
        "child",
        "child",
        vec![left_parent_a.erase(), left_parent_b.erase()],
        22,
    );
    let right_child = publish_text(
        &right,
        snapshot,
        "child",
        "child",
        vec![right_parent_b.erase(), right_parent_a.erase()],
        22,
    );

    assert_eq!(left_child.output(), right_child.output());
    assert_eq!(left_child.content_hash(), right_child.content_hash());
    assert_eq!(left_child.side_table_hash(), right_child.side_table_hash());
    assert_eq!(left_child.lineage(), right_child.lineage());
}

#[test]
fn collected_handles_fail_closed_before_reuse() {
    let snapshot = snapshot(2);
    let units = ["parent", "child"]
        .into_iter()
        .map(str::to_owned)
        .collect::<Vec<_>>();
    let publisher = publisher(snapshot, &units);
    let parent = publish_text(&publisher, snapshot, "parent", "parent", Vec::new(), 30);

    let summary = publisher.storage().collect(CollectInput {
        snapshot,
        protected_outputs: Vec::new(),
    });
    assert_eq!(summary.outputs_dropped, 1);
    assert!(matches!(
        publisher.storage().get(&parent),
        Err(StorageError::CollectedOutput { output }) if output == parent.output()
    ));

    let error = try_publish_text(
        &publisher,
        snapshot,
        "child",
        "child",
        vec![parent.erase()],
        31,
    )
    .expect_err("collected parent fails before reuse");
    assert!(matches!(
        error,
        PublishError::Storage { error }
            if matches!(*error, StorageError::CollectedOutput { output } if output == parent.output())
    ));
}

#[test]
fn collection_is_idempotent_after_snapshot_replacement() {
    let old_snapshot = snapshot(3);
    let new_snapshot = snapshot(4);
    let units = ["old"].into_iter().map(str::to_owned).collect::<Vec<_>>();
    let publisher = publisher(old_snapshot, &units);
    let old = publish_text(&publisher, old_snapshot, "old", "old", Vec::new(), 40);

    publisher
        .replace_current_snapshot(old_snapshot, new_snapshot)
        .expect("snapshot replacement succeeds");
    let first = publisher.storage().collect(CollectInput {
        snapshot: old_snapshot,
        protected_outputs: Vec::new(),
    });
    let second = publisher.storage().collect(CollectInput {
        snapshot: old_snapshot,
        protected_outputs: Vec::new(),
    });

    assert_eq!(first.outputs_dropped, 1);
    assert_eq!(second.outputs_dropped, 0);
    assert_eq!(second.retained_outputs, 0);
    assert_eq!(second.protected_outputs, 0);
    assert!(matches!(
        publisher.storage().get(&old),
        Err(StorageError::CollectedOutput { output }) if output == old.output()
    ));
    assert!(matches!(
        publisher.validate_current_snapshot(old_snapshot),
        Err(PublishError::ObsoleteSnapshot { snapshot }) if snapshot == old_snapshot
    ));
    publisher
        .validate_current_snapshot(new_snapshot)
        .expect("replacement snapshot remains current");
}

#[test]
fn randomized_retain_release_sequence_does_not_leak_lifetimes() {
    let snapshot = snapshot(5);
    let units = (0..12)
        .map(|index| format!("unit-{index}"))
        .collect::<Vec<_>>();
    let publisher = publisher(snapshot, &units);
    let handles = units
        .iter()
        .enumerate()
        .map(|(index, unit)| {
            publish_text(
                &publisher,
                snapshot,
                unit,
                &format!("payload-{index}"),
                Vec::new(),
                50 + index as u8,
            )
        })
        .collect::<Vec<_>>();
    let owners = (0..5)
        .map(|index| RetainOwner::new(format!("owner-{index}")))
        .collect::<Vec<_>>();
    let mut live = HashSet::<(usize, usize)>::new();
    let mut rng = DeterministicRng::new(0x5eed_f00d_cafe_babe);

    for _ in 0..128 {
        let output = rng.index(handles.len());
        let owner = rng.index(owners.len());
        if rng.next_bool() {
            publisher
                .storage()
                .retain(handles[output].output(), owners[owner].clone())
                .expect("retain succeeds before collection");
            live.insert((output, owner));
        } else {
            publisher
                .storage()
                .release(handles[output].output(), &owners[owner])
                .expect("release succeeds before collection");
            live.remove(&(output, owner));
        }
    }

    publisher
        .storage()
        .retain(handles[0].output(), owners[0].clone())
        .expect("at least one output remains live");
    live.insert((0, 0));
    for (owner_index, owner) in owners.iter().enumerate() {
        publisher
            .storage()
            .release(handles[handles.len() - 1].output(), owner)
            .expect("release succeeds before collection");
        live.remove(&(handles.len() - 1, owner_index));
    }

    let retained_outputs = retained_output_count(handles.len(), &live);
    let first = publisher.storage().collect(CollectInput {
        snapshot,
        protected_outputs: Vec::new(),
    });
    assert_eq!(first.retained_outputs, retained_outputs);
    assert_eq!(first.outputs_dropped, handles.len() - retained_outputs);

    for (index, handle) in handles.iter().enumerate() {
        if output_has_owner(index, &live) {
            assert_eq!(
                &*publisher
                    .storage()
                    .get(handle)
                    .expect("retained output survives collection"),
                &format!("payload-{index}")
            );
        } else {
            assert!(matches!(
                publisher.storage().get(handle),
                Err(StorageError::CollectedOutput { output }) if output == handle.output()
            ));
        }
    }

    for (output_index, owner_index) in live.iter().copied().collect::<Vec<_>>() {
        publisher
            .storage()
            .release(handles[output_index].output(), &owners[owner_index])
            .expect("release of retained owner succeeds");
    }
    let second = publisher.storage().collect(CollectInput {
        snapshot,
        protected_outputs: Vec::new(),
    });
    assert_eq!(second.outputs_dropped, retained_outputs);
    assert_eq!(second.retained_outputs, 0);
    for handle in &handles {
        assert!(matches!(
            publisher.storage().get(handle),
            Err(StorageError::CollectedOutput { output }) if output == handle.output()
        ));
    }
    let third = publisher.storage().collect(CollectInput {
        snapshot,
        protected_outputs: Vec::new(),
    });
    assert_eq!(third.outputs_dropped, 0);
    assert_eq!(third.retained_outputs, 0);
}

fn retained_output_count(output_count: usize, live: &HashSet<(usize, usize)>) -> usize {
    (0..output_count)
        .filter(|output| output_has_owner(*output, live))
        .count()
}

fn output_has_owner(output: usize, live: &HashSet<(usize, usize)>) -> bool {
    live.iter()
        .any(|(live_output, _owner)| *live_output == output)
}

struct DeterministicRng {
    state: u64,
}

impl DeterministicRng {
    const fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    fn next_u64(&mut self) -> u64 {
        self.state = self
            .state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        self.state
    }

    fn next_bool(&mut self) -> bool {
        self.next_u64() & 1 == 0
    }

    fn index(&mut self, len: usize) -> usize {
        (self.next_u64() as usize) % len
    }
}

mod q1_comments {
    use super::*;
    use mizar_artifact::{
        comment_projection::{
            commented_artifact_json, read_commented_artifact, write_commented_artifact,
        },
        module_summary::{ModuleSummaryIdentity, SourceRangeSummary},
        registration_summary::{ArtifactHashClass, ArtifactHashRef},
        store::{CanonicalJson, SchemaVersion as ArtifactSchemaVersion},
        verified_artifact::{
            BuildProvenance, ExportVisibility, VerifiedArtifactReadOptions, VerifiedExport,
        },
    };
    use mizar_frontend::{
        lexical_env::{
            FrontendLexicalEnvironmentError, LexicalEnvironmentRequest, LexicalSummaryProvider,
            ResolvedImports,
        },
        orchestration::{CommentMetadata, Frontend, FrontendOutput},
        parsing::MizarParserSeam,
        source::{FrontendSourceLoader, source_unit_from_loaded},
    };
    use mizar_ir::{
        projection::{
            ArtifactProjectionService, CommentedArtifactDraft, ProjectVerifiedArtifactInput,
        },
        publisher::OutputAuthorization,
    };
    use mizar_session::{
        BuildSnapshot, DiskSourceLoader, Edition, InMemorySessionIdAllocator, ModulePath,
        PackageId, SessionIdAllocator, SnapshotInput, SnapshotRegistry, SourceInput, SourceOrigin,
        SourceOriginInput, ToolchainInfo, WorkspaceRoot, normalize_path,
    };
    use mizar_syntax::{SurfaceAst, SyntaxKind};
    use std::{
        fs,
        path::PathBuf,
        sync::atomic::{AtomicUsize, Ordering},
    };

    static NEXT: AtomicUsize = AtomicUsize::new(0);
    const TEXT: &str = "::: retained\n@[label(\"attached\")] public theorem Named: thesis;\n";

    struct EmptyImports;
    impl LexicalSummaryProvider for EmptyImports {
        fn resolve_imports(
            &self,
            _: &LexicalEnvironmentRequest<'_>,
        ) -> Result<ResolvedImports, FrontendLexicalEnvironmentError> {
            Ok(ResolvedImports {
                imports: vec![],
                summaries: vec![],
                diagnostics: vec![],
            })
        }
    }

    struct Fixture {
        root: PathBuf,
        snapshots: SnapshotRegistry,
        snapshot: BuildSnapshot,
        publisher: Arc<PhaseOutputPublisher>,
        output: FrontendOutput<SurfaceAst>,
        metadata: CommentMetadata,
        open: bool,
    }

    impl Fixture {
        fn new(text: &str, version: Option<i64>) -> Self {
            let root = std::env::temp_dir().join(format!(
                "mizar-ir-q1-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir_all(root.join("src")).unwrap();
            let path = root.join("src/main.miz");
            fs::write(&path, text).unwrap();
            let source = SourceInput {
                package_id: PackageId::new("fixture"),
                module_path: ModulePath::new("main"),
                normalized_path: normalize_path(&root, &path).unwrap(),
                edition: Edition::new("2026"),
                origin: match version {
                    None => SourceOriginInput::Disk { path: path.clone() },
                    Some(version) => SourceOriginInput::OpenBuffer {
                        uri: format!("file://{}", path.display()),
                        expected_version: version,
                        actual_version: version,
                        text: Arc::from(text),
                    },
                },
            };
            let snapshots = SnapshotRegistry::new();
            let (snapshot, _, mut loaded) = snapshots
                .load_snapshot(
                    InMemorySessionIdAllocator::new().next_request_id().unwrap(),
                    (
                        WorkspaceRoot::new(root.to_string_lossy()),
                        vec![],
                        hash(90),
                        ToolchainInfo::new("mizar-2026.1"),
                        hash(91),
                    ),
                    vec![(DiskSourceLoader::new(&root), source)],
                )
                .unwrap();
            let (loaded, _) = loaded.pop().unwrap();
            let (output, metadata) = Frontend::new(
                FrontendSourceLoader::new(DiskSourceLoader::new(&root)),
                EmptyImports,
                MizarParserSeam,
            )
            .run_loaded_with_comment_metadata(source_unit_from_loaded(loaded, path), text)
            .unwrap();
            let publisher = Arc::new(PhaseOutputPublisher::new(
                Arc::new(IrStorageService::new()),
                Arc::new(SnapshotHandleRegistry::new()),
            ));
            publisher.register_current_snapshot(snapshot.id);
            Self {
                root,
                snapshots,
                snapshot,
                publisher,
                output,
                metadata: metadata.unwrap(),
                open: version.is_some(),
            }
        }

        fn publish<T: Send + Sync + 'static>(
            &self,
            payload: T,
            unit: &str,
            frontend: bool,
            parents: Vec<AnyPhaseOutputRef>,
        ) -> PhaseOutputRef<T> {
            let phase = PipelinePhase::new(if frontend { "Frontend" } else { "verify" });
            let kind = OutputKind::new(if frontend {
                "FrontendOutput"
            } else {
                "VerifiedModule"
            });
            let work = WorkUnit::new(unit);
            self.publisher.allow_work_unit(AllowedWorkUnit::new(
                phase.clone(),
                kind.clone(),
                work.clone(),
            ));
            self.publisher
                .publish(PublishOutputInput {
                    slot: self.publisher.allocate(
                        self.snapshot.id,
                        phase.clone(),
                        work.clone(),
                        kind.clone(),
                        schema(),
                    ),
                    snapshot: self.snapshot.id,
                    phase,
                    work_unit: work,
                    output_kind: kind,
                    schema_version: schema(),
                    payload,
                    storage_payload: Some(unit.as_bytes().to_vec()),
                    canonical_payload: Some(unit.as_bytes().to_vec()),
                    decode: BlobDecoder::new(|_| {
                        Err(BlobDecodeError::new("resident-only fixture"))
                    }),
                    parents,
                    named_input_hashes: vec![],
                    side_tables: IrSideTables::default(),
                    origin: if self.open {
                        OutputOrigin::OpenBuffer
                    } else {
                        OutputOrigin::PackageSource
                    },
                    target: if self.open {
                        PublicationTarget::InternalOnly
                    } else {
                        PublicationTarget::CurrentPackage
                    },
                })
                .unwrap()
        }

        fn frontend(&self) -> PhaseOutputRef<FrontendOutput<SurfaceAst>> {
            self.publish(self.output.clone(), "frontend", true, vec![])
        }

        fn input(&self, required_outputs: Vec<AnyPhaseOutputRef>) -> ProjectVerifiedArtifactInput {
            let source = &self.output.source;
            let reference = |class, family, digest| {
                ArtifactHashRef::new(class, family, ArtifactSchemaVersion::new(1, 0), digest)
            };
            ProjectVerifiedArtifactInput {
                snapshot: self.snapshot.id,
                required_outputs,
                module: ModuleSummaryIdentity {
                    package_id: source.package_id.as_str().to_owned(),
                    package_version: Some("1.2.3".to_owned()),
                    lockfile_identity: Some("lock:fixture".to_owned()),
                    module_path: source.module_path.as_str().to_owned(),
                    language_edition: "2026".to_owned(),
                },
                source_file: "src/main.miz".to_owned(),
                source_hash: source.source_hash,
                verified_at: None,
                exports: vec![VerifiedExport {
                    origin_id: "Named".to_owned(),
                    fully_qualified_name: "Fixture::Named".to_owned(),
                    namespace_path: vec!["Fixture".to_owned()],
                    visibility: ExportVisibility::Public,
                    export_kind: "theorem".to_owned(),
                    source_range: SourceRangeSummary {
                        start_byte: 0,
                        end_byte: source.source_text.len() as u64,
                    },
                    rendered_signature: "theorem Named: thesis".to_owned(),
                    interface_fingerprint: reference(
                        ArtifactHashClass::Interface,
                        "mizar-ir/export",
                        hash(20),
                    ),
                    proof_status: None,
                    documentation_ref: None,
                }],
                statements: vec![],
                expressions: vec![],
                obligations: vec![],
                proof_witnesses: vec![],
                diagnostics: vec![],
                provenance: BuildProvenance {
                    captured_source: Some((
                        source.source_id,
                        source.origin.clone(),
                        source.source_hash,
                    )),
                    toolchain: "mizar-2026.1".to_owned(),
                    language_edition: "2026".to_owned(),
                    lockfile_hash: reference(
                        ArtifactHashClass::Artifact,
                        "mizar-build/lockfile",
                        hash(90),
                    ),
                    verifier_config_hash: reference(
                        ArtifactHashClass::Interface,
                        "mizar-proof/policy",
                        hash(91),
                    ),
                    dependency_artifact_hashes: vec![],
                    cache_key: None,
                },
            }
        }

        fn project(
            &self,
            frontend: &PhaseOutputRef<FrontendOutput<SurfaceAst>>,
            required: Vec<AnyPhaseOutputRef>,
        ) -> CommentedArtifactDraft {
            ArtifactProjectionService::new(Arc::clone(&self.publisher))
                .project_commented_module(
                    self.input(required),
                    &self.snapshots,
                    frontend,
                    &self.metadata,
                    &vec![None; self.metadata.boundaries().len()],
                )
                .unwrap()
        }
    }

    impl Drop for Fixture {
        fn drop(&mut self) {
            fs::remove_dir_all(&self.root).unwrap();
        }
    }

    fn field<'a>(json: &'a CanonicalJson, name: &str) -> &'a CanonicalJson {
        let CanonicalJson::Object(fields) = json else {
            panic!("expected object")
        };
        &fields[name]
    }
    fn rows(json: &CanonicalJson) -> &[CanonicalJson] {
        let CanonicalJson::Array(rows) = json else {
            panic!("expected array")
        };
        rows
    }

    fn foreign_source_id(original: mizar_session::SourceId) -> mizar_session::SourceId {
        let ids = InMemorySessionIdAllocator::new();
        let first = ids.next_source_id(snapshot(199)).unwrap();
        let foreign = if first == original {
            ids.next_source_id(snapshot(199)).unwrap()
        } else {
            first
        };
        assert_ne!(foreign, original);
        foreign
    }

    #[test]
    fn q1_comment_disk_draft_preserves_identity_and_explicit_targets() {
        let f = Fixture::new(TEXT, None);
        let frontend = f.frontend();
        let middle = f.publish("resolved", "resolved", false, vec![frontend.erase()]);
        let verified = f.publish("verified", "verified", false, vec![middle.erase()]);
        let mut targets = vec![None; f.metadata.boundaries().len()];
        let index = f
            .metadata
            .boundaries()
            .iter()
            .position(|b| b.kind() == SyntaxKind::TheoremItem)
            .unwrap();
        targets[index] = Some(("Named".to_owned(), "Fixture::Named".to_owned()));
        let input = f.input(vec![verified.erase(), frontend.erase()]);
        let expected_module = input.module.clone();
        let expected_provenance = input.provenance.clone();
        let draft = ArtifactProjectionService::new(Arc::clone(&f.publisher))
            .project_commented_module(input, &f.snapshots, &frontend, &f.metadata, &targets)
            .unwrap();
        assert_eq!(draft.snapshot, f.snapshot.id);
        assert_eq!(draft.required_outputs.len(), 2);
        assert!(draft.required_outputs.contains(&frontend.output()));
        assert!(draft.required_outputs.contains(&verified.output()));
        assert_eq!(draft.artifact.artifact.module, expected_module);
        assert_eq!(draft.artifact.artifact.provenance, expected_provenance);
        assert_eq!(draft.artifact.artifact.source_file, "src/main.miz");
        assert_eq!(
            draft.artifact.artifact.source_hash,
            f.output.source.source_hash
        );
        assert!(!draft.external_dependency_gaps.is_empty());
        let landmarks = rows(field(&draft.artifact.comments, "landmarks"));
        assert_eq!(
            rows(&landmarks[index])[4],
            CanonicalJson::array([
                CanonicalJson::string("Named"),
                CanonicalJson::string("Fixture::Named")
            ])
        );
        assert!(
            landmarks
                .iter()
                .any(|row| rows(row)[4] == CanonicalJson::Null)
        );
        let json = commented_artifact_json(&draft.artifact).unwrap();
        assert_eq!(
            read_commented_artifact(
                &json,
                VerifiedArtifactReadOptions {
                    artifact_path: None,
                    expected_module: Some(&expected_module),
                    expected_interface_hash: Some(draft.artifact.compute_interface_hash().unwrap()),
                    expected_implementation_hash: Some(
                        draft.artifact.compute_implementation_hash().unwrap()
                    )
                }
            )
            .unwrap(),
            draft.artifact
        );
        assert_eq!(*f.publisher.storage().get(&frontend).unwrap(), f.output);
    }

    #[test]
    fn q1_comment_open_buffer_frames_maps_wrappers_and_recovery_are_lossless() {
        let text = concat!(
            "\u{feff}::: Ω\r\n:::\r\n:: ordinary\r\n",
            "theorem::= β =::T: thesis;\r\n",
            "@[label(\"attached\")] public theorem Named: thesis;\r\n",
            "@show_type(x);\r\n",
            "definition algorithm broken() do return; end;\r\n"
        );
        let f = Fixture::new(text, Some(7));
        let frontend = f.frontend();
        let draft = f.project(&frontend, vec![frontend.erase()]);
        let comments = &draft.artifact.comments;
        assert_eq!(field(comments, "original"), &CanonicalJson::string(text));
        assert_eq!(
            field(comments, "loaded"),
            &CanonicalJson::string(text.trim_start_matches('\u{feff}').replace("\r\n", "\n"))
        );
        assert_eq!(
            field(comments, "lexical"),
            &CanonicalJson::string(f.metadata.lexical_text())
        );
        let loading = rows(field(comments, "loading"));
        assert_eq!(
            loading[0],
            CanonicalJson::array([
                CanonicalJson::string("bom"),
                CanonicalJson::Null,
                CanonicalJson::array([CanonicalJson::integer(0), CanonicalJson::integer(3)])
            ])
        );
        assert!(
            loading
                .iter()
                .any(|row| rows(row)[0] == CanonicalJson::string("newline"))
        );
        let preprocess = rows(field(comments, "preprocess"));
        assert!(
            preprocess
                .iter()
                .any(|row| rows(row)[0] == CanonicalJson::string("synthetic"))
        );
        assert_eq!(preprocess.len(), f.metadata.preprocess_map().segments.len());
        assert_eq!(loading.len(), f.metadata.loading_segments().len());
        let loaded = f.metadata.loaded_text();
        let anchor = loaded.find("::= β =::").unwrap();
        let lexical = f.metadata.lexical_text().find("theorem T").unwrap() + "theorem".len();
        assert!(preprocess.contains(&CanonicalJson::array([
            CanonicalJson::string("synthetic"),
            CanonicalJson::array([
                CanonicalJson::integer(lexical as i64),
                CanonicalJson::integer((lexical + 1) as i64)
            ]),
            CanonicalJson::array([
                CanonicalJson::integer(anchor as i64),
                CanonicalJson::integer((anchor + "::= β =::".len()) as i64)
            ]),
            CanonicalJson::Null,
        ])));
        assert_eq!(
            rows(field(comments, "whitespace")).len(),
            f.metadata.whitespace().len()
        );
        for (row, span) in rows(field(comments, "whitespace"))
            .iter()
            .zip(f.metadata.whitespace())
        {
            assert_eq!(
                *row,
                CanonicalJson::array([
                    CanonicalJson::integer(span.start as i64),
                    CanonicalJson::integer(span.end as i64),
                ])
            );
        }
        let comment_rows = rows(field(comments, "comments"));
        assert_eq!(comment_rows.len(), 4);
        for (row, raw) in
            comment_rows
                .iter()
                .zip(["::: Ω\n", ":::\n", ":: ordinary\n", "::= β =::"])
        {
            assert_eq!(rows(row)[2], CanonicalJson::string(raw));
        }
        let landmarks = rows(field(comments, "landmarks"));
        assert_eq!(landmarks.len(), f.metadata.boundaries().len());
        for (row, boundary) in landmarks.iter().zip(f.metadata.boundaries()) {
            let row = rows(row);
            assert_eq!(
                row[0],
                CanonicalJson::string(format!("{:?}", boundary.kind()))
            );
            assert_eq!(
                row[1],
                CanonicalJson::array([
                    CanonicalJson::integer(boundary.range().start as i64),
                    CanonicalJson::integer(boundary.range().end as i64)
                ])
            );
            assert_eq!(
                row[2],
                boundary
                    .parent()
                    .map_or(CanonicalJson::Null, |parent| CanonicalJson::integer(
                        parent as i64
                    ))
            );
            assert_eq!(row[3], CanonicalJson::bool(boundary.recovered()));
        }
        for kind in [
            "VisibleItem",
            "Annotation",
            "StandaloneDiagnosticAnnotation",
            "TheoremItem",
            "DefinitionBlockItem",
            "AlgorithmDefinition",
        ] {
            assert!(
                landmarks
                    .iter()
                    .any(|row| rows(row)[0] == CanonicalJson::string(kind)),
                "missing {kind}"
            );
        }
        assert!(
            landmarks
                .iter()
                .any(|row| rows(row)[2] != CanonicalJson::Null)
        );
        assert!(
            landmarks
                .iter()
                .any(|row| rows(row)[3] == CanonicalJson::bool(true))
        );
        assert!(
            landmarks
                .iter()
                .all(|row| rows(row)[4] == CanonicalJson::Null)
        );
        assert_eq!(
            draft
                .artifact
                .artifact
                .provenance
                .captured_source
                .as_ref()
                .unwrap()
                .1,
            SourceOrigin::OpenBuffer { version: 7 }
        );
        assert!(
            f.publisher
                .validate_output(
                    f.snapshot.id,
                    &frontend.erase(),
                    OutputAuthorization::CurrentPackage
                )
                .is_err()
        );
        write_commented_artifact(&draft.artifact).unwrap();
    }

    #[test]
    fn q1_comment_availability_hashes_lexical_reuse_and_determinism() {
        let empty = Fixture::new("", None);
        let doc = Fixture::new(":::", None);
        let a = Fixture::new(":: short\ntheorem Named: thesis;\n", None);
        let b = Fixture::new(":: longer changed body\ntheorem Named: thesis;\n", None);
        let project = |f: &Fixture| {
            let frontend = f.frontend();
            f.project(&frontend, vec![frontend.erase()]).artifact
        };
        let empty_artifact = project(&empty);
        let doc_artifact = project(&doc);
        assert_ne!(empty_artifact.comments, CanonicalJson::Null);
        assert!(rows(field(&empty_artifact.comments, "comments")).is_empty());
        assert_eq!(rows(field(&doc_artifact.comments, "comments")).len(), 1);
        let mut unavailable = empty_artifact.clone();
        unavailable.comments = CanonicalJson::Null;
        assert_ne!(
            unavailable.compute_implementation_hash().unwrap(),
            empty_artifact.compute_implementation_hash().unwrap()
        );
        assert_ne!(
            empty_artifact.compute_artifact_hash().unwrap(),
            doc_artifact.compute_artifact_hash().unwrap()
        );
        assert_eq!(
            a.output.preprocessed.lexical_hash,
            b.output.preprocessed.lexical_hash
        );
        assert_eq!(a.output.cache_keys.tokens, b.output.cache_keys.tokens);
        assert_eq!(a.output.cache_keys.ast, b.output.cache_keys.ast);
        let aa = project(&a);
        let bb = project(&b);
        assert_eq!(
            aa.compute_interface_hash().unwrap(),
            bb.compute_interface_hash().unwrap()
        );
        assert_ne!(
            aa.compute_implementation_hash().unwrap(),
            bb.compute_implementation_hash().unwrap()
        );
        assert_ne!(
            aa.compute_artifact_hash().unwrap(),
            bb.compute_artifact_hash().unwrap()
        );
        let f = Fixture::new(TEXT, None);
        let frontend = f.frontend();
        let child = f.publish("verified", "verified", false, vec![frontend.erase()]);
        let first = f.project(&frontend, vec![frontend.erase(), child.erase()]);
        let second = f.project(&frontend, vec![child.erase(), frontend.erase()]);
        assert_eq!(
            write_commented_artifact(&first.artifact).unwrap(),
            write_commented_artifact(&second.artifact).unwrap()
        );
        let repeat = Fixture::new(TEXT, None);
        let repeat_frontend = repeat.frontend();
        let repeated = repeat.project(&repeat_frontend, vec![repeat_frontend.erase()]);
        assert_eq!(first.artifact.comments, repeated.artifact.comments);
        assert_eq!(
            first.artifact.compute_artifact_hash().unwrap(),
            repeated.artifact.compute_artifact_hash().unwrap()
        );
    }

    #[test]
    fn q1_comment_refuses_payload_snapshot_identity_and_provenance_substitutions() {
        let f = Fixture::new(TEXT, None);
        let service = ArtifactProjectionService::new(Arc::clone(&f.publisher));
        let targets = vec![None; f.metadata.boundaries().len()];
        for variant in 0..6 {
            let mut payload = f.output.clone();
            match variant {
                0 => payload.source.source_text = Arc::from("theorem Other: thesis;"),
                1 => payload
                    .preprocessed
                    .source_map
                    .preprocess_map
                    .segments
                    .clear(),
                2 => payload.tokens.tokens[0].span.end += 1,
                3 => payload.ast = None,
                4 => payload.source.source_id = foreign_source_id(payload.source.source_id),
                _ => {
                    payload.source.line_map =
                        mizar_session::LineMap::new(payload.source.source_id, "different\nframe");
                }
            }
            let handle = f.publish(payload, &format!("substitution-{variant}"), true, vec![]);
            assert!(
                service
                    .project_commented_module(
                        f.input(vec![handle.erase()]),
                        &f.snapshots,
                        &handle,
                        &f.metadata,
                        &targets
                    )
                    .is_err(),
                "payload substitution {variant}"
            );
        }
        let frontend = f.frontend();
        for variant in 0..13 {
            let mut input = f.input(vec![frontend.erase()]);
            match variant {
                0 => input.module.package_id = "foreign".to_owned(),
                1 => input.module.module_path = "other".to_owned(),
                2 => input.source_file = "other/main.miz".to_owned(),
                3 => input.module.language_edition = "2025".to_owned(),
                4 => input.source_hash = hash(101),
                5 => input.provenance.captured_source = None,
                6 => {
                    input.provenance.captured_source.as_mut().unwrap().0 =
                        foreign_source_id(f.output.source.source_id)
                }
                7 => {
                    input.provenance.captured_source.as_mut().unwrap().1 =
                        SourceOrigin::OpenBuffer { version: 1 }
                }
                8 => input.provenance.captured_source.as_mut().unwrap().2 = hash(102),
                9 => input.provenance.toolchain = "other-toolchain".to_owned(),
                10 => input.provenance.language_edition = "2025".to_owned(),
                11 => input.provenance.lockfile_hash.digest = hash(103),
                _ => input.provenance.verifier_config_hash.digest = hash(104),
            }
            assert!(
                service
                    .project_commented_module(input, &f.snapshots, &frontend, &f.metadata, &targets)
                    .is_err(),
                "identity substitution {variant}"
            );
        }
        assert!(
            service
                .project_commented_module(
                    f.input(vec![frontend.erase()]),
                    &SnapshotRegistry::new(),
                    &frontend,
                    &f.metadata,
                    &targets
                )
                .is_err()
        );
        let before = f.output.clone();
        // A detached public snapshot record cannot replace the registry's contents.
        let mut detached = f.snapshots.get(f.snapshot.id).unwrap();
        detached.source_versions[0].source_hash = hash(105);
        assert_ne!(detached, f.snapshots.get(f.snapshot.id).unwrap());
        f.project(&frontend, vec![frontend.erase()]);
        assert_eq!(*f.publisher.storage().get(&frontend).unwrap(), before);
        for variant in 0..8 {
            let mut bad = Fixture::new(TEXT, Some(7));
            let mut versions = bad.snapshot.source_versions.clone();
            match variant {
                0 => versions[0].source_id = foreign_source_id(versions[0].source_id),
                1 => versions[0].package_id = PackageId::new("foreign"),
                2 => versions[0].module_path = ModulePath::new("other"),
                3 => {
                    fs::write(bad.root.join("src/other.miz"), TEXT).unwrap();
                    versions[0].normalized_path =
                        normalize_path(&bad.root, &bad.root.join("src/other.miz")).unwrap()
                }
                4 => versions[0].source_hash = hash(106),
                5 => versions[0].edition = Edition::new("2025"),
                6 => versions[0].origin = SourceOrigin::Disk,
                _ => versions[0].origin = SourceOrigin::OpenBuffer { version: 8 },
            }
            let (registered, _) = bad
                .snapshots
                .create_snapshot(
                    InMemorySessionIdAllocator::new().next_request_id().unwrap(),
                    SnapshotInput {
                        workspace_root: bad.snapshot.workspace_root.clone(),
                        source_versions: versions,
                        dependency_artifacts: vec![],
                        lockfile_hash: bad.snapshot.lockfile_hash,
                        toolchain: bad.snapshot.toolchain.clone(),
                        verifier_config_hash: bad.snapshot.verifier_config_hash,
                    },
                )
                .unwrap();
            bad.snapshot = registered;
            bad.publisher.register_current_snapshot(bad.snapshot.id);
            let handle = bad.frontend();
            assert!(
                ArtifactProjectionService::new(Arc::clone(&bad.publisher))
                    .project_commented_module(
                        bad.input(vec![handle.erase()]),
                        &bad.snapshots,
                        &handle,
                        &bad.metadata,
                        &vec![None; bad.metadata.boundaries().len()]
                    )
                    .is_err(),
                "registered source substitution {variant}"
            );
        }
    }

    #[test]
    fn q1_comment_refuses_missing_unpublished_unreadable_and_unrelated_outputs() {
        let f = Fixture::new(TEXT, None);
        let frontend = f.frontend();
        let targets = vec![None; f.metadata.boundaries().len()];
        let service = ArtifactProjectionService::new(Arc::clone(&f.publisher));
        let before = f.publisher.storage().get(&frontend).unwrap();
        for variant in 0..3 {
            let phase = PipelinePhase::new(if variant == 0 { "Other" } else { "Frontend" });
            let kind = OutputKind::new(if variant == 1 {
                "Other"
            } else {
                "FrontendOutput"
            });
            let version = SchemaVersion::new(if variant == 2 { 2 } else { 1 });
            let unit = WorkUnit::new(format!("frontend-tag-{variant}"));
            f.publisher.allow_work_unit(AllowedWorkUnit::new(
                phase.clone(),
                kind.clone(),
                unit.clone(),
            ));
            let handle = f
                .publisher
                .publish(PublishOutputInput {
                    slot: f.publisher.allocate(
                        f.snapshot.id,
                        phase.clone(),
                        unit.clone(),
                        kind.clone(),
                        version,
                    ),
                    snapshot: f.snapshot.id,
                    phase,
                    work_unit: unit,
                    output_kind: kind,
                    schema_version: version,
                    payload: f.output.clone(),
                    canonical_payload: Some(vec![variant]),
                    storage_payload: Some(vec![variant]),
                    decode: BlobDecoder::new(|_| Err(BlobDecodeError::new("resident fixture"))),
                    parents: vec![],
                    named_input_hashes: vec![],
                    side_tables: IrSideTables::default(),
                    origin: OutputOrigin::PackageSource,
                    target: PublicationTarget::CurrentPackage,
                })
                .unwrap();
            f.publisher
                .validate_output(f.snapshot.id, handle.any(), OutputAuthorization::Published)
                .unwrap();
            assert_eq!(*f.publisher.storage().get(&handle).unwrap(), f.output);
            assert!(matches!(
                service.project_commented_module(
                    f.input(vec![handle.erase()]),
                    &f.snapshots,
                    &handle,
                    &f.metadata,
                    &targets,
                ),
                Err(mizar_ir::projection::ProjectionError::CommentBinding { .. })
            ));
        }
        let unrelated = f.publish("unrelated", "unrelated", false, vec![]);
        let foreign = Fixture::new("::: foreign\n", None);
        let foreign_output = foreign.frontend();
        for required in [
            vec![],
            vec![unrelated.erase()],
            vec![frontend.erase(), frontend.erase()],
            vec![frontend.erase(), unrelated.erase()],
            vec![frontend.erase(), foreign_output.erase()],
        ] {
            assert!(
                service
                    .project_commented_module(
                        f.input(required),
                        &f.snapshots,
                        &frontend,
                        &f.metadata,
                        &targets
                    )
                    .is_err()
            );
        }
        let unrooted = Arc::new(PhaseOutputPublisher::new(
            Arc::clone(f.publisher.storage()),
            Arc::clone(f.publisher.registry()),
        ));
        unrooted.register_current_snapshot(f.snapshot.id);
        assert!(
            ArtifactProjectionService::new(unrooted)
                .project_commented_module(
                    f.input(vec![frontend.erase()]),
                    &f.snapshots,
                    &frontend,
                    &f.metadata,
                    &targets
                )
                .is_err()
        );
        assert_eq!(*f.publisher.storage().get(&frontend).unwrap(), *before);
        // Current publication and lineage records survive storage collection;
        // those records alone must not grant readability.
        f.publisher.storage().collect(CollectInput {
            snapshot: f.snapshot.id,
            protected_outputs: vec![],
        });
        assert!(
            service
                .project_commented_module(
                    f.input(vec![frontend.erase()]),
                    &f.snapshots,
                    &frontend,
                    &f.metadata,
                    &targets
                )
                .is_err()
        );
        let obsolete = Fixture::new(TEXT, None);
        let old = obsolete.frontend();
        obsolete
            .publisher
            .mark_obsolete(obsolete.snapshot.id)
            .unwrap();
        assert!(
            ArtifactProjectionService::new(Arc::clone(&obsolete.publisher))
                .project_commented_module(
                    obsolete.input(vec![old.erase()]),
                    &obsolete.snapshots,
                    &old,
                    &obsolete.metadata,
                    &vec![None; obsolete.metadata.boundaries().len()]
                )
                .is_err()
        );
        let mut corrupt = Fixture::new(TEXT, None);
        corrupt.publisher = Arc::new(PhaseOutputPublisher::new(
            Arc::new(IrStorageService::with_policy(
                mizar_ir::storage::StoragePolicy {
                    blob_spill_threshold: 0,
                },
            )),
            Arc::new(SnapshotHandleRegistry::new()),
        ));
        corrupt
            .publisher
            .register_current_snapshot(corrupt.snapshot.id);
        let unreadable = corrupt.frontend();
        assert!(corrupt.publisher.storage().get(&unreadable).is_err());
        assert!(
            ArtifactProjectionService::new(Arc::clone(&corrupt.publisher))
                .project_commented_module(
                    corrupt.input(vec![unreadable.erase()]),
                    &corrupt.snapshots,
                    &unreadable,
                    &corrupt.metadata,
                    &vec![None; corrupt.metadata.boundaries().len()]
                )
                .is_err()
        );
    }

    #[test]
    fn q1_comment_targets_codec_bounds_raw_leakage_and_legacy_compatibility() {
        let f = Fixture::new(TEXT, None);
        let frontend = f.frontend();
        let service = ArtifactProjectionService::new(Arc::clone(&f.publisher));
        let targets = vec![None; f.metadata.boundaries().len()];
        for bad in [
            vec![],
            vec![None; targets.len() + 1],
            vec![Some(("missing".to_owned(), "Fixture::Named".to_owned())); targets.len()],
            vec![Some(("Named".to_owned(), "Other::Named".to_owned())); targets.len()],
        ] {
            assert!(
                service
                    .project_commented_module(
                        f.input(vec![frontend.erase()]),
                        &f.snapshots,
                        &frontend,
                        &f.metadata,
                        &bad
                    )
                    .is_err()
            );
        }
        let draft = f.project(&frontend, vec![frontend.erase()]);
        assert!(
            rows(field(&draft.artifact.comments, "landmarks"))
                .iter()
                .all(|row| rows(row)[4] == CanonicalJson::Null)
        );
        for key in ["loading", "preprocess", "whitespace", "landmarks"] {
            let mut bad = draft.artifact.clone();
            let CanonicalJson::Object(fields) = &mut bad.comments else {
                panic!("available comments")
            };
            fields.insert(
                key.to_owned(),
                CanonicalJson::array([CanonicalJson::array([
                    CanonicalJson::integer(-1),
                    CanonicalJson::integer(i64::MAX),
                ])]),
            );
            assert!(write_commented_artifact(&bad).is_err(), "invalid {key}");
        }
        let mut leaked = f.input(vec![frontend.erase()]);
        leaked.exports[0].rendered_signature = "CoreIr { raw: true }".to_owned();
        assert!(
            service
                .project_commented_module(leaked, &f.snapshots, &frontend, &f.metadata, &targets)
                .is_err()
        );
        let legacy = service
            .project_module(f.input(vec![frontend.erase()]))
            .unwrap();
        assert_eq!(legacy.artifact, draft.artifact.artifact);
        assert_eq!(
            legacy.external_dependency_gaps,
            draft.external_dependency_gaps
        );
        assert_eq!(*f.publisher.storage().get(&frontend).unwrap(), f.output);
    }
}
