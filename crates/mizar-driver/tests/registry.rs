use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

use mizar_build::task_graph::{PipelinePhase, WorkUnit};
use mizar_driver::registry::{
    PhaseCacheContext, PhaseCacheIntent, PhaseDescriptor, PhaseExecutionContext, PhaseInput,
    PhaseInputIdentities, PhaseOwner, PhaseRegistry, PhaseRegistryBuilder, PhaseRegistryError,
    PhaseResult, PhaseService, PhaseServiceAvailability, required_phase_services,
};
use mizar_session::{BuildSnapshotId, Hash};

#[test]
fn phase_service_table_covers_pipeline_phases_with_classified_gaps() {
    let requirements = required_phase_services();
    let phases = requirements
        .iter()
        .flat_map(|requirement| requirement.phases.iter().copied())
        .collect::<Vec<_>>();

    assert_eq!(
        phases,
        vec![
            PipelinePhase::PackageResolve,
            PipelinePhase::SourceLoad,
            PipelinePhase::Frontend,
            PipelinePhase::ModuleResolve,
            PipelinePhase::SignatureCollection,
            PipelinePhase::TypeChecking,
            PipelinePhase::RegistrationResolution,
            PipelinePhase::OverloadResolution,
            PipelinePhase::Elaboration,
            PipelinePhase::AlgorithmPreparation,
            PipelinePhase::VcGenerate,
            PipelinePhase::VcDischarge,
            PipelinePhase::AtpSolve,
            PipelinePhase::BackendRun,
            PipelinePhase::KernelCheck,
            PipelinePhase::ArtifactCommit,
            PipelinePhase::DocumentationExtract,
        ]
    );
    assert_eq!(
        requirements
            .iter()
            .find(|requirement| requirement.service_name == "WorkspacePlanner")
            .map(|requirement| requirement.availability),
        Some(PhaseServiceAvailability::AvailableOwner)
    );
    assert_eq!(
        requirements
            .iter()
            .find(|requirement| requirement.service_name == "ArtifactService")
            .map(|requirement| requirement.availability),
        Some(PhaseServiceAvailability::ExternalDependencyGap)
    );
    assert_eq!(
        requirements
            .iter()
            .find(|requirement| requirement.service_name == "DocExtractionService")
            .map(|requirement| requirement.availability),
        Some(PhaseServiceAvailability::Deferred)
    );
}

#[test]
fn registration_is_deterministic_independent_of_input_order() {
    let first = registry_with_services(vec![
        fixture_service(
            "SourceFrontend",
            PhaseOwner::MizarFrontend,
            vec![PipelinePhase::SourceLoad, PipelinePhase::Frontend],
            2,
        ),
        fixture_service(
            "WorkspacePlanner",
            PhaseOwner::MizarBuild,
            vec![PipelinePhase::PackageResolve],
            1,
        ),
    ]);
    let second = registry_with_services(vec![
        fixture_service(
            "WorkspacePlanner",
            PhaseOwner::MizarBuild,
            vec![PipelinePhase::PackageResolve],
            1,
        ),
        fixture_service(
            "SourceFrontend",
            PhaseOwner::MizarFrontend,
            vec![PipelinePhase::SourceLoad, PipelinePhase::Frontend],
            2,
        ),
    ]);

    assert_eq!(descriptor_names(&first), descriptor_names(&second));
    assert_eq!(
        descriptor_names(&first),
        vec!["WorkspacePlanner", "SourceFrontend"]
    );
}

