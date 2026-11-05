use crate::{
    binding_env::{
        BinderIdentity, BindingContextDraft, BindingContextId, BindingContextLayer,
        BindingContextOwner, BindingContextRecovery, BindingContextTable, BindingDiagnosticTable,
        BindingDraft, BindingEnv, BindingEnvParts, BindingId, BindingKind, BindingLookupResult,
        BindingLookupSite, BindingRecoveryState, BindingStatus, BindingTable, BindingTypeSite,
        CapturedFreeVariables,
    },
    source_statement::{
        SourceStatementHandoff, SourceStatementWitnessHandoff, SourceStatementWitnessId,
        SourceStatementWitnessNameId, SourceStatementWitnessTermTarget,
    },
    source_term::SourcePrimaryTermHandoff,
    typed_ast::{
        NodeRecoveryState, SourceNode, TypedArena, TypedAst, source_child_at, source_children,
        source_contiguous_child_count, source_unique_child, unique_source_node,
    },
};
#[cfg(test)]
use crate::{
    source_statement::SourceStatementWitnessError,
    source_term::SourcePrimaryTermId,
    typed_ast::{TypedArenaBuilder, TypedNode, TypedNodeId, TypedNodeLinks, TypingState},
};
use mizar_resolve::{
    env::{DefinitionId, SourceContributionId, SymbolEnv},
    names::{LocalTermBinding, LocalTermScope, ResolvedVariableScope, SourceVariableBindingKind},
    resolved_ast::{ModuleId, ResolvedNodeId, SurfaceResolvedArena, SymbolId},
};
#[cfg(test)]
use mizar_session::SourceAnchor;
use mizar_session::{SourceId, SourceRange};
use mizar_syntax::ast::{
    SurfaceFormulaConnective, SurfaceFormulaConstant, SurfaceNodeKind, SurfaceQuantifierKind,
    SurfaceTokenKind,
};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;
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

/// State of retained source intent, without a checked goal or accepted evidence.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum SourceProofIntentStatus {
    /// Source operation awaits its logical or evidence elaboration.
    Pending,
    /// An authenticated source dependency prevents this intent from proceeding.
    Blocked,
}

/// A derived, unproved goal. Origins identify source provenance, not unchanged syntax.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum SourceProofGoal {
    Equality {
        origin: crate::typed_ast::TypedSiteRef,
        left: BindingId,
        right: BindingId,
    },
    Conjunction {
        origin: crate::typed_ast::TypedSiteRef,
        left: Box<Self>,
        right: Box<Self>,
    },
    Universal {
        origin: crate::typed_ast::TypedSiteRef,
        binding: BindingId,
        radix: mizar_resolve::names::SourceVariableTypeRadix,
        body: Box<Self>,
    },
    Existential {
        origin: crate::typed_ast::TypedSiteRef,
        binding: BindingId,
        radix: mizar_resolve::names::SourceVariableTypeRadix,
        body: Box<Self>,
    },
}

/// One source-ordered transformation; all proof obligations remain unproved.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceProofGoalStep {
    owner: crate::typed_ast::TypedSiteRef,
    context: BindingContextId,
    site: crate::typed_ast::TypedSiteRef,
    matched: Option<(BindingId, BindingId)>,
    before: SourceProofGoal,
    after: SourceProofGoal,
    free: Vec<BindingId>,
    avoided: Vec<BindingId>,
}

impl SourceProofGoalStep {
    pub fn owner(&self) -> &crate::typed_ast::TypedSiteRef {
        &self.owner
    }
    pub const fn context(&self) -> BindingContextId {
        self.context
    }
    pub fn site(&self) -> &crate::typed_ast::TypedSiteRef {
        &self.site
    }
    pub const fn matched(&self) -> Option<(BindingId, BindingId)> {
        self.matched
    }
    pub fn before(&self) -> &SourceProofGoal {
        &self.before
    }
    pub fn after(&self) -> &SourceProofGoal {
        &self.after
    }
    pub fn free(&self) -> &[BindingId] {
        &self.free
    }
    pub fn avoided(&self) -> &[BindingId] {
        &self.avoided
    }
}

impl SourceProofGoal {
    fn binding_sets(
        &self,
        bound: &mut Vec<BindingId>,
        free: &mut BTreeSet<BindingId>,
        retained: &mut BTreeSet<BindingId>,
    ) {
        match self {
            Self::Equality { left, right, .. } => {
                for id in [left, right] {
                    if !bound.contains(id) {
                        free.insert(*id);
                    }
                }
            }
            Self::Conjunction { left, right, .. } => {
                left.binding_sets(bound, free, retained);
                right.binding_sets(bound, free, retained);
            }
            Self::Universal { binding, body, .. } | Self::Existential { binding, body, .. } => {
                retained.insert(*binding);
                bound.push(*binding);
                body.binding_sets(bound, free, retained);
                bound.pop();
            }
        }
    }

    fn substitute(&self, formal: BindingId, actual: BindingId) -> Self {
        match self {
            Self::Equality {
                origin,
                left,
                right,
            } => Self::Equality {
                origin: origin.clone(),
                left: if *left == formal { actual } else { *left },
                right: if *right == formal { actual } else { *right },
            },
            Self::Conjunction {
                origin,
                left,
                right,
            } => Self::Conjunction {
                origin: origin.clone(),
                left: Box::new(left.substitute(formal, actual)),
                right: Box::new(right.substitute(formal, actual)),
            },
            Self::Universal {
                origin,
                binding,
                radix,
                body,
            } => Self::Universal {
                origin: origin.clone(),
                binding: *binding,
                radix: *radix,
                body: if *binding == formal {
                    body.clone()
                } else {
                    Box::new(body.substitute(formal, actual))
                },
            },
            Self::Existential {
                origin,
                binding,
                radix,
                body,
            } => Self::Existential {
                origin: origin.clone(),
                binding: *binding,
                radix: *radix,
                body: if *binding == formal {
                    body.clone()
                } else {
                    Box::new(body.substitute(formal, actual))
                },
            },
        }
    }

    // Replay checks each result edge/leaf independently of the producer traversal.
    fn replays_substitution(&self, before: &Self, formal: BindingId, actual: BindingId) -> bool {
        match (before, self) {
            (
                Self::Equality {
                    origin: a,
                    left: l,
                    right: r,
                },
                Self::Equality {
                    origin: b,
                    left,
                    right,
                },
            ) => {
                a == b
                    && *left == if *l == formal { actual } else { *l }
                    && *right == if *r == formal { actual } else { *r }
            }
            (
                Self::Conjunction {
                    origin: a,
                    left: l,
                    right: r,
                },
                Self::Conjunction {
                    origin: b,
                    left,
                    right,
                },
            ) => {
                a == b
                    && left.replays_substitution(l, formal, actual)
                    && right.replays_substitution(r, formal, actual)
            }
            (
                Self::Universal {
                    origin: a,
                    binding: x,
                    radix: t,
                    body: p,
                },
                Self::Universal {
                    origin: b,
                    binding: y,
                    radix: u,
                    body: q,
                },
            )
            | (
                Self::Existential {
                    origin: a,
                    binding: x,
                    radix: t,
                    body: p,
                },
                Self::Existential {
                    origin: b,
                    binding: y,
                    radix: u,
                    body: q,
                },
            ) => {
                a == b
                    && x == y
                    && t == u
                    && if *x == formal {
                        p == q
                    } else {
                        q.replays_substitution(p, formal, actual)
                    }
            }
            _ => false,
        }
    }

    fn alpha_matches(&self, other: &Self, binders: &mut Vec<(BindingId, BindingId)>) -> bool {
        let same = |left: BindingId, right: BindingId, pairs: &[(BindingId, BindingId)]| match (
            pairs.iter().rposition(|(a, _)| *a == left),
            pairs.iter().rposition(|(_, b)| *b == right),
        ) {
            (Some(a), Some(b)) => a == b,
            (None, None) => left == right,
            _ => false,
        };
        match (self, other) {
            (
                Self::Equality {
                    left: a, right: b, ..
                },
                Self::Equality {
                    left: c, right: d, ..
                },
            ) => same(*a, *c, binders) && same(*b, *d, binders),
            (
                Self::Conjunction {
                    left: a, right: b, ..
                },
                Self::Conjunction {
                    left: c, right: d, ..
                },
            ) => a.alpha_matches(c, binders) && b.alpha_matches(d, binders),
            (
                Self::Universal {
                    binding: a,
                    radix: t,
                    body: p,
                    ..
                },
                Self::Universal {
                    binding: b,
                    radix: u,
                    body: q,
                    ..
                },
            )
            | (
                Self::Existential {
                    binding: a,
                    radix: t,
                    body: p,
                    ..
                },
                Self::Existential {
                    binding: b,
                    radix: u,
                    body: q,
                    ..
                },
            ) => {
                if t != u {
                    return false;
                }
                binders.push((*a, *b));
                let result = p.alpha_matches(q, binders);
                binders.pop();
                result
            }
            _ => false,
        }
    }
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
    #[cfg(test)]
    pub(crate) fn ownership_sentinel_for_test(
        source_id: SourceId,
        module_id: ModuleId,
        binding_env: BindingEnv,
    ) -> Self {
        Self {
            source_id,
            module_id,
            base_binding_fingerprint: binding_env.debug_text(),
            statement_fingerprint: String::new(),
            witness_fingerprint: String::new(),
            primary_term_fingerprint: String::new(),
            final_binding_fingerprint: binding_env.debug_text(),
            binding_env,
            declarations: SourceProofLocalDeclarationTable { rows: Vec::new() },
        }
    }

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
        #[cfg(not(test))]
        {
            let _ = arena;
            Err(SourceProofLocalDeclarationError::DependencyMismatch)
        }
        #[cfg(test)]
        {
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
                return Err(SourceProofLocalDeclarationError::InvalidDeclaration {
                    declaration: id,
                });
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
        #[cfg(not(test))]
        {
            let _ = arena;
            Err(SourceProofLocalDeclarationError::DependencyMismatch)
        }
        #[cfg(test)]
        {
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
                return Err(SourceProofLocalDeclarationError::InvalidDeclaration {
                    declaration: id,
                });
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

#[cfg(test)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LowerFailure {
    Aggregate,
    Declaration,
}

#[cfg(test)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FrozenProfile {
    Task258B3N,
    Task258B3M1,
}

#[cfg(test)]
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

#[cfg(test)]
fn validate_lower_dependencies(
    source_id: SourceId,
    module_id: &ModuleId,
    statements: &SourceStatementHandoff,
    witnesses: &SourceStatementWitnessHandoff,
    primary_terms: &SourcePrimaryTermHandoff,
) -> Result<LowerValidation, SourceProofLocalDeclarationError> {
    let profile = if statements.is_task_258b3m1_profile() {
        FrozenProfile::Task258B3M1
    } else if statements.is_task_258b3n_profile() {
        FrozenProfile::Task258B3N
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

#[cfg(test)]
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

#[cfg(test)]
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

#[cfg(test)]
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

#[cfg(test)]
fn exact_local_fields(local: &LocalTermBinding, profile: FrozenProfile) -> bool {
    let (start, end) = frozen_local_range(profile);
    local.spelling() == "y"
        && local.scope().path() == [0]
        && local.declaration_range().start == start
        && local.declaration_range().end == end
        && local.visible_after_ordinal() == 1
}

#[cfg(test)]
fn exact_local(source_id: SourceId, profile: FrozenProfile) -> LocalTermBinding {
    let (start, end) = frozen_local_range(profile);
    LocalTermBinding::new(
        "y",
        LocalTermScope::new(vec![0]),
        range(source_id, start, end),
        1,
    )
}

#[cfg(test)]
const fn frozen_local_range(profile: FrozenProfile) -> (usize, usize) {
    match profile {
        #[cfg(test)]
        FrozenProfile::Task258B3N => (81, 82),
        FrozenProfile::Task258B3M1 => (84, 85),
    }
}

#[cfg(test)]
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

#[cfg(test)]
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

#[cfg(test)]
fn range(source_id: SourceId, start: usize, end: usize) -> SourceRange {
    SourceRange {
        source_id,
        start,
        end,
    }
}

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
fn exact_frozen_arena(profile: FrozenProfile, source_id: SourceId, arena: &TypedArena) -> bool {
    match profile {
        #[cfg(test)]
        FrozenProfile::Task258B3N => exact_task258b3n_arena(source_id, arena),
        FrozenProfile::Task258B3M1 => exact_task258b3m1_arena(source_id, arena),
    }
}

#[cfg(test)]
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

#[cfg(test)]
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

#[cfg(test)]
fn frozen_canonical_arena(profile: FrozenProfile, source_id: SourceId) -> TypedArena {
    match profile {
        #[cfg(test)]
        FrozenProfile::Task258B3N => task258b3n_canonical_arena(source_id),
        FrozenProfile::Task258B3M1 => task258b3m1_canonical_arena(source_id),
    }
}

#[cfg(test)]
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

#[cfg(test)]
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

/// Authenticated native proof-local declarations and their lexical binding environment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceProofLocalBindingHandoff {
    original: Arc<TypedAst>,
    native: Arc<SurfaceResolvedArena>,
    symbols: Arc<SymbolEnv>,
    scope: Box<ResolvedVariableScope>,
    binding_env: BindingEnv,
}

impl SourceProofLocalBindingHandoff {
    pub fn binding_env(&self) -> &BindingEnv {
        &self.binding_env
    }
    pub fn source(&self) -> &TypedAst {
        &self.original
    }
    pub fn scope(&self) -> &ResolvedVariableScope {
        &self.scope
    }
    pub fn debug_text(&self) -> String {
        format!(
            "source-proof-local-binding-debug-v1\n{}",
            self.binding_env.debug_text()
        )
    }

    fn witness_goal_site(
        &self,
        row: &SourceNode,
    ) -> Result<crate::typed_ast::TypedSiteRef, SourceProofLocalDeclarationError> {
        let invalid = || SourceProofLocalDeclarationError::DependencyMismatch;
        let mut found = self
            .original
            .nodes()
            .iter()
            .filter(|(_, node)| node.resolved_node == Some(row.site()));
        let (id, node) = found.next().ok_or_else(invalid)?;
        if found.next().is_some()
            || node.anchor != mizar_session::SourceAnchor::Range(row.range())
            || node.recovery != NodeRecoveryState::Normal
        {
            return Err(invalid());
        }
        Ok(crate::typed_ast::TypedSiteRef::Node(id))
    }

    fn witness_goal_binding(
        &self,
        name: ResolvedNodeId,
    ) -> Result<BindingId, SourceProofLocalDeclarationError> {
        let invalid = || SourceProofLocalDeclarationError::DependencyMismatch;
        let source = self
            .scope
            .bindings()
            .iter()
            .find(|binding| self.native.resolved_node_for(binding.node()) == Some(name))
            .ok_or_else(invalid)?;
        let identity = BinderIdentity::ResolverLocal {
            scope: source.scope().clone(),
            ordinal: source.ordinal(),
            declaration_range: source.range(),
        };
        let mut entries = self
            .binding_env
            .bindings()
            .iter()
            .filter(|(_, entry)| entry.identity == identity);
        let (id, entry) = entries.next().ok_or_else(invalid)?;
        if entries.next().is_some()
            || entry.spelling != source.spelling()
            || entry.declaration_range != source.range()
            || entry.visible_after_ordinal != source.ordinal()
        {
            return Err(invalid());
        }
        Ok(id)
    }

    fn witness_goal_guard(
        &self,
        binding: BindingId,
        request: &crate::typed_ast::TypedSiteRef,
        role: &str,
    ) -> Result<crate::type_checker::TypeExpressionInput, SourceProofLocalDeclarationError> {
        use crate::{
            type_checker::{TypeExpressionInput, TypeHeadInput},
            typed_ast::{TypeRole, TypedSiteRef},
        };
        use SurfaceNodeKind as K;
        use mizar_resolve::names::SourceVariableTypeRadix as R;
        let invalid = || SourceProofLocalDeclarationError::DependencyMismatch;
        let rows = self.original.source_nodes().ok_or_else(invalid)?;
        let entry = self
            .binding_env
            .bindings()
            .get(binding)
            .ok_or_else(invalid)?;
        let BinderIdentity::ResolverLocal {
            scope,
            ordinal,
            declaration_range,
        } = &entry.identity
        else {
            return Err(invalid());
        };
        let source = self
            .scope
            .bindings()
            .iter()
            .find(|b| {
                b.scope() == scope && b.ordinal() == *ordinal && b.range() == *declaration_range
            })
            .ok_or_else(invalid)?;
        if !matches!(
            source.kind(),
            SourceVariableBindingKind::Let | SourceVariableBindingKind::Quantifier
        ) || entry.type_site != BindingTypeSite::Missing
        {
            return Err(invalid());
        }
        let name = rows
            .iter()
            .find(|r| Some(r.site()) == self.native.resolved_node_for(source.node()))
            .ok_or_else(invalid)?;
        let segment = rows
            .iter()
            .find(|r| Some(r.site()) == name.parent())
            .ok_or_else(invalid)?;
        if !matches!(
            segment.kind(),
            K::QuantifierVariableSegment | K::QualifiedVariableSegment
        ) {
            return Err(invalid());
        }
        let parts = source_proof_frame(rows, segment).ok_or_else(invalid)?;
        if parts.len() < 3 || parts.len().is_multiple_of(2) {
            return Err(invalid());
        }
        let end = parts.len() - 2;
        if !(source_inline_token(&parts, end, "be") || source_inline_token(&parts, end, "being"))
            || parts[..end].iter().enumerate().any(|(i, r)| {
                if i.is_multiple_of(2) {
                    !matches!(r.kind(), K::Token(t) if t.kind == SurfaceTokenKind::Identifier)
                } else {
                    !source_inline_token(&parts, i, ",")
                }
            })
        {
            return Err(invalid());
        }
        let ty = parts[end + 1];
        let head = source_unique_child(rows, ty, K::TypeHead).ok_or_else(invalid)?;
        let words = source_proof_frame(rows, head).ok_or_else(invalid)?;
        if ty.kind() != &K::TypeExpression
            || source_children(rows, ty.site()).count() != 1
            || words.len() != 1
        {
            return Err(invalid());
        }
        let (radix, head) = match words[0].kind() {
            K::Token(t) if t.kind == SurfaceTokenKind::ReservedWord && t.text.as_ref() == "set" => {
                (R::Set, TypeHeadInput::BuiltinSet)
            }
            K::Token(t)
                if t.kind == SurfaceTokenKind::ReservedWord && t.text.as_ref() == "object" =>
            {
                (R::Object, TypeHeadInput::BuiltinObject)
            }
            _ => return Err(invalid()),
        };
        if source
            .declared_type()
            .is_none_or(|t| t.radix() != radix || !t.attributes().is_empty())
        {
            return Err(invalid());
        }
        Ok(TypeExpressionInput::new(
            TypedSiteRef::Role {
                node: request.node(),
                role: TypeRole::new(format!("witness-goal.type.{role}")),
            },
            ty.range(),
            ty.spelling(),
            head,
        ))
    }

    fn witness_goal_value(
        &self,
        row: &SourceNode,
    ) -> Result<BindingId, SourceProofLocalDeclarationError> {
        use SurfaceNodeKind as K;
        let invalid = || SourceProofLocalDeclarationError::DependencyMismatch;
        let rows = self.original.source_nodes().ok_or_else(invalid)?;
        let parts = source_proof_frame(rows, row).ok_or_else(invalid)?;
        let [reference] = parts.as_slice() else {
            return Err(invalid());
        };
        let words = source_proof_frame(rows, reference).ok_or_else(invalid)?;
        if row.kind() != &K::TermExpression
            || reference.kind() != &K::TermReference
            || words.len() != 1
            || !matches!(words[0].kind(), K::Token(t) if t.kind == SurfaceTokenKind::Identifier)
        {
            return Err(invalid());
        }
        let reference = self
            .scope
            .references()
            .iter()
            .find(|r| self.native.resolved_node_for(r.node()) == Some(reference.site()))
            .ok_or_else(invalid)?;
        if reference.kind() != mizar_resolve::names::SourceVariableReferenceKind::Term {
            return Err(invalid());
        }
        let source = self
            .scope
            .bindings()
            .iter()
            .find(|b| b.id() == reference.binding())
            .ok_or_else(invalid)?;
        self.witness_goal_binding(
            self.native
                .resolved_node_for(source.node())
                .ok_or_else(invalid)?,
        )
    }

    fn witness_goal_formula(
        &self,
        row: &SourceNode,
    ) -> Result<SourceProofGoal, SourceProofLocalDeclarationError> {
        use crate::type_checker::TypeHeadInput;
        use SurfaceNodeKind as K;
        use mizar_resolve::names::SourceVariableTypeRadix as R;
        let invalid = || SourceProofLocalDeclarationError::DependencyMismatch;
        let rows = self.original.source_nodes().ok_or_else(invalid)?;
        let parts = source_proof_frame(rows, row).ok_or_else(invalid)?;
        let origin = self.witness_goal_site(row)?;
        match row.kind() {
            K::FormulaExpression if parts.len() == 1 => self.witness_goal_formula(parts[0]),
            K::ParenthesizedFormula
                if parts.len() == 3
                    && source_inline_token(&parts, 0, "(")
                    && source_inline_token(&parts, 2, ")") =>
            {
                self.witness_goal_formula(parts[1])
            }
            K::BuiltinPredicateApplication
                if parts.len() == 3 && source_inline_token(&parts, 1, "=") =>
            {
                Ok(SourceProofGoal::Equality {
                    origin,
                    left: self.witness_goal_value(parts[0])?,
                    right: self.witness_goal_value(parts[2])?,
                })
            }
            K::BinaryFormula(op)
                if op.connective == SurfaceFormulaConnective::And
                    && !op.repeated
                    && parts.len() == 3
                    && source_inline_token(&parts, 1, "&") =>
            {
                Ok(SourceProofGoal::Conjunction {
                    origin,
                    left: Box::new(self.witness_goal_formula(parts[0])?),
                    right: Box::new(self.witness_goal_formula(parts[2])?),
                })
            }
            K::QuantifiedFormula(kind) => {
                if !matches!(parts.first().map(|r| r.kind()), Some(K::Token(t)) if t.kind == SurfaceTokenKind::ReservedWord && t.text.as_ref() == if *kind == SurfaceQuantifierKind::Universal { "for" } else { "ex" })
                {
                    return Err(invalid());
                }
                let mut cursor = 1;
                let mut binders = Vec::new();
                loop {
                    let segment = *parts.get(cursor).ok_or_else(invalid)?;
                    if segment.kind() != &K::QuantifierVariableSegment {
                        return Err(invalid());
                    }
                    for name in source_children(rows, segment.site()).filter(|r| matches!(r.kind(), K::Token(t) if t.kind == SurfaceTokenKind::Identifier)) {
                        let binding = self.witness_goal_binding(name.site())?;
                        let ty = self.witness_goal_guard(binding, &self.witness_goal_site(name)?, "binder")?;
                        let radix = match ty.head { TypeHeadInput::BuiltinSet => R::Set, TypeHeadInput::BuiltinObject => R::Object, _ => return Err(invalid()) };
                        binders.push((binding, radix));
                    }
                    cursor += 1;
                    if source_inline_token(&parts, cursor, ",") {
                        cursor += 1;
                    } else {
                        break;
                    }
                }
                if matches!(parts.get(cursor).map(|r| r.kind()), Some(K::Token(t)) if t.kind == SurfaceTokenKind::ReservedWord && t.text.as_ref() == if *kind == SurfaceQuantifierKind::Universal { "holds" } else { "st" })
                {
                    cursor += 1;
                } else if *kind != SurfaceQuantifierKind::Universal
                    || !matches!(
                        parts.get(cursor).map(|r| r.kind()),
                        Some(K::QuantifiedFormula(_))
                    )
                {
                    return Err(invalid());
                }
                if binders.is_empty() || parts.len() != cursor + 1 {
                    return Err(invalid());
                }
                let mut body = self.witness_goal_formula(parts[cursor])?;
                for (binding, radix) in binders.into_iter().rev() {
                    body = if *kind == SurfaceQuantifierKind::Universal {
                        SourceProofGoal::Universal {
                            origin: origin.clone(),
                            binding,
                            radix,
                            body: Box::new(body),
                        }
                    } else {
                        SourceProofGoal::Existential {
                            origin: origin.clone(),
                            binding,
                            radix,
                            body: Box::new(body),
                        }
                    };
                }
                Ok(body)
            }
            _ => Err(invalid()),
        }
    }

    // Source-derived products are checked again at construction, replay and coherent-input tests.
    fn witness_goal_conversion_checkpoints(
        &self,
        requests: &[crate::type_checker::CoercionInput],
        checked: &crate::type_checker::CoercionCheckingOutput,
        queries: &[crate::type_checker::TypeFactQueryOutput],
        intents: &[(
            crate::typed_ast::TypedSiteRef,
            Option<crate::typed_ast::TypedSiteRef>,
            SourceProofIntentStatus,
        )],
    ) -> Result<BTreeMap<crate::typed_ast::TypedSiteRef, BindingId>, SourceProofLocalDeclarationError>
    {
        use crate::{
            type_checker::{CoercionJustification, CoercionRequestKind, TypeFactQueryStatus},
            typed_ast::CoercionStatus,
        };
        use SurfaceNodeKind as K;
        let invalid = || SourceProofLocalDeclarationError::DependencyMismatch;
        let (fresh_requests, fresh_checked, fresh_queries) = self.check_conversions()?;
        if requests != fresh_requests
            || checked != &fresh_checked
            || queries != fresh_queries
            || intents != self.check_proof_intents()?.0
        {
            return Err(invalid());
        }
        let rows = self.original.source_nodes().ok_or_else(invalid)?;
        let mut checkpoints = BTreeMap::new();
        for request in requests {
            let item = rows
                .iter()
                .find(|r| self.witness_goal_site(r).as_ref() == Ok(&request.site))
                .ok_or_else(invalid)?;
            let (name, value) = source_reconsider_item_roles(rows, item).ok_or_else(invalid)?;
            let value = value.ok_or_else(invalid)?;
            let statement = rows
                .iter()
                .find(|r| Some(r.site()) == item.parent())
                .ok_or_else(invalid)?;
            let (_, target, tail) =
                source_reconsider_statement_roles(rows, statement).ok_or_else(invalid)?;
            if tail.is_some_and(|r| r.kind() != &K::JustificationClause) {
                return Err(invalid());
            }
            let proof = rows
                .iter()
                .find(|r| Some(r.site()) == statement.parent())
                .ok_or_else(invalid)?;
            let owner = rows
                .iter()
                .find(|r| Some(r.site()) == proof.parent())
                .ok_or_else(invalid)?;
            if proof.kind() != &K::ProofBlock || owner.kind() != &K::TheoremItem {
                return Err(invalid());
            }
            let alias = self
                .scope
                .bindings()
                .iter()
                .find(|b| self.native.resolved_node_for(b.node()) == Some(name.site()))
                .ok_or_else(invalid)?;
            if alias.kind() != SourceVariableBindingKind::Reconsider
                || self
                    .scope
                    .references()
                    .iter()
                    .any(|r| r.binding() == alias.id())
            {
                return Err(invalid());
            }
            let rhs = self.witness_goal_value(value)?;
            let binding = self.binding_env.bindings().get(rhs).ok_or_else(invalid)?;
            let alias_entry = self
                .binding_env
                .bindings()
                .get(self.witness_goal_binding(name.site())?)
                .ok_or_else(invalid)?;
            if binding.kind != BindingKind::LetBinding
                || binding.owner_context != alias_entry.owner_context
            {
                return Err(invalid());
            }
            let original = self.witness_goal_guard(rhs, &request.site, "checkpoint-source")?;
            let from = request.from_type.as_ref().ok_or_else(invalid)?;
            if from.head != original.head
                || from.source_range != original.source_range
                || from.spelling != original.spelling
                || !from.args.is_empty()
                || !from.attributes.is_empty()
                || request.source_range != item.range()
                || request.to_type.site.node() != self.witness_goal_site(target)?.node()
                || !matches!(
                    request.kind,
                    CoercionRequestKind::Widening | CoercionRequestKind::Narrowing
                )
            {
                return Err(invalid());
            }
            let result = checked
                .coercions()
                .iter()
                .map(|(_, r)| r)
                .find(|r| r.site == request.site)
                .ok_or_else(invalid)?;
            let query = queries
                .iter()
                .find(|q| q.query().subject == request.site)
                .ok_or_else(invalid)?;
            if matches!(
                result.status,
                CoercionStatus::Rejected | CoercionStatus::Blocked
            ) {
                for source in [item, statement, proof, owner] {
                    let site = self.witness_goal_site(source)?;
                    if !intents.iter().any(|(s, _, status)| {
                        *s == site && *status == SourceProofIntentStatus::Blocked
                    }) {
                        return Err(invalid());
                    }
                }
            }
            if request.kind == CoercionRequestKind::Narrowing
                && request.justification == CoercionJustification::Omitted
                && (result.status != CoercionStatus::Rejected
                    || result.obligation.is_some()
                    || query.status() != TypeFactQueryStatus::Missing
                    || !checked.diagnostics().iter().any(|(_, d)| {
                        d.owner.as_ref() == Some(&request.site)
                            && d.source_range == item.range()
                            && d.message_key == "type.narrowing_requires_proof"
                    }))
            {
                return Err(invalid());
            }
            if checkpoints.insert(request.site.clone(), rhs).is_some() {
                return Err(invalid());
            }
        }
        Ok(checkpoints)
    }

    // Rationale: shared source census is used by construction, independent replay and corruption tests.
    #[allow(clippy::type_complexity)]
    fn witness_goal_inputs(
        &self,
    ) -> Result<
        (
            Vec<(
                crate::typed_ast::TypedSiteRef,
                BindingContextId,
                crate::typed_ast::TypedSiteRef,
                Option<BindingId>,
            )>,
            BTreeMap<crate::typed_ast::TypedSiteRef, SourceProofGoal>,
            Vec<crate::type_checker::TermInput>,
            Vec<crate::type_checker::FormulaInput>,
            BTreeMap<crate::typed_ast::TypedSiteRef, BindingId>,
        ),
        SourceProofLocalDeclarationError,
    > {
        use crate::{
            type_checker::{FormulaInput, FormulaKind, TermInput, TermKind, TermReference},
            typed_ast::{TypeRole, TypedSiteRef},
        };
        use SurfaceNodeKind as K;
        let invalid = || SourceProofLocalDeclarationError::DependencyMismatch;
        let (intents, _) = self.check_proof_intents()?;
        let (conversions, conversion_output, queries) = self.check_conversions()?;
        let checkpoints = self.witness_goal_conversion_checkpoints(
            &conversions,
            &conversion_output,
            &queries,
            &intents,
        )?;
        let rows = self.original.source_nodes().ok_or_else(invalid)?;
        let mut theorem_rows = BTreeSet::new();
        let mut metadata_rows = BTreeSet::new();
        for row in rows.iter() {
            let mut current = Some(row);
            let mut theorem = false;
            let mut metadata = false;
            while let Some(ancestor) = current {
                theorem |= ancestor.kind() == &K::TheoremItem;
                metadata |= matches!(
                    ancestor.kind(),
                    K::DefinitionBlockItem | K::RegistrationBlockItem
                );
                current = ancestor
                    .parent()
                    .map(|id| rows.iter().find(|r| r.site() == id).ok_or_else(invalid))
                    .transpose()?;
            }
            if theorem && metadata {
                return Err(invalid());
            }
            if theorem {
                theorem_rows.insert(row.site());
            }
            if metadata {
                metadata_rows.insert(row.site());
            }
            if row.kind() == &K::ItemList
                && source_children(rows, row.site()).any(|r| {
                    !matches!(
                        r.kind(),
                        K::TheoremItem
                            | K::DefinitionBlockItem
                            | K::RegistrationBlockItem
                            | K::VisibleItem
                    )
                })
            {
                return Err(invalid());
            }
            if row.kind() == &K::VisibleItem && !theorem && !metadata {
                let parts = source_proof_frame(rows, row).ok_or_else(invalid)?;
                if parts.len() != 2
                    || parts[0].kind() != &K::VisibilityMarker
                    || !matches!(
                        parts[1].kind(),
                        K::TheoremItem | K::DefinitionBlockItem | K::RegistrationBlockItem
                    )
                {
                    return Err(invalid());
                }
            }
            if !theorem
                && !metadata
                && (row.kind() == &K::LemmaItem
                    || row.kind() == &K::ProofBlock
                    || row.kind() == &K::TermExpression
                    || row.kind() == &K::BuiltinPredicateApplication
                    || row.kind() == &K::StatementItem
                    || source_proof_statement_kind(row.kind()))
            {
                return Err(invalid());
            }
        }
        let (source_scopes, _) = mizar_resolve::symbols::source_proof_scope_maps(
            &self.native,
            &self.symbols,
            &self.scope,
        )
        .map_err(|_| invalid())?;
        let mut contexts = BTreeMap::new();
        for (node, scope) in source_scopes {
            let site = self.native.resolved_node_for(node).ok_or_else(invalid)?;
            let mut matching = self
                .binding_env
                .contexts()
                .iter()
                .filter(|(_, c)| c.lexical_scope.as_ref() == Some(&scope));
            let (context, _) = matching.next().ok_or_else(invalid)?;
            if matching.next().is_some() {
                return Err(invalid());
            }
            contexts.insert(site, context);
        }
        let mut terms = Vec::new();
        for binding in self.scope.bindings() {
            let name_site = self
                .native
                .resolved_node_for(binding.node())
                .ok_or_else(invalid)?;
            if metadata_rows.contains(&name_site) {
                continue;
            }
            if !theorem_rows.contains(&name_site) {
                return Err(invalid());
            }
            if !matches!(
                binding.kind(),
                SourceVariableBindingKind::Let
                    | SourceVariableBindingKind::Quantifier
                    | SourceVariableBindingKind::NamedWitness
                    | SourceVariableBindingKind::Reconsider
            ) {
                return Err(invalid());
            }
            if matches!(
                binding.kind(),
                SourceVariableBindingKind::NamedWitness | SourceVariableBindingKind::Reconsider
            ) {
                continue;
            }
            let row = rows
                .iter()
                .find(|r| Some(r.site()) == self.native.resolved_node_for(binding.node()))
                .ok_or_else(invalid)?;
            let id = self.witness_goal_binding(row.site())?;
            let entry = self.binding_env.bindings().get(id).ok_or_else(invalid)?;
            let site = TypedSiteRef::Role {
                node: self.witness_goal_site(row)?.node(),
                role: TypeRole::new("witness-goal.declaration-value"),
            };
            terms.push(
                TermInput::new(
                    site.clone(),
                    entry.owner_context,
                    row.range(),
                    TermKind::Variable,
                )
                .with_reference(TermReference::Binding(id))
                .with_result_type(self.witness_goal_guard(
                    id,
                    &site,
                    "declaration-result",
                )?),
            );
        }
        let mut formulas = Vec::new();
        for row in rows.iter().filter(|r| theorem_rows.contains(&r.site())) {
            if row.kind() == &K::TermExpression {
                let id = self.witness_goal_value(row)?;
                let site = self.witness_goal_site(row)?;
                terms.push(
                    TermInput::new(
                        site.clone(),
                        *contexts.get(&row.site()).ok_or_else(invalid)?,
                        row.range(),
                        TermKind::Variable,
                    )
                    .with_reference(TermReference::Binding(id))
                    .with_result_type(self.witness_goal_guard(id, &site, "occurrence-result")?),
                );
            }
            if row.kind() == &K::BuiltinPredicateApplication {
                let parts = source_proof_frame(rows, row).ok_or_else(invalid)?;
                if parts.len() != 3 || !source_inline_token(&parts, 1, "=") {
                    return Err(invalid());
                }
                let _ = self.witness_goal_formula(row)?;
                formulas.push(
                    FormulaInput::new(
                        self.witness_goal_site(row)?,
                        *contexts.get(&row.site()).ok_or_else(invalid)?,
                        row.range(),
                        FormulaKind::Equality,
                    )
                    .with_terms(vec![
                        self.witness_goal_site(parts[0])?,
                        self.witness_goal_site(parts[2])?,
                    ]),
                );
            }
        }
        let mut events = Vec::new();
        let mut goals = BTreeMap::new();
        let mut used = BTreeSet::new();
        for theorem in rows.iter().filter(|r| r.kind() == &K::TheoremItem) {
            let parts = source_proof_frame(rows, theorem).ok_or_else(invalid)?;
            if parts.len() != 6
                || !matches!(parts[0].kind(), K::Token(t) if t.kind == SurfaceTokenKind::ReservedWord && t.text.as_ref() == "theorem")
                || parts[3].kind() != &K::FormulaExpression
                || parts[4].kind() != &K::ProofBlock
            {
                return Err(invalid());
            }
            let owner = self.witness_goal_site(theorem)?;
            let goal = self.witness_goal_formula(parts[3])?;
            let mut free = BTreeSet::new();
            goal.binding_sets(&mut Vec::new(), &mut free, &mut BTreeSet::new());
            if !free.is_empty() {
                return Err(invalid());
            }
            goals.insert(owner.clone(), goal);
            let proof = parts[4];
            used.insert(proof.site());
            let context = *contexts.get(&proof.site()).ok_or_else(invalid)?;
            if self.binding_env.contexts().get(context).is_none_or(|c| {
                c.layer != BindingContextLayer::Proof
                    || c.owner
                        != (BindingContextOwner::SourceStatement {
                            source_range: proof.range(),
                        })
            }) {
                return Err(invalid());
            }
            events.push((owner.clone(), context, owner.clone(), None));
            let body = source_proof_frame(rows, proof).ok_or_else(invalid)?;
            if body.len() < 3
                || !source_inline_token(&body, 0, "proof")
                || !source_inline_token(&body, body.len() - 1, "end")
            {
                return Err(invalid());
            }
            for (index, statement) in body.iter().enumerate().take(body.len() - 1).skip(1) {
                used.insert(statement.site());
                if index == body.len() - 2 {
                    if statement.kind() != &K::ConclusionStatement {
                        return Err(invalid());
                    }
                    let tail = source_proof_frame(rows, statement).ok_or_else(invalid)?;
                    if tail.len() != 3
                        || !source_inline_token(&tail, 0, "thus")
                        || tail[1].kind() != &K::Proposition
                        || !source_inline_token(&tail, 2, ";")
                    {
                        return Err(invalid());
                    }
                    let formula = source_unique_child(rows, tail[1], K::FormulaExpression)
                        .ok_or_else(invalid)?;
                    if source_children(rows, tail[1].site()).count() != 1 {
                        return Err(invalid());
                    }
                    let content = source_proof_frame(rows, formula).ok_or_else(invalid)?;
                    let thesis = content.len() == 1
                        && content[0].kind() == &K::FormulaConstant(SurfaceFormulaConstant::Thesis)
                        && matches!(source_proof_frame(rows, content[0]).as_deref(), Some([word]) if matches!(word.kind(), K::Token(t) if t.kind == SurfaceTokenKind::ReservedWord && t.text.as_ref() == "thesis"));
                    let site = self.witness_goal_site(statement)?;
                    if !thesis {
                        goals.insert(site.clone(), self.witness_goal_formula(formula)?);
                    }
                    events.push((owner.clone(), context, site, None));
                } else if statement.kind() == &K::LetStatement {
                    let parts = source_proof_frame(rows, statement).ok_or_else(invalid)?;
                    if parts.len() < 3
                        || !source_inline_token(&parts, 0, "let")
                        || !source_inline_token(&parts, parts.len() - 1, ";")
                    {
                        return Err(invalid());
                    }
                    for (i, segment) in parts.iter().enumerate().take(parts.len() - 1).skip(1) {
                        if i.is_multiple_of(2) {
                            if !source_inline_token(&parts, i, ",") {
                                return Err(invalid());
                            }
                            continue;
                        }
                        if segment.kind() != &K::QualifiedVariableSegment {
                            return Err(invalid());
                        }
                        for name in source_children(rows, segment.site()).filter(|r| matches!(r.kind(), K::Token(t) if t.kind == SurfaceTokenKind::Identifier)) {
                            let binding = self.witness_goal_binding(name.site())?;
                            if self.binding_env.bindings().get(binding).is_none_or(|b| b.kind != BindingKind::LetBinding || b.owner_context != context) { return Err(invalid()); }
                            events.push((owner.clone(), context, self.witness_goal_site(name)?, Some(binding)));
                        }
                    }
                } else if statement.kind() == &K::ReconsiderStatement {
                    let (items, _, _) =
                        source_reconsider_statement_roles(rows, statement).ok_or_else(invalid)?;
                    for item in items {
                        let site = self.witness_goal_site(item)?;
                        let binding = *checkpoints.get(&site).ok_or_else(invalid)?;
                        if self
                            .binding_env
                            .bindings()
                            .get(binding)
                            .is_none_or(|b| b.owner_context != context)
                        {
                            return Err(invalid());
                        }
                        used.insert(item.site());
                        events.push((owner.clone(), context, site, None));
                    }
                } else if statement.kind() == &K::TakeStatement {
                    for witness in
                        source_children(rows, statement.site()).filter(|r| r.kind() == &K::Witness)
                    {
                        let values = source_children(rows, witness.site())
                            .filter(|r| r.kind() == &K::TermExpression)
                            .collect::<Vec<_>>();
                        let [value] = values.as_slice() else {
                            return Err(invalid());
                        };
                        let binding = self.witness_goal_value(value)?;
                        if self.binding_env.bindings().get(binding).is_none_or(|b| {
                            b.kind != BindingKind::LetBinding || b.owner_context != context
                        }) {
                            return Err(invalid());
                        }
                        events.push((
                            owner.clone(),
                            context,
                            self.witness_goal_site(witness)?,
                            Some(binding),
                        ));
                    }
                } else {
                    return Err(invalid());
                }
            }
        }
        if goals.is_empty()
            || rows.iter().any(|r| {
                (r.kind() == &K::ProofBlock || source_proof_statement_kind(r.kind()))
                    && !used.contains(&r.site())
                    && !metadata_rows.contains(&r.site())
            })
        {
            return Err(invalid());
        }
        if checkpoints
            .keys()
            .any(|site| events.iter().filter(|(_, _, s, _)| s == site).count() != 1)
        {
            return Err(invalid());
        }
        Ok((events, goals, terms, formulas, checkpoints))
    }

    fn witness_goal_guard_request(
        &self,
        owner: &crate::typed_ast::TypedSiteRef,
        site: &crate::typed_ast::TypedSiteRef,
        formal: BindingId,
        actual: BindingId,
    ) -> Result<
        (
            Option<crate::type_checker::CoercionInput>,
            Option<crate::type_checker::InitialObligationInput>,
        ),
        SourceProofLocalDeclarationError,
    > {
        use crate::{
            type_checker::{
                CoercionEvidence, CoercionInput, CoercionJustification, CoercionRequestKind,
                InitialObligationInput, InitialRequirementKind, TypeHeadInput,
            },
            typed_ast::{InitialObligationGoal, InitialObligationProvenance},
        };
        let invalid = || SourceProofLocalDeclarationError::DependencyMismatch;
        let row = self
            .original
            .nodes()
            .node(site.node())
            .and_then(|n| n.resolved_node)
            .and_then(|id| {
                self.original
                    .source_node_entries()?
                    .find(|r| r.site() == id)
            })
            .ok_or_else(invalid)?;
        if row.kind() != &SurfaceNodeKind::Witness {
            return Err(invalid());
        }
        let from = self.witness_goal_guard(actual, site, "guard-source")?;
        let to = self.witness_goal_guard(formal, site, "guard-target")?;
        if matches!(
            (&from.head, &to.head),
            (
                TypeHeadInput::BuiltinSet,
                TypeHeadInput::BuiltinSet | TypeHeadInput::BuiltinObject
            ) | (TypeHeadInput::BuiltinObject, TypeHeadInput::BuiltinObject)
        ) {
            Ok((
                Some(
                    CoercionInput::new(
                        site.clone(),
                        row.range(),
                        CoercionRequestKind::Widening,
                        to,
                    )
                    .with_from_type(from)
                    .with_evidence(CoercionEvidence::BuiltinRadix)
                    .with_justification(CoercionJustification::Omitted),
                ),
                None,
            ))
        } else if from.head == TypeHeadInput::BuiltinObject && to.head == TypeHeadInput::BuiltinSet
        {
            Ok((
                None,
                Some(
                    InitialObligationInput::new(
                        site.clone(),
                        row.range(),
                        InitialRequirementKind::Narrowing,
                        to,
                    )
                    .with_goal(InitialObligationGoal::new(format!(
                        "source-witness-membership:{}:{}:{}:{}",
                        owner.node().index(),
                        site.node().index(),
                        formal.index(),
                        actual.index()
                    )))
                    .with_provenance(InitialObligationProvenance::new(
                        "source-proof-witness-type",
                    )),
                ),
            ))
        } else {
            Err(invalid())
        }
    }

    fn witness_goal_formation(
        &self,
        goal: &SourceProofGoal,
        frontier: &BTreeSet<BindingId>,
        bound: &mut Vec<BindingId>,
    ) -> Result<(), SourceProofLocalDeclarationError> {
        use crate::type_checker::TypeHeadInput;
        use mizar_resolve::names::SourceVariableTypeRadix as R;
        let invalid = || SourceProofLocalDeclarationError::DependencyMismatch;
        match goal {
            SourceProofGoal::Equality {
                origin,
                left,
                right,
            } => {
                for binding in [left, right] {
                    if !bound.contains(binding) && !frontier.contains(binding) {
                        return Err(invalid());
                    }
                    self.witness_goal_guard(*binding, origin, "formation")?;
                }
            }
            SourceProofGoal::Conjunction { left, right, .. } => {
                self.witness_goal_formation(left, frontier, bound)?;
                self.witness_goal_formation(right, frontier, bound)?;
            }
            SourceProofGoal::Universal {
                origin,
                binding,
                radix,
                body,
            }
            | SourceProofGoal::Existential {
                origin,
                binding,
                radix,
                body,
            } => {
                let guard = self.witness_goal_guard(*binding, origin, "formation")?;
                if bound.contains(binding)
                    || frontier.contains(binding)
                    || !matches!(
                        (&guard.head, radix),
                        (TypeHeadInput::BuiltinSet, R::Set)
                            | (TypeHeadInput::BuiltinObject, R::Object)
                    )
                {
                    return Err(invalid());
                }
                bound.push(*binding);
                self.witness_goal_formation(body, frontier, bound)?;
                bound.pop();
            }
        }
        Ok(())
    }

    // Rationale: real existing checking outputs avoid a new witness-output adapter or slot.
    #[allow(clippy::type_complexity)]
    pub fn check_witness_goals(
        &self,
    ) -> Result<
        (
            Vec<SourceProofGoalStep>,
            Vec<crate::type_checker::CoercionInput>,
            Vec<crate::type_checker::InitialObligationInput>,
            crate::type_checker::CoercionCheckingOutput,
            crate::type_checker::TermFormulaInferenceOutput,
        ),
        SourceProofLocalDeclarationError,
    > {
        use crate::{
            type_checker::{CoercionObligationChecker, TermFormulaChecker},
            typed_ast::TypeFactTable,
        };
        let invalid = || SourceProofLocalDeclarationError::DependencyMismatch;
        let (events, goals, terms, formulas, _) = self.witness_goal_inputs()?;
        let inference = TermFormulaChecker::default()
            .try_infer(&self.symbols, &self.binding_env, terms, formulas)
            .map_err(|_| invalid())?;
        let mut steps = Vec::new();
        let mut current = None;
        let mut inputs = Vec::new();
        let mut obligations = Vec::new();
        for (owner, context, site, actual) in events {
            if site == owner {
                current = Some(goals.get(&owner).ok_or_else(invalid)?.clone());
            }
            let before = current.clone().ok_or_else(invalid)?;
            let mut avoided = BTreeSet::new();
            let (matched, after) = if let Some(actual) = actual {
                let row = self
                    .original
                    .nodes()
                    .node(site.node())
                    .and_then(|n| n.resolved_node)
                    .and_then(|id| {
                        self.original
                            .source_node_entries()?
                            .find(|r| r.site() == id)
                    })
                    .ok_or_else(invalid)?;
                let (formal, body) = match (&before, row.kind()) {
                    (
                        SourceProofGoal::Universal { binding, body, .. },
                        SurfaceNodeKind::Token(_),
                    ) => (*binding, body),
                    (
                        SourceProofGoal::Existential { binding, body, .. },
                        SurfaceNodeKind::Witness,
                    ) => (*binding, body),
                    _ => return Err(invalid()),
                };
                body.binding_sets(&mut Vec::new(), &mut BTreeSet::new(), &mut avoided);
                if row.kind() == &SurfaceNodeKind::Witness {
                    let (input, obligation) =
                        self.witness_goal_guard_request(&owner, &site, formal, actual)?;
                    inputs.extend(input);
                    obligations.extend(obligation);
                }
                (Some((formal, actual)), body.substitute(formal, actual))
            } else {
                (None, before.clone())
            };
            let mut free = BTreeSet::new();
            after.binding_sets(&mut Vec::new(), &mut free, &mut BTreeSet::new());
            steps.push(SourceProofGoalStep {
                owner,
                context,
                site,
                matched,
                before,
                after: after.clone(),
                free: free.into_iter().collect(),
                avoided: avoided.into_iter().collect(),
            });
            current = Some(after);
        }
        let checked = CoercionObligationChecker::default().check(
            &self.symbols,
            &TypeFactTable::new(),
            inputs.clone(),
            obligations.clone(),
        );
        self.replay_witness_goals(&steps, &inputs, &obligations, &checked, &inference)?;
        Ok((steps, inputs, obligations, checked, inference))
    }

    fn replay_witness_goals(
        &self,
        steps: &[SourceProofGoalStep],
        inputs: &[crate::type_checker::CoercionInput],
        obligations: &[crate::type_checker::InitialObligationInput],
        checked: &crate::type_checker::CoercionCheckingOutput,
        inference: &crate::type_checker::TermFormulaInferenceOutput,
    ) -> Result<(), SourceProofLocalDeclarationError> {
        use crate::{
            type_checker::{
                CoercionObligationChecker, FormulaStatus, NormalizedTypeStatus, TermFormulaChecker,
                TermStatus, TypeHeadInput, TypeHeadRef,
            },
            typed_ast::{
                CoercionStatus, FactStatus, InitialObligationStatus, TypeEntryActual,
                TypeFactTable, TypeStatus,
            },
        };
        let invalid = || SourceProofLocalDeclarationError::DependencyMismatch;
        let (events, goals, terms, formulas, checkpoints) = self.witness_goal_inputs()?;
        if steps.len() != events.len()
            || !inference.diagnostics().is_empty()
            || !inference.facts().is_empty()
            || !inference.candidate_sets().is_empty()
            || inference.terms().len() != terms.len()
            || inference.formulas().len() != formulas.len()
            || *inference
                != TermFormulaChecker::default()
                    .try_infer(&self.symbols, &self.binding_env, terms.clone(), formulas)
                    .map_err(|_| invalid())?
            || inference.formulas().iter().any(|(_, f)| {
                f.status != FormulaStatus::Checked
                    || f.recovery != NodeRecoveryState::Normal
                    || !f.facts.is_empty()
                    || !f.deferred.is_empty()
            })
            || inference
                .type_entries()
                .iter()
                .any(|(_, t)| t.status != TypeStatus::Known)
        {
            return Err(invalid());
        }
        for input in &terms {
            let term = inference
                .terms()
                .iter()
                .map(|(_, t)| t)
                .find(|t| t.site == input.site)
                .ok_or_else(invalid)?;
            let entry = inference
                .type_entries()
                .get(term.type_entry)
                .ok_or_else(invalid)?;
            let TypeEntryActual::Known(id) = entry.actual else {
                return Err(invalid());
            };
            let ty = inference.normalized_types().get(id).ok_or_else(invalid)?;
            if term.status != TermStatus::Inferred
                || term.reference != input.reference
                || term.context != input.context
                || !term.deferred.is_empty()
                || ty.status != NormalizedTypeStatus::Known
                || !ty.args.is_empty()
                || !ty.attributes.positive().is_empty()
                || !ty.attributes.negative().is_empty()
                || !matches!(
                    (
                        &input.result_type.as_ref().ok_or_else(invalid)?.head,
                        &ty.head
                    ),
                    (TypeHeadInput::BuiltinSet, TypeHeadRef::BuiltinSet)
                        | (TypeHeadInput::BuiltinObject, TypeHeadRef::BuiltinObject)
                )
            {
                return Err(invalid());
            }
        }
        let mut current = None;
        let mut frontier = BTreeSet::new();
        let mut expected_inputs = Vec::new();
        let mut expected_obligations = Vec::new();
        for (step, (owner, context, site, actual)) in steps.iter().zip(events) {
            if step.owner != owner || step.context != context || step.site != site {
                return Err(invalid());
            }
            let row = self
                .original
                .nodes()
                .node(site.node())
                .and_then(|n| n.resolved_node)
                .and_then(|id| {
                    self.original
                        .source_node_entries()?
                        .find(|r| r.site() == id)
                })
                .ok_or_else(invalid)?;
            if site == owner {
                current = Some(goals.get(&owner).ok_or_else(invalid)?.clone());
                frontier.clear();
            }
            let before = current.as_ref().ok_or_else(invalid)?;
            if step.before != *before {
                return Err(invalid());
            }
            let mut avoided = BTreeSet::new();
            if let Some(actual) = actual {
                let (formal, radix, body) = match (before, row.kind()) {
                    (
                        SourceProofGoal::Universal {
                            binding,
                            radix,
                            body,
                            ..
                        },
                        SurfaceNodeKind::Token(_),
                    ) => {
                        if !frontier.insert(actual) {
                            return Err(invalid());
                        }
                        let a = self.witness_goal_guard(actual, &site, "let-guard")?;
                        let b = self.witness_goal_guard(*binding, &site, "formal-guard")?;
                        if a.head != b.head {
                            return Err(invalid());
                        }
                        (*binding, *radix, body)
                    }
                    (
                        SourceProofGoal::Existential {
                            binding,
                            radix,
                            body,
                            ..
                        },
                        SurfaceNodeKind::Witness,
                    ) => {
                        if !frontier.contains(&actual) {
                            return Err(invalid());
                        }
                        let (input, obligation) =
                            self.witness_goal_guard_request(&owner, &site, *binding, actual)?;
                        expected_inputs.extend(input);
                        expected_obligations.extend(obligation);
                        (*binding, *radix, body)
                    }
                    _ => return Err(invalid()),
                };
                let guard = self.witness_goal_guard(formal, &site, "matched-guard")?;
                if !matches!(
                    (&guard.head, radix),
                    (
                        TypeHeadInput::BuiltinSet,
                        mizar_resolve::names::SourceVariableTypeRadix::Set
                    ) | (
                        TypeHeadInput::BuiltinObject,
                        mizar_resolve::names::SourceVariableTypeRadix::Object
                    )
                ) {
                    return Err(invalid());
                }
                body.binding_sets(&mut Vec::new(), &mut BTreeSet::new(), &mut avoided);
                if avoided.contains(&actual)
                    || step.matched != Some((formal, actual))
                    || !step.after.replays_substitution(body, formal, actual)
                {
                    return Err(invalid());
                }
            } else {
                if step.matched.is_some() || step.before != step.after {
                    return Err(invalid());
                }
                if row.kind() == &SurfaceNodeKind::ReconsiderItem {
                    if !checkpoints
                        .get(&site)
                        .is_some_and(|rhs| frontier.contains(rhs))
                    {
                        return Err(invalid());
                    }
                } else if site != owner
                    && (row.kind() != &SurfaceNodeKind::ConclusionStatement
                        || goals
                            .get(&site)
                            .is_some_and(|goal| !before.alpha_matches(goal, &mut Vec::new())))
                {
                    return Err(invalid());
                }
            }
            self.witness_goal_formation(&step.after, &frontier, &mut Vec::new())?;
            let mut free = BTreeSet::new();
            step.after
                .binding_sets(&mut Vec::new(), &mut free, &mut BTreeSet::new());
            if step.free != free.into_iter().collect::<Vec<_>>()
                || step.avoided != avoided.into_iter().collect::<Vec<_>>()
            {
                return Err(invalid());
            }
            current = Some(step.after.clone());
        }
        if inputs != expected_inputs
            || obligations != expected_obligations
            || !checked.diagnostics().is_empty()
            || *checked
                != CoercionObligationChecker::default().check(
                    &self.symbols,
                    &TypeFactTable::new(),
                    expected_inputs,
                    expected_obligations,
                )
            || checked.coercions().len() != inputs.len()
            || checked.initial_obligations().len() != obligations.len()
            || checked
                .coercions()
                .iter()
                .any(|(_, c)| c.status != CoercionStatus::Candidate || c.obligation.is_some())
            || checked.initial_obligations().iter().any(|(_, o)| {
                o.status != InitialObligationStatus::Pending || !o.assumptions.is_empty()
            })
            || checked.facts().len() != inputs.len() + obligations.len()
            || checked.facts().iter().any(|(_, f)| {
                !matches!(f.status, FactStatus::Known | FactStatus::PendingObligation)
            })
        {
            return Err(invalid());
        }
        Ok(())
    }

    // Rationale: reuse existing request/check/query types without adding a stored conversion DTO.
    #[allow(clippy::type_complexity)]
    pub fn check_conversions(
        &self,
    ) -> Result<
        (
            Vec<crate::type_checker::CoercionInput>,
            crate::type_checker::CoercionCheckingOutput,
            Vec<crate::type_checker::TypeFactQueryOutput>,
        ),
        SourceProofLocalDeclarationError,
    > {
        self.validate_complete_installation(
            self.original.source_id(),
            self.original.module_id(),
            true,
        )?;
        let original = self.original.as_ref();
        let native = self.native.as_ref();
        let symbols = self.symbols.as_ref();
        let scope = self.scope.as_ref();

        use crate::type_checker::{
            AttributeInput, AttributePolarity, CoercionEvidence, CoercionInput,
            CoercionJustification, CoercionObligationChecker, CoercionRequestKind,
            NormalizedTypeStatus, TypeExpressionInput, TypeFactQuery, TypeFactQueryEngine,
            TypeHeadInput,
        };
        use crate::typed_ast::{
            CoercionStatus, Polarity, TypeFactTable, TypePredicateRef, TypeRole, TypeStatus,
            TypedSiteRef,
        };
        use SurfaceNodeKind as K;
        use mizar_resolve::env::{ContributionKind, DefinitionKind, NamespacePath, SymbolKind};
        use mizar_resolve::names::{
            LocalTermScope, SourceVariableBindingId, SourceVariableReferenceKind,
        };
        let invalid = || SourceProofLocalDeclarationError::DependencyMismatch;
        let rows = original.source_nodes().ok_or_else(invalid)?;
        let row_for = |site| {
            rows.iter()
                .find(|row| row.site() == site)
                .ok_or_else(invalid)
        };
        let mut typed_nodes = BTreeMap::new();
        for (id, node) in original.nodes().iter() {
            let site = node.resolved_node.ok_or_else(invalid)?;
            let row = row_for(site)?;
            if node.anchor != mizar_session::SourceAnchor::Range(row.range())
                || node.recovery != NodeRecoveryState::Normal
                || typed_nodes.insert(site, id).is_some()
            {
                return Err(invalid());
            }
        }
        let (source_scopes, _) =
            mizar_resolve::symbols::source_proof_scope_maps(native, symbols, scope)
                .map_err(|_| invalid())?;
        let scopes = source_scopes
            .into_iter()
            .map(|(node, local)| {
                native
                    .resolved_node_for(node)
                    .map(|site| (site, local))
                    .ok_or_else(invalid)
            })
            .collect::<Result<BTreeMap<_, _>, _>>()?;
        let references = scope
            .references()
            .iter()
            .map(|reference| {
                native
                    .resolved_node_for(reference.node())
                    .map(|site| (site, reference))
                    .ok_or_else(invalid)
            })
            .collect::<Result<BTreeMap<_, _>, _>>()?;
        let names = scope
            .bindings()
            .iter()
            .map(|binding| {
                native
                    .resolved_node_for(binding.node())
                    .map(|site| (site, binding))
                    .ok_or_else(invalid)
            })
            .collect::<Result<BTreeMap<_, _>, _>>()?;
        let namespace = NamespacePath::new(original.module_id().path().as_str());
        let type_input = |ty: &SourceNode, request: ResolvedNodeId, role: &str| {
            let children = source_children(rows, ty.site()).collect::<Vec<_>>();
            let (head, chain) = match children.as_slice() {
                [head] => (*head, None),
                [chain, head] if chain.kind() == &K::AttributeChain => (*head, Some(*chain)),
                _ => return Err(invalid()),
            };
            let head_parts = source_children(rows, head.site()).collect::<Vec<_>>();
            let [word] = head_parts.as_slice() else {
                return Err(invalid());
            };
            if ty.kind() != &K::TypeExpression || head.kind() != &K::TypeHead {
                return Err(invalid());
            }
            let head = match word.kind() {
                K::Token(token) if token.kind == SurfaceTokenKind::ReservedWord => {
                    match token.text.as_ref() {
                        "set" => TypeHeadInput::BuiltinSet,
                        "object" => TypeHeadInput::BuiltinObject,
                        _ => return Err(invalid()),
                    }
                }
                _ => return Err(invalid()),
            };
            let mut attributes = Vec::new();
            if let Some(chain) = chain {
                for attribute in source_children(rows, chain.site()) {
                    let parts = source_children(rows, attribute.site()).collect::<Vec<_>>();
                    let [qualified] = parts.as_slice() else {
                        return Err(invalid());
                    };
                    let parts = source_children(rows, qualified.site()).collect::<Vec<_>>();
                    let [segment] = parts.as_slice() else {
                        return Err(invalid());
                    };
                    let parts = source_children(rows, segment.site()).collect::<Vec<_>>();
                    let [token] = parts.as_slice() else {
                        return Err(invalid());
                    };
                    if attribute.kind() != &K::AttributeRef
                        || qualified.kind() != &K::QualifiedSymbol
                        || segment.kind() != &K::PathSegment
                        || !matches!(token.kind(), K::Token(t) if matches!(t.kind,
                        SurfaceTokenKind::Identifier | SurfaceTokenKind::UserSymbol))
                    {
                        return Err(invalid());
                    }
                    let candidates = symbols
                        .symbols()
                        .visible_candidates(&namespace, token.spelling());
                    let [entry] = candidates.as_slice() else {
                        return Err(invalid());
                    };
                    let definition = symbols
                        .definitions()
                        .by_symbol(entry.symbol())
                        .ok_or_else(invalid)?;
                    let contribution = symbols
                        .contributions()
                        .get(entry.contribution())
                        .ok_or_else(invalid)?;
                    let origin = entry.origin();
                    let mizar_session::SourceAnchor::Range(declaration) = origin.anchor() else {
                        return Err(invalid());
                    };
                    if entry.kind() != SymbolKind::Attribute
                        || entry.symbol().module() != original.module_id()
                        || origin.source_id() != original.source_id()
                        || origin.module_id() != original.module_id()
                        || origin.import_edge().is_some()
                        || origin.is_recovered()
                        || declaration.end > token.range().start
                        || definition.kind() != DefinitionKind::Attribute
                        || definition.arity() != Some(0)
                        || definition.origin() != origin
                        || definition.contribution() != entry.contribution()
                        || definition.conflict().is_some()
                        || contribution.module() != original.module_id()
                        || !matches!(contribution.kind(), ContributionKind::LocalSource { source_id }
                        if *source_id == original.source_id())
                        || !contribution.effects().symbols().contains(entry.symbol())
                        || !contribution
                            .effects()
                            .definitions()
                            .contains(&definition.id())
                        || !rows.iter().any(|row| {
                            row.kind() == &K::AttributeDefinition && row.range() == *declaration
                        })
                    {
                        return Err(invalid());
                    }
                    attributes.push(AttributeInput::new(
                        entry.symbol().clone(),
                        AttributePolarity::Positive,
                        attribute.range(),
                        token.spelling(),
                    ));
                }
                if attributes.is_empty() {
                    return Err(invalid());
                }
            }
            Ok(TypeExpressionInput::new(
                TypedSiteRef::Role {
                    node: *typed_nodes.get(&ty.site()).ok_or_else(invalid)?,
                    role: TypeRole::new(format!(
                        "conversion.{}.{}",
                        typed_nodes.get(&request).ok_or_else(invalid)?.index(),
                        role
                    )),
                },
                ty.range(),
                ty.spelling(),
                head,
            )
            .with_attributes(attributes))
        };
        let supports = |from: &TypeExpressionInput, to: &TypeExpressionInput| {
            from.args.is_empty()
                && to.args.is_empty()
                && from.attributes.is_empty()
                && to.attributes.is_empty()
                && matches!(
                    (&from.head, &to.head),
                    (
                        TypeHeadInput::BuiltinSet,
                        TypeHeadInput::BuiltinSet | TypeHeadInput::BuiltinObject
                    ) | (TypeHeadInput::BuiltinObject, TypeHeadInput::BuiltinObject)
                )
        };
        // Originals are immutable; qualified views never erase source type knowledge.
        let mut originals = BTreeMap::new();
        for binding in scope.bindings().iter().filter(|binding| {
            matches!(
                binding.kind(),
                SourceVariableBindingKind::Let
                    | SourceVariableBindingKind::Reserve
                    | SourceVariableBindingKind::Quantifier
            )
        }) {
            if binding
                .declared_type()
                .is_none_or(|ty| !ty.attributes().is_empty())
            {
                continue;
            }
            let name = row_for(
                native
                    .resolved_node_for(binding.node())
                    .ok_or_else(invalid)?,
            )?;
            let segment = row_for(name.parent().ok_or_else(invalid)?)?;
            if let Some(ty) = source_unique_child(rows, segment, K::TypeExpression) {
                let input = type_input(ty, name.site(), "original")?;
                if supports(&input, &input) {
                    originals.insert(binding.id(), ty.site());
                }
            }
        }
        let mut views: BTreeMap<
            SourceVariableBindingId,
            Vec<(usize, LocalTermScope, ResolvedNodeId)>,
        > = BTreeMap::new();
        let mut expression_views = BTreeMap::new();
        let mut events = rows
            .iter()
            .filter(|row| matches!(row.kind(), K::QuaExpression | K::ReconsiderItem))
            .collect::<Vec<_>>();
        // A contained qua is checked before the item or outer qua that consumes it.
        events.sort_by_key(|row| {
            (
                row.range().end,
                std::cmp::Reverse(row.range().start),
                std::cmp::Reverse(row.preorder_ordinal()),
            )
        });
        let checker = CoercionObligationChecker::default();
        let no_facts = TypeFactTable::new();
        let mut requests = Vec::new();
        let mut query_sites = Vec::new();
        let mut statuses = BTreeMap::new();
        for event in events {
            let qua = event.kind() == &K::QuaExpression;
            let (value, target, statement, declared, explicit) = if qua {
                let [base, target] = source_qua_roles(rows, event).ok_or_else(invalid)?;
                (base, target, None, None, true)
            } else {
                let statement = row_for(event.parent().ok_or_else(invalid)?)?;
                let (items, target, tail) =
                    source_reconsider_statement_roles(rows, statement).ok_or_else(invalid)?;
                if !items.iter().any(|item| item.site() == event.site()) {
                    return Err(invalid());
                }
                let (name, value) =
                    source_reconsider_item_roles(rows, event).ok_or_else(invalid)?;
                let declared = if value.is_some() {
                    Some(names.get(&name.site()).ok_or_else(invalid)?.id())
                } else {
                    None
                };
                (
                    value.unwrap_or(name),
                    target,
                    Some(statement),
                    declared,
                    tail.is_some(),
                )
            };
            // Resolve supported value graphs through sealed occurrence identities.
            let (original_type, candidates) = {
                let mut current = value;
                loop {
                    match current.kind() {
                        K::TermExpression => {
                            let children =
                                source_children(rows, current.site()).collect::<Vec<_>>();
                            let [child] = children.as_slice() else {
                                return Err(invalid());
                            };
                            current = child;
                        }
                        K::ParenthesizedTerm => {
                            let children =
                                source_children(rows, current.site()).collect::<Vec<_>>();
                            if children.len() != 3
                                || !source_inline_token(&children, 0, "(")
                                || !source_inline_token(&children, 2, ")")
                                || children[1].kind() != &K::TermExpression
                            {
                                return Err(invalid());
                            }
                            current = children[1];
                        }
                        K::QuaExpression => {
                            let target =
                                *expression_views.get(&current.site()).ok_or_else(invalid)?;
                            break (target, Vec::new());
                        }
                        K::TermReference | K::Token(_) => {
                            let reference = references.get(&current.site()).ok_or_else(invalid)?;
                            if reference.kind() != SourceVariableReferenceKind::Term {
                                return Err(invalid());
                            }
                            let binding = reference.binding();
                            let original_type = *originals.get(&binding).ok_or_else(invalid)?;
                            let local = scopes.get(&event.site()).ok_or_else(invalid)?;
                            let available = views
                                .get(&binding)
                                .into_iter()
                                .flatten()
                                .rev()
                                .filter(|(order, scope, _)| {
                                    *order <= event.range().start
                                        && local.path().starts_with(scope.path())
                                })
                                .map(|(_, _, ty)| *ty)
                                .collect::<Vec<_>>();
                            break (original_type, available);
                        }
                        _ => return Err(invalid()),
                    }
                }
            };
            let to_type = type_input(target, event.site(), "to")?;
            let mut from_type = type_input(row_for(original_type)?, event.site(), "from")?;
            for candidate in candidates {
                let input = type_input(row_for(candidate)?, event.site(), "from")?;
                if qua || supports(&input, &to_type) {
                    from_type = input;
                    break;
                }
            }
            let intrinsic = supports(&from_type, &to_type);
            let site = TypedSiteRef::Node(*typed_nodes.get(&event.site()).ok_or_else(invalid)?);
            let mut input = CoercionInput::new(
                site.clone(),
                event.range(),
                if qua {
                    CoercionRequestKind::SourceQua
                } else if intrinsic {
                    CoercionRequestKind::Widening
                } else {
                    CoercionRequestKind::Narrowing
                },
                to_type,
            )
            .with_from_type(from_type)
            .with_justification(if explicit {
                CoercionJustification::Explicit
            } else {
                CoercionJustification::Omitted
            });
            if intrinsic {
                input = input.with_evidence(if qua {
                    CoercionEvidence::StaticUpcast
                } else {
                    CoercionEvidence::BuiltinRadix
                });
            }
            let checked = checker.check(symbols, &no_facts, [input.clone()], []);
            let mut entries = checked.coercions().iter().map(|(_, row)| row);
            let result = entries.next().ok_or_else(invalid)?;
            if entries.next().is_some()
                || result.site != site
                || statuses.insert(site.clone(), result.status).is_some()
            {
                return Err(invalid());
            }
            let qualified = intrinsic
                && result.status == CoercionStatus::Candidate
                && result
                    .from
                    .and_then(|id| checked.normalized_types().get(id))
                    .is_some_and(|ty| ty.status == NormalizedTypeStatus::Known)
                && checked
                    .normalized_types()
                    .get(result.to)
                    .is_some_and(|ty| ty.status == NormalizedTypeStatus::Known)
                && [
                    &input.from_type.as_ref().ok_or_else(invalid)?.site,
                    &input.to_type.site,
                ]
                .iter()
                .all(|site| {
                    checked.type_entries().iter().any(|(_, entry)| {
                        &entry.owner == *site && entry.status == TypeStatus::Known
                    })
                });
            if qualified {
                if qua {
                    expression_views.insert(event.site(), target.site());
                } else {
                    let binding = if let Some(declared) = declared {
                        originals.insert(declared, original_type);
                        declared
                    } else {
                        references.get(&value.site()).ok_or_else(invalid)?.binding()
                    };
                    views.entry(binding).or_default().push((
                        event.range().end,
                        scopes.get(&event.site()).ok_or_else(invalid)?.clone(),
                        target.site(),
                    ));
                }
            }
            if statement.is_some() {
                query_sites.push((site, event.range()));
            }
            requests.push(input);
        }
        let output = checker.check(symbols, &no_facts, requests.clone(), []);
        if output.coercions().len() != requests.len()
            || output
                .coercions()
                .iter()
                .any(|(_, row)| statuses.get(&row.site) != Some(&row.status))
        {
            return Err(invalid());
        }
        // Core output sorting can differ from dependency order; subject joins are exact.
        let queries = query_sites
            .into_iter()
            .map(|(site, range)| {
                TypeFactQueryEngine::new(output.facts()).query(TypeFactQuery::new(
                    site,
                    TypePredicateRef::new("coercion.widening.builtin_radix"),
                    Polarity::Positive,
                    range,
                ))
            })
            .collect();
        Ok((requests, output, queries))
    }

    // Rationale: existing sites and resolver output avoid another retained proof graph/DTO.
    #[allow(clippy::type_complexity)]
    pub fn check_proof_intents(
        &self,
    ) -> Result<
        (
            Vec<(
                crate::typed_ast::TypedSiteRef,
                Option<crate::typed_ast::TypedSiteRef>,
                SourceProofIntentStatus,
            )>,
            mizar_resolve::labels::LabelResolutionResult,
        ),
        SourceProofLocalDeclarationError,
    > {
        use crate::typed_ast::{CoercionStatus, TypedSiteRef};
        use SurfaceNodeKind as K;
        use mizar_resolve::{
            env::{ContributionKind, NamespacePath},
            labels::{LabelResolver, ProofLabelSourceCollector},
            resolved_ast::{LabelKind, LabelResolution},
        };
        let invalid = || SourceProofLocalDeclarationError::DependencyMismatch;
        self.validate_complete_installation(
            self.original.source_id(),
            self.original.module_id(),
            true,
        )?;
        let original = self.original.as_ref();
        let rows = original.source_nodes().ok_or_else(invalid)?;
        let ast = mizar_resolve::symbols::validated_source_symbol_ast(&self.native, &self.symbols)
            .map_err(|_| invalid())?;
        let contribution = self.symbols.contributions().iter().find(|entry|
            matches!(entry.kind(), ContributionKind::LocalSource { source_id } if *source_id == original.source_id())
                && entry.module() == original.module_id()).ok_or_else(invalid)?;
        let namespace = NamespacePath::new(original.module_id().path().as_str());
        let collection = ProofLabelSourceCollector::new(
            &ast,
            original.module_id(),
            namespace.clone(),
            contribution.id(),
            &self.native,
        )
        .and_then(|collector| collector.collect_with_native_proof_organization(&self.symbols))
        .map_err(|_| invalid())?;
        let labels = LabelResolver::new(collection.projections()).resolve(
            original.module_id(),
            &namespace,
            collection.references(),
        );
        if !labels.diagnostics().is_empty() {
            return Err(invalid());
        }
        let row_for = |site| {
            rows.iter()
                .find(|row| row.site() == site)
                .ok_or_else(invalid)
        };
        let typed = |row: &SourceNode| {
            let mut nodes = original
                .nodes()
                .iter()
                .filter(|(_, node)| node.resolved_node == Some(row.site()));
            let (id, node) = nodes.next().ok_or_else(invalid)?;
            if nodes.next().is_some()
                || node.anchor != mizar_session::SourceAnchor::Range(row.range())
                || node.recovery != NodeRecoveryState::Normal
            {
                return Err(invalid());
            }
            Ok(TypedSiteRef::Node(id))
        };
        let mut expected_labels = Vec::new();
        let mut references = BTreeSet::new();
        let mut intents = Vec::new();
        let mut blocked = BTreeSet::new();
        for row in rows.iter() {
            let children = source_children(rows, row.site()).collect::<Vec<_>>();
            if matches!(
                row.kind(),
                K::TheoremItem
                    | K::ProofBlock
                    | K::AssumptionStatement
                    | K::CompactStatement
                    | K::ConclusionStatement
                    | K::TakeStatement
                    | K::Proposition
                    | K::ConditionList
                    | K::JustificationClause
                    | K::ReferenceList
                    | K::Reference
            ) && source_contiguous_child_count(children.iter().copied()) != Some(children.len())
            {
                return Err(invalid());
            }
            let theorem = {
                let mut ancestor = Some(row);
                let mut found = None;
                while let Some(current) = ancestor {
                    if current.kind() == &K::TheoremItem {
                        found = Some(current);
                        break;
                    }
                    ancestor = current.parent().map(row_for).transpose()?;
                }
                found
            };
            match row.kind() {
                K::LemmaItem => return Err(invalid()),
                K::TheoremItem => {
                    let start = usize::from(
                        matches!(children.first().map(|row| row.kind()), Some(K::Token(t))
                        if t.kind == SurfaceTokenKind::ReservedWord && matches!(t.text.as_ref(), "open" | "assumed")),
                    );
                    if !matches!(children.get(start).map(|row| row.kind()), Some(K::Token(t))
                        if t.kind == SurfaceTokenKind::ReservedWord && t.text.as_ref() == "theorem")
                        || !matches!(children.get(start + 1).map(|row| row.kind()), Some(K::Token(t)) if t.kind == SurfaceTokenKind::Identifier)
                        || !source_inline_token(&children, start + 2, ":")
                        || children
                            .get(start + 3)
                            .is_none_or(|r| r.kind() != &K::FormulaExpression)
                        || !source_inline_token(&children, children.len().saturating_sub(1), ";")
                        || !(children.len() == start + 5
                            || children.len() == start + 6
                                && matches!(
                                    children[start + 4].kind(),
                                    K::JustificationClause | K::ProofBlock
                                ))
                    {
                        return Err(invalid());
                    }
                    expected_labels.push((
                        children[start + 1].range(),
                        children[start + 1].spelling().to_owned(),
                        LabelKind::Theorem,
                    ));
                }
                K::GivenStatement => {
                    source_given_roles(rows, row).ok_or_else(invalid)?;
                }
                K::ConsiderStatement => {
                    source_consider_roles(rows, row).ok_or_else(invalid)?;
                }
                K::NowStatement => {
                    source_now_roles(rows, row).ok_or_else(invalid)?;
                }
                K::HerebyStatement => {
                    source_hereby_roles(rows, row).ok_or_else(invalid)?;
                }
                K::CaseItem => {
                    source_case_item_roles(rows, row).ok_or_else(invalid)?;
                }
                K::SupposeItem => {
                    source_suppose_item_roles(rows, row).ok_or_else(invalid)?;
                }
                K::CaseReasoningStatement => {
                    source_case_reasoning_roles(rows, row).ok_or_else(invalid)?;
                }
                K::SetStatement => {
                    source_set_roles(rows, row).ok_or_else(invalid)?;
                }
                K::StatementItem => {
                    source_statement_item_roles(rows, row).ok_or_else(invalid)?;
                }
                K::IterativeEqualityStatement => {
                    source_iterative_equality_roles(rows, row).ok_or_else(invalid)?;
                }
                K::IterativeEqualityStep => {
                    source_iterative_equality_step_roles(rows, row).ok_or_else(invalid)?;
                }
                K::LetStatement => {
                    source_proof_introduction_roles(rows, row, "let", false, false)
                        .ok_or_else(invalid)?;
                }
                K::AnnotatedStatement => {
                    if children.len() < 2
                        || source_contiguous_child_count(children.iter().copied())
                            != Some(children.len())
                        || children[..children.len() - 1]
                            .iter()
                            .any(|r| !matches!(r.kind(), K::Annotation | K::LibraryAnnotation))
                        || !source_proof_statement_kind(children[children.len() - 1].kind())
                        || matches!(
                            children[children.len() - 1].kind(),
                            K::AnnotatedStatement | K::StandaloneDiagnosticAnnotation
                        )
                    {
                        return Err(invalid());
                    }
                }
                K::TakeStatement => {
                    if children.len() < 3
                        || children.len().is_multiple_of(2)
                        || !source_inline_token(&children, 0, "take")
                        || !source_inline_token(&children, children.len() - 1, ";")
                        || (1..children.len() - 1).any(|i| {
                            if !i.is_multiple_of(2) {
                                children[i].kind() != &K::Witness
                            } else {
                                !source_inline_token(&children, i, ",")
                            }
                        })
                    {
                        return Err(invalid());
                    }
                }
                K::ProofBlock => {
                    if children.len() < 2
                        || !source_inline_token(&children, 0, "proof")
                        || !source_inline_token(&children, children.len() - 1, "end")
                        || children[1..children.len() - 1]
                            .iter()
                            .any(|r| !source_proof_statement_kind(r.kind()))
                    {
                        return Err(invalid());
                    }
                    let parent = row_for(row.parent().ok_or_else(invalid)?)?;
                    if !matches!(
                        parent.kind(),
                        K::TheoremItem
                            | K::CompactStatement
                            | K::ConclusionStatement
                            | K::ReconsiderStatement
                            | K::CorrectnessCondition
                    ) {
                        return Err(invalid());
                    }
                    if parent.kind() == &K::CorrectnessCondition {
                        let registration = row_for(parent.parent().ok_or_else(invalid)?)?;
                        let keyword = match registration.kind() {
                            K::ExistentialRegistration => "existence",
                            K::ConditionalRegistration | K::FunctorialRegistration => "coherence",
                            K::ReductionRegistration => "reducibility",
                            _ => return Err(invalid()),
                        };
                        let condition = source_proof_frame(rows, parent).ok_or_else(invalid)?;
                        if condition.len() != 3
                            || !matches!(condition[0].kind(), K::Token(t)
                            if t.kind == SurfaceTokenKind::ReservedWord && t.text.as_ref() == keyword)
                            || condition[1].site() != row.site()
                            || !source_inline_token(&condition, 2, ";")
                            || source_children(rows, registration.site())
                                .filter(|r| r.kind() == &K::CorrectnessCondition)
                                .count()
                                != 1
                        {
                            return Err(invalid());
                        }
                        let mut container = row_for(registration.parent().ok_or_else(invalid)?)?;
                        if container.kind() == &K::AnnotatedRegistrationContent {
                            let parts = source_proof_frame(rows, container).ok_or_else(invalid)?;
                            if parts.len() < 2
                                || parts.last().is_none_or(|r| r.site() != registration.site())
                                || parts[..parts.len() - 1].iter().any(|r| {
                                    !matches!(r.kind(), K::Annotation | K::LibraryAnnotation)
                                })
                            {
                                return Err(invalid());
                            }
                            container = row_for(container.parent().ok_or_else(invalid)?)?;
                        }
                        if container.kind() != &K::RegistrationBlockItem {
                            return Err(invalid());
                        }
                    }
                }
                K::CompactStatement | K::AssumptionStatement | K::ConclusionStatement => {
                    let start = usize::from(row.kind() != &K::CompactStatement);
                    if start == 1
                        && !source_inline_token(
                            &children,
                            0,
                            if row.kind() == &K::AssumptionStatement {
                                "assume"
                            } else if source_inline_token(&children, 0, "hence") {
                                "hence"
                            } else {
                                "thus"
                            },
                        )
                    {
                        return Err(invalid());
                    }
                    if children.get(start).is_none_or(|r| {
                        r.kind() != &K::Proposition
                            && !(row.kind() == &K::AssumptionStatement
                                && r.kind() == &K::ConditionList)
                    }) {
                        return Err(invalid());
                    }
                    let mut end = start + 1;
                    if row.kind() != &K::AssumptionStatement
                        && children.get(end).is_some_and(|r| {
                            matches!(r.kind(), K::JustificationClause | K::ProofBlock)
                        })
                    {
                        end += 1;
                    }
                    if !source_inline_token(&children, end, ";") || children.len() != end + 1 {
                        return Err(invalid());
                    }
                }
                K::Proposition => {
                    let start = usize::from(children.len() == 3) * 2;
                    if start == 2
                        && (!matches!(children[0].kind(), K::Token(t) if t.kind == SurfaceTokenKind::Identifier)
                            || !source_inline_token(&children, 1, ":"))
                    {
                        return Err(invalid());
                    }
                    if children.len() != start + 1
                        || children[start].kind() != &K::FormulaExpression
                    {
                        return Err(invalid());
                    }
                    if start == 2 {
                        theorem.ok_or_else(invalid)?;
                        expected_labels.push((
                            children[0].range(),
                            children[0].spelling().to_owned(),
                            LabelKind::ProofStep,
                        ));
                    }
                }
                K::ConditionList => {
                    if children.len() < 2
                        || !children.len().is_multiple_of(2)
                        || !source_inline_token(&children, 0, "that")
                        || children.iter().enumerate().skip(1).any(|(i, r)| {
                            if !i.is_multiple_of(2) {
                                r.kind() != &K::Proposition
                            } else {
                                !source_inline_token(&children, i, "and")
                            }
                        })
                    {
                        return Err(invalid());
                    }
                }
                K::JustificationClause => {
                    if children.len() != 2
                        || !source_inline_token(&children, 0, "by")
                        || children[1].kind() != &K::ReferenceList
                        || theorem.is_none()
                    {
                        return Err(invalid());
                    }
                }
                K::ReferenceList => {
                    if children.is_empty()
                        || children.len().is_multiple_of(2)
                        || children.iter().enumerate().any(|(i, r)| {
                            if i.is_multiple_of(2) {
                                r.kind() != &K::Reference
                            } else {
                                !source_inline_token(&children, i, ",")
                            }
                        })
                        || row_for(row.parent().ok_or_else(invalid)?)?.kind()
                            != &K::JustificationClause
                    {
                        return Err(invalid());
                    }
                }
                K::Reference => {
                    if children.len() != 1
                        || !matches!(children[0].kind(), K::Token(t) if t.kind == SurfaceTokenKind::Identifier)
                        || theorem.is_none()
                        || row_for(row.parent().ok_or_else(invalid)?)?.kind() != &K::ReferenceList
                    {
                        return Err(invalid());
                    }
                    references.insert(row.site());
                }
                _ => {}
            }
            if matches!(row.kind(), K::NowStatement | K::IterativeEqualityStatement)
                && matches!(children.first().map(|r| r.kind()), Some(K::Token(t)) if t.kind == SurfaceTokenKind::Identifier)
            {
                theorem.ok_or_else(invalid)?;
                expected_labels.push((
                    children[0].range(),
                    children[0].spelling().to_owned(),
                    LabelKind::ProofStep,
                ));
            }
            if row.kind() == &K::ThenStatement
                || row.kind() == &K::ConclusionStatement
                    && source_inline_token(&children, 0, "hence")
            {
                if row.kind() == &K::ThenStatement {
                    source_then_roles(rows, row).ok_or_else(invalid)?;
                }
                let mut current = row;
                while matches!(
                    row_for(current.parent().ok_or_else(invalid)?)?.kind(),
                    K::AnnotatedStatement | K::StatementItem | K::ThenStatement
                ) {
                    current = row_for(current.parent().ok_or_else(invalid)?)?;
                }
                let parent = row_for(current.parent().ok_or_else(invalid)?)?;
                let siblings = source_proof_frame(rows, parent).ok_or_else(invalid)?;
                let index = siblings
                    .iter()
                    .position(|r| r.site() == current.site())
                    .ok_or_else(invalid)?;
                let previous = index.checked_sub(1).map(|i| siblings[i]);
                let previous =
                    if matches!(parent.kind(), K::CaseItem | K::SupposeItem) && index == 3 {
                        Some(siblings[1])
                    } else {
                        previous
                    };
                let mut previous = previous;
                while previous.is_some_and(|r| {
                    matches!(
                        r.kind(),
                        K::AnnotatedStatement | K::StatementItem | K::ThenStatement
                    )
                }) {
                    let parts = source_children(rows, previous.ok_or_else(invalid)?.site())
                        .collect::<Vec<_>>();
                    previous = parts.last().copied();
                }
                if !previous.is_some_and(|r| {
                    matches!(
                        r.kind(),
                        K::Proposition
                            | K::ConditionList
                            | K::AssumptionStatement
                            | K::CompactStatement
                            | K::ConclusionStatement
                            | K::ConsiderStatement
                            | K::ReconsiderStatement
                            | K::NowStatement
                            | K::HerebyStatement
                            | K::CaseReasoningStatement
                            | K::IterativeEqualityStatement
                    ) || matches!(r.kind(), K::GivenStatement | K::LetStatement)
                        && source_children(rows, r.site()).any(|c| c.kind() == &K::ConditionList)
                }) {
                    blocked.insert(row.site());
                }
            }
            if matches!(
                row.kind(),
                K::TheoremItem
                    | K::ProofBlock
                    | K::LetStatement
                    | K::AssumptionStatement
                    | K::CompactStatement
                    | K::ConclusionStatement
                    | K::TakeStatement
                    | K::ReconsiderStatement
                    | K::ReconsiderItem
                    | K::GivenStatement
                    | K::ConsiderStatement
                    | K::NowStatement
                    | K::HerebyStatement
                    | K::CaseItem
                    | K::SupposeItem
                    | K::CaseReasoningStatement
                    | K::SetStatement
                    | K::StatementItem
                    | K::ThenStatement
                    | K::IterativeEqualityStatement
                    | K::IterativeEqualityStep
            ) {
                let parent = row_for(row.parent().ok_or_else(invalid)?)?;
                if row.kind() == &K::TheoremItem {
                    if !matches!(parent.kind(), K::ItemList | K::VisibleItem) {
                        return Err(invalid());
                    }
                } else if !matches!(
                    row.kind(),
                    K::StatementItem
                        | K::ProofBlock
                        | K::ReconsiderItem
                        | K::IterativeEqualityStep
                        | K::CaseItem
                        | K::SupposeItem
                ) && !matches!(
                    parent.kind(),
                    K::ProofBlock
                        | K::NowStatement
                        | K::HerebyStatement
                        | K::CaseItem
                        | K::SupposeItem
                        | K::ThenStatement
                        | K::AnnotatedStatement
                        | K::StatementItem
                ) {
                    return Err(invalid());
                }
                let mut tails = children
                    .iter()
                    .filter(|r| matches!(r.kind(), K::JustificationClause | K::ProofBlock));
                let tail = tails.next().copied();
                if tails.next().is_some() {
                    return Err(invalid());
                }
                intents.push((row, typed(row)?, tail.map(typed).transpose()?));
            }
        }
        let actual_labels = collection
            .projections()
            .iter()
            .map(|p| {
                (
                    p.declaration_range(),
                    p.primary_spelling().to_owned(),
                    p.kind(),
                )
            })
            .collect::<Vec<_>>();
        if expected_labels.len() != actual_labels.len()
            || expected_labels.iter().any(|expected| {
                actual_labels
                    .iter()
                    .filter(|actual| *actual == expected)
                    .count()
                    != 1
            })
            || references
                != collection
                    .references()
                    .iter()
                    .map(|r| r.site().node())
                    .collect::<BTreeSet<_>>()
            || references.len() != collection.references().len()
        {
            return Err(invalid());
        }
        for reference in collection.references() {
            let row = row_for(reference.site().node())?;
            if row.kind() != &K::Reference
                || row.range() != reference.site().range()
                || reference.origin().source_id() != original.source_id()
                || reference.origin().module_id() != original.module_id()
                || reference.site().spelling()
                    != source_children(rows, row.site())
                        .next()
                        .ok_or_else(invalid)?
                        .spelling()
            {
                return Err(invalid());
            }
        }
        for id in labels.ids() {
            let entry = labels.table().get(*id).ok_or_else(invalid)?;
            if matches!(entry.resolution(), LabelResolution::Unresolved(_)) {
                blocked.insert(entry.site().node());
            }
        }
        let (conversions, checked, _) = self.check_conversions()?;
        for (_, conversion) in checked.coercions().iter() {
            if matches!(
                conversion.status,
                CoercionStatus::Rejected | CoercionStatus::Blocked
            ) {
                let input = conversions
                    .iter()
                    .find(|r| r.site == conversion.site)
                    .ok_or_else(invalid)?;
                let row = rows
                    .iter()
                    .find(|row| typed(row).as_ref() == Ok(&input.site))
                    .ok_or_else(invalid)?;
                blocked.insert(row.site());
            }
        }
        let mut affected = BTreeSet::new();
        for site in blocked {
            let mut current = Some(row_for(site)?);
            while let Some(row) = current {
                affected.insert(row.site());
                current = row.parent().map(row_for).transpose()?;
            }
        }
        Ok((
            intents
                .into_iter()
                .map(|(row, site, tail)| {
                    (
                        site,
                        tail,
                        if affected.contains(&row.site()) {
                            SourceProofIntentStatus::Blocked
                        } else {
                            SourceProofIntentStatus::Pending
                        },
                    )
                })
                .collect(),
            labels,
        ))
    }

    pub(crate) fn validate_complete_installation(
        &self,
        source: SourceId,
        module: &ModuleId,
        available: bool,
    ) -> Result<(), SourceProofLocalDeclarationError> {
        if self.binding_env.source_id() != source
            || self.binding_env.module_id() != module
            || self.original.source_id() != source
            || self.original.module_id() != module
        {
            return Err(SourceProofLocalDeclarationError::InvalidTransaction);
        }
        let expected = native_proof_local_binding_env(
            &self.original,
            &self.native,
            &self.symbols,
            &self.scope,
        )?;
        if self.binding_env != expected {
            return Err(SourceProofLocalDeclarationError::InvalidBindingEnvironment);
        }
        if !available {
            return Err(SourceProofLocalDeclarationError::InvalidInstallation);
        }
        Ok(())
    }
}

impl SourceProofLocalDeclarationProducer {
    pub fn build_bindings(
        original: &TypedAst,
        native: &SurfaceResolvedArena,
        symbols: &SymbolEnv,
        scope: &ResolvedVariableScope,
    ) -> Result<SourceProofLocalBindingHandoff, SourceProofLocalDeclarationError> {
        let binding_env = native_proof_local_binding_env(original, native, symbols, scope)?;
        Ok(SourceProofLocalBindingHandoff {
            original: Arc::new(original.clone()),
            native: Arc::new(native.clone()),
            symbols: Arc::new(symbols.clone()),
            scope: Box::new(scope.clone()),
            binding_env,
        })
    }
}

fn source_inline_token(children: &[&SourceNode], ordinal: usize, text: &str) -> bool {
    let kind = if matches!(
        text,
        "deffunc"
            | "defpred"
            | "equals"
            | "means"
            | "be"
            | "being"
            | "reconsider"
            | "as"
            | "qua"
            | "by"
            | "proof"
            | "case"
            | "suppose"
            | "per"
            | "cases"
            | "given"
            | "such"
            | "that"
            | "consider"
            | "hereby"
            | "now"
            | "end"
            | "set"
            | "then"
            | "let"
            | "assume"
            | "take"
            | "thus"
            | "hence"
            | "and"
    ) {
        SurfaceTokenKind::ReservedWord
    } else {
        SurfaceTokenKind::ReservedSymbol
    };
    children.get(ordinal).is_some_and(|row|
        matches!(row.kind(), SurfaceNodeKind::Token(token) if token.kind == kind && token.text.as_ref() == text))
}

fn source_inline_parameter_roles<'a>(
    rows: &'a [SourceNode],
    parameter: &SourceNode,
) -> Option<[&'a SourceNode; 2]> {
    let children = source_children(rows, parameter.site()).collect::<Vec<_>>();
    if parameter.kind() != &SurfaceNodeKind::TypedParameter
        || source_contiguous_child_count(children.iter().copied()) != Some(3)
        || !(source_inline_token(&children, 1, "be") || source_inline_token(&children, 1, "being"))
    {
        return None;
    }
    let name = children[0];
    let guard = source_unique_child(rows, parameter, SurfaceNodeKind::TypeExpression)?;
    if !matches!(name.kind(), SurfaceNodeKind::Token(token) if token.kind == SurfaceTokenKind::Identifier)
        || guard.child_ordinal() != 2
    {
        return None;
    }
    Some([name, guard])
}

// Rationale: borrowed original name/formal/guard/body/result roles, with no duplicate stored graph.
#[allow(clippy::type_complexity)]
pub(crate) fn source_inline_functor_roles<'a>(
    rows: &'a [SourceNode],
    owner: &SourceNode,
) -> Option<(
    &'a SourceNode,
    Vec<[&'a SourceNode; 2]>,
    &'a SourceNode,
    Option<&'a SourceNode>,
)> {
    use SurfaceNodeKind as K;
    let children = source_children(rows, owner.site()).collect::<Vec<_>>();
    if owner.kind() != &K::InlineFunctorDefinition
        || source_contiguous_child_count(children.iter().copied()) != Some(children.len())
        || !source_inline_token(&children, 0, "deffunc")
        || !source_inline_token(&children, 2, "(")
    {
        return None;
    }
    let name = *children.get(1)?;
    if !matches!(name.kind(), K::Token(token) if token.kind == SurfaceTokenKind::Identifier) {
        return None;
    }
    let formals = children
        .iter()
        .filter(|row| row.kind() == &K::TypedParameter)
        .enumerate()
        .map(|(index, parameter)| {
            if parameter.child_ordinal() != index * 2 + 3
                || (index > 0 && !source_inline_token(&children, index * 2 + 2, ","))
            {
                return None;
            }
            source_inline_parameter_roles(rows, parameter)
        })
        .collect::<Option<Vec<_>>>()?;
    let close = if formals.is_empty() {
        3
    } else {
        formals.len() * 2 + 2
    };
    let result = source_unique_child(rows, owner, K::TypeExpression)?;
    let body = source_unique_child(rows, owner, K::TermExpression)?;
    if !source_inline_token(&children, close, ")")
        || !source_inline_token(&children, close + 1, "->")
        || result.child_ordinal() != close + 2
        || !source_inline_token(&children, close + 3, "equals")
        || body.child_ordinal() != close + 4
        || !source_inline_token(&children, close + 5, ";")
        || children.len() != close + 6
    {
        return None;
    }
    Some((name, formals, body, Some(result)))
}

// Rationale: same borrowed role shape as the functor rule; no public or stored role DTO.
#[allow(clippy::type_complexity)]
pub(crate) fn source_inline_predicate_roles<'a>(
    rows: &'a [SourceNode],
    owner: &SourceNode,
) -> Option<(
    &'a SourceNode,
    Vec<[&'a SourceNode; 2]>,
    &'a SourceNode,
    Option<&'a SourceNode>,
)> {
    use SurfaceNodeKind as K;
    let children = source_children(rows, owner.site()).collect::<Vec<_>>();
    if owner.kind() != &K::InlinePredicateDefinition
        || source_contiguous_child_count(children.iter().copied()) != Some(children.len())
        || !source_inline_token(&children, 0, "defpred")
        || !source_inline_token(&children, 2, "(")
    {
        return None;
    }
    let name = *children.get(1)?;
    if !matches!(name.kind(), K::Token(token) if token.kind == SurfaceTokenKind::Identifier) {
        return None;
    }
    let formals = children
        .iter()
        .filter(|row| row.kind() == &K::TypedParameter)
        .enumerate()
        .map(|(index, parameter)| {
            if parameter.child_ordinal() != index * 2 + 3
                || (index > 0 && !source_inline_token(&children, index * 2 + 2, ","))
            {
                return None;
            }
            source_inline_parameter_roles(rows, parameter)
        })
        .collect::<Option<Vec<_>>>()?;
    let close = if formals.is_empty() {
        3
    } else {
        formals.len() * 2 + 2
    };
    let body = source_unique_child(rows, owner, K::FormulaExpression)?;
    if !source_inline_token(&children, close, ")")
        || !source_inline_token(&children, close + 1, "means")
        || body.child_ordinal() != close + 2
        || !source_inline_token(&children, close + 3, ";")
        || children.len() != close + 4
    {
        return None;
    }
    Some((name, formals, body, None))
}

pub(crate) fn source_qua_roles<'a>(
    rows: &'a [SourceNode],
    owner: &SourceNode,
) -> Option<[&'a SourceNode; 2]> {
    use SurfaceNodeKind as K;
    let children = source_children(rows, owner.site()).collect::<Vec<_>>();
    if owner.kind() != &K::QuaExpression
        || source_contiguous_child_count(children.iter().copied()) != Some(3)
        || !source_inline_token(&children, 1, "qua")
    {
        return None;
    }
    let base = children[0];
    let target = source_unique_child(rows, owner, K::TypeExpression)?;
    if target.child_ordinal() != 2
        || !matches!(
            base.kind(),
            K::TermExpression
                | K::TermReference
                | K::NumeralTerm
                | K::ItTerm
                | K::ParenthesizedTerm
                | K::ChoiceTerm
                | K::ApplicationTerm
                | K::StructureConstructor
                | K::SetEnumeration
                | K::SetComprehension
                | K::SelectorAccess
                | K::StructureUpdate
                | K::QuaExpression
                | K::PrefixExpression(_)
                | K::PostfixExpression(_)
                | K::InfixExpression(_)
                | K::BinderExpression
        )
    {
        return None;
    }
    Some([base, target])
}

pub(crate) fn source_reconsider_item_roles<'a>(
    rows: &'a [SourceNode],
    item: &SourceNode,
) -> Option<(&'a SourceNode, Option<&'a SourceNode>)> {
    use SurfaceNodeKind as K;
    let children = source_children(rows, item.site()).collect::<Vec<_>>();
    let count = source_contiguous_child_count(children.iter().copied())?;
    if item.kind() != &K::ReconsiderItem
        || !matches!(count, 1 | 3)
        || !rows.iter().any(|parent| {
            Some(parent.site()) == item.parent() && parent.kind() == &K::ReconsiderStatement
        })
    {
        return None;
    }
    let name = children[0];
    if !matches!(name.kind(), K::Token(token) if token.kind == SurfaceTokenKind::Identifier) {
        return None;
    }
    let value = if count == 3 {
        let value = source_unique_child(rows, item, K::TermExpression)?;
        if !source_inline_token(&children, 1, "=") || value.child_ordinal() != 2 {
            return None;
        }
        Some(value)
    } else {
        None
    };
    Some((name, value))
}

// Rationale: borrowed original item/target/tail roles, without a second statement representation.
#[allow(clippy::type_complexity)]
pub(crate) fn source_reconsider_statement_roles<'a>(
    rows: &'a [SourceNode],
    owner: &SourceNode,
) -> Option<(Vec<&'a SourceNode>, &'a SourceNode, Option<&'a SourceNode>)> {
    use SurfaceNodeKind as K;
    let children = source_children(rows, owner.site()).collect::<Vec<_>>();
    if owner.kind() != &K::ReconsiderStatement
        || source_contiguous_child_count(children.iter().copied()) != Some(children.len())
        || !source_inline_token(&children, 0, "reconsider")
    {
        return None;
    }
    let items = children
        .iter()
        .copied()
        .filter(|row| row.kind() == &K::ReconsiderItem)
        .collect::<Vec<_>>();
    if items.is_empty()
        || items.iter().enumerate().any(|(index, item)| {
            item.child_ordinal() != index * 2 + 1
                || (index > 0 && !source_inline_token(&children, index * 2, ","))
        })
    {
        return None;
    }
    let target = source_unique_child(rows, owner, K::TypeExpression)?;
    let mut cursor = items.len() * 2;
    if !source_inline_token(&children, cursor, "as") || target.child_ordinal() != cursor + 1 {
        return None;
    }
    cursor += 2;
    let tail = children
        .get(cursor)
        .copied()
        .filter(|row| matches!(row.kind(), K::JustificationClause | K::ProofBlock));
    if let Some(tail) = tail {
        let parts = source_children(rows, tail.site()).collect::<Vec<_>>();
        if !source_inline_token(
            &parts,
            0,
            if tail.kind() == &K::ProofBlock {
                "proof"
            } else {
                "by"
            },
        ) {
            return None;
        }
        cursor += 1;
    }
    if !source_inline_token(&children, cursor, ";") || children.len() != cursor + 1 {
        return None;
    }
    Some((items, target, tail))
}

fn source_proof_frame<'a>(
    rows: &'a [SourceNode],
    owner: &SourceNode,
) -> Option<Vec<&'a SourceNode>> {
    let children = source_children(rows, owner.site()).collect::<Vec<_>>();
    (source_contiguous_child_count(children.iter().copied()) == Some(children.len()))
        .then_some(children)
}

fn source_proof_statement_kind(kind: &SurfaceNodeKind) -> bool {
    use SurfaceNodeKind as K;
    matches!(
        kind,
        K::LetStatement
            | K::AssumptionStatement
            | K::GivenStatement
            | K::ConsiderStatement
            | K::SetStatement
            | K::TakeStatement
            | K::CompactStatement
            | K::ConclusionStatement
            | K::NowStatement
            | K::HerebyStatement
            | K::ThenStatement
            | K::CaseReasoningStatement
            | K::IterativeEqualityStatement
            | K::ReconsiderStatement
            | K::InlineFunctorDefinition
            | K::InlinePredicateDefinition
            | K::AnnotatedStatement
            | K::StandaloneDiagnosticAnnotation
    )
}

fn source_proof_body<'a>(children: &[&'a SourceNode], start: usize) -> Option<Vec<&'a SourceNode>> {
    let end = children.len().checked_sub(2)?;
    if start > end
        || !source_inline_token(children, end, "end")
        || !source_inline_token(children, end + 1, ";")
        || children[start..end]
            .iter()
            .any(|row| !source_proof_statement_kind(row.kind()))
    {
        return None;
    }
    Some(children[start..end].to_vec())
}

fn source_proof_introduction_roles<'a>(
    rows: &'a [SourceNode],
    owner: &SourceNode,
    opener: &str,
    require_condition: bool,
    require_by: bool,
) -> Option<Vec<&'a SourceNode>> {
    use SurfaceNodeKind as K;
    let children = source_proof_frame(rows, owner)?;
    if !source_inline_token(&children, 0, opener) {
        return None;
    }
    let mut cursor = 1;
    let mut roles = Vec::new();
    loop {
        let segment = *children.get(cursor)?;
        if segment.kind() != &K::QualifiedVariableSegment {
            return None;
        }
        let parts = source_proof_frame(rows, segment)?;
        let mut index = 0;
        loop {
            if !matches!(parts.get(index)?.kind(), K::Token(token) if token.kind == SurfaceTokenKind::Identifier)
            {
                return None;
            }
            index += 1;
            if !source_inline_token(&parts, index, ",") {
                break;
            }
            index += 1;
        }
        if index != parts.len()
            && (!(source_inline_token(&parts, index, "be")
                || source_inline_token(&parts, index, "being"))
                || parts.get(index + 1)?.kind() != &K::TypeExpression
                || parts.len() != index + 2)
        {
            return None;
        }
        roles.push(segment);
        cursor += 1;
        if !source_inline_token(&children, cursor, ",") {
            break;
        }
        cursor += 1;
    }
    let condition = source_inline_token(&children, cursor, "such");
    if require_condition && !condition {
        return None;
    }
    if condition {
        let row = *children.get(cursor + 1)?;
        if row.kind() != &K::ConditionList {
            return None;
        }
        roles.push(row);
        cursor += 2;
    }
    let by = children
        .get(cursor)
        .is_some_and(|row| row.kind() == &K::JustificationClause);
    if require_by && !by || by && owner.kind() == &K::GivenStatement {
        return None;
    }
    if by {
        roles.push(children[cursor]);
        cursor += 1;
    }
    (source_inline_token(&children, cursor, ";") && children.len() == cursor + 1).then_some(roles)
}

pub(crate) fn source_given_roles<'a>(
    rows: &'a [SourceNode],
    owner: &SourceNode,
) -> Option<Vec<&'a SourceNode>> {
    (owner.kind() == &SurfaceNodeKind::GivenStatement).then_some(())?;
    source_proof_introduction_roles(rows, owner, "given", false, false)
}

pub(crate) fn source_consider_roles<'a>(
    rows: &'a [SourceNode],
    owner: &SourceNode,
) -> Option<Vec<&'a SourceNode>> {
    (owner.kind() == &SurfaceNodeKind::ConsiderStatement).then_some(())?;
    source_proof_introduction_roles(rows, owner, "consider", true, true)
}

pub(crate) fn source_now_roles<'a>(
    rows: &'a [SourceNode],
    owner: &SourceNode,
) -> Option<Vec<&'a SourceNode>> {
    let children = source_proof_frame(rows, owner)?;
    let start = if matches!(children.first()?.kind(), SurfaceNodeKind::Token(t) if t.kind == SurfaceTokenKind::Identifier)
    {
        if !source_inline_token(&children, 1, ":") {
            return None;
        }
        2
    } else {
        0
    };
    if owner.kind() != &SurfaceNodeKind::NowStatement
        || !source_inline_token(&children, start, "now")
    {
        return None;
    }
    source_proof_body(&children, start + 1)
}

pub(crate) fn source_hereby_roles<'a>(
    rows: &'a [SourceNode],
    owner: &SourceNode,
) -> Option<Vec<&'a SourceNode>> {
    let children = source_proof_frame(rows, owner)?;
    if owner.kind() != &SurfaceNodeKind::HerebyStatement
        || !source_inline_token(&children, 0, "hereby")
    {
        return None;
    }
    source_proof_body(&children, 1)
}

pub(crate) fn source_case_item_roles<'a>(
    rows: &'a [SourceNode],
    owner: &SourceNode,
) -> Option<Vec<&'a SourceNode>> {
    let children = source_proof_frame(rows, owner)?;
    if owner.kind() != &SurfaceNodeKind::CaseItem
        || !source_inline_token(&children, 0, "case")
        || !matches!(
            children.get(1)?.kind(),
            SurfaceNodeKind::Proposition | SurfaceNodeKind::ConditionList
        )
        || !source_inline_token(&children, 2, ";")
        || !rows.iter().any(|row| {
            Some(row.site()) == owner.parent()
                && row.kind() == &SurfaceNodeKind::CaseReasoningStatement
        })
    {
        return None;
    }
    let mut roles = vec![children[1]];
    roles.extend(source_proof_body(&children, 3)?);
    Some(roles)
}

pub(crate) fn source_suppose_item_roles<'a>(
    rows: &'a [SourceNode],
    owner: &SourceNode,
) -> Option<Vec<&'a SourceNode>> {
    let children = source_proof_frame(rows, owner)?;
    if owner.kind() != &SurfaceNodeKind::SupposeItem
        || !source_inline_token(&children, 0, "suppose")
        || !matches!(
            children.get(1)?.kind(),
            SurfaceNodeKind::Proposition | SurfaceNodeKind::ConditionList
        )
        || !source_inline_token(&children, 2, ";")
        || !rows.iter().any(|row| {
            Some(row.site()) == owner.parent()
                && row.kind() == &SurfaceNodeKind::CaseReasoningStatement
        })
    {
        return None;
    }
    let mut roles = vec![children[1]];
    roles.extend(source_proof_body(&children, 3)?);
    Some(roles)
}

pub(crate) fn source_case_reasoning_roles<'a>(
    rows: &'a [SourceNode],
    owner: &SourceNode,
) -> Option<Vec<&'a SourceNode>> {
    use SurfaceNodeKind as K;
    let children = source_proof_frame(rows, owner)?;
    if owner.kind() != &K::CaseReasoningStatement
        || !source_inline_token(&children, 0, "per")
        || !source_inline_token(&children, 1, "cases")
    {
        return None;
    }
    let mut cursor = 2;
    let mut roles = Vec::new();
    if children
        .get(cursor)
        .is_some_and(|row| row.kind() == &K::JustificationClause)
    {
        roles.push(children[cursor]);
        cursor += 1;
    }
    if !source_inline_token(&children, cursor, ";") {
        return None;
    }
    cursor += 1;
    let kind = children.get(cursor)?.kind();
    if !matches!(kind, K::CaseItem | K::SupposeItem)
        || children[cursor..].iter().any(|row| row.kind() != kind)
    {
        return None;
    }
    roles.extend_from_slice(&children[cursor..]);
    Some(roles)
}

pub(crate) fn source_set_roles<'a>(
    rows: &'a [SourceNode],
    owner: &SourceNode,
) -> Option<Vec<&'a SourceNode>> {
    let children = source_proof_frame(rows, owner)?;
    if owner.kind() != &SurfaceNodeKind::SetStatement
        || !source_inline_token(&children, 0, "set")
        || children.len() < 3
        || children.len().is_multiple_of(2)
        || !source_inline_token(&children, children.len() - 1, ";")
    {
        return None;
    }
    let mut roles = Vec::new();
    for cursor in (1..children.len() - 1).step_by(2) {
        let row = children[cursor];
        if row.kind() != &SurfaceNodeKind::Equating
            || source_equating_roles(rows, row).is_none()
            || (cursor > 1 && !source_inline_token(&children, cursor - 1, ","))
        {
            return None;
        }
        roles.push(row);
    }
    Some(roles)
}

pub(crate) fn source_then_roles<'a>(
    rows: &'a [SourceNode],
    owner: &SourceNode,
) -> Option<&'a SourceNode> {
    use SurfaceNodeKind as K;
    let children = source_proof_frame(rows, owner)?;
    if owner.kind() != &K::ThenStatement
        || children.len() != 2
        || !source_inline_token(&children, 0, "then")
        || !matches!(
            children[1].kind(),
            K::CompactStatement
                | K::ConclusionStatement
                | K::ConsiderStatement
                | K::ReconsiderStatement
                | K::IterativeEqualityStatement
                | K::CaseReasoningStatement
        )
    {
        return None;
    }
    Some(children[1])
}

pub(crate) fn source_statement_item_roles<'a>(
    rows: &'a [SourceNode],
    owner: &SourceNode,
) -> Option<&'a SourceNode> {
    let children = source_proof_frame(rows, owner)?;
    if owner.kind() != &SurfaceNodeKind::StatementItem
        || children.len() != 1
        || !source_proof_statement_kind(children[0].kind())
        || !rows.iter().any(|row| {
            Some(row.site()) == owner.parent() && row.kind() == &SurfaceNodeKind::ItemList
        })
    {
        return None;
    }
    Some(children[0])
}

pub(crate) fn source_iterative_equality_step_roles<'a>(
    rows: &'a [SourceNode],
    owner: &SourceNode,
) -> Option<Vec<&'a SourceNode>> {
    use SurfaceNodeKind as K;
    let children = source_proof_frame(rows, owner)?;
    if owner.kind() != &K::IterativeEqualityStep
        || !matches!(children.len(), 2 | 3)
        || !source_inline_token(&children, 0, ".=")
        || children[1].kind() != &K::TermExpression
        || (children.len() == 3 && children[2].kind() != &K::JustificationClause)
        || !rows.iter().any(|row| {
            Some(row.site()) == owner.parent() && row.kind() == &K::IterativeEqualityStatement
        })
    {
        return None;
    }
    Some(children[1..].to_vec())
}

pub(crate) fn source_iterative_equality_roles<'a>(
    rows: &'a [SourceNode],
    owner: &SourceNode,
) -> Option<Vec<&'a SourceNode>> {
    use SurfaceNodeKind as K;
    let children = source_proof_frame(rows, owner)?;
    let mut cursor = if matches!(children.first()?.kind(), K::Token(t) if t.kind == SurfaceTokenKind::Identifier)
    {
        if !source_inline_token(&children, 1, ":") {
            return None;
        }
        2
    } else {
        0
    };
    if owner.kind() != &K::IterativeEqualityStatement
        || children.get(cursor)?.kind() != &K::TermExpression
        || !source_inline_token(&children, cursor + 1, "=")
        || children.get(cursor + 2)?.kind() != &K::TermExpression
    {
        return None;
    }
    let mut roles = vec![children[cursor], children[cursor + 2]];
    cursor += 3;
    if children
        .get(cursor)
        .is_some_and(|row| row.kind() == &K::JustificationClause)
    {
        roles.push(children[cursor]);
        cursor += 1;
    }
    let start = cursor;
    while children
        .get(cursor)
        .is_some_and(|row| row.kind() == &K::IterativeEqualityStep)
    {
        roles.push(children[cursor]);
        cursor += 1;
    }
    if start == cursor
        || !source_inline_token(&children, cursor, ";")
        || children.len() != cursor + 1
    {
        return None;
    }
    Some(roles)
}

fn native_proof_local_binding_env(
    original: &TypedAst,
    native: &SurfaceResolvedArena,
    symbols: &SymbolEnv,
    scope: &ResolvedVariableScope,
) -> Result<BindingEnv, SourceProofLocalDeclarationError> {
    use SourceVariableBindingKind as B;
    use SurfaceNodeKind as K;
    use mizar_resolve::names::SourceVariableReferenceKind as R;
    let invalid = || SourceProofLocalDeclarationError::DependencyMismatch;
    if native.source_id() != original.source_id()
        || native.module() != original.module_id()
        || symbols.module_id() != original.module_id()
        || scope.source_id() != original.source_id()
        || scope.module_id() != original.module_id()
    {
        return Err(SourceProofLocalDeclarationError::InvalidTransaction);
    }
    let rows = original.source_nodes().ok_or_else(invalid)?;
    if original.source_proof_local_binding().is_some()
        || !original.has_no_other_proof_local_binding_handoff()
        || !original.contexts().is_empty()
        || !original.types().is_empty()
        || !original.facts().is_empty()
        || !original.coercions().is_empty()
        || !original.initial_obligations().is_empty()
        || !original.diagnostics().is_empty()
        || scope.thesis().is_some()
        || !scope.statements().is_empty()
        || rows
            .iter()
            .any(|row| row.recovery() != NodeRecoveryState::Normal)
        || crate::type_checker::source_nodes(native, original)
            .map_err(|_| invalid())?
            .as_slice()
            != rows.as_ref()
    {
        return Err(invalid());
    }
    let (scopes, parents) = mizar_resolve::symbols::source_proof_scope_maps(native, symbols, scope)
        .map_err(|_| invalid())?;
    let shells = mizar_resolve::symbols::source_declaration_shells(native, symbols)
        .map_err(|_| invalid())?;
    let row_for = |id| {
        rows.iter()
            .find(|row| Some(row.site()) == native.resolved_node_for(id))
    };
    let root_scope = scopes
        .iter()
        .find(|(id, _)| native.resolved_node_for(**id) == Some(native.arena().root()))
        .map(|(_, scope)| scope.clone())
        .ok_or_else(invalid)?;
    let mut boundaries = BTreeMap::from([(
        root_scope.clone(),
        (
            BindingContextOwner::Module,
            BindingContextLayer::Module,
            None,
        ),
    )]);
    for (id, local) in &scopes {
        let Some(parent) = parents.get(id) else {
            continue;
        };
        let parent_scope = scopes.get(parent).ok_or_else(invalid)?;
        if local == parent_scope {
            continue;
        }
        let row = row_for(*id).ok_or_else(invalid)?;
        let (owner, layer) = match row.kind() {
            K::ProofBlock => (
                BindingContextOwner::SourceStatement {
                    source_range: row.range(),
                },
                BindingContextLayer::Proof,
            ),
            K::NowStatement | K::HerebyStatement | K::CaseItem | K::SupposeItem => (
                BindingContextOwner::SourceStatement {
                    source_range: row.range(),
                },
                BindingContextLayer::Block,
            ),
            K::QuantifiedFormula(_) => (
                BindingContextOwner::SourceFormula {
                    source_range: row.range(),
                },
                BindingContextLayer::Expression,
            ),
            K::SetComprehension => (
                BindingContextOwner::SourceComprehension {
                    source_range: row.range(),
                },
                BindingContextLayer::Expression,
            ),
            K::InlineFunctorDefinition | K::InlinePredicateDefinition => (
                BindingContextOwner::SourceStatement {
                    source_range: row.range(),
                },
                BindingContextLayer::Declaration,
            ),
            K::DefinitionBlockItem => (
                BindingContextOwner::DeclarationShell(shells.declarations().iter()
                    .find(|shell| native.resolved_node_for(shell.node_id()) == Some(row.site())
                        && shell.kind() == mizar_resolve::declarations::DeclarationShellKind::DefinitionBlock)
                    .ok_or_else(invalid)?.id()),
                BindingContextLayer::Declaration,
            ),
            _ => return Err(invalid()),
        };
        if boundaries
            .insert(local.clone(), (owner, layer, Some(parent_scope.clone())))
            .is_some()
        {
            return Err(invalid());
        }
    }
    if scopes.values().any(|local| !boundaries.contains_key(local)) {
        return Err(invalid());
    }
    let context_ids = boundaries
        .keys()
        .enumerate()
        .map(|(index, local)| (local.clone(), BindingContextId::new(index)))
        .collect::<BTreeMap<_, _>>();

    // Census every original introduction, including names not referenced later.
    let mut names = BTreeMap::new();
    let mut inline_definitions = BTreeMap::new();
    let mut written_types = BTreeMap::new();
    let mut bare_names = BTreeSet::new();
    for row in rows.iter() {
        let valid_proof_roles = match row.kind() {
            K::CaseItem => source_case_item_roles(rows, row).is_some(),
            K::CaseReasoningStatement => source_case_reasoning_roles(rows, row).is_some(),
            K::ConsiderStatement => source_consider_roles(rows, row).is_some(),
            K::GivenStatement => source_given_roles(rows, row).is_some(),
            K::HerebyStatement => source_hereby_roles(rows, row).is_some(),
            K::IterativeEqualityStatement => source_iterative_equality_roles(rows, row).is_some(),
            K::IterativeEqualityStep => source_iterative_equality_step_roles(rows, row).is_some(),
            K::NowStatement => source_now_roles(rows, row).is_some(),
            K::SetStatement => source_set_roles(rows, row).is_some(),
            K::StatementItem => source_statement_item_roles(rows, row).is_some(),
            K::SupposeItem => source_suppose_item_roles(rows, row).is_some(),
            K::ThenStatement => source_then_roles(rows, row).is_some(),
            _ => true,
        };
        if !valid_proof_roles {
            return Err(invalid());
        }
        if row.kind() == &K::ReconsiderItem {
            source_reconsider_item_roles(rows, row).ok_or_else(invalid)?;
        }
        let mut ancestor = Some(row);
        let mut namespace_owned = false;
        while let Some(current) = ancestor {
            if current.kind() == &K::DefinitionBlockItem {
                namespace_owned = true;
                break;
            }
            ancestor = rows
                .iter()
                .find(|parent| Some(parent.site()) == current.parent());
        }
        if namespace_owned {
            if matches!(
                row.kind(),
                K::QuaExpression | K::ReconsiderStatement | K::ReconsiderItem
            ) {
                return Err(invalid());
            }
            continue;
        }
        let (kind, rhs) = match row.kind() {
            K::QuaExpression => {
                source_qua_roles(rows, row).ok_or_else(invalid)?;
                continue;
            }
            K::ReconsiderStatement => {
                let (items, target, _) =
                    source_reconsider_statement_roles(rows, row).ok_or_else(invalid)?;
                for item in items {
                    let (name, value) =
                        source_reconsider_item_roles(rows, item).ok_or_else(invalid)?;
                    if let Some(value) = value {
                        if names
                            .insert(name.site(), (B::Reconsider, Some(value.site())))
                            .is_some()
                            || written_types.insert(name.site(), target.range()).is_some()
                        {
                            return Err(invalid());
                        }
                    } else if !bare_names.insert(name.site()) {
                        return Err(invalid());
                    }
                }
                continue;
            }
            K::InlineFunctorDefinition | K::InlinePredicateDefinition => {
                let functor = row.kind() == &K::InlineFunctorDefinition;
                let (name, parameters, body, result) = if functor {
                    source_inline_functor_roles(rows, row)
                } else {
                    source_inline_predicate_roles(rows, row)
                }
                .ok_or_else(invalid)?;
                let declaration = scope
                    .bindings()
                    .iter()
                    .find(|binding| {
                        row_for(binding.node()).is_some_and(|r| r.site() == name.site())
                    })
                    .ok_or_else(invalid)?;
                let kind = if functor {
                    B::InlineFunctor
                } else {
                    B::InlinePredicate
                };
                let mut formals = Vec::new();
                for [formal, guard] in parameters {
                    let binding = scope
                        .bindings()
                        .iter()
                        .find(|binding| {
                            row_for(binding.node()).is_some_and(|r| r.site() == formal.site())
                        })
                        .ok_or_else(invalid)?;
                    if binding.kind() != B::InlineParameter
                        || binding.declared_type().is_none()
                        || formals
                            .last()
                            .is_some_and(|previous| previous >= &binding.id())
                        || names
                            .insert(formal.site(), (B::InlineParameter, None))
                            .is_some()
                        || written_types.insert(formal.site(), guard.range()).is_some()
                    {
                        return Err(invalid());
                    }
                    formals.push(binding.id());
                }
                if declaration.kind() != kind
                    || declaration.arity() != Some(formals.len())
                    || (functor && declaration.declared_type().is_none())
                    || names
                        .insert(name.site(), (kind, Some(body.site())))
                        .is_some()
                    || inline_definitions
                        .insert(declaration.id(), formals)
                        .is_some()
                {
                    return Err(invalid());
                }
                if let Some(result) = result {
                    written_types.insert(name.site(), result.range());
                }
                continue;
            }
            K::ReserveSegment => (B::Reserve, None),
            K::QuantifierVariableSegment => (B::Quantifier, None),
            K::ComprehensionVariableSegment => (B::ComprehensionGenerator, None),
            K::QualifiedVariableSegment => {
                let parent = rows
                    .iter()
                    .find(|p| Some(p.site()) == row.parent())
                    .ok_or_else(invalid)?;
                (
                    match parent.kind() {
                        K::LetStatement => B::Let,
                        K::GivenStatement => B::GivenWitness,
                        K::ConsiderStatement => B::ConsiderWitness,
                        _ => return Err(invalid()),
                    },
                    None,
                )
            }
            K::Equating => {
                let parent = rows
                    .iter()
                    .find(|p| Some(p.site()) == row.parent())
                    .ok_or_else(invalid)?;
                if parent.kind() != &K::SetStatement {
                    return Err(invalid());
                }
                let [name, value] = source_equating_roles(rows, row).ok_or_else(invalid)?;
                if names
                    .insert(name.site(), (B::Set, Some(value.site())))
                    .is_some()
                {
                    return Err(invalid());
                }
                continue;
            }
            K::Witness => {
                let count = source_contiguous_child_count(source_children(rows, row.site()))
                    .ok_or_else(invalid)?;
                if count == 1
                    && source_child_at(rows, row.site(), 0)
                        .is_some_and(|r| r.kind() == &K::TermExpression)
                {
                    continue;
                }
                if count != 3 {
                    return Err(invalid());
                }
                let name = source_child_at(rows, row.site(), 0).ok_or_else(invalid)?;
                let equals = source_child_at(rows, row.site(), 1).ok_or_else(invalid)?;
                let value =
                    source_unique_child(rows, row, K::TermExpression).ok_or_else(invalid)?;
                if !matches!(name.kind(), K::Token(t) if t.kind == SurfaceTokenKind::Identifier)
                    || !matches!(equals.kind(), K::Token(t) if t.text.as_ref() == "=" && t.kind == SurfaceTokenKind::ReservedSymbol)
                    || value.child_ordinal() != 2
                    || names
                        .insert(name.site(), (B::NamedWitness, Some(value.site())))
                        .is_some()
                {
                    return Err(invalid());
                }
                continue;
            }
            _ => continue,
        };
        let mut found = false;
        for name in source_children(rows, row.site()).filter(
            |name| matches!(name.kind(), K::Token(t) if t.kind == SurfaceTokenKind::Identifier),
        ) {
            found = true;
            if names.insert(name.site(), (kind, rhs)).is_some() {
                return Err(invalid());
            }
        }
        if !found {
            return Err(invalid());
        }
    }
    let mut drafts = Vec::new();
    let mut introduced = BTreeSet::new();
    let mut rhs_owners = BTreeMap::new();
    for binding in scope.bindings() {
        let name = row_for(binding.node()).ok_or_else(invalid)?;
        let (kind, rhs) = names.remove(&name.site()).ok_or_else(invalid)?;
        let expected_scope = if matches!(kind, B::InlineFunctor | B::InlinePredicate) {
            let definition = rows
                .iter()
                .find(|r| Some(r.site()) == name.parent())
                .ok_or_else(invalid)?;
            scopes
                .iter()
                .find(|(id, _)| native.resolved_node_for(**id) == definition.parent())
                .map(|(_, local)| local)
        } else {
            scopes.get(&binding.node())
        };
        if binding.kind() != kind
            || binding.range() != name.range()
            || binding.spelling() != name.spelling()
            || expected_scope != Some(binding.scope())
            || (kind != B::Reserve
                && !introduced.insert((binding.scope().clone(), binding.spelling())))
        {
            return Err(invalid());
        }
        let local = LocalTermBinding::new(
            binding.spelling(),
            binding.scope().clone(),
            binding.range(),
            binding.ordinal(),
        );
        let mut draft = BindingDraft::from_local_term(
            *context_ids.get(binding.scope()).ok_or_else(invalid)?,
            match kind {
                B::Reserve => BindingKind::ReservedVariable,
                B::Quantifier | B::ComprehensionGenerator => BindingKind::QuantifierBinder,
                B::Let => BindingKind::LetBinding,
                B::GivenWitness | B::ConsiderWitness => BindingKind::GivenWitness,
                B::Set | B::NamedWitness | B::Reconsider => BindingKind::LocalAbbreviation,
                B::InlineFunctor | B::InlinePredicate => BindingKind::LocalAbbreviation,
                B::InlineParameter => BindingKind::DefinitionParameter,
                _ => return Err(invalid()),
            },
            &local,
        );
        if kind == B::Reserve {
            draft.identity = BinderIdentity::ReservedVariable {
                spelling: binding.spelling().to_owned(),
                declaration_range: binding.range(),
            };
        }
        if let Some(written) = written_types.remove(&name.site()) {
            draft.type_site = BindingTypeSite::Source(written);
        }
        let actual = BindingId::new(drafts.len());
        drafts.push(draft);
        if actual.index() != binding.id().index() {
            return Err(invalid());
        }
        if let Some(rhs) = rhs
            && rhs_owners.insert(rhs, actual).is_some()
        {
            return Err(invalid());
        }
    }
    if !names.is_empty() || !written_types.is_empty() {
        return Err(invalid());
    }
    let mut captures: BTreeMap<BindingId, Vec<BinderIdentity>> = BTreeMap::new();
    for reference in scope.references() {
        let target = scope
            .bindings()
            .get(reference.binding().index())
            .ok_or_else(invalid)?;
        if target.id() != reference.binding() {
            return Err(invalid());
        }
        if let Some(row) = row_for(reference.node())
            && bare_names.remove(&row.site())
            && reference.kind() != R::Term
        {
            return Err(invalid());
        }
        match (reference.kind(), target.kind()) {
            (R::InlineFunctor { .. }, B::InlineFunctor)
            | (R::InlinePredicate { .. }, B::InlinePredicate) => {}
            (_, B::InlineFunctor | B::InlinePredicate)
            | (R::InlineFunctor { .. } | R::InlinePredicate { .. }, _) => return Err(invalid()),
            _ => {}
        }
        if let Some(arity) = reference.kind().arity() {
            let formals = inline_definitions.get(&target.id()).ok_or_else(invalid)?;
            let callee = row_for(reference.node()).ok_or_else(invalid)?;
            let call = rows
                .iter()
                .find(|row| Some(row.site()) == callee.parent())
                .ok_or_else(invalid)?;
            let expected_kind = if target.kind() == B::InlineFunctor {
                K::ApplicationTerm
            } else {
                K::InlinePredicateApplication
            };
            let children = source_children(rows, call.site()).collect::<Vec<_>>();
            let token = |index, text| source_inline_token(&children, index, text);
            let callee_name = if target.kind() == B::InlineFunctor {
                if source_contiguous_child_count(source_children(rows, callee.site())) != Some(1) {
                    return Err(invalid());
                }
                source_child_at(rows, callee.site(), 0).ok_or_else(invalid)?
            } else {
                callee
            };
            if !matches!(callee_name.kind(), K::Token(t) if t.kind == SurfaceTokenKind::Identifier)
                || call.kind() != &expected_kind
                || callee.child_ordinal() != 0
                || formals.len() != arity
                || !token(1, "(")
                || source_contiguous_child_count(children.iter().copied()) != Some(children.len())
                || children.len() != if arity == 0 { 3 } else { arity * 2 + 2 }
                || !token(children.len().saturating_sub(1), ")")
            {
                return Err(invalid());
            }
            // The request domain is exactly the ordered formal list; guards stay written,
            // not discharged. Original actual subtrees remain source-scoped graph edges.
            for (index, formal) in formals.iter().enumerate() {
                let binding = scope.bindings().get(formal.index()).ok_or_else(invalid)?;
                let name = row_for(binding.node()).ok_or_else(invalid)?;
                let parameter = rows
                    .iter()
                    .find(|row| Some(row.site()) == name.parent())
                    .ok_or_else(invalid)?;
                let [written_name, guard] =
                    source_inline_parameter_roles(rows, parameter).ok_or_else(invalid)?;
                if written_name.site() != name.site()
                    || drafts.get(formal.index()).is_none_or(|draft| {
                        draft.type_site != BindingTypeSite::Source(guard.range())
                    })
                    || children
                        .get(index * 2 + 2)
                        .is_none_or(|arg| arg.kind() != &K::TermExpression)
                    || (index + 1 < arity && !token(index * 2 + 3, ","))
                    || scope.bindings().get(formal.index()).is_none_or(|binding| {
                        binding.kind() != B::InlineParameter || binding.declared_type().is_none()
                    })
                {
                    return Err(invalid());
                }
            }
        }
        let mut current = Some(reference.node());
        while let Some(id) = current {
            let site = native.resolved_node_for(id).ok_or_else(invalid)?;
            if let Some(owner) = rhs_owners.get(&site) {
                let declaration = &scope.bindings()[owner.index()];
                if owner.index() == target.id().index()
                    && matches!(declaration.kind(), B::Set | B::NamedWitness | B::Reconsider)
                {
                    return Err(invalid());
                }
                if matches!(
                    declaration.kind(),
                    B::Set | B::InlineFunctor | B::InlinePredicate
                ) && owner.index() != target.id().index()
                    && declaration
                        .scope()
                        .path()
                        .starts_with(target.scope().path())
                {
                    captures.entry(*owner).or_default().push(
                        drafts
                            .get(target.id().index())
                            .ok_or_else(invalid)?
                            .identity
                            .clone(),
                    );
                }
                break;
            }
            current = parents.get(&id).copied();
        }
    }
    if !bare_names.is_empty() {
        return Err(invalid());
    }
    for id in inline_definitions.keys() {
        let expected = CapturedFreeVariables::new(
            scope.bindings()[id.index()]
                .captures()
                .iter()
                .map(|id| drafts[id.index()].identity.clone())
                .collect(),
        );
        let actual = CapturedFreeVariables::new(
            captures
                .get(&BindingId::new(id.index()))
                .cloned()
                .unwrap_or_default(),
        );
        if expected != actual {
            return Err(invalid());
        }
    }
    let mut bindings = BindingTable::new();
    for (index, mut draft) in drafts.into_iter().enumerate() {
        draft.captured =
            CapturedFreeVariables::new(captures.remove(&BindingId::new(index)).unwrap_or_default());
        bindings.insert(draft);
    }
    let mut contexts = BindingContextTable::new();
    for (local, (owner, layer, parent)) in boundaries {
        contexts.insert(BindingContextDraft {
            owner,
            layer,
            parent: parent.map(|p| context_ids[&p]),
            lexical_scope: Some(local.clone()),
            bindings: scope
                .bindings()
                .iter()
                .filter(|b| b.scope() == &local)
                .map(|b| BindingId::new(b.id().index()))
                .collect(),
            visible_bindings: scope
                .bindings()
                .iter()
                .filter(|b| local.path().starts_with(b.scope().path()))
                .map(|b| BindingId::new(b.id().index()))
                .collect(),
            recovery: BindingContextRecovery::Normal,
        });
    }
    BindingEnv::try_new(BindingEnvParts {
        source_id: original.source_id(),
        module_id: original.module_id().clone(),
        contexts,
        bindings,
        diagnostics: BindingDiagnosticTable::new(),
    })
    .map_err(|_| SourceProofLocalDeclarationError::InvalidBindingEnvironment)
}

/// Complete syntax-free input for the frozen proof-local `let` binding transaction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceProofLocalLetBindingHandoffInput {
    pub source_id: SourceId,
    pub module_id: ModuleId,
    pub lower_fingerprint: SourceProofLocalIntakeReceipt,
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

/// Authenticated original-source intake for the supported proof-local binding profiles.
#[derive(Clone, PartialEq, Eq)]
pub struct SourceProofLocalIntakeReceipt {
    source_id: SourceId,
    module_id: ModuleId,
    original: Arc<[SourceNode]>,
    declaration: ResolvedNodeId,
    fingerprint: String,
}

impl fmt::Debug for SourceProofLocalIntakeReceipt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("SourceProofLocalIntakeReceipt")
    }
}

impl SourceProofLocalIntakeReceipt {
    pub fn from_source(source: &TypedAst, declaration: ResolvedNodeId) -> Option<Self> {
        let mut receipt = Self {
            source_id: source.source_id(),
            module_id: source.module_id().clone(),
            original: source.source_nodes()?.clone(),
            declaration,
            fingerprint: String::new(),
        };
        receipt.profile()?;
        receipt.fingerprint = receipt.structural_fingerprint();
        Some(receipt)
    }

    pub fn fingerprint(&self) -> &str {
        &self.fingerprint
    }

    fn role(&self, kind: SurfaceNodeKind) -> Option<&SourceNode> {
        unique_source_node(self.original.iter(), |row| row.kind() == &kind)
    }

    fn profile(&self) -> Option<usize> {
        use SurfaceNodeKind::*;
        use SurfaceTokenKind as T;
        let root = self.role(Root)?;
        let profile = [
            "reserve x for set ; theorem FormulaStatementLetSmoke : x = x proof let y be set ; thus x = x ; end ;",
            "reserve x for set ; theorem FormulaStatementGivenSmoke : thesis proof given y being set such that G : thesis ; thus thesis ; end ;",
            "reserve x for set ; theorem FormulaStatementGivenSmoke : thesis proof given y being set such that G : thesis ; thus y = y ; end ;",
            "reserve x for set ; theorem ProofLocalGivenConditionUseSmoke : thesis proof given y being set such that G : y = y ; thus thesis ; end ;",
            "reserve x for set ; theorem ProofLocalGivenDescendantCaptureSmoke : thesis proof given y being set ; now set z = y ; set q = z ; thus thesis ; end ; thus thesis ; end ;",
        ].iter().position(|text| root.spelling() == *text)?;
        let token_count = self
            .original
            .iter()
            .filter(|row| matches!(row.kind(), Token(_)))
            .count();
        // The original rows supply grammar, not merely the normalized token string.
        for row in self.original.iter() {
            if row.range().source_id != self.source_id
                || row.recovery() != NodeRecoveryState::Normal
            {
                return None;
            }
            let formula_frame;
            let (count, roles): (usize, &[(usize, SurfaceNodeKind)]) = match row.kind() {
                Root => (1, &[(0, CompilationUnit)]),
                CompilationUnit => (1, &[(0, ItemList)]),
                ItemList => (2, &[(0, ReserveItem), (1, TheoremItem)]),
                ReserveItem => (3, &[(1, ReserveSegment)]),
                ReserveSegment | QualifiedVariableSegment => (3, &[(2, TypeExpression)]),
                TheoremItem => (6, &[(3, FormulaExpression), (4, ProofBlock)]),
                ProofBlock if profile == 0 => (4, &[(1, LetStatement), (2, ConclusionStatement)]),
                ProofBlock if profile == 4 => (
                    5,
                    &[
                        (1, GivenStatement),
                        (2, NowStatement),
                        (3, ConclusionStatement),
                    ],
                ),
                ProofBlock => (4, &[(1, GivenStatement), (2, ConclusionStatement)]),
                LetStatement => (3, &[(1, QualifiedVariableSegment)]),
                GivenStatement if profile == 4 => (3, &[(1, QualifiedVariableSegment)]),
                GivenStatement => (5, &[(1, QualifiedVariableSegment), (3, ConditionList)]),
                ConditionList => (2, &[(1, Proposition)]),
                Proposition
                    if self
                        .original
                        .iter()
                        .any(|p| Some(p.site()) == row.parent() && p.kind() == &ConditionList) =>
                {
                    (3, &[(2, FormulaExpression)])
                }
                Proposition => (1, &[(0, FormulaExpression)]),
                ConclusionStatement => (3, &[(1, Proposition)]),
                FormulaExpression => {
                    let child = source_child_at(&self.original, row.site(), 0)?;
                    if !matches!(
                        child.kind(),
                        BuiltinPredicateApplication
                            | FormulaConstant(SurfaceFormulaConstant::Thesis)
                    ) {
                        return None;
                    }
                    formula_frame = [(0, child.kind().clone())];
                    (1, &formula_frame)
                }
                BuiltinPredicateApplication => (3, &[(0, TermExpression), (2, TermExpression)]),
                TermExpression => (1, &[(0, TermReference)]),
                TermReference | TypeHead | FormulaConstant(SurfaceFormulaConstant::Thesis) => {
                    (1, &[])
                }
                TypeExpression => (1, &[(0, TypeHead)]),
                NowStatement => (
                    6,
                    &[
                        (1, SetStatement),
                        (2, SetStatement),
                        (3, ConclusionStatement),
                    ],
                ),
                SetStatement => (3, &[(1, Equating)]),
                Equating => (3, &[(2, TermExpression)]),
                Token(token) => {
                    let kind = match token.text.as_ref() {
                        ";" | ":" | "=" => T::ReservedSymbol,
                        "reserve" | "for" | "set" | "theorem" | "proof" | "let" | "be"
                        | "being" | "thus" | "end" | "given" | "such" | "that" | "thesis"
                        | "now" => T::ReservedWord,
                        _ => T::Identifier,
                    };
                    if token.kind != kind || token.text.as_ref() != row.spelling() {
                        return None;
                    }
                    (0, &[])
                }
                _ => return None,
            };
            let children = source_children(&self.original, row.site()).collect::<Vec<_>>();
            if children.len() != count
                || children.iter().enumerate().any(|(i, child)| {
                    (child.child_ordinal() != i
                        && !(row.kind() == &Root && child.child_ordinal() == token_count))
                        || child.preorder_ordinal() <= row.preorder_ordinal()
                        || child.range().start < row.range().start
                        || child.range().end > row.range().end
                        || match roles.iter().find(|(ordinal, _)| *ordinal == i) {
                            Some((_, kind)) => child.kind() != kind,
                            None => !matches!(child.kind(), Token(_)),
                        }
                })
            {
                return None;
            }
        }
        let theorem = self.role(TheoremItem)?;
        let proof = source_unique_child(&self.original, theorem, ProofBlock)?;
        let declaration = source_unique_child(
            &self.original,
            proof,
            if profile == 0 {
                LetStatement
            } else {
                GivenStatement
            },
        )?;
        if declaration.site() != self.declaration {
            return None;
        }
        Some(profile)
    }

    fn structural_fingerprint(&self) -> String {
        let mut text = format!(
            "proof-local-intake-v1:{:?}:{:?}:{};",
            self.module_id.package().as_str(),
            self.module_id.path().as_str(),
            self.declaration.index()
        );
        for row in self.original.iter() {
            let kind = match row.kind() {
                SurfaceNodeKind::Token(token) => format!("token:{:?}:{:?}", token.kind, token.text),
                kind => format!("{kind:?}"),
            };
            write!(
                text,
                "{:?};",
                (
                    row.site().index(),
                    kind,
                    row.parent().map(|p| p.index()),
                    row.child_ordinal(),
                    row.preorder_ordinal(),
                    row.range().start,
                    row.range().end,
                    row.recovery(),
                    row.spelling()
                )
            )
            .expect("String write");
        }
        text
    }

    fn matches(
        &self,
        source: SourceId,
        module: &ModuleId,
        profile: usize,
        ranges: [SourceRange; 5],
    ) -> bool {
        use SurfaceNodeKind as K;
        self.source_id == source
            && &self.module_id == module
            && self.profile() == Some(profile)
            && self.fingerprint == self.structural_fingerprint()
            && [
                self.role(K::TheoremItem),
                self.role(K::ProofBlock),
                self.role(if profile == 0 {
                    K::LetStatement
                } else {
                    K::GivenStatement
                }),
                self.role(K::QualifiedVariableSegment),
                self.role(K::QualifiedVariableSegment)
                    .and_then(|p| source_child_at(&self.original, p.site(), 0)),
            ]
            .into_iter()
            .zip(ranges)
            .all(|(row, range)| row.is_some_and(|r| r.range() == range))
    }

    pub(crate) fn type_rows(&self) -> [&SourceNode; 2] {
        use SurfaceNodeKind as K;
        [K::ReserveSegment, K::QualifiedVariableSegment].map(|kind| {
            source_unique_child(
                &self.original,
                self.role(kind).expect("validated receipt segment"),
                K::TypeExpression,
            )
            .expect("validated receipt type")
        })
    }
    pub(crate) fn type_heads(&self) -> [&SourceNode; 2] {
        self.type_rows().map(|row| {
            source_unique_child(&self.original, row, SurfaceNodeKind::TypeHead)
                .expect("validated type head")
        })
    }
    pub(crate) fn root_row(&self) -> &SourceNode {
        self.role(SurfaceNodeKind::Root)
            .expect("validated receipt root")
    }
    fn reserve_name(&self) -> &SourceNode {
        source_child_at(
            &self.original,
            self.role(SurfaceNodeKind::ReserveSegment)
                .expect("validated reserve")
                .site(),
            0,
        )
        .expect("validated reserve name")
    }
    fn descendant_range(&self) -> Option<SourceRange> {
        self.role(SurfaceNodeKind::NowStatement)
            .map(SourceNode::range)
    }
    pub(crate) fn term_rows(&self) -> Option<Vec<&SourceNode>> {
        use SurfaceNodeKind::*;
        let owner = match self.profile()? {
            2 => source_unique_child(&self.original, self.role(ProofBlock)?, ConclusionStatement)?,
            3 => source_unique_child(&self.original, self.role(GivenStatement)?, ConditionList)?,
            4 => {
                let now = self.role(NowStatement)?;
                let set = source_child_at(&self.original, now.site(), 1)?;
                let equating = source_unique_child(&self.original, set, Equating)?;
                return Some(vec![source_unique_child(
                    &self.original,
                    equating,
                    TermExpression,
                )?]);
            }
            _ => return None,
        };
        let proposition = source_unique_child(&self.original, owner, Proposition)?;
        let formula = source_unique_child(&self.original, proposition, FormulaExpression)?;
        let predicate = source_unique_child(&self.original, formula, BuiltinPredicateApplication)?;
        [0, 2]
            .into_iter()
            .map(|ordinal| {
                let term = source_child_at(&self.original, predicate.site(), ordinal)?;
                if term.kind() != &TermExpression {
                    return None;
                }
                let reference = source_unique_child(&self.original, term, TermReference)?;
                let name = source_child_at(&self.original, reference.site(), 0)?;
                let valid_name = matches!(name.kind(), Token(token)
                if token.kind == SurfaceTokenKind::Identifier && token.text.as_ref() == "y");
                valid_name.then_some(reference)
            })
            .collect()
    }
}

/// Immutable proof-local `let` binding transaction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceProofLocalLetBindingHandoff {
    source_id: SourceId,
    module_id: ModuleId,
    lower_fingerprint: Box<SourceProofLocalIntakeReceipt>,
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
    pub(crate) fn intake(&self) -> &SourceProofLocalIntakeReceipt {
        &self.lower_fingerprint
    }

    pub const fn source_id(&self) -> SourceId {
        self.source_id
    }

    pub const fn module_id(&self) -> &ModuleId {
        &self.module_id
    }

    pub fn lower_fingerprint(&self) -> &str {
        self.lower_fingerprint.fingerprint()
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
            self.lower_fingerprint.fingerprint(),
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
        if !exact_task269c_base_binding_env(&self.base_binding_env, &self.lower_fingerprint)
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
            || !exact_task269_binding(&self.binding_env, self.name_range, None)
        {
            return Err(SourceProofLocalLetBindingError::InvalidDeclaration { binding: id });
        }
        let expected = extend_task269c_binding_env(
            &self.base_binding_env,
            &self.lower_fingerprint,
            self.proof_range,
            self.name_range,
        )?;
        if self.binding_env != expected
            || self.final_binding_fingerprint != self.binding_env.debug_text()
            || !exact_task269_binding(
                &self.binding_env,
                self.name_range,
                Some((BindingKind::LetBinding, self.proof_range)),
            )
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
        self.lower_fingerprint.fingerprint = value.into();
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
        if !exact_task269c_base_binding_env(base_binding_env, &input.lower_fingerprint) {
            return Err(SourceProofLocalLetBindingError::InvalidBaseBindingEnvironment);
        }
        if !exact_task269c_input_declaration(&input) {
            return Err(SourceProofLocalLetBindingError::InvalidDeclaration {
                binding: SourceProofLocalLetBindingId::new(0),
            });
        }
        let binding_env = extend_task269c_binding_env(
            base_binding_env,
            &input.lower_fingerprint,
            input.proof_range,
            input.name_range,
        )?;
        if !exact_task269c_lookup_behavior(&binding_env) {
            return Err(SourceProofLocalLetBindingError::InvalidBindingEnvironment);
        }
        let base_binding_fingerprint = base_binding_env.debug_text();
        let final_binding_fingerprint = binding_env.debug_text();
        Ok(SourceProofLocalLetBindingHandoff {
            source_id: input.source_id,
            module_id: input.module_id,
            lower_fingerprint: Box::new(input.lower_fingerprint),
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
    lower_fingerprint: &'a SourceProofLocalIntakeReceipt,
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
        || !dependency.lower_fingerprint.matches(
            dependency.source_id,
            dependency.module_id,
            0,
            [
                dependency.theorem_range,
                dependency.proof_range,
                dependency.let_range,
                dependency.segment_range,
                dependency.name_range,
            ],
        )
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

fn exact_task269c_base_binding_env(
    base: &BindingEnv,
    receipt: &SourceProofLocalIntakeReceipt,
) -> bool {
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
                && *declaration_range == receipt.reserve_name().range()
        )
        && binding.owner_context == BindingContextId::new(0)
        && binding.declaration_range == receipt.reserve_name().range()
        && binding.visible_after_ordinal == 0
        && binding.type_site == BindingTypeSite::Source(receipt.type_rows()[0].range())
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

fn exact_task269_binding(
    binding_env: &BindingEnv,
    name_range: SourceRange,
    local: Option<(BindingKind, SourceRange)>,
) -> bool {
    let Some(binding) = binding_env.bindings().get(BindingId::new(1)) else {
        return false;
    };
    if !(binding.id == BindingId::new(1)
        && binding.spelling == "y"
        && matches!(
            &binding.identity,
            BinderIdentity::ResolverLocal {
                scope,
                ordinal: 1,
                declaration_range,
            } if scope.path() == [0]
                && *declaration_range == name_range
        )
        && binding.owner_context == BindingContextId::new(1)
        && binding.declaration_range == name_range
        && binding.visible_after_ordinal == 1
        && binding.recovery == BindingRecoveryState::Normal)
    {
        return false;
    }
    // Declaration replay deliberately defers the later context and binding-kind checks.
    let Some((kind, proof_range)) = local else {
        return true;
    };
    let Some(context) = binding_env.contexts().get(BindingContextId::new(1)) else {
        return false;
    };
    context.id == BindingContextId::new(1)
        && context.owner
            == BindingContextOwner::SourceStatement {
                source_range: proof_range,
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
        && binding.kind == kind
        && binding.type_site == BindingTypeSite::Missing
        && binding.status == BindingStatus::Active
        && binding.captured.identities().is_empty()
        && binding.diagnostics.is_empty()
}

fn extend_task269c_binding_env(
    base: &BindingEnv,
    receipt: &SourceProofLocalIntakeReceipt,
    proof_range: SourceRange,
    name_range: SourceRange,
) -> Result<BindingEnv, SourceProofLocalLetBindingError> {
    if !exact_task269c_base_binding_env(base, receipt) {
        return Err(SourceProofLocalLetBindingError::InvalidBaseBindingEnvironment);
    }
    let local = LocalTermBinding::new("y", LocalTermScope::new(vec![0]), name_range, 1);
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
            source_range: proof_range,
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
    pub lower_fingerprint: SourceProofLocalIntakeReceipt,
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
    lower_fingerprint: Box<SourceProofLocalIntakeReceipt>,
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
    pub(crate) fn intake(&self) -> &SourceProofLocalIntakeReceipt {
        &self.lower_fingerprint
    }

    pub const fn source_id(&self) -> SourceId {
        self.source_id
    }

    pub const fn module_id(&self) -> &ModuleId {
        &self.module_id
    }

    pub fn lower_fingerprint(&self) -> &str {
        self.lower_fingerprint.fingerprint()
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
            self.lower_fingerprint.fingerprint(),
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
        if !exact_task269c_base_binding_env(&self.base_binding_env, &self.lower_fingerprint)
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
            || !exact_task269_binding(&self.binding_env, self.name_range, None)
        {
            return Err(SourceProofLocalGivenBindingError::InvalidDeclaration { binding: id });
        }
        let expected = extend_task269g_binding_env(
            &self.base_binding_env,
            &self.lower_fingerprint,
            self.proof_range,
            self.name_range,
        )?;
        if self.binding_env != expected
            || self.final_binding_fingerprint != self.binding_env.debug_text()
            || !exact_task269_binding(
                &self.binding_env,
                self.name_range,
                Some((BindingKind::GivenWitness, self.proof_range)),
            )
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
        self.lower_fingerprint.fingerprint = value.into();
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
        if !exact_task269c_base_binding_env(base_binding_env, &input.lower_fingerprint) {
            return Err(SourceProofLocalGivenBindingError::InvalidBaseBindingEnvironment);
        }
        if !exact_task269g_input_declaration(&input) {
            return Err(SourceProofLocalGivenBindingError::InvalidDeclaration {
                binding: SourceProofLocalGivenBindingId::new(0),
            });
        }
        let binding_env = extend_task269g_binding_env(
            base_binding_env,
            &input.lower_fingerprint,
            input.proof_range,
            input.name_range,
        )?;
        if !exact_task269g_lookup_behavior(&binding_env) {
            return Err(SourceProofLocalGivenBindingError::InvalidBindingEnvironment);
        }
        let base_binding_fingerprint = base_binding_env.debug_text();
        let final_binding_fingerprint = binding_env.debug_text();
        Ok(SourceProofLocalGivenBindingHandoff {
            source_id: input.source_id,
            module_id: input.module_id,
            lower_fingerprint: Box::new(input.lower_fingerprint),
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
    lower_fingerprint: &'a SourceProofLocalIntakeReceipt,
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
        || !dependency.lower_fingerprint.matches(
            dependency.source_id,
            dependency.module_id,
            1,
            [
                dependency.theorem_range,
                dependency.proof_range,
                dependency.given_range,
                dependency.segment_range,
                dependency.name_range,
            ],
        )
    {
        return Err(SourceProofLocalGivenBindingError::DependencyMismatch);
    }
    Ok(())
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

fn extend_task269g_binding_env(
    base: &BindingEnv,
    receipt: &SourceProofLocalIntakeReceipt,
    proof_range: SourceRange,
    name_range: SourceRange,
) -> Result<BindingEnv, SourceProofLocalGivenBindingError> {
    if !exact_task269c_base_binding_env(base, receipt) {
        return Err(SourceProofLocalGivenBindingError::InvalidBaseBindingEnvironment);
    }
    let local = LocalTermBinding::new("y", LocalTermScope::new(vec![0]), name_range, 1);
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
            source_range: proof_range,
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
    pub lower_fingerprint: SourceProofLocalIntakeReceipt,
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
    lower_fingerprint: Box<SourceProofLocalIntakeReceipt>,
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
    pub(crate) fn intake(&self) -> &SourceProofLocalIntakeReceipt {
        &self.lower_fingerprint
    }

    pub const fn source_id(&self) -> SourceId {
        self.source_id
    }

    pub const fn module_id(&self) -> &ModuleId {
        &self.module_id
    }

    pub fn lower_fingerprint(&self) -> &str {
        self.lower_fingerprint.fingerprint()
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
            self.lower_fingerprint.fingerprint(),
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
        if !exact_task269c_base_binding_env(&self.base_binding_env, &self.lower_fingerprint)
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
        if !exact_task269g_output_binding(binding)
            || !exact_task269_binding(&self.binding_env, self.name_range, None)
        {
            return Err(SourceProofLocalGivenUseBindingError::InvalidDeclaration { binding: id });
        }
        let expected = extend_task269gup_binding_env(
            &self.base_binding_env,
            &self.lower_fingerprint,
            self.proof_range,
            self.name_range,
        )?;
        if self.binding_env != expected
            || !exact_task269_binding(
                &self.binding_env,
                self.name_range,
                Some((BindingKind::GivenWitness, self.proof_range)),
            )
            || !exact_task269g_lookup_behavior(&self.binding_env)
            || self.final_binding_fingerprint != self.binding_env.debug_text()
        {
            return Err(SourceProofLocalGivenUseBindingError::InvalidBindingEnvironment);
        }
        Ok(())
    }

    #[cfg(test)]
    pub(crate) fn set_lower_fingerprint_for_task269gup_test(&mut self, value: impl Into<String>) {
        self.lower_fingerprint.fingerprint = value.into();
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
        if !exact_task269c_base_binding_env(base_binding_env, &input.lower_fingerprint) {
            return Err(SourceProofLocalGivenUseBindingError::InvalidBaseBindingEnvironment);
        }
        if !exact_task269gup_input_declaration(&input) {
            return Err(SourceProofLocalGivenUseBindingError::InvalidDeclaration {
                binding: SourceProofLocalGivenBindingId::new(0),
            });
        }
        let binding_env = extend_task269gup_binding_env(
            base_binding_env,
            &input.lower_fingerprint,
            input.proof_range,
            input.name_range,
        )?;
        if !exact_task269g_lookup_behavior(&binding_env) {
            return Err(SourceProofLocalGivenUseBindingError::InvalidBindingEnvironment);
        }
        let base_binding_fingerprint = base_binding_env.debug_text();
        let final_binding_fingerprint = binding_env.debug_text();
        Ok(SourceProofLocalGivenUseBindingHandoff {
            source_id: input.source_id,
            module_id: input.module_id,
            lower_fingerprint: Box::new(input.lower_fingerprint),
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
    lower_fingerprint: &'a SourceProofLocalIntakeReceipt,
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
        || !dependency.lower_fingerprint.matches(
            dependency.source_id,
            dependency.module_id,
            2,
            [
                dependency.theorem_range,
                dependency.proof_range,
                dependency.given_range,
                dependency.segment_range,
                dependency.name_range,
            ],
        )
    {
        return Err(SourceProofLocalGivenUseBindingError::DependencyMismatch);
    }
    Ok(())
}

fn exact_task269gup_input_declaration(input: &SourceProofLocalGivenUseBindingHandoffInput) -> bool {
    input.source_ordinal == 1
        && input.local.spelling() == "y"
        && input.local.scope().path() == [0]
        && input.local.declaration_range() == input.name_range
        && input.local.visible_after_ordinal() == 1
        && input.recovery == SourceProofLocalGivenBindingRecovery::Normal
}

fn extend_task269gup_binding_env(
    base: &BindingEnv,
    receipt: &SourceProofLocalIntakeReceipt,
    proof_range: SourceRange,
    name_range: SourceRange,
) -> Result<BindingEnv, SourceProofLocalGivenUseBindingError> {
    if !exact_task269c_base_binding_env(base, receipt) {
        return Err(SourceProofLocalGivenUseBindingError::InvalidBaseBindingEnvironment);
    }
    let local = LocalTermBinding::new("y", LocalTermScope::new(vec![0]), name_range, 1);
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
            source_range: proof_range,
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceProofLocalGivenConditionBindingHandoffInput {
    pub source_id: SourceId,
    pub module_id: ModuleId,
    pub lower_fingerprint: SourceProofLocalIntakeReceipt,
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
    lower_fingerprint: Box<SourceProofLocalIntakeReceipt>,
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
    pub(crate) fn intake(&self) -> &SourceProofLocalIntakeReceipt {
        &self.lower_fingerprint
    }

    pub const fn source_id(&self) -> SourceId {
        self.source_id
    }

    pub const fn module_id(&self) -> &ModuleId {
        &self.module_id
    }

    pub fn lower_fingerprint(&self) -> &str {
        self.lower_fingerprint.fingerprint()
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
            self.lower_fingerprint.fingerprint(),
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
        if !exact_task269c_base_binding_env(&self.base_binding_env, &self.lower_fingerprint)
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
        if !exact_task269g_output_binding(binding)
            || !exact_task269_binding(&self.binding_env, self.name_range, None)
        {
            return Err(
                SourceProofLocalGivenConditionBindingError::InvalidDeclaration { binding: id },
            );
        }
        let expected = extend_task269gc_binding_env(
            &self.base_binding_env,
            &self.lower_fingerprint,
            self.proof_range,
            self.name_range,
        )?;
        if self.binding_env != expected
            || !exact_task269_binding(
                &self.binding_env,
                self.name_range,
                Some((BindingKind::GivenWitness, self.proof_range)),
            )
            || !exact_task269g_lookup_behavior(&self.binding_env)
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
        self.lower_fingerprint.fingerprint = value.into();
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
        if !exact_task269c_base_binding_env(base_binding_env, &input.lower_fingerprint) {
            return Err(SourceProofLocalGivenConditionBindingError::InvalidBaseBindingEnvironment);
        }
        if !exact_task269gc_input_declaration(&input) {
            return Err(
                SourceProofLocalGivenConditionBindingError::InvalidDeclaration {
                    binding: SourceProofLocalGivenBindingId::new(0),
                },
            );
        }
        let binding_env = extend_task269gc_binding_env(
            base_binding_env,
            &input.lower_fingerprint,
            input.proof_range,
            input.name_range,
        )?;
        if !exact_task269g_lookup_behavior(&binding_env) {
            return Err(SourceProofLocalGivenConditionBindingError::InvalidBindingEnvironment);
        }
        let base_binding_fingerprint = base_binding_env.debug_text();
        let final_binding_fingerprint = binding_env.debug_text();
        Ok(SourceProofLocalGivenConditionBindingHandoff {
            source_id: input.source_id,
            module_id: input.module_id,
            lower_fingerprint: Box::new(input.lower_fingerprint),
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
    lower_fingerprint: &'a SourceProofLocalIntakeReceipt,
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
        || !dependency.lower_fingerprint.matches(
            dependency.source_id,
            dependency.module_id,
            3,
            [
                dependency.theorem_range,
                dependency.proof_range,
                dependency.given_range,
                dependency.segment_range,
                dependency.name_range,
            ],
        )
    {
        return Err(SourceProofLocalGivenConditionBindingError::DependencyMismatch);
    }
    Ok(())
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

fn extend_task269gc_binding_env(
    base: &BindingEnv,
    receipt: &SourceProofLocalIntakeReceipt,
    proof_range: SourceRange,
    name_range: SourceRange,
) -> Result<BindingEnv, SourceProofLocalGivenConditionBindingError> {
    if !exact_task269c_base_binding_env(base, receipt) {
        return Err(SourceProofLocalGivenConditionBindingError::InvalidBaseBindingEnvironment);
    }
    let local = LocalTermBinding::new("y", LocalTermScope::new(vec![0]), name_range, 1);
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
            source_range: proof_range,
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

/// Complete syntax-free input for the frozen proof-local `given` descendant
/// binding transaction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceProofLocalGivenDescendantBindingHandoffInput {
    pub source_id: SourceId,
    pub module_id: ModuleId,
    pub lower_fingerprint: SourceProofLocalIntakeReceipt,
    pub theorem_symbol: SymbolId,
    pub theorem_definition: DefinitionId,
    pub contribution: SourceContributionId,
    pub theorem_range: SourceRange,
    pub proof_range: SourceRange,
    pub given_range: SourceRange,
    pub segment_range: SourceRange,
    pub name_range: SourceRange,
    pub descendant_range: SourceRange,
    pub source_ordinal: usize,
    pub local: LocalTermBinding,
    pub descendant_scope: LocalTermScope,
    pub recovery: SourceProofLocalGivenBindingRecovery,
}

/// Immutable proof-local `given` binding plus descendant-context transaction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceProofLocalGivenDescendantBindingHandoff {
    source_id: SourceId,
    module_id: ModuleId,
    lower_fingerprint: Box<SourceProofLocalIntakeReceipt>,
    theorem_symbol: SymbolId,
    theorem_definition: DefinitionId,
    contribution: SourceContributionId,
    theorem_range: SourceRange,
    proof_range: SourceRange,
    given_range: SourceRange,
    segment_range: SourceRange,
    name_range: SourceRange,
    descendant_range: SourceRange,
    base_binding_env: BindingEnv,
    base_binding_fingerprint: String,
    binding_env: BindingEnv,
    final_binding_fingerprint: String,
    bindings: SourceProofLocalGivenBindingTable,
    descendant_context: BindingContextId,
    set_scope: Option<Box<ResolvedVariableScope>>,
}

impl SourceProofLocalGivenDescendantBindingHandoff {
    pub(crate) fn intake(&self) -> &SourceProofLocalIntakeReceipt {
        &self.lower_fingerprint
    }

    pub const fn source_id(&self) -> SourceId {
        self.source_id
    }

    pub const fn module_id(&self) -> &ModuleId {
        &self.module_id
    }

    pub fn lower_fingerprint(&self) -> &str {
        self.lower_fingerprint.fingerprint()
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

    pub const fn descendant_range(&self) -> SourceRange {
        self.descendant_range
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

    pub const fn descendant_context(&self) -> BindingContextId {
        self.descendant_context
    }

    pub fn debug_text(&self) -> String {
        let binding = self
            .bindings
            .get(SourceProofLocalGivenBindingId::new(0))
            .expect("validated Task269SDC handoff has one dense row");
        let descendant = self
            .binding_env
            .contexts()
            .get(self.descendant_context)
            .expect("validated Task269SDC handoff has one descendant context");
        let parent = descendant
            .parent
            .expect("validated Task269SDC descendant has a parent");
        format!(
            concat!(
                "source-proof-local-given-descendant-binding-debug-v1\n",
                "module: {}::{}\n",
                "lower-fingerprint: {:?}\n",
                "theorem symbol={:?} definition={} contribution={} range={}..{} proof={}..{}\n",
                "given range={}..{} segment={}..{} name={}..{} source_ordinal={}\n",
                "base-binding-fingerprint: {:?}\n",
                "binding#0 binding={} context={} source_ordinal={} visible_after={} recovery={}\n",
                "descendant range={}..{} context={} parent={} scope=[{}] recovery={}\n",
                "final-binding-fingerprint: {:?}\n",
            ),
            self.module_id.package().as_str(),
            self.module_id.path().as_str(),
            self.lower_fingerprint.fingerprint(),
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
            self.descendant_range.start,
            self.descendant_range.end,
            descendant.id.index(),
            parent.index(),
            descendant
                .lexical_scope
                .as_ref()
                .expect("validated descendant scope")
                .path()
                .iter()
                .map(u32::to_string)
                .collect::<Vec<_>>()
                .join(","),
            binding_context_recovery_key(descendant.recovery),
            self.final_binding_fingerprint,
        )
    }

    pub(crate) fn validate_installation(
        &self,
        source_id: SourceId,
        module_id: &ModuleId,
    ) -> Result<(), SourceProofLocalGivenDescendantBindingError> {
        if self.source_id != source_id
            || &self.module_id != module_id
            || self.base_binding_env.source_id() != source_id
            || self.base_binding_env.module_id() != module_id
            || self.binding_env.source_id() != source_id
            || self.binding_env.module_id() != module_id
        {
            return Err(SourceProofLocalGivenDescendantBindingError::InvalidTransaction);
        }
        validate_task269sdc_dependency(Task269sdcDependency {
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
        if !exact_task269c_base_binding_env(&self.base_binding_env, &self.lower_fingerprint)
            || self.base_binding_fingerprint != self.base_binding_env.debug_text()
        {
            return Err(SourceProofLocalGivenDescendantBindingError::InvalidBaseBindingEnvironment);
        }
        if self.bindings.len() != 1 {
            return Err(SourceProofLocalGivenDescendantBindingError::InvalidAggregate);
        }
        let id = SourceProofLocalGivenBindingId::new(0);
        let binding = self.bindings.get(id).ok_or(
            SourceProofLocalGivenDescendantBindingError::InvalidDeclaration { binding: id },
        )?;
        if !exact_task269g_output_binding(binding)
            || (self.set_scope.is_none()
                && !exact_task269_binding(&self.binding_env, self.name_range, None))
        {
            return Err(
                SourceProofLocalGivenDescendantBindingError::InvalidDeclaration { binding: id },
            );
        }
        if let Some(scope) = &self.set_scope {
            let expected = source_descendant_set_binding_env(
                &self.base_binding_env,
                &self.lower_fingerprint,
                scope,
            )?;
            if Some(self.descendant_range) != self.lower_fingerprint.descendant_range()
                || self.descendant_context != BindingContextId::new(2)
            {
                return Err(SourceProofLocalGivenDescendantBindingError::InvalidDescendantContext);
            }
            if self.binding_env != expected
                || self.final_binding_fingerprint != self.binding_env.debug_text()
            {
                return Err(SourceProofLocalGivenDescendantBindingError::InvalidBindingEnvironment);
            }
            return Ok(());
        }
        if Some(self.descendant_range) != self.lower_fingerprint.descendant_range()
            || self.descendant_context != BindingContextId::new(2)
            || !exact_task269sdc_descendant_context(&self.binding_env, self.descendant_range)
        {
            return Err(SourceProofLocalGivenDescendantBindingError::InvalidDescendantContext);
        }
        let expected = extend_task269sdc_binding_env(
            &self.base_binding_env,
            &self.lower_fingerprint,
            self.proof_range,
            self.name_range,
            self.descendant_range,
        )?;
        if self.binding_env != expected
            || self.final_binding_fingerprint != self.binding_env.debug_text()
            || !exact_task269_binding(
                &self.binding_env,
                self.name_range,
                Some((BindingKind::GivenWitness, self.proof_range)),
            )
            || !exact_task269sdc_lookup_behavior(&self.binding_env)
        {
            return Err(SourceProofLocalGivenDescendantBindingError::InvalidBindingEnvironment);
        }
        Ok(())
    }

    pub(crate) fn validate_complete_installation(
        &self,
        source_id: SourceId,
        module_id: &ModuleId,
        installation_available: bool,
    ) -> Result<(), SourceProofLocalGivenDescendantBindingError> {
        self.validate_installation(source_id, module_id)?;
        if !installation_available {
            return Err(SourceProofLocalGivenDescendantBindingError::InvalidInstallation);
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct SourceProofLocalGivenDescendantBindingProducer;

impl SourceProofLocalGivenDescendantBindingProducer {
    pub fn build(
        input: SourceProofLocalGivenDescendantBindingHandoffInput,
        base_binding_env: &BindingEnv,
    ) -> Result<
        SourceProofLocalGivenDescendantBindingHandoff,
        SourceProofLocalGivenDescendantBindingError,
    > {
        if input.source_id != base_binding_env.source_id()
            || &input.module_id != base_binding_env.module_id()
        {
            return Err(SourceProofLocalGivenDescendantBindingError::InvalidTransaction);
        }
        validate_task269sdc_dependency(Task269sdcDependency {
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
        if !exact_task269c_base_binding_env(base_binding_env, &input.lower_fingerprint) {
            return Err(SourceProofLocalGivenDescendantBindingError::InvalidBaseBindingEnvironment);
        }
        if !exact_task269sdc_input_declaration(&input) {
            return Err(
                SourceProofLocalGivenDescendantBindingError::InvalidDeclaration {
                    binding: SourceProofLocalGivenBindingId::new(0),
                },
            );
        }
        if Some(input.descendant_range) != input.lower_fingerprint.descendant_range()
            || input.descendant_scope.path() != [0, 0]
        {
            return Err(SourceProofLocalGivenDescendantBindingError::InvalidDescendantContext);
        }
        let binding_env = extend_task269sdc_binding_env(
            base_binding_env,
            &input.lower_fingerprint,
            input.proof_range,
            input.name_range,
            input.descendant_range,
        )?;
        if !exact_task269sdc_lookup_behavior(&binding_env) {
            return Err(SourceProofLocalGivenDescendantBindingError::InvalidBindingEnvironment);
        }
        let base_binding_fingerprint = base_binding_env.debug_text();
        let final_binding_fingerprint = binding_env.debug_text();
        Ok(SourceProofLocalGivenDescendantBindingHandoff {
            source_id: input.source_id,
            module_id: input.module_id,
            lower_fingerprint: Box::new(input.lower_fingerprint),
            theorem_symbol: input.theorem_symbol,
            theorem_definition: input.theorem_definition,
            contribution: input.contribution,
            theorem_range: input.theorem_range,
            proof_range: input.proof_range,
            given_range: input.given_range,
            segment_range: input.segment_range,
            name_range: input.name_range,
            descendant_range: input.descendant_range,
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
            descendant_context: BindingContextId::new(2),
            set_scope: None,
        })
    }

    /// Retains the authenticated descendant Set names without inferring or expanding their values.
    pub fn build_set_bindings(
        input: SourceProofLocalGivenDescendantBindingHandoffInput,
        base_binding_env: &BindingEnv,
        scope: &ResolvedVariableScope,
    ) -> Result<
        SourceProofLocalGivenDescendantBindingHandoff,
        SourceProofLocalGivenDescendantBindingError,
    > {
        let mut handoff = Self::build(input, base_binding_env)?;
        handoff.binding_env =
            source_descendant_set_binding_env(base_binding_env, &handoff.lower_fingerprint, scope)?;
        handoff.final_binding_fingerprint = handoff.binding_env.debug_text();
        handoff.set_scope = Some(Box::new(scope.clone()));
        Ok(handoff)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum SourceProofLocalGivenDescendantBindingError {
    InvalidTransaction,
    DependencyMismatch,
    InvalidBaseBindingEnvironment,
    InvalidAggregate,
    InvalidDeclaration {
        binding: SourceProofLocalGivenBindingId,
    },
    InvalidDescendantContext,
    InvalidBindingEnvironment,
    InvalidInstallation,
}

impl fmt::Display for SourceProofLocalGivenDescendantBindingError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidTransaction => formatter
                .write_str("source proof-local given-descendant binding transaction is invalid"),
            Self::DependencyMismatch => formatter
                .write_str("source proof-local given-descendant binding dependency mismatch"),
            Self::InvalidBaseBindingEnvironment => formatter.write_str(
                "source proof-local given-descendant binding base binding environment is invalid",
            ),
            Self::InvalidAggregate => formatter
                .write_str("source proof-local given-descendant binding aggregate is invalid"),
            Self::InvalidDeclaration { binding } => write!(
                formatter,
                "source proof-local given-descendant binding {} is invalid",
                binding.index()
            ),
            Self::InvalidDescendantContext => formatter.write_str(
                "source proof-local given-descendant binding descendant context is invalid",
            ),
            Self::InvalidBindingEnvironment => formatter.write_str(
                "source proof-local given-descendant binding binding environment is invalid",
            ),
            Self::InvalidInstallation => formatter
                .write_str("source proof-local given-descendant binding installation is invalid"),
        }
    }
}

impl Error for SourceProofLocalGivenDescendantBindingError {}

#[derive(Debug, Clone, Copy)]
struct Task269sdcDependency<'a> {
    source_id: SourceId,
    module_id: &'a ModuleId,
    lower_fingerprint: &'a SourceProofLocalIntakeReceipt,
    theorem_symbol: &'a SymbolId,
    theorem_definition: DefinitionId,
    contribution: SourceContributionId,
    theorem_range: SourceRange,
    proof_range: SourceRange,
    given_range: SourceRange,
    segment_range: SourceRange,
    name_range: SourceRange,
}

fn validate_task269sdc_dependency(
    dependency: Task269sdcDependency<'_>,
) -> Result<(), SourceProofLocalGivenDescendantBindingError> {
    let expected_prefix = format!(
        "{}::{}::",
        dependency.module_id.package().as_str(),
        dependency.module_id.path().as_str()
    );
    let expected_local = format!(
        concat!(
            "contribution=0:namespace={}:owner=theorem#1:shell=theorem:kind=theorem:",
            "name=ProofLocalGivenDescendantCaptureSmoke:notation=_:arity=_:definition=theorem:",
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
        || !dependency.lower_fingerprint.matches(
            dependency.source_id,
            dependency.module_id,
            4,
            [
                dependency.theorem_range,
                dependency.proof_range,
                dependency.given_range,
                dependency.segment_range,
                dependency.name_range,
            ],
        )
    {
        return Err(SourceProofLocalGivenDescendantBindingError::DependencyMismatch);
    }
    Ok(())
}

fn exact_task269sdc_input_declaration(
    input: &SourceProofLocalGivenDescendantBindingHandoffInput,
) -> bool {
    input.source_ordinal == 1
        && input.local.spelling() == "y"
        && input.local.scope().path() == [0]
        && input.local.declaration_range() == input.name_range
        && input.local.visible_after_ordinal() == 1
        && input.recovery == SourceProofLocalGivenBindingRecovery::Normal
}

fn exact_task269sdc_descendant_context(
    binding_env: &BindingEnv,
    descendant_range: SourceRange,
) -> bool {
    let Some(context) = binding_env.contexts().get(BindingContextId::new(2)) else {
        return false;
    };
    context.id == BindingContextId::new(2)
        && context.owner
            == BindingContextOwner::SourceStatement {
                source_range: descendant_range,
            }
        && context.parent == Some(BindingContextId::new(1))
        && context.layer == BindingContextLayer::Block
        && context
            .lexical_scope
            .as_ref()
            .is_some_and(|scope| scope.path() == [0, 0])
        && context.bindings.is_empty()
        && context.visible_bindings == [BindingId::new(0), BindingId::new(1)]
        && context.recovery == BindingContextRecovery::Normal
}

fn extend_task269sdc_binding_env(
    base: &BindingEnv,
    receipt: &SourceProofLocalIntakeReceipt,
    proof_range: SourceRange,
    name_range: SourceRange,
    descendant_range: SourceRange,
) -> Result<BindingEnv, SourceProofLocalGivenDescendantBindingError> {
    if !exact_task269c_base_binding_env(base, receipt) {
        return Err(SourceProofLocalGivenDescendantBindingError::InvalidBaseBindingEnvironment);
    }
    let local = LocalTermBinding::new("y", LocalTermScope::new(vec![0]), name_range, 1);
    let mut bindings = base.bindings().clone();
    let binding = bindings.insert(BindingDraft::from_local_term(
        BindingContextId::new(1),
        BindingKind::GivenWitness,
        &local,
    ));
    if binding != BindingId::new(1) {
        return Err(SourceProofLocalGivenDescendantBindingError::InvalidBindingEnvironment);
    }
    let mut contexts = base.contexts().clone();
    let proof_context = contexts.insert(BindingContextDraft {
        owner: BindingContextOwner::SourceStatement {
            source_range: proof_range,
        },
        parent: Some(BindingContextId::new(0)),
        layer: BindingContextLayer::Proof,
        lexical_scope: Some(LocalTermScope::new(vec![0])),
        bindings: vec![binding],
        visible_bindings: vec![BindingId::new(0), binding],
        recovery: BindingContextRecovery::Normal,
    });
    if proof_context != BindingContextId::new(1) {
        return Err(SourceProofLocalGivenDescendantBindingError::InvalidBindingEnvironment);
    }
    let descendant_context = contexts.insert(BindingContextDraft {
        owner: BindingContextOwner::SourceStatement {
            source_range: descendant_range,
        },
        parent: Some(proof_context),
        layer: BindingContextLayer::Block,
        lexical_scope: Some(LocalTermScope::new(vec![0, 0])),
        bindings: Vec::new(),
        visible_bindings: vec![BindingId::new(0), binding],
        recovery: BindingContextRecovery::Normal,
    });
    if descendant_context != BindingContextId::new(2) {
        return Err(SourceProofLocalGivenDescendantBindingError::InvalidDescendantContext);
    }
    BindingEnv::try_new(BindingEnvParts {
        source_id: base.source_id(),
        module_id: base.module_id().clone(),
        contexts,
        bindings,
        diagnostics: base.diagnostics().clone(),
    })
    .map_err(|_| SourceProofLocalGivenDescendantBindingError::InvalidBindingEnvironment)
}

fn exact_task269sdc_lookup_behavior(binding_env: &BindingEnv) -> bool {
    let descendant = BindingLookupSite::new(
        "y",
        BindingContextId::new(2),
        Some(LocalTermScope::new(vec![0, 0])),
        2,
    );
    exact_task269g_lookup_behavior(binding_env)
        && binding_env.lookup(&descendant) == Ok(BindingLookupResult::Local(BindingId::new(1)))
}

// Equating owns the name/value roles; the enclosing Set statement remains a skeleton rule.
fn source_equating_roles<'a>(
    rows: &'a [SourceNode],
    equating: &SourceNode,
) -> Option<[&'a SourceNode; 2]> {
    if equating.kind() != &SurfaceNodeKind::Equating
        || source_contiguous_child_count(source_children(rows, equating.site())) != Some(3)
    {
        return None;
    }
    let name = source_child_at(rows, equating.site(), 0)?;
    let equals = source_child_at(rows, equating.site(), 1)?;
    let value = source_unique_child(rows, equating, SurfaceNodeKind::TermExpression)?;
    if !matches!(name.kind(), SurfaceNodeKind::Token(token)
        if token.kind == SurfaceTokenKind::Identifier && token.text.as_ref() == name.spelling())
        || !matches!(equals.kind(), SurfaceNodeKind::Token(token)
            if token.kind == SurfaceTokenKind::ReservedSymbol && token.text.as_ref() == "=")
        || value.child_ordinal() != 2
    {
        return None;
    }
    Some([name, value])
}

fn source_descendant_set_binding_env(
    base: &BindingEnv,
    receipt: &SourceProofLocalIntakeReceipt,
    scope: &ResolvedVariableScope,
) -> Result<BindingEnv, SourceProofLocalGivenDescendantBindingError> {
    use SurfaceNodeKind as K;
    use mizar_resolve::names::{
        SourceVariableBindingKind as B, SourceVariableReferenceKind as R, SourceVariableTypeRadix,
    };
    let invalid = || SourceProofLocalGivenDescendantBindingError::DependencyMismatch;
    let proof = receipt.role(K::ProofBlock).ok_or_else(invalid)?;
    let now = receipt.role(K::NowStatement).ok_or_else(invalid)?;
    let rows = &receipt.original;
    let sets = [1, 2].map(|ordinal| {
        let set = source_child_at(rows, now.site(), ordinal)?;
        let equating = source_unique_child(rows, set, K::Equating)?;
        source_equating_roles(rows, equating)
    });
    let [Some(first), Some(second)] = sets else {
        return Err(invalid());
    };
    let given = source_child_at(
        rows,
        receipt
            .role(K::QualifiedVariableSegment)
            .ok_or_else(invalid)?
            .site(),
        0,
    )
    .ok_or_else(invalid)?;
    let proof_scope = LocalTermScope::new(vec![
        u32::try_from(proof.child_ordinal()).map_err(|_| invalid())?,
    ]);
    let mut descendant_path = proof_scope.path().to_vec();
    descendant_path.push(u32::try_from(now.child_ordinal()).map_err(|_| invalid())?);
    let descendant_scope = LocalTermScope::new(descendant_path);
    if scope.source_id() != receipt.source_id
        || scope.module_id() != &receipt.module_id
        || scope.bindings().len() != 4
        || scope.references().len() != 2
        || scope.thesis().is_some()
        || !scope.statements().is_empty()
    {
        return Err(invalid());
    }
    for (index, ((name, kind), binding)) in [
        (receipt.reserve_name(), B::Reserve),
        (given, B::GivenWitness),
        (first[0], B::Set),
        (second[0], B::Set),
    ]
    .into_iter()
    .zip(scope.bindings())
    .enumerate()
    {
        let expected_scope = match index {
            0 => LocalTermScope::default(),
            1 => proof_scope.clone(),
            _ => descendant_scope.clone(),
        };
        if binding.kind() != kind
            || binding.range() != name.range()
            || binding.spelling() != name.spelling()
            || binding.ordinal() != index
            || binding.id().index() != index
            || binding.scope() != &expected_scope
            || binding.arity().is_some()
            || !binding.captures().is_empty()
            || if index < 2 {
                binding.declared_type().is_none_or(|ty| {
                    ty.radix() != SourceVariableTypeRadix::Set || !ty.attributes().is_empty()
                })
            } else {
                binding.declared_type().is_some()
            }
        {
            return Err(invalid());
        }
    }
    for (index, (roles, reference)) in [first, second]
        .into_iter()
        .zip(scope.references())
        .enumerate()
    {
        let rhs = source_unique_child(rows, roles[1], K::TermReference).ok_or_else(invalid)?;
        if reference.kind() != R::Term
            || reference.range() != rhs.range()
            || reference.spelling() != rhs.spelling()
            || reference.scope() != &descendant_scope
            || reference.ordinal() != index
            || reference.id().index() != index
            || reference.binding() != scope.bindings()[index + 1].id()
        {
            return Err(invalid());
        }
    }
    let mut bindings = base.bindings().clone();
    for binding in &scope.bindings()[1..] {
        let context = BindingContextId::new(if binding.kind() == B::GivenWitness {
            1
        } else {
            2
        });
        let local = LocalTermBinding::new(
            binding.spelling(),
            binding.scope().clone(),
            binding.range(),
            binding.ordinal(),
        );
        let mut draft = BindingDraft::from_local_term(
            context,
            if binding.kind() == B::GivenWitness {
                BindingKind::GivenWitness
            } else {
                BindingKind::LocalAbbreviation
            },
            &local,
        );
        if binding.kind() == B::Set {
            let reference = &scope.references()[binding.ordinal() - 2];
            let captured = bindings
                .get(BindingId::new(reference.binding().index()))
                .ok_or_else(invalid)?;
            draft.captured = CapturedFreeVariables::new(vec![captured.identity.clone()]);
        }
        bindings.insert(draft);
    }
    let mut contexts = base.contexts().clone();
    for (index, owner, lexical_scope, own, visible) in [
        (
            1,
            proof,
            proof_scope,
            vec![BindingId::new(1)],
            vec![BindingId::new(0), BindingId::new(1)],
        ),
        (
            2,
            now,
            descendant_scope,
            vec![BindingId::new(2), BindingId::new(3)],
            (0..4).map(BindingId::new).collect(),
        ),
    ] {
        contexts.insert(BindingContextDraft {
            owner: BindingContextOwner::SourceStatement {
                source_range: owner.range(),
            },
            parent: Some(BindingContextId::new(index - 1)),
            layer: if index == 1 {
                BindingContextLayer::Proof
            } else {
                BindingContextLayer::Block
            },
            lexical_scope: Some(lexical_scope),
            bindings: own,
            visible_bindings: visible,
            recovery: BindingContextRecovery::Normal,
        });
    }
    BindingEnv::try_new(BindingEnvParts {
        source_id: base.source_id(),
        module_id: base.module_id().clone(),
        contexts,
        bindings,
        diagnostics: base.diagnostics().clone(),
    })
    .map_err(|_| SourceProofLocalGivenDescendantBindingError::InvalidBindingEnvironment)
}

const fn binding_context_recovery_key(recovery: BindingContextRecovery) -> &'static str {
    match recovery {
        BindingContextRecovery::Normal => "normal",
        BindingContextRecovery::Recovered => "recovered",
        BindingContextRecovery::Degraded => "degraded",
    }
}

const fn given_binding_recovery_key(
    recovery: SourceProofLocalGivenBindingRecovery,
) -> &'static str {
    match recovery {
        SourceProofLocalGivenBindingRecovery::Normal => "normal",
    }
}

#[cfg(test)]
pub(crate) fn task286_source(
    source: SourceId,
    module: ModuleId,
    profile: usize,
    mutation: usize,
) -> TypedAst {
    use SurfaceNodeKind as K;
    use SurfaceTokenKind as T;
    use mizar_resolve::resolved_ast::SurfaceResolvedArena;
    let (name, theorem_formula, declaration, conclusion) = match profile {
        0 => (
            "FormulaStatementLetSmoke",
            "x = x",
            "let y be set;",
            "x = x",
        ),
        1 => (
            "FormulaStatementGivenSmoke",
            "thesis",
            "given y being set such that G: thesis;",
            "thesis",
        ),
        2 => (
            "FormulaStatementGivenSmoke",
            "thesis",
            "given y being set such that G: thesis;",
            "y = y",
        ),
        3 => (
            "ProofLocalGivenConditionUseSmoke",
            "thesis",
            "given y being set such that G: y = y;",
            "thesis",
        ),
        4 => (
            "ProofLocalGivenDescendantCaptureSmoke",
            "thesis",
            "given y being set;\n  now\n    set z = y;\n    set q = z;\n    thus thesis;\n  end;",
            "thesis",
        ),
        _ => unreachable!(),
    };
    let text = format!(
        "reserve x for set;\ntheorem {name}: {theorem_formula} proof\n  {declaration}\n  thus {conclusion};\nend;\n"
    );
    let mut b = SurfaceAstBuilder::new(source);
    let mut t = Vec::new();
    let mut cursor = 0;
    while cursor < text.len() {
        if text.as_bytes()[cursor].is_ascii_whitespace() {
            cursor += 1;
            continue;
        }
        let start = cursor;
        cursor += 1;
        if text.as_bytes()[start].is_ascii_alphanumeric() {
            while cursor < text.len() && text.as_bytes()[cursor].is_ascii_alphanumeric() {
                cursor += 1;
            }
        }
        let word = &text[start..cursor];
        let kind = match word {
            ";" | ":" | "=" => T::ReservedSymbol,
            "reserve" | "for" | "set" | "theorem" | "proof" | "let" | "be" | "being" | "thus"
            | "end" | "given" | "such" | "that" | "thesis" | "now" => T::ReservedWord,
            _ => T::Identifier,
        };
        let kind = if mutation == 2 && word == "y" {
            T::UserSymbol
        } else {
            kind
        };
        let range = range(source, start, cursor);
        let id = if mutation == 3 && word == "y" {
            b.add_recovered_token(kind, word, range)
        } else {
            b.add_token(kind, word, range)
        };
        t.push(((id, range), word));
    }
    fn branch(
        b: &mut SurfaceAstBuilder,
        kind: K,
        children: Vec<(SurfaceBuilderNodeId, SourceRange)>,
    ) -> (SurfaceBuilderNodeId, SourceRange) {
        let range = SourceRange {
            end: children.last().unwrap().1.end,
            ..children[0].1
        };
        (
            b.add_node(kind, range, children.into_iter().map(|c| c.0).collect()),
            range,
        )
    }
    fn formula(
        b: &mut SurfaceAstBuilder,
        t: &[((SurfaceBuilderNodeId, SourceRange), &str)],
        i: &mut usize,
    ) -> (SurfaceBuilderNodeId, SourceRange) {
        let body = if t[*i].1 == "thesis" {
            *i += 1;
            branch(
                b,
                K::FormulaConstant(SurfaceFormulaConstant::Thesis),
                vec![t[*i - 1].0],
            )
        } else {
            let mut terms = Vec::new();
            for index in [*i, *i + 2] {
                let reference = branch(b, K::TermReference, vec![t[index].0]);
                terms.push(branch(b, K::TermExpression, vec![reference]));
            }
            *i += 3;
            branch(
                b,
                K::BuiltinPredicateApplication,
                vec![terms[0], t[*i - 2].0, terms[1]],
            )
        };
        branch(b, K::FormulaExpression, vec![body])
    }
    fn statement(
        b: &mut SurfaceAstBuilder,
        t: &[((SurfaceBuilderNodeId, SourceRange), &str)],
        i: &mut usize,
        mutation: usize,
    ) -> (SurfaceBuilderNodeId, SourceRange) {
        let start = *i;
        let mut children = vec![t[*i].0];
        *i += 1;
        let kind = match t[start].1 {
            "let" | "given" => {
                let head = branch(b, K::TypeHead, vec![t[*i + 2].0]);
                let ty = branch(b, K::TypeExpression, vec![head]);
                children.push(branch(
                    b,
                    if mutation == 1 {
                        K::TermList
                    } else {
                        K::QualifiedVariableSegment
                    },
                    vec![t[*i].0, t[*i + 1].0, ty],
                ));
                *i += 3;
                if t[*i].1 == "such" {
                    children.push(t[*i].0);
                    *i += 1;
                    let begin = *i;
                    *i += 3;
                    let f = formula(b, t, i);
                    let p = branch(b, K::Proposition, vec![t[begin + 1].0, t[begin + 2].0, f]);
                    children.push(branch(b, K::ConditionList, vec![t[begin].0, p]));
                }
                if t[start].1 == "let" {
                    K::LetStatement
                } else {
                    K::GivenStatement
                }
            }
            "thus" => {
                let f = formula(b, t, i);
                children.push(branch(b, K::Proposition, vec![f]));
                K::ConclusionStatement
            }
            "set" => {
                let reference = branch(b, K::TermReference, vec![t[*i + 2].0]);
                let term = branch(b, K::TermExpression, vec![reference]);
                children.push(branch(b, K::Equating, vec![t[*i].0, t[*i + 1].0, term]));
                *i += 3;
                K::SetStatement
            }
            "now" => {
                while t[*i].1 != "end" {
                    children.push(statement(b, t, i, mutation));
                }
                children.push(t[*i].0);
                *i += 1;
                K::NowStatement
            }
            _ => unreachable!(),
        };
        children.push(t[*i].0);
        *i += 1;
        branch(b, kind, children)
    }
    let head = branch(&mut b, K::TypeHead, vec![t[3].0]);
    let ty = branch(&mut b, K::TypeExpression, vec![head]);
    let segment = branch(&mut b, K::ReserveSegment, vec![t[1].0, t[2].0, ty]);
    let reserve = branch(&mut b, K::ReserveItem, vec![t[0].0, segment, t[4].0]);
    let mut i = 8;
    let f = formula(&mut b, &t, &mut i);
    let mut proof_children = vec![t[i].0];
    i += 1;
    while t[i].1 != "end" {
        proof_children.push(statement(&mut b, &t, &mut i, mutation));
    }
    proof_children.push(t[i].0);
    i += 1;
    if mutation == 4 {
        let range = SourceRange {
            start: proof_children[1].1.end,
            ..proof_children[1].1
        };
        proof_children.insert(2, (b.add_node(K::LetStatement, range, vec![]), range));
    }
    let proof = branch(&mut b, K::ProofBlock, proof_children);
    let theorem = branch(
        &mut b,
        K::TheoremItem,
        vec![t[5].0, t[6].0, t[7].0, f, proof, t[i].0],
    );
    let items = branch(&mut b, K::ItemList, vec![reserve, theorem]);
    let compilation = branch(&mut b, K::CompilationUnit, vec![items]);
    let root = branch(
        &mut b,
        K::Root,
        t.iter().map(|t| t.0).chain(Some(compilation)).collect(),
    );
    let ast = b.finish(Some(root.0), None);
    let lowered = SurfaceResolvedArena::lower(&ast, &module).unwrap();
    let nodes = lowered
        .arena()
        .iter()
        .map(|(id, row)| {
            TypedNode::new("syntax", row.origin().anchor().clone())
                .with_recovery(if row.origin().is_recovered() {
                    NodeRecoveryState::Recovered
                } else {
                    NodeRecoveryState::Normal
                })
                .with_resolved_node(id)
                .with_children(
                    row.children()
                        .iter()
                        .map(|id| TypedNodeId::new(id.index()))
                        .collect(),
                )
        })
        .collect();
    use crate::typed_ast::{
        CoercionTable, InitialObligationTable, LocalTypeContextTable, TypeDiagnosticTable,
        TypeFactTable, TypeTable, TypedAstParts,
    };
    let mut typed = TypedAst::try_new(TypedAstParts {
        source_id: source,
        module_id: module,
        resolved_root: Some(lowered.arena().root()),
        source_context: None,
        source_type: None,
        source_attribute: None,
        nodes: TypedArena::try_new(
            Some(TypedNodeId::new(lowered.arena().root().index())),
            nodes,
        )
        .unwrap(),
        contexts: LocalTypeContextTable::new(),
        types: TypeTable::new(),
        facts: TypeFactTable::new(),
        coercions: CoercionTable::new(),
        initial_obligations: InitialObligationTable::new(),
        diagnostics: TypeDiagnosticTable::new(),
    })
    .unwrap();
    typed.install_source_nodes(&lowered).unwrap();
    typed
}

#[cfg(test)]
use mizar_syntax::{SurfaceAstBuilder, SurfaceBuilderNodeId};

#[cfg(test)]
pub(crate) fn task286_receipt(
    source: SourceId,
    module: ModuleId,
    profile: usize,
) -> SourceProofLocalIntakeReceipt {
    let typed = task286_source(source, module, profile, 0);
    let declaration = typed
        .source_node_entries()
        .unwrap()
        .find(|row| {
            matches!(
                row.kind(),
                SurfaceNodeKind::LetStatement | SurfaceNodeKind::GivenStatement
            )
        })
        .unwrap()
        .site();
    SourceProofLocalIntakeReceipt::from_source(&typed, declaration).unwrap()
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
    fn task286_receipt_origin_grammar_and_fingerprint_are_independent() {
        let f = fixture();
        for profile in 0..5 {
            let source = task286_source(f.source, f.module.clone(), profile, 0);
            let before = source.clone();
            let receipt = task286_receipt(f.source, f.module.clone(), profile);
            let actual =
                SourceProofLocalIntakeReceipt::from_source(&source, receipt.declaration).unwrap();
            assert_eq!(actual.fingerprint(), receipt.fingerprint());
            assert!(Arc::ptr_eq(
                &actual.original,
                source.source_nodes().unwrap()
            ));
            assert_eq!(format!("{actual:?}"), "SourceProofLocalIntakeReceipt");
            assert_eq!(
                actual.fingerprint(),
                task286_receipt(other_source_id(), f.module.clone(), profile).fingerprint()
            );
            assert!(
                SourceProofLocalIntakeReceipt::from_source(&source, actual.root_row().site())
                    .is_none()
            );
            assert!(
                SourceProofLocalIntakeReceipt::from_source(
                    &empty_typed(f.source, f.module.clone()),
                    receipt.declaration
                )
                .is_none()
            );
            assert_eq!(source, before);
        }
        // Independently spell the versioned module/site header and first original row record.
        let r = &f.input.lower_fingerprint;
        let root = r.root_row();
        let header = format!(
            "proof-local-intake-v1:\"pkg\":\"task269c\":{};",
            r.declaration.index()
        );
        let first = format!(
            "{:?};",
            (
                root.site().index(),
                "Root",
                None::<usize>,
                0,
                0,
                0,
                99,
                NodeRecoveryState::Normal,
                root.spelling()
            )
        );
        assert!(r.fingerprint().starts_with(&(header + &first)));
        for mutation in 1..5 {
            let source = task286_source(f.source, f.module.clone(), 0, mutation);
            assert_eq!(
                source.source_nodes().unwrap()[0].spelling(),
                r.root_row().spelling()
            );
            assert!(SourceProofLocalIntakeReceipt::from_source(&source, r.declaration).is_none());
        }
        let mut foreign = f.input.clone();
        foreign.lower_fingerprint = task286_receipt(other_source_id(), f.module.clone(), 0);
        assert_eq!(
            SourceProofLocalLetBindingProducer::build(foreign, &empty_base(&f)),
            Err(SourceProofLocalLetBindingError::DependencyMismatch)
        );
    }

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
            task286_receipt(fixture.source, fixture.module.clone(), 0).fingerprint(),
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
        wrong_dependency
            .lower_fingerprint
            .fingerprint
            .push_str("corrupt");
        assert_eq!(
            SourceProofLocalLetBindingProducer::build(wrong_dependency, &empty_base(&fixture)),
            Err(SourceProofLocalLetBindingError::DependencyMismatch)
        );
        let mut dependency_inputs = Vec::new();
        let mut input = fixture.input.clone();
        input.lower_fingerprint.declaration = input
            .lower_fingerprint
            .role(SurfaceNodeKind::QualifiedVariableSegment)
            .unwrap()
            .site();
        input.lower_fingerprint.fingerprint = input.lower_fingerprint.structural_fingerprint();
        dependency_inputs.push(input);
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
        for change_name in [true, false] {
            let mut coherent = handoff.clone();
            if change_name {
                coherent.name_range.end += 1;
            } else {
                coherent.proof_range.end += 1;
            }
            coherent.binding_env = binding_env_with_local_ranges(
                &coherent.binding_env,
                coherent.proof_range,
                coherent.name_range,
            );
            coherent.final_binding_fingerprint = coherent.binding_env.debug_text();
            assert_eq!(
                coherent.validate_installation(fixture.source, &fixture.module),
                Err(SourceProofLocalLetBindingError::DependencyMismatch)
            );
            assert!(Arc::ptr_eq(
                &coherent.lower_fingerprint.original,
                &handoff.lower_fingerprint.original
            ));
            let mut typed = empty_typed(fixture.source, fixture.module.clone());
            assert_eq!(
                typed
                    .clone()
                    .with_source_proof_local_let_binding(coherent.clone()),
                Err(TypedAstError::InvalidSourceProofLocalLetBinding)
            );
            typed.inject_source_proof_local_let_binding_for_test(coherent);
            assert_eq!(
                assemble_empty(&typed),
                Err(ResolvedTypedAstError::InvalidSourceProofLocalLetBinding)
            );
        }
        assert_eq!(
            handoff.validate_installation(fixture.source, &fixture.module),
            Ok(())
        );

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
        assert!(exact_task269_binding(
            handoff.binding_env(),
            handoff.name_range(),
            Some((BindingKind::GivenWitness, handoff.proof_range()))
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
            task286_receipt(fixture.source, fixture.module.clone(), 1).fingerprint(),
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
        transaction_first
            .lower_fingerprint
            .fingerprint
            .push_str("corrupt");
        assert_eq!(
            SourceProofLocalGivenBindingProducer::build(
                transaction_first,
                &empty_given_base(&fixture),
            ),
            Err(SourceProofLocalGivenBindingError::InvalidTransaction)
        );

        let mut dependency_first = fixture.input.clone();
        dependency_first
            .lower_fingerprint
            .fingerprint
            .push_str("corrupt");
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
        for change_name in [true, false] {
            let mut coherent = handoff.clone();
            if change_name {
                coherent.name_range.end += 1;
            } else {
                coherent.proof_range.end += 1;
            }
            coherent.binding_env = binding_env_with_local_ranges(
                &coherent.binding_env,
                coherent.proof_range,
                coherent.name_range,
            );
            coherent.final_binding_fingerprint = coherent.binding_env.debug_text();
            assert_eq!(
                coherent.validate_installation(fixture.source, &fixture.module),
                Err(SourceProofLocalGivenBindingError::DependencyMismatch)
            );
        }
        assert_eq!(
            handoff.validate_installation(fixture.source, &fixture.module),
            Ok(())
        );

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
        assert_eq!(
            handoff.lower_fingerprint(),
            fixture.input.lower_fingerprint.fingerprint()
        );
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
        assert!(exact_task269g_output_binding(row));
        assert_eq!(
            handoff.bindings().iter().collect::<Vec<_>>(),
            [(SourceProofLocalGivenBindingId::new(0), row)]
        );
        assert!(exact_task269_binding(
            handoff.binding_env(),
            handoff.name_range(),
            Some((BindingKind::GivenWitness, handoff.proof_range()))
        ));
        assert!(exact_task269g_lookup_behavior(handoff.binding_env()));
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
            fixture.input.lower_fingerprint.fingerprint(),
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
        input.lower_fingerprint.fingerprint.push('!');
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
        wrong_transaction_and_dependency
            .lower_fingerprint
            .fingerprint = "corrupt".to_owned();
        assert_eq!(
            SourceProofLocalGivenUseBindingProducer::build(
                wrong_transaction_and_dependency,
                &fixture.base,
            ),
            Err(SourceProofLocalGivenUseBindingError::InvalidTransaction)
        );
        let mut wrong_dependency_and_declaration = fixture.input.clone();
        wrong_dependency_and_declaration
            .lower_fingerprint
            .fingerprint = "corrupt".to_owned();
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
        for change_name in [true, false] {
            let mut coherent = handoff.clone();
            if change_name {
                coherent.name_range.end += 1;
            } else {
                coherent.proof_range.end += 1;
            }
            coherent.binding_env = binding_env_with_local_ranges(
                &coherent.binding_env,
                coherent.proof_range,
                coherent.name_range,
            );
            coherent.final_binding_fingerprint = coherent.binding_env.debug_text();
            assert_eq!(
                coherent.validate_installation(fixture.source, &fixture.module),
                Err(SourceProofLocalGivenUseBindingError::DependencyMismatch)
            );
        }
        assert_eq!(
            handoff.validate_installation(fixture.source, &fixture.module),
            Ok(())
        );

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
        assert_eq!(
            handoff.lower_fingerprint(),
            fixture.input.lower_fingerprint.fingerprint()
        );
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
        assert!(exact_task269g_output_binding(row));
        assert_eq!(
            handoff.bindings().iter().collect::<Vec<_>>(),
            [(SourceProofLocalGivenBindingId::new(0), row)]
        );
        assert!(exact_task269_binding(
            handoff.binding_env(),
            handoff.name_range(),
            Some((BindingKind::GivenWitness, handoff.proof_range()))
        ));
        assert!(exact_task269g_lookup_behavior(handoff.binding_env()));
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
            fixture.input.lower_fingerprint.fingerprint(),
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
        input.lower_fingerprint.fingerprint.push('!');
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
        input.lower_fingerprint.fingerprint = input
            .lower_fingerprint
            .fingerprint
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
        for change_name in [true, false] {
            let mut coherent = handoff.clone();
            if change_name {
                coherent.name_range.end += 1;
            } else {
                coherent.proof_range.end += 1;
            }
            coherent.binding_env = binding_env_with_local_ranges(
                &coherent.binding_env,
                coherent.proof_range,
                coherent.name_range,
            );
            coherent.final_binding_fingerprint = coherent.binding_env.debug_text();
            assert_eq!(
                coherent.validate_installation(fixture.source, &fixture.module),
                Err(SourceProofLocalGivenConditionBindingError::DependencyMismatch)
            );
        }
        assert_eq!(
            handoff.validate_installation(fixture.source, &fixture.module),
            Ok(())
        );

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
        let input = SourceProofLocalGivenConditionBindingHandoffInput {
            source_id: source,
            module_id: module.clone(),
            lower_fingerprint: task286_receipt(source, module.clone(), 3),
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
        let input = SourceProofLocalGivenUseBindingHandoffInput {
            source_id: source,
            module_id: module.clone(),
            lower_fingerprint: task286_receipt(source, module.clone(), 2),
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

    #[test]
    fn source_proof_local_given_descendant_binding_is_exact_and_installed() {
        let fixture = task269sdc_fixture();
        let handoff = SourceProofLocalGivenDescendantBindingProducer::build(
            fixture.input.clone(),
            &fixture.base,
        )
        .expect("Task269SDC exact checker transaction");
        assert_eq!(handoff.source_id(), fixture.source);
        assert_eq!(handoff.module_id(), &fixture.module);
        assert_eq!(
            handoff.lower_fingerprint(),
            fixture.input.lower_fingerprint.fingerprint()
        );
        assert_eq!(handoff.theorem_symbol(), &fixture.input.theorem_symbol);
        assert_eq!(
            handoff.theorem_definition(),
            fixture.input.theorem_definition
        );
        assert_eq!(handoff.contribution(), fixture.input.contribution);
        assert_eq!(handoff.theorem_range(), range(fixture.source, 19, 179));
        assert_eq!(handoff.proof_range(), range(fixture.source, 73, 178));
        assert_eq!(handoff.given_range(), range(fixture.source, 81, 99));
        assert_eq!(handoff.segment_range(), range(fixture.source, 87, 98));
        assert_eq!(handoff.name_range(), range(fixture.source, 87, 88));
        assert_eq!(handoff.descendant_range(), range(fixture.source, 102, 159));
        assert_eq!(handoff.base_binding_env(), &fixture.base);
        assert_eq!(
            handoff.base_binding_fingerprint(),
            fixture.base.debug_text()
        );
        assert_eq!(handoff.descendant_context(), BindingContextId::new(2));
        assert_eq!(
            (
                handoff.binding_env().contexts().len(),
                handoff.binding_env().bindings().len(),
                handoff.binding_env().diagnostics().len(),
            ),
            (3, 2, 0)
        );
        assert!(exact_task269_binding(
            handoff.binding_env(),
            handoff.name_range(),
            Some((BindingKind::GivenWitness, handoff.proof_range()))
        ));
        assert!(exact_task269sdc_descendant_context(
            handoff.binding_env(),
            handoff.descendant_range()
        ));
        assert!(exact_task269sdc_lookup_behavior(handoff.binding_env()));
        let row = handoff
            .bindings()
            .get(SourceProofLocalGivenBindingId::new(0))
            .expect("Task269SDC dense binding row");
        assert!(exact_task269g_output_binding(row));
        let expected_debug = format!(
            concat!(
                "source-proof-local-given-descendant-binding-debug-v1\n",
                "module: {}::{}\n",
                "lower-fingerprint: {:?}\n",
                "theorem symbol={:?} definition=0 contribution=0 range=19..179 proof=73..178\n",
                "given range=81..99 segment=87..98 name=87..88 source_ordinal=1\n",
                "base-binding-fingerprint: {:?}\n",
                "binding#0 binding=1 context=1 source_ordinal=1 visible_after=1 recovery=normal\n",
                "descendant range=102..159 context=2 parent=1 scope=[0,0] recovery=normal\n",
                "final-binding-fingerprint: {:?}\n",
            ),
            fixture.module.package().as_str(),
            fixture.module.path().as_str(),
            fixture.input.lower_fingerprint.fingerprint(),
            fixture.input.theorem_symbol.fqn().as_str(),
            handoff.base_binding_fingerprint(),
            handoff.final_binding_fingerprint(),
        );
        assert_eq!(handoff.debug_text(), expected_debug);
        assert!(handoff.debug_text().ends_with('\n'));
        assert!(!handoff.debug_text().ends_with("\n\n"));

        let empty = empty_typed(fixture.source, fixture.module.clone());
        let absent_debug = empty.debug_text();
        let typed = empty
            .clone()
            .with_source_proof_local_given_descendant_binding(handoff.clone())
            .expect("Task269SDC Typed owner");
        assert_eq!(
            typed.source_proof_local_given_descendant_binding(),
            Some(&handoff)
        );
        assert_eq!(
            typed
                .debug_text()
                .matches("source-proof-local-given-descendant-binding-debug-v1")
                .count(),
            1
        );
        assert!(!absent_debug.contains("source-proof-local-given-descendant-binding-debug-v1"));
        assert_eq!(
            typed
                .clone()
                .with_source_proof_local_given_descendant_binding(handoff.clone()),
            Err(TypedAstError::InvalidSourceProofLocalGivenDescendantBinding)
        );
        let resolved = assemble_empty(&typed).expect("Task269SDC Resolved owner");
        assert_eq!(
            resolved.source_proof_local_given_descendant_binding(),
            Some(&handoff)
        );
        assert_eq!(
            resolved
                .debug_text()
                .matches("source-proof-local-given-descendant-binding-debug-v1")
                .count(),
            1
        );
    }

    #[test]
    fn source_proof_local_given_descendant_binding_rejects_every_corruption_in_precedence() {
        let fixture = task269sdc_fixture();

        let mut transaction = fixture.input.clone();
        transaction.source_id = other_source_id();
        transaction.lower_fingerprint.fingerprint.push('!');
        assert_eq!(
            SourceProofLocalGivenDescendantBindingProducer::build(transaction, &fixture.base),
            Err(SourceProofLocalGivenDescendantBindingError::InvalidTransaction)
        );
        let mut transaction = fixture.input.clone();
        transaction.module_id = ModuleId::new(PackageId::new("pkg"), ModulePath::new("wrong"));
        assert_eq!(
            SourceProofLocalGivenDescendantBindingProducer::build(transaction, &fixture.base),
            Err(SourceProofLocalGivenDescendantBindingError::InvalidTransaction)
        );

        let mut dependency_inputs = Vec::new();
        let mut input = fixture.input.clone();
        input.lower_fingerprint.fingerprint.push('!');
        dependency_inputs.push(input);
        let mut input = fixture.input.clone();
        input.theorem_symbol = SymbolId::new(
            fixture.module.clone(),
            LocalSymbolId::new("wrong"),
            FullyQualifiedName::new("pkg::task269sdc::wrong"),
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
                SourceProofLocalGivenDescendantBindingProducer::build(input, &fixture.base),
                Err(SourceProofLocalGivenDescendantBindingError::DependencyMismatch)
            );
        }

        let mut wrong_base_and_declaration = fixture.input.clone();
        wrong_base_and_declaration.source_ordinal = 2;
        assert_eq!(
            SourceProofLocalGivenDescendantBindingProducer::build(
                wrong_base_and_declaration,
                &task269sdc_empty_base(&fixture),
            ),
            Err(SourceProofLocalGivenDescendantBindingError::InvalidBaseBindingEnvironment)
        );
        for local in [
            LocalTermBinding::new(
                "z",
                LocalTermScope::new(vec![0]),
                range(fixture.source, 87, 88),
                1,
            ),
            LocalTermBinding::new(
                "y",
                LocalTermScope::new(vec![1]),
                range(fixture.source, 87, 88),
                1,
            ),
            LocalTermBinding::new(
                "y",
                LocalTermScope::new(vec![0]),
                range(fixture.source, 86, 88),
                1,
            ),
            LocalTermBinding::new(
                "y",
                LocalTermScope::new(vec![0]),
                range(fixture.source, 87, 88),
                2,
            ),
        ] {
            let mut input = fixture.input.clone();
            input.local = local;
            assert_eq!(
                SourceProofLocalGivenDescendantBindingProducer::build(input, &fixture.base),
                Err(
                    SourceProofLocalGivenDescendantBindingError::InvalidDeclaration {
                        binding: SourceProofLocalGivenBindingId::new(0),
                    }
                )
            );
        }
        let mut input = fixture.input.clone();
        input.source_ordinal = 2;
        assert_eq!(
            SourceProofLocalGivenDescendantBindingProducer::build(input, &fixture.base),
            Err(
                SourceProofLocalGivenDescendantBindingError::InvalidDeclaration {
                    binding: SourceProofLocalGivenBindingId::new(0),
                }
            )
        );
        let mut wrong_descendant_range = fixture.input.clone();
        wrong_descendant_range.descendant_range.end += 1;
        let mut wrong_descendant_scope = fixture.input.clone();
        wrong_descendant_scope.descendant_scope = LocalTermScope::new(vec![0, 1]);
        for input in [wrong_descendant_range, wrong_descendant_scope] {
            assert_eq!(
                SourceProofLocalGivenDescendantBindingProducer::build(input, &fixture.base),
                Err(SourceProofLocalGivenDescendantBindingError::InvalidDescendantContext)
            );
        }

        let handoff = SourceProofLocalGivenDescendantBindingProducer::build(
            fixture.input.clone(),
            &fixture.base,
        )
        .expect("Task269SDC valid handoff before replay mutations");
        for change_name in [true, false] {
            let mut coherent = handoff.clone();
            if change_name {
                coherent.name_range.end += 1;
            } else {
                coherent.proof_range.end += 1;
            }
            coherent.binding_env = binding_env_with_local_ranges(
                &coherent.binding_env,
                coherent.proof_range,
                coherent.name_range,
            );
            coherent.final_binding_fingerprint = coherent.binding_env.debug_text();
            assert_eq!(
                coherent.validate_installation(fixture.source, &fixture.module),
                Err(SourceProofLocalGivenDescendantBindingError::DependencyMismatch)
            );
        }
        assert_eq!(
            handoff.validate_installation(fixture.source, &fixture.module),
            Ok(())
        );

        let mut transaction = handoff.clone();
        transaction.source_id = other_source_id();
        transaction.lower_fingerprint.fingerprint.push('!');
        assert_eq!(
            transaction.validate_installation(fixture.source, &fixture.module),
            Err(SourceProofLocalGivenDescendantBindingError::InvalidTransaction)
        );
        let mut transaction = handoff.clone();
        transaction.module_id = ModuleId::new(PackageId::new("pkg"), ModulePath::new("wrong"));
        assert_eq!(
            transaction.validate_installation(fixture.source, &fixture.module),
            Err(SourceProofLocalGivenDescendantBindingError::InvalidTransaction)
        );
        for mutate in 0..9 {
            let mut dependency = handoff.clone();
            match mutate {
                0 => dependency.lower_fingerprint.fingerprint.push('!'),
                1 => {
                    dependency.theorem_symbol = SymbolId::new(
                        fixture.module.clone(),
                        LocalSymbolId::new("wrong"),
                        FullyQualifiedName::new("pkg::task269sdc::wrong"),
                    )
                }
                2 => dependency.theorem_definition = fixture.other_definition,
                3 => dependency.contribution = fixture.other_contribution,
                4 => dependency.theorem_range.end += 1,
                5 => dependency.proof_range.end += 1,
                6 => dependency.given_range.end += 1,
                7 => dependency.segment_range.end += 1,
                8 => dependency.name_range.end += 1,
                _ => unreachable!(),
            }
            dependency.base_binding_fingerprint = "wrong".to_owned();
            assert_eq!(
                dependency.validate_installation(fixture.source, &fixture.module),
                Err(SourceProofLocalGivenDescendantBindingError::DependencyMismatch)
            );
        }
        let mut coherent_dependency = handoff.clone();
        coherent_dependency.theorem_symbol = SymbolId::new(
            fixture.module.clone(),
            LocalSymbolId::new("coherent-wrong"),
            FullyQualifiedName::new("pkg::task269sdc::CoherentWrong"),
        );
        coherent_dependency.lower_fingerprint.fingerprint = coherent_dependency
            .lower_fingerprint
            .structural_fingerprint();
        assert_eq!(
            coherent_dependency.validate_installation(fixture.source, &fixture.module),
            Err(SourceProofLocalGivenDescendantBindingError::DependencyMismatch)
        );
        for inexact_base in task269sdc_inexact_bases(&fixture) {
            let mut base = handoff.clone();
            base.base_binding_env = inexact_base;
            base.base_binding_fingerprint = base.base_binding_env.debug_text();
            base.bindings.rows.clear();
            assert_eq!(
                base.validate_installation(fixture.source, &fixture.module),
                Err(SourceProofLocalGivenDescendantBindingError::InvalidBaseBindingEnvironment)
            );
        }
        let mut base = handoff.clone();
        base.base_binding_fingerprint = "wrong".to_owned();
        base.bindings.rows.clear();
        assert_eq!(
            base.validate_installation(fixture.source, &fixture.module),
            Err(SourceProofLocalGivenDescendantBindingError::InvalidBaseBindingEnvironment)
        );
        let mut aggregate = handoff.clone();
        aggregate.bindings.rows.clear();
        aggregate.descendant_range.end += 1;
        assert_eq!(
            aggregate.validate_installation(fixture.source, &fixture.module),
            Err(SourceProofLocalGivenDescendantBindingError::InvalidAggregate)
        );
        for mutate in 0..4 {
            let mut declaration = handoff.clone();
            match mutate {
                0 => declaration.bindings.rows[0].binding = BindingId::new(0),
                1 => declaration.bindings.rows[0].binding_context = BindingContextId::new(0),
                2 => declaration.bindings.rows[0].source_ordinal = 2,
                3 => declaration.bindings.rows[0].visible_after_ordinal = 2,
                _ => unreachable!(),
            }
            declaration.descendant_range.end += 1;
            assert_eq!(
                declaration.validate_installation(fixture.source, &fixture.module),
                Err(
                    SourceProofLocalGivenDescendantBindingError::InvalidDeclaration {
                        binding: SourceProofLocalGivenBindingId::new(0),
                    }
                )
            );
        }
        for mut declaration in task269sdc_inexact_declaration_handoffs(&handoff, fixture.source) {
            declaration.descendant_range.end += 1;
            assert_eq!(
                declaration.validate_installation(fixture.source, &fixture.module),
                Err(
                    SourceProofLocalGivenDescendantBindingError::InvalidDeclaration {
                        binding: SourceProofLocalGivenBindingId::new(0),
                    }
                )
            );
        }
        let mut descendant = handoff.clone();
        descendant.descendant_range.end += 1;
        descendant.final_binding_fingerprint = "wrong".to_owned();
        assert_eq!(
            descendant.validate_installation(fixture.source, &fixture.module),
            Err(SourceProofLocalGivenDescendantBindingError::InvalidDescendantContext)
        );
        let mut descendant = handoff.clone();
        descendant.descendant_context = BindingContextId::new(1);
        assert_eq!(
            descendant.validate_installation(fixture.source, &fixture.module),
            Err(SourceProofLocalGivenDescendantBindingError::InvalidDescendantContext)
        );
        let mut coherent_range_controls = 0;
        for mut descendant in task269sdc_inexact_descendant_context_handoffs(&handoff) {
            if let BindingContextOwner::SourceStatement { source_range } = &descendant
                .binding_env
                .contexts()
                .get(BindingContextId::new(2))
                .unwrap()
                .owner
                && *source_range != handoff.descendant_range
            {
                coherent_range_controls += 1;
                descendant.descendant_range = *source_range;
                descendant.final_binding_fingerprint = descendant.binding_env.debug_text();
                assert_eq!(
                    descendant.validate_installation(fixture.source, &fixture.module),
                    Err(SourceProofLocalGivenDescendantBindingError::InvalidDescendantContext)
                );
                descendant.descendant_range = handoff.descendant_range;
            }
            descendant.final_binding_fingerprint = "wrong".to_owned();
            assert_eq!(
                descendant.validate_installation(fixture.source, &fixture.module),
                Err(SourceProofLocalGivenDescendantBindingError::InvalidDescendantContext)
            );
        }
        assert_eq!(coherent_range_controls, 1);
        assert_eq!(
            handoff.validate_installation(fixture.source, &fixture.module),
            Ok(())
        );
        let mut descendant = handoff.clone();
        descendant.binding_env = task269sdc_environment_with_generated_descendant(&handoff);
        descendant.final_binding_fingerprint = descendant.binding_env.debug_text();
        assert_eq!(
            descendant.validate_installation(fixture.source, &fixture.module),
            Err(SourceProofLocalGivenDescendantBindingError::InvalidDescendantContext)
        );
        let mut environment = handoff.clone();
        environment
            .binding_env
            .binding_mut_for_test(BindingId::new(1))
            .expect("Task269SDC binding row")
            .status = BindingStatus::Reserved;
        environment.final_binding_fingerprint = environment.binding_env.debug_text();
        assert_eq!(
            environment.validate_installation(fixture.source, &fixture.module),
            Err(SourceProofLocalGivenDescendantBindingError::InvalidBindingEnvironment)
        );
        for environment in task269sdc_inexact_final_environment_handoffs(&handoff, fixture.source) {
            assert_eq!(
                environment.validate_installation(fixture.source, &fixture.module),
                Err(SourceProofLocalGivenDescendantBindingError::InvalidBindingEnvironment)
            );
        }
        let mut environment = handoff.clone();
        environment.final_binding_fingerprint = "wrong".to_owned();
        assert_eq!(
            environment.validate_installation(fixture.source, &fixture.module),
            Err(SourceProofLocalGivenDescendantBindingError::InvalidBindingEnvironment)
        );
        assert_eq!(
            handoff.validate_complete_installation(fixture.source, &fixture.module, false),
            Err(SourceProofLocalGivenDescendantBindingError::InvalidInstallation)
        );
        let mut invalid_before_unavailable = handoff;
        invalid_before_unavailable.final_binding_fingerprint = "wrong".to_owned();
        assert_eq!(
            invalid_before_unavailable.validate_complete_installation(
                fixture.source,
                &fixture.module,
                false,
            ),
            Err(SourceProofLocalGivenDescendantBindingError::InvalidBindingEnvironment)
        );

        for (error, expected) in [
            (
                SourceProofLocalGivenDescendantBindingError::InvalidTransaction,
                "source proof-local given-descendant binding transaction is invalid".to_owned(),
            ),
            (
                SourceProofLocalGivenDescendantBindingError::DependencyMismatch,
                "source proof-local given-descendant binding dependency mismatch".to_owned(),
            ),
            (
                SourceProofLocalGivenDescendantBindingError::InvalidBaseBindingEnvironment,
                "source proof-local given-descendant binding base binding environment is invalid"
                    .to_owned(),
            ),
            (
                SourceProofLocalGivenDescendantBindingError::InvalidAggregate,
                "source proof-local given-descendant binding aggregate is invalid".to_owned(),
            ),
            (
                SourceProofLocalGivenDescendantBindingError::InvalidDeclaration {
                    binding: SourceProofLocalGivenBindingId::new(0),
                },
                "source proof-local given-descendant binding 0 is invalid".to_owned(),
            ),
            (
                SourceProofLocalGivenDescendantBindingError::InvalidDescendantContext,
                "source proof-local given-descendant binding descendant context is invalid"
                    .to_owned(),
            ),
            (
                SourceProofLocalGivenDescendantBindingError::InvalidBindingEnvironment,
                "source proof-local given-descendant binding binding environment is invalid"
                    .to_owned(),
            ),
            (
                SourceProofLocalGivenDescendantBindingError::InvalidInstallation,
                "source proof-local given-descendant binding installation is invalid".to_owned(),
            ),
        ] {
            assert_eq!(error.to_string(), expected);
            let _: &dyn std::error::Error = &error;
        }
    }

    #[test]
    fn source_proof_local_given_descendant_binding_inherits_shadows_restores_and_excludes() {
        let fixture = task269sdc_fixture();
        let handoff =
            SourceProofLocalGivenDescendantBindingProducer::build(fixture.input, &fixture.base)
                .expect("Task269SDC checker handoff");
        let mut bindings = handoff.binding_env().bindings().clone();
        let shadow = bindings.insert(BindingDraft {
            spelling: "y".to_owned(),
            kind: BindingKind::GivenWitness,
            identity: BinderIdentity::ResolverLocal {
                scope: LocalTermScope::new(vec![0, 1]),
                ordinal: 2,
                declaration_range: range(fixture.source, 106, 107),
            },
            owner_context: BindingContextId::new(3),
            declaration_range: range(fixture.source, 106, 107),
            visible_after_ordinal: 2,
            type_site: BindingTypeSite::Missing,
            status: BindingStatus::Active,
            captured: CapturedFreeVariables::default(),
            diagnostics: Vec::new(),
            recovery: BindingRecoveryState::Normal,
        });
        assert_eq!(shadow, BindingId::new(2));
        let mut contexts = handoff.binding_env().contexts().clone();
        assert_eq!(
            contexts.insert(BindingContextDraft {
                owner: BindingContextOwner::Generated("task269sdc-shadow-child".to_owned()),
                parent: Some(BindingContextId::new(1)),
                layer: BindingContextLayer::Block,
                lexical_scope: Some(LocalTermScope::new(vec![0, 1])),
                bindings: vec![shadow],
                visible_bindings: vec![BindingId::new(0), BindingId::new(1), shadow],
                recovery: BindingContextRecovery::Normal,
            }),
            BindingContextId::new(3)
        );
        assert_eq!(
            contexts.insert(BindingContextDraft {
                owner: BindingContextOwner::Generated("task269sdc-sibling-child".to_owned()),
                parent: Some(BindingContextId::new(1)),
                layer: BindingContextLayer::Block,
                lexical_scope: Some(LocalTermScope::new(vec![0, 2])),
                bindings: Vec::new(),
                visible_bindings: vec![BindingId::new(0), BindingId::new(1)],
                recovery: BindingContextRecovery::Normal,
            }),
            BindingContextId::new(4)
        );
        assert_eq!(
            contexts.insert(BindingContextDraft {
                owner: BindingContextOwner::Generated("task269sdc-proof-sibling".to_owned()),
                parent: Some(BindingContextId::new(0)),
                layer: BindingContextLayer::Proof,
                lexical_scope: Some(LocalTermScope::new(vec![1])),
                bindings: Vec::new(),
                visible_bindings: vec![BindingId::new(0)],
                recovery: BindingContextRecovery::Normal,
            }),
            BindingContextId::new(5)
        );
        let matrix = BindingEnv::try_new(BindingEnvParts {
            source_id: fixture.source,
            module_id: fixture.module,
            contexts,
            bindings,
            diagnostics: handoff.binding_env().diagnostics().clone(),
        })
        .expect("Task269SDC synthetic scope matrix");
        let lookup = |context, scope: Vec<u32>, ordinal| {
            matrix.lookup(&BindingLookupSite::new(
                "y",
                BindingContextId::new(context),
                Some(LocalTermScope::new(scope)),
                ordinal,
            ))
        };
        assert!(matches!(
            lookup(1, vec![0], 1),
            Ok(BindingLookupResult::ForwardReference { candidates, .. })
                if candidates == [BindingId::new(1)]
        ));
        assert_eq!(
            lookup(1, vec![0], 2),
            Ok(BindingLookupResult::Local(BindingId::new(1)))
        );
        assert_eq!(
            lookup(2, vec![0, 0], 2),
            Ok(BindingLookupResult::Local(BindingId::new(1)))
        );
        assert_eq!(
            lookup(3, vec![0, 1], 3),
            Ok(BindingLookupResult::Local(BindingId::new(2)))
        );
        assert_eq!(
            lookup(1, vec![0], 3),
            Ok(BindingLookupResult::Local(BindingId::new(1)))
        );
        assert_eq!(
            lookup(4, vec![0, 2], 2),
            Ok(BindingLookupResult::Local(BindingId::new(1)))
        );
        assert_eq!(
            lookup(0, Vec::new(), 2),
            Ok(BindingLookupResult::Unresolved)
        );
        assert_eq!(lookup(5, vec![1], 2), Ok(BindingLookupResult::Unresolved));
    }

    #[test]
    fn source_proof_local_given_descendant_binding_has_no_occurrence_capture_or_semantic_owner() {
        let fixture = task269sdc_fixture();
        let handoff =
            SourceProofLocalGivenDescendantBindingProducer::build(fixture.input, &fixture.base)
                .expect("Task269SDC checker handoff");
        let binding = handoff
            .binding_env()
            .bindings()
            .get(BindingId::new(1))
            .expect("Task269SDC Given binding");
        assert_eq!(binding.type_site, BindingTypeSite::Missing);
        assert_eq!(binding.status, BindingStatus::Active);
        assert!(binding.captured.identities().is_empty());
        assert!(binding.diagnostics.is_empty());
        for excluded in [
            "term-reference",
            "occurrence",
            "local-abbreviation",
            "captured-variable",
            "condition-fact",
            "initial-obligation",
            "terminal-goal",
            "accepted",
            "discharged",
            "verification-condition",
        ] {
            assert!(
                !handoff.debug_text().contains(excluded),
                "excluded {excluded}"
            );
        }

        let empty = empty_typed(fixture.source, fixture.module.clone());
        let generic = SourcePrimaryTermProducer::build(
            SourcePrimaryTermHandoffInput {
                source_id: fixture.source,
                module_id: fixture.module.clone(),
                terms: Vec::new(),
                references: Vec::new(),
                numeric_type_requests: Vec::new(),
            },
            &fixture.base,
            empty.nodes(),
        )
        .expect("Task269SDC generic source-term sentinel");
        let generic_first = empty
            .clone()
            .with_source_term(generic.clone())
            .expect("Task269SDC generic first");
        let generic_before = generic_first.debug_text();
        assert_eq!(
            generic_first
                .clone()
                .with_source_proof_local_given_descendant_binding(handoff.clone()),
            Err(TypedAstError::InvalidSourceProofLocalGivenDescendantBinding)
        );
        assert_eq!(generic_first.debug_text(), generic_before);
        let mut generic_collision = generic_first.clone();
        generic_collision
            .inject_source_proof_local_given_descendant_binding_for_test(handoff.clone());
        assert_eq!(
            assemble_empty(&generic_collision),
            Err(ResolvedTypedAstError::InvalidSourceProofLocalGivenDescendantBinding)
        );
        assert_eq!(generic_first.debug_text(), generic_before);
        let sdc_first = empty
            .with_source_proof_local_given_descendant_binding(handoff.clone())
            .expect("Task269SDC first owner");
        let sdc_before = sdc_first.debug_text();
        assert_eq!(
            sdc_first.clone().with_source_term(generic),
            Err(TypedAstError::InvalidSourceTerm)
        );
        assert_eq!(sdc_first.debug_text(), sdc_before);
        assert!(sdc_first.nodes().is_empty());
        assert!(sdc_first.contexts().is_empty());
        assert!(sdc_first.types().is_empty());
        assert!(sdc_first.facts().is_empty());
        assert!(sdc_first.coercions().is_empty());
        assert!(sdc_first.initial_obligations().is_empty());
        assert!(sdc_first.diagnostics().is_empty());
        let resolved = assemble_empty(&sdc_first).expect("Task269SDC final replay");
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
        assert_eq!(
            TypedAstError::InvalidSourceProofLocalGivenDescendantBinding.to_string(),
            "typed AST source proof-local given-descendant binding handoff is inconsistent"
        );
        assert_eq!(
            ResolvedTypedAstError::InvalidSourceProofLocalGivenDescendantBinding.to_string(),
            "resolved typed AST source proof-local given-descendant binding handoff is inconsistent"
        );

        let mut corrupted = handoff;
        corrupted.final_binding_fingerprint = "corrupt".to_owned();
        let mut injected = empty_typed(fixture.source, fixture.module);
        injected.inject_source_proof_local_given_descendant_binding_for_test(corrupted);
        assert_eq!(
            assemble_empty(&injected),
            Err(ResolvedTypedAstError::InvalidSourceProofLocalGivenDescendantBinding)
        );
    }

    struct Task269sdcFixture {
        source: SourceId,
        module: ModuleId,
        input: SourceProofLocalGivenDescendantBindingHandoffInput,
        base: BindingEnv,
        other_definition: DefinitionId,
        other_contribution: SourceContributionId,
    }

    fn task269sdc_fixture() -> Task269sdcFixture {
        let source = source_id();
        let module = ModuleId::new(PackageId::new("pkg"), ModulePath::new("task269sdc"));
        let local = concat!(
            "contribution=0:namespace=task269sdc:owner=theorem#1:shell=theorem:",
            "kind=theorem:name=ProofLocalGivenDescendantCaptureSmoke:notation=_:arity=_:",
            "definition=theorem:registration=_:policy=non-overloadable:",
            "slot=non-overloadable:_:theorem:_"
        );
        let theorem_symbol = SymbolId::new(
            module.clone(),
            LocalSymbolId::new(local),
            FullyQualifiedName::new(format!("pkg::task269sdc::{local}")),
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
                SourceAnchor::Range(range(source, 19, 179)),
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
                SourceAnchor::Range(range(source, 19, 179)),
                vec![2, 1],
            ),
            contribution,
        ));
        let input = SourceProofLocalGivenDescendantBindingHandoffInput {
            source_id: source,
            module_id: module.clone(),
            lower_fingerprint: task286_receipt(source, module.clone(), 4),
            theorem_symbol,
            theorem_definition,
            contribution,
            theorem_range: range(source, 19, 179),
            proof_range: range(source, 73, 178),
            given_range: range(source, 81, 99),
            segment_range: range(source, 87, 98),
            name_range: range(source, 87, 88),
            descendant_range: range(source, 102, 159),
            source_ordinal: 1,
            local: LocalTermBinding::new(
                "y",
                LocalTermScope::new(vec![0]),
                range(source, 87, 88),
                1,
            ),
            descendant_scope: LocalTermScope::new(vec![0, 0]),
            recovery: SourceProofLocalGivenBindingRecovery::Normal,
        };
        Task269sdcFixture {
            source,
            module: module.clone(),
            input,
            base: exact_base(source, module),
            other_definition,
            other_contribution,
        }
    }

    fn native_binding_inputs(
        template: &TypedAst,
        rename: bool,
    ) -> (
        TypedAst,
        SurfaceResolvedArena,
        SymbolEnv,
        ResolvedVariableScope,
    ) {
        use mizar_resolve::{
            declarations::DeclarationShellCollector,
            env::NamespacePath,
            names::{SourceVariableScopeInput, SourceVariableScopeResolver},
            symbols::SignatureProjectionExtractor,
        };
        let mut builder = SurfaceAstBuilder::new(template.source_id());
        let mut ids = Vec::new();
        for (_, node) in template.nodes().iter() {
            let row = template
                .source_node_entries()
                .unwrap()
                .find(|row| Some(row.site()) == node.resolved_node)
                .unwrap();
            let id = if let SurfaceNodeKind::Token(token) = row.kind() {
                builder.add_token(
                    token.kind,
                    if rename && token.text.as_ref() == "y" {
                        "w"
                    } else {
                        token.text.as_ref()
                    },
                    row.range(),
                )
            } else {
                builder.add_node(
                    row.kind().clone(),
                    row.range(),
                    node.children
                        .iter()
                        .map(|child| ids[child.index()])
                        .collect(),
                )
            };
            ids.push(id);
        }
        let ast = builder.finish(Some(ids[template.nodes().root().unwrap().index()]), None);
        let native = SurfaceResolvedArena::lower(&ast, template.module_id()).unwrap();
        let shells = DeclarationShellCollector::new(&ast, template.module_id()).collect();
        let collected = SignatureProjectionExtractor::new(
            &ast,
            &shells,
            NamespacePath::new(template.module_id().path().as_str()),
        )
        .collect(template.module_id());
        assert!(collected.diagnostics().is_empty());
        let symbols = collected.env().clone();
        let scope = SourceVariableScopeResolver::resolve_proof_bindings(
            SourceVariableScopeInput::new(&ast, template.module_id(), &symbols),
        )
        .unwrap();
        let mut original = TypedAst::try_new(TypedAstParts {
            source_id: template.source_id(),
            module_id: template.module_id().clone(),
            resolved_root: template.resolved_root(),
            source_context: None,
            source_type: None,
            source_attribute: None,
            nodes: template.nodes().clone(),
            contexts: LocalTypeContextTable::new(),
            types: TypeTable::new(),
            facts: TypeFactTable::new(),
            coercions: CoercionTable::new(),
            initial_obligations: InitialObligationTable::new(),
            diagnostics: TypeDiagnosticTable::new(),
        })
        .unwrap();
        original.install_source_nodes(&native).unwrap();
        (original, native, symbols, scope)
    }

    #[test]
    fn native_conversion_authority_and_coherent_environment_are_replayed() {
        use mizar_resolve::{
            declarations::DeclarationShellCollector,
            env::NamespacePath,
            names::{SourceVariableScopeInput, SourceVariableScopeResolver},
            symbols::SignatureProjectionExtractor,
        };
        let inputs = |from: &str, to: &str| {
            let source = source_id();
            let module = ModuleId::new(PackageId::new("pkg"), ModulePath::new("conversion_trust"));
            let words = [
                "theorem",
                "T",
                ":",
                "thesis",
                "proof",
                "let",
                "s",
                "be",
                from,
                ";",
                "reconsider",
                "a",
                "=",
                "s",
                "as",
                to,
                ";",
                "end",
                ";",
            ];
            let mut builder = SurfaceAstBuilder::new(source);
            let mut tokens = Vec::new();
            let mut spans = Vec::new();
            let mut cursor = 0;
            for word in words {
                let span = range(source, cursor, cursor + word.len());
                cursor += word.len() + 1;
                let kind = if matches!(word, "T" | "s" | "a") {
                    SurfaceTokenKind::Identifier
                } else if matches!(word, ":" | ";" | "=") {
                    SurfaceTokenKind::ReservedSymbol
                } else {
                    SurfaceTokenKind::ReservedWord
                };
                tokens.push(builder.add_token(kind, word, span));
                spans.push(span);
            }
            let node =
                |builder: &mut SurfaceAstBuilder, kind, start: usize, end: usize, children| {
                    builder.add_node(
                        kind,
                        range(source, spans[start].start, spans[end].end),
                        children,
                    )
                };
            let from_head = node(
                &mut builder,
                SurfaceNodeKind::TypeHead,
                8,
                8,
                vec![tokens[8]],
            );
            let from_type = node(
                &mut builder,
                SurfaceNodeKind::TypeExpression,
                8,
                8,
                vec![from_head],
            );
            let segment = node(
                &mut builder,
                SurfaceNodeKind::QualifiedVariableSegment,
                6,
                8,
                vec![tokens[6], tokens[7], from_type],
            );
            let declaration = node(
                &mut builder,
                SurfaceNodeKind::LetStatement,
                5,
                9,
                vec![tokens[5], segment, tokens[9]],
            );
            let reference = node(
                &mut builder,
                SurfaceNodeKind::TermReference,
                13,
                13,
                vec![tokens[13]],
            );
            let value = node(
                &mut builder,
                SurfaceNodeKind::TermExpression,
                13,
                13,
                vec![reference],
            );
            let item = node(
                &mut builder,
                SurfaceNodeKind::ReconsiderItem,
                11,
                13,
                vec![tokens[11], tokens[12], value],
            );
            let to_head = node(
                &mut builder,
                SurfaceNodeKind::TypeHead,
                15,
                15,
                vec![tokens[15]],
            );
            let target = node(
                &mut builder,
                SurfaceNodeKind::TypeExpression,
                15,
                15,
                vec![to_head],
            );
            let conversion = node(
                &mut builder,
                SurfaceNodeKind::ReconsiderStatement,
                10,
                16,
                vec![tokens[10], item, tokens[14], target, tokens[16]],
            );
            let thesis = node(
                &mut builder,
                SurfaceNodeKind::FormulaConstant(SurfaceFormulaConstant::Thesis),
                3,
                3,
                vec![tokens[3]],
            );
            let formula = node(
                &mut builder,
                SurfaceNodeKind::FormulaExpression,
                3,
                3,
                vec![thesis],
            );
            let proof = node(
                &mut builder,
                SurfaceNodeKind::ProofBlock,
                4,
                17,
                vec![tokens[4], declaration, conversion, tokens[17]],
            );
            let theorem = node(
                &mut builder,
                SurfaceNodeKind::TheoremItem,
                0,
                18,
                vec![tokens[0], tokens[1], tokens[2], formula, proof, tokens[18]],
            );
            let items = node(
                &mut builder,
                SurfaceNodeKind::ItemList,
                0,
                18,
                vec![theorem],
            );
            let compilation = node(
                &mut builder,
                SurfaceNodeKind::CompilationUnit,
                0,
                18,
                vec![items],
            );
            tokens.push(compilation);
            let root = node(&mut builder, SurfaceNodeKind::Root, 0, 18, tokens);
            let ast = builder.finish(Some(root), None);
            let native = SurfaceResolvedArena::lower(&ast, &module).unwrap();
            let shells = DeclarationShellCollector::new(&ast, &module).collect();
            let collection = SignatureProjectionExtractor::new(
                &ast,
                &shells,
                NamespacePath::new(module.path().as_str()),
            )
            .collect(&module);
            assert!(collection.diagnostics().is_empty());
            let symbols = collection.env().clone();
            let scope = SourceVariableScopeResolver::resolve_proof_bindings(
                SourceVariableScopeInput::new(&ast, &module, &symbols),
            )
            .unwrap();
            let mut original = TypedAst::try_new(TypedAstParts {
                source_id: source,
                module_id: module,
                resolved_root: Some(native.arena().root()),
                source_context: None,
                source_type: None,
                source_attribute: None,
                nodes: TypedArena::try_new(
                    Some(TypedNodeId::new(native.arena().root().index())),
                    native
                        .arena()
                        .iter()
                        .map(|(id, node)| {
                            TypedNode::new("syntax", node.origin().anchor().clone())
                                .with_resolved_node(id)
                                .with_children(
                                    node.children()
                                        .iter()
                                        .map(|id| TypedNodeId::new(id.index()))
                                        .collect(),
                                )
                        })
                        .collect(),
                )
                .unwrap(),
                contexts: Default::default(),
                types: Default::default(),
                facts: Default::default(),
                coercions: Default::default(),
                initial_obligations: Default::default(),
                diagnostics: Default::default(),
            })
            .unwrap();
            original.install_source_nodes(&native).unwrap();
            (original, native, symbols, scope)
        };
        let (original, native, symbols, scope) = inputs("set", "object");
        let handoff = SourceProofLocalDeclarationProducer::build_bindings(
            &original, &native, &symbols, &scope,
        )
        .unwrap();
        let (_, checked, queries) = handoff.check_conversions().unwrap();
        assert_eq!(checked.coercions().len(), 1);
        assert_eq!(
            queries[0].status(),
            crate::type_checker::TypeFactQueryStatus::Satisfied
        );
        let (other, other_native, other_symbols, other_scope) = inputs("object", "object");
        assert_eq!(original.source_id(), other.source_id());
        assert!(
            SourceProofLocalDeclarationProducer::build_bindings(
                &other,
                &other_native,
                &other_symbols,
                &other_scope
            )
            .is_ok()
        );
        for forged in [
            SourceProofLocalDeclarationProducer::build_bindings(&other, &native, &symbols, &scope),
            SourceProofLocalDeclarationProducer::build_bindings(
                &original,
                &other_native,
                &other_symbols,
                &other_scope,
            ),
            SourceProofLocalDeclarationProducer::build_bindings(
                &original,
                &native,
                &other_symbols,
                &scope,
            ),
            SourceProofLocalDeclarationProducer::build_bindings(
                &original,
                &native,
                &symbols,
                &other_scope,
            ),
        ] {
            assert_eq!(
                forged,
                Err(SourceProofLocalDeclarationError::DependencyMismatch)
            );
        }
        let (narrow, narrow_native, narrow_symbols, narrow_scope) = inputs("object", "set");
        let negative = SourceProofLocalDeclarationProducer::build_bindings(
            &narrow,
            &narrow_native,
            &narrow_symbols,
            &narrow_scope,
        )
        .unwrap()
        .check_conversions()
        .unwrap();
        assert_eq!(
            negative.1.coercions().iter().next().unwrap().1.status,
            crate::typed_ast::CoercionStatus::Rejected
        );
        assert!(negative.1.initial_obligations().is_empty());
        let empty = empty_typed(original.source_id(), original.module_id().clone());
        let installed = empty
            .clone()
            .with_source_proof_local_binding(handoff.clone())
            .unwrap();
        assert!(
            installed.types().is_empty()
                && installed.facts().is_empty()
                && installed.coercions().is_empty()
        );
        assert_eq!(
            assemble_empty(&installed)
                .unwrap()
                .source_proof_local_binding(),
            Some(&handoff)
        );
        assert!(
            installed
                .with_source_proof_local_binding(handoff.clone())
                .is_err()
        );
        let mut bad_site = handoff.clone();
        let source_guard = bad_site
            .original
            .source_node_entries()
            .unwrap()
            .find(|row| row.kind() == &SurfaceNodeKind::TypeExpression)
            .unwrap()
            .range();
        bad_site
            .binding_env
            .binding_mut_for_test(BindingId::new(1))
            .unwrap()
            .type_site = BindingTypeSite::Source(source_guard);
        let mut bad_capture = handoff.clone();
        let identity = bad_capture
            .binding_env
            .bindings()
            .get(BindingId::new(0))
            .unwrap()
            .identity
            .clone();
        bad_capture
            .binding_env
            .binding_mut_for_test(BindingId::new(1))
            .unwrap()
            .captured = CapturedFreeVariables::new(vec![identity]);
        let mut bad_original = handoff;
        bad_original.original = Arc::new(other);
        for forged in [bad_site, bad_capture, bad_original] {
            assert!(forged.check_conversions().is_err());
            assert!(
                empty
                    .clone()
                    .with_source_proof_local_binding(forged.clone())
                    .is_err()
            );
            let mut injected = empty.clone();
            injected.inject_source_proof_local_binding_for_test(forged);
            assert_eq!(
                assemble_empty(&injected),
                Err(ResolvedTypedAstError::InvalidSourceProofLocalDeclaration)
            );
        }
    }

    #[test]
    fn conversion_element_rules_reject_isolated_role_corruption() {
        use SurfaceNodeKind as K;
        use SurfaceTokenKind as T;
        let source = source_id();
        let receipt = task286_receipt(
            source,
            ModuleId::new(PackageId::new("pkg"), ModulePath::new("conversion_roles")),
            4,
        );
        let prototype = receipt
            .original
            .iter()
            .find(|row| matches!(row.kind(), K::Token(_)))
            .unwrap();
        let sites = receipt
            .original
            .iter()
            .map(SourceNode::site)
            .collect::<Vec<_>>();
        let token = |kind, text: &str| {
            let K::Token(mut token) = prototype.kind.clone() else {
                unreachable!()
            };
            token.kind = kind;
            token.text = text.into();
            K::Token(token)
        };
        let rows = |descriptors: Vec<(K, Option<usize>, usize)>| {
            descriptors
                .into_iter()
                .enumerate()
                .map(|(index, (kind, parent, ordinal))| {
                    let mut row = prototype.clone();
                    row.spelling = match &kind {
                        K::Token(t) => t.text.to_string(),
                        _ => String::new(),
                    };
                    row.kind = kind;
                    row.site = sites[index];
                    row.parent = parent.map(|index| sites[index]);
                    row.child_ordinal = ordinal;
                    row.preorder_ordinal = index;
                    row.range = range(source, 0, 100);
                    row
                })
                .collect::<Vec<_>>()
        };
        let statement = rows(vec![
            (K::ReconsiderStatement, None, 0),
            (token(T::ReservedWord, "reconsider"), Some(0), 0),
            (K::ReconsiderItem, Some(0), 1),
            (token(T::ReservedSymbol, ","), Some(0), 2),
            (K::ReconsiderItem, Some(0), 3),
            (token(T::ReservedWord, "as"), Some(0), 4),
            (K::TypeExpression, Some(0), 5),
            (K::JustificationClause, Some(0), 6),
            (token(T::ReservedSymbol, ";"), Some(0), 7),
            (token(T::Identifier, "x"), Some(2), 0),
            (token(T::ReservedSymbol, "="), Some(2), 1),
            (K::TermExpression, Some(2), 2),
            (token(T::Identifier, "y"), Some(4), 0),
            (token(T::ReservedWord, "by"), Some(7), 0),
        ]);
        let (items, target, tail) =
            source_reconsider_statement_roles(&statement, &statement[0]).unwrap();
        assert_eq!(
            items.iter().map(|row| row.site()).collect::<Vec<_>>(),
            [sites[2], sites[4]]
        );
        assert_eq!(target.site(), sites[6]);
        assert_eq!(tail.unwrap().site(), sites[7]);
        assert_eq!(
            source_reconsider_item_roles(&statement, items[0])
                .unwrap()
                .1
                .unwrap()
                .site(),
            sites[11]
        );
        assert!(
            source_reconsider_item_roles(&statement, items[1])
                .unwrap()
                .1
                .is_none()
        );
        for (index, kind) in [
            (1, token(T::Identifier, "reconsider")),
            (3, token(T::Identifier, ",")),
            (5, token(T::ReservedSymbol, "as")),
            (6, K::TermExpression),
            (7, K::ProofBlock),
            (8, token(T::Identifier, ";")),
        ] {
            let mut bad = statement.clone();
            bad[index].kind = kind;
            assert!(source_reconsider_statement_roles(&bad, &bad[0]).is_none());
        }
        for (index, kind) in [
            (9, token(T::ReservedWord, "x")),
            (10, token(T::ReservedWord, "=")),
            (11, K::TypeExpression),
        ] {
            let mut bad = statement.clone();
            bad[index].kind = kind;
            assert!(source_reconsider_item_roles(&bad, &bad[2]).is_none());
        }
        let mut orphan = statement.clone();
        orphan[2].parent = None;
        assert!(source_reconsider_item_roles(&orphan, &orphan[2]).is_none());
        let qua = rows(vec![
            (K::QuaExpression, None, 0),
            (K::TermReference, Some(0), 0),
            (token(T::ReservedWord, "qua"), Some(0), 1),
            (K::TypeExpression, Some(0), 2),
        ]);
        assert_eq!(
            source_qua_roles(&qua, &qua[0])
                .unwrap()
                .map(SourceNode::site),
            [sites[1], sites[3]]
        );
        for (index, kind) in [
            (1, K::FormulaExpression),
            (2, token(T::Identifier, "qua")),
            (3, K::TermExpression),
        ] {
            let mut bad = qua.clone();
            bad[index].kind = kind;
            assert!(source_qua_roles(&bad, &bad[0]).is_none());
        }
        let mut reordered = qua;
        reordered[3].child_ordinal = 1;
        assert!(source_qua_roles(&reordered, &reordered[0]).is_none());
    }

    #[test]
    fn inline_element_rules_reject_isolated_role_corruption() {
        use SurfaceNodeKind as K;
        use SurfaceTokenKind as T;
        let source = source_id();
        let receipt = task286_receipt(
            source,
            ModuleId::new(PackageId::new("pkg"), ModulePath::new("inline_roles")),
            4,
        );
        let prototype = receipt
            .original
            .iter()
            .find(|row| matches!(row.kind(), K::Token(_)))
            .unwrap()
            .clone();
        let sites = receipt
            .original
            .iter()
            .map(SourceNode::site)
            .collect::<Vec<_>>();
        let token = |kind, text: &str| {
            let K::Token(mut token) = prototype.kind.clone() else {
                unreachable!()
            };
            token.kind = kind;
            token.text = text.into();
            K::Token(token)
        };
        for functor in [true, false] {
            let mut frame = vec![
                token(T::ReservedWord, if functor { "deffunc" } else { "defpred" }),
                token(T::Identifier, "F"),
                token(T::ReservedSymbol, "("),
                K::TypedParameter,
                token(T::ReservedSymbol, ")"),
            ];
            if functor {
                frame.extend([token(T::ReservedSymbol, "->"), K::TypeExpression]);
            }
            frame.extend([
                token(T::ReservedWord, if functor { "equals" } else { "means" }),
                if functor {
                    K::TermExpression
                } else {
                    K::FormulaExpression
                },
                token(T::ReservedSymbol, ";"),
            ]);
            let mut descriptors = vec![(
                if functor {
                    K::InlineFunctorDefinition
                } else {
                    K::InlinePredicateDefinition
                },
                None,
                0,
            )];
            descriptors.extend(
                frame
                    .into_iter()
                    .enumerate()
                    .map(|(ordinal, kind)| (kind, Some(0), ordinal)),
            );
            let qualifier = descriptors.len() + 1;
            descriptors.extend([
                (token(T::Identifier, "n"), Some(4), 0),
                (token(T::ReservedWord, "be"), Some(4), 1),
                (K::TypeExpression, Some(4), 2),
            ]);
            let rows = descriptors
                .into_iter()
                .enumerate()
                .map(|(index, (kind, parent, ordinal))| {
                    let mut row = prototype.clone();
                    row.spelling = match &kind {
                        K::Token(token) => token.text.to_string(),
                        _ => String::new(),
                    };
                    row.kind = kind;
                    row.site = sites[index];
                    row.parent = parent.map(|index| sites[index]);
                    row.child_ordinal = ordinal;
                    row.preorder_ordinal = index;
                    row.range = range(source, 0, 100);
                    row
                })
                .collect::<Vec<_>>();
            let rule = if functor {
                source_inline_functor_roles
            } else {
                source_inline_predicate_roles
            };
            let (name, formals, body, result) = rule(&rows, &rows[0]).unwrap();
            assert_eq!(name.site(), rows[2].site());
            assert_eq!(formals.len(), 1);
            assert_eq!(result.is_some(), functor);
            let body = rows
                .iter()
                .position(|row| row.site() == body.site())
                .unwrap();
            let guard = rows
                .iter()
                .position(|row| row.site() == formals[0][1].site())
                .unwrap();
            for (index, kind) in [
                (
                    1,
                    token(T::Identifier, if functor { "deffunc" } else { "defpred" }),
                ),
                (2, token(T::ReservedWord, "F")),
                (qualifier, token(T::Identifier, "be")),
                (body, K::TermList),
                (guard, K::TermList),
            ] {
                let mut corrupted = rows.clone();
                corrupted[index].kind = kind;
                assert!(rule(&corrupted, &corrupted[0]).is_none());
            }
        }
    }

    #[test]
    fn native_bindings_authenticate_original_scope_and_replay_without_donor_privileges() {
        let f = task269sdc_fixture();
        let template = task286_source(f.source, f.module.clone(), 4, 0);
        let (original, native, symbols, scope) = native_binding_inputs(&template, false);
        let handoff = SourceProofLocalDeclarationProducer::build_bindings(
            &original, &native, &symbols, &scope,
        )
        .unwrap();
        assert_eq!(
            (
                handoff.binding_env().bindings().len(),
                handoff.binding_env().contexts().len()
            ),
            (4, 3)
        );
        let empty = empty_typed(f.source, f.module.clone());
        let typed = empty
            .clone()
            .with_source_proof_local_binding(handoff.clone())
            .unwrap();
        let resolved = assemble_empty(&typed).unwrap();
        assert_eq!(resolved.source_proof_local_binding(), Some(&handoff));
        assert!(
            typed.facts().is_empty()
                && typed.types().is_empty()
                && typed.initial_obligations().is_empty()
        );
        assert!(resolved.checked_proofs().is_empty() && resolved.checked_formulas().is_empty());
        assert_eq!(
            typed
                .clone()
                .with_source_proof_local_binding(handoff.clone()),
            Err(TypedAstError::InvalidSourceProofLocalDeclaration)
        );
        let old = SourceProofLocalGivenDescendantBindingProducer::build(f.input, &f.base).unwrap();
        assert_eq!(
            typed.with_source_proof_local_given_descendant_binding(old.clone()),
            Err(TypedAstError::InvalidSourceProofLocalGivenDescendantBinding)
        );
        assert_eq!(
            empty
                .clone()
                .with_source_proof_local_given_descendant_binding(old.clone())
                .unwrap()
                .with_source_proof_local_binding(handoff.clone()),
            Err(TypedAstError::InvalidSourceProofLocalDeclaration)
        );

        let (other, other_native, other_symbols, other_scope) =
            native_binding_inputs(&template, true);
        for result in [
            SourceProofLocalDeclarationProducer::build_bindings(&other, &native, &symbols, &scope),
            SourceProofLocalDeclarationProducer::build_bindings(
                &original,
                &other_native,
                &other_symbols,
                &other_scope,
            ),
            SourceProofLocalDeclarationProducer::build_bindings(
                &original,
                &native,
                &symbols,
                &other_scope,
            ),
        ] {
            assert_eq!(
                result,
                Err(SourceProofLocalDeclarationError::DependencyMismatch)
            );
        }
        // An original graph with a coherent upper transaction is not raw source intake.
        let mut hybrid = original.clone();
        hybrid.inject_source_proof_local_given_descendant_binding_for_test(old);
        assert_eq!(
            SourceProofLocalDeclarationProducer::build_bindings(&hybrid, &native, &symbols, &scope),
            Err(SourceProofLocalDeclarationError::DependencyMismatch)
        );
        let mut bad_capture = handoff.clone();
        let wrong = bad_capture
            .binding_env
            .bindings()
            .get(BindingId::new(1))
            .unwrap()
            .identity
            .clone();
        bad_capture
            .binding_env
            .binding_mut_for_test(BindingId::new(3))
            .unwrap()
            .captured = CapturedFreeVariables::new(vec![wrong]);
        let mut bad_clock = handoff.clone();
        let row = bad_clock
            .binding_env
            .binding_mut_for_test(BindingId::new(3))
            .unwrap();
        row.visible_after_ordinal += 1;
        if let BinderIdentity::ResolverLocal { ordinal, .. } = &mut row.identity {
            *ordinal += 1;
        }
        let mut bad_origin = handoff.clone();
        bad_origin.original = Arc::new(other);
        let mut bad_upper = handoff;
        bad_upper.original = Arc::new(hybrid);
        for forged in [bad_capture, bad_clock, bad_origin, bad_upper] {
            assert!(
                empty
                    .clone()
                    .with_source_proof_local_binding(forged.clone())
                    .is_err()
            );
            let mut injected = empty.clone();
            injected.inject_source_proof_local_binding_for_test(forged);
            assert_eq!(
                assemble_empty(&injected),
                Err(ResolvedTypedAstError::InvalidSourceProofLocalDeclaration)
            );
        }
        for profile in [0, 1] {
            let template = task286_source(f.source, f.module.clone(), profile, 0);
            let (original, native, symbols, scope) = native_binding_inputs(&template, false);
            let other = SourceProofLocalDeclarationProducer::build_bindings(
                &original, &native, &symbols, &scope,
            )
            .unwrap();
            assert_eq!(other.binding_env().bindings().len(), 2);
            assert!(
                assemble_empty(
                    &empty
                        .clone()
                        .with_source_proof_local_binding(other)
                        .unwrap()
                )
                .is_ok()
            );
        }
    }

    fn task269_set_scope(receipt: &SourceProofLocalIntakeReceipt) -> ResolvedVariableScope {
        use mizar_resolve::{
            env::{SymbolEnv, SymbolEnvIndexes},
            names::{SourceVariableScopeInput, SourceVariableScopeResolver},
        };
        let mut builder = SurfaceAstBuilder::new(receipt.source_id);
        let mut nodes = std::collections::BTreeMap::new();
        let mut rows = receipt.original.iter().collect::<Vec<_>>();
        rows.sort_by_key(|row| std::cmp::Reverse(row.preorder_ordinal()));
        for row in rows {
            let node = if let SurfaceNodeKind::Token(token) = row.kind() {
                builder.add_token(token.kind, token.text.clone(), row.range())
            } else {
                builder.add_node(
                    row.kind().clone(),
                    row.range(),
                    source_children(&receipt.original, row.site())
                        .map(|child| nodes[&child.site()])
                        .collect(),
                )
            };
            nodes.insert(row.site(), node);
        }
        let ast = builder.finish(Some(nodes[&receipt.root_row().site()]), None);
        let symbols = SymbolEnv::new(receipt.module_id.clone(), SymbolEnvIndexes::default());
        SourceVariableScopeResolver::resolve_proof_occurrences(SourceVariableScopeInput::new(
            &ast,
            &receipt.module_id,
            &symbols,
        ))
        .unwrap()
    }

    #[test]
    fn source_proof_local_set_bindings_preserve_resolver_identity_and_replay() {
        let fixture = task269sdc_fixture();
        let scope = task269_set_scope(&fixture.input.lower_fingerprint);
        let handoff = SourceProofLocalGivenDescendantBindingProducer::build_set_bindings(
            fixture.input.clone(),
            &fixture.base,
            &scope,
        )
        .unwrap();
        assert_eq!(handoff.bindings().len(), 1);
        assert_eq!(handoff.binding_env().bindings().len(), 4);
        for (index, actual) in handoff.binding_env().bindings().iter().skip(1) {
            let original = &scope.bindings()[index.index()];
            assert_eq!(
                actual.identity,
                BinderIdentity::ResolverLocal {
                    scope: original.scope().clone(),
                    ordinal: original.ordinal(),
                    declaration_range: original.range()
                }
            );
            assert_eq!(actual.type_site, BindingTypeSite::Missing);
            if index.index() == 1 {
                assert!(actual.captured.identities().is_empty());
            } else {
                let target = &scope.bindings()[index.index() - 1];
                assert_eq!(
                    actual.captured.identities(),
                    &[BinderIdentity::ResolverLocal {
                        scope: target.scope().clone(),
                        ordinal: target.ordinal(),
                        declaration_range: target.range(),
                    }]
                );
            }
        }
        assert!(handoff.debug_text().contains("scope=[4,2]"));
        handoff
            .validate_installation(fixture.source, &fixture.module)
            .unwrap();
        let empty = empty_typed(fixture.source, fixture.module.clone());
        let typed = empty
            .clone()
            .with_source_proof_local_given_descendant_binding(handoff.clone())
            .unwrap();
        assert_eq!(
            assemble_empty(&typed)
                .unwrap()
                .source_proof_local_given_descendant_binding(),
            Some(&handoff)
        );
        for wrong_capture in [
            handoff
                .binding_env
                .bindings()
                .get(BindingId::new(1))
                .unwrap()
                .identity
                .clone(),
            BinderIdentity::ResolverLocal {
                scope: scope.bindings()[2].scope().clone(),
                ordinal: scope.bindings()[2].ordinal() + 1,
                declaration_range: scope.bindings()[2].range(),
            },
        ] {
            let mut forged = handoff.clone();
            forged
                .binding_env
                .binding_mut_for_test(BindingId::new(3))
                .unwrap()
                .captured = CapturedFreeVariables::new(vec![wrong_capture]);
            forged.final_binding_fingerprint = forged.binding_env.debug_text();
            assert_eq!(
                empty
                    .clone()
                    .with_source_proof_local_given_descendant_binding(forged.clone()),
                Err(TypedAstError::InvalidSourceProofLocalGivenDescendantBinding)
            );
            let mut injected = empty.clone();
            injected.inject_source_proof_local_given_descendant_binding_for_test(forged);
            assert_eq!(
                assemble_empty(&injected),
                Err(ResolvedTypedAstError::InvalidSourceProofLocalGivenDescendantBinding)
            );
        }
        let mut forged = handoff.clone();
        let mut parts = BindingEnvParts {
            source_id: fixture.source,
            module_id: fixture.module.clone(),
            contexts: forged.binding_env.contexts().clone(),
            bindings: fixture.base.bindings().clone(),
            diagnostics: forged.binding_env.diagnostics().clone(),
        };
        for binding in scope.bindings().iter().skip(1) {
            let local = LocalTermBinding::new(
                binding.spelling(),
                binding.scope().clone(),
                binding.range(),
                binding.ordinal() + 1,
            );
            let mut draft = BindingDraft::from_local_term(
                BindingContextId::new(if binding.ordinal() == 1 { 1 } else { 2 }),
                if binding.ordinal() == 1 {
                    BindingKind::GivenWitness
                } else {
                    BindingKind::LocalAbbreviation
                },
                &local,
            );
            draft.captured = handoff
                .binding_env
                .bindings()
                .get(BindingId::new(binding.ordinal()))
                .unwrap()
                .captured
                .clone();
            parts.bindings.insert(draft);
        }
        forged.binding_env = BindingEnv::try_new(parts).unwrap();
        forged.final_binding_fingerprint = forged.binding_env.debug_text();
        assert_eq!(
            forged.validate_installation(fixture.source, &fixture.module),
            Err(SourceProofLocalGivenDescendantBindingError::InvalidBindingEnvironment)
        );
        assert_eq!(
            empty
                .clone()
                .with_source_proof_local_given_descendant_binding(forged.clone()),
            Err(TypedAstError::InvalidSourceProofLocalGivenDescendantBinding)
        );
        let mut injected = empty;
        injected.inject_source_proof_local_given_descendant_binding_for_test(forged);
        assert_eq!(
            assemble_empty(&injected),
            Err(ResolvedTypedAstError::InvalidSourceProofLocalGivenDescendantBinding)
        );
        let mut shifted = fixture.input.lower_fingerprint.clone();
        for row in Arc::make_mut(&mut shifted.original) {
            row.range.start += 1;
            row.range.end += 1;
        }
        assert_eq!(
            SourceProofLocalGivenDescendantBindingProducer::build_set_bindings(
                fixture.input.clone(),
                &fixture.base,
                &task269_set_scope(&shifted)
            ),
            Err(SourceProofLocalGivenDescendantBindingError::DependencyMismatch)
        );
        let mut foreign = fixture.input.lower_fingerprint.clone();
        foreign.module_id = ModuleId::new(PackageId::new("pkg"), ModulePath::new("foreign"));
        assert_eq!(
            SourceProofLocalGivenDescendantBindingProducer::build_set_bindings(
                fixture.input.clone(),
                &fixture.base,
                &task269_set_scope(&foreign)
            ),
            Err(SourceProofLocalGivenDescendantBindingError::DependencyMismatch)
        );
        let mut malformed = fixture.input;
        let rows = Arc::make_mut(&mut malformed.lower_fingerprint.original);
        let equating = rows
            .iter_mut()
            .find(|row| row.kind() == &SurfaceNodeKind::Equating)
            .unwrap();
        equating.kind = SurfaceNodeKind::TermList;
        malformed.lower_fingerprint.fingerprint =
            malformed.lower_fingerprint.structural_fingerprint();
        assert_eq!(
            SourceProofLocalGivenDescendantBindingProducer::build_set_bindings(
                malformed,
                &fixture.base,
                &scope
            ),
            Err(SourceProofLocalGivenDescendantBindingError::DependencyMismatch)
        );
    }

    fn task269sdc_empty_base(fixture: &Task269sdcFixture) -> BindingEnv {
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
        .expect("Task269SDC valid but inexact empty base")
    }

    fn task269sdc_inexact_bases(fixture: &Task269sdcFixture) -> Vec<BindingEnv> {
        let mut bases = Vec::new();
        for mutate in 0..13 {
            let mut base = fixture.base.clone();
            let binding = base
                .binding_mut_for_test(BindingId::new(0))
                .expect("Task269SDC reserve binding");
            match mutate {
                0 => binding.id = BindingId::new(1),
                1 => binding.spelling = "wrong".to_owned(),
                2 => binding.kind = BindingKind::GivenWitness,
                3 => {
                    binding.identity = BinderIdentity::ReservedVariable {
                        spelling: "wrong".to_owned(),
                        declaration_range: range(fixture.source, 8, 9),
                    }
                }
                4 => {
                    binding.identity = BinderIdentity::ReservedVariable {
                        spelling: "x".to_owned(),
                        declaration_range: range(fixture.source, 7, 9),
                    }
                }
                5 => binding.owner_context = BindingContextId::new(1),
                6 => binding.declaration_range = range(fixture.source, 7, 9),
                7 => binding.visible_after_ordinal = 1,
                8 => binding.type_site = BindingTypeSite::Missing,
                9 => binding.status = BindingStatus::Active,
                10 => {
                    binding.captured =
                        CapturedFreeVariables::new(vec![BinderIdentity::ReservedVariable {
                            spelling: "x".to_owned(),
                            declaration_range: range(fixture.source, 8, 9),
                        }])
                }
                11 => binding.diagnostics = vec![BindingDiagnosticId::new(0)],
                12 => binding.recovery = BindingRecoveryState::Recovered,
                _ => unreachable!(),
            }
            bases.push(base);
        }
        for mutate in 0..6 {
            let mut contexts = BindingContextTable::new();
            let mut root = BindingContextDraft {
                owner: BindingContextOwner::Module,
                parent: None,
                layer: BindingContextLayer::Module,
                lexical_scope: None,
                bindings: vec![BindingId::new(0)],
                visible_bindings: vec![BindingId::new(0)],
                recovery: BindingContextRecovery::Normal,
            };
            match mutate {
                0 => root.owner = BindingContextOwner::Generated("task269sdc-base".to_owned()),
                1 => root.layer = BindingContextLayer::Expression,
                2 => root.lexical_scope = Some(LocalTermScope::new(vec![0])),
                3 => root.bindings.clear(),
                4 => root.visible_bindings.clear(),
                5 => root.recovery = BindingContextRecovery::Recovered,
                _ => unreachable!(),
            }
            contexts.insert(root);
            bases.push(
                BindingEnv::try_new(BindingEnvParts {
                    source_id: fixture.source,
                    module_id: fixture.module.clone(),
                    contexts,
                    bindings: fixture.base.bindings().clone(),
                    diagnostics: fixture.base.diagnostics().clone(),
                })
                .unwrap_or_else(|_| {
                    let mut base = fixture.base.clone();
                    base.binding_mut_for_test(BindingId::new(0))
                        .expect("Task269SDC fallback reserve binding")
                        .spelling = format!("wrong-context-{mutate}");
                    base
                }),
            );
        }
        let mut diagnostics = BindingDiagnosticTable::new();
        diagnostics.insert(crate::binding_env::BindingDiagnosticDraft {
            source_range: Some(range(fixture.source, 8, 9)),
            class: crate::binding_env::BindingDiagnosticClass::UnsupportedSourceShape,
            severity: crate::binding_env::BindingDiagnosticSeverity::Error,
            message_key: "task269sdc.base".to_owned(),
            recovery: crate::binding_env::BindingDiagnosticRecovery::Normal,
        });
        bases.push(
            BindingEnv::try_new(BindingEnvParts {
                source_id: fixture.source,
                module_id: fixture.module.clone(),
                contexts: fixture.base.contexts().clone(),
                bindings: fixture.base.bindings().clone(),
                diagnostics,
            })
            .expect("Task269SDC inexact base diagnostic table"),
        );
        bases
    }

    fn task269sdc_inexact_declaration_handoffs(
        handoff: &SourceProofLocalGivenDescendantBindingHandoff,
        source: SourceId,
    ) -> Vec<SourceProofLocalGivenDescendantBindingHandoff> {
        let mut handoffs = Vec::new();
        for mutate in 0..10 {
            let mut corrupted = handoff.clone();
            let binding = corrupted
                .binding_env
                .binding_mut_for_test(BindingId::new(1))
                .expect("Task269SDC final Given binding");
            match mutate {
                0 => binding.id = BindingId::new(2),
                1 => binding.spelling = "wrong".to_owned(),
                2 => {
                    binding.identity = BinderIdentity::ResolverLocal {
                        scope: LocalTermScope::new(vec![1]),
                        ordinal: 1,
                        declaration_range: range(source, 87, 88),
                    }
                }
                3 => {
                    binding.identity = BinderIdentity::ResolverLocal {
                        scope: LocalTermScope::new(vec![0]),
                        ordinal: 2,
                        declaration_range: range(source, 87, 88),
                    }
                }
                4 => {
                    binding.identity = BinderIdentity::ResolverLocal {
                        scope: LocalTermScope::new(vec![0]),
                        ordinal: 1,
                        declaration_range: range(source, 86, 88),
                    }
                }
                5 => binding.owner_context = BindingContextId::new(0),
                6 => binding.declaration_range = range(source, 86, 88),
                7 => binding.visible_after_ordinal = 2,
                8 => binding.recovery = BindingRecoveryState::Recovered,
                9 => binding.spelling.clear(),
                _ => unreachable!(),
            }
            corrupted.final_binding_fingerprint = corrupted.binding_env.debug_text();
            handoffs.push(corrupted);
        }
        handoffs
    }

    fn task269sdc_inexact_descendant_context_handoffs(
        handoff: &SourceProofLocalGivenDescendantBindingHandoff,
    ) -> Vec<SourceProofLocalGivenDescendantBindingHandoff> {
        let mut handoffs = Vec::new();
        for mutate in 0..8 {
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
                    source_range: range(handoff.source_id, 73, 178),
                },
                parent: Some(BindingContextId::new(0)),
                layer: BindingContextLayer::Proof,
                lexical_scope: Some(LocalTermScope::new(vec![0])),
                bindings: vec![BindingId::new(1)],
                visible_bindings: vec![BindingId::new(0), BindingId::new(1)],
                recovery: BindingContextRecovery::Normal,
            });
            let mut descendant = BindingContextDraft {
                owner: BindingContextOwner::SourceStatement {
                    source_range: range(handoff.source_id, 102, 159),
                },
                parent: Some(BindingContextId::new(1)),
                layer: BindingContextLayer::Block,
                lexical_scope: Some(LocalTermScope::new(vec![0, 0])),
                bindings: Vec::new(),
                visible_bindings: vec![BindingId::new(0), BindingId::new(1)],
                recovery: BindingContextRecovery::Normal,
            };
            let mut bindings = handoff.binding_env.bindings().clone();
            match mutate {
                0 => {
                    descendant.owner =
                        BindingContextOwner::Generated("task269sdc-descendant".to_owned())
                }
                1 => {
                    descendant.parent = Some(BindingContextId::new(0));
                    descendant.visible_bindings = vec![BindingId::new(0)];
                }
                2 => descendant.layer = BindingContextLayer::Expression,
                3 => descendant.lexical_scope = Some(LocalTermScope::new(vec![0, 1])),
                4 => {
                    let extra = bindings.insert(BindingDraft {
                        spelling: "extra".to_owned(),
                        kind: BindingKind::Generated,
                        identity: BinderIdentity::Generated {
                            context: BindingContextId::new(2),
                            counter: 269,
                        },
                        owner_context: BindingContextId::new(2),
                        declaration_range: range(handoff.source_id, 103, 104),
                        visible_after_ordinal: 2,
                        type_site: BindingTypeSite::Missing,
                        status: BindingStatus::Active,
                        captured: CapturedFreeVariables::default(),
                        diagnostics: Vec::new(),
                        recovery: BindingRecoveryState::Normal,
                    });
                    descendant.bindings = vec![extra];
                    descendant.visible_bindings.push(extra);
                }
                5 => descendant.visible_bindings = vec![BindingId::new(0)],
                6 => descendant.recovery = BindingContextRecovery::Recovered,
                7 => {
                    let BindingContextOwner::SourceStatement { source_range } =
                        &mut descendant.owner
                    else {
                        unreachable!();
                    };
                    source_range.end += 1;
                }
                _ => unreachable!(),
            }
            contexts.insert(descendant);
            let mut corrupted = handoff.clone();
            corrupted.binding_env = BindingEnv::try_new(BindingEnvParts {
                source_id: handoff.source_id,
                module_id: handoff.module_id.clone(),
                contexts,
                bindings,
                diagnostics: handoff.binding_env.diagnostics().clone(),
            })
            .expect("Task269SDC inexact descendant context remains structurally valid");
            corrupted.final_binding_fingerprint = corrupted.binding_env.debug_text();
            handoffs.push(corrupted);
        }
        handoffs
    }

    fn task269sdc_inexact_final_environment_handoffs(
        handoff: &SourceProofLocalGivenDescendantBindingHandoff,
        source: SourceId,
    ) -> Vec<SourceProofLocalGivenDescendantBindingHandoff> {
        let mut handoffs = Vec::new();
        for mutate in 0..13 {
            let mut corrupted = handoff.clone();
            let binding = corrupted
                .binding_env
                .binding_mut_for_test(BindingId::new(0))
                .expect("Task269SDC final reserve binding");
            match mutate {
                0 => binding.id = BindingId::new(2),
                1 => binding.spelling = "wrong".to_owned(),
                2 => binding.kind = BindingKind::GivenWitness,
                3 => {
                    binding.identity = BinderIdentity::ReservedVariable {
                        spelling: "wrong".to_owned(),
                        declaration_range: range(source, 8, 9),
                    }
                }
                4 => {
                    binding.identity = BinderIdentity::ReservedVariable {
                        spelling: "x".to_owned(),
                        declaration_range: range(source, 7, 9),
                    }
                }
                5 => binding.owner_context = BindingContextId::new(1),
                6 => binding.declaration_range = range(source, 7, 9),
                7 => binding.visible_after_ordinal = 1,
                8 => binding.type_site = BindingTypeSite::Missing,
                9 => binding.status = BindingStatus::Active,
                10 => {
                    binding.captured =
                        CapturedFreeVariables::new(vec![BinderIdentity::ReservedVariable {
                            spelling: "x".to_owned(),
                            declaration_range: range(source, 8, 9),
                        }])
                }
                11 => binding.diagnostics = vec![BindingDiagnosticId::new(0)],
                12 => binding.recovery = BindingRecoveryState::Recovered,
                _ => unreachable!(),
            }
            corrupted.final_binding_fingerprint = corrupted.binding_env.debug_text();
            handoffs.push(corrupted);
        }
        for mutate in 0..5 {
            let mut corrupted = handoff.clone();
            let binding = corrupted
                .binding_env
                .binding_mut_for_test(BindingId::new(1))
                .expect("Task269SDC semantic-free Given binding");
            match mutate {
                0 => binding.kind = BindingKind::Generated,
                1 => binding.type_site = BindingTypeSite::Source(range(source, 95, 98)),
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
        for mutate in 0..3 {
            let mut contexts = BindingContextTable::new();
            let mut root = BindingContextDraft {
                owner: BindingContextOwner::Module,
                parent: None,
                layer: BindingContextLayer::Module,
                lexical_scope: None,
                bindings: vec![BindingId::new(0)],
                visible_bindings: vec![BindingId::new(0)],
                recovery: BindingContextRecovery::Normal,
            };
            match mutate {
                0 => root.lexical_scope = Some(LocalTermScope::new(Vec::new())),
                1 => root.visible_bindings.clear(),
                2 => root.recovery = BindingContextRecovery::Recovered,
                _ => unreachable!(),
            }
            contexts.insert(root);
            for context_id in [BindingContextId::new(1), BindingContextId::new(2)] {
                let context = handoff
                    .binding_env
                    .contexts()
                    .get(context_id)
                    .expect("Task269SDC final context");
                contexts.insert(BindingContextDraft {
                    owner: context.owner.clone(),
                    parent: context.parent,
                    layer: context.layer,
                    lexical_scope: context.lexical_scope.clone(),
                    bindings: context.bindings.clone(),
                    visible_bindings: context.visible_bindings.clone(),
                    recovery: context.recovery,
                });
            }
            let mut corrupted = handoff.clone();
            corrupted.binding_env = BindingEnv::try_new(BindingEnvParts {
                source_id: source,
                module_id: handoff.module_id.clone(),
                contexts,
                bindings: handoff.binding_env.bindings().clone(),
                diagnostics: handoff.binding_env.diagnostics().clone(),
            })
            .expect("Task269SDC inexact final root context remains structurally valid");
            corrupted.final_binding_fingerprint = corrupted.binding_env.debug_text();
            handoffs.push(corrupted);
        }
        let mut diagnostics = BindingDiagnosticTable::new();
        diagnostics.insert(crate::binding_env::BindingDiagnosticDraft {
            source_range: Some(range(source, 102, 103)),
            class: crate::binding_env::BindingDiagnosticClass::UnsupportedSourceShape,
            severity: crate::binding_env::BindingDiagnosticSeverity::Error,
            message_key: "task269sdc.final".to_owned(),
            recovery: crate::binding_env::BindingDiagnosticRecovery::Normal,
        });
        let mut corrupted = handoff.clone();
        corrupted.binding_env = BindingEnv::try_new(BindingEnvParts {
            source_id: source,
            module_id: handoff.module_id.clone(),
            contexts: handoff.binding_env.contexts().clone(),
            bindings: handoff.binding_env.bindings().clone(),
            diagnostics,
        })
        .expect("Task269SDC inexact final diagnostic table");
        corrupted.final_binding_fingerprint = corrupted.binding_env.debug_text();
        handoffs.push(corrupted);
        for mutate in 0..6 {
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
            let mut proof = BindingContextDraft {
                owner: BindingContextOwner::SourceStatement {
                    source_range: range(source, 73, 178),
                },
                parent: Some(BindingContextId::new(0)),
                layer: BindingContextLayer::Proof,
                lexical_scope: Some(LocalTermScope::new(vec![0])),
                bindings: vec![BindingId::new(1)],
                visible_bindings: vec![BindingId::new(0), BindingId::new(1)],
                recovery: BindingContextRecovery::Normal,
            };
            match mutate {
                0 => {
                    proof.owner = BindingContextOwner::SourceStatement {
                        source_range: range(source, 72, 178),
                    }
                }
                1 => proof.layer = BindingContextLayer::Block,
                2 => proof.lexical_scope = Some(LocalTermScope::new(vec![1])),
                3 => proof.visible_bindings = vec![BindingId::new(1)],
                4 => proof.recovery = BindingContextRecovery::Recovered,
                5 => proof.owner = BindingContextOwner::Generated("task269sdc-proof".to_owned()),
                _ => unreachable!(),
            }
            contexts.insert(proof);
            contexts.insert(BindingContextDraft {
                owner: BindingContextOwner::SourceStatement {
                    source_range: range(source, 102, 159),
                },
                parent: Some(BindingContextId::new(1)),
                layer: BindingContextLayer::Block,
                lexical_scope: Some(LocalTermScope::new(vec![0, 0])),
                bindings: Vec::new(),
                visible_bindings: vec![BindingId::new(0), BindingId::new(1)],
                recovery: BindingContextRecovery::Normal,
            });
            let mut corrupted = handoff.clone();
            corrupted.binding_env = BindingEnv::try_new(BindingEnvParts {
                source_id: source,
                module_id: handoff.module_id.clone(),
                contexts,
                bindings: handoff.binding_env.bindings().clone(),
                diagnostics: handoff.binding_env.diagnostics().clone(),
            })
            .expect("Task269SDC inexact proof context remains structurally valid");
            corrupted.final_binding_fingerprint = corrupted.binding_env.debug_text();
            handoffs.push(corrupted);
        }
        handoffs
    }

    fn task269sdc_environment_with_generated_descendant(
        handoff: &SourceProofLocalGivenDescendantBindingHandoff,
    ) -> BindingEnv {
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
                source_range: range(handoff.source_id, 73, 178),
            },
            parent: Some(BindingContextId::new(0)),
            layer: BindingContextLayer::Proof,
            lexical_scope: Some(LocalTermScope::new(vec![0])),
            bindings: vec![BindingId::new(1)],
            visible_bindings: vec![BindingId::new(0), BindingId::new(1)],
            recovery: BindingContextRecovery::Normal,
        });
        contexts.insert(BindingContextDraft {
            owner: BindingContextOwner::Generated("task269sdc-corrupt".to_owned()),
            parent: Some(BindingContextId::new(1)),
            layer: BindingContextLayer::Block,
            lexical_scope: Some(LocalTermScope::new(vec![0, 0])),
            bindings: Vec::new(),
            visible_bindings: vec![BindingId::new(0), BindingId::new(1)],
            recovery: BindingContextRecovery::Normal,
        });
        BindingEnv::try_new(BindingEnvParts {
            source_id: handoff.source_id,
            module_id: handoff.module_id.clone(),
            contexts,
            bindings: handoff.binding_env.bindings().clone(),
            diagnostics: handoff.binding_env.diagnostics().clone(),
        })
        .expect("Task269SDC generated descendant remains structurally valid")
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
        let input = SourceProofLocalGivenBindingHandoffInput {
            source_id: source,
            module_id: module.clone(),
            lower_fingerprint: task286_receipt(source, module.clone(), 1),
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
        let input = SourceProofLocalLetBindingHandoffInput {
            source_id: source,
            module_id: module.clone(),
            lower_fingerprint: task286_receipt(source, module.clone(), 0),
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

    fn binding_env_with_local_ranges(
        original: &BindingEnv,
        proof_range: SourceRange,
        name_range: SourceRange,
    ) -> BindingEnv {
        let mut bindings = original.bindings().clone();
        let binding = bindings.get_mut_for_test(BindingId::new(1)).unwrap();
        binding.declaration_range = name_range;
        let BinderIdentity::ResolverLocal {
            declaration_range, ..
        } = &mut binding.identity
        else {
            panic!("expected the proof-local resolver binding");
        };
        *declaration_range = name_range;
        let mut contexts = BindingContextTable::new();
        for (id, context) in original.contexts().iter() {
            contexts.insert(BindingContextDraft {
                owner: if id == BindingContextId::new(1) {
                    BindingContextOwner::SourceStatement {
                        source_range: proof_range,
                    }
                } else {
                    context.owner.clone()
                },
                parent: context.parent,
                layer: context.layer,
                lexical_scope: context.lexical_scope.clone(),
                bindings: context.bindings.clone(),
                visible_bindings: context.visible_bindings.clone(),
                recovery: context.recovery,
            });
        }
        BindingEnv::try_new(BindingEnvParts {
            source_id: original.source_id(),
            module_id: original.module_id().clone(),
            contexts,
            bindings,
            diagnostics: original.diagnostics().clone(),
        })
        .expect("coherent proof/name range corruption remains structurally valid")
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

    #[test]
    fn native_conversion_local_attribute_authority_is_source_point_checked() {
        use SurfaceNodeKind as K;
        use mizar_resolve::{
            declarations::DeclarationShellCollector,
            env::{NamespacePath, SymbolKind},
            names::{SourceVariableScopeInput, SourceVariableScopeResolver},
            symbols::SignatureProjectionExtractor,
        };
        let inputs = |predicate: bool, future: bool| {
            let source = source_id();
            let module = ModuleId::new(PackageId::new("pkg"), ModulePath::new("attribute_trust"));
            let definition = if predicate {
                vec![
                    "definition",
                    "pred",
                    "Marked",
                    ":",
                    "marked1",
                    "means",
                    "thesis",
                    ";",
                    "end",
                    ";",
                ]
            } else {
                vec![
                    "definition",
                    "let",
                    "d",
                    "be",
                    "set",
                    ";",
                    "attr",
                    "Marked",
                    ":",
                    "d",
                    "is",
                    "marked1",
                    "means",
                    "d",
                    "=",
                    "d",
                    ";",
                    "end",
                    ";",
                ]
            };
            let theorem = vec![
                "theorem",
                "T",
                ":",
                "thesis",
                "proof",
                "let",
                "s",
                "be",
                "set",
                ";",
                "reconsider",
                "a",
                "=",
                "s",
                "as",
                "marked1",
                "set",
                ";",
                "end",
                ";",
            ];
            let sections = [definition, theorem];
            let mut builder = SurfaceAstBuilder::new(source);
            let mut tokens = [Vec::new(), Vec::new()];
            let mut spans = [Vec::new(), Vec::new()];
            let mut cursor = 0;
            for section in if future { [1, 0] } else { [0, 1] } {
                for word in &sections[section] {
                    let span = range(source, cursor, cursor + word.len());
                    cursor += word.len() + 1;
                    let kind = if *word == "marked1" && (section == 0 || !future) {
                        SurfaceTokenKind::UserSymbol
                    } else if matches!(*word, "T" | "Marked" | "d" | "s" | "a" | "marked1") {
                        SurfaceTokenKind::Identifier
                    } else if matches!(*word, ":" | ";" | "=") {
                        SurfaceTokenKind::ReservedSymbol
                    } else {
                        SurfaceTokenKind::ReservedWord
                    };
                    tokens[section].push(builder.add_token(kind, *word, span));
                    spans[section].push(span);
                }
            }
            let node = |builder: &mut SurfaceAstBuilder,
                        kind,
                        section: usize,
                        first: usize,
                        last: usize,
                        children| {
                builder.add_node(
                    kind,
                    range(
                        source,
                        spans[section][first].start,
                        spans[section][last].end,
                    ),
                    children,
                )
            };
            let d = &tokens[0];
            let declaration = if predicate {
                let pattern = node(&mut builder, K::PredicatePattern, 0, 4, 4, vec![d[4]]);
                let thesis = node(
                    &mut builder,
                    K::FormulaConstant(SurfaceFormulaConstant::Thesis),
                    0,
                    6,
                    6,
                    vec![d[6]],
                );
                let formula = node(&mut builder, K::FormulaExpression, 0, 6, 6, vec![thesis]);
                let body = node(&mut builder, K::FormulaDefiniens, 0, 6, 6, vec![formula]);
                let definition = node(
                    &mut builder,
                    K::PredicateDefinition,
                    0,
                    1,
                    7,
                    vec![d[1], d[2], d[3], pattern, d[5], body, d[7]],
                );
                node(
                    &mut builder,
                    K::DefinitionBlockItem,
                    0,
                    0,
                    9,
                    vec![d[0], definition, d[8], d[9]],
                )
            } else {
                let head = node(&mut builder, K::TypeHead, 0, 4, 4, vec![d[4]]);
                let ty = node(&mut builder, K::TypeExpression, 0, 4, 4, vec![head]);
                let segment = node(
                    &mut builder,
                    K::QualifiedVariableSegment,
                    0,
                    2,
                    4,
                    vec![d[2], d[3], ty],
                );
                let parameter = node(
                    &mut builder,
                    K::DefinitionParameter,
                    0,
                    1,
                    5,
                    vec![d[1], segment, d[5]],
                );
                let pattern = node(&mut builder, K::AttributePattern, 0, 11, 11, vec![d[11]]);
                let left_ref = node(&mut builder, K::TermReference, 0, 13, 13, vec![d[13]]);
                let left = node(&mut builder, K::TermExpression, 0, 13, 13, vec![left_ref]);
                let right_ref = node(&mut builder, K::TermReference, 0, 15, 15, vec![d[15]]);
                let right = node(&mut builder, K::TermExpression, 0, 15, 15, vec![right_ref]);
                let equality = node(
                    &mut builder,
                    K::BuiltinPredicateApplication,
                    0,
                    13,
                    15,
                    vec![left, d[14], right],
                );
                let formula = node(
                    &mut builder,
                    K::FormulaExpression,
                    0,
                    13,
                    15,
                    vec![equality],
                );
                let body = node(&mut builder, K::FormulaDefiniens, 0, 13, 15, vec![formula]);
                let definition = node(
                    &mut builder,
                    K::AttributeDefinition,
                    0,
                    6,
                    16,
                    vec![d[6], d[7], d[8], d[9], d[10], pattern, d[12], body, d[16]],
                );
                node(
                    &mut builder,
                    K::DefinitionBlockItem,
                    0,
                    0,
                    18,
                    vec![d[0], parameter, definition, d[17], d[18]],
                )
            };
            let t = &tokens[1];
            let head = node(&mut builder, K::TypeHead, 1, 8, 8, vec![t[8]]);
            let ty = node(&mut builder, K::TypeExpression, 1, 8, 8, vec![head]);
            let segment = node(
                &mut builder,
                K::QualifiedVariableSegment,
                1,
                6,
                8,
                vec![t[6], t[7], ty],
            );
            let let_statement = node(
                &mut builder,
                K::LetStatement,
                1,
                5,
                9,
                vec![t[5], segment, t[9]],
            );
            let reference = node(&mut builder, K::TermReference, 1, 13, 13, vec![t[13]]);
            let value = node(&mut builder, K::TermExpression, 1, 13, 13, vec![reference]);
            let item = node(
                &mut builder,
                K::ReconsiderItem,
                1,
                11,
                13,
                vec![t[11], t[12], value],
            );
            let segment = node(&mut builder, K::PathSegment, 1, 15, 15, vec![t[15]]);
            let qualified = node(&mut builder, K::QualifiedSymbol, 1, 15, 15, vec![segment]);
            let attribute = node(&mut builder, K::AttributeRef, 1, 15, 15, vec![qualified]);
            let chain = node(&mut builder, K::AttributeChain, 1, 15, 15, vec![attribute]);
            let head = node(&mut builder, K::TypeHead, 1, 16, 16, vec![t[16]]);
            let target = node(
                &mut builder,
                K::TypeExpression,
                1,
                15,
                16,
                vec![chain, head],
            );
            let conversion = node(
                &mut builder,
                K::ReconsiderStatement,
                1,
                10,
                17,
                vec![t[10], item, t[14], target, t[17]],
            );
            let thesis = node(
                &mut builder,
                K::FormulaConstant(SurfaceFormulaConstant::Thesis),
                1,
                3,
                3,
                vec![t[3]],
            );
            let formula = node(&mut builder, K::FormulaExpression, 1, 3, 3, vec![thesis]);
            let proof = node(
                &mut builder,
                K::ProofBlock,
                1,
                4,
                18,
                vec![t[4], let_statement, conversion, t[18]],
            );
            let theorem = node(
                &mut builder,
                K::TheoremItem,
                1,
                0,
                19,
                vec![t[0], t[1], t[2], formula, proof, t[19]],
            );
            let order = if future {
                vec![theorem, declaration]
            } else {
                vec![declaration, theorem]
            };
            let span = range(source, 0, cursor - 1);
            let items = builder.add_node(K::ItemList, span, order);
            let compilation = builder.add_node(K::CompilationUnit, span, vec![items]);
            let mut root_children = Vec::new();
            for section in if future { [1, 0] } else { [0, 1] } {
                root_children.extend_from_slice(&tokens[section]);
            }
            root_children.push(compilation);
            let root = builder.add_node(K::Root, span, root_children);
            let ast = builder.finish(Some(root), None);
            let native = SurfaceResolvedArena::lower(&ast, &module).unwrap();
            let shells = DeclarationShellCollector::new(&ast, &module).collect();
            let collection = SignatureProjectionExtractor::new(
                &ast,
                &shells,
                NamespacePath::new(module.path().as_str()),
            )
            .collect(&module);
            assert!(
                collection.diagnostics().is_empty(),
                "{:?}",
                collection.diagnostics()
            );
            let symbols = collection.env().clone();
            let scope = SourceVariableScopeResolver::resolve_proof_bindings(
                SourceVariableScopeInput::new(&ast, &module, &symbols),
            )
            .unwrap();
            let mut original = TypedAst::try_new(TypedAstParts {
                source_id: source,
                module_id: module,
                resolved_root: Some(native.arena().root()),
                source_context: None,
                source_type: None,
                source_attribute: None,
                nodes: TypedArena::try_new(
                    Some(TypedNodeId::new(native.arena().root().index())),
                    native
                        .arena()
                        .iter()
                        .map(|(id, node)| {
                            TypedNode::new("syntax", node.origin().anchor().clone())
                                .with_resolved_node(id)
                                .with_children(
                                    node.children()
                                        .iter()
                                        .map(|id| TypedNodeId::new(id.index()))
                                        .collect(),
                                )
                        })
                        .collect(),
                )
                .unwrap(),
                contexts: Default::default(),
                types: Default::default(),
                facts: Default::default(),
                coercions: Default::default(),
                initial_obligations: Default::default(),
                diagnostics: Default::default(),
            })
            .unwrap();
            original.install_source_nodes(&native).unwrap();
            SourceProofLocalDeclarationProducer::build_bindings(
                &original, &native, &symbols, &scope,
            )
            .unwrap()
        };
        let genuine = inputs(false, false);
        let attribute = genuine
            .symbols
            .symbols()
            .iter()
            .find(|entry| entry.kind() == SymbolKind::Attribute)
            .unwrap();
        assert_eq!(
            genuine
                .symbols
                .definitions()
                .by_symbol(attribute.symbol())
                .unwrap()
                .arity(),
            Some(0)
        );
        let (requests, checked, queries) = genuine.check_conversions().unwrap();
        assert_eq!(requests.len(), 1);
        assert_eq!(
            requests[0].to_type.attributes[0].symbol,
            *attribute.symbol()
        );
        assert_eq!(
            queries[0].status(),
            crate::type_checker::TypeFactQueryStatus::Missing
        );
        assert_eq!(
            checked.coercions().iter().next().unwrap().1.status,
            crate::typed_ast::CoercionStatus::Rejected
        );
        assert!(checked.initial_obligations().is_empty());
        let wrong_kind = inputs(true, false);
        assert!(
            wrong_kind
                .symbols
                .symbols()
                .iter()
                .any(|entry| entry.kind() == SymbolKind::Predicate
                    && entry.primary_spelling() == "marked1")
        );
        let future = inputs(false, true);
        for handoff in [&wrong_kind, &future] {
            assert_eq!(
                handoff.check_conversions(),
                Err(SourceProofLocalDeclarationError::DependencyMismatch)
            );
        }
        let mut mismatched = genuine.clone();
        mismatched.symbols = wrong_kind.symbols;
        assert_eq!(
            mismatched.check_conversions(),
            Err(SourceProofLocalDeclarationError::DependencyMismatch)
        );
    }

    #[test]
    fn native_proof_element_rules_preserve_borrowed_roles_and_refuse_isolated_shapes() {
        use SurfaceNodeKind as K;
        use SurfaceTokenKind as T;
        let source = source_id();
        let receipt = task286_receipt(
            source,
            ModuleId::new(PackageId::new("pkg"), ModulePath::new("proof_roles")),
            4,
        );
        let prototype = receipt
            .original
            .iter()
            .find(|row| matches!(row.kind(), K::Token(_)))
            .unwrap();
        let sites = receipt
            .original
            .iter()
            .map(SourceNode::site)
            .collect::<Vec<_>>();
        let token = |kind, text: &str| {
            let K::Token(mut t) = prototype.kind.clone() else {
                unreachable!()
            };
            t.kind = kind;
            t.text = text.into();
            K::Token(t)
        };
        let rows = |descriptors: Vec<(K, Option<usize>, usize)>| {
            descriptors
                .into_iter()
                .enumerate()
                .map(|(index, (kind, parent, ordinal))| {
                    let mut row = prototype.clone();
                    row.spelling = match &kind {
                        K::Token(t) => t.text.to_string(),
                        _ => String::new(),
                    };
                    row.kind = kind;
                    row.site = sites[index];
                    row.parent = parent.map(|i| sites[i]);
                    row.child_ordinal = ordinal;
                    row.preorder_ordinal = index;
                    row.range = range(source, 0, 100);
                    row
                })
                .collect::<Vec<_>>()
        };
        let roles = |case: usize, rows: &[SourceNode]| {
            let owner = &rows[if matches!(case, 4 | 5 | 9 | 11) { 1 } else { 0 }];
            let borrowed = match case {
                0 => source_given_roles(rows, owner),
                1 => source_consider_roles(rows, owner),
                2 => source_now_roles(rows, owner),
                3 => source_hereby_roles(rows, owner),
                4 => source_case_item_roles(rows, owner),
                5 => source_suppose_item_roles(rows, owner),
                6 => source_case_reasoning_roles(rows, owner),
                7 => source_set_roles(rows, owner),
                8 => source_then_roles(rows, owner).map(|row| vec![row]),
                9 => source_statement_item_roles(rows, owner).map(|row| vec![row]),
                10 => source_iterative_equality_roles(rows, owner),
                11 => source_iterative_equality_step_roles(rows, owner),
                _ => unreachable!(),
            };
            borrowed.map(|rows| rows.into_iter().map(SourceNode::site).collect::<Vec<_>>())
        };
        let mut frames = Vec::new();
        for (kind, word, by) in [
            (K::GivenStatement, "given", false),
            (K::ConsiderStatement, "consider", true),
        ] {
            let mut frame = vec![
                (kind, None, 0),
                (token(T::ReservedWord, word), Some(0), 0),
                (K::QualifiedVariableSegment, Some(0), 1),
                (token(T::Identifier, "x"), Some(2), 0),
                (token(T::ReservedWord, "be"), Some(2), 1),
                (K::TypeExpression, Some(2), 2),
                (token(T::ReservedWord, "such"), Some(0), 2),
                (K::ConditionList, Some(0), 3),
            ];
            if by {
                frame.push((K::JustificationClause, Some(0), 4));
            }
            frame.push((
                token(T::ReservedSymbol, ";"),
                Some(0),
                if by { 5 } else { 4 },
            ));
            frames.push(frame);
        }
        for (kind, word) in [(K::NowStatement, "now"), (K::HerebyStatement, "hereby")] {
            frames.push(vec![
                (kind, None, 0),
                (token(T::ReservedWord, word), Some(0), 0),
                (K::CompactStatement, Some(0), 1),
                (token(T::ReservedWord, "end"), Some(0), 2),
                (token(T::ReservedSymbol, ";"), Some(0), 3),
            ]);
        }
        for (kind, word) in [(K::CaseItem, "case"), (K::SupposeItem, "suppose")] {
            frames.push(vec![
                (K::CaseReasoningStatement, None, 0),
                (kind, Some(0), 0),
                (token(T::ReservedWord, word), Some(1), 0),
                (K::Proposition, Some(1), 1),
                (token(T::ReservedSymbol, ";"), Some(1), 2),
                (K::CompactStatement, Some(1), 3),
                (token(T::ReservedWord, "end"), Some(1), 4),
                (token(T::ReservedSymbol, ";"), Some(1), 5),
            ]);
        }
        frames.push(vec![
            (K::CaseReasoningStatement, None, 0),
            (token(T::ReservedWord, "per"), Some(0), 0),
            (token(T::ReservedWord, "cases"), Some(0), 1),
            (K::JustificationClause, Some(0), 2),
            (token(T::ReservedSymbol, ";"), Some(0), 3),
            (K::CaseItem, Some(0), 4),
            (K::CaseItem, Some(0), 5),
        ]);
        frames.push(vec![
            (K::SetStatement, None, 0),
            (token(T::ReservedWord, "set"), Some(0), 0),
            (K::Equating, Some(0), 1),
            (token(T::ReservedSymbol, ";"), Some(0), 2),
            (token(T::Identifier, "x"), Some(2), 0),
            (token(T::ReservedSymbol, "="), Some(2), 1),
            (K::TermExpression, Some(2), 2),
        ]);
        frames.push(vec![
            (K::ThenStatement, None, 0),
            (token(T::ReservedWord, "then"), Some(0), 0),
            (K::CompactStatement, Some(0), 1),
        ]);
        frames.push(vec![
            (K::ItemList, None, 0),
            (K::StatementItem, Some(0), 0),
            (K::CompactStatement, Some(1), 0),
        ]);
        frames.push(vec![
            (K::IterativeEqualityStatement, None, 0),
            (K::TermExpression, Some(0), 0),
            (token(T::ReservedSymbol, "="), Some(0), 1),
            (K::TermExpression, Some(0), 2),
            (K::JustificationClause, Some(0), 3),
            (K::IterativeEqualityStep, Some(0), 4),
            (token(T::ReservedSymbol, ";"), Some(0), 5),
        ]);
        frames.push(vec![
            (K::IterativeEqualityStatement, None, 0),
            (K::IterativeEqualityStep, Some(0), 0),
            (token(T::ReservedSymbol, ".="), Some(1), 0),
            (K::TermExpression, Some(1), 1),
            (K::JustificationClause, Some(1), 2),
        ]);
        let expected = [
            vec![2, 7],
            vec![2, 7, 8],
            vec![2],
            vec![2],
            vec![3, 5],
            vec![3, 5],
            vec![3, 5, 6],
            vec![2],
            vec![2],
            vec![2],
            vec![1, 3, 4, 5],
            vec![3, 4],
        ];
        for (case, frame) in frames.into_iter().enumerate() {
            let original = rows(frame);
            assert_eq!(
                roles(case, &original),
                Some(expected[case].iter().map(|index| sites[*index]).collect()),
                "case {case}"
            );
            let owner = if matches!(case, 4 | 5 | 9 | 11) { 1 } else { 0 };
            let mut bad = original.clone();
            bad[owner].kind = K::Proposition;
            assert!(roles(case, &bad).is_none(), "kind {case}");
            let first = original
                .iter()
                .position(|row| row.parent() == Some(original[owner].site()))
                .unwrap();
            let mut bad = original.clone();
            bad[first].child_ordinal = 1;
            assert!(roles(case, &bad).is_none(), "order {case}");
            if case != 9 {
                let keyword = if case == 10 { 2 } else { first };
                let K::Token(t) = original[keyword].kind() else {
                    unreachable!()
                };
                for kind in [token(T::Identifier, &t.text), token(t.kind, "wrong")] {
                    let mut bad = original.clone();
                    bad[keyword].kind = kind;
                    assert!(roles(case, &bad).is_none(), "token {case}");
                }
            }
            let mut bad = original.clone();
            let mut extra = prototype.clone();
            extra.kind = K::Proposition;
            extra.site = sites[bad.len()];
            extra.parent = Some(bad[owner].site());
            extra.child_ordinal = source_children(&bad, bad[owner].site()).count();
            bad.push(extra);
            assert!(roles(case, &bad).is_none(), "tail {case}");
            if matches!(case, 4 | 5 | 9 | 11) {
                let mut bad = original.clone();
                bad[owner].parent = None;
                assert!(roles(case, &bad).is_none(), "orphan {case}");
            }
            if case == 6 {
                let mut bad = original.clone();
                bad[6].kind = K::SupposeItem;
                assert!(roles(case, &bad).is_none());
            }
            if case == 0 {
                let mut bad = original.clone();
                let mut by = prototype.clone();
                by.kind = K::JustificationClause;
                by.site = sites[9];
                by.parent = Some(sites[0]);
                by.child_ordinal = 4;
                bad[8].child_ordinal = 5;
                bad.insert(8, by);
                assert!(roles(case, &bad).is_none());
            }
            if case == 1 {
                let mut bad = original.clone();
                bad.remove(8);
                bad[8].child_ordinal = 4;
                assert!(roles(case, &bad).is_none());
            }
            if case == 7 {
                let mut bad = original.clone();
                bad[6].kind = K::TypeExpression;
                assert!(roles(case, &bad).is_none());
            }
            if case == 8 {
                let mut bad = original.clone();
                bad[2].kind = K::LetStatement;
                assert!(roles(case, &bad).is_none());
            }
            if case == 11 {
                let mut bad = original.clone();
                bad[4].kind = K::ProofBlock;
                assert!(roles(case, &bad).is_none());
            }
        }
    }

    #[test]
    fn native_proof_intents_require_take_assumption_and_given_source_frames() {
        use SurfaceNodeKind as K;
        let template = |mutation: usize| {
            let source = source_id();
            let module = ModuleId::new(PackageId::new("pkg"), ModulePath::new("proof_frames"));
            let sections = [
                vec!["reserve", "x", "for", "set", ";"],
                vec!["theorem", "T", ":", "thesis", "proof"],
                if mutation == 2 {
                    vec!["assume", "A", ":", "thesis", "by", "A", ";"]
                } else {
                    vec!["assume", "A", ":", "thesis", ";"]
                },
                vec!["take", "x", ";"],
                if mutation == 3 {
                    vec!["given", "y", "be", "set", "by", "A", ";"]
                } else {
                    vec!["given", "y", "be", "set", ";"]
                },
                vec!["thus", "thesis", "by", "A", ";"],
                vec!["end", ";"],
            ];
            let mut builder = SurfaceAstBuilder::new(source);
            let mut tokens = vec![Vec::new(); sections.len()];
            let mut spans = vec![Vec::new(); sections.len()];
            let mut cursor = 0;
            for (section, words) in sections.iter().enumerate() {
                for (index, word) in words.iter().enumerate() {
                    let span = range(source, cursor, cursor + word.len());
                    cursor += word.len() + 1;
                    let kind = if matches!(*word, "x" | "y" | "T" | "A")
                        || mutation == 1 && section == 3 && index == 0
                    {
                        SurfaceTokenKind::Identifier
                    } else if matches!(*word, ":" | ";") {
                        SurfaceTokenKind::ReservedSymbol
                    } else {
                        SurfaceTokenKind::ReservedWord
                    };
                    tokens[section].push(builder.add_token(kind, *word, span));
                    spans[section].push(span);
                }
            }
            let node = |b: &mut SurfaceAstBuilder,
                        kind,
                        section: usize,
                        first: usize,
                        last: usize,
                        children| {
                b.add_node(
                    kind,
                    range(
                        source,
                        spans[section][first].start,
                        spans[section][last].end,
                    ),
                    children,
                )
            };
            let reserve_head = node(&mut builder, K::TypeHead, 0, 3, 3, vec![tokens[0][3]]);
            let reserve_type = node(&mut builder, K::TypeExpression, 0, 3, 3, vec![reserve_head]);
            let reserve_segment = node(
                &mut builder,
                K::ReserveSegment,
                0,
                1,
                3,
                vec![tokens[0][1], tokens[0][2], reserve_type],
            );
            let reserve = node(
                &mut builder,
                K::ReserveItem,
                0,
                0,
                4,
                vec![tokens[0][0], reserve_segment, tokens[0][4]],
            );
            let mut formulas = BTreeMap::new();
            for (section, index) in [(1, 3), (2, 3), (5, 1)] {
                let thesis = node(
                    &mut builder,
                    K::FormulaConstant(SurfaceFormulaConstant::Thesis),
                    section,
                    index,
                    index,
                    vec![tokens[section][index]],
                );
                formulas.insert(
                    section,
                    node(
                        &mut builder,
                        K::FormulaExpression,
                        section,
                        index,
                        index,
                        vec![thesis],
                    ),
                );
            }
            let mut tails = BTreeMap::new();
            for section in [2, 4, 5] {
                if section != 5 && sections[section].len() == 5 {
                    continue;
                }
                let index = if section == 5 { 3 } else { 5 };
                let reference = node(
                    &mut builder,
                    K::Reference,
                    section,
                    index,
                    index,
                    vec![tokens[section][index]],
                );
                let list = node(
                    &mut builder,
                    K::ReferenceList,
                    section,
                    index,
                    index,
                    vec![reference],
                );
                tails.insert(
                    section,
                    node(
                        &mut builder,
                        K::JustificationClause,
                        section,
                        index - 1,
                        index,
                        vec![tokens[section][index - 1], list],
                    ),
                );
            }
            let assumption_proposition = node(
                &mut builder,
                K::Proposition,
                2,
                1,
                3,
                vec![tokens[2][1], tokens[2][2], formulas[&2]],
            );
            let mut children = vec![tokens[2][0], assumption_proposition];
            children.extend(tails.get(&2));
            children.push(*tokens[2].last().unwrap());
            let assumption = node(
                &mut builder,
                K::AssumptionStatement,
                2,
                0,
                sections[2].len() - 1,
                children,
            );
            let reference = node(&mut builder, K::TermReference, 3, 1, 1, vec![tokens[3][1]]);
            let term = node(&mut builder, K::TermExpression, 3, 1, 1, vec![reference]);
            let witness = node(&mut builder, K::Witness, 3, 1, 1, vec![term]);
            let take = node(
                &mut builder,
                K::TakeStatement,
                3,
                0,
                2,
                vec![tokens[3][0], witness, tokens[3][2]],
            );
            let head = node(&mut builder, K::TypeHead, 4, 3, 3, vec![tokens[4][3]]);
            let ty = node(&mut builder, K::TypeExpression, 4, 3, 3, vec![head]);
            let segment = node(
                &mut builder,
                K::QualifiedVariableSegment,
                4,
                1,
                3,
                vec![tokens[4][1], tokens[4][2], ty],
            );
            let mut children = vec![tokens[4][0], segment];
            children.extend(tails.get(&4));
            children.push(*tokens[4].last().unwrap());
            let given = node(
                &mut builder,
                K::GivenStatement,
                4,
                0,
                sections[4].len() - 1,
                children,
            );
            let proposition = node(&mut builder, K::Proposition, 5, 1, 1, vec![formulas[&5]]);
            let conclusion = node(
                &mut builder,
                K::ConclusionStatement,
                5,
                0,
                4,
                vec![tokens[5][0], proposition, tails[&5], tokens[5][4]],
            );
            let proof = builder.add_node(
                K::ProofBlock,
                range(source, spans[1][4].start, spans[6][0].end),
                vec![
                    tokens[1][4],
                    assumption,
                    take,
                    given,
                    conclusion,
                    tokens[6][0],
                ],
            );
            let theorem = builder.add_node(
                K::TheoremItem,
                range(source, spans[1][0].start, spans[6][1].end),
                vec![
                    tokens[1][0],
                    tokens[1][1],
                    tokens[1][2],
                    formulas[&1],
                    proof,
                    tokens[6][1],
                ],
            );
            let span = range(source, 0, cursor - 1);
            let items = builder.add_node(K::ItemList, span, vec![reserve, theorem]);
            let compilation = builder.add_node(K::CompilationUnit, span, vec![items]);
            let mut roots = tokens.into_iter().flatten().collect::<Vec<_>>();
            roots.push(compilation);
            let root = builder.add_node(K::Root, span, roots);
            let ast = builder.finish(Some(root), None);
            let native = SurfaceResolvedArena::lower(&ast, &module).unwrap();
            let mut original = TypedAst::try_new(TypedAstParts {
                source_id: source,
                module_id: module,
                resolved_root: Some(native.arena().root()),
                source_context: None,
                source_type: None,
                source_attribute: None,
                nodes: TypedArena::try_new(
                    Some(TypedNodeId::new(native.arena().root().index())),
                    native
                        .arena()
                        .iter()
                        .map(|(id, row)| {
                            TypedNode::new("syntax", row.origin().anchor().clone())
                                .with_resolved_node(id)
                                .with_children(
                                    row.children()
                                        .iter()
                                        .map(|id| TypedNodeId::new(id.index()))
                                        .collect(),
                                )
                        })
                        .collect(),
                )
                .unwrap(),
                contexts: Default::default(),
                types: Default::default(),
                facts: Default::default(),
                coercions: Default::default(),
                initial_obligations: Default::default(),
                diagnostics: Default::default(),
            })
            .unwrap();
            original.install_source_nodes(&native).unwrap();
            original
        };
        for mutation in 0..4 {
            let (original, native, symbols, scope) =
                native_binding_inputs(&template(mutation), false);
            let built = SourceProofLocalDeclarationProducer::build_bindings(
                &original, &native, &symbols, &scope,
            );
            if mutation != 0 {
                assert_eq!(
                    built.and_then(|handoff| handoff.check_proof_intents().map(|_| ())),
                    Err(SourceProofLocalDeclarationError::DependencyMismatch),
                    "mutation {mutation}"
                );
                continue;
            }
            let handoff = built.unwrap();
            let (intents, labels) = handoff.check_proof_intents().unwrap();
            assert!(
                intents
                    .iter()
                    .all(|(_, _, status)| *status == SourceProofIntentStatus::Pending)
            );
            assert!(!labels.has_unresolved());
            for kind in [K::TakeStatement, K::AssumptionStatement, K::GivenStatement] {
                let row = original
                    .source_node_entries()
                    .unwrap()
                    .find(|row| row.kind() == &kind)
                    .unwrap();
                let site = original
                    .nodes()
                    .iter()
                    .find(|(_, node)| node.resolved_node == Some(row.site()))
                    .unwrap()
                    .0;
                assert!(intents.iter().any(|(actual, _, _)| actual.node() == site));
            }
            let installed = empty_typed(original.source_id(), original.module_id().clone())
                .with_source_proof_local_binding(handoff)
                .unwrap();
            assert!(
                installed.types().is_empty()
                    && installed.facts().is_empty()
                    && installed.coercions().is_empty()
            );
            let resolved = assemble_empty(&installed).unwrap();
            assert!(resolved.checked_proofs().is_empty() && resolved.checked_formulas().is_empty());
        }
    }
    #[test]
    fn task272b1_witness_goals_replay_real_typed_results_and_coherent_mutations() {
        use crate::{
            type_checker::{CoercionObligationChecker, TermFormulaChecker, TermReference},
            typed_ast::{CoercionStatus, InitialObligationStatus},
        };
        use SurfaceNodeKind as K;
        use mizar_resolve::{
            declarations::DeclarationShellCollector,
            env::NamespacePath,
            names::{SourceVariableScopeInput, SourceVariableScopeResolver},
            symbols::SignatureProjectionExtractor,
        };
        use mizar_syntax::ast::SurfaceFormulaBinaryOperator;
        let fixture = |swapped: bool| {
            let source = source_id();
            let module = ModuleId::new(PackageId::new("pkg"), ModulePath::new("witness_replay"));
            let words = [
                "theorem",
                "T",
                ":",
                "for",
                "s",
                "being",
                "set",
                ",",
                "o",
                "being",
                "object",
                "ex",
                "a",
                "being",
                "set",
                ",",
                "b",
                "being",
                "object",
                "st",
                "a",
                "=",
                "s",
                "&",
                "b",
                "=",
                "o",
                "proof",
                "let",
                "s",
                "be",
                "set",
                ";",
                "let",
                "o",
                "be",
                "object",
                ";",
                "take",
                "k",
                "=",
                if swapped { "o" } else { "s" },
                ",",
                if swapped { "s" } else { "o" },
                ";",
                "thus",
                "thesis",
                ";",
                "end",
                ";",
            ];
            let mut builder = SurfaceAstBuilder::new(source);
            let mut spans = Vec::new();
            let mut tokens = Vec::new();
            let mut cursor = 0;
            for word in words {
                let span = range(source, cursor, cursor + word.len());
                cursor += word.len() + 1;
                let kind = if matches!(word, "T" | "s" | "o" | "a" | "b" | "k") {
                    SurfaceTokenKind::Identifier
                } else if matches!(word, ":" | "," | "=" | "&" | ";") {
                    SurfaceTokenKind::ReservedSymbol
                } else {
                    SurfaceTokenKind::ReservedWord
                };
                tokens.push(builder.add_token(kind, word, span));
                spans.push(span);
            }
            let node = |b: &mut SurfaceAstBuilder, kind, start: usize, end: usize, children| {
                b.add_node(
                    kind,
                    range(source, spans[start].start, spans[end].end),
                    children,
                )
            };
            let mut segments = BTreeMap::new();
            for (i, kind) in [
                (4, K::QuantifierVariableSegment),
                (8, K::QuantifierVariableSegment),
                (12, K::QuantifierVariableSegment),
                (16, K::QuantifierVariableSegment),
                (29, K::QualifiedVariableSegment),
                (34, K::QualifiedVariableSegment),
            ] {
                let head = node(&mut builder, K::TypeHead, i + 2, i + 2, vec![tokens[i + 2]]);
                let ty = node(&mut builder, K::TypeExpression, i + 2, i + 2, vec![head]);
                segments.insert(
                    i,
                    node(
                        &mut builder,
                        kind,
                        i,
                        i + 2,
                        vec![tokens[i], tokens[i + 1], ty],
                    ),
                );
            }
            let mut values = BTreeMap::new();
            for i in [20, 22, 24, 26, 41, 43] {
                let reference = node(&mut builder, K::TermReference, i, i, vec![tokens[i]]);
                values.insert(
                    i,
                    node(&mut builder, K::TermExpression, i, i, vec![reference]),
                );
            }
            let eq1 = node(
                &mut builder,
                K::BuiltinPredicateApplication,
                20,
                22,
                vec![values[&20], tokens[21], values[&22]],
            );
            let eq2 = node(
                &mut builder,
                K::BuiltinPredicateApplication,
                24,
                26,
                vec![values[&24], tokens[25], values[&26]],
            );
            let and = node(
                &mut builder,
                K::BinaryFormula(SurfaceFormulaBinaryOperator {
                    connective: SurfaceFormulaConnective::And,
                    repeated: false,
                }),
                20,
                26,
                vec![eq1, tokens[23], eq2],
            );
            let ex = node(
                &mut builder,
                K::QuantifiedFormula(SurfaceQuantifierKind::Existential),
                11,
                26,
                vec![
                    tokens[11],
                    segments[&12],
                    tokens[15],
                    segments[&16],
                    tokens[19],
                    and,
                ],
            );
            let forall = node(
                &mut builder,
                K::QuantifiedFormula(SurfaceQuantifierKind::Universal),
                3,
                26,
                vec![tokens[3], segments[&4], tokens[7], segments[&8], ex],
            );
            let header = node(&mut builder, K::FormulaExpression, 3, 26, vec![forall]);
            let let1 = node(
                &mut builder,
                K::LetStatement,
                28,
                32,
                vec![tokens[28], segments[&29], tokens[32]],
            );
            let let2 = node(
                &mut builder,
                K::LetStatement,
                33,
                37,
                vec![tokens[33], segments[&34], tokens[37]],
            );
            let first = node(
                &mut builder,
                K::Witness,
                39,
                41,
                vec![tokens[39], tokens[40], values[&41]],
            );
            let second = node(&mut builder, K::Witness, 43, 43, vec![values[&43]]);
            let take = node(
                &mut builder,
                K::TakeStatement,
                38,
                44,
                vec![tokens[38], first, tokens[42], second, tokens[44]],
            );
            let thesis = node(
                &mut builder,
                K::FormulaConstant(SurfaceFormulaConstant::Thesis),
                46,
                46,
                vec![tokens[46]],
            );
            let terminal = node(&mut builder, K::FormulaExpression, 46, 46, vec![thesis]);
            let proposition = node(&mut builder, K::Proposition, 46, 46, vec![terminal]);
            let conclusion = node(
                &mut builder,
                K::ConclusionStatement,
                45,
                47,
                vec![tokens[45], proposition, tokens[47]],
            );
            let proof = node(
                &mut builder,
                K::ProofBlock,
                27,
                48,
                vec![tokens[27], let1, let2, take, conclusion, tokens[48]],
            );
            let theorem = node(
                &mut builder,
                K::TheoremItem,
                0,
                49,
                vec![tokens[0], tokens[1], tokens[2], header, proof, tokens[49]],
            );
            let items = node(&mut builder, K::ItemList, 0, 49, vec![theorem]);
            let compilation = node(&mut builder, K::CompilationUnit, 0, 49, vec![items]);
            let mut root_children = tokens;
            root_children.push(compilation);
            let root = node(&mut builder, K::Root, 0, 49, root_children);
            let ast = builder.finish(Some(root), None);
            let native = SurfaceResolvedArena::lower(&ast, &module).unwrap();
            let shells = DeclarationShellCollector::new(&ast, &module).collect();
            let collection = SignatureProjectionExtractor::new(
                &ast,
                &shells,
                NamespacePath::new(module.path().as_str()),
            )
            .collect(&module);
            assert!(collection.diagnostics().is_empty());
            let symbols = collection.env().clone();
            let scope = SourceVariableScopeResolver::resolve_proof_bindings(
                SourceVariableScopeInput::new(&ast, &module, &symbols),
            )
            .unwrap();
            let mut original = TypedAst::try_new(TypedAstParts {
                source_id: source,
                module_id: module,
                resolved_root: Some(native.arena().root()),
                source_context: None,
                source_type: None,
                source_attribute: None,
                nodes: TypedArena::try_new(
                    Some(TypedNodeId::new(native.arena().root().index())),
                    native
                        .arena()
                        .iter()
                        .map(|(id, n)| {
                            TypedNode::new("syntax", n.origin().anchor().clone())
                                .with_resolved_node(id)
                                .with_children(
                                    n.children()
                                        .iter()
                                        .map(|c| TypedNodeId::new(c.index()))
                                        .collect(),
                                )
                        })
                        .collect(),
                )
                .unwrap(),
                contexts: Default::default(),
                types: Default::default(),
                facts: Default::default(),
                coercions: Default::default(),
                initial_obligations: Default::default(),
                diagnostics: Default::default(),
            })
            .unwrap();
            original.install_source_nodes(&native).unwrap();
            SourceProofLocalDeclarationProducer::build_bindings(
                &original, &native, &symbols, &scope,
            )
            .unwrap()
        };
        let genuine = fixture(false);
        let (steps, inputs, initial, checked, inference) = genuine.check_witness_goals().unwrap();
        assert_eq!(steps.len(), 6);
        assert_eq!(inputs.len(), 2);
        assert!(initial.is_empty());
        assert_eq!(inference.terms().len(), 12);
        assert_eq!(inference.formulas().len(), 2);
        assert!(
            checked
                .coercions()
                .iter()
                .all(|(_, c)| c.status == CoercionStatus::Candidate)
        );
        let values = steps[1].matched.unwrap().1;
        let other = steps[2].matched.unwrap().1;
        assert_ne!(values, other);
        assert_eq!(steps[5].free, vec![values, other]);
        assert!(
            matches!(&steps[5].after,SourceProofGoal::Conjunction{left,right,..}
            if matches!(left.as_ref(),SourceProofGoal::Equality{left,right,..} if *left==values && *right==values)
                && matches!(right.as_ref(),SourceProofGoal::Equality{left,right,..} if *left==other && *right==other))
        );
        assert_eq!(genuine.witness_goal_inputs().unwrap().0.len(), 6);
        for mutation in 0..16 {
            let mut bad = steps.clone();
            match mutation {
                0 => bad[3].matched.as_mut().unwrap().1 = other,
                1 => bad[3].after = steps[4].after.clone(),
                2 => bad[2].before = steps[0].before.clone(),
                3 => bad[3].free.clear(),
                4 => {
                    bad[1].avoided.pop();
                }
                5 => bad[5].owner = steps[1].site.clone(),
                6 => bad[3].context = BindingContextId::new(0),
                7 => {
                    bad.swap(3, 4);
                }
                8 => {
                    bad.remove(4);
                }
                9 => bad[3].matched.as_mut().unwrap().0 = values,
                10 => {
                    let SourceProofGoal::Existential { body, .. } = &mut bad[3].after else {
                        panic!("retained existential")
                    };
                    let SourceProofGoal::Conjunction { left, right, .. } = body.as_mut() else {
                        panic!("body")
                    };
                    let SourceProofGoal::Equality {
                        origin: other_origin,
                        ..
                    } = right.as_ref()
                    else {
                        panic!("right atom")
                    };
                    let SourceProofGoal::Equality { origin, .. } = left.as_mut() else {
                        panic!("left atom")
                    };
                    *origin = other_origin.clone();
                }
                11 => {
                    let SourceProofGoal::Existential { binding, .. } = &mut bad[3].after else {
                        panic!("retained binder")
                    };
                    *binding = values;
                }
                12 => bad[3].site = steps[1].site.clone(),
                13 => bad[3].free = vec![other],
                14 => {
                    bad[1].avoided[0] = values;
                    bad[1].avoided.sort();
                }
                _ => bad[1].after = bad[1].after.substitute(values, other),
            }
            if matches!(mutation, 11 | 15) {
                let mut free = BTreeSet::new();
                bad[if mutation == 11 { 3 } else { 1 }].after.binding_sets(
                    &mut Vec::new(),
                    &mut free,
                    &mut BTreeSet::new(),
                );
                bad[if mutation == 11 { 3 } else { 1 }].free = free.into_iter().collect();
            }
            assert!(
                genuine
                    .replay_witness_goals(&bad, &inputs, &initial, &checked, &inference)
                    .is_err(),
                "mutation{mutation}"
            );
        }
        let mut badinputs = inputs.clone();
        badinputs[0].from_type.as_mut().unwrap().head =
            crate::type_checker::TypeHeadInput::BuiltinObject;
        let forged = CoercionObligationChecker::default().check(
            &genuine.symbols,
            &TypeFactTable::new(),
            badinputs.clone(),
            initial.clone(),
        );
        assert!(
            genuine
                .replay_witness_goals(&steps, &badinputs, &initial, &forged, &inference)
                .is_err()
        );
        let (_, _, mut terms, formulas, _) = genuine.witness_goal_inputs().unwrap();
        let term = terms
            .iter_mut()
            .find(|t| {
                t.reference == Some(TermReference::Binding(values))
                    && t.context == steps[1].context
                    && matches!(t.site, crate::typed_ast::TypedSiteRef::Node(_))
            })
            .unwrap();
        term.reference = Some(TermReference::Binding(other));
        term.result_type = Some(
            genuine
                .witness_goal_guard(other, &term.site, "occurrence-result")
                .unwrap(),
        );
        let forged = TermFormulaChecker::default()
            .try_infer(&genuine.symbols, &genuine.binding_env, terms, formulas)
            .unwrap();
        assert!(forged.diagnostics().is_empty());
        assert!(
            forged
                .terms()
                .iter()
                .all(|(_, t)| t.status == crate::type_checker::TermStatus::Inferred)
        );
        assert!(
            genuine
                .replay_witness_goals(&steps, &inputs, &initial, &checked, &forged)
                .is_err()
        );
        for change_role in [false, true] {
            let (_, _, mut terms, formulas, _) = genuine.witness_goal_inputs().unwrap();
            let ty = terms[0].result_type.as_mut().unwrap();
            if change_role {
                ty.site = crate::typed_ast::TypedSiteRef::Role {
                    node: ty.site.node(),
                    role: crate::typed_ast::TypeRole::new("wrong-valid-type-role"),
                };
            } else {
                ty.head = crate::type_checker::TypeHeadInput::BuiltinObject;
            }
            let forged = TermFormulaChecker::default()
                .try_infer(&genuine.symbols, &genuine.binding_env, terms, formulas)
                .unwrap();
            assert!(forged.diagnostics().is_empty());
            assert!(
                forged
                    .terms()
                    .iter()
                    .all(|(_, t)| t.status == crate::type_checker::TermStatus::Inferred)
            );
            assert!(
                genuine
                    .replay_witness_goals(&steps, &inputs, &initial, &checked, &forged)
                    .is_err()
            );
        }
        let swapped = fixture(true);
        let (pending, pinputs, pinitial, pchecked, pinference) =
            swapped.check_witness_goals().unwrap();
        assert_eq!(pinputs.len(), 1);
        assert_eq!(pinitial.len(), 1);
        assert_eq!(
            pchecked
                .initial_obligations()
                .iter()
                .next()
                .unwrap()
                .1
                .status,
            InitialObligationStatus::Pending
        );
        assert_ne!(pending[5].after, steps[5].after);
        assert_eq!(genuine.original.source_id(), swapped.original.source_id());
        assert_eq!(genuine.original.module_id(), swapped.original.module_id());
        assert_eq!(
            steps.iter().map(|s| &s.site).collect::<Vec<_>>(),
            pending.iter().map(|s| &s.site).collect::<Vec<_>>()
        );
        for mutation in 0..3 {
            let mut bad = pinitial.clone();
            match mutation {
                0 => {
                    bad[0].goal = Some(crate::typed_ast::InitialObligationGoal::new(
                        "wrong-source-membership",
                    ))
                }
                1 => {
                    bad[0].provenance = Some(crate::typed_ast::InitialObligationProvenance::new(
                        "wrong-source-owner",
                    ))
                }
                _ => bad[0].site = pending[4].site.clone(),
            }
            let coherent = CoercionObligationChecker::default().check(
                &swapped.symbols,
                &TypeFactTable::new(),
                pinputs.clone(),
                bad.clone(),
            );
            assert!(coherent.diagnostics().is_empty());
            assert!(
                coherent
                    .initial_obligations()
                    .iter()
                    .all(|(_, o)| o.status == InitialObligationStatus::Pending)
            );
            assert!(
                swapped
                    .replay_witness_goals(&pending, &pinputs, &bad, &coherent, &pinference)
                    .is_err()
            );
        }
        let stripped = CoercionObligationChecker::default().check(
            &swapped.symbols,
            &TypeFactTable::new(),
            pinputs.clone(),
            [],
        );
        assert!(
            swapped
                .replay_witness_goals(&pending, &pinputs, &[], &stripped, &pinference)
                .is_err()
        );
        assert!(
            genuine
                .replay_witness_goals(&pending, &pinputs, &pinitial, &pchecked, &pinference)
                .is_err()
        );
        let installed = empty_typed(
            genuine.original.source_id(),
            genuine.original.module_id().clone(),
        )
        .with_source_proof_local_binding(genuine)
        .unwrap();
        let resolved = assemble_empty(&installed).unwrap();
        assert!(
            installed.facts().is_empty()
                && installed.types().is_empty()
                && resolved.checked_formulas().is_empty()
                && resolved.checked_proofs().is_empty()
        );
    }
    #[test]
    fn task272b2_conversion_checkpoints_replay_goal_status_and_coherent_products() {
        use crate::{
            type_checker::{
                CoercionEvidence, CoercionObligationChecker, CoercionRequestKind, TypeFactQuery,
                TypeFactQueryEngine, TypeFactQueryStatus,
            },
            typed_ast::{CoercionStatus, Polarity, TypePredicateRef},
        };
        use SurfaceNodeKind as K;
        use mizar_resolve::{
            declarations::DeclarationShellCollector,
            env::NamespacePath,
            names::{SourceVariableScopeInput, SourceVariableScopeResolver},
            symbols::SignatureProjectionExtractor,
        };
        let fixture = |from: &str, to: &str, reserve: bool| {
            let source = source_id();
            let module = ModuleId::new(PackageId::new("pkg"), ModulePath::new("checkpoint_replay"));
            let mut builder = SurfaceAstBuilder::new(source);
            let mut spans = Vec::new();
            let mut tokens = Vec::new();
            let mut cursor = 0;
            let prefix = if reserve {
                vec!["reserve", "r", "for", "object", ";"]
            } else {
                Vec::new()
            };
            let words = [
                "theorem",
                "T",
                ":",
                "for",
                "s",
                "being",
                from,
                "holds",
                "s",
                "=",
                "s",
                "proof",
                "let",
                "s",
                "be",
                from,
                ";",
                "reconsider",
                "k",
                "=",
                "s",
                "as",
                to,
                ";",
                "thus",
                "thesis",
                ";",
                "end",
                ";",
            ];
            for word in prefix.iter().copied().chain(words) {
                let span = range(source, cursor, cursor + word.len());
                cursor += word.len() + 1;
                let kind = if matches!(word, "T" | "s" | "k" | "r") {
                    SurfaceTokenKind::Identifier
                } else if matches!(word, ":" | "=" | ";") {
                    SurfaceTokenKind::ReservedSymbol
                } else {
                    SurfaceTokenKind::ReservedWord
                };
                tokens.push(builder.add_token(kind, word, span));
                spans.push(span);
            }
            let mut items = Vec::new();
            if reserve {
                let head = builder.add_node(K::TypeHead, spans[3], vec![tokens[3]]);
                let ty = builder.add_node(K::TypeExpression, spans[3], vec![head]);
                let segment = builder.add_node(
                    K::ReserveSegment,
                    range(source, spans[1].start, spans[3].end),
                    vec![tokens[1], tokens[2], ty],
                );
                items.push(builder.add_node(
                    K::ReserveItem,
                    range(source, spans[0].start, spans[4].end),
                    vec![tokens[0], segment, tokens[4]],
                ));
            }
            let offset = prefix.len();
            let t = &tokens[offset..];
            let ranges = &spans[offset..];
            let node = |b: &mut SurfaceAstBuilder, kind, start: usize, end: usize, children| {
                b.add_node(
                    kind,
                    range(source, ranges[start].start, ranges[end].end),
                    children,
                )
            };
            let mut types = BTreeMap::new();
            for i in [6, 15, 22] {
                let head = node(&mut builder, K::TypeHead, i, i, vec![t[i]]);
                types.insert(i, node(&mut builder, K::TypeExpression, i, i, vec![head]));
            }
            let segment = node(
                &mut builder,
                K::QuantifierVariableSegment,
                4,
                6,
                vec![t[4], t[5], types[&6]],
            );
            let mut values = BTreeMap::new();
            for i in [8, 10, 20] {
                let reference = node(&mut builder, K::TermReference, i, i, vec![t[i]]);
                values.insert(
                    i,
                    node(&mut builder, K::TermExpression, i, i, vec![reference]),
                );
            }
            let eq = node(
                &mut builder,
                K::BuiltinPredicateApplication,
                8,
                10,
                vec![values[&8], t[9], values[&10]],
            );
            let quant = node(
                &mut builder,
                K::QuantifiedFormula(SurfaceQuantifierKind::Universal),
                3,
                10,
                vec![t[3], segment, t[7], eq],
            );
            let formula = node(&mut builder, K::FormulaExpression, 3, 10, vec![quant]);
            let segment = node(
                &mut builder,
                K::QualifiedVariableSegment,
                13,
                15,
                vec![t[13], t[14], types[&15]],
            );
            let introduction = node(
                &mut builder,
                K::LetStatement,
                12,
                16,
                vec![t[12], segment, t[16]],
            );
            let item = node(
                &mut builder,
                K::ReconsiderItem,
                18,
                20,
                vec![t[18], t[19], values[&20]],
            );
            let reconsider = node(
                &mut builder,
                K::ReconsiderStatement,
                17,
                23,
                vec![t[17], item, t[21], types[&22], t[23]],
            );
            let thesis = node(
                &mut builder,
                K::FormulaConstant(SurfaceFormulaConstant::Thesis),
                25,
                25,
                vec![t[25]],
            );
            let terminal = node(&mut builder, K::FormulaExpression, 25, 25, vec![thesis]);
            let proposition = node(&mut builder, K::Proposition, 25, 25, vec![terminal]);
            let conclusion = node(
                &mut builder,
                K::ConclusionStatement,
                24,
                26,
                vec![t[24], proposition, t[26]],
            );
            let proof = node(
                &mut builder,
                K::ProofBlock,
                11,
                27,
                vec![t[11], introduction, reconsider, conclusion, t[27]],
            );
            items.push(node(
                &mut builder,
                K::TheoremItem,
                0,
                28,
                vec![t[0], t[1], t[2], formula, proof, t[28]],
            ));
            let span = range(source, spans[0].start, spans.last().unwrap().end);
            let items = builder.add_node(K::ItemList, span, items);
            let compilation = builder.add_node(K::CompilationUnit, span, vec![items]);
            let mut roots = tokens;
            roots.push(compilation);
            let root = builder.add_node(K::Root, span, roots);
            let ast = builder.finish(Some(root), None);
            let native = SurfaceResolvedArena::lower(&ast, &module).unwrap();
            let shells = DeclarationShellCollector::new(&ast, &module).collect();
            let collection = SignatureProjectionExtractor::new(
                &ast,
                &shells,
                NamespacePath::new(module.path().as_str()),
            )
            .collect(&module);
            assert!(collection.diagnostics().is_empty());
            let symbols = collection.env().clone();
            let scope = SourceVariableScopeResolver::resolve_proof_bindings(
                SourceVariableScopeInput::new(&ast, &module, &symbols),
            )
            .unwrap();
            let mut original = TypedAst::try_new(TypedAstParts {
                source_id: source,
                module_id: module,
                resolved_root: Some(native.arena().root()),
                source_context: None,
                source_type: None,
                source_attribute: None,
                nodes: TypedArena::try_new(
                    Some(TypedNodeId::new(native.arena().root().index())),
                    native
                        .arena()
                        .iter()
                        .map(|(id, n)| {
                            TypedNode::new("syntax", n.origin().anchor().clone())
                                .with_resolved_node(id)
                                .with_children(
                                    n.children()
                                        .iter()
                                        .map(|c| TypedNodeId::new(c.index()))
                                        .collect(),
                                )
                        })
                        .collect(),
                )
                .unwrap(),
                contexts: Default::default(),
                types: Default::default(),
                facts: Default::default(),
                coercions: Default::default(),
                initial_obligations: Default::default(),
                diagnostics: Default::default(),
            })
            .unwrap();
            original.install_source_nodes(&native).unwrap();
            SourceProofLocalDeclarationProducer::build_bindings(
                &original, &native, &symbols, &scope,
            )
            .unwrap()
        };
        let genuine = fixture("object", "set", false);
        let (steps, guards, initial, guard_output, inference) =
            genuine.check_witness_goals().unwrap();
        assert_eq!(steps.len(), 4);
        assert_eq!(inference.terms().len(), 5);
        assert_eq!(inference.formulas().len(), 1);
        assert!(guards.is_empty() && initial.is_empty() && guard_output.coercions().is_empty());
        assert_eq!(steps[2].before, steps[1].after);
        assert_eq!(steps[2].before, steps[2].after);
        assert_eq!(steps[3].after, steps[2].after);
        assert!(steps[2].matched.is_none() && steps[2].avoided.is_empty());
        let (requests, checked, queries) = genuine.check_conversions().unwrap();
        let (intents, _) = genuine.check_proof_intents().unwrap();
        assert_eq!(
            checked.coercions().iter().next().unwrap().1.status,
            CoercionStatus::Rejected
        );
        assert_eq!(queries[0].status(), TypeFactQueryStatus::Missing);
        assert!(
            intents
                .iter()
                .any(|(site, _, status)| site == steps[2].site()
                    && *status == SourceProofIntentStatus::Blocked)
        );
        assert!(
            intents
                .iter()
                .any(|(site, _, status)| site == steps[3].site()
                    && *status == SourceProofIntentStatus::Pending)
        );
        assert_eq!(
            genuine
                .witness_goal_conversion_checkpoints(&requests, &checked, &queries, &intents)
                .unwrap()
                .len(),
            1
        );
        for mutation in 0..11 {
            let mut bad = steps.clone();
            match mutation {
                0 => {
                    bad.remove(2);
                }
                1 => {
                    bad.insert(2, steps[2].clone());
                }
                2 => bad.swap(2, 3),
                3 => bad[2].site = steps[3].site.clone(),
                4 => bad[2].owner = steps[3].site.clone(),
                5 => bad[2].after = steps[0].before.clone(),
                6 => bad[2].free.clear(),
                7 => bad[2].context = BindingContextId::new(0),
                8 => bad[2].avoided.push(steps[1].matched.unwrap().1),
                9 => bad[2].matched = steps[1].matched,
                _ => bad[2].before = steps[0].before.clone(),
            }
            assert!(
                genuine
                    .replay_witness_goals(&bad, &guards, &initial, &guard_output, &inference)
                    .is_err(),
                "checkpoint{mutation}"
            );
        }
        for mutation in 0..3 {
            let mut bad = requests.clone();
            match mutation {
                0 => {
                    bad[0].kind = CoercionRequestKind::Widening;
                    bad[0].evidence = CoercionEvidence::BuiltinRadix;
                }
                1 => bad[0].site = steps[3].site.clone(),
                _ => bad[0].to_type.head = crate::type_checker::TypeHeadInput::BuiltinObject,
            }
            let coherent = CoercionObligationChecker::default().check(
                &genuine.symbols,
                &TypeFactTable::new(),
                bad.clone(),
                [],
            );
            let coherent_queries = bad
                .iter()
                .map(|r| {
                    TypeFactQueryEngine::new(coherent.facts()).query(TypeFactQuery::new(
                        r.site.clone(),
                        TypePredicateRef::new("coercion.widening.builtin_radix"),
                        Polarity::Positive,
                        r.source_range,
                    ))
                })
                .collect::<Vec<_>>();
            assert!(
                genuine
                    .witness_goal_conversion_checkpoints(
                        &bad,
                        &coherent,
                        &coherent_queries,
                        &intents
                    )
                    .is_err(),
                "ordinary{mutation}"
            );
        }
        let mut bad_status = intents.clone();
        bad_status
            .iter_mut()
            .find(|(site, _, _)| site == steps[2].site())
            .unwrap()
            .2 = SourceProofIntentStatus::Pending;
        assert!(
            genuine
                .witness_goal_conversion_checkpoints(&requests, &checked, &queries, &bad_status)
                .is_err()
        );
        let mut bad_queries = queries.clone();
        bad_queries[0] = TypeFactQueryEngine::new(checked.facts()).query(TypeFactQuery::new(
            steps[3].site.clone(),
            TypePredicateRef::new("coercion.widening.builtin_radix"),
            Polarity::Positive,
            requests[0].source_range,
        ));
        assert!(
            genuine
                .witness_goal_conversion_checkpoints(&requests, &checked, &bad_queries, &intents)
                .is_err()
        );
        let (_, _, mut terms, formulas, _) = genuine.witness_goal_inputs().unwrap();
        let actual = steps[1].matched.unwrap().1;
        let alias = genuine
            .binding_env
            .bindings()
            .iter()
            .find(|(_, b)| b.kind == BindingKind::LocalAbbreviation)
            .unwrap()
            .0;
        let rhs = terms
            .iter_mut()
            .find(|t| {
                matches!(t.site, crate::typed_ast::TypedSiteRef::Node(_))
                    && t.reference == Some(crate::type_checker::TermReference::Binding(actual))
            })
            .unwrap();
        rhs.reference = Some(crate::type_checker::TermReference::Binding(alias));
        let mut target = requests[0].to_type.clone();
        target.site = crate::typed_ast::TypedSiteRef::Role {
            node: rhs.site.node(),
            role: crate::typed_ast::TypeRole::new("forged-rejected-alias-type"),
        };
        rhs.result_type = Some(target);
        let coherent = crate::type_checker::TermFormulaChecker::default()
            .try_infer(&genuine.symbols, &genuine.binding_env, terms, formulas)
            .unwrap();
        assert!(coherent.diagnostics().is_empty());
        assert!(
            coherent
                .terms()
                .iter()
                .all(|(_, t)| t.status == crate::type_checker::TermStatus::Inferred)
        );
        assert!(
            genuine
                .replay_witness_goals(&steps, &guards, &initial, &guard_output, &coherent)
                .is_err()
        );
        let candidate = fixture("set", "object", false);
        assert_eq!(candidate.check_witness_goals().unwrap().0.len(), 4);
        assert_eq!(
            candidate
                .check_conversions()
                .unwrap()
                .1
                .coercions()
                .iter()
                .next()
                .unwrap()
                .1
                .status,
            CoercionStatus::Candidate
        );
        let unowned = fixture("object", "object", true);
        assert!(unowned.check_proof_intents().is_ok());
        assert!(unowned.check_witness_goals().is_err());
        let installed = empty_typed(
            genuine.original.source_id(),
            genuine.original.module_id().clone(),
        )
        .with_source_proof_local_binding(genuine)
        .unwrap();
        let resolved = assemble_empty(&installed).unwrap();
        assert!(
            installed.types().is_empty()
                && installed.facts().is_empty()
                && resolved.checked_proofs().is_empty()
                && resolved.checked_formulas().is_empty()
        );
    }
}
