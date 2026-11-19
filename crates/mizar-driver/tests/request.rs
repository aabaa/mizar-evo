use mizar_driver::request::{
    BatchInvocation, BatchRequest, BuildLaneId, BuildProfile, BuildRequestDraft,
    BuildRequestGeneration, BuildRequestOrigin, BuildSessionOutcome, BuildSessionState,
    BuildTargets, DependencyInputSet, DriverLanes, LspFocus, LspPriority, LspRequest,
    PublicationDecision, SourceInputSet, VerifierConfigInput, WatchRequest,
};
use mizar_session::{
    DependencyArtifactRef, Edition, Hash, InMemorySessionIdAllocator, ModulePath, NormalizedPath,
    PackageId, RetentionReason, SessionIdAllocator, SnapshotError, SnapshotRegistry, SourceOrigin,
    SourceVersion, ToolchainInfo, WorkspaceRoot, normalize_source_path,
};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT_FIXTURE_ID: AtomicUsize = AtomicUsize::new(0);

#[test]
fn constructs_batch_watch_and_lsp_requests_without_protocol_payloads() {
    let ids = InMemorySessionIdAllocator::new();
    let batch = draft(
        BuildLaneId::new(1),
        BuildRequestGeneration::new(0),
        BuildRequestOrigin::Batch(BatchRequest {
            invocation: BatchInvocation {
                args: vec!["verify".to_owned(), "all".to_owned()],
            },
        }),
        snapshot_input_parts(1),
    )
    .allocate(&ids)
    .unwrap();
    let watch = draft(
        BuildLaneId::new(2),
        BuildRequestGeneration::new(7),
        BuildRequestOrigin::Watch(WatchRequest {
            watch_root: WorkspaceRoot::new("workspace"),
            changed_paths: vec![normalized_path("src/alpha.miz")],
        }),
        snapshot_input_parts(2),
    )
    .allocate(&ids)
    .unwrap();
    let lsp = draft(
        BuildLaneId::new(3),
        BuildRequestGeneration::new(11),
        BuildRequestOrigin::Lsp(LspRequest {
            focus: Some(LspFocus {
                package: PackageId::new("mml"),
                module: ModulePath::new("alpha"),
            }),
            priority: LspPriority::Focused,
        }),
        snapshot_input_parts(3),
    )
    .allocate(&ids)
    .unwrap();

    assert!(matches!(batch.request.origin, BuildRequestOrigin::Batch(_)));
    assert!(matches!(watch.request.origin, BuildRequestOrigin::Watch(_)));
    assert!(matches!(lsp.request.origin, BuildRequestOrigin::Lsp(_)));
    assert_ne!(batch.request.id, watch.request.id);
    assert_ne!(watch.request.id, lsp.request.id);
}

#[test]
fn captures_snapshot_through_session_registry_with_active_build_lease() {
    let registry = SnapshotRegistry::new();
    let session = pending_session(BuildLaneId::new(4), BuildRequestGeneration::new(0), 4)
        .capture_snapshot(&registry)
        .unwrap();

    assert_eq!(session.state(), BuildSessionState::SnapshotCaptured);
    assert_eq!(
        session.captured().active_snapshot_lease().snapshot,
        session.captured().snapshot().id
    );
    assert_eq!(
        session.captured().active_snapshot_lease().reason,
        RetentionReason::ActiveBuild
    );
    assert!(
        registry.is_current_for_request(session.captured().snapshot().id, session.request().id)
    );
}

#[test]
fn snapshot_capture_error_returns_pending_request_context() {
    let registry = SnapshotRegistry::new();
    let ids = InMemorySessionIdAllocator::new();
    let mut parts = snapshot_input_parts(15);
    parts.workspace_root = WorkspaceRoot::new("");
    let pending = draft(
        BuildLaneId::new(15),
        BuildRequestGeneration::new(0),
        BuildRequestOrigin::Batch(BatchRequest {
            invocation: BatchInvocation::default(),
        }),
        parts,
    )
    .allocate(&ids)
    .unwrap();
    let session_id = pending.session_id;
    let request_id = pending.request.id;
    let error = pending.capture_snapshot(&registry).unwrap_err();

    assert_eq!(error.pending.session_id, session_id);
    assert_eq!(error.pending.request.id, request_id);
    assert!(matches!(
        error.error,
        SnapshotError::InvalidWorkspaceRoot { .. }
    ));
}

#[test]
fn snapshot_input_preserves_request_identity_fields() {
    let ids = InMemorySessionIdAllocator::new();
    let parts = snapshot_input_parts(15);
    let expected = parts.clone().snapshot_input();
    let pending = draft(
        BuildLaneId::new(15),
        BuildRequestGeneration::new(0),
        BuildRequestOrigin::Batch(BatchRequest {
            invocation: BatchInvocation::default(),
        }),
        parts,
    )
    .allocate(&ids)
    .unwrap();

    assert_eq!(pending.request.snapshot_input(), expected);
}

#[test]
fn identical_snapshot_inputs_reuse_snapshot_id_but_not_request_or_session_ids() {
    let registry = SnapshotRegistry::new();
    let ids = InMemorySessionIdAllocator::new();
    let lane = BuildLaneId::new(5);
    let first = pending_session_with_ids(&ids, lane, BuildRequestGeneration::new(0), 5)
        .capture_snapshot(&registry)
        .unwrap();
    let second = pending_session_with_ids(&ids, lane, BuildRequestGeneration::new(1), 5)
        .capture_snapshot(&registry)
        .unwrap();

    assert_eq!(
        first.captured().snapshot().id,
        second.captured().snapshot().id
    );
    assert_ne!(first.id(), second.id());
    assert_ne!(first.request().id, second.request().id);
}

#[test]
fn watch_lsp_generations_share_lane_while_request_ids_stay_fresh() {
    let ids = InMemorySessionIdAllocator::new();
    let first =
        pending_session_with_ids(&ids, BuildLaneId::new(6), BuildRequestGeneration::new(0), 6);
    let second =
        pending_session_with_ids(&ids, BuildLaneId::new(6), BuildRequestGeneration::new(1), 7);

    assert_eq!(first.request.lane, second.request.lane);
    assert_ne!(first.request.generation, second.request.generation);
    assert_ne!(first.request.id, second.request.id);
}

#[test]
fn lane_supersession_rejects_older_session_even_for_same_snapshot_id() {
    let registry = SnapshotRegistry::new();
    let ids = InMemorySessionIdAllocator::new();
    let mut lanes = DriverLanes::default();
    let lane = BuildLaneId::new(7);
    let first = pending_session_with_ids(&ids, lane, BuildRequestGeneration::new(0), 8)
        .capture_snapshot(&registry)
        .unwrap();
    let second = pending_session_with_ids(&ids, lane, BuildRequestGeneration::new(1), 8)
        .capture_snapshot(&registry)
        .unwrap();

    assert_eq!(
        first.captured().snapshot().id,
        second.captured().snapshot().id
    );
    assert!(lanes.mark_current(&first));
    assert_eq!(
        lanes.publication_decision(&registry, &first),
        PublicationDecision::Current
    );

    assert!(lanes.mark_current(&second));

    assert_eq!(
        lanes.publication_decision(&registry, &first),
        PublicationDecision::Suppressed(mizar_driver::request::ObsoletePublication {
            lane_current: false,
            request_snapshot_current: true,
        })
    );
    assert_eq!(
        lanes.publication_decision(&registry, &second),
        PublicationDecision::Current
    );
}

#[test]
fn lane_currentness_rejects_older_generation_reactivation() {
    let registry = SnapshotRegistry::new();
    let ids = InMemorySessionIdAllocator::new();
    let mut lanes = DriverLanes::default();
    let lane = BuildLaneId::new(16);
    let first = pending_session_with_ids(&ids, lane, BuildRequestGeneration::new(0), 16)
        .capture_snapshot(&registry)
        .unwrap();
    let second = pending_session_with_ids(&ids, lane, BuildRequestGeneration::new(1), 17)
        .capture_snapshot(&registry)
        .unwrap();

    assert!(lanes.mark_current(&first));
    assert!(lanes.mark_current(&second));
    assert!(!lanes.mark_current(&first));
    assert_eq!(
        lanes.publication_decision(&registry, &first),
        PublicationDecision::Suppressed(mizar_driver::request::ObsoletePublication {
            lane_current: false,
            request_snapshot_current: true,
        })
    );
    assert_eq!(
        lanes.publication_decision(&registry, &second),
        PublicationDecision::Current
    );
}

#[test]
fn lane_currentness_rejects_same_generation_different_session() {
    let registry = SnapshotRegistry::new();
    let ids = InMemorySessionIdAllocator::new();
    let mut lanes = DriverLanes::default();
    let lane = BuildLaneId::new(17);
    let first = pending_session_with_ids(&ids, lane, BuildRequestGeneration::new(0), 18)
        .capture_snapshot(&registry)
        .unwrap();
    let duplicate_generation =
        pending_session_with_ids(&ids, lane, BuildRequestGeneration::new(0), 19)
            .capture_snapshot(&registry)
            .unwrap();

    assert!(lanes.mark_current(&first));
    assert!(lanes.mark_current(&first));
    assert!(!lanes.mark_current(&duplicate_generation));
    assert_eq!(
        lanes.publication_decision(&registry, &first),
        PublicationDecision::Current
    );
    assert_eq!(
        lanes.publication_decision(&registry, &duplicate_generation),
        PublicationDecision::Suppressed(mizar_driver::request::ObsoletePublication {
            lane_current: false,
            request_snapshot_current: true,
        })
    );
}

#[test]
fn publication_guard_rejects_stale_request_snapshot_even_when_lane_matches() {
    let registry = SnapshotRegistry::new();
    let mut lanes = DriverLanes::default();
    let lane = BuildLaneId::new(8);
    let stale = pending_session(lane, BuildRequestGeneration::new(0), 9)
        .capture_snapshot(&registry)
        .unwrap();
    let _newer_same_request = registry
        .create_snapshot(
            stale.request().id,
            snapshot_input_parts(10).snapshot_input(),
        )
        .unwrap();
    assert!(lanes.mark_current(&stale));

    assert_eq!(
        lanes.publication_decision(&registry, &stale),
        PublicationDecision::Suppressed(mizar_driver::request::ObsoletePublication {
            lane_current: true,
            request_snapshot_current: false,
        })
    );
}

