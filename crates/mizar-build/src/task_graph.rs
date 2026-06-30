use std::collections::{BTreeMap, BTreeSet, VecDeque};

use crate::{
    module_index::{ModuleId, ModuleIndex, ModuleIndexEntry, ModuleIndexLocation},
    planner::{BuildPlan, DependencyEdge},
};
use mizar_session::{BuildSnapshotId, PackageId};

pub const TASK_GRAPH_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TaskGraphVersion(u32);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskGraphInput {
    pub graph_version: TaskGraphVersion,
    pub snapshot: BuildSnapshotId,
    pub build_plan: BuildPlan,
    pub module_index: ModuleIndex,
    pub dependency_overlay: ModuleDependencyOverlay,
    pub vc_descriptors: Vec<VcTaskDescriptor>,
    pub profile: TaskGraphProfile,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskGraph {
    pub version: TaskGraphVersion,
    pub snapshot: BuildSnapshotId,
    pub tasks: Vec<BuildTask>,
    pub edges: Vec<TaskEdge>,
    pub diagnostics: Vec<TaskGraphDiagnostic>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildTask {
    pub id: TaskId,
    pub kind: TaskKind,
    pub unit: WorkUnit,
    pub phases: Vec<PipelinePhase>,
    pub dependencies: Vec<TaskId>,
    pub dependency_coverage: DependencyCoverage,
    pub resource_class: ResourceClass,
    pub priority_class: PriorityClass,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskEdge {
    pub dependent: TaskId,
    pub dependency: TaskId,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TaskId(String);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TaskKind {
    PackageResolve,
    SourceLoad,
    Frontend,
    ModuleResolve,
    CheckAndElaborate,
    VcGenerate,
    VcDischarge,
    AtpSolve,
    BackendRun,
    KernelCheck,
    ArtifactCommit,
    DocumentationExtract,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PipelinePhase {
    PackageResolve,
    SourceLoad,
    Frontend,
    ModuleResolve,
    SignatureCollection,
    TypeChecking,
    RegistrationResolution,
    OverloadResolution,
    Elaboration,
    AlgorithmPreparation,
    VcGenerate,
    VcDischarge,
    AtpSolve,
    BackendRun,
    KernelCheck,
    ArtifactCommit,
    DocumentationExtract,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorkUnit {
    Workspace,
    Package {
        package_id: PackageId,
    },
    Module {
        module: ModuleId,
    },
    Vc {
        module: ModuleId,
        descriptor: VcTaskDescriptorId,
    },
    BackendAttempt {
        module: ModuleId,
        descriptor: VcTaskDescriptorId,
        backend_profile: BackendProfileId,
    },
    EvidenceCandidate {
        module: ModuleId,
        descriptor: VcTaskDescriptorId,
        evidence_candidate: EvidenceCandidateId,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DependencyCoverage {
    Complete,
    PackageConservative,
    MissingModuleDependencyOverlay,
    MissingVcDescriptors,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResourceClass {
    Coordinator,
    SourceIo,
    CpuLocal,
    ProofLocal,
    AtpProcess,
    Kernel,
    ArtifactIo,
    Documentation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PriorityClass {
    Root,
    Source,
    Semantic,
    Proof,
    Commit,
    Documentation,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleDependencyOverlay {
    pub coverage: ModuleDependencyCoverage,
    pub edges: Vec<ModuleDependencyEdge>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleDependencyEdge {
    pub dependent: ModuleId,
    pub dependency: ModuleId,
    pub kind: ModuleDependencyKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModuleDependencyCoverage {
    Complete,
    CoveredModules(Vec<ModuleId>),
    PackageOnly,
    Unavailable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModuleDependencyKind {
    ImportSummary,
    VisibleRegistration,
    PackageConservative,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VcTaskDescriptor {
    pub id: VcTaskDescriptorId,
    pub module: ModuleId,
    pub vc_order_key: VcOrderKey,
    pub backend_profiles: Vec<BackendProfileId>,
    pub evidence_candidates: Vec<EvidenceCandidateId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskGraphProfile {
    pub documentation: DocumentationProfile,
    pub vc_descriptor_policy: VcDescriptorPolicy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DocumentationProfile {
    Disabled,
    Enabled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VcDescriptorPolicy {
    Optional,
    RequiredForArtifactCommit,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VcTaskDescriptorId(String);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BackendProfileId(String);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EvidenceCandidateId(String);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VcOrderKey(String);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskGraphDiagnostics {
    diagnostics: Vec<TaskGraphDiagnostic>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskGraphDiagnostic {
    pub package_id: Option<String>,
    pub module: Option<ModuleId>,
    pub task_id: Option<TaskId>,
    pub kind: TaskGraphDiagnosticKind,
    pub value: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
pub enum TaskGraphDiagnosticKind {
    UnsupportedTaskGraphSchema,
    DuplicateTaskId,
    UnknownPackage,
    UnknownModule,
    DependencyCycle,
    SelfDependency,
    MissingModuleDependencyOverlay,
    MissingVcDescriptors,
    UnsupportedDescriptorFamily,
    BoundaryViolation,
}

#[derive(Clone)]
struct ModuleTaskIds {
    module_resolve: TaskId,
    vc_generate: TaskId,
    artifact_commit: TaskId,
}

struct TaskGraphBuilder {
    input: TaskGraphInput,
    snapshot_identity: String,
    tasks: Vec<BuildTask>,
    task_ids: BTreeSet<TaskId>,
    edges: BTreeSet<(TaskId, TaskId)>,
    diagnostics: Vec<TaskGraphDiagnostic>,
    root_id: Option<TaskId>,
    module_tasks: BTreeMap<String, ModuleTaskIds>,
    descriptor_tasks_by_module: BTreeMap<String, Vec<TaskId>>,
    task_sequence: BTreeMap<TaskId, usize>,
    next_task_sequence: usize,
}

impl TaskGraphVersion {
    #[must_use]
    pub const fn current() -> Self {
        Self(TASK_GRAPH_SCHEMA_VERSION)
    }

    #[must_use]
    pub const fn new(value: u32) -> Self {
        Self(value)
    }

    #[must_use]
    pub const fn value(self) -> u32 {
        self.0
    }
}

impl TaskGraph {
    pub fn build(input: TaskGraphInput) -> Result<Self, TaskGraphDiagnostics> {
        build_task_graph(input)
    }

    #[must_use]
    pub fn tasks(&self) -> &[BuildTask] {
        &self.tasks
    }

    #[must_use]
    pub fn edges(&self) -> &[TaskEdge] {
        &self.edges
    }

    #[must_use]
    pub fn diagnostics(&self) -> &[TaskGraphDiagnostic] {
        &self.diagnostics
    }
}

pub fn build_task_graph(input: TaskGraphInput) -> Result<TaskGraph, TaskGraphDiagnostics> {
    TaskGraphBuilder::new(input).build()
}

impl TaskId {
    #[cfg(test)]
    #[must_use]
    pub(crate) fn new_for_test(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl ModuleDependencyOverlay {
    #[must_use]
    pub fn complete(edges: Vec<ModuleDependencyEdge>) -> Self {
        Self {
            coverage: ModuleDependencyCoverage::Complete,
            edges,
        }
    }

    #[must_use]
    pub fn package_only(edges: Vec<ModuleDependencyEdge>) -> Self {
        Self {
            coverage: ModuleDependencyCoverage::PackageOnly,
            edges,
        }
    }

    #[must_use]
    pub fn unavailable() -> Self {
        Self {
            coverage: ModuleDependencyCoverage::Unavailable,
            edges: Vec::new(),
        }
    }
}

impl Default for ModuleDependencyOverlay {
    fn default() -> Self {
        Self::unavailable()
    }
}

impl ModuleDependencyEdge {
    #[must_use]
    pub fn new(dependent: ModuleId, dependency: ModuleId, kind: ModuleDependencyKind) -> Self {
        Self {
            dependent,
            dependency,
            kind,
        }
    }
}

impl VcTaskDescriptor {
    #[must_use]
    pub fn new(
        id: VcTaskDescriptorId,
        module: ModuleId,
        vc_order_key: VcOrderKey,
        backend_profiles: Vec<BackendProfileId>,
        evidence_candidates: Vec<EvidenceCandidateId>,
    ) -> Self {
        Self {
            id,
            module,
            vc_order_key,
            backend_profiles,
            evidence_candidates,
        }
    }
}

impl Default for TaskGraphProfile {
    fn default() -> Self {
        Self {
            documentation: DocumentationProfile::Disabled,
            vc_descriptor_policy: VcDescriptorPolicy::Optional,
        }
    }
}

impl VcTaskDescriptorId {
    #[must_use]
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl BackendProfileId {
    #[must_use]
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl EvidenceCandidateId {
    #[must_use]
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl VcOrderKey {
    #[must_use]
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TaskGraphDiagnostics {
    #[must_use]
    pub fn new(mut diagnostics: Vec<TaskGraphDiagnostic>) -> Self {
        sort_diagnostics(&mut diagnostics);
        Self { diagnostics }
    }

    #[must_use]
    pub fn diagnostics(&self) -> &[TaskGraphDiagnostic] {
        &self.diagnostics
    }

    #[must_use]
    pub fn into_diagnostics(self) -> Vec<TaskGraphDiagnostic> {
        self.diagnostics
    }
}

impl TaskGraphDiagnostic {
    fn new(
        package_id: Option<String>,
        module: Option<ModuleId>,
        task_id: Option<TaskId>,
        kind: TaskGraphDiagnosticKind,
        value: Option<String>,
    ) -> Self {
        Self {
            package_id,
            module,
            task_id,
            kind,
            value,
        }
    }
}

impl TaskGraphBuilder {
    fn new(input: TaskGraphInput) -> Self {
        let snapshot_identity = input
            .snapshot
            .to_published_schema_string()
            .expect("build snapshot ids have published-schema strings");
        Self {
            input,
            snapshot_identity,
            tasks: Vec::new(),
            task_ids: BTreeSet::new(),
            edges: BTreeSet::new(),
            diagnostics: Vec::new(),
            root_id: None,
            module_tasks: BTreeMap::new(),
            descriptor_tasks_by_module: BTreeMap::new(),
            task_sequence: BTreeMap::new(),
            next_task_sequence: 0,
        }
    }

    fn build(mut self) -> Result<TaskGraph, TaskGraphDiagnostics> {
        if self.input.graph_version != TaskGraphVersion::current() {
            self.diagnostics.push(TaskGraphDiagnostic::new(
                None,
                None,
                None,
                TaskGraphDiagnosticKind::UnsupportedTaskGraphSchema,
                Some(self.input.graph_version.value().to_string()),
            ));
            return Err(TaskGraphDiagnostics::new(self.diagnostics));
        }

        self.validate_package_edges();
        self.validate_overlay_references();
        let valid_descriptors = self.valid_vc_descriptors();
        let descriptor_modules = descriptor_module_keys(&valid_descriptors);
        self.add_root_task();
        self.add_workspace_module_tasks(&descriptor_modules);
        self.add_package_dependency_edges();
        self.add_module_overlay_edges();
        self.add_vc_descriptor_tasks(valid_descriptors);
        self.add_documentation_tasks();
        self.detect_cycles();

        let mut edges = self.sorted_edges();
        let mut tasks = std::mem::take(&mut self.tasks);
        fill_task_dependencies(&mut tasks, &edges);
        sort_tasks(
            &mut tasks,
            &self.input.build_plan,
            &self.input.module_index,
            &self.task_sequence,
        );
        sort_diagnostics(&mut self.diagnostics);
        edges.sort_by(|left, right| edge_sort_key(left).cmp(&edge_sort_key(right)));

        if self.diagnostics.is_empty() {
            Ok(TaskGraph {
                version: self.input.graph_version,
                snapshot: self.input.snapshot,
                tasks,
                edges,
                diagnostics: Vec::new(),
            })
        } else {
            Err(TaskGraphDiagnostics::new(self.diagnostics))
        }
    }

    fn validate_package_edges(&mut self) {
        let packages = self.package_keys();
        for edge in self.input.build_plan.dependency_graph.edges.clone() {
            if !packages.contains(edge.dependent.as_str()) {
                self.diagnostics.push(TaskGraphDiagnostic::new(
                    Some(edge.dependent.clone()),
                    None,
                    None,
                    TaskGraphDiagnosticKind::UnknownPackage,
                    Some(edge.dependent),
                ));
            }
            if !packages.contains(edge.dependency.as_str()) {
                self.diagnostics.push(TaskGraphDiagnostic::new(
                    Some(edge.dependency.clone()),
                    None,
                    None,
                    TaskGraphDiagnosticKind::UnknownPackage,
                    Some(edge.dependency),
                ));
            }
        }
    }

    fn validate_overlay_references(&mut self) {
        let modules = self.module_keys();
        if let ModuleDependencyCoverage::CoveredModules(covered_modules) =
            self.input.dependency_overlay.coverage.clone()
        {
            for module in covered_modules {
                if !modules.contains(module_key(&module).as_str()) {
                    self.diagnostics.push(TaskGraphDiagnostic::new(
                        Some(module.package.as_str().to_owned()),
                        Some(module.clone()),
                        None,
                        TaskGraphDiagnosticKind::UnknownModule,
                        Some(module_key(&module)),
                    ));
                }
            }
        }

        for edge in self.input.dependency_overlay.edges.clone() {
            if edge.dependent == edge.dependency {
                self.diagnostics.push(TaskGraphDiagnostic::new(
                    Some(edge.dependent.package.as_str().to_owned()),
                    Some(edge.dependent.clone()),
                    None,
                    TaskGraphDiagnosticKind::SelfDependency,
                    Some(module_key(&edge.dependent)),
                ));
            }
            for module in [&edge.dependent, &edge.dependency] {
                if !modules.contains(module_key(module).as_str()) {
                    self.diagnostics.push(TaskGraphDiagnostic::new(
                        Some(module.package.as_str().to_owned()),
                        Some(module.clone()),
                        None,
                        TaskGraphDiagnosticKind::UnknownModule,
                        Some(module_key(module)),
                    ));
                }
            }
        }
    }

    fn valid_vc_descriptors(&mut self) -> Vec<VcTaskDescriptor> {
        let modules = self.module_keys();
        let workspace_modules = self.workspace_module_keys();
        let mut descriptors = self.input.vc_descriptors.clone();
        descriptors.sort_by_key(descriptor_sort_key);

        let mut valid_descriptors = Vec::new();
        for descriptor in descriptors {
            let key = module_key(&descriptor.module);
            if !modules.contains(key.as_str()) {
                self.diagnostics.push(TaskGraphDiagnostic::new(
                    Some(descriptor.module.package.as_str().to_owned()),
                    Some(descriptor.module.clone()),
                    None,
                    TaskGraphDiagnosticKind::UnknownModule,
                    Some(key),
                ));
                continue;
            }
            if !workspace_modules.contains(key.as_str()) {
                self.diagnostics.push(TaskGraphDiagnostic::new(
                    Some(descriptor.module.package.as_str().to_owned()),
                    Some(descriptor.module.clone()),
                    None,
                    TaskGraphDiagnosticKind::BoundaryViolation,
                    Some("vc descriptor targets dependency-summary input".to_owned()),
                ));
                continue;
            }
            valid_descriptors.push(descriptor);
        }
        valid_descriptors
    }

    fn add_root_task(&mut self) {
        let root = self.add_task(
            TaskKind::PackageResolve,
            WorkUnit::Workspace,
            DependencyCoverage::Complete,
        );
        self.root_id = Some(root);
    }

    fn add_workspace_module_tasks(&mut self, descriptor_modules: &BTreeSet<String>) {
        for entry in self.workspace_modules() {
            let semantic_coverage = self.semantic_dependency_coverage(&entry.module);
            let artifact_coverage = self.artifact_dependency_coverage(
                &entry.module,
                semantic_coverage,
                descriptor_modules,
            );

            let source_load = self.add_task(
                TaskKind::SourceLoad,
                WorkUnit::Module {
                    module: entry.module.clone(),
                },
                DependencyCoverage::Complete,
            );
            let frontend = self.add_task(
                TaskKind::Frontend,
                WorkUnit::Module {
                    module: entry.module.clone(),
                },
                DependencyCoverage::Complete,
            );
            let module_resolve = self.add_task(
                TaskKind::ModuleResolve,
                WorkUnit::Module {
                    module: entry.module.clone(),
                },
                semantic_coverage,
            );
            let check_and_elaborate = self.add_task(
                TaskKind::CheckAndElaborate,
                WorkUnit::Module {
                    module: entry.module.clone(),
                },
                semantic_coverage,
            );
            let vc_generate = self.add_task(
                TaskKind::VcGenerate,
                WorkUnit::Module {
                    module: entry.module.clone(),
                },
                semantic_coverage,
            );
            let artifact_commit = self.add_task(
                TaskKind::ArtifactCommit,
                WorkUnit::Module {
                    module: entry.module.clone(),
                },
                artifact_coverage,
            );

            self.add_edge(frontend.clone(), source_load.clone());
            self.add_edge(module_resolve.clone(), frontend.clone());
            self.add_edge(check_and_elaborate.clone(), module_resolve.clone());
            self.add_edge(vc_generate.clone(), check_and_elaborate.clone());
            self.add_edge(artifact_commit.clone(), vc_generate.clone());

            self.module_tasks.insert(
                module_key(&entry.module),
                ModuleTaskIds {
                    module_resolve,
                    vc_generate,
                    artifact_commit,
                },
            );
        }
    }

    fn add_package_dependency_edges(&mut self) {
        for edge in self.input.build_plan.dependency_graph.edges.clone() {
            if !self.package_edge_is_known(&edge) {
                continue;
            }
            let dependent_modules = self.workspace_module_keys_for_package(edge.dependent.as_str());
            let dependency_modules =
                self.workspace_module_keys_for_package(edge.dependency.as_str());
            for dependent in &dependent_modules {
                for dependency in &dependency_modules {
                    let Some(dependent_tasks) = self.module_tasks.get(dependent).cloned() else {
                        continue;
                    };
                    let Some(dependency_tasks) = self.module_tasks.get(dependency).cloned() else {
                        continue;
                    };
                    self.add_edge(
                        dependent_tasks.module_resolve.clone(),
                        dependency_tasks.artifact_commit.clone(),
                    );
                }
            }
        }
    }

    fn add_module_overlay_edges(&mut self) {
        let coverage = self.input.dependency_overlay.coverage.clone();
        if matches!(
            coverage,
            ModuleDependencyCoverage::PackageOnly | ModuleDependencyCoverage::Unavailable
        ) {
            return;
        }

        for edge in self.input.dependency_overlay.edges.clone() {
            if edge.dependent == edge.dependency {
                continue;
            }
            if !self.overlay_covers_module(&edge.dependent) {
                continue;
            }
            let dependent_key = module_key(&edge.dependent);
            let dependency_key = module_key(&edge.dependency);
            let Some(dependent_tasks) = self.module_tasks.get(&dependent_key).cloned() else {
                continue;
            };
            let Some(dependency_tasks) = self.module_tasks.get(&dependency_key).cloned() else {
                continue;
            };
            self.add_edge(
                dependent_tasks.module_resolve.clone(),
                dependency_tasks.artifact_commit.clone(),
            );
        }
    }

    fn add_vc_descriptor_tasks(&mut self, descriptors: Vec<VcTaskDescriptor>) {
        for descriptor in descriptors {
            let module_key = module_key(&descriptor.module);
            let Some(module_tasks) = self.module_tasks.get(&module_key).cloned() else {
                continue;
            };

            let vc_discharge = self.add_task(
                TaskKind::VcDischarge,
                WorkUnit::Vc {
                    module: descriptor.module.clone(),
                    descriptor: descriptor.id.clone(),
                },
                DependencyCoverage::Complete,
            );
            self.add_edge(vc_discharge.clone(), module_tasks.vc_generate.clone());
            self.add_descriptor_artifact_edge(&module_key, &vc_discharge);

            let mut backend_run_ids = Vec::new();
            if !descriptor.backend_profiles.is_empty() {
                let atp_solve = self.add_task(
                    TaskKind::AtpSolve,
                    WorkUnit::Vc {
                        module: descriptor.module.clone(),
                        descriptor: descriptor.id.clone(),
                    },
                    DependencyCoverage::Complete,
                );
                self.add_edge(atp_solve.clone(), vc_discharge.clone());
                self.add_descriptor_artifact_edge(&module_key, &atp_solve);

                for backend_profile in sorted_backend_profiles(&descriptor.backend_profiles) {
                    let backend_run = self.add_task(
                        TaskKind::BackendRun,
                        WorkUnit::BackendAttempt {
                            module: descriptor.module.clone(),
                            descriptor: descriptor.id.clone(),
                            backend_profile,
                        },
                        DependencyCoverage::Complete,
                    );
                    self.add_edge(backend_run.clone(), atp_solve.clone());
                    self.add_descriptor_artifact_edge(&module_key, &backend_run);
                    backend_run_ids.push(backend_run);
                }
            }

            for evidence_candidate in sorted_evidence_candidates(&descriptor.evidence_candidates) {
                let kernel_check = self.add_task(
                    TaskKind::KernelCheck,
                    WorkUnit::EvidenceCandidate {
                        module: descriptor.module.clone(),
                        descriptor: descriptor.id.clone(),
                        evidence_candidate,
                    },
                    DependencyCoverage::Complete,
                );
                if backend_run_ids.is_empty() {
                    self.add_edge(kernel_check.clone(), vc_discharge.clone());
                } else {
                    for backend_run in &backend_run_ids {
                        self.add_edge(kernel_check.clone(), backend_run.clone());
                    }
                }
                self.add_descriptor_artifact_edge(&module_key, &kernel_check);
            }
        }
    }

    fn add_documentation_tasks(&mut self) {
        if self.input.profile.documentation != DocumentationProfile::Enabled {
            return;
        }

        for package in self.input.build_plan.packages.clone() {
            let module_keys = self.workspace_module_keys_for_package(package.package_id.as_str());
            if module_keys.is_empty() {
                continue;
            }
            let documentation = self.add_task(
                TaskKind::DocumentationExtract,
                WorkUnit::Package {
                    package_id: package.package_id.clone(),
                },
                DependencyCoverage::Complete,
            );
            for module_key in module_keys {
                let Some(module_tasks) = self.module_tasks.get(&module_key).cloned() else {
                    continue;
                };
                self.add_edge(documentation.clone(), module_tasks.artifact_commit.clone());
            }
        }
    }

    fn add_descriptor_artifact_edge(&mut self, module_key: &str, descriptor_task: &TaskId) {
        self.descriptor_tasks_by_module
            .entry(module_key.to_owned())
            .or_default()
            .push(descriptor_task.clone());
        if let Some(module_tasks) = self.module_tasks.get(module_key).cloned() {
            self.add_edge(module_tasks.artifact_commit, descriptor_task.clone());
        }
    }

    fn add_task(
        &mut self,
        kind: TaskKind,
        unit: WorkUnit,
        dependency_coverage: DependencyCoverage,
    ) -> TaskId {
        let phases = phases_for_kind(kind);
        let id = self.task_id(kind, &unit, &phases);
        if !self.task_ids.insert(id.clone()) {
            self.diagnostics.push(TaskGraphDiagnostic::new(
                package_id_for_unit(&unit),
                module_for_unit(&unit),
                Some(id.clone()),
                TaskGraphDiagnosticKind::DuplicateTaskId,
                Some(id.as_str().to_owned()),
            ));
        }

        self.task_sequence.entry(id.clone()).or_insert_with(|| {
            let sequence = self.next_task_sequence;
            self.next_task_sequence += 1;
            sequence
        });

        self.tasks.push(BuildTask {
            id: id.clone(),
            kind,
            unit,
            phases,
            dependencies: Vec::new(),
            dependency_coverage,
            resource_class: resource_class_for_kind(kind),
            priority_class: priority_class_for_kind(kind),
        });

        if let Some(root_id) = &self.root_id
            && kind != TaskKind::PackageResolve
        {
            self.add_edge(id.clone(), root_id.clone());
        }

        id
    }

    fn add_edge(&mut self, dependent: TaskId, dependency: TaskId) {
        if dependent == dependency {
            self.diagnostics.push(TaskGraphDiagnostic::new(
                None,
                None,
                Some(dependent.clone()),
                TaskGraphDiagnosticKind::SelfDependency,
                Some(dependent.as_str().to_owned()),
            ));
            return;
        }
        self.edges.insert((dependent, dependency));
    }

    fn task_id(&self, kind: TaskKind, unit: &WorkUnit, phases: &[PipelinePhase]) -> TaskId {
        TaskId(format!(
            "mizar-build-task-v{}|snapshot={}|kind={}|unit={}|phases={}",
            self.input.graph_version.value(),
            escape_component(&self.snapshot_identity),
            kind.as_str(),
            work_unit_identity(unit),
            phase_identity(phases)
        ))
    }

    fn semantic_dependency_coverage(&mut self, module: &ModuleId) -> DependencyCoverage {
        match &self.input.dependency_overlay.coverage {
            ModuleDependencyCoverage::Complete => DependencyCoverage::Complete,
            ModuleDependencyCoverage::CoveredModules(covered_modules) => {
                if covered_modules.iter().any(|covered| covered == module) {
                    DependencyCoverage::Complete
                } else {
                    self.push_missing_overlay_diagnostic(module);
                    DependencyCoverage::MissingModuleDependencyOverlay
                }
            }
            ModuleDependencyCoverage::PackageOnly => DependencyCoverage::PackageConservative,
            ModuleDependencyCoverage::Unavailable => {
                self.push_missing_overlay_diagnostic(module);
                DependencyCoverage::MissingModuleDependencyOverlay
            }
        }
    }

    fn artifact_dependency_coverage(
        &mut self,
        module: &ModuleId,
        semantic_coverage: DependencyCoverage,
        descriptor_modules: &BTreeSet<String>,
    ) -> DependencyCoverage {
        if semantic_coverage == DependencyCoverage::MissingModuleDependencyOverlay {
            return semantic_coverage;
        }
        if self.input.profile.vc_descriptor_policy == VcDescriptorPolicy::RequiredForArtifactCommit
            && !descriptor_modules.contains(module_key(module).as_str())
        {
            self.diagnostics.push(TaskGraphDiagnostic::new(
                Some(module.package.as_str().to_owned()),
                Some(module.clone()),
                None,
                TaskGraphDiagnosticKind::MissingVcDescriptors,
                Some(module_key(module)),
            ));
            DependencyCoverage::MissingVcDescriptors
        } else {
            semantic_coverage
        }
    }

    fn push_missing_overlay_diagnostic(&mut self, module: &ModuleId) {
        self.diagnostics.push(TaskGraphDiagnostic::new(
            Some(module.package.as_str().to_owned()),
            Some(module.clone()),
            None,
            TaskGraphDiagnosticKind::MissingModuleDependencyOverlay,
            Some(module_key(module)),
        ));
    }

    fn detect_cycles(&mut self) {
        let mut indegree = BTreeMap::new();
        let mut outgoing: BTreeMap<TaskId, Vec<TaskId>> = BTreeMap::new();
        for task in &self.tasks {
            indegree.entry(task.id.clone()).or_insert(0_usize);
        }
        for (dependent, dependency) in &self.edges {
            outgoing
                .entry(dependency.clone())
                .or_default()
                .push(dependent.clone());
            *indegree.entry(dependent.clone()).or_insert(0) += 1;
            indegree.entry(dependency.clone()).or_insert(0);
        }

        let mut queue = indegree
            .iter()
            .filter(|(_task_id, degree)| **degree == 0)
            .map(|(task_id, _degree)| task_id.clone())
            .collect::<VecDeque<_>>();
        let mut visited = 0_usize;
        while let Some(task_id) = queue.pop_front() {
            visited += 1;
            if let Some(dependents) = outgoing.get(&task_id) {
                for dependent in dependents {
                    let degree = indegree
                        .get_mut(dependent)
                        .expect("dependent task has an indegree entry");
                    *degree -= 1;
                    if *degree == 0 {
                        queue.push_back(dependent.clone());
                    }
                }
            }
        }

        if visited != indegree.len() {
            let cycle_task = indegree
                .iter()
                .find_map(|(task_id, degree)| (*degree > 0).then(|| task_id.clone()));
            self.diagnostics.push(TaskGraphDiagnostic::new(
                None,
                None,
                cycle_task.clone(),
                TaskGraphDiagnosticKind::DependencyCycle,
                cycle_task.map(|task_id| task_id.as_str().to_owned()),
            ));
        }
    }

    fn sorted_edges(&self) -> Vec<TaskEdge> {
        self.edges
            .iter()
            .map(|(dependent, dependency)| TaskEdge {
                dependent: dependent.clone(),
                dependency: dependency.clone(),
            })
            .collect()
    }

    fn package_keys(&self) -> BTreeSet<String> {
        self.input
            .build_plan
            .packages
            .iter()
            .map(|package| package.package_id.as_str().to_owned())
            .collect()
    }

    fn module_keys(&self) -> BTreeSet<String> {
        self.input
            .module_index
            .modules
            .iter()
            .map(|entry| module_key(&entry.module))
            .collect()
    }

    fn workspace_module_keys(&self) -> BTreeSet<String> {
        self.workspace_modules()
            .into_iter()
            .map(|entry| module_key(&entry.module))
            .collect()
    }

    fn workspace_modules(&self) -> Vec<ModuleIndexEntry> {
        self.input
            .module_index
            .modules
            .iter()
            .filter(|entry| matches!(entry.location, ModuleIndexLocation::WorkspaceFile { .. }))
            .cloned()
            .collect()
    }

    fn workspace_module_keys_for_package(&self, package_id: &str) -> Vec<String> {
        self.workspace_modules()
            .into_iter()
            .filter(|entry| entry.module.package.as_str() == package_id)
            .map(|entry| module_key(&entry.module))
            .collect()
    }

    fn package_edge_is_known(&self, edge: &DependencyEdge) -> bool {
        let packages = self.package_keys();
        packages.contains(&edge.dependent) && packages.contains(&edge.dependency)
    }

    fn overlay_covers_module(&self, module: &ModuleId) -> bool {
        match &self.input.dependency_overlay.coverage {
            ModuleDependencyCoverage::Complete => true,
            ModuleDependencyCoverage::CoveredModules(covered_modules) => {
                covered_modules.iter().any(|covered| covered == module)
            }
            ModuleDependencyCoverage::PackageOnly | ModuleDependencyCoverage::Unavailable => false,
        }
    }
}

impl TaskKind {
    fn as_str(self) -> &'static str {
        match self {
            Self::PackageResolve => "PackageResolve",
            Self::SourceLoad => "SourceLoad",
            Self::Frontend => "Frontend",
            Self::ModuleResolve => "ModuleResolve",
            Self::CheckAndElaborate => "CheckAndElaborate",
            Self::VcGenerate => "VcGenerate",
            Self::VcDischarge => "VcDischarge",
            Self::AtpSolve => "AtpSolve",
            Self::BackendRun => "BackendRun",
            Self::KernelCheck => "KernelCheck",
            Self::ArtifactCommit => "ArtifactCommit",
            Self::DocumentationExtract => "DocumentationExtract",
        }
    }
}

impl PipelinePhase {
    fn as_str(self) -> &'static str {
        match self {
            Self::PackageResolve => "PackageResolve",
            Self::SourceLoad => "SourceLoad",
            Self::Frontend => "Frontend",
            Self::ModuleResolve => "ModuleResolve",
            Self::SignatureCollection => "SignatureCollection",
            Self::TypeChecking => "TypeChecking",
            Self::RegistrationResolution => "RegistrationResolution",
            Self::OverloadResolution => "OverloadResolution",
            Self::Elaboration => "Elaboration",
            Self::AlgorithmPreparation => "AlgorithmPreparation",
            Self::VcGenerate => "VcGenerate",
            Self::VcDischarge => "VcDischarge",
            Self::AtpSolve => "AtpSolve",
            Self::BackendRun => "BackendRun",
            Self::KernelCheck => "KernelCheck",
            Self::ArtifactCommit => "ArtifactCommit",
            Self::DocumentationExtract => "DocumentationExtract",
        }
    }
}

fn phases_for_kind(kind: TaskKind) -> Vec<PipelinePhase> {
    match kind {
        TaskKind::PackageResolve => vec![PipelinePhase::PackageResolve],
        TaskKind::SourceLoad => vec![PipelinePhase::SourceLoad],
        TaskKind::Frontend => vec![PipelinePhase::Frontend],
        TaskKind::ModuleResolve => vec![PipelinePhase::ModuleResolve],
        TaskKind::CheckAndElaborate => vec![
            PipelinePhase::SignatureCollection,
            PipelinePhase::TypeChecking,
            PipelinePhase::RegistrationResolution,
            PipelinePhase::OverloadResolution,
            PipelinePhase::Elaboration,
            PipelinePhase::AlgorithmPreparation,
        ],
        TaskKind::VcGenerate => vec![PipelinePhase::VcGenerate],
        TaskKind::VcDischarge => vec![PipelinePhase::VcDischarge],
        TaskKind::AtpSolve => vec![PipelinePhase::AtpSolve],
        TaskKind::BackendRun => vec![PipelinePhase::BackendRun],
        TaskKind::KernelCheck => vec![PipelinePhase::KernelCheck],
        TaskKind::ArtifactCommit => vec![PipelinePhase::ArtifactCommit],
        TaskKind::DocumentationExtract => vec![PipelinePhase::DocumentationExtract],
    }
}

fn resource_class_for_kind(kind: TaskKind) -> ResourceClass {
    match kind {
        TaskKind::PackageResolve => ResourceClass::Coordinator,
        TaskKind::SourceLoad => ResourceClass::SourceIo,
        TaskKind::Frontend
        | TaskKind::ModuleResolve
        | TaskKind::CheckAndElaborate
        | TaskKind::VcGenerate => ResourceClass::CpuLocal,
        TaskKind::VcDischarge => ResourceClass::ProofLocal,
        TaskKind::AtpSolve | TaskKind::BackendRun => ResourceClass::AtpProcess,
        TaskKind::KernelCheck => ResourceClass::Kernel,
        TaskKind::ArtifactCommit => ResourceClass::ArtifactIo,
        TaskKind::DocumentationExtract => ResourceClass::Documentation,
    }
}

fn priority_class_for_kind(kind: TaskKind) -> PriorityClass {
    match kind {
        TaskKind::PackageResolve => PriorityClass::Root,
        TaskKind::SourceLoad | TaskKind::Frontend => PriorityClass::Source,
        TaskKind::ModuleResolve | TaskKind::CheckAndElaborate | TaskKind::VcGenerate => {
            PriorityClass::Semantic
        }
        TaskKind::VcDischarge
        | TaskKind::AtpSolve
        | TaskKind::BackendRun
        | TaskKind::KernelCheck => PriorityClass::Proof,
        TaskKind::ArtifactCommit => PriorityClass::Commit,
        TaskKind::DocumentationExtract => PriorityClass::Documentation,
    }
}

fn descriptor_module_keys(descriptors: &[VcTaskDescriptor]) -> BTreeSet<String> {
    descriptors
        .iter()
        .map(|descriptor| module_key(&descriptor.module))
        .collect()
}

fn sorted_backend_profiles(profiles: &[BackendProfileId]) -> Vec<BackendProfileId> {
    let mut indexed = profiles.iter().cloned().enumerate().collect::<Vec<_>>();
    indexed.sort_by(|left, right| (left.0, left.1.as_str()).cmp(&(right.0, right.1.as_str())));
    indexed
        .into_iter()
        .map(|(_priority, profile)| profile)
        .collect()
}

fn sorted_evidence_candidates(candidates: &[EvidenceCandidateId]) -> Vec<EvidenceCandidateId> {
    let mut sorted = candidates.to_vec();
    sorted.sort_by(|left, right| left.as_str().cmp(right.as_str()));
    sorted
}

fn fill_task_dependencies(tasks: &mut [BuildTask], edges: &[TaskEdge]) {
    let mut dependencies_by_task: BTreeMap<TaskId, Vec<TaskId>> = BTreeMap::new();
    for edge in edges {
        dependencies_by_task
            .entry(edge.dependent.clone())
            .or_default()
            .push(edge.dependency.clone());
    }
    for dependencies in dependencies_by_task.values_mut() {
        dependencies.sort();
        dependencies.dedup();
    }
    for task in tasks {
        task.dependencies = dependencies_by_task.remove(&task.id).unwrap_or_default();
    }
}

fn sort_tasks(
    tasks: &mut [BuildTask],
    build_plan: &BuildPlan,
    module_index: &ModuleIndex,
    task_sequence: &BTreeMap<TaskId, usize>,
) {
    let package_order = build_plan
        .packages
        .iter()
        .enumerate()
        .map(|(order, package)| (package.package_id.as_str().to_owned(), order))
        .collect::<BTreeMap<_, _>>();
    let module_order = module_index
        .modules
        .iter()
        .enumerate()
        .map(|(order, entry)| (module_key(&entry.module), order))
        .collect::<BTreeMap<_, _>>();

    tasks.sort_by(|left, right| {
        task_sort_key(left, &package_order, &module_order, task_sequence).cmp(&task_sort_key(
            right,
            &package_order,
            &module_order,
            task_sequence,
        ))
    });
}

fn task_sort_key(
    task: &BuildTask,
    package_order: &BTreeMap<String, usize>,
    module_order: &BTreeMap<String, usize>,
    task_sequence: &BTreeMap<TaskId, usize>,
) -> (usize, usize, usize, usize, usize, String) {
    let root_rank = usize::from(task.kind != TaskKind::PackageResolve);
    let package_rank = package_for_unit(&task.unit)
        .and_then(|package| package_order.get(package.as_str()).copied())
        .unwrap_or(usize::MAX);
    let module_rank = module_for_unit(&task.unit)
        .and_then(|module| module_order.get(module_key(&module).as_str()).copied())
        .unwrap_or(usize::MAX);
    let sequence = task_sequence.get(&task.id).copied().unwrap_or(usize::MAX);
    (
        root_rank,
        package_rank,
        module_rank,
        task_kind_rank(task.kind),
        sequence,
        task.id.as_str().to_owned(),
    )
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

fn sort_diagnostics(diagnostics: &mut [TaskGraphDiagnostic]) {
    diagnostics.sort_by_key(diagnostic_sort_key);
}

fn diagnostic_sort_key(
    diagnostic: &TaskGraphDiagnostic,
) -> (String, String, usize, String, String) {
    (
        diagnostic.package_id.clone().unwrap_or_default(),
        diagnostic
            .module
            .as_ref()
            .map(module_key)
            .unwrap_or_default(),
        diagnostic_kind_rank(diagnostic.kind),
        diagnostic
            .task_id
            .as_ref()
            .map(|task_id| task_id.as_str().to_owned())
            .unwrap_or_default(),
        diagnostic.value.clone().unwrap_or_default(),
    )
}

fn diagnostic_kind_rank(kind: TaskGraphDiagnosticKind) -> usize {
    match kind {
        TaskGraphDiagnosticKind::UnsupportedTaskGraphSchema => 0,
        TaskGraphDiagnosticKind::UnknownPackage => 1,
        TaskGraphDiagnosticKind::UnknownModule => 2,
        TaskGraphDiagnosticKind::SelfDependency => 3,
        TaskGraphDiagnosticKind::MissingModuleDependencyOverlay => 4,
        TaskGraphDiagnosticKind::MissingVcDescriptors => 5,
        TaskGraphDiagnosticKind::DuplicateTaskId => 6,
        TaskGraphDiagnosticKind::DependencyCycle => 7,
        TaskGraphDiagnosticKind::UnsupportedDescriptorFamily => 8,
        TaskGraphDiagnosticKind::BoundaryViolation => 9,
    }
}

fn edge_sort_key(edge: &TaskEdge) -> (&str, &str) {
    (edge.dependent.as_str(), edge.dependency.as_str())
}

fn descriptor_sort_key(descriptor: &VcTaskDescriptor) -> (String, String, String) {
    (
        module_key(&descriptor.module),
        descriptor.vc_order_key.as_str().to_owned(),
        descriptor.id.as_str().to_owned(),
    )
}

fn module_key(module: &ModuleId) -> String {
    format!("{}:{}", module.package.as_str(), module.path.as_str())
}

fn package_for_unit(unit: &WorkUnit) -> Option<PackageId> {
    match unit {
        WorkUnit::Workspace => None,
        WorkUnit::Package { package_id } => Some(package_id.clone()),
        WorkUnit::Module { module }
        | WorkUnit::Vc { module, .. }
        | WorkUnit::BackendAttempt { module, .. }
        | WorkUnit::EvidenceCandidate { module, .. } => Some(module.package.clone()),
    }
}

fn package_id_for_unit(unit: &WorkUnit) -> Option<String> {
    package_for_unit(unit).map(|package| package.as_str().to_owned())
}

fn module_for_unit(unit: &WorkUnit) -> Option<ModuleId> {
    match unit {
        WorkUnit::Workspace | WorkUnit::Package { .. } => None,
        WorkUnit::Module { module }
        | WorkUnit::Vc { module, .. }
        | WorkUnit::BackendAttempt { module, .. }
        | WorkUnit::EvidenceCandidate { module, .. } => Some(module.clone()),
    }
}

fn work_unit_identity(unit: &WorkUnit) -> String {
    match unit {
        WorkUnit::Workspace => "workspace".to_owned(),
        WorkUnit::Package { package_id } => {
            format!("package={}", escape_component(package_id.as_str()))
        }
        WorkUnit::Module { module } => format!("module={}", escape_component(&module_key(module))),
        WorkUnit::Vc { module, descriptor } => format!(
            "vc={}:{}",
            escape_component(&module_key(module)),
            escape_component(descriptor.as_str())
        ),
        WorkUnit::BackendAttempt {
            module,
            descriptor,
            backend_profile,
        } => format!(
            "backend={}:{}:{}",
            escape_component(&module_key(module)),
            escape_component(descriptor.as_str()),
            escape_component(backend_profile.as_str())
        ),
        WorkUnit::EvidenceCandidate {
            module,
            descriptor,
            evidence_candidate,
        } => format!(
            "evidence={}:{}:{}",
            escape_component(&module_key(module)),
            escape_component(descriptor.as_str()),
            escape_component(evidence_candidate.as_str())
        ),
    }
}

fn phase_identity(phases: &[PipelinePhase]) -> String {
    let mut identity = String::new();
    for (index, phase) in phases.iter().enumerate() {
        if index > 0 {
            identity.push(',');
        }
        identity.push_str(phase.as_str());
    }
    identity
}

fn escape_component(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for character in value.chars() {
        match character {
            '%' => escaped.push_str("%25"),
            '|' => escaped.push_str("%7C"),
            '=' => escaped.push_str("%3D"),
            ',' => escaped.push_str("%2C"),
            ':' => escaped.push_str("%3A"),
            character => escaped.push(character),
        }
    }
    escaped
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        module_index::{ModuleIndexLocation, PackageIndexEntry, PackageIndexSource},
        planner::{
            BuildConfig, DependencyGraph, DependencyKind, Lockfile, PackagePlan, PackagePlanSource,
            VerifierConfig, WorkspaceBuildConfig, WorkspaceVerifierConfig,
        },
    };
    use mizar_session::{Edition, Hash, ModulePath, ToolchainInfo, WorkspaceRoot};
    use semver::Version;

    #[test]
    fn task_ids_are_deterministic_and_snapshot_scoped() {
        let graph = graph_for_modules(vec![workspace_module("app", "main")], Vec::new());
        let second_graph = graph_for_modules(vec![workspace_module("app", "main")], Vec::new());
        let different_snapshot = snapshot(15);
        let different_snapshot_graph = graph_for_modules_with_snapshot(
            different_snapshot,
            vec![workspace_module("app", "main")],
            Vec::new(),
        );
        let different_snapshot_identity = different_snapshot
            .to_published_schema_string()
            .expect("snapshot identity serializes");

        let ids = task_ids(&graph);
        assert_eq!(ids, task_ids(&second_graph));
        assert_ne!(ids, task_ids(&different_snapshot_graph));
        assert!(
            task_ids(&different_snapshot_graph)
                .iter()
                .all(|id| id.contains(&escape_component(&different_snapshot_identity)))
        );
        assert_eq!(graph.tasks[0].kind, TaskKind::PackageResolve);
        assert!(
            ids.iter()
                .all(|id| id.contains("mizar-session-build-snapshot-v1"))
        );
        assert!(
            ids.iter()
                .all(|id| !contains_forbidden_authority_term(id.as_str()))
        );
    }

    #[test]
    fn workspace_modules_create_phase_tasks_and_summary_modules_remain_inputs() {
        let graph = graph_for_modules(
            vec![
                workspace_module("app", "main"),
                dependency_summary_module("registry_dep", "core"),
            ],
            Vec::new(),
        );

        let workspace_kinds = module_task_kinds(&graph, "app", "main");
        assert_eq!(
            workspace_kinds,
            vec![
                TaskKind::SourceLoad,
                TaskKind::Frontend,
                TaskKind::ModuleResolve,
                TaskKind::CheckAndElaborate,
                TaskKind::VcGenerate,
                TaskKind::ArtifactCommit,
            ]
        );
        assert!(module_task_kinds(&graph, "registry_dep", "core").is_empty());

        let check_task = module_task(&graph, TaskKind::CheckAndElaborate, "app", "main");
        assert_eq!(
            check_task.phases,
            vec![
                PipelinePhase::SignatureCollection,
                PipelinePhase::TypeChecking,
                PipelinePhase::RegistrationResolution,
                PipelinePhase::OverloadResolution,
                PipelinePhase::Elaboration,
                PipelinePhase::AlgorithmPreparation,
            ]
        );
        assert_eq!(check_task.resource_class, ResourceClass::CpuLocal);
        assert_eq!(check_task.priority_class, PriorityClass::Semantic);
    }

    #[test]
    fn tasks_preserve_package_module_descriptor_backend_and_evidence_ordering() {
        let app_main = module_id("app", "main");
        let graph = build_task_graph(TaskGraphInput {
            graph_version: TaskGraphVersion::current(),
            snapshot: snapshot(1),
            build_plan: build_plan(
                vec![workspace_package("lib"), workspace_package("app")],
                Vec::new(),
            ),
            module_index: module_index(vec![
                workspace_module("lib", "core"),
                workspace_module("app", "main"),
            ]),
            dependency_overlay: ModuleDependencyOverlay::complete(Vec::new()),
            vc_descriptors: vec![
                vc_descriptor(
                    "vc-late",
                    app_main.clone(),
                    "002",
                    vec!["z3", "cvc5"],
                    vec!["candidate-b", "candidate-a"],
                ),
                vc_descriptor("vc-early", app_main, "001", vec!["vampire"], Vec::new()),
            ],
            profile: TaskGraphProfile {
                documentation: DocumentationProfile::Enabled,
                ..TaskGraphProfile::default()
            },
        })
        .expect("task graph builds");

        assert_eq!(
            graph
                .tasks
                .iter()
                .filter(|task| task.kind == TaskKind::SourceLoad)
                .map(module_label)
                .collect::<Vec<_>>(),
            vec!["lib:core", "app:main"]
        );
        assert_eq!(
            graph
                .tasks
                .iter()
                .filter(|task| task.kind == TaskKind::VcDischarge)
                .map(descriptor_label)
                .collect::<Vec<_>>(),
            vec!["vc-early", "vc-late"]
        );
        assert_eq!(
            graph
                .tasks
                .iter()
                .filter(|task| task.kind == TaskKind::BackendRun)
                .map(backend_label)
                .collect::<Vec<_>>(),
            vec!["vampire", "z3", "cvc5"]
        );
        assert_eq!(
            graph
                .tasks
                .iter()
                .filter(|task| task.kind == TaskKind::KernelCheck)
                .map(evidence_label)
                .collect::<Vec<_>>(),
            vec!["candidate-a", "candidate-b"]
        );
        assert_eq!(
            graph
                .tasks
                .iter()
                .filter(|task| task.kind == TaskKind::DocumentationExtract)
                .map(package_label)
                .collect::<Vec<_>>(),
            vec!["lib", "app"]
        );
    }

    #[test]
    fn required_edge_rules_cover_root_pipeline_proof_commit_and_docs() {
        let graph = build_task_graph(TaskGraphInput {
            graph_version: TaskGraphVersion::current(),
            snapshot: snapshot(10),
            build_plan: build_plan(vec![workspace_package("app")], Vec::new()),
            module_index: module_index(vec![workspace_module("app", "main")]),
            dependency_overlay: ModuleDependencyOverlay::complete(Vec::new()),
            vc_descriptors: vec![vc_descriptor(
                "vc-main",
                module_id("app", "main"),
                "001",
                vec!["vampire"],
                vec!["kernel-candidate"],
            )],
            profile: TaskGraphProfile {
                documentation: DocumentationProfile::Enabled,
                ..TaskGraphProfile::default()
            },
        })
        .expect("task graph builds");

        let root = single_task(&graph, TaskKind::PackageResolve);
        let source = module_task(&graph, TaskKind::SourceLoad, "app", "main");
        let frontend = module_task(&graph, TaskKind::Frontend, "app", "main");
        let module_resolve = module_task(&graph, TaskKind::ModuleResolve, "app", "main");
        let check = module_task(&graph, TaskKind::CheckAndElaborate, "app", "main");
        let vc_generate = module_task(&graph, TaskKind::VcGenerate, "app", "main");
        let vc_discharge = single_task(&graph, TaskKind::VcDischarge);
        let atp = single_task(&graph, TaskKind::AtpSolve);
        let backend = single_task(&graph, TaskKind::BackendRun);
        let kernel = single_task(&graph, TaskKind::KernelCheck);
        let artifact = module_task(&graph, TaskKind::ArtifactCommit, "app", "main");
        let documentation = single_task(&graph, TaskKind::DocumentationExtract);

        for task in graph
            .tasks
            .iter()
            .filter(|task| task.kind != TaskKind::PackageResolve)
        {
            assert_has_edge(&graph, task, root);
        }
        assert_has_edge(&graph, frontend, source);
        assert_has_edge(&graph, module_resolve, frontend);
        assert_has_edge(&graph, check, module_resolve);
        assert_has_edge(&graph, vc_generate, check);
        assert_has_edge(&graph, vc_discharge, vc_generate);
        assert_has_edge(&graph, atp, vc_discharge);
        assert_has_edge(&graph, backend, atp);
        assert_has_edge(&graph, kernel, backend);
        assert_has_edge(&graph, artifact, vc_generate);
        assert_has_edge(&graph, artifact, vc_discharge);
        assert_has_edge(&graph, artifact, atp);
        assert_has_edge(&graph, artifact, backend);
        assert_has_edge(&graph, artifact, kernel);
        assert_has_edge(&graph, documentation, artifact);
        assert_dependencies_match_edges(&graph);
        assert_edges_are_sorted(&graph);

        assert_task_metadata(
            root,
            DependencyCoverage::Complete,
            ResourceClass::Coordinator,
            PriorityClass::Root,
        );
        assert_task_metadata(
            source,
            DependencyCoverage::Complete,
            ResourceClass::SourceIo,
            PriorityClass::Source,
        );
        assert_task_metadata(
            vc_discharge,
            DependencyCoverage::Complete,
            ResourceClass::ProofLocal,
            PriorityClass::Proof,
        );
        assert_task_metadata(
            atp,
            DependencyCoverage::Complete,
            ResourceClass::AtpProcess,
            PriorityClass::Proof,
        );
        assert_task_metadata(
            backend,
            DependencyCoverage::Complete,
            ResourceClass::AtpProcess,
            PriorityClass::Proof,
        );
        assert_task_metadata(
            kernel,
            DependencyCoverage::Complete,
            ResourceClass::Kernel,
            PriorityClass::Proof,
        );
        assert_task_metadata(
            artifact,
            DependencyCoverage::Complete,
            ResourceClass::ArtifactIo,
            PriorityClass::Commit,
        );
        assert_task_metadata(
            documentation,
            DependencyCoverage::Complete,
            ResourceClass::Documentation,
            PriorityClass::Documentation,
        );
    }

    #[test]
    fn kernel_checks_with_deterministic_evidence_wait_on_vc_discharge() {
        let graph = build_task_graph(TaskGraphInput {
            graph_version: TaskGraphVersion::current(),
            snapshot: snapshot(16),
            build_plan: build_plan(vec![workspace_package("app")], Vec::new()),
            module_index: module_index(vec![workspace_module("app", "main")]),
            dependency_overlay: ModuleDependencyOverlay::complete(Vec::new()),
            vc_descriptors: vec![vc_descriptor(
                "vc-main",
                module_id("app", "main"),
                "001",
                Vec::new(),
                vec!["deterministic-candidate"],
            )],
            profile: TaskGraphProfile::default(),
        })
        .expect("task graph builds");

        let vc_discharge = single_task(&graph, TaskKind::VcDischarge);
        let kernel = single_task(&graph, TaskKind::KernelCheck);
        assert_has_edge(&graph, kernel, vc_discharge);
        assert!(
            graph.tasks.iter().all(|task| {
                task.kind != TaskKind::AtpSolve && task.kind != TaskKind::BackendRun
            })
        );
    }

    #[test]
    fn package_dependency_edges_gate_downstream_semantic_tasks() {
        let graph = build_task_graph(TaskGraphInput {
            graph_version: TaskGraphVersion::current(),
            snapshot: snapshot(2),
            build_plan: build_plan(
                vec![workspace_package("lib"), workspace_package("app")],
                vec![DependencyEdge {
                    dependent: "app".to_owned(),
                    dependency: "lib".to_owned(),
                    kind: DependencyKind::Normal,
                }],
            ),
            module_index: module_index(vec![
                workspace_module("lib", "core"),
                workspace_module("app", "main"),
            ]),
            dependency_overlay: ModuleDependencyOverlay::complete(Vec::new()),
            vc_descriptors: Vec::new(),
            profile: TaskGraphProfile::default(),
        })
        .expect("task graph builds");

        assert_has_edge(
            &graph,
            module_task(&graph, TaskKind::ModuleResolve, "app", "main"),
            module_task(&graph, TaskKind::ArtifactCommit, "lib", "core"),
        );
    }

    #[test]
    fn dependency_summary_package_dependencies_are_ready_inputs() {
        let graph = build_task_graph(TaskGraphInput {
            graph_version: TaskGraphVersion::current(),
            snapshot: snapshot(11),
            build_plan: build_plan(
                vec![registry_package("dep"), workspace_package("app")],
                vec![DependencyEdge {
                    dependent: "app".to_owned(),
                    dependency: "dep".to_owned(),
                    kind: DependencyKind::Normal,
                }],
            ),
            module_index: module_index(vec![
                dependency_summary_module("dep", "core"),
                workspace_module("app", "main"),
            ]),
            dependency_overlay: ModuleDependencyOverlay::complete(Vec::new()),
            vc_descriptors: Vec::new(),
            profile: TaskGraphProfile::default(),
        })
        .expect("dependency-summary package is a ready input");

        assert!(module_task_kinds(&graph, "dep", "core").is_empty());
        assert_eq!(
            module_task(&graph, TaskKind::ModuleResolve, "app", "main")
                .dependencies
                .iter()
                .filter(|dependency| dependency.as_str().contains("dep%3Acore"))
                .count(),
            0
        );
    }

    #[test]
    fn module_dependency_overlay_edges_gate_dependent_module_resolution() {
        let main = module_id("app", "main");
        let util = module_id("app", "util");
        let graph =
            build_task_graph(TaskGraphInput {
                graph_version: TaskGraphVersion::current(),
                snapshot: snapshot(3),
                build_plan: build_plan(vec![workspace_package("app")], Vec::new()),
                module_index: module_index(vec![
                    workspace_module("app", "util"),
                    workspace_module("app", "main"),
                ]),
                dependency_overlay: ModuleDependencyOverlay::complete(vec![
                    ModuleDependencyEdge::new(main, util, ModuleDependencyKind::ImportSummary),
                ]),
                vc_descriptors: Vec::new(),
                profile: TaskGraphProfile::default(),
            })
            .expect("task graph builds");

        assert_has_edge(
            &graph,
            module_task(&graph, TaskKind::ModuleResolve, "app", "main"),
            module_task(&graph, TaskKind::ArtifactCommit, "app", "util"),
        );
    }

    #[test]
    fn missing_module_dependency_coverage_is_diagnostic() {
        let missing = module_id("app", "main");
        let covered = module_id("app", "util");
        let diagnostics = build_task_graph(TaskGraphInput {
            graph_version: TaskGraphVersion::current(),
            snapshot: snapshot(4),
            build_plan: build_plan(vec![workspace_package("app")], Vec::new()),
            module_index: module_index(vec![
                workspace_module("app", "main"),
                workspace_module("app", "util"),
            ]),
            dependency_overlay: ModuleDependencyOverlay {
                coverage: ModuleDependencyCoverage::CoveredModules(vec![covered]),
                edges: Vec::new(),
            },
            vc_descriptors: Vec::new(),
            profile: TaskGraphProfile::default(),
        })
        .expect_err("missing overlay is reported");

        assert!(diagnostics.diagnostics().iter().any(|diagnostic| {
            diagnostic.kind == TaskGraphDiagnosticKind::MissingModuleDependencyOverlay
                && diagnostic.module.as_ref() == Some(&missing)
        }));
    }

    #[test]
    fn package_only_coverage_conservatively_marks_semantic_tasks() {
        let graph = build_task_graph(TaskGraphInput {
            graph_version: TaskGraphVersion::current(),
            snapshot: snapshot(12),
            build_plan: build_plan(vec![workspace_package("app")], Vec::new()),
            module_index: module_index(vec![workspace_module("app", "main")]),
            dependency_overlay: ModuleDependencyOverlay::package_only(Vec::new()),
            vc_descriptors: Vec::new(),
            profile: TaskGraphProfile::default(),
        })
        .expect("package-only coverage is conservatively schedulable");

        assert_eq!(
            module_task(&graph, TaskKind::SourceLoad, "app", "main").dependency_coverage,
            DependencyCoverage::Complete
        );
        assert_eq!(
            module_task(&graph, TaskKind::Frontend, "app", "main").dependency_coverage,
            DependencyCoverage::Complete
        );
        for kind in [
            TaskKind::ModuleResolve,
            TaskKind::CheckAndElaborate,
            TaskKind::VcGenerate,
            TaskKind::ArtifactCommit,
        ] {
            assert_eq!(
                module_task(&graph, kind, "app", "main").dependency_coverage,
                DependencyCoverage::PackageConservative
            );
        }
    }

    #[test]
    fn unavailable_coverage_reports_missing_overlay_for_each_module() {
        let diagnostics = build_task_graph(TaskGraphInput {
            graph_version: TaskGraphVersion::current(),
            snapshot: snapshot(13),
            build_plan: build_plan(vec![workspace_package("app")], Vec::new()),
            module_index: module_index(vec![
                workspace_module("app", "main"),
                workspace_module("app", "util"),
            ]),
            dependency_overlay: ModuleDependencyOverlay::unavailable(),
            vc_descriptors: Vec::new(),
            profile: TaskGraphProfile::default(),
        })
        .expect_err("unavailable overlay is reported");

        assert_eq!(
            diagnostics
                .diagnostics()
                .iter()
                .map(|diagnostic| diagnostic.kind)
                .collect::<Vec<_>>(),
            vec![
                TaskGraphDiagnosticKind::MissingModuleDependencyOverlay,
                TaskGraphDiagnosticKind::MissingModuleDependencyOverlay,
            ]
        );
        assert_eq!(
            diagnostics
                .diagnostics()
                .iter()
                .map(|diagnostic| diagnostic.module.as_ref().map(module_key))
                .collect::<Vec<_>>(),
            vec![Some("app:main".to_owned()), Some("app:util".to_owned())]
        );
    }

    #[test]
    fn required_vc_descriptors_mark_artifact_commit_coverage() {
        let diagnostics = build_task_graph(TaskGraphInput {
            graph_version: TaskGraphVersion::current(),
            snapshot: snapshot(5),
            build_plan: build_plan(vec![workspace_package("app")], Vec::new()),
            module_index: module_index(vec![workspace_module("app", "main")]),
            dependency_overlay: ModuleDependencyOverlay::complete(Vec::new()),
            vc_descriptors: Vec::new(),
            profile: TaskGraphProfile {
                vc_descriptor_policy: VcDescriptorPolicy::RequiredForArtifactCommit,
                ..TaskGraphProfile::default()
            },
        })
        .expect_err("missing VC descriptors are reported");

        assert_eq!(
            diagnostics
                .diagnostics()
                .iter()
                .map(|diagnostic| diagnostic.kind)
                .collect::<Vec<_>>(),
            vec![TaskGraphDiagnosticKind::MissingVcDescriptors]
        );
    }

    #[test]
    fn vc_descriptors_targeting_dependency_summaries_are_rejected() {
        let diagnostics = build_task_graph(TaskGraphInput {
            graph_version: TaskGraphVersion::current(),
            snapshot: snapshot(17),
            build_plan: build_plan(vec![registry_package("dep")], Vec::new()),
            module_index: module_index(vec![dependency_summary_module("dep", "core")]),
            dependency_overlay: ModuleDependencyOverlay::complete(Vec::new()),
            vc_descriptors: vec![vc_descriptor(
                "vc-dep",
                module_id("dep", "core"),
                "001",
                Vec::new(),
                Vec::new(),
            )],
            profile: TaskGraphProfile::default(),
        })
        .expect_err("dependency-summary descriptor is rejected");

        assert_eq!(
            diagnostics
                .diagnostics()
                .iter()
                .map(|diagnostic| diagnostic.kind)
                .collect::<Vec<_>>(),
            vec![TaskGraphDiagnosticKind::BoundaryViolation]
        );
    }

    #[test]
    fn duplicate_task_ids_are_rejected() {
        let diagnostics = build_task_graph(TaskGraphInput {
            graph_version: TaskGraphVersion::current(),
            snapshot: snapshot(6),
            build_plan: build_plan(vec![workspace_package("app")], Vec::new()),
            module_index: module_index(vec![
                workspace_module("app", "main"),
                workspace_module("app", "main"),
            ]),
            dependency_overlay: ModuleDependencyOverlay::complete(Vec::new()),
            vc_descriptors: Vec::new(),
            profile: TaskGraphProfile::default(),
        })
        .expect_err("duplicate task ids are reported");

        assert!(
            diagnostics
                .diagnostics()
                .iter()
                .any(|diagnostic| diagnostic.kind == TaskGraphDiagnosticKind::DuplicateTaskId)
        );
    }

    #[test]
    fn cyclic_dependencies_are_rejected() {
        let main = module_id("app", "main");
        let util = module_id("app", "util");
        let diagnostics = build_task_graph(TaskGraphInput {
            graph_version: TaskGraphVersion::current(),
            snapshot: snapshot(7),
            build_plan: build_plan(vec![workspace_package("app")], Vec::new()),
            module_index: module_index(vec![
                workspace_module("app", "main"),
                workspace_module("app", "util"),
            ]),
            dependency_overlay: ModuleDependencyOverlay::complete(vec![
                ModuleDependencyEdge::new(
                    main.clone(),
                    util.clone(),
                    ModuleDependencyKind::ImportSummary,
                ),
                ModuleDependencyEdge::new(util, main, ModuleDependencyKind::ImportSummary),
            ]),
            vc_descriptors: Vec::new(),
            profile: TaskGraphProfile::default(),
        })
        .expect_err("dependency cycle is reported");

        assert!(
            diagnostics
                .diagnostics()
                .iter()
                .any(|diagnostic| diagnostic.kind == TaskGraphDiagnosticKind::DependencyCycle)
        );
    }

    #[test]
    fn structural_diagnostics_are_stable_and_cover_unknowns_and_self_edges() {
        let diagnostics = build_task_graph(TaskGraphInput {
            graph_version: TaskGraphVersion::current(),
            snapshot: snapshot(8),
            build_plan: build_plan(
                vec![workspace_package("app")],
                vec![DependencyEdge {
                    dependent: "app".to_owned(),
                    dependency: "missing_pkg".to_owned(),
                    kind: DependencyKind::Normal,
                }],
            ),
            module_index: module_index(vec![workspace_module("app", "main")]),
            dependency_overlay: ModuleDependencyOverlay::complete(vec![
                ModuleDependencyEdge::new(
                    module_id("app", "main"),
                    module_id("app", "main"),
                    ModuleDependencyKind::ImportSummary,
                ),
                ModuleDependencyEdge::new(
                    module_id("app", "main"),
                    module_id("app", "missing"),
                    ModuleDependencyKind::ImportSummary,
                ),
            ]),
            vc_descriptors: vec![vc_descriptor(
                "vc-missing",
                module_id("missing_pkg", "main"),
                "001",
                Vec::new(),
                Vec::new(),
            )],
            profile: TaskGraphProfile::default(),
        })
        .expect_err("structural diagnostics are reported");

        assert_eq!(
            diagnostics
                .diagnostics()
                .iter()
                .map(|diagnostic| diagnostic.kind)
                .collect::<Vec<_>>(),
            vec![
                TaskGraphDiagnosticKind::SelfDependency,
                TaskGraphDiagnosticKind::UnknownModule,
                TaskGraphDiagnosticKind::UnknownPackage,
                TaskGraphDiagnosticKind::UnknownModule,
            ]
        );
    }

    #[test]
    fn missing_overlay_diagnostics_sort_by_package_and_module() {
        let diagnostics = build_task_graph(TaskGraphInput {
            graph_version: TaskGraphVersion::current(),
            snapshot: snapshot(14),
            build_plan: build_plan(
                vec![workspace_package("zeta"), workspace_package("alpha")],
                Vec::new(),
            ),
            module_index: module_index(vec![
                workspace_module("zeta", "main"),
                workspace_module("alpha", "b"),
                workspace_module("alpha", "a"),
            ]),
            dependency_overlay: ModuleDependencyOverlay::unavailable(),
            vc_descriptors: Vec::new(),
            profile: TaskGraphProfile::default(),
        })
        .expect_err("missing overlay diagnostics are sorted");

        assert_eq!(
            diagnostics
                .diagnostics()
                .iter()
                .map(|diagnostic| diagnostic.module.as_ref().map(module_key))
                .collect::<Vec<_>>(),
            vec![
                Some("alpha:a".to_owned()),
                Some("alpha:b".to_owned()),
                Some("zeta:main".to_owned()),
            ]
        );
    }

    #[test]
    fn unsupported_schema_is_rejected_before_graph_construction() {
        let diagnostics = build_task_graph(TaskGraphInput {
            graph_version: TaskGraphVersion::new(TASK_GRAPH_SCHEMA_VERSION + 1),
            snapshot: snapshot(9),
            build_plan: build_plan(vec![workspace_package("app")], Vec::new()),
            module_index: module_index(vec![workspace_module("app", "main")]),
            dependency_overlay: ModuleDependencyOverlay::complete(Vec::new()),
            vc_descriptors: Vec::new(),
            profile: TaskGraphProfile::default(),
        })
        .expect_err("unsupported schema is reported");

        assert_eq!(
            diagnostics.diagnostics()[0].kind,
            TaskGraphDiagnosticKind::UnsupportedTaskGraphSchema
        );
    }

    #[test]
    fn cache_driver_ir_and_trust_placeholders_are_absent_from_boundary() {
        let manifest = include_str!("../Cargo.toml");
        assert!(!manifest.contains("mizar-cache"));
        assert!(!manifest.contains("mizar-driver"));
        assert!(!manifest.contains("mizar-ir"));

        let graph = graph_for_modules(vec![workspace_module("app", "main")], Vec::new());
        let graph_text = format!("{graph:#?}");
        for forbidden in [
            "cachekey",
            "dependencyfingerprint",
            "proofreuse",
            "proofauthority",
            "proofacceptance",
            "trustedstatus",
        ] {
            assert!(
                !graph_text.to_lowercase().contains(forbidden),
                "{forbidden} must not appear in task graph state"
            );
        }

        // Unsupported descriptor families are not expressible in the typed
        // descriptor input; adding one requires extending this public model,
        // not silently carrying an opaque placeholder.
    }

    fn graph_for_modules(
        modules: Vec<ModuleIndexEntry>,
        descriptors: Vec<VcTaskDescriptor>,
    ) -> TaskGraph {
        graph_for_modules_with_snapshot(snapshot(0), modules, descriptors)
    }

    fn graph_for_modules_with_snapshot(
        snapshot: BuildSnapshotId,
        modules: Vec<ModuleIndexEntry>,
        descriptors: Vec<VcTaskDescriptor>,
    ) -> TaskGraph {
        let packages = modules
            .iter()
            .map(|entry| entry.module.package.as_str())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .map(workspace_package)
            .collect();
        build_task_graph(TaskGraphInput {
            graph_version: TaskGraphVersion::current(),
            snapshot,
            build_plan: build_plan(packages, Vec::new()),
            module_index: module_index(modules),
            dependency_overlay: ModuleDependencyOverlay::complete(Vec::new()),
            vc_descriptors: descriptors,
            profile: TaskGraphProfile::default(),
        })
        .expect("task graph builds")
    }

    fn build_plan(packages: Vec<PackagePlan>, edges: Vec<DependencyEdge>) -> BuildPlan {
        BuildPlan {
            workspace_root: WorkspaceRoot::new("."),
            packages,
            dependency_graph: DependencyGraph { edges },
            lockfile: Lockfile {
                schema_version: 1,
                packages: Vec::new(),
            },
            toolchain: ToolchainInfo::new("test"),
            verifier_config: WorkspaceVerifierConfig {
                packages: Vec::new(),
            },
            build_config: WorkspaceBuildConfig {
                packages: Vec::new(),
            },
        }
    }

    fn workspace_package(package_id: &str) -> PackagePlan {
        PackagePlan {
            package_id: PackageId::new(package_id),
            version: Version::new(1, 0, 0),
            source: PackagePlanSource::Workspace {
                root: package_id.to_owned(),
                source_root: format!("{package_id}/src"),
                manifest_path: format!("{package_id}/mizar.pkg"),
            },
            edition: Edition::new("2025"),
            dependencies: Vec::new(),
            verifier_config: VerifierConfig::default(),
            build_config: BuildConfig::default(),
        }
    }

    fn registry_package(package_id: &str) -> PackagePlan {
        PackagePlan {
            package_id: PackageId::new(package_id),
            version: Version::new(1, 0, 0),
            source: PackagePlanSource::Registry {
                registry: "default".to_owned(),
                checksum: format!("sha256:{package_id}"),
            },
            edition: Edition::new("2025"),
            dependencies: Vec::new(),
            verifier_config: VerifierConfig::default(),
            build_config: BuildConfig::default(),
        }
    }

    fn module_index(modules: Vec<ModuleIndexEntry>) -> ModuleIndex {
        let mut package_has_workspace = BTreeMap::new();
        for entry in &modules {
            let has_workspace = matches!(entry.location, ModuleIndexLocation::WorkspaceFile { .. });
            package_has_workspace
                .entry(entry.module.package.as_str().to_owned())
                .and_modify(|current| *current |= has_workspace)
                .or_insert(has_workspace);
        }

        let packages = package_has_workspace
            .into_iter()
            .map(|(package_id, has_workspace)| PackageIndexEntry {
                package_id: PackageId::new(package_id.clone()),
                version: Version::new(1, 0, 0),
                edition: Edition::new("2025"),
                source: if has_workspace {
                    PackageIndexSource::Workspace {
                        package_root: package_id.to_owned(),
                        source_root: format!("{package_id}/src"),
                        manifest_path: format!("{package_id}/mizar.pkg"),
                    }
                } else {
                    PackageIndexSource::RegistryArtifact {
                        registry: "default".to_owned(),
                        checksum: format!("sha256:{package_id}"),
                    }
                },
                dependencies: Vec::new(),
            })
            .collect();
        ModuleIndex {
            packages,
            namespace_bindings: Vec::new(),
            modules,
            dependency_summaries: Vec::new(),
        }
    }

    fn workspace_module(package_id: &str, module_path: &str) -> ModuleIndexEntry {
        let module = module_id(package_id, module_path);
        ModuleIndexEntry {
            module: module.clone(),
            package_id: module.package.clone(),
            module_path: module.path.clone(),
            location: ModuleIndexLocation::WorkspaceFile {
                source_root: format!("{package_id}/src"),
                normalized_path: format!("{package_id}/src/{module_path}.miz"),
                source_relative_path: format!("{module_path}.miz"),
            },
            edition: Edition::new("2025"),
        }
    }

    fn dependency_summary_module(package_id: &str, module_path: &str) -> ModuleIndexEntry {
        let module = module_id(package_id, module_path);
        ModuleIndexEntry {
            module: module.clone(),
            package_id: module.package.clone(),
            module_path: module.path.clone(),
            location: ModuleIndexLocation::DependencySummary {
                artifact: format!("{package_id}-{module_path}.summary"),
                content_hash: Hash::from_bytes([1; Hash::BYTE_LEN]),
            },
            edition: Edition::new("2025"),
        }
    }

    fn module_id(package_id: &str, module_path: &str) -> ModuleId {
        ModuleId::new(PackageId::new(package_id), ModulePath::new(module_path))
    }

    fn vc_descriptor(
        id: &str,
        module: ModuleId,
        order_key: &str,
        backend_profiles: Vec<&str>,
        evidence_candidates: Vec<&str>,
    ) -> VcTaskDescriptor {
        VcTaskDescriptor::new(
            VcTaskDescriptorId::new(id),
            module,
            VcOrderKey::new(order_key),
            backend_profiles
                .into_iter()
                .map(BackendProfileId::new)
                .collect(),
            evidence_candidates
                .into_iter()
                .map(EvidenceCandidateId::new)
                .collect(),
        )
    }

    fn snapshot(seed: u8) -> BuildSnapshotId {
        let hex = format!("{seed:02x}").repeat(Hash::BYTE_LEN);
        BuildSnapshotId::from_published_schema_str(&format!(
            "mizar-session-build-snapshot-v1:{hex}"
        ))
        .expect("valid snapshot id")
    }

    fn task_ids(graph: &TaskGraph) -> Vec<String> {
        graph
            .tasks
            .iter()
            .map(|task| task.id.as_str().to_owned())
            .collect()
    }

    fn module_task_kinds(graph: &TaskGraph, package_id: &str, module_path: &str) -> Vec<TaskKind> {
        graph
            .tasks
            .iter()
            .filter(|task| task_module_matches(task, package_id, module_path))
            .map(|task| task.kind)
            .collect()
    }

    fn module_task<'a>(
        graph: &'a TaskGraph,
        kind: TaskKind,
        package_id: &str,
        module_path: &str,
    ) -> &'a BuildTask {
        graph
            .tasks
            .iter()
            .find(|task| task.kind == kind && task_module_matches(task, package_id, module_path))
            .expect("module task exists")
    }

    fn task_module_matches(task: &BuildTask, package_id: &str, module_path: &str) -> bool {
        module_for_unit(&task.unit).is_some_and(|module| {
            module.package.as_str() == package_id && module.path.as_str() == module_path
        })
    }

    fn assert_has_edge(graph: &TaskGraph, dependent: &BuildTask, dependency: &BuildTask) {
        assert!(
            graph
                .edges
                .iter()
                .any(|edge| { edge.dependent == dependent.id && edge.dependency == dependency.id }),
            "expected {} to depend on {}",
            dependent.id.as_str(),
            dependency.id.as_str()
        );
    }

    fn assert_dependencies_match_edges(graph: &TaskGraph) {
        let mut dependencies_by_task: BTreeMap<TaskId, Vec<TaskId>> = BTreeMap::new();
        for edge in &graph.edges {
            dependencies_by_task
                .entry(edge.dependent.clone())
                .or_default()
                .push(edge.dependency.clone());
        }
        for dependencies in dependencies_by_task.values_mut() {
            dependencies.sort();
            dependencies.dedup();
        }
        for task in &graph.tasks {
            let expected = dependencies_by_task.remove(&task.id).unwrap_or_default();
            assert_eq!(task.dependencies, expected, "{}", task.id.as_str());
        }
    }

    fn assert_edges_are_sorted(graph: &TaskGraph) {
        let edges = graph
            .edges
            .iter()
            .map(|edge| {
                (
                    edge.dependent.as_str().to_owned(),
                    edge.dependency.as_str().to_owned(),
                )
            })
            .collect::<Vec<_>>();
        let mut sorted_edges = edges.clone();
        sorted_edges.sort();
        assert_eq!(edges, sorted_edges);
    }

    fn assert_task_metadata(
        task: &BuildTask,
        dependency_coverage: DependencyCoverage,
        resource_class: ResourceClass,
        priority_class: PriorityClass,
    ) {
        assert_eq!(task.dependency_coverage, dependency_coverage);
        assert_eq!(task.resource_class, resource_class);
        assert_eq!(task.priority_class, priority_class);
        assert!(!task.phases.is_empty());
    }

    fn single_task(graph: &TaskGraph, kind: TaskKind) -> &BuildTask {
        let mut tasks = graph.tasks.iter().filter(|task| task.kind == kind);
        let task = tasks.next().expect("task exists");
        assert!(tasks.next().is_none(), "expected only one {kind:?} task");
        task
    }

    fn module_label(task: &BuildTask) -> String {
        let module = module_for_unit(&task.unit).expect("module unit");
        format!("{}:{}", module.package.as_str(), module.path.as_str())
    }

    fn descriptor_label(task: &BuildTask) -> &str {
        match &task.unit {
            WorkUnit::Vc { descriptor, .. }
            | WorkUnit::BackendAttempt { descriptor, .. }
            | WorkUnit::EvidenceCandidate { descriptor, .. } => descriptor.as_str(),
            WorkUnit::Workspace | WorkUnit::Package { .. } | WorkUnit::Module { .. } => {
                panic!("expected descriptor work unit")
            }
        }
    }

    fn backend_label(task: &BuildTask) -> &str {
        let WorkUnit::BackendAttempt {
            backend_profile, ..
        } = &task.unit
        else {
            panic!("expected backend work unit");
        };
        backend_profile.as_str()
    }

    fn evidence_label(task: &BuildTask) -> &str {
        let WorkUnit::EvidenceCandidate {
            evidence_candidate, ..
        } = &task.unit
        else {
            panic!("expected evidence work unit");
        };
        evidence_candidate.as_str()
    }

    fn package_label(task: &BuildTask) -> &str {
        let WorkUnit::Package { package_id } = &task.unit else {
            panic!("expected package work unit");
        };
        package_id.as_str()
    }

    fn contains_forbidden_authority_term(value: &str) -> bool {
        let lower = value.to_lowercase();
        [
            "cachekey",
            "dependencyfingerprint",
            "proofreuse",
            "proofauthority",
            "proofacceptance",
            "trustedstatus",
        ]
        .iter()
        .any(|forbidden| lower.contains(forbidden))
    }
}