#[test]
fn descriptor_phase_order_is_normalized_at_constructor_and_registry_boundary() {
    let descriptor = PhaseDescriptor::new(
        "SourceFrontend",
        PhaseOwner::MizarFrontend,
        vec![PipelinePhase::Frontend, PipelinePhase::SourceLoad],
        "fixture-v1",
        "fixture-output",
    )
    .unwrap();
    assert_eq!(
        descriptor.phases,
        vec![PipelinePhase::SourceLoad, PipelinePhase::Frontend]
    );

    let service = FixtureService {
        descriptor: PhaseDescriptor {
            service_name: "RawSourceFrontend".to_owned(),
            owner: PhaseOwner::MizarFrontend,
            phases: vec![PipelinePhase::Frontend, PipelinePhase::SourceLoad],
            schema_version: "fixture-v1".to_owned(),
            output_kind: "fixture-output".to_owned(),
        },
        salt: 3,
        cache_calls: Arc::new(AtomicUsize::new(0)),
        execute_calls: Arc::new(AtomicUsize::new(0)),
    };
    let registry = registry_with_services(vec![service]);
    assert_eq!(
        registry
            .descriptor_for_phase(PipelinePhase::SourceLoad)
            .unwrap()
            .phases
            .as_slice(),
        &[PipelinePhase::SourceLoad, PipelinePhase::Frontend]
    );
}

#[test]
fn duplicate_phase_coverage_is_rejected_for_every_pipeline_phase() {
    for (index, (phase, owner)) in requirement_phases().into_iter().enumerate() {
        let mut builder = PhaseRegistryBuilder::new();
        let first_service = format!("DuplicateA{index:02}");
        let duplicate_service = format!("DuplicateB{index:02}");
        builder
            .register(fixture_service(
                &first_service,
                owner,
                vec![phase],
                index as u8,
            ))
            .register(fixture_service(
                &duplicate_service,
                owner,
                vec![phase],
                index as u8 + 1,
            ));

        let error = match builder.build() {
            Ok(_) => panic!("duplicate phase registration must fail for {phase:?}"),
            Err(error) => error,
        };
        assert_eq!(
            error,
            PhaseRegistryError::DuplicatePhase {
                phase,
                first_service,
                duplicate_service,
            }
        );
    }
}

#[test]
fn invalid_phase_descriptors_reject_non_contiguous_and_wrong_owner_spans() {
    assert!(matches!(
        PhaseDescriptor::new(
            "bad-gap",
            PhaseOwner::MizarFrontend,
            vec![PipelinePhase::SourceLoad, PipelinePhase::ModuleResolve],
            "schema",
            "output",
        ),
        Err(PhaseRegistryError::NonContiguousPhaseSpan { .. })
    ));
    assert!(matches!(
        PhaseDescriptor::new(
            "bad-owner",
            PhaseOwner::MizarFrontend,
            vec![PipelinePhase::ModuleResolve],
            "schema",
            "output",
        ),
        Err(PhaseRegistryError::OwnerMismatch { .. })
    ));
}

#[test]
fn missing_real_owner_seams_are_reported_without_synthetic_outputs() {
    let registry = PhaseRegistry::empty();

    for requirement in required_phase_services() {
        for phase in requirement.phases {
            let error = match registry.descriptor_for_phase(*phase) {
                Ok(_) => panic!("missing real owner seam must not return a descriptor"),
                Err(error) => error,
            };
            assert_eq!(
                error,
                PhaseRegistryError::MissingPhaseService {
                    phase: *phase,
                    availability: requirement.availability,
                }
            );
        }
    }
}

#[test]
fn cache_key_purity_uses_only_cache_context_identities() {
    let cache_calls = Arc::new(AtomicUsize::new(0));
    let service = fixture_service_with_calls(
        "SourceFrontend",
        PhaseOwner::MizarFrontend,
        vec![PipelinePhase::SourceLoad, PipelinePhase::Frontend],
        7,
        cache_calls.clone(),
        Arc::new(AtomicUsize::new(0)),
    );
    let registry = registry_with_services(vec![service]);
    let input = phase_input(1, 2);

    let first = registry
        .cache_key_for_phase(PipelinePhase::SourceLoad, &input)
        .unwrap();
    let second = registry
        .cache_key_for_phase(PipelinePhase::SourceLoad, &input)
        .unwrap();
    let changed = registry
        .cache_key_for_phase(PipelinePhase::SourceLoad, &phase_input(1, 3))
        .unwrap();
    let changed_dependency = registry
        .cache_key_for_phase(
            PipelinePhase::SourceLoad,
            &phase_input_with_hashes(1, 2, vec![42], vec![5]),
        )
        .unwrap();
    let changed_parent = registry
        .cache_key_for_phase(
            PipelinePhase::SourceLoad,
            &phase_input_with_hashes(1, 2, vec![3], vec![43]),
        )
        .unwrap();

    assert_eq!(first.intent, second.intent);
    assert_eq!(first.observation, second.observation);
    assert_ne!(first.intent, changed.intent);
    assert_ne!(first.intent, changed_dependency.intent);
    assert_ne!(first.intent, changed_parent.intent);
    assert_eq!(cache_calls.load(Ordering::SeqCst), 5);
}

