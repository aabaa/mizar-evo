use std::collections::BTreeSet;

use mizar_build::task_graph::PipelinePhase;
use mizar_session::Hash;

use crate::registry::{
    PhaseDescriptor, PhaseInput, PhaseOwner, PhaseRequirement, PhaseServiceAvailability,
};

#[derive(Debug, Clone, Copy)]
pub(super) enum QueryAdapterKind {
    CacheKey,
    Execute,
}

pub(super) const PHASE_REQUIREMENTS: [PhaseRequirement; 10] = [
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

pub(super) fn phases_are_contiguous(phases: &[PipelinePhase]) -> bool {
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

pub(super) fn owner_for_phase(phase: PipelinePhase) -> Option<PhaseOwner> {
    PHASE_REQUIREMENTS
        .iter()
        .find(|requirement| requirement.phases.contains(&phase))
        .map(|requirement| requirement.owner)
}

pub(super) fn availability_for_phase(phase: PipelinePhase) -> Option<PhaseServiceAvailability> {
    PHASE_REQUIREMENTS
        .iter()
        .find(|requirement| requirement.phases.contains(&phase))
        .map(|requirement| requirement.availability)
}

pub(super) fn phase_rank(phase: PipelinePhase) -> u8 {
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

pub(super) fn registry_query_identity_fingerprint(
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
    state.write_hash(input.identities().input_hash());
    state.write_usize(input.identities().dependency_hashes().len());
    for hash in input.identities().dependency_hashes() {
        state.write_hash(*hash);
    }
    state.write_usize(input.identities().parent_output_hashes().len());
    for hash in input.identities().parent_output_hashes() {
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
