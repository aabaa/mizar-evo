//! Deterministic pre-ATP discharge for canonical verification conditions.
//!
//! This module implements the task-11 slice specified in
//! [discharge.md](../../../doc/design/mizar-vc/en/discharge.md): a
//! prover-independent, fail-closed discharge pass over validated `VcSet` data.

use crate::vc_ir::{
    AnchorUnavailableReason, ComputationHint, ContextEntry, ContextEntryId, ContextEntryKind,
    DefinitionOpacityOverride, DefinitionUnfoldRequest, DischargeEvidenceRef, HashMarker,
    LocalContext, PolicyKey, PolicyValue, PremiseRef, ProofHintKey, VcFormulaRef,
    VcGeneratedFormulaId, VcGeneratedFormulaShape, VcId, VcIr, VcIrError, VcProvenance,
    VcProvenancePhase, VcSet, VcSetParts, VcStatus, VcText, stable_fingerprint_hash,
};
use mizar_core::core_ir::CoreDefinitionId;
use std::{collections::BTreeSet, fmt::Write as _};

pub const DEFAULT_COMPUTATION_STEP_LIMIT: u32 = 64;
pub const DEFAULT_COMPUTATION_LIMIT_POLICY: &str = "task-11-computation-step-limit";
pub const DEFINITIONAL_REDUCTION_POLICY: &str = "task-11-definitional-reduction";
pub const DEFINITIONAL_REDUCTION_ALLOW: &str = "allow";

#[derive(Debug, Clone, Copy)]
pub struct DischargeInput<'a> {
    pub vc_set: &'a VcSet,
    pub policy: &'a DischargePolicy,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DischargePolicy {
    pub computation_limit: ComputationLimit,
}

