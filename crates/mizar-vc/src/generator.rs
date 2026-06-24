//! Verification-condition generation candidates.
//!
//! This module implements the task-6 and task-7 slices specified in
//! [generator.md](../../../doc/design/mizar-vc/en/generator.md): core
//! theorem/definition/generated candidates and the currently explicit
//! goal-bearing algorithm candidates from flow-derived obligation seeds.

use crate::vc_ir::{
    AnchorCompleteness, AnchorIngredient, AnchorLabel, AnchorLabelRole, AnchorOwner,
    CanonicalSortKey, CollectionLoopObligation, ContextEntry, ContextEntryId, ContextEntryKind,
    DefinitionOpacityOverride, DefinitionUnfoldRequest, GenerationSchemaVersion, LocalContext,
    LoopInvariantPhase, PolicyKey, PolicyValue, PremiseRef, ProofHint, RangeLoopObligation,
    RegistrationCorrectnessKind, SeedAccounting, SeedIntakeMapping, SeedIntakeTable,
    SeedNoVcReason, SeedOriginRef, SeedVcMapping, SeedVcRef, VcFormulaRef, VcId, VcIr, VcIrError,
    VcKind, VcModuleRef, VcProvenance, VcProvenancePhase, VcSchemaVersion, VcSet, VcSetParts,
    VcSourceRef, VcStatus, VcText, VerifierPolicyInput, canonical_goal_hash_marker,
    hash_marker_for_payload, local_context_hash_marker,
};
use mizar_core::{
    control_flow::{
        AssertionPlacement, ContractSiteKind, ContractSitePlacement, ControlFlowExitKind,
        ControlFlowId, ControlFlowIr, ControlFlowObligationSite, ControlFlowObligationSiteKind,
        ControlFlowOutput, LoopInvariantPlacement, ObligationHandoffId, ObligationHandoffOrigin,
        ObligationSeedHandoff,
    },
    core_ir::{
        CoreAlgorithmId, CoreFormulaId, CoreNodeRef, CoreSourceAnchor, CoreSourceRef,
        LocalProofOrProgramPath, NormalizedSemanticOrigin, ObligationSeed, ObligationSeedKind,
        ObligationSeedStatus,
    },
};
use mizar_session::{BuildSnapshotId, SourceId, SourceRange};
use std::{
    collections::{BTreeMap, BTreeSet},
    error::Error,
    fmt::{self, Write as _},
};

#[derive(Debug, Clone, Copy)]
pub struct CoreGenerationInput<'a> {
    pub schema_version: &'a GenerationSchemaVersion,
    pub module: &'a VcModuleRef,
    pub intake: &'a SeedIntakeTable,
    pub handoff: &'a ObligationSeedHandoff,
    pub flow_output: Option<&'a ControlFlowOutput>,
}

