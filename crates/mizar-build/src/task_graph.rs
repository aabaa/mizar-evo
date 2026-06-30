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
#[non_exhaustive]
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
#[non_exhaustive]
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
#[non_exhaustive]
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
#[non_exhaustive]
pub enum DependencyCoverage {
    Complete,
    PackageConservative,
    MissingModuleDependencyOverlay,
    MissingVcDescriptors,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
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
#[non_exhaustive]
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
#[non_exhaustive]
pub enum ModuleDependencyCoverage {
    Complete,
    CoveredModules(Vec<ModuleId>),
    PackageOnly,
    Unavailable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
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
#[non_exhaustive]
pub enum DocumentationProfile {
    Disabled,
    Enabled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
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
mod tests;
