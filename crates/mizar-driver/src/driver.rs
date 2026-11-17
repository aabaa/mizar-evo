use std::{
    collections::{BTreeMap, HashMap},
    sync::Arc,
};

use mizar_build::{
    cache_seam::CacheSchedulingPlan,
    cancel::{CancellationPolicy, CancellationReason, CancellationToken},
    module_index::{
        DependencyArtifactIndex, ModuleIndex, ModuleIndexDiagnostics, SourceLayoutProvider,
        build_module_index,
    },
    planner::{
        BuildPlan, Lockfile, ManifestDiagnostics, PlanRequest, WorkspacePackage, produce_build_plan,
    },
    resource::ResourceBudget,
    scheduler::{
        CacheSchedulingPolicy, CompletionOrder, PriorityHints, SchedulerDiagnostic,
        SchedulerDiagnostics, SchedulerEvent, SchedulerExecution, SchedulerInput, SchedulerMode,
        TaskStateRecord,
    },
    task_graph::{
        BuildTask, ModuleDependencyOverlay, PipelinePhase, TaskGraph, TaskGraphDiagnostics,
        TaskGraphInput, TaskGraphProfile, TaskId, VcTaskDescriptor, build_task_graph,
    },
};
pub use mizar_ir::dispatch_input::PhaseDispatchInputProvider;
use mizar_ir::publisher::{OutputAuthorization, PhaseOutputPublisher, PublishError};
use mizar_session::{
    BuildSessionId, BuildSnapshotId, IdError, SessionIdAllocator, SnapshotRegistry,
};

use crate::{
    events::{BuildEventStream, OwnerGapClassification, PlanningEventStatus},
    registry::{PhaseRegistry, PhaseRegistryError, PhaseResult, PhaseServiceAvailability},
    request::{
        BuildRequestDraft, BuildRequestOrigin, BuildSession, BuildSessionOutcome,
        BuildSessionState, CaptureSnapshotError, DriverLanes, PublicationDecision,
    },
};

mod event_log;
mod scheduler;
#[cfg(test)]
mod tests;
mod watch;

