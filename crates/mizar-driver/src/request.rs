use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;

use mizar_frontend::{
    orchestration::{CommentMetadata, FrontendOutput},
    parsing::{MizarParserSeam, ParserSeam},
    source::{diagnostic_file_path, source_unit_from_loaded},
};
use mizar_ir::{
    projection::{ArtifactProjectionService, CommentedArtifactDraft, ProjectVerifiedArtifactInput},
    publisher::{OutputAuthorization, PhaseOutputPublisher},
    storage::{AnyPhaseOutputRef, PhaseOutputRef},
};

use mizar_session::{
    BuildRequestId, BuildSessionId, BuildSnapshot, BuildSnapshotId, DependencyArtifactRef,
    DiskSourceLoader, Hash, IdError, LoadedSource, ModulePath, NormalizedPath, PackageId,
    SessionIdAllocator, SnapshotError, SnapshotInput, SnapshotLease, SnapshotRegistry, SourceId,
    SourceInput, SourceLoadError, SourceLoader, SourceOriginInput, SourceVersion, ToolchainInfo,
    WorkspaceRoot,
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
    pub(crate) snapshot: BuildSnapshot,
    pub(crate) active_snapshot_lease: SnapshotLease,
    native_sources: Option<std::sync::Arc<[(DiskSourceLoader, LoadedSource, SourceInput)]>>,
    source_load_failures: Vec<(SourceInput, SourceLoadError)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildSession {
    pub(crate) id: BuildSessionId,
    pub(crate) request: BuildRequest,
    pub(crate) captured: CapturedSnapshot,
    pub(crate) state: BuildSessionState,
    commented_modules: HashMap<
        SourceId,
        Arc<(
            CommentedArtifactDraft,
            Vec<AnyPhaseOutputRef>,
            mizar_ir::identity::PhaseOutputId,
        )>,
    >,
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
    /// Captures native sources once, before the owner authorizes prepared submission.
    pub fn capture_sources<A: SessionIdAllocator>(
        self,
        allocator: &A,
        registry: &SnapshotRegistry<A>,
        sources: Vec<(DiskSourceLoader, SourceInput)>,
    ) -> Result<BuildSession, crate::driver::DriverSubmitError> {
        use crate::driver::DriverSubmitError;
        if !self.source_inputs.versions.is_empty() {
            return Err(DriverSubmitError::NativeSourceVersionsProvided {
                request: Box::new(self),
            });
        }
        let mut pending = self
            .allocate(allocator)
            .map_err(|error| DriverSubmitError::RequestIdAllocation { error })?;
        let loaders = sources
            .iter()
            .map(|(loader, _)| loader.clone())
            .collect::<Vec<_>>();
        let result = registry.load_snapshot(
            pending.request.id,
            (
                pending.request.workspace_root.clone(),
                pending.request.dependency_inputs.artifacts.clone(),
                pending.request.dependency_inputs.lockfile_hash,
                pending.request.dependency_inputs.toolchain.clone(),
                pending.request.verifier_config.hash,
            ),
            sources,
        );
        let (snapshot, active_snapshot_lease, sources) =
            result.map_err(|error| DriverSubmitError::SnapshotCapture {
                error: Box::new(CaptureSnapshotError {
                    pending: pending.clone(),
                    error,
                }),
            })?;
        pending.request.source_inputs.versions = snapshot.source_versions.clone();
        Ok(BuildSession {
            id: pending.session_id,
            request: pending.request,
            captured: CapturedSnapshot {
                snapshot,
                active_snapshot_lease,
                source_load_failures: Vec::new(),
                native_sources: Some(
                    loaders
                        .into_iter()
                        .zip(sources)
                        .map(|(loader, (loaded, input))| (loader, loaded, input))
                        .collect::<Vec<_>>()
                        .into(),
                ),
            },
            state: BuildSessionState::SnapshotCaptured,
            commented_modules: HashMap::new(),
        })
    }

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
                native_sources: None,
                source_load_failures: Vec::new(),
            },
            state: BuildSessionState::SnapshotCaptured,
            commented_modules: HashMap::new(),
        })
    }
}

