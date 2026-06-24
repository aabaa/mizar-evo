//! Verification-condition generation candidates.
//!
//! This module implements the task-6 slice specified in
//! [generator.md](../../../doc/design/mizar-vc/en/generator.md): theorem,
//! definition, generated core, and registration-style correctness candidates
//! over explicit core/checker obligation seeds.

use crate::vc_ir::{
    AnchorCompleteness, AnchorIngredient, AnchorLabel, AnchorLabelRole, AnchorOwner,
    AnchorUnavailableReason, CanonicalSortKey, ContextEntry, ContextEntryId, ContextEntryKind,
    DefinitionOpacityOverride, DefinitionUnfoldRequest, GenerationSchemaVersion, HashMarker,
    LocalContext, PolicyKey, PolicyValue, PremiseRef, ProofHint, RegistrationCorrectnessKind,
    SeedIntakeMapping, SeedIntakeTable, SeedNoVcReason, SeedOriginRef, VcFormulaRef, VcIrError,
    VcKind, VcModuleRef, VcProvenance, VcProvenancePhase, VcSourceRef, VcStatus, VcText,
    VerifierPolicyInput,
};
use mizar_core::{
    control_flow::{ObligationHandoffId, ObligationSeedHandoff},
    core_ir::{
        CoreFormulaId, CoreNodeRef, CoreSourceAnchor, CoreSourceRef, LocalProofOrProgramPath,
        NormalizedSemanticOrigin, ObligationSeed, ObligationSeedKind, ObligationSeedStatus,
    },
};
use mizar_session::SourceRange;
use std::{
    collections::BTreeSet,
    error::Error,
    fmt::{self, Write as _},
};

#[derive(Debug, Clone, Copy)]
pub struct CoreGenerationInput<'a> {
    pub schema_version: &'a GenerationSchemaVersion,
    pub module: &'a VcModuleRef,
    pub intake: &'a SeedIntakeTable,
    pub handoff: &'a ObligationSeedHandoff,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CoreGenerationCandidateSet {
    schema_version: GenerationSchemaVersion,
    module: VcModuleRef,
    candidates: Vec<CoreGenerationCandidate>,
    no_candidates: Vec<CoreGenerationNoCandidate>,
}

impl CoreGenerationCandidateSet {
    pub fn try_from_seed_intake(input: CoreGenerationInput<'_>) -> Result<Self, GeneratorError> {
        let mut candidates = Vec::new();
        let mut no_candidates = Vec::new();
        let mut sort_keys = BTreeSet::new();
        let fresh_intake =
            SeedIntakeTable::try_from_handoff(input.handoff).map_err(GeneratorError::SeedIntake)?;

        if fresh_intake.rows() != input.intake.rows() {
            return Err(GeneratorError::IntakeHandoffMismatch {
                handoff: first_mismatched_handoff(input.intake.rows(), fresh_intake.rows()),
            });
        }

        for row in input.intake.rows() {
            let entry = input.handoff.entries.get(row.handoff).ok_or(
                GeneratorError::MissingHandoffEntry {
                    handoff: row.handoff,
                },
            )?;
            let seed = &entry.seed;

            match &row.mapping {
                SeedIntakeMapping::EligibleOneVc { goal } => {
                    if let Some(kind) = task_six_kind(seed) {
                        let candidate = build_candidate(BuildCandidateInput {
                            schema_version: input.schema_version,
                            module: input.module,
                            handoff: row.handoff,
                            origin: row.origin.clone(),
                            seed_status: row.seed_status,
                            seed,
                            source: row.source.clone(),
                            goal: *goal,
                            kind,
                        })?;
                        if !sort_keys.insert(candidate.sort_key.clone()) {
                            return Err(GeneratorError::DuplicateCandidateSortKey {
                                sort_key: candidate.sort_key,
                            });
                        }
                        candidates.push(candidate);
                    } else {
                        no_candidates.push(no_candidate(
                            row.handoff,
                            row.origin.clone(),
                            row.seed_status,
                            deferred_task_six_kind(seed),
                        ));
                    }
                }
                SeedIntakeMapping::NoConcreteVc { reason } => {
                    no_candidates.push(no_candidate(
                        row.handoff,
                        row.origin.clone(),
                        row.seed_status,
                        reason.clone(),
                    ));
                }
            }
        }

        Ok(Self {
            schema_version: input.schema_version.clone(),
            module: input.module.clone(),
            candidates,
            no_candidates,
        })
    }

    pub const fn schema_version(&self) -> &GenerationSchemaVersion {
        &self.schema_version
    }

    pub const fn module(&self) -> &VcModuleRef {
        &self.module
    }

    pub fn candidates(&self) -> &[CoreGenerationCandidate] {
        &self.candidates
    }

    pub fn no_candidates(&self) -> &[CoreGenerationNoCandidate] {
        &self.no_candidates
    }

    pub fn candidate_for_handoff(
        &self,
        handoff: ObligationHandoffId,
    ) -> Option<&CoreGenerationCandidate> {
        self.candidates
            .iter()
            .find(|candidate| candidate.handoff == handoff)
    }

