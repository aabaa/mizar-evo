use std::{
    collections::{BTreeMap, BTreeSet},
    error::Error,
    fmt,
    sync::Arc,
};

use mizar_build::{
    cancel::CancellationToken,
    task_graph::{PipelinePhase, WorkUnit},
};
use mizar_diagnostics::sink::{DiagnosticBatch, DiagnosticSink};
use mizar_ir::{publisher::PhaseOutputPublisher, storage::AnyPhaseOutputRef};
use mizar_session::{BuildSnapshotId, Hash};

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
    pub identities: PhaseInputIdentities,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PhaseInputIdentities {
    pub input_hash: Hash,
    pub dependency_hashes: Vec<Hash>,
    pub parent_output_hashes: Vec<Hash>,
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

#[derive(Debug)]
pub struct PhaseExecutionContext {
    pub common: PhaseContext,
    pub cancellation: Option<CancellationToken>,
    pub diagnostics: Option<DiagnosticSink>,
    pub output_publisher: Option<PhaseOutputPublisher>,
}

#[derive(Debug, Default)]
pub struct PhaseExecutionResources {
    pub cancellation: Option<CancellationToken>,
    pub diagnostics: Option<DiagnosticSink>,
    pub output_publisher: Option<PhaseOutputPublisher>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PhaseResult {
    pub status: PhaseStatus,
    pub diagnostics: Vec<DiagnosticBatch>,
    pub output_refs: Vec<AnyPhaseOutputRef>,
    pub cache_observation: Option<PhaseCacheObservation>,
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
}

struct RegisteredPhaseService {
    descriptor: PhaseDescriptor,
    service: Arc<dyn PhaseService>,
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

pub trait PhaseService: Send + Sync {
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
    pub fn new(
        snapshot: BuildSnapshotId,
        work_unit: WorkUnit,
        identities: PhaseInputIdentities,
    ) -> Self {
        Self {
            snapshot,
            work_unit,
            identities,
        }
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

    fn execution_context(&self, resources: PhaseExecutionResources) -> PhaseExecutionContext {
        PhaseExecutionContext {
            common: PhaseContext {
                snapshot: self.snapshot,
                work_unit: self.work_unit.clone(),
            },
            cancellation: resources.cancellation,
            diagnostics: resources.diagnostics,
            output_publisher: resources.output_publisher,
        }
    }
}

impl PhaseInputIdentities {
    pub fn new(
        input_hash: Hash,
        mut dependency_hashes: Vec<Hash>,
        mut parent_output_hashes: Vec<Hash>,
    ) -> Self {
        dependency_hashes.sort_by(|left, right| left.as_bytes().cmp(right.as_bytes()));
        parent_output_hashes.sort_by(|left, right| left.as_bytes().cmp(right.as_bytes()));
        Self {
            input_hash,
            dependency_hashes,
            parent_output_hashes,
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
        })
    }
}

impl PhaseRegistry {
    pub fn empty() -> Self {
        Self {
            services: Vec::new(),
            phase_index: BTreeMap::new(),
            query_boundary: DriverQueryBoundary::default(),
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

    fn service_for_phase(
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

#[derive(Debug, Clone, Copy)]
enum QueryAdapterKind {
    CacheKey,
    Execute,
}

const PHASE_REQUIREMENTS: [PhaseRequirement; 10] = [
    PhaseRequirement {
        service_name: "WorkspacePlanner",
        owner: PhaseOwner::MizarBuild,
        phases: &[PipelinePhase::PackageResolve],
        availability: PhaseServiceAvailability::AvailableOwner,
    },
    PhaseRequirement {
        service_name: "SourceFrontend",
        owner: PhaseOwner::MizarFrontend,
        phases: &[PipelinePhase::SourceLoad, PipelinePhase::Frontend],
        availability: PhaseServiceAvailability::ExternalDependencyGap,
    },
    PhaseRequirement {
        service_name: "ModuleResolver",
        owner: PhaseOwner::MizarResolve,
        phases: &[
            PipelinePhase::ModuleResolve,
            PipelinePhase::SignatureCollection,
        ],
        availability: PhaseServiceAvailability::ExternalDependencyGap,
    },
    PhaseRequirement {
        service_name: "SemanticChecker",
        owner: PhaseOwner::MizarChecker,
        phases: &[
            PipelinePhase::TypeChecking,
            PipelinePhase::RegistrationResolution,
            PipelinePhase::OverloadResolution,
        ],
        availability: PhaseServiceAvailability::ExternalDependencyGap,
    },
    PhaseRequirement {
        service_name: "Elaborator",
        owner: PhaseOwner::MizarCore,
        phases: &[
            PipelinePhase::Elaboration,
            PipelinePhase::AlgorithmPreparation,
        ],
        availability: PhaseServiceAvailability::ExternalDependencyGap,
    },
    PhaseRequirement {
        service_name: "VcService",
        owner: PhaseOwner::MizarVc,
        phases: &[PipelinePhase::VcGenerate, PipelinePhase::VcDischarge],
        availability: PhaseServiceAvailability::ExternalDependencyGap,
    },
    PhaseRequirement {
        service_name: "AtpService",
        owner: PhaseOwner::MizarAtp,
        phases: &[PipelinePhase::AtpSolve, PipelinePhase::BackendRun],
        availability: PhaseServiceAvailability::ExternalDependencyGap,
    },
    PhaseRequirement {
        service_name: "KernelService",
        owner: PhaseOwner::MizarKernelProof,
        phases: &[PipelinePhase::KernelCheck],
        availability: PhaseServiceAvailability::ExternalDependencyGap,
    },
    PhaseRequirement {
        service_name: "ArtifactService",
        owner: PhaseOwner::MizarArtifact,
        phases: &[PipelinePhase::ArtifactCommit],
        availability: PhaseServiceAvailability::ExternalDependencyGap,
    },
    PhaseRequirement {
        service_name: "DocExtractionService",
        owner: PhaseOwner::MizarDoc,
        phases: &[PipelinePhase::DocumentationExtract],
        availability: PhaseServiceAvailability::Deferred,
    },
];

fn phases_are_contiguous(phases: &[PipelinePhase]) -> bool {
    let mut ranks = phases
        .iter()
        .copied()
        .map(phase_rank)
        .collect::<BTreeSet<_>>();
    let Some(first) = ranks.pop_first() else {
        return false;
    };
    let mut expected = first;
    for rank in ranks {
        expected += 1;
        if rank != expected {
            return false;
        }
    }
    true
}

fn owner_for_phase(phase: PipelinePhase) -> Option<PhaseOwner> {
    required_phase_services()
        .iter()
        .find(|requirement| requirement.phases.contains(&phase))
        .map(|requirement| requirement.owner)
}

fn availability_for_phase(phase: PipelinePhase) -> Option<PhaseServiceAvailability> {
    required_phase_services()
        .iter()
        .find(|requirement| requirement.phases.contains(&phase))
        .map(|requirement| requirement.availability)
}

fn phase_rank(phase: PipelinePhase) -> u8 {
    match phase {
        PipelinePhase::PackageResolve => 0,
        PipelinePhase::SourceLoad => 1,
        PipelinePhase::Frontend => 2,
        PipelinePhase::ModuleResolve => 3,
        PipelinePhase::SignatureCollection => 4,
        PipelinePhase::TypeChecking => 5,
        PipelinePhase::RegistrationResolution => 6,
        PipelinePhase::OverloadResolution => 7,
        PipelinePhase::Elaboration => 8,
        PipelinePhase::AlgorithmPreparation => 9,
        PipelinePhase::VcGenerate => 10,
        PipelinePhase::VcDischarge => 11,
        PipelinePhase::AtpSolve => 12,
        PipelinePhase::BackendRun => 13,
        PipelinePhase::KernelCheck => 14,
        PipelinePhase::ArtifactCommit => 15,
        PipelinePhase::DocumentationExtract => 16,
        _ => u8::MAX,
    }
}

fn registry_query_identity_fingerprint(
    descriptor: &PhaseDescriptor,
    input: &PhaseInput,
    adapter: QueryAdapterKind,
) -> u64 {
    let mut state = StableHasher::default();
    state.write_str(descriptor.service_name.as_str());
    state.write_str(descriptor.schema_version.as_str());
    state.write_str(descriptor.output_kind.as_str());
    state.write_usize(descriptor.phases.len());
    for phase in &descriptor.phases {
        state.write_u8(phase_rank(*phase));
    }
    let snapshot = input
        .snapshot
        .to_published_schema_string()
        .unwrap_or_else(|_| format!("{:?}", input.snapshot));
    state.write_str(snapshot.as_str());
    state.write_str(format!("{:?}", input.work_unit).as_str());
    state.write_hash(input.identities.input_hash);
    state.write_usize(input.identities.dependency_hashes.len());
    for hash in &input.identities.dependency_hashes {
        state.write_hash(*hash);
    }
    state.write_usize(input.identities.parent_output_hashes.len());
    for hash in &input.identities.parent_output_hashes {
        state.write_hash(*hash);
    }
    state.write_u8(match adapter {
        QueryAdapterKind::CacheKey => 1,
        QueryAdapterKind::Execute => 2,
    });
    state.finish()
}

#[derive(Default)]
struct StableHasher {
    value: u64,
}

impl StableHasher {
    fn write_u8(&mut self, value: u8) {
        self.value = self.value.wrapping_mul(16_777_619) ^ u64::from(value);
    }

    fn write_usize(&mut self, value: usize) {
        self.write_u64(value as u64);
    }

    fn write_u64(&mut self, value: u64) {
        for byte in value.to_le_bytes() {
            self.write_u8(byte);
        }
    }

    fn write_hash(&mut self, hash: Hash) {
        for byte in hash.as_bytes() {
            self.write_u8(*byte);
        }
    }

    fn write_str(&mut self, value: &str) {
        for byte in value.as_bytes() {
            self.write_u8(*byte);
        }
        self.write_u8(0xff);
    }

    const fn finish(self) -> u64 {
        self.value
    }
}