impl CapturedSnapshot {
    pub(crate) fn source_load_failures(&self) -> &[(SourceInput, SourceLoadError)] {
        &self.source_load_failures
    }

    pub fn snapshot(&self) -> &BuildSnapshot {
        &self.snapshot
    }
    pub fn active_snapshot_lease(&self) -> &SnapshotLease {
        &self.active_snapshot_lease
    }
    pub fn native_sources(&self) -> Option<&[(DiskSourceLoader, LoadedSource, SourceInput)]> {
        self.native_sources.as_deref()
    }
}

impl BuildSession {
    /// Retains an unpublished comment draft bound to this session's native capture.
    pub fn capture_commented_module<A: SessionIdAllocator>(
        &mut self,
        publisher: &Arc<PhaseOutputPublisher>,
        snapshots: &SnapshotRegistry<A>,
        input: ProjectVerifiedArtifactInput,
        frontend: &PhaseOutputRef<FrontendOutput<<MizarParserSeam as ParserSeam>::Ast>>,
        metadata: &CommentMetadata,
        targets: &[Option<(String, String)>],
    ) -> Result<(), String> {
        if matches!(
            self.state,
            BuildSessionState::Cancelling
                | BuildSessionState::Finished(
                    BuildSessionOutcome::Cancelled | BuildSessionOutcome::Superseded
                )
        ) || input.snapshot != self.captured.snapshot.id
            || snapshots.get(input.snapshot).as_ref() != Some(&self.captured.snapshot)
            || !snapshots.is_current_for_request(input.snapshot, self.request.id)
        {
            return Err("comment draft requires a current captured session snapshot".into());
        }
        let sources = self
            .captured
            .native_sources()
            .ok_or("comment draft requires a native source capture")?;
        let output = publisher
            .storage()
            .get(frontend)
            .map_err(|error| error.to_string())?;
        let source = &output.source;
        let mut captured = sources
            .iter()
            .filter(|(_, loaded, _)| loaded.source_id == source.source_id);
        let (_, loaded, captured_input) = captured
            .next()
            .ok_or("comment draft source is not captured by this session")?;
        if captured.next().is_some()
            || source_unit_from_loaded(loaded.clone(), diagnostic_file_path(captured_input))
                != *source
        {
            return Err("comment draft source differs from the native capture".into());
        }
        let handles = input.required_outputs.clone();
        let draft = ArtifactProjectionService::new(Arc::clone(publisher))
            .project_commented_module(input, snapshots, frontend, metadata, targets)
            .map_err(|error| error.to_string())?;
        if snapshots.get(self.captured.snapshot.id).as_ref() != Some(&self.captured.snapshot)
            || !snapshots.is_current_for_request(self.captured.snapshot.id, self.request.id)
        {
            return Err("comment draft capture is no longer current".into());
        }
        self.commented_modules.insert(
            source.source_id,
            Arc::new((draft, handles, frontend.output())),
        );
        Ok(())
    }

