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

    assert_eq!(session.state, BuildSessionState::SnapshotCaptured);
    assert_eq!(
        session.captured.active_snapshot_lease.snapshot,
        session.captured.snapshot.id
    );
    assert_eq!(
        session.captured.active_snapshot_lease.reason,
        RetentionReason::ActiveBuild
    );
    assert!(registry.is_current_for_request(session.captured.snapshot.id, session.request.id));
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

    assert_eq!(first.captured.snapshot.id, second.captured.snapshot.id);
    assert_ne!(first.id, second.id);
    assert_ne!(first.request.id, second.request.id);
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

    assert_eq!(first.captured.snapshot.id, second.captured.snapshot.id);
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
        .create_snapshot(stale.request.id, snapshot_input_parts(10).snapshot_input())
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
    assert_eq!(completed.state, BuildSessionState::Submitted);
    completed.mark_running();
    assert_eq!(completed.state, BuildSessionState::Running);
    assert!(completed.finish(BuildSessionOutcome::Succeeded));
    completed.mark_submitted();
    completed.mark_running();
    assert!(!completed.cancel());
    assert!(!completed.finish(BuildSessionOutcome::Cancelled));
    assert_eq!(
        completed.state,
        BuildSessionState::Finished(BuildSessionOutcome::Succeeded)
    );

    let mut session = pending_session(BuildLaneId::new(12), BuildRequestGeneration::new(0), 16)
        .capture_snapshot(&registry)
        .unwrap();
    assert!(session.cancel());
    session.mark_submitted();
    assert_eq!(session.state, BuildSessionState::Cancelling);
    session.mark_running();
    assert_eq!(session.state, BuildSessionState::Cancelling);

    assert!(session.finish(BuildSessionOutcome::Cancelled));
    assert!(!session.finish(BuildSessionOutcome::Succeeded));
    assert_eq!(
        session.state,
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
