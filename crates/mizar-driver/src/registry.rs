use std::{collections::BTreeMap, error::Error, fmt, sync::Arc};

use mizar_build::{
    cancel::CancellationToken,
    module_index::ModuleIndex,
    planner::BuildPlan,
    task_graph::{BuildTask, PipelinePhase, TaskGraph, TaskId, WorkUnit},
};
use mizar_diagnostics::sink::{DiagnosticBatch, DiagnosticSink};
use mizar_ir::{
    dispatch_input::{PhaseDispatchInputBundle, PhaseInputIdentities, SealedParentOutputHandle},
    publisher::PhaseOutputPublisher,
    storage::AnyPhaseOutputRef,
};
use mizar_session::{BuildSnapshot, BuildSnapshotId, Hash, PackageId, SessionIdAllocator};
use std::path::PathBuf;

mod catalog;

use catalog::{
    PHASE_REQUIREMENTS, QueryAdapterKind, availability_for_phase, owner_for_phase, phase_rank,
    phases_are_contiguous, registry_query_identity_fingerprint,
};

/// Driver-owned composition settings for the existing source pipeline adapters.
#[derive(Debug, Clone)]
pub struct StageConfig {
    pub through: PipelinePhase,
    pub artifact_roots: Vec<(PackageId, PathBuf)>,
    pub generation_schema: mizar_vc::vc_ir::GenerationSchemaVersion,
    pub vc_schema: mizar_vc::vc_ir::VcSchemaVersion,
    pub discharge_policy: mizar_vc::discharge::DischargePolicy,
    pub logic_profile: mizar_atp::problem::LogicProfile,
    pub concrete_format: mizar_atp::problem::ConcreteFormat,
    pub verifier_policy: mizar_proof::policy::VerifierPolicy,
    pub witness_path_prefix: String,
    pub main_path: String,
    pub output_parent: String,
}

impl StageConfig {
    pub fn frontend(artifact_roots: Vec<(PackageId, PathBuf)>) -> Self {
        Self {
            through: PipelinePhase::Frontend,
            ..Self::source_theorem(artifact_roots)
        }
    }

    pub fn source_theorem(artifact_roots: Vec<(PackageId, PathBuf)>) -> Self {
        use mizar_atp::problem::{
            ConcreteFormat, EqualitySupport, LogicFragment, LogicProfile, NativePropertySupport,
            QuantifierPolicy, SoftTypeStrategy,
        };
        Self {
            through: PipelinePhase::ArtifactCommit,
            artifact_roots,
            generation_schema: "snapshot-generation-v1".into(),
            vc_schema: "snapshot-vc-v1".into(),
            discharge_policy: Default::default(),
            logic_profile: LogicProfile::try_new(
                "snapshot-native",
                LogicFragment::Fof,
                EqualitySupport::Supported,
                QuantifierPolicy::FirstOrder,
                SoftTypeStrategy::ScopedGuardPredicates,
                NativePropertySupport::Supported,
                [ConcreteFormat::Tptp].into_iter().collect(),
            )
            .expect("fixed source theorem profile is valid"),
            concrete_format: ConcreteFormat::Tptp,
            verifier_policy: Default::default(),
            witness_path_prefix: "proof-witnesses/snapshot".into(),
            main_path: "main.mizir.json".into(),
            output_parent: "artifacts".into(),
        }
    }
}