    pub fn debug_text(&self) -> String {
        let mut output = String::from("core-generation-candidates-debug-v1\n");
        writeln!(&mut output, "schema-version: {:?}", self.schema_version).expect("write string");
        writeln!(&mut output, "module: {:?}", self.module).expect("write string");
        for candidate in &self.candidates {
            writeln!(
                &mut output,
                "candidate {:?}: sort-key={:?}; kind={:?}; goal={:?}; status={:?}; source={:?}",
                candidate.handoff,
                candidate.sort_key,
                candidate.kind,
                candidate.goal,
                candidate.status,
                candidate.source,
            )
            .expect("write string");
        }
        for no_candidate in &self.no_candidates {
            writeln!(
                &mut output,
                "no-candidate {:?}: origin={:?}; status={:?}; reason={:?}",
                no_candidate.handoff,
                no_candidate.origin,
                no_candidate.seed_status,
                no_candidate.reason,
            )
            .expect("write string");
        }
        output
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CoreGenerationCandidate {
    pub handoff: ObligationHandoffId,
    pub origin: SeedOriginRef,
    pub seed_status: ObligationSeedStatus,
    pub sort_key: CanonicalSortKey,
    pub kind: VcKind,
    pub source: VcSourceRef,
    pub owner: AnchorOwner,
    pub local_path: LocalProofOrProgramPath,
    pub label: Option<AnchorLabel>,
    pub semantic_origin: NormalizedSemanticOrigin,
    pub local_context: LocalContext,
    pub premises: Vec<PremiseRef>,
    pub goal: VcFormulaRef,
    pub proof_hint: Option<ProofHint>,
    pub status: VcStatus,
    pub provenance: Vec<VcProvenance>,
    pub anchor: crate::vc_ir::ObligationAnchor,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CoreGenerationNoCandidate {
    pub handoff: ObligationHandoffId,
    pub origin: SeedOriginRef,
    pub seed_status: ObligationSeedStatus,
    pub reason: SeedNoVcReason,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum GeneratorError {
    MissingHandoffEntry { handoff: ObligationHandoffId },
    IntakeHandoffMismatch { handoff: ObligationHandoffId },
    DuplicateCandidateSortKey { sort_key: CanonicalSortKey },
    LocalContext(VcIrError),
    SeedIntake(VcIrError),
}

impl fmt::Display for GeneratorError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingHandoffEntry { handoff } => {
                write!(formatter, "seed intake references missing {handoff:?}")
            }
            Self::IntakeHandoffMismatch { handoff } => write!(
                formatter,
                "seed intake table no longer matches handoff entry {handoff:?}"
            ),
            Self::DuplicateCandidateSortKey { sort_key } => {
                write!(
                    formatter,
                    "duplicate generation candidate sort key {sort_key:?}"
                )
            }
            Self::LocalContext(error) => write!(formatter, "invalid generated context: {error}"),
            Self::SeedIntake(error) => write!(formatter, "invalid seed intake: {error}"),
        }
    }
}

impl Error for GeneratorError {}

fn first_mismatched_handoff(
    expected: &[crate::vc_ir::SeedIntakeRow],
    actual: &[crate::vc_ir::SeedIntakeRow],
) -> ObligationHandoffId {
    expected
        .iter()
        .zip(actual)
        .find_map(|(left, right)| (left != right).then_some(left.handoff))
        .or_else(|| expected.get(actual.len()).map(|row| row.handoff))
        .or_else(|| actual.get(expected.len()).map(|row| row.handoff))
        .unwrap_or_else(|| ObligationHandoffId::new(0))
}

struct BuildCandidateInput<'a> {
    schema_version: &'a GenerationSchemaVersion,
    module: &'a VcModuleRef,
    handoff: ObligationHandoffId,
    origin: SeedOriginRef,
    seed_status: ObligationSeedStatus,
    seed: &'a ObligationSeed,
    source: CoreSourceRef,
    goal: CoreFormulaId,
    kind: VcKind,
}

fn build_candidate(
    input: BuildCandidateInput<'_>,
) -> Result<CoreGenerationCandidate, GeneratorError> {
    let BuildCandidateInput {
        schema_version,
        module,
        handoff,
        origin,
        seed_status,
        seed,
        source,
        goal,
        kind,
    } = input;
    let local_context = local_context_from_seed(seed)?;
    let mut premises = local_context
        .entries()
        .iter()
        .map(|entry| PremiseRef::LocalContext(entry.id))
        .collect::<Vec<_>>();
    if let Some(label) = &seed.label {
        premises.push(PremiseRef::LocalLabel {
            label: label.clone(),
        });
    }
    let proof_hint = proof_hint_from_seed(seed);
    let provenance = generator_provenance(seed);
    let owner = owner_for_kind(&kind, seed);
    let label = seed.label.as_ref().map(|label| AnchorLabel {
        role: AnchorLabelRole::UserLabel,
        hint: Some(label.clone()),
    });
    let source_ref = VcSourceRef {
        primary: source.clone(),
        related: related_sources(&source, &seed.source),
    };
    let anchor = anchor_for_seed(
        schema_version,
        seed,
        &kind,
        owner.clone(),
        label.clone(),
        &source,
    );

    Ok(CoreGenerationCandidate {
        handoff,
        origin,
        seed_status,
        sort_key: candidate_sort_key(schema_version, module, handoff, seed, &kind),
        kind,
        source: source_ref,
        owner,
        local_path: seed.local_path.clone(),
        label,
        semantic_origin: seed.semantic_origin.clone(),
        local_context,
        premises,
        goal: VcFormulaRef::Core(goal),
        proof_hint,
        status: VcStatus::Open,
        provenance,
        anchor,
    })
}

fn no_candidate(
    handoff: ObligationHandoffId,
    origin: SeedOriginRef,
    seed_status: ObligationSeedStatus,
    reason: SeedNoVcReason,
) -> CoreGenerationNoCandidate {
    CoreGenerationNoCandidate {
        handoff,
        origin,
        seed_status,
        reason,
    }
}

fn task_six_kind(seed: &ObligationSeed) -> Option<VcKind> {
    match seed.kind {
        ObligationSeedKind::TheoremProof => Some(
            if explicit_marker_values(seed, "vc-proof-goal").any(|value| value == "terminal") {
                VcKind::TerminalProofGoal
            } else {
                VcKind::TheoremProofStep
            },
        ),
        ObligationSeedKind::DefinitionCorrectness => registration_style_kind(seed)
            .map_or(Some(VcKind::DefinitionCorrectness), |style| {
                Some(VcKind::RegistrationStyleCorrectness { style })
            }),
        ObligationSeedKind::CheckerInitial => registration_style_kind(seed)
            .map_or(Some(VcKind::CheckerInitial), |style| {
                Some(VcKind::RegistrationStyleCorrectness { style })
            }),
        ObligationSeedKind::GeneratedNonEmptiness => Some(VcKind::GeneratedNonEmptiness),
        ObligationSeedKind::GeneratedSethood => Some(VcKind::GeneratedSethood),
        ObligationSeedKind::FraenkelMembershipAxiom => Some(VcKind::FraenkelMembershipAxiom),
        ObligationSeedKind::AlgorithmContract
        | ObligationSeedKind::AlgorithmTermination
        | ObligationSeedKind::GhostErasure => None,
        _ => None,
    }
}

