//! Syntax-free transport for proof-local declaration binding transactions.

use crate::{
    binding_env::{
        BinderIdentity, BindingContextDraft, BindingContextId, BindingContextTable, BindingDraft,
        BindingEnv, BindingEnvParts, BindingId, BindingKind, BindingLookupResult,
        BindingLookupSite,
    },
    source_statement::{
        SourceStatementHandoff, SourceStatementWitnessError, SourceStatementWitnessHandoff,
        SourceStatementWitnessId, SourceStatementWitnessNameId, SourceStatementWitnessTermTarget,
    },
    source_term::{SourcePrimaryTermHandoff, SourcePrimaryTermId},
    typed_ast::{
        NodeRecoveryState, TypedArena, TypedArenaBuilder, TypedNode, TypedNodeId, TypedNodeLinks,
        TypingState,
    },
};
use mizar_resolve::{
    names::{LocalTermBinding, LocalTermScope},
    resolved_ast::ModuleId,
};
use mizar_session::{SourceAnchor, SourceId, SourceRange};
use std::{
    error::Error,
    fmt::{self, Write as _},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SourceProofLocalDeclarationId(usize);

impl SourceProofLocalDeclarationId {
    pub const fn new(index: usize) -> Self {
        Self(index)
    }

    pub const fn index(self) -> usize {
        self.0
    }
}

/// Complete input for one proof-local declaration transaction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceProofLocalDeclarationHandoffInput {
    pub source_id: SourceId,
    pub module_id: ModuleId,
    pub declarations: Vec<SourceProofLocalDeclarationInput>,
}

/// One resolver-authenticated proof-local declaration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceProofLocalDeclarationInput {
    pub witness: SourceStatementWitnessId,
    pub name: SourceStatementWitnessNameId,
    pub rhs: SourceStatementWitnessTermTarget,
    pub binding_context: BindingContextId,
    pub source_ordinal: usize,
    pub local: LocalTermBinding,
    pub kind: SourceProofLocalDeclarationKind,
    pub recovery: SourceProofLocalDeclarationRecovery,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum SourceProofLocalDeclarationKind {
    NamedWitness,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum SourceProofLocalDeclarationRecovery {
    Normal,
}

/// One validated proof-local declaration and its checker-owned binding.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceProofLocalDeclaration {
    witness: SourceStatementWitnessId,
    name: SourceStatementWitnessNameId,
    rhs: SourceStatementWitnessTermTarget,
    binding: BindingId,
    binding_context: BindingContextId,
    source_ordinal: usize,
    visible_after_ordinal: usize,
    kind: SourceProofLocalDeclarationKind,
    recovery: SourceProofLocalDeclarationRecovery,
}

impl SourceProofLocalDeclaration {
    pub const fn witness(&self) -> SourceStatementWitnessId {
        self.witness
    }

    pub const fn name(&self) -> SourceStatementWitnessNameId {
        self.name
    }

    pub const fn rhs(&self) -> SourceStatementWitnessTermTarget {
        self.rhs
    }

    pub const fn binding(&self) -> BindingId {
        self.binding
    }

    pub const fn binding_context(&self) -> BindingContextId {
        self.binding_context
    }

    pub const fn source_ordinal(&self) -> usize {
        self.source_ordinal
    }

    pub const fn visible_after_ordinal(&self) -> usize {
        self.visible_after_ordinal
    }

    pub const fn kind(&self) -> SourceProofLocalDeclarationKind {
        self.kind
    }

