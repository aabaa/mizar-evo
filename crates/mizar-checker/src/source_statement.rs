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
        SourceAtomicFormulaKind, SourceAtomicTermTarget,
    },
    source_composite_formula::{SourceCompositeFormulaHandoff, SourceCompositeFormulaId},
    source_formula_composition::SourceFormulaCompositionHandoff,
    source_set_term::{
        SourceSetEdgeRole, SourceSetRequestKind, SourceSetTarget, SourceSetTermHandoff,
        SourceSetTermId, SourceSetTermKind, SourceSetTermRecovery, SourceSetTypeHead,
        SourceSetTypeOwner, SourceSetTypeRole, SourceSetTypeSiteId,
    },
    source_structure::{SourceStructureHandoff, SourceStructureTermId},
    source_term::{
        SourcePrimaryTermHandoff, SourcePrimaryTermId, SourcePrimaryTermKind,
        SourcePrimaryTermRecovery, SourcePrimaryTermReferenceId, SourcePrimaryTermReferenceRole,
        SourcePrimaryTermRole,
    },
    type_checker::{CheckedStatementOwner, TermFormulaChecker},
    typed_ast::{NodeRecoveryState, TypedArena, TypedAst, TypedSiteRef},
};
use mizar_resolve::{
    env::{ExportStatus, SourceContributionId, SymbolEnv, Visibility},
    labels::{
        LabelProjection, LabelProjectionSource, LabelReferenceCandidate, LabelReferenceScope,
        LabelResolutionResult, LabelResolver, LabelScopePath,
    },
    names::ResolvedVariableScope,
    resolved_ast::{
        LabelExpectation, LabelKind, LabelOriginPath, LabelRefId, LabelResolution, ModuleId,
        NodeReferenceKey, NodeResolutionState, RecoveryState, ResolvedArena, ResolvedAst,
        ResolvedNode, ResolvedNodeId, SemanticOrigin, SurfaceResolvedArena, SymbolId,
    },
};
use mizar_session::{SourceAnchor, SourceId, SourceRange};
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
    LetTypeGuard,
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
            composite_formula_fingerprint: typed
                .source_composite_formula()
                .map(|row| row.debug_text()),
            formula_composition_fingerprint: typed
                .source_formula_composition()
                .map(|row| row.debug_text()),
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
        if let (Some(composite), Some(composition)) = (
            typed.source_composite_formula(),
            typed.source_formula_composition(),
        ) {
            handoff.validate_native_installation(&typed, composite, composition)?;
        } else {
            handoff.validate_installation(
                source.source_id(),
                source.module(),
                primary,
                atomic,
                typed.nodes(),
            )?;
        }
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

    /// Called only with the complete native graph and freshly replayed B5 pair.
    pub(crate) fn validate_native_installation(
        &self,
        typed: &TypedAst,
        composite: &SourceCompositeFormulaHandoff,
        composition: &SourceFormulaCompositionHandoff,
    ) -> Result<(), SourceStatementError> {
        let invalid = || SourceStatementError::DependencyMismatch;
        let input = TermFormulaChecker::source_native_statement_inputs(
            typed,
            composite,
            composition,
            &self.checked_owner,
        )?;
        let bindings = typed.source_context().ok_or_else(invalid)?.binding_env();
        if self.source_id != input.source_id
            || self.module_id != input.module_id
            || &self.binding_env != bindings
            || self.binding_fingerprint != bindings.debug_text()
            || self.primary_term_fingerprint
                != typed.source_term().ok_or_else(invalid)?.debug_text()
            || self.atomic_formula_fingerprint
                != typed
                    .source_atomic_formula()
                    .ok_or_else(invalid)?
                    .debug_text()
            || self.composite_formula_fingerprint.as_deref()
                != Some(composite.debug_text().as_str())
            || self.formula_composition_fingerprint.as_deref()
                != Some(composition.debug_text().as_str())
            || typed.source_statement_references().is_some()
            || typed.source_statement_witnesses().is_some()
            || self.owner_contribution != input.owners.first().ok_or_else(invalid)?.contribution
            || self.owners.len() != input.owners.len()
            || self.statements.len() != input.statements.len()
            || self.contexts.len() != input.contexts.len()
            || self.input_facts.len() != input.input_facts.len()
            || self.candidate_facts.len() != input.candidate_facts.len()
            || !self.owners.iter().zip(&input.owners).all(|((_, a), b)| {
                a.symbol == b.symbol
                    && a.contribution == b.contribution
                    && a.site == b.site
                    && a.source_range == b.source_range
                    && a.spelling == b.spelling
                    && a.role == b.role
                    && a.status == b.status
                    && a.recovery == b.recovery
            })
            || !self
                .statements
                .iter()
                .zip(&input.statements)
                .all(|((_, a), b)| {
                    a.owner == b.owner
                        && a.context == b.context
                        && a.formula == b.formula
                        && a.site == b.site
                        && a.source_range == b.source_range
                        && a.source_ordinal == b.source_ordinal
                        && a.spelling == b.spelling
                        && a.kind == b.kind
                        && a.recovery == b.recovery
                })
            || !self
                .contexts
                .iter()
                .zip(&input.contexts)
                .all(|((_, a), b)| {
                    a.statement == b.statement
                        && a.binding_context == b.binding_context
                        && a.source_range == b.source_range
                        && a.visible_bindings == b.visible_bindings
                })
            || !self
                .input_facts
                .iter()
                .zip(&input.input_facts)
                .all(|((_, a), b)| {
                    a.statement == b.statement
                        && a.context == b.context
                        && a.ordinal == b.ordinal
                        && a.kind == b.kind
                        && a.binding == b.binding
                        && a.uses == b.uses
                })
            || !self
                .candidate_facts
                .iter()
                .zip(&input.candidate_facts)
                .all(|((_, a), b)| {
                    a.statement == b.statement
                        && a.context == b.context
                        && a.ordinal == b.ordinal
                        && a.kind == b.kind
                        && a.formula == b.formula
                })
        {
            return Err(invalid());
        }
        Ok(())
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
        Err(SourceStatementError::DependencyMismatch)
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
        Err(SourceStatementWitnessError::DependencyMismatch)
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
        Err(SourceStatementReferenceError::DependencyMismatch)
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

fn resolved_node_by_index(
    arena: &ResolvedArena,
    index: usize,
) -> Option<(ResolvedNodeId, &ResolvedNode)> {
    arena.iter().find(|(id, _)| id.index() == index)
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
        SourceStatementInputFactKind::LetTypeGuard => "let-type-guard",
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
