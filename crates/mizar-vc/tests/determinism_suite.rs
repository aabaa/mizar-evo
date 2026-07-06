use mizar_core::{
    control_flow::{
        ObligationHandoffEntry, ObligationHandoffOrigin, ObligationHandoffTable,
        ObligationSeedHandoff,
    },
    core_ir::{
        CoreDiagnosticId, CoreFormulaId, CoreItemId, CoreNodeRef, CoreProvenance,
        CoreProvenancePhase, CoreSourceRef, LocalProofOrProgramPath, NormalizedSemanticOrigin,
        ObligationSeed, ObligationSeedId, ObligationSeedKind, ObligationSeedStatus,
    },
};
use mizar_session::{
    BuildSnapshotId, InMemorySessionIdAllocator, SessionIdAllocator, SourceId, SourceRange,
};
use mizar_vc::{
    dependency_slice::{
        DependencySliceInput, DependencySliceSet, KernelEvidenceDependencyInput,
        try_compute_dependency_slices, try_compute_dependency_slices_with_kernel_evidence,
    },
    discharge::{DischargeInput, DischargeOutput, DischargePolicy, DischargeRule, try_discharge},
    generator::{CoreGenerationCandidateSet, CoreGenerationInput, VcNormalizationInput},
    kernel_evidence_handoff::{
        KERNEL_FORMULA_FINGERPRINT_ALGORITHM_ID, KernelClauseTautologyPolicy,
        KernelEvidenceFingerprint, KernelEvidenceHandoffInput, KernelEvidenceProfile,
        KernelFormulaPayload, KernelFormulaProjection, KernelGoalPolarity, VcKernelEvidenceHandoff,
        build_kernel_evidence_handoff,
    },
    vc_ir::{
        AnchorCompleteness, AnchorIngredient, AnchorUnavailableReason, CanonicalSortKey,
        ContextEntry, ContextEntryId, ContextEntryKind, DischargeEvidenceRef,
        GenerationSchemaVersion, HashMarker, LocalContext, PolicyKey, PolicyValue, PremiseRef,
        SeedIntakeTable, VcFormulaRef, VcGeneratedFormula, VcGeneratedFormulaId,
        VcGeneratedFormulaKind, VcGeneratedFormulaShape, VcId, VcKind, VcModuleRef, VcProvenance,
        VcProvenancePhase, VcSchemaVersion, VcSet, VcSetParts, VcStatus, VcStatusAction,
        VcStatusOverride, VcStatusPlan, VcText, VerifierPolicyInput,
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

#[test]
fn kernel_evidence_reuse_key_requires_handoff_and_tracks_kernel_hash() {
    let base = run_reuse_pipeline(false, VcId::new(0));
    let edited = run_reuse_pipeline(true, VcId::new(1));

    assert_eq!(vc_ids(&base.normalized), vec![VcId::new(0)]);
    assert_eq!(vc_ids(&edited.normalized), vec![VcId::new(0), VcId::new(1)]);
    assert_eq!(base.discharge.evidence_records()[0].vc, VcId::new(0));
    assert_eq!(edited.discharge.evidence_records()[0].vc, VcId::new(1));

    assert!(
        base.slices
            .proof_reuse_key_for(&base.discharge, VcId::new(0))
            .is_none(),
        "kernel evidence handoff identity is required before proof reuse"
    );
    let base_key = kernel_reuse_key(&base, VcId::new(0));
    let edited_key = kernel_reuse_key(&edited, VcId::new(1));

    assert_ne!(
        base_key, edited_key,
        "changed canonical kernel handoff identity conservatively invalidates reuse"
    );

    let generated_id_shifted = run_generated_formula_id_shift_reuse_pipeline();
    let generated_id_shifted_key = kernel_reuse_key(&generated_id_shifted, VcId::new(0));
    assert_ne!(
        base_key, generated_id_shifted_key,
        "changed canonical kernel handoff identity conservatively invalidates reuse"
    );

    let base_handoff = kernel_handoff_for(&base, VcId::new(0));
    let shifted_handoff = kernel_handoff_for(&generated_id_shifted, VcId::new(0));
    assert_ne!(
        base_handoff.canonical_hash(),
        shifted_handoff.canonical_hash()
    );
    let base_kernel_slices = slices_with_kernel_handoff(&base, VcId::new(0), &base_handoff);
    assert_ne!(
        slice_fingerprints(&base.slices),
        slice_fingerprints(&base_kernel_slices),
        "kernel evidence identity must participate in dependency-slice fingerprints"
    );

    let policy_changed = run_policy_changed_reuse_pipeline();
    let policy_changed_key = kernel_reuse_key(&policy_changed, VcId::new(0));
    assert_ne!(base_key, policy_changed_key);
    assert!(
        base.slices
            .proof_reuse_key_for_kernel_handoff(
                &policy_changed.discharge,
                VcId::new(0),
                &kernel_handoff_for(&policy_changed, VcId::new(0))
            )
            .is_none(),
        "stale slice fingerprints must not authorize reuse"
    );

    let context_changed = run_context_changed_reuse_pipeline();
    let context_changed_key = kernel_reuse_key(&context_changed, VcId::new(0));
    assert_ne!(base_key, context_changed_key);
    assert_ne!(
        base_handoff.canonical_hash(),
        kernel_handoff_for(&context_changed, VcId::new(0)).canonical_hash(),
        "canonical kernel evidence hash changes must invalidate reuse identity"
    );
    assert_ne!(
        base.discharge.evidence_records()[0]
            .status_evidence
            .evidence_hash,
        context_changed.discharge.evidence_records()[0]
            .status_evidence
            .evidence_hash
    );

    let changed_goal = run_changed_generated_goal_reuse_pipeline();
    assert!(
        !changed_goal.discharge.evidence_records()[0]
            .status_evidence
            .evidence_hash
            .is_available()
    );
    assert!(
        changed_goal
            .slices
            .proof_reuse_key_for_kernel_handoff(
                &changed_goal.discharge,
                VcId::new(0),
                &kernel_handoff_for(&changed_goal, VcId::new(0))
            )
            .is_none(),
        "changed generated goal without complete evidence must fail closed"
    );

    let pre_existing = run_pre_existing_discharge_pipeline();
    assert!(
        pre_existing
            .slices
            .proof_reuse_key_for_kernel_handoff(
                &pre_existing.discharge,
                VcId::new(0),
                &kernel_handoff_for(&pre_existing, VcId::new(0))
            )
            .is_none(),
        "pre-existing discharged status is preserved evidence, not a newly produced reuse key"
    );

    let incomplete_anchor = run_incomplete_anchor_reuse_pipeline();
    assert!(
        incomplete_anchor
            .slices
            .proof_reuse_key_for_kernel_handoff(
                &incomplete_anchor.discharge,
                VcId::new(0),
                &kernel_handoff_for(&incomplete_anchor, VcId::new(0))
            )
            .is_none(),
        "incomplete anchors must fail closed"
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

fn run_reuse_pipeline(insert_before_target: bool, target: VcId) -> PipelineOutput {
    let normalized = normalize_reuse_handoff(insert_before_target);
    run_reuse_pipeline_from_normalized(with_generated_tautology_goal_at(normalized, target))
}

fn run_policy_changed_reuse_pipeline() -> PipelineOutput {
    let normalized = with_policy_input_at(
        normalize_reuse_handoff(false),
        VcId::new(0),
        "task-20-policy",
        "changed",
    );
    run_reuse_pipeline_from_normalized(with_generated_tautology_goal_at(normalized, VcId::new(0)))
}

fn run_generated_formula_id_shift_reuse_pipeline() -> PipelineOutput {
    let normalized =
        with_shifted_generated_tautology_goal_at(normalize_reuse_handoff(false), VcId::new(0));
    run_reuse_pipeline_from_normalized(normalized)
}

fn run_context_changed_reuse_pipeline() -> PipelineOutput {
    let normalized = with_context_entry_at(
        with_generated_tautology_goal_at(normalize_reuse_handoff(false), VcId::new(0)),
        VcId::new(0),
    );
    run_reuse_pipeline_from_normalized(normalized)
}

fn run_changed_generated_goal_reuse_pipeline() -> PipelineOutput {
    let normalized = with_generated_goal_at(
        normalize_reuse_handoff(false),
        VcId::new(0),
        VcGeneratedFormulaShape::Diagnostic(CoreDiagnosticId::new(20)),
        22,
    );
    run_reuse_pipeline_from_normalized(normalized)
}

fn run_pre_existing_discharge_pipeline() -> PipelineOutput {
    let normalized = with_generated_tautology_goal_at(normalize_reuse_handoff(false), VcId::new(0));
    let mut parts = parts_from_set(&normalized);
    parts.vcs[0].status = VcStatus::Discharged {
        evidence: DischargeEvidenceRef {
            rule: VcText::new("task-11-generated-tautology-v1"),
            evidence_hash: HashMarker::Available(sample_hash(7)),
        },
    };
    let pre_existing = VcSet::try_new(parts).expect("pre-existing discharged fixture");
    run_reuse_pipeline_from_normalized(pre_existing)
}

fn run_incomplete_anchor_reuse_pipeline() -> PipelineOutput {
    let normalized = with_generated_tautology_goal_at(normalize_reuse_handoff(false), VcId::new(0));
    let mut parts = parts_from_set(&normalized);
    parts.vcs[0].anchor.source_shape_hash = HashMarker::Unavailable {
        reason: AnchorUnavailableReason::new("task-20 test removes source-shape hash"),
    };
    parts.vcs[0].anchor.completeness = AnchorCompleteness::Incomplete {
        missing: vec![AnchorIngredient::SourceShapeHash],
    };
    let incomplete = VcSet::try_new(parts).expect("incomplete anchor fixture");
    run_reuse_pipeline_from_normalized(incomplete)
}

fn run_reuse_pipeline_from_normalized(discharge_input: VcSet) -> PipelineOutput {
    let discharge = try_discharge(DischargeInput {
        vc_set: &discharge_input,
        policy: &DischargePolicy::default(),
    })
    .expect("deterministic reuse discharge");
    let slices = try_compute_dependency_slices(DependencySliceInput {
        vc_set: discharge.vc_set(),
        discharge_output: Some(&discharge),
    })
    .expect("reuse dependency slices");

    PipelineOutput {
        normalized: discharge_input.clone(),
        projected_statuses: discharge_input,
        discharge,
        slices,
    }
}

fn kernel_reuse_key(
    output: &PipelineOutput,
    vc: VcId,
) -> mizar_vc::dependency_slice::ProofReuseCandidateKey {
    let handoff = kernel_handoff_for(output, vc);
    let slices = slices_with_kernel_handoff(output, vc, &handoff);
    slices
        .proof_reuse_key_for_kernel_handoff(&output.discharge, vc, &handoff)
        .expect("kernel evidence handoff enables proof reuse key")
}

fn slices_with_kernel_handoff(
    output: &PipelineOutput,
    vc: VcId,
    handoff: &VcKernelEvidenceHandoff,
) -> DependencySliceSet {
    try_compute_dependency_slices_with_kernel_evidence(
        DependencySliceInput {
            vc_set: output.discharge.vc_set(),
            discharge_output: Some(&output.discharge),
        },
        &[KernelEvidenceDependencyInput { vc, handoff }],
    )
    .expect("kernel-evidence-aware slices")
}

fn kernel_handoff_for(output: &PipelineOutput, vc: VcId) -> VcKernelEvidenceHandoff {
    let vc_set = output.discharge.vc_set();
    let payloads = vc_set
        .generated_formulas()
        .iter()
        .map(|formula| KernelFormulaPayload {
            formula_ref: VcFormulaRef::Generated(formula.id),
            projection: KernelFormulaProjection {
                formula_fingerprint: kernel_fingerprint(
                    format!("formula-{}", formula.id.index()).as_bytes(),
                ),
                formula_bytes: format!("kernel-formula-{}", formula.id.index()).into_bytes(),
                provenance_payload: format!("provenance-{}", formula.id.index()).into_bytes(),
            },
        })
        .collect::<Vec<_>>();
    build_kernel_evidence_handoff(KernelEvidenceHandoffInput {
        vc_set,
        vc,
        goal_polarity: KernelGoalPolarity::AssertFalseForRefutation,
        kernel_profile: KernelEvidenceProfile::v1(1, KernelClauseTautologyPolicy::Reject),
        symbol_manifest: &[],
        variable_manifest: &[],
        formula_payloads: &payloads,
        imported_formula_payloads: &[],
        substitutions: &[],
        formula_context: None,
        discharge_output: Some(&output.discharge),
    })
    .expect("kernel evidence handoff")
}

fn kernel_fingerprint(digest: &[u8]) -> KernelEvidenceFingerprint {
    KernelEvidenceFingerprint::new(KERNEL_FORMULA_FINGERPRINT_ALGORITHM_ID, digest.to_vec())
        .expect("kernel fingerprint")
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

fn normalize_reuse_handoff(insert_before_target: bool) -> VcSet {
    let snapshot = sample_snapshot_id();
    let source = sample_source_id(snapshot);
    let target = (
        obligation_seed(
            source,
            ObligationSeedKind::TheoremProof,
            Some(CoreFormulaId::new(20)),
            "proof/reuse-target",
            "theorem:task-20:reuse-target",
        )
        .into(),
        ObligationHandoffOrigin::ExistingCore {
            seed: ObligationSeedId::new(20),
        },
    );
    let entries = if insert_before_target {
        vec![
            (
                obligation_seed(
                    source,
                    ObligationSeedKind::TheoremProof,
                    Some(CoreFormulaId::new(19)),
                    "proof/inserted-before-target",
                    "theorem:task-20:inserted",
                )
                .with_context(vec![CoreFormulaId::new(1)])
                .into(),
                ObligationHandoffOrigin::ExistingCore {
                    seed: ObligationSeedId::new(19),
                },
            ),
            target,
        ]
    } else {
        vec![target]
    };
    let handoff = seed_handoff(entries);
    let intake = SeedIntakeTable::try_from_handoff(&handoff).expect("reuse seed intake");
    let candidates = CoreGenerationCandidateSet::try_from_seed_intake(CoreGenerationInput {
        schema_version: &GenerationSchemaVersion::new("task-20-generator"),
        module: &VcModuleRef::new("task-20"),
        intake: &intake,
        handoff: &handoff,
        flow_output: None,
    })
    .expect("reuse generation candidates");

    CoreGenerationCandidateSet::try_normalize(VcNormalizationInput {
        schema_version: &VcSchemaVersion::new("task-20-vc"),
        snapshot,
        source,
        candidates: &candidates,
    })
    .expect("reuse normalized VC set")
}

fn with_generated_tautology_goal(input: VcSet) -> VcSet {
    with_generated_tautology_goal_at(input, VcId::new(0))
}

fn with_generated_tautology_goal_at(input: VcSet, target: VcId) -> VcSet {
    with_generated_goal_at(input, target, VcGeneratedFormulaShape::True, 20)
}

fn with_generated_goal_at(
    input: VcSet,
    target: VcId,
    shape: VcGeneratedFormulaShape,
    canonical_goal_hash: u8,
) -> VcSet {
    let generated_goal = VcFormulaRef::Generated(VcGeneratedFormulaId::new(0));
    let mut parts = parts_from_set(&input);
    parts.generated_formulas = vec![generated_formula(0, shape)];
    let index = target.index();
    parts.vcs[index].goal = generated_goal;
    parts.vcs[index].premises = vec![PremiseRef::GeneratedFact {
        formula: generated_goal,
    }];
    parts.vcs[index].status = VcStatus::NeedsAtp;
    parts.vcs[index].anchor.canonical_goal_hash =
        HashMarker::Available(sample_hash(canonical_goal_hash));
    parts.vcs[index].anchor.canonical_context_hash = HashMarker::Available(sample_hash(21));
    if parts.vcs[index].anchor.source_shape_hash.is_available() {
        parts.vcs[index].anchor.completeness = AnchorCompleteness::Complete;
    }

    VcSet::try_new(parts).expect("discharge fixture VC set")
}

fn with_shifted_generated_tautology_goal_at(input: VcSet, target: VcId) -> VcSet {
    let generated_goal = VcFormulaRef::Generated(VcGeneratedFormulaId::new(1));
    let mut parts = parts_from_set(&input);
    parts.generated_formulas = vec![
        generated_formula(
            0,
            VcGeneratedFormulaShape::Diagnostic(CoreDiagnosticId::new(99)),
        ),
        generated_formula(1, VcGeneratedFormulaShape::True),
    ];
    let index = target.index();
    parts.vcs[index].goal = generated_goal;
    parts.vcs[index].premises = vec![PremiseRef::GeneratedFact {
        formula: generated_goal,
    }];
    parts.vcs[index].status = VcStatus::NeedsAtp;
    parts.vcs[index].anchor.canonical_goal_hash = HashMarker::Available(sample_hash(20));
    parts.vcs[index].anchor.canonical_context_hash = HashMarker::Available(sample_hash(21));
    if parts.vcs[index].anchor.source_shape_hash.is_available() {
        parts.vcs[index].anchor.completeness = AnchorCompleteness::Complete;
    }

    VcSet::try_new(parts).expect("shifted generated formula fixture VC set")
}

fn with_context_entry_at(input: VcSet, target: VcId) -> VcSet {
    let mut parts = parts_from_set(&input);
    let index = target.index();
    let context = &parts.vcs[index].local_context;
    let mut entries = context.entries().to_vec();
    let policy_inputs = context.policy_inputs().to_vec();
    entries.push(ContextEntry {
        id: ContextEntryId::new(entries.len()),
        sort_key: CanonicalSortKey::new("999-task-20-context"),
        kind: ContextEntryKind::TypePredicate,
        formula: None,
        provenance: vec![VcProvenance {
            phase: VcProvenancePhase::Generator,
            key: VcText::new("task-20-context-change"),
            core: None,
        }],
    });
    parts.vcs[index].local_context =
        LocalContext::try_new(entries, policy_inputs).expect("context changed fixture");
    parts.vcs[index].anchor.canonical_context_hash = HashMarker::Available(sample_hash(31));

    VcSet::try_new(parts).expect("context changed VC set")
}

fn with_policy_input_at(input: VcSet, target: VcId, key: &str, value: &str) -> VcSet {
    let mut parts = parts_from_set(&input);
    let index = target.index();
    let context = &parts.vcs[index].local_context;
    parts.vcs[index].local_context = LocalContext::try_new(
        context.entries().to_vec(),
        vec![VerifierPolicyInput {
            sort_key: CanonicalSortKey::new("000-task-20-policy"),
            key: PolicyKey::new(key),
            value: PolicyValue::new(value),
        }],
    )
    .expect("policy context");
    VcSet::try_new(parts).expect("policy changed VC set")
}

fn parts_from_set(input: &VcSet) -> VcSetParts {
    VcSetParts {
        schema_version: input.schema_version().clone(),
        snapshot: input.snapshot(),
        source: input.source(),
        module: input.module().clone(),
        generated_formulas: input.generated_formulas().to_vec(),
        vcs: input.vcs().to_vec(),
        seed_accounting: input.seed_accounting().to_vec(),
    }
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

fn sample_hash(seed: u8) -> mizar_session::Hash {
    mizar_session::Hash::from_bytes([seed; mizar_session::Hash::BYTE_LEN])
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