/// Assemble only the existing source adapters up to the selected stage.
pub fn assemble_registry(config: &StageConfig) -> Result<PhaseRegistry, String> {
    use PipelinePhase::*;
    let stages = [
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
        KernelCheck,
        ArtifactCommit,
    ];
    let cutoff = stages
        .iter()
        .position(|phase| *phase == config.through)
        .ok_or_else(|| format!("unsupported source pipeline cutoff: {:?}", config.through))?;
    let mut builder = PhaseRegistryBuilder::new();
    for stage in &stages[..=cutoff] {
        match stage {
            SourceLoad => builder.register_source_load(),
            Frontend => builder.register_frontend(config.artifact_roots.clone()),
            ModuleResolve => builder.register_source_theorem_module_resolver(),
            SignatureCollection => builder.register_source_theorem_signature_collection(),
            TypeChecking => builder.register_source_theorem_type_checking(),
            RegistrationResolution => builder.register_source_theorem_registration_resolution(),
            OverloadResolution => builder.register_source_theorem_overload_resolution(),
            Elaboration => builder.register_source_theorem_elaboration(),
            AlgorithmPreparation => builder.register_source_theorem_algorithm_preparation(),
            VcGenerate => builder.register_source_theorem_vc_generation(
                config.generation_schema.clone(),
                config.vc_schema.clone(),
            ),
            VcDischarge => {
                builder.register_source_theorem_vc_discharge(config.discharge_policy.clone())
            }
            AtpSolve => builder.register_source_theorem_atp_solve(
                config.discharge_policy.clone(),
                config.logic_profile.clone(),
                config.concrete_format,
            )?,
            KernelCheck => builder.register_source_theorem_candidate_check_with_witness_files(
                config.discharge_policy.clone(),
                config.logic_profile.clone(),
                config.concrete_format,
                config.verifier_policy.clone(),
                config.witness_path_prefix.clone(),
            )?,
            ArtifactCommit => builder.register_source_theorem_artifact_commit(
                config.discharge_policy.clone(),
                config.logic_profile.clone(),
                config.concrete_format,
                config.verifier_policy.clone(),
                (
                    config.witness_path_prefix.clone(),
                    config.main_path.clone(),
                    config.output_parent.clone(),
                ),
            )?,
            _ => unreachable!("source adapter stage list is fixed"),
        };
    }
    let mut registry = builder.build().map_err(|error| error.to_string())?;
    if let Ok(image) = crate::frontend_adapter::SourceLoadService::running_native_image() {
        registry.native_source_bootstrap = Some((
            config.verifier_policy.clone(),
            image.clone(),
            registry.services.clone(),
        ));
    }
    Ok(registry)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PhaseDescriptor {
    pub service_name: String,
    pub owner: PhaseOwner,
    pub phases: Vec<PipelinePhase>,
    pub schema_version: String,
    pub output_kind: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PhaseRequirement {
    pub service_name: &'static str,
    pub owner: PhaseOwner,
    pub phases: &'static [PipelinePhase],
    pub availability: PhaseServiceAvailability,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PhaseInput {
    pub snapshot: BuildSnapshotId,
    pub work_unit: WorkUnit,
    identities: PhaseInputIdentities,
    parent_outputs: Vec<SealedParentOutputHandle>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PhaseContext {
    pub snapshot: BuildSnapshotId,
    pub work_unit: WorkUnit,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PhaseCacheContext {
    pub common: PhaseContext,
    pub input_identities: PhaseInputIdentities,
}

#[derive(Clone, Copy)]
pub struct SourceLoadInputs<'a> {
    pub snapshot: &'a BuildSnapshot,
    pub captured: Option<&'a crate::request::CapturedSnapshot>,
    pub build_plan: &'a BuildPlan,
    pub module_index: &'a ModuleIndex,
    pub allocator: &'a dyn SessionIdAllocator,
}

impl fmt::Debug for SourceLoadInputs<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("SourceLoadInputs")
            .field("snapshot", &self.snapshot.id)
            .field("build_plan", &self.build_plan.workspace_root)
            .field("module_index", &self.module_index.modules.len())
            .finish_non_exhaustive()
    }
}

#[derive(Debug)]
pub struct PhaseExecutionContext<'a> {
    pub common: PhaseContext,
    pub cancellation: Option<CancellationToken>,
    pub diagnostics: Option<DiagnosticSink>,
    pub output_publisher: Option<Arc<PhaseOutputPublisher>>,
    pub parent_outputs: Vec<SealedParentOutputHandle>,
    pub source_load: Option<SourceLoadInputs<'a>>,
}

#[derive(Debug, Default)]
pub struct PhaseExecutionResources<'a> {
    pub cancellation: Option<CancellationToken>,
    pub diagnostics: Option<DiagnosticSink>,
    pub output_publisher: Option<Arc<PhaseOutputPublisher>>,
    pub source_load: Option<SourceLoadInputs<'a>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PhaseResult {
    pub status: PhaseStatus,
    pub diagnostics: Vec<DiagnosticBatch>,
    pub output_refs: Vec<AnyPhaseOutputRef>,
    pub cache_observation: Option<PhaseCacheObservation>,
    pub artifact_commit: Option<mizar_artifact::manifest::ManifestCommit>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PhaseCacheObservation {
    pub query_fingerprint: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PhaseCacheQueryResult {
    pub observation: PhaseCacheObservation,
    pub intent: PhaseCacheIntent,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PhaseExecutionQueryResult {
    pub observation: PhaseCacheObservation,
    pub result: PhaseResult,
}

#[derive(Default)]
pub struct PhaseRegistryBuilder {
    services: Vec<Arc<dyn PhaseService>>,
}

pub struct PhaseRegistry {
    services: Vec<RegisteredPhaseService>,
    phase_index: BTreeMap<PipelinePhase, usize>,
    query_boundary: DriverQueryBoundary,
    native_source_bootstrap: Option<(
        mizar_proof::policy::VerifierPolicy,
        mizar_cache::cache_key::CompatibilityField,
        Vec<RegisteredPhaseService>,
    )>,
}

#[derive(Clone)]
struct RegisteredPhaseService {
    descriptor: PhaseDescriptor,
    service: Arc<dyn PhaseService>,
}

pub(crate) fn native_frontend_cache_parents(
    task: &BuildTask,
    graph: &TaskGraph,
    results: &BTreeMap<TaskId, Vec<PhaseResult>>,
) -> Option<(Vec<AnyPhaseOutputRef>, Vec<AnyPhaseOutputRef>)> {
    use mizar_build::task_graph::{TaskKind, WorkUnit};
    use mizar_session::module_work_unit_label;
    use std::collections::BTreeSet;
    if task.kind != TaskKind::Frontend
        || task.phases != [PipelinePhase::Frontend]
        || graph
            .tasks()
            .iter()
            .filter(|candidate| candidate.id == task.id)
            .count()
            != 1
        || graph
            .tasks()
            .iter()
            .find(|candidate| candidate.id == task.id)?
            != task
    {
        return None;
    }
    let mut roots = graph.tasks().iter().filter(|candidate| {
        candidate.kind == TaskKind::PackageResolve
            && candidate.unit == WorkUnit::Workspace
            && candidate.phases == [PipelinePhase::PackageResolve]
    });
    let root = roots.next()?;
    if roots.next().is_some()
        || !root.dependencies.is_empty()
        || !task.dependencies.contains(&root.id)
    {
        return None;
    }
    let output = |parent: &BuildTask, kind: &str| {
        let [result] = results.get(&parent.id)?.as_slice() else {
            return None;
        };
        let [output] = result.output_refs.as_slice() else {
            return None;
        };
        let WorkUnit::Module { module } = &parent.unit else {
            return None;
        };
        (result.status == PhaseStatus::Complete
            && result.diagnostics.is_empty()
            && output.snapshot() == graph.snapshot
            && output.phase() == &mizar_ir::identity::PipelinePhase::from(parent.phases[0])
            && output.output_kind() == &mizar_ir::identity::OutputKind::new(kind)
            && output.work_unit().as_str()
                == module_work_unit_label(module.package.as_str(), module.path.as_str()))
        .then(|| output.clone())
    };
    let mut parents = Vec::new();
    let mut leaf_sources = Vec::new();
    let mut own_source = false;
    let mut seen = BTreeSet::new();
    let mut leaf_units = Vec::new();
    for id in &task.dependencies {
        if !seen.insert(id) {
            return None;
        }
        if id == &root.id {
            continue;
        }
        let mut matches = graph.tasks().iter().filter(|candidate| &candidate.id == id);
        let parent = matches.next()?;
        if matches.next().is_some() {
            return None;
        }
        if parent.kind == TaskKind::SourceLoad
            && parent.phases == [PipelinePhase::SourceLoad]
            && parent.unit == task.unit
            && !own_source
            && parent.dependencies == [root.id.clone()]
        {
            own_source = true;
            parents.push(output(parent, "SourceUnit")?);
        } else if parent.kind == TaskKind::Frontend
            && parent.phases == [PipelinePhase::Frontend]
            && parent.unit != task.unit
            && !leaf_units.contains(&&parent.unit)
        {
            leaf_units.push(&parent.unit);
            let mut sources = graph.tasks().iter().filter(|candidate| {
                candidate.unit == parent.unit && candidate.phases == [PipelinePhase::SourceLoad]
            });
            let source = sources.next()?;
            if source.kind != TaskKind::SourceLoad
                || sources.next().is_some()
                || source.dependencies != [root.id.clone()]
                || !(2..=3).contains(&parent.dependencies.len())
                || !parent.dependencies.contains(&root.id)
                || !parent.dependencies.contains(&source.id)
            {
                return None;
            }
            let leaf = output(parent, "FrontendOutput")?;
            let source_output = output(source, "SourceUnit")?;
            if parent.dependencies.len() == 3 {
                if task.dependencies.len() != 3 || !leaf_sources.is_empty() {
                    return None;
                }
                let mut ancestors = parent
                    .dependencies
                    .iter()
                    .filter(|id| **id != root.id && **id != source.id);
                let ancestor_id = ancestors.next()?;
                if ancestors.next().is_some() {
                    return None;
                }
                let ancestor = graph
                    .tasks()
                    .iter()
                    .find(|candidate| &candidate.id == ancestor_id)?;
                let WorkUnit::Module { module: own_module } = &task.unit else {
                    return None;
                };
                let WorkUnit::Module {
                    module: middle_module,
                } = &parent.unit
                else {
                    return None;
                };
                let WorkUnit::Module {
                    module: ancestor_module,
                } = &ancestor.unit
                else {
                    return None;
                };
                if ancestor.kind != TaskKind::Frontend
                    || ancestor.phases != [PipelinePhase::Frontend]
                    || ancestor.unit == task.unit
                    || ancestor.unit == parent.unit
                    || own_module.package != middle_module.package
                    || own_module.package != ancestor_module.package
                {
                    return None;
                }
                let mut ancestor_sources = graph.tasks().iter().filter(|candidate| {
                    candidate.unit == ancestor.unit
                        && candidate.phases == [PipelinePhase::SourceLoad]
                });
                let ancestor_source = ancestor_sources.next()?;
                if ancestor_sources.next().is_some()
                    || ancestor_source.kind != TaskKind::SourceLoad
                    || ancestor_source.dependencies != [root.id.clone()]
                    || ancestor.dependencies.len() != 2
                    || !ancestor.dependencies.contains(&root.id)
                    || !ancestor.dependencies.contains(&ancestor_source.id)
                    || graph
                        .tasks()
                        .iter()
                        .filter(|candidate| candidate.id == ancestor.id)
                        .count()
                        != 1
                {
                    return None;
                }
                let ancestor_output = output(ancestor, "FrontendOutput")?;
                let ancestor_source_output = output(ancestor_source, "SourceUnit")?;
                if ancestor_output.lineage().parents != [ancestor_source_output.output()]
                    || leaf.lineage().parents.len() != 2
                    || !leaf.lineage().parents.contains(&source_output.output())
                    || !leaf.lineage().parents.contains(&ancestor_output.output())
                {
                    return None;
                }
                leaf_sources.extend([ancestor_output, ancestor_source_output]);
            } else if leaf.lineage().parents != [source_output.output()] {
                return None;
            }
            leaf_sources.push(source_output);
            parents.push(leaf);
        } else {
            return None;
        }
    }
    (own_source
        && (parents.len() == 1 + leaf_sources.len()
            || (parents.len() == 2 && leaf_sources.len() == 3)))
        .then_some((parents, leaf_sources))
}

#[derive(Clone, Default)]
pub struct DriverQueryBoundary {
    database: DriverQueryDatabase,
}

#[salsa::db]
#[derive(Clone, Default)]
pub struct DriverQueryDatabase {
    storage: salsa::Storage<Self>,
}

#[salsa::input(debug)]
struct RegistryQueryIdentity {
    fingerprint: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum PhaseOwner {
    MizarBuild,
    MizarFrontend,
    MizarResolve,
    MizarChecker,
    MizarCore,
    MizarVc,
    MizarAtp,
    MizarKernelProof,
    MizarArtifact,
    MizarDoc,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum PhaseServiceAvailability {
    AvailableOwner,
    ExternalDependencyGap,
    Deferred,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum PhaseStatus {
    Complete,
    Recoverable,
    Blocking,
    Fatal,
    Cancelled,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum PhaseCacheIntent {
    Cacheable { intent_hash: Hash },
    Uncacheable { reason: String },
    NoKey { reason: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum PhaseRegistryError {
    EmptyPhaseSpan {
        service_name: String,
    },
    NonContiguousPhaseSpan {
        service_name: String,
        phases: Vec<PipelinePhase>,
    },
    OwnerMismatch {
        service_name: String,
        phase: PipelinePhase,
        expected: PhaseOwner,
        actual: PhaseOwner,
    },
    UnknownPipelinePhase {
        phase: PipelinePhase,
    },
    DuplicatePhase {
        phase: PipelinePhase,
        first_service: String,
        duplicate_service: String,
    },
    MissingPhaseService {
        phase: PipelinePhase,
        availability: PhaseServiceAvailability,
    },
}

pub trait PhaseService: Send + Sync + std::any::Any {
    fn phase(&self) -> PhaseDescriptor;

    fn cache_key(&self, input: &PhaseInput, context: &PhaseCacheContext) -> PhaseCacheIntent;

    fn execute(&self, input: PhaseInput, context: PhaseExecutionContext) -> PhaseResult;
}

#[salsa::db]
trait DriverRegistryDb: salsa::Database {}

#[salsa::db]
impl salsa::Database for DriverQueryDatabase {}

#[salsa::db]
impl DriverRegistryDb for DriverQueryDatabase {}

#[salsa::tracked]
fn registry_query_fingerprint(db: &dyn DriverRegistryDb, identity: RegistryQueryIdentity) -> u64 {
    identity.fingerprint(db)
}

impl PhaseDescriptor {
    pub fn new(
        service_name: impl Into<String>,
        owner: PhaseOwner,
        phases: Vec<PipelinePhase>,
        schema_version: impl Into<String>,
        output_kind: impl Into<String>,
    ) -> Result<Self, PhaseRegistryError> {
        let mut descriptor = Self {
            service_name: service_name.into(),
            owner,
            phases,
            schema_version: schema_version.into(),
            output_kind: output_kind.into(),
        };
        descriptor.normalize_phases();
        descriptor.validate()?;
        Ok(descriptor)
    }

    fn normalize_phases(&mut self) {
        self.phases.sort_by_key(|phase| phase_rank(*phase));
    }

    fn validate(&self) -> Result<(), PhaseRegistryError> {
        if self.phases.is_empty() {
            return Err(PhaseRegistryError::EmptyPhaseSpan {
                service_name: self.service_name.clone(),
            });
        }
        if !phases_are_contiguous(&self.phases) {
            return Err(PhaseRegistryError::NonContiguousPhaseSpan {
                service_name: self.service_name.clone(),
                phases: self.phases.clone(),
            });
        }
        for phase in &self.phases {
            let Some(expected) = owner_for_phase(*phase) else {
                return Err(PhaseRegistryError::UnknownPipelinePhase { phase: *phase });
            };
            if expected != self.owner {
                return Err(PhaseRegistryError::OwnerMismatch {
                    service_name: self.service_name.clone(),
                    phase: *phase,
                    expected,
                    actual: self.owner,
                });
            }
        }
        Ok(())
    }

    fn registration_key(&self) -> (u8, usize, &str, &str) {
        (
            phase_rank(self.phases[0]),
            self.phases.len(),
            self.service_name.as_str(),
            self.schema_version.as_str(),
        )
    }
}

impl PhaseInput {
    pub fn new(work_unit: WorkUnit, bundle: PhaseDispatchInputBundle) -> Self {
        let (snapshot, identities, parent_outputs) = bundle.into_parts();
        Self {
            snapshot,
            work_unit,
            identities,
            parent_outputs,
        }
    }

    pub fn identities(&self) -> &PhaseInputIdentities {
        &self.identities
    }

    pub fn parent_outputs(&self) -> &[SealedParentOutputHandle] {
        &self.parent_outputs
    }

    fn cache_context(&self) -> PhaseCacheContext {
        PhaseCacheContext {
            common: PhaseContext {
                snapshot: self.snapshot,
                work_unit: self.work_unit.clone(),
            },
            input_identities: self.identities.clone(),
        }
    }

    fn execution_context<'a>(
        &self,
        resources: PhaseExecutionResources<'a>,
    ) -> PhaseExecutionContext<'a> {
        PhaseExecutionContext {
            common: PhaseContext {
                snapshot: self.snapshot,
                work_unit: self.work_unit.clone(),
            },
            cancellation: resources.cancellation,
            diagnostics: resources.diagnostics,
            output_publisher: resources.output_publisher,
            parent_outputs: self.parent_outputs.clone(),
            source_load: resources.source_load,
        }
    }
}

impl PhaseResult {
    pub fn complete() -> Self {
        Self {
            status: PhaseStatus::Complete,
            diagnostics: Vec::new(),
            output_refs: Vec::new(),
            cache_observation: None,
            artifact_commit: None,
        }
    }

    fn with_cache_observation(mut self, observation: PhaseCacheObservation) -> Self {
        self.cache_observation = Some(observation);
        self
    }
}

impl PhaseRegistryBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register<S>(&mut self, service: S) -> &mut Self
    where
        S: PhaseService + 'static,
    {
        self.services.push(Arc::new(service));
        self
    }

    pub fn register_arc(&mut self, service: Arc<dyn PhaseService>) -> &mut Self {
        self.services.push(service);
        self
    }

    pub fn register_source_load(&mut self) -> &mut Self {
        self.register(crate::frontend_adapter::SourceLoadService)
    }

    pub fn register_frontend(&mut self, artifact_roots: Vec<(PackageId, PathBuf)>) -> &mut Self {
        self.register(crate::frontend_adapter::FrontendService { artifact_roots })
    }

    pub fn register_source_theorem_module_resolver(&mut self) -> &mut Self {
        self.register(crate::frontend_adapter::SourceTheoremModuleResolverService)
    }

    pub fn register_source_theorem_signature_collection(&mut self) -> &mut Self {
        self.register(crate::frontend_adapter::SourceTheoremSignatureCollectionService)
    }

    pub fn register_source_theorem_type_checking(&mut self) -> &mut Self {
        self.register(crate::frontend_adapter::SourceTheoremTypeCheckingService)
    }

    pub fn register_source_theorem_registration_resolution(&mut self) -> &mut Self {
        self.register(crate::frontend_adapter::SourceTheoremRegistrationResolutionService)
    }

    pub fn register_source_theorem_overload_resolution(&mut self) -> &mut Self {
        self.register(crate::frontend_adapter::SourceTheoremOverloadResolutionService)
    }

    pub fn register_source_theorem_elaboration(&mut self) -> &mut Self {
        self.register(crate::frontend_adapter::SourceTheoremElaborationService)
    }

    pub fn register_source_theorem_algorithm_preparation(&mut self) -> &mut Self {
        self.register(crate::frontend_adapter::SourceTheoremAlgorithmPreparationService)
    }

    pub fn register_source_theorem_vc_generation(
        &mut self,
        generation_schema: mizar_vc::vc_ir::GenerationSchemaVersion,
        vc_schema: mizar_vc::vc_ir::VcSchemaVersion,
    ) -> &mut Self {
        self.register(crate::frontend_adapter::SourceTheoremVcGenerationService {
            generation_schema,
            vc_schema,
        })
    }

    pub fn register_source_theorem_reflexivity_kernel_check(
        &mut self,
        verifier_policy: mizar_proof::policy::VerifierPolicy,
    ) -> &mut Self {
        self.register(
            crate::frontend_adapter::SourceTheoremReflexivityKernelCheckService { verifier_policy },
        )
    }

    pub fn register_source_theorem_reflexivity_artifact_commit(
        &mut self,
        verifier_policy: mizar_proof::policy::VerifierPolicy,
        staging: (String, String, String),
    ) -> Result<&mut Self, String> {
        if (staging.0 != "proof-witnesses" && !staging.0.starts_with("proof-witnesses/"))
            || mizar_artifact::store::PublishedArtifactPath::new(format!(
                "{}/candidate.json",
                staging.0
            ))
            .is_err()
        {
            return Err("unsupported source theorem witness path prefix".to_owned());
        }
        if !staging.1.ends_with(".mizir.json")
            || mizar_artifact::store::PublishedArtifactPath::new(&staging.1).is_err()
        {
            return Err("unsupported source theorem artifact main path".to_owned());
        }
        if mizar_artifact::store::PublishedArtifactPath::new(&staging.2).is_err() {
            return Err("unsupported source theorem artifact output parent".to_owned());
        }
        Ok(self.register(
            crate::frontend_adapter::SourceTheoremReflexivityArtifactCommitService {
                verifier_policy,
                witness_path_prefix: staging.0,
                main_path: staging.1,
                output_parent: staging.2,
            },
        ))
    }

    pub fn register_source_theorem_vc_discharge(
        &mut self,
        policy: mizar_vc::discharge::DischargePolicy,
    ) -> &mut Self {
        self.register(crate::frontend_adapter::SourceTheoremVcDischargeService { policy })
    }

    pub fn register_source_theorem_atp_solve(
        &mut self,
        policy: mizar_vc::discharge::DischargePolicy,
        logic_profile: mizar_atp::problem::LogicProfile,
        concrete_format: mizar_atp::problem::ConcreteFormat,
    ) -> Result<&mut Self, String> {
        let service = crate::frontend_adapter::SourceTheoremAtpSolveService {
            policy,
            logic_profile,
            concrete_format,
        };
        if service.configuration_tags().is_none() {
            return Err("unsupported source theorem ATP transport configuration".to_owned());
        }
        Ok(self.register(service))
    }

    pub fn register_source_theorem_candidate_check(
        &mut self,
        discharge_policy: mizar_vc::discharge::DischargePolicy,
        logic_profile: mizar_atp::problem::LogicProfile,
        concrete_format: mizar_atp::problem::ConcreteFormat,
        verifier_policy: mizar_proof::policy::VerifierPolicy,
    ) -> Result<&mut Self, String> {
        let candidate = crate::frontend_adapter::SourceTheoremAtpSolveService {
            policy: discharge_policy,
            logic_profile,
            concrete_format,
        };
        if candidate.configuration_tags().is_none() {
            return Err(
                "unsupported source theorem candidate check transport configuration".to_owned(),
            );
        }
        Ok(self.register(
            crate::frontend_adapter::SourceTheoremCandidateCheckService {
                candidate,
                verifier_policy,
                witness_path_prefix: None,
            },
        ))
    }

    pub fn register_source_theorem_candidate_check_with_witness_files(
        &mut self,
        discharge_policy: mizar_vc::discharge::DischargePolicy,
        logic_profile: mizar_atp::problem::LogicProfile,
        concrete_format: mizar_atp::problem::ConcreteFormat,
        verifier_policy: mizar_proof::policy::VerifierPolicy,
        witness_path_prefix: String,
    ) -> Result<&mut Self, String> {
        let candidate = crate::frontend_adapter::SourceTheoremAtpSolveService {
            policy: discharge_policy,
            logic_profile,
            concrete_format,
        };
        if candidate.configuration_tags().is_none() {
            return Err(
                "unsupported source theorem candidate check transport configuration".to_owned(),
            );
        }
        if (witness_path_prefix != "proof-witnesses"
            && !witness_path_prefix.starts_with("proof-witnesses/"))
            || mizar_artifact::store::PublishedArtifactPath::new(format!(
                "{witness_path_prefix}/candidate.json"
            ))
            .is_err()
        {
            return Err("unsupported source theorem witness path prefix".to_owned());
        }
        Ok(self.register(
            crate::frontend_adapter::SourceTheoremCandidateCheckService {
                candidate,
                verifier_policy,
                witness_path_prefix: Some(witness_path_prefix),
            },
        ))
    }

    pub fn register_source_theorem_artifact_commit(
        &mut self,
        discharge_policy: mizar_vc::discharge::DischargePolicy,
        logic_profile: mizar_atp::problem::LogicProfile,
        concrete_format: mizar_atp::problem::ConcreteFormat,
        verifier_policy: mizar_proof::policy::VerifierPolicy,
        staging: (String, String, String),
    ) -> Result<&mut Self, String> {
        let candidate = crate::frontend_adapter::SourceTheoremAtpSolveService {
            policy: discharge_policy,
            logic_profile,
            concrete_format,
        };
        if candidate.configuration_tags().is_none() {
            return Err(
                "unsupported source theorem artifact commit transport configuration".to_owned(),
            );
        }
        if (staging.0 != "proof-witnesses" && !staging.0.starts_with("proof-witnesses/"))
            || mizar_artifact::store::PublishedArtifactPath::new(format!(
                "{}/candidate.json",
                staging.0
            ))
            .is_err()
        {
            return Err("unsupported source theorem witness path prefix".to_owned());
        }
        if !staging.1.ends_with(".mizir.json")
            || mizar_artifact::store::PublishedArtifactPath::new(&staging.1).is_err()
        {
            return Err("unsupported source theorem artifact main path".to_owned());
        }
        if mizar_artifact::store::PublishedArtifactPath::new(&staging.2).is_err() {
            return Err("unsupported source theorem artifact output parent".to_owned());
        }
        Ok(self.register(
            crate::frontend_adapter::SourceTheoremArtifactCommitService {
                check: crate::frontend_adapter::SourceTheoremCandidateCheckService {
                    candidate,
                    verifier_policy,
                    witness_path_prefix: Some(staging.0),
                },
                main_path: staging.1,
                output_parent: staging.2,
            },
        ))
    }

    pub fn build(self) -> Result<PhaseRegistry, PhaseRegistryError> {
        let mut services = self
            .services
            .into_iter()
            .map(|service| {
                let mut descriptor = service.phase();
                descriptor.normalize_phases();
                descriptor.validate()?;
                Ok(RegisteredPhaseService {
                    descriptor,
                    service,
                })
            })
            .collect::<Result<Vec<_>, PhaseRegistryError>>()?;

        services.sort_by(|left, right| {
            left.descriptor
                .registration_key()
                .cmp(&right.descriptor.registration_key())
        });

        let mut phase_index = BTreeMap::new();
        for (index, registered) in services.iter().enumerate() {
            for phase in &registered.descriptor.phases {
                if let Some(first_index) = phase_index.insert(*phase, index) {
                    return Err(PhaseRegistryError::DuplicatePhase {
                        phase: *phase,
                        first_service: services[first_index].descriptor.service_name.clone(),
                        duplicate_service: registered.descriptor.service_name.clone(),
                    });
                }
            }
        }

        Ok(PhaseRegistry {
            services,
            phase_index,
            query_boundary: DriverQueryBoundary::default(),
            native_source_bootstrap: None,
        })
    }
}

impl PhaseRegistry {
    pub(crate) fn native_source_policy(
        &self,
        requested: &mizar_proof::policy::VerifierPolicy,
    ) -> Option<&mizar_proof::policy::VerifierPolicy> {
        let (policy, image, services) = self.native_source_bootstrap.as_ref()?;
        (policy.policy_fingerprint() == requested.policy_fingerprint()
            && services.len() == self.services.len()
            && services
                .iter()
                .zip(&self.services)
                .all(|(original, selected)| {
                    Arc::ptr_eq(&original.service, &selected.service)
                        && original.descriptor == selected.descriptor
                        && selected.service.phase() == original.descriptor
                        && original.descriptor.phases.iter().all(|phase| {
                            self.descriptor_for_phase(*phase).ok() == Some(&original.descriptor)
                        })
                })
            && crate::frontend_adapter::SourceLoadService::running_native_image().ok()? == image)
            .then_some(policy)
    }

    pub(crate) fn native_frontend_service(
        &self,
    ) -> Option<&crate::frontend_adapter::FrontendService> {
        let (policy, _, _) = self.native_source_bootstrap.as_ref()?;
        self.native_source_policy(policy)?;
        let descriptor = self.descriptor_for_phase(PipelinePhase::Frontend).ok()?;
        let service = self.service_for_phase(PipelinePhase::Frontend).ok()?;
        if service.phase() != *descriptor {
            return None;
        }
        (service as &dyn std::any::Any).downcast_ref()
    }

    pub fn empty() -> Self {
        Self {
            services: Vec::new(),
            phase_index: BTreeMap::new(),
            query_boundary: DriverQueryBoundary::default(),
            native_source_bootstrap: None,
        }
    }

    pub fn descriptors(&self) -> impl ExactSizeIterator<Item = &PhaseDescriptor> {
        self.services.iter().map(|service| &service.descriptor)
    }

    pub fn query_boundary(&self) -> &DriverQueryBoundary {
        &self.query_boundary
    }

    pub fn descriptor_for_phase(
        &self,
        phase: PipelinePhase,
    ) -> Result<&PhaseDescriptor, PhaseRegistryError> {
        let Some(index) = self.phase_index.get(&phase) else {
            let Some(availability) = availability_for_phase(phase) else {
                return Err(PhaseRegistryError::UnknownPipelinePhase { phase });
            };
            return Err(PhaseRegistryError::MissingPhaseService {
                phase,
                availability,
            });
        };
        Ok(&self.services[*index].descriptor)
    }

    pub fn cache_key_for_phase(
        &self,
        phase: PipelinePhase,
        input: &PhaseInput,
    ) -> Result<PhaseCacheQueryResult, PhaseRegistryError> {
        let descriptor = self.descriptor_for_phase(phase)?;
        let service = self.service_for_phase(phase)?;
        Ok(self.query_boundary.cache_key(descriptor, service, input))
    }

    pub fn execute_phase(
        &self,
        phase: PipelinePhase,
        input: PhaseInput,
        cancellation: Option<CancellationToken>,
    ) -> Result<PhaseExecutionQueryResult, PhaseRegistryError> {
        self.execute_phase_with_resources(
            phase,
            input,
            PhaseExecutionResources {
                cancellation,
                ..PhaseExecutionResources::default()
            },
        )
    }

    pub fn execute_phase_with_resources(
        &self,
        phase: PipelinePhase,
        input: PhaseInput,
        resources: PhaseExecutionResources,
    ) -> Result<PhaseExecutionQueryResult, PhaseRegistryError> {
        let descriptor = self.descriptor_for_phase(phase)?;
        let service = self.service_for_phase(phase)?;
        Ok(self
            .query_boundary
            .execute(descriptor, service, input, resources))
    }

    pub(crate) fn service_for_phase(
        &self,
        phase: PipelinePhase,
    ) -> Result<&dyn PhaseService, PhaseRegistryError> {
        let Some(index) = self.phase_index.get(&phase) else {
            let Some(availability) = availability_for_phase(phase) else {
                return Err(PhaseRegistryError::UnknownPipelinePhase { phase });
            };
            return Err(PhaseRegistryError::MissingPhaseService {
                phase,
                availability,
            });
        };
        Ok(self.services[*index].service.as_ref())
    }
}

impl DriverQueryBoundary {
    pub fn cache_key(
        &self,
        descriptor: &PhaseDescriptor,
        service: &dyn PhaseService,
        input: &PhaseInput,
    ) -> PhaseCacheQueryResult {
        let observation = self.observe(descriptor, input, QueryAdapterKind::CacheKey);
        let intent = service.cache_key(input, &input.cache_context());
        PhaseCacheQueryResult {
            observation,
            intent,
        }
    }

    pub fn execute(
        &self,
        descriptor: &PhaseDescriptor,
        service: &dyn PhaseService,
        input: PhaseInput,
        resources: PhaseExecutionResources,
    ) -> PhaseExecutionQueryResult {
        let observation = self.observe(descriptor, &input, QueryAdapterKind::Execute);
        let context = input.execution_context(resources);
        let result = service
            .execute(input, context)
            .with_cache_observation(observation);
        PhaseExecutionQueryResult {
            observation,
            result,
        }
    }

    fn observe(
        &self,
        descriptor: &PhaseDescriptor,
        input: &PhaseInput,
        adapter: QueryAdapterKind,
    ) -> PhaseCacheObservation {
        let identity = RegistryQueryIdentity::new(
            &self.database,
            registry_query_identity_fingerprint(descriptor, input, adapter),
        );
        PhaseCacheObservation {
            query_fingerprint: registry_query_fingerprint(&self.database, identity),
        }
    }
}

pub fn required_phase_services() -> &'static [PhaseRequirement] {
    &PHASE_REQUIREMENTS
}

impl fmt::Display for PhaseRegistryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyPhaseSpan { service_name } => {
                write!(f, "phase service {service_name:?} covers no phases")
            }
            Self::NonContiguousPhaseSpan {
                service_name,
                phases,
            } => write!(
                f,
                "phase service {service_name:?} has a non-contiguous phase span: {phases:?}"
            ),
            Self::OwnerMismatch {
                service_name,
                phase,
                expected,
                actual,
            } => write!(
                f,
                "phase service {service_name:?} claims owner {actual:?} for {phase:?}; expected {expected:?}"
            ),
            Self::DuplicatePhase {
                phase,
                first_service,
                duplicate_service,
            } => write!(
                f,
                "phase {phase:?} is registered by both {first_service:?} and {duplicate_service:?}"
            ),
            Self::UnknownPipelinePhase { phase } => {
                write!(f, "phase {phase:?} is not in the driver registry table")
            }
            Self::MissingPhaseService {
                phase,
                availability,
            } => write!(
                f,
                "phase {phase:?} has no registered service; availability is {availability:?}"
            ),
        }
    }
}

impl Error for PhaseRegistryError {}
