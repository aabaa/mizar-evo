//! Deterministic VC-to-ATP problem translation.
//!
//! This module implements the task-5/task-6 slices specified in
//! [translator.md](../../../doc/design/mizar-atp/en/translator.md). It lowers
//! structured VC projections into backend-neutral ATP problems and does not
//! run proof search, backends, SAT solvers, or kernel acceptance checks.

use crate::problem::{
    AtpDeclaration, AtpDeclarationId, AtpDeclarationKind, AtpDiagnostic, AtpFingerprint,
    AtpFormula, AtpFormulaId, AtpFormulaTree, AtpPayload, AtpProblem, AtpProblemError,
    AtpProblemParts, AtpProvenance, AtpProvenanceId, AtpRequiredProofStatus, AtpSourceBinding,
    AtpSourceRef, AtpSymbolMapEntry, AtpSymbolName, AtpSymbolSource, AtpTargetBinding, AtpTerm,
    AtpTypeContext, AtpTypeGuard, AtpTypeGuardId, EqualitySupport, ExpectedBackendResult,
    LogicProfile, QuantifierPolicy, SoftTypeStrategy,
};
use mizar_vc::{
    kernel_evidence_handoff::{
        KernelEvidenceHandoffError, KernelFormulaEvidenceEntry, KernelFormulaSource,
        KernelGoalPolarity, KernelImportedFactRequirement, KernelRequiredProofStatus,
        VcKernelEvidenceHandoff,
    },
    vc_ir::{
        ContextEntryId, ContextEntryKind, PremiseRef, VcFormulaRef, VcGeneratedFormulaId, VcId,
        VcSet, VcStatus, VcText,
    },
};
use std::{
    collections::{BTreeMap, BTreeSet},
    error::Error,
    fmt,
};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AtpProjectionKey(String);

impl AtpProjectionKey {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn is_empty(&self) -> bool {
        self.0.trim().is_empty()
    }
}