fn registration_style_kind(seed: &ObligationSeed) -> Option<RegistrationCorrectnessKind> {
    explicit_marker_values(seed, "vc-registration-style").find_map(|value| match value {
        "registration" => Some(RegistrationCorrectnessKind::Registration),
        "redefinition" => Some(RegistrationCorrectnessKind::Redefinition),
        "reduction" | "reducibility" => Some(RegistrationCorrectnessKind::Reduction),
        "explicit-core-seed" | "explicit" => Some(RegistrationCorrectnessKind::ExplicitCoreSeed),
        _ => None,
    })
}

fn explicit_marker_values<'a>(
    seed: &'a ObligationSeed,
    marker: &'static str,
) -> impl Iterator<Item = &'a str> {
    seed.provenance.iter().filter_map(move |provenance| {
        provenance
            .key
            .as_str()
            .strip_prefix(marker)?
            .strip_prefix(':')
    })
}

fn deferred_task_six_kind(seed: &ObligationSeed) -> SeedNoVcReason {
    SeedNoVcReason::DeferredExternal(VcText::new(format!(
        "task 6 does not generate {:?} seeds",
        seed.kind
    )))
}

fn local_context_from_seed(seed: &ObligationSeed) -> Result<LocalContext, GeneratorError> {
    let mut formulas = seed.context.iter().copied().enumerate().collect::<Vec<_>>();
    formulas.sort_by_key(|(source_index, formula)| (formula.index(), *source_index));

    let entries = formulas
        .into_iter()
        .enumerate()
        .map(|(entry_index, (source_index, formula))| ContextEntry {
            id: ContextEntryId::new(entry_index),
            sort_key: CanonicalSortKey::new(format!(
                "core-context-formula-{:08}-source-ordinal-{:08}",
                formula.index(),
                source_index
            )),
            kind: context_kind_for_seed(seed),
            formula: Some(VcFormulaRef::Core(formula)),
            provenance: context_provenance(seed, formula),
        })
        .collect();

    LocalContext::try_new(entries, theorem_status_policy_inputs(seed))
        .map_err(GeneratorError::LocalContext)
}

fn context_kind_for_seed(seed: &ObligationSeed) -> ContextEntryKind {
    match seed.kind {
        ObligationSeedKind::TheoremProof => ContextEntryKind::ProofAssumption,
        ObligationSeedKind::DefinitionCorrectness | ObligationSeedKind::CheckerInitial => {
            ContextEntryKind::CheckerFact
        }
        ObligationSeedKind::GeneratedNonEmptiness
        | ObligationSeedKind::GeneratedSethood
        | ObligationSeedKind::FraenkelMembershipAxiom => ContextEntryKind::GeneratedFact,
        _ => ContextEntryKind::CheckerFact,
    }
}

fn proof_hint_from_seed(seed: &ObligationSeed) -> Option<ProofHint> {
    let citations = seed
        .label
        .iter()
        .cloned()
        .map(|label| PremiseRef::LocalLabel { label })
        .collect::<Vec<_>>();
    let unfold_requests = if explicit_marker_values(seed, "vc-unfold")
        .any(|value| matches!(value, "transparent" | "local" | "request" | "permitted"))
    {
        seed.core_refs
            .iter()
            .filter_map(|reference| match reference {
                CoreNodeRef::Definition(definition) => Some(DefinitionUnfoldRequest {
                    definition: *definition,
                    opacity_override: Some(DefinitionOpacityOverride::Transparent),
                }),
                _ => None,
            })
            .collect::<Vec<_>>()
    } else {
        Vec::new()
    };

    if citations.is_empty() && unfold_requests.is_empty() {
        return None;
    }

    Some(ProofHint {
        citations,
        unfold_requests,
        premise_restrictions: Vec::new(),
        solver: None,
        max_axioms: None,
        timeout: None,
        computation: None,
        provenance: generator_provenance(seed),
    })
}

fn theorem_status_policy_inputs(seed: &ObligationSeed) -> Vec<VerifierPolicyInput> {
    if !matches!(seed.kind, ObligationSeedKind::TheoremProof) {
        return Vec::new();
    }

    let explicit = explicit_marker_values(seed, "vc-theorem-status").collect::<Vec<_>>();
    let statuses = ["non-clean", "clean", "open", "assumed", "conditional"]
        .into_iter()
        .filter(|status| explicit.contains(status))
        .collect::<Vec<_>>();

    statuses
        .into_iter()
        .enumerate()
        .map(|(index, status)| VerifierPolicyInput {
            sort_key: CanonicalSortKey::new(format!("{index:04}-theorem-status-{status}")),
            key: PolicyKey::new("theorem-status-dependency"),
            value: PolicyValue::new(status),
        })
        .collect()
}

fn context_provenance(seed: &ObligationSeed, formula: CoreFormulaId) -> Vec<VcProvenance> {
    seed.provenance
        .iter()
        .cloned()
        .map(|core| VcProvenance {
            phase: VcProvenancePhase::CoreHandoff,
            key: VcText::new(format!("context-formula-{}", formula.index())),
            core: Some(core),
        })
        .collect()
}

fn generator_provenance(seed: &ObligationSeed) -> Vec<VcProvenance> {
    let mut provenance = seed
        .provenance
        .iter()
        .cloned()
        .map(|core| VcProvenance {
            phase: VcProvenancePhase::CoreHandoff,
            key: VcText::new(format!("{:?}", seed.kind)),
            core: Some(core),
        })
        .collect::<Vec<_>>();
    provenance.push(VcProvenance {
        phase: VcProvenancePhase::Generator,
        key: VcText::new("task-6-core-candidate"),
        core: None,
    });
    provenance
}

