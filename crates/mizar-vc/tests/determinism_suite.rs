use mizar_core::{
    control_flow::{
        ObligationHandoffEntry, ObligationHandoffOrigin, ObligationHandoffTable,
        ObligationSeedHandoff,
    },
    core_ir::{
        CoreFormulaId, CoreItemId, CoreNodeRef, CoreProvenance, CoreProvenancePhase, CoreSourceRef,
        LocalProofOrProgramPath, NormalizedSemanticOrigin, ObligationSeed, ObligationSeedId,
        ObligationSeedKind, ObligationSeedStatus,
    },
};
use mizar_session::{
    BuildSnapshotId, InMemorySessionIdAllocator, SessionIdAllocator, SourceId, SourceRange,
};
use mizar_vc::{
    dependency_slice::{DependencySliceInput, DependencySliceSet, try_compute_dependency_slices},
    discharge::{DischargeInput, DischargeOutput, DischargePolicy, DischargeRule, try_discharge},
    generator::{CoreGenerationCandidateSet, CoreGenerationInput, VcNormalizationInput},
    vc_ir::{
        GenerationSchemaVersion, PremiseRef, SeedIntakeTable, VcFormulaRef, VcGeneratedFormula,
        VcGeneratedFormulaId, VcGeneratedFormulaKind, VcGeneratedFormulaShape, VcId, VcKind,
        VcModuleRef, VcProvenance, VcProvenancePhase, VcSchemaVersion, VcSet, VcSetParts, VcStatus,
        VcStatusAction, VcStatusOverride, VcStatusPlan, VcText,
    },
};
use std::collections::BTreeMap;

#[test]
fn identical_public_inputs_have_deterministic_pipeline_outputs() {
    let first = run_public_pipeline();
    let second = run_public_pipeline();

    assert_eq!(first.normalized, second.normalized);
    assert_eq!(
        first.normalized.debug_text(),
        second.normalized.debug_text()
    );
    assert_eq!(vc_ids(&first.normalized), vec![VcId::new(0), VcId::new(1)]);
    assert_eq!(
        first
            .normalized
            .vcs()
            .iter()
            .map(|vc| &vc.kind)
            .collect::<Vec<_>>(),
        vec![&VcKind::TheoremProofStep, &VcKind::GeneratedSethood]
    );
    assert!(
        first
            .normalized
            .vcs()
            .iter()
            .all(|vc| vc.status == VcStatus::Open)
    );

    assert_eq!(first.projected_statuses, second.projected_statuses);
    assert_eq!(
        first
            .projected_statuses
            .vcs()
            .iter()
            .map(|vc| &vc.status)
            .collect::<Vec<_>>(),
        vec![
            &VcStatus::NeedsAtp,
            &VcStatus::PolicyOpen {
                policy: "task-16-manual-review".into(),
            },
        ]
    );
    assert_eq!(
        first.projected_statuses.debug_text(),
        second.projected_statuses.debug_text()
    );

    assert_eq!(first.discharge, second.discharge);
    assert_eq!(first.discharge.debug_text(), second.discharge.debug_text());
    assert_eq!(
        first
            .discharge
            .vc_set()
            .vcs()
            .iter()
            .map(|vc| vc.id)
            .collect::<Vec<_>>(),
        vec![VcId::new(0), VcId::new(1)]
    );
    assert!(matches!(
        &first.discharge.vc_set().vcs()[0].status,
        VcStatus::Discharged { .. }
    ));
    assert!(matches!(
        &first.discharge.vc_set().vcs()[1].status,
        VcStatus::PolicyOpen {
            policy
        } if policy.as_str() == "task-16-manual-review"
    ));
    assert_eq!(first.discharge.evidence_records().len(), 1);
    assert_eq!(first.discharge.evidence_records()[0].vc, VcId::new(0));
    assert_eq!(
        first.discharge.evidence_records()[0].rule,
        Some(DischargeRule::GeneratedTautology)
    );

    assert_eq!(first.slices, second.slices);
    assert_eq!(first.slices.debug_text(), second.slices.debug_text());
    assert_eq!(slice_ids(&first.slices), vec![VcId::new(0), VcId::new(1)]);
    assert_eq!(
        slice_fingerprints(&first.slices),
        slice_fingerprints(&second.slices)
    );
}

struct PipelineOutput {
    normalized: VcSet,
    projected_statuses: VcSet,
    discharge: DischargeOutput,
    slices: DependencySliceSet,
}

fn run_public_pipeline() -> PipelineOutput {
    let normalized = normalize_public_handoff();
    let projected_statuses = normalized
        .try_with_status_plan(
            &VcStatusPlan::try_new(
                VcStatusAction::NeedsAtp,
                vec![VcStatusOverride {
                    vc: VcId::new(1),
                    action: VcStatusAction::PolicyOpen {
                        policy: "task-16-manual-review".into(),
                    },
                }],
            )
            .expect("status plan"),
        )
        .expect("status projection");

    let discharge_input = with_generated_tautology_goal(projected_statuses.clone());
    let discharge = try_discharge(DischargeInput {
        vc_set: &discharge_input,
        policy: &DischargePolicy::default(),
    })
    .expect("deterministic discharge");
    let slices = try_compute_dependency_slices(DependencySliceInput {
        vc_set: discharge.vc_set(),
        discharge_output: Some(&discharge),
    })
    .expect("dependency slices");

    PipelineOutput {
        normalized,
        projected_statuses,
        discharge,
        slices,
    }
}

