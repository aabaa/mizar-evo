use std::collections::{BTreeMap, HashMap};

use mizar_build::{
    cache_seam::CacheSchedulingPlan,
    cancel::CancellationPolicy,
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
        SchedulerDiagnostics, SchedulerEvent, SchedulerInput, SchedulerMode, TaskStateRecord,
        run_scheduler_with_dispatcher,
    },
    task_graph::{
        BuildTask, ModuleDependencyOverlay, PipelinePhase, TaskGraph, TaskGraphDiagnostics,
        TaskGraphInput, TaskGraphProfile, VcTaskDescriptor, build_task_graph,
    },
};
use mizar_ir::publisher::{PhaseOutputPublisher, PublishError};
use mizar_session::{
    BuildSessionId, BuildSnapshotId, IdError, SessionIdAllocator, SnapshotRegistry,
};

use crate::{
    events::{BuildEventStream, OwnerGapClassification, PlanningEventStatus},
    registry::{PhaseInputIdentities, PhaseRegistry, PhaseRegistryError, PhaseServiceAvailability},
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
    pub phase_dispatch_inputs: Option<Box<dyn PhaseDispatchInputProvider>>,
    pub worker_count: usize,
    pub completion_order: CompletionOrder,
}

pub trait PhaseDispatchInputProvider {
    fn input_identities_for_task(&self, task: &BuildTask) -> Option<PhaseInputIdentities>;
}

pub struct CompilerDriver {
    sessions: HashMap<BuildSessionId, DriverSessionRecord>,
    lanes: DriverLanes,
    registry: PhaseRegistry,
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
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DriverSchedulerRun {
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
            registry,
        }
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

    pub fn submit_watch_change<A, L>(
        &mut self,
        request: BuildRequestDraft,
        allocator: &A,
        snapshots: &SnapshotRegistry<A>,
        input: DriverSubmitInput<L>,
        control: WatchSubmitControl<'_>,
    ) -> Result<WatchSubmission, WatchSubmitError>
    where
        A: SessionIdAllocator,
        L: SourceLayoutProvider,
    {
        if !matches!(&request.origin, BuildRequestOrigin::Watch(_)) {
            return Err(WatchSubmitError::NonWatchRequest {
                origin: request_origin_name(&request.origin),
            });
        }

        let previous_watch = self.validate_watch_previous(control.previous_session, &request)?;
        let gaps = watch_mode_gaps(&control);
        let submission = match self.submit(request, allocator, snapshots, input) {
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
    ) -> Result<BuildSubmission, DriverSubmitError>
    where
        A: SessionIdAllocator,
        L: SourceLayoutProvider,
    {
        let pending = request
            .allocate(allocator)
            .map_err(|error| DriverSubmitError::RequestIdAllocation { error })?;
        let mut session = pending
            .capture_snapshot(snapshots)
            .map_err(|error| DriverSubmitError::SnapshotCapture { error })?;

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
        let task_graph = match build_task_graph(TaskGraphInput {
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
        if !missing_services.is_empty() {
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
            });
        }

        session.mark_running();
        let mut scheduler_input = SchedulerInput::new(task_graph.clone());
        scheduler_input.mode = scheduler_mode;
        scheduler_input.priority_hints = priority_hints;
        scheduler_input.cache = cache_policy;
        scheduler_input.cache_decisions = cache_decisions;
        scheduler_input.resource_budget = resource_budget;
        scheduler_input.cancellation = cancellation.clone();
        scheduler_input.worker_count = worker_count.max(1);
        scheduler_input.completion_order = completion_order;
        let mut dispatcher =
            RegistrySchedulerDispatcher::new(&self.registry, phase_dispatch_inputs.as_deref());
        let scheduler_run = match run_scheduler_with_dispatcher(scheduler_input, &mut dispatcher) {
            Ok(scheduler_run) => scheduler_run,
            Err(diagnostics) => {
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

        let outcome = scheduler_outcome(&scheduler_run);
        let dispatch_gap_phases = dispatch_gap_phases(&task_graph, &scheduler_run);
        let status =
            if matches!(outcome, BuildSessionOutcome::Blocked) && !dispatch_gap_phases.is_empty() {
                DriverSubmissionStatus::BlockedByPhaseDispatchGap
            } else {
                DriverSubmissionStatus::SchedulerValidated
            };
        let driver_scheduler_run = DriverSchedulerRun::from_scheduler_run(scheduler_run);
        session.finish(outcome);
        let publication_decision = self.lanes.publication_decision(snapshots, &session);
        let events = submission_events(
            &session,
            publication_decision,
            DriverEventDetails {
                planning: Some(PlanningEventStatus::Ready),
                dispatch_gap_phases: &dispatch_gap_phases,
                scheduler_events: &driver_scheduler_run.events,
                scheduler_task_states: &driver_scheduler_run.task_states,
                ..DriverEventDetails::default()
            },
        );
        self.store_session(session.clone(), cancellation, events);
        Ok(BuildSubmission {
            session,
            build_plan: Some(build_plan),
            module_index: Some(module_index),
            task_graph: Some(task_graph),
            scheduler_run: Some(driver_scheduler_run),
            missing_services: Vec::new(),
            dispatch_gap_phases,
            status,
            publication_decision,
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
        self.sessions.insert(
            session.id,
            DriverSessionRecord {
                session,
                cancellation,
                events,
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