    pub const fn recovery(&self) -> SourceProofLocalDeclarationRecovery {
        self.recovery
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceProofLocalDeclarationTable {
    rows: Vec<SourceProofLocalDeclaration>,
}

impl SourceProofLocalDeclarationTable {
    pub fn get(&self, id: SourceProofLocalDeclarationId) -> Option<&SourceProofLocalDeclaration> {
        self.rows.get(id.index())
    }

    pub fn iter(
        &self,
    ) -> impl Iterator<Item = (SourceProofLocalDeclarationId, &SourceProofLocalDeclaration)> {
        self.rows
            .iter()
            .enumerate()
            .map(|(index, row)| (SourceProofLocalDeclarationId::new(index), row))
    }

    pub const fn len(&self) -> usize {
        self.rows.len()
    }

    pub const fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }
}

/// Immutable proof-local declaration transaction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceProofLocalDeclarationHandoff {
    source_id: SourceId,
    module_id: ModuleId,
    base_binding_fingerprint: String,
    statement_fingerprint: String,
    witness_fingerprint: String,
    primary_term_fingerprint: String,
    binding_env: BindingEnv,
    final_binding_fingerprint: String,
    declarations: SourceProofLocalDeclarationTable,
}

impl SourceProofLocalDeclarationHandoff {
    pub const fn source_id(&self) -> SourceId {
        self.source_id
    }

    pub const fn module_id(&self) -> &ModuleId {
        &self.module_id
    }

    pub fn base_binding_fingerprint(&self) -> &str {
        &self.base_binding_fingerprint
    }

    pub fn statement_fingerprint(&self) -> &str {
        &self.statement_fingerprint
    }

    pub fn witness_fingerprint(&self) -> &str {
        &self.witness_fingerprint
    }

    pub fn primary_term_fingerprint(&self) -> &str {
        &self.primary_term_fingerprint
    }

    pub const fn binding_env(&self) -> &BindingEnv {
        &self.binding_env
    }

    pub fn final_binding_fingerprint(&self) -> &str {
        &self.final_binding_fingerprint
    }

    pub const fn declarations(&self) -> &SourceProofLocalDeclarationTable {
        &self.declarations
    }

    pub fn debug_text(&self) -> String {
        let mut output = String::from("source-proof-local-declaration-debug-v1\n");
        let _ = writeln!(
            output,
            "module: {}::{}",
            self.module_id.package().as_str(),
            self.module_id.path().as_str()
        );
        let _ = writeln!(
            output,
            "base-binding-fingerprint: {:?}",
            self.base_binding_fingerprint
        );
        let _ = writeln!(
            output,
            "statement-fingerprint: {:?}",
            self.statement_fingerprint
        );
        let _ = writeln!(
            output,
            "witness-fingerprint: {:?}",
            self.witness_fingerprint
        );
        let _ = writeln!(
            output,
            "primary-term-fingerprint: {:?}",
            self.primary_term_fingerprint
        );
        for (id, declaration) in self.declarations.iter() {
            let _ = writeln!(
                output,
                "declaration#{} kind={} witness={} name={} rhs={} binding={} context={} source_ordinal={} visible_after={} recovery={}",
                id.index(),
                declaration_kind_key(declaration.kind),
                declaration.witness.index(),
                declaration.name.index(),
                witness_target_key(declaration.rhs),
                declaration.binding.index(),
                declaration.binding_context.index(),
                declaration.source_ordinal,
                declaration.visible_after_ordinal,
                declaration_recovery_key(declaration.recovery),
            );
        }
        let _ = writeln!(
            output,
            "final-binding-fingerprint: {:?}",
            self.final_binding_fingerprint
        );
        output
    }

