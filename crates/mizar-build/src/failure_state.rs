use crate::{
    scheduler::{SchedulerDiagnosticRef, SchedulerOrderKey},
    task_graph::{BuildTask, PipelinePhase, TaskId, TaskKind},
};
use mizar_session::BuildSnapshotId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
pub enum FailureCategory {
    ParseError,
    ResolveError,
    TypeError,
    OverloadAmbiguity,
    ClusterLoop,
    AtpTimeout,
    CertificateRejection,
    KernelRejection,
    BuildInfrastructure,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
pub enum BlockReason {
    DependencyFailed,
    DependencyBlocked,
    DependencyCancelled,
    MissingDependencyCoverage,
    ImpossibleResourceRequest,
    PhaseDispatchBlocked,
    NoSchedulablePath,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct FailureSourceOrder {
    pub package_id: Option<String>,
    pub module_path: Option<String>,
    pub source_range: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildFailureRecord {
    pub task_id: TaskId,
    pub snapshot: BuildSnapshotId,
    pub category: FailureCategory,
    pub phase: PipelinePhase,
    pub source_order: Option<FailureSourceOrder>,
    pub severity_rank: usize,
    pub canonical_order: SchedulerOrderKey,
    pub diagnostic_code: String,
    pub stable_detail_key: String,
    pub rejection_reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockedTaskRecord {
    pub task_id: TaskId,
    pub snapshot: BuildSnapshotId,
    pub blocked_by: Vec<TaskId>,
    pub reason: BlockReason,
    pub canonical_order: SchedulerOrderKey,
}

impl FailureCategory {
    #[must_use]
    pub const fn for_synthetic_task(kind: TaskKind) -> Self {
        match kind {
            TaskKind::PackageResolve
            | TaskKind::ArtifactCommit
            | TaskKind::DocumentationExtract => Self::BuildInfrastructure,
            TaskKind::SourceLoad | TaskKind::Frontend => Self::ParseError,
            TaskKind::ModuleResolve => Self::ResolveError,
            TaskKind::CheckAndElaborate | TaskKind::VcGenerate | TaskKind::VcDischarge => {
                Self::TypeError
            }
            TaskKind::AtpSolve | TaskKind::BackendRun => Self::AtpTimeout,
            TaskKind::KernelCheck => Self::KernelRejection,
        }
    }
}

impl BuildFailureRecord {
    #[must_use]
    pub fn from_failed_task(
        task: &BuildTask,
        snapshot: BuildSnapshotId,
        canonical_order: SchedulerOrderKey,
        diagnostics: &[SchedulerDiagnosticRef],
    ) -> Vec<Self> {
        if diagnostics.is_empty() {
            return vec![Self {
                task_id: task.id.clone(),
                snapshot,
                category: FailureCategory::for_synthetic_task(task.kind),
                phase: primary_phase(task),
                source_order: None,
                severity_rank: 0,
                canonical_order,
                diagnostic_code: "synthetic_failure".to_owned(),
                stable_detail_key: task.id.as_str().to_owned(),
                rejection_reason: None,
            }];
        }

        diagnostics
            .iter()
            .map(|diagnostic| Self {
                task_id: task.id.clone(),
                snapshot,
                category: FailureCategory::for_synthetic_task(task.kind),
                phase: primary_phase(task),
                source_order: Some(FailureSourceOrder {
                    package_id: None,
                    module_path: Some(diagnostic.source.clone()),
                    source_range: None,
                }),
                severity_rank: 0,
                canonical_order,
                diagnostic_code: diagnostic.code.clone(),
                stable_detail_key: stable_detail_key(task, &diagnostic.code),
                rejection_reason: None,
            })
            .collect()
    }

    #[must_use]
    pub fn sort_key(
        &self,
    ) -> (
        String,
        String,
        String,
        String,
        PipelinePhase,
        usize,
        String,
        String,
        SchedulerOrderKey,
        String,
        String,
    ) {
        let source_order = self.source_order.as_ref();
        (
            snapshot_key(self.snapshot),
            source_order
                .and_then(|order| order.package_id.clone())
                .unwrap_or_default(),
            source_order
                .and_then(|order| order.module_path.clone())
                .unwrap_or_default(),
            source_order
                .and_then(|order| order.source_range.clone())
                .unwrap_or_default(),
            self.phase,
            self.severity_rank,
            self.diagnostic_code.clone(),
            self.stable_detail_key.clone(),
            self.canonical_order,
            self.task_id.as_str().to_owned(),
            self.rejection_reason.clone().unwrap_or_default(),
        )
    }
}

impl BlockedTaskRecord {
    #[must_use]
    pub fn new(
        task_id: TaskId,
        snapshot: BuildSnapshotId,
        mut blocked_by: Vec<TaskId>,
        reason: BlockReason,
        canonical_order: SchedulerOrderKey,
    ) -> Self {
        blocked_by.sort();
        blocked_by.dedup();
        Self {
            task_id,
            snapshot,
            blocked_by,
            reason,
            canonical_order,
        }
    }

    #[must_use]
    pub fn sort_key(&self) -> (String, SchedulerOrderKey, String, BlockReason, Vec<String>) {
        (
            snapshot_key(self.snapshot),
            self.canonical_order,
            self.task_id.as_str().to_owned(),
            self.reason,
            self.blocked_by
                .iter()
                .map(|task_id| task_id.as_str().to_owned())
                .collect(),
        )
    }
}

fn primary_phase(task: &BuildTask) -> PipelinePhase {
    task.phases
        .first()
        .copied()
        .unwrap_or_else(|| phase_for_task_kind(task.kind))
}

fn phase_for_task_kind(kind: TaskKind) -> PipelinePhase {
    match kind {
        TaskKind::PackageResolve => PipelinePhase::PackageResolve,
        TaskKind::SourceLoad => PipelinePhase::SourceLoad,
        TaskKind::Frontend => PipelinePhase::Frontend,
        TaskKind::ModuleResolve => PipelinePhase::ModuleResolve,
        TaskKind::CheckAndElaborate => PipelinePhase::TypeChecking,
        TaskKind::VcGenerate => PipelinePhase::VcGenerate,
        TaskKind::VcDischarge => PipelinePhase::VcDischarge,
        TaskKind::AtpSolve => PipelinePhase::AtpSolve,
        TaskKind::BackendRun => PipelinePhase::BackendRun,
        TaskKind::KernelCheck => PipelinePhase::KernelCheck,
        TaskKind::ArtifactCommit => PipelinePhase::ArtifactCommit,
        TaskKind::DocumentationExtract => PipelinePhase::DocumentationExtract,
    }
}

fn stable_detail_key(task: &BuildTask, diagnostic_code: &str) -> String {
    format!("{}:{diagnostic_code}", task.id.as_str())
}

fn snapshot_key(snapshot: BuildSnapshotId) -> String {
    snapshot
        .to_published_schema_string()
        .expect("build snapshot ids have published-schema strings")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::task_graph::{DependencyCoverage, PriorityClass, ResourceClass, TaskId, WorkUnit};
    use mizar_session::Hash;

    #[test]
    fn failed_task_records_preserve_diagnostic_order_inputs_and_scheduler_fallback() {
        let task = task(
            "frontend",
            TaskKind::Frontend,
            vec![PipelinePhase::Frontend],
        );
        let records = BuildFailureRecord::from_failed_task(
            &task,
            snapshot(1),
            order(3),
            &[
                SchedulerDiagnosticRef::new("main", "E002", "later"),
                SchedulerDiagnosticRef::new("main", "E001", "earlier"),
            ],
        );

        assert_eq!(records.len(), 2);
        assert_eq!(records[0].category, FailureCategory::ParseError);
        assert_eq!(records[0].phase, PipelinePhase::Frontend);
        assert_eq!(records[0].canonical_order, order(3));
        assert_eq!(records[0].stable_detail_key, "frontend:E002");
        assert!(!records[0].stable_detail_key.contains("later"));
        assert_eq!(
            records[0]
                .source_order
                .as_ref()
                .and_then(|source| source.module_path.as_deref()),
            Some("main")
        );

        let fallback = BuildFailureRecord::from_failed_task(&task, snapshot(1), order(3), &[]);
        assert_eq!(fallback[0].diagnostic_code, "synthetic_failure");
        assert!(fallback[0].source_order.is_none());
    }

    #[test]
    fn failure_records_sort_without_completion_order_inputs() {
        let task = task(
            "frontend",
            TaskKind::Frontend,
            vec![PipelinePhase::Frontend],
        );
        let mut records = BuildFailureRecord::from_failed_task(
            &task,
            snapshot(1),
            order(2),
            &[
                SchedulerDiagnosticRef::new("util", "E010", "util"),
                SchedulerDiagnosticRef::new("main", "E001", "main"),
            ],
        );

        records.sort_by_key(BuildFailureRecord::sort_key);

        assert_eq!(
            records
                .iter()
                .map(|record| record.diagnostic_code.as_str())
                .collect::<Vec<_>>(),
            vec!["E001", "E010"]
        );
    }

    #[test]
    fn blocked_records_sort_dependencies_and_preserve_direct_block_reasons() {
        let a = TaskId::new_for_test("a");
        let z = TaskId::new_for_test("z");
        let record = BlockedTaskRecord::new(
            TaskId::new_for_test("blocked"),
            snapshot(2),
            vec![z.clone(), a.clone(), z],
            BlockReason::DependencyFailed,
            order(5),
        );

        assert_eq!(record.blocked_by, vec![a, TaskId::new_for_test("z")]);

        let direct = BlockedTaskRecord::new(
            TaskId::new_for_test("direct"),
            snapshot(2),
            Vec::new(),
            BlockReason::ImpossibleResourceRequest,
            order(6),
        );
        assert!(direct.blocked_by.is_empty());
        assert_eq!(direct.reason, BlockReason::ImpossibleResourceRequest);
    }

    fn task(id: &str, kind: TaskKind, phases: Vec<PipelinePhase>) -> BuildTask {
        BuildTask {
            id: TaskId::new_for_test(id),
            kind,
            unit: WorkUnit::Workspace,
            phases,
            dependencies: Vec::new(),
            dependency_coverage: DependencyCoverage::Complete,
            resource_class: ResourceClass::Coordinator,
            priority_class: PriorityClass::Root,
        }
    }

    const fn order(graph_index: usize) -> SchedulerOrderKey {
        SchedulerOrderKey {
            graph_index,
            lifecycle_rank: 4,
        }
    }

    fn snapshot(seed: u8) -> BuildSnapshotId {
        let hex = format!("{seed:02x}").repeat(Hash::BYTE_LEN);
        BuildSnapshotId::from_published_schema_str(&format!(
            "mizar-session-build-snapshot-v1:{hex}"
        ))
        .expect("snapshot id is valid")
    }
}
