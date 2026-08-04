//! Syntax-free transport for proof-local declaration binding transactions.

use crate::{
    binding_env::{
        BinderIdentity, BindingContextDraft, BindingContextId, BindingContextLayer,
        BindingContextOwner, BindingContextRecovery, BindingContextTable, BindingDraft, BindingEnv,
        BindingEnvParts, BindingId, BindingKind, BindingLookupResult, BindingLookupSite,
        BindingRecoveryState, BindingStatus, BindingTypeSite,
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
    env::{DefinitionId, SourceContributionId},
    names::{LocalTermBinding, LocalTermScope},
    resolved_ast::{ModuleId, SymbolId},
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

        let lower = validate_lower_dependencies(
            self.source_id,
            &self.module_id,
            statements,
            witnesses,
            primary_terms,
        )?;
        if self.declarations.len() != 1 || lower.failure == Some(LowerFailure::Aggregate) {
            return Err(SourceProofLocalDeclarationError::InvalidAggregate);
        }
        let id = SourceProofLocalDeclarationId::new(0);
        let declaration = self
            .declarations
            .get(id)
            .ok_or(SourceProofLocalDeclarationError::InvalidDeclaration { declaration: id })?;
        if lower.failure == Some(LowerFailure::Declaration)
            || !exact_output_declaration(declaration)
            || !exact_output_local_fields(&self.binding_env, self.source_id, lower.profile)
        {
            return Err(SourceProofLocalDeclarationError::InvalidDeclaration { declaration: id });
        }
        if !exact_frozen_arena(lower.profile, self.source_id, arena) {
            return Err(SourceProofLocalDeclarationError::InvalidArena);
        }

        let expected = extend_binding_env(
            statements.binding_env(),
            &exact_local(self.source_id, lower.profile),
            lower.profile,
        )?;
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
            .expect("Task269 local binding");
        match corruption {
            SourceProofLocalBindingCorruptionForTest::Spelling => {
                binding.spelling = "z".to_owned();
            }
            SourceProofLocalBindingCorruptionForTest::Scope => {
                let BinderIdentity::ResolverLocal { scope, .. } = &mut binding.identity else {
                    panic!("Task269 resolver-local identity");
                };
                *scope = LocalTermScope::new(vec![1]);
            }
            SourceProofLocalBindingCorruptionForTest::Range => {
                let corrupted = range(
                    source_id,
                    binding.declaration_range.start.saturating_sub(1),
                    binding.declaration_range.end,
                );
                binding.declaration_range = corrupted;
                let BinderIdentity::ResolverLocal {
                    declaration_range, ..
                } = &mut binding.identity
                else {
                    panic!("Task269 resolver-local identity");
                };
                *declaration_range = corrupted;
            }
            SourceProofLocalBindingCorruptionForTest::Ordinal => {
                binding.visible_after_ordinal = 2;
                let BinderIdentity::ResolverLocal { ordinal, .. } = &mut binding.identity else {
                    panic!("Task269 resolver-local identity");
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
        let lower = validate_lower_dependencies(
            input.source_id,
            &input.module_id,
            statements,
            witnesses,
            primary_terms,
        )?;
        if input.declarations.len() != 1 || lower.failure == Some(LowerFailure::Aggregate) {
            return Err(SourceProofLocalDeclarationError::InvalidAggregate);
        }
        let id = SourceProofLocalDeclarationId::new(0);
        let declaration = &input.declarations[0];
        if lower.failure == Some(LowerFailure::Declaration)
            || !exact_input_declaration(declaration, witnesses, lower.profile)
        {
            return Err(SourceProofLocalDeclarationError::InvalidDeclaration { declaration: id });
        }
        if !exact_frozen_arena(lower.profile, input.source_id, arena) {
            return Err(SourceProofLocalDeclarationError::InvalidArena);
        }

        let binding_env =
            extend_binding_env(statements.binding_env(), &declaration.local, lower.profile)?;
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FrozenProfile {
    Task258B3N,
    Task258B3M1,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct LowerValidation {
    profile: FrozenProfile,
    failure: Option<LowerFailure>,
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
) -> Result<LowerValidation, SourceProofLocalDeclarationError> {
    let profile = if statements.is_task_258b3n_profile() {
        FrozenProfile::Task258B3N
    } else if statements.is_task_258b3m1_profile() {
        FrozenProfile::Task258B3M1
    } else {
        return Err(SourceProofLocalDeclarationError::DependencyMismatch);
    };
    if statements.binding_env().source_id() != source_id
        || statements.binding_env().module_id() != module_id
        || witnesses.statement_fingerprint() != statements.debug_text()
        || witnesses.primary_term_fingerprint() != primary_terms.debug_text()
    {
        return Err(SourceProofLocalDeclarationError::DependencyMismatch);
    }
    let canonical_arena = frozen_canonical_arena(profile, source_id);
    let failure = match witnesses.validate_installation(
        source_id,
        module_id,
        statements,
        primary_terms,
        &canonical_arena,
    ) {
        Ok(()) => None,
        Err(SourceStatementWitnessError::DependencyMismatch) => {
            return Err(SourceProofLocalDeclarationError::DependencyMismatch);
        }
        Err(SourceStatementWitnessError::InvalidAggregate) => Some(LowerFailure::Aggregate),
        Err(
            SourceStatementWitnessError::InvalidWitness { .. }
            | SourceStatementWitnessError::InvalidName { .. },
        ) => Some(LowerFailure::Declaration),
    };
    Ok(LowerValidation { profile, failure })
}

fn exact_input_declaration(
    declaration: &SourceProofLocalDeclarationInput,
    witnesses: &SourceStatementWitnessHandoff,
    profile: FrozenProfile,
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
        && exact_local_fields(&declaration.local, profile)
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

fn exact_output_local_fields(
    binding_env: &BindingEnv,
    source_id: SourceId,
    profile: FrozenProfile,
) -> bool {
    let (start, end) = frozen_local_range(profile);
    binding_env
        .bindings()
        .get(BindingId::new(1))
        .is_some_and(|binding| {
            binding.spelling == "y"
                && binding.kind == BindingKind::LocalAbbreviation
                && binding.owner_context == BindingContextId::new(1)
                && binding.declaration_range == range(source_id, start, end)
                && binding.visible_after_ordinal == 1
                && matches!(
                    &binding.identity,
                    BinderIdentity::ResolverLocal {
                        scope,
                        ordinal: 1,
                        declaration_range,
                    } if scope.path() == [0]
                        && *declaration_range == range(source_id, start, end)
                )
        })
}

fn exact_local_fields(local: &LocalTermBinding, profile: FrozenProfile) -> bool {
    let (start, end) = frozen_local_range(profile);
    local.spelling() == "y"
        && local.scope().path() == [0]
        && local.declaration_range().start == start
        && local.declaration_range().end == end
        && local.visible_after_ordinal() == 1
}

fn exact_local(source_id: SourceId, profile: FrozenProfile) -> LocalTermBinding {
    let (start, end) = frozen_local_range(profile);
    LocalTermBinding::new(
        "y",
        LocalTermScope::new(vec![0]),
        range(source_id, start, end),
        1,
    )
}

const fn frozen_local_range(profile: FrozenProfile) -> (usize, usize) {
    match profile {
        FrozenProfile::Task258B3N => (81, 82),
        FrozenProfile::Task258B3M1 => (84, 85),
    }
}

fn extend_binding_env(
    base: &BindingEnv,
    local: &LocalTermBinding,
    profile: FrozenProfile,
) -> Result<BindingEnv, SourceProofLocalDeclarationError> {
    if !exact_local_fields(local, profile)
        || local.declaration_range().source_id != base.source_id()
    {
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

fn exact_frozen_arena(profile: FrozenProfile, source_id: SourceId, arena: &TypedArena) -> bool {
    match profile {
        FrozenProfile::Task258B3N => exact_task258b3n_arena(source_id, arena),
        FrozenProfile::Task258B3M1 => exact_task258b3m1_arena(source_id, arena),
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

fn exact_task258b3m1_arena(source_id: SourceId, arena: &TypedArena) -> bool {
    arena.len() == TASK258B3M1_NODE_RANGES.len()
        && arena.root() == Some(TypedNodeId::new(55))
        && TASK258B3M1_NODE_RANGES
            .iter()
            .copied()
            .enumerate()
            .all(|(index, (start, end))| {
                arena.node(TypedNodeId::new(index)).is_some_and(|node| {
                    node.anchor == SourceAnchor::Range(range(source_id, start, end))
                        && node.kind.as_str() == task258b3m1_node_kind(index)
                        && node.resolved_node.is_none()
                        && node.recovery == NodeRecoveryState::Normal
                        && node.typing == TypingState::Unknown
                        && node.links == TypedNodeLinks::default()
                        && node
                            .children
                            .iter()
                            .map(|child| child.index())
                            .eq(task258b3m1_node_children(index).iter().copied())
                })
            })
}

fn frozen_canonical_arena(profile: FrozenProfile, source_id: SourceId) -> TypedArena {
    match profile {
        FrozenProfile::Task258B3N => task258b3n_canonical_arena(source_id),
        FrozenProfile::Task258B3M1 => task258b3m1_canonical_arena(source_id),
    }
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

fn task258b3m1_canonical_arena(source_id: SourceId) -> TypedArena {
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
                    SourceAnchor::Range(range(source_id, start, end)),
                )
                .with_children(children),
            )
            .expect("frozen Task258B3M1 node graph is valid");
        ids.push(id);
    }
    builder
        .finish(Some(ids[55]))
        .expect("frozen Task258B3M1 arena is valid")
}

/// Complete syntax-free input for the frozen proof-local `let` binding transaction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceProofLocalLetBindingHandoffInput {
    pub source_id: SourceId,
    pub module_id: ModuleId,
    pub lower_fingerprint: String,
    pub theorem_symbol: SymbolId,
    pub theorem_definition: DefinitionId,
    pub contribution: SourceContributionId,
    pub theorem_range: SourceRange,
    pub proof_range: SourceRange,
    pub let_range: SourceRange,
    pub segment_range: SourceRange,
    pub name_range: SourceRange,
    pub source_ordinal: usize,
    pub local: LocalTermBinding,
    pub recovery: SourceProofLocalLetBindingRecovery,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SourceProofLocalLetBindingId(usize);

impl SourceProofLocalLetBindingId {
    pub const fn new(index: usize) -> Self {
        Self(index)
    }

    pub const fn index(self) -> usize {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum SourceProofLocalLetBindingRecovery {
    Normal,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceProofLocalLetBinding {
    binding: BindingId,
    binding_context: BindingContextId,
    source_ordinal: usize,
    visible_after_ordinal: usize,
    recovery: SourceProofLocalLetBindingRecovery,
}

impl SourceProofLocalLetBinding {
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

    pub const fn recovery(&self) -> SourceProofLocalLetBindingRecovery {
        self.recovery
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceProofLocalLetBindingTable {
    rows: Vec<SourceProofLocalLetBinding>,
}

impl SourceProofLocalLetBindingTable {
    pub fn get(&self, id: SourceProofLocalLetBindingId) -> Option<&SourceProofLocalLetBinding> {
        self.rows.get(id.index())
    }

    pub fn iter(
        &self,
    ) -> impl Iterator<Item = (SourceProofLocalLetBindingId, &SourceProofLocalLetBinding)> {
        self.rows
            .iter()
            .enumerate()
            .map(|(index, row)| (SourceProofLocalLetBindingId::new(index), row))
    }

    pub const fn len(&self) -> usize {
        self.rows.len()
    }

    pub const fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }
}

/// Immutable proof-local `let` binding transaction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceProofLocalLetBindingHandoff {
    source_id: SourceId,
    module_id: ModuleId,
    lower_fingerprint: String,
    theorem_symbol: SymbolId,
    theorem_definition: DefinitionId,
    contribution: SourceContributionId,
    theorem_range: SourceRange,
    proof_range: SourceRange,
    let_range: SourceRange,
    segment_range: SourceRange,
    name_range: SourceRange,
    base_binding_env: BindingEnv,
    base_binding_fingerprint: String,
    binding_env: BindingEnv,
    final_binding_fingerprint: String,
    bindings: SourceProofLocalLetBindingTable,
}

impl SourceProofLocalLetBindingHandoff {
    pub const fn source_id(&self) -> SourceId {
        self.source_id
    }

    pub const fn module_id(&self) -> &ModuleId {
        &self.module_id
    }

    pub fn lower_fingerprint(&self) -> &str {
        &self.lower_fingerprint
    }

    pub const fn theorem_symbol(&self) -> &SymbolId {
        &self.theorem_symbol
    }

    pub const fn theorem_definition(&self) -> DefinitionId {
        self.theorem_definition
    }

    pub const fn contribution(&self) -> SourceContributionId {
        self.contribution
    }

    pub const fn theorem_range(&self) -> SourceRange {
        self.theorem_range
    }

    pub const fn proof_range(&self) -> SourceRange {
        self.proof_range
    }

    pub const fn let_range(&self) -> SourceRange {
        self.let_range
    }

    pub const fn segment_range(&self) -> SourceRange {
        self.segment_range
    }

    pub const fn name_range(&self) -> SourceRange {
        self.name_range
    }

    pub const fn base_binding_env(&self) -> &BindingEnv {
        &self.base_binding_env
    }

    pub fn base_binding_fingerprint(&self) -> &str {
        &self.base_binding_fingerprint
    }

    pub const fn binding_env(&self) -> &BindingEnv {
        &self.binding_env
    }

    pub fn final_binding_fingerprint(&self) -> &str {
        &self.final_binding_fingerprint
    }

    pub const fn bindings(&self) -> &SourceProofLocalLetBindingTable {
        &self.bindings
    }

    pub fn debug_text(&self) -> String {
        let binding = self
            .bindings
            .get(SourceProofLocalLetBindingId::new(0))
            .expect("validated Task269C handoff has one dense row");
        format!(
            concat!(
                "source-proof-local-let-binding-debug-v1\n",
                "module: {}::{}\n",
                "lower-fingerprint: {:?}\n",
                "theorem symbol={:?} definition={} contribution={} range={}..{} proof={}..{}\n",
                "let range={}..{} segment={}..{} name={}..{} source_ordinal={}\n",
                "base-binding-fingerprint: {:?}\n",
                "binding#0 binding={} context={} source_ordinal={} visible_after={} recovery={}\n",
                "final-binding-fingerprint: {:?}\n",
            ),
            self.module_id.package().as_str(),
            self.module_id.path().as_str(),
            self.lower_fingerprint,
            self.theorem_symbol.fqn().as_str(),
            self.theorem_definition.index(),
            self.contribution.index(),
            self.theorem_range.start,
            self.theorem_range.end,
            self.proof_range.start,
            self.proof_range.end,
            self.let_range.start,
            self.let_range.end,
            self.segment_range.start,
            self.segment_range.end,
            self.name_range.start,
            self.name_range.end,
            binding.source_ordinal,
            self.base_binding_fingerprint,
            binding.binding.index(),
            binding.binding_context.index(),
            binding.source_ordinal,
            binding.visible_after_ordinal,
            let_binding_recovery_key(binding.recovery),
            self.final_binding_fingerprint,
        )
    }

    pub(crate) fn validate_installation(
        &self,
        source_id: SourceId,
        module_id: &ModuleId,
    ) -> Result<(), SourceProofLocalLetBindingError> {
        if self.source_id != source_id
            || &self.module_id != module_id
            || self.base_binding_env.source_id() != source_id
            || self.base_binding_env.module_id() != module_id
            || self.binding_env.source_id() != source_id
            || self.binding_env.module_id() != module_id
        {
            return Err(SourceProofLocalLetBindingError::InvalidTransaction);
        }
        validate_task269c_dependency(Task269cDependency {
            source_id: self.source_id,
            module_id: &self.module_id,
            lower_fingerprint: &self.lower_fingerprint,
            theorem_symbol: &self.theorem_symbol,
            theorem_definition: self.theorem_definition,
            contribution: self.contribution,
            theorem_range: self.theorem_range,
            proof_range: self.proof_range,
            let_range: self.let_range,
            segment_range: self.segment_range,
            name_range: self.name_range,
        })?;
        if !exact_task269c_base_binding_env(&self.base_binding_env)
            || self.base_binding_fingerprint != self.base_binding_env.debug_text()
        {
            return Err(SourceProofLocalLetBindingError::InvalidBaseBindingEnvironment);
        }
        if self.bindings.len() != 1 {
            return Err(SourceProofLocalLetBindingError::InvalidAggregate);
        }
        let id = SourceProofLocalLetBindingId::new(0);
        let binding = self
            .bindings
            .get(id)
            .ok_or(SourceProofLocalLetBindingError::InvalidDeclaration { binding: id })?;
        if !exact_task269c_output_binding(binding)
            || !exact_task269c_declaration_binding(&self.binding_env, self.source_id)
        {
            return Err(SourceProofLocalLetBindingError::InvalidDeclaration { binding: id });
        }
        let expected = extend_task269c_binding_env(&self.base_binding_env)?;
        if self.binding_env != expected
            || self.final_binding_fingerprint != self.binding_env.debug_text()
            || !exact_task269c_local_binding(&self.binding_env, self.source_id)
            || !exact_task269c_lookup_behavior(&self.binding_env)
        {
            return Err(SourceProofLocalLetBindingError::InvalidBindingEnvironment);
        }
        Ok(())
    }

    pub(crate) fn validate_complete_installation(
        &self,
        source_id: SourceId,
        module_id: &ModuleId,
        installation_available: bool,
    ) -> Result<(), SourceProofLocalLetBindingError> {
        self.validate_installation(source_id, module_id)?;
        if !installation_available {
            return Err(SourceProofLocalLetBindingError::InvalidInstallation);
        }
        Ok(())
    }

    #[cfg(test)]
    pub(crate) fn set_lower_fingerprint_for_test(&mut self, value: impl Into<String>) {
        self.lower_fingerprint = value.into();
    }

    #[cfg(test)]
    pub(crate) fn set_base_binding_fingerprint_for_task269c_test(
        &mut self,
        value: impl Into<String>,
    ) {
        self.base_binding_fingerprint = value.into();
    }

    #[cfg(test)]
    pub(crate) fn truncate_task269c_bindings_for_test(&mut self) {
        self.bindings.rows.clear();
    }

    #[cfg(test)]
    pub(crate) fn corrupt_task269c_binding_row_for_test(&mut self) {
        if let Some(binding) = self.bindings.rows.first_mut() {
            binding.source_ordinal += 1;
        }
    }

    #[cfg(test)]
    pub(crate) fn set_final_binding_fingerprint_for_task269c_test(
        &mut self,
        value: impl Into<String>,
    ) {
        self.final_binding_fingerprint = value.into();
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct SourceProofLocalLetBindingProducer;

impl SourceProofLocalLetBindingProducer {
    pub fn build(
        input: SourceProofLocalLetBindingHandoffInput,
        base_binding_env: &BindingEnv,
    ) -> Result<SourceProofLocalLetBindingHandoff, SourceProofLocalLetBindingError> {
        if input.source_id != base_binding_env.source_id()
            || &input.module_id != base_binding_env.module_id()
        {
            return Err(SourceProofLocalLetBindingError::InvalidTransaction);
        }
        validate_task269c_dependency(Task269cDependency {
            source_id: input.source_id,
            module_id: &input.module_id,
            lower_fingerprint: &input.lower_fingerprint,
            theorem_symbol: &input.theorem_symbol,
            theorem_definition: input.theorem_definition,
            contribution: input.contribution,
            theorem_range: input.theorem_range,
            proof_range: input.proof_range,
            let_range: input.let_range,
            segment_range: input.segment_range,
            name_range: input.name_range,
        })?;
        if !exact_task269c_base_binding_env(base_binding_env) {
            return Err(SourceProofLocalLetBindingError::InvalidBaseBindingEnvironment);
        }
        if !exact_task269c_input_declaration(&input) {
            return Err(SourceProofLocalLetBindingError::InvalidDeclaration {
                binding: SourceProofLocalLetBindingId::new(0),
            });
        }
        let binding_env = extend_task269c_binding_env(base_binding_env)?;
        if !exact_task269c_lookup_behavior(&binding_env) {
            return Err(SourceProofLocalLetBindingError::InvalidBindingEnvironment);
        }
        let base_binding_fingerprint = base_binding_env.debug_text();
        let final_binding_fingerprint = binding_env.debug_text();
        Ok(SourceProofLocalLetBindingHandoff {
            source_id: input.source_id,
            module_id: input.module_id,
            lower_fingerprint: input.lower_fingerprint,
            theorem_symbol: input.theorem_symbol,
            theorem_definition: input.theorem_definition,
            contribution: input.contribution,
            theorem_range: input.theorem_range,
            proof_range: input.proof_range,
            let_range: input.let_range,
            segment_range: input.segment_range,
            name_range: input.name_range,
            base_binding_env: base_binding_env.clone(),
            base_binding_fingerprint,
            binding_env,
            final_binding_fingerprint,
            bindings: SourceProofLocalLetBindingTable {
                rows: vec![SourceProofLocalLetBinding {
                    binding: BindingId::new(1),
                    binding_context: BindingContextId::new(1),
                    source_ordinal: input.source_ordinal,
                    visible_after_ordinal: input.local.visible_after_ordinal(),
                    recovery: input.recovery,
                }],
            },
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum SourceProofLocalLetBindingError {
    InvalidTransaction,
    DependencyMismatch,
    InvalidBaseBindingEnvironment,
    InvalidAggregate,
    InvalidDeclaration {
        binding: SourceProofLocalLetBindingId,
    },
    InvalidBindingEnvironment,
    InvalidInstallation,
}

impl fmt::Display for SourceProofLocalLetBindingError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidTransaction => {
                formatter.write_str("source proof-local let-binding transaction is invalid")
            }
            Self::DependencyMismatch => {
                formatter.write_str("source proof-local let-binding dependency mismatch")
            }
            Self::InvalidBaseBindingEnvironment => formatter
                .write_str("source proof-local let-binding base binding environment is invalid"),
            Self::InvalidAggregate => {
                formatter.write_str("source proof-local let-binding aggregate is invalid")
            }
            Self::InvalidDeclaration { binding } => write!(
                formatter,
                "source proof-local let-binding {} is invalid",
                binding.index()
            ),
            Self::InvalidBindingEnvironment => {
                formatter.write_str("source proof-local let-binding binding environment is invalid")
            }
            Self::InvalidInstallation => {
                formatter.write_str("source proof-local let-binding installation is invalid")
            }
        }
    }
}

impl Error for SourceProofLocalLetBindingError {}

#[derive(Debug, Clone, Copy)]
struct Task269cDependency<'a> {
    source_id: SourceId,
    module_id: &'a ModuleId,
    lower_fingerprint: &'a str,
    theorem_symbol: &'a SymbolId,
    theorem_definition: DefinitionId,
    contribution: SourceContributionId,
    theorem_range: SourceRange,
    proof_range: SourceRange,
    let_range: SourceRange,
    segment_range: SourceRange,
    name_range: SourceRange,
}

fn validate_task269c_dependency(
    dependency: Task269cDependency<'_>,
) -> Result<(), SourceProofLocalLetBindingError> {
    let expected_prefix = format!(
        "{}::{}::",
        dependency.module_id.package().as_str(),
        dependency.module_id.path().as_str()
    );
    let expected_local = format!(
        concat!(
            "contribution=0:namespace={}:owner=theorem#1:shell=theorem:kind=theorem:",
            "name=FormulaStatementLetSmoke:notation=_:arity=_:definition=theorem:",
            "registration=_:policy=non-overloadable:slot=non-overloadable:_:theorem:_"
        ),
        escape_task269c_symbol_component(dependency.module_id.path().as_str()),
    );
    let exact_symbol_identity = dependency.theorem_symbol.module() == dependency.module_id
        && dependency.theorem_symbol.local().as_str() == expected_local
        && dependency.theorem_symbol.fqn().as_str() == format!("{expected_prefix}{expected_local}");
    if !exact_symbol_identity
        || dependency.theorem_definition.index() != 0
        || dependency.contribution.index() != 0
        || dependency.theorem_range != range(dependency.source_id, 19, 99)
        || dependency.proof_range != range(dependency.source_id, 59, 98)
        || dependency.let_range != range(dependency.source_id, 67, 80)
        || dependency.segment_range != range(dependency.source_id, 71, 79)
        || dependency.name_range != range(dependency.source_id, 71, 72)
        || dependency.lower_fingerprint != exact_task269cp_lower_fingerprint(dependency)
    {
        return Err(SourceProofLocalLetBindingError::DependencyMismatch);
    }
    Ok(())
}

fn escape_task269c_symbol_component(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace(':', "\\c")
        .replace('|', "\\p")
        .replace('/', "\\s")
}

fn exact_task269cp_lower_fingerprint(dependency: Task269cDependency<'_>) -> String {
    format!(
        concat!(
            "source-proof-local-let-lower-debug-v1\n",
            "module: {}::{}\n",
            "source-fingerprint: \"7860a3fe5af89063ac6a2b9a4465cac36d26f6d64e892ba6e2c89bcbaaf9763a\"\n",
            "surface-fingerprint: \"1fc35ec18db82efc0968b2f42b08cfaae678184983210cd26f060d45354c7f68\"\n",
            "theorem symbol={:?} definition=0 contribution=0 range=19..99 proof=59..98\n",
            "let range=67..80 segment=71..79 source_ordinal=1\n",
            "name range=71..72 spelling=\"y\" scope=[0] visible_after=1\n",
            "type range=76..79 head=76..79 spelling=\"set\" form=bare\n",
        ),
        dependency.module_id.package().as_str(),
        dependency.module_id.path().as_str(),
        dependency.theorem_symbol.fqn().as_str(),
    )
}

fn exact_task269c_base_binding_env(base: &BindingEnv) -> bool {
    let Some(context) = base.contexts().get(BindingContextId::new(0)) else {
        return false;
    };
    let Some(binding) = base.bindings().get(BindingId::new(0)) else {
        return false;
    };
    base.contexts().len() == 1
        && base.bindings().len() == 1
        && base.diagnostics().is_empty()
        && context.id == BindingContextId::new(0)
        && context.owner == BindingContextOwner::Module
        && context.parent.is_none()
        && context.layer == BindingContextLayer::Module
        && context.lexical_scope.is_none()
        && context.bindings == [BindingId::new(0)]
        && context.visible_bindings == [BindingId::new(0)]
        && context.recovery == BindingContextRecovery::Normal
        && binding.id == BindingId::new(0)
        && binding.spelling == "x"
        && binding.kind == BindingKind::ReservedVariable
        && matches!(
            &binding.identity,
            BinderIdentity::ReservedVariable {
                spelling,
                declaration_range,
            } if spelling == "x"
                && *declaration_range == range(base.source_id(), 8, 9)
        )
        && binding.owner_context == BindingContextId::new(0)
        && binding.declaration_range == range(base.source_id(), 8, 9)
        && binding.visible_after_ordinal == 0
        && binding.type_site == BindingTypeSite::Source(range(base.source_id(), 14, 17))
        && binding.status == BindingStatus::Reserved
        && binding.captured.identities().is_empty()
        && binding.diagnostics.is_empty()
        && binding.recovery == BindingRecoveryState::Normal
}

fn exact_task269c_input_declaration(input: &SourceProofLocalLetBindingHandoffInput) -> bool {
    input.source_ordinal == 1
        && input.local.spelling() == "y"
        && input.local.scope().path() == [0]
        && input.local.declaration_range() == input.name_range
        && input.local.visible_after_ordinal() == 1
        && input.recovery == SourceProofLocalLetBindingRecovery::Normal
}

fn exact_task269c_output_binding(binding: &SourceProofLocalLetBinding) -> bool {
    binding.binding == BindingId::new(1)
        && binding.binding_context == BindingContextId::new(1)
        && binding.source_ordinal == 1
        && binding.visible_after_ordinal == 1
        && binding.recovery == SourceProofLocalLetBindingRecovery::Normal
}

fn exact_task269c_local_binding(binding_env: &BindingEnv, source_id: SourceId) -> bool {
    let Some(context) = binding_env.contexts().get(BindingContextId::new(1)) else {
        return false;
    };
    let Some(binding) = binding_env.bindings().get(BindingId::new(1)) else {
        return false;
    };
    context.id == BindingContextId::new(1)
        && context.owner
            == BindingContextOwner::SourceStatement {
                source_range: range(source_id, 59, 98),
            }
        && context.parent == Some(BindingContextId::new(0))
        && context.layer == BindingContextLayer::Proof
        && context
            .lexical_scope
            .as_ref()
            .is_some_and(|scope| scope.path() == [0])
        && context.bindings == [BindingId::new(1)]
        && context.visible_bindings == [BindingId::new(0), BindingId::new(1)]
        && context.recovery == BindingContextRecovery::Normal
        && binding.id == BindingId::new(1)
        && binding.spelling == "y"
        && binding.kind == BindingKind::LetBinding
        && matches!(
            &binding.identity,
            BinderIdentity::ResolverLocal {
                scope,
                ordinal: 1,
                declaration_range,
            } if scope.path() == [0]
                && *declaration_range == range(source_id, 71, 72)
        )
        && binding.owner_context == BindingContextId::new(1)
        && binding.declaration_range == range(source_id, 71, 72)
        && binding.visible_after_ordinal == 1
        && binding.type_site == BindingTypeSite::Missing
        && binding.status == BindingStatus::Active
        && binding.captured.identities().is_empty()
        && binding.diagnostics.is_empty()
        && binding.recovery == BindingRecoveryState::Normal
}

fn exact_task269c_declaration_binding(binding_env: &BindingEnv, source_id: SourceId) -> bool {
    binding_env
        .bindings()
        .get(BindingId::new(1))
        .is_some_and(|binding| {
            binding.id == BindingId::new(1)
                && binding.spelling == "y"
                && matches!(
                    &binding.identity,
                    BinderIdentity::ResolverLocal {
                        scope,
                        ordinal: 1,
                        declaration_range,
                    } if scope.path() == [0]
                        && *declaration_range == range(source_id, 71, 72)
                )
                && binding.owner_context == BindingContextId::new(1)
                && binding.declaration_range == range(source_id, 71, 72)
                && binding.visible_after_ordinal == 1
                && binding.recovery == BindingRecoveryState::Normal
        })
}

fn extend_task269c_binding_env(
    base: &BindingEnv,
) -> Result<BindingEnv, SourceProofLocalLetBindingError> {
    if !exact_task269c_base_binding_env(base) {
        return Err(SourceProofLocalLetBindingError::InvalidBaseBindingEnvironment);
    }
    let local = LocalTermBinding::new(
        "y",
        LocalTermScope::new(vec![0]),
        range(base.source_id(), 71, 72),
        1,
    );
    let mut bindings = base.bindings().clone();
    let binding = bindings.insert(BindingDraft::from_local_term(
        BindingContextId::new(1),
        BindingKind::LetBinding,
        &local,
    ));
    if binding != BindingId::new(1) {
        return Err(SourceProofLocalLetBindingError::InvalidBindingEnvironment);
    }
    let mut contexts = base.contexts().clone();
    let context = contexts.insert(BindingContextDraft {
        owner: BindingContextOwner::SourceStatement {
            source_range: range(base.source_id(), 59, 98),
        },
        parent: Some(BindingContextId::new(0)),
        layer: BindingContextLayer::Proof,
        lexical_scope: Some(LocalTermScope::new(vec![0])),
        bindings: vec![binding],
        visible_bindings: vec![BindingId::new(0), binding],
        recovery: BindingContextRecovery::Normal,
    });
    if context != BindingContextId::new(1) {
        return Err(SourceProofLocalLetBindingError::InvalidBindingEnvironment);
    }
    BindingEnv::try_new(BindingEnvParts {
        source_id: base.source_id(),
        module_id: base.module_id().clone(),
        contexts,
        bindings,
        diagnostics: base.diagnostics().clone(),
    })
    .map_err(|_| SourceProofLocalLetBindingError::InvalidBindingEnvironment)
}

fn exact_task269c_lookup_behavior(binding_env: &BindingEnv) -> bool {
    let definition = BindingLookupSite::new(
        "y",
        BindingContextId::new(1),
        Some(LocalTermScope::new(vec![0])),
        1,
    );
    let later = BindingLookupSite::new(
        "y",
        BindingContextId::new(1),
        Some(LocalTermScope::new(vec![0])),
        2,
    );
    matches!(
        binding_env.lookup(&definition),
        Ok(BindingLookupResult::ForwardReference { candidates, .. })
            if candidates == [BindingId::new(1)]
    ) && binding_env.lookup(&later) == Ok(BindingLookupResult::Local(BindingId::new(1)))
}

const fn let_binding_recovery_key(recovery: SourceProofLocalLetBindingRecovery) -> &'static str {
    match recovery {
        SourceProofLocalLetBindingRecovery::Normal => "normal",
    }
}

/// Complete syntax-free input for the frozen proof-local `given` binding transaction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceProofLocalGivenBindingHandoffInput {
    pub source_id: SourceId,
    pub module_id: ModuleId,
    pub lower_fingerprint: String,
    pub theorem_symbol: SymbolId,
    pub theorem_definition: DefinitionId,
    pub contribution: SourceContributionId,
    pub theorem_range: SourceRange,
    pub proof_range: SourceRange,
    pub given_range: SourceRange,
    pub segment_range: SourceRange,
    pub name_range: SourceRange,
    pub source_ordinal: usize,
    pub local: LocalTermBinding,
    pub recovery: SourceProofLocalGivenBindingRecovery,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SourceProofLocalGivenBindingId(usize);

impl SourceProofLocalGivenBindingId {
    pub const fn new(index: usize) -> Self {
        Self(index)
    }

    pub const fn index(self) -> usize {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum SourceProofLocalGivenBindingRecovery {
    Normal,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceProofLocalGivenBinding {
    binding: BindingId,
    binding_context: BindingContextId,
    source_ordinal: usize,
    visible_after_ordinal: usize,
    recovery: SourceProofLocalGivenBindingRecovery,
}

impl SourceProofLocalGivenBinding {
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

    pub const fn recovery(&self) -> SourceProofLocalGivenBindingRecovery {
        self.recovery
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceProofLocalGivenBindingTable {
    rows: Vec<SourceProofLocalGivenBinding>,
}

impl SourceProofLocalGivenBindingTable {
    pub fn get(&self, id: SourceProofLocalGivenBindingId) -> Option<&SourceProofLocalGivenBinding> {
        self.rows.get(id.index())
    }

    pub fn iter(
        &self,
    ) -> impl Iterator<
        Item = (
            SourceProofLocalGivenBindingId,
            &SourceProofLocalGivenBinding,
        ),
    > {
        self.rows
            .iter()
            .enumerate()
            .map(|(index, row)| (SourceProofLocalGivenBindingId::new(index), row))
    }

    pub const fn len(&self) -> usize {
        self.rows.len()
    }

    pub const fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }
}

/// Immutable proof-local `given` binding transaction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceProofLocalGivenBindingHandoff {
    source_id: SourceId,
    module_id: ModuleId,
    lower_fingerprint: String,
    theorem_symbol: SymbolId,
    theorem_definition: DefinitionId,
    contribution: SourceContributionId,
    theorem_range: SourceRange,
    proof_range: SourceRange,
    given_range: SourceRange,
    segment_range: SourceRange,
    name_range: SourceRange,
    base_binding_env: BindingEnv,
    base_binding_fingerprint: String,
    binding_env: BindingEnv,
    final_binding_fingerprint: String,
    bindings: SourceProofLocalGivenBindingTable,
}

impl SourceProofLocalGivenBindingHandoff {
    pub const fn source_id(&self) -> SourceId {
        self.source_id
    }

    pub const fn module_id(&self) -> &ModuleId {
        &self.module_id
    }

    pub fn lower_fingerprint(&self) -> &str {
        &self.lower_fingerprint
    }

    pub const fn theorem_symbol(&self) -> &SymbolId {
        &self.theorem_symbol
    }

    pub const fn theorem_definition(&self) -> DefinitionId {
        self.theorem_definition
    }

    pub const fn contribution(&self) -> SourceContributionId {
        self.contribution
    }

    pub const fn theorem_range(&self) -> SourceRange {
        self.theorem_range
    }

    pub const fn proof_range(&self) -> SourceRange {
        self.proof_range
    }

    pub const fn given_range(&self) -> SourceRange {
        self.given_range
    }

    pub const fn segment_range(&self) -> SourceRange {
        self.segment_range
    }

    pub const fn name_range(&self) -> SourceRange {
        self.name_range
    }

    pub const fn base_binding_env(&self) -> &BindingEnv {
        &self.base_binding_env
    }

    pub fn base_binding_fingerprint(&self) -> &str {
        &self.base_binding_fingerprint
    }

    pub const fn binding_env(&self) -> &BindingEnv {
        &self.binding_env
    }

    pub fn final_binding_fingerprint(&self) -> &str {
        &self.final_binding_fingerprint
    }

    pub const fn bindings(&self) -> &SourceProofLocalGivenBindingTable {
        &self.bindings
    }

    pub fn debug_text(&self) -> String {
        let binding = self
            .bindings
            .get(SourceProofLocalGivenBindingId::new(0))
            .expect("validated Task269G handoff has one dense row");
        format!(
            concat!(
                "source-proof-local-given-binding-debug-v1\n",
                "module: {}::{}\n",
                "lower-fingerprint: {:?}\n",
                "theorem symbol={:?} definition={} contribution={} range={}..{} proof={}..{}\n",
                "given range={}..{} segment={}..{} name={}..{} source_ordinal={}\n",
                "base-binding-fingerprint: {:?}\n",
                "binding#0 binding={} context={} source_ordinal={} visible_after={} recovery={}\n",
                "final-binding-fingerprint: {:?}\n",
            ),
            self.module_id.package().as_str(),
            self.module_id.path().as_str(),
            self.lower_fingerprint,
            self.theorem_symbol.fqn().as_str(),
            self.theorem_definition.index(),
            self.contribution.index(),
            self.theorem_range.start,
            self.theorem_range.end,
            self.proof_range.start,
            self.proof_range.end,
            self.given_range.start,
            self.given_range.end,
            self.segment_range.start,
            self.segment_range.end,
            self.name_range.start,
            self.name_range.end,
            binding.source_ordinal,
            self.base_binding_fingerprint,
            binding.binding.index(),
            binding.binding_context.index(),
            binding.source_ordinal,
            binding.visible_after_ordinal,
            given_binding_recovery_key(binding.recovery),
            self.final_binding_fingerprint,
        )
    }

    pub(crate) fn validate_installation(
        &self,
        source_id: SourceId,
        module_id: &ModuleId,
    ) -> Result<(), SourceProofLocalGivenBindingError> {
        if self.source_id != source_id
            || &self.module_id != module_id
            || self.base_binding_env.source_id() != source_id
            || self.base_binding_env.module_id() != module_id
            || self.binding_env.source_id() != source_id
            || self.binding_env.module_id() != module_id
        {
            return Err(SourceProofLocalGivenBindingError::InvalidTransaction);
        }
        validate_task269g_dependency(Task269gDependency {
            source_id: self.source_id,
            module_id: &self.module_id,
            lower_fingerprint: &self.lower_fingerprint,
            theorem_symbol: &self.theorem_symbol,
            theorem_definition: self.theorem_definition,
            contribution: self.contribution,
            theorem_range: self.theorem_range,
            proof_range: self.proof_range,
            given_range: self.given_range,
            segment_range: self.segment_range,
            name_range: self.name_range,
        })?;
        if !exact_task269c_base_binding_env(&self.base_binding_env)
            || self.base_binding_fingerprint != self.base_binding_env.debug_text()
        {
            return Err(SourceProofLocalGivenBindingError::InvalidBaseBindingEnvironment);
        }
        if self.bindings.len() != 1 {
            return Err(SourceProofLocalGivenBindingError::InvalidAggregate);
        }
        let id = SourceProofLocalGivenBindingId::new(0);
        let binding = self
            .bindings
            .get(id)
            .ok_or(SourceProofLocalGivenBindingError::InvalidDeclaration { binding: id })?;
        if !exact_task269g_output_binding(binding)
            || !exact_task269g_declaration_binding(&self.binding_env, self.source_id)
        {
            return Err(SourceProofLocalGivenBindingError::InvalidDeclaration { binding: id });
        }
        let expected = extend_task269g_binding_env(&self.base_binding_env)?;
        if self.binding_env != expected
            || self.final_binding_fingerprint != self.binding_env.debug_text()
            || !exact_task269g_local_binding(&self.binding_env, self.source_id)
            || !exact_task269g_lookup_behavior(&self.binding_env)
        {
            return Err(SourceProofLocalGivenBindingError::InvalidBindingEnvironment);
        }
        Ok(())
    }

    pub(crate) fn validate_complete_installation(
        &self,
        source_id: SourceId,
        module_id: &ModuleId,
        installation_available: bool,
    ) -> Result<(), SourceProofLocalGivenBindingError> {
        self.validate_installation(source_id, module_id)?;
        if !installation_available {
            return Err(SourceProofLocalGivenBindingError::InvalidInstallation);
        }
        Ok(())
    }

    #[cfg(test)]
    pub(crate) fn set_lower_fingerprint_for_test(&mut self, value: impl Into<String>) {
        self.lower_fingerprint = value.into();
    }

    #[cfg(test)]
    pub(crate) fn set_base_binding_fingerprint_for_task269g_test(
        &mut self,
        value: impl Into<String>,
    ) {
        self.base_binding_fingerprint = value.into();
    }

    #[cfg(test)]
    pub(crate) fn truncate_task269g_bindings_for_test(&mut self) {
        self.bindings.rows.clear();
    }

    #[cfg(test)]
    pub(crate) fn corrupt_task269g_binding_row_for_test(&mut self) {
        if let Some(binding) = self.bindings.rows.first_mut() {
            binding.source_ordinal += 1;
        }
    }

    #[cfg(test)]
    pub(crate) fn set_final_binding_fingerprint_for_task269g_test(
        &mut self,
        value: impl Into<String>,
    ) {
        self.final_binding_fingerprint = value.into();
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct SourceProofLocalGivenBindingProducer;

impl SourceProofLocalGivenBindingProducer {
    pub fn build(
        input: SourceProofLocalGivenBindingHandoffInput,
        base_binding_env: &BindingEnv,
    ) -> Result<SourceProofLocalGivenBindingHandoff, SourceProofLocalGivenBindingError> {
        if input.source_id != base_binding_env.source_id()
            || &input.module_id != base_binding_env.module_id()
        {
            return Err(SourceProofLocalGivenBindingError::InvalidTransaction);
        }
        validate_task269g_dependency(Task269gDependency {
            source_id: input.source_id,
            module_id: &input.module_id,
            lower_fingerprint: &input.lower_fingerprint,
            theorem_symbol: &input.theorem_symbol,
            theorem_definition: input.theorem_definition,
            contribution: input.contribution,
            theorem_range: input.theorem_range,
            proof_range: input.proof_range,
            given_range: input.given_range,
            segment_range: input.segment_range,
            name_range: input.name_range,
        })?;
        if !exact_task269c_base_binding_env(base_binding_env) {
            return Err(SourceProofLocalGivenBindingError::InvalidBaseBindingEnvironment);
        }
        if !exact_task269g_input_declaration(&input) {
            return Err(SourceProofLocalGivenBindingError::InvalidDeclaration {
                binding: SourceProofLocalGivenBindingId::new(0),
            });
        }
        let binding_env = extend_task269g_binding_env(base_binding_env)?;
        if !exact_task269g_lookup_behavior(&binding_env) {
            return Err(SourceProofLocalGivenBindingError::InvalidBindingEnvironment);
        }
        let base_binding_fingerprint = base_binding_env.debug_text();
        let final_binding_fingerprint = binding_env.debug_text();
        Ok(SourceProofLocalGivenBindingHandoff {
            source_id: input.source_id,
            module_id: input.module_id,
            lower_fingerprint: input.lower_fingerprint,
            theorem_symbol: input.theorem_symbol,
            theorem_definition: input.theorem_definition,
            contribution: input.contribution,
            theorem_range: input.theorem_range,
            proof_range: input.proof_range,
            given_range: input.given_range,
            segment_range: input.segment_range,
            name_range: input.name_range,
            base_binding_env: base_binding_env.clone(),
            base_binding_fingerprint,
            binding_env,
            final_binding_fingerprint,
            bindings: SourceProofLocalGivenBindingTable {
                rows: vec![SourceProofLocalGivenBinding {
                    binding: BindingId::new(1),
                    binding_context: BindingContextId::new(1),
                    source_ordinal: input.source_ordinal,
                    visible_after_ordinal: input.local.visible_after_ordinal(),
                    recovery: input.recovery,
                }],
            },
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum SourceProofLocalGivenBindingError {
    InvalidTransaction,
    DependencyMismatch,
    InvalidBaseBindingEnvironment,
    InvalidAggregate,
    InvalidDeclaration {
        binding: SourceProofLocalGivenBindingId,
    },
    InvalidBindingEnvironment,
    InvalidInstallation,
}

impl fmt::Display for SourceProofLocalGivenBindingError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidTransaction => {
                formatter.write_str("source proof-local given-binding transaction is invalid")
            }
            Self::DependencyMismatch => {
                formatter.write_str("source proof-local given-binding dependency mismatch")
            }
            Self::InvalidBaseBindingEnvironment => formatter
                .write_str("source proof-local given-binding base binding environment is invalid"),
            Self::InvalidAggregate => {
                formatter.write_str("source proof-local given-binding aggregate is invalid")
            }
            Self::InvalidDeclaration { binding } => write!(
                formatter,
                "source proof-local given-binding {} is invalid",
                binding.index()
            ),
            Self::InvalidBindingEnvironment => formatter
                .write_str("source proof-local given-binding binding environment is invalid"),
            Self::InvalidInstallation => {
                formatter.write_str("source proof-local given-binding installation is invalid")
            }
        }
    }
}

impl Error for SourceProofLocalGivenBindingError {}

#[derive(Debug, Clone, Copy)]
struct Task269gDependency<'a> {
    source_id: SourceId,
    module_id: &'a ModuleId,
    lower_fingerprint: &'a str,
    theorem_symbol: &'a SymbolId,
    theorem_definition: DefinitionId,
    contribution: SourceContributionId,
    theorem_range: SourceRange,
    proof_range: SourceRange,
    given_range: SourceRange,
    segment_range: SourceRange,
    name_range: SourceRange,
}

fn validate_task269g_dependency(
    dependency: Task269gDependency<'_>,
) -> Result<(), SourceProofLocalGivenBindingError> {
    let expected_prefix = format!(
        "{}::{}::",
        dependency.module_id.package().as_str(),
        dependency.module_id.path().as_str()
    );
    let expected_local = format!(
        concat!(
            "contribution=0:namespace={}:owner=theorem#1:shell=theorem:kind=theorem:",
            "name=FormulaStatementGivenSmoke:notation=_:arity=_:definition=theorem:",
            "registration=_:policy=non-overloadable:slot=non-overloadable:_:theorem:_"
        ),
        escape_task269c_symbol_component(dependency.module_id.path().as_str()),
    );
    let exact_symbol_identity = dependency.theorem_symbol.module() == dependency.module_id
        && dependency.theorem_symbol.local().as_str() == expected_local
        && dependency.theorem_symbol.fqn().as_str() == format!("{expected_prefix}{expected_local}");
    if !exact_symbol_identity
        || dependency.theorem_definition.index() != 0
        || dependency.contribution.index() != 0
        || dependency.theorem_range != range(dependency.source_id, 19, 128)
        || dependency.proof_range != range(dependency.source_id, 62, 127)
        || dependency.given_range != range(dependency.source_id, 70, 108)
        || dependency.segment_range != range(dependency.source_id, 76, 87)
        || dependency.name_range != range(dependency.source_id, 76, 77)
        || dependency.lower_fingerprint != exact_task269gp_lower_fingerprint(dependency)
    {
        return Err(SourceProofLocalGivenBindingError::DependencyMismatch);
    }
    Ok(())
}

fn exact_task269gp_lower_fingerprint(dependency: Task269gDependency<'_>) -> String {
    format!(
        concat!(
            "source-proof-local-given-lower-debug-v1\n",
            "module: {}::{}\n",
            "source-fingerprint: \"04e54b8ada9af54fde9f937e1bb0f96bd8cf85002b2b57f4d348b11c8eb72a2f\"\n",
            "surface-fingerprint: \"58ac16a3c75860180a8bec5dc8e87ec8b269fe75715a6d8363f7ef064e3deea8\"\n",
            "theorem symbol={:?} definition=0 contribution=0 range=19..128 proof=62..127\n",
            "given range=70..108 segment=76..87 source_ordinal=1\n",
            "name range=76..77 spelling=\"y\"\n",
            "type range=84..87 head=84..87 spelling=\"set\" form=bare\n",
        ),
        dependency.module_id.package().as_str(),
        dependency.module_id.path().as_str(),
        dependency.theorem_symbol.fqn().as_str(),
    )
}

fn exact_task269g_input_declaration(input: &SourceProofLocalGivenBindingHandoffInput) -> bool {
    input.source_ordinal == 1
        && input.local.spelling() == "y"
        && input.local.scope().path() == [0]
        && input.local.declaration_range() == input.name_range
        && input.local.visible_after_ordinal() == 1
        && input.recovery == SourceProofLocalGivenBindingRecovery::Normal
}

fn exact_task269g_output_binding(binding: &SourceProofLocalGivenBinding) -> bool {
    binding.binding == BindingId::new(1)
        && binding.binding_context == BindingContextId::new(1)
        && binding.source_ordinal == 1
        && binding.visible_after_ordinal == 1
        && binding.recovery == SourceProofLocalGivenBindingRecovery::Normal
}

fn exact_task269g_local_binding(binding_env: &BindingEnv, source_id: SourceId) -> bool {
    let Some(context) = binding_env.contexts().get(BindingContextId::new(1)) else {
        return false;
    };
    let Some(binding) = binding_env.bindings().get(BindingId::new(1)) else {
        return false;
    };
    context.id == BindingContextId::new(1)
        && context.owner
            == BindingContextOwner::SourceStatement {
                source_range: range(source_id, 62, 127),
            }
        && context.parent == Some(BindingContextId::new(0))
        && context.layer == BindingContextLayer::Proof
        && context
            .lexical_scope
            .as_ref()
            .is_some_and(|scope| scope.path() == [0])
        && context.bindings == [BindingId::new(1)]
        && context.visible_bindings == [BindingId::new(0), BindingId::new(1)]
        && context.recovery == BindingContextRecovery::Normal
        && binding.id == BindingId::new(1)
        && binding.spelling == "y"
        && binding.kind == BindingKind::GivenWitness
        && matches!(
            &binding.identity,
            BinderIdentity::ResolverLocal {
                scope,
                ordinal: 1,
                declaration_range,
            } if scope.path() == [0]
                && *declaration_range == range(source_id, 76, 77)
        )
        && binding.owner_context == BindingContextId::new(1)
        && binding.declaration_range == range(source_id, 76, 77)
        && binding.visible_after_ordinal == 1
        && binding.type_site == BindingTypeSite::Missing
        && binding.status == BindingStatus::Active
        && binding.captured.identities().is_empty()
        && binding.diagnostics.is_empty()
        && binding.recovery == BindingRecoveryState::Normal
}

fn exact_task269g_declaration_binding(binding_env: &BindingEnv, source_id: SourceId) -> bool {
    binding_env
        .bindings()
        .get(BindingId::new(1))
        .is_some_and(|binding| {
            binding.id == BindingId::new(1)
                && binding.spelling == "y"
                && matches!(
                    &binding.identity,
                    BinderIdentity::ResolverLocal {
                        scope,
                        ordinal: 1,
                        declaration_range,
                    } if scope.path() == [0]
                        && *declaration_range == range(source_id, 76, 77)
                )
                && binding.owner_context == BindingContextId::new(1)
                && binding.declaration_range == range(source_id, 76, 77)
                && binding.visible_after_ordinal == 1
                && binding.recovery == BindingRecoveryState::Normal
        })
}

fn extend_task269g_binding_env(
    base: &BindingEnv,
) -> Result<BindingEnv, SourceProofLocalGivenBindingError> {
    if !exact_task269c_base_binding_env(base) {
        return Err(SourceProofLocalGivenBindingError::InvalidBaseBindingEnvironment);
    }
    let local = LocalTermBinding::new(
        "y",
        LocalTermScope::new(vec![0]),
        range(base.source_id(), 76, 77),
        1,
    );
    let mut bindings = base.bindings().clone();
    let binding = bindings.insert(BindingDraft::from_local_term(
        BindingContextId::new(1),
        BindingKind::GivenWitness,
        &local,
    ));
    if binding != BindingId::new(1) {
        return Err(SourceProofLocalGivenBindingError::InvalidBindingEnvironment);
    }
    let mut contexts = base.contexts().clone();
    let context = contexts.insert(BindingContextDraft {
        owner: BindingContextOwner::SourceStatement {
            source_range: range(base.source_id(), 62, 127),
        },
        parent: Some(BindingContextId::new(0)),
        layer: BindingContextLayer::Proof,
        lexical_scope: Some(LocalTermScope::new(vec![0])),
        bindings: vec![binding],
        visible_bindings: vec![BindingId::new(0), binding],
        recovery: BindingContextRecovery::Normal,
    });
    if context != BindingContextId::new(1) {
        return Err(SourceProofLocalGivenBindingError::InvalidBindingEnvironment);
    }
    BindingEnv::try_new(BindingEnvParts {
        source_id: base.source_id(),
        module_id: base.module_id().clone(),
        contexts,
        bindings,
        diagnostics: base.diagnostics().clone(),
    })
    .map_err(|_| SourceProofLocalGivenBindingError::InvalidBindingEnvironment)
}

fn exact_task269g_lookup_behavior(binding_env: &BindingEnv) -> bool {
    let definition = BindingLookupSite::new(
        "y",
        BindingContextId::new(1),
        Some(LocalTermScope::new(vec![0])),
        1,
    );
    let later = BindingLookupSite::new(
        "y",
        BindingContextId::new(1),
        Some(LocalTermScope::new(vec![0])),
        2,
    );
    matches!(
        binding_env.lookup(&definition),
        Ok(BindingLookupResult::ForwardReference { candidates, .. })
            if candidates == [BindingId::new(1)]
    ) && binding_env.lookup(&later) == Ok(BindingLookupResult::Local(BindingId::new(1)))
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceProofLocalGivenUseBindingHandoffInput {
    pub source_id: SourceId,
    pub module_id: ModuleId,
    pub lower_fingerprint: String,
    pub theorem_symbol: SymbolId,
    pub theorem_definition: DefinitionId,
    pub contribution: SourceContributionId,
    pub theorem_range: SourceRange,
    pub proof_range: SourceRange,
    pub given_range: SourceRange,
    pub segment_range: SourceRange,
    pub name_range: SourceRange,
    pub source_ordinal: usize,
    pub local: LocalTermBinding,
    pub recovery: SourceProofLocalGivenBindingRecovery,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceProofLocalGivenUseBindingHandoff {
    source_id: SourceId,
    module_id: ModuleId,
    lower_fingerprint: String,
    theorem_symbol: SymbolId,
    theorem_definition: DefinitionId,
    contribution: SourceContributionId,
    theorem_range: SourceRange,
    proof_range: SourceRange,
    given_range: SourceRange,
    segment_range: SourceRange,
    name_range: SourceRange,
    base_binding_env: BindingEnv,
    base_binding_fingerprint: String,
    binding_env: BindingEnv,
    final_binding_fingerprint: String,
    bindings: SourceProofLocalGivenBindingTable,
}

// Rationale: Task 269GUP exposes this validator for the separate Task-269GUPT consumer.
#[cfg_attr(not(test), allow(dead_code))]
impl SourceProofLocalGivenUseBindingHandoff {
    pub const fn source_id(&self) -> SourceId {
        self.source_id
    }

    pub const fn module_id(&self) -> &ModuleId {
        &self.module_id
    }

    pub fn lower_fingerprint(&self) -> &str {
        &self.lower_fingerprint
    }

    pub const fn theorem_symbol(&self) -> &SymbolId {
        &self.theorem_symbol
    }

    pub const fn theorem_definition(&self) -> DefinitionId {
        self.theorem_definition
    }

    pub const fn contribution(&self) -> SourceContributionId {
        self.contribution
    }

    pub const fn theorem_range(&self) -> SourceRange {
        self.theorem_range
    }

    pub const fn proof_range(&self) -> SourceRange {
        self.proof_range
    }

    pub const fn given_range(&self) -> SourceRange {
        self.given_range
    }

    pub const fn segment_range(&self) -> SourceRange {
        self.segment_range
    }

    pub const fn name_range(&self) -> SourceRange {
        self.name_range
    }

    pub const fn base_binding_env(&self) -> &BindingEnv {
        &self.base_binding_env
    }

    pub fn base_binding_fingerprint(&self) -> &str {
        &self.base_binding_fingerprint
    }

    pub const fn binding_env(&self) -> &BindingEnv {
        &self.binding_env
    }

    pub fn final_binding_fingerprint(&self) -> &str {
        &self.final_binding_fingerprint
    }

    pub const fn bindings(&self) -> &SourceProofLocalGivenBindingTable {
        &self.bindings
    }

    pub fn debug_text(&self) -> String {
        let binding = self
            .bindings
            .get(SourceProofLocalGivenBindingId::new(0))
            .expect("validated Task269GUP handoff has one dense row");
        format!(
            concat!(
                "source-proof-local-given-use-binding-debug-v1\n",
                "module: {}::{}\n",
                "lower-fingerprint: {:?}\n",
                "theorem symbol={:?} definition={} contribution={} range={}..{} proof={}..{}\n",
                "given range={}..{} segment={}..{} name={}..{} source_ordinal={}\n",
                "base-binding-fingerprint: {:?}\n",
                "binding#0 binding={} context={} source_ordinal={} visible_after={} recovery={}\n",
                "final-binding-fingerprint: {:?}\n",
            ),
            self.module_id.package().as_str(),
            self.module_id.path().as_str(),
            self.lower_fingerprint,
            self.theorem_symbol.fqn().as_str(),
            self.theorem_definition.index(),
            self.contribution.index(),
            self.theorem_range.start,
            self.theorem_range.end,
            self.proof_range.start,
            self.proof_range.end,
            self.given_range.start,
            self.given_range.end,
            self.segment_range.start,
            self.segment_range.end,
            self.name_range.start,
            self.name_range.end,
            binding.source_ordinal,
            self.base_binding_fingerprint,
            binding.binding.index(),
            binding.binding_context.index(),
            binding.source_ordinal,
            binding.visible_after_ordinal,
            given_binding_recovery_key(binding.recovery),
            self.final_binding_fingerprint,
        )
    }

    pub(crate) fn validate_installation(
        &self,
        source_id: SourceId,
        module_id: &ModuleId,
    ) -> Result<(), SourceProofLocalGivenUseBindingError> {
        if self.source_id != source_id
            || &self.module_id != module_id
            || self.base_binding_env.source_id() != source_id
            || self.base_binding_env.module_id() != module_id
            || self.binding_env.source_id() != source_id
            || self.binding_env.module_id() != module_id
        {
            return Err(SourceProofLocalGivenUseBindingError::InvalidTransaction);
        }
        validate_task269gup_dependency(Task269gupDependency {
            source_id: self.source_id,
            module_id: &self.module_id,
            lower_fingerprint: &self.lower_fingerprint,
            theorem_symbol: &self.theorem_symbol,
            theorem_definition: self.theorem_definition,
            contribution: self.contribution,
            theorem_range: self.theorem_range,
            proof_range: self.proof_range,
            given_range: self.given_range,
            segment_range: self.segment_range,
            name_range: self.name_range,
        })?;
        if !exact_task269c_base_binding_env(&self.base_binding_env)
            || self.base_binding_fingerprint != self.base_binding_env.debug_text()
        {
            return Err(SourceProofLocalGivenUseBindingError::InvalidBaseBindingEnvironment);
        }
        if self.bindings.len() != 1 {
            return Err(SourceProofLocalGivenUseBindingError::InvalidAggregate);
        }
        let id = SourceProofLocalGivenBindingId::new(0);
        let binding = self
            .bindings
            .get(id)
            .ok_or(SourceProofLocalGivenUseBindingError::InvalidDeclaration { binding: id })?;
        if !exact_task269gup_output_binding(binding)
            || !exact_task269gup_declaration_binding(&self.binding_env, self.source_id)
        {
            return Err(SourceProofLocalGivenUseBindingError::InvalidDeclaration { binding: id });
        }
        let expected = extend_task269gup_binding_env(&self.base_binding_env)?;
        if self.binding_env != expected
            || !exact_task269gup_local_binding(&self.binding_env, self.source_id)
            || !exact_task269gup_lookup_behavior(&self.binding_env)
            || self.final_binding_fingerprint != self.binding_env.debug_text()
        {
            return Err(SourceProofLocalGivenUseBindingError::InvalidBindingEnvironment);
        }
        Ok(())
    }

    #[cfg(test)]
    pub(crate) fn set_lower_fingerprint_for_task269gup_test(&mut self, value: impl Into<String>) {
        self.lower_fingerprint = value.into();
    }

    #[cfg(test)]
    pub(crate) fn set_base_binding_fingerprint_for_task269gup_test(
        &mut self,
        value: impl Into<String>,
    ) {
        self.base_binding_fingerprint = value.into();
    }

    #[cfg(test)]
    pub(crate) fn truncate_task269gup_bindings_for_test(&mut self) {
        self.bindings.rows.clear();
    }

    #[cfg(test)]
    pub(crate) fn corrupt_task269gup_binding_row_for_test(&mut self) {
        if let Some(binding) = self.bindings.rows.first_mut() {
            binding.source_ordinal += 1;
        }
    }

    #[cfg(test)]
    pub(crate) fn set_final_binding_fingerprint_for_task269gup_test(
        &mut self,
        value: impl Into<String>,
    ) {
        self.final_binding_fingerprint = value.into();
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct SourceProofLocalGivenUseBindingProducer;

impl SourceProofLocalGivenUseBindingProducer {
    pub fn build(
        input: SourceProofLocalGivenUseBindingHandoffInput,
        base_binding_env: &BindingEnv,
    ) -> Result<SourceProofLocalGivenUseBindingHandoff, SourceProofLocalGivenUseBindingError> {
        if input.source_id != base_binding_env.source_id()
            || &input.module_id != base_binding_env.module_id()
        {
            return Err(SourceProofLocalGivenUseBindingError::InvalidTransaction);
        }
        validate_task269gup_dependency(Task269gupDependency {
            source_id: input.source_id,
            module_id: &input.module_id,
            lower_fingerprint: &input.lower_fingerprint,
            theorem_symbol: &input.theorem_symbol,
            theorem_definition: input.theorem_definition,
            contribution: input.contribution,
            theorem_range: input.theorem_range,
            proof_range: input.proof_range,
            given_range: input.given_range,
            segment_range: input.segment_range,
            name_range: input.name_range,
        })?;
        if !exact_task269c_base_binding_env(base_binding_env) {
            return Err(SourceProofLocalGivenUseBindingError::InvalidBaseBindingEnvironment);
        }
        if !exact_task269gup_input_declaration(&input) {
            return Err(SourceProofLocalGivenUseBindingError::InvalidDeclaration {
                binding: SourceProofLocalGivenBindingId::new(0),
            });
        }
        let binding_env = extend_task269gup_binding_env(base_binding_env)?;
        if !exact_task269gup_lookup_behavior(&binding_env) {
            return Err(SourceProofLocalGivenUseBindingError::InvalidBindingEnvironment);
        }
        let base_binding_fingerprint = base_binding_env.debug_text();
        let final_binding_fingerprint = binding_env.debug_text();
        Ok(SourceProofLocalGivenUseBindingHandoff {
            source_id: input.source_id,
            module_id: input.module_id,
            lower_fingerprint: input.lower_fingerprint,
            theorem_symbol: input.theorem_symbol,
            theorem_definition: input.theorem_definition,
            contribution: input.contribution,
            theorem_range: input.theorem_range,
            proof_range: input.proof_range,
            given_range: input.given_range,
            segment_range: input.segment_range,
            name_range: input.name_range,
            base_binding_env: base_binding_env.clone(),
            base_binding_fingerprint,
            binding_env,
            final_binding_fingerprint,
            bindings: SourceProofLocalGivenBindingTable {
                rows: vec![SourceProofLocalGivenBinding {
                    binding: BindingId::new(1),
                    binding_context: BindingContextId::new(1),
                    source_ordinal: input.source_ordinal,
                    visible_after_ordinal: input.local.visible_after_ordinal(),
                    recovery: input.recovery,
                }],
            },
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum SourceProofLocalGivenUseBindingError {
    InvalidTransaction,
    DependencyMismatch,
    InvalidBaseBindingEnvironment,
    InvalidAggregate,
    InvalidDeclaration {
        binding: SourceProofLocalGivenBindingId,
    },
    InvalidBindingEnvironment,
}

impl fmt::Display for SourceProofLocalGivenUseBindingError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidTransaction => {
                formatter.write_str("source proof-local given-use binding transaction is invalid")
            }
            Self::DependencyMismatch => {
                formatter.write_str("source proof-local given-use binding dependency mismatch")
            }
            Self::InvalidBaseBindingEnvironment => formatter.write_str(
                "source proof-local given-use binding base binding environment is invalid",
            ),
            Self::InvalidAggregate => {
                formatter.write_str("source proof-local given-use binding aggregate is invalid")
            }
            Self::InvalidDeclaration { binding } => write!(
                formatter,
                "source proof-local given-use binding {} is invalid",
                binding.index()
            ),
            Self::InvalidBindingEnvironment => formatter
                .write_str("source proof-local given-use binding binding environment is invalid"),
        }
    }
}

impl Error for SourceProofLocalGivenUseBindingError {}

#[derive(Debug, Clone, Copy)]
struct Task269gupDependency<'a> {
    source_id: SourceId,
    module_id: &'a ModuleId,
    lower_fingerprint: &'a str,
    theorem_symbol: &'a SymbolId,
    theorem_definition: DefinitionId,
    contribution: SourceContributionId,
    theorem_range: SourceRange,
    proof_range: SourceRange,
    given_range: SourceRange,
    segment_range: SourceRange,
    name_range: SourceRange,
}

fn validate_task269gup_dependency(
    dependency: Task269gupDependency<'_>,
) -> Result<(), SourceProofLocalGivenUseBindingError> {
    let expected_prefix = format!(
        "{}::{}::",
        dependency.module_id.package().as_str(),
        dependency.module_id.path().as_str()
    );
    let expected_local = format!(
        concat!(
            "contribution=0:namespace={}:owner=theorem#1:shell=theorem:kind=theorem:",
            "name=FormulaStatementGivenSmoke:notation=_:arity=_:definition=theorem:",
            "registration=_:policy=non-overloadable:slot=non-overloadable:_:theorem:_"
        ),
        escape_task269c_symbol_component(dependency.module_id.path().as_str()),
    );
    let exact_symbol_identity = dependency.theorem_symbol.module() == dependency.module_id
        && dependency.theorem_symbol.local().as_str() == expected_local
        && dependency.theorem_symbol.fqn().as_str() == format!("{expected_prefix}{expected_local}");
    if !exact_symbol_identity
        || dependency.theorem_definition.index() != 0
        || dependency.contribution.index() != 0
        || dependency.theorem_range != range(dependency.source_id, 19, 127)
        || dependency.proof_range != range(dependency.source_id, 62, 126)
        || dependency.given_range != range(dependency.source_id, 70, 108)
        || dependency.segment_range != range(dependency.source_id, 76, 87)
        || dependency.name_range != range(dependency.source_id, 76, 77)
        || dependency.lower_fingerprint != exact_task269gup_lower_fingerprint(dependency)
    {
        return Err(SourceProofLocalGivenUseBindingError::DependencyMismatch);
    }
    Ok(())
}

fn exact_task269gup_lower_fingerprint(dependency: Task269gupDependency<'_>) -> String {
    format!(
        concat!(
            "source-proof-local-given-use-lower-debug-v1\n",
            "module: {}::{}\n",
            "source-fingerprint: \"ec15ded78ae96022840a8419a85d74643de3b37337e9a202cbda77ee97aa7c01\"\n",
            "surface-fingerprint: \"c64297ce72e380a2e4146276966e085d780f8b38f2528d5abaa440a50c67db6d\"\n",
            "theorem symbol={:?} definition=0 contribution=0 range=19..127 proof=62..126\n",
            "given range=70..108 segment=76..87 source_ordinal=1\n",
            "name range=76..77 spelling=\"y\"\n",
            "type range=84..87 head=84..87 spelling=\"set\" form=bare\n",
        ),
        dependency.module_id.package().as_str(),
        dependency.module_id.path().as_str(),
        dependency.theorem_symbol.fqn().as_str(),
    )
}

fn exact_task269gup_input_declaration(input: &SourceProofLocalGivenUseBindingHandoffInput) -> bool {
    input.source_ordinal == 1
        && input.local.spelling() == "y"
        && input.local.scope().path() == [0]
        && input.local.declaration_range() == input.name_range
        && input.local.visible_after_ordinal() == 1
        && input.recovery == SourceProofLocalGivenBindingRecovery::Normal
}

// Rationale: Task 269GUP keeps final-row replay private until Task 269GUPT consumes it.
#[cfg_attr(not(test), allow(dead_code))]
fn exact_task269gup_output_binding(binding: &SourceProofLocalGivenBinding) -> bool {
    binding.binding == BindingId::new(1)
        && binding.binding_context == BindingContextId::new(1)
        && binding.source_ordinal == 1
        && binding.visible_after_ordinal == 1
        && binding.recovery == SourceProofLocalGivenBindingRecovery::Normal
}

// Rationale: Task 269GUP keeps final-environment replay private until Task 269GUPT.
#[cfg_attr(not(test), allow(dead_code))]
fn exact_task269gup_local_binding(binding_env: &BindingEnv, source_id: SourceId) -> bool {
    let Some(context) = binding_env.contexts().get(BindingContextId::new(1)) else {
        return false;
    };
    let Some(binding) = binding_env.bindings().get(BindingId::new(1)) else {
        return false;
    };
    context.id == BindingContextId::new(1)
        && context.owner
            == BindingContextOwner::SourceStatement {
                source_range: range(source_id, 62, 126),
            }
        && context.parent == Some(BindingContextId::new(0))
        && context.layer == BindingContextLayer::Proof
        && context
            .lexical_scope
            .as_ref()
            .is_some_and(|scope| scope.path() == [0])
        && context.bindings == [BindingId::new(1)]
        && context.visible_bindings == [BindingId::new(0), BindingId::new(1)]
        && context.recovery == BindingContextRecovery::Normal
        && binding.id == BindingId::new(1)
        && binding.spelling == "y"
        && binding.kind == BindingKind::GivenWitness
        && matches!(
            &binding.identity,
            BinderIdentity::ResolverLocal {
                scope,
                ordinal: 1,
                declaration_range,
            } if scope.path() == [0]
                && *declaration_range == range(source_id, 76, 77)
        )
        && binding.owner_context == BindingContextId::new(1)
        && binding.declaration_range == range(source_id, 76, 77)
        && binding.visible_after_ordinal == 1
        && binding.type_site == BindingTypeSite::Missing
        && binding.status == BindingStatus::Active
        && binding.captured.identities().is_empty()
        && binding.diagnostics.is_empty()
        && binding.recovery == BindingRecoveryState::Normal
}

// Rationale: Task 269GUP keeps declaration replay private until Task 269GUPT.
#[cfg_attr(not(test), allow(dead_code))]
fn exact_task269gup_declaration_binding(binding_env: &BindingEnv, source_id: SourceId) -> bool {
    binding_env
        .bindings()
        .get(BindingId::new(1))
        .is_some_and(|binding| {
            binding.id == BindingId::new(1)
                && binding.spelling == "y"
                && matches!(
                    &binding.identity,
                    BinderIdentity::ResolverLocal {
                        scope,
                        ordinal: 1,
                        declaration_range,
                    } if scope.path() == [0]
                        && *declaration_range == range(source_id, 76, 77)
                )
                && binding.owner_context == BindingContextId::new(1)
                && binding.declaration_range == range(source_id, 76, 77)
                && binding.visible_after_ordinal == 1
                && binding.recovery == BindingRecoveryState::Normal
        })
}

fn extend_task269gup_binding_env(
    base: &BindingEnv,
) -> Result<BindingEnv, SourceProofLocalGivenUseBindingError> {
    if !exact_task269c_base_binding_env(base) {
        return Err(SourceProofLocalGivenUseBindingError::InvalidBaseBindingEnvironment);
    }
    let local = LocalTermBinding::new(
        "y",
        LocalTermScope::new(vec![0]),
        range(base.source_id(), 76, 77),
        1,
    );
    let mut bindings = base.bindings().clone();
    let binding = bindings.insert(BindingDraft::from_local_term(
        BindingContextId::new(1),
        BindingKind::GivenWitness,
        &local,
    ));
    if binding != BindingId::new(1) {
        return Err(SourceProofLocalGivenUseBindingError::InvalidBindingEnvironment);
    }
    let mut contexts = base.contexts().clone();
    let context = contexts.insert(BindingContextDraft {
        owner: BindingContextOwner::SourceStatement {
            source_range: range(base.source_id(), 62, 126),
        },
        parent: Some(BindingContextId::new(0)),
        layer: BindingContextLayer::Proof,
        lexical_scope: Some(LocalTermScope::new(vec![0])),
        bindings: vec![binding],
        visible_bindings: vec![BindingId::new(0), binding],
        recovery: BindingContextRecovery::Normal,
    });
    if context != BindingContextId::new(1) {
        return Err(SourceProofLocalGivenUseBindingError::InvalidBindingEnvironment);
    }
    BindingEnv::try_new(BindingEnvParts {
        source_id: base.source_id(),
        module_id: base.module_id().clone(),
        contexts,
        bindings,
        diagnostics: base.diagnostics().clone(),
    })
    .map_err(|_| SourceProofLocalGivenUseBindingError::InvalidBindingEnvironment)
}

fn exact_task269gup_lookup_behavior(binding_env: &BindingEnv) -> bool {
    let definition = BindingLookupSite::new(
        "y",
        BindingContextId::new(1),
        Some(LocalTermScope::new(vec![0])),
        1,
    );
    let later = BindingLookupSite::new(
        "y",
        BindingContextId::new(1),
        Some(LocalTermScope::new(vec![0])),
        2,
    );
    matches!(
        binding_env.lookup(&definition),
        Ok(BindingLookupResult::ForwardReference { candidates, .. })
            if candidates == [BindingId::new(1)]
    ) && binding_env.lookup(&later) == Ok(BindingLookupResult::Local(BindingId::new(1)))
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceProofLocalGivenConditionBindingHandoffInput {
    pub source_id: SourceId,
    pub module_id: ModuleId,
    pub lower_fingerprint: String,
    pub theorem_symbol: SymbolId,
    pub theorem_definition: DefinitionId,
    pub contribution: SourceContributionId,
    pub theorem_range: SourceRange,
    pub proof_range: SourceRange,
    pub given_range: SourceRange,
    pub segment_range: SourceRange,
    pub name_range: SourceRange,
    pub source_ordinal: usize,
    pub local: LocalTermBinding,
    pub recovery: SourceProofLocalGivenBindingRecovery,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceProofLocalGivenConditionBindingHandoff {
    source_id: SourceId,
    module_id: ModuleId,
    lower_fingerprint: String,
    theorem_symbol: SymbolId,
    theorem_definition: DefinitionId,
    contribution: SourceContributionId,
    theorem_range: SourceRange,
    proof_range: SourceRange,
    given_range: SourceRange,
    segment_range: SourceRange,
    name_range: SourceRange,
    base_binding_env: BindingEnv,
    base_binding_fingerprint: String,
    binding_env: BindingEnv,
    final_binding_fingerprint: String,
    bindings: SourceProofLocalGivenBindingTable,
}

impl SourceProofLocalGivenConditionBindingHandoff {
    pub const fn source_id(&self) -> SourceId {
        self.source_id
    }

    pub const fn module_id(&self) -> &ModuleId {
        &self.module_id
    }

    pub fn lower_fingerprint(&self) -> &str {
        &self.lower_fingerprint
    }

    pub const fn theorem_symbol(&self) -> &SymbolId {
        &self.theorem_symbol
    }

    pub const fn theorem_definition(&self) -> DefinitionId {
        self.theorem_definition
    }

    pub const fn contribution(&self) -> SourceContributionId {
        self.contribution
    }

    pub const fn theorem_range(&self) -> SourceRange {
        self.theorem_range
    }

    pub const fn proof_range(&self) -> SourceRange {
        self.proof_range
    }

    pub const fn given_range(&self) -> SourceRange {
        self.given_range
    }

    pub const fn segment_range(&self) -> SourceRange {
        self.segment_range
    }

    pub const fn name_range(&self) -> SourceRange {
        self.name_range
    }

    pub const fn base_binding_env(&self) -> &BindingEnv {
        &self.base_binding_env
    }

    pub fn base_binding_fingerprint(&self) -> &str {
        &self.base_binding_fingerprint
    }

    pub const fn binding_env(&self) -> &BindingEnv {
        &self.binding_env
    }

    pub fn final_binding_fingerprint(&self) -> &str {
        &self.final_binding_fingerprint
    }

    pub const fn bindings(&self) -> &SourceProofLocalGivenBindingTable {
        &self.bindings
    }

    pub fn debug_text(&self) -> String {
        let binding = self
            .bindings
            .get(SourceProofLocalGivenBindingId::new(0))
            .expect("validated Task269GC handoff has one dense row");
        format!(
            concat!(
                "source-proof-local-given-condition-binding-debug-v1\n",
                "module: {}::{}\n",
                "lower-fingerprint: {:?}\n",
                "theorem symbol={:?} definition={} contribution={} range={}..{} proof={}..{}\n",
                "given range={}..{} segment={}..{} name={}..{} source_ordinal={}\n",
                "base-binding-fingerprint: {:?}\n",
                "binding#0 binding={} context={} source_ordinal={} visible_after={} recovery={}\n",
                "final-binding-fingerprint: {:?}\n",
            ),
            self.module_id.package().as_str(),
            self.module_id.path().as_str(),
            self.lower_fingerprint,
            self.theorem_symbol.fqn().as_str(),
            self.theorem_definition.index(),
            self.contribution.index(),
            self.theorem_range.start,
            self.theorem_range.end,
            self.proof_range.start,
            self.proof_range.end,
            self.given_range.start,
            self.given_range.end,
            self.segment_range.start,
            self.segment_range.end,
            self.name_range.start,
            self.name_range.end,
            binding.source_ordinal,
            self.base_binding_fingerprint,
            binding.binding.index(),
            binding.binding_context.index(),
            binding.source_ordinal,
            binding.visible_after_ordinal,
            given_binding_recovery_key(binding.recovery),
            self.final_binding_fingerprint,
        )
    }

    pub(crate) fn validate_installation(
        &self,
        source_id: SourceId,
        module_id: &ModuleId,
    ) -> Result<(), SourceProofLocalGivenConditionBindingError> {
        if self.source_id != source_id
            || &self.module_id != module_id
            || self.base_binding_env.source_id() != source_id
            || self.base_binding_env.module_id() != module_id
            || self.binding_env.source_id() != source_id
            || self.binding_env.module_id() != module_id
        {
            return Err(SourceProofLocalGivenConditionBindingError::InvalidTransaction);
        }
        validate_task269gc_dependency(Task269gcDependency {
            source_id: self.source_id,
            module_id: &self.module_id,
            lower_fingerprint: &self.lower_fingerprint,
            theorem_symbol: &self.theorem_symbol,
            theorem_definition: self.theorem_definition,
            contribution: self.contribution,
            theorem_range: self.theorem_range,
            proof_range: self.proof_range,
            given_range: self.given_range,
            segment_range: self.segment_range,
            name_range: self.name_range,
        })?;
        if !exact_task269c_base_binding_env(&self.base_binding_env)
            || self.base_binding_fingerprint != self.base_binding_env.debug_text()
        {
            return Err(SourceProofLocalGivenConditionBindingError::InvalidBaseBindingEnvironment);
        }
        if self.bindings.len() != 1 {
            return Err(SourceProofLocalGivenConditionBindingError::InvalidAggregate);
        }
        let id = SourceProofLocalGivenBindingId::new(0);
        let binding = self.bindings.get(id).ok_or(
            SourceProofLocalGivenConditionBindingError::InvalidDeclaration { binding: id },
        )?;
        if !exact_task269gc_output_binding(binding)
            || !exact_task269gc_declaration_binding(&self.binding_env, self.source_id)
        {
            return Err(
                SourceProofLocalGivenConditionBindingError::InvalidDeclaration { binding: id },
            );
        }
        let expected = extend_task269gc_binding_env(&self.base_binding_env)?;
        if self.binding_env != expected
            || !exact_task269gc_local_binding(&self.binding_env, self.source_id)
            || !exact_task269gc_lookup_behavior(&self.binding_env)
            || self.final_binding_fingerprint != self.binding_env.debug_text()
        {
            return Err(SourceProofLocalGivenConditionBindingError::InvalidBindingEnvironment);
        }
        Ok(())
    }

    pub(crate) fn validate_complete_installation(
        &self,
        source_id: SourceId,
        module_id: &ModuleId,
        installation_available: bool,
    ) -> Result<(), SourceProofLocalGivenConditionBindingError> {
        self.validate_installation(source_id, module_id)?;
        if !installation_available {
            return Err(SourceProofLocalGivenConditionBindingError::InvalidInstallation);
        }
        Ok(())
    }

    #[cfg(test)]
    pub(crate) fn set_lower_fingerprint_for_task269gc_test(&mut self, value: impl Into<String>) {
        self.lower_fingerprint = value.into();
    }

    #[cfg(test)]
    pub(crate) fn set_base_binding_fingerprint_for_task269gc_test(
        &mut self,
        value: impl Into<String>,
    ) {
        self.base_binding_fingerprint = value.into();
    }

    #[cfg(test)]
    pub(crate) fn truncate_task269gc_bindings_for_test(&mut self) {
        self.bindings.rows.clear();
    }

    #[cfg(test)]
    pub(crate) fn corrupt_task269gc_binding_row_for_test(&mut self) {
        if let Some(binding) = self.bindings.rows.first_mut() {
            binding.source_ordinal += 1;
        }
    }

    #[cfg(test)]
    pub(crate) fn set_final_binding_fingerprint_for_task269gc_test(
        &mut self,
        value: impl Into<String>,
    ) {
        self.final_binding_fingerprint = value.into();
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct SourceProofLocalGivenConditionBindingProducer;

impl SourceProofLocalGivenConditionBindingProducer {
    pub fn build(
        input: SourceProofLocalGivenConditionBindingHandoffInput,
        base_binding_env: &BindingEnv,
    ) -> Result<
        SourceProofLocalGivenConditionBindingHandoff,
        SourceProofLocalGivenConditionBindingError,
    > {
        if input.source_id != base_binding_env.source_id()
            || &input.module_id != base_binding_env.module_id()
        {
            return Err(SourceProofLocalGivenConditionBindingError::InvalidTransaction);
        }
        validate_task269gc_dependency(Task269gcDependency {
            source_id: input.source_id,
            module_id: &input.module_id,
            lower_fingerprint: &input.lower_fingerprint,
            theorem_symbol: &input.theorem_symbol,
            theorem_definition: input.theorem_definition,
            contribution: input.contribution,
            theorem_range: input.theorem_range,
            proof_range: input.proof_range,
            given_range: input.given_range,
            segment_range: input.segment_range,
            name_range: input.name_range,
        })?;
        if !exact_task269c_base_binding_env(base_binding_env) {
            return Err(SourceProofLocalGivenConditionBindingError::InvalidBaseBindingEnvironment);
        }
        if !exact_task269gc_input_declaration(&input) {
            return Err(
                SourceProofLocalGivenConditionBindingError::InvalidDeclaration {
                    binding: SourceProofLocalGivenBindingId::new(0),
                },
            );
        }
        let binding_env = extend_task269gc_binding_env(base_binding_env)?;
        if !exact_task269gc_lookup_behavior(&binding_env) {
            return Err(SourceProofLocalGivenConditionBindingError::InvalidBindingEnvironment);
        }
        let base_binding_fingerprint = base_binding_env.debug_text();
        let final_binding_fingerprint = binding_env.debug_text();
        Ok(SourceProofLocalGivenConditionBindingHandoff {
            source_id: input.source_id,
            module_id: input.module_id,
            lower_fingerprint: input.lower_fingerprint,
            theorem_symbol: input.theorem_symbol,
            theorem_definition: input.theorem_definition,
            contribution: input.contribution,
            theorem_range: input.theorem_range,
            proof_range: input.proof_range,
            given_range: input.given_range,
            segment_range: input.segment_range,
            name_range: input.name_range,
            base_binding_env: base_binding_env.clone(),
            base_binding_fingerprint,
            binding_env,
            final_binding_fingerprint,
            bindings: SourceProofLocalGivenBindingTable {
                rows: vec![SourceProofLocalGivenBinding {
                    binding: BindingId::new(1),
                    binding_context: BindingContextId::new(1),
                    source_ordinal: input.source_ordinal,
                    visible_after_ordinal: input.local.visible_after_ordinal(),
                    recovery: input.recovery,
                }],
            },
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum SourceProofLocalGivenConditionBindingError {
    InvalidTransaction,
    DependencyMismatch,
    InvalidBaseBindingEnvironment,
    InvalidAggregate,
    InvalidDeclaration {
        binding: SourceProofLocalGivenBindingId,
    },
    InvalidBindingEnvironment,
    InvalidInstallation,
}

impl fmt::Display for SourceProofLocalGivenConditionBindingError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidTransaction => formatter
                .write_str("source proof-local given-condition binding transaction is invalid"),
            Self::DependencyMismatch => formatter
                .write_str("source proof-local given-condition binding dependency mismatch"),
            Self::InvalidBaseBindingEnvironment => formatter.write_str(
                "source proof-local given-condition binding base binding environment is invalid",
            ),
            Self::InvalidAggregate => formatter
                .write_str("source proof-local given-condition binding aggregate is invalid"),
            Self::InvalidDeclaration { binding } => write!(
                formatter,
                "source proof-local given-condition binding {} is invalid",
                binding.index()
            ),
            Self::InvalidBindingEnvironment => formatter.write_str(
                "source proof-local given-condition binding binding environment is invalid",
            ),
            Self::InvalidInstallation => formatter
                .write_str("source proof-local given-condition binding installation is invalid"),
        }
    }
}

impl Error for SourceProofLocalGivenConditionBindingError {}

#[derive(Debug, Clone, Copy)]
struct Task269gcDependency<'a> {
    source_id: SourceId,
    module_id: &'a ModuleId,
    lower_fingerprint: &'a str,
    theorem_symbol: &'a SymbolId,
    theorem_definition: DefinitionId,
    contribution: SourceContributionId,
    theorem_range: SourceRange,
    proof_range: SourceRange,
    given_range: SourceRange,
    segment_range: SourceRange,
    name_range: SourceRange,
}

fn validate_task269gc_dependency(
    dependency: Task269gcDependency<'_>,
) -> Result<(), SourceProofLocalGivenConditionBindingError> {
    let expected_prefix = format!(
        "{}::{}::",
        dependency.module_id.package().as_str(),
        dependency.module_id.path().as_str()
    );
    let expected_local = format!(
        concat!(
            "contribution=0:namespace={}:owner=theorem#1:shell=theorem:kind=theorem:",
            "name=ProofLocalGivenConditionUseSmoke:notation=_:arity=_:definition=theorem:",
            "registration=_:policy=non-overloadable:slot=non-overloadable:_:theorem:_"
        ),
        escape_task269c_symbol_component(dependency.module_id.path().as_str()),
    );
    let expected_fqn = format!("{expected_prefix}{expected_local}");
    let exact_symbol_identity = dependency.theorem_symbol.module() == dependency.module_id
        && dependency.theorem_symbol.local().as_str() == expected_local
        && dependency.theorem_symbol.fqn().as_str() == expected_fqn;
    if !exact_symbol_identity
        || dependency.theorem_definition.index() != 0
        || dependency.contribution.index() != 0
        || dependency.theorem_range != range(dependency.source_id, 19, 133)
        || dependency.proof_range != range(dependency.source_id, 68, 132)
        || dependency.given_range != range(dependency.source_id, 76, 113)
        || dependency.segment_range != range(dependency.source_id, 82, 93)
        || dependency.name_range != range(dependency.source_id, 82, 83)
        || dependency.lower_fingerprint
            != exact_task269gc_lower_fingerprint(dependency.module_id, &expected_fqn)
    {
        return Err(SourceProofLocalGivenConditionBindingError::DependencyMismatch);
    }
    Ok(())
}

fn exact_task269gc_lower_fingerprint(module_id: &ModuleId, expected_fqn: &str) -> String {
    format!(
        concat!(
            "source-proof-local-given-condition-lower-debug-v1\n",
            "module: {}::{}\n",
            "source-fingerprint: \"2c2d767a0654670412b377bdcc6c5970ecec05b41c02aa754766320927bc6aad\"\n",
            "surface-fingerprint: \"49d46d5f24338772e6e968f12c2216a8957b35242474132690db843b510b430f\"\n",
            "theorem symbol={:?} definition=0 contribution=0 range=19..133 proof=68..132\n",
            "given range=76..113 segment=82..93 source_ordinal=1\n",
            "name range=82..83 spelling=\"y\"\n",
            "type range=90..93 head=90..93 spelling=\"set\" form=bare\n",
        ),
        module_id.package().as_str(),
        module_id.path().as_str(),
        expected_fqn,
    )
}

fn exact_task269gc_input_declaration(
    input: &SourceProofLocalGivenConditionBindingHandoffInput,
) -> bool {
    input.source_ordinal == 1
        && input.local.spelling() == "y"
        && input.local.scope().path() == [0]
        && input.local.declaration_range() == input.name_range
        && input.local.visible_after_ordinal() == 1
        && input.recovery == SourceProofLocalGivenBindingRecovery::Normal
}

fn exact_task269gc_output_binding(binding: &SourceProofLocalGivenBinding) -> bool {
    binding.binding == BindingId::new(1)
        && binding.binding_context == BindingContextId::new(1)
        && binding.source_ordinal == 1
        && binding.visible_after_ordinal == 1
        && binding.recovery == SourceProofLocalGivenBindingRecovery::Normal
}

fn exact_task269gc_local_binding(binding_env: &BindingEnv, source_id: SourceId) -> bool {
    let Some(context) = binding_env.contexts().get(BindingContextId::new(1)) else {
        return false;
    };
    let Some(binding) = binding_env.bindings().get(BindingId::new(1)) else {
        return false;
    };
    context.id == BindingContextId::new(1)
        && context.owner
            == BindingContextOwner::SourceStatement {
                source_range: range(source_id, 68, 132),
            }
        && context.parent == Some(BindingContextId::new(0))
        && context.layer == BindingContextLayer::Proof
        && context
            .lexical_scope
            .as_ref()
            .is_some_and(|scope| scope.path() == [0])
        && context.bindings == [BindingId::new(1)]
        && context.visible_bindings == [BindingId::new(0), BindingId::new(1)]
        && context.recovery == BindingContextRecovery::Normal
        && binding.id == BindingId::new(1)
        && binding.spelling == "y"
        && binding.kind == BindingKind::GivenWitness
        && matches!(
            &binding.identity,
            BinderIdentity::ResolverLocal {
                scope,
                ordinal: 1,
                declaration_range,
            } if scope.path() == [0]
                && *declaration_range == range(source_id, 82, 83)
        )
        && binding.owner_context == BindingContextId::new(1)
        && binding.declaration_range == range(source_id, 82, 83)
        && binding.visible_after_ordinal == 1
        && binding.type_site == BindingTypeSite::Missing
        && binding.status == BindingStatus::Active
        && binding.captured.identities().is_empty()
        && binding.diagnostics.is_empty()
        && binding.recovery == BindingRecoveryState::Normal
}

fn exact_task269gc_declaration_binding(binding_env: &BindingEnv, source_id: SourceId) -> bool {
    binding_env
        .bindings()
        .get(BindingId::new(1))
        .is_some_and(|binding| {
            binding.id == BindingId::new(1)
                && binding.spelling == "y"
                && matches!(
                    &binding.identity,
                    BinderIdentity::ResolverLocal {
                        scope,
                        ordinal: 1,
                        declaration_range,
                    } if scope.path() == [0]
                        && *declaration_range == range(source_id, 82, 83)
                )
                && binding.owner_context == BindingContextId::new(1)
                && binding.declaration_range == range(source_id, 82, 83)
                && binding.visible_after_ordinal == 1
                && binding.recovery == BindingRecoveryState::Normal
        })
}

fn extend_task269gc_binding_env(
    base: &BindingEnv,
) -> Result<BindingEnv, SourceProofLocalGivenConditionBindingError> {
    if !exact_task269c_base_binding_env(base) {
        return Err(SourceProofLocalGivenConditionBindingError::InvalidBaseBindingEnvironment);
    }
    let local = LocalTermBinding::new(
        "y",
        LocalTermScope::new(vec![0]),
        range(base.source_id(), 82, 83),
        1,
    );
    let mut bindings = base.bindings().clone();
    let binding = bindings.insert(BindingDraft::from_local_term(
        BindingContextId::new(1),
        BindingKind::GivenWitness,
        &local,
    ));
    if binding != BindingId::new(1) {
        return Err(SourceProofLocalGivenConditionBindingError::InvalidBindingEnvironment);
    }
    let mut contexts = base.contexts().clone();
    let context = contexts.insert(BindingContextDraft {
        owner: BindingContextOwner::SourceStatement {
            source_range: range(base.source_id(), 68, 132),
        },
        parent: Some(BindingContextId::new(0)),
        layer: BindingContextLayer::Proof,
        lexical_scope: Some(LocalTermScope::new(vec![0])),
        bindings: vec![binding],
        visible_bindings: vec![BindingId::new(0), binding],
        recovery: BindingContextRecovery::Normal,
    });
    if context != BindingContextId::new(1) {
        return Err(SourceProofLocalGivenConditionBindingError::InvalidBindingEnvironment);
    }
    BindingEnv::try_new(BindingEnvParts {
        source_id: base.source_id(),
        module_id: base.module_id().clone(),
        contexts,
        bindings,
        diagnostics: base.diagnostics().clone(),
    })
    .map_err(|_| SourceProofLocalGivenConditionBindingError::InvalidBindingEnvironment)
}

fn exact_task269gc_lookup_behavior(binding_env: &BindingEnv) -> bool {
    let prior = BindingLookupSite::new(
        "y",
        BindingContextId::new(1),
        Some(LocalTermScope::new(vec![0])),
        1,
    );
    let visible = BindingLookupSite::new(
        "y",
        BindingContextId::new(1),
        Some(LocalTermScope::new(vec![0])),
        2,
    );
    matches!(
        binding_env.lookup(&prior),
        Ok(BindingLookupResult::ForwardReference { candidates, .. })
            if candidates == [BindingId::new(1)]
    ) && binding_env.lookup(&visible) == Ok(BindingLookupResult::Local(BindingId::new(1)))
}

const fn given_binding_recovery_key(
    recovery: SourceProofLocalGivenBindingRecovery,
) -> &'static str {
    match recovery {
        SourceProofLocalGivenBindingRecovery::Normal => "normal",
    }
}

#[cfg(test)]
mod task269c_tests {
    use super::*;
    use crate::{
        binding_env::{
            BindingContextLayer, BindingContextOwner, BindingContextRecovery, BindingDiagnosticId,
            BindingDiagnosticTable, BindingStatus, BindingTable, BindingTypeSite,
            CapturedFreeVariables,
        },
        cluster_trace::ClusterFactTable,
        overload_resolution::{
            CandidateViabilityInput, CandidateViabilityOutput, OverloadCandidateInput,
            OverloadCollectionOutput, OverloadSelectionOutput, OverloadSiteInput,
            OverloadSiteResolutionInput, SpecificityComparisonInput, SpecificityGraphOutput,
            TemplateExpansionOutput,
        },
        resolved_typed_ast::{
            ResolvedNodeKindHint, ResolvedNodeKindHintKind, ResolvedTypedAst,
            ResolvedTypedAstError, ResolvedTypedAstInputs, SourceNodeRole,
        },
        source_term::{SourcePrimaryTermHandoffInput, SourcePrimaryTermProducer},
        typed_ast::{
            CoercionTable, InitialObligationTable, LocalTypeContextTable,
            StatementTransportTableForTest, TypeDiagnosticTable, TypeFactTable, TypeTable,
            TypedArena, TypedArenaBuilder, TypedAst, TypedAstError, TypedAstParts, TypedNode,
            TypedNodeId,
        },
    };
    use mizar_resolve::{
        env::{
            ContributionKind, DefinitionIndex, DefinitionKind, DefinitionShell,
            SourceContributionIndex,
        },
        resolved_ast::{FullyQualifiedName, LocalSymbolId, SemanticOrigin},
    };
    use mizar_session::{
        BuildSnapshotId, InMemorySessionIdAllocator, ModulePath, PackageId, SessionIdAllocator,
        SourceAnchor,
    };

    #[test]
    fn exact_producer_output_debug_and_lookup_are_stable() {
        let fixture = fixture();
        let handoff = SourceProofLocalLetBindingProducer::build(fixture.input, &fixture.base)
            .expect("Task269C exact checker transaction");
        assert_eq!(handoff.source_id(), fixture.source);
        assert_eq!(handoff.module_id(), &fixture.module);
        assert_eq!(handoff.base_binding_env(), &fixture.base);
        assert_eq!(
            handoff.base_binding_fingerprint(),
            fixture.base.debug_text()
        );
        assert_eq!(
            (
                handoff.binding_env().contexts().len(),
                handoff.binding_env().bindings().len(),
                handoff.binding_env().diagnostics().len(),
            ),
            (2, 2, 0)
        );
        let row = handoff
            .bindings()
            .get(SourceProofLocalLetBindingId::new(0))
            .expect("Task269C dense binding row");
        assert!(exact_task269c_output_binding(row));
        assert!(exact_task269c_lookup_behavior(handoff.binding_env()));
        let expected_debug = format!(
            concat!(
                "source-proof-local-let-binding-debug-v1\n",
                "module: pkg::task269c\n",
                "lower-fingerprint: {:?}\n",
                "theorem symbol={:?} definition=0 contribution=0 range=19..99 proof=59..98\n",
                "let range=67..80 segment=71..79 name=71..72 source_ordinal=1\n",
                "base-binding-fingerprint: {:?}\n",
                "binding#0 binding=1 context=1 source_ordinal=1 visible_after=1 recovery=normal\n",
                "final-binding-fingerprint: {:?}\n",
            ),
            handoff.lower_fingerprint(),
            handoff.theorem_symbol().fqn().as_str(),
            handoff.base_binding_fingerprint(),
            handoff.final_binding_fingerprint(),
        );
        assert_eq!(handoff.debug_text(), expected_debug);
    }

    #[test]
    fn corruption_classes_follow_the_frozen_precedence() {
        let fixture = fixture();
        let mut wrong_transaction = fixture.input.clone();
        wrong_transaction.source_id = other_source_id();
        assert_eq!(
            SourceProofLocalLetBindingProducer::build(wrong_transaction, &fixture.base),
            Err(SourceProofLocalLetBindingError::InvalidTransaction)
        );

        let mut wrong_dependency = fixture.input.clone();
        wrong_dependency.lower_fingerprint.push_str("corrupt");
        assert_eq!(
            SourceProofLocalLetBindingProducer::build(wrong_dependency, &empty_base(&fixture)),
            Err(SourceProofLocalLetBindingError::DependencyMismatch)
        );
        let mut dependency_inputs = Vec::new();
        let mut input = fixture.input.clone();
        let extra_local = format!("{}extra", input.theorem_symbol.local().as_str());
        input.theorem_symbol = SymbolId::new(
            fixture.module.clone(),
            LocalSymbolId::new(extra_local.clone()),
            FullyQualifiedName::new(format!("pkg::task269c::{extra_local}")),
        );
        dependency_inputs.push(input);
        let mut input = fixture.input.clone();
        input.theorem_symbol = SymbolId::new(
            fixture.module.clone(),
            input.theorem_symbol.local().clone(),
            FullyQualifiedName::new(format!("{}extra", input.theorem_symbol.fqn().as_str())),
        );
        dependency_inputs.push(input);
        let mut input = fixture.input.clone();
        input.theorem_symbol = SymbolId::new(
            ModuleId::new(PackageId::new("pkg"), ModulePath::new("task269c.wrong")),
            input.theorem_symbol.local().clone(),
            input.theorem_symbol.fqn().clone(),
        );
        dependency_inputs.push(input);
        let mut input = fixture.input.clone();
        input.theorem_definition = fixture.other_definition;
        dependency_inputs.push(input);
        let mut input = fixture.input.clone();
        input.contribution = fixture.other_contribution;
        dependency_inputs.push(input);
        for select in 0..5 {
            let mut input = fixture.input.clone();
            let target = match select {
                0 => &mut input.theorem_range,
                1 => &mut input.proof_range,
                2 => &mut input.let_range,
                3 => &mut input.segment_range,
                4 => &mut input.name_range,
                _ => unreachable!(),
            };
            target.end += 1;
            dependency_inputs.push(input);
        }
        for input in dependency_inputs {
            assert_eq!(
                SourceProofLocalLetBindingProducer::build(input, &fixture.base),
                Err(SourceProofLocalLetBindingError::DependencyMismatch)
            );
        }

        let mut declaration_inputs = Vec::new();
        let mut input = fixture.input.clone();
        input.source_ordinal += 1;
        declaration_inputs.push(input);
        let mut input = fixture.input.clone();
        input.local = LocalTermBinding::new(
            "z",
            input.local.scope().clone(),
            input.local.declaration_range(),
            input.local.visible_after_ordinal(),
        );
        declaration_inputs.push(input);
        let mut input = fixture.input.clone();
        input.local = LocalTermBinding::new(
            input.local.spelling(),
            LocalTermScope::new(vec![1]),
            input.local.declaration_range(),
            input.local.visible_after_ordinal(),
        );
        declaration_inputs.push(input);
        let mut input = fixture.input.clone();
        input.local = LocalTermBinding::new(
            input.local.spelling(),
            input.local.scope().clone(),
            range(fixture.source, 70, 72),
            input.local.visible_after_ordinal(),
        );
        declaration_inputs.push(input);
        let mut input = fixture.input.clone();
        input.local = LocalTermBinding::new(
            input.local.spelling(),
            input.local.scope().clone(),
            input.local.declaration_range(),
            2,
        );
        declaration_inputs.push(input);
        for input in declaration_inputs {
            assert_eq!(
                SourceProofLocalLetBindingProducer::build(input, &fixture.base),
                Err(SourceProofLocalLetBindingError::InvalidDeclaration {
                    binding: SourceProofLocalLetBindingId::new(0),
                })
            );
        }

        assert_eq!(
            SourceProofLocalLetBindingProducer::build(fixture.input.clone(), &empty_base(&fixture),),
            Err(SourceProofLocalLetBindingError::InvalidBaseBindingEnvironment)
        );
        for base in inexact_bases(&fixture) {
            assert_eq!(
                SourceProofLocalLetBindingProducer::build(fixture.input.clone(), &base),
                Err(SourceProofLocalLetBindingError::InvalidBaseBindingEnvironment)
            );
        }

        let handoff = SourceProofLocalLetBindingProducer::build(fixture.input, &fixture.base)
            .expect("Task269C handoff before replay corruption");
        let mut dependency = handoff.clone();
        dependency.set_lower_fingerprint_for_test("corrupt");
        assert_eq!(
            dependency.validate_installation(fixture.source, &fixture.module),
            Err(SourceProofLocalLetBindingError::DependencyMismatch)
        );
        let mut base_fingerprint = handoff.clone();
        base_fingerprint.set_base_binding_fingerprint_for_task269c_test("corrupt");
        assert_eq!(
            base_fingerprint.validate_installation(fixture.source, &fixture.module),
            Err(SourceProofLocalLetBindingError::InvalidBaseBindingEnvironment)
        );
        let mut aggregate = handoff.clone();
        aggregate.truncate_task269c_bindings_for_test();
        assert_eq!(
            aggregate.validate_installation(fixture.source, &fixture.module),
            Err(SourceProofLocalLetBindingError::InvalidAggregate)
        );
        let mut declaration = handoff.clone();
        declaration.corrupt_task269c_binding_row_for_test();
        assert_eq!(
            declaration.validate_installation(fixture.source, &fixture.module),
            Err(SourceProofLocalLetBindingError::InvalidDeclaration {
                binding: SourceProofLocalLetBindingId::new(0),
            })
        );
        let mut row_corruptions = Vec::new();
        let mut corrupted = handoff.clone();
        corrupted.bindings.rows[0].binding = BindingId::new(0);
        row_corruptions.push(corrupted);
        let mut corrupted = handoff.clone();
        corrupted.bindings.rows[0].binding_context = BindingContextId::new(0);
        row_corruptions.push(corrupted);
        let mut corrupted = handoff.clone();
        corrupted.bindings.rows[0].visible_after_ordinal = 2;
        row_corruptions.push(corrupted);
        for corrupted in row_corruptions {
            assert_eq!(
                corrupted.validate_installation(fixture.source, &fixture.module),
                Err(SourceProofLocalLetBindingError::InvalidDeclaration {
                    binding: SourceProofLocalLetBindingId::new(0),
                })
            );
        }
        for corrupted in inexact_final_declaration_handoffs(&handoff, fixture.source) {
            assert_eq!(
                corrupted.validate_installation(fixture.source, &fixture.module),
                Err(SourceProofLocalLetBindingError::InvalidDeclaration {
                    binding: SourceProofLocalLetBindingId::new(0),
                })
            );
        }
        for corrupted in inexact_final_environment_handoffs(&handoff, fixture.source) {
            assert_eq!(
                corrupted.validate_installation(fixture.source, &fixture.module),
                Err(SourceProofLocalLetBindingError::InvalidBindingEnvironment)
            );
        }
        let mut wrong_final_module = handoff.clone();
        wrong_final_module.binding_env = BindingEnv::try_new(BindingEnvParts {
            source_id: handoff.binding_env.source_id(),
            module_id: ModuleId::new(PackageId::new("pkg"), ModulePath::new("task269c.wrong")),
            contexts: handoff.binding_env.contexts().clone(),
            bindings: handoff.binding_env.bindings().clone(),
            diagnostics: handoff.binding_env.diagnostics().clone(),
        })
        .expect("Task269C final module corruption remains structurally valid");
        assert_eq!(
            wrong_final_module.validate_installation(fixture.source, &fixture.module),
            Err(SourceProofLocalLetBindingError::InvalidTransaction)
        );
        let mut binding_env = handoff.clone();
        binding_env.set_final_binding_fingerprint_for_task269c_test("corrupt");
        assert_eq!(
            binding_env.validate_installation(fixture.source, &fixture.module),
            Err(SourceProofLocalLetBindingError::InvalidBindingEnvironment)
        );
        assert_eq!(
            handoff.validate_complete_installation(fixture.source, &fixture.module, false),
            Err(SourceProofLocalLetBindingError::InvalidInstallation)
        );
        for (error, expected) in [
            (
                SourceProofLocalLetBindingError::InvalidTransaction,
                "source proof-local let-binding transaction is invalid",
            ),
            (
                SourceProofLocalLetBindingError::DependencyMismatch,
                "source proof-local let-binding dependency mismatch",
            ),
            (
                SourceProofLocalLetBindingError::InvalidBaseBindingEnvironment,
                "source proof-local let-binding base binding environment is invalid",
            ),
            (
                SourceProofLocalLetBindingError::InvalidAggregate,
                "source proof-local let-binding aggregate is invalid",
            ),
            (
                SourceProofLocalLetBindingError::InvalidDeclaration {
                    binding: SourceProofLocalLetBindingId::new(0),
                },
                "source proof-local let-binding 0 is invalid",
            ),
            (
                SourceProofLocalLetBindingError::InvalidBindingEnvironment,
                "source proof-local let-binding binding environment is invalid",
            ),
            (
                SourceProofLocalLetBindingError::InvalidInstallation,
                "source proof-local let-binding installation is invalid",
            ),
        ] {
            assert_eq!(error.to_string(), expected);
        }
    }

    #[test]
    fn typed_and_resolved_ownership_is_one_shot_and_replayed() {
        let fixture = fixture();
        let handoff =
            SourceProofLocalLetBindingProducer::build(fixture.input.clone(), &fixture.base)
                .expect("Task269C checker handoff");
        let empty = empty_typed(fixture.source, fixture.module.clone());
        let typed = empty
            .clone()
            .with_source_proof_local_let_binding(handoff.clone())
            .expect("Task269C typed installation");
        assert_eq!(typed.source_proof_local_let_binding(), Some(&handoff));
        assert_eq!(
            typed
                .clone()
                .with_source_proof_local_let_binding(handoff.clone()),
            Err(TypedAstError::InvalidSourceProofLocalLetBinding)
        );
        assert!(empty.source_proof_local_let_binding().is_none());

        let family = task269ab_dummy_handoff(&fixture);
        let mut forward_cross_family = empty.clone();
        forward_cross_family.inject_source_proof_local_declaration_for_test(family.clone());
        assert_eq!(
            forward_cross_family.with_source_proof_local_let_binding(handoff.clone()),
            Err(TypedAstError::InvalidSourceProofLocalLetBinding)
        );
        assert_eq!(
            typed
                .clone()
                .with_source_proof_local_declaration(family.clone()),
            Err(TypedAstError::InvalidSourceProofLocalDeclaration)
        );

        let empty_term = SourcePrimaryTermProducer::build(
            SourcePrimaryTermHandoffInput {
                source_id: fixture.source,
                module_id: fixture.module.clone(),
                terms: Vec::new(),
                references: Vec::new(),
                numeric_type_requests: Vec::new(),
            },
            &fixture.base,
            typed.nodes(),
        )
        .expect("Task269C reverse-order empty term");
        assert_eq!(
            typed.clone().with_source_term(empty_term),
            Err(TypedAstError::InvalidSourceTerm)
        );

        let resolved = assemble_empty(&typed).expect("Task269C resolved replay");
        assert_eq!(resolved.source_proof_local_let_binding(), Some(&handoff));
        assert!(resolved.debug_text().contains(&handoff.debug_text()));

        let mut resolved_cross_family = typed.clone();
        resolved_cross_family.inject_source_proof_local_declaration_for_test(family);
        assert_eq!(
            assemble_empty(&resolved_cross_family),
            Err(ResolvedTypedAstError::InvalidSourceProofLocalLetBinding)
        );
        assert_eq!(
            assemble_with_node_hints(
                &typed,
                vec![ResolvedNodeKindHint {
                    typed_node: TypedNodeId::new(0),
                    kind: ResolvedNodeKindHintKind::SourcePreserved {
                        role: SourceNodeRole::new("Task269C.forbidden"),
                    },
                }],
            ),
            Err(ResolvedTypedAstError::InvalidSourceProofLocalLetBinding)
        );
        let mut stale_handoff = handoff.clone();
        stale_handoff.set_lower_fingerprint_for_test("stale");
        let mut stale_typed = empty.clone();
        stale_typed.inject_source_proof_local_let_binding_for_test(stale_handoff);
        assert_eq!(
            assemble_empty(&stale_typed),
            Err(ResolvedTypedAstError::InvalidSourceProofLocalLetBinding)
        );
        assert!(empty.source_proof_local_let_binding().is_none());

        let occupied = occupied_typed(fixture.source, fixture.module);
        assert_eq!(
            occupied.with_source_proof_local_let_binding(handoff),
            Err(TypedAstError::InvalidSourceProofLocalLetBinding)
        );
    }

    #[test]
    fn missing_type_and_empty_semantic_profile_are_preserved() {
        let fixture = fixture();
        let handoff = SourceProofLocalLetBindingProducer::build(fixture.input, &fixture.base)
            .expect("Task269C checker handoff");
        let binding = handoff
            .binding_env()
            .bindings()
            .get(BindingId::new(1))
            .expect("Task269C local binding");
        assert_eq!(binding.kind, BindingKind::LetBinding);
        assert_eq!(binding.type_site, BindingTypeSite::Missing);
        assert!(binding.captured.identities().is_empty());
        assert!(binding.diagnostics.is_empty());

        let typed = empty_typed(fixture.source, fixture.module)
            .with_source_proof_local_let_binding(handoff)
            .expect("Task269C typed installation");
        assert!(typed.nodes().is_empty());
        assert!(typed.contexts().is_empty());
        assert!(typed.types().is_empty());
        assert!(typed.facts().is_empty());
        assert!(typed.coercions().is_empty());
        assert!(typed.initial_obligations().is_empty());
        assert!(typed.diagnostics().is_empty());
        let resolved = assemble_empty(&typed).expect("Task269C resolved assembly");
        assert!(resolved.nodes().is_empty());
        assert!(resolved.expr_metadata().is_empty());
        assert!(resolved.checked_formulas().is_empty());
        assert!(resolved.statement_semantics().is_empty());
        assert!(resolved.checked_proofs().is_empty());
        assert!(resolved.checked_terminal_goals().is_empty());
        assert!(resolved.diagnostics().is_empty());
        for forbidden in [
            "initial-obligation#",
            "fact#",
            "terminal-goal#",
            "accepted",
            "discharged",
        ] {
            assert!(!resolved.debug_text().contains(forbidden));
        }
    }

    #[test]
    fn source_proof_local_given_binding_builds_exact_scope_transaction() {
        let fixture = given_fixture();
        let handoff = SourceProofLocalGivenBindingProducer::build(fixture.input, &fixture.base)
            .expect("Task269G exact checker transaction");
        assert_eq!(handoff.source_id(), fixture.source);
        assert_eq!(handoff.module_id(), &fixture.module);
        assert_eq!(handoff.base_binding_env(), &fixture.base);
        assert_eq!(
            handoff.base_binding_fingerprint(),
            fixture.base.debug_text()
        );
        assert_eq!(
            (
                handoff.binding_env().contexts().len(),
                handoff.binding_env().bindings().len(),
                handoff.binding_env().diagnostics().len(),
            ),
            (2, 2, 0)
        );
        assert!(
            handoff
                .base_binding_fingerprint()
                .contains("reserved_variable")
        );
        assert!(!handoff.base_binding_fingerprint().contains("given_witness"));
        assert!(
            handoff
                .final_binding_fingerprint()
                .contains("given_witness")
        );

        let row = handoff
            .bindings()
            .get(SourceProofLocalGivenBindingId::new(0))
            .expect("Task269G dense binding row");
        assert!(exact_task269g_output_binding(row));
        assert_eq!(
            handoff.bindings().iter().collect::<Vec<_>>(),
            [(SourceProofLocalGivenBindingId::new(0), row)]
        );
        assert!(exact_task269g_local_binding(
            handoff.binding_env(),
            fixture.source
        ));
        assert!(exact_task269g_lookup_behavior(handoff.binding_env()));
        assert_eq!(
            handoff.binding_env().lookup(&BindingLookupSite::new(
                "y",
                BindingContextId::new(1),
                Some(LocalTermScope::new(vec![0])),
                1,
            )),
            Ok(BindingLookupResult::ForwardReference {
                candidates: vec![BindingId::new(1)],
                diagnostic: crate::binding_env::BindingDiagnosticDraft {
                    source_range: None,
                    class: crate::binding_env::BindingDiagnosticClass::ForwardLocalReference,
                    severity: crate::binding_env::BindingDiagnosticSeverity::Error,
                    message_key: "checker.binding.forward_reference".to_owned(),
                    recovery: crate::binding_env::BindingDiagnosticRecovery::Degraded,
                },
            })
        );
        for ordinal in [2, 3] {
            assert_eq!(
                handoff.binding_env().lookup(&BindingLookupSite::new(
                    "y",
                    BindingContextId::new(1),
                    Some(LocalTermScope::new(vec![0])),
                    ordinal,
                )),
                Ok(BindingLookupResult::Local(BindingId::new(1)))
            );
        }

        let expected_debug = format!(
            concat!(
                "source-proof-local-given-binding-debug-v1\n",
                "module: pkg::task269g\n",
                "lower-fingerprint: {:?}\n",
                "theorem symbol={:?} definition=0 contribution=0 range=19..128 proof=62..127\n",
                "given range=70..108 segment=76..87 name=76..77 source_ordinal=1\n",
                "base-binding-fingerprint: {:?}\n",
                "binding#0 binding=1 context=1 source_ordinal=1 visible_after=1 recovery=normal\n",
                "final-binding-fingerprint: {:?}\n",
            ),
            handoff.lower_fingerprint(),
            handoff.theorem_symbol().fqn().as_str(),
            handoff.base_binding_fingerprint(),
            handoff.final_binding_fingerprint(),
        );
        assert_eq!(handoff.debug_text(), expected_debug);
    }

    #[test]
    fn source_proof_local_given_binding_rejects_corruption_with_stable_precedence() {
        let fixture = given_fixture();

        let mut transaction_first = fixture.input.clone();
        transaction_first.source_id = other_source_id();
        transaction_first.lower_fingerprint.push_str("corrupt");
        assert_eq!(
            SourceProofLocalGivenBindingProducer::build(
                transaction_first,
                &empty_given_base(&fixture),
            ),
            Err(SourceProofLocalGivenBindingError::InvalidTransaction)
        );

        let mut dependency_first = fixture.input.clone();
        dependency_first.lower_fingerprint.push_str("corrupt");
        dependency_first.source_ordinal += 1;
        assert_eq!(
            SourceProofLocalGivenBindingProducer::build(
                dependency_first,
                &empty_given_base(&fixture),
            ),
            Err(SourceProofLocalGivenBindingError::DependencyMismatch)
        );
        let mut dependency_inputs = Vec::new();
        let mut input = fixture.input.clone();
        input.theorem_definition = fixture.other_definition;
        dependency_inputs.push(input);
        let mut input = fixture.input.clone();
        input.contribution = fixture.other_contribution;
        dependency_inputs.push(input);
        for select in 0..5 {
            let mut input = fixture.input.clone();
            let target = match select {
                0 => &mut input.theorem_range,
                1 => &mut input.proof_range,
                2 => &mut input.given_range,
                3 => &mut input.segment_range,
                4 => &mut input.name_range,
                _ => unreachable!(),
            };
            target.end += 1;
            dependency_inputs.push(input);
        }
        for input in dependency_inputs {
            assert_eq!(
                SourceProofLocalGivenBindingProducer::build(input, &fixture.base),
                Err(SourceProofLocalGivenBindingError::DependencyMismatch)
            );
        }

        let mut base_first = fixture.input.clone();
        base_first.source_ordinal += 1;
        assert_eq!(
            SourceProofLocalGivenBindingProducer::build(base_first, &empty_given_base(&fixture),),
            Err(SourceProofLocalGivenBindingError::InvalidBaseBindingEnvironment)
        );
        let mut wrong_local = fixture.input.clone();
        wrong_local.local = LocalTermBinding::new(
            "z",
            wrong_local.local.scope().clone(),
            wrong_local.local.declaration_range(),
            wrong_local.local.visible_after_ordinal(),
        );
        assert_eq!(
            SourceProofLocalGivenBindingProducer::build(wrong_local, &fixture.base),
            Err(SourceProofLocalGivenBindingError::InvalidDeclaration {
                binding: SourceProofLocalGivenBindingId::new(0),
            })
        );

        let handoff = SourceProofLocalGivenBindingProducer::build(fixture.input, &fixture.base)
            .expect("Task269G handoff before replay corruption");
        let mut transaction = handoff.clone();
        transaction.source_id = other_source_id();
        transaction.set_lower_fingerprint_for_test("corrupt");
        assert_eq!(
            transaction.validate_installation(fixture.source, &fixture.module),
            Err(SourceProofLocalGivenBindingError::InvalidTransaction)
        );
        let mut dependency = handoff.clone();
        dependency.set_lower_fingerprint_for_test("corrupt");
        dependency.set_base_binding_fingerprint_for_task269g_test("corrupt");
        assert_eq!(
            dependency.validate_installation(fixture.source, &fixture.module),
            Err(SourceProofLocalGivenBindingError::DependencyMismatch)
        );
        let mut base = handoff.clone();
        base.set_base_binding_fingerprint_for_task269g_test("corrupt");
        base.truncate_task269g_bindings_for_test();
        assert_eq!(
            base.validate_installation(fixture.source, &fixture.module),
            Err(SourceProofLocalGivenBindingError::InvalidBaseBindingEnvironment)
        );
        let mut aggregate = handoff.clone();
        aggregate.truncate_task269g_bindings_for_test();
        aggregate.set_final_binding_fingerprint_for_task269g_test("corrupt");
        assert_eq!(
            aggregate.validate_installation(fixture.source, &fixture.module),
            Err(SourceProofLocalGivenBindingError::InvalidAggregate)
        );
        let mut declaration = handoff.clone();
        declaration.corrupt_task269g_binding_row_for_test();
        declaration.set_final_binding_fingerprint_for_task269g_test("corrupt");
        assert_eq!(
            declaration.validate_installation(fixture.source, &fixture.module),
            Err(SourceProofLocalGivenBindingError::InvalidDeclaration {
                binding: SourceProofLocalGivenBindingId::new(0),
            })
        );
        let mut final_fingerprint = handoff.clone();
        final_fingerprint.set_final_binding_fingerprint_for_task269g_test("corrupt");
        assert_eq!(
            final_fingerprint.validate_installation(fixture.source, &fixture.module),
            Err(SourceProofLocalGivenBindingError::InvalidBindingEnvironment)
        );
        assert_eq!(
            final_fingerprint.validate_complete_installation(
                fixture.source,
                &fixture.module,
                false,
            ),
            Err(SourceProofLocalGivenBindingError::InvalidBindingEnvironment)
        );

        let mut contexts = BindingContextTable::new();
        contexts.insert(BindingContextDraft {
            owner: BindingContextOwner::Module,
            parent: None,
            layer: BindingContextLayer::Module,
            lexical_scope: None,
            bindings: vec![BindingId::new(0)],
            visible_bindings: vec![BindingId::new(0)],
            recovery: BindingContextRecovery::Normal,
        });
        contexts.insert(BindingContextDraft {
            owner: BindingContextOwner::SourceStatement {
                source_range: range(fixture.source, 62, 127),
            },
            parent: Some(BindingContextId::new(0)),
            layer: BindingContextLayer::Proof,
            lexical_scope: Some(LocalTermScope::new(vec![0])),
            bindings: vec![BindingId::new(1)],
            visible_bindings: vec![BindingId::new(0)],
            recovery: BindingContextRecovery::Normal,
        });
        let mut lookup_environment = handoff.clone();
        lookup_environment.binding_env = BindingEnv::try_new(BindingEnvParts {
            source_id: fixture.source,
            module_id: fixture.module.clone(),
            contexts,
            bindings: handoff.binding_env().bindings().clone(),
            diagnostics: handoff.binding_env().diagnostics().clone(),
        })
        .expect("Task269G structurally valid lookup corruption");
        lookup_environment.final_binding_fingerprint = lookup_environment.binding_env.debug_text();
        assert_eq!(
            lookup_environment.validate_installation(fixture.source, &fixture.module),
            Err(SourceProofLocalGivenBindingError::InvalidBindingEnvironment)
        );
        assert_eq!(
            handoff.validate_complete_installation(fixture.source, &fixture.module, false),
            Err(SourceProofLocalGivenBindingError::InvalidInstallation)
        );

        for (error, expected) in [
            (
                SourceProofLocalGivenBindingError::InvalidTransaction,
                "source proof-local given-binding transaction is invalid",
            ),
            (
                SourceProofLocalGivenBindingError::DependencyMismatch,
                "source proof-local given-binding dependency mismatch",
            ),
            (
                SourceProofLocalGivenBindingError::InvalidBaseBindingEnvironment,
                "source proof-local given-binding base binding environment is invalid",
            ),
            (
                SourceProofLocalGivenBindingError::InvalidAggregate,
                "source proof-local given-binding aggregate is invalid",
            ),
            (
                SourceProofLocalGivenBindingError::InvalidDeclaration {
                    binding: SourceProofLocalGivenBindingId::new(0),
                },
                "source proof-local given-binding 0 is invalid",
            ),
            (
                SourceProofLocalGivenBindingError::InvalidBindingEnvironment,
                "source proof-local given-binding binding environment is invalid",
            ),
            (
                SourceProofLocalGivenBindingError::InvalidInstallation,
                "source proof-local given-binding installation is invalid",
            ),
        ] {
            assert_eq!(error.to_string(), expected);
        }
    }

    #[test]
    fn source_proof_local_given_binding_typed_and_resolved_ownership_is_atomic() {
        let fixture = given_fixture();
        let handoff = SourceProofLocalGivenBindingProducer::build(fixture.input, &fixture.base)
            .expect("Task269G checker handoff");
        let empty = empty_typed(fixture.source, fixture.module.clone());
        let typed = empty
            .clone()
            .with_source_proof_local_given_binding(handoff.clone())
            .expect("Task269G typed installation");
        assert_eq!(typed.source_proof_local_given_binding(), Some(&handoff));
        assert_eq!(
            typed
                .clone()
                .with_source_proof_local_given_binding(handoff.clone()),
            Err(TypedAstError::InvalidSourceProofLocalGivenBinding)
        );
        assert!(empty.source_proof_local_given_binding().is_none());

        let let_fixture = self::fixture();
        let same_module_given_fixture = given_fixture_for_module(let_fixture.module.clone());
        let same_module_given = SourceProofLocalGivenBindingProducer::build(
            same_module_given_fixture.input,
            &same_module_given_fixture.base,
        )
        .expect("Task269G same-module cross-family given handoff");
        let let_handoff =
            SourceProofLocalLetBindingProducer::build(let_fixture.input, &let_fixture.base)
                .expect("Task269G cross-family let handoff");
        assert_eq!(
            typed
                .clone()
                .with_source_proof_local_let_binding(let_handoff.clone()),
            Err(TypedAstError::InvalidSourceProofLocalLetBinding)
        );
        let let_typed = empty_typed(let_fixture.source, let_fixture.module)
            .with_source_proof_local_let_binding(let_handoff)
            .expect("Task269G reverse cross-family let owner");
        let mut resolved_cross_family = let_typed.clone();
        resolved_cross_family.inject_source_proof_local_given_binding_for_test(same_module_given);
        assert_eq!(
            assemble_empty(&resolved_cross_family),
            Err(ResolvedTypedAstError::InvalidSourceProofLocalLetBinding)
        );
        assert_eq!(
            let_typed.with_source_proof_local_given_binding(handoff.clone()),
            Err(TypedAstError::InvalidSourceProofLocalGivenBinding)
        );

        let resolved = assemble_empty(&typed).expect("Task269G resolved replay");
        assert_eq!(resolved.source_proof_local_given_binding(), Some(&handoff));
        assert!(resolved.debug_text().contains(&handoff.debug_text()));

        let mut stale_handoff = handoff.clone();
        stale_handoff.set_lower_fingerprint_for_test("stale");
        let mut stale_typed = empty.clone();
        stale_typed.inject_source_proof_local_given_binding_for_test(stale_handoff);
        assert_eq!(
            assemble_empty(&stale_typed),
            Err(ResolvedTypedAstError::InvalidSourceProofLocalGivenBinding)
        );
        assert_eq!(
            assemble_with_node_hints(
                &typed,
                vec![ResolvedNodeKindHint {
                    typed_node: TypedNodeId::new(0),
                    kind: ResolvedNodeKindHintKind::SourcePreserved {
                        role: SourceNodeRole::new("Task269G.forbidden"),
                    },
                }],
            ),
            Err(ResolvedTypedAstError::InvalidSourceProofLocalGivenBinding)
        );
        assert_eq!(
            occupied_typed(fixture.source, fixture.module)
                .with_source_proof_local_given_binding(handoff.clone()),
            Err(TypedAstError::InvalidSourceProofLocalGivenBinding)
        );
        for table in [
            StatementTransportTableForTest::Context,
            StatementTransportTableForTest::Type,
            StatementTransportTableForTest::Fact,
            StatementTransportTableForTest::Coercion,
            StatementTransportTableForTest::InitialObligation,
            StatementTransportTableForTest::Diagnostic,
        ] {
            let mut occupied = empty.clone();
            occupied.occupy_statement_transport_table_for_test(table);
            assert_eq!(
                occupied.with_source_proof_local_given_binding(handoff.clone()),
                Err(TypedAstError::InvalidSourceProofLocalGivenBinding),
                "Task269G unexpectedly accepted occupied {table:?} table",
            );
        }
        assert_eq!(
            TypedAstError::InvalidSourceProofLocalGivenBinding.to_string(),
            "typed AST source proof-local given-binding handoff is inconsistent"
        );
        assert_eq!(
            ResolvedTypedAstError::InvalidSourceProofLocalGivenBinding.to_string(),
            "resolved typed AST source proof-local given-binding handoff is inconsistent"
        );
    }

    #[test]
    fn source_proof_local_given_binding_scope_matrix_is_lexical_and_semantically_empty() {
        let fixture = given_fixture();
        let handoff = SourceProofLocalGivenBindingProducer::build(fixture.input, &fixture.base)
            .expect("Task269G checker handoff");
        let binding = handoff
            .binding_env()
            .bindings()
            .get(BindingId::new(1))
            .expect("Task269G witness binding");
        assert_eq!(binding.kind, BindingKind::GivenWitness);
        assert_eq!(binding.type_site, BindingTypeSite::Missing);
        assert!(binding.captured.identities().is_empty());
        assert!(binding.diagnostics.is_empty());

        let mut bindings = handoff.binding_env().bindings().clone();
        let shadow = bindings.insert(BindingDraft {
            spelling: "y".to_owned(),
            kind: BindingKind::GivenWitness,
            identity: BinderIdentity::ResolverLocal {
                scope: LocalTermScope::new(vec![0, 1]),
                ordinal: 2,
                declaration_range: range(fixture.source, 109, 110),
            },
            owner_context: BindingContextId::new(3),
            declaration_range: range(fixture.source, 109, 110),
            visible_after_ordinal: 2,
            type_site: BindingTypeSite::Missing,
            status: BindingStatus::Active,
            captured: CapturedFreeVariables::default(),
            diagnostics: Vec::new(),
            recovery: BindingRecoveryState::Normal,
        });
        assert_eq!(shadow, BindingId::new(2));
        let mut contexts = handoff.binding_env().contexts().clone();
        let child = contexts.insert(BindingContextDraft {
            owner: BindingContextOwner::Generated("task269g-unshadowed-child".to_owned()),
            parent: Some(BindingContextId::new(1)),
            layer: BindingContextLayer::Block,
            lexical_scope: Some(LocalTermScope::new(vec![0, 0])),
            bindings: Vec::new(),
            visible_bindings: vec![BindingId::new(0), BindingId::new(1)],
            recovery: BindingContextRecovery::Normal,
        });
        let shadow_child = contexts.insert(BindingContextDraft {
            owner: BindingContextOwner::Generated("task269g-shadow-child".to_owned()),
            parent: Some(BindingContextId::new(1)),
            layer: BindingContextLayer::Block,
            lexical_scope: Some(LocalTermScope::new(vec![0, 1])),
            bindings: vec![shadow],
            visible_bindings: vec![BindingId::new(0), BindingId::new(1), shadow],
            recovery: BindingContextRecovery::Normal,
        });
        let sibling = contexts.insert(BindingContextDraft {
            owner: BindingContextOwner::Generated("task269g-sibling".to_owned()),
            parent: Some(BindingContextId::new(0)),
            layer: BindingContextLayer::Block,
            lexical_scope: Some(LocalTermScope::new(vec![1])),
            bindings: Vec::new(),
            visible_bindings: vec![BindingId::new(0)],
            recovery: BindingContextRecovery::Normal,
        });
        assert_eq!(
            (child, shadow_child, sibling),
            (
                BindingContextId::new(2),
                BindingContextId::new(3),
                BindingContextId::new(4),
            )
        );
        let matrix = BindingEnv::try_new(BindingEnvParts {
            source_id: fixture.source,
            module_id: fixture.module.clone(),
            contexts,
            bindings,
            diagnostics: handoff.binding_env().diagnostics().clone(),
        })
        .expect("Task269G exact synthetic scope matrix");
        assert_eq!(
            matrix.lookup(&BindingLookupSite::new(
                "y",
                child,
                Some(LocalTermScope::new(vec![0, 0])),
                2,
            )),
            Ok(BindingLookupResult::Local(BindingId::new(1)))
        );
        assert_eq!(
            matrix.lookup(&BindingLookupSite::new(
                "y",
                shadow_child,
                Some(LocalTermScope::new(vec![0, 1])),
                3,
            )),
            Ok(BindingLookupResult::Local(BindingId::new(2)))
        );
        assert_eq!(
            matrix.lookup(&BindingLookupSite::new(
                "y",
                BindingContextId::new(1),
                Some(LocalTermScope::new(vec![0])),
                3,
            )),
            Ok(BindingLookupResult::Local(BindingId::new(1)))
        );
        assert_eq!(
            matrix.lookup(&BindingLookupSite::new(
                "y",
                BindingContextId::new(0),
                Some(LocalTermScope::new(Vec::new())),
                2,
            )),
            Ok(BindingLookupResult::Unresolved)
        );
        assert_eq!(
            matrix.lookup(&BindingLookupSite::new(
                "y",
                sibling,
                Some(LocalTermScope::new(vec![1])),
                2,
            )),
            Ok(BindingLookupResult::Unresolved)
        );

        let typed = empty_typed(fixture.source, fixture.module)
            .with_source_proof_local_given_binding(handoff)
            .expect("Task269G typed installation");
        assert!(typed.nodes().is_empty());
        assert!(typed.contexts().is_empty());
        assert!(typed.types().is_empty());
        assert!(typed.facts().is_empty());
        assert!(typed.coercions().is_empty());
        assert!(typed.initial_obligations().is_empty());
        assert!(typed.diagnostics().is_empty());
        let resolved = assemble_empty(&typed).expect("Task269G resolved assembly");
        assert!(resolved.nodes().is_empty());
        assert!(resolved.expr_metadata().is_empty());
        assert!(resolved.checked_formulas().is_empty());
        assert!(resolved.statement_semantics().is_empty());
        assert!(resolved.checked_proofs().is_empty());
        assert!(resolved.checked_terminal_goals().is_empty());
        assert!(resolved.diagnostics().is_empty());
        for forbidden in [
            "initial-obligation#",
            "fact#",
            "terminal-goal#",
            "accepted",
            "discharged",
        ] {
            assert!(!resolved.debug_text().contains(forbidden));
        }
    }

    #[test]
    fn source_proof_local_given_use_binding_is_exact_and_new_source_local() {
        let fixture = given_use_fixture();
        let handoff =
            SourceProofLocalGivenUseBindingProducer::build(fixture.input.clone(), &fixture.base)
                .expect("Task269GUP exact checker handoff");
        handoff
            .validate_installation(fixture.source, &fixture.module)
            .expect("Task269GUP exact installation");

        assert_eq!(handoff.source_id(), fixture.source);
        assert_eq!(handoff.module_id(), &fixture.module);
        assert_eq!(handoff.lower_fingerprint(), fixture.input.lower_fingerprint);
        assert_eq!(handoff.theorem_symbol(), &fixture.input.theorem_symbol);
        assert_eq!(
            handoff.theorem_definition(),
            fixture.input.theorem_definition
        );
        assert_eq!(handoff.contribution(), fixture.input.contribution);
        assert_eq!(handoff.theorem_range(), range(fixture.source, 19, 127));
        assert_eq!(handoff.proof_range(), range(fixture.source, 62, 126));
        assert_eq!(handoff.given_range(), range(fixture.source, 70, 108));
        assert_eq!(handoff.segment_range(), range(fixture.source, 76, 87));
        assert_eq!(handoff.name_range(), range(fixture.source, 76, 77));
        assert_eq!(handoff.base_binding_env(), &fixture.base);
        assert_eq!(
            handoff.base_binding_fingerprint(),
            fixture.base.debug_text()
        );
        assert_eq!(handoff.binding_env().contexts().len(), 2);
        assert_eq!(handoff.binding_env().bindings().len(), 2);
        assert!(handoff.binding_env().diagnostics().is_empty());
        assert_eq!(
            handoff.final_binding_fingerprint(),
            handoff.binding_env().debug_text()
        );
        let row = handoff
            .bindings()
            .get(SourceProofLocalGivenBindingId::new(0))
            .expect("Task269GUP dense binding row");
        assert!(exact_task269gup_output_binding(row));
        assert_eq!(
            handoff.bindings().iter().collect::<Vec<_>>(),
            [(SourceProofLocalGivenBindingId::new(0), row)]
        );
        assert!(exact_task269gup_local_binding(
            handoff.binding_env(),
            fixture.source
        ));
        assert!(exact_task269gup_lookup_behavior(handoff.binding_env()));
        let expected_debug = format!(
            concat!(
                "source-proof-local-given-use-binding-debug-v1\n",
                "module: {}::{}\n",
                "lower-fingerprint: {:?}\n",
                "theorem symbol={:?} definition=0 contribution=0 range=19..127 proof=62..126\n",
                "given range=70..108 segment=76..87 name=76..77 source_ordinal=1\n",
                "base-binding-fingerprint: {:?}\n",
                "binding#0 binding=1 context=1 source_ordinal=1 visible_after=1 recovery=normal\n",
                "final-binding-fingerprint: {:?}\n",
            ),
            fixture.module.package().as_str(),
            fixture.module.path().as_str(),
            fixture.input.lower_fingerprint,
            fixture.input.theorem_symbol.fqn().as_str(),
            handoff.base_binding_fingerprint(),
            handoff.final_binding_fingerprint(),
        );
        assert_eq!(handoff.debug_text(), expected_debug);
        assert_eq!(handoff.debug_text(), handoff.clone().debug_text());
        assert!(handoff.debug_text().ends_with('\n'));
        assert!(!handoff.debug_text().ends_with("\n\n"));

        let old = given_fixture();
        assert_ne!(fixture.source, other_source_id());
        assert_ne!(fixture.input.lower_fingerprint, old.input.lower_fingerprint);
        assert_ne!(
            fixture.input.theorem_range, old.input.theorem_range,
            "Task269GUP must retain its distinct 128-byte source transaction"
        );
    }

    #[test]
    fn source_proof_local_given_use_binding_rejects_every_corruption_in_precedence() {
        let fixture = given_use_fixture();

        let mut wrong_source = fixture.input.clone();
        wrong_source.source_id = other_source_id();
        assert_eq!(
            SourceProofLocalGivenUseBindingProducer::build(wrong_source, &fixture.base),
            Err(SourceProofLocalGivenUseBindingError::InvalidTransaction)
        );
        let mut wrong_module = fixture.input.clone();
        wrong_module.module_id = ModuleId::new(PackageId::new("pkg"), ModulePath::new("wrong"));
        assert_eq!(
            SourceProofLocalGivenUseBindingProducer::build(wrong_module, &fixture.base),
            Err(SourceProofLocalGivenUseBindingError::InvalidTransaction)
        );

        let mut dependency_inputs = Vec::new();
        let mut input = fixture.input.clone();
        input.lower_fingerprint.push('!');
        dependency_inputs.push(input);
        let mut input = fixture.input.clone();
        input.theorem_symbol = SymbolId::new(
            input.module_id.clone(),
            LocalSymbolId::new("wrong"),
            FullyQualifiedName::new("pkg::task269gup::wrong"),
        );
        dependency_inputs.push(input);
        let mut input = fixture.input.clone();
        input.theorem_definition = fixture.other_definition;
        dependency_inputs.push(input);
        let mut input = fixture.input.clone();
        input.contribution = fixture.other_contribution;
        dependency_inputs.push(input);
        for select in 0..5 {
            let mut input = fixture.input.clone();
            match select {
                0 => input.theorem_range.end += 1,
                1 => input.proof_range.end += 1,
                2 => input.given_range.end += 1,
                3 => input.segment_range.end += 1,
                4 => input.name_range.end += 1,
                _ => unreachable!(),
            }
            dependency_inputs.push(input);
        }
        for input in dependency_inputs {
            assert_eq!(
                SourceProofLocalGivenUseBindingProducer::build(input, &fixture.base),
                Err(SourceProofLocalGivenUseBindingError::DependencyMismatch)
            );
        }

        assert_eq!(
            SourceProofLocalGivenUseBindingProducer::build(
                fixture.input.clone(),
                &empty_given_use_base(&fixture),
            ),
            Err(SourceProofLocalGivenUseBindingError::InvalidBaseBindingEnvironment)
        );
        for local in [
            LocalTermBinding::new(
                "z",
                LocalTermScope::new(vec![0]),
                range(fixture.source, 76, 77),
                1,
            ),
            LocalTermBinding::new(
                "y",
                LocalTermScope::new(vec![1]),
                range(fixture.source, 76, 77),
                1,
            ),
            LocalTermBinding::new(
                "y",
                LocalTermScope::new(vec![0]),
                range(fixture.source, 75, 77),
                1,
            ),
            LocalTermBinding::new(
                "y",
                LocalTermScope::new(vec![0]),
                range(fixture.source, 76, 77),
                2,
            ),
        ] {
            let mut input = fixture.input.clone();
            input.local = local;
            assert_eq!(
                SourceProofLocalGivenUseBindingProducer::build(input, &fixture.base),
                Err(SourceProofLocalGivenUseBindingError::InvalidDeclaration {
                    binding: SourceProofLocalGivenBindingId::new(0),
                })
            );
        }
        let mut input = fixture.input.clone();
        input.source_ordinal = 2;
        assert_eq!(
            SourceProofLocalGivenUseBindingProducer::build(input, &fixture.base),
            Err(SourceProofLocalGivenUseBindingError::InvalidDeclaration {
                binding: SourceProofLocalGivenBindingId::new(0),
            })
        );

        let mut wrong_transaction_and_dependency = fixture.input.clone();
        wrong_transaction_and_dependency.source_id = other_source_id();
        wrong_transaction_and_dependency.lower_fingerprint = "corrupt".to_owned();
        assert_eq!(
            SourceProofLocalGivenUseBindingProducer::build(
                wrong_transaction_and_dependency,
                &fixture.base,
            ),
            Err(SourceProofLocalGivenUseBindingError::InvalidTransaction)
        );
        let mut wrong_dependency_and_declaration = fixture.input.clone();
        wrong_dependency_and_declaration.lower_fingerprint = "corrupt".to_owned();
        wrong_dependency_and_declaration.source_ordinal = 2;
        assert_eq!(
            SourceProofLocalGivenUseBindingProducer::build(
                wrong_dependency_and_declaration,
                &empty_given_use_base(&fixture),
            ),
            Err(SourceProofLocalGivenUseBindingError::DependencyMismatch)
        );
        let mut wrong_base_and_declaration = fixture.input.clone();
        wrong_base_and_declaration.source_ordinal = 2;
        assert_eq!(
            SourceProofLocalGivenUseBindingProducer::build(
                wrong_base_and_declaration,
                &empty_given_use_base(&fixture),
            ),
            Err(SourceProofLocalGivenUseBindingError::InvalidBaseBindingEnvironment)
        );

        let handoff =
            SourceProofLocalGivenUseBindingProducer::build(fixture.input.clone(), &fixture.base)
                .expect("Task269GUP checker handoff");
        let mut transaction = handoff.clone();
        transaction.source_id = other_source_id();
        transaction.set_lower_fingerprint_for_task269gup_test("corrupt");
        assert_eq!(
            transaction.validate_installation(fixture.source, &fixture.module),
            Err(SourceProofLocalGivenUseBindingError::InvalidTransaction)
        );
        let mut dependency = handoff.clone();
        dependency.set_lower_fingerprint_for_task269gup_test("corrupt");
        dependency.set_base_binding_fingerprint_for_task269gup_test("corrupt");
        assert_eq!(
            dependency.validate_installation(fixture.source, &fixture.module),
            Err(SourceProofLocalGivenUseBindingError::DependencyMismatch)
        );
        let mut base = handoff.clone();
        base.set_base_binding_fingerprint_for_task269gup_test("corrupt");
        base.truncate_task269gup_bindings_for_test();
        assert_eq!(
            base.validate_installation(fixture.source, &fixture.module),
            Err(SourceProofLocalGivenUseBindingError::InvalidBaseBindingEnvironment)
        );
        let mut aggregate = handoff.clone();
        aggregate.truncate_task269gup_bindings_for_test();
        aggregate.set_final_binding_fingerprint_for_task269gup_test("corrupt");
        assert_eq!(
            aggregate.validate_installation(fixture.source, &fixture.module),
            Err(SourceProofLocalGivenUseBindingError::InvalidAggregate)
        );
        let mut declaration = handoff.clone();
        declaration.corrupt_task269gup_binding_row_for_test();
        declaration.set_final_binding_fingerprint_for_task269gup_test("corrupt");
        assert_eq!(
            declaration.validate_installation(fixture.source, &fixture.module),
            Err(SourceProofLocalGivenUseBindingError::InvalidDeclaration {
                binding: SourceProofLocalGivenBindingId::new(0),
            })
        );
        let mut environment = handoff;
        environment.set_final_binding_fingerprint_for_task269gup_test("corrupt");
        assert_eq!(
            environment.validate_installation(fixture.source, &fixture.module),
            Err(SourceProofLocalGivenUseBindingError::InvalidBindingEnvironment)
        );
        let mut contexts = BindingContextTable::new();
        contexts.insert(BindingContextDraft {
            owner: BindingContextOwner::Module,
            parent: None,
            layer: BindingContextLayer::Module,
            lexical_scope: None,
            bindings: vec![BindingId::new(0)],
            visible_bindings: vec![BindingId::new(0)],
            recovery: BindingContextRecovery::Normal,
        });
        contexts.insert(BindingContextDraft {
            owner: BindingContextOwner::SourceStatement {
                source_range: range(fixture.source, 62, 126),
            },
            parent: Some(BindingContextId::new(0)),
            layer: BindingContextLayer::Proof,
            lexical_scope: Some(LocalTermScope::new(vec![0])),
            bindings: vec![BindingId::new(1)],
            visible_bindings: vec![BindingId::new(0)],
            recovery: BindingContextRecovery::Normal,
        });
        let mut lookup_environment = environment.clone();
        lookup_environment.binding_env = BindingEnv::try_new(BindingEnvParts {
            source_id: fixture.source,
            module_id: fixture.module.clone(),
            contexts,
            bindings: environment.binding_env.bindings().clone(),
            diagnostics: environment.binding_env.diagnostics().clone(),
        })
        .expect("Task269GUP structurally valid lookup corruption");
        lookup_environment.final_binding_fingerprint = lookup_environment.binding_env.debug_text();
        assert_eq!(
            lookup_environment.validate_installation(fixture.source, &fixture.module),
            Err(SourceProofLocalGivenUseBindingError::InvalidBindingEnvironment)
        );

        for (error, expected) in [
            (
                SourceProofLocalGivenUseBindingError::InvalidTransaction,
                "source proof-local given-use binding transaction is invalid".to_owned(),
            ),
            (
                SourceProofLocalGivenUseBindingError::DependencyMismatch,
                "source proof-local given-use binding dependency mismatch".to_owned(),
            ),
            (
                SourceProofLocalGivenUseBindingError::InvalidBaseBindingEnvironment,
                "source proof-local given-use binding base binding environment is invalid"
                    .to_owned(),
            ),
            (
                SourceProofLocalGivenUseBindingError::InvalidAggregate,
                "source proof-local given-use binding aggregate is invalid".to_owned(),
            ),
            (
                SourceProofLocalGivenUseBindingError::InvalidDeclaration {
                    binding: SourceProofLocalGivenBindingId::new(0),
                },
                "source proof-local given-use binding 0 is invalid".to_owned(),
            ),
            (
                SourceProofLocalGivenUseBindingError::InvalidBindingEnvironment,
                "source proof-local given-use binding binding environment is invalid".to_owned(),
            ),
        ] {
            assert_eq!(error.to_string(), expected);
        }
    }

    #[test]
    fn source_proof_local_given_use_binding_inherits_shadows_restores_and_excludes() {
        let fixture = given_use_fixture();
        let handoff = SourceProofLocalGivenUseBindingProducer::build(fixture.input, &fixture.base)
            .expect("Task269GUP checker handoff");
        let mut bindings = handoff.binding_env().bindings().clone();
        let shadow = bindings.insert(BindingDraft {
            spelling: "y".to_owned(),
            kind: BindingKind::GivenWitness,
            identity: BinderIdentity::ResolverLocal {
                scope: LocalTermScope::new(vec![0, 1]),
                ordinal: 2,
                declaration_range: range(fixture.source, 109, 110),
            },
            owner_context: BindingContextId::new(3),
            declaration_range: range(fixture.source, 109, 110),
            visible_after_ordinal: 2,
            type_site: BindingTypeSite::Missing,
            status: BindingStatus::Active,
            captured: CapturedFreeVariables::default(),
            diagnostics: Vec::new(),
            recovery: BindingRecoveryState::Normal,
        });
        assert_eq!(shadow, BindingId::new(2));
        let mut contexts = handoff.binding_env().contexts().clone();
        let child = contexts.insert(BindingContextDraft {
            owner: BindingContextOwner::Generated("task269gup-child".to_owned()),
            parent: Some(BindingContextId::new(1)),
            layer: BindingContextLayer::Block,
            lexical_scope: Some(LocalTermScope::new(vec![0, 0])),
            bindings: Vec::new(),
            visible_bindings: vec![BindingId::new(0), BindingId::new(1)],
            recovery: BindingContextRecovery::Normal,
        });
        let shadow_child = contexts.insert(BindingContextDraft {
            owner: BindingContextOwner::Generated("task269gup-shadow".to_owned()),
            parent: Some(BindingContextId::new(1)),
            layer: BindingContextLayer::Block,
            lexical_scope: Some(LocalTermScope::new(vec![0, 1])),
            bindings: vec![shadow],
            visible_bindings: vec![BindingId::new(0), BindingId::new(1), shadow],
            recovery: BindingContextRecovery::Normal,
        });
        let sibling = contexts.insert(BindingContextDraft {
            owner: BindingContextOwner::Generated("task269gup-sibling".to_owned()),
            parent: Some(BindingContextId::new(0)),
            layer: BindingContextLayer::Block,
            lexical_scope: Some(LocalTermScope::new(vec![1])),
            bindings: Vec::new(),
            visible_bindings: vec![BindingId::new(0)],
            recovery: BindingContextRecovery::Normal,
        });
        let matrix = BindingEnv::try_new(BindingEnvParts {
            source_id: fixture.source,
            module_id: fixture.module,
            contexts,
            bindings,
            diagnostics: handoff.binding_env().diagnostics().clone(),
        })
        .expect("Task269GUP synthetic scope matrix");
        assert_eq!(
            matrix.lookup(&BindingLookupSite::new(
                "y",
                child,
                Some(LocalTermScope::new(vec![0, 0])),
                2,
            )),
            Ok(BindingLookupResult::Local(BindingId::new(1)))
        );
        assert_eq!(
            matrix.lookup(&BindingLookupSite::new(
                "y",
                shadow_child,
                Some(LocalTermScope::new(vec![0, 1])),
                3,
            )),
            Ok(BindingLookupResult::Local(BindingId::new(2)))
        );
        assert_eq!(
            matrix.lookup(&BindingLookupSite::new(
                "y",
                BindingContextId::new(1),
                Some(LocalTermScope::new(vec![0])),
                3,
            )),
            Ok(BindingLookupResult::Local(BindingId::new(1)))
        );
        for (context, scope) in [
            (BindingContextId::new(0), LocalTermScope::new(Vec::new())),
            (sibling, LocalTermScope::new(vec![1])),
        ] {
            assert_eq!(
                matrix.lookup(&BindingLookupSite::new("y", context, Some(scope), 2)),
                Ok(BindingLookupResult::Unresolved)
            );
        }
    }

    #[test]
    fn source_proof_local_given_use_binding_has_no_type_term_or_semantic_owner() {
        let fixture = given_use_fixture();
        let handoff = SourceProofLocalGivenUseBindingProducer::build(fixture.input, &fixture.base)
            .expect("Task269GUP checker handoff");
        let binding = handoff
            .binding_env()
            .bindings()
            .get(BindingId::new(1))
            .expect("Task269GUP witness binding");
        assert_eq!(binding.type_site, BindingTypeSite::Missing);
        assert!(binding.captured.identities().is_empty());
        assert!(binding.diagnostics.is_empty());
        assert!(handoff.binding_env().diagnostics().is_empty());
        for forbidden in [
            "term-reference",
            "checked-formula",
            "condition-fact",
            "initial-obligation",
            "terminal-goal",
            "accepted",
            "discharged",
        ] {
            assert!(!handoff.debug_text().contains(forbidden));
        }
    }

    #[test]
    fn source_proof_local_given_condition_binding_builds_exact_scope_transaction() {
        let fixture = given_condition_fixture();
        let handoff = SourceProofLocalGivenConditionBindingProducer::build(
            fixture.input.clone(),
            &fixture.base,
        )
        .expect("Task269GC exact checker transaction");
        handoff
            .validate_installation(fixture.source, &fixture.module)
            .expect("Task269GC exact installation");

        assert_eq!(handoff.source_id(), fixture.source);
        assert_eq!(handoff.module_id(), &fixture.module);
        assert_eq!(handoff.lower_fingerprint(), fixture.input.lower_fingerprint);
        assert_eq!(handoff.theorem_symbol(), &fixture.input.theorem_symbol);
        assert_eq!(
            handoff.theorem_definition(),
            fixture.input.theorem_definition
        );
        assert_eq!(handoff.contribution(), fixture.input.contribution);
        assert_eq!(handoff.theorem_range(), range(fixture.source, 19, 133));
        assert_eq!(handoff.proof_range(), range(fixture.source, 68, 132));
        assert_eq!(handoff.given_range(), range(fixture.source, 76, 113));
        assert_eq!(handoff.segment_range(), range(fixture.source, 82, 93));
        assert_eq!(handoff.name_range(), range(fixture.source, 82, 83));
        assert_eq!(handoff.base_binding_env(), &fixture.base);
        assert_eq!(
            handoff.base_binding_fingerprint(),
            fixture.base.debug_text()
        );
        assert_eq!(
            (
                handoff.binding_env().contexts().len(),
                handoff.binding_env().bindings().len(),
                handoff.binding_env().diagnostics().len(),
            ),
            (2, 2, 0)
        );
        assert_eq!(
            handoff.final_binding_fingerprint(),
            handoff.binding_env().debug_text()
        );
        let row = handoff
            .bindings()
            .get(SourceProofLocalGivenBindingId::new(0))
            .expect("Task269GC dense binding row");
        assert!(exact_task269gc_output_binding(row));
        assert_eq!(
            handoff.bindings().iter().collect::<Vec<_>>(),
            [(SourceProofLocalGivenBindingId::new(0), row)]
        );
        assert!(exact_task269gc_local_binding(
            handoff.binding_env(),
            fixture.source
        ));
        assert!(exact_task269gc_lookup_behavior(handoff.binding_env()));
        let expected_debug = format!(
            concat!(
                "source-proof-local-given-condition-binding-debug-v1\n",
                "module: {}::{}\n",
                "lower-fingerprint: {:?}\n",
                "theorem symbol={:?} definition=0 contribution=0 range=19..133 proof=68..132\n",
                "given range=76..113 segment=82..93 name=82..83 source_ordinal=1\n",
                "base-binding-fingerprint: {:?}\n",
                "binding#0 binding=1 context=1 source_ordinal=1 visible_after=1 recovery=normal\n",
                "final-binding-fingerprint: {:?}\n",
            ),
            fixture.module.package().as_str(),
            fixture.module.path().as_str(),
            fixture.input.lower_fingerprint,
            fixture.input.theorem_symbol.fqn().as_str(),
            handoff.base_binding_fingerprint(),
            handoff.final_binding_fingerprint(),
        );
        assert_eq!(handoff.debug_text(), expected_debug);
        assert!(handoff.debug_text().ends_with('\n'));
        assert!(!handoff.debug_text().ends_with("\n\n"));

        let escaped_fixture = given_condition_fixture_for_module(ModuleId::new(
            PackageId::new("pkg"),
            ModulePath::new(r"task\:|/269gc"),
        ));
        let expected_namespace = r"task\\\c\p\s269gc";
        assert_eq!(
            escape_task269c_symbol_component(escaped_fixture.module.path().as_str()),
            expected_namespace,
        );
        let expected_local = format!(
            concat!(
                "contribution=0:namespace={}:owner=theorem#1:shell=theorem:",
                "kind=theorem:name=ProofLocalGivenConditionUseSmoke:notation=_:arity=_:",
                "definition=theorem:registration=_:policy=non-overloadable:",
                "slot=non-overloadable:_:theorem:_"
            ),
            expected_namespace,
        );
        assert_eq!(
            escaped_fixture.input.theorem_symbol.local().as_str(),
            expected_local,
        );
        assert_eq!(
            escaped_fixture.input.theorem_symbol.fqn().as_str(),
            format!("pkg::task\\:|/269gc::{expected_local}"),
        );
        SourceProofLocalGivenConditionBindingProducer::build(
            escaped_fixture.input,
            &escaped_fixture.base,
        )
        .expect("Task269GC escaped module identity");
    }

    #[test]
    fn source_proof_local_given_condition_binding_rejects_corruption_with_stable_precedence() {
        let fixture = given_condition_fixture();

        let mut wrong_transaction = fixture.input.clone();
        wrong_transaction.source_id = other_source_id();
        assert_eq!(
            SourceProofLocalGivenConditionBindingProducer::build(wrong_transaction, &fixture.base,),
            Err(SourceProofLocalGivenConditionBindingError::InvalidTransaction)
        );

        let mut dependencies = Vec::new();
        let mut input = fixture.input.clone();
        input.lower_fingerprint.push('!');
        dependencies.push(input);
        let mut input = fixture.input.clone();
        input.theorem_symbol = SymbolId::new(
            ModuleId::new(PackageId::new("pkg"), ModulePath::new("task269gc-other")),
            input.theorem_symbol.local().clone(),
            input.theorem_symbol.fqn().clone(),
        );
        dependencies.push(input);
        let mut input = fixture.input.clone();
        input.theorem_symbol = SymbolId::new(
            input.theorem_symbol.module().clone(),
            LocalSymbolId::new(format!("{}!", input.theorem_symbol.local().as_str())),
            input.theorem_symbol.fqn().clone(),
        );
        dependencies.push(input);
        let mut input = fixture.input.clone();
        input.theorem_symbol = SymbolId::new(
            input.theorem_symbol.module().clone(),
            input.theorem_symbol.local().clone(),
            FullyQualifiedName::new(format!("{}!", input.theorem_symbol.fqn().as_str())),
        );
        dependencies.push(input);
        let mut input = fixture.input.clone();
        let wrong_local = format!("{}!", input.theorem_symbol.local().as_str());
        let wrong_fqn = format!("{}!", input.theorem_symbol.fqn().as_str());
        input.theorem_symbol = SymbolId::new(
            input.module_id.clone(),
            LocalSymbolId::new(wrong_local),
            FullyQualifiedName::new(wrong_fqn.clone()),
        );
        input.lower_fingerprint = input
            .lower_fingerprint
            .replace(fixture.input.theorem_symbol.fqn().as_str(), &wrong_fqn);
        dependencies.push(input);
        let mut input = fixture.input.clone();
        input.theorem_definition = fixture.other_definition;
        dependencies.push(input);
        let mut input = fixture.input.clone();
        input.contribution = fixture.other_contribution;
        dependencies.push(input);
        for select in 0..5 {
            let mut input = fixture.input.clone();
            match select {
                0 => input.theorem_range.end += 1,
                1 => input.proof_range.end += 1,
                2 => input.given_range.end += 1,
                3 => input.segment_range.end += 1,
                4 => input.name_range.end += 1,
                _ => unreachable!(),
            }
            dependencies.push(input);
        }
        for input in dependencies {
            assert_eq!(
                SourceProofLocalGivenConditionBindingProducer::build(input, &fixture.base),
                Err(SourceProofLocalGivenConditionBindingError::DependencyMismatch)
            );
        }

        assert_eq!(
            SourceProofLocalGivenConditionBindingProducer::build(
                fixture.input.clone(),
                &empty_given_condition_base(&fixture),
            ),
            Err(SourceProofLocalGivenConditionBindingError::InvalidBaseBindingEnvironment)
        );
        for local in [
            LocalTermBinding::new(
                "z",
                LocalTermScope::new(vec![0]),
                range(fixture.source, 82, 83),
                1,
            ),
            LocalTermBinding::new(
                "y",
                LocalTermScope::new(vec![1]),
                range(fixture.source, 82, 83),
                1,
            ),
            LocalTermBinding::new(
                "y",
                LocalTermScope::new(vec![0]),
                range(fixture.source, 81, 83),
                1,
            ),
            LocalTermBinding::new(
                "y",
                LocalTermScope::new(vec![0]),
                range(fixture.source, 82, 83),
                2,
            ),
        ] {
            let mut input = fixture.input.clone();
            input.local = local;
            assert_eq!(
                SourceProofLocalGivenConditionBindingProducer::build(input, &fixture.base),
                Err(
                    SourceProofLocalGivenConditionBindingError::InvalidDeclaration {
                        binding: SourceProofLocalGivenBindingId::new(0),
                    }
                )
            );
        }
        let mut wrong_ordinal = fixture.input.clone();
        wrong_ordinal.source_ordinal = 2;
        assert_eq!(
            SourceProofLocalGivenConditionBindingProducer::build(wrong_ordinal, &fixture.base),
            Err(
                SourceProofLocalGivenConditionBindingError::InvalidDeclaration {
                    binding: SourceProofLocalGivenBindingId::new(0),
                }
            )
        );

        let handoff = SourceProofLocalGivenConditionBindingProducer::build(
            fixture.input.clone(),
            &fixture.base,
        )
        .expect("Task269GC exact handoff");
        let mut transaction = handoff.clone();
        transaction.source_id = other_source_id();
        transaction.set_lower_fingerprint_for_task269gc_test("corrupt");
        assert_eq!(
            transaction.validate_installation(fixture.source, &fixture.module),
            Err(SourceProofLocalGivenConditionBindingError::InvalidTransaction)
        );
        let mut dependency = handoff.clone();
        dependency.set_lower_fingerprint_for_task269gc_test("corrupt");
        dependency.set_base_binding_fingerprint_for_task269gc_test("corrupt");
        assert_eq!(
            dependency.validate_installation(fixture.source, &fixture.module),
            Err(SourceProofLocalGivenConditionBindingError::DependencyMismatch)
        );
        let mut base = handoff.clone();
        base.set_base_binding_fingerprint_for_task269gc_test("corrupt");
        base.truncate_task269gc_bindings_for_test();
        assert_eq!(
            base.validate_installation(fixture.source, &fixture.module),
            Err(SourceProofLocalGivenConditionBindingError::InvalidBaseBindingEnvironment)
        );
        let mut aggregate = handoff.clone();
        aggregate.truncate_task269gc_bindings_for_test();
        aggregate.set_final_binding_fingerprint_for_task269gc_test("corrupt");
        assert_eq!(
            aggregate.validate_installation(fixture.source, &fixture.module),
            Err(SourceProofLocalGivenConditionBindingError::InvalidAggregate)
        );
        let mut declaration = handoff.clone();
        declaration.corrupt_task269gc_binding_row_for_test();
        declaration.set_final_binding_fingerprint_for_task269gc_test("corrupt");
        assert_eq!(
            declaration.validate_installation(fixture.source, &fixture.module),
            Err(
                SourceProofLocalGivenConditionBindingError::InvalidDeclaration {
                    binding: SourceProofLocalGivenBindingId::new(0),
                }
            )
        );
        let mut environment = handoff.clone();
        environment.set_final_binding_fingerprint_for_task269gc_test("corrupt");
        assert_eq!(
            environment.validate_installation(fixture.source, &fixture.module),
            Err(SourceProofLocalGivenConditionBindingError::InvalidBindingEnvironment)
        );
        assert_eq!(
            environment.validate_complete_installation(fixture.source, &fixture.module, false),
            Err(SourceProofLocalGivenConditionBindingError::InvalidBindingEnvironment)
        );
        assert_eq!(
            handoff.validate_complete_installation(fixture.source, &fixture.module, false),
            Err(SourceProofLocalGivenConditionBindingError::InvalidInstallation)
        );

        for (error, expected) in [
            (
                SourceProofLocalGivenConditionBindingError::InvalidTransaction,
                "source proof-local given-condition binding transaction is invalid".to_owned(),
            ),
            (
                SourceProofLocalGivenConditionBindingError::DependencyMismatch,
                "source proof-local given-condition binding dependency mismatch".to_owned(),
            ),
            (
                SourceProofLocalGivenConditionBindingError::InvalidBaseBindingEnvironment,
                "source proof-local given-condition binding base binding environment is invalid"
                    .to_owned(),
            ),
            (
                SourceProofLocalGivenConditionBindingError::InvalidAggregate,
                "source proof-local given-condition binding aggregate is invalid".to_owned(),
            ),
            (
                SourceProofLocalGivenConditionBindingError::InvalidDeclaration {
                    binding: SourceProofLocalGivenBindingId::new(0),
                },
                "source proof-local given-condition binding 0 is invalid".to_owned(),
            ),
            (
                SourceProofLocalGivenConditionBindingError::InvalidBindingEnvironment,
                "source proof-local given-condition binding binding environment is invalid"
                    .to_owned(),
            ),
            (
                SourceProofLocalGivenConditionBindingError::InvalidInstallation,
                "source proof-local given-condition binding installation is invalid".to_owned(),
            ),
        ] {
            assert_eq!(error.to_string(), expected);
        }
    }

    #[test]
    fn source_proof_local_given_condition_binding_typed_and_resolved_ownership_is_atomic() {
        let fixture = given_condition_fixture();
        let handoff = SourceProofLocalGivenConditionBindingProducer::build(
            fixture.input.clone(),
            &fixture.base,
        )
        .expect("Task269GC checker handoff");
        let empty = empty_typed(fixture.source, fixture.module.clone());
        let typed = empty
            .clone()
            .with_source_proof_local_given_condition_binding(handoff.clone())
            .expect("Task269GC Typed owner");
        assert_eq!(
            typed.source_proof_local_given_condition_binding(),
            Some(&handoff)
        );
        assert_eq!(
            typed
                .clone()
                .with_source_proof_local_given_condition_binding(handoff.clone()),
            Err(TypedAstError::InvalidSourceProofLocalGivenConditionBinding)
        );
        let resolved = assemble_empty(&typed).expect("Task269GC Resolved owner");
        assert_eq!(
            resolved.source_proof_local_given_condition_binding(),
            Some(&handoff)
        );
        assert!(resolved.debug_text().contains(&handoff.debug_text()));

        let old_fixture = given_fixture_for_module(fixture.module.clone());
        assert_eq!(old_fixture.source, fixture.source);
        let old_handoff =
            SourceProofLocalGivenBindingProducer::build(old_fixture.input, &old_fixture.base)
                .expect("Task269G cross-family handoff");
        let old_first = empty_typed(fixture.source, fixture.module.clone())
            .with_source_proof_local_given_binding(old_handoff.clone())
            .expect("Task269G first owner");
        assert_eq!(
            old_first.with_source_proof_local_given_condition_binding(handoff.clone()),
            Err(TypedAstError::InvalidSourceProofLocalGivenConditionBinding)
        );
        let gc_first = empty_typed(fixture.source, fixture.module.clone())
            .with_source_proof_local_given_condition_binding(handoff.clone())
            .expect("Task269GC first owner");
        assert_eq!(
            gc_first.with_source_proof_local_given_binding(old_handoff),
            Err(TypedAstError::InvalidSourceProofLocalGivenBinding)
        );
        assert_eq!(
            occupied_typed(fixture.source, fixture.module.clone())
                .with_source_proof_local_given_condition_binding(handoff.clone()),
            Err(TypedAstError::InvalidSourceProofLocalGivenConditionBinding)
        );
        for table in [
            StatementTransportTableForTest::Context,
            StatementTransportTableForTest::Type,
            StatementTransportTableForTest::Fact,
            StatementTransportTableForTest::Coercion,
            StatementTransportTableForTest::InitialObligation,
            StatementTransportTableForTest::Diagnostic,
        ] {
            let mut occupied = empty.clone();
            occupied.occupy_statement_transport_table_for_test(table);
            assert_eq!(
                occupied.with_source_proof_local_given_condition_binding(handoff.clone()),
                Err(TypedAstError::InvalidSourceProofLocalGivenConditionBinding),
                "Task269GC unexpectedly accepted occupied {table:?} table",
            );
        }
        assert_eq!(
            assemble_with_node_hints(
                &typed,
                vec![ResolvedNodeKindHint {
                    typed_node: TypedNodeId::new(0),
                    kind: ResolvedNodeKindHintKind::SourcePreserved {
                        role: SourceNodeRole::new("Task269GC.forbidden"),
                    },
                }],
            ),
            Err(ResolvedTypedAstError::InvalidSourceProofLocalGivenConditionBinding)
        );
        assert_eq!(
            TypedAstError::InvalidSourceProofLocalGivenConditionBinding.to_string(),
            "typed AST source proof-local given-condition binding handoff is inconsistent"
        );
        assert_eq!(
            ResolvedTypedAstError::InvalidSourceProofLocalGivenConditionBinding.to_string(),
            "resolved typed AST source proof-local given-condition binding handoff is inconsistent"
        );

        let mut corrupted = handoff;
        corrupted.set_final_binding_fingerprint_for_task269gc_test("corrupt");
        let mut injected = empty;
        injected.inject_source_proof_local_given_condition_binding_for_test(corrupted);
        assert_eq!(
            assemble_empty(&injected),
            Err(ResolvedTypedAstError::InvalidSourceProofLocalGivenConditionBinding)
        );
    }

    #[test]
    fn source_proof_local_given_condition_binding_scope_matrix_is_lexical_and_semantically_empty() {
        let fixture = given_condition_fixture();
        let handoff =
            SourceProofLocalGivenConditionBindingProducer::build(fixture.input, &fixture.base)
                .expect("Task269GC checker handoff");
        let mut bindings = handoff.binding_env().bindings().clone();
        let shadow = bindings.insert(BindingDraft {
            spelling: "y".to_owned(),
            kind: BindingKind::GivenWitness,
            identity: BinderIdentity::ResolverLocal {
                scope: LocalTermScope::new(vec![0, 1]),
                ordinal: 2,
                declaration_range: range(fixture.source, 114, 115),
            },
            owner_context: BindingContextId::new(3),
            declaration_range: range(fixture.source, 114, 115),
            visible_after_ordinal: 2,
            type_site: BindingTypeSite::Missing,
            status: BindingStatus::Active,
            captured: CapturedFreeVariables::default(),
            diagnostics: Vec::new(),
            recovery: BindingRecoveryState::Normal,
        });
        assert_eq!(shadow, BindingId::new(2));
        let mut contexts = handoff.binding_env().contexts().clone();
        let child = contexts.insert(BindingContextDraft {
            owner: BindingContextOwner::Generated("task269gc-unshadowed-child".to_owned()),
            parent: Some(BindingContextId::new(1)),
            layer: BindingContextLayer::Block,
            lexical_scope: Some(LocalTermScope::new(vec![0, 0])),
            bindings: Vec::new(),
            visible_bindings: vec![BindingId::new(0), BindingId::new(1)],
            recovery: BindingContextRecovery::Normal,
        });
        let shadow_child = contexts.insert(BindingContextDraft {
            owner: BindingContextOwner::Generated("task269gc-shadow-child".to_owned()),
            parent: Some(BindingContextId::new(1)),
            layer: BindingContextLayer::Block,
            lexical_scope: Some(LocalTermScope::new(vec![0, 1])),
            bindings: vec![shadow],
            visible_bindings: vec![BindingId::new(0), BindingId::new(1), shadow],
            recovery: BindingContextRecovery::Normal,
        });
        let sibling = contexts.insert(BindingContextDraft {
            owner: BindingContextOwner::Generated("task269gc-sibling".to_owned()),
            parent: Some(BindingContextId::new(0)),
            layer: BindingContextLayer::Block,
            lexical_scope: Some(LocalTermScope::new(vec![1])),
            bindings: Vec::new(),
            visible_bindings: vec![BindingId::new(0)],
            recovery: BindingContextRecovery::Normal,
        });
        assert_eq!(
            (child, shadow_child, sibling),
            (
                BindingContextId::new(2),
                BindingContextId::new(3),
                BindingContextId::new(4),
            )
        );
        let matrix = BindingEnv::try_new(BindingEnvParts {
            source_id: fixture.source,
            module_id: fixture.module.clone(),
            contexts,
            bindings,
            diagnostics: handoff.binding_env().diagnostics().clone(),
        })
        .expect("Task269GC synthetic scope matrix");
        assert!(matches!(
            matrix.lookup(&BindingLookupSite::new(
                "y",
                BindingContextId::new(1),
                Some(LocalTermScope::new(vec![0])),
                1,
            )),
            Ok(BindingLookupResult::ForwardReference { candidates, .. })
                if candidates == [BindingId::new(1)]
        ));
        for _intent in ["own-such-that", "subsequent-statement"] {
            assert_eq!(
                matrix.lookup(&BindingLookupSite::new(
                    "y",
                    BindingContextId::new(1),
                    Some(LocalTermScope::new(vec![0])),
                    2,
                )),
                Ok(BindingLookupResult::Local(BindingId::new(1)))
            );
        }
        assert_eq!(
            matrix.lookup(&BindingLookupSite::new(
                "y",
                child,
                Some(LocalTermScope::new(vec![0, 0])),
                2,
            )),
            Ok(BindingLookupResult::Local(BindingId::new(1)))
        );
        assert_eq!(
            matrix.lookup(&BindingLookupSite::new(
                "y",
                shadow_child,
                Some(LocalTermScope::new(vec![0, 1])),
                3,
            )),
            Ok(BindingLookupResult::Local(BindingId::new(2)))
        );
        assert_eq!(
            matrix.lookup(&BindingLookupSite::new(
                "y",
                BindingContextId::new(1),
                Some(LocalTermScope::new(vec![0])),
                3,
            )),
            Ok(BindingLookupResult::Local(BindingId::new(1)))
        );
        for (context, scope) in [
            (BindingContextId::new(0), LocalTermScope::new(Vec::new())),
            (sibling, LocalTermScope::new(vec![1])),
        ] {
            assert_eq!(
                matrix.lookup(&BindingLookupSite::new("y", context, Some(scope), 2)),
                Ok(BindingLookupResult::Unresolved)
            );
        }

        let typed = empty_typed(fixture.source, fixture.module)
            .with_source_proof_local_given_condition_binding(handoff)
            .expect("Task269GC Typed owner");
        assert!(typed.nodes().is_empty());
        assert!(typed.contexts().is_empty());
        assert!(typed.types().is_empty());
        assert!(typed.facts().is_empty());
        assert!(typed.coercions().is_empty());
        assert!(typed.initial_obligations().is_empty());
        assert!(typed.diagnostics().is_empty());
        let resolved = assemble_empty(&typed).expect("Task269GC Resolved owner");
        assert!(resolved.nodes().is_empty());
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
        assert!(resolved.checked_formulas().is_empty());
        assert!(resolved.statement_semantics().is_empty());
        assert!(resolved.checked_proofs().is_empty());
        assert!(resolved.checked_proof_nodes().is_empty());
        assert!(resolved.checked_terminal_goals().is_empty());
        assert!(resolved.diagnostics().is_empty());
        for forbidden in [
            "107..108",
            "111..112",
            "term-reference",
            "checked-formula",
            "condition-fact",
            "initial-obligation#",
            "terminal-goal#",
            "accepted",
            "discharged",
            "verification-condition",
        ] {
            assert!(
                !resolved.debug_text().contains(forbidden),
                "excluded {forbidden}"
            );
        }
    }

    struct GivenConditionFixture {
        source: SourceId,
        module: ModuleId,
        input: SourceProofLocalGivenConditionBindingHandoffInput,
        base: BindingEnv,
        other_definition: DefinitionId,
        other_contribution: SourceContributionId,
    }

    fn given_condition_fixture() -> GivenConditionFixture {
        given_condition_fixture_for_module(ModuleId::new(
            PackageId::new("pkg"),
            ModulePath::new("task269gc"),
        ))
    }

    fn given_condition_fixture_for_module(module: ModuleId) -> GivenConditionFixture {
        let source = source_id();
        let local = format!(
            concat!(
                "contribution=0:namespace={}:owner=theorem#1:shell=theorem:",
                "kind=theorem:name=ProofLocalGivenConditionUseSmoke:notation=_:arity=_:",
                "definition=theorem:registration=_:policy=non-overloadable:",
                "slot=non-overloadable:_:theorem:_"
            ),
            escape_task269c_symbol_component(module.path().as_str()),
        );
        let theorem_symbol = SymbolId::new(
            module.clone(),
            LocalSymbolId::new(local.clone()),
            FullyQualifiedName::new(format!(
                "{}::{}::{local}",
                module.package().as_str(),
                module.path().as_str(),
            )),
        );
        let mut contributions = SourceContributionIndex::new();
        let contribution = contributions.insert(
            module.clone(),
            ContributionKind::LocalSource { source_id: source },
            SourceAnchor::Range(range(source, 0, 18)),
        );
        let other_contribution = contributions.insert(
            module.clone(),
            ContributionKind::LocalSource { source_id: source },
            SourceAnchor::Range(range(source, 0, 18)),
        );
        let mut definitions = DefinitionIndex::new();
        let theorem_definition = definitions.insert(DefinitionShell::new(
            theorem_symbol.clone(),
            DefinitionKind::Theorem,
            SemanticOrigin::new(
                source,
                module.clone(),
                SourceAnchor::Range(range(source, 19, 133)),
                vec![2, 1],
            ),
            contribution,
        ));
        let other_definition = definitions.insert(DefinitionShell::new(
            theorem_symbol.clone(),
            DefinitionKind::Theorem,
            SemanticOrigin::new(
                source,
                module.clone(),
                SourceAnchor::Range(range(source, 19, 133)),
                vec![2, 1],
            ),
            contribution,
        ));
        let mut input = SourceProofLocalGivenConditionBindingHandoffInput {
            source_id: source,
            module_id: module.clone(),
            lower_fingerprint: String::new(),
            theorem_symbol,
            theorem_definition,
            contribution,
            theorem_range: range(source, 19, 133),
            proof_range: range(source, 68, 132),
            given_range: range(source, 76, 113),
            segment_range: range(source, 82, 93),
            name_range: range(source, 82, 83),
            source_ordinal: 1,
            local: LocalTermBinding::new(
                "y",
                LocalTermScope::new(vec![0]),
                range(source, 82, 83),
                1,
            ),
            recovery: SourceProofLocalGivenBindingRecovery::Normal,
        };
        input.lower_fingerprint = exact_task269gc_lower_fingerprint(
            &input.module_id,
            input.theorem_symbol.fqn().as_str(),
        );
        GivenConditionFixture {
            source,
            module: module.clone(),
            input,
            base: exact_base(source, module),
            other_definition,
            other_contribution,
        }
    }

    fn empty_given_condition_base(fixture: &GivenConditionFixture) -> BindingEnv {
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
        BindingEnv::try_new(BindingEnvParts {
            source_id: fixture.source,
            module_id: fixture.module.clone(),
            contexts,
            bindings: BindingTable::new(),
            diagnostics: BindingDiagnosticTable::new(),
        })
        .expect("Task269GC valid but inexact empty base")
    }

    struct GivenUseFixture {
        source: SourceId,
        module: ModuleId,
        input: SourceProofLocalGivenUseBindingHandoffInput,
        base: BindingEnv,
        other_definition: DefinitionId,
        other_contribution: SourceContributionId,
    }

    fn given_use_fixture() -> GivenUseFixture {
        let source = source_id();
        let module = ModuleId::new(PackageId::new("pkg"), ModulePath::new("task269gup"));
        let local = concat!(
            "contribution=0:namespace=task269gup:owner=theorem#1:shell=theorem:",
            "kind=theorem:name=FormulaStatementGivenSmoke:notation=_:arity=_:",
            "definition=theorem:registration=_:policy=non-overloadable:",
            "slot=non-overloadable:_:theorem:_"
        );
        let theorem_symbol = SymbolId::new(
            module.clone(),
            LocalSymbolId::new(local),
            FullyQualifiedName::new(format!("pkg::task269gup::{local}")),
        );
        let mut contributions = SourceContributionIndex::new();
        let contribution = contributions.insert(
            module.clone(),
            ContributionKind::LocalSource { source_id: source },
            SourceAnchor::Range(range(source, 0, 18)),
        );
        let other_contribution = contributions.insert(
            module.clone(),
            ContributionKind::LocalSource { source_id: source },
            SourceAnchor::Range(range(source, 0, 18)),
        );
        let mut definitions = DefinitionIndex::new();
        let theorem_definition = definitions.insert(DefinitionShell::new(
            theorem_symbol.clone(),
            DefinitionKind::Theorem,
            SemanticOrigin::new(
                source,
                module.clone(),
                SourceAnchor::Range(range(source, 19, 127)),
                vec![2, 1],
            ),
            contribution,
        ));
        let other_definition = definitions.insert(DefinitionShell::new(
            theorem_symbol.clone(),
            DefinitionKind::Theorem,
            SemanticOrigin::new(
                source,
                module.clone(),
                SourceAnchor::Range(range(source, 19, 127)),
                vec![2, 1],
            ),
            contribution,
        ));
        let mut input = SourceProofLocalGivenUseBindingHandoffInput {
            source_id: source,
            module_id: module.clone(),
            lower_fingerprint: String::new(),
            theorem_symbol,
            theorem_definition,
            contribution,
            theorem_range: range(source, 19, 127),
            proof_range: range(source, 62, 126),
            given_range: range(source, 70, 108),
            segment_range: range(source, 76, 87),
            name_range: range(source, 76, 77),
            source_ordinal: 1,
            local: LocalTermBinding::new(
                "y",
                LocalTermScope::new(vec![0]),
                range(source, 76, 77),
                1,
            ),
            recovery: SourceProofLocalGivenBindingRecovery::Normal,
        };
        input.lower_fingerprint = exact_task269gup_lower_fingerprint(Task269gupDependency {
            source_id: input.source_id,
            module_id: &input.module_id,
            lower_fingerprint: "",
            theorem_symbol: &input.theorem_symbol,
            theorem_definition: input.theorem_definition,
            contribution: input.contribution,
            theorem_range: input.theorem_range,
            proof_range: input.proof_range,
            given_range: input.given_range,
            segment_range: input.segment_range,
            name_range: input.name_range,
        });
        GivenUseFixture {
            source,
            module: module.clone(),
            input,
            base: exact_base(source, module),
            other_definition,
            other_contribution,
        }
    }

    fn empty_given_use_base(fixture: &GivenUseFixture) -> BindingEnv {
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
        BindingEnv::try_new(BindingEnvParts {
            source_id: fixture.source,
            module_id: fixture.module.clone(),
            contexts,
            bindings: BindingTable::new(),
            diagnostics: BindingDiagnosticTable::new(),
        })
        .expect("Task269GUP valid but inexact empty base")
    }

    struct GivenFixture {
        source: SourceId,
        module: ModuleId,
        input: SourceProofLocalGivenBindingHandoffInput,
        base: BindingEnv,
        other_definition: DefinitionId,
        other_contribution: SourceContributionId,
    }

    fn given_fixture() -> GivenFixture {
        given_fixture_for_module(ModuleId::new(
            PackageId::new("pkg"),
            ModulePath::new("task269g"),
        ))
    }

    fn given_fixture_for_module(module: ModuleId) -> GivenFixture {
        let source = source_id();
        let local = format!(
            concat!(
                "contribution=0:namespace={}:owner=theorem#1:shell=theorem:",
                "kind=theorem:name=FormulaStatementGivenSmoke:notation=_:arity=_:",
                "definition=theorem:registration=_:policy=non-overloadable:",
                "slot=non-overloadable:_:theorem:_"
            ),
            escape_task269c_symbol_component(module.path().as_str()),
        );
        let theorem_symbol = SymbolId::new(
            module.clone(),
            LocalSymbolId::new(local.clone()),
            FullyQualifiedName::new(format!(
                "{}::{}::{local}",
                module.package().as_str(),
                module.path().as_str(),
            )),
        );
        let mut contributions = SourceContributionIndex::new();
        let contribution = contributions.insert(
            module.clone(),
            ContributionKind::LocalSource { source_id: source },
            SourceAnchor::Range(range(source, 0, 18)),
        );
        let other_contribution = contributions.insert(
            module.clone(),
            ContributionKind::LocalSource { source_id: source },
            SourceAnchor::Range(range(source, 0, 18)),
        );
        let mut definitions = DefinitionIndex::new();
        let theorem_definition = definitions.insert(DefinitionShell::new(
            theorem_symbol.clone(),
            DefinitionKind::Theorem,
            SemanticOrigin::new(
                source,
                module.clone(),
                SourceAnchor::Range(range(source, 19, 128)),
                vec![2, 1],
            ),
            contribution,
        ));
        let other_definition = definitions.insert(DefinitionShell::new(
            theorem_symbol.clone(),
            DefinitionKind::Theorem,
            SemanticOrigin::new(
                source,
                module.clone(),
                SourceAnchor::Range(range(source, 19, 128)),
                vec![2, 1],
            ),
            contribution,
        ));
        let mut input = SourceProofLocalGivenBindingHandoffInput {
            source_id: source,
            module_id: module.clone(),
            lower_fingerprint: String::new(),
            theorem_symbol,
            theorem_definition,
            contribution,
            theorem_range: range(source, 19, 128),
            proof_range: range(source, 62, 127),
            given_range: range(source, 70, 108),
            segment_range: range(source, 76, 87),
            name_range: range(source, 76, 77),
            source_ordinal: 1,
            local: LocalTermBinding::new(
                "y",
                LocalTermScope::new(vec![0]),
                range(source, 76, 77),
                1,
            ),
            recovery: SourceProofLocalGivenBindingRecovery::Normal,
        };
        input.lower_fingerprint = exact_task269gp_lower_fingerprint(Task269gDependency {
            source_id: input.source_id,
            module_id: &input.module_id,
            lower_fingerprint: "",
            theorem_symbol: &input.theorem_symbol,
            theorem_definition: input.theorem_definition,
            contribution: input.contribution,
            theorem_range: input.theorem_range,
            proof_range: input.proof_range,
            given_range: input.given_range,
            segment_range: input.segment_range,
            name_range: input.name_range,
        });
        GivenFixture {
            source,
            module: module.clone(),
            input,
            base: exact_base(source, module),
            other_definition,
            other_contribution,
        }
    }

    fn empty_given_base(fixture: &GivenFixture) -> BindingEnv {
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
        BindingEnv::try_new(BindingEnvParts {
            source_id: fixture.source,
            module_id: fixture.module.clone(),
            contexts,
            bindings: BindingTable::new(),
            diagnostics: BindingDiagnosticTable::new(),
        })
        .expect("Task269G valid but inexact empty base")
    }

    struct Fixture {
        source: SourceId,
        module: ModuleId,
        input: SourceProofLocalLetBindingHandoffInput,
        base: BindingEnv,
        other_definition: DefinitionId,
        other_contribution: SourceContributionId,
    }

    fn fixture() -> Fixture {
        let source = source_id();
        let module = ModuleId::new(PackageId::new("pkg"), ModulePath::new("task269c"));
        let local = concat!(
            "contribution=0:namespace=task269c:owner=theorem#1:shell=theorem:",
            "kind=theorem:name=FormulaStatementLetSmoke:notation=_:arity=_:",
            "definition=theorem:registration=_:policy=non-overloadable:",
            "slot=non-overloadable:_:theorem:_"
        );
        let theorem_symbol = SymbolId::new(
            module.clone(),
            LocalSymbolId::new(local),
            FullyQualifiedName::new(format!("pkg::task269c::{local}")),
        );
        let mut contributions = SourceContributionIndex::new();
        let contribution = contributions.insert(
            module.clone(),
            ContributionKind::LocalSource { source_id: source },
            SourceAnchor::Range(range(source, 0, 18)),
        );
        let other_contribution = contributions.insert(
            module.clone(),
            ContributionKind::LocalSource { source_id: source },
            SourceAnchor::Range(range(source, 0, 18)),
        );
        let mut definitions = DefinitionIndex::new();
        let theorem_definition = definitions.insert(DefinitionShell::new(
            theorem_symbol.clone(),
            DefinitionKind::Theorem,
            SemanticOrigin::new(
                source,
                module.clone(),
                SourceAnchor::Range(range(source, 19, 99)),
                vec![2, 1],
            ),
            contribution,
        ));
        let other_definition = definitions.insert(DefinitionShell::new(
            theorem_symbol.clone(),
            DefinitionKind::Theorem,
            SemanticOrigin::new(
                source,
                module.clone(),
                SourceAnchor::Range(range(source, 19, 99)),
                vec![2, 1],
            ),
            contribution,
        ));
        let mut input = SourceProofLocalLetBindingHandoffInput {
            source_id: source,
            module_id: module.clone(),
            lower_fingerprint: String::new(),
            theorem_symbol,
            theorem_definition,
            contribution,
            theorem_range: range(source, 19, 99),
            proof_range: range(source, 59, 98),
            let_range: range(source, 67, 80),
            segment_range: range(source, 71, 79),
            name_range: range(source, 71, 72),
            source_ordinal: 1,
            local: LocalTermBinding::new(
                "y",
                LocalTermScope::new(vec![0]),
                range(source, 71, 72),
                1,
            ),
            recovery: SourceProofLocalLetBindingRecovery::Normal,
        };
        input.lower_fingerprint = exact_task269cp_lower_fingerprint(Task269cDependency {
            source_id: input.source_id,
            module_id: &input.module_id,
            lower_fingerprint: "",
            theorem_symbol: &input.theorem_symbol,
            theorem_definition: input.theorem_definition,
            contribution: input.contribution,
            theorem_range: input.theorem_range,
            proof_range: input.proof_range,
            let_range: input.let_range,
            segment_range: input.segment_range,
            name_range: input.name_range,
        });
        Fixture {
            source,
            module,
            base: exact_base(source, input.module_id.clone()),
            input,
            other_definition,
            other_contribution,
        }
    }

    fn exact_base(source: SourceId, module: ModuleId) -> BindingEnv {
        let mut bindings = BindingTable::new();
        let reserved = bindings.insert(BindingDraft {
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
        let mut contexts = BindingContextTable::new();
        contexts.insert(BindingContextDraft {
            owner: BindingContextOwner::Module,
            parent: None,
            layer: BindingContextLayer::Module,
            lexical_scope: None,
            bindings: vec![reserved],
            visible_bindings: vec![reserved],
            recovery: BindingContextRecovery::Normal,
        });
        BindingEnv::try_new(BindingEnvParts {
            source_id: source,
            module_id: module,
            contexts,
            bindings,
            diagnostics: BindingDiagnosticTable::new(),
        })
        .expect("Task269C exact base")
    }

    fn empty_base(fixture: &Fixture) -> BindingEnv {
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
        BindingEnv::try_new(BindingEnvParts {
            source_id: fixture.source,
            module_id: fixture.module.clone(),
            contexts,
            bindings: BindingTable::new(),
            diagnostics: BindingDiagnosticTable::new(),
        })
        .expect("Task269C valid but inexact empty base")
    }

    fn inexact_bases(fixture: &Fixture) -> Vec<BindingEnv> {
        let mut bases = Vec::new();
        for mutate in 0..9 {
            let mut base = fixture.base.clone();
            let binding = base
                .binding_mut_for_test(BindingId::new(0))
                .expect("Task269C base binding");
            match mutate {
                0 => binding.spelling = "z".to_owned(),
                1 => {
                    binding.identity = BinderIdentity::ReservedVariable {
                        spelling: "z".to_owned(),
                        declaration_range: range(fixture.source, 8, 9),
                    }
                }
                2 => binding.declaration_range = range(fixture.source, 7, 9),
                3 => binding.visible_after_ordinal = 1,
                4 => binding.type_site = BindingTypeSite::Missing,
                5 => binding.status = BindingStatus::Active,
                6 => {
                    binding.captured = CapturedFreeVariables::new(vec![BinderIdentity::Generated {
                        context: BindingContextId::new(0),
                        counter: 269,
                    }])
                }
                7 => binding.diagnostics = vec![BindingDiagnosticId::new(0)],
                8 => binding.recovery = BindingRecoveryState::Recovered,
                _ => unreachable!(),
            }
            bases.push(base);
        }
        for (owned, visible, scope, recovery) in [
            (Vec::new(), Vec::new(), None, BindingContextRecovery::Normal),
            (
                vec![BindingId::new(0)],
                Vec::new(),
                None,
                BindingContextRecovery::Normal,
            ),
            (
                vec![BindingId::new(0)],
                vec![BindingId::new(0)],
                Some(LocalTermScope::new(vec![0])),
                BindingContextRecovery::Normal,
            ),
            (
                vec![BindingId::new(0)],
                vec![BindingId::new(0)],
                None,
                BindingContextRecovery::Recovered,
            ),
        ] {
            let mut contexts = BindingContextTable::new();
            contexts.insert(BindingContextDraft {
                owner: BindingContextOwner::Module,
                parent: None,
                layer: BindingContextLayer::Module,
                lexical_scope: scope,
                bindings: owned,
                visible_bindings: visible,
                recovery,
            });
            bases.push(
                BindingEnv::try_new(BindingEnvParts {
                    source_id: fixture.source,
                    module_id: fixture.module.clone(),
                    contexts,
                    bindings: fixture.base.bindings().clone(),
                    diagnostics: fixture.base.diagnostics().clone(),
                })
                .expect("Task269C inexact context base remains structurally valid"),
            );
        }
        bases
    }

    fn inexact_final_declaration_handoffs(
        handoff: &SourceProofLocalLetBindingHandoff,
        source: SourceId,
    ) -> Vec<SourceProofLocalLetBindingHandoff> {
        let mut handoffs = Vec::new();
        for mutate in 0..6 {
            let mut corrupted = handoff.clone();
            let binding = corrupted
                .binding_env
                .binding_mut_for_test(BindingId::new(1))
                .expect("Task269C final binding");
            match mutate {
                0 => binding.spelling = "z".to_owned(),
                1 => {
                    binding.identity = BinderIdentity::ResolverLocal {
                        scope: LocalTermScope::new(vec![1]),
                        ordinal: 1,
                        declaration_range: range(source, 71, 72),
                    }
                }
                2 => binding.declaration_range = range(source, 70, 72),
                3 => binding.visible_after_ordinal = 2,
                4 => binding.owner_context = BindingContextId::new(0),
                5 => binding.recovery = BindingRecoveryState::Recovered,
                _ => unreachable!(),
            }
            corrupted.final_binding_fingerprint = corrupted.binding_env.debug_text();
            handoffs.push(corrupted);
        }
        handoffs
    }

    fn inexact_final_environment_handoffs(
        handoff: &SourceProofLocalLetBindingHandoff,
        source: SourceId,
    ) -> Vec<SourceProofLocalLetBindingHandoff> {
        let mut handoffs = Vec::new();
        for mutate in 0..5 {
            let mut corrupted = handoff.clone();
            let binding = corrupted
                .binding_env
                .binding_mut_for_test(BindingId::new(1))
                .expect("Task269C final binding");
            match mutate {
                0 => binding.kind = BindingKind::LocalAbbreviation,
                1 => binding.type_site = BindingTypeSite::Source(range(source, 76, 79)),
                2 => binding.status = BindingStatus::Reserved,
                3 => {
                    binding.captured = CapturedFreeVariables::new(vec![BinderIdentity::Generated {
                        context: BindingContextId::new(1),
                        counter: 269,
                    }])
                }
                4 => binding.diagnostics = vec![BindingDiagnosticId::new(0)],
                _ => unreachable!(),
            }
            corrupted.final_binding_fingerprint = corrupted.binding_env.debug_text();
            handoffs.push(corrupted);
        }
        for (owner, layer, scope, visible, recovery) in [
            (
                BindingContextOwner::SourceStatement {
                    source_range: range(source, 58, 98),
                },
                BindingContextLayer::Proof,
                Some(LocalTermScope::new(vec![0])),
                vec![BindingId::new(0), BindingId::new(1)],
                BindingContextRecovery::Normal,
            ),
            (
                BindingContextOwner::SourceStatement {
                    source_range: range(source, 59, 98),
                },
                BindingContextLayer::Block,
                Some(LocalTermScope::new(vec![0])),
                vec![BindingId::new(0), BindingId::new(1)],
                BindingContextRecovery::Normal,
            ),
            (
                BindingContextOwner::SourceStatement {
                    source_range: range(source, 59, 98),
                },
                BindingContextLayer::Proof,
                Some(LocalTermScope::new(vec![1])),
                vec![BindingId::new(0), BindingId::new(1)],
                BindingContextRecovery::Normal,
            ),
            (
                BindingContextOwner::SourceStatement {
                    source_range: range(source, 59, 98),
                },
                BindingContextLayer::Proof,
                Some(LocalTermScope::new(vec![0])),
                vec![BindingId::new(0)],
                BindingContextRecovery::Normal,
            ),
            (
                BindingContextOwner::SourceStatement {
                    source_range: range(source, 59, 98),
                },
                BindingContextLayer::Proof,
                Some(LocalTermScope::new(vec![0])),
                vec![BindingId::new(0), BindingId::new(1)],
                BindingContextRecovery::Recovered,
            ),
        ] {
            let mut contexts = BindingContextTable::new();
            contexts.insert(BindingContextDraft {
                owner: BindingContextOwner::Module,
                parent: None,
                layer: BindingContextLayer::Module,
                lexical_scope: None,
                bindings: vec![BindingId::new(0)],
                visible_bindings: vec![BindingId::new(0)],
                recovery: BindingContextRecovery::Normal,
            });
            contexts.insert(BindingContextDraft {
                owner,
                parent: Some(BindingContextId::new(0)),
                layer,
                lexical_scope: scope,
                bindings: vec![BindingId::new(1)],
                visible_bindings: visible,
                recovery,
            });
            let mut corrupted = handoff.clone();
            corrupted.binding_env = BindingEnv::try_new(BindingEnvParts {
                source_id: source,
                module_id: handoff.module_id.clone(),
                contexts,
                bindings: handoff.binding_env.bindings().clone(),
                diagnostics: handoff.binding_env.diagnostics().clone(),
            })
            .expect("Task269C inexact final context remains structurally valid");
            corrupted.final_binding_fingerprint = corrupted.binding_env.debug_text();
            handoffs.push(corrupted);
        }
        handoffs
    }

    fn empty_typed(source: SourceId, module: ModuleId) -> TypedAst {
        TypedAst::try_new(TypedAstParts {
            source_id: source,
            module_id: module,
            resolved_root: None,
            source_context: None,
            source_type: None,
            source_attribute: None,
            nodes: TypedArena::try_new(None, Vec::new()).expect("empty arena"),
            contexts: LocalTypeContextTable::new(),
            types: TypeTable::new(),
            facts: TypeFactTable::new(),
            coercions: CoercionTable::new(),
            initial_obligations: InitialObligationTable::new(),
            diagnostics: TypeDiagnosticTable::new(),
        })
        .expect("Task269C empty typed AST")
    }

    fn occupied_typed(source: SourceId, module: ModuleId) -> TypedAst {
        let mut builder = TypedArenaBuilder::new();
        let root = builder
            .push(TypedNode::new(
                "Task269CExcludedNode",
                SourceAnchor::Range(range(source, 0, 1)),
            ))
            .expect("Task269C occupied node");
        TypedAst::try_new(TypedAstParts {
            source_id: source,
            module_id: module,
            resolved_root: None,
            source_context: None,
            source_type: None,
            source_attribute: None,
            nodes: builder.finish(Some(root)).expect("Task269C occupied arena"),
            contexts: LocalTypeContextTable::new(),
            types: TypeTable::new(),
            facts: TypeFactTable::new(),
            coercions: CoercionTable::new(),
            initial_obligations: InitialObligationTable::new(),
            diagnostics: TypeDiagnosticTable::new(),
        })
        .expect("Task269C occupied typed AST")
    }

    fn task269ab_dummy_handoff(fixture: &Fixture) -> SourceProofLocalDeclarationHandoff {
        SourceProofLocalDeclarationHandoff {
            source_id: fixture.source,
            module_id: fixture.module.clone(),
            base_binding_fingerprint: fixture.base.debug_text(),
            statement_fingerprint: "Task269C cross-family statement".to_owned(),
            witness_fingerprint: "Task269C cross-family witness".to_owned(),
            primary_term_fingerprint: "Task269C cross-family term".to_owned(),
            binding_env: fixture.base.clone(),
            final_binding_fingerprint: fixture.base.debug_text(),
            declarations: SourceProofLocalDeclarationTable { rows: Vec::new() },
        }
    }

    fn assemble_empty(
        typed_ast: &TypedAst,
    ) -> Result<ResolvedTypedAst, crate::resolved_typed_ast::ResolvedTypedAstError> {
        assemble_with_node_hints(typed_ast, Vec::new())
    }

    fn assemble_with_node_hints(
        typed_ast: &TypedAst,
        node_hints: Vec<ResolvedNodeKindHint>,
    ) -> Result<ResolvedTypedAst, crate::resolved_typed_ast::ResolvedTypedAstError> {
        let cluster_facts = ClusterFactTable::new();
        let overload_collection = OverloadCollectionOutput::collect(
            Vec::<OverloadSiteInput>::new(),
            Vec::<OverloadCandidateInput>::new(),
        );
        let template_expansion = TemplateExpansionOutput::expand(&overload_collection);
        let viability = CandidateViabilityOutput::filter(
            &template_expansion,
            Vec::<CandidateViabilityInput>::new(),
        );
        let specificity =
            SpecificityGraphOutput::build(&viability, Vec::<SpecificityComparisonInput>::new());
        let overload_selection = OverloadSelectionOutput::resolve(
            &specificity,
            Vec::<OverloadSiteResolutionInput>::new(),
        );
        ResolvedTypedAst::assemble(ResolvedTypedAstInputs {
            typed_ast,
            cluster_facts: &cluster_facts,
            overload_collection: &overload_collection,
            template_expansion: &template_expansion,
            viability: &viability,
            specificity: &specificity,
            overload_selection: &overload_selection,
            expressions: Vec::new(),
            node_hints,
            statement_semantics: None,
            statement_proofs: None,
        })
    }

    fn source_id() -> SourceId {
        let snapshot = BuildSnapshotId::from_published_schema_str(&format!(
            "mizar-session-build-snapshot-v1:{}",
            "c9".repeat(32)
        ))
        .expect("Task269C snapshot");
        InMemorySessionIdAllocator::new()
            .next_source_id(snapshot)
            .expect("Task269C source")
    }

    fn other_source_id() -> SourceId {
        let snapshot = BuildSnapshotId::from_published_schema_str(&format!(
            "mizar-session-build-snapshot-v1:{}",
            "ca".repeat(32)
        ))
        .expect("Task269C other snapshot");
        let allocator = InMemorySessionIdAllocator::new();
        allocator
            .next_source_id(snapshot)
            .expect("Task269C skipped source");
        allocator
            .next_source_id(snapshot)
            .expect("Task269C other source")
    }
}