fn anchor_for_seed(
    schema_version: &GenerationSchemaVersion,
    seed: &ObligationSeed,
    kind: &VcKind,
    owner: AnchorOwner,
    label: Option<AnchorLabel>,
    source: &CoreSourceRef,
) -> crate::vc_ir::ObligationAnchor {
    let anchor_provenance = source_provenance(seed, source);
    let mut missing = vec![
        AnchorIngredient::SourceShapeHash,
        AnchorIngredient::CanonicalGoalHash,
        AnchorIngredient::CanonicalContextHash,
    ];
    if anchor_provenance.is_empty() {
        missing.insert(0, AnchorIngredient::SourceProvenance);
    }

    crate::vc_ir::ObligationAnchor {
        owner,
        kind: kind.clone(),
        local_path: seed.local_path.clone(),
        label,
        semantic_origin: seed.semantic_origin.clone(),
        source_range: source_range(source),
        provenance: anchor_provenance,
        source_shape_hash: unavailable_hash("task-6 candidate lacks source-shape hash"),
        canonical_goal_hash: unavailable_hash("task-8 owns canonical goal hashes"),
        canonical_context_hash: unavailable_hash("task-8 owns canonical context hashes"),
        generation_schema_version: schema_version.clone(),
        completeness: AnchorCompleteness::Incomplete { missing },
    }
}

fn source_provenance(seed: &ObligationSeed, source: &CoreSourceRef) -> Vec<VcProvenance> {
    seed.provenance
        .iter()
        .chain(source.provenance.iter())
        .cloned()
        .map(|core| VcProvenance {
            phase: VcProvenancePhase::CoreHandoff,
            key: VcText::new("source-provenance"),
            core: Some(core),
        })
        .collect()
}

fn unavailable_hash(reason: &str) -> HashMarker {
    HashMarker::Unavailable {
        reason: AnchorUnavailableReason::new(reason),
    }
}

fn source_range(source: &CoreSourceRef) -> Option<SourceRange> {
    match &source.anchor {
        CoreSourceAnchor::SourceRange(range) => Some(*range),
        CoreSourceAnchor::GeneratedFrom(_) => None,
        _ => None,
    }
}

fn related_sources(row_source: &CoreSourceRef, seed_source: &CoreSourceRef) -> Vec<CoreSourceRef> {
    if row_source == seed_source {
        Vec::new()
    } else {
        vec![seed_source.clone()]
    }
}

fn owner_for_kind(kind: &VcKind, seed: &ObligationSeed) -> AnchorOwner {
    match kind {
        VcKind::TheoremProofStep | VcKind::TerminalProofGoal => AnchorOwner::Theorem(seed.owner),
        VcKind::DefinitionCorrectness => AnchorOwner::Definition(seed.owner),
        VcKind::RegistrationStyleCorrectness { .. } => AnchorOwner::Registration(seed.owner),
        VcKind::CheckerInitial => AnchorOwner::CheckerOrigin(VcText::new(format!(
            "{}:{:?}",
            seed.semantic_origin.as_str(),
            seed.kind
        ))),
        VcKind::GeneratedNonEmptiness
        | VcKind::GeneratedSethood
        | VcKind::FraenkelMembershipAxiom => AnchorOwner::GeneratedSymbol(seed.owner),
        _ => AnchorOwner::CheckerOrigin(VcText::new(format!("{:?}", seed.kind))),
    }
}