    #[allow(clippy::too_many_arguments)] // Rationale: replay the complete frozen lower transaction at installation.
    pub(crate) fn validate_installation(
        &self,
        source_id: SourceId,
        module_id: &ModuleId,
        statements: &SourceStatementHandoff,
        witnesses: &SourceStatementWitnessHandoff,
        primary_terms: &SourcePrimaryTermHandoff,
        arena: &TypedArena,
    ) -> Result<(), SourceProofLocalDeclarationError> {
        if self.source_id != source_id || &self.module_id != module_id {
            return Err(SourceProofLocalDeclarationError::InvalidTransaction);
        }
        validate_transaction_identity(source_id, module_id, statements, witnesses, primary_terms)?;
        if self.base_binding_fingerprint != statements.binding_env().debug_text()
            || self.statement_fingerprint != statements.debug_text()
            || self.witness_fingerprint != witnesses.debug_text()
            || self.primary_term_fingerprint != primary_terms.debug_text()
        {
            return Err(SourceProofLocalDeclarationError::DependencyMismatch);
        }

        let lower_failure = validate_lower_dependencies(
            self.source_id,
            &self.module_id,
            statements,
            witnesses,
            primary_terms,
        )?;
        if self.declarations.len() != 1 || lower_failure == Some(LowerFailure::Aggregate) {
            return Err(SourceProofLocalDeclarationError::InvalidAggregate);
        }
        let id = SourceProofLocalDeclarationId::new(0);
        let declaration = self
            .declarations
            .get(id)
            .ok_or(SourceProofLocalDeclarationError::InvalidDeclaration { declaration: id })?;
        if lower_failure == Some(LowerFailure::Declaration)
            || !exact_output_declaration(declaration)
            || !exact_output_local_fields(&self.binding_env, self.source_id)
        {
            return Err(SourceProofLocalDeclarationError::InvalidDeclaration { declaration: id });
        }
        if !exact_task258b3n_arena(self.source_id, arena) {
            return Err(SourceProofLocalDeclarationError::InvalidArena);
        }

        let expected = extend_binding_env(statements.binding_env(), &exact_local(self.source_id))?;
        if self.binding_env != expected
            || self.final_binding_fingerprint != self.binding_env.debug_text()
            || !exact_lookup_behavior(&self.binding_env)
        {
            return Err(SourceProofLocalDeclarationError::InvalidBindingEnvironment);
        }
        Ok(())
    }

    #[allow(clippy::too_many_arguments)] // Rationale: append the phase-7 owner guard to the complete frozen replay.
    pub(crate) fn validate_complete_installation(
        &self,
        source_id: SourceId,
        module_id: &ModuleId,
        statements: &SourceStatementHandoff,
        witnesses: &SourceStatementWitnessHandoff,
        primary_terms: &SourcePrimaryTermHandoff,
        arena: &TypedArena,
        installation_available: bool,
    ) -> Result<(), SourceProofLocalDeclarationError> {
        self.validate_installation(
            source_id,
            module_id,
            statements,
            witnesses,
            primary_terms,
            arena,
        )?;
        if !installation_available {
            return Err(SourceProofLocalDeclarationError::InvalidInstallation);
        }
        Ok(())
    }

    #[cfg(test)]
    pub(crate) fn set_statement_fingerprint_for_test(&mut self, value: impl Into<String>) {
        self.statement_fingerprint = value.into();
    }

    #[cfg(test)]
    pub(crate) fn set_base_binding_fingerprint_for_test(&mut self, value: impl Into<String>) {
        self.base_binding_fingerprint = value.into();
    }

    #[cfg(test)]
    pub(crate) fn set_witness_fingerprint_for_test(&mut self, value: impl Into<String>) {
        self.witness_fingerprint = value.into();
    }

    #[cfg(test)]
    pub(crate) fn set_primary_term_fingerprint_for_test(&mut self, value: impl Into<String>) {
        self.primary_term_fingerprint = value.into();
    }

    #[cfg(test)]
    pub(crate) fn set_final_binding_fingerprint_for_test(&mut self, value: impl Into<String>) {
        self.final_binding_fingerprint = value.into();
    }

    #[cfg(test)]
    pub(crate) fn truncate_declarations_for_test(&mut self, len: usize) {
        self.declarations.rows.truncate(len);
    }

    #[cfg(test)]
    pub(crate) fn duplicate_declaration_for_test(&mut self) {
        if let Some(row) = self.declarations.rows.first().cloned() {
            self.declarations.rows.push(row);
        }
    }

