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
        SchedulerDiagnostics, SchedulerEvent, SchedulerInput, SchedulerMode, SchedulerRun,
        TaskState, TaskStateRecord, run_scheduler,
    },
    task_graph::{
        ModuleDependencyOverlay, PipelinePhase, TaskGraph, TaskGraphDiagnostics, TaskGraphInput,
        TaskGraphProfile, VcTaskDescriptor, build_task_graph,
    },
};
use mizar_session::{BuildSessionId, IdError, SessionIdAllocator, SnapshotRegistry};

use crate::{
    registry::{PhaseRegistry, PhaseRegistryError, PhaseServiceAvailability},
    request::{
        BuildRequestDraft, BuildSession, BuildSessionOutcome, BuildSessionState,
        CaptureSnapshotError, DriverLanes, PublicationDecision,
    },
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
    pub worker_count: usize,
    pub completion_order: CompletionOrder,
}

pub struct CompilerDriver {
    sessions: HashMap<BuildSessionId, DriverSessionRecord>,
    lanes: DriverLanes,
    registry: PhaseRegistry,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct DriverSessionRecord {
    session: BuildSession,
    cancellation: CancellationPolicy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BuildEventStream {
    pub session: BuildSessionId,
    pub known_session: bool,
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
            self.store_session(session.clone(), input.cancellation);
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
            worker_count,
            completion_order,
        } = input;

        let build_plan = match produce_build_plan(plan_request, workspace_packages, lockfile) {
            Ok(build_plan) => build_plan,
            Err(diagnostics) => {
                session.finish(BuildSessionOutcome::Failed);
                let failed_session = session.clone();
                self.store_session(session, cancellation);
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
                    self.store_session(session, cancellation);
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
                self.store_session(session, cancellation);
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
                self.store_session(session, cancellation);
                return Err(DriverSubmitError::PhaseRegistry {
                    session: Box::new(failed_session),
                    error,
                });
            }
        };
        if !missing_services.is_empty() {
            session.finish(BuildSessionOutcome::Blocked);
            let publication_decision = self.lanes.publication_decision(snapshots, &session);
            self.store_session(session.clone(), cancellation);
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

        let dispatch_gap_phases = dispatch_gap_phases(&task_graph);
        if !dispatch_gap_phases.is_empty() {
            session.finish(BuildSessionOutcome::Blocked);
            let publication_decision = self.lanes.publication_decision(snapshots, &session);
            self.store_session(session.clone(), cancellation);
            return Ok(BuildSubmission {
                session,
                build_plan: Some(build_plan),
                module_index: Some(module_index),
                task_graph: Some(task_graph),
                scheduler_run: None,
                missing_services: Vec::new(),
                dispatch_gap_phases,
                status: DriverSubmissionStatus::BlockedByPhaseDispatchGap,
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
        let scheduler_run = match run_scheduler(scheduler_input) {
            Ok(scheduler_run) => scheduler_run,
            Err(diagnostics) => {
                session.finish(BuildSessionOutcome::Failed);
                let failed_session = session.clone();
                self.store_session(session, cancellation);
                return Err(DriverSubmitError::Scheduler {
                    session: Box::new(failed_session),
                    diagnostics,
                });
            }
        };

        let outcome = scheduler_outcome(&scheduler_run);
        let driver_scheduler_run = DriverSchedulerRun::from_scheduler_run(scheduler_run);
        session.finish(outcome);
        let publication_decision = self.lanes.publication_decision(snapshots, &session);
        self.store_session(session.clone(), cancellation);
        Ok(BuildSubmission {
            session,
            build_plan: Some(build_plan),
            module_index: Some(module_index),
            task_graph: Some(task_graph),
            scheduler_run: Some(driver_scheduler_run),
            missing_services: Vec::new(),
            dispatch_gap_phases: Vec::new(),
            status: DriverSubmissionStatus::SchedulerValidated,
            publication_decision,
        })
    }

    pub fn cancel(
        &mut self,
        session: BuildSessionId,
        reason: DriverCancelReason,
    ) -> DriverCancelOutcome {
        let Some(record) = self.sessions.get_mut(&session) else {
            return DriverCancelOutcome {
                session,
                changed: false,
                state: None,
            };
        };
        let changed = record.session.cancel();
        if changed && reason == DriverCancelReason::Superseded {
            record
                .cancellation
                .supersede_snapshot(record.session.captured.snapshot.id);
        }
        DriverCancelOutcome {
            session,
            changed,
            state: Some(record.session.state),
        }
    }

    pub fn events(&self, session: BuildSessionId) -> BuildEventStream {
        BuildEventStream {
            session,
            known_session: self.sessions.contains_key(&session),
        }
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

    fn store_session(&mut self, session: BuildSession, cancellation: CancellationPolicy) {
        self.sessions.insert(
            session.id,
            DriverSessionRecord {
                session,
                cancellation,
            },
        );
    }
}

impl DriverSchedulerRun {
    fn from_scheduler_run(run: SchedulerRun) -> Self {
        Self {
            task_states: run.task_states,
            events: run.events,
            diagnostics: run.diagnostics,
        }
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
            worker_count: 1,
            completion_order: CompletionOrder::Canonical,
        }
    }
}

fn scheduler_outcome(run: &SchedulerRun) -> BuildSessionOutcome {
    if run
        .task_states
        .iter()
        .any(|record| record.state == TaskState::Failed)
    {
        BuildSessionOutcome::Failed
    } else if run
        .task_states
        .iter()
        .any(|record| record.state == TaskState::Blocked)
    {
        BuildSessionOutcome::Blocked
    } else if run
        .task_states
        .iter()
        .any(|record| record.state == TaskState::Cancelled)
    {
        BuildSessionOutcome::Cancelled
    } else {
        BuildSessionOutcome::Succeeded
    }
}

fn dispatch_gap_phases(task_graph: &TaskGraph) -> Vec<PipelinePhase> {
    let mut phases = BTreeMap::new();
    for task in task_graph.tasks() {
        for phase in &task.phases {
            if *phase != PipelinePhase::PackageResolve {
                phases.entry(*phase).or_insert(());
            }
        }
    }
    phases.into_keys().collect()
}

#[cfg(test)]
mod tests {
    use mizar_build::cancel::CancellationPolicy;

    use super::*;
    use crate::request::{
        BatchInvocation, BatchRequest, BuildLaneId, BuildProfile, BuildRequestGeneration,
        BuildRequestOrigin, BuildTargets, DependencyInputSet, SourceInputSet, VerifierConfigInput,
    };
    use mizar_session::{
        DependencyArtifactRef, Hash, InMemorySessionIdAllocator, SnapshotRegistry, ToolchainInfo,
        WorkspaceRoot,
    };

    #[test]
    fn cancel_active_session_updates_build_policy_once() {
        let ids = InMemorySessionIdAllocator::new();
        let snapshots = SnapshotRegistry::new();
        let session = request(9)
            .allocate(&ids)
            .unwrap()
            .capture_snapshot(&snapshots)
            .unwrap();
        let session_id = session.id;
        let mut driver = CompilerDriver::default();
        driver.store_session(session, CancellationPolicy::default());

        let first = driver.cancel(session_id, DriverCancelReason::Superseded);
        let second = driver.cancel(session_id, DriverCancelReason::Superseded);

        assert!(first.changed);
        assert_eq!(first.state, Some(BuildSessionState::Cancelling));
        assert!(!second.changed);
        assert_eq!(second.state, Some(BuildSessionState::Cancelling));
        assert_eq!(
            driver
                .cancellation_policy(session_id)
                .unwrap()
                .superseded_snapshots()
                .len(),
            1
        );
    }

    #[test]
    fn explicit_and_shutdown_cancel_do_not_claim_snapshot_supersession() {
        for (seed, reason) in [
            (10, DriverCancelReason::ExplicitRequest),
            (11, DriverCancelReason::Shutdown),
        ] {
            let ids = InMemorySessionIdAllocator::new();
            let snapshots = SnapshotRegistry::new();
            let session = request(seed)
                .allocate(&ids)
                .unwrap()
                .capture_snapshot(&snapshots)
                .unwrap();
            let session_id = session.id;
            let mut driver = CompilerDriver::default();
            driver.store_session(session, CancellationPolicy::default());

            let outcome = driver.cancel(session_id, reason);

            assert!(outcome.changed);
            assert_eq!(outcome.state, Some(BuildSessionState::Cancelling));
            assert!(
                driver
                    .cancellation_policy(session_id)
                    .unwrap()
                    .superseded_snapshots()
                    .is_empty()
            );
        }
    }

    fn request(seed: u8) -> BuildRequestDraft {
        BuildRequestDraft {
            lane: BuildLaneId::new(u64::from(seed)),
            origin: BuildRequestOrigin::Batch(BatchRequest {
                invocation: BatchInvocation::default(),
            }),
            generation: BuildRequestGeneration::new(0),
            workspace_root: WorkspaceRoot::new("workspace"),
            profile: BuildProfile::new("check"),
            targets: BuildTargets::default(),
            source_inputs: SourceInputSet::default(),
            dependency_inputs: DependencyInputSet::new(
                vec![DependencyArtifactRef {
                    artifact: format!("dep-{seed}"),
                    content_hash: Hash::from_bytes([seed; Hash::BYTE_LEN]),
                }],
                Hash::from_bytes([seed.wrapping_add(1); Hash::BYTE_LEN]),
                ToolchainInfo::new("mizar-evo-test"),
            ),
            verifier_config: VerifierConfigInput::new(Hash::from_bytes(
                [seed.wrapping_add(2); Hash::BYTE_LEN],
            )),
        }
    }
}