fn candidate_sort_key(
    schema_version: &GenerationSchemaVersion,
    module: &VcModuleRef,
    handoff: ObligationHandoffId,
    seed: &ObligationSeed,
    kind: &VcKind,
) -> CanonicalSortKey {
    CanonicalSortKey::new(format!(
        "module={};schema={};owner={};seed-key={:?};source={:?};core-provenance={:?};expansion=000000;handoff={:08};kind={:?}",
        module.as_str(),
        schema_version.as_str(),
        seed.owner.index(),
        seed.canonical_key(),
        seed.source,
        seed.provenance,
        handoff.index(),
        kind
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use mizar_core::control_flow::{
        ObligationHandoffEntry, ObligationHandoffOrigin, ObligationHandoffTable,
    };
    use mizar_core::core_ir::{
        CoreDefinitionId, CoreDiagnosticId, CoreItemId, CoreLabelRef, CoreProvenance,
        CoreProvenanceKey, CoreProvenancePhase, GeneratedFrom, GeneratedOriginKey,
        GeneratedOriginKind, ObligationSeedId,
    };
    use mizar_session::{BuildSnapshotId, InMemorySessionIdAllocator, SessionIdAllocator};

    #[test]
    fn generates_task_six_candidates_for_core_seed_families() {
        let source = sample_source_id();
        let handoff = seed_handoff(vec![
            (
                obligation_seed(
                    source,
                    ObligationSeedKind::TheoremProof,
                    Some(CoreFormulaId::new(10)),
                    "proof/step/0",
                    "theorem:sample:proof-step:0",
                )
                .with_context(vec![CoreFormulaId::new(1)])
                .with_label("A1")
                .with_core_ref(CoreNodeRef::Definition(CoreDefinitionId::new(0)))
                .with_provenance_key("vc-unfold:transparent")
                .into(),
                ObligationHandoffOrigin::ExistingCore {
                    seed: ObligationSeedId::new(0),
                },
            ),
            (
                obligation_seed(
                    source,
                    ObligationSeedKind::TheoremProof,
                    Some(CoreFormulaId::new(15)),
                    "proof/terminal/0",
                    "theorem:sample:terminal",
                )
                .with_provenance_key("vc-proof-goal:terminal")
                .into(),
                ObligationHandoffOrigin::ExistingCore {
                    seed: ObligationSeedId::new(5),
                },
            ),
            (
                obligation_seed(
                    source,
                    ObligationSeedKind::DefinitionCorrectness,
                    Some(CoreFormulaId::new(11)),
                    "definition/existence",
                    "definition:sample:existence",
                )
                .into(),
                ObligationHandoffOrigin::ExistingCore {
                    seed: ObligationSeedId::new(1),
                },
            ),
            (
                obligation_seed(
                    source,
                    ObligationSeedKind::GeneratedNonEmptiness,
                    Some(CoreFormulaId::new(12)),
                    "generated/non-emptiness/0",
                    "generated:non-emptiness:0",
                )
                .into(),
                ObligationHandoffOrigin::ExistingCore {
                    seed: ObligationSeedId::new(2),
                },
            ),
            (
                obligation_seed(
                    source,
                    ObligationSeedKind::GeneratedSethood,
                    Some(CoreFormulaId::new(13)),
                    "generated/sethood/0",
                    "generated:sethood:0",
                )
                .into(),
                ObligationHandoffOrigin::ExistingCore {
                    seed: ObligationSeedId::new(3),
                },
            ),
            (
                obligation_seed(
                    source,
                    ObligationSeedKind::FraenkelMembershipAxiom,
                    Some(CoreFormulaId::new(14)),
                    "generated/fraenkel/0",
                    "generated:fraenkel-membership:0",
                )
                .into(),
                ObligationHandoffOrigin::ExistingCore {
                    seed: ObligationSeedId::new(4),
                },
            ),
        ]);

        let set = generate(&handoff);

        assert_eq!(set.no_candidates(), []);
        assert_eq!(
            set.candidates()
                .iter()
                .map(|candidate| &candidate.kind)
                .collect::<Vec<_>>(),
            vec![
                &VcKind::TheoremProofStep,
                &VcKind::TerminalProofGoal,
                &VcKind::DefinitionCorrectness,
                &VcKind::GeneratedNonEmptiness,
                &VcKind::GeneratedSethood,
                &VcKind::FraenkelMembershipAxiom,
            ]
        );
        let theorem = &set.candidates()[0];
        assert_eq!(
            theorem.local_context.entries()[0].formula,
            Some(VcFormulaRef::Core(CoreFormulaId::new(1)))
        );
        assert!(
            theorem
                .premises
                .contains(&PremiseRef::LocalContext(ContextEntryId::new(0)))
        );
        assert!(theorem.premises.contains(&PremiseRef::LocalLabel {
            label: CoreLabelRef::new("A1")
        }));
        let hint = theorem.proof_hint.as_ref().expect("symbolic proof hint");
        assert_eq!(hint.unfold_requests[0].definition, CoreDefinitionId::new(0));
        assert!(matches!(
            theorem.anchor.completeness,
            AnchorCompleteness::Incomplete { .. }
        ));
    }

    #[test]
    fn classifies_explicit_registration_style_payloads() {
        let source = sample_source_id();
        let handoff = seed_handoff(vec![
            (
                obligation_seed(
                    source,
                    ObligationSeedKind::DefinitionCorrectness,
                    Some(CoreFormulaId::new(0)),
                    "definition/existence/0",
                    "definition:existence",
                )
                .with_provenance_key("vc-registration-style:registration")
                .into(),
                ObligationHandoffOrigin::ExistingCore {
                    seed: ObligationSeedId::new(0),
                },
            ),
            (
                obligation_seed(
                    source,
                    ObligationSeedKind::DefinitionCorrectness,
                    Some(CoreFormulaId::new(1)),
                    "definition/compatibility",
                    "definition:compatibility",
                )
                .with_provenance_key("vc-registration-style:redefinition")
                .into(),
                ObligationHandoffOrigin::ExistingCore {
                    seed: ObligationSeedId::new(1),
                },
            ),
            (
                obligation_seed(
                    source,
                    ObligationSeedKind::CheckerInitial,
                    Some(CoreFormulaId::new(2)),
                    "checker/reducibility",
                    "checker:reducibility",
                )
                .with_provenance_key("vc-registration-style:reduction")
                .into(),
                ObligationHandoffOrigin::ExistingCore {
                    seed: ObligationSeedId::new(2),
                },
            ),
            (
                obligation_seed(
                    source,
                    ObligationSeedKind::CheckerInitial,
                    Some(CoreFormulaId::new(3)),
                    "checker/carried/0",
                    "checker:carried",
                )
                .with_provenance_key("vc-registration-style:explicit-core-seed")
                .into(),
                ObligationHandoffOrigin::ExistingCore {
                    seed: ObligationSeedId::new(3),
                },
            ),
        ]);

        let set = generate(&handoff);

        assert!(matches!(
            set.candidates()[0].kind,
            VcKind::RegistrationStyleCorrectness {
                style: RegistrationCorrectnessKind::Registration
            }
        ));
        assert!(matches!(
            set.candidates()[1].kind,
            VcKind::RegistrationStyleCorrectness {
                style: RegistrationCorrectnessKind::Redefinition
            }
        ));
        assert!(matches!(
            set.candidates()[2].kind,
            VcKind::RegistrationStyleCorrectness {
                style: RegistrationCorrectnessKind::Reduction
            }
        ));
        assert!(matches!(
            set.candidates()[3].kind,
            VcKind::RegistrationStyleCorrectness {
                style: RegistrationCorrectnessKind::ExplicitCoreSeed
            }
        ));
    }

    #[test]
    fn definition_correctness_families_stay_ordinary_definition_candidates() {
        let source = sample_source_id();
        let families = [
            "existence",
            "uniqueness",
            "coherence",
            "compatibility",
            "consistency",
            "reducibility",
            "sethood",
            "non-emptiness",
        ];
        let handoff = seed_handoff(
            families
                .iter()
                .enumerate()
                .map(|(index, family)| {
                    (
                        obligation_seed(
                            source,
                            ObligationSeedKind::DefinitionCorrectness,
                            Some(CoreFormulaId::new(index)),
                            &format!("definition/{family}"),
                            &format!("definition:sample:{family}"),
                        )
                        .into(),
                        ObligationHandoffOrigin::ExistingCore {
                            seed: ObligationSeedId::new(index),
                        },
                    )
                })
                .collect(),
        );

        let set = generate(&handoff);

        assert_eq!(set.candidates().len(), families.len());
        assert_eq!(set.no_candidates(), []);
        for (candidate, family) in set.candidates().iter().zip(families) {
            assert_eq!(candidate.kind, VcKind::DefinitionCorrectness);
            assert!(candidate.local_path.as_str().contains(family));
            assert!(candidate.semantic_origin.as_str().contains(family));
        }
    }

    #[test]
    fn preserves_explicit_theorem_status_dependency_markers_without_invention() {
        let source = sample_source_id();
        let handoff = seed_handoff(vec![
            (
                obligation_seed(
                    source,
                    ObligationSeedKind::TheoremProof,
                    Some(CoreFormulaId::new(0)),
                    "proof/non-clean/open/conditional",
                    "theorem:non-clean:open:conditional",
                )
                .with_provenance_key("vc-theorem-status:non-clean")
                .with_provenance_key("vc-theorem-status:open")
                .with_provenance_key("vc-theorem-status:conditional")
                .into(),
                ObligationHandoffOrigin::ExistingCore {
                    seed: ObligationSeedId::new(0),
                },
            ),
            (
                obligation_seed(
                    source,
                    ObligationSeedKind::TheoremProof,
                    Some(CoreFormulaId::new(1)),
                    "proof/clean/assumed",
                    "theorem:clean:assumed",
                )
                .with_provenance_key("vc-theorem-status:clean")
                .with_provenance_key("vc-theorem-status:assumed")
                .into(),
                ObligationHandoffOrigin::ExistingCore {
                    seed: ObligationSeedId::new(1),
                },
            ),
            (
                obligation_seed(
                    source,
                    ObligationSeedKind::TheoremProof,
                    Some(CoreFormulaId::new(2)),
                    "proof/plain",
                    "theorem:plain",
                )
                .with_label("registration")
                .into(),
                ObligationHandoffOrigin::ExistingCore {
                    seed: ObligationSeedId::new(2),
                },
            ),
        ]);

        let set = generate(&handoff);
        let policy_inputs = set.candidates()[0].local_context.policy_inputs();

        assert_eq!(policy_inputs.len(), 3);
        assert_eq!(policy_inputs[0].value, PolicyValue::new("non-clean"));
        assert_eq!(policy_inputs[1].value, PolicyValue::new("open"));
        assert_eq!(policy_inputs[2].value, PolicyValue::new("conditional"));
        let clean_assumed = set.candidates()[1].local_context.policy_inputs();
        assert_eq!(clean_assumed.len(), 2);
        assert_eq!(clean_assumed[0].value, PolicyValue::new("clean"));
        assert_eq!(clean_assumed[1].value, PolicyValue::new("assumed"));
        assert_eq!(set.candidates()[2].local_context.policy_inputs(), []);
        assert_eq!(set.candidates()[2].kind, VcKind::TheoremProofStep);
    }

    #[test]
    fn local_context_entries_are_canonicalized_before_dense_ids() {
        let source = sample_source_id();
        let handoff = seed_handoff(vec![(
            obligation_seed(
                source,
                ObligationSeedKind::TheoremProof,
                Some(CoreFormulaId::new(9)),
                "proof/context-order",
                "theorem:context-order",
            )
            .with_context(vec![CoreFormulaId::new(2), CoreFormulaId::new(1)])
            .into(),
            ObligationHandoffOrigin::ExistingCore {
                seed: ObligationSeedId::new(0),
            },
        )]);

        let set = generate(&handoff);
        let entries = set.candidates()[0].local_context.entries();

        assert_eq!(entries[0].id, ContextEntryId::new(0));
        assert_eq!(
            entries[0].formula,
            Some(VcFormulaRef::Core(CoreFormulaId::new(1)))
        );
        assert_eq!(entries[1].id, ContextEntryId::new(1));
        assert_eq!(
            entries[1].formula,
            Some(VcFormulaRef::Core(CoreFormulaId::new(2)))
        );
        assert!(entries[0].sort_key < entries[1].sort_key);
    }

    #[test]
    fn registration_style_labels_alone_do_not_classify_and_deferred_payloads_stay_visible() {
        let source = sample_source_id();
        let handoff = seed_handoff(vec![
            (
                obligation_seed(
                    source,
                    ObligationSeedKind::DefinitionCorrectness,
                    Some(CoreFormulaId::new(0)),
                    "definition/coherence",
                    "definition:coherence",
                )
                .with_label("registration")
                .into(),
                ObligationHandoffOrigin::ExistingCore {
                    seed: ObligationSeedId::new(0),
                },
            ),
            (
                obligation_seed(
                    source,
                    ObligationSeedKind::DefinitionCorrectness,
                    Some(CoreFormulaId::new(1)),
                    "registration/coherence/deferred",
                    "registration:coherence:deferred",
                )
                .with_provenance_key("vc-registration-style:registration")
                .with_status(ObligationSeedStatus::Deferred)
                .into(),
                ObligationHandoffOrigin::ExistingCore {
                    seed: ObligationSeedId::new(1),
                },
            ),
        ]);

        let set = generate(&handoff);

        assert_eq!(set.candidates().len(), 1);
        assert_eq!(set.candidates()[0].kind, VcKind::DefinitionCorrectness);
        assert_eq!(set.no_candidates().len(), 1);
        assert!(matches!(
            &set.no_candidates()[0].reason,
            SeedNoVcReason::DeferredExternal(reason)
                if reason.as_str().contains("seed status is deferred")
        ));
    }

    #[test]
    fn records_deferred_rows_for_no_vc_and_later_task_seed_kinds() {
        let source = sample_source_id();
        let handoff = seed_handoff(vec![
            (
                obligation_seed(
                    source,
                    ObligationSeedKind::AlgorithmContract,
                    Some(CoreFormulaId::new(0)),
                    "algorithm/requires/0",
                    "algorithm:requires:0",
                )
                .into(),
                ObligationHandoffOrigin::ExistingCore {
                    seed: ObligationSeedId::new(0),
                },
            ),
            (
                obligation_seed(
                    source,
                    ObligationSeedKind::AlgorithmTermination,
                    Some(CoreFormulaId::new(1)),
                    "algorithm/termination/0",
                    "algorithm:termination:0",
                )
                .into(),
                ObligationHandoffOrigin::ExistingCore {
                    seed: ObligationSeedId::new(1),
                },
            ),
            (
                obligation_seed(
                    source,
                    ObligationSeedKind::GhostErasure,
                    Some(CoreFormulaId::new(2)),
                    "algorithm/ghost-erasure/0",
                    "algorithm:ghost-erasure:0",
                )
                .into(),
                ObligationHandoffOrigin::ExistingCore {
                    seed: ObligationSeedId::new(2),
                },
            ),
            (
                obligation_seed(
                    source,
                    ObligationSeedKind::TheoremProof,
                    None,
                    "proof/missing-goal",
                    "theorem:missing-goal",
                )
                .into(),
                ObligationHandoffOrigin::ExistingCore {
                    seed: ObligationSeedId::new(3),
                },
            ),
            (
                obligation_seed(
                    source,
                    ObligationSeedKind::TheoremProof,
                    Some(CoreFormulaId::new(3)),
                    "proof/deferred",
                    "theorem:deferred",
                )
                .with_status(ObligationSeedStatus::Deferred)
                .into(),
                ObligationHandoffOrigin::ExistingCore {
                    seed: ObligationSeedId::new(4),
                },
            ),
            (
                obligation_seed(
                    source,
                    ObligationSeedKind::TheoremProof,
                    Some(CoreFormulaId::new(4)),
                    "proof/skipped",
                    "theorem:skipped",
                )
                .with_status(ObligationSeedStatus::Skipped)
                .into(),
                ObligationHandoffOrigin::ExistingCore {
                    seed: ObligationSeedId::new(5),
                },
            ),
            (
                obligation_seed(
                    source,
                    ObligationSeedKind::TheoremProof,
                    Some(CoreFormulaId::new(5)),
                    "proof/error",
                    "theorem:error",
                )
                .with_status(ObligationSeedStatus::Error)
                .with_diagnostics(vec![CoreDiagnosticId::new(7)])
                .into(),
                ObligationHandoffOrigin::ExistingCore {
                    seed: ObligationSeedId::new(6),
                },
            ),
        ]);

        let set = generate(&handoff);

        assert_eq!(set.candidates(), []);
        assert_eq!(set.no_candidates().len(), 7);
        assert!(matches!(
            &set.no_candidates()[0].reason,
            SeedNoVcReason::DeferredExternal(reason)
                if reason.as_str().contains("AlgorithmContract")
        ));
        assert!(matches!(
            &set.no_candidates()[1].reason,
            SeedNoVcReason::DeferredExternal(reason)
                if reason.as_str().contains("AlgorithmTermination")
        ));
        assert!(matches!(
            &set.no_candidates()[2].reason,
            SeedNoVcReason::DeferredExternal(reason)
                if reason.as_str().contains("GhostErasure")
        ));
        assert!(matches!(
            &set.no_candidates()[3].reason,
            SeedNoVcReason::MissingGoal(reason)
                if reason.as_str().contains("active seed has no goal")
        ));
        assert!(matches!(
            &set.no_candidates()[4].reason,
            SeedNoVcReason::DeferredExternal(reason)
                if reason.as_str().contains("seed status is deferred")
        ));
        assert!(matches!(
            set.no_candidates()[5].reason,
            SeedNoVcReason::SkippedInvalidInput
        ));
        assert!(matches!(
            set.no_candidates()[6].reason,
            SeedNoVcReason::Error(diagnostic)
                if diagnostic == CoreDiagnosticId::new(7)
        ));
    }

    #[test]
    fn rejects_stale_intake_handoff_mismatch() {
        let source = sample_source_id();
        let original = seed_handoff(vec![(
            obligation_seed(
                source,
                ObligationSeedKind::TheoremProof,
                Some(CoreFormulaId::new(0)),
                "proof/original",
                "theorem:original",
            )
            .into(),
            ObligationHandoffOrigin::ExistingCore {
                seed: ObligationSeedId::new(0),
            },
        )]);
        let changed = seed_handoff(vec![(
            obligation_seed(
                source,
                ObligationSeedKind::TheoremProof,
                Some(CoreFormulaId::new(0)),
                "proof/original",
                "theorem:original",
            )
            .with_status(ObligationSeedStatus::Deferred)
            .into(),
            ObligationHandoffOrigin::ExistingCore {
                seed: ObligationSeedId::new(0),
            },
        )]);
        let intake = SeedIntakeTable::try_from_handoff(&original).expect("intake");

        assert!(matches!(
            CoreGenerationCandidateSet::try_from_seed_intake(CoreGenerationInput {
                schema_version: &GenerationSchemaVersion::new("generator-task-6-test"),
                module: &VcModuleRef::new("sample"),
                intake: &intake,
                handoff: &changed,
            }),
            Err(GeneratorError::IntakeHandoffMismatch { handoff })
                if handoff == ObligationHandoffId::new(0)
        ));
    }

    #[test]
    fn rejects_partial_intake_when_handoff_adds_obligations() {
        let source = sample_source_id();
        let original = seed_handoff(vec![(
            obligation_seed(
                source,
                ObligationSeedKind::TheoremProof,
                Some(CoreFormulaId::new(0)),
                "proof/original",
                "theorem:original",
            )
            .into(),
            ObligationHandoffOrigin::ExistingCore {
                seed: ObligationSeedId::new(0),
            },
        )]);
        let expanded = seed_handoff(vec![
            (
                obligation_seed(
                    source,
                    ObligationSeedKind::TheoremProof,
                    Some(CoreFormulaId::new(0)),
                    "proof/original",
                    "theorem:original",
                )
                .into(),
                ObligationHandoffOrigin::ExistingCore {
                    seed: ObligationSeedId::new(0),
                },
            ),
            (
                obligation_seed(
                    source,
                    ObligationSeedKind::GeneratedNonEmptiness,
                    Some(CoreFormulaId::new(1)),
                    "generated/non-emptiness/0",
                    "generated:non-emptiness:0",
                )
                .into(),
                ObligationHandoffOrigin::ExistingCore {
                    seed: ObligationSeedId::new(1),
                },
            ),
        ]);
        let stale_intake = SeedIntakeTable::try_from_handoff(&original).expect("stale intake");

        assert!(matches!(
            CoreGenerationCandidateSet::try_from_seed_intake(CoreGenerationInput {
                schema_version: &GenerationSchemaVersion::new("generator-task-6-test"),
                module: &VcModuleRef::new("sample"),
                intake: &stale_intake,
                handoff: &expanded,
            }),
            Err(GeneratorError::IntakeHandoffMismatch { handoff })
                if handoff == ObligationHandoffId::new(1)
        ));
    }

    #[test]
    fn debug_rendering_is_deterministic() {
        let source = sample_source_id();
        let handoff = seed_handoff(vec![
            (
                obligation_seed(
                    source,
                    ObligationSeedKind::GeneratedSethood,
                    Some(CoreFormulaId::new(0)),
                    "generated/sethood/0",
                    "generated:sethood:0",
                )
                .into(),
                ObligationHandoffOrigin::ExistingCore {
                    seed: ObligationSeedId::new(0),
                },
            ),
            (
                obligation_seed(
                    source,
                    ObligationSeedKind::AlgorithmContract,
                    Some(CoreFormulaId::new(1)),
                    "algorithm/requires/0",
                    "algorithm:requires:0",
                )
                .into(),
                ObligationHandoffOrigin::ExistingCore {
                    seed: ObligationSeedId::new(1),
                },
            ),
            (
                obligation_seed(
                    source,
                    ObligationSeedKind::DefinitionCorrectness,
                    Some(CoreFormulaId::new(2)),
                    "definition/existence",
                    "definition:existence",
                )
                .into(),
                ObligationHandoffOrigin::ExistingCore {
                    seed: ObligationSeedId::new(2),
                },
            ),
        ]);

        let first = generate(&handoff);
        let second = generate(&handoff);

        assert_eq!(first.debug_text(), second.debug_text());
        assert!(
            first
                .debug_text()
                .contains("core-generation-candidates-debug-v1")
        );
        assert!(first.debug_text().contains("GeneratedSethood"));
        assert!(
            first
                .debug_text()
                .contains("no-candidate ObligationHandoffId(1)")
        );
    }

    fn generate(handoff: &ObligationSeedHandoff) -> CoreGenerationCandidateSet {
        let intake = SeedIntakeTable::try_from_handoff(handoff).expect("intake");
        CoreGenerationCandidateSet::try_from_seed_intake(CoreGenerationInput {
            schema_version: &GenerationSchemaVersion::new("generator-task-6-test"),
            module: &VcModuleRef::new("sample"),
            intake: &intake,
            handoff,
        })
        .expect("generation candidates")
    }

    fn seed_handoff(
        entries: Vec<(ObligationSeed, ObligationHandoffOrigin)>,
    ) -> ObligationSeedHandoff {
        let mut table = ObligationHandoffTable::new();
        let mut source_map = std::collections::BTreeMap::new();

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
        source: mizar_session::SourceId,
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
                    CoreProvenanceKey::new(local_path),
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

        fn with_label(mut self, label: &str) -> Self {
            self.seed.label = Some(CoreLabelRef::new(label));
            self
        }

        fn with_core_ref(mut self, reference: CoreNodeRef) -> Self {
            self.seed.core_refs.push(reference);
            self
        }

        fn with_provenance_key(mut self, key: &str) -> Self {
            self.seed.provenance.push(CoreProvenance::new(
                CoreProvenancePhase::Generated,
                CoreProvenanceKey::new(key),
            ));
            self
        }

        fn with_status(mut self, status: ObligationSeedStatus) -> Self {
            self.seed.status = status;
            self
        }

        fn with_diagnostics(mut self, diagnostics: Vec<CoreDiagnosticId>) -> Self {
            self.seed.diagnostics = diagnostics;
            self
        }
    }

    impl From<ObligationSeedBuilder> for ObligationSeed {
        fn from(builder: ObligationSeedBuilder) -> Self {
            builder.seed
        }
    }

    fn source_ref(source: mizar_session::SourceId) -> CoreSourceRef {
        CoreSourceRef::direct(SourceRange {
            source_id: source,
            start: 0,
            end: 10,
        })
        .with_provenance(vec![CoreProvenance::new(
            CoreProvenancePhase::Generated,
            CoreProvenanceKey::new("source"),
        )])
    }

    fn generated_source_ref() -> CoreSourceRef {
        CoreSourceRef::generated(GeneratedFrom {
            owner: CoreNodeRef::Item(CoreItemId::new(0)),
            kind: GeneratedOriginKind::TypePredicate,
            key: GeneratedOriginKey::new("generated-source"),
            reason: CoreProvenanceKey::new("generated"),
        })
    }

    fn sample_source_id() -> mizar_session::SourceId {
        let snapshot = BuildSnapshotId::from_published_schema_str(
            "mizar-session-build-snapshot-v1:\
             2222222222222222222222222222222222222222222222222222222222222222",
        )
        .expect("snapshot id");
        InMemorySessionIdAllocator::new()
            .next_source_id(snapshot)
            .expect("source id")
    }

    #[test]
    fn generated_source_refs_remain_unranged() {
        let source = generated_source_ref();
        assert_eq!(source_range(&source), None);
    }
}