fn normalize_public_handoff() -> VcSet {
    let snapshot = sample_snapshot_id();
    let source = sample_source_id(snapshot);
    let handoff = seed_handoff(vec![
        (
            obligation_seed(
                source,
                ObligationSeedKind::TheoremProof,
                Some(CoreFormulaId::new(10)),
                "proof/step/0",
                "theorem:task-16:proof-step:0",
            )
            .with_context(vec![CoreFormulaId::new(1)])
            .into(),
            ObligationHandoffOrigin::ExistingCore {
                seed: ObligationSeedId::new(0),
            },
        ),
        (
            obligation_seed(
                source,
                ObligationSeedKind::GeneratedSethood,
                Some(CoreFormulaId::new(11)),
                "generated/sethood/0",
                "generated:sethood:0",
            )
            .into(),
            ObligationHandoffOrigin::ExistingCore {
                seed: ObligationSeedId::new(1),
            },
        ),
    ]);
    let intake = SeedIntakeTable::try_from_handoff(&handoff).expect("seed intake");
    let candidates = CoreGenerationCandidateSet::try_from_seed_intake(CoreGenerationInput {
        schema_version: &GenerationSchemaVersion::new("task-16-generator"),
        module: &VcModuleRef::new("task-16"),
        intake: &intake,
        handoff: &handoff,
        flow_output: None,
    })
    .expect("generation candidates");

    CoreGenerationCandidateSet::try_normalize(VcNormalizationInput {
        schema_version: &VcSchemaVersion::new("task-16-vc"),
        snapshot,
        source,
        candidates: &candidates,
    })
    .expect("normalized VC set")
}

fn with_generated_tautology_goal(input: VcSet) -> VcSet {
    let generated_goal = VcFormulaRef::Generated(VcGeneratedFormulaId::new(0));
    let mut parts = VcSetParts {
        schema_version: input.schema_version().clone(),
        snapshot: input.snapshot(),
        source: input.source(),
        module: input.module().clone(),
        generated_formulas: vec![generated_formula(0, VcGeneratedFormulaShape::True)],
        vcs: input.vcs().to_vec(),
        seed_accounting: input.seed_accounting().to_vec(),
    };
    parts.vcs[0].goal = generated_goal;
    parts.vcs[0].premises = vec![PremiseRef::GeneratedFact {
        formula: generated_goal,
    }];
    parts.vcs[0].status = VcStatus::NeedsAtp;

    VcSet::try_new(parts).expect("discharge fixture VC set")
}

fn seed_handoff(entries: Vec<(ObligationSeed, ObligationHandoffOrigin)>) -> ObligationSeedHandoff {
    let mut table = ObligationHandoffTable::new();
    let mut source_map = BTreeMap::new();

    for (seed, origin) in entries {
        let source = seed.source.clone();
        let id = table.insert(ObligationHandoffEntry {
            seed,
            origin,
            flow_site: None,
        });
        source_map.insert(id, source);
    }

    ObligationSeedHandoff {
        entries: table,
        source_map,
    }
}

fn obligation_seed(
    source: SourceId,
    kind: ObligationSeedKind,
    goal: Option<CoreFormulaId>,
    local_path: &str,
    semantic_origin: &str,
) -> ObligationSeedBuilder {
    ObligationSeedBuilder {
        seed: ObligationSeed {
            owner: CoreItemId::new(0),
            kind,
            goal,
            context: Vec::new(),
            local_path: LocalProofOrProgramPath::new(local_path),
            label: None,
            semantic_origin: NormalizedSemanticOrigin::new(semantic_origin),
            provenance: vec![CoreProvenance::new(
                CoreProvenancePhase::ProofSkeleton,
                local_path,
            )],
            source: source_ref(source),
            core_refs: goal.map(CoreNodeRef::Formula).into_iter().collect(),
            status: ObligationSeedStatus::Active,
            diagnostics: Vec::new(),
        },
    }
}

struct ObligationSeedBuilder {
    seed: ObligationSeed,
}

impl ObligationSeedBuilder {
    fn with_context(mut self, context: Vec<CoreFormulaId>) -> Self {
        self.seed.context = context;
        self
    }
}

impl From<ObligationSeedBuilder> for ObligationSeed {
    fn from(builder: ObligationSeedBuilder) -> Self {
        builder.seed
    }
}

fn generated_formula(index: usize, shape: VcGeneratedFormulaShape) -> VcGeneratedFormula {
    VcGeneratedFormula {
        id: VcGeneratedFormulaId::new(index),
        kind: VcGeneratedFormulaKind::GeneratedTypeObligation,
        shape,
        provenance: vec![VcProvenance {
            phase: VcProvenancePhase::Generator,
            key: VcText::new("task-16-generated"),
            core: None,
        }],
    }
}

fn source_ref(source: SourceId) -> CoreSourceRef {
    CoreSourceRef::direct(SourceRange {
        source_id: source,
        start: 0,
        end: 10,
    })
    .with_provenance(vec![CoreProvenance::new(
        CoreProvenancePhase::Generated,
        "task-16-source",
    )])
}

fn sample_source_id(snapshot: BuildSnapshotId) -> SourceId {
    InMemorySessionIdAllocator::new()
        .next_source_id(snapshot)
        .expect("source id")
}

fn sample_snapshot_id() -> BuildSnapshotId {
    BuildSnapshotId::from_published_schema_str(
        "mizar-session-build-snapshot-v1:\
         4444444444444444444444444444444444444444444444444444444444444444",
    )
    .expect("snapshot id")
}

fn vc_ids(set: &VcSet) -> Vec<VcId> {
    set.vcs().iter().map(|vc| vc.id).collect()
}

fn slice_ids(slices: &DependencySliceSet) -> Vec<VcId> {
    slices.slices().iter().map(|slice| slice.vc()).collect()
}

fn slice_fingerprints(slices: &DependencySliceSet) -> Vec<[u8; 32]> {
    slices
        .slices()
        .iter()
        .map(|slice| *slice.fingerprint().hash().as_bytes())
        .collect()
}