#[derive(Debug, Clone, Copy)]
pub struct VcNormalizationInput<'a> {
    pub schema_version: &'a VcSchemaVersion,
    pub snapshot: BuildSnapshotId,
    pub source: SourceId,
    pub candidates: &'a CoreGenerationCandidateSet,
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
            let flow_id = flow_id_from_origin(&entry.origin);
            let flow_algorithm = flow_algorithm_from_origin(&entry.origin);
            let flow = flow_from_origin(input.flow_output, &entry.origin);

            match &row.mapping {
                SeedIntakeMapping::EligibleOneVc { goal } => {
                    if let Some(kind) = generation_kind(seed, entry.flow_site.as_ref(), flow) {
                        let candidate = build_candidate(BuildCandidateInput {
                            schema_version: input.schema_version,
                            module: input.module,
                            handoff: row.handoff,
                            origin: row.origin.clone(),
                            seed_status: row.seed_status,
                            seed,
                            flow_id,
                            flow_algorithm,
                            flow_site: entry.flow_site.as_ref(),
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
                            no_candidate_reason_for_seed(
                                seed,
                                &entry.origin,
                                entry.flow_site.as_ref(),
                                input.flow_output,
                                None,
                            ),
                        ));
                    }
                }
                SeedIntakeMapping::NoConcreteVc { reason } => {
                    no_candidates.push(no_candidate(
                        row.handoff,
                        row.origin.clone(),
                        row.seed_status,
                        no_candidate_reason_for_seed(
                            seed,
                            &entry.origin,
                            entry.flow_site.as_ref(),
                            input.flow_output,
                            Some(reason),
                        ),
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

    pub fn try_normalize(input: VcNormalizationInput<'_>) -> Result<VcSet, GeneratorError> {
        let mut candidates = input.candidates.candidates.iter().collect::<Vec<_>>();
        let mut seen_sort_keys = BTreeSet::new();

        for candidate in &candidates {
            if !seen_sort_keys.insert(candidate.sort_key.clone()) {
                return Err(GeneratorError::DuplicateCandidateSortKey {
                    sort_key: candidate.sort_key.clone(),
                });
            }
        }

        candidates.sort_by_key(|candidate| {
            (
                kind_classification_rank(&candidate.kind),
                candidate.sort_key.clone(),
                candidate.handoff,
            )
        });

        let mut seed_accounting = BTreeMap::new();
        let mut vcs = Vec::with_capacity(candidates.len());

        for (index, candidate) in candidates.into_iter().enumerate() {
            let id = VcId::new(index);
            insert_seed_accounting(
                &mut seed_accounting,
                SeedAccounting {
                    handoff: candidate.handoff,
                    origin: candidate.origin.clone(),
                    seed_status: candidate.seed_status,
                    mapping: SeedVcMapping::One { vc: id },
                },
            )?;
            vcs.push(VcIr {
                id,
                kind: candidate.kind.clone(),
                source: candidate.source.clone(),
                seed: SeedVcRef {
                    handoff: candidate.handoff,
                },
                anchor: candidate.anchor.clone(),
                local_context: candidate.local_context.clone(),
                premises: candidate.premises.clone(),
                goal: candidate.goal,
                proof_hint: candidate.proof_hint.clone(),
                status: candidate.status.clone(),
                provenance: normalized_provenance(candidate.provenance.clone()),
            });
        }

        let mut no_candidates = input.candidates.no_candidates.iter().collect::<Vec<_>>();
        no_candidates.sort_by_key(|no_candidate| no_candidate.handoff);
        for no_candidate in no_candidates {
            insert_seed_accounting(
                &mut seed_accounting,
                SeedAccounting {
                    handoff: no_candidate.handoff,
                    origin: no_candidate.origin.clone(),
                    seed_status: no_candidate.seed_status,
                    mapping: SeedVcMapping::NoConcreteVc {
                        reason: no_candidate.reason.clone(),
                    },
                },
            )?;
        }

        VcSet::try_new(VcSetParts {
            schema_version: input.schema_version.clone(),
            snapshot: input.snapshot,
            source: input.source,
            module: input.candidates.module.clone(),
            generated_formulas: Vec::new(),
            vcs,
            seed_accounting: seed_accounting.into_values().collect(),
        })
        .map_err(GeneratorError::VcSet)
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
    DuplicateSeedOutput { handoff: ObligationHandoffId },
    LocalContext(VcIrError),
    SeedIntake(VcIrError),
    VcSet(VcIrError),
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
            Self::DuplicateSeedOutput { handoff } => {
                write!(formatter, "duplicate generation output for {handoff:?}")
            }
            Self::LocalContext(error) => write!(formatter, "invalid generated context: {error}"),
            Self::SeedIntake(error) => write!(formatter, "invalid seed intake: {error}"),
            Self::VcSet(error) => write!(formatter, "invalid normalized VC set: {error}"),
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
    flow_id: Option<ControlFlowId>,
    flow_algorithm: Option<CoreAlgorithmId>,
    flow_site: Option<&'a ControlFlowObligationSite>,
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
        flow_id,
        flow_algorithm,
        flow_site,
        source,
        goal,
        kind,
    } = input;
    let local_context = local_context_from_seed(seed, flow_id, flow_site)?;
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
    let provenance = generator_provenance(seed, generator_stage_key(&kind));
    let owner = owner_for_kind(&kind, seed, flow_algorithm);
    let label = seed.label.as_ref().map(|label| AnchorLabel {
        role: AnchorLabelRole::UserLabel,
        hint: Some(label.clone()),
    });
    let source_ref = VcSourceRef {
        primary: source.clone(),
        related: related_sources(&source, &seed.source),
    };
    let goal = VcFormulaRef::Core(goal);
    let anchor = anchor_for_seed(AnchorForSeedInput {
        schema_version,
        seed,
        kind: &kind,
        owner: owner.clone(),
        label: label.clone(),
        source: &source,
        goal,
        local_context: &local_context,
    });

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
        goal,
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

fn insert_seed_accounting(
    rows: &mut BTreeMap<ObligationHandoffId, SeedAccounting>,
    row: SeedAccounting,
) -> Result<(), GeneratorError> {
    let handoff = row.handoff;
    if rows.insert(handoff, row).is_some() {
        return Err(GeneratorError::DuplicateSeedOutput { handoff });
    }
    Ok(())
}

fn normalized_provenance(mut provenance: Vec<VcProvenance>) -> Vec<VcProvenance> {
    provenance.push(VcProvenance {
        phase: VcProvenancePhase::Normalization,
        key: VcText::new("task-8-normalized-vc"),
        core: None,
    });
    provenance
}

fn kind_classification_rank(kind: &VcKind) -> (u8, u8) {
    match kind {
        VcKind::TheoremProofStep => (0, 0),
        VcKind::TerminalProofGoal => (1, 0),
        VcKind::DefinitionCorrectness => (2, 0),
        VcKind::RegistrationStyleCorrectness { style } => {
            (3, registration_correctness_rank(*style))
        }
        VcKind::CheckerInitial => (4, 0),
        VcKind::GeneratedNonEmptiness => (5, 0),
        VcKind::GeneratedSethood => (6, 0),
        VcKind::FraenkelMembershipAxiom => (7, 0),
        VcKind::AlgorithmPrecondition => (8, 0),
        VcKind::AlgorithmPostcondition => (9, 0),
        VcKind::CallPrecondition => (10, 0),
        VcKind::AlgorithmAssertion => (11, 0),
        VcKind::LoopInvariant { phase } => (12, loop_invariant_rank(*phase)),
        VcKind::RangeLoop { obligation } => (13, range_loop_rank(*obligation)),
        VcKind::CollectionLoop { obligation } => (14, collection_loop_rank(*obligation)),
        VcKind::Termination => (15, 0),
        VcKind::PartialTermination => (16, 0),
        VcKind::GhostErasureSafety => (17, 0),
        VcKind::PolicyDeferredTraceability => (18, 0),
    }
}

fn registration_correctness_rank(style: RegistrationCorrectnessKind) -> u8 {
    match style {
        RegistrationCorrectnessKind::Registration => 0,
        RegistrationCorrectnessKind::Redefinition => 1,
        RegistrationCorrectnessKind::Reduction => 2,
        RegistrationCorrectnessKind::ExplicitCoreSeed => 3,
    }
}

fn loop_invariant_rank(phase: LoopInvariantPhase) -> u8 {
    match phase {
        LoopInvariantPhase::Entry => 0,
        LoopInvariantPhase::Preservation => 1,
        LoopInvariantPhase::Break => 2,
        LoopInvariantPhase::Continue => 3,
        LoopInvariantPhase::Exit => 4,
    }
}

fn range_loop_rank(obligation: RangeLoopObligation) -> u8 {
    match obligation {
        RangeLoopObligation::PositiveStep => 0,
        RangeLoopObligation::RangeBound => 1,
        RangeLoopObligation::HiddenIndex => 2,
    }
}

fn collection_loop_rank(obligation: CollectionLoopObligation) -> u8 {
    match obligation {
        CollectionLoopObligation::Finiteness => 0,
        CollectionLoopObligation::OrderIndependence => 1,
    }
}

fn generation_kind(
    seed: &ObligationSeed,
    flow_site: Option<&ControlFlowObligationSite>,
    flow: Option<&ControlFlowIr>,
) -> Option<VcKind> {
    task_six_kind(seed).or_else(|| task_seven_algorithm_kind(seed, flow_site?, flow?))
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

fn task_seven_algorithm_kind(
    seed: &ObligationSeed,
    site: &ControlFlowObligationSite,
    flow: &ControlFlowIr,
) -> Option<VcKind> {
    if !matches!(seed.kind, ObligationSeedKind::AlgorithmContract) {
        return None;
    }

    match site.kind {
        ControlFlowObligationSiteKind::Requires => {
            flow_requires_site(seed, site, flow).then_some(VcKind::AlgorithmPrecondition)
        }
        ControlFlowObligationSiteKind::Ensures => {
            flow_ensures_site(seed, site, flow).then_some(VcKind::AlgorithmPostcondition)
        }
        ControlFlowObligationSiteKind::AlgorithmAssertion
        | ControlFlowObligationSiteKind::StatementAssertion => {
            flow_assertion_site(seed, site, flow).then_some(VcKind::AlgorithmAssertion)
        }
        ControlFlowObligationSiteKind::AlgorithmInvariant => flow_invariant_site(seed, site, flow)
            .is_some_and(|phase| phase == LoopInvariantPhase::Entry)
            .then_some(VcKind::LoopInvariant {
                phase: LoopInvariantPhase::Entry,
            }),
        ControlFlowObligationSiteKind::LoopInvariant => Some(VcKind::LoopInvariant {
            phase: flow_invariant_site(seed, site, flow)?,
        }),
        ControlFlowObligationSiteKind::TerminationMeasure
        | ControlFlowObligationSiteKind::PartialTermination
        | ControlFlowObligationSiteKind::GhostPick
        | ControlFlowObligationSiteKind::GhostAssignment => None,
        _ => None,
    }
}

fn flow_requires_site(
    seed: &ObligationSeed,
    site: &ControlFlowObligationSite,
    flow: &ControlFlowIr,
) -> bool {
    let Some(contract) = flow.contracts.requires.get(site.ordinal) else {
        return false;
    };
    if contract.kind != ContractSiteKind::Requires || Some(contract.formula) != seed.goal {
        return false;
    }
    match contract.placement {
        ContractSitePlacement::Entry { block, .. } => {
            site.block.is_none_or(|site_block| site_block == block)
                && site.exit.is_none()
                && site.statement.is_none()
        }
        ContractSitePlacement::Return { block, exit } => {
            site.block.is_none_or(|site_block| site_block == block)
                && site.exit.is_none_or(|site_exit| site_exit == exit)
        }
        _ => false,
    }
}

fn flow_ensures_site(
    seed: &ObligationSeed,
    site: &ControlFlowObligationSite,
    flow: &ControlFlowIr,
) -> bool {
    let Some(contract) = flow.contracts.ensures.get(site.ordinal) else {
        return false;
    };
    if contract.kind != ContractSiteKind::Ensures || Some(contract.formula) != seed.goal {
        return false;
    }
    match contract.placement {
        ContractSitePlacement::Entry { block, .. } => {
            site.block.is_none_or(|site_block| site_block == block)
                && site.exit.is_none()
                && site.statement.is_none()
        }
        ContractSitePlacement::Return { block, exit } => {
            site.block.is_none_or(|site_block| site_block == block)
                && site.exit.is_none_or(|site_exit| site_exit == exit)
        }
        _ => false,
    }
}

fn flow_assertion_site(
    seed: &ObligationSeed,
    site: &ControlFlowObligationSite,
    flow: &ControlFlowIr,
) -> bool {
    let Some(assertion) = flow.contracts.assertions.get(site.ordinal) else {
        return false;
    };
    if Some(assertion.formula) != seed.goal {
        return false;
    }
    match assertion.placement {
        AssertionPlacement::AlgorithmContract { block, .. } => {
            site.kind == ControlFlowObligationSiteKind::AlgorithmAssertion
                && site.block.is_none_or(|site_block| site_block == block)
                && site.statement.is_none()
        }
        AssertionPlacement::Statement {
            statement, block, ..
        } => {
            site.kind == ControlFlowObligationSiteKind::StatementAssertion
                && site
                    .statement
                    .is_none_or(|site_statement| site_statement == statement)
                && site.block.is_none_or(|site_block| site_block == block)
        }
        _ => false,
    }
}

fn flow_invariant_site(
    seed: &ObligationSeed,
    site: &ControlFlowObligationSite,
    flow: &ControlFlowIr,
) -> Option<LoopInvariantPhase> {
    let invariant = flow.contracts.loop_invariants.get(site.ordinal)?;
    if Some(invariant.formula) != seed.goal {
        return None;
    }
    match invariant.placement {
        LoopInvariantPlacement::AlgorithmContract { block, .. } => (site.kind
            == ControlFlowObligationSiteKind::AlgorithmInvariant
            && site.block.is_none_or(|site_block| site_block == block)
            && site.loop_id.is_none()
            && site.exit.is_none())
        .then_some(LoopInvariantPhase::Entry),
        LoopInvariantPlacement::Header { loop_id, block } => (site.kind
            == ControlFlowObligationSiteKind::LoopInvariant
            && site.loop_id == Some(loop_id)
            && site.block == Some(block)
            && site.exit.is_none())
        .then_some(LoopInvariantPhase::Entry),
        LoopInvariantPlacement::NormalBackedge { loop_id, from, .. } => (site.kind
            == ControlFlowObligationSiteKind::LoopInvariant
            && site.loop_id == Some(loop_id)
            && site.block == Some(from)
            && site.exit.is_none())
        .then_some(LoopInvariantPhase::Preservation),
        LoopInvariantPlacement::BreakExit { loop_id, exit } => {
            let flow_exit = flow.exits.get(exit)?;
            (site.kind == ControlFlowObligationSiteKind::LoopInvariant
                && site.loop_id == Some(loop_id)
                && site.exit == Some(exit)
                && matches!(
                    &flow_exit.kind,
                    ControlFlowExitKind::Break { loop_id: exit_loop } if *exit_loop == loop_id
                ))
            .then_some(LoopInvariantPhase::Break)
        }
        LoopInvariantPlacement::ContinueExit { loop_id, exit } => {
            let flow_exit = flow.exits.get(exit)?;
            (site.kind == ControlFlowObligationSiteKind::LoopInvariant
                && site.loop_id == Some(loop_id)
                && site.exit == Some(exit)
                && matches!(
                    &flow_exit.kind,
                    ControlFlowExitKind::Continue { loop_id: exit_loop } if *exit_loop == loop_id
                ))
            .then_some(LoopInvariantPhase::Continue)
        }
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

fn flow_id_from_origin(origin: &ObligationHandoffOrigin) -> Option<ControlFlowId> {
    match origin {
        ObligationHandoffOrigin::FlowDerived { flow, .. } => Some(*flow),
        _ => None,
    }
}

fn flow_algorithm_from_origin(origin: &ObligationHandoffOrigin) -> Option<CoreAlgorithmId> {
    match origin {
        ObligationHandoffOrigin::FlowDerived { algorithm, .. } => Some(*algorithm),
        _ => None,
    }
}

fn flow_from_origin<'a>(
    flow_output: Option<&'a ControlFlowOutput>,
    origin: &ObligationHandoffOrigin,
) -> Option<&'a ControlFlowIr> {
    let flow = flow_id_from_origin(origin)?;
    let algorithm = flow_algorithm_from_origin(origin)?;
    flow_output?
        .flows
        .get(flow)
        .filter(|flow_ir| flow_ir.algorithm == algorithm)
}

fn no_candidate_reason_for_seed(
    seed: &ObligationSeed,
    origin: &ObligationHandoffOrigin,
    flow_site: Option<&ControlFlowObligationSite>,
    flow_output: Option<&ControlFlowOutput>,
    intake_reason: Option<&SeedNoVcReason>,
) -> SeedNoVcReason {
    if matches!(
        seed.status,
        ObligationSeedStatus::Skipped | ObligationSeedStatus::Error
    ) {
        return intake_reason
            .cloned()
            .unwrap_or_else(|| deferred_task_six_kind(seed));
    }

    if matches!(
        seed.kind,
        ObligationSeedKind::AlgorithmContract
            | ObligationSeedKind::AlgorithmTermination
            | ObligationSeedKind::GhostErasure
    ) {
        return task_seven_no_candidate_reason(seed, origin, flow_site, flow_output);
    }

    intake_reason
        .cloned()
        .unwrap_or_else(|| deferred_task_six_kind(seed))
}

fn task_seven_no_candidate_reason(
    seed: &ObligationSeed,
    origin: &ObligationHandoffOrigin,
    flow_site: Option<&ControlFlowObligationSite>,
    flow_output: Option<&ControlFlowOutput>,
) -> SeedNoVcReason {
    let Some(flow) = flow_id_from_origin(origin) else {
        return SeedNoVcReason::DeferredExternal(VcText::new(format!(
            "task 7 requires FlowDerived origin for {:?} seed",
            seed.kind
        )));
    };
    let Some(site) = flow_site else {
        return SeedNoVcReason::DeferredExternal(VcText::new(format!(
            "task 7 requires explicit ControlFlowObligationSite for {:?} seed",
            seed.kind
        )));
    };
    if flow_from_origin(flow_output, origin).is_none() {
        return SeedNoVcReason::DeferredExternal(VcText::new(format!(
            "task 7 requires matching ControlFlowOutput for flow {}",
            flow.index()
        )));
    }
    if seed.goal.is_none() {
        return SeedNoVcReason::DeferredExternal(VcText::new(format!(
            "task 7 requires explicit goal formula for {:?} site {:?}",
            seed.kind, site.kind
        )));
    }
    SeedNoVcReason::DeferredExternal(VcText::new(format!(
        "task 7 does not generate {:?} site {:?}",
        seed.kind, site.kind
    )))
}

fn local_context_from_seed(
    seed: &ObligationSeed,
    flow_id: Option<ControlFlowId>,
    flow_site: Option<&ControlFlowObligationSite>,
) -> Result<LocalContext, GeneratorError> {
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

    LocalContext::try_new(entries, policy_inputs_from_seed(seed, flow_id, flow_site))
        .map_err(GeneratorError::LocalContext)
}

fn policy_inputs_from_seed(
    seed: &ObligationSeed,
    flow_id: Option<ControlFlowId>,
    flow_site: Option<&ControlFlowObligationSite>,
) -> Vec<VerifierPolicyInput> {
    let mut inputs = theorem_status_policy_inputs(seed);
    inputs.extend(algorithm_site_policy_inputs(flow_id, flow_site));
    inputs.sort_by(|left, right| left.sort_key.cmp(&right.sort_key));
    inputs
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
        provenance: generator_provenance(seed, "task-6-core-candidate"),
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

fn algorithm_site_policy_inputs(
    flow_id: Option<ControlFlowId>,
    flow_site: Option<&ControlFlowObligationSite>,
) -> Vec<VerifierPolicyInput> {
    let Some(site) = flow_site else {
        return Vec::new();
    };
    let mut values = Vec::new();
    if let Some(flow) = flow_id {
        values.push(("flow", flow.index().to_string()));
    }
    values.push(("site-kind", format!("{:?}", site.kind)));
    values.push(("ordinal", site.ordinal.to_string()));
    if let Some(statement) = site.statement {
        values.push(("statement", statement.index().to_string()));
    }
    if let Some(block) = site.block {
        values.push(("block", block.index().to_string()));
    }
    if let Some(loop_id) = site.loop_id {
        values.push(("loop", loop_id.index().to_string()));
    }
    if let Some(exit) = site.exit {
        values.push(("exit", exit.index().to_string()));
    }
    if let Some(local) = site.local {
        values.push(("local", local.index().to_string()));
    }
    if let Some(effect) = site.assignment_effect {
        values.push(("assignment-effect", effect.index().to_string()));
    }

    values
        .into_iter()
        .enumerate()
        .map(|(index, (key, value))| VerifierPolicyInput {
            sort_key: CanonicalSortKey::new(format!("{index:04}-algorithm-site-{key}")),
            key: PolicyKey::new(format!("algorithm-site-{key}")),
            value: PolicyValue::new(value),
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

fn generator_provenance(seed: &ObligationSeed, stage_key: &'static str) -> Vec<VcProvenance> {
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
        key: VcText::new(stage_key),
        core: None,
    });
    provenance
}

fn generator_stage_key(kind: &VcKind) -> &'static str {
    if matches!(
        kind,
        VcKind::AlgorithmPrecondition
            | VcKind::AlgorithmPostcondition
            | VcKind::CallPrecondition
            | VcKind::AlgorithmAssertion
            | VcKind::LoopInvariant { .. }
            | VcKind::RangeLoop { .. }
            | VcKind::CollectionLoop { .. }
            | VcKind::Termination
            | VcKind::PartialTermination
            | VcKind::GhostErasureSafety
    ) {
        "task-7-algorithm-candidate"
    } else {
        "task-6-core-candidate"
    }
}

struct AnchorForSeedInput<'a> {
    schema_version: &'a GenerationSchemaVersion,
    seed: &'a ObligationSeed,
    kind: &'a VcKind,
    owner: AnchorOwner,
    label: Option<AnchorLabel>,
    source: &'a CoreSourceRef,
    goal: VcFormulaRef,
    local_context: &'a LocalContext,
}

fn anchor_for_seed(input: AnchorForSeedInput<'_>) -> crate::vc_ir::ObligationAnchor {
    let AnchorForSeedInput {
        schema_version,
        seed,
        kind,
        owner,
        label,
        source,
        goal,
        local_context,
    } = input;
    let anchor_provenance = source_provenance(seed, source);
    let source_shape_hash =
        source_shape_hash_marker(&owner, kind, seed, label.as_ref(), &anchor_provenance);
    let canonical_goal_hash = canonical_goal_hash_marker(goal);
    let canonical_context_hash = local_context_hash_marker(local_context);
    let mut missing = Vec::new();
    if anchor_provenance.is_empty() {
        missing.push(AnchorIngredient::SourceProvenance);
    }
    for (ingredient, marker) in [
        (AnchorIngredient::SourceShapeHash, &source_shape_hash),
        (AnchorIngredient::CanonicalGoalHash, &canonical_goal_hash),
        (
            AnchorIngredient::CanonicalContextHash,
            &canonical_context_hash,
        ),
    ] {
        if !marker.is_available() {
            missing.push(ingredient);
        }
    }
    missing.sort();

    let completeness = if missing.is_empty() {
        AnchorCompleteness::Complete
    } else {
        AnchorCompleteness::Incomplete { missing }
    };

    crate::vc_ir::ObligationAnchor {
        owner,
        kind: kind.clone(),
        local_path: seed.local_path.clone(),
        label,
        semantic_origin: seed.semantic_origin.clone(),
        source_range: source_range(source),
        provenance: anchor_provenance,
        source_shape_hash,
        canonical_goal_hash,
        canonical_context_hash,
        generation_schema_version: schema_version.clone(),
        completeness,
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

fn source_shape_hash_marker(
    owner: &AnchorOwner,
    kind: &VcKind,
    seed: &ObligationSeed,
    label: Option<&AnchorLabel>,
    provenance: &[VcProvenance],
) -> crate::vc_ir::HashMarker {
    let mut payload = String::from("source-shape-hash-v1\n");
    writeln!(
        &mut payload,
        "owner: {}",
        stable_anchor_owner_payload(owner)
    )
    .expect("write string");
    writeln!(&mut payload, "kind: {kind:?}").expect("write string");
    writeln!(&mut payload, "local-path: {:?}", seed.local_path).expect("write string");
    writeln!(&mut payload, "label: {label:?}").expect("write string");
    writeln!(&mut payload, "semantic-origin: {:?}", seed.semantic_origin).expect("write string");
    writeln!(&mut payload, "provenance: {provenance:?}").expect("write string");
    hash_marker_for_payload("mizar-vc-source-shape", &payload)
}

fn stable_anchor_owner_payload(owner: &AnchorOwner) -> String {
    match owner {
        AnchorOwner::Theorem(_) => "theorem".to_owned(),
        AnchorOwner::Definition(_) => "definition".to_owned(),
        AnchorOwner::Registration(_) => "registration".to_owned(),
        AnchorOwner::GeneratedSymbol(_) => "generated-symbol".to_owned(),
        AnchorOwner::Algorithm(_) => "algorithm".to_owned(),
        AnchorOwner::ProofBlock(_) => "proof-block".to_owned(),
        AnchorOwner::CheckerOrigin(origin) => format!("checker-origin:{}", origin.as_str()),
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

fn owner_for_kind(
    kind: &VcKind,
    seed: &ObligationSeed,
    flow_algorithm: Option<CoreAlgorithmId>,
) -> AnchorOwner {
    match kind {
        VcKind::TheoremProofStep | VcKind::TerminalProofGoal => AnchorOwner::Theorem(seed.owner),
        VcKind::DefinitionCorrectness => AnchorOwner::Definition(seed.owner),
        VcKind::RegistrationStyleCorrectness { .. } => AnchorOwner::Registration(seed.owner),
        VcKind::AlgorithmPrecondition
        | VcKind::AlgorithmPostcondition
        | VcKind::CallPrecondition
        | VcKind::AlgorithmAssertion
        | VcKind::LoopInvariant { .. }
        | VcKind::RangeLoop { .. }
        | VcKind::CollectionLoop { .. }
        | VcKind::Termination
        | VcKind::PartialTermination
        | VcKind::GhostErasureSafety => flow_algorithm.map_or_else(
            || AnchorOwner::CheckerOrigin(VcText::new(format!("{:?}", seed.kind))),
            AnchorOwner::Algorithm,
        ),
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
    use crate::vc_ir::{ExpandedVcRef, ExpansionSchemaVersion};
    use mizar_core::control_flow::{
        AssertionPlacement, AssertionSite, BasicBlockId, CallSiteTable, ContextFactTable,
        ContractSite, ContractSiteKind, ContractSitePlacement, ControlFlowBlockTable,
        ControlFlowContractSet, ControlFlowDiagnosticTable, ControlFlowExit, ControlFlowExitId,
        ControlFlowExitKind, ControlFlowExitTable, ControlFlowGhostTable, ControlFlowId,
        ControlFlowIr, ControlFlowLocalTable, ControlFlowLoop, ControlFlowLoopTable,
        ControlFlowObligationSite, ControlFlowObligationSiteKind, ControlFlowOutput,
        ControlFlowSourceMap, ControlFlowTable, LocalId, LoopId, LoopInvariantPlacement,
        LoopInvariantSite, ObligationHandoffEntry, ObligationHandoffOrigin, ObligationHandoffTable,
        ProgramContextId, ProgramContextTable,
    };
    use mizar_core::core_ir::{
        CoreAlgorithmId, CoreDefinitionId, CoreDiagnosticId, CoreItemId, CoreLabelRef,
        CoreProvenance, CoreProvenanceKey, CoreProvenancePhase, GeneratedFrom, GeneratedOriginKey,
        GeneratedOriginKind, ObligationSeedId,
    };
    use mizar_resolve::resolved_ast::{FullyQualifiedName, LocalSymbolId, ModuleId, SymbolId};
    use mizar_session::snapshot::{ModulePath, PackageId};
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
        for candidate in set.candidates() {
            assert!(candidate.anchor.source_shape_hash.is_available());
            assert!(!candidate.anchor.canonical_goal_hash.is_available());
            let AnchorCompleteness::Incomplete { missing } = &candidate.anchor.completeness else {
                panic!("core goal candidates must fail closed until canonical payloads exist");
            };
            assert!(missing.contains(&AnchorIngredient::CanonicalGoalHash));
        }
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
        assert!(theorem.anchor.source_shape_hash.is_available());
        assert!(!theorem.anchor.canonical_goal_hash.is_available());
        assert!(!theorem.anchor.canonical_context_hash.is_available());
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
    fn generates_goal_bearing_algorithm_candidates_from_flow_sites() {
        let source = sample_source_id();
        let flow_output = sample_flow_output(source);
        let handoff = seed_handoff_with_sites(vec![
            flow_seed(
                source,
                CoreFormulaId::new(10),
                "program/0/contract/requires/0",
                "flow:0:requires:0",
                flow_site(ControlFlowObligationSiteKind::Requires, 0),
            ),
            flow_seed(
                source,
                CoreFormulaId::new(11),
                "program/0/contract/ensures/0",
                "flow:0:ensures:0",
                flow_site(ControlFlowObligationSiteKind::Ensures, 0),
            ),
            flow_seed(
                source,
                CoreFormulaId::new(12),
                "program/0/assertion/algorithm/0",
                "flow:0:assertion:algorithm:0",
                flow_site(ControlFlowObligationSiteKind::AlgorithmAssertion, 0).with_block(0),
            ),
            flow_seed(
                source,
                CoreFormulaId::new(13),
                "program/0/assertion/0",
                "flow:0:assertion:0",
                flow_site(ControlFlowObligationSiteKind::StatementAssertion, 1)
                    .with_statement(0)
                    .with_block(1),
            ),
            flow_seed(
                source,
                CoreFormulaId::new(14),
                "program/0/invariant/algorithm/0",
                "flow:0:invariant:algorithm:0",
                flow_site(ControlFlowObligationSiteKind::AlgorithmInvariant, 0).with_block(0),
            ),
            flow_seed(
                source,
                CoreFormulaId::new(15),
                "program/0/invariant/header/0",
                "flow:0:invariant:header:0",
                flow_site(ControlFlowObligationSiteKind::LoopInvariant, 1)
                    .with_loop(0)
                    .with_block(0),
            ),
            flow_seed(
                source,
                CoreFormulaId::new(16),
                "program/0/invariant/backedge/0",
                "flow:0:invariant:backedge:0",
                flow_site(ControlFlowObligationSiteKind::LoopInvariant, 2)
                    .with_loop(0)
                    .with_block(1),
            ),
            flow_seed(
                source,
                CoreFormulaId::new(17),
                "program/0/invariant/break/0",
                "flow:0:invariant:break:0",
                flow_site(ControlFlowObligationSiteKind::LoopInvariant, 3)
                    .with_loop(0)
                    .with_exit(0),
            ),
            flow_seed(
                source,
                CoreFormulaId::new(18),
                "program/0/invariant/continue/0",
                "flow:0:invariant:continue:0",
                flow_site(ControlFlowObligationSiteKind::LoopInvariant, 4)
                    .with_loop(0)
                    .with_exit(1),
            ),
        ]);

        let set = generate_with_flows(&handoff, &flow_output);

        assert_eq!(set.no_candidates(), []);
        assert_eq!(
            set.candidates()
                .iter()
                .map(|candidate| &candidate.kind)
                .collect::<Vec<_>>(),
            vec![
                &VcKind::AlgorithmPrecondition,
                &VcKind::AlgorithmPostcondition,
                &VcKind::AlgorithmAssertion,
                &VcKind::AlgorithmAssertion,
                &VcKind::LoopInvariant {
                    phase: LoopInvariantPhase::Entry
                },
                &VcKind::LoopInvariant {
                    phase: LoopInvariantPhase::Entry
                },
                &VcKind::LoopInvariant {
                    phase: LoopInvariantPhase::Preservation
                },
                &VcKind::LoopInvariant {
                    phase: LoopInvariantPhase::Break
                },
                &VcKind::LoopInvariant {
                    phase: LoopInvariantPhase::Continue
                },
            ]
        );
        for candidate in set.candidates() {
            assert!(candidate.anchor.source_shape_hash.is_available());
            assert!(!candidate.anchor.canonical_goal_hash.is_available());
            let AnchorCompleteness::Incomplete { missing } = &candidate.anchor.completeness else {
                panic!(
                    "algorithm core-goal candidates must fail closed until canonical payloads exist"
                );
            };
            assert!(missing.contains(&AnchorIngredient::CanonicalGoalHash));
        }
        let requires = &set.candidates()[0];
        assert_eq!(requires.seed_status, ObligationSeedStatus::Deferred);
        assert_eq!(requires.status, VcStatus::Open);
        assert_eq!(
            requires.owner,
            AnchorOwner::Algorithm(CoreAlgorithmId::new(0))
        );
        assert!(requires.local_context.policy_inputs().iter().any(|input| {
            input.key == PolicyKey::new("algorithm-site-site-kind")
                && input.value == PolicyValue::new("Requires")
        }));
        let statement_assertion = &set.candidates()[3];
        assert_policy_input(statement_assertion, "algorithm-site-statement", "0");
        assert_policy_input(statement_assertion, "algorithm-site-block", "1");
        let break_invariant = &set.candidates()[7];
        assert_policy_input(break_invariant, "algorithm-site-loop", "0");
        assert_policy_input(break_invariant, "algorithm-site-exit", "0");
        assert!(requires.provenance.iter().any(|provenance| {
            provenance.phase == VcProvenancePhase::Generator
                && provenance.key == VcText::new("task-7-algorithm-candidate")
        }));
    }

    #[test]
    fn algorithm_candidate_debug_rendering_is_deterministic() {
        let source = sample_source_id();
        let flow_output = sample_flow_output(source);
        let handoff = seed_handoff_with_sites(vec![
            flow_seed(
                source,
                CoreFormulaId::new(13),
                "program/0/assertion/0",
                "flow:0:assertion:0",
                flow_site(ControlFlowObligationSiteKind::StatementAssertion, 1)
                    .with_statement(0)
                    .with_block(1),
            ),
            flow_seed(
                source,
                CoreFormulaId::new(10),
                "program/0/contract/requires/0",
                "flow:0:requires:0",
                flow_site(ControlFlowObligationSiteKind::Requires, 0),
            ),
        ]);

        let first = generate_with_flows(&handoff, &flow_output);
        let second = generate_with_flows(&handoff, &flow_output);

        assert_eq!(first.debug_text(), second.debug_text());
        assert!(first.debug_text().contains("AlgorithmPrecondition"));
        assert!(first.debug_text().contains("AlgorithmAssertion"));
    }

    #[test]
    fn records_no_candidates_for_unavailable_algorithm_payloads() {
        let source = sample_source_id();
        let flow_output = sample_flow_output(source);
        let handoff = seed_handoff_with_sites(vec![
            (
                obligation_seed(
                    source,
                    ObligationSeedKind::AlgorithmContract,
                    Some(CoreFormulaId::new(0)),
                    "program/0/contract/requires/missing-site",
                    "flow:0:requires:missing-site",
                )
                .with_status(ObligationSeedStatus::Deferred)
                .into(),
                flow_origin(),
                None,
            ),
            flow_seed(
                source,
                CoreFormulaId::new(10),
                "program/0/contract/requires/valid",
                "flow:0:requires:valid",
                flow_site(ControlFlowObligationSiteKind::Requires, 0),
            ),
            flow_seed(
                source,
                CoreFormulaId::new(1),
                "program/0/contract/requires/missing-flow",
                "flow:0:requires:missing-flow",
                flow_site(ControlFlowObligationSiteKind::Requires, 1),
            ),
            flow_seed(
                source,
                CoreFormulaId::new(3),
                "program/0/invariant/incomplete-phase",
                "flow:0:invariant:incomplete-phase",
                flow_site(ControlFlowObligationSiteKind::LoopInvariant, 5).with_loop(0),
            ),
            (
                obligation_seed(
                    source,
                    ObligationSeedKind::AlgorithmTermination,
                    None,
                    "program/0/termination/measure/0",
                    "flow:0:termination:measure:0",
                )
                .with_status(ObligationSeedStatus::Deferred)
                .into(),
                flow_origin(),
                Some(flow_site(ControlFlowObligationSiteKind::TerminationMeasure, 2).into()),
            ),
            (
                obligation_seed(
                    source,
                    ObligationSeedKind::GhostErasure,
                    None,
                    "program/0/ghost/pick/0",
                    "flow:0:ghost:pick:0",
                )
                .with_status(ObligationSeedStatus::Deferred)
                .into(),
                flow_origin(),
                Some(
                    flow_site(ControlFlowObligationSiteKind::GhostPick, 3)
                        .with_local(0)
                        .into(),
                ),
            ),
            (
                obligation_seed(
                    source,
                    ObligationSeedKind::AlgorithmTermination,
                    None,
                    "program/0/termination/partial/0",
                    "flow:0:termination:partial:0",
                )
                .with_status(ObligationSeedStatus::Deferred)
                .into(),
                flow_origin(),
                Some(flow_site(ControlFlowObligationSiteKind::PartialTermination, 4).into()),
            ),
            (
                obligation_seed(
                    source,
                    ObligationSeedKind::GhostErasure,
                    None,
                    "program/0/ghost/assignment/0",
                    "flow:0:ghost:assignment:0",
                )
                .with_status(ObligationSeedStatus::Deferred)
                .into(),
                flow_origin(),
                Some(
                    flow_site(ControlFlowObligationSiteKind::GhostAssignment, 5)
                        .with_assignment_effect(0)
                        .into(),
                ),
            ),
            (
                obligation_seed(
                    source,
                    ObligationSeedKind::AlgorithmContract,
                    Some(CoreFormulaId::new(2)),
                    "program/0/contract/requires/existing-core",
                    "flow:0:requires:existing-core",
                )
                .into(),
                ObligationHandoffOrigin::ExistingCore {
                    seed: ObligationSeedId::new(9),
                },
                Some(flow_site(ControlFlowObligationSiteKind::Requires, 4).into()),
            ),
        ]);
        let empty_flow_output = ControlFlowOutput {
            flows: ControlFlowTable::new(),
            flow_map: std::collections::BTreeMap::new(),
        };

        let missing_flow_set = generate_with_flows(&handoff, &empty_flow_output);
        assert_eq!(missing_flow_set.candidates(), []);
        assert!(missing_flow_set.no_candidates().iter().any(|no_candidate| {
            matches!(
                &no_candidate.reason,
                SeedNoVcReason::DeferredExternal(reason)
                    if reason.as_str().contains("matching ControlFlowOutput")
            )
        }));

        let set = generate_with_flows(&handoff, &flow_output);
        assert_eq!(set.candidates().len(), 1);
        assert_eq!(set.candidates()[0].kind, VcKind::AlgorithmPrecondition);
        assert!(set.no_candidates().iter().any(|no_candidate| {
            matches!(
                &no_candidate.reason,
                SeedNoVcReason::DeferredExternal(reason)
                    if reason.as_str().contains("ControlFlowObligationSite")
            )
        }));
        assert!(set.no_candidates().iter().any(|no_candidate| {
            matches!(
                &no_candidate.reason,
                SeedNoVcReason::DeferredExternal(reason)
                    if reason.as_str().contains("explicit goal formula")
                        && reason.as_str().contains("TerminationMeasure")
            )
        }));
        assert!(set.no_candidates().iter().any(|no_candidate| {
            matches!(
                &no_candidate.reason,
                SeedNoVcReason::DeferredExternal(reason)
                    if reason.as_str().contains("explicit goal formula")
                        && reason.as_str().contains("GhostPick")
            )
        }));
        assert!(set.no_candidates().iter().any(|no_candidate| {
            matches!(
                &no_candidate.reason,
                SeedNoVcReason::DeferredExternal(reason)
                    if reason.as_str().contains("explicit goal formula")
                        && reason.as_str().contains("PartialTermination")
            )
        }));
        assert!(set.no_candidates().iter().any(|no_candidate| {
            matches!(
                &no_candidate.reason,
                SeedNoVcReason::DeferredExternal(reason)
                    if reason.as_str().contains("explicit goal formula")
                        && reason.as_str().contains("GhostAssignment")
            )
        }));
        assert!(set.no_candidates().iter().any(|no_candidate| {
            matches!(
                &no_candidate.reason,
                SeedNoVcReason::DeferredExternal(reason)
                    if reason.as_str().contains("LoopInvariant")
            )
        }));
        assert!(set.no_candidates().iter().any(|no_candidate| {
            matches!(
                &no_candidate.reason,
                SeedNoVcReason::DeferredExternal(reason)
                    if reason.as_str().contains("FlowDerived origin")
            )
        }));
    }

    #[test]
    fn seed_intake_marks_goal_bearing_deferred_flow_rows_eligible() {
        let source = sample_source_id();
        let handoff = seed_handoff_with_sites(vec![
            flow_seed(
                source,
                CoreFormulaId::new(0),
                "program/0/contract/requires/0",
                "flow:0:requires:0",
                flow_site(ControlFlowObligationSiteKind::Requires, 0),
            ),
            (
                obligation_seed(
                    source,
                    ObligationSeedKind::AlgorithmTermination,
                    None,
                    "program/0/termination/measure/0",
                    "flow:0:termination:measure:0",
                )
                .with_status(ObligationSeedStatus::Deferred)
                .into(),
                flow_origin(),
                Some(flow_site(ControlFlowObligationSiteKind::TerminationMeasure, 1).into()),
            ),
        ]);

        let intake = SeedIntakeTable::try_from_handoff(&handoff).expect("intake");

        assert!(matches!(
            intake.rows()[0].mapping,
            SeedIntakeMapping::EligibleOneVc { goal }
                if goal == CoreFormulaId::new(0)
        ));
        assert!(matches!(
            intake.rows()[1].mapping,
            SeedIntakeMapping::NoConcreteVc { .. }
        ));
    }

    #[test]
    fn rejects_flow_output_algorithm_mismatch() {
        let source = sample_source_id();
        let mismatched_flow_output =
            sample_flow_output_for_algorithm(source, CoreAlgorithmId::new(1));
        let handoff = seed_handoff_with_sites(vec![flow_seed(
            source,
            CoreFormulaId::new(10),
            "program/0/contract/requires/0",
            "flow:0:requires:0",
            flow_site(ControlFlowObligationSiteKind::Requires, 0),
        )]);

        let set = generate_with_flows(&handoff, &mismatched_flow_output);

        assert_eq!(set.candidates(), []);
        assert!(matches!(
            &set.no_candidates()[0].reason,
            SeedNoVcReason::DeferredExternal(reason)
                if reason.as_str().contains("matching ControlFlowOutput")
        ));
    }

    #[test]
    fn unavailable_algorithm_families_remain_documented_deferred() {
        let generator_doc = include_str!("../../../doc/design/mizar-vc/en/generator.md");
        let todo_doc = include_str!("../../../doc/design/mizar-vc/en/todo.md");

        for family in [
            "call-precondition",
            "branch",
            "match",
            "range-loop",
            "collection-loop",
            "Pick non-emptiness",
            "ghost-erasure",
        ] {
            assert!(
                generator_doc.contains(family),
                "generator.md must classify unavailable {family} payloads"
            );
            assert!(
                todo_doc.contains(family),
                "todo.md must keep unavailable {family} payloads deferred"
            );
        }
        assert!(generator_doc.contains("visible no-candidate/deferred"));
        assert!(todo_doc.contains("deferred/no-candidate"));
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
                flow_output: None,
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
                flow_output: None,
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

    #[test]
    fn normalizes_candidates_to_dense_vc_set_and_seed_accounting() {
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
            (
                obligation_seed(
                    source,
                    ObligationSeedKind::DefinitionCorrectness,
                    None,
                    "definition/missing-goal",
                    "definition:missing-goal",
                )
                .into(),
                ObligationHandoffOrigin::ExistingCore {
                    seed: ObligationSeedId::new(2),
                },
            ),
        ]);
        let candidates = generate(&handoff);

        let vc_set = normalize(&candidates);

        assert_eq!(vc_set.generated_formulas(), []);
        assert_eq!(
            vc_set.vcs().iter().map(|vc| vc.id).collect::<Vec<_>>(),
            vec![VcId::new(0), VcId::new(1)]
        );
        assert_eq!(
            vc_set.vcs().iter().map(|vc| &vc.kind).collect::<Vec<_>>(),
            vec![&VcKind::TheoremProofStep, &VcKind::GeneratedSethood]
        );
        assert!(
            vc_set.vcs()[0]
                .provenance
                .iter()
                .any(|provenance| provenance.phase == VcProvenancePhase::Normalization)
        );
        assert_eq!(
            vc_set
                .seed_accounting()
                .iter()
                .map(|row| row.handoff)
                .collect::<Vec<_>>(),
            vec![
                ObligationHandoffId::new(0),
                ObligationHandoffId::new(1),
                ObligationHandoffId::new(2),
            ]
        );
        assert!(matches!(
            vc_set.seed_accounting_for(ObligationHandoffId::new(0)),
            Some(SeedAccounting {
                mapping: SeedVcMapping::One { vc },
                ..
            }) if *vc == VcId::new(0)
        ));
        assert!(matches!(
            vc_set.seed_accounting_for(ObligationHandoffId::new(1)),
            Some(SeedAccounting {
                mapping: SeedVcMapping::One { vc },
                ..
            }) if *vc == VcId::new(1)
        ));
        assert!(matches!(
            vc_set.seed_accounting_for(ObligationHandoffId::new(2)),
            Some(SeedAccounting {
                mapping: SeedVcMapping::NoConcreteVc {
                    reason: SeedNoVcReason::MissingGoal(reason),
                },
                ..
            }) if reason.as_str().contains("active seed has no goal")
        ));
    }

    #[test]
    fn normalization_uses_documented_kind_rank_before_candidate_sort_key() {
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
                    ObligationSeedKind::TheoremProof,
                    Some(CoreFormulaId::new(1)),
                    "proof/step/0",
                    "theorem:sample:proof-step:0",
                )
                .into(),
                ObligationHandoffOrigin::ExistingCore {
                    seed: ObligationSeedId::new(1),
                },
            ),
        ]);
        let mut candidates = generate(&handoff);
        candidates
            .candidates
            .iter_mut()
            .find(|candidate| candidate.kind == VcKind::TheoremProofStep)
            .expect("theorem candidate")
            .sort_key = CanonicalSortKey::new("zzz-theorem-tiebreak");
        candidates
            .candidates
            .iter_mut()
            .find(|candidate| candidate.kind == VcKind::GeneratedSethood)
            .expect("sethood candidate")
            .sort_key = CanonicalSortKey::new("aaa-sethood-tiebreak");

        let vc_set = normalize(&candidates);

        assert_eq!(vc_set.vcs()[0].kind, VcKind::TheoremProofStep);
        assert_eq!(vc_set.vcs()[1].kind, VcKind::GeneratedSethood);
    }

    #[test]
    fn documented_kind_rank_covers_all_task_eight_variants() {
        let kinds = vec![
            VcKind::TheoremProofStep,
            VcKind::TerminalProofGoal,
            VcKind::DefinitionCorrectness,
            VcKind::RegistrationStyleCorrectness {
                style: RegistrationCorrectnessKind::Registration,
            },
            VcKind::RegistrationStyleCorrectness {
                style: RegistrationCorrectnessKind::Redefinition,
            },
            VcKind::RegistrationStyleCorrectness {
                style: RegistrationCorrectnessKind::Reduction,
            },
            VcKind::RegistrationStyleCorrectness {
                style: RegistrationCorrectnessKind::ExplicitCoreSeed,
            },
            VcKind::CheckerInitial,
            VcKind::GeneratedNonEmptiness,
            VcKind::GeneratedSethood,
            VcKind::FraenkelMembershipAxiom,
            VcKind::AlgorithmPrecondition,
            VcKind::AlgorithmPostcondition,
            VcKind::CallPrecondition,
            VcKind::AlgorithmAssertion,
            VcKind::LoopInvariant {
                phase: LoopInvariantPhase::Entry,
            },
            VcKind::LoopInvariant {
                phase: LoopInvariantPhase::Preservation,
            },
            VcKind::LoopInvariant {
                phase: LoopInvariantPhase::Break,
            },
            VcKind::LoopInvariant {
                phase: LoopInvariantPhase::Continue,
            },
            VcKind::LoopInvariant {
                phase: LoopInvariantPhase::Exit,
            },
            VcKind::RangeLoop {
                obligation: RangeLoopObligation::PositiveStep,
            },
            VcKind::RangeLoop {
                obligation: RangeLoopObligation::RangeBound,
            },
            VcKind::RangeLoop {
                obligation: RangeLoopObligation::HiddenIndex,
            },
            VcKind::CollectionLoop {
                obligation: CollectionLoopObligation::Finiteness,
            },
            VcKind::CollectionLoop {
                obligation: CollectionLoopObligation::OrderIndependence,
            },
            VcKind::Termination,
            VcKind::PartialTermination,
            VcKind::GhostErasureSafety,
            VcKind::PolicyDeferredTraceability,
        ];

        for pair in kinds.windows(2) {
            assert!(
                kind_classification_rank(&pair[0]) < kind_classification_rank(&pair[1]),
                "{:?} must rank before {:?}",
                pair[0],
                pair[1]
            );
        }
    }

    #[test]
    fn normalization_preserves_deferred_flow_status_accounting() {
        let source = sample_source_id();
        let flow_output = sample_flow_output(source);
        let handoff = seed_handoff_with_sites(vec![flow_seed(
            source,
            CoreFormulaId::new(10),
            "program/0/contract/requires/0",
            "flow:0:requires:0",
            flow_site(ControlFlowObligationSiteKind::Requires, 0),
        )]);
        let candidates = generate_with_flows(&handoff, &flow_output);

        let vc_set = normalize(&candidates);

        assert_eq!(vc_set.vcs().len(), 1);
        assert_eq!(vc_set.vcs()[0].kind, VcKind::AlgorithmPrecondition);
        assert_eq!(vc_set.vcs()[0].status, VcStatus::Open);
        assert!(matches!(
            vc_set.seed_accounting_for(ObligationHandoffId::new(0)),
            Some(SeedAccounting {
                seed_status: ObligationSeedStatus::Deferred,
                mapping: SeedVcMapping::One { vc },
                ..
            }) if *vc == VcId::new(0)
        ));
    }

    #[test]
    fn normalization_preserves_non_open_status_without_later_phase_outputs() {
        let source = sample_source_id();
        let handoff = seed_handoff(vec![(
            obligation_seed(
                source,
                ObligationSeedKind::TheoremProof,
                Some(CoreFormulaId::new(0)),
                "proof/step/0",
                "theorem:sample:proof-step:0",
            )
            .into(),
            ObligationHandoffOrigin::ExistingCore {
                seed: ObligationSeedId::new(0),
            },
        )]);
        let mut candidates = generate(&handoff);
        candidates.candidates[0].status = VcStatus::PolicyOpen {
            policy: PolicyKey::new("task-8-preserve-status"),
        };

        let vc_set = normalize(&candidates);
        let vc = &vc_set.vcs()[0];

        assert_eq!(
            vc.status,
            VcStatus::PolicyOpen {
                policy: PolicyKey::new("task-8-preserve-status")
            }
        );
        assert!(
            vc.provenance
                .iter()
                .any(|provenance| provenance.phase == VcProvenancePhase::Normalization)
        );
        assert!(!vc.provenance.iter().any(|provenance| matches!(
            provenance.phase,
            VcProvenancePhase::StatusPolicy
                | VcProvenancePhase::Discharge
                | VcProvenancePhase::DependencySlice
        )));
        assert_eq!(vc_set.generated_formulas(), []);
    }

    #[test]
    fn normalization_debug_rendering_is_deterministic_and_fails_closed_for_core_goal_anchors() {
        let source = sample_source_id();
        let handoff = seed_handoff(vec![(
            obligation_seed(
                source,
                ObligationSeedKind::TheoremProof,
                Some(CoreFormulaId::new(0)),
                "proof/step/0",
                "theorem:sample:proof-step:0",
            )
            .with_context(vec![CoreFormulaId::new(1)])
            .with_label("A1")
            .into(),
            ObligationHandoffOrigin::ExistingCore {
                seed: ObligationSeedId::new(0),
            },
        )]);

        let first = normalize(&generate(&handoff));
        let second = normalize(&generate(&handoff));

        assert_eq!(first.debug_text(), second.debug_text());
        assert!(first.debug_text().contains("vc-set-debug-v1"));
        assert_eq!(first.vcs()[0].local_context.entries().len(), 1);
        let anchor = &first.vcs()[0].anchor;
        assert!(matches!(
            anchor.completeness,
            AnchorCompleteness::Incomplete { .. }
        ));
        assert!(anchor.source_shape_hash.is_available());
        assert!(!anchor.canonical_goal_hash.is_available());
        assert!(!anchor.canonical_context_hash.is_available());
    }

    #[test]
    fn normalization_rejects_duplicate_candidate_sort_key() {
        let source = sample_source_id();
        let handoff = seed_handoff(vec![
            (
                obligation_seed(
                    source,
                    ObligationSeedKind::TheoremProof,
                    Some(CoreFormulaId::new(0)),
                    "proof/step/0",
                    "theorem:sample:proof-step:0",
                )
                .into(),
                ObligationHandoffOrigin::ExistingCore {
                    seed: ObligationSeedId::new(0),
                },
            ),
            (
                obligation_seed(
                    source,
                    ObligationSeedKind::GeneratedSethood,
                    Some(CoreFormulaId::new(1)),
                    "generated/sethood/0",
                    "generated:sethood:0",
                )
                .into(),
                ObligationHandoffOrigin::ExistingCore {
                    seed: ObligationSeedId::new(1),
                },
            ),
        ]);
        let mut candidates = generate(&handoff);
        candidates.candidates[1].sort_key = candidates.candidates[0].sort_key.clone();

        assert!(matches!(
            try_normalize(&candidates),
            Err(GeneratorError::DuplicateCandidateSortKey { .. })
        ));
    }

    #[test]
    fn normalization_rejects_duplicate_seed_output() {
        let source = sample_source_id();
        let handoff = seed_handoff(vec![(
            obligation_seed(
                source,
                ObligationSeedKind::TheoremProof,
                Some(CoreFormulaId::new(0)),
                "proof/step/0",
                "theorem:sample:proof-step:0",
            )
            .into(),
            ObligationHandoffOrigin::ExistingCore {
                seed: ObligationSeedId::new(0),
            },
        )]);
        let mut candidates = generate(&handoff);
        let candidate = candidates.candidates()[0].clone();
        candidates.no_candidates.push(CoreGenerationNoCandidate {
            handoff: candidate.handoff,
            origin: candidate.origin,
            seed_status: candidate.seed_status,
            reason: SeedNoVcReason::DeferredExternal(VcText::new("duplicate test row")),
        });

        assert!(matches!(
            try_normalize(&candidates),
            Err(GeneratorError::DuplicateSeedOutput { handoff })
                if handoff == ObligationHandoffId::new(0)
        ));
    }

    #[test]
    fn normalization_keeps_existing_expanded_mapping_contract_validated() {
        let source = sample_source_id();
        let handoff = seed_handoff(vec![(
            obligation_seed(
                source,
                ObligationSeedKind::TheoremProof,
                Some(CoreFormulaId::new(0)),
                "proof/step/0",
                "theorem:sample:proof-step:0",
            )
            .into(),
            ObligationHandoffOrigin::ExistingCore {
                seed: ObligationSeedId::new(0),
            },
        )]);
        let vc_set = normalize(&generate(&handoff));
        let mut parts = VcSetParts {
            schema_version: vc_set.schema_version().clone(),
            snapshot: vc_set.snapshot(),
            source: vc_set.source(),
            module: vc_set.module().clone(),
            generated_formulas: vc_set.generated_formulas().to_vec(),
            vcs: vc_set.vcs().to_vec(),
            seed_accounting: vc_set.seed_accounting().to_vec(),
        };
        parts.seed_accounting[0].mapping = SeedVcMapping::Expanded {
            vcs: vec![ExpandedVcRef {
                expansion_index: 0,
                vc: VcId::new(0),
            }],
            expansion_schema: ExpansionSchemaVersion::new("task-8-validation-test"),
        };

        VcSet::try_new(parts).expect("expanded mapping remains validated by VcSet");
    }

    fn generate(handoff: &ObligationSeedHandoff) -> CoreGenerationCandidateSet {
        let intake = SeedIntakeTable::try_from_handoff(handoff).expect("intake");
        CoreGenerationCandidateSet::try_from_seed_intake(CoreGenerationInput {
            schema_version: &GenerationSchemaVersion::new("generator-task-6-test"),
            module: &VcModuleRef::new("sample"),
            intake: &intake,
            handoff,
            flow_output: None,
        })
        .expect("generation candidates")
    }

    fn normalize(candidates: &CoreGenerationCandidateSet) -> VcSet {
        try_normalize(candidates).expect("normalized vc set")
    }

    fn try_normalize(candidates: &CoreGenerationCandidateSet) -> Result<VcSet, GeneratorError> {
        let source = sample_source_id();
        CoreGenerationCandidateSet::try_normalize(VcNormalizationInput {
            schema_version: &VcSchemaVersion::new("vc-task-8-test"),
            snapshot: sample_snapshot_id(),
            source,
            candidates,
        })
    }

    fn generate_with_flows(
        handoff: &ObligationSeedHandoff,
        flow_output: &ControlFlowOutput,
    ) -> CoreGenerationCandidateSet {
        let intake = SeedIntakeTable::try_from_handoff(handoff).expect("intake");
        CoreGenerationCandidateSet::try_from_seed_intake(CoreGenerationInput {
            schema_version: &GenerationSchemaVersion::new("generator-task-7-test"),
            module: &VcModuleRef::new("sample"),
            intake: &intake,
            handoff,
            flow_output: Some(flow_output),
        })
        .expect("generation candidates")
    }

    fn assert_policy_input(candidate: &CoreGenerationCandidate, key: &str, value: &str) {
        assert!(
            candidate
                .local_context
                .policy_inputs()
                .iter()
                .any(|input| input.key == PolicyKey::new(key)
                    && input.value == PolicyValue::new(value)),
            "missing policy input {key}={value} in {:?}",
            candidate.local_context.policy_inputs()
        );
    }

    fn seed_handoff(
        entries: Vec<(ObligationSeed, ObligationHandoffOrigin)>,
    ) -> ObligationSeedHandoff {
        seed_handoff_with_sites(
            entries
                .into_iter()
                .map(|(seed, origin)| (seed, origin, None))
                .collect(),
        )
    }

    fn seed_handoff_with_sites(
        entries: Vec<(
            ObligationSeed,
            ObligationHandoffOrigin,
            Option<ControlFlowObligationSite>,
        )>,
    ) -> ObligationSeedHandoff {
        let mut table = ObligationHandoffTable::new();
        let mut source_map = std::collections::BTreeMap::new();

        for (seed, origin, flow_site) in entries {
            let source = seed.source.clone();
            let id = table.insert(ObligationHandoffEntry {
                seed,
                origin,
                flow_site,
            });
            source_map.insert(id, source);
        }

        ObligationSeedHandoff {
            entries: table,
            source_map,
        }
    }

    fn flow_seed(
        source: mizar_session::SourceId,
        goal: CoreFormulaId,
        local_path: &str,
        semantic_origin: &str,
        site: impl Into<ControlFlowObligationSite>,
    ) -> (
        ObligationSeed,
        ObligationHandoffOrigin,
        Option<ControlFlowObligationSite>,
    ) {
        (
            obligation_seed(
                source,
                ObligationSeedKind::AlgorithmContract,
                Some(goal),
                local_path,
                semantic_origin,
            )
            .with_status(ObligationSeedStatus::Deferred)
            .into(),
            flow_origin(),
            Some(site.into()),
        )
    }

    fn flow_origin() -> ObligationHandoffOrigin {
        ObligationHandoffOrigin::FlowDerived {
            flow: ControlFlowId::new(0),
            algorithm: CoreAlgorithmId::new(0),
        }
    }

    fn flow_site(kind: ControlFlowObligationSiteKind, ordinal: usize) -> FlowSiteBuilder {
        FlowSiteBuilder {
            site: ControlFlowObligationSite {
                kind,
                ordinal,
                statement: None,
                block: None,
                loop_id: None,
                exit: None,
                local: None,
                assignment_effect: None,
            },
        }
    }

    struct FlowSiteBuilder {
        site: ControlFlowObligationSite,
    }

    impl FlowSiteBuilder {
        fn with_statement(mut self, statement: usize) -> Self {
            self.site.statement = Some(mizar_core::core_ir::CoreAlgorithmStmtId::new(statement));
            self
        }

        fn with_block(mut self, block: usize) -> Self {
            self.site.block = Some(BasicBlockId::new(block));
            self
        }

        fn with_loop(mut self, loop_id: usize) -> Self {
            self.site.loop_id = Some(LoopId::new(loop_id));
            self
        }

        fn with_exit(mut self, exit: usize) -> Self {
            self.site.exit = Some(ControlFlowExitId::new(exit));
            self
        }

        fn with_local(mut self, local: usize) -> Self {
            self.site.local = Some(LocalId::new(local));
            self
        }

        fn with_assignment_effect(mut self, effect: usize) -> Self {
            self.site.assignment_effect =
                Some(mizar_core::control_flow::AssignmentEffectId::new(effect));
            self
        }
    }

    impl From<FlowSiteBuilder> for ControlFlowObligationSite {
        fn from(builder: FlowSiteBuilder) -> Self {
            builder.site
        }
    }

    fn sample_flow_output(source: mizar_session::SourceId) -> ControlFlowOutput {
        sample_flow_output_for_algorithm(source, CoreAlgorithmId::new(0))
    }

    fn sample_flow_output_for_algorithm(
        source: mizar_session::SourceId,
        algorithm: CoreAlgorithmId,
    ) -> ControlFlowOutput {
        let mut loops = ControlFlowLoopTable::new();
        let loop_id = loops.insert(ControlFlowLoop {
            algorithm,
            header: BasicBlockId::new(0),
            body: BasicBlockId::new(1),
            exit: BasicBlockId::new(2),
            condition: CoreFormulaId::new(90),
            invariants: vec![CoreFormulaId::new(91)],
            decreasing: Vec::new(),
            source: source_ref(source),
        });
        assert_eq!(loop_id, LoopId::new(0));

        let mut exits = ControlFlowExitTable::new();
        let break_exit = exits.insert(ControlFlowExit {
            algorithm,
            statement: None,
            from: BasicBlockId::new(3),
            target: Some(BasicBlockId::new(2)),
            kind: ControlFlowExitKind::Break { loop_id },
            source: source_ref(source),
        });
        let continue_exit = exits.insert(ControlFlowExit {
            algorithm,
            statement: None,
            from: BasicBlockId::new(4),
            target: Some(BasicBlockId::new(0)),
            kind: ControlFlowExitKind::Continue { loop_id },
            source: source_ref(source),
        });
        assert_eq!(break_exit, ControlFlowExitId::new(0));
        assert_eq!(continue_exit, ControlFlowExitId::new(1));

        let mut flows = ControlFlowTable::new();
        let flow_id = flows.insert(ControlFlowIr {
            algorithm,
            item: CoreItemId::new(0),
            symbol: sample_symbol("Algorithm"),
            entry: BasicBlockId::new(0),
            blocks: ControlFlowBlockTable::new(),
            locals: ControlFlowLocalTable::new(),
            contexts: ProgramContextTable::new(),
            context_facts: ContextFactTable::new(),
            assignment_effects: mizar_core::control_flow::AssignmentEffectTable::new(),
            call_sites: CallSiteTable::new(),
            contracts: ControlFlowContractSet {
                requires: vec![ContractSite {
                    kind: ContractSiteKind::Requires,
                    formula: CoreFormulaId::new(10),
                    placement: ContractSitePlacement::Entry {
                        block: BasicBlockId::new(0),
                        context: ProgramContextId::new(0),
                    },
                    source: source_ref(source),
                }],
                ensures: vec![ContractSite {
                    kind: ContractSiteKind::Ensures,
                    formula: CoreFormulaId::new(11),
                    placement: ContractSitePlacement::Return {
                        block: BasicBlockId::new(2),
                        exit: ControlFlowExitId::new(0),
                    },
                    source: source_ref(source),
                }],
                calls: Vec::new(),
                assertions: vec![
                    AssertionSite {
                        formula: CoreFormulaId::new(12),
                        placement: AssertionPlacement::AlgorithmContract {
                            block: BasicBlockId::new(0),
                            context: ProgramContextId::new(0),
                        },
                        source: source_ref(source),
                    },
                    AssertionSite {
                        formula: CoreFormulaId::new(13),
                        placement: AssertionPlacement::Statement {
                            statement: mizar_core::core_ir::CoreAlgorithmStmtId::new(0),
                            block: BasicBlockId::new(1),
                            successor_context: ProgramContextId::new(0),
                        },
                        source: source_ref(source),
                    },
                ],
                loop_invariants: vec![
                    LoopInvariantSite {
                        formula: CoreFormulaId::new(14),
                        placement: LoopInvariantPlacement::AlgorithmContract {
                            block: BasicBlockId::new(0),
                            context: ProgramContextId::new(0),
                        },
                        source: source_ref(source),
                    },
                    LoopInvariantSite {
                        formula: CoreFormulaId::new(15),
                        placement: LoopInvariantPlacement::Header {
                            loop_id: LoopId::new(0),
                            block: BasicBlockId::new(0),
                        },
                        source: source_ref(source),
                    },
                    LoopInvariantSite {
                        formula: CoreFormulaId::new(16),
                        placement: LoopInvariantPlacement::NormalBackedge {
                            loop_id: LoopId::new(0),
                            from: BasicBlockId::new(1),
                            to: BasicBlockId::new(0),
                        },
                        source: source_ref(source),
                    },
                    LoopInvariantSite {
                        formula: CoreFormulaId::new(17),
                        placement: LoopInvariantPlacement::BreakExit {
                            loop_id: LoopId::new(0),
                            exit: ControlFlowExitId::new(0),
                        },
                        source: source_ref(source),
                    },
                    LoopInvariantSite {
                        formula: CoreFormulaId::new(18),
                        placement: LoopInvariantPlacement::ContinueExit {
                            loop_id: LoopId::new(0),
                            exit: ControlFlowExitId::new(1),
                        },
                        source: source_ref(source),
                    },
                ],
                decreasing: Vec::new(),
            },
            loops,
            exits,
            ghost_effects: ControlFlowGhostTable::default(),
            termination: mizar_core::control_flow::ControlFlowTerminationPlan::default(),
            source_map: ControlFlowSourceMap::default(),
            diagnostics: ControlFlowDiagnosticTable::new(),
        });
        assert_eq!(flow_id, ControlFlowId::new(0));

        ControlFlowOutput {
            flows,
            flow_map: std::collections::BTreeMap::from([(algorithm, flow_id)]),
        }
    }

    fn sample_symbol(name: &str) -> SymbolId {
        let module = ModuleId::new(PackageId::new("pkg"), ModulePath::new("vc_fixture"));
        SymbolId::new(
            module,
            LocalSymbolId::new(name),
            FullyQualifiedName::new(format!("pkg::vc_fixture::{name}")),
        )
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
        InMemorySessionIdAllocator::new()
            .next_source_id(sample_snapshot_id())
            .expect("source id")
    }

    fn sample_snapshot_id() -> BuildSnapshotId {
        BuildSnapshotId::from_published_schema_str(
            "mizar-session-build-snapshot-v1:\
             2222222222222222222222222222222222222222222222222222222222222222",
        )
        .expect("snapshot id")
    }

    #[test]
    fn generated_source_refs_remain_unranged() {
        let source = generated_source_ref();
        assert_eq!(source_range(&source), None);
    }
}
