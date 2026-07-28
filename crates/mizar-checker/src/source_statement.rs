//! Syntax-free transport for source theorem statements.

use crate::{
    binding_env::{
        BinderIdentity, BindingContextId, BindingContextLayer, BindingContextOwner,
        BindingContextRecovery, BindingEnv, BindingId, BindingKind, BindingRecoveryState,
        BindingStatus, BindingTypeSite,
    },
    source_atomic_formula::{
        SourceAtomicEdgeRole, SourceAtomicFormulaHandoff, SourceAtomicFormulaId,
        SourceAtomicFormulaKind, SourceAtomicFormulaRecovery, SourceAtomicRequestKind,
        SourceAtomicTermTarget,
    },
    source_term::{
        SourcePrimaryTermHandoff, SourcePrimaryTermId, SourcePrimaryTermKind,
        SourcePrimaryTermRecovery, SourcePrimaryTermReferenceId, SourcePrimaryTermReferenceRole,
        SourcePrimaryTermRole,
    },
    type_checker::CheckedStatementOwner,
    typed_ast::{NodeRecoveryState, TypedArena, TypedSiteRef},
};
use mizar_resolve::{
    env::{ContributionKind, ExportStatus, SourceContributionId, SymbolEnv, Visibility},
    labels::{
        LabelProjection, LabelProjectionSource, LabelReferenceCandidate, LabelReferenceScope,
        LabelResolutionResult, LabelResolver, LabelScopePath,
    },
    resolved_ast::{
        LabelExpectation, LabelKind, LabelOriginPath, LabelRefId, LabelResolution, ModuleId,
        NodeReferenceKey, NodeResolutionState, RecoveryState, ResolvedArena, ResolvedAst,
        ResolvedNode, ResolvedNodeId, SemanticOrigin, SymbolId,
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

/// Resolver-authenticated local proof-step citation input.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceStatementCitationInput {
    pub statement: SourceStatementId,
    pub context: SourceStatementContextId,
    pub label: SourceStatementLabelId,
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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum SourceTheoremStatus {
    Unmodified,
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
pub enum SourceStatementCitationKind {
    SimpleLocal,
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
    label_projection: LabelProjection,
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
    witnesses: SourceStatementWitnessTable,
    names: SourceStatementWitnessNameTable,
}

impl SourceStatementHandoff {
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

    pub(crate) fn is_task_258a_profile(&self) -> bool {
        self.binding_env.contexts().len() == 1
            && self.statements.len() == 1
            && self.contexts.len() == 1
            && self.input_facts.len() == 1
            && self.candidate_facts.len() == 1
    }

    pub(crate) fn is_task_258b1_profile(&self) -> bool {
        self.binding_env.contexts().len() == 3
            && self.statements.len() == 4
            && self.contexts.len() == 4
            && self.input_facts.len() == 4
            && self.candidate_facts.len() == 4
    }

    pub(crate) fn is_task_258b2_profile(&self) -> bool {
        self.binding_env.contexts().len() == 2
            && self.statements.len() == 3
            && self.contexts.len() == 3
            && self.input_facts.len() == 3
            && self.candidate_facts.len() == 3
    }

    pub(crate) fn is_task_258b3_profile(&self) -> bool {
        self.binding_env.contexts().len() == 2
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

    pub(crate) fn is_task_258b3n_profile(&self) -> bool {
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

    pub(crate) fn is_task_258b3m1_profile(&self) -> bool {
        self.binding_env.contexts().len() == 2
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
        {
            return Err(SourceStatementError::DependencyMismatch);
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
        let profile = validate_witness_dependencies(
            self.source_id,
            &self.module_id,
            &self.statement_fingerprint,
            &self.primary_term_fingerprint,
            statements,
            primary_terms,
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
            arena,
        )?;
        validate_witness_name_rows(profile, self.source_id, &self.names, &self.witnesses, arena)
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
        &self.label_projection
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
        let label = self
            .labels
            .get(SourceStatementLabelId::new(0))
            .expect("validated source statement label");
        let citation = self
            .citations
            .get(SourceStatementCitationId::new(0))
            .expect("validated source statement citation");
        let label_node = resolved_node_by_index(self.resolver_ast.nodes(), 12)
            .map(|(id, _)| id.index())
            .expect("validated label resolver node");
        let reference_node = self.reference_candidate.site().node().index();
        let reference_state = resolved_node_by_index(self.resolver_ast.nodes(), reference_node)
            .map(|(_, node)| resolution_state_key(node.resolution()))
            .expect("validated reference resolver node");
        let reference_key = resolved_node_by_index(self.resolver_ast.nodes(), reference_node)
            .and_then(|(_, node)| node.reference_key())
            .map(node_reference_key)
            .expect("validated reference resolver key");
        let LabelProjectionSource::CurrentModule {
            visible_after_ordinal,
            proof_scope,
        } = self.label_projection.source()
        else {
            unreachable!("validated current-module proof-step projection")
        };
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
        let _ = writeln!(
            output,
            "resolver-projection origin={} namespace={} range={}..{} visible_after={} scope={} kind={} visibility={} export={} spelling={:?}",
            self.label_projection.origin_path().as_str(),
            self.label_projection.namespace().as_str(),
            self.label_projection.declaration_range().start,
            self.label_projection.declaration_range().end,
            visible_after_ordinal,
            label_scope_key(proof_scope.as_ref().expect("validated projection scope")),
            label_kind_key(self.label_projection.kind()),
            visibility_key(self.label_projection.visibility()),
            export_status_key(self.label_projection.export_status()),
            self.label_projection.primary_spelling(),
        );
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
        let _ = writeln!(
            output,
            "citation#0 statement={} context={} label={} label_ref={} scope={} range={}..{} ordinal={} kind={} recovery={}",
            citation.statement.index(),
            citation.context.index(),
            citation.label.index(),
            citation.label_ref.index(),
            label_scope_key(&citation.proof_scope),
            citation.source_range.start,
            citation.source_range.end,
            citation.ordinal,
            statement_citation_kind_key(citation.kind),
            statement_recovery_key(citation.recovery),
        );
        output
    }

    pub(crate) fn validate_installation(
        &self,
        source_id: SourceId,
        module_id: &ModuleId,
        statements: &SourceStatementHandoff,
        arena: &TypedArena,
    ) -> Result<(), SourceStatementReferenceError> {
        validate_reference_dependencies(
            self.source_id,
            &self.module_id,
            &self.statement_fingerprint,
            statements,
            &self.resolver_ast,
            &self.label_projection,
            &self.reference_candidate,
            &self.label_resolution,
            arena,
        )?;
        if self.source_id != source_id || &self.module_id != module_id {
            return Err(SourceStatementReferenceError::DependencyMismatch);
        }
        validate_reference_aggregate(self.labels.len(), self.citations.len())?;
        validate_label_rows(
            self.source_id,
            &self.module_id,
            &self.labels,
            statements,
            &self.label_projection,
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

/// One validated local proof-step citation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceStatementCitation {
    statement: SourceStatementId,
    context: SourceStatementContextId,
    label: SourceStatementLabelId,
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
    pub const fn label(&self) -> SourceStatementLabelId {
        self.label
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
        let statement_fingerprint = statements.debug_text();
        let primary_term_fingerprint = primary_terms.debug_text();
        let profile = validate_witness_dependencies(
            input.source_id,
            &input.module_id,
            &statement_fingerprint,
            &primary_term_fingerprint,
            statements,
            primary_terms,
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
        validate_witness_name_rows(profile, input.source_id, &names, &witnesses, arena)?;
        Ok(SourceStatementWitnessHandoff {
            source_id: input.source_id,
            module_id: input.module_id,
            statement_fingerprint,
            primary_term_fingerprint,
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
        validate_reference_aggregate(input.labels.len(), input.citations.len())?;
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
                    label: row.label,
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
            label_projection: projection.clone(),
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
    Task258A,
    Task258B1,
    Task258B2,
    Task258B3,
    Task258B3N,
    Task258B3M1,
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
    let b3m1_binding = exact_binding_profile(StatementProfile::Task258B3M1, source_id, bindings);
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
    Err(SourceStatementError::DependencyMismatch)
}

fn exact_binding_profile(
    profile: StatementProfile,
    source_id: SourceId,
    bindings: &BindingEnv,
) -> bool {
    let expected_contexts = match profile {
        StatementProfile::Task258A => 1,
        StatementProfile::Task258B1 => 3,
        StatementProfile::Task258B2 => 2,
        StatementProfile::Task258B3 => 2,
        StatementProfile::Task258B3N => 2,
        StatementProfile::Task258B3M1 => 2,
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
        StatementProfile::Task258A => {}
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
    }
    let Some(binding) = bindings.bindings().get(BindingId::new(0)) else {
        return false;
    };
    binding.id == BindingId::new(0)
        && binding.spelling == "x"
        && binding.kind == BindingKind::ReservedVariable
        && binding.identity
            == BinderIdentity::ReservedVariable {
                spelling: "x".to_owned(),
                declaration_range: range(source_id, 8, 9),
            }
        && binding.owner_context == BindingContextId::new(0)
        && binding.declaration_range == range(source_id, 8, 9)
        && binding.visible_after_ordinal == 0
        && binding.type_site == BindingTypeSite::Source(range(source_id, 14, 17))
        && binding.status == BindingStatus::Reserved
        && binding.captured.identities().is_empty()
        && binding.diagnostics.is_empty()
        && binding.recovery == BindingRecoveryState::Normal
}

fn exact_primary_profile(
    profile: StatementProfile,
    primary_terms: &SourcePrimaryTermHandoff,
) -> bool {
    let expected_ranges: &[(usize, usize)] = match profile {
        StatementProfile::Task258A => &[(74, 75), (78, 79)],
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
        StatementProfile::Task258B2 => &[
            (66, 67),
            (70, 71),
            (87, 88),
            (91, 92),
            (101, 102),
            (105, 106),
        ],
        StatementProfile::Task258B3 => &[(63, 64), (67, 68), (82, 83), (92, 93), (96, 97)],
        StatementProfile::Task258B3N => &[(62, 63), (66, 67), (85, 86), (95, 96), (99, 100)],
        StatementProfile::Task258B3M1 => &[
            (65, 66),
            (69, 70),
            (88, 89),
            (91, 92),
            (101, 102),
            (105, 106),
        ],
    };
    let expected_contexts: &[usize] = match profile {
        StatementProfile::Task258A => &[0, 0],
        StatementProfile::Task258B1 => &[0, 0, 1, 1, 2, 2, 1, 1],
        StatementProfile::Task258B2 => &[0, 0, 1, 1, 1, 1],
        StatementProfile::Task258B3 => &[0, 0, 1, 1, 1],
        StatementProfile::Task258B3N => &[0, 0, 1, 1, 1],
        StatementProfile::Task258B3M1 => &[0, 0, 1, 1, 1, 1],
    };
    let expected_scopes: &[&[u32]] = match profile {
        StatementProfile::Task258A => &[&[], &[]],
        StatementProfile::Task258B1 => &[&[], &[], &[0], &[0], &[0, 0], &[0, 0], &[0], &[0]],
        StatementProfile::Task258B2 => &[&[], &[], &[0], &[0], &[0], &[0]],
        StatementProfile::Task258B3 => &[&[], &[], &[0], &[0], &[0]],
        StatementProfile::Task258B3N => &[&[], &[], &[0], &[0], &[0]],
        StatementProfile::Task258B3M1 => &[&[], &[], &[0], &[0], &[0], &[0]],
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

fn exact_atomic_profile(
    profile: StatementProfile,
    atomic_formulas: &SourceAtomicFormulaHandoff,
    primary_terms: &SourcePrimaryTermHandoff,
    arena: &TypedArena,
) -> bool {
    let expected_ranges: &[(usize, usize)] = match profile {
        StatementProfile::Task258A => &[(74, 79)],
        StatementProfile::Task258B1 => &[(63, 68), (80, 85), (101, 106), (122, 127)],
        StatementProfile::Task258B2 => &[(66, 71), (87, 92), (101, 106)],
        StatementProfile::Task258B3 => &[(63, 68), (92, 97)],
        StatementProfile::Task258B3N => &[(62, 67), (95, 100)],
        StatementProfile::Task258B3M1 => &[(65, 70), (101, 106)],
    };
    let expected_contexts: &[usize] = match profile {
        StatementProfile::Task258A => &[0],
        StatementProfile::Task258B1 => &[0, 1, 2, 1],
        StatementProfile::Task258B2 => &[0, 1, 1],
        StatementProfile::Task258B3 => &[0, 1],
        StatementProfile::Task258B3N => &[0, 1],
        StatementProfile::Task258B3M1 => &[0, 1],
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
            StatementProfile::Task258B3 | StatementProfile::Task258B3N if formula_index == 1 => 3,
            StatementProfile::Task258B3M1 if formula_index == 1 => 4,
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
        match (
            containing_child_position(arena, formula.site().node(), left.site().node()),
            containing_child_position(arena, formula.site().node(), right.site().node()),
        ) {
            (Some(left_position), Some(right_position)) if left_position < right_position => {}
            _ => return false,
        }
    }
    true
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
        StatementProfile::Task258A => (1, 1, 1, 1, 1),
        StatementProfile::Task258B1 => (1, 4, 4, 4, 4),
        StatementProfile::Task258B2 => (1, 3, 3, 3, 3),
        StatementProfile::Task258B3 => (1, 2, 2, 2, 2),
        StatementProfile::Task258B3N => (1, 2, 2, 2, 2),
        StatementProfile::Task258B3M1 => (1, 2, 2, 2, 2),
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
    let label = labels.as_slice().first().copied().ok_or_else(invalid)?;
    if labels.len() != 1
        || owner.contribution != symbol.contribution()
        || owner.contribution != definition.contribution()
        || symbol.primary_spelling() != owner.spelling
        || symbol.origin() != checked.origin()
        || contribution.module() != &input.module_id
        || contribution.kind()
            != &(ContributionKind::LocalSource {
                source_id: input.source_id,
            })
        || contribution.anchor() != &SourceAnchor::Range(range(input.source_id, 0, 18))
        || !contribution.effects().symbols().contains(&owner.symbol)
        || !contribution
            .effects()
            .definitions()
            .contains(&definition.id())
        || !contribution
            .effects()
            .labels()
            .contains(label.origin_path())
        || !contribution.effects().imports().is_empty()
        || label.kind() != LabelKind::Theorem
        || label.visibility() != Visibility::Public
        || label.export_status() != ExportStatus::Exported
        || symbol.namespace().as_str() != input.module_id.path().as_str()
        || label.namespace() != symbol.namespace()
        || label.primary_spelling() != owner.spelling
        || label.origin() != symbol.origin()
        || label.contribution() != owner.contribution
        || label.recovery() != RecoveryState::Normal
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
        StatementProfile::Task258A => (
            range(source_id, 19, 80),
            "FormulaStatementReservedVariableEqualitySmoke",
        ),
        StatementProfile::Task258B1 => (
            range(source_id, 19, 138),
            "FormulaStatementNestedContextSmoke",
        ),
        StatementProfile::Task258B2 => (
            range(source_id, 19, 112),
            "FormulaStatementSingleAssumptionSmoke",
        ),
        StatementProfile::Task258B3 => (
            range(source_id, 19, 103),
            "FormulaStatementSingleWitnessSmoke",
        ),
        StatementProfile::Task258B3N => (
            range(source_id, 19, 106),
            "FormulaStatementNamedWitnessSmoke",
        ),
        StatementProfile::Task258B3M1 => (
            range(source_id, 19, 112),
            "FormulaStatementMultipleWitnessSmoke",
        ),
    };
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
        || (matches!(
            profile,
            StatementProfile::Task258B2
                | StatementProfile::Task258B3
                | StatementProfile::Task258B3N
                | StatementProfile::Task258B3M1
        ) && (owner.contribution.index() != 0
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
        StatementProfile::Task258A => &[(
            0,
            19,
            80,
            SourceStatementKind::TheoremProposition,
            "source.statement.theorem",
            "theorem FormulaStatementReservedVariableEqualitySmoke : x = x ;",
        )],
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
                != if matches!(
                    profile,
                    StatementProfile::Task258B3
                        | StatementProfile::Task258B3N
                        | StatementProfile::Task258B3M1
                ) && index == 1
                {
                    2
                } else {
                    index
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
        StatementProfile::Task258B1 => 4,
        StatementProfile::Task258B2 => 3,
        StatementProfile::Task258B3 => 2,
        StatementProfile::Task258B3N => 2,
        StatementProfile::Task258B3M1 => 2,
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
        StatementProfile::Task258B1 => &[
            &[false, true, true, true],
            &[false, false, true, false],
            &[false, false, false, false],
            &[false, false, false, false],
        ],
        StatementProfile::Task258B2 => &[
            &[false, true, true],
            &[false, false, false],
            &[false, false, false],
        ],
        StatementProfile::Task258B3 => &[&[false, true], &[false, false]],
        StatementProfile::Task258B3N => &[&[false, true], &[false, false]],
        StatementProfile::Task258B3M1 => &[&[false, true], &[false, false]],
        StatementProfile::Task258A => unreachable!("Task258A returns above"),
    };
    let formula_descendants: &[&[bool]] = match profile {
        StatementProfile::Task258B1 => &[
            &[true, true, true, true],
            &[false, true, true, false],
            &[false, false, true, false],
            &[false, false, false, true],
        ],
        StatementProfile::Task258B2 => &[
            &[true, true, true],
            &[false, true, false],
            &[false, false, true],
        ],
        StatementProfile::Task258B3 => &[&[true, true], &[false, true]],
        StatementProfile::Task258B3N => &[&[true, true], &[false, true]],
        StatementProfile::Task258B3M1 => &[&[true, true], &[false, true]],
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
        StatementProfile::Task258A => &[(0, 19, 80)],
        StatementProfile::Task258B1 => &[(0, 19, 138), (1, 77, 114), (2, 96, 107), (1, 117, 133)],
        StatementProfile::Task258B2 => &[(0, 19, 112), (1, 80, 93), (1, 96, 107)],
        StatementProfile::Task258B3 => &[(0, 19, 103), (1, 87, 98)],
        StatementProfile::Task258B3N => &[(0, 19, 106), (1, 90, 101)],
        StatementProfile::Task258B3M1 => &[(0, 19, 112), (1, 96, 107)],
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
        StatementProfile::Task258A => 1,
        StatementProfile::Task258B1 => 4,
        StatementProfile::Task258B2 => 3,
        StatementProfile::Task258B3 => 2,
        StatementProfile::Task258B3N => 2,
        StatementProfile::Task258B3M1 => 2,
    };
    for index in 0..rows {
        let id = SourceStatementInputFactId::new(index);
        let Some(fact) = input_facts.get(id) else {
            return Err(SourceStatementError::InvalidAggregate);
        };
        let first_term = if matches!(
            profile,
            StatementProfile::Task258B3
                | StatementProfile::Task258B3N
                | StatementProfile::Task258B3M1
        ) && index == 1
        {
            if profile == StatementProfile::Task258B3M1 {
                4
            } else {
                3
            }
        } else {
            index * 2
        };
        let expected_uses = [
            SourcePrimaryTermReferenceId::new(first_term),
            SourcePrimaryTermReferenceId::new(first_term + 1),
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
        StatementProfile::Task258A => 1,
        StatementProfile::Task258B1 => 4,
        StatementProfile::Task258B2 => 3,
        StatementProfile::Task258B3 => 2,
        StatementProfile::Task258B3N => 2,
        StatementProfile::Task258B3M1 => 2,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WitnessProfile {
    Task258B3,
    Task258B3N,
    Task258B3M1,
}

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

fn validate_witness_dependencies(
    source_id: SourceId,
    module_id: &ModuleId,
    statement_fingerprint: &str,
    primary_term_fingerprint: &str,
    statements: &SourceStatementHandoff,
    primary_terms: &SourcePrimaryTermHandoff,
    arena: &TypedArena,
) -> Result<WitnessProfile, SourceStatementWitnessError> {
    if statements.source_id() != source_id
        || statements.module_id() != module_id
        || primary_terms.source_id() != source_id
        || primary_terms.module_id() != module_id
        || statement_fingerprint != statements.debug_text()
        || primary_term_fingerprint != primary_terms.debug_text()
        || statements.primary_term_fingerprint() != primary_term_fingerprint
    {
        return Err(SourceStatementWitnessError::DependencyMismatch);
    }
    let profile = if statements.is_task_258b3_profile()
        && exact_binding_profile(
            StatementProfile::Task258B3,
            source_id,
            statements.binding_env(),
        )
        && exact_primary_profile(StatementProfile::Task258B3, primary_terms)
        && exact_task258b3_shared_arena(source_id, arena)
    {
        WitnessProfile::Task258B3
    } else if statements.is_task_258b3n_profile()
        && exact_binding_profile(
            StatementProfile::Task258B3N,
            source_id,
            statements.binding_env(),
        )
        && exact_primary_profile(StatementProfile::Task258B3N, primary_terms)
        && exact_task258b3n_shared_arena(source_id, arena)
    {
        WitnessProfile::Task258B3N
    } else if statements.is_task_258b3m1_profile()
        && exact_binding_profile(
            StatementProfile::Task258B3M1,
            source_id,
            statements.binding_env(),
        )
        && exact_primary_profile(StatementProfile::Task258B3M1, primary_terms)
        && exact_task258b3m1_shared_arena(source_id, arena)
    {
        WitnessProfile::Task258B3M1
    } else {
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
        WitnessProfile::Task258B3 => (range(source_id, 19, 103), 45, range(source_id, 87, 98), 43),
        WitnessProfile::Task258B3N => {
            (range(source_id, 19, 106), 47, range(source_id, 90, 101), 45)
        }
        WitnessProfile::Task258B3M1 => {
            (range(source_id, 19, 112), 52, range(source_id, 96, 107), 50)
        }
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
        WitnessProfile::Task258B3 => (1, 0),
        WitnessProfile::Task258B3N => (1, 1),
        WitnessProfile::Task258B3M1 => (2, 1),
    };
    if (witnesses, names) != expected {
        return Err(SourceStatementWitnessError::InvalidAggregate);
    }
    Ok(())
}

fn validate_witness_rows(
    profile: WitnessProfile,
    source_id: SourceId,
    witnesses: &SourceStatementWitnessTable,
    statements: &SourceStatementHandoff,
    primary_terms: &SourcePrimaryTermHandoff,
    arena: &TypedArena,
) -> Result<(), SourceStatementWitnessError> {
    let Some(theorem) = statements.statements().get(SourceStatementId::new(0)) else {
        return Err(SourceStatementWitnessError::InvalidWitness {
            witness: SourceStatementWitnessId::new(0),
        });
    };
    let Some(conclusion) = statements.statements().get(SourceStatementId::new(1)) else {
        return Err(SourceStatementWitnessError::InvalidWitness {
            witness: SourceStatementWitnessId::new(0),
        });
    };
    let witness_count = match profile {
        WitnessProfile::Task258B3 | WitnessProfile::Task258B3N => 1,
        WitnessProfile::Task258B3M1 => 2,
    };
    for index in 0..witness_count {
        let id = SourceStatementWitnessId::new(index);
        let Some(witness) = witnesses.get(id) else {
            return Err(SourceStatementWitnessError::InvalidAggregate);
        };
        let invalid = || SourceStatementWitnessError::InvalidWitness { witness: id };
        let expected = match (profile, index) {
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
            _ => unreachable!("witness count matches the frozen profile"),
        };
        let term_id = SourcePrimaryTermId::new(expected.term_index);
        let expected_term = SourceStatementWitnessTermTarget::Primary(term_id);
        let Some(term) = primary_terms.terms().get(term_id) else {
            return Err(invalid());
        };
        let Some(reference) = primary_terms
            .references()
            .get(SourcePrimaryTermReferenceId::new(expected.term_index))
        else {
            return Err(invalid());
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
            || term.spelling() != "x"
            || term.kind() != SourcePrimaryTermKind::VariableReference
            || term.role() != SourcePrimaryTermRole::Value
            || term.recovery() != SourcePrimaryTermRecovery::Normal
            || term.parent().is_some()
            || reference.term() != term_id
            || reference.binding() != BindingId::new(0)
            || reference.role() != SourcePrimaryTermReferenceRole::Variable
            || reference
                .lexical_scope()
                .is_none_or(|scope| scope.path() != [0])
            || reference.use_ordinal() != 1
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
    let expected_source_ordinals: &[usize] = match profile {
        WitnessProfile::Task258B3 | WitnessProfile::Task258B3N => &[0, 1, 2],
        WitnessProfile::Task258B3M1 => &[0, 1, 1, 2],
    };
    let actual_source_ordinals = std::iter::once(theorem.source_ordinal())
        .chain(witnesses.iter().map(|(_, witness)| witness.source_ordinal))
        .chain(std::iter::once(conclusion.source_ordinal()));
    let expected_witness_nodes = match profile {
        WitnessProfile::Task258B3 => 2,
        WitnessProfile::Task258B3N => 3,
        WitnessProfile::Task258B3M1 => 4,
    };
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

fn validate_witness_name_rows(
    profile: WitnessProfile,
    source_id: SourceId,
    names: &SourceStatementWitnessNameTable,
    witnesses: &SourceStatementWitnessTable,
    arena: &TypedArena,
) -> Result<(), SourceStatementWitnessError> {
    if profile == WitnessProfile::Task258B3 {
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
        WitnessProfile::Task258B3 => unreachable!("Task258B3 has no witness-name rows"),
        WitnessProfile::Task258B3N => range(source_id, 81, 82),
        WitnessProfile::Task258B3M1 => range(source_id, 84, 85),
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
    if statements.source_id() != source_id
        || statements.module_id() != module_id
        || !statements.is_task_258b1_profile()
        || statement_fingerprint != statements.debug_text()
        || resolver_ast.source_id() != source_id
        || resolver_ast.module_id() != module_id
        || resolver_ast.nodes().len() != 77
        || resolver_ast.nodes().root().index() != 76
        || !resolver_ast.name_refs().is_empty()
        || resolver_ast.label_refs() != resolution.table()
        || resolver_ast.imports().imports().next().is_some()
        || resolver_ast.imports().exports().next().is_some()
        || arena.len() != 77
        || arena.root().is_none_or(|root| root.index() != 76)
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
        if resolved_node.origin().source_id() != source_id
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
        let is_reference = resolved_id.index() == 68;
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
    let Some((_, label_node)) = resolved_node_by_index(resolver_ast.nodes(), 12) else {
        return Err(SourceStatementReferenceError::DependencyMismatch);
    };
    let Some((_, reference_node)) = resolved_node_by_index(resolver_ast.nodes(), 68) else {
        return Err(SourceStatementReferenceError::DependencyMismatch);
    };
    if label_node.origin() != projection.origin()
        || reference_node.origin() != reference.origin()
        || reference.site().node().index() != 68
        || reference.site().range() != range(source_id, 131, 132)
        || arena
            .iter()
            .find_map(|(id, node)| (id.index() == 68).then_some(node))
            .is_none_or(|node| node.anchor != SourceAnchor::Range(reference.site().range()))
    {
        return Err(SourceStatementReferenceError::DependencyMismatch);
    }
    Ok(())
}

fn validate_reference_aggregate(
    labels: usize,
    citations: usize,
) -> Result<(), SourceStatementReferenceError> {
    if (labels, citations) != (1, 1) {
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
        || label.source_range != range(source_id, 77, 78)
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
    if citation.statement != SourceStatementId::new(3)
        || statements.statements().get(citation.statement).is_none()
        || citation.context != SourceStatementContextId::new(3)
        || statements.contexts().get(citation.context).is_none()
        || citation.label != SourceStatementLabelId::new(0)
        || labels.get(citation.label).is_none()
        || citation.label_ref.index() != 0
        || citation.proof_scope.path() != [0]
        || citation.source_range != range(source_id, 131, 132)
        || citation.ordinal != 0
        || citation.kind != SourceStatementCitationKind::SimpleLocal
        || citation.recovery != SourceStatementRecovery::Normal
        || reference.site().range() != citation.source_range
        || reference.site().spelling() != "A"
        || reference.ordinal() != 3
        || reference.expectation() != LabelExpectation::ProofOrTheorem
        || proof_scope.as_ref() != Some(&citation.proof_scope)
        || !exact_semantic_origin(
            reference.origin(),
            source_id,
            statements.module_id(),
            citation.source_range,
            68,
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
            resolved.origin()
                != labels
                    .get(citation.label)
                    .expect("label existence checked")
                    .origin_path()
                || resolved.kind() != LabelKind::ProofStep
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
    profile: StatementProfile,
    arena: &TypedArena,
    statement: crate::typed_ast::TypedNodeId,
    formula_id: crate::typed_ast::TypedNodeId,
) -> bool {
    let Some(statement) = arena.node(statement) else {
        return false;
    };
    if profile == StatementProfile::Task258A {
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
    }
}

fn theorem_status_key(status: SourceTheoremStatus) -> &'static str {
    match status {
        SourceTheoremStatus::Unmodified => "unmodified",
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
    }
}

fn witness_term_target_key(target: SourceStatementWitnessTermTarget) -> String {
    match target {
        SourceStatementWitnessTermTarget::Primary(id) => format!("primary#{}", id.index()),
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        binding_env::{
            BindingContextDraft, BindingContextTable, BindingDiagnosticTable, BindingDraft,
            BindingEnvParts, BindingTable, CapturedFreeVariables,
        },
        cluster_trace::{
            ClusterAttributeFingerprint, ClusterFactDraft, ClusterFactFingerprint,
            ClusterFactProvenance, ClusterFactTable, ClusterTypeFingerprint,
        },
        overload_resolution::{
            CandidateDeclarationKind, CandidateOrigin, CandidateProvenance, CandidateProvenanceKey,
            CandidateScope, CandidateViabilityInput, CandidateViabilityOutput, OverloadCandidateId,
            OverloadCandidateInput, OverloadCollectionOutput, OverloadNameKey,
            OverloadSelectionOutput, OverloadSiteId, OverloadSiteInput, OverloadSiteKey,
            OverloadSiteKind, OverloadSiteRecovery, OverloadSiteResolutionInput,
            RefinementJoinPayload, RefinementJoinStatus, SourceQuaView, SpecificityComparisonInput,
            SpecificityGraphOutput, TemplateExpansionOutput,
        },
        resolved_typed_ast::{
            ExprId, ExpressionMetadataInput, ResolvedNodeKindHint, ResolvedNodeKindHintKind,
            ResolvedTypedAst, ResolvedTypedAstError, ResolvedTypedAstInputs, SourceNodeRole,
            StatementProofInputs, StatementSemanticInputs,
        },
        source_atomic_formula::{
            SourceAtomicEdgeId, SourceAtomicEdgeInput, SourceAtomicFormulaHandoffInput,
            SourceAtomicFormulaInput, SourceAtomicFormulaProducer, SourceAtomicRequestId,
            SourceAtomicRequestInput,
        },
        source_term::{
            SourcePrimaryTermHandoffInput, SourcePrimaryTermInput, SourcePrimaryTermProducer,
            SourcePrimaryTermReferenceInput,
        },
        type_checker::TermFormulaChecker,
        typed_ast::{
            CoercionTable, FactProvenance, FactStatus, InitialObligationTable,
            LocalTypeContextTable, Polarity, StatementTransportTableForTest, TypeDiagnosticTable,
            TypeFactDraft, TypeFactTable, TypePredicateRef, TypeRole, TypeRuleId, TypeTable,
            TypedArenaBuilder, TypedAst, TypedAstError, TypedAstParts, TypedNode, TypedNodeId,
        },
    };
    use mizar_resolve::{
        env::{
            DefinitionIndex, DefinitionKind, DefinitionShell, LabelEntry, LabelIndex,
            NamespacePath, SourceContributionIndex, SymbolEntry, SymbolEnvIndexes, SymbolIndex,
            SymbolKind,
        },
        names::LocalTermScope,
        resolved_ast::{FullyQualifiedName, LabelOriginPath, LocalSymbolId, SemanticOrigin},
    };
    use mizar_session::{
        BuildSnapshotId, InMemorySessionIdAllocator, ModulePath, PackageId, SessionIdAllocator as _,
    };
    use mizar_syntax as syntax;

    const LABEL: &str = "FormulaStatementReservedVariableEqualitySmoke";
    const STATEMENT: &str = "theorem FormulaStatementReservedVariableEqualitySmoke : x = x ;";

    #[derive(Clone)]
    struct Fixture {
        source: SourceId,
        module: ModuleId,
        symbol: SymbolId,
        contribution: SourceContributionId,
        symbols: SymbolEnv,
        bindings: BindingEnv,
        primary: SourcePrimaryTermHandoff,
        atomic: SourceAtomicFormulaHandoff,
        arena: TypedArena,
    }

    impl Fixture {
        fn new(source_ordinal: usize) -> Self {
            let source = source_id(source_ordinal);
            let module = ModuleId::new(PackageId::new("pkg"), ModulePath::new("statement.fixture"));
            let (symbol, contribution, symbols) = symbol_env(source, &module);
            let bindings = binding_env(source, &module);
            let arena = typed_arena(source);
            let primary = SourcePrimaryTermProducer::build(
                SourcePrimaryTermHandoffInput {
                    source_id: source,
                    module_id: module.clone(),
                    terms: vec![
                        SourcePrimaryTermInput {
                            site: node(0),
                            source_range: range(source, 74, 75),
                            source_ordinal: 0,
                            context: BindingContextId::new(0),
                            recovery: SourcePrimaryTermRecovery::Normal,
                            spelling: "x".to_owned(),
                            kind: SourcePrimaryTermKind::VariableReference,
                            role: SourcePrimaryTermRole::Value,
                            parent: None,
                        },
                        SourcePrimaryTermInput {
                            site: node(1),
                            source_range: range(source, 78, 79),
                            source_ordinal: 1,
                            context: BindingContextId::new(0),
                            recovery: SourcePrimaryTermRecovery::Normal,
                            spelling: "x".to_owned(),
                            kind: SourcePrimaryTermKind::VariableReference,
                            role: SourcePrimaryTermRole::Value,
                            parent: None,
                        },
                    ],
                    references: vec![
                        SourcePrimaryTermReferenceInput {
                            term: SourcePrimaryTermId::new(0),
                            binding: BindingId::new(0),
                            role: SourcePrimaryTermReferenceRole::Variable,
                        },
                        SourcePrimaryTermReferenceInput {
                            term: SourcePrimaryTermId::new(1),
                            binding: BindingId::new(0),
                            role: SourcePrimaryTermReferenceRole::Variable,
                        },
                    ],
                    numeric_type_requests: Vec::new(),
                },
                &bindings,
                &arena,
            )
            .expect("exact primary handoff");
            let atomic = SourceAtomicFormulaProducer::build(
                SourceAtomicFormulaHandoffInput {
                    source_id: source,
                    module_id: module.clone(),
                    formulas: vec![SourceAtomicFormulaInput {
                        site: node(3),
                        source_range: range(source, 74, 79),
                        source_ordinal: 0,
                        context: BindingContextId::new(0),
                        recovery: SourceAtomicFormulaRecovery::Normal,
                        spelling: "x = x".to_owned(),
                        kind: SourceAtomicFormulaKind::Equality,
                    }],
                    wrappers: Vec::new(),
                    predicate_segments: Vec::new(),
                    predicate_heads: Vec::new(),
                    candidates: Vec::new(),
                    type_sites: Vec::new(),
                    attributes: Vec::new(),
                    edges: vec![
                        SourceAtomicEdgeInput {
                            formula: SourceAtomicFormulaId::new(0),
                            ordinal: 0,
                            role: SourceAtomicEdgeRole::BuiltinLeftOperand,
                            target: SourceAtomicTermTarget::Primary(SourcePrimaryTermId::new(0)),
                        },
                        SourceAtomicEdgeInput {
                            formula: SourceAtomicFormulaId::new(0),
                            ordinal: 1,
                            role: SourceAtomicEdgeRole::BuiltinRightOperand,
                            target: SourceAtomicTermTarget::Primary(SourcePrimaryTermId::new(1)),
                        },
                    ],
                    requests: (0..2)
                        .map(|ordinal| SourceAtomicRequestInput {
                            formula: SourceAtomicFormulaId::new(0),
                            ordinal,
                            kind: SourceAtomicRequestKind::OperandExpectedType,
                            edge: Some(SourceAtomicEdgeId::new(ordinal)),
                            candidate: None,
                            type_site: None,
                            attribute: None,
                        })
                        .collect(),
                },
                &bindings,
                &symbols,
                &primary,
                None,
                None,
                None,
                &arena,
            )
            .expect("exact atomic handoff");
            Self {
                source,
                module,
                symbol,
                contribution,
                symbols,
                bindings,
                primary,
                atomic,
                arena,
            }
        }

        fn input(&self) -> SourceStatementHandoffInput {
            SourceStatementHandoffInput {
                source_id: self.source,
                module_id: self.module.clone(),
                owners: vec![SourceTheoremOwnerInput {
                    symbol: self.symbol.clone(),
                    contribution: self.contribution,
                    site: node(5),
                    source_range: range(self.source, 19, 80),
                    spelling: LABEL.to_owned(),
                    role: SourceTheoremRole::Theorem,
                    status: SourceTheoremStatus::Unmodified,
                    recovery: SourceStatementRecovery::Normal,
                }],
                statements: vec![SourceStatementInput {
                    owner: SourceTheoremOwnerId::new(0),
                    context: SourceStatementContextId::new(0),
                    formula: SourceStatementFormulaTarget::Atomic(SourceAtomicFormulaId::new(0)),
                    site: node(5),
                    source_range: range(self.source, 19, 80),
                    source_ordinal: 0,
                    spelling: STATEMENT.to_owned(),
                    kind: SourceStatementKind::TheoremProposition,
                    recovery: SourceStatementRecovery::Normal,
                }],
                contexts: vec![SourceStatementContextInput {
                    statement: SourceStatementId::new(0),
                    binding_context: BindingContextId::new(0),
                    source_range: range(self.source, 19, 80),
                    visible_bindings: vec![BindingId::new(0)],
                }],
                input_facts: vec![SourceStatementInputFactInput {
                    statement: SourceStatementId::new(0),
                    context: SourceStatementContextId::new(0),
                    ordinal: 0,
                    kind: SourceStatementInputFactKind::ReservedTypeGuard,
                    binding: BindingId::new(0),
                    uses: vec![
                        SourcePrimaryTermReferenceId::new(0),
                        SourcePrimaryTermReferenceId::new(1),
                    ],
                }],
                candidate_facts: vec![SourceStatementCandidateFactInput {
                    statement: SourceStatementId::new(0),
                    context: SourceStatementContextId::new(0),
                    ordinal: 0,
                    kind: SourceStatementCandidateFactKind::UnverifiedProposition,
                    formula: SourceStatementFormulaTarget::Atomic(SourceAtomicFormulaId::new(0)),
                }],
            }
        }

        fn build(
            &self,
            input: SourceStatementHandoffInput,
        ) -> Result<SourceStatementHandoff, SourceStatementError> {
            SourceStatementProducer::build(
                input,
                &self.symbols,
                &self.bindings,
                &self.primary,
                &self.atomic,
                &self.arena,
            )
        }
    }

    #[derive(Clone)]
    struct B3Fixture {
        source: SourceId,
        module: ModuleId,
        symbol: SymbolId,
        contribution: SourceContributionId,
        symbols: SymbolEnv,
        bindings: BindingEnv,
        primary: SourcePrimaryTermHandoff,
        atomic: SourceAtomicFormulaHandoff,
        arena: TypedArena,
    }

    fn b3_primary_input(source: SourceId, module: &ModuleId) -> SourcePrimaryTermHandoffInput {
        let term_sites = [26, 28, 32, 36, 38];
        let term_ranges = [(63, 64), (67, 68), (82, 83), (92, 93), (96, 97)];
        let term_contexts = [0, 0, 1, 1, 1];
        SourcePrimaryTermHandoffInput {
            source_id: source,
            module_id: module.clone(),
            terms: (0..5)
                .map(|index| SourcePrimaryTermInput {
                    site: node(term_sites[index]),
                    source_range: range(source, term_ranges[index].0, term_ranges[index].1),
                    source_ordinal: index,
                    context: BindingContextId::new(term_contexts[index]),
                    recovery: SourcePrimaryTermRecovery::Normal,
                    spelling: "x".to_owned(),
                    kind: SourcePrimaryTermKind::VariableReference,
                    role: SourcePrimaryTermRole::Value,
                    parent: None,
                })
                .collect(),
            references: (0..5)
                .map(|index| SourcePrimaryTermReferenceInput {
                    term: SourcePrimaryTermId::new(index),
                    binding: BindingId::new(0),
                    role: SourcePrimaryTermReferenceRole::Variable,
                })
                .collect(),
            numeric_type_requests: Vec::new(),
        }
    }

    impl B3Fixture {
        fn new(source_ordinal: usize) -> Self {
            let source = source_id(source_ordinal);
            let module = ModuleId::new(PackageId::new("pkg"), ModulePath::new("statement.fixture"));
            let (symbol, contribution, symbols) = b3_symbol_env(source, &module);
            let bindings = b3_binding_env(source, &module);
            let arena = b3_typed_arena(source);
            let primary = SourcePrimaryTermProducer::build(
                b3_primary_input(source, &module),
                &bindings,
                &arena,
            )
            .expect("Task258B3 primary terms");
            let formula_sites = [30, 40];
            let formula_ranges = [(63, 68), (92, 97)];
            let mut edges = Vec::new();
            let mut requests = Vec::new();
            for formula in 0..2 {
                let first_term = if formula == 0 { 0 } else { 3 };
                for ordinal in 0..2 {
                    let edge = SourceAtomicEdgeId::new(formula * 2 + ordinal);
                    edges.push(SourceAtomicEdgeInput {
                        formula: SourceAtomicFormulaId::new(formula),
                        ordinal,
                        role: if ordinal == 0 {
                            SourceAtomicEdgeRole::BuiltinLeftOperand
                        } else {
                            SourceAtomicEdgeRole::BuiltinRightOperand
                        },
                        target: SourceAtomicTermTarget::Primary(SourcePrimaryTermId::new(
                            first_term + ordinal,
                        )),
                    });
                    requests.push(SourceAtomicRequestInput {
                        formula: SourceAtomicFormulaId::new(formula),
                        ordinal,
                        kind: SourceAtomicRequestKind::OperandExpectedType,
                        edge: Some(edge),
                        candidate: None,
                        type_site: None,
                        attribute: None,
                    });
                }
            }
            let atomic = SourceAtomicFormulaProducer::build(
                SourceAtomicFormulaHandoffInput {
                    source_id: source,
                    module_id: module.clone(),
                    formulas: (0..2)
                        .map(|index| SourceAtomicFormulaInput {
                            site: node(formula_sites[index]),
                            source_range: range(
                                source,
                                formula_ranges[index].0,
                                formula_ranges[index].1,
                            ),
                            source_ordinal: index,
                            context: BindingContextId::new(index),
                            recovery: SourceAtomicFormulaRecovery::Normal,
                            spelling: "x = x".to_owned(),
                            kind: SourceAtomicFormulaKind::Equality,
                        })
                        .collect(),
                    wrappers: Vec::new(),
                    predicate_segments: Vec::new(),
                    predicate_heads: Vec::new(),
                    candidates: Vec::new(),
                    type_sites: Vec::new(),
                    attributes: Vec::new(),
                    edges,
                    requests,
                },
                &bindings,
                &symbols,
                &primary,
                None,
                None,
                None,
                &arena,
            )
            .expect("Task258B3 atomic formulas");
            Self {
                source,
                module,
                symbol,
                contribution,
                symbols,
                bindings,
                primary,
                atomic,
                arena,
            }
        }

        fn statement_input(&self) -> SourceStatementHandoffInput {
            let statement_sites = [45, 43];
            let statement_ranges = [(19, 103), (87, 98)];
            let source_ordinals = [0, 2];
            let spellings = [
                "theorem FormulaStatementSingleWitnessSmoke : x = x proof take x ; thus x = x ; end ;",
                "thus x = x ;",
            ];
            SourceStatementHandoffInput {
                source_id: self.source,
                module_id: self.module.clone(),
                owners: vec![SourceTheoremOwnerInput {
                    symbol: self.symbol.clone(),
                    contribution: self.contribution,
                    site: node(45),
                    source_range: range(self.source, 19, 103),
                    spelling: "FormulaStatementSingleWitnessSmoke".to_owned(),
                    role: SourceTheoremRole::Theorem,
                    status: SourceTheoremStatus::Unmodified,
                    recovery: SourceStatementRecovery::Normal,
                }],
                statements: (0..2)
                    .map(|index| SourceStatementInput {
                        owner: SourceTheoremOwnerId::new(0),
                        context: SourceStatementContextId::new(index),
                        formula: SourceStatementFormulaTarget::Atomic(SourceAtomicFormulaId::new(
                            index,
                        )),
                        site: node(statement_sites[index]),
                        source_range: range(
                            self.source,
                            statement_ranges[index].0,
                            statement_ranges[index].1,
                        ),
                        source_ordinal: source_ordinals[index],
                        spelling: spellings[index].to_owned(),
                        kind: if index == 0 {
                            SourceStatementKind::TheoremProposition
                        } else {
                            SourceStatementKind::Conclusion
                        },
                        recovery: SourceStatementRecovery::Normal,
                    })
                    .collect(),
                contexts: (0..2)
                    .map(|index| SourceStatementContextInput {
                        statement: SourceStatementId::new(index),
                        binding_context: BindingContextId::new(index),
                        source_range: range(
                            self.source,
                            statement_ranges[index].0,
                            statement_ranges[index].1,
                        ),
                        visible_bindings: vec![BindingId::new(0)],
                    })
                    .collect(),
                input_facts: (0..2)
                    .map(|index| {
                        let first_term = if index == 0 { 0 } else { 3 };
                        SourceStatementInputFactInput {
                            statement: SourceStatementId::new(index),
                            context: SourceStatementContextId::new(index),
                            ordinal: 0,
                            kind: SourceStatementInputFactKind::ReservedTypeGuard,
                            binding: BindingId::new(0),
                            uses: vec![
                                SourcePrimaryTermReferenceId::new(first_term),
                                SourcePrimaryTermReferenceId::new(first_term + 1),
                            ],
                        }
                    })
                    .collect(),
                candidate_facts: (0..2)
                    .map(|index| SourceStatementCandidateFactInput {
                        statement: SourceStatementId::new(index),
                        context: SourceStatementContextId::new(index),
                        ordinal: 0,
                        kind: SourceStatementCandidateFactKind::UnverifiedProposition,
                        formula: SourceStatementFormulaTarget::Atomic(SourceAtomicFormulaId::new(
                            index,
                        )),
                    })
                    .collect(),
            }
        }

        fn witness_input(&self) -> SourceStatementWitnessHandoffInput {
            SourceStatementWitnessHandoffInput {
                source_id: self.source,
                module_id: self.module.clone(),
                witnesses: vec![SourceStatementWitnessInput {
                    owner: SourceTheoremOwnerId::new(0),
                    binding_context: BindingContextId::new(1),
                    term: SourceStatementWitnessTermTarget::Primary(SourcePrimaryTermId::new(2)),
                    take_site: node(35),
                    take_range: range(self.source, 77, 84),
                    site: node(34),
                    source_range: range(self.source, 82, 83),
                    source_ordinal: 1,
                    ordinal: 0,
                    spelling: "x".to_owned(),
                    kind: SourceStatementWitnessKind::Unnamed,
                    recovery: SourceStatementRecovery::Normal,
                    name: None,
                }],
                names: Vec::new(),
            }
        }

        fn statement(&self) -> SourceStatementHandoff {
            SourceStatementProducer::build(
                self.statement_input(),
                &self.symbols,
                &self.bindings,
                &self.primary,
                &self.atomic,
                &self.arena,
            )
            .expect("Task258B3 base statement")
        }

        fn witnesses(
            &self,
            statements: &SourceStatementHandoff,
            input: SourceStatementWitnessHandoffInput,
        ) -> Result<SourceStatementWitnessHandoff, SourceStatementWitnessError> {
            SourceStatementWitnessProducer::build(input, statements, &self.primary, &self.arena)
        }

        fn empty_typed(&self) -> TypedAst {
            TypedAst::try_new(TypedAstParts {
                source_id: self.source,
                module_id: self.module.clone(),
                resolved_root: None,
                source_context: None,
                source_type: None,
                source_attribute: None,
                nodes: self.arena.clone(),
                contexts: LocalTypeContextTable::new(),
                types: TypeTable::new(),
                facts: TypeFactTable::new(),
                coercions: CoercionTable::new(),
                initial_obligations: InitialObligationTable::new(),
                diagnostics: TypeDiagnosticTable::new(),
            })
            .expect("Task258B3 empty typed AST")
            .with_source_term(self.primary.clone())
            .expect("Task258B3 Task252")
            .with_source_atomic_formula(self.atomic.clone())
            .expect("Task258B3 Task256")
        }
    }

    #[derive(Clone)]
    struct B3NFixture {
        source: SourceId,
        module: ModuleId,
        symbol: SymbolId,
        contribution: SourceContributionId,
        symbols: SymbolEnv,
        bindings: BindingEnv,
        primary: SourcePrimaryTermHandoff,
        atomic: SourceAtomicFormulaHandoff,
        arena: TypedArena,
    }

    impl B3NFixture {
        fn new(source_ordinal: usize) -> Self {
            let source = source_id(source_ordinal);
            let module = ModuleId::new(PackageId::new("pkg"), ModulePath::new("statement.fixture"));
            let (symbol, contribution, symbols) = b3n_symbol_env(source, &module);
            let bindings = b3n_binding_env(source, &module);
            let arena = b3n_typed_arena(source);
            let term_sites = [28, 30, 34, 38, 40];
            let term_ranges = [(62, 63), (66, 67), (85, 86), (95, 96), (99, 100)];
            let term_contexts = [0, 0, 1, 1, 1];
            let primary = SourcePrimaryTermProducer::build(
                SourcePrimaryTermHandoffInput {
                    source_id: source,
                    module_id: module.clone(),
                    terms: (0..5)
                        .map(|index| SourcePrimaryTermInput {
                            site: node(term_sites[index]),
                            source_range: range(source, term_ranges[index].0, term_ranges[index].1),
                            source_ordinal: index,
                            context: BindingContextId::new(term_contexts[index]),
                            recovery: SourcePrimaryTermRecovery::Normal,
                            spelling: "x".to_owned(),
                            kind: SourcePrimaryTermKind::VariableReference,
                            role: SourcePrimaryTermRole::Value,
                            parent: None,
                        })
                        .collect(),
                    references: (0..5)
                        .map(|index| SourcePrimaryTermReferenceInput {
                            term: SourcePrimaryTermId::new(index),
                            binding: BindingId::new(0),
                            role: SourcePrimaryTermReferenceRole::Variable,
                        })
                        .collect(),
                    numeric_type_requests: Vec::new(),
                },
                &bindings,
                &arena,
            )
            .expect("Task258B3N primary terms");
            let formula_sites = [32, 42];
            let formula_ranges = [(62, 67), (95, 100)];
            let mut edges = Vec::new();
            let mut requests = Vec::new();
            for formula in 0..2 {
                let first_term = if formula == 0 { 0 } else { 3 };
                for ordinal in 0..2 {
                    let edge = SourceAtomicEdgeId::new(formula * 2 + ordinal);
                    edges.push(SourceAtomicEdgeInput {
                        formula: SourceAtomicFormulaId::new(formula),
                        ordinal,
                        role: if ordinal == 0 {
                            SourceAtomicEdgeRole::BuiltinLeftOperand
                        } else {
                            SourceAtomicEdgeRole::BuiltinRightOperand
                        },
                        target: SourceAtomicTermTarget::Primary(SourcePrimaryTermId::new(
                            first_term + ordinal,
                        )),
                    });
                    requests.push(SourceAtomicRequestInput {
                        formula: SourceAtomicFormulaId::new(formula),
                        ordinal,
                        kind: SourceAtomicRequestKind::OperandExpectedType,
                        edge: Some(edge),
                        candidate: None,
                        type_site: None,
                        attribute: None,
                    });
                }
            }
            let atomic = SourceAtomicFormulaProducer::build(
                SourceAtomicFormulaHandoffInput {
                    source_id: source,
                    module_id: module.clone(),
                    formulas: (0..2)
                        .map(|index| SourceAtomicFormulaInput {
                            site: node(formula_sites[index]),
                            source_range: range(
                                source,
                                formula_ranges[index].0,
                                formula_ranges[index].1,
                            ),
                            source_ordinal: index,
                            context: BindingContextId::new(index),
                            recovery: SourceAtomicFormulaRecovery::Normal,
                            spelling: "x = x".to_owned(),
                            kind: SourceAtomicFormulaKind::Equality,
                        })
                        .collect(),
                    wrappers: Vec::new(),
                    predicate_segments: Vec::new(),
                    predicate_heads: Vec::new(),
                    candidates: Vec::new(),
                    type_sites: Vec::new(),
                    attributes: Vec::new(),
                    edges,
                    requests,
                },
                &bindings,
                &symbols,
                &primary,
                None,
                None,
                None,
                &arena,
            )
            .expect("Task258B3N atomic formulas");
            Self {
                source,
                module,
                symbol,
                contribution,
                symbols,
                bindings,
                primary,
                atomic,
                arena,
            }
        }

        fn statement_input(&self) -> SourceStatementHandoffInput {
            let statement_sites = [47, 45];
            let statement_ranges = [(19, 106), (90, 101)];
            let spellings = [
                "theorem FormulaStatementNamedWitnessSmoke : x = x proof take y = x ; thus x = x ; end ;",
                "thus x = x ;",
            ];
            SourceStatementHandoffInput {
                source_id: self.source,
                module_id: self.module.clone(),
                owners: vec![SourceTheoremOwnerInput {
                    symbol: self.symbol.clone(),
                    contribution: self.contribution,
                    site: node(47),
                    source_range: range(self.source, 19, 106),
                    spelling: "FormulaStatementNamedWitnessSmoke".to_owned(),
                    role: SourceTheoremRole::Theorem,
                    status: SourceTheoremStatus::Unmodified,
                    recovery: SourceStatementRecovery::Normal,
                }],
                statements: (0..2)
                    .map(|index| SourceStatementInput {
                        owner: SourceTheoremOwnerId::new(0),
                        context: SourceStatementContextId::new(index),
                        formula: SourceStatementFormulaTarget::Atomic(SourceAtomicFormulaId::new(
                            index,
                        )),
                        site: node(statement_sites[index]),
                        source_range: range(
                            self.source,
                            statement_ranges[index].0,
                            statement_ranges[index].1,
                        ),
                        source_ordinal: [0, 2][index],
                        spelling: spellings[index].to_owned(),
                        kind: if index == 0 {
                            SourceStatementKind::TheoremProposition
                        } else {
                            SourceStatementKind::Conclusion
                        },
                        recovery: SourceStatementRecovery::Normal,
                    })
                    .collect(),
                contexts: (0..2)
                    .map(|index| SourceStatementContextInput {
                        statement: SourceStatementId::new(index),
                        binding_context: BindingContextId::new(index),
                        source_range: range(
                            self.source,
                            statement_ranges[index].0,
                            statement_ranges[index].1,
                        ),
                        visible_bindings: vec![BindingId::new(0)],
                    })
                    .collect(),
                input_facts: (0..2)
                    .map(|index| {
                        let first_term = if index == 0 { 0 } else { 3 };
                        SourceStatementInputFactInput {
                            statement: SourceStatementId::new(index),
                            context: SourceStatementContextId::new(index),
                            ordinal: 0,
                            kind: SourceStatementInputFactKind::ReservedTypeGuard,
                            binding: BindingId::new(0),
                            uses: vec![
                                SourcePrimaryTermReferenceId::new(first_term),
                                SourcePrimaryTermReferenceId::new(first_term + 1),
                            ],
                        }
                    })
                    .collect(),
                candidate_facts: (0..2)
                    .map(|index| SourceStatementCandidateFactInput {
                        statement: SourceStatementId::new(index),
                        context: SourceStatementContextId::new(index),
                        ordinal: 0,
                        kind: SourceStatementCandidateFactKind::UnverifiedProposition,
                        formula: SourceStatementFormulaTarget::Atomic(SourceAtomicFormulaId::new(
                            index,
                        )),
                    })
                    .collect(),
            }
        }

        fn witness_input(&self) -> SourceStatementWitnessHandoffInput {
            SourceStatementWitnessHandoffInput {
                source_id: self.source,
                module_id: self.module.clone(),
                witnesses: vec![SourceStatementWitnessInput {
                    owner: SourceTheoremOwnerId::new(0),
                    binding_context: BindingContextId::new(1),
                    term: SourceStatementWitnessTermTarget::Primary(SourcePrimaryTermId::new(2)),
                    take_site: node(37),
                    take_range: range(self.source, 76, 87),
                    site: node(36),
                    source_range: range(self.source, 81, 86),
                    source_ordinal: 1,
                    ordinal: 0,
                    spelling: "y = x".to_owned(),
                    kind: SourceStatementWitnessKind::Named,
                    recovery: SourceStatementRecovery::Normal,
                    name: Some(SourceStatementWitnessNameId::new(0)),
                }],
                names: vec![SourceStatementWitnessNameInput {
                    witness: SourceStatementWitnessId::new(0),
                    site: node(13),
                    source_range: range(self.source, 81, 82),
                    spelling: "y".to_owned(),
                    recovery: SourceStatementRecovery::Normal,
                }],
            }
        }

        fn statement(&self) -> SourceStatementHandoff {
            SourceStatementProducer::build(
                self.statement_input(),
                &self.symbols,
                &self.bindings,
                &self.primary,
                &self.atomic,
                &self.arena,
            )
            .expect("Task258B3N base statement")
        }

        fn witnesses(
            &self,
            statement: &SourceStatementHandoff,
            input: SourceStatementWitnessHandoffInput,
        ) -> Result<SourceStatementWitnessHandoff, SourceStatementWitnessError> {
            SourceStatementWitnessProducer::build(input, statement, &self.primary, &self.arena)
        }

        fn empty_typed(&self) -> TypedAst {
            TypedAst::try_new(TypedAstParts {
                source_id: self.source,
                module_id: self.module.clone(),
                resolved_root: None,
                source_context: None,
                source_type: None,
                source_attribute: None,
                nodes: self.arena.clone(),
                contexts: LocalTypeContextTable::new(),
                types: TypeTable::new(),
                facts: TypeFactTable::new(),
                coercions: CoercionTable::new(),
                initial_obligations: InitialObligationTable::new(),
                diagnostics: TypeDiagnosticTable::new(),
            })
            .expect("Task258B3N empty typed AST")
            .with_source_term(self.primary.clone())
            .expect("Task258B3N Task252")
            .with_source_atomic_formula(self.atomic.clone())
            .expect("Task258B3N Task256")
        }
    }

    #[derive(Clone)]
    struct B3M1Fixture {
        source: SourceId,
        module: ModuleId,
        symbol: SymbolId,
        contribution: SourceContributionId,
        symbols: SymbolEnv,
        bindings: BindingEnv,
        primary: SourcePrimaryTermHandoff,
        atomic: SourceAtomicFormulaHandoff,
        arena: TypedArena,
    }

    impl B3M1Fixture {
        fn new(source_ordinal: usize) -> Self {
            let source = source_id(source_ordinal);
            let module = ModuleId::new(PackageId::new("pkg"), ModulePath::new("statement.fixture"));
            let (symbol, contribution, symbols) = b3m1_symbol_env(source, &module);
            let bindings = b3m1_binding_env(source, &module);
            let arena = b3m1_typed_arena(source);
            let term_sites = [30, 32, 36, 39, 43, 45];
            let term_ranges = [
                (65, 66),
                (69, 70),
                (88, 89),
                (91, 92),
                (101, 102),
                (105, 106),
            ];
            let term_contexts = [0, 0, 1, 1, 1, 1];
            let primary = SourcePrimaryTermProducer::build(
                SourcePrimaryTermHandoffInput {
                    source_id: source,
                    module_id: module.clone(),
                    terms: (0..6)
                        .map(|index| SourcePrimaryTermInput {
                            site: node(term_sites[index]),
                            source_range: range(source, term_ranges[index].0, term_ranges[index].1),
                            source_ordinal: index,
                            context: BindingContextId::new(term_contexts[index]),
                            recovery: SourcePrimaryTermRecovery::Normal,
                            spelling: "x".to_owned(),
                            kind: SourcePrimaryTermKind::VariableReference,
                            role: SourcePrimaryTermRole::Value,
                            parent: None,
                        })
                        .collect(),
                    references: (0..6)
                        .map(|index| SourcePrimaryTermReferenceInput {
                            term: SourcePrimaryTermId::new(index),
                            binding: BindingId::new(0),
                            role: SourcePrimaryTermReferenceRole::Variable,
                        })
                        .collect(),
                    numeric_type_requests: Vec::new(),
                },
                &bindings,
                &arena,
            )
            .expect("Task258B3M1 primary terms");
            let formula_sites = [34, 47];
            let formula_ranges = [(65, 70), (101, 106)];
            let mut edges = Vec::new();
            let mut requests = Vec::new();
            for formula in 0..2 {
                let first_term = if formula == 0 { 0 } else { 4 };
                for ordinal in 0..2 {
                    let edge = SourceAtomicEdgeId::new(formula * 2 + ordinal);
                    edges.push(SourceAtomicEdgeInput {
                        formula: SourceAtomicFormulaId::new(formula),
                        ordinal,
                        role: if ordinal == 0 {
                            SourceAtomicEdgeRole::BuiltinLeftOperand
                        } else {
                            SourceAtomicEdgeRole::BuiltinRightOperand
                        },
                        target: SourceAtomicTermTarget::Primary(SourcePrimaryTermId::new(
                            first_term + ordinal,
                        )),
                    });
                    requests.push(SourceAtomicRequestInput {
                        formula: SourceAtomicFormulaId::new(formula),
                        ordinal,
                        kind: SourceAtomicRequestKind::OperandExpectedType,
                        edge: Some(edge),
                        candidate: None,
                        type_site: None,
                        attribute: None,
                    });
                }
            }
            let atomic = SourceAtomicFormulaProducer::build(
                SourceAtomicFormulaHandoffInput {
                    source_id: source,
                    module_id: module.clone(),
                    formulas: (0..2)
                        .map(|index| SourceAtomicFormulaInput {
                            site: node(formula_sites[index]),
                            source_range: range(
                                source,
                                formula_ranges[index].0,
                                formula_ranges[index].1,
                            ),
                            source_ordinal: index,
                            context: BindingContextId::new(index),
                            recovery: SourceAtomicFormulaRecovery::Normal,
                            spelling: "x = x".to_owned(),
                            kind: SourceAtomicFormulaKind::Equality,
                        })
                        .collect(),
                    wrappers: Vec::new(),
                    predicate_segments: Vec::new(),
                    predicate_heads: Vec::new(),
                    candidates: Vec::new(),
                    type_sites: Vec::new(),
                    attributes: Vec::new(),
                    edges,
                    requests,
                },
                &bindings,
                &symbols,
                &primary,
                None,
                None,
                None,
                &arena,
            )
            .expect("Task258B3M1 atomic formulas");
            Self {
                source,
                module,
                symbol,
                contribution,
                symbols,
                bindings,
                primary,
                atomic,
                arena,
            }
        }

        fn statement_input(&self) -> SourceStatementHandoffInput {
            let statement_sites = [52, 50];
            let statement_ranges = [(19, 112), (96, 107)];
            let spellings = [
                "theorem FormulaStatementMultipleWitnessSmoke : x = x proof take y = x , x ; thus x = x ; end ;",
                "thus x = x ;",
            ];
            SourceStatementHandoffInput {
                source_id: self.source,
                module_id: self.module.clone(),
                owners: vec![SourceTheoremOwnerInput {
                    symbol: self.symbol.clone(),
                    contribution: self.contribution,
                    site: node(52),
                    source_range: range(self.source, 19, 112),
                    spelling: "FormulaStatementMultipleWitnessSmoke".to_owned(),
                    role: SourceTheoremRole::Theorem,
                    status: SourceTheoremStatus::Unmodified,
                    recovery: SourceStatementRecovery::Normal,
                }],
                statements: (0..2)
                    .map(|index| SourceStatementInput {
                        owner: SourceTheoremOwnerId::new(0),
                        context: SourceStatementContextId::new(index),
                        formula: SourceStatementFormulaTarget::Atomic(SourceAtomicFormulaId::new(
                            index,
                        )),
                        site: node(statement_sites[index]),
                        source_range: range(
                            self.source,
                            statement_ranges[index].0,
                            statement_ranges[index].1,
                        ),
                        source_ordinal: [0, 2][index],
                        spelling: spellings[index].to_owned(),
                        kind: if index == 0 {
                            SourceStatementKind::TheoremProposition
                        } else {
                            SourceStatementKind::Conclusion
                        },
                        recovery: SourceStatementRecovery::Normal,
                    })
                    .collect(),
                contexts: (0..2)
                    .map(|index| SourceStatementContextInput {
                        statement: SourceStatementId::new(index),
                        binding_context: BindingContextId::new(index),
                        source_range: range(
                            self.source,
                            statement_ranges[index].0,
                            statement_ranges[index].1,
                        ),
                        visible_bindings: vec![BindingId::new(0)],
                    })
                    .collect(),
                input_facts: (0..2)
                    .map(|index| {
                        let first_term = if index == 0 { 0 } else { 4 };
                        SourceStatementInputFactInput {
                            statement: SourceStatementId::new(index),
                            context: SourceStatementContextId::new(index),
                            ordinal: 0,
                            kind: SourceStatementInputFactKind::ReservedTypeGuard,
                            binding: BindingId::new(0),
                            uses: vec![
                                SourcePrimaryTermReferenceId::new(first_term),
                                SourcePrimaryTermReferenceId::new(first_term + 1),
                            ],
                        }
                    })
                    .collect(),
                candidate_facts: (0..2)
                    .map(|index| SourceStatementCandidateFactInput {
                        statement: SourceStatementId::new(index),
                        context: SourceStatementContextId::new(index),
                        ordinal: 0,
                        kind: SourceStatementCandidateFactKind::UnverifiedProposition,
                        formula: SourceStatementFormulaTarget::Atomic(SourceAtomicFormulaId::new(
                            index,
                        )),
                    })
                    .collect(),
            }
        }

        fn witness_input(&self) -> SourceStatementWitnessHandoffInput {
            SourceStatementWitnessHandoffInput {
                source_id: self.source,
                module_id: self.module.clone(),
                witnesses: vec![
                    SourceStatementWitnessInput {
                        owner: SourceTheoremOwnerId::new(0),
                        binding_context: BindingContextId::new(1),
                        term: SourceStatementWitnessTermTarget::Primary(SourcePrimaryTermId::new(
                            2,
                        )),
                        take_site: node(42),
                        take_range: range(self.source, 79, 93),
                        site: node(38),
                        source_range: range(self.source, 84, 89),
                        source_ordinal: 1,
                        ordinal: 0,
                        spelling: "y = x".to_owned(),
                        kind: SourceStatementWitnessKind::Named,
                        recovery: SourceStatementRecovery::Normal,
                        name: Some(SourceStatementWitnessNameId::new(0)),
                    },
                    SourceStatementWitnessInput {
                        owner: SourceTheoremOwnerId::new(0),
                        binding_context: BindingContextId::new(1),
                        term: SourceStatementWitnessTermTarget::Primary(SourcePrimaryTermId::new(
                            3,
                        )),
                        take_site: node(42),
                        take_range: range(self.source, 79, 93),
                        site: node(41),
                        source_range: range(self.source, 91, 92),
                        source_ordinal: 1,
                        ordinal: 1,
                        spelling: "x".to_owned(),
                        kind: SourceStatementWitnessKind::Unnamed,
                        recovery: SourceStatementRecovery::Normal,
                        name: None,
                    },
                ],
                names: vec![SourceStatementWitnessNameInput {
                    witness: SourceStatementWitnessId::new(0),
                    site: node(13),
                    source_range: range(self.source, 84, 85),
                    spelling: "y".to_owned(),
                    recovery: SourceStatementRecovery::Normal,
                }],
            }
        }

        fn statement(&self) -> SourceStatementHandoff {
            SourceStatementProducer::build(
                self.statement_input(),
                &self.symbols,
                &self.bindings,
                &self.primary,
                &self.atomic,
                &self.arena,
            )
            .expect("Task258B3M1 base statement")
        }

        fn witnesses(
            &self,
            statement: &SourceStatementHandoff,
            input: SourceStatementWitnessHandoffInput,
        ) -> Result<SourceStatementWitnessHandoff, SourceStatementWitnessError> {
            SourceStatementWitnessProducer::build(input, statement, &self.primary, &self.arena)
        }

        fn empty_typed(&self) -> TypedAst {
            TypedAst::try_new(TypedAstParts {
                source_id: self.source,
                module_id: self.module.clone(),
                resolved_root: None,
                source_context: None,
                source_type: None,
                source_attribute: None,
                nodes: self.arena.clone(),
                contexts: LocalTypeContextTable::new(),
                types: TypeTable::new(),
                facts: TypeFactTable::new(),
                coercions: CoercionTable::new(),
                initial_obligations: InitialObligationTable::new(),
                diagnostics: TypeDiagnosticTable::new(),
            })
            .expect("Task258B3M1 empty typed AST")
            .with_source_term(self.primary.clone())
            .expect("Task258B3M1 Task252")
            .with_source_atomic_formula(self.atomic.clone())
            .expect("Task258B3M1 Task256")
        }
    }

    #[derive(Clone)]
    struct B2Fixture {
        source: SourceId,
        module: ModuleId,
        symbol: SymbolId,
        contribution: SourceContributionId,
        symbols: SymbolEnv,
        bindings: BindingEnv,
        primary: SourcePrimaryTermHandoff,
        atomic: SourceAtomicFormulaHandoff,
        arena: TypedArena,
    }

    impl B2Fixture {
        fn new(source_ordinal: usize) -> Self {
            let source = source_id(source_ordinal);
            let module = ModuleId::new(PackageId::new("pkg"), ModulePath::new("statement.fixture"));
            let (symbol, contribution, symbols) = b2_symbol_env(source, &module);
            let bindings = b2_binding_env(source, &module);
            let arena = b2_typed_arena(source);
            let term_sites = [28, 30, 34, 36, 42, 44];
            let term_ranges = [
                (66, 67),
                (70, 71),
                (87, 88),
                (91, 92),
                (101, 102),
                (105, 106),
            ];
            let term_contexts = [0, 0, 1, 1, 1, 1];
            let primary = SourcePrimaryTermProducer::build(
                SourcePrimaryTermHandoffInput {
                    source_id: source,
                    module_id: module.clone(),
                    terms: (0..6)
                        .map(|index| SourcePrimaryTermInput {
                            site: node(term_sites[index]),
                            source_range: range(source, term_ranges[index].0, term_ranges[index].1),
                            source_ordinal: index,
                            context: BindingContextId::new(term_contexts[index]),
                            recovery: SourcePrimaryTermRecovery::Normal,
                            spelling: "x".to_owned(),
                            kind: SourcePrimaryTermKind::VariableReference,
                            role: SourcePrimaryTermRole::Value,
                            parent: None,
                        })
                        .collect(),
                    references: (0..6)
                        .map(|index| SourcePrimaryTermReferenceInput {
                            term: SourcePrimaryTermId::new(index),
                            binding: BindingId::new(0),
                            role: SourcePrimaryTermReferenceRole::Variable,
                        })
                        .collect(),
                    numeric_type_requests: Vec::new(),
                },
                &bindings,
                &arena,
            )
            .expect("Task258B2 primary terms");
            let formula_sites = [32, 38, 46];
            let formula_ranges = [(66, 71), (87, 92), (101, 106)];
            let formula_contexts = [0, 1, 1];
            let mut edges = Vec::new();
            let mut requests = Vec::new();
            for formula in 0..3 {
                for ordinal in 0..2 {
                    let edge = SourceAtomicEdgeId::new(formula * 2 + ordinal);
                    edges.push(SourceAtomicEdgeInput {
                        formula: SourceAtomicFormulaId::new(formula),
                        ordinal,
                        role: if ordinal == 0 {
                            SourceAtomicEdgeRole::BuiltinLeftOperand
                        } else {
                            SourceAtomicEdgeRole::BuiltinRightOperand
                        },
                        target: SourceAtomicTermTarget::Primary(SourcePrimaryTermId::new(
                            formula * 2 + ordinal,
                        )),
                    });
                    requests.push(SourceAtomicRequestInput {
                        formula: SourceAtomicFormulaId::new(formula),
                        ordinal,
                        kind: SourceAtomicRequestKind::OperandExpectedType,
                        edge: Some(edge),
                        candidate: None,
                        type_site: None,
                        attribute: None,
                    });
                }
            }
            let atomic = SourceAtomicFormulaProducer::build(
                SourceAtomicFormulaHandoffInput {
                    source_id: source,
                    module_id: module.clone(),
                    formulas: (0..3)
                        .map(|index| SourceAtomicFormulaInput {
                            site: node(formula_sites[index]),
                            source_range: range(
                                source,
                                formula_ranges[index].0,
                                formula_ranges[index].1,
                            ),
                            source_ordinal: index,
                            context: BindingContextId::new(formula_contexts[index]),
                            recovery: SourceAtomicFormulaRecovery::Normal,
                            spelling: "x = x".to_owned(),
                            kind: SourceAtomicFormulaKind::Equality,
                        })
                        .collect(),
                    wrappers: Vec::new(),
                    predicate_segments: Vec::new(),
                    predicate_heads: Vec::new(),
                    candidates: Vec::new(),
                    type_sites: Vec::new(),
                    attributes: Vec::new(),
                    edges,
                    requests,
                },
                &bindings,
                &symbols,
                &primary,
                None,
                None,
                None,
                &arena,
            )
            .expect("Task258B2 atomic formulas");
            Self {
                source,
                module,
                symbol,
                contribution,
                symbols,
                bindings,
                primary,
                atomic,
                arena,
            }
        }

        fn input(&self) -> SourceStatementHandoffInput {
            let statement_sites = [51, 41, 49];
            let statement_ranges = [(19, 112), (80, 93), (96, 107)];
            let statement_kinds = [
                SourceStatementKind::TheoremProposition,
                SourceStatementKind::Assumption,
                SourceStatementKind::Conclusion,
            ];
            let spellings = [
                "theorem FormulaStatementSingleAssumptionSmoke : x = x proof assume x = x ; thus x = x ; end ;",
                "assume x = x ;",
                "thus x = x ;",
            ];
            let binding_contexts = [0, 1, 1];
            SourceStatementHandoffInput {
                source_id: self.source,
                module_id: self.module.clone(),
                owners: vec![SourceTheoremOwnerInput {
                    symbol: self.symbol.clone(),
                    contribution: self.contribution,
                    site: node(51),
                    source_range: range(self.source, 19, 112),
                    spelling: "FormulaStatementSingleAssumptionSmoke".to_owned(),
                    role: SourceTheoremRole::Theorem,
                    status: SourceTheoremStatus::Unmodified,
                    recovery: SourceStatementRecovery::Normal,
                }],
                statements: (0..3)
                    .map(|index| SourceStatementInput {
                        owner: SourceTheoremOwnerId::new(0),
                        context: SourceStatementContextId::new(index),
                        formula: SourceStatementFormulaTarget::Atomic(SourceAtomicFormulaId::new(
                            index,
                        )),
                        site: node(statement_sites[index]),
                        source_range: range(
                            self.source,
                            statement_ranges[index].0,
                            statement_ranges[index].1,
                        ),
                        source_ordinal: index,
                        spelling: spellings[index].to_owned(),
                        kind: statement_kinds[index],
                        recovery: SourceStatementRecovery::Normal,
                    })
                    .collect(),
                contexts: (0..3)
                    .map(|index| SourceStatementContextInput {
                        statement: SourceStatementId::new(index),
                        binding_context: BindingContextId::new(binding_contexts[index]),
                        source_range: range(
                            self.source,
                            statement_ranges[index].0,
                            statement_ranges[index].1,
                        ),
                        visible_bindings: vec![BindingId::new(0)],
                    })
                    .collect(),
                input_facts: (0..3)
                    .map(|index| SourceStatementInputFactInput {
                        statement: SourceStatementId::new(index),
                        context: SourceStatementContextId::new(index),
                        ordinal: 0,
                        kind: SourceStatementInputFactKind::ReservedTypeGuard,
                        binding: BindingId::new(0),
                        uses: vec![
                            SourcePrimaryTermReferenceId::new(index * 2),
                            SourcePrimaryTermReferenceId::new(index * 2 + 1),
                        ],
                    })
                    .collect(),
                candidate_facts: (0..3)
                    .map(|index| SourceStatementCandidateFactInput {
                        statement: SourceStatementId::new(index),
                        context: SourceStatementContextId::new(index),
                        ordinal: 0,
                        kind: SourceStatementCandidateFactKind::UnverifiedProposition,
                        formula: SourceStatementFormulaTarget::Atomic(SourceAtomicFormulaId::new(
                            index,
                        )),
                    })
                    .collect(),
            }
        }

        fn build(
            &self,
            input: SourceStatementHandoffInput,
        ) -> Result<SourceStatementHandoff, SourceStatementError> {
            SourceStatementProducer::build(
                input,
                &self.symbols,
                &self.bindings,
                &self.primary,
                &self.atomic,
                &self.arena,
            )
        }

        fn empty_typed(&self) -> TypedAst {
            TypedAst::try_new(TypedAstParts {
                source_id: self.source,
                module_id: self.module.clone(),
                resolved_root: None,
                source_context: None,
                source_type: None,
                source_attribute: None,
                nodes: self.arena.clone(),
                contexts: LocalTypeContextTable::new(),
                types: TypeTable::new(),
                facts: TypeFactTable::new(),
                coercions: CoercionTable::new(),
                initial_obligations: InitialObligationTable::new(),
                diagnostics: TypeDiagnosticTable::new(),
            })
            .expect("Task258B2 empty typed AST")
            .with_source_term(self.primary.clone())
            .expect("Task258B2 Task252")
            .with_source_atomic_formula(self.atomic.clone())
            .expect("Task258B2 Task256")
        }
    }

    #[derive(Clone)]
    struct B1Fixture {
        source: SourceId,
        module: ModuleId,
        symbols: SymbolEnv,
        bindings: BindingEnv,
        primary: SourcePrimaryTermHandoff,
        atomic: SourceAtomicFormulaHandoff,
        arena: TypedArena,
        statement: SourceStatementHandoff,
        resolver_ast: ResolvedAst,
        projection: LabelProjection,
        reference: LabelReferenceCandidate,
        resolution: LabelResolutionResult,
    }

    impl B1Fixture {
        fn new(source_ordinal: usize) -> Self {
            let source = source_id(source_ordinal);
            let module = ModuleId::new(PackageId::new("pkg"), ModulePath::new("statement.fixture"));
            let (symbol, contribution, symbols) = b1_symbol_env(source, &module);
            let bindings = b1_binding_env(source, &module);
            let arena = b1_typed_arena(source);
            let term_sites = [38, 40, 44, 46, 51, 53, 61, 63];
            let term_ranges = [
                (63, 64),
                (67, 68),
                (80, 81),
                (84, 85),
                (101, 102),
                (105, 106),
                (122, 123),
                (126, 127),
            ];
            let term_contexts = [0, 0, 1, 1, 2, 2, 1, 1];
            let primary = SourcePrimaryTermProducer::build(
                SourcePrimaryTermHandoffInput {
                    source_id: source,
                    module_id: module.clone(),
                    terms: (0..8)
                        .map(|index| SourcePrimaryTermInput {
                            site: node(term_sites[index]),
                            source_range: range(source, term_ranges[index].0, term_ranges[index].1),
                            source_ordinal: index,
                            context: BindingContextId::new(term_contexts[index]),
                            recovery: SourcePrimaryTermRecovery::Normal,
                            spelling: "x".to_owned(),
                            kind: SourcePrimaryTermKind::VariableReference,
                            role: SourcePrimaryTermRole::Value,
                            parent: None,
                        })
                        .collect(),
                    references: (0..8)
                        .map(|index| SourcePrimaryTermReferenceInput {
                            term: SourcePrimaryTermId::new(index),
                            binding: BindingId::new(0),
                            role: SourcePrimaryTermReferenceRole::Variable,
                        })
                        .collect(),
                    numeric_type_requests: Vec::new(),
                },
                &bindings,
                &arena,
            )
            .expect("Task258B1 primary terms");
            let formula_sites = [42, 48, 55, 65];
            let formula_ranges = [(63, 68), (80, 85), (101, 106), (122, 127)];
            let formula_contexts = [0, 1, 2, 1];
            let mut edges = Vec::new();
            let mut requests = Vec::new();
            for formula in 0..4 {
                for ordinal in 0..2 {
                    let edge = SourceAtomicEdgeId::new(formula * 2 + ordinal);
                    edges.push(SourceAtomicEdgeInput {
                        formula: SourceAtomicFormulaId::new(formula),
                        ordinal,
                        role: if ordinal == 0 {
                            SourceAtomicEdgeRole::BuiltinLeftOperand
                        } else {
                            SourceAtomicEdgeRole::BuiltinRightOperand
                        },
                        target: SourceAtomicTermTarget::Primary(SourcePrimaryTermId::new(
                            formula * 2 + ordinal,
                        )),
                    });
                    requests.push(SourceAtomicRequestInput {
                        formula: SourceAtomicFormulaId::new(formula),
                        ordinal,
                        kind: SourceAtomicRequestKind::OperandExpectedType,
                        edge: Some(edge),
                        candidate: None,
                        type_site: None,
                        attribute: None,
                    });
                }
            }
            let atomic = SourceAtomicFormulaProducer::build(
                SourceAtomicFormulaHandoffInput {
                    source_id: source,
                    module_id: module.clone(),
                    formulas: (0..4)
                        .map(|index| SourceAtomicFormulaInput {
                            site: node(formula_sites[index]),
                            source_range: range(
                                source,
                                formula_ranges[index].0,
                                formula_ranges[index].1,
                            ),
                            source_ordinal: index,
                            context: BindingContextId::new(formula_contexts[index]),
                            recovery: SourceAtomicFormulaRecovery::Normal,
                            spelling: "x = x".to_owned(),
                            kind: SourceAtomicFormulaKind::Equality,
                        })
                        .collect(),
                    wrappers: Vec::new(),
                    predicate_segments: Vec::new(),
                    predicate_heads: Vec::new(),
                    candidates: Vec::new(),
                    type_sites: Vec::new(),
                    attributes: Vec::new(),
                    edges,
                    requests,
                },
                &bindings,
                &symbols,
                &primary,
                None,
                None,
                None,
                &arena,
            )
            .expect("Task258B1 atomic formulas");
            let statement_sites = [73, 60, 58, 71];
            let statement_ranges = [(19, 138), (77, 114), (96, 107), (117, 133)];
            let statement_kinds = [
                SourceStatementKind::TheoremProposition,
                SourceStatementKind::ProofStepProposition,
                SourceStatementKind::Conclusion,
                SourceStatementKind::Conclusion,
            ];
            let binding_contexts = [0, 1, 2, 1];
            let statement = SourceStatementProducer::build(
                SourceStatementHandoffInput {
                    source_id: source,
                    module_id: module.clone(),
                    owners: vec![SourceTheoremOwnerInput {
                        symbol,
                        contribution,
                        site: node(73),
                        source_range: range(source, 19, 138),
                        spelling: "FormulaStatementNestedContextSmoke".to_owned(),
                        role: SourceTheoremRole::Theorem,
                        status: SourceTheoremStatus::Unmodified,
                        recovery: SourceStatementRecovery::Normal,
                    }],
                    statements: (0..4)
                        .map(|index| SourceStatementInput {
                            owner: SourceTheoremOwnerId::new(0),
                            context: SourceStatementContextId::new(index),
                            formula: SourceStatementFormulaTarget::Atomic(
                                SourceAtomicFormulaId::new(index),
                            ),
                            site: node(statement_sites[index]),
                            source_range: range(
                                source,
                                statement_ranges[index].0,
                                statement_ranges[index].1,
                            ),
                            source_ordinal: index,
                            spelling: [
                                "theorem FormulaStatementNestedContextSmoke : x = x proof A : x = x proof thus x = x ; end ; thus x = x by A ; end ;",
                                "A : x = x proof thus x = x ; end ;",
                                "thus x = x ;",
                                "thus x = x by A ;",
                            ][index]
                                .to_owned(),
                            kind: statement_kinds[index],
                            recovery: SourceStatementRecovery::Normal,
                        })
                        .collect(),
                    contexts: (0..4)
                        .map(|index| SourceStatementContextInput {
                            statement: SourceStatementId::new(index),
                            binding_context: BindingContextId::new(binding_contexts[index]),
                            source_range: range(
                                source,
                                statement_ranges[index].0,
                                statement_ranges[index].1,
                            ),
                            visible_bindings: vec![BindingId::new(0)],
                        })
                        .collect(),
                    input_facts: (0..4)
                        .map(|index| SourceStatementInputFactInput {
                            statement: SourceStatementId::new(index),
                            context: SourceStatementContextId::new(index),
                            ordinal: 0,
                            kind: SourceStatementInputFactKind::ReservedTypeGuard,
                            binding: BindingId::new(0),
                            uses: vec![
                                SourcePrimaryTermReferenceId::new(index * 2),
                                SourcePrimaryTermReferenceId::new(index * 2 + 1),
                            ],
                        })
                        .collect(),
                    candidate_facts: (0..4)
                        .map(|index| SourceStatementCandidateFactInput {
                            statement: SourceStatementId::new(index),
                            context: SourceStatementContextId::new(index),
                            ordinal: 0,
                            kind: SourceStatementCandidateFactKind::UnverifiedProposition,
                            formula: SourceStatementFormulaTarget::Atomic(
                                SourceAtomicFormulaId::new(index),
                            ),
                        })
                        .collect(),
                },
                &symbols,
                &bindings,
                &primary,
                &atomic,
                &arena,
            )
            .expect("Task258B1 base statement");
            let preliminary = b1_resolved_arena(source, &module, None);
            let reference_node = preliminary
                .iter()
                .find_map(|(id, _)| (id.index() == 68).then_some(id))
                .expect("reference node");
            let namespace = NamespacePath::new(module.path().as_str());
            let projection = LabelProjection::proof_step(
                mizar_resolve::labels::LabelProjectionData {
                    origin_path: LabelOriginPath::new("pkg::statement.fixture::proof::A"),
                    module: module.clone(),
                    namespace: namespace.clone(),
                    primary_spelling: "A".to_owned(),
                    kind: LabelKind::ProofStep,
                    declaration_range: range(source, 77, 78),
                    origin: SemanticOrigin::new(
                        source,
                        module.clone(),
                        SourceAnchor::Range(range(source, 77, 78)),
                        vec![12],
                    ),
                    contribution,
                },
                1,
                LabelScopePath::new(vec![0]),
            );
            let reference = LabelReferenceCandidate::unqualified_citation(
                mizar_resolve::resolved_ast::ReferenceSite::new(
                    reference_node,
                    range(source, 131, 132),
                    "A",
                ),
                SemanticOrigin::new(
                    source,
                    module.clone(),
                    SourceAnchor::Range(range(source, 131, 132)),
                    vec![68],
                ),
                3,
                Some(LabelScopePath::new(vec![0])),
            );
            let resolution = LabelResolver::new(std::slice::from_ref(&projection)).resolve(
                &module,
                &namespace,
                std::slice::from_ref(&reference),
            );
            let resolver_ast = ResolvedAst::try_new(
                source,
                module.clone(),
                b1_resolved_arena(source, &module, Some(resolution.ids()[0])),
                mizar_resolve::resolved_ast::NameRefTable::new(),
                resolution.table().clone(),
                mizar_resolve::resolved_ast::ResolvedImports::new(),
            )
            .expect("Task258B1 resolver AST");
            Self {
                source,
                module,
                symbols,
                bindings,
                primary,
                atomic,
                arena,
                statement,
                resolver_ast,
                projection,
                reference,
                resolution,
            }
        }

        fn reference_input(&self) -> SourceStatementReferenceHandoffInput {
            SourceStatementReferenceHandoffInput {
                source_id: self.source,
                module_id: self.module.clone(),
                labels: vec![SourceStatementLabelInput {
                    statement: SourceStatementId::new(1),
                    context: SourceStatementContextId::new(1),
                    candidate: SourceStatementCandidateFactId::new(1),
                    origin_path: self.projection.origin_path().clone(),
                    proof_scope: LabelScopePath::new(vec![0]),
                    source_range: range(self.source, 77, 78),
                    source_ordinal: 0,
                    visible_after_ordinal: 1,
                    spelling: "A".to_owned(),
                    kind: SourceStatementLabelKind::ProofStep,
                    recovery: SourceStatementRecovery::Normal,
                }],
                citations: vec![SourceStatementCitationInput {
                    statement: SourceStatementId::new(3),
                    context: SourceStatementContextId::new(3),
                    label: SourceStatementLabelId::new(0),
                    label_ref: self.resolution.ids()[0],
                    proof_scope: LabelScopePath::new(vec![0]),
                    source_range: range(self.source, 131, 132),
                    ordinal: 0,
                    kind: SourceStatementCitationKind::SimpleLocal,
                    recovery: SourceStatementRecovery::Normal,
                }],
            }
        }

        fn references(
            &self,
            input: SourceStatementReferenceHandoffInput,
        ) -> Result<SourceStatementReferenceHandoff, SourceStatementReferenceError> {
            SourceStatementReferenceProducer::build(
                input,
                &self.statement,
                &self.resolver_ast,
                &self.projection,
                &self.reference,
                &self.resolution,
                &self.arena,
            )
        }

        fn empty_typed(&self) -> TypedAst {
            TypedAst::try_new(TypedAstParts {
                source_id: self.source,
                module_id: self.module.clone(),
                resolved_root: None,
                source_context: None,
                source_type: None,
                source_attribute: None,
                nodes: self.arena.clone(),
                contexts: LocalTypeContextTable::new(),
                types: TypeTable::new(),
                facts: TypeFactTable::new(),
                coercions: CoercionTable::new(),
                initial_obligations: InitialObligationTable::new(),
                diagnostics: TypeDiagnosticTable::new(),
            })
            .expect("empty typed AST")
            .with_source_term(self.primary.clone())
            .expect("Task252")
            .with_source_atomic_formula(self.atomic.clone())
            .expect("Task256")
        }
    }

    #[test]
    fn task258b1_reference_api_debug_and_task258a_compatibility() {
        let fixture = B1Fixture::new(20);
        let references = fixture
            .references(fixture.reference_input())
            .expect("Task258B1 references");
        assert_eq!(references.source_id(), fixture.source);
        assert_eq!(references.module_id(), &fixture.module);
        assert_eq!(
            references.statement_fingerprint(),
            fixture.statement.debug_text()
        );
        assert_eq!(fixture.statement.binding_env(), &fixture.bindings);
        assert_eq!(references.resolver_ast(), &fixture.resolver_ast);
        assert_eq!(references.label_projection(), &fixture.projection);
        assert_eq!(references.reference_candidate(), &fixture.reference);
        assert_eq!(references.label_resolution(), &fixture.resolution);
        assert_eq!(fixture.symbols.module_id(), &fixture.module);

        let label_id = SourceStatementLabelId::new(0);
        let label = references.labels().get(label_id).expect("label");
        assert_eq!(label_id.index(), 0);
        assert_eq!(label.statement(), SourceStatementId::new(1));
        assert_eq!(label.context(), SourceStatementContextId::new(1));
        assert_eq!(label.candidate(), SourceStatementCandidateFactId::new(1));
        assert_eq!(label.origin_path(), fixture.projection.origin_path());
        assert_eq!(label.proof_scope().path(), [0]);
        assert_eq!(label.source_range(), range(fixture.source, 77, 78));
        assert_eq!(label.source_ordinal(), 0);
        assert_eq!(label.visible_after_ordinal(), 1);
        assert_eq!(label.spelling(), "A");
        assert_eq!(label.kind(), SourceStatementLabelKind::ProofStep);
        assert_eq!(label.recovery(), SourceStatementRecovery::Normal);
        assert_eq!(references.labels().iter().count(), 1);
        assert!(!references.labels().is_empty());
        assert_eq!(
            references.labels().get(SourceStatementLabelId::new(1)),
            None
        );

        let citation_id = SourceStatementCitationId::new(0);
        let citation = references.citations().get(citation_id).expect("citation");
        assert_eq!(citation_id.index(), 0);
        assert_eq!(citation.statement(), SourceStatementId::new(3));
        assert_eq!(citation.context(), SourceStatementContextId::new(3));
        assert_eq!(citation.label(), label_id);
        assert_eq!(citation.label_ref(), fixture.resolution.ids()[0]);
        assert_eq!(citation.proof_scope().path(), [0]);
        assert_eq!(citation.source_range(), range(fixture.source, 131, 132));
        assert_eq!(citation.ordinal(), 0);
        assert_eq!(citation.kind(), SourceStatementCitationKind::SimpleLocal);
        assert_eq!(citation.recovery(), SourceStatementRecovery::Normal);
        assert_eq!(references.citations().iter().count(), 1);
        assert!(!references.citations().is_empty());
        assert_eq!(
            references
                .citations()
                .get(SourceStatementCitationId::new(1)),
            None
        );

        let debug = references.debug_text();
        for line in [
            "source-statement-reference-debug-v1\n",
            "module: pkg::statement.fixture\n",
            "resolver-ast root=76 nodes=77 name_refs=0 label_refs=1 imports=0 exports=0 label_node=12 reference_node=68 reference_state=resolved reference_key=label#0\n",
            "resolver-projection origin=pkg::statement.fixture::proof::A namespace=statement.fixture range=77..78 visible_after=1 scope=[0] kind=proof-step visibility=private export=local-only spelling=\"A\"\n",
            "resolver-reference node=68 range=131..132 source_ordinal=3 scope=[0] expectation=proof-or-theorem spelling=\"A\"\n",
            "resolver-result index=1 references=1 ids=[0] diagnostics=0\n",
            "label#0 statement=1 context=1 candidate=1 origin=pkg::statement.fixture::proof::A scope=[0] range=77..78 source_ordinal=0 visible_after=1 kind=proof-step recovery=normal spelling=\"A\"\n",
            "citation#0 statement=3 context=3 label=0 label_ref=0 scope=[0] range=131..132 ordinal=0 kind=simple-local recovery=normal\n",
        ] {
            assert!(debug.contains(line), "missing debug line: {line:?}");
        }
        assert_eq!(references.clone(), references);

        let task258a = Fixture::new(21);
        let base = task258a
            .build(task258a.input())
            .expect("Task258A compatibility");
        assert!(base.is_task_258a_profile());
        assert_eq!(base.debug_text(), base.clone().debug_text());
    }

    #[test]
    fn task258b1_dependency_aggregate_label_citation_and_provenance_fail_closed() {
        let fixture = B1Fixture::new(22);
        let input = fixture.reference_input();
        let valid = fixture
            .references(input.clone())
            .expect("Task258B1 baseline");

        let mut invalid_owner = fixture.statement.clone();
        invalid_owner.owners.rows[0].source_range = range(fixture.source, 19, 137);
        assert_eq!(
            invalid_owner.validate_installation(
                fixture.source,
                &fixture.module,
                &fixture.primary,
                &fixture.atomic,
                &fixture.arena,
            ),
            Err(SourceStatementError::InvalidOwner {
                owner: SourceTheoremOwnerId::new(0)
            })
        );
        let mut invalid_statement = fixture.statement.clone();
        invalid_statement.statements.rows[2].source_ordinal = 3;
        assert_eq!(
            invalid_statement.validate_installation(
                fixture.source,
                &fixture.module,
                &fixture.primary,
                &fixture.atomic,
                &fixture.arena,
            ),
            Err(SourceStatementError::InvalidStatement {
                statement: SourceStatementId::new(2)
            })
        );
        let mut invalid_context = fixture.statement.clone();
        invalid_context.contexts.rows[2].binding_context = BindingContextId::new(1);
        assert_eq!(
            invalid_context.validate_installation(
                fixture.source,
                &fixture.module,
                &fixture.primary,
                &fixture.atomic,
                &fixture.arena,
            ),
            Err(SourceStatementError::InvalidContext {
                context: SourceStatementContextId::new(2)
            })
        );
        let mut invalid_input = fixture.statement.clone();
        invalid_input.input_facts.rows[2].uses.swap(0, 1);
        assert_eq!(
            invalid_input.validate_installation(
                fixture.source,
                &fixture.module,
                &fixture.primary,
                &fixture.atomic,
                &fixture.arena,
            ),
            Err(SourceStatementError::InvalidInputFact {
                fact: SourceStatementInputFactId::new(2)
            })
        );
        let mut invalid_candidate = fixture.statement.clone();
        invalid_candidate.candidate_facts.rows[2].ordinal = 1;
        assert_eq!(
            invalid_candidate.validate_installation(
                fixture.source,
                &fixture.module,
                &fixture.primary,
                &fixture.atomic,
                &fixture.arena,
            ),
            Err(SourceStatementError::InvalidCandidateFact {
                fact: SourceStatementCandidateFactId::new(2)
            })
        );
        for crossed in [
            mutate_arena(&fixture.arena, |id, row| {
                if id == TypedNodeId::new(71) {
                    row.children.push(TypedNodeId::new(58));
                }
            }),
            mutate_arena(&fixture.arena, |id, row| {
                if id == TypedNodeId::new(58) {
                    row.children.push(TypedNodeId::new(65));
                }
            }),
        ] {
            assert_eq!(
                fixture.statement.validate_installation(
                    fixture.source,
                    &fixture.module,
                    &fixture.primary,
                    &fixture.atomic,
                    &crossed,
                ),
                Err(SourceStatementError::InvalidStatement {
                    statement: SourceStatementId::new(0)
                })
            );
        }

        let mut missing = input.clone();
        missing.labels.clear();
        assert_eq!(
            fixture.references(missing),
            Err(SourceStatementReferenceError::InvalidAggregate)
        );
        let mut extra = input.clone();
        extra.citations.push(extra.citations[0].clone());
        assert_eq!(
            fixture.references(extra),
            Err(SourceStatementReferenceError::InvalidAggregate)
        );

        let mut invalid_label = input.clone();
        invalid_label.labels[0].visible_after_ordinal = 0;
        assert_eq!(
            fixture.references(invalid_label),
            Err(SourceStatementReferenceError::InvalidLabel {
                label: SourceStatementLabelId::new(0)
            })
        );
        let mut invalid_citation = input;
        invalid_citation.citations[0].ordinal = 1;
        assert_eq!(
            fixture.references(invalid_citation),
            Err(SourceStatementReferenceError::InvalidCitation {
                citation: SourceStatementCitationId::new(0)
            })
        );

        let mut dependency_and_rows = valid.clone();
        dependency_and_rows.statement_fingerprint.push_str(":stale");
        dependency_and_rows.labels.rows[0].visible_after_ordinal = 0;
        dependency_and_rows.citations.rows[0].ordinal = 1;
        assert_eq!(
            dependency_and_rows.validate_installation(
                fixture.source,
                &fixture.module,
                &fixture.statement,
                &fixture.arena,
            ),
            Err(SourceStatementReferenceError::DependencyMismatch)
        );
        let mut aggregate_and_rows = valid.clone();
        aggregate_and_rows.labels.rows.clear();
        aggregate_and_rows.citations.rows[0].ordinal = 1;
        assert_eq!(
            aggregate_and_rows.validate_installation(
                fixture.source,
                &fixture.module,
                &fixture.statement,
                &fixture.arena,
            ),
            Err(SourceStatementReferenceError::InvalidAggregate)
        );
        let mut label_and_citation = valid.clone();
        label_and_citation.labels.rows[0].visible_after_ordinal = 0;
        label_and_citation.citations.rows[0].ordinal = 1;
        assert_eq!(
            label_and_citation.validate_installation(
                fixture.source,
                &fixture.module,
                &fixture.statement,
                &fixture.arena,
            ),
            Err(SourceStatementReferenceError::InvalidLabel {
                label: SourceStatementLabelId::new(0)
            })
        );

        for mutation in [
            B1ResolverMutation::Root,
            B1ResolverMutation::Count,
            B1ResolverMutation::StructuralPath,
            B1ResolverMutation::Anchor,
            B1ResolverMutation::Children,
            B1ResolverMutation::Recovered,
            B1ResolverMutation::ReferenceWithoutKey,
            B1ResolverMutation::ExtraResolution,
        ] {
            let mut corrupted = valid.clone();
            corrupted.resolver_ast = b1_mutated_resolver_ast(&fixture, mutation);
            assert_eq!(
                corrupted.validate_installation(
                    fixture.source,
                    &fixture.module,
                    &fixture.statement,
                    &fixture.arena,
                ),
                Err(SourceStatementReferenceError::DependencyMismatch)
            );
        }
        assert!(matches!(
            b1_mutated_resolver_ast_result(&fixture, B1ResolverMutation::OriginSource),
            Err(mizar_resolve::resolved_ast::ResolvedAstError::OriginSourceMismatch)
        ));
        assert!(matches!(
            b1_mutated_resolver_ast_result(&fixture, B1ResolverMutation::OriginModule),
            Err(mizar_resolve::resolved_ast::ResolvedAstError::NodeModuleMismatch { .. })
        ));

        let auxiliary_node = fixture
            .resolver_ast
            .nodes()
            .iter()
            .find_map(|(id, _)| (id.index() == 67).then_some(id))
            .expect("auxiliary resolver node");
        let auxiliary_range = range(fixture.source, 117, 127);
        let auxiliary_origin = SemanticOrigin::new(
            fixture.source,
            fixture.module.clone(),
            SourceAnchor::Range(auxiliary_range),
            vec![67],
        );
        let mut name_refs = mizar_resolve::resolved_ast::NameRefTable::new();
        name_refs.insert(mizar_resolve::resolved_ast::NameRefEntry::new(
            mizar_resolve::resolved_ast::ReferenceSite::new(
                auxiliary_node,
                auxiliary_range,
                "missing",
            ),
            mizar_resolve::resolved_ast::NameResolution::Unresolved(
                mizar_resolve::resolved_ast::UnresolvedNameRef::new(
                    "missing",
                    auxiliary_range,
                    mizar_resolve::resolved_ast::NameLookupClass::Symbol,
                ),
            ),
            auxiliary_origin.clone(),
        ));
        let mut corrupted = valid.clone();
        corrupted.resolver_ast = ResolvedAst::try_new(
            fixture.source,
            fixture.module.clone(),
            fixture.resolver_ast.nodes().clone(),
            name_refs,
            fixture.resolution.table().clone(),
            mizar_resolve::resolved_ast::ResolvedImports::new(),
        )
        .expect("valid resolver AST with an extra name table");
        assert_eq!(
            corrupted.validate_installation(
                fixture.source,
                &fixture.module,
                &fixture.statement,
                &fixture.arena,
            ),
            Err(SourceStatementReferenceError::DependencyMismatch)
        );

        for include_export in [false, true] {
            let mut imports = mizar_resolve::resolved_ast::ResolvedImports::new();
            let nodes = if include_export {
                imports.push_export(mizar_resolve::resolved_ast::ResolvedExport::new(
                    auxiliary_node,
                    auxiliary_range,
                    "missing",
                    mizar_resolve::resolved_ast::ExportTarget::Unresolved(
                        mizar_resolve::resolved_ast::UnresolvedExport::new(
                            "missing",
                            auxiliary_range,
                            mizar_resolve::resolved_ast::ExportFailureClass::TargetNotFound,
                        ),
                    ),
                    auxiliary_origin.clone(),
                ));
                fixture.resolver_ast.nodes().clone()
            } else {
                let import = imports.push_import(mizar_resolve::resolved_ast::ResolvedImport::new(
                    auxiliary_node,
                    auxiliary_range,
                    "missing",
                    None,
                    mizar_resolve::resolved_ast::ImportResolution::Unresolved(
                        mizar_resolve::resolved_ast::UnresolvedImport::new(
                            "missing",
                            auxiliary_range,
                            mizar_resolve::resolved_ast::ImportFailureClass::ModuleNotFound,
                        ),
                    ),
                    auxiliary_origin.clone(),
                ));
                b1_resolved_arena_with_import_edge(&fixture, import)
            };
            let mut corrupted = valid.clone();
            corrupted.resolver_ast = ResolvedAst::try_new(
                fixture.source,
                fixture.module.clone(),
                nodes,
                mizar_resolve::resolved_ast::NameRefTable::new(),
                fixture.resolution.table().clone(),
                imports,
            )
            .expect("valid resolver AST with directive table");
            assert_eq!(
                corrupted.validate_installation(
                    fixture.source,
                    &fixture.module,
                    &fixture.statement,
                    &fixture.arena,
                ),
                Err(SourceStatementReferenceError::DependencyMismatch)
            );
        }

        let mut alternate_labels = mizar_resolve::resolved_ast::LabelRefTable::new();
        alternate_labels.insert(mizar_resolve::resolved_ast::LabelRefEntry::new(
            fixture.reference.site().clone(),
            mizar_resolve::resolved_ast::LabelResolution::Unresolved(
                mizar_resolve::resolved_ast::UnresolvedLabelRef::new(
                    "A",
                    fixture.reference.site().range(),
                    LabelExpectation::ProofOrTheorem,
                ),
            ),
            fixture.reference.origin().clone(),
        ));
        let mut corrupted = valid.clone();
        corrupted.resolver_ast = ResolvedAst::try_new(
            fixture.source,
            fixture.module.clone(),
            fixture.resolver_ast.nodes().clone(),
            mizar_resolve::resolved_ast::NameRefTable::new(),
            alternate_labels,
            mizar_resolve::resolved_ast::ResolvedImports::new(),
        )
        .expect("valid resolver AST with stale label table");
        assert_eq!(
            corrupted.validate_installation(
                fixture.source,
                &fixture.module,
                &fixture.statement,
                &fixture.arena,
            ),
            Err(SourceStatementReferenceError::DependencyMismatch)
        );

        let mut arena_rows = fixture
            .arena
            .iter()
            .map(|(_, row)| row.clone())
            .collect::<Vec<_>>();
        arena_rows.push(TypedNode::new(
            "source.surface.unowned",
            SourceAnchor::Range(range(fixture.source, 0, 0)),
        ));
        let typed_count = TypedArena::try_new(fixture.arena.root(), arena_rows)
            .expect("typed count mutation is structurally valid");
        let typed_root = TypedArena::try_new(
            Some(TypedNodeId::new(75)),
            fixture.arena.iter().map(|(_, row)| row.clone()).collect(),
        )
        .expect("typed root mutation is structurally valid");
        let typed_anchor = mutate_arena(&fixture.arena, |id, row| {
            if id == TypedNodeId::new(50) {
                row.anchor = SourceAnchor::Range(range(fixture.source, 0, 0));
            }
        });
        let typed_children = mutate_arena(&fixture.arena, |id, row| {
            if id == TypedNodeId::new(71) {
                row.children.push(TypedNodeId::new(58));
            }
        });
        for arena in [typed_count, typed_root, typed_anchor, typed_children] {
            assert_eq!(
                valid.validate_installation(
                    fixture.source,
                    &fixture.module,
                    &fixture.statement,
                    &arena,
                ),
                Err(SourceStatementReferenceError::DependencyMismatch)
            );
        }
        for recovery in [NodeRecoveryState::Recovered, NodeRecoveryState::Degraded] {
            let arena = mutate_arena(&fixture.arena, |id, row| {
                if id == TypedNodeId::new(50) {
                    row.recovery = recovery;
                }
            });
            assert_eq!(
                valid.validate_installation(
                    fixture.source,
                    &fixture.module,
                    &fixture.statement,
                    &arena,
                ),
                Err(SourceStatementReferenceError::DependencyMismatch)
            );
        }

        let public_projection = fixture
            .projection
            .clone()
            .with_visibility(Visibility::Public);
        let public_resolution = LabelResolver::new(std::slice::from_ref(&public_projection))
            .resolve(
                &fixture.module,
                &NamespacePath::new(fixture.module.path().as_str()),
                std::slice::from_ref(&fixture.reference),
            );
        let public_ast = ResolvedAst::try_new(
            fixture.source,
            fixture.module.clone(),
            b1_resolved_arena(
                fixture.source,
                &fixture.module,
                Some(public_resolution.ids()[0]),
            ),
            mizar_resolve::resolved_ast::NameRefTable::new(),
            public_resolution.table().clone(),
            mizar_resolve::resolved_ast::ResolvedImports::new(),
        )
        .expect("coherent public resolver AST");
        assert_eq!(
            SourceStatementReferenceProducer::build(
                fixture.reference_input(),
                &fixture.statement,
                &public_ast,
                &public_projection,
                &fixture.reference,
                &public_resolution,
                &fixture.arena,
            ),
            Err(SourceStatementReferenceError::InvalidLabel {
                label: SourceStatementLabelId::new(0)
            })
        );

        let late_reference = LabelReferenceCandidate::unqualified_citation(
            fixture.reference.site().clone(),
            fixture.reference.origin().clone(),
            4,
            Some(LabelScopePath::new(vec![0])),
        );
        let late_resolution = LabelResolver::new(std::slice::from_ref(&fixture.projection))
            .resolve(
                &fixture.module,
                &NamespacePath::new(fixture.module.path().as_str()),
                std::slice::from_ref(&late_reference),
            );
        let late_ast = ResolvedAst::try_new(
            fixture.source,
            fixture.module.clone(),
            b1_resolved_arena(
                fixture.source,
                &fixture.module,
                Some(late_resolution.ids()[0]),
            ),
            mizar_resolve::resolved_ast::NameRefTable::new(),
            late_resolution.table().clone(),
            mizar_resolve::resolved_ast::ResolvedImports::new(),
        )
        .expect("coherent late-reference resolver AST");
        assert_eq!(
            SourceStatementReferenceProducer::build(
                fixture.reference_input(),
                &fixture.statement,
                &late_ast,
                &fixture.projection,
                &late_reference,
                &late_resolution,
                &fixture.arena,
            ),
            Err(SourceStatementReferenceError::InvalidCitation {
                citation: SourceStatementCitationId::new(0)
            })
        );

        let wrong_reference = LabelReferenceCandidate::unqualified_citation(
            mizar_resolve::resolved_ast::ReferenceSite::new(
                fixture.reference.site().node(),
                range(fixture.source, 130, 132),
                "A",
            ),
            SemanticOrigin::new(
                fixture.source,
                fixture.module.clone(),
                SourceAnchor::Range(range(fixture.source, 130, 132)),
                vec![68],
            ),
            3,
            Some(LabelScopePath::new(vec![0])),
        );
        let mut stale_provenance = valid.clone();
        stale_provenance.reference_candidate = wrong_reference;
        assert_eq!(
            stale_provenance.validate_installation(
                fixture.source,
                &fixture.module,
                &fixture.statement,
                &fixture.arena,
            ),
            Err(SourceStatementReferenceError::DependencyMismatch)
        );

        let mut contexts = BindingContextTable::new();
        contexts.insert(BindingContextDraft {
            owner: BindingContextOwner::Module,
            parent: None,
            layer: BindingContextLayer::Module,
            lexical_scope: None,
            bindings: Vec::new(),
            visible_bindings: Vec::new(),
            recovery: BindingContextRecovery::Normal,
        });
        contexts.insert(BindingContextDraft {
            owner: BindingContextOwner::SourceStatement {
                source_range: range(fixture.source, 10, 10),
            },
            parent: Some(BindingContextId::new(0)),
            layer: BindingContextLayer::Proof,
            lexical_scope: Some(LocalTermScope::new(vec![0])),
            bindings: Vec::new(),
            visible_bindings: Vec::new(),
            recovery: BindingContextRecovery::Normal,
        });
        assert!(
            BindingEnv::try_new(BindingEnvParts {
                source_id: fixture.source,
                module_id: fixture.module.clone(),
                contexts,
                bindings: BindingTable::new(),
                diagnostics: BindingDiagnosticTable::new(),
            })
            .is_err()
        );
        assert!(exact_binding_profile(
            StatementProfile::Task258B1,
            fixture.source,
            &fixture.bindings,
        ));
        let binding_debug = fixture.bindings.debug_text();
        assert!(binding_debug.contains("context#1 owner=source-statement(69..137)"));
        assert!(binding_debug.contains("context#2 owner=source-statement(86..113)"));
        assert!(matches!(
            b1_binding_env_with_mutation(fixture.source, &fixture.module, |contexts| {
                contexts[2].parent = None;
            }),
            Err(crate::binding_env::BindingEnvError::MultipleRootContexts { .. })
        ));
        for mutation in 1..6 {
            let altered =
                b1_binding_env_with_mutation(fixture.source, &fixture.module, |contexts| {
                    match mutation {
                        1 => contexts[1].layer = BindingContextLayer::Block,
                        2 => contexts[2].lexical_scope = Some(LocalTermScope::new(vec![0])),
                        3 => contexts[1].visible_bindings.clear(),
                        4 => contexts[2].recovery = BindingContextRecovery::Recovered,
                        5 => {
                            contexts[1].owner = BindingContextOwner::SourceStatement {
                                source_range: range(fixture.source, 69, 136),
                            };
                        }
                        _ => unreachable!(),
                    }
                })
                .expect("binding mutation remains a valid generic environment");
            assert!(
                !exact_binding_profile(StatementProfile::Task258B1, fixture.source, &altered),
                "binding mutation {mutation} must leave the frozen profile"
            );
            assert_ne!(altered.debug_text(), fixture.bindings.debug_text());
        }
        let wrong_source = source_id(222);
        let wrong_source_bindings = b1_binding_env(wrong_source, &fixture.module);
        assert!(!exact_binding_profile(
            StatementProfile::Task258B1,
            fixture.source,
            &wrong_source_bindings,
        ));
    }

    #[test]
    fn task258b1_typed_pair_installation_ownership_and_rollback_are_atomic() {
        let fixture = B1Fixture::new(23);
        let references = fixture
            .references(fixture.reference_input())
            .expect("Task258B1 references");
        let empty = fixture.empty_typed();
        let before = empty.clone();
        let installed = empty
            .clone()
            .with_source_statement_references(fixture.statement.clone(), references.clone())
            .expect("atomic pair installation");
        assert_eq!(installed.source_statement(), Some(&fixture.statement));
        assert_eq!(installed.source_statement_references(), Some(&references));
        assert_eq!(before, fixture.empty_typed());
        assert_eq!(
            fixture
                .empty_typed()
                .with_source_statement(fixture.statement.clone()),
            Err(TypedAstError::InvalidSourceStatement)
        );
        assert_eq!(
            installed
                .clone()
                .with_source_statement_references(fixture.statement.clone(), references.clone()),
            Err(TypedAstError::InvalidSourceStatement)
        );

        let task248 = crate::source_context::tests::task_248_occupied_typed_ast(
            fixture.source,
            fixture.module.clone(),
        );
        let task248_debug = task248.debug_text();
        assert_eq!(
            task248
                .clone()
                .with_source_statement_references(fixture.statement.clone(), references.clone(),),
            Err(TypedAstError::InvalidSourceStatement)
        );
        assert_eq!(task248.debug_text(), task248_debug);
        let mut context_collision = installed.clone();
        assert_eq!(
            context_collision.clone().with_source_context_for_test(
                task248
                    .source_context()
                    .expect("Task248 source context")
                    .clone(),
            ),
            Err(TypedAstError::InvalidSourceContext)
        );
        assert_eq!(context_collision, installed);

        let task257 = crate::source_composite_formula::tests::task_257a_installed_typed_ast();
        let task257_debug = task257.debug_text();
        assert_eq!(
            task257
                .clone()
                .with_source_statement_references(fixture.statement.clone(), references.clone(),),
            Err(TypedAstError::InvalidSourceStatement)
        );
        assert_eq!(task257.debug_text(), task257_debug);
        assert_eq!(
            installed.clone().with_source_composite_formula(
                task257
                    .source_composite_formula()
                    .expect("Task257A composite formula")
                    .clone(),
            ),
            Err(TypedAstError::InvalidSourceCompositeFormula)
        );
        context_collision.inject_source_composite_formula_for_test(
            task257
                .source_composite_formula()
                .expect("Task257A composite formula")
                .clone(),
        );
        assert_eq!(
            assemble_empty_resolved(&context_collision),
            Err(ResolvedTypedAstError::InvalidSourceCompositeFormula)
        );

        let task257b = crate::source_formula_composition::tests::task_257b_installed_typed_ast();
        let task257c2 = crate::source_formula_composition::tests::task_257c2_installed_typed_ast();
        let task257c3 = crate::source_formula_composition::tests::task_257c3_installed_typed_ast();
        for (family, occupied) in [
            ("Task257B", &task257b),
            ("Task257C2", &task257c2),
            ("Task257C3", &task257c3),
        ] {
            let debug = occupied.debug_text();
            assert_eq!(
                occupied.clone().with_source_statement_references(
                    fixture.statement.clone(),
                    references.clone(),
                ),
                Err(TypedAstError::InvalidSourceStatement),
                "{family}"
            );
            assert_eq!(occupied.debug_text(), debug, "{family}");
        }
        assert_eq!(
            installed.clone().with_source_formula_composition(
                task257b
                    .source_composite_formula()
                    .expect("Task257B composite")
                    .clone(),
                task257b
                    .source_formula_composition()
                    .expect("Task257B composition")
                    .clone(),
            ),
            Err(TypedAstError::InvalidSourceFormulaComposition)
        );
        assert_eq!(
            installed.clone().with_source_condition_formula_composition(
                task257c2
                    .source_condition_formula_composition()
                    .expect("Task257C2 composition")
                    .clone(),
            ),
            Err(TypedAstError::InvalidSourceConditionFormulaComposition)
        );
        assert_eq!(
            installed.clone().with_source_predicate_chain_composition(
                task257c3
                    .source_predicate_chain_composition()
                    .expect("Task257C3 composition")
                    .clone(),
            ),
            Err(TypedAstError::InvalidSourcePredicateChainComposition)
        );

        let task258a = Fixture::new(23);
        let task258a_statement = task258a
            .build(task258a.input())
            .expect("Task258A statement");
        assert_eq!(
            fixture
                .empty_typed()
                .with_source_statement_references(task258a_statement, references.clone()),
            Err(TypedAstError::InvalidSourceStatement)
        );
        let task258a_installed = TypedAst::try_new(TypedAstParts {
            source_id: task258a.source,
            module_id: task258a.module.clone(),
            resolved_root: None,
            source_context: None,
            source_type: None,
            source_attribute: None,
            nodes: task258a.arena.clone(),
            contexts: LocalTypeContextTable::new(),
            types: TypeTable::new(),
            facts: TypeFactTable::new(),
            coercions: CoercionTable::new(),
            initial_obligations: InitialObligationTable::new(),
            diagnostics: TypeDiagnosticTable::new(),
        })
        .expect("Task258A empty typed AST")
        .with_source_term(task258a.primary.clone())
        .expect("Task258A Task252")
        .with_source_atomic_formula(task258a.atomic.clone())
        .expect("Task258A Task256")
        .with_source_statement(
            task258a
                .build(task258a.input())
                .expect("Task258A statement replay"),
        )
        .expect("Task258A install");
        let task258a_debug = task258a_installed.debug_text();
        assert_eq!(
            task258a_installed
                .clone()
                .with_source_statement_references(fixture.statement.clone(), references.clone(),),
            Err(TypedAstError::InvalidSourceStatement)
        );
        assert_eq!(task258a_installed.debug_text(), task258a_debug);

        let mut semantic_facts = TypeFactTable::new();
        semantic_facts.insert(TypeFactDraft {
            subject: node(38),
            predicate: TypePredicateRef::new("set"),
            polarity: Polarity::Positive,
            provenance: FactProvenance::Inferred(TypeRuleId::new(
                "statement-reference-coexistence",
            )),
            status: FactStatus::Known,
        });
        let semantic_base = TypedAst::try_new(TypedAstParts {
            source_id: fixture.source,
            module_id: fixture.module.clone(),
            resolved_root: None,
            source_context: None,
            source_type: None,
            source_attribute: None,
            nodes: fixture.arena.clone(),
            contexts: LocalTypeContextTable::new(),
            types: TypeTable::new(),
            facts: semantic_facts,
            coercions: CoercionTable::new(),
            initial_obligations: InitialObligationTable::new(),
            diagnostics: TypeDiagnosticTable::new(),
        })
        .expect("semantic typed base")
        .with_source_term(fixture.primary.clone())
        .expect("Task252 semantic base")
        .with_source_atomic_formula(fixture.atomic.clone())
        .expect("Task256 semantic base");
        let semantic_debug = semantic_base.debug_text();
        assert_eq!(
            semantic_base
                .clone()
                .with_source_statement_references(fixture.statement.clone(), references.clone(),),
            Err(TypedAstError::InvalidSourceStatement)
        );
        assert_eq!(semantic_base.debug_text(), semantic_debug);

        let mut stale = references.clone();
        stale.statement_fingerprint.push_str(":stale");
        assert_eq!(
            fixture
                .empty_typed()
                .with_source_statement_references(fixture.statement.clone(), stale),
            Err(TypedAstError::InvalidSourceStatement)
        );
        assert_eq!(before, fixture.empty_typed());
        assert_eq!(
            before
                .with_source_statement_references(fixture.statement.clone(), references)
                .expect("valid replay")
                .debug_text(),
            installed.debug_text()
        );
    }

    #[test]
    fn task258b1_final_revalidation_clone_debug_and_semantic_boundary() {
        let fixture = B1Fixture::new(24);
        let references = fixture
            .references(fixture.reference_input())
            .expect("Task258B1 references");
        let typed = fixture
            .empty_typed()
            .with_source_statement_references(fixture.statement.clone(), references.clone())
            .expect("Task258B1 typed AST");
        let resolved = assemble_empty_resolved(&typed).expect("Task258B1 final assembly");
        assert_eq!(resolved.source_statement(), Some(&fixture.statement));
        assert_eq!(resolved.source_statement_references(), Some(&references));
        assert!(resolved.statement_semantics().is_empty());
        assert!(resolved.checked_formulas().is_empty());
        assert!(resolved.checked_proofs().is_empty());
        assert!(resolved.checked_proof_nodes().is_empty());
        assert!(resolved.checked_terminal_goals().is_empty());
        assert_eq!(resolved.clone(), resolved);
        let debug = resolved.debug_text();
        let statement = debug
            .find("source-statement-debug-v1\n")
            .expect("base debug");
        let references_debug = debug
            .find("source-statement-reference-debug-v1\n")
            .expect("reference debug");
        let nodes = debug.find("nodes:\n").expect("node debug");
        assert!(statement < references_debug && references_debug < nodes);

        let mut stale = fixture
            .references(fixture.reference_input())
            .expect("Task258B1 references");
        stale.statement_fingerprint.push_str(":stale");
        let mut corrupted = typed.clone();
        corrupted.inject_source_statement_bundle_for_test(fixture.statement.clone(), stale);
        assert_eq!(
            assemble_empty_resolved(&corrupted),
            Err(ResolvedTypedAstError::InvalidSourceStatement)
        );

        let mut missing_references = fixture.empty_typed();
        missing_references.inject_source_statement_for_test(fixture.statement.clone());
        assert_eq!(
            assemble_empty_resolved(&missing_references),
            Err(ResolvedTypedAstError::InvalidSourceStatement)
        );
        let mut missing_statement = fixture.empty_typed();
        missing_statement.inject_source_statement_references_for_test(references.clone());
        assert_eq!(
            assemble_empty_resolved(&missing_statement),
            Err(ResolvedTypedAstError::InvalidSourceStatement)
        );

        let task248 = crate::source_context::tests::task_248_occupied_typed_ast(
            fixture.source,
            fixture.module.clone(),
        );
        let mut context_collision = task248.clone();
        context_collision
            .inject_source_statement_bundle_for_test(fixture.statement.clone(), references.clone());
        assert_eq!(
            assemble_empty_resolved(&context_collision),
            Err(ResolvedTypedAstError::InvalidSourceStatement)
        );
        assert_eq!(task248, task248.clone());

        let mut cluster_facts = ClusterFactTable::new();
        cluster_facts.insert(ClusterFactDraft {
            fingerprint: ClusterFactFingerprint::new("statement-reference-coexistence"),
            source_type: ClusterTypeFingerprint::new("set"),
            attribute: ClusterAttributeFingerprint::new("inhabited"),
            generated_type: ClusterTypeFingerprint::new("set"),
            provenance: ClusterFactProvenance::Input,
            source_range: range(fixture.source, 63, 64),
        });
        assert_eq!(
            assemble_resolved(&typed, &cluster_facts, Vec::new(), None),
            Err(ResolvedTypedAstError::InvalidSourceStatement)
        );
        assert_eq!(
            assemble_resolved(
                &typed,
                &ClusterFactTable::new(),
                vec![ResolvedNodeKindHint {
                    typed_node: TypedNodeId::new(38),
                    kind: ResolvedNodeKindHintKind::SourcePreserved {
                        role: SourceNodeRole::new(
                            "source.statement-reference.semantic-coexistence",
                        ),
                    },
                }],
                None,
            ),
            Err(ResolvedTypedAstError::InvalidSourceStatement)
        );
        assert_eq!(
            assemble_resolved(
                &typed,
                &ClusterFactTable::new(),
                Vec::new(),
                Some(StatementProofInputs {
                    owner: fixture.statement.checked_owner(),
                    rows: Vec::new(),
                }),
            ),
            Err(ResolvedTypedAstError::InvalidSourceStatement)
        );
        for table in [
            StatementTransportTableForTest::Context,
            StatementTransportTableForTest::Type,
            StatementTransportTableForTest::Fact,
            StatementTransportTableForTest::Coercion,
            StatementTransportTableForTest::InitialObligation,
            StatementTransportTableForTest::Diagnostic,
        ] {
            let mut occupied = typed.clone();
            occupied.occupy_statement_transport_table_for_test(table);
            assert_eq!(
                assemble_empty_resolved(&occupied),
                Err(ResolvedTypedAstError::InvalidSourceStatement),
                "{table:?}"
            );
        }
        assert_eq!(
            assemble_resolved_with_expressions(
                &typed,
                vec![ExpressionMetadataInput {
                    expr: ExprId::new("statement-transport-guard"),
                    typed_site: node(38),
                    local_context: None,
                    cluster_facts: Vec::new(),
                }],
            ),
            Err(ResolvedTypedAstError::InvalidSourceStatement)
        );
        let term_formula = TermFormulaChecker::default().infer(
            &fixture.symbols,
            &fixture.bindings,
            Vec::<crate::type_checker::TermInput>::new(),
            Vec::<crate::type_checker::FormulaInput>::new(),
        );
        assert_eq!(
            assemble_resolved_with_statement_semantics(
                &typed,
                fixture.statement.checked_owner(),
                &fixture.bindings,
                &term_formula,
            ),
            Err(ResolvedTypedAstError::InvalidSourceStatement)
        );
        let collection = OverloadCollectionOutput::collect(
            vec![OverloadSiteInput {
                key: OverloadSiteKey::new("statement-transport-guard"),
                owner: node(38),
                source_range: range(fixture.source, 63, 64),
                kind: OverloadSiteKind::FunctorApplication,
                name: OverloadNameKey::new("statement-transport-guard"),
                arguments: vec![TypedSiteRef::Role {
                    node: TypedNodeId::new(38),
                    role: TypeRole::new("argument"),
                }],
                expected: None,
                source_qua: Vec::<SourceQuaView>::new(),
                recovery: OverloadSiteRecovery::Normal,
            }],
            Vec::<OverloadCandidateInput>::new(),
        );
        let expansion = TemplateExpansionOutput::expand(&collection);
        let viability =
            CandidateViabilityOutput::filter(&expansion, Vec::<CandidateViabilityInput>::new());
        let specificity =
            SpecificityGraphOutput::build(&viability, Vec::<SpecificityComparisonInput>::new());
        let selection = OverloadSelectionOutput::resolve(
            &specificity,
            Vec::<OverloadSiteResolutionInput>::new(),
        );
        assert_eq!(collection.sites().len(), 1);
        assert_eq!(
            ResolvedTypedAst::assemble(ResolvedTypedAstInputs {
                typed_ast: &typed,
                cluster_facts: &ClusterFactTable::new(),
                overload_collection: &collection,
                template_expansion: &expansion,
                viability: &viability,
                specificity: &specificity,
                overload_selection: &selection,
                expressions: Vec::new(),
                node_hints: Vec::new(),
                statement_semantics: None,
                statement_proofs: None,
            }),
            Err(ResolvedTypedAstError::InvalidSourceStatement)
        );
        assert_eq!(
            assemble_empty_resolved(&typed)
                .expect("valid final replay")
                .debug_text(),
            resolved.debug_text()
        );
    }

    #[test]
    fn exact_profile_exposes_complete_api_and_stable_debug() {
        let fixture = Fixture::new(1);
        let handoff = fixture.build(fixture.input()).expect("valid statement");
        assert_eq!(handoff.source_id(), fixture.source);
        assert_eq!(handoff.module_id(), &fixture.module);
        assert_eq!(handoff.binding_env(), &fixture.bindings);
        assert_eq!(handoff.binding_fingerprint(), fixture.bindings.debug_text());
        assert_eq!(
            handoff.primary_term_fingerprint(),
            fixture.primary.debug_text()
        );
        assert_eq!(
            handoff.atomic_formula_fingerprint(),
            fixture.atomic.debug_text()
        );
        assert_eq!(handoff.checked_owner().symbol(), &fixture.symbol);

        let owner_id = SourceTheoremOwnerId::new(0);
        let owner = handoff.owners().get(owner_id).expect("owner");
        assert_eq!(owner_id.index(), 0);
        assert_eq!(owner.symbol(), &fixture.symbol);
        assert_eq!(owner.contribution(), fixture.contribution);
        assert_eq!(owner.site(), &node(5));
        assert_eq!(owner.source_range(), range(fixture.source, 19, 80));
        assert_eq!(owner.spelling(), LABEL);
        assert_eq!(owner.role(), SourceTheoremRole::Theorem);
        assert_eq!(owner.status(), SourceTheoremStatus::Unmodified);
        assert_eq!(owner.recovery(), SourceStatementRecovery::Normal);
        assert_eq!(handoff.owners().iter().count(), 1);
        assert!(!handoff.owners().is_empty());

        let statement_id = SourceStatementId::new(0);
        let statement = handoff.statements().get(statement_id).expect("statement");
        assert_eq!(statement.owner(), owner_id);
        assert_eq!(statement.context(), SourceStatementContextId::new(0));
        assert_eq!(
            statement.formula(),
            SourceStatementFormulaTarget::Atomic(SourceAtomicFormulaId::new(0))
        );
        assert_eq!(statement.site(), &node(5));
        assert_eq!(statement.source_range(), range(fixture.source, 19, 80));
        assert_eq!(statement.source_ordinal(), 0);
        assert_eq!(statement.spelling(), STATEMENT);
        assert_eq!(statement.kind(), SourceStatementKind::TheoremProposition);
        assert_eq!(statement.recovery(), SourceStatementRecovery::Normal);
        let theorem_node = fixture.arena.node(node(5).node()).expect("theorem node");
        assert_eq!(theorem_node.children, [TypedNodeId::new(4)]);
        let wrapper_node = fixture.arena.node(node(4).node()).expect("formula wrapper");
        assert_eq!(wrapper_node.kind.as_str(), "source.surface.unowned");
        assert_eq!(wrapper_node.children, [TypedNodeId::new(3)]);
        assert_eq!(handoff.statements().iter().count(), 1);
        assert!(!handoff.statements().is_empty());

        let context = handoff
            .contexts()
            .get(SourceStatementContextId::new(0))
            .expect("context");
        assert_eq!(context.statement(), statement_id);
        assert_eq!(context.binding_context(), BindingContextId::new(0));
        assert_eq!(context.source_range(), range(fixture.source, 19, 80));
        assert_eq!(context.visible_bindings(), [BindingId::new(0)]);
        assert_eq!(handoff.contexts().iter().count(), 1);
        assert!(!handoff.contexts().is_empty());

        let input_fact = handoff
            .input_facts()
            .get(SourceStatementInputFactId::new(0))
            .expect("input fact");
        assert_eq!(input_fact.statement(), statement_id);
        assert_eq!(input_fact.context(), SourceStatementContextId::new(0));
        assert_eq!(input_fact.ordinal(), 0);
        assert_eq!(
            input_fact.kind(),
            SourceStatementInputFactKind::ReservedTypeGuard
        );
        assert_eq!(input_fact.binding(), BindingId::new(0));
        assert_eq!(
            input_fact.uses(),
            [
                SourcePrimaryTermReferenceId::new(0),
                SourcePrimaryTermReferenceId::new(1)
            ]
        );
        assert_eq!(handoff.input_facts().iter().count(), 1);
        assert!(!handoff.input_facts().is_empty());

        let candidate = handoff
            .candidate_facts()
            .get(SourceStatementCandidateFactId::new(0))
            .expect("candidate");
        assert_eq!(candidate.statement(), statement_id);
        assert_eq!(candidate.context(), SourceStatementContextId::new(0));
        assert_eq!(candidate.ordinal(), 0);
        assert_eq!(
            candidate.kind(),
            SourceStatementCandidateFactKind::UnverifiedProposition
        );
        assert_eq!(candidate.formula(), statement.formula());
        assert_eq!(handoff.candidate_facts().iter().count(), 1);
        assert!(!handoff.candidate_facts().is_empty());

        let expected = format!(
            concat!(
                "source-statement-debug-v1\n",
                "module: pkg::statement.fixture\n",
                "binding-env-fingerprint: {:?}\n",
                "primary-term-fingerprint: {:?}\n",
                "atomic-formula-fingerprint: {:?}\n",
                "owner#0 symbol={:?} contribution={} role=theorem status=unmodified range=19..80 site=5 recovery=normal spelling={:?}\n",
                "statement#0 ordinal=0 owner=0 context=0 formula=atomic:0 kind=theorem-proposition range=19..80 site=5 recovery=normal spelling={:?}\n",
                "context#0 statement=0 binding_context=0 range=19..80 visible_bindings=[0]\n",
                "input-fact#0 statement=0 context=0 ordinal=0 kind=reserved-type-guard binding=0 uses=[0, 1]\n",
                "candidate-fact#0 statement=0 context=0 ordinal=0 kind=unverified-proposition formula=atomic:0\n",
            ),
            fixture.bindings.debug_text(),
            fixture.primary.debug_text(),
            fixture.atomic.debug_text(),
            fixture.symbol,
            fixture.contribution.index(),
            LABEL,
            STATEMENT,
        );
        assert_eq!(handoff.debug_text(), expected);
        assert_eq!(handoff.clone(), handoff);
        assert_eq!(handoff.owners().get(SourceTheoremOwnerId::new(1)), None);
        assert_eq!(handoff.statements().get(SourceStatementId::new(1)), None);
        assert_eq!(
            handoff.contexts().get(SourceStatementContextId::new(1)),
            None
        );
        assert_eq!(
            handoff
                .input_facts()
                .get(SourceStatementInputFactId::new(1)),
            None
        );
        assert_eq!(
            handoff
                .candidate_facts()
                .get(SourceStatementCandidateFactId::new(1)),
            None
        );
    }

    #[test]
    fn corruption_precedence_provenance_and_revalidation_fail_closed() {
        let fixture = Fixture::new(2);
        let valid = fixture.build(fixture.input()).expect("baseline");
        let valid_debug = valid.debug_text();
        let mut wrong_ordinal_primary = fixture.primary.clone();
        wrong_ordinal_primary
            .set_reference_use_ordinal_for_test(SourcePrimaryTermReferenceId::new(0), 2);
        let mut matching_atomic = fixture.atomic.clone();
        matching_atomic.set_primary_term_fingerprint_for_test(wrong_ordinal_primary.debug_text());
        let mut wrong_second_ordinal_primary = fixture.primary.clone();
        wrong_second_ordinal_primary
            .set_reference_use_ordinal_for_test(SourcePrimaryTermReferenceId::new(1), 2);
        let mut matching_second_atomic = fixture.atomic.clone();
        matching_second_atomic
            .set_primary_term_fingerprint_for_test(wrong_second_ordinal_primary.debug_text());

        let mut aggregate = fixture.input();
        aggregate.owners.push(aggregate.owners[0].clone());
        aggregate.statements[0].source_ordinal = 7;
        assert_eq!(
            fixture.build(aggregate),
            Err(SourceStatementError::InvalidAggregate)
        );

        let mut owner = fixture.input();
        owner.owners[0].spelling.push('x');
        assert_eq!(
            fixture.build(owner),
            Err(SourceStatementError::InvalidOwner {
                owner: SourceTheoremOwnerId::new(0)
            })
        );
        let mut statement = fixture.input();
        statement.statements[0].source_ordinal = 1;
        assert_eq!(
            fixture.build(statement),
            Err(SourceStatementError::InvalidStatement {
                statement: SourceStatementId::new(0)
            })
        );
        let mut context = fixture.input();
        context.contexts[0].visible_bindings.clear();
        assert_eq!(
            fixture.build(context),
            Err(SourceStatementError::InvalidContext {
                context: SourceStatementContextId::new(0)
            })
        );
        let mut input_fact = fixture.input();
        input_fact.input_facts[0].uses.swap(0, 1);
        assert_eq!(
            fixture.build(input_fact),
            Err(SourceStatementError::InvalidInputFact {
                fact: SourceStatementInputFactId::new(0)
            })
        );
        let mut candidate = fixture.input();
        candidate.candidate_facts[0].ordinal = 1;
        assert_eq!(
            fixture.build(candidate),
            Err(SourceStatementError::InvalidCandidateFact {
                fact: SourceStatementCandidateFactId::new(0)
            })
        );

        let mut dependency_and_aggregate_input = fixture.input();
        dependency_and_aggregate_input.owners.clear();
        assert_eq!(
            SourceStatementProducer::build(
                dependency_and_aggregate_input,
                &fixture.symbols,
                &fixture.bindings,
                &wrong_ordinal_primary,
                &matching_atomic,
                &fixture.arena,
            ),
            Err(SourceStatementError::DependencyMismatch)
        );
        let mut owner_and_later_rows = fixture.input();
        owner_and_later_rows.owners[0].source_range = range(fixture.source, 20, 80);
        owner_and_later_rows.statements[0].source_ordinal = 1;
        owner_and_later_rows.contexts[0].visible_bindings.clear();
        owner_and_later_rows.input_facts[0].ordinal = 1;
        owner_and_later_rows.candidate_facts[0].ordinal = 1;
        assert_eq!(
            fixture.build(owner_and_later_rows),
            Err(SourceStatementError::InvalidOwner {
                owner: SourceTheoremOwnerId::new(0)
            })
        );
        let mut statement_and_later_rows = fixture.input();
        statement_and_later_rows.statements[0].source_ordinal = 1;
        statement_and_later_rows.contexts[0]
            .visible_bindings
            .clear();
        statement_and_later_rows.input_facts[0].ordinal = 1;
        statement_and_later_rows.candidate_facts[0].ordinal = 1;
        assert_eq!(
            fixture.build(statement_and_later_rows),
            Err(SourceStatementError::InvalidStatement {
                statement: SourceStatementId::new(0)
            })
        );
        let mut context_and_later_rows = fixture.input();
        context_and_later_rows.contexts[0].visible_bindings.clear();
        context_and_later_rows.input_facts[0].ordinal = 1;
        context_and_later_rows.candidate_facts[0].ordinal = 1;
        assert_eq!(
            fixture.build(context_and_later_rows),
            Err(SourceStatementError::InvalidContext {
                context: SourceStatementContextId::new(0)
            })
        );
        let mut input_and_candidate_rows = fixture.input();
        input_and_candidate_rows.input_facts[0].ordinal = 1;
        input_and_candidate_rows.candidate_facts[0].ordinal = 1;
        assert_eq!(
            fixture.build(input_and_candidate_rows),
            Err(SourceStatementError::InvalidInputFact {
                fact: SourceStatementInputFactId::new(0)
            })
        );

        let mut dependency_and_aggregate = valid.clone();
        dependency_and_aggregate.binding_fingerprint.push('x');
        dependency_and_aggregate.owners.rows.clear();
        assert_eq!(
            dependency_and_aggregate.validate_installation(
                fixture.source,
                &fixture.module,
                &fixture.primary,
                &fixture.atomic,
                &fixture.arena,
            ),
            Err(SourceStatementError::DependencyMismatch)
        );
        let mut aggregate_and_row = valid.clone();
        aggregate_and_row.contexts.rows.clear();
        aggregate_and_row.statements.rows[0].source_ordinal = 1;
        assert_eq!(
            aggregate_and_row.validate_installation(
                fixture.source,
                &fixture.module,
                &fixture.primary,
                &fixture.atomic,
                &fixture.arena,
            ),
            Err(SourceStatementError::InvalidAggregate)
        );
        let mut stale_primary_fingerprint = valid.clone();
        stale_primary_fingerprint
            .primary_term_fingerprint
            .push_str(":stale");
        assert_eq!(
            stale_primary_fingerprint.validate_installation(
                fixture.source,
                &fixture.module,
                &fixture.primary,
                &fixture.atomic,
                &fixture.arena,
            ),
            Err(SourceStatementError::DependencyMismatch)
        );
        let mut stale_atomic_fingerprint = valid.clone();
        stale_atomic_fingerprint
            .atomic_formula_fingerprint
            .push_str(":stale");
        assert_eq!(
            stale_atomic_fingerprint.validate_installation(
                fixture.source,
                &fixture.module,
                &fixture.primary,
                &fixture.atomic,
                &fixture.arena,
            ),
            Err(SourceStatementError::DependencyMismatch)
        );
        let mut coherent_wrong_ordinal = valid.clone();
        coherent_wrong_ordinal.primary_term_fingerprint = wrong_ordinal_primary.debug_text();
        coherent_wrong_ordinal.atomic_formula_fingerprint = matching_atomic.debug_text();
        assert_eq!(
            coherent_wrong_ordinal.validate_installation(
                fixture.source,
                &fixture.module,
                &wrong_ordinal_primary,
                &matching_atomic,
                &fixture.arena,
            ),
            Err(SourceStatementError::DependencyMismatch)
        );
        let mut coherent_wrong_second_ordinal = valid.clone();
        coherent_wrong_second_ordinal.primary_term_fingerprint =
            wrong_second_ordinal_primary.debug_text();
        coherent_wrong_second_ordinal.atomic_formula_fingerprint =
            matching_second_atomic.debug_text();
        assert_eq!(
            coherent_wrong_second_ordinal.validate_installation(
                fixture.source,
                &fixture.module,
                &wrong_second_ordinal_primary,
                &matching_second_atomic,
                &fixture.arena,
            ),
            Err(SourceStatementError::DependencyMismatch)
        );
        let substituted_binding =
            binding_env_with_visible_after(fixture.source, &fixture.module, 1);
        let mut coherent_wrong_binding = valid.clone();
        coherent_wrong_binding.binding_env = substituted_binding.clone();
        coherent_wrong_binding.binding_fingerprint = substituted_binding.debug_text();
        assert_eq!(
            coherent_wrong_binding.validate_installation(
                fixture.source,
                &fixture.module,
                &fixture.primary,
                &fixture.atomic,
                &fixture.arena,
            ),
            Err(SourceStatementError::DependencyMismatch)
        );

        let mut stale_checked = valid.clone();
        stale_checked.checked_owner = CheckedStatementOwner::from_validated_parts_for_test(
            fixture.symbol.clone(),
            range(fixture.source, 20, 80),
            SemanticOrigin::new(
                fixture.source,
                fixture.module.clone(),
                SourceAnchor::Range(range(fixture.source, 20, 80)),
                vec![1],
            ),
        );
        assert_eq!(
            stale_checked.validate_installation(
                fixture.source,
                &fixture.module,
                &fixture.primary,
                &fixture.atomic,
                &fixture.arena,
            ),
            Err(SourceStatementError::InvalidOwner {
                owner: SourceTheoremOwnerId::new(0)
            })
        );
        let mut foreign_contributions = SourceContributionIndex::new();
        foreign_contributions.insert(
            fixture.module.clone(),
            ContributionKind::LocalSource {
                source_id: fixture.source,
            },
            SourceAnchor::Range(range(fixture.source, 0, 1)),
        );
        let foreign_contribution = foreign_contributions.insert(
            fixture.module.clone(),
            ContributionKind::LocalSource {
                source_id: fixture.source,
            },
            SourceAnchor::Range(range(fixture.source, 1, 2)),
        );
        let mut stale_contribution = valid.clone();
        stale_contribution.owners.rows[0].contribution = foreign_contribution;
        assert_eq!(
            stale_contribution.validate_installation(
                fixture.source,
                &fixture.module,
                &fixture.primary,
                &fixture.atomic,
                &fixture.arena,
            ),
            Err(SourceStatementError::InvalidOwner {
                owner: SourceTheoremOwnerId::new(0)
            })
        );

        for mutation in [
            ResolverMutation::MissingLabelEffect,
            ResolverMutation::RecoveredLabel,
            ResolverMutation::WrongContribution,
            ResolverMutation::DuplicateLabel,
            ResolverMutation::StaleLabel,
            ResolverMutation::WrongLabelKind,
            ResolverMutation::PrivateLabel,
            ResolverMutation::LocalOnlyLabel,
            ResolverMutation::WrongLabelSource,
            ResolverMutation::WrongLabelModule,
            ResolverMutation::WrongLabelNamespace,
            ResolverMutation::WrongResolverNamespace,
            ResolverMutation::WrongContributionAnchor,
        ] {
            let (symbol, contribution, symbols) =
                symbol_env_with_mutation(fixture.source, &fixture.module, mutation);
            let mut input = fixture.input();
            input.owners[0].symbol = symbol;
            input.owners[0].contribution = contribution;
            assert_eq!(
                SourceStatementProducer::build(
                    input,
                    &symbols,
                    &fixture.bindings,
                    &fixture.primary,
                    &fixture.atomic,
                    &fixture.arena,
                ),
                Err(SourceStatementError::InvalidOwner {
                    owner: SourceTheoremOwnerId::new(0)
                })
            );
        }

        let replay = fixture.build(fixture.input()).expect("valid replay");
        assert_eq!(replay.debug_text(), valid_debug);
        assert_eq!(replay, valid);
    }

    #[test]
    fn lower_and_arena_identity_are_revalidated_without_publication() {
        let fixture = Fixture::new(3);
        let handoff = fixture.build(fixture.input()).expect("baseline");
        let other_module = ModuleId::new(PackageId::new("pkg"), ModulePath::new("statement.other"));
        handoff
            .validate_installation(
                fixture.source,
                &fixture.module,
                &fixture.primary,
                &fixture.atomic,
                &fixture.arena,
            )
            .expect("exact installation");

        let substituted = Fixture::new(4);
        for result in [
            handoff.validate_installation(
                substituted.source,
                &fixture.module,
                &fixture.primary,
                &fixture.atomic,
                &fixture.arena,
            ),
            handoff.validate_installation(
                fixture.source,
                &other_module,
                &fixture.primary,
                &fixture.atomic,
                &fixture.arena,
            ),
            handoff.validate_installation(
                fixture.source,
                &fixture.module,
                &substituted.primary,
                &fixture.atomic,
                &fixture.arena,
            ),
            handoff.validate_installation(
                fixture.source,
                &fixture.module,
                &fixture.primary,
                &substituted.atomic,
                &fixture.arena,
            ),
            handoff.validate_installation(
                fixture.source,
                &fixture.module,
                &fixture.primary,
                &fixture.atomic,
                &substituted.arena,
            ),
        ] {
            assert_eq!(result, Err(SourceStatementError::DependencyMismatch));
        }

        let mut wrong_formula_children = typed_arena(fixture.source);
        let replacement = TypedArena::try_new(
            wrong_formula_children.root(),
            wrong_formula_children
                .iter()
                .map(|(id, row)| {
                    let mut row = row.clone();
                    if id == TypedNodeId::new(3) {
                        row.children.reverse();
                    }
                    row
                })
                .collect(),
        )
        .expect("valid but substituted arena");
        wrong_formula_children = replacement;
        assert_eq!(
            handoff.validate_installation(
                fixture.source,
                &fixture.module,
                &fixture.primary,
                &fixture.atomic,
                &wrong_formula_children,
            ),
            Err(SourceStatementError::DependencyMismatch)
        );
        let direct_formula = mutate_arena(&fixture.arena, |id, row| {
            if id == TypedNodeId::new(5) {
                row.children = vec![TypedNodeId::new(3)];
            }
        });
        assert_eq!(
            handoff.validate_installation(
                fixture.source,
                &fixture.module,
                &fixture.primary,
                &fixture.atomic,
                &direct_formula,
            ),
            Err(SourceStatementError::InvalidStatement {
                statement: SourceStatementId::new(0)
            })
        );
        let escaped_formula = mutate_arena(&fixture.arena, |id, row| {
            if id == TypedNodeId::new(4) {
                row.children.clear();
            }
        });
        assert_eq!(
            handoff.validate_installation(
                fixture.source,
                &fixture.module,
                &fixture.primary,
                &fixture.atomic,
                &escaped_formula,
            ),
            Err(SourceStatementError::InvalidStatement {
                statement: SourceStatementId::new(0)
            })
        );
        let escaped_term = mutate_arena(&fixture.arena, |id, row| {
            if id == TypedNodeId::new(3) {
                row.children.pop();
            }
        });
        assert_eq!(
            handoff.validate_installation(
                fixture.source,
                &fixture.module,
                &fixture.primary,
                &fixture.atomic,
                &escaped_term,
            ),
            Err(SourceStatementError::DependencyMismatch)
        );
        let excluded_wrapper = mutate_arena(&fixture.arena, |id, row| {
            if id == TypedNodeId::new(4) {
                row.kind = "source.statement.assumption".into();
            }
        });
        assert_eq!(
            handoff.validate_installation(
                fixture.source,
                &fixture.module,
                &fixture.primary,
                &fixture.atomic,
                &excluded_wrapper,
            ),
            Err(SourceStatementError::InvalidStatement {
                statement: SourceStatementId::new(0)
            })
        );
        let excluded_formula_descendant = mutate_arena(&fixture.arena, |id, row| {
            if id == TypedNodeId::new(2) {
                row.kind = "source.statement.assumption".into();
            }
        });
        assert_eq!(
            handoff.validate_installation(
                fixture.source,
                &fixture.module,
                &fixture.primary,
                &fixture.atomic,
                &excluded_formula_descendant,
            ),
            Err(SourceStatementError::InvalidStatement {
                statement: SourceStatementId::new(0)
            })
        );
        assert_eq!(handoff.binding_env(), &fixture.bindings);
        assert_eq!(handoff.clone().debug_text(), handoff.debug_text());

        let task248 = crate::source_context::tests::task_248_occupied_typed_ast(
            fixture.source,
            fixture.module.clone(),
        );
        let task248_debug = task248.debug_text();
        assert!(matches!(
            task248.clone().with_source_statement(handoff.clone()),
            Err(TypedAstError::InvalidSourceStatement)
        ));
        assert_eq!(task248.debug_text(), task248_debug);

        let base = TypedAst::try_new(TypedAstParts {
            source_id: fixture.source,
            module_id: fixture.module.clone(),
            resolved_root: None,
            source_context: None,
            source_type: None,
            source_attribute: None,
            nodes: fixture.arena.clone(),
            contexts: LocalTypeContextTable::new(),
            types: TypeTable::new(),
            facts: TypeFactTable::new(),
            coercions: CoercionTable::new(),
            initial_obligations: InitialObligationTable::new(),
            diagnostics: TypeDiagnosticTable::new(),
        })
        .expect("empty exact typed AST")
        .with_source_term(fixture.primary.clone())
        .expect("primary installation")
        .with_source_atomic_formula(fixture.atomic.clone())
        .expect("atomic installation");
        let mut semantic_facts = TypeFactTable::new();
        semantic_facts.insert(TypeFactDraft {
            subject: node(0),
            predicate: TypePredicateRef::new("set"),
            polarity: Polarity::Positive,
            provenance: FactProvenance::Inferred(TypeRuleId::new("statement-coexistence")),
            status: FactStatus::Known,
        });
        let semantic_base = TypedAst::try_new(TypedAstParts {
            source_id: fixture.source,
            module_id: fixture.module.clone(),
            resolved_root: None,
            source_context: None,
            source_type: None,
            source_attribute: None,
            nodes: fixture.arena.clone(),
            contexts: LocalTypeContextTable::new(),
            types: TypeTable::new(),
            facts: semantic_facts,
            coercions: CoercionTable::new(),
            initial_obligations: InitialObligationTable::new(),
            diagnostics: TypeDiagnosticTable::new(),
        })
        .expect("semantically occupied typed AST")
        .with_source_term(fixture.primary.clone())
        .expect("primary installation into semantic fixture")
        .with_source_atomic_formula(fixture.atomic.clone())
        .expect("atomic installation into semantic fixture");
        let semantic_debug = semantic_base.debug_text();
        assert!(matches!(
            semantic_base.clone().with_source_statement(handoff.clone()),
            Err(TypedAstError::InvalidSourceStatement)
        ));
        assert_eq!(semantic_base.debug_text(), semantic_debug);
        let typed = base
            .clone()
            .with_source_statement(handoff.clone())
            .expect("statement installation");
        let typed_debug = typed.debug_text();
        let source_context = task248
            .source_context()
            .expect("Task248 source context")
            .clone();
        assert!(matches!(
            typed
                .clone()
                .with_source_context_for_test(source_context.clone()),
            Err(TypedAstError::InvalidSourceContext)
        ));
        assert_eq!(typed.debug_text(), typed_debug);
        assert_eq!(
            base.with_source_statement(handoff.clone())
                .expect("valid replay")
                .debug_text(),
            typed_debug
        );

        let mut injected = task248.clone();
        injected.inject_source_statement_for_test(handoff.clone());
        let injected_debug = injected.debug_text();
        assert!(injected.source_context().is_some());
        assert!(injected.source_statement().is_some());
        assert_eq!(
            assemble_empty_resolved(&injected),
            Err(ResolvedTypedAstError::InvalidSourceStatement)
        );
        assert_eq!(injected.debug_text(), injected_debug);
        assemble_empty_resolved(&typed).expect("valid final replay");
        let mut cluster_facts = ClusterFactTable::new();
        cluster_facts.insert(ClusterFactDraft {
            fingerprint: ClusterFactFingerprint::new("statement-coexistence"),
            source_type: ClusterTypeFingerprint::new("set"),
            attribute: ClusterAttributeFingerprint::new("inhabited"),
            generated_type: ClusterTypeFingerprint::new("set"),
            provenance: ClusterFactProvenance::Input,
            source_range: range(fixture.source, 74, 75),
        });
        assert_eq!(
            assemble_resolved(&typed, &cluster_facts, Vec::new(), None,),
            Err(ResolvedTypedAstError::InvalidSourceStatement)
        );
        assert_eq!(
            assemble_resolved(
                &typed,
                &ClusterFactTable::new(),
                vec![ResolvedNodeKindHint {
                    typed_node: TypedNodeId::new(0),
                    kind: ResolvedNodeKindHintKind::SourcePreserved {
                        role: SourceNodeRole::new("source.statement.semantic-coexistence"),
                    },
                }],
                None,
            ),
            Err(ResolvedTypedAstError::InvalidSourceStatement)
        );
        assert_eq!(
            assemble_resolved(
                &typed,
                &ClusterFactTable::new(),
                Vec::new(),
                Some(StatementProofInputs {
                    owner: handoff.checked_owner(),
                    rows: Vec::new(),
                }),
            ),
            Err(ResolvedTypedAstError::InvalidSourceStatement)
        );
        assemble_resolved(
            &typed,
            &ClusterFactTable::new(),
            typed
                .nodes()
                .iter()
                .map(|(typed_node, _)| ResolvedNodeKindHint {
                    typed_node,
                    kind: ResolvedNodeKindHintKind::SourcePreserved {
                        role: SourceNodeRole::new("source.statement.transport"),
                    },
                })
                .collect(),
            None,
        )
        .expect("valid syntax-preserved final replay");
        assert_eq!(task248.debug_text(), task248_debug);
    }

    #[test]
    fn task258b3_exact_witness_api_debug_and_lower_profile_are_stable() {
        let fixture = B3Fixture::new(40);
        let statement = fixture.statement();
        assert!(statement.is_task_258b3_profile());
        assert!(!statement.is_task_258a_profile());
        assert!(!statement.is_task_258b1_profile());
        assert!(!statement.is_task_258b2_profile());
        assert_eq!(statement.binding_env(), &fixture.bindings);
        assert_eq!(statement.binding_env().contexts().len(), 2);
        assert_eq!(statement.binding_env().bindings().len(), 1);
        assert!(statement.binding_env().diagnostics().is_empty());
        let proof_context = statement
            .binding_env()
            .contexts()
            .get(BindingContextId::new(1))
            .expect("proof context");
        assert_eq!(
            proof_context.owner,
            BindingContextOwner::SourceStatement {
                source_range: range(fixture.source, 69, 102),
            }
        );
        assert_eq!(proof_context.parent, Some(BindingContextId::new(0)));
        assert_eq!(proof_context.layer, BindingContextLayer::Proof);
        assert_eq!(
            proof_context.lexical_scope.as_ref().expect("scope").path(),
            [0]
        );
        assert!(proof_context.bindings.is_empty());
        assert_eq!(proof_context.visible_bindings, [BindingId::new(0)]);
        assert_eq!(proof_context.recovery, BindingContextRecovery::Normal);
        assert_eq!(
            statement
                .statements()
                .iter()
                .map(|(_, row)| row.source_ordinal())
                .collect::<Vec<_>>(),
            [0, 2]
        );
        assert_eq!(fixture.primary.terms().len(), 5);
        assert_eq!(fixture.primary.references().len(), 5);
        assert!(fixture.primary.numeric_type_requests().is_empty());
        assert_eq!(
            fixture
                .primary
                .terms()
                .iter()
                .map(|(_, row)| row.context().index())
                .collect::<Vec<_>>(),
            [0, 0, 1, 1, 1]
        );
        for (index, ((_, term), (_, reference))) in fixture
            .primary
            .terms()
            .iter()
            .zip(fixture.primary.references().iter())
            .enumerate()
        {
            let sites = [26, 28, 32, 36, 38];
            let ranges = [(63, 64), (67, 68), (82, 83), (92, 93), (96, 97)];
            let contexts = [0, 0, 1, 1, 1];
            assert_eq!(term.site().node().index(), sites[index]);
            assert_eq!(
                term.source_range(),
                range(fixture.source, ranges[index].0, ranges[index].1)
            );
            assert_eq!(term.source_ordinal(), index);
            assert_eq!(term.context(), BindingContextId::new(contexts[index]));
            assert_eq!(term.spelling(), "x");
            assert_eq!(term.kind(), SourcePrimaryTermKind::VariableReference);
            assert_eq!(term.role(), SourcePrimaryTermRole::Value);
            assert_eq!(term.recovery(), SourcePrimaryTermRecovery::Normal);
            assert!(term.parent().is_none());
            assert_eq!(reference.term(), SourcePrimaryTermId::new(index));
            assert_eq!(reference.binding(), BindingId::new(0));
            assert_eq!(reference.role(), SourcePrimaryTermReferenceRole::Variable);
            if index < 2 {
                assert!(reference.lexical_scope().is_none());
            } else {
                assert_eq!(
                    reference.lexical_scope().expect("reference scope").path(),
                    [0]
                );
            }
            assert_eq!(reference.use_ordinal(), 1);
        }
        assert_eq!(fixture.atomic.formulas().len(), 2);
        assert!(fixture.atomic.wrappers().is_empty());
        assert!(fixture.atomic.predicate_segments().is_empty());
        assert!(fixture.atomic.predicate_heads().is_empty());
        assert!(fixture.atomic.candidates().is_empty());
        assert!(fixture.atomic.type_sites().is_empty());
        assert!(fixture.atomic.attributes().is_empty());
        assert_eq!(fixture.atomic.edges().len(), 4);
        assert_eq!(fixture.atomic.requests().len(), 4);
        for index in 0..2 {
            let formula = fixture
                .atomic
                .formulas()
                .get(SourceAtomicFormulaId::new(index))
                .expect("atomic formula");
            assert_eq!(formula.site().node().index(), [30, 40][index]);
            assert_eq!(
                formula.source_range(),
                range(
                    fixture.source,
                    [(63, 68), (92, 97)][index].0,
                    [(63, 68), (92, 97)][index].1
                )
            );
            assert_eq!(formula.source_ordinal(), index);
            assert_eq!(formula.context(), BindingContextId::new(index));
            assert_eq!(formula.spelling(), "x = x");
            assert_eq!(formula.kind(), SourceAtomicFormulaKind::Equality);
            assert_eq!(formula.recovery(), SourceAtomicFormulaRecovery::Normal);
        }
        assert_eq!(
            fixture
                .atomic
                .edges()
                .iter()
                .map(|(_, row)| row.target())
                .collect::<Vec<_>>(),
            [
                SourceAtomicTermTarget::Primary(SourcePrimaryTermId::new(0)),
                SourceAtomicTermTarget::Primary(SourcePrimaryTermId::new(1)),
                SourceAtomicTermTarget::Primary(SourcePrimaryTermId::new(3)),
                SourceAtomicTermTarget::Primary(SourcePrimaryTermId::new(4)),
            ]
        );
        for edge_index in 0..4 {
            let edge = fixture
                .atomic
                .edges()
                .get(SourceAtomicEdgeId::new(edge_index))
                .expect("atomic edge");
            let formula = if edge_index < 2 { 0 } else { 1 };
            let ordinal = edge_index % 2;
            assert_eq!(edge.formula(), SourceAtomicFormulaId::new(formula));
            assert_eq!(edge.ordinal(), ordinal);
            assert_eq!(
                edge.role(),
                if ordinal == 0 {
                    SourceAtomicEdgeRole::BuiltinLeftOperand
                } else {
                    SourceAtomicEdgeRole::BuiltinRightOperand
                }
            );
            let request = fixture
                .atomic
                .requests()
                .get(crate::source_atomic_formula::SourceAtomicRequestId::new(
                    edge_index,
                ))
                .expect("atomic request");
            assert_eq!(request.formula(), SourceAtomicFormulaId::new(formula));
            assert_eq!(request.ordinal(), ordinal);
            assert_eq!(request.kind(), SourceAtomicRequestKind::OperandExpectedType);
            assert_eq!(request.edge(), Some(SourceAtomicEdgeId::new(edge_index)));
            assert!(request.candidate().is_none());
            assert!(request.type_site().is_none());
            assert!(request.attribute().is_none());
        }
        assert_eq!(statement.owners().len(), 1);
        assert_eq!(statement.contexts().len(), 2);
        assert_eq!(statement.input_facts().len(), 2);
        assert_eq!(statement.candidate_facts().len(), 2);
        let owner = statement
            .owners()
            .get(SourceTheoremOwnerId::new(0))
            .expect("owner");
        assert_eq!(owner.symbol(), &fixture.symbol);
        assert_eq!(owner.contribution(), fixture.contribution);
        assert_eq!(owner.site().node().index(), 45);
        assert_eq!(owner.source_range(), range(fixture.source, 19, 103));
        assert_eq!(owner.spelling(), "FormulaStatementSingleWitnessSmoke");
        assert_eq!(owner.role(), SourceTheoremRole::Theorem);
        assert_eq!(owner.status(), SourceTheoremStatus::Unmodified);
        assert_eq!(owner.recovery(), SourceStatementRecovery::Normal);
        assert_eq!(statement.checked_owner().origin().structural_path(), [2, 1]);
        assert_eq!(
            statement.binding_fingerprint(),
            fixture.bindings.debug_text()
        );
        assert_eq!(
            statement.primary_term_fingerprint(),
            fixture.primary.debug_text()
        );
        assert_eq!(
            statement.atomic_formula_fingerprint(),
            fixture.atomic.debug_text()
        );
        for index in 0..2 {
            let row = statement
                .statements()
                .get(SourceStatementId::new(index))
                .expect("statement");
            assert_eq!(row.owner(), SourceTheoremOwnerId::new(0));
            assert_eq!(row.context(), SourceStatementContextId::new(index));
            assert_eq!(
                row.formula(),
                SourceStatementFormulaTarget::Atomic(SourceAtomicFormulaId::new(index))
            );
            assert_eq!(row.site().node().index(), [45, 43][index]);
            assert_eq!(
                row.source_range(),
                range(
                    fixture.source,
                    [(19, 103), (87, 98)][index].0,
                    [(19, 103), (87, 98)][index].1
                )
            );
            assert_eq!(row.source_ordinal(), [0, 2][index]);
            assert_eq!(
                row.kind(),
                [
                    SourceStatementKind::TheoremProposition,
                    SourceStatementKind::Conclusion,
                ][index]
            );
            assert_eq!(row.recovery(), SourceStatementRecovery::Normal);
            let context = statement
                .contexts()
                .get(SourceStatementContextId::new(index))
                .expect("statement context");
            assert_eq!(context.statement(), SourceStatementId::new(index));
            assert_eq!(context.binding_context(), BindingContextId::new(index));
            assert_eq!(context.source_range(), row.source_range());
            assert_eq!(context.visible_bindings(), [BindingId::new(0)]);
            let fact = statement
                .input_facts()
                .get(SourceStatementInputFactId::new(index))
                .expect("input fact");
            assert_eq!(fact.statement(), SourceStatementId::new(index));
            assert_eq!(fact.context(), SourceStatementContextId::new(index));
            assert_eq!(fact.ordinal(), 0);
            assert_eq!(fact.kind(), SourceStatementInputFactKind::ReservedTypeGuard);
            assert_eq!(fact.binding(), BindingId::new(0));
            let first_term = if index == 0 { 0 } else { 3 };
            assert_eq!(
                fact.uses(),
                [
                    SourcePrimaryTermReferenceId::new(first_term),
                    SourcePrimaryTermReferenceId::new(first_term + 1),
                ]
            );
            let candidate = statement
                .candidate_facts()
                .get(SourceStatementCandidateFactId::new(index))
                .expect("candidate");
            assert_eq!(candidate.statement(), SourceStatementId::new(index));
            assert_eq!(candidate.context(), SourceStatementContextId::new(index));
            assert_eq!(candidate.ordinal(), 0);
            assert_eq!(
                candidate.kind(),
                SourceStatementCandidateFactKind::UnverifiedProposition
            );
            assert_eq!(candidate.formula(), row.formula());
        }
        let witnesses = fixture
            .witnesses(&statement, fixture.witness_input())
            .expect("Task258B3 witnesses");
        assert_eq!(witnesses.source_id(), fixture.source);
        assert_eq!(witnesses.module_id(), &fixture.module);
        assert_eq!(witnesses.statement_fingerprint(), statement.debug_text());
        assert_eq!(
            witnesses.primary_term_fingerprint(),
            fixture.primary.debug_text()
        );
        assert_eq!(witnesses.witnesses().len(), 1);
        assert!(!witnesses.witnesses().is_empty());
        let row = witnesses
            .witnesses()
            .get(SourceStatementWitnessId::new(0))
            .expect("witness row");
        assert_eq!(row.owner(), SourceTheoremOwnerId::new(0));
        assert_eq!(row.binding_context(), BindingContextId::new(1));
        assert_eq!(
            row.term(),
            SourceStatementWitnessTermTarget::Primary(SourcePrimaryTermId::new(2))
        );
        assert_eq!(
            (row.take_site().node().index(), row.site().node().index()),
            (35, 34)
        );
        assert_eq!((row.take_range().start, row.take_range().end), (77, 84));
        assert_eq!((row.source_range().start, row.source_range().end), (82, 83));
        assert_eq!((row.source_ordinal(), row.ordinal()), (1, 0));
        assert_eq!(row.spelling(), "x");
        assert_eq!(row.kind(), SourceStatementWitnessKind::Unnamed);
        assert_eq!(row.recovery(), SourceStatementRecovery::Normal);
        let debug = witnesses.debug_text();
        assert!(debug.starts_with("source-statement-witness-debug-v1\n"));
        assert!(debug.contains(
            "witness#0 owner=0 binding_context=1 term=primary#2 take_range=77..84 take_site=35 range=82..83 site=34 source_ordinal=1 ordinal=0 kind=unnamed recovery=normal spelling=\"x\""
        ));
        assert_eq!(
            fixture
                .witnesses(&statement, fixture.witness_input())
                .expect("replay")
                .debug_text(),
            debug
        );
    }

    #[test]
    fn task258b3_dependency_aggregate_rows_provenance_and_replay_fail_closed() {
        let fixture = B3Fixture::new(41);
        let statement = fixture.statement();
        let valid = fixture
            .witnesses(&statement, fixture.witness_input())
            .expect("baseline");
        let statement_baseline = statement.debug_text();
        let baseline = valid.debug_text();
        let assert_replay = || {
            let replay_statement = fixture.statement();
            assert_eq!(replay_statement.debug_text(), statement_baseline);
            assert_eq!(
                fixture
                    .witnesses(&replay_statement, fixture.witness_input())
                    .expect("witness replay")
                    .debug_text(),
                baseline
            );
        };
        let mut empty = fixture.witness_input();
        empty.witnesses.clear();
        assert_eq!(
            fixture.witnesses(&statement, empty),
            Err(SourceStatementWitnessError::InvalidAggregate)
        );
        assert_replay();
        let mut duplicate = fixture.witness_input();
        duplicate.witnesses.push(duplicate.witnesses[0].clone());
        assert_eq!(
            fixture.witnesses(&statement, duplicate),
            Err(SourceStatementWitnessError::InvalidAggregate)
        );
        assert_replay();
        let mut wrong_source = fixture.witness_input();
        wrong_source.source_id = source_id(999);
        assert_eq!(
            fixture.witnesses(&statement, wrong_source),
            Err(SourceStatementWitnessError::DependencyMismatch)
        );
        assert_replay();
        let mut wrong_module = fixture.witness_input();
        wrong_module.module_id =
            ModuleId::new(PackageId::new("pkg"), ModulePath::new("statement.foreign"));
        assert_eq!(
            fixture.witnesses(&statement, wrong_module),
            Err(SourceStatementWitnessError::DependencyMismatch)
        );
        assert_replay();
        for (label, mutate) in [
            (
                "owner",
                (|row: &mut SourceStatementWitnessInput| row.owner = SourceTheoremOwnerId::new(1))
                    as fn(&mut SourceStatementWitnessInput),
            ),
            ("context", |row| {
                row.binding_context = BindingContextId::new(0)
            }),
            ("term 0", |row| {
                row.term = SourceStatementWitnessTermTarget::Primary(SourcePrimaryTermId::new(0))
            }),
            ("term 1", |row| {
                row.term = SourceStatementWitnessTermTarget::Primary(SourcePrimaryTermId::new(1))
            }),
            ("term 3", |row| {
                row.term = SourceStatementWitnessTermTarget::Primary(SourcePrimaryTermId::new(3))
            }),
            ("term 4", |row| {
                row.term = SourceStatementWitnessTermTarget::Primary(SourcePrimaryTermId::new(4))
            }),
            ("take site", |row| row.take_site = node(34)),
            ("take range start", |row| row.take_range.start = 78),
            ("take range end", |row| row.take_range.end = 83),
            ("site", |row| row.site = node(35)),
            ("range start", |row| row.source_range.start = 81),
            ("range end", |row| row.source_range.end = 84),
            ("source ordinal", |row| row.source_ordinal = 2),
            ("ordinal", |row| row.ordinal = 1),
            ("spelling", |row| row.spelling = "y".to_owned()),
            ("recovery", |row| {
                row.recovery = SourceStatementRecovery::Degraded
            }),
        ] {
            let mut input = fixture.witness_input();
            mutate(&mut input.witnesses[0]);
            assert_eq!(
                fixture.witnesses(&statement, input),
                Err(SourceStatementWitnessError::InvalidWitness {
                    witness: SourceStatementWitnessId::new(0)
                }),
                "{label}"
            );
            assert_replay();
        }
        let assert_base_rejects =
            |label: &str, mutate: &dyn Fn(&mut SourceStatementHandoffInput)| {
                let mut input = fixture.statement_input();
                mutate(&mut input);
                assert!(
                    SourceStatementProducer::build(
                        input,
                        &fixture.symbols,
                        &fixture.bindings,
                        &fixture.primary,
                        &fixture.atomic,
                        &fixture.arena,
                    )
                    .is_err(),
                    "{label}"
                );
                assert_replay();
            };
        let mut foreign_contributions = SourceContributionIndex::new();
        foreign_contributions.insert(
            fixture.module.clone(),
            ContributionKind::LocalSource {
                source_id: fixture.source,
            },
            SourceAnchor::Range(range(fixture.source, 0, 1)),
        );
        let foreign_contribution = foreign_contributions.insert(
            fixture.module.clone(),
            ContributionKind::LocalSource {
                source_id: fixture.source,
            },
            SourceAnchor::Range(range(fixture.source, 1, 2)),
        );
        assert_eq!(foreign_contribution.index(), 1);
        assert_base_rejects("base source", &|input| input.source_id = source_id(999));
        assert_base_rejects("base module", &|input| {
            input.module_id =
                ModuleId::new(PackageId::new("pkg"), ModulePath::new("statement.foreign"))
        });
        assert_base_rejects("base owner contribution", &|input| {
            input.owners[0].contribution = foreign_contribution
        });
        assert_base_rejects("extra base owner", &|input| {
            input.owners.push(input.owners[0].clone())
        });
        for (label, mutate) in [
            (
                "missing base statement",
                (|input: &mut SourceStatementHandoffInput| {
                    input.statements.pop();
                }) as fn(&mut SourceStatementHandoffInput),
            ),
            ("extra base statement", |input| {
                input.statements.push(input.statements[1].clone())
            }),
            ("missing base context", |input| {
                input.contexts.pop();
            }),
            ("extra base context", |input| {
                input.contexts.push(input.contexts[1].clone())
            }),
            ("missing base input fact", |input| {
                input.input_facts.pop();
            }),
            ("extra base input fact", |input| {
                input.input_facts.push(input.input_facts[1].clone())
            }),
            ("missing base candidate", |input| {
                input.candidate_facts.pop();
            }),
            ("extra base candidate", |input| {
                input.candidate_facts.push(input.candidate_facts[1].clone())
            }),
        ] {
            assert_base_rejects(label, &mutate);
        }
        for (label, mutate) in [
            (
                "base owner symbol",
                (|input: &mut SourceStatementHandoffInput| {
                    input.owners[0].symbol = SymbolId::new(
                        input.module_id.clone(),
                        LocalSymbolId::new("ForeignTask258B3Owner"),
                        FullyQualifiedName::new(
                            "pkg::statement.fixture::theorem::ForeignTask258B3Owner",
                        ),
                    )
                }) as fn(&mut SourceStatementHandoffInput),
            ),
            ("base owner site", |input| input.owners[0].site = node(43)),
            ("base owner range start", |input| {
                input.owners[0].source_range.start += 1
            }),
            ("base owner range end", |input| {
                input.owners[0].source_range.end -= 1
            }),
            ("base owner spelling", |input| {
                input.owners[0].spelling.push('x')
            }),
            ("base owner recovery", |input| {
                input.owners[0].recovery = SourceStatementRecovery::Degraded
            }),
        ] {
            assert_base_rejects(label, &mutate);
        }
        for index in 0..2 {
            let other = 1 - index;
            assert_base_rejects(&format!("base statement {index} owner"), &|input| {
                input.statements[index].owner = SourceTheoremOwnerId::new(1)
            });
            assert_base_rejects(&format!("base statement {index} context"), &|input| {
                input.statements[index].context = SourceStatementContextId::new(other)
            });
            assert_base_rejects(&format!("base statement {index} formula"), &|input| {
                input.statements[index].formula =
                    SourceStatementFormulaTarget::Atomic(SourceAtomicFormulaId::new(other))
            });
            assert_base_rejects(&format!("base statement {index} site"), &|input| {
                input.statements[index].site = node([43, 45][index])
            });
            assert_base_rejects(&format!("base statement {index} range start"), &|input| {
                input.statements[index].source_range.start += 1
            });
            assert_base_rejects(&format!("base statement {index} range end"), &|input| {
                input.statements[index].source_range.end -= 1
            });
            assert_base_rejects(&format!("base statement {index} spelling"), &|input| {
                input.statements[index].spelling.push('x')
            });
            assert_base_rejects(&format!("base statement {index} kind"), &|input| {
                input.statements[index].kind = [
                    SourceStatementKind::Conclusion,
                    SourceStatementKind::TheoremProposition,
                ][index]
            });
            assert_base_rejects(&format!("base statement {index} recovery"), &|input| {
                input.statements[index].recovery = SourceStatementRecovery::Degraded
            });
            assert_base_rejects(&format!("base context {index} statement"), &|input| {
                input.contexts[index].statement = SourceStatementId::new(other)
            });
            assert_base_rejects(&format!("base context {index} binding context"), &|input| {
                input.contexts[index].binding_context = BindingContextId::new(other)
            });
            assert_base_rejects(&format!("base context {index} range start"), &|input| {
                input.contexts[index].source_range.start += 1
            });
            assert_base_rejects(&format!("base context {index} range end"), &|input| {
                input.contexts[index].source_range.end -= 1
            });
            assert_base_rejects(&format!("base context {index} visibility"), &|input| {
                input.contexts[index].visible_bindings.clear()
            });
            assert_base_rejects(&format!("base input fact {index} statement"), &|input| {
                input.input_facts[index].statement = SourceStatementId::new(other)
            });
            assert_base_rejects(&format!("base input fact {index} context"), &|input| {
                input.input_facts[index].context = SourceStatementContextId::new(other)
            });
            assert_base_rejects(&format!("base input fact {index} ordinal"), &|input| {
                input.input_facts[index].ordinal = 1
            });
            assert_base_rejects(&format!("base input fact {index} binding"), &|input| {
                input.input_facts[index].binding = BindingId::new(1)
            });
            assert_base_rejects(&format!("base input fact {index} uses"), &|input| {
                input.input_facts[index].uses.swap(0, 1)
            });
            assert_base_rejects(&format!("base candidate {index} statement"), &|input| {
                input.candidate_facts[index].statement = SourceStatementId::new(other)
            });
            assert_base_rejects(&format!("base candidate {index} context"), &|input| {
                input.candidate_facts[index].context = SourceStatementContextId::new(other)
            });
            assert_base_rejects(&format!("base candidate {index} ordinal"), &|input| {
                input.candidate_facts[index].ordinal = 1
            });
            assert_base_rejects(&format!("base candidate {index} formula"), &|input| {
                input.candidate_facts[index].formula =
                    SourceStatementFormulaTarget::Atomic(SourceAtomicFormulaId::new(other))
            });
        }
        for (index, replacement) in [(0, 1), (1, 1)] {
            let mut input = fixture.statement_input();
            input.statements[index].source_ordinal = replacement;
            assert_eq!(
                SourceStatementProducer::build(
                    input,
                    &fixture.symbols,
                    &fixture.bindings,
                    &fixture.primary,
                    &fixture.atomic,
                    &fixture.arena,
                ),
                Err(SourceStatementError::InvalidStatement {
                    statement: SourceStatementId::new(index),
                }),
                "base source ordinal {index}"
            );
            assert_replay();
        }
        let mut missing_owner = fixture.statement_input();
        missing_owner.owners.clear();
        assert_eq!(
            SourceStatementProducer::build(
                missing_owner,
                &fixture.symbols,
                &fixture.bindings,
                &fixture.primary,
                &fixture.atomic,
                &fixture.arena,
            ),
            Err(SourceStatementError::InvalidAggregate)
        );
        assert_replay();
        let mut crossed_context = fixture.statement_input();
        crossed_context.contexts[1].binding_context = BindingContextId::new(0);
        assert_eq!(
            SourceStatementProducer::build(
                crossed_context,
                &fixture.symbols,
                &fixture.bindings,
                &fixture.primary,
                &fixture.atomic,
                &fixture.arena,
            ),
            Err(SourceStatementError::InvalidContext {
                context: SourceStatementContextId::new(1),
            })
        );
        assert_replay();
        let mut crossed_input = fixture.statement_input();
        crossed_input.input_facts[1].uses.swap(0, 1);
        assert_eq!(
            SourceStatementProducer::build(
                crossed_input,
                &fixture.symbols,
                &fixture.bindings,
                &fixture.primary,
                &fixture.atomic,
                &fixture.arena,
            ),
            Err(SourceStatementError::InvalidInputFact {
                fact: SourceStatementInputFactId::new(1),
            })
        );
        assert_replay();
        let mut crossed_candidate = fixture.statement_input();
        crossed_candidate.candidate_facts[1].formula =
            SourceStatementFormulaTarget::Atomic(SourceAtomicFormulaId::new(0));
        assert_eq!(
            SourceStatementProducer::build(
                crossed_candidate,
                &fixture.symbols,
                &fixture.bindings,
                &fixture.primary,
                &fixture.atomic,
                &fixture.arena,
            ),
            Err(SourceStatementError::InvalidCandidateFact {
                fact: SourceStatementCandidateFactId::new(1),
            })
        );
        assert_replay();
        let mut stale_statement = valid.clone();
        stale_statement.statement_fingerprint.push('x');
        assert_eq!(
            stale_statement.validate_installation(
                fixture.source,
                &fixture.module,
                &statement,
                &fixture.primary,
                &fixture.arena,
            ),
            Err(SourceStatementWitnessError::DependencyMismatch)
        );
        assert_replay();
        let mut stale_primary = valid.clone();
        stale_primary.primary_term_fingerprint.push('x');
        assert_eq!(
            stale_primary.validate_installation(
                fixture.source,
                &fixture.module,
                &statement,
                &fixture.primary,
                &fixture.arena,
            ),
            Err(SourceStatementWitnessError::DependencyMismatch)
        );
        assert_replay();
        let b1 = B1Fixture::new(41);
        assert_eq!(
            valid.validate_installation(
                fixture.source,
                &fixture.module,
                &statement,
                &b1.primary,
                &fixture.arena,
            ),
            Err(SourceStatementWitnessError::DependencyMismatch)
        );
        assert_replay();
        let mut foreign_contexts = binding_env(fixture.source, &fixture.module)
            .contexts()
            .clone();
        foreign_contexts.insert(BindingContextDraft {
            owner: BindingContextOwner::SourceStatement {
                source_range: range(fixture.source, 69, 102),
            },
            parent: Some(BindingContextId::new(0)),
            layer: BindingContextLayer::Proof,
            lexical_scope: Some(LocalTermScope::new(vec![9])),
            bindings: Vec::new(),
            visible_bindings: vec![BindingId::new(0)],
            recovery: BindingContextRecovery::Normal,
        });
        let foreign_binding = BindingEnv::try_new(BindingEnvParts {
            source_id: fixture.source,
            module_id: fixture.module.clone(),
            contexts: foreign_contexts,
            bindings: fixture.bindings.bindings().clone(),
            diagnostics: fixture.bindings.diagnostics().clone(),
        })
        .expect("foreign proof scope binding env");
        let foreign_primary = SourcePrimaryTermProducer::build(
            b3_primary_input(fixture.source, &fixture.module),
            &foreign_binding,
            &fixture.arena,
        )
        .expect("foreign proof scope lower handoff");
        assert_eq!(
            valid.validate_installation(
                fixture.source,
                &fixture.module,
                &statement,
                &foreign_primary,
                &fixture.arena,
            ),
            Err(SourceStatementWitnessError::DependencyMismatch)
        );
        assert_replay();
        assert_eq!(
            valid.validate_installation(
                source_id(999),
                &fixture.module,
                &statement,
                &fixture.primary,
                &fixture.arena,
            ),
            Err(SourceStatementWitnessError::DependencyMismatch)
        );
        assert_eq!(
            valid.validate_installation(
                fixture.source,
                &ModuleId::new(PackageId::new("pkg"), ModulePath::new("statement.foreign")),
                &statement,
                &fixture.primary,
                &fixture.arena,
            ),
            Err(SourceStatementWitnessError::DependencyMismatch)
        );
        assert_replay();
        let substituted_wrapper = mutate_arena(&fixture.arena, |id, row| {
            if id == TypedNodeId::new(33) {
                row.children = vec![TypedNodeId::new(26)];
            }
        });
        assert_eq!(
            valid.validate_installation(
                fixture.source,
                &fixture.module,
                &statement,
                &fixture.primary,
                &substituted_wrapper,
            ),
            Err(SourceStatementWitnessError::DependencyMismatch)
        );
        assert_replay();
        let substituted_reference = mutate_arena(&fixture.arena, |id, row| {
            if id == TypedNodeId::new(32) {
                row.children = vec![TypedNodeId::new(8)];
            }
        });
        assert_eq!(
            valid.validate_installation(
                fixture.source,
                &fixture.module,
                &statement,
                &fixture.primary,
                &substituted_reference,
            ),
            Err(SourceStatementWitnessError::DependencyMismatch)
        );
        assert_replay();
        for index in 0..49 {
            let drifted_range = mutate_arena(&fixture.arena, |id, row| {
                if id.index() == index {
                    let SourceAnchor::Range(mut source_range) = row.anchor.clone() else {
                        unreachable!("Task258B3 range anchor")
                    };
                    source_range.end += 1;
                    row.anchor = SourceAnchor::Range(source_range);
                }
            });
            assert_eq!(
                valid.validate_installation(
                    fixture.source,
                    &fixture.module,
                    &statement,
                    &fixture.primary,
                    &drifted_range,
                ),
                Err(SourceStatementWitnessError::DependencyMismatch),
                "node {index} range"
            );
            let recovered = mutate_arena(&fixture.arena, |id, row| {
                if id.index() == index {
                    row.recovery = NodeRecoveryState::Recovered;
                }
            });
            assert_eq!(
                valid.validate_installation(
                    fixture.source,
                    &fixture.module,
                    &statement,
                    &fixture.primary,
                    &recovered,
                ),
                Err(SourceStatementWitnessError::DependencyMismatch),
                "node {index} recovery"
            );
            let degraded = mutate_arena(&fixture.arena, |id, row| {
                if id.index() == index {
                    row.recovery = NodeRecoveryState::Degraded;
                }
            });
            assert_eq!(
                valid.validate_installation(
                    fixture.source,
                    &fixture.module,
                    &statement,
                    &fixture.primary,
                    &degraded,
                ),
                Err(SourceStatementWitnessError::DependencyMismatch),
                "node {index} degraded recovery"
            );
            let drifted_kind = mutate_arena(&fixture.arena, |id, row| {
                if id.index() == index {
                    row.kind = "source.task258b3.mutated".into();
                }
            });
            assert_eq!(
                valid.validate_installation(
                    fixture.source,
                    &fixture.module,
                    &statement,
                    &fixture.primary,
                    &drifted_kind,
                ),
                Err(SourceStatementWitnessError::DependencyMismatch),
                "node {index} kind"
            );
            let drifted_children = mutate_arena(&fixture.arena, |id, row| {
                if id.index() == index {
                    if row.children.len() > 1 {
                        row.children.swap(0, 1);
                    } else if row.children.len() == 1 {
                        row.children.clear();
                    } else {
                        row.children.push(TypedNodeId::new(usize::from(index == 0)));
                    }
                }
            });
            assert_eq!(
                valid.validate_installation(
                    fixture.source,
                    &fixture.module,
                    &statement,
                    &fixture.primary,
                    &drifted_children,
                ),
                Err(SourceStatementWitnessError::DependencyMismatch),
                "node {index} children"
            );
            assert_replay();
        }
    }

    #[test]
    fn task258b3_paired_typed_ownership_and_cross_family_orders_are_atomic() {
        let fixture = B3Fixture::new(42);
        let statement = fixture.statement();
        let witnesses = fixture
            .witnesses(&statement, fixture.witness_input())
            .expect("witnesses");
        let base = fixture.empty_typed();
        let base_debug = base.debug_text();
        assert_eq!(
            base.clone().with_source_statement(statement.clone()),
            Err(TypedAstError::InvalidSourceStatement)
        );
        assert_eq!(base.debug_text(), base_debug);
        let installed = base
            .clone()
            .with_source_statement_witnesses(statement.clone(), witnesses.clone())
            .expect("paired install");
        assert_eq!(installed.source_statement(), Some(&statement));
        assert_eq!(installed.source_statement_witnesses(), Some(&witnesses));
        assert!(installed.source_statement_references().is_none());
        let installed_debug = installed.debug_text();
        assert_eq!(
            installed
                .clone()
                .with_source_statement_witnesses(statement.clone(), witnesses.clone()),
            Err(TypedAstError::InvalidSourceStatement)
        );
        assert_eq!(installed.debug_text(), installed_debug);

        let task258a = Fixture::new(42);
        let task258a_statement = task258a
            .build(task258a.input())
            .expect("Task258A statement");
        let task258a_installed = TypedAst::try_new(TypedAstParts {
            source_id: task258a.source,
            module_id: task258a.module.clone(),
            resolved_root: None,
            source_context: None,
            source_type: None,
            source_attribute: None,
            nodes: task258a.arena.clone(),
            contexts: LocalTypeContextTable::new(),
            types: TypeTable::new(),
            facts: TypeFactTable::new(),
            coercions: CoercionTable::new(),
            initial_obligations: InitialObligationTable::new(),
            diagnostics: TypeDiagnosticTable::new(),
        })
        .expect("Task258A empty typed AST")
        .with_source_term(task258a.primary.clone())
        .expect("Task258A Task252")
        .with_source_atomic_formula(task258a.atomic.clone())
        .expect("Task258A Task256")
        .with_source_statement(task258a_statement.clone())
        .expect("Task258A install");
        let task258a_debug = task258a_installed.debug_text();
        assert_eq!(
            task258a_installed
                .clone()
                .with_source_statement_witnesses(statement.clone(), witnesses.clone()),
            Err(TypedAstError::InvalidSourceStatement)
        );
        assert_eq!(task258a_installed.debug_text(), task258a_debug);
        assert_eq!(
            installed.clone().with_source_statement(task258a_statement),
            Err(TypedAstError::InvalidSourceStatement)
        );
        assert_eq!(installed.debug_text(), installed_debug);

        let b2 = B2Fixture::new(42);
        let b2_statement = b2.build(b2.input()).expect("B2 statement");
        let b2_installed = b2
            .empty_typed()
            .with_source_statement(b2_statement.clone())
            .expect("B2 install");
        let b2_debug = b2_installed.debug_text();
        assert_eq!(
            b2_installed
                .clone()
                .with_source_statement_witnesses(statement.clone(), witnesses.clone()),
            Err(TypedAstError::InvalidSourceStatement)
        );
        assert_eq!(b2_installed.debug_text(), b2_debug);
        assert_eq!(
            installed.clone().with_source_statement(b2_statement),
            Err(TypedAstError::InvalidSourceStatement)
        );
        assert_eq!(installed.debug_text(), installed_debug);
        let b1 = B1Fixture::new(42);
        let b1_references = b1.references(b1.reference_input()).expect("B1 references");
        let b1_installed = b1
            .empty_typed()
            .with_source_statement_references(b1.statement.clone(), b1_references.clone())
            .expect("B1 install");
        let b1_debug = b1_installed.debug_text();
        assert_eq!(
            b1_installed
                .clone()
                .with_source_statement_witnesses(statement.clone(), witnesses.clone()),
            Err(TypedAstError::InvalidSourceStatement)
        );
        assert_eq!(b1_installed.debug_text(), b1_debug);
        assert_eq!(
            installed
                .clone()
                .with_source_statement_references(b1.statement.clone(), b1_references,),
            Err(TypedAstError::InvalidSourceStatement)
        );
        assert_eq!(installed.debug_text(), installed_debug);
        let task248 = crate::source_context::tests::task_248_occupied_typed_ast(
            fixture.source,
            fixture.module.clone(),
        );
        let task248_debug = task248.debug_text();
        assert_eq!(
            task248
                .clone()
                .with_source_statement_witnesses(statement.clone(), witnesses.clone()),
            Err(TypedAstError::InvalidSourceStatement)
        );
        assert_eq!(task248.debug_text(), task248_debug);
        assert_eq!(
            installed.clone().with_source_context_for_test(
                task248
                    .source_context()
                    .expect("Task248 source context")
                    .clone(),
            ),
            Err(TypedAstError::InvalidSourceContext)
        );
        assert_eq!(installed.debug_text(), installed_debug);

        let task257a = crate::source_composite_formula::tests::task_257a_installed_typed_ast();
        let task257b = crate::source_formula_composition::tests::task_257b_installed_typed_ast();
        let task257c2 = crate::source_formula_composition::tests::task_257c2_installed_typed_ast();
        let task257c3 = crate::source_formula_composition::tests::task_257c3_installed_typed_ast();
        for (family, occupied) in [
            ("Task257A", &task257a),
            ("Task257B", &task257b),
            ("Task257C2", &task257c2),
            ("Task257C3", &task257c3),
        ] {
            let debug = occupied.debug_text();
            assert_eq!(
                occupied
                    .clone()
                    .with_source_statement_witnesses(statement.clone(), witnesses.clone()),
                Err(TypedAstError::InvalidSourceStatement),
                "{family} first"
            );
            assert_eq!(occupied.debug_text(), debug, "{family} rollback");
        }
        assert_eq!(
            installed.clone().with_source_composite_formula(
                task257a
                    .source_composite_formula()
                    .expect("Task257A composite")
                    .clone(),
            ),
            Err(TypedAstError::InvalidSourceCompositeFormula)
        );
        assert_eq!(
            installed.clone().with_source_formula_composition(
                task257b
                    .source_composite_formula()
                    .expect("Task257B composite")
                    .clone(),
                task257b
                    .source_formula_composition()
                    .expect("Task257B composition")
                    .clone(),
            ),
            Err(TypedAstError::InvalidSourceFormulaComposition)
        );
        assert_eq!(
            installed.clone().with_source_condition_formula_composition(
                task257c2
                    .source_condition_formula_composition()
                    .expect("Task257C2 composition")
                    .clone(),
            ),
            Err(TypedAstError::InvalidSourceConditionFormulaComposition)
        );
        assert_eq!(
            installed.clone().with_source_predicate_chain_composition(
                task257c3
                    .source_predicate_chain_composition()
                    .expect("Task257C3 composition")
                    .clone(),
            ),
            Err(TypedAstError::InvalidSourcePredicateChainComposition)
        );
        assert_eq!(installed.debug_text(), installed_debug);
        assert_eq!(
            base.with_source_statement_witnesses(statement, witnesses)
                .expect("Task258B3 replay")
                .debug_text(),
            installed_debug
        );
    }

    #[test]
    fn task258b3_final_clone_orphan_stale_half_and_empty_semantics_are_stable() {
        let fixture = B3Fixture::new(43);
        let statement = fixture.statement();
        let witnesses = fixture
            .witnesses(&statement, fixture.witness_input())
            .expect("witnesses");
        let typed = fixture
            .empty_typed()
            .with_source_statement_witnesses(statement.clone(), witnesses.clone())
            .expect("typed pair");
        let resolved = assemble_empty_resolved(&typed).expect("final assembly");
        let typed_debug = typed.debug_text();
        let resolved_debug = resolved.debug_text();
        assert_eq!(resolved.source_statement(), Some(&statement));
        assert_eq!(resolved.source_statement_witnesses(), Some(&witnesses));
        assert!(resolved.source_statement_references().is_none());
        assert_eq!(resolved.clone().debug_text(), resolved.debug_text());
        assert!(resolved.expr_metadata().is_empty());
        assert!(resolved.collection_candidates().is_empty());
        assert!(resolved.expanded_candidates().is_empty());
        assert!(resolved.template_expansions().is_empty());
        assert!(resolved.viable_candidates().is_empty());
        assert!(resolved.viability_decisions().is_empty());
        assert!(resolved.specificity_graphs().is_empty());
        assert!(resolved.resolved_overloads().is_empty());
        assert!(resolved.inserted_coercions().is_empty());
        assert!(resolved.cluster_facts().is_empty());
        assert!(resolved.diagnostics().is_empty());
        assert!(resolved.checked_formulas().is_empty());
        assert!(resolved.statement_semantics().is_empty());
        assert!(resolved.checked_proofs().is_empty());
        assert!(resolved.checked_proof_nodes().is_empty());
        assert!(resolved.checked_terminal_goals().is_empty());

        let mut orphan = fixture.empty_typed();
        orphan.inject_source_statement_witnesses_for_test(witnesses.clone());
        assert_eq!(
            assemble_empty_resolved(&orphan),
            Err(ResolvedTypedAstError::InvalidSourceStatement)
        );
        let mut standalone = fixture.empty_typed();
        standalone.inject_source_statement_for_test(statement.clone());
        assert_eq!(
            assemble_empty_resolved(&standalone),
            Err(ResolvedTypedAstError::InvalidSourceStatement)
        );
        let mut stale = witnesses;
        stale.statement_fingerprint.push('x');
        let mut stale_pair = fixture.empty_typed();
        stale_pair.inject_source_statement_witness_bundle_for_test(statement, stale);
        assert_eq!(
            assemble_empty_resolved(&stale_pair),
            Err(ResolvedTypedAstError::InvalidSourceStatement)
        );
        let statement = typed.source_statement().expect("statement").clone();
        let witnesses = typed
            .source_statement_witnesses()
            .expect("witnesses")
            .clone();
        let mut stale_primary = witnesses.clone();
        stale_primary.primary_term_fingerprint.push('x');
        let mut stale_primary_pair = fixture.empty_typed();
        stale_primary_pair
            .inject_source_statement_witness_bundle_for_test(statement.clone(), stale_primary);
        assert_eq!(
            assemble_empty_resolved(&stale_primary_pair),
            Err(ResolvedTypedAstError::InvalidSourceStatement)
        );
        let b2 = B2Fixture::new(43);
        let mut cross_profile = fixture.empty_typed();
        cross_profile.inject_source_statement_witness_bundle_for_test(
            b2.build(b2.input()).expect("B2 statement"),
            witnesses.clone(),
        );
        assert_eq!(
            assemble_empty_resolved(&cross_profile),
            Err(ResolvedTypedAstError::InvalidSourceStatement)
        );
        let b1 = B1Fixture::new(43);
        let mut references_and_witnesses = fixture.empty_typed();
        references_and_witnesses.inject_source_statement_bundle_for_test(
            statement.clone(),
            b1.references(b1.reference_input()).expect("B1 references"),
        );
        references_and_witnesses.inject_source_statement_witnesses_for_test(witnesses);
        assert_eq!(
            assemble_empty_resolved(&references_and_witnesses),
            Err(ResolvedTypedAstError::InvalidSourceStatement)
        );

        for table in [
            StatementTransportTableForTest::Context,
            StatementTransportTableForTest::Type,
            StatementTransportTableForTest::Fact,
            StatementTransportTableForTest::Coercion,
            StatementTransportTableForTest::InitialObligation,
            StatementTransportTableForTest::Diagnostic,
        ] {
            let mut occupied = typed.clone();
            occupied.occupy_statement_transport_table_for_test(table);
            let occupied_debug = occupied.debug_text();
            assert_eq!(
                assemble_empty_resolved(&occupied),
                Err(ResolvedTypedAstError::InvalidSourceStatement),
                "{table:?}"
            );
            assert_eq!(occupied.debug_text(), occupied_debug, "{table:?} rollback");
        }
        let mut cluster_facts = ClusterFactTable::new();
        cluster_facts.insert(ClusterFactDraft {
            fingerprint: ClusterFactFingerprint::new("task258b3-coexistence"),
            source_type: ClusterTypeFingerprint::new("set"),
            attribute: ClusterAttributeFingerprint::new("inhabited"),
            generated_type: ClusterTypeFingerprint::new("set"),
            provenance: ClusterFactProvenance::Input,
            source_range: range(fixture.source, 82, 83),
        });
        assert_eq!(
            assemble_resolved(&typed, &cluster_facts, Vec::new(), None),
            Err(ResolvedTypedAstError::InvalidSourceStatement)
        );
        assert_eq!(
            assemble_resolved(
                &typed,
                &ClusterFactTable::new(),
                vec![ResolvedNodeKindHint {
                    typed_node: TypedNodeId::new(32),
                    kind: ResolvedNodeKindHintKind::SourcePreserved {
                        role: SourceNodeRole::new("source.statement-witness.semantic-coexistence",),
                    },
                }],
                None,
            ),
            Err(ResolvedTypedAstError::InvalidSourceStatement)
        );
        assert_eq!(
            assemble_resolved(
                &typed,
                &ClusterFactTable::new(),
                Vec::new(),
                Some(StatementProofInputs {
                    owner: statement.checked_owner(),
                    rows: Vec::new(),
                }),
            ),
            Err(ResolvedTypedAstError::InvalidSourceStatement)
        );
        assert_eq!(
            assemble_resolved_with_expressions(
                &typed,
                vec![ExpressionMetadataInput {
                    expr: ExprId::new("task258b3-semantic-coexistence"),
                    typed_site: node(32),
                    local_context: None,
                    cluster_facts: Vec::new(),
                }],
            ),
            Err(ResolvedTypedAstError::InvalidSourceStatement)
        );
        let term_formula = TermFormulaChecker::default().infer(
            &fixture.symbols,
            &fixture.bindings,
            Vec::<crate::type_checker::TermInput>::new(),
            Vec::<crate::type_checker::FormulaInput>::new(),
        );
        assert_eq!(
            assemble_resolved_with_statement_semantics(
                &typed,
                statement.checked_owner(),
                &fixture.bindings,
                &term_formula,
            ),
            Err(ResolvedTypedAstError::InvalidSourceStatement)
        );
        assert_eq!(typed.debug_text(), typed_debug);
        assert_eq!(
            assemble_empty_resolved(&typed)
                .expect("replay")
                .debug_text(),
            resolved_debug
        );
    }

    #[test]
    fn task258b3n_exact_name_api_debug_and_b3_byte_compatibility_are_stable() {
        let fixture = B3NFixture::new(44);
        let statement = fixture.statement();
        assert!(statement.is_task_258b3n_profile());
        assert!(!statement.is_task_258b3_profile());
        let handoff = fixture
            .witnesses(&statement, fixture.witness_input())
            .expect("Task258B3N witness handoff");
        assert_eq!(handoff.source_id(), fixture.source);
        assert_eq!(handoff.module_id(), &fixture.module);
        assert_eq!((handoff.witnesses().len(), handoff.names().len()), (1, 1));
        let witness_id = SourceStatementWitnessId::new(0);
        let name_id = SourceStatementWitnessNameId::new(0);
        assert_eq!((witness_id.index(), name_id.index()), (0, 0));
        let witness = handoff.witnesses().get(witness_id).expect("witness");
        assert_eq!(witness.owner(), SourceTheoremOwnerId::new(0));
        assert_eq!(witness.binding_context(), BindingContextId::new(1));
        assert_eq!(
            witness.term(),
            SourceStatementWitnessTermTarget::Primary(SourcePrimaryTermId::new(2))
        );
        assert_eq!(
            (
                witness.take_site().node().index(),
                witness.site().node().index()
            ),
            (37, 36)
        );
        assert_eq!(witness.take_range(), range(fixture.source, 76, 87));
        assert_eq!(witness.source_range(), range(fixture.source, 81, 86));
        assert_eq!((witness.source_ordinal(), witness.ordinal()), (1, 0));
        assert_eq!(witness.spelling(), "y = x");
        assert_eq!(witness.kind(), SourceStatementWitnessKind::Named);
        assert_eq!(witness.recovery(), SourceStatementRecovery::Normal);
        assert_eq!(witness.name(), Some(name_id));
        let name = handoff.names().get(name_id).expect("name");
        assert_eq!(name.witness(), witness_id);
        assert_eq!(name.site().node().index(), 13);
        assert_eq!(name.source_range(), range(fixture.source, 81, 82));
        assert_eq!(name.spelling(), "y");
        assert_eq!(name.recovery(), SourceStatementRecovery::Normal);
        assert_eq!(
            handoff
                .names()
                .iter()
                .map(|(id, row)| (id.index(), row.spelling()))
                .collect::<Vec<_>>(),
            [(0, "y")]
        );
        assert_eq!(
            handoff.debug_text(),
            format!(
                "source-statement-witness-debug-v1\nmodule: pkg::statement.fixture\nstatement-fingerprint: {:?}\nprimary-term-fingerprint: {:?}\nwitness#0 owner=0 binding_context=1 term=primary#2 take_range=76..87 take_site=37 range=81..86 site=36 source_ordinal=1 ordinal=0 kind=named recovery=normal spelling=\"y = x\" name=0\nwitness-name#0 witness=0 range=81..82 site=13 recovery=normal spelling=\"y\"\n",
                statement.debug_text(),
                fixture.primary.debug_text(),
            )
        );
        assert_eq!(
            (
                fixture.bindings.contexts().len(),
                fixture.bindings.bindings().len(),
                fixture.bindings.diagnostics().len(),
                fixture.primary.terms().len(),
                fixture.primary.references().len(),
                fixture.primary.numeric_type_requests().len(),
            ),
            (2, 1, 0, 5, 5, 0)
        );
        assert_eq!(
            (
                fixture.atomic.formulas().len(),
                fixture.atomic.wrappers().len(),
                fixture.atomic.predicate_segments().len(),
                fixture.atomic.predicate_heads().len(),
                fixture.atomic.candidates().len(),
                fixture.atomic.type_sites().len(),
                fixture.atomic.attributes().len(),
                fixture.atomic.edges().len(),
                fixture.atomic.requests().len(),
            ),
            (2, 0, 0, 0, 0, 0, 0, 4, 4)
        );
        let proof = fixture
            .bindings
            .contexts()
            .get(BindingContextId::new(1))
            .expect("proof context");
        assert_eq!(
            (
                &proof.owner,
                proof.parent,
                proof.layer,
                proof
                    .lexical_scope
                    .as_ref()
                    .map(|scope| scope.path().to_vec()),
                proof
                    .bindings
                    .iter()
                    .map(|id| id.index())
                    .collect::<Vec<_>>(),
                proof
                    .visible_bindings
                    .iter()
                    .map(|id| id.index())
                    .collect::<Vec<_>>(),
                proof.recovery,
            ),
            (
                &BindingContextOwner::SourceStatement {
                    source_range: range(fixture.source, 68, 105),
                },
                Some(BindingContextId::new(0)),
                BindingContextLayer::Proof,
                Some(vec![0]),
                Vec::new(),
                vec![0],
                BindingContextRecovery::Normal,
            )
        );
        for (index, ((_, term), (_, reference))) in fixture
            .primary
            .terms()
            .iter()
            .zip(fixture.primary.references().iter())
            .enumerate()
        {
            assert_eq!(
                (
                    term.site().node().index(),
                    term.source_range(),
                    term.source_ordinal(),
                    term.context().index(),
                    term.spelling(),
                    term.kind(),
                    term.role(),
                    term.recovery(),
                    term.parent(),
                ),
                (
                    [28, 30, 34, 38, 40][index],
                    range(
                        fixture.source,
                        [62, 66, 85, 95, 99][index],
                        [63, 67, 86, 96, 100][index],
                    ),
                    index,
                    [0, 0, 1, 1, 1][index],
                    "x",
                    SourcePrimaryTermKind::VariableReference,
                    SourcePrimaryTermRole::Value,
                    SourcePrimaryTermRecovery::Normal,
                    None,
                )
            );
            assert_eq!(
                (
                    reference.term().index(),
                    reference.binding().index(),
                    reference.role(),
                    reference.use_ordinal(),
                    reference.lexical_scope().map(|scope| scope.path().to_vec()),
                ),
                (
                    index,
                    0,
                    SourcePrimaryTermReferenceRole::Variable,
                    1,
                    if index < 2 { None } else { Some(vec![0]) },
                )
            );
        }
        for index in 0..2 {
            let formula = fixture
                .atomic
                .formulas()
                .get(SourceAtomicFormulaId::new(index))
                .expect("formula");
            assert_eq!(
                (
                    formula.site().node().index(),
                    formula.source_range(),
                    formula.source_ordinal(),
                    formula.context().index(),
                    formula.spelling(),
                    formula.kind(),
                    formula.recovery(),
                ),
                (
                    [32, 42][index],
                    range(fixture.source, [62, 95][index], [67, 100][index]),
                    index,
                    index,
                    "x = x",
                    SourceAtomicFormulaKind::Equality,
                    SourceAtomicFormulaRecovery::Normal,
                )
            );
        }
        assert_eq!(
            fixture
                .atomic
                .edges()
                .iter()
                .map(|(_, edge)| match edge.target() {
                    SourceAtomicTermTarget::Primary(term) => term.index(),
                    _ => usize::MAX,
                })
                .collect::<Vec<_>>(),
            [0, 1, 3, 4]
        );
        for index in 0..4 {
            let formula = usize::from(index >= 2);
            let ordinal = index % 2;
            let edge = fixture
                .atomic
                .edges()
                .get(SourceAtomicEdgeId::new(index))
                .expect("edge");
            assert_eq!((edge.formula().index(), edge.ordinal()), (formula, ordinal));
            assert_eq!(
                edge.role(),
                if ordinal == 0 {
                    SourceAtomicEdgeRole::BuiltinLeftOperand
                } else {
                    SourceAtomicEdgeRole::BuiltinRightOperand
                }
            );
            let request = fixture
                .atomic
                .requests()
                .get(SourceAtomicRequestId::new(index))
                .expect("request");
            assert_eq!(
                (
                    request.formula().index(),
                    request.ordinal(),
                    request.kind(),
                    request.edge().map(|id| id.index()),
                    request.candidate(),
                    request.type_site(),
                    request.attribute(),
                ),
                (
                    formula,
                    ordinal,
                    SourceAtomicRequestKind::OperandExpectedType,
                    Some(index),
                    None,
                    None,
                    None,
                )
            );
        }
        assert_eq!(
            (
                statement.owners().len(),
                statement.statements().len(),
                statement.contexts().len(),
                statement.input_facts().len(),
                statement.candidate_facts().len(),
            ),
            (1, 2, 2, 2, 2)
        );
        let owner = statement
            .owners()
            .get(SourceTheoremOwnerId::new(0))
            .expect("owner");
        assert_eq!(
            (
                owner.symbol(),
                owner.contribution(),
                owner.site().node().index(),
                owner.source_range(),
                owner.spelling(),
                owner.role(),
                owner.status(),
                owner.recovery(),
            ),
            (
                &fixture.symbol,
                fixture.contribution,
                47,
                range(fixture.source, 19, 106),
                "FormulaStatementNamedWitnessSmoke",
                SourceTheoremRole::Theorem,
                SourceTheoremStatus::Unmodified,
                SourceStatementRecovery::Normal,
            )
        );
        assert_eq!(statement.checked_owner().symbol(), &fixture.symbol);
        assert_eq!(statement.checked_owner().origin().structural_path(), [2, 1]);
        let labels = fixture
            .symbols
            .labels()
            .by_contribution(fixture.contribution);
        assert_eq!(labels.len(), 1);
        assert_eq!(
            (
                labels[0].primary_spelling(),
                labels[0].kind(),
                labels[0].visibility(),
                labels[0].export_status(),
                labels[0].contribution(),
                labels[0].origin().structural_path(),
            ),
            (
                "FormulaStatementNamedWitnessSmoke",
                mizar_resolve::resolved_ast::LabelKind::Theorem,
                Visibility::Public,
                ExportStatus::Exported,
                fixture.contribution,
                &[2, 1][..],
            )
        );
        assert!(fixture.symbols.imports().is_empty());
        for index in 0..2 {
            let row = statement
                .statements()
                .get(SourceStatementId::new(index))
                .expect("statement row");
            assert_eq!(
                (
                    row.owner().index(),
                    row.context().index(),
                    row.formula(),
                    row.site().node().index(),
                    row.source_range(),
                    row.source_ordinal(),
                    row.spelling(),
                    row.kind(),
                    row.recovery(),
                ),
                (
                    0,
                    index,
                    SourceStatementFormulaTarget::Atomic(SourceAtomicFormulaId::new(index)),
                    [47, 45][index],
                    range(fixture.source, [19, 90][index], [106, 101][index]),
                    [0, 2][index],
                    [
                        "theorem FormulaStatementNamedWitnessSmoke : x = x proof take y = x ; thus x = x ; end ;",
                        "thus x = x ;",
                    ][index],
                    [
                        SourceStatementKind::TheoremProposition,
                        SourceStatementKind::Conclusion,
                    ][index],
                    SourceStatementRecovery::Normal,
                )
            );
            let context = statement
                .contexts()
                .get(SourceStatementContextId::new(index))
                .expect("statement context");
            assert_eq!(
                (
                    context.statement().index(),
                    context.binding_context().index(),
                    context.source_range(),
                    context
                        .visible_bindings()
                        .iter()
                        .map(|id| id.index())
                        .collect::<Vec<_>>(),
                ),
                (index, index, row.source_range(), vec![0])
            );
            let fact = statement
                .input_facts()
                .get(SourceStatementInputFactId::new(index))
                .expect("input fact");
            assert_eq!(
                (
                    fact.statement().index(),
                    fact.context().index(),
                    fact.ordinal(),
                    fact.kind(),
                    fact.binding().index(),
                    fact.uses().iter().map(|id| id.index()).collect::<Vec<_>>(),
                ),
                (
                    index,
                    index,
                    0,
                    SourceStatementInputFactKind::ReservedTypeGuard,
                    0,
                    if index == 0 { vec![0, 1] } else { vec![3, 4] },
                )
            );
            let candidate = statement
                .candidate_facts()
                .get(SourceStatementCandidateFactId::new(index))
                .expect("candidate fact");
            assert_eq!(
                (
                    candidate.statement().index(),
                    candidate.context().index(),
                    candidate.ordinal(),
                    candidate.kind(),
                    candidate.formula(),
                ),
                (
                    index,
                    index,
                    0,
                    SourceStatementCandidateFactKind::UnverifiedProposition,
                    row.formula(),
                )
            );
        }
        let typed = fixture
            .empty_typed()
            .with_source_statement_witnesses(statement.clone(), handoff.clone())
            .expect("paired typed publication");
        let resolved = assemble_empty_resolved(&typed).expect("paired final publication");
        assert_eq!(typed.source_statement(), Some(&statement));
        assert_eq!(typed.source_statement_witnesses(), Some(&handoff));
        assert!(typed.source_statement_references().is_none());
        assert_eq!(resolved.source_statement(), Some(&statement));
        assert_eq!(resolved.source_statement_witnesses(), Some(&handoff));
        assert!(resolved.source_statement_references().is_none());
        assert!(resolved.expr_metadata().is_empty());
        assert!(resolved.collection_candidates().is_empty());
        assert!(resolved.expanded_candidates().is_empty());
        assert!(resolved.template_expansions().is_empty());
        assert!(resolved.viable_candidates().is_empty());
        assert!(resolved.viability_decisions().is_empty());
        assert!(resolved.specificity_graphs().is_empty());
        assert!(resolved.resolved_overloads().is_empty());
        assert!(resolved.inserted_coercions().is_empty());
        assert!(resolved.cluster_facts().is_empty());
        assert!(resolved.diagnostics().is_empty());
        assert!(resolved.checked_formulas().is_empty());
        assert!(resolved.statement_semantics().is_empty());
        assert!(resolved.checked_proofs().is_empty());
        assert!(resolved.checked_proof_nodes().is_empty());
        assert!(resolved.checked_terminal_goals().is_empty());

        let b3 = B3Fixture::new(45);
        let b3_statement = b3.statement();
        let b3_handoff = b3
            .witnesses(&b3_statement, b3.witness_input())
            .expect("B3 handoff");
        assert!(b3_handoff.names().is_empty());
        assert_eq!(
            b3_handoff.debug_text(),
            format!(
                "source-statement-witness-debug-v1\nmodule: pkg::statement.fixture\nstatement-fingerprint: {:?}\nprimary-term-fingerprint: {:?}\nwitness#0 owner=0 binding_context=1 term=primary#2 take_range=77..84 take_site=35 range=82..83 site=34 source_ordinal=1 ordinal=0 kind=unnamed recovery=normal spelling=\"x\"\n",
                b3_statement.debug_text(),
                b3.primary.debug_text(),
            )
        );
    }

    #[test]
    fn task258b3n_dependency_aggregate_witness_name_and_all_nodes_fail_closed() {
        let fixture = B3NFixture::new(46);
        let statement = fixture.statement();
        let baseline = fixture
            .witnesses(&statement, fixture.witness_input())
            .expect("baseline");
        let baseline_debug = baseline.debug_text();

        let mut aggregate = fixture.witness_input();
        aggregate.names.clear();
        assert_eq!(
            fixture.witnesses(&statement, aggregate),
            Err(SourceStatementWitnessError::InvalidAggregate)
        );
        let mut duplicate_witness = fixture.witness_input();
        duplicate_witness
            .witnesses
            .push(duplicate_witness.witnesses[0].clone());
        assert_eq!(
            fixture.witnesses(&statement, duplicate_witness),
            Err(SourceStatementWitnessError::InvalidAggregate)
        );
        let mut duplicate_name = fixture.witness_input();
        duplicate_name.names.push(duplicate_name.names[0].clone());
        assert_eq!(
            fixture.witnesses(&statement, duplicate_name),
            Err(SourceStatementWitnessError::InvalidAggregate)
        );
        let mut invalid_witness = fixture.witness_input();
        invalid_witness.witnesses[0].kind = SourceStatementWitnessKind::Unnamed;
        assert_eq!(
            fixture.witnesses(&statement, invalid_witness),
            Err(SourceStatementWitnessError::InvalidWitness {
                witness: SourceStatementWitnessId::new(0)
            })
        );
        let mut invalid_forward = fixture.witness_input();
        invalid_forward.witnesses[0].name = Some(SourceStatementWitnessNameId::new(1));
        assert_eq!(
            fixture.witnesses(&statement, invalid_forward),
            Err(SourceStatementWitnessError::InvalidWitness {
                witness: SourceStatementWitnessId::new(0)
            })
        );
        let mut invalid_reverse = fixture.witness_input();
        invalid_reverse.names[0].witness = SourceStatementWitnessId::new(1);
        assert_eq!(
            fixture.witnesses(&statement, invalid_reverse),
            Err(SourceStatementWitnessError::InvalidName {
                name: SourceStatementWitnessNameId::new(0)
            })
        );
        let mut invalid_name = fixture.witness_input();
        invalid_name.names[0].spelling = "z".to_owned();
        assert_eq!(
            fixture.witnesses(&statement, invalid_name),
            Err(SourceStatementWitnessError::InvalidName {
                name: SourceStatementWitnessNameId::new(0)
            })
        );
        let mut aggregate_precedence = fixture.witness_input();
        aggregate_precedence.names.clear();
        aggregate_precedence.witnesses[0].kind = SourceStatementWitnessKind::Unnamed;
        assert_eq!(
            fixture.witnesses(&statement, aggregate_precedence),
            Err(SourceStatementWitnessError::InvalidAggregate)
        );
        let mut witness_precedence = fixture.witness_input();
        witness_precedence.witnesses[0].kind = SourceStatementWitnessKind::Unnamed;
        witness_precedence.names[0].spelling = "z".to_owned();
        assert_eq!(
            fixture.witnesses(&statement, witness_precedence),
            Err(SourceStatementWitnessError::InvalidWitness {
                witness: SourceStatementWitnessId::new(0)
            })
        );
        let mut precedence = fixture.witness_input();
        precedence.source_id = B3Fixture::new(47).source;
        precedence.names.clear();
        assert_eq!(
            fixture.witnesses(&statement, precedence),
            Err(SourceStatementWitnessError::DependencyMismatch)
        );
        for mutation in 0..16 {
            let mut input = fixture.witness_input();
            let row = &mut input.witnesses[0];
            match mutation {
                0 => row.owner = SourceTheoremOwnerId::new(1),
                1 => row.binding_context = BindingContextId::new(0),
                2 => {
                    row.term =
                        SourceStatementWitnessTermTarget::Primary(SourcePrimaryTermId::new(1))
                }
                3 => row.take_site = node(36),
                4 => row.take_range.start += 1,
                5 => row.take_range.end -= 1,
                6 => row.site = node(37),
                7 => row.source_range.start += 1,
                8 => row.source_range.end -= 1,
                9 => row.source_ordinal = 2,
                10 => row.ordinal = 1,
                11 => row.spelling.push('x'),
                12 => row.kind = SourceStatementWitnessKind::Unnamed,
                13 => row.recovery = SourceStatementRecovery::Degraded,
                14 => row.name = None,
                15 => row.name = Some(SourceStatementWitnessNameId::new(1)),
                _ => unreachable!(),
            }
            assert_eq!(
                fixture.witnesses(&statement, input),
                Err(SourceStatementWitnessError::InvalidWitness {
                    witness: SourceStatementWitnessId::new(0)
                }),
                "witness field {mutation}"
            );
        }
        for mutation in 0..6 {
            let mut input = fixture.witness_input();
            let row = &mut input.names[0];
            match mutation {
                0 => row.witness = SourceStatementWitnessId::new(1),
                1 => row.site = node(14),
                2 => row.source_range.start += 1,
                3 => row.source_range.end -= 1,
                4 => row.spelling.push('x'),
                5 => row.recovery = SourceStatementRecovery::Degraded,
                _ => unreachable!(),
            }
            assert_eq!(
                fixture.witnesses(&statement, input),
                Err(SourceStatementWitnessError::InvalidName {
                    name: SourceStatementWitnessNameId::new(0)
                }),
                "name field {mutation}"
            );
        }
        let mut stale_statement = baseline.clone();
        stale_statement.statement_fingerprint.push('x');
        assert_eq!(
            stale_statement.validate_installation(
                fixture.source,
                &fixture.module,
                &statement,
                &fixture.primary,
                &fixture.arena,
            ),
            Err(SourceStatementWitnessError::DependencyMismatch)
        );
        let mut stale_primary = baseline.clone();
        stale_primary.primary_term_fingerprint.push('x');
        assert_eq!(
            stale_primary.validate_installation(
                fixture.source,
                &fixture.module,
                &statement,
                &fixture.primary,
                &fixture.arena,
            ),
            Err(SourceStatementWitnessError::DependencyMismatch)
        );
        let b3 = B3Fixture::new(47);
        assert_eq!(
            baseline.validate_installation(
                fixture.source,
                &fixture.module,
                &b3.statement(),
                &fixture.primary,
                &fixture.arena,
            ),
            Err(SourceStatementWitnessError::DependencyMismatch)
        );
        assert_eq!(
            baseline.validate_installation(
                fixture.source,
                &fixture.module,
                &statement,
                &b3.primary,
                &fixture.arena,
            ),
            Err(SourceStatementWitnessError::DependencyMismatch)
        );
        for mutation in 0..6 {
            let mut input = fixture.statement_input();
            let row = &mut input.owners[0];
            match mutation {
                0 => row.symbol = b3.symbol.clone(),
                1 => row.site = node(45),
                2 => row.source_range.start += 1,
                3 => row.source_range.end -= 1,
                4 => row.spelling.push('x'),
                5 => row.recovery = SourceStatementRecovery::Degraded,
                _ => unreachable!(),
            }
            assert!(
                SourceStatementProducer::build(
                    input,
                    &fixture.symbols,
                    &fixture.bindings,
                    &fixture.primary,
                    &fixture.atomic,
                    &fixture.arena,
                )
                .is_err(),
                "owner provenance field {mutation}"
            );
        }
        let mut contributions = fixture.symbols.contributions().clone();
        let foreign_contribution = contributions.insert(
            fixture.module.clone(),
            ContributionKind::LocalSource {
                source_id: fixture.source,
            },
            SourceAnchor::Range(range(fixture.source, 0, 18)),
        );
        let foreign_symbols = SymbolEnv::new(
            fixture.module.clone(),
            SymbolEnvIndexes {
                imports: fixture.symbols.imports().clone(),
                exports: fixture.symbols.exports().clone(),
                symbols: fixture.symbols.symbols().clone(),
                labels: fixture.symbols.labels().clone(),
                definitions: fixture.symbols.definitions().clone(),
                overloads: fixture.symbols.overloads().clone(),
                registrations: fixture.symbols.registrations().clone(),
                lexical_summaries: fixture.symbols.lexical_summaries().clone(),
                namespace_graph: fixture.symbols.namespace_graph().clone(),
                declaration_dependencies: fixture.symbols.declaration_dependencies().clone(),
                contributions,
                module_summaries: fixture.symbols.module_summaries().clone(),
            },
        );
        let mut foreign_contribution_input = fixture.statement_input();
        foreign_contribution_input.owners[0].contribution = foreign_contribution;
        assert!(
            SourceStatementProducer::build(
                foreign_contribution_input,
                &foreign_symbols,
                &fixture.bindings,
                &fixture.primary,
                &fixture.atomic,
                &fixture.arena,
            )
            .is_err(),
            "owner contribution provenance"
        );
        for aggregate_mutation in 0..10 {
            let mut input = fixture.statement_input();
            match aggregate_mutation {
                0 => input.owners.clear(),
                1 => input.owners.push(input.owners[0].clone()),
                2 => {
                    input.statements.pop();
                }
                3 => input.statements.push(input.statements[1].clone()),
                4 => {
                    input.contexts.pop();
                }
                5 => input.contexts.push(input.contexts[1].clone()),
                6 => {
                    input.input_facts.pop();
                }
                7 => input.input_facts.push(input.input_facts[1].clone()),
                8 => {
                    input.candidate_facts.pop();
                }
                9 => input.candidate_facts.push(input.candidate_facts[1].clone()),
                _ => unreachable!(),
            }
            assert_eq!(
                SourceStatementProducer::build(
                    input,
                    &fixture.symbols,
                    &fixture.bindings,
                    &fixture.primary,
                    &fixture.atomic,
                    &fixture.arena,
                ),
                Err(SourceStatementError::InvalidAggregate),
                "base aggregate {aggregate_mutation}"
            );
        }
        for index in 0..2 {
            let other = 1 - index;
            for mutation in 0..9 {
                let mut input = fixture.statement_input();
                let row = &mut input.statements[index];
                match mutation {
                    0 => row.owner = SourceTheoremOwnerId::new(1),
                    1 => row.context = SourceStatementContextId::new(other),
                    2 => {
                        row.formula =
                            SourceStatementFormulaTarget::Atomic(SourceAtomicFormulaId::new(other))
                    }
                    3 => row.site = node([45, 47][index]),
                    4 => row.source_range.start += 1,
                    5 => row.source_range.end -= 1,
                    6 => row.source_ordinal = 1,
                    7 => row.spelling.push('x'),
                    8 => row.recovery = SourceStatementRecovery::Degraded,
                    _ => unreachable!(),
                }
                assert!(
                    SourceStatementProducer::build(
                        input,
                        &fixture.symbols,
                        &fixture.bindings,
                        &fixture.primary,
                        &fixture.atomic,
                        &fixture.arena,
                    )
                    .is_err(),
                    "base statement {index} field {mutation}"
                );
            }
            for mutation in 0..5 {
                let mut input = fixture.statement_input();
                let row = &mut input.contexts[index];
                match mutation {
                    0 => row.statement = SourceStatementId::new(other),
                    1 => row.binding_context = BindingContextId::new(other),
                    2 => row.source_range.start += 1,
                    3 => row.source_range.end -= 1,
                    4 => row.visible_bindings.clear(),
                    _ => unreachable!(),
                }
                assert!(
                    SourceStatementProducer::build(
                        input,
                        &fixture.symbols,
                        &fixture.bindings,
                        &fixture.primary,
                        &fixture.atomic,
                        &fixture.arena,
                    )
                    .is_err(),
                    "base context {index} field {mutation}"
                );
            }
            for mutation in 0..5 {
                let mut input = fixture.statement_input();
                let row = &mut input.input_facts[index];
                match mutation {
                    0 => row.statement = SourceStatementId::new(other),
                    1 => row.context = SourceStatementContextId::new(other),
                    2 => row.ordinal = 1,
                    3 => row.binding = BindingId::new(1),
                    4 => row.uses.swap(0, 1),
                    _ => unreachable!(),
                }
                assert!(
                    SourceStatementProducer::build(
                        input,
                        &fixture.symbols,
                        &fixture.bindings,
                        &fixture.primary,
                        &fixture.atomic,
                        &fixture.arena,
                    )
                    .is_err(),
                    "base input fact {index} field {mutation}"
                );
            }
            for mutation in 0..4 {
                let mut input = fixture.statement_input();
                let row = &mut input.candidate_facts[index];
                match mutation {
                    0 => row.statement = SourceStatementId::new(other),
                    1 => row.context = SourceStatementContextId::new(other),
                    2 => row.ordinal = 1,
                    3 => {
                        row.formula =
                            SourceStatementFormulaTarget::Atomic(SourceAtomicFormulaId::new(other))
                    }
                    _ => unreachable!(),
                }
                assert!(
                    SourceStatementProducer::build(
                        input,
                        &fixture.symbols,
                        &fixture.bindings,
                        &fixture.primary,
                        &fixture.atomic,
                        &fixture.arena,
                    )
                    .is_err(),
                    "base candidate {index} field {mutation}"
                );
            }
        }
        for index in 0..51 {
            for mutation in 0..5 {
                let arena = mutate_arena(&fixture.arena, |id, row| {
                    if id.index() == index {
                        match mutation {
                            0 => {
                                let SourceAnchor::Range(mut source_range) = row.anchor.clone()
                                else {
                                    unreachable!("Task258B3N range")
                                };
                                source_range.end += 1;
                                row.anchor = SourceAnchor::Range(source_range);
                            }
                            1 => row.recovery = NodeRecoveryState::Recovered,
                            2 => row.recovery = NodeRecoveryState::Degraded,
                            3 => row.kind = "source.task258b3n.mutated".into(),
                            4 if row.children.len() > 1 => row.children.swap(0, 1),
                            4 if row.children.len() == 1 => row.children.clear(),
                            4 => row.children.push(TypedNodeId::new(usize::from(index == 0))),
                            _ => unreachable!(),
                        }
                    }
                });
                assert_eq!(
                    baseline.validate_installation(
                        fixture.source,
                        &fixture.module,
                        &statement,
                        &fixture.primary,
                        &arena,
                    ),
                    Err(SourceStatementWitnessError::DependencyMismatch),
                    "node {index} mutation {mutation}"
                );
            }
        }
        assert_eq!(
            fixture
                .witnesses(&statement, fixture.witness_input())
                .expect("replay")
                .debug_text(),
            baseline_debug
        );
    }

    #[test]
    fn task258b3n_paired_typed_ownership_hybrids_and_orders_are_atomic() {
        let fixture = B3NFixture::new(48);
        let statement = fixture.statement();
        let witnesses = fixture
            .witnesses(&statement, fixture.witness_input())
            .expect("witnesses");
        let base = fixture.empty_typed();
        let base_debug = base.debug_text();
        let typed = base
            .clone()
            .with_source_statement_witnesses(statement.clone(), witnesses.clone())
            .expect("paired install");
        assert_eq!(typed.source_statement(), Some(&statement));
        assert_eq!(typed.source_statement_witnesses(), Some(&witnesses));
        assert!(typed.source_statement_references().is_none());
        assert_eq!(base.debug_text(), base_debug);
        assert_eq!(
            typed
                .clone()
                .with_source_statement_witnesses(statement.clone(), witnesses.clone()),
            Err(TypedAstError::InvalidSourceStatement)
        );
        assert_eq!(
            base.clone().with_source_statement(statement.clone()),
            Err(TypedAstError::InvalidSourceStatement)
        );
        let task258a = Fixture::new(48);
        let task258a_statement = task258a
            .build(task258a.input())
            .expect("Task258A statement");
        let task258a_installed = TypedAst::try_new(TypedAstParts {
            source_id: task258a.source,
            module_id: task258a.module.clone(),
            resolved_root: None,
            source_context: None,
            source_type: None,
            source_attribute: None,
            nodes: task258a.arena.clone(),
            contexts: LocalTypeContextTable::new(),
            types: TypeTable::new(),
            facts: TypeFactTable::new(),
            coercions: CoercionTable::new(),
            initial_obligations: InitialObligationTable::new(),
            diagnostics: TypeDiagnosticTable::new(),
        })
        .expect("Task258A empty")
        .with_source_term(task258a.primary.clone())
        .expect("Task258A primary")
        .with_source_atomic_formula(task258a.atomic.clone())
        .expect("Task258A atomic")
        .with_source_statement(task258a_statement.clone())
        .expect("Task258A install");
        let task258a_debug = task258a_installed.debug_text();
        assert_eq!(
            task258a_installed
                .clone()
                .with_source_statement_witnesses(statement.clone(), witnesses.clone()),
            Err(TypedAstError::InvalidSourceStatement)
        );
        assert_eq!(task258a_installed.debug_text(), task258a_debug);
        assert_eq!(
            typed.clone().with_source_statement(task258a_statement),
            Err(TypedAstError::InvalidSourceStatement)
        );
        let b2 = B2Fixture::new(48);
        let b2_statement = b2.build(b2.input()).expect("B2 statement");
        let b2_installed = b2
            .empty_typed()
            .with_source_statement(b2_statement.clone())
            .expect("B2 install");
        let b2_debug = b2_installed.debug_text();
        assert_eq!(
            b2_installed
                .clone()
                .with_source_statement_witnesses(statement.clone(), witnesses.clone()),
            Err(TypedAstError::InvalidSourceStatement)
        );
        assert_eq!(b2_installed.debug_text(), b2_debug);
        assert_eq!(
            typed.clone().with_source_statement(b2_statement),
            Err(TypedAstError::InvalidSourceStatement)
        );
        let b3 = B3Fixture::new(49);
        let b3_statement = b3.statement();
        let b3_witnesses = b3
            .witnesses(&b3_statement, b3.witness_input())
            .expect("B3 witnesses");
        let b3_installed = b3
            .empty_typed()
            .with_source_statement_witnesses(b3_statement.clone(), b3_witnesses.clone())
            .expect("B3 paired install");
        let b3_debug = b3_installed.debug_text();
        assert_eq!(
            base.clone()
                .with_source_statement_witnesses(b3_statement.clone(), b3_witnesses.clone()),
            Err(TypedAstError::InvalidSourceStatement)
        );
        assert_eq!(
            b3_installed
                .clone()
                .with_source_statement_witnesses(statement.clone(), witnesses.clone()),
            Err(TypedAstError::InvalidSourceStatement)
        );
        assert_eq!(b3_installed.debug_text(), b3_debug);
        assert_eq!(
            typed
                .clone()
                .with_source_statement_witnesses(b3_statement, b3_witnesses),
            Err(TypedAstError::InvalidSourceStatement)
        );
        let b1 = B1Fixture::new(49);
        let b1_references = b1.references(b1.reference_input()).expect("B1 references");
        let b1_installed = b1
            .empty_typed()
            .with_source_statement_references(b1.statement.clone(), b1_references.clone())
            .expect("B1 paired install");
        let b1_debug = b1_installed.debug_text();
        assert_eq!(
            typed
                .clone()
                .with_source_statement_references(b1.statement.clone(), b1_references,),
            Err(TypedAstError::InvalidSourceStatement)
        );
        assert_eq!(
            b1_installed
                .clone()
                .with_source_statement_witnesses(statement.clone(), witnesses.clone()),
            Err(TypedAstError::InvalidSourceStatement)
        );
        assert_eq!(b1_installed.debug_text(), b1_debug);
        let task248 = crate::source_context::tests::task_248_occupied_typed_ast(
            fixture.source,
            fixture.module.clone(),
        );
        let task248_debug = task248.debug_text();
        assert_eq!(
            task248
                .clone()
                .with_source_statement_witnesses(statement.clone(), witnesses.clone()),
            Err(TypedAstError::InvalidSourceStatement)
        );
        assert_eq!(task248.debug_text(), task248_debug);
        assert_eq!(
            typed.clone().with_source_context_for_test(
                task248
                    .source_context()
                    .expect("Task248 source context")
                    .clone(),
            ),
            Err(TypedAstError::InvalidSourceContext)
        );
        let task257a = crate::source_composite_formula::tests::task_257a_installed_typed_ast();
        let task257b = crate::source_formula_composition::tests::task_257b_installed_typed_ast();
        let task257c2 = crate::source_formula_composition::tests::task_257c2_installed_typed_ast();
        let task257c3 = crate::source_formula_composition::tests::task_257c3_installed_typed_ast();
        for (family, occupied) in [
            ("Task257A", &task257a),
            ("Task257B", &task257b),
            ("Task257C2", &task257c2),
            ("Task257C3", &task257c3),
        ] {
            let debug = occupied.debug_text();
            assert_eq!(
                occupied
                    .clone()
                    .with_source_statement_witnesses(statement.clone(), witnesses.clone()),
                Err(TypedAstError::InvalidSourceStatement),
                "{family}"
            );
            assert_eq!(occupied.debug_text(), debug, "{family} rollback");
        }
        assert_eq!(
            typed.clone().with_source_composite_formula(
                task257a
                    .source_composite_formula()
                    .expect("Task257A composite")
                    .clone(),
            ),
            Err(TypedAstError::InvalidSourceCompositeFormula)
        );
        assert_eq!(
            typed.clone().with_source_formula_composition(
                task257b
                    .source_composite_formula()
                    .expect("Task257B composite")
                    .clone(),
                task257b
                    .source_formula_composition()
                    .expect("Task257B composition")
                    .clone(),
            ),
            Err(TypedAstError::InvalidSourceFormulaComposition)
        );
        assert_eq!(
            typed.clone().with_source_condition_formula_composition(
                task257c2
                    .source_condition_formula_composition()
                    .expect("Task257C2 composition")
                    .clone(),
            ),
            Err(TypedAstError::InvalidSourceConditionFormulaComposition)
        );
        assert_eq!(
            typed.clone().with_source_predicate_chain_composition(
                task257c3
                    .source_predicate_chain_composition()
                    .expect("Task257C3 composition")
                    .clone(),
            ),
            Err(TypedAstError::InvalidSourcePredicateChainComposition)
        );
        assert_eq!(typed.source_statement(), Some(&statement));
        assert_eq!(typed.source_statement_witnesses(), Some(&witnesses));
        assert_eq!(
            base.with_source_statement_witnesses(statement, witnesses)
                .expect("replay")
                .debug_text(),
            typed.debug_text()
        );
    }

    #[test]
    fn task258b3n_final_clone_revalidation_and_semantic_deferrals_are_stable() {
        let fixture = B3NFixture::new(50);
        let statement = fixture.statement();
        let witnesses = fixture
            .witnesses(&statement, fixture.witness_input())
            .expect("witnesses");
        let typed = fixture
            .empty_typed()
            .with_source_statement_witnesses(statement.clone(), witnesses.clone())
            .expect("typed");
        let resolved = assemble_empty_resolved(&typed).expect("resolved");
        assert_eq!(resolved.source_statement(), Some(&statement));
        assert_eq!(resolved.source_statement_witnesses(), Some(&witnesses));
        assert!(resolved.source_statement_references().is_none());
        assert_eq!(resolved.clone().debug_text(), resolved.debug_text());
        assert!(resolved.expr_metadata().is_empty());
        assert!(resolved.collection_candidates().is_empty());
        assert!(resolved.expanded_candidates().is_empty());
        assert!(resolved.template_expansions().is_empty());
        assert!(resolved.viable_candidates().is_empty());
        assert!(resolved.viability_decisions().is_empty());
        assert!(resolved.specificity_graphs().is_empty());
        assert!(resolved.resolved_overloads().is_empty());
        assert!(resolved.inserted_coercions().is_empty());
        assert!(resolved.cluster_facts().is_empty());
        assert!(resolved.diagnostics().is_empty());
        assert!(resolved.checked_formulas().is_empty());
        assert!(resolved.statement_semantics().is_empty());
        assert!(resolved.checked_proofs().is_empty());
        assert!(resolved.checked_proof_nodes().is_empty());
        assert!(resolved.checked_terminal_goals().is_empty());

        let mut orphan = fixture.empty_typed();
        orphan.inject_source_statement_witnesses_for_test(witnesses.clone());
        assert_eq!(
            assemble_empty_resolved(&orphan),
            Err(ResolvedTypedAstError::InvalidSourceStatement)
        );
        let mut standalone = fixture.empty_typed();
        standalone.inject_source_statement_for_test(statement.clone());
        assert_eq!(
            assemble_empty_resolved(&standalone),
            Err(ResolvedTypedAstError::InvalidSourceStatement)
        );
        let mut stale = witnesses.clone();
        stale.statement_fingerprint.push('x');
        let mut stale_pair = fixture.empty_typed();
        stale_pair.inject_source_statement_witness_bundle_for_test(statement.clone(), stale);
        assert_eq!(
            assemble_empty_resolved(&stale_pair),
            Err(ResolvedTypedAstError::InvalidSourceStatement)
        );
        let mut stale_primary = witnesses.clone();
        stale_primary.primary_term_fingerprint.push('x');
        let mut stale_primary_pair = fixture.empty_typed();
        stale_primary_pair
            .inject_source_statement_witness_bundle_for_test(statement.clone(), stale_primary);
        assert_eq!(
            assemble_empty_resolved(&stale_primary_pair),
            Err(ResolvedTypedAstError::InvalidSourceStatement)
        );
        let b1 = B1Fixture::new(50);
        let mut reference_hybrid = fixture.empty_typed();
        reference_hybrid.inject_source_statement_bundle_for_test(
            statement.clone(),
            b1.references(b1.reference_input()).expect("B1 references"),
        );
        reference_hybrid.inject_source_statement_witnesses_for_test(witnesses.clone());
        assert_eq!(
            assemble_empty_resolved(&reference_hybrid),
            Err(ResolvedTypedAstError::InvalidSourceStatement)
        );
        for table in [
            StatementTransportTableForTest::Context,
            StatementTransportTableForTest::Type,
            StatementTransportTableForTest::Fact,
            StatementTransportTableForTest::Coercion,
            StatementTransportTableForTest::InitialObligation,
            StatementTransportTableForTest::Diagnostic,
        ] {
            let mut occupied = typed.clone();
            occupied.occupy_statement_transport_table_for_test(table);
            assert_eq!(
                assemble_empty_resolved(&occupied),
                Err(ResolvedTypedAstError::InvalidSourceStatement),
                "{table:?}"
            );
        }
        let term_formula = TermFormulaChecker::default().infer(
            &fixture.symbols,
            &fixture.bindings,
            Vec::<crate::type_checker::TermInput>::new(),
            Vec::<crate::type_checker::FormulaInput>::new(),
        );
        assert_eq!(
            assemble_resolved_with_statement_semantics(
                &typed,
                statement.checked_owner(),
                &fixture.bindings,
                &term_formula,
            ),
            Err(ResolvedTypedAstError::InvalidSourceStatement)
        );
        let mut cluster_facts = ClusterFactTable::new();
        cluster_facts.insert(ClusterFactDraft {
            fingerprint: ClusterFactFingerprint::new("task258b3n-coexistence"),
            source_type: ClusterTypeFingerprint::new("set"),
            attribute: ClusterAttributeFingerprint::new("inhabited"),
            generated_type: ClusterTypeFingerprint::new("set"),
            provenance: ClusterFactProvenance::Input,
            source_range: range(fixture.source, 81, 82),
        });
        assert_eq!(
            assemble_resolved(&typed, &cluster_facts, Vec::new(), None),
            Err(ResolvedTypedAstError::InvalidSourceStatement)
        );
        assert_eq!(
            assemble_resolved(
                &typed,
                &ClusterFactTable::new(),
                vec![ResolvedNodeKindHint {
                    typed_node: TypedNodeId::new(13),
                    kind: ResolvedNodeKindHintKind::SourcePreserved {
                        role: SourceNodeRole::new(
                            "source.statement-witness-name.semantic-coexistence",
                        ),
                    },
                }],
                None,
            ),
            Err(ResolvedTypedAstError::InvalidSourceStatement)
        );
        assert_eq!(
            assemble_resolved(
                &typed,
                &ClusterFactTable::new(),
                Vec::new(),
                Some(StatementProofInputs {
                    owner: statement.checked_owner(),
                    rows: Vec::new(),
                }),
            ),
            Err(ResolvedTypedAstError::InvalidSourceStatement)
        );
        assert_eq!(
            assemble_resolved_with_expressions(
                &typed,
                vec![ExpressionMetadataInput {
                    expr: ExprId::new("task258b3n-semantic-coexistence"),
                    typed_site: node(34),
                    local_context: None,
                    cluster_facts: Vec::new(),
                }],
            ),
            Err(ResolvedTypedAstError::InvalidSourceStatement)
        );
        let empty_collection = OverloadCollectionOutput::collect(
            Vec::<OverloadSiteInput>::new(),
            Vec::<OverloadCandidateInput>::new(),
        );
        let empty_expansion = TemplateExpansionOutput::expand(&empty_collection);
        let empty_viability = CandidateViabilityOutput::filter(
            &empty_expansion,
            Vec::<CandidateViabilityInput>::new(),
        );
        let empty_specificity = SpecificityGraphOutput::build(
            &empty_viability,
            Vec::<SpecificityComparisonInput>::new(),
        );
        let empty_selection = OverloadSelectionOutput::resolve(
            &empty_specificity,
            Vec::<OverloadSiteResolutionInput>::new(),
        );
        let occupied_collection = OverloadCollectionOutput::collect(
            vec![OverloadSiteInput {
                key: OverloadSiteKey::new("task258b3n-overload"),
                owner: TypedSiteRef::Role {
                    node: TypedNodeId::new(34),
                    role: TypeRole::new("task258b3n-overload-owner"),
                },
                source_range: range(fixture.source, 85, 86),
                kind: OverloadSiteKind::FunctorApplication,
                name: OverloadNameKey::new("task258b3n-overload"),
                arguments: Vec::new(),
                expected: None,
                source_qua: Vec::<SourceQuaView>::new(),
                recovery: OverloadSiteRecovery::Normal,
            }],
            vec![OverloadCandidateInput {
                site: OverloadSiteKey::new("task258b3n-overload"),
                symbol: fixture.symbol.clone(),
                ordinary_root: fixture.symbol.clone(),
                declaration_kind: CandidateDeclarationKind::Functor,
                parameters: Vec::new(),
                result: None,
                origin: CandidateOrigin::Ordinary,
                template: None,
                coherence: None,
                provenance: CandidateProvenance {
                    stable_key: CandidateProvenanceKey::new("task258b3n-overload"),
                    source_range: Some(range(fixture.source, 85, 86)),
                    scope: CandidateScope::Local,
                    declaration_order: 0,
                },
            }],
        );
        let occupied_expansion = TemplateExpansionOutput::expand(&occupied_collection);
        let occupied_viability = CandidateViabilityOutput::filter(
            &occupied_expansion,
            vec![CandidateViabilityInput {
                candidate: OverloadCandidateId::new(0),
                arguments: Vec::new(),
            }],
        );
        let occupied_specificity = SpecificityGraphOutput::build(
            &occupied_viability,
            Vec::<SpecificityComparisonInput>::new(),
        );
        let occupied_selection = OverloadSelectionOutput::resolve(
            &occupied_specificity,
            vec![OverloadSiteResolutionInput {
                site: OverloadSiteId::new(0),
                refinements: vec![OverloadCandidateId::new(0)],
                refinement_join: RefinementJoinPayload {
                    status: RefinementJoinStatus::Compatible,
                    exposed_result: None,
                },
                inserted_views: Vec::new(),
            }],
        );
        assert_eq!(occupied_collection.candidates().len(), 1);
        assert_eq!(occupied_expansion.candidates().len(), 1);
        assert_eq!(occupied_viability.candidates().len(), 1);
        assert_eq!(occupied_specificity.candidates().len(), 1);
        assert_eq!(occupied_selection.results().len(), 1);
        let cluster_facts = ClusterFactTable::new();
        for (label, collection, expansion, viability, specificity, selection) in [
            (
                "collection",
                &occupied_collection,
                &empty_expansion,
                &empty_viability,
                &empty_specificity,
                &empty_selection,
            ),
            (
                "expansion",
                &empty_collection,
                &occupied_expansion,
                &empty_viability,
                &empty_specificity,
                &empty_selection,
            ),
            (
                "viability",
                &empty_collection,
                &empty_expansion,
                &occupied_viability,
                &empty_specificity,
                &empty_selection,
            ),
            (
                "specificity",
                &empty_collection,
                &empty_expansion,
                &empty_viability,
                &occupied_specificity,
                &empty_selection,
            ),
            (
                "selection",
                &empty_collection,
                &empty_expansion,
                &empty_viability,
                &empty_specificity,
                &occupied_selection,
            ),
        ] {
            assert_eq!(
                ResolvedTypedAst::assemble(ResolvedTypedAstInputs {
                    typed_ast: &typed,
                    cluster_facts: &cluster_facts,
                    overload_collection: collection,
                    template_expansion: expansion,
                    viability,
                    specificity,
                    overload_selection: selection,
                    expressions: Vec::new(),
                    node_hints: Vec::new(),
                    statement_semantics: None,
                    statement_proofs: None,
                }),
                Err(ResolvedTypedAstError::InvalidSourceStatement),
                "{label}"
            );
        }
        assert_eq!(
            assemble_empty_resolved(&typed)
                .expect("replay")
                .debug_text(),
            resolved.debug_text()
        );
    }

    #[test]
    fn task258b3m1_exact_multi_witness_api_debug_and_compatibility_are_stable() {
        let fixture = B3M1Fixture::new(51);
        let statement = fixture.statement();
        assert!(statement.is_task_258b3m1_profile());
        assert!(!statement.is_task_258b3_profile());
        assert!(!statement.is_task_258b3n_profile());
        assert_eq!(
            (
                fixture.bindings.contexts().len(),
                fixture.bindings.bindings().len(),
                fixture.bindings.diagnostics().len(),
                fixture.primary.terms().len(),
                fixture.primary.references().len(),
                fixture.primary.numeric_type_requests().len(),
                fixture.atomic.formulas().len(),
                fixture.atomic.edges().len(),
                fixture.atomic.requests().len(),
            ),
            (2, 1, 0, 6, 6, 0, 2, 4, 4)
        );
        let proof = fixture
            .bindings
            .contexts()
            .get(BindingContextId::new(1))
            .expect("proof context");
        assert_eq!(
            (
                &proof.owner,
                proof.parent,
                proof.layer,
                proof
                    .lexical_scope
                    .as_ref()
                    .map(|scope| scope.path().to_vec()),
                proof.bindings.as_slice(),
                proof.visible_bindings.as_slice(),
                proof.recovery,
            ),
            (
                &BindingContextOwner::SourceStatement {
                    source_range: range(fixture.source, 71, 111),
                },
                Some(BindingContextId::new(0)),
                BindingContextLayer::Proof,
                Some(vec![0]),
                &[][..],
                &[BindingId::new(0)][..],
                BindingContextRecovery::Normal,
            )
        );
        for (index, ((_, term), (_, reference))) in fixture
            .primary
            .terms()
            .iter()
            .zip(fixture.primary.references().iter())
            .enumerate()
        {
            assert_eq!(
                (
                    term.site().node().index(),
                    term.source_range(),
                    term.source_ordinal(),
                    term.context().index(),
                    term.spelling(),
                    term.kind(),
                    term.role(),
                    term.recovery(),
                    term.parent(),
                ),
                (
                    [30, 32, 36, 39, 43, 45][index],
                    range(
                        fixture.source,
                        [65, 69, 88, 91, 101, 105][index],
                        [66, 70, 89, 92, 102, 106][index],
                    ),
                    index,
                    [0, 0, 1, 1, 1, 1][index],
                    "x",
                    SourcePrimaryTermKind::VariableReference,
                    SourcePrimaryTermRole::Value,
                    SourcePrimaryTermRecovery::Normal,
                    None,
                )
            );
            assert_eq!(
                (
                    reference.term().index(),
                    reference.binding().index(),
                    reference.role(),
                    reference.use_ordinal(),
                    reference.lexical_scope().map(|scope| scope.path().to_vec()),
                ),
                (
                    index,
                    0,
                    SourcePrimaryTermReferenceRole::Variable,
                    1,
                    if index < 2 { None } else { Some(vec![0]) },
                )
            );
        }
        assert_eq!(
            fixture
                .atomic
                .edges()
                .iter()
                .map(|(_, edge)| match edge.target() {
                    SourceAtomicTermTarget::Primary(term) => term.index(),
                    _ => usize::MAX,
                })
                .collect::<Vec<_>>(),
            [0, 1, 4, 5]
        );
        assert_eq!(
            (
                statement.owners().len(),
                statement.statements().len(),
                statement.contexts().len(),
                statement.input_facts().len(),
                statement.candidate_facts().len(),
            ),
            (1, 2, 2, 2, 2)
        );
        let owner = statement
            .owners()
            .get(SourceTheoremOwnerId::new(0))
            .expect("owner");
        assert_eq!(
            (
                owner.symbol(),
                owner.contribution(),
                owner.site().node().index(),
                owner.source_range(),
                owner.spelling(),
                owner.role(),
                owner.status(),
                owner.recovery(),
            ),
            (
                &fixture.symbol,
                fixture.contribution,
                52,
                range(fixture.source, 19, 112),
                "FormulaStatementMultipleWitnessSmoke",
                SourceTheoremRole::Theorem,
                SourceTheoremStatus::Unmodified,
                SourceStatementRecovery::Normal,
            )
        );
        assert_eq!(statement.checked_owner().origin().structural_path(), [2, 1]);
        let labels = fixture
            .symbols
            .labels()
            .by_contribution(fixture.contribution);
        assert_eq!(labels.len(), 1);
        assert_eq!(
            (
                labels[0].primary_spelling(),
                labels[0].origin().structural_path(),
                labels[0].contribution(),
            ),
            (
                "FormulaStatementMultipleWitnessSmoke",
                &[2, 1][..],
                fixture.contribution,
            )
        );
        for index in 0..2 {
            let row = statement
                .statements()
                .get(SourceStatementId::new(index))
                .expect("statement");
            assert_eq!(
                (
                    row.site().node().index(),
                    row.source_range(),
                    row.source_ordinal(),
                    row.spelling(),
                    row.kind(),
                ),
                (
                    [52, 50][index],
                    range(fixture.source, [19, 96][index], [112, 107][index]),
                    [0, 2][index],
                    [
                        "theorem FormulaStatementMultipleWitnessSmoke : x = x proof take y = x , x ; thus x = x ; end ;",
                        "thus x = x ;",
                    ][index],
                    [
                        SourceStatementKind::TheoremProposition,
                        SourceStatementKind::Conclusion,
                    ][index],
                )
            );
            assert_eq!(
                statement
                    .input_facts()
                    .get(SourceStatementInputFactId::new(index))
                    .expect("fact")
                    .uses()
                    .iter()
                    .map(|id| id.index())
                    .collect::<Vec<_>>(),
                if index == 0 { vec![0, 1] } else { vec![4, 5] },
            );
        }
        let handoff = fixture
            .witnesses(&statement, fixture.witness_input())
            .expect("witness handoff");
        assert_eq!(handoff.source_id(), fixture.source);
        assert_eq!(handoff.module_id(), &fixture.module);
        assert_eq!(handoff.statement_fingerprint(), statement.debug_text());
        assert_eq!(
            handoff.primary_term_fingerprint(),
            fixture.primary.debug_text()
        );
        assert_eq!((handoff.witnesses().len(), handoff.names().len()), (2, 1));
        for index in 0..2 {
            let row = handoff
                .witnesses()
                .get(SourceStatementWitnessId::new(index))
                .expect("witness");
            assert_eq!(
                (
                    row.owner().index(),
                    row.binding_context().index(),
                    row.term(),
                    row.take_site().node().index(),
                    row.take_range(),
                    row.site().node().index(),
                    row.source_range(),
                    row.source_ordinal(),
                ),
                (
                    0,
                    1,
                    SourceStatementWitnessTermTarget::Primary(SourcePrimaryTermId::new(2 + index)),
                    42,
                    range(fixture.source, 79, 93),
                    [38, 41][index],
                    range(fixture.source, [84, 91][index], [89, 92][index]),
                    1,
                )
            );
            assert_eq!(
                (
                    row.ordinal(),
                    row.spelling(),
                    row.kind(),
                    row.recovery(),
                    row.name(),
                ),
                (
                    index,
                    ["y = x", "x"][index],
                    [
                        SourceStatementWitnessKind::Named,
                        SourceStatementWitnessKind::Unnamed,
                    ][index],
                    SourceStatementRecovery::Normal,
                    [Some(SourceStatementWitnessNameId::new(0)), None][index],
                )
            );
        }
        let name = handoff
            .names()
            .get(SourceStatementWitnessNameId::new(0))
            .expect("name");
        assert_eq!(
            (
                name.witness().index(),
                name.site().node().index(),
                name.source_range(),
                name.spelling(),
                name.recovery(),
            ),
            (
                0,
                13,
                range(fixture.source, 84, 85),
                "y",
                SourceStatementRecovery::Normal,
            )
        );
        assert_eq!(
            handoff.debug_text(),
            format!(
                "source-statement-witness-debug-v1\nmodule: pkg::statement.fixture\nstatement-fingerprint: {:?}\nprimary-term-fingerprint: {:?}\nwitness#0 owner=0 binding_context=1 term=primary#2 take_range=79..93 take_site=42 range=84..89 site=38 source_ordinal=1 ordinal=0 kind=named recovery=normal spelling=\"y = x\" name=0\nwitness#1 owner=0 binding_context=1 term=primary#3 take_range=79..93 take_site=42 range=91..92 site=41 source_ordinal=1 ordinal=1 kind=unnamed recovery=normal spelling=\"x\"\nwitness-name#0 witness=0 range=84..85 site=13 recovery=normal spelling=\"y\"\n",
                statement.debug_text(),
                fixture.primary.debug_text(),
            )
        );
        for (index, (start, end)) in TASK258B3M1_NODE_RANGES.iter().copied().enumerate() {
            let node = fixture
                .arena
                .node(TypedNodeId::new(index))
                .expect("arena node");
            assert_eq!(
                (
                    &node.anchor,
                    node.kind.as_str(),
                    node.recovery,
                    node.children
                        .iter()
                        .map(|child| child.index())
                        .collect::<Vec<_>>(),
                ),
                (
                    &SourceAnchor::Range(range(fixture.source, start, end)),
                    task258b3m1_node_kind(index),
                    NodeRecoveryState::Normal,
                    task258b3m1_node_children(index).to_vec(),
                ),
                "node {index}"
            );
        }
        let b3 = B3Fixture::new(52);
        let b3_statement = b3.statement();
        let b3_debug = b3
            .witnesses(&b3_statement, b3.witness_input())
            .expect("B3")
            .debug_text();
        assert_eq!(
            b3.witnesses(&b3_statement, b3.witness_input())
                .expect("B3 replay")
                .debug_text(),
            b3_debug
        );
        let b3n = B3NFixture::new(53);
        let b3n_statement = b3n.statement();
        let b3n_debug = b3n
            .witnesses(&b3n_statement, b3n.witness_input())
            .expect("B3N")
            .debug_text();
        assert_eq!(
            b3n.witnesses(&b3n_statement, b3n.witness_input())
                .expect("B3N replay")
                .debug_text(),
            b3n_debug
        );
    }

    #[test]
    fn task258b3m1_dependencies_rows_precedence_and_all_nodes_fail_closed() {
        let fixture = B3M1Fixture::new(54);
        let statement = fixture.statement();
        let baseline = fixture
            .witnesses(&statement, fixture.witness_input())
            .expect("baseline");
        let baseline_debug = baseline.debug_text();

        for mutation in 0..6 {
            let mut input = fixture.witness_input();
            match mutation {
                0 => {
                    input.witnesses.pop();
                }
                1 => input.witnesses.push(input.witnesses[1].clone()),
                2 => input.names.clear(),
                3 => input.names.push(input.names[0].clone()),
                4 => input.witnesses.clear(),
                5 => {
                    input.witnesses.swap(0, 1);
                }
                _ => unreachable!(),
            };
            assert_eq!(
                fixture.witnesses(&statement, input),
                if mutation == 5 {
                    Err(SourceStatementWitnessError::InvalidWitness {
                        witness: SourceStatementWitnessId::new(0),
                    })
                } else {
                    Err(SourceStatementWitnessError::InvalidAggregate)
                },
                "aggregate/dense order {mutation}"
            );
        }
        for index in 0..2 {
            for mutation in 0..16 {
                let mut input = fixture.witness_input();
                let row = &mut input.witnesses[index];
                match mutation {
                    0 => row.owner = SourceTheoremOwnerId::new(1),
                    1 => row.binding_context = BindingContextId::new(0),
                    2 => {
                        row.term = SourceStatementWitnessTermTarget::Primary(
                            SourcePrimaryTermId::new(if index == 0 { 3 } else { 2 }),
                        )
                    }
                    3 => row.take_site = node(41),
                    4 => row.take_range.start += 1,
                    5 => row.take_range.end -= 1,
                    6 => row.site = node(if index == 0 { 41 } else { 38 }),
                    7 => row.source_range.start += 1,
                    8 => row.source_range.end -= 1,
                    9 => row.source_ordinal = 2,
                    10 => row.ordinal = 1 - index,
                    11 => row.spelling.push('x'),
                    12 => {
                        row.kind = if index == 0 {
                            SourceStatementWitnessKind::Unnamed
                        } else {
                            SourceStatementWitnessKind::Named
                        }
                    }
                    13 => row.recovery = SourceStatementRecovery::Degraded,
                    14 => {
                        row.name = if index == 0 {
                            None
                        } else {
                            Some(SourceStatementWitnessNameId::new(0))
                        }
                    }
                    15 => row.name = Some(SourceStatementWitnessNameId::new(1)),
                    _ => unreachable!(),
                }
                assert_eq!(
                    fixture.witnesses(&statement, input),
                    Err(SourceStatementWitnessError::InvalidWitness {
                        witness: SourceStatementWitnessId::new(index),
                    }),
                    "witness {index} field {mutation}"
                );
            }
        }
        for mutation in 0..6 {
            let mut input = fixture.witness_input();
            let row = &mut input.names[0];
            match mutation {
                0 => row.witness = SourceStatementWitnessId::new(1),
                1 => row.site = node(14),
                2 => row.source_range.start += 1,
                3 => row.source_range.end -= 1,
                4 => row.spelling.push('x'),
                5 => row.recovery = SourceStatementRecovery::Degraded,
                _ => unreachable!(),
            }
            assert_eq!(
                fixture.witnesses(&statement, input),
                Err(SourceStatementWitnessError::InvalidName {
                    name: SourceStatementWitnessNameId::new(0),
                }),
                "name field {mutation}"
            );
        }
        let mut aggregate_first = fixture.witness_input();
        aggregate_first.names.clear();
        aggregate_first.witnesses[0].owner = SourceTheoremOwnerId::new(1);
        assert_eq!(
            fixture.witnesses(&statement, aggregate_first),
            Err(SourceStatementWitnessError::InvalidAggregate)
        );
        let mut witness_zero_first = fixture.witness_input();
        witness_zero_first.witnesses[0].owner = SourceTheoremOwnerId::new(1);
        witness_zero_first.witnesses[1].owner = SourceTheoremOwnerId::new(1);
        witness_zero_first.names[0].spelling.push('x');
        assert_eq!(
            fixture.witnesses(&statement, witness_zero_first),
            Err(SourceStatementWitnessError::InvalidWitness {
                witness: SourceStatementWitnessId::new(0),
            })
        );
        let mut witness_one_first = fixture.witness_input();
        witness_one_first.witnesses[1].owner = SourceTheoremOwnerId::new(1);
        witness_one_first.names[0].spelling.push('x');
        assert_eq!(
            fixture.witnesses(&statement, witness_one_first),
            Err(SourceStatementWitnessError::InvalidWitness {
                witness: SourceStatementWitnessId::new(1),
            })
        );
        let mut name_last = fixture.witness_input();
        name_last.names[0].spelling.push('x');
        assert_eq!(
            fixture.witnesses(&statement, name_last),
            Err(SourceStatementWitnessError::InvalidName {
                name: SourceStatementWitnessNameId::new(0),
            })
        );
        let mut dependency_first = fixture.witness_input();
        dependency_first.source_id = B3M1Fixture::new(55).source;
        dependency_first.names.clear();
        assert_eq!(
            fixture.witnesses(&statement, dependency_first),
            Err(SourceStatementWitnessError::DependencyMismatch)
        );
        let mut stale_statement = baseline.clone();
        stale_statement.statement_fingerprint.push('x');
        assert_eq!(
            stale_statement.validate_installation(
                fixture.source,
                &fixture.module,
                &statement,
                &fixture.primary,
                &fixture.arena,
            ),
            Err(SourceStatementWitnessError::DependencyMismatch)
        );
        let mut stale_primary = baseline.clone();
        stale_primary.primary_term_fingerprint.push('x');
        assert_eq!(
            stale_primary.validate_installation(
                fixture.source,
                &fixture.module,
                &statement,
                &fixture.primary,
                &fixture.arena,
            ),
            Err(SourceStatementWitnessError::DependencyMismatch)
        );
        let b3n = B3NFixture::new(56);
        assert_eq!(
            baseline.validate_installation(
                fixture.source,
                &fixture.module,
                &b3n.statement(),
                &fixture.primary,
                &fixture.arena,
            ),
            Err(SourceStatementWitnessError::DependencyMismatch)
        );
        assert_eq!(
            baseline.validate_installation(
                fixture.source,
                &fixture.module,
                &statement,
                &b3n.primary,
                &fixture.arena,
            ),
            Err(SourceStatementWitnessError::DependencyMismatch)
        );
        for index in 0..56 {
            for mutation in 0..5 {
                let arena = mutate_arena(&fixture.arena, |id, row| {
                    if id.index() == index {
                        match mutation {
                            0 => {
                                let SourceAnchor::Range(mut source_range) = row.anchor.clone()
                                else {
                                    unreachable!("Task258B3M1 range")
                                };
                                source_range.end += 1;
                                row.anchor = SourceAnchor::Range(source_range);
                            }
                            1 => row.recovery = NodeRecoveryState::Recovered,
                            2 => row.recovery = NodeRecoveryState::Degraded,
                            3 => row.kind = "source.task258b3m1.mutated".into(),
                            4 if row.children.len() > 1 => row.children.swap(0, 1),
                            4 if row.children.len() == 1 => row.children.clear(),
                            4 => row.children.push(TypedNodeId::new(usize::from(index == 0))),
                            _ => unreachable!(),
                        }
                    }
                });
                assert_eq!(
                    baseline.validate_installation(
                        fixture.source,
                        &fixture.module,
                        &statement,
                        &fixture.primary,
                        &arena,
                    ),
                    Err(SourceStatementWitnessError::DependencyMismatch),
                    "node {index} mutation {mutation}"
                );
            }
        }
        assert_eq!(
            fixture
                .witnesses(&statement, fixture.witness_input())
                .expect("replay")
                .debug_text(),
            baseline_debug
        );
    }

    #[test]
    fn task258b3m1_paired_ownership_hybrids_and_all_family_orders_are_atomic() {
        let fixture = B3M1Fixture::new(57);
        let statement = fixture.statement();
        let witnesses = fixture
            .witnesses(&statement, fixture.witness_input())
            .expect("witnesses");
        let base = fixture.empty_typed();
        let base_debug = base.debug_text();
        assert_eq!(
            base.clone().with_source_statement(statement.clone()),
            Err(TypedAstError::InvalidSourceStatement)
        );
        assert_eq!(base.debug_text(), base_debug);
        let typed = base
            .clone()
            .with_source_statement_witnesses(statement.clone(), witnesses.clone())
            .expect("paired install");
        let typed_debug = typed.debug_text();
        assert_eq!(typed.source_statement(), Some(&statement));
        assert_eq!(typed.source_statement_witnesses(), Some(&witnesses));
        assert!(typed.source_statement_references().is_none());
        assert_eq!(
            typed
                .clone()
                .with_source_statement_witnesses(statement.clone(), witnesses.clone()),
            Err(TypedAstError::InvalidSourceStatement)
        );
        assert_eq!(typed.debug_text(), typed_debug);

        let task258a = Fixture::new(57);
        let task258a_statement = task258a
            .build(task258a.input())
            .expect("Task258A statement");
        let task258a_typed = TypedAst::try_new(TypedAstParts {
            source_id: task258a.source,
            module_id: task258a.module.clone(),
            resolved_root: None,
            source_context: None,
            source_type: None,
            source_attribute: None,
            nodes: task258a.arena.clone(),
            contexts: LocalTypeContextTable::new(),
            types: TypeTable::new(),
            facts: TypeFactTable::new(),
            coercions: CoercionTable::new(),
            initial_obligations: InitialObligationTable::new(),
            diagnostics: TypeDiagnosticTable::new(),
        })
        .expect("Task258A empty")
        .with_source_term(task258a.primary.clone())
        .expect("Task258A primary")
        .with_source_atomic_formula(task258a.atomic.clone())
        .expect("Task258A atomic")
        .with_source_statement(task258a_statement.clone())
        .expect("Task258A install");
        let task258a_debug = task258a_typed.debug_text();
        assert_eq!(
            task258a_typed
                .clone()
                .with_source_statement_witnesses(statement.clone(), witnesses.clone()),
            Err(TypedAstError::InvalidSourceStatement)
        );
        assert_eq!(task258a_typed.debug_text(), task258a_debug);
        assert_eq!(
            typed.clone().with_source_statement(task258a_statement),
            Err(TypedAstError::InvalidSourceStatement)
        );

        let b1 = B1Fixture::new(57);
        let b1_references = b1.references(b1.reference_input()).expect("B1 references");
        let b1_typed = b1
            .empty_typed()
            .with_source_statement_references(b1.statement.clone(), b1_references.clone())
            .expect("B1 install");
        let b1_debug = b1_typed.debug_text();
        assert_eq!(
            b1_typed
                .clone()
                .with_source_statement_witnesses(statement.clone(), witnesses.clone()),
            Err(TypedAstError::InvalidSourceStatement)
        );
        assert_eq!(b1_typed.debug_text(), b1_debug);
        assert_eq!(
            typed
                .clone()
                .with_source_statement_references(b1.statement.clone(), b1_references.clone(),),
            Err(TypedAstError::InvalidSourceStatement)
        );

        let b2 = B2Fixture::new(57);
        let b2_statement = b2.build(b2.input()).expect("B2 statement");
        let b2_typed = b2
            .empty_typed()
            .with_source_statement(b2_statement.clone())
            .expect("B2 install");
        let b2_debug = b2_typed.debug_text();
        assert_eq!(
            b2_typed
                .clone()
                .with_source_statement_witnesses(statement.clone(), witnesses.clone()),
            Err(TypedAstError::InvalidSourceStatement)
        );
        assert_eq!(b2_typed.debug_text(), b2_debug);
        assert_eq!(
            typed.clone().with_source_statement(b2_statement),
            Err(TypedAstError::InvalidSourceStatement)
        );

        let b3 = B3Fixture::new(58);
        let b3_statement = b3.statement();
        let b3_witnesses = b3
            .witnesses(&b3_statement, b3.witness_input())
            .expect("B3 witnesses");
        let b3_typed = b3
            .empty_typed()
            .with_source_statement_witnesses(b3_statement.clone(), b3_witnesses.clone())
            .expect("B3 install");
        let b3_debug = b3_typed.debug_text();
        assert_eq!(
            base.clone()
                .with_source_statement_witnesses(b3_statement.clone(), b3_witnesses.clone(),),
            Err(TypedAstError::InvalidSourceStatement)
        );
        assert_eq!(
            b3_typed
                .clone()
                .with_source_statement_witnesses(statement.clone(), witnesses.clone()),
            Err(TypedAstError::InvalidSourceStatement)
        );
        assert_eq!(b3_typed.debug_text(), b3_debug);
        assert_eq!(
            typed
                .clone()
                .with_source_statement_witnesses(b3_statement, b3_witnesses),
            Err(TypedAstError::InvalidSourceStatement)
        );

        let b3n = B3NFixture::new(59);
        let b3n_statement = b3n.statement();
        let b3n_witnesses = b3n
            .witnesses(&b3n_statement, b3n.witness_input())
            .expect("B3N witnesses");
        let b3n_typed = b3n
            .empty_typed()
            .with_source_statement_witnesses(b3n_statement.clone(), b3n_witnesses.clone())
            .expect("B3N install");
        let b3n_debug = b3n_typed.debug_text();
        assert_eq!(
            base.clone()
                .with_source_statement_witnesses(b3n_statement.clone(), b3n_witnesses.clone(),),
            Err(TypedAstError::InvalidSourceStatement)
        );
        assert_eq!(
            b3n_typed
                .clone()
                .with_source_statement_witnesses(statement.clone(), witnesses.clone()),
            Err(TypedAstError::InvalidSourceStatement)
        );
        assert_eq!(b3n_typed.debug_text(), b3n_debug);
        assert_eq!(
            typed
                .clone()
                .with_source_statement_witnesses(b3n_statement, b3n_witnesses),
            Err(TypedAstError::InvalidSourceStatement)
        );

        let task248 = crate::source_context::tests::task_248_occupied_typed_ast(
            fixture.source,
            fixture.module.clone(),
        );
        let task248_debug = task248.debug_text();
        assert_eq!(
            task248
                .clone()
                .with_source_statement_witnesses(statement.clone(), witnesses.clone()),
            Err(TypedAstError::InvalidSourceStatement)
        );
        assert_eq!(task248.debug_text(), task248_debug);
        assert_eq!(
            typed.clone().with_source_context_for_test(
                task248
                    .source_context()
                    .expect("Task248 source context")
                    .clone(),
            ),
            Err(TypedAstError::InvalidSourceContext)
        );

        let task257a = crate::source_composite_formula::tests::task_257a_installed_typed_ast();
        let task257b = crate::source_formula_composition::tests::task_257b_installed_typed_ast();
        let task257c2 = crate::source_formula_composition::tests::task_257c2_installed_typed_ast();
        let task257c3 = crate::source_formula_composition::tests::task_257c3_installed_typed_ast();
        for (family, occupied) in [
            ("Task257A", &task257a),
            ("Task257B", &task257b),
            ("Task257C2", &task257c2),
            ("Task257C3", &task257c3),
        ] {
            let debug = occupied.debug_text();
            assert_eq!(
                occupied
                    .clone()
                    .with_source_statement_witnesses(statement.clone(), witnesses.clone()),
                Err(TypedAstError::InvalidSourceStatement),
                "{family}"
            );
            assert_eq!(occupied.debug_text(), debug, "{family} rollback");
        }
        assert_eq!(
            typed.clone().with_source_composite_formula(
                task257a
                    .source_composite_formula()
                    .expect("Task257A composite")
                    .clone(),
            ),
            Err(TypedAstError::InvalidSourceCompositeFormula)
        );
        assert_eq!(
            typed.clone().with_source_formula_composition(
                task257b
                    .source_composite_formula()
                    .expect("Task257B composite")
                    .clone(),
                task257b
                    .source_formula_composition()
                    .expect("Task257B composition")
                    .clone(),
            ),
            Err(TypedAstError::InvalidSourceFormulaComposition)
        );
        assert_eq!(
            typed.clone().with_source_condition_formula_composition(
                task257c2
                    .source_condition_formula_composition()
                    .expect("Task257C2 composition")
                    .clone(),
            ),
            Err(TypedAstError::InvalidSourceConditionFormulaComposition)
        );
        assert_eq!(
            typed.clone().with_source_predicate_chain_composition(
                task257c3
                    .source_predicate_chain_composition()
                    .expect("Task257C3 composition")
                    .clone(),
            ),
            Err(TypedAstError::InvalidSourcePredicateChainComposition)
        );
        assert_eq!(typed.debug_text(), typed_debug);
        assert_eq!(
            base.with_source_statement_witnesses(statement, witnesses)
                .expect("replay")
                .debug_text(),
            typed_debug
        );
    }

    #[test]
    fn task258b3m1_final_clone_revalidation_and_semantic_deferrals_are_stable() {
        let fixture = B3M1Fixture::new(60);
        let statement = fixture.statement();
        let witnesses = fixture
            .witnesses(&statement, fixture.witness_input())
            .expect("witnesses");
        let typed = fixture
            .empty_typed()
            .with_source_statement_witnesses(statement.clone(), witnesses.clone())
            .expect("typed");
        let typed_debug = typed.debug_text();
        let resolved = assemble_empty_resolved(&typed).expect("resolved");
        let resolved_debug = resolved.debug_text();
        assert_eq!(typed.clone().debug_text(), typed_debug);
        assert_eq!(resolved.clone().debug_text(), resolved_debug);
        assert_eq!(resolved.source_statement(), Some(&statement));
        assert_eq!(resolved.source_statement_witnesses(), Some(&witnesses));
        assert!(resolved.source_statement_references().is_none());
        assert!(resolved.expr_metadata().is_empty());
        assert!(resolved.collection_candidates().is_empty());
        assert!(resolved.expanded_candidates().is_empty());
        assert!(resolved.template_expansions().is_empty());
        assert!(resolved.viable_candidates().is_empty());
        assert!(resolved.viability_decisions().is_empty());
        assert!(resolved.specificity_graphs().is_empty());
        assert!(resolved.resolved_overloads().is_empty());
        assert!(resolved.inserted_coercions().is_empty());
        assert!(resolved.cluster_facts().is_empty());
        assert!(resolved.diagnostics().is_empty());
        assert!(resolved.checked_formulas().is_empty());
        assert!(resolved.statement_semantics().is_empty());
        assert!(resolved.checked_proofs().is_empty());
        assert!(resolved.checked_proof_nodes().is_empty());
        assert!(resolved.checked_terminal_goals().is_empty());

        let mut orphan = fixture.empty_typed();
        orphan.inject_source_statement_witnesses_for_test(witnesses.clone());
        assert_eq!(
            assemble_empty_resolved(&orphan),
            Err(ResolvedTypedAstError::InvalidSourceStatement)
        );
        let mut standalone = fixture.empty_typed();
        standalone.inject_source_statement_for_test(statement.clone());
        assert_eq!(
            assemble_empty_resolved(&standalone),
            Err(ResolvedTypedAstError::InvalidSourceStatement)
        );
        let mut stale_statement = witnesses.clone();
        stale_statement.statement_fingerprint.push('x');
        let mut stale_statement_pair = fixture.empty_typed();
        stale_statement_pair
            .inject_source_statement_witness_bundle_for_test(statement.clone(), stale_statement);
        assert_eq!(
            assemble_empty_resolved(&stale_statement_pair),
            Err(ResolvedTypedAstError::InvalidSourceStatement)
        );
        let mut stale_primary = witnesses.clone();
        stale_primary.primary_term_fingerprint.push('x');
        let mut stale_primary_pair = fixture.empty_typed();
        stale_primary_pair
            .inject_source_statement_witness_bundle_for_test(statement.clone(), stale_primary);
        assert_eq!(
            assemble_empty_resolved(&stale_primary_pair),
            Err(ResolvedTypedAstError::InvalidSourceStatement)
        );
        let b1 = B1Fixture::new(60);
        let mut reference_hybrid = fixture.empty_typed();
        reference_hybrid.inject_source_statement_bundle_for_test(
            statement.clone(),
            b1.references(b1.reference_input()).expect("B1 references"),
        );
        reference_hybrid.inject_source_statement_witnesses_for_test(witnesses.clone());
        assert_eq!(
            assemble_empty_resolved(&reference_hybrid),
            Err(ResolvedTypedAstError::InvalidSourceStatement)
        );
        for table in [
            StatementTransportTableForTest::Context,
            StatementTransportTableForTest::Type,
            StatementTransportTableForTest::Fact,
            StatementTransportTableForTest::Coercion,
            StatementTransportTableForTest::InitialObligation,
            StatementTransportTableForTest::Diagnostic,
        ] {
            let mut occupied = typed.clone();
            occupied.occupy_statement_transport_table_for_test(table);
            let occupied_debug = occupied.debug_text();
            assert_eq!(
                assemble_empty_resolved(&occupied),
                Err(ResolvedTypedAstError::InvalidSourceStatement),
                "{table:?}"
            );
            assert_eq!(occupied.debug_text(), occupied_debug, "{table:?} rollback");
        }
        let term_formula = TermFormulaChecker::default().infer(
            &fixture.symbols,
            &fixture.bindings,
            Vec::<crate::type_checker::TermInput>::new(),
            Vec::<crate::type_checker::FormulaInput>::new(),
        );
        assert_eq!(
            assemble_resolved_with_statement_semantics(
                &typed,
                statement.checked_owner(),
                &fixture.bindings,
                &term_formula,
            ),
            Err(ResolvedTypedAstError::InvalidSourceStatement)
        );
        let mut cluster_facts = ClusterFactTable::new();
        cluster_facts.insert(ClusterFactDraft {
            fingerprint: ClusterFactFingerprint::new("task258b3m1-coexistence"),
            source_type: ClusterTypeFingerprint::new("set"),
            attribute: ClusterAttributeFingerprint::new("inhabited"),
            generated_type: ClusterTypeFingerprint::new("set"),
            provenance: ClusterFactProvenance::Input,
            source_range: range(fixture.source, 84, 85),
        });
        assert_eq!(
            assemble_resolved(&typed, &cluster_facts, Vec::new(), None),
            Err(ResolvedTypedAstError::InvalidSourceStatement)
        );
        assert_eq!(
            assemble_resolved(
                &typed,
                &ClusterFactTable::new(),
                vec![ResolvedNodeKindHint {
                    typed_node: TypedNodeId::new(13),
                    kind: ResolvedNodeKindHintKind::SourcePreserved {
                        role: SourceNodeRole::new(
                            "source.statement-witness-name.semantic-coexistence",
                        ),
                    },
                }],
                None,
            ),
            Err(ResolvedTypedAstError::InvalidSourceStatement)
        );
        assert_eq!(
            assemble_resolved(
                &typed,
                &ClusterFactTable::new(),
                Vec::new(),
                Some(StatementProofInputs {
                    owner: statement.checked_owner(),
                    rows: Vec::new(),
                }),
            ),
            Err(ResolvedTypedAstError::InvalidSourceStatement)
        );
        assert_eq!(
            assemble_resolved_with_expressions(
                &typed,
                vec![ExpressionMetadataInput {
                    expr: ExprId::new("task258b3m1-semantic-coexistence"),
                    typed_site: node(36),
                    local_context: None,
                    cluster_facts: Vec::new(),
                }],
            ),
            Err(ResolvedTypedAstError::InvalidSourceStatement)
        );
        let empty_collection = OverloadCollectionOutput::collect(
            Vec::<OverloadSiteInput>::new(),
            Vec::<OverloadCandidateInput>::new(),
        );
        let empty_expansion = TemplateExpansionOutput::expand(&empty_collection);
        let empty_viability = CandidateViabilityOutput::filter(
            &empty_expansion,
            Vec::<CandidateViabilityInput>::new(),
        );
        let empty_specificity = SpecificityGraphOutput::build(
            &empty_viability,
            Vec::<SpecificityComparisonInput>::new(),
        );
        let empty_selection = OverloadSelectionOutput::resolve(
            &empty_specificity,
            Vec::<OverloadSiteResolutionInput>::new(),
        );
        let occupied_collection = OverloadCollectionOutput::collect(
            vec![OverloadSiteInput {
                key: OverloadSiteKey::new("task258b3m1-overload"),
                owner: TypedSiteRef::Role {
                    node: TypedNodeId::new(36),
                    role: TypeRole::new("task258b3m1-overload-owner"),
                },
                source_range: range(fixture.source, 88, 89),
                kind: OverloadSiteKind::FunctorApplication,
                name: OverloadNameKey::new("task258b3m1-overload"),
                arguments: Vec::new(),
                expected: None,
                source_qua: Vec::<SourceQuaView>::new(),
                recovery: OverloadSiteRecovery::Normal,
            }],
            vec![OverloadCandidateInput {
                site: OverloadSiteKey::new("task258b3m1-overload"),
                symbol: fixture.symbol.clone(),
                ordinary_root: fixture.symbol.clone(),
                declaration_kind: CandidateDeclarationKind::Functor,
                parameters: Vec::new(),
                result: None,
                origin: CandidateOrigin::Ordinary,
                template: None,
                coherence: None,
                provenance: CandidateProvenance {
                    stable_key: CandidateProvenanceKey::new("task258b3m1-overload"),
                    source_range: Some(range(fixture.source, 88, 89)),
                    scope: CandidateScope::Local,
                    declaration_order: 0,
                },
            }],
        );
        let occupied_expansion = TemplateExpansionOutput::expand(&occupied_collection);
        let occupied_viability = CandidateViabilityOutput::filter(
            &occupied_expansion,
            vec![CandidateViabilityInput {
                candidate: OverloadCandidateId::new(0),
                arguments: Vec::new(),
            }],
        );
        let occupied_specificity = SpecificityGraphOutput::build(
            &occupied_viability,
            Vec::<SpecificityComparisonInput>::new(),
        );
        let occupied_selection = OverloadSelectionOutput::resolve(
            &occupied_specificity,
            vec![OverloadSiteResolutionInput {
                site: OverloadSiteId::new(0),
                refinements: vec![OverloadCandidateId::new(0)],
                refinement_join: RefinementJoinPayload {
                    status: RefinementJoinStatus::Compatible,
                    exposed_result: None,
                },
                inserted_views: Vec::new(),
            }],
        );
        assert_eq!(occupied_collection.candidates().len(), 1);
        assert_eq!(occupied_expansion.candidates().len(), 1);
        assert_eq!(occupied_viability.candidates().len(), 1);
        assert_eq!(occupied_specificity.candidates().len(), 1);
        assert_eq!(occupied_selection.results().len(), 1);
        let cluster_facts = ClusterFactTable::new();
        for (label, collection, expansion, viability, specificity, selection) in [
            (
                "collection",
                &occupied_collection,
                &empty_expansion,
                &empty_viability,
                &empty_specificity,
                &empty_selection,
            ),
            (
                "expansion",
                &empty_collection,
                &occupied_expansion,
                &empty_viability,
                &empty_specificity,
                &empty_selection,
            ),
            (
                "viability",
                &empty_collection,
                &empty_expansion,
                &occupied_viability,
                &empty_specificity,
                &empty_selection,
            ),
            (
                "specificity",
                &empty_collection,
                &empty_expansion,
                &empty_viability,
                &occupied_specificity,
                &empty_selection,
            ),
            (
                "selection",
                &empty_collection,
                &empty_expansion,
                &empty_viability,
                &empty_specificity,
                &occupied_selection,
            ),
        ] {
            assert_eq!(
                ResolvedTypedAst::assemble(ResolvedTypedAstInputs {
                    typed_ast: &typed,
                    cluster_facts: &cluster_facts,
                    overload_collection: collection,
                    template_expansion: expansion,
                    viability,
                    specificity,
                    overload_selection: selection,
                    expressions: Vec::new(),
                    node_hints: Vec::new(),
                    statement_semantics: None,
                    statement_proofs: None,
                }),
                Err(ResolvedTypedAstError::InvalidSourceStatement),
                "{label}"
            );
        }
        assert_eq!(typed.debug_text(), typed_debug);
        assert_eq!(
            assemble_empty_resolved(&typed)
                .expect("final replay")
                .debug_text(),
            resolved_debug
        );
    }

    #[test]
    fn task258b2_exact_assumption_profile_accessors_and_debug_are_stable() {
        let fixture = B2Fixture::new(30);
        let handoff = fixture.build(fixture.input()).expect("Task258B2 handoff");
        assert!(handoff.is_task_258b2_profile());
        assert!(!handoff.is_task_258a_profile());
        assert!(!handoff.is_task_258b1_profile());
        assert_eq!(handoff.source_id(), fixture.source);
        assert_eq!(handoff.module_id(), &fixture.module);
        assert_eq!(handoff.binding_env(), &fixture.bindings);
        assert_eq!(handoff.binding_env().contexts().len(), 2);
        assert_eq!(handoff.binding_env().bindings().len(), 1);
        assert!(handoff.binding_env().diagnostics().is_empty());
        let proof = handoff
            .binding_env()
            .contexts()
            .get(BindingContextId::new(1))
            .expect("proof context");
        assert_eq!(
            proof.owner,
            BindingContextOwner::SourceStatement {
                source_range: range(fixture.source, 72, 111),
            }
        );
        assert_eq!(proof.parent, Some(BindingContextId::new(0)));
        assert_eq!(proof.layer, BindingContextLayer::Proof);
        assert_eq!(proof.lexical_scope.as_ref().expect("scope").path(), [0]);
        assert!(proof.bindings.is_empty());
        assert_eq!(proof.visible_bindings, [BindingId::new(0)]);

        assert_eq!(fixture.primary.terms().len(), 6);
        assert_eq!(fixture.primary.references().len(), 6);
        assert!(fixture.primary.numeric_type_requests().is_empty());
        assert_eq!(
            fixture
                .primary
                .terms()
                .iter()
                .map(|(_, row)| row.context().index())
                .collect::<Vec<_>>(),
            [0, 0, 1, 1, 1, 1]
        );
        assert_eq!(
            fixture
                .primary
                .references()
                .iter()
                .map(|(_, row)| row.use_ordinal())
                .collect::<Vec<_>>(),
            [1; 6]
        );
        assert_eq!(fixture.atomic.formulas().len(), 3);
        assert_eq!(fixture.atomic.edges().len(), 6);
        assert_eq!(fixture.atomic.requests().len(), 6);
        assert_eq!(handoff.binding_fingerprint(), fixture.bindings.debug_text());
        assert_eq!(
            handoff.primary_term_fingerprint(),
            fixture.primary.debug_text()
        );
        assert_eq!(
            handoff.atomic_formula_fingerprint(),
            fixture.atomic.debug_text()
        );
        assert_eq!(
            fixture
                .atomic
                .formulas()
                .iter()
                .map(|(_, row)| row.context().index())
                .collect::<Vec<_>>(),
            [0, 1, 1]
        );

        assert_eq!(handoff.owners().len(), 1);
        assert_eq!(handoff.statements().len(), 3);
        assert_eq!(handoff.contexts().len(), 3);
        assert_eq!(handoff.input_facts().len(), 3);
        assert_eq!(handoff.candidate_facts().len(), 3);
        let owner = handoff
            .owners()
            .get(SourceTheoremOwnerId::new(0))
            .expect("owner");
        assert_eq!(owner.contribution().index(), 0);
        assert_eq!(owner.source_range(), range(fixture.source, 19, 112));
        assert_eq!(handoff.checked_owner().origin().structural_path(), [2, 1]);
        let expected = [
            (
                SourceStatementKind::TheoremProposition,
                51,
                (19, 112),
                0,
                "theorem FormulaStatementSingleAssumptionSmoke : x = x proof assume x = x ; thus x = x ; end ;",
            ),
            (
                SourceStatementKind::Assumption,
                41,
                (80, 93),
                1,
                "assume x = x ;",
            ),
            (
                SourceStatementKind::Conclusion,
                49,
                (96, 107),
                1,
                "thus x = x ;",
            ),
        ];
        for (index, (kind, site, (start, end), binding_context, spelling)) in
            expected.into_iter().enumerate()
        {
            let statement = handoff
                .statements()
                .get(SourceStatementId::new(index))
                .expect("statement");
            assert_eq!(statement.owner(), SourceTheoremOwnerId::new(0));
            assert_eq!(statement.context(), SourceStatementContextId::new(index));
            assert_eq!(
                statement.formula(),
                SourceStatementFormulaTarget::Atomic(SourceAtomicFormulaId::new(index))
            );
            assert_eq!(statement.site().node().index(), site);
            assert_eq!(statement.source_range(), range(fixture.source, start, end));
            assert_eq!(statement.source_ordinal(), index);
            assert_eq!(statement.spelling(), spelling);
            assert_eq!(statement.kind(), kind);
            assert_eq!(statement.recovery(), SourceStatementRecovery::Normal);
            let context = handoff
                .contexts()
                .get(SourceStatementContextId::new(index))
                .expect("context");
            assert_eq!(context.statement(), SourceStatementId::new(index));
            assert_eq!(
                context.binding_context(),
                BindingContextId::new(binding_context)
            );
            assert_eq!(context.source_range(), statement.source_range());
            assert_eq!(context.visible_bindings(), [BindingId::new(0)]);
            let input = handoff
                .input_facts()
                .get(SourceStatementInputFactId::new(index))
                .expect("input fact");
            assert_eq!(input.statement(), SourceStatementId::new(index));
            assert_eq!(input.context(), SourceStatementContextId::new(index));
            assert_eq!(input.ordinal(), 0);
            assert_eq!(
                input.kind(),
                SourceStatementInputFactKind::ReservedTypeGuard
            );
            assert_eq!(input.binding(), BindingId::new(0));
            assert_eq!(
                input.uses(),
                [
                    SourcePrimaryTermReferenceId::new(index * 2),
                    SourcePrimaryTermReferenceId::new(index * 2 + 1),
                ]
            );
            let candidate = handoff
                .candidate_facts()
                .get(SourceStatementCandidateFactId::new(index))
                .expect("candidate fact");
            assert_eq!(candidate.statement(), SourceStatementId::new(index));
            assert_eq!(candidate.context(), SourceStatementContextId::new(index));
            assert_eq!(candidate.ordinal(), 0);
            assert_eq!(
                candidate.kind(),
                SourceStatementCandidateFactKind::UnverifiedProposition
            );
            assert_eq!(candidate.formula(), statement.formula());
        }
        let debug = handoff.debug_text();
        assert!(debug.contains(
            "statement#1 ordinal=1 owner=0 context=1 formula=atomic:1 kind=assumption range=80..93 site=41 recovery=normal spelling=\"assume x = x ;\""
        ));
        assert_eq!(handoff.clone().debug_text(), debug);
        assert_eq!(
            fixture
                .build(fixture.input())
                .expect("Task258B2 replay")
                .debug_text(),
            debug
        );
    }

    #[test]
    fn task258b2_dependency_rows_containment_and_replay_fail_closed() {
        let fixture = B2Fixture::new(31);
        let valid = fixture.build(fixture.input()).expect("Task258B2 baseline");
        let baseline = valid.debug_text();

        let mut aggregate = fixture.input();
        aggregate.candidate_facts.pop();
        assert_eq!(
            fixture.build(aggregate),
            Err(SourceStatementError::InvalidAggregate)
        );
        let mut assumption_kind = fixture.input();
        assumption_kind.statements[1].kind = SourceStatementKind::Conclusion;
        assert_eq!(
            fixture.build(assumption_kind),
            Err(SourceStatementError::InvalidStatement {
                statement: SourceStatementId::new(1)
            })
        );
        let mut cross_formula = fixture.input();
        cross_formula.statements[1].formula =
            SourceStatementFormulaTarget::Atomic(SourceAtomicFormulaId::new(2));
        assert_eq!(
            fixture.build(cross_formula),
            Err(SourceStatementError::InvalidStatement {
                statement: SourceStatementId::new(1)
            })
        );
        let mut wrong_context = fixture.input();
        wrong_context.contexts[1].binding_context = BindingContextId::new(0);
        assert_eq!(
            fixture.build(wrong_context),
            Err(SourceStatementError::InvalidContext {
                context: SourceStatementContextId::new(1)
            })
        );
        let mut wrong_uses = fixture.input();
        wrong_uses.input_facts[1].uses.swap(0, 1);
        assert_eq!(
            fixture.build(wrong_uses),
            Err(SourceStatementError::InvalidInputFact {
                fact: SourceStatementInputFactId::new(1)
            })
        );
        let mut wrong_candidate = fixture.input();
        wrong_candidate.candidate_facts[1].formula =
            SourceStatementFormulaTarget::Atomic(SourceAtomicFormulaId::new(2));
        assert_eq!(
            fixture.build(wrong_candidate),
            Err(SourceStatementError::InvalidCandidateFact {
                fact: SourceStatementCandidateFactId::new(1)
            })
        );

        let escaped_assumption = mutate_arena(&fixture.arena, |id, row| {
            if id == TypedNodeId::new(41) {
                row.children.retain(|child| *child != TypedNodeId::new(40));
            }
        });
        assert_eq!(
            valid.validate_installation(
                fixture.source,
                &fixture.module,
                &fixture.primary,
                &fixture.atomic,
                &escaped_assumption,
            ),
            Err(SourceStatementError::InvalidStatement {
                statement: SourceStatementId::new(1)
            })
        );
        let cross_subtree = mutate_arena(&fixture.arena, |id, row| {
            if id == TypedNodeId::new(41) {
                row.children.push(TypedNodeId::new(46));
            }
        });
        assert_eq!(
            valid.validate_installation(
                fixture.source,
                &fixture.module,
                &fixture.primary,
                &fixture.atomic,
                &cross_subtree,
            ),
            Err(SourceStatementError::InvalidStatement {
                statement: SourceStatementId::new(0)
            })
        );
        let recovered_assumption = mutate_arena(&fixture.arena, |id, row| {
            if id == TypedNodeId::new(41) {
                row.recovery = NodeRecoveryState::Recovered;
            }
        });
        assert_eq!(
            valid.validate_installation(
                fixture.source,
                &fixture.module,
                &fixture.primary,
                &fixture.atomic,
                &recovered_assumption,
            ),
            Err(SourceStatementError::InvalidStatement {
                statement: SourceStatementId::new(1)
            })
        );

        let mut stale_origin = valid.clone();
        stale_origin.checked_owner = CheckedStatementOwner::from_validated_parts_for_test(
            fixture.symbol.clone(),
            range(fixture.source, 19, 112),
            SemanticOrigin::new(
                fixture.source,
                fixture.module.clone(),
                SourceAnchor::Range(range(fixture.source, 19, 112)),
                vec![1],
            ),
        );
        assert_eq!(
            stale_origin.validate_installation(
                fixture.source,
                &fixture.module,
                &fixture.primary,
                &fixture.atomic,
                &fixture.arena,
            ),
            Err(SourceStatementError::InvalidOwner {
                owner: SourceTheoremOwnerId::new(0)
            })
        );
        let mut cross_profile = valid.clone();
        cross_profile.binding_env = b1_binding_env(fixture.source, &fixture.module);
        cross_profile.binding_fingerprint = cross_profile.binding_env.debug_text();
        assert_eq!(
            cross_profile.validate_installation(
                fixture.source,
                &fixture.module,
                &fixture.primary,
                &fixture.atomic,
                &fixture.arena,
            ),
            Err(SourceStatementError::DependencyMismatch)
        );
        let foreign = B2Fixture::new(32);
        assert_eq!(
            valid.validate_installation(
                fixture.source,
                &fixture.module,
                &foreign.primary,
                &fixture.atomic,
                &fixture.arena,
            ),
            Err(SourceStatementError::DependencyMismatch)
        );
        assert_eq!(
            valid.validate_installation(
                fixture.source,
                &fixture.module,
                &fixture.primary,
                &foreign.atomic,
                &fixture.arena,
            ),
            Err(SourceStatementError::DependencyMismatch)
        );
        let mut stale_atomic_fingerprint = valid.clone();
        stale_atomic_fingerprint
            .atomic_formula_fingerprint
            .push('x');
        assert_eq!(
            stale_atomic_fingerprint.validate_installation(
                fixture.source,
                &fixture.module,
                &fixture.primary,
                &fixture.atomic,
                &fixture.arena,
            ),
            Err(SourceStatementError::DependencyMismatch)
        );
        assert_eq!(
            fixture
                .build(fixture.input())
                .expect("Task258B2 replay")
                .debug_text(),
            baseline
        );
    }

    #[test]
    fn task258b2_base_only_typed_ownership_and_cross_profiles_are_atomic() {
        let fixture = B2Fixture::new(33);
        let handoff = fixture.build(fixture.input()).expect("Task258B2 handoff");
        let base = fixture.empty_typed();
        let base_debug = base.debug_text();
        let installed = base
            .clone()
            .with_source_statement(handoff.clone())
            .expect("Task258B2 base-only install");
        assert_eq!(installed.source_statement(), Some(&handoff));
        assert!(installed.source_statement_references().is_none());
        let installed_debug = installed.debug_text();
        assert_eq!(
            installed.clone().with_source_statement(handoff.clone()),
            Err(TypedAstError::InvalidSourceStatement)
        );
        assert_eq!(installed.debug_text(), installed_debug);

        let b1 = B1Fixture::new(33);
        let references = b1
            .references(b1.reference_input())
            .expect("Task258B1 references");
        assert_eq!(
            base.clone()
                .with_source_statement_references(handoff.clone(), references),
            Err(TypedAstError::InvalidSourceStatement)
        );
        assert_eq!(base.debug_text(), base_debug);
        assert_eq!(
            base.clone().with_source_statement(b1.statement.clone()),
            Err(TypedAstError::InvalidSourceStatement)
        );
        let task258a = Fixture::new(33);
        assert_eq!(
            base.clone().with_source_statement(
                task258a
                    .build(task258a.input())
                    .expect("Task258A statement")
            ),
            Err(TypedAstError::InvalidSourceStatement)
        );

        let task248 = crate::source_context::tests::task_248_occupied_typed_ast(
            fixture.source,
            fixture.module.clone(),
        );
        let task248_debug = task248.debug_text();
        assert_eq!(
            task248.clone().with_source_statement(handoff.clone()),
            Err(TypedAstError::InvalidSourceStatement)
        );
        assert_eq!(task248.debug_text(), task248_debug);

        let task257a = crate::source_composite_formula::tests::task_257a_installed_typed_ast();
        let task257b = crate::source_formula_composition::tests::task_257b_installed_typed_ast();
        let task257c2 = crate::source_formula_composition::tests::task_257c2_installed_typed_ast();
        let task257c3 = crate::source_formula_composition::tests::task_257c3_installed_typed_ast();
        for (family, occupied) in [
            ("Task257A", &task257a),
            ("Task257B", &task257b),
            ("Task257C2", &task257c2),
            ("Task257C3", &task257c3),
        ] {
            let debug = occupied.debug_text();
            assert_eq!(
                occupied.clone().with_source_statement(handoff.clone()),
                Err(TypedAstError::InvalidSourceStatement),
                "{family} first"
            );
            assert_eq!(occupied.debug_text(), debug, "{family} rollback");
        }
        assert_eq!(
            installed.clone().with_source_composite_formula(
                task257a
                    .source_composite_formula()
                    .expect("Task257A composite")
                    .clone(),
            ),
            Err(TypedAstError::InvalidSourceCompositeFormula)
        );
        assert_eq!(
            installed.clone().with_source_formula_composition(
                task257b
                    .source_composite_formula()
                    .expect("Task257B composite")
                    .clone(),
                task257b
                    .source_formula_composition()
                    .expect("Task257B composition")
                    .clone(),
            ),
            Err(TypedAstError::InvalidSourceFormulaComposition)
        );
        assert_eq!(
            installed.clone().with_source_condition_formula_composition(
                task257c2
                    .source_condition_formula_composition()
                    .expect("Task257C2 composition")
                    .clone(),
            ),
            Err(TypedAstError::InvalidSourceConditionFormulaComposition)
        );
        assert_eq!(
            installed.clone().with_source_predicate_chain_composition(
                task257c3
                    .source_predicate_chain_composition()
                    .expect("Task257C3 composition")
                    .clone(),
            ),
            Err(TypedAstError::InvalidSourcePredicateChainComposition)
        );

        let mut semantic_facts = TypeFactTable::new();
        semantic_facts.insert(TypeFactDraft {
            subject: node(28),
            predicate: TypePredicateRef::new("set"),
            polarity: Polarity::Positive,
            provenance: FactProvenance::Inferred(TypeRuleId::new("task258b2-coexistence")),
            status: FactStatus::Known,
        });
        let semantic = TypedAst::try_new(TypedAstParts {
            source_id: fixture.source,
            module_id: fixture.module.clone(),
            resolved_root: None,
            source_context: None,
            source_type: None,
            source_attribute: None,
            nodes: fixture.arena.clone(),
            contexts: LocalTypeContextTable::new(),
            types: TypeTable::new(),
            facts: semantic_facts,
            coercions: CoercionTable::new(),
            initial_obligations: InitialObligationTable::new(),
            diagnostics: TypeDiagnosticTable::new(),
        })
        .expect("Task258B2 semantic base")
        .with_source_term(fixture.primary.clone())
        .expect("Task252")
        .with_source_atomic_formula(fixture.atomic.clone())
        .expect("Task256");
        let semantic_debug = semantic.debug_text();
        assert_eq!(
            semantic.clone().with_source_statement(handoff.clone()),
            Err(TypedAstError::InvalidSourceStatement)
        );
        assert_eq!(semantic.debug_text(), semantic_debug);
        assert_eq!(
            base.with_source_statement(handoff)
                .expect("Task258B2 replay")
                .debug_text(),
            installed_debug
        );
    }

    #[test]
    fn task258b2_final_clone_revalidation_and_empty_semantics_are_stable() {
        let fixture = B2Fixture::new(34);
        let handoff = fixture.build(fixture.input()).expect("Task258B2 handoff");
        let typed = fixture
            .empty_typed()
            .with_source_statement(handoff.clone())
            .expect("Task258B2 typed AST");
        let resolved = assemble_empty_resolved(&typed).expect("Task258B2 final assembly");
        assert_eq!(resolved.source_statement(), Some(&handoff));
        assert!(resolved.source_statement_references().is_none());
        assert_eq!(resolved.source_term(), typed.source_term());
        assert_eq!(
            resolved.source_atomic_formula(),
            typed.source_atomic_formula()
        );
        assert_eq!(resolved.clone().debug_text(), resolved.debug_text());
        assert!(resolved.expr_metadata().is_empty());
        assert!(resolved.collection_candidates().is_empty());
        assert!(resolved.expanded_candidates().is_empty());
        assert!(resolved.template_expansions().is_empty());
        assert!(resolved.viable_candidates().is_empty());
        assert!(resolved.viability_decisions().is_empty());
        assert!(resolved.specificity_graphs().is_empty());
        assert!(resolved.resolved_overloads().is_empty());
        assert!(resolved.inserted_coercions().is_empty());
        assert!(resolved.cluster_facts().is_empty());
        assert!(resolved.diagnostics().is_empty());
        assert!(resolved.checked_formulas().is_empty());
        assert!(resolved.statement_semantics().is_empty());
        assert!(resolved.checked_proofs().is_empty());
        assert!(resolved.checked_proof_nodes().is_empty());
        assert!(resolved.checked_terminal_goals().is_empty());

        let b1 = B1Fixture::new(34);
        let references = b1
            .references(b1.reference_input())
            .expect("Task258B1 references");
        let mut orphan_reference = typed.clone();
        orphan_reference.inject_source_statement_references_for_test(references);
        assert_eq!(
            assemble_empty_resolved(&orphan_reference),
            Err(ResolvedTypedAstError::InvalidSourceStatement)
        );

        let mut cluster_facts = ClusterFactTable::new();
        cluster_facts.insert(ClusterFactDraft {
            fingerprint: ClusterFactFingerprint::new("task258b2-coexistence"),
            source_type: ClusterTypeFingerprint::new("set"),
            attribute: ClusterAttributeFingerprint::new("inhabited"),
            generated_type: ClusterTypeFingerprint::new("set"),
            provenance: ClusterFactProvenance::Input,
            source_range: range(fixture.source, 87, 88),
        });
        assert_eq!(
            assemble_resolved(&typed, &cluster_facts, Vec::new(), None),
            Err(ResolvedTypedAstError::InvalidSourceStatement)
        );
        assert_eq!(
            assemble_resolved(
                &typed,
                &ClusterFactTable::new(),
                Vec::new(),
                Some(StatementProofInputs {
                    owner: handoff.checked_owner(),
                    rows: Vec::new(),
                }),
            ),
            Err(ResolvedTypedAstError::InvalidSourceStatement)
        );
        let replay = assemble_empty_resolved(&typed).expect("Task258B2 final replay");
        assert_eq!(replay.debug_text(), resolved.debug_text());
    }

    #[derive(Clone, Copy)]
    enum ResolverMutation {
        MissingLabelEffect,
        RecoveredLabel,
        WrongContribution,
        DuplicateLabel,
        StaleLabel,
        WrongLabelKind,
        PrivateLabel,
        LocalOnlyLabel,
        WrongLabelSource,
        WrongLabelModule,
        WrongLabelNamespace,
        WrongResolverNamespace,
        WrongContributionAnchor,
    }

    fn source_id(source_ordinal: usize) -> SourceId {
        let snapshot = BuildSnapshotId::from_published_schema_str(&format!(
            "mizar-session-build-snapshot-v1:{}",
            "d9".repeat(32)
        ))
        .expect("snapshot");
        let allocator = InMemorySessionIdAllocator::new();
        (0..source_ordinal)
            .map(|_| allocator.next_source_id(snapshot).expect("source"))
            .last()
            .expect("positive source ordinal")
    }

    fn node(index: usize) -> TypedSiteRef {
        TypedSiteRef::Node(TypedNodeId::new(index))
    }

    fn b3_node_range(index: usize) -> (usize, usize) {
        const RANGES: [(usize, usize); 49] = [
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
        RANGES[index]
    }

    fn b3_node_children(index: usize) -> Vec<usize> {
        match index {
            22 => vec![3],
            23 => vec![22],
            24 => vec![1, 2, 23],
            25 => vec![0, 24, 4],
            26 => vec![8],
            27 => vec![26],
            28 => vec![10],
            29 => vec![28],
            30 => vec![27, 9, 29],
            31 => vec![30],
            32 => vec![13],
            33 => vec![32],
            34 => vec![33],
            35 => vec![12, 34, 14],
            36 => vec![16],
            37 => vec![36],
            38 => vec![18],
            39 => vec![38],
            40 => vec![37, 17, 39],
            41 => vec![40],
            42 => vec![41],
            43 => vec![15, 42, 19],
            44 => vec![11, 35, 43, 20],
            45 => vec![5, 6, 7, 31, 44, 21],
            46 => vec![25, 45],
            47 => vec![46],
            48 => (0..22).chain(std::iter::once(47)).collect(),
            _ => Vec::new(),
        }
    }

    fn b3_typed_arena(source: SourceId) -> TypedArena {
        let mut builder = TypedArenaBuilder::new();
        let mut ids = Vec::with_capacity(49);
        for index in 0..49 {
            let (start, end) = b3_node_range(index);
            let kind = match index {
                26 | 28 | 32 | 36 | 38 => "source.term.variable-reference",
                30 | 40 => "source.formula.atomic.equality",
                34 => "source.statement-witness.item",
                35 => "source.statement-witness.take",
                43 => "source.statement.conclusion",
                45 => "source.statement.theorem",
                _ => "source.surface.unowned",
            };
            let children = b3_node_children(index)
                .into_iter()
                .map(|child| ids[child])
                .collect();
            let id = builder
                .push(
                    TypedNode::new(kind, SourceAnchor::Range(range(source, start, end)))
                        .with_children(children),
                )
                .expect("Task258B3 typed node");
            assert_eq!(id.index(), index);
            ids.push(id);
        }
        builder.finish(Some(ids[48])).expect("Task258B3 arena")
    }

    fn b3n_typed_arena(source: SourceId) -> TypedArena {
        let mut builder = TypedArenaBuilder::new();
        let mut ids = Vec::with_capacity(TASK258B3N_NODE_RANGES.len());
        for (index, (start, end)) in TASK258B3N_NODE_RANGES.iter().copied().enumerate() {
            let children = task258b3n_node_children(index)
                .iter()
                .map(|child| ids[*child])
                .collect();
            let id = builder
                .push(
                    TypedNode::new(
                        task258b3n_node_kind(index),
                        SourceAnchor::Range(range(source, start, end)),
                    )
                    .with_children(children),
                )
                .expect("Task258B3N typed node");
            assert_eq!(id.index(), index);
            ids.push(id);
        }
        builder.finish(Some(ids[50])).expect("Task258B3N arena")
    }

    fn b3m1_typed_arena(source: SourceId) -> TypedArena {
        let mut builder = TypedArenaBuilder::new();
        let mut ids = Vec::with_capacity(TASK258B3M1_NODE_RANGES.len());
        for (index, (start, end)) in TASK258B3M1_NODE_RANGES.iter().copied().enumerate() {
            let children = task258b3m1_node_children(index)
                .iter()
                .map(|child| ids[*child])
                .collect();
            let id = builder
                .push(
                    TypedNode::new(
                        task258b3m1_node_kind(index),
                        SourceAnchor::Range(range(source, start, end)),
                    )
                    .with_children(children),
                )
                .expect("Task258B3M1 typed node");
            assert_eq!(id.index(), index);
            ids.push(id);
        }
        builder.finish(Some(ids[55])).expect("Task258B3M1 arena")
    }

    fn b3_binding_env(source: SourceId, module: &ModuleId) -> BindingEnv {
        let base = binding_env(source, module);
        let mut contexts = base.contexts().clone();
        let proof = contexts.insert(BindingContextDraft {
            owner: BindingContextOwner::SourceStatement {
                source_range: range(source, 69, 102),
            },
            parent: Some(BindingContextId::new(0)),
            layer: BindingContextLayer::Proof,
            lexical_scope: Some(LocalTermScope::new(vec![0])),
            bindings: Vec::new(),
            visible_bindings: vec![BindingId::new(0)],
            recovery: BindingContextRecovery::Normal,
        });
        assert_eq!(proof, BindingContextId::new(1));
        BindingEnv::try_new(BindingEnvParts {
            source_id: source,
            module_id: module.clone(),
            contexts,
            bindings: base.bindings().clone(),
            diagnostics: base.diagnostics().clone(),
        })
        .expect("Task258B3 binding env")
    }

    fn b3n_binding_env(source: SourceId, module: &ModuleId) -> BindingEnv {
        let base = binding_env(source, module);
        let mut contexts = base.contexts().clone();
        let proof = contexts.insert(BindingContextDraft {
            owner: BindingContextOwner::SourceStatement {
                source_range: range(source, 68, 105),
            },
            parent: Some(BindingContextId::new(0)),
            layer: BindingContextLayer::Proof,
            lexical_scope: Some(LocalTermScope::new(vec![0])),
            bindings: Vec::new(),
            visible_bindings: vec![BindingId::new(0)],
            recovery: BindingContextRecovery::Normal,
        });
        assert_eq!(proof, BindingContextId::new(1));
        BindingEnv::try_new(BindingEnvParts {
            source_id: source,
            module_id: module.clone(),
            contexts,
            bindings: base.bindings().clone(),
            diagnostics: base.diagnostics().clone(),
        })
        .expect("Task258B3N binding env")
    }

    fn b3m1_binding_env(source: SourceId, module: &ModuleId) -> BindingEnv {
        let base = binding_env(source, module);
        let mut contexts = base.contexts().clone();
        let proof = contexts.insert(BindingContextDraft {
            owner: BindingContextOwner::SourceStatement {
                source_range: range(source, 71, 111),
            },
            parent: Some(BindingContextId::new(0)),
            layer: BindingContextLayer::Proof,
            lexical_scope: Some(LocalTermScope::new(vec![0])),
            bindings: Vec::new(),
            visible_bindings: vec![BindingId::new(0)],
            recovery: BindingContextRecovery::Normal,
        });
        assert_eq!(proof, BindingContextId::new(1));
        BindingEnv::try_new(BindingEnvParts {
            source_id: source,
            module_id: module.clone(),
            contexts,
            bindings: base.bindings().clone(),
            diagnostics: base.diagnostics().clone(),
        })
        .expect("Task258B3M1 binding env")
    }

    fn b3_symbol_env(
        source: SourceId,
        module: &ModuleId,
    ) -> (SymbolId, SourceContributionId, SymbolEnv) {
        const B3_LABEL: &str = "FormulaStatementSingleWitnessSmoke";
        let symbol = SymbolId::new(
            module.clone(),
            LocalSymbolId::new(B3_LABEL),
            FullyQualifiedName::new(format!("pkg::statement.fixture::theorem::{B3_LABEL}")),
        );
        let origin = SemanticOrigin::new(
            source,
            module.clone(),
            SourceAnchor::Range(range(source, 19, 103)),
            vec![2, 1],
        );
        let mut contributions = SourceContributionIndex::new();
        let contribution = contributions.insert(
            module.clone(),
            ContributionKind::LocalSource { source_id: source },
            SourceAnchor::Range(range(source, 0, 18)),
        );
        let namespace = NamespacePath::new(module.path().as_str());
        let mut symbols = SymbolIndex::new();
        symbols.insert(
            SymbolEntry::new(
                symbol.clone(),
                SymbolKind::Theorem,
                namespace.clone(),
                B3_LABEL,
                origin.clone(),
                contribution,
            )
            .with_visibility(Visibility::Public)
            .with_export_status(ExportStatus::Exported),
        );
        let mut definitions = DefinitionIndex::new();
        let definition = definitions.insert(
            DefinitionShell::new(
                symbol.clone(),
                DefinitionKind::Theorem,
                origin.clone(),
                contribution,
            )
            .with_visibility(Visibility::Public),
        );
        let origin_path = LabelOriginPath::new("statement.fixture.theorem.b3");
        let mut labels = LabelIndex::new();
        labels.insert(
            LabelEntry::new(
                origin_path.clone(),
                LabelKind::Theorem,
                namespace,
                B3_LABEL,
                origin,
                contribution,
            )
            .with_visibility(Visibility::Public)
            .with_export_status(ExportStatus::Exported),
        );
        contributions.add_symbol(contribution, symbol.clone());
        contributions.add_definition(contribution, definition);
        contributions.add_label(contribution, origin_path);
        (
            symbol,
            contribution,
            SymbolEnv::new(
                module.clone(),
                SymbolEnvIndexes {
                    symbols,
                    labels,
                    definitions,
                    contributions,
                    ..SymbolEnvIndexes::default()
                },
            ),
        )
    }

    fn b3n_symbol_env(
        source: SourceId,
        module: &ModuleId,
    ) -> (SymbolId, SourceContributionId, SymbolEnv) {
        const LABEL: &str = "FormulaStatementNamedWitnessSmoke";
        let symbol = SymbolId::new(
            module.clone(),
            LocalSymbolId::new(LABEL),
            FullyQualifiedName::new(format!("pkg::statement.fixture::theorem::{LABEL}")),
        );
        let origin = SemanticOrigin::new(
            source,
            module.clone(),
            SourceAnchor::Range(range(source, 19, 106)),
            vec![2, 1],
        );
        let mut contributions = SourceContributionIndex::new();
        let contribution = contributions.insert(
            module.clone(),
            ContributionKind::LocalSource { source_id: source },
            SourceAnchor::Range(range(source, 0, 18)),
        );
        let namespace = NamespacePath::new(module.path().as_str());
        let mut symbols = SymbolIndex::new();
        symbols.insert(
            SymbolEntry::new(
                symbol.clone(),
                SymbolKind::Theorem,
                namespace.clone(),
                LABEL,
                origin.clone(),
                contribution,
            )
            .with_visibility(Visibility::Public)
            .with_export_status(ExportStatus::Exported),
        );
        let mut definitions = DefinitionIndex::new();
        let definition = definitions.insert(
            DefinitionShell::new(
                symbol.clone(),
                DefinitionKind::Theorem,
                origin.clone(),
                contribution,
            )
            .with_visibility(Visibility::Public),
        );
        let origin_path = LabelOriginPath::new("statement.fixture.theorem.b3n");
        let mut labels = LabelIndex::new();
        labels.insert(
            LabelEntry::new(
                origin_path.clone(),
                LabelKind::Theorem,
                namespace,
                LABEL,
                origin,
                contribution,
            )
            .with_visibility(Visibility::Public)
            .with_export_status(ExportStatus::Exported),
        );
        contributions.add_symbol(contribution, symbol.clone());
        contributions.add_definition(contribution, definition);
        contributions.add_label(contribution, origin_path);
        (
            symbol,
            contribution,
            SymbolEnv::new(
                module.clone(),
                SymbolEnvIndexes {
                    symbols,
                    labels,
                    definitions,
                    contributions,
                    ..SymbolEnvIndexes::default()
                },
            ),
        )
    }

    fn b3m1_symbol_env(
        source: SourceId,
        module: &ModuleId,
    ) -> (SymbolId, SourceContributionId, SymbolEnv) {
        const LABEL: &str = "FormulaStatementMultipleWitnessSmoke";
        let symbol = SymbolId::new(
            module.clone(),
            LocalSymbolId::new(LABEL),
            FullyQualifiedName::new(format!("pkg::statement.fixture::theorem::{LABEL}")),
        );
        let origin = SemanticOrigin::new(
            source,
            module.clone(),
            SourceAnchor::Range(range(source, 19, 112)),
            vec![2, 1],
        );
        let mut contributions = SourceContributionIndex::new();
        let contribution = contributions.insert(
            module.clone(),
            ContributionKind::LocalSource { source_id: source },
            SourceAnchor::Range(range(source, 0, 18)),
        );
        let namespace = NamespacePath::new(module.path().as_str());
        let mut symbols = SymbolIndex::new();
        symbols.insert(
            SymbolEntry::new(
                symbol.clone(),
                SymbolKind::Theorem,
                namespace.clone(),
                LABEL,
                origin.clone(),
                contribution,
            )
            .with_visibility(Visibility::Public)
            .with_export_status(ExportStatus::Exported),
        );
        let mut definitions = DefinitionIndex::new();
        let definition = definitions.insert(
            DefinitionShell::new(
                symbol.clone(),
                DefinitionKind::Theorem,
                origin.clone(),
                contribution,
            )
            .with_visibility(Visibility::Public),
        );
        let origin_path = LabelOriginPath::new("statement.fixture.theorem.b3m1");
        let mut labels = LabelIndex::new();
        labels.insert(
            LabelEntry::new(
                origin_path.clone(),
                LabelKind::Theorem,
                namespace,
                LABEL,
                origin,
                contribution,
            )
            .with_visibility(Visibility::Public)
            .with_export_status(ExportStatus::Exported),
        );
        contributions.add_symbol(contribution, symbol.clone());
        contributions.add_definition(contribution, definition);
        contributions.add_label(contribution, origin_path);
        (
            symbol,
            contribution,
            SymbolEnv::new(
                module.clone(),
                SymbolEnvIndexes {
                    symbols,
                    labels,
                    definitions,
                    contributions,
                    ..SymbolEnvIndexes::default()
                },
            ),
        )
    }

    fn b2_node_range(index: usize) -> (usize, usize) {
        const RANGES: [(usize, usize); 55] = [
            (0, 7),
            (8, 9),
            (10, 13),
            (14, 17),
            (17, 18),
            (19, 26),
            (27, 64),
            (64, 65),
            (66, 67),
            (68, 69),
            (70, 71),
            (72, 77),
            (80, 86),
            (87, 88),
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
            (66, 67),
            (66, 67),
            (70, 71),
            (70, 71),
            (66, 71),
            (66, 71),
            (87, 88),
            (87, 88),
            (91, 92),
            (91, 92),
            (87, 92),
            (87, 92),
            (87, 92),
            (80, 93),
            (101, 102),
            (101, 102),
            (105, 106),
            (105, 106),
            (101, 106),
            (101, 106),
            (101, 106),
            (96, 107),
            (72, 111),
            (19, 112),
            (0, 112),
            (0, 112),
            (0, 112),
        ];
        RANGES[index]
    }

    fn b2_node_children(index: usize) -> Vec<usize> {
        match index {
            24 => vec![3],
            25 => vec![24],
            26 => vec![1, 2, 25],
            27 => vec![0, 26, 4],
            28 => vec![8],
            29 => vec![28],
            30 => vec![10],
            31 => vec![30],
            32 => vec![29, 9, 31],
            33 => vec![32],
            34 => vec![13],
            35 => vec![34],
            36 => vec![15],
            37 => vec![36],
            38 => vec![35, 14, 37],
            39 => vec![38],
            40 => vec![39],
            41 => vec![12, 40, 16],
            42 => vec![18],
            43 => vec![42],
            44 => vec![20],
            45 => vec![44],
            46 => vec![43, 19, 45],
            47 => vec![46],
            48 => vec![47],
            49 => vec![17, 48, 21],
            50 => vec![11, 41, 49, 22],
            51 => vec![5, 6, 7, 33, 50, 23],
            52 => vec![27, 51],
            53 => vec![52],
            54 => (0..24).chain(std::iter::once(53)).collect(),
            _ => Vec::new(),
        }
    }

    fn b2_typed_arena(source: SourceId) -> TypedArena {
        let mut builder = TypedArenaBuilder::new();
        let mut ids = Vec::with_capacity(55);
        for index in 0..55 {
            let (start, end) = b2_node_range(index);
            let kind = match index {
                28 | 30 | 34 | 36 | 42 | 44 => "source.term.variable-reference",
                32 | 38 | 46 => "source.formula.atomic.equality",
                41 => "source.statement.assumption",
                49 => "source.statement.conclusion",
                51 => "source.statement.theorem",
                _ => "source.surface.unowned",
            };
            let children = b2_node_children(index)
                .into_iter()
                .map(|child| ids[child])
                .collect();
            let id = builder
                .push(
                    TypedNode::new(kind, SourceAnchor::Range(range(source, start, end)))
                        .with_children(children),
                )
                .expect("Task258B2 typed node");
            assert_eq!(id.index(), index);
            ids.push(id);
        }
        builder.finish(Some(ids[54])).expect("Task258B2 arena")
    }

    fn b2_binding_env(source: SourceId, module: &ModuleId) -> BindingEnv {
        let base = binding_env(source, module);
        let mut contexts = base.contexts().clone();
        let proof = contexts.insert(BindingContextDraft {
            owner: BindingContextOwner::SourceStatement {
                source_range: range(source, 72, 111),
            },
            parent: Some(BindingContextId::new(0)),
            layer: BindingContextLayer::Proof,
            lexical_scope: Some(LocalTermScope::new(vec![0])),
            bindings: Vec::new(),
            visible_bindings: vec![BindingId::new(0)],
            recovery: BindingContextRecovery::Normal,
        });
        assert_eq!(proof, BindingContextId::new(1));
        BindingEnv::try_new(BindingEnvParts {
            source_id: source,
            module_id: module.clone(),
            contexts,
            bindings: base.bindings().clone(),
            diagnostics: base.diagnostics().clone(),
        })
        .expect("Task258B2 binding env")
    }

    fn b2_symbol_env(
        source: SourceId,
        module: &ModuleId,
    ) -> (SymbolId, SourceContributionId, SymbolEnv) {
        const B2_LABEL: &str = "FormulaStatementSingleAssumptionSmoke";
        let symbol = SymbolId::new(
            module.clone(),
            LocalSymbolId::new(B2_LABEL),
            FullyQualifiedName::new(format!("pkg::statement.fixture::theorem::{B2_LABEL}")),
        );
        let origin = SemanticOrigin::new(
            source,
            module.clone(),
            SourceAnchor::Range(range(source, 19, 112)),
            vec![2, 1],
        );
        let mut contributions = SourceContributionIndex::new();
        let contribution = contributions.insert(
            module.clone(),
            ContributionKind::LocalSource { source_id: source },
            SourceAnchor::Range(range(source, 0, 18)),
        );
        let namespace = NamespacePath::new(module.path().as_str());
        let mut symbols = SymbolIndex::new();
        symbols.insert(
            SymbolEntry::new(
                symbol.clone(),
                SymbolKind::Theorem,
                namespace.clone(),
                B2_LABEL,
                origin.clone(),
                contribution,
            )
            .with_visibility(Visibility::Public)
            .with_export_status(ExportStatus::Exported),
        );
        let mut definitions = DefinitionIndex::new();
        let definition = definitions.insert(
            DefinitionShell::new(
                symbol.clone(),
                DefinitionKind::Theorem,
                origin.clone(),
                contribution,
            )
            .with_visibility(Visibility::Public),
        );
        let origin_path = LabelOriginPath::new("statement.fixture.theorem.b2");
        let mut labels = LabelIndex::new();
        labels.insert(
            LabelEntry::new(
                origin_path.clone(),
                LabelKind::Theorem,
                namespace,
                B2_LABEL,
                origin,
                contribution,
            )
            .with_visibility(Visibility::Public)
            .with_export_status(ExportStatus::Exported),
        );
        contributions.add_symbol(contribution, symbol.clone());
        contributions.add_definition(contribution, definition);
        contributions.add_label(contribution, origin_path);
        (
            symbol,
            contribution,
            SymbolEnv::new(
                module.clone(),
                SymbolEnvIndexes {
                    symbols,
                    labels,
                    definitions,
                    contributions,
                    ..SymbolEnvIndexes::default()
                },
            ),
        )
    }

    fn b1_node_range(index: usize) -> (usize, usize) {
        const RANGES: [(usize, usize); 77] = [
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
            (77, 78),
            (78, 79),
            (80, 81),
            (82, 83),
            (84, 85),
            (86, 91),
            (96, 100),
            (101, 102),
            (103, 104),
            (105, 106),
            (106, 107),
            (110, 113),
            (113, 114),
            (117, 121),
            (122, 123),
            (124, 125),
            (126, 127),
            (128, 130),
            (131, 132),
            (132, 133),
            (134, 137),
            (137, 138),
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
            (80, 81),
            (80, 81),
            (84, 85),
            (84, 85),
            (80, 85),
            (80, 85),
            (77, 85),
            (101, 102),
            (101, 102),
            (105, 106),
            (105, 106),
            (101, 106),
            (101, 106),
            (101, 106),
            (96, 107),
            (86, 113),
            (77, 114),
            (122, 123),
            (122, 123),
            (126, 127),
            (126, 127),
            (122, 127),
            (122, 127),
            (122, 127),
            (131, 132),
            (131, 132),
            (128, 132),
            (117, 133),
            (69, 137),
            (19, 138),
            (0, 138),
            (0, 138),
            (0, 138),
        ];
        RANGES[index]
    }

    fn b1_node_children(index: usize) -> Vec<usize> {
        match index {
            34 => vec![3],
            35 => vec![34],
            36 => vec![1, 2, 35],
            37 => vec![0, 36, 4],
            38 => vec![8],
            39 => vec![38],
            40 => vec![10],
            41 => vec![40],
            42 => vec![39, 9, 41],
            43 => vec![42],
            44 => vec![14],
            45 => vec![44],
            46 => vec![16],
            47 => vec![46],
            48 => vec![45, 15, 47],
            49 => vec![48],
            50 => vec![12, 13, 49],
            51 => vec![19],
            52 => vec![51],
            53 => vec![21],
            54 => vec![53],
            55 => vec![52, 20, 54],
            56 => vec![55],
            57 => vec![56],
            58 => vec![18, 57, 22],
            59 => vec![17, 58, 23],
            60 => vec![50, 59, 24],
            61 => vec![26],
            62 => vec![61],
            63 => vec![28],
            64 => vec![63],
            65 => vec![62, 27, 64],
            66 => vec![65],
            67 => vec![66],
            68 => vec![30],
            69 => vec![68],
            70 => vec![29, 69],
            71 => vec![25, 67, 70, 31],
            72 => vec![11, 60, 71, 32],
            73 => vec![5, 6, 7, 43, 72, 33],
            74 => vec![37, 73],
            75 => vec![74],
            76 => (0..34).chain(std::iter::once(75)).collect(),
            _ => Vec::new(),
        }
    }

    fn b1_surface_kind(index: usize) -> syntax::SurfaceNodeKind {
        match index {
            34 => syntax::SurfaceNodeKind::TypeHead,
            35 => syntax::SurfaceNodeKind::TypeExpression,
            36 => syntax::SurfaceNodeKind::ReserveSegment,
            37 => syntax::SurfaceNodeKind::ReserveItem,
            38 | 40 | 44 | 46 | 51 | 53 | 61 | 63 => syntax::SurfaceNodeKind::TermReference,
            39 | 41 | 45 | 47 | 52 | 54 | 62 | 64 => syntax::SurfaceNodeKind::TermExpression,
            42 | 48 | 55 | 65 => syntax::SurfaceNodeKind::BuiltinPredicateApplication,
            43 | 49 | 56 | 66 => syntax::SurfaceNodeKind::FormulaExpression,
            50 | 57 | 67 => syntax::SurfaceNodeKind::Proposition,
            58 | 71 => syntax::SurfaceNodeKind::ConclusionStatement,
            59 | 72 => syntax::SurfaceNodeKind::ProofBlock,
            60 => syntax::SurfaceNodeKind::CompactStatement,
            68 => syntax::SurfaceNodeKind::Reference,
            69 => syntax::SurfaceNodeKind::ReferenceList,
            70 => syntax::SurfaceNodeKind::JustificationClause,
            73 => syntax::SurfaceNodeKind::TheoremItem,
            74 => syntax::SurfaceNodeKind::ItemList,
            75 => syntax::SurfaceNodeKind::CompilationUnit,
            _ => syntax::SurfaceNodeKind::Root,
        }
    }

    fn b1_typed_arena(source: SourceId) -> TypedArena {
        let mut builder = TypedArenaBuilder::new();
        let mut ids = Vec::with_capacity(77);
        for index in 0..77 {
            let (start, end) = b1_node_range(index);
            let kind = match index {
                38 | 40 | 44 | 46 | 51 | 53 | 61 | 63 => "source.term.variable-reference",
                42 | 48 | 55 | 65 => "source.formula.atomic.equality",
                58 | 71 => "source.statement.conclusion",
                60 => "source.statement.proof-step",
                73 => "source.statement.theorem",
                _ => "source.surface.unowned",
            };
            let children = b1_node_children(index)
                .into_iter()
                .map(|child| ids[child])
                .collect();
            let id = builder
                .push(
                    TypedNode::new(kind, SourceAnchor::Range(range(source, start, end)))
                        .with_children(children),
                )
                .expect("Task258B1 typed node");
            assert_eq!(id.index(), index);
            ids.push(id);
        }
        builder.finish(Some(ids[76])).expect("Task258B1 arena")
    }

    fn b1_resolved_arena(
        source: SourceId,
        module: &ModuleId,
        label_ref: Option<LabelRefId>,
    ) -> ResolvedArena {
        let mut builder = mizar_resolve::resolved_ast::ResolvedArenaBuilder::new();
        let mut ids = Vec::with_capacity(77);
        for index in 0..77 {
            let (start, end) = b1_node_range(index);
            let children = b1_node_children(index)
                .into_iter()
                .map(|child| ids[child])
                .collect();
            let origin = SemanticOrigin::new(
                source,
                module.clone(),
                SourceAnchor::Range(range(source, start, end)),
                vec![index as u32],
            );
            let mut row = ResolvedNode::new(b1_surface_kind(index), children, origin);
            if index == 68
                && let Some(label_ref) = label_ref
            {
                row = row
                    .with_resolution(NodeResolutionState::Resolved)
                    .with_reference_key(NodeReferenceKey::Label(label_ref));
            }
            let id = builder.push(row).expect("Task258B1 resolver node");
            assert_eq!(id.index(), index);
            ids.push(id);
        }
        builder.finish(ids[76]).expect("Task258B1 resolver arena")
    }

    fn b1_resolved_arena_with_import_edge(
        fixture: &B1Fixture,
        import: mizar_resolve::resolved_ast::ResolvedImportId,
    ) -> ResolvedArena {
        let mut builder = mizar_resolve::resolved_ast::ResolvedArenaBuilder::new();
        let mut ids = Vec::with_capacity(77);
        for index in 0..77 {
            let (start, end) = b1_node_range(index);
            let children = b1_node_children(index)
                .into_iter()
                .map(|child| ids[child])
                .collect();
            let mut origin = SemanticOrigin::new(
                fixture.source,
                fixture.module.clone(),
                SourceAnchor::Range(range(fixture.source, start, end)),
                vec![index as u32],
            );
            if index == 50 {
                origin = origin.with_import_edge(import);
            }
            let mut row = ResolvedNode::new(b1_surface_kind(index), children, origin);
            if index == 68 {
                row = row
                    .with_resolution(NodeResolutionState::Resolved)
                    .with_reference_key(NodeReferenceKey::Label(fixture.resolution.ids()[0]));
            }
            let id = builder.push(row).expect("import-edge resolver row");
            assert_eq!(id.index(), index);
            ids.push(id);
        }
        builder.finish(ids[76]).expect("import-edge resolver arena")
    }

    #[derive(Clone, Copy, PartialEq, Eq)]
    enum B1ResolverMutation {
        Root,
        Count,
        OriginSource,
        OriginModule,
        StructuralPath,
        Anchor,
        Children,
        Recovered,
        ReferenceWithoutKey,
        ExtraResolution,
    }

    fn b1_mutated_resolver_ast(fixture: &B1Fixture, mutation: B1ResolverMutation) -> ResolvedAst {
        b1_mutated_resolver_ast_result(fixture, mutation).expect("valid mutated resolver AST")
    }

    fn b1_mutated_resolver_ast_result(
        fixture: &B1Fixture,
        mutation: B1ResolverMutation,
    ) -> Result<ResolvedAst, mizar_resolve::resolved_ast::ResolvedAstError> {
        let mut builder = mizar_resolve::resolved_ast::ResolvedArenaBuilder::new();
        let mut ids = Vec::with_capacity(78);
        for (id, node) in fixture.resolver_ast.nodes().iter() {
            let index = id.index();
            let mut structural_path = node.origin().structural_path().to_vec();
            let mut anchor = node.origin().anchor().clone();
            let mut children = node
                .children()
                .iter()
                .map(|child| ids[child.index()])
                .collect::<Vec<_>>();
            if mutation == B1ResolverMutation::StructuralPath && index == 50 {
                structural_path = vec![999];
            }
            if mutation == B1ResolverMutation::Anchor && index == 50 {
                anchor = SourceAnchor::Range(range(fixture.source, 0, 0));
            }
            if mutation == B1ResolverMutation::Children && index == 71 {
                children.push(ids[58]);
            }
            let origin_source = if mutation == B1ResolverMutation::OriginSource && index == 50 {
                source_id(999)
            } else {
                fixture.source
            };
            let origin_module = if mutation == B1ResolverMutation::OriginModule && index == 50 {
                ModuleId::new(PackageId::new("pkg"), ModulePath::new("statement.other"))
            } else {
                fixture.module.clone()
            };
            let mut row = ResolvedNode::new(
                node.kind().clone(),
                children,
                SemanticOrigin::new(origin_source, origin_module, anchor, structural_path),
            );
            if mutation == B1ResolverMutation::Recovered && index == 50 {
                row = row.with_recovery(RecoveryState::Recovered);
            } else {
                row = row.with_recovery(node.recovery());
            }
            let resolution = if mutation == B1ResolverMutation::ExtraResolution && index == 67 {
                NodeResolutionState::Resolved
            } else {
                node.resolution()
            };
            row = row.with_resolution(resolution);
            if !(mutation == B1ResolverMutation::ReferenceWithoutKey && index == 68)
                && let Some(key) = node.reference_key()
            {
                row = row.with_reference_key(key);
            }
            let inserted = builder.push(row).expect("mutated resolver row");
            assert_eq!(inserted.index(), index);
            ids.push(inserted);
        }
        if mutation == B1ResolverMutation::Count {
            let extra = builder
                .push(ResolvedNode::new(
                    syntax::SurfaceNodeKind::Root,
                    Vec::new(),
                    SemanticOrigin::new(
                        fixture.source,
                        fixture.module.clone(),
                        SourceAnchor::Range(range(fixture.source, 0, 0)),
                        vec![77],
                    ),
                ))
                .expect("extra resolver row");
            assert_eq!(extra.index(), 77);
            ids.push(extra);
        }
        let root = if mutation == B1ResolverMutation::Root {
            ids[75]
        } else {
            ids[76]
        };
        ResolvedAst::try_new(
            fixture.source,
            fixture.module.clone(),
            builder.finish(root).expect("mutated resolver arena"),
            mizar_resolve::resolved_ast::NameRefTable::new(),
            fixture.resolution.table().clone(),
            mizar_resolve::resolved_ast::ResolvedImports::new(),
        )
    }

    fn b1_binding_env(source: SourceId, module: &ModuleId) -> BindingEnv {
        b1_binding_env_with_mutation(source, module, |_| {}).expect("Task258B1 binding env")
    }

    fn b1_binding_env_with_mutation(
        source: SourceId,
        module: &ModuleId,
        mutate: impl FnOnce(&mut [BindingContextDraft]),
    ) -> Result<BindingEnv, crate::binding_env::BindingEnvError> {
        let binding = BindingId::new(0);
        let mut drafts = [
            BindingContextDraft {
                owner: BindingContextOwner::Module,
                parent: None,
                layer: BindingContextLayer::Module,
                lexical_scope: None,
                bindings: vec![binding],
                visible_bindings: vec![binding],
                recovery: BindingContextRecovery::Normal,
            },
            BindingContextDraft {
                owner: BindingContextOwner::SourceStatement {
                    source_range: range(source, 69, 137),
                },
                parent: Some(BindingContextId::new(0)),
                layer: BindingContextLayer::Proof,
                lexical_scope: Some(LocalTermScope::new(vec![0])),
                bindings: Vec::new(),
                visible_bindings: vec![binding],
                recovery: BindingContextRecovery::Normal,
            },
            BindingContextDraft {
                owner: BindingContextOwner::SourceStatement {
                    source_range: range(source, 86, 113),
                },
                parent: Some(BindingContextId::new(1)),
                layer: BindingContextLayer::Proof,
                lexical_scope: Some(LocalTermScope::new(vec![0, 0])),
                bindings: Vec::new(),
                visible_bindings: vec![binding],
                recovery: BindingContextRecovery::Normal,
            },
        ];
        mutate(&mut drafts);
        let mut contexts = BindingContextTable::new();
        for draft in drafts {
            contexts.insert(draft);
        }
        let mut rows = BindingTable::new();
        rows.insert(BindingDraft {
            spelling: "x".to_owned(),
            kind: BindingKind::ReservedVariable,
            identity: BinderIdentity::ReservedVariable {
                spelling: "x".to_owned(),
                declaration_range: range(source, 8, 9),
            },
            owner_context: BindingContextId::new(0),
            declaration_range: range(source, 8, 9),
            visible_after_ordinal: 0,
            type_site: BindingTypeSite::Source(range(source, 14, 17)),
            status: BindingStatus::Reserved,
            captured: CapturedFreeVariables::default(),
            diagnostics: Vec::new(),
            recovery: BindingRecoveryState::Normal,
        });
        BindingEnv::try_new(BindingEnvParts {
            source_id: source,
            module_id: module.clone(),
            contexts,
            bindings: rows,
            diagnostics: BindingDiagnosticTable::new(),
        })
    }

    fn b1_symbol_env(
        source: SourceId,
        module: &ModuleId,
    ) -> (SymbolId, SourceContributionId, SymbolEnv) {
        const B1_LABEL: &str = "FormulaStatementNestedContextSmoke";
        let symbol = SymbolId::new(
            module.clone(),
            LocalSymbolId::new(B1_LABEL),
            FullyQualifiedName::new(format!("pkg::statement.fixture::theorem::{B1_LABEL}")),
        );
        let origin = SemanticOrigin::new(
            source,
            module.clone(),
            SourceAnchor::Range(range(source, 19, 138)),
            vec![1],
        );
        let mut contributions = SourceContributionIndex::new();
        let contribution = contributions.insert(
            module.clone(),
            ContributionKind::LocalSource { source_id: source },
            SourceAnchor::Range(range(source, 0, 18)),
        );
        let namespace = NamespacePath::new(module.path().as_str());
        let mut symbols = SymbolIndex::new();
        symbols.insert(
            SymbolEntry::new(
                symbol.clone(),
                SymbolKind::Theorem,
                namespace.clone(),
                B1_LABEL,
                origin.clone(),
                contribution,
            )
            .with_visibility(Visibility::Public)
            .with_export_status(ExportStatus::Exported),
        );
        let mut definitions = DefinitionIndex::new();
        let definition = definitions.insert(
            DefinitionShell::new(
                symbol.clone(),
                DefinitionKind::Theorem,
                origin.clone(),
                contribution,
            )
            .with_visibility(Visibility::Public),
        );
        let origin_path = LabelOriginPath::new("statement.fixture.theorem.b1");
        let mut labels = LabelIndex::new();
        labels.insert(
            LabelEntry::new(
                origin_path.clone(),
                LabelKind::Theorem,
                namespace,
                B1_LABEL,
                origin,
                contribution,
            )
            .with_visibility(Visibility::Public)
            .with_export_status(ExportStatus::Exported),
        );
        contributions.add_symbol(contribution, symbol.clone());
        contributions.add_definition(contribution, definition);
        contributions.add_label(contribution, origin_path);
        (
            symbol,
            contribution,
            SymbolEnv::new(
                module.clone(),
                SymbolEnvIndexes {
                    symbols,
                    labels,
                    definitions,
                    contributions,
                    ..SymbolEnvIndexes::default()
                },
            ),
        )
    }

    fn typed_arena(source: SourceId) -> TypedArena {
        let mut builder = TypedArenaBuilder::new();
        let left = builder
            .push(TypedNode::new(
                "source.term.variable-reference",
                SourceAnchor::Range(range(source, 74, 75)),
            ))
            .expect("left");
        let right = builder
            .push(TypedNode::new(
                "source.term.variable-reference",
                SourceAnchor::Range(range(source, 78, 79)),
            ))
            .expect("right");
        let equality_token = builder
            .push(TypedNode::new(
                "source.surface.unowned",
                SourceAnchor::Range(range(source, 76, 77)),
            ))
            .expect("equality token");
        let formula = builder
            .push(
                TypedNode::new(
                    "source.formula.atomic.equality",
                    SourceAnchor::Range(range(source, 74, 79)),
                )
                .with_children(vec![left, equality_token, right]),
            )
            .expect("formula");
        let wrapper = builder
            .push(
                TypedNode::new(
                    "source.surface.unowned",
                    SourceAnchor::Range(range(source, 74, 79)),
                )
                .with_children(vec![formula]),
            )
            .expect("formula-expression wrapper");
        let theorem = builder
            .push(
                TypedNode::new(
                    "source.statement.theorem",
                    SourceAnchor::Range(range(source, 19, 80)),
                )
                .with_children(vec![wrapper]),
            )
            .expect("theorem");
        let root = builder
            .push(
                TypedNode::new("source.module", SourceAnchor::Range(range(source, 0, 80)))
                    .with_children(vec![theorem]),
            )
            .expect("root");
        builder.finish(Some(root)).expect("arena")
    }

    fn mutate_arena(
        arena: &TypedArena,
        mut mutate: impl FnMut(TypedNodeId, &mut TypedNode),
    ) -> TypedArena {
        TypedArena::try_new(
            arena.root(),
            arena
                .iter()
                .map(|(id, row)| {
                    let mut row = row.clone();
                    mutate(id, &mut row);
                    row
                })
                .collect(),
        )
        .expect("mutated arena remains structurally valid")
    }

    fn binding_env(source: SourceId, module: &ModuleId) -> BindingEnv {
        binding_env_with_visible_after(source, module, 0)
    }

    fn binding_env_with_visible_after(
        source: SourceId,
        module: &ModuleId,
        visible_after_ordinal: usize,
    ) -> BindingEnv {
        let binding = BindingId::new(0);
        let mut contexts = BindingContextTable::new();
        contexts.insert(BindingContextDraft {
            owner: BindingContextOwner::Module,
            parent: None,
            layer: BindingContextLayer::Module,
            lexical_scope: None,
            bindings: vec![binding],
            visible_bindings: vec![binding],
            recovery: BindingContextRecovery::Normal,
        });
        let mut rows = BindingTable::new();
        rows.insert(BindingDraft {
            spelling: "x".to_owned(),
            kind: BindingKind::ReservedVariable,
            identity: BinderIdentity::ReservedVariable {
                spelling: "x".to_owned(),
                declaration_range: range(source, 8, 9),
            },
            owner_context: BindingContextId::new(0),
            declaration_range: range(source, 8, 9),
            visible_after_ordinal,
            type_site: BindingTypeSite::Source(range(source, 14, 17)),
            status: BindingStatus::Reserved,
            captured: CapturedFreeVariables::default(),
            diagnostics: Vec::new(),
            recovery: BindingRecoveryState::Normal,
        });
        BindingEnv::try_new(BindingEnvParts {
            source_id: source,
            module_id: module.clone(),
            contexts,
            bindings: rows,
            diagnostics: BindingDiagnosticTable::new(),
        })
        .expect("binding env")
    }

    fn symbol_env(
        source: SourceId,
        module: &ModuleId,
    ) -> (SymbolId, SourceContributionId, SymbolEnv) {
        symbol_env_with_mutation(source, module, None)
    }

    fn symbol_env_with_mutation(
        source: SourceId,
        module: &ModuleId,
        mutation: impl Into<Option<ResolverMutation>>,
    ) -> (SymbolId, SourceContributionId, SymbolEnv) {
        let mutation = mutation.into();
        let symbol = SymbolId::new(
            module.clone(),
            LocalSymbolId::new(LABEL),
            FullyQualifiedName::new(format!("pkg::statement.fixture::theorem::{LABEL}")),
        );
        let origin = SemanticOrigin::new(
            source,
            module.clone(),
            SourceAnchor::Range(range(source, 19, 80)),
            vec![1],
        );
        let mut contributions = SourceContributionIndex::new();
        let contribution_anchor =
            if matches!(mutation, Some(ResolverMutation::WrongContributionAnchor)) {
                SourceAnchor::Range(range(source, 19, 80))
            } else {
                SourceAnchor::Range(range(source, 0, 18))
            };
        let contribution = contributions.insert(
            module.clone(),
            ContributionKind::LocalSource { source_id: source },
            contribution_anchor,
        );
        let row_contribution = if matches!(mutation, Some(ResolverMutation::WrongContribution)) {
            contributions.insert(
                module.clone(),
                ContributionKind::LocalSource { source_id: source },
                SourceAnchor::Range(range(source, 0, 18)),
            )
        } else {
            contribution
        };
        let mut symbols = SymbolIndex::new();
        let symbol_namespace = if matches!(mutation, Some(ResolverMutation::WrongResolverNamespace))
        {
            NamespacePath::new("statement.wrong")
        } else {
            NamespacePath::new(module.path().as_str())
        };
        symbols.insert(
            SymbolEntry::new(
                symbol.clone(),
                SymbolKind::Theorem,
                symbol_namespace.clone(),
                LABEL,
                origin.clone(),
                row_contribution,
            )
            .with_visibility(Visibility::Public)
            .with_export_status(ExportStatus::Exported),
        );
        let mut definitions = DefinitionIndex::new();
        let definition = definitions.insert(
            DefinitionShell::new(
                symbol.clone(),
                DefinitionKind::Theorem,
                origin.clone(),
                row_contribution,
            )
            .with_visibility(Visibility::Public),
        );
        let label_path = LabelOriginPath::new("statement.fixture.theorem.0");
        let label_origin = match mutation {
            Some(ResolverMutation::RecoveredLabel) => origin.clone().recovered(),
            Some(ResolverMutation::StaleLabel) => SemanticOrigin::new(
                source,
                module.clone(),
                SourceAnchor::Range(range(source, 20, 80)),
                vec![1],
            ),
            Some(ResolverMutation::WrongLabelSource) => SemanticOrigin::new(
                source_id(97),
                module.clone(),
                SourceAnchor::Range(range(source_id(97), 19, 80)),
                vec![1],
            ),
            Some(ResolverMutation::WrongLabelModule) => SemanticOrigin::new(
                source,
                ModuleId::new(PackageId::new("pkg"), ModulePath::new("statement.wrong")),
                SourceAnchor::Range(range(source, 19, 80)),
                vec![1],
            ),
            _ => origin.clone(),
        };
        let label_kind = if matches!(mutation, Some(ResolverMutation::WrongLabelKind)) {
            LabelKind::ProofStep
        } else {
            LabelKind::Theorem
        };
        let label_visibility = if matches!(mutation, Some(ResolverMutation::PrivateLabel)) {
            Visibility::Private
        } else {
            Visibility::Public
        };
        let label_export = if matches!(mutation, Some(ResolverMutation::LocalOnlyLabel)) {
            ExportStatus::LocalOnly
        } else {
            ExportStatus::Exported
        };
        let label_namespace = if matches!(mutation, Some(ResolverMutation::WrongLabelNamespace)) {
            NamespacePath::new("statement.wrong")
        } else {
            symbol_namespace
        };
        let mut labels = LabelIndex::new();
        labels.insert(
            LabelEntry::new(
                label_path.clone(),
                label_kind,
                label_namespace,
                LABEL,
                label_origin,
                row_contribution,
            )
            .with_visibility(label_visibility)
            .with_export_status(label_export),
        );
        let duplicate_label_path = LabelOriginPath::new("statement.fixture.theorem.1");
        if matches!(mutation, Some(ResolverMutation::DuplicateLabel)) {
            labels.insert(
                LabelEntry::new(
                    duplicate_label_path.clone(),
                    LabelKind::Theorem,
                    NamespacePath::new(module.path().as_str()),
                    LABEL,
                    origin.clone(),
                    row_contribution,
                )
                .with_visibility(Visibility::Public)
                .with_export_status(ExportStatus::Exported),
            );
        }
        contributions.add_symbol(row_contribution, symbol.clone());
        contributions.add_definition(row_contribution, definition);
        if !matches!(mutation, Some(ResolverMutation::MissingLabelEffect)) {
            contributions.add_label(row_contribution, label_path);
            if matches!(mutation, Some(ResolverMutation::DuplicateLabel)) {
                contributions.add_label(row_contribution, duplicate_label_path);
            }
        }
        (
            symbol,
            contribution,
            SymbolEnv::new(
                module.clone(),
                SymbolEnvIndexes {
                    symbols,
                    labels,
                    definitions,
                    contributions,
                    ..SymbolEnvIndexes::default()
                },
            ),
        )
    }

    fn assemble_empty_resolved(
        typed_ast: &TypedAst,
    ) -> Result<ResolvedTypedAst, ResolvedTypedAstError> {
        assemble_resolved(typed_ast, &ClusterFactTable::new(), Vec::new(), None)
    }

    fn assemble_resolved(
        typed_ast: &TypedAst,
        cluster_facts: &ClusterFactTable,
        node_hints: Vec<ResolvedNodeKindHint>,
        statement_proofs: Option<StatementProofInputs<'_>>,
    ) -> Result<ResolvedTypedAst, ResolvedTypedAstError> {
        let collection = OverloadCollectionOutput::collect(
            Vec::<OverloadSiteInput>::new(),
            Vec::<OverloadCandidateInput>::new(),
        );
        let expansion = TemplateExpansionOutput::expand(&collection);
        let viability =
            CandidateViabilityOutput::filter(&expansion, Vec::<CandidateViabilityInput>::new());
        let specificity =
            SpecificityGraphOutput::build(&viability, Vec::<SpecificityComparisonInput>::new());
        let selection = OverloadSelectionOutput::resolve(
            &specificity,
            Vec::<OverloadSiteResolutionInput>::new(),
        );
        ResolvedTypedAst::assemble(ResolvedTypedAstInputs {
            typed_ast,
            cluster_facts,
            overload_collection: &collection,
            template_expansion: &expansion,
            viability: &viability,
            specificity: &specificity,
            overload_selection: &selection,
            expressions: Vec::new(),
            node_hints,
            statement_semantics: None,
            statement_proofs,
        })
    }

    fn assemble_resolved_with_expressions(
        typed_ast: &TypedAst,
        expressions: Vec<ExpressionMetadataInput>,
    ) -> Result<ResolvedTypedAst, ResolvedTypedAstError> {
        let cluster_facts = ClusterFactTable::new();
        let collection = OverloadCollectionOutput::collect(
            Vec::<OverloadSiteInput>::new(),
            Vec::<OverloadCandidateInput>::new(),
        );
        let expansion = TemplateExpansionOutput::expand(&collection);
        let viability =
            CandidateViabilityOutput::filter(&expansion, Vec::<CandidateViabilityInput>::new());
        let specificity =
            SpecificityGraphOutput::build(&viability, Vec::<SpecificityComparisonInput>::new());
        let selection = OverloadSelectionOutput::resolve(
            &specificity,
            Vec::<OverloadSiteResolutionInput>::new(),
        );
        ResolvedTypedAst::assemble(ResolvedTypedAstInputs {
            typed_ast,
            cluster_facts: &cluster_facts,
            overload_collection: &collection,
            template_expansion: &expansion,
            viability: &viability,
            specificity: &specificity,
            overload_selection: &selection,
            expressions,
            node_hints: Vec::new(),
            statement_semantics: None,
            statement_proofs: None,
        })
    }

    fn assemble_resolved_with_statement_semantics(
        typed_ast: &TypedAst,
        owner: &CheckedStatementOwner,
        binding_env: &BindingEnv,
        term_formula: &crate::type_checker::TermFormulaInferenceOutput,
    ) -> Result<ResolvedTypedAst, ResolvedTypedAstError> {
        let cluster_facts = ClusterFactTable::new();
        let collection = OverloadCollectionOutput::collect(
            Vec::<OverloadSiteInput>::new(),
            Vec::<OverloadCandidateInput>::new(),
        );
        let expansion = TemplateExpansionOutput::expand(&collection);
        let viability =
            CandidateViabilityOutput::filter(&expansion, Vec::<CandidateViabilityInput>::new());
        let specificity =
            SpecificityGraphOutput::build(&viability, Vec::<SpecificityComparisonInput>::new());
        let selection = OverloadSelectionOutput::resolve(
            &specificity,
            Vec::<OverloadSiteResolutionInput>::new(),
        );
        ResolvedTypedAst::assemble(ResolvedTypedAstInputs {
            typed_ast,
            cluster_facts: &cluster_facts,
            overload_collection: &collection,
            template_expansion: &expansion,
            viability: &viability,
            specificity: &specificity,
            overload_selection: &selection,
            expressions: Vec::new(),
            node_hints: Vec::new(),
            statement_semantics: Some(StatementSemanticInputs {
                owner,
                binding_env,
                term_formula,
                rows: Vec::new(),
            }),
            statement_proofs: Some(StatementProofInputs {
                owner,
                rows: Vec::new(),
            }),
        })
    }
}
