//! Prover-independent verification-condition IR data shapes.
//!
//! This module implements the data-layer contract specified in
//! [vc_ir.md](../../../doc/design/mizar-vc/en/vc_ir.md).

use mizar_core::{
    control_flow::{ControlFlowId, ObligationHandoffId},
    core_ir::{
        CoreAlgorithmId, CoreDefinitionId, CoreDiagnosticId, CoreFormulaId, CoreItemId,
        CoreLabelRef, CoreProvenance, CoreSourceRef, CoreVarId, LocalProofOrProgramPath,
        NormalizedSemanticOrigin, ObligationSeedId, ObligationSeedStatus,
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
#[non_exhaustive]
pub enum SeedOriginRef {
    ExistingCore {
        seed: ObligationSeedId,
    },
    FlowDerived {
        flow: ControlFlowId,
        algorithm: CoreAlgorithmId,
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
    PolicyDisabled(PolicyKey),
    Error(CoreDiagnosticId),
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
    DuplicateSeedAccounting {
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
            Self::DuplicateSeedAccounting { handoff } => {
                write!(formatter, "duplicate seed accounting row for {handoff:?}")
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
    use mizar_core::core_ir::{
        CoreNodeRef, CoreProvenanceKey, CoreProvenancePhase, GeneratedFrom, GeneratedOriginKey,
        GeneratedOriginKind,
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
