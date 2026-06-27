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
mod tests;