#[test]
fn stale_suppressed_decision_does_not_publish_current_state() {
    let registry = SnapshotRegistry::new();
    let lanes = DriverLanes::default();
    let session = pending_session(BuildLaneId::new(9), BuildRequestGeneration::new(0), 11)
        .capture_snapshot(&registry)
        .unwrap();

    assert_eq!(
        lanes.publication_decision(&registry, &session),
        PublicationDecision::Suppressed(mizar_driver::request::ObsoletePublication {
            lane_current: false,
            request_snapshot_current: true,
        })
    );
}

#[test]
fn cancellation_is_idempotent_for_current_and_superseded_sessions() {
    let registry = SnapshotRegistry::new();
    let ids = InMemorySessionIdAllocator::new();
    let mut lanes = DriverLanes::default();
    let lane = BuildLaneId::new(10);
    let mut superseded = pending_session_with_ids(&ids, lane, BuildRequestGeneration::new(0), 12)
        .capture_snapshot(&registry)
        .unwrap();
    let mut current = pending_session_with_ids(&ids, lane, BuildRequestGeneration::new(1), 13)
        .capture_snapshot(&registry)
        .unwrap();
    assert!(lanes.mark_current(&superseded));
    assert!(lanes.mark_current(&current));
    assert!(superseded.finish(BuildSessionOutcome::Superseded));

    assert!(!superseded.cancel());
    assert!(current.cancel());
    assert!(!current.cancel());
    assert!(current.finish(BuildSessionOutcome::Cancelled));
    assert!(!current.cancel());
}

#[test]
fn lifecycle_does_not_reactivate_cancelling_or_terminal_sessions() {
    let registry = SnapshotRegistry::new();
    let mut completed = pending_session(BuildLaneId::new(11), BuildRequestGeneration::new(0), 14)
        .capture_snapshot(&registry)
        .unwrap();

    completed.mark_submitted();
    assert_eq!(completed.state(), BuildSessionState::Submitted);
    completed.mark_running();
    assert_eq!(completed.state(), BuildSessionState::Running);
    assert!(completed.finish(BuildSessionOutcome::Succeeded));
    completed.mark_submitted();
    completed.mark_running();
    assert!(!completed.cancel());
    assert!(!completed.finish(BuildSessionOutcome::Cancelled));
    assert_eq!(
        completed.state(),
        BuildSessionState::Finished(BuildSessionOutcome::Succeeded)
    );

    let mut session = pending_session(BuildLaneId::new(12), BuildRequestGeneration::new(0), 16)
        .capture_snapshot(&registry)
        .unwrap();
    assert!(session.cancel());
    session.mark_submitted();
    assert_eq!(session.state(), BuildSessionState::Cancelling);
    session.mark_running();
    assert_eq!(session.state(), BuildSessionState::Cancelling);

    assert!(session.finish(BuildSessionOutcome::Cancelled));
    assert!(!session.finish(BuildSessionOutcome::Succeeded));
    assert_eq!(
        session.state(),
        BuildSessionState::Finished(BuildSessionOutcome::Cancelled)
    );
}

fn pending_session(
    lane: BuildLaneId,
    generation: BuildRequestGeneration,
    seed: u8,
) -> mizar_driver::request::PendingBuildRequest {
    let ids = InMemorySessionIdAllocator::new();
    pending_session_with_ids(&ids, lane, generation, seed)
}

fn pending_session_with_ids(
    ids: &InMemorySessionIdAllocator,
    lane: BuildLaneId,
    generation: BuildRequestGeneration,
    seed: u8,
) -> mizar_driver::request::PendingBuildRequest {
    draft(
        lane,
        generation,
        BuildRequestOrigin::Watch(WatchRequest {
            watch_root: WorkspaceRoot::new("workspace"),
            changed_paths: vec![normalized_path("src/alpha.miz")],
        }),
        snapshot_input_parts(seed),
    )
    .allocate(ids)
    .unwrap()
}

fn draft(
    lane: BuildLaneId,
    generation: BuildRequestGeneration,
    origin: BuildRequestOrigin,
    parts: SnapshotInputParts,
) -> BuildRequestDraft {
    BuildRequestDraft {
        lane,
        origin,
        generation,
        workspace_root: parts.workspace_root,
        profile: BuildProfile::new("verify"),
        targets: BuildTargets {
            packages: vec![PackageId::new("mml")],
            modules: vec![ModulePath::new("alpha")],
        },
        source_inputs: SourceInputSet {
            versions: parts.source_versions,
        },
        dependency_inputs: DependencyInputSet::new(
            parts.dependency_artifacts,
            parts.lockfile_hash,
            parts.toolchain,
        ),
        verifier_config: VerifierConfigInput::new(parts.verifier_config_hash),
    }
}

#[derive(Debug, Clone)]
struct SnapshotInputParts {
    workspace_root: WorkspaceRoot,
    source_versions: Vec<SourceVersion>,
    dependency_artifacts: Vec<DependencyArtifactRef>,
    lockfile_hash: Hash,
    toolchain: ToolchainInfo,
    verifier_config_hash: Hash,
}

impl SnapshotInputParts {
    fn snapshot_input(self) -> mizar_session::SnapshotInput {
        mizar_session::SnapshotInput {
            workspace_root: self.workspace_root,
            source_versions: self.source_versions,
            dependency_artifacts: self.dependency_artifacts,
            lockfile_hash: self.lockfile_hash,
            toolchain: self.toolchain,
            verifier_config_hash: self.verifier_config_hash,
        }
    }
}

fn snapshot_input_parts(seed: u8) -> SnapshotInputParts {
    let allocator = InMemorySessionIdAllocator::new();
    let snapshot_for_source_ids = mizar_session::BuildSnapshotId::from_published_schema_str(
        "mizar-session-build-snapshot-v1:\
         0000000000000000000000000000000000000000000000000000000000000000",
    )
    .unwrap();
    SnapshotInputParts {
        workspace_root: WorkspaceRoot::new("workspace"),
        source_versions: vec![SourceVersion {
            source_id: allocator.next_source_id(snapshot_for_source_ids).unwrap(),
            package_id: PackageId::new("mml"),
            module_path: ModulePath::new("alpha"),
            normalized_path: normalized_path("src/alpha.miz"),
            source_hash: hash(seed),
            edition: Edition::new("2026"),
            origin: SourceOrigin::Disk,
        }],
        dependency_artifacts: vec![DependencyArtifactRef::new("kernel/base.vo", hash(90))],
        lockfile_hash: hash(91),
        toolchain: ToolchainInfo::new("mizar-2026.1"),
        verifier_config_hash: hash(92),
    }
}

fn hash(first_byte: u8) -> Hash {
    let mut bytes = [0; Hash::BYTE_LEN];
    bytes[0] = first_byte;
    Hash::from_bytes(bytes)
}

fn normalized_path(path: &str) -> NormalizedPath {
    let fixture = SourcePathFixture::new();
    fixture.write(path, "");
    normalize_source_path(fixture.root(), Path::new(path)).unwrap()
}

struct SourcePathFixture {
    base: PathBuf,
    root: PathBuf,
}

impl SourcePathFixture {
    fn new() -> Self {
        let id = NEXT_FIXTURE_ID.fetch_add(1, Ordering::Relaxed);
        let base = std::env::temp_dir().join(format!(
            "mizar_driver_request_source_path_{}_{}",
            std::process::id(),
            id
        ));
        let root = base.join("package");
        std::fs::create_dir_all(root.join("src")).unwrap();
        Self { base, root }
    }

    fn root(&self) -> &Path {
        &self.root
    }

    fn write(&self, path: &str, contents: &str) {
        let target = self.root.join(path);
        if let Some(parent) = target.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::write(target, contents).unwrap();
    }
}

impl Drop for SourcePathFixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.base);
    }
}

