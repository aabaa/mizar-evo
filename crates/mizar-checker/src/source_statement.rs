//! Syntax-free transport for source theorem statements.

use crate::{
    binding_env::{
        BinderIdentity, BindingContextId, BindingContextLayer, BindingContextOwner,
        BindingContextRecovery, BindingEnv, BindingId, BindingKind, BindingRecoveryState,
        BindingStatus, BindingTypeSite,
    },
    source_application::{SourceFunctorApplicationHandoff, SourceFunctorApplicationId},
    source_atomic_formula::{
        SourceAtomicEdgeRole, SourceAtomicFormulaHandoff, SourceAtomicFormulaId,
        SourceAtomicFormulaKind, SourceAtomicFormulaRecovery, SourceAtomicRequestKind,
        SourceAtomicTermTarget,
    },
    source_composite_formula::{
        SourceCompositeFormulaHandoff, SourceCompositeFormulaId, SourceCompositeFormulaKind,
        SourceCompositeFormulaRecovery, SourceFormulaRootOwnership,
    },
    source_formula_composition::SourceFormulaCompositionHandoff,
    source_set_term::{
        SourceSetEdgeRole, SourceSetRequestKind, SourceSetTarget, SourceSetTermHandoff,
        SourceSetTermId, SourceSetTermKind, SourceSetTermRecovery, SourceSetTypeHead,
        SourceSetTypeOwner, SourceSetTypeRole, SourceSetTypeSiteId,
    },
    source_structure::{
        SourceStructureEdgeRole, SourceStructureHandoff, SourceStructureMemberId,
        SourceStructureMemberRole, SourceStructureRecovery, SourceStructureRequestKind,
        SourceStructureRootId, SourceStructureTarget, SourceStructureTermId,
        SourceStructureTermKind,
    },
    source_term::{
        SourcePrimaryTermHandoff, SourcePrimaryTermId, SourcePrimaryTermKind,
        SourcePrimaryTermRecovery, SourcePrimaryTermReferenceId, SourcePrimaryTermReferenceRole,
        SourcePrimaryTermRole,
    },
    type_checker::{CheckedStatementOwner, TermFormulaChecker},
    typed_ast::{NodeRecoveryState, TypedArena, TypedAst, TypedSiteRef, TypingState},
};
#[cfg(test)]
use crate::{source_set_term::SourceSetGeneratorId, source_structure::SourceFieldUpdateId};
use mizar_resolve::{
    env::{ContributionKind, ExportStatus, SourceContributionId, SymbolEnv, Visibility},
    labels::{
        LabelProjection, LabelProjectionSource, LabelReferenceCandidate, LabelReferenceScope,
        LabelResolutionResult, LabelResolver, LabelScopePath,
    },
    names::ResolvedVariableScope,
    resolved_ast::{
        ImportResolution, LabelExpectation, LabelKind, LabelOriginPath, LabelRefId,
        LabelResolution, ModuleId, NodeReferenceKey, NodeResolutionState, RecoveryState,
        ResolvedArena, ResolvedAst, ResolvedNode, ResolvedNodeId, SemanticOrigin,
        SurfaceResolvedArena, SymbolId,
    },
};
use mizar_session::{ModulePath, SourceAnchor, SourceId, SourceRange};
use std::{
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

dense_id!(SourceTheoremOwnerId);
dense_id!(SourceStatementId);
dense_id!(SourceStatementContextId);
dense_id!(SourceStatementInputFactId);
dense_id!(SourceStatementCandidateFactId);
dense_id!(SourceStatementLabelId);
dense_id!(SourceStatementCitationId);
dense_id!(SourceStatementWitnessId);
dense_id!(SourceStatementWitnessNameId);

/// Complete input for one source/module statement transaction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceStatementHandoffInput {
    pub source_id: SourceId,
    pub module_id: ModuleId,
    pub owners: Vec<SourceTheoremOwnerInput>,
    pub statements: Vec<SourceStatementInput>,
    pub contexts: Vec<SourceStatementContextInput>,
    pub input_facts: Vec<SourceStatementInputFactInput>,
    pub candidate_facts: Vec<SourceStatementCandidateFactInput>,
}

/// Resolver-authenticated source theorem owner input.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceTheoremOwnerInput {
    pub symbol: SymbolId,
    pub contribution: SourceContributionId,
    pub site: TypedSiteRef,
    pub source_range: SourceRange,
    pub spelling: String,
    pub role: SourceTheoremRole,
    pub status: SourceTheoremStatus,
    pub recovery: SourceStatementRecovery,
}

/// Source theorem statement input.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceStatementInput {
    pub owner: SourceTheoremOwnerId,
    pub context: SourceStatementContextId,
    pub formula: SourceStatementFormulaTarget,
    pub site: TypedSiteRef,
    pub source_range: SourceRange,
    pub source_ordinal: usize,
    pub spelling: String,
    pub kind: SourceStatementKind,
    pub recovery: SourceStatementRecovery,
}

/// Visibility context input for one source statement.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceStatementContextInput {
    pub statement: SourceStatementId,
    pub binding_context: BindingContextId,
    pub source_range: SourceRange,
    pub visible_bindings: Vec<BindingId>,
}

/// Input fact visible to one source statement.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceStatementInputFactInput {
    pub statement: SourceStatementId,
    pub context: SourceStatementContextId,
    pub ordinal: usize,
    pub kind: SourceStatementInputFactKind,
    pub binding: BindingId,
    pub uses: Vec<SourcePrimaryTermReferenceId>,
}

/// Unverified candidate fact owned by one source statement.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceStatementCandidateFactInput {
    pub statement: SourceStatementId,
    pub context: SourceStatementContextId,
    pub ordinal: usize,
    pub kind: SourceStatementCandidateFactKind,
    pub formula: SourceStatementFormulaTarget,
}

/// Complete input for one source-statement reference transaction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceStatementReferenceHandoffInput {
    pub source_id: SourceId,
    pub module_id: ModuleId,
    pub labels: Vec<SourceStatementLabelInput>,
    pub citations: Vec<SourceStatementCitationInput>,
}

/// Complete input for one source-statement witness transaction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceStatementWitnessHandoffInput {
    pub source_id: SourceId,
    pub module_id: ModuleId,
    pub witnesses: Vec<SourceStatementWitnessInput>,
    pub names: Vec<SourceStatementWitnessNameInput>,
}

/// One source witness occurrence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceStatementWitnessInput {
    pub owner: SourceTheoremOwnerId,
    pub binding_context: BindingContextId,
    pub term: SourceStatementWitnessTermTarget,
    pub take_site: TypedSiteRef,
    pub take_range: SourceRange,
    pub site: TypedSiteRef,
    pub source_range: SourceRange,
    pub source_ordinal: usize,
    pub ordinal: usize,
    pub spelling: String,
    pub kind: SourceStatementWitnessKind,
    pub recovery: SourceStatementRecovery,
    pub name: Option<SourceStatementWitnessNameId>,
}

/// One source name attached to a witness occurrence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceStatementWitnessNameInput {
    pub witness: SourceStatementWitnessId,
    pub site: TypedSiteRef,
    pub source_range: SourceRange,
    pub spelling: String,
    pub recovery: SourceStatementRecovery,
}

/// Resolver-authenticated local proof-step label input.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceStatementLabelInput {
    pub statement: SourceStatementId,
    pub context: SourceStatementContextId,
    pub candidate: SourceStatementCandidateFactId,
    pub origin_path: LabelOriginPath,
    pub proof_scope: LabelScopePath,
    pub source_range: SourceRange,
    pub source_ordinal: usize,
    pub visible_after_ordinal: usize,
    pub spelling: String,
    pub kind: SourceStatementLabelKind,
    pub recovery: SourceStatementRecovery,
}

/// Resolver-authenticated local or imported proof/theorem citation input.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceStatementCitationInput {
    pub statement: SourceStatementId,
    pub context: SourceStatementContextId,
    pub target: SourceStatementCitationTarget,
    pub label_ref: LabelRefId,
    pub proof_scope: LabelScopePath,
    pub source_range: SourceRange,
    pub ordinal: usize,
    pub kind: SourceStatementCitationKind,
    pub recovery: SourceStatementRecovery,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum SourceTheoremRole {
    Theorem,
    Lemma,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum SourceTheoremStatus {
    Unmodified,
    Open,
    Assumed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum SourceStatementKind {
    TheoremProposition,
    ProofStepProposition,
    Assumption,
    Conclusion,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum SourceStatementRecovery {
    Normal,
    Degraded,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum SourceStatementFormulaTarget {
    Atomic(SourceAtomicFormulaId),
    Composite(SourceCompositeFormulaId),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum SourceStatementInputFactKind {
    ReservedTypeGuard,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum SourceStatementCandidateFactKind {
    UnverifiedProposition,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum SourceStatementWitnessTermTarget {
    Primary(SourcePrimaryTermId),
    Application(SourceFunctorApplicationId),
    Structure(SourceStructureTermId),
    SetTerm(SourceSetTermId),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum SourceStatementWitnessKind {
    Unnamed,
    Named,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum SourceStatementLabelKind {
    ProofStep,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum SourceStatementCitationTarget {
    Local(SourceStatementLabelId),
    Imported,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum SourceStatementCitationKind {
    SimpleLocal,
    SimpleImported,
}

/// Immutable source statement transaction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceStatementHandoff {
    source_id: SourceId,
    module_id: ModuleId,
    binding_env: BindingEnv,
    binding_fingerprint: String,
    primary_term_fingerprint: String,
    atomic_formula_fingerprint: String,
    composite_formula_fingerprint: Option<String>,
    formula_composition_fingerprint: Option<String>,
    checked_owner: CheckedStatementOwner,
    owner_contribution: SourceContributionId,
    owners: SourceTheoremOwnerTable,
    statements: SourceStatementTable,
    contexts: SourceStatementContextTable,
    input_facts: SourceStatementInputFactTable,
    candidate_facts: SourceStatementCandidateFactTable,
}

/// Immutable resolver-authenticated source-statement reference transaction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceStatementReferenceHandoff {
    source_id: SourceId,
    module_id: ModuleId,
    statement_fingerprint: String,
    resolver_ast: ResolvedAst,
    label_projection: Vec<LabelProjection>,
    reference_candidate: LabelReferenceCandidate,
    label_resolution: LabelResolutionResult,
    labels: SourceStatementLabelTable,
    citations: SourceStatementCitationTable,
}

/// Immutable source-statement witness transaction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceStatementWitnessHandoff {
    source_id: SourceId,
    module_id: ModuleId,
    statement_fingerprint: String,
    primary_term_fingerprint: String,
    application_fingerprint: Option<String>,
    structure_fingerprint: Option<String>,
    set_term_fingerprint: Option<String>,
    witnesses: SourceStatementWitnessTable,
    names: SourceStatementWitnessNameTable,
}

impl SourceStatementHandoff {
    #[allow(clippy::type_complexity)] // Rationale: retain the existing owning tuple and companion types.
    pub(crate) fn from_original(
        source: &SurfaceResolvedArena,
        symbols: &SymbolEnv,
        scope: &ResolvedVariableScope,
    ) -> Result<
        (
            Self,
            Option<SourceStatementReferenceHandoff>,
            Option<SourceStatementWitnessHandoff>,
            TypedAst,
        ),
        SourceStatementError,
    > {
        let invalid = || SourceStatementError::DependencyMismatch;
        let (input, bindings, typed, reference_input, witness_input) =
            TermFormulaChecker::check_source_reserved_statement_inputs(source, symbols, scope)?;
        let owner = input.owners.first().ok_or_else(invalid)?;
        let checked_owner = CheckedStatementOwner::validate_exact_local_theorem(
            symbols,
            owner.symbol.clone(),
            source.source_id(),
            source.module(),
        )
        .map_err(|_| SourceStatementError::InvalidOwner {
            owner: SourceTheoremOwnerId::new(0),
        })?;
        let primary = typed.source_term().ok_or_else(invalid)?;
        let atomic = typed.source_atomic_formula().ok_or_else(invalid)?;
        let handoff = Self {
            source_id: input.source_id,
            module_id: input.module_id,
            binding_fingerprint: bindings.debug_text(),
            binding_env: bindings,
            primary_term_fingerprint: primary.debug_text(),
            atomic_formula_fingerprint: atomic.debug_text(),
            composite_formula_fingerprint: None,
            formula_composition_fingerprint: None,
            checked_owner,
            owner_contribution: owner.contribution,
            owners: SourceTheoremOwnerTable {
                rows: input
                    .owners
                    .into_iter()
                    .map(|row| SourceTheoremOwner {
                        symbol: row.symbol,
                        contribution: row.contribution,
                        site: row.site,
                        source_range: row.source_range,
                        spelling: row.spelling,
                        role: row.role,
                        status: row.status,
                        recovery: row.recovery,
                    })
                    .collect(),
            },
            statements: SourceStatementTable {
                rows: input
                    .statements
                    .into_iter()
                    .map(|row| SourceStatement {
                        owner: row.owner,
                        context: row.context,
                        formula: row.formula,
                        site: row.site,
                        source_range: row.source_range,
                        source_ordinal: row.source_ordinal,
                        spelling: row.spelling,
                        kind: row.kind,
                        recovery: row.recovery,
                    })
                    .collect(),
            },
            contexts: SourceStatementContextTable {
                rows: input
                    .contexts
                    .into_iter()
                    .map(|row| SourceStatementContext {
                        statement: row.statement,
                        binding_context: row.binding_context,
                        source_range: row.source_range,
                        visible_bindings: row.visible_bindings,
                    })
                    .collect(),
            },
            input_facts: SourceStatementInputFactTable {
                rows: input
                    .input_facts
                    .into_iter()
                    .map(|row| SourceStatementInputFact {
                        statement: row.statement,
                        context: row.context,
                        ordinal: row.ordinal,
                        kind: row.kind,
                        binding: row.binding,
                        uses: row.uses,
                    })
                    .collect(),
            },
            candidate_facts: SourceStatementCandidateFactTable {
                rows: input
                    .candidate_facts
                    .into_iter()
                    .map(|row| SourceStatementCandidateFact {
                        statement: row.statement,
                        context: row.context,
                        ordinal: row.ordinal,
                        kind: row.kind,
                        formula: row.formula,
                    })
                    .collect(),
            },
        };
        handoff.validate_installation(
            source.source_id(),
            source.module(),
            primary,
            atomic,
            typed.nodes(),
        )?;
        let references = match reference_input {
            Some((input, ast, labels, resolution)) => {
                let candidate = labels.references().first().ok_or_else(invalid)?.clone();
                let references = SourceStatementReferenceHandoff {
                    source_id: input.source_id,
                    module_id: input.module_id,
                    statement_fingerprint: handoff.debug_text(),
                    resolver_ast: ast,
                    label_projection: labels.projections().to_vec(),
                    reference_candidate: candidate,
                    label_resolution: resolution,
                    labels: SourceStatementLabelTable {
                        rows: input
                            .labels
                            .into_iter()
                            .map(|row| SourceStatementLabel {
                                statement: row.statement,
                                context: row.context,
                                candidate: row.candidate,
                                origin_path: row.origin_path,
                                proof_scope: row.proof_scope,
                                source_range: row.source_range,
                                source_ordinal: row.source_ordinal,
                                visible_after_ordinal: row.visible_after_ordinal,
                                spelling: row.spelling,
                                kind: row.kind,
                                recovery: row.recovery,
                            })
                            .collect(),
                    },
                    citations: SourceStatementCitationTable {
                        rows: input
                            .citations
                            .into_iter()
                            .map(|row| SourceStatementCitation {
                                statement: row.statement,
                                context: row.context,
                                target: row.target,
                                label_ref: row.label_ref,
                                proof_scope: row.proof_scope,
                                source_range: row.source_range,
                                ordinal: row.ordinal,
                                kind: row.kind,
                                recovery: row.recovery,
                            })
                            .collect(),
                    },
                };
                references
                    .validate_installation(
                        source.source_id(),
                        source.module(),
                        &handoff,
                        typed.nodes(),
                    )
                    .map_err(|_| invalid())?;
                Some(references)
            }
            None => None,
        };
        let witnesses = match witness_input {
            Some(input) => {
                let witnesses = SourceStatementWitnessHandoff {
                    source_id: input.source_id,
                    module_id: input.module_id,
                    statement_fingerprint: handoff.debug_text(),
                    primary_term_fingerprint: primary.debug_text(),
                    application_fingerprint: None,
                    structure_fingerprint: None,
                    set_term_fingerprint: typed.source_set_term().map(|set| set.debug_text()),
                    witnesses: SourceStatementWitnessTable {
                        rows: input
                            .witnesses
                            .into_iter()
                            .map(|row| SourceStatementWitness {
                                owner: row.owner,
                                binding_context: row.binding_context,
                                term: row.term,
                                take_site: row.take_site,
                                take_range: row.take_range,
                                site: row.site,
                                source_range: row.source_range,
                                source_ordinal: row.source_ordinal,
                                ordinal: row.ordinal,
                                spelling: row.spelling,
                                kind: row.kind,
                                recovery: row.recovery,
                                name: row.name,
                            })
                            .collect(),
                    },
                    names: SourceStatementWitnessNameTable {
                        rows: input
                            .names
                            .into_iter()
                            .map(|row| SourceStatementWitnessName {
                                witness: row.witness,
                                site: row.site,
                                source_range: row.source_range,
                                spelling: row.spelling,
                                recovery: row.recovery,
                            })
                            .collect(),
                    },
                };
                if references.is_some() {
                    return Err(invalid());
                }
                witnesses
                    .validate_installation_with_set_term(
                        source.source_id(),
                        source.module(),
                        &handoff,
                        primary,
                        typed.source_set_term(),
                        typed.nodes(),
                    )
                    .map_err(|_| invalid())?;
                Some(witnesses)
            }
            None => None,
        };
        Ok((handoff, references, witnesses, typed))
    }

    pub const fn source_id(&self) -> SourceId {
        self.source_id
    }

    pub const fn module_id(&self) -> &ModuleId {
        &self.module_id
    }

    pub const fn binding_env(&self) -> &BindingEnv {
        &self.binding_env
    }

    pub fn binding_fingerprint(&self) -> &str {
        &self.binding_fingerprint
    }

    pub fn primary_term_fingerprint(&self) -> &str {
        &self.primary_term_fingerprint
    }

    pub fn atomic_formula_fingerprint(&self) -> &str {
        &self.atomic_formula_fingerprint
    }

    pub fn composite_formula_fingerprint(&self) -> Option<&str> {
        self.composite_formula_fingerprint.as_deref()
    }

    pub fn formula_composition_fingerprint(&self) -> Option<&str> {
        self.formula_composition_fingerprint.as_deref()
    }

    pub const fn checked_owner(&self) -> &CheckedStatementOwner {
        &self.checked_owner
    }

    pub const fn owners(&self) -> &SourceTheoremOwnerTable {
        &self.owners
    }

    pub const fn statements(&self) -> &SourceStatementTable {
        &self.statements
    }

    pub const fn contexts(&self) -> &SourceStatementContextTable {
        &self.contexts
    }

    pub const fn input_facts(&self) -> &SourceStatementInputFactTable {
        &self.input_facts
    }

    pub const fn candidate_facts(&self) -> &SourceStatementCandidateFactTable {
        &self.candidate_facts
    }

    #[cfg(test)]
    pub(crate) fn is_task_258a_profile(&self) -> bool {
        self.binding_env
            .bindings()
            .get(BindingId::new(0))
            .is_some_and(|binding| binding.status == BindingStatus::Reserved)
            && self.binding_env.contexts().len() == 1
            && self.statements.len() == 1
            && self.contexts.len() == 1
            && self.input_facts.len() == 1
            && self.candidate_facts.len() == 1
    }

    #[cfg(test)]
    pub(crate) fn is_task_258b1_profile(&self) -> bool {
        self.binding_env
            .bindings()
            .get(BindingId::new(0))
            .is_some_and(|binding| binding.status == BindingStatus::Reserved)
            && self.binding_env.contexts().len() == 3
            && self.statements.len() == 4
            && self.contexts.len() == 4
            && self.input_facts.len() == 4
            && self.candidate_facts.len() == 4
    }

    #[cfg(test)]
    pub(crate) fn is_task_258b5a_profile(&self) -> bool {
        self.binding_env
            .bindings()
            .get(BindingId::new(0))
            .is_some_and(|binding| binding.status == BindingStatus::Reserved)
            && self.binding_env.contexts().len() == 4
            && self.statements.len() == 5
            && self.contexts.len() == 5
            && self.input_facts.len() == 5
            && self.candidate_facts.len() == 5
            && self
                .owners
                .get(SourceTheoremOwnerId::new(0))
                .is_some_and(|owner| {
                    owner.source_range == range(self.source_id, 19, 184)
                        && owner.spelling == "FormulaStatementAncestorLabelDescendantCitationSmoke"
                })
    }

    pub(crate) fn is_task_258b5b_profile(&self) -> bool {
        self.binding_env.contexts().len() == 2
            && self.statements.len() == 2
            && self.contexts.len() == 2
            && self.input_facts.len() == 2
            && self.candidate_facts.len() == 2
            && self
                .owners
                .get(SourceTheoremOwnerId::new(0))
                .is_some_and(|owner| {
                    owner.source_range == range(self.source_id, 48, 145)
                        && owner.spelling == "FormulaStatementImportedPublicTheoremCitationSmoke"
                })
    }

    #[cfg(test)]
    pub(crate) fn is_task_258b2_profile(&self) -> bool {
        self.binding_env
            .bindings()
            .get(BindingId::new(0))
            .is_some_and(|binding| binding.status == BindingStatus::Reserved)
            && self.binding_env.contexts().len() == 2
            && self.statements.len() == 3
            && self.contexts.len() == 3
            && self.input_facts.len() == 3
            && self.candidate_facts.len() == 3
    }

    #[cfg(test)]
    pub(crate) fn is_task_258b3_profile(&self) -> bool {
        self.binding_env
            .bindings()
            .get(BindingId::new(0))
            .is_some_and(|binding| binding.status == BindingStatus::Reserved)
            && self.binding_env.contexts().len() == 2
            && self.statements.len() == 2
            && self.contexts.len() == 2
            && self.input_facts.len() == 2
            && self.candidate_facts.len() == 2
            && self
                .owners
                .get(SourceTheoremOwnerId::new(0))
                .is_some_and(|owner| {
                    owner.source_range == range(self.source_id, 19, 103)
                        && owner.spelling == "FormulaStatementSingleWitnessSmoke"
                })
            && self
                .statements
                .get(SourceStatementId::new(1))
                .is_some_and(|statement| {
                    statement.source_range == range(self.source_id, 87, 98)
                        && statement.source_ordinal == 2
                })
    }

    #[cfg(test)]
    pub(crate) fn is_task_258b3n_profile(&self) -> bool {
        self.binding_env
            .bindings()
            .get(BindingId::new(0))
            .is_some_and(|binding| binding.status == BindingStatus::Reserved)
            && self.binding_env.contexts().len() == 2
            && self.statements.len() == 2
            && self.contexts.len() == 2
            && self.input_facts.len() == 2
            && self.candidate_facts.len() == 2
            && self
                .owners
                .get(SourceTheoremOwnerId::new(0))
                .is_some_and(|owner| {
                    owner.source_range == range(self.source_id, 19, 106)
                        && owner.spelling == "FormulaStatementNamedWitnessSmoke"
                })
            && self
                .statements
                .get(SourceStatementId::new(1))
                .is_some_and(|statement| {
                    statement.source_range == range(self.source_id, 90, 101)
                        && statement.source_ordinal == 2
                })
    }

    #[cfg(test)]
    pub(crate) fn is_task_258b3m1_profile(&self) -> bool {
        self.binding_env
            .bindings()
            .get(BindingId::new(0))
            .is_some_and(|binding| binding.status == BindingStatus::Reserved)
            && self.binding_env.contexts().len() == 2
            && self.statements.len() == 2
            && self.contexts.len() == 2
            && self.input_facts.len() == 2
            && self.candidate_facts.len() == 2
            && self
                .owners
                .get(SourceTheoremOwnerId::new(0))
                .is_some_and(|owner| {
                    owner.source_range == range(self.source_id, 19, 112)
                        && owner.spelling == "FormulaStatementMultipleWitnessSmoke"
                })
            && self
                .statements
                .get(SourceStatementId::new(1))
                .is_some_and(|statement| {
                    statement.source_range == range(self.source_id, 96, 107)
                        && statement.source_ordinal == 2
                })
    }

    #[cfg(test)]
    pub(crate) fn is_task_258b3m2a_profile(&self) -> bool {
        self.binding_env.contexts().len() == 2
            && self.statements.len() == 2
            && self.contexts.len() == 2
            && self.input_facts.len() == 2
            && self.candidate_facts.len() == 2
            && self
                .owners
                .get(SourceTheoremOwnerId::new(0))
                .is_some_and(|owner| {
                    owner.source_range == range(self.source_id, 19, 106)
                        && owner.spelling == "FormulaStatementNumeralWitnessSmoke"
                })
            && self
                .statements
                .get(SourceStatementId::new(1))
                .is_some_and(|statement| {
                    statement.source_range == range(self.source_id, 90, 101)
                        && statement.source_ordinal == 2
                })
    }

    #[cfg(test)]
    pub(crate) fn is_task_258b3m2b1_profile(&self) -> bool {
        self.binding_env
            .bindings()
            .get(BindingId::new(0))
            .is_some_and(|binding| binding.status == BindingStatus::Reserved)
            && self.binding_env.contexts().len() == 2
            && self.statements.len() == 2
            && self.contexts.len() == 2
            && self.input_facts.len() == 2
            && self.candidate_facts.len() == 2
            && self
                .owners
                .get(SourceTheoremOwnerId::new(0))
                .is_some_and(|owner| {
                    owner.source_range == range(self.source_id, 19, 112)
                        && owner.spelling == "FormulaStatementParenthesizedWitnessSmoke"
                })
            && self
                .statements
                .get(SourceStatementId::new(1))
                .is_some_and(|statement| {
                    statement.source_range == range(self.source_id, 96, 107)
                        && statement.source_ordinal == 2
                })
    }

    #[cfg(test)]
    pub(crate) fn is_task_258b3m2b2a_profile(&self) -> bool {
        self.binding_env
            .bindings()
            .get(BindingId::new(0))
            .is_some_and(|binding| binding.status == BindingStatus::Reserved)
            && self.binding_env.contexts().len() == 2
            && self.statements.len() == 2
            && self.contexts.len() == 2
            && self.input_facts.len() == 2
            && self.candidate_facts.len() == 2
            && self
                .owners
                .get(SourceTheoremOwnerId::new(0))
                .is_some_and(|owner| {
                    owner.source_range == range(self.source_id, 19, 120)
                        && owner.spelling == "FormulaStatementNestedParenthesizedWitnessSmoke"
                })
            && self
                .statements
                .get(SourceStatementId::new(1))
                .is_some_and(|statement| {
                    statement.source_range == range(self.source_id, 104, 115)
                        && statement.source_ordinal == 2
                })
    }

    #[cfg(test)]
    pub(crate) fn is_task_258b3m2b2b1a_profile(&self) -> bool {
        self.binding_env.contexts().len() == 2
            && self.statements.len() == 2
            && self.contexts.len() == 2
            && self.input_facts.len() == 2
            && self.candidate_facts.len() == 2
            && self
                .owners
                .get(SourceTheoremOwnerId::new(0))
                .is_some_and(|owner| {
                    owner.source_range == range(self.source_id, 48, 142)
                        && owner.spelling == "FormulaStatementApplicationWitnessSmoke"
                })
            && self
                .statements
                .get(SourceStatementId::new(1))
                .is_some_and(|statement| {
                    statement.source_range == range(self.source_id, 126, 137)
                        && statement.source_ordinal == 2
                })
    }

    #[cfg(test)]
    pub(crate) fn is_task_258b3m2b2b1b1_profile(&self) -> bool {
        self.binding_env.contexts().len() == 2
            && self.statements.len() == 2
            && self.contexts.len() == 2
            && self.input_facts.len() == 2
            && self.candidate_facts.len() == 2
            && self
                .owners
                .get(SourceTheoremOwnerId::new(0))
                .is_some_and(|owner| {
                    owner.source_range == range(self.source_id, 48, 157)
                        && owner.spelling == "FormulaStatementParenthesizedApplicationWitnessSmoke"
                })
            && self
                .statements
                .get(SourceStatementId::new(1))
                .is_some_and(|statement| {
                    statement.source_range == range(self.source_id, 141, 152)
                        && statement.source_ordinal == 2
                })
    }

    pub(crate) fn is_task_258b3m2b2b2a_profile(&self) -> bool {
        self.binding_env.contexts().len() == 2
            && self.statements.len() == 2
            && self.contexts.len() == 2
            && self.input_facts.len() == 2
            && self.candidate_facts.len() == 2
            && self
                .owners
                .get(SourceTheoremOwnerId::new(0))
                .is_some_and(|owner| {
                    owner.source_range == range(self.source_id, 48, 171)
                        && owner.spelling == "FormulaStatementStructureConstructorWitnessSmoke"
                })
            && self
                .statements
                .get(SourceStatementId::new(1))
                .is_some_and(|statement| {
                    statement.source_range == range(self.source_id, 155, 166)
                        && statement.source_ordinal == 2
                })
    }

    pub(crate) fn is_task_258b3m2b2b2b_profile(&self) -> bool {
        self.binding_env.contexts().len() == 2
            && self.statements.len() == 2
            && self.contexts.len() == 2
            && self.input_facts.len() == 2
            && self.candidate_facts.len() == 2
            && self
                .owners
                .get(SourceTheoremOwnerId::new(0))
                .is_some_and(|owner| {
                    owner.source_range == range(self.source_id, 48, 170)
                        && owner.spelling == "FormulaStatementStructureSelectorWitnessSmoke"
                })
            && self
                .statements
                .get(SourceStatementId::new(1))
                .is_some_and(|statement| {
                    statement.source_range == range(self.source_id, 154, 165)
                        && statement.source_ordinal == 2
                })
    }

    #[cfg(test)]
    pub(crate) fn is_task_258b3m2b2b2c_profile(&self) -> bool {
        self.binding_env.contexts().len() == 2
            && self.statements.len() == 2
            && self.contexts.len() == 2
            && self.input_facts.len() == 2
            && self.candidate_facts.len() == 2
            && self
                .owners
                .get(SourceTheoremOwnerId::new(0))
                .is_some_and(|owner| {
                    owner.source_range == range(self.source_id, 48, 180)
                        && owner.spelling == "FormulaStatementStructureUpdateWitnessSmoke"
                })
            && self
                .statements
                .get(SourceStatementId::new(1))
                .is_some_and(|statement| {
                    statement.source_range == range(self.source_id, 164, 175)
                        && statement.source_ordinal == 2
                })
    }

    #[cfg(test)]
    pub(crate) fn is_task_258b3m2b2b3a_profile(&self) -> bool {
        self.binding_env.contexts().len() == 2
            && self.statements.len() == 2
            && self.contexts.len() == 2
            && self.input_facts.len() == 2
            && self.candidate_facts.len() == 2
            && self
                .owners
                .get(SourceTheoremOwnerId::new(0))
                .is_some_and(|owner| {
                    owner.source_range == range(self.source_id, 19, 116)
                        && owner.spelling == "FormulaStatementSetEnumerationWitnessSmoke"
                })
            && self
                .statements
                .get(SourceStatementId::new(1))
                .is_some_and(|statement| {
                    statement.source_range == range(self.source_id, 100, 111)
                        && statement.source_ordinal == 2
                })
    }

    #[cfg(test)]
    pub(crate) fn is_task_258b3m2b2b3b_profile(&self) -> bool {
        self.binding_env.contexts().len() == 2
            && self.statements.len() == 2
            && self.contexts.len() == 2
            && self.input_facts.len() == 2
            && self.candidate_facts.len() == 2
            && self
                .owners
                .get(SourceTheoremOwnerId::new(0))
                .is_some_and(|owner| {
                    owner.source_range == range(self.source_id, 19, 117)
                        && owner.spelling == "FormulaStatementEmptySetEnumerationWitnessSmoke"
                })
            && self
                .statements
                .get(SourceStatementId::new(1))
                .is_some_and(|statement| {
                    statement.source_range == range(self.source_id, 101, 112)
                        && statement.source_ordinal == 2
                })
    }

    #[cfg(test)]
    pub(crate) fn is_task_258b3m2b2b3c_profile(&self) -> bool {
        self.binding_env.contexts().len() == 2
            && self.statements.len() == 2
            && self.contexts.len() == 2
            && self.input_facts.len() == 2
            && self.candidate_facts.len() == 2
            && self
                .owners
                .get(SourceTheoremOwnerId::new(0))
                .is_some_and(|owner| {
                    owner.source_range == range(self.source_id, 19, 109)
                        && owner.spelling == "FormulaStatementChoiceWitnessSmoke"
                })
            && self
                .statements
                .get(SourceStatementId::new(1))
                .is_some_and(|statement| {
                    statement.source_range == range(self.source_id, 93, 104)
                        && statement.source_ordinal == 2
                })
    }

    #[cfg(test)]
    pub(crate) fn is_task_258b3m2b2b3d_profile(&self) -> bool {
        self.binding_env.contexts().len() == 2
            && self.statements.len() == 2
            && self.contexts.len() == 2
            && self.input_facts.len() == 2
            && self.candidate_facts.len() == 2
            && self
                .owners
                .get(SourceTheoremOwnerId::new(0))
                .is_some_and(|owner| {
                    owner.source_range == range(self.source_id, 19, 108)
                        && owner.spelling == "FormulaStatementQuaWitnessSmoke"
                })
            && self
                .statements
                .get(SourceStatementId::new(1))
                .is_some_and(|statement| {
                    statement.source_range == range(self.source_id, 92, 103)
                        && statement.source_ordinal == 2
                })
    }

    #[cfg(test)]
    pub(crate) fn is_task_258b3m2b2b3e_profile(&self) -> bool {
        self.binding_env.contexts().len() == 2
            && self.statements.len() == 2
            && self.contexts.len() == 2
            && self.input_facts.len() == 2
            && self.candidate_facts.len() == 2
            && self
                .owners
                .get(SourceTheoremOwnerId::new(0))
                .is_some_and(|owner| {
                    owner.source_range == range(self.source_id, 19, 138)
                        && owner.spelling == "FormulaStatementComprehensionWitnessSmoke"
                })
            && self
                .statements
                .get(SourceStatementId::new(1))
                .is_some_and(|statement| {
                    statement.source_range == range(self.source_id, 122, 133)
                        && statement.source_ordinal == 2
                })
    }

    fn has_task_258b4_shape(&self) -> bool {
        self.owners.len() == 1
            && self.statements.len() == 1
            && self.contexts.len() == 1
            && self.input_facts.is_empty()
            && self.candidate_facts.len() == 1
            && self.composite_formula_fingerprint.is_some()
            && self.formula_composition_fingerprint.is_some()
    }

    pub(crate) fn is_task_258b4a_profile(&self) -> bool {
        self.has_task_258b4_shape()
            && self.binding_env.contexts().len() == 2
            && self.binding_env.bindings().len() == 1
            && self.binding_env.diagnostics().len() == 4
            && self
                .owners
                .get(SourceTheoremOwnerId::new(0))
                .is_some_and(|owner| {
                    owner.source_range == range(self.source_id, 0, 78)
                        && owner.spelling == "FormulaQuantifierBoundUsePayloadBoundary"
                })
    }

    pub(crate) fn is_task_258b4b_profile(&self) -> bool {
        self.has_task_258b4_shape()
            && self.binding_env.contexts().len() == 2
            && self.binding_env.bindings().len() == 1
            && self.binding_env.diagnostics().len() == 4
            && self
                .owners
                .get(SourceTheoremOwnerId::new(0))
                .is_some_and(|owner| {
                    owner.source_range == range(self.source_id, 0, 165)
                        && owner.spelling == "FormulaConnectiveGroupingPayloadBoundary"
                })
    }

    pub(crate) fn is_task_258b4c_profile(&self) -> bool {
        self.has_task_258b4_shape()
            && self.binding_env.contexts().len() == 4
            && self.binding_env.bindings().len() == 4
            && self.binding_env.diagnostics().is_empty()
            && self
                .owners
                .get(SourceTheoremOwnerId::new(0))
                .is_some_and(|owner| {
                    owner.source_range == range(self.source_id, 19, 137)
                        && owner.spelling == "FormulaNestedQuantifierPayloadBoundary"
                })
    }

    pub fn debug_text(&self) -> String {
        let mut output = String::from("source-statement-debug-v1\n");
        let _ = writeln!(
            output,
            "module: {}::{}",
            self.module_id.package().as_str(),
            self.module_id.path().as_str()
        );
        let _ = writeln!(
            output,
            "binding-env-fingerprint: {:?}",
            self.binding_fingerprint
        );
        let _ = writeln!(
            output,
            "primary-term-fingerprint: {:?}",
            self.primary_term_fingerprint
        );
        let _ = writeln!(
            output,
            "atomic-formula-fingerprint: {:?}",
            self.atomic_formula_fingerprint
        );
        if let Some(composite_formula_fingerprint) = &self.composite_formula_fingerprint {
            let _ = writeln!(
                output,
                "composite-formula-fingerprint: {:?}",
                composite_formula_fingerprint
            );
        }
        if let Some(formula_composition_fingerprint) = &self.formula_composition_fingerprint {
            let _ = writeln!(
                output,
                "formula-composition-fingerprint: {:?}",
                formula_composition_fingerprint
            );
        }
        for (id, row) in self.owners.iter() {
            let _ = writeln!(
                output,
                "owner#{} symbol={:?} contribution={} role={} status={} range={}..{} site={} recovery={} spelling={:?}",
                id.index(),
                row.symbol,
                row.contribution.index(),
                theorem_role_key(row.role),
                theorem_status_key(row.status),
                row.source_range.start,
                row.source_range.end,
                row.site.node().index(),
                statement_recovery_key(row.recovery),
                row.spelling,
            );
        }
        for (id, row) in self.statements.iter() {
            let _ = writeln!(
                output,
                "statement#{} ordinal={} owner={} context={} formula={} kind={} range={}..{} site={} recovery={} spelling={:?}",
                id.index(),
                row.source_ordinal,
                row.owner.index(),
                row.context.index(),
                formula_target_key(row.formula),
                statement_kind_key(row.kind),
                row.source_range.start,
                row.source_range.end,
                row.site.node().index(),
                statement_recovery_key(row.recovery),
                row.spelling,
            );
        }
        for (id, row) in self.contexts.iter() {
            let _ = write!(
                output,
                "context#{} statement={} binding_context={} range={}..{} visible_bindings=",
                id.index(),
                row.statement.index(),
                row.binding_context.index(),
                row.source_range.start,
                row.source_range.end,
            );
            write_dense_ids(&mut output, &row.visible_bindings, BindingId::index);
            output.push('\n');
        }
        for (id, row) in self.input_facts.iter() {
            let _ = write!(
                output,
                "input-fact#{} statement={} context={} ordinal={} kind={} binding={} uses=",
                id.index(),
                row.statement.index(),
                row.context.index(),
                row.ordinal,
                input_fact_kind_key(row.kind),
                row.binding.index(),
            );
            write_dense_ids(&mut output, &row.uses, SourcePrimaryTermReferenceId::index);
            output.push('\n');
        }
        for (id, row) in self.candidate_facts.iter() {
            let _ = writeln!(
                output,
                "candidate-fact#{} statement={} context={} ordinal={} kind={} formula={}",
                id.index(),
                row.statement.index(),
                row.context.index(),
                row.ordinal,
                candidate_fact_kind_key(row.kind),
                formula_target_key(row.formula),
            );
        }
        output
    }

    pub(crate) fn validate_installation(
        &self,
        source_id: SourceId,
        module_id: &ModuleId,
        primary_terms: &SourcePrimaryTermHandoff,
        atomic_formulas: &SourceAtomicFormulaHandoff,
        arena: &TypedArena,
    ) -> Result<(), SourceStatementError> {
        if self.source_id != source_id
            || &self.module_id != module_id
            || self.binding_fingerprint != self.binding_env.debug_text()
            || self.primary_term_fingerprint != primary_terms.debug_text()
            || self.atomic_formula_fingerprint != atomic_formulas.debug_text()
            || self.composite_formula_fingerprint.is_some()
            || self.formula_composition_fingerprint.is_some()
        {
            return Err(SourceStatementError::DependencyMismatch);
        }
        if arena
            .iter()
            .any(|(_, node)| node.kind.as_str() == "TheoremItem")
        {
            let invalid = || SourceStatementError::DependencyMismatch;
            let nested = arena
                .iter()
                .any(|(_, node)| node.kind.as_str() == "CompactStatement");
            let assumption = arena
                .iter()
                .any(|(_, node)| node.kind.as_str() == "AssumptionStatement");
            let witness = arena
                .iter()
                .any(|(_, node)| node.kind.as_str() == "TakeStatement");
            let witness_count = arena
                .iter()
                .filter(|(_, node)| node.kind.as_str() == "Witness")
                .count();
            let named_witness = witness
                && arena
                    .iter()
                    .any(|(_, node)| node.kind.as_str() == "Witness" && node.children.len() == 3);
            let enumerations = arena
                .iter()
                .filter(|(_, node)| node.kind.as_str() == "source.term.set.enumeration")
                .collect::<Vec<_>>();
            let enumeration = match enumerations.as_slice() {
                [] => None,
                [(id, node)] if witness && witness_count == 1 => {
                    let [_, contents @ .., _] = node.children.as_slice() else {
                        return Err(invalid());
                    };
                    if !contents.is_empty() && contents.len() % 2 == 0 {
                        return Err(invalid());
                    }
                    let mut elements = Vec::new();
                    for (index, child) in contents.iter().enumerate() {
                        let expression = arena.node(*child).ok_or_else(invalid)?;
                        if index % 2 == 1 {
                            if expression.kind.as_str()
                                != r#"Token(SurfaceToken { kind: ReservedSymbol, text: "," })"#
                            {
                                return Err(invalid());
                            }
                            continue;
                        }
                        let [site] = expression.children.as_slice() else {
                            return Err(invalid());
                        };
                        let (_, term) = primary_terms
                            .terms()
                            .iter()
                            .find(|(_, term)| term.site().node() == *site)
                            .ok_or_else(invalid)?;
                        if expression.kind.as_str() != "TermExpression"
                            || expression.recovery != NodeRecoveryState::Normal
                            || expression.anchor != SourceAnchor::Range(term.source_range())
                            || term.kind() != SourcePrimaryTermKind::VariableReference
                            || term.parent().is_some()
                        {
                            return Err(invalid());
                        }
                        elements.push(term);
                    }
                    Some((*id, elements))
                }
                _ => return Err(invalid()),
            };
            let choices = arena
                .iter()
                .filter(|(_, node)| node.kind.as_str() == "source.term.set.choice")
                .collect::<Vec<_>>();
            let choice = match choices.as_slice() {
                [] => None,
                [(id, node)] if witness && witness_count == 1 && enumeration.is_none() => {
                    let [keyword, target] = node.children.as_slice() else {
                        return Err(invalid());
                    };
                    let keyword = arena.node(*keyword).ok_or_else(invalid)?;
                    let target = arena.node(*target).ok_or_else(invalid)?;
                    let [head] = target.children.as_slice() else {
                        return Err(invalid());
                    };
                    let head = arena.node(*head).ok_or_else(invalid)?;
                    let [token] = head.children.as_slice() else {
                        return Err(invalid());
                    };
                    let token = arena.node(*token).ok_or_else(invalid)?;
                    let SourceAnchor::Range(range) = node.anchor else {
                        return Err(invalid());
                    };
                    let SourceAnchor::Range(keyword_range) = keyword.anchor else {
                        return Err(invalid());
                    };
                    let SourceAnchor::Range(target_range) = target.anchor else {
                        return Err(invalid());
                    };
                    if [*node, keyword, target, head, token]
                        .iter()
                        .any(|node| node.recovery != NodeRecoveryState::Normal)
                        || target.kind.as_str() != "source.term.set.target-type"
                        || head.kind.as_str() != "source.term.set.target-type-head"
                        || keyword.kind.as_str()
                            != r#"Token(SurfaceToken { kind: ReservedWord, text: "the" })"#
                        || token.kind.as_str()
                            != r#"Token(SurfaceToken { kind: ReservedWord, text: "set" })"#
                        || !keyword.children.is_empty()
                        || !token.children.is_empty()
                        || range.source_id != source_id
                        || keyword_range.source_id != source_id
                        || target_range.source_id != source_id
                        || keyword_range.start != range.start
                        || keyword_range.end.checked_sub(keyword_range.start) != Some(3)
                        || keyword_range.end > target_range.start
                        || target_range.end.checked_sub(target_range.start) != Some(3)
                        || target_range.end != range.end
                        || head.anchor != target.anchor
                        || token.anchor != target.anchor
                    {
                        return Err(invalid());
                    }
                    Some(*id)
                }
                _ => return Err(invalid()),
            };
            let groups = primary_terms
                .terms()
                .iter()
                .filter(|(_, term)| term.kind() == SourcePrimaryTermKind::Parenthesized)
                .collect::<Vec<_>>();
            let group = match groups.as_slice() {
                [] => None,
                [first, ..]
                    if witness
                        && witness_count == 1
                        && !named_witness
                        && enumeration.is_none()
                        && choice.is_none() =>
                {
                    Some(*first)
                }
                _ => return Err(invalid()),
            };
            let descendant = arena.iter().any(|(_, node)| {
                node.kind.as_str() == "ConclusionStatement"
                    && node
                        .children
                        .get(2)
                        .and_then(|id| arena.node(*id))
                        .is_some_and(|node| node.kind.as_str() == "ProofBlock")
            });
            let count = if witness {
                2
            } else if assumption {
                3
            } else if descendant {
                5
            } else if nested {
                4
            } else {
                1
            };
            if arena.root().is_none()
                || self.binding_env.source_id() != source_id
                || self.binding_env.module_id() != module_id
                || primary_terms.source_id() != source_id
                || primary_terms.module_id() != module_id
                || atomic_formulas.source_id() != source_id
                || atomic_formulas.module_id() != module_id
                || atomic_formulas.primary_term_fingerprint() != primary_terms.debug_text()
                || atomic_formulas.application_fingerprint().is_some()
                || atomic_formulas.structure_fingerprint().is_some()
                || atomic_formulas.set_term_fingerprint().is_some()
            {
                return Err(invalid());
            }
            primary_terms
                .validate_installation(source_id, module_id, arena)
                .map_err(|_| invalid())?;
            atomic_formulas
                .validate_installation(source_id, module_id, primary_terms, None, None, None, arena)
                .map_err(|_| invalid())?;
            let [owner] = self.owners.rows.as_slice() else {
                return Err(SourceStatementError::InvalidAggregate);
            };
            if self.statements.len() != count
                || self.contexts.len() != count
                || self.input_facts.len() != count
                || self.candidate_facts.len() != count
            {
                return Err(SourceStatementError::InvalidAggregate);
            }
            let binding = self
                .binding_env
                .bindings()
                .get(BindingId::new(0))
                .ok_or_else(invalid)?;
            let module_context = self
                .binding_env
                .contexts()
                .get(BindingContextId::new(0))
                .ok_or_else(invalid)?;
            let mut proofs = arena
                .iter()
                .filter(|(_, node)| node.kind.as_str() == "ProofBlock")
                .collect::<Vec<_>>();
            proofs.sort_by_key(|(_, node)| match node.anchor {
                SourceAnchor::Range(range) => range.start,
                _ => usize::MAX,
            });
            if self.binding_env.bindings().len() != 1
                || self.binding_env.contexts().len()
                    != if assumption || witness {
                        2
                    } else if descendant {
                        4
                    } else if nested {
                        3
                    } else {
                        1
                    }
                || proofs.len()
                    != if assumption || witness {
                        1
                    } else if descendant {
                        3
                    } else if nested {
                        2
                    } else {
                        0
                    }
                || !self.binding_env.diagnostics().is_empty()
                || binding.kind != BindingKind::ReservedVariable
                || binding.identity
                    != (BinderIdentity::ReservedVariable {
                        spelling: binding.spelling.clone(),
                        declaration_range: binding.declaration_range,
                    })
                || binding.type_site != BindingTypeSite::Missing
                || binding.status != BindingStatus::Active
                || binding.owner_context != module_context.id
                || binding.visible_after_ordinal != 0
                || binding.recovery != BindingRecoveryState::Normal
                || !binding.diagnostics.is_empty()
                || !binding.captured.identities().is_empty()
                || module_context.owner != BindingContextOwner::Module
                || module_context.parent.is_some()
                || module_context.layer != BindingContextLayer::Module
                || module_context.lexical_scope.is_some()
                || module_context.recovery != BindingContextRecovery::Normal
                || module_context.bindings != [binding.id]
                || module_context.visible_bindings != [binding.id]
                || primary_terms.terms().len()
                    != count * 2 + witness_count
                        - usize::from(enumeration.is_some() || choice.is_some())
                        + enumeration
                            .as_ref()
                            .map_or(0, |(_, elements)| elements.len())
                        + groups.len()
                || primary_terms.references().len()
                    != count * 2 + witness_count
                        - usize::from(enumeration.is_some() || choice.is_some())
                        + enumeration
                            .as_ref()
                            .map_or(0, |(_, elements)| elements.len())
                || !primary_terms.numeric_type_requests().is_empty()
                || atomic_formulas.formulas().len() != count
                || atomic_formulas.edges().len() != count * 2
                || atomic_formulas.requests().len() != count * 2
                || !atomic_formulas.wrappers().is_empty()
                || !atomic_formulas.predicate_segments().is_empty()
                || !atomic_formulas.predicate_heads().is_empty()
                || !atomic_formulas.candidates().is_empty()
                || !atomic_formulas.type_sites().is_empty()
                || !atomic_formulas.attributes().is_empty()
            {
                return Err(invalid());
            }
            for (index, (proof_id, proof)) in proofs.iter().enumerate() {
                let parent = proofs
                    .iter()
                    .enumerate()
                    .rfind(|(_, (id, _))| is_descendant(arena, *id, *proof_id))
                    .map_or(BindingContextId::new(0), |(index, _)| {
                        BindingContextId::new(index + 1)
                    });
                let context = self
                    .binding_env
                    .contexts()
                    .get(BindingContextId::new(index + 1))
                    .ok_or_else(invalid)?;
                let SourceAnchor::Range(range) = proof.anchor else {
                    return Err(invalid());
                };
                if context.owner
                    != (BindingContextOwner::SourceStatement {
                        source_range: range,
                    })
                    || context.parent != Some(parent)
                    || context.layer != BindingContextLayer::Proof
                    || context.lexical_scope.is_none()
                    || !context.bindings.is_empty()
                    || context.visible_bindings != [binding.id]
                    || context.recovery != BindingContextRecovery::Normal
                {
                    return Err(invalid());
                }
            }
            let owner_node = arena.node(owner.site.node()).ok_or_else(invalid)?;
            if owner.symbol != *self.checked_owner.symbol()
                || owner.contribution != self.owner_contribution
                || owner.source_range != self.checked_owner.source_range()
                || self.checked_owner.origin().source_id() != source_id
                || self.checked_owner.origin().module_id() != module_id
                || self.checked_owner.origin().import_edge().is_some()
                || self.checked_owner.origin().is_recovered()
                || self.checked_owner.visibility() != Visibility::Public
                || self.checked_owner.export_status() != ExportStatus::Exported
                || owner.role != SourceTheoremRole::Theorem
                || owner.status != SourceTheoremStatus::Unmodified
                || owner.recovery != SourceStatementRecovery::Normal
                || owner_node.kind.as_str() != "TheoremItem"
                || owner.site != TypedSiteRef::Node(owner.site.node())
                || owner_node.anchor != SourceAnchor::Range(owner.source_range)
                || owner_node.recovery != NodeRecoveryState::Normal
                || owner_node.children.len()
                    != if nested || assumption || witness {
                        6
                    } else {
                        5
                    }
            {
                return Err(SourceStatementError::InvalidOwner {
                    owner: SourceTheoremOwnerId::new(0),
                });
            }
            let mut sites = arena
                .iter()
                .filter(|(_, node)| {
                    matches!(
                        node.kind.as_str(),
                        "TheoremItem"
                            | "CompactStatement"
                            | "AssumptionStatement"
                            | "ConclusionStatement"
                    )
                })
                .collect::<Vec<_>>();
            sites.sort_by_key(|(_, node)| match node.anchor {
                SourceAnchor::Range(range) => range.start,
                _ => usize::MAX,
            });
            if sites.len() != count {
                return Err(invalid());
            }
            if witness {
                let (proof_id, proof) = proofs[0];
                let takes = arena
                    .iter()
                    .filter(|(_, node)| node.kind.as_str() == "TakeStatement")
                    .collect::<Vec<_>>();
                let mut items = arena
                    .iter()
                    .filter(|(_, node)| node.kind.as_str() == "Witness")
                    .collect::<Vec<_>>();
                items.sort_by_key(|(_, node)| match node.anchor {
                    SourceAnchor::Range(range) => range.start,
                    _ => usize::MAX,
                });
                let [(take_id, take)] = takes.as_slice() else {
                    return Err(invalid());
                };
                match (take.children.as_slice(), items.as_slice()) {
                    ([_, item, _], [(item_id, _)]) if item == item_id => {}
                    ([_, named, comma, unnamed, _], [(named_id, first), (unnamed_id, second)])
                        if named == named_id
                            && unnamed == unnamed_id
                            && first.children.len() == 3
                            && second.children.len() == 1
                            && arena.node(*comma).is_some_and(|node| {
                                node.kind.as_str()
                                    == r#"Token(SurfaceToken { kind: ReservedSymbol, text: "," })"#
                                    && node.recovery == NodeRecoveryState::Normal
                                    && node.children.is_empty()
                            }) => {}
                    _ => return Err(invalid()),
                }
                if nested
                    || assumption
                    || descendant
                    || sites[0].0 != owner.site.node()
                    || sites[1].1.kind.as_str() != "ConclusionStatement"
                    || owner_node.children[4] != proof_id
                    || proof.children.len() != 4
                    || proof.children[1..3] != [*take_id, sites[1].0]
                    || !is_descendant(arena, owner.site.node(), proof_id)
                {
                    return Err(invalid());
                }
                if choice.is_some() {
                    for (parent, kind, parts) in [
                        (
                            owner_node,
                            "TheoremItem",
                            &[
                                ("ReservedWord", "theorem"),
                                ("Identifier", owner.spelling()),
                                ("ReservedSymbol", ":"),
                                ("", "FormulaExpression"),
                                ("", "ProofBlock"),
                                ("ReservedSymbol", ";"),
                            ][..],
                        ),
                        (
                            proof,
                            "ProofBlock",
                            &[
                                ("ReservedWord", "proof"),
                                ("", "TakeStatement"),
                                ("", "ConclusionStatement"),
                                ("ReservedWord", "end"),
                            ][..],
                        ),
                        (
                            *take,
                            "TakeStatement",
                            &[
                                ("ReservedWord", "take"),
                                ("", "Witness"),
                                ("ReservedSymbol", ";"),
                            ][..],
                        ),
                        (
                            sites[1].1,
                            "ConclusionStatement",
                            &[
                                ("ReservedWord", "thus"),
                                ("", "Proposition"),
                                ("ReservedSymbol", ";"),
                            ][..],
                        ),
                    ] {
                        let SourceAnchor::Range(parent_range) = parent.anchor else {
                            return Err(invalid());
                        };
                        if parent.kind.as_str() != kind
                            || parent.recovery != NodeRecoveryState::Normal
                            || parent_range.source_id != source_id
                            || parent.children.len() != parts.len()
                        {
                            return Err(invalid());
                        }
                        let mut previous_end = parent_range.start;
                        for (index, (child, (token_kind, text))) in
                            parent.children.iter().zip(parts).enumerate()
                        {
                            let child = arena.node(*child).ok_or_else(invalid)?;
                            let SourceAnchor::Range(range) = child.anchor else {
                                return Err(invalid());
                            };
                            let expected = if token_kind.is_empty() {
                                (*text).to_owned()
                            } else {
                                format!(
                                    "Token(SurfaceToken {{ kind: {token_kind}, text: {text:?} }})"
                                )
                            };
                            if child.kind.as_str() != expected
                                || child.recovery != NodeRecoveryState::Normal
                                || range.source_id != source_id
                                || range.start < previous_end
                                || range.start >= range.end
                                || range.end > parent_range.end
                                || (index == 0 && range.start != parent_range.start)
                                || (index + 1 == parts.len() && range.end != parent_range.end)
                                || (!token_kind.is_empty()
                                    && (!child.children.is_empty()
                                        || range.end.checked_sub(range.start) != Some(text.len())))
                            {
                                return Err(invalid());
                            }
                            previous_end = range.end;
                        }
                    }
                }
                for (_, item) in items {
                    let expression = match item.children.as_slice() {
                        [expression] | [_, _, expression] => *expression,
                        _ => return Err(invalid()),
                    };
                    let expression = arena.node(expression).ok_or_else(invalid)?;
                    let [term_id] = expression.children.as_slice() else {
                        return Err(invalid());
                    };
                    let term = arena.node(*term_id).ok_or_else(invalid)?;
                    let SourceAnchor::Range(item_range) = item.anchor else {
                        return Err(invalid());
                    };
                    let SourceAnchor::Range(expression_range) = expression.anchor else {
                        return Err(invalid());
                    };
                    if expression.kind.as_str() != "TermExpression"
                        || !(matches!(
                            term.kind.as_str(),
                            "source.term.variable-reference" | "source.term.parenthesized"
                        ) || enumeration.as_ref().is_some_and(|(id, _)| *id == *term_id)
                            || choice == Some(*term_id))
                        || expression.anchor != term.anchor
                        || item_range.source_id != source_id
                        || expression_range.source_id != source_id
                        || item_range.start > expression_range.start
                        || expression_range.end != item_range.end
                        || (item.children.len() == 1 && item_range != expression_range)
                        || !is_descendant(arena, *take_id, *term_id)
                        || group.is_some_and(|(_, group)| group.site().node() != *term_id)
                    {
                        return Err(invalid());
                    }
                }
                let mut term_sites = arena
                    .iter()
                    .filter(|(_, node)| {
                        matches!(
                            node.kind.as_str(),
                            "source.term.variable-reference" | "source.term.parenthesized"
                        )
                    })
                    .collect::<Vec<_>>();
                term_sites.sort_by_key(|(_, node)| match node.anchor {
                    SourceAnchor::Range(range) => range.start,
                    _ => usize::MAX,
                });
                if term_sites.len() != primary_terms.terms().len() {
                    return Err(invalid());
                }
                let mut reference_ordinal = 0;
                for ((ordinal, (id, term)), (site, node)) in
                    primary_terms.terms().iter().enumerate().zip(term_sites)
                {
                    let reference = primary_terms
                        .references()
                        .iter()
                        .find(|(_, reference)| reference.term() == id);
                    let context = if is_descendant(arena, proof_id, site) {
                        BindingContextId::new(1)
                    } else {
                        BindingContextId::new(0)
                    };
                    let grouped_parent = groups
                        .iter()
                        .find(|(_, group)| {
                            arena
                                .node(group.site().node())
                                .and_then(|group| group.children.get(1))
                                .and_then(|expression| arena.node(*expression))
                                .is_some_and(|expression| {
                                    expression.kind.as_str() == "TermExpression"
                                        && expression.children == [site]
                                })
                        })
                        .map(|(id, _)| *id);
                    if term.kind() == SourcePrimaryTermKind::Parenthesized {
                        let [_, expression, _] = node.children.as_slice() else {
                            return Err(invalid());
                        };
                        let expression = arena.node(*expression).ok_or_else(invalid)?;
                        let mut children = primary_terms
                            .terms()
                            .iter()
                            .filter(|(_, child)| child.parent() == Some(id));
                        let (_, child) = children.next().ok_or_else(invalid)?;
                        if context != BindingContextId::new(1)
                            || reference.is_some()
                            || children.next().is_some()
                            || expression.kind.as_str() != "TermExpression"
                            || expression.children != [child.site().node()]
                            || !matches!(
                                child.kind(),
                                SourcePrimaryTermKind::VariableReference
                                    | SourcePrimaryTermKind::Parenthesized
                            )
                            || term.spelling() != format!("( {} )", child.spelling())
                            || term.parent() != grouped_parent
                        {
                            return Err(invalid());
                        }
                    } else {
                        let (reference_id, reference) = reference.ok_or_else(invalid)?;
                        let scope_matches = if context == BindingContextId::new(0) {
                            reference
                                .lexical_scope()
                                .is_some_and(|scope| scope.path().is_empty())
                        } else {
                            reference.lexical_scope()
                                == self
                                    .binding_env
                                    .contexts()
                                    .get(context)
                                    .ok_or_else(invalid)?
                                    .lexical_scope
                                    .as_ref()
                        };
                        if term.kind() != SourcePrimaryTermKind::VariableReference
                            || term.spelling() != binding.spelling
                            || term.parent() != grouped_parent
                            || reference_id.index() != reference_ordinal
                            || reference.use_ordinal() != reference_ordinal
                            || reference.binding() != binding.id
                            || reference.role() != SourcePrimaryTermReferenceRole::Variable
                            || !scope_matches
                        {
                            return Err(invalid());
                        }
                        reference_ordinal += 1;
                    }
                    if term.site() != &TypedSiteRef::Node(site)
                        || node.anchor != SourceAnchor::Range(term.source_range())
                        || term.source_ordinal() != ordinal
                        || term.context() != context
                        || term.role() != SourcePrimaryTermRole::Value
                        || term.recovery() != SourcePrimaryTermRecovery::Normal
                    {
                        return Err(invalid());
                    }
                }
            }
            if assumption {
                let (proof_id, proof) = proofs[0];
                if nested
                    || descendant
                    || sites[0].0 != owner.site.node()
                    || sites[1].1.kind.as_str() != "AssumptionStatement"
                    || sites[2].1.kind.as_str() != "ConclusionStatement"
                    || owner_node.children[4] != proof_id
                    || proof.children.len() != 4
                    || proof.children[1..3] != [sites[1].0, sites[2].0]
                    || !is_descendant(arena, owner.site.node(), proof_id)
                {
                    return Err(invalid());
                }
            }
            for (index, ((site_id, site), (formula_id, formula))) in sites
                .into_iter()
                .zip(atomic_formulas.formulas().iter())
                .enumerate()
            {
                let statement_id = SourceStatementId::new(index);
                let context_id = SourceStatementContextId::new(index);
                let target = SourceStatementFormulaTarget::Atomic(formula_id);
                let expected_context = proofs
                    .iter()
                    .enumerate()
                    .rfind(|(_, (proof, _))| is_descendant(arena, *proof, formula.site().node()))
                    .map_or(BindingContextId::new(0), |(index, _)| {
                        BindingContextId::new(index + 1)
                    });
                let context = self
                    .binding_env
                    .contexts()
                    .get(expected_context)
                    .ok_or_else(invalid)?;
                let statement = &self.statements.rows[index];
                let SourceAnchor::Range(range) = site.anchor else {
                    return Err(invalid());
                };
                let kind = match site.kind.as_str() {
                    "TheoremItem" => SourceStatementKind::TheoremProposition,
                    "CompactStatement" => SourceStatementKind::ProofStepProposition,
                    "AssumptionStatement" => SourceStatementKind::Assumption,
                    _ => SourceStatementKind::Conclusion,
                };
                if assumption || witness {
                    let expression = if index == 0 {
                        site.children[3]
                    } else {
                        let [_, proposition, _] = site.children.as_slice() else {
                            return Err(invalid());
                        };
                        let proposition = arena.node(*proposition).ok_or_else(invalid)?;
                        let [expression] = proposition.children.as_slice() else {
                            return Err(invalid());
                        };
                        if proposition.kind.as_str() != "Proposition" {
                            return Err(invalid());
                        }
                        *expression
                    };
                    let expression = arena.node(expression).ok_or_else(invalid)?;
                    // The required witness companion joins the authenticated name below.
                    let expected_spelling = if index == 0 && named_witness {
                        None
                    } else {
                        Some(match index {
                            0 if witness => format!(
                                "theorem {} : {} proof take {} ; thus {} ; end ;",
                                owner.spelling,
                                formula.spelling(),
                                if let Some((_, elements)) = &enumeration {
                                    if elements.is_empty() {
                                        "{ }".to_owned()
                                    } else {
                                        format!(
                                            "{{ {} }}",
                                            elements
                                                .iter()
                                                .map(|term| term.spelling())
                                                .collect::<Vec<_>>()
                                                .join(" , ")
                                        )
                                    }
                                } else if choice.is_some() {
                                    "the set".to_owned()
                                } else {
                                    group
                                        .map_or(binding.spelling.as_str(), |(_, term)| {
                                            term.spelling()
                                        })
                                        .to_owned()
                                },
                                formula.spelling()
                            ),
                            0 => format!(
                                "theorem {} : {} proof assume {} ; thus {} ; end ;",
                                owner.spelling,
                                formula.spelling(),
                                formula.spelling(),
                                formula.spelling()
                            ),
                            1 if !witness => format!("assume {} ;", formula.spelling()),
                            _ => format!("thus {} ;", formula.spelling()),
                        })
                    };
                    if expression.kind.as_str() != "FormulaExpression"
                        || expression.anchor != SourceAnchor::Range(formula.source_range())
                        || expression.children != [formula.site().node()]
                        || expected_spelling.is_some_and(|expected| statement.spelling != expected)
                        || (index != 0 && !is_descendant(arena, proofs[0].0, site_id))
                    {
                        return Err(invalid());
                    }
                }
                let source_ordinal = index + arena.iter().filter(|(_, node)| {
                    node.kind.as_str() == "TakeStatement"
                        && matches!(node.anchor, SourceAnchor::Range(take) if take.start < range.start)
                }).count();
                if formula.kind() != SourceAtomicFormulaKind::Equality
                    || formula.source_ordinal() != index
                    || formula.context() != expected_context
                    || formula.spelling() != format!("{} = {}", binding.spelling, binding.spelling)
                    || !is_descendant(arena, site_id, formula.site().node())
                    || statement.owner != SourceTheoremOwnerId::new(0)
                    || statement.context != context_id
                    || statement.formula != target
                    || statement.site != TypedSiteRef::Node(site_id)
                    || statement.source_range != range
                    || statement.source_ordinal != source_ordinal
                    || statement.kind != kind
                    || statement.recovery != SourceStatementRecovery::Normal
                    || statement.spelling.is_empty()
                    || (!nested
                        && !assumption
                        && !witness
                        && statement.spelling
                            != format!("theorem {} : {} ;", owner.spelling, formula.spelling()))
                {
                    return Err(SourceStatementError::InvalidStatement {
                        statement: statement_id,
                    });
                }
                if self.contexts.rows[index]
                    != (SourceStatementContext {
                        statement: statement_id,
                        binding_context: expected_context,
                        source_range: range,
                        visible_bindings: context.visible_bindings.clone(),
                    })
                {
                    return Err(SourceStatementError::InvalidContext {
                        context: context_id,
                    });
                }
                let uses = atomic_formulas
                    .edges()
                    .iter()
                    .filter(|(_, edge)| edge.formula() == formula_id)
                    .enumerate()
                    .map(|(ordinal, (_, edge))| {
                        let SourceAtomicTermTarget::Primary(term_id) = edge.target() else {
                            return Err(invalid());
                        };
                        let term = primary_terms.terms().get(term_id).ok_or_else(invalid)?;
                        let (id, reference) = primary_terms
                            .references()
                            .iter()
                            .find(|(_, reference)| reference.term() == term_id)
                            .ok_or_else(invalid)?;
                        let scope_matches = if expected_context == BindingContextId::new(0) {
                            reference
                                .lexical_scope()
                                .is_some_and(|scope| scope.path().is_empty())
                        } else {
                            reference.lexical_scope() == context.lexical_scope.as_ref()
                        };
                        if assumption || witness {
                            let formula_node =
                                arena.node(formula.site().node()).ok_or_else(invalid)?;
                            let operand = formula_node
                                .children
                                .get(ordinal * 2)
                                .and_then(|id| arena.node(*id))
                                .ok_or_else(invalid)?;
                            if formula_node.children.len() != 3
                                || operand.kind.as_str() != "TermExpression"
                                || operand.children != [term.site().node()]
                            {
                                return Err(invalid());
                            }
                        }
                        if reference.binding() != binding.id
                            || (!witness && reference.use_ordinal() != index * 2 + ordinal)
                            || !scope_matches
                            || reference.role() != SourcePrimaryTermReferenceRole::Variable
                            || term.context() != expected_context
                            || term.spelling() != binding.spelling
                            || term.kind() != SourcePrimaryTermKind::VariableReference
                            || term.parent().is_some()
                            || edge.ordinal() != ordinal
                            || edge.role()
                                != if ordinal == 0 {
                                    SourceAtomicEdgeRole::BuiltinLeftOperand
                                } else {
                                    SourceAtomicEdgeRole::BuiltinRightOperand
                                }
                        {
                            return Err(invalid());
                        }
                        Ok(id)
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                if self.input_facts.rows[index]
                    != (SourceStatementInputFact {
                        statement: statement_id,
                        context: context_id,
                        ordinal: 0,
                        kind: SourceStatementInputFactKind::ReservedTypeGuard,
                        binding: binding.id,
                        uses,
                    })
                {
                    return Err(SourceStatementError::InvalidInputFact {
                        fact: SourceStatementInputFactId::new(index),
                    });
                }
                if self.candidate_facts.rows[index]
                    != (SourceStatementCandidateFact {
                        statement: statement_id,
                        context: context_id,
                        ordinal: 0,
                        kind: SourceStatementCandidateFactKind::UnverifiedProposition,
                        formula: target,
                    })
                {
                    return Err(SourceStatementError::InvalidCandidateFact {
                        fact: SourceStatementCandidateFactId::new(index),
                    });
                }
            }
            return Ok(());
        }
        let profile = validate_dependencies(
            source_id,
            module_id,
            &self.binding_env,
            primary_terms,
            atomic_formulas,
            arena,
        )?;
        validate_aggregate_lengths(
            profile,
            self.owners.len(),
            self.statements.len(),
            self.contexts.len(),
            self.input_facts.len(),
            self.candidate_facts.len(),
        )?;
        validate_owner_rows(
            profile,
            self.source_id,
            &self.module_id,
            &self.owners,
            &self.checked_owner,
            self.owner_contribution,
            arena,
        )?;
        validate_statement_rows(
            profile,
            self.source_id,
            &self.statements,
            &self.owners,
            &self.contexts,
            atomic_formulas,
            arena,
        )?;
        validate_context_rows(
            profile,
            self.source_id,
            &self.contexts,
            &self.statements,
            &self.binding_env,
        )?;
        validate_input_fact_rows(
            profile,
            &self.input_facts,
            &self.statements,
            &self.contexts,
            &self.binding_env,
            primary_terms,
        )?;
        validate_candidate_fact_rows(
            profile,
            &self.candidate_facts,
            &self.statements,
            &self.contexts,
            atomic_formulas,
        )
    }

    #[allow(clippy::too_many_arguments)] // Rationale: revalidate the complete frozen paired lower-family transaction.
    pub(crate) fn validate_installation_with_formula_composition(
        &self,
        source_id: SourceId,
        module_id: &ModuleId,
        primary_terms: &SourcePrimaryTermHandoff,
        atomic_formulas: &SourceAtomicFormulaHandoff,
        composite_formulas: &SourceCompositeFormulaHandoff,
        formula_composition: &SourceFormulaCompositionHandoff,
        arena: &TypedArena,
    ) -> Result<(), SourceStatementError> {
        if self.source_id != source_id
            || &self.module_id != module_id
            || self.binding_fingerprint != self.binding_env.debug_text()
            || self.primary_term_fingerprint != primary_terms.debug_text()
            || self.atomic_formula_fingerprint != atomic_formulas.debug_text()
            || self.composite_formula_fingerprint.as_deref()
                != Some(composite_formulas.debug_text().as_str())
            || self.formula_composition_fingerprint.as_deref()
                != Some(formula_composition.debug_text().as_str())
        {
            return Err(SourceStatementError::DependencyMismatch);
        }
        let profile = task_258b4_profile(self).ok_or(SourceStatementError::DependencyMismatch)?;
        validate_task_258b4_dependencies(
            profile,
            source_id,
            module_id,
            &self.binding_env,
            primary_terms,
            atomic_formulas,
            composite_formulas,
            formula_composition,
            arena,
        )?;
        validate_task_258b4_rows(
            profile,
            self.source_id,
            &self.module_id,
            &self.owners,
            &self.checked_owner,
            self.owner_contribution,
            &self.statements,
            &self.contexts,
            &self.input_facts,
            &self.candidate_facts,
            composite_formulas,
            arena,
        )
    }
}

impl SourceStatementWitnessHandoff {
    pub const fn source_id(&self) -> SourceId {
        self.source_id
    }

    pub const fn module_id(&self) -> &ModuleId {
        &self.module_id
    }

    pub fn statement_fingerprint(&self) -> &str {
        &self.statement_fingerprint
    }

    pub fn primary_term_fingerprint(&self) -> &str {
        &self.primary_term_fingerprint
    }

    pub fn application_fingerprint(&self) -> Option<&str> {
        self.application_fingerprint.as_deref()
    }

    pub fn structure_fingerprint(&self) -> Option<&str> {
        self.structure_fingerprint.as_deref()
    }

    pub fn set_term_fingerprint(&self) -> Option<&str> {
        self.set_term_fingerprint.as_deref()
    }

    pub const fn witnesses(&self) -> &SourceStatementWitnessTable {
        &self.witnesses
    }

    pub const fn names(&self) -> &SourceStatementWitnessNameTable {
        &self.names
    }

    pub fn debug_text(&self) -> String {
        let mut output = String::from("source-statement-witness-debug-v1\n");
        let _ = writeln!(
            output,
            "module: {}::{}",
            self.module_id.package().as_str(),
            self.module_id.path().as_str()
        );
        let _ = writeln!(
            output,
            "statement-fingerprint: {:?}",
            self.statement_fingerprint
        );
        let _ = writeln!(
            output,
            "primary-term-fingerprint: {:?}",
            self.primary_term_fingerprint
        );
        if let Some(application_fingerprint) = &self.application_fingerprint {
            let _ = writeln!(
                output,
                "application-fingerprint: Some({application_fingerprint:?})"
            );
        }
        if let Some(structure_fingerprint) = &self.structure_fingerprint {
            let _ = writeln!(
                output,
                "structure-fingerprint: Some({structure_fingerprint:?})"
            );
        }
        if let Some(set_term_fingerprint) = &self.set_term_fingerprint {
            let _ = writeln!(
                output,
                "set-term-fingerprint: Some({set_term_fingerprint:?})"
            );
        }
        for (id, row) in self.witnesses.iter() {
            let mut line = format!(
                "witness#{} owner={} binding_context={} term={} take_range={}..{} take_site={} range={}..{} site={} source_ordinal={} ordinal={} kind={} recovery={} spelling={:?}",
                id.index(),
                row.owner.index(),
                row.binding_context.index(),
                witness_term_target_key(row.term),
                row.take_range.start,
                row.take_range.end,
                row.take_site.node().index(),
                row.source_range.start,
                row.source_range.end,
                row.site.node().index(),
                row.source_ordinal,
                row.ordinal,
                witness_kind_key(row.kind),
                statement_recovery_key(row.recovery),
                row.spelling,
            );
            if let Some(name) = row.name {
                let _ = write!(line, " name={}", name.index());
            }
            let _ = writeln!(output, "{line}");
        }
        for (id, row) in self.names.iter() {
            let _ = writeln!(
                output,
                "witness-name#{} witness={} range={}..{} site={} recovery={} spelling={:?}",
                id.index(),
                row.witness.index(),
                row.source_range.start,
                row.source_range.end,
                row.site.node().index(),
                statement_recovery_key(row.recovery),
                row.spelling,
            );
        }
        output
    }

    pub(crate) fn validate_installation(
        &self,
        source_id: SourceId,
        module_id: &ModuleId,
        statements: &SourceStatementHandoff,
        primary_terms: &SourcePrimaryTermHandoff,
        arena: &TypedArena,
    ) -> Result<(), SourceStatementWitnessError> {
        self.validate_installation_with_application(
            source_id,
            module_id,
            statements,
            primary_terms,
            None,
            arena,
        )
    }

    pub(crate) fn validate_installation_with_application(
        &self,
        source_id: SourceId,
        module_id: &ModuleId,
        statements: &SourceStatementHandoff,
        primary_terms: &SourcePrimaryTermHandoff,
        application: Option<&SourceFunctorApplicationHandoff>,
        arena: &TypedArena,
    ) -> Result<(), SourceStatementWitnessError> {
        self.validate_installation_inner(
            source_id,
            module_id,
            statements,
            primary_terms,
            application,
            None,
            None,
            arena,
        )
    }

    pub(crate) fn validate_installation_with_structure(
        &self,
        source_id: SourceId,
        module_id: &ModuleId,
        statements: &SourceStatementHandoff,
        primary_terms: &SourcePrimaryTermHandoff,
        structure: Option<&SourceStructureHandoff>,
        arena: &TypedArena,
    ) -> Result<(), SourceStatementWitnessError> {
        self.validate_installation_inner(
            source_id,
            module_id,
            statements,
            primary_terms,
            None,
            structure,
            None,
            arena,
        )
    }

    pub(crate) fn validate_installation_with_set_term(
        &self,
        source_id: SourceId,
        module_id: &ModuleId,
        statements: &SourceStatementHandoff,
        primary_terms: &SourcePrimaryTermHandoff,
        set_terms: Option<&SourceSetTermHandoff>,
        arena: &TypedArena,
    ) -> Result<(), SourceStatementWitnessError> {
        self.validate_installation_inner(
            source_id,
            module_id,
            statements,
            primary_terms,
            None,
            None,
            set_terms,
            arena,
        )
    }

    #[allow(clippy::too_many_arguments)] // Rationale: validate the complete mutually exclusive lower-family tuple.
    fn validate_installation_inner(
        &self,
        source_id: SourceId,
        module_id: &ModuleId,
        statements: &SourceStatementHandoff,
        primary_terms: &SourcePrimaryTermHandoff,
        application: Option<&SourceFunctorApplicationHandoff>,
        structure: Option<&SourceStructureHandoff>,
        set_terms: Option<&SourceSetTermHandoff>,
        arena: &TypedArena,
    ) -> Result<(), SourceStatementWitnessError> {
        if arena
            .iter()
            .any(|(_, node)| node.kind.as_str() == "TheoremItem")
        {
            let invalid = || SourceStatementWitnessError::DependencyMismatch;
            if arena.root().is_none()
                || self.source_id != source_id
                || &self.module_id != module_id
                || statements.source_id() != source_id
                || statements.module_id() != module_id
                || primary_terms.source_id() != source_id
                || primary_terms.module_id() != module_id
                || self.statement_fingerprint != statements.debug_text()
                || self.primary_term_fingerprint != primary_terms.debug_text()
                || statements.primary_term_fingerprint() != self.primary_term_fingerprint
                || self.application_fingerprint.is_some()
                || self.structure_fingerprint.is_some()
                || application.is_some()
                || structure.is_some()
                || statements.owners().len() != 1
                || statements.statements().len() != 2
            {
                return Err(invalid());
            }
            primary_terms
                .validate_installation(source_id, module_id, arena)
                .map_err(|_| invalid())?;
            let set_term = match (set_terms, self.set_term_fingerprint.as_deref()) {
                (None, None) => None,
                (Some(set), Some(fingerprint)) => {
                    set.validate_installation(
                        source_id,
                        module_id,
                        primary_terms,
                        None,
                        None,
                        arena,
                    )
                    .map_err(|_| invalid())?;
                    let (id, term) = set.terms().iter().next().ok_or_else(invalid)?;
                    let choice = term.kind() == SourceSetTermKind::Choice;
                    let request = set.requests().iter().next().ok_or_else(invalid)?.1;
                    let mut elements = Vec::new();
                    for (ordinal, (_, edge)) in set.edges().iter().enumerate() {
                        let SourceSetTarget::Primary(primary_id) = edge.target() else {
                            return Err(invalid());
                        };
                        if edge.term() != id
                            || edge.ordinal() != ordinal
                            || edge.role() != SourceSetEdgeRole::EnumerationElement
                        {
                            return Err(invalid());
                        }
                        elements.push((
                            primary_id,
                            primary_terms.terms().get(primary_id).ok_or_else(invalid)?,
                        ));
                    }
                    if fingerprint != set.debug_text()
                        || set.terms().len() != 1
                        || !matches!(
                            term.kind(),
                            SourceSetTermKind::Enumeration | SourceSetTermKind::Choice
                        )
                        || term.recovery() != SourceSetTermRecovery::Normal
                        || term.source_ordinal() != 0
                        || term.spelling()
                            != if choice {
                                "the set".to_owned()
                            } else {
                                if elements.is_empty() {
                                    "{ }".to_owned()
                                } else {
                                    format!(
                                        "{{ {} }}",
                                        elements
                                            .iter()
                                            .map(|(_, child)| child.spelling())
                                            .collect::<Vec<_>>()
                                            .join(" , ")
                                    )
                                }
                            }
                        || term.context() != BindingContextId::new(1)
                        || !set.wrappers().is_empty()
                        || !set.generators().is_empty()
                        || !set.conditions().is_empty()
                        || set.requests().len() != if choice { 2 } else { 1 }
                        || request.term() != id
                        || request.ordinal() != 0
                        || request.kind()
                            != if choice {
                                SourceSetRequestKind::ChoiceNonempty
                            } else {
                                SourceSetRequestKind::ResultType
                            }
                        || request.generator().is_some()
                        || request.type_site() != choice.then_some(SourceSetTypeSiteId::new(0))
                        || self.witnesses.len() != 1
                        || primary_terms.terms().len() != 4 + elements.len()
                        || primary_terms.references().len() != 4 + elements.len()
                    {
                        return Err(invalid());
                    }
                    if choice {
                        let (_, target) = set.type_sites().iter().next().ok_or_else(invalid)?;
                        let (_, result) = set.requests().iter().nth(1).ok_or_else(invalid)?;
                        if id != SourceSetTermId::new(0)
                            || !set.edges().is_empty()
                            || set.type_sites().len() != 1
                            || target.owner()
                                != (SourceSetTypeOwner::Term {
                                    term: id,
                                    role: SourceSetTypeRole::ChoiceTarget,
                                })
                            || target.head() != SourceSetTypeHead::BuiltinSet
                            || target.context() != term.context()
                            || target.recovery() != SourceSetTermRecovery::Normal
                            || target.spelling() != "set"
                            || target.head_spelling() != "set"
                            || result.term() != id
                            || result.ordinal() != 1
                            || result.kind() != SourceSetRequestKind::ResultType
                            || result.generator().is_some()
                            || result.type_site().is_some()
                        {
                            return Err(invalid());
                        }
                    } else if !set.type_sites().is_empty() {
                        return Err(invalid());
                    }
                    Some((id, term, elements))
                }
                _ => return Err(invalid()),
            };
            let takes = arena
                .iter()
                .filter(|(_, node)| node.kind.as_str() == "TakeStatement")
                .collect::<Vec<_>>();
            let mut items = arena
                .iter()
                .filter(|(_, node)| node.kind.as_str() == "Witness")
                .collect::<Vec<_>>();
            items.sort_by_key(|(_, node)| match node.anchor {
                SourceAnchor::Range(range) => range.start,
                _ => usize::MAX,
            });
            let proofs = arena
                .iter()
                .filter(|(_, node)| node.kind.as_str() == "ProofBlock")
                .collect::<Vec<_>>();
            let [(take_id, take)] = takes.as_slice() else {
                return Err(invalid());
            };
            let [(proof_id, proof)] = proofs.as_slice() else {
                return Err(invalid());
            };
            match (take.children.as_slice(), items.as_slice()) {
                ([_, item, _], [(item_id, _)]) if item == item_id => {}
                ([_, named, comma, unnamed, _], [(named_id, first), (unnamed_id, second)])
                    if named == named_id
                        && unnamed == unnamed_id
                        && first.children.len() == 3
                        && second.children.len() == 1
                        && arena.node(*comma).is_some_and(|node| {
                            node.kind.as_str()
                                == r#"Token(SurfaceToken { kind: ReservedSymbol, text: "," })"#
                                && node.recovery == NodeRecoveryState::Normal
                                && node.children.is_empty()
                        }) => {}
                _ => return Err(invalid()),
            }
            if self.witnesses.len() != items.len()
                || self.names.len()
                    != items
                        .iter()
                        .filter(|(_, item)| item.children.len() == 3)
                        .count()
            {
                return Err(invalid());
            }
            for (ordinal, (item_id, item)) in items.iter().enumerate() {
                let (name_site, expression_id) = match item.children.as_slice() {
                    [expression] => (None, *expression),
                    [name, equals, expression] => (Some((*name, *equals)), *expression),
                    _ => return Err(invalid()),
                };
                let expression = arena.node(expression_id).ok_or_else(invalid)?;
                let [term_site] = expression.children.as_slice() else {
                    return Err(invalid());
                };
                let (target, binding_context, term_range, term_spelling, reference) =
                    if let Some((id, term, _)) = set_term
                        .as_ref()
                        .filter(|(_, term, _)| term.kind() == SourceSetTermKind::Choice)
                    {
                        let node = arena.node(*term_site).ok_or_else(invalid)?;
                        let [keyword, target_id] = node.children.as_slice() else {
                            return Err(invalid());
                        };
                        let keyword = arena.node(*keyword).ok_or_else(invalid)?;
                        let target = arena.node(*target_id).ok_or_else(invalid)?;
                        let [head_id] = target.children.as_slice() else {
                            return Err(invalid());
                        };
                        let head = arena.node(*head_id).ok_or_else(invalid)?;
                        let [token] = head.children.as_slice() else {
                            return Err(invalid());
                        };
                        let token = arena.node(*token).ok_or_else(invalid)?;
                        let (_, type_site) = set_terms
                            .ok_or_else(invalid)?
                            .type_sites()
                            .iter()
                            .next()
                            .ok_or_else(invalid)?;
                        let SourceAnchor::Range(keyword_range) = keyword.anchor else {
                            return Err(invalid());
                        };
                        let range = term.source_range();
                        let target_range = type_site.source_range();
                        if term.site() != &TypedSiteRef::Node(*term_site)
                            || node.kind.as_str() != "source.term.set.choice"
                            || node.anchor != SourceAnchor::Range(range)
                            || [node, keyword, target, head, token]
                                .iter()
                                .any(|node| node.recovery != NodeRecoveryState::Normal)
                            || target.kind.as_str() != "source.term.set.target-type"
                            || head.kind.as_str() != "source.term.set.target-type-head"
                            || type_site.site() != &TypedSiteRef::Node(*target_id)
                            || type_site.head_site() != &TypedSiteRef::Node(*head_id)
                            || target.anchor != SourceAnchor::Range(target_range)
                            || head.anchor != SourceAnchor::Range(type_site.head_range())
                            || head.anchor != target.anchor
                            || token.anchor != target.anchor
                            || keyword.kind.as_str()
                                != r#"Token(SurfaceToken { kind: ReservedWord, text: "the" })"#
                            || token.kind.as_str()
                                != r#"Token(SurfaceToken { kind: ReservedWord, text: "set" })"#
                            || !keyword.children.is_empty()
                            || !token.children.is_empty()
                            || range.source_id != source_id
                            || keyword_range.source_id != source_id
                            || target_range.source_id != source_id
                            || keyword_range.start != range.start
                            || keyword_range.end.checked_sub(keyword_range.start) != Some(3)
                            || keyword_range.end > target_range.start
                            || target_range.end.checked_sub(target_range.start) != Some(3)
                            || target_range.end != range.end
                        {
                            return Err(invalid());
                        }
                        (
                            SourceStatementWitnessTermTarget::SetTerm(*id),
                            term.context(),
                            range,
                            term.spelling(),
                            None,
                        )
                    } else if let Some((id, term, elements)) = &set_term {
                        let node = arena.node(*term_site).ok_or_else(invalid)?;
                        let [open, contents @ .., close] = node.children.as_slice() else {
                            return Err(invalid());
                        };
                        if contents.len() != elements.len().saturating_mul(2).saturating_sub(1) {
                            return Err(invalid());
                        }
                        let open = arena.node(*open).ok_or_else(invalid)?;
                        let close = arena.node(*close).ok_or_else(invalid)?;
                        let SourceAnchor::Range(open_range) = open.anchor else {
                            return Err(invalid());
                        };
                        let SourceAnchor::Range(close_range) = close.anchor else {
                            return Err(invalid());
                        };
                        if term.site().node() != *term_site
                            || node.kind.as_str() != "source.term.set.enumeration"
                            || node.anchor != SourceAnchor::Range(term.source_range())
                            || node.recovery != NodeRecoveryState::Normal
                            || open.kind.as_str()
                                != r#"Token(SurfaceToken { kind: ReservedSymbol, text: "{" })"#
                            || close.kind.as_str()
                                != r#"Token(SurfaceToken { kind: ReservedSymbol, text: "}" })"#
                            || open.recovery != NodeRecoveryState::Normal
                            || close.recovery != NodeRecoveryState::Normal
                            || !open.children.is_empty()
                            || !close.children.is_empty()
                            || open_range.source_id != source_id
                            || close_range.source_id != source_id
                            || open_range.start != term.source_range().start
                            || open_range.end.checked_sub(open_range.start) != Some(1)
                            || open_range.end > close_range.start
                            || close_range.end.checked_sub(close_range.start) != Some(1)
                            || close_range.end != term.source_range().end
                        {
                            return Err(invalid());
                        }
                        let mut previous_end = open_range.end;
                        let mut element_reference = None;
                        for (ordinal, (primary_id, child)) in elements.iter().enumerate() {
                            let expression =
                                arena.node(contents[ordinal * 2]).ok_or_else(invalid)?;
                            let child_node = arena.node(child.site().node()).ok_or_else(invalid)?;
                            let (reference_id, reference) = primary_terms
                                .references()
                                .iter()
                                .find(|(_, reference)| reference.term() == *primary_id)
                                .ok_or_else(invalid)?;
                            let context = statements
                                .binding_env()
                                .contexts()
                                .get(term.context())
                                .ok_or_else(invalid)?;
                            let binding = statements
                                .binding_env()
                                .bindings()
                                .get(reference.binding())
                                .ok_or_else(invalid)?;
                            let range = child.source_range();
                            if expression.kind.as_str() != "TermExpression"
                                || expression.children != [child.site().node()]
                                || expression.anchor != SourceAnchor::Range(range)
                                || expression.recovery != NodeRecoveryState::Normal
                                || child_node.kind.as_str() != "source.term.variable-reference"
                                || child_node.anchor != SourceAnchor::Range(range)
                                || child_node.recovery != NodeRecoveryState::Normal
                                || child.site() != &TypedSiteRef::Node(child.site().node())
                                || child.kind() != SourcePrimaryTermKind::VariableReference
                                || child.role() != SourcePrimaryTermRole::Value
                                || child.recovery() != SourcePrimaryTermRecovery::Normal
                                || child.parent().is_some()
                                || child.context() != term.context()
                                || child.source_ordinal() != ordinal + 2
                                || range.source_id != source_id
                                || range.start < previous_end
                                || range.end > close_range.start
                                || range.start >= range.end
                                || reference_id.index() != ordinal + 2
                                || reference.use_ordinal() != ordinal + 2
                                || reference.binding() != BindingId::new(0)
                                || reference.role() != SourcePrimaryTermReferenceRole::Variable
                                || reference.lexical_scope() != context.lexical_scope.as_ref()
                                || !context.visible_bindings.contains(&binding.id)
                                || child.spelling() != binding.spelling
                                || !is_descendant(arena, *term_site, child.site().node())
                                || statements
                                    .input_facts()
                                    .iter()
                                    .any(|(_, fact)| fact.uses().contains(&reference_id))
                            {
                                return Err(invalid());
                            }
                            previous_end = range.end;
                            if ordinal + 1 < elements.len() {
                                let comma =
                                    arena.node(contents[ordinal * 2 + 1]).ok_or_else(invalid)?;
                                let SourceAnchor::Range(comma_range) = comma.anchor else {
                                    return Err(invalid());
                                };
                                if comma.kind.as_str()
                                    != r#"Token(SurfaceToken { kind: ReservedSymbol, text: "," })"#
                                    || comma.recovery != NodeRecoveryState::Normal
                                    || !comma.children.is_empty()
                                    || comma_range.source_id != source_id
                                    || comma_range.start < previous_end
                                    || comma_range.end.checked_sub(comma_range.start) != Some(1)
                                    || comma_range.end > close_range.start
                                {
                                    return Err(invalid());
                                }
                                previous_end = comma_range.end;
                            }
                            element_reference = Some((reference_id, child.spelling()));
                        }
                        (
                            SourceStatementWitnessTermTarget::SetTerm(*id),
                            term.context(),
                            term.source_range(),
                            term.spelling(),
                            element_reference,
                        )
                    } else {
                        let (term_id, term) = primary_terms
                            .terms()
                            .iter()
                            .find(|(_, term)| term.site().node() == *term_site)
                            .ok_or_else(invalid)?;
                        let (mut reference_term_id, mut reference_term) = (term_id, term);
                        while reference_term.kind() == SourcePrimaryTermKind::Parenthesized {
                            let node = arena
                                .node(reference_term.site().node())
                                .ok_or_else(invalid)?;
                            let [_, child_expression, _] = node.children.as_slice() else {
                                return Err(invalid());
                            };
                            let child_expression =
                                arena.node(*child_expression).ok_or_else(invalid)?;
                            let mut children = primary_terms
                                .terms()
                                .iter()
                                .filter(|(_, child)| child.parent() == Some(reference_term_id));
                            let child = children.next().ok_or_else(invalid)?;
                            if name_site.is_some()
                                || items.len() != 1
                                || children.next().is_some()
                                || node.kind.as_str() != "source.term.parenthesized"
                                || child_expression.kind.as_str() != "TermExpression"
                                || child_expression.children != [child.1.site().node()]
                                || !matches!(
                                    child.1.kind(),
                                    SourcePrimaryTermKind::VariableReference
                                        | SourcePrimaryTermKind::Parenthesized
                                )
                                || child.1.context() != term.context()
                                || child.1.role() != SourcePrimaryTermRole::Value
                                || child.1.recovery() != SourcePrimaryTermRecovery::Normal
                                || reference_term.spelling()
                                    != format!("( {} )", child.1.spelling())
                                || primary_terms
                                    .references()
                                    .iter()
                                    .any(|(_, reference)| reference.term() == reference_term_id)
                            {
                                return Err(invalid());
                            }
                            (reference_term_id, reference_term) = child;
                        }
                        let (reference_id, reference) = primary_terms
                            .references()
                            .iter()
                            .find(|(_, reference)| reference.term() == reference_term_id)
                            .ok_or_else(invalid)?;
                        let context = statements
                            .binding_env()
                            .contexts()
                            .get(term.context())
                            .ok_or_else(invalid)?;
                        if !matches!(
                            term.kind(),
                            SourcePrimaryTermKind::VariableReference
                                | SourcePrimaryTermKind::Parenthesized
                        ) || term.role() != SourcePrimaryTermRole::Value
                            || term.parent().is_some()
                            || term.recovery() != SourcePrimaryTermRecovery::Normal
                            || reference.lexical_scope() != context.lexical_scope.as_ref()
                            || reference.use_ordinal() != reference_id.index()
                            || reference.role() != SourcePrimaryTermReferenceRole::Variable
                            || !context.visible_bindings.contains(&reference.binding())
                        {
                            return Err(invalid());
                        }
                        (
                            SourceStatementWitnessTermTarget::Primary(term_id),
                            term.context(),
                            term.source_range(),
                            term.spelling(),
                            Some((reference_id, reference_term.spelling())),
                        )
                    };
                let context = statements
                    .binding_env()
                    .contexts()
                    .get(binding_context)
                    .ok_or_else(invalid)?;
                let SourceAnchor::Range(take_range) = take.anchor else {
                    return Err(invalid());
                };
                let SourceAnchor::Range(item_range) = item.anchor else {
                    return Err(invalid());
                };
                let SourceAnchor::Range(expression_range) = expression.anchor else {
                    return Err(invalid());
                };
                let name = match name_site {
                    Some((name_site, equals_site)) => {
                        let [name] = self.names.rows.as_slice() else {
                            return Err(invalid());
                        };
                        let token = arena.node(name_site).ok_or_else(invalid)?;
                        let equals = arena.node(equals_site).ok_or_else(invalid)?;
                        let SourceAnchor::Range(equals_range) = equals.anchor else {
                            return Err(invalid());
                        };
                        if name.witness != SourceStatementWitnessId::new(ordinal)
                            || name.site.node() != name_site
                            || name.recovery != SourceStatementRecovery::Normal
                            || !mizar_lexer::is_identifier(&name.spelling)
                            || token.anchor != SourceAnchor::Range(name.source_range)
                            || token.kind.as_str()
                                != format!(
                                    "Token(SurfaceToken {{ kind: Identifier, text: {:?} }})",
                                    name.spelling
                                )
                            || equals.kind.as_str()
                                != r#"Token(SurfaceToken { kind: ReservedSymbol, text: "=" })"#
                            || token.recovery != NodeRecoveryState::Normal
                            || equals.recovery != NodeRecoveryState::Normal
                            || !token.children.is_empty()
                            || !equals.children.is_empty()
                            || name.source_range.source_id != source_id
                            || equals_range.source_id != source_id
                            || name.source_range.start != item_range.start
                            || name.source_range.end > equals_range.start
                            || equals_range.end > expression_range.start
                            || expression_range.end != item_range.end
                            || statements
                                .binding_env()
                                .bindings()
                                .iter()
                                .any(|(_, binding)| binding.spelling == name.spelling)
                            || primary_terms
                                .terms()
                                .iter()
                                .any(|(_, term)| term.spelling() == name.spelling)
                        {
                            return Err(invalid());
                        }
                        Some(SourceStatementWitnessNameId::new(0))
                    }
                    None => {
                        if expression_range != item_range {
                            return Err(invalid());
                        }
                        None
                    }
                };
                let owner = statements
                    .owners()
                    .get(SourceTheoremOwnerId::new(0))
                    .ok_or_else(invalid)?;
                let theorem = statements
                    .statements()
                    .get(SourceStatementId::new(0))
                    .ok_or_else(invalid)?;
                let conclusion = statements
                    .statements()
                    .get(SourceStatementId::new(1))
                    .ok_or_else(invalid)?;
                let source_ordinal = statements
                    .statements()
                    .iter()
                    .filter(|(_, statement)| statement.source_range().start < take_range.start)
                    .count();
                let spelling = match name {
                    Some(id) => format!(
                        "{} = {}",
                        self.names.get(id).ok_or_else(invalid)?.spelling,
                        term_spelling
                    ),
                    None => term_spelling.to_owned(),
                };
                let theorem_node = arena.node(theorem.site().node()).ok_or_else(invalid)?;
                let formula_expression = theorem_node
                    .children
                    .get(3)
                    .and_then(|id| arena.node(*id))
                    .ok_or_else(invalid)?;
                let [formula_site] = formula_expression.children.as_slice() else {
                    return Err(invalid());
                };
                let formula = arena.node(*formula_site).ok_or_else(invalid)?;
                let [left, _, right] = formula.children.as_slice() else {
                    return Err(invalid());
                };
                let mut operands = Vec::new();
                for expression in [left, right] {
                    let expression = arena.node(*expression).ok_or_else(invalid)?;
                    let [site] = expression.children.as_slice() else {
                        return Err(invalid());
                    };
                    let (_, operand) = primary_terms
                        .terms()
                        .iter()
                        .find(|(_, term)| term.site().node() == *site)
                        .ok_or_else(invalid)?;
                    if expression.kind.as_str() != "TermExpression"
                        || operand.kind() != SourcePrimaryTermKind::VariableReference
                        || operand.parent().is_some()
                    {
                        return Err(invalid());
                    }
                    operands.push(operand.spelling());
                }
                let equality = format!("{} = {}", operands[0], operands[1]);
                if set_term
                    .as_ref()
                    .is_some_and(|(_, term, _)| term.kind() == SourceSetTermKind::Choice)
                {
                    let conclusion_node =
                        arena.node(conclusion.site().node()).ok_or_else(invalid)?;
                    for (parent, kind, parts) in [
                        (
                            theorem_node,
                            "TheoremItem",
                            &[
                                ("ReservedWord", "theorem"),
                                ("Identifier", owner.spelling()),
                                ("ReservedSymbol", ":"),
                                ("", "FormulaExpression"),
                                ("", "ProofBlock"),
                                ("ReservedSymbol", ";"),
                            ][..],
                        ),
                        (
                            *proof,
                            "ProofBlock",
                            &[
                                ("ReservedWord", "proof"),
                                ("", "TakeStatement"),
                                ("", "ConclusionStatement"),
                                ("ReservedWord", "end"),
                            ][..],
                        ),
                        (
                            *take,
                            "TakeStatement",
                            &[
                                ("ReservedWord", "take"),
                                ("", "Witness"),
                                ("ReservedSymbol", ";"),
                            ][..],
                        ),
                        (
                            conclusion_node,
                            "ConclusionStatement",
                            &[
                                ("ReservedWord", "thus"),
                                ("", "Proposition"),
                                ("ReservedSymbol", ";"),
                            ][..],
                        ),
                    ] {
                        let SourceAnchor::Range(parent_range) = parent.anchor else {
                            return Err(invalid());
                        };
                        if parent.kind.as_str() != kind
                            || parent.recovery != NodeRecoveryState::Normal
                            || parent_range.source_id != source_id
                            || parent.children.len() != parts.len()
                        {
                            return Err(invalid());
                        }
                        let mut previous_end = parent_range.start;
                        for (index, (child, (token_kind, text))) in
                            parent.children.iter().zip(parts).enumerate()
                        {
                            let child = arena.node(*child).ok_or_else(invalid)?;
                            let SourceAnchor::Range(range) = child.anchor else {
                                return Err(invalid());
                            };
                            let expected = if token_kind.is_empty() {
                                (*text).to_owned()
                            } else {
                                format!(
                                    "Token(SurfaceToken {{ kind: {token_kind}, text: {text:?} }})"
                                )
                            };
                            if child.kind.as_str() != expected
                                || child.recovery != NodeRecoveryState::Normal
                                || range.source_id != source_id
                                || range.start < previous_end
                                || range.start >= range.end
                                || range.end > parent_range.end
                                || (index == 0 && range.start != parent_range.start)
                                || (index + 1 == parts.len() && range.end != parent_range.end)
                                || (!token_kind.is_empty()
                                    && (!child.children.is_empty()
                                        || range.end.checked_sub(range.start) != Some(text.len())))
                            {
                                return Err(invalid());
                            }
                            previous_end = range.end;
                        }
                    }
                }
                if expression.kind.as_str() != "TermExpression"
                    || expression_range.source_id != source_id
                    || term_range != expression_range
                    || context.lexical_scope.is_none()
                    || proof.children.len() != 4
                    || proof.children[1..3] != [*take_id, conclusion.site().node()]
                    || !is_descendant(arena, owner.site().node(), *proof_id)
                    || !is_descendant(arena, *proof_id, *term_site)
                    || take_range.source_id != source_id
                    || item_range.source_id != source_id
                    || take_range.start > item_range.start
                    || item_range.end > take_range.end
                    || take_range.end > conclusion.source_range().start
                    || theorem.source_ordinal() != 0
                    || source_ordinal != 1
                    || conclusion.source_ordinal() != 2
                    || formula_expression.kind.as_str() != "FormulaExpression"
                    || formula.kind.as_str() != "source.formula.atomic.equality"
                    || operands
                        != [
                            reference.map_or(operands[0], |(_, spelling)| spelling),
                            reference.map_or(operands[0], |(_, spelling)| spelling),
                        ]
                    || conclusion.spelling() != format!("thus {} ;", equality)
                    || theorem.spelling()
                        != format!(
                            "theorem {} : {} proof take {} ; thus {} ; end ;",
                            owner.spelling(),
                            equality,
                            self.witnesses
                                .rows
                                .iter()
                                .map(|row| row.spelling.as_str())
                                .collect::<Vec<_>>()
                                .join(" , "),
                            equality
                        )
                    || reference.is_some_and(|(reference_id, _)| {
                        statements
                            .input_facts()
                            .iter()
                            .any(|(_, fact)| fact.uses().contains(&reference_id))
                    })
                    || take.recovery != NodeRecoveryState::Normal
                    || item.recovery != NodeRecoveryState::Normal
                    || expression.recovery != NodeRecoveryState::Normal
                    || self.witnesses.rows[ordinal]
                        != (SourceStatementWitness {
                            owner: SourceTheoremOwnerId::new(0),
                            binding_context,
                            term: target,
                            take_site: TypedSiteRef::Node(*take_id),
                            take_range,
                            site: TypedSiteRef::Node(*item_id),
                            source_range: item_range,
                            source_ordinal,
                            ordinal,
                            spelling,
                            kind: if name.is_some() {
                                SourceStatementWitnessKind::Named
                            } else {
                                SourceStatementWitnessKind::Unnamed
                            },
                            recovery: SourceStatementRecovery::Normal,
                            name,
                        })
                {
                    return Err(invalid());
                }
            }
            return Ok(());
        }
        let profile = validate_witness_dependencies(
            self.source_id,
            &self.module_id,
            &self.statement_fingerprint,
            &self.primary_term_fingerprint,
            self.application_fingerprint.as_deref(),
            self.structure_fingerprint.as_deref(),
            self.set_term_fingerprint.as_deref(),
            statements,
            primary_terms,
            application,
            structure,
            set_terms,
            arena,
        )?;
        if self.source_id != source_id || &self.module_id != module_id {
            return Err(SourceStatementWitnessError::DependencyMismatch);
        }
        validate_witness_aggregate(profile, self.witnesses.len(), self.names.len())?;
        validate_witness_rows(
            profile,
            self.source_id,
            &self.witnesses,
            statements,
            primary_terms,
            application,
            structure,
            set_terms,
            arena,
        )?;
        #[cfg(test)]
        validate_witness_name_rows(profile, self.source_id, &self.names, &self.witnesses, arena)?;
        Ok(())
    }
}

impl SourceStatementReferenceHandoff {
    pub const fn source_id(&self) -> SourceId {
        self.source_id
    }

    pub const fn module_id(&self) -> &ModuleId {
        &self.module_id
    }

    pub fn statement_fingerprint(&self) -> &str {
        &self.statement_fingerprint
    }

    pub const fn resolver_ast(&self) -> &ResolvedAst {
        &self.resolver_ast
    }

    pub const fn label_projection(&self) -> &LabelProjection {
        match self.label_projection.as_slice() {
            [projection] => projection,
            [first, second] => match (first.source(), second.source()) {
                (
                    LabelProjectionSource::CurrentModule {
                        proof_scope: Some(_),
                        ..
                    },
                    LabelProjectionSource::CurrentModule {
                        proof_scope: None, ..
                    },
                ) => first,
                (
                    LabelProjectionSource::CurrentModule {
                        proof_scope: None, ..
                    },
                    LabelProjectionSource::CurrentModule {
                        proof_scope: Some(_),
                        ..
                    },
                ) => second,
                _ => panic!("validated native projection batch"),
            },
            _ => panic!("validated projection batch"),
        }
    }

    pub const fn reference_candidate(&self) -> &LabelReferenceCandidate {
        &self.reference_candidate
    }

    pub const fn label_resolution(&self) -> &LabelResolutionResult {
        &self.label_resolution
    }

    pub const fn labels(&self) -> &SourceStatementLabelTable {
        &self.labels
    }

    pub const fn citations(&self) -> &SourceStatementCitationTable {
        &self.citations
    }

    pub fn debug_text(&self) -> String {
        let imported = self.labels.is_empty();
        let label = (!imported).then(|| {
            self.labels
                .get(SourceStatementLabelId::new(0))
                .expect("validated source statement label")
        });
        let citation = self
            .citations
            .get(SourceStatementCitationId::new(0))
            .expect("validated source statement citation");
        let label_node = if imported {
            "absent".to_owned()
        } else {
            self.resolver_ast
                .nodes()
                .iter()
                .find(|(_, node)| {
                    label.is_some_and(|label| {
                        node.origin().anchor() == &SourceAnchor::Range(label.source_range)
                    })
                })
                .map(|(id, _)| id.index().to_string())
                .expect("validated label resolver node")
        };
        let reference_node = self.reference_candidate.site().node().index();
        let reference_state = resolved_node_by_index(self.resolver_ast.nodes(), reference_node)
            .map(|(_, node)| resolution_state_key(node.resolution()))
            .expect("validated reference resolver node");
        let reference_key = resolved_node_by_index(self.resolver_ast.nodes(), reference_node)
            .and_then(|(_, node)| node.reference_key())
            .map(node_reference_key)
            .expect("validated reference resolver key");
        let LabelReferenceScope::Unqualified {
            proof_scope: use_scope,
        } = self.reference_candidate.scope()
        else {
            unreachable!("validated unqualified proof-step reference")
        };
        let mut output = String::from("source-statement-reference-debug-v1\n");
        let _ = writeln!(
            output,
            "module: {}::{}",
            self.module_id.package().as_str(),
            self.module_id.path().as_str()
        );
        let _ = writeln!(
            output,
            "statement-fingerprint: {:?}",
            self.statement_fingerprint
        );
        let _ = writeln!(
            output,
            "resolver-ast root={} nodes={} name_refs={} label_refs={} imports={} exports={} label_node={} reference_node={} reference_state={} reference_key={}",
            self.resolver_ast.nodes().root().index(),
            self.resolver_ast.nodes().len(),
            self.resolver_ast.name_refs().len(),
            self.resolver_ast.label_refs().len(),
            self.resolver_ast.imports().imports().count(),
            self.resolver_ast.imports().exports().count(),
            label_node,
            reference_node,
            reference_state,
            reference_key,
        );
        for projection in &self.label_projection {
            match projection.source() {
                LabelProjectionSource::CurrentModule {
                    visible_after_ordinal,
                    proof_scope,
                } => {
                    let _ = writeln!(
                        output,
                        "resolver-projection origin={} namespace={} range={}..{} visible_after={} scope={} kind={} visibility={} export={} spelling={:?}",
                        projection.origin_path().as_str(),
                        projection.namespace().as_str(),
                        projection.declaration_range().start,
                        projection.declaration_range().end,
                        visible_after_ordinal,
                        proof_scope
                            .as_ref()
                            .map(label_scope_key)
                            .unwrap_or_else(|| "module".to_owned()),
                        label_kind_key(projection.kind()),
                        visibility_key(projection.visibility()),
                        export_status_key(projection.export_status()),
                        projection.primary_spelling(),
                    );
                }
                LabelProjectionSource::Imported => {
                    let _ = writeln!(
                        output,
                        "resolver-projection source=imported origin={} module={} namespace={} range={}..{} contribution={} path={} kind={} visibility={} export={} spelling={:?}",
                        projection.origin_path().as_str(),
                        projection.module().path().as_str(),
                        projection.namespace().as_str(),
                        projection.declaration_range().start,
                        projection.declaration_range().end,
                        projection.contribution().index(),
                        semantic_origin_path_key(projection.origin()),
                        label_kind_key(projection.kind()),
                        visibility_key(projection.visibility()),
                        export_status_key(projection.export_status()),
                        projection.primary_spelling(),
                    );
                }
                _ => unreachable!("validated source-statement label projection"),
            }
        }
        let _ = writeln!(
            output,
            "resolver-reference node={} range={}..{} source_ordinal={} scope={} expectation={} spelling={:?}",
            reference_node,
            self.reference_candidate.site().range().start,
            self.reference_candidate.site().range().end,
            self.reference_candidate.ordinal(),
            label_scope_key(use_scope.as_ref().expect("validated reference scope")),
            label_expectation_key(self.reference_candidate.expectation()),
            self.reference_candidate.site().spelling(),
        );
        let _ = write!(
            output,
            "resolver-result index={} references={} ids=",
            self.label_resolution.index().len(),
            self.label_resolution.table().len(),
        );
        write_dense_ids(&mut output, self.label_resolution.ids(), LabelRefId::index);
        let _ = writeln!(
            output,
            " diagnostics={}",
            self.label_resolution.diagnostics().len()
        );
        if let Some(label) = label {
            let _ = writeln!(
                output,
                "label#0 statement={} context={} candidate={} origin={} scope={} range={}..{} source_ordinal={} visible_after={} kind={} recovery={} spelling={:?}",
                label.statement.index(),
                label.context.index(),
                label.candidate.index(),
                label.origin_path.as_str(),
                label_scope_key(&label.proof_scope),
                label.source_range.start,
                label.source_range.end,
                label.source_ordinal,
                label.visible_after_ordinal,
                statement_label_kind_key(label.kind),
                statement_recovery_key(label.recovery),
                label.spelling,
            );
            let SourceStatementCitationTarget::Local(label_id) = citation.target else {
                unreachable!("validated local citation target")
            };
            let _ = writeln!(
                output,
                "citation#0 statement={} context={} label={} label_ref={} scope={} range={}..{} ordinal={} kind={} recovery={}",
                citation.statement.index(),
                citation.context.index(),
                label_id.index(),
                citation.label_ref.index(),
                label_scope_key(&citation.proof_scope),
                citation.source_range.start,
                citation.source_range.end,
                citation.ordinal,
                statement_citation_kind_key(citation.kind),
                statement_recovery_key(citation.recovery),
            );
        } else {
            let _ = writeln!(
                output,
                "citation#0 statement={} context={} target=imported label_ref={} scope={} range={}..{} ordinal={} kind={} recovery={}",
                citation.statement.index(),
                citation.context.index(),
                citation.label_ref.index(),
                label_scope_key(&citation.proof_scope),
                citation.source_range.start,
                citation.source_range.end,
                citation.ordinal,
                statement_citation_kind_key(citation.kind),
                statement_recovery_key(citation.recovery),
            );
        }
        output
    }

    pub(crate) fn validate_installation(
        &self,
        source_id: SourceId,
        module_id: &ModuleId,
        statements: &SourceStatementHandoff,
        arena: &TypedArena,
    ) -> Result<(), SourceStatementReferenceError> {
        if arena
            .iter()
            .any(|(_, node)| node.kind.as_str() == "TheoremItem")
        {
            let invalid = || SourceStatementReferenceError::DependencyMismatch;
            let descendant = arena.iter().any(|(_, node)| {
                node.kind.as_str() == "ConclusionStatement"
                    && node
                        .children
                        .get(2)
                        .and_then(|id| arena.node(*id))
                        .is_some_and(|node| node.kind.as_str() == "ProofBlock")
            });
            if self.source_id != source_id
                || &self.module_id != module_id
                || statements.source_id() != source_id
                || statements.module_id() != module_id
                || self.statement_fingerprint != statements.debug_text()
                || statements.statements.len() != if descendant { 5 } else { 4 }
                || statements.binding_env.contexts().len() != if descendant { 4 } else { 3 }
                || self.resolver_ast.source_id() != source_id
                || self.resolver_ast.module_id() != module_id
                || !self.resolver_ast.name_refs().is_empty()
                || self.resolver_ast.imports().imports().next().is_some()
                || self.resolver_ast.imports().exports().next().is_some()
                || self.resolver_ast.label_refs() != self.label_resolution.table()
                || self.label_projection.len() != 2
                || self.label_resolution.index().len() != 2
                || self.label_resolution.table().len() != 1
                || self.label_resolution.ids().len() != 1
                || !self.label_resolution.diagnostics().is_empty()
                || self.label_resolution.has_unresolved()
            {
                return Err(invalid());
            }
            let [label] = self.labels.rows.as_slice() else {
                return Err(SourceStatementReferenceError::InvalidAggregate);
            };
            let [citation] = self.citations.rows.as_slice() else {
                return Err(SourceStatementReferenceError::InvalidAggregate);
            };
            let mut proof = None;
            let mut owner_projection = None;
            for projection in &self.label_projection {
                let LabelProjectionSource::CurrentModule { proof_scope, .. } = projection.source()
                else {
                    return Err(invalid());
                };
                if projection.module() != module_id
                    || projection.origin().source_id() != source_id
                    || projection.namespace().as_str() != module_id.path().as_str()
                    || projection.origin().module_id() != module_id
                    || projection.origin().is_recovered()
                    || projection.origin().import_edge().is_some()
                    || projection.contribution() != statements.owner_contribution
                {
                    return Err(invalid());
                }
                if proof_scope.is_some() {
                    if proof.replace(projection).is_some()
                        || projection.kind() != LabelKind::ProofStep
                        || projection.visibility() != Visibility::Private
                        || projection.export_status() != ExportStatus::LocalOnly
                    {
                        return Err(invalid());
                    }
                } else if owner_projection.replace(projection).is_some()
                    || projection.kind() != LabelKind::Theorem
                    || projection.visibility() != Visibility::Public
                    || projection.export_status() != ExportStatus::Exported
                {
                    return Err(invalid());
                }
            }
            let projection = proof.ok_or_else(invalid)?;
            let owner_projection = owner_projection.ok_or_else(invalid)?;
            let LabelProjectionSource::CurrentModule {
                visible_after_ordinal,
                proof_scope: Some(proof_scope),
            } = projection.source()
            else {
                return Err(invalid());
            };
            let LabelReferenceScope::Unqualified {
                proof_scope: Some(use_scope),
            } = self.reference_candidate.scope()
            else {
                return Err(invalid());
            };
            let owner = statements.owners.rows.first().ok_or_else(invalid)?;
            if owner_projection.primary_spelling() != owner.spelling
                || owner_projection.origin().anchor() != &SourceAnchor::Range(owner.source_range)
                || projection.origin().anchor()
                    != &SourceAnchor::Range(projection.declaration_range())
            {
                return Err(invalid());
            }
            let namespace = mizar_resolve::env::NamespacePath::new(module_id.path().as_str());
            let replay = LabelResolver::new(&self.label_projection).resolve(
                module_id,
                &namespace,
                std::slice::from_ref(&self.reference_candidate),
            );
            if replay != self.label_resolution {
                return Err(invalid());
            }
            let citing_statement_id = TermFormulaChecker::validate_reserved_statement_snapshot(
                &self.resolver_ast,
                arena,
                statements,
                &self.label_projection,
                &self.reference_candidate,
            )?;
            let citing_statement = statements
                .statements
                .get(citing_statement_id)
                .ok_or_else(invalid)?;
            let reference_id = self.label_resolution.ids()[0];
            let entry = self
                .label_resolution
                .table()
                .get(reference_id)
                .ok_or_else(invalid)?;
            if entry.site() != self.reference_candidate.site()
                || entry.origin() != self.reference_candidate.origin()
                || entry.recovery() != RecoveryState::Normal
                || self.reference_candidate.expectation() != LabelExpectation::ProofOrTheorem
                || !matches!(entry.resolution(), LabelResolution::Resolved(target) if target.origin() == projection.origin_path() && target.kind() == LabelKind::ProofStep && target.range() == citation.source_range)
            {
                return Err(invalid());
            }
            for (id, node) in self.resolver_ast.nodes().iter() {
                let expected = if id == self.reference_candidate.site().node() {
                    (
                        NodeResolutionState::Resolved,
                        Some(NodeReferenceKey::Label(reference_id)),
                    )
                } else {
                    (NodeResolutionState::NotApplicable, None)
                };
                if (node.resolution(), node.reference_key()) != expected {
                    return Err(invalid());
                }
            }
            if label.statement != SourceStatementId::new(1)
                || label.context != SourceStatementContextId::new(1)
                || label.candidate != SourceStatementCandidateFactId::new(1)
                || label.origin_path != *projection.origin_path()
                || label.proof_scope != *proof_scope
                || label.source_range != projection.declaration_range()
                || label.source_ordinal != 0
                || label.visible_after_ordinal != *visible_after_ordinal
                || label.spelling != projection.primary_spelling()
                || label.kind != SourceStatementLabelKind::ProofStep
                || label.recovery != SourceStatementRecovery::Normal
                || label.source_range.start < statements.statements.rows[1].source_range.start
                || label.source_range.end > statements.statements.rows[1].source_range.end
            {
                return Err(SourceStatementReferenceError::InvalidLabel {
                    label: SourceStatementLabelId::new(0),
                });
            }
            if citation.statement != citing_statement_id
                || citation.context != citing_statement.context
                || citation.target
                    != SourceStatementCitationTarget::Local(SourceStatementLabelId::new(0))
                || citation.label_ref != reference_id
                || citation.proof_scope != *use_scope
                || citation.source_range != self.reference_candidate.site().range()
                || citation.ordinal != 0
                || citation.kind != SourceStatementCitationKind::SimpleLocal
                || citation.recovery != SourceStatementRecovery::Normal
                || self.reference_candidate.site().spelling() != label.spelling
                || *visible_after_ordinal >= self.reference_candidate.ordinal()
                || citation.source_range.start < citing_statement.source_range.start
                || citation.source_range.end > citing_statement.source_range.end
                || citation.source_range.start <= statements.statements.rows[1].source_range.end
            {
                return Err(SourceStatementReferenceError::InvalidCitation {
                    citation: SourceStatementCitationId::new(0),
                });
            }
            return Ok(());
        }
        if self.label_projection.len() != 1 {
            return Err(SourceStatementReferenceError::DependencyMismatch);
        }
        validate_reference_dependencies(
            self.source_id,
            &self.module_id,
            &self.statement_fingerprint,
            statements,
            &self.resolver_ast,
            self.label_projection(),
            &self.reference_candidate,
            &self.label_resolution,
            arena,
        )?;
        if self.source_id != source_id || &self.module_id != module_id {
            return Err(SourceStatementReferenceError::DependencyMismatch);
        }
        validate_reference_aggregate(statements, self.labels.len(), self.citations.len())?;
        validate_label_rows(
            self.source_id,
            &self.module_id,
            &self.labels,
            statements,
            self.label_projection(),
        )?;
        validate_citation_rows(
            self.source_id,
            &self.citations,
            &self.labels,
            statements,
            &self.reference_candidate,
            &self.label_resolution,
        )
    }
}

/// One validated theorem owner.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceTheoremOwner {
    symbol: SymbolId,
    contribution: SourceContributionId,
    site: TypedSiteRef,
    source_range: SourceRange,
    spelling: String,
    role: SourceTheoremRole,
    status: SourceTheoremStatus,
    recovery: SourceStatementRecovery,
}

impl SourceTheoremOwner {
    pub const fn symbol(&self) -> &SymbolId {
        &self.symbol
    }
    pub const fn contribution(&self) -> SourceContributionId {
        self.contribution
    }
    pub const fn site(&self) -> &TypedSiteRef {
        &self.site
    }
    pub const fn source_range(&self) -> SourceRange {
        self.source_range
    }
    pub fn spelling(&self) -> &str {
        &self.spelling
    }
    pub const fn role(&self) -> SourceTheoremRole {
        self.role
    }
    pub const fn status(&self) -> SourceTheoremStatus {
        self.status
    }
    pub const fn recovery(&self) -> SourceStatementRecovery {
        self.recovery
    }
}

/// One validated source theorem statement.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceStatement {
    owner: SourceTheoremOwnerId,
    context: SourceStatementContextId,
    formula: SourceStatementFormulaTarget,
    site: TypedSiteRef,
    source_range: SourceRange,
    source_ordinal: usize,
    spelling: String,
    kind: SourceStatementKind,
    recovery: SourceStatementRecovery,
}

impl SourceStatement {
    pub const fn owner(&self) -> SourceTheoremOwnerId {
        self.owner
    }
    pub const fn context(&self) -> SourceStatementContextId {
        self.context
    }
    pub const fn formula(&self) -> SourceStatementFormulaTarget {
        self.formula
    }
    pub const fn site(&self) -> &TypedSiteRef {
        &self.site
    }
    pub const fn source_range(&self) -> SourceRange {
        self.source_range
    }
    pub const fn source_ordinal(&self) -> usize {
        self.source_ordinal
    }
    pub fn spelling(&self) -> &str {
        &self.spelling
    }
    pub const fn kind(&self) -> SourceStatementKind {
        self.kind
    }
    pub const fn recovery(&self) -> SourceStatementRecovery {
        self.recovery
    }
}

/// One validated statement visibility context.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceStatementContext {
    statement: SourceStatementId,
    binding_context: BindingContextId,
    source_range: SourceRange,
    visible_bindings: Vec<BindingId>,
}

impl SourceStatementContext {
    pub const fn statement(&self) -> SourceStatementId {
        self.statement
    }
    pub const fn binding_context(&self) -> BindingContextId {
        self.binding_context
    }
    pub const fn source_range(&self) -> SourceRange {
        self.source_range
    }
    pub fn visible_bindings(&self) -> &[BindingId] {
        &self.visible_bindings
    }
}

/// One validated statement input fact.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceStatementInputFact {
    statement: SourceStatementId,
    context: SourceStatementContextId,
    ordinal: usize,
    kind: SourceStatementInputFactKind,
    binding: BindingId,
    uses: Vec<SourcePrimaryTermReferenceId>,
}

impl SourceStatementInputFact {
    pub const fn statement(&self) -> SourceStatementId {
        self.statement
    }
    pub const fn context(&self) -> SourceStatementContextId {
        self.context
    }
    pub const fn ordinal(&self) -> usize {
        self.ordinal
    }
    pub const fn kind(&self) -> SourceStatementInputFactKind {
        self.kind
    }
    pub const fn binding(&self) -> BindingId {
        self.binding
    }
    pub fn uses(&self) -> &[SourcePrimaryTermReferenceId] {
        &self.uses
    }
}

/// One validated unverified statement candidate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceStatementCandidateFact {
    statement: SourceStatementId,
    context: SourceStatementContextId,
    ordinal: usize,
    kind: SourceStatementCandidateFactKind,
    formula: SourceStatementFormulaTarget,
}

impl SourceStatementCandidateFact {
    pub const fn statement(&self) -> SourceStatementId {
        self.statement
    }
    pub const fn context(&self) -> SourceStatementContextId {
        self.context
    }
    pub const fn ordinal(&self) -> usize {
        self.ordinal
    }
    pub const fn kind(&self) -> SourceStatementCandidateFactKind {
        self.kind
    }
    pub const fn formula(&self) -> SourceStatementFormulaTarget {
        self.formula
    }
}

/// One validated source-statement witness.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceStatementWitness {
    owner: SourceTheoremOwnerId,
    binding_context: BindingContextId,
    term: SourceStatementWitnessTermTarget,
    take_site: TypedSiteRef,
    take_range: SourceRange,
    site: TypedSiteRef,
    source_range: SourceRange,
    source_ordinal: usize,
    ordinal: usize,
    spelling: String,
    kind: SourceStatementWitnessKind,
    recovery: SourceStatementRecovery,
    name: Option<SourceStatementWitnessNameId>,
}

impl SourceStatementWitness {
    pub const fn owner(&self) -> SourceTheoremOwnerId {
        self.owner
    }
    pub const fn binding_context(&self) -> BindingContextId {
        self.binding_context
    }
    pub const fn term(&self) -> SourceStatementWitnessTermTarget {
        self.term
    }
    pub const fn take_site(&self) -> &TypedSiteRef {
        &self.take_site
    }
    pub const fn take_range(&self) -> SourceRange {
        self.take_range
    }
    pub const fn site(&self) -> &TypedSiteRef {
        &self.site
    }
    pub const fn source_range(&self) -> SourceRange {
        self.source_range
    }
    pub const fn source_ordinal(&self) -> usize {
        self.source_ordinal
    }
    pub const fn ordinal(&self) -> usize {
        self.ordinal
    }
    pub fn spelling(&self) -> &str {
        &self.spelling
    }
    pub const fn kind(&self) -> SourceStatementWitnessKind {
        self.kind
    }
    pub const fn recovery(&self) -> SourceStatementRecovery {
        self.recovery
    }
    pub const fn name(&self) -> Option<SourceStatementWitnessNameId> {
        self.name
    }
}

/// One validated source name attached to a witness occurrence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceStatementWitnessName {
    witness: SourceStatementWitnessId,
    site: TypedSiteRef,
    source_range: SourceRange,
    spelling: String,
    recovery: SourceStatementRecovery,
}

impl SourceStatementWitnessName {
    pub const fn witness(&self) -> SourceStatementWitnessId {
        self.witness
    }
    pub const fn site(&self) -> &TypedSiteRef {
        &self.site
    }
    pub const fn source_range(&self) -> SourceRange {
        self.source_range
    }
    pub fn spelling(&self) -> &str {
        &self.spelling
    }
    pub const fn recovery(&self) -> SourceStatementRecovery {
        self.recovery
    }
}

/// One validated local proof-step label.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceStatementLabel {
    statement: SourceStatementId,
    context: SourceStatementContextId,
    candidate: SourceStatementCandidateFactId,
    origin_path: LabelOriginPath,
    proof_scope: LabelScopePath,
    source_range: SourceRange,
    source_ordinal: usize,
    visible_after_ordinal: usize,
    spelling: String,
    kind: SourceStatementLabelKind,
    recovery: SourceStatementRecovery,
}

impl SourceStatementLabel {
    pub const fn statement(&self) -> SourceStatementId {
        self.statement
    }
    pub const fn context(&self) -> SourceStatementContextId {
        self.context
    }
    pub const fn candidate(&self) -> SourceStatementCandidateFactId {
        self.candidate
    }
    pub const fn origin_path(&self) -> &LabelOriginPath {
        &self.origin_path
    }
    pub const fn proof_scope(&self) -> &LabelScopePath {
        &self.proof_scope
    }
    pub const fn source_range(&self) -> SourceRange {
        self.source_range
    }
    pub const fn source_ordinal(&self) -> usize {
        self.source_ordinal
    }
    pub const fn visible_after_ordinal(&self) -> usize {
        self.visible_after_ordinal
    }
    pub fn spelling(&self) -> &str {
        &self.spelling
    }
    pub const fn kind(&self) -> SourceStatementLabelKind {
        self.kind
    }
    pub const fn recovery(&self) -> SourceStatementRecovery {
        self.recovery
    }
}

/// One validated local or imported proof/theorem citation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceStatementCitation {
    statement: SourceStatementId,
    context: SourceStatementContextId,
    target: SourceStatementCitationTarget,
    label_ref: LabelRefId,
    proof_scope: LabelScopePath,
    source_range: SourceRange,
    ordinal: usize,
    kind: SourceStatementCitationKind,
    recovery: SourceStatementRecovery,
}

impl SourceStatementCitation {
    pub const fn statement(&self) -> SourceStatementId {
        self.statement
    }
    pub const fn context(&self) -> SourceStatementContextId {
        self.context
    }
    pub const fn target(&self) -> SourceStatementCitationTarget {
        self.target
    }
    pub const fn label_ref(&self) -> LabelRefId {
        self.label_ref
    }
    pub const fn proof_scope(&self) -> &LabelScopePath {
        &self.proof_scope
    }
    pub const fn source_range(&self) -> SourceRange {
        self.source_range
    }
    pub const fn ordinal(&self) -> usize {
        self.ordinal
    }
    pub const fn kind(&self) -> SourceStatementCitationKind {
        self.kind
    }
    pub const fn recovery(&self) -> SourceStatementRecovery {
        self.recovery
    }
}

macro_rules! dense_table {
    ($table:ident, $row:ident, $id:ident) => {
        #[derive(Debug, Clone, PartialEq, Eq)]
        pub struct $table {
            rows: Vec<$row>,
        }

        impl $table {
            pub fn get(&self, id: $id) -> Option<&$row> {
                self.rows.get(id.index())
            }

            pub fn iter(&self) -> impl Iterator<Item = ($id, &$row)> {
                self.rows
                    .iter()
                    .enumerate()
                    .map(|(index, row)| ($id::new(index), row))
            }

            pub const fn len(&self) -> usize {
                self.rows.len()
            }

            pub const fn is_empty(&self) -> bool {
                self.rows.is_empty()
            }
        }
    };
}

dense_table!(
    SourceTheoremOwnerTable,
    SourceTheoremOwner,
    SourceTheoremOwnerId
);
dense_table!(SourceStatementTable, SourceStatement, SourceStatementId);
dense_table!(
    SourceStatementContextTable,
    SourceStatementContext,
    SourceStatementContextId
);
dense_table!(
    SourceStatementInputFactTable,
    SourceStatementInputFact,
    SourceStatementInputFactId
);
dense_table!(
    SourceStatementCandidateFactTable,
    SourceStatementCandidateFact,
    SourceStatementCandidateFactId
);
dense_table!(
    SourceStatementLabelTable,
    SourceStatementLabel,
    SourceStatementLabelId
);
dense_table!(
    SourceStatementCitationTable,
    SourceStatementCitation,
    SourceStatementCitationId
);
dense_table!(
    SourceStatementWitnessTable,
    SourceStatementWitness,
    SourceStatementWitnessId
);
dense_table!(
    SourceStatementWitnessNameTable,
    SourceStatementWitnessName,
    SourceStatementWitnessNameId
);

/// Atomically validates and constructs source statement handoffs.
#[derive(Debug, Clone, Copy, Default)]
pub struct SourceStatementProducer;

impl SourceStatementProducer {
    pub fn build(
        input: SourceStatementHandoffInput,
        symbols: &SymbolEnv,
        bindings: &BindingEnv,
        primary_terms: &SourcePrimaryTermHandoff,
        atomic_formulas: &SourceAtomicFormulaHandoff,
        arena: &TypedArena,
    ) -> Result<SourceStatementHandoff, SourceStatementError> {
        let profile = validate_dependencies(
            input.source_id,
            &input.module_id,
            bindings,
            primary_terms,
            atomic_formulas,
            arena,
        )?;
        validate_aggregate_lengths(
            profile,
            input.owners.len(),
            input.statements.len(),
            input.contexts.len(),
            input.input_facts.len(),
            input.candidate_facts.len(),
        )?;
        let checked_owner = validate_resolver_owner(&input, symbols)?;
        let owner_contribution = input.owners[0].contribution;
        let owners = SourceTheoremOwnerTable {
            rows: input
                .owners
                .into_iter()
                .map(|row| SourceTheoremOwner {
                    symbol: row.symbol,
                    contribution: row.contribution,
                    site: row.site,
                    source_range: row.source_range,
                    spelling: row.spelling,
                    role: row.role,
                    status: row.status,
                    recovery: row.recovery,
                })
                .collect(),
        };
        validate_owner_rows(
            profile,
            input.source_id,
            &input.module_id,
            &owners,
            &checked_owner,
            owner_contribution,
            arena,
        )?;
        let statements = SourceStatementTable {
            rows: input
                .statements
                .into_iter()
                .map(|row| SourceStatement {
                    owner: row.owner,
                    context: row.context,
                    formula: row.formula,
                    site: row.site,
                    source_range: row.source_range,
                    source_ordinal: row.source_ordinal,
                    spelling: row.spelling,
                    kind: row.kind,
                    recovery: row.recovery,
                })
                .collect(),
        };
        let contexts = SourceStatementContextTable {
            rows: input
                .contexts
                .into_iter()
                .map(|row| SourceStatementContext {
                    statement: row.statement,
                    binding_context: row.binding_context,
                    source_range: row.source_range,
                    visible_bindings: row.visible_bindings,
                })
                .collect(),
        };
        validate_statement_rows(
            profile,
            input.source_id,
            &statements,
            &owners,
            &contexts,
            atomic_formulas,
            arena,
        )?;
        validate_context_rows(profile, input.source_id, &contexts, &statements, bindings)?;
        let input_facts = SourceStatementInputFactTable {
            rows: input
                .input_facts
                .into_iter()
                .map(|row| SourceStatementInputFact {
                    statement: row.statement,
                    context: row.context,
                    ordinal: row.ordinal,
                    kind: row.kind,
                    binding: row.binding,
                    uses: row.uses,
                })
                .collect(),
        };
        validate_input_fact_rows(
            profile,
            &input_facts,
            &statements,
            &contexts,
            bindings,
            primary_terms,
        )?;
        let candidate_facts = SourceStatementCandidateFactTable {
            rows: input
                .candidate_facts
                .into_iter()
                .map(|row| SourceStatementCandidateFact {
                    statement: row.statement,
                    context: row.context,
                    ordinal: row.ordinal,
                    kind: row.kind,
                    formula: row.formula,
                })
                .collect(),
        };
        validate_candidate_fact_rows(
            profile,
            &candidate_facts,
            &statements,
            &contexts,
            atomic_formulas,
        )?;
        Ok(SourceStatementHandoff {
            source_id: input.source_id,
            module_id: input.module_id,
            binding_env: bindings.clone(),
            binding_fingerprint: bindings.debug_text(),
            primary_term_fingerprint: primary_terms.debug_text(),
            atomic_formula_fingerprint: atomic_formulas.debug_text(),
            composite_formula_fingerprint: None,
            formula_composition_fingerprint: None,
            checked_owner,
            owner_contribution,
            owners,
            statements,
            contexts,
            input_facts,
            candidate_facts,
        })
    }

    #[allow(clippy::too_many_arguments)] // Rationale: the composite-root transaction must reauthenticate every frozen lower-family dependency explicitly.
    pub fn build_with_formula_composition(
        input: SourceStatementHandoffInput,
        symbols: &SymbolEnv,
        bindings: &BindingEnv,
        primary_terms: &SourcePrimaryTermHandoff,
        atomic_formulas: &SourceAtomicFormulaHandoff,
        composite_formulas: &SourceCompositeFormulaHandoff,
        formula_composition: &SourceFormulaCompositionHandoff,
        arena: &TypedArena,
    ) -> Result<SourceStatementHandoff, SourceStatementError> {
        let profile =
            task_258b4_input_profile(&input).ok_or(SourceStatementError::DependencyMismatch)?;
        validate_task_258b4_dependencies(
            profile,
            input.source_id,
            &input.module_id,
            bindings,
            primary_terms,
            atomic_formulas,
            composite_formulas,
            formula_composition,
            arena,
        )?;
        if (
            input.owners.len(),
            input.statements.len(),
            input.contexts.len(),
            input.input_facts.len(),
            input.candidate_facts.len(),
        ) != (1, 1, 1, 0, 1)
        {
            return Err(SourceStatementError::InvalidAggregate);
        }
        let checked_owner = validate_resolver_owner(&input, symbols)?;
        let owner_contribution = input.owners[0].contribution;
        let owners = SourceTheoremOwnerTable {
            rows: input
                .owners
                .into_iter()
                .map(|row| SourceTheoremOwner {
                    symbol: row.symbol,
                    contribution: row.contribution,
                    site: row.site,
                    source_range: row.source_range,
                    spelling: row.spelling,
                    role: row.role,
                    status: row.status,
                    recovery: row.recovery,
                })
                .collect(),
        };
        let statements = SourceStatementTable {
            rows: input
                .statements
                .into_iter()
                .map(|row| SourceStatement {
                    owner: row.owner,
                    context: row.context,
                    formula: row.formula,
                    site: row.site,
                    source_range: row.source_range,
                    source_ordinal: row.source_ordinal,
                    spelling: row.spelling,
                    kind: row.kind,
                    recovery: row.recovery,
                })
                .collect(),
        };
        let contexts = SourceStatementContextTable {
            rows: input
                .contexts
                .into_iter()
                .map(|row| SourceStatementContext {
                    statement: row.statement,
                    binding_context: row.binding_context,
                    source_range: row.source_range,
                    visible_bindings: row.visible_bindings,
                })
                .collect(),
        };
        let input_facts = SourceStatementInputFactTable {
            rows: input
                .input_facts
                .into_iter()
                .map(|row| SourceStatementInputFact {
                    statement: row.statement,
                    context: row.context,
                    ordinal: row.ordinal,
                    kind: row.kind,
                    binding: row.binding,
                    uses: row.uses,
                })
                .collect(),
        };
        let candidate_facts = SourceStatementCandidateFactTable {
            rows: input
                .candidate_facts
                .into_iter()
                .map(|row| SourceStatementCandidateFact {
                    statement: row.statement,
                    context: row.context,
                    ordinal: row.ordinal,
                    kind: row.kind,
                    formula: row.formula,
                })
                .collect(),
        };
        validate_task_258b4_rows(
            profile,
            input.source_id,
            &input.module_id,
            &owners,
            &checked_owner,
            owner_contribution,
            &statements,
            &contexts,
            &input_facts,
            &candidate_facts,
            composite_formulas,
            arena,
        )?;
        Ok(SourceStatementHandoff {
            source_id: input.source_id,
            module_id: input.module_id,
            binding_env: bindings.clone(),
            binding_fingerprint: bindings.debug_text(),
            primary_term_fingerprint: primary_terms.debug_text(),
            atomic_formula_fingerprint: atomic_formulas.debug_text(),
            composite_formula_fingerprint: Some(composite_formulas.debug_text()),
            formula_composition_fingerprint: Some(formula_composition.debug_text()),
            checked_owner,
            owner_contribution,
            owners,
            statements,
            contexts,
            input_facts,
            candidate_facts,
        })
    }
}

/// Atomically validates and constructs source-statement witness handoffs.
#[derive(Debug, Clone, Copy, Default)]
pub struct SourceStatementWitnessProducer;

impl SourceStatementWitnessProducer {
    pub fn build(
        input: SourceStatementWitnessHandoffInput,
        statements: &SourceStatementHandoff,
        primary_terms: &SourcePrimaryTermHandoff,
        arena: &TypedArena,
    ) -> Result<SourceStatementWitnessHandoff, SourceStatementWitnessError> {
        Self::build_inner(input, statements, primary_terms, None, None, None, arena)
    }

    pub fn build_with_application(
        input: SourceStatementWitnessHandoffInput,
        statements: &SourceStatementHandoff,
        primary_terms: &SourcePrimaryTermHandoff,
        application: &SourceFunctorApplicationHandoff,
        arena: &TypedArena,
    ) -> Result<SourceStatementWitnessHandoff, SourceStatementWitnessError> {
        Self::build_inner(
            input,
            statements,
            primary_terms,
            Some(application),
            None,
            None,
            arena,
        )
    }

    pub fn build_with_structure(
        input: SourceStatementWitnessHandoffInput,
        statements: &SourceStatementHandoff,
        primary_terms: &SourcePrimaryTermHandoff,
        structure: &SourceStructureHandoff,
        arena: &TypedArena,
    ) -> Result<SourceStatementWitnessHandoff, SourceStatementWitnessError> {
        Self::build_inner(
            input,
            statements,
            primary_terms,
            None,
            Some(structure),
            None,
            arena,
        )
    }

    pub fn build_with_set_term(
        input: SourceStatementWitnessHandoffInput,
        statements: &SourceStatementHandoff,
        primary_terms: &SourcePrimaryTermHandoff,
        set_terms: &SourceSetTermHandoff,
        arena: &TypedArena,
    ) -> Result<SourceStatementWitnessHandoff, SourceStatementWitnessError> {
        Self::build_inner(
            input,
            statements,
            primary_terms,
            None,
            None,
            Some(set_terms),
            arena,
        )
    }

    #[allow(clippy::too_many_arguments)] // Rationale: construct the complete mutually exclusive lower-family tuple.
    fn build_inner(
        input: SourceStatementWitnessHandoffInput,
        statements: &SourceStatementHandoff,
        primary_terms: &SourcePrimaryTermHandoff,
        application: Option<&SourceFunctorApplicationHandoff>,
        structure: Option<&SourceStructureHandoff>,
        set_terms: Option<&SourceSetTermHandoff>,
        arena: &TypedArena,
    ) -> Result<SourceStatementWitnessHandoff, SourceStatementWitnessError> {
        if arena
            .iter()
            .any(|(_, node)| node.kind.as_str() == "TheoremItem")
        {
            return Err(SourceStatementWitnessError::DependencyMismatch);
        }
        let statement_fingerprint = statements.debug_text();
        let primary_term_fingerprint = primary_terms.debug_text();
        let application_fingerprint = application.map(SourceFunctorApplicationHandoff::debug_text);
        let structure_fingerprint = structure.map(SourceStructureHandoff::debug_text);
        let set_term_fingerprint = set_terms.map(SourceSetTermHandoff::debug_text);
        let profile = validate_witness_dependencies(
            input.source_id,
            &input.module_id,
            &statement_fingerprint,
            &primary_term_fingerprint,
            application_fingerprint.as_deref(),
            structure_fingerprint.as_deref(),
            set_term_fingerprint.as_deref(),
            statements,
            primary_terms,
            application,
            structure,
            set_terms,
            arena,
        )?;
        validate_witness_aggregate(profile, input.witnesses.len(), input.names.len())?;
        let witnesses = SourceStatementWitnessTable {
            rows: input
                .witnesses
                .into_iter()
                .map(|row| SourceStatementWitness {
                    owner: row.owner,
                    binding_context: row.binding_context,
                    term: row.term,
                    take_site: row.take_site,
                    take_range: row.take_range,
                    site: row.site,
                    source_range: row.source_range,
                    source_ordinal: row.source_ordinal,
                    ordinal: row.ordinal,
                    spelling: row.spelling,
                    kind: row.kind,
                    recovery: row.recovery,
                    name: row.name,
                })
                .collect(),
        };
        validate_witness_rows(
            profile,
            input.source_id,
            &witnesses,
            statements,
            primary_terms,
            application,
            structure,
            set_terms,
            arena,
        )?;
        let names = SourceStatementWitnessNameTable {
            rows: input
                .names
                .into_iter()
                .map(|row| SourceStatementWitnessName {
                    witness: row.witness,
                    site: row.site,
                    source_range: row.source_range,
                    spelling: row.spelling,
                    recovery: row.recovery,
                })
                .collect(),
        };
        #[cfg(test)]
        validate_witness_name_rows(profile, input.source_id, &names, &witnesses, arena)?;
        Ok(SourceStatementWitnessHandoff {
            source_id: input.source_id,
            module_id: input.module_id,
            statement_fingerprint,
            primary_term_fingerprint,
            application_fingerprint,
            structure_fingerprint,
            set_term_fingerprint,
            witnesses,
            names,
        })
    }
}

/// Atomically validates and constructs source-statement reference handoffs.
#[derive(Debug, Clone, Copy, Default)]
pub struct SourceStatementReferenceProducer;

impl SourceStatementReferenceProducer {
    pub fn build(
        input: SourceStatementReferenceHandoffInput,
        statements: &SourceStatementHandoff,
        resolver_ast: &ResolvedAst,
        projection: &LabelProjection,
        reference: &LabelReferenceCandidate,
        resolution: &LabelResolutionResult,
        arena: &TypedArena,
    ) -> Result<SourceStatementReferenceHandoff, SourceStatementReferenceError> {
        let statement_fingerprint = statements.debug_text();
        validate_reference_dependencies(
            input.source_id,
            &input.module_id,
            &statement_fingerprint,
            statements,
            resolver_ast,
            projection,
            reference,
            resolution,
            arena,
        )?;
        validate_reference_aggregate(statements, input.labels.len(), input.citations.len())?;
        let labels = SourceStatementLabelTable {
            rows: input
                .labels
                .into_iter()
                .map(|row| SourceStatementLabel {
                    statement: row.statement,
                    context: row.context,
                    candidate: row.candidate,
                    origin_path: row.origin_path,
                    proof_scope: row.proof_scope,
                    source_range: row.source_range,
                    source_ordinal: row.source_ordinal,
                    visible_after_ordinal: row.visible_after_ordinal,
                    spelling: row.spelling,
                    kind: row.kind,
                    recovery: row.recovery,
                })
                .collect(),
        };
        validate_label_rows(
            input.source_id,
            &input.module_id,
            &labels,
            statements,
            projection,
        )?;
        let citations = SourceStatementCitationTable {
            rows: input
                .citations
                .into_iter()
                .map(|row| SourceStatementCitation {
                    statement: row.statement,
                    context: row.context,
                    target: row.target,
                    label_ref: row.label_ref,
                    proof_scope: row.proof_scope,
                    source_range: row.source_range,
                    ordinal: row.ordinal,
                    kind: row.kind,
                    recovery: row.recovery,
                })
                .collect(),
        };
        validate_citation_rows(
            input.source_id,
            &citations,
            &labels,
            statements,
            reference,
            resolution,
        )?;
        Ok(SourceStatementReferenceHandoff {
            source_id: input.source_id,
            module_id: input.module_id,
            statement_fingerprint,
            resolver_ast: resolver_ast.clone(),
            label_projection: vec![projection.clone()],
            reference_candidate: reference.clone(),
            label_resolution: resolution.clone(),
            labels,
            citations,
        })
    }
}

/// Atomic Task-258A/258B1 base source-statement failure.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum SourceStatementError {
    DependencyMismatch,
    InvalidOwner {
        owner: SourceTheoremOwnerId,
    },
    InvalidStatement {
        statement: SourceStatementId,
    },
    InvalidContext {
        context: SourceStatementContextId,
    },
    InvalidInputFact {
        fact: SourceStatementInputFactId,
    },
    InvalidCandidateFact {
        fact: SourceStatementCandidateFactId,
    },
    InvalidAggregate,
}

impl fmt::Display for SourceStatementError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DependencyMismatch => formatter.write_str("source statement dependency mismatch"),
            Self::InvalidOwner { owner } => {
                write!(
                    formatter,
                    "source theorem owner {} is invalid",
                    owner.index()
                )
            }
            Self::InvalidStatement { statement } => {
                write!(
                    formatter,
                    "source statement {} is invalid",
                    statement.index()
                )
            }
            Self::InvalidContext { context } => {
                write!(
                    formatter,
                    "source statement context {} is invalid",
                    context.index()
                )
            }
            Self::InvalidInputFact { fact } => {
                write!(
                    formatter,
                    "source statement input fact {} is invalid",
                    fact.index()
                )
            }
            Self::InvalidCandidateFact { fact } => {
                write!(
                    formatter,
                    "source statement candidate fact {} is invalid",
                    fact.index()
                )
            }
            Self::InvalidAggregate => formatter.write_str("source statement aggregate is invalid"),
        }
    }
}

impl Error for SourceStatementError {}

/// Atomic Task-258B3 source-statement witness failure.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum SourceStatementWitnessError {
    DependencyMismatch,
    InvalidWitness { witness: SourceStatementWitnessId },
    InvalidName { name: SourceStatementWitnessNameId },
    InvalidAggregate,
}

impl fmt::Display for SourceStatementWitnessError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DependencyMismatch => {
                formatter.write_str("source statement witness dependency mismatch")
            }
            Self::InvalidWitness { witness } => {
                write!(
                    formatter,
                    "source statement witness {} is invalid",
                    witness.index()
                )
            }
            Self::InvalidName { name } => {
                write!(
                    formatter,
                    "source statement witness name {} is invalid",
                    name.index()
                )
            }
            Self::InvalidAggregate => {
                formatter.write_str("source statement witness aggregate is invalid")
            }
        }
    }
}

impl Error for SourceStatementWitnessError {}

/// Atomic Task-258B1 source-statement reference failure.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum SourceStatementReferenceError {
    DependencyMismatch,
    InvalidLabel { label: SourceStatementLabelId },
    InvalidCitation { citation: SourceStatementCitationId },
    InvalidAggregate,
}

impl fmt::Display for SourceStatementReferenceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DependencyMismatch => {
                formatter.write_str("source statement reference dependency mismatch")
            }
            Self::InvalidLabel { label } => {
                write!(
                    formatter,
                    "source statement label {} is invalid",
                    label.index()
                )
            }
            Self::InvalidCitation { citation } => write!(
                formatter,
                "source statement citation {} is invalid",
                citation.index()
            ),
            Self::InvalidAggregate => {
                formatter.write_str("source statement reference aggregate is invalid")
            }
        }
    }
}

impl Error for SourceStatementReferenceError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StatementProfile {
    #[cfg(test)]
    Task258A,
    #[cfg(test)]
    Task258B1,
    #[cfg(test)]
    Task258B5A,
    Task258B5B,
    #[cfg(test)]
    Task258B2,
    #[cfg(test)]
    Task258B3,
    #[cfg(test)]
    Task258B3N,
    #[cfg(test)]
    Task258B3M1,
    #[cfg(test)]
    Task258B3M2A,
    #[cfg(test)]
    Task258B3M2B1,
    #[cfg(test)]
    Task258B3M2B2A,
    #[cfg(test)]
    Task258B3M2B2B1A,
    #[cfg(test)]
    Task258B3M2B2B1B1,
    Task258B3M2B2B2A,
    Task258B3M2B2B2B,
    #[cfg(test)]
    Task258B3M2B2B2C,
    #[cfg(test)]
    Task258B3M2B2B3A,
    #[cfg(test)]
    Task258B3M2B2B3B,
    #[cfg(test)]
    Task258B3M2B2B3C,
    #[cfg(test)]
    Task258B3M2B2B3D,
    #[cfg(test)]
    Task258B3M2B2B3E,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StatementReferenceProfile {
    #[cfg(test)]
    Task258B1,
    #[cfg(test)]
    Task258B5A,
    Task258B5B,
}

fn statement_reference_profile(
    statements: &SourceStatementHandoff,
) -> Option<StatementReferenceProfile> {
    #[cfg(test)]
    if statements.is_task_258b1_profile() {
        return Some(StatementReferenceProfile::Task258B1);
    }
    #[cfg(test)]
    if statements.is_task_258b5a_profile() {
        return Some(StatementReferenceProfile::Task258B5A);
    }
    if statements.is_task_258b5b_profile() {
        Some(StatementReferenceProfile::Task258B5B)
    } else {
        None
    }
}

fn validate_dependencies(
    source_id: SourceId,
    module_id: &ModuleId,
    bindings: &BindingEnv,
    primary_terms: &SourcePrimaryTermHandoff,
    atomic_formulas: &SourceAtomicFormulaHandoff,
    arena: &TypedArena,
) -> Result<StatementProfile, SourceStatementError> {
    if bindings.source_id() != source_id
        || bindings.module_id() != module_id
        || primary_terms.source_id() != source_id
        || primary_terms.module_id() != module_id
        || atomic_formulas.source_id() != source_id
        || atomic_formulas.module_id() != module_id
        || atomic_formulas.primary_term_fingerprint() != primary_terms.debug_text()
        || atomic_formulas.application_fingerprint().is_some()
        || atomic_formulas.structure_fingerprint().is_some()
        || atomic_formulas.set_term_fingerprint().is_some()
    {
        return Err(SourceStatementError::DependencyMismatch);
    }
    primary_terms
        .validate_installation(source_id, module_id, arena)
        .map_err(|_| SourceStatementError::DependencyMismatch)?;
    atomic_formulas
        .validate_installation(source_id, module_id, primary_terms, None, None, None, arena)
        .map_err(|_| SourceStatementError::DependencyMismatch)?;
    #[cfg(test)]
    if exact_binding_profile(StatementProfile::Task258A, source_id, bindings)
        && exact_primary_profile(StatementProfile::Task258A, primary_terms)
        && exact_atomic_profile(
            StatementProfile::Task258A,
            atomic_formulas,
            primary_terms,
            arena,
        )
    {
        return Ok(StatementProfile::Task258A);
    }
    #[cfg(test)]
    {
        let b1_binding = exact_binding_profile(StatementProfile::Task258B1, source_id, bindings);
        let b1_primary = exact_primary_profile(StatementProfile::Task258B1, primary_terms);
        let b1_atomic = exact_atomic_profile(
            StatementProfile::Task258B1,
            atomic_formulas,
            primary_terms,
            arena,
        );
        if b1_binding && b1_primary && b1_atomic {
            return Ok(StatementProfile::Task258B1);
        }
    }
    #[cfg(test)]
    {
        let b5a_binding = exact_binding_profile(StatementProfile::Task258B5A, source_id, bindings);
        let b5a_primary = exact_primary_profile(StatementProfile::Task258B5A, primary_terms);
        let b5a_atomic = exact_atomic_profile(
            StatementProfile::Task258B5A,
            atomic_formulas,
            primary_terms,
            arena,
        );
        if b5a_binding
            && b5a_primary
            && b5a_atomic
            && exact_task258b5a_shared_arena(source_id, arena)
        {
            return Ok(StatementProfile::Task258B5A);
        }
    }
    let b5b_binding = exact_binding_profile(StatementProfile::Task258B5B, source_id, bindings);
    let b5b_primary = exact_primary_profile(StatementProfile::Task258B5B, primary_terms);
    let b5b_atomic = exact_atomic_profile(
        StatementProfile::Task258B5B,
        atomic_formulas,
        primary_terms,
        arena,
    );
    if b5b_binding && b5b_primary && b5b_atomic && exact_task258b5b_shared_arena(source_id, arena) {
        return Ok(StatementProfile::Task258B5B);
    }
    #[cfg(test)]
    {
        let b2_binding = exact_binding_profile(StatementProfile::Task258B2, source_id, bindings);
        let b2_primary = exact_primary_profile(StatementProfile::Task258B2, primary_terms);
        let b2_atomic = exact_atomic_profile(
            StatementProfile::Task258B2,
            atomic_formulas,
            primary_terms,
            arena,
        );
        if b2_binding && b2_primary && b2_atomic {
            return Ok(StatementProfile::Task258B2);
        }
    }
    #[cfg(test)]
    {
        let b3_binding = exact_binding_profile(StatementProfile::Task258B3, source_id, bindings);
        let b3_primary = exact_primary_profile(StatementProfile::Task258B3, primary_terms);
        let b3_atomic = exact_atomic_profile(
            StatementProfile::Task258B3,
            atomic_formulas,
            primary_terms,
            arena,
        );
        if b3_binding && b3_primary && b3_atomic {
            return Ok(StatementProfile::Task258B3);
        }
    }
    #[cfg(test)]
    {
        let b3n_binding = exact_binding_profile(StatementProfile::Task258B3N, source_id, bindings);
        let b3n_primary = exact_primary_profile(StatementProfile::Task258B3N, primary_terms);
        let b3n_atomic = exact_atomic_profile(
            StatementProfile::Task258B3N,
            atomic_formulas,
            primary_terms,
            arena,
        );
        if b3n_binding && b3n_primary && b3n_atomic {
            return Ok(StatementProfile::Task258B3N);
        }
    }
    #[cfg(test)]
    {
        let b3m1_binding =
            exact_binding_profile(StatementProfile::Task258B3M1, source_id, bindings);
        let b3m1_primary = exact_primary_profile(StatementProfile::Task258B3M1, primary_terms);
        let b3m1_atomic = exact_atomic_profile(
            StatementProfile::Task258B3M1,
            atomic_formulas,
            primary_terms,
            arena,
        );
        if b3m1_binding && b3m1_primary && b3m1_atomic {
            return Ok(StatementProfile::Task258B3M1);
        }
    }
    #[cfg(test)]
    {
        let b3m2a_binding =
            exact_binding_profile(StatementProfile::Task258B3M2A, source_id, bindings);
        let b3m2a_primary = exact_primary_profile(StatementProfile::Task258B3M2A, primary_terms);
        let b3m2a_atomic = exact_atomic_profile(
            StatementProfile::Task258B3M2A,
            atomic_formulas,
            primary_terms,
            arena,
        );
        if b3m2a_binding && b3m2a_primary && b3m2a_atomic {
            return Ok(StatementProfile::Task258B3M2A);
        }
    }
    #[cfg(test)]
    {
        let b3m2b1_binding =
            exact_binding_profile(StatementProfile::Task258B3M2B1, source_id, bindings);
        let b3m2b1_primary = exact_primary_profile(StatementProfile::Task258B3M2B1, primary_terms);
        let b3m2b1_atomic = exact_atomic_profile(
            StatementProfile::Task258B3M2B1,
            atomic_formulas,
            primary_terms,
            arena,
        );
        if b3m2b1_binding && b3m2b1_primary && b3m2b1_atomic {
            return Ok(StatementProfile::Task258B3M2B1);
        }
    }
    #[cfg(test)]
    {
        let b3m2b2a_binding =
            exact_binding_profile(StatementProfile::Task258B3M2B2A, source_id, bindings);
        let b3m2b2a_primary =
            exact_primary_profile(StatementProfile::Task258B3M2B2A, primary_terms);
        let b3m2b2a_atomic = exact_atomic_profile(
            StatementProfile::Task258B3M2B2A,
            atomic_formulas,
            primary_terms,
            arena,
        );
        if b3m2b2a_binding && b3m2b2a_primary && b3m2b2a_atomic {
            return Ok(StatementProfile::Task258B3M2B2A);
        }
    }
    #[cfg(test)]
    {
        let b3m2b2b1a_binding =
            exact_binding_profile(StatementProfile::Task258B3M2B2B1A, source_id, bindings);
        let b3m2b2b1a_primary =
            exact_primary_profile(StatementProfile::Task258B3M2B2B1A, primary_terms);
        let b3m2b2b1a_atomic = exact_atomic_profile(
            StatementProfile::Task258B3M2B2B1A,
            atomic_formulas,
            primary_terms,
            arena,
        );
        if b3m2b2b1a_binding && b3m2b2b1a_primary && b3m2b2b1a_atomic {
            return Ok(StatementProfile::Task258B3M2B2B1A);
        }
    }
    #[cfg(test)]
    {
        let b3m2b2b1b1_binding =
            exact_binding_profile(StatementProfile::Task258B3M2B2B1B1, source_id, bindings);
        let b3m2b2b1b1_primary =
            exact_primary_profile(StatementProfile::Task258B3M2B2B1B1, primary_terms);
        let b3m2b2b1b1_atomic = exact_atomic_profile(
            StatementProfile::Task258B3M2B2B1B1,
            atomic_formulas,
            primary_terms,
            arena,
        );
        if b3m2b2b1b1_binding && b3m2b2b1b1_primary && b3m2b2b1b1_atomic {
            return Ok(StatementProfile::Task258B3M2B2B1B1);
        }
    }
    let b3m2b2b2a_binding =
        exact_binding_profile(StatementProfile::Task258B3M2B2B2A, source_id, bindings);
    let b3m2b2b2a_primary =
        exact_primary_profile(StatementProfile::Task258B3M2B2B2A, primary_terms);
    let b3m2b2b2a_atomic = exact_atomic_profile(
        StatementProfile::Task258B3M2B2B2A,
        atomic_formulas,
        primary_terms,
        arena,
    );
    if b3m2b2b2a_binding && b3m2b2b2a_primary && b3m2b2b2a_atomic {
        return Ok(StatementProfile::Task258B3M2B2B2A);
    }
    let b3m2b2b2b_binding =
        exact_binding_profile(StatementProfile::Task258B3M2B2B2B, source_id, bindings);
    let b3m2b2b2b_primary =
        exact_primary_profile(StatementProfile::Task258B3M2B2B2B, primary_terms);
    let b3m2b2b2b_atomic = exact_atomic_profile(
        StatementProfile::Task258B3M2B2B2B,
        atomic_formulas,
        primary_terms,
        arena,
    );
    if b3m2b2b2b_binding && b3m2b2b2b_primary && b3m2b2b2b_atomic {
        return Ok(StatementProfile::Task258B3M2B2B2B);
    }
    #[cfg(test)]
    {
        let b3m2b2b2c_binding =
            exact_binding_profile(StatementProfile::Task258B3M2B2B2C, source_id, bindings);
        let b3m2b2b2c_primary =
            exact_primary_profile(StatementProfile::Task258B3M2B2B2C, primary_terms);
        let b3m2b2b2c_atomic = exact_atomic_profile(
            StatementProfile::Task258B3M2B2B2C,
            atomic_formulas,
            primary_terms,
            arena,
        );
        if b3m2b2b2c_binding && b3m2b2b2c_primary && b3m2b2b2c_atomic {
            return Ok(StatementProfile::Task258B3M2B2B2C);
        }
    }
    #[cfg(test)]
    {
        let b3m2b2b3a_binding =
            exact_binding_profile(StatementProfile::Task258B3M2B2B3A, source_id, bindings);
        let b3m2b2b3a_primary =
            exact_primary_profile(StatementProfile::Task258B3M2B2B3A, primary_terms);
        let b3m2b2b3a_atomic = exact_atomic_profile(
            StatementProfile::Task258B3M2B2B3A,
            atomic_formulas,
            primary_terms,
            arena,
        );
        if b3m2b2b3a_binding && b3m2b2b3a_primary && b3m2b2b3a_atomic {
            return Ok(StatementProfile::Task258B3M2B2B3A);
        }
    }
    #[cfg(test)]
    {
        let b3m2b2b3b_binding =
            exact_binding_profile(StatementProfile::Task258B3M2B2B3B, source_id, bindings);
        let b3m2b2b3b_primary =
            exact_primary_profile(StatementProfile::Task258B3M2B2B3B, primary_terms);
        let b3m2b2b3b_atomic = exact_atomic_profile(
            StatementProfile::Task258B3M2B2B3B,
            atomic_formulas,
            primary_terms,
            arena,
        );
        if b3m2b2b3b_binding && b3m2b2b3b_primary && b3m2b2b3b_atomic {
            return Ok(StatementProfile::Task258B3M2B2B3B);
        }
    }
    #[cfg(test)]
    {
        let b3m2b2b3c_binding =
            exact_binding_profile(StatementProfile::Task258B3M2B2B3C, source_id, bindings);
        let b3m2b2b3c_primary =
            exact_primary_profile(StatementProfile::Task258B3M2B2B3C, primary_terms);
        let b3m2b2b3c_atomic = exact_atomic_profile(
            StatementProfile::Task258B3M2B2B3C,
            atomic_formulas,
            primary_terms,
            arena,
        );
        if b3m2b2b3c_binding && b3m2b2b3c_primary && b3m2b2b3c_atomic {
            return Ok(StatementProfile::Task258B3M2B2B3C);
        }
    }
    #[cfg(test)]
    {
        let b3m2b2b3d_binding =
            exact_binding_profile(StatementProfile::Task258B3M2B2B3D, source_id, bindings);
        let b3m2b2b3d_primary =
            exact_primary_profile(StatementProfile::Task258B3M2B2B3D, primary_terms);
        let b3m2b2b3d_atomic = exact_atomic_profile(
            StatementProfile::Task258B3M2B2B3D,
            atomic_formulas,
            primary_terms,
            arena,
        );
        if b3m2b2b3d_binding && b3m2b2b3d_primary && b3m2b2b3d_atomic {
            return Ok(StatementProfile::Task258B3M2B2B3D);
        }
    }
    #[cfg(test)]
    {
        let b3m2b2b3e_binding =
            exact_binding_profile(StatementProfile::Task258B3M2B2B3E, source_id, bindings);
        let b3m2b2b3e_primary =
            exact_primary_profile(StatementProfile::Task258B3M2B2B3E, primary_terms);
        let b3m2b2b3e_atomic = exact_atomic_profile(
            StatementProfile::Task258B3M2B2B3E,
            atomic_formulas,
            primary_terms,
            arena,
        );
        if b3m2b2b3e_binding && b3m2b2b3e_primary && b3m2b2b3e_atomic {
            return Ok(StatementProfile::Task258B3M2B2B3E);
        }
    }
    Err(SourceStatementError::DependencyMismatch)
}

fn exact_binding_profile(
    profile: StatementProfile,
    source_id: SourceId,
    bindings: &BindingEnv,
) -> bool {
    let expected_contexts = match profile {
        #[cfg(test)]
        StatementProfile::Task258A => 1,
        #[cfg(test)]
        StatementProfile::Task258B1 => 3,
        #[cfg(test)]
        StatementProfile::Task258B5A => 4,
        StatementProfile::Task258B5B => 2,
        #[cfg(test)]
        StatementProfile::Task258B2 => 2,
        #[cfg(test)]
        StatementProfile::Task258B3 => 2,
        #[cfg(test)]
        StatementProfile::Task258B3N => 2,
        #[cfg(test)]
        StatementProfile::Task258B3M1 => 2,
        #[cfg(test)]
        StatementProfile::Task258B3M2A => 2,
        #[cfg(test)]
        StatementProfile::Task258B3M2B1 => 2,
        #[cfg(test)]
        StatementProfile::Task258B3M2B2A => 2,
        #[cfg(test)]
        StatementProfile::Task258B3M2B2B1A => 2,
        #[cfg(test)]
        StatementProfile::Task258B3M2B2B1B1 => 2,
        StatementProfile::Task258B3M2B2B2A => 2,
        StatementProfile::Task258B3M2B2B2B => 2,
        #[cfg(test)]
        StatementProfile::Task258B3M2B2B2C => 2,
        #[cfg(test)]
        StatementProfile::Task258B3M2B2B3A => 2,
        #[cfg(test)]
        StatementProfile::Task258B3M2B2B3B => 2,
        #[cfg(test)]
        StatementProfile::Task258B3M2B2B3C => 2,
        #[cfg(test)]
        StatementProfile::Task258B3M2B2B3D => 2,
        #[cfg(test)]
        StatementProfile::Task258B3M2B2B3E => 2,
    };
    if bindings.contexts().len() != expected_contexts
        || bindings.bindings().len() != 1
        || !bindings.diagnostics().is_empty()
    {
        return false;
    }
    let Some(context) = bindings.contexts().get(BindingContextId::new(0)) else {
        return false;
    };
    if context.id != BindingContextId::new(0)
        || context.owner != BindingContextOwner::Module
        || context.parent.is_some()
        || context.layer != BindingContextLayer::Module
        || context.lexical_scope.is_some()
        || context.bindings != [BindingId::new(0)]
        || context.visible_bindings != [BindingId::new(0)]
        || context.recovery != BindingContextRecovery::Normal
    {
        return false;
    }
    match profile {
        #[cfg(test)]
        StatementProfile::Task258A => {}
        #[cfg(test)]
        StatementProfile::Task258B1 => {
            let Some(outer) = bindings.contexts().get(BindingContextId::new(1)) else {
                return false;
            };
            let Some(nested) = bindings.contexts().get(BindingContextId::new(2)) else {
                return false;
            };
            if outer.id != BindingContextId::new(1)
                || outer.owner
                    != (BindingContextOwner::SourceStatement {
                        source_range: range(source_id, 69, 137),
                    })
                || outer.parent != Some(BindingContextId::new(0))
                || outer.layer != BindingContextLayer::Proof
                || outer
                    .lexical_scope
                    .as_ref()
                    .is_none_or(|scope| scope.path() != [0])
                || !outer.bindings.is_empty()
                || outer.visible_bindings != [BindingId::new(0)]
                || outer.recovery != BindingContextRecovery::Normal
                || nested.id != BindingContextId::new(2)
                || nested.owner
                    != (BindingContextOwner::SourceStatement {
                        source_range: range(source_id, 86, 113),
                    })
                || nested.parent != Some(BindingContextId::new(1))
                || nested.layer != BindingContextLayer::Proof
                || nested
                    .lexical_scope
                    .as_ref()
                    .is_none_or(|scope| scope.path() != [0, 0])
                || !nested.bindings.is_empty()
                || nested.visible_bindings != [BindingId::new(0)]
                || nested.recovery != BindingContextRecovery::Normal
            {
                return false;
            }
        }
        #[cfg(test)]
        StatementProfile::Task258B5A => {
            let Some(outer) = bindings.contexts().get(BindingContextId::new(1)) else {
                return false;
            };
            let Some(first_descendant) = bindings.contexts().get(BindingContextId::new(2)) else {
                return false;
            };
            let Some(second_descendant) = bindings.contexts().get(BindingContextId::new(3)) else {
                return false;
            };
            if outer.id != BindingContextId::new(1)
                || outer.owner
                    != (BindingContextOwner::SourceStatement {
                        source_range: range(source_id, 87, 183),
                    })
                || outer.parent != Some(BindingContextId::new(0))
                || outer.layer != BindingContextLayer::Proof
                || outer
                    .lexical_scope
                    .as_ref()
                    .is_none_or(|scope| scope.path() != [0])
                || !outer.bindings.is_empty()
                || outer.visible_bindings != [BindingId::new(0)]
                || outer.recovery != BindingContextRecovery::Normal
                || first_descendant.id != BindingContextId::new(2)
                || first_descendant.owner
                    != (BindingContextOwner::SourceStatement {
                        source_range: range(source_id, 104, 131),
                    })
                || first_descendant.parent != Some(BindingContextId::new(1))
                || first_descendant.layer != BindingContextLayer::Proof
                || first_descendant
                    .lexical_scope
                    .as_ref()
                    .is_none_or(|scope| scope.path() != [0, 0])
                || !first_descendant.bindings.is_empty()
                || first_descendant.visible_bindings != [BindingId::new(0)]
                || first_descendant.recovery != BindingContextRecovery::Normal
                || second_descendant.id != BindingContextId::new(3)
                || second_descendant.owner
                    != (BindingContextOwner::SourceStatement {
                        source_range: range(source_id, 146, 178),
                    })
                || second_descendant.parent != Some(BindingContextId::new(1))
                || second_descendant.layer != BindingContextLayer::Proof
                || second_descendant
                    .lexical_scope
                    .as_ref()
                    .is_none_or(|scope| scope.path() != [0, 1])
                || !second_descendant.bindings.is_empty()
                || second_descendant.visible_bindings != [BindingId::new(0)]
                || second_descendant.recovery != BindingContextRecovery::Normal
            {
                return false;
            }
        }
        StatementProfile::Task258B5B => {
            let Some(proof) = bindings.contexts().get(BindingContextId::new(1)) else {
                return false;
            };
            if proof.id != BindingContextId::new(1)
                || proof.owner
                    != (BindingContextOwner::SourceStatement {
                        source_range: range(source_id, 114, 144),
                    })
                || proof.parent != Some(BindingContextId::new(0))
                || proof.layer != BindingContextLayer::Proof
                || proof
                    .lexical_scope
                    .as_ref()
                    .is_none_or(|scope| scope.path() != [0])
                || !proof.bindings.is_empty()
                || proof.visible_bindings != [BindingId::new(0)]
                || proof.recovery != BindingContextRecovery::Normal
            {
                return false;
            }
        }
        #[cfg(test)]
        StatementProfile::Task258B2 => {
            let Some(proof) = bindings.contexts().get(BindingContextId::new(1)) else {
                return false;
            };
            if proof.id != BindingContextId::new(1)
                || proof.owner
                    != (BindingContextOwner::SourceStatement {
                        source_range: range(source_id, 72, 111),
                    })
                || proof.parent != Some(BindingContextId::new(0))
                || proof.layer != BindingContextLayer::Proof
                || proof
                    .lexical_scope
                    .as_ref()
                    .is_none_or(|scope| scope.path() != [0])
                || !proof.bindings.is_empty()
                || proof.visible_bindings != [BindingId::new(0)]
                || proof.recovery != BindingContextRecovery::Normal
            {
                return false;
            }
        }
        #[cfg(test)]
        StatementProfile::Task258B3 => {
            let Some(proof) = bindings.contexts().get(BindingContextId::new(1)) else {
                return false;
            };
            if proof.id != BindingContextId::new(1)
                || proof.owner
                    != (BindingContextOwner::SourceStatement {
                        source_range: range(source_id, 69, 102),
                    })
                || proof.parent != Some(BindingContextId::new(0))
                || proof.layer != BindingContextLayer::Proof
                || proof
                    .lexical_scope
                    .as_ref()
                    .is_none_or(|scope| scope.path() != [0])
                || !proof.bindings.is_empty()
                || proof.visible_bindings != [BindingId::new(0)]
                || proof.recovery != BindingContextRecovery::Normal
            {
                return false;
            }
        }
        #[cfg(test)]
        StatementProfile::Task258B3N => {
            let Some(proof) = bindings.contexts().get(BindingContextId::new(1)) else {
                return false;
            };
            if proof.id != BindingContextId::new(1)
                || proof.owner
                    != (BindingContextOwner::SourceStatement {
                        source_range: range(source_id, 68, 105),
                    })
                || proof.parent != Some(BindingContextId::new(0))
                || proof.layer != BindingContextLayer::Proof
                || proof
                    .lexical_scope
                    .as_ref()
                    .is_none_or(|scope| scope.path() != [0])
                || !proof.bindings.is_empty()
                || proof.visible_bindings != [BindingId::new(0)]
                || proof.recovery != BindingContextRecovery::Normal
            {
                return false;
            }
        }
        #[cfg(test)]
        StatementProfile::Task258B3M1 => {
            let Some(proof) = bindings.contexts().get(BindingContextId::new(1)) else {
                return false;
            };
            if proof.id != BindingContextId::new(1)
                || proof.owner
                    != (BindingContextOwner::SourceStatement {
                        source_range: range(source_id, 71, 111),
                    })
                || proof.parent != Some(BindingContextId::new(0))
                || proof.layer != BindingContextLayer::Proof
                || proof
                    .lexical_scope
                    .as_ref()
                    .is_none_or(|scope| scope.path() != [0])
                || !proof.bindings.is_empty()
                || proof.visible_bindings != [BindingId::new(0)]
                || proof.recovery != BindingContextRecovery::Normal
            {
                return false;
            }
        }
        #[cfg(test)]
        StatementProfile::Task258B3M2A => {
            let Some(proof) = bindings.contexts().get(BindingContextId::new(1)) else {
                return false;
            };
            if proof.id != BindingContextId::new(1)
                || proof.owner
                    != (BindingContextOwner::SourceStatement {
                        source_range: range(source_id, 70, 105),
                    })
                || proof.parent != Some(BindingContextId::new(0))
                || proof.layer != BindingContextLayer::Proof
                || proof
                    .lexical_scope
                    .as_ref()
                    .is_none_or(|scope| scope.path() != [0])
                || !proof.bindings.is_empty()
                || proof.visible_bindings != [BindingId::new(0)]
                || proof.recovery != BindingContextRecovery::Normal
            {
                return false;
            }
        }
        #[cfg(test)]
        StatementProfile::Task258B3M2B1 => {
            let Some(proof) = bindings.contexts().get(BindingContextId::new(1)) else {
                return false;
            };
            if proof.id != BindingContextId::new(1)
                || proof.owner
                    != (BindingContextOwner::SourceStatement {
                        source_range: range(source_id, 76, 111),
                    })
                || proof.parent != Some(BindingContextId::new(0))
                || proof.layer != BindingContextLayer::Proof
                || proof
                    .lexical_scope
                    .as_ref()
                    .is_none_or(|scope| scope.path() != [0])
                || !proof.bindings.is_empty()
                || proof.visible_bindings != [BindingId::new(0)]
                || proof.recovery != BindingContextRecovery::Normal
            {
                return false;
            }
        }
        #[cfg(test)]
        StatementProfile::Task258B3M2B2A => {
            let Some(proof) = bindings.contexts().get(BindingContextId::new(1)) else {
                return false;
            };
            if proof.id != BindingContextId::new(1)
                || proof.owner
                    != (BindingContextOwner::SourceStatement {
                        source_range: range(source_id, 82, 119),
                    })
                || proof.parent != Some(BindingContextId::new(0))
                || proof.layer != BindingContextLayer::Proof
                || proof
                    .lexical_scope
                    .as_ref()
                    .is_none_or(|scope| scope.path() != [0])
                || !proof.bindings.is_empty()
                || proof.visible_bindings != [BindingId::new(0)]
                || proof.recovery != BindingContextRecovery::Normal
            {
                return false;
            }
        }
        #[cfg(test)]
        StatementProfile::Task258B3M2B2B1A => {
            let Some(proof) = bindings.contexts().get(BindingContextId::new(1)) else {
                return false;
            };
            if proof.id != BindingContextId::new(1)
                || proof.owner
                    != (BindingContextOwner::SourceStatement {
                        source_range: range(source_id, 103, 141),
                    })
                || proof.parent != Some(BindingContextId::new(0))
                || proof.layer != BindingContextLayer::Proof
                || proof
                    .lexical_scope
                    .as_ref()
                    .is_none_or(|scope| scope.path() != [0])
                || !proof.bindings.is_empty()
                || proof.visible_bindings != [BindingId::new(0)]
                || proof.recovery != BindingContextRecovery::Normal
            {
                return false;
            }
        }
        #[cfg(test)]
        StatementProfile::Task258B3M2B2B1B1 => {
            let Some(proof) = bindings.contexts().get(BindingContextId::new(1)) else {
                return false;
            };
            if proof.id != BindingContextId::new(1)
                || proof.owner
                    != (BindingContextOwner::SourceStatement {
                        source_range: range(source_id, 116, 156),
                    })
                || proof.parent != Some(BindingContextId::new(0))
                || proof.layer != BindingContextLayer::Proof
                || proof
                    .lexical_scope
                    .as_ref()
                    .is_none_or(|scope| scope.path() != [0])
                || !proof.bindings.is_empty()
                || proof.visible_bindings != [BindingId::new(0)]
                || proof.recovery != BindingContextRecovery::Normal
            {
                return false;
            }
        }
        StatementProfile::Task258B3M2B2B2A => {
            let Some(proof) = bindings.contexts().get(BindingContextId::new(1)) else {
                return false;
            };
            if proof.id != BindingContextId::new(1)
                || proof.owner
                    != (BindingContextOwner::SourceStatement {
                        source_range: range(source_id, 112, 170),
                    })
                || proof.parent != Some(BindingContextId::new(0))
                || proof.layer != BindingContextLayer::Proof
                || proof
                    .lexical_scope
                    .as_ref()
                    .is_none_or(|scope| scope.path() != [0])
                || !proof.bindings.is_empty()
                || proof.visible_bindings != [BindingId::new(0)]
                || proof.recovery != BindingContextRecovery::Normal
            {
                return false;
            }
        }
        StatementProfile::Task258B3M2B2B2B => {
            let Some(proof) = bindings.contexts().get(BindingContextId::new(1)) else {
                return false;
            };
            if proof.id != BindingContextId::new(1)
                || proof.owner
                    != (BindingContextOwner::SourceStatement {
                        source_range: range(source_id, 109, 169),
                    })
                || proof.parent != Some(BindingContextId::new(0))
                || proof.layer != BindingContextLayer::Proof
                || proof
                    .lexical_scope
                    .as_ref()
                    .is_none_or(|scope| scope.path() != [0])
                || !proof.bindings.is_empty()
                || proof.visible_bindings != [BindingId::new(0)]
                || proof.recovery != BindingContextRecovery::Normal
            {
                return false;
            }
        }
        #[cfg(test)]
        StatementProfile::Task258B3M2B2B2C => {
            let Some(proof) = bindings.contexts().get(BindingContextId::new(1)) else {
                return false;
            };
            if proof.id != BindingContextId::new(1)
                || proof.owner
                    != (BindingContextOwner::SourceStatement {
                        source_range: range(source_id, 107, 179),
                    })
                || proof.parent != Some(BindingContextId::new(0))
                || proof.layer != BindingContextLayer::Proof
                || proof
                    .lexical_scope
                    .as_ref()
                    .is_none_or(|scope| scope.path() != [0])
                || !proof.bindings.is_empty()
                || proof.visible_bindings != [BindingId::new(0)]
                || proof.recovery != BindingContextRecovery::Normal
            {
                return false;
            }
        }
        #[cfg(test)]
        StatementProfile::Task258B3M2B2B3A => {
            let Some(proof) = bindings.contexts().get(BindingContextId::new(1)) else {
                return false;
            };
            if proof.id != BindingContextId::new(1)
                || proof.owner
                    != (BindingContextOwner::SourceStatement {
                        source_range: range(source_id, 77, 115),
                    })
                || proof.parent != Some(BindingContextId::new(0))
                || proof.layer != BindingContextLayer::Proof
                || proof
                    .lexical_scope
                    .as_ref()
                    .is_none_or(|scope| scope.path() != [0])
                || !proof.bindings.is_empty()
                || proof.visible_bindings != [BindingId::new(0)]
                || proof.recovery != BindingContextRecovery::Normal
            {
                return false;
            }
        }
        #[cfg(test)]
        StatementProfile::Task258B3M2B2B3B => {
            let Some(proof) = bindings.contexts().get(BindingContextId::new(1)) else {
                return false;
            };
            if proof.id != BindingContextId::new(1)
                || proof.owner
                    != (BindingContextOwner::SourceStatement {
                        source_range: range(source_id, 82, 116),
                    })
                || proof.parent != Some(BindingContextId::new(0))
                || proof.layer != BindingContextLayer::Proof
                || proof
                    .lexical_scope
                    .as_ref()
                    .is_none_or(|scope| scope.path() != [0])
                || !proof.bindings.is_empty()
                || proof.visible_bindings != [BindingId::new(0)]
                || proof.recovery != BindingContextRecovery::Normal
            {
                return false;
            }
        }
        #[cfg(test)]
        StatementProfile::Task258B3M2B2B3C => {
            let Some(proof) = bindings.contexts().get(BindingContextId::new(1)) else {
                return false;
            };
            if proof.id != BindingContextId::new(1)
                || proof.owner
                    != (BindingContextOwner::SourceStatement {
                        source_range: range(source_id, 69, 108),
                    })
                || proof.parent != Some(BindingContextId::new(0))
                || proof.layer != BindingContextLayer::Proof
                || proof
                    .lexical_scope
                    .as_ref()
                    .is_none_or(|scope| scope.path() != [0])
                || !proof.bindings.is_empty()
                || proof.visible_bindings != [BindingId::new(0)]
                || proof.recovery != BindingContextRecovery::Normal
            {
                return false;
            }
        }
        #[cfg(test)]
        StatementProfile::Task258B3M2B2B3D => {
            let Some(proof) = bindings.contexts().get(BindingContextId::new(1)) else {
                return false;
            };
            if proof.id != BindingContextId::new(1)
                || proof.owner
                    != (BindingContextOwner::SourceStatement {
                        source_range: range(source_id, 66, 107),
                    })
                || proof.parent != Some(BindingContextId::new(0))
                || proof.layer != BindingContextLayer::Proof
                || proof
                    .lexical_scope
                    .as_ref()
                    .is_none_or(|scope| scope.path() != [0])
                || !proof.bindings.is_empty()
                || proof.visible_bindings != [BindingId::new(0)]
                || proof.recovery != BindingContextRecovery::Normal
            {
                return false;
            }
        }
        #[cfg(test)]
        StatementProfile::Task258B3M2B2B3E => {
            let Some(proof) = bindings.contexts().get(BindingContextId::new(1)) else {
                return false;
            };
            if proof.id != BindingContextId::new(1)
                || proof.owner
                    != (BindingContextOwner::SourceStatement {
                        source_range: range(source_id, 76, 137),
                    })
                || proof.parent != Some(BindingContextId::new(0))
                || proof.layer != BindingContextLayer::Proof
                || proof
                    .lexical_scope
                    .as_ref()
                    .is_none_or(|scope| scope.path() != [0])
                || !proof.bindings.is_empty()
                || proof.visible_bindings != [BindingId::new(0)]
                || proof.recovery != BindingContextRecovery::Normal
            {
                return false;
            }
        }
    }
    let (declaration_range, type_range) = match profile {
        #[cfg(test)]
        StatementProfile::Task258B3M2B2B1B1 => (range(source_id, 37, 38), range(source_id, 43, 46)),
        #[cfg(test)]
        StatementProfile::Task258B3M2B2B1A => (range(source_id, 37, 38), range(source_id, 43, 46)),
        StatementProfile::Task258B3M2B2B2A
        | StatementProfile::Task258B3M2B2B2B
        | StatementProfile::Task258B5B => (range(source_id, 37, 38), range(source_id, 43, 46)),
        #[cfg(test)]
        StatementProfile::Task258B3M2B2B2C => (range(source_id, 37, 38), range(source_id, 43, 46)),
        #[cfg(test)]
        _ => (range(source_id, 8, 9), range(source_id, 14, 17)),
    };
    let Some(binding) = bindings.bindings().get(BindingId::new(0)) else {
        return false;
    };
    binding.id == BindingId::new(0)
        && binding.spelling == "x"
        && binding.kind == BindingKind::ReservedVariable
        && binding.identity
            == BinderIdentity::ReservedVariable {
                spelling: "x".to_owned(),
                declaration_range,
            }
        && binding.owner_context == BindingContextId::new(0)
        && binding.declaration_range == declaration_range
        && binding.visible_after_ordinal == 0
        && binding.type_site == BindingTypeSite::Source(type_range)
        && binding.status == BindingStatus::Reserved
        && binding.captured.identities().is_empty()
        && binding.diagnostics.is_empty()
        && binding.recovery == BindingRecoveryState::Normal
}

fn exact_primary_profile(
    profile: StatementProfile,
    primary_terms: &SourcePrimaryTermHandoff,
) -> bool {
    match profile {
        #[cfg(test)]
        StatementProfile::Task258B3M2A => {
            return exact_task258b3m2a_primary_profile(primary_terms);
        }
        #[cfg(test)]
        StatementProfile::Task258B3M2B1 => {
            return exact_task258b3m2b1_primary_profile(primary_terms);
        }
        #[cfg(test)]
        StatementProfile::Task258B3M2B2A => {
            return exact_task258b3m2b2a_primary_profile(primary_terms);
        }
        #[cfg(test)]
        StatementProfile::Task258B3M2B2B1A => {
            return exact_task258b3m2b2b1a_primary_profile(primary_terms);
        }
        #[cfg(test)]
        StatementProfile::Task258B3M2B2B1B1 => {
            return exact_task258b3m2b2b1b1_primary_profile(primary_terms);
        }
        StatementProfile::Task258B3M2B2B2A => {
            return exact_task258b3m2b2b2a_primary_profile(primary_terms);
        }
        StatementProfile::Task258B3M2B2B2B => {
            return exact_task258b3m2b2b2b_primary_profile(primary_terms);
        }
        #[cfg(test)]
        StatementProfile::Task258B3M2B2B2C => {
            return exact_task258b3m2b2b2c_primary_profile(primary_terms);
        }
        #[cfg(test)]
        StatementProfile::Task258B3M2B2B3A => {
            return exact_task258b3m2b2b3a_primary_profile(primary_terms);
        }
        #[cfg(test)]
        StatementProfile::Task258B3M2B2B3B => {
            return exact_task258b3m2b2b3b_primary_profile(primary_terms);
        }
        #[cfg(test)]
        StatementProfile::Task258B3M2B2B3C => {
            return exact_task258b3m2b2b3c_primary_profile(primary_terms);
        }
        #[cfg(test)]
        StatementProfile::Task258B3M2B2B3D => {
            return exact_task258b3m2b2b3d_primary_profile(primary_terms);
        }
        #[cfg(test)]
        StatementProfile::Task258B3M2B2B3E => {
            return exact_task258b3m2b2b3e_primary_profile(primary_terms);
        }
        _ => {}
    }
    let expected_ranges: &[(usize, usize)] = match profile {
        #[cfg(test)]
        StatementProfile::Task258A => &[(74, 75), (78, 79)],
        #[cfg(test)]
        StatementProfile::Task258B1 => &[
            (63, 64),
            (67, 68),
            (80, 81),
            (84, 85),
            (101, 102),
            (105, 106),
            (122, 123),
            (126, 127),
        ],
        #[cfg(test)]
        StatementProfile::Task258B5A => &[
            (81, 82),
            (85, 86),
            (98, 99),
            (102, 103),
            (119, 120),
            (123, 124),
            (140, 141),
            (144, 145),
            (161, 162),
            (165, 166),
        ],
        StatementProfile::Task258B5B => &[(108, 109), (112, 113), (127, 128), (131, 132)],
        #[cfg(test)]
        StatementProfile::Task258B2 => &[
            (66, 67),
            (70, 71),
            (87, 88),
            (91, 92),
            (101, 102),
            (105, 106),
        ],
        #[cfg(test)]
        StatementProfile::Task258B3 => &[(63, 64), (67, 68), (82, 83), (92, 93), (96, 97)],
        #[cfg(test)]
        StatementProfile::Task258B3N => &[(62, 63), (66, 67), (85, 86), (95, 96), (99, 100)],
        #[cfg(test)]
        StatementProfile::Task258B3M1 => &[
            (65, 66),
            (69, 70),
            (88, 89),
            (91, 92),
            (101, 102),
            (105, 106),
        ],
        #[cfg(test)]
        StatementProfile::Task258B3M2A => unreachable!("handled above"),
        #[cfg(test)]
        StatementProfile::Task258B3M2B1 => unreachable!("handled above"),
        #[cfg(test)]
        StatementProfile::Task258B3M2B2A => unreachable!("handled above"),
        #[cfg(test)]
        StatementProfile::Task258B3M2B2B1A => unreachable!("handled above"),
        #[cfg(test)]
        StatementProfile::Task258B3M2B2B1B1 => unreachable!("handled above"),
        StatementProfile::Task258B3M2B2B2A => unreachable!("handled above"),
        StatementProfile::Task258B3M2B2B2B => unreachable!("handled above"),
        #[cfg(test)]
        StatementProfile::Task258B3M2B2B2C => unreachable!("handled above"),
        #[cfg(test)]
        StatementProfile::Task258B3M2B2B3A => unreachable!("handled above"),
        #[cfg(test)]
        StatementProfile::Task258B3M2B2B3B => unreachable!("handled above"),
        #[cfg(test)]
        StatementProfile::Task258B3M2B2B3C => unreachable!("handled above"),
        #[cfg(test)]
        StatementProfile::Task258B3M2B2B3D => unreachable!("handled above"),
        #[cfg(test)]
        StatementProfile::Task258B3M2B2B3E => unreachable!("handled above"),
    };
    let expected_contexts: &[usize] = match profile {
        #[cfg(test)]
        StatementProfile::Task258A => &[0, 0],
        #[cfg(test)]
        StatementProfile::Task258B1 => &[0, 0, 1, 1, 2, 2, 1, 1],
        #[cfg(test)]
        StatementProfile::Task258B5A => &[0, 0, 1, 1, 2, 2, 1, 1, 3, 3],
        StatementProfile::Task258B5B => &[0, 0, 1, 1],
        #[cfg(test)]
        StatementProfile::Task258B2 => &[0, 0, 1, 1, 1, 1],
        #[cfg(test)]
        StatementProfile::Task258B3 => &[0, 0, 1, 1, 1],
        #[cfg(test)]
        StatementProfile::Task258B3N => &[0, 0, 1, 1, 1],
        #[cfg(test)]
        StatementProfile::Task258B3M1 => &[0, 0, 1, 1, 1, 1],
        #[cfg(test)]
        StatementProfile::Task258B3M2A => unreachable!("handled above"),
        #[cfg(test)]
        StatementProfile::Task258B3M2B1 => unreachable!("handled above"),
        #[cfg(test)]
        StatementProfile::Task258B3M2B2A => unreachable!("handled above"),
        #[cfg(test)]
        StatementProfile::Task258B3M2B2B1A => unreachable!("handled above"),
        #[cfg(test)]
        StatementProfile::Task258B3M2B2B1B1 => unreachable!("handled above"),
        StatementProfile::Task258B3M2B2B2A => unreachable!("handled above"),
        StatementProfile::Task258B3M2B2B2B => unreachable!("handled above"),
        #[cfg(test)]
        StatementProfile::Task258B3M2B2B2C => unreachable!("handled above"),
        #[cfg(test)]
        StatementProfile::Task258B3M2B2B3A => unreachable!("handled above"),
        #[cfg(test)]
        StatementProfile::Task258B3M2B2B3B => unreachable!("handled above"),
        #[cfg(test)]
        StatementProfile::Task258B3M2B2B3C => unreachable!("handled above"),
        #[cfg(test)]
        StatementProfile::Task258B3M2B2B3D => unreachable!("handled above"),
        #[cfg(test)]
        StatementProfile::Task258B3M2B2B3E => unreachable!("handled above"),
    };
    let expected_scopes: &[&[u32]] = match profile {
        #[cfg(test)]
        StatementProfile::Task258A => &[&[], &[]],
        #[cfg(test)]
        StatementProfile::Task258B1 => &[&[], &[], &[0], &[0], &[0, 0], &[0, 0], &[0], &[0]],
        #[cfg(test)]
        StatementProfile::Task258B5A => &[
            &[],
            &[],
            &[0],
            &[0],
            &[0, 0],
            &[0, 0],
            &[0],
            &[0],
            &[0, 1],
            &[0, 1],
        ],
        StatementProfile::Task258B5B => &[&[], &[], &[0], &[0]],
        #[cfg(test)]
        StatementProfile::Task258B2 => &[&[], &[], &[0], &[0], &[0], &[0]],
        #[cfg(test)]
        StatementProfile::Task258B3 => &[&[], &[], &[0], &[0], &[0]],
        #[cfg(test)]
        StatementProfile::Task258B3N => &[&[], &[], &[0], &[0], &[0]],
        #[cfg(test)]
        StatementProfile::Task258B3M1 => &[&[], &[], &[0], &[0], &[0], &[0]],
        #[cfg(test)]
        StatementProfile::Task258B3M2A => unreachable!("handled above"),
        #[cfg(test)]
        StatementProfile::Task258B3M2B1 => unreachable!("handled above"),
        #[cfg(test)]
        StatementProfile::Task258B3M2B2A => unreachable!("handled above"),
        #[cfg(test)]
        StatementProfile::Task258B3M2B2B1A => unreachable!("handled above"),
        #[cfg(test)]
        StatementProfile::Task258B3M2B2B1B1 => unreachable!("handled above"),
        StatementProfile::Task258B3M2B2B2A => unreachable!("handled above"),
        StatementProfile::Task258B3M2B2B2B => unreachable!("handled above"),
        #[cfg(test)]
        StatementProfile::Task258B3M2B2B2C => unreachable!("handled above"),
        #[cfg(test)]
        StatementProfile::Task258B3M2B2B3A => unreachable!("handled above"),
        #[cfg(test)]
        StatementProfile::Task258B3M2B2B3B => unreachable!("handled above"),
        #[cfg(test)]
        StatementProfile::Task258B3M2B2B3C => unreachable!("handled above"),
        #[cfg(test)]
        StatementProfile::Task258B3M2B2B3D => unreachable!("handled above"),
        #[cfg(test)]
        StatementProfile::Task258B3M2B2B3E => unreachable!("handled above"),
    };
    if primary_terms.terms().len() != expected_ranges.len()
        || primary_terms.references().len() != expected_ranges.len()
        || !primary_terms.numeric_type_requests().is_empty()
    {
        return false;
    }
    for (index, expected_range) in expected_ranges.iter().copied().enumerate() {
        let Some(term) = primary_terms.terms().get(SourcePrimaryTermId::new(index)) else {
            return false;
        };
        if term.source_range()
            != range(
                primary_terms.source_id(),
                expected_range.0,
                expected_range.1,
            )
            || term.source_ordinal() != index
            || term.context() != BindingContextId::new(expected_contexts[index])
            || term.recovery() != SourcePrimaryTermRecovery::Normal
            || term.spelling() != "x"
            || term.kind() != SourcePrimaryTermKind::VariableReference
            || term.role() != SourcePrimaryTermRole::Value
            || term.parent().is_some()
        {
            return false;
        }
        let Some(reference) = primary_terms
            .references()
            .get(SourcePrimaryTermReferenceId::new(index))
        else {
            return false;
        };
        if reference.term() != SourcePrimaryTermId::new(index)
            || reference.binding() != BindingId::new(0)
            || reference.role() != SourcePrimaryTermReferenceRole::Variable
            || reference
                .lexical_scope()
                .map_or(&[][..], |scope| scope.path())
                != expected_scopes[index]
            || reference.use_ordinal() != 1
        {
            return false;
        }
    }
    true
}

#[cfg(test)]
fn exact_task258b3m2a_primary_profile(primary_terms: &SourcePrimaryTermHandoff) -> bool {
    let expected_sites = [26, 28, 32, 36, 38];
    let expected_ranges = [(64, 65), (68, 69), (83, 86), (95, 96), (99, 100)];
    let expected_contexts = [0, 0, 1, 1, 1];
    if primary_terms.terms().len() != 5
        || primary_terms.references().len() != 4
        || primary_terms.numeric_type_requests().len() != 1
    {
        return false;
    }
    for index in 0..5 {
        let Some(term) = primary_terms.terms().get(SourcePrimaryTermId::new(index)) else {
            return false;
        };
        let numeral = index == 2;
        if term.site().node().index() != expected_sites[index]
            || term.source_range()
                != range(
                    primary_terms.source_id(),
                    expected_ranges[index].0,
                    expected_ranges[index].1,
                )
            || term.source_ordinal() != index
            || term.context() != BindingContextId::new(expected_contexts[index])
            || term.recovery() != SourcePrimaryTermRecovery::Normal
            || term.spelling() != if numeral { "101" } else { "x" }
            || term.kind()
                != if numeral {
                    SourcePrimaryTermKind::Numeral
                } else {
                    SourcePrimaryTermKind::VariableReference
                }
            || term.role() != SourcePrimaryTermRole::Value
            || term.parent().is_some()
        {
            return false;
        }
    }
    for (reference_index, term_index) in [0, 1, 3, 4].into_iter().enumerate() {
        let Some(reference) = primary_terms
            .references()
            .get(SourcePrimaryTermReferenceId::new(reference_index))
        else {
            return false;
        };
        if reference.term() != SourcePrimaryTermId::new(term_index)
            || reference.binding() != BindingId::new(0)
            || reference.role() != SourcePrimaryTermReferenceRole::Variable
            || reference.use_ordinal() != 1
            || reference.lexical_scope().map(|scope| scope.path())
                != if term_index < 2 { None } else { Some(&[0][..]) }
        {
            return false;
        }
    }
    let Some(request) = primary_terms
        .numeric_type_requests()
        .get(crate::source_term::SourceNumericTypeRequestId::new(0))
    else {
        return false;
    };
    request.term() == SourcePrimaryTermId::new(2)
        && request.owner().node().index() == 32
        && request.source_range() == range(primary_terms.source_id(), 83, 86)
        && request.spelling() == "101"
        && request.request_ordinal() == 0
}

#[cfg(test)]
fn exact_task258b3m2b1_primary_profile(primary_terms: &SourcePrimaryTermHandoff) -> bool {
    let expected_sites = [28, 30, 36, 34, 40, 42];
    let expected_ranges = [
        (70, 71),
        (74, 75),
        (89, 92),
        (90, 91),
        (101, 102),
        (105, 106),
    ];
    let expected_contexts = [0, 0, 1, 1, 1, 1];
    if primary_terms.terms().len() != 6
        || primary_terms.references().len() != 5
        || !primary_terms.numeric_type_requests().is_empty()
    {
        return false;
    }
    for index in 0..6 {
        let Some(term) = primary_terms.terms().get(SourcePrimaryTermId::new(index)) else {
            return false;
        };
        let parenthesized = index == 2;
        if term.site().node().index() != expected_sites[index]
            || term.source_range()
                != range(
                    primary_terms.source_id(),
                    expected_ranges[index].0,
                    expected_ranges[index].1,
                )
            || term.source_ordinal() != index
            || term.context() != BindingContextId::new(expected_contexts[index])
            || term.recovery() != SourcePrimaryTermRecovery::Normal
            || term.spelling() != if parenthesized { "( x )" } else { "x" }
            || term.kind()
                != if parenthesized {
                    SourcePrimaryTermKind::Parenthesized
                } else {
                    SourcePrimaryTermKind::VariableReference
                }
            || term.role() != SourcePrimaryTermRole::Value
            || term.parent()
                != if index == 3 {
                    Some(SourcePrimaryTermId::new(2))
                } else {
                    None
                }
        {
            return false;
        }
    }
    for (reference_index, term_index) in [0, 1, 3, 4, 5].into_iter().enumerate() {
        let Some(reference) = primary_terms
            .references()
            .get(SourcePrimaryTermReferenceId::new(reference_index))
        else {
            return false;
        };
        if reference.term() != SourcePrimaryTermId::new(term_index)
            || reference.binding() != BindingId::new(0)
            || reference.role() != SourcePrimaryTermReferenceRole::Variable
            || reference.use_ordinal() != 1
            || reference.lexical_scope().map(|scope| scope.path())
                != if term_index < 2 { None } else { Some(&[0][..]) }
        {
            return false;
        }
    }
    true
}

#[cfg(test)]
fn exact_task258b3m2b2a_primary_profile(primary_terms: &SourcePrimaryTermHandoff) -> bool {
    let expected_sites = [30, 32, 40, 38, 36, 44, 46];
    let expected_ranges = [
        (76, 77),
        (80, 81),
        (95, 100),
        (96, 99),
        (97, 98),
        (109, 110),
        (113, 114),
    ];
    if primary_terms.terms().len() != 7
        || primary_terms.references().len() != 5
        || !primary_terms.numeric_type_requests().is_empty()
    {
        return false;
    }
    for index in 0..7 {
        let Some(term) = primary_terms.terms().get(SourcePrimaryTermId::new(index)) else {
            return false;
        };
        let parenthesized = matches!(index, 2 | 3);
        let expected_parent = match index {
            3 => Some(SourcePrimaryTermId::new(2)),
            4 => Some(SourcePrimaryTermId::new(3)),
            _ => None,
        };
        if term.site().node().index() != expected_sites[index]
            || term.source_range()
                != range(
                    primary_terms.source_id(),
                    expected_ranges[index].0,
                    expected_ranges[index].1,
                )
            || term.source_ordinal() != index
            || term.context() != BindingContextId::new(usize::from(index >= 2))
            || term.recovery() != SourcePrimaryTermRecovery::Normal
            || term.spelling()
                != match index {
                    2 => "( ( x ) )",
                    3 => "( x )",
                    _ => "x",
                }
            || term.kind()
                != if parenthesized {
                    SourcePrimaryTermKind::Parenthesized
                } else {
                    SourcePrimaryTermKind::VariableReference
                }
            || term.role() != SourcePrimaryTermRole::Value
            || term.parent() != expected_parent
        {
            return false;
        }
    }
    for (reference_index, term_index) in [0, 1, 4, 5, 6].into_iter().enumerate() {
        let Some(reference) = primary_terms
            .references()
            .get(SourcePrimaryTermReferenceId::new(reference_index))
        else {
            return false;
        };
        if reference.term() != SourcePrimaryTermId::new(term_index)
            || reference.binding() != BindingId::new(0)
            || reference.role() != SourcePrimaryTermReferenceRole::Variable
            || reference.use_ordinal() != 1
            || reference.lexical_scope().map(|scope| scope.path())
                != if term_index < 2 { None } else { Some(&[0][..]) }
        {
            return false;
        }
    }
    true
}

#[cfg(test)]
fn exact_task258b3m2b2b1a_primary_profile(primary_terms: &SourcePrimaryTermHandoff) -> bool {
    let expected_sites = [38, 40, 44, 45, 50, 52];
    let expected_ranges = [
        (97, 98),
        (101, 102),
        (116, 117),
        (121, 122),
        (131, 132),
        (135, 136),
    ];
    if primary_terms.terms().len() != 6
        || primary_terms.references().len() != 4
        || primary_terms.numeric_type_requests().len() != 2
    {
        return false;
    }
    for index in 0..6 {
        let Some(term) = primary_terms.terms().get(SourcePrimaryTermId::new(index)) else {
            return false;
        };
        let numeral = matches!(index, 2 | 3);
        let expected_spelling = match index {
            2 => "1",
            3 => "2",
            _ => "x",
        };
        if term.site().node().index() != expected_sites[index]
            || term.source_range()
                != range(
                    primary_terms.source_id(),
                    expected_ranges[index].0,
                    expected_ranges[index].1,
                )
            || term.source_ordinal() != index
            || term.context() != BindingContextId::new(usize::from(index >= 2))
            || term.recovery() != SourcePrimaryTermRecovery::Normal
            || term.spelling() != expected_spelling
            || term.kind()
                != if numeral {
                    SourcePrimaryTermKind::Numeral
                } else {
                    SourcePrimaryTermKind::VariableReference
                }
            || term.role() != SourcePrimaryTermRole::Value
            || term.parent().is_some()
        {
            return false;
        }
    }
    for (reference_index, term_index) in [0, 1, 4, 5].into_iter().enumerate() {
        let Some(reference) = primary_terms
            .references()
            .get(SourcePrimaryTermReferenceId::new(reference_index))
        else {
            return false;
        };
        if reference.term() != SourcePrimaryTermId::new(term_index)
            || reference.binding() != BindingId::new(0)
            || reference.role() != SourcePrimaryTermReferenceRole::Variable
            || reference.use_ordinal() != 1
            || reference.lexical_scope().map(|scope| scope.path())
                != if term_index < 2 { None } else { Some(&[0][..]) }
        {
            return false;
        }
    }
    for (request_index, term_index) in [2, 3].into_iter().enumerate() {
        let Some(request) = primary_terms.numeric_type_requests().get(
            crate::source_term::SourceNumericTypeRequestId::new(request_index),
        ) else {
            return false;
        };
        if request.term() != SourcePrimaryTermId::new(term_index)
            || request.owner().node().index() != expected_sites[term_index]
            || request.source_range()
                != range(
                    primary_terms.source_id(),
                    expected_ranges[term_index].0,
                    expected_ranges[term_index].1,
                )
            || request.spelling() != if term_index == 2 { "1" } else { "2" }
            || request.request_ordinal() != request_index
        {
            return false;
        }
    }
    true
}

#[cfg(test)]
fn exact_task258b3m2b2b1b1_primary_profile(primary_terms: &SourcePrimaryTermHandoff) -> bool {
    let expected_sites = [40, 42, 46, 47, 54, 56];
    let expected_ranges = [
        (110, 111),
        (114, 115),
        (130, 131),
        (135, 136),
        (146, 147),
        (150, 151),
    ];
    if primary_terms.terms().len() != 6
        || primary_terms.references().len() != 4
        || primary_terms.numeric_type_requests().len() != 2
    {
        return false;
    }
    for index in 0..6 {
        let Some(term) = primary_terms.terms().get(SourcePrimaryTermId::new(index)) else {
            return false;
        };
        let numeral = matches!(index, 2 | 3);
        let expected_spelling = match index {
            2 => "1",
            3 => "2",
            _ => "x",
        };
        if term.site().node().index() != expected_sites[index]
            || term.source_range()
                != range(
                    primary_terms.source_id(),
                    expected_ranges[index].0,
                    expected_ranges[index].1,
                )
            || term.source_ordinal() != index
            || term.context() != BindingContextId::new(usize::from(index >= 2))
            || term.recovery() != SourcePrimaryTermRecovery::Normal
            || term.spelling() != expected_spelling
            || term.kind()
                != if numeral {
                    SourcePrimaryTermKind::Numeral
                } else {
                    SourcePrimaryTermKind::VariableReference
                }
            || term.role() != SourcePrimaryTermRole::Value
            || term.parent().is_some()
        {
            return false;
        }
    }
    for (reference_index, term_index) in [0, 1, 4, 5].into_iter().enumerate() {
        let Some(reference) = primary_terms
            .references()
            .get(SourcePrimaryTermReferenceId::new(reference_index))
        else {
            return false;
        };
        if reference.term() != SourcePrimaryTermId::new(term_index)
            || reference.binding() != BindingId::new(0)
            || reference.role() != SourcePrimaryTermReferenceRole::Variable
            || reference.use_ordinal() != 1
            || reference.lexical_scope().map(|scope| scope.path())
                != if term_index < 2 { None } else { Some(&[0][..]) }
        {
            return false;
        }
    }
    for (request_index, term_index) in [2, 3].into_iter().enumerate() {
        let Some(request) = primary_terms.numeric_type_requests().get(
            crate::source_term::SourceNumericTypeRequestId::new(request_index),
        ) else {
            return false;
        };
        if request.term() != SourcePrimaryTermId::new(term_index)
            || request.owner().node().index() != expected_sites[term_index]
            || request.source_range()
                != range(
                    primary_terms.source_id(),
                    expected_ranges[term_index].0,
                    expected_ranges[term_index].1,
                )
            || request.spelling() != if term_index == 2 { "1" } else { "2" }
            || request.request_ordinal() != request_index
        {
            return false;
        }
    }
    true
}

fn exact_task258b3m2b2b2a_primary_profile(primary_terms: &SourcePrimaryTermHandoff) -> bool {
    exact_structure_witness_primary_profile(
        primary_terms,
        [45, 47, 53, 56, 63, 65],
        [
            (106, 107),
            (110, 111),
            (143, 144),
            (149, 150),
            (160, 161),
            (164, 165),
        ],
    )
}

fn exact_task258b3m2b2b2b_primary_profile(primary_terms: &SourcePrimaryTermHandoff) -> bool {
    exact_structure_witness_primary_profile(
        primary_terms,
        [47, 49, 55, 58, 66, 68],
        [
            (103, 104),
            (107, 108),
            (140, 141),
            (146, 147),
            (159, 160),
            (163, 164),
        ],
    )
}

#[cfg(test)]
fn exact_task258b3m2b2b2c_primary_profile(primary_terms: &SourcePrimaryTermHandoff) -> bool {
    let expected_sites = [51, 53, 59, 62, 66, 73, 75];
    let expected_ranges = [
        (101, 102),
        (105, 106),
        (138, 139),
        (144, 145),
        (158, 159),
        (169, 170),
        (173, 174),
    ];
    if primary_terms.terms().len() != 7
        || primary_terms.references().len() != 4
        || primary_terms.numeric_type_requests().len() != 3
    {
        return false;
    }
    for index in 0..7 {
        let Some(term) = primary_terms.terms().get(SourcePrimaryTermId::new(index)) else {
            return false;
        };
        let is_reference = matches!(index, 0 | 1 | 5 | 6);
        if term.site().node().index() != expected_sites[index]
            || term.source_range()
                != range(
                    primary_terms.source_id(),
                    expected_ranges[index].0,
                    expected_ranges[index].1,
                )
            || term.source_ordinal() != index
            || term.context() != BindingContextId::new(usize::from(index >= 2))
            || term.spelling()
                != if is_reference {
                    "x"
                } else {
                    match index {
                        2 => "1",
                        3 => "2",
                        4 => "3",
                        _ => unreachable!("only three numerals"),
                    }
                }
            || term.kind()
                != if is_reference {
                    SourcePrimaryTermKind::VariableReference
                } else {
                    SourcePrimaryTermKind::Numeral
                }
            || term.role() != SourcePrimaryTermRole::Value
            || term.recovery() != SourcePrimaryTermRecovery::Normal
            || term.parent().is_some()
        {
            return false;
        }
    }
    for (reference_index, term_index) in [0, 1, 5, 6].into_iter().enumerate() {
        let Some(reference) = primary_terms
            .references()
            .get(SourcePrimaryTermReferenceId::new(reference_index))
        else {
            return false;
        };
        if reference.term() != SourcePrimaryTermId::new(term_index)
            || reference.binding() != BindingId::new(0)
            || reference.role() != SourcePrimaryTermReferenceRole::Variable
            || reference.use_ordinal() != 1
            || reference.lexical_scope().map(|scope| scope.path())
                != if term_index < 2 { None } else { Some(&[0][..]) }
        {
            return false;
        }
    }
    for (request_index, term_index) in [2, 3, 4].into_iter().enumerate() {
        let Some(request) = primary_terms.numeric_type_requests().get(
            crate::source_term::SourceNumericTypeRequestId::new(request_index),
        ) else {
            return false;
        };
        if request.term() != SourcePrimaryTermId::new(term_index)
            || request.owner().node().index() != expected_sites[term_index]
            || request.source_range()
                != range(
                    primary_terms.source_id(),
                    expected_ranges[term_index].0,
                    expected_ranges[term_index].1,
                )
            || request.spelling()
                != match term_index {
                    2 => "1",
                    3 => "2",
                    4 => "3",
                    _ => unreachable!("only three numerals"),
                }
            || request.request_ordinal() != request_index
        {
            return false;
        }
    }
    true
}

#[cfg(test)]
fn exact_task258b3m2b2b3a_primary_profile(primary_terms: &SourcePrimaryTermHandoff) -> bool {
    exact_structure_witness_primary_profile(
        primary_terms,
        [30, 32, 36, 38, 44, 46],
        [
            (71, 72),
            (75, 76),
            (91, 92),
            (94, 95),
            (105, 106),
            (109, 110),
        ],
    )
}

#[cfg(test)]
fn exact_task258b3m2b2b3b_primary_profile(primary_terms: &SourcePrimaryTermHandoff) -> bool {
    const SITES: [usize; 4] = [27, 29, 37, 39];
    const RANGES: [(usize, usize); 4] = [(76, 77), (80, 81), (106, 107), (110, 111)];
    if primary_terms.terms().len() != 4
        || primary_terms.references().len() != 4
        || !primary_terms.numeric_type_requests().is_empty()
    {
        return false;
    }
    for index in 0..4 {
        let Some(term) = primary_terms.terms().get(SourcePrimaryTermId::new(index)) else {
            return false;
        };
        let context = usize::from(index >= 2);
        if term.site().node().index() != SITES[index]
            || term.source_range()
                != range(primary_terms.source_id(), RANGES[index].0, RANGES[index].1)
            || term.source_ordinal() != index
            || term.context() != BindingContextId::new(context)
            || term.recovery() != SourcePrimaryTermRecovery::Normal
            || term.spelling() != "x"
            || term.kind() != SourcePrimaryTermKind::VariableReference
            || term.role() != SourcePrimaryTermRole::Value
            || term.parent().is_some()
        {
            return false;
        }
        let Some(reference) = primary_terms
            .references()
            .get(SourcePrimaryTermReferenceId::new(index))
        else {
            return false;
        };
        if reference.term() != SourcePrimaryTermId::new(index)
            || reference.binding() != BindingId::new(0)
            || reference.role() != SourcePrimaryTermReferenceRole::Variable
            || reference.use_ordinal() != 1
            || reference.lexical_scope().map(|scope| scope.path())
                != if context == 0 { None } else { Some(&[0][..]) }
        {
            return false;
        }
    }
    true
}

#[cfg(test)]
fn exact_task258b3m2b2b3c_primary_profile(primary_terms: &SourcePrimaryTermHandoff) -> bool {
    const SITES: [usize; 4] = [27, 29, 39, 41];
    const RANGES: [(usize, usize); 4] = [(63, 64), (67, 68), (98, 99), (102, 103)];
    if primary_terms.terms().len() != 4
        || primary_terms.references().len() != 4
        || !primary_terms.numeric_type_requests().is_empty()
    {
        return false;
    }
    for index in 0..4 {
        let Some(term) = primary_terms.terms().get(SourcePrimaryTermId::new(index)) else {
            return false;
        };
        let context = usize::from(index >= 2);
        if term.site().node().index() != SITES[index]
            || term.source_range()
                != range(primary_terms.source_id(), RANGES[index].0, RANGES[index].1)
            || term.source_ordinal() != index
            || term.context() != BindingContextId::new(context)
            || term.recovery() != SourcePrimaryTermRecovery::Normal
            || term.spelling() != "x"
            || term.kind() != SourcePrimaryTermKind::VariableReference
            || term.role() != SourcePrimaryTermRole::Value
            || term.parent().is_some()
        {
            return false;
        }
        let Some(reference) = primary_terms
            .references()
            .get(SourcePrimaryTermReferenceId::new(index))
        else {
            return false;
        };
        if reference.term() != SourcePrimaryTermId::new(index)
            || reference.binding() != BindingId::new(0)
            || reference.role() != SourcePrimaryTermReferenceRole::Variable
            || reference.use_ordinal() != 1
            || reference.lexical_scope().map(|scope| scope.path())
                != if context == 0 { None } else { Some(&[0][..]) }
        {
            return false;
        }
    }
    true
}

#[cfg(test)]
fn exact_task258b3m2b2b3d_primary_profile(primary_terms: &SourcePrimaryTermHandoff) -> bool {
    const SITES: [usize; 5] = [28, 30, 34, 41, 43];
    const RANGES: [(usize, usize); 5] = [(60, 61), (64, 65), (79, 80), (97, 98), (101, 102)];
    if primary_terms.terms().len() != 5
        || primary_terms.references().len() != 4
        || primary_terms.numeric_type_requests().len() != 1
    {
        return false;
    }
    for index in 0..5 {
        let Some(term) = primary_terms.terms().get(SourcePrimaryTermId::new(index)) else {
            return false;
        };
        let numeral = index == 2;
        if term.site().node().index() != SITES[index]
            || term.source_range()
                != range(primary_terms.source_id(), RANGES[index].0, RANGES[index].1)
            || term.source_ordinal() != index
            || term.context() != BindingContextId::new(usize::from(index >= 2))
            || term.recovery() != SourcePrimaryTermRecovery::Normal
            || term.spelling() != if numeral { "4" } else { "x" }
            || term.kind()
                != if numeral {
                    SourcePrimaryTermKind::Numeral
                } else {
                    SourcePrimaryTermKind::VariableReference
                }
            || term.role() != SourcePrimaryTermRole::Value
            || term.parent().is_some()
        {
            return false;
        }
    }
    for (reference_index, term_index) in [0, 1, 3, 4].into_iter().enumerate() {
        let Some(reference) = primary_terms
            .references()
            .get(SourcePrimaryTermReferenceId::new(reference_index))
        else {
            return false;
        };
        if reference.term() != SourcePrimaryTermId::new(term_index)
            || reference.binding() != BindingId::new(0)
            || reference.role() != SourcePrimaryTermReferenceRole::Variable
            || reference.use_ordinal() != 1
            || reference.lexical_scope().map(|scope| scope.path())
                != if term_index < 2 { None } else { Some(&[0][..]) }
        {
            return false;
        }
    }
    let Some(request) = primary_terms
        .numeric_type_requests()
        .get(crate::source_term::SourceNumericTypeRequestId::new(0))
    else {
        return false;
    };
    request.term() == SourcePrimaryTermId::new(2)
        && request.owner().node().index() == 34
        && request.source_range() == range(primary_terms.source_id(), 79, 80)
        && request.spelling() == "4"
        && request.request_ordinal() == 0
}

#[cfg(test)]
fn exact_task258b3m2b2b3e_primary_profile(primary_terms: &SourcePrimaryTermHandoff) -> bool {
    const SITES: [usize; 5] = [32, 34, 38, 47, 49];
    const RANGES: [(usize, usize); 5] = [(70, 71), (74, 75), (90, 91), (127, 128), (131, 132)];
    if primary_terms.terms().len() != 5
        || primary_terms.references().len() != 4
        || primary_terms.numeric_type_requests().len() != 1
    {
        return false;
    }
    for index in 0..5 {
        let Some(term) = primary_terms.terms().get(SourcePrimaryTermId::new(index)) else {
            return false;
        };
        let numeral = index == 2;
        if term.site().node().index() != SITES[index]
            || term.source_range()
                != range(primary_terms.source_id(), RANGES[index].0, RANGES[index].1)
            || term.source_ordinal() != index
            || term.context() != BindingContextId::new(usize::from(index >= 2))
            || term.recovery() != SourcePrimaryTermRecovery::Normal
            || term.spelling() != if numeral { "3" } else { "x" }
            || term.kind()
                != if numeral {
                    SourcePrimaryTermKind::Numeral
                } else {
                    SourcePrimaryTermKind::VariableReference
                }
            || term.role() != SourcePrimaryTermRole::Value
            || term.parent().is_some()
        {
            return false;
        }
    }
    for (reference_index, term_index) in [0, 1, 3, 4].into_iter().enumerate() {
        let Some(reference) = primary_terms
            .references()
            .get(SourcePrimaryTermReferenceId::new(reference_index))
        else {
            return false;
        };
        if reference.term() != SourcePrimaryTermId::new(term_index)
            || reference.binding() != BindingId::new(0)
            || reference.role() != SourcePrimaryTermReferenceRole::Variable
            || reference.use_ordinal() != 1
            || reference.lexical_scope().map(|scope| scope.path())
                != if term_index < 2 { None } else { Some(&[0][..]) }
        {
            return false;
        }
    }
    let Some(request) = primary_terms
        .numeric_type_requests()
        .get(crate::source_term::SourceNumericTypeRequestId::new(0))
    else {
        return false;
    };
    request.term() == SourcePrimaryTermId::new(2)
        && request.owner().node().index() == 38
        && request.source_range() == range(primary_terms.source_id(), 90, 91)
        && request.spelling() == "3"
        && request.request_ordinal() == 0
}

fn exact_structure_witness_primary_profile(
    primary_terms: &SourcePrimaryTermHandoff,
    expected_sites: [usize; 6],
    expected_ranges: [(usize, usize); 6],
) -> bool {
    if primary_terms.terms().len() != 6
        || primary_terms.references().len() != 4
        || primary_terms.numeric_type_requests().len() != 2
    {
        return false;
    }
    for index in 0..6 {
        let Some(term) = primary_terms.terms().get(SourcePrimaryTermId::new(index)) else {
            return false;
        };
        let numeral = matches!(index, 2 | 3);
        let expected_spelling = match index {
            2 => "1",
            3 => "2",
            _ => "x",
        };
        if term.site().node().index() != expected_sites[index]
            || term.source_range()
                != range(
                    primary_terms.source_id(),
                    expected_ranges[index].0,
                    expected_ranges[index].1,
                )
            || term.source_ordinal() != index
            || term.context() != BindingContextId::new(usize::from(index >= 2))
            || term.recovery() != SourcePrimaryTermRecovery::Normal
            || term.spelling() != expected_spelling
            || term.kind()
                != if numeral {
                    SourcePrimaryTermKind::Numeral
                } else {
                    SourcePrimaryTermKind::VariableReference
                }
            || term.role() != SourcePrimaryTermRole::Value
            || term.parent().is_some()
        {
            return false;
        }
    }
    for (reference_index, term_index) in [0, 1, 4, 5].into_iter().enumerate() {
        let Some(reference) = primary_terms
            .references()
            .get(SourcePrimaryTermReferenceId::new(reference_index))
        else {
            return false;
        };
        if reference.term() != SourcePrimaryTermId::new(term_index)
            || reference.binding() != BindingId::new(0)
            || reference.role() != SourcePrimaryTermReferenceRole::Variable
            || reference.use_ordinal() != 1
            || reference.lexical_scope().map(|scope| scope.path())
                != if term_index < 2 { None } else { Some(&[0][..]) }
        {
            return false;
        }
    }
    for (request_index, term_index) in [2, 3].into_iter().enumerate() {
        let Some(request) = primary_terms.numeric_type_requests().get(
            crate::source_term::SourceNumericTypeRequestId::new(request_index),
        ) else {
            return false;
        };
        if request.term() != SourcePrimaryTermId::new(term_index)
            || request.owner().node().index() != expected_sites[term_index]
            || request.source_range()
                != range(
                    primary_terms.source_id(),
                    expected_ranges[term_index].0,
                    expected_ranges[term_index].1,
                )
            || request.spelling() != if term_index == 2 { "1" } else { "2" }
            || request.request_ordinal() != request_index
        {
            return false;
        }
    }
    true
}

fn exact_atomic_profile(
    profile: StatementProfile,
    atomic_formulas: &SourceAtomicFormulaHandoff,
    primary_terms: &SourcePrimaryTermHandoff,
    arena: &TypedArena,
) -> bool {
    let expected_ranges: &[(usize, usize)] = match profile {
        #[cfg(test)]
        StatementProfile::Task258A => &[(74, 79)],
        #[cfg(test)]
        StatementProfile::Task258B1 => &[(63, 68), (80, 85), (101, 106), (122, 127)],
        #[cfg(test)]
        StatementProfile::Task258B5A => &[(81, 86), (98, 103), (119, 124), (140, 145), (161, 166)],
        StatementProfile::Task258B5B => &[(108, 113), (127, 132)],
        #[cfg(test)]
        StatementProfile::Task258B2 => &[(66, 71), (87, 92), (101, 106)],
        #[cfg(test)]
        StatementProfile::Task258B3 => &[(63, 68), (92, 97)],
        #[cfg(test)]
        StatementProfile::Task258B3N => &[(62, 67), (95, 100)],
        #[cfg(test)]
        StatementProfile::Task258B3M1 => &[(65, 70), (101, 106)],
        #[cfg(test)]
        StatementProfile::Task258B3M2A => &[(64, 69), (95, 100)],
        #[cfg(test)]
        StatementProfile::Task258B3M2B1 => &[(70, 75), (101, 106)],
        #[cfg(test)]
        StatementProfile::Task258B3M2B2A => &[(76, 81), (109, 114)],
        #[cfg(test)]
        StatementProfile::Task258B3M2B2B1A => &[(97, 102), (131, 136)],
        #[cfg(test)]
        StatementProfile::Task258B3M2B2B1B1 => &[(110, 115), (146, 151)],
        StatementProfile::Task258B3M2B2B2A => &[(106, 111), (160, 165)],
        StatementProfile::Task258B3M2B2B2B => &[(103, 108), (159, 164)],
        #[cfg(test)]
        StatementProfile::Task258B3M2B2B2C => &[(101, 106), (169, 174)],
        #[cfg(test)]
        StatementProfile::Task258B3M2B2B3A => &[(71, 76), (105, 110)],
        #[cfg(test)]
        StatementProfile::Task258B3M2B2B3B => &[(76, 81), (106, 111)],
        #[cfg(test)]
        StatementProfile::Task258B3M2B2B3C => &[(63, 68), (98, 103)],
        #[cfg(test)]
        StatementProfile::Task258B3M2B2B3D => &[(60, 65), (97, 102)],
        #[cfg(test)]
        StatementProfile::Task258B3M2B2B3E => &[(70, 75), (127, 132)],
    };
    let expected_contexts: &[usize] = match profile {
        #[cfg(test)]
        StatementProfile::Task258A => &[0],
        #[cfg(test)]
        StatementProfile::Task258B1 => &[0, 1, 2, 1],
        #[cfg(test)]
        StatementProfile::Task258B5A => &[0, 1, 2, 1, 3],
        StatementProfile::Task258B5B => &[0, 1],
        #[cfg(test)]
        StatementProfile::Task258B2 => &[0, 1, 1],
        #[cfg(test)]
        StatementProfile::Task258B3 => &[0, 1],
        #[cfg(test)]
        StatementProfile::Task258B3N => &[0, 1],
        #[cfg(test)]
        StatementProfile::Task258B3M1 => &[0, 1],
        #[cfg(test)]
        StatementProfile::Task258B3M2A => &[0, 1],
        #[cfg(test)]
        StatementProfile::Task258B3M2B1 => &[0, 1],
        #[cfg(test)]
        StatementProfile::Task258B3M2B2A => &[0, 1],
        #[cfg(test)]
        StatementProfile::Task258B3M2B2B1A => &[0, 1],
        #[cfg(test)]
        StatementProfile::Task258B3M2B2B1B1 => &[0, 1],
        StatementProfile::Task258B3M2B2B2A => &[0, 1],
        StatementProfile::Task258B3M2B2B2B => &[0, 1],
        #[cfg(test)]
        StatementProfile::Task258B3M2B2B2C => &[0, 1],
        #[cfg(test)]
        StatementProfile::Task258B3M2B2B3A => &[0, 1],
        #[cfg(test)]
        StatementProfile::Task258B3M2B2B3B => &[0, 1],
        #[cfg(test)]
        StatementProfile::Task258B3M2B2B3C => &[0, 1],
        #[cfg(test)]
        StatementProfile::Task258B3M2B2B3D => &[0, 1],
        #[cfg(test)]
        StatementProfile::Task258B3M2B2B3E => &[0, 1],
    };
    if atomic_formulas.formulas().len() != expected_ranges.len()
        || !atomic_formulas.wrappers().is_empty()
        || !atomic_formulas.predicate_segments().is_empty()
        || !atomic_formulas.predicate_heads().is_empty()
        || !atomic_formulas.candidates().is_empty()
        || !atomic_formulas.type_sites().is_empty()
        || !atomic_formulas.attributes().is_empty()
        || atomic_formulas.edges().len() != expected_ranges.len() * 2
        || atomic_formulas.requests().len() != expected_ranges.len() * 2
    {
        return false;
    }
    for (formula_index, expected_range) in expected_ranges.iter().copied().enumerate() {
        let formula_id = SourceAtomicFormulaId::new(formula_index);
        let Some(formula) = atomic_formulas.formulas().get(formula_id) else {
            return false;
        };
        if formula.source_range()
            != range(
                atomic_formulas.source_id(),
                expected_range.0,
                expected_range.1,
            )
            || formula.source_ordinal() != formula_index
            || formula.context() != BindingContextId::new(expected_contexts[formula_index])
            || formula.recovery() != SourceAtomicFormulaRecovery::Normal
            || formula.spelling() != "x = x"
            || formula.kind() != SourceAtomicFormulaKind::Equality
            || arena.node(formula.site().node()).is_none()
        {
            return false;
        }
        let first_term = match profile {
            #[cfg(test)]
            StatementProfile::Task258B3 if formula_index == 1 => 3,
            #[cfg(test)]
            StatementProfile::Task258B3N if formula_index == 1 => 3,
            #[cfg(test)]
            StatementProfile::Task258B3M1 if formula_index == 1 => 4,
            #[cfg(test)]
            StatementProfile::Task258B3M2A if formula_index == 1 => 3,
            #[cfg(test)]
            StatementProfile::Task258B3M2B1 if formula_index == 1 => 4,
            #[cfg(test)]
            StatementProfile::Task258B3M2B2A if formula_index == 1 => 5,
            #[cfg(test)]
            StatementProfile::Task258B3M2B2B1A if formula_index == 1 => 4,
            #[cfg(test)]
            StatementProfile::Task258B3M2B2B1B1 if formula_index == 1 => 4,
            StatementProfile::Task258B3M2B2B2A if formula_index == 1 => 4,
            StatementProfile::Task258B3M2B2B2B if formula_index == 1 => 4,
            #[cfg(test)]
            StatementProfile::Task258B3M2B2B2C if formula_index == 1 => 5,
            #[cfg(test)]
            StatementProfile::Task258B3M2B2B3A if formula_index == 1 => 4,
            #[cfg(test)]
            StatementProfile::Task258B3M2B2B3B if formula_index == 1 => 2,
            #[cfg(test)]
            StatementProfile::Task258B3M2B2B3C if formula_index == 1 => 2,
            #[cfg(test)]
            StatementProfile::Task258B3M2B2B3D if formula_index == 1 => 3,
            #[cfg(test)]
            StatementProfile::Task258B3M2B2B3E if formula_index == 1 => 3,
            _ => formula_index * 2,
        };
        let expected_edges = [
            (
                SourceAtomicEdgeRole::BuiltinLeftOperand,
                SourceAtomicTermTarget::Primary(SourcePrimaryTermId::new(first_term)),
            ),
            (
                SourceAtomicEdgeRole::BuiltinRightOperand,
                SourceAtomicTermTarget::Primary(SourcePrimaryTermId::new(first_term + 1)),
            ),
        ];
        for (edge_ordinal, (role, target)) in expected_edges.into_iter().enumerate() {
            let edge_index = formula_index * 2 + edge_ordinal;
            let edge_id = crate::source_atomic_formula::SourceAtomicEdgeId::new(edge_index);
            let Some(edge) = atomic_formulas.edges().get(edge_id) else {
                return false;
            };
            if edge.formula() != formula_id
                || edge.ordinal() != edge_ordinal
                || edge.role() != role
                || edge.target() != target
            {
                return false;
            }
            let Some(request) = atomic_formulas.requests().get(
                crate::source_atomic_formula::SourceAtomicRequestId::new(edge_index),
            ) else {
                return false;
            };
            if request.formula() != formula_id
                || request.ordinal() != edge_ordinal
                || request.kind() != SourceAtomicRequestKind::OperandExpectedType
                || request.edge() != Some(edge_id)
                || request.candidate().is_some()
                || request.type_site().is_some()
                || request.attribute().is_some()
            {
                return false;
            }
        }
        let Some(left) = primary_terms
            .terms()
            .get(SourcePrimaryTermId::new(first_term))
        else {
            return false;
        };
        let Some(right) = primary_terms
            .terms()
            .get(SourcePrimaryTermId::new(first_term + 1))
        else {
            return false;
        };
        let operands_are_ordered = if profile == StatementProfile::Task258B5B {
            matches!(
                (
                    containing_child_path(arena, formula.site().node(), left.site().node()),
                    containing_child_path(arena, formula.site().node(), right.site().node()),
                ),
                (Some(left_path), Some(right_path)) if left_path < right_path
            )
        } else {
            matches!(
                (
                    containing_child_position(arena, formula.site().node(), left.site().node()),
                    containing_child_position(arena, formula.site().node(), right.site().node()),
                ),
                (Some(left_position), Some(right_position))
                    if left_position < right_position
            )
        };
        if !operands_are_ordered {
            return false;
        }
    }
    true
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Task258B4Profile {
    QuantifierBoundUse,
    ConnectiveGrouping,
    NestedQuantifier,
}

fn task_258b4_profile(statement: &SourceStatementHandoff) -> Option<Task258B4Profile> {
    if statement.is_task_258b4a_profile() {
        Some(Task258B4Profile::QuantifierBoundUse)
    } else if statement.is_task_258b4b_profile() {
        Some(Task258B4Profile::ConnectiveGrouping)
    } else if statement.is_task_258b4c_profile() {
        Some(Task258B4Profile::NestedQuantifier)
    } else {
        None
    }
}

fn task_258b4_input_profile(input: &SourceStatementHandoffInput) -> Option<Task258B4Profile> {
    let owner = input.owners.first();
    let statement = input.statements.first();
    if owner.is_some_and(|owner| {
        owner.source_range == range(input.source_id, 0, 78)
            && owner.spelling == "FormulaQuantifierBoundUsePayloadBoundary"
    }) || statement.is_some_and(|statement| {
        statement.spelling
            == "theorem FormulaQuantifierBoundUsePayloadBoundary : for x being set holds x = x ;"
    })
    {
        Some(Task258B4Profile::QuantifierBoundUse)
    } else if owner.is_some_and(|owner| {
        owner.source_range == range(input.source_id, 0, 165)
            && owner.spelling == "FormulaConnectiveGroupingPayloadBoundary"
    }) || statement.is_some_and(|statement| {
        statement.spelling
            == "theorem FormulaConnectiveGroupingPayloadBoundary : for x being set holds ( ( 0 = 0 & ... & 0 = 3 ) or ( 0 = 0 or ... or 0 = 3 ) ) iff ( ( 0 = 0 & 0 = 0 ) or ( 0 = 0 or 0 = 0 ) ) ;"
    })
    {
        Some(Task258B4Profile::ConnectiveGrouping)
    } else if owner.is_some_and(|owner| {
        owner.source_range == range(input.source_id, 19, 137)
            && owner.spelling == "FormulaNestedQuantifierPayloadBoundary"
    }) || statement.is_some_and(|statement| {
        statement.spelling
            == "theorem FormulaNestedQuantifierPayloadBoundary : for x being set st x = x ex y being set st for r st r = y holds x = r ;"
    })
    {
        Some(Task258B4Profile::NestedQuantifier)
    } else {
        None
    }
}

#[allow(clippy::too_many_arguments)] // Rationale: each B4 profile must reauthenticate the complete matched lower-family tuple.
fn validate_task_258b4_dependencies(
    profile: Task258B4Profile,
    source_id: SourceId,
    module_id: &ModuleId,
    bindings: &BindingEnv,
    primary_terms: &SourcePrimaryTermHandoff,
    atomic_formulas: &SourceAtomicFormulaHandoff,
    composite_formulas: &SourceCompositeFormulaHandoff,
    formula_composition: &SourceFormulaCompositionHandoff,
    arena: &TypedArena,
) -> Result<(), SourceStatementError> {
    let (
        binding_contexts_len,
        bindings_len,
        binding_diagnostics_len,
        primary_terms_len,
        primary_references_len,
        numeric_requests_len,
        atomic_formulas_len,
        atomic_edges_len,
        atomic_requests_len,
        composition_edges_len,
        composition_bound_uses_len,
        arena_len,
    ) = match profile {
        Task258B4Profile::QuantifierBoundUse => (2, 1, 4, 2, 2, 0, 1, 2, 2, 1, 2, 26),
        Task258B4Profile::ConnectiveGrouping => (2, 1, 4, 16, 0, 16, 8, 16, 16, 8, 0, 124),
        Task258B4Profile::NestedQuantifier => (4, 4, 0, 6, 6, 0, 3, 6, 6, 3, 6, 66),
    };
    if bindings.source_id() != source_id
        || bindings.module_id() != module_id
        || bindings != composite_formulas.binding_env()
        || bindings.contexts().len() != binding_contexts_len
        || bindings.bindings().len() != bindings_len
        || bindings.diagnostics().len() != binding_diagnostics_len
        || primary_terms.source_id() != source_id
        || primary_terms.module_id() != module_id
        || primary_terms.terms().len() != primary_terms_len
        || primary_terms.references().len() != primary_references_len
        || primary_terms.numeric_type_requests().len() != numeric_requests_len
        || atomic_formulas.source_id() != source_id
        || atomic_formulas.module_id() != module_id
        || atomic_formulas.formulas().len() != atomic_formulas_len
        || !atomic_formulas.wrappers().is_empty()
        || !atomic_formulas.predicate_segments().is_empty()
        || !atomic_formulas.predicate_heads().is_empty()
        || !atomic_formulas.candidates().is_empty()
        || !atomic_formulas.type_sites().is_empty()
        || !atomic_formulas.attributes().is_empty()
        || atomic_formulas.edges().len() != atomic_edges_len
        || atomic_formulas.requests().len() != atomic_requests_len
        || composite_formulas.source_id() != source_id
        || composite_formulas.module_id() != module_id
        || match profile {
            Task258B4Profile::QuantifierBoundUse => !composite_formulas.is_task_257b1_profile(),
            Task258B4Profile::ConnectiveGrouping => !composite_formulas.is_task_257b2_profile(),
            Task258B4Profile::NestedQuantifier => !composite_formulas.is_task_257b3_profile(),
        }
        || formula_composition.source_id() != source_id
        || formula_composition.module_id() != module_id
        || formula_composition.atomic_edges().len() != composition_edges_len
        || formula_composition.bound_uses().len() != composition_bound_uses_len
        || arena.len() != arena_len
        || arena.root().is_some()
    {
        return Err(SourceStatementError::DependencyMismatch);
    }
    primary_terms
        .validate_installation(source_id, module_id, arena)
        .map_err(|_| SourceStatementError::DependencyMismatch)?;
    atomic_formulas
        .validate_installation(source_id, module_id, primary_terms, None, None, None, arena)
        .map_err(|_| SourceStatementError::DependencyMismatch)?;
    composite_formulas
        .validate_installation(source_id, module_id, arena)
        .map_err(|_| SourceStatementError::DependencyMismatch)?;
    formula_composition
        .validate_installation(
            source_id,
            module_id,
            primary_terms,
            atomic_formulas,
            composite_formulas,
            arena,
        )
        .map_err(|_| SourceStatementError::DependencyMismatch)?;
    if !validate_task_258b4_primary_atomic_sites(profile, source_id, primary_terms, atomic_formulas)
        || !validate_task_258b4_arena_partition(profile, source_id, arena)
    {
        return Err(SourceStatementError::DependencyMismatch);
    }
    let formula = composite_formulas
        .formulas()
        .get(SourceCompositeFormulaId::new(0))
        .ok_or(SourceStatementError::DependencyMismatch)?;
    let root = composite_formulas
        .roots()
        .get(crate::source_composite_formula::SourceFormulaRootId::new(0))
        .ok_or(SourceStatementError::DependencyMismatch)?;
    let binder = composite_formulas
        .binders()
        .get(crate::source_composite_formula::SourceQuantifierBinderId::new(0))
        .ok_or(SourceStatementError::DependencyMismatch)?;
    let type_site = composite_formulas
        .type_sites()
        .get(crate::source_composite_formula::SourceBinderTypeSiteId::new(0))
        .ok_or(SourceStatementError::DependencyMismatch)?;
    let (
        formula_site,
        formula_range,
        binder_segment_site,
        binder_identifier_site,
        type_site_id,
        type_head_site,
    ) = match profile {
        Task258B4Profile::QuantifierBoundUse => (20, (50, 77), 14, 4, 13, 12),
        Task258B4Profile::ConnectiveGrouping => (118, (50, 164), 58, 4, 57, 56),
        Task258B4Profile::NestedQuantifier => (60, (67, 136), 38, 9, 37, 36),
    };
    let (formula_context, formula_spelling, binder_range, identifier_range, type_range) =
        match profile {
            Task258B4Profile::QuantifierBoundUse | Task258B4Profile::ConnectiveGrouping => {
                (0, "for holds", (54, 65), (54, 55), (62, 65))
            }
            Task258B4Profile::NestedQuantifier => (0, "for st", (71, 82), (71, 72), (79, 82)),
        };
    if formula.site() != &TypedSiteRef::Node(crate::typed_ast::TypedNodeId::new(formula_site))
        || formula.source_range() != range(source_id, formula_range.0, formula_range.1)
        || formula.source_ordinal() != 0
        || formula.context() != BindingContextId::new(formula_context)
        || formula.recovery() != SourceCompositeFormulaRecovery::Normal
        || formula.spelling() != formula_spelling
        || formula.kind() != SourceCompositeFormulaKind::Universal
        || root.formula() != SourceCompositeFormulaId::new(0)
        || root.ordinal() != 0
        || root.ownership() != SourceFormulaRootOwnership::UnassignedStatement
        || binder.segment_site()
            != &TypedSiteRef::Node(crate::typed_ast::TypedNodeId::new(binder_segment_site))
        || binder.segment_range() != range(source_id, binder_range.0, binder_range.1)
        || binder.identifier_site()
            != &TypedSiteRef::Node(crate::typed_ast::TypedNodeId::new(binder_identifier_site))
        || binder.identifier_range() != range(source_id, identifier_range.0, identifier_range.1)
        || type_site.site() != &TypedSiteRef::Node(crate::typed_ast::TypedNodeId::new(type_site_id))
        || type_site.source_range() != range(source_id, type_range.0, type_range.1)
        || type_site.head_site()
            != &TypedSiteRef::Node(crate::typed_ast::TypedNodeId::new(type_head_site))
        || type_site.head_range() != range(source_id, type_range.0, type_range.1)
    {
        return Err(SourceStatementError::DependencyMismatch);
    }
    Ok(())
}

fn validate_task_258b4_arena_partition(
    profile: Task258B4Profile,
    source_id: SourceId,
    arena: &TypedArena,
) -> bool {
    let (lower_owned, theorem_site): (&[usize], usize) = match profile {
        Task258B4Profile::QuantifierBoundUse => (&[4, 12, 13, 14, 15, 17, 19, 20], 22),
        Task258B4Profile::ConnectiveGrouping => (
            &[
                4, 56, 57, 58, 59, 61, 63, 64, 66, 68, 69, 71, 72, 74, 76, 77, 79, 81, 82, 84, 85,
                87, 88, 90, 92, 93, 95, 97, 98, 100, 101, 103, 105, 106, 108, 110, 111, 113, 114,
                116, 117, 118,
            ],
            120,
        ),
        Task258B4Profile::NestedQuantifier => (
            &[
                9, 17, 22, 32, 33, 36, 37, 38, 39, 41, 43, 44, 45, 46, 47, 48, 50, 52, 53, 55, 57,
                58, 59, 60,
            ],
            62,
        ),
    };
    const TASK258B4C_ARENA_RANGES: [(usize, usize); 66] = [
        (0, 7),
        (8, 9),
        (10, 13),
        (14, 17),
        (17, 18),
        (19, 26),
        (27, 65),
        (65, 66),
        (67, 70),
        (71, 72),
        (73, 78),
        (79, 82),
        (83, 85),
        (86, 87),
        (88, 89),
        (90, 91),
        (92, 94),
        (95, 96),
        (97, 102),
        (103, 106),
        (107, 109),
        (110, 113),
        (114, 115),
        (116, 118),
        (119, 120),
        (121, 122),
        (123, 124),
        (125, 130),
        (131, 132),
        (133, 134),
        (135, 136),
        (136, 137),
        (14, 17),
        (14, 17),
        (8, 17),
        (0, 18),
        (79, 82),
        (79, 82),
        (71, 82),
        (86, 87),
        (86, 87),
        (90, 91),
        (90, 91),
        (86, 91),
        (103, 106),
        (103, 106),
        (95, 106),
        (114, 115),
        (119, 120),
        (119, 120),
        (123, 124),
        (123, 124),
        (119, 124),
        (131, 132),
        (131, 132),
        (135, 136),
        (135, 136),
        (131, 136),
        (110, 136),
        (92, 136),
        (67, 136),
        (67, 136),
        (19, 137),
        (0, 137),
        (0, 137),
        (0, 137),
    ];
    arena.iter().all(|(id, node)| {
        let id = id.index();
        let exact_profile = profile != Task258B4Profile::NestedQuantifier
            || (node.anchor
                == SourceAnchor::Range(range(
                    source_id,
                    TASK258B4C_ARENA_RANGES[id].0,
                    TASK258B4C_ARENA_RANGES[id].1,
                ))
                && node.recovery == NodeRecoveryState::Normal);
        exact_profile
            && if lower_owned.contains(&id) {
                node.kind.as_str() != "source.formula.composition.unowned"
                    && !node.kind.as_str().starts_with("source.statement.")
            } else if id == theorem_site {
                node.kind.as_str() == "source.statement.theorem"
            } else {
                node.kind.as_str() == "source.formula.composition.unowned"
            }
    })
}

fn validate_task_258b4_primary_atomic_sites(
    profile: Task258B4Profile,
    source_id: SourceId,
    primary_terms: &SourcePrimaryTermHandoff,
    atomic_formulas: &SourceAtomicFormulaHandoff,
) -> bool {
    struct SiteProfile {
        term_sites: &'static [usize],
        term_ranges: &'static [(usize, usize)],
        atomic_sites: &'static [usize],
        atomic_ranges: &'static [(usize, usize)],
    }
    let sites = match profile {
        Task258B4Profile::QuantifierBoundUse => SiteProfile {
            term_sites: &[15, 17],
            term_ranges: &[(72, 73), (76, 77)],
            atomic_sites: &[19],
            atomic_ranges: &[(72, 77)],
        },
        Task258B4Profile::ConnectiveGrouping => SiteProfile {
            term_sites: &[
                59, 61, 64, 66, 72, 74, 77, 79, 88, 90, 93, 95, 101, 103, 106, 108,
            ],
            term_ranges: &[
                (74, 75),
                (78, 79),
                (88, 89),
                (92, 93),
                (99, 100),
                (103, 104),
                (115, 116),
                (119, 120),
                (129, 130),
                (133, 134),
                (137, 138),
                (141, 142),
                (148, 149),
                (152, 153),
                (157, 158),
                (161, 162),
            ],
            atomic_sites: &[63, 68, 76, 81, 92, 97, 105, 110],
            atomic_ranges: &[
                (74, 79),
                (88, 93),
                (99, 104),
                (115, 120),
                (129, 134),
                (137, 142),
                (148, 153),
                (157, 162),
            ],
        },
        Task258B4Profile::NestedQuantifier => SiteProfile {
            term_sites: &[39, 41, 48, 50, 53, 55],
            term_ranges: &[
                (86, 87),
                (90, 91),
                (119, 120),
                (123, 124),
                (131, 132),
                (135, 136),
            ],
            atomic_sites: &[43, 52, 57],
            atomic_ranges: &[(86, 91), (119, 124), (131, 136)],
        },
    };
    primary_terms
        .terms()
        .iter()
        .zip(sites.term_sites.iter().zip(sites.term_ranges))
        .all(|((_, term), (site, (start, end)))| {
            term.site() == &TypedSiteRef::Node(crate::typed_ast::TypedNodeId::new(*site))
                && term.source_range() == range(source_id, *start, *end)
        })
        && atomic_formulas
            .formulas()
            .iter()
            .zip(sites.atomic_sites.iter().zip(sites.atomic_ranges))
            .all(|((_, formula), (site, (start, end)))| {
                formula.site() == &TypedSiteRef::Node(crate::typed_ast::TypedNodeId::new(*site))
                    && formula.source_range() == range(source_id, *start, *end)
            })
}

#[allow(clippy::too_many_arguments)] // Rationale: each frozen one-row profile is validated as one indivisible publication boundary.
fn validate_task_258b4_rows(
    profile: Task258B4Profile,
    source_id: SourceId,
    module_id: &ModuleId,
    owners: &SourceTheoremOwnerTable,
    checked_owner: &CheckedStatementOwner,
    authenticated_contribution: SourceContributionId,
    statements: &SourceStatementTable,
    contexts: &SourceStatementContextTable,
    input_facts: &SourceStatementInputFactTable,
    candidate_facts: &SourceStatementCandidateFactTable,
    composite_formulas: &SourceCompositeFormulaHandoff,
    arena: &TypedArena,
) -> Result<(), SourceStatementError> {
    if owners.len() != 1
        || statements.len() != 1
        || contexts.len() != 1
        || !input_facts.is_empty()
        || candidate_facts.len() != 1
    {
        return Err(SourceStatementError::InvalidAggregate);
    }
    let owner_id = SourceTheoremOwnerId::new(0);
    let owner = owners
        .get(owner_id)
        .ok_or(SourceStatementError::InvalidOwner { owner: owner_id })?;
    let (
        owner_start,
        owner_end,
        owner_site,
        owner_spelling,
        statement_spelling,
        composite_site,
        composite_start,
        composite_end,
        structural_path,
        visible_binding,
    ) = match profile {
        Task258B4Profile::QuantifierBoundUse => (
            0,
            78,
            22,
            "FormulaQuantifierBoundUsePayloadBoundary",
            "theorem FormulaQuantifierBoundUsePayloadBoundary : for x being set holds x = x ;",
            20,
            50,
            77,
            &[2, 0][..],
            None,
        ),
        Task258B4Profile::ConnectiveGrouping => (
            0,
            165,
            120,
            "FormulaConnectiveGroupingPayloadBoundary",
            "theorem FormulaConnectiveGroupingPayloadBoundary : for x being set holds ( ( 0 = 0 & ... & 0 = 3 ) or ( 0 = 0 or ... or 0 = 3 ) ) iff ( ( 0 = 0 & 0 = 0 ) or ( 0 = 0 or 0 = 0 ) ) ;",
            118,
            50,
            164,
            &[2, 0][..],
            None,
        ),
        Task258B4Profile::NestedQuantifier => (
            19,
            137,
            62,
            "FormulaNestedQuantifierPayloadBoundary",
            "theorem FormulaNestedQuantifierPayloadBoundary : for x being set st x = x ex y being set st for r st r = y holds x = r ;",
            60,
            67,
            136,
            &[2, 1][..],
            Some(BindingId::new(0)),
        ),
    };
    let owner_range = range(source_id, owner_start, owner_end);
    if owner.symbol != *checked_owner.symbol()
        || owner.contribution != authenticated_contribution
        || owner.contribution.index() != 0
        || owner.site != TypedSiteRef::Node(crate::typed_ast::TypedNodeId::new(owner_site))
        || owner.source_range != owner_range
        || owner.source_range != checked_owner.source_range()
        || owner.spelling != owner_spelling
        || owner.role != SourceTheoremRole::Theorem
        || owner.status != SourceTheoremStatus::Unmodified
        || owner.recovery != SourceStatementRecovery::Normal
        || checked_owner.origin().source_id() != source_id
        || checked_owner.origin().module_id() != module_id
        || checked_owner.origin().anchor() != &SourceAnchor::Range(owner_range)
        || checked_owner.origin().structural_path() != structural_path
        || checked_owner.origin().import_edge().is_some()
        || checked_owner.origin().is_recovered()
        || checked_owner.visibility() != Visibility::Public
        || checked_owner.export_status() != ExportStatus::Exported
        || !validate_theorem_site(&owner.site, owner_range, arena)
    {
        return Err(SourceStatementError::InvalidOwner { owner: owner_id });
    }

    let statement_id = SourceStatementId::new(0);
    let statement = statements
        .get(statement_id)
        .ok_or(SourceStatementError::InvalidStatement {
            statement: statement_id,
        })?;
    if statement.owner != owner_id
        || statement.context != SourceStatementContextId::new(0)
        || statement.formula
            != SourceStatementFormulaTarget::Composite(SourceCompositeFormulaId::new(0))
        || statement.site != owner.site
        || statement.source_range != owner_range
        || statement.source_ordinal != 0
        || statement.spelling != statement_spelling
        || statement.kind != SourceStatementKind::TheoremProposition
        || statement.recovery != SourceStatementRecovery::Normal
    {
        return Err(SourceStatementError::InvalidStatement {
            statement: statement_id,
        });
    }

    let context_id = SourceStatementContextId::new(0);
    let context = contexts
        .get(context_id)
        .ok_or(SourceStatementError::InvalidContext {
            context: context_id,
        })?;
    if context.statement != statement_id
        || context.binding_context != BindingContextId::new(0)
        || context.source_range != owner_range
        || match visible_binding {
            Some(binding) => context.visible_bindings.as_slice() != [binding],
            None => !context.visible_bindings.is_empty(),
        }
    {
        return Err(SourceStatementError::InvalidContext {
            context: context_id,
        });
    }

    let candidate_id = SourceStatementCandidateFactId::new(0);
    let candidate = candidate_facts
        .get(candidate_id)
        .ok_or(SourceStatementError::InvalidCandidateFact { fact: candidate_id })?;
    if candidate.statement != statement_id
        || candidate.context != context_id
        || candidate.ordinal != 0
        || candidate.kind != SourceStatementCandidateFactKind::UnverifiedProposition
        || candidate.formula != statement.formula
    {
        return Err(SourceStatementError::InvalidCandidateFact { fact: candidate_id });
    }

    let composite = composite_formulas
        .formulas()
        .get(SourceCompositeFormulaId::new(0))
        .ok_or(SourceStatementError::DependencyMismatch)?;
    if composite.site().node() != crate::typed_ast::TypedNodeId::new(composite_site)
        || composite.source_range() != range(source_id, composite_start, composite_end)
        || arena.iter().any(|(id, node)| {
            id != crate::typed_ast::TypedNodeId::new(owner_site)
                && (node.kind.as_str().starts_with("source.statement.")
                    || node.kind.as_str().contains("proof")
                    || node.kind.as_str().contains("justification"))
        })
    {
        return Err(SourceStatementError::InvalidStatement {
            statement: statement_id,
        });
    }
    Ok(())
}

fn validate_aggregate_lengths(
    profile: StatementProfile,
    owners: usize,
    statements: usize,
    contexts: usize,
    input_facts: usize,
    candidate_facts: usize,
) -> Result<(), SourceStatementError> {
    let expected = match profile {
        #[cfg(test)]
        StatementProfile::Task258A => (1, 1, 1, 1, 1),
        #[cfg(test)]
        StatementProfile::Task258B1 => (1, 4, 4, 4, 4),
        #[cfg(test)]
        StatementProfile::Task258B5A => (1, 5, 5, 5, 5),
        StatementProfile::Task258B5B => (1, 2, 2, 2, 2),
        #[cfg(test)]
        StatementProfile::Task258B2 => (1, 3, 3, 3, 3),
        #[cfg(test)]
        StatementProfile::Task258B3 => (1, 2, 2, 2, 2),
        #[cfg(test)]
        StatementProfile::Task258B3N => (1, 2, 2, 2, 2),
        #[cfg(test)]
        StatementProfile::Task258B3M1 => (1, 2, 2, 2, 2),
        #[cfg(test)]
        StatementProfile::Task258B3M2A => (1, 2, 2, 2, 2),
        #[cfg(test)]
        StatementProfile::Task258B3M2B1 => (1, 2, 2, 2, 2),
        #[cfg(test)]
        StatementProfile::Task258B3M2B2A => (1, 2, 2, 2, 2),
        #[cfg(test)]
        StatementProfile::Task258B3M2B2B1A => (1, 2, 2, 2, 2),
        #[cfg(test)]
        StatementProfile::Task258B3M2B2B1B1 => (1, 2, 2, 2, 2),
        StatementProfile::Task258B3M2B2B2A => (1, 2, 2, 2, 2),
        StatementProfile::Task258B3M2B2B2B => (1, 2, 2, 2, 2),
        #[cfg(test)]
        StatementProfile::Task258B3M2B2B2C => (1, 2, 2, 2, 2),
        #[cfg(test)]
        StatementProfile::Task258B3M2B2B3A => (1, 2, 2, 2, 2),
        #[cfg(test)]
        StatementProfile::Task258B3M2B2B3B => (1, 2, 2, 2, 2),
        #[cfg(test)]
        StatementProfile::Task258B3M2B2B3C => (1, 2, 2, 2, 2),
        #[cfg(test)]
        StatementProfile::Task258B3M2B2B3D => (1, 2, 2, 2, 2),
        #[cfg(test)]
        StatementProfile::Task258B3M2B2B3E => (1, 2, 2, 2, 2),
    };
    if (owners, statements, contexts, input_facts, candidate_facts) != expected {
        return Err(SourceStatementError::InvalidAggregate);
    }
    Ok(())
}

fn validate_resolver_owner(
    input: &SourceStatementHandoffInput,
    symbols: &SymbolEnv,
) -> Result<CheckedStatementOwner, SourceStatementError> {
    let id = SourceTheoremOwnerId::new(0);
    let owner = &input.owners[0];
    let invalid = || SourceStatementError::InvalidOwner { owner: id };
    let checked = CheckedStatementOwner::validate_exact_local_theorem(
        symbols,
        owner.symbol.clone(),
        input.source_id,
        &input.module_id,
    )
    .map_err(|_| invalid())?;
    let symbol = symbols.symbols().get(&owner.symbol).ok_or_else(invalid)?;
    let definition = symbols
        .definitions()
        .by_symbol(&owner.symbol)
        .ok_or_else(invalid)?;
    let contribution = symbols
        .contributions()
        .get(owner.contribution)
        .ok_or_else(invalid)?;
    let labels = symbols.labels().by_contribution(owner.contribution);
    let imported_citation_owner =
        owner.spelling == "FormulaStatementImportedPublicTheoremCitationSmoke";
    let exact_owner_labels = if imported_citation_owner {
        labels.is_empty() && contribution.effects().labels().is_empty()
    } else {
        labels.as_slice().first().is_some_and(|label| {
            labels.len() == 1
                && contribution
                    .effects()
                    .labels()
                    .contains(label.origin_path())
                && label.kind() == LabelKind::Theorem
                && label.visibility() == Visibility::Public
                && label.export_status() == ExportStatus::Exported
                && label.namespace() == symbol.namespace()
                && label.primary_spelling() == owner.spelling
                && label.origin() == symbol.origin()
                && label.contribution() == owner.contribution
                && label.recovery() == RecoveryState::Normal
        })
    };
    let expected_contribution_range = match owner.spelling.as_str() {
        "FormulaQuantifierBoundUsePayloadBoundary" => range(input.source_id, 0, 78),
        "FormulaConnectiveGroupingPayloadBoundary" => range(input.source_id, 0, 165),
        "FormulaStatementApplicationWitnessSmoke"
        | "FormulaStatementParenthesizedApplicationWitnessSmoke"
        | "FormulaStatementStructureConstructorWitnessSmoke"
        | "FormulaStatementStructureSelectorWitnessSmoke"
        | "FormulaStatementImportedPublicTheoremCitationSmoke" => range(input.source_id, 29, 47),
        #[cfg(test)]
        "FormulaStatementStructureUpdateWitnessSmoke" => range(input.source_id, 29, 47),
        _ => range(input.source_id, 0, 18),
    };
    if !exact_owner_labels
        || owner.contribution != symbol.contribution()
        || owner.contribution != definition.contribution()
        || symbol.primary_spelling() != owner.spelling
        || symbol.origin() != checked.origin()
        || contribution.module() != &input.module_id
        || contribution.kind()
            != &(ContributionKind::LocalSource {
                source_id: input.source_id,
            })
        || contribution.anchor() != &SourceAnchor::Range(expected_contribution_range)
        || !contribution.effects().symbols().contains(&owner.symbol)
        || !contribution
            .effects()
            .definitions()
            .contains(&definition.id())
        || !contribution.effects().imports().is_empty()
        || symbol.namespace().as_str() != input.module_id.path().as_str()
    {
        return Err(invalid());
    }
    Ok(checked)
}

fn validate_owner_rows(
    profile: StatementProfile,
    source_id: SourceId,
    module_id: &ModuleId,
    owners: &SourceTheoremOwnerTable,
    checked_owner: &CheckedStatementOwner,
    authenticated_contribution: SourceContributionId,
    arena: &TypedArena,
) -> Result<(), SourceStatementError> {
    let id = SourceTheoremOwnerId::new(0);
    let Some(owner) = owners.get(id) else {
        return Err(SourceStatementError::InvalidAggregate);
    };
    let (expected_range, expected_spelling) = match profile {
        #[cfg(test)]
        StatementProfile::Task258A => (
            range(source_id, 19, 80),
            "FormulaStatementReservedVariableEqualitySmoke",
        ),
        #[cfg(test)]
        StatementProfile::Task258B1 => (
            range(source_id, 19, 138),
            "FormulaStatementNestedContextSmoke",
        ),
        #[cfg(test)]
        StatementProfile::Task258B5A => (
            range(source_id, 19, 184),
            "FormulaStatementAncestorLabelDescendantCitationSmoke",
        ),
        StatementProfile::Task258B5B => (
            range(source_id, 48, 145),
            "FormulaStatementImportedPublicTheoremCitationSmoke",
        ),
        #[cfg(test)]
        StatementProfile::Task258B2 => (
            range(source_id, 19, 112),
            "FormulaStatementSingleAssumptionSmoke",
        ),
        #[cfg(test)]
        StatementProfile::Task258B3 => (
            range(source_id, 19, 103),
            "FormulaStatementSingleWitnessSmoke",
        ),
        #[cfg(test)]
        StatementProfile::Task258B3N => (
            range(source_id, 19, 106),
            "FormulaStatementNamedWitnessSmoke",
        ),
        #[cfg(test)]
        StatementProfile::Task258B3M1 => (
            range(source_id, 19, 112),
            "FormulaStatementMultipleWitnessSmoke",
        ),
        #[cfg(test)]
        StatementProfile::Task258B3M2A => (
            range(source_id, 19, 106),
            "FormulaStatementNumeralWitnessSmoke",
        ),
        #[cfg(test)]
        StatementProfile::Task258B3M2B1 => (
            range(source_id, 19, 112),
            "FormulaStatementParenthesizedWitnessSmoke",
        ),
        #[cfg(test)]
        StatementProfile::Task258B3M2B2A => (
            range(source_id, 19, 120),
            "FormulaStatementNestedParenthesizedWitnessSmoke",
        ),
        #[cfg(test)]
        StatementProfile::Task258B3M2B2B1A => (
            range(source_id, 48, 142),
            "FormulaStatementApplicationWitnessSmoke",
        ),
        #[cfg(test)]
        StatementProfile::Task258B3M2B2B1B1 => (
            range(source_id, 48, 157),
            "FormulaStatementParenthesizedApplicationWitnessSmoke",
        ),
        StatementProfile::Task258B3M2B2B2A => (
            range(source_id, 48, 171),
            "FormulaStatementStructureConstructorWitnessSmoke",
        ),
        StatementProfile::Task258B3M2B2B2B => (
            range(source_id, 48, 170),
            "FormulaStatementStructureSelectorWitnessSmoke",
        ),
        #[cfg(test)]
        StatementProfile::Task258B3M2B2B2C => (
            range(source_id, 48, 180),
            "FormulaStatementStructureUpdateWitnessSmoke",
        ),
        #[cfg(test)]
        StatementProfile::Task258B3M2B2B3A => (
            range(source_id, 19, 116),
            "FormulaStatementSetEnumerationWitnessSmoke",
        ),
        #[cfg(test)]
        StatementProfile::Task258B3M2B2B3B => (
            range(source_id, 19, 117),
            "FormulaStatementEmptySetEnumerationWitnessSmoke",
        ),
        #[cfg(test)]
        StatementProfile::Task258B3M2B2B3C => (
            range(source_id, 19, 109),
            "FormulaStatementChoiceWitnessSmoke",
        ),
        #[cfg(test)]
        StatementProfile::Task258B3M2B2B3D => {
            (range(source_id, 19, 108), "FormulaStatementQuaWitnessSmoke")
        }
        #[cfg(test)]
        StatementProfile::Task258B3M2B2B3E => (
            range(source_id, 19, 138),
            "FormulaStatementComprehensionWitnessSmoke",
        ),
    };
    #[cfg(test)]
    if matches!(
        profile,
        StatementProfile::Task258B5A | StatementProfile::Task258B2
    ) && (owner.contribution.index() != 0 || checked_owner.origin().structural_path() != [2, 1])
    {
        return Err(SourceStatementError::InvalidOwner { owner: id });
    }
    #[cfg(not(test))]
    let proof_owner = true;
    #[cfg(test)]
    let proof_owner = !matches!(
        profile,
        StatementProfile::Task258A
            | StatementProfile::Task258B1
            | StatementProfile::Task258B5A
            | StatementProfile::Task258B2
    );
    if owner.symbol != *checked_owner.symbol()
        || owner.contribution != authenticated_contribution
        || owner.source_range != expected_range
        || owner.source_range != checked_owner.source_range()
        || checked_owner.origin().source_id() != source_id
        || checked_owner.origin().module_id() != module_id
        || checked_owner.origin().anchor() != &SourceAnchor::Range(owner.source_range)
        || checked_owner.origin().import_edge().is_some()
        || checked_owner.origin().is_recovered()
        || checked_owner.visibility() != Visibility::Public
        || checked_owner.export_status() != ExportStatus::Exported
        || owner.spelling != expected_spelling
        || owner.role != SourceTheoremRole::Theorem
        || owner.status != SourceTheoremStatus::Unmodified
        || owner.recovery != SourceStatementRecovery::Normal
        || !validate_theorem_site(&owner.site, owner.source_range, arena)
        || (proof_owner
            && (owner.contribution.index() != 0
                || checked_owner.origin().structural_path() != [2, 1]))
    {
        return Err(SourceStatementError::InvalidOwner { owner: id });
    }
    Ok(())
}

fn validate_statement_rows(
    profile: StatementProfile,
    source_id: SourceId,
    statements: &SourceStatementTable,
    owners: &SourceTheoremOwnerTable,
    contexts: &SourceStatementContextTable,
    atomic_formulas: &SourceAtomicFormulaHandoff,
    arena: &TypedArena,
) -> Result<(), SourceStatementError> {
    let expected: &[(usize, usize, usize, SourceStatementKind, &str, &str)] = match profile {
        #[cfg(test)]
        StatementProfile::Task258A => &[(
            0,
            19,
            80,
            SourceStatementKind::TheoremProposition,
            "source.statement.theorem",
            "theorem FormulaStatementReservedVariableEqualitySmoke : x = x ;",
        )],
        #[cfg(test)]
        StatementProfile::Task258B1 => &[
            (
                0,
                19,
                138,
                SourceStatementKind::TheoremProposition,
                "source.statement.theorem",
                "theorem FormulaStatementNestedContextSmoke : x = x proof A : x = x proof thus x = x ; end ; thus x = x by A ; end ;",
            ),
            (
                1,
                77,
                114,
                SourceStatementKind::ProofStepProposition,
                "source.statement.proof-step",
                "A : x = x proof thus x = x ; end ;",
            ),
            (
                2,
                96,
                107,
                SourceStatementKind::Conclusion,
                "source.statement.conclusion",
                "thus x = x ;",
            ),
            (
                3,
                117,
                133,
                SourceStatementKind::Conclusion,
                "source.statement.conclusion",
                "thus x = x by A ;",
            ),
        ],
        StatementProfile::Task258B5B => &[
            (
                0,
                48,
                145,
                SourceStatementKind::TheoremProposition,
                "source.statement.theorem",
                "theorem FormulaStatementImportedPublicTheoremCitationSmoke : x = x proof thus x = x by Ref ; end ;",
            ),
            (
                1,
                122,
                140,
                SourceStatementKind::Conclusion,
                "source.statement.conclusion",
                "thus x = x by Ref ;",
            ),
        ],
        #[cfg(test)]
        StatementProfile::Task258B5A => &[
            (
                0,
                19,
                184,
                SourceStatementKind::TheoremProposition,
                "source.statement.theorem",
                "theorem FormulaStatementAncestorLabelDescendantCitationSmoke : x = x proof A : x = x proof thus x = x ; end ; thus x = x proof thus x = x by A ; end ; end ;",
            ),
            (
                1,
                95,
                132,
                SourceStatementKind::ProofStepProposition,
                "source.statement.proof-step",
                "A : x = x proof thus x = x ; end ;",
            ),
            (
                2,
                114,
                125,
                SourceStatementKind::Conclusion,
                "source.statement.conclusion",
                "thus x = x ;",
            ),
            (
                3,
                135,
                179,
                SourceStatementKind::Conclusion,
                "source.statement.conclusion",
                "thus x = x proof thus x = x by A ; end ;",
            ),
            (
                4,
                156,
                172,
                SourceStatementKind::Conclusion,
                "source.statement.conclusion",
                "thus x = x by A ;",
            ),
        ],
        #[cfg(test)]
        StatementProfile::Task258B3 => &[
            (
                0,
                19,
                103,
                SourceStatementKind::TheoremProposition,
                "source.statement.theorem",
                "theorem FormulaStatementSingleWitnessSmoke : x = x proof take x ; thus x = x ; end ;",
            ),
            (
                1,
                87,
                98,
                SourceStatementKind::Conclusion,
                "source.statement.conclusion",
                "thus x = x ;",
            ),
        ],
        #[cfg(test)]
        StatementProfile::Task258B3N => &[
            (
                0,
                19,
                106,
                SourceStatementKind::TheoremProposition,
                "source.statement.theorem",
                "theorem FormulaStatementNamedWitnessSmoke : x = x proof take y = x ; thus x = x ; end ;",
            ),
            (
                1,
                90,
                101,
                SourceStatementKind::Conclusion,
                "source.statement.conclusion",
                "thus x = x ;",
            ),
        ],
        #[cfg(test)]
        StatementProfile::Task258B3M1 => &[
            (
                0,
                19,
                112,
                SourceStatementKind::TheoremProposition,
                "source.statement.theorem",
                "theorem FormulaStatementMultipleWitnessSmoke : x = x proof take y = x , x ; thus x = x ; end ;",
            ),
            (
                1,
                96,
                107,
                SourceStatementKind::Conclusion,
                "source.statement.conclusion",
                "thus x = x ;",
            ),
        ],
        #[cfg(test)]
        StatementProfile::Task258B3M2A => &[
            (
                0,
                19,
                106,
                SourceStatementKind::TheoremProposition,
                "source.statement.theorem",
                "theorem FormulaStatementNumeralWitnessSmoke : x = x proof take 101 ; thus x = x ; end ;",
            ),
            (
                1,
                90,
                101,
                SourceStatementKind::Conclusion,
                "source.statement.conclusion",
                "thus x = x ;",
            ),
        ],
        #[cfg(test)]
        StatementProfile::Task258B3M2B1 => &[
            (
                0,
                19,
                112,
                SourceStatementKind::TheoremProposition,
                "source.statement.theorem",
                "theorem FormulaStatementParenthesizedWitnessSmoke : x = x proof take ( x ) ; thus x = x ; end ;",
            ),
            (
                1,
                96,
                107,
                SourceStatementKind::Conclusion,
                "source.statement.conclusion",
                "thus x = x ;",
            ),
        ],
        #[cfg(test)]
        StatementProfile::Task258B3M2B2A => &[
            (
                0,
                19,
                120,
                SourceStatementKind::TheoremProposition,
                "source.statement.theorem",
                "theorem FormulaStatementNestedParenthesizedWitnessSmoke : x = x proof take ( ( x ) ) ; thus x = x ; end ;",
            ),
            (
                1,
                104,
                115,
                SourceStatementKind::Conclusion,
                "source.statement.conclusion",
                "thus x = x ;",
            ),
        ],
        #[cfg(test)]
        StatementProfile::Task258B3M2B2B1A => &[
            (
                0,
                48,
                142,
                SourceStatementKind::TheoremProposition,
                "source.statement.theorem",
                "theorem FormulaStatementApplicationWitnessSmoke : x = x proof take 1 ++ 2 ; thus x = x ; end ;",
            ),
            (
                1,
                126,
                137,
                SourceStatementKind::Conclusion,
                "source.statement.conclusion",
                "thus x = x ;",
            ),
        ],
        #[cfg(test)]
        StatementProfile::Task258B3M2B2B1B1 => &[
            (
                0,
                48,
                157,
                SourceStatementKind::TheoremProposition,
                "source.statement.theorem",
                "theorem FormulaStatementParenthesizedApplicationWitnessSmoke : x = x proof take ( 1 ++ 2 ) ; thus x = x ; end ;",
            ),
            (
                1,
                141,
                152,
                SourceStatementKind::Conclusion,
                "source.statement.conclusion",
                "thus x = x ;",
            ),
        ],
        StatementProfile::Task258B3M2B2B2A => &[
            (
                0,
                48,
                171,
                SourceStatementKind::TheoremProposition,
                "source.statement.theorem",
                "theorem FormulaStatementStructureConstructorWitnessSmoke : x = x proof take TypeCaseStruct ( x : 1 , y : 2 ) ; thus x = x ; end ;",
            ),
            (
                1,
                155,
                166,
                SourceStatementKind::Conclusion,
                "source.statement.conclusion",
                "thus x = x ;",
            ),
        ],
        StatementProfile::Task258B3M2B2B2B => &[
            (
                0,
                48,
                170,
                SourceStatementKind::TheoremProposition,
                "source.statement.theorem",
                "theorem FormulaStatementStructureSelectorWitnessSmoke : x = x proof take TypeCaseStruct ( x : 1 , y : 2 ) . x ; thus x = x ; end ;",
            ),
            (
                1,
                154,
                165,
                SourceStatementKind::Conclusion,
                "source.statement.conclusion",
                "thus x = x ;",
            ),
        ],
        #[cfg(test)]
        StatementProfile::Task258B3M2B2B2C => &[
            (
                0,
                48,
                180,
                SourceStatementKind::TheoremProposition,
                "source.statement.theorem",
                "theorem FormulaStatementStructureUpdateWitnessSmoke : x = x proof take TypeCaseStruct ( x : 1 , y : 2 ) with ( x := 3 ) ; thus x = x ; end ;",
            ),
            (
                1,
                164,
                175,
                SourceStatementKind::Conclusion,
                "source.statement.conclusion",
                "thus x = x ;",
            ),
        ],
        #[cfg(test)]
        StatementProfile::Task258B3M2B2B3A => &[
            (
                0,
                19,
                116,
                SourceStatementKind::TheoremProposition,
                "source.statement.theorem",
                "theorem FormulaStatementSetEnumerationWitnessSmoke : x = x proof take { 1 , 2 } ; thus x = x ; end ;",
            ),
            (
                1,
                100,
                111,
                SourceStatementKind::Conclusion,
                "source.statement.conclusion",
                "thus x = x ;",
            ),
        ],
        #[cfg(test)]
        StatementProfile::Task258B3M2B2B3B => &[
            (
                0,
                19,
                117,
                SourceStatementKind::TheoremProposition,
                "source.statement.theorem",
                "theorem FormulaStatementEmptySetEnumerationWitnessSmoke : x = x proof take { } ; thus x = x ; end ;",
            ),
            (
                1,
                101,
                112,
                SourceStatementKind::Conclusion,
                "source.statement.conclusion",
                "thus x = x ;",
            ),
        ],
        #[cfg(test)]
        StatementProfile::Task258B3M2B2B3C => &[
            (
                0,
                19,
                109,
                SourceStatementKind::TheoremProposition,
                "source.statement.theorem",
                "theorem FormulaStatementChoiceWitnessSmoke : x = x proof take the set ; thus x = x ; end ;",
            ),
            (
                1,
                93,
                104,
                SourceStatementKind::Conclusion,
                "source.statement.conclusion",
                "thus x = x ;",
            ),
        ],
        #[cfg(test)]
        StatementProfile::Task258B3M2B2B3D => &[
            (
                0,
                19,
                108,
                SourceStatementKind::TheoremProposition,
                "source.statement.theorem",
                "theorem FormulaStatementQuaWitnessSmoke : x = x proof take 4 qua set ; thus x = x ; end ;",
            ),
            (
                1,
                92,
                103,
                SourceStatementKind::Conclusion,
                "source.statement.conclusion",
                "thus x = x ;",
            ),
        ],
        #[cfg(test)]
        StatementProfile::Task258B3M2B2B3E => &[
            (
                0,
                19,
                138,
                SourceStatementKind::TheoremProposition,
                "source.statement.theorem",
                "theorem FormulaStatementComprehensionWitnessSmoke : x = x proof take { 3 where candidate255 is set } ; thus x = x ; end ;",
            ),
            (
                1,
                122,
                133,
                SourceStatementKind::Conclusion,
                "source.statement.conclusion",
                "thus x = x ;",
            ),
        ],
        #[cfg(test)]
        StatementProfile::Task258B2 => &[
            (
                0,
                19,
                112,
                SourceStatementKind::TheoremProposition,
                "source.statement.theorem",
                "theorem FormulaStatementSingleAssumptionSmoke : x = x proof assume x = x ; thus x = x ; end ;",
            ),
            (
                1,
                80,
                93,
                SourceStatementKind::Assumption,
                "source.statement.assumption",
                "assume x = x ;",
            ),
            (
                2,
                96,
                107,
                SourceStatementKind::Conclusion,
                "source.statement.conclusion",
                "thus x = x ;",
            ),
        ],
    };
    for (index, (context, start, end, kind, site_kind, spelling)) in
        expected.iter().copied().enumerate()
    {
        let id = SourceStatementId::new(index);
        let Some(statement) = statements.get(id) else {
            return Err(SourceStatementError::InvalidAggregate);
        };
        let formula_id = SourceAtomicFormulaId::new(index);
        let expected_formula = SourceStatementFormulaTarget::Atomic(formula_id);
        let Some(formula_site) = atomic_formulas
            .formulas()
            .get(formula_id)
            .map(|formula| formula.site().node())
        else {
            return Err(SourceStatementError::InvalidStatement { statement: id });
        };
        let expected_site = if index == 0 {
            owners
                .get(SourceTheoremOwnerId::new(0))
                .map(|owner| owner.site.clone())
        } else {
            Some(statement.site.clone())
        };
        if statement.owner != SourceTheoremOwnerId::new(0)
            || owners.get(statement.owner).is_none()
            || statement.context != SourceStatementContextId::new(index)
            || context != index
            || contexts.get(statement.context).is_none()
            || statement.formula != expected_formula
            || expected_site.as_ref() != Some(&statement.site)
            || statement.source_range != range(source_id, start, end)
            || statement.source_ordinal
                != match profile {
                    #[cfg(test)]
                    StatementProfile::Task258A
                    | StatementProfile::Task258B1
                    | StatementProfile::Task258B5A
                    | StatementProfile::Task258B2 => index,
                    StatementProfile::Task258B5B => index,
                    _ if index == 1 => 2,
                    _ => index,
                }
            || statement.spelling != spelling
            || statement.kind != kind
            || statement.recovery != SourceStatementRecovery::Normal
            || arena.node(statement.site.node()).is_none_or(|node| {
                node.anchor != SourceAnchor::Range(statement.source_range)
                    || node.kind.as_str() != site_kind
                    || node.recovery != NodeRecoveryState::Normal
                    || !valid_statement_formula_path(
                        profile,
                        arena,
                        statement.site.node(),
                        formula_site,
                    )
            })
        {
            return Err(SourceStatementError::InvalidStatement { statement: id });
        }
    }
    #[cfg(test)]
    if profile == StatementProfile::Task258A {
        let statement = statements
            .get(SourceStatementId::new(0))
            .expect("validated aggregate");
        let formula = atomic_formulas
            .formulas()
            .get(SourceAtomicFormulaId::new(0))
            .expect("validated aggregate");
        if arena.node(statement.site.node()).is_none_or(|node| {
            node.children.iter().any(|child| {
                subtree_contains_excluded_statement_owner(arena, *child, formula.site().node())
            })
        }) {
            return Err(SourceStatementError::InvalidStatement {
                statement: SourceStatementId::new(0),
            });
        }
        return Ok(());
    }
    let rows = match profile {
        #[cfg(test)]
        StatementProfile::Task258B1 => 4,
        #[cfg(test)]
        StatementProfile::Task258B5A => 5,
        StatementProfile::Task258B5B => 2,
        #[cfg(test)]
        StatementProfile::Task258B2 => 3,
        #[cfg(test)]
        StatementProfile::Task258B3 => 2,
        #[cfg(test)]
        StatementProfile::Task258B3N => 2,
        #[cfg(test)]
        StatementProfile::Task258B3M1 => 2,
        #[cfg(test)]
        StatementProfile::Task258B3M2A => 2,
        #[cfg(test)]
        StatementProfile::Task258B3M2B1 => 2,
        #[cfg(test)]
        StatementProfile::Task258B3M2B2A => 2,
        #[cfg(test)]
        StatementProfile::Task258B3M2B2B1A => 2,
        #[cfg(test)]
        StatementProfile::Task258B3M2B2B1B1 => 2,
        StatementProfile::Task258B3M2B2B2A => 2,
        StatementProfile::Task258B3M2B2B2B => 2,
        #[cfg(test)]
        StatementProfile::Task258B3M2B2B2C => 2,
        #[cfg(test)]
        StatementProfile::Task258B3M2B2B3A => 2,
        #[cfg(test)]
        StatementProfile::Task258B3M2B2B3B => 2,
        #[cfg(test)]
        StatementProfile::Task258B3M2B2B3C => 2,
        #[cfg(test)]
        StatementProfile::Task258B3M2B2B3D => 2,
        #[cfg(test)]
        StatementProfile::Task258B3M2B2B3E => 2,
        #[cfg(test)]
        StatementProfile::Task258A => unreachable!("Task258A returns above"),
    };
    let sites = (0..rows)
        .map(|index| {
            statements
                .get(SourceStatementId::new(index))
                .expect("validated statement row")
                .site
                .node()
        })
        .collect::<Vec<_>>();
    let formulas = (0..rows)
        .map(|index| {
            atomic_formulas
                .formulas()
                .get(SourceAtomicFormulaId::new(index))
                .expect("validated formula row")
                .site()
                .node()
        })
        .collect::<Vec<_>>();
    let statement_descendants: &[&[bool]] = match profile {
        #[cfg(test)]
        StatementProfile::Task258B1 => &[
            &[false, true, true, true],
            &[false, false, true, false],
            &[false, false, false, false],
            &[false, false, false, false],
        ],
        #[cfg(test)]
        StatementProfile::Task258B5A => &[
            &[false, true, true, true, true],
            &[false, false, true, false, false],
            &[false, false, false, false, false],
            &[false, false, false, false, true],
            &[false, false, false, false, false],
        ],
        StatementProfile::Task258B5B => &[&[false, true], &[false, false]],
        #[cfg(test)]
        StatementProfile::Task258B2 => &[
            &[false, true, true],
            &[false, false, false],
            &[false, false, false],
        ],
        #[cfg(test)]
        StatementProfile::Task258B3 => &[&[false, true], &[false, false]],
        #[cfg(test)]
        StatementProfile::Task258B3N => &[&[false, true], &[false, false]],
        #[cfg(test)]
        StatementProfile::Task258B3M1 => &[&[false, true], &[false, false]],
        #[cfg(test)]
        StatementProfile::Task258B3M2A => &[&[false, true], &[false, false]],
        #[cfg(test)]
        StatementProfile::Task258B3M2B1 => &[&[false, true], &[false, false]],
        #[cfg(test)]
        StatementProfile::Task258B3M2B2A => &[&[false, true], &[false, false]],
        #[cfg(test)]
        StatementProfile::Task258B3M2B2B1A => &[&[false, true], &[false, false]],
        #[cfg(test)]
        StatementProfile::Task258B3M2B2B1B1 => &[&[false, true], &[false, false]],
        StatementProfile::Task258B3M2B2B2A => &[&[false, true], &[false, false]],
        StatementProfile::Task258B3M2B2B2B => &[&[false, true], &[false, false]],
        #[cfg(test)]
        StatementProfile::Task258B3M2B2B2C => &[&[false, true], &[false, false]],
        #[cfg(test)]
        StatementProfile::Task258B3M2B2B3A => &[&[false, true], &[false, false]],
        #[cfg(test)]
        StatementProfile::Task258B3M2B2B3B => &[&[false, true], &[false, false]],
        #[cfg(test)]
        StatementProfile::Task258B3M2B2B3C => &[&[false, true], &[false, false]],
        #[cfg(test)]
        StatementProfile::Task258B3M2B2B3D => &[&[false, true], &[false, false]],
        #[cfg(test)]
        StatementProfile::Task258B3M2B2B3E => &[&[false, true], &[false, false]],
        #[cfg(test)]
        StatementProfile::Task258A => unreachable!("Task258A returns above"),
    };
    let formula_descendants: &[&[bool]] = match profile {
        #[cfg(test)]
        StatementProfile::Task258B1 => &[
            &[true, true, true, true],
            &[false, true, true, false],
            &[false, false, true, false],
            &[false, false, false, true],
        ],
        #[cfg(test)]
        StatementProfile::Task258B5A => &[
            &[true, true, true, true, true],
            &[false, true, true, false, false],
            &[false, false, true, false, false],
            &[false, false, false, true, true],
            &[false, false, false, false, true],
        ],
        StatementProfile::Task258B5B => &[&[true, true], &[false, true]],
        #[cfg(test)]
        StatementProfile::Task258B2 => &[
            &[true, true, true],
            &[false, true, false],
            &[false, false, true],
        ],
        #[cfg(test)]
        StatementProfile::Task258B3 => &[&[true, true], &[false, true]],
        #[cfg(test)]
        StatementProfile::Task258B3N => &[&[true, true], &[false, true]],
        #[cfg(test)]
        StatementProfile::Task258B3M1 => &[&[true, true], &[false, true]],
        #[cfg(test)]
        StatementProfile::Task258B3M2A => &[&[true, true], &[false, true]],
        #[cfg(test)]
        StatementProfile::Task258B3M2B1 => &[&[true, true], &[false, true]],
        #[cfg(test)]
        StatementProfile::Task258B3M2B2A => &[&[true, true], &[false, true]],
        #[cfg(test)]
        StatementProfile::Task258B3M2B2B1A => &[&[true, true], &[false, true]],
        #[cfg(test)]
        StatementProfile::Task258B3M2B2B1B1 => &[&[true, true], &[false, true]],
        StatementProfile::Task258B3M2B2B2A => &[&[true, true], &[false, true]],
        StatementProfile::Task258B3M2B2B2B => &[&[true, true], &[false, true]],
        #[cfg(test)]
        StatementProfile::Task258B3M2B2B2C => &[&[true, true], &[false, true]],
        #[cfg(test)]
        StatementProfile::Task258B3M2B2B3A => &[&[true, true], &[false, true]],
        #[cfg(test)]
        StatementProfile::Task258B3M2B2B3B => &[&[true, true], &[false, true]],
        #[cfg(test)]
        StatementProfile::Task258B3M2B2B3C => &[&[true, true], &[false, true]],
        #[cfg(test)]
        StatementProfile::Task258B3M2B2B3D => &[&[true, true], &[false, true]],
        #[cfg(test)]
        StatementProfile::Task258B3M2B2B3E => &[&[true, true], &[false, true]],
        #[cfg(test)]
        StatementProfile::Task258A => unreachable!("Task258A returns above"),
    };
    if sites
        .iter()
        .copied()
        .collect::<std::collections::BTreeSet<_>>()
        .len()
        != rows
        || formulas
            .iter()
            .copied()
            .collect::<std::collections::BTreeSet<_>>()
            .len()
            != rows
        || sites.iter().enumerate().any(|(statement, site)| {
            sites.iter().enumerate().any(|(candidate, target)| {
                is_descendant(arena, *site, *target) != statement_descendants[statement][candidate]
            }) || formulas.iter().enumerate().any(|(formula, target)| {
                is_descendant(arena, *site, *target) != formula_descendants[statement][formula]
            })
        })
        || formulas.iter().any(|formula| {
            sites
                .iter()
                .chain(formulas.iter())
                .any(|target| is_descendant(arena, *formula, *target))
        })
        || arena
            .iter()
            .filter(|(_, node)| node.kind.as_str().starts_with("source.statement."))
            .count()
            != rows
    {
        return Err(SourceStatementError::InvalidStatement {
            statement: SourceStatementId::new(0),
        });
    }
    Ok(())
}

fn validate_context_rows(
    profile: StatementProfile,
    source_id: SourceId,
    contexts: &SourceStatementContextTable,
    statements: &SourceStatementTable,
    bindings: &BindingEnv,
) -> Result<(), SourceStatementError> {
    let expected: &[(usize, usize, usize)] = match profile {
        #[cfg(test)]
        StatementProfile::Task258A => &[(0, 19, 80)],
        #[cfg(test)]
        StatementProfile::Task258B1 => &[(0, 19, 138), (1, 77, 114), (2, 96, 107), (1, 117, 133)],
        #[cfg(test)]
        StatementProfile::Task258B5A => &[
            (0, 19, 184),
            (1, 95, 132),
            (2, 114, 125),
            (1, 135, 179),
            (3, 156, 172),
        ],
        StatementProfile::Task258B5B => &[(0, 48, 145), (1, 122, 140)],
        #[cfg(test)]
        StatementProfile::Task258B2 => &[(0, 19, 112), (1, 80, 93), (1, 96, 107)],
        #[cfg(test)]
        StatementProfile::Task258B3 => &[(0, 19, 103), (1, 87, 98)],
        #[cfg(test)]
        StatementProfile::Task258B3N => &[(0, 19, 106), (1, 90, 101)],
        #[cfg(test)]
        StatementProfile::Task258B3M1 => &[(0, 19, 112), (1, 96, 107)],
        #[cfg(test)]
        StatementProfile::Task258B3M2A => &[(0, 19, 106), (1, 90, 101)],
        #[cfg(test)]
        StatementProfile::Task258B3M2B1 => &[(0, 19, 112), (1, 96, 107)],
        #[cfg(test)]
        StatementProfile::Task258B3M2B2A => &[(0, 19, 120), (1, 104, 115)],
        #[cfg(test)]
        StatementProfile::Task258B3M2B2B1A => &[(0, 48, 142), (1, 126, 137)],
        #[cfg(test)]
        StatementProfile::Task258B3M2B2B1B1 => &[(0, 48, 157), (1, 141, 152)],
        StatementProfile::Task258B3M2B2B2A => &[(0, 48, 171), (1, 155, 166)],
        StatementProfile::Task258B3M2B2B2B => &[(0, 48, 170), (1, 154, 165)],
        #[cfg(test)]
        StatementProfile::Task258B3M2B2B2C => &[(0, 48, 180), (1, 164, 175)],
        #[cfg(test)]
        StatementProfile::Task258B3M2B2B3A => &[(0, 19, 116), (1, 100, 111)],
        #[cfg(test)]
        StatementProfile::Task258B3M2B2B3B => &[(0, 19, 117), (1, 101, 112)],
        #[cfg(test)]
        StatementProfile::Task258B3M2B2B3C => &[(0, 19, 109), (1, 93, 104)],
        #[cfg(test)]
        StatementProfile::Task258B3M2B2B3D => &[(0, 19, 108), (1, 92, 103)],
        #[cfg(test)]
        StatementProfile::Task258B3M2B2B3E => &[(0, 19, 138), (1, 122, 133)],
    };
    for (index, (binding_context, start, end)) in expected.iter().copied().enumerate() {
        let id = SourceStatementContextId::new(index);
        let Some(context) = contexts.get(id) else {
            return Err(SourceStatementError::InvalidAggregate);
        };
        let binding_row = bindings
            .contexts()
            .get(BindingContextId::new(binding_context));
        if context.statement != SourceStatementId::new(index)
            || statements.get(context.statement).is_none()
            || context.binding_context != BindingContextId::new(binding_context)
            || context.source_range != range(source_id, start, end)
            || context.visible_bindings != [BindingId::new(0)]
            || binding_row.is_none_or(|row| row.visible_bindings != context.visible_bindings)
        {
            return Err(SourceStatementError::InvalidContext { context: id });
        }
    }
    Ok(())
}

fn validate_input_fact_rows(
    profile: StatementProfile,
    input_facts: &SourceStatementInputFactTable,
    statements: &SourceStatementTable,
    contexts: &SourceStatementContextTable,
    bindings: &BindingEnv,
    primary_terms: &SourcePrimaryTermHandoff,
) -> Result<(), SourceStatementError> {
    let rows = match profile {
        #[cfg(test)]
        StatementProfile::Task258A => 1,
        #[cfg(test)]
        StatementProfile::Task258B1 => 4,
        #[cfg(test)]
        StatementProfile::Task258B5A => 5,
        StatementProfile::Task258B5B => 2,
        #[cfg(test)]
        StatementProfile::Task258B2 => 3,
        #[cfg(test)]
        StatementProfile::Task258B3 => 2,
        #[cfg(test)]
        StatementProfile::Task258B3N => 2,
        #[cfg(test)]
        StatementProfile::Task258B3M1 => 2,
        #[cfg(test)]
        StatementProfile::Task258B3M2A => 2,
        #[cfg(test)]
        StatementProfile::Task258B3M2B1 => 2,
        #[cfg(test)]
        StatementProfile::Task258B3M2B2A => 2,
        #[cfg(test)]
        StatementProfile::Task258B3M2B2B1A => 2,
        #[cfg(test)]
        StatementProfile::Task258B3M2B2B1B1 => 2,
        StatementProfile::Task258B3M2B2B2A => 2,
        StatementProfile::Task258B3M2B2B2B => 2,
        #[cfg(test)]
        StatementProfile::Task258B3M2B2B2C => 2,
        #[cfg(test)]
        StatementProfile::Task258B3M2B2B3A => 2,
        #[cfg(test)]
        StatementProfile::Task258B3M2B2B3B => 2,
        #[cfg(test)]
        StatementProfile::Task258B3M2B2B3C => 2,
        #[cfg(test)]
        StatementProfile::Task258B3M2B2B3D => 2,
        #[cfg(test)]
        StatementProfile::Task258B3M2B2B3E => 2,
    };
    for index in 0..rows {
        let id = SourceStatementInputFactId::new(index);
        let Some(fact) = input_facts.get(id) else {
            return Err(SourceStatementError::InvalidAggregate);
        };
        let first_term = match (profile, index) {
            #[cfg(test)]
            (StatementProfile::Task258B3M2B2A, 1) => 5,
            #[cfg(test)]
            (StatementProfile::Task258B3M2B2B1A, 1) => 4,
            #[cfg(test)]
            (StatementProfile::Task258B3M2B2B1B1, 1) => 4,
            (StatementProfile::Task258B3M2B2B2A, 1) => 4,
            (StatementProfile::Task258B3M2B2B2B, 1) => 4,
            #[cfg(test)]
            (StatementProfile::Task258B3M2B2B2C, 1) => 5,
            #[cfg(test)]
            (StatementProfile::Task258B3M2B2B3A, 1) => 4,
            #[cfg(test)]
            (StatementProfile::Task258B3M2B2B3B, 1) => 2,
            #[cfg(test)]
            (StatementProfile::Task258B3M2B2B3C, 1) => 2,
            #[cfg(test)]
            (StatementProfile::Task258B3M2B2B3D, 1) => 3,
            #[cfg(test)]
            (StatementProfile::Task258B3M2B2B3E, 1) => 3,
            #[cfg(test)]
            (StatementProfile::Task258B3M1, 1) => 4,
            #[cfg(test)]
            (StatementProfile::Task258B3M2B1, 1) => 4,
            #[cfg(test)]
            (StatementProfile::Task258B3, 1) => 3,
            #[cfg(test)]
            (StatementProfile::Task258B3N, 1) => 3,
            #[cfg(test)]
            (StatementProfile::Task258B3M2A, 1) => 3,
            _ => index * 2,
        };
        let first_reference = match (profile, index) {
            #[cfg(test)]
            (StatementProfile::Task258B3M2A, 1) => 2,
            #[cfg(test)]
            (StatementProfile::Task258B3M2B1, 1) => 3,
            #[cfg(test)]
            (StatementProfile::Task258B3M2B2A, 1) => 3,
            #[cfg(test)]
            (StatementProfile::Task258B3M2B2B1A, 1) => 2,
            #[cfg(test)]
            (StatementProfile::Task258B3M2B2B1B1, 1) => 2,
            (StatementProfile::Task258B3M2B2B2A, 1) => 2,
            (StatementProfile::Task258B3M2B2B2B, 1) => 2,
            #[cfg(test)]
            (StatementProfile::Task258B3M2B2B2C, 1) => 2,
            #[cfg(test)]
            (StatementProfile::Task258B3M2B2B3A, 1) => 2,
            #[cfg(test)]
            (StatementProfile::Task258B3M2B2B3B, 1) => 2,
            #[cfg(test)]
            (StatementProfile::Task258B3M2B2B3C, 1) => 2,
            #[cfg(test)]
            (StatementProfile::Task258B3M2B2B3D, 1) => 2,
            #[cfg(test)]
            (StatementProfile::Task258B3M2B2B3E, 1) => 2,
            _ => first_term,
        };
        let expected_uses = [
            SourcePrimaryTermReferenceId::new(first_reference),
            SourcePrimaryTermReferenceId::new(first_reference + 1),
        ];
        if fact.statement != SourceStatementId::new(index)
            || statements.get(fact.statement).is_none()
            || fact.context != SourceStatementContextId::new(index)
            || contexts.get(fact.context).is_none()
            || fact.ordinal != 0
            || fact.kind != SourceStatementInputFactKind::ReservedTypeGuard
            || fact.binding != BindingId::new(0)
            || bindings.bindings().get(fact.binding).is_none()
            || fact.uses != expected_uses
        {
            return Err(SourceStatementError::InvalidInputFact { fact: id });
        }
        for (term_index, use_id) in expected_uses.into_iter().enumerate() {
            let Some(reference) = primary_terms.references().get(use_id) else {
                return Err(SourceStatementError::InvalidInputFact { fact: id });
            };
            if reference.term() != SourcePrimaryTermId::new(first_term + term_index)
                || reference.binding() != fact.binding
                || reference.role() != SourcePrimaryTermReferenceRole::Variable
                || reference.use_ordinal() != 1
            {
                return Err(SourceStatementError::InvalidInputFact { fact: id });
            }
        }
    }
    Ok(())
}

fn validate_candidate_fact_rows(
    profile: StatementProfile,
    candidate_facts: &SourceStatementCandidateFactTable,
    statements: &SourceStatementTable,
    contexts: &SourceStatementContextTable,
    atomic_formulas: &SourceAtomicFormulaHandoff,
) -> Result<(), SourceStatementError> {
    let rows = match profile {
        #[cfg(test)]
        StatementProfile::Task258A => 1,
        #[cfg(test)]
        StatementProfile::Task258B1 => 4,
        #[cfg(test)]
        StatementProfile::Task258B5A => 5,
        StatementProfile::Task258B5B => 2,
        #[cfg(test)]
        StatementProfile::Task258B2 => 3,
        #[cfg(test)]
        StatementProfile::Task258B3 => 2,
        #[cfg(test)]
        StatementProfile::Task258B3N => 2,
        #[cfg(test)]
        StatementProfile::Task258B3M1 => 2,
        #[cfg(test)]
        StatementProfile::Task258B3M2A => 2,
        #[cfg(test)]
        StatementProfile::Task258B3M2B1 => 2,
        #[cfg(test)]
        StatementProfile::Task258B3M2B2A => 2,
        #[cfg(test)]
        StatementProfile::Task258B3M2B2B1A => 2,
        #[cfg(test)]
        StatementProfile::Task258B3M2B2B1B1 => 2,
        StatementProfile::Task258B3M2B2B2A => 2,
        StatementProfile::Task258B3M2B2B2B => 2,
        #[cfg(test)]
        StatementProfile::Task258B3M2B2B2C => 2,
        #[cfg(test)]
        StatementProfile::Task258B3M2B2B3A => 2,
        #[cfg(test)]
        StatementProfile::Task258B3M2B2B3B => 2,
        #[cfg(test)]
        StatementProfile::Task258B3M2B2B3C => 2,
        #[cfg(test)]
        StatementProfile::Task258B3M2B2B3D => 2,
        #[cfg(test)]
        StatementProfile::Task258B3M2B2B3E => 2,
    };
    for index in 0..rows {
        let id = SourceStatementCandidateFactId::new(index);
        let Some(fact) = candidate_facts.get(id) else {
            return Err(SourceStatementError::InvalidAggregate);
        };
        let formula = SourceStatementFormulaTarget::Atomic(SourceAtomicFormulaId::new(index));
        if fact.statement != SourceStatementId::new(index)
            || statements.get(fact.statement).is_none()
            || fact.context != SourceStatementContextId::new(index)
            || contexts.get(fact.context).is_none()
            || fact.ordinal != 0
            || fact.kind != SourceStatementCandidateFactKind::UnverifiedProposition
            || fact.formula != formula
            || statements
                .get(fact.statement)
                .is_none_or(|statement| statement.formula != fact.formula)
            || atomic_formulas
                .formulas()
                .get(SourceAtomicFormulaId::new(index))
                .is_none()
        {
            return Err(SourceStatementError::InvalidCandidateFact { fact: id });
        }
    }
    Ok(())
}

#[cfg(test)]
const TASK258B5A_NODE_RANGES: [(usize, usize); 93] = [
    (0, 7),
    (8, 9),
    (10, 13),
    (14, 17),
    (17, 18),
    (19, 26),
    (27, 79),
    (79, 80),
    (81, 82),
    (83, 84),
    (85, 86),
    (87, 92),
    (95, 96),
    (96, 97),
    (98, 99),
    (100, 101),
    (102, 103),
    (104, 109),
    (114, 118),
    (119, 120),
    (121, 122),
    (123, 124),
    (124, 125),
    (128, 131),
    (131, 132),
    (135, 139),
    (140, 141),
    (142, 143),
    (144, 145),
    (146, 151),
    (156, 160),
    (161, 162),
    (163, 164),
    (165, 166),
    (167, 169),
    (170, 171),
    (171, 172),
    (175, 178),
    (178, 179),
    (180, 183),
    (183, 184),
    (14, 17),
    (14, 17),
    (8, 17),
    (0, 18),
    (81, 82),
    (81, 82),
    (85, 86),
    (85, 86),
    (81, 86),
    (81, 86),
    (98, 99),
    (98, 99),
    (102, 103),
    (102, 103),
    (98, 103),
    (98, 103),
    (95, 103),
    (119, 120),
    (119, 120),
    (123, 124),
    (123, 124),
    (119, 124),
    (119, 124),
    (119, 124),
    (114, 125),
    (104, 131),
    (95, 132),
    (140, 141),
    (140, 141),
    (144, 145),
    (144, 145),
    (140, 145),
    (140, 145),
    (140, 145),
    (161, 162),
    (161, 162),
    (165, 166),
    (165, 166),
    (161, 166),
    (161, 166),
    (161, 166),
    (170, 171),
    (170, 171),
    (167, 171),
    (156, 172),
    (146, 178),
    (135, 179),
    (87, 183),
    (19, 184),
    (0, 184),
    (0, 184),
    (0, 184),
];

#[cfg(test)]
fn task258b5a_node_children(index: usize) -> &'static [usize] {
    match index {
        41 => &[3],
        42 => &[41],
        43 => &[1, 2, 42],
        44 => &[0, 43, 4],
        45 => &[8],
        46 => &[45],
        47 => &[10],
        48 => &[47],
        49 => &[46, 9, 48],
        50 => &[49],
        51 => &[14],
        52 => &[51],
        53 => &[16],
        54 => &[53],
        55 => &[52, 15, 54],
        56 => &[55],
        57 => &[12, 13, 56],
        58 => &[19],
        59 => &[58],
        60 => &[21],
        61 => &[60],
        62 => &[59, 20, 61],
        63 => &[62],
        64 => &[63],
        65 => &[18, 64, 22],
        66 => &[17, 65, 23],
        67 => &[57, 66, 24],
        68 => &[26],
        69 => &[68],
        70 => &[28],
        71 => &[70],
        72 => &[69, 27, 71],
        73 => &[72],
        74 => &[73],
        75 => &[31],
        76 => &[75],
        77 => &[33],
        78 => &[77],
        79 => &[76, 32, 78],
        80 => &[79],
        81 => &[80],
        82 => &[35],
        83 => &[82],
        84 => &[34, 83],
        85 => &[30, 81, 84, 36],
        86 => &[29, 85, 37],
        87 => &[25, 74, 86, 38],
        88 => &[11, 67, 87, 39],
        89 => &[5, 6, 7, 50, 88, 40],
        90 => &[44, 89],
        91 => &[90],
        92 => &[
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23,
            24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 91,
        ],
        _ => &[],
    }
}

#[cfg(test)]
fn task258b5a_node_kind(index: usize) -> &'static str {
    match index {
        45 | 47 | 51 | 53 | 58 | 60 | 68 | 70 | 75 | 77 => "source.term.variable-reference",
        49 | 55 | 62 | 72 | 79 => "source.formula.atomic.equality",
        65 | 85 | 87 => "source.statement.conclusion",
        67 => "source.statement.proof-step",
        89 => "source.statement.theorem",
        _ => "source.surface.unowned",
    }
}

#[cfg(test)]
fn task258b5a_resolver_node_kind(index: usize) -> &'static str {
    match index {
        0 => r#"Token(SurfaceToken { kind: ReservedWord, text: "reserve" })"#,
        1 => r#"Token(SurfaceToken { kind: Identifier, text: "x" })"#,
        2 => r#"Token(SurfaceToken { kind: ReservedWord, text: "for" })"#,
        3 => r#"Token(SurfaceToken { kind: ReservedWord, text: "set" })"#,
        4 => r#"Token(SurfaceToken { kind: ReservedSymbol, text: ";" })"#,
        5 => r#"Token(SurfaceToken { kind: ReservedWord, text: "theorem" })"#,
        6 => {
            r#"Token(SurfaceToken { kind: Identifier, text: "FormulaStatementAncestorLabelDescendantCitationSmoke" })"#
        }
        7 => r#"Token(SurfaceToken { kind: ReservedSymbol, text: ":" })"#,
        8 => r#"Token(SurfaceToken { kind: Identifier, text: "x" })"#,
        9 => r#"Token(SurfaceToken { kind: ReservedSymbol, text: "=" })"#,
        10 => r#"Token(SurfaceToken { kind: Identifier, text: "x" })"#,
        11 => r#"Token(SurfaceToken { kind: ReservedWord, text: "proof" })"#,
        12 => r#"Token(SurfaceToken { kind: Identifier, text: "A" })"#,
        13 => r#"Token(SurfaceToken { kind: ReservedSymbol, text: ":" })"#,
        14 => r#"Token(SurfaceToken { kind: Identifier, text: "x" })"#,
        15 => r#"Token(SurfaceToken { kind: ReservedSymbol, text: "=" })"#,
        16 => r#"Token(SurfaceToken { kind: Identifier, text: "x" })"#,
        17 => r#"Token(SurfaceToken { kind: ReservedWord, text: "proof" })"#,
        18 => r#"Token(SurfaceToken { kind: ReservedWord, text: "thus" })"#,
        19 => r#"Token(SurfaceToken { kind: Identifier, text: "x" })"#,
        20 => r#"Token(SurfaceToken { kind: ReservedSymbol, text: "=" })"#,
        21 => r#"Token(SurfaceToken { kind: Identifier, text: "x" })"#,
        22 => r#"Token(SurfaceToken { kind: ReservedSymbol, text: ";" })"#,
        23 => r#"Token(SurfaceToken { kind: ReservedWord, text: "end" })"#,
        24 => r#"Token(SurfaceToken { kind: ReservedSymbol, text: ";" })"#,
        25 => r#"Token(SurfaceToken { kind: ReservedWord, text: "thus" })"#,
        26 => r#"Token(SurfaceToken { kind: Identifier, text: "x" })"#,
        27 => r#"Token(SurfaceToken { kind: ReservedSymbol, text: "=" })"#,
        28 => r#"Token(SurfaceToken { kind: Identifier, text: "x" })"#,
        29 => r#"Token(SurfaceToken { kind: ReservedWord, text: "proof" })"#,
        30 => r#"Token(SurfaceToken { kind: ReservedWord, text: "thus" })"#,
        31 => r#"Token(SurfaceToken { kind: Identifier, text: "x" })"#,
        32 => r#"Token(SurfaceToken { kind: ReservedSymbol, text: "=" })"#,
        33 => r#"Token(SurfaceToken { kind: Identifier, text: "x" })"#,
        34 => r#"Token(SurfaceToken { kind: ReservedWord, text: "by" })"#,
        35 => r#"Token(SurfaceToken { kind: Identifier, text: "A" })"#,
        36 => r#"Token(SurfaceToken { kind: ReservedSymbol, text: ";" })"#,
        37 => r#"Token(SurfaceToken { kind: ReservedWord, text: "end" })"#,
        38 => r#"Token(SurfaceToken { kind: ReservedSymbol, text: ";" })"#,
        39 => r#"Token(SurfaceToken { kind: ReservedWord, text: "end" })"#,
        40 => r#"Token(SurfaceToken { kind: ReservedSymbol, text: ";" })"#,
        41 => "TypeHead",
        42 => "TypeExpression",
        43 => "ReserveSegment",
        44 => "ReserveItem",
        45 | 47 | 51 | 53 | 58 | 60 | 68 | 70 | 75 | 77 => "TermReference",
        46 | 48 | 52 | 54 | 59 | 61 | 69 | 71 | 76 | 78 => "TermExpression",
        49 | 55 | 62 | 72 | 79 => "BuiltinPredicateApplication",
        50 | 56 | 63 | 73 | 80 => "FormulaExpression",
        57 | 64 | 74 | 81 => "Proposition",
        65 | 85 | 87 => "ConclusionStatement",
        66 | 86 | 88 => "ProofBlock",
        67 => "CompactStatement",
        82 => "Reference",
        83 => "ReferenceList",
        84 => "JustificationClause",
        89 => "TheoremItem",
        90 => "ItemList",
        91 => "CompilationUnit",
        92 => "Root",
        _ => unreachable!("Task258B5A has exactly 93 resolver nodes"),
    }
}

#[cfg(test)]
fn exact_task258b5a_shared_arena(source_id: SourceId, arena: &TypedArena) -> bool {
    if arena.len() != TASK258B5A_NODE_RANGES.len()
        || arena.root() != Some(crate::typed_ast::TypedNodeId::new(92))
    {
        return false;
    }
    TASK258B5A_NODE_RANGES
        .iter()
        .copied()
        .enumerate()
        .all(|(index, (start, end))| {
            arena
                .node(crate::typed_ast::TypedNodeId::new(index))
                .is_some_and(|node| {
                    node.anchor == SourceAnchor::Range(range(source_id, start, end))
                        && node.kind.as_str() == task258b5a_node_kind(index)
                        && node.resolved_node.is_none()
                        && node.typing == TypingState::Unknown
                        && node.recovery == NodeRecoveryState::Normal
                        && node.links.context.is_none()
                        && node.links.type_entry.is_none()
                        && node.links.facts.is_empty()
                        && node.links.coercions.is_empty()
                        && node.links.initial_obligations.is_empty()
                        && node.links.diagnostics.is_empty()
                        && node
                            .children
                            .iter()
                            .map(|child| child.index())
                            .eq(task258b5a_node_children(index).iter().copied())
                })
        })
}

const TASK258B5B_NODE_RANGES: [(usize, usize); 57] = [
    (0, 6),
    (7, 13),
    (13, 14),
    (14, 27),
    (27, 28),
    (29, 36),
    (37, 38),
    (39, 42),
    (43, 46),
    (46, 47),
    (48, 55),
    (56, 106),
    (106, 107),
    (108, 109),
    (110, 111),
    (112, 113),
    (114, 119),
    (122, 126),
    (127, 128),
    (129, 130),
    (131, 132),
    (133, 135),
    (136, 139),
    (139, 140),
    (141, 144),
    (144, 145),
    (7, 13),
    (14, 27),
    (7, 27),
    (7, 27),
    (0, 28),
    (43, 46),
    (43, 46),
    (37, 46),
    (29, 47),
    (108, 109),
    (108, 109),
    (112, 113),
    (112, 113),
    (108, 113),
    (108, 113),
    (127, 128),
    (127, 128),
    (131, 132),
    (131, 132),
    (127, 132),
    (127, 132),
    (127, 132),
    (136, 139),
    (136, 139),
    (133, 139),
    (122, 140),
    (114, 144),
    (48, 145),
    (0, 145),
    (0, 145),
    (0, 145),
];

const TASK258B5B_NODE_CHILDREN: [&[usize]; 57] = [
    &[],
    &[],
    &[],
    &[],
    &[],
    &[],
    &[],
    &[],
    &[],
    &[],
    &[],
    &[],
    &[],
    &[],
    &[],
    &[],
    &[],
    &[],
    &[],
    &[],
    &[],
    &[],
    &[],
    &[],
    &[],
    &[],
    &[1],
    &[3],
    &[26, 2, 27],
    &[28],
    &[0, 29, 4],
    &[8],
    &[31],
    &[6, 7, 32],
    &[5, 33, 9],
    &[13],
    &[35],
    &[15],
    &[37],
    &[36, 14, 38],
    &[39],
    &[18],
    &[41],
    &[20],
    &[43],
    &[42, 19, 44],
    &[45],
    &[46],
    &[22],
    &[48],
    &[21, 49],
    &[17, 47, 50, 23],
    &[16, 51, 24],
    &[10, 11, 12, 40, 52, 25],
    &[30, 34, 53],
    &[54],
    &[
        0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24,
        25, 55,
    ],
];

const TASK258B5B_RESOLVER_NODE_KINDS: [&str; 57] = [
    r#"Token(SurfaceToken { kind: ReservedWord, text: "import" })"#,
    r#"Token(SurfaceToken { kind: Identifier, text: "parser" })"#,
    r#"Token(SurfaceToken { kind: ReservedSymbol, text: "." })"#,
    r#"Token(SurfaceToken { kind: Identifier, text: "type_fixtures" })"#,
    r#"Token(SurfaceToken { kind: ReservedSymbol, text: ";" })"#,
    r#"Token(SurfaceToken { kind: ReservedWord, text: "reserve" })"#,
    r#"Token(SurfaceToken { kind: Identifier, text: "x" })"#,
    r#"Token(SurfaceToken { kind: ReservedWord, text: "for" })"#,
    r#"Token(SurfaceToken { kind: ReservedWord, text: "set" })"#,
    r#"Token(SurfaceToken { kind: ReservedSymbol, text: ";" })"#,
    r#"Token(SurfaceToken { kind: ReservedWord, text: "theorem" })"#,
    r#"Token(SurfaceToken { kind: Identifier, text: "FormulaStatementImportedPublicTheoremCitationSmoke" })"#,
    r#"Token(SurfaceToken { kind: ReservedSymbol, text: ":" })"#,
    r#"Token(SurfaceToken { kind: Identifier, text: "x" })"#,
    r#"Token(SurfaceToken { kind: ReservedSymbol, text: "=" })"#,
    r#"Token(SurfaceToken { kind: Identifier, text: "x" })"#,
    r#"Token(SurfaceToken { kind: ReservedWord, text: "proof" })"#,
    r#"Token(SurfaceToken { kind: ReservedWord, text: "thus" })"#,
    r#"Token(SurfaceToken { kind: Identifier, text: "x" })"#,
    r#"Token(SurfaceToken { kind: ReservedSymbol, text: "=" })"#,
    r#"Token(SurfaceToken { kind: Identifier, text: "x" })"#,
    r#"Token(SurfaceToken { kind: ReservedWord, text: "by" })"#,
    r#"Token(SurfaceToken { kind: Identifier, text: "Ref" })"#,
    r#"Token(SurfaceToken { kind: ReservedSymbol, text: ";" })"#,
    r#"Token(SurfaceToken { kind: ReservedWord, text: "end" })"#,
    r#"Token(SurfaceToken { kind: ReservedSymbol, text: ";" })"#,
    "PathSegment",
    "PathSegment",
    "ModulePath",
    "ImportAliasDecl",
    "ImportItem",
    "TypeHead",
    "TypeExpression",
    "ReserveSegment",
    "ReserveItem",
    "TermReference",
    "TermExpression",
    "TermReference",
    "TermExpression",
    "BuiltinPredicateApplication",
    "FormulaExpression",
    "TermReference",
    "TermExpression",
    "TermReference",
    "TermExpression",
    "BuiltinPredicateApplication",
    "FormulaExpression",
    "Proposition",
    "Reference",
    "ReferenceList",
    "JustificationClause",
    "ConclusionStatement",
    "ProofBlock",
    "TheoremItem",
    "ItemList",
    "CompilationUnit",
    "Root",
];

fn task258b5b_typed_node_kind(index: usize) -> &'static str {
    match index {
        35 | 37 | 41 | 43 => "source.term.variable-reference",
        40 | 46 => "source.formula.atomic.equality",
        51 => "source.statement.conclusion",
        53 => "source.statement.theorem",
        _ => "source.surface.unowned",
    }
}

fn exact_task258b5b_shared_arena(source_id: SourceId, arena: &TypedArena) -> bool {
    if arena.len() != TASK258B5B_NODE_RANGES.len()
        || arena.root().is_none_or(|root| root.index() != 56)
    {
        return false;
    }
    arena.iter().all(|(id, node)| {
        let index = id.index();
        node.kind.as_str() == task258b5b_typed_node_kind(index)
            && node.anchor
                == SourceAnchor::Range(range(
                    source_id,
                    TASK258B5B_NODE_RANGES[index].0,
                    TASK258B5B_NODE_RANGES[index].1,
                ))
            && node
                .children
                .iter()
                .map(|child| child.index())
                .eq(TASK258B5B_NODE_CHILDREN[index].iter().copied())
            && node.resolved_node.is_none()
            && node.typing == TypingState::Unknown
            && node.recovery == NodeRecoveryState::Normal
            && node.links == Default::default()
    })
}

#[cfg(test)]
const TASK258B3_NODE_RANGES: [(usize, usize); 49] = [
    (0, 7),
    (8, 9),
    (10, 13),
    (14, 17),
    (17, 18),
    (19, 26),
    (27, 61),
    (61, 62),
    (63, 64),
    (65, 66),
    (67, 68),
    (69, 74),
    (77, 81),
    (82, 83),
    (83, 84),
    (87, 91),
    (92, 93),
    (94, 95),
    (96, 97),
    (97, 98),
    (99, 102),
    (102, 103),
    (14, 17),
    (14, 17),
    (8, 17),
    (0, 18),
    (63, 64),
    (63, 64),
    (67, 68),
    (67, 68),
    (63, 68),
    (63, 68),
    (82, 83),
    (82, 83),
    (82, 83),
    (77, 84),
    (92, 93),
    (92, 93),
    (96, 97),
    (96, 97),
    (92, 97),
    (92, 97),
    (92, 97),
    (87, 98),
    (69, 102),
    (19, 103),
    (0, 103),
    (0, 103),
    (0, 103),
];

#[cfg(test)]
const TASK258B3N_NODE_RANGES: [(usize, usize); 51] = [
    (0, 7),
    (8, 9),
    (10, 13),
    (14, 17),
    (17, 18),
    (19, 26),
    (27, 60),
    (60, 61),
    (62, 63),
    (64, 65),
    (66, 67),
    (68, 73),
    (76, 80),
    (81, 82),
    (83, 84),
    (85, 86),
    (86, 87),
    (90, 94),
    (95, 96),
    (97, 98),
    (99, 100),
    (100, 101),
    (102, 105),
    (105, 106),
    (14, 17),
    (14, 17),
    (8, 17),
    (0, 18),
    (62, 63),
    (62, 63),
    (66, 67),
    (66, 67),
    (62, 67),
    (62, 67),
    (85, 86),
    (85, 86),
    (81, 86),
    (76, 87),
    (95, 96),
    (95, 96),
    (99, 100),
    (99, 100),
    (95, 100),
    (95, 100),
    (95, 100),
    (90, 101),
    (68, 105),
    (19, 106),
    (0, 106),
    (0, 106),
    (0, 106),
];

#[cfg(test)]
const TASK258B3M1_NODE_RANGES: [(usize, usize); 56] = [
    (0, 7),
    (8, 9),
    (10, 13),
    (14, 17),
    (17, 18),
    (19, 26),
    (27, 63),
    (63, 64),
    (65, 66),
    (67, 68),
    (69, 70),
    (71, 76),
    (79, 83),
    (84, 85),
    (86, 87),
    (88, 89),
    (89, 90),
    (91, 92),
    (92, 93),
    (96, 100),
    (101, 102),
    (103, 104),
    (105, 106),
    (106, 107),
    (108, 111),
    (111, 112),
    (14, 17),
    (14, 17),
    (8, 17),
    (0, 18),
    (65, 66),
    (65, 66),
    (69, 70),
    (69, 70),
    (65, 70),
    (65, 70),
    (88, 89),
    (88, 89),
    (84, 89),
    (91, 92),
    (91, 92),
    (91, 92),
    (79, 93),
    (101, 102),
    (101, 102),
    (105, 106),
    (105, 106),
    (101, 106),
    (101, 106),
    (101, 106),
    (96, 107),
    (71, 111),
    (19, 112),
    (0, 112),
    (0, 112),
    (0, 112),
];

#[cfg(test)]
const TASK258B3M2A_NODE_RANGES: [(usize, usize); 49] = [
    (0, 7),
    (8, 9),
    (10, 13),
    (14, 17),
    (17, 18),
    (19, 26),
    (27, 62),
    (62, 63),
    (64, 65),
    (66, 67),
    (68, 69),
    (70, 75),
    (78, 82),
    (83, 86),
    (86, 87),
    (90, 94),
    (95, 96),
    (97, 98),
    (99, 100),
    (100, 101),
    (102, 105),
    (105, 106),
    (14, 17),
    (14, 17),
    (8, 17),
    (0, 18),
    (64, 65),
    (64, 65),
    (68, 69),
    (68, 69),
    (64, 69),
    (64, 69),
    (83, 86),
    (83, 86),
    (83, 86),
    (78, 87),
    (95, 96),
    (95, 96),
    (99, 100),
    (99, 100),
    (95, 100),
    (95, 100),
    (95, 100),
    (90, 101),
    (70, 105),
    (19, 106),
    (0, 106),
    (0, 106),
    (0, 106),
];

#[cfg(test)]
const TASK258B3M2B1_NODE_RANGES: [(usize, usize); 53] = [
    (0, 7),
    (8, 9),
    (10, 13),
    (14, 17),
    (17, 18),
    (19, 26),
    (27, 68),
    (68, 69),
    (70, 71),
    (72, 73),
    (74, 75),
    (76, 81),
    (84, 88),
    (89, 90),
    (90, 91),
    (91, 92),
    (92, 93),
    (96, 100),
    (101, 102),
    (103, 104),
    (105, 106),
    (106, 107),
    (108, 111),
    (111, 112),
    (14, 17),
    (14, 17),
    (8, 17),
    (0, 18),
    (70, 71),
    (70, 71),
    (74, 75),
    (74, 75),
    (70, 75),
    (70, 75),
    (90, 91),
    (90, 91),
    (89, 92),
    (89, 92),
    (89, 92),
    (84, 93),
    (101, 102),
    (101, 102),
    (105, 106),
    (105, 106),
    (101, 106),
    (101, 106),
    (101, 106),
    (96, 107),
    (76, 111),
    (19, 112),
    (0, 112),
    (0, 112),
    (0, 112),
];

#[cfg(test)]
const TASK258B3M2B2A_NODE_RANGES: [(usize, usize); 57] = [
    (0, 7),
    (8, 9),
    (10, 13),
    (14, 17),
    (17, 18),
    (19, 26),
    (27, 74),
    (74, 75),
    (76, 77),
    (78, 79),
    (80, 81),
    (82, 87),
    (90, 94),
    (95, 96),
    (96, 97),
    (97, 98),
    (98, 99),
    (99, 100),
    (100, 101),
    (104, 108),
    (109, 110),
    (111, 112),
    (113, 114),
    (114, 115),
    (116, 119),
    (119, 120),
    (14, 17),
    (14, 17),
    (8, 17),
    (0, 18),
    (76, 77),
    (76, 77),
    (80, 81),
    (80, 81),
    (76, 81),
    (76, 81),
    (97, 98),
    (97, 98),
    (96, 99),
    (96, 99),
    (95, 100),
    (95, 100),
    (95, 100),
    (90, 101),
    (109, 110),
    (109, 110),
    (113, 114),
    (113, 114),
    (109, 114),
    (109, 114),
    (109, 114),
    (104, 115),
    (82, 119),
    (19, 120),
    (0, 120),
    (0, 120),
    (0, 120),
];

#[cfg(test)]
const TASK258B3M2B2B1A_NODE_RANGES: [(usize, usize); 63] = [
    (0, 6),
    (7, 13),
    (13, 14),
    (14, 27),
    (27, 28),
    (29, 36),
    (37, 38),
    (39, 42),
    (43, 46),
    (46, 47),
    (48, 55),
    (56, 95),
    (95, 96),
    (97, 98),
    (99, 100),
    (101, 102),
    (103, 108),
    (111, 115),
    (116, 117),
    (118, 120),
    (121, 122),
    (122, 123),
    (126, 130),
    (131, 132),
    (133, 134),
    (135, 136),
    (136, 137),
    (138, 141),
    (141, 142),
    (7, 13),
    (14, 27),
    (7, 27),
    (7, 27),
    (0, 28),
    (43, 46),
    (43, 46),
    (37, 46),
    (29, 47),
    (97, 98),
    (97, 98),
    (101, 102),
    (101, 102),
    (97, 102),
    (97, 102),
    (116, 117),
    (121, 122),
    (116, 122),
    (116, 122),
    (116, 122),
    (111, 123),
    (131, 132),
    (131, 132),
    (135, 136),
    (135, 136),
    (131, 136),
    (131, 136),
    (131, 136),
    (126, 137),
    (103, 141),
    (48, 142),
    (0, 142),
    (0, 142),
    (0, 142),
];

#[cfg(test)]
const TASK258B3M2B2B1B1_NODE_RANGES: [(usize, usize); 67] = [
    (0, 6),
    (7, 13),
    (13, 14),
    (14, 27),
    (27, 28),
    (29, 36),
    (37, 38),
    (39, 42),
    (43, 46),
    (46, 47),
    (48, 55),
    (56, 108),
    (108, 109),
    (110, 111),
    (112, 113),
    (114, 115),
    (116, 121),
    (124, 128),
    (129, 130),
    (130, 131),
    (132, 134),
    (135, 136),
    (136, 137),
    (137, 138),
    (141, 145),
    (146, 147),
    (148, 149),
    (150, 151),
    (151, 152),
    (153, 156),
    (156, 157),
    (7, 13),
    (14, 27),
    (7, 27),
    (7, 27),
    (0, 28),
    (43, 46),
    (43, 46),
    (37, 46),
    (29, 47),
    (110, 111),
    (110, 111),
    (114, 115),
    (114, 115),
    (110, 115),
    (110, 115),
    (130, 131),
    (135, 136),
    (130, 136),
    (130, 136),
    (129, 137),
    (129, 137),
    (129, 137),
    (124, 138),
    (146, 147),
    (146, 147),
    (150, 151),
    (150, 151),
    (146, 151),
    (146, 151),
    (146, 151),
    (141, 152),
    (116, 156),
    (48, 157),
    (0, 157),
    (0, 157),
    (0, 157),
];

const TASK258B3M2B2B2A_NODE_RANGES: [(usize, usize); 76] = [
    (0, 6),
    (7, 13),
    (13, 14),
    (14, 27),
    (27, 28),
    (29, 36),
    (37, 38),
    (39, 42),
    (43, 46),
    (46, 47),
    (48, 55),
    (56, 104),
    (104, 105),
    (106, 107),
    (108, 109),
    (110, 111),
    (112, 117),
    (120, 124),
    (125, 139),
    (139, 140),
    (140, 141),
    (141, 142),
    (143, 144),
    (144, 145),
    (146, 147),
    (147, 148),
    (149, 150),
    (150, 151),
    (151, 152),
    (155, 159),
    (160, 161),
    (162, 163),
    (164, 165),
    (165, 166),
    (167, 170),
    (170, 171),
    (7, 13),
    (14, 27),
    (7, 27),
    (7, 27),
    (0, 28),
    (43, 46),
    (43, 46),
    (37, 46),
    (29, 47),
    (106, 107),
    (106, 107),
    (110, 111),
    (110, 111),
    (106, 111),
    (106, 111),
    (125, 139),
    (125, 139),
    (143, 144),
    (143, 144),
    (140, 144),
    (149, 150),
    (149, 150),
    (146, 150),
    (125, 151),
    (125, 151),
    (125, 151),
    (120, 152),
    (160, 161),
    (160, 161),
    (164, 165),
    (164, 165),
    (160, 165),
    (160, 165),
    (160, 165),
    (155, 166),
    (112, 170),
    (48, 171),
    (0, 171),
    (0, 171),
    (0, 171),
];

const TASK258B3M2B2B2B_NODE_RANGES: [(usize, usize); 79] = [
    (0, 6),
    (7, 13),
    (13, 14),
    (14, 27),
    (27, 28),
    (29, 36),
    (37, 38),
    (39, 42),
    (43, 46),
    (46, 47),
    (48, 55),
    (56, 101),
    (101, 102),
    (103, 104),
    (105, 106),
    (107, 108),
    (109, 114),
    (117, 121),
    (122, 136),
    (136, 137),
    (137, 138),
    (138, 139),
    (140, 141),
    (141, 142),
    (143, 144),
    (144, 145),
    (146, 147),
    (147, 148),
    (148, 149),
    (149, 150),
    (150, 151),
    (154, 158),
    (159, 160),
    (161, 162),
    (163, 164),
    (164, 165),
    (166, 169),
    (169, 170),
    (7, 13),
    (14, 27),
    (7, 27),
    (7, 27),
    (0, 28),
    (43, 46),
    (43, 46),
    (37, 46),
    (29, 47),
    (103, 104),
    (103, 104),
    (107, 108),
    (107, 108),
    (103, 108),
    (103, 108),
    (122, 136),
    (122, 136),
    (140, 141),
    (140, 141),
    (137, 141),
    (146, 147),
    (146, 147),
    (143, 147),
    (122, 148),
    (122, 150),
    (122, 150),
    (122, 150),
    (117, 151),
    (159, 160),
    (159, 160),
    (163, 164),
    (163, 164),
    (159, 164),
    (159, 164),
    (159, 164),
    (154, 165),
    (109, 169),
    (48, 170),
    (0, 170),
    (0, 170),
    (0, 170),
];

#[cfg(test)]
const TASK258B3M2B2B2C_NODE_RANGES: [(usize, usize); 86] = [
    (0, 6),
    (7, 13),
    (13, 14),
    (14, 27),
    (27, 28),
    (29, 36),
    (37, 38),
    (39, 42),
    (43, 46),
    (46, 47),
    (48, 55),
    (56, 99),
    (99, 100),
    (101, 102),
    (103, 104),
    (105, 106),
    (107, 112),
    (115, 119),
    (120, 134),
    (134, 135),
    (135, 136),
    (136, 137),
    (138, 139),
    (139, 140),
    (141, 142),
    (142, 143),
    (144, 145),
    (145, 146),
    (147, 151),
    (152, 153),
    (153, 154),
    (155, 157),
    (158, 159),
    (159, 160),
    (160, 161),
    (164, 168),
    (169, 170),
    (171, 172),
    (173, 174),
    (174, 175),
    (176, 179),
    (179, 180),
    (7, 13),
    (14, 27),
    (7, 27),
    (7, 27),
    (0, 28),
    (43, 46),
    (43, 46),
    (37, 46),
    (29, 47),
    (101, 102),
    (101, 102),
    (105, 106),
    (105, 106),
    (101, 106),
    (101, 106),
    (120, 134),
    (120, 134),
    (138, 139),
    (138, 139),
    (135, 139),
    (144, 145),
    (144, 145),
    (141, 145),
    (120, 146),
    (158, 159),
    (158, 159),
    (153, 159),
    (120, 160),
    (120, 160),
    (120, 160),
    (115, 161),
    (169, 170),
    (169, 170),
    (173, 174),
    (173, 174),
    (169, 174),
    (169, 174),
    (169, 174),
    (164, 175),
    (107, 179),
    (48, 180),
    (0, 180),
    (0, 180),
    (0, 180),
];

#[cfg(test)]
fn task258b3_node_children(index: usize) -> &'static [usize] {
    match index {
        22 => &[3],
        23 => &[22],
        24 => &[1, 2, 23],
        25 => &[0, 24, 4],
        26 => &[8],
        27 => &[26],
        28 => &[10],
        29 => &[28],
        30 => &[27, 9, 29],
        31 => &[30],
        32 => &[13],
        33 => &[32],
        34 => &[33],
        35 => &[12, 34, 14],
        36 => &[16],
        37 => &[36],
        38 => &[18],
        39 => &[38],
        40 => &[37, 17, 39],
        41 => &[40],
        42 => &[41],
        43 => &[15, 42, 19],
        44 => &[11, 35, 43, 20],
        45 => &[5, 6, 7, 31, 44, 21],
        46 => &[25, 45],
        47 => &[46],
        48 => &[
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 47,
        ],
        _ => &[],
    }
}

#[cfg(test)]
fn task258b3_node_kind(index: usize) -> &'static str {
    match index {
        26 | 28 | 32 | 36 | 38 => "source.term.variable-reference",
        30 | 40 => "source.formula.atomic.equality",
        34 => "source.statement-witness.item",
        35 => "source.statement-witness.take",
        43 => "source.statement.conclusion",
        45 => "source.statement.theorem",
        _ => "source.surface.unowned",
    }
}

#[cfg(test)]
fn task258b3n_node_children(index: usize) -> &'static [usize] {
    match index {
        24 => &[3],
        25 => &[24],
        26 => &[1, 2, 25],
        27 => &[0, 26, 4],
        28 => &[8],
        29 => &[28],
        30 => &[10],
        31 => &[30],
        32 => &[29, 9, 31],
        33 => &[32],
        34 => &[15],
        35 => &[34],
        36 => &[13, 14, 35],
        37 => &[12, 36, 16],
        38 => &[18],
        39 => &[38],
        40 => &[20],
        41 => &[40],
        42 => &[39, 19, 41],
        43 => &[42],
        44 => &[43],
        45 => &[17, 44, 21],
        46 => &[11, 37, 45, 22],
        47 => &[5, 6, 7, 33, 46, 23],
        48 => &[27, 47],
        49 => &[48],
        50 => &[
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23,
            49,
        ],
        _ => &[],
    }
}

#[cfg(test)]
fn task258b3n_node_kind(index: usize) -> &'static str {
    match index {
        28 | 30 | 34 | 38 | 40 => "source.term.variable-reference",
        32 | 42 => "source.formula.atomic.equality",
        13 => "source.statement-witness.name",
        36 => "source.statement-witness.item",
        37 => "source.statement-witness.take",
        45 => "source.statement.conclusion",
        47 => "source.statement.theorem",
        _ => "source.surface.unowned",
    }
}

#[cfg(test)]
fn task258b3m1_node_children(index: usize) -> &'static [usize] {
    match index {
        26 => &[3],
        27 => &[26],
        28 => &[1, 2, 27],
        29 => &[0, 28, 4],
        30 => &[8],
        31 => &[30],
        32 => &[10],
        33 => &[32],
        34 => &[31, 9, 33],
        35 => &[34],
        36 => &[15],
        37 => &[36],
        38 => &[13, 14, 37],
        39 => &[17],
        40 => &[39],
        41 => &[40],
        42 => &[12, 38, 16, 41, 18],
        43 => &[20],
        44 => &[43],
        45 => &[22],
        46 => &[45],
        47 => &[44, 21, 46],
        48 => &[47],
        49 => &[48],
        50 => &[19, 49, 23],
        51 => &[11, 42, 50, 24],
        52 => &[5, 6, 7, 35, 51, 25],
        53 => &[29, 52],
        54 => &[53],
        55 => &[
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23,
            24, 25, 54,
        ],
        _ => &[],
    }
}

#[cfg(test)]
fn task258b3m1_node_kind(index: usize) -> &'static str {
    match index {
        30 | 32 | 36 | 39 | 43 | 45 => "source.term.variable-reference",
        34 | 47 => "source.formula.atomic.equality",
        13 => "source.statement-witness.name",
        38 | 41 => "source.statement-witness.item",
        42 => "source.statement-witness.take",
        50 => "source.statement.conclusion",
        52 => "source.statement.theorem",
        _ => "source.surface.unowned",
    }
}

#[cfg(test)]
fn task258b3m2a_node_children(index: usize) -> &'static [usize] {
    match index {
        22 => &[3],
        23 => &[22],
        24 => &[1, 2, 23],
        25 => &[0, 24, 4],
        26 => &[8],
        27 => &[26],
        28 => &[10],
        29 => &[28],
        30 => &[27, 9, 29],
        31 => &[30],
        32 => &[13],
        33 => &[32],
        34 => &[33],
        35 => &[12, 34, 14],
        36 => &[16],
        37 => &[36],
        38 => &[18],
        39 => &[38],
        40 => &[37, 17, 39],
        41 => &[40],
        42 => &[41],
        43 => &[15, 42, 19],
        44 => &[11, 35, 43, 20],
        45 => &[5, 6, 7, 31, 44, 21],
        46 => &[25, 45],
        47 => &[46],
        48 => &[
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 47,
        ],
        _ => &[],
    }
}

#[cfg(test)]
fn task258b3m2a_node_kind(index: usize) -> &'static str {
    match index {
        26 | 28 | 36 | 38 => "source.term.variable-reference",
        32 => "source.term.numeral",
        30 | 40 => "source.formula.atomic.equality",
        34 => "source.statement-witness.item",
        35 => "source.statement-witness.take",
        43 => "source.statement.conclusion",
        45 => "source.statement.theorem",
        _ => "source.surface.unowned",
    }
}

#[cfg(test)]
fn task258b3m2b1_node_children(index: usize) -> &'static [usize] {
    match index {
        24 => &[3],
        25 => &[24],
        26 => &[1, 2, 25],
        27 => &[0, 26, 4],
        28 => &[8],
        29 => &[28],
        30 => &[10],
        31 => &[30],
        32 => &[29, 9, 31],
        33 => &[32],
        34 => &[14],
        35 => &[34],
        36 => &[13, 35, 15],
        37 => &[36],
        38 => &[37],
        39 => &[12, 38, 16],
        40 => &[18],
        41 => &[40],
        42 => &[20],
        43 => &[42],
        44 => &[41, 19, 43],
        45 => &[44],
        46 => &[45],
        47 => &[17, 46, 21],
        48 => &[11, 39, 47, 22],
        49 => &[5, 6, 7, 33, 48, 23],
        50 => &[27, 49],
        51 => &[50],
        52 => &[
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23,
            51,
        ],
        _ => &[],
    }
}

#[cfg(test)]
fn task258b3m2b1_node_kind(index: usize) -> &'static str {
    match index {
        28 | 30 | 34 | 40 | 42 => "source.term.variable-reference",
        36 => "source.term.parenthesized",
        32 | 44 => "source.formula.atomic.equality",
        38 => "source.statement-witness.item",
        39 => "source.statement-witness.take",
        47 => "source.statement.conclusion",
        49 => "source.statement.theorem",
        _ => "source.surface.unowned",
    }
}

#[cfg(test)]
fn task258b3m2b2a_node_children(index: usize) -> &'static [usize] {
    match index {
        26 => &[3],
        27 => &[26],
        28 => &[1, 2, 27],
        29 => &[0, 28, 4],
        30 => &[8],
        31 => &[30],
        32 => &[10],
        33 => &[32],
        34 => &[31, 9, 33],
        35 => &[34],
        36 => &[15],
        37 => &[36],
        38 => &[14, 37, 16],
        39 => &[38],
        40 => &[13, 39, 17],
        41 => &[40],
        42 => &[41],
        43 => &[12, 42, 18],
        44 => &[20],
        45 => &[44],
        46 => &[22],
        47 => &[46],
        48 => &[45, 21, 47],
        49 => &[48],
        50 => &[49],
        51 => &[19, 50, 23],
        52 => &[11, 43, 51, 24],
        53 => &[5, 6, 7, 35, 52, 25],
        54 => &[29, 53],
        55 => &[54],
        56 => &[
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23,
            24, 25, 55,
        ],
        _ => &[],
    }
}

#[cfg(test)]
fn task258b3m2b2a_node_kind(index: usize) -> &'static str {
    match index {
        30 | 32 | 36 | 44 | 46 => "source.term.variable-reference",
        38 | 40 => "source.term.parenthesized",
        34 | 48 => "source.formula.atomic.equality",
        42 => "source.statement-witness.item",
        43 => "source.statement-witness.take",
        51 => "source.statement.conclusion",
        53 => "source.statement.theorem",
        _ => "source.surface.unowned",
    }
}

#[cfg(test)]
fn task258b3m2b2b1a_node_children(index: usize) -> &'static [usize] {
    match index {
        29 => &[1],
        30 => &[3],
        31 => &[29, 2, 30],
        32 => &[31],
        33 => &[0, 32, 4],
        34 => &[8],
        35 => &[34],
        36 => &[6, 7, 35],
        37 => &[5, 36, 9],
        38 => &[13],
        39 => &[38],
        40 => &[15],
        41 => &[40],
        42 => &[39, 14, 41],
        43 => &[42],
        44 => &[18],
        45 => &[20],
        46 => &[44, 19, 45],
        47 => &[46],
        48 => &[47],
        49 => &[17, 48, 21],
        50 => &[23],
        51 => &[50],
        52 => &[25],
        53 => &[52],
        54 => &[51, 24, 53],
        55 => &[54],
        56 => &[55],
        57 => &[22, 56, 26],
        58 => &[16, 49, 57, 27],
        59 => &[10, 11, 12, 43, 58, 28],
        60 => &[33, 37, 59],
        61 => &[60],
        62 => &[
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23,
            24, 25, 26, 27, 28, 61,
        ],
        _ => &[],
    }
}

#[cfg(test)]
fn task258b3m2b2b1a_node_kind(index: usize) -> &'static str {
    match index {
        38 | 40 | 50 | 52 => "source.term.variable-reference",
        44 | 45 => "source.term.numeral",
        19 => "source.term.functor-head.single",
        46 => "source.term.functor-application.symbolic",
        42 | 54 => "source.formula.atomic.equality",
        48 => "source.statement-witness.item",
        49 => "source.statement-witness.take",
        57 => "source.statement.conclusion",
        59 => "source.statement.theorem",
        _ => "source.surface.unowned",
    }
}

#[cfg(test)]
fn task258b3m2b2b1b1_node_children(index: usize) -> &'static [usize] {
    match index {
        31 => &[1],
        32 => &[3],
        33 => &[31, 2, 32],
        34 => &[33],
        35 => &[0, 34, 4],
        36 => &[8],
        37 => &[36],
        38 => &[6, 7, 37],
        39 => &[5, 38, 9],
        40 => &[13],
        41 => &[40],
        42 => &[15],
        43 => &[42],
        44 => &[41, 14, 43],
        45 => &[44],
        46 => &[19],
        47 => &[21],
        48 => &[46, 20, 47],
        49 => &[48],
        50 => &[18, 49, 22],
        51 => &[50],
        52 => &[51],
        53 => &[17, 52, 23],
        54 => &[25],
        55 => &[54],
        56 => &[27],
        57 => &[56],
        58 => &[55, 26, 57],
        59 => &[58],
        60 => &[59],
        61 => &[24, 60, 28],
        62 => &[16, 53, 61, 29],
        63 => &[10, 11, 12, 45, 62, 30],
        64 => &[35, 39, 63],
        65 => &[64],
        66 => &[
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23,
            24, 25, 26, 27, 28, 29, 30, 65,
        ],
        _ => &[],
    }
}

#[cfg(test)]
fn task258b3m2b2b1b1_node_kind(index: usize) -> &'static str {
    match index {
        40 | 42 | 54 | 56 => "source.term.variable-reference",
        46 | 47 => "source.term.numeral",
        20 => "source.term.functor-head.single",
        48 => "source.term.functor-application.symbolic",
        50 => "source.term.functor-application.parenthesized",
        44 | 58 => "source.formula.atomic.equality",
        52 => "source.statement-witness.item",
        53 => "source.statement-witness.take",
        61 => "source.statement.conclusion",
        63 => "source.statement.theorem",
        _ => "source.surface.unowned",
    }
}

fn task258b3m2b2b2a_node_children(index: usize) -> &'static [usize] {
    match index {
        36 => &[1],
        37 => &[3],
        38 => &[36, 2, 37],
        39 => &[38],
        40 => &[0, 39, 4],
        41 => &[8],
        42 => &[41],
        43 => &[6, 7, 42],
        44 => &[5, 43, 9],
        45 => &[13],
        46 => &[45],
        47 => &[15],
        48 => &[47],
        49 => &[46, 14, 48],
        50 => &[49],
        51 => &[18],
        52 => &[51],
        53 => &[22],
        54 => &[53],
        55 => &[20, 21, 54],
        56 => &[26],
        57 => &[56],
        58 => &[24, 25, 57],
        59 => &[52, 19, 55, 23, 58, 27],
        60 => &[59],
        61 => &[60],
        62 => &[17, 61, 28],
        63 => &[30],
        64 => &[63],
        65 => &[32],
        66 => &[65],
        67 => &[64, 31, 66],
        68 => &[67],
        69 => &[68],
        70 => &[29, 69, 33],
        71 => &[16, 62, 70, 34],
        72 => &[10, 11, 12, 50, 71, 35],
        73 => &[40, 44, 72],
        74 => &[73],
        75 => &[
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23,
            24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 74,
        ],
        _ => &[],
    }
}

fn task258b3m2b2b2a_node_kind(index: usize) -> &'static str {
    match index {
        45 | 47 | 63 | 65 => "source.term.variable-reference",
        53 | 56 => "source.term.numeral",
        20 | 24 => "source.term.structure.member.constructor-assignment",
        59 => "source.term.structure.constructor",
        49 | 67 => "source.formula.atomic.equality",
        61 => "source.statement-witness.item",
        62 => "source.statement-witness.take",
        70 => "source.statement.conclusion",
        72 => "source.statement.theorem",
        _ => "source.surface.unowned",
    }
}

fn task258b3m2b2b2b_node_children(index: usize) -> &'static [usize] {
    match index {
        38 => &[1],
        39 => &[3],
        40 => &[38, 2, 39],
        41 => &[40],
        42 => &[0, 41, 4],
        43 => &[8],
        44 => &[43],
        45 => &[6, 7, 44],
        46 => &[5, 45, 9],
        47 => &[13],
        48 => &[47],
        49 => &[15],
        50 => &[49],
        51 => &[48, 14, 50],
        52 => &[51],
        53 => &[18],
        54 => &[53],
        55 => &[22],
        56 => &[55],
        57 => &[20, 21, 56],
        58 => &[26],
        59 => &[58],
        60 => &[24, 25, 59],
        61 => &[54, 19, 57, 23, 60, 27],
        62 => &[61, 28, 29],
        63 => &[62],
        64 => &[63],
        65 => &[17, 64, 30],
        66 => &[32],
        67 => &[66],
        68 => &[34],
        69 => &[68],
        70 => &[67, 33, 69],
        71 => &[70],
        72 => &[71],
        73 => &[31, 72, 35],
        74 => &[16, 65, 73, 36],
        75 => &[10, 11, 12, 52, 74, 37],
        76 => &[42, 46, 75],
        77 => &[76],
        78 => &[
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23,
            24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 77,
        ],
        _ => &[],
    }
}

fn task258b3m2b2b2b_node_kind(index: usize) -> &'static str {
    match index {
        47 | 49 | 66 | 68 => "source.term.variable-reference",
        55 | 58 => "source.term.numeral",
        20 | 24 => "source.term.structure.member.constructor-assignment",
        29 => "source.term.structure.member.selector",
        61 => "source.term.structure.constructor",
        62 => "source.term.structure.selector",
        51 | 70 => "source.formula.atomic.equality",
        64 => "source.statement-witness.item",
        65 => "source.statement-witness.take",
        73 => "source.statement.conclusion",
        75 => "source.statement.theorem",
        _ => "source.surface.unowned",
    }
}

#[cfg(test)]
fn task258b3m2b2b2c_node_children(index: usize) -> &'static [usize] {
    match index {
        42 => &[1],
        43 => &[3],
        44 => &[42, 2, 43],
        45 => &[44],
        46 => &[0, 45, 4],
        47 => &[8],
        48 => &[47],
        49 => &[6, 7, 48],
        50 => &[5, 49, 9],
        51 => &[13],
        52 => &[51],
        53 => &[15],
        54 => &[53],
        55 => &[52, 14, 54],
        56 => &[55],
        57 => &[18],
        58 => &[57],
        59 => &[22],
        60 => &[59],
        61 => &[20, 21, 60],
        62 => &[26],
        63 => &[62],
        64 => &[24, 25, 63],
        65 => &[58, 19, 61, 23, 64, 27],
        66 => &[32],
        67 => &[66],
        68 => &[30, 31, 67],
        69 => &[65, 28, 29, 68, 33],
        70 => &[69],
        71 => &[70],
        72 => &[17, 71, 34],
        73 => &[36],
        74 => &[73],
        75 => &[38],
        76 => &[75],
        77 => &[74, 37, 76],
        78 => &[77],
        79 => &[78],
        80 => &[35, 79, 39],
        81 => &[16, 72, 80, 40],
        82 => &[10, 11, 12, 56, 81, 41],
        83 => &[46, 50, 82],
        84 => &[83],
        85 => &[
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23,
            24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 84,
        ],
        _ => &[],
    }
}

#[cfg(test)]
fn task258b3m2b2b2c_node_kind(index: usize) -> &'static str {
    match index {
        51 | 53 | 73 | 75 => "source.term.variable-reference",
        59 | 62 | 66 => "source.term.numeral",
        20 | 24 => "source.term.structure.member.constructor-assignment",
        30 => "source.term.structure.member.update-path-segment",
        65 => "source.term.structure.constructor",
        68 => "source.term.structure.field-update",
        69 => "source.term.structure.update",
        55 | 77 => "source.formula.atomic.equality",
        71 => "source.statement-witness.item",
        72 => "source.statement-witness.take",
        80 => "source.statement.conclusion",
        82 => "source.statement.theorem",
        _ => "source.surface.unowned",
    }
}

#[cfg(test)]
const TASK258B3M2B2B3A_NODE_RANGES: [(usize, usize); 57] = [
    (0, 7),
    (8, 9),
    (10, 13),
    (14, 17),
    (17, 18),
    (19, 26),
    (27, 69),
    (69, 70),
    (71, 72),
    (73, 74),
    (75, 76),
    (77, 82),
    (85, 89),
    (90, 91),
    (91, 92),
    (92, 93),
    (94, 95),
    (95, 96),
    (96, 97),
    (100, 104),
    (105, 106),
    (107, 108),
    (109, 110),
    (110, 111),
    (112, 115),
    (115, 116),
    (14, 17),
    (14, 17),
    (8, 17),
    (0, 18),
    (71, 72),
    (71, 72),
    (75, 76),
    (75, 76),
    (71, 76),
    (71, 76),
    (91, 92),
    (91, 92),
    (94, 95),
    (94, 95),
    (90, 96),
    (90, 96),
    (90, 96),
    (85, 97),
    (105, 106),
    (105, 106),
    (109, 110),
    (109, 110),
    (105, 110),
    (105, 110),
    (105, 110),
    (100, 111),
    (77, 115),
    (19, 116),
    (0, 116),
    (0, 116),
    (0, 116),
];

#[cfg(test)]
fn task258b3m2b2b3a_node_children(index: usize) -> &'static [usize] {
    match index {
        26 => &[3],
        27 => &[26],
        28 => &[1, 2, 27],
        29 => &[0, 28, 4],
        30 => &[8],
        31 => &[30],
        32 => &[10],
        33 => &[32],
        34 => &[31, 9, 33],
        35 => &[34],
        36 => &[14],
        37 => &[36],
        38 => &[16],
        39 => &[38],
        40 => &[13, 37, 15, 39, 17],
        41 => &[40],
        42 => &[41],
        43 => &[12, 42, 18],
        44 => &[20],
        45 => &[44],
        46 => &[22],
        47 => &[46],
        48 => &[45, 21, 47],
        49 => &[48],
        50 => &[49],
        51 => &[19, 50, 23],
        52 => &[11, 43, 51, 24],
        53 => &[5, 6, 7, 35, 52, 25],
        54 => &[29, 53],
        55 => &[54],
        56 => &[
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23,
            24, 25, 55,
        ],
        _ => &[],
    }
}

#[cfg(test)]
fn task258b3m2b2b3a_node_kind(index: usize) -> &'static str {
    match index {
        30 | 32 | 44 | 46 => "source.term.variable-reference",
        36 | 38 => "source.term.numeral",
        40 => "source.term.set.enumeration",
        34 | 48 => "source.formula.atomic.equality",
        42 => "source.statement-witness.item",
        43 => "source.statement-witness.take",
        51 => "source.statement.conclusion",
        53 => "source.statement.theorem",
        _ => "source.surface.unowned",
    }
}

#[cfg(test)]
const TASK258B3M2B2B3B_NODE_RANGES: [(usize, usize); 50] = [
    (0, 7),
    (8, 9),
    (10, 13),
    (14, 17),
    (17, 18),
    (19, 26),
    (27, 74),
    (74, 75),
    (76, 77),
    (78, 79),
    (80, 81),
    (82, 87),
    (90, 94),
    (95, 96),
    (96, 97),
    (97, 98),
    (101, 105),
    (106, 107),
    (108, 109),
    (110, 111),
    (111, 112),
    (113, 116),
    (116, 117),
    (14, 17),
    (14, 17),
    (8, 17),
    (0, 18),
    (76, 77),
    (76, 77),
    (80, 81),
    (80, 81),
    (76, 81),
    (76, 81),
    (95, 97),
    (95, 97),
    (95, 97),
    (90, 98),
    (106, 107),
    (106, 107),
    (110, 111),
    (110, 111),
    (106, 111),
    (106, 111),
    (106, 111),
    (101, 112),
    (82, 116),
    (19, 117),
    (0, 117),
    (0, 117),
    (0, 117),
];

#[cfg(test)]
fn task258b3m2b2b3b_node_children(index: usize) -> &'static [usize] {
    match index {
        23 => &[3],
        24 => &[23],
        25 => &[1, 2, 24],
        26 => &[0, 25, 4],
        27 => &[8],
        28 => &[27],
        29 => &[10],
        30 => &[29],
        31 => &[28, 9, 30],
        32 => &[31],
        33 => &[13, 14],
        34 => &[33],
        35 => &[34],
        36 => &[12, 35, 15],
        37 => &[17],
        38 => &[37],
        39 => &[19],
        40 => &[39],
        41 => &[38, 18, 40],
        42 => &[41],
        43 => &[42],
        44 => &[16, 43, 20],
        45 => &[11, 36, 44, 21],
        46 => &[5, 6, 7, 32, 45, 22],
        47 => &[26, 46],
        48 => &[47],
        49 => &[
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 48,
        ],
        _ => &[],
    }
}

#[cfg(test)]
fn task258b3m2b2b3b_node_kind(index: usize) -> &'static str {
    match index {
        27 | 29 | 37 | 39 => "source.term.variable-reference",
        33 => "source.term.set.enumeration",
        31 | 41 => "source.formula.atomic.equality",
        35 => "source.statement-witness.item",
        36 => "source.statement-witness.take",
        44 => "source.statement.conclusion",
        46 => "source.statement.theorem",
        _ => "source.surface.unowned",
    }
}

#[cfg(test)]
const TASK258B3M2B2B3C_NODE_RANGES: [(usize, usize); 52] = [
    (0, 7),
    (8, 9),
    (10, 13),
    (14, 17),
    (17, 18),
    (19, 26),
    (27, 61),
    (61, 62),
    (63, 64),
    (65, 66),
    (67, 68),
    (69, 74),
    (77, 81),
    (82, 85),
    (86, 89),
    (89, 90),
    (93, 97),
    (98, 99),
    (100, 101),
    (102, 103),
    (103, 104),
    (105, 108),
    (108, 109),
    (14, 17),
    (14, 17),
    (8, 17),
    (0, 18),
    (63, 64),
    (63, 64),
    (67, 68),
    (67, 68),
    (63, 68),
    (63, 68),
    (86, 89),
    (86, 89),
    (82, 89),
    (82, 89),
    (82, 89),
    (77, 90),
    (98, 99),
    (98, 99),
    (102, 103),
    (102, 103),
    (98, 103),
    (98, 103),
    (98, 103),
    (93, 104),
    (69, 108),
    (19, 109),
    (0, 109),
    (0, 109),
    (0, 109),
];

#[cfg(test)]
fn task258b3m2b2b3c_node_children(index: usize) -> &'static [usize] {
    match index {
        23 => &[3],
        24 => &[23],
        25 => &[1, 2, 24],
        26 => &[0, 25, 4],
        27 => &[8],
        28 => &[27],
        29 => &[10],
        30 => &[29],
        31 => &[28, 9, 30],
        32 => &[31],
        33 => &[14],
        34 => &[33],
        35 => &[13, 34],
        36 => &[35],
        37 => &[36],
        38 => &[12, 37, 15],
        39 => &[17],
        40 => &[39],
        41 => &[19],
        42 => &[41],
        43 => &[40, 18, 42],
        44 => &[43],
        45 => &[44],
        46 => &[16, 45, 20],
        47 => &[11, 38, 46, 21],
        48 => &[5, 6, 7, 32, 47, 22],
        49 => &[26, 48],
        50 => &[49],
        51 => &[
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 50,
        ],
        _ => &[],
    }
}

#[cfg(test)]
fn task258b3m2b2b3c_node_kind(index: usize) -> &'static str {
    match index {
        27 | 29 | 39 | 41 => "source.term.variable-reference",
        33 => "source.term.set.target-type-head",
        34 => "source.term.set.target-type",
        35 => "source.term.set.choice",
        31 | 43 => "source.formula.atomic.equality",
        37 => "source.statement-witness.item",
        38 => "source.statement-witness.take",
        46 => "source.statement.conclusion",
        48 => "source.statement.theorem",
        _ => "source.surface.unowned",
    }
}

#[cfg(test)]
const TASK258B3M2B2B3D_NODE_RANGES: [(usize, usize); 54] = [
    (0, 7),
    (8, 9),
    (10, 13),
    (14, 17),
    (17, 18),
    (19, 26),
    (27, 58),
    (58, 59),
    (60, 61),
    (62, 63),
    (64, 65),
    (66, 71),
    (74, 78),
    (79, 80),
    (81, 84),
    (85, 88),
    (88, 89),
    (92, 96),
    (97, 98),
    (99, 100),
    (101, 102),
    (102, 103),
    (104, 107),
    (107, 108),
    (14, 17),
    (14, 17),
    (8, 17),
    (0, 18),
    (60, 61),
    (60, 61),
    (64, 65),
    (64, 65),
    (60, 65),
    (60, 65),
    (79, 80),
    (85, 88),
    (85, 88),
    (79, 88),
    (79, 88),
    (79, 88),
    (74, 89),
    (97, 98),
    (97, 98),
    (101, 102),
    (101, 102),
    (97, 102),
    (97, 102),
    (97, 102),
    (92, 103),
    (66, 107),
    (19, 108),
    (0, 108),
    (0, 108),
    (0, 108),
];

#[cfg(test)]
fn task258b3m2b2b3d_node_children(index: usize) -> &'static [usize] {
    match index {
        24 => &[3],
        25 => &[24],
        26 => &[1, 2, 25],
        27 => &[0, 26, 4],
        28 => &[8],
        29 => &[28],
        30 => &[10],
        31 => &[30],
        32 => &[29, 9, 31],
        33 => &[32],
        34 => &[13],
        35 => &[15],
        36 => &[35],
        37 => &[34, 14, 36],
        38 => &[37],
        39 => &[38],
        40 => &[12, 39, 16],
        41 => &[18],
        42 => &[41],
        43 => &[20],
        44 => &[43],
        45 => &[42, 19, 44],
        46 => &[45],
        47 => &[46],
        48 => &[17, 47, 21],
        49 => &[11, 40, 48, 22],
        50 => &[5, 6, 7, 33, 49, 23],
        51 => &[27, 50],
        52 => &[51],
        53 => &[
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23,
            52,
        ],
        _ => &[],
    }
}

#[cfg(test)]
fn task258b3m2b2b3d_node_kind(index: usize) -> &'static str {
    match index {
        28 | 30 | 41 | 43 => "source.term.variable-reference",
        34 => "source.term.numeral",
        35 => "source.term.set.target-type-head",
        36 => "source.term.set.target-type",
        37 => "source.term.set.qua",
        32 | 45 => "source.formula.atomic.equality",
        39 => "source.statement-witness.item",
        40 => "source.statement-witness.take",
        48 => "source.statement.conclusion",
        50 => "source.statement.theorem",
        _ => "source.surface.unowned",
    }
}

#[cfg(test)]
const TASK258B3M2B2B3E_NODE_RANGES: [(usize, usize); 60] = [
    (0, 7),
    (8, 9),
    (10, 13),
    (14, 17),
    (17, 18),
    (19, 26),
    (27, 68),
    (68, 69),
    (70, 71),
    (72, 73),
    (74, 75),
    (76, 81),
    (84, 88),
    (89, 90),
    (90, 91),
    (92, 97),
    (98, 110),
    (111, 113),
    (114, 117),
    (117, 118),
    (118, 119),
    (122, 126),
    (127, 128),
    (129, 130),
    (131, 132),
    (132, 133),
    (134, 137),
    (137, 138),
    (14, 17),
    (14, 17),
    (8, 17),
    (0, 18),
    (70, 71),
    (70, 71),
    (74, 75),
    (74, 75),
    (70, 75),
    (70, 75),
    (90, 91),
    (90, 91),
    (114, 117),
    (114, 117),
    (98, 117),
    (89, 118),
    (89, 118),
    (89, 118),
    (84, 119),
    (127, 128),
    (127, 128),
    (131, 132),
    (131, 132),
    (127, 132),
    (127, 132),
    (127, 132),
    (122, 133),
    (76, 137),
    (19, 138),
    (0, 138),
    (0, 138),
    (0, 138),
];

#[cfg(test)]
fn task258b3m2b2b3e_node_children(index: usize) -> &'static [usize] {
    match index {
        28 => &[3],
        29 => &[28],
        30 => &[1, 2, 29],
        31 => &[0, 30, 4],
        32 => &[8],
        33 => &[32],
        34 => &[10],
        35 => &[34],
        36 => &[33, 9, 35],
        37 => &[36],
        38 => &[14],
        39 => &[38],
        40 => &[18],
        41 => &[40],
        42 => &[16, 17, 41],
        43 => &[13, 39, 15, 42, 19],
        44 => &[43],
        45 => &[44],
        46 => &[12, 45, 20],
        47 => &[22],
        48 => &[47],
        49 => &[24],
        50 => &[49],
        51 => &[48, 23, 50],
        52 => &[51],
        53 => &[52],
        54 => &[21, 53, 25],
        55 => &[11, 46, 54, 26],
        56 => &[5, 6, 7, 37, 55, 27],
        57 => &[31, 56],
        58 => &[57],
        59 => &[
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23,
            24, 25, 26, 27, 58,
        ],
        _ => &[],
    }
}

#[cfg(test)]
fn task258b3m2b2b3e_node_kind(index: usize) -> &'static str {
    match index {
        32 | 34 | 47 | 49 => "source.term.variable-reference",
        38 => "source.term.numeral",
        16 => "source.term.set.comprehension-generator",
        40 => "source.term.set.target-type-head",
        41 => "source.term.set.target-type",
        43 => "source.term.set.comprehension",
        36 | 51 => "source.formula.atomic.equality",
        45 => "source.statement-witness.item",
        46 => "source.statement-witness.take",
        54 => "source.statement.conclusion",
        56 => "source.statement.theorem",
        _ => "source.surface.unowned",
    }
}

#[cfg(test)]
fn exact_task258b3_shared_arena(source_id: SourceId, arena: &TypedArena) -> bool {
    if arena.len() != TASK258B3_NODE_RANGES.len()
        || arena.root() != Some(crate::typed_ast::TypedNodeId::new(48))
    {
        return false;
    }
    TASK258B3_NODE_RANGES
        .iter()
        .copied()
        .enumerate()
        .all(|(index, (start, end))| {
            arena
                .node(crate::typed_ast::TypedNodeId::new(index))
                .is_some_and(|node| {
                    node.anchor == SourceAnchor::Range(range(source_id, start, end))
                        && node.kind.as_str() == task258b3_node_kind(index)
                        && node.recovery == NodeRecoveryState::Normal
                        && node
                            .children
                            .iter()
                            .map(|child| child.index())
                            .eq(task258b3_node_children(index).iter().copied())
                })
        })
}

#[cfg(test)]
fn exact_task258b3n_shared_arena(source_id: SourceId, arena: &TypedArena) -> bool {
    if arena.len() != TASK258B3N_NODE_RANGES.len()
        || arena.root() != Some(crate::typed_ast::TypedNodeId::new(50))
    {
        return false;
    }
    TASK258B3N_NODE_RANGES
        .iter()
        .copied()
        .enumerate()
        .all(|(index, (start, end))| {
            arena
                .node(crate::typed_ast::TypedNodeId::new(index))
                .is_some_and(|node| {
                    node.anchor == SourceAnchor::Range(range(source_id, start, end))
                        && node.kind.as_str() == task258b3n_node_kind(index)
                        && node.recovery == NodeRecoveryState::Normal
                        && node
                            .children
                            .iter()
                            .map(|child| child.index())
                            .eq(task258b3n_node_children(index).iter().copied())
                })
        })
}

#[cfg(test)]
fn exact_task258b3m1_shared_arena(source_id: SourceId, arena: &TypedArena) -> bool {
    if arena.len() != TASK258B3M1_NODE_RANGES.len()
        || arena.root() != Some(crate::typed_ast::TypedNodeId::new(55))
    {
        return false;
    }
    TASK258B3M1_NODE_RANGES
        .iter()
        .copied()
        .enumerate()
        .all(|(index, (start, end))| {
            arena
                .node(crate::typed_ast::TypedNodeId::new(index))
                .is_some_and(|node| {
                    node.anchor == SourceAnchor::Range(range(source_id, start, end))
                        && node.kind.as_str() == task258b3m1_node_kind(index)
                        && node.recovery == NodeRecoveryState::Normal
                        && node
                            .children
                            .iter()
                            .map(|child| child.index())
                            .eq(task258b3m1_node_children(index).iter().copied())
                })
        })
}

#[cfg(test)]
fn exact_task258b3m2a_shared_arena(source_id: SourceId, arena: &TypedArena) -> bool {
    if arena.len() != TASK258B3M2A_NODE_RANGES.len()
        || arena.root() != Some(crate::typed_ast::TypedNodeId::new(48))
    {
        return false;
    }
    TASK258B3M2A_NODE_RANGES
        .iter()
        .copied()
        .enumerate()
        .all(|(index, (start, end))| {
            arena
                .node(crate::typed_ast::TypedNodeId::new(index))
                .is_some_and(|node| {
                    node.anchor == SourceAnchor::Range(range(source_id, start, end))
                        && node.kind.as_str() == task258b3m2a_node_kind(index)
                        && node.recovery == NodeRecoveryState::Normal
                        && node
                            .children
                            .iter()
                            .map(|child| child.index())
                            .eq(task258b3m2a_node_children(index).iter().copied())
                })
        })
}

#[cfg(test)]
fn exact_task258b3m2b1_shared_arena(source_id: SourceId, arena: &TypedArena) -> bool {
    if arena.len() != TASK258B3M2B1_NODE_RANGES.len()
        || arena.root() != Some(crate::typed_ast::TypedNodeId::new(52))
    {
        return false;
    }
    TASK258B3M2B1_NODE_RANGES
        .iter()
        .copied()
        .enumerate()
        .all(|(index, (start, end))| {
            arena
                .node(crate::typed_ast::TypedNodeId::new(index))
                .is_some_and(|node| {
                    node.anchor == SourceAnchor::Range(range(source_id, start, end))
                        && node.kind.as_str() == task258b3m2b1_node_kind(index)
                        && node.recovery == NodeRecoveryState::Normal
                        && node
                            .children
                            .iter()
                            .map(|child| child.index())
                            .eq(task258b3m2b1_node_children(index).iter().copied())
                })
        })
}

#[cfg(test)]
fn exact_task258b3m2b2a_shared_arena(source_id: SourceId, arena: &TypedArena) -> bool {
    if arena.len() != TASK258B3M2B2A_NODE_RANGES.len()
        || arena.root() != Some(crate::typed_ast::TypedNodeId::new(56))
    {
        return false;
    }
    TASK258B3M2B2A_NODE_RANGES
        .iter()
        .copied()
        .enumerate()
        .all(|(index, (start, end))| {
            arena
                .node(crate::typed_ast::TypedNodeId::new(index))
                .is_some_and(|node| {
                    node.anchor == SourceAnchor::Range(range(source_id, start, end))
                        && node.kind.as_str() == task258b3m2b2a_node_kind(index)
                        && node.recovery == NodeRecoveryState::Normal
                        && node
                            .children
                            .iter()
                            .map(|child| child.index())
                            .eq(task258b3m2b2a_node_children(index).iter().copied())
                })
        })
}

#[cfg(test)]
fn exact_task258b3m2b2b1a_shared_arena(source_id: SourceId, arena: &TypedArena) -> bool {
    if arena.len() != TASK258B3M2B2B1A_NODE_RANGES.len()
        || arena.root() != Some(crate::typed_ast::TypedNodeId::new(62))
    {
        return false;
    }
    TASK258B3M2B2B1A_NODE_RANGES
        .iter()
        .copied()
        .enumerate()
        .all(|(index, (start, end))| {
            arena
                .node(crate::typed_ast::TypedNodeId::new(index))
                .is_some_and(|node| {
                    node.anchor == SourceAnchor::Range(range(source_id, start, end))
                        && node.kind.as_str() == task258b3m2b2b1a_node_kind(index)
                        && node.recovery == NodeRecoveryState::Normal
                        && node
                            .children
                            .iter()
                            .map(|child| child.index())
                            .eq(task258b3m2b2b1a_node_children(index).iter().copied())
                })
        })
}

#[cfg(test)]
fn exact_task258b3m2b2b1b1_shared_arena(source_id: SourceId, arena: &TypedArena) -> bool {
    if arena.len() != TASK258B3M2B2B1B1_NODE_RANGES.len()
        || arena.root() != Some(crate::typed_ast::TypedNodeId::new(66))
    {
        return false;
    }
    TASK258B3M2B2B1B1_NODE_RANGES
        .iter()
        .copied()
        .enumerate()
        .all(|(index, (start, end))| {
            arena
                .node(crate::typed_ast::TypedNodeId::new(index))
                .is_some_and(|node| {
                    node.anchor == SourceAnchor::Range(range(source_id, start, end))
                        && node.kind.as_str() == task258b3m2b2b1b1_node_kind(index)
                        && node.recovery == NodeRecoveryState::Normal
                        && node
                            .children
                            .iter()
                            .map(|child| child.index())
                            .eq(task258b3m2b2b1b1_node_children(index).iter().copied())
                })
        })
}

fn exact_task258b3m2b2b2a_shared_arena(source_id: SourceId, arena: &TypedArena) -> bool {
    if arena.len() != TASK258B3M2B2B2A_NODE_RANGES.len()
        || arena.root() != Some(crate::typed_ast::TypedNodeId::new(75))
    {
        return false;
    }
    TASK258B3M2B2B2A_NODE_RANGES
        .iter()
        .copied()
        .enumerate()
        .all(|(index, (start, end))| {
            arena
                .node(crate::typed_ast::TypedNodeId::new(index))
                .is_some_and(|node| {
                    node.anchor == SourceAnchor::Range(range(source_id, start, end))
                        && node.kind.as_str() == task258b3m2b2b2a_node_kind(index)
                        && node.resolved_node.is_none()
                        && node.typing == TypingState::Unknown
                        && node.recovery == NodeRecoveryState::Normal
                        && node.links.context.is_none()
                        && node.links.type_entry.is_none()
                        && node.links.facts.is_empty()
                        && node.links.coercions.is_empty()
                        && node.links.initial_obligations.is_empty()
                        && node.links.diagnostics.is_empty()
                        && node
                            .children
                            .iter()
                            .map(|child| child.index())
                            .eq(task258b3m2b2b2a_node_children(index).iter().copied())
                })
        })
}

fn exact_task258b3m2b2b2b_shared_arena(source_id: SourceId, arena: &TypedArena) -> bool {
    if arena.len() != TASK258B3M2B2B2B_NODE_RANGES.len()
        || arena.root() != Some(crate::typed_ast::TypedNodeId::new(78))
    {
        return false;
    }
    TASK258B3M2B2B2B_NODE_RANGES
        .iter()
        .copied()
        .enumerate()
        .all(|(index, (start, end))| {
            arena
                .node(crate::typed_ast::TypedNodeId::new(index))
                .is_some_and(|node| {
                    node.anchor == SourceAnchor::Range(range(source_id, start, end))
                        && node.kind.as_str() == task258b3m2b2b2b_node_kind(index)
                        && node.resolved_node.is_none()
                        && node.typing == TypingState::Unknown
                        && node.recovery == NodeRecoveryState::Normal
                        && node.links.context.is_none()
                        && node.links.type_entry.is_none()
                        && node.links.facts.is_empty()
                        && node.links.coercions.is_empty()
                        && node.links.initial_obligations.is_empty()
                        && node.links.diagnostics.is_empty()
                        && node
                            .children
                            .iter()
                            .map(|child| child.index())
                            .eq(task258b3m2b2b2b_node_children(index).iter().copied())
                })
        })
}

#[cfg(test)]
fn exact_task258b3m2b2b2c_shared_arena(source_id: SourceId, arena: &TypedArena) -> bool {
    if arena.len() != TASK258B3M2B2B2C_NODE_RANGES.len()
        || arena.root() != Some(crate::typed_ast::TypedNodeId::new(85))
    {
        return false;
    }
    TASK258B3M2B2B2C_NODE_RANGES
        .iter()
        .copied()
        .enumerate()
        .all(|(index, (start, end))| {
            arena
                .node(crate::typed_ast::TypedNodeId::new(index))
                .is_some_and(|node| {
                    node.anchor == SourceAnchor::Range(range(source_id, start, end))
                        && node.kind.as_str() == task258b3m2b2b2c_node_kind(index)
                        && node.resolved_node.is_none()
                        && node.typing == TypingState::Unknown
                        && node.recovery == NodeRecoveryState::Normal
                        && node.links.context.is_none()
                        && node.links.type_entry.is_none()
                        && node.links.facts.is_empty()
                        && node.links.coercions.is_empty()
                        && node.links.initial_obligations.is_empty()
                        && node.links.diagnostics.is_empty()
                        && node
                            .children
                            .iter()
                            .map(|child| child.index())
                            .eq(task258b3m2b2b2c_node_children(index).iter().copied())
                })
        })
}

#[cfg(test)]
fn exact_task258b3m2b2b3a_shared_arena(source_id: SourceId, arena: &TypedArena) -> bool {
    if arena.len() != TASK258B3M2B2B3A_NODE_RANGES.len()
        || arena.root() != Some(crate::typed_ast::TypedNodeId::new(56))
    {
        return false;
    }
    TASK258B3M2B2B3A_NODE_RANGES
        .iter()
        .copied()
        .enumerate()
        .all(|(index, (start, end))| {
            arena
                .node(crate::typed_ast::TypedNodeId::new(index))
                .is_some_and(|node| {
                    node.anchor == SourceAnchor::Range(range(source_id, start, end))
                        && node.kind.as_str() == task258b3m2b2b3a_node_kind(index)
                        && node.resolved_node.is_none()
                        && node.typing == TypingState::Unknown
                        && node.recovery == NodeRecoveryState::Normal
                        && node.links.context.is_none()
                        && node.links.type_entry.is_none()
                        && node.links.facts.is_empty()
                        && node.links.coercions.is_empty()
                        && node.links.initial_obligations.is_empty()
                        && node.links.diagnostics.is_empty()
                        && node
                            .children
                            .iter()
                            .map(|child| child.index())
                            .eq(task258b3m2b2b3a_node_children(index).iter().copied())
                })
        })
}

#[cfg(test)]
fn exact_task258b3m2b2b3b_shared_arena(source_id: SourceId, arena: &TypedArena) -> bool {
    if arena.len() != TASK258B3M2B2B3B_NODE_RANGES.len()
        || arena.root() != Some(crate::typed_ast::TypedNodeId::new(49))
    {
        return false;
    }
    TASK258B3M2B2B3B_NODE_RANGES
        .iter()
        .copied()
        .enumerate()
        .all(|(index, (start, end))| {
            arena
                .node(crate::typed_ast::TypedNodeId::new(index))
                .is_some_and(|node| {
                    node.anchor == SourceAnchor::Range(range(source_id, start, end))
                        && node.kind.as_str() == task258b3m2b2b3b_node_kind(index)
                        && node.resolved_node.is_none()
                        && node.typing == TypingState::Unknown
                        && node.recovery == NodeRecoveryState::Normal
                        && node.links.context.is_none()
                        && node.links.type_entry.is_none()
                        && node.links.facts.is_empty()
                        && node.links.coercions.is_empty()
                        && node.links.initial_obligations.is_empty()
                        && node.links.diagnostics.is_empty()
                        && node
                            .children
                            .iter()
                            .map(|child| child.index())
                            .eq(task258b3m2b2b3b_node_children(index).iter().copied())
                })
        })
}

#[cfg(test)]
fn exact_task258b3m2b2b3c_shared_arena(source_id: SourceId, arena: &TypedArena) -> bool {
    if arena.len() != TASK258B3M2B2B3C_NODE_RANGES.len()
        || arena.root() != Some(crate::typed_ast::TypedNodeId::new(51))
    {
        return false;
    }
    TASK258B3M2B2B3C_NODE_RANGES
        .iter()
        .copied()
        .enumerate()
        .all(|(index, (start, end))| {
            arena
                .node(crate::typed_ast::TypedNodeId::new(index))
                .is_some_and(|node| {
                    node.anchor == SourceAnchor::Range(range(source_id, start, end))
                        && node.kind.as_str() == task258b3m2b2b3c_node_kind(index)
                        && node.resolved_node.is_none()
                        && node.typing == TypingState::Unknown
                        && node.recovery == NodeRecoveryState::Normal
                        && node.links.context.is_none()
                        && node.links.type_entry.is_none()
                        && node.links.facts.is_empty()
                        && node.links.coercions.is_empty()
                        && node.links.initial_obligations.is_empty()
                        && node.links.diagnostics.is_empty()
                        && node
                            .children
                            .iter()
                            .map(|child| child.index())
                            .eq(task258b3m2b2b3c_node_children(index).iter().copied())
                })
        })
}

#[cfg(test)]
fn exact_task258b3m2b2b3d_shared_arena(source_id: SourceId, arena: &TypedArena) -> bool {
    if arena.len() != TASK258B3M2B2B3D_NODE_RANGES.len()
        || arena.root() != Some(crate::typed_ast::TypedNodeId::new(53))
    {
        return false;
    }
    TASK258B3M2B2B3D_NODE_RANGES
        .iter()
        .copied()
        .enumerate()
        .all(|(index, (start, end))| {
            arena
                .node(crate::typed_ast::TypedNodeId::new(index))
                .is_some_and(|node| {
                    node.anchor == SourceAnchor::Range(range(source_id, start, end))
                        && node.kind.as_str() == task258b3m2b2b3d_node_kind(index)
                        && node.resolved_node.is_none()
                        && node.typing == TypingState::Unknown
                        && node.recovery == NodeRecoveryState::Normal
                        && node.links.context.is_none()
                        && node.links.type_entry.is_none()
                        && node.links.facts.is_empty()
                        && node.links.coercions.is_empty()
                        && node.links.initial_obligations.is_empty()
                        && node.links.diagnostics.is_empty()
                        && node
                            .children
                            .iter()
                            .map(|child| child.index())
                            .eq(task258b3m2b2b3d_node_children(index).iter().copied())
                })
        })
}

#[cfg(test)]
fn exact_task258b3m2b2b3e_shared_arena(source_id: SourceId, arena: &TypedArena) -> bool {
    if arena.len() != TASK258B3M2B2B3E_NODE_RANGES.len()
        || arena.root() != Some(crate::typed_ast::TypedNodeId::new(59))
    {
        return false;
    }
    TASK258B3M2B2B3E_NODE_RANGES
        .iter()
        .copied()
        .enumerate()
        .all(|(index, (start, end))| {
            arena
                .node(crate::typed_ast::TypedNodeId::new(index))
                .is_some_and(|node| {
                    node.anchor == SourceAnchor::Range(range(source_id, start, end))
                        && node.kind.as_str() == task258b3m2b2b3e_node_kind(index)
                        && node.resolved_node.is_none()
                        && node.typing == TypingState::Unknown
                        && node.recovery == NodeRecoveryState::Normal
                        && node.links.context.is_none()
                        && node.links.type_entry.is_none()
                        && node.links.facts.is_empty()
                        && node.links.coercions.is_empty()
                        && node.links.initial_obligations.is_empty()
                        && node.links.diagnostics.is_empty()
                        && node
                            .children
                            .iter()
                            .map(|child| child.index())
                            .eq(task258b3m2b2b3e_node_children(index).iter().copied())
                })
        })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WitnessProfile {
    #[cfg(test)]
    Task258B3,
    #[cfg(test)]
    Task258B3N,
    #[cfg(test)]
    Task258B3M1,
    #[cfg(test)]
    Task258B3M2A,
    #[cfg(test)]
    Task258B3M2B1,
    #[cfg(test)]
    Task258B3M2B2A,
    #[cfg(test)]
    Task258B3M2B2B1A,
    #[cfg(test)]
    Task258B3M2B2B1B1,
    Task258B3M2B2B2A,
    Task258B3M2B2B2B,
    #[cfg(test)]
    Task258B3M2B2B2C,
    #[cfg(test)]
    Task258B3M2B2B3A,
    #[cfg(test)]
    Task258B3M2B2B3B,
    #[cfg(test)]
    Task258B3M2B2B3C,
    #[cfg(test)]
    Task258B3M2B2B3D,
    #[cfg(test)]
    Task258B3M2B2B3E,
}

#[cfg(test)]
struct ExpectedWitnessRow {
    term_index: usize,
    take_site: usize,
    take_range: SourceRange,
    item_site: usize,
    item_range: SourceRange,
    item_spelling: &'static str,
    item_kind: SourceStatementWitnessKind,
    item_name: Option<SourceStatementWitnessNameId>,
    term_site: usize,
    term_range: SourceRange,
    wrapper_site: usize,
    take_children: &'static [usize],
    item_children: &'static [usize],
}

#[cfg(test)]
fn exact_task258b3m2b2b1a_application_profile(
    source_id: SourceId,
    module_id: &ModuleId,
    primary_terms: &SourcePrimaryTermHandoff,
    application: &SourceFunctorApplicationHandoff,
) -> bool {
    let imported_module = ModuleId::new(
        module_id.package().clone(),
        ModulePath::new("parser.type_fixtures"),
    );
    application
        .validate_installation(source_id, module_id, primary_terms)
        .is_ok()
        && application.source_id() == source_id
        && application.module_id() == module_id
        && application.primary_term_fingerprint() == primary_terms.debug_text()
        && application.applications().len() == 1
        && application.wrappers().is_empty()
        && application.candidates().len() == 1
        && application.arguments().len() == 2
        && application.type_requests().len() == 2
        && application
            .applications()
            .get(SourceFunctorApplicationId::new(0))
            .is_some_and(|row| {
                row.site().node().index() == 46
                    && row.source_range() == range(source_id, 116, 122)
                    && row.source_ordinal() == 0
                    && row.context() == BindingContextId::new(1)
                    && row.recovery()
                        == crate::source_application::SourceFunctorApplicationRecovery::Normal
                    && row.spelling() == "1 ++ 2"
                    && row.kind()
                        == crate::source_application::SourceFunctorApplicationKind::Symbolic
                    && row.form()
                        == crate::source_application::SourceFunctorApplicationForm::Infix
                    && row.head_ordinal() == 1
                    && matches!(
                        row.head(),
                        crate::source_application::SourceFunctorHeadSite::Single {
                            site,
                            source_range,
                            spelling,
                        } if site.node().index() == 19
                            && *source_range == range(source_id, 118, 120)
                            && spelling == "++"
                    )
            })
        && application
            .candidates()
            .get(crate::source_application::SourceFunctorCandidateId::new(0))
            .is_some_and(|row| {
                row.application() == SourceFunctorApplicationId::new(0)
                    && row.ordinal() == 0
                    && row.symbol().module() == &imported_module
                    && row.symbol().local().as_str()
                        == "summary:parser.type_fixtures#parse-only#++:12"
                    && row.symbol().fqn().as_str() == "parser.type_fixtures::++#12"
                    && row.contribution().index() == 2
                    && row.origin().source_id() == source_id
                    && row.origin().module_id() == &imported_module
                    && row.origin().anchor()
                        == &SourceAnchor::Range(range(source_id, 7, 27))
                    && row.origin().structural_path() == [12]
                    && row.origin().import_edge().is_none()
                    && !row.origin().is_recovered()
                    && row.visibility() == Visibility::Public
                    && row.export_status() == ExportStatus::Exported
                    && row.signature().is_none()
            })
        && application.arguments().iter().enumerate().all(|(index, (_, row))| {
            row.application() == SourceFunctorApplicationId::new(0)
                && row.ordinal() == index
                && row.target()
                    == crate::source_application::SourceFunctorArgumentTarget::Primary(
                        SourcePrimaryTermId::new(index + 2),
                    )
        })
        && application
            .type_requests()
            .iter()
            .enumerate()
            .all(|(index, (_, row))| {
                row.application() == SourceFunctorApplicationId::new(0)
                    && row.request_ordinal() == index
                    && row.candidate()
                        == if index == 0 {
                            Some(crate::source_application::SourceFunctorCandidateId::new(0))
                        } else {
                            None
                        }
                    && row.kind()
                        == if index == 0 {
                            crate::source_application::SourceFunctorTypeRequestKind::CandidateSignature
                        } else {
                            crate::source_application::SourceFunctorTypeRequestKind::ApplicationResultType
                        }
            })
}

#[cfg(test)]
fn exact_task258b3m2b2b1b1_application_profile(
    source_id: SourceId,
    module_id: &ModuleId,
    primary_terms: &SourcePrimaryTermHandoff,
    application: &SourceFunctorApplicationHandoff,
) -> bool {
    let imported_module = ModuleId::new(
        module_id.package().clone(),
        ModulePath::new("parser.type_fixtures"),
    );
    application
        .validate_installation(source_id, module_id, primary_terms)
        .is_ok()
        && application.source_id() == source_id
        && application.module_id() == module_id
        && application.primary_term_fingerprint() == primary_terms.debug_text()
        && application.applications().len() == 1
        && application.wrappers().len() == 1
        && application.candidates().len() == 1
        && application.arguments().len() == 2
        && application.type_requests().len() == 2
        && application
            .applications()
            .get(SourceFunctorApplicationId::new(0))
            .is_some_and(|row| {
                row.site().node().index() == 48
                    && row.source_range() == range(source_id, 130, 136)
                    && row.source_ordinal() == 0
                    && row.context() == BindingContextId::new(1)
                    && row.recovery()
                        == crate::source_application::SourceFunctorApplicationRecovery::Normal
                    && row.spelling() == "1 ++ 2"
                    && row.kind()
                        == crate::source_application::SourceFunctorApplicationKind::Symbolic
                    && row.form()
                        == crate::source_application::SourceFunctorApplicationForm::Infix
                    && row.head_ordinal() == 1
                    && matches!(
                        row.head(),
                        crate::source_application::SourceFunctorHeadSite::Single {
                            site,
                            source_range,
                            spelling,
                        } if site.node().index() == 20
                            && *source_range == range(source_id, 132, 134)
                            && spelling == "++"
                    )
            })
        && application
            .wrappers()
            .get(crate::source_application::SourceFunctorWrapperId::new(0))
            .is_some_and(|row| {
                row.application() == SourceFunctorApplicationId::new(0)
                    && row.ordinal() == 0
                    && row.site().node().index() == 50
                    && row.source_range() == range(source_id, 129, 137)
                    && row.context() == BindingContextId::new(1)
                    && row.spelling() == "( 1 ++ 2 )"
                    && row.recovery()
                        == crate::source_application::SourceFunctorApplicationRecovery::Normal
            })
        && application
            .candidates()
            .get(crate::source_application::SourceFunctorCandidateId::new(0))
            .is_some_and(|row| {
                row.application() == SourceFunctorApplicationId::new(0)
                    && row.ordinal() == 0
                    && row.symbol().module() == &imported_module
                    && row.symbol().local().as_str()
                        == "summary:parser.type_fixtures#parse-only#++:12"
                    && row.symbol().fqn().as_str() == "parser.type_fixtures::++#12"
                    && row.contribution().index() == 2
                    && row.origin().source_id() == source_id
                    && row.origin().module_id() == &imported_module
                    && row.origin().anchor()
                        == &SourceAnchor::Range(range(source_id, 7, 27))
                    && row.origin().structural_path() == [12]
                    && row.origin().import_edge().is_none()
                    && !row.origin().is_recovered()
                    && row.visibility() == Visibility::Public
                    && row.export_status() == ExportStatus::Exported
                    && row.signature().is_none()
            })
        && application.arguments().iter().enumerate().all(|(index, (_, row))| {
            row.application() == SourceFunctorApplicationId::new(0)
                && row.ordinal() == index
                && row.target()
                    == crate::source_application::SourceFunctorArgumentTarget::Primary(
                        SourcePrimaryTermId::new(index + 2),
                    )
        })
        && application
            .type_requests()
            .iter()
            .enumerate()
            .all(|(index, (_, row))| {
                row.application() == SourceFunctorApplicationId::new(0)
                    && row.request_ordinal() == index
                    && row.candidate()
                        == if index == 0 {
                            Some(crate::source_application::SourceFunctorCandidateId::new(0))
                        } else {
                            None
                        }
                    && row.kind()
                        == if index == 0 {
                            crate::source_application::SourceFunctorTypeRequestKind::CandidateSignature
                        } else {
                            crate::source_application::SourceFunctorTypeRequestKind::ApplicationResultType
                        }
            })
}

fn exact_task258b3m2b2b2a_structure_profile(
    source_id: SourceId,
    module_id: &ModuleId,
    primary_terms: &SourcePrimaryTermHandoff,
    structure: &SourceStructureHandoff,
    arena: &TypedArena,
) -> bool {
    let imported_module = ModuleId::new(
        module_id.package().clone(),
        ModulePath::new("parser.type_fixtures"),
    );
    let Some(term) = structure.terms().get(SourceStructureTermId::new(0)) else {
        return false;
    };
    let Some(root) = structure.roots().get(SourceStructureRootId::new(0)) else {
        return false;
    };
    structure
        .validate_installation(source_id, module_id, primary_terms, None, arena)
        .is_ok()
        && structure.source_id() == source_id
        && structure.module_id() == module_id
        && structure.primary_term_fingerprint() == primary_terms.debug_text()
        && structure.application_fingerprint().is_none()
        && structure.terms().len() == 1
        && structure.wrappers().is_empty()
        && structure.roots().len() == 1
        && structure.members().len() == 2
        && structure.field_updates().is_empty()
        && structure.edges().len() == 2
        && structure.requests().len() == 6
        && term.site().node().index() == 59
        && term.source_range() == range(source_id, 125, 151)
        && term.source_ordinal() == 0
        && term.context() == BindingContextId::new(1)
        && term.recovery() == SourceStructureRecovery::Normal
        && term.spelling() == "TypeCaseStruct ( x : 1 , y : 2 )"
        && term.kind() == SourceStructureTermKind::Constructor
        && root.term() == SourceStructureTermId::new(0)
        && root.symbol().module() == &imported_module
        && root.symbol().local().as_str()
            == "summary:parser.type_fixtures#parse-only#TypeCaseStruct:5"
        && root.symbol().fqn().as_str() == "parser.type_fixtures::TypeCaseStruct#5"
        && root.contribution().index() == 2
        && root.origin().source_id() == source_id
        && root.origin().module_id() == &imported_module
        && root.origin().anchor() == &SourceAnchor::Range(range(source_id, 7, 27))
        && root.origin().structural_path() == [5]
        && root.origin().import_edge().is_none()
        && !root.origin().is_recovered()
        && root.visibility() == Visibility::Public
        && root.export_status() == ExportStatus::Exported
        && root.signature().is_none()
        && structure
            .members()
            .iter()
            .enumerate()
            .all(|(index, (_, member))| {
                let (site, start, end, spelling) = if index == 0 {
                    (20, 140, 141, "x")
                } else {
                    (24, 146, 147, "y")
                };
                member.term() == SourceStructureTermId::new(0)
                    && member.ordinal() == index
                    && member.site().node().index() == site
                    && member.source_range() == range(source_id, start, end)
                    && member.spelling() == spelling
                    && member.role() == SourceStructureMemberRole::ConstructorAssignment
                    && member.parent().is_none()
            })
        && structure
            .edges()
            .iter()
            .enumerate()
            .all(|(index, (_, edge))| {
                edge.term() == SourceStructureTermId::new(0)
                    && edge.ordinal() == index
                    && edge.role() == SourceStructureEdgeRole::ConstructorValue
                    && edge.member() == Some(SourceStructureMemberId::new(index))
                    && edge.target()
                        == SourceStructureTarget::Primary(SourcePrimaryTermId::new(index + 2))
            })
        && structure
            .requests()
            .iter()
            .enumerate()
            .all(|(index, (_, request))| {
                let (member, kind) = match index {
                    0 => (None, SourceStructureRequestKind::ConstructorSignature),
                    1 => (
                        Some(SourceStructureMemberId::new(0)),
                        SourceStructureRequestKind::MemberIdentity,
                    ),
                    2 => (
                        Some(SourceStructureMemberId::new(0)),
                        SourceStructureRequestKind::InheritancePath,
                    ),
                    3 => (
                        Some(SourceStructureMemberId::new(1)),
                        SourceStructureRequestKind::MemberIdentity,
                    ),
                    4 => (
                        Some(SourceStructureMemberId::new(1)),
                        SourceStructureRequestKind::InheritancePath,
                    ),
                    5 => (None, SourceStructureRequestKind::ResultType),
                    _ => unreachable!("six requests were checked"),
                };
                request.term() == SourceStructureTermId::new(0)
                    && request.request_ordinal() == index
                    && request.member() == member
                    && request.kind() == kind
            })
}

fn exact_task258b3m2b2b2b_structure_profile(
    source_id: SourceId,
    module_id: &ModuleId,
    primary_terms: &SourcePrimaryTermHandoff,
    structure: &SourceStructureHandoff,
    arena: &TypedArena,
) -> bool {
    let imported_module = ModuleId::new(
        module_id.package().clone(),
        ModulePath::new("parser.type_fixtures"),
    );
    let Some(selector) = structure.terms().get(SourceStructureTermId::new(0)) else {
        return false;
    };
    let Some(constructor) = structure.terms().get(SourceStructureTermId::new(1)) else {
        return false;
    };
    let Some(root) = structure.roots().get(SourceStructureRootId::new(0)) else {
        return false;
    };
    structure
        .validate_installation(source_id, module_id, primary_terms, None, arena)
        .is_ok()
        && structure.source_id() == source_id
        && structure.module_id() == module_id
        && structure.primary_term_fingerprint() == primary_terms.debug_text()
        && structure.application_fingerprint().is_none()
        && structure.terms().len() == 2
        && structure.wrappers().is_empty()
        && structure.roots().len() == 1
        && structure.members().len() == 3
        && structure.field_updates().is_empty()
        && structure.edges().len() == 3
        && structure.requests().len() == 9
        && selector.site().node().index() == 62
        && selector.source_range() == range(source_id, 122, 150)
        && selector.source_ordinal() == 0
        && selector.context() == BindingContextId::new(1)
        && selector.recovery() == SourceStructureRecovery::Normal
        && selector.spelling() == "TypeCaseStruct ( x : 1 , y : 2 ) . x"
        && selector.kind() == SourceStructureTermKind::SelectorAccess
        && constructor.site().node().index() == 61
        && constructor.source_range() == range(source_id, 122, 148)
        && constructor.source_ordinal() == 1
        && constructor.context() == BindingContextId::new(1)
        && constructor.recovery() == SourceStructureRecovery::Normal
        && constructor.spelling() == "TypeCaseStruct ( x : 1 , y : 2 )"
        && constructor.kind() == SourceStructureTermKind::Constructor
        && root.term() == SourceStructureTermId::new(1)
        && root.symbol().module() == &imported_module
        && root.symbol().local().as_str()
            == "summary:parser.type_fixtures#parse-only#TypeCaseStruct:5"
        && root.symbol().fqn().as_str() == "parser.type_fixtures::TypeCaseStruct#5"
        && root.contribution().index() == 2
        && root.origin().source_id() == source_id
        && root.origin().module_id() == &imported_module
        && root.origin().anchor() == &SourceAnchor::Range(range(source_id, 7, 27))
        && root.origin().structural_path() == [5]
        && root.origin().import_edge().is_none()
        && !root.origin().is_recovered()
        && root.visibility() == Visibility::Public
        && root.export_status() == ExportStatus::Exported
        && root.signature().is_none()
        && structure
            .members()
            .iter()
            .enumerate()
            .all(|(index, (_, member))| {
                let (term, ordinal, site, start, end, spelling, role) = match index {
                    0 => (
                        SourceStructureTermId::new(0),
                        0,
                        29,
                        149,
                        150,
                        "x",
                        SourceStructureMemberRole::Selector,
                    ),
                    1 => (
                        SourceStructureTermId::new(1),
                        0,
                        20,
                        137,
                        138,
                        "x",
                        SourceStructureMemberRole::ConstructorAssignment,
                    ),
                    2 => (
                        SourceStructureTermId::new(1),
                        1,
                        24,
                        143,
                        144,
                        "y",
                        SourceStructureMemberRole::ConstructorAssignment,
                    ),
                    _ => unreachable!("three members were checked"),
                };
                member.term() == term
                    && member.ordinal() == ordinal
                    && member.site().node().index() == site
                    && member.source_range() == range(source_id, start, end)
                    && member.spelling() == spelling
                    && member.role() == role
                    && member.parent().is_none()
            })
        && structure
            .edges()
            .iter()
            .enumerate()
            .all(|(index, (_, edge))| {
                let (term, ordinal, role, member, target) = match index {
                    0 => (
                        SourceStructureTermId::new(0),
                        0,
                        SourceStructureEdgeRole::SelectorBase,
                        None,
                        SourceStructureTarget::Structure(SourceStructureTermId::new(1)),
                    ),
                    1 => (
                        SourceStructureTermId::new(1),
                        0,
                        SourceStructureEdgeRole::ConstructorValue,
                        Some(SourceStructureMemberId::new(1)),
                        SourceStructureTarget::Primary(SourcePrimaryTermId::new(2)),
                    ),
                    2 => (
                        SourceStructureTermId::new(1),
                        1,
                        SourceStructureEdgeRole::ConstructorValue,
                        Some(SourceStructureMemberId::new(2)),
                        SourceStructureTarget::Primary(SourcePrimaryTermId::new(3)),
                    ),
                    _ => unreachable!("three edges were checked"),
                };
                edge.term() == term
                    && edge.ordinal() == ordinal
                    && edge.role() == role
                    && edge.member() == member
                    && edge.target() == target
            })
        && structure
            .requests()
            .iter()
            .enumerate()
            .all(|(index, (_, request))| {
                let (term, member, kind) = match index {
                    0 => (
                        SourceStructureTermId::new(0),
                        Some(SourceStructureMemberId::new(0)),
                        SourceStructureRequestKind::MemberIdentity,
                    ),
                    1 => (
                        SourceStructureTermId::new(0),
                        Some(SourceStructureMemberId::new(0)),
                        SourceStructureRequestKind::InheritancePath,
                    ),
                    2 => (
                        SourceStructureTermId::new(0),
                        None,
                        SourceStructureRequestKind::ResultType,
                    ),
                    3 => (
                        SourceStructureTermId::new(1),
                        None,
                        SourceStructureRequestKind::ConstructorSignature,
                    ),
                    4 => (
                        SourceStructureTermId::new(1),
                        Some(SourceStructureMemberId::new(1)),
                        SourceStructureRequestKind::MemberIdentity,
                    ),
                    5 => (
                        SourceStructureTermId::new(1),
                        Some(SourceStructureMemberId::new(1)),
                        SourceStructureRequestKind::InheritancePath,
                    ),
                    6 => (
                        SourceStructureTermId::new(1),
                        Some(SourceStructureMemberId::new(2)),
                        SourceStructureRequestKind::MemberIdentity,
                    ),
                    7 => (
                        SourceStructureTermId::new(1),
                        Some(SourceStructureMemberId::new(2)),
                        SourceStructureRequestKind::InheritancePath,
                    ),
                    8 => (
                        SourceStructureTermId::new(1),
                        None,
                        SourceStructureRequestKind::ResultType,
                    ),
                    _ => unreachable!("nine requests were checked"),
                };
                request.term() == term
                    && request.request_ordinal()
                        == if term == SourceStructureTermId::new(0) {
                            index
                        } else {
                            index - 3
                        }
                    && request.member() == member
                    && request.kind() == kind
            })
}

#[cfg(test)]
fn exact_task258b3m2b2b2c_structure_profile(
    source_id: SourceId,
    module_id: &ModuleId,
    primary_terms: &SourcePrimaryTermHandoff,
    structure: &SourceStructureHandoff,
    arena: &TypedArena,
) -> bool {
    let imported_module = ModuleId::new(
        module_id.package().clone(),
        ModulePath::new("parser.type_fixtures"),
    );
    let Some(update) = structure.terms().get(SourceStructureTermId::new(0)) else {
        return false;
    };
    let Some(constructor) = structure.terms().get(SourceStructureTermId::new(1)) else {
        return false;
    };
    let Some(root) = structure.roots().get(SourceStructureRootId::new(0)) else {
        return false;
    };
    let Some(field_update) = structure.field_updates().get(SourceFieldUpdateId::new(0)) else {
        return false;
    };
    structure
        .validate_installation(source_id, module_id, primary_terms, None, arena)
        .is_ok()
        && structure.source_id() == source_id
        && structure.module_id() == module_id
        && structure.primary_term_fingerprint() == primary_terms.debug_text()
        && structure.application_fingerprint().is_none()
        && structure.terms().len() == 2
        && structure.wrappers().is_empty()
        && structure.roots().len() == 1
        && structure.members().len() == 3
        && structure.field_updates().len() == 1
        && structure.edges().len() == 4
        && structure.requests().len() == 9
        && update.site().node().index() == 69
        && update.source_range() == range(source_id, 120, 160)
        && update.source_ordinal() == 0
        && update.context() == BindingContextId::new(1)
        && update.recovery() == SourceStructureRecovery::Normal
        && update.spelling() == "TypeCaseStruct ( x : 1 , y : 2 ) with ( x := 3 )"
        && update.kind() == SourceStructureTermKind::FunctionalUpdate
        && constructor.site().node().index() == 65
        && constructor.source_range() == range(source_id, 120, 146)
        && constructor.source_ordinal() == 1
        && constructor.context() == BindingContextId::new(1)
        && constructor.recovery() == SourceStructureRecovery::Normal
        && constructor.spelling() == "TypeCaseStruct ( x : 1 , y : 2 )"
        && constructor.kind() == SourceStructureTermKind::Constructor
        && root.term() == SourceStructureTermId::new(1)
        && root.symbol().module() == &imported_module
        && root.symbol().local().as_str()
            == "summary:parser.type_fixtures#parse-only#TypeCaseStruct:5"
        && root.symbol().fqn().as_str() == "parser.type_fixtures::TypeCaseStruct#5"
        && root.contribution().index() == 2
        && root.origin().source_id() == source_id
        && root.origin().module_id() == &imported_module
        && root.origin().anchor() == &SourceAnchor::Range(range(source_id, 7, 27))
        && root.origin().structural_path() == [5]
        && root.origin().import_edge().is_none()
        && !root.origin().is_recovered()
        && root.visibility() == Visibility::Public
        && root.export_status() == ExportStatus::Exported
        && root.signature().is_none()
        && structure
            .members()
            .iter()
            .enumerate()
            .all(|(index, (_, member))| {
                let (term, ordinal, site, start, end, spelling, role) = match index {
                    0 => (
                        SourceStructureTermId::new(0),
                        0,
                        30,
                        153,
                        154,
                        "x",
                        SourceStructureMemberRole::UpdatePathSegment,
                    ),
                    1 => (
                        SourceStructureTermId::new(1),
                        0,
                        20,
                        135,
                        136,
                        "x",
                        SourceStructureMemberRole::ConstructorAssignment,
                    ),
                    2 => (
                        SourceStructureTermId::new(1),
                        1,
                        24,
                        141,
                        142,
                        "y",
                        SourceStructureMemberRole::ConstructorAssignment,
                    ),
                    _ => unreachable!("three members were checked"),
                };
                member.term() == term
                    && member.ordinal() == ordinal
                    && member.site().node().index() == site
                    && member.source_range() == range(source_id, start, end)
                    && member.spelling() == spelling
                    && member.role() == role
                    && member.parent().is_none()
            })
        && field_update.term() == SourceStructureTermId::new(0)
        && field_update.ordinal() == 0
        && field_update.site().node().index() == 68
        && field_update.source_range() == range(source_id, 153, 159)
        && field_update.spelling() == "x := 3"
        && field_update.first_member() == SourceStructureMemberId::new(0)
        && field_update.final_member() == SourceStructureMemberId::new(0)
        && structure
            .edges()
            .iter()
            .enumerate()
            .all(|(index, (_, edge))| {
                let (term, ordinal, role, member, target) = match index {
                    0 => (
                        SourceStructureTermId::new(0),
                        0,
                        SourceStructureEdgeRole::UpdateBase,
                        None,
                        SourceStructureTarget::Structure(SourceStructureTermId::new(1)),
                    ),
                    1 => (
                        SourceStructureTermId::new(0),
                        1,
                        SourceStructureEdgeRole::UpdateValue,
                        Some(SourceStructureMemberId::new(0)),
                        SourceStructureTarget::Primary(SourcePrimaryTermId::new(4)),
                    ),
                    2 => (
                        SourceStructureTermId::new(1),
                        0,
                        SourceStructureEdgeRole::ConstructorValue,
                        Some(SourceStructureMemberId::new(1)),
                        SourceStructureTarget::Primary(SourcePrimaryTermId::new(2)),
                    ),
                    3 => (
                        SourceStructureTermId::new(1),
                        1,
                        SourceStructureEdgeRole::ConstructorValue,
                        Some(SourceStructureMemberId::new(2)),
                        SourceStructureTarget::Primary(SourcePrimaryTermId::new(3)),
                    ),
                    _ => unreachable!("four edges were checked"),
                };
                edge.term() == term
                    && edge.ordinal() == ordinal
                    && edge.role() == role
                    && edge.member() == member
                    && edge.target() == target
            })
        && structure
            .requests()
            .iter()
            .enumerate()
            .all(|(index, (_, request))| {
                let (term, ordinal, member, kind) = match index {
                    0 => (
                        SourceStructureTermId::new(0),
                        0,
                        Some(SourceStructureMemberId::new(0)),
                        SourceStructureRequestKind::MemberIdentity,
                    ),
                    1 => (
                        SourceStructureTermId::new(0),
                        1,
                        Some(SourceStructureMemberId::new(0)),
                        SourceStructureRequestKind::InheritancePath,
                    ),
                    2 => (
                        SourceStructureTermId::new(0),
                        2,
                        None,
                        SourceStructureRequestKind::ResultType,
                    ),
                    3 => (
                        SourceStructureTermId::new(1),
                        0,
                        None,
                        SourceStructureRequestKind::ConstructorSignature,
                    ),
                    4 => (
                        SourceStructureTermId::new(1),
                        1,
                        Some(SourceStructureMemberId::new(1)),
                        SourceStructureRequestKind::MemberIdentity,
                    ),
                    5 => (
                        SourceStructureTermId::new(1),
                        2,
                        Some(SourceStructureMemberId::new(1)),
                        SourceStructureRequestKind::InheritancePath,
                    ),
                    6 => (
                        SourceStructureTermId::new(1),
                        3,
                        Some(SourceStructureMemberId::new(2)),
                        SourceStructureRequestKind::MemberIdentity,
                    ),
                    7 => (
                        SourceStructureTermId::new(1),
                        4,
                        Some(SourceStructureMemberId::new(2)),
                        SourceStructureRequestKind::InheritancePath,
                    ),
                    8 => (
                        SourceStructureTermId::new(1),
                        5,
                        None,
                        SourceStructureRequestKind::ResultType,
                    ),
                    _ => unreachable!("nine requests were checked"),
                };
                request.term() == term
                    && request.request_ordinal() == ordinal
                    && request.member() == member
                    && request.kind() == kind
            })
}

#[cfg(test)]
fn exact_task258b3m2b2b3a_set_profile(
    source_id: SourceId,
    module_id: &ModuleId,
    primary_terms: &SourcePrimaryTermHandoff,
    set_terms: &SourceSetTermHandoff,
    arena: &TypedArena,
) -> bool {
    let Some(term) = set_terms.terms().get(SourceSetTermId::new(0)) else {
        return false;
    };
    set_terms
        .validate_installation(source_id, module_id, primary_terms, None, None, arena)
        .is_ok()
        && set_terms.source_id() == source_id
        && set_terms.module_id() == module_id
        && set_terms.primary_term_fingerprint() == primary_terms.debug_text()
        && set_terms.application_fingerprint().is_none()
        && set_terms.structure_fingerprint().is_none()
        && set_terms.terms().len() == 1
        && set_terms.wrappers().is_empty()
        && set_terms.generators().is_empty()
        && set_terms.type_sites().is_empty()
        && set_terms.conditions().is_empty()
        && set_terms.edges().len() == 2
        && set_terms.requests().len() == 1
        && term.site().node().index() == 40
        && term.source_range() == range(source_id, 90, 96)
        && term.source_ordinal() == 0
        && term.context() == BindingContextId::new(1)
        && term.recovery() == SourceSetTermRecovery::Normal
        && term.spelling() == "{ 1 , 2 }"
        && term.kind() == SourceSetTermKind::Enumeration
        && set_terms
            .edges()
            .iter()
            .enumerate()
            .all(|(index, (_, edge))| {
                edge.term() == SourceSetTermId::new(0)
                    && edge.ordinal() == index
                    && edge.role() == SourceSetEdgeRole::EnumerationElement
                    && edge.target()
                        == SourceSetTarget::Primary(SourcePrimaryTermId::new(index + 2))
            })
        && set_terms
            .requests()
            .get(crate::source_set_term::SourceSetRequestId::new(0))
            .is_some_and(|request| {
                request.term() == SourceSetTermId::new(0)
                    && request.ordinal() == 0
                    && request.kind() == SourceSetRequestKind::ResultType
                    && request.generator().is_none()
                    && request.type_site().is_none()
            })
}

#[cfg(test)]
fn exact_task258b3m2b2b3b_set_profile(
    source_id: SourceId,
    module_id: &ModuleId,
    primary_terms: &SourcePrimaryTermHandoff,
    set_terms: &SourceSetTermHandoff,
    arena: &TypedArena,
) -> bool {
    let Some(term) = set_terms.terms().get(SourceSetTermId::new(0)) else {
        return false;
    };
    set_terms
        .validate_installation(source_id, module_id, primary_terms, None, None, arena)
        .is_ok()
        && set_terms.source_id() == source_id
        && set_terms.module_id() == module_id
        && set_terms.primary_term_fingerprint() == primary_terms.debug_text()
        && set_terms.application_fingerprint().is_none()
        && set_terms.structure_fingerprint().is_none()
        && set_terms.terms().len() == 1
        && set_terms.wrappers().is_empty()
        && set_terms.generators().is_empty()
        && set_terms.type_sites().is_empty()
        && set_terms.conditions().is_empty()
        && set_terms.edges().is_empty()
        && set_terms.requests().len() == 1
        && term.site().node().index() == 33
        && term.source_range() == range(source_id, 95, 97)
        && term.source_ordinal() == 0
        && term.context() == BindingContextId::new(1)
        && term.recovery() == SourceSetTermRecovery::Normal
        && term.spelling() == "{ }"
        && term.kind() == SourceSetTermKind::Enumeration
        && set_terms
            .requests()
            .get(crate::source_set_term::SourceSetRequestId::new(0))
            .is_some_and(|request| {
                request.term() == SourceSetTermId::new(0)
                    && request.ordinal() == 0
                    && request.kind() == SourceSetRequestKind::ResultType
                    && request.generator().is_none()
                    && request.type_site().is_none()
            })
}

#[cfg(test)]
fn exact_task258b3m2b2b3c_set_profile(
    source_id: SourceId,
    module_id: &ModuleId,
    primary_terms: &SourcePrimaryTermHandoff,
    set_terms: &SourceSetTermHandoff,
    arena: &TypedArena,
) -> bool {
    let Some(term) = set_terms.terms().get(SourceSetTermId::new(0)) else {
        return false;
    };
    let Some(type_site) = set_terms.type_sites().get(SourceSetTypeSiteId::new(0)) else {
        return false;
    };
    set_terms
        .validate_installation(source_id, module_id, primary_terms, None, None, arena)
        .is_ok()
        && set_terms.source_id() == source_id
        && set_terms.module_id() == module_id
        && set_terms.primary_term_fingerprint() == primary_terms.debug_text()
        && set_terms.application_fingerprint().is_none()
        && set_terms.structure_fingerprint().is_none()
        && set_terms.terms().len() == 1
        && set_terms.wrappers().is_empty()
        && set_terms.generators().is_empty()
        && set_terms.type_sites().len() == 1
        && set_terms.conditions().is_empty()
        && set_terms.edges().is_empty()
        && set_terms.requests().len() == 2
        && term.site().node().index() == 35
        && term.source_range() == range(source_id, 82, 89)
        && term.source_ordinal() == 0
        && term.context() == BindingContextId::new(1)
        && term.recovery() == SourceSetTermRecovery::Normal
        && term.spelling() == "the set"
        && term.kind() == SourceSetTermKind::Choice
        && type_site.owner()
            == (SourceSetTypeOwner::Term {
                term: SourceSetTermId::new(0),
                role: SourceSetTypeRole::ChoiceTarget,
            })
        && type_site.site().node().index() == 34
        && type_site.source_range() == range(source_id, 86, 89)
        && type_site.spelling() == "set"
        && type_site.head_site().node().index() == 33
        && type_site.head_range() == range(source_id, 86, 89)
        && type_site.head_spelling() == "set"
        && type_site.context() == BindingContextId::new(1)
        && type_site.recovery() == SourceSetTermRecovery::Normal
        && type_site.head() == SourceSetTypeHead::BuiltinSet
        && set_terms
            .requests()
            .get(crate::source_set_term::SourceSetRequestId::new(0))
            .is_some_and(|request| {
                request.term() == SourceSetTermId::new(0)
                    && request.ordinal() == 0
                    && request.kind() == SourceSetRequestKind::ChoiceNonempty
                    && request.generator().is_none()
                    && request.type_site() == Some(SourceSetTypeSiteId::new(0))
            })
        && set_terms
            .requests()
            .get(crate::source_set_term::SourceSetRequestId::new(1))
            .is_some_and(|request| {
                request.term() == SourceSetTermId::new(0)
                    && request.ordinal() == 1
                    && request.kind() == SourceSetRequestKind::ResultType
                    && request.generator().is_none()
                    && request.type_site().is_none()
            })
}

#[cfg(test)]
fn exact_task258b3m2b2b3d_set_profile(
    source_id: SourceId,
    module_id: &ModuleId,
    primary_terms: &SourcePrimaryTermHandoff,
    set_terms: &SourceSetTermHandoff,
    arena: &TypedArena,
) -> bool {
    let Some(term) = set_terms.terms().get(SourceSetTermId::new(0)) else {
        return false;
    };
    let Some(type_site) = set_terms.type_sites().get(SourceSetTypeSiteId::new(0)) else {
        return false;
    };
    let Some(edge) = set_terms
        .edges()
        .get(crate::source_set_term::SourceSetEdgeId::new(0))
    else {
        return false;
    };
    set_terms
        .validate_installation(source_id, module_id, primary_terms, None, None, arena)
        .is_ok()
        && set_terms.source_id() == source_id
        && set_terms.module_id() == module_id
        && set_terms.primary_term_fingerprint() == primary_terms.debug_text()
        && set_terms.application_fingerprint().is_none()
        && set_terms.structure_fingerprint().is_none()
        && set_terms.terms().len() == 1
        && set_terms.wrappers().is_empty()
        && set_terms.generators().is_empty()
        && set_terms.type_sites().len() == 1
        && set_terms.conditions().is_empty()
        && set_terms.edges().len() == 1
        && set_terms.requests().len() == 2
        && term.site().node().index() == 37
        && term.source_range() == range(source_id, 79, 88)
        && term.source_ordinal() == 0
        && term.context() == BindingContextId::new(1)
        && term.recovery() == SourceSetTermRecovery::Normal
        && term.spelling() == "4 qua set"
        && term.kind() == SourceSetTermKind::Qua
        && type_site.owner()
            == (SourceSetTypeOwner::Term {
                term: SourceSetTermId::new(0),
                role: SourceSetTypeRole::QuaTarget,
            })
        && type_site.site().node().index() == 36
        && type_site.source_range() == range(source_id, 85, 88)
        && type_site.spelling() == "set"
        && type_site.head_site().node().index() == 35
        && type_site.head_range() == range(source_id, 85, 88)
        && type_site.head_spelling() == "set"
        && type_site.context() == BindingContextId::new(1)
        && type_site.recovery() == SourceSetTermRecovery::Normal
        && type_site.head() == SourceSetTypeHead::BuiltinSet
        && edge.term() == SourceSetTermId::new(0)
        && edge.ordinal() == 0
        && edge.role() == SourceSetEdgeRole::QuaBase
        && edge.target() == SourceSetTarget::Primary(SourcePrimaryTermId::new(2))
        && set_terms
            .requests()
            .get(crate::source_set_term::SourceSetRequestId::new(0))
            .is_some_and(|request| {
                request.term() == SourceSetTermId::new(0)
                    && request.ordinal() == 0
                    && request.kind() == SourceSetRequestKind::QuaWidening
                    && request.generator().is_none()
                    && request.type_site() == Some(SourceSetTypeSiteId::new(0))
            })
        && set_terms
            .requests()
            .get(crate::source_set_term::SourceSetRequestId::new(1))
            .is_some_and(|request| {
                request.term() == SourceSetTermId::new(0)
                    && request.ordinal() == 1
                    && request.kind() == SourceSetRequestKind::ResultType
                    && request.generator().is_none()
                    && request.type_site().is_none()
            })
}

#[cfg(test)]
fn exact_task258b3m2b2b3e_set_profile(
    source_id: SourceId,
    module_id: &ModuleId,
    primary_terms: &SourcePrimaryTermHandoff,
    set_terms: &SourceSetTermHandoff,
    arena: &TypedArena,
) -> bool {
    let Some(term) = set_terms.terms().get(SourceSetTermId::new(0)) else {
        return false;
    };
    let Some(generator) = set_terms.generators().get(SourceSetGeneratorId::new(0)) else {
        return false;
    };
    let Some(type_site) = set_terms.type_sites().get(SourceSetTypeSiteId::new(0)) else {
        return false;
    };
    let Some(edge) = set_terms
        .edges()
        .get(crate::source_set_term::SourceSetEdgeId::new(0))
    else {
        return false;
    };
    set_terms
        .validate_installation(source_id, module_id, primary_terms, None, None, arena)
        .is_ok()
        && set_terms.source_id() == source_id
        && set_terms.module_id() == module_id
        && set_terms.primary_term_fingerprint() == primary_terms.debug_text()
        && set_terms.application_fingerprint().is_none()
        && set_terms.structure_fingerprint().is_none()
        && set_terms.terms().len() == 1
        && set_terms.wrappers().is_empty()
        && set_terms.generators().len() == 1
        && set_terms.type_sites().len() == 1
        && set_terms.conditions().is_empty()
        && set_terms.edges().len() == 1
        && set_terms.requests().len() == 2
        && term.site().node().index() == 43
        && term.source_range() == range(source_id, 89, 118)
        && term.source_ordinal() == 0
        && term.context() == BindingContextId::new(1)
        && term.recovery() == SourceSetTermRecovery::Normal
        && term.spelling() == "{ 3 where candidate255 is set }"
        && term.kind() == SourceSetTermKind::Comprehension
        && generator.term() == SourceSetTermId::new(0)
        && generator.ordinal() == 0
        && generator.site().node().index() == 16
        && generator.source_range() == range(source_id, 98, 110)
        && generator.spelling() == "candidate255"
        && generator.context() == BindingContextId::new(1)
        && generator.recovery() == SourceSetTermRecovery::Normal
        && generator.type_site() == SourceSetTypeSiteId::new(0)
        && type_site.owner() == SourceSetTypeOwner::Generator(SourceSetGeneratorId::new(0))
        && type_site.site().node().index() == 41
        && type_site.source_range() == range(source_id, 114, 117)
        && type_site.spelling() == "set"
        && type_site.head_site().node().index() == 40
        && type_site.head_range() == range(source_id, 114, 117)
        && type_site.head_spelling() == "set"
        && type_site.context() == BindingContextId::new(1)
        && type_site.recovery() == SourceSetTermRecovery::Normal
        && type_site.head() == SourceSetTypeHead::BuiltinSet
        && edge.term() == SourceSetTermId::new(0)
        && edge.ordinal() == 0
        && edge.role() == SourceSetEdgeRole::ComprehensionMapper
        && edge.target() == SourceSetTarget::Primary(SourcePrimaryTermId::new(2))
        && set_terms
            .requests()
            .get(crate::source_set_term::SourceSetRequestId::new(0))
            .is_some_and(|request| {
                request.term() == SourceSetTermId::new(0)
                    && request.ordinal() == 0
                    && request.kind() == SourceSetRequestKind::GeneratorSethood
                    && request.generator() == Some(SourceSetGeneratorId::new(0))
                    && request.type_site() == Some(SourceSetTypeSiteId::new(0))
            })
        && set_terms
            .requests()
            .get(crate::source_set_term::SourceSetRequestId::new(1))
            .is_some_and(|request| {
                request.term() == SourceSetTermId::new(0)
                    && request.ordinal() == 1
                    && request.kind() == SourceSetRequestKind::ResultType
                    && request.generator().is_none()
                    && request.type_site().is_none()
            })
}

#[allow(clippy::too_many_arguments)] // Rationale: authenticate the complete mutually exclusive statement/lower-family dependency tuple.
fn validate_witness_dependencies(
    source_id: SourceId,
    module_id: &ModuleId,
    statement_fingerprint: &str,
    primary_term_fingerprint: &str,
    application_fingerprint: Option<&str>,
    structure_fingerprint: Option<&str>,
    set_term_fingerprint: Option<&str>,
    statements: &SourceStatementHandoff,
    primary_terms: &SourcePrimaryTermHandoff,
    application: Option<&SourceFunctorApplicationHandoff>,
    structure: Option<&SourceStructureHandoff>,
    set_terms: Option<&SourceSetTermHandoff>,
    arena: &TypedArena,
) -> Result<WitnessProfile, SourceStatementWitnessError> {
    if statements.source_id() != source_id
        || statements.module_id() != module_id
        || primary_terms.source_id() != source_id
        || primary_terms.module_id() != module_id
        || statement_fingerprint != statements.debug_text()
        || primary_term_fingerprint != primary_terms.debug_text()
        || statements.primary_term_fingerprint() != primary_term_fingerprint
        || application_fingerprint
            != application
                .map(SourceFunctorApplicationHandoff::debug_text)
                .as_deref()
        || structure_fingerprint != structure.map(SourceStructureHandoff::debug_text).as_deref()
        || set_term_fingerprint != set_terms.map(SourceSetTermHandoff::debug_text).as_deref()
        || usize::from(application.is_some())
            + usize::from(structure.is_some())
            + usize::from(set_terms.is_some())
            > 1
    {
        return Err(SourceStatementWitnessError::DependencyMismatch);
    }
    let profile = if application.is_none()
        && set_terms.is_none()
        && statements.is_task_258b3m2b2b2a_profile()
        && exact_binding_profile(
            StatementProfile::Task258B3M2B2B2A,
            source_id,
            statements.binding_env(),
        )
        && exact_primary_profile(StatementProfile::Task258B3M2B2B2A, primary_terms)
        && exact_task258b3m2b2b2a_shared_arena(source_id, arena)
        && structure.is_some_and(|structure| {
            exact_task258b3m2b2b2a_structure_profile(
                source_id,
                module_id,
                primary_terms,
                structure,
                arena,
            )
        }) {
        WitnessProfile::Task258B3M2B2B2A
    } else if application.is_none()
        && set_terms.is_none()
        && statements.is_task_258b3m2b2b2b_profile()
        && exact_binding_profile(
            StatementProfile::Task258B3M2B2B2B,
            source_id,
            statements.binding_env(),
        )
        && exact_primary_profile(StatementProfile::Task258B3M2B2B2B, primary_terms)
        && exact_task258b3m2b2b2b_shared_arena(source_id, arena)
        && structure.is_some_and(|structure| {
            exact_task258b3m2b2b2b_structure_profile(
                source_id,
                module_id,
                primary_terms,
                structure,
                arena,
            )
        })
    {
        WitnessProfile::Task258B3M2B2B2B
    } else {
        #[cfg(test)]
        {
            if application.is_none()
                && set_terms.is_none()
                && statements.is_task_258b3m2b2b2c_profile()
                && exact_binding_profile(
                    StatementProfile::Task258B3M2B2B2C,
                    source_id,
                    statements.binding_env(),
                )
                && exact_primary_profile(StatementProfile::Task258B3M2B2B2C, primary_terms)
                && exact_task258b3m2b2b2c_shared_arena(source_id, arena)
                && structure.is_some_and(|structure| {
                    exact_task258b3m2b2b2c_structure_profile(
                        source_id,
                        module_id,
                        primary_terms,
                        structure,
                        arena,
                    )
                })
            {
                WitnessProfile::Task258B3M2B2B2C
            } else if structure.is_none()
                && set_terms.is_none()
                && statements.is_task_258b3m2b2b1a_profile()
                && exact_binding_profile(
                    StatementProfile::Task258B3M2B2B1A,
                    source_id,
                    statements.binding_env(),
                )
                && exact_primary_profile(StatementProfile::Task258B3M2B2B1A, primary_terms)
                && exact_task258b3m2b2b1a_shared_arena(source_id, arena)
                && application.is_some_and(|application| {
                    exact_task258b3m2b2b1a_application_profile(
                        source_id,
                        module_id,
                        primary_terms,
                        application,
                    )
                })
            {
                WitnessProfile::Task258B3M2B2B1A
            } else if structure.is_none()
                && set_terms.is_none()
                && statements.is_task_258b3m2b2b1b1_profile()
                && exact_binding_profile(
                    StatementProfile::Task258B3M2B2B1B1,
                    source_id,
                    statements.binding_env(),
                )
                && exact_primary_profile(StatementProfile::Task258B3M2B2B1B1, primary_terms)
                && exact_task258b3m2b2b1b1_shared_arena(source_id, arena)
                && application.is_some_and(|application| {
                    exact_task258b3m2b2b1b1_application_profile(
                        source_id,
                        module_id,
                        primary_terms,
                        application,
                    )
                })
            {
                WitnessProfile::Task258B3M2B2B1B1
            } else if application.is_none()
                && structure.is_none()
                && statements.is_task_258b3m2b2b3a_profile()
                && exact_binding_profile(
                    StatementProfile::Task258B3M2B2B3A,
                    source_id,
                    statements.binding_env(),
                )
                && exact_primary_profile(StatementProfile::Task258B3M2B2B3A, primary_terms)
                && exact_task258b3m2b2b3a_shared_arena(source_id, arena)
                && set_terms.is_some_and(|set_terms| {
                    exact_task258b3m2b2b3a_set_profile(
                        source_id,
                        module_id,
                        primary_terms,
                        set_terms,
                        arena,
                    )
                })
            {
                WitnessProfile::Task258B3M2B2B3A
            } else if application.is_none()
                && structure.is_none()
                && set_terms.is_none()
                && statements.is_task_258b3m2a_profile()
                && exact_binding_profile(
                    StatementProfile::Task258B3M2A,
                    source_id,
                    statements.binding_env(),
                )
                && exact_primary_profile(StatementProfile::Task258B3M2A, primary_terms)
                && exact_task258b3m2a_shared_arena(source_id, arena)
            {
                WitnessProfile::Task258B3M2A
            } else if application.is_none()
                && structure.is_none()
                && statements.is_task_258b3m2b2b3b_profile()
                && exact_binding_profile(
                    StatementProfile::Task258B3M2B2B3B,
                    source_id,
                    statements.binding_env(),
                )
                && exact_primary_profile(StatementProfile::Task258B3M2B2B3B, primary_terms)
                && exact_task258b3m2b2b3b_shared_arena(source_id, arena)
                && set_terms.is_some_and(|set_terms| {
                    exact_task258b3m2b2b3b_set_profile(
                        source_id,
                        module_id,
                        primary_terms,
                        set_terms,
                        arena,
                    )
                })
            {
                WitnessProfile::Task258B3M2B2B3B
            } else if application.is_none()
                && structure.is_none()
                && statements.is_task_258b3m2b2b3c_profile()
                && exact_binding_profile(
                    StatementProfile::Task258B3M2B2B3C,
                    source_id,
                    statements.binding_env(),
                )
                && exact_primary_profile(StatementProfile::Task258B3M2B2B3C, primary_terms)
                && exact_task258b3m2b2b3c_shared_arena(source_id, arena)
                && set_terms.is_some_and(|set_terms| {
                    exact_task258b3m2b2b3c_set_profile(
                        source_id,
                        module_id,
                        primary_terms,
                        set_terms,
                        arena,
                    )
                })
            {
                WitnessProfile::Task258B3M2B2B3C
            } else if application.is_none()
                && structure.is_none()
                && statements.is_task_258b3m2b2b3d_profile()
                && exact_binding_profile(
                    StatementProfile::Task258B3M2B2B3D,
                    source_id,
                    statements.binding_env(),
                )
                && exact_primary_profile(StatementProfile::Task258B3M2B2B3D, primary_terms)
                && exact_task258b3m2b2b3d_shared_arena(source_id, arena)
                && set_terms.is_some_and(|set_terms| {
                    exact_task258b3m2b2b3d_set_profile(
                        source_id,
                        module_id,
                        primary_terms,
                        set_terms,
                        arena,
                    )
                })
            {
                WitnessProfile::Task258B3M2B2B3D
            } else if application.is_none()
                && structure.is_none()
                && statements.is_task_258b3m2b2b3e_profile()
                && exact_binding_profile(
                    StatementProfile::Task258B3M2B2B3E,
                    source_id,
                    statements.binding_env(),
                )
                && exact_primary_profile(StatementProfile::Task258B3M2B2B3E, primary_terms)
                && exact_task258b3m2b2b3e_shared_arena(source_id, arena)
                && set_terms.is_some_and(|set_terms| {
                    exact_task258b3m2b2b3e_set_profile(
                        source_id,
                        module_id,
                        primary_terms,
                        set_terms,
                        arena,
                    )
                })
            {
                WitnessProfile::Task258B3M2B2B3E
            } else if application.is_none()
                && structure.is_none()
                && set_terms.is_none()
                && statements.is_task_258b3m2b2a_profile()
                && exact_binding_profile(
                    StatementProfile::Task258B3M2B2A,
                    source_id,
                    statements.binding_env(),
                )
                && exact_primary_profile(StatementProfile::Task258B3M2B2A, primary_terms)
                && exact_task258b3m2b2a_shared_arena(source_id, arena)
            {
                WitnessProfile::Task258B3M2B2A
            } else if application.is_none()
                && structure.is_none()
                && set_terms.is_none()
                && statements.is_task_258b3m2b1_profile()
                && exact_binding_profile(
                    StatementProfile::Task258B3M2B1,
                    source_id,
                    statements.binding_env(),
                )
                && exact_primary_profile(StatementProfile::Task258B3M2B1, primary_terms)
                && exact_task258b3m2b1_shared_arena(source_id, arena)
            {
                WitnessProfile::Task258B3M2B1
            } else if application.is_none()
                && structure.is_none()
                && set_terms.is_none()
                && statements.is_task_258b3m1_profile()
                && exact_binding_profile(
                    StatementProfile::Task258B3M1,
                    source_id,
                    statements.binding_env(),
                )
                && exact_primary_profile(StatementProfile::Task258B3M1, primary_terms)
                && exact_task258b3m1_shared_arena(source_id, arena)
            {
                WitnessProfile::Task258B3M1
            } else if application.is_none()
                && structure.is_none()
                && set_terms.is_none()
                && statements.is_task_258b3n_profile()
                && exact_binding_profile(
                    StatementProfile::Task258B3N,
                    source_id,
                    statements.binding_env(),
                )
                && exact_primary_profile(StatementProfile::Task258B3N, primary_terms)
                && exact_task258b3n_shared_arena(source_id, arena)
            {
                WitnessProfile::Task258B3N
            } else if application.is_none()
                && structure.is_none()
                && set_terms.is_none()
                && statements.is_task_258b3_profile()
                && exact_binding_profile(
                    StatementProfile::Task258B3,
                    source_id,
                    statements.binding_env(),
                )
                && exact_primary_profile(StatementProfile::Task258B3, primary_terms)
                && exact_task258b3_shared_arena(source_id, arena)
            {
                WitnessProfile::Task258B3
            } else {
                return Err(SourceStatementWitnessError::DependencyMismatch);
            }
        }
        #[cfg(not(test))]
        return Err(SourceStatementWitnessError::DependencyMismatch);
    };
    primary_terms
        .validate_installation(source_id, module_id, arena)
        .map_err(|_| SourceStatementWitnessError::DependencyMismatch)?;
    let Some(owner) = statements.owners().get(SourceTheoremOwnerId::new(0)) else {
        return Err(SourceStatementWitnessError::DependencyMismatch);
    };
    let Some(theorem) = statements.statements().get(SourceStatementId::new(0)) else {
        return Err(SourceStatementWitnessError::DependencyMismatch);
    };
    let Some(conclusion) = statements.statements().get(SourceStatementId::new(1)) else {
        return Err(SourceStatementWitnessError::DependencyMismatch);
    };
    let (owner_range, theorem_site, conclusion_range, conclusion_site) = match profile {
        #[cfg(test)]
        WitnessProfile::Task258B3 => (range(source_id, 19, 103), 45, range(source_id, 87, 98), 43),
        #[cfg(test)]
        WitnessProfile::Task258B3N => {
            (range(source_id, 19, 106), 47, range(source_id, 90, 101), 45)
        }
        #[cfg(test)]
        WitnessProfile::Task258B3M1 => {
            (range(source_id, 19, 112), 52, range(source_id, 96, 107), 50)
        }
        #[cfg(test)]
        WitnessProfile::Task258B3M2A => {
            (range(source_id, 19, 106), 45, range(source_id, 90, 101), 43)
        }
        #[cfg(test)]
        WitnessProfile::Task258B3M2B1 => {
            (range(source_id, 19, 112), 49, range(source_id, 96, 107), 47)
        }
        #[cfg(test)]
        WitnessProfile::Task258B3M2B2A => (
            range(source_id, 19, 120),
            53,
            range(source_id, 104, 115),
            51,
        ),
        #[cfg(test)]
        WitnessProfile::Task258B3M2B2B1A => (
            range(source_id, 48, 142),
            59,
            range(source_id, 126, 137),
            57,
        ),
        #[cfg(test)]
        WitnessProfile::Task258B3M2B2B1B1 => (
            range(source_id, 48, 157),
            63,
            range(source_id, 141, 152),
            61,
        ),
        WitnessProfile::Task258B3M2B2B2A => (
            range(source_id, 48, 171),
            72,
            range(source_id, 155, 166),
            70,
        ),
        WitnessProfile::Task258B3M2B2B2B => (
            range(source_id, 48, 170),
            75,
            range(source_id, 154, 165),
            73,
        ),
        #[cfg(test)]
        WitnessProfile::Task258B3M2B2B2C => (
            range(source_id, 48, 180),
            82,
            range(source_id, 164, 175),
            80,
        ),
        #[cfg(test)]
        WitnessProfile::Task258B3M2B2B3A => (
            range(source_id, 19, 116),
            53,
            range(source_id, 100, 111),
            51,
        ),
        #[cfg(test)]
        WitnessProfile::Task258B3M2B2B3B => (
            range(source_id, 19, 117),
            46,
            range(source_id, 101, 112),
            44,
        ),
        #[cfg(test)]
        WitnessProfile::Task258B3M2B2B3C => {
            (range(source_id, 19, 109), 48, range(source_id, 93, 104), 46)
        }
        #[cfg(test)]
        WitnessProfile::Task258B3M2B2B3D => {
            (range(source_id, 19, 108), 50, range(source_id, 92, 103), 48)
        }
        #[cfg(test)]
        WitnessProfile::Task258B3M2B2B3E => (
            range(source_id, 19, 138),
            56,
            range(source_id, 122, 133),
            54,
        ),
    };
    if owner.source_range() != owner_range
        || owner.site().node().index() != theorem_site
        || theorem.source_range() != owner_range
        || theorem.site().node().index() != theorem_site
        || theorem.source_ordinal() != 0
        || theorem.formula() != SourceStatementFormulaTarget::Atomic(SourceAtomicFormulaId::new(0))
        || conclusion.source_range() != conclusion_range
        || conclusion.site().node().index() != conclusion_site
        || conclusion.source_ordinal() != 2
        || conclusion.formula()
            != SourceStatementFormulaTarget::Atomic(SourceAtomicFormulaId::new(1))
    {
        return Err(SourceStatementWitnessError::DependencyMismatch);
    }
    Ok(profile)
}

fn validate_witness_aggregate(
    profile: WitnessProfile,
    witnesses: usize,
    names: usize,
) -> Result<(), SourceStatementWitnessError> {
    let expected = match profile {
        #[cfg(test)]
        WitnessProfile::Task258B3 => (1, 0),
        #[cfg(test)]
        WitnessProfile::Task258B3N => (1, 1),
        #[cfg(test)]
        WitnessProfile::Task258B3M1 => (2, 1),
        #[cfg(test)]
        WitnessProfile::Task258B3M2A => (1, 0),
        #[cfg(test)]
        WitnessProfile::Task258B3M2B1 => (1, 0),
        #[cfg(test)]
        WitnessProfile::Task258B3M2B2A => (1, 0),
        #[cfg(test)]
        WitnessProfile::Task258B3M2B2B1A => (1, 0),
        #[cfg(test)]
        WitnessProfile::Task258B3M2B2B1B1 => (1, 0),
        WitnessProfile::Task258B3M2B2B2A => (1, 0),
        WitnessProfile::Task258B3M2B2B2B => (1, 0),
        #[cfg(test)]
        WitnessProfile::Task258B3M2B2B2C => (1, 0),
        #[cfg(test)]
        WitnessProfile::Task258B3M2B2B3A => (1, 0),
        #[cfg(test)]
        WitnessProfile::Task258B3M2B2B3B => (1, 0),
        #[cfg(test)]
        WitnessProfile::Task258B3M2B2B3C => (1, 0),
        #[cfg(test)]
        WitnessProfile::Task258B3M2B2B3D => (1, 0),
        #[cfg(test)]
        WitnessProfile::Task258B3M2B2B3E => (1, 0),
    };
    if (witnesses, names) != expected {
        return Err(SourceStatementWitnessError::InvalidAggregate);
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)] // Rationale: validate the complete mutually exclusive witness/lower-family tuple.
fn validate_witness_rows(
    profile: WitnessProfile,
    source_id: SourceId,
    witnesses: &SourceStatementWitnessTable,
    statements: &SourceStatementHandoff,
    primary_terms: &SourcePrimaryTermHandoff,
    _application: Option<&SourceFunctorApplicationHandoff>,
    structure: Option<&SourceStructureHandoff>,
    _set_terms: Option<&SourceSetTermHandoff>,
    arena: &TypedArena,
) -> Result<(), SourceStatementWitnessError> {
    #[cfg(test)]
    let set_terms = _set_terms;
    #[cfg(test)]
    let application = _application;
    #[cfg(test)]
    if profile == WitnessProfile::Task258B3M2B2B1A {
        return validate_task258b3m2b2b1a_witness_rows(
            source_id,
            witnesses,
            statements,
            primary_terms,
            application,
            arena,
        );
    }
    #[cfg(test)]
    if profile == WitnessProfile::Task258B3M2B2B1B1 {
        return validate_task258b3m2b2b1b1_witness_rows(
            source_id,
            witnesses,
            statements,
            primary_terms,
            application,
            arena,
        );
    }
    if profile == WitnessProfile::Task258B3M2B2B2A {
        return validate_task258b3m2b2b2a_witness_rows(
            source_id,
            witnesses,
            statements,
            primary_terms,
            structure,
            arena,
        );
    }
    if profile == WitnessProfile::Task258B3M2B2B2B {
        return validate_task258b3m2b2b2b_witness_rows(
            source_id,
            witnesses,
            statements,
            primary_terms,
            structure,
            arena,
        );
    }
    #[cfg(test)]
    if profile == WitnessProfile::Task258B3M2B2B2C {
        return validate_task258b3m2b2b2c_witness_rows(
            source_id,
            witnesses,
            statements,
            primary_terms,
            structure,
            arena,
        );
    }
    #[cfg(test)]
    if profile == WitnessProfile::Task258B3M2B2B3A {
        return validate_task258b3m2b2b3a_witness_rows(
            source_id,
            witnesses,
            statements,
            primary_terms,
            set_terms,
            arena,
        );
    }
    #[cfg(test)]
    if profile == WitnessProfile::Task258B3M2B2B3B {
        return validate_task258b3m2b2b3b_witness_rows(
            source_id,
            witnesses,
            statements,
            primary_terms,
            set_terms,
            arena,
        );
    }
    #[cfg(test)]
    if profile == WitnessProfile::Task258B3M2B2B3C {
        return validate_task258b3m2b2b3c_witness_rows(
            source_id,
            witnesses,
            statements,
            primary_terms,
            set_terms,
            arena,
        );
    }
    #[cfg(test)]
    if profile == WitnessProfile::Task258B3M2B2B3D {
        return validate_task258b3m2b2b3d_witness_rows(
            source_id,
            witnesses,
            statements,
            primary_terms,
            set_terms,
            arena,
        );
    }
    #[cfg(test)]
    if profile == WitnessProfile::Task258B3M2B2B3E {
        return validate_task258b3m2b2b3e_witness_rows(
            source_id,
            witnesses,
            statements,
            primary_terms,
            set_terms,
            arena,
        );
    }
    #[cfg(test)]
    let Some(theorem) = statements.statements().get(SourceStatementId::new(0)) else {
        return Err(SourceStatementWitnessError::InvalidWitness {
            witness: SourceStatementWitnessId::new(0),
        });
    };
    #[cfg(test)]
    let Some(conclusion) = statements.statements().get(SourceStatementId::new(1)) else {
        return Err(SourceStatementWitnessError::InvalidWitness {
            witness: SourceStatementWitnessId::new(0),
        });
    };
    #[cfg(test)]
    let witness_count = match profile {
        #[cfg(test)]
        WitnessProfile::Task258B3 => 1,
        #[cfg(test)]
        WitnessProfile::Task258B3N => 1,
        #[cfg(test)]
        WitnessProfile::Task258B3M2B1 => 1,
        #[cfg(test)]
        WitnessProfile::Task258B3M2A => 1,
        #[cfg(test)]
        WitnessProfile::Task258B3M2B2A => 1,
        #[cfg(test)]
        WitnessProfile::Task258B3M1 => 2,
        #[cfg(test)]
        WitnessProfile::Task258B3M2B2B1A => {
            unreachable!("application witness returns above")
        }
        #[cfg(test)]
        WitnessProfile::Task258B3M2B2B1B1 => {
            unreachable!("wrapped application witness returns above")
        }
        WitnessProfile::Task258B3M2B2B2A => {
            unreachable!("structure witness returns above")
        }
        WitnessProfile::Task258B3M2B2B2B => {
            unreachable!("structure selector witness returns above")
        }
        #[cfg(test)]
        WitnessProfile::Task258B3M2B2B2C => {
            unreachable!("structure update witness returns above")
        }
        #[cfg(test)]
        WitnessProfile::Task258B3M2B2B3A => {
            unreachable!("set enumeration witness returns above")
        }
        #[cfg(test)]
        WitnessProfile::Task258B3M2B2B3B => {
            unreachable!("empty set enumeration witness returns above")
        }
        #[cfg(test)]
        WitnessProfile::Task258B3M2B2B3C => {
            unreachable!("choice witness returns above")
        }
        #[cfg(test)]
        WitnessProfile::Task258B3M2B2B3D => {
            unreachable!("qua witness returns above")
        }
        #[cfg(test)]
        WitnessProfile::Task258B3M2B2B3E => {
            unreachable!("comprehension witness returns above")
        }
    };
    #[cfg(test)]
    for index in 0..witness_count {
        let id = SourceStatementWitnessId::new(index);
        let Some(witness) = witnesses.get(id) else {
            return Err(SourceStatementWitnessError::InvalidAggregate);
        };
        let invalid = || SourceStatementWitnessError::InvalidWitness { witness: id };
        let expected = match (profile, index) {
            #[cfg(test)]
            (WitnessProfile::Task258B3, 0) => ExpectedWitnessRow {
                term_index: 2,
                take_site: 35,
                take_range: range(source_id, 77, 84),
                item_site: 34,
                item_range: range(source_id, 82, 83),
                item_spelling: "x",
                item_kind: SourceStatementWitnessKind::Unnamed,
                item_name: None,
                term_site: 32,
                term_range: range(source_id, 82, 83),
                wrapper_site: 33,
                take_children: &[12, 34, 14],
                item_children: &[33],
            },
            #[cfg(test)]
            (WitnessProfile::Task258B3N, 0) => ExpectedWitnessRow {
                term_index: 2,
                take_site: 37,
                take_range: range(source_id, 76, 87),
                item_site: 36,
                item_range: range(source_id, 81, 86),
                item_spelling: "y = x",
                item_kind: SourceStatementWitnessKind::Named,
                item_name: Some(SourceStatementWitnessNameId::new(0)),
                term_site: 34,
                term_range: range(source_id, 85, 86),
                wrapper_site: 35,
                take_children: &[12, 36, 16],
                item_children: &[13, 14, 35],
            },
            #[cfg(test)]
            (WitnessProfile::Task258B3M1, 0) => ExpectedWitnessRow {
                term_index: 2,
                take_site: 42,
                take_range: range(source_id, 79, 93),
                item_site: 38,
                item_range: range(source_id, 84, 89),
                item_spelling: "y = x",
                item_kind: SourceStatementWitnessKind::Named,
                item_name: Some(SourceStatementWitnessNameId::new(0)),
                term_site: 36,
                term_range: range(source_id, 88, 89),
                wrapper_site: 37,
                take_children: &[12, 38, 16, 41, 18],
                item_children: &[13, 14, 37],
            },
            #[cfg(test)]
            (WitnessProfile::Task258B3M1, 1) => ExpectedWitnessRow {
                term_index: 3,
                take_site: 42,
                take_range: range(source_id, 79, 93),
                item_site: 41,
                item_range: range(source_id, 91, 92),
                item_spelling: "x",
                item_kind: SourceStatementWitnessKind::Unnamed,
                item_name: None,
                term_site: 39,
                term_range: range(source_id, 91, 92),
                wrapper_site: 40,
                take_children: &[12, 38, 16, 41, 18],
                item_children: &[40],
            },
            #[cfg(test)]
            (WitnessProfile::Task258B3M2A, 0) => ExpectedWitnessRow {
                term_index: 2,
                take_site: 35,
                take_range: range(source_id, 78, 87),
                item_site: 34,
                item_range: range(source_id, 83, 86),
                item_spelling: "101",
                item_kind: SourceStatementWitnessKind::Unnamed,
                item_name: None,
                term_site: 32,
                term_range: range(source_id, 83, 86),
                wrapper_site: 33,
                take_children: &[12, 34, 14],
                item_children: &[33],
            },
            #[cfg(test)]
            (WitnessProfile::Task258B3M2B1, 0) => ExpectedWitnessRow {
                term_index: 2,
                take_site: 39,
                take_range: range(source_id, 84, 93),
                item_site: 38,
                item_range: range(source_id, 89, 92),
                item_spelling: "( x )",
                item_kind: SourceStatementWitnessKind::Unnamed,
                item_name: None,
                term_site: 36,
                term_range: range(source_id, 89, 92),
                wrapper_site: 37,
                take_children: &[12, 38, 16],
                item_children: &[37],
            },
            #[cfg(test)]
            (WitnessProfile::Task258B3M2B2A, 0) => ExpectedWitnessRow {
                term_index: 2,
                take_site: 43,
                take_range: range(source_id, 90, 101),
                item_site: 42,
                item_range: range(source_id, 95, 100),
                item_spelling: "( ( x ) )",
                item_kind: SourceStatementWitnessKind::Unnamed,
                item_name: None,
                term_site: 40,
                term_range: range(source_id, 95, 100),
                wrapper_site: 41,
                take_children: &[12, 42, 18],
                item_children: &[41],
            },
            _ => unreachable!("witness count matches the frozen profile"),
        };
        let term_id = SourcePrimaryTermId::new(expected.term_index);
        let expected_term = SourceStatementWitnessTermTarget::Primary(term_id);
        let Some(term) = primary_terms.terms().get(term_id) else {
            return Err(invalid());
        };
        let numeral_witness = profile == WitnessProfile::Task258B3M2A;
        #[cfg(not(test))]
        let parenthesized_witness = false;
        #[cfg(test)]
        let parenthesized_witness = matches!(
            profile,
            WitnessProfile::Task258B3M2B2A | WitnessProfile::Task258B3M2B1
        );
        let reference_valid = if numeral_witness || parenthesized_witness {
            primary_terms
                .references()
                .iter()
                .all(|(_, reference)| reference.term() != term_id)
        } else {
            primary_terms
                .references()
                .get(SourcePrimaryTermReferenceId::new(expected.term_index))
                .is_some_and(|reference| {
                    reference.term() == term_id
                        && reference.binding() == BindingId::new(0)
                        && reference.role() == SourcePrimaryTermReferenceRole::Variable
                        && reference
                            .lexical_scope()
                            .is_some_and(|scope| scope.path() == [0])
                        && reference.use_ordinal() == 1
                })
        };
        let Some(take) = arena.node(witness.take_site.node()) else {
            return Err(invalid());
        };
        let Some(item) = arena.node(witness.site.node()) else {
            return Err(invalid());
        };
        let Some(wrapper) = arena.node(crate::typed_ast::TypedNodeId::new(expected.wrapper_site))
        else {
            return Err(invalid());
        };
        if witness.owner != SourceTheoremOwnerId::new(0)
            || statements.owners().get(witness.owner).is_none()
            || witness.binding_context != BindingContextId::new(1)
            || witness.term != expected_term
            || !matches!(&witness.take_site, TypedSiteRef::Node(_))
            || witness.take_site.node().index() != expected.take_site
            || witness.take_range != expected.take_range
            || !matches!(&witness.site, TypedSiteRef::Node(_))
            || witness.site.node().index() != expected.item_site
            || witness.source_range != expected.item_range
            || witness.source_ordinal != 1
            || witness.ordinal != index
            || witness.spelling != expected.item_spelling
            || witness.kind != expected.item_kind
            || witness.recovery != SourceStatementRecovery::Normal
            || witness.name != expected.item_name
            || theorem.source_ordinal() != 0
            || conclusion.source_ordinal() != 2
            || term.site().node().index() != expected.term_site
            || term.source_range() != expected.term_range
            || term.context() != witness.binding_context
            || term.source_ordinal() != expected.term_index
            || term.spelling()
                != if numeral_witness {
                    "101"
                } else if parenthesized_witness {
                    expected.item_spelling
                } else {
                    "x"
                }
            || term.kind()
                != if numeral_witness {
                    SourcePrimaryTermKind::Numeral
                } else if parenthesized_witness {
                    SourcePrimaryTermKind::Parenthesized
                } else {
                    SourcePrimaryTermKind::VariableReference
                }
            || term.role() != SourcePrimaryTermRole::Value
            || term.recovery() != SourcePrimaryTermRecovery::Normal
            || term.parent().is_some()
            || !reference_valid
            || take.anchor != SourceAnchor::Range(witness.take_range)
            || take.kind.as_str() != "source.statement-witness.take"
            || take.recovery != NodeRecoveryState::Normal
            || take
                .children
                .iter()
                .map(|child| child.index())
                .ne(expected.take_children.iter().copied())
            || item.anchor != SourceAnchor::Range(witness.source_range)
            || item.kind.as_str() != "source.statement-witness.item"
            || item.recovery != NodeRecoveryState::Normal
            || item
                .children
                .iter()
                .map(|child| child.index())
                .ne(expected.item_children.iter().copied())
            || wrapper.anchor != SourceAnchor::Range(expected.term_range)
            || wrapper.kind.as_str() != "source.surface.unowned"
            || wrapper.recovery != NodeRecoveryState::Normal
            || wrapper.children != [term.site().node()]
            || !is_descendant(arena, theorem.site().node(), witness.take_site.node())
            || !is_descendant(arena, theorem.site().node(), witness.site.node())
            || is_descendant(arena, conclusion.site().node(), witness.take_site.node())
            || is_descendant(arena, conclusion.site().node(), witness.site.node())
        {
            return Err(invalid());
        }
    }
    #[cfg(test)]
    let expected_source_ordinals: &[usize] = match profile {
        #[cfg(test)]
        WitnessProfile::Task258B3 => &[0, 1, 2],
        #[cfg(test)]
        WitnessProfile::Task258B3N => &[0, 1, 2],
        #[cfg(test)]
        WitnessProfile::Task258B3M2B1 => &[0, 1, 2],
        #[cfg(test)]
        WitnessProfile::Task258B3M2A => &[0, 1, 2],
        #[cfg(test)]
        WitnessProfile::Task258B3M2B2A => &[0, 1, 2],
        #[cfg(test)]
        WitnessProfile::Task258B3M1 => &[0, 1, 1, 2],
        #[cfg(test)]
        WitnessProfile::Task258B3M2B2B1A => {
            unreachable!("application witness returns above")
        }
        #[cfg(test)]
        WitnessProfile::Task258B3M2B2B1B1 => {
            unreachable!("wrapped application witness returns above")
        }
        WitnessProfile::Task258B3M2B2B2A => {
            unreachable!("structure witness returns above")
        }
        WitnessProfile::Task258B3M2B2B2B => {
            unreachable!("structure selector witness returns above")
        }
        #[cfg(test)]
        WitnessProfile::Task258B3M2B2B2C => {
            unreachable!("structure update witness returns above")
        }
        #[cfg(test)]
        WitnessProfile::Task258B3M2B2B3A => {
            unreachable!("set enumeration witness returns above")
        }
        #[cfg(test)]
        WitnessProfile::Task258B3M2B2B3B => {
            unreachable!("empty set enumeration witness returns above")
        }
        #[cfg(test)]
        WitnessProfile::Task258B3M2B2B3C => {
            unreachable!("choice witness returns above")
        }
        #[cfg(test)]
        WitnessProfile::Task258B3M2B2B3D => {
            unreachable!("qua witness returns above")
        }
        #[cfg(test)]
        WitnessProfile::Task258B3M2B2B3E => {
            unreachable!("comprehension witness returns above")
        }
    };
    #[cfg(test)]
    let actual_source_ordinals = std::iter::once(theorem.source_ordinal())
        .chain(witnesses.iter().map(|(_, witness)| witness.source_ordinal))
        .chain(std::iter::once(conclusion.source_ordinal()));
    #[cfg(test)]
    let expected_witness_nodes = match profile {
        #[cfg(test)]
        WitnessProfile::Task258B3 => 2,
        #[cfg(test)]
        WitnessProfile::Task258B3N => 3,
        #[cfg(test)]
        WitnessProfile::Task258B3M1 => 4,
        #[cfg(test)]
        WitnessProfile::Task258B3M2A => 2,
        #[cfg(test)]
        WitnessProfile::Task258B3M2B1 => 2,
        #[cfg(test)]
        WitnessProfile::Task258B3M2B2A => 2,
        #[cfg(test)]
        WitnessProfile::Task258B3M2B2B1A => {
            unreachable!("application witness returns above")
        }
        #[cfg(test)]
        WitnessProfile::Task258B3M2B2B1B1 => {
            unreachable!("wrapped application witness returns above")
        }
        WitnessProfile::Task258B3M2B2B2A => {
            unreachable!("structure witness returns above")
        }
        WitnessProfile::Task258B3M2B2B2B => {
            unreachable!("structure selector witness returns above")
        }
        #[cfg(test)]
        WitnessProfile::Task258B3M2B2B2C => {
            unreachable!("structure update witness returns above")
        }
        #[cfg(test)]
        WitnessProfile::Task258B3M2B2B3A => {
            unreachable!("set enumeration witness returns above")
        }
        #[cfg(test)]
        WitnessProfile::Task258B3M2B2B3B => {
            unreachable!("empty set enumeration witness returns above")
        }
        #[cfg(test)]
        WitnessProfile::Task258B3M2B2B3C => {
            unreachable!("choice witness returns above")
        }
        #[cfg(test)]
        WitnessProfile::Task258B3M2B2B3D => {
            unreachable!("qua witness returns above")
        }
        #[cfg(test)]
        WitnessProfile::Task258B3M2B2B3E => {
            unreachable!("comprehension witness returns above")
        }
    };
    #[cfg(test)]
    if actual_source_ordinals.ne(expected_source_ordinals.iter().copied())
        || arena
            .iter()
            .filter(|(_, node)| node.kind.as_str().starts_with("source.statement-witness."))
            .count()
            != expected_witness_nodes
    {
        return Err(SourceStatementWitnessError::InvalidWitness {
            witness: SourceStatementWitnessId::new(0),
        });
    }
    Ok(())
}

#[cfg(test)]
fn validate_task258b3m2b2b3a_witness_rows(
    source_id: SourceId,
    witnesses: &SourceStatementWitnessTable,
    statements: &SourceStatementHandoff,
    primary_terms: &SourcePrimaryTermHandoff,
    set_terms: Option<&SourceSetTermHandoff>,
    arena: &TypedArena,
) -> Result<(), SourceStatementWitnessError> {
    let id = SourceStatementWitnessId::new(0);
    let invalid = || SourceStatementWitnessError::InvalidWitness { witness: id };
    let Some(witness) = witnesses.get(id) else {
        return Err(SourceStatementWitnessError::InvalidAggregate);
    };
    let Some(set_terms) = set_terms else {
        return Err(SourceStatementWitnessError::DependencyMismatch);
    };
    if !exact_task258b3m2b2b3a_set_profile(
        source_id,
        statements.module_id(),
        primary_terms,
        set_terms,
        arena,
    ) {
        return Err(SourceStatementWitnessError::DependencyMismatch);
    }
    let Some(theorem) = statements.statements().get(SourceStatementId::new(0)) else {
        return Err(invalid());
    };
    let Some(conclusion) = statements.statements().get(SourceStatementId::new(1)) else {
        return Err(invalid());
    };
    let Some(take) = arena.node(witness.take_site.node()) else {
        return Err(invalid());
    };
    let Some(item) = arena.node(witness.site.node()) else {
        return Err(invalid());
    };
    let Some(transparent) = arena.node(crate::typed_ast::TypedNodeId::new(41)) else {
        return Err(invalid());
    };
    let Some(set_node) = arena.node(crate::typed_ast::TypedNodeId::new(40)) else {
        return Err(invalid());
    };
    if witness.owner != SourceTheoremOwnerId::new(0)
        || witness.binding_context != BindingContextId::new(1)
        || witness.term != SourceStatementWitnessTermTarget::SetTerm(SourceSetTermId::new(0))
        || witness.take_site.node().index() != 43
        || witness.take_range != range(source_id, 85, 97)
        || witness.site.node().index() != 42
        || witness.source_range != range(source_id, 90, 96)
        || witness.source_ordinal != 1
        || witness.ordinal != 0
        || witness.spelling != "{ 1 , 2 }"
        || witness.kind != SourceStatementWitnessKind::Unnamed
        || witness.recovery != SourceStatementRecovery::Normal
        || witness.name.is_some()
        || theorem.source_ordinal() != 0
        || conclusion.source_ordinal() != 2
        || take.anchor != SourceAnchor::Range(witness.take_range)
        || take.kind.as_str() != "source.statement-witness.take"
        || take.recovery != NodeRecoveryState::Normal
        || take.children
            != [
                crate::typed_ast::TypedNodeId::new(12),
                witness.site.node(),
                crate::typed_ast::TypedNodeId::new(18),
            ]
        || item.anchor != SourceAnchor::Range(witness.source_range)
        || item.kind.as_str() != "source.statement-witness.item"
        || item.recovery != NodeRecoveryState::Normal
        || item.children != [crate::typed_ast::TypedNodeId::new(41)]
        || transparent.anchor != SourceAnchor::Range(witness.source_range)
        || transparent.kind.as_str() != "source.surface.unowned"
        || transparent.recovery != NodeRecoveryState::Normal
        || transparent.children != [crate::typed_ast::TypedNodeId::new(40)]
        || set_node.anchor != SourceAnchor::Range(witness.source_range)
        || set_node.kind.as_str() != "source.term.set.enumeration"
        || set_node.recovery != NodeRecoveryState::Normal
        || !is_descendant(arena, theorem.site().node(), witness.take_site.node())
        || !is_descendant(arena, theorem.site().node(), witness.site.node())
        || !is_descendant(
            arena,
            witness.site.node(),
            crate::typed_ast::TypedNodeId::new(40),
        )
        || is_descendant(arena, conclusion.site().node(), witness.site.node())
        || arena
            .iter()
            .filter(|(_, node)| node.kind.as_str().starts_with("source.statement-witness."))
            .count()
            != 2
    {
        return Err(invalid());
    }
    Ok(())
}

#[cfg(test)]
fn validate_task258b3m2b2b3b_witness_rows(
    source_id: SourceId,
    witnesses: &SourceStatementWitnessTable,
    statements: &SourceStatementHandoff,
    primary_terms: &SourcePrimaryTermHandoff,
    set_terms: Option<&SourceSetTermHandoff>,
    arena: &TypedArena,
) -> Result<(), SourceStatementWitnessError> {
    let id = SourceStatementWitnessId::new(0);
    let invalid = || SourceStatementWitnessError::InvalidWitness { witness: id };
    let Some(witness) = witnesses.get(id) else {
        return Err(SourceStatementWitnessError::InvalidAggregate);
    };
    let Some(set_terms) = set_terms else {
        return Err(SourceStatementWitnessError::DependencyMismatch);
    };
    if !exact_task258b3m2b2b3b_set_profile(
        source_id,
        statements.module_id(),
        primary_terms,
        set_terms,
        arena,
    ) {
        return Err(SourceStatementWitnessError::DependencyMismatch);
    }
    let Some(theorem) = statements.statements().get(SourceStatementId::new(0)) else {
        return Err(invalid());
    };
    let Some(conclusion) = statements.statements().get(SourceStatementId::new(1)) else {
        return Err(invalid());
    };
    let Some(take) = arena.node(witness.take_site.node()) else {
        return Err(invalid());
    };
    let Some(item) = arena.node(witness.site.node()) else {
        return Err(invalid());
    };
    let Some(transparent) = arena.node(crate::typed_ast::TypedNodeId::new(34)) else {
        return Err(invalid());
    };
    let Some(set_node) = arena.node(crate::typed_ast::TypedNodeId::new(33)) else {
        return Err(invalid());
    };
    if witness.owner != SourceTheoremOwnerId::new(0)
        || witness.binding_context != BindingContextId::new(1)
        || witness.term != SourceStatementWitnessTermTarget::SetTerm(SourceSetTermId::new(0))
        || witness.take_site.node().index() != 36
        || witness.take_range != range(source_id, 90, 98)
        || witness.site.node().index() != 35
        || witness.source_range != range(source_id, 95, 97)
        || witness.source_ordinal != 1
        || witness.ordinal != 0
        || witness.spelling != "{ }"
        || witness.kind != SourceStatementWitnessKind::Unnamed
        || witness.recovery != SourceStatementRecovery::Normal
        || witness.name.is_some()
        || theorem.source_ordinal() != 0
        || conclusion.source_ordinal() != 2
        || take.anchor != SourceAnchor::Range(witness.take_range)
        || take.kind.as_str() != "source.statement-witness.take"
        || take.recovery != NodeRecoveryState::Normal
        || take.children
            != [
                crate::typed_ast::TypedNodeId::new(12),
                witness.site.node(),
                crate::typed_ast::TypedNodeId::new(15),
            ]
        || item.anchor != SourceAnchor::Range(witness.source_range)
        || item.kind.as_str() != "source.statement-witness.item"
        || item.recovery != NodeRecoveryState::Normal
        || item.children != [crate::typed_ast::TypedNodeId::new(34)]
        || transparent.anchor != SourceAnchor::Range(witness.source_range)
        || transparent.kind.as_str() != "source.surface.unowned"
        || transparent.recovery != NodeRecoveryState::Normal
        || transparent.children != [crate::typed_ast::TypedNodeId::new(33)]
        || set_node.anchor != SourceAnchor::Range(witness.source_range)
        || set_node.kind.as_str() != "source.term.set.enumeration"
        || set_node.recovery != NodeRecoveryState::Normal
        || !is_descendant(arena, theorem.site().node(), witness.take_site.node())
        || !is_descendant(arena, theorem.site().node(), witness.site.node())
        || !is_descendant(
            arena,
            witness.site.node(),
            crate::typed_ast::TypedNodeId::new(33),
        )
        || is_descendant(arena, conclusion.site().node(), witness.site.node())
        || arena
            .iter()
            .filter(|(_, node)| node.kind.as_str().starts_with("source.statement-witness."))
            .count()
            != 2
    {
        return Err(invalid());
    }
    Ok(())
}

#[cfg(test)]
fn validate_task258b3m2b2b3c_witness_rows(
    source_id: SourceId,
    witnesses: &SourceStatementWitnessTable,
    statements: &SourceStatementHandoff,
    primary_terms: &SourcePrimaryTermHandoff,
    set_terms: Option<&SourceSetTermHandoff>,
    arena: &TypedArena,
) -> Result<(), SourceStatementWitnessError> {
    let id = SourceStatementWitnessId::new(0);
    let invalid = || SourceStatementWitnessError::InvalidWitness { witness: id };
    let Some(witness) = witnesses.get(id) else {
        return Err(SourceStatementWitnessError::InvalidAggregate);
    };
    let Some(set_terms) = set_terms else {
        return Err(SourceStatementWitnessError::DependencyMismatch);
    };
    if !exact_task258b3m2b2b3c_set_profile(
        source_id,
        statements.module_id(),
        primary_terms,
        set_terms,
        arena,
    ) {
        return Err(SourceStatementWitnessError::DependencyMismatch);
    }
    let Some(theorem) = statements.statements().get(SourceStatementId::new(0)) else {
        return Err(invalid());
    };
    let Some(conclusion) = statements.statements().get(SourceStatementId::new(1)) else {
        return Err(invalid());
    };
    let Some(take) = arena.node(witness.take_site.node()) else {
        return Err(invalid());
    };
    let Some(item) = arena.node(witness.site.node()) else {
        return Err(invalid());
    };
    let Some(transparent) = arena.node(crate::typed_ast::TypedNodeId::new(36)) else {
        return Err(invalid());
    };
    let Some(choice_node) = arena.node(crate::typed_ast::TypedNodeId::new(35)) else {
        return Err(invalid());
    };
    if witness.owner != SourceTheoremOwnerId::new(0)
        || witness.binding_context != BindingContextId::new(1)
        || witness.term != SourceStatementWitnessTermTarget::SetTerm(SourceSetTermId::new(0))
        || witness.take_site.node().index() != 38
        || witness.take_range != range(source_id, 77, 90)
        || witness.site.node().index() != 37
        || witness.source_range != range(source_id, 82, 89)
        || witness.source_ordinal != 1
        || witness.ordinal != 0
        || witness.spelling != "the set"
        || witness.kind != SourceStatementWitnessKind::Unnamed
        || witness.recovery != SourceStatementRecovery::Normal
        || witness.name.is_some()
        || theorem.source_ordinal() != 0
        || conclusion.source_ordinal() != 2
        || take.anchor != SourceAnchor::Range(witness.take_range)
        || take.kind.as_str() != "source.statement-witness.take"
        || take.recovery != NodeRecoveryState::Normal
        || take.children
            != [
                crate::typed_ast::TypedNodeId::new(12),
                witness.site.node(),
                crate::typed_ast::TypedNodeId::new(15),
            ]
        || item.anchor != SourceAnchor::Range(witness.source_range)
        || item.kind.as_str() != "source.statement-witness.item"
        || item.recovery != NodeRecoveryState::Normal
        || item.children != [crate::typed_ast::TypedNodeId::new(36)]
        || transparent.anchor != SourceAnchor::Range(witness.source_range)
        || transparent.kind.as_str() != "source.surface.unowned"
        || transparent.recovery != NodeRecoveryState::Normal
        || transparent.children != [crate::typed_ast::TypedNodeId::new(35)]
        || choice_node.anchor != SourceAnchor::Range(witness.source_range)
        || choice_node.kind.as_str() != "source.term.set.choice"
        || choice_node.recovery != NodeRecoveryState::Normal
        || !is_descendant(arena, theorem.site().node(), witness.take_site.node())
        || !is_descendant(arena, theorem.site().node(), witness.site.node())
        || !is_descendant(
            arena,
            witness.site.node(),
            crate::typed_ast::TypedNodeId::new(35),
        )
        || is_descendant(arena, conclusion.site().node(), witness.site.node())
        || arena
            .iter()
            .filter(|(_, node)| node.kind.as_str().starts_with("source.statement-witness."))
            .count()
            != 2
    {
        return Err(invalid());
    }
    Ok(())
}

#[cfg(test)]
fn validate_task258b3m2b2b3d_witness_rows(
    source_id: SourceId,
    witnesses: &SourceStatementWitnessTable,
    statements: &SourceStatementHandoff,
    primary_terms: &SourcePrimaryTermHandoff,
    set_terms: Option<&SourceSetTermHandoff>,
    arena: &TypedArena,
) -> Result<(), SourceStatementWitnessError> {
    let id = SourceStatementWitnessId::new(0);
    let invalid = || SourceStatementWitnessError::InvalidWitness { witness: id };
    let Some(witness) = witnesses.get(id) else {
        return Err(SourceStatementWitnessError::InvalidAggregate);
    };
    let Some(set_terms) = set_terms else {
        return Err(SourceStatementWitnessError::DependencyMismatch);
    };
    if !exact_task258b3m2b2b3d_set_profile(
        source_id,
        statements.module_id(),
        primary_terms,
        set_terms,
        arena,
    ) {
        return Err(SourceStatementWitnessError::DependencyMismatch);
    }
    let Some(theorem) = statements.statements().get(SourceStatementId::new(0)) else {
        return Err(invalid());
    };
    let Some(conclusion) = statements.statements().get(SourceStatementId::new(1)) else {
        return Err(invalid());
    };
    let Some(take) = arena.node(witness.take_site.node()) else {
        return Err(invalid());
    };
    let Some(item) = arena.node(witness.site.node()) else {
        return Err(invalid());
    };
    let Some(transparent) = arena.node(crate::typed_ast::TypedNodeId::new(38)) else {
        return Err(invalid());
    };
    let Some(qua_node) = arena.node(crate::typed_ast::TypedNodeId::new(37)) else {
        return Err(invalid());
    };
    if witness.owner != SourceTheoremOwnerId::new(0)
        || witness.binding_context != BindingContextId::new(1)
        || witness.term != SourceStatementWitnessTermTarget::SetTerm(SourceSetTermId::new(0))
        || witness.take_site.node().index() != 40
        || witness.take_range != range(source_id, 74, 89)
        || witness.site.node().index() != 39
        || witness.source_range != range(source_id, 79, 88)
        || witness.source_ordinal != 1
        || witness.ordinal != 0
        || witness.spelling != "4 qua set"
        || witness.kind != SourceStatementWitnessKind::Unnamed
        || witness.recovery != SourceStatementRecovery::Normal
        || witness.name.is_some()
        || theorem.source_ordinal() != 0
        || conclusion.source_ordinal() != 2
        || take.anchor != SourceAnchor::Range(witness.take_range)
        || take.kind.as_str() != "source.statement-witness.take"
        || take.recovery != NodeRecoveryState::Normal
        || take.children
            != [
                crate::typed_ast::TypedNodeId::new(12),
                witness.site.node(),
                crate::typed_ast::TypedNodeId::new(16),
            ]
        || item.anchor != SourceAnchor::Range(witness.source_range)
        || item.kind.as_str() != "source.statement-witness.item"
        || item.recovery != NodeRecoveryState::Normal
        || item.children != [crate::typed_ast::TypedNodeId::new(38)]
        || transparent.anchor != SourceAnchor::Range(witness.source_range)
        || transparent.kind.as_str() != "source.surface.unowned"
        || transparent.recovery != NodeRecoveryState::Normal
        || transparent.children != [crate::typed_ast::TypedNodeId::new(37)]
        || qua_node.anchor != SourceAnchor::Range(witness.source_range)
        || qua_node.kind.as_str() != "source.term.set.qua"
        || qua_node.recovery != NodeRecoveryState::Normal
        || !is_descendant(arena, theorem.site().node(), witness.take_site.node())
        || !is_descendant(arena, theorem.site().node(), witness.site.node())
        || !is_descendant(
            arena,
            witness.site.node(),
            crate::typed_ast::TypedNodeId::new(37),
        )
        || is_descendant(arena, conclusion.site().node(), witness.site.node())
        || arena
            .iter()
            .filter(|(_, node)| node.kind.as_str().starts_with("source.statement-witness."))
            .count()
            != 2
    {
        return Err(invalid());
    }
    Ok(())
}

#[cfg(test)]
fn validate_task258b3m2b2b3e_witness_rows(
    source_id: SourceId,
    witnesses: &SourceStatementWitnessTable,
    statements: &SourceStatementHandoff,
    primary_terms: &SourcePrimaryTermHandoff,
    set_terms: Option<&SourceSetTermHandoff>,
    arena: &TypedArena,
) -> Result<(), SourceStatementWitnessError> {
    let id = SourceStatementWitnessId::new(0);
    let invalid = || SourceStatementWitnessError::InvalidWitness { witness: id };
    let Some(witness) = witnesses.get(id) else {
        return Err(SourceStatementWitnessError::InvalidAggregate);
    };
    let Some(set_terms) = set_terms else {
        return Err(SourceStatementWitnessError::DependencyMismatch);
    };
    if !exact_task258b3m2b2b3e_set_profile(
        source_id,
        statements.module_id(),
        primary_terms,
        set_terms,
        arena,
    ) {
        return Err(SourceStatementWitnessError::DependencyMismatch);
    }
    let Some(theorem) = statements.statements().get(SourceStatementId::new(0)) else {
        return Err(invalid());
    };
    let Some(conclusion) = statements.statements().get(SourceStatementId::new(1)) else {
        return Err(invalid());
    };
    let Some(take) = arena.node(witness.take_site.node()) else {
        return Err(invalid());
    };
    let Some(item) = arena.node(witness.site.node()) else {
        return Err(invalid());
    };
    let Some(transparent) = arena.node(crate::typed_ast::TypedNodeId::new(44)) else {
        return Err(invalid());
    };
    let Some(comprehension) = arena.node(crate::typed_ast::TypedNodeId::new(43)) else {
        return Err(invalid());
    };
    let Some(generator_segment) = arena.node(crate::typed_ast::TypedNodeId::new(42)) else {
        return Err(invalid());
    };
    if witness.owner != SourceTheoremOwnerId::new(0)
        || witness.binding_context != BindingContextId::new(1)
        || witness.term != SourceStatementWitnessTermTarget::SetTerm(SourceSetTermId::new(0))
        || witness.take_site.node().index() != 46
        || witness.take_range != range(source_id, 84, 119)
        || witness.site.node().index() != 45
        || witness.source_range != range(source_id, 89, 118)
        || witness.source_ordinal != 1
        || witness.ordinal != 0
        || witness.spelling != "{ 3 where candidate255 is set }"
        || witness.kind != SourceStatementWitnessKind::Unnamed
        || witness.recovery != SourceStatementRecovery::Normal
        || witness.name.is_some()
        || theorem.source_ordinal() != 0
        || conclusion.source_ordinal() != 2
        || take.anchor != SourceAnchor::Range(witness.take_range)
        || take.kind.as_str() != "source.statement-witness.take"
        || take.recovery != NodeRecoveryState::Normal
        || take.children
            != [
                crate::typed_ast::TypedNodeId::new(12),
                witness.site.node(),
                crate::typed_ast::TypedNodeId::new(20),
            ]
        || item.anchor != SourceAnchor::Range(witness.source_range)
        || item.kind.as_str() != "source.statement-witness.item"
        || item.recovery != NodeRecoveryState::Normal
        || item.children != [crate::typed_ast::TypedNodeId::new(44)]
        || transparent.anchor != SourceAnchor::Range(witness.source_range)
        || transparent.kind.as_str() != "source.surface.unowned"
        || transparent.recovery != NodeRecoveryState::Normal
        || transparent.children != [crate::typed_ast::TypedNodeId::new(43)]
        || comprehension.anchor != SourceAnchor::Range(witness.source_range)
        || comprehension.kind.as_str() != "source.term.set.comprehension"
        || comprehension.recovery != NodeRecoveryState::Normal
        || generator_segment.anchor != SourceAnchor::Range(range(source_id, 98, 117))
        || generator_segment.kind.as_str() != "source.surface.unowned"
        || generator_segment.recovery != NodeRecoveryState::Normal
        || generator_segment.children
            != [
                crate::typed_ast::TypedNodeId::new(16),
                crate::typed_ast::TypedNodeId::new(17),
                crate::typed_ast::TypedNodeId::new(41),
            ]
        || !is_descendant(arena, theorem.site().node(), witness.take_site.node())
        || !is_descendant(arena, theorem.site().node(), witness.site.node())
        || !is_descendant(
            arena,
            witness.site.node(),
            crate::typed_ast::TypedNodeId::new(43),
        )
        || !is_descendant(
            arena,
            witness.site.node(),
            crate::typed_ast::TypedNodeId::new(16),
        )
        || !is_descendant(
            arena,
            witness.site.node(),
            crate::typed_ast::TypedNodeId::new(40),
        )
        || !is_descendant(
            arena,
            witness.site.node(),
            crate::typed_ast::TypedNodeId::new(41),
        )
        || is_descendant(arena, conclusion.site().node(), witness.site.node())
        || arena
            .iter()
            .filter(|(_, node)| node.kind.as_str().starts_with("source.statement-witness."))
            .count()
            != 2
    {
        return Err(invalid());
    }
    Ok(())
}

#[cfg(test)]
fn validate_task258b3m2b2b1a_witness_rows(
    source_id: SourceId,
    witnesses: &SourceStatementWitnessTable,
    statements: &SourceStatementHandoff,
    primary_terms: &SourcePrimaryTermHandoff,
    application: Option<&SourceFunctorApplicationHandoff>,
    arena: &TypedArena,
) -> Result<(), SourceStatementWitnessError> {
    let id = SourceStatementWitnessId::new(0);
    let invalid = || SourceStatementWitnessError::InvalidWitness { witness: id };
    let Some(witness) = witnesses.get(id) else {
        return Err(SourceStatementWitnessError::InvalidAggregate);
    };
    let Some(application) = application else {
        return Err(SourceStatementWitnessError::DependencyMismatch);
    };
    if !exact_task258b3m2b2b1a_application_profile(
        source_id,
        statements.module_id(),
        primary_terms,
        application,
    ) {
        return Err(SourceStatementWitnessError::DependencyMismatch);
    }
    let Some(theorem) = statements.statements().get(SourceStatementId::new(0)) else {
        return Err(invalid());
    };
    let Some(conclusion) = statements.statements().get(SourceStatementId::new(1)) else {
        return Err(invalid());
    };
    let Some(take) = arena.node(witness.take_site.node()) else {
        return Err(invalid());
    };
    let Some(item) = arena.node(witness.site.node()) else {
        return Err(invalid());
    };
    let Some(transparent) = arena.node(crate::typed_ast::TypedNodeId::new(47)) else {
        return Err(invalid());
    };
    let Some(application_node) = arena.node(crate::typed_ast::TypedNodeId::new(46)) else {
        return Err(invalid());
    };
    if witness.owner != SourceTheoremOwnerId::new(0)
        || witness.binding_context != BindingContextId::new(1)
        || witness.term
            != SourceStatementWitnessTermTarget::Application(SourceFunctorApplicationId::new(0))
        || witness.take_site.node().index() != 49
        || witness.take_range != range(source_id, 111, 123)
        || witness.site.node().index() != 48
        || witness.source_range != range(source_id, 116, 122)
        || witness.source_ordinal != 1
        || witness.ordinal != 0
        || witness.spelling != "1 ++ 2"
        || witness.kind != SourceStatementWitnessKind::Unnamed
        || witness.recovery != SourceStatementRecovery::Normal
        || witness.name.is_some()
        || theorem.source_ordinal() != 0
        || conclusion.source_ordinal() != 2
        || take.anchor != SourceAnchor::Range(witness.take_range)
        || take.kind.as_str() != "source.statement-witness.take"
        || take.recovery != NodeRecoveryState::Normal
        || take.children
            != [
                crate::typed_ast::TypedNodeId::new(17),
                witness.site.node(),
                crate::typed_ast::TypedNodeId::new(21),
            ]
        || item.anchor != SourceAnchor::Range(witness.source_range)
        || item.kind.as_str() != "source.statement-witness.item"
        || item.recovery != NodeRecoveryState::Normal
        || item.children != [crate::typed_ast::TypedNodeId::new(47)]
        || transparent.anchor != SourceAnchor::Range(witness.source_range)
        || transparent.kind.as_str() != "source.surface.unowned"
        || transparent.recovery != NodeRecoveryState::Normal
        || transparent.children != [crate::typed_ast::TypedNodeId::new(46)]
        || application_node.anchor != SourceAnchor::Range(witness.source_range)
        || application_node.kind.as_str() != "source.term.functor-application.symbolic"
        || application_node.recovery != NodeRecoveryState::Normal
        || !is_descendant(arena, theorem.site().node(), witness.take_site.node())
        || !is_descendant(arena, theorem.site().node(), witness.site.node())
        || !is_descendant(
            arena,
            witness.site.node(),
            crate::typed_ast::TypedNodeId::new(46),
        )
        || is_descendant(arena, conclusion.site().node(), witness.site.node())
        || arena
            .iter()
            .filter(|(_, node)| node.kind.as_str().starts_with("source.statement-witness."))
            .count()
            != 2
    {
        return Err(invalid());
    }
    Ok(())
}

#[cfg(test)]
fn validate_task258b3m2b2b1b1_witness_rows(
    source_id: SourceId,
    witnesses: &SourceStatementWitnessTable,
    statements: &SourceStatementHandoff,
    primary_terms: &SourcePrimaryTermHandoff,
    application: Option<&SourceFunctorApplicationHandoff>,
    arena: &TypedArena,
) -> Result<(), SourceStatementWitnessError> {
    let id = SourceStatementWitnessId::new(0);
    let invalid = || SourceStatementWitnessError::InvalidWitness { witness: id };
    let Some(witness) = witnesses.get(id) else {
        return Err(SourceStatementWitnessError::InvalidAggregate);
    };
    let Some(application) = application else {
        return Err(SourceStatementWitnessError::DependencyMismatch);
    };
    if !exact_task258b3m2b2b1b1_application_profile(
        source_id,
        statements.module_id(),
        primary_terms,
        application,
    ) {
        return Err(SourceStatementWitnessError::DependencyMismatch);
    }
    let Some(theorem) = statements.statements().get(SourceStatementId::new(0)) else {
        return Err(invalid());
    };
    let Some(conclusion) = statements.statements().get(SourceStatementId::new(1)) else {
        return Err(invalid());
    };
    let Some(take) = arena.node(witness.take_site.node()) else {
        return Err(invalid());
    };
    let Some(item) = arena.node(witness.site.node()) else {
        return Err(invalid());
    };
    let Some(outer_transparent) = arena.node(crate::typed_ast::TypedNodeId::new(51)) else {
        return Err(invalid());
    };
    let Some(wrapper) = arena.node(crate::typed_ast::TypedNodeId::new(50)) else {
        return Err(invalid());
    };
    let Some(inner_transparent) = arena.node(crate::typed_ast::TypedNodeId::new(49)) else {
        return Err(invalid());
    };
    let Some(application_node) = arena.node(crate::typed_ast::TypedNodeId::new(48)) else {
        return Err(invalid());
    };
    if witness.owner != SourceTheoremOwnerId::new(0)
        || witness.binding_context != BindingContextId::new(1)
        || witness.term
            != SourceStatementWitnessTermTarget::Application(SourceFunctorApplicationId::new(0))
        || witness.take_site.node().index() != 53
        || witness.take_range != range(source_id, 124, 138)
        || witness.site.node().index() != 52
        || witness.source_range != range(source_id, 129, 137)
        || witness.source_ordinal != 1
        || witness.ordinal != 0
        || witness.spelling != "( 1 ++ 2 )"
        || witness.kind != SourceStatementWitnessKind::Unnamed
        || witness.recovery != SourceStatementRecovery::Normal
        || witness.name.is_some()
        || theorem.source_ordinal() != 0
        || conclusion.source_ordinal() != 2
        || take.anchor != SourceAnchor::Range(witness.take_range)
        || take.kind.as_str() != "source.statement-witness.take"
        || take.recovery != NodeRecoveryState::Normal
        || take.children
            != [
                crate::typed_ast::TypedNodeId::new(17),
                witness.site.node(),
                crate::typed_ast::TypedNodeId::new(23),
            ]
        || item.anchor != SourceAnchor::Range(witness.source_range)
        || item.kind.as_str() != "source.statement-witness.item"
        || item.recovery != NodeRecoveryState::Normal
        || item.children != [crate::typed_ast::TypedNodeId::new(51)]
        || outer_transparent.anchor != SourceAnchor::Range(witness.source_range)
        || outer_transparent.kind.as_str() != "source.surface.unowned"
        || outer_transparent.recovery != NodeRecoveryState::Normal
        || outer_transparent.children != [crate::typed_ast::TypedNodeId::new(50)]
        || wrapper.anchor != SourceAnchor::Range(witness.source_range)
        || wrapper.kind.as_str() != "source.term.functor-application.parenthesized"
        || wrapper.recovery != NodeRecoveryState::Normal
        || wrapper.children
            != [
                crate::typed_ast::TypedNodeId::new(18),
                crate::typed_ast::TypedNodeId::new(49),
                crate::typed_ast::TypedNodeId::new(22),
            ]
        || inner_transparent.anchor != SourceAnchor::Range(range(source_id, 130, 136))
        || inner_transparent.kind.as_str() != "source.surface.unowned"
        || inner_transparent.recovery != NodeRecoveryState::Normal
        || inner_transparent.children != [crate::typed_ast::TypedNodeId::new(48)]
        || application_node.anchor != SourceAnchor::Range(range(source_id, 130, 136))
        || application_node.kind.as_str() != "source.term.functor-application.symbolic"
        || application_node.recovery != NodeRecoveryState::Normal
        || !is_descendant(arena, theorem.site().node(), witness.take_site.node())
        || !is_descendant(arena, theorem.site().node(), witness.site.node())
        || !is_descendant(
            arena,
            witness.site.node(),
            crate::typed_ast::TypedNodeId::new(48),
        )
        || is_descendant(arena, conclusion.site().node(), witness.site.node())
        || arena
            .iter()
            .filter(|(_, node)| node.kind.as_str().starts_with("source.statement-witness."))
            .count()
            != 2
    {
        return Err(invalid());
    }
    Ok(())
}

fn validate_task258b3m2b2b2a_witness_rows(
    source_id: SourceId,
    witnesses: &SourceStatementWitnessTable,
    statements: &SourceStatementHandoff,
    primary_terms: &SourcePrimaryTermHandoff,
    structure: Option<&SourceStructureHandoff>,
    arena: &TypedArena,
) -> Result<(), SourceStatementWitnessError> {
    let id = SourceStatementWitnessId::new(0);
    let invalid = || SourceStatementWitnessError::InvalidWitness { witness: id };
    let Some(witness) = witnesses.get(id) else {
        return Err(SourceStatementWitnessError::InvalidAggregate);
    };
    let Some(structure) = structure else {
        return Err(SourceStatementWitnessError::DependencyMismatch);
    };
    if !exact_task258b3m2b2b2a_structure_profile(
        source_id,
        statements.module_id(),
        primary_terms,
        structure,
        arena,
    ) {
        return Err(SourceStatementWitnessError::DependencyMismatch);
    }
    let Some(theorem) = statements.statements().get(SourceStatementId::new(0)) else {
        return Err(invalid());
    };
    let Some(conclusion) = statements.statements().get(SourceStatementId::new(1)) else {
        return Err(invalid());
    };
    let Some(take) = arena.node(witness.take_site.node()) else {
        return Err(invalid());
    };
    let Some(item) = arena.node(witness.site.node()) else {
        return Err(invalid());
    };
    let Some(transparent) = arena.node(crate::typed_ast::TypedNodeId::new(60)) else {
        return Err(invalid());
    };
    let Some(structure_node) = arena.node(crate::typed_ast::TypedNodeId::new(59)) else {
        return Err(invalid());
    };
    if witness.owner != SourceTheoremOwnerId::new(0)
        || witness.binding_context != BindingContextId::new(1)
        || witness.term
            != SourceStatementWitnessTermTarget::Structure(SourceStructureTermId::new(0))
        || witness.take_site.node().index() != 62
        || witness.take_range != range(source_id, 120, 152)
        || witness.site.node().index() != 61
        || witness.source_range != range(source_id, 125, 151)
        || witness.source_ordinal != 1
        || witness.ordinal != 0
        || witness.spelling != "TypeCaseStruct ( x : 1 , y : 2 )"
        || witness.kind != SourceStatementWitnessKind::Unnamed
        || witness.recovery != SourceStatementRecovery::Normal
        || witness.name.is_some()
        || theorem.source_ordinal() != 0
        || conclusion.source_ordinal() != 2
        || take.anchor != SourceAnchor::Range(witness.take_range)
        || take.kind.as_str() != "source.statement-witness.take"
        || take.recovery != NodeRecoveryState::Normal
        || take.children
            != [
                crate::typed_ast::TypedNodeId::new(17),
                witness.site.node(),
                crate::typed_ast::TypedNodeId::new(28),
            ]
        || item.anchor != SourceAnchor::Range(witness.source_range)
        || item.kind.as_str() != "source.statement-witness.item"
        || item.recovery != NodeRecoveryState::Normal
        || item.children != [crate::typed_ast::TypedNodeId::new(60)]
        || transparent.anchor != SourceAnchor::Range(witness.source_range)
        || transparent.kind.as_str() != "source.surface.unowned"
        || transparent.recovery != NodeRecoveryState::Normal
        || transparent.children != [crate::typed_ast::TypedNodeId::new(59)]
        || structure_node.anchor != SourceAnchor::Range(witness.source_range)
        || structure_node.kind.as_str() != "source.term.structure.constructor"
        || structure_node.recovery != NodeRecoveryState::Normal
        || !is_descendant(arena, theorem.site().node(), witness.take_site.node())
        || !is_descendant(arena, theorem.site().node(), witness.site.node())
        || !is_descendant(
            arena,
            witness.site.node(),
            crate::typed_ast::TypedNodeId::new(59),
        )
        || is_descendant(arena, conclusion.site().node(), witness.site.node())
        || arena
            .iter()
            .filter(|(_, node)| node.kind.as_str().starts_with("source.statement-witness."))
            .count()
            != 2
    {
        return Err(invalid());
    }
    Ok(())
}

fn validate_task258b3m2b2b2b_witness_rows(
    source_id: SourceId,
    witnesses: &SourceStatementWitnessTable,
    statements: &SourceStatementHandoff,
    primary_terms: &SourcePrimaryTermHandoff,
    structure: Option<&SourceStructureHandoff>,
    arena: &TypedArena,
) -> Result<(), SourceStatementWitnessError> {
    let id = SourceStatementWitnessId::new(0);
    let invalid = || SourceStatementWitnessError::InvalidWitness { witness: id };
    let Some(witness) = witnesses.get(id) else {
        return Err(SourceStatementWitnessError::InvalidAggregate);
    };
    let Some(structure) = structure else {
        return Err(SourceStatementWitnessError::DependencyMismatch);
    };
    if !exact_task258b3m2b2b2b_structure_profile(
        source_id,
        statements.module_id(),
        primary_terms,
        structure,
        arena,
    ) {
        return Err(SourceStatementWitnessError::DependencyMismatch);
    }
    let Some(theorem) = statements.statements().get(SourceStatementId::new(0)) else {
        return Err(invalid());
    };
    let Some(conclusion) = statements.statements().get(SourceStatementId::new(1)) else {
        return Err(invalid());
    };
    let Some(take) = arena.node(witness.take_site.node()) else {
        return Err(invalid());
    };
    let Some(item) = arena.node(witness.site.node()) else {
        return Err(invalid());
    };
    let Some(transparent) = arena.node(crate::typed_ast::TypedNodeId::new(63)) else {
        return Err(invalid());
    };
    let Some(selector_node) = arena.node(crate::typed_ast::TypedNodeId::new(62)) else {
        return Err(invalid());
    };
    if witness.owner != SourceTheoremOwnerId::new(0)
        || witness.binding_context != BindingContextId::new(1)
        || witness.term
            != SourceStatementWitnessTermTarget::Structure(SourceStructureTermId::new(0))
        || witness.take_site.node().index() != 65
        || witness.take_range != range(source_id, 117, 151)
        || witness.site.node().index() != 64
        || witness.source_range != range(source_id, 122, 150)
        || witness.source_ordinal != 1
        || witness.ordinal != 0
        || witness.spelling != "TypeCaseStruct ( x : 1 , y : 2 ) . x"
        || witness.kind != SourceStatementWitnessKind::Unnamed
        || witness.recovery != SourceStatementRecovery::Normal
        || witness.name.is_some()
        || theorem.source_ordinal() != 0
        || conclusion.source_ordinal() != 2
        || take.anchor != SourceAnchor::Range(witness.take_range)
        || take.kind.as_str() != "source.statement-witness.take"
        || take.recovery != NodeRecoveryState::Normal
        || take.children
            != [
                crate::typed_ast::TypedNodeId::new(17),
                witness.site.node(),
                crate::typed_ast::TypedNodeId::new(30),
            ]
        || item.anchor != SourceAnchor::Range(witness.source_range)
        || item.kind.as_str() != "source.statement-witness.item"
        || item.recovery != NodeRecoveryState::Normal
        || item.children != [crate::typed_ast::TypedNodeId::new(63)]
        || transparent.anchor != SourceAnchor::Range(witness.source_range)
        || transparent.kind.as_str() != "source.surface.unowned"
        || transparent.recovery != NodeRecoveryState::Normal
        || transparent.children != [crate::typed_ast::TypedNodeId::new(62)]
        || selector_node.anchor != SourceAnchor::Range(witness.source_range)
        || selector_node.kind.as_str() != "source.term.structure.selector"
        || selector_node.recovery != NodeRecoveryState::Normal
        || selector_node.children
            != [
                crate::typed_ast::TypedNodeId::new(61),
                crate::typed_ast::TypedNodeId::new(28),
                crate::typed_ast::TypedNodeId::new(29),
            ]
        || !is_descendant(arena, theorem.site().node(), witness.take_site.node())
        || !is_descendant(arena, theorem.site().node(), witness.site.node())
        || !is_descendant(
            arena,
            witness.site.node(),
            crate::typed_ast::TypedNodeId::new(62),
        )
        || is_descendant(arena, conclusion.site().node(), witness.site.node())
        || arena
            .iter()
            .filter(|(_, node)| node.kind.as_str().starts_with("source.statement-witness."))
            .count()
            != 2
    {
        return Err(invalid());
    }
    Ok(())
}

#[cfg(test)]
fn validate_task258b3m2b2b2c_witness_rows(
    source_id: SourceId,
    witnesses: &SourceStatementWitnessTable,
    statements: &SourceStatementHandoff,
    primary_terms: &SourcePrimaryTermHandoff,
    structure: Option<&SourceStructureHandoff>,
    arena: &TypedArena,
) -> Result<(), SourceStatementWitnessError> {
    let id = SourceStatementWitnessId::new(0);
    let invalid = || SourceStatementWitnessError::InvalidWitness { witness: id };
    let Some(witness) = witnesses.get(id) else {
        return Err(SourceStatementWitnessError::InvalidAggregate);
    };
    let Some(structure) = structure else {
        return Err(SourceStatementWitnessError::DependencyMismatch);
    };
    if !exact_task258b3m2b2b2c_structure_profile(
        source_id,
        statements.module_id(),
        primary_terms,
        structure,
        arena,
    ) {
        return Err(SourceStatementWitnessError::DependencyMismatch);
    }
    let Some(theorem) = statements.statements().get(SourceStatementId::new(0)) else {
        return Err(invalid());
    };
    let Some(conclusion) = statements.statements().get(SourceStatementId::new(1)) else {
        return Err(invalid());
    };
    let Some(take) = arena.node(witness.take_site.node()) else {
        return Err(invalid());
    };
    let Some(item) = arena.node(witness.site.node()) else {
        return Err(invalid());
    };
    let Some(transparent) = arena.node(crate::typed_ast::TypedNodeId::new(70)) else {
        return Err(invalid());
    };
    let Some(update_node) = arena.node(crate::typed_ast::TypedNodeId::new(69)) else {
        return Err(invalid());
    };
    if witness.owner != SourceTheoremOwnerId::new(0)
        || witness.binding_context != BindingContextId::new(1)
        || witness.term
            != SourceStatementWitnessTermTarget::Structure(SourceStructureTermId::new(0))
        || witness.take_site.node().index() != 72
        || witness.take_range != range(source_id, 115, 161)
        || witness.site.node().index() != 71
        || witness.source_range != range(source_id, 120, 160)
        || witness.source_ordinal != 1
        || witness.ordinal != 0
        || witness.spelling != "TypeCaseStruct ( x : 1 , y : 2 ) with ( x := 3 )"
        || witness.kind != SourceStatementWitnessKind::Unnamed
        || witness.recovery != SourceStatementRecovery::Normal
        || witness.name.is_some()
        || theorem.source_ordinal() != 0
        || conclusion.source_ordinal() != 2
        || take.anchor != SourceAnchor::Range(witness.take_range)
        || take.kind.as_str() != "source.statement-witness.take"
        || take.recovery != NodeRecoveryState::Normal
        || take.children
            != [
                crate::typed_ast::TypedNodeId::new(17),
                witness.site.node(),
                crate::typed_ast::TypedNodeId::new(34),
            ]
        || item.anchor != SourceAnchor::Range(witness.source_range)
        || item.kind.as_str() != "source.statement-witness.item"
        || item.recovery != NodeRecoveryState::Normal
        || item.children != [crate::typed_ast::TypedNodeId::new(70)]
        || transparent.anchor != SourceAnchor::Range(witness.source_range)
        || transparent.kind.as_str() != "source.surface.unowned"
        || transparent.recovery != NodeRecoveryState::Normal
        || transparent.children != [crate::typed_ast::TypedNodeId::new(69)]
        || update_node.anchor != SourceAnchor::Range(witness.source_range)
        || update_node.kind.as_str() != "source.term.structure.update"
        || update_node.recovery != NodeRecoveryState::Normal
        || update_node.children
            != [
                crate::typed_ast::TypedNodeId::new(65),
                crate::typed_ast::TypedNodeId::new(28),
                crate::typed_ast::TypedNodeId::new(29),
                crate::typed_ast::TypedNodeId::new(68),
                crate::typed_ast::TypedNodeId::new(33),
            ]
        || !is_descendant(arena, theorem.site().node(), witness.take_site.node())
        || !is_descendant(arena, theorem.site().node(), witness.site.node())
        || !is_descendant(
            arena,
            witness.site.node(),
            crate::typed_ast::TypedNodeId::new(69),
        )
        || is_descendant(arena, conclusion.site().node(), witness.site.node())
        || arena
            .iter()
            .filter(|(_, node)| node.kind.as_str().starts_with("source.statement-witness."))
            .count()
            != 2
    {
        return Err(invalid());
    }
    Ok(())
}

#[cfg(test)]
fn validate_witness_name_rows(
    profile: WitnessProfile,
    source_id: SourceId,
    names: &SourceStatementWitnessNameTable,
    witnesses: &SourceStatementWitnessTable,
    arena: &TypedArena,
) -> Result<(), SourceStatementWitnessError> {
    let named = profile == WitnessProfile::Task258B3M1;
    #[cfg(test)]
    let named = named || profile == WitnessProfile::Task258B3N;
    if !named {
        return Ok(());
    }
    let id = SourceStatementWitnessNameId::new(0);
    let Some(name) = names.get(id) else {
        return Err(SourceStatementWitnessError::InvalidAggregate);
    };
    let invalid = || SourceStatementWitnessError::InvalidName { name: id };
    let Some(witness) = witnesses.get(name.witness) else {
        return Err(invalid());
    };
    let Some(site) = arena.node(name.site.node()) else {
        return Err(invalid());
    };
    let expected_range = match profile {
        #[cfg(test)]
        WitnessProfile::Task258B3 => unreachable!("Task258B3 has no witness-name rows"),
        #[cfg(test)]
        WitnessProfile::Task258B3N => range(source_id, 81, 82),
        #[cfg(test)]
        WitnessProfile::Task258B3M1 => range(source_id, 84, 85),
        #[cfg(test)]
        WitnessProfile::Task258B3M2A => {
            unreachable!("Task258B3M2A has no witness-name rows")
        }
        #[cfg(test)]
        WitnessProfile::Task258B3M2B1 => {
            unreachable!("Task258B3M2B1 has no witness-name rows")
        }
        #[cfg(test)]
        WitnessProfile::Task258B3M2B2A => {
            unreachable!("Task258B3M2B2A has no witness-name rows")
        }
        #[cfg(test)]
        WitnessProfile::Task258B3M2B2B1A => {
            unreachable!("Task258B3M2B2B1A has no witness-name rows")
        }
        #[cfg(test)]
        WitnessProfile::Task258B3M2B2B1B1 => {
            unreachable!("Task258B3M2B2B1B1 has no witness-name rows")
        }
        WitnessProfile::Task258B3M2B2B2A => {
            unreachable!("Task258B3M2B2B2A has no witness-name rows")
        }
        WitnessProfile::Task258B3M2B2B2B => {
            unreachable!("Task258B3M2B2B2B has no witness-name rows")
        }
        #[cfg(test)]
        WitnessProfile::Task258B3M2B2B2C => {
            unreachable!("Task258B3M2B2B2C has no witness-name rows")
        }
        #[cfg(test)]
        WitnessProfile::Task258B3M2B2B3A => {
            unreachable!("Task258B3M2B2B3A has no witness-name rows")
        }
        #[cfg(test)]
        WitnessProfile::Task258B3M2B2B3B => {
            unreachable!("Task258B3M2B2B3B has no witness-name rows")
        }
        #[cfg(test)]
        WitnessProfile::Task258B3M2B2B3C => {
            unreachable!("Task258B3M2B2B3C has no witness-name rows")
        }
        #[cfg(test)]
        WitnessProfile::Task258B3M2B2B3D => {
            unreachable!("Task258B3M2B2B3D has no witness-name rows")
        }
        #[cfg(test)]
        WitnessProfile::Task258B3M2B2B3E => {
            unreachable!("Task258B3M2B2B3E has no witness-name rows")
        }
    };
    if name.witness != SourceStatementWitnessId::new(0)
        || witness.name != Some(id)
        || !matches!(&name.site, TypedSiteRef::Node(_))
        || name.site.node().index() != 13
        || name.source_range != expected_range
        || name.spelling != "y"
        || name.recovery != SourceStatementRecovery::Normal
        || site.anchor != SourceAnchor::Range(name.source_range)
        || site.kind.as_str() != "source.statement-witness.name"
        || site.recovery != NodeRecoveryState::Normal
        || !site.children.is_empty()
        || !is_descendant(arena, witness.site.node(), name.site.node())
    {
        return Err(invalid());
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)] // Rationale: authenticate the full resolver tuple with fail-closed precedence.
fn validate_reference_dependencies(
    source_id: SourceId,
    module_id: &ModuleId,
    statement_fingerprint: &str,
    statements: &SourceStatementHandoff,
    resolver_ast: &ResolvedAst,
    projection: &LabelProjection,
    reference: &LabelReferenceCandidate,
    resolution: &LabelResolutionResult,
    arena: &TypedArena,
) -> Result<(), SourceStatementReferenceError> {
    let Some(profile) = statement_reference_profile(statements) else {
        return Err(SourceStatementReferenceError::DependencyMismatch);
    };
    let (arena_len, root_index, reference_node_index, reference_start, reference_end) =
        match profile {
            #[cfg(test)]
            StatementReferenceProfile::Task258B1 => (77, 76, 68, 131, 132),
            #[cfg(test)]
            StatementReferenceProfile::Task258B5A => (93, 92, 82, 170, 171),
            StatementReferenceProfile::Task258B5B => (57, 56, 48, 136, 139),
        };
    let import_count = resolver_ast.imports().imports().count();
    if statements.source_id() != source_id
        || statements.module_id() != module_id
        || statement_fingerprint != statements.debug_text()
        || resolver_ast.source_id() != source_id
        || resolver_ast.module_id() != module_id
        || resolver_ast.nodes().len() != arena_len
        || resolver_ast.nodes().root().index() != root_index
        || !resolver_ast.name_refs().is_empty()
        || resolver_ast.label_refs() != resolution.table()
        || if profile == StatementReferenceProfile::Task258B5B {
            import_count != 1
        } else {
            import_count != 0
        }
        || resolver_ast.imports().exports().next().is_some()
        || arena.len() != arena_len
        || arena.root().is_none_or(|root| root.index() != root_index)
        || (profile == StatementReferenceProfile::Task258B5B
            && !exact_task258b5b_shared_arena(source_id, arena))
    {
        return Err(SourceStatementReferenceError::DependencyMismatch);
    }
    #[cfg(test)]
    if profile == StatementReferenceProfile::Task258B5A
        && !exact_task258b5a_shared_arena(source_id, arena)
    {
        return Err(SourceStatementReferenceError::DependencyMismatch);
    }
    let namespace = mizar_resolve::env::NamespacePath::new(module_id.path().as_str());
    let replay = LabelResolver::new(std::slice::from_ref(projection)).resolve(
        module_id,
        &namespace,
        std::slice::from_ref(reference),
    );
    if &replay != resolution
        || resolution.index().len() != 1
        || resolution.table().len() != 1
        || resolution.ids().len() != 1
        || !resolution.diagnostics().is_empty()
        || resolution.has_unresolved()
    {
        return Err(SourceStatementReferenceError::DependencyMismatch);
    }
    let reference_id = resolution.ids()[0];
    if reference_id.index() != 0 {
        return Err(SourceStatementReferenceError::DependencyMismatch);
    }
    for (resolved_id, resolved_node) in resolver_ast.nodes().iter() {
        let Some(typed_node) = arena
            .iter()
            .find_map(|(typed_id, node)| (typed_id.index() == resolved_id.index()).then_some(node))
        else {
            return Err(SourceStatementReferenceError::DependencyMismatch);
        };
        #[cfg(test)]
        if profile == StatementReferenceProfile::Task258B5A
            && format!("{:?}", resolved_node.kind())
                != task258b5a_resolver_node_kind(resolved_id.index())
        {
            return Err(SourceStatementReferenceError::DependencyMismatch);
        }
        if (profile == StatementReferenceProfile::Task258B5B
            && !task258b5b_resolver_node_kind_is_exact(
                resolved_id.index(),
                &format!("{:?}", resolved_node.kind()),
            ))
            || resolved_node.origin().source_id() != source_id
            || resolved_node.origin().module_id() != module_id
            || resolved_node.origin().import_edge().is_some()
            || resolved_node.origin().structural_path() != [resolved_id.index() as u32]
            || resolved_node.origin().anchor() != &typed_node.anchor
            || resolved_node
                .children()
                .iter()
                .map(|child| child.index())
                .ne(typed_node.children.iter().map(|child| child.index()))
            || resolved_node.recovery() != RecoveryState::Normal
            || typed_node.recovery != NodeRecoveryState::Normal
        {
            return Err(SourceStatementReferenceError::DependencyMismatch);
        }
        let is_reference = resolved_id.index() == reference_node_index;
        if is_reference {
            if resolved_node.resolution() != NodeResolutionState::Resolved
                || resolved_node.reference_key() != Some(NodeReferenceKey::Label(reference_id))
            {
                return Err(SourceStatementReferenceError::DependencyMismatch);
            }
        } else if resolved_node.resolution() != NodeResolutionState::NotApplicable
            || resolved_node.reference_key().is_some()
        {
            return Err(SourceStatementReferenceError::DependencyMismatch);
        }
    }
    let Some((_, reference_node)) =
        resolved_node_by_index(resolver_ast.nodes(), reference_node_index)
    else {
        return Err(SourceStatementReferenceError::DependencyMismatch);
    };
    #[cfg(test)]
    if profile != StatementReferenceProfile::Task258B5B
        && resolved_node_by_index(resolver_ast.nodes(), 12)
            .is_none_or(|(_, label_node)| label_node.origin() != projection.origin())
    {
        return Err(SourceStatementReferenceError::DependencyMismatch);
    }
    if reference_node.origin() != reference.origin()
        || reference.site().node().index() != reference_node_index
        || reference.site().range() != range(source_id, reference_start, reference_end)
        || arena
            .iter()
            .find_map(|(id, node)| (id.index() == reference_node_index).then_some(node))
            .is_none_or(|node| node.anchor != SourceAnchor::Range(reference.site().range()))
    {
        return Err(SourceStatementReferenceError::DependencyMismatch);
    }
    if profile == StatementReferenceProfile::Task258B5B
        && !exact_task258b5b_import(source_id, module_id, resolver_ast, projection.module())
    {
        return Err(SourceStatementReferenceError::DependencyMismatch);
    }
    Ok(())
}

fn validate_reference_aggregate(
    statements: &SourceStatementHandoff,
    labels: usize,
    citations: usize,
) -> Result<(), SourceStatementReferenceError> {
    let expected = if statements.is_task_258b5b_profile() {
        (0, 1)
    } else {
        (1, 1)
    };
    if (labels, citations) != expected {
        return Err(SourceStatementReferenceError::InvalidAggregate);
    }
    Ok(())
}

fn validate_label_rows(
    source_id: SourceId,
    module_id: &ModuleId,
    labels: &SourceStatementLabelTable,
    statements: &SourceStatementHandoff,
    projection: &LabelProjection,
) -> Result<(), SourceStatementReferenceError> {
    let id = SourceStatementLabelId::new(0);
    let Some(profile) = statement_reference_profile(statements) else {
        return Err(SourceStatementReferenceError::InvalidLabel { label: id });
    };
    if profile == StatementReferenceProfile::Task258B5B {
        if !labels.is_empty() || !exact_task258b5b_projection(source_id, module_id, projection) {
            return Err(SourceStatementReferenceError::InvalidLabel { label: id });
        }
        return Ok(());
    }
    #[cfg(test)]
    {
        let (label_start, label_end) = match profile {
            #[cfg(test)]
            StatementReferenceProfile::Task258B1 => (77, 78),
            #[cfg(test)]
            StatementReferenceProfile::Task258B5A => (95, 96),
            StatementReferenceProfile::Task258B5B => unreachable!("handled above"),
        };
        let Some(label) = labels.get(id) else {
            return Err(SourceStatementReferenceError::InvalidAggregate);
        };
        let expected_origin = format!(
            "{}::{}::proof::A",
            module_id.package().as_str(),
            module_id.path().as_str()
        );
        let namespace = mizar_resolve::env::NamespacePath::new(module_id.path().as_str());
        let LabelProjectionSource::CurrentModule {
            visible_after_ordinal,
            proof_scope,
        } = projection.source()
        else {
            return Err(SourceStatementReferenceError::InvalidLabel { label: id });
        };
        let owner_contribution = statements
            .owners()
            .get(SourceTheoremOwnerId::new(0))
            .map(SourceTheoremOwner::contribution);
        if label.statement != SourceStatementId::new(1)
            || statements.statements().get(label.statement).is_none()
            || label.context != SourceStatementContextId::new(1)
            || statements.contexts().get(label.context).is_none()
            || label.candidate != SourceStatementCandidateFactId::new(1)
            || statements.candidate_facts().get(label.candidate).is_none()
            || label.origin_path.as_str() != expected_origin
            || label.proof_scope.path() != [0]
            || label.source_range != range(source_id, label_start, label_end)
            || label.source_ordinal != 0
            || label.visible_after_ordinal != 1
            || label.spelling != "A"
            || label.kind != SourceStatementLabelKind::ProofStep
            || label.recovery != SourceStatementRecovery::Normal
            || projection.origin_path() != &label.origin_path
            || projection.module() != module_id
            || projection.namespace() != &namespace
            || projection.primary_spelling() != label.spelling
            || projection.kind() != LabelKind::ProofStep
            || projection.visibility() != Visibility::Private
            || projection.export_status() != ExportStatus::LocalOnly
            || projection.declaration_range() != label.source_range
            || owner_contribution != Some(projection.contribution())
            || *visible_after_ordinal != label.visible_after_ordinal
            || proof_scope.as_ref() != Some(&label.proof_scope)
            || !exact_semantic_origin(
                projection.origin(),
                source_id,
                module_id,
                label.source_range,
                12,
            )
        {
            return Err(SourceStatementReferenceError::InvalidLabel { label: id });
        }
        Ok(())
    }
    #[cfg(not(test))]
    Err(SourceStatementReferenceError::InvalidLabel { label: id })
}

fn validate_citation_rows(
    source_id: SourceId,
    citations: &SourceStatementCitationTable,
    labels: &SourceStatementLabelTable,
    statements: &SourceStatementHandoff,
    reference: &LabelReferenceCandidate,
    resolution: &LabelResolutionResult,
) -> Result<(), SourceStatementReferenceError> {
    let id = SourceStatementCitationId::new(0);
    let Some(citation) = citations.get(id) else {
        return Err(SourceStatementReferenceError::InvalidAggregate);
    };
    let Some(profile) = statement_reference_profile(statements) else {
        return Err(SourceStatementReferenceError::InvalidCitation { citation: id });
    };
    let (
        statement_index,
        context_index,
        proof_scope_path,
        citation_start,
        citation_end,
        reference_ordinal,
        reference_node_index,
        reference_spelling,
    ) = match profile {
        #[cfg(test)]
        StatementReferenceProfile::Task258B1 => (3, 3, &[0][..], 131, 132, 3, 68, "A"),
        #[cfg(test)]
        StatementReferenceProfile::Task258B5A => (4, 4, &[0, 1][..], 170, 171, 4, 82, "A"),
        StatementReferenceProfile::Task258B5B => (1, 1, &[0][..], 136, 139, 1, 48, "Ref"),
    };
    let LabelReferenceScope::Unqualified { proof_scope } = reference.scope() else {
        return Err(SourceStatementReferenceError::InvalidCitation { citation: id });
    };
    let resolved = resolution
        .table()
        .get(citation.label_ref)
        .and_then(|entry| match entry.resolution() {
            LabelResolution::Resolved(label) => Some(label),
            _ => None,
        });
    if citation.statement != SourceStatementId::new(statement_index)
        || statements.statements().get(citation.statement).is_none()
        || citation.context != SourceStatementContextId::new(context_index)
        || statements.contexts().get(citation.context).is_none()
        || match citation.target {
            SourceStatementCitationTarget::Local(label) => {
                profile == StatementReferenceProfile::Task258B5B
                    || label != SourceStatementLabelId::new(0)
                    || labels.get(label).is_none()
            }
            SourceStatementCitationTarget::Imported => {
                profile != StatementReferenceProfile::Task258B5B || !labels.is_empty()
            }
        }
        || citation.label_ref.index() != 0
        || citation.proof_scope.path() != proof_scope_path
        || citation.source_range != range(source_id, citation_start, citation_end)
        || citation.ordinal != 0
        || citation.kind
            != if profile == StatementReferenceProfile::Task258B5B {
                SourceStatementCitationKind::SimpleImported
            } else {
                SourceStatementCitationKind::SimpleLocal
            }
        || citation.recovery != SourceStatementRecovery::Normal
        || reference.site().range() != citation.source_range
        || reference.site().spelling() != reference_spelling
        || reference.ordinal() != reference_ordinal
        || reference.expectation() != LabelExpectation::ProofOrTheorem
        || proof_scope.as_ref() != Some(&citation.proof_scope)
        || !exact_semantic_origin(
            reference.origin(),
            source_id,
            statements.module_id(),
            citation.source_range,
            reference_node_index as u32,
        )
        || resolution.ids() != [citation.label_ref]
        || resolution
            .table()
            .get(citation.label_ref)
            .is_none_or(|entry| {
                entry.site() != reference.site()
                    || entry.origin() != reference.origin()
                    || entry.recovery() != RecoveryState::Normal
            })
        || resolved.is_none_or(|resolved| {
            resolved.origin().as_str()
                != match citation.target {
                    SourceStatementCitationTarget::Local(label) => labels
                        .get(label)
                        .expect("local label existence checked")
                        .origin_path()
                        .as_str(),
                    SourceStatementCitationTarget::Imported => {
                        "summary:parser.type_fixtures::Ref:label:Ref"
                    }
                }
                || resolved.kind()
                    != if profile == StatementReferenceProfile::Task258B5B {
                        LabelKind::Theorem
                    } else {
                        LabelKind::ProofStep
                    }
                || resolved.range() != citation.source_range
        })
    {
        return Err(SourceStatementReferenceError::InvalidCitation { citation: id });
    }
    Ok(())
}

fn exact_semantic_origin(
    origin: &SemanticOrigin,
    source_id: SourceId,
    module_id: &ModuleId,
    source_range: SourceRange,
    structural_index: u32,
) -> bool {
    origin.source_id() == source_id
        && origin.module_id() == module_id
        && origin.anchor() == &SourceAnchor::Range(source_range)
        && origin.structural_path() == [structural_index]
        && origin.import_edge().is_none()
        && !origin.is_recovered()
}

fn task258b5b_resolver_node_kind_is_exact(index: usize, kind: &str) -> bool {
    TASK258B5B_RESOLVER_NODE_KINDS
        .get(index)
        .is_some_and(|expected| kind == *expected)
}

fn exact_task258b5b_import(
    source_id: SourceId,
    module_id: &ModuleId,
    resolver_ast: &ResolvedAst,
    projection_module: &ModuleId,
) -> bool {
    let imported_module = ModuleId::new(
        module_id.package().clone(),
        ModulePath::new("parser.type_fixtures"),
    );
    let mut imports = resolver_ast.imports().imports();
    let Some((id, import)) = imports.next() else {
        return false;
    };
    id.index() == 0
        && imports.next().is_none()
        && import.owner().index() == 29
        && import.range() == range(source_id, 7, 27)
        && import.source_spelling() == "import parser.type_fixtures;"
        && import.alias().is_none()
        && import.resolution() == &ImportResolution::Resolved(imported_module.clone())
        && import.recovery() == RecoveryState::Normal
        && exact_semantic_origin(import.origin(), source_id, module_id, import.range(), 0)
        && projection_module == &imported_module
}

fn exact_task258b5b_projection(
    source_id: SourceId,
    module_id: &ModuleId,
    projection: &LabelProjection,
) -> bool {
    let imported_module = ModuleId::new(
        module_id.package().clone(),
        ModulePath::new("parser.type_fixtures"),
    );
    projection.source() == &LabelProjectionSource::Imported
        && projection.origin_path().as_str() == "summary:parser.type_fixtures::Ref:label:Ref"
        && projection.module() == &imported_module
        && projection.namespace().as_str() == module_id.path().as_str()
        && projection.primary_spelling() == "Ref"
        && projection.kind() == LabelKind::Theorem
        && projection.visibility() == Visibility::Public
        && projection.export_status() == ExportStatus::Exported
        && projection.declaration_range() == range(source_id, 7, 27)
        && projection.contribution().index() == 2
        && projection.origin().source_id() == source_id
        && projection.origin().module_id() == &imported_module
        && projection.origin().anchor() == &SourceAnchor::Range(range(source_id, 7, 27))
        && projection.origin().structural_path() == [1, 0]
        && projection.origin().import_edge().is_none()
        && !projection.origin().is_recovered()
}

fn resolved_node_by_index(
    arena: &ResolvedArena,
    index: usize,
) -> Option<(ResolvedNodeId, &ResolvedNode)> {
    arena.iter().find(|(id, _)| id.index() == index)
}

fn validate_theorem_site(
    site: &TypedSiteRef,
    source_range: SourceRange,
    arena: &TypedArena,
) -> bool {
    if !matches!(site, TypedSiteRef::Node(_)) {
        return false;
    }
    arena.node(site.node()).is_some_and(|node| {
        node.anchor == SourceAnchor::Range(source_range)
            && node.kind.as_str() == "source.statement.theorem"
            && node.recovery == NodeRecoveryState::Normal
    })
}

fn valid_statement_formula_path(
    _profile: StatementProfile,
    arena: &TypedArena,
    statement: crate::typed_ast::TypedNodeId,
    formula_id: crate::typed_ast::TypedNodeId,
) -> bool {
    let Some(statement) = arena.node(statement) else {
        return false;
    };
    #[cfg(test)]
    if _profile == StatementProfile::Task258A {
        let Some(formula) = arena.node(formula_id) else {
            return false;
        };
        return statement
            .children
            .iter()
            .filter(|child| {
                arena.node(**child).is_some_and(|wrapper| {
                    wrapper.kind.as_str() == "source.surface.unowned"
                        && wrapper.anchor == formula.anchor
                        && wrapper.recovery == NodeRecoveryState::Normal
                        && wrapper.children.as_slice() == [formula_id]
                })
            })
            .count()
            == 1;
    }
    statement
        .children
        .iter()
        .filter(|child| **child == formula_id || unowned_path_contains(arena, **child, formula_id))
        .count()
        == 1
}

fn unowned_path_contains(
    arena: &TypedArena,
    root: crate::typed_ast::TypedNodeId,
    target: crate::typed_ast::TypedNodeId,
) -> bool {
    let Some(node) = arena.node(root) else {
        return false;
    };
    node.kind.as_str() == "source.surface.unowned"
        && node.recovery == NodeRecoveryState::Normal
        && node
            .children
            .iter()
            .any(|child| *child == target || unowned_path_contains(arena, *child, target))
}

fn containing_child_path(
    arena: &TypedArena,
    parent: crate::typed_ast::TypedNodeId,
    target: crate::typed_ast::TypedNodeId,
) -> Option<Vec<usize>> {
    let parent = arena.node(parent)?;
    for (position, child) in parent.children.iter().copied().enumerate() {
        if child == target {
            return Some(vec![position]);
        }
        if let Some(mut suffix) = containing_child_path(arena, child, target) {
            suffix.insert(0, position);
            return Some(suffix);
        }
    }
    None
}

fn containing_child_position(
    arena: &TypedArena,
    parent: crate::typed_ast::TypedNodeId,
    target: crate::typed_ast::TypedNodeId,
) -> Option<usize> {
    arena
        .node(parent)?
        .children
        .iter()
        .position(|child| *child == target || is_descendant(arena, *child, target))
}

fn is_descendant(
    arena: &TypedArena,
    parent: crate::typed_ast::TypedNodeId,
    target: crate::typed_ast::TypedNodeId,
) -> bool {
    let Some(node) = arena.node(parent) else {
        return false;
    };
    node.children
        .iter()
        .any(|child| *child == target || is_descendant(arena, *child, target))
}

#[cfg(test)]
fn subtree_contains_excluded_statement_owner(
    arena: &TypedArena,
    root: crate::typed_ast::TypedNodeId,
    admitted_formula: crate::typed_ast::TypedNodeId,
) -> bool {
    let Some(node) = arena.node(root) else {
        return true;
    };
    let key = node.kind.as_str();
    if root != admitted_formula
        && (key.starts_with("source.statement.")
            || key.contains("proof")
            || key.contains("justification"))
    {
        return true;
    }
    node.children
        .iter()
        .any(|child| subtree_contains_excluded_statement_owner(arena, *child, admitted_formula))
}

const fn range(source_id: SourceId, start: usize, end: usize) -> SourceRange {
    SourceRange {
        source_id,
        start,
        end,
    }
}

fn write_dense_ids<T: Copy>(output: &mut String, rows: &[T], index: impl Fn(T) -> usize) {
    output.push('[');
    for (ordinal, row) in rows.iter().copied().enumerate() {
        if ordinal != 0 {
            output.push_str(", ");
        }
        let _ = write!(output, "{}", index(row));
    }
    output.push(']');
}

fn theorem_role_key(role: SourceTheoremRole) -> &'static str {
    match role {
        SourceTheoremRole::Theorem => "theorem",
        SourceTheoremRole::Lemma => "lemma",
    }
}

fn theorem_status_key(status: SourceTheoremStatus) -> &'static str {
    match status {
        SourceTheoremStatus::Unmodified => "unmodified",
        SourceTheoremStatus::Open => "open",
        SourceTheoremStatus::Assumed => "assumed",
    }
}

fn statement_kind_key(kind: SourceStatementKind) -> &'static str {
    match kind {
        SourceStatementKind::TheoremProposition => "theorem-proposition",
        SourceStatementKind::ProofStepProposition => "proof-step-proposition",
        SourceStatementKind::Assumption => "assumption",
        SourceStatementKind::Conclusion => "conclusion",
    }
}

fn statement_recovery_key(recovery: SourceStatementRecovery) -> &'static str {
    match recovery {
        SourceStatementRecovery::Normal => "normal",
        SourceStatementRecovery::Degraded => "degraded",
    }
}

fn formula_target_key(target: SourceStatementFormulaTarget) -> String {
    match target {
        SourceStatementFormulaTarget::Atomic(id) => format!("atomic:{}", id.index()),
        SourceStatementFormulaTarget::Composite(id) => format!("composite:{}", id.index()),
    }
}

fn witness_term_target_key(target: SourceStatementWitnessTermTarget) -> String {
    match target {
        SourceStatementWitnessTermTarget::Primary(id) => format!("primary#{}", id.index()),
        SourceStatementWitnessTermTarget::Application(id) => {
            format!("application#{}", id.index())
        }
        SourceStatementWitnessTermTarget::Structure(id) => {
            format!("structure#{}", id.index())
        }
        SourceStatementWitnessTermTarget::SetTerm(id) => format!("set-term#{}", id.index()),
    }
}

fn witness_kind_key(kind: SourceStatementWitnessKind) -> &'static str {
    match kind {
        SourceStatementWitnessKind::Unnamed => "unnamed",
        SourceStatementWitnessKind::Named => "named",
    }
}

fn input_fact_kind_key(kind: SourceStatementInputFactKind) -> &'static str {
    match kind {
        SourceStatementInputFactKind::ReservedTypeGuard => "reserved-type-guard",
    }
}

fn candidate_fact_kind_key(kind: SourceStatementCandidateFactKind) -> &'static str {
    match kind {
        SourceStatementCandidateFactKind::UnverifiedProposition => "unverified-proposition",
    }
}

fn statement_label_kind_key(kind: SourceStatementLabelKind) -> &'static str {
    match kind {
        SourceStatementLabelKind::ProofStep => "proof-step",
    }
}

fn statement_citation_kind_key(kind: SourceStatementCitationKind) -> &'static str {
    match kind {
        SourceStatementCitationKind::SimpleLocal => "simple-local",
        SourceStatementCitationKind::SimpleImported => "simple-imported",
    }
}

fn label_kind_key(kind: LabelKind) -> &'static str {
    match kind {
        LabelKind::ProofStep => "proof-step",
        LabelKind::Theorem => "theorem",
        _ => "other",
    }
}

fn visibility_key(visibility: Visibility) -> &'static str {
    match visibility {
        Visibility::Private => "private",
        Visibility::Public => "public",
        _ => "other",
    }
}

fn export_status_key(status: ExportStatus) -> &'static str {
    match status {
        ExportStatus::LocalOnly => "local-only",
        ExportStatus::Exported => "exported",
        ExportStatus::ReExported => "re-exported",
        _ => "other",
    }
}

fn label_expectation_key(expectation: LabelExpectation) -> &'static str {
    match expectation {
        LabelExpectation::ProofOrTheorem => "proof-or-theorem",
        LabelExpectation::ProofStep => "proof-step",
        LabelExpectation::Theorem => "theorem",
        _ => "other",
    }
}

fn resolution_state_key(state: NodeResolutionState) -> &'static str {
    match state {
        NodeResolutionState::NotApplicable => "not-applicable",
        NodeResolutionState::Resolved => "resolved",
        NodeResolutionState::Unresolved => "unresolved",
        NodeResolutionState::Ambiguous => "ambiguous",
        NodeResolutionState::Deferred => "deferred",
        _ => "other",
    }
}

fn node_reference_key(key: NodeReferenceKey) -> String {
    match key {
        NodeReferenceKey::Label(id) => format!("label#{}", id.index()),
        NodeReferenceKey::Name(id) => format!("name#{}", id.index()),
        NodeReferenceKey::Import(id) => format!("import#{}", id.index()),
        NodeReferenceKey::Export(id) => format!("export#{}", id.index()),
        _ => "other".to_owned(),
    }
}

fn label_scope_key(scope: &LabelScopePath) -> String {
    let mut output = String::new();
    write_dense_ids(&mut output, scope.path(), |part| part as usize);
    output
}

fn semantic_origin_path_key(origin: &SemanticOrigin) -> String {
    let mut output = String::from("[");
    for (index, part) in origin.structural_path().iter().enumerate() {
        if index != 0 {
            output.push(',');
        }
        let _ = write!(output, "{part}");
    }
    output.push(']');
    output
}

#[cfg(test)]
mod tests;
