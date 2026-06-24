//! Prover-independent verification-condition IR data shapes.
//!
//! This module implements the data-layer contract specified in
//! [vc_ir.md](../../../doc/design/mizar-vc/en/vc_ir.md).

use mizar_core::{
    control_flow::{
        ControlFlowId, ControlFlowObligationSiteKind, ObligationHandoffEntry, ObligationHandoffId,
        ObligationHandoffOrigin, ObligationSeedHandoff,
    },
    core_ir::{
        CoreAlgorithmId, CoreDefinitionId, CoreDiagnosticId, CoreFormulaId, CoreItemId,
        CoreLabelRef, CoreProvenance, CoreSourceRef, CoreVarId, LocalProofOrProgramPath,
        NormalizedSemanticOrigin, ObligationSeedCanonicalKey, ObligationSeedId, ObligationSeedKind,
        ObligationSeedStatus,
    },
};
use mizar_session::{BuildSnapshotId, Hash, SourceId, SourceRange};
use std::{
    collections::{BTreeMap, BTreeSet},
    error::Error,
    fmt::{self, Write as _},
};

macro_rules! dense_id {
    ($name:ident) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $name(usize);

        impl $name {
            pub const fn new(index: usize) -> Self {
                Self(index)
            }

            pub const fn index(self) -> usize {
                self.0
            }
        }
    };
}

