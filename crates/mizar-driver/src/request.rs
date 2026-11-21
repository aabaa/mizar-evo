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
    /// Acquires and captures the bounded workspace-only native LSP context.
    /// The caller composes `StageConfig::frontend(vec![])`; this does not run stages.
    pub fn capture_native_lsp<A: SessionIdAllocator>(
        workspace_root: WorkspaceRoot,
        lane: BuildLaneId,
        generation: BuildRequestGeneration,
        request: LspRequest,
        allocator: &A,
        registry: &SnapshotRegistry<A>,
    ) -> Result<
        (
            Self,
            crate::driver::DriverSubmitInput<mizar_build::module_index::StaticSourceLayout>,
        ),
        String,
    > {
        use mizar_build::{
            module_index::{ModuleIndexLocation, StaticSourceLayout, build_module_index},
            planner::{
                DependencySelection, PackagePlanSource, PlanRequest, lockfile_content_hash,
                produce_build_plan, read_native_workspace_inputs, workspace_config_content_hash,
            },
        };

        let image = crate::frontend_adapter::SourceLoadService::running_native_image()?;
        let toolchain = ToolchainInfo::new(format!("{}:{}", image.family, image.value));
        let (packages, lockfile) = read_native_workspace_inputs(&workspace_root)?;
        let plan_request = PlanRequest {
            workspace_root: workspace_root.clone(),
            dependency_selection: DependencySelection::Normal,
            toolchain: toolchain.clone(),
        };
        let plan = produce_build_plan(plan_request.clone(), packages.clone(), lockfile.clone())
            .map_err(|error| error.to_string())?;
        if plan
            .packages
            .iter()
            .any(|package| !matches!(package.source, PackagePlanSource::Workspace { .. }))
        {
            return Err("native LSP context requires workspace-only plan rows".into());
        }
        let layout = StaticSourceLayout::from_native_disk(&plan)?;
        let index =
            build_module_index(&plan, &layout, &[]).map_err(|error| format!("{error:?}"))?;
        let mut sources = Vec::new();
        for package in &plan.packages {
            let PackagePlanSource::Workspace { root, .. } = &package.source else {
                return Err("native LSP context requires workspace-only plan rows".into());
            };
            let package_root = std::path::Path::new(workspace_root.as_str()).join(root);
            let loader = DiskSourceLoader::new(&package_root);
            for module in index
                .modules
                .iter()
                .filter(|module| module.package_id == package.package_id)
            {
                let ModuleIndexLocation::WorkspaceFile {
                    normalized_path, ..
                } = &module.location
                else {
                    return Err("native LSP context requires workspace source inputs".into());
                };
                let input = SourceInput {
                    package_id: package.package_id.clone(),
                    module_path: module.module_path.clone(),
                    normalized_path: mizar_session::normalize_path(
                        &package_root,
                        std::path::Path::new(normalized_path),
                    )
                    .map_err(|error| error.to_string())?,
                    edition: module.edition.clone(),
                    origin: SourceOriginInput::Disk {
                        path: normalized_path.clone().into(),
                    },
                };
                sources.push((loader.clone(), input));
            }
        }
        let draft = BuildRequestDraft {
            lane,
            origin: BuildRequestOrigin::Lsp(request),
            generation,
            workspace_root,
            profile: BuildProfile::new("lsp"),
            targets: BuildTargets {
                packages: plan
                    .packages
                    .iter()
                    .map(|package| package.package_id.clone())
                    .collect(),
                modules: Vec::new(),
            },
            source_inputs: SourceInputSet::default(),
            dependency_inputs: DependencyInputSet::new(
                Vec::new(),
                lockfile_content_hash(&lockfile),
                toolchain,
            ),
            verifier_config: VerifierConfigInput::new(workspace_config_content_hash(
                &plan.verifier_config,
                &plan.build_config,
            )),
        };
        let session = draft
            .capture_sources(allocator, registry, sources)
            .map_err(|error| format!("{error:?}"))?;
        Ok((
            session,
            crate::driver::DriverSubmitInput::new(plan_request, packages, lockfile, layout),
        ))
    }

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

    /// Publishes the exact retained envelope through the store and manifest transaction.
    pub fn emit_commented_module<A: SessionIdAllocator>(
        &self,
        publisher: &PhaseOutputPublisher,
        snapshots: &SnapshotRegistry<A>,
        source: SourceId,
        artifact_root: &std::path::Path,
        main_path: &str,
    ) -> Result<Option<mizar_artifact::manifest::ManifestCommit>, String> {
        use mizar_artifact::{
            comment_projection::{
                COMMENTED_ARTIFACT_SCHEMA_FAMILY, COMMENTED_ARTIFACT_SCHEMA_VERSION,
                artifact_hash_excluded_paths, commented_artifact_json, read_commented_artifact,
            },
            manifest::{
                ArtifactManifest, ManifestCommitOptions, ManifestFileReadOptions,
                ManifestProofWitnessEntry, ManifestProvenance, ManifestTransaction,
                ModuleArtifactEntry, PackageIdentity, current_schema_version, read_manifest_file,
            },
            registration_summary::{ArtifactHashClass, ArtifactHashRef},
            store::{
                PublishedArtifactPath, PublishedArtifactReadOptions, artifact_hash_domain,
                read_published_artifact, write_published_artifact,
            },
            verified_artifact::VerifiedArtifactReadOptions,
        };
        let Some(draft) = self.commented_module(publisher, snapshots, source)? else {
            return Ok(None);
        };
        let retained = &self.commented_modules[&source];
        let validate = || -> Result<(), String> {
            self.commented_module(publisher, snapshots, source)?
                .ok_or("comment draft is no longer retained")?;
            for handle in &retained.1 {
                publisher
                    .validate_output(draft.snapshot, handle, OutputAuthorization::CurrentPackage)
                    .map_err(|error| error.to_string())?;
            }
            Ok(())
        };
        validate()?;
        let (loader, _, _) = self
            .captured
            .native_sources()
            .and_then(|sources| {
                sources
                    .iter()
                    .find(|(_, loaded, _)| loaded.source_id == source)
            })
            .ok_or("comment draft requires its captured source loader")?;
        let package_root =
            std::fs::canonicalize(loader.package_root()).map_err(|error| error.to_string())?;
        let root = std::fs::canonicalize(artifact_root).map_err(|error| error.to_string())?;
        let relative = root
            .strip_prefix(&package_root)
            .map_err(|_| "comment artifact root must be below package root")?;
        if relative.as_os_str().is_empty() {
            return Err("comment artifact root must be below package root".into());
        }
        let label = relative
            .components()
            .map(|component| match component {
                std::path::Component::Normal(value) => {
                    value.to_str().ok_or("comment artifact root is not UTF-8")
                }
                _ => Err("comment artifact root has an invalid component"),
            })
            .collect::<Result<Vec<_>, _>>()?
            .join("/");
        let label = PublishedArtifactPath::new(label).map_err(|error| error.to_string())?;
        let path = PublishedArtifactPath::new(main_path).map_err(|error| error.to_string())?;
        if !path.as_str().ends_with(".mizir.json") {
            return Err("comment artifact requires a .mizir.json main path".into());
        }
        let envelope = &draft.artifact;
        let artifact = &envelope.artifact;
        let value = commented_artifact_json(envelope).map_err(|error| error.to_string())?;
        let interface_hash = envelope
            .compute_interface_hash()
            .map_err(|error| error.to_string())?;
        let implementation_hash = envelope
            .compute_implementation_hash()
            .map_err(|error| error.to_string())?;
        let guard = draft
            .snapshot
            .to_published_schema_string()
            .map_err(|error| error.to_string())?;
        let freshness =
            |candidate: Option<&str>| candidate == Some(guard.as_str()) && validate().is_ok();
        let seed = ArtifactManifest {
            schema_version: current_schema_version(),
            package: PackageIdentity {
                package_id: artifact.module.package_id.clone(),
                package_version: artifact.module.package_version.clone(),
                lockfile_identity: artifact.module.lockfile_identity.clone(),
            },
            artifact_root: label.as_str().to_owned(),
            lockfile_hash: artifact.provenance.lockfile_hash.clone(),
            toolchain: artifact.provenance.toolchain.clone(),
            language_edition: artifact.provenance.language_edition.clone(),
            verifier_config_hash: artifact.provenance.verifier_config_hash.clone(),
            modules: Vec::new(),
            development_artifacts: Vec::new(),
            provenance: ManifestProvenance {
                generated_by: concat!("mizar-driver/", env!("CARGO_PKG_VERSION")).to_owned(),
                manifest_policy: "commented-module".to_owned(),
                transaction_format: "mizar-artifact/manifest-transaction-v1".to_owned(),
            },
        };
        let mut transaction = ManifestTransaction::begin(&root, seed, Some(guard.clone()))
            .map_err(|error| error.to_string())?;
        let domain = artifact_hash_domain(
            COMMENTED_ARTIFACT_SCHEMA_FAMILY,
            COMMENTED_ARTIFACT_SCHEMA_VERSION,
        );
        let excluded = artifact_hash_excluded_paths();
        validate()?;
        // Reserve the main before the replacing store writer can touch it. Other
        // Driver emissions must refuse this path, even if our transaction fails.
        let _root_lock =
            crate::frontend_adapter::lock_artifact_writes().map_err(|error| error.to_string())?;
        validate()?;
        // Legacy cleanup must retain this root even after a failed Q1 emission.
        match std::fs::File::create_new(
            root.join(crate::frontend_adapter::COMMENT_ARTIFACT_RESERVATION),
        ) {
            Ok(_) => {}
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
            Err(error) => return Err(error.to_string()),
        }
        crate::frontend_adapter::reserve_native_artifact_path(
            &root,
            &path,
            "comment artifact main parent must stay below artifact root",
        )
        .map_err(|error| error.to_string())?;
        let written = write_published_artifact(&root, &path, &value, &domain, &excluded)
            .map_err(|error| error.to_string())?;
        validate()?;
        let read = read_published_artifact(
            &root,
            &path,
            PublishedArtifactReadOptions {
                artifact_hash_domain: Some(&domain),
                hash_excluded_paths: &excluded,
                expected_artifact_hash: Some(written.artifact_hash),
            },
        )
        .map_err(|error| error.to_string())?;
        let decoded = read_commented_artifact(
            &read.value,
            VerifiedArtifactReadOptions {
                artifact_path: Some(path.as_str()),
                expected_module: Some(&artifact.module),
                expected_interface_hash: Some(interface_hash),
                expected_implementation_hash: Some(implementation_hash),
            },
        )
        .map_err(|error| error.to_string())?;
        if decoded != *envelope {
            return Err("comment artifact readback mismatch".into());
        }
        validate()?;
        let hash_ref = |class, hash| {
            ArtifactHashRef::new(
                class,
                COMMENTED_ARTIFACT_SCHEMA_FAMILY,
                COMMENTED_ARTIFACT_SCHEMA_VERSION,
                hash,
            )
        };
        transaction
            .stage_module(ModuleArtifactEntry {
                module: artifact.module.clone(),
                source_file: artifact.source_file.clone(),
                source_hash: artifact.source_hash,
                artifact_file: written.path.as_str().to_owned(),
                artifact_hash: hash_ref(ArtifactHashClass::Artifact, written.artifact_hash),
                interface_hash: hash_ref(ArtifactHashClass::Interface, interface_hash),
                implementation_hash: hash_ref(
                    ArtifactHashClass::Implementation,
                    implementation_hash,
                ),
                module_summary_file: None,
                module_summary_hash: None,
                module_summary_interface_hash: None,
                registration_summary_file: None,
                registration_summary_hash: None,
                registration_interface_hash: None,
                proof_witnesses: artifact
                    .proof_witnesses
                    .iter()
                    .map(|witness| ManifestProofWitnessEntry {
                        obligation_id: witness.obligation_id.clone(),
                        obligation_fingerprint: witness.obligation_fingerprint.clone(),
                        witness_path: witness.witness_path.clone(),
                        witness_artifact_hash: witness.witness_artifact_hash.clone(),
                    })
                    .collect(),
                diagnostics_hash: None,
            })
            .map_err(|error| error.to_string())?;
        let commit = transaction
            .commit(ManifestCommitOptions {
                freshness_check: Some(&freshness),
            })
            .map_err(|error| error.to_string())?;
        validate()?;
        let read = read_manifest_file(
            &root,
            ManifestFileReadOptions {
                validate_references: true,
                validate_witness_files: true,
                validate_development_artifacts: true,
                expected_manifest_hash: Some(commit.write().artifact_hash),
            },
        )
        .map_err(|error| error.to_string())?;
        if read.manifest != *commit.manifest() || read.artifact_hash != commit.write().artifact_hash
        {
            return Err("comment artifact manifest readback mismatch".into());
        }
        validate()?;
        Ok(Some(commit))
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