impl From<&str> for AtpProjectionKey {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl From<String> for AtpProjectionKey {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AtpProjectionProvenance {
    pub source: AtpSourceRef,
    pub payload: AtpPayload,
}

impl AtpProjectionProvenance {
    pub fn new(source: AtpSourceRef, payload: impl Into<AtpPayload>) -> Self {
        Self {
            source,
            payload: payload.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AtpDeclarationProjection {
    pub key: AtpProjectionKey,
    pub kind: AtpDeclarationKind,
    pub symbol: AtpSymbolName,
    pub arity: u32,
    pub provenance: AtpProjectionProvenance,
    pub symbol_source: AtpSymbolSourceProjection,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum AtpSymbolSourceProjection {
    MizarSymbol(AtpSourceBinding),
    GeneratedBinder(AtpSourceBinding),
    TypeGuard(AtpProjectionKey),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AtpSoftTypeProjection {
    pub key: AtpProjectionKey,
    pub representation: AtpSoftTypeRepresentation,
    pub provenance: AtpProjectionProvenance,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum AtpSoftTypeRepresentation {
    BackendSortLossless,
    GuardFormula(AtpFormulaTree),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum AtpFormulaProjectionTarget {
    VcFormula(VcFormulaRef),
    ImportedFact(VcText),
}

impl AtpFormulaProjectionTarget {
    pub fn imported(symbol: impl Into<VcText>) -> Self {
        Self::ImportedFact(symbol.into())
    }

    pub const fn vc_formula(formula: VcFormulaRef) -> Self {
        Self::VcFormula(formula)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AtpFormulaProjection {
    pub target: AtpFormulaProjectionTarget,
    pub formula: AtpFormulaTree,
    pub provenance: AtpProjectionProvenance,
    pub source_identity: AtpProjectionKey,
    pub handoff_formula_fingerprint: AtpFingerprint,
    pub handoff_provenance_payload: Vec<u8>,
}

pub struct AtpDeclarationTranslationInput<'a> {
    pub vc_set: &'a VcSet,
    pub vc: VcId,
    pub kernel_handoff: &'a VcKernelEvidenceHandoff,
    pub logic_profile: LogicProfile,
    pub declaration_projections: Vec<AtpDeclarationProjection>,
    pub soft_type_projections: Vec<AtpSoftTypeProjection>,
    pub diagnostics: Vec<AtpDiagnostic>,
}

pub struct AtpTranslationInput<'a> {
    pub vc_set: &'a VcSet,
    pub vc: VcId,
    pub kernel_handoff: &'a VcKernelEvidenceHandoff,
    pub logic_profile: LogicProfile,
    pub declaration_projections: Vec<AtpDeclarationProjection>,
    pub soft_type_projections: Vec<AtpSoftTypeProjection>,
    pub formula_projections: Vec<AtpFormulaProjection>,
    pub diagnostics: Vec<AtpDiagnostic>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AtpDeclarationTranslation {
    vc_id: VcId,
    target_binding: AtpTargetBinding,
    logic_profile: LogicProfile,
    declarations: Vec<AtpDeclaration>,
    type_context: AtpTypeContext,
    symbol_map: Vec<AtpSymbolMapEntry>,
    provenance: Vec<AtpProvenance>,
    diagnostics: Vec<AtpDiagnostic>,
}

impl AtpDeclarationTranslation {
    pub const fn vc_id(&self) -> VcId {
        self.vc_id
    }

    pub const fn target_binding(&self) -> &AtpTargetBinding {
        &self.target_binding
    }

    pub const fn logic_profile(&self) -> &LogicProfile {
        &self.logic_profile
    }

    pub fn declarations(&self) -> &[AtpDeclaration] {
        &self.declarations
    }

    pub const fn type_context(&self) -> &AtpTypeContext {
        &self.type_context
    }

    pub fn symbol_map(&self) -> &[AtpSymbolMapEntry] {
        &self.symbol_map
    }

    pub fn provenance(&self) -> &[AtpProvenance] {
        &self.provenance
    }

    pub fn diagnostics(&self) -> &[AtpDiagnostic] {
        &self.diagnostics
    }
}

#[derive(Debug)]
#[non_exhaustive]
pub enum AtpTranslationError {
    UnknownVc {
        vc: VcId,
    },
    NonNeedsAtpStatus {
        vc: VcId,
        status: VcStatus,
    },
    HandoffTarget {
        source: KernelEvidenceHandoffError,
    },
    MismatchedTargetHandoff {
        vc: VcId,
    },
    EmptyProjectionKey {
        section: &'static str,
    },
    DuplicateProjectionKey {
        section: &'static str,
        key: AtpProjectionKey,
    },
    DuplicateFormulaProjection {
        target: AtpFormulaProjectionTarget,
    },
    DuplicateFormulaProjectionIdentity {
        identity: AtpProjectionKey,
    },
    MissingFormulaProjection {
        target: AtpFormulaProjectionTarget,
    },
    MissingFormulaHandoffEvidence {
        target: AtpFormulaProjectionTarget,
    },
    FormulaHandoffAgreement {
        target: AtpFormulaProjectionTarget,
    },
    DuplicatePremiseRef {
        premise: PremiseRef,
    },
    DuplicatePremiseIdentity {
        identity: AtpProjectionKey,
    },
    DuplicatePremiseFormula {
        formula: VcFormulaRef,
    },
    PremiseCopiesGoal {
        formula: VcFormulaRef,
    },
    MissingLocalContextPremise {
        context: ContextEntryId,
    },
    LocalContextPremiseWithoutFormula {
        context: ContextEntryId,
    },
    UnsupportedPremiseRef {
        premise: PremiseRef,
        reason: &'static str,
    },
    MissingTypeGuardProjection {
        key: AtpProjectionKey,
    },
    MissingSoftTypeGuard {
        key: AtpProjectionKey,
        strategy: SoftTypeStrategy,
    },
    IdOverflow {
        section: &'static str,
    },
    Problem {
        source: AtpProblemError,
    },
}

impl fmt::Display for AtpTranslationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownVc { vc } => write!(formatter, "unknown VC {vc:?}"),
            Self::NonNeedsAtpStatus { vc, status } => {
                write!(
                    formatter,
                    "VC {vc:?} has non-ATP translator status {status:?}"
                )
            }
            Self::HandoffTarget { source } => {
                write!(formatter, "failed to validate handoff target: {source}")
            }
            Self::MismatchedTargetHandoff { vc } => {
                write!(formatter, "kernel handoff does not target VC {vc:?}")
            }
            Self::EmptyProjectionKey { section } => {
                write!(formatter, "empty projection key in {section}")
            }
            Self::DuplicateProjectionKey { section, key } => {
                write!(
                    formatter,
                    "duplicate projection key {} in {section}",
                    key.as_str()
                )
            }
            Self::DuplicateFormulaProjection { target } => {
                write!(formatter, "duplicate formula projection for {target:?}")
            }
            Self::DuplicateFormulaProjectionIdentity { identity } => {
                write!(
                    formatter,
                    "duplicate formula projection source identity {}",
                    identity.as_str()
                )
            }
            Self::MissingFormulaProjection { target } => {
                write!(formatter, "missing formula projection for {target:?}")
            }
            Self::MissingFormulaHandoffEvidence { target } => {
                write!(
                    formatter,
                    "missing kernel handoff formula evidence for {target:?}"
                )
            }
            Self::FormulaHandoffAgreement { target } => {
                write!(
                    formatter,
                    "formula projection does not agree with kernel handoff for {target:?}"
                )
            }
            Self::DuplicatePremiseRef { premise } => {
                write!(formatter, "duplicate premise reference {premise:?}")
            }
            Self::DuplicatePremiseIdentity { identity } => {
                write!(
                    formatter,
                    "duplicate premise source/formula identity {}",
                    identity.as_str()
                )
            }
            Self::DuplicatePremiseFormula { formula } => {
                write!(formatter, "duplicate premise formula reference {formula:?}")
            }
            Self::PremiseCopiesGoal { formula } => {
                write!(formatter, "premise formula {formula:?} is also the VC goal")
            }
            Self::MissingLocalContextPremise { context } => {
                write!(formatter, "missing local context premise {context:?}")
            }
            Self::LocalContextPremiseWithoutFormula { context } => {
                write!(
                    formatter,
                    "local context premise {context:?} has no formula payload"
                )
            }
            Self::UnsupportedPremiseRef { premise, reason } => {
                write!(
                    formatter,
                    "unsupported premise reference {premise:?}: {reason}"
                )
            }
            Self::MissingTypeGuardProjection { key } => {
                write!(formatter, "missing type-guard projection {}", key.as_str())
            }
            Self::MissingSoftTypeGuard { key, strategy } => {
                write!(
                    formatter,
                    "soft-type projection {} requires a guard under {strategy:?}",
                    key.as_str()
                )
            }
            Self::IdOverflow { section } => write!(formatter, "{section} id overflow"),
            Self::Problem { source } => write!(formatter, "{source}"),
        }
    }
}

impl Error for AtpTranslationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::HandoffTarget { source } => Some(source),
            Self::Problem { source } => Some(source),
            _ => None,
        }
    }
}

impl From<AtpProblemError> for AtpTranslationError {
    fn from(source: AtpProblemError) -> Self {
        Self::Problem { source }
    }
}

pub fn translate_declarations(
    input: AtpDeclarationTranslationInput<'_>,
) -> Result<AtpDeclarationTranslation, AtpTranslationError> {
    let vc = input
        .vc_set
        .vc(input.vc)
        .ok_or(AtpTranslationError::UnknownVc { vc: input.vc })?;
    if vc.status != VcStatus::NeedsAtp {
        return Err(AtpTranslationError::NonNeedsAtpStatus {
            vc: input.vc,
            status: vc.status.clone(),
        });
    }
    if !input
        .kernel_handoff
        .targets_vc(input.vc_set, input.vc)
        .map_err(|source| AtpTranslationError::HandoffTarget { source })?
    {
        return Err(AtpTranslationError::MismatchedTargetHandoff { vc: input.vc });
    }

    let target_binding = target_binding(input.kernel_handoff)?;
    let soft_type_projections = sort_soft_type_projections(input.soft_type_projections)?;
    let type_guard_ids = type_guard_ids(&soft_type_projections)?;
    let declaration_projections = sort_declaration_projections(input.declaration_projections)?;
    let mut provenance = Vec::new();

    let (type_context, mut soft_type_provenance) = build_type_context(
        &soft_type_projections,
        &input.logic_profile,
        next_provenance_id(provenance.len())?,
    )?;
    provenance.append(&mut soft_type_provenance);

    let mut declaration_output = build_declarations(
        &declaration_projections,
        &type_guard_ids,
        next_provenance_id(provenance.len())?,
    )?;
    provenance.append(&mut declaration_output.provenance);

    let translation = AtpDeclarationTranslation {
        vc_id: input.vc,
        target_binding,
        logic_profile: input.logic_profile,
        declarations: declaration_output.declarations,
        type_context,
        symbol_map: declaration_output.symbol_map,
        provenance,
        diagnostics: input.diagnostics,
    };
    validate_translation(&translation)?;
    Ok(translation)
}

pub fn translate_problem(
    input: AtpTranslationInput<'_>,
) -> Result<AtpProblem, AtpTranslationError> {
    let vc_set = input.vc_set;
    let vc_id = input.vc;
    let kernel_handoff = input.kernel_handoff;
    let formula_projections = input.formula_projections;
    let declaration_translation = translate_declarations(AtpDeclarationTranslationInput {
        vc_set,
        vc: vc_id,
        kernel_handoff,
        logic_profile: input.logic_profile,
        declaration_projections: input.declaration_projections,
        soft_type_projections: input.soft_type_projections,
        diagnostics: input.diagnostics,
    })?;
    let vc = vc_set
        .vc(vc_id)
        .ok_or(AtpTranslationError::UnknownVc { vc: vc_id })?;
    let formula_projections = formula_projection_map(formula_projections)?;
    let mut provenance = declaration_translation.provenance.clone();
    let first_formula_provenance = next_provenance_id(provenance.len())?;
    let (axioms, mut formula_provenance) = materialize_axioms(
        vc,
        &formula_projections,
        kernel_handoff,
        first_formula_provenance,
    )?;
    provenance.append(&mut formula_provenance);
    let conjecture_provenance_id = next_provenance_id(provenance.len())?;
    let (conjecture, conjecture_provenance) = materialize_conjecture(
        vc.goal,
        &formula_projections,
        kernel_handoff,
        AtpFormulaId::new(index_to_u32(axioms.len(), "formulas")?),
        conjecture_provenance_id,
    )?;
    provenance.push(conjecture_provenance);

    AtpProblem::try_new(AtpProblemParts {
        vc_id,
        target_binding: declaration_translation.target_binding,
        logic_profile: declaration_translation.logic_profile,
        expected_result: ExpectedBackendResult::Unsat,
        declarations: declaration_translation.declarations,
        axioms,
        conjecture,
        type_context: declaration_translation.type_context,
        properties: Vec::new(),
        symbol_map: declaration_translation.symbol_map,
        provenance,
        diagnostics: declaration_translation.diagnostics,
    })
    .map_err(Into::into)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FormulaSourceExpectation {
    LocalHypothesis { context: ContextEntryId },
    CitedPremise { context: ContextEntryId },
    GeneratedPremise { formula: VcGeneratedFormulaId },
    ImportedFact,
}

fn formula_projection_map(
    mut projections: Vec<AtpFormulaProjection>,
) -> Result<BTreeMap<AtpFormulaProjectionTarget, AtpFormulaProjection>, AtpTranslationError> {
    projections.sort_by(|left, right| {
        left.target
            .cmp(&right.target)
            .then_with(|| left.source_identity.cmp(&right.source_identity))
    });

    let mut by_target = BTreeMap::new();
    let mut by_identity = BTreeSet::new();
    for projection in projections {
        validate_formula_projection(&projection)?;
        if !by_identity.insert(projection.source_identity.clone()) {
            return Err(AtpTranslationError::DuplicateFormulaProjectionIdentity {
                identity: projection.source_identity,
            });
        }
        let target = projection.target.clone();
        if by_target.insert(target.clone(), projection).is_some() {
            return Err(AtpTranslationError::DuplicateFormulaProjection { target });
        }
    }
    Ok(by_target)
}

fn validate_formula_projection(
    projection: &AtpFormulaProjection,
) -> Result<(), AtpTranslationError> {
    if projection.source_identity.is_empty() {
        return Err(AtpTranslationError::EmptyProjectionKey {
            section: "formula projection source identities",
        });
    }
    if let AtpFormulaProjectionTarget::ImportedFact(symbol) = &projection.target
        && symbol.as_str().trim().is_empty()
    {
        return Err(AtpProblemError::EmptyField {
            field: "formula_projection.imported_symbol",
        }
        .into());
    }
    if projection.handoff_provenance_payload.is_empty() {
        return Err(AtpProblemError::EmptyField {
            field: "formula_projection.handoff_provenance_payload",
        }
        .into());
    }
    validate_projection_provenance(&projection.provenance)
}

fn materialize_axioms(
    vc: &mizar_vc::vc_ir::VcIr,
    projections: &BTreeMap<AtpFormulaProjectionTarget, AtpFormulaProjection>,
    handoff: &VcKernelEvidenceHandoff,
    first_provenance_id: AtpProvenanceId,
) -> Result<(Vec<AtpFormula>, Vec<AtpProvenance>), AtpTranslationError> {
    let mut premise_refs = vc.premises.clone();
    premise_refs.sort();
    reject_duplicate_premise_refs(&premise_refs)?;

    let mut axioms = Vec::new();
    let mut provenance = Vec::new();
    let mut source_identities = BTreeSet::new();
    let mut formula_refs = BTreeSet::new();

    for premise in premise_refs {
        let (target, expectation, resolved_formula) = premise_projection_target(vc, &premise)?;
        if let Some(formula) = resolved_formula
            && !formula_refs.insert(formula)
        {
            return Err(AtpTranslationError::DuplicatePremiseFormula { formula });
        }
        if let Some(formula) = resolved_formula
            && formula == vc.goal
        {
            return Err(AtpTranslationError::PremiseCopiesGoal { formula });
        }
        let projection = projections.get(&target).ok_or_else(|| {
            AtpTranslationError::MissingFormulaProjection {
                target: target.clone(),
            }
        })?;
        validate_formula_source(
            &projection.provenance.source,
            expectation,
            &projection.source_identity,
            &target,
        )?;
        validate_formula_handoff(&target, projection, handoff, Some(expectation))?;
        let premise_identity = premise_identity_key(&target, projection, handoff)?;
        if !source_identities.insert(premise_identity.clone()) {
            return Err(AtpTranslationError::DuplicatePremiseIdentity {
                identity: premise_identity,
            });
        }

        let provenance_id = offset_provenance_id(first_provenance_id, provenance.len())?;
        provenance.push(AtpProvenance::new(
            provenance_id,
            projection.provenance.source.clone(),
            handoff_anchor_payload(&projection.handoff_provenance_payload),
        ));
        axioms.push(AtpFormula::new(
            AtpFormulaId::new(index_to_u32(axioms.len(), "formulas")?),
            projection.formula.clone(),
            provenance_id,
        ));
    }

    Ok((axioms, provenance))
}

fn materialize_conjecture(
    goal: VcFormulaRef,
    projections: &BTreeMap<AtpFormulaProjectionTarget, AtpFormulaProjection>,
    handoff: &VcKernelEvidenceHandoff,
    formula_id: AtpFormulaId,
    provenance_id: AtpProvenanceId,
) -> Result<(AtpFormula, AtpProvenance), AtpTranslationError> {
    let target = AtpFormulaProjectionTarget::VcFormula(goal);
    let projection =
        projections
            .get(&target)
            .ok_or_else(|| AtpTranslationError::MissingFormulaProjection {
                target: target.clone(),
            })?;
    validate_goal_source(&projection.provenance.source, &projection.source_identity)?;
    validate_formula_handoff(&target, projection, handoff, None)?;
    Ok((
        AtpFormula::new(formula_id, projection.formula.clone(), provenance_id),
        AtpProvenance::new(
            provenance_id,
            projection.provenance.source.clone(),
            handoff_anchor_payload(&projection.handoff_provenance_payload),
        ),
    ))
}

fn reject_duplicate_premise_refs(premises: &[PremiseRef]) -> Result<(), AtpTranslationError> {
    let mut seen = BTreeSet::new();
    for premise in premises {
        if !seen.insert(premise.clone()) {
            return Err(AtpTranslationError::DuplicatePremiseRef {
                premise: premise.clone(),
            });
        }
    }
    Ok(())
}

fn premise_projection_target(
    vc: &mizar_vc::vc_ir::VcIr,
    premise: &PremiseRef,
) -> Result<
    (
        AtpFormulaProjectionTarget,
        FormulaSourceExpectation,
        Option<VcFormulaRef>,
    ),
    AtpTranslationError,
> {
    match premise {
        PremiseRef::LocalContext(context) => {
            let entry = vc
                .local_context
                .entries()
                .iter()
                .find(|entry| entry.id == *context)
                .ok_or(AtpTranslationError::MissingLocalContextPremise { context: *context })?;
            let formula =
                entry
                    .formula
                    .ok_or(AtpTranslationError::LocalContextPremiseWithoutFormula {
                        context: *context,
                    })?;
            let expectation = if matches!(entry.kind, ContextEntryKind::CitedPremise) {
                FormulaSourceExpectation::CitedPremise { context: *context }
            } else {
                FormulaSourceExpectation::LocalHypothesis { context: *context }
            };
            Ok((
                AtpFormulaProjectionTarget::VcFormula(formula),
                expectation,
                Some(formula),
            ))
        }
        PremiseRef::GeneratedFact { formula } => {
            let VcFormulaRef::Generated(generated) = formula else {
                return Err(AtpTranslationError::UnsupportedPremiseRef {
                    premise: premise.clone(),
                    reason: "generated fact premise is not a generated VC formula",
                });
            };
            Ok((
                AtpFormulaProjectionTarget::VcFormula(*formula),
                FormulaSourceExpectation::GeneratedPremise {
                    formula: *generated,
                },
                Some(*formula),
            ))
        }
        PremiseRef::CheckerFact { .. } => Err(AtpTranslationError::UnsupportedPremiseRef {
            premise: premise.clone(),
            reason: "checker-owned premise requires an explicit handoff source class",
        }),
        PremiseRef::TypePredicate { .. } => Err(AtpTranslationError::UnsupportedPremiseRef {
            premise: premise.clone(),
            reason: "type-predicate premise requires an explicit handoff source class",
        }),
        PremiseRef::ImportedFact { symbol } => Ok((
            AtpFormulaProjectionTarget::ImportedFact(symbol.clone()),
            FormulaSourceExpectation::ImportedFact,
            None,
        )),
        PremiseRef::ConservativeUnknown { .. } => Err(AtpTranslationError::UnsupportedPremiseRef {
            premise: premise.clone(),
            reason: "conservative unknown premise has no formula payload",
        }),
        _ => Err(AtpTranslationError::UnsupportedPremiseRef {
            premise: premise.clone(),
            reason: "premise has no explicit ATP formula projection binding",
        }),
    }
}

fn validate_formula_source(
    source: &AtpSourceRef,
    expectation: FormulaSourceExpectation,
    source_identity: &AtpProjectionKey,
    target: &AtpFormulaProjectionTarget,
) -> Result<(), AtpTranslationError> {
    let valid = match (expectation, source) {
        (
            FormulaSourceExpectation::LocalHypothesis { context },
            AtpSourceRef::LocalHypothesis(binding),
        ) => source_binding_matches(
            binding,
            source_identity,
            &format!("local-context:{}", context.index() + 1),
        ),
        (
            FormulaSourceExpectation::CitedPremise { context },
            AtpSourceRef::CitedPremise(binding),
        ) => source_binding_matches(
            binding,
            source_identity,
            &format!("cited-premise:{}", context.index() + 1),
        ),
        (
            FormulaSourceExpectation::GeneratedPremise { formula },
            AtpSourceRef::GeneratedVcFact(binding),
        ) => source_binding_matches(
            binding,
            source_identity,
            &format!("generated:{}", formula.index() + 1),
        ),
        (
            FormulaSourceExpectation::ImportedFact,
            AtpSourceRef::ImportedAxiom { .. } | AtpSourceRef::ImportedTheorem { .. },
        ) => {
            let AtpFormulaProjectionTarget::ImportedFact(symbol) = target else {
                return Err(AtpTranslationError::FormulaHandoffAgreement {
                    target: target.clone(),
                });
            };
            source_identity.as_str() == format!("imported:{}", symbol.as_str())
        }
        _ => false,
    };
    if valid {
        Ok(())
    } else {
        Err(AtpProblemError::UnsupportedProfileFeature {
            feature: "formula provenance source class",
        }
        .into())
    }
}

fn validate_goal_source(
    source: &AtpSourceRef,
    source_identity: &AtpProjectionKey,
) -> Result<(), AtpTranslationError> {
    if matches!(
        source,
        AtpSourceRef::GeneratedVcFact(binding)
            if source_binding_matches(binding, source_identity, "goal:1")
    ) {
        Ok(())
    } else {
        Err(AtpProblemError::UnsupportedProfileFeature {
            feature: "conjecture provenance source class",
        }
        .into())
    }
}

fn handoff_anchor_payload(payload: &[u8]) -> AtpPayload {
    AtpPayload::new(format!("mizar-vc-handoff-provenance:{}", hex(payload)))
}

fn premise_identity_key(
    target: &AtpFormulaProjectionTarget,
    projection: &AtpFormulaProjection,
    handoff: &VcKernelEvidenceHandoff,
) -> Result<AtpProjectionKey, AtpTranslationError> {
    if matches!(target, AtpFormulaProjectionTarget::ImportedFact(_)) {
        Ok(imported_identity_key(&imported_projection_metadata(
            projection, handoff,
        )?))
    } else {
        Ok(projection.source_identity.clone())
    }
}

fn source_binding_matches(
    binding: &AtpSourceBinding,
    source_identity: &AtpProjectionKey,
    expected: &str,
) -> bool {
    binding.as_str() == expected && source_identity.as_str() == expected
}

fn validate_formula_handoff(
    target: &AtpFormulaProjectionTarget,
    projection: &AtpFormulaProjection,
    handoff: &VcKernelEvidenceHandoff,
    expectation: Option<FormulaSourceExpectation>,
) -> Result<(), AtpTranslationError> {
    match target {
        AtpFormulaProjectionTarget::VcFormula(formula) => {
            if expectation.is_none() {
                let final_goal = handoff.canonical_evidence().final_goal();
                if final_goal.producer_formula_ref != *formula
                    || final_goal.polarity != KernelGoalPolarity::AssertFalseForRefutation
                {
                    return Err(AtpTranslationError::FormulaHandoffAgreement {
                        target: target.clone(),
                    });
                }
                validate_handoff_fingerprint_and_payload(
                    target,
                    projection,
                    &final_goal.formula_fingerprint,
                    final_goal.provenance_id,
                    handoff,
                )
            } else {
                let entry = handoff
                    .canonical_evidence()
                    .formula_evidence()
                    .iter()
                    .find(|entry| {
                        entry.producer_formula_ref() == Some(*formula)
                            && expectation.is_some_and(|expected| {
                                kernel_source_matches(entry.source(), expected)
                            })
                    })
                    .ok_or_else(|| AtpTranslationError::MissingFormulaHandoffEvidence {
                        target: target.clone(),
                    })?;
                validate_entry_handoff_agreement(target, projection, entry, handoff)
            }
        }
        AtpFormulaProjectionTarget::ImportedFact(_) => {
            let imported = imported_projection_metadata(projection, handoff)?;
            let imported_matches = handoff
                .canonical_evidence()
                .formula_evidence()
                .iter()
                .filter(|entry| {
                    imported_source_matches(entry.source(), &imported)
                        && formula_fingerprint_matches(
                            &projection.handoff_formula_fingerprint,
                            entry.formula_fingerprint(),
                        )
                        && handoff_provenance_payload(handoff, entry.provenance_id())
                            == Some(projection.handoff_provenance_payload.as_slice())
                })
                .count();
            match imported_matches {
                1 => Ok(()),
                0 => Err(AtpTranslationError::MissingFormulaHandoffEvidence {
                    target: target.clone(),
                }),
                _ => Err(AtpTranslationError::FormulaHandoffAgreement {
                    target: target.clone(),
                }),
            }
        }
    }
}

fn validate_entry_handoff_agreement(
    target: &AtpFormulaProjectionTarget,
    projection: &AtpFormulaProjection,
    entry: &KernelFormulaEvidenceEntry,
    handoff: &VcKernelEvidenceHandoff,
) -> Result<(), AtpTranslationError> {
    validate_handoff_fingerprint_and_payload(
        target,
        projection,
        entry.formula_fingerprint(),
        entry.provenance_id(),
        handoff,
    )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum ImportedProjectionClass {
    Axiom,
    Theorem,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct ImportedProjectionMetadata {
    class: ImportedProjectionClass,
    requirement: KernelImportedFactRequirement,
}

struct ImportedProjectionFields<'a> {
    package: &'a AtpSourceBinding,
    module: &'a AtpSourceBinding,
    item: &'a AtpSourceBinding,
    statement_fingerprint: &'a AtpFingerprint,
    required_status: &'a AtpRequiredProofStatus,
    context_requirement: &'a AtpSourceBinding,
}

fn imported_projection_metadata(
    projection: &AtpFormulaProjection,
    handoff: &VcKernelEvidenceHandoff,
) -> Result<ImportedProjectionMetadata, AtpTranslationError> {
    match &projection.provenance.source {
        AtpSourceRef::ImportedAxiom {
            package,
            module,
            item,
            statement_fingerprint,
            required_status,
            context_requirement,
        } => imported_metadata(
            ImportedProjectionClass::Axiom,
            ImportedProjectionFields {
                package,
                module,
                item,
                statement_fingerprint,
                required_status,
                context_requirement,
            },
            handoff,
        ),
        AtpSourceRef::ImportedTheorem {
            package,
            module,
            item,
            statement_fingerprint,
            required_status,
            context_requirement,
        } => imported_metadata(
            ImportedProjectionClass::Theorem,
            ImportedProjectionFields {
                package,
                module,
                item,
                statement_fingerprint,
                required_status,
                context_requirement,
            },
            handoff,
        ),
        _ => Err(AtpProblemError::UnsupportedProfileFeature {
            feature: "imported formula provenance source class",
        }
        .into()),
    }
}

fn imported_metadata(
    class: ImportedProjectionClass,
    fields: ImportedProjectionFields<'_>,
    handoff: &VcKernelEvidenceHandoff,
) -> Result<ImportedProjectionMetadata, AtpTranslationError> {
    let context = handoff.formula_context_requirements().ok_or_else(|| {
        AtpTranslationError::MissingFormulaHandoffEvidence {
            target: AtpFormulaProjectionTarget::ImportedFact(VcText::new("<context>")),
        }
    })?;
    let expected_context = formula_context_binding(context);
    if fields.context_requirement.as_str() != expected_context {
        return Err(AtpProblemError::UnsupportedProfileFeature {
            feature: "imported formula context requirement",
        }
        .into());
    }
    Ok(ImportedProjectionMetadata {
        class,
        requirement: KernelImportedFactRequirement {
            imported_fact_id: 0,
            package_id: fields.package.as_str().as_bytes().to_vec(),
            module_path: fields.module.as_str().as_bytes().to_vec(),
            exported_item_id: fields.item.as_str().as_bytes().to_vec(),
            statement_fingerprint: mizar_vc::kernel_evidence_handoff::KernelEvidenceFingerprint {
                algorithm_id: fields.statement_fingerprint.algorithm_id(),
                digest: fields.statement_fingerprint.digest().to_vec(),
            },
            required_proof_status: required_kernel_status(fields.required_status)?,
        },
    })
}

fn formula_context_binding(
    context: &mizar_vc::kernel_evidence_handoff::KernelFormulaContextRequirements,
) -> String {
    format!(
        "mizar-vc-kernel-formula-context:{}:{}",
        context.provenance_fingerprint.algorithm_id,
        hex(&context.provenance_fingerprint.digest)
    )
}

fn required_kernel_status(
    status: &AtpRequiredProofStatus,
) -> Result<KernelRequiredProofStatus, AtpTranslationError> {
    match status.as_str() {
        "KernelVerified" => Ok(KernelRequiredProofStatus::KernelVerified),
        "DischargedBuiltin" => Ok(KernelRequiredProofStatus::DischargedBuiltin),
        "ExternallyAttestedPolicyPermitted" => {
            Ok(KernelRequiredProofStatus::ExternallyAttestedPolicyPermitted)
        }
        _ => Err(AtpProblemError::UnsupportedProfileFeature {
            feature: "imported required proof status",
        }
        .into()),
    }
}

fn validate_handoff_fingerprint_and_payload(
    target: &AtpFormulaProjectionTarget,
    projection: &AtpFormulaProjection,
    fingerprint: &mizar_vc::kernel_evidence_handoff::KernelEvidenceFingerprint,
    provenance_id: u32,
    handoff: &VcKernelEvidenceHandoff,
) -> Result<(), AtpTranslationError> {
    if !formula_fingerprint_matches(&projection.handoff_formula_fingerprint, fingerprint)
        || handoff_provenance_payload(handoff, provenance_id)
            != Some(projection.handoff_provenance_payload.as_slice())
    {
        return Err(AtpTranslationError::FormulaHandoffAgreement {
            target: target.clone(),
        });
    }
    Ok(())
}

fn kernel_source_matches(
    source: &KernelFormulaSource,
    expectation: FormulaSourceExpectation,
) -> bool {
    match expectation {
        FormulaSourceExpectation::LocalHypothesis { context } => {
            matches!(
                source,
                KernelFormulaSource::LocalHypothesis { local_context_id }
                    if *local_context_id == (context.index() + 1) as u32
            )
        }
        FormulaSourceExpectation::CitedPremise { context } => {
            matches!(
                source,
                KernelFormulaSource::CitedPremise { local_context_id }
                    if *local_context_id == (context.index() + 1) as u32
            )
        }
        FormulaSourceExpectation::GeneratedPremise { formula } => {
            matches!(
                source,
                KernelFormulaSource::GeneratedVcFact { vc_fact_id }
                    if *vc_fact_id == (formula.index() + 1) as u32
            )
        }
        FormulaSourceExpectation::ImportedFact => false,
    }
}

fn imported_source_matches(
    source: &KernelFormulaSource,
    expected: &ImportedProjectionMetadata,
) -> bool {
    match (source, expected.class) {
        (KernelFormulaSource::AcceptedImportedAxiom(actual), ImportedProjectionClass::Axiom)
        | (
            KernelFormulaSource::AcceptedImportedTheorem(actual),
            ImportedProjectionClass::Theorem,
        ) => imported_requirement_matches(actual, &expected.requirement),
        _ => false,
    }
}

fn imported_requirement_matches(
    actual: &KernelImportedFactRequirement,
    expected: &KernelImportedFactRequirement,
) -> bool {
    actual.package_id == expected.package_id
        && actual.module_path == expected.module_path
        && actual.exported_item_id == expected.exported_item_id
        && actual.statement_fingerprint == expected.statement_fingerprint
        && actual.required_proof_status == expected.required_proof_status
}

fn imported_identity_key(metadata: &ImportedProjectionMetadata) -> AtpProjectionKey {
    let requirement = &metadata.requirement;
    AtpProjectionKey::new(format!(
        "imported-source:{}:pkg={}:module={}:item={}:statement={}:{}:status={}",
        imported_class_label(metadata.class),
        hex(&requirement.package_id),
        hex(&requirement.module_path),
        hex(&requirement.exported_item_id),
        requirement.statement_fingerprint.algorithm_id,
        hex(&requirement.statement_fingerprint.digest),
        kernel_required_status_label(requirement.required_proof_status),
    ))
}

fn imported_class_label(class: ImportedProjectionClass) -> &'static str {
    match class {
        ImportedProjectionClass::Axiom => "axiom",
        ImportedProjectionClass::Theorem => "theorem",
    }
}

fn kernel_required_status_label(status: KernelRequiredProofStatus) -> &'static str {
    match status {
        KernelRequiredProofStatus::KernelVerified => "KernelVerified",
        KernelRequiredProofStatus::DischargedBuiltin => "DischargedBuiltin",
        KernelRequiredProofStatus::ExternallyAttestedPolicyPermitted => {
            "ExternallyAttestedPolicyPermitted"
        }
        _ => "Unknown",
    }
}

fn formula_fingerprint_matches(
    projection: &AtpFingerprint,
    handoff: &mizar_vc::kernel_evidence_handoff::KernelEvidenceFingerprint,
) -> bool {
    projection.algorithm_id() == handoff.algorithm_id
        && projection.digest() == handoff.digest.as_slice()
}

fn handoff_provenance_payload(
    handoff: &VcKernelEvidenceHandoff,
    provenance_id: u32,
) -> Option<&[u8]> {
    handoff
        .canonical_evidence()
        .provenance()
        .iter()
        .find(|provenance| provenance.provenance_id == provenance_id)
        .map(|provenance| provenance.payload.as_slice())
}

fn sort_declaration_projections(
    mut projections: Vec<AtpDeclarationProjection>,
) -> Result<Vec<AtpDeclarationProjection>, AtpTranslationError> {
    projections.sort_by(|left, right| {
        left.key
            .cmp(&right.key)
            .then_with(|| left.kind.cmp(&right.kind))
            .then_with(|| left.arity.cmp(&right.arity))
            .then_with(|| left.symbol.cmp(&right.symbol))
    });
    reject_empty_or_duplicate_keys(
        projections.iter().map(|projection| &projection.key),
        "declaration projections",
    )?;
    Ok(projections)
}

fn sort_soft_type_projections(
    mut projections: Vec<AtpSoftTypeProjection>,
) -> Result<Vec<AtpSoftTypeProjection>, AtpTranslationError> {
    projections.sort_by(|left, right| left.key.cmp(&right.key));
    reject_empty_or_duplicate_keys(
        projections.iter().map(|projection| &projection.key),
        "soft-type projections",
    )?;
    Ok(projections)
}

fn reject_empty_or_duplicate_keys<'a>(
    keys: impl Iterator<Item = &'a AtpProjectionKey>,
    section: &'static str,
) -> Result<(), AtpTranslationError> {
    let mut seen = BTreeSet::new();
    for key in keys {
        if key.is_empty() {
            return Err(AtpTranslationError::EmptyProjectionKey { section });
        }
        if !seen.insert(key.clone()) {
            return Err(AtpTranslationError::DuplicateProjectionKey {
                section,
                key: key.clone(),
            });
        }
    }
    Ok(())
}

fn type_guard_ids(
    projections: &[AtpSoftTypeProjection],
) -> Result<BTreeMap<AtpProjectionKey, AtpTypeGuardId>, AtpTranslationError> {
    let mut ids = BTreeMap::new();
    let mut guard_index = 0usize;
    for projection in projections {
        if matches!(
            projection.representation,
            AtpSoftTypeRepresentation::GuardFormula(_)
        ) {
            ids.insert(
                projection.key.clone(),
                AtpTypeGuardId::new(index_to_u32(guard_index, "type guards")?),
            );
            guard_index += 1;
        }
    }
    Ok(ids)
}

fn build_type_context(
    projections: &[AtpSoftTypeProjection],
    profile: &LogicProfile,
    first_provenance_id: AtpProvenanceId,
) -> Result<(AtpTypeContext, Vec<AtpProvenance>), AtpTranslationError> {
    let mut guards = Vec::new();
    let mut provenance = Vec::new();
    for projection in projections {
        validate_projection_provenance(&projection.provenance)?;
        match &projection.representation {
            AtpSoftTypeRepresentation::BackendSortLossless => {
                if soft_type_strategy_requires_guards(profile.soft_types())? {
                    return Err(AtpTranslationError::MissingSoftTypeGuard {
                        key: projection.key.clone(),
                        strategy: profile.soft_types(),
                    });
                }
            }
            AtpSoftTypeRepresentation::GuardFormula(formula) => {
                let provenance_id = offset_provenance_id(first_provenance_id, provenance.len())?;
                let type_guard_id = AtpTypeGuardId::new(index_to_u32(guards.len(), "type guards")?);
                provenance.push(AtpProvenance::new(
                    provenance_id,
                    projection.provenance.source.clone(),
                    projection.provenance.payload.clone(),
                ));
                guards.push(AtpTypeGuard::new(
                    type_guard_id,
                    formula.clone(),
                    provenance_id,
                ));
            }
        }
    }
    Ok((AtpTypeContext::new(guards), provenance))
}

struct DeclarationBuildOutput {
    declarations: Vec<AtpDeclaration>,
    symbol_map: Vec<AtpSymbolMapEntry>,
    provenance: Vec<AtpProvenance>,
}

fn build_declarations(
    projections: &[AtpDeclarationProjection],
    type_guard_ids: &BTreeMap<AtpProjectionKey, AtpTypeGuardId>,
    first_provenance_id: AtpProvenanceId,
) -> Result<DeclarationBuildOutput, AtpTranslationError> {
    let mut declarations = Vec::new();
    let mut symbol_map = Vec::new();
    let mut provenance = Vec::new();
    for (index, projection) in projections.iter().enumerate() {
        validate_projection_provenance(&projection.provenance)?;
        let provenance_id = offset_provenance_id(first_provenance_id, index)?;
        let declaration_id = AtpDeclarationId::new(index_to_u32(index, "declarations")?);
        provenance.push(AtpProvenance::new(
            provenance_id,
            projection.provenance.source.clone(),
            projection.provenance.payload.clone(),
        ));
        declarations.push(AtpDeclaration::new(
            declaration_id,
            projection.kind,
            projection.symbol.clone(),
            projection.arity,
            provenance_id,
        ));
        symbol_map.push(AtpSymbolMapEntry::new(
            projection.symbol.clone(),
            symbol_source(&projection.symbol_source, type_guard_ids)?,
        ));
    }
    Ok(DeclarationBuildOutput {
        declarations,
        symbol_map,
        provenance,
    })
}

fn symbol_source(
    projection: &AtpSymbolSourceProjection,
    type_guard_ids: &BTreeMap<AtpProjectionKey, AtpTypeGuardId>,
) -> Result<AtpSymbolSource, AtpTranslationError> {
    match projection {
        AtpSymbolSourceProjection::MizarSymbol(binding) => {
            Ok(AtpSymbolSource::MizarSymbol(binding.clone()))
        }
        AtpSymbolSourceProjection::GeneratedBinder(binding) => {
            Ok(AtpSymbolSource::GeneratedBinder(binding.clone()))
        }
        AtpSymbolSourceProjection::TypeGuard(key) => {
            let id = type_guard_ids.get(key).copied().ok_or_else(|| {
                AtpTranslationError::MissingTypeGuardProjection { key: key.clone() }
            })?;
            Ok(AtpSymbolSource::TypeGuard(id))
        }
    }
}

fn validate_projection_provenance(
    provenance: &AtpProjectionProvenance,
) -> Result<(), AtpTranslationError> {
    if provenance.payload.is_empty() {
        return Err(AtpProblemError::EmptyField {
            field: "projection.provenance.payload",
        }
        .into());
    }
    validate_source_ref(&provenance.source).map_err(Into::into)
}

fn validate_source_ref(source: &AtpSourceRef) -> Result<(), AtpProblemError> {
    match source {
        AtpSourceRef::LocalHypothesis(binding)
        | AtpSourceRef::CitedPremise(binding)
        | AtpSourceRef::GeneratedVcFact(binding)
        | AtpSourceRef::CheckerOwnedFact(binding)
        | AtpSourceRef::TypeFact(binding)
        | AtpSourceRef::EncodedProperty(binding) => {
            require_nonempty_binding(binding, "projection.provenance.source")
        }
        AtpSourceRef::ImportedAxiom {
            package,
            module,
            item,
            required_status,
            context_requirement,
            ..
        }
        | AtpSourceRef::ImportedTheorem {
            package,
            module,
            item,
            required_status,
            context_requirement,
            ..
        } => {
            require_nonempty_binding(package, "imported.package")?;
            require_nonempty_binding(module, "imported.module")?;
            require_nonempty_binding(item, "imported.item")?;
            require_nonempty_status(required_status, "imported.required_status")?;
            require_nonempty_binding(context_requirement, "imported.context_requirement")
        }
    }
}

fn require_nonempty_binding(
    binding: &AtpSourceBinding,
    field: &'static str,
) -> Result<(), AtpProblemError> {
    if binding.is_empty() {
        Err(AtpProblemError::EmptyField { field })
    } else {
        Ok(())
    }
}

fn require_nonempty_status(
    status: &AtpRequiredProofStatus,
    field: &'static str,
) -> Result<(), AtpProblemError> {
    if status.is_empty() {
        Err(AtpProblemError::EmptyField { field })
    } else {
        Ok(())
    }
}

fn validate_translation(
    translation: &AtpDeclarationTranslation,
) -> Result<(), AtpTranslationError> {
    let provenance_ids = validate_provenance_rows(translation.provenance())?;
    let type_guard_ids = validate_type_context_skeleton(
        translation.type_context(),
        translation.logic_profile(),
        &provenance_ids,
    )?;
    let symbol_map = validate_symbol_map(translation.symbol_map(), &type_guard_ids)?;
    let declarations =
        validate_declarations(translation.declarations(), &provenance_ids, &symbol_map)?;
    validate_type_guard_formulas(
        translation.type_context(),
        translation.logic_profile(),
        &symbol_map,
        &declarations,
    )?;
    Ok(())
}

fn validate_provenance_rows(
    provenance: &[AtpProvenance],
) -> Result<BTreeSet<AtpProvenanceId>, AtpTranslationError> {
    let mut ids = BTreeSet::new();
    for row in provenance {
        if !ids.insert(row.id()) {
            return Err(AtpProblemError::DuplicateId {
                section: "provenance",
                id: row.id().index(),
            }
            .into());
        }
        if row.payload().is_empty() {
            return Err(AtpProblemError::EmptyField {
                field: "provenance.payload",
            }
            .into());
        }
        validate_source_ref(row.source())?;
    }
    Ok(ids)
}

fn validate_type_context_skeleton(
    context: &AtpTypeContext,
    profile: &LogicProfile,
    provenance: &BTreeSet<AtpProvenanceId>,
) -> Result<BTreeSet<AtpTypeGuardId>, AtpTranslationError> {
    let mut ids = BTreeSet::new();
    for guard in context.guards() {
        if !ids.insert(guard.id()) {
            return Err(AtpProblemError::DuplicateId {
                section: "type-guards",
                id: guard.id().index(),
            }
            .into());
        }
        require_provenance("type-guard", guard.provenance(), provenance)?;
    }
    if soft_type_strategy_requires_guards(profile.soft_types())? && context.guards().is_empty() {
        return Err(AtpProblemError::MissingTypeContextBinding {
            strategy: profile.soft_types(),
        }
        .into());
    }
    Ok(ids)
}

fn validate_symbol_map(
    entries: &[AtpSymbolMapEntry],
    type_guards: &BTreeSet<AtpTypeGuardId>,
) -> Result<BTreeSet<AtpSymbolName>, AtpTranslationError> {
    let mut symbols = BTreeSet::new();
    for entry in entries {
        if entry.backend_symbol().is_empty() {
            return Err(AtpProblemError::EmptyField {
                field: "symbol_map.backend_symbol",
            }
            .into());
        }
        if !symbols.insert(entry.backend_symbol().clone()) {
            return Err(AtpProblemError::DuplicateSymbolMap {
                symbol: entry.backend_symbol().clone(),
            }
            .into());
        }
        validate_symbol_source(entry.source(), type_guards)?;
    }
    Ok(symbols)
}

fn validate_symbol_source(
    source: &AtpSymbolSource,
    type_guards: &BTreeSet<AtpTypeGuardId>,
) -> Result<(), AtpTranslationError> {
    match source {
        AtpSymbolSource::MizarSymbol(binding) | AtpSymbolSource::GeneratedBinder(binding) => {
            require_nonempty_binding(binding, "symbol_map.source")?;
        }
        AtpSymbolSource::TypeGuard(id) => {
            if !type_guards.contains(id) {
                return Err(AtpProblemError::MissingTypeGuard { type_guard: *id }.into());
            }
        }
    }
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct DeclarationSignature {
    kind: AtpDeclarationKind,
    arity: u32,
}

fn validate_declarations(
    declarations: &[AtpDeclaration],
    provenance: &BTreeSet<AtpProvenanceId>,
    symbol_map: &BTreeSet<AtpSymbolName>,
) -> Result<BTreeMap<AtpSymbolName, DeclarationSignature>, AtpTranslationError> {
    let mut ids = BTreeSet::new();
    let mut by_symbol = BTreeMap::new();
    for declaration in declarations {
        if !ids.insert(declaration.id()) {
            return Err(AtpProblemError::DuplicateId {
                section: "declarations",
                id: declaration.id().index(),
            }
            .into());
        }
        if declaration.symbol().is_empty() {
            return Err(AtpProblemError::EmptyField {
                field: "declaration.symbol",
            }
            .into());
        }
        if by_symbol
            .insert(
                declaration.symbol().clone(),
                DeclarationSignature {
                    kind: declaration.kind(),
                    arity: declaration.arity(),
                },
            )
            .is_some()
        {
            return Err(AtpProblemError::DuplicateDeclarationSymbol {
                symbol: declaration.symbol().clone(),
            }
            .into());
        }
        require_provenance("declaration", declaration.provenance(), provenance)?;
        require_symbol(declaration.symbol(), symbol_map)?;
    }
    Ok(by_symbol)
}

fn validate_type_guard_formulas(
    context: &AtpTypeContext,
    profile: &LogicProfile,
    symbol_map: &BTreeSet<AtpSymbolName>,
    declarations: &BTreeMap<AtpSymbolName, DeclarationSignature>,
) -> Result<(), AtpTranslationError> {
    for guard in context.guards() {
        validate_formula_tree(guard.formula(), profile, symbol_map, declarations)?;
    }
    Ok(())
}

fn validate_formula_tree(
    formula: &AtpFormulaTree,
    profile: &LogicProfile,
    symbol_map: &BTreeSet<AtpSymbolName>,
    declarations: &BTreeMap<AtpSymbolName, DeclarationSignature>,
) -> Result<(), AtpTranslationError> {
    match formula {
        AtpFormulaTree::True | AtpFormulaTree::False => Ok(()),
        AtpFormulaTree::Atom(atom) => {
            require_symbol_signature(
                atom.predicate(),
                symbol_map,
                declarations,
                AtpDeclarationKind::Predicate,
                atom.arguments().len() as u32,
                "predicate",
            )?;
            for argument in atom.arguments() {
                validate_term(argument, symbol_map, declarations)?;
            }
            Ok(())
        }
        AtpFormulaTree::Equality { left, right } => {
            if profile.equality() != EqualitySupport::Supported {
                return Err(AtpProblemError::UnsupportedProfileFeature {
                    feature: "equality",
                }
                .into());
            }
            validate_term(left, symbol_map, declarations)?;
            validate_term(right, symbol_map, declarations)
        }
        AtpFormulaTree::Not(inner) => {
            validate_formula_tree(inner, profile, symbol_map, declarations)
        }
        AtpFormulaTree::And(formulas) | AtpFormulaTree::Or(formulas) => {
            if formulas.is_empty() {
                return Err(AtpProblemError::MissingFormulaPayload {
                    formula_id: crate::problem::AtpFormulaId::new(0),
                }
                .into());
            }
            for inner in formulas {
                validate_formula_tree(inner, profile, symbol_map, declarations)?;
            }
            Ok(())
        }
        AtpFormulaTree::Implies(left, right) => {
            validate_formula_tree(left, profile, symbol_map, declarations)?;
            validate_formula_tree(right, profile, symbol_map, declarations)
        }
        AtpFormulaTree::Forall { binders, body } | AtpFormulaTree::Exists { binders, body } => {
            if profile.quantifiers() != QuantifierPolicy::FirstOrder {
                return Err(AtpProblemError::UnsupportedProfileFeature {
                    feature: "quantifier",
                }
                .into());
            }
            if binders.is_empty() {
                return Err(AtpProblemError::MissingFormulaPayload {
                    formula_id: crate::problem::AtpFormulaId::new(0),
                }
                .into());
            }
            for binder in binders {
                require_symbol_signature(
                    binder.variable(),
                    symbol_map,
                    declarations,
                    AtpDeclarationKind::GeneratedBinder,
                    0,
                    "generated binder",
                )?;
                if let Some(sort) = binder.sort() {
                    require_symbol_signature(
                        sort,
                        symbol_map,
                        declarations,
                        AtpDeclarationKind::Sort,
                        0,
                        "sort",
                    )?;
                }
            }
            validate_formula_tree(body, profile, symbol_map, declarations)
        }
    }
}

fn validate_term(
    term: &AtpTerm,
    symbol_map: &BTreeSet<AtpSymbolName>,
    declarations: &BTreeMap<AtpSymbolName, DeclarationSignature>,
) -> Result<(), AtpTranslationError> {
    match term {
        AtpTerm::Variable(variable) => require_symbol_signature(
            variable,
            symbol_map,
            declarations,
            AtpDeclarationKind::GeneratedBinder,
            0,
            "generated binder",
        ),
        AtpTerm::Function {
            function,
            arguments,
        } => {
            require_symbol_signature(
                function,
                symbol_map,
                declarations,
                AtpDeclarationKind::Function,
                arguments.len() as u32,
                "function",
            )?;
            for argument in arguments {
                validate_term(argument, symbol_map, declarations)?;
            }
            Ok(())
        }
    }
}

fn require_symbol_signature(
    symbol: &AtpSymbolName,
    symbol_map: &BTreeSet<AtpSymbolName>,
    declarations: &BTreeMap<AtpSymbolName, DeclarationSignature>,
    expected_kind: AtpDeclarationKind,
    expected_arity: u32,
    expected_label: &'static str,
) -> Result<(), AtpTranslationError> {
    require_symbol(symbol, symbol_map)?;
    let Some(signature) = declarations.get(symbol) else {
        return Err(AtpProblemError::MissingDeclarationSymbol {
            symbol: symbol.clone(),
        }
        .into());
    };
    if signature.kind != expected_kind {
        return Err(AtpProblemError::InvalidSymbolDeclaration {
            symbol: symbol.clone(),
            expected: expected_label,
            actual: signature.kind,
        }
        .into());
    }
    if signature.arity != expected_arity {
        return Err(AtpProblemError::InvalidSymbolArity {
            symbol: symbol.clone(),
            expected: expected_arity,
            actual: signature.arity,
        }
        .into());
    }
    Ok(())
}

fn require_symbol(
    symbol: &AtpSymbolName,
    symbol_map: &BTreeSet<AtpSymbolName>,
) -> Result<(), AtpTranslationError> {
    if symbol_map.contains(symbol) {
        Ok(())
    } else {
        Err(AtpProblemError::MissingSymbolMap {
            symbol: symbol.clone(),
        }
        .into())
    }
}

fn require_provenance(
    owner: &'static str,
    provenance_id: AtpProvenanceId,
    provenance: &BTreeSet<AtpProvenanceId>,
) -> Result<(), AtpTranslationError> {
    if provenance.contains(&provenance_id) {
        Ok(())
    } else {
        Err(AtpProblemError::MissingProvenance {
            owner,
            provenance_id,
        }
        .into())
    }
}

fn target_binding(
    handoff: &VcKernelEvidenceHandoff,
) -> Result<AtpTargetBinding, AtpTranslationError> {
    let target = handoff.canonical_evidence().target_vc();
    let fingerprint = AtpFingerprint::new(target.algorithm_id, target.digest.clone())?;
    AtpTargetBinding::new(
        fingerprint,
        format!(
            "mizar-vc-kernel-evidence-handoff:{}",
            hex(handoff.canonical_hash().as_bytes())
        ),
    )
    .map_err(Into::into)
}

fn soft_type_strategy_requires_guards(
    strategy: SoftTypeStrategy,
) -> Result<bool, AtpTranslationError> {
    match strategy {
        SoftTypeStrategy::BackendSorts => Ok(false),
        SoftTypeStrategy::GuardPredicates | SoftTypeStrategy::SortsAndGuards => Ok(true),
    }
}

fn next_provenance_id(current_len: usize) -> Result<AtpProvenanceId, AtpTranslationError> {
    Ok(AtpProvenanceId::new(index_to_u32(
        current_len,
        "provenance",
    )?))
}

fn offset_provenance_id(
    first: AtpProvenanceId,
    offset: usize,
) -> Result<AtpProvenanceId, AtpTranslationError> {
    let id = first
        .index()
        .checked_add(index_to_u32(offset, "provenance")?)
        .ok_or(AtpTranslationError::IdOverflow {
            section: "provenance",
        })?;
    Ok(AtpProvenanceId::new(id))
}

fn index_to_u32(index: usize, section: &'static str) -> Result<u32, AtpTranslationError> {
    u32::try_from(index).map_err(|_| AtpTranslationError::IdOverflow { section })
}

fn hex(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::problem::{
        AtpAtom, AtpBinder, ConcreteFormat, EqualitySupport, LogicFragment, NativePropertySupport,
        QuantifierPolicy,
    };
    use mizar_core::{
        control_flow::ObligationHandoffId,
        core_ir::{
            CoreItemId, CoreLabelRef, CoreProvenance, CoreProvenancePhase, CoreSourceRef,
            LocalProofOrProgramPath, NormalizedSemanticOrigin, ObligationSeedId,
            ObligationSeedStatus,
        },
    };
    use mizar_session::{
        BuildSnapshotId, InMemorySessionIdAllocator, SessionIdAllocator, SourceId, SourceRange,
    };
    use mizar_vc::{
        kernel_evidence_handoff::{
            KERNEL_FORMULA_FINGERPRINT_ALGORITHM_ID, KernelClauseTautologyPolicy,
            KernelEvidenceFingerprint, KernelEvidenceHandoffInput, KernelEvidenceProfile,
            KernelFormulaContextRequirements, KernelFormulaPayload, KernelFormulaProjection,
            KernelImportedFactRequirement, KernelImportedFormulaClass,
            KernelImportedFormulaPayload, KernelRequiredProofStatus, build_kernel_evidence_handoff,
        },
        vc_ir::{
            AnchorCompleteness, AnchorIngredient, AnchorLabel, AnchorLabelRole, AnchorOwner,
            AnchorUnavailableReason, CanonicalSortKey, ContextEntry, ContextEntryId,
            ContextEntryKind, GenerationSchemaVersion, HashMarker, LocalContext, PremiseRef,
            PremiseRestriction, ProofHint, SeedAccounting, SeedOriginRef, SeedVcMapping,
            VcFormulaRef, VcGeneratedFormula, VcGeneratedFormulaId, VcGeneratedFormulaKind,
            VcGeneratedFormulaShape, VcIr, VcKind, VcModuleRef, VcProvenance, VcProvenancePhase,
            VcSchemaVersion, VcSet, VcSetParts, VcSourceRef, VcText,
        },
    };
    use std::collections::BTreeSet;

    #[test]
    fn translates_declarations_in_projection_key_order() {
        let set = fixture_set(VcStatus::NeedsAtp, "sample");
        let handoff = handoff(&set);
        let logic_profile = profile(SoftTypeStrategy::BackendSorts);
        let output = translate_declarations(AtpDeclarationTranslationInput {
            vc_set: &set,
            vc: VcId::new(0),
            kernel_handoff: &handoff,
            logic_profile: logic_profile.clone(),
            declaration_projections: vec![
                declaration_projection("zeta", "z", AtpDeclarationKind::Function, 0),
                declaration_projection("alpha", "a", AtpDeclarationKind::Predicate, 1),
            ],
            soft_type_projections: vec![],
            diagnostics: vec![AtpDiagnostic::new("note", "deterministic")],
        })
        .expect("translation");

        assert_eq!(output.vc_id(), VcId::new(0));
        assert_eq!(
            output
                .declarations()
                .iter()
                .map(|declaration| (declaration.id().index(), declaration.symbol().as_str()))
                .collect::<Vec<_>>(),
            [(0, "a"), (1, "z")]
        );
        assert_eq!(
            output
                .symbol_map()
                .iter()
                .map(|entry| entry.backend_symbol().as_str())
                .collect::<Vec<_>>(),
            ["a", "z"]
        );
        assert_eq!(output.logic_profile(), &logic_profile);
        assert!(
            output
                .target_binding()
                .producer_binding()
                .as_str()
                .contains("mizar-vc-kernel-evidence-handoff:")
        );
        assert_eq!(output.diagnostics().len(), 1);
    }

    #[test]
    fn shuffled_equivalent_projection_inputs_have_identical_outputs() {
        let set = fixture_set(VcStatus::NeedsAtp, "sample");
        let handoff = handoff(&set);
        let mut first = empty_input(&set, &handoff);
        first.declaration_projections = vec![
            declaration_projection("beta", "b", AtpDeclarationKind::Function, 0),
            declaration_projection("alpha", "a", AtpDeclarationKind::Predicate, 1),
        ];
        let mut second = empty_input(&set, &handoff);
        second.declaration_projections = vec![
            declaration_projection("alpha", "a", AtpDeclarationKind::Predicate, 1),
            declaration_projection("beta", "b", AtpDeclarationKind::Function, 0),
        ];

        assert_eq!(
            translate_declarations(first).expect("first"),
            translate_declarations(second).expect("second")
        );
    }

    #[test]
    fn shuffled_equivalent_soft_type_inputs_have_identical_outputs() {
        let set = fixture_set(VcStatus::NeedsAtp, "sample");
        let handoff = handoff(&set);
        let mut first = empty_input(&set, &handoff);
        first.logic_profile = profile(SoftTypeStrategy::GuardPredicates);
        first.declaration_projections = vec![
            declaration_projection("pred-a", "a", AtpDeclarationKind::Predicate, 0),
            declaration_projection("pred-b", "b", AtpDeclarationKind::Predicate, 0),
        ];
        first.soft_type_projections = vec![
            soft_guard(
                "z-guard",
                AtpFormulaTree::Atom(AtpAtom::new("b", Vec::new())),
            ),
            soft_guard(
                "a-guard",
                AtpFormulaTree::Atom(AtpAtom::new("a", Vec::new())),
            ),
        ];
        let mut second = empty_input(&set, &handoff);
        second.logic_profile = profile(SoftTypeStrategy::GuardPredicates);
        second.declaration_projections = vec![
            declaration_projection("pred-b", "b", AtpDeclarationKind::Predicate, 0),
            declaration_projection("pred-a", "a", AtpDeclarationKind::Predicate, 0),
        ];
        second.soft_type_projections = vec![
            soft_guard(
                "a-guard",
                AtpFormulaTree::Atom(AtpAtom::new("a", Vec::new())),
            ),
            soft_guard(
                "z-guard",
                AtpFormulaTree::Atom(AtpAtom::new("b", Vec::new())),
            ),
        ];

        assert_eq!(
            translate_declarations(first).expect("first"),
            translate_declarations(second).expect("second")
        );
    }

    #[test]
    fn rejects_non_needs_atp_vcs() {
        let set = fixture_set(VcStatus::Open, "sample");
        let handoff = handoff(&set);
        let error =
            translate_declarations(empty_input(&set, &handoff)).expect_err("non-NeedsAtp status");

        assert!(matches!(
            error,
            AtpTranslationError::NonNeedsAtpStatus {
                status: VcStatus::Open,
                ..
            }
        ));
    }

    #[test]
    fn rejects_mismatched_handoff_targets() {
        let handoff_set = fixture_set(VcStatus::NeedsAtp, "left");
        let input_set = fixture_set(VcStatus::NeedsAtp, "right");
        let handoff = handoff(&handoff_set);
        let error = translate_declarations(empty_input(&input_set, &handoff))
            .expect_err("mismatched target");

        assert!(matches!(
            error,
            AtpTranslationError::MismatchedTargetHandoff { vc } if vc == VcId::new(0)
        ));
    }

    #[test]
    fn duplicate_projection_keys_fail_closed() {
        let set = fixture_set(VcStatus::NeedsAtp, "sample");
        let handoff = handoff(&set);
        let mut input = empty_input(&set, &handoff);
        input.declaration_projections = vec![
            declaration_projection("dup", "p", AtpDeclarationKind::Predicate, 0),
            declaration_projection("dup", "q", AtpDeclarationKind::Predicate, 0),
        ];
        let error = translate_declarations(input).expect_err("duplicate key");

        assert!(matches!(
            error,
            AtpTranslationError::DuplicateProjectionKey {
                section: "declaration projections",
                ..
            }
        ));
    }

    #[test]
    fn malformed_projection_inputs_fail_closed() {
        let set = fixture_set(VcStatus::NeedsAtp, "sample");
        let handoff = handoff(&set);
        let mut empty_key = empty_input(&set, &handoff);
        empty_key.declaration_projections = vec![declaration_projection(
            "",
            "p",
            AtpDeclarationKind::Predicate,
            0,
        )];
        assert!(matches!(
            translate_declarations(empty_key).expect_err("empty projection key"),
            AtpTranslationError::EmptyProjectionKey {
                section: "declaration projections"
            }
        ));

        let mut empty_payload = empty_input(&set, &handoff);
        let mut projection = declaration_projection("p", "p", AtpDeclarationKind::Predicate, 0);
        projection.provenance.payload = AtpPayload::new("");
        empty_payload.declaration_projections = vec![projection];
        assert!(matches!(
            translate_declarations(empty_payload).expect_err("empty provenance payload"),
            AtpTranslationError::Problem {
                source: AtpProblemError::EmptyField {
                    field: "projection.provenance.payload"
                }
            }
        ));

        let mut empty_source = empty_input(&set, &handoff);
        let mut projection = declaration_projection("q", "q", AtpDeclarationKind::Predicate, 0);
        projection.provenance.source = AtpSourceRef::GeneratedVcFact(AtpSourceBinding::new(""));
        empty_source.declaration_projections = vec![projection];
        assert!(matches!(
            translate_declarations(empty_source).expect_err("empty provenance source"),
            AtpTranslationError::Problem {
                source: AtpProblemError::EmptyField {
                    field: "projection.provenance.source"
                }
            }
        ));
    }

    #[test]
    fn malformed_soft_type_projection_inputs_fail_closed() {
        let set = fixture_set(VcStatus::NeedsAtp, "sample");
        let handoff = handoff(&set);
        let mut empty_key = empty_input(&set, &handoff);
        empty_key.soft_type_projections = vec![soft_guard("", AtpFormulaTree::True)];
        assert!(matches!(
            translate_declarations(empty_key).expect_err("empty soft-type projection key"),
            AtpTranslationError::EmptyProjectionKey {
                section: "soft-type projections"
            }
        ));

        let mut empty_payload = empty_input(&set, &handoff);
        let mut projection = soft_guard("guard", AtpFormulaTree::True);
        projection.provenance.payload = AtpPayload::new("");
        empty_payload.soft_type_projections = vec![projection];
        assert!(matches!(
            translate_declarations(empty_payload).expect_err("empty soft-type provenance payload"),
            AtpTranslationError::Problem {
                source: AtpProblemError::EmptyField {
                    field: "projection.provenance.payload"
                }
            }
        ));

        let mut empty_source = empty_input(&set, &handoff);
        let mut projection = soft_guard("guard", AtpFormulaTree::True);
        projection.provenance.source = AtpSourceRef::TypeFact(AtpSourceBinding::new(""));
        empty_source.soft_type_projections = vec![projection];
        assert!(matches!(
            translate_declarations(empty_source).expect_err("empty soft-type provenance source"),
            AtpTranslationError::Problem {
                source: AtpProblemError::EmptyField {
                    field: "projection.provenance.source"
                }
            }
        ));
    }

    #[test]
    fn missing_type_guard_projection_fails_closed() {
        let set = fixture_set(VcStatus::NeedsAtp, "sample");
        let handoff = handoff(&set);
        let mut input = empty_input(&set, &handoff);
        input.declaration_projections = vec![AtpDeclarationProjection {
            key: AtpProjectionKey::new("type-predicate"),
            kind: AtpDeclarationKind::Predicate,
            symbol: AtpSymbolName::new("type_guard"),
            arity: 0,
            provenance: declaration_provenance("type-predicate"),
            symbol_source: AtpSymbolSourceProjection::TypeGuard(AtpProjectionKey::new("missing")),
        }];
        let error = translate_declarations(input).expect_err("missing type guard projection");

        assert!(matches!(
            error,
            AtpTranslationError::MissingTypeGuardProjection { .. }
        ));
    }

    #[test]
    fn duplicate_symbols_and_missing_guard_symbols_fail_closed() {
        let set = fixture_set(VcStatus::NeedsAtp, "sample");
        let handoff = handoff(&set);
        let mut duplicate_symbol = empty_input(&set, &handoff);
        duplicate_symbol.declaration_projections = vec![
            declaration_projection("alpha", "p", AtpDeclarationKind::Predicate, 0),
            declaration_projection("beta", "p", AtpDeclarationKind::Predicate, 0),
        ];
        assert!(matches!(
            translate_declarations(duplicate_symbol).expect_err("duplicate declaration symbol"),
            AtpTranslationError::Problem {
                source: AtpProblemError::DuplicateSymbolMap { .. }
            }
        ));

        let mut missing_symbol = empty_input(&set, &handoff);
        missing_symbol.logic_profile = profile(SoftTypeStrategy::GuardPredicates);
        missing_symbol.soft_type_projections = vec![soft_guard(
            "guard",
            AtpFormulaTree::Atom(AtpAtom::new("missing", Vec::new())),
        )];
        assert!(matches!(
            translate_declarations(missing_symbol).expect_err("missing guard symbol"),
            AtpTranslationError::Problem {
                source: AtpProblemError::MissingSymbolMap { .. }
            }
        ));
    }

    #[test]
    fn kind_and_arity_mismatches_are_rejected_by_translator_signature_validation() {
        let set = fixture_set(VcStatus::NeedsAtp, "sample");
        let handoff = handoff(&set);
        let mut input = empty_input(&set, &handoff);
        input.logic_profile = profile(SoftTypeStrategy::GuardPredicates);
        input.declaration_projections = vec![declaration_projection(
            "pred-p",
            "p",
            AtpDeclarationKind::Predicate,
            1,
        )];
        input.soft_type_projections = vec![soft_guard(
            "guard",
            AtpFormulaTree::Atom(AtpAtom::new("p", Vec::new())),
        )];
        let error = translate_declarations(input).expect_err("arity mismatch");

        assert!(matches!(
            error,
            AtpTranslationError::Problem {
                source: AtpProblemError::InvalidSymbolArity { .. }
            }
        ));
    }

    #[test]
    fn kind_mismatches_are_rejected_by_translator_signature_validation() {
        let set = fixture_set(VcStatus::NeedsAtp, "sample");
        let handoff = handoff(&set);
        let mut input = empty_input(&set, &handoff);
        input.logic_profile = profile(SoftTypeStrategy::GuardPredicates);
        input.declaration_projections = vec![declaration_projection(
            "fn-f",
            "f",
            AtpDeclarationKind::Function,
            0,
        )];
        input.soft_type_projections = vec![soft_guard(
            "guard",
            AtpFormulaTree::Atom(AtpAtom::new("f", Vec::new())),
        )];
        let error = translate_declarations(input).expect_err("kind mismatch");

        assert!(matches!(
            error,
            AtpTranslationError::Problem {
                source: AtpProblemError::InvalidSymbolDeclaration { .. }
            }
        ));
    }

    #[test]
    fn guard_profiles_do_not_accept_sort_only_soft_type_projection() {
        let set = fixture_set(VcStatus::NeedsAtp, "sample");
        let handoff = handoff(&set);
        let mut input = empty_input(&set, &handoff);
        input.logic_profile = profile(SoftTypeStrategy::GuardPredicates);
        input.soft_type_projections = vec![AtpSoftTypeProjection {
            key: AtpProjectionKey::new("type-fact"),
            representation: AtpSoftTypeRepresentation::BackendSortLossless,
            provenance: type_provenance("type-fact"),
        }];
        let error = translate_declarations(input).expect_err("missing guard");

        assert!(matches!(
            error,
            AtpTranslationError::MissingSoftTypeGuard { .. }
        ));
    }

    #[test]
    fn explicit_profile_is_not_changed_to_accept_quantified_guard() {
        let set = fixture_set(VcStatus::NeedsAtp, "sample");
        let handoff = handoff(&set);
        let mut input = empty_input(&set, &handoff);
        input.logic_profile = propositional_guard_profile();
        input.declaration_projections = vec![declaration_projection(
            "binder-x",
            "x",
            AtpDeclarationKind::GeneratedBinder,
            0,
        )];
        input.soft_type_projections = vec![soft_guard(
            "guard",
            AtpFormulaTree::Forall {
                binders: vec![AtpBinder::new("x", None)],
                body: Box::new(AtpFormulaTree::True),
            },
        )];
        let error = translate_declarations(input).expect_err("unsupported quantifier");

        assert!(matches!(
            error,
            AtpTranslationError::Problem {
                source: AtpProblemError::UnsupportedProfileFeature {
                    feature: "quantifier"
                }
            }
        ));
    }

    #[test]
    fn type_guard_symbol_sources_resolve_to_generated_guard_ids() {
        let set = fixture_set(VcStatus::NeedsAtp, "sample");
        let handoff = handoff(&set);
        let mut input = empty_input(&set, &handoff);
        input.logic_profile = profile(SoftTypeStrategy::GuardPredicates);
        input.declaration_projections = vec![AtpDeclarationProjection {
            key: AtpProjectionKey::new("type-predicate"),
            kind: AtpDeclarationKind::Predicate,
            symbol: AtpSymbolName::new("type_guard"),
            arity: 0,
            provenance: declaration_provenance("type-predicate"),
            symbol_source: AtpSymbolSourceProjection::TypeGuard(AtpProjectionKey::new("guard")),
        }];
        input.soft_type_projections = vec![soft_guard(
            "guard",
            AtpFormulaTree::Atom(AtpAtom::new("type_guard", Vec::new())),
        )];
        let output = translate_declarations(input).expect("translation");

        assert!(matches!(
            output.symbol_map()[0].source(),
            AtpSymbolSource::TypeGuard(id) if *id == AtpTypeGuardId::new(0)
        ));
        assert_eq!(output.type_context().guards().len(), 1);
        assert_eq!(
            output.type_context().guards()[0].formula(),
            &AtpFormulaTree::Atom(AtpAtom::new("type_guard", Vec::new()))
        );
        assert_eq!(
            output.provenance()[0].source(),
            &AtpSourceRef::TypeFact(AtpSourceBinding::new("type:guard"))
        );
        assert_eq!(
            output.provenance()[0].payload().as_str(),
            "type-payload:guard"
        );
    }

    #[test]
    fn imported_projection_missing_required_status_fails_closed() {
        let set = fixture_set(VcStatus::NeedsAtp, "sample");
        let handoff = handoff(&set);
        let mut input = empty_input(&set, &handoff);
        input.declaration_projections = vec![AtpDeclarationProjection {
            key: AtpProjectionKey::new("imported"),
            kind: AtpDeclarationKind::Predicate,
            symbol: AtpSymbolName::new("imported"),
            arity: 0,
            provenance: AtpProjectionProvenance::new(
                AtpSourceRef::ImportedAxiom {
                    package: AtpSourceBinding::new("pkg"),
                    module: AtpSourceBinding::new("module"),
                    item: AtpSourceBinding::new("item"),
                    statement_fingerprint: AtpFingerprint::new(2, vec![1]).expect("fingerprint"),
                    required_status: AtpRequiredProofStatus::new(""),
                    context_requirement: AtpSourceBinding::new("ctx"),
                },
                "imported",
            ),
            symbol_source: AtpSymbolSourceProjection::MizarSymbol(AtpSourceBinding::new(
                "imported",
            )),
        }];
        let error = translate_declarations(input).expect_err("empty imported status");

        assert!(matches!(
            error,
            AtpTranslationError::Problem {
                source: AtpProblemError::EmptyField {
                    field: "imported.required_status"
                }
            }
        ));
    }

    #[test]
    fn imported_projection_missing_context_requirement_fails_closed() {
        let set = fixture_set(VcStatus::NeedsAtp, "sample");
        let handoff = handoff(&set);
        let mut input = empty_input(&set, &handoff);
        input.declaration_projections = vec![AtpDeclarationProjection {
            key: AtpProjectionKey::new("imported"),
            kind: AtpDeclarationKind::Predicate,
            symbol: AtpSymbolName::new("imported"),
            arity: 0,
            provenance: AtpProjectionProvenance::new(
                AtpSourceRef::ImportedAxiom {
                    package: AtpSourceBinding::new("pkg"),
                    module: AtpSourceBinding::new("module"),
                    item: AtpSourceBinding::new("item"),
                    statement_fingerprint: AtpFingerprint::new(2, vec![1]).expect("fingerprint"),
                    required_status: AtpRequiredProofStatus::new("translator-fixture-verified"),
                    context_requirement: AtpSourceBinding::new(""),
                },
                "imported",
            ),
            symbol_source: AtpSymbolSourceProjection::MizarSymbol(AtpSourceBinding::new(
                "imported",
            )),
        }];
        let error = translate_declarations(input).expect_err("empty imported context");

        assert!(matches!(
            error,
            AtpTranslationError::Problem {
                source: AtpProblemError::EmptyField {
                    field: "imported.context_requirement"
                }
            }
        ));
    }

    #[test]
    fn translates_axioms_and_conjecture_with_unsat_polarity() {
        let set = fixture_set(VcStatus::NeedsAtp, "sample");
        let hinted_handoff = handoff(&set);
        let mut input = basic_problem_input(&set, &hinted_handoff);
        input.formula_projections[0].provenance.payload =
            AtpPayload::new("caller-selected-axiom-provenance");
        input.formula_projections[1].provenance.payload =
            AtpPayload::new("caller-selected-goal-provenance");
        let problem = translate_problem(input).expect("problem");

        assert_eq!(problem.vc_id(), VcId::new(0));
        assert_eq!(problem.expected_result(), ExpectedBackendResult::Unsat);
        assert_eq!(problem.axioms().len(), 1);
        assert_eq!(
            problem.axioms()[0].formula(),
            Some(&AtpFormulaTree::Atom(AtpAtom::new("p", Vec::new())))
        );
        assert_eq!(problem.conjecture().formula(), Some(&AtpFormulaTree::False));
        assert_eq!(problem.properties(), []);
        assert_eq!(
            problem.provenance()[1].payload().as_str(),
            "mizar-vc-handoff-provenance:70726f76656e616e63652d30"
        );
        assert_eq!(
            problem.provenance()[2].payload().as_str(),
            "mizar-vc-handoff-provenance:70726f76656e616e63652d31"
        );
        assert!(problem.debug_text().contains("expected-result: Unsat"));
    }

    #[test]
    fn shuffled_formula_projection_inputs_have_identical_problem_output() {
        let set = fixture_set(VcStatus::NeedsAtp, "sample");
        let handoff = handoff(&set);
        let first = basic_problem_input(&set, &handoff);
        let mut second = basic_problem_input(&set, &handoff);
        second.formula_projections.reverse();

        assert_eq!(
            translate_problem(first).expect("first").debug_text(),
            translate_problem(second).expect("second").debug_text()
        );
    }

    #[test]
    fn translate_problem_rejects_non_needs_atp_and_mismatched_handoff() {
        let open_set = fixture_set(VcStatus::Open, "sample");
        let open_handoff = handoff(&open_set);
        assert!(matches!(
            translate_problem(basic_problem_input(&open_set, &open_handoff))
                .expect_err("non-NeedsAtp"),
            AtpTranslationError::NonNeedsAtpStatus {
                status: VcStatus::Open,
                ..
            }
        ));

        let handoff_set = fixture_set(VcStatus::NeedsAtp, "left");
        let input_set = fixture_set(VcStatus::NeedsAtp, "right");
        let stale_handoff = handoff(&handoff_set);
        assert!(matches!(
            translate_problem(basic_problem_input(&input_set, &stale_handoff))
                .expect_err("stale handoff"),
            AtpTranslationError::MismatchedTargetHandoff { .. }
        ));
    }

    #[test]
    fn missing_malformed_and_duplicate_formula_projections_fail_closed() {
        let set = fixture_set(VcStatus::NeedsAtp, "sample");
        let handoff = handoff(&set);
        let mut missing_goal = basic_problem_input(&set, &handoff);
        missing_goal.formula_projections.pop();
        assert!(matches!(
            translate_problem(missing_goal).expect_err("missing goal projection"),
            AtpTranslationError::MissingFormulaProjection { .. }
        ));

        let mut duplicate_target = basic_problem_input(&set, &handoff);
        let mut duplicate_projection = local_formula_projection(AtpFormulaTree::True);
        duplicate_projection.source_identity = AtpProjectionKey::new("local-context:duplicate");
        duplicate_target
            .formula_projections
            .push(duplicate_projection);
        assert!(matches!(
            translate_problem(duplicate_target).expect_err("duplicate target"),
            AtpTranslationError::DuplicateFormulaProjection { .. }
        ));

        let mut duplicate_identity = basic_problem_input(&set, &handoff);
        duplicate_identity.formula_projections[1].source_identity = duplicate_identity
            .formula_projections[0]
            .source_identity
            .clone();
        assert!(matches!(
            translate_problem(duplicate_identity).expect_err("duplicate source identity"),
            AtpTranslationError::DuplicateFormulaProjectionIdentity { .. }
        ));

        let mut empty_handoff_payload = basic_problem_input(&set, &handoff);
        empty_handoff_payload.formula_projections[0]
            .handoff_provenance_payload
            .clear();
        assert!(matches!(
            translate_problem(empty_handoff_payload).expect_err("empty handoff payload"),
            AtpTranslationError::Problem {
                source: AtpProblemError::EmptyField {
                    field: "formula_projection.handoff_provenance_payload"
                }
            }
        ));

        let mut empty_identity = basic_problem_input(&set, &handoff);
        empty_identity.formula_projections[0].source_identity = AtpProjectionKey::new("");
        assert!(matches!(
            translate_problem(empty_identity).expect_err("empty source identity"),
            AtpTranslationError::EmptyProjectionKey {
                section: "formula projection source identities"
            }
        ));

        let mut empty_provenance_payload = basic_problem_input(&set, &handoff);
        empty_provenance_payload.formula_projections[0]
            .provenance
            .payload = AtpPayload::new("");
        assert!(matches!(
            translate_problem(empty_provenance_payload)
                .expect_err("empty formula provenance payload"),
            AtpTranslationError::Problem {
                source: AtpProblemError::EmptyField {
                    field: "projection.provenance.payload"
                }
            }
        ));

        let mut empty_provenance_source = basic_problem_input(&set, &handoff);
        empty_provenance_source.formula_projections[0]
            .provenance
            .source = AtpSourceRef::LocalHypothesis(AtpSourceBinding::new(""));
        assert!(matches!(
            translate_problem(empty_provenance_source)
                .expect_err("empty formula provenance source"),
            AtpTranslationError::Problem {
                source: AtpProblemError::EmptyField {
                    field: "projection.provenance.source"
                }
            }
        ));

        let imported_set = fixture_set_with(
            VcStatus::NeedsAtp,
            "empty-imported-symbol",
            vec![PremiseRef::ImportedFact {
                symbol: VcText::new("Imported::A1"),
            }],
            None,
        );
        let imported_handoff = handoff_with_imported(&imported_set);
        let mut empty_symbol = imported_problem_input(&imported_set, &imported_handoff);
        empty_symbol.formula_projections[0].target =
            AtpFormulaProjectionTarget::ImportedFact(VcText::new(""));
        assert!(matches!(
            translate_problem(empty_symbol).expect_err("empty imported symbol"),
            AtpTranslationError::Problem {
                source: AtpProblemError::EmptyField {
                    field: "formula_projection.imported_symbol"
                }
            }
        ));
    }

    #[test]
    fn formula_handoff_fingerprint_mismatch_fails_closed() {
        let set = fixture_set(VcStatus::NeedsAtp, "sample");
        let base_handoff = handoff(&set);
        let mut input = basic_problem_input(&set, &base_handoff);
        input.formula_projections[0].handoff_formula_fingerprint =
            AtpFingerprint::new(KERNEL_FORMULA_FINGERPRINT_ALGORITHM_ID, b"wrong".to_vec())
                .expect("fingerprint");

        assert!(matches!(
            translate_problem(input).expect_err("handoff mismatch"),
            AtpTranslationError::FormulaHandoffAgreement { .. }
        ));

        let mut payload_mismatch = basic_problem_input(&set, &base_handoff);
        payload_mismatch.formula_projections[0].handoff_provenance_payload =
            b"wrong-provenance".to_vec();
        assert!(matches!(
            translate_problem(payload_mismatch).expect_err("handoff payload mismatch"),
            AtpTranslationError::FormulaHandoffAgreement { .. }
        ));

        let mut wrong_local_source = basic_problem_input(&set, &base_handoff);
        wrong_local_source.formula_projections[0].provenance.source =
            AtpSourceRef::LocalHypothesis(AtpSourceBinding::new("local-context:99"));
        assert!(matches!(
            translate_problem(wrong_local_source).expect_err("wrong local source binding"),
            AtpTranslationError::Problem {
                source: AtpProblemError::UnsupportedProfileFeature {
                    feature: "formula provenance source class"
                }
            }
        ));

        let mut wrong_goal_source_class = basic_problem_input(&set, &base_handoff);
        wrong_goal_source_class.formula_projections[1]
            .provenance
            .source = AtpSourceRef::LocalHypothesis(AtpSourceBinding::new("goal:1"));
        assert!(matches!(
            translate_problem(wrong_goal_source_class).expect_err("wrong goal source class"),
            AtpTranslationError::Problem {
                source: AtpProblemError::UnsupportedProfileFeature {
                    feature: "conjecture provenance source class"
                }
            }
        ));

        let mut checker_goal_source = basic_problem_input(&set, &base_handoff);
        checker_goal_source.formula_projections[1].provenance.source =
            AtpSourceRef::CheckerOwnedFact(AtpSourceBinding::new("goal:1"));
        assert!(matches!(
            translate_problem(checker_goal_source).expect_err("checker-owned goal source"),
            AtpTranslationError::Problem {
                source: AtpProblemError::UnsupportedProfileFeature {
                    feature: "conjecture provenance source class"
                }
            }
        ));

        let mut wrong_goal_binding = basic_problem_input(&set, &base_handoff);
        wrong_goal_binding.formula_projections[1].provenance.source =
            AtpSourceRef::GeneratedVcFact(AtpSourceBinding::new("generated:2"));
        wrong_goal_binding.formula_projections[1].source_identity =
            AtpProjectionKey::new("generated:2");
        assert!(matches!(
            translate_problem(wrong_goal_binding).expect_err("wrong goal binding"),
            AtpTranslationError::Problem {
                source: AtpProblemError::UnsupportedProfileFeature {
                    feature: "conjecture provenance source class"
                }
            }
        ));

        let mut wrong_goal_identity = basic_problem_input(&set, &base_handoff);
        wrong_goal_identity.formula_projections[1].source_identity =
            AtpProjectionKey::new("goal:spoof");
        assert!(matches!(
            translate_problem(wrong_goal_identity).expect_err("wrong goal source identity"),
            AtpTranslationError::Problem {
                source: AtpProblemError::UnsupportedProfileFeature {
                    feature: "conjecture provenance source class"
                }
            }
        ));

        let mut wrong_goal_fingerprint = basic_problem_input(&set, &base_handoff);
        wrong_goal_fingerprint.formula_projections[1].handoff_formula_fingerprint =
            AtpFingerprint::new(
                KERNEL_FORMULA_FINGERPRINT_ALGORITHM_ID,
                b"wrong-goal".to_vec(),
            )
            .expect("fingerprint");
        assert!(matches!(
            translate_problem(wrong_goal_fingerprint).expect_err("wrong goal fingerprint"),
            AtpTranslationError::FormulaHandoffAgreement { .. }
        ));

        let mut wrong_goal_payload = basic_problem_input(&set, &base_handoff);
        wrong_goal_payload.formula_projections[1].handoff_provenance_payload =
            b"wrong-goal-provenance".to_vec();
        assert!(matches!(
            translate_problem(wrong_goal_payload).expect_err("wrong goal payload"),
            AtpTranslationError::FormulaHandoffAgreement { .. }
        ));

        let generated_set = fixture_set_with(
            VcStatus::NeedsAtp,
            "generated-source-mismatch",
            vec![
                PremiseRef::LocalContext(ContextEntryId::new(0)),
                PremiseRef::GeneratedFact {
                    formula: VcFormulaRef::Generated(VcGeneratedFormulaId::new(2)),
                },
            ],
            None,
        );
        let generated_handoff = handoff(&generated_set);
        let mut wrong_generated_source =
            two_premise_problem_input(&generated_set, &generated_handoff);
        wrong_generated_source.formula_projections[2]
            .provenance
            .source = AtpSourceRef::GeneratedVcFact(AtpSourceBinding::new("generated:99"));
        wrong_generated_source.formula_projections[2].source_identity =
            AtpProjectionKey::new("generated:99");
        assert!(matches!(
            translate_problem(wrong_generated_source).expect_err("wrong generated source"),
            AtpTranslationError::Problem {
                source: AtpProblemError::UnsupportedProfileFeature {
                    feature: "formula provenance source class"
                }
            }
        ));

        let cited_set = fixture_set_with_local_entry_kind(
            VcStatus::NeedsAtp,
            "cited-source-mismatch",
            vec![PremiseRef::LocalContext(ContextEntryId::new(0))],
            None,
            ContextEntryKind::CitedPremise,
        );
        let cited_handoff = handoff(&cited_set);
        let mut cited_input = basic_problem_input(&cited_set, &cited_handoff);
        cited_input.formula_projections[0] =
            cited_formula_projection(AtpFormulaTree::Atom(AtpAtom::new("p", Vec::new())));
        translate_problem(cited_input).expect("cited premise source");

        let mut wrong_cited_source = basic_problem_input(&cited_set, &cited_handoff);
        wrong_cited_source.formula_projections[0] =
            cited_formula_projection(AtpFormulaTree::Atom(AtpAtom::new("p", Vec::new())));
        wrong_cited_source.formula_projections[0].provenance.source =
            AtpSourceRef::CitedPremise(AtpSourceBinding::new("cited-premise:99"));
        wrong_cited_source.formula_projections[0].source_identity =
            AtpProjectionKey::new("cited-premise:99");
        assert!(matches!(
            translate_problem(wrong_cited_source).expect_err("wrong cited source"),
            AtpTranslationError::Problem {
                source: AtpProblemError::UnsupportedProfileFeature {
                    feature: "formula provenance source class"
                }
            }
        ));
    }

    #[test]
    fn unsupported_formula_profile_features_are_not_silently_reprofiled() {
        let set = fixture_set(VcStatus::NeedsAtp, "sample");
        let handoff = handoff(&set);
        let mut input = basic_problem_input(&set, &handoff);
        input.logic_profile = profile_without_equality();
        input.declaration_projections = vec![declaration_projection(
            "fn-a",
            "a",
            AtpDeclarationKind::Function,
            0,
        )];
        input.formula_projections[0].formula = AtpFormulaTree::Equality {
            left: AtpTerm::Function {
                function: AtpSymbolName::new("a"),
                arguments: Vec::new(),
            },
            right: AtpTerm::Function {
                function: AtpSymbolName::new("a"),
                arguments: Vec::new(),
            },
        };

        assert!(matches!(
            translate_problem(input).expect_err("unsupported equality"),
            AtpTranslationError::Problem {
                source: AtpProblemError::UnsupportedProfileFeature {
                    feature: "equality"
                }
            }
        ));
    }

    #[test]
    fn duplicate_premise_refs_and_formula_identities_fail_closed() {
        let duplicate_ref_set = fixture_set_with(
            VcStatus::NeedsAtp,
            "dup-ref",
            vec![
                PremiseRef::LocalContext(ContextEntryId::new(0)),
                PremiseRef::LocalContext(ContextEntryId::new(0)),
            ],
            None,
        );
        let duplicate_ref_handoff = handoff(&duplicate_ref_set);
        assert!(matches!(
            translate_problem(basic_problem_input(
                &duplicate_ref_set,
                &duplicate_ref_handoff
            ))
            .expect_err("duplicate premise ref"),
            AtpTranslationError::DuplicatePremiseRef { .. }
        ));

        let duplicate_formula_set = fixture_set_with(
            VcStatus::NeedsAtp,
            "dup-formula",
            vec![
                PremiseRef::LocalContext(ContextEntryId::new(0)),
                PremiseRef::GeneratedFact {
                    formula: VcFormulaRef::Generated(VcGeneratedFormulaId::new(0)),
                },
            ],
            None,
        );
        let duplicate_formula_handoff = handoff(&duplicate_formula_set);
        assert!(matches!(
            translate_problem(basic_problem_input(
                &duplicate_formula_set,
                &duplicate_formula_handoff
            ))
            .expect_err("duplicate formula ref"),
            AtpTranslationError::DuplicatePremiseFormula { .. }
        ));
    }

    #[test]
    fn premise_formula_must_not_copy_goal_into_axioms() {
        let generated_goal_premise_set = fixture_set_with(
            VcStatus::NeedsAtp,
            "goal-generated-premise",
            vec![PremiseRef::GeneratedFact {
                formula: VcFormulaRef::Generated(VcGeneratedFormulaId::new(1)),
            }],
            None,
        );
        let generated_goal_handoff = handoff(&generated_goal_premise_set);
        assert!(matches!(
            translate_problem(basic_problem_input(
                &generated_goal_premise_set,
                &generated_goal_handoff
            ))
            .expect_err("goal copied as generated premise"),
            AtpTranslationError::PremiseCopiesGoal { .. }
        ));

        let local_goal_premise_set = fixture_set_with_local_formula(
            VcStatus::NeedsAtp,
            "goal-local-premise",
            vec![PremiseRef::LocalContext(ContextEntryId::new(0))],
            None,
            VcFormulaRef::Generated(VcGeneratedFormulaId::new(1)),
        );
        let local_goal_handoff = handoff(&local_goal_premise_set);
        assert!(matches!(
            translate_problem(basic_problem_input(
                &local_goal_premise_set,
                &local_goal_handoff
            ))
            .expect_err("goal copied as local premise"),
            AtpTranslationError::PremiseCopiesGoal { .. }
        ));
    }

    #[test]
    fn unsupported_checker_owned_and_type_predicate_premises_fail_closed() {
        for premise in [
            PremiseRef::CheckerFact {
                formula: mizar_core::core_ir::CoreFormulaId::new(0),
            },
            PremiseRef::TypePredicate {
                formula: mizar_core::core_ir::CoreFormulaId::new(0),
            },
        ] {
            let set = fixture_set(VcStatus::NeedsAtp, "unsupported");
            let vc = set.vc(VcId::new(0)).expect("vc");
            assert!(matches!(
                premise_projection_target(vc, &premise).expect_err("unsupported premise family"),
                AtpTranslationError::UnsupportedPremiseRef { .. }
            ));
        }

        let unsupported_set = fixture_set_with(
            VcStatus::NeedsAtp,
            "unsupported-public",
            vec![PremiseRef::CheckerFact {
                formula: mizar_core::core_ir::CoreFormulaId::new(0),
            }],
            None,
        );
        let valid_set = fixture_set(VcStatus::NeedsAtp, "unsupported-public");
        let valid_handoff = handoff(&valid_set);
        assert!(matches!(
            translate_problem(basic_problem_input(&unsupported_set, &valid_handoff))
                .expect_err("public translator boundary fails closed"),
            AtpTranslationError::MismatchedTargetHandoff { .. }
        ));
    }

    #[test]
    fn generated_fact_premises_materialize_and_premise_order_is_canonical() {
        let first = fixture_set_with(
            VcStatus::NeedsAtp,
            "generated-order",
            vec![
                PremiseRef::GeneratedFact {
                    formula: VcFormulaRef::Generated(VcGeneratedFormulaId::new(2)),
                },
                PremiseRef::LocalContext(ContextEntryId::new(0)),
            ],
            None,
        );
        let second = fixture_set_with(
            VcStatus::NeedsAtp,
            "generated-order",
            vec![
                PremiseRef::LocalContext(ContextEntryId::new(0)),
                PremiseRef::GeneratedFact {
                    formula: VcFormulaRef::Generated(VcGeneratedFormulaId::new(2)),
                },
            ],
            None,
        );
        let first_handoff = handoff(&first);
        let second_handoff = handoff(&second);
        let first_problem =
            translate_problem(two_premise_problem_input(&first, &first_handoff)).expect("first");
        let second_problem =
            translate_problem(two_premise_problem_input(&second, &second_handoff)).expect("second");

        assert_eq!(
            first_problem
                .axioms()
                .iter()
                .map(|axiom| axiom.formula().cloned())
                .collect::<Vec<_>>(),
            second_problem
                .axioms()
                .iter()
                .map(|axiom| axiom.formula().cloned())
                .collect::<Vec<_>>()
        );
        assert_eq!(first_problem.problem_id(), second_problem.problem_id());
        assert_eq!(first_problem.debug_text(), second_problem.debug_text());
        assert!(first_problem.provenance().iter().any(|provenance| {
            matches!(
                provenance.source(),
                AtpSourceRef::GeneratedVcFact(binding) if binding.as_str() == "generated:3"
            )
        }));
    }

    #[test]
    fn proof_hint_restrictions_do_not_prune_premises() {
        let set = fixture_set_with(
            VcStatus::NeedsAtp,
            "hinted",
            vec![PremiseRef::LocalContext(ContextEntryId::new(0))],
            Some(ProofHint {
                citations: vec![],
                unfold_requests: vec![],
                premise_restrictions: vec![PremiseRestriction::Exclude(vec![
                    PremiseRef::LocalContext(ContextEntryId::new(0)),
                ])],
                solver: None,
                max_axioms: None,
                timeout: None,
                computation: None,
                provenance: vec![vc_provenance("hint")],
            }),
        );
        let hinted_handoff = handoff(&set);
        let problem =
            translate_problem(basic_problem_input(&set, &hinted_handoff)).expect("problem");

        assert_eq!(problem.axioms().len(), 1);
        assert_eq!(
            problem.axioms()[0].formula(),
            Some(&AtpFormulaTree::Atom(AtpAtom::new("p", Vec::new())))
        );

        let only_set = fixture_set_with(
            VcStatus::NeedsAtp,
            "hinted-only",
            vec![PremiseRef::LocalContext(ContextEntryId::new(0))],
            Some(ProofHint {
                citations: vec![],
                unfold_requests: vec![],
                premise_restrictions: vec![PremiseRestriction::Only(Vec::new())],
                solver: None,
                max_axioms: None,
                timeout: None,
                computation: None,
                provenance: vec![vc_provenance("hint-only")],
            }),
        );
        let only_handoff = handoff(&only_set);
        let only_problem =
            translate_problem(basic_problem_input(&only_set, &only_handoff)).expect("problem");
        assert_eq!(only_problem.axioms().len(), 1);
    }

    #[test]
    fn imported_formula_projection_requires_imported_provenance_fields() {
        let set = fixture_set_with(
            VcStatus::NeedsAtp,
            "imported",
            vec![PremiseRef::ImportedFact {
                symbol: VcText::new("Imported::A1"),
            }],
            None,
        );
        let handoff = handoff_with_imported(&set);
        let mut input = imported_problem_input(&set, &handoff);
        if let AtpSourceRef::ImportedAxiom {
            required_status, ..
        } = &mut input.formula_projections[0].provenance.source
        {
            *required_status = AtpRequiredProofStatus::new("");
        }

        assert!(matches!(
            translate_problem(input).expect_err("missing imported status"),
            AtpTranslationError::Problem {
                source: AtpProblemError::EmptyField {
                    field: "imported.required_status"
                }
            }
        ));

        let mut wrong_package = imported_problem_input(&set, &handoff);
        if let AtpSourceRef::ImportedAxiom { package, .. } =
            &mut wrong_package.formula_projections[0].provenance.source
        {
            *package = AtpSourceBinding::new("other-pkg");
        }
        assert!(matches!(
            translate_problem(wrong_package).expect_err("wrong imported package"),
            AtpTranslationError::MissingFormulaHandoffEvidence { .. }
        ));

        let mut wrong_module = imported_problem_input(&set, &handoff);
        if let AtpSourceRef::ImportedAxiom { module, .. } =
            &mut wrong_module.formula_projections[0].provenance.source
        {
            *module = AtpSourceBinding::new("other-module");
        }
        assert!(matches!(
            translate_problem(wrong_module).expect_err("wrong imported module"),
            AtpTranslationError::MissingFormulaHandoffEvidence { .. }
        ));

        let mut wrong_item = imported_problem_input(&set, &handoff);
        if let AtpSourceRef::ImportedAxiom { item, .. } =
            &mut wrong_item.formula_projections[0].provenance.source
        {
            *item = AtpSourceBinding::new("other-item");
        }
        assert!(matches!(
            translate_problem(wrong_item).expect_err("wrong imported item"),
            AtpTranslationError::MissingFormulaHandoffEvidence { .. }
        ));

        let mut wrong_statement = imported_problem_input(&set, &handoff);
        if let AtpSourceRef::ImportedAxiom {
            statement_fingerprint,
            ..
        } = &mut wrong_statement.formula_projections[0].provenance.source
        {
            *statement_fingerprint =
                AtpFingerprint::new(KERNEL_FORMULA_FINGERPRINT_ALGORITHM_ID, b"other".to_vec())
                    .expect("fingerprint");
        }
        assert!(matches!(
            translate_problem(wrong_statement).expect_err("wrong imported statement"),
            AtpTranslationError::MissingFormulaHandoffEvidence { .. }
        ));

        let mut wrong_status = imported_problem_input(&set, &handoff);
        if let AtpSourceRef::ImportedAxiom {
            required_status, ..
        } = &mut wrong_status.formula_projections[0].provenance.source
        {
            *required_status = AtpRequiredProofStatus::new("DischargedBuiltin");
        }
        assert!(matches!(
            translate_problem(wrong_status).expect_err("wrong imported status"),
            AtpTranslationError::MissingFormulaHandoffEvidence { .. }
        ));

        let mut wrong_class = imported_problem_input(&set, &handoff);
        wrong_class.formula_projections[0].provenance.source = imported_theorem_source();
        assert!(matches!(
            translate_problem(wrong_class).expect_err("wrong imported class"),
            AtpTranslationError::MissingFormulaHandoffEvidence { .. }
        ));

        let mut wrong_imported_fingerprint = imported_problem_input(&set, &handoff);
        wrong_imported_fingerprint.formula_projections[0].handoff_formula_fingerprint =
            AtpFingerprint::new(
                KERNEL_FORMULA_FINGERPRINT_ALGORITHM_ID,
                b"wrong-imported".to_vec(),
            )
            .expect("fingerprint");
        assert!(matches!(
            translate_problem(wrong_imported_fingerprint)
                .expect_err("wrong imported handoff fingerprint"),
            AtpTranslationError::MissingFormulaHandoffEvidence { .. }
        ));

        let mut wrong_imported_payload = imported_problem_input(&set, &handoff);
        wrong_imported_payload.formula_projections[0].handoff_provenance_payload =
            b"wrong-imported-provenance".to_vec();
        assert!(matches!(
            translate_problem(wrong_imported_payload).expect_err("wrong imported handoff payload"),
            AtpTranslationError::MissingFormulaHandoffEvidence { .. }
        ));

        let mut missing_context = imported_problem_input(&set, &handoff);
        if let AtpSourceRef::ImportedAxiom {
            context_requirement,
            ..
        } = &mut missing_context.formula_projections[0].provenance.source
        {
            *context_requirement = AtpSourceBinding::new("");
        }
        assert!(matches!(
            translate_problem(missing_context).expect_err("missing imported context"),
            AtpTranslationError::Problem {
                source: AtpProblemError::EmptyField {
                    field: "imported.context_requirement"
                }
            }
        ));

        let mut wrong_context = imported_problem_input(&set, &handoff);
        if let AtpSourceRef::ImportedAxiom {
            context_requirement,
            ..
        } = &mut wrong_context.formula_projections[0].provenance.source
        {
            *context_requirement = AtpSourceBinding::new("wrong-context");
        }
        assert!(matches!(
            translate_problem(wrong_context).expect_err("wrong imported context"),
            AtpTranslationError::Problem {
                source: AtpProblemError::UnsupportedProfileFeature {
                    feature: "imported formula context requirement"
                }
            }
        ));
    }

    #[test]
    fn imported_duplicate_source_tuple_fails_closed_across_symbols() {
        let set = fixture_set_with(
            VcStatus::NeedsAtp,
            "imported-duplicate-source",
            vec![
                PremiseRef::ImportedFact {
                    symbol: VcText::new("Imported::A1"),
                },
                PremiseRef::ImportedFact {
                    symbol: VcText::new("Imported::A2"),
                },
            ],
            None,
        );
        let handoff = handoff_with_imported_symbols(&set, &["Imported::A1", "Imported::A2"]);
        let mut input = imported_problem_input(&set, &handoff);
        let mut second = imported_formula_projection(
            "Imported::A2",
            AtpFormulaTree::Atom(AtpAtom::new("p", Vec::new())),
        );
        second.handoff_provenance_payload =
            imported_provenance_payload("Imported::A1").into_bytes();
        input.formula_projections.push(second);

        assert!(matches!(
            translate_problem(input).expect_err("duplicate imported source tuple"),
            AtpTranslationError::DuplicatePremiseIdentity { .. }
        ));
    }

    #[test]
    fn imported_formula_materializes_with_handoff_agreement() {
        let set = fixture_set_with(
            VcStatus::NeedsAtp,
            "imported-ok",
            vec![PremiseRef::ImportedFact {
                symbol: VcText::new("Imported::A1"),
            }],
            None,
        );
        let handoff = handoff_with_imported(&set);
        let problem = translate_problem(imported_problem_input(&set, &handoff)).expect("problem");

        assert_eq!(problem.axioms().len(), 1);
        assert!(matches!(
            problem.provenance()[1].source(),
            AtpSourceRef::ImportedAxiom { .. }
        ));
    }

    #[test]
    fn soft_type_guards_are_preserved_in_full_problem_translation() {
        let set = fixture_set(VcStatus::NeedsAtp, "soft-type");
        let handoff = handoff(&set);
        let mut input = basic_problem_input(&set, &handoff);
        input.logic_profile = profile(SoftTypeStrategy::GuardPredicates);
        input
            .declaration_projections
            .push(AtpDeclarationProjection {
                key: AtpProjectionKey::new("type-predicate"),
                kind: AtpDeclarationKind::Predicate,
                symbol: AtpSymbolName::new("type_guard"),
                arity: 0,
                provenance: declaration_provenance("type-predicate"),
                symbol_source: AtpSymbolSourceProjection::TypeGuard(AtpProjectionKey::new("guard")),
            });
        input.soft_type_projections = vec![soft_guard(
            "guard",
            AtpFormulaTree::Atom(AtpAtom::new("type_guard", Vec::new())),
        )];
        let problem = translate_problem(input).expect("problem");

        assert_eq!(problem.type_context().guards().len(), 1);
        assert_eq!(
            problem.type_context().guards()[0].formula(),
            &AtpFormulaTree::Atom(AtpAtom::new("type_guard", Vec::new()))
        );
    }

    fn empty_input<'a>(
        set: &'a VcSet,
        handoff: &'a VcKernelEvidenceHandoff,
    ) -> AtpDeclarationTranslationInput<'a> {
        AtpDeclarationTranslationInput {
            vc_set: set,
            vc: VcId::new(0),
            kernel_handoff: handoff,
            logic_profile: profile(SoftTypeStrategy::BackendSorts),
            declaration_projections: Vec::new(),
            soft_type_projections: Vec::new(),
            diagnostics: Vec::new(),
        }
    }