    /// Borrows a retained draft only while its session and original sealed handles remain eligible.
    pub fn commented_module<A: SessionIdAllocator>(
        &self,
        publisher: &PhaseOutputPublisher,
        snapshots: &SnapshotRegistry<A>,
        source: SourceId,
    ) -> Result<Option<&CommentedArtifactDraft>, String> {
        if matches!(
            self.state,
            BuildSessionState::Cancelling
                | BuildSessionState::Finished(
                    BuildSessionOutcome::Cancelled | BuildSessionOutcome::Superseded
                )
        ) || snapshots.get(self.captured.snapshot.id).as_ref() != Some(&self.captured.snapshot)
            || !snapshots.is_current_for_request(self.captured.snapshot.id, self.request.id)
        {
            return Err(
                "comment draft session is cancelled, superseded or no longer current".into(),
            );
        }
        publisher
            .validate_current_snapshot(self.captured.snapshot.id)
            .map_err(|error| error.to_string())?;
        let Some(retained) = self.commented_modules.get(&source) else {
            return Ok(None);
        };
        for handle in &retained.1 {
            publisher
                .validate_output(retained.0.snapshot, handle, OutputAuthorization::Published)
                .map_err(|error| error.to_string())?;
            publisher
                .storage()
                .validate_handle(handle)
                .map_err(|error| error.to_string())?;
            if handle.output() == retained.2 {
                let frontend = publisher
                    .storage()
                    .typed_handle::<FrontendOutput<<MizarParserSeam as ParserSeam>::Ast>>(
                        handle,
                        handle.output_kind(),
                    )
                    .map_err(|error| error.to_string())?;
                publisher
                    .storage()
                    .get(&frontend)
                    .map_err(|error| error.to_string())?;
            }
        }
        publisher
            .validate_current_snapshot(self.captured.snapshot.id)
            .map_err(|error| error.to_string())?;
        if snapshots.get(self.captured.snapshot.id).as_ref() != Some(&self.captured.snapshot)
            || !snapshots.is_current_for_request(self.captured.snapshot.id, self.request.id)
        {
            return Err("comment draft session is no longer current".into());
        }
        Ok(Some(&retained.0))
    }

    /// Retains genuine additional Disk failures without changing the original capture.
    pub fn capture_source_load_failures(
        &mut self,
        allocator: &dyn SessionIdAllocator,
        sources: Vec<(DiskSourceLoader, SourceInput)>,
    ) -> Result<(), String> {
        let captured = self
            .captured
            .native_sources()
            .filter(|sources| !sources.is_empty());
        let Some(captured) = captured else {
            return Err("nonempty native capture required".into());
        };
        if self.state != BuildSessionState::SnapshotCaptured
            || !self.captured.source_load_failures.is_empty()
            || sources.is_empty()
        {
            return Err("invalid supplemental source capture".into());
        }
        for (index, (loader, input)) in sources.iter().enumerate() {
            let SourceOriginInput::Disk { path } = &input.origin else {
                return Err("supplemental source must be Disk".into());
            };
            let relative = std::path::Path::new(input.normalized_path.as_str());
            if !crate::frontend_adapter::safe_relative(input.normalized_path.as_str())
                || (path != relative && *path != loader.package_root().join(relative))
                || !captured.iter().any(|(original, loaded, _)| {
                    original == loader && loaded.package_id == input.package_id
                })
                || captured.iter().any(|(_, loaded, _)| {
                    loaded.package_id == input.package_id
                        && (loaded.normalized_path == input.normalized_path
                            || loaded.module_path == input.module_path)
                })
                || sources[..index].iter().any(|(_, previous)| {
                    previous.package_id == input.package_id
                        && (previous.normalized_path == input.normalized_path
                            || previous.module_path == input.module_path)
                })
            {
                return Err("unbound or duplicate supplemental source".into());
            }
        }
        let failures = sources
            .into_iter()
            .map(|(loader, input)| {
                match loader.load(self.captured.snapshot.id, input.clone(), allocator) {
                    Err(error) => Ok((input, error)),
                    Ok(_) => Err("supplemental source unexpectedly loaded".to_owned()),
                }
            })
            .collect::<Result<Vec<_>, _>>()?;
        self.captured.source_load_failures = failures;
        Ok(())
    }

    pub const fn id(&self) -> BuildSessionId {
        self.id
    }
    pub fn request(&self) -> &BuildRequest {
        &self.request
    }
    pub fn captured(&self) -> &CapturedSnapshot {
        &self.captured
    }
    pub const fn state(&self) -> BuildSessionState {
        self.state
    }

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