    #[cfg(test)]
    pub(crate) fn set_binding_for_test(
        &mut self,
        declaration: SourceProofLocalDeclarationId,
        binding: BindingId,
    ) {
        if let Some(row) = self.declarations.rows.get_mut(declaration.index()) {
            row.binding = binding;
        }
    }

    #[cfg(test)]
    pub(crate) fn corrupt_local_binding_for_test(
        &mut self,
        corruption: SourceProofLocalBindingCorruptionForTest,
    ) {
        let source_id = self.source_id;
        let binding = self
            .binding_env
            .binding_mut_for_test(BindingId::new(1))
            .expect("Task269A local binding");
        match corruption {
            SourceProofLocalBindingCorruptionForTest::Spelling => {
                binding.spelling = "z".to_owned();
            }
            SourceProofLocalBindingCorruptionForTest::Scope => {
                let BinderIdentity::ResolverLocal { scope, .. } = &mut binding.identity else {
                    panic!("Task269A resolver-local identity");
                };
                *scope = LocalTermScope::new(vec![1]);
            }
            SourceProofLocalBindingCorruptionForTest::Range => {
                let corrupted = range(source_id, 80, 82);
                binding.declaration_range = corrupted;
                let BinderIdentity::ResolverLocal {
                    declaration_range, ..
                } = &mut binding.identity
                else {
                    panic!("Task269A resolver-local identity");
                };
                *declaration_range = corrupted;
            }
            SourceProofLocalBindingCorruptionForTest::Ordinal => {
                binding.visible_after_ordinal = 2;
                let BinderIdentity::ResolverLocal { ordinal, .. } = &mut binding.identity else {
                    panic!("Task269A resolver-local identity");
                };
                *ordinal = 2;
            }
        }
    }
}

#[cfg(test)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SourceProofLocalBindingCorruptionForTest {
    Spelling,
    Scope,
    Range,
    Ordinal,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct SourceProofLocalDeclarationProducer;

