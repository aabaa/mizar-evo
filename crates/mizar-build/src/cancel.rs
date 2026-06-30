use std::collections::{BTreeMap, BTreeSet};

use crate::task_graph::{BuildTask, TaskGraph, TaskId, TaskKind};
use mizar_session::BuildSnapshotId;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct CancellationGeneration(u64);

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CancellationPolicy {
    current_snapshot: Option<BuildSnapshotId>,
    generation: CancellationGeneration,
    superseded_snapshots: Vec<BuildSnapshotId>,
    cancelled_tasks: Vec<TaskId>,
    ready_cancelled_tasks: Vec<TaskId>,
    checkpoint_cancelled_tasks: Vec<TaskId>,
    obsolete_completed_tasks: Vec<TaskId>,
    commit_started_tasks: Vec<TaskId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CancellationState {
    snapshot: BuildSnapshotId,
    generation: CancellationGeneration,
    obsolete_snapshot: bool,
    cancel_snapshot: bool,
    cancelled_tasks: BTreeSet<TaskId>,
    ready_cancelled_tasks: BTreeSet<TaskId>,
    checkpoint_cancelled_tasks: BTreeSet<TaskId>,
    obsolete_completed_tasks: BTreeSet<TaskId>,
    commit_started_tasks: BTreeSet<TaskId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CancellationToken {
    pub snapshot: BuildSnapshotId,
    pub generation: CancellationGeneration,
    pub reason: CancellationReason,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CancellationTaskDecision {
    pub task_id: TaskId,
    pub snapshot: BuildSnapshotId,
    pub generation: CancellationGeneration,
    pub reason: Option<CancellationReason>,
    pub decision: CancellationDecision,
    pub graph_index: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum CancellationReason {
    SupersededSnapshot,
    ExplicitRequest,
    Shutdown,
    BudgetPolicy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum CancellationDecision {
    Continue,
    CancelBeforeStart,
    CancelAtCheckpoint,
    DiscardObsoleteResult,
    CommitAlreadyStarted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum CancellationCheckpoint {
    Pending,
    Ready,
    RunningSafeCheckpoint,
    CompletedBeforePublication,
    CommitStarted,
}

impl CancellationGeneration {
    #[must_use]
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    #[must_use]
    pub const fn value(self) -> u64 {
        self.0
    }

    #[must_use]
    pub const fn next(self) -> Self {
        Self(self.0 + 1)
    }
}

impl CancellationPolicy {
    #[must_use]
    pub fn current_snapshot(&self) -> Option<BuildSnapshotId> {
        self.current_snapshot
    }

    #[must_use]
    pub const fn generation(&self) -> CancellationGeneration {
        self.generation
    }

    #[must_use]
    pub fn superseded_snapshots(&self) -> &[BuildSnapshotId] {
        &self.superseded_snapshots
    }

    #[must_use]
    pub fn cancelled_tasks(&self) -> &[TaskId] {
        &self.cancelled_tasks
    }

    #[must_use]
    pub fn ready_cancelled_tasks(&self) -> &[TaskId] {
        &self.ready_cancelled_tasks
    }

    #[must_use]
    pub fn checkpoint_cancelled_tasks(&self) -> &[TaskId] {
        &self.checkpoint_cancelled_tasks
    }

    #[must_use]
    pub fn obsolete_completed_tasks(&self) -> &[TaskId] {
        &self.obsolete_completed_tasks
    }

    #[must_use]
    pub fn commit_started_tasks(&self) -> &[TaskId] {
        &self.commit_started_tasks
    }

    pub fn set_current_snapshot(&mut self, snapshot: BuildSnapshotId) {
        let changed = self.current_snapshot != Some(snapshot);
        self.current_snapshot = Some(snapshot);
        self.advance_if(changed);
    }

    #[must_use]
    pub fn with_current_snapshot(mut self, snapshot: BuildSnapshotId) -> Self {
        self.set_current_snapshot(snapshot);
        self
    }

    pub fn supersede_snapshot(&mut self, snapshot: BuildSnapshotId) {
        let inserted = push_unique_snapshot(&mut self.superseded_snapshots, snapshot);
        self.advance_if(inserted);
    }

    pub fn cancel_task(&mut self, task_id: TaskId) {
        let inserted = push_unique_task(&mut self.cancelled_tasks, task_id);
        self.advance_if(inserted);
    }

    pub fn cancel_ready_task(&mut self, task_id: TaskId) {
        let inserted = push_unique_task(&mut self.ready_cancelled_tasks, task_id);
        self.advance_if(inserted);
    }

    pub fn cancel_task_at_checkpoint(&mut self, task_id: TaskId) {
        let inserted = push_unique_task(&mut self.checkpoint_cancelled_tasks, task_id);
        self.advance_if(inserted);
    }

    pub fn discard_obsolete_completed_task(&mut self, task_id: TaskId) {
        let inserted = push_unique_task(&mut self.obsolete_completed_tasks, task_id);
        self.advance_if(inserted);
    }

    pub fn mark_commit_started(&mut self, task_id: TaskId) {
        let inserted = push_unique_task(&mut self.commit_started_tasks, task_id);
        self.advance_if(inserted);
    }

    #[must_use]
    pub fn with_cancelled_task(mut self, task_id: TaskId) -> Self {
        self.cancel_task(task_id);
        self
    }

    #[must_use]
    pub fn with_ready_cancelled_task(mut self, task_id: TaskId) -> Self {
        self.cancel_ready_task(task_id);
        self
    }

    #[must_use]
    pub fn with_checkpoint_cancelled_task(mut self, task_id: TaskId) -> Self {
        self.cancel_task_at_checkpoint(task_id);
        self
    }

    #[must_use]
    pub fn with_obsolete_completed_task(mut self, task_id: TaskId) -> Self {
        self.discard_obsolete_completed_task(task_id);
        self
    }

    #[must_use]
    pub fn with_commit_started_task(mut self, task_id: TaskId) -> Self {
        self.mark_commit_started(task_id);
        self
    }

    fn advance_if(&mut self, inserted: bool) {
        if inserted {
            self.generation = self.generation.next();
        }
    }
}

impl CancellationState {
    #[must_use]
    pub fn from_policy(policy: &CancellationPolicy, graph: &TaskGraph) -> Self {
        let snapshot = graph.snapshot;
        let current_mismatch = policy
            .current_snapshot
            .is_some_and(|current| current != snapshot);
        let superseded = policy
            .superseded_snapshots
            .iter()
            .any(|candidate| candidate == &snapshot);
        Self {
            snapshot,
            generation: policy.generation,
            obsolete_snapshot: current_mismatch || superseded,
            cancel_snapshot: current_mismatch || superseded,
            cancelled_tasks: task_set(&policy.cancelled_tasks),
            ready_cancelled_tasks: task_set(&policy.ready_cancelled_tasks),
            checkpoint_cancelled_tasks: task_set(&policy.checkpoint_cancelled_tasks),
            obsolete_completed_tasks: task_set(&policy.obsolete_completed_tasks),
            commit_started_tasks: task_set(&policy.commit_started_tasks),
        }
    }

    #[must_use]
    pub const fn snapshot(&self) -> BuildSnapshotId {
        self.snapshot
    }

    #[must_use]
    pub const fn generation(&self) -> CancellationGeneration {
        self.generation
    }

    #[must_use]
    pub const fn obsolete_snapshot(&self) -> bool {
        self.obsolete_snapshot
    }

    #[must_use]
    pub fn token_for_task(&self, task_id: &TaskId) -> Option<CancellationToken> {
        self.token_reason_for_task(task_id)
            .map(|reason| CancellationToken {
                snapshot: self.snapshot,
                generation: self.generation,
                reason,
            })
    }

    #[must_use]
    pub fn decision_for_checkpoint(
        &self,
        task: &BuildTask,
        graph_index: usize,
        checkpoint: CancellationCheckpoint,
    ) -> CancellationTaskDecision {
        let commit_started = task.kind == TaskKind::ArtifactCommit
            && self.commit_started_tasks.contains(&task.id)
            && self.reason_for_task(&task.id).is_some();
        if commit_started {
            let reason = self
                .reason_for_task(&task.id)
                .expect("commit-started cancellation has a reason");
            return self.task_decision(
                task,
                graph_index,
                Some(reason),
                CancellationDecision::CommitAlreadyStarted,
            );
        }

        let (reason, decision) = match checkpoint {
            CancellationCheckpoint::Pending => decision_from_reason(
                self.reason_for_task(&task.id),
                CancellationDecision::CancelBeforeStart,
            ),
            CancellationCheckpoint::Ready => decision_from_reason(
                self.ready_reason_for_task(&task.id),
                CancellationDecision::CancelBeforeStart,
            ),
            CancellationCheckpoint::RunningSafeCheckpoint => decision_from_reason(
                self.checkpoint_reason_for_task(&task.id),
                CancellationDecision::CancelAtCheckpoint,
            ),
            CancellationCheckpoint::CompletedBeforePublication => decision_from_reason(
                self.publication_reason_for_task(&task.id),
                CancellationDecision::DiscardObsoleteResult,
            ),
            CancellationCheckpoint::CommitStarted => decision_from_reason(
                self.reason_for_task(&task.id),
                CancellationDecision::CommitAlreadyStarted,
            ),
        };

        self.task_decision(task, graph_index, reason, decision)
    }

    #[must_use]
    pub fn decisions_for_graph(&self, graph: &TaskGraph) -> Vec<CancellationTaskDecision> {
        let order = graph_order(graph);
        let mut decisions = graph
            .tasks
            .iter()
            .filter_map(|task| {
                let graph_index = order.get(&task.id).copied().unwrap_or(usize::MAX);
                [
                    CancellationCheckpoint::Pending,
                    CancellationCheckpoint::Ready,
                    CancellationCheckpoint::RunningSafeCheckpoint,
                    CancellationCheckpoint::CompletedBeforePublication,
                ]
                .into_iter()
                .map(|checkpoint| self.decision_for_checkpoint(task, graph_index, checkpoint))
                .find(|decision| decision.decision != CancellationDecision::Continue)
            })
            .collect::<Vec<_>>();
        decisions.sort_by_key(CancellationTaskDecision::sort_key);
        decisions
    }

    fn task_decision(
        &self,
        task: &BuildTask,
        graph_index: usize,
        reason: Option<CancellationReason>,
        decision: CancellationDecision,
    ) -> CancellationTaskDecision {
        CancellationTaskDecision {
            task_id: task.id.clone(),
            snapshot: self.snapshot,
            generation: self.generation,
            reason: if decision == CancellationDecision::Continue {
                None
            } else {
                reason
            },
            decision,
            graph_index,
        }
    }

    fn reason_for_task(&self, task_id: &TaskId) -> Option<CancellationReason> {
        if self.cancel_snapshot {
            return Some(CancellationReason::SupersededSnapshot);
        }
        self.cancelled_tasks
            .contains(task_id)
            .then_some(CancellationReason::ExplicitRequest)
    }

    fn ready_reason_for_task(&self, task_id: &TaskId) -> Option<CancellationReason> {
        self.reason_for_task(task_id).or_else(|| {
            self.ready_cancelled_tasks
                .contains(task_id)
                .then_some(CancellationReason::ExplicitRequest)
        })
    }

    fn checkpoint_reason_for_task(&self, task_id: &TaskId) -> Option<CancellationReason> {
        self.reason_for_task(task_id).or_else(|| {
            self.checkpoint_cancelled_tasks
                .contains(task_id)
                .then_some(CancellationReason::ExplicitRequest)
        })
    }

    fn publication_reason_for_task(&self, task_id: &TaskId) -> Option<CancellationReason> {
        if self.obsolete_snapshot {
            return Some(CancellationReason::SupersededSnapshot);
        }
        self.obsolete_completed_tasks
            .contains(task_id)
            .then_some(CancellationReason::SupersededSnapshot)
    }

    fn token_reason_for_task(&self, task_id: &TaskId) -> Option<CancellationReason> {
        self.reason_for_task(task_id)
            .or_else(|| self.ready_reason_for_task(task_id))
            .or_else(|| self.checkpoint_reason_for_task(task_id))
            .or_else(|| self.publication_reason_for_task(task_id))
    }
}

impl CancellationTaskDecision {
    #[must_use]
    pub fn sort_key(&self) -> (String, usize, String, usize) {
        (
            snapshot_key(self.snapshot),
            self.graph_index,
            self.task_id.as_str().to_owned(),
            cancellation_decision_rank(self.decision),
        )
    }
}

fn push_unique_task(tasks: &mut Vec<TaskId>, task_id: TaskId) -> bool {
    if tasks.iter().any(|candidate| candidate == &task_id) {
        return false;
    }
    tasks.push(task_id);
    true
}

fn push_unique_snapshot(snapshots: &mut Vec<BuildSnapshotId>, snapshot: BuildSnapshotId) -> bool {
    if snapshots.iter().any(|candidate| candidate == &snapshot) {
        return false;
    }
    snapshots.push(snapshot);
    true
}

fn task_set(tasks: &[TaskId]) -> BTreeSet<TaskId> {
    tasks.iter().cloned().collect()
}

fn graph_order(graph: &TaskGraph) -> BTreeMap<TaskId, usize> {
    graph
        .tasks
        .iter()
        .enumerate()
        .map(|(index, task)| (task.id.clone(), index))
        .collect()
}

fn snapshot_key(snapshot: BuildSnapshotId) -> String {
    snapshot
        .to_published_schema_string()
        .expect("build snapshot ids have published-schema strings")
}

fn cancellation_decision_rank(decision: CancellationDecision) -> usize {
    match decision {
        CancellationDecision::Continue => 0,
        CancellationDecision::CancelBeforeStart => 1,
        CancellationDecision::CancelAtCheckpoint => 2,
        CancellationDecision::DiscardObsoleteResult => 3,
        CancellationDecision::CommitAlreadyStarted => 4,
    }
}

fn decision_from_reason(
    reason: Option<CancellationReason>,
    cancellation_decision: CancellationDecision,
) -> (Option<CancellationReason>, CancellationDecision) {
    reason
        .map(|reason| (Some(reason), cancellation_decision))
        .unwrap_or((None, CancellationDecision::Continue))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::task_graph::{
        BuildTask, DependencyCoverage, PipelinePhase, PriorityClass, ResourceClass,
        TaskGraphVersion, WorkUnit,
    };
    use mizar_session::Hash;

    #[test]
    fn generation_advances_monotonically_and_duplicate_requests_are_idempotent() {
        let task = TaskId::new_for_test("source");
        let commit = TaskId::new_for_test("commit");
        let superseded_snapshot = snapshot(1);
        let current = snapshot(2);
        let mut policy = CancellationPolicy::default();

        policy.cancel_task(task.clone());
        assert_eq!(policy.generation().value(), 1);
        policy.cancel_task(task);
        assert_eq!(policy.generation().value(), 1);

        policy.supersede_snapshot(superseded_snapshot);
        assert_eq!(policy.generation().value(), 2);
        policy.supersede_snapshot(superseded_snapshot);
        assert_eq!(policy.generation().value(), 2);

        policy.set_current_snapshot(current);
        assert_eq!(policy.generation().value(), 3);
        policy.set_current_snapshot(current);
        assert_eq!(policy.generation().value(), 3);

        policy.mark_commit_started(commit.clone());
        assert_eq!(policy.generation().value(), 4);
        policy.mark_commit_started(commit);
        assert_eq!(policy.generation().value(), 4);
    }

    #[test]
    fn cancelled_snapshot_tasks_share_same_or_newer_token_generation() {
        let graph = graph_with_tasks(["root", "source", "frontend"]);
        let mut policy = CancellationPolicy::default();
        policy.supersede_snapshot(graph.snapshot);
        let state = CancellationState::from_policy(&policy, &graph);

        for task in &graph.tasks {
            let token = state
                .token_for_task(&task.id)
                .expect("superseded snapshot has cancellation token");
            assert_eq!(token.reason, CancellationReason::SupersededSnapshot);
            assert!(token.generation >= policy.generation());
        }
    }

    #[test]
    fn current_snapshot_mismatch_cancels_pending_and_guards_publication() {
        let graph = graph_with_tasks(["root", "source"]);
        let source = task_by_id(&graph, "source");
        let policy = CancellationPolicy::default().with_current_snapshot(snapshot(99));
        let state = CancellationState::from_policy(&policy, &graph);

        assert!(state.obsolete_snapshot());
        assert_eq!(
            state
                .decision_for_checkpoint(source, 1, CancellationCheckpoint::Pending)
                .decision,
            CancellationDecision::CancelBeforeStart
        );
        assert_eq!(
            state
                .decision_for_checkpoint(source, 1, CancellationCheckpoint::Pending)
                .reason,
            Some(CancellationReason::SupersededSnapshot)
        );
        assert_eq!(
            state
                .decision_for_checkpoint(
                    source,
                    1,
                    CancellationCheckpoint::CompletedBeforePublication
                )
                .decision,
            CancellationDecision::DiscardObsoleteResult
        );
        assert_eq!(
            state
                .token_for_task(&source.id)
                .expect("current snapshot mismatch emits a cancellation token")
                .reason,
            CancellationReason::SupersededSnapshot
        );
    }

    #[test]
    fn decisions_are_sorted_by_snapshot_graph_order_and_task_id() {
        let graph = graph_with_tasks(["root", "b", "a"]);
        let mut policy = CancellationPolicy::default();
        policy.cancel_task(TaskId::new_for_test("a"));
        policy.cancel_task(TaskId::new_for_test("b"));
        let state = CancellationState::from_policy(&policy, &graph);

        let decisions = state.decisions_for_graph(&graph);

        assert_eq!(
            decisions
                .iter()
                .map(|decision| decision.task_id.as_str().to_owned())
                .collect::<Vec<_>>(),
            vec!["b".to_owned(), "a".to_owned()]
        );
        assert!(
            decisions
                .windows(2)
                .all(|pair| pair[0].sort_key() <= pair[1].sort_key())
        );

        let first_snapshot = snapshot(1);
        let second_snapshot = snapshot(2);
        let mut manual_decisions = [
            manual_decision(second_snapshot, "a", 0),
            manual_decision(first_snapshot, "z", 1),
            manual_decision(first_snapshot, "a", 1),
        ];
        manual_decisions.sort_by_key(CancellationTaskDecision::sort_key);
        assert_eq!(
            manual_decisions
                .iter()
                .map(|decision| {
                    (
                        decision.snapshot,
                        decision.graph_index,
                        decision.task_id.as_str().to_owned(),
                    )
                })
                .collect::<Vec<_>>(),
            vec![
                (first_snapshot, 1, "a".to_owned()),
                (first_snapshot, 1, "z".to_owned()),
                (second_snapshot, 0, "a".to_owned()),
            ]
        );
    }

    #[test]
    fn pending_ready_running_and_completed_checkpoints_have_distinct_decisions() {
        let graph = graph_with_tasks(["root", "source", "frontend", "commit"]);
        let source = graph
            .tasks
            .iter()
            .find(|task| task.id.as_str() == "source")
            .expect("source task exists");
        let frontend = graph
            .tasks
            .iter()
            .find(|task| task.id.as_str() == "frontend")
            .expect("frontend task exists");
        let commit = graph
            .tasks
            .iter()
            .find(|task| task.id.as_str() == "commit")
            .expect("commit task exists");
        let mut policy = CancellationPolicy::default();
        policy.cancel_task(source.id.clone());
        policy.cancel_ready_task(frontend.id.clone());
        policy.cancel_task_at_checkpoint(frontend.id.clone());
        policy.discard_obsolete_completed_task(frontend.id.clone());
        policy.cancel_task(commit.id.clone());
        policy.mark_commit_started(commit.id.clone());
        let state = CancellationState::from_policy(&policy, &graph);

        assert_eq!(
            state
                .decision_for_checkpoint(source, 1, CancellationCheckpoint::Pending)
                .reason,
            Some(CancellationReason::ExplicitRequest)
        );
        assert_eq!(
            state
                .decision_for_checkpoint(frontend, 2, CancellationCheckpoint::Ready)
                .reason,
            Some(CancellationReason::ExplicitRequest)
        );
        assert_eq!(
            state
                .decision_for_checkpoint(frontend, 2, CancellationCheckpoint::RunningSafeCheckpoint)
                .reason,
            Some(CancellationReason::ExplicitRequest)
        );
        assert_eq!(
            state
                .decision_for_checkpoint(
                    frontend,
                    2,
                    CancellationCheckpoint::CompletedBeforePublication
                )
                .reason,
            Some(CancellationReason::SupersededSnapshot)
        );
        assert_eq!(
            state
                .decision_for_checkpoint(commit, 3, CancellationCheckpoint::CommitStarted)
                .decision,
            CancellationDecision::CommitAlreadyStarted
        );
        assert_eq!(
            state
                .token_for_task(&frontend.id)
                .expect("checkpoint cancellation emits a token")
                .reason,
            CancellationReason::ExplicitRequest
        );
    }

    #[test]
    fn checkpoint_and_publication_requests_emit_reasons_and_tokens() {
        let graph = graph_with_tasks(["root", "ready", "running", "stale"]);
        let ready = task_by_id(&graph, "ready");
        let running = task_by_id(&graph, "running");
        let stale = task_by_id(&graph, "stale");
        let mut policy = CancellationPolicy::default();
        policy.cancel_ready_task(ready.id.clone());
        policy.cancel_task_at_checkpoint(running.id.clone());
        policy.discard_obsolete_completed_task(stale.id.clone());
        let state = CancellationState::from_policy(&policy, &graph);

        assert_eq!(
            state
                .decision_for_checkpoint(ready, 1, CancellationCheckpoint::Ready)
                .reason,
            Some(CancellationReason::ExplicitRequest)
        );
        assert_eq!(
            state
                .decision_for_checkpoint(running, 2, CancellationCheckpoint::RunningSafeCheckpoint)
                .reason,
            Some(CancellationReason::ExplicitRequest)
        );
        assert_eq!(
            state
                .decision_for_checkpoint(
                    stale,
                    3,
                    CancellationCheckpoint::CompletedBeforePublication
                )
                .reason,
            Some(CancellationReason::SupersededSnapshot)
        );
        assert_eq!(
            state
                .token_for_task(&running.id)
                .expect("running checkpoint cancellation emits a token")
                .reason,
            CancellationReason::ExplicitRequest
        );
        assert_eq!(
            state
                .token_for_task(&stale.id)
                .expect("obsolete completed cancellation emits a token")
                .reason,
            CancellationReason::SupersededSnapshot
        );
    }

    fn graph_with_tasks<const N: usize>(ids: [&str; N]) -> TaskGraph {
        let snapshot = snapshot(7);
        TaskGraph {
            version: TaskGraphVersion::current(),
            snapshot,
            tasks: ids
                .into_iter()
                .map(|id| task(id, snapshot))
                .collect::<Vec<_>>(),
            edges: Vec::new(),
            diagnostics: Vec::new(),
        }
    }

    fn task(id: &str, _snapshot: BuildSnapshotId) -> BuildTask {
        let kind = if id == "root" {
            TaskKind::PackageResolve
        } else if id == "commit" {
            TaskKind::ArtifactCommit
        } else {
            TaskKind::SourceLoad
        };
        BuildTask {
            id: TaskId::new_for_test(id),
            kind,
            unit: WorkUnit::Workspace,
            phases: vec![PipelinePhase::SourceLoad],
            dependencies: Vec::new(),
            dependency_coverage: DependencyCoverage::Complete,
            resource_class: ResourceClass::SourceIo,
            priority_class: PriorityClass::Source,
        }
    }

    fn task_by_id<'a>(graph: &'a TaskGraph, id: &str) -> &'a BuildTask {
        graph
            .tasks
            .iter()
            .find(|task| task.id.as_str() == id)
            .expect("task exists")
    }

    fn manual_decision(
        snapshot: BuildSnapshotId,
        task_id: &str,
        graph_index: usize,
    ) -> CancellationTaskDecision {
        CancellationTaskDecision {
            task_id: TaskId::new_for_test(task_id),
            snapshot,
            generation: CancellationGeneration::new(1),
            reason: Some(CancellationReason::ExplicitRequest),
            decision: CancellationDecision::CancelBeforeStart,
            graph_index,
        }
    }

    fn snapshot(seed: u8) -> BuildSnapshotId {
        let hex = format!("{seed:02x}").repeat(Hash::BYTE_LEN);
        BuildSnapshotId::from_published_schema_str(&format!(
            "mizar-session-build-snapshot-v1:{hex}"
        ))
        .expect("valid snapshot id")
    }
}
