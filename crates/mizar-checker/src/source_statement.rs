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
    resolved_ast::{LabelKind, ModuleId, RecoveryState, SymbolId},
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
        validate_dependencies(
            source_id,
            module_id,
            &self.binding_env,
            primary_terms,
            atomic_formulas,
            arena,
        )?;
        validate_aggregate_lengths(
            self.owners.len(),
            self.statements.len(),
            self.contexts.len(),
            self.input_facts.len(),
            self.candidate_facts.len(),
        )?;
        validate_owner_rows(
            self.source_id,
            &self.module_id,
            &self.owners,
            &self.checked_owner,
            self.owner_contribution,
            arena,
        )?;
        validate_statement_rows(
            self.source_id,
            &self.statements,
            &self.owners,
            &self.contexts,
            atomic_formulas,
            arena,
        )?;
        validate_context_rows(
            self.source_id,
            &self.contexts,
            &self.statements,
            &self.binding_env,
        )?;
        validate_input_fact_rows(
            &self.input_facts,
            &self.statements,
            &self.contexts,
            &self.binding_env,
            primary_terms,
        )?;
        validate_candidate_fact_rows(
            &self.candidate_facts,
            &self.statements,
            &self.contexts,
            atomic_formulas,
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
        validate_dependencies(
            input.source_id,
            &input.module_id,
            bindings,
            primary_terms,
            atomic_formulas,
            arena,
        )?;
        validate_aggregate_lengths(
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
            input.source_id,
            &statements,
            &owners,
            &contexts,
            atomic_formulas,
            arena,
        )?;
        validate_context_rows(input.source_id, &contexts, &statements, bindings)?;
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
        validate_candidate_fact_rows(&candidate_facts, &statements, &contexts, atomic_formulas)?;
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

/// Atomic Task-258A source-statement failure.
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

fn validate_dependencies(
    source_id: SourceId,
    module_id: &ModuleId,
    bindings: &BindingEnv,
    primary_terms: &SourcePrimaryTermHandoff,
    atomic_formulas: &SourceAtomicFormulaHandoff,
    arena: &TypedArena,
) -> Result<(), SourceStatementError> {
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
    if !exact_binding_profile(source_id, bindings)
        || !exact_primary_profile(primary_terms)
        || !exact_atomic_profile(atomic_formulas, primary_terms, arena)
    {
        return Err(SourceStatementError::DependencyMismatch);
    }
    Ok(())
}

fn exact_binding_profile(source_id: SourceId, bindings: &BindingEnv) -> bool {
    if bindings.contexts().len() != 1
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

fn exact_primary_profile(primary_terms: &SourcePrimaryTermHandoff) -> bool {
    if primary_terms.terms().len() != 2
        || primary_terms.references().len() != 2
        || !primary_terms.numeric_type_requests().is_empty()
    {
        return false;
    }
    for (index, expected_range) in [(0, (74, 75)), (1, (78, 79))] {
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
            || term.context() != BindingContextId::new(0)
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
            || reference.lexical_scope().is_some()
            || reference.use_ordinal() != 1
        {
            return false;
        }
    }
    true
}

fn exact_atomic_profile(
    atomic_formulas: &SourceAtomicFormulaHandoff,
    primary_terms: &SourcePrimaryTermHandoff,
    arena: &TypedArena,
) -> bool {
    if atomic_formulas.formulas().len() != 1
        || !atomic_formulas.wrappers().is_empty()
        || !atomic_formulas.predicate_segments().is_empty()
        || !atomic_formulas.predicate_heads().is_empty()
        || !atomic_formulas.candidates().is_empty()
        || !atomic_formulas.type_sites().is_empty()
        || !atomic_formulas.attributes().is_empty()
        || atomic_formulas.edges().len() != 2
        || atomic_formulas.requests().len() != 2
    {
        return false;
    }
    let Some(formula) = atomic_formulas
        .formulas()
        .get(SourceAtomicFormulaId::new(0))
    else {
        return false;
    };
    if formula.source_range() != range(atomic_formulas.source_id(), 74, 79)
        || formula.source_ordinal() != 0
        || formula.context() != BindingContextId::new(0)
        || formula.recovery() != SourceAtomicFormulaRecovery::Normal
        || formula.spelling() != "x = x"
        || formula.kind() != SourceAtomicFormulaKind::Equality
    {
        return false;
    }
    let expected_edges = [
        (
            SourceAtomicEdgeRole::BuiltinLeftOperand,
            SourceAtomicTermTarget::Primary(SourcePrimaryTermId::new(0)),
        ),
        (
            SourceAtomicEdgeRole::BuiltinRightOperand,
            SourceAtomicTermTarget::Primary(SourcePrimaryTermId::new(1)),
        ),
    ];
    for (index, (role, target)) in expected_edges.into_iter().enumerate() {
        let Some(edge) = atomic_formulas
            .edges()
            .get(crate::source_atomic_formula::SourceAtomicEdgeId::new(index))
        else {
            return false;
        };
        if edge.formula() != SourceAtomicFormulaId::new(0)
            || edge.ordinal() != index
            || edge.role() != role
            || edge.target() != target
        {
            return false;
        }
        let Some(request) = atomic_formulas.requests().get(
            crate::source_atomic_formula::SourceAtomicRequestId::new(index),
        ) else {
            return false;
        };
        if request.formula() != SourceAtomicFormulaId::new(0)
            || request.ordinal() != index
            || request.kind() != SourceAtomicRequestKind::OperandExpectedType
            || request.edge() != Some(crate::source_atomic_formula::SourceAtomicEdgeId::new(index))
            || request.candidate().is_some()
            || request.type_site().is_some()
            || request.attribute().is_some()
        {
            return false;
        }
    }
    if arena.node(formula.site().node()).is_none() {
        return false;
    }
    let Some(left) = primary_terms.terms().get(SourcePrimaryTermId::new(0)) else {
        return false;
    };
    let Some(right) = primary_terms.terms().get(SourcePrimaryTermId::new(1)) else {
        return false;
    };
    match (
        containing_child_position(arena, formula.site().node(), left.site().node()),
        containing_child_position(arena, formula.site().node(), right.site().node()),
    ) {
        (Some(left_position), Some(right_position)) => left_position < right_position,
        _ => false,
    }
}

fn validate_aggregate_lengths(
    owners: usize,
    statements: usize,
    contexts: usize,
    input_facts: usize,
    candidate_facts: usize,
) -> Result<(), SourceStatementError> {
    if (owners, statements, contexts, input_facts, candidate_facts) != (1, 1, 1, 1, 1) {
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
    if owner.symbol != *checked_owner.symbol()
        || owner.contribution != authenticated_contribution
        || owner.source_range != range(source_id, 19, 80)
        || owner.source_range != checked_owner.source_range()
        || checked_owner.origin().source_id() != source_id
        || checked_owner.origin().module_id() != module_id
        || checked_owner.origin().anchor() != &SourceAnchor::Range(owner.source_range)
        || checked_owner.origin().import_edge().is_some()
        || checked_owner.origin().is_recovered()
        || checked_owner.visibility() != Visibility::Public
        || checked_owner.export_status() != ExportStatus::Exported
        || owner.spelling != "FormulaStatementReservedVariableEqualitySmoke"
        || owner.role != SourceTheoremRole::Theorem
        || owner.status != SourceTheoremStatus::Unmodified
        || owner.recovery != SourceStatementRecovery::Normal
        || !validate_theorem_site(&owner.site, owner.source_range, arena)
    {
        return Err(SourceStatementError::InvalidOwner { owner: id });
    }
    Ok(())
}

fn validate_statement_rows(
    source_id: SourceId,
    statements: &SourceStatementTable,
    owners: &SourceTheoremOwnerTable,
    contexts: &SourceStatementContextTable,
    atomic_formulas: &SourceAtomicFormulaHandoff,
    arena: &TypedArena,
) -> Result<(), SourceStatementError> {
    let id = SourceStatementId::new(0);
    let Some(statement) = statements.get(id) else {
        return Err(SourceStatementError::InvalidAggregate);
    };
    let expected_formula = SourceStatementFormulaTarget::Atomic(SourceAtomicFormulaId::new(0));
    let formula_site = atomic_formulas
        .formulas()
        .get(SourceAtomicFormulaId::new(0))
        .map(|formula| formula.site().node());
    let theorem_node = arena.node(statement.site.node());
    if statement.owner != SourceTheoremOwnerId::new(0)
        || owners.get(statement.owner).is_none()
        || statement.context != SourceStatementContextId::new(0)
        || contexts.get(statement.context).is_none()
        || statement.formula != expected_formula
        || statement.site != owners.get(SourceTheoremOwnerId::new(0)).unwrap().site
        || statement.source_range != range(source_id, 19, 80)
        || statement.source_ordinal != 0
        || statement.spelling != "theorem FormulaStatementReservedVariableEqualitySmoke : x = x ;"
        || statement.kind != SourceStatementKind::TheoremProposition
        || statement.recovery != SourceStatementRecovery::Normal
        || formula_site.is_none()
        || theorem_node.is_none_or(|node| {
            !valid_statement_formula_path(arena, statement.site.node(), formula_site.unwrap())
                || node.children.iter().any(|child| {
                    subtree_contains_excluded_statement_owner(arena, *child, formula_site.unwrap())
                })
        })
    {
        return Err(SourceStatementError::InvalidStatement { statement: id });
    }
    Ok(())
}

fn validate_context_rows(
    source_id: SourceId,
    contexts: &SourceStatementContextTable,
    statements: &SourceStatementTable,
    bindings: &BindingEnv,
) -> Result<(), SourceStatementError> {
    let id = SourceStatementContextId::new(0);
    let Some(context) = contexts.get(id) else {
        return Err(SourceStatementError::InvalidAggregate);
    };
    let binding_context = bindings.contexts().get(BindingContextId::new(0));
    if context.statement != SourceStatementId::new(0)
        || statements.get(context.statement).is_none()
        || context.binding_context != BindingContextId::new(0)
        || context.source_range != range(source_id, 19, 80)
        || context.visible_bindings != [BindingId::new(0)]
        || binding_context.is_none_or(|row| row.visible_bindings != context.visible_bindings)
    {
        return Err(SourceStatementError::InvalidContext { context: id });
    }
    Ok(())
}

fn validate_input_fact_rows(
    input_facts: &SourceStatementInputFactTable,
    statements: &SourceStatementTable,
    contexts: &SourceStatementContextTable,
    bindings: &BindingEnv,
    primary_terms: &SourcePrimaryTermHandoff,
) -> Result<(), SourceStatementError> {
    let id = SourceStatementInputFactId::new(0);
    let Some(fact) = input_facts.get(id) else {
        return Err(SourceStatementError::InvalidAggregate);
    };
    let expected_uses = [
        SourcePrimaryTermReferenceId::new(0),
        SourcePrimaryTermReferenceId::new(1),
    ];
    if fact.statement != SourceStatementId::new(0)
        || statements.get(fact.statement).is_none()
        || fact.context != SourceStatementContextId::new(0)
        || contexts.get(fact.context).is_none()
        || fact.ordinal != 0
        || fact.kind != SourceStatementInputFactKind::ReservedTypeGuard
        || fact.binding != BindingId::new(0)
        || bindings.bindings().get(fact.binding).is_none()
        || fact.uses != expected_uses
    {
        return Err(SourceStatementError::InvalidInputFact { fact: id });
    }
    for (index, use_id) in expected_uses.into_iter().enumerate() {
        let Some(reference) = primary_terms.references().get(use_id) else {
            return Err(SourceStatementError::InvalidInputFact { fact: id });
        };
        if reference.term() != SourcePrimaryTermId::new(index)
            || reference.binding() != fact.binding
            || reference.role() != SourcePrimaryTermReferenceRole::Variable
            || reference.use_ordinal() != 1
        {
            return Err(SourceStatementError::InvalidInputFact { fact: id });
        }
    }
    Ok(())
}

fn validate_candidate_fact_rows(
    candidate_facts: &SourceStatementCandidateFactTable,
    statements: &SourceStatementTable,
    contexts: &SourceStatementContextTable,
    atomic_formulas: &SourceAtomicFormulaHandoff,
) -> Result<(), SourceStatementError> {
    let id = SourceStatementCandidateFactId::new(0);
    let Some(fact) = candidate_facts.get(id) else {
        return Err(SourceStatementError::InvalidAggregate);
    };
    let formula = SourceStatementFormulaTarget::Atomic(SourceAtomicFormulaId::new(0));
    if fact.statement != SourceStatementId::new(0)
        || statements.get(fact.statement).is_none()
        || fact.context != SourceStatementContextId::new(0)
        || contexts.get(fact.context).is_none()
        || fact.ordinal != 0
        || fact.kind != SourceStatementCandidateFactKind::UnverifiedProposition
        || fact.formula != formula
        || statements
            .get(fact.statement)
            .is_none_or(|statement| statement.formula != fact.formula)
        || atomic_formulas
            .formulas()
            .get(SourceAtomicFormulaId::new(0))
            .is_none()
    {
        return Err(SourceStatementError::InvalidCandidateFact { fact: id });
    }
    Ok(())
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
    arena: &TypedArena,
    theorem: crate::typed_ast::TypedNodeId,
    formula: crate::typed_ast::TypedNodeId,
) -> bool {
    let Some(theorem) = arena.node(theorem) else {
        return false;
    };
    let wrappers = theorem
        .children
        .iter()
        .filter_map(|child| {
            let wrapper = arena.node(*child)?;
            (wrapper.kind.as_str() == "source.surface.unowned"
                && wrapper.anchor == arena.node(formula)?.anchor
                && wrapper.recovery == NodeRecoveryState::Normal
                && wrapper.children.as_slice() == [formula])
            .then_some(*child)
        })
        .collect::<Vec<_>>();
    wrappers.len() == 1
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
            CandidateViabilityInput, CandidateViabilityOutput, OverloadCandidateInput,
            OverloadCollectionOutput, OverloadSelectionOutput, OverloadSiteInput,
            OverloadSiteResolutionInput, SpecificityComparisonInput, SpecificityGraphOutput,
            TemplateExpansionOutput,
        },
        resolved_typed_ast::{
            ResolvedNodeKindHint, ResolvedNodeKindHintKind, ResolvedTypedAst,
            ResolvedTypedAstError, ResolvedTypedAstInputs, SourceNodeRole, StatementProofInputs,
        },
        source_atomic_formula::{
            SourceAtomicEdgeId, SourceAtomicEdgeInput, SourceAtomicFormulaHandoffInput,
            SourceAtomicFormulaInput, SourceAtomicFormulaProducer, SourceAtomicRequestInput,
        },
        source_term::{
            SourcePrimaryTermHandoffInput, SourcePrimaryTermInput, SourcePrimaryTermProducer,
            SourcePrimaryTermReferenceInput,
        },
        typed_ast::{
            CoercionTable, FactProvenance, FactStatus, InitialObligationTable,
            LocalTypeContextTable, Polarity, TypeDiagnosticTable, TypeFactDraft, TypeFactTable,
            TypePredicateRef, TypeRuleId, TypeTable, TypedArenaBuilder, TypedAst, TypedAstError,
            TypedAstParts, TypedNode, TypedNodeId,
        },
    };
    use mizar_resolve::{
        env::{
            DefinitionIndex, DefinitionKind, DefinitionShell, LabelEntry, LabelIndex,
            NamespacePath, SourceContributionIndex, SymbolEntry, SymbolEnvIndexes, SymbolIndex,
            SymbolKind,
        },
        resolved_ast::{FullyQualifiedName, LabelOriginPath, LocalSymbolId, SemanticOrigin},
    };
    use mizar_session::{
        BuildSnapshotId, InMemorySessionIdAllocator, ModulePath, PackageId, SessionIdAllocator as _,
    };

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
}