use event_log::{
    DriverEventDetails, append_terminal_events, submission_events, suppress_watch_replay_events,
};
use scheduler::{RegistrySchedulerDispatcher, dispatch_gap_phases, scheduler_outcome};
use watch::{
    cancellation_outcome, replace_watch_snapshot, request_origin_name, submit_error_session,
    watch_mode_gaps,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildSubmission {
    pub session: BuildSession,
    pub build_plan: Option<BuildPlan>,
    pub module_index: Option<ModuleIndex>,
    pub task_graph: Option<TaskGraph>,
    pub scheduler_run: Option<DriverSchedulerRun>,
    pub missing_services: Vec<DriverMissingPhaseService>,
    pub dispatch_gap_phases: Vec<PipelinePhase>,
    pub status: DriverSubmissionStatus,
    pub publication_decision: PublicationDecision,
    pub diagnostic_index: Option<mizar_diagnostics::aggregator::BuildDiagnosticIndex>,
}

/// A native checkpoint whose matching cancellation return was validated.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NativeCheckpointObservation {
    pub session: BuildSessionId,
    pub task: TaskId,
    pub checkpoint: NativeCheckpoint,
    pub input_snapshot: BuildSnapshotId,
    pub cancellation: CancellationToken,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum NativeCheckpoint {
    FrontendServiceEntry,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum NativeCheckpointObservations<'a> {
    UnknownSession,
    Disabled,
    Enabled(&'a [NativeCheckpointObservation]),
}

/// Submission choices shared by draft, captured, and watch lifecycle entries.
#[derive(Clone, Copy, Default)]
pub struct SubmitOptions<'a> {
    pub discover_imports: bool,
    pub diagnostic_index: Option<DiagnosticIndexMode>,
    pub native_cache: Option<NativeCacheOptions<'a>>,
    pub retained_frontend: Option<(
        &'a crate::registry::PhaseInput,
        &'a mizar_ir::cache_adapter::IrCacheAdapter,
        &'a crate::registry::PhaseResult,
    )>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum DiagnosticIndexMode {
    Disk,
    Native,
}

#[derive(Clone, Copy)]
pub struct NativeCacheOptions<'a> {
    pub policy: &'a mizar_proof::policy::VerifierPolicy,
    pub store: &'a mizar_cache::cache_store::CacheStoreRoot,
    pub mode: NativeCacheMode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum NativeCacheMode {
    ReadOnly,
    ConfiguredReadOnly,
    PersistentSource,
    PersistentPrefix,
}

impl SubmitOptions<'_> {
    fn validate(self, captured: bool, watch: bool) -> Result<(), DriverSubmitError> {
        let invalid = if watch {
            self.discover_imports
                || self.diagnostic_index.is_some()
                || self.native_cache.is_some()
                || self.retained_frontend.is_some()
        } else if !captured {
            self.diagnostic_index.is_some()
                || self.native_cache.is_some()
                || self.retained_frontend.is_some()
        } else if let Some(cache) = self.native_cache {
            self.discover_imports
                || self.diagnostic_index.is_some()
                || (self.retained_frontend.is_some() && cache.mode != NativeCacheMode::ReadOnly)
        } else {
            self.retained_frontend.is_some()
                || (self.discover_imports
                    && self.diagnostic_index == Some(DiagnosticIndexMode::Disk))
        };
        if invalid {
            Err(DriverSubmitError::InvalidSubmitOptions {
                reason: "unsupported submission option combination",
            })
        } else {
            Ok(())
        }
    }
}

pub struct DriverSubmitInput<L>
where
    L: SourceLayoutProvider,
{
    pub plan_request: PlanRequest,
    pub workspace_packages: Vec<WorkspacePackage>,
    pub lockfile: Lockfile,
    pub source_layout: L,
    pub dependency_artifacts: Vec<DependencyArtifactIndex>,
    pub dependency_overlay: ModuleDependencyOverlay,
    pub vc_descriptors: Vec<VcTaskDescriptor>,
    pub task_graph_profile: TaskGraphProfile,
    pub scheduler_mode: SchedulerMode,
    pub priority_hints: PriorityHints,
    pub cache_policy: CacheSchedulingPolicy,
    pub cache_decisions: CacheSchedulingPlan,
    pub resource_budget: ResourceBudget,
    pub cancellation: CancellationPolicy,
    pub phase_dispatch_inputs: Option<Box<dyn PhaseDispatchInputProvider<BuildTask>>>,
    pub worker_count: usize,
    pub completion_order: CompletionOrder,
}

pub struct CompilerDriver {
    sessions: HashMap<BuildSessionId, DriverSessionRecord>,
    lanes: DriverLanes,
    executions: HashMap<BuildSessionId, DriverExecution>,
    registry: PhaseRegistry,
    output_publisher: Option<Arc<PhaseOutputPublisher>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WatchSubmission {
    pub submission: BuildSubmission,
    pub superseded: Option<WatchSupersededSession>,
    pub snapshot_replacement: WatchSnapshotReplacement,
    pub gaps: Vec<WatchModeGap>,
}

#[derive(Debug, Clone, Copy)]
pub struct WatchSubmitControl<'a> {
    pub previous_session: Option<BuildSessionId>,
    pub output_publisher: Option<&'a PhaseOutputPublisher>,
    pub file_watcher: WatchOwnerSeam,
    pub lsp_bridge: WatchOwnerSeam,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WatchSupersededSession {
    pub session: BuildSessionId,
    pub snapshot: BuildSnapshotId,
    pub previous_state: BuildSessionState,
    pub state: BuildSessionState,
    pub publication_decision: PublicationDecision,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WatchSnapshotReplacement {
    pub old_snapshot: Option<BuildSnapshotId>,
    pub new_snapshot: BuildSnapshotId,
    pub status: WatchSnapshotReplacementStatus,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum WatchSnapshotReplacementStatus {
    RegisteredInitialSnapshot,
    Replaced,
    SameSnapshot,
    ExternalDependencyGap,
    Failed { error: Box<PublishError> },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WatchModeGap {
    pub owner: WatchModeGapOwner,
    pub classification: OwnerGapClassification,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum WatchModeGapOwner {
    FileWatcher,
    LspBridge,
    IrSnapshotReplacement,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum WatchOwnerSeam {
    OwnerProvided,
    ExternalDependencyGap,
    Deferred,
    Unavailable,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum WatchSubmitError {
    NonWatchRequest {
        origin: &'static str,
    },
    PreviousSessionUnknown {
        session: BuildSessionId,
    },
    PreviousSessionLaneMismatch {
        session: BuildSessionId,
    },
    PreviousSessionNotWatch {
        session: BuildSessionId,
    },
    PreviousSessionNotCurrent {
        session: BuildSessionId,
        current: BuildSessionId,
    },
    NonMonotonicGeneration {
        session: BuildSessionId,
    },
    Submit(Box<WatchSubmitFailure>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WatchSubmitFailure {
    pub error: Box<DriverSubmitError>,
    pub superseded: Option<WatchSupersededSession>,
    pub snapshot_replacement: Option<WatchSnapshotReplacement>,
    pub gaps: Vec<WatchModeGap>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct WatchPreviousSession {
    session: Option<BuildSessionId>,
    snapshot: Option<BuildSnapshotId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct DriverSessionRecord {
    session: BuildSession,
    cancellation: CancellationPolicy,
    events: BuildEventStream,
    terminal_submission: Option<BuildSubmission>,
    native_checkpoint_observations: Option<Vec<NativeCheckpointObservation>>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum SubmissionMode {
    Draft,
    Captured,
    Deferred,
}

struct DriverExecution {
    scheduler: SchedulerExecution,
    output_publisher: Option<Arc<PhaseOutputPublisher>>,
    build_plan: BuildPlan,
    module_index: ModuleIndex,
    task_graph: TaskGraph,
    missing_services: Vec<DriverMissingPhaseService>,
    phase_inputs: Option<Box<dyn PhaseDispatchInputProvider<BuildTask>>>,
    phase_results: BTreeMap<TaskId, Vec<PhaseResult>>,
    diagnostic_index: Option<DiagnosticIndexMode>,
    cached_sources: Vec<(
        TaskId,
        mizar_ir::identity::PhaseOutputId,
        mizar_ir::storage::RetainOwner,
    )>,
    pending_sources: Vec<(TaskId, crate::registry::PhaseInput)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DriverSchedulerRun {
    pub phase_results: BTreeMap<TaskId, Vec<PhaseResult>>,
    pub task_states: Vec<TaskStateRecord>,
    pub events: Vec<SchedulerEvent>,
    pub diagnostics: Vec<SchedulerDiagnostic>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DriverMissingPhaseService {
    pub phase: PipelinePhase,
    pub availability: PhaseServiceAvailability,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DriverCancelOutcome {
    pub session: BuildSessionId,
    pub changed: bool,
    pub state: Option<BuildSessionState>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum DriverSubmissionStatus {
    SourceLoadFailed,
    SchedulerValidated,
    BlockedByMissingPhaseServices,
    BlockedByPhaseDispatchGap,
    SupersededBeforeSubmission,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum DriverCancelReason {
    ExplicitRequest,
    Superseded,
    Shutdown,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum DriverSubmitError {
    InvalidSubmitOptions {
        reason: &'static str,
    },
    NativeSourceVersionsProvided {
        request: Box<BuildRequestDraft>,
    },
    InvalidCapturedSession {
        session: Box<BuildSession>,
    },
    RequestIdAllocation {
        error: IdError,
    },
    SnapshotCapture {
        error: Box<CaptureSnapshotError>,
    },
    Planning {
        session: Box<BuildSession>,
        diagnostics: ManifestDiagnostics,
    },
    ModuleIndex {
        session: Box<BuildSession>,
        diagnostics: ModuleIndexDiagnostics,
    },
    TaskGraph {
        session: Box<BuildSession>,
        diagnostics: TaskGraphDiagnostics,
    },
    PhaseRegistry {
        session: Box<BuildSession>,
        error: PhaseRegistryError,
    },
    Scheduler {
        session: Box<BuildSession>,
        diagnostics: SchedulerDiagnostics,
    },
}

impl CompilerDriver {
    pub fn new(registry: PhaseRegistry) -> Self {
        Self {
            sessions: HashMap::new(),
            lanes: DriverLanes::default(),
            executions: HashMap::new(),
            registry,
            output_publisher: None,
        }
    }

    /// Supplies the owner-managed publisher shared by scheduled phase services.
    pub fn with_output_publisher(mut self, publisher: Arc<PhaseOutputPublisher>) -> Self {
        self.output_publisher = Some(publisher);
        self
    }

    pub fn registry(&self) -> &PhaseRegistry {
        &self.registry
    }

    pub fn session(&self, session: BuildSessionId) -> Option<&BuildSession> {
        self.sessions.get(&session).map(|record| &record.session)
    }

    pub fn cancellation_policy(&self, session: BuildSessionId) -> Option<&CancellationPolicy> {
        self.sessions
            .get(&session)
            .map(|record| &record.cancellation)
    }

    /// Reads retained native cancellation receipts without advancing execution.
    pub fn native_checkpoint_observations(
        &self,
        session: BuildSessionId,
    ) -> NativeCheckpointObservations<'_> {
        match self.sessions.get(&session) {
            None => NativeCheckpointObservations::UnknownSession,
            Some(record) => match &record.native_checkpoint_observations {
                None => NativeCheckpointObservations::Disabled,
                Some(observations) => NativeCheckpointObservations::Enabled(observations),
            },
        }
    }

    pub fn submit_watch_change<A, L>(
        &mut self,
        request: BuildRequestDraft,
        allocator: &A,
        snapshots: &SnapshotRegistry<A>,
        input: DriverSubmitInput<L>,
        control: WatchSubmitControl<'_>,
        options: SubmitOptions<'_>,
    ) -> Result<WatchSubmission, WatchSubmitError>
    where
        A: SessionIdAllocator,
        L: SourceLayoutProvider,
    {
        options.validate(false, true).map_err(|error| {
            WatchSubmitError::Submit(Box::new(WatchSubmitFailure {
                error: Box::new(error),
                superseded: None,
                snapshot_replacement: None,
                gaps: watch_mode_gaps(&control),
            }))
        })?;
        if !matches!(&request.origin, BuildRequestOrigin::Watch(_)) {
            return Err(WatchSubmitError::NonWatchRequest {
                origin: request_origin_name(&request.origin),
            });
        }

        let previous_watch = self.validate_watch_previous(control.previous_session, &request)?;
        let gaps = watch_mode_gaps(&control);
        let submission = match self.submit(request, allocator, snapshots, input, options) {
            Ok(submission) => submission,
            Err(error) => {
                let captured = submit_error_session(&error).cloned();
                let (superseded, snapshot_replacement) = if let Some(session) = captured.as_ref() {
                    (
                        self.supersede_previous_watch_session(previous_watch.session, snapshots),
                        Some(replace_watch_snapshot(
                            control.output_publisher,
                            previous_watch.snapshot,
                            session,
                        )),
                    )
                } else {
                    (None, None)
                };
                return Err(WatchSubmitError::Submit(Box::new(WatchSubmitFailure {
                    error: Box::new(error),
                    superseded,
                    snapshot_replacement,
                    gaps,
                })));
            }
        };
        let superseded = self.supersede_previous_watch_session(previous_watch.session, snapshots);
        let snapshot_replacement = replace_watch_snapshot(
            control.output_publisher,
            previous_watch.snapshot,
            &submission.session,
        );

        Ok(WatchSubmission {
            submission,
            superseded,
            snapshot_replacement,
            gaps,
        })
    }

    pub fn submit<A, L>(
        &mut self,
        request: BuildRequestDraft,
        allocator: &A,
        snapshots: &SnapshotRegistry<A>,
        input: DriverSubmitInput<L>,
        options: SubmitOptions<'_>,
    ) -> Result<BuildSubmission, DriverSubmitError>
    where
        A: SessionIdAllocator,
        L: SourceLayoutProvider,
    {
        options.validate(false, false)?;
        let pending = request
            .allocate(allocator)
            .map_err(|error| DriverSubmitError::RequestIdAllocation { error })?;
        let session = pending
            .capture_snapshot(snapshots)
            .map_err(|error| DriverSubmitError::SnapshotCapture { error })?;
        self.submit_inner(
            session,
            allocator,
            snapshots,
            input,
            options,
            SubmissionMode::Draft,
        )
    }

    pub fn submit_captured<A: SessionIdAllocator, L: SourceLayoutProvider>(
        &mut self,
        session: BuildSession,
        allocator: &A,
        snapshots: &SnapshotRegistry<A>,
        input: DriverSubmitInput<L>,
        options: SubmitOptions<'_>,
    ) -> Result<BuildSubmission, DriverSubmitError> {
        options.validate(true, false)?;
        let mut submission = self.submit_inner(
            session,
            allocator,
            snapshots,
            input,
            options,
            SubmissionMode::Captured,
        )?;
        submission.diagnostic_index = match options.diagnostic_index {
            Some(DiagnosticIndexMode::Disk)
                if submission.status == DriverSubmissionStatus::SourceLoadFailed =>
            {
                None
            }
            Some(DiagnosticIndexMode::Disk) => self.current_diagnostic_index(
                &submission,
                snapshots,
                Self::validate_current_disk_diagnostics,
            ),
            Some(DiagnosticIndexMode::Native) => self.current_diagnostic_index(
                &submission,
                snapshots,
                Self::validate_current_native_diagnostics,
            ),
            None => None,
        };
        if submission.status == DriverSubmissionStatus::SourceLoadFailed {
            self.sessions
                .get_mut(&submission.session.id)
                .expect("accepted session is stored")
                .terminal_submission = Some(submission.clone());
        }
        Ok(submission)
    }

    /// Accepts captured work and yields at a scheduler-owned dispatch boundary.
    pub fn start_captured<A: SessionIdAllocator, L: SourceLayoutProvider>(
        &mut self,
        session: BuildSession,
        allocator: &A,
        snapshots: &SnapshotRegistry<A>,
        input: DriverSubmitInput<L>,
        options: SubmitOptions<'_>,
    ) -> Result<BuildSubmission, DriverSubmitError> {
        options.validate(true, false)?;
        if options.native_cache.is_some() || options.retained_frontend.is_some() {
            return Err(DriverSubmitError::InvalidSubmitOptions {
                reason: "nonterminal execution requires owned submission options",
            });
        }
        let mut submission = self.submit_inner(
            session,
            allocator,
            snapshots,
            input,
            options,
            SubmissionMode::Deferred,
        )?;
        if submission.session.is_terminal() {
            submission.diagnostic_index = options.diagnostic_index.and_then(|mode| {
                if mode == DiagnosticIndexMode::Disk
                    && submission.status == DriverSubmissionStatus::SourceLoadFailed
                {
                    return None;
                }
                self.current_diagnostic_index(
                    &submission,
                    snapshots,
                    match mode {
                        DiagnosticIndexMode::Disk => Self::validate_current_disk_diagnostics,
                        DiagnosticIndexMode::Native => Self::validate_current_native_diagnostics,
                    },
                )
            });
            self.sessions
                .get_mut(&submission.session.id)
                .expect("accepted session is stored")
                .terminal_submission = Some(submission.clone());
        }
        Ok(submission)
    }

    /// Starts captured work with native checkpoint observations enabled before progress.
    pub fn start_captured_observed<A: SessionIdAllocator, L: SourceLayoutProvider>(
        &mut self,
        session: BuildSession,
        allocator: &A,
        snapshots: &SnapshotRegistry<A>,
        input: DriverSubmitInput<L>,
        options: SubmitOptions<'_>,
    ) -> Result<BuildSubmission, DriverSubmitError> {
        let submission = self.start_captured(session, allocator, snapshots, input, options)?;
        self.sessions
            .get_mut(&submission.session.id)
            .expect("accepted session is stored")
            .native_checkpoint_observations = Some(Vec::new());
        Ok(submission)
    }

    /// The actual admitted task whose service will execute on a later progress call.
    pub fn pending_task(&self, session: BuildSessionId) -> Option<&BuildTask> {
        self.executions.get(&session)?.scheduler.pending_task()
    }

    /// Advances one owner dispatch boundary, returning a terminal submission after drainage.
    pub fn progress<A: SessionIdAllocator>(
        &mut self,
        session: BuildSessionId,
        allocator: &A,
        snapshots: &SnapshotRegistry<A>,
    ) -> Result<Option<BuildSubmission>, DriverSubmitError> {
        let Some(mut execution) = self.executions.remove(&session) else {
            return Ok(self.terminal_submission(session, snapshots));
        };
        let record = self
            .sessions
            .get_mut(&session)
            .expect("execution has a session");
        if self.lanes.publication_decision(snapshots, &record.session)
            != PublicationDecision::Current
        {
            execution
                .scheduler
                .cancellation_signal()
                .request(CancellationReason::SupersededSnapshot);
            record.session.cancel();
        }
        let session_inputs = record.session.clone();
        let mut dispatcher = RegistrySchedulerDispatcher::new(
            &self.registry,
            execution.phase_inputs.as_deref(),
            execution.output_publisher.as_ref(),
            Some(crate::registry::SourceLoadInputs {
                snapshot: &session_inputs.captured.snapshot,
                captured: Some(&session_inputs.captured),
                build_plan: &execution.build_plan,
                module_index: &execution.module_index,
                allocator,
            }),
        );
        dispatcher.native_checkpoint_observations = record
            .native_checkpoint_observations
            .as_mut()
            .map(|observations| (session, observations));
        dispatcher.phase_results = execution.phase_results;
        execution.scheduler.advance(&mut dispatcher);
        execution.phase_results = dispatcher.phase_results;
        if !execution.scheduler.is_finished() {
            self.executions.insert(session, execution);
            return Ok(None);
        }
        let diagnostic_index = execution.diagnostic_index;
        let record = self
            .sessions
            .get(&session)
            .expect("execution has a session");
        let mut submission = self.finish_submission(
            record.session.clone(),
            allocator,
            snapshots,
            record.cancellation.clone(),
            execution,
            None,
        )?;
        submission.diagnostic_index = diagnostic_index.and_then(|mode| {
            self.current_diagnostic_index(
                &submission,
                snapshots,
                match mode {
                    DiagnosticIndexMode::Disk => Self::validate_current_disk_diagnostics,
                    DiagnosticIndexMode::Native => Self::validate_current_native_diagnostics,
                },
            )
        });
        self.sessions
            .get_mut(&session)
            .expect("finished session is stored")
            .terminal_submission = Some(submission);
        Ok(self.terminal_submission(session, snapshots))
    }

    /// Replays a drained submission with current publication and output guards.
    pub fn terminal_submission<A: SessionIdAllocator>(
        &self,
        session: BuildSessionId,
        snapshots: &SnapshotRegistry<A>,
    ) -> Option<BuildSubmission> {
        let record = self.sessions.get(&session)?;
        let mut submission = record.terminal_submission.clone()?;
        submission.session = record.session.clone();
        submission.publication_decision =
            self.lanes.publication_decision(snapshots, &record.session);
        let valid = submission.publication_decision == PublicationDecision::Current
            && self.output_publisher.as_ref().is_none_or(|publisher| {
                publisher
                    .validate_current_snapshot(record.session.captured.snapshot.id)
                    .is_ok()
                    && submission.scheduler_run.as_ref().is_none_or(|run| {
                        run.phase_results.values().flatten().all(|result| {
                            result.output_refs.iter().all(|output| {
                                publisher
                                    .validate_output(
                                        record.session.captured.snapshot.id,
                                        output,
                                        if record.session.captured.native_sources().is_some() {
                                            OutputAuthorization::Published
                                        } else {
                                            OutputAuthorization::CurrentPackage
                                        },
                                    )
                                    .is_ok()
                                    && publisher.storage().validate_handle(output).is_ok()
                            })
                        })
                    })
            });
        if !valid {
            submission.diagnostic_index = None;
            if let Some(run) = &mut submission.scheduler_run {
                run.phase_results.clear();
            }
        }
        Some(submission)
    }

    /// Checks live session currentness for a native Disk diagnostic handoff.
    /// This does not authenticate caller-owned batches or authorize a later publication.
    pub fn validate_current_disk_diagnostics<A: SessionIdAllocator>(
        &self,
        session: BuildSessionId,
        snapshots: &SnapshotRegistry<A>,
    ) -> Result<(), String> {
        let record = self
            .sessions
            .get(&session)
            .ok_or_else(|| "unknown diagnostic session".to_owned())?;
        if !record
            .session
            .captured
            .snapshot
            .source_versions
            .iter()
            .all(|version| matches!(version.origin, mizar_session::SourceOrigin::Disk))
        {
            return Err("unsupported diagnostic session".into());
        }
        self.validate_current_native_diagnostics(session, snapshots)
    }

    /// Checks live session currentness for a native Disk/OpenBuffer diagnostic handoff.
    /// This does not authenticate caller-owned batches or authorize a later publication.
    pub fn validate_current_native_diagnostics<A: SessionIdAllocator>(
        &self,
        session: BuildSessionId,
        snapshots: &SnapshotRegistry<A>,
    ) -> Result<(), String> {
        let record = self
            .sessions
            .get(&session)
            .ok_or_else(|| "unknown diagnostic session".to_owned())?;
        let session = &record.session;
        let captured = &session.captured;
        if captured.native_sources().is_none()
            || captured.snapshot.source_versions.is_empty()
            || !captured.snapshot.source_versions.iter().all(|version| {
                matches!(
                    version.origin,
                    mizar_session::SourceOrigin::Disk
                        | mizar_session::SourceOrigin::OpenBuffer { .. }
                )
            })
            || !matches!(
                session.state,
                BuildSessionState::Finished(
                    BuildSessionOutcome::Succeeded
                        | BuildSessionOutcome::Failed
                        | BuildSessionOutcome::Blocked
                )
            )
        {
            return Err("unsupported diagnostic session".into());
        }
        if self.lanes.publication_decision(snapshots, session) != PublicationDecision::Current {
            return Err("obsolete diagnostic session".into());
        }
        self.output_publisher
            .as_ref()
            .ok_or_else(|| "diagnostic publisher required".to_owned())?
            .validate_current_snapshot(captured.snapshot.id)
            .map_err(|error| error.to_string())
    }

    fn current_diagnostic_index<A: SessionIdAllocator>(
        &self,
        submission: &BuildSubmission,
        snapshots: &SnapshotRegistry<A>,
        validate: fn(&Self, BuildSessionId, &SnapshotRegistry<A>) -> Result<(), String>,
    ) -> Option<mizar_diagnostics::aggregator::BuildDiagnosticIndex> {
        validate(self, submission.session.id, snapshots).ok()?;
        let batches = if submission.status == DriverSubmissionStatus::SourceLoadFailed {
            use mizar_diagnostics::{
                failure_record::PipelinePhase,
                sink::{DiagnosticProducerScope, DiagnosticSink},
            };
            let mut sink = DiagnosticSink::new(DiagnosticProducerScope::new(
                PipelinePhase::SourceLoad,
                submission.session.captured.snapshot.id,
                "SourceLoad",
            ));
            for (input, error) in submission.session.captured.source_load_failures() {
                let draft = crate::frontend_adapter::SourceLoadService::failure_draft(
                    submission.session.captured.snapshot.id,
                    input,
                    error,
                )?;
                sink.emit(draft).ok()?;
            }
            vec![sink.seal()]
        } else {
            submission
                .scheduler_run
                .as_ref()?
                .phase_results
                .values()
                .flatten()
                .flat_map(|result| result.diagnostics.iter().cloned())
                .collect()
        };
        let index = mizar_diagnostics::aggregator::BuildDiagnosticIndex::from_batches(
            submission.session.captured.snapshot.id,
            batches,
        )
        .ok()?;
        validate(self, submission.session.id, snapshots).ok()?;
        Some(index)
    }

    fn submit_inner<A, L>(
        &mut self,
        mut session: BuildSession,
        allocator: &A,
        snapshots: &SnapshotRegistry<A>,
        input: DriverSubmitInput<L>,
        options: SubmitOptions<'_>,
        mode: SubmissionMode,
    ) -> Result<BuildSubmission, DriverSubmitError>
    where
        A: SessionIdAllocator,
        L: SourceLayoutProvider,
    {
        let SubmitOptions {
            discover_imports,
            native_cache,
            retained_frontend: oracle,
            ..
        } = options;
        if mode != SubmissionMode::Draft
            && (session.captured.native_sources().is_none()
                || session.state != BuildSessionState::SnapshotCaptured
                || self.sessions.contains_key(&session.id)
                || !snapshots
                    .is_current_for_request(session.captured.snapshot.id, session.request.id))
        {
            return Err(DriverSubmitError::InvalidCapturedSession {
                session: Box::new(session),
            });
        }
        if !self.lanes.mark_current(&session) {
            session.finish(BuildSessionOutcome::Superseded);
            let publication_decision = self.lanes.publication_decision(snapshots, &session);
            let events = submission_events(
                &session,
                publication_decision,
                DriverEventDetails::default(),
            );
            self.store_session(session.clone(), input.cancellation, events);
            return Ok(BuildSubmission {
                session,
                build_plan: None,
                module_index: None,
                task_graph: None,
                scheduler_run: None,
                missing_services: Vec::new(),
                dispatch_gap_phases: Vec::new(),
                status: DriverSubmissionStatus::SupersededBeforeSubmission,
                publication_decision,
                diagnostic_index: None,
            });
        }

        if !session.captured.source_load_failures().is_empty() {
            let snapshot = session.captured.snapshot.id;
            let superseded = input
                .cancellation
                .superseded_snapshots()
                .contains(&snapshot)
                || input
                    .cancellation
                    .current_snapshot()
                    .is_some_and(|current| current != snapshot);
            session.finish(if superseded {
                BuildSessionOutcome::Superseded
            } else {
                BuildSessionOutcome::Failed
            });
            let publication_decision = self.lanes.publication_decision(snapshots, &session);
            let events = submission_events(
                &session,
                publication_decision,
                DriverEventDetails::default(),
            );
            self.store_session(session.clone(), input.cancellation, events);
            return Ok(BuildSubmission {
                session,
                build_plan: None,
                module_index: None,
                task_graph: None,
                scheduler_run: None,
                missing_services: Vec::new(),
                dispatch_gap_phases: Vec::new(),
                status: if superseded {
                    DriverSubmissionStatus::SupersededBeforeSubmission
                } else {
                    DriverSubmissionStatus::SourceLoadFailed
                },
                publication_decision,
                diagnostic_index: None,
            });
        }

        let DriverSubmitInput {
            plan_request,
            workspace_packages,
            lockfile,
            source_layout,
            dependency_artifacts,
            dependency_overlay,
            vc_descriptors,
            task_graph_profile,
            scheduler_mode,
            priority_hints,
            cache_policy,
            cache_decisions,
            resource_budget,
            cancellation,
            phase_dispatch_inputs,
            worker_count,
            completion_order,
        } = input;

        let build_plan = match produce_build_plan(plan_request, workspace_packages, lockfile) {
            Ok(build_plan) => build_plan,
            Err(diagnostics) => {
                session.finish(BuildSessionOutcome::Failed);
                let failed_session = session.clone();
                let publication_decision = self.lanes.publication_decision(snapshots, &session);
                let events = submission_events(
                    &session,
                    publication_decision,
                    DriverEventDetails {
                        planning: Some(PlanningEventStatus::Failed),
                        ..DriverEventDetails::default()
                    },
                );
                self.store_session(session, cancellation, events);
                return Err(DriverSubmitError::Planning {
                    session: Box::new(failed_session),
                    diagnostics,
                });
            }
        };
        let module_index =
            match build_module_index(&build_plan, &source_layout, &dependency_artifacts) {
                Ok(module_index) => module_index,
                Err(diagnostics) => {
                    session.finish(BuildSessionOutcome::Failed);
                    let failed_session = session.clone();
                    let publication_decision = self.lanes.publication_decision(snapshots, &session);
                    let events = submission_events(
                        &session,
                        publication_decision,
                        DriverEventDetails {
                            planning: Some(PlanningEventStatus::Failed),
                            ..DriverEventDetails::default()
                        },
                    );
                    self.store_session(session, cancellation, events);
                    return Err(DriverSubmitError::ModuleIndex {
                        session: Box::new(failed_session),
                        diagnostics,
                    });
                }
            };
        let dependency_overlay = if discover_imports
            && dependency_overlay.coverage
                == mizar_build::task_graph::ModuleDependencyCoverage::Unavailable
            && dependency_overlay.edges.is_empty()
        {
            crate::registry::SourceLoadInputs {
                snapshot: &session.captured.snapshot,
                captured: Some(&session.captured),
                build_plan: &build_plan,
                module_index: &module_index,
                allocator,
            }
            .discover_import_overlay()
            .unwrap_or(dependency_overlay)
        } else {
            dependency_overlay
        };
        let mut task_graph = match build_task_graph(TaskGraphInput {
            graph_version: mizar_build::task_graph::TaskGraphVersion::current(),
            snapshot: session.captured.snapshot.id,
            build_plan: build_plan.clone(),
            module_index: module_index.clone(),
            dependency_overlay,
            vc_descriptors,
            profile: task_graph_profile,
        }) {
            Ok(task_graph) => task_graph,
            Err(diagnostics) => {
                session.finish(BuildSessionOutcome::Failed);
                let failed_session = session.clone();
                let publication_decision = self.lanes.publication_decision(snapshots, &session);
                let events = submission_events(
                    &session,
                    publication_decision,
                    DriverEventDetails {
                        planning: Some(PlanningEventStatus::Failed),
                        ..DriverEventDetails::default()
                    },
                );
                self.store_session(session, cancellation, events);
                return Err(DriverSubmitError::TaskGraph {
                    session: Box::new(failed_session),
                    diagnostics,
                });
            }
        };

        session.mark_submitted();

        let missing_services = match self.missing_services(&task_graph) {
            Ok(missing_services) => missing_services,
            Err(error) => {
                session.finish(BuildSessionOutcome::Failed);
                let failed_session = session.clone();
                let publication_decision = self.lanes.publication_decision(snapshots, &session);
                let events = submission_events(
                    &session,
                    publication_decision,
                    DriverEventDetails {
                        planning: Some(PlanningEventStatus::Failed),
                        ..DriverEventDetails::default()
                    },
                );
                self.store_session(session, cancellation, events);
                return Err(DriverSubmitError::PhaseRegistry {
                    session: Box::new(failed_session),
                    error,
                });
            }
        };
        let reflexivity_profile;
        let reflexivity_artifact_profile;
        let reflexivity_discharge_profile;
        let built_in_prefix = {
            use crate::{
                frontend_adapter::{
                    FrontendService, SourceLoadService, SourceTheoremAlgorithmPreparationService,
                    SourceTheoremArtifactCommitService, SourceTheoremAtpSolveService,
                    SourceTheoremCandidateCheckService, SourceTheoremElaborationService,
                    SourceTheoremModuleResolverService, SourceTheoremOverloadResolutionService,
                    SourceTheoremReflexivityArtifactCommitService,
                    SourceTheoremReflexivityKernelCheckService,
                    SourceTheoremRegistrationResolutionService,
                    SourceTheoremSignatureCollectionService, SourceTheoremTypeCheckingService,
                    SourceTheoremVcDischargeService, SourceTheoremVcGenerationService,
                },
                registry::PhaseService,
            };
            reflexivity_discharge_profile = self.registry.descriptors().len() == 13
                && self
                    .registry
                    .service_for_phase(PipelinePhase::VcDischarge)
                    .is_ok_and(|service| {
                        std::any::Any::type_id(service)
                            == std::any::TypeId::of::<SourceTheoremVcDischargeService>()
                            && self
                                .registry
                                .descriptor_for_phase(PipelinePhase::VcDischarge)
                                .ok()
                                == Some(&service.phase())
                    })
                && {
                    use mizar_build::task_graph::{TaskKind, WorkUnit};
                    let mut modules = std::collections::BTreeSet::new();
                    task_graph.tasks().iter().filter(|task| task.kind == TaskKind::VcDischarge).all(|task| {
                        let WorkUnit::Vc { module, .. } = &task.unit else { return false; };
                        task.phases == [PipelinePhase::VcDischarge]
                            && modules.insert((module.package.as_str(), module.path.as_str()))
                            && task.dependencies.len() == 2
                            && task_graph.tasks().iter().any(|producer| {
                                task.dependencies.contains(&producer.id)
                                    && producer.kind == TaskKind::VcGenerate
                                    && producer.phases == [PipelinePhase::VcGenerate]
                                    && matches!(&producer.unit, WorkUnit::Module { module: parent } if parent == module)
                            })
                            && task_graph.tasks().iter().any(|producer| {
                                task.dependencies.contains(&producer.id)
                                    && producer.kind == TaskKind::PackageResolve
                                    && producer.unit == WorkUnit::Workspace
                                    && producer.phases == [PipelinePhase::PackageResolve]
                            })
                    })
                };
            reflexivity_artifact_profile =
                (self.registry.descriptors().len() == 12 || reflexivity_discharge_profile)
                    && self
                        .registry
                        .service_for_phase(PipelinePhase::ArtifactCommit)
                        .is_ok_and(|service| {
                            std::any::Any::type_id(service)
                                == std::any::TypeId::of::<
                                    SourceTheoremReflexivityArtifactCommitService,
                                >()
                                && self
                                    .registry
                                    .descriptor_for_phase(PipelinePhase::ArtifactCommit)
                                    .ok()
                                    == Some(&service.phase())
                        });
            reflexivity_profile = (self.registry.descriptors().len() == 11
                || reflexivity_artifact_profile)
                && self
                    .registry
                    .service_for_phase(PipelinePhase::KernelCheck)
                    .is_ok_and(|service| {
                        std::any::Any::type_id(service)
                            == std::any::TypeId::of::<SourceTheoremReflexivityKernelCheckService>()
                            && self
                                .registry
                                .descriptor_for_phase(PipelinePhase::KernelCheck)
                                .ok()
                                == Some(&service.phase())
                    })
                && [
                    (
                        PipelinePhase::SourceLoad,
                        std::any::TypeId::of::<SourceLoadService>(),
                    ),
                    (
                        PipelinePhase::Frontend,
                        std::any::TypeId::of::<FrontendService>(),
                    ),
                ]
                .into_iter()
                .all(|(phase, concrete)| {
                    self.registry
                        .service_for_phase(phase)
                        .is_ok_and(|service| std::any::Any::type_id(service) == concrete)
                })
                && {
                    use mizar_build::task_graph::{TaskKind, WorkUnit};
                    let mut modules = std::collections::BTreeSet::new();
                    !task_graph.tasks().iter().any(|task| matches!(task.kind, TaskKind::BackendRun | TaskKind::AtpSolve))
                        && task_graph.tasks().iter().filter(|task| task.kind == TaskKind::KernelCheck).all(|task| {
                            let WorkUnit::EvidenceCandidate { module, descriptor, .. } = &task.unit else { return false; };
                            task.phases == [PipelinePhase::KernelCheck]
                                && modules.insert((module.package.as_str(), module.path.as_str()))
                                && task.dependencies.len() == 2
                                && task_graph.tasks().iter().any(|producer| {
                                    task.dependencies.contains(&producer.id)
                                        && producer.kind == TaskKind::VcDischarge
                                        && producer.phases == [PipelinePhase::VcDischarge]
                                        && matches!(&producer.unit, WorkUnit::Vc { module: parent_module, descriptor: parent_descriptor }
                                            if parent_module == module && parent_descriptor == descriptor)
                                })
                                && task_graph.tasks().iter().any(|producer| {
                                    task.dependencies.contains(&producer.id)
                                        && producer.kind == TaskKind::PackageResolve
                                        && producer.unit == WorkUnit::Workspace
                                        && producer.phases == [PipelinePhase::PackageResolve]
                                })
                                && task_graph.tasks().iter().filter(|producer| {
                                    producer.kind == TaskKind::VcGenerate
                                        && producer.phases == [PipelinePhase::VcGenerate]
                                        && matches!(&producer.unit, WorkUnit::Module { module: parent_module } if parent_module == module)
                                }).count() == 1
                        })
                };
            let artifact_profile = self.registry.descriptors().len() == 14
                && self
                    .registry
                    .service_for_phase(PipelinePhase::ArtifactCommit)
                    .is_ok_and(|service| {
                        std::any::Any::type_id(service)
                            == std::any::TypeId::of::<SourceTheoremArtifactCommitService>()
                            && self
                                .registry
                                .descriptor_for_phase(PipelinePhase::ArtifactCommit)
                                .ok()
                                == Some(&service.phase())
                    })
                && [
                    (
                        PipelinePhase::SourceLoad,
                        std::any::TypeId::of::<SourceLoadService>(),
                    ),
                    (
                        PipelinePhase::Frontend,
                        std::any::TypeId::of::<FrontendService>(),
                    ),
                ]
                .into_iter()
                .all(|(phase, concrete)| {
                    self.registry
                        .service_for_phase(phase)
                        .is_ok_and(|service| std::any::Any::type_id(service) == concrete)
                });
            let kernel_profile = (self.registry.descriptors().len() == 13 || artifact_profile)
                && self
                    .registry
                    .service_for_phase(PipelinePhase::KernelCheck)
                    .is_ok_and(|service| {
                        std::any::Any::type_id(service)
                            == std::any::TypeId::of::<SourceTheoremCandidateCheckService>()
                            && self
                                .registry
                                .descriptor_for_phase(PipelinePhase::KernelCheck)
                                .ok()
                                == Some(&service.phase())
                    })
                && {
                    let mut modules = std::collections::BTreeSet::new();
                    task_graph
                        .tasks()
                        .iter()
                        .filter(|task| task.kind == mizar_build::task_graph::TaskKind::KernelCheck)
                        .all(|task| {
                            let mizar_build::task_graph::WorkUnit::EvidenceCandidate {
                                module, ..
                            } = &task.unit
                            else {
                                return false;
                            };
                            modules.insert((module.package.as_str(), module.path.as_str()))
                        })
                };
            let atp_profile = (self.registry.descriptors().len() == 12 || kernel_profile)
                && self
                    .registry
                    .service_for_phase(PipelinePhase::AtpSolve)
                    .is_ok_and(|service| {
                        std::any::Any::type_id(service)
                            == std::any::TypeId::of::<SourceTheoremAtpSolveService>()
                            && self
                                .registry
                                .descriptor_for_phase(PipelinePhase::AtpSolve)
                                .ok()
                                == Some(&service.phase())
                    });
            let discharge_profile = (self.registry.descriptors().len() == 11 || atp_profile)
                && self
                    .registry
                    .service_for_phase(PipelinePhase::VcDischarge)
                    .is_ok_and(|service| {
                        std::any::Any::type_id(service)
                            == std::any::TypeId::of::<SourceTheoremVcDischargeService>()
                            && self
                                .registry
                                .descriptor_for_phase(PipelinePhase::VcDischarge)
                                .ok()
                                == Some(&service.phase())
                    })
                && {
                    let mut modules = std::collections::BTreeSet::new();
                    task_graph
                        .tasks()
                        .iter()
                        .filter(|task| task.kind == mizar_build::task_graph::TaskKind::VcDischarge)
                        .all(|task| {
                            let mizar_build::task_graph::WorkUnit::Vc { module, .. } = &task.unit
                            else {
                                return false;
                            };
                            modules.insert((module.package.as_str(), module.path.as_str()))
                        })
                };
            let vc_profile = (self.registry.descriptors().len() == 10
                || discharge_profile
                || reflexivity_profile)
                && self
                    .registry
                    .service_for_phase(PipelinePhase::VcGenerate)
                    .is_ok_and(|service| {
                        std::any::Any::type_id(service)
                            == std::any::TypeId::of::<SourceTheoremVcGenerationService>()
                            && self
                                .registry
                                .descriptor_for_phase(PipelinePhase::VcGenerate)
                                .ok()
                                == Some(&service.phase())
                    });
            let preparation_profile = (self.registry.descriptors().len() == 9 || vc_profile)
                && self
                    .registry
                    .descriptor_for_phase(PipelinePhase::AlgorithmPreparation)
                    .ok()
                    == Some(&SourceTheoremAlgorithmPreparationService.phase())
                && self
                    .registry
                    .service_for_phase(PipelinePhase::AlgorithmPreparation)
                    .is_ok_and(|service| {
                        std::any::Any::type_id(service)
                            == std::any::TypeId::of::<SourceTheoremAlgorithmPreparationService>()
                    });
            (self.registry.descriptors().len() == 2
                || ((self.registry.descriptors().len() == 3
                        || ((self.registry.descriptors().len() == 4
                            || ((self.registry.descriptors().len() == 5
                                || ((self.registry.descriptors().len() == 6
                                    || ((self.registry.descriptors().len() == 7
                                        || ((self.registry.descriptors().len() == 8
                                            || preparation_profile)
                                            && self.registry.descriptor_for_phase(PipelinePhase::Elaboration).ok()
                                                == Some(&SourceTheoremElaborationService.phase())
                                            && self.registry.service_for_phase(PipelinePhase::Elaboration).is_ok_and(|service| {
                                                std::any::Any::type_id(service) == std::any::TypeId::of::<SourceTheoremElaborationService>()
                                            })))
                                        && self.registry.descriptor_for_phase(PipelinePhase::OverloadResolution).ok()
                                            == Some(&SourceTheoremOverloadResolutionService.phase())
                                        && self.registry.service_for_phase(PipelinePhase::OverloadResolution).is_ok_and(|service| {
                                            std::any::Any::type_id(service) == std::any::TypeId::of::<SourceTheoremOverloadResolutionService>()
                                        })))
                                    && self.registry.descriptor_for_phase(PipelinePhase::RegistrationResolution).ok()
                                        == Some(&SourceTheoremRegistrationResolutionService.phase())
                                    && self.registry.service_for_phase(PipelinePhase::RegistrationResolution).is_ok_and(|service| {
                                        std::any::Any::type_id(service) == std::any::TypeId::of::<SourceTheoremRegistrationResolutionService>()
                                    })))
                                && self.registry.descriptor_for_phase(PipelinePhase::TypeChecking).ok()
                                    == Some(&SourceTheoremTypeCheckingService.phase())
                                && self.registry.service_for_phase(PipelinePhase::TypeChecking).is_ok_and(|service| {
                                    std::any::Any::type_id(service) == std::any::TypeId::of::<SourceTheoremTypeCheckingService>()
                                })))
                            && self.registry.descriptor_for_phase(PipelinePhase::SignatureCollection).ok()
                                == Some(&SourceTheoremSignatureCollectionService.phase())
                            && self.registry.service_for_phase(PipelinePhase::SignatureCollection).is_ok_and(|service| {
                                std::any::Any::type_id(service) == std::any::TypeId::of::<SourceTheoremSignatureCollectionService>()
                            })))
                    && self
                        .registry
                        .descriptor_for_phase(PipelinePhase::ModuleResolve)
                        .ok()
                        == Some(&SourceTheoremModuleResolverService.phase())
                    && self
                        .registry
                        .service_for_phase(PipelinePhase::ModuleResolve)
                        .is_ok_and(|service| {
                            std::any::Any::type_id(service)
                                == std::any::TypeId::of::<SourceTheoremModuleResolverService>()
                        })))
                && self
                    .registry
                    .descriptor_for_phase(PipelinePhase::SourceLoad)
                    .ok()
                    == Some(&SourceLoadService.phase())
                && self
                    .registry
                    .descriptor_for_phase(PipelinePhase::Frontend)
                    .ok()
                    == Some(
                        &FrontendService {
                            artifact_roots: Vec::new(),
                        }
                        .phase(),
                    )
                && self.output_publisher.as_ref().is_some_and(|publisher| {
                    publisher
                        .validate_current_snapshot(session.captured.snapshot.id)
                        .is_ok()
                })
        };
        if built_in_prefix && reflexivity_profile {
            use mizar_build::task_graph::{TaskEdge, TaskKind, WorkUnit};
            let replacements = task_graph.tasks().iter().filter_map(|task| {
                let module = match (&task.kind, &task.unit) {
                    (TaskKind::KernelCheck, WorkUnit::EvidenceCandidate { module, .. }) => module,
                    (TaskKind::ArtifactCommit, WorkUnit::Module { module }) if reflexivity_artifact_profile => module,
                    (TaskKind::VcDischarge, WorkUnit::Vc { module, .. }) if reflexivity_discharge_profile => module,
                    _ => return None,
                };
                let vc = task_graph.tasks().iter().find(|producer| {
                    producer.kind == TaskKind::VcGenerate
                        && producer.phases == [PipelinePhase::VcGenerate]
                        && matches!(&producer.unit, WorkUnit::Module { module: parent } if parent == module)
                })?;
                let mut dependencies = vec![vc.id.clone()];
                if matches!(task.kind, TaskKind::ArtifactCommit | TaskKind::VcDischarge) {
                    let source = task_graph.tasks().iter().find(|producer| {
                        producer.kind == TaskKind::CheckAndElaborate
                            && vc.dependencies.contains(&producer.id)
                            && matches!(&producer.unit, WorkUnit::Module { module: parent } if parent == module)
                    })?;
                    dependencies.push(source.id.clone());
                }
                if task.kind == TaskKind::ArtifactCommit {
                    let kernel = task_graph.tasks().iter().find(|producer| {
                        producer.kind == TaskKind::KernelCheck
                            && producer.phases == [PipelinePhase::KernelCheck]
                            && matches!(&producer.unit, WorkUnit::EvidenceCandidate { module: parent, .. } if parent == module)
                    })?;
                    dependencies.push(kernel.id.clone());
                }
                dependencies.sort();
                Some((task.id.clone(), dependencies))
            }).collect::<Vec<_>>();
            for (dependent, dependencies) in replacements {
                if let Some(task) = task_graph
                    .tasks
                    .iter_mut()
                    .find(|task| task.id == dependent)
                {
                    task.dependencies = dependencies.clone();
                }
                task_graph.edges.retain(|edge| edge.dependent != dependent);
                task_graph
                    .edges
                    .extend(dependencies.into_iter().map(|dependency| TaskEdge {
                        dependent: dependent.clone(),
                        dependency,
                    }));
            }
            task_graph.edges.sort_by(|left, right| {
                (&left.dependent, &left.dependency).cmp(&(&right.dependent, &right.dependency))
            });
        }
        if !missing_services.is_empty() && !built_in_prefix {
            session.finish(BuildSessionOutcome::Blocked);
            let publication_decision = self.lanes.publication_decision(snapshots, &session);
            let events = submission_events(
                &session,
                publication_decision,
                DriverEventDetails {
                    planning: Some(PlanningEventStatus::Ready),
                    missing_services: &missing_services,
                    ..DriverEventDetails::default()
                },
            );
            self.store_session(session.clone(), cancellation, events);
            return Ok(BuildSubmission {
                session,
                build_plan: Some(build_plan),
                module_index: Some(module_index),
                task_graph: Some(task_graph),
                scheduler_run: None,
                missing_services,
                dispatch_gap_phases: Vec::new(),
                status: DriverSubmissionStatus::BlockedByMissingPhaseServices,
                publication_decision,
                diagnostic_index: None,
            });
        }

        session.mark_running();
        let mut scheduler_input = SchedulerInput::new(task_graph.clone());
        scheduler_input.mode = scheduler_mode;
        scheduler_input.priority_hints = priority_hints;
        scheduler_input.cache = cache_policy;
        scheduler_input.cache_decisions = if native_cache.is_some() {
            CacheSchedulingPlan::empty()
        } else {
            cache_decisions
        };
        if !missing_services.is_empty() {
            use mizar_build::cache_seam::{CacheFallbackReason, CacheSchedulingOutcome};
            for decision in &mut scheduler_input.cache_decisions.decisions {
                if matches!(decision.outcome, CacheSchedulingOutcome::ValidatedHit(_))
                    && task_graph.tasks().iter().any(|task| {
                        task.id == decision.task_id
                            && missing_services
                                .iter()
                                .any(|missing| task.phases.contains(&missing.phase))
                    })
                {
                    decision.outcome =
                        CacheSchedulingOutcome::Unavailable(CacheFallbackReason::Unavailable);
                }
            }
        }
        scheduler_input.resource_budget = resource_budget;
        scheduler_input.cancellation = cancellation.clone();
        scheduler_input.worker_count = worker_count.max(1);
        scheduler_input.completion_order = completion_order;
        let output_publisher = self.output_publisher.as_ref().map(|publisher| {
            if mode == SubmissionMode::Deferred {
                Arc::new(publisher.for_captured_execution(session.captured.snapshot.id))
            } else {
                publisher.clone()
            }
        });
        let mut dispatcher = RegistrySchedulerDispatcher::new(
            &self.registry,
            phase_dispatch_inputs.as_deref(),
            output_publisher.as_ref(),
            Some(crate::registry::SourceLoadInputs {
                snapshot: &session.captured.snapshot,
                captured: Some(&session.captured),
                build_plan: &build_plan,
                module_index: &module_index,
                allocator,
            }),
        );
        let mut cached_sources = Vec::new();
        let mut pending_sources = Vec::new();
        if let Some(NativeCacheOptions {
            policy,
            store,
            mode,
        }) = native_cache
            && let Some(policy) = self.registry.native_source_policy(policy)
            && cache_policy == CacheSchedulingPolicy::Enabled
            && phase_dispatch_inputs.is_none()
            && built_in_prefix
            && session.captured.native_sources().is_some()
            && !session.captured.snapshot.source_versions.is_empty()
            && session
                .captured
                .snapshot
                .source_versions
                .iter()
                .all(|version| version.origin == mizar_session::SourceOrigin::Disk)
        {
            use crate::{
                frontend_adapter::{FrontendService, SourceLoadService},
                registry::{PhaseInput, PhaseStatus, SourceLoadInputs},
            };
            use mizar_build::cache_seam::{
                CacheOutputRef, CacheSchedulingOutcome, CacheTaskDecision, ValidatedCacheHit,
            };
            use mizar_ir::{
                cache_adapter::{CacheRehydrateOutcome, IrCacheAdapter},
                dispatch_input::PhaseDispatchInputBundle,
                storage::RetainOwner,
            };
            let concrete_builtins = self
                .registry
                .service_for_phase(PipelinePhase::SourceLoad)
                .is_ok_and(|service| {
                    std::any::Any::type_id(service) == std::any::TypeId::of::<SourceLoadService>()
                })
                && self
                    .registry
                    .service_for_phase(PipelinePhase::Frontend)
                    .is_ok_and(|service| {
                        std::any::Any::type_id(service) == std::any::TypeId::of::<FrontendService>()
                    });
            let inputs = SourceLoadInputs {
                snapshot: &session.captured.snapshot,
                captured: Some(&session.captured),
                build_plan: &build_plan,
                module_index: &module_index,
                allocator,
            };
            if concrete_builtins
                && inputs.validate_native_lock_content().is_ok()
                && inputs.validate_native_config_content().is_ok()
                && let Some(publisher) = self.output_publisher.as_ref()
            {
                let adapter = IrCacheAdapter::new(publisher.clone());
                for task in task_graph.tasks() {
                    if task.phases != [PipelinePhase::SourceLoad] {
                        continue;
                    }
                    let mizar_build::task_graph::WorkUnit::Module { module } = &task.unit else {
                        continue;
                    };
                    if mode != NativeCacheMode::ReadOnly {
                        let mut packages = build_plan
                            .packages
                            .iter()
                            .filter(|package| package.package_id == module.package);
                        let Some(package) = packages.next() else {
                            continue;
                        };
                        if packages.next().is_some() {
                            continue;
                        }
                        let mizar_build::planner::PackagePlanSource::Workspace { root, .. } =
                            &package.source
                        else {
                            continue;
                        };
                        let config = &package.build_config;
                        if !config.incremental
                            || config.cache_dir == "."
                            || !crate::frontend_adapter::safe_relative(&config.cache_dir)
                            || store.root()
                                != std::path::Path::new(build_plan.workspace_root.as_str())
                                    .join(root)
                                    .join(&config.cache_dir)
                        {
                            continue;
                        }
                    }
                    let mut versions = inputs.snapshot.source_versions.iter().filter(|version| {
                        version.package_id == module.package && version.module_path == module.path
                    });
                    let Some(version) = versions.next() else {
                        continue;
                    };
                    if versions.next().is_some() {
                        continue;
                    }
                    let input = PhaseInput::new(
                        task.unit.clone(),
                        PhaseDispatchInputBundle::without_parent_outputs(
                            inputs.snapshot.id,
                            crate::frontend_adapter::source_input_hash(version),
                            Vec::new(),
                        ),
                    );
                    if matches!(
                        mode,
                        NativeCacheMode::PersistentSource | NativeCacheMode::PersistentPrefix
                    ) {
                        pending_sources.push((task.id.clone(), input.clone()));
                    }
                    let Ok(CacheRehydrateOutcome::Rehydrated(handle)) =
                        inputs.rehydrate_native_disk_cache(&input, policy, store, &adapter)
                    else {
                        continue;
                    };
                    let owner = RetainOwner::new(format!(
                        "native-disk-cache:{:p}:{:?}:{}",
                        self,
                        session.id,
                        task.id.as_str()
                    ));
                    if publisher
                        .validate_output(
                            input.snapshot,
                            handle.any(),
                            OutputAuthorization::CurrentPackage,
                        )
                        .is_err()
                        || publisher.storage().validate_handle(handle.any()).is_err()
                        || publisher
                            .storage()
                            .retain(handle.output(), owner.clone())
                            .is_err()
                    {
                        continue;
                    }
                    scheduler_input
                        .cache_decisions
                        .decisions
                        .push(CacheTaskDecision::new(
                            task.id.clone(),
                            CacheSchedulingOutcome::ValidatedHit(ValidatedCacheHit::new(
                                vec![CacheOutputRef::new(
                                    format!("{:?}", handle.output()),
                                    format!("{:?}", handle.content_hash()),
                                )],
                                Vec::new(),
                            )),
                        ));
                    dispatcher.phase_results.insert(
                        task.id.clone(),
                        vec![PhaseResult {
                            status: PhaseStatus::Complete,
                            diagnostics: Vec::new(),
                            output_refs: vec![handle.erase()],
                            cache_observation: None,
                            artifact_commit: None,
                        }],
                    );
                    cached_sources.push((task.id.clone(), handle.output(), owner));
                    if task_graph
                        .tasks()
                        .iter()
                        .filter(|candidate| {
                            candidate.unit == task.unit
                                && candidate.phases == [PipelinePhase::SourceLoad]
                        })
                        .count()
                        != 1
                    {
                        continue;
                    }
                    let mut roots = task_graph.tasks().iter().filter(|candidate| {
                        candidate.kind == mizar_build::task_graph::TaskKind::PackageResolve
                            && candidate.unit == mizar_build::task_graph::WorkUnit::Workspace
                            && candidate.phases == [PipelinePhase::PackageResolve]
                    });
                    let Some(root) = roots.next() else { continue };
                    if roots.next().is_some() {
                        continue;
                    }
                    let mut frontends = task_graph.tasks().iter().filter(|candidate| {
                        candidate.unit == task.unit && candidate.phases == [PipelinePhase::Frontend]
                    });
                    let Some(frontend) = frontends.next() else {
                        continue;
                    };
                    if frontends.next().is_some()
                        || frontend.dependencies.len() != 2
                        || !frontend.dependencies.contains(&task.id)
                        || !frontend.dependencies.contains(&root.id)
                    {
                        continue;
                    }
                    let Ok(parent) =
                        mizar_ir::dispatch_input::SealedParentOutputHandle::from_published_output(
                            publisher,
                            input.snapshot,
                            handle.erase(),
                        )
                    else {
                        continue;
                    };
                    let Ok(bundle) = PhaseDispatchInputBundle::new(
                        input.snapshot,
                        input.identities().input_hash(),
                        Vec::new(),
                        vec![parent],
                    ) else {
                        continue;
                    };
                    let frontend_input = PhaseInput::new(task.unit.clone(), bundle);
                    let lookup = if let Some((oracle_input, oracle_adapter, oracle_result)) = oracle
                    {
                        if task.unit != oracle_input.work_unit
                            || oracle_result.status != PhaseStatus::Complete
                            || !oracle_result.diagnostics.is_empty()
                        {
                            continue;
                        }
                        let [oracle_output] = oracle_result.output_refs.as_slice() else {
                            continue;
                        };
                        let Ok(oracle_handle) = oracle_adapter.publisher().storage().typed_handle(
                            oracle_output,
                            &mizar_ir::identity::OutputKind::new("FrontendOutput"),
                        ) else {
                            continue;
                        };
                        inputs.rehydrate_native_disk_frontend_cache(
                            &frontend_input,
                            policy,
                            store,
                            (oracle_input, oracle_adapter, &oracle_handle),
                            &adapter,
                        )
                    } else if let Some(service) = self.registry.native_frontend_service() {
                        inputs.rehydrate_native_disk_frontend_cache_cold(
                            &frontend_input,
                            policy,
                            store,
                            &adapter,
                            service,
                        )
                    } else {
                        continue;
                    };
                    let Ok(CacheRehydrateOutcome::Rehydrated(frontend_handle)) = lookup else {
                        continue;
                    };
                    let owner = RetainOwner::new(format!(
                        "native-disk-cache:{:p}:{:?}:{}",
                        self,
                        session.id,
                        frontend.id.as_str()
                    ));
                    if publisher
                        .validate_output(
                            input.snapshot,
                            frontend_handle.any(),
                            OutputAuthorization::CurrentPackage,
                        )
                        .is_err()
                        || publisher
                            .storage()
                            .validate_handle(frontend_handle.any())
                            .is_err()
                        || publisher
                            .storage()
                            .retain(frontend_handle.output(), owner.clone())
                            .is_err()
                    {
                        continue;
                    }
                    scheduler_input
                        .cache_decisions
                        .decisions
                        .push(CacheTaskDecision::new(
                            frontend.id.clone(),
                            CacheSchedulingOutcome::ValidatedHit(ValidatedCacheHit::new(
                                vec![CacheOutputRef::new(
                                    format!("{:?}", frontend_handle.output()),
                                    format!("{:?}", frontend_handle.content_hash()),
                                )],
                                Vec::new(),
                            )),
                        ));
                    dispatcher.phase_results.insert(
                        frontend.id.clone(),
                        vec![PhaseResult {
                            status: PhaseStatus::Complete,
                            diagnostics: Vec::new(),
                            output_refs: vec![frontend_handle.erase()],
                            cache_observation: None,
                            artifact_commit: None,
                        }],
                    );
                    cached_sources.push((frontend.id.clone(), frontend_handle.output(), owner));
                }
            }
        }
        let mut execution = match SchedulerExecution::start(scheduler_input) {
            Ok(execution) => execution,
            Err(diagnostics) => {
                for (_, output, owner) in &cached_sources {
                    if let Some(publisher) = self.output_publisher.as_ref() {
                        let _ = publisher.storage().release(*output, owner);
                    }
                }
                session.finish(BuildSessionOutcome::Failed);
                let failed_session = session.clone();
                let publication_decision = self.lanes.publication_decision(snapshots, &session);
                let events = submission_events(
                    &session,
                    publication_decision,
                    DriverEventDetails {
                        planning: Some(PlanningEventStatus::Ready),
                        ..DriverEventDetails::default()
                    },
                );
                self.store_session(session, cancellation, events);
                return Err(DriverSubmitError::Scheduler {
                    session: Box::new(failed_session),
                    diagnostics,
                });
            }
        };

        if mode == SubmissionMode::Deferred {
            execution.advance(&mut dispatcher);
        }
        if mode == SubmissionMode::Deferred && !execution.is_finished() {
            let phase_results = std::mem::take(&mut dispatcher.phase_results);
            drop(dispatcher);
            let publication_decision = self.lanes.publication_decision(snapshots, &session);
            let events = submission_events(
                &session,
                publication_decision,
                DriverEventDetails {
                    planning: Some(PlanningEventStatus::Ready),
                    missing_services: &missing_services,
                    ..DriverEventDetails::default()
                },
            );
            self.store_session(session.clone(), cancellation, events);
            self.executions.insert(
                session.id,
                DriverExecution {
                    scheduler: execution,
                    output_publisher,
                    build_plan: build_plan.clone(),
                    module_index: module_index.clone(),
                    task_graph: task_graph.clone(),
                    missing_services: missing_services.clone(),
                    phase_inputs: phase_dispatch_inputs,
                    phase_results,
                    diagnostic_index: options.diagnostic_index,
                    cached_sources,
                    pending_sources,
                },
            );
            return Ok(BuildSubmission {
                session,
                build_plan: Some(build_plan),
                module_index: Some(module_index),
                task_graph: Some(task_graph),
                scheduler_run: None,
                missing_services,
                dispatch_gap_phases: Vec::new(),
                status: DriverSubmissionStatus::SchedulerValidated,
                publication_decision,
                diagnostic_index: None,
            });
        }
        while !execution.is_finished() {
            execution.advance(&mut dispatcher);
        }
        let phase_results = std::mem::take(&mut dispatcher.phase_results);
        drop(dispatcher);
        self.finish_submission(
            session,
            allocator,
            snapshots,
            cancellation,
            DriverExecution {
                scheduler: execution,
                output_publisher,
                build_plan,
                module_index,
                task_graph,
                missing_services,
                phase_inputs: phase_dispatch_inputs,
                phase_results,
                diagnostic_index: options.diagnostic_index,
                cached_sources,
                pending_sources,
            },
            native_cache,
        )
    }

    fn finish_submission<A: SessionIdAllocator>(
        &mut self,
        mut session: BuildSession,
        allocator: &A,
        snapshots: &SnapshotRegistry<A>,
        cancellation: CancellationPolicy,
        execution: DriverExecution,
        native_cache: Option<NativeCacheOptions<'_>>,
    ) -> Result<BuildSubmission, DriverSubmitError> {
        let DriverExecution {
            scheduler,
            build_plan,
            module_index,
            task_graph,
            missing_services,
            mut phase_results,
            cached_sources,
            pending_sources,
            ..
        } = execution;
        let cancel_reason = match scheduler.cancellation_signal().requested_reason() {
            Some(CancellationReason::SupersededSnapshot) => Some(DriverCancelReason::Superseded),
            Some(CancellationReason::ExplicitRequest) => Some(DriverCancelReason::ExplicitRequest),
            Some(CancellationReason::Shutdown) => Some(DriverCancelReason::Shutdown),
            _ => None,
        };
        let scheduler_run = scheduler.finish().expect("drained scheduler execution");
        let outcome = cancel_reason
            .map(cancellation_outcome)
            .unwrap_or_else(|| scheduler_outcome(&scheduler_run));

        let dispatch_gap_phases = dispatch_gap_phases(&task_graph, &scheduler_run);
        let status = if matches!(outcome, BuildSessionOutcome::Blocked)
            && !dispatch_gap_phases.is_empty()
        {
            DriverSubmissionStatus::BlockedByPhaseDispatchGap
        } else if matches!(outcome, BuildSessionOutcome::Blocked) && !missing_services.is_empty() {
            DriverSubmissionStatus::BlockedByMissingPhaseServices
        } else {
            DriverSubmissionStatus::SchedulerValidated
        };
        for (task, _, _) in &cached_sources {
            if !scheduler_run.task_states.iter().any(|record| {
                &record.task_id == task
                    && record.state == mizar_build::scheduler::TaskState::CacheHit
            }) {
                phase_results.remove(task);
            }
        }
        // OpenBuffer roots live only until the same-call coherent payload has consumed them.
        // Capture genuine commit roots before stale-output suppression can discard the handoffs.
        let open_staging_roots =
            phase_results
                .iter()
                .flat_map(|(task_id, results)| {
                    let task = task_graph.tasks().iter().find(|task| task.id == *task_id);
                    results
                        .iter()
                        .filter_map(|result| {
                            let commit = result.artifact_commit.as_ref()?;
                            let mizar_build::task_graph::WorkUnit::Module { module } = &task?.unit
                            else {
                                return None;
                            };
                            let version = session.captured.snapshot.source_versions.iter().find(
                                |version| {
                                    version.package_id == module.package
                                        && version.module_path == module.path
                                },
                            )?;
                            if !matches!(
                                version.origin,
                                mizar_session::SourceOrigin::OpenBuffer { .. }
                            ) {
                                return None;
                            }
                            let package = build_plan
                                .packages
                                .iter()
                                .find(|package| package.package_id == module.package)?;
                            let mizar_build::planner::PackagePlanSource::Workspace { root, .. } =
                                &package.source
                            else {
                                return None;
                            };
                            let package_root = std::path::PathBuf::from(
                                session.captured.snapshot.workspace_root.as_str(),
                            )
                            .join(root)
                            .canonicalize()
                            .ok()?;
                            let label = mizar_artifact::store::PublishedArtifactPath::new(
                                &commit.manifest().artifact_root,
                            )
                            .ok()?;
                            Some((package_root.join(label.as_str()), package_root))
                        })
                        .collect::<Vec<_>>()
                })
                .collect::<Vec<_>>();
        if cancel_reason.is_some() {
            for result in phase_results.values_mut().flatten() {
                result.output_refs.clear();
                result.artifact_commit = None;
            }
        }
        let publisher_current = self.output_publisher.as_ref().is_none_or(|publisher| {
            publisher
                .validate_current_snapshot(session.captured.snapshot.id)
                .is_ok()
                && phase_results.values().flatten().all(|result| {
                    result.output_refs.iter().all(|output| {
                        publisher
                            .validate_output(
                                session.captured.snapshot.id,
                                output,
                                if session.captured.native_sources().is_some() {
                                    OutputAuthorization::Published
                                } else {
                                    OutputAuthorization::CurrentPackage
                                },
                            )
                            .is_ok()
                            && publisher.storage().validate_handle(output).is_ok()
                    })
                })
        });
        if !publisher_current
            || self.lanes.publication_decision(snapshots, &session) != PublicationDecision::Current
        {
            phase_results.clear();
        }
        if let Some(NativeCacheOptions {
            policy,
            store,
            mode,
        }) = native_cache
            && matches!(
                mode,
                NativeCacheMode::PersistentSource | NativeCacheMode::PersistentPrefix
            )
            && !scheduler_run
                .task_states
                .iter()
                .any(|record| record.state == mizar_build::scheduler::TaskState::Cancelled)
            && let Some(publisher) = self.output_publisher.as_ref()
        {
            let inputs = crate::registry::SourceLoadInputs {
                snapshot: &session.captured.snapshot,
                captured: Some(&session.captured),
                build_plan: &build_plan,
                module_index: &module_index,
                allocator,
            };
            let adapter = mizar_ir::cache_adapter::IrCacheAdapter::new(publisher.clone());
            for (task, input) in &pending_sources {
                let Some(policy) = self.registry.native_source_policy(policy) else {
                    continue;
                };
                if !scheduler_run.task_states.iter().any(|record| {
                    &record.task_id == task
                        && record.state == mizar_build::scheduler::TaskState::Completed
                }) {
                    continue;
                }
                let Some(results) = phase_results.get(task) else {
                    continue;
                };
                let [result] = results.as_slice() else {
                    continue;
                };
                let [output] = result.output_refs.as_slice() else {
                    continue;
                };
                if result.status != crate::registry::PhaseStatus::Complete {
                    continue;
                }
                let Ok(handle) = publisher
                    .storage()
                    .typed_handle(output, &mizar_ir::identity::OutputKind::new("SourceUnit"))
                else {
                    continue;
                };
                let Ok(mizar_ir::cache_adapter::EncodeCacheRecordOutcome::Encoded(record)) =
                    inputs.encode_native_disk_cache_record(input, policy, &adapter, handle)
                else {
                    continue;
                };
                if self.lanes.publication_decision(snapshots, &session)
                    != PublicationDecision::Current
                    || publisher
                        .validate_output(
                            input.snapshot,
                            output,
                            OutputAuthorization::CurrentPackage,
                        )
                        .is_err()
                    || publisher.storage().validate_handle(output).is_err()
                {
                    break;
                }
                let _ = store.insert(&record);
            }
            if mode == NativeCacheMode::PersistentPrefix {
                use crate::registry::{PhaseInput, PhaseStatus};
                use mizar_ir::dispatch_input::{
                    PhaseDispatchInputBundle, SealedParentOutputHandle,
                };
                let mut roots = task_graph.tasks().iter().filter(|task| {
                    task.kind == mizar_build::task_graph::TaskKind::PackageResolve
                        && task.unit == mizar_build::task_graph::WorkUnit::Workspace
                        && task.phases == [PipelinePhase::PackageResolve]
                });
                let root = roots.next().filter(|root| {
                    roots.next().is_none()
                        && scheduler_run.task_states.iter().any(|record| {
                            record.task_id == root.id
                                && record.state == mizar_build::scheduler::TaskState::Completed
                        })
                });
                for (source_task, source_input) in &pending_sources {
                    if !scheduler_run.task_states.iter().any(|record| {
                        &record.task_id == source_task
                            && matches!(
                                record.state,
                                mizar_build::scheduler::TaskState::Completed
                                    | mizar_build::scheduler::TaskState::CacheHit
                            )
                    }) {
                        continue;
                    }
                    let Some([source_result]) = phase_results.get(source_task).map(Vec::as_slice)
                    else {
                        continue;
                    };
                    let [source_output] = source_result.output_refs.as_slice() else {
                        continue;
                    };
                    if source_result.status != PhaseStatus::Complete {
                        continue;
                    }
                    let mut frontends = task_graph.tasks().iter().filter(|task| {
                        task.unit == source_input.work_unit
                            && task.phases == [PipelinePhase::Frontend]
                            && root.is_some_and(|root| {
                                task.dependencies.len() == 2
                                    && task.dependencies.contains(source_task)
                                    && task.dependencies.contains(&root.id)
                            })
                    });
                    let Some(frontend_task) = frontends.next() else {
                        continue;
                    };
                    if frontends.next().is_some()
                        || !scheduler_run.task_states.iter().any(|record| {
                            record.task_id == frontend_task.id
                                && record.state == mizar_build::scheduler::TaskState::Completed
                        })
                    {
                        continue;
                    }
                    let Some([result]) = phase_results.get(&frontend_task.id).map(Vec::as_slice)
                    else {
                        continue;
                    };
                    let [output] = result.output_refs.as_slice() else {
                        continue;
                    };
                    if result.status != PhaseStatus::Complete || !result.diagnostics.is_empty() {
                        continue;
                    }
                    let Ok(parent) = SealedParentOutputHandle::from_published_output(
                        publisher,
                        source_input.snapshot,
                        source_output.clone(),
                    ) else {
                        continue;
                    };
                    let Ok(bundle) = PhaseDispatchInputBundle::new(
                        source_input.snapshot,
                        source_input.identities().input_hash(),
                        Vec::new(),
                        vec![parent],
                    ) else {
                        continue;
                    };
                    let input = PhaseInput::new(source_input.work_unit.clone(), bundle);
                    let Ok(handle) = publisher.storage().typed_handle(
                        output,
                        &mizar_ir::identity::OutputKind::new("FrontendOutput"),
                    ) else {
                        continue;
                    };
                    let Ok(mizar_ir::cache_adapter::EncodeCacheRecordOutcome::Encoded(record)) =
                        inputs.encode_native_disk_frontend_cache_record(
                            &input, policy, &adapter, handle,
                        )
                    else {
                        continue;
                    };
                    if self.lanes.publication_decision(snapshots, &session)
                        != PublicationDecision::Current
                        || [source_output, output].iter().any(|output| {
                            publisher
                                .validate_output(
                                    input.snapshot,
                                    output,
                                    OutputAuthorization::CurrentPackage,
                                )
                                .is_err()
                                || publisher.storage().validate_handle(output).is_err()
                        })
                    {
                        break;
                    }
                    let _ = store.insert(&record);
                }
            }
        }
        let mut driver_scheduler_run = DriverSchedulerRun::from_scheduler_run(scheduler_run);
        driver_scheduler_run.phase_results = phase_results;
        session.finish(outcome);
        let publication_decision = self.lanes.publication_decision(snapshots, &session);
        let mut artifact_commits = Vec::new();
        for (task_id, results) in &mut driver_scheduler_run.phase_results {
            for (result_index, result) in results.iter_mut().enumerate() {
                if result.artifact_commit.is_none() {
                    continue;
                }
                let eligible = (|| {
                    use mizar_ir::identity::{OutputKind, PipelinePhase as IrPhase, WorkUnit};
                    if publication_decision != PublicationDecision::Current {
                        return None;
                    }
                    let task = task_graph.tasks().iter().find(|task| task.id == *task_id)?;
                    let mizar_build::task_graph::WorkUnit::Module { module } = &task.unit else {
                        return None;
                    };
                    if task.kind != mizar_build::task_graph::TaskKind::ArtifactCommit
                        || task.phases != [PipelinePhase::ArtifactCommit]
                    {
                        return None;
                    }
                    let [output] = result.output_refs.as_slice() else {
                        return None;
                    };
                    let work_unit = mizar_session::module_work_unit_label(
                        module.package.as_str(),
                        module.path.as_str(),
                    );
                    if output.snapshot() != session.captured.snapshot.id
                        || output.phase() != &IrPhase::from(PipelinePhase::ArtifactCommit)
                        || output.output_kind() != &OutputKind::new("SourceTheoremArtifactData")
                        || output.work_unit() != &WorkUnit::new(&work_unit)
                        || output.schema_version() != mizar_ir::storage::SchemaVersion::new(1)
                    {
                        return None;
                    }
                    let publisher = self.output_publisher.as_ref()?;
                    publisher
                        .validate_output(
                            session.captured.snapshot.id,
                            output,
                            OutputAuthorization::Published,
                        )
                        .ok()?;
                    publisher.storage().validate_handle(output).ok()?;
                    let finished = driver_scheduler_run.events.iter().find(|event| {
                        event.task_id.as_ref() == Some(task_id)
                            && event.kind
                                == mizar_build::scheduler::SchedulerEventKind::TaskFinished
                    })?;
                    Some((finished.order, work_unit))
                })();
                let Some((order, work_unit)) = eligible else {
                    result.artifact_commit = None;
                    continue;
                };
                artifact_commits.push((order, work_unit, task_id.clone(), result_index));
            }
        }
        let snapshot_publication = self
            .output_publisher
            .as_ref()
            .filter(|_| {
                publication_decision == PublicationDecision::Current
                    && outcome == BuildSessionOutcome::Succeeded
            })
            .and_then(|publisher| {
                crate::registry::SourceLoadInputs {
                    snapshot: &session.captured.snapshot,
                    captured: Some(&session.captured),
                    build_plan: &build_plan,
                    module_index: &module_index,
                    allocator,
                }
                .complete_snapshot_publication(&task_graph, &driver_scheduler_run, publisher)
                .filter(|_| {
                    self.lanes.publication_decision(snapshots, &session)
                        == PublicationDecision::Current
                })
                .map(Arc::new)
            });
        for (root, boundary) in open_staging_roots {
            crate::frontend_adapter::remove_native_artifact_staging(&root, &boundary);
        }
        // Borrow the same-call owner handoffs only after mutation and complete assembly.
        let artifact_commits = artifact_commits
            .into_iter()
            .map(|(order, work_unit, task_id, result_index)| {
                let commit = driver_scheduler_run.phase_results[&task_id][result_index]
                    .artifact_commit
                    .as_ref()
                    .expect("eligible same-call artifact handoff remains stored");
                (order, work_unit, commit)
            })
            .collect::<Vec<_>>();
        let events = submission_events(
            &session,
            publication_decision,
            DriverEventDetails {
                planning: Some(PlanningEventStatus::Ready),
                missing_services: &missing_services,
                dispatch_gap_phases: &dispatch_gap_phases,
                scheduler_events: &driver_scheduler_run.events,
                scheduler_task_states: &driver_scheduler_run.task_states,
                artifact_commits: &artifact_commits,
                snapshot_publication: snapshot_publication.as_ref(),
            },
        );
        for (_, output, owner) in &cached_sources {
            if let Some(publisher) = self.output_publisher.as_ref() {
                let _ = publisher.storage().release(*output, owner);
            }
        }
        self.store_session(session.clone(), cancellation, events);
        Ok(BuildSubmission {
            session,
            build_plan: Some(build_plan),
            module_index: Some(module_index),
            task_graph: Some(task_graph),
            scheduler_run: Some(driver_scheduler_run),
            missing_services,
            dispatch_gap_phases,
            status,
            publication_decision,
            diagnostic_index: None,
        })
    }

    pub fn cancel<A>(
        &mut self,
        session: BuildSessionId,
        reason: DriverCancelReason,
        snapshots: &SnapshotRegistry<A>,
    ) -> DriverCancelOutcome
    where
        A: SessionIdAllocator,
    {
        let Some(record) = self.sessions.get_mut(&session) else {
            return DriverCancelOutcome {
                session,
                changed: false,
                state: None,
            };
        };
        if record.session.is_terminal() {
            return DriverCancelOutcome {
                session,
                changed: false,
                state: Some(record.session.state),
            };
        }

        if let Some(execution) = self.executions.get(&session) {
            let changed = execution
                .scheduler
                .cancellation_signal()
                .request(match reason {
                    DriverCancelReason::ExplicitRequest => CancellationReason::ExplicitRequest,
                    DriverCancelReason::Superseded => CancellationReason::SupersededSnapshot,
                    DriverCancelReason::Shutdown => CancellationReason::Shutdown,
                });
            if changed {
                record.session.cancel();
            }
            return DriverCancelOutcome {
                session,
                changed,
                state: Some(record.session.state),
            };
        }

        let cancellation_started = record.session.cancel();
        let outcome = cancellation_outcome(reason);
        let terminal_changed = record.session.finish(outcome);
        let changed = cancellation_started || terminal_changed;
        if changed && reason == DriverCancelReason::Superseded {
            record
                .cancellation
                .supersede_snapshot(record.session.captured.snapshot.id);
        }
        if changed {
            let publication = self.lanes.publication_decision(snapshots, &record.session);
            record.events = append_terminal_events(&record.events, &record.session, publication);
        }
        DriverCancelOutcome {
            session,
            changed,
            state: Some(record.session.state),
        }
    }

    pub fn events(&self, session: BuildSessionId) -> BuildEventStream {
        self.sessions
            .get(&session)
            .map(|record| record.events.replay())
            .unwrap_or_else(|| BuildEventStream::empty(session, false))
    }

    fn validate_watch_previous(
        &self,
        previous_session: Option<BuildSessionId>,
        request: &BuildRequestDraft,
    ) -> Result<WatchPreviousSession, WatchSubmitError> {
        let current = self.lanes.current(request.lane);
        let derived_previous = current.map(|current| current.session);
        let previous_session = match (previous_session, derived_previous) {
            (Some(supplied), Some(current)) if supplied != current => {
                return Err(WatchSubmitError::PreviousSessionNotCurrent {
                    session: supplied,
                    current,
                });
            }
            (Some(supplied), None) => supplied,
            (Some(supplied), Some(_current)) => supplied,
            (None, Some(current)) => current,
            (None, None) => {
                return Ok(WatchPreviousSession {
                    session: None,
                    snapshot: None,
                });
            }
        };
        let Some(record) = self.sessions.get(&previous_session) else {
            return Err(WatchSubmitError::PreviousSessionUnknown {
                session: previous_session,
            });
        };
        if record.session.request.lane != request.lane {
            return Err(WatchSubmitError::PreviousSessionLaneMismatch {
                session: previous_session,
            });
        }
        if !matches!(&record.session.request.origin, BuildRequestOrigin::Watch(_)) {
            return Err(WatchSubmitError::PreviousSessionNotWatch {
                session: previous_session,
            });
        }
        if record.session.request.generation.get() >= request.generation.get() {
            return Err(WatchSubmitError::NonMonotonicGeneration {
                session: previous_session,
            });
        }
        Ok(WatchPreviousSession {
            session: Some(previous_session),
            snapshot: Some(record.session.captured.snapshot.id),
        })
    }

    fn supersede_previous_watch_session<A>(
        &mut self,
        previous_session: Option<BuildSessionId>,
        snapshots: &SnapshotRegistry<A>,
    ) -> Option<WatchSupersededSession>
    where
        A: SessionIdAllocator,
    {
        let previous_session = previous_session?;
        let record = self.sessions.get_mut(&previous_session)?;
        let previous_state = record.session.state;
        let snapshot = record.session.captured.snapshot.id;
        record.session.state = BuildSessionState::Finished(BuildSessionOutcome::Superseded);
        record.cancellation.supersede_snapshot(snapshot);
        let publication_decision = self.lanes.publication_decision(snapshots, &record.session);
        record.events =
            suppress_watch_replay_events(&record.events, &record.session, publication_decision);
        Some(WatchSupersededSession {
            session: previous_session,
            snapshot,
            previous_state,
            state: record.session.state,
            publication_decision,
        })
    }

    fn missing_services(
        &self,
        task_graph: &TaskGraph,
    ) -> Result<Vec<DriverMissingPhaseService>, PhaseRegistryError> {
        let mut missing = BTreeMap::new();
        for task in task_graph.tasks() {
            for phase in &task.phases {
                if *phase == PipelinePhase::PackageResolve {
                    continue;
                }
                match self.registry.descriptor_for_phase(*phase) {
                    Ok(_descriptor) => {}
                    Err(PhaseRegistryError::MissingPhaseService {
                        phase,
                        availability,
                    }) => {
                        missing.entry(phase).or_insert(availability);
                    }
                    Err(error) => {
                        return Err(error);
                    }
                }
            }
        }
        Ok(missing
            .into_iter()
            .map(|(phase, availability)| DriverMissingPhaseService {
                phase,
                availability,
            })
            .collect())
    }

    fn store_session(
        &mut self,
        session: BuildSession,
        cancellation: CancellationPolicy,
        events: BuildEventStream,
    ) {
        let native_checkpoint_observations = self
            .sessions
            .get_mut(&session.id)
            .and_then(|record| record.native_checkpoint_observations.take());
        self.sessions.insert(
            session.id,
            DriverSessionRecord {
                session,
                cancellation,
                events,
                terminal_submission: None,
                native_checkpoint_observations,
            },
        );
    }
}

impl Default for CompilerDriver {
    fn default() -> Self {
        Self::new(PhaseRegistry::empty())
    }
}

impl<L> DriverSubmitInput<L>
where
    L: SourceLayoutProvider,
{
    pub fn new(
        plan_request: PlanRequest,
        workspace_packages: Vec<WorkspacePackage>,
        lockfile: Lockfile,
        source_layout: L,
    ) -> Self {
        Self {
            plan_request,
            workspace_packages,
            lockfile,
            source_layout,
            dependency_artifacts: Vec::new(),
            dependency_overlay: ModuleDependencyOverlay::unavailable(),
            vc_descriptors: Vec::new(),
            task_graph_profile: TaskGraphProfile::default(),
            scheduler_mode: SchedulerMode::Batch,
            priority_hints: PriorityHints::default(),
            cache_policy: CacheSchedulingPolicy::Unavailable,
            cache_decisions: CacheSchedulingPlan::empty(),
            resource_budget: ResourceBudget::default(),
            cancellation: CancellationPolicy::default(),
            phase_dispatch_inputs: None,
            worker_count: 1,
            completion_order: CompletionOrder::Canonical,
        }
    }
}
