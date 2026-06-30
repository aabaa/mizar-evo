use std::collections::BTreeMap;

use crate::{
    module_index::ModuleId,
    scheduler::SchedulerQueue,
    task_graph::{BuildTask, TaskId, TaskKind, VcTaskDescriptorId, WorkUnit},
};
use mizar_session::PackageId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceBudget {
    pub workspace_workers: usize,
    pub package_workers: usize,
    pub module_workers: usize,
    pub obligation_workers: usize,
    pub source_workers: usize,
    pub proof_workers: usize,
    pub atp_portfolios: usize,
    pub atp_processes: usize,
    pub backend_fanout: usize,
    pub kernel_workers: usize,
    pub io_commits: usize,
    pub documentation_workers: usize,
    pub memory_units: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskResourceRequest {
    pub task_id: TaskId,
    pub queue: SchedulerQueue,
    pub package: Option<PackageId>,
    pub module: Option<ModuleId>,
    pub vc: Option<VcTaskDescriptorId>,
    pub units: ResourceRequestUnits,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ResourceRequestUnits {
    pub worker_units: usize,
    pub memory_units: usize,
    pub external_process_slots: usize,
    pub commit_permits: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
pub enum ResourceAdmissionStatus {
    Admitted,
    Delayed,
    Impossible,
    Released,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceAdmission {
    pub status: ResourceAdmissionStatus,
    pub request: TaskResourceRequest,
    pub blocking_scope: Option<ResourceScope>,
    pub admission_order: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceTelemetry {
    pub task_id: TaskId,
    pub queue: SchedulerQueue,
    pub status: ResourceAdmissionStatus,
    pub requested_units: ResourceRequestUnits,
    pub blocking_scope: Option<ResourceScope>,
    pub admission_order: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ResourceScope {
    WorkspaceWorkers,
    PackageWorkers {
        package_id: PackageId,
    },
    ModuleWorkers {
        module: ModuleId,
    },
    ObligationWorkers {
        module: ModuleId,
        vc: VcTaskDescriptorId,
    },
    SourceWorkers,
    ProofWorkers,
    AtpPortfolios,
    AtpProcesses,
    BackendFanout {
        module: ModuleId,
        vc: VcTaskDescriptorId,
    },
    KernelWorkers,
    IoCommits,
    DocumentationWorkers,
    Memory,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceManager {
    budget: ResourceBudget,
    usage: ResourceUsage,
    reservations: BTreeMap<TaskId, ResourceReservation>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ResourceReservation {
    request: TaskResourceRequest,
    admission_order: usize,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct ResourceUsage {
    workspace_workers: usize,
    source_workers: usize,
    proof_workers: usize,
    atp_portfolios: usize,
    atp_processes: usize,
    kernel_workers: usize,
    io_commits: usize,
    documentation_workers: usize,
    memory_units: usize,
    package_workers: BTreeMap<String, usize>,
    module_workers: BTreeMap<String, usize>,
    obligation_workers: BTreeMap<String, usize>,
    backend_fanout: BTreeMap<String, usize>,
}

impl ResourceBudget {
    #[must_use]
    pub fn unbounded() -> Self {
        let limit = usize::MAX / 4;
        Self {
            workspace_workers: limit,
            package_workers: limit,
            module_workers: limit,
            obligation_workers: limit,
            source_workers: limit,
            proof_workers: limit,
            atp_portfolios: limit,
            atp_processes: limit,
            backend_fanout: limit,
            kernel_workers: limit,
            io_commits: limit,
            documentation_workers: limit,
            memory_units: limit,
        }
    }
}

impl Default for ResourceBudget {
    fn default() -> Self {
        Self::unbounded()
    }
}

impl TaskResourceRequest {
    #[must_use]
    pub fn for_task(task: &BuildTask, queue: SchedulerQueue) -> Self {
        let (package, module, vc) = scopes_for_unit(&task.unit);
        Self {
            task_id: task.id.clone(),
            queue,
            package,
            module,
            vc,
            units: ResourceRequestUnits::for_task(task.kind),
        }
    }
}

impl ResourceRequestUnits {
    #[must_use]
    pub fn for_task(kind: TaskKind) -> Self {
        if kind == TaskKind::PackageResolve {
            return Self::default();
        }

        Self {
            worker_units: 1,
            memory_units: 1,
            external_process_slots: usize::from(kind == TaskKind::BackendRun),
            commit_permits: usize::from(kind == TaskKind::ArtifactCommit),
        }
    }
}

impl ResourceAdmission {
    #[must_use]
    pub fn telemetry(&self) -> ResourceTelemetry {
        ResourceTelemetry {
            task_id: self.request.task_id.clone(),
            queue: self.request.queue,
            status: self.status,
            requested_units: self.request.units,
            blocking_scope: self.blocking_scope.clone(),
            admission_order: self.admission_order,
        }
    }
}

impl ResourceTelemetry {
    #[must_use]
    pub fn sort_key(&self, graph_index: usize) -> (usize, usize, usize, String, String) {
        (
            graph_index,
            self.admission_order,
            resource_status_rank(self.status),
            self.task_id.as_str().to_owned(),
            self.blocking_scope
                .as_ref()
                .map(ResourceScope::stable_label)
                .unwrap_or_default(),
        )
    }
}

impl ResourceScope {
    #[must_use]
    pub fn stable_label(&self) -> String {
        match self {
            Self::WorkspaceWorkers => "workspace-workers".to_owned(),
            Self::PackageWorkers { package_id } => {
                format!("package-workers:{}", package_id.as_str())
            }
            Self::ModuleWorkers { module } => {
                format!(
                    "module-workers:{}:{}",
                    module.package.as_str(),
                    module.path.as_str()
                )
            }
            Self::ObligationWorkers { module, vc } => {
                format!(
                    "obligation-workers:{}:{}:{}",
                    module.package.as_str(),
                    module.path.as_str(),
                    vc.as_str()
                )
            }
            Self::SourceWorkers => "source-workers".to_owned(),
            Self::ProofWorkers => "proof-workers".to_owned(),
            Self::AtpPortfolios => "atp-portfolios".to_owned(),
            Self::AtpProcesses => "atp-processes".to_owned(),
            Self::BackendFanout { module, vc } => {
                format!(
                    "backend-fanout:{}:{}:{}",
                    module.package.as_str(),
                    module.path.as_str(),
                    vc.as_str()
                )
            }
            Self::KernelWorkers => "kernel-workers".to_owned(),
            Self::IoCommits => "io-commits".to_owned(),
            Self::DocumentationWorkers => "documentation-workers".to_owned(),
            Self::Memory => "memory".to_owned(),
        }
    }
}

impl ResourceManager {
    #[must_use]
    pub fn new(budget: ResourceBudget) -> Self {
        Self {
            budget,
            usage: ResourceUsage::default(),
            reservations: BTreeMap::new(),
        }
    }

    #[must_use]
    pub fn try_admit(
        &mut self,
        request: TaskResourceRequest,
        admission_order: usize,
    ) -> ResourceAdmission {
        if let Some(reservation) = self.reservations.get(&request.task_id) {
            return ResourceAdmission {
                status: ResourceAdmissionStatus::Admitted,
                request: reservation.request.clone(),
                blocking_scope: None,
                admission_order: reservation.admission_order,
            };
        }

        let status = self.admission_status(&request);
        let admission = ResourceAdmission {
            status: status.0,
            request: request.clone(),
            blocking_scope: status.1,
            admission_order,
        };
        if admission.status == ResourceAdmissionStatus::Admitted {
            self.reserve(request, admission_order);
        }
        admission
    }

    #[must_use]
    pub fn release(&mut self, task_id: &TaskId) -> Option<(TaskResourceRequest, usize)> {
        let reservation = self.reservations.remove(task_id)?;
        self.release_request(&reservation.request);
        Some((reservation.request, reservation.admission_order))
    }

    #[must_use]
    pub fn active_reservations(&self) -> usize {
        self.reservations.len()
    }

    fn admission_status(
        &self,
        request: &TaskResourceRequest,
    ) -> (ResourceAdmissionStatus, Option<ResourceScope>) {
        if let Some(scope) = self.impossible_scope(request) {
            return (ResourceAdmissionStatus::Impossible, Some(scope));
        }
        if let Some(scope) = self.delayed_scope(request) {
            return (ResourceAdmissionStatus::Delayed, Some(scope));
        }
        (ResourceAdmissionStatus::Admitted, None)
    }

    fn impossible_scope(&self, request: &TaskResourceRequest) -> Option<ResourceScope> {
        self.scope_over_limit(request, false)
    }

    fn delayed_scope(&self, request: &TaskResourceRequest) -> Option<ResourceScope> {
        self.scope_over_limit(request, true)
    }

    fn scope_over_limit(
        &self,
        request: &TaskResourceRequest,
        include_usage: bool,
    ) -> Option<ResourceScope> {
        let units = request.units;
        self.check_limit(
            units.worker_units,
            self.usage.workspace_workers,
            self.budget.workspace_workers,
            include_usage,
            ResourceScope::WorkspaceWorkers,
        )
        .or_else(|| {
            request.package.as_ref().and_then(|package_id| {
                self.check_limit(
                    units.worker_units,
                    usage_for(&self.usage.package_workers, package_key(package_id)),
                    self.budget.package_workers,
                    include_usage,
                    ResourceScope::PackageWorkers {
                        package_id: package_id.clone(),
                    },
                )
            })
        })
        .or_else(|| {
            request.module.as_ref().and_then(|module| {
                self.check_limit(
                    units.worker_units,
                    usage_for(&self.usage.module_workers, module_key(module)),
                    self.budget.module_workers,
                    include_usage,
                    ResourceScope::ModuleWorkers {
                        module: module.clone(),
                    },
                )
            })
        })
        .or_else(|| {
            request.module.as_ref().and_then(|module| {
                request.vc.as_ref().and_then(|vc| {
                    self.check_limit(
                        units.worker_units,
                        usage_for(&self.usage.obligation_workers, obligation_key(module, vc)),
                        self.budget.obligation_workers,
                        include_usage,
                        ResourceScope::ObligationWorkers {
                            module: module.clone(),
                            vc: vc.clone(),
                        },
                    )
                })
            })
        })
        .or_else(|| {
            self.check_limit(
                units.memory_units,
                self.usage.memory_units,
                self.budget.memory_units,
                include_usage,
                ResourceScope::Memory,
            )
        })
        .or_else(|| self.pool_scope_over_limit(request, include_usage))
    }

    fn pool_scope_over_limit(
        &self,
        request: &TaskResourceRequest,
        include_usage: bool,
    ) -> Option<ResourceScope> {
        let units = request.units;
        match request.queue {
            SchedulerQueue::Coordinator => None,
            SchedulerQueue::SourceLocalCpu => self.check_limit(
                units.worker_units,
                self.usage.source_workers,
                self.budget.source_workers,
                include_usage,
                ResourceScope::SourceWorkers,
            ),
            SchedulerQueue::DeterministicProof => self.check_limit(
                units.worker_units,
                self.usage.proof_workers,
                self.budget.proof_workers,
                include_usage,
                ResourceScope::ProofWorkers,
            ),
            SchedulerQueue::AtpPortfolio => self.check_limit(
                units.worker_units,
                self.usage.atp_portfolios,
                self.budget.atp_portfolios,
                include_usage,
                ResourceScope::AtpPortfolios,
            ),
            SchedulerQueue::AtpProcess => self
                .check_limit(
                    units.external_process_slots,
                    self.usage.atp_processes,
                    self.budget.atp_processes,
                    include_usage,
                    ResourceScope::AtpProcesses,
                )
                .or_else(|| {
                    request.module.as_ref().and_then(|module| {
                        request.vc.as_ref().and_then(|vc| {
                            self.check_limit(
                                units.external_process_slots,
                                usage_for(&self.usage.backend_fanout, obligation_key(module, vc)),
                                self.budget.backend_fanout,
                                include_usage,
                                ResourceScope::BackendFanout {
                                    module: module.clone(),
                                    vc: vc.clone(),
                                },
                            )
                        })
                    })
                }),
            SchedulerQueue::Kernel => self.check_limit(
                units.worker_units,
                self.usage.kernel_workers,
                self.budget.kernel_workers,
                include_usage,
                ResourceScope::KernelWorkers,
            ),
            SchedulerQueue::IoCommit => self.check_limit(
                units.commit_permits,
                self.usage.io_commits,
                self.budget.io_commits,
                include_usage,
                ResourceScope::IoCommits,
            ),
            SchedulerQueue::Documentation => self.check_limit(
                units.worker_units,
                self.usage.documentation_workers,
                self.budget.documentation_workers,
                include_usage,
                ResourceScope::DocumentationWorkers,
            ),
        }
    }

    fn check_limit(
        &self,
        amount: usize,
        used: usize,
        limit: usize,
        include_usage: bool,
        scope: ResourceScope,
    ) -> Option<ResourceScope> {
        if amount == 0 {
            return None;
        }
        let requested = if include_usage {
            used.saturating_add(amount)
        } else {
            amount
        };
        (requested > limit).then_some(scope)
    }

    fn reserve(&mut self, request: TaskResourceRequest, admission_order: usize) {
        self.apply_request(&request);
        self.reservations.insert(
            request.task_id.clone(),
            ResourceReservation {
                request,
                admission_order,
            },
        );
    }

    fn apply_request(&mut self, request: &TaskResourceRequest) {
        let units = request.units;
        self.usage.workspace_workers += units.worker_units;
        self.usage.memory_units += units.memory_units;
        if let Some(package_id) = &request.package {
            add_usage(
                &mut self.usage.package_workers,
                package_key(package_id),
                units.worker_units,
            );
        }
        if let Some(module) = &request.module {
            add_usage(
                &mut self.usage.module_workers,
                module_key(module),
                units.worker_units,
            );
            if let Some(vc) = &request.vc {
                add_usage(
                    &mut self.usage.obligation_workers,
                    obligation_key(module, vc),
                    units.worker_units,
                );
            }
        }
        match request.queue {
            SchedulerQueue::Coordinator => {}
            SchedulerQueue::SourceLocalCpu => self.usage.source_workers += units.worker_units,
            SchedulerQueue::DeterministicProof => self.usage.proof_workers += units.worker_units,
            SchedulerQueue::AtpPortfolio => self.usage.atp_portfolios += units.worker_units,
            SchedulerQueue::AtpProcess => {
                self.usage.atp_processes += units.external_process_slots;
                if let (Some(module), Some(vc)) = (&request.module, &request.vc) {
                    add_usage(
                        &mut self.usage.backend_fanout,
                        obligation_key(module, vc),
                        units.external_process_slots,
                    );
                }
            }
            SchedulerQueue::Kernel => self.usage.kernel_workers += units.worker_units,
            SchedulerQueue::IoCommit => self.usage.io_commits += units.commit_permits,
            SchedulerQueue::Documentation => {
                self.usage.documentation_workers += units.worker_units;
            }
        }
    }

    fn release_request(&mut self, request: &TaskResourceRequest) {
        let units = request.units;
        self.usage.workspace_workers -= units.worker_units;
        self.usage.memory_units -= units.memory_units;
        if let Some(package_id) = &request.package {
            subtract_usage(
                &mut self.usage.package_workers,
                package_key(package_id),
                units.worker_units,
            );
        }
        if let Some(module) = &request.module {
            subtract_usage(
                &mut self.usage.module_workers,
                module_key(module),
                units.worker_units,
            );
            if let Some(vc) = &request.vc {
                subtract_usage(
                    &mut self.usage.obligation_workers,
                    obligation_key(module, vc),
                    units.worker_units,
                );
            }
        }
        match request.queue {
            SchedulerQueue::Coordinator => {}
            SchedulerQueue::SourceLocalCpu => self.usage.source_workers -= units.worker_units,
            SchedulerQueue::DeterministicProof => self.usage.proof_workers -= units.worker_units,
            SchedulerQueue::AtpPortfolio => self.usage.atp_portfolios -= units.worker_units,
            SchedulerQueue::AtpProcess => {
                self.usage.atp_processes -= units.external_process_slots;
                if let (Some(module), Some(vc)) = (&request.module, &request.vc) {
                    subtract_usage(
                        &mut self.usage.backend_fanout,
                        obligation_key(module, vc),
                        units.external_process_slots,
                    );
                }
            }
            SchedulerQueue::Kernel => self.usage.kernel_workers -= units.worker_units,
            SchedulerQueue::IoCommit => self.usage.io_commits -= units.commit_permits,
            SchedulerQueue::Documentation => {
                self.usage.documentation_workers -= units.worker_units;
            }
        }
    }
}

#[must_use]
pub fn resource_queue_rank(queue: SchedulerQueue) -> usize {
    match queue {
        SchedulerQueue::Coordinator => 0,
        SchedulerQueue::SourceLocalCpu => 1,
        SchedulerQueue::DeterministicProof => 2,
        SchedulerQueue::AtpPortfolio => 3,
        SchedulerQueue::AtpProcess => 4,
        SchedulerQueue::Kernel => 5,
        SchedulerQueue::IoCommit => 6,
        SchedulerQueue::Documentation => 7,
    }
}

fn scopes_for_unit(
    unit: &WorkUnit,
) -> (
    Option<PackageId>,
    Option<ModuleId>,
    Option<VcTaskDescriptorId>,
) {
    match unit {
        WorkUnit::Workspace => (None, None, None),
        WorkUnit::Package { package_id } => (Some(package_id.clone()), None, None),
        WorkUnit::Module { module } => (Some(module.package.clone()), Some(module.clone()), None),
        WorkUnit::Vc { module, descriptor } => (
            Some(module.package.clone()),
            Some(module.clone()),
            Some(descriptor.clone()),
        ),
        WorkUnit::BackendAttempt {
            module, descriptor, ..
        }
        | WorkUnit::EvidenceCandidate {
            module, descriptor, ..
        } => (
            Some(module.package.clone()),
            Some(module.clone()),
            Some(descriptor.clone()),
        ),
    }
}

fn resource_status_rank(status: ResourceAdmissionStatus) -> usize {
    match status {
        ResourceAdmissionStatus::Admitted => 0,
        ResourceAdmissionStatus::Delayed => 1,
        ResourceAdmissionStatus::Impossible => 2,
        ResourceAdmissionStatus::Released => 3,
    }
}

fn package_key(package_id: &PackageId) -> String {
    package_id.as_str().to_owned()
}

fn module_key(module: &ModuleId) -> String {
    format!("{}:{}", module.package.as_str(), module.path.as_str())
}

fn obligation_key(module: &ModuleId, vc: &VcTaskDescriptorId) -> String {
    format!(
        "{}:{}:{}",
        module.package.as_str(),
        module.path.as_str(),
        vc.as_str()
    )
}

fn usage_for(usage: &BTreeMap<String, usize>, key: String) -> usize {
    usage.get(&key).copied().unwrap_or_default()
}

fn add_usage(usage: &mut BTreeMap<String, usize>, key: String, amount: usize) {
    if amount > 0 {
        *usage.entry(key).or_default() += amount;
    }
}

fn subtract_usage(usage: &mut BTreeMap<String, usize>, key: String, amount: usize) {
    if amount == 0 {
        return;
    }
    let current = usage
        .get_mut(&key)
        .expect("released resource scope was reserved");
    *current -= amount;
    if *current == 0 {
        usage.remove(&key);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        module_index::ModuleId,
        scheduler::SchedulerQueue,
        task_graph::{
            BackendProfileId, BuildTask, DependencyCoverage, PipelinePhase, PriorityClass,
            ResourceClass, TaskKind, VcTaskDescriptorId, WorkUnit,
        },
    };
    use mizar_session::{ModulePath, PackageId};

    #[test]
    fn admission_delays_temporary_exhaustion_without_overcommit() {
        let source = task(TaskKind::SourceLoad, WorkUnit::Module { module: module() });
        let first_request = TaskResourceRequest::for_task(&source, SchedulerQueue::SourceLocalCpu);
        let mut second_request = first_request.clone();
        second_request.task_id = crate::task_graph::TaskId::new_for_test("second-source");
        let mut budget = ResourceBudget::unbounded();
        budget.source_workers = 1;
        let mut manager = ResourceManager::new(budget);

        let first = manager.try_admit(first_request, 0);
        let second = manager.try_admit(second_request, 1);

        assert_eq!(first.status, ResourceAdmissionStatus::Admitted);
        assert_eq!(second.status, ResourceAdmissionStatus::Delayed);
        assert_eq!(second.blocking_scope, Some(ResourceScope::SourceWorkers));
        assert_eq!(manager.active_reservations(), 1);
    }

    #[test]
    fn impossible_request_reports_stable_scope() {
        let source = task(TaskKind::SourceLoad, WorkUnit::Module { module: module() });
        let request = TaskResourceRequest::for_task(&source, SchedulerQueue::SourceLocalCpu);
        let mut budget = ResourceBudget::unbounded();
        budget.source_workers = 0;
        let mut manager = ResourceManager::new(budget);

        let admission = manager.try_admit(request, 0);

        assert_eq!(admission.status, ResourceAdmissionStatus::Impossible);
        assert_eq!(admission.blocking_scope, Some(ResourceScope::SourceWorkers));
        assert_eq!(manager.active_reservations(), 0);
    }

    #[test]
    fn hierarchical_scopes_are_isolated_budget_limits() {
        assert_isolated_scope_limit(
            |budget| budget.workspace_workers = 1,
            ResourceScope::WorkspaceWorkers,
        );
        assert_isolated_scope_limit(
            |budget| budget.package_workers = 1,
            ResourceScope::PackageWorkers {
                package_id: PackageId::new("app"),
            },
        );
        assert_isolated_scope_limit(
            |budget| budget.module_workers = 1,
            ResourceScope::ModuleWorkers { module: module() },
        );
        assert_isolated_scope_limit(
            |budget| budget.obligation_workers = 1,
            ResourceScope::ObligationWorkers {
                module: module(),
                vc: VcTaskDescriptorId::new("vc-main"),
            },
        );
        assert_isolated_scope_limit(|budget| budget.io_commits = 1, ResourceScope::IoCommits);
        assert_isolated_backend_fanout_limit();
    }

    #[test]
    fn modeled_pool_and_memory_limits_are_isolated() {
        assert_isolated_limit(
            TaskKind::VcDischarge,
            WorkUnit::Vc {
                module: module(),
                descriptor: VcTaskDescriptorId::new("vc-main"),
            },
            SchedulerQueue::DeterministicProof,
            |budget| budget.proof_workers = 1,
            ResourceScope::ProofWorkers,
        );
        assert_isolated_limit(
            TaskKind::AtpSolve,
            WorkUnit::Vc {
                module: module(),
                descriptor: VcTaskDescriptorId::new("vc-main"),
            },
            SchedulerQueue::AtpPortfolio,
            |budget| budget.atp_portfolios = 1,
            ResourceScope::AtpPortfolios,
        );
        assert_isolated_limit(
            TaskKind::KernelCheck,
            WorkUnit::EvidenceCandidate {
                module: module(),
                descriptor: VcTaskDescriptorId::new("vc-main"),
                evidence_candidate: crate::task_graph::EvidenceCandidateId::new("kernel-a"),
            },
            SchedulerQueue::Kernel,
            |budget| budget.kernel_workers = 1,
            ResourceScope::KernelWorkers,
        );
        assert_isolated_limit(
            TaskKind::DocumentationExtract,
            WorkUnit::Module { module: module() },
            SchedulerQueue::Documentation,
            |budget| budget.documentation_workers = 1,
            ResourceScope::DocumentationWorkers,
        );
        assert_isolated_limit(
            TaskKind::SourceLoad,
            WorkUnit::Module { module: module() },
            SchedulerQueue::SourceLocalCpu,
            |budget| budget.memory_units = 1,
            ResourceScope::Memory,
        );
    }

    #[test]
    fn impossible_memory_request_reports_memory_scope() {
        let source = task(TaskKind::SourceLoad, WorkUnit::Module { module: module() });
        let request = TaskResourceRequest::for_task(&source, SchedulerQueue::SourceLocalCpu);
        let mut budget = ResourceBudget::unbounded();
        budget.memory_units = 0;
        let mut manager = ResourceManager::new(budget);

        let admission = manager.try_admit(request, 0);

        assert_eq!(admission.status, ResourceAdmissionStatus::Impossible);
        assert_eq!(admission.blocking_scope, Some(ResourceScope::Memory));
        assert_eq!(manager.active_reservations(), 0);
    }

    #[test]
    fn atp_portfolio_does_not_consume_backend_process_slot() {
        let mut budget = ResourceBudget::unbounded();
        budget.atp_portfolios = 1;
        budget.atp_processes = 0;
        let mut manager = ResourceManager::new(budget);
        let atp = task(
            TaskKind::AtpSolve,
            WorkUnit::Vc {
                module: module(),
                descriptor: VcTaskDescriptorId::new("vc-main"),
            },
        );
        let backend = task(
            TaskKind::BackendRun,
            WorkUnit::BackendAttempt {
                module: module(),
                descriptor: VcTaskDescriptorId::new("vc-main"),
                backend_profile: BackendProfileId::new("vampire"),
            },
        );

        let portfolio = manager.try_admit(
            TaskResourceRequest::for_task(&atp, SchedulerQueue::AtpPortfolio),
            0,
        );
        let process = manager.try_admit(
            TaskResourceRequest::for_task(&backend, SchedulerQueue::AtpProcess),
            1,
        );

        assert_eq!(portfolio.status, ResourceAdmissionStatus::Admitted);
        assert_eq!(process.status, ResourceAdmissionStatus::Impossible);
        assert_eq!(process.blocking_scope, Some(ResourceScope::AtpProcesses));
    }

    #[test]
    fn release_happens_once_per_reservation() {
        let source = task(TaskKind::SourceLoad, WorkUnit::Module { module: module() });
        let request = TaskResourceRequest::for_task(&source, SchedulerQueue::SourceLocalCpu);
        let task_id = request.task_id.clone();
        let mut manager = ResourceManager::new(ResourceBudget::unbounded());

        assert_eq!(
            manager.try_admit(request, 0).status,
            ResourceAdmissionStatus::Admitted
        );
        assert!(manager.release(&task_id).is_some());
        assert!(manager.release(&task_id).is_none());
        assert_eq!(manager.active_reservations(), 0);
    }

    #[test]
    fn duplicate_admission_is_idempotent_and_does_not_leak_usage() {
        let source = task(TaskKind::SourceLoad, WorkUnit::Module { module: module() });
        let request = TaskResourceRequest::for_task(&source, SchedulerQueue::SourceLocalCpu);
        let task_id = request.task_id.clone();
        let mut budget = ResourceBudget::unbounded();
        budget.source_workers = 1;
        let mut manager = ResourceManager::new(budget);

        let first = manager.try_admit(request.clone(), 7);
        let duplicate = manager.try_admit(request.clone(), 8);

        assert_eq!(first.status, ResourceAdmissionStatus::Admitted);
        assert_eq!(duplicate.status, ResourceAdmissionStatus::Admitted);
        assert_eq!(duplicate.admission_order, first.admission_order);
        assert_eq!(manager.active_reservations(), 1);
        assert!(manager.release(&task_id).is_some());
        let after_release = manager.try_admit(request, 9);
        assert_eq!(after_release.status, ResourceAdmissionStatus::Admitted);
    }

    fn assert_isolated_scope_limit(
        configure: impl FnOnce(&mut ResourceBudget),
        expected_scope: ResourceScope,
    ) {
        let artifact = task(
            TaskKind::ArtifactCommit,
            WorkUnit::Vc {
                module: module(),
                descriptor: VcTaskDescriptorId::new("vc-main"),
            },
        );
        let mut first = TaskResourceRequest::for_task(&artifact, SchedulerQueue::IoCommit);
        first.task_id = crate::task_graph::TaskId::new_for_test("first");
        let mut second = first.clone();
        second.task_id = crate::task_graph::TaskId::new_for_test("second");
        let mut budget = ResourceBudget::unbounded();
        configure(&mut budget);
        let mut manager = ResourceManager::new(budget);

        assert_eq!(
            manager.try_admit(first, 0).status,
            ResourceAdmissionStatus::Admitted
        );
        let delayed = manager.try_admit(second, 1);

        assert_eq!(delayed.status, ResourceAdmissionStatus::Delayed);
        assert_eq!(delayed.blocking_scope, Some(expected_scope));
        assert_eq!(manager.active_reservations(), 1);
    }

    fn assert_isolated_limit(
        kind: TaskKind,
        unit: WorkUnit,
        queue: SchedulerQueue,
        configure: impl FnOnce(&mut ResourceBudget),
        expected_scope: ResourceScope,
    ) {
        let task = task(kind, unit);
        let mut first = TaskResourceRequest::for_task(&task, queue);
        first.task_id = crate::task_graph::TaskId::new_for_test("first");
        let mut second = first.clone();
        second.task_id = crate::task_graph::TaskId::new_for_test("second");
        let mut budget = ResourceBudget::unbounded();
        configure(&mut budget);
        let mut manager = ResourceManager::new(budget);

        assert_eq!(
            manager.try_admit(first, 0).status,
            ResourceAdmissionStatus::Admitted
        );
        let delayed = manager.try_admit(second, 1);

        assert_eq!(delayed.status, ResourceAdmissionStatus::Delayed);
        assert_eq!(delayed.blocking_scope, Some(expected_scope));
        assert_eq!(manager.active_reservations(), 1);
    }

    fn assert_isolated_backend_fanout_limit() {
        let backend = task(
            TaskKind::BackendRun,
            WorkUnit::BackendAttempt {
                module: module(),
                descriptor: VcTaskDescriptorId::new("vc-main"),
                backend_profile: BackendProfileId::new("vampire"),
            },
        );
        let mut first = TaskResourceRequest::for_task(&backend, SchedulerQueue::AtpProcess);
        first.task_id = crate::task_graph::TaskId::new_for_test("first-backend");
        let mut second = first.clone();
        second.task_id = crate::task_graph::TaskId::new_for_test("second-backend");
        let mut budget = ResourceBudget::unbounded();
        budget.backend_fanout = 1;
        let mut manager = ResourceManager::new(budget);

        assert_eq!(
            manager.try_admit(first, 0).status,
            ResourceAdmissionStatus::Admitted
        );
        let delayed = manager.try_admit(second, 1);

        assert_eq!(delayed.status, ResourceAdmissionStatus::Delayed);
        assert_eq!(
            delayed.blocking_scope,
            Some(ResourceScope::BackendFanout {
                module: module(),
                vc: VcTaskDescriptorId::new("vc-main"),
            })
        );
    }

    fn task(kind: TaskKind, unit: WorkUnit) -> BuildTask {
        BuildTask {
            id: crate::task_graph::TaskId::new_for_test(format!("{kind:?}")),
            kind,
            unit,
            phases: vec![PipelinePhase::SourceLoad],
            dependencies: Vec::new(),
            dependency_coverage: DependencyCoverage::Complete,
            resource_class: match kind {
                TaskKind::AtpSolve | TaskKind::BackendRun => ResourceClass::AtpProcess,
                TaskKind::KernelCheck => ResourceClass::Kernel,
                TaskKind::ArtifactCommit => ResourceClass::ArtifactIo,
                TaskKind::DocumentationExtract => ResourceClass::Documentation,
                _ => ResourceClass::CpuLocal,
            },
            priority_class: PriorityClass::Source,
        }
    }

    fn module() -> ModuleId {
        ModuleId::new(PackageId::new("app"), ModulePath::new("main"))
    }
}