macro_rules! string_key {
    ($name:ident) => {
        #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $name(String);

        impl $name {
            pub fn new(value: impl Into<String>) -> Self {
                Self(value.into())
            }

            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl From<&str> for $name {
            fn from(value: &str) -> Self {
                Self::new(value)
            }
        }

        impl From<String> for $name {
            fn from(value: String) -> Self {
                Self::new(value)
            }
        }
    };
}

dense_id!(VcId);
dense_id!(VcGeneratedFormulaId);
dense_id!(ContextEntryId);

string_key!(VcSchemaVersion);
string_key!(GenerationSchemaVersion);
string_key!(ExpansionSchemaVersion);
string_key!(VcModuleRef);
string_key!(CanonicalSortKey);
string_key!(PolicyKey);
string_key!(PolicyValue);
string_key!(ProofHintKey);
string_key!(AnchorUnavailableReason);
string_key!(VcText);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CanonicalVcFingerprint(Hash);

impl CanonicalVcFingerprint {
    pub const fn hash(self) -> Hash {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LocalContextFingerprint(Hash);

impl LocalContextFingerprint {
    pub const fn hash(self) -> Hash {
        self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VcSet {
    schema_version: VcSchemaVersion,
    snapshot: BuildSnapshotId,
    source: SourceId,
    module: VcModuleRef,
    generated_formulas: Vec<VcGeneratedFormula>,
    vcs: Vec<VcIr>,
    seed_accounting: Vec<SeedAccounting>,
}

impl VcSet {
    pub fn try_new(parts: VcSetParts) -> Result<Self, VcIrError> {
        validate_vc_set_parts(&parts)?;
        Ok(Self {
            schema_version: parts.schema_version,
            snapshot: parts.snapshot,
            source: parts.source,
            module: parts.module,
            generated_formulas: parts.generated_formulas,
            vcs: parts.vcs,
            seed_accounting: parts.seed_accounting,
        })
    }

    pub const fn schema_version(&self) -> &VcSchemaVersion {
        &self.schema_version
    }

    pub const fn snapshot(&self) -> BuildSnapshotId {
        self.snapshot
    }

    pub const fn source(&self) -> SourceId {
        self.source
    }

    pub const fn module(&self) -> &VcModuleRef {
        &self.module
    }

    pub fn generated_formulas(&self) -> &[VcGeneratedFormula] {
        &self.generated_formulas
    }

    pub fn vcs(&self) -> &[VcIr] {
        &self.vcs
    }

    pub fn seed_accounting(&self) -> &[SeedAccounting] {
        &self.seed_accounting
    }

    pub fn vc(&self, id: VcId) -> Option<&VcIr> {
        self.vcs.get(id.index()).filter(|vc| vc.id == id)
    }

    pub fn generated_formula(&self, id: VcGeneratedFormulaId) -> Option<&VcGeneratedFormula> {
        self.generated_formulas
            .get(id.index())
            .filter(|formula| formula.id == id)
    }

    pub fn seed_accounting_for(&self, handoff: ObligationHandoffId) -> Option<&SeedAccounting> {
        self.seed_accounting
            .iter()
            .find(|accounting| accounting.handoff == handoff)
    }

    pub fn try_with_status_plan(&self, plan: &VcStatusPlan) -> Result<Self, VcIrError> {
        validate_status_plan_targets(plan, &self.vcs)?;
        let overrides = plan
            .overrides()
            .iter()
            .map(|entry| (entry.vc, &entry.action))
            .collect::<BTreeMap<_, _>>();
        let vcs = self
            .vcs
            .iter()
            .map(|vc| {
                project_vc_status(
                    vc,
                    overrides
                        .get(&vc.id)
                        .copied()
                        .unwrap_or_else(|| plan.default()),
                )
            })
            .collect();

        Self::try_new(VcSetParts {
            schema_version: self.schema_version.clone(),
            snapshot: self.snapshot,
            source: self.source,
            module: self.module.clone(),
            generated_formulas: self.generated_formulas.clone(),
            vcs,
            seed_accounting: self.seed_accounting.clone(),
        })
    }

    pub fn debug_text(&self) -> String {
        let mut output = String::from("vc-set-debug-v1\n");
        writeln!(&mut output, "schema-version: {:?}", self.schema_version).expect("write string");
        writeln!(&mut output, "snapshot: {:?}", self.snapshot).expect("write string");
        writeln!(&mut output, "source: {:?}", self.source).expect("write string");
        writeln!(&mut output, "module: {:?}", self.module).expect("write string");
        write_generated_formulas(&mut output, &self.generated_formulas);
        write_seed_accounting(&mut output, &self.seed_accounting);
        write_vcs(&mut output, &self.vcs);
        output
    }

    pub fn canonical_vc_fingerprint(&self, vc: VcId) -> Option<CanonicalVcFingerprint> {
        let vc = self.vc(vc)?;
        canonical_vc_fingerprint(vc, &generated_formula_map(&self.generated_formulas))
    }

    pub fn local_context_fingerprint(&self, vc: VcId) -> Option<LocalContextFingerprint> {
        let vc = self.vc(vc)?;
        local_context_fingerprint(
            &vc.local_context,
            &generated_formula_map(&self.generated_formulas),
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VcSetParts {
    pub schema_version: VcSchemaVersion,
    pub snapshot: BuildSnapshotId,
    pub source: SourceId,
    pub module: VcModuleRef,
    pub generated_formulas: Vec<VcGeneratedFormula>,
    pub vcs: Vec<VcIr>,
    pub seed_accounting: Vec<SeedAccounting>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VcStatusPlan {
    default: VcStatusAction,
    overrides: Vec<VcStatusOverride>,
}

impl VcStatusPlan {
    pub fn try_new(
        default: VcStatusAction,
        overrides: Vec<VcStatusOverride>,
    ) -> Result<Self, VcIrError> {
        validate_status_overrides(&overrides)?;
        Ok(Self { default, overrides })
    }

    pub const fn default(&self) -> &VcStatusAction {
        &self.default
    }

    pub fn overrides(&self) -> &[VcStatusOverride] {
        &self.overrides
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VcStatusOverride {
    pub vc: VcId,
    pub action: VcStatusAction,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum VcStatusAction {
    Preserve,
    NeedsAtp,
    PolicyOpen {
        policy: PolicyKey,
    },
    AssumeByPolicy {
        policy: PolicyKey,
        marker: PremiseRef,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VcIr {
    pub id: VcId,
    pub kind: VcKind,
    pub source: VcSourceRef,
    pub seed: SeedVcRef,
    pub anchor: ObligationAnchor,
    pub local_context: LocalContext,
    pub premises: Vec<PremiseRef>,
    pub goal: VcFormulaRef,
    pub proof_hint: Option<ProofHint>,
    pub status: VcStatus,
    pub provenance: Vec<VcProvenance>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VcSourceRef {
    pub primary: CoreSourceRef,
    pub related: Vec<CoreSourceRef>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SeedVcRef {
    pub handoff: ObligationHandoffId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VcGeneratedFormula {
    pub id: VcGeneratedFormulaId,
    pub kind: VcGeneratedFormulaKind,
    pub shape: VcGeneratedFormulaShape,
    pub provenance: Vec<VcProvenance>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum VcGeneratedFormulaKind {
    Conjunction,
    SplitGoal,
    NegatedPremise,
    GeneratedTypeObligation,
    AlgorithmPathCondition,
    PolicyMarker,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum VcGeneratedFormulaShape {
    True,
    False,
    Ref(VcFormulaRef),
    Not(VcFormulaRef),
    And(Vec<VcFormulaRef>),
    Or(Vec<VcFormulaRef>),
    Implies {
        premise: VcFormulaRef,
        conclusion: VcFormulaRef,
    },
    Quantified {
        kind: QuantifierKind,
        binders: Vec<ContextEntryId>,
        body: VcFormulaRef,
    },
    Diagnostic(CoreDiagnosticId),
}

impl VcGeneratedFormulaShape {
    fn referenced_formulas(&self) -> Vec<VcFormulaRef> {
        match self {
            Self::True | Self::False | Self::Diagnostic(_) => Vec::new(),
            Self::Ref(formula) | Self::Not(formula) => vec![*formula],
            Self::And(formulas) | Self::Or(formulas) => formulas.clone(),
            Self::Implies {
                premise,
                conclusion,
            } => vec![*premise, *conclusion],
            Self::Quantified { body, .. } => vec![*body],
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum QuantifierKind {
    Forall,
    Exists,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum VcFormulaRef {
    Core(CoreFormulaId),
    Generated(VcGeneratedFormulaId),
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum VcKind {
    TheoremProofStep,
    TerminalProofGoal,
    DefinitionCorrectness,
    RegistrationStyleCorrectness {
        style: RegistrationCorrectnessKind,
    },
    CheckerInitial,
    GeneratedNonEmptiness,
    GeneratedSethood,
    FraenkelMembershipAxiom,
    AlgorithmPrecondition,
    AlgorithmPostcondition,
    CallPrecondition,
    AlgorithmAssertion,
    LoopInvariant {
        phase: LoopInvariantPhase,
    },
    RangeLoop {
        obligation: RangeLoopObligation,
    },
    CollectionLoop {
        obligation: CollectionLoopObligation,
    },
    Termination,
    PartialTermination,
    GhostErasureSafety,
    PolicyDeferredTraceability,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum RegistrationCorrectnessKind {
    Registration,
    Redefinition,
    Reduction,
    ExplicitCoreSeed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum LoopInvariantPhase {
    Entry,
    Preservation,
    Break,
    Continue,
    Exit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum RangeLoopObligation {
    PositiveStep,
    RangeBound,
    HiddenIndex,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum CollectionLoopObligation {
    Finiteness,
    OrderIndependence,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalContext {
    entries: Vec<ContextEntry>,
    policy_inputs: Vec<VerifierPolicyInput>,
}

impl LocalContext {
    pub fn try_new(
        entries: Vec<ContextEntry>,
        policy_inputs: Vec<VerifierPolicyInput>,
    ) -> Result<Self, VcIrError> {
        validate_context_entries(&entries)?;
        validate_policy_inputs(&policy_inputs)?;
        Ok(Self {
            entries,
            policy_inputs,
        })
    }

    pub fn entries(&self) -> &[ContextEntry] {
        &self.entries
    }

    pub fn policy_inputs(&self) -> &[VerifierPolicyInput] {
        &self.policy_inputs
    }

    fn validate(&self) -> Result<(), VcIrError> {
        validate_context_entries(&self.entries)?;
        validate_policy_inputs(&self.policy_inputs)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextEntry {
    pub id: ContextEntryId,
    pub sort_key: CanonicalSortKey,
    pub kind: ContextEntryKind,
    pub formula: Option<VcFormulaRef>,
    pub provenance: Vec<VcProvenance>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ContextEntryKind {
    BinderDeclaration { var: CoreVarId, role: VcText },
    TypePredicate,
    SethoodFact,
    NonEmptinessFact,
    ProofAssumption,
    CurrentThesis,
    LocalLabel { label: CoreLabelRef },
    CitedPremise,
    AlgorithmPathCondition,
    LoopInvariantAvailable,
    PostHavocFact,
    CheckerFact,
    RegistrationTrace,
    QuaEvidence,
    DefinitionUnfoldingPolicy,
    VerifierPolicyInput { key: PolicyKey },
    GeneratedFact,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct VerifierPolicyInput {
    pub sort_key: CanonicalSortKey,
    pub key: PolicyKey,
    pub value: PolicyValue,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
pub enum PremiseRef {
    LocalContext(ContextEntryId),
    LocalLabel { label: CoreLabelRef },
    ImportedFact { symbol: VcText },
    DefinitionBoundary { definition: CoreDefinitionId },
    PermittedUnfolding { definition: CoreDefinitionId },
    CheckerFact { formula: CoreFormulaId },
    TypePredicate { formula: CoreFormulaId },
    RegistrationTrace { trace: VcText },
    ClusterTrace { trace: VcText },
    ReductionTrace { trace: VcText },
    GeneratedFact { formula: VcFormulaRef },
    PolicyAssumption { marker: PolicyKey },
    ConservativeUnknown { reason: VcText },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProofHint {
    pub citations: Vec<PremiseRef>,
    pub unfold_requests: Vec<DefinitionUnfoldRequest>,
    pub premise_restrictions: Vec<PremiseRestriction>,
    pub solver: Option<ProofHintKey>,
    pub max_axioms: Option<u32>,
    pub timeout: Option<ProofHintKey>,
    pub computation: Option<ComputationHint>,
    pub provenance: Vec<VcProvenance>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct DefinitionUnfoldRequest {
    pub definition: CoreDefinitionId,
    pub opacity_override: Option<DefinitionOpacityOverride>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum DefinitionOpacityOverride {
    Transparent,
    Opaque,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
pub enum PremiseRestriction {
    Only(Vec<PremiseRef>),
    Exclude(Vec<PremiseRef>),
    Intent(ProofHintKey),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
pub enum ComputationHint {
    ByComputation,
    LimitPolicy(PolicyKey),
    SymbolicRequest(ProofHintKey),
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum VcStatus {
    Open,
    NeedsAtp,
    Discharged {
        evidence: DischargeEvidenceRef,
    },
    PolicyOpen {
        policy: PolicyKey,
    },
    AssumedByPolicy {
        policy: PolicyKey,
        marker: PremiseRef,
    },
    SkippedDueToInvalidInput {
        reason: VcText,
    },
    DeferredExternal {
        reason: VcText,
    },
    Error {
        diagnostic: CoreDiagnosticId,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DischargeEvidenceRef {
    pub rule: VcText,
    pub evidence_hash: HashMarker,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SeedAccounting {
    pub handoff: ObligationHandoffId,
    pub origin: SeedOriginRef,
    pub seed_status: ObligationSeedStatus,
    pub mapping: SeedVcMapping,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SeedIntakeTable {
    rows: Vec<SeedIntakeRow>,
}

impl SeedIntakeTable {
    pub fn try_from_handoff(handoff: &ObligationSeedHandoff) -> Result<Self, VcIrError> {
        let mut rows = Vec::new();
        let mut seen = BTreeMap::new();

        for (handoff_id, entry) in handoff.entries.iter() {
            let origin = seed_origin_ref(&entry.origin);
            let canonical_key = entry.seed.canonical_key();
            if let Some(first_handoff) =
                seen.insert((canonical_key.clone(), origin.clone()), handoff_id)
            {
                return Err(VcIrError::DuplicateSeedIntake {
                    first_handoff,
                    second_handoff: handoff_id,
                });
            }
            let source = handoff.source_map.get(&handoff_id).cloned().ok_or(
                VcIrError::MissingHandoffSource {
                    handoff: handoff_id,
                },
            )?;

            rows.push(SeedIntakeRow {
                handoff: handoff_id,
                origin,
                seed_status: entry.seed.status,
                canonical_key,
                source,
                mapping: intake_mapping(entry),
            });
        }

        Ok(Self { rows })
    }

    pub fn rows(&self) -> &[SeedIntakeRow] {
        &self.rows
    }

    pub fn debug_text(&self) -> String {
        let mut output = String::from("seed-intake-table-debug-v1\n");
        for row in &self.rows {
            writeln!(
                &mut output,
                "handoff {:?}: origin={:?}; status={:?}; canonical-key={:?}; source={:?}; mapping={:?}",
                row.handoff, row.origin, row.seed_status, row.canonical_key, row.source, row.mapping
            )
            .expect("write string");
        }
        output
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SeedIntakeRow {
    pub handoff: ObligationHandoffId,
    pub origin: SeedOriginRef,
    pub seed_status: ObligationSeedStatus,
    pub canonical_key: ObligationSeedCanonicalKey,
    pub source: CoreSourceRef,
    pub mapping: SeedIntakeMapping,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum SeedIntakeMapping {
    EligibleOneVc { goal: CoreFormulaId },
    NoConcreteVc { reason: SeedNoVcReason },
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
pub enum SeedOriginRef {
    ExistingCore {
        seed: ObligationSeedId,
    },
    FlowDerived {
        flow: ControlFlowId,
        algorithm: CoreAlgorithmId,
    },
    Unsupported {
        origin: VcText,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum SeedVcMapping {
    NoConcreteVc {
        reason: SeedNoVcReason,
    },
    One {
        vc: VcId,
    },
    Expanded {
        vcs: Vec<ExpandedVcRef>,
        expansion_schema: ExpansionSchemaVersion,
    },
}

impl SeedVcMapping {
    fn referenced_vcs(&self) -> Vec<VcId> {
        match self {
            Self::NoConcreteVc { .. } => Vec::new(),
            Self::One { vc } => vec![*vc],
            Self::Expanded { vcs, .. } => vcs.iter().map(|entry| entry.vc).collect(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum SeedNoVcReason {
    SkippedInvalidInput,
    DeferredExternal(VcText),
    MissingGoal(VcText),
    PolicyDisabled(PolicyKey),
    Error(CoreDiagnosticId),
    ErrorWithoutDiagnostic(VcText),
}

fn seed_origin_ref(origin: &ObligationHandoffOrigin) -> SeedOriginRef {
    match origin {
        ObligationHandoffOrigin::ExistingCore { seed } => {
            SeedOriginRef::ExistingCore { seed: *seed }
        }
        ObligationHandoffOrigin::FlowDerived { flow, algorithm } => SeedOriginRef::FlowDerived {
            flow: *flow,
            algorithm: *algorithm,
        },
        origin => SeedOriginRef::Unsupported {
            origin: VcText::new(format!("{origin:?}")),
        },
    }
}

fn intake_mapping(entry: &ObligationHandoffEntry) -> SeedIntakeMapping {
    let status = entry.seed.status;
    let goal = entry.seed.goal;
    let diagnostics = &entry.seed.diagnostics;

    if let Some(goal) = eligible_deferred_flow_goal(entry) {
        return SeedIntakeMapping::EligibleOneVc { goal };
    }

    match (status, goal) {
        (ObligationSeedStatus::Active, Some(goal)) => SeedIntakeMapping::EligibleOneVc { goal },
        (ObligationSeedStatus::Active, None) => SeedIntakeMapping::NoConcreteVc {
            reason: SeedNoVcReason::MissingGoal(VcText::new("active seed has no goal")),
        },
        (ObligationSeedStatus::Skipped, _) => SeedIntakeMapping::NoConcreteVc {
            reason: SeedNoVcReason::SkippedInvalidInput,
        },
        (ObligationSeedStatus::Deferred, _) => SeedIntakeMapping::NoConcreteVc {
            reason: SeedNoVcReason::DeferredExternal(VcText::new("seed status is deferred")),
        },
        (ObligationSeedStatus::Error, _) => SeedIntakeMapping::NoConcreteVc {
            reason: diagnostics.first().copied().map_or_else(
                || {
                    SeedNoVcReason::ErrorWithoutDiagnostic(VcText::new(
                        "error seed has no diagnostic",
                    ))
                },
                SeedNoVcReason::Error,
            ),
        },
        (status, _) => SeedIntakeMapping::NoConcreteVc {
            reason: SeedNoVcReason::DeferredExternal(VcText::new(format!(
                "unsupported seed status {status:?}"
            ))),
        },
    }
}

fn eligible_deferred_flow_goal(entry: &ObligationHandoffEntry) -> Option<CoreFormulaId> {
    if entry.seed.status != ObligationSeedStatus::Deferred
        || !matches!(entry.origin, ObligationHandoffOrigin::FlowDerived { .. })
        || !matches!(entry.seed.kind, ObligationSeedKind::AlgorithmContract)
    {
        return None;
    }

    let site = entry.flow_site.as_ref()?;
    if !matches!(
        site.kind,
        ControlFlowObligationSiteKind::Requires
            | ControlFlowObligationSiteKind::Ensures
            | ControlFlowObligationSiteKind::AlgorithmAssertion
            | ControlFlowObligationSiteKind::StatementAssertion
            | ControlFlowObligationSiteKind::AlgorithmInvariant
            | ControlFlowObligationSiteKind::LoopInvariant
    ) {
        return None;
    }

    entry.seed.goal
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ExpandedVcRef {
    pub expansion_index: usize,
    pub vc: VcId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObligationAnchor {
    pub owner: AnchorOwner,
    pub kind: VcKind,
    pub local_path: LocalProofOrProgramPath,
    pub label: Option<AnchorLabel>,
    pub semantic_origin: NormalizedSemanticOrigin,
    pub source_range: Option<SourceRange>,
    pub provenance: Vec<VcProvenance>,
    pub source_shape_hash: HashMarker,
    pub canonical_goal_hash: HashMarker,
    pub canonical_context_hash: HashMarker,
    pub generation_schema_version: GenerationSchemaVersion,
    pub completeness: AnchorCompleteness,
}

impl ObligationAnchor {
    pub fn is_complete(&self) -> bool {
        matches!(self.completeness, AnchorCompleteness::Complete)
    }

    fn validate(&self) -> Result<(), VcIrError> {
        match &self.completeness {
            AnchorCompleteness::Complete => {
                for (ingredient, marker) in [
                    (AnchorIngredient::SourceShapeHash, &self.source_shape_hash),
                    (
                        AnchorIngredient::CanonicalGoalHash,
                        &self.canonical_goal_hash,
                    ),
                    (
                        AnchorIngredient::CanonicalContextHash,
                        &self.canonical_context_hash,
                    ),
                ] {
                    if !marker.is_available() {
                        return Err(VcIrError::CompleteAnchorMissingIngredient { ingredient });
                    }
                }
                if self.provenance.is_empty() {
                    return Err(VcIrError::CompleteAnchorMissingIngredient {
                        ingredient: AnchorIngredient::SourceProvenance,
                    });
                }
                Ok(())
            }
            AnchorCompleteness::Incomplete { missing } => {
                if missing.is_empty() {
                    return Err(VcIrError::IncompleteAnchorWithoutMissingIngredients);
                }
                validate_anchor_missing_ingredients(missing)?;
                self.validate_incomplete_missing_markers(missing)
            }
        }
    }

    fn validate_incomplete_missing_markers(
        &self,
        missing: &[AnchorIngredient],
    ) -> Result<(), VcIrError> {
        let declared = missing.iter().copied().collect::<BTreeSet<_>>();
        let actual = self.checkable_missing_ingredients();

        for ingredient in &actual {
            if !declared.contains(ingredient) {
                return Err(VcIrError::IncompleteAnchorOmitsMissingIngredient {
                    ingredient: *ingredient,
                });
            }
        }

        for ingredient in &declared {
            if checkable_anchor_ingredient(*ingredient) && !actual.contains(ingredient) {
                return Err(VcIrError::IncompleteAnchorClaimsAvailableIngredient {
                    ingredient: *ingredient,
                });
            }
        }

        Ok(())
    }

    fn checkable_missing_ingredients(&self) -> BTreeSet<AnchorIngredient> {
        let mut missing = BTreeSet::new();
        if self.provenance.is_empty() {
            missing.insert(AnchorIngredient::SourceProvenance);
        }
        if !self.source_shape_hash.is_available() {
            missing.insert(AnchorIngredient::SourceShapeHash);
        }
        if !self.canonical_goal_hash.is_available() {
            missing.insert(AnchorIngredient::CanonicalGoalHash);
        }
        if !self.canonical_context_hash.is_available() {
            missing.insert(AnchorIngredient::CanonicalContextHash);
        }
        missing
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum AnchorOwner {
    Theorem(CoreItemId),
    Definition(CoreItemId),
    Registration(CoreItemId),
    GeneratedSymbol(CoreItemId),
    Algorithm(CoreAlgorithmId),
    ProofBlock(CoreItemId),
    CheckerOrigin(VcText),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnchorLabel {
    pub role: AnchorLabelRole,
    pub hint: Option<CoreLabelRef>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum AnchorLabelRole {
    UserLabel,
    GeneratedLabel,
    CitationOnly,
    Unavailable,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum AnchorCompleteness {
    Complete,
    Incomplete { missing: Vec<AnchorIngredient> },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum AnchorIngredient {
    Owner,
    Kind,
    LocalPath,
    LabelRole,
    SemanticOrigin,
    SourceProvenance,
    SourceShapeHash,
    CanonicalGoalHash,
    CanonicalContextHash,
    GenerationSchemaVersion,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VcProvenance {
    pub phase: VcProvenancePhase,
    pub key: VcText,
    pub core: Option<CoreProvenance>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum VcProvenancePhase {
    CoreHandoff,
    ControlFlow,
    SeedAccounting,
    Generator,
    Normalization,
    StatusPolicy,
    Discharge,
    DependencySlice,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum HashMarker {
    Available(Hash),
    Unavailable { reason: AnchorUnavailableReason },
    ConservativeUnknown { reason: AnchorUnavailableReason },
}

impl HashMarker {
    pub const fn is_available(&self) -> bool {
        matches!(self, Self::Available(_))
    }
}

pub(crate) fn hash_marker_for_payload(domain: &str, payload: &str) -> HashMarker {
    HashMarker::Available(stable_fingerprint_hash(domain, payload.as_bytes()))
}

pub(crate) fn canonical_goal_hash_marker(goal: VcFormulaRef) -> HashMarker {
    match goal {
        VcFormulaRef::Generated(_) => HashMarker::ConservativeUnknown {
            reason: AnchorUnavailableReason::new(
                "generated formula payload table is unavailable to anchor builder",
            ),
        },
        VcFormulaRef::Core(_) => HashMarker::ConservativeUnknown {
            reason: AnchorUnavailableReason::new("core formula payload is unavailable to mizar-vc"),
        },
    }
}

pub(crate) fn local_context_hash_marker(context: &LocalContext) -> HashMarker {
    let empty_generated = BTreeMap::new();
    local_context_fingerprint(context, &empty_generated).map_or_else(
        || HashMarker::ConservativeUnknown {
            reason: AnchorUnavailableReason::new(
                "local context contains unresolved core row payloads",
            ),
        },
        |fingerprint| HashMarker::Available(fingerprint.hash()),
    )
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum VcIrError {
    NonDenseVcId {
        expected: VcId,
        actual: VcId,
    },
    NonDenseGeneratedFormulaId {
        expected: VcGeneratedFormulaId,
        actual: VcGeneratedFormulaId,
    },
    NonDenseContextEntryId {
        expected: ContextEntryId,
        actual: ContextEntryId,
    },
    DuplicateContextSortKey {
        sort_key: CanonicalSortKey,
    },
    ContextEntriesNotSorted {
        previous: CanonicalSortKey,
        current: CanonicalSortKey,
    },
    PolicyInputsNotSorted {
        previous: CanonicalSortKey,
        current: CanonicalSortKey,
    },
    DuplicateStatusOverride {
        vc: VcId,
    },
    StatusOverridesNotSorted {
        previous: VcId,
        current: VcId,
    },
    MissingStatusOverrideTarget {
        vc: VcId,
    },
    DuplicateSeedAccounting {
        handoff: ObligationHandoffId,
    },
    DuplicateSeedIntake {
        first_handoff: ObligationHandoffId,
        second_handoff: ObligationHandoffId,
    },
    MissingHandoffSource {
        handoff: ObligationHandoffId,
    },
    SeedAccountingNotSorted {
        previous: ObligationHandoffId,
        current: ObligationHandoffId,
    },
    MissingSeedAccounting {
        vc: VcId,
        handoff: ObligationHandoffId,
    },
    MissingMappedVc {
        handoff: ObligationHandoffId,
        vc: VcId,
    },
    VcMappedFromWrongSeed {
        vc: VcId,
        expected_handoff: ObligationHandoffId,
        actual_handoff: ObligationHandoffId,
    },
    DuplicateVcMapping {
        vc: VcId,
        first_handoff: ObligationHandoffId,
        second_handoff: ObligationHandoffId,
    },
    VcMissingFromSeedMapping {
        vc: VcId,
        handoff: ObligationHandoffId,
    },
    MissingContextEntry {
        context: ContextEntryId,
    },
    MissingGeneratedFormula {
        formula: VcGeneratedFormulaId,
    },
    GeneratedFormulaCycle {
        formula: VcGeneratedFormulaId,
    },
    ExpandedMappingEmpty {
        handoff: ObligationHandoffId,
    },
    ExpandedMappingNotSorted {
        handoff: ObligationHandoffId,
        previous: usize,
        current: usize,
    },
    ExpandedMappingIndexGap {
        handoff: ObligationHandoffId,
        expected: usize,
        actual: usize,
    },
    CompleteAnchorMissingIngredient {
        ingredient: AnchorIngredient,
    },
    IncompleteAnchorWithoutMissingIngredients,
    DuplicateAnchorMissingIngredient {
        ingredient: AnchorIngredient,
    },
    AnchorMissingIngredientsNotSorted {
        previous: AnchorIngredient,
        current: AnchorIngredient,
    },
    IncompleteAnchorOmitsMissingIngredient {
        ingredient: AnchorIngredient,
    },
    IncompleteAnchorClaimsAvailableIngredient {
        ingredient: AnchorIngredient,
    },
}

impl fmt::Display for VcIrError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NonDenseVcId { expected, actual } => write!(
                formatter,
                "VC ids must be dense and sorted; expected {expected:?}, found {actual:?}"
            ),
            Self::NonDenseGeneratedFormulaId { expected, actual } => write!(
                formatter,
                "generated formula ids must be dense and sorted; expected {expected:?}, found {actual:?}"
            ),
            Self::NonDenseContextEntryId { expected, actual } => write!(
                formatter,
                "context entry ids must be dense and sorted; expected {expected:?}, found {actual:?}"
            ),
            Self::DuplicateContextSortKey { sort_key } => {
                write!(formatter, "duplicate context sort key {sort_key:?}")
            }
            Self::ContextEntriesNotSorted { previous, current } => write!(
                formatter,
                "context entries must be sorted; {current:?} appears after {previous:?}"
            ),
            Self::PolicyInputsNotSorted { previous, current } => write!(
                formatter,
                "policy inputs must be sorted; {current:?} appears after {previous:?}"
            ),
            Self::DuplicateStatusOverride { vc } => {
                write!(formatter, "duplicate status override for {vc:?}")
            }
            Self::StatusOverridesNotSorted { previous, current } => write!(
                formatter,
                "status overrides must be sorted; {current:?} appears after {previous:?}"
            ),
            Self::MissingStatusOverrideTarget { vc } => {
                write!(formatter, "status override targets missing {vc:?}")
            }
            Self::DuplicateSeedAccounting { handoff } => {
                write!(formatter, "duplicate seed accounting row for {handoff:?}")
            }
            Self::DuplicateSeedIntake {
                first_handoff,
                second_handoff,
            } => write!(
                formatter,
                "duplicate seed intake rows {first_handoff:?} and {second_handoff:?}"
            ),
            Self::MissingHandoffSource { handoff } => {
                write!(
                    formatter,
                    "missing handoff source map entry for {handoff:?}"
                )
            }
            Self::SeedAccountingNotSorted { previous, current } => write!(
                formatter,
                "seed accounting rows must be sorted; {current:?} appears after {previous:?}"
            ),
            Self::MissingSeedAccounting { vc, handoff } => {
                write!(
                    formatter,
                    "{vc:?} references missing seed accounting {handoff:?}"
                )
            }
            Self::MissingMappedVc { handoff, vc } => {
                write!(formatter, "{handoff:?} maps to missing {vc:?}")
            }
            Self::VcMappedFromWrongSeed {
                vc,
                expected_handoff,
                actual_handoff,
            } => write!(
                formatter,
                "{vc:?} belongs to {expected_handoff:?}, but {actual_handoff:?} maps to it"
            ),
            Self::DuplicateVcMapping {
                vc,
                first_handoff,
                second_handoff,
            } => write!(
                formatter,
                "{vc:?} is mapped more than once: {first_handoff:?} and {second_handoff:?}"
            ),
            Self::VcMissingFromSeedMapping { vc, handoff } => write!(
                formatter,
                "{vc:?} references {handoff:?}, but that row does not map back to the VC"
            ),
            Self::MissingContextEntry { context } => {
                write!(formatter, "missing local context entry {context:?}")
            }
            Self::MissingGeneratedFormula { formula } => {
                write!(formatter, "missing generated formula {formula:?}")
            }
            Self::GeneratedFormulaCycle { formula } => {
                write!(formatter, "generated formula cycle at {formula:?}")
            }
            Self::ExpandedMappingEmpty { handoff } => {
                write!(formatter, "{handoff:?} uses an empty expanded mapping")
            }
            Self::ExpandedMappingNotSorted {
                handoff,
                previous,
                current,
            } => write!(
                formatter,
                "{handoff:?} expanded mapping indices must be sorted; {current} appears after {previous}"
            ),
            Self::ExpandedMappingIndexGap {
                handoff,
                expected,
                actual,
            } => write!(
                formatter,
                "{handoff:?} expanded mapping indices must be dense from zero; expected {expected}, found {actual}"
            ),
            Self::CompleteAnchorMissingIngredient { ingredient } => {
                write!(formatter, "complete anchor has unavailable {ingredient:?}")
            }
            Self::IncompleteAnchorWithoutMissingIngredients => {
                formatter.write_str("incomplete anchor must record at least one missing ingredient")
            }
            Self::DuplicateAnchorMissingIngredient { ingredient } => {
                write!(
                    formatter,
                    "duplicate incomplete-anchor ingredient {ingredient:?}"
                )
            }
            Self::AnchorMissingIngredientsNotSorted { previous, current } => write!(
                formatter,
                "incomplete-anchor missing ingredients must be sorted; {current:?} appears after {previous:?}"
            ),
            Self::IncompleteAnchorOmitsMissingIngredient { ingredient } => write!(
                formatter,
                "incomplete anchor omits unavailable ingredient {ingredient:?}"
            ),
            Self::IncompleteAnchorClaimsAvailableIngredient { ingredient } => write!(
                formatter,
                "incomplete anchor claims available ingredient {ingredient:?} is missing"
            ),
        }
    }
}

impl Error for VcIrError {}

fn validate_vc_set_parts(parts: &VcSetParts) -> Result<(), VcIrError> {
    validate_generated_formulas(&parts.generated_formulas)?;
    validate_vcs(&parts.vcs, &parts.generated_formulas)?;
    validate_seed_accounting(&parts.seed_accounting, &parts.vcs)
}

fn validate_status_overrides(overrides: &[VcStatusOverride]) -> Result<(), VcIrError> {
    let mut previous = None;
    for override_entry in overrides {
        if let Some(previous_vc) = previous
            && override_entry.vc <= previous_vc
        {
            return if override_entry.vc == previous_vc {
                Err(VcIrError::DuplicateStatusOverride {
                    vc: override_entry.vc,
                })
            } else {
                Err(VcIrError::StatusOverridesNotSorted {
                    previous: previous_vc,
                    current: override_entry.vc,
                })
            };
        }
        previous = Some(override_entry.vc);
    }
    Ok(())
}

fn validate_status_plan_targets(plan: &VcStatusPlan, vcs: &[VcIr]) -> Result<(), VcIrError> {
    let vc_ids = vcs.iter().map(|vc| vc.id).collect::<BTreeSet<_>>();
    for override_entry in plan.overrides() {
        if !vc_ids.contains(&override_entry.vc) {
            return Err(VcIrError::MissingStatusOverrideTarget {
                vc: override_entry.vc,
            });
        }
    }
    Ok(())
}

fn project_vc_status(vc: &VcIr, action: &VcStatusAction) -> VcIr {
    let status = status_for_action(&vc.status, action);
    if status == vc.status {
        return vc.clone();
    }

    let mut projected = vc.clone();
    projected.status = status;
    projected.provenance.push(status_policy_provenance(action));
    projected
}

fn status_for_action(current: &VcStatus, action: &VcStatusAction) -> VcStatus {
    match action {
        VcStatusAction::Preserve => current.clone(),
        VcStatusAction::NeedsAtp => VcStatus::NeedsAtp,
        VcStatusAction::PolicyOpen { policy } => VcStatus::PolicyOpen {
            policy: policy.clone(),
        },
        VcStatusAction::AssumeByPolicy { policy, marker } => VcStatus::AssumedByPolicy {
            policy: policy.clone(),
            marker: marker.clone(),
        },
    }
}

fn status_policy_provenance(action: &VcStatusAction) -> VcProvenance {
    VcProvenance {
        phase: VcProvenancePhase::StatusPolicy,
        key: status_policy_key(action),
        core: None,
    }
}

fn status_policy_key(action: &VcStatusAction) -> VcText {
    match action {
        VcStatusAction::Preserve => VcText::new("task-9-status:preserve"),
        VcStatusAction::NeedsAtp => VcText::new("task-9-status:needs-atp"),
        VcStatusAction::PolicyOpen { policy } => {
            VcText::new(format!("task-9-status:policy-open:{}", policy.as_str()))
        }
        VcStatusAction::AssumeByPolicy { policy, .. } => VcText::new(format!(
            "task-9-status:assumed-by-policy:{}",
            policy.as_str()
        )),
    }
}

fn validate_generated_formulas(formulas: &[VcGeneratedFormula]) -> Result<(), VcIrError> {
    let formula_ids = formulas
        .iter()
        .map(|formula| formula.id)
        .collect::<BTreeSet<_>>();
    for (index, formula) in formulas.iter().enumerate() {
        let expected = VcGeneratedFormulaId::new(index);
        if formula.id != expected {
            return Err(VcIrError::NonDenseGeneratedFormulaId {
                expected,
                actual: formula.id,
            });
        }
        for referenced in formula.shape.referenced_formulas() {
            validate_formula_ref(referenced, &formula_ids)?;
        }
    }
    Ok(())
}

fn validate_vcs(vcs: &[VcIr], generated_formulas: &[VcGeneratedFormula]) -> Result<(), VcIrError> {
    let formula_ids = generated_formulas
        .iter()
        .map(|formula| formula.id)
        .collect::<BTreeSet<_>>();
    let generated_formula_map = generated_formulas
        .iter()
        .map(|formula| (formula.id, formula))
        .collect::<BTreeMap<_, _>>();
    for (index, vc) in vcs.iter().enumerate() {
        let expected = VcId::new(index);
        if vc.id != expected {
            return Err(VcIrError::NonDenseVcId {
                expected,
                actual: vc.id,
            });
        }
        vc.local_context.validate()?;
        let context_ids =
            validate_local_context_refs(&vc.local_context, &formula_ids, &generated_formula_map)?;
        validate_formula_ref_with_context(
            vc.goal,
            &formula_ids,
            &generated_formula_map,
            &context_ids,
        )?;
        for premise in &vc.premises {
            validate_premise_ref(premise, &formula_ids, &generated_formula_map, &context_ids)?;
        }
        if let Some(proof_hint) = &vc.proof_hint {
            validate_proof_hint(
                proof_hint,
                &formula_ids,
                &generated_formula_map,
                &context_ids,
            )?;
        }
        validate_status(
            &vc.status,
            &formula_ids,
            &generated_formula_map,
            &context_ids,
        )?;
        vc.anchor.validate()?;
    }
    Ok(())
}

fn validate_seed_accounting(rows: &[SeedAccounting], vcs: &[VcIr]) -> Result<(), VcIrError> {
    let vc_ids = vcs.iter().map(|vc| vc.id).collect::<BTreeSet<_>>();
    let vc_handoffs = vcs
        .iter()
        .map(|vc| (vc.id, vc.seed.handoff))
        .collect::<BTreeMap<_, _>>();
    let mut rows_by_handoff = BTreeMap::new();
    let mut mapped_once = BTreeMap::new();
    let mut previous = None;

    for row in rows {
        if let Some(previous_handoff) = previous
            && row.handoff <= previous_handoff
        {
            return if row.handoff == previous_handoff {
                Err(VcIrError::DuplicateSeedAccounting {
                    handoff: row.handoff,
                })
            } else {
                Err(VcIrError::SeedAccountingNotSorted {
                    previous: previous_handoff,
                    current: row.handoff,
                })
            };
        }
        previous = Some(row.handoff);
        validate_mapping(row, &vc_ids)?;
        let referenced_vcs = row.mapping.referenced_vcs();
        for vc in &referenced_vcs {
            let expected_handoff = vc_handoffs
                .get(vc)
                .expect("mapping target was validated as existing");
            if *expected_handoff != row.handoff {
                return Err(VcIrError::VcMappedFromWrongSeed {
                    vc: *vc,
                    expected_handoff: *expected_handoff,
                    actual_handoff: row.handoff,
                });
            }
            if let Some(first_handoff) = mapped_once.insert(*vc, row.handoff) {
                return Err(VcIrError::DuplicateVcMapping {
                    vc: *vc,
                    first_handoff,
                    second_handoff: row.handoff,
                });
            }
        }
        rows_by_handoff.insert(row.handoff, referenced_vcs);
    }

    for vc in vcs {
        let Some(mapped_vcs) = rows_by_handoff.get(&vc.seed.handoff) else {
            return Err(VcIrError::MissingSeedAccounting {
                vc: vc.id,
                handoff: vc.seed.handoff,
            });
        };
        if !mapped_vcs.contains(&vc.id) {
            return Err(VcIrError::VcMissingFromSeedMapping {
                vc: vc.id,
                handoff: vc.seed.handoff,
            });
        }
    }

    Ok(())
}

fn validate_mapping(row: &SeedAccounting, vc_ids: &BTreeSet<VcId>) -> Result<(), VcIrError> {
    match &row.mapping {
        SeedVcMapping::NoConcreteVc { .. } => Ok(()),
        SeedVcMapping::One { vc } => validate_mapped_vc(row.handoff, *vc, vc_ids),
        SeedVcMapping::Expanded { vcs, .. } => {
            if vcs.is_empty() {
                return Err(VcIrError::ExpandedMappingEmpty {
                    handoff: row.handoff,
                });
            }
            let mut previous = None;
            for (expected_index, expanded) in vcs.iter().enumerate() {
                if let Some(previous_index) = previous
                    && expanded.expansion_index <= previous_index
                {
                    return Err(VcIrError::ExpandedMappingNotSorted {
                        handoff: row.handoff,
                        previous: previous_index,
                        current: expanded.expansion_index,
                    });
                }
                if expanded.expansion_index != expected_index {
                    return Err(VcIrError::ExpandedMappingIndexGap {
                        handoff: row.handoff,
                        expected: expected_index,
                        actual: expanded.expansion_index,
                    });
                }
                previous = Some(expanded.expansion_index);
                validate_mapped_vc(row.handoff, expanded.vc, vc_ids)?;
            }
            Ok(())
        }
    }
}

fn validate_mapped_vc(
    handoff: ObligationHandoffId,
    vc: VcId,
    vc_ids: &BTreeSet<VcId>,
) -> Result<(), VcIrError> {
    if vc_ids.contains(&vc) {
        Ok(())
    } else {
        Err(VcIrError::MissingMappedVc { handoff, vc })
    }
}

fn validate_formula_ref(
    reference: VcFormulaRef,
    generated_formulas: &BTreeSet<VcGeneratedFormulaId>,
) -> Result<(), VcIrError> {
    match reference {
        VcFormulaRef::Core(_) => Ok(()),
        VcFormulaRef::Generated(formula) if generated_formulas.contains(&formula) => Ok(()),
        VcFormulaRef::Generated(formula) => Err(VcIrError::MissingGeneratedFormula { formula }),
    }
}

fn validate_premise_ref(
    premise: &PremiseRef,
    generated_formulas: &BTreeSet<VcGeneratedFormulaId>,
    generated_formula_map: &BTreeMap<VcGeneratedFormulaId, &VcGeneratedFormula>,
    context_entries: &BTreeSet<ContextEntryId>,
) -> Result<(), VcIrError> {
    match premise {
        PremiseRef::GeneratedFact { formula } => validate_formula_ref_with_context(
            *formula,
            generated_formulas,
            generated_formula_map,
            context_entries,
        ),
        PremiseRef::LocalContext(context) if context_entries.contains(context) => Ok(()),
        PremiseRef::LocalContext(context) => {
            Err(VcIrError::MissingContextEntry { context: *context })
        }
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
        | PremiseRef::ConservativeUnknown { .. } => Ok(()),
    }
}

fn validate_proof_hint(
    proof_hint: &ProofHint,
    generated_formulas: &BTreeSet<VcGeneratedFormulaId>,
    generated_formula_map: &BTreeMap<VcGeneratedFormulaId, &VcGeneratedFormula>,
    context_entries: &BTreeSet<ContextEntryId>,
) -> Result<(), VcIrError> {
    for premise in &proof_hint.citations {
        validate_premise_ref(
            premise,
            generated_formulas,
            generated_formula_map,
            context_entries,
        )?;
    }
    for restriction in &proof_hint.premise_restrictions {
        match restriction {
            PremiseRestriction::Only(premises) | PremiseRestriction::Exclude(premises) => {
                for premise in premises {
                    validate_premise_ref(
                        premise,
                        generated_formulas,
                        generated_formula_map,
                        context_entries,
                    )?;
                }
            }
            PremiseRestriction::Intent(_) => {}
        }
    }
    Ok(())
}

fn validate_local_context_refs(
    context: &LocalContext,
    generated_formulas: &BTreeSet<VcGeneratedFormulaId>,
    generated_formula_map: &BTreeMap<VcGeneratedFormulaId, &VcGeneratedFormula>,
) -> Result<BTreeSet<ContextEntryId>, VcIrError> {
    let mut ids = BTreeSet::new();
    for entry in context.entries() {
        ids.insert(entry.id);
    }
    for entry in context.entries() {
        if let Some(formula) = entry.formula {
            validate_formula_ref_with_context(
                formula,
                generated_formulas,
                generated_formula_map,
                &ids,
            )?;
        }
    }
    Ok(ids)
}

fn validate_status(
    status: &VcStatus,
    generated_formulas: &BTreeSet<VcGeneratedFormulaId>,
    generated_formula_map: &BTreeMap<VcGeneratedFormulaId, &VcGeneratedFormula>,
    context_entries: &BTreeSet<ContextEntryId>,
) -> Result<(), VcIrError> {
    match status {
        VcStatus::AssumedByPolicy { marker, .. } => validate_premise_ref(
            marker,
            generated_formulas,
            generated_formula_map,
            context_entries,
        ),
        VcStatus::Open
        | VcStatus::NeedsAtp
        | VcStatus::Discharged { .. }
        | VcStatus::PolicyOpen { .. }
        | VcStatus::SkippedDueToInvalidInput { .. }
        | VcStatus::DeferredExternal { .. }
        | VcStatus::Error { .. } => Ok(()),
    }
}

fn validate_formula_ref_with_context(
    reference: VcFormulaRef,
    generated_formulas: &BTreeSet<VcGeneratedFormulaId>,
    generated_formula_map: &BTreeMap<VcGeneratedFormulaId, &VcGeneratedFormula>,
    context_entries: &BTreeSet<ContextEntryId>,
) -> Result<(), VcIrError> {
    validate_formula_ref_with_context_inner(
        reference,
        generated_formulas,
        generated_formula_map,
        context_entries,
        &mut BTreeSet::new(),
    )
}

fn validate_formula_ref_with_context_inner(
    reference: VcFormulaRef,
    generated_formulas: &BTreeSet<VcGeneratedFormulaId>,
    generated_formula_map: &BTreeMap<VcGeneratedFormulaId, &VcGeneratedFormula>,
    context_entries: &BTreeSet<ContextEntryId>,
    active: &mut BTreeSet<VcGeneratedFormulaId>,
) -> Result<(), VcIrError> {
    match reference {
        VcFormulaRef::Core(_) => Ok(()),
        VcFormulaRef::Generated(formula) => {
            validate_formula_ref(reference, generated_formulas)?;
            if !active.insert(formula) {
                return Err(VcIrError::GeneratedFormulaCycle { formula });
            }
            let generated = generated_formula_map
                .get(&formula)
                .expect("generated formula existence already validated");
            validate_formula_shape_with_context(
                &generated.shape,
                generated_formulas,
                generated_formula_map,
                context_entries,
                active,
            )?;
            active.remove(&formula);
            Ok(())
        }
    }
}

fn validate_formula_shape_with_context(
    shape: &VcGeneratedFormulaShape,
    generated_formulas: &BTreeSet<VcGeneratedFormulaId>,
    generated_formula_map: &BTreeMap<VcGeneratedFormulaId, &VcGeneratedFormula>,
    context_entries: &BTreeSet<ContextEntryId>,
    active: &mut BTreeSet<VcGeneratedFormulaId>,
) -> Result<(), VcIrError> {
    if let VcGeneratedFormulaShape::Quantified { binders, body, .. } = shape {
        for binder in binders {
            if !context_entries.contains(binder) {
                return Err(VcIrError::MissingContextEntry { context: *binder });
            }
        }
        return validate_formula_ref_with_context_inner(
            *body,
            generated_formulas,
            generated_formula_map,
            context_entries,
            active,
        );
    }

    for referenced in shape.referenced_formulas() {
        validate_formula_ref_with_context_inner(
            referenced,
            generated_formulas,
            generated_formula_map,
            context_entries,
            active,
        )?;
    }
    Ok(())
}

fn validate_anchor_missing_ingredients(missing: &[AnchorIngredient]) -> Result<(), VcIrError> {
    let mut previous = None;
    for ingredient in missing {
        if let Some(previous_ingredient) = previous
            && *ingredient <= previous_ingredient
        {
            return if *ingredient == previous_ingredient {
                Err(VcIrError::DuplicateAnchorMissingIngredient {
                    ingredient: *ingredient,
                })
            } else {
                Err(VcIrError::AnchorMissingIngredientsNotSorted {
                    previous: previous_ingredient,
                    current: *ingredient,
                })
            };
        }
        previous = Some(*ingredient);
    }
    Ok(())
}

fn checkable_anchor_ingredient(ingredient: AnchorIngredient) -> bool {
    matches!(
        ingredient,
        AnchorIngredient::SourceProvenance
            | AnchorIngredient::SourceShapeHash
            | AnchorIngredient::CanonicalGoalHash
            | AnchorIngredient::CanonicalContextHash
    )
}

fn validate_context_entries(entries: &[ContextEntry]) -> Result<(), VcIrError> {
    let mut sort_keys = BTreeSet::new();
    let mut previous = None;
    for (index, entry) in entries.iter().enumerate() {
        let expected = ContextEntryId::new(index);
        if entry.id != expected {
            return Err(VcIrError::NonDenseContextEntryId {
                expected,
                actual: entry.id,
            });
        }
        if let Some(previous_key) = previous
            && entry.sort_key <= previous_key
        {
            return if entry.sort_key == previous_key {
                Err(VcIrError::DuplicateContextSortKey {
                    sort_key: entry.sort_key.clone(),
                })
            } else {
                Err(VcIrError::ContextEntriesNotSorted {
                    previous: previous_key,
                    current: entry.sort_key.clone(),
                })
            };
        }
        previous = Some(entry.sort_key.clone());
        if !sort_keys.insert(entry.sort_key.clone()) {
            return Err(VcIrError::DuplicateContextSortKey {
                sort_key: entry.sort_key.clone(),
            });
        }
    }
    Ok(())
}

fn validate_policy_inputs(policy_inputs: &[VerifierPolicyInput]) -> Result<(), VcIrError> {
    let mut previous = None;
    for input in policy_inputs {
        if let Some(previous_key) = previous
            && input.sort_key <= previous_key
        {
            return Err(VcIrError::PolicyInputsNotSorted {
                previous: previous_key,
                current: input.sort_key.clone(),
            });
        }
        previous = Some(input.sort_key.clone());
    }
    Ok(())
}

fn write_generated_formulas(output: &mut String, formulas: &[VcGeneratedFormula]) {
    writeln!(output, "[generated-formulas]").expect("write string");
    for formula in formulas {
        writeln!(
            output,
            "generated-formula {:?}: kind={:?}; shape={:?}; provenance={:?}",
            formula.id, formula.kind, formula.shape, formula.provenance
        )
        .expect("write string");
    }
}

fn write_seed_accounting(output: &mut String, rows: &[SeedAccounting]) {
    writeln!(output, "[seed-accounting]").expect("write string");
    for row in rows {
        writeln!(
            output,
            "seed {:?}: origin={:?}; status={:?}; mapping={:?}",
            row.handoff, row.origin, row.seed_status, row.mapping
        )
        .expect("write string");
    }
}

fn write_vcs(output: &mut String, vcs: &[VcIr]) {
    writeln!(output, "[vcs]").expect("write string");
    for vc in vcs {
        writeln!(
            output,
            "vc {:?}: kind={:?}; seed={:?}",
            vc.id, vc.kind, vc.seed
        )
        .expect("write string");
        writeln!(output, "  source: {:?}", vc.source).expect("write string");
        writeln!(output, "  anchor: {}", render_anchor(&vc.anchor)).expect("write string");
        writeln!(output, "  context: {:?}", vc.local_context.entries()).expect("write string");
        writeln!(
            output,
            "  policy-inputs: {:?}",
            vc.local_context.policy_inputs()
        )
        .expect("write string");
        writeln!(output, "  premises: {:?}", sorted_premises(&vc.premises)).expect("write string");
        writeln!(output, "  goal: {:?}", vc.goal).expect("write string");
        writeln!(output, "  proof-hint: {:?}", vc.proof_hint).expect("write string");
        writeln!(output, "  status: {:?}", vc.status).expect("write string");
        writeln!(output, "  provenance: {:?}", vc.provenance).expect("write string");
    }
}

fn sorted_premises(premises: &[PremiseRef]) -> Vec<PremiseRef> {
    let mut sorted = premises.to_vec();
    sorted.sort();
    sorted
}

fn generated_formula_map(
    generated_formulas: &[VcGeneratedFormula],
) -> BTreeMap<VcGeneratedFormulaId, &VcGeneratedFormula> {
    generated_formulas
        .iter()
        .map(|formula| (formula.id, formula))
        .collect()
}

fn canonical_vc_fingerprint(
    vc: &VcIr,
    generated_formulas: &BTreeMap<VcGeneratedFormulaId, &VcGeneratedFormula>,
) -> Option<CanonicalVcFingerprint> {
    let mut payload = String::from("canonical-vc-fingerprint-v1\n");
    let mut available = true;
    writeln!(&mut payload, "kind: {:?}", vc.kind).expect("write string");
    writeln!(&mut payload, "[goal]").expect("write string");
    available &= write_formula_payload(
        &mut payload,
        vc.goal,
        generated_formulas,
        &mut BTreeSet::new(),
    );
    writeln!(&mut payload, "[premises]").expect("write string");
    for premise in sorted_premises(&vc.premises) {
        available &= write_premise_payload(
            &mut payload,
            &premise,
            &vc.local_context,
            generated_formulas,
            &mut BTreeSet::new(),
        );
    }
    writeln!(&mut payload, "[proof-hint]").expect("write string");
    available &= write_proof_hint_payload(
        &mut payload,
        vc.proof_hint.as_ref(),
        &vc.local_context,
        generated_formulas,
    );
    available.then(|| {
        CanonicalVcFingerprint(stable_fingerprint_hash(
            "mizar-vc-canonical-vc",
            payload.as_bytes(),
        ))
    })
}

fn local_context_fingerprint(
    context: &LocalContext,
    generated_formulas: &BTreeMap<VcGeneratedFormulaId, &VcGeneratedFormula>,
) -> Option<LocalContextFingerprint> {
    let mut payload = String::from("local-context-fingerprint-v1\n");
    let mut available = true;
    writeln!(&mut payload, "[entries]").expect("write string");
    for entry in context.entries() {
        available &= write_context_entry_payload(&mut payload, entry, generated_formulas);
    }
    writeln!(&mut payload, "[policy-inputs]").expect("write string");
    for input in context.policy_inputs() {
        writeln!(
            &mut payload,
            "policy-input sort={:?}; key={:?}; value={:?}",
            input.sort_key, input.key, input.value
        )
        .expect("write string");
    }
    available.then(|| {
        LocalContextFingerprint(stable_fingerprint_hash(
            "mizar-vc-local-context",
            payload.as_bytes(),
        ))
    })
}

fn write_context_entry_payload(
    output: &mut String,
    entry: &ContextEntry,
    generated_formulas: &BTreeMap<VcGeneratedFormulaId, &VcGeneratedFormula>,
) -> bool {
    let mut available = !matches!(entry.kind, ContextEntryKind::BinderDeclaration { .. });
    writeln!(
        output,
        "context sort={:?}; kind={:?}",
        entry.sort_key, entry.kind
    )
    .expect("write string");
    if let Some(formula) = entry.formula {
        available &=
            write_formula_payload(output, formula, generated_formulas, &mut BTreeSet::new());
    } else {
        writeln!(output, "formula: <none>").expect("write string");
    }
    writeln!(output, "provenance: {:?}", entry.provenance).expect("write string");
    available
}

fn write_formula_payload(
    output: &mut String,
    formula: VcFormulaRef,
    generated_formulas: &BTreeMap<VcGeneratedFormulaId, &VcGeneratedFormula>,
    active_generated: &mut BTreeSet<VcGeneratedFormulaId>,
) -> bool {
    match formula {
        VcFormulaRef::Core(core) => {
            writeln!(output, "core-formula-unresolved: {core:?}").expect("write string");
            false
        }
        VcFormulaRef::Generated(generated) => {
            if !active_generated.insert(generated) {
                writeln!(output, "generated-formula-cycle: {generated:?}").expect("write string");
                return false;
            }
            let available = if let Some(formula) = generated_formulas.get(&generated) {
                writeln!(
                    output,
                    "generated-formula kind={:?}; provenance={:?}",
                    formula.kind, formula.provenance
                )
                .expect("write string");
                write_formula_shape_payload(
                    output,
                    &formula.shape,
                    generated_formulas,
                    active_generated,
                )
            } else {
                writeln!(output, "generated-formula-missing: {generated:?}").expect("write string");
                false
            };
            active_generated.remove(&generated);
            available
        }
    }
}

fn write_formula_shape_payload(
    output: &mut String,
    shape: &VcGeneratedFormulaShape,
    generated_formulas: &BTreeMap<VcGeneratedFormulaId, &VcGeneratedFormula>,
    active_generated: &mut BTreeSet<VcGeneratedFormulaId>,
) -> bool {
    match shape {
        VcGeneratedFormulaShape::True => {
            writeln!(output, "shape: true").expect("write string");
            true
        }
        VcGeneratedFormulaShape::False => {
            writeln!(output, "shape: false").expect("write string");
            true
        }
        VcGeneratedFormulaShape::Diagnostic(diagnostic) => {
            writeln!(output, "shape: diagnostic {diagnostic:?}").expect("write string");
            false
        }
        VcGeneratedFormulaShape::Ref(formula) => {
            writeln!(output, "shape: ref").expect("write string");
            write_formula_payload(output, *formula, generated_formulas, active_generated)
        }
        VcGeneratedFormulaShape::Not(formula) => {
            writeln!(output, "shape: not").expect("write string");
            write_formula_payload(output, *formula, generated_formulas, active_generated)
        }
        VcGeneratedFormulaShape::And(formulas) => {
            writeln!(output, "shape: and len={}", formulas.len()).expect("write string");
            let mut available = true;
            for formula in formulas {
                available &=
                    write_formula_payload(output, *formula, generated_formulas, active_generated);
            }
            available
        }
        VcGeneratedFormulaShape::Or(formulas) => {
            writeln!(output, "shape: or len={}", formulas.len()).expect("write string");
            let mut available = true;
            for formula in formulas {
                available &=
                    write_formula_payload(output, *formula, generated_formulas, active_generated);
            }
            available
        }
        VcGeneratedFormulaShape::Implies {
            premise,
            conclusion,
        } => {
            writeln!(output, "shape: implies premise").expect("write string");
            let premise_available =
                write_formula_payload(output, *premise, generated_formulas, active_generated);
            writeln!(output, "shape: implies conclusion").expect("write string");
            let conclusion_available =
                write_formula_payload(output, *conclusion, generated_formulas, active_generated);
            premise_available && conclusion_available
        }
        VcGeneratedFormulaShape::Quantified {
            kind,
            binders,
            body,
        } => {
            writeln!(
                output,
                "shape: quantified {kind:?}; binder-count={}",
                binders.len()
            )
            .expect("write string");
            let _ = write_formula_payload(output, *body, generated_formulas, active_generated);
            false
        }
    }
}

fn write_premise_payload(
    output: &mut String,
    premise: &PremiseRef,
    context: &LocalContext,
    generated_formulas: &BTreeMap<VcGeneratedFormulaId, &VcGeneratedFormula>,
    active_generated: &mut BTreeSet<VcGeneratedFormulaId>,
) -> bool {
    match premise {
        PremiseRef::LocalContext(context_id) => {
            writeln!(output, "premise: local-context").expect("write string");
            if let Some(entry) = context
                .entries()
                .iter()
                .find(|entry| entry.id == *context_id)
            {
                write_context_entry_payload(output, entry, generated_formulas)
            } else {
                writeln!(output, "premise-local-context-missing: {context_id:?}")
                    .expect("write string");
                false
            }
        }
        PremiseRef::GeneratedFact { formula } => {
            writeln!(output, "premise: generated-fact").expect("write string");
            write_formula_payload(output, *formula, generated_formulas, active_generated)
        }
        PremiseRef::DefinitionBoundary { .. }
        | PremiseRef::PermittedUnfolding { .. }
        | PremiseRef::CheckerFact { .. }
        | PremiseRef::TypePredicate { .. }
        | PremiseRef::ConservativeUnknown { .. } => {
            writeln!(output, "premise-unresolved: {premise:?}").expect("write string");
            false
        }
        other => {
            writeln!(output, "premise: {other:?}").expect("write string");
            true
        }
    }
}

fn write_proof_hint_payload(
    output: &mut String,
    proof_hint: Option<&ProofHint>,
    context: &LocalContext,
    generated_formulas: &BTreeMap<VcGeneratedFormulaId, &VcGeneratedFormula>,
) -> bool {
    let Some(proof_hint) = proof_hint else {
        writeln!(output, "<none>").expect("write string");
        return true;
    };

    let mut available = true;
    writeln!(output, "citations").expect("write string");
    for citation in &proof_hint.citations {
        available &= write_premise_payload(
            output,
            citation,
            context,
            generated_formulas,
            &mut BTreeSet::new(),
        );
    }
    writeln!(output, "unfold-requests: {:?}", proof_hint.unfold_requests).expect("write string");
    available &= proof_hint.unfold_requests.is_empty();
    writeln!(output, "premise-restrictions").expect("write string");
    for restriction in &proof_hint.premise_restrictions {
        available &=
            write_premise_restriction_payload(output, restriction, context, generated_formulas);
    }
    writeln!(
        output,
        "solver={:?}; max-axioms={:?}; timeout={:?}; computation={:?}; provenance={:?}",
        proof_hint.solver,
        proof_hint.max_axioms,
        proof_hint.timeout,
        proof_hint.computation,
        proof_hint.provenance
    )
    .expect("write string");
    available
}

fn write_premise_restriction_payload(
    output: &mut String,
    restriction: &PremiseRestriction,
    context: &LocalContext,
    generated_formulas: &BTreeMap<VcGeneratedFormulaId, &VcGeneratedFormula>,
) -> bool {
    match restriction {
        PremiseRestriction::Only(premises) => {
            writeln!(output, "restriction: only").expect("write string");
            let mut available = true;
            for premise in premises {
                available &= write_premise_payload(
                    output,
                    premise,
                    context,
                    generated_formulas,
                    &mut BTreeSet::new(),
                );
            }
            available
        }
        PremiseRestriction::Exclude(premises) => {
            writeln!(output, "restriction: exclude").expect("write string");
            let mut available = true;
            for premise in premises {
                available &= write_premise_payload(
                    output,
                    premise,
                    context,
                    generated_formulas,
                    &mut BTreeSet::new(),
                );
            }
            available
        }
        PremiseRestriction::Intent(intent) => {
            writeln!(output, "restriction: intent {intent:?}").expect("write string");
            true
        }
    }
}

pub(crate) fn stable_fingerprint_hash(domain: &str, bytes: &[u8]) -> Hash {
    let mut lanes = [
        0x6d_69_7a_61_72_2d_76_63_u64,
        0x70_72_6f_6f_66_2d_69_64_u64,
        0x74_61_73_6b_32_30_2d_76_u64,
        0x66_69_6e_67_65_72_2d_31_u64,
    ];

    for (index, byte) in domain
        .as_bytes()
        .iter()
        .copied()
        .chain([0])
        .chain(bytes.iter().copied())
        .enumerate()
    {
        let lane = index % lanes.len();
        let mixed_index = (index as u64).rotate_left((lane as u32) + 1);
        lanes[lane] ^= u64::from(byte)
            .wrapping_add(0x9e37_79b9_7f4a_7c15)
            .wrapping_add(mixed_index);
        lanes[lane] = lanes[lane]
            .rotate_left(11 + lane as u32)
            .wrapping_mul(0x1000_0000_01b3);
    }

    lanes[0] ^= bytes.len() as u64;
    lanes[1] ^= (domain.len() as u64).rotate_left(17);
    lanes[2] ^= lanes[0].rotate_left(7);
    lanes[3] ^= lanes[1].rotate_left(13);

    let mut output = [0_u8; Hash::BYTE_LEN];
    for (chunk, lane) in output.chunks_exact_mut(8).zip(lanes) {
        chunk.copy_from_slice(&lane.to_be_bytes());
    }
    Hash::from_bytes(output)
}

fn render_anchor(anchor: &ObligationAnchor) -> String {
    let completeness = match &anchor.completeness {
        AnchorCompleteness::Complete => "complete".to_owned(),
        AnchorCompleteness::Incomplete { missing } => {
            format!(
                "incomplete missing={:?} cache-miss=true",
                sorted_anchor_missing(missing)
            )
        }
    };
    format!(
        "owner={:?}; kind={:?}; local-path={:?}; label={:?}; semantic-origin={:?}; \
         source-range={:?}; source-shape={:?}; goal-hash={:?}; context-hash={:?}; \
         schema={:?}; {completeness}; provenance={:?}",
        anchor.owner,
        anchor.kind,
        anchor.local_path,
        anchor.label,
        anchor.semantic_origin,
        anchor.source_range,
        anchor.source_shape_hash,
        anchor.canonical_goal_hash,
        anchor.canonical_context_hash,
        anchor.generation_schema_version,
        anchor.provenance
    )
}

fn sorted_anchor_missing(missing: &[AnchorIngredient]) -> Vec<AnchorIngredient> {
    let mut sorted = missing.to_vec();
    sorted.sort();
    sorted
}

#[cfg(test)]
mod tests {
    use super::*;
    use mizar_core::control_flow::{
        ObligationHandoffEntry, ObligationHandoffOrigin, ObligationHandoffTable,
        ObligationSeedHandoff,
    };
    use mizar_core::core_ir::{
        CoreNodeRef, CoreProvenanceKey, CoreProvenancePhase, GeneratedFrom, GeneratedOriginKey,
        GeneratedOriginKind, ObligationSeed, ObligationSeedKind,
    };
    use mizar_session::{InMemorySessionIdAllocator, SessionIdAllocator};

    #[test]
    fn constructs_minimal_vc_set_with_symbolic_refs() {
        let set = fixture_set(VcStatus::NeedsAtp);

        assert_eq!(set.vcs()[0].id, VcId::new(0));
        assert_eq!(
            set.vcs()[0].premises,
            vec![PremiseRef::LocalLabel {
                label: CoreLabelRef::new("A1")
            }]
        );
        assert_eq!(
            set.seed_accounting()[0].mapping,
            SeedVcMapping::One { vc: VcId::new(0) }
        );
    }

    #[test]
    fn needs_atp_and_policy_statuses_preserve_context() {
        let needs_atp = fixture_set(VcStatus::NeedsAtp);
        let policy_open = fixture_set(VcStatus::PolicyOpen {
            policy: PolicyKey::new("interactive-open"),
        });
        let needs_atp_vc = &needs_atp.vcs()[0];
        let policy_vc = &policy_open.vcs()[0];

        assert_eq!(
            needs_atp_vc.local_context.entries(),
            policy_vc.local_context.entries()
        );
        assert_eq!(
            needs_atp_vc.local_context.policy_inputs(),
            policy_vc.local_context.policy_inputs()
        );
        assert_eq!(needs_atp_vc.premises, policy_vc.premises);
        assert_eq!(needs_atp_vc.source, policy_vc.source);
        assert_eq!(needs_atp_vc.anchor, policy_vc.anchor);
        assert_eq!(needs_atp_vc.proof_hint, policy_vc.proof_hint);
        assert_eq!(needs_atp_vc.provenance, policy_vc.provenance);
        assert_eq!(needs_atp.seed_accounting(), policy_open.seed_accounting());
        assert!(matches!(needs_atp_vc.status, VcStatus::NeedsAtp));
        assert!(matches!(policy_vc.status, VcStatus::PolicyOpen { .. }));
    }

    #[test]
    fn status_plan_marks_open_vcs_needs_atp_without_losing_data() {
        let original = fixture_set(VcStatus::Open);
        let plan =
            VcStatusPlan::try_new(VcStatusAction::NeedsAtp, Vec::new()).expect("status plan");

        let projected = original.try_with_status_plan(&plan).expect("projection");
        let original_vc = &original.vcs()[0];
        let projected_vc = &projected.vcs()[0];

        assert!(matches!(original_vc.status, VcStatus::Open));
        assert!(matches!(projected_vc.status, VcStatus::NeedsAtp));
        assert_eq!(projected_vc.id, original_vc.id);
        assert_eq!(projected_vc.kind, original_vc.kind);
        assert_eq!(projected_vc.source, original_vc.source);
        assert_eq!(projected_vc.seed, original_vc.seed);
        assert_eq!(projected_vc.anchor, original_vc.anchor);
        assert_eq!(projected_vc.local_context, original_vc.local_context);
        assert_eq!(projected_vc.premises, original_vc.premises);
        assert_eq!(projected_vc.goal, original_vc.goal);
        assert_eq!(projected_vc.proof_hint, original_vc.proof_hint);
        assert_eq!(
            projected.generated_formulas(),
            original.generated_formulas()
        );
        assert_eq!(projected.seed_accounting(), original.seed_accounting());
        assert_eq!(
            projected_vc.provenance.len(),
            original_vc.provenance.len() + 1
        );
        let provenance = projected_vc.provenance.last().expect("status provenance");
        assert_eq!(provenance.phase, VcProvenancePhase::StatusPolicy);
        assert_eq!(provenance.key.as_str(), "task-9-status:needs-atp");
        assert!(!matches!(projected_vc.status, VcStatus::Discharged { .. }));
        assert!(!projected_vc.provenance.iter().any(|provenance| matches!(
            provenance.phase,
            VcProvenancePhase::Discharge | VcProvenancePhase::DependencySlice
        )));
    }

    #[test]
    fn status_plan_policy_overrides_preserve_context_and_seed_accounting() {
        let original = fixture_set(VcStatus::Open);
        let policy_open_plan = VcStatusPlan::try_new(
            VcStatusAction::NeedsAtp,
            vec![VcStatusOverride {
                vc: VcId::new(0),
                action: VcStatusAction::PolicyOpen {
                    policy: PolicyKey::new("interactive-open"),
                },
            }],
        )
        .expect("policy-open plan");

        let policy_open = original
            .try_with_status_plan(&policy_open_plan)
            .expect("policy-open projection");

        assert!(matches!(
            &policy_open.vcs()[0].status,
            VcStatus::PolicyOpen { policy } if policy == &PolicyKey::new("interactive-open")
        ));
        assert_eq!(
            policy_open.vcs()[0].local_context,
            original.vcs()[0].local_context
        );
        assert_eq!(
            policy_open.vcs()[0].proof_hint,
            original.vcs()[0].proof_hint
        );
        assert_eq!(policy_open.seed_accounting(), original.seed_accounting());
        let policy_provenance = policy_open.vcs()[0]
            .provenance
            .last()
            .expect("policy-open provenance");
        assert_eq!(policy_provenance.phase, VcProvenancePhase::StatusPolicy);
        assert_eq!(
            policy_provenance.key.as_str(),
            "task-9-status:policy-open:interactive-open"
        );

        let assumed_plan = VcStatusPlan::try_new(
            VcStatusAction::Preserve,
            vec![VcStatusOverride {
                vc: VcId::new(0),
                action: VcStatusAction::AssumeByPolicy {
                    policy: PolicyKey::new("assume-labeled-premise"),
                    marker: PremiseRef::LocalContext(ContextEntryId::new(0)),
                },
            }],
        )
        .expect("assumed plan");
        let assumed = original
            .try_with_status_plan(&assumed_plan)
            .expect("assumed projection");

        assert!(matches!(
            &assumed.vcs()[0].status,
            VcStatus::AssumedByPolicy { policy, marker }
                if policy == &PolicyKey::new("assume-labeled-premise")
                    && marker == &PremiseRef::LocalContext(ContextEntryId::new(0))
        ));
        assert_eq!(
            assumed.vcs()[0].local_context,
            original.vcs()[0].local_context
        );
        assert_eq!(assumed.vcs()[0].premises, original.vcs()[0].premises);
        assert_eq!(assumed.seed_accounting(), original.seed_accounting());
        let assumed_provenance = assumed.vcs()[0]
            .provenance
            .last()
            .expect("assumed provenance");
        assert_eq!(assumed_provenance.phase, VcProvenancePhase::StatusPolicy);
        assert_eq!(
            assumed_provenance.key.as_str(),
            "task-9-status:assumed-by-policy:assume-labeled-premise"
        );
    }

    #[test]
    fn status_plan_applies_defaults_and_sorted_overrides_across_multiple_vcs() {
        let original = two_vc_fixture_set(VcStatus::Open, VcStatus::Open);
        let plan = VcStatusPlan::try_new(
            VcStatusAction::NeedsAtp,
            vec![VcStatusOverride {
                vc: VcId::new(1),
                action: VcStatusAction::PolicyOpen {
                    policy: PolicyKey::new("second-vc-open"),
                },
            }],
        )
        .expect("multi-vc status plan");

        let projected = original.try_with_status_plan(&plan).expect("projection");

        assert_eq!(
            projected.vcs().iter().map(|vc| vc.id).collect::<Vec<_>>(),
            vec![VcId::new(0), VcId::new(1)]
        );
        assert!(matches!(projected.vcs()[0].status, VcStatus::NeedsAtp));
        assert!(matches!(
            &projected.vcs()[1].status,
            VcStatus::PolicyOpen { policy } if policy == &PolicyKey::new("second-vc-open")
        ));
        assert_eq!(
            projected.generated_formulas(),
            original.generated_formulas()
        );
        assert_eq!(projected.seed_accounting(), original.seed_accounting());
        assert_eq!(
            projected.vcs()[0].local_context,
            original.vcs()[0].local_context
        );
        assert_eq!(
            projected.vcs()[1].local_context,
            original.vcs()[1].local_context
        );
        assert_eq!(
            projected.vcs()[0]
                .provenance
                .last()
                .expect("default provenance")
                .key
                .as_str(),
            "task-9-status:needs-atp"
        );
        assert_eq!(
            projected.vcs()[1]
                .provenance
                .last()
                .expect("override provenance")
                .key
                .as_str(),
            "task-9-status:policy-open:second-vc-open"
        );
    }

    #[test]
    fn status_plan_preserve_noop_does_not_add_status_provenance() {
        let original = fixture_set(VcStatus::NeedsAtp);
        let plan =
            VcStatusPlan::try_new(VcStatusAction::Preserve, Vec::new()).expect("preserve plan");

        let projected = original.try_with_status_plan(&plan).expect("projection");

        assert_eq!(projected, original);
    }

    #[test]
    fn status_plan_rejects_duplicate_unsorted_and_missing_overrides() {
        assert!(matches!(
            VcStatusPlan::try_new(
                VcStatusAction::Preserve,
                vec![
                    VcStatusOverride {
                        vc: VcId::new(1),
                        action: VcStatusAction::NeedsAtp,
                    },
                    VcStatusOverride {
                        vc: VcId::new(0),
                        action: VcStatusAction::NeedsAtp,
                    },
                ],
            ),
            Err(VcIrError::StatusOverridesNotSorted {
                previous,
                current
            }) if previous == VcId::new(1) && current == VcId::new(0)
        ));

        assert!(matches!(
            VcStatusPlan::try_new(
                VcStatusAction::Preserve,
                vec![
                    VcStatusOverride {
                        vc: VcId::new(0),
                        action: VcStatusAction::NeedsAtp,
                    },
                    VcStatusOverride {
                        vc: VcId::new(0),
                        action: VcStatusAction::PolicyOpen {
                            policy: PolicyKey::new("duplicate")
                        },
                    },
                ],
            ),
            Err(VcIrError::DuplicateStatusOverride { vc }) if vc == VcId::new(0)
        ));

        let original = fixture_set(VcStatus::Open);
        let plan = VcStatusPlan::try_new(
            VcStatusAction::Preserve,
            vec![VcStatusOverride {
                vc: VcId::new(1),
                action: VcStatusAction::NeedsAtp,
            }],
        )
        .expect("missing target plan");

        assert!(matches!(
            original.try_with_status_plan(&plan),
            Err(VcIrError::MissingStatusOverrideTarget { vc }) if vc == VcId::new(1)
        ));
    }

    #[test]
    fn status_plan_invalid_assumption_marker_fails_closed() {
        let original = fixture_set(VcStatus::Open);
        let plan = VcStatusPlan::try_new(
            VcStatusAction::Preserve,
            vec![VcStatusOverride {
                vc: VcId::new(0),
                action: VcStatusAction::AssumeByPolicy {
                    policy: PolicyKey::new("invalid-marker"),
                    marker: PremiseRef::LocalContext(ContextEntryId::new(99)),
                },
            }],
        )
        .expect("invalid marker plan");

        assert!(matches!(
            original.try_with_status_plan(&plan),
            Err(VcIrError::MissingContextEntry { context })
                if context == ContextEntryId::new(99)
        ));

        let generated_formula_plan = VcStatusPlan::try_new(
            VcStatusAction::Preserve,
            vec![VcStatusOverride {
                vc: VcId::new(0),
                action: VcStatusAction::AssumeByPolicy {
                    policy: PolicyKey::new("invalid-generated-marker"),
                    marker: PremiseRef::GeneratedFact {
                        formula: VcFormulaRef::Generated(VcGeneratedFormulaId::new(99)),
                    },
                },
            }],
        )
        .expect("invalid generated marker plan");

        assert!(matches!(
            original.try_with_status_plan(&generated_formula_plan),
            Err(VcIrError::MissingGeneratedFormula { formula })
                if formula == VcGeneratedFormulaId::new(99)
        ));
    }

    #[test]
    fn generated_formula_table_owns_split_goal_shape() {
        let set = fixture_set(VcStatus::NeedsAtp);

        assert_eq!(set.generated_formulas().len(), 1);
        assert_eq!(
            set.vcs()[0].goal,
            VcFormulaRef::Generated(VcGeneratedFormulaId::new(0))
        );
        assert!(matches!(
            set.generated_formula(VcGeneratedFormulaId::new(0))
                .expect("generated formula")
                .shape,
            VcGeneratedFormulaShape::And(_)
        ));
    }

    #[test]
    fn seed_intake_preserves_handoff_order_and_debug_rendering() {
        let source = sample_source_id();
        let handoff = seed_handoff(vec![
            (
                obligation_seed(
                    source,
                    ObligationSeedKind::TheoremProof,
                    ObligationSeedStatus::Active,
                    Some(CoreFormulaId::new(0)),
                    "proof/step/0",
                    Vec::new(),
                ),
                ObligationHandoffOrigin::ExistingCore {
                    seed: ObligationSeedId::new(0),
                },
            ),
            (
                obligation_seed(
                    source,
                    ObligationSeedKind::DefinitionCorrectness,
                    ObligationSeedStatus::Deferred,
                    None,
                    "definition/deferred",
                    Vec::new(),
                ),
                ObligationHandoffOrigin::ExistingCore {
                    seed: ObligationSeedId::new(1),
                },
            ),
        ]);

        let first = SeedIntakeTable::try_from_handoff(&handoff).expect("seed intake");
        let second = SeedIntakeTable::try_from_handoff(&handoff).expect("seed intake");

        assert_eq!(first.rows().len(), 2);
        assert_eq!(first.rows()[0].handoff, ObligationHandoffId::new(0));
        assert_eq!(first.rows()[1].handoff, ObligationHandoffId::new(1));
        assert!(matches!(
            first.rows()[0].mapping,
            SeedIntakeMapping::EligibleOneVc { goal }
                if goal == CoreFormulaId::new(0)
        ));
        assert!(matches!(
            first.rows()[1].mapping,
            SeedIntakeMapping::NoConcreteVc {
                reason: SeedNoVcReason::DeferredExternal(_)
            }
        ));
        assert_eq!(first.debug_text(), second.debug_text());
        assert!(first.debug_text().contains("seed-intake-table-debug-v1"));
        assert!(
            first
                .debug_text()
                .contains("handoff ObligationHandoffId(0):")
        );
    }

    #[test]
    fn seed_intake_records_visible_no_vc_reasons() {
        let source = sample_source_id();
        let handoff = seed_handoff(vec![
            (
                obligation_seed(
                    source,
                    ObligationSeedKind::CheckerInitial,
                    ObligationSeedStatus::Skipped,
                    None,
                    "checker/skipped",
                    Vec::new(),
                ),
                ObligationHandoffOrigin::ExistingCore {
                    seed: ObligationSeedId::new(0),
                },
            ),
            (
                obligation_seed(
                    source,
                    ObligationSeedKind::AlgorithmTermination,
                    ObligationSeedStatus::Active,
                    None,
                    "algorithm/missing-goal",
                    Vec::new(),
                ),
                ObligationHandoffOrigin::ExistingCore {
                    seed: ObligationSeedId::new(1),
                },
            ),
            (
                obligation_seed(
                    source,
                    ObligationSeedKind::GeneratedSethood,
                    ObligationSeedStatus::Error,
                    None,
                    "generated/error",
                    vec![CoreDiagnosticId::new(3)],
                ),
                ObligationHandoffOrigin::ExistingCore {
                    seed: ObligationSeedId::new(2),
                },
            ),
        ]);

        let table = SeedIntakeTable::try_from_handoff(&handoff).expect("seed intake");

        assert!(matches!(
            table.rows()[0].mapping,
            SeedIntakeMapping::NoConcreteVc {
                reason: SeedNoVcReason::SkippedInvalidInput
            }
        ));
        assert!(matches!(
            table.rows()[1].mapping,
            SeedIntakeMapping::NoConcreteVc {
                reason: SeedNoVcReason::MissingGoal(_)
            }
        ));
        assert!(matches!(
            table.rows()[2].mapping,
            SeedIntakeMapping::NoConcreteVc {
                reason: SeedNoVcReason::Error(diagnostic)
            } if diagnostic == CoreDiagnosticId::new(3)
        ));
    }

    #[test]
    fn seed_intake_rejects_duplicate_seed_origin_and_missing_source() {
        let source = sample_source_id();
        let seed = obligation_seed(
            source,
            ObligationSeedKind::TheoremProof,
            ObligationSeedStatus::Active,
            Some(CoreFormulaId::new(0)),
            "proof/duplicate",
            Vec::new(),
        );
        let duplicate = seed_handoff(vec![
            (
                seed.clone(),
                ObligationHandoffOrigin::ExistingCore {
                    seed: ObligationSeedId::new(0),
                },
            ),
            (
                seed,
                ObligationHandoffOrigin::ExistingCore {
                    seed: ObligationSeedId::new(0),
                },
            ),
        ]);

        assert!(matches!(
            SeedIntakeTable::try_from_handoff(&duplicate),
            Err(VcIrError::DuplicateSeedIntake {
                first_handoff,
                second_handoff
            }) if first_handoff == ObligationHandoffId::new(0)
                && second_handoff == ObligationHandoffId::new(1)
        ));

        let mut missing_source = seed_handoff(vec![(
            obligation_seed(
                source,
                ObligationSeedKind::TheoremProof,
                ObligationSeedStatus::Active,
                Some(CoreFormulaId::new(0)),
                "proof/source-missing",
                Vec::new(),
            ),
            ObligationHandoffOrigin::ExistingCore {
                seed: ObligationSeedId::new(0),
            },
        )]);
        missing_source.source_map.clear();

        assert!(matches!(
            SeedIntakeTable::try_from_handoff(&missing_source),
            Err(VcIrError::MissingHandoffSource { handoff })
                if handoff == ObligationHandoffId::new(0)
        ));
    }

    #[test]
    fn seed_intake_preserves_distinct_origins_for_same_canonical_seed() {
        let source = sample_source_id();
        let seed = obligation_seed(
            source,
            ObligationSeedKind::AlgorithmContract,
            ObligationSeedStatus::Active,
            Some(CoreFormulaId::new(0)),
            "algorithm/requires/0",
            Vec::new(),
        );
        let handoff = seed_handoff(vec![
            (
                seed.clone(),
                ObligationHandoffOrigin::ExistingCore {
                    seed: ObligationSeedId::new(0),
                },
            ),
            (
                seed,
                ObligationHandoffOrigin::FlowDerived {
                    flow: ControlFlowId::new(0),
                    algorithm: CoreAlgorithmId::new(2),
                },
            ),
        ]);

        let table = SeedIntakeTable::try_from_handoff(&handoff)
            .expect("same canonical key with distinct origins is allowed");

        assert_eq!(table.rows().len(), 2);
        assert!(matches!(
            table.rows()[0].origin,
            SeedOriginRef::ExistingCore { seed }
                if seed == ObligationSeedId::new(0)
        ));
        assert!(matches!(
            table.rows()[1].origin,
            SeedOriginRef::FlowDerived {
                flow,
                algorithm,
            } if flow == ControlFlowId::new(0)
                && algorithm == CoreAlgorithmId::new(2)
        ));
    }

    #[test]
    fn validation_rejects_duplicate_vc_ids_and_missing_mapping() {
        let mut parts = fixture_parts(VcStatus::NeedsAtp);
        parts.vcs.push(parts.vcs[0].clone());

        assert!(matches!(
            VcSet::try_new(parts),
            Err(VcIrError::NonDenseVcId {
                expected,
                actual
            }) if expected == VcId::new(1) && actual == VcId::new(0)
        ));

        let mut parts = fixture_parts(VcStatus::NeedsAtp);
        parts.seed_accounting[0].mapping = SeedVcMapping::NoConcreteVc {
            reason: SeedNoVcReason::DeferredExternal(VcText::new("runner unavailable")),
        };

        assert!(matches!(
            VcSet::try_new(parts),
            Err(VcIrError::VcMissingFromSeedMapping { vc, .. }) if vc == VcId::new(0)
        ));
    }

    #[test]
    fn validation_rejects_missing_generated_goal() {
        let mut parts = fixture_parts(VcStatus::NeedsAtp);
        parts.vcs[0].goal = VcFormulaRef::Generated(VcGeneratedFormulaId::new(1));

        assert!(matches!(
            VcSet::try_new(parts),
            Err(VcIrError::MissingGeneratedFormula { formula })
                if formula == VcGeneratedFormulaId::new(1)
        ));
    }

    #[test]
    fn validation_rejects_nested_missing_formula_and_context_refs() {
        let mut parts = fixture_parts(VcStatus::NeedsAtp);
        parts.vcs[0].local_context = LocalContext::try_new(
            vec![ContextEntry {
                id: ContextEntryId::new(0),
                sort_key: CanonicalSortKey::new("000-generated-context"),
                kind: ContextEntryKind::ProofAssumption,
                formula: Some(VcFormulaRef::Generated(VcGeneratedFormulaId::new(1))),
                provenance: Vec::new(),
            }],
            Vec::new(),
        )
        .expect("context shape is locally valid");

        assert!(matches!(
            VcSet::try_new(parts),
            Err(VcIrError::MissingGeneratedFormula { formula })
                if formula == VcGeneratedFormulaId::new(1)
        ));

        let mut parts = fixture_parts(VcStatus::NeedsAtp);
        parts.vcs[0].premises = vec![PremiseRef::LocalContext(ContextEntryId::new(7))];

        assert!(matches!(
            VcSet::try_new(parts),
            Err(VcIrError::MissingContextEntry { context })
                if context == ContextEntryId::new(7)
        ));

        let parts = fixture_parts(VcStatus::AssumedByPolicy {
            policy: PolicyKey::new("assume-for-test"),
            marker: PremiseRef::LocalContext(ContextEntryId::new(9)),
        });

        assert!(matches!(
            VcSet::try_new(parts),
            Err(VcIrError::MissingContextEntry { context })
                if context == ContextEntryId::new(9)
        ));

        let mut parts = fixture_parts(VcStatus::NeedsAtp);
        parts.generated_formulas[0].shape = VcGeneratedFormulaShape::Quantified {
            kind: QuantifierKind::Forall,
            binders: vec![ContextEntryId::new(99)],
            body: VcFormulaRef::Core(CoreFormulaId::new(0)),
        };

        assert!(matches!(
            VcSet::try_new(parts),
            Err(VcIrError::MissingContextEntry { context })
                if context == ContextEntryId::new(99)
        ));
    }

    #[test]
    fn validation_rejects_unsorted_context_entries() {
        let error = LocalContext::try_new(
            vec![
                ContextEntry {
                    id: ContextEntryId::new(0),
                    sort_key: CanonicalSortKey::new("001-later"),
                    kind: ContextEntryKind::ProofAssumption,
                    formula: Some(VcFormulaRef::Core(CoreFormulaId::new(0))),
                    provenance: Vec::new(),
                },
                ContextEntry {
                    id: ContextEntryId::new(1),
                    sort_key: CanonicalSortKey::new("000-earlier"),
                    kind: ContextEntryKind::CurrentThesis,
                    formula: Some(VcFormulaRef::Core(CoreFormulaId::new(1))),
                    provenance: Vec::new(),
                },
            ],
            Vec::new(),
        )
        .expect_err("unsorted context");

        assert!(matches!(error, VcIrError::ContextEntriesNotSorted { .. }));
    }

    #[test]
    fn expanded_seed_mapping_requires_sorted_existing_targets() {
        let mut parts = fixture_parts(VcStatus::NeedsAtp);
        parts.seed_accounting[0].mapping = SeedVcMapping::Expanded {
            vcs: vec![
                ExpandedVcRef {
                    expansion_index: 1,
                    vc: VcId::new(0),
                },
                ExpandedVcRef {
                    expansion_index: 0,
                    vc: VcId::new(0),
                },
            ],
            expansion_schema: ExpansionSchemaVersion::new("split-v1"),
        };

        assert!(matches!(
            VcSet::try_new(parts),
            Err(VcIrError::ExpandedMappingIndexGap {
                expected: 0,
                actual: 1,
                ..
            })
        ));

        let mut parts = fixture_parts(VcStatus::NeedsAtp);
        parts.seed_accounting[0].mapping = SeedVcMapping::Expanded {
            vcs: vec![
                ExpandedVcRef {
                    expansion_index: 0,
                    vc: VcId::new(0),
                },
                ExpandedVcRef {
                    expansion_index: 0,
                    vc: VcId::new(0),
                },
            ],
            expansion_schema: ExpansionSchemaVersion::new("split-v1"),
        };

        assert!(matches!(
            VcSet::try_new(parts),
            Err(VcIrError::ExpandedMappingNotSorted { .. })
        ));
    }

    #[test]
    fn validation_rejects_duplicate_and_unsorted_seed_accounting() {
        let mut parts = fixture_parts(VcStatus::NeedsAtp);
        parts.seed_accounting.push(parts.seed_accounting[0].clone());

        assert!(matches!(
            VcSet::try_new(parts),
            Err(VcIrError::DuplicateSeedAccounting { handoff })
                if handoff == ObligationHandoffId::new(0)
        ));

        let mut parts = fixture_parts(VcStatus::NeedsAtp);
        parts.seed_accounting.insert(
            0,
            SeedAccounting {
                handoff: ObligationHandoffId::new(1),
                origin: SeedOriginRef::ExistingCore {
                    seed: ObligationSeedId::new(1),
                },
                seed_status: ObligationSeedStatus::Skipped,
                mapping: SeedVcMapping::NoConcreteVc {
                    reason: SeedNoVcReason::SkippedInvalidInput,
                },
            },
        );

        assert!(matches!(
            VcSet::try_new(parts),
            Err(VcIrError::SeedAccountingNotSorted {
                previous,
                current
            }) if previous == ObligationHandoffId::new(1)
                && current == ObligationHandoffId::new(0)
        ));
    }

    #[test]
    fn validation_rejects_wrong_seed_and_duplicate_vc_mapping() {
        let mut parts = fixture_parts(VcStatus::NeedsAtp);
        parts.seed_accounting.push(SeedAccounting {
            handoff: ObligationHandoffId::new(1),
            origin: SeedOriginRef::ExistingCore {
                seed: ObligationSeedId::new(1),
            },
            seed_status: ObligationSeedStatus::Active,
            mapping: SeedVcMapping::One { vc: VcId::new(0) },
        });

        assert!(matches!(
            VcSet::try_new(parts),
            Err(VcIrError::VcMappedFromWrongSeed {
                vc,
                expected_handoff,
                actual_handoff
            }) if vc == VcId::new(0)
                && expected_handoff == ObligationHandoffId::new(0)
                && actual_handoff == ObligationHandoffId::new(1)
        ));

        let mut parts = fixture_parts(VcStatus::NeedsAtp);
        parts.seed_accounting[0].mapping = SeedVcMapping::Expanded {
            vcs: vec![
                ExpandedVcRef {
                    expansion_index: 0,
                    vc: VcId::new(0),
                },
                ExpandedVcRef {
                    expansion_index: 1,
                    vc: VcId::new(0),
                },
            ],
            expansion_schema: ExpansionSchemaVersion::new("split-v1"),
        };

        assert!(matches!(
            VcSet::try_new(parts),
            Err(VcIrError::DuplicateVcMapping {
                vc,
                first_handoff,
                second_handoff
            }) if vc == VcId::new(0)
                && first_handoff == ObligationHandoffId::new(0)
                && second_handoff == ObligationHandoffId::new(0)
        ));
    }

    #[test]
    fn rendering_is_byte_identical_and_marks_incomplete_anchor() {
        let first = fixture_set(VcStatus::NeedsAtp);
        let second = fixture_set(VcStatus::NeedsAtp);
        let rendering = first.debug_text();

        assert_eq!(rendering, second.debug_text());
        assert!(rendering.contains("schema-version: VcSchemaVersion(\"vc-ir-test-v1\")"));
        assert!(rendering.contains("[generated-formulas]"));
        assert!(rendering.contains("[seed-accounting]"));
        assert!(rendering.contains("seed ObligationHandoffId(0):"));
        assert!(rendering.contains("[vcs]"));
        assert!(rendering.contains("vc VcId(0): kind=TheoremProofStep"));
        assert!(rendering.contains("source: VcSourceRef"));
        assert!(rendering.contains("context: [ContextEntry"));
        assert!(rendering.contains("policy-inputs: [VerifierPolicyInput"));
        assert!(rendering.contains("premises: [LocalLabel"));
        assert!(rendering.contains("proof-hint: Some(ProofHint"));
        assert!(rendering.contains("status: NeedsAtp"));
        assert!(rendering.contains("provenance: [VcProvenance"));
        assert!(rendering.contains("incomplete missing="));
        assert!(rendering.contains("cache-miss=true"));
        assert!(!rendering.contains("backend"));
    }

    #[test]
    fn complete_anchor_rejects_unavailable_hash_markers() {
        let mut parts = fixture_parts(VcStatus::NeedsAtp);
        parts.vcs[0].anchor.completeness = AnchorCompleteness::Complete;

        assert!(matches!(
            VcSet::try_new(parts),
            Err(VcIrError::CompleteAnchorMissingIngredient {
                ingredient: AnchorIngredient::SourceShapeHash
            })
        ));
    }

    #[test]
    fn anchor_validation_rejects_unsorted_missing_and_missing_provenance() {
        let mut parts = fixture_parts(VcStatus::NeedsAtp);
        parts.vcs[0].anchor.completeness = AnchorCompleteness::Incomplete {
            missing: vec![
                AnchorIngredient::CanonicalGoalHash,
                AnchorIngredient::SourceShapeHash,
            ],
        };

        assert!(matches!(
            VcSet::try_new(parts),
            Err(VcIrError::AnchorMissingIngredientsNotSorted { .. })
        ));

        let mut parts = fixture_parts(VcStatus::NeedsAtp);
        parts.vcs[0].anchor.completeness = AnchorCompleteness::Incomplete {
            missing: vec![
                AnchorIngredient::SourceShapeHash,
                AnchorIngredient::SourceShapeHash,
            ],
        };

        assert!(matches!(
            VcSet::try_new(parts),
            Err(VcIrError::DuplicateAnchorMissingIngredient {
                ingredient: AnchorIngredient::SourceShapeHash
            })
        ));

        let mut parts = fixture_parts(VcStatus::NeedsAtp);
        parts.vcs[0].anchor.completeness = AnchorCompleteness::Incomplete {
            missing: vec![
                AnchorIngredient::SourceShapeHash,
                AnchorIngredient::CanonicalContextHash,
            ],
        };

        assert!(matches!(
            VcSet::try_new(parts),
            Err(VcIrError::IncompleteAnchorOmitsMissingIngredient {
                ingredient: AnchorIngredient::CanonicalGoalHash
            })
        ));

        let mut parts = fixture_parts(VcStatus::NeedsAtp);
        parts.vcs[0].anchor.source_shape_hash = available_hash(1);
        parts.vcs[0].anchor.completeness = AnchorCompleteness::Incomplete {
            missing: vec![
                AnchorIngredient::SourceShapeHash,
                AnchorIngredient::CanonicalGoalHash,
                AnchorIngredient::CanonicalContextHash,
            ],
        };

        assert!(matches!(
            VcSet::try_new(parts),
            Err(VcIrError::IncompleteAnchorClaimsAvailableIngredient {
                ingredient: AnchorIngredient::SourceShapeHash
            })
        ));

        let mut parts = fixture_parts(VcStatus::NeedsAtp);
        parts.vcs[0].anchor.source_shape_hash = available_hash(1);
        parts.vcs[0].anchor.canonical_goal_hash = available_hash(2);
        parts.vcs[0].anchor.canonical_context_hash = available_hash(3);
        parts.vcs[0].anchor.provenance.clear();
        parts.vcs[0].anchor.completeness = AnchorCompleteness::Complete;

        assert!(matches!(
            VcSet::try_new(parts),
            Err(VcIrError::CompleteAnchorMissingIngredient {
                ingredient: AnchorIngredient::SourceProvenance
            })
        ));
    }

    fn fixture_set(status: VcStatus) -> VcSet {
        VcSet::try_new(fixture_parts(status)).expect("valid fixture")
    }

    fn two_vc_fixture_set(first_status: VcStatus, second_status: VcStatus) -> VcSet {
        let mut parts = fixture_parts(first_status);
        let mut second_vc = parts.vcs[0].clone();
        second_vc.id = VcId::new(1);
        second_vc.seed = SeedVcRef {
            handoff: ObligationHandoffId::new(1),
        };
        second_vc.status = second_status;
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

        VcSet::try_new(parts).expect("valid two-vc fixture")
    }

    fn fixture_parts(status: VcStatus) -> VcSetParts {
        let snapshot = BuildSnapshotId::from_published_schema_str(
            "mizar-session-build-snapshot-v1:\
             1111111111111111111111111111111111111111111111111111111111111111",
        )
        .expect("snapshot id");
        let source = InMemorySessionIdAllocator::new()
            .next_source_id(snapshot)
            .expect("source id");
        let source_ref = source_ref(source);
        let context = LocalContext::try_new(
            vec![ContextEntry {
                id: ContextEntryId::new(0),
                sort_key: CanonicalSortKey::new("000-assumption"),
                kind: ContextEntryKind::ProofAssumption,
                formula: Some(VcFormulaRef::Core(CoreFormulaId::new(0))),
                provenance: vec![provenance("context")],
            }],
            vec![VerifierPolicyInput {
                sort_key: CanonicalSortKey::new("000-policy"),
                key: PolicyKey::new("atp-required"),
                value: PolicyValue::new("true"),
            }],
        )
        .expect("local context");
        let handoff = ObligationHandoffId::new(0);

        VcSetParts {
            schema_version: VcSchemaVersion::new("vc-ir-test-v1"),
            snapshot,
            source,
            module: VcModuleRef::new("sample"),
            generated_formulas: vec![VcGeneratedFormula {
                id: VcGeneratedFormulaId::new(0),
                kind: VcGeneratedFormulaKind::SplitGoal,
                shape: VcGeneratedFormulaShape::And(vec![
                    VcFormulaRef::Core(CoreFormulaId::new(0)),
                    VcFormulaRef::Core(CoreFormulaId::new(1)),
                ]),
                provenance: vec![provenance("generated-goal")],
            }],
            vcs: vec![VcIr {
                id: VcId::new(0),
                kind: VcKind::TheoremProofStep,
                source: VcSourceRef {
                    primary: source_ref.clone(),
                    related: vec![generated_source_ref()],
                },
                seed: SeedVcRef { handoff },
                anchor: incomplete_anchor(source, source_ref),
                local_context: context,
                premises: vec![PremiseRef::LocalLabel {
                    label: CoreLabelRef::new("A1"),
                }],
                goal: VcFormulaRef::Generated(VcGeneratedFormulaId::new(0)),
                proof_hint: Some(ProofHint {
                    citations: vec![PremiseRef::LocalLabel {
                        label: CoreLabelRef::new("A1"),
                    }],
                    unfold_requests: vec![DefinitionUnfoldRequest {
                        definition: CoreDefinitionId::new(0),
                        opacity_override: Some(DefinitionOpacityOverride::Transparent),
                    }],
                    premise_restrictions: vec![PremiseRestriction::Only(vec![
                        PremiseRef::LocalContext(ContextEntryId::new(0)),
                    ])],
                    solver: Some(ProofHintKey::new("symbolic-solver-intent")),
                    max_axioms: Some(32),
                    timeout: Some(ProofHintKey::new("policy-timeout-short")),
                    computation: Some(ComputationHint::ByComputation),
                    provenance: vec![provenance("hint")],
                }),
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
        source: SourceId,
        kind: ObligationSeedKind,
        status: ObligationSeedStatus,
        goal: Option<CoreFormulaId>,
        local_path: &str,
        diagnostics: Vec<CoreDiagnosticId>,
    ) -> ObligationSeed {
        ObligationSeed {
            owner: CoreItemId::new(0),
            kind,
            goal,
            context: Vec::new(),
            local_path: LocalProofOrProgramPath::new(local_path),
            label: None,
            semantic_origin: NormalizedSemanticOrigin::new(local_path),
            provenance: Vec::new(),
            source: source_ref(source),
            core_refs: goal.map(CoreNodeRef::Formula).into_iter().collect(),
            status,
            diagnostics,
        }
    }

    fn incomplete_anchor(source: SourceId, source_ref: CoreSourceRef) -> ObligationAnchor {
        ObligationAnchor {
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
                reason: AnchorUnavailableReason::new("task-3 fixture lacks source-shape hash"),
            },
            canonical_goal_hash: HashMarker::Unavailable {
                reason: AnchorUnavailableReason::new("task-3 fixture lacks canonical goal hash"),
            },
            canonical_context_hash: HashMarker::Unavailable {
                reason: AnchorUnavailableReason::new("task-3 fixture lacks context hash"),
            },
            generation_schema_version: GenerationSchemaVersion::new("task-3-test"),
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
        fn with_extra_provenance(self, source_ref: CoreSourceRef) -> Self;
    }

    impl TestAnchorExt for ObligationAnchor {
        fn with_extra_provenance(mut self, source_ref: CoreSourceRef) -> Self {
            self.provenance.push(VcProvenance {
                phase: VcProvenancePhase::CoreHandoff,
                key: VcText::new("source-ref"),
                core: source_ref.provenance.first().cloned(),
            });
            self
        }
    }

    fn sample_source_id() -> SourceId {
        let snapshot = BuildSnapshotId::from_published_schema_str(
            "mizar-session-build-snapshot-v1:\
             1111111111111111111111111111111111111111111111111111111111111111",
        )
        .expect("snapshot id");
        InMemorySessionIdAllocator::new()
            .next_source_id(snapshot)
            .expect("source id")
    }

    fn source_ref(source: SourceId) -> CoreSourceRef {
        CoreSourceRef::direct(SourceRange {
            source_id: source,
            start: 0,
            end: 10,
        })
        .with_provenance(vec![CoreProvenance::new(
            CoreProvenancePhase::ProofSkeleton,
            "direct-source",
        )])
    }

    fn generated_source_ref() -> CoreSourceRef {
        CoreSourceRef::generated(GeneratedFrom {
            owner: CoreNodeRef::Item(CoreItemId::new(0)),
            kind: GeneratedOriginKind::TypePredicate,
            key: GeneratedOriginKey::new("generated-type"),
            reason: CoreProvenanceKey::new("task-3-test"),
        })
    }

    fn provenance(key: &str) -> VcProvenance {
        VcProvenance {
            phase: VcProvenancePhase::Generator,
            key: VcText::new(key),
            core: None,
        }
    }

    fn available_hash(byte: u8) -> HashMarker {
        HashMarker::Available(Hash::from_bytes([byte; Hash::BYTE_LEN]))
    }
}
