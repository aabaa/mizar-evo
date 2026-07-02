use std::collections::{BTreeMap, BTreeSet, VecDeque};

use crate::{
    cache_seam::{
        CacheSchedulingOutcome, CacheSchedulingPlan, CacheSchedulingPlanDiagnosticKind,
        ValidatedCacheHit,
    },
    cancel::{CancellationCheckpoint, CancellationDecision, CancellationState, CancellationToken},
    failure_state::{BlockReason, BlockedTaskRecord, BuildFailureRecord},
    resource::{
        ResourceAdmissionStatus, ResourceBudget, ResourceManager, ResourceTelemetry,
        TaskResourceRequest, resource_queue_rank,
    },
    task_graph::{
        BuildTask, DependencyCoverage, PriorityClass, ResourceClass, TaskGraph, TaskGraphVersion,
        TaskId, TaskKind,
    },
};
use mizar_session::BuildSnapshotId;

pub use crate::cancel::CancellationPolicy;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SchedulerInput {
    pub graph: TaskGraph,
    pub mode: SchedulerMode,
    pub priority_hints: PriorityHints,
    pub cache: CacheSchedulingPolicy,
    pub cache_decisions: CacheSchedulingPlan,
    pub resource_budget: ResourceBudget,
    pub cancellation: CancellationPolicy,
    pub task_outcomes: Vec<SyntheticTaskOutcome>,
    pub worker_count: usize,
    pub completion_order: CompletionOrder,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SchedulerRun {
    pub graph_version: TaskGraphVersion,
    pub snapshot: BuildSnapshotId,
    pub task_states: Vec<TaskStateRecord>,
    pub results: Vec<SchedulerResult>,
    pub failure_records: Vec<BuildFailureRecord>,
    pub blocked_records: Vec<BlockedTaskRecord>,
    pub events: Vec<SchedulerEvent>,
    pub resource_telemetry: Vec<ResourceTelemetry>,
    pub diagnostics: Vec<SchedulerDiagnostic>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskStateRecord {
    pub task_id: TaskId,
    pub state: TaskState,
    pub dependencies: Vec<TaskId>,
    pub blocked_by: Vec<TaskId>,
    pub queue: SchedulerQueue,
    pub dependency_coverage: DependencyCoverage,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SchedulerResult {
    pub task_id: TaskId,
    pub state: TaskState,
    pub canonical_order: SchedulerOrderKey,
    pub output_refs: Vec<SyntheticOutputRef>,
    pub diagnostics: Vec<SchedulerDiagnosticRef>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SchedulerEvent {
    pub kind: SchedulerEventKind,
    pub task_id: Option<TaskId>,
    pub order: SchedulerOrderKey,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum TaskState {
    Pending,
    Ready,
    Running,
    Completed,
    CacheHit,
    Skipped,
    Failed,
    Blocked,
    Cancelled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum SchedulerMode {
    Batch,
    Watch,
    Lsp,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PriorityHints {
    pub preferred_tasks: Vec<TaskId>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum CacheSchedulingPolicy {
    Disabled,
    Enabled,
    Miss,
    Unavailable,
    ErrorAsMiss,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyntheticTaskOutcome {
    pub task_id: TaskId,
    pub status: SyntheticTaskStatus,
    pub outputs: Vec<SyntheticOutputRef>,
    pub diagnostics: Vec<SchedulerDiagnosticRef>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum SyntheticTaskStatus {
    Complete,
    Fail,
    Skip,
}

pub trait SchedulerTaskDispatcher {
    fn dispatch(&mut self, task: SchedulerDispatchTask<'_>) -> SchedulerDispatchOutcome;
}

impl<F> SchedulerTaskDispatcher for F
where
    F: FnMut(SchedulerDispatchTask<'_>) -> SchedulerDispatchOutcome,
{
    fn dispatch(&mut self, task: SchedulerDispatchTask<'_>) -> SchedulerDispatchOutcome {
        self(task)
    }
}

#[derive(Debug, Clone)]
pub struct SchedulerDispatchTask<'a> {
    pub task: &'a BuildTask,
    pub snapshot: BuildSnapshotId,
    pub cancellation: Option<CancellationToken>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SchedulerDispatchOutcome {
    pub status: SchedulerDispatchStatus,
    pub diagnostics: Vec<SchedulerDiagnosticRef>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum SchedulerDispatchStatus {
    Complete,
    Failed,
    Blocked,
    Skipped,
    Cancelled,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct SyntheticOutputRef {
    pub identity: String,
    pub content: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct SchedulerDiagnosticRef {
    pub source: String,
    pub code: String,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ExecutedTaskOutcome {
    state: TaskState,
    output_refs: Vec<SyntheticOutputRef>,
    diagnostics: Vec<SchedulerDiagnosticRef>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum CompletionOrder {
    Canonical,
    Reverse,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
pub enum SchedulerQueue {
    Coordinator,
    SourceLocalCpu,
    DeterministicProof,
    AtpPortfolio,
    AtpProcess,
    Kernel,
    IoCommit,
    Documentation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
pub enum SchedulerEventKind {
    TaskBecameReady,
    TaskStarted,
    TaskSkipped,
    TaskBlocked,
    TaskFinished,
    RunFinished,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct SchedulerOrderKey {
    pub graph_index: usize,
    pub lifecycle_rank: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SchedulerDiagnostics {
    diagnostics: Vec<SchedulerDiagnostic>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SchedulerDiagnostic {
    pub task_id: Option<TaskId>,
    pub kind: SchedulerDiagnosticKind,
    pub value: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
pub enum SchedulerDiagnosticKind {
    DuplicateCacheDecision,
    DuplicateOutcome,
    UnknownCacheDecision,
    UnknownTask,
    MissingRootTask,
    IncompleteDependencyCoverage,
    NoSchedulablePath,
    ImpossibleResourceRequest,
}

struct SchedulerBuilder {
    input: SchedulerInput,
    graph_order: BTreeMap<TaskId, usize>,
    tasks_by_id: BTreeMap<TaskId, BuildTask>,
    outcomes_by_task: BTreeMap<TaskId, SyntheticTaskOutcome>,
    cache_decisions_by_task: BTreeMap<TaskId, CacheSchedulingOutcome>,
    dependents_by_task: BTreeMap<TaskId, Vec<TaskId>>,
    states: BTreeMap<TaskId, TaskStateRecord>,
    results: BTreeMap<TaskId, SchedulerResult>,
    failure_records: Vec<BuildFailureRecord>,
    blocked_records: Vec<BlockedTaskRecord>,
    events: Vec<SchedulerEvent>,
    resource_manager: ResourceManager,
    cancellation_state: CancellationState,
    resource_telemetry: Vec<ResourceTelemetry>,
    diagnostics: Vec<SchedulerDiagnostic>,
    dispatch_batches: Vec<Vec<TaskId>>,
    admission_counter: usize,
}

impl SchedulerInput {
    #[must_use]
    pub fn new(graph: TaskGraph) -> Self {
        Self {
            graph,
            mode: SchedulerMode::Batch,
            priority_hints: PriorityHints::default(),
            cache: CacheSchedulingPolicy::Disabled,
            cache_decisions: CacheSchedulingPlan::default(),
            resource_budget: ResourceBudget::default(),
            cancellation: CancellationPolicy::default(),
            task_outcomes: Vec::new(),
            worker_count: 1,
            completion_order: CompletionOrder::Canonical,
        }
    }
}

impl SyntheticTaskOutcome {
    #[must_use]
    pub fn complete(task_id: TaskId, outputs: Vec<SyntheticOutputRef>) -> Self {
        Self {
            task_id,
            status: SyntheticTaskStatus::Complete,
            outputs,
            diagnostics: Vec::new(),
        }
    }

    #[must_use]
    pub fn fail(task_id: TaskId, diagnostics: Vec<SchedulerDiagnosticRef>) -> Self {
        Self {
            task_id,
            status: SyntheticTaskStatus::Fail,
            outputs: Vec::new(),
            diagnostics,
        }
    }

    #[must_use]
    pub fn skip(task_id: TaskId) -> Self {
        Self {
            task_id,
            status: SyntheticTaskStatus::Skip,
            outputs: Vec::new(),
            diagnostics: Vec::new(),
        }
    }
}

impl SchedulerDispatchOutcome {
    #[must_use]
    pub fn complete() -> Self {
        Self {
            status: SchedulerDispatchStatus::Complete,
            diagnostics: Vec::new(),
        }
    }

    #[must_use]
    pub fn failed(diagnostics: Vec<SchedulerDiagnosticRef>) -> Self {
        Self {
            status: SchedulerDispatchStatus::Failed,
            diagnostics,
        }
    }

    #[must_use]
    pub fn blocked(diagnostics: Vec<SchedulerDiagnosticRef>) -> Self {
        Self {
            status: SchedulerDispatchStatus::Blocked,
            diagnostics,
        }
    }

    #[must_use]
    pub fn skipped() -> Self {
        Self {
            status: SchedulerDispatchStatus::Skipped,
            diagnostics: Vec::new(),
        }
    }

    #[must_use]
    pub fn cancelled() -> Self {
        Self {
            status: SchedulerDispatchStatus::Cancelled,
            diagnostics: Vec::new(),
        }
    }
}

impl ExecutedTaskOutcome {
    fn new(
        state: TaskState,
        output_refs: Vec<SyntheticOutputRef>,
        diagnostics: Vec<SchedulerDiagnosticRef>,
    ) -> Self {
        Self {
            state,
            output_refs,
            diagnostics,
        }
    }
}

impl SyntheticOutputRef {
    #[must_use]
    pub fn new(identity: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            identity: identity.into(),
            content: content.into(),
        }
    }
}

impl SchedulerDiagnosticRef {
    #[must_use]
    pub fn new(
        source: impl Into<String>,
        code: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            source: source.into(),
            code: code.into(),
            message: message.into(),
        }
    }
}

impl SchedulerDiagnostics {
    #[must_use]
    pub fn new(mut diagnostics: Vec<SchedulerDiagnostic>) -> Self {
        sort_scheduler_diagnostics(&mut diagnostics, &BTreeMap::new());
        Self { diagnostics }
    }

    #[must_use]
    pub fn diagnostics(&self) -> &[SchedulerDiagnostic] {
        &self.diagnostics
    }

    #[must_use]
    pub fn into_diagnostics(self) -> Vec<SchedulerDiagnostic> {
        self.diagnostics
    }
}

pub fn run_scheduler(input: SchedulerInput) -> Result<SchedulerRun, SchedulerDiagnostics> {
    run_scheduler_with_dispatch_batches(input).map(|(run, _dispatch_batches)| run)
}

pub fn run_scheduler_with_dispatcher<D>(
    input: SchedulerInput,
    dispatcher: &mut D,
) -> Result<SchedulerRun, SchedulerDiagnostics>
where
    D: SchedulerTaskDispatcher,
{
    let dispatcher: &mut dyn SchedulerTaskDispatcher = dispatcher;
    SchedulerBuilder::new(input)
        .run(Some(dispatcher))
        .map(|(run, _dispatch_batches)| run)
}

impl SchedulerBuilder {
    fn new(input: SchedulerInput) -> Self {
        let graph_order = input
            .graph
            .tasks
            .iter()
            .enumerate()
            .map(|(index, task)| (task.id.clone(), index))
            .collect::<BTreeMap<_, _>>();
        let tasks_by_id = input
            .graph
            .tasks
            .iter()
            .map(|task| (task.id.clone(), task.clone()))
            .collect::<BTreeMap<_, _>>();
        let dependents_by_task = dependents_by_task(&input.graph);
        let resource_manager = ResourceManager::new(input.resource_budget.clone());
        let cancellation_state = CancellationState::from_policy(&input.cancellation, &input.graph);
        Self {
            input,
            graph_order,
            tasks_by_id,
            outcomes_by_task: BTreeMap::new(),
            cache_decisions_by_task: BTreeMap::new(),
            dependents_by_task,
            states: BTreeMap::new(),
            results: BTreeMap::new(),
            failure_records: Vec::new(),
            blocked_records: Vec::new(),
            events: Vec::new(),
            resource_manager,
            cancellation_state,
            resource_telemetry: Vec::new(),
            diagnostics: Vec::new(),
            dispatch_batches: Vec::new(),
            admission_counter: 0,
        }
    }

    fn run(
        mut self,
        dispatcher: Option<&mut dyn SchedulerTaskDispatcher>,
    ) -> Result<(SchedulerRun, Vec<Vec<TaskId>>), SchedulerDiagnostics> {
        self.validate_outcomes();
        self.validate_cache_decisions();
        if !self.diagnostics.is_empty() {
            sort_scheduler_diagnostics(&mut self.diagnostics, &self.graph_order);
            return Err(SchedulerDiagnostics {
                diagnostics: self.diagnostics,
            });
        }

        self.initialize_states();
        if self
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.kind == SchedulerDiagnosticKind::MissingRootTask)
        {
            sort_scheduler_diagnostics(&mut self.diagnostics, &self.graph_order);
            return Err(SchedulerDiagnostics {
                diagnostics: self.diagnostics,
            });
        }

        self.process_ready_queue(dispatcher);
        self.block_remaining_pending_tasks();
        self.events.push(SchedulerEvent {
            kind: SchedulerEventKind::RunFinished,
            task_id: None,
            order: SchedulerOrderKey {
                graph_index: usize::MAX,
                lifecycle_rank: lifecycle_rank(SchedulerEventKind::RunFinished),
            },
        });

        let graph_order = self.graph_order.clone();
        let mut task_states = self.states.into_values().collect::<Vec<_>>();
        task_states.sort_by_key(|record| {
            graph_order
                .get(&record.task_id)
                .copied()
                .unwrap_or(usize::MAX)
        });
        let mut results = self.results.into_values().collect::<Vec<_>>();
        results.sort_by_key(|result| (result.canonical_order, result.task_id.as_str().to_owned()));
        let mut failure_records = self.failure_records;
        failure_records.sort_by_key(BuildFailureRecord::sort_key);
        let mut blocked_records = self.blocked_records;
        blocked_records.sort_by_key(BlockedTaskRecord::sort_key);
        self.events.sort_by_key(|event| {
            (
                event.order,
                event
                    .task_id
                    .as_ref()
                    .map(|task_id| task_id.as_str().to_owned())
                    .unwrap_or_default(),
            )
        });
        self.resource_telemetry.sort_by_key(|telemetry| {
            telemetry.sort_key(
                graph_order
                    .get(&telemetry.task_id)
                    .copied()
                    .unwrap_or(usize::MAX),
            )
        });
        sort_scheduler_diagnostics(&mut self.diagnostics, &graph_order);

        Ok((
            SchedulerRun {
                graph_version: self.input.graph.version,
                snapshot: self.input.graph.snapshot,
                task_states,
                results,
                failure_records,
                blocked_records,
                events: self.events,
                resource_telemetry: self.resource_telemetry,
                diagnostics: self.diagnostics,
            },
            self.dispatch_batches,
        ))
    }

    fn validate_outcomes(&mut self) {
        let mut seen = BTreeSet::new();
        for outcome in &self.input.task_outcomes {
            if !self.tasks_by_id.contains_key(&outcome.task_id) {
                self.diagnostics.push(SchedulerDiagnostic {
                    task_id: Some(outcome.task_id.clone()),
                    kind: SchedulerDiagnosticKind::UnknownTask,
                    value: Some(outcome.task_id.as_str().to_owned()),
                });
                continue;
            }
            if !seen.insert(outcome.task_id.clone()) {
                self.diagnostics.push(SchedulerDiagnostic {
                    task_id: Some(outcome.task_id.clone()),
                    kind: SchedulerDiagnosticKind::DuplicateOutcome,
                    value: Some(outcome.task_id.as_str().to_owned()),
                });
                continue;
            }
            self.outcomes_by_task
                .insert(outcome.task_id.clone(), outcome.clone());
        }
    }

    fn validate_cache_decisions(&mut self) {
        let known_tasks = self.tasks_by_id.keys().cloned().collect::<BTreeSet<_>>();
        match self
            .input
            .cache_decisions
            .validated_decision_map(&known_tasks)
        {
            Ok(decisions) => {
                self.cache_decisions_by_task = decisions;
            }
            Err(diagnostics) => {
                self.diagnostics
                    .extend(
                        diagnostics
                            .into_diagnostics()
                            .into_iter()
                            .map(|diagnostic| SchedulerDiagnostic {
                                task_id: Some(diagnostic.task_id.clone()),
                                kind: match diagnostic.kind {
                                    CacheSchedulingPlanDiagnosticKind::DuplicateDecision => {
                                        SchedulerDiagnosticKind::DuplicateCacheDecision
                                    }
                                    CacheSchedulingPlanDiagnosticKind::UnknownTask => {
                                        SchedulerDiagnosticKind::UnknownCacheDecision
                                    }
                                },
                                value: Some(diagnostic.task_id.as_str().to_owned()),
                            }),
                    );
            }
        }
    }

    fn initialize_states(&mut self) {
        let mut root_count = 0_usize;
        for task in self.input.graph.tasks.clone() {
            let queue = scheduler_queue(&task);
            let mut record = TaskStateRecord {
                task_id: task.id.clone(),
                state: TaskState::Pending,
                dependencies: task.dependencies.clone(),
                blocked_by: Vec::new(),
                queue,
                dependency_coverage: task.dependency_coverage,
            };

            let cancelled = self.cancellation_decision(&task, CancellationCheckpoint::Pending)
                == CancellationDecision::CancelBeforeStart;

            if task.kind == TaskKind::PackageResolve {
                root_count += 1;
            }

            if cancelled {
                record.state = TaskState::Cancelled;
                self.record_result(&task, TaskState::Cancelled);
                self.events.push(event_for_task(
                    SchedulerEventKind::TaskFinished,
                    &task.id,
                    self.graph_index(&task.id),
                ));
            } else if task.kind == TaskKind::PackageResolve {
                record.state = TaskState::Completed;
                self.record_result(&task, TaskState::Completed);
                self.events.push(event_for_task(
                    SchedulerEventKind::TaskFinished,
                    &task.id,
                    self.graph_index(&task.id),
                ));
            }

            self.states.insert(task.id, record);
        }

        if root_count == 0 {
            self.diagnostics.push(SchedulerDiagnostic {
                task_id: None,
                kind: SchedulerDiagnosticKind::MissingRootTask,
                value: Some("PackageResolve".to_owned()),
            });
        }
    }

    fn process_ready_queue(&mut self, mut dispatcher: Option<&mut dyn SchedulerTaskDispatcher>) {
        let uses_real_dispatcher = dispatcher.is_some();
        loop {
            let mut ready = self.new_ready_tasks();
            if ready.is_empty() {
                break;
            }

            let mut batch = Vec::new();
            let mut made_progress = false;
            while batch.len() < self.worker_count() {
                let Some(task_id) = ready.pop_front() else {
                    break;
                };
                if self
                    .states
                    .get(&task_id)
                    .is_some_and(|record| record.state == TaskState::Ready)
                {
                    let task = self
                        .tasks_by_id
                        .get(&task_id)
                        .expect("ready task has task metadata")
                        .clone();
                    if self.cancel_before_start_if_requested(&task) {
                        made_progress = true;
                        self.block_dependents(&task_id);
                        continue;
                    }
                    if self.apply_cache_decision_if_hit(&task) {
                        made_progress = true;
                        if self.states.get(&task_id).is_some_and(|record| {
                            matches!(record.state, TaskState::Cancelled | TaskState::Failed)
                        }) {
                            self.block_dependents(&task_id);
                        }
                        continue;
                    }
                    match self.try_admit_ready_task(&task) {
                        ResourceAdmissionStatus::Admitted => {
                            made_progress = true;
                            self.events.push(event_for_task(
                                SchedulerEventKind::TaskStarted,
                                &task_id,
                                self.graph_index(&task_id),
                            ));
                            self.set_state(&task_id, TaskState::Running);
                            batch.push(task_id);
                        }
                        ResourceAdmissionStatus::Delayed => {}
                        ResourceAdmissionStatus::Impossible => {
                            made_progress = true;
                        }
                        ResourceAdmissionStatus::Released => {}
                    }
                }
            }

            if batch.is_empty() {
                if made_progress {
                    continue;
                }
                break;
            }

            self.dispatch_batches.push(batch.clone());
            if !uses_real_dispatcher && self.input.completion_order == CompletionOrder::Reverse {
                batch.reverse();
            }

            for task_id in batch {
                if self
                    .states
                    .get(&task_id)
                    .is_some_and(|record| record.state != TaskState::Running)
                {
                    continue;
                }
                let task = self
                    .tasks_by_id
                    .get(&task_id)
                    .expect("running task has task metadata")
                    .clone();

                let mut execution = match dispatcher.as_deref_mut() {
                    Some(dispatcher) => self.dispatch_task(&task, dispatcher),
                    None => self.execute_task(&task),
                };
                execution.state = self.apply_publication_freshness(&task, execution.state);
                if execution.state == TaskState::Blocked {
                    self.record_dispatch_block(&task, execution.diagnostics);
                    self.release_task_resources(&task_id);
                    self.block_dependents(&task_id);
                    continue;
                }
                let final_state = execution.state;
                self.set_state(&task_id, final_state);
                self.record_execution_result(&task, execution);
                self.events.push(event_for_task(
                    if final_state == TaskState::Skipped {
                        SchedulerEventKind::TaskSkipped
                    } else {
                        SchedulerEventKind::TaskFinished
                    },
                    &task_id,
                    self.graph_index(&task_id),
                ));
                self.release_task_resources(&task_id);

                if final_state == TaskState::Failed || final_state == TaskState::Cancelled {
                    self.block_dependents(&task_id);
                }
            }
        }
    }

    fn execute_task(&self, task: &BuildTask) -> ExecutedTaskOutcome {
        match self.input.cache {
            CacheSchedulingPolicy::Disabled
            | CacheSchedulingPolicy::Enabled
            | CacheSchedulingPolicy::Miss
            | CacheSchedulingPolicy::Unavailable
            | CacheSchedulingPolicy::ErrorAsMiss => {}
        }

        if self.cancellation_decision(task, CancellationCheckpoint::RunningSafeCheckpoint)
            == CancellationDecision::CancelAtCheckpoint
        {
            return ExecutedTaskOutcome::new(TaskState::Cancelled, Vec::new(), Vec::new());
        }

        let state = match self
            .outcomes_by_task
            .get(&task.id)
            .map(|outcome| outcome.status)
        {
            Some(SyntheticTaskStatus::Fail) => TaskState::Failed,
            Some(SyntheticTaskStatus::Skip) => TaskState::Skipped,
            Some(SyntheticTaskStatus::Complete) | None if self.skip_due_to_dependency(task) => {
                TaskState::Skipped
            }
            Some(SyntheticTaskStatus::Complete) | None => TaskState::Completed,
        };
        let (output_refs, diagnostics) = self.synthetic_payload_for_task(task, state);
        ExecutedTaskOutcome::new(state, output_refs, diagnostics)
    }

    fn dispatch_task(
        &self,
        task: &BuildTask,
        dispatcher: &mut dyn SchedulerTaskDispatcher,
    ) -> ExecutedTaskOutcome {
        if self.cancellation_decision(task, CancellationCheckpoint::RunningSafeCheckpoint)
            == CancellationDecision::CancelAtCheckpoint
        {
            return ExecutedTaskOutcome::new(TaskState::Cancelled, Vec::new(), Vec::new());
        }
        if self.skip_due_to_dependency(task) {
            return ExecutedTaskOutcome::new(TaskState::Skipped, Vec::new(), Vec::new());
        }

        let outcome = dispatcher.dispatch(SchedulerDispatchTask {
            task,
            snapshot: self.input.graph.snapshot,
            cancellation: self.cancellation_state.token_for_task(&task.id),
        });
        let state = match outcome.status {
            SchedulerDispatchStatus::Complete => TaskState::Completed,
            SchedulerDispatchStatus::Failed => TaskState::Failed,
            SchedulerDispatchStatus::Blocked => TaskState::Blocked,
            SchedulerDispatchStatus::Skipped => TaskState::Skipped,
            SchedulerDispatchStatus::Cancelled => TaskState::Cancelled,
        };
        let mut diagnostics = outcome.diagnostics;
        diagnostics.sort();
        diagnostics.dedup();
        ExecutedTaskOutcome::new(state, Vec::new(), diagnostics)
    }

    fn apply_cache_decision_if_hit(&mut self, task: &BuildTask) -> bool {
        if self.input.cache != CacheSchedulingPolicy::Enabled {
            return false;
        }
        let Some(decision) = self.cache_decisions_by_task.get(&task.id).cloned() else {
            return false;
        };
        let CacheSchedulingOutcome::ValidatedHit(hit) = decision else {
            return false;
        };

        let final_state = self.apply_publication_freshness(task, TaskState::CacheHit);
        self.set_state(&task.id, final_state);
        if final_state == TaskState::CacheHit {
            self.record_cache_hit_result(task, hit);
        } else {
            self.record_result(task, final_state);
        }
        self.events.push(event_for_task(
            SchedulerEventKind::TaskFinished,
            &task.id,
            self.graph_index(&task.id),
        ));
        true
    }

    fn new_ready_tasks(&mut self) -> VecDeque<TaskId> {
        let mut ready = Vec::new();
        let task_ids = self
            .input
            .graph
            .tasks
            .iter()
            .map(|task| task.id.clone())
            .collect::<Vec<_>>();

        for task_id in task_ids {
            let Some(record) = self.states.get(&task_id) else {
                continue;
            };
            if record.state == TaskState::Ready {
                ready.push(task_id);
                continue;
            }
            if record.state != TaskState::Pending {
                continue;
            }

            let task = self
                .tasks_by_id
                .get(&task_id)
                .expect("state record has task metadata")
                .clone();
            if self.coverage_blocks(&task) {
                if self.block_task(
                    &task.id,
                    Vec::new(),
                    BlockReason::MissingDependencyCoverage,
                    Some((
                        SchedulerDiagnosticKind::IncompleteDependencyCoverage,
                        "dependency coverage is incomplete".to_owned(),
                    )),
                ) {
                    self.block_dependents(&task.id);
                }
                continue;
            }

            let dependency_states = task
                .dependencies
                .iter()
                .map(|dependency| {
                    let state = self
                        .states
                        .get(dependency)
                        .map(|record| record.state)
                        .unwrap_or(TaskState::Blocked);
                    (dependency.clone(), state)
                })
                .collect::<Vec<_>>();

            let blocking_dependencies = dependency_states
                .iter()
                .filter(|(_dependency, state)| blocking_terminal(*state))
                .map(|(dependency, _state)| dependency.clone())
                .collect::<Vec<_>>();
            if !blocking_dependencies.is_empty() {
                let reason = primary_dependency_block_reason(&dependency_states);
                if self.block_task(&task.id, blocking_dependencies, reason, None) {
                    self.block_dependents(&task.id);
                }
                continue;
            }

            if dependency_states.iter().all(|(dependency, state)| {
                self.unblocking_dependency(dependency, *state, task.kind)
            }) {
                if self.cancel_before_start_if_requested(&task) {
                    self.block_dependents(&task.id);
                    continue;
                }
                self.set_state(&task.id, TaskState::Ready);
                self.events.push(event_for_task(
                    SchedulerEventKind::TaskBecameReady,
                    &task.id,
                    self.graph_index(&task.id),
                ));
                ready.push(task.id.clone());
            }
        }

        self.sort_ready_tasks(&mut ready);
        ready.into()
    }

    fn sort_ready_tasks(&self, ready: &mut [TaskId]) {
        let preferred = self
            .input
            .priority_hints
            .preferred_tasks
            .iter()
            .enumerate()
            .map(|(index, task_id)| (task_id.clone(), index))
            .collect::<BTreeMap<_, _>>();
        ready.sort_by_key(|task_id| {
            (
                self.mode_priority_rank(task_id),
                preferred.get(task_id).copied().unwrap_or(usize::MAX),
                self.downstream_rank(task_id),
                self.task_kind_rank(task_id),
                self.resource_queue_rank(task_id),
                self.graph_index(task_id),
                task_id.as_str().to_owned(),
            )
        });
    }

    fn try_admit_ready_task(&mut self, task: &BuildTask) -> ResourceAdmissionStatus {
        let request = TaskResourceRequest::for_task(task, scheduler_queue(task));
        let admission_order = self.next_admission_order();
        let admission = self.resource_manager.try_admit(request, admission_order);
        let status = admission.status;
        let blocking_scope = admission.blocking_scope.clone();
        self.resource_telemetry.push(admission.telemetry());

        if status == ResourceAdmissionStatus::Impossible {
            let value = blocking_scope
                .map(|scope| scope.stable_label())
                .unwrap_or_else(|| "resource request cannot be admitted".to_owned());
            if self.block_task(
                &task.id,
                Vec::new(),
                BlockReason::ImpossibleResourceRequest,
                Some((SchedulerDiagnosticKind::ImpossibleResourceRequest, value)),
            ) {
                self.block_dependents(&task.id);
            }
        }

        status
    }

    fn release_task_resources(&mut self, task_id: &TaskId) {
        if let Some((request, admission_order)) = self.resource_manager.release(task_id) {
            self.resource_telemetry.push(ResourceTelemetry {
                task_id: request.task_id,
                queue: request.queue,
                status: ResourceAdmissionStatus::Released,
                requested_units: request.units,
                blocking_scope: None,
                admission_order,
            });
        }
    }

    fn cancel_before_start_if_requested(&mut self, task: &BuildTask) -> bool {
        if self.cancellation_decision(task, CancellationCheckpoint::Ready)
            != CancellationDecision::CancelBeforeStart
        {
            return false;
        }

        self.set_state(&task.id, TaskState::Cancelled);
        self.record_result(task, TaskState::Cancelled);
        self.events.push(event_for_task(
            SchedulerEventKind::TaskFinished,
            &task.id,
            self.graph_index(&task.id),
        ));
        self.release_task_resources(&task.id);
        true
    }

    fn apply_publication_freshness(&self, task: &BuildTask, state: TaskState) -> TaskState {
        if matches!(
            state,
            TaskState::Completed | TaskState::Failed | TaskState::Skipped | TaskState::CacheHit
        ) && self.cancellation_decision(task, CancellationCheckpoint::CompletedBeforePublication)
            == CancellationDecision::DiscardObsoleteResult
        {
            return TaskState::Cancelled;
        }

        state
    }

    fn cancellation_decision(
        &self,
        task: &BuildTask,
        checkpoint: CancellationCheckpoint,
    ) -> CancellationDecision {
        self.cancellation_state
            .decision_for_checkpoint(task, self.graph_index(&task.id), checkpoint)
            .decision
    }

    fn next_admission_order(&mut self) -> usize {
        let order = self.admission_counter;
        self.admission_counter += 1;
        order
    }

    fn coverage_blocks(&self, task: &BuildTask) -> bool {
        matches!(
            task.dependency_coverage,
            DependencyCoverage::MissingModuleDependencyOverlay
                | DependencyCoverage::MissingVcDescriptors
        )
    }

    fn block_dependents(&mut self, task_id: &TaskId) {
        let mut queue = self
            .dependents_by_task
            .get(task_id)
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .map(|dependent| (dependent, task_id.clone()))
            .collect::<VecDeque<_>>();
        while let Some((dependent, blocker)) = queue.pop_front() {
            if self.states.get(&dependent).is_some_and(|record| {
                matches!(record.state, TaskState::Pending | TaskState::Blocked)
            }) {
                let reason = self
                    .states
                    .get(&blocker)
                    .and_then(|record| block_reason_for_terminal(record.state))
                    .unwrap_or(BlockReason::DependencyBlocked);
                if !self.block_task(&dependent, vec![blocker], reason, None) {
                    continue;
                }
                if let Some(next_dependents) = self.dependents_by_task.get(&dependent) {
                    for next in next_dependents {
                        queue.push_back((next.clone(), dependent.clone()));
                    }
                }
            }
        }
    }

    fn block_remaining_pending_tasks(&mut self) {
        let pending = self
            .states
            .iter()
            .filter(|(_task_id, record)| record.state == TaskState::Pending)
            .map(|(task_id, _record)| task_id.clone())
            .collect::<Vec<_>>();
        for task_id in pending {
            self.block_task(
                &task_id,
                Vec::new(),
                BlockReason::NoSchedulablePath,
                Some((
                    SchedulerDiagnosticKind::NoSchedulablePath,
                    "no schedulable path".to_owned(),
                )),
            );
        }
    }

    fn block_task(
        &mut self,
        task_id: &TaskId,
        blocked_by: Vec<TaskId>,
        reason: BlockReason,
        diagnostic: Option<(SchedulerDiagnosticKind, String)>,
    ) -> bool {
        let blocked_by = normalized_task_ids(blocked_by);
        let mut merge_existing = None;
        let mut already_blocked = false;
        if let Some(record) = self.states.get_mut(task_id) {
            if record.state == TaskState::Blocked {
                already_blocked = true;
                if !blocked_by.is_empty() && !record.blocked_by.is_empty() {
                    let mut merged = record.blocked_by.clone();
                    merged.extend(blocked_by.clone());
                    let merged = normalized_task_ids(merged);
                    record.blocked_by = merged.clone();
                    merge_existing = Some(merged);
                }
            } else {
                record.state = TaskState::Blocked;
                record.blocked_by = blocked_by.clone();
            }
        }
        if let Some(merged) = merge_existing {
            self.merge_blocked_record(task_id, merged, reason);
            return false;
        }
        if already_blocked {
            return false;
        }
        let task = self
            .tasks_by_id
            .get(task_id)
            .expect("blocked task has task metadata")
            .clone();
        self.record_result(&task, TaskState::Blocked);
        self.blocked_records.push(BlockedTaskRecord::new(
            task.id.clone(),
            self.input.graph.snapshot,
            blocked_by,
            reason,
            SchedulerOrderKey {
                graph_index: self.graph_index(task_id),
                lifecycle_rank: lifecycle_rank(SchedulerEventKind::TaskBlocked),
            },
        ));
        self.events.push(event_for_task(
            SchedulerEventKind::TaskBlocked,
            task_id,
            self.graph_index(task_id),
        ));
        if let Some((kind, value)) = diagnostic {
            self.diagnostics.push(SchedulerDiagnostic {
                task_id: Some(task_id.clone()),
                kind,
                value: Some(value),
            });
        }
        true
    }

    fn merge_blocked_record(
        &mut self,
        task_id: &TaskId,
        blocked_by: Vec<TaskId>,
        reason: BlockReason,
    ) {
        let blocked_by = normalized_task_ids(blocked_by);
        if let Some(record) = self
            .blocked_records
            .iter_mut()
            .find(|record| &record.task_id == task_id)
        {
            record.blocked_by = blocked_by;
            record.reason = record.reason.min(reason);
        }
    }

    fn set_state(&mut self, task_id: &TaskId, state: TaskState) {
        if let Some(record) = self.states.get_mut(task_id) {
            record.state = state;
        }
    }

    fn record_result(&mut self, task: &BuildTask, state: TaskState) {
        let (output_refs, diagnostics) = self.synthetic_payload_for_task(task, state);
        self.insert_result(task, state, output_refs, diagnostics);
    }

    fn synthetic_payload_for_task(
        &self,
        task: &BuildTask,
        state: TaskState,
    ) -> (Vec<SyntheticOutputRef>, Vec<SchedulerDiagnosticRef>) {
        let outcome = self.outcomes_by_task.get(&task.id);
        let mut outputs = outcome
            .map(|outcome| outcome.outputs.clone())
            .unwrap_or_else(|| vec![default_output_for_task(task)]);
        outputs.sort();
        outputs.dedup();
        let mut diagnostics = outcome
            .map(|outcome| outcome.diagnostics.clone())
            .unwrap_or_default();
        diagnostics.sort();
        diagnostics.dedup();
        match state {
            TaskState::Completed => (outputs, diagnostics),
            TaskState::Failed => (Vec::new(), diagnostics),
            TaskState::Pending
            | TaskState::Ready
            | TaskState::Running
            | TaskState::CacheHit
            | TaskState::Skipped
            | TaskState::Blocked
            | TaskState::Cancelled => (Vec::new(), Vec::new()),
        }
    }

    fn record_execution_result(&mut self, task: &BuildTask, execution: ExecutedTaskOutcome) {
        self.insert_result(
            task,
            execution.state,
            execution.output_refs,
            execution.diagnostics,
        );
    }

    fn record_dispatch_block(
        &mut self,
        task: &BuildTask,
        diagnostics: Vec<SchedulerDiagnosticRef>,
    ) {
        if self.block_task(
            &task.id,
            Vec::new(),
            BlockReason::PhaseDispatchBlocked,
            None,
        ) {
            self.insert_result(task, TaskState::Blocked, Vec::new(), diagnostics);
        }
    }

    fn insert_result(
        &mut self,
        task: &BuildTask,
        state: TaskState,
        output_refs: Vec<SyntheticOutputRef>,
        diagnostics: Vec<SchedulerDiagnosticRef>,
    ) {
        let (mut output_refs, mut diagnostics) = match state {
            TaskState::Completed => (output_refs, diagnostics),
            TaskState::Failed | TaskState::Blocked => (Vec::new(), diagnostics),
            TaskState::Pending
            | TaskState::Ready
            | TaskState::Running
            | TaskState::CacheHit
            | TaskState::Skipped
            | TaskState::Cancelled => (Vec::new(), Vec::new()),
        };
        output_refs.sort();
        output_refs.dedup();
        diagnostics.sort();
        diagnostics.dedup();
        let canonical_order = SchedulerOrderKey {
            graph_index: self.graph_index(&task.id),
            lifecycle_rank: lifecycle_rank(SchedulerEventKind::TaskFinished),
        };
        if state == TaskState::Failed {
            self.failure_records
                .extend(BuildFailureRecord::from_failed_task(
                    task,
                    self.input.graph.snapshot,
                    canonical_order,
                    &diagnostics,
                ));
        }
        self.results.insert(
            task.id.clone(),
            SchedulerResult {
                task_id: task.id.clone(),
                state,
                canonical_order,
                output_refs,
                diagnostics,
            },
        );
    }

    fn record_cache_hit_result(&mut self, task: &BuildTask, hit: ValidatedCacheHit) {
        let mut output_refs = hit
            .output_refs
            .into_iter()
            .map(|output| SyntheticOutputRef::new(output.identity, output.content))
            .collect::<Vec<_>>();
        output_refs.sort();
        output_refs.dedup();
        let mut diagnostics = hit
            .diagnostics
            .into_iter()
            .map(|diagnostic| {
                SchedulerDiagnosticRef::new(diagnostic.source, diagnostic.code, diagnostic.message)
            })
            .collect::<Vec<_>>();
        diagnostics.sort();
        diagnostics.dedup();
        self.results.insert(
            task.id.clone(),
            SchedulerResult {
                task_id: task.id.clone(),
                state: TaskState::CacheHit,
                canonical_order: SchedulerOrderKey {
                    graph_index: self.graph_index(&task.id),
                    lifecycle_rank: lifecycle_rank(SchedulerEventKind::TaskFinished),
                },
                output_refs,
                diagnostics,
            },
        );
    }

    fn graph_index(&self, task_id: &TaskId) -> usize {
        self.graph_order.get(task_id).copied().unwrap_or(usize::MAX)
    }

    fn worker_count(&self) -> usize {
        self.input.worker_count.max(1)
    }

    fn skip_due_to_dependency(&self, task: &BuildTask) -> bool {
        matches!(task.kind, TaskKind::BackendRun | TaskKind::KernelCheck)
            && task.dependencies.iter().any(|dependency| {
                self.states
                    .get(dependency)
                    .is_some_and(|record| record.state == TaskState::Skipped)
            })
    }

    fn unblocking_dependency(
        &self,
        dependency: &TaskId,
        state: TaskState,
        dependent_kind: TaskKind,
    ) -> bool {
        match state {
            TaskState::Completed | TaskState::CacheHit => true,
            TaskState::Skipped => self.skipped_dependency_unblocks(dependency, dependent_kind),
            TaskState::Pending
            | TaskState::Ready
            | TaskState::Running
            | TaskState::Failed
            | TaskState::Blocked
            | TaskState::Cancelled => false,
        }
    }

    fn skipped_dependency_unblocks(&self, dependency: &TaskId, dependent_kind: TaskKind) -> bool {
        let Some(task) = self.tasks_by_id.get(dependency) else {
            return false;
        };
        match (task.kind, dependent_kind) {
            (TaskKind::AtpSolve, TaskKind::BackendRun | TaskKind::ArtifactCommit) => true,
            (TaskKind::BackendRun, TaskKind::KernelCheck | TaskKind::ArtifactCommit) => {
                self.skip_due_to_dependency(task)
            }
            (TaskKind::KernelCheck, TaskKind::ArtifactCommit) => self.skip_due_to_dependency(task),
            _ => false,
        }
    }

    fn mode_priority_rank(&self, task_id: &TaskId) -> usize {
        let Some(task) = self.tasks_by_id.get(task_id) else {
            return usize::MAX;
        };
        match self.input.mode {
            SchedulerMode::Batch => priority_class_rank(task.priority_class),
            SchedulerMode::Watch | SchedulerMode::Lsp
                if matches!(
                    task.kind,
                    TaskKind::SourceLoad | TaskKind::Frontend | TaskKind::ModuleResolve
                ) =>
            {
                0
            }
            SchedulerMode::Watch | SchedulerMode::Lsp => {
                priority_class_rank(task.priority_class) + 1
            }
        }
    }

    fn downstream_rank(&self, task_id: &TaskId) -> usize {
        usize::MAX
            - self
                .dependents_by_task
                .get(task_id)
                .map(Vec::len)
                .unwrap_or_default()
    }

    fn task_kind_rank(&self, task_id: &TaskId) -> usize {
        self.tasks_by_id
            .get(task_id)
            .map(|task| task_kind_rank(task.kind))
            .unwrap_or(usize::MAX)
    }

    fn resource_queue_rank(&self, task_id: &TaskId) -> usize {
        self.tasks_by_id
            .get(task_id)
            .map(|task| resource_queue_rank(scheduler_queue(task)))
            .unwrap_or(usize::MAX)
    }
}

fn run_scheduler_with_dispatch_batches(
    input: SchedulerInput,
) -> Result<(SchedulerRun, Vec<Vec<TaskId>>), SchedulerDiagnostics> {
    SchedulerBuilder::new(input).run(None)
}

fn dependents_by_task(graph: &TaskGraph) -> BTreeMap<TaskId, Vec<TaskId>> {
    let mut dependents = BTreeMap::<TaskId, Vec<TaskId>>::new();
    for task in &graph.tasks {
        for dependency in &task.dependencies {
            dependents
                .entry(dependency.clone())
                .or_default()
                .push(task.id.clone());
        }
    }
    for dependents in dependents.values_mut() {
        dependents.sort();
        dependents.dedup();
    }
    dependents
}

fn event_for_task(
    kind: SchedulerEventKind,
    task_id: &TaskId,
    graph_index: usize,
) -> SchedulerEvent {
    SchedulerEvent {
        kind,
        task_id: Some(task_id.clone()),
        order: SchedulerOrderKey {
            graph_index,
            lifecycle_rank: lifecycle_rank(kind),
        },
    }
}

fn lifecycle_rank(kind: SchedulerEventKind) -> usize {
    match kind {
        SchedulerEventKind::TaskBecameReady => 0,
        SchedulerEventKind::TaskStarted => 1,
        SchedulerEventKind::TaskSkipped => 2,
        SchedulerEventKind::TaskBlocked => 3,
        SchedulerEventKind::TaskFinished => 4,
        SchedulerEventKind::RunFinished => 5,
    }
}

fn scheduler_queue(task: &BuildTask) -> SchedulerQueue {
    match (task.resource_class, task.kind) {
        (ResourceClass::Coordinator, _) => SchedulerQueue::Coordinator,
        (ResourceClass::SourceIo | ResourceClass::CpuLocal, _) => SchedulerQueue::SourceLocalCpu,
        (ResourceClass::ProofLocal, _) => SchedulerQueue::DeterministicProof,
        (ResourceClass::AtpProcess, TaskKind::AtpSolve) => SchedulerQueue::AtpPortfolio,
        (ResourceClass::AtpProcess, _) => SchedulerQueue::AtpProcess,
        (ResourceClass::Kernel, _) => SchedulerQueue::Kernel,
        (ResourceClass::ArtifactIo, _) => SchedulerQueue::IoCommit,
        (ResourceClass::Documentation, _) => SchedulerQueue::Documentation,
    }
}

fn blocking_terminal(state: TaskState) -> bool {
    matches!(
        state,
        TaskState::Failed | TaskState::Blocked | TaskState::Cancelled
    )
}

fn block_reason_for_terminal(state: TaskState) -> Option<BlockReason> {
    match state {
        TaskState::Failed => Some(BlockReason::DependencyFailed),
        TaskState::Blocked => Some(BlockReason::DependencyBlocked),
        TaskState::Cancelled => Some(BlockReason::DependencyCancelled),
        TaskState::Pending
        | TaskState::Ready
        | TaskState::Running
        | TaskState::Completed
        | TaskState::CacheHit
        | TaskState::Skipped => None,
    }
}

fn primary_dependency_block_reason(dependency_states: &[(TaskId, TaskState)]) -> BlockReason {
    dependency_states
        .iter()
        .filter_map(|(_dependency, state)| block_reason_for_terminal(*state))
        .min()
        .unwrap_or(BlockReason::DependencyBlocked)
}

fn normalized_task_ids(mut task_ids: Vec<TaskId>) -> Vec<TaskId> {
    task_ids.sort();
    task_ids.dedup();
    task_ids
}

fn default_output_for_task(task: &BuildTask) -> SyntheticOutputRef {
    SyntheticOutputRef::new(
        format!("synthetic-output:{}", task.id.as_str()),
        format!("{:?}", task.kind),
    )
}

fn priority_class_rank(priority_class: PriorityClass) -> usize {
    match priority_class {
        PriorityClass::Root => 0,
        PriorityClass::Source => 1,
        PriorityClass::Semantic => 2,
        PriorityClass::Proof => 3,
        PriorityClass::Commit => 4,
        PriorityClass::Documentation => 5,
    }
}

fn task_kind_rank(kind: TaskKind) -> usize {
    match kind {
        TaskKind::PackageResolve => 0,
        TaskKind::SourceLoad => 1,
        TaskKind::Frontend => 2,
        TaskKind::ModuleResolve => 3,
        TaskKind::CheckAndElaborate => 4,
        TaskKind::VcGenerate => 5,
        TaskKind::VcDischarge => 6,
        TaskKind::AtpSolve => 7,
        TaskKind::BackendRun => 8,
        TaskKind::KernelCheck => 9,
        TaskKind::ArtifactCommit => 10,
        TaskKind::DocumentationExtract => 11,
    }
}

fn sort_scheduler_diagnostics(
    diagnostics: &mut [SchedulerDiagnostic],
    graph_order: &BTreeMap<TaskId, usize>,
) {
    diagnostics.sort_by_key(|diagnostic| {
        (
            diagnostic
                .task_id
                .as_ref()
                .and_then(|task_id| graph_order.get(task_id).copied())
                .unwrap_or(usize::MAX),
            diagnostic.kind,
            diagnostic
                .task_id
                .as_ref()
                .map(|task_id| task_id.as_str().to_owned())
                .unwrap_or_default(),
            diagnostic.value.clone().unwrap_or_default(),
        )
    });
}

#[cfg(test)]
mod tests;