#[test]
fn query_identity_distinguishes_dependency_and_parent_partitions() {
    let registry = registry_with_services(vec![fixture_service(
        "WorkspacePlanner",
        PhaseOwner::MizarBuild,
        vec![PipelinePhase::PackageResolve],
        11,
    )]);
    let dependency_input = phase_input_with_hashes(3, 4, vec![0xfe], Vec::new());
    let parent_input = phase_input_with_hashes(3, 4, Vec::new(), vec![0xfe]);

    let dependency_query = registry
        .cache_key_for_phase(PipelinePhase::PackageResolve, &dependency_input)
        .unwrap();
    let parent_query = registry
        .cache_key_for_phase(PipelinePhase::PackageResolve, &parent_input)
        .unwrap();

    assert_ne!(dependency_query.observation, parent_query.observation);
}

#[test]
fn query_boundary_mediates_cache_key_and_execute_calls() {
    let cache_calls = Arc::new(AtomicUsize::new(0));
    let execute_calls = Arc::new(AtomicUsize::new(0));
    let service = fixture_service_with_calls(
        "WorkspacePlanner",
        PhaseOwner::MizarBuild,
        vec![PipelinePhase::PackageResolve],
        5,
        cache_calls.clone(),
        execute_calls.clone(),
    );
    let registry = registry_with_services(vec![service]);
    let input = phase_input(9, 4);

    let cache = registry
        .cache_key_for_phase(PipelinePhase::PackageResolve, &input)
        .unwrap();
    let execution = registry
        .execute_phase(PipelinePhase::PackageResolve, input, None)
        .unwrap();

    assert_ne!(cache.observation.query_fingerprint, 0);
    assert_ne!(execution.observation.query_fingerprint, 0);
    assert_ne!(cache.observation, execution.observation);
    assert_eq!(
        execution.result.status,
        mizar_driver::registry::PhaseStatus::Complete
    );
    assert_eq!(
        execution.result.cache_observation,
        Some(execution.observation)
    );
    assert_eq!(cache_calls.load(Ordering::SeqCst), 1);
    assert_eq!(execute_calls.load(Ordering::SeqCst), 1);
}

#[test]
fn source_contains_driver_owned_salsa_boundary_but_no_semantic_authority_terms() {
    let source = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/registry.rs"),
    )
    .unwrap();

    for required in ["#[salsa::db]", "#[salsa::input", "#[salsa::tracked]"] {
        assert!(source.contains(required), "missing {required}");
    }
    for forbidden in [
        "CacheKeyBuilder",
        "ProofReuse",
        "TrustedStatus",
        "PublicationToken",
        "JsonRpc",
        "ReadyQueue",
        "BuildSessionId",
        "BuildRequestId",
        "BuildLaneId",
        "BuildRequestGeneration",
        "mizar_build::scheduler",
        "SchedulerInput",
        "TaskStateRecord",
        "CacheSchedulingPolicy",
    ] {
        assert!(
            !source.contains(forbidden),
            "registry source must not own forbidden authority term {forbidden}"
        );
    }
}

fn requirement_phases() -> Vec<(PipelinePhase, PhaseOwner)> {
    required_phase_services()
        .iter()
        .flat_map(|requirement| {
            requirement
                .phases
                .iter()
                .copied()
                .map(|phase| (phase, requirement.owner))
        })
        .collect()
}

