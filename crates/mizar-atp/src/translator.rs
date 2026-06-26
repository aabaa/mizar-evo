//! Deterministic VC-to-ATP declaration translation.
//!
//! This module implements the task-5 slice specified in
//! [translator.md](../../../doc/design/mizar-atp/en/translator.md). It builds
//! only the declaration, symbol-map, soft-type, provenance, and target-binding
//! handoff needed by later translator tasks; it does not materialize premise
//! axioms or conjectures and it does not run proof search.

use crate::problem::{
    AtpDeclaration, AtpDeclarationId, AtpDeclarationKind, AtpDiagnostic, AtpFingerprint,
    AtpFormulaTree, AtpPayload, AtpProblemError, AtpProvenance, AtpProvenanceId,
    AtpRequiredProofStatus, AtpSourceBinding, AtpSourceRef, AtpSymbolMapEntry, AtpSymbolName,
    AtpSymbolSource, AtpTargetBinding, AtpTerm, AtpTypeContext, AtpTypeGuard, AtpTypeGuardId,
    EqualitySupport, LogicProfile, QuantifierPolicy, SoftTypeStrategy,
};
use mizar_vc::{
    kernel_evidence_handoff::{KernelEvidenceHandoffError, VcKernelEvidenceHandoff},
    vc_ir::{VcId, VcSet, VcStatus},
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

pub struct AtpDeclarationTranslationInput<'a> {
    pub vc_set: &'a VcSet,
    pub vc: VcId,
    pub kernel_handoff: &'a VcKernelEvidenceHandoff,
    pub logic_profile: LogicProfile,
    pub declaration_projections: Vec<AtpDeclarationProjection>,
    pub soft_type_projections: Vec<AtpSoftTypeProjection>,
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
            KernelFormulaPayload, KernelFormulaProjection, build_kernel_evidence_handoff,
        },
        vc_ir::{
            AnchorCompleteness, AnchorIngredient, AnchorLabel, AnchorLabelRole, AnchorOwner,
            AnchorUnavailableReason, CanonicalSortKey, ContextEntry, ContextEntryId,
            ContextEntryKind, GenerationSchemaVersion, HashMarker, LocalContext, PremiseRef,
            SeedAccounting, SeedOriginRef, SeedVcMapping, VcFormulaRef, VcGeneratedFormula,
            VcGeneratedFormulaId, VcGeneratedFormulaKind, VcGeneratedFormulaShape, VcIr, VcKind,
            VcModuleRef, VcProvenance, VcProvenancePhase, VcSchemaVersion, VcSet, VcSetParts,
            VcSourceRef, VcText,
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

    fn fixture_set(status: VcStatus, module: &str) -> VcSet {
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
                        kind: ContextEntryKind::ProofAssumption,
                        formula: Some(VcFormulaRef::Generated(VcGeneratedFormulaId::new(0))),
                        provenance: vec![vc_provenance("local")],
                    }],
                    Vec::new(),
                )
                .expect("context"),
                premises: vec![PremiseRef::LocalContext(ContextEntryId::new(0))],
                goal: VcFormulaRef::Generated(VcGeneratedFormulaId::new(1)),
                proof_hint: None,
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