mod q1_comments {
    use super::*;
    use mizar_artifact::{
        comment_projection::{
            COMMENTED_ARTIFACT_SCHEMA_FAMILY, COMMENTED_ARTIFACT_SCHEMA_VERSION,
            artifact_hash_excluded_paths, read_commented_artifact, write_commented_artifact,
        },
        manifest::{
            ManifestCommit, ManifestCommitOptions, ManifestFileReadOptions, ManifestTransaction,
            read_manifest_file, write_manifest_file,
        },
        module_summary::ModuleSummaryIdentity,
        registration_summary::{ArtifactHashClass, ArtifactHashRef},
        store::{
            CanonicalJson, PublishedArtifactPath, PublishedArtifactReadOptions,
            SchemaVersion as ArtifactSchemaVersion, artifact_hash_domain,
            parse_canonical_json_bytes, read_published_artifact,
        },
        verified_artifact::{BuildProvenance, VerifiedArtifactReadOptions},
    };
    use mizar_driver::request::BuildSession;
    use mizar_frontend::{
        lexical_env::{
            FrontendLexicalEnvironmentError, LexicalEnvironmentRequest, LexicalSummaryProvider,
            ResolvedImports,
        },
        orchestration::{CommentMetadata, Frontend, FrontendOutput},
        parsing::{MizarParserSeam, ParserSeam},
        source::{FrontendSourceLoader, source_unit_from_loaded},
    };
    use mizar_ir::{
        identity::{OutputKind, PipelinePhase, SnapshotHandleRegistry, WorkUnit},
        projection::{
            ArtifactProjectionService, CommentedArtifactDraft, ProjectVerifiedArtifactInput,
        },
        publisher::{
            AllowedWorkUnit, OutputAuthorization, OutputOrigin, PhaseOutputPublisher,
            PublicationTarget, PublishOutputInput,
        },
        storage::{
            AnyPhaseOutputRef, BlobDecodeError, BlobDecoder, CollectInput, IrSideTables,
            IrStorageService, PhaseOutputRef, SchemaVersion, StoragePolicy,
        },
    };
    use mizar_session::{DiskSourceLoader, SourceInput, SourceOriginInput};
    use std::sync::{Arc, atomic::AtomicBool};

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
        files: SourcePathFixture,
        snapshots: SnapshotRegistry,
        session: BuildSession,
        publisher: Arc<PhaseOutputPublisher>,
        outputs: Vec<FrontendOutput<<MizarParserSeam as ParserSeam>::Ast>>,
        metadata: Vec<CommentMetadata>,
        readable: Arc<AtomicBool>,
        open: bool,
    }

    impl Fixture {
        fn new(text: &str, version: Option<i64>) -> Self {
            Self::sources(&[text], version)
        }

        fn sources(texts: &[&str], version: Option<i64>) -> Self {
            let files = SourcePathFixture::new();
            let sources = texts
                .iter()
                .enumerate()
                .map(|(index, text)| {
                    let relative = format!("src/module{index}.miz");
                    files.write(&relative, text);
                    let path = files.root().join(&relative);
                    let input = SourceInput {
                        package_id: PackageId::new("fixture"),
                        module_path: ModulePath::new(format!("module{index}")),
                        normalized_path: normalize_source_path(files.root(), Path::new(&relative))
                            .unwrap(),
                        edition: Edition::new("2026"),
                        origin: version.map_or_else(
                            || SourceOriginInput::Disk { path: path.clone() },
                            |version| SourceOriginInput::OpenBuffer {
                                uri: format!("file://{}", path.display()),
                                expected_version: version,
                                actual_version: version,
                                text: Arc::from(*text),
                            },
                        ),
                    };
                    (DiskSourceLoader::new(files.root()), input)
                })
                .collect();
            let allocator = InMemorySessionIdAllocator::new();
            for _ in 0..NEXT_FIXTURE_ID.fetch_add(4, Ordering::Relaxed) + 1 {
                allocator.next_source_id(foreign_snapshot()).unwrap();
            }
            let snapshots = SnapshotRegistry::with_allocator(allocator);
            let mut parts = snapshot_input_parts(0);
            parts.workspace_root = WorkspaceRoot::new(files.root().to_string_lossy());
            parts.source_versions.clear();
            parts.dependency_artifacts.clear();
            let session = draft(
                BuildLaneId::new(50),
                BuildRequestGeneration::new(0),
                BuildRequestOrigin::Batch(BatchRequest {
                    invocation: BatchInvocation::default(),
                }),
                parts,
            )
            .capture_sources(&InMemorySessionIdAllocator::new(), &snapshots, sources)
            .unwrap();
            let (outputs, metadata) = session
                .captured()
                .native_sources()
                .unwrap()
                .iter()
                .zip(texts)
                .map(|((loader, loaded, _), text)| {
                    let path = files.root().join(loaded.normalized_path.as_str());
                    let (output, metadata) = Frontend::new(
                        FrontendSourceLoader::new(loader.clone()),
                        EmptyImports,
                        MizarParserSeam,
                    )
                    .run_loaded_with_comment_metadata(
                        source_unit_from_loaded(loaded.clone(), path),
                        text,
                    )
                    .unwrap();
                    (output, metadata.unwrap())
                })
                .unzip();
            let publisher = Arc::new(PhaseOutputPublisher::new(
                Arc::new(IrStorageService::with_policy(
                    StoragePolicy::with_blob_spill_threshold(0),
                )),
                Arc::new(SnapshotHandleRegistry::new()),
            ));
            publisher.register_current_snapshot(session.captured().snapshot().id);
            Self {
                files,
                snapshots,
                session,
                publisher,
                outputs,
                metadata,
                readable: Arc::new(AtomicBool::new(true)),
                open: version.is_some(),
            }
        }

        fn publish<T: Clone + Send + Sync + 'static>(
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
            let schema = SchemaVersion::new(1);
            self.publisher.allow_work_unit(AllowedWorkUnit::new(
                phase.clone(),
                kind.clone(),
                work.clone(),
            ));
            let decoded = payload.clone();
            let readable = Arc::clone(&self.readable);
            self.publisher
                .publish(PublishOutputInput {
                    slot: self.publisher.allocate(
                        self.session.captured().snapshot().id,
                        phase.clone(),
                        work.clone(),
                        kind.clone(),
                        schema,
                    ),
                    snapshot: self.session.captured().snapshot().id,
                    phase,
                    work_unit: work,
                    output_kind: kind,
                    schema_version: schema,
                    payload,
                    storage_payload: Some(unit.as_bytes().to_vec()),
                    canonical_payload: Some(unit.as_bytes().to_vec()),
                    decode: BlobDecoder::new(move |_| {
                        if readable.load(Ordering::Relaxed) {
                            Ok(decoded.clone())
                        } else {
                            Err(BlobDecodeError::new("unreadable fixture blob"))
                        }
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

        fn frontend(
            &self,
            index: usize,
        ) -> PhaseOutputRef<FrontendOutput<<MizarParserSeam as ParserSeam>::Ast>> {
            self.publish(
                self.outputs[index].clone(),
                &format!("frontend-{index}"),
                true,
                vec![],
            )
        }

        fn input(
            &self,
            index: usize,
            required_outputs: Vec<AnyPhaseOutputRef>,
        ) -> ProjectVerifiedArtifactInput {
            let source = &self.outputs[index].source;
            let snapshot = self.session.captured().snapshot();
            let reference = |family, digest| {
                ArtifactHashRef::new(
                    ArtifactHashClass::Artifact,
                    family,
                    ArtifactSchemaVersion::new(1, 0),
                    digest,
                )
            };
            ProjectVerifiedArtifactInput {
                snapshot: snapshot.id,
                required_outputs,
                module: ModuleSummaryIdentity {
                    package_id: "fixture".into(),
                    package_version: Some("1.2.3".into()),
                    lockfile_identity: Some("lock:fixture".into()),
                    module_path: source.module_path.as_str().into(),
                    language_edition: "2026".into(),
                },
                source_file: source.normalized_path.as_str().into(),
                source_hash: source.source_hash,
                verified_at: None,
                exports: vec![],
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
                    toolchain: snapshot.toolchain.identity().into(),
                    language_edition: "2026".into(),
                    lockfile_hash: reference("mizar-build/lockfile", snapshot.lockfile_hash),
                    verifier_config_hash: ArtifactHashRef::new(
                        ArtifactHashClass::Interface,
                        "mizar-proof/policy",
                        ArtifactSchemaVersion::new(1, 0),
                        snapshot.verifier_config_hash,
                    ),
                    dependency_artifact_hashes: vec![],
                    cache_key: None,
                },
            }
        }

        fn capture(
            &mut self,
            index: usize,
            frontend: &PhaseOutputRef<FrontendOutput<<MizarParserSeam as ParserSeam>::Ast>>,
        ) {
            self.session
                .capture_commented_module(
                    &self.publisher,
                    &self.snapshots,
                    self.input(index, vec![frontend.erase()]),
                    frontend,
                    &self.metadata[index],
                    &vec![None; self.metadata[index].boundaries().len()],
                )
                .unwrap();
        }

        fn emission_root(&self) -> PathBuf {
            let root = self.files.root().join("artifacts");
            std::fs::create_dir_all(&root).unwrap();
            root
        }

        fn emit(&self, root: &Path, main: &str) -> Result<Option<ManifestCommit>, String> {
            self.session.emit_commented_module(
                &self.publisher,
                &self.snapshots,
                self.outputs[0].source.source_id,
                root,
                main,
            )
        }

        fn retained(&self, index: usize) -> &CommentedArtifactDraft {
            self.session
                .commented_module(
                    &self.publisher,
                    &self.snapshots,
                    self.outputs[index].source.source_id,
                )
                .unwrap()
                .unwrap()
        }
    }

    fn validated_manifest(root: &Path) -> mizar_artifact::manifest::PublishedManifestRead {
        read_manifest_file(
            root,
            ManifestFileReadOptions {
                validate_references: true,
                validate_witness_files: true,
                ..ManifestFileReadOptions::default()
            },
        )
        .unwrap()
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

    #[test]
    fn doc13_coordinates_genuine_commented_publication_preserves_native_bytes() {
        use mizar_artifact::direct_by::{DirectByTarget, project_frontend_direct_by};
        use mizar_doc::doc_build::*;
        use mizar_session::TextRange;

        let original = "\u{feff}::: café\r\ntheorem Named: thesis by Named, Named;\r\n";
        let mut f = Fixture::new(original, None);
        let frontend = f.frontend(0);
        f.capture(0, &frontend);
        let root = f.emission_root();
        let commit = f.emit(&root, "native.mizir.json").unwrap().unwrap();
        let publication = validated_manifest(&root);
        let artifact = &f.retained(0).artifact.artifact;
        let snapshot = f.session.captured().snapshot();
        let request = DocBuildRequest {
            profile: Phase16Profile {
                render_docs: true,
                extract_code: false,
                fail_on_doc_warnings: false,
                fail_on_extraction_warnings: false,
                include_private_docs: false,
            },
            publications: vec![DocPublicationRequest {
                package: commit.manifest().package.clone(),
                artifact_root: root.clone(),
                expected_manifest_hash: publication.artifact_hash,
                output_dir: f.files.root().join("doc-output"),
                source_bundle: None,
                dependency_indexes: None,
                import_order: Default::default(),
            }],
        };
        let plans = build_doc_plans(
            &f.snapshots,
            f.session.request().id,
            snapshot.id,
            &snapshot.workspace_root,
            &request,
        )
        .unwrap();
        let inputs =
            authenticate_doc_imports(&f.snapshots, &snapshot.workspace_root, &plans[0], &[], &[])
                .unwrap();
        let producer = project_frontend_direct_by(
            &f.outputs[0],
            artifact,
            publication.artifact_hash,
            |_| None,
            |_, _| DirectByTarget::Unmapped,
        )
        .unwrap();
        let direct_by = authenticate_doc_direct_by(&inputs, &[&producer]).unwrap();
        let before = [
            root.join("artifact-manifest.json"),
            root.join("native.mizir.json"),
            f.files.root().join("src/module0.miz"),
        ]
        .map(|path| {
            let bytes = std::fs::read(&path).unwrap();
            (path, bytes)
        });
        let retained = f.retained(0).clone();
        let sources = authenticate_doc_sources(
            &f.snapshots,
            &snapshot.workspace_root,
            &inputs,
            &[],
            &direct_by,
        )
        .unwrap();
        assert_eq!(
            sources.source_binding(&artifact.module, publication.artifact_hash),
            Some(("src/module0.miz", Some(mizar_session::hash_text(original))))
        );
        assert_eq!(
            sources.original_digest(
                &artifact.module,
                publication.artifact_hash,
                "src/module0.miz"
            ),
            Some(mizar_session::hash_text(original))
        );
        assert_eq!(
            sources
                .original_text(
                    &artifact.module,
                    publication.artifact_hash,
                    &artifact.source_file,
                    mizar_session::hash_text(original),
                )
                .unwrap(),
            Some(original)
        );
        let identity = mizar_artifact::verified_artifact::project_frontend_identity(
            &f.outputs[0],
            artifact,
            publication.artifact_hash,
        )
        .unwrap();
        let identities = authenticate_doc_identities(
            &f.snapshots,
            &snapshot.workspace_root,
            &inputs,
            &[],
            &[&identity],
        )
        .unwrap();
        let doc_dir = &request.publications[0].output_dir;
        let source_views = mizar_doc::render::DocSourceViews::allocate(
            &inputs,
            &identities,
            &sources,
            doc_dir,
            "source-view-spec",
            mizar_doc::allocations::DocAllocationTable {
                output_root: doc_dir.join("v1.2.3"),
                coverage: DocImportCoverage::Complete,
                pages: vec![],
                sections: None,
            },
        )
        .unwrap();
        let (view_path, html) = source_views
            .view(&artifact.module, publication.artifact_hash, "Fixture docs")
            .unwrap()
            .unwrap();
        assert_eq!(view_path, "sources/src/module0.miz.html");
        assert!(
            html.contains("<li id=\"L1\"><pre><code>\u{feff}::: café\\u{d}</code></pre></li>\n")
        );
        assert!(html.contains("<li id=\"L3\"><pre><code></code></pre></li>\n"));
        let loaded_start = f.outputs[0].source.source_text.find("theorem").unwrap();
        assert_eq!(
            source_views
                .loaded_source_slot(
                    &artifact.module,
                    publication.artifact_hash,
                    &[TextRange::new(loaded_start, loaded_start)],
                    "module0/index.html",
                )
                .unwrap(),
            "<p class=\"mizar-source\">Source: <a href=\"../sources/src/module0.miz.html#L2\">line 2</a></p>\n"
        );
        assert_eq!(
            source_views
                .view(&artifact.module, publication.artifact_hash, "Fixture docs")
                .unwrap()
                .unwrap()
                .1,
            html
        );
        let attachments =
            mizar_doc::comments::attach_comments(&plans[0], &artifact.module, None).unwrap();
        let parsed = mizar_doc::markdown::parse_comments(&attachments, &[]);
        assert_eq!(
            mizar_doc::render::summary_text(&parsed, mizar_doc::comments::CommentTarget::Module)
                .unwrap(),
            Some("café".into())
        );
        let comment_allocations = mizar_doc::allocations::validate_doc_allocations(
            &inputs,
            doc_dir,
            "source-view-spec",
            mizar_doc::allocations::DocAllocationTable {
                output_root: doc_dir.clone(),
                coverage: DocImportCoverage::Unavailable,
                pages: vec![],
                sections: None,
            },
        )
        .unwrap();
        let (fragments, links) = mizar_doc::render::comment_fragments(
            &parsed,
            mizar_doc::comments::CommentTarget::Module,
            &inputs,
            &direct_by,
            &comment_allocations,
            "source-view-spec",
            &[vec![]],
        )
        .unwrap();
        assert_eq!(fragments.complete.as_deref(), Some("<p>café</p>\n"));
        assert!(fragments.parameters.is_empty());
        assert!(fragments.see.is_empty());
        assert!(links.references.is_empty());
        let (templates, template_links) = mizar_doc::render::comment_template_fragments(
            &parsed,
            mizar_doc::comments::CommentTarget::Module,
            &inputs,
            &direct_by,
            &comment_allocations,
            "source-view-spec",
            &[vec![]],
        )
        .unwrap();
        assert_eq!(
            templates.unwrap(),
            [
                "<div class=\"mizar-comment\"><p>café</p>\n</div>\n",
                "<section class=\"mizar-parameters\"><h2>Parameters and returns</h2><table><tbody></tbody></table></section>\n",
                "<section class=\"mizar-see-also\"><h2>See also</h2><ul></ul></section>\n",
            ]
        );
        assert_eq!(format!("{template_links:?}"), format!("{links:?}"));
        let search = mizar_doc::render::search_summaries(&parsed).unwrap();
        let module = mizar_doc::render::module_summary_fragments(&parsed).unwrap();
        assert_eq!(search.len(), attachments.input.exports.len());
        assert_eq!(module.len(), search.len());
        for ((item, summary), (listed, html)) in search.iter().zip(&module) {
            assert!(std::ptr::eq(*item, *listed));
            assert_eq!(summary, &None);
            assert_eq!(html, "<p class=\"mizar-summary\"></p>\n");
        }
        let start = parsed.blocks[0].text.find("café").unwrap();
        let expected = original.find("café").unwrap();
        assert_eq!(
            mizar_doc::markdown::original_ranges(
                &sources,
                &parsed,
                0,
                std::slice::from_ref(&(start..start + "café".len()))
            )
            .unwrap(),
            (
                vec![TextRange::new(expected, expected + "café".len())],
                DocImportCoverage::Complete
            )
        );
        let expected_targets = original
            .match_indices("Named")
            .skip(1)
            .map(|(start, _)| TextRange::new(start, start + "Named".len()))
            .collect::<Vec<_>>();
        assert_eq!(producer.occurrences().len(), 2);
        for (ordinal, edge) in producer.occurrences().iter().enumerate() {
            assert_eq!((edge.by_ordinal, edge.target_ordinal), (0, ordinal));
            assert_eq!(
                sources
                    .citation_ranges(&producer, &[edge.target_range])
                    .unwrap(),
                (vec![expected_targets[ordinal]], DocImportCoverage::Complete)
            );
        }
        assert_eq!(direct_by.citation_coverage(), DocImportCoverage::Incomplete);
        assert_eq!(f.retained(0), &retained);
        for (path, bytes) in before {
            assert_eq!(std::fs::read(path).unwrap(), bytes);
        }
        assert!(!request.publications[0].output_dir.exists());
    }

    #[test]
    fn doc13_used_by_preserves_native_mapped_self_citations_and_offsets() {
        use mizar_artifact::direct_by::{
            DirectByEndpoint, DirectByTarget, project_frontend_direct_by,
        };
        use mizar_doc::doc_build::*;
        use mizar_session::TextRange;

        let original = "\u{feff}::: café\r\ntheorem Named: thesis by Named, Named;\r\n";
        let mut f = Fixture::new(original, None);
        let frontend = f.frontend(0);
        let mut input = f.input(0, vec![frontend.erase()]);
        let loaded = &f.outputs[0].source.source_text;
        input
            .exports
            .push(mizar_artifact::verified_artifact::VerifiedExport {
                origin_id: "Named".into(),
                fully_qualified_name: "Fixture::Named".into(),
                namespace_path: vec!["Fixture".into()],
                visibility: mizar_artifact::verified_artifact::ExportVisibility::Public,
                export_kind: "theorem".into(),
                source_range: mizar_artifact::module_summary::SourceRangeSummary {
                    start_byte: loaded.find("theorem").unwrap() as u64,
                    end_byte: loaded.len() as u64,
                },
                rendered_signature: "theorem Named: thesis".into(),
                interface_fingerprint: ArtifactHashRef::new(
                    ArtifactHashClass::Interface,
                    "mizar-ir/export",
                    ArtifactSchemaVersion::new(1, 0),
                    hash(20),
                ),
                proof_status: None,
                documentation_ref: None,
            });
        let mut targets = vec![None; f.metadata[0].boundaries().len()];
        let theorem = f.metadata[0]
            .boundaries()
            .iter()
            .position(|boundary| format!("{:?}", boundary.kind()) == "TheoremItem")
            .unwrap();
        targets[theorem] = Some(("Named".into(), "Fixture::Named".into()));
        f.session
            .capture_commented_module(
                &f.publisher,
                &f.snapshots,
                input,
                &frontend,
                &f.metadata[0],
                &targets,
            )
            .unwrap();
        let root = f.emission_root();
        let commit = f.emit(&root, "native.mizir.json").unwrap().unwrap();
        let publication = validated_manifest(&root);
        let artifact = &f.retained(0).artifact.artifact;
        let retained = f.retained(0).clone();
        let paths = [
            root.join("artifact-manifest.json"),
            root.join("native.mizir.json"),
            f.files.root().join("src/module0.miz"),
        ];
        let bytes = paths.each_ref().map(|path| std::fs::read(path).unwrap());
        let snapshot = f.session.captured().snapshot();
        let request = DocBuildRequest {
            profile: Phase16Profile {
                render_docs: true,
                extract_code: false,
                fail_on_doc_warnings: false,
                fail_on_extraction_warnings: false,
                include_private_docs: false,
            },
            publications: vec![DocPublicationRequest {
                package: commit.manifest().package.clone(),
                artifact_root: root.clone(),
                expected_manifest_hash: publication.artifact_hash,
                output_dir: f.files.root().join("doc-output"),
                source_bundle: None,
                dependency_indexes: None,
                import_order: Default::default(),
            }],
        };
        let plans = build_doc_plans(
            &f.snapshots,
            f.session.request().id,
            snapshot.id,
            &snapshot.workspace_root,
            &request,
        )
        .unwrap();
        let inputs =
            authenticate_doc_imports(&f.snapshots, &snapshot.workspace_root, &plans[0], &[], &[])
                .unwrap();
        let endpoint = DirectByEndpoint {
            module: artifact.module.clone(),
            publication_hash: publication.artifact_hash,
            export: artifact.exports[0].clone(),
        };
        let producer = project_frontend_direct_by(
            &f.outputs[0],
            artifact,
            publication.artifact_hash,
            |id| {
                let start = f.outputs[0].ast.as_ref()?.node(id)?.range.start as u64;
                (start == endpoint.export.source_range.start_byte).then(|| endpoint.export.clone())
            },
            |_, _| DirectByTarget::Theorem(Box::new(endpoint.clone())),
        )
        .unwrap();
        let direct_by = authenticate_doc_direct_by(&inputs, &[&producer]).unwrap();
        assert_eq!(direct_by.citation_coverage(), DocImportCoverage::Complete);
        let sources = authenticate_doc_sources(
            &f.snapshots,
            &snapshot.workspace_root,
            &inputs,
            &[],
            &direct_by,
        )
        .unwrap();
        let attachments =
            mizar_doc::comments::attach_comments(&plans[0], &artifact.module, None).unwrap();
        let parsed = mizar_doc::markdown::parse_comments(&attachments, &[]);
        let output = &request.publications[0].output_dir;
        let allocations = mizar_doc::allocations::validate_doc_allocations(
            &inputs,
            output,
            "used-by-spec",
            mizar_doc::allocations::DocAllocationTable {
                output_root: output.clone(),
                coverage: DocImportCoverage::Unavailable,
                pages: vec![],
                sections: None,
            },
        )
        .unwrap();
        let expected = mizar_doc::links::resolve_doc_links(
            &parsed,
            &inputs,
            &direct_by,
            &allocations,
            "used-by-spec",
            &[],
        )
        .unwrap();
        let (fragments, resolved) = mizar_doc::render::used_by_fragments(
            &parsed,
            &inputs,
            &direct_by,
            &allocations,
            "used-by-spec",
        )
        .unwrap();
        assert_eq!(
            fragments,
            vec![
                "<p>Used by: unavailable (coverage is incomplete or unavailable).</p>\n<ul>\n<li>Fixture::Named</li>\n</ul>\n"
            ]
        );
        assert_eq!(resolved.references, expected.references);
        assert_eq!(resolved.used_by, expected.used_by);
        assert_eq!(resolved.warnings, expected.warnings);
        let (templates, template_links) = mizar_doc::render::used_by_template_fragments(
            &parsed,
            &inputs,
            &direct_by,
            &allocations,
            "used-by-spec",
        )
        .unwrap();
        assert_eq!(
            templates,
            [
                "<section class=\"mizar-used-by\"><h2>Used by</h2><p>Used by: unavailable (coverage is incomplete or unavailable).</p>\n<ul>\n<li>Fixture::Named</li>\n</ul>\n</section>\n"
            ]
        );
        assert_eq!(format!("{template_links:?}"), format!("{resolved:?}"));
        assert_eq!(
            resolved.documentation_failed(),
            expected.documentation_failed()
        );
        let used = &resolved.used_by[0];
        assert_eq!(used.citing, vec![used.target.clone()]);
        assert_eq!(used.target.export, endpoint.export);
        assert_eq!(used.occurrences, producer.occurrences());
        assert_eq!(used.occurrences.len(), 2);
        let offsets = original.match_indices("Named").skip(1);
        for (ordinal, (edge, (start, _))) in used.occurrences.iter().zip(offsets).enumerate() {
            assert_eq!((edge.by_ordinal, edge.target_ordinal), (0, ordinal));
            assert_eq!(edge.citing.as_ref(), Some(&endpoint));
            assert_eq!(
                edge.target,
                DirectByTarget::Theorem(Box::new(endpoint.clone()))
            );
            assert_eq!(
                sources
                    .citation_ranges(&producer, &[edge.target_range])
                    .unwrap(),
                (
                    vec![TextRange::new(start, start + "Named".len())],
                    DocImportCoverage::Complete
                )
            );
        }
        assert_eq!(f.retained(0), &retained);
        for (path, before) in paths.iter().zip(bytes) {
            assert_eq!(std::fs::read(path).unwrap(), before);
        }
        assert!(!output.exists());
    }

    #[test]
    fn q1_emission_disk_keeps_exact_draft_after_file_removal_and_clone() {
        let mut f = Fixture::new(TEXT, None);
        let frontend = f.frontend(0);
        let child = f.publish("verified", "verified", false, vec![frontend.erase()]);
        let mut input = f.input(0, vec![child.erase(), frontend.erase()]);
        input
            .exports
            .push(mizar_artifact::verified_artifact::VerifiedExport {
                origin_id: "Named".into(),
                fully_qualified_name: "Fixture::Named".into(),
                namespace_path: vec!["Fixture".into()],
                visibility: mizar_artifact::verified_artifact::ExportVisibility::Public,
                export_kind: "theorem".into(),
                source_range: mizar_artifact::module_summary::SourceRangeSummary {
                    start_byte: 0,
                    end_byte: f.outputs[0].source.source_text.len() as u64,
                },
                rendered_signature: "theorem Named: thesis".into(),
                interface_fingerprint: ArtifactHashRef::new(
                    ArtifactHashClass::Interface,
                    "mizar-ir/export",
                    ArtifactSchemaVersion::new(1, 0),
                    hash(20),
                ),
                proof_status: None,
                documentation_ref: None,
            });
        let mut targets = vec![None; f.metadata[0].boundaries().len()];
        let index = f.metadata[0]
            .boundaries()
            .iter()
            .position(|boundary| format!("{:?}", boundary.kind()) == "TheoremItem")
            .unwrap();
        targets[index] = Some(("Named".into(), "Fixture::Named".into()));
        let mut expected_input = f.input(0, vec![child.erase(), frontend.erase()]);
        expected_input.exports = input.exports.clone();
        let expected = ArtifactProjectionService::new(Arc::clone(&f.publisher))
            .project_commented_module(
                expected_input,
                &f.snapshots,
                &frontend,
                &f.metadata[0],
                &targets,
            )
            .unwrap();
        f.session
            .capture_commented_module(
                &f.publisher,
                &f.snapshots,
                input,
                &frontend,
                &f.metadata[0],
                &targets,
            )
            .unwrap();
        assert_eq!(f.retained(0), &expected);
        assert_eq!(
            rows(&rows(field(&expected.artifact.comments, "landmarks"))[index])[4],
            CanonicalJson::array([
                CanonicalJson::string("Named"),
                CanonicalJson::string("Fixture::Named")
            ])
        );
        assert!(
            rows(field(&expected.artifact.comments, "landmarks"))
                .iter()
                .any(|row| rows(row)[4] == CanonicalJson::Null)
        );
        f.files.write("src/module0.miz", "changed");
        assert_eq!(f.retained(0), &expected);
        std::fs::remove_file(f.files.root().join("src/module0.miz")).unwrap();
        let cloned = f.session.clone();
        let retained = cloned
            .commented_module(&f.publisher, &f.snapshots, f.outputs[0].source.source_id)
            .unwrap()
            .unwrap();
        assert!(std::ptr::eq(f.retained(0), retained));
        assert_eq!(retained, &expected);
        assert_eq!(*f.publisher.storage().get(&frontend).unwrap(), f.outputs[0]);
        assert!(!retained.external_dependency_gaps.is_empty());
        let root = f.emission_root();
        let commit = cloned
            .emit_commented_module(
                &f.publisher,
                &f.snapshots,
                f.outputs[0].source.source_id,
                &root,
                "module.mizir.json",
            )
            .unwrap()
            .unwrap();
        let entry = &commit.manifest().modules[0];
        assert_eq!(
            std::fs::read(root.join(&entry.artifact_file)).unwrap(),
            write_commented_artifact(&expected.artifact).unwrap()
        );
        let domain = artifact_hash_domain(
            COMMENTED_ARTIFACT_SCHEMA_FAMILY,
            COMMENTED_ARTIFACT_SCHEMA_VERSION,
        );
        let stored = read_published_artifact(
            &root,
            &PublishedArtifactPath::new(&entry.artifact_file).unwrap(),
            PublishedArtifactReadOptions {
                artifact_hash_domain: Some(&domain),
                hash_excluded_paths: &artifact_hash_excluded_paths(),
                expected_artifact_hash: Some(entry.artifact_hash.digest),
            },
        )
        .unwrap();
        assert_eq!(
            read_commented_artifact(
                &stored.value,
                VerifiedArtifactReadOptions {
                    expected_interface_hash: Some(entry.interface_hash.digest),
                    expected_implementation_hash: Some(entry.implementation_hash.digest),
                    ..VerifiedArtifactReadOptions::default()
                }
            )
            .unwrap(),
            expected.artifact
        );
        assert_eq!(
            stored.artifact_hash,
            Some(expected.artifact.compute_artifact_hash().unwrap())
        );
        for (reference, class) in [
            (&entry.artifact_hash, ArtifactHashClass::Artifact),
            (&entry.interface_hash, ArtifactHashClass::Interface),
            (
                &entry.implementation_hash,
                ArtifactHashClass::Implementation,
            ),
        ] {
            assert_eq!(reference.class, class);
            assert_eq!(reference.schema_family, COMMENTED_ARTIFACT_SCHEMA_FAMILY);
            assert_eq!(reference.schema_version, COMMENTED_ARTIFACT_SCHEMA_VERSION);
        }
        let nested = &expected.artifact.artifact;
        assert_eq!(entry.module, nested.module);
        assert_eq!(entry.source_file, nested.source_file);
        assert_eq!(entry.source_hash, nested.source_hash);
        assert_eq!(
            commit.manifest().package.package_id,
            nested.module.package_id
        );
        assert_eq!(
            commit.manifest().package.package_version,
            nested.module.package_version
        );
        assert_eq!(
            commit.manifest().package.lockfile_identity,
            nested.module.lockfile_identity
        );
        assert_eq!(commit.manifest().artifact_root, "artifacts");
        assert_eq!(commit.manifest().toolchain, nested.provenance.toolchain);
        assert_eq!(
            commit.manifest().language_edition,
            nested.provenance.language_edition
        );
        assert_eq!(
            commit.manifest().lockfile_hash,
            nested.provenance.lockfile_hash
        );
        assert_eq!(
            commit.manifest().verifier_config_hash,
            nested.provenance.verifier_config_hash
        );
        assert!(entry.proof_witnesses.is_empty());
        assert!(
            entry.module_summary_file.is_none()
                && entry.module_summary_hash.is_none()
                && entry.module_summary_interface_hash.is_none()
        );
        assert!(
            entry.registration_summary_file.is_none()
                && entry.registration_summary_hash.is_none()
                && entry.registration_interface_hash.is_none()
                && entry.diagnostics_hash.is_none()
        );
        assert_eq!(validated_manifest(&root).manifest, *commit.manifest());
        assert_eq!(f.retained(0), &expected);
    }

    #[test]
    fn q1_open_buffer_retains_original_frames_and_internal_only_version() {
        let text = "\u{feff}::: Ω\r\n:: ordinary\r\ntheorem::= β =::Named: thesis;\r\n";
        let mut f = Fixture::new(text, Some(7));
        let frontend = f.frontend(0);
        f.capture(0, &frontend);
        let draft = f.retained(0);
        assert_eq!(
            field(&draft.artifact.comments, "original"),
            &CanonicalJson::string(text)
        );
        assert_eq!(
            field(&draft.artifact.comments, "loaded"),
            &CanonicalJson::string(text.trim_start_matches('\u{feff}').replace("\r\n", "\n"))
        );
        assert_eq!(
            rows(field(&draft.artifact.comments, "loading")).len(),
            f.metadata[0].loading_segments().len()
        );
        assert_eq!(
            rows(field(&draft.artifact.comments, "preprocess")).len(),
            f.metadata[0].preprocess_map().segments.len()
        );
        let raw: Vec<_> = rows(field(&draft.artifact.comments, "comments"))
            .iter()
            .map(|row| rows(row)[2].clone())
            .collect();
        assert_eq!(
            raw,
            ["::: Ω\n", ":: ordinary\n", "::= β =::"].map(CanonicalJson::string)
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
                    draft.snapshot,
                    frontend.any(),
                    OutputAuthorization::CurrentPackage
                )
                .is_err()
        );
        let mut foreign = f.outputs[0].clone();
        foreign.source.origin = SourceOrigin::OpenBuffer { version: 8 };
        let bad = f.publish(foreign, "foreign-version", true, vec![]);
        let before = f.retained(0).clone();
        assert!(
            f.session
                .capture_commented_module(
                    &f.publisher,
                    &f.snapshots,
                    f.input(0, vec![bad.erase()]),
                    &bad,
                    &f.metadata[0],
                    &vec![None; f.metadata[0].boundaries().len()]
                )
                .is_err()
        );
        assert_eq!(f.retained(0), &before);
    }

    #[test]
    fn q1_empty_absent_and_doc_entries_remain_distinct_and_replace_source_locally() {
        let mut f = Fixture::sources(&["", ":::"], None);
        let empty = f.frontend(0);
        let doc = f.frontend(1);
        assert!(
            f.session
                .commented_module(&f.publisher, &f.snapshots, f.outputs[0].source.source_id)
                .unwrap()
                .is_none()
        );
        f.capture(0, &empty);
        assert!(rows(field(&f.retained(0).artifact.comments, "comments")).is_empty());
        assert!(
            f.session
                .commented_module(&f.publisher, &f.snapshots, f.outputs[1].source.source_id)
                .unwrap()
                .is_none()
        );
        f.capture(1, &doc);
        assert_eq!(
            rows(field(&f.retained(1).artifact.comments, "comments")).len(),
            1
        );
        let other = f.retained(1).clone();
        let old_session = f.session.clone();
        let child = f.publish("verified", "replacement", false, vec![empty.erase()]);
        f.session
            .capture_commented_module(
                &f.publisher,
                &f.snapshots,
                f.input(0, vec![empty.erase(), child.erase()]),
                &empty,
                &f.metadata[0],
                &[],
            )
            .unwrap();
        assert_eq!(f.retained(0).required_outputs.len(), 2);
        assert_eq!(
            old_session
                .commented_module(&f.publisher, &f.snapshots, f.outputs[0].source.source_id)
                .unwrap()
                .unwrap()
                .required_outputs
                .len(),
            1
        );
        assert_eq!(f.retained(1), &other);
        let before = f.retained(0).clone();
        let mut bad = f.input(0, vec![empty.erase()]);
        bad.source_hash = hash(199);
        assert!(
            f.session
                .capture_commented_module(
                    &f.publisher,
                    &f.snapshots,
                    bad,
                    &empty,
                    &f.metadata[0],
                    &[]
                )
                .is_err()
        );
        assert_eq!(f.retained(0), &before);
        assert_eq!(f.retained(1), &other);
    }

    #[test]
    fn q1_capture_rejects_detached_native_evidence_without_partial_retention() {
        let mut f = Fixture::new(TEXT, None);
        let frontend = f.frontend(0);
        let targets = vec![None; f.metadata[0].boundaries().len()];
        let mut summary = (*f.session.request()).clone();
        let ids = InMemorySessionIdAllocator::new();
        let mut parts = snapshot_input_parts(0);
        parts.workspace_root = summary.workspace_root.clone();
        parts.source_versions = std::mem::take(&mut summary.source_inputs.versions);
        parts.dependency_artifacts = summary.dependency_inputs.artifacts;
        parts.lockfile_hash = summary.dependency_inputs.lockfile_hash;
        parts.toolchain = summary.dependency_inputs.toolchain;
        parts.verifier_config_hash = summary.verifier_config.hash;
        let mut summary = draft(
            BuildLaneId::new(51),
            BuildRequestGeneration::new(0),
            BuildRequestOrigin::Batch(BatchRequest {
                invocation: BatchInvocation::default(),
            }),
            parts,
        )
        .allocate(&ids)
        .unwrap()
        .capture_snapshot(&f.snapshots)
        .unwrap();
        assert!(
            summary
                .capture_commented_module(
                    &f.publisher,
                    &f.snapshots,
                    f.input(0, vec![frontend.erase()]),
                    &frontend,
                    &f.metadata[0],
                    &targets
                )
                .is_err()
        );
        assert!(
            summary
                .commented_module(&f.publisher, &f.snapshots, f.outputs[0].source.source_id)
                .unwrap()
                .is_none()
        );
        assert!(
            f.session
                .capture_commented_module(
                    &f.publisher,
                    &SnapshotRegistry::new(),
                    f.input(0, vec![frontend.erase()]),
                    &frontend,
                    &f.metadata[0],
                    &targets
                )
                .is_err()
        );
        let foreign = Fixture::new(TEXT, None);
        let foreign_handle = foreign.frontend(0);
        assert!(
            f.session
                .capture_commented_module(
                    &foreign.publisher,
                    &foreign.snapshots,
                    foreign.input(0, vec![foreign_handle.erase()]),
                    &foreign_handle,
                    &foreign.metadata[0],
                    &targets
                )
                .is_err()
        );
        for variant in 0..5 {
            let mut output = f.outputs[0].clone();
            match variant {
                0 => output.source.source_id = foreign.outputs[0].source.source_id,
                1 => output.source.source_text = Arc::from("theorem Other: thesis;"),
                2 => {
                    output.source.line_map =
                        mizar_session::LineMap::new(output.source.source_id, "different\nframe")
                }
                3 => output.source.origin = SourceOrigin::OpenBuffer { version: 1 },
                _ => output.source.file_path = f.files.root().join("wrong.miz"),
            }
            let bad = f.publish(output, &format!("bad-{variant}"), true, vec![]);
            assert!(
                f.session
                    .capture_commented_module(
                        &f.publisher,
                        &f.snapshots,
                        f.input(0, vec![bad.erase()]),
                        &bad,
                        &f.metadata[0],
                        &targets
                    )
                    .is_err(),
                "native substitution {variant}"
            );
        }
        let unrelated = f.publish("unrelated", "unrelated", false, vec![]);
        assert!(
            f.session
                .capture_commented_module(
                    &f.publisher,
                    &f.snapshots,
                    f.input(0, vec![frontend.erase(), unrelated.erase()]),
                    &frontend,
                    &f.metadata[0],
                    &targets
                )
                .is_err()
        );
        let mut bad_targets = targets.clone();
        bad_targets[0] = Some(("missing".into(), "Missing::Export".into()));
        assert!(
            f.session
                .capture_commented_module(
                    &f.publisher,
                    &f.snapshots,
                    f.input(0, vec![frontend.erase()]),
                    &frontend,
                    &f.metadata[0],
                    &bad_targets
                )
                .is_err()
        );
        assert!(
            f.session
                .commented_module(&f.publisher, &f.snapshots, f.outputs[0].source.source_id)
                .unwrap()
                .is_none()
        );
        assert_eq!(*f.publisher.storage().get(&frontend).unwrap(), f.outputs[0]);
    }

    #[test]
    fn q1_emission_refuses_stale_unreadable_collected_and_cancelled() {
        for mode in [
            "obsolete",
            "session-obsolete",
            "unpublished",
            "unreadable",
            "collected",
            "collected-child",
            "resealed-child",
            "cancelled",
        ] {
            let mut f = Fixture::new(TEXT, None);
            let frontend = f.frontend(0);
            let child = f.publish("verified", "verified", false, vec![frontend.erase()]);
            f.session
                .capture_commented_module(
                    &f.publisher,
                    &f.snapshots,
                    f.input(0, vec![frontend.erase(), child.erase()]),
                    &frontend,
                    &f.metadata[0],
                    &vec![None; f.metadata[0].boundaries().len()],
                )
                .unwrap();
            let source = f.outputs[0].source.source_id;
            let snapshot = f.session.captured().snapshot().id;
            let root = f.emission_root();
            f.emit(&root, "published.mizir.json").unwrap().unwrap();
            let manifest_before = std::fs::read(root.join("artifact-manifest.json")).unwrap();
            let main_before = std::fs::read(root.join("published.mizir.json")).unwrap();
            match mode {
                "obsolete" => f.publisher.mark_obsolete(snapshot).unwrap(),
                "session-obsolete" => {
                    let mut input = f.session.request().snapshot_input();
                    input.verifier_config_hash = hash(201);
                    f.snapshots
                        .create_snapshot(f.session.request().id, input)
                        .unwrap();
                    assert!(
                        !f.snapshots
                            .is_current_for_request(snapshot, f.session.request().id)
                    );
                    f.publisher.validate_current_snapshot(snapshot).unwrap();
                }
                "unreadable" => f.readable.store(false, Ordering::Relaxed),
                "collected" => {
                    assert_eq!(
                        f.publisher
                            .storage()
                            .collect(CollectInput {
                                snapshot,
                                protected_outputs: vec![]
                            })
                            .outputs_dropped,
                        2
                    );
                    let resealed = f.frontend(0);
                    assert_eq!(frontend.output(), resealed.output());
                    assert_ne!(frontend.any().generation(), resealed.any().generation());
                }
                "collected-child" | "resealed-child" => {
                    assert_eq!(
                        f.publisher
                            .storage()
                            .collect(CollectInput {
                                snapshot,
                                protected_outputs: vec![frontend.output()]
                            })
                            .outputs_dropped,
                        1
                    );
                    assert_eq!(*f.publisher.storage().get(&frontend).unwrap(), f.outputs[0]);
                    if mode == "resealed-child" {
                        let resealed =
                            f.publish("verified", "verified", false, vec![frontend.erase()]);
                        assert_eq!(child.output(), resealed.output());
                        assert_ne!(child.any().generation(), resealed.any().generation());
                        f.publisher
                            .validate_output(
                                snapshot,
                                resealed.any(),
                                OutputAuthorization::CurrentPackage,
                            )
                            .unwrap();
                    }
                }
                "cancelled" => {
                    assert!(f.session.cancel());
                }
                _ => {}
            }
            let unpublished = PhaseOutputPublisher::new(
                Arc::clone(f.publisher.storage()),
                Arc::clone(f.publisher.registry()),
            );
            unpublished.register_current_snapshot(snapshot);
            let publisher = if mode == "unpublished" {
                &unpublished
            } else {
                &*f.publisher
            };
            assert!(
                f.session
                    .commented_module(publisher, &f.snapshots, source)
                    .is_err(),
                "{mode}"
            );
            assert!(
                f.session
                    .emit_commented_module(
                        publisher,
                        &f.snapshots,
                        source,
                        &root,
                        "refused.mizir.json",
                    )
                    .is_err(),
                "{mode}"
            );
            assert!(!root.join("refused.mizir.json").exists(), "{mode}");
            assert_eq!(
                std::fs::read(root.join("artifact-manifest.json")).unwrap(),
                manifest_before
            );
            assert_eq!(
                std::fs::read(root.join("published.mizir.json")).unwrap(),
                main_before
            );
            let missing = InMemorySessionIdAllocator::new()
                .next_source_id(snapshot)
                .unwrap();
            assert_ne!(missing, source);
            assert!(
                matches!(
                    f.session.commented_module(publisher, &f.snapshots, missing),
                    Ok(None) | Err(_)
                ),
                "{mode}"
            );
        }
    }

    fn foreign_snapshot() -> mizar_session::BuildSnapshotId {
        mizar_session::BuildSnapshotId::from_published_schema_str("mizar-session-build-snapshot-v1:ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff").unwrap()
    }

    #[test]
    fn q1_comment_edits_remain_source_bound_deterministic_and_preserve_legacy_projection() {
        let mut a = Fixture::new(":: short\ntheorem Named: thesis;\n", None);
        let mut b = Fixture::new(":: longer body\ntheorem Named: thesis;\n", None);
        assert_eq!(
            a.outputs[0].preprocessed.lexical_hash,
            b.outputs[0].preprocessed.lexical_hash
        );
        let frontend_a = a.frontend(0);
        let frontend_b = b.frontend(0);
        let service = ArtifactProjectionService::new(Arc::clone(&a.publisher));
        let legacy = service
            .project_module(a.input(0, vec![frontend_a.erase()]))
            .unwrap();
        a.capture(0, &frontend_a);
        b.capture(0, &frontend_b);
        assert_ne!(a.outputs[0].source.source_id, b.outputs[0].source.source_id);
        assert_ne!(
            a.retained(0).artifact.comments,
            b.retained(0).artifact.comments
        );
        assert_ne!(
            a.retained(0).artifact.compute_artifact_hash().unwrap(),
            b.retained(0).artifact.compute_artifact_hash().unwrap()
        );
        assert!(
            b.session
                .commented_module(&b.publisher, &b.snapshots, a.outputs[0].source.source_id)
                .unwrap()
                .is_none()
        );
        let first = a.retained(0).clone();
        a.capture(0, &frontend_a);
        assert_eq!(a.retained(0), &first);
        assert_eq!(
            service
                .project_module(a.input(0, vec![frontend_a.erase()]))
                .unwrap(),
            legacy
        );
        a.publisher
            .validate_output(
                first.snapshot,
                frontend_a.any(),
                OutputAuthorization::CurrentPackage,
            )
            .unwrap();
        let mut equivalent = Fixture::new(":: short\ntheorem Named: thesis;\n", None);
        let equivalent_frontend = equivalent.frontend(0);
        equivalent.capture(0, &equivalent_frontend);
        assert_eq!(
            first.artifact.comments,
            equivalent.retained(0).artifact.comments
        );
        assert_eq!(
            first.artifact.compute_artifact_hash().unwrap(),
            equivalent
                .retained(0)
                .artifact
                .compute_artifact_hash()
                .unwrap()
        );
    }
    #[test]
    fn q1_emission_empty_doc_determinism_and_comment_hashes() {
        let mut empty = Fixture::new("", None);
        let mut doc = Fixture::new(":::", None);
        for f in [&mut empty, &mut doc] {
            let frontend = f.frontend(0);
            f.capture(0, &frontend);
            let before = f.retained(0).clone();
            let root = f.emission_root();
            let first = f.emit(&root, "first.mizir.json").unwrap().unwrap();
            f.capture(0, &frontend);
            let second = f.emit(&root, "second.mizir.json").unwrap().unwrap();
            assert_eq!(
                std::fs::read(root.join("first.mizir.json")).unwrap(),
                std::fs::read(root.join("second.mizir.json")).unwrap()
            );
            assert_eq!(
                first.manifest().modules[0].artifact_hash,
                second.manifest().modules[0].artifact_hash
            );
            assert_eq!(
                first.manifest().modules[0].interface_hash,
                second.manifest().modules[0].interface_hash
            );
            assert_eq!(
                first.manifest().modules[0].implementation_hash,
                second.manifest().modules[0].implementation_hash
            );
            assert_eq!(f.retained(0), &before);
            assert_eq!(validated_manifest(&root).manifest, *second.manifest());
        }
        assert!(rows(field(&empty.retained(0).artifact.comments, "comments")).is_empty());
        let doc_rows = rows(field(&doc.retained(0).artifact.comments, "comments"));
        assert_eq!(doc_rows.len(), 1);
        assert_ne!(
            doc.retained(0).artifact.comments,
            empty.retained(0).artifact.comments
        );
        let mut entries = vec![];
        for text in [
            ":: short\ntheorem Named: thesis;\n",
            ":: longer body\ntheorem Named: thesis;\n",
        ] {
            let mut f = Fixture::new(text, None);
            let frontend = f.frontend(0);
            f.capture(0, &frontend);
            entries.push(
                f.emit(&f.emission_root(), "module.mizir.json")
                    .unwrap()
                    .unwrap()
                    .manifest()
                    .modules[0]
                    .clone(),
            );
        }
        assert_eq!(entries[0].interface_hash, entries[1].interface_hash);
        assert_ne!(
            entries[0].implementation_hash,
            entries[1].implementation_hash
        );
        assert_ne!(entries[0].artifact_hash, entries[1].artifact_hash);
    }

    #[test]
    fn q1_emission_missing_open_buffer_and_internal_only_write_nothing() {
        let mut f = Fixture::new(TEXT, None);
        let root = f.emission_root();
        assert!(f.emit(&root, "missing.mizir.json").unwrap().is_none());
        assert_eq!(std::fs::read_dir(&root).unwrap().count(), 0);
        let frontend = f.frontend(0);
        f.open = true;
        let internal = f.publish("verified", "internal", false, vec![frontend.erase()]);
        f.open = false;
        f.session
            .capture_commented_module(
                &f.publisher,
                &f.snapshots,
                f.input(0, vec![frontend.erase(), internal.erase()]),
                &frontend,
                &f.metadata[0],
                &vec![None; f.metadata[0].boundaries().len()],
            )
            .unwrap();
        f.publisher
            .validate_output(
                f.retained(0).snapshot,
                frontend.any(),
                OutputAuthorization::CurrentPackage,
            )
            .unwrap();
        assert!(
            f.publisher
                .validate_output(
                    f.retained(0).snapshot,
                    internal.any(),
                    OutputAuthorization::CurrentPackage
                )
                .is_err()
        );
        assert!(f.emit(&root, "internal.mizir.json").is_err());
        assert_eq!(std::fs::read_dir(&root).unwrap().count(), 0);
        let mut open = Fixture::new(TEXT, Some(7));
        let frontend = open.frontend(0);
        open.capture(0, &frontend);
        let root = open.emission_root();
        let before = open.retained(0).clone();
        assert!(open.emit(&root, "open.mizir.json").is_err());
        assert_eq!(std::fs::read_dir(&root).unwrap().count(), 0);
        assert_eq!(open.retained(0), &before);
    }

    #[test]
    fn q1_emission_invalid_paths_seed_base_and_witness_references_refuse() {
        let mut f = Fixture::new(TEXT, None);
        let frontend = f.frontend(0);
        f.capture(0, &frontend);
        let root = f.emission_root();
        let outside = SourcePathFixture::new();
        for bad_root in [
            f.files.root().to_path_buf(),
            f.files.root().join("missing"),
            f.files.root().join("src/module0.miz"),
            outside.root().to_path_buf(),
        ] {
            assert!(
                f.emit(&bad_root, "module.mizir.json").is_err(),
                "{}",
                bad_root.display()
            );
        }
        for main in [
            "../escape.mizir.json",
            "/absolute.mizir.json",
            "bad.json",
            "artifact-manifest.json",
            "nested/../escape.mizir.json",
            "bad\\path.mizir.json",
        ] {
            assert!(f.emit(&root, main).is_err(), "{main}");
        }
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(outside.root(), root.join("escape")).unwrap();
            assert!(f.emit(&root, "escape/module.mizir.json").is_err());
            assert!(f.emit(&root.join("escape"), "module.mizir.json").is_err());
        }
        std::fs::write(root.join("existing.mizir.json"), b"untouched").unwrap();
        assert!(f.emit(&root, "existing.mizir.json").is_err());
        assert_eq!(
            std::fs::read(root.join("existing.mizir.json")).unwrap(),
            b"untouched"
        );
        assert!(!root.join("artifact-manifest.json").exists());
        assert!(!outside.root().join("module.mizir.json").exists());
        let commit = f.emit(&root, "published.mizir.json").unwrap().unwrap();
        let before = std::fs::read(root.join("artifact-manifest.json")).unwrap();
        let main_before = std::fs::read(root.join("published.mizir.json")).unwrap();
        let mut conflict = commit.manifest().clone();
        conflict.package.package_version = Some("conflicting".into());
        write_manifest_file(&root, &conflict).unwrap();
        let conflicting_bytes = std::fs::read(root.join("artifact-manifest.json")).unwrap();
        assert!(f.emit(&root, "seed.mizir.json").is_err());
        assert!(!root.join("seed.mizir.json").exists());
        assert_eq!(
            std::fs::read(root.join("artifact-manifest.json")).unwrap(),
            conflicting_bytes
        );
        std::fs::write(root.join("artifact-manifest.json"), &before).unwrap();
        let mut stale = ManifestTransaction::begin(&root, commit.manifest().clone(), None).unwrap();
        stale
            .stage_module(commit.manifest().modules[0].clone())
            .unwrap();
        let mut intervening = commit.manifest().clone();
        intervening.provenance.generated_by.push_str("-intervening");
        write_manifest_file(&root, &intervening).unwrap();
        let intervening_bytes = std::fs::read(root.join("artifact-manifest.json")).unwrap();
        assert!(matches!(
            stale.commit(ManifestCommitOptions::default()),
            Err(mizar_artifact::manifest::ManifestError::BaseManifestHashMismatch { .. })
        ));
        assert_eq!(
            std::fs::read(root.join("artifact-manifest.json")).unwrap(),
            intervening_bytes
        );
        std::fs::write(root.join("artifact-manifest.json"), &before).unwrap();
        let mut input = f.input(0, vec![frontend.erase()]);
        let interface = |family| {
            ArtifactHashRef::new(
                ArtifactHashClass::Interface,
                family,
                ArtifactSchemaVersion::new(1, 0),
                hash(30),
            )
        };
        let policy = input.provenance.verifier_config_hash.clone();
        input
            .obligations
            .push(mizar_artifact::verified_artifact::ObligationMetadata {
                obligation_id: "obligation".into(),
                obligation_anchor: None,
                owner_origin_id: None,
                source_range: mizar_artifact::module_summary::SourceRangeSummary {
                    start_byte: 0,
                    end_byte: 1,
                },
                obligation_kind: "theorem_body".into(),
                statement_summary: "thesis".into(),
                obligation_fingerprint: interface("mizar-vc/obligation"),
                vc_fingerprint: interface("mizar-vc/vc"),
                local_context_fingerprint: interface("mizar-vc/local-context"),
                dependency_slice_fingerprint: interface("mizar-vc/dependency-slice"),
                verifier_policy_fingerprint: policy.clone(),
                status: mizar_artifact::verified_artifact::ObligationStatus::Accepted,
                accepted_witness_obligation_id: Some("obligation".into()),
                deterministic_discharge_hash: None,
                diagnostic_ref: None,
            });
        input
            .proof_witnesses
            .push(mizar_artifact::proof_witness::ProofWitnessRef {
                schema_version: mizar_artifact::proof_witness::current_schema_version(),
                obligation_id: "obligation".into(),
                obligation_fingerprint: interface("mizar-vc/obligation"),
                proof_status: mizar_artifact::proof_witness::ProofStatus::KernelVerified,
                evidence_kind:
                    mizar_artifact::proof_witness::EvidenceKind::FormulaSubstitutionKernelEvidence,
                witness_path: "proof-witnesses/witness.json".into(),
                witness_artifact_hash: ArtifactHashRef::new(
                    ArtifactHashClass::Artifact,
                    "mizar-kernel/formula-evidence-witness",
                    ArtifactSchemaVersion::new(1, 0),
                    hash(31),
                ),
                kernel_acceptance: mizar_artifact::proof_witness::KernelAcceptanceMetadata {
                    kernel_profile_fingerprint: interface("mizar-kernel/profile"),
                    verifier_policy_fingerprint: policy,
                    checker_schema_version: ArtifactSchemaVersion::new(1, 0),
                    evidence_schema_version: ArtifactSchemaVersion::new(1, 0),
                    target_binding_hash: interface("mizar-vc/kernel-target-binding"),
                    formula_evidence_hash: interface("mizar-kernel/formula-evidence"),
                    substitution_evidence_hash: interface("mizar-kernel/substitution-evidence"),
                    provenance_hash: interface("mizar-kernel/evidence-provenance"),
                    formula_context_hash: None,
                    accepted_result_hash: interface("mizar-kernel/accepted-result"),
                },
            });
        f.session
            .capture_commented_module(
                &f.publisher,
                &f.snapshots,
                input,
                &frontend,
                &f.metadata[0],
                &vec![None; f.metadata[0].boundaries().len()],
            )
            .unwrap();
        let retained = f.retained(0).clone();
        assert!(f.emit(&root, "orphan.mizir.json").is_err());
        assert_eq!(
            std::fs::read(root.join("artifact-manifest.json")).unwrap(),
            before
        );
        assert_eq!(
            std::fs::read(root.join("published.mizir.json")).unwrap(),
            main_before
        );
        assert_eq!(
            validated_manifest(&root).manifest.modules[0].artifact_file,
            "published.mizir.json"
        );
        std::fs::create_dir(root.join("proof-witnesses")).unwrap();
        std::fs::write(
            root.join("proof-witnesses/witness.json"),
            b"external lower-owner witness fixture",
        )
        .unwrap();
        let witness_commit = f.emit(&root, "witnessed.mizir.json").unwrap().unwrap();
        let reference = &retained.artifact.artifact.proof_witnesses[0];
        let published = &witness_commit.manifest().modules[0].proof_witnesses[0];
        assert_eq!(published.obligation_id, reference.obligation_id);
        assert_eq!(
            published.obligation_fingerprint,
            reference.obligation_fingerprint
        );
        assert_eq!(published.witness_path, reference.witness_path);
        assert_eq!(
            published.witness_artifact_hash,
            reference.witness_artifact_hash
        );
        let mut tampered = witness_commit.manifest().clone();
        tampered.modules[0].proof_witnesses[0]
            .witness_artifact_hash
            .digest = hash(99);
        write_manifest_file(&root, &tampered).unwrap();
        assert!(
            read_manifest_file(
                &root,
                ManifestFileReadOptions {
                    validate_references: true,
                    validate_witness_files: true,
                    ..ManifestFileReadOptions::default()
                }
            )
            .is_err()
        );
        assert_eq!(f.retained(0), &retained);

        let mut concurrent = Fixture::sources(&[TEXT, "::: other\ntheorem Other: thesis;\n"], None);
        for index in 0..2 {
            let frontend = concurrent.frontend(index);
            concurrent.capture(index, &frontend);
        }
        let root = concurrent.emission_root();
        let barrier = std::sync::Barrier::new(2);
        let outcomes = std::thread::scope(|scope| {
            let writers = (0..2)
                .map(|index| {
                    let f = &concurrent;
                    let root = &root;
                    let barrier = &barrier;
                    scope.spawn(move || {
                        barrier.wait();
                        f.session.emit_commented_module(
                            &f.publisher,
                            &f.snapshots,
                            f.outputs[index].source.source_id,
                            root,
                            "shared.mizir.json",
                        )
                    })
                })
                .collect::<Vec<_>>();
            writers
                .into_iter()
                .map(|writer| writer.join().unwrap())
                .collect::<Vec<_>>()
        });
        assert_eq!(outcomes.iter().filter(|outcome| outcome.is_ok()).count(), 1);
        let winner = outcomes.into_iter().find_map(Result::ok).unwrap().unwrap();
        assert_eq!(validated_manifest(&root).manifest, *winner.manifest());
        let entry = &winner.manifest().modules[0];
        let index = concurrent
            .outputs
            .iter()
            .position(|output| output.source.module_path.as_str() == entry.module.module_path)
            .unwrap();
        assert_eq!(
            std::fs::read(root.join("shared.mizir.json")).unwrap(),
            mizar_artifact::comment_projection::write_commented_artifact(
                &concurrent.retained(index).artifact
            )
            .unwrap(),
        );
        // Fresh reads must also reject corruption without changing retention.
        let mut f = Fixture::new(TEXT, None);
        let frontend = f.frontend(0);
        f.capture(0, &frontend);
        let retained = f.retained(0).clone();
        let root = f.emission_root();
        let commit = f.emit(&root, "published.mizir.json").unwrap().unwrap();
        let manifest_before = std::fs::read(root.join("artifact-manifest.json")).unwrap();
        let main_before = std::fs::read(root.join("published.mizir.json")).unwrap();
        for class in [
            ArtifactHashClass::Artifact,
            ArtifactHashClass::Interface,
            ArtifactHashClass::Implementation,
        ] {
            let mut corrupted = commit.manifest().clone();
            let entry = &mut corrupted.modules[0];
            match class {
                ArtifactHashClass::Artifact => entry.artifact_hash.digest = hash(90),
                ArtifactHashClass::Interface => entry.interface_hash.digest = hash(90),
                ArtifactHashClass::Implementation => entry.implementation_hash.digest = hash(90),
                _ => unreachable!("only the three enclosing hash classes"),
            }
            write_manifest_file(&root, &corrupted).unwrap();
            assert!(
                read_manifest_file(
                    &root,
                    ManifestFileReadOptions {
                        validate_references: true,
                        ..ManifestFileReadOptions::default()
                    }
                )
                .is_err()
            );
        }
        std::fs::write(root.join("artifact-manifest.json"), &manifest_before).unwrap();
        let mut envelope =
            parse_canonical_json_bytes(&main_before, "published.mizir.json").unwrap();
        let CanonicalJson::Object(fields) = &mut envelope else {
            panic!("envelope object")
        };
        fields.insert("comments".into(), CanonicalJson::Null);
        std::fs::write(
            root.join("published.mizir.json"),
            mizar_artifact::store::canonical_json_bytes(&envelope),
        )
        .unwrap();
        assert!(
            read_commented_artifact(&envelope, VerifiedArtifactReadOptions::default()).is_err()
        );
        assert!(
            read_manifest_file(
                &root,
                ManifestFileReadOptions {
                    validate_references: true,
                    ..ManifestFileReadOptions::default()
                }
            )
            .is_err()
        );
        assert_eq!(f.retained(0), &retained);
        std::fs::write(root.join("published.mizir.json"), &main_before).unwrap();
        assert!(f.emit(&root, "published.mizir.json").is_err());
        assert_eq!(
            std::fs::read(root.join("artifact-manifest.json")).unwrap(),
            manifest_before
        );
        assert_eq!(
            std::fs::read(root.join("published.mizir.json")).unwrap(),
            main_before
        );
        assert_eq!(f.retained(0), &retained);
        assert_eq!(validated_manifest(&root).manifest, *commit.manifest());
    }
}