impl SourceProofLocalDeclarationProducer {
    pub fn build(
        input: SourceProofLocalDeclarationHandoffInput,
        statements: &SourceStatementHandoff,
        witnesses: &SourceStatementWitnessHandoff,
        primary_terms: &SourcePrimaryTermHandoff,
        arena: &TypedArena,
    ) -> Result<SourceProofLocalDeclarationHandoff, SourceProofLocalDeclarationError> {
        validate_transaction_identity(
            input.source_id,
            &input.module_id,
            statements,
            witnesses,
            primary_terms,
        )?;
        let lower_failure = validate_lower_dependencies(
            input.source_id,
            &input.module_id,
            statements,
            witnesses,
            primary_terms,
        )?;
        if input.declarations.len() != 1 || lower_failure == Some(LowerFailure::Aggregate) {
            return Err(SourceProofLocalDeclarationError::InvalidAggregate);
        }
        let id = SourceProofLocalDeclarationId::new(0);
        let declaration = &input.declarations[0];
        if lower_failure == Some(LowerFailure::Declaration)
            || !exact_input_declaration(declaration, witnesses)
        {
            return Err(SourceProofLocalDeclarationError::InvalidDeclaration { declaration: id });
        }
        if !exact_task258b3n_arena(input.source_id, arena) {
            return Err(SourceProofLocalDeclarationError::InvalidArena);
        }

        let binding_env = extend_binding_env(statements.binding_env(), &declaration.local)?;
        if !exact_lookup_behavior(&binding_env) {
            return Err(SourceProofLocalDeclarationError::InvalidBindingEnvironment);
        }
        let final_binding_fingerprint = binding_env.debug_text();
        Ok(SourceProofLocalDeclarationHandoff {
            source_id: input.source_id,
            module_id: input.module_id,
            base_binding_fingerprint: statements.binding_env().debug_text(),
            statement_fingerprint: statements.debug_text(),
            witness_fingerprint: witnesses.debug_text(),
            primary_term_fingerprint: primary_terms.debug_text(),
            binding_env,
            final_binding_fingerprint,
            declarations: SourceProofLocalDeclarationTable {
                rows: vec![SourceProofLocalDeclaration {
                    witness: declaration.witness,
                    name: declaration.name,
                    rhs: declaration.rhs,
                    binding: BindingId::new(1),
                    binding_context: declaration.binding_context,
                    source_ordinal: declaration.source_ordinal,
                    visible_after_ordinal: declaration.local.visible_after_ordinal(),
                    kind: declaration.kind,
                    recovery: declaration.recovery,
                }],
            },
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum SourceProofLocalDeclarationError {
    InvalidTransaction,
    DependencyMismatch,
    InvalidAggregate,
    InvalidDeclaration {
        declaration: SourceProofLocalDeclarationId,
    },
    InvalidArena,
    InvalidBindingEnvironment,
    InvalidInstallation,
}

impl fmt::Display for SourceProofLocalDeclarationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidTransaction => {
                formatter.write_str("source proof-local declaration transaction is invalid")
            }
            Self::DependencyMismatch => {
                formatter.write_str("source proof-local declaration dependency mismatch")
            }
            Self::InvalidAggregate => {
                formatter.write_str("source proof-local declaration aggregate is invalid")
            }
            Self::InvalidDeclaration { declaration } => write!(
                formatter,
                "source proof-local declaration {} is invalid",
                declaration.index()
            ),
            Self::InvalidArena => {
                formatter.write_str("source proof-local declaration arena is invalid")
            }
            Self::InvalidBindingEnvironment => {
                formatter.write_str("source proof-local declaration binding environment is invalid")
            }
            Self::InvalidInstallation => {
                formatter.write_str("source proof-local declaration installation is invalid")
            }
        }
    }
}

impl Error for SourceProofLocalDeclarationError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LowerFailure {
    Aggregate,
    Declaration,
}

fn validate_transaction_identity(
    source_id: SourceId,
    module_id: &ModuleId,
    statements: &SourceStatementHandoff,
    witnesses: &SourceStatementWitnessHandoff,
    primary_terms: &SourcePrimaryTermHandoff,
) -> Result<(), SourceProofLocalDeclarationError> {
    if statements.source_id() != source_id
        || statements.module_id() != module_id
        || witnesses.source_id() != source_id
        || witnesses.module_id() != module_id
        || primary_terms.source_id() != source_id
        || primary_terms.module_id() != module_id
    {
        return Err(SourceProofLocalDeclarationError::InvalidTransaction);
    }
    Ok(())
}

fn validate_lower_dependencies(
    source_id: SourceId,
    module_id: &ModuleId,
    statements: &SourceStatementHandoff,
    witnesses: &SourceStatementWitnessHandoff,
    primary_terms: &SourcePrimaryTermHandoff,
) -> Result<Option<LowerFailure>, SourceProofLocalDeclarationError> {
    if !statements.is_task_258b3n_profile()
        || statements.binding_env().source_id() != source_id
        || statements.binding_env().module_id() != module_id
        || witnesses.statement_fingerprint() != statements.debug_text()
        || witnesses.primary_term_fingerprint() != primary_terms.debug_text()
    {
        return Err(SourceProofLocalDeclarationError::DependencyMismatch);
    }
    let canonical_arena = task258b3n_canonical_arena(source_id);
    match witnesses.validate_installation(
        source_id,
        module_id,
        statements,
        primary_terms,
        &canonical_arena,
    ) {
        Ok(()) => Ok(None),
        Err(SourceStatementWitnessError::DependencyMismatch) => {
            Err(SourceProofLocalDeclarationError::DependencyMismatch)
        }
        Err(SourceStatementWitnessError::InvalidAggregate) => Ok(Some(LowerFailure::Aggregate)),
        Err(
            SourceStatementWitnessError::InvalidWitness { .. }
            | SourceStatementWitnessError::InvalidName { .. },
        ) => Ok(Some(LowerFailure::Declaration)),
    }
}

fn exact_input_declaration(
    declaration: &SourceProofLocalDeclarationInput,
    witnesses: &SourceStatementWitnessHandoff,
) -> bool {
    let witness = witnesses.witnesses().get(declaration.witness);
    let name = witnesses.names().get(declaration.name);
    witness.is_some_and(|witness| {
        witness.name() == Some(declaration.name)
            && witness.binding_context() == declaration.binding_context
            && witness.term() == declaration.rhs
            && witness.source_ordinal() == declaration.source_ordinal
    }) && name.is_some_and(|name| name.witness() == declaration.witness)
        && declaration.witness == SourceStatementWitnessId::new(0)
        && declaration.name == SourceStatementWitnessNameId::new(0)
        && declaration.rhs == SourceStatementWitnessTermTarget::Primary(SourcePrimaryTermId::new(2))
        && declaration.binding_context == BindingContextId::new(1)
        && declaration.source_ordinal == 1
        && exact_local_fields(&declaration.local)
        && declaration.local.declaration_range().source_id == witnesses.source_id()
        && declaration.kind == SourceProofLocalDeclarationKind::NamedWitness
        && declaration.recovery == SourceProofLocalDeclarationRecovery::Normal
}

fn exact_output_declaration(declaration: &SourceProofLocalDeclaration) -> bool {
    declaration.witness == SourceStatementWitnessId::new(0)
        && declaration.name == SourceStatementWitnessNameId::new(0)
        && declaration.rhs == SourceStatementWitnessTermTarget::Primary(SourcePrimaryTermId::new(2))
        && declaration.binding == BindingId::new(1)
        && declaration.binding_context == BindingContextId::new(1)
        && declaration.source_ordinal == 1
        && declaration.visible_after_ordinal == 1
        && declaration.kind == SourceProofLocalDeclarationKind::NamedWitness
        && declaration.recovery == SourceProofLocalDeclarationRecovery::Normal
}

fn exact_output_local_fields(binding_env: &BindingEnv, source_id: SourceId) -> bool {
    binding_env
        .bindings()
        .get(BindingId::new(1))
        .is_some_and(|binding| {
            binding.spelling == "y"
                && binding.kind == BindingKind::LocalAbbreviation
                && binding.owner_context == BindingContextId::new(1)
                && binding.declaration_range == range(source_id, 81, 82)
                && binding.visible_after_ordinal == 1
                && matches!(
                    &binding.identity,
                    BinderIdentity::ResolverLocal {
                        scope,
                        ordinal: 1,
                        declaration_range,
                    } if scope.path() == [0]
                        && *declaration_range == range(source_id, 81, 82)
                )
        })
}

fn exact_local_fields(local: &LocalTermBinding) -> bool {
    local.spelling() == "y"
        && local.scope().path() == [0]
        && local.declaration_range().start == 81
        && local.declaration_range().end == 82
        && local.visible_after_ordinal() == 1
}

fn exact_local(source_id: SourceId) -> LocalTermBinding {
    LocalTermBinding::new(
        "y",
        LocalTermScope::new(vec![0]),
        range(source_id, 81, 82),
        1,
    )
}

fn extend_binding_env(
    base: &BindingEnv,
    local: &LocalTermBinding,
) -> Result<BindingEnv, SourceProofLocalDeclarationError> {
    if !exact_local_fields(local) || local.declaration_range().source_id != base.source_id() {
        return Err(SourceProofLocalDeclarationError::InvalidBindingEnvironment);
    }
    let mut bindings = base.bindings().clone();
    let binding = bindings.insert(BindingDraft::from_local_term(
        BindingContextId::new(1),
        BindingKind::LocalAbbreviation,
        local,
    ));
    if binding != BindingId::new(1) {
        return Err(SourceProofLocalDeclarationError::InvalidBindingEnvironment);
    }

    let mut contexts = BindingContextTable::new();
    for (id, context) in base.contexts().iter() {
        let mut owned = context.bindings.clone();
        let mut visible = context.visible_bindings.clone();
        if id == BindingContextId::new(1) {
            owned.push(binding);
            visible.push(binding);
        }
        let inserted = contexts.insert(BindingContextDraft {
            owner: context.owner.clone(),
            parent: context.parent,
            layer: context.layer,
            lexical_scope: context.lexical_scope.clone(),
            bindings: owned,
            visible_bindings: visible,
            recovery: context.recovery,
        });
        if inserted != id {
            return Err(SourceProofLocalDeclarationError::InvalidBindingEnvironment);
        }
    }
    BindingEnv::try_new(BindingEnvParts {
        source_id: base.source_id(),
        module_id: base.module_id().clone(),
        contexts,
        bindings,
        diagnostics: base.diagnostics().clone(),
    })
    .map_err(|_| SourceProofLocalDeclarationError::InvalidBindingEnvironment)
}

fn exact_lookup_behavior(binding_env: &BindingEnv) -> bool {
    let definition_site = BindingLookupSite::new(
        "y",
        BindingContextId::new(1),
        Some(LocalTermScope::new(vec![0])),
        1,
    );
    let later_site = BindingLookupSite::new(
        "y",
        BindingContextId::new(1),
        Some(LocalTermScope::new(vec![0])),
        2,
    );
    matches!(
        binding_env.lookup(&definition_site),
        Ok(BindingLookupResult::ForwardReference { candidates, .. })
            if candidates == [BindingId::new(1)]
    ) && binding_env.lookup(&later_site) == Ok(BindingLookupResult::Local(BindingId::new(1)))
}

fn declaration_kind_key(kind: SourceProofLocalDeclarationKind) -> &'static str {
    match kind {
        SourceProofLocalDeclarationKind::NamedWitness => "named-witness",
    }
}