impl Default for DischargePolicy {
    fn default() -> Self {
        Self {
            computation_limit: ComputationLimit {
                policy: PolicyKey::new(DEFAULT_COMPUTATION_LIMIT_POLICY),
                max_steps: DEFAULT_COMPUTATION_STEP_LIMIT,
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComputationLimit {
    pub policy: PolicyKey,
    pub max_steps: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DischargeOutput {
    vc_set: VcSet,
    evidence_records: Vec<DischargeEvidenceRecord>,
    explanations: Vec<DischargeExplanation>,
}

impl DischargeOutput {
    pub const fn vc_set(&self) -> &VcSet {
        &self.vc_set
    }

    pub fn evidence_records(&self) -> &[DischargeEvidenceRecord] {
        &self.evidence_records
    }

    pub fn explanations(&self) -> &[DischargeExplanation] {
        &self.explanations
    }

    pub fn debug_text(&self) -> String {
        let mut output = String::from("discharge-output-debug-v1\n");
        output.push_str(&self.vc_set.debug_text());
        writeln!(&mut output, "[discharge-evidence]").expect("write string");
        for record in &self.evidence_records {
            writeln!(
                &mut output,
                "evidence {:?}: source={:?}; rule={:?}; rule-name={:?}; \
                 status-evidence={:?}; inputs={:?}; replay={:?}",
                record.vc,
                record.source,
                record.rule,
                record.rule_name,
                record.status_evidence,
                record.inputs,
                record.replay
            )
            .expect("write string");
        }
        writeln!(&mut output, "[discharge-explanations]").expect("write string");
        for explanation in &self.explanations {
            writeln!(
                &mut output,
                "explanation {:?}: category={:?}; rule={:?}; detail={:?}",
                explanation.vc, explanation.category, explanation.rule, explanation.detail
            )
            .expect("write string");
        }
        output
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DischargeEvidenceRecord {
    pub vc: VcId,
    pub source: DischargeEvidenceSource,
    pub rule: Option<DischargeRule>,
    pub rule_name: VcText,
    pub status_evidence: DischargeEvidenceRef,
    pub inputs: DischargeEvidenceInputs,
    pub replay: DischargeEvidenceReplay,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum DischargeEvidenceSource {
    NewlyProduced,
    PreExistingStatus,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DischargeEvidenceInputs {
    pub goal: VcFormulaRef,
    pub local_context: Vec<ContextEntryId>,
    pub premises: Vec<PremiseRef>,
    pub generated_formulas: Vec<VcGeneratedFormulaId>,
    pub policy_inputs: Vec<DischargePolicyEvidence>,
    pub unfold_requests: Vec<DefinitionUnfoldRequest>,
    pub computation: Option<DischargeComputationEvidence>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct DischargePolicyEvidence {
    pub key: PolicyKey,
    pub value: PolicyValue,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DischargeComputationEvidence {
    pub hint: ComputationHint,
    pub active_policy: PolicyKey,
    pub max_steps: u32,
    pub requested_steps: Option<u32>,
    pub timeout: Option<ProofHintKey>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum DischargeEvidenceReplay {
    ExplicitInputRefs,
    PreservedStatusOnly { reason: VcText },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DischargeExplanation {
    pub vc: VcId,
    pub category: DischargeExplanationCategory,
    pub rule: Option<DischargeRule>,
    pub detail: VcText,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum DischargeExplanationCategory {
    Discharged,
    NeedsAtp,
    PolicyOpen,
    AssumedByPolicy,
    SkippedInvalidInput,
    DeferredExternal,
    Error,
    MissingTrace,
    LimitExceeded,
    UnsupportedRule,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum DischargeRule {
    GeneratedTautology,
    LocalContradiction,
    DirectLocalFact,
    TracePremise,
    DefinitionalReduction,
    BoundedComputation,
}

pub fn try_discharge(input: DischargeInput<'_>) -> Result<DischargeOutput, VcIrError> {
    let mut vcs = input.vc_set.vcs().to_vec();
    let mut evidence_records = Vec::new();
    let mut explanations = Vec::with_capacity(vcs.len());

    for vc in &mut vcs {
        let original = vc.clone();
        let decision = select_discharge(&original, input.vc_set, input.policy);
        explanations.push(explanation_for(vc.id, &decision));
        apply_decision(vc, input.vc_set, &decision);
        if let Some(record) =
            evidence_record_for(&original, vc, input.vc_set, input.policy, &decision)
        {
            evidence_records.push(record);
        }
    }

    let vc_set = VcSet::try_new(VcSetParts {
        schema_version: input.vc_set.schema_version().clone(),
        snapshot: input.vc_set.snapshot(),
        source: input.vc_set.source(),
        module: input.vc_set.module().clone(),
        generated_formulas: input.vc_set.generated_formulas().to_vec(),
        vcs,
        seed_accounting: input.vc_set.seed_accounting().to_vec(),
    })?;

    Ok(DischargeOutput {
        vc_set,
        evidence_records,
        explanations,
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum DischargeDecision {
    Discharged {
        rule: DischargeRule,
        detail: VcText,
    },
    NeedsAtp {
        category: DischargeExplanationCategory,
        rule: Option<DischargeRule>,
        detail: VcText,
    },
    Preserved {
        category: DischargeExplanationCategory,
        detail: VcText,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FormulaTruth {
    True,
    False,
    Unknown,
}

impl FormulaTruth {
    const fn negate(self) -> Self {
        match self {
            Self::True => Self::False,
            Self::False => Self::True,
            Self::Unknown => Self::Unknown,
        }
    }
}

fn select_discharge(vc: &VcIr, vc_set: &VcSet, policy: &DischargePolicy) -> DischargeDecision {
    match &vc.status {
        VcStatus::Open | VcStatus::NeedsAtp => select_open_or_needs_atp(vc, vc_set, policy),
        VcStatus::Discharged { .. } => DischargeDecision::Preserved {
            category: DischargeExplanationCategory::Discharged,
            detail: VcText::new("preserved existing discharge evidence"),
        },
        VcStatus::PolicyOpen { policy } => DischargeDecision::Preserved {
            category: DischargeExplanationCategory::PolicyOpen,
            detail: VcText::new(format!("policy-open:{}", policy.as_str())),
        },
        VcStatus::AssumedByPolicy { policy, .. } => DischargeDecision::Preserved {
            category: DischargeExplanationCategory::AssumedByPolicy,
            detail: VcText::new(format!("assumed-by-policy:{}", policy.as_str())),
        },
        VcStatus::SkippedDueToInvalidInput { reason } => DischargeDecision::Preserved {
            category: DischargeExplanationCategory::SkippedInvalidInput,
            detail: reason.clone(),
        },
        VcStatus::DeferredExternal { reason } => DischargeDecision::Preserved {
            category: DischargeExplanationCategory::DeferredExternal,
            detail: reason.clone(),
        },
        VcStatus::Error { diagnostic } => DischargeDecision::Preserved {
            category: DischargeExplanationCategory::Error,
            detail: VcText::new(format!("diagnostic:{diagnostic:?}")),
        },
    }
}

fn select_open_or_needs_atp(
    vc: &VcIr,
    vc_set: &VcSet,
    policy: &DischargePolicy,
) -> DischargeDecision {
    if formula_truth(vc_set, vc.goal) == FormulaTruth::True {
        return discharged(
            DischargeRule::GeneratedTautology,
            "goal is a task-11 syntactic tautology",
        );
    }

    if has_local_contradiction(vc, vc_set) {
        return discharged(
            DischargeRule::LocalContradiction,
            "local context or generated premise is explicitly contradictory",
        );
    }

    if has_trace_premise(vc) && has_goal_linked_support(vc, vc_set) {
        return discharged(
            DischargeRule::TracePremise,
            "cluster, registration, or reduction trace premise is explicit and goal-linked",
        );
    }

    if definitional_reduction_allowed(vc, vc_set) {
        return discharged(
            DischargeRule::DefinitionalReduction,
            "definition unfold requests are explicit, policy-allowed, and goal-linked",
        );
    }

    if let Some(decision) = computation_decision(vc, vc_set, policy) {
        return decision;
    }

    if has_direct_local_fact(vc, vc_set) {
        return discharged(
            DischargeRule::DirectLocalFact,
            "goal matches an explicit local fact",
        );
    }

    if has_trace_premise(vc) {
        return DischargeDecision::NeedsAtp {
            category: DischargeExplanationCategory::MissingTrace,
            rule: Some(DischargeRule::TracePremise),
            detail: VcText::new("trace premise has no explicit goal-linked fact"),
        };
    }

    if has_missing_trace_marker(vc) {
        return DischargeDecision::NeedsAtp {
            category: DischargeExplanationCategory::MissingTrace,
            rule: Some(DischargeRule::TracePremise),
            detail: VcText::new("trace premise is conservative-unknown"),
        };
    }

    DischargeDecision::NeedsAtp {
        category: DischargeExplanationCategory::UnsupportedRule,
        rule: None,
        detail: VcText::new("no task-11 deterministic discharge rule matched"),
    }
}

fn discharged(rule: DischargeRule, detail: &str) -> DischargeDecision {
    DischargeDecision::Discharged {
        rule,
        detail: VcText::new(detail),
    }
}

fn computation_decision(
    vc: &VcIr,
    vc_set: &VcSet,
    policy: &DischargePolicy,
) -> Option<DischargeDecision> {
    let hint = vc.proof_hint.as_ref()?;
    let computation = hint.computation.as_ref()?;

    if !supported_computation_hint(computation, &policy.computation_limit.policy) {
        return Some(DischargeDecision::NeedsAtp {
            category: DischargeExplanationCategory::UnsupportedRule,
            rule: Some(DischargeRule::BoundedComputation),
            detail: VcText::new("computation hint is not supported by the active limit policy"),
        });
    }

    let Some(steps) = hint.max_axioms else {
        return Some(DischargeDecision::NeedsAtp {
            category: DischargeExplanationCategory::UnsupportedRule,
            rule: Some(DischargeRule::BoundedComputation),
            detail: VcText::new("computation hint has no explicit step budget"),
        });
    };

    if steps > policy.computation_limit.max_steps {
        return Some(DischargeDecision::NeedsAtp {
            category: DischargeExplanationCategory::LimitExceeded,
            rule: Some(DischargeRule::BoundedComputation),
            detail: VcText::new(format!(
                "requested {steps} steps exceeds {}:{}",
                policy.computation_limit.policy.as_str(),
                policy.computation_limit.max_steps
            )),
        });
    }

    if !has_goal_linked_support(vc, vc_set) {
        return Some(DischargeDecision::NeedsAtp {
            category: DischargeExplanationCategory::UnsupportedRule,
            rule: Some(DischargeRule::BoundedComputation),
            detail: VcText::new("computation hint has no explicit goal-linked result fact"),
        });
    }

    Some(discharged(
        DischargeRule::BoundedComputation,
        "bounded computation hint is within the active deterministic limit and goal-linked",
    ))
}

fn supported_computation_hint(
    hint: &crate::vc_ir::ComputationHint,
    active_policy: &PolicyKey,
) -> bool {
    match hint {
        crate::vc_ir::ComputationHint::ByComputation => true,
        crate::vc_ir::ComputationHint::LimitPolicy(policy) => policy == active_policy,
        crate::vc_ir::ComputationHint::SymbolicRequest(_) => false,
    }
}

fn explanation_for(vc: VcId, decision: &DischargeDecision) -> DischargeExplanation {
    match decision {
        DischargeDecision::Discharged { rule, detail } => DischargeExplanation {
            vc,
            category: DischargeExplanationCategory::Discharged,
            rule: Some(*rule),
            detail: detail.clone(),
        },
        DischargeDecision::NeedsAtp {
            category,
            rule,
            detail,
        } => DischargeExplanation {
            vc,
            category: *category,
            rule: *rule,
            detail: detail.clone(),
        },
        DischargeDecision::Preserved { category, detail } => DischargeExplanation {
            vc,
            category: *category,
            rule: None,
            detail: detail.clone(),
        },
    }
}

fn apply_decision(vc: &mut VcIr, vc_set: &VcSet, decision: &DischargeDecision) {
    match decision {
        DischargeDecision::Discharged { rule, .. } => apply_discharged(vc, vc_set, *rule),
        DischargeDecision::NeedsAtp { category, .. } => apply_needs_atp(vc, *category),
        DischargeDecision::Preserved { .. } => {}
    }
}

fn apply_discharged(vc: &mut VcIr, vc_set: &VcSet, rule: DischargeRule) {
    let status = VcStatus::Discharged {
        evidence: DischargeEvidenceRef {
            rule: VcText::new(rule.key()),
            evidence_hash: evidence_hash(vc_set, vc.id, rule),
        },
    };

    if vc.status != status {
        vc.status = status;
        vc.provenance.push(discharge_provenance(rule.key()));
    }
}

fn apply_needs_atp(vc: &mut VcIr, category: DischargeExplanationCategory) {
    if !matches!(vc.status, VcStatus::NeedsAtp) {
        vc.status = VcStatus::NeedsAtp;
        vc.provenance
            .push(discharge_provenance(needs_atp_key(category)));
    }
}

fn discharge_provenance(key: impl Into<String>) -> VcProvenance {
    VcProvenance {
        phase: VcProvenancePhase::Discharge,
        key: VcText::new(key.into()),
        core: None,
    }
}

fn evidence_hash(vc_set: &VcSet, vc: VcId, rule: DischargeRule) -> HashMarker {
    let Some(canonical_vc) = vc_set.canonical_vc_fingerprint(vc) else {
        return HashMarker::ConservativeUnknown {
            reason: AnchorUnavailableReason::new(
                "discharged VC contains unresolved core formula payloads",
            ),
        };
    };
    let Some(local_context) = vc_set.local_context_fingerprint(vc) else {
        return HashMarker::ConservativeUnknown {
            reason: AnchorUnavailableReason::new(
                "discharged VC contains unresolved local-context payloads",
            ),
        };
    };
    let mut payload = String::from("deterministic-discharge-evidence-v2\n");
    writeln!(&mut payload, "rule: {}", rule.key()).expect("write string");
    writeln!(&mut payload, "canonical-vc: {:?}", canonical_vc.hash()).expect("write string");
    writeln!(&mut payload, "local-context: {:?}", local_context.hash()).expect("write string");
    HashMarker::Available(stable_fingerprint_hash(
        "mizar-vc-deterministic-discharge",
        payload.as_bytes(),
    ))
}

fn evidence_record_for(
    original: &VcIr,
    output: &VcIr,
    vc_set: &VcSet,
    policy: &DischargePolicy,
    decision: &DischargeDecision,
) -> Option<DischargeEvidenceRecord> {
    match decision {
        DischargeDecision::Discharged { rule, .. } => {
            let evidence = status_evidence(output)?;
            Some(DischargeEvidenceRecord {
                vc: output.id,
                source: DischargeEvidenceSource::NewlyProduced,
                rule: Some(*rule),
                rule_name: evidence.rule.clone(),
                status_evidence: evidence.clone(),
                inputs: evidence_inputs(original, vc_set, policy),
                replay: DischargeEvidenceReplay::ExplicitInputRefs,
            })
        }
        DischargeDecision::Preserved {
            category: DischargeExplanationCategory::Discharged,
            ..
        } => {
            let evidence = status_evidence(output)?;
            Some(DischargeEvidenceRecord {
                vc: output.id,
                source: DischargeEvidenceSource::PreExistingStatus,
                rule: DischargeRule::from_key(evidence.rule.as_str()),
                rule_name: evidence.rule.clone(),
                status_evidence: evidence.clone(),
                inputs: evidence_inputs(original, vc_set, policy),
                replay: DischargeEvidenceReplay::PreservedStatusOnly {
                    reason: VcText::new(
                        "pre-existing discharged status preserved; replay data not reconstructed",
                    ),
                },
            })
        }
        DischargeDecision::NeedsAtp { .. } | DischargeDecision::Preserved { .. } => None,
    }
}

fn status_evidence(vc: &VcIr) -> Option<&DischargeEvidenceRef> {
    match &vc.status {
        VcStatus::Discharged { evidence } => Some(evidence),
        VcStatus::Open
        | VcStatus::NeedsAtp
        | VcStatus::PolicyOpen { .. }
        | VcStatus::AssumedByPolicy { .. }
        | VcStatus::SkippedDueToInvalidInput { .. }
        | VcStatus::DeferredExternal { .. }
        | VcStatus::Error { .. } => None,
    }
}

fn evidence_inputs(vc: &VcIr, vc_set: &VcSet, policy: &DischargePolicy) -> DischargeEvidenceInputs {
    let mut generated_formulas = BTreeSet::new();
    collect_generated_formula_refs(vc_set, vc.goal, &mut generated_formulas);

    let mut premises = BTreeSet::new();
    for premise in &vc.premises {
        collect_premise_evidence(vc_set, premise, &mut premises, &mut generated_formulas);
    }

    let mut local_context = Vec::with_capacity(vc.local_context.entries().len());
    for entry in vc.local_context.entries() {
        local_context.push(entry.id);
        if let Some(formula) = entry.formula {
            collect_generated_formula_refs(vc_set, formula, &mut generated_formulas);
        }
    }

    let mut unfold_requests = Vec::new();
    let mut computation = None;
    if let Some(hint) = &vc.proof_hint {
        for premise in &hint.citations {
            collect_premise_evidence(vc_set, premise, &mut premises, &mut generated_formulas);
        }
        for restriction in &hint.premise_restrictions {
            match restriction {
                crate::vc_ir::PremiseRestriction::Only(restricted)
                | crate::vc_ir::PremiseRestriction::Exclude(restricted) => {
                    for premise in restricted {
                        collect_premise_evidence(
                            vc_set,
                            premise,
                            &mut premises,
                            &mut generated_formulas,
                        );
                    }
                }
                crate::vc_ir::PremiseRestriction::Intent(_) => {}
            }
        }
        unfold_requests = hint.unfold_requests.clone();
        computation =
            hint.computation
                .clone()
                .map(|computation_hint| DischargeComputationEvidence {
                    hint: computation_hint,
                    active_policy: policy.computation_limit.policy.clone(),
                    max_steps: policy.computation_limit.max_steps,
                    requested_steps: hint.max_axioms,
                    timeout: hint.timeout.clone(),
                });
    }

    DischargeEvidenceInputs {
        goal: vc.goal,
        local_context,
        premises: premises.into_iter().collect(),
        generated_formulas: generated_formulas.into_iter().collect(),
        policy_inputs: vc
            .local_context
            .policy_inputs()
            .iter()
            .map(|input| DischargePolicyEvidence {
                key: input.key.clone(),
                value: input.value.clone(),
            })
            .collect(),
        unfold_requests,
        computation,
    }
}

fn collect_premise_evidence(
    vc_set: &VcSet,
    premise: &PremiseRef,
    premises: &mut BTreeSet<PremiseRef>,
    generated_formulas: &mut BTreeSet<VcGeneratedFormulaId>,
) {
    premises.insert(premise.clone());
    if let PremiseRef::GeneratedFact { formula } = premise {
        collect_generated_formula_refs(vc_set, *formula, generated_formulas);
    }
}

fn collect_generated_formula_refs(
    vc_set: &VcSet,
    formula: VcFormulaRef,
    generated_formulas: &mut BTreeSet<VcGeneratedFormulaId>,
) {
    collect_generated_formula_refs_inner(vc_set, formula, generated_formulas, &mut BTreeSet::new());
}

fn collect_generated_formula_refs_inner(
    vc_set: &VcSet,
    formula: VcFormulaRef,
    generated_formulas: &mut BTreeSet<VcGeneratedFormulaId>,
    active: &mut BTreeSet<VcGeneratedFormulaId>,
) {
    let VcFormulaRef::Generated(id) = formula else {
        return;
    };
    if !generated_formulas.insert(id) || !active.insert(id) {
        return;
    }

    if let Some(generated) = vc_set.generated_formula(id) {
        match &generated.shape {
            VcGeneratedFormulaShape::True
            | VcGeneratedFormulaShape::False
            | VcGeneratedFormulaShape::Diagnostic(_) => {}
            VcGeneratedFormulaShape::Ref(inner) | VcGeneratedFormulaShape::Not(inner) => {
                collect_generated_formula_refs_inner(vc_set, *inner, generated_formulas, active);
            }
            VcGeneratedFormulaShape::And(formulas) | VcGeneratedFormulaShape::Or(formulas) => {
                for formula in formulas {
                    collect_generated_formula_refs_inner(
                        vc_set,
                        *formula,
                        generated_formulas,
                        active,
                    );
                }
            }
            VcGeneratedFormulaShape::Implies {
                premise,
                conclusion,
            } => {
                collect_generated_formula_refs_inner(vc_set, *premise, generated_formulas, active);
                collect_generated_formula_refs_inner(
                    vc_set,
                    *conclusion,
                    generated_formulas,
                    active,
                );
            }
            VcGeneratedFormulaShape::Quantified { body, .. } => {
                collect_generated_formula_refs_inner(vc_set, *body, generated_formulas, active);
            }
        }
    }

    active.remove(&id);
}

fn has_local_contradiction(vc: &VcIr, vc_set: &VcSet) -> bool {
    vc.local_context
        .entries()
        .iter()
        .filter_map(|entry| entry.formula)
        .any(|formula| formula_truth(vc_set, formula) == FormulaTruth::False)
        || vc
            .premises
            .iter()
            .chain(proof_hint_citations(vc))
            .any(|premise| premise_is_false(premise, &vc.local_context, vc_set))
}

fn has_direct_local_fact(vc: &VcIr, vc_set: &VcSet) -> bool {
    vc.local_context.entries().iter().any(|entry| {
        direct_fact_kind(&entry.kind)
            && entry
                .formula
                .is_some_and(|formula| formula_refs_match(vc_set, formula, vc.goal))
    }) || vc
        .premises
        .iter()
        .any(|premise| premise_matches_goal(premise, &vc.local_context, vc_set, vc.goal))
}

fn direct_fact_kind(kind: &ContextEntryKind) -> bool {
    matches!(
        kind,
        ContextEntryKind::TypePredicate
            | ContextEntryKind::SethoodFact
            | ContextEntryKind::NonEmptinessFact
            | ContextEntryKind::ProofAssumption
            | ContextEntryKind::CurrentThesis
            | ContextEntryKind::CitedPremise
            | ContextEntryKind::CheckerFact
            | ContextEntryKind::GeneratedFact
    )
}

fn premise_is_false(premise: &PremiseRef, context: &LocalContext, vc_set: &VcSet) -> bool {
    match premise {
        PremiseRef::GeneratedFact { formula } => {
            formula_truth(vc_set, *formula) == FormulaTruth::False
        }
        PremiseRef::LocalContext(id) => context_entry(context, *id)
            .and_then(|entry| entry.formula)
            .is_some_and(|formula| formula_truth(vc_set, formula) == FormulaTruth::False),
        PremiseRef::LocalLabel { .. }
        | PremiseRef::ImportedFact { .. }
        | PremiseRef::DefinitionBoundary { .. }
        | PremiseRef::PermittedUnfolding { .. }
        | PremiseRef::CheckerFact { .. }
        | PremiseRef::TypePredicate { .. }
        | PremiseRef::RegistrationTrace { .. }
        | PremiseRef::ClusterTrace { .. }
        | PremiseRef::ReductionTrace { .. }
        | PremiseRef::PolicyAssumption { .. }
        | PremiseRef::ConservativeUnknown { .. } => false,
    }
}

fn premise_matches_goal(
    premise: &PremiseRef,
    context: &LocalContext,
    vc_set: &VcSet,
    goal: VcFormulaRef,
) -> bool {
    match premise {
        PremiseRef::GeneratedFact { formula } => formula_refs_match(vc_set, *formula, goal),
        PremiseRef::LocalContext(id) => context_entry(context, *id).is_some_and(|entry| {
            direct_fact_kind(&entry.kind)
                && entry
                    .formula
                    .is_some_and(|formula| formula_refs_match(vc_set, formula, goal))
        }),
        PremiseRef::CheckerFact { formula } | PremiseRef::TypePredicate { formula } => {
            formula_refs_match(vc_set, VcFormulaRef::Core(*formula), goal)
        }
        PremiseRef::LocalLabel { .. }
        | PremiseRef::ImportedFact { .. }
        | PremiseRef::DefinitionBoundary { .. }
        | PremiseRef::PermittedUnfolding { .. }
        | PremiseRef::RegistrationTrace { .. }
        | PremiseRef::ClusterTrace { .. }
        | PremiseRef::ReductionTrace { .. }
        | PremiseRef::PolicyAssumption { .. }
        | PremiseRef::ConservativeUnknown { .. } => false,
    }
}

fn has_goal_linked_support(vc: &VcIr, vc_set: &VcSet) -> bool {
    vc.local_context.entries().iter().any(|entry| {
        direct_fact_kind(&entry.kind)
            && entry
                .formula
                .is_some_and(|formula| formula_refs_match(vc_set, formula, vc.goal))
    }) || vc
        .premises
        .iter()
        .chain(proof_hint_citations(vc))
        .any(|premise| premise_matches_goal(premise, &vc.local_context, vc_set, vc.goal))
}

fn context_entry(context: &LocalContext, id: ContextEntryId) -> Option<&ContextEntry> {
    context
        .entries()
        .get(id.index())
        .filter(|entry| entry.id == id)
}

fn has_trace_premise(vc: &VcIr) -> bool {
    vc.premises
        .iter()
        .chain(proof_hint_citations(vc))
        .any(trace_premise)
}

fn trace_premise(premise: &PremiseRef) -> bool {
    matches!(
        premise,
        PremiseRef::RegistrationTrace { .. }
            | PremiseRef::ClusterTrace { .. }
            | PremiseRef::ReductionTrace { .. }
    )
}

fn definitional_reduction_allowed(vc: &VcIr, vc_set: &VcSet) -> bool {
    let Some(hint) = &vc.proof_hint else {
        return false;
    };
    if hint.unfold_requests.is_empty()
        || !has_definitional_policy_input(&vc.local_context)
        || !has_goal_linked_support(vc, vc_set)
    {
        return false;
    }

    hint.unfold_requests.iter().all(|request| {
        request.opacity_override == Some(DefinitionOpacityOverride::Transparent)
            || has_permitted_unfolding(vc, request.definition)
    })
}

fn has_definitional_policy_input(context: &LocalContext) -> bool {
    context.policy_inputs().iter().any(|input| {
        input.key.as_str() == DEFINITIONAL_REDUCTION_POLICY
            && input.value.as_str() == DEFINITIONAL_REDUCTION_ALLOW
    })
}

fn has_permitted_unfolding(vc: &VcIr, definition: CoreDefinitionId) -> bool {
    vc.premises
        .iter()
        .chain(proof_hint_citations(vc))
        .any(|premise| {
            matches!(
                premise,
                PremiseRef::PermittedUnfolding { definition: permitted }
                    if *permitted == definition
            )
        })
}

fn has_missing_trace_marker(vc: &VcIr) -> bool {
    vc.premises
        .iter()
        .chain(proof_hint_citations(vc))
        .any(|premise| matches!(premise, PremiseRef::ConservativeUnknown { .. }))
}

fn proof_hint_citations(vc: &VcIr) -> impl Iterator<Item = &PremiseRef> {
    vc.proof_hint
        .as_ref()
        .map_or([].as_slice(), |hint| hint.citations.as_slice())
        .iter()
}

fn formula_truth(vc_set: &VcSet, formula: VcFormulaRef) -> FormulaTruth {
    formula_truth_inner(vc_set, formula, &mut BTreeSet::new())
}

fn formula_truth_inner(
    vc_set: &VcSet,
    formula: VcFormulaRef,
    active: &mut BTreeSet<VcGeneratedFormulaId>,
) -> FormulaTruth {
    let VcFormulaRef::Generated(id) = formula else {
        return FormulaTruth::Unknown;
    };
    if !active.insert(id) {
        return FormulaTruth::Unknown;
    }

    let truth = vc_set
        .generated_formula(id)
        .map_or(FormulaTruth::Unknown, |generated| match &generated.shape {
            VcGeneratedFormulaShape::True => FormulaTruth::True,
            VcGeneratedFormulaShape::False => FormulaTruth::False,
            VcGeneratedFormulaShape::Ref(inner) => formula_truth_inner(vc_set, *inner, active),
            VcGeneratedFormulaShape::Not(inner) => {
                formula_truth_inner(vc_set, *inner, active).negate()
            }
            VcGeneratedFormulaShape::And(formulas) => and_truth(vc_set, formulas, active),
            VcGeneratedFormulaShape::Or(formulas) => or_truth(vc_set, formulas, active),
            VcGeneratedFormulaShape::Implies {
                premise,
                conclusion,
            } => implies_truth(vc_set, *premise, *conclusion, active),
            VcGeneratedFormulaShape::Quantified { .. } | VcGeneratedFormulaShape::Diagnostic(_) => {
                FormulaTruth::Unknown
            }
        });

    active.remove(&id);
    truth
}

fn and_truth(
    vc_set: &VcSet,
    formulas: &[VcFormulaRef],
    active: &mut BTreeSet<VcGeneratedFormulaId>,
) -> FormulaTruth {
    let mut all_true = true;
    for formula in formulas {
        match formula_truth_inner(vc_set, *formula, active) {
            FormulaTruth::True => {}
            FormulaTruth::False => return FormulaTruth::False,
            FormulaTruth::Unknown => all_true = false,
        }
    }
    if all_true {
        FormulaTruth::True
    } else {
        FormulaTruth::Unknown
    }
}

fn or_truth(
    vc_set: &VcSet,
    formulas: &[VcFormulaRef],
    active: &mut BTreeSet<VcGeneratedFormulaId>,
) -> FormulaTruth {
    let mut all_false = true;
    for formula in formulas {
        match formula_truth_inner(vc_set, *formula, active) {
            FormulaTruth::True => return FormulaTruth::True,
            FormulaTruth::False => {}
            FormulaTruth::Unknown => all_false = false,
        }
    }
    if all_false {
        FormulaTruth::False
    } else {
        FormulaTruth::Unknown
    }
}

fn implies_truth(
    vc_set: &VcSet,
    premise: VcFormulaRef,
    conclusion: VcFormulaRef,
    active: &mut BTreeSet<VcGeneratedFormulaId>,
) -> FormulaTruth {
    if formula_refs_match(vc_set, premise, conclusion) {
        return FormulaTruth::True;
    }

    let premise_truth = formula_truth_inner(vc_set, premise, active);
    let conclusion_truth = formula_truth_inner(vc_set, conclusion, active);

    match (premise_truth, conclusion_truth) {
        (FormulaTruth::False, _) | (_, FormulaTruth::True) => FormulaTruth::True,
        (FormulaTruth::True, FormulaTruth::False) => FormulaTruth::False,
        (FormulaTruth::True | FormulaTruth::Unknown, FormulaTruth::Unknown)
        | (FormulaTruth::Unknown, FormulaTruth::False) => FormulaTruth::Unknown,
    }
}

fn formula_refs_match(vc_set: &VcSet, left: VcFormulaRef, right: VcFormulaRef) -> bool {
    normalize_formula_ref(vc_set, left) == normalize_formula_ref(vc_set, right)
}

fn normalize_formula_ref(vc_set: &VcSet, formula: VcFormulaRef) -> VcFormulaRef {
    normalize_formula_ref_inner(vc_set, formula, &mut BTreeSet::new())
}

fn normalize_formula_ref_inner(
    vc_set: &VcSet,
    formula: VcFormulaRef,
    active: &mut BTreeSet<VcGeneratedFormulaId>,
) -> VcFormulaRef {
    let VcFormulaRef::Generated(id) = formula else {
        return formula;
    };
    if !active.insert(id) {
        return formula;
    }

    let normalized = match vc_set
        .generated_formula(id)
        .map(|generated| &generated.shape)
    {
        Some(VcGeneratedFormulaShape::Ref(inner)) => {
            normalize_formula_ref_inner(vc_set, *inner, active)
        }
        Some(_) | None => formula,
    };
    active.remove(&id);
    normalized
}

fn needs_atp_key(category: DischargeExplanationCategory) -> &'static str {
    match category {
        DischargeExplanationCategory::NeedsAtp => "task-11-discharge:needs-atp",
        DischargeExplanationCategory::MissingTrace => "task-11-discharge:missing-trace",
        DischargeExplanationCategory::LimitExceeded => "task-11-discharge:limit-exceeded",
        DischargeExplanationCategory::UnsupportedRule => "task-11-discharge:unsupported-rule",
        DischargeExplanationCategory::Discharged
        | DischargeExplanationCategory::PolicyOpen
        | DischargeExplanationCategory::AssumedByPolicy
        | DischargeExplanationCategory::SkippedInvalidInput
        | DischargeExplanationCategory::DeferredExternal
        | DischargeExplanationCategory::Error => "task-11-discharge:needs-atp",
    }
}

impl DischargeRule {
    fn from_key(key: &str) -> Option<Self> {
        match key {
            "task-11-generated-tautology-v1" => Some(Self::GeneratedTautology),
            "task-11-local-contradiction-v1" => Some(Self::LocalContradiction),
            "task-11-direct-local-fact-v1" => Some(Self::DirectLocalFact),
            "task-11-trace-premise-v1" => Some(Self::TracePremise),
            "task-11-definitional-reduction-v1" => Some(Self::DefinitionalReduction),
            "task-11-bounded-computation-v1" => Some(Self::BoundedComputation),
            _ => None,
        }
    }

    const fn key(self) -> &'static str {
        match self {
            Self::GeneratedTautology => "task-11-generated-tautology-v1",
            Self::LocalContradiction => "task-11-local-contradiction-v1",
            Self::DirectLocalFact => "task-11-direct-local-fact-v1",
            Self::TracePremise => "task-11-trace-premise-v1",
            Self::DefinitionalReduction => "task-11-definitional-reduction-v1",
            Self::BoundedComputation => "task-11-bounded-computation-v1",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vc_ir::{
        AnchorCompleteness, AnchorIngredient, AnchorLabel, AnchorLabelRole, AnchorOwner,
        AnchorUnavailableReason, CanonicalSortKey, ComputationHint, DefinitionUnfoldRequest,
        GenerationSchemaVersion, PolicyValue, PremiseRestriction, ProofHint, ProofHintKey,
        SeedAccounting, SeedOriginRef, SeedVcMapping, SeedVcRef, VcGeneratedFormula,
        VcGeneratedFormulaKind, VcKind, VcModuleRef, VcSchemaVersion, VcSourceRef,
        VerifierPolicyInput,
    };
    use mizar_core::{
        control_flow::ObligationHandoffId,
        core_ir::{
            CoreDiagnosticId, CoreFormulaId, CoreItemId, CoreLabelRef, CoreNodeRef, CoreProvenance,
            CoreProvenanceKey, CoreProvenancePhase, GeneratedFrom, GeneratedOriginKey,
            GeneratedOriginKind, LocalProofOrProgramPath, NormalizedSemanticOrigin,
            ObligationSeedId, ObligationSeedStatus,
        },
    };
    use mizar_session::{
        BuildSnapshotId, Hash, InMemorySessionIdAllocator, SessionIdAllocator, SourceId,
        SourceRange,
    };

    #[test]
    fn discharges_generated_tautology_and_preserves_shape() {
        let original = fixture_set(fixture_parts(
            VcStatus::NeedsAtp,
            VcFormulaRef::Generated(VcGeneratedFormulaId::new(0)),
            vec![generated_formula(0, VcGeneratedFormulaShape::True)],
        ));
        let output = try_discharge(DischargeInput {
            vc_set: &original,
            policy: &DischargePolicy::default(),
        })
        .expect("discharge");
        let discharged = output.vc_set();
        let vc = &discharged.vcs()[0];

        assert_discharged_rule(vc, DischargeRule::GeneratedTautology);
        assert_eq!(
            discharged.generated_formulas(),
            original.generated_formulas()
        );
        assert_eq!(discharged.seed_accounting(), original.seed_accounting());
        assert_eq!(vc.local_context, original.vcs()[0].local_context);
        assert_eq!(vc.premises, original.vcs()[0].premises);
        assert_eq!(vc.proof_hint, original.vcs()[0].proof_hint);
        assert_eq!(
            output.explanations(),
            [DischargeExplanation {
                vc: VcId::new(0),
                category: DischargeExplanationCategory::Discharged,
                rule: Some(DischargeRule::GeneratedTautology),
                detail: VcText::new("goal is a task-11 syntactic tautology"),
            }]
        );
        let record = only_evidence_record(&output);
        assert_eq!(record.source, DischargeEvidenceSource::NewlyProduced);
        assert_eq!(record.rule, Some(DischargeRule::GeneratedTautology));
        assert_eq!(
            &record.status_evidence,
            status_evidence(vc).expect("discharged status evidence")
        );
        assert_eq!(
            record.inputs.generated_formulas,
            vec![VcGeneratedFormulaId::new(0)]
        );
        assert!(matches!(
            record.replay,
            DischargeEvidenceReplay::ExplicitInputRefs
        ));
        assert!(output.debug_text().contains("discharge-output-debug-v1"));
        assert!(output.debug_text().contains("[discharge-evidence]"));
    }

    #[test]
    fn discharges_reflexive_implication_and_ref_normalized_fact() {
        let reflexive = fixture_set(fixture_parts(
            VcStatus::NeedsAtp,
            VcFormulaRef::Generated(VcGeneratedFormulaId::new(0)),
            vec![generated_formula(
                0,
                VcGeneratedFormulaShape::Implies {
                    premise: VcFormulaRef::Core(CoreFormulaId::new(20)),
                    conclusion: VcFormulaRef::Core(CoreFormulaId::new(20)),
                },
            )],
        ));
        let reflexive_output = try_discharge(DischargeInput {
            vc_set: &reflexive,
            policy: &DischargePolicy::default(),
        })
        .expect("reflexive implication discharge");

        assert_discharged_rule(
            &reflexive_output.vc_set().vcs()[0],
            DischargeRule::GeneratedTautology,
        );

        let normalized = fixture_set(with_context_entry(
            fixture_parts(
                VcStatus::NeedsAtp,
                VcFormulaRef::Generated(VcGeneratedFormulaId::new(0)),
                vec![generated_formula(
                    0,
                    VcGeneratedFormulaShape::Ref(VcFormulaRef::Core(CoreFormulaId::new(21))),
                )],
            ),
            context_entry(
                0,
                "000-normalized",
                ContextEntryKind::SethoodFact,
                Some(VcFormulaRef::Core(CoreFormulaId::new(21))),
            ),
        ));
        let normalized_output = try_discharge(DischargeInput {
            vc_set: &normalized,
            policy: &DischargePolicy::default(),
        })
        .expect("ref-normalized fact discharge");

        assert_discharged_rule(
            &normalized_output.vc_set().vcs()[0],
            DischargeRule::DirectLocalFact,
        );
    }

    #[test]
    fn discharges_contradiction_and_exact_local_facts() {
        let contradiction = fixture_set(with_context_entry(
            fixture_parts(
                VcStatus::NeedsAtp,
                VcFormulaRef::Core(CoreFormulaId::new(7)),
                vec![generated_formula(0, VcGeneratedFormulaShape::False)],
            ),
            context_entry(
                0,
                "000-false",
                ContextEntryKind::ProofAssumption,
                Some(VcFormulaRef::Generated(VcGeneratedFormulaId::new(0))),
            ),
        ));
        let contradiction_output = try_discharge(DischargeInput {
            vc_set: &contradiction,
            policy: &DischargePolicy::default(),
        })
        .expect("contradiction discharge");

        assert_discharged_rule(
            &contradiction_output.vc_set().vcs()[0],
            DischargeRule::LocalContradiction,
        );

        for (index, kind) in [
            ContextEntryKind::TypePredicate,
            ContextEntryKind::SethoodFact,
            ContextEntryKind::NonEmptinessFact,
        ]
        .into_iter()
        .enumerate()
        {
            let formula = CoreFormulaId::new(11 + index);
            let direct_fact = fixture_set(with_context_entry(
                fixture_parts(VcStatus::NeedsAtp, VcFormulaRef::Core(formula), Vec::new()),
                context_entry(
                    0,
                    &format!("000-direct-fact-{index}"),
                    kind,
                    Some(VcFormulaRef::Core(formula)),
                ),
            ));
            let direct_output = try_discharge(DischargeInput {
                vc_set: &direct_fact,
                policy: &DischargePolicy::default(),
            })
            .expect("direct fact discharge");

            assert_discharged_rule(
                &direct_output.vc_set().vcs()[0],
                DischargeRule::DirectLocalFact,
            );
        }
    }

    #[test]
    fn discharges_trace_definitional_and_bounded_computation_inputs() {
        for (index, premise) in [
            PremiseRef::ClusterTrace {
                trace: VcText::new("cluster:0"),
            },
            PremiseRef::RegistrationTrace {
                trace: VcText::new("registration:0"),
            },
        ]
        .into_iter()
        .enumerate()
        {
            let formula = CoreFormulaId::new(12 + index);
            let trace = fixture_set(with_premises(
                parts_with_generated_goal_support(VcStatus::NeedsAtp, formula),
                vec![premise.clone(), goal_generated_fact()],
            ));
            let trace_output = try_discharge(DischargeInput {
                vc_set: &trace,
                policy: &DischargePolicy::default(),
            })
            .expect("trace discharge");
            assert_discharged_rule(&trace_output.vc_set().vcs()[0], DischargeRule::TracePremise);
            let trace_record = only_evidence_record(&trace_output);
            assert_eq!(trace_record.rule, Some(DischargeRule::TracePremise));
            assert!(trace_record.inputs.premises.contains(&premise));
            assert!(
                trace_record
                    .inputs
                    .premises
                    .contains(&goal_generated_fact())
            );
            assert_eq!(
                trace_record.inputs.generated_formulas,
                vec![VcGeneratedFormulaId::new(0)]
            );
        }

        let reduction_citation = fixture_set(with_proof_hint(
            parts_with_generated_goal_support(VcStatus::NeedsAtp, CoreFormulaId::new(22)),
            trace_hint(vec![
                PremiseRef::ReductionTrace {
                    trace: VcText::new("reduction:0"),
                },
                goal_generated_fact(),
            ]),
        ));
        let reduction_output = try_discharge(DischargeInput {
            vc_set: &reduction_citation,
            policy: &DischargePolicy::default(),
        })
        .expect("trace citation discharge");
        assert_discharged_rule(
            &reduction_output.vc_set().vcs()[0],
            DischargeRule::TracePremise,
        );
        let reduction_record = only_evidence_record(&reduction_output);
        assert!(
            reduction_record
                .inputs
                .premises
                .contains(&PremiseRef::ReductionTrace {
                    trace: VcText::new("reduction:0"),
                })
        );
        assert!(
            reduction_record
                .inputs
                .premises
                .contains(&goal_generated_fact())
        );

        let definitional = fixture_set(with_proof_hint(
            with_policy_input(
                parts_with_generated_goal_support(VcStatus::NeedsAtp, CoreFormulaId::new(13)),
                DEFINITIONAL_REDUCTION_POLICY,
                DEFINITIONAL_REDUCTION_ALLOW,
            ),
            definition_hint(vec![goal_generated_fact()]),
        ));
        let definitional_output = try_discharge(DischargeInput {
            vc_set: &definitional,
            policy: &DischargePolicy::default(),
        })
        .expect("definitional discharge");
        assert_discharged_rule(
            &definitional_output.vc_set().vcs()[0],
            DischargeRule::DefinitionalReduction,
        );
        let definitional_record = only_evidence_record(&definitional_output);
        assert_eq!(
            definitional_record.inputs.policy_inputs,
            vec![DischargePolicyEvidence {
                key: PolicyKey::new(DEFINITIONAL_REDUCTION_POLICY),
                value: PolicyValue::new(DEFINITIONAL_REDUCTION_ALLOW),
            }]
        );
        assert_eq!(
            definitional_record.inputs.unfold_requests,
            vec![transparent_request(CoreDefinitionId::new(0))]
        );
        assert!(
            definitional_record
                .inputs
                .premises
                .contains(&goal_generated_fact())
        );

        let computation = fixture_set(with_proof_hint(
            parts_with_generated_goal_support(VcStatus::NeedsAtp, CoreFormulaId::new(14)),
            computation_hint(3, vec![goal_generated_fact()]),
        ));
        let policy = DischargePolicy {
            computation_limit: ComputationLimit {
                policy: PolicyKey::new(DEFAULT_COMPUTATION_LIMIT_POLICY),
                max_steps: 4,
            },
        };
        let computation_output = try_discharge(DischargeInput {
            vc_set: &computation,
            policy: &policy,
        })
        .expect("computation discharge");
        assert_discharged_rule(
            &computation_output.vc_set().vcs()[0],
            DischargeRule::BoundedComputation,
        );
        let computation_record = only_evidence_record(&computation_output);
        assert_eq!(
            computation_record.inputs.computation,
            Some(DischargeComputationEvidence {
                hint: ComputationHint::ByComputation,
                active_policy: PolicyKey::new(DEFAULT_COMPUTATION_LIMIT_POLICY),
                max_steps: 4,
                requested_steps: Some(3),
                timeout: Some(ProofHintKey::new(DEFAULT_COMPUTATION_LIMIT_POLICY)),
            })
        );
        assert!(
            computation_record
                .inputs
                .premises
                .contains(&goal_generated_fact())
        );
    }

    #[test]
    fn evidence_records_cover_preserved_discharged_statuses() {
        let evidence = DischargeEvidenceRef {
            rule: VcText::new(DischargeRule::DirectLocalFact.key()),
            evidence_hash: HashMarker::Available(Hash::from_bytes([7; Hash::BYTE_LEN])),
        };
        let input = fixture_set(fixture_parts(
            VcStatus::Discharged {
                evidence: evidence.clone(),
            },
            VcFormulaRef::Core(CoreFormulaId::new(31)),
            Vec::new(),
        ));

        let output = try_discharge(DischargeInput {
            vc_set: &input,
            policy: &DischargePolicy::default(),
        })
        .expect("preserved discharged evidence");

        assert_eq!(output.vc_set(), &input);
        assert_eq!(
            output.explanations()[0].category,
            DischargeExplanationCategory::Discharged
        );
        let record = only_evidence_record(&output);
        assert_eq!(record.source, DischargeEvidenceSource::PreExistingStatus);
        assert_eq!(record.rule, Some(DischargeRule::DirectLocalFact));
        assert_eq!(
            record.rule_name.as_str(),
            DischargeRule::DirectLocalFact.key()
        );
        assert_eq!(record.status_evidence, evidence);
        assert!(matches!(
            &record.replay,
            DischargeEvidenceReplay::PreservedStatusOnly { reason }
                if reason.as_str().contains("pre-existing discharged status preserved")
        ));
    }

    #[test]
    fn evidence_records_cover_multiple_discharged_outputs_in_order() {
        let preserved_evidence = DischargeEvidenceRef {
            rule: VcText::new(DischargeRule::DirectLocalFact.key()),
            evidence_hash: HashMarker::Available(Hash::from_bytes([9; Hash::BYTE_LEN])),
        };
        let mut parts = fixture_parts(
            VcStatus::NeedsAtp,
            VcFormulaRef::Generated(VcGeneratedFormulaId::new(0)),
            vec![generated_formula(0, VcGeneratedFormulaShape::True)],
        );
        push_second_vc(
            &mut parts,
            VcStatus::Discharged {
                evidence: preserved_evidence.clone(),
            },
            VcFormulaRef::Core(CoreFormulaId::new(32)),
            Vec::new(),
        );
        let input = fixture_set(parts);

        let output = try_discharge(DischargeInput {
            vc_set: &input,
            policy: &DischargePolicy::default(),
        })
        .expect("mixed evidence output");

        assert_eq!(
            output
                .evidence_records()
                .iter()
                .map(|record| (record.vc, record.source))
                .collect::<Vec<_>>(),
            vec![
                (VcId::new(0), DischargeEvidenceSource::NewlyProduced),
                (VcId::new(1), DischargeEvidenceSource::PreExistingStatus),
            ]
        );
        for record in output.evidence_records() {
            let vc = output.vc_set().vc(record.vc).expect("recorded vc");
            assert_eq!(
                Some(&record.status_evidence),
                status_evidence(vc),
                "evidence record must match output status evidence"
            );
        }
        assert_eq!(
            output.evidence_records()[1].status_evidence,
            preserved_evidence
        );
    }

    #[test]
    fn definitional_reduction_fails_closed_without_policy_or_full_permission() {
        let missing_policy = fixture_set(with_proof_hint(
            parts_with_generated_goal_support(VcStatus::NeedsAtp, CoreFormulaId::new(23)),
            definition_hint(vec![goal_generated_fact()]),
        ));
        assert_unsupported_after_discharge(&missing_policy);

        let opaque_without_permission = fixture_set(with_proof_hint(
            with_policy_input(
                parts_with_generated_goal_support(VcStatus::NeedsAtp, CoreFormulaId::new(24)),
                DEFINITIONAL_REDUCTION_POLICY,
                DEFINITIONAL_REDUCTION_ALLOW,
            ),
            definition_hint_with_requests(
                vec![goal_generated_fact()],
                vec![opaque_request(CoreDefinitionId::new(1))],
            ),
        ));
        assert_unsupported_after_discharge(&opaque_without_permission);

        let mixed_requests = fixture_set(with_proof_hint(
            with_policy_input(
                parts_with_generated_goal_support(VcStatus::NeedsAtp, CoreFormulaId::new(25)),
                DEFINITIONAL_REDUCTION_POLICY,
                DEFINITIONAL_REDUCTION_ALLOW,
            ),
            definition_hint_with_requests(
                vec![goal_generated_fact()],
                vec![
                    transparent_request(CoreDefinitionId::new(0)),
                    opaque_request(CoreDefinitionId::new(2)),
                ],
            ),
        ));
        assert_unsupported_after_discharge(&mixed_requests);
    }

    #[test]
    fn marker_only_trace_unfold_and_computation_stay_needs_atp() {
        let trace_marker_only = fixture_set(with_premises(
            fixture_parts(
                VcStatus::NeedsAtp,
                VcFormulaRef::Core(CoreFormulaId::new(26)),
                Vec::new(),
            ),
            vec![PremiseRef::ClusterTrace {
                trace: VcText::new("cluster-without-goal-fact"),
            }],
        ));
        let trace_output = try_discharge(DischargeInput {
            vc_set: &trace_marker_only,
            policy: &DischargePolicy::default(),
        })
        .expect("trace marker fallback");

        assert_eq!(trace_output.vc_set(), &trace_marker_only);
        assert_eq!(
            trace_output.explanations()[0].category,
            DischargeExplanationCategory::MissingTrace
        );
        assert_eq!(
            trace_output.explanations()[0].rule,
            Some(DischargeRule::TracePremise)
        );
        assert!(trace_output.evidence_records().is_empty());

        let unfold_marker_only = fixture_set(with_proof_hint(
            with_policy_input(
                fixture_parts(
                    VcStatus::NeedsAtp,
                    VcFormulaRef::Core(CoreFormulaId::new(27)),
                    Vec::new(),
                ),
                DEFINITIONAL_REDUCTION_POLICY,
                DEFINITIONAL_REDUCTION_ALLOW,
            ),
            definition_hint(Vec::new()),
        ));
        assert_unsupported_after_discharge(&unfold_marker_only);

        let computation_marker_only = fixture_set(with_proof_hint(
            fixture_parts(
                VcStatus::NeedsAtp,
                VcFormulaRef::Core(CoreFormulaId::new(28)),
                Vec::new(),
            ),
            computation_hint(3, Vec::new()),
        ));
        let computation_output = try_discharge(DischargeInput {
            vc_set: &computation_marker_only,
            policy: &DischargePolicy::default(),
        })
        .expect("computation marker fallback");

        assert_eq!(computation_output.vc_set(), &computation_marker_only);
        assert_eq!(
            computation_output.explanations()[0].category,
            DischargeExplanationCategory::UnsupportedRule
        );
        assert_eq!(
            computation_output.explanations()[0].rule,
            Some(DischargeRule::BoundedComputation)
        );
        assert!(computation_output.evidence_records().is_empty());
    }

    #[test]
    fn limit_exceeded_and_unsupported_vcs_preserve_atp_context() {
        let limit_input = fixture_set(with_proof_hint(
            parts_with_generated_goal_support(VcStatus::Open, CoreFormulaId::new(15)),
            computation_hint(9, vec![goal_generated_fact()]),
        ));
        let policy = DischargePolicy {
            computation_limit: ComputationLimit {
                policy: PolicyKey::new(DEFAULT_COMPUTATION_LIMIT_POLICY),
                max_steps: 4,
            },
        };
        let limit_output = try_discharge(DischargeInput {
            vc_set: &limit_input,
            policy: &policy,
        })
        .expect("limit fallback");
        let limit_vc = &limit_output.vc_set().vcs()[0];

        assert!(matches!(limit_vc.status, VcStatus::NeedsAtp));
        assert_eq!(limit_vc.local_context, limit_input.vcs()[0].local_context);
        assert_eq!(limit_vc.premises, limit_input.vcs()[0].premises);
        assert_eq!(limit_vc.goal, limit_input.vcs()[0].goal);
        assert_eq!(
            limit_output.vc_set().seed_accounting(),
            limit_input.seed_accounting()
        );
        assert_eq!(
            limit_output.explanations()[0].category,
            DischargeExplanationCategory::LimitExceeded
        );
        assert_eq!(
            limit_output.explanations()[0].rule,
            Some(DischargeRule::BoundedComputation)
        );
        assert_eq!(
            limit_vc
                .provenance
                .last()
                .expect("fallback provenance")
                .phase,
            VcProvenancePhase::Discharge
        );

        let unsupported = fixture_set(fixture_parts(
            VcStatus::NeedsAtp,
            VcFormulaRef::Core(CoreFormulaId::new(16)),
            Vec::new(),
        ));
        let unsupported_output = try_discharge(DischargeInput {
            vc_set: &unsupported,
            policy: &DischargePolicy::default(),
        })
        .expect("unsupported fallback");

        assert_eq!(unsupported_output.vc_set(), &unsupported);
        assert_eq!(
            unsupported_output.explanations()[0].category,
            DischargeExplanationCategory::UnsupportedRule
        );
        assert!(limit_output.evidence_records().is_empty());
        assert!(unsupported_output.evidence_records().is_empty());
    }

    #[test]
    fn policy_and_deferred_statuses_are_preserved_without_discharge_evidence() {
        for (status, category) in [
            (
                VcStatus::PolicyOpen {
                    policy: PolicyKey::new("interactive"),
                },
                DischargeExplanationCategory::PolicyOpen,
            ),
            (
                VcStatus::AssumedByPolicy {
                    policy: PolicyKey::new("assume-for-test"),
                    marker: PremiseRef::GeneratedFact {
                        formula: VcFormulaRef::Generated(VcGeneratedFormulaId::new(0)),
                    },
                },
                DischargeExplanationCategory::AssumedByPolicy,
            ),
            (
                VcStatus::SkippedDueToInvalidInput {
                    reason: VcText::new("invalid task-12 fixture"),
                },
                DischargeExplanationCategory::SkippedInvalidInput,
            ),
            (
                VcStatus::DeferredExternal {
                    reason: VcText::new("atp bridge unavailable"),
                },
                DischargeExplanationCategory::DeferredExternal,
            ),
            (
                VcStatus::Error {
                    diagnostic: CoreDiagnosticId::new(3),
                },
                DischargeExplanationCategory::Error,
            ),
        ] {
            let input = fixture_set(fixture_parts(
                status,
                VcFormulaRef::Generated(VcGeneratedFormulaId::new(0)),
                vec![generated_formula(0, VcGeneratedFormulaShape::True)],
            ));
            let output = try_discharge(DischargeInput {
                vc_set: &input,
                policy: &DischargePolicy::default(),
            })
            .expect("status preserved");

            assert_eq!(output.vc_set(), &input);
            assert_eq!(output.explanations()[0].category, category);
            assert!(output.evidence_records().is_empty());
            assert!(
                !output.vc_set().vcs()[0]
                    .provenance
                    .iter()
                    .any(|provenance| provenance.phase == VcProvenancePhase::Discharge)
            );
        }
    }

    #[test]
    fn repeated_discharge_is_deterministic_and_preserves_order() {
        let mut parts = fixture_parts(
            VcStatus::NeedsAtp,
            VcFormulaRef::Generated(VcGeneratedFormulaId::new(0)),
            vec![generated_formula(0, VcGeneratedFormulaShape::True)],
        );
        push_second_unsupported_vc(&mut parts);
        let input = fixture_set(parts);
        let policy = DischargePolicy::default();

        let first = try_discharge(DischargeInput {
            vc_set: &input,
            policy: &policy,
        })
        .expect("first discharge");
        let second = try_discharge(DischargeInput {
            vc_set: &input,
            policy: &policy,
        })
        .expect("second discharge");

        assert_eq!(first, second);
        assert_eq!(first.clone(), first);
        assert_eq!(
            first.evidence_records()[0].clone(),
            first.evidence_records()[0]
        );
        assert_eq!(first.vc_set().debug_text(), second.vc_set().debug_text());
        assert_eq!(first.debug_text(), second.debug_text());
        assert_eq!(
            first
                .explanations()
                .iter()
                .map(|explanation| explanation.vc)
                .collect::<Vec<_>>(),
            vec![VcId::new(0), VcId::new(1)]
        );
        assert_discharged_rule(&first.vc_set().vcs()[0], DischargeRule::GeneratedTautology);
        assert!(matches!(first.vc_set().vcs()[1].status, VcStatus::NeedsAtp));
        assert_eq!(
            first
                .evidence_records()
                .iter()
                .map(|record| record.vc)
                .collect::<Vec<_>>(),
            vec![VcId::new(0)]
        );
    }

    fn assert_unsupported_after_discharge(input: &VcSet) {
        let output = try_discharge(DischargeInput {
            vc_set: input,
            policy: &DischargePolicy::default(),
        })
        .expect("unsupported fallback");

        assert_eq!(output.vc_set(), input);
        assert_eq!(
            output.explanations()[0].category,
            DischargeExplanationCategory::UnsupportedRule
        );
        assert_eq!(output.explanations()[0].rule, None);
        assert!(output.evidence_records().is_empty());
    }

    fn only_evidence_record(output: &DischargeOutput) -> &DischargeEvidenceRecord {
        assert_eq!(output.evidence_records().len(), 1);
        &output.evidence_records()[0]
    }

    fn assert_discharged_rule(vc: &VcIr, expected: DischargeRule) {
        assert!(matches!(
            &vc.status,
            VcStatus::Discharged { evidence }
                if evidence.rule.as_str() == expected.key()
        ));
        assert_eq!(
            vc.provenance.last().expect("discharge provenance").phase,
            VcProvenancePhase::Discharge
        );
    }

    fn fixture_set(parts: VcSetParts) -> VcSet {
        VcSet::try_new(parts).expect("valid discharge fixture")
    }

    fn fixture_parts(
        status: VcStatus,
        goal: VcFormulaRef,
        generated_formulas: Vec<VcGeneratedFormula>,
    ) -> VcSetParts {
        let snapshot = sample_snapshot_id();
        let source = InMemorySessionIdAllocator::new()
            .next_source_id(snapshot)
            .expect("source id");
        let source_ref = source_ref(source);
        let handoff = ObligationHandoffId::new(0);

        VcSetParts {
            schema_version: VcSchemaVersion::new("discharge-test-v1"),
            snapshot,
            source,
            module: VcModuleRef::new("sample"),
            generated_formulas,
            vcs: vec![VcIr {
                id: VcId::new(0),
                kind: VcKind::TheoremProofStep,
                source: VcSourceRef {
                    primary: source_ref.clone(),
                    related: vec![generated_source_ref()],
                },
                seed: SeedVcRef { handoff },
                anchor: incomplete_anchor(source, source_ref),
                local_context: LocalContext::try_new(Vec::new(), Vec::new())
                    .expect("empty local context"),
                premises: Vec::new(),
                goal,
                proof_hint: None,
                status,
                provenance: vec![provenance("vc")],
            }],
            seed_accounting: vec![SeedAccounting {
                handoff,
                origin: SeedOriginRef::ExistingCore {
                    seed: ObligationSeedId::new(0),
                },
                seed_status: ObligationSeedStatus::Active,
                mapping: SeedVcMapping::One { vc: VcId::new(0) },
            }],
        }
    }

    fn parts_with_generated_goal_support(status: VcStatus, formula: CoreFormulaId) -> VcSetParts {
        fixture_parts(
            status,
            VcFormulaRef::Core(formula),
            vec![generated_formula(
                0,
                VcGeneratedFormulaShape::Ref(VcFormulaRef::Core(formula)),
            )],
        )
    }

    fn goal_generated_fact() -> PremiseRef {
        PremiseRef::GeneratedFact {
            formula: VcFormulaRef::Generated(VcGeneratedFormulaId::new(0)),
        }
    }

    fn generated_formula(index: usize, shape: VcGeneratedFormulaShape) -> VcGeneratedFormula {
        VcGeneratedFormula {
            id: VcGeneratedFormulaId::new(index),
            kind: VcGeneratedFormulaKind::GeneratedTypeObligation,
            shape,
            provenance: vec![provenance("generated")],
        }
    }

    fn with_context_entry(mut parts: VcSetParts, entry: ContextEntry) -> VcSetParts {
        parts.vcs[0].local_context = LocalContext::try_new(
            vec![entry],
            parts.vcs[0].local_context.policy_inputs().to_vec(),
        )
        .expect("local context");
        parts
    }

    fn with_policy_input(mut parts: VcSetParts, key: &str, value: &str) -> VcSetParts {
        parts.vcs[0].local_context = LocalContext::try_new(
            parts.vcs[0].local_context.entries().to_vec(),
            vec![VerifierPolicyInput {
                sort_key: CanonicalSortKey::new("000-policy"),
                key: PolicyKey::new(key),
                value: PolicyValue::new(value),
            }],
        )
        .expect("policy context");
        parts
    }

    fn with_premises(mut parts: VcSetParts, premises: Vec<PremiseRef>) -> VcSetParts {
        parts.vcs[0].premises = premises;
        parts
    }

    fn with_proof_hint(mut parts: VcSetParts, proof_hint: ProofHint) -> VcSetParts {
        parts.vcs[0].proof_hint = Some(proof_hint);
        parts
    }

    fn context_entry(
        id: usize,
        sort_key: &str,
        kind: ContextEntryKind,
        formula: Option<VcFormulaRef>,
    ) -> ContextEntry {
        ContextEntry {
            id: ContextEntryId::new(id),
            sort_key: CanonicalSortKey::new(sort_key),
            kind,
            formula,
            provenance: vec![provenance("context")],
        }
    }

    fn trace_hint(citations: Vec<PremiseRef>) -> ProofHint {
        ProofHint {
            citations,
            unfold_requests: Vec::new(),
            premise_restrictions: Vec::new(),
            solver: Some(ProofHintKey::new("task-11-trace")),
            max_axioms: None,
            timeout: None,
            computation: None,
            provenance: vec![provenance("trace-hint")],
        }
    }

    fn definition_hint(citations: Vec<PremiseRef>) -> ProofHint {
        definition_hint_with_requests(
            citations,
            vec![transparent_request(CoreDefinitionId::new(0))],
        )
    }

    fn definition_hint_with_requests(
        citations: Vec<PremiseRef>,
        unfold_requests: Vec<DefinitionUnfoldRequest>,
    ) -> ProofHint {
        ProofHint {
            citations,
            unfold_requests,
            premise_restrictions: Vec::new(),
            solver: Some(ProofHintKey::new("task-11-definition")),
            max_axioms: None,
            timeout: None,
            computation: None,
            provenance: vec![provenance("definition-hint")],
        }
    }

    fn transparent_request(definition: CoreDefinitionId) -> DefinitionUnfoldRequest {
        DefinitionUnfoldRequest {
            definition,
            opacity_override: Some(DefinitionOpacityOverride::Transparent),
        }
    }

    fn opaque_request(definition: CoreDefinitionId) -> DefinitionUnfoldRequest {
        DefinitionUnfoldRequest {
            definition,
            opacity_override: Some(DefinitionOpacityOverride::Opaque),
        }
    }

    fn computation_hint(max_axioms: u32, citations: Vec<PremiseRef>) -> ProofHint {
        ProofHint {
            citations,
            unfold_requests: Vec::new(),
            premise_restrictions: vec![PremiseRestriction::Intent(ProofHintKey::new(
                "task-11-computation",
            ))],
            solver: Some(ProofHintKey::new("task-11-computation")),
            max_axioms: Some(max_axioms),
            timeout: Some(ProofHintKey::new(DEFAULT_COMPUTATION_LIMIT_POLICY)),
            computation: Some(ComputationHint::ByComputation),
            provenance: vec![provenance("computation-hint")],
        }
    }

    fn push_second_unsupported_vc(parts: &mut VcSetParts) {
        push_second_vc(
            parts,
            VcStatus::NeedsAtp,
            VcFormulaRef::Core(CoreFormulaId::new(99)),
            Vec::new(),
        );
    }

    fn push_second_vc(
        parts: &mut VcSetParts,
        status: VcStatus,
        goal: VcFormulaRef,
        generated_formulas: Vec<VcGeneratedFormula>,
    ) {
        parts.generated_formulas.extend(generated_formulas);
        let mut second_vc = parts.vcs[0].clone();
        second_vc.id = VcId::new(1);
        second_vc.seed = SeedVcRef {
            handoff: ObligationHandoffId::new(1),
        };
        second_vc.goal = goal;
        second_vc.status = status;
        second_vc.provenance = vec![provenance("vc-second")];
        parts.vcs.push(second_vc);
        parts.seed_accounting.push(SeedAccounting {
            handoff: ObligationHandoffId::new(1),
            origin: SeedOriginRef::ExistingCore {
                seed: ObligationSeedId::new(1),
            },
            seed_status: ObligationSeedStatus::Active,
            mapping: SeedVcMapping::One { vc: VcId::new(1) },
        });
    }

    fn incomplete_anchor(
        source: SourceId,
        source_ref: mizar_core::core_ir::CoreSourceRef,
    ) -> crate::vc_ir::ObligationAnchor {
        crate::vc_ir::ObligationAnchor {
            owner: AnchorOwner::Theorem(CoreItemId::new(0)),
            kind: VcKind::TheoremProofStep,
            local_path: LocalProofOrProgramPath::new("proof/step/0"),
            label: Some(AnchorLabel {
                role: AnchorLabelRole::UserLabel,
                hint: Some(CoreLabelRef::new("A1")),
            }),
            semantic_origin: NormalizedSemanticOrigin::new("theorem:sample:proof-step:0"),
            source_range: Some(SourceRange {
                source_id: source,
                start: 0,
                end: 10,
            }),
            provenance: vec![provenance("anchor")],
            source_shape_hash: HashMarker::Unavailable {
                reason: AnchorUnavailableReason::new("task-11 fixture lacks source-shape hash"),
            },
            canonical_goal_hash: HashMarker::Unavailable {
                reason: AnchorUnavailableReason::new("task-11 fixture lacks goal hash"),
            },
            canonical_context_hash: HashMarker::Unavailable {
                reason: AnchorUnavailableReason::new("task-11 fixture lacks context hash"),
            },
            generation_schema_version: GenerationSchemaVersion::new("task-11-test"),
            completeness: AnchorCompleteness::Incomplete {
                missing: vec![
                    AnchorIngredient::SourceShapeHash,
                    AnchorIngredient::CanonicalGoalHash,
                    AnchorIngredient::CanonicalContextHash,
                ],
            },
        }
        .with_extra_provenance(source_ref)
    }

    trait TestAnchorExt {
        fn with_extra_provenance(self, source_ref: mizar_core::core_ir::CoreSourceRef) -> Self;
    }

    impl TestAnchorExt for crate::vc_ir::ObligationAnchor {
        fn with_extra_provenance(mut self, source_ref: mizar_core::core_ir::CoreSourceRef) -> Self {
            self.provenance.push(VcProvenance {
                phase: VcProvenancePhase::CoreHandoff,
                key: VcText::new("source-ref"),
                core: source_ref.provenance.first().cloned(),
            });
            self
        }
    }

    fn source_ref(source: SourceId) -> mizar_core::core_ir::CoreSourceRef {
        mizar_core::core_ir::CoreSourceRef::direct(SourceRange {
            source_id: source,
            start: 0,
            end: 10,
        })
        .with_provenance(vec![CoreProvenance::new(
            CoreProvenancePhase::ProofSkeleton,
            "direct-source",
        )])
    }

    fn generated_source_ref() -> mizar_core::core_ir::CoreSourceRef {
        mizar_core::core_ir::CoreSourceRef::generated(GeneratedFrom {
            owner: CoreNodeRef::Item(CoreItemId::new(0)),
            kind: GeneratedOriginKind::TypePredicate,
            key: GeneratedOriginKey::new("generated-type"),
            reason: CoreProvenanceKey::new("task-11-test"),
        })
    }

    fn sample_snapshot_id() -> BuildSnapshotId {
        BuildSnapshotId::from_published_schema_str(
            "mizar-session-build-snapshot-v1:\
             3333333333333333333333333333333333333333333333333333333333333333",
        )
        .expect("snapshot id")
    }

    fn provenance(key: &str) -> VcProvenance {
        VcProvenance {
            phase: VcProvenancePhase::Generator,
            key: VcText::new(key),
            core: None,
        }
    }
}