    fn basic_problem_input<'a>(
        set: &'a VcSet,
        handoff: &'a VcKernelEvidenceHandoff,
    ) -> AtpTranslationInput<'a> {
        AtpTranslationInput {
            vc_set: set,
            vc: VcId::new(0),
            kernel_handoff: handoff,
            logic_profile: profile(SoftTypeStrategy::BackendSorts),
            declaration_projections: vec![declaration_projection(
                "pred-p",
                "p",
                AtpDeclarationKind::Predicate,
                0,
            )],
            soft_type_projections: Vec::new(),
            formula_projections: vec![
                local_formula_projection(AtpFormulaTree::Atom(AtpAtom::new("p", Vec::new()))),
                goal_formula_projection(AtpFormulaTree::False),
            ],
            diagnostics: Vec::new(),
        }
    }

    fn imported_problem_input<'a>(
        set: &'a VcSet,
        handoff: &'a VcKernelEvidenceHandoff,
    ) -> AtpTranslationInput<'a> {
        AtpTranslationInput {
            vc_set: set,
            vc: VcId::new(0),
            kernel_handoff: handoff,
            logic_profile: profile(SoftTypeStrategy::BackendSorts),
            declaration_projections: vec![declaration_projection(
                "pred-p",
                "p",
                AtpDeclarationKind::Predicate,
                0,
            )],
            soft_type_projections: Vec::new(),
            formula_projections: vec![
                imported_formula_projection(
                    "Imported::A1",
                    AtpFormulaTree::Atom(AtpAtom::new("p", Vec::new())),
                ),
                goal_formula_projection(AtpFormulaTree::False),
            ],
            diagnostics: Vec::new(),
        }
    }

    fn two_premise_problem_input<'a>(
        set: &'a VcSet,
        handoff: &'a VcKernelEvidenceHandoff,
    ) -> AtpTranslationInput<'a> {
        let mut input = basic_problem_input(set, handoff);
        input.formula_projections.push(generated_formula_projection(
            2,
            AtpFormulaTree::Atom(AtpAtom::new("q", Vec::new())),
        ));
        input.declaration_projections.push(declaration_projection(
            "pred-q",
            "q",
            AtpDeclarationKind::Predicate,
            0,
        ));
        input
    }

    fn declaration_projection(
        key: &str,
        symbol: &str,
        kind: AtpDeclarationKind,
        arity: u32,
    ) -> AtpDeclarationProjection {
        AtpDeclarationProjection {
            key: AtpProjectionKey::new(key),
            kind,
            symbol: AtpSymbolName::new(symbol),
            arity,
            provenance: declaration_provenance(key),
            symbol_source: match kind {
                AtpDeclarationKind::GeneratedBinder => {
                    AtpSymbolSourceProjection::GeneratedBinder(AtpSourceBinding::new(key))
                }
                _ => AtpSymbolSourceProjection::MizarSymbol(AtpSourceBinding::new(key)),
            },
        }
    }

    fn local_formula_projection(formula: AtpFormulaTree) -> AtpFormulaProjection {
        formula_projection(
            AtpFormulaProjectionTarget::VcFormula(VcFormulaRef::Generated(
                VcGeneratedFormulaId::new(0),
            )),
            formula,
            AtpSourceRef::LocalHypothesis(AtpSourceBinding::new("local-context:1")),
            "local-context:1",
            "atp-provenance:local:0",
            b"formula-0",
            b"provenance-0",
        )
    }

    fn cited_formula_projection(formula: AtpFormulaTree) -> AtpFormulaProjection {
        formula_projection(
            AtpFormulaProjectionTarget::VcFormula(VcFormulaRef::Generated(
                VcGeneratedFormulaId::new(0),
            )),
            formula,
            AtpSourceRef::CitedPremise(AtpSourceBinding::new("cited-premise:1")),
            "cited-premise:1",
            "atp-provenance:cited:0",
            b"formula-0",
            b"provenance-0",
        )
    }

    fn goal_formula_projection(formula: AtpFormulaTree) -> AtpFormulaProjection {
        formula_projection(
            AtpFormulaProjectionTarget::VcFormula(VcFormulaRef::Generated(
                VcGeneratedFormulaId::new(1),
            )),
            formula,
            AtpSourceRef::GeneratedVcFact(AtpSourceBinding::new("goal:1")),
            "goal:1",
            "atp-provenance:goal:1",
            b"formula-1",
            b"provenance-1",
        )
    }

    fn generated_formula_projection(index: usize, formula: AtpFormulaTree) -> AtpFormulaProjection {
        let source_index = index + 1;
        formula_projection(
            AtpFormulaProjectionTarget::VcFormula(VcFormulaRef::Generated(
                VcGeneratedFormulaId::new(index),
            )),
            formula,
            AtpSourceRef::GeneratedVcFact(AtpSourceBinding::new(format!(
                "generated:{source_index}"
            ))),
            format!("generated:{source_index}"),
            format!("atp-provenance:generated:{index}"),
            format!("formula-{index}").as_bytes(),
            format!("provenance-{index}").as_bytes(),
        )
    }

    fn imported_formula_projection(symbol: &str, formula: AtpFormulaTree) -> AtpFormulaProjection {
        formula_projection(
            AtpFormulaProjectionTarget::ImportedFact(VcText::new(symbol)),
            formula,
            AtpSourceRef::ImportedAxiom {
                package: AtpSourceBinding::new("pkg"),
                module: AtpSourceBinding::new("module"),
                item: AtpSourceBinding::new("item"),
                statement_fingerprint: AtpFingerprint::new(
                    KERNEL_FORMULA_FINGERPRINT_ALGORITHM_ID,
                    b"statement".to_vec(),
                )
                .expect("statement fingerprint"),
                required_status: AtpRequiredProofStatus::new("KernelVerified"),
                context_requirement: AtpSourceBinding::new(fixture_formula_context_binding()),
            },
            format!("imported:{symbol}"),
            format!("atp-provenance:imported:{symbol}"),
            imported_fingerprint_digest(symbol).as_bytes(),
            imported_provenance_payload(symbol).as_bytes(),
        )
    }

    fn imported_theorem_source() -> AtpSourceRef {
        AtpSourceRef::ImportedTheorem {
            package: AtpSourceBinding::new("pkg"),
            module: AtpSourceBinding::new("module"),
            item: AtpSourceBinding::new("item"),
            statement_fingerprint: AtpFingerprint::new(
                KERNEL_FORMULA_FINGERPRINT_ALGORITHM_ID,
                b"statement".to_vec(),
            )
            .expect("statement fingerprint"),
            required_status: AtpRequiredProofStatus::new("KernelVerified"),
            context_requirement: AtpSourceBinding::new(fixture_formula_context_binding()),
        }
    }

    fn formula_projection(
        target: AtpFormulaProjectionTarget,
        formula: AtpFormulaTree,
        source: AtpSourceRef,
        source_identity: impl Into<AtpProjectionKey>,
        provenance_payload: impl Into<AtpPayload>,
        fingerprint_digest: &[u8],
        handoff_provenance_payload: &[u8],
    ) -> AtpFormulaProjection {
        AtpFormulaProjection {
            target,
            formula,
            provenance: AtpProjectionProvenance::new(source, provenance_payload),
            source_identity: source_identity.into(),
            handoff_formula_fingerprint: AtpFingerprint::new(
                KERNEL_FORMULA_FINGERPRINT_ALGORITHM_ID,
                fingerprint_digest.to_vec(),
            )
            .expect("formula fingerprint"),
            handoff_provenance_payload: handoff_provenance_payload.to_vec(),
        }
    }

    fn soft_guard(key: &str, formula: AtpFormulaTree) -> AtpSoftTypeProjection {
        AtpSoftTypeProjection {
            key: AtpProjectionKey::new(key),
            representation: AtpSoftTypeRepresentation::GuardFormula(formula),
            provenance: type_provenance(key),
        }
    }

    fn declaration_provenance(key: &str) -> AtpProjectionProvenance {
        AtpProjectionProvenance::new(
            AtpSourceRef::GeneratedVcFact(AtpSourceBinding::new(format!("decl:{key}"))),
            format!("decl-payload:{key}"),
        )
    }

    fn type_provenance(key: &str) -> AtpProjectionProvenance {
        AtpProjectionProvenance::new(
            AtpSourceRef::TypeFact(AtpSourceBinding::new(format!("type:{key}"))),
            format!("type-payload:{key}"),
        )
    }

    fn profile(soft_types: SoftTypeStrategy) -> LogicProfile {
        LogicProfile::try_new(
            "fof",
            LogicFragment::Fof,
            EqualitySupport::Supported,
            QuantifierPolicy::FirstOrder,
            soft_types,
            NativePropertySupport::Unsupported,
            BTreeSet::from([ConcreteFormat::Tptp]),
        )
        .expect("logic profile")
    }

    fn profile_without_equality() -> LogicProfile {
        LogicProfile::try_new(
            "fof-no-equality",
            LogicFragment::Fof,
            EqualitySupport::Unsupported,
            QuantifierPolicy::FirstOrder,
            SoftTypeStrategy::BackendSorts,
            NativePropertySupport::Unsupported,
            BTreeSet::from([ConcreteFormat::Tptp]),
        )
        .expect("logic profile")
    }

    fn propositional_guard_profile() -> LogicProfile {
        LogicProfile::try_new(
            "propositional-guards",
            LogicFragment::Fof,
            EqualitySupport::Supported,
            QuantifierPolicy::PropositionalOnly,
            SoftTypeStrategy::GuardPredicates,
            NativePropertySupport::Unsupported,
            BTreeSet::from([ConcreteFormat::Tptp]),
        )
        .expect("logic profile")
    }

    fn handoff(set: &VcSet) -> VcKernelEvidenceHandoff {
        let payloads = formula_payloads(set);
        build_kernel_evidence_handoff(KernelEvidenceHandoffInput {
            vc_set: set,
            vc: VcId::new(0),
            kernel_profile: KernelEvidenceProfile::v1(1, KernelClauseTautologyPolicy::Reject),
            symbol_manifest: &[],
            variable_manifest: &[],
            formula_payloads: &payloads,
            imported_formula_payloads: &[],
            substitutions: &[],
            formula_context: None,
            discharge_output: None,
        })
        .expect("kernel handoff")
    }

    fn handoff_with_imported(set: &VcSet) -> VcKernelEvidenceHandoff {
        handoff_with_imported_symbols(set, &["Imported::A1"])
    }

    fn handoff_with_imported_symbols(set: &VcSet, symbols: &[&str]) -> VcKernelEvidenceHandoff {
        let payloads = formula_payloads(set);
        let imported_payloads = symbols
            .iter()
            .map(|symbol| imported_payload(&VcText::new(*symbol)))
            .collect::<Vec<_>>();
        let context = imported_context_for_payloads(&imported_payloads);
        build_kernel_evidence_handoff(KernelEvidenceHandoffInput {
            vc_set: set,
            vc: VcId::new(0),
            kernel_profile: KernelEvidenceProfile::v1(1, KernelClauseTautologyPolicy::Reject),
            symbol_manifest: &[],
            variable_manifest: &[],
            formula_payloads: &payloads,
            imported_formula_payloads: &imported_payloads,
            substitutions: &[],
            formula_context: Some(&context),
            discharge_output: None,
        })
        .expect("kernel handoff")
    }

    fn formula_payloads(set: &VcSet) -> Vec<KernelFormulaPayload> {
        set.generated_formulas()
            .iter()
            .map(|formula| KernelFormulaPayload {
                formula_ref: VcFormulaRef::Generated(formula.id),
                projection: KernelFormulaProjection {
                    formula_fingerprint: fingerprint(
                        KERNEL_FORMULA_FINGERPRINT_ALGORITHM_ID,
                        format!("formula-{}", formula.id.index()).as_bytes(),
                    ),
                    formula_bytes: format!("kernel-formula-{}", formula.id.index()).into_bytes(),
                    provenance_payload: format!("provenance-{}", formula.id.index()).into_bytes(),
                },
            })
            .collect()
    }

    fn imported_payload(symbol: &VcText) -> KernelImportedFormulaPayload {
        KernelImportedFormulaPayload {
            symbol: symbol.clone(),
            class: KernelImportedFormulaClass::Axiom,
            requirement: imported_requirement(),
            projection: KernelFormulaProjection {
                formula_fingerprint: fingerprint(
                    KERNEL_FORMULA_FINGERPRINT_ALGORITHM_ID,
                    imported_fingerprint_digest(symbol.as_str()).as_bytes(),
                ),
                formula_bytes: format!("imported-formula-{}", symbol.as_str()).into_bytes(),
                provenance_payload: imported_provenance_payload(symbol.as_str()).into_bytes(),
            },
        }
    }

    fn imported_context(
        payload: &KernelImportedFormulaPayload,
    ) -> KernelFormulaContextRequirements {
        imported_context_for_payloads(std::slice::from_ref(payload))
    }

    fn imported_context_for_payloads(
        payloads: &[KernelImportedFormulaPayload],
    ) -> KernelFormulaContextRequirements {
        let mut imported_axioms = Vec::new();
        let mut imported_theorems = Vec::new();
        for payload in payloads {
            match payload.class {
                KernelImportedFormulaClass::Axiom => {
                    imported_axioms.push(payload.requirement.clone());
                }
                KernelImportedFormulaClass::Theorem => {
                    imported_theorems.push(payload.requirement.clone());
                }
                _ => panic!("unsupported imported formula class in fixture"),
            }
        }
        KernelFormulaContextRequirements {
            provenance_fingerprint: fingerprint(
                KERNEL_FORMULA_FINGERPRINT_ALGORITHM_ID,
                b"imported-context",
            ),
            imported_axioms,
            imported_theorems,
        }
    }

    fn fixture_formula_context_binding() -> String {
        let payload = imported_payload(&VcText::new("Imported::A1"));
        formula_context_binding(&imported_context(&payload))
    }

    fn imported_requirement() -> KernelImportedFactRequirement {
        KernelImportedFactRequirement {
            imported_fact_id: 0,
            package_id: b"pkg".to_vec(),
            module_path: b"module".to_vec(),
            exported_item_id: b"item".to_vec(),
            statement_fingerprint: fingerprint(
                KERNEL_FORMULA_FINGERPRINT_ALGORITHM_ID,
                b"statement",
            ),
            required_proof_status: KernelRequiredProofStatus::KernelVerified,
        }
    }

    fn imported_fingerprint_digest(_symbol: &str) -> String {
        "statement".to_owned()
    }

    fn imported_provenance_payload(symbol: &str) -> String {
        format!("imported-provenance-{symbol}")
    }

    fn fixture_set(status: VcStatus, module: &str) -> VcSet {
        fixture_set_with(
            status,
            module,
            vec![PremiseRef::LocalContext(ContextEntryId::new(0))],
            None,
        )
    }

    fn fixture_set_with(
        status: VcStatus,
        module: &str,
        premises: Vec<PremiseRef>,
        proof_hint: Option<ProofHint>,
    ) -> VcSet {
        fixture_set_with_local_formula(
            status,
            module,
            premises,
            proof_hint,
            VcFormulaRef::Generated(VcGeneratedFormulaId::new(0)),
        )
    }

    fn fixture_set_with_local_formula(
        status: VcStatus,
        module: &str,
        premises: Vec<PremiseRef>,
        proof_hint: Option<ProofHint>,
        local_formula: VcFormulaRef,
    ) -> VcSet {
        fixture_set_with_local_formula_and_kind(
            status,
            module,
            premises,
            proof_hint,
            local_formula,
            ContextEntryKind::ProofAssumption,
        )
    }

    fn fixture_set_with_local_entry_kind(
        status: VcStatus,
        module: &str,
        premises: Vec<PremiseRef>,
        proof_hint: Option<ProofHint>,
        local_kind: ContextEntryKind,
    ) -> VcSet {
        fixture_set_with_local_formula_and_kind(
            status,
            module,
            premises,
            proof_hint,
            VcFormulaRef::Generated(VcGeneratedFormulaId::new(0)),
            local_kind,
        )
    }

    fn fixture_set_with_local_formula_and_kind(
        status: VcStatus,
        module: &str,
        premises: Vec<PremiseRef>,
        proof_hint: Option<ProofHint>,
        local_formula: VcFormulaRef,
        local_kind: ContextEntryKind,
    ) -> VcSet {
        let snapshot = BuildSnapshotId::from_published_schema_str(
            "mizar-session-build-snapshot-v1:\
             3333333333333333333333333333333333333333333333333333333333333333",
        )
        .expect("snapshot id");
        let source = InMemorySessionIdAllocator::new()
            .next_source_id(snapshot)
            .expect("source id");
        let generated_formulas = vec![
            VcGeneratedFormula {
                id: VcGeneratedFormulaId::new(0),
                kind: VcGeneratedFormulaKind::GeneratedTypeObligation,
                shape: VcGeneratedFormulaShape::True,
                provenance: vec![vc_provenance("generated-0")],
            },
            VcGeneratedFormula {
                id: VcGeneratedFormulaId::new(1),
                kind: VcGeneratedFormulaKind::SplitGoal,
                shape: VcGeneratedFormulaShape::False,
                provenance: vec![vc_provenance("generated-1")],
            },
            VcGeneratedFormula {
                id: VcGeneratedFormulaId::new(2),
                kind: VcGeneratedFormulaKind::Conjunction,
                shape: VcGeneratedFormulaShape::True,
                provenance: vec![vc_provenance("generated-2")],
            },
        ];
        VcSet::try_new(VcSetParts {
            schema_version: VcSchemaVersion::new("atp-translator-test-v1"),
            snapshot,
            source,
            module: VcModuleRef::new(module),
            generated_formulas,
            vcs: vec![VcIr {
                id: VcId::new(0),
                kind: VcKind::TheoremProofStep,
                source: VcSourceRef {
                    primary: source_ref(source),
                    related: Vec::new(),
                },
                seed: mizar_vc::vc_ir::SeedVcRef {
                    handoff: ObligationHandoffId::new(0),
                },
                anchor: incomplete_anchor(source),
                local_context: LocalContext::try_new(
                    vec![ContextEntry {
                        id: ContextEntryId::new(0),
                        sort_key: CanonicalSortKey::new("000-local"),
                        kind: local_kind,
                        formula: Some(local_formula),
                        provenance: vec![vc_provenance("local")],
                    }],
                    Vec::new(),
                )
                .expect("context"),
                premises,
                goal: VcFormulaRef::Generated(VcGeneratedFormulaId::new(1)),
                proof_hint,
                status,
                provenance: vec![vc_provenance("vc")],
            }],
            seed_accounting: vec![SeedAccounting {
                handoff: ObligationHandoffId::new(0),
                origin: SeedOriginRef::ExistingCore {
                    seed: ObligationSeedId::new(0),
                },
                seed_status: ObligationSeedStatus::Active,
                mapping: SeedVcMapping::One { vc: VcId::new(0) },
            }],
        })
        .expect("vc set")
    }

    fn incomplete_anchor(source: SourceId) -> mizar_vc::vc_ir::ObligationAnchor {
        mizar_vc::vc_ir::ObligationAnchor {
            owner: AnchorOwner::Theorem(CoreItemId::new(0)),
            kind: VcKind::TheoremProofStep,
            local_path: LocalProofOrProgramPath::new("proof/0"),
            label: Some(AnchorLabel {
                role: AnchorLabelRole::UserLabel,
                hint: Some(CoreLabelRef::new("A1")),
            }),
            semantic_origin: NormalizedSemanticOrigin::new("theorem:sample"),
            source_range: Some(SourceRange {
                source_id: source,
                start: 0,
                end: 4,
            }),
            provenance: vec![vc_provenance("anchor")],
            source_shape_hash: HashMarker::Unavailable {
                reason: AnchorUnavailableReason::new("test fixture"),
            },
            canonical_goal_hash: HashMarker::Unavailable {
                reason: AnchorUnavailableReason::new("test fixture"),
            },
            canonical_context_hash: HashMarker::Unavailable {
                reason: AnchorUnavailableReason::new("test fixture"),
            },
            generation_schema_version: GenerationSchemaVersion::new("atp-translator-test"),
            completeness: AnchorCompleteness::Incomplete {
                missing: vec![
                    AnchorIngredient::SourceShapeHash,
                    AnchorIngredient::CanonicalGoalHash,
                    AnchorIngredient::CanonicalContextHash,
                ],
            },
        }
    }

    fn source_ref(source: SourceId) -> CoreSourceRef {
        CoreSourceRef::direct(SourceRange {
            source_id: source,
            start: 0,
            end: 4,
        })
        .with_provenance(vec![CoreProvenance::new(
            CoreProvenancePhase::ProofSkeleton,
            "atp-translator-test",
        )])
    }

    fn vc_provenance(key: &str) -> VcProvenance {
        VcProvenance {
            phase: VcProvenancePhase::Generator,
            key: VcText::new(key),
            core: None,
        }
    }

    fn fingerprint(algorithm_id: u8, digest: &[u8]) -> KernelEvidenceFingerprint {
        KernelEvidenceFingerprint::new(algorithm_id, digest.to_vec()).expect("fingerprint")
    }
}