fn declaration_recovery_key(recovery: SourceProofLocalDeclarationRecovery) -> &'static str {
    match recovery {
        SourceProofLocalDeclarationRecovery::Normal => "normal",
    }
}

fn witness_target_key(target: SourceStatementWitnessTermTarget) -> String {
    match target {
        SourceStatementWitnessTermTarget::Primary(term) => format!("primary#{}", term.index()),
        _ => "unsupported".to_owned(),
    }
}

fn range(source_id: SourceId, start: usize, end: usize) -> SourceRange {
    SourceRange {
        source_id,
        start,
        end,
    }
}

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

fn exact_task258b3n_arena(source_id: SourceId, arena: &TypedArena) -> bool {
    arena.len() == TASK258B3N_NODE_RANGES.len()
        && arena.root() == Some(TypedNodeId::new(50))
        && TASK258B3N_NODE_RANGES
            .iter()
            .copied()
            .enumerate()
            .all(|(index, (start, end))| {
                arena.node(TypedNodeId::new(index)).is_some_and(|node| {
                    node.anchor == SourceAnchor::Range(range(source_id, start, end))
                        && node.kind.as_str() == task258b3n_node_kind(index)
                        && node.resolved_node.is_none()
                        && node.recovery == NodeRecoveryState::Normal
                        && node.typing == TypingState::Unknown
                        && node.links == TypedNodeLinks::default()
                        && node
                            .children
                            .iter()
                            .map(|child| child.index())
                            .eq(task258b3n_node_children(index).iter().copied())
                })
            })
}

fn task258b3n_canonical_arena(source_id: SourceId) -> TypedArena {
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
                    SourceAnchor::Range(range(source_id, start, end)),
                )
                .with_children(children),
            )
            .expect("frozen Task258B3N node graph is valid");
        ids.push(id);
    }
    builder
        .finish(Some(ids[50]))
        .expect("frozen Task258B3N arena is valid")
}