fn registry_with_services(services: Vec<FixtureService>) -> PhaseRegistry {
    let mut builder = PhaseRegistryBuilder::new();
    for service in services {
        builder.register(service);
    }
    builder.build().unwrap()
}

fn descriptor_names(registry: &PhaseRegistry) -> Vec<&str> {
    registry
        .descriptors()
        .map(|descriptor| descriptor.service_name.as_str())
        .collect()
}

fn fixture_service(
    name: &str,
    owner: PhaseOwner,
    phases: Vec<PipelinePhase>,
    salt: u8,
) -> FixtureService {
    fixture_service_with_calls(
        name,
        owner,
        phases,
        salt,
        Arc::new(AtomicUsize::new(0)),
        Arc::new(AtomicUsize::new(0)),
    )
}

fn fixture_service_with_calls(
    name: &str,
    owner: PhaseOwner,
    phases: Vec<PipelinePhase>,
    salt: u8,
    cache_calls: Arc<AtomicUsize>,
    execute_calls: Arc<AtomicUsize>,
) -> FixtureService {
    FixtureService {
        descriptor: PhaseDescriptor::new(name, owner, phases, "fixture-v1", "fixture-output")
            .unwrap(),
        salt,
        cache_calls,
        execute_calls,
    }
}

fn phase_input(snapshot_seed: u8, input_seed: u8) -> PhaseInput {
    phase_input_with_hashes(
        snapshot_seed,
        input_seed,
        vec![input_seed.wrapping_add(1), input_seed.wrapping_add(2)],
        vec![input_seed.wrapping_add(3)],
    )
}

fn phase_input_with_hashes(
    snapshot_seed: u8,
    input_seed: u8,
    dependency_seeds: Vec<u8>,
    parent_seeds: Vec<u8>,
) -> PhaseInput {
    PhaseInput::new(
        snapshot(snapshot_seed),
        WorkUnit::Workspace,
        PhaseInputIdentities::new(
            hash(input_seed),
            dependency_seeds.into_iter().map(hash).collect(),
            parent_seeds.into_iter().map(hash).collect(),
        ),
    )
}

fn snapshot(seed: u8) -> BuildSnapshotId {
    let serialized = format!(
        "mizar-session-build-snapshot-v1:{}",
        format!("{seed:02x}").repeat(Hash::BYTE_LEN)
    );
    BuildSnapshotId::from_published_schema_str(&serialized).unwrap()
}

fn hash(seed: u8) -> Hash {
    Hash::from_bytes([seed; Hash::BYTE_LEN])
}

#[derive(Debug)]
struct FixtureService {
    descriptor: PhaseDescriptor,
    salt: u8,
    cache_calls: Arc<AtomicUsize>,
    execute_calls: Arc<AtomicUsize>,
}

impl PhaseService for FixtureService {
    fn phase(&self) -> PhaseDescriptor {
        self.descriptor.clone()
    }

    fn cache_key(&self, input: &PhaseInput, context: &PhaseCacheContext) -> PhaseCacheIntent {
        self.cache_calls.fetch_add(1, Ordering::SeqCst);
        assert_eq!(context.common.snapshot, input.snapshot);
        assert_eq!(context.input_identities, input.identities);

        let mut bytes = *input.identities.input_hash.as_bytes();
        bytes[0] = bytes[0].wrapping_add(self.salt);
        for (index, hash) in input
            .identities
            .dependency_hashes
            .iter()
            .chain(input.identities.parent_output_hashes.iter())
            .enumerate()
        {
            let target = index % Hash::BYTE_LEN;
            bytes[target] = bytes[target].wrapping_add(hash.as_bytes()[target]);
        }
        PhaseCacheIntent::Cacheable {
            intent_hash: Hash::from_bytes(bytes),
        }
    }

    fn execute(&self, input: PhaseInput, context: PhaseExecutionContext) -> PhaseResult {
        self.execute_calls.fetch_add(1, Ordering::SeqCst);
        assert_eq!(context.common.snapshot, input.snapshot);
        PhaseResult::complete()
    }
}
