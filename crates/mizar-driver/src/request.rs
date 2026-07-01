use std::collections::HashMap;
use std::fmt;

use mizar_session::{
    BuildRequestId, BuildSessionId, BuildSnapshot, BuildSnapshotId, DependencyArtifactRef, Hash,
    IdError, ModulePath, NormalizedPath, PackageId, SessionIdAllocator, SnapshotError,
    SnapshotInput, SnapshotLease, SnapshotRegistry, SourceVersion, ToolchainInfo, WorkspaceRoot,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildRequestDraft {
    pub lane: BuildLaneId,
    pub origin: BuildRequestOrigin,
    pub generation: BuildRequestGeneration,
    pub workspace_root: WorkspaceRoot,
    pub profile: BuildProfile,
    pub targets: BuildTargets,
    pub source_inputs: SourceInputSet,
    pub dependency_inputs: DependencyInputSet,
    pub verifier_config: VerifierConfigInput,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildRequest {
    pub id: BuildRequestId,
    pub lane: BuildLaneId,
    pub origin: BuildRequestOrigin,
    pub generation: BuildRequestGeneration,
    pub workspace_root: WorkspaceRoot,
    pub profile: BuildProfile,
    pub targets: BuildTargets,
    pub source_inputs: SourceInputSet,
    pub dependency_inputs: DependencyInputSet,
    pub verifier_config: VerifierConfigInput,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PendingBuildRequest {
    pub session_id: BuildSessionId,
    pub request: BuildRequest,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CaptureSnapshotError {
    pub pending: PendingBuildRequest,
    pub error: SnapshotError,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapturedSnapshot {
    pub snapshot: BuildSnapshot,
    pub active_snapshot_lease: SnapshotLease,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildSession {
    pub id: BuildSessionId,
    pub request: BuildRequest,
    pub captured: CapturedSnapshot,
    pub state: BuildSessionState,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct DriverLanes {
    current_by_lane: HashMap<BuildLaneId, LaneCurrentSession>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LaneCurrentSession {
    pub lane: BuildLaneId,
    pub generation: BuildRequestGeneration,
    pub session: BuildSessionId,
    pub snapshot: BuildSnapshotId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BuildLaneId(u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BuildRequestGeneration(u64);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildProfile {
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct BuildTargets {
    pub packages: Vec<PackageId>,
    pub modules: Vec<ModulePath>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SourceInputSet {
    pub versions: Vec<SourceVersion>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DependencyInputSet {
    pub artifacts: Vec<DependencyArtifactRef>,
    pub lockfile_hash: Hash,
    pub toolchain: ToolchainInfo,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VerifierConfigInput {
    pub hash: Hash,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum BuildRequestOrigin {
    Batch(BatchRequest),
    Watch(WatchRequest),
    Lsp(LspRequest),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BatchRequest {
    pub invocation: BatchInvocation,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct BatchInvocation {
    pub args: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WatchRequest {
    pub watch_root: WorkspaceRoot,
    pub changed_paths: Vec<NormalizedPath>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LspRequest {
    pub focus: Option<LspFocus>,
    pub priority: LspPriority,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LspFocus {
    pub package: PackageId,
    pub module: ModulePath,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum LspPriority {
    Normal,
    Focused,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum BuildSessionState {
    SnapshotCaptured,
    Submitted,
    Running,
    Cancelling,
    Finished(BuildSessionOutcome),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum BuildSessionOutcome {
    Succeeded,
    Failed,
    Blocked,
    Cancelled,
    Superseded,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum PublicationDecision {
    Current,
    Suppressed(ObsoletePublication),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ObsoletePublication {
    pub lane_current: bool,
    pub request_snapshot_current: bool,
}

impl BuildLaneId {
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    pub const fn get(self) -> u64 {
        self.0
    }
}

impl BuildRequestGeneration {
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    pub const fn get(self) -> u64 {
        self.0
    }
}

impl BuildProfile {
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }
}

impl DependencyInputSet {
    pub fn new(
        artifacts: Vec<DependencyArtifactRef>,
        lockfile_hash: Hash,
        toolchain: ToolchainInfo,
    ) -> Self {
        Self {
            artifacts,
            lockfile_hash,
            toolchain,
        }
    }
}

impl VerifierConfigInput {
    pub const fn new(hash: Hash) -> Self {
        Self { hash }
    }
}

impl BuildRequestDraft {
    pub fn allocate<A: SessionIdAllocator>(
        self,
        allocator: &A,
    ) -> Result<PendingBuildRequest, IdError> {
        let session_id = allocator.next_session_id()?;
        let request_id = allocator.next_request_id()?;
        Ok(PendingBuildRequest {
            session_id,
            request: BuildRequest {
                id: request_id,
                lane: self.lane,
                origin: self.origin,
                generation: self.generation,
                workspace_root: self.workspace_root,
                profile: self.profile,
                targets: self.targets,
                source_inputs: self.source_inputs,
                dependency_inputs: self.dependency_inputs,
                verifier_config: self.verifier_config,
            },
        })
    }
}

impl BuildRequest {
    pub fn snapshot_input(&self) -> SnapshotInput {
        SnapshotInput {
            workspace_root: self.workspace_root.clone(),
            source_versions: self.source_inputs.versions.clone(),
            dependency_artifacts: self.dependency_inputs.artifacts.clone(),
            lockfile_hash: self.dependency_inputs.lockfile_hash,
            toolchain: self.dependency_inputs.toolchain.clone(),
            verifier_config_hash: self.verifier_config.hash,
        }
    }
}

impl PendingBuildRequest {
    pub fn capture_snapshot<A: SessionIdAllocator>(
        self,
        registry: &SnapshotRegistry<A>,
    ) -> Result<BuildSession, Box<CaptureSnapshotError>> {
        let result = registry.create_snapshot(self.request.id, self.request.snapshot_input());
        let (snapshot, active_snapshot_lease) = match result {
            Ok(captured) => captured,
            Err(error) => {
                return Err(Box::new(CaptureSnapshotError {
                    pending: self,
                    error,
                }));
            }
        };
        Ok(BuildSession {
            id: self.session_id,
            request: self.request,
            captured: CapturedSnapshot {
                snapshot,
                active_snapshot_lease,
            },
            state: BuildSessionState::SnapshotCaptured,
        })
    }
}

impl BuildSession {
    pub fn lane_current_session(&self) -> LaneCurrentSession {
        LaneCurrentSession {
            lane: self.request.lane,
            generation: self.request.generation,
            session: self.id,
            snapshot: self.captured.snapshot.id,
        }
    }

    pub fn mark_submitted(&mut self) {
        if self.state == BuildSessionState::SnapshotCaptured {
            self.state = BuildSessionState::Submitted;
        }
    }

    pub fn mark_running(&mut self) {
        if self.state == BuildSessionState::Submitted {
            self.state = BuildSessionState::Running;
        }
    }

    pub fn cancel(&mut self) -> bool {
        if self.is_terminal() || self.state == BuildSessionState::Cancelling {
            return false;
        }
        self.state = BuildSessionState::Cancelling;
        true
    }

    pub fn finish(&mut self, outcome: BuildSessionOutcome) -> bool {
        if self.is_terminal() {
            return false;
        }
        self.state = BuildSessionState::Finished(outcome);
        true
    }

    pub const fn is_terminal(&self) -> bool {
        matches!(self.state, BuildSessionState::Finished(_))
    }
}

impl DriverLanes {
    pub fn mark_current(&mut self, session: &BuildSession) -> bool {
        let attempted = session.lane_current_session();
        if let Some(current) = self.current_by_lane.get(&attempted.lane) {
            if current.generation.get() > attempted.generation.get() {
                return false;
            }
            if current.generation == attempted.generation
                && (current.session != attempted.session || current.snapshot != attempted.snapshot)
            {
                return false;
            }
        }
        self.current_by_lane.insert(attempted.lane, attempted);
        true
    }

    pub fn current(&self, lane: BuildLaneId) -> Option<LaneCurrentSession> {
        self.current_by_lane.get(&lane).copied()
    }

    pub fn is_current_session(
        &self,
        lane: BuildLaneId,
        generation: BuildRequestGeneration,
        session: BuildSessionId,
        snapshot: BuildSnapshotId,
    ) -> bool {
        self.current_by_lane.get(&lane).is_some_and(|current| {
            current.generation == generation
                && current.session == session
                && current.snapshot == snapshot
        })
    }

    pub fn is_session_current(&self, session: &BuildSession) -> bool {
        self.is_current_session(
            session.request.lane,
            session.request.generation,
            session.id,
            session.captured.snapshot.id,
        )
    }

    pub fn publication_decision<A: SessionIdAllocator>(
        &self,
        registry: &SnapshotRegistry<A>,
        session: &BuildSession,
    ) -> PublicationDecision {
        let lane_current = self.is_session_current(session);
        let request_snapshot_current =
            registry.is_current_for_request(session.captured.snapshot.id, session.request.id);
        if lane_current && request_snapshot_current {
            PublicationDecision::Current
        } else {
            PublicationDecision::Suppressed(ObsoletePublication {
                lane_current,
                request_snapshot_current,
            })
        }
    }
}

impl fmt::Display for BuildLaneId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

impl fmt::Display for BuildRequestGeneration {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}
