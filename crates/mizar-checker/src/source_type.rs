//! Syntax-free source type-expression application handoff.

use crate::{
    binding_env::{
        BinderIdentity, BindingContextLayer, BindingContextOwner, BindingContextRecovery,
        BindingDraft, BindingEnv, BindingEnvParts, BindingId, BindingKind, BindingRecoveryState,
        BindingStatus, BindingTable, BindingTypeSite,
    },
    source_proof_local_declaration::{
        SourceProofLocalGivenBindingHandoff, SourceProofLocalGivenConditionBindingHandoff,
        SourceProofLocalGivenUseBindingHandoff, SourceProofLocalLetBindingHandoff,
    },
    typed_ast::{
        NodeRecoveryState, TypedArena, TypedNodeId, TypedNodeLinks, TypedSiteRef, TypingState,
    },
};
use mizar_resolve::{
    env::{
        ContributionKind, ExportStatus, SourceContributionId, SymbolEnv, SymbolKind, Visibility,
    },
    resolved_ast::{ModuleId, SemanticOrigin, SymbolId},
};
use mizar_session::{SourceAnchor, SourceId, SourceRange};
use std::{
    collections::BTreeSet,
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

dense_id!(SourceTypeApplicationId);
dense_id!(SourceTypeExpressionId);
dense_id!(SourceTypeArgumentId);
dense_id!(SourceTypeDefinitionReturnId);
dense_id!(SourceTypeModeRhsId);
dense_id!(SourceTypeStructureMemberId);

/// Syntax-free inputs for one complete source type-expression transaction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceTypeHandoffInput {
    pub source_id: SourceId,
    pub module_id: ModuleId,
    pub applications: Vec<SourceTypeApplicationInput>,
    pub expressions: Vec<SourceTypeExpressionInput>,
    pub arguments: Vec<SourceTypeArgumentInput>,
}

/// Syntax-free extension input for independently written definition return
/// types.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceTypeDefinitionReturnExtensionInput {
    pub source_id: SourceId,
    pub module_id: ModuleId,
    pub returns: Vec<SourceTypeDefinitionReturnInput>,
}

/// One definition owner linked to its independently written return type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceTypeDefinitionReturnInput {
    pub definition_site: TypedSiteRef,
    pub definition_range: SourceRange,
    pub source_ordinal: usize,
    pub expression: SourceTypeExpressionInput,
}

/// Syntax-free extension input for one independently written mode RHS.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceTypeModeRhsExtensionInput {
    pub source_id: SourceId,
    pub module_id: ModuleId,
    pub rhs: Vec<SourceTypeModeRhsInput>,
}

/// One mode-definition owner linked to its independently written RHS type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceTypeModeRhsInput {
    pub definition_site: TypedSiteRef,
    pub definition_range: SourceRange,
    pub source_ordinal: usize,
    pub expression: SourceTypeExpressionInput,
}

/// Syntax-free input for independently written structure-member types.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceTypeStructureMemberHandoffInput {
    pub source_id: SourceId,
    pub module_id: ModuleId,
    pub members: Vec<SourceTypeStructureMemberInput>,
}

/// One structure-member owner linked to its independently written type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceTypeStructureMemberInput {
    pub member_site: TypedSiteRef,
    pub member_range: SourceRange,
    pub source_ordinal: usize,
    pub expression: SourceTypeExpressionInput,
}

/// One top-level binding-to-source-type application input.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceTypeApplicationInput {
    pub binding: BindingId,
    pub source_ordinal: usize,
    pub root: SourceTypeExpressionId,
}

/// One flat source type-expression input.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceTypeExpressionInput {
    pub source_id: SourceId,
    pub module_id: ModuleId,
    pub site: TypedSiteRef,
    pub source_range: SourceRange,
    pub spelling: String,
    pub head_site: TypedSiteRef,
    pub head_range: SourceRange,
    pub head_spelling: String,
    pub form: SourceTypeApplicationForm,
    pub head: SourceTypeHead,
    pub recovery: NodeRecoveryState,
}

/// One ordered flat source type argument input.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceTypeArgumentInput {
    pub parent: SourceTypeExpressionId,
    pub ordinal: usize,
    pub argument: SourceTypeArgument,
}

/// Source-written type application form.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum SourceTypeApplicationForm {
    Bare,
    Of,
    Over,
    Bracket,
}

/// Source-written type head authenticated by built-in identity or `SymbolEnv`.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum SourceTypeHead {
    BuiltinSet,
    BuiltinObject,
    Symbol {
        symbol: SymbolId,
        contribution: SourceContributionId,
    },
}

/// Source-written argument payload. Term and qua sites intentionally carry no
/// checker `BindingId`.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum SourceTypeArgument {
    TermSite {
        site: TypedSiteRef,
        source_range: SourceRange,
        spelling: String,
        recovery: NodeRecoveryState,
        provenance: SemanticOrigin,
    },
    TypeSite {
        expression: SourceTypeExpressionId,
    },
    QuaSite {
        site: TypedSiteRef,
        source_range: SourceRange,
        spelling: String,
        recovery: NodeRecoveryState,
        provenance: SemanticOrigin,
        radix: Vec<SourceTypeExpressionId>,
    },
}

/// Immutable validated source type-expression handoff.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceTypeApplicationHandoff {
    source_id: SourceId,
    module_id: ModuleId,
    applications: SourceTypeApplicationTable,
    expressions: SourceTypeExpressionTable,
    arguments: SourceTypeArgumentTable,
    definition_returns: SourceTypeDefinitionReturnTable,
    mode_rhs: SourceTypeModeRhsTable,
    structure_members: SourceTypeStructureMemberTable,
}

impl SourceTypeApplicationHandoff {
    pub const fn source_id(&self) -> SourceId {
        self.source_id
    }

    pub const fn module_id(&self) -> &ModuleId {
        &self.module_id
    }

    pub const fn applications(&self) -> &SourceTypeApplicationTable {
        &self.applications
    }

    pub const fn expressions(&self) -> &SourceTypeExpressionTable {
        &self.expressions
    }

    pub const fn arguments(&self) -> &SourceTypeArgumentTable {
        &self.arguments
    }

    pub const fn definition_returns(&self) -> &SourceTypeDefinitionReturnTable {
        &self.definition_returns
    }

    pub const fn mode_rhs(&self) -> &SourceTypeModeRhsTable {
        &self.mode_rhs
    }

    pub const fn structure_members(&self) -> &SourceTypeStructureMemberTable {
        &self.structure_members
    }

    pub fn debug_text(&self) -> String {
        let mut output = String::from("source-type-application-debug-v1\n");
        output.push_str("module: ");
        output.push_str(self.module_id.path().as_str());
        output.push('\n');
        for (id, application) in self.applications.iter() {
            let _ = writeln!(
                output,
                "application#{} binding={} ordinal={} root={}",
                id.index(),
                application.binding.index(),
                application.source_ordinal,
                application.root.index(),
            );
        }
        for (id, definition_return) in self.definition_returns.iter() {
            let _ = write!(
                output,
                "definition-return#{} ordinal={} definition_range={}..{} definition_site=",
                id.index(),
                definition_return.source_ordinal,
                definition_return.definition_range.start,
                definition_return.definition_range.end,
            );
            write_definition_site(&mut output, &definition_return.definition_site);
            let _ = writeln!(output, " root={}", definition_return.root.index());
        }
        for (id, mode_rhs) in self.mode_rhs.iter() {
            let _ = write!(
                output,
                "mode-rhs#{} ordinal={} definition_range={}..{} definition_site=",
                id.index(),
                mode_rhs.source_ordinal,
                mode_rhs.definition_range.start,
                mode_rhs.definition_range.end,
            );
            write_definition_site(&mut output, &mode_rhs.definition_site);
            let _ = writeln!(output, " root={}", mode_rhs.root.index());
        }
        for (id, member) in self.structure_members.iter() {
            let _ = write!(
                output,
                "structure-member#{} ordinal={} member_range={}..{} member_site=",
                id.index(),
                member.source_ordinal,
                member.member_range.start,
                member.member_range.end,
            );
            write_definition_site(&mut output, &member.member_site);
            let _ = writeln!(output, " root={}", member.root.index());
        }
        for (id, expression) in self.expressions.iter() {
            let _ = write!(
                output,
                "expression#{} form={} range={}..{} site=",
                id.index(),
                form_key(expression.form),
                expression.source_range.start,
                expression.source_range.end,
            );
            write_site(&mut output, &expression.site);
            output.push_str(" head=");
            write_head(&mut output, &expression.head);
            output.push_str(" head_range=");
            let _ = write!(
                output,
                "{}..{} head_site=",
                expression.head_range.start, expression.head_range.end
            );
            write_site(&mut output, &expression.head_site);
            let _ = writeln!(
                output,
                " recovery={} spelling={:?} head_spelling={:?}",
                recovery_key(expression.recovery),
                expression.spelling,
                expression.head_spelling,
            );
        }
        for (id, argument) in self.arguments.iter() {
            let _ = write!(
                output,
                "argument#{} parent={} ordinal={} ",
                id.index(),
                argument.parent.index(),
                argument.ordinal,
            );
            write_argument(&mut output, &argument.argument);
            output.push('\n');
        }
        output
    }

    pub(crate) fn validate_installation(
        &self,
        source_id: SourceId,
        module_id: &ModuleId,
        arena: &TypedArena,
    ) -> Result<(), SourceTypeError> {
        if self.source_id != source_id || &self.module_id != module_id {
            return Err(SourceTypeError::EnvironmentMismatch);
        }
        validate_structure_member_handoff(self, arena)?;
        validate_definition_return_extension(self, arena)?;
        validate_mode_rhs_extension(self, arena)?;
        for (id, expression) in self.expressions.iter() {
            validate_arena_site(
                id,
                &expression.site,
                expression.source_range,
                expression.recovery,
                arena,
            )?;
            validate_arena_site(
                id,
                &expression.head_site,
                expression.head_range,
                expression.recovery,
                arena,
            )?;
        }
        for (id, argument) in self.arguments.iter() {
            match &argument.argument {
                SourceTypeArgument::TermSite {
                    site,
                    source_range,
                    recovery,
                    ..
                }
                | SourceTypeArgument::QuaSite {
                    site,
                    source_range,
                    recovery,
                    ..
                } => validate_argument_arena_site(id, site, *source_range, *recovery, arena)?,
                SourceTypeArgument::TypeSite { .. } => {}
            }
        }
        Ok(())
    }

    #[cfg(test)]
    pub(crate) fn set_structure_member_root_for_test(
        &mut self,
        index: usize,
        root: SourceTypeExpressionId,
    ) {
        self.structure_members.entries[index].root = root;
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SourceTypeApplicationTable {
    entries: Vec<SourceTypeApplication>,
}

impl SourceTypeApplicationTable {
    pub fn get(&self, id: SourceTypeApplicationId) -> Option<&SourceTypeApplication> {
        self.entries.get(id.index())
    }

    pub fn iter(&self) -> impl Iterator<Item = (SourceTypeApplicationId, &SourceTypeApplication)> {
        self.entries.iter().map(|entry| (entry.id, entry))
    }

    pub const fn len(&self) -> usize {
        self.entries.len()
    }

    pub const fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceTypeApplication {
    id: SourceTypeApplicationId,
    binding: BindingId,
    source_ordinal: usize,
    root: SourceTypeExpressionId,
}

impl SourceTypeApplication {
    pub const fn id(&self) -> SourceTypeApplicationId {
        self.id
    }

    pub const fn binding(&self) -> BindingId {
        self.binding
    }

    pub const fn source_ordinal(&self) -> usize {
        self.source_ordinal
    }

    pub const fn root(&self) -> SourceTypeExpressionId {
        self.root
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SourceTypeDefinitionReturnTable {
    entries: Vec<SourceTypeDefinitionReturn>,
}

impl SourceTypeDefinitionReturnTable {
    pub fn get(&self, id: SourceTypeDefinitionReturnId) -> Option<&SourceTypeDefinitionReturn> {
        self.entries.get(id.index())
    }

    pub fn iter(
        &self,
    ) -> impl Iterator<Item = (SourceTypeDefinitionReturnId, &SourceTypeDefinitionReturn)> {
        self.entries.iter().map(|entry| (entry.id, entry))
    }

    pub const fn len(&self) -> usize {
        self.entries.len()
    }

    pub const fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceTypeDefinitionReturn {
    id: SourceTypeDefinitionReturnId,
    definition_site: TypedSiteRef,
    definition_range: SourceRange,
    source_ordinal: usize,
    root: SourceTypeExpressionId,
}

impl SourceTypeDefinitionReturn {
    pub const fn id(&self) -> SourceTypeDefinitionReturnId {
        self.id
    }

    pub const fn definition_site(&self) -> &TypedSiteRef {
        &self.definition_site
    }

    pub const fn definition_range(&self) -> SourceRange {
        self.definition_range
    }

    pub const fn source_ordinal(&self) -> usize {
        self.source_ordinal
    }

    pub const fn root(&self) -> SourceTypeExpressionId {
        self.root
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SourceTypeModeRhsTable {
    entries: Vec<SourceTypeModeRhs>,
}

impl SourceTypeModeRhsTable {
    pub fn get(&self, id: SourceTypeModeRhsId) -> Option<&SourceTypeModeRhs> {
        self.entries.get(id.index())
    }

    pub fn iter(&self) -> impl Iterator<Item = (SourceTypeModeRhsId, &SourceTypeModeRhs)> {
        self.entries.iter().map(|entry| (entry.id, entry))
    }

    pub const fn len(&self) -> usize {
        self.entries.len()
    }

    pub const fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceTypeModeRhs {
    id: SourceTypeModeRhsId,
    definition_site: TypedSiteRef,
    definition_range: SourceRange,
    source_ordinal: usize,
    root: SourceTypeExpressionId,
}

impl SourceTypeModeRhs {
    pub const fn id(&self) -> SourceTypeModeRhsId {
        self.id
    }

    pub const fn definition_site(&self) -> &TypedSiteRef {
        &self.definition_site
    }

    pub const fn definition_range(&self) -> SourceRange {
        self.definition_range
    }

    pub const fn source_ordinal(&self) -> usize {
        self.source_ordinal
    }

    pub const fn root(&self) -> SourceTypeExpressionId {
        self.root
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SourceTypeStructureMemberTable {
    entries: Vec<SourceTypeStructureMember>,
}

impl SourceTypeStructureMemberTable {
    pub fn get(&self, id: SourceTypeStructureMemberId) -> Option<&SourceTypeStructureMember> {
        self.entries.get(id.index())
    }

    pub fn iter(
        &self,
    ) -> impl Iterator<Item = (SourceTypeStructureMemberId, &SourceTypeStructureMember)> {
        self.entries.iter().map(|entry| (entry.id, entry))
    }

    pub const fn len(&self) -> usize {
        self.entries.len()
    }

    pub const fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceTypeStructureMember {
    id: SourceTypeStructureMemberId,
    member_site: TypedSiteRef,
    member_range: SourceRange,
    source_ordinal: usize,
    root: SourceTypeExpressionId,
}

impl SourceTypeStructureMember {
    pub const fn id(&self) -> SourceTypeStructureMemberId {
        self.id
    }

    pub const fn member_site(&self) -> &TypedSiteRef {
        &self.member_site
    }

    pub const fn member_range(&self) -> SourceRange {
        self.member_range
    }

    pub const fn source_ordinal(&self) -> usize {
        self.source_ordinal
    }

    pub const fn root(&self) -> SourceTypeExpressionId {
        self.root
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SourceTypeExpressionTable {
    entries: Vec<SourceTypeExpression>,
}

impl SourceTypeExpressionTable {
    pub fn get(&self, id: SourceTypeExpressionId) -> Option<&SourceTypeExpression> {
        self.entries.get(id.index())
    }

    pub fn iter(&self) -> impl Iterator<Item = (SourceTypeExpressionId, &SourceTypeExpression)> {
        self.entries.iter().map(|entry| (entry.id, entry))
    }

    pub const fn len(&self) -> usize {
        self.entries.len()
    }

    pub const fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceTypeExpression {
    id: SourceTypeExpressionId,
    source_id: SourceId,
    module_id: ModuleId,
    site: TypedSiteRef,
    source_range: SourceRange,
    spelling: String,
    head_site: TypedSiteRef,
    head_range: SourceRange,
    head_spelling: String,
    form: SourceTypeApplicationForm,
    head: SourceTypeHead,
    recovery: NodeRecoveryState,
}

impl SourceTypeExpression {
    pub const fn id(&self) -> SourceTypeExpressionId {
        self.id
    }

    pub const fn source_id(&self) -> SourceId {
        self.source_id
    }

    pub const fn module_id(&self) -> &ModuleId {
        &self.module_id
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

    pub const fn head_site(&self) -> &TypedSiteRef {
        &self.head_site
    }

    pub const fn head_range(&self) -> SourceRange {
        self.head_range
    }

    pub fn head_spelling(&self) -> &str {
        &self.head_spelling
    }

    pub const fn form(&self) -> SourceTypeApplicationForm {
        self.form
    }

    pub const fn head(&self) -> &SourceTypeHead {
        &self.head
    }

    pub const fn recovery(&self) -> NodeRecoveryState {
        self.recovery
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SourceTypeArgumentTable {
    entries: Vec<SourceTypeArgumentRow>,
}

impl SourceTypeArgumentTable {
    pub fn get(&self, id: SourceTypeArgumentId) -> Option<&SourceTypeArgumentRow> {
        self.entries.get(id.index())
    }

    pub fn iter(&self) -> impl Iterator<Item = (SourceTypeArgumentId, &SourceTypeArgumentRow)> {
        self.entries.iter().map(|entry| (entry.id, entry))
    }

    pub const fn len(&self) -> usize {
        self.entries.len()
    }

    pub const fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceTypeArgumentRow {
    id: SourceTypeArgumentId,
    parent: SourceTypeExpressionId,
    ordinal: usize,
    argument: SourceTypeArgument,
}

impl SourceTypeArgumentRow {
    pub const fn id(&self) -> SourceTypeArgumentId {
        self.id
    }

    pub const fn parent(&self) -> SourceTypeExpressionId {
        self.parent
    }

    pub const fn ordinal(&self) -> usize {
        self.ordinal
    }

    pub const fn argument(&self) -> &SourceTypeArgument {
        &self.argument
    }
}

/// Validates and transactionally constructs source type-expression handoffs.
pub struct SourceTypeProducer;

impl SourceTypeProducer {
    pub fn build(
        input: SourceTypeHandoffInput,
        bindings: &BindingEnv,
        symbols: &SymbolEnv,
        arena: &TypedArena,
    ) -> Result<SourceTypeApplicationHandoff, SourceTypeError> {
        validate_input(
            &input,
            bindings,
            symbols,
            arena,
            SourceTypeBindingProfile::Generic,
        )?;
        Ok(build_source_type_handoff(input))
    }
}

fn build_source_type_handoff(input: SourceTypeHandoffInput) -> SourceTypeApplicationHandoff {
    SourceTypeApplicationHandoff {
        source_id: input.source_id,
        module_id: input.module_id,
        applications: SourceTypeApplicationTable {
            entries: input
                .applications
                .into_iter()
                .enumerate()
                .map(|(index, input)| SourceTypeApplication {
                    id: SourceTypeApplicationId::new(index),
                    binding: input.binding,
                    source_ordinal: input.source_ordinal,
                    root: input.root,
                })
                .collect(),
        },
        expressions: SourceTypeExpressionTable {
            entries: input
                .expressions
                .into_iter()
                .enumerate()
                .map(|(index, input)| SourceTypeExpression {
                    id: SourceTypeExpressionId::new(index),
                    source_id: input.source_id,
                    module_id: input.module_id,
                    site: input.site,
                    source_range: input.source_range,
                    spelling: input.spelling,
                    head_site: input.head_site,
                    head_range: input.head_range,
                    head_spelling: input.head_spelling,
                    form: input.form,
                    head: input.head,
                    recovery: input.recovery,
                })
                .collect(),
        },
        arguments: SourceTypeArgumentTable {
            entries: input
                .arguments
                .into_iter()
                .enumerate()
                .map(|(index, input)| SourceTypeArgumentRow {
                    id: SourceTypeArgumentId::new(index),
                    parent: input.parent,
                    ordinal: input.ordinal,
                    argument: input.argument,
                })
                .collect(),
        },
        definition_returns: SourceTypeDefinitionReturnTable::default(),
        mode_rhs: SourceTypeModeRhsTable::default(),
        structure_members: SourceTypeStructureMemberTable::default(),
    }
}

/// Immutable exact Task-269CT proof-local `let` type-composition handoff.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceProofLocalLetTypeHandoff {
    source_id: SourceId,
    module_id: ModuleId,
    dependency: SourceProofLocalLetBindingHandoff,
    dependency_fingerprint: String,
    binding_env: BindingEnv,
    binding_fingerprint: String,
    source_type: SourceTypeApplicationHandoff,
    source_type_fingerprint: String,
}

impl SourceProofLocalLetTypeHandoff {
    pub const fn source_id(&self) -> SourceId {
        self.source_id
    }

    pub const fn module_id(&self) -> &ModuleId {
        &self.module_id
    }

    pub const fn dependency(&self) -> &SourceProofLocalLetBindingHandoff {
        &self.dependency
    }

    pub fn dependency_fingerprint(&self) -> &str {
        &self.dependency_fingerprint
    }

    pub const fn binding_env(&self) -> &BindingEnv {
        &self.binding_env
    }

    pub fn binding_fingerprint(&self) -> &str {
        &self.binding_fingerprint
    }

    pub const fn source_type(&self) -> &SourceTypeApplicationHandoff {
        &self.source_type
    }

    pub fn source_type_fingerprint(&self) -> &str {
        &self.source_type_fingerprint
    }

    pub fn debug_text(&self) -> String {
        format!(
            concat!(
                "source-proof-local-let-type-debug-v1\n",
                "module: {}::{}\n",
                "dependency-fingerprint: {:?}\n",
                "binding-fingerprint: {:?}\n",
                "source-type-fingerprint: {:?}\n",
            ),
            self.module_id.package().as_str(),
            self.module_id.path().as_str(),
            self.dependency_fingerprint,
            self.binding_fingerprint,
            self.source_type_fingerprint,
        )
    }

    pub(crate) fn validate_installation(
        &self,
        source_id: SourceId,
        module_id: &ModuleId,
        arena: &TypedArena,
    ) -> Result<(), SourceProofLocalLetTypeError> {
        self.dependency
            .validate_installation(source_id, module_id)
            .map_err(|_| SourceProofLocalLetTypeError::InvalidDependency)?;
        if self.source_id != source_id
            || &self.module_id != module_id
            || self.dependency_fingerprint != self.dependency.debug_text()
        {
            return Err(SourceProofLocalLetTypeError::InvalidDependency);
        }
        let expected_binding = task269ct_binding_env(&self.dependency)?;
        if self.binding_env != expected_binding
            || self.binding_fingerprint != self.binding_env.debug_text()
        {
            return Err(SourceProofLocalLetTypeError::InvalidBindingEnvironment);
        }
        if !exact_task269ct_source_type(&self.source_type, source_id, module_id)
            || self.source_type_fingerprint != self.source_type.debug_text()
            || !exact_task269ct_arena(source_id, arena)
            || self
                .source_type
                .validate_installation(source_id, module_id, arena)
                .is_err()
        {
            return Err(SourceProofLocalLetTypeError::InvalidSourceType);
        }
        Ok(())
    }

    pub(crate) fn validate_complete_installation(
        &self,
        source_id: SourceId,
        module_id: &ModuleId,
        arena: &TypedArena,
        installation_available: bool,
    ) -> Result<(), SourceProofLocalLetTypeError> {
        self.validate_installation(source_id, module_id, arena)?;
        if !installation_available {
            return Err(SourceProofLocalLetTypeError::InvalidInstallation);
        }
        Ok(())
    }
}

/// Builds only the exact Task-269CT proof-local `let` type composition.
pub struct SourceProofLocalLetTypeProducer;

impl SourceProofLocalLetTypeProducer {
    pub fn build(
        dependency: SourceProofLocalLetBindingHandoff,
        input: SourceTypeHandoffInput,
        symbols: &SymbolEnv,
        arena: &TypedArena,
    ) -> Result<SourceProofLocalLetTypeHandoff, SourceProofLocalLetTypeError> {
        dependency
            .validate_installation(input.source_id, &input.module_id)
            .map_err(|_| SourceProofLocalLetTypeError::InvalidDependency)?;
        let dependency_fingerprint = dependency.debug_text();
        let binding_env = task269ct_binding_env(&dependency)?;
        if !exact_task269ct_input(&input) || !exact_task269ct_arena(input.source_id, arena) {
            return Err(SourceProofLocalLetTypeError::InvalidSourceType);
        }
        validate_input(
            &input,
            &binding_env,
            symbols,
            arena,
            SourceTypeBindingProfile::ProofLocalLet,
        )
        .map_err(|_| SourceProofLocalLetTypeError::InvalidSourceType)?;
        let binding_fingerprint = binding_env.debug_text();
        let source_type = build_source_type_handoff(input);
        let source_type_fingerprint = source_type.debug_text();
        let handoff = SourceProofLocalLetTypeHandoff {
            source_id: dependency.source_id(),
            module_id: dependency.module_id().clone(),
            dependency,
            dependency_fingerprint,
            binding_env,
            binding_fingerprint,
            source_type,
            source_type_fingerprint,
        };
        handoff.validate_installation(handoff.source_id, &handoff.module_id, arena)?;
        Ok(handoff)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum SourceProofLocalLetTypeError {
    InvalidDependency,
    InvalidBindingEnvironment,
    InvalidSourceType,
    InvalidInstallation,
}

impl fmt::Display for SourceProofLocalLetTypeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidDependency => {
                formatter.write_str("source proof-local let type dependency is invalid")
            }
            Self::InvalidBindingEnvironment => {
                formatter.write_str("source proof-local let typed binding environment is invalid")
            }
            Self::InvalidSourceType => {
                formatter.write_str("source proof-local let source type is invalid")
            }
            Self::InvalidInstallation => {
                formatter.write_str("source proof-local let type installation is invalid")
            }
        }
    }
}

impl Error for SourceProofLocalLetTypeError {}

/// Immutable exact Task-269GT proof-local `given` type-composition handoff.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceProofLocalGivenTypeHandoff {
    source_id: SourceId,
    module_id: ModuleId,
    dependency: SourceProofLocalGivenBindingHandoff,
    dependency_fingerprint: String,
    binding_env: BindingEnv,
    binding_fingerprint: String,
    source_type: SourceTypeApplicationHandoff,
    source_type_fingerprint: String,
}

impl SourceProofLocalGivenTypeHandoff {
    pub const fn source_id(&self) -> SourceId {
        self.source_id
    }

    pub const fn module_id(&self) -> &ModuleId {
        &self.module_id
    }

    pub const fn dependency(&self) -> &SourceProofLocalGivenBindingHandoff {
        &self.dependency
    }

    pub fn dependency_fingerprint(&self) -> &str {
        &self.dependency_fingerprint
    }

    pub const fn binding_env(&self) -> &BindingEnv {
        &self.binding_env
    }

    pub fn binding_fingerprint(&self) -> &str {
        &self.binding_fingerprint
    }

    pub const fn source_type(&self) -> &SourceTypeApplicationHandoff {
        &self.source_type
    }

    pub fn source_type_fingerprint(&self) -> &str {
        &self.source_type_fingerprint
    }

    pub fn debug_text(&self) -> String {
        format!(
            concat!(
                "source-proof-local-given-type-debug-v1\n",
                "module: {}::{}\n",
                "dependency-fingerprint: {:?}\n",
                "binding-fingerprint: {:?}\n",
                "source-type-fingerprint: {:?}\n",
            ),
            self.module_id.package().as_str(),
            self.module_id.path().as_str(),
            self.dependency_fingerprint,
            self.binding_fingerprint,
            self.source_type_fingerprint,
        )
    }

    pub(crate) fn validate_installation(
        &self,
        source_id: SourceId,
        module_id: &ModuleId,
        arena: &TypedArena,
    ) -> Result<(), SourceProofLocalGivenTypeError> {
        self.dependency
            .validate_installation(source_id, module_id)
            .map_err(|_| SourceProofLocalGivenTypeError::InvalidDependency)?;
        if self.source_id != source_id
            || &self.module_id != module_id
            || self.dependency_fingerprint != self.dependency.debug_text()
        {
            return Err(SourceProofLocalGivenTypeError::InvalidDependency);
        }
        let expected_binding = task269gt_binding_env(&self.dependency)?;
        if self.binding_env != expected_binding
            || self.binding_fingerprint != self.binding_env.debug_text()
        {
            return Err(SourceProofLocalGivenTypeError::InvalidBindingEnvironment);
        }
        if !exact_task269gt_source_type(&self.source_type, source_id, module_id)
            || self.source_type_fingerprint != self.source_type.debug_text()
            || !exact_task269gt_arena(source_id, arena)
            || self
                .source_type
                .validate_installation(source_id, module_id, arena)
                .is_err()
        {
            return Err(SourceProofLocalGivenTypeError::InvalidSourceType);
        }
        Ok(())
    }

    pub(crate) fn validate_complete_installation(
        &self,
        source_id: SourceId,
        module_id: &ModuleId,
        arena: &TypedArena,
        installation_available: bool,
    ) -> Result<(), SourceProofLocalGivenTypeError> {
        self.validate_installation(source_id, module_id, arena)?;
        if !installation_available {
            return Err(SourceProofLocalGivenTypeError::InvalidInstallation);
        }
        Ok(())
    }
}

/// Builds only the exact Task-269GT proof-local `given` type composition.
pub struct SourceProofLocalGivenTypeProducer;

impl SourceProofLocalGivenTypeProducer {
    pub fn build(
        dependency: SourceProofLocalGivenBindingHandoff,
        input: SourceTypeHandoffInput,
        symbols: &SymbolEnv,
        arena: &TypedArena,
    ) -> Result<SourceProofLocalGivenTypeHandoff, SourceProofLocalGivenTypeError> {
        dependency
            .validate_installation(input.source_id, &input.module_id)
            .map_err(|_| SourceProofLocalGivenTypeError::InvalidDependency)?;
        let dependency_fingerprint = dependency.debug_text();
        let binding_env = task269gt_binding_env(&dependency)?;
        if !exact_task269gt_input(&input) || !exact_task269gt_arena(input.source_id, arena) {
            return Err(SourceProofLocalGivenTypeError::InvalidSourceType);
        }
        validate_input(
            &input,
            &binding_env,
            symbols,
            arena,
            SourceTypeBindingProfile::ProofLocalGiven,
        )
        .map_err(|_| SourceProofLocalGivenTypeError::InvalidSourceType)?;
        let binding_fingerprint = binding_env.debug_text();
        let source_type = build_source_type_handoff(input);
        let source_type_fingerprint = source_type.debug_text();
        let handoff = SourceProofLocalGivenTypeHandoff {
            source_id: dependency.source_id(),
            module_id: dependency.module_id().clone(),
            dependency,
            dependency_fingerprint,
            binding_env,
            binding_fingerprint,
            source_type,
            source_type_fingerprint,
        };
        handoff.validate_installation(handoff.source_id, &handoff.module_id, arena)?;
        Ok(handoff)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum SourceProofLocalGivenTypeError {
    InvalidDependency,
    InvalidBindingEnvironment,
    InvalidSourceType,
    InvalidInstallation,
}

impl fmt::Display for SourceProofLocalGivenTypeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidDependency => {
                formatter.write_str("source proof-local given type dependency is invalid")
            }
            Self::InvalidBindingEnvironment => {
                formatter.write_str("source proof-local given typed binding environment is invalid")
            }
            Self::InvalidSourceType => {
                formatter.write_str("source proof-local given source type is invalid")
            }
            Self::InvalidInstallation => {
                formatter.write_str("source proof-local given type installation is invalid")
            }
        }
    }
}

impl std::error::Error for SourceProofLocalGivenTypeError {}

/// Immutable exact Task-269GCT proof-local `given` condition type handoff.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceProofLocalGivenConditionTypeHandoff {
    source_id: SourceId,
    module_id: ModuleId,
    dependency: SourceProofLocalGivenConditionBindingHandoff,
    dependency_fingerprint: String,
    binding_env: BindingEnv,
    binding_fingerprint: String,
    source_type: SourceTypeApplicationHandoff,
    source_type_fingerprint: String,
}

impl SourceProofLocalGivenConditionTypeHandoff {
    pub const fn source_id(&self) -> SourceId {
        self.source_id
    }

    pub const fn module_id(&self) -> &ModuleId {
        &self.module_id
    }

    pub const fn dependency(&self) -> &SourceProofLocalGivenConditionBindingHandoff {
        &self.dependency
    }

    pub fn dependency_fingerprint(&self) -> &str {
        &self.dependency_fingerprint
    }

    pub const fn binding_env(&self) -> &BindingEnv {
        &self.binding_env
    }

    pub fn binding_fingerprint(&self) -> &str {
        &self.binding_fingerprint
    }

    pub const fn source_type(&self) -> &SourceTypeApplicationHandoff {
        &self.source_type
    }

    pub fn source_type_fingerprint(&self) -> &str {
        &self.source_type_fingerprint
    }

    pub fn debug_text(&self) -> String {
        format!(
            concat!(
                "source-proof-local-given-condition-type-debug-v1\n",
                "module: {}::{}\n",
                "dependency-fingerprint: {:?}\n",
                "binding-fingerprint: {:?}\n",
                "source-type-fingerprint: {:?}\n",
            ),
            self.module_id.package().as_str(),
            self.module_id.path().as_str(),
            self.dependency_fingerprint,
            self.binding_fingerprint,
            self.source_type_fingerprint,
        )
    }

    pub(crate) fn validate_installation(
        &self,
        source_id: SourceId,
        module_id: &ModuleId,
        arena: &TypedArena,
    ) -> Result<(), SourceProofLocalGivenConditionTypeError> {
        self.dependency
            .validate_installation(source_id, module_id)
            .map_err(|_| SourceProofLocalGivenConditionTypeError::InvalidDependency)?;
        if self.source_id != source_id
            || &self.module_id != module_id
            || self.dependency_fingerprint != self.dependency.debug_text()
        {
            return Err(SourceProofLocalGivenConditionTypeError::InvalidDependency);
        }
        let expected_binding = task269gct_binding_env(&self.dependency)?;
        if self.binding_env != expected_binding
            || self.binding_fingerprint != self.binding_env.debug_text()
        {
            return Err(SourceProofLocalGivenConditionTypeError::InvalidBindingEnvironment);
        }
        if !exact_task269gct_source_type(&self.source_type, source_id, module_id)
            || self.source_type_fingerprint != self.source_type.debug_text()
            || !exact_task269gct_arena(source_id, arena)
            || self
                .source_type
                .validate_installation(source_id, module_id, arena)
                .is_err()
        {
            return Err(SourceProofLocalGivenConditionTypeError::InvalidSourceType);
        }
        Ok(())
    }

    pub(crate) fn validate_complete_installation(
        &self,
        source_id: SourceId,
        module_id: &ModuleId,
        arena: &TypedArena,
        installation_available: bool,
    ) -> Result<(), SourceProofLocalGivenConditionTypeError> {
        self.validate_installation(source_id, module_id, arena)?;
        if !installation_available {
            return Err(SourceProofLocalGivenConditionTypeError::InvalidInstallation);
        }
        Ok(())
    }
}

/// Builds only the exact Task-269GCT proof-local `given` condition type.
pub struct SourceProofLocalGivenConditionTypeProducer;

impl SourceProofLocalGivenConditionTypeProducer {
    pub fn build(
        dependency: SourceProofLocalGivenConditionBindingHandoff,
        input: SourceTypeHandoffInput,
        symbols: &SymbolEnv,
        arena: &TypedArena,
    ) -> Result<SourceProofLocalGivenConditionTypeHandoff, SourceProofLocalGivenConditionTypeError>
    {
        dependency
            .validate_installation(input.source_id, &input.module_id)
            .map_err(|_| SourceProofLocalGivenConditionTypeError::InvalidDependency)?;
        let dependency_fingerprint = dependency.debug_text();
        let binding_env = task269gct_binding_env(&dependency)?;
        if !exact_task269gct_input(&input) || !exact_task269gct_arena(input.source_id, arena) {
            return Err(SourceProofLocalGivenConditionTypeError::InvalidSourceType);
        }
        validate_input(
            &input,
            &binding_env,
            symbols,
            arena,
            SourceTypeBindingProfile::ProofLocalGiven,
        )
        .map_err(|_| SourceProofLocalGivenConditionTypeError::InvalidSourceType)?;
        let binding_fingerprint = binding_env.debug_text();
        let source_type = build_source_type_handoff(input);
        let source_type_fingerprint = source_type.debug_text();
        let handoff = SourceProofLocalGivenConditionTypeHandoff {
            source_id: dependency.source_id(),
            module_id: dependency.module_id().clone(),
            dependency,
            dependency_fingerprint,
            binding_env,
            binding_fingerprint,
            source_type,
            source_type_fingerprint,
        };
        handoff.validate_installation(handoff.source_id, &handoff.module_id, arena)?;
        Ok(handoff)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum SourceProofLocalGivenConditionTypeError {
    InvalidDependency,
    InvalidBindingEnvironment,
    InvalidSourceType,
    InvalidInstallation,
}

impl fmt::Display for SourceProofLocalGivenConditionTypeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidDependency => {
                formatter.write_str("source proof-local given-condition type dependency is invalid")
            }
            Self::InvalidBindingEnvironment => formatter.write_str(
                "source proof-local given-condition typed binding environment is invalid",
            ),
            Self::InvalidSourceType => {
                formatter.write_str("source proof-local given-condition source type is invalid")
            }
            Self::InvalidInstallation => formatter
                .write_str("source proof-local given-condition type installation is invalid"),
        }
    }
}

impl std::error::Error for SourceProofLocalGivenConditionTypeError {}

/// Immutable exact Task-269GUPT proof-local `given` use-profile type handoff.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceProofLocalGivenUseTypeHandoff {
    source_id: SourceId,
    module_id: ModuleId,
    dependency: SourceProofLocalGivenUseBindingHandoff,
    dependency_fingerprint: String,
    binding_env: BindingEnv,
    binding_fingerprint: String,
    source_type: SourceTypeApplicationHandoff,
    source_type_fingerprint: String,
}

impl SourceProofLocalGivenUseTypeHandoff {
    pub const fn source_id(&self) -> SourceId {
        self.source_id
    }

    pub const fn module_id(&self) -> &ModuleId {
        &self.module_id
    }

    pub const fn dependency(&self) -> &SourceProofLocalGivenUseBindingHandoff {
        &self.dependency
    }

    pub fn dependency_fingerprint(&self) -> &str {
        &self.dependency_fingerprint
    }

    pub const fn binding_env(&self) -> &BindingEnv {
        &self.binding_env
    }

    pub fn binding_fingerprint(&self) -> &str {
        &self.binding_fingerprint
    }

    pub const fn source_type(&self) -> &SourceTypeApplicationHandoff {
        &self.source_type
    }

    pub fn source_type_fingerprint(&self) -> &str {
        &self.source_type_fingerprint
    }

    pub fn debug_text(&self) -> String {
        format!(
            concat!(
                "source-proof-local-given-use-type-debug-v1\n",
                "module: {}::{}\n",
                "dependency-fingerprint: {:?}\n",
                "binding-fingerprint: {:?}\n",
                "source-type-fingerprint: {:?}\n",
            ),
            self.module_id.package().as_str(),
            self.module_id.path().as_str(),
            self.dependency_fingerprint,
            self.binding_fingerprint,
            self.source_type_fingerprint,
        )
    }

    pub(crate) fn validate_installation(
        &self,
        source_id: SourceId,
        module_id: &ModuleId,
        arena: &TypedArena,
    ) -> Result<(), SourceProofLocalGivenUseTypeError> {
        self.dependency
            .validate_installation(source_id, module_id)
            .map_err(|_| SourceProofLocalGivenUseTypeError::InvalidDependency)?;
        if self.source_id != source_id
            || &self.module_id != module_id
            || self.dependency_fingerprint != self.dependency.debug_text()
        {
            return Err(SourceProofLocalGivenUseTypeError::InvalidDependency);
        }
        let expected_binding = task269gupt_binding_env(&self.dependency)?;
        if self.binding_env != expected_binding
            || self.binding_fingerprint != self.binding_env.debug_text()
        {
            return Err(SourceProofLocalGivenUseTypeError::InvalidBindingEnvironment);
        }
        if !exact_task269gupt_source_type(&self.source_type, source_id, module_id)
            || self.source_type_fingerprint != self.source_type.debug_text()
            || !exact_task269gupt_arena(source_id, arena)
            || self
                .source_type
                .validate_installation(source_id, module_id, arena)
                .is_err()
        {
            return Err(SourceProofLocalGivenUseTypeError::InvalidSourceType);
        }
        Ok(())
    }

    pub(crate) fn validate_complete_installation(
        &self,
        source_id: SourceId,
        module_id: &ModuleId,
        arena: &TypedArena,
        installation_available: bool,
    ) -> Result<(), SourceProofLocalGivenUseTypeError> {
        self.validate_installation(source_id, module_id, arena)?;
        if !installation_available {
            return Err(SourceProofLocalGivenUseTypeError::InvalidInstallation);
        }
        Ok(())
    }
}

/// Builds only the exact Task-269GUPT proof-local `given` use-profile type.
pub struct SourceProofLocalGivenUseTypeProducer;

impl SourceProofLocalGivenUseTypeProducer {
    pub fn build(
        dependency: SourceProofLocalGivenUseBindingHandoff,
        input: SourceTypeHandoffInput,
        symbols: &SymbolEnv,
        arena: &TypedArena,
    ) -> Result<SourceProofLocalGivenUseTypeHandoff, SourceProofLocalGivenUseTypeError> {
        dependency
            .validate_installation(input.source_id, &input.module_id)
            .map_err(|_| SourceProofLocalGivenUseTypeError::InvalidDependency)?;
        let dependency_fingerprint = dependency.debug_text();
        let binding_env = task269gupt_binding_env(&dependency)?;
        if !exact_task269gupt_input(&input) || !exact_task269gupt_arena(input.source_id, arena) {
            return Err(SourceProofLocalGivenUseTypeError::InvalidSourceType);
        }
        validate_input(
            &input,
            &binding_env,
            symbols,
            arena,
            SourceTypeBindingProfile::ProofLocalGiven,
        )
        .map_err(|_| SourceProofLocalGivenUseTypeError::InvalidSourceType)?;
        let binding_fingerprint = binding_env.debug_text();
        let source_type = build_source_type_handoff(input);
        let source_type_fingerprint = source_type.debug_text();
        let handoff = SourceProofLocalGivenUseTypeHandoff {
            source_id: dependency.source_id(),
            module_id: dependency.module_id().clone(),
            dependency,
            dependency_fingerprint,
            binding_env,
            binding_fingerprint,
            source_type,
            source_type_fingerprint,
        };
        handoff.validate_installation(handoff.source_id, &handoff.module_id, arena)?;
        Ok(handoff)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum SourceProofLocalGivenUseTypeError {
    InvalidDependency,
    InvalidBindingEnvironment,
    InvalidSourceType,
    InvalidInstallation,
}

impl fmt::Display for SourceProofLocalGivenUseTypeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidDependency => {
                formatter.write_str("source proof-local given-use type dependency is invalid")
            }
            Self::InvalidBindingEnvironment => formatter
                .write_str("source proof-local given-use typed binding environment is invalid"),
            Self::InvalidSourceType => {
                formatter.write_str("source proof-local given-use source type is invalid")
            }
            Self::InvalidInstallation => {
                formatter.write_str("source proof-local given-use type installation is invalid")
            }
        }
    }
}

impl std::error::Error for SourceProofLocalGivenUseTypeError {}

/// Builds the exact standalone Task-263 structure-member type handoff without
/// fabricating binding-linked applications.
pub struct SourceTypeStructureMemberProducer;

impl SourceTypeStructureMemberProducer {
    pub fn build(
        input: SourceTypeStructureMemberHandoffInput,
        arena: &TypedArena,
    ) -> Result<SourceTypeApplicationHandoff, SourceTypeError> {
        if input.members.is_empty() {
            return Err(SourceTypeError::EmptyStructureMembers);
        }
        if input.members.len() != 4 {
            return Err(SourceTypeError::StructureMemberCardinalityMismatch);
        }

        let mut expressions = Vec::with_capacity(input.members.len());
        let mut members = Vec::with_capacity(input.members.len());
        for (index, member) in input.members.into_iter().enumerate() {
            let root = SourceTypeExpressionId::new(index);
            members.push(SourceTypeStructureMember {
                id: SourceTypeStructureMemberId::new(index),
                member_site: member.member_site,
                member_range: member.member_range,
                source_ordinal: member.source_ordinal,
                root,
            });
            expressions.push(SourceTypeExpression {
                id: root,
                source_id: member.expression.source_id,
                module_id: member.expression.module_id,
                site: member.expression.site,
                source_range: member.expression.source_range,
                spelling: member.expression.spelling,
                head_site: member.expression.head_site,
                head_range: member.expression.head_range,
                head_spelling: member.expression.head_spelling,
                form: member.expression.form,
                head: member.expression.head,
                recovery: member.expression.recovery,
            });
        }

        let handoff = SourceTypeApplicationHandoff {
            source_id: input.source_id,
            module_id: input.module_id,
            applications: SourceTypeApplicationTable::default(),
            expressions: SourceTypeExpressionTable {
                entries: expressions,
            },
            arguments: SourceTypeArgumentTable::default(),
            definition_returns: SourceTypeDefinitionReturnTable::default(),
            mode_rhs: SourceTypeModeRhsTable::default(),
            structure_members: SourceTypeStructureMemberTable { entries: members },
        };
        validate_structure_member_handoff(&handoff, arena)?;
        Ok(handoff)
    }

    /// Extends the exact Task-249PI property-implementation base with its two
    /// independently written structure-member return types.
    pub fn extend_property_implementation(
        base: &SourceTypeApplicationHandoff,
        input: SourceTypeStructureMemberHandoffInput,
        arena: &TypedArena,
    ) -> Result<SourceTypeApplicationHandoff, SourceTypeError> {
        if !base.structure_members.is_empty() {
            return Err(SourceTypeError::StructureMembersAlreadyPresent);
        }
        if input.members.is_empty() {
            return Err(SourceTypeError::EmptyStructureMembers);
        }
        if input.members.len() != 2 {
            return Err(SourceTypeError::StructureMemberExtensionCardinalityMismatch);
        }
        if input.source_id != base.source_id || input.module_id != base.module_id {
            return Err(SourceTypeError::EnvironmentMismatch);
        }
        validate_property_implementation_structure_member_base(base, arena)?;

        let mut handoff = base.clone();
        for (index, member) in input.members.into_iter().enumerate() {
            let root = SourceTypeExpressionId::new(handoff.expressions.entries.len());
            handoff
                .structure_members
                .entries
                .push(SourceTypeStructureMember {
                    id: SourceTypeStructureMemberId::new(index),
                    member_site: member.member_site,
                    member_range: member.member_range,
                    source_ordinal: member.source_ordinal,
                    root,
                });
            handoff.expressions.entries.push(SourceTypeExpression {
                id: root,
                source_id: member.expression.source_id,
                module_id: member.expression.module_id,
                site: member.expression.site,
                source_range: member.expression.source_range,
                spelling: member.expression.spelling,
                head_site: member.expression.head_site,
                head_range: member.expression.head_range,
                head_spelling: member.expression.head_spelling,
                form: member.expression.form,
                head: member.expression.head,
                recovery: member.expression.recovery,
            });
        }
        validate_structure_member_handoff(&handoff, arena)?;
        Ok(handoff)
    }
}

/// Extends one exact Task-262 lower base with its standalone mode RHS without
/// fabricating a binding-linked application.
pub struct SourceTypeModeRhsProducer;

impl SourceTypeModeRhsProducer {
    pub fn extend(
        base: &SourceTypeApplicationHandoff,
        input: SourceTypeModeRhsExtensionInput,
        arena: &TypedArena,
    ) -> Result<SourceTypeApplicationHandoff, SourceTypeError> {
        if !base.mode_rhs.is_empty() {
            return Err(SourceTypeError::ModeRhsAlreadyPresent);
        }
        if input.rhs.is_empty() {
            return Err(SourceTypeError::EmptyModeRhs);
        }
        if input.rhs.len() != 1 {
            return Err(SourceTypeError::ModeRhsCardinalityMismatch);
        }
        if input.source_id != base.source_id || input.module_id != base.module_id {
            return Err(SourceTypeError::EnvironmentMismatch);
        }
        validate_mode_rhs_base(base, arena)?;

        let mut handoff = base.clone();
        let input = input.rhs.into_iter().next().expect("cardinality checked");
        let root = SourceTypeExpressionId::new(handoff.expressions.entries.len());
        handoff.mode_rhs.entries.push(SourceTypeModeRhs {
            id: SourceTypeModeRhsId::new(0),
            definition_site: input.definition_site,
            definition_range: input.definition_range,
            source_ordinal: input.source_ordinal,
            root,
        });
        handoff.expressions.entries.push(SourceTypeExpression {
            id: root,
            source_id: input.expression.source_id,
            module_id: input.expression.module_id,
            site: input.expression.site,
            source_range: input.expression.source_range,
            spelling: input.expression.spelling,
            head_site: input.expression.head_site,
            head_range: input.expression.head_range,
            head_spelling: input.expression.head_spelling,
            form: input.expression.form,
            head: input.expression.head,
            recovery: input.expression.recovery,
        });
        validate_mode_rhs_extension(&handoff, arena)?;
        Ok(handoff)
    }
}

/// Extends one exact Task-249 base with independently owned definition return
/// types without fabricating binding-linked applications.
pub struct SourceTypeDefinitionReturnProducer;

impl SourceTypeDefinitionReturnProducer {
    pub fn extend(
        base: &SourceTypeApplicationHandoff,
        input: SourceTypeDefinitionReturnExtensionInput,
        arena: &TypedArena,
    ) -> Result<SourceTypeApplicationHandoff, SourceTypeError> {
        if !base.definition_returns.is_empty() {
            return Err(SourceTypeError::DefinitionReturnsAlreadyPresent);
        }
        if input.returns.is_empty() {
            return Err(SourceTypeError::EmptyDefinitionReturns);
        }
        if input.returns.len() != 2 {
            return Err(SourceTypeError::DefinitionReturnCardinalityMismatch);
        }
        if input.source_id != base.source_id || input.module_id != base.module_id {
            return Err(SourceTypeError::EnvironmentMismatch);
        }
        validate_definition_return_base(base, arena)?;

        let mut handoff = base.clone();
        for (index, input) in input.returns.into_iter().enumerate() {
            let root = SourceTypeExpressionId::new(handoff.expressions.entries.len());
            handoff
                .definition_returns
                .entries
                .push(SourceTypeDefinitionReturn {
                    id: SourceTypeDefinitionReturnId::new(index),
                    definition_site: input.definition_site,
                    definition_range: input.definition_range,
                    source_ordinal: input.source_ordinal,
                    root,
                });
            handoff.expressions.entries.push(SourceTypeExpression {
                id: root,
                source_id: input.expression.source_id,
                module_id: input.expression.module_id,
                site: input.expression.site,
                source_range: input.expression.source_range,
                spelling: input.expression.spelling,
                head_site: input.expression.head_site,
                head_range: input.expression.head_range,
                head_spelling: input.expression.head_spelling,
                form: input.expression.form,
                head: input.expression.head,
                recovery: input.expression.recovery,
            });
        }
        validate_definition_return_extension(&handoff, arena)?;
        Ok(handoff)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum SourceTypeError {
    EmptyApplications,
    EmptyExpressions,
    EnvironmentMismatch,
    BindingCardinalityMismatch,
    InvalidApplication {
        application: SourceTypeApplicationId,
    },
    InvalidBinding {
        application: SourceTypeApplicationId,
    },
    InvalidExpression {
        expression: SourceTypeExpressionId,
    },
    InvalidHead {
        expression: SourceTypeExpressionId,
    },
    InvalidSymbolHead {
        expression: SourceTypeExpressionId,
    },
    InvalidExpressionSite {
        expression: SourceTypeExpressionId,
    },
    InvalidArgument {
        argument: SourceTypeArgumentId,
    },
    InvalidArgumentSite {
        argument: SourceTypeArgumentId,
    },
    InvalidProvenance {
        argument: SourceTypeArgumentId,
    },
    DuplicateSite,
    ReorderedArgument {
        argument: SourceTypeArgumentId,
    },
    DanglingChild {
        argument: SourceTypeArgumentId,
        child: SourceTypeExpressionId,
    },
    DuplicateChild {
        argument: SourceTypeArgumentId,
        child: SourceTypeExpressionId,
    },
    MultipleParents {
        child: SourceTypeExpressionId,
    },
    RootHasParent {
        root: SourceTypeExpressionId,
    },
    ForwardParent {
        parent: SourceTypeExpressionId,
        child: SourceTypeExpressionId,
    },
    Cycle {
        expression: SourceTypeExpressionId,
    },
    UnreachableExpression {
        expression: SourceTypeExpressionId,
    },
    WrongApplicationForm {
        expression: SourceTypeExpressionId,
    },
    ChildOutsideParent {
        parent: SourceTypeExpressionId,
        child: SourceTypeExpressionId,
    },
    OverlappingSiblings {
        parent: SourceTypeExpressionId,
    },
    OverlappingApplications {
        application: SourceTypeApplicationId,
    },
    EmptyDefinitionReturns,
    DefinitionReturnCardinalityMismatch,
    DefinitionReturnsAlreadyPresent,
    InvalidDefinitionReturnBase,
    InvalidDefinitionReturn {
        definition_return: SourceTypeDefinitionReturnId,
    },
    InvalidDefinitionReturnSite {
        definition_return: SourceTypeDefinitionReturnId,
    },
    UnsupportedDefinitionReturn {
        definition_return: SourceTypeDefinitionReturnId,
    },
    OverlappingDefinitionReturns {
        definition_return: SourceTypeDefinitionReturnId,
    },
    EmptyModeRhs,
    ModeRhsCardinalityMismatch,
    ModeRhsAlreadyPresent,
    InvalidModeRhsBase,
    InvalidModeRhs {
        mode_rhs: SourceTypeModeRhsId,
    },
    InvalidModeRhsSite {
        mode_rhs: SourceTypeModeRhsId,
    },
    UnsupportedModeRhs {
        mode_rhs: SourceTypeModeRhsId,
    },
    EmptyStructureMembers,
    StructureMemberCardinalityMismatch,
    StructureMembersAlreadyPresent,
    StructureMemberExtensionCardinalityMismatch,
    InvalidStructureMemberBase,
    InvalidStructureMember {
        structure_member: SourceTypeStructureMemberId,
    },
    InvalidStructureMemberSite {
        structure_member: SourceTypeStructureMemberId,
    },
    UnsupportedStructureMember {
        structure_member: SourceTypeStructureMemberId,
    },
}

impl fmt::Display for SourceTypeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyApplications => formatter.write_str("source type input has no applications"),
            Self::EmptyExpressions => formatter.write_str("source type input has no expressions"),
            Self::EnvironmentMismatch => {
                formatter.write_str("source type input environment identity mismatch")
            }
            Self::BindingCardinalityMismatch => {
                formatter.write_str("source type application and binding cardinalities differ")
            }
            Self::InvalidApplication { application } => write!(
                formatter,
                "source type application {} is invalid",
                application.index()
            ),
            Self::InvalidBinding { application } => write!(
                formatter,
                "source type application {} has an invalid binding",
                application.index()
            ),
            Self::InvalidExpression { expression } => write!(
                formatter,
                "source type expression {} is invalid",
                expression.index()
            ),
            Self::InvalidHead { expression } => write!(
                formatter,
                "source type expression {} has an invalid head",
                expression.index()
            ),
            Self::InvalidSymbolHead { expression } => write!(
                formatter,
                "source type expression {} has an unauthenticated symbol head",
                expression.index()
            ),
            Self::InvalidExpressionSite { expression } => write!(
                formatter,
                "source type expression {} has an invalid typed site",
                expression.index()
            ),
            Self::InvalidArgument { argument } => write!(
                formatter,
                "source type argument {} is invalid",
                argument.index()
            ),
            Self::InvalidArgumentSite { argument } => write!(
                formatter,
                "source type argument {} has an invalid typed site",
                argument.index()
            ),
            Self::InvalidProvenance { argument } => write!(
                formatter,
                "source type argument {} has invalid provenance",
                argument.index()
            ),
            Self::DuplicateSite => formatter.write_str("source type input repeats a typed site"),
            Self::ReorderedArgument { argument } => write!(
                formatter,
                "source type argument {} is out of canonical order",
                argument.index()
            ),
            Self::DanglingChild { argument, child } => write!(
                formatter,
                "source type argument {} references missing expression {}",
                argument.index(),
                child.index()
            ),
            Self::DuplicateChild { argument, child } => write!(
                formatter,
                "source type argument {} repeats expression {}",
                argument.index(),
                child.index()
            ),
            Self::MultipleParents { child } => write!(
                formatter,
                "source type expression {} has multiple parents",
                child.index()
            ),
            Self::RootHasParent { root } => write!(
                formatter,
                "source type root expression {} also has a parent",
                root.index()
            ),
            Self::ForwardParent { parent, child } => write!(
                formatter,
                "source type parent {} does not precede child {}",
                parent.index(),
                child.index()
            ),
            Self::Cycle { expression } => write!(
                formatter,
                "source type expression {} participates in a cycle",
                expression.index()
            ),
            Self::UnreachableExpression { expression } => write!(
                formatter,
                "source type expression {} is unreachable",
                expression.index()
            ),
            Self::WrongApplicationForm { expression } => write!(
                formatter,
                "source type expression {} has arguments incompatible with its form",
                expression.index()
            ),
            Self::ChildOutsideParent { parent, child } => write!(
                formatter,
                "source type expression {} does not contain child {}",
                parent.index(),
                child.index()
            ),
            Self::OverlappingSiblings { parent } => write!(
                formatter,
                "source type expression {} has overlapping argument siblings",
                parent.index()
            ),
            Self::OverlappingApplications { application } => write!(
                formatter,
                "source type application {} overlaps its predecessor",
                application.index()
            ),
            Self::EmptyDefinitionReturns => {
                formatter.write_str("source type definition return input is empty")
            }
            Self::DefinitionReturnCardinalityMismatch => formatter
                .write_str("source type definition return cardinality is not the frozen pair"),
            Self::DefinitionReturnsAlreadyPresent => {
                formatter.write_str("source type definition returns are already installed")
            }
            Self::InvalidDefinitionReturnBase => {
                formatter.write_str("source type definition return base is invalid")
            }
            Self::InvalidDefinitionReturn { definition_return } => write!(
                formatter,
                "source type definition return {} is invalid",
                definition_return.index()
            ),
            Self::InvalidDefinitionReturnSite { definition_return } => write!(
                formatter,
                "source type definition return {} has an invalid typed site",
                definition_return.index()
            ),
            Self::UnsupportedDefinitionReturn { definition_return } => write!(
                formatter,
                "source type definition return {} has an unsupported expression",
                definition_return.index()
            ),
            Self::OverlappingDefinitionReturns { definition_return } => write!(
                formatter,
                "source type definition return {} overlaps its predecessor",
                definition_return.index()
            ),
            Self::EmptyModeRhs => formatter.write_str("source type mode RHS input is empty"),
            Self::ModeRhsCardinalityMismatch => {
                formatter.write_str("source type mode RHS cardinality is not the frozen singleton")
            }
            Self::ModeRhsAlreadyPresent => {
                formatter.write_str("source type mode RHS is already installed")
            }
            Self::InvalidModeRhsBase => formatter.write_str("source type mode RHS base is invalid"),
            Self::InvalidModeRhs { mode_rhs } => write!(
                formatter,
                "source type mode RHS {} is invalid",
                mode_rhs.index()
            ),
            Self::InvalidModeRhsSite { mode_rhs } => write!(
                formatter,
                "source type mode RHS {} has an invalid typed site",
                mode_rhs.index()
            ),
            Self::UnsupportedModeRhs { mode_rhs } => write!(
                formatter,
                "source type mode RHS {} has an unsupported expression",
                mode_rhs.index()
            ),
            Self::EmptyStructureMembers => {
                formatter.write_str("source type structure-member input is empty")
            }
            Self::StructureMemberCardinalityMismatch => formatter.write_str(
                "source type structure-member cardinality is not the frozen four-row profile",
            ),
            Self::StructureMembersAlreadyPresent => {
                formatter.write_str("source type structure members are already installed")
            }
            Self::StructureMemberExtensionCardinalityMismatch => formatter.write_str(
                "source type structure-member extension cardinality is not the frozen pair",
            ),
            Self::InvalidStructureMemberBase => {
                formatter.write_str("source type structure-member base is invalid")
            }
            Self::InvalidStructureMember { structure_member } => write!(
                formatter,
                "source type structure member {} is invalid",
                structure_member.index()
            ),
            Self::InvalidStructureMemberSite { structure_member } => write!(
                formatter,
                "source type structure member {} has an invalid typed site",
                structure_member.index()
            ),
            Self::UnsupportedStructureMember { structure_member } => write!(
                formatter,
                "source type structure member {} has an unsupported expression",
                structure_member.index()
            ),
        }
    }
}

impl Error for SourceTypeError {}

const TASK_249R_BASE_EXPRESSIONS: [(usize, usize, usize, usize); 2] =
    [(63, 62, 22, 25), (67, 66, 38, 41)];
const TASK_249R_DEFINITION_RETURNS: [(usize, usize, usize, usize, usize, usize); 2] =
    [(84, 61, 118, 80, 79, 105), (95, 121, 179, 87, 86, 163)];
const TASK_249M_BASE_EXPRESSIONS: [(usize, usize, usize, usize); 2] =
    [(35, 34, 22, 25), (39, 38, 38, 41)];
const TASK_249M_MODE_RHS: (usize, usize, usize, usize, usize, usize, usize) =
    (49, 45, 135, 44, 43, 95, 98);
const TASK_249S_STRUCTURE_MEMBERS: [(usize, usize, usize, usize, usize, usize, usize); 4] = [
    (53, 42, 63, 52, 51, 59, 62),
    (56, 68, 91, 55, 54, 87, 90),
    (61, 134, 155, 60, 59, 151, 154),
    (64, 160, 183, 63, 62, 179, 182),
];
const TASK_249PI_MEANS_PARAMETER_SITES: (usize, usize) = (63, 64);
const TASK_249PI_EQUALS_PARAMETER_SITES: (usize, usize) = (45, 46);
type Task249PiStructureMemberProfile = [(usize, usize, usize, usize, usize, usize, usize); 2];
const TASK_249PI_MEANS_STRUCTURE_MEMBERS: Task249PiStructureMemberProfile =
    [(56, 45, 66, 55, 54, 62, 65), (59, 71, 94, 58, 57, 90, 93)];
const TASK_249PI_EQUALS_STRUCTURE_MEMBERS: Task249PiStructureMemberProfile =
    [(38, 45, 66, 37, 36, 62, 65), (41, 71, 94, 40, 39, 90, 93)];

fn validate_definition_return_base(
    base: &SourceTypeApplicationHandoff,
    arena: &TypedArena,
) -> Result<(), SourceTypeError> {
    if !base.definition_returns.is_empty()
        || !base.mode_rhs.is_empty()
        || !base.structure_members.is_empty()
        || base.expressions.len() != 2
        || !task_249r_base_shape_matches(base)
        || base
            .validate_installation(base.source_id, &base.module_id, arena)
            .is_err()
    {
        return Err(SourceTypeError::InvalidDefinitionReturnBase);
    }
    Ok(())
}

fn task_249r_base_shape_matches(handoff: &SourceTypeApplicationHandoff) -> bool {
    if handoff.applications.len() != 2
        || handoff.expressions.len() < 2
        || !handoff.arguments.is_empty()
        || !handoff.structure_members.is_empty()
    {
        return false;
    }
    for (index, (site, head_site, start, end)) in TASK_249R_BASE_EXPRESSIONS.into_iter().enumerate()
    {
        let Some(application) = handoff
            .applications
            .get(SourceTypeApplicationId::new(index))
        else {
            return false;
        };
        let Some(expression) = handoff.expressions.get(SourceTypeExpressionId::new(index)) else {
            return false;
        };
        if application.id != SourceTypeApplicationId::new(index)
            || application.binding != BindingId::new(index)
            || application.source_ordinal != index
            || application.root != SourceTypeExpressionId::new(index)
            || expression.id != SourceTypeExpressionId::new(index)
            || expression.source_id != handoff.source_id
            || expression.module_id != handoff.module_id
            || expression.source_range != task_249r_range(handoff.source_id, start, end)
            || !is_node_site(&expression.site, site)
            || expression.spelling != "set"
            || !is_node_site(&expression.head_site, head_site)
            || expression.head_range != task_249r_range(handoff.source_id, start, end)
            || expression.head_spelling != "set"
            || expression.form != SourceTypeApplicationForm::Bare
            || !matches!(expression.head, SourceTypeHead::BuiltinSet)
            || expression.recovery != NodeRecoveryState::Normal
        {
            return false;
        }
    }
    true
}

fn validate_definition_return_extension(
    handoff: &SourceTypeApplicationHandoff,
    arena: &TypedArena,
) -> Result<(), SourceTypeError> {
    if handoff.definition_returns.is_empty() {
        return Ok(());
    }
    if !handoff.mode_rhs.is_empty() || !handoff.structure_members.is_empty() {
        return Err(SourceTypeError::InvalidDefinitionReturnBase);
    }
    if handoff.definition_returns.len() != 2 || handoff.expressions.len() != 4 {
        return Err(SourceTypeError::DefinitionReturnCardinalityMismatch);
    }
    if !task_249r_base_shape_matches(handoff) {
        return Err(SourceTypeError::InvalidDefinitionReturnBase);
    }

    let mut sites = BTreeSet::new();
    for index in 0..2 {
        let expression = handoff
            .expressions
            .get(SourceTypeExpressionId::new(index))
            .expect("Task 249R base shape checked");
        sites.insert(expression.site.clone());
        sites.insert(expression.head_site.clone());
    }

    let mut previous_definition_range = None;
    for (index, (definition_node, definition_start, definition_end, site, head_site, start)) in
        TASK_249R_DEFINITION_RETURNS.into_iter().enumerate()
    {
        let id = SourceTypeDefinitionReturnId::new(index);
        let Some(definition_return) = handoff.definition_returns.get(id) else {
            return Err(SourceTypeError::InvalidDefinitionReturn {
                definition_return: id,
            });
        };
        let expected_root = SourceTypeExpressionId::new(index + 2);
        let Some(expression) = handoff.expressions.get(expected_root) else {
            return Err(SourceTypeError::InvalidDefinitionReturn {
                definition_return: id,
            });
        };
        if definition_return.id != id
            || definition_return.source_ordinal != index
            || definition_return.root != expected_root
            || !valid_range(handoff.source_id, definition_return.definition_range)
            || !range_contains(definition_return.definition_range, expression.source_range)
        {
            return Err(SourceTypeError::InvalidDefinitionReturn {
                definition_return: id,
            });
        }
        if previous_definition_range.is_some_and(|previous: SourceRange| {
            previous.start >= definition_return.definition_range.start
                || previous.end > definition_return.definition_range.start
        }) {
            return Err(SourceTypeError::OverlappingDefinitionReturns {
                definition_return: id,
            });
        }
        previous_definition_range = Some(definition_return.definition_range);

        if definition_return.definition_range
            != task_249r_range(handoff.source_id, definition_start, definition_end)
        {
            return Err(SourceTypeError::InvalidDefinitionReturn {
                definition_return: id,
            });
        }
        if !is_node_site(&definition_return.definition_site, definition_node)
            || !sites.insert(definition_return.definition_site.clone())
        {
            return Err(SourceTypeError::InvalidDefinitionReturnSite {
                definition_return: id,
            });
        }
        validate_definition_owner_site(
            id,
            &definition_return.definition_site,
            definition_return.definition_range,
            arena,
        )?;

        let end = start + 3;
        if expression.id != expected_root
            || expression.source_id != handoff.source_id
            || expression.module_id != handoff.module_id
            || expression.source_range != task_249r_range(handoff.source_id, start, end)
            || expression.head_range != task_249r_range(handoff.source_id, start, end)
        {
            return Err(SourceTypeError::InvalidDefinitionReturn {
                definition_return: id,
            });
        }
        if expression.form != SourceTypeApplicationForm::Bare
            || !matches!(expression.head, SourceTypeHead::BuiltinSet)
            || expression.spelling != "set"
            || expression.head_spelling != "set"
            || expression.recovery != NodeRecoveryState::Normal
        {
            return Err(SourceTypeError::UnsupportedDefinitionReturn {
                definition_return: id,
            });
        }
        if !is_node_site(&expression.site, site)
            || !is_node_site(&expression.head_site, head_site)
            || !sites.insert(expression.site.clone())
            || !sites.insert(expression.head_site.clone())
        {
            return Err(SourceTypeError::InvalidDefinitionReturnSite {
                definition_return: id,
            });
        }
        validate_arena_site(
            expected_root,
            &expression.site,
            expression.source_range,
            expression.recovery,
            arena,
        )
        .map_err(|_| SourceTypeError::InvalidDefinitionReturnSite {
            definition_return: id,
        })?;
        validate_arena_site(
            expected_root,
            &expression.head_site,
            expression.head_range,
            expression.recovery,
            arena,
        )
        .map_err(|_| SourceTypeError::InvalidDefinitionReturnSite {
            definition_return: id,
        })?;
    }
    Ok(())
}

fn validate_definition_owner_site(
    definition_return: SourceTypeDefinitionReturnId,
    site: &TypedSiteRef,
    range: SourceRange,
    arena: &TypedArena,
) -> Result<(), SourceTypeError> {
    let TypedSiteRef::Node(node_id) = site else {
        return Err(SourceTypeError::InvalidDefinitionReturnSite { definition_return });
    };
    let Some(node) = arena.node(*node_id) else {
        return Err(SourceTypeError::InvalidDefinitionReturnSite { definition_return });
    };
    if node.recovery != NodeRecoveryState::Normal || source_range(&node.anchor) != Some(range) {
        return Err(SourceTypeError::InvalidDefinitionReturnSite { definition_return });
    }
    Ok(())
}

fn is_node_site(site: &TypedSiteRef, expected: usize) -> bool {
    matches!(site, TypedSiteRef::Node(node) if node.index() == expected)
}

fn task_249r_range(source_id: SourceId, start: usize, end: usize) -> SourceRange {
    SourceRange {
        source_id,
        start,
        end,
    }
}

fn validate_mode_rhs_base(
    base: &SourceTypeApplicationHandoff,
    arena: &TypedArena,
) -> Result<(), SourceTypeError> {
    if !base.mode_rhs.is_empty()
        || !base.definition_returns.is_empty()
        || !base.structure_members.is_empty()
        || base.expressions.len() != 2
        || !task_249m_base_shape_matches(base)
        || base
            .validate_installation(base.source_id, &base.module_id, arena)
            .is_err()
    {
        return Err(SourceTypeError::InvalidModeRhsBase);
    }
    Ok(())
}

fn task_249m_base_shape_matches(handoff: &SourceTypeApplicationHandoff) -> bool {
    if handoff.applications.len() != 2
        || handoff.expressions.len() < 2
        || !handoff.arguments.is_empty()
        || !handoff.definition_returns.is_empty()
        || !handoff.structure_members.is_empty()
    {
        return false;
    }
    for (index, (site, head_site, start, end)) in TASK_249M_BASE_EXPRESSIONS.into_iter().enumerate()
    {
        let Some(application) = handoff
            .applications
            .get(SourceTypeApplicationId::new(index))
        else {
            return false;
        };
        let Some(expression) = handoff.expressions.get(SourceTypeExpressionId::new(index)) else {
            return false;
        };
        if application.id != SourceTypeApplicationId::new(index)
            || application.binding != BindingId::new(index)
            || application.source_ordinal != index
            || application.root != SourceTypeExpressionId::new(index)
            || expression.id != SourceTypeExpressionId::new(index)
            || expression.source_id != handoff.source_id
            || expression.module_id != handoff.module_id
            || expression.source_range != task_249m_range(handoff.source_id, start, end)
            || !is_node_site(&expression.site, site)
            || expression.spelling != "set"
            || !is_node_site(&expression.head_site, head_site)
            || expression.head_range != task_249m_range(handoff.source_id, start, end)
            || expression.head_spelling != "set"
            || expression.form != SourceTypeApplicationForm::Bare
            || !matches!(expression.head, SourceTypeHead::BuiltinSet)
            || expression.recovery != NodeRecoveryState::Normal
        {
            return false;
        }
    }
    true
}

fn validate_mode_rhs_extension(
    handoff: &SourceTypeApplicationHandoff,
    arena: &TypedArena,
) -> Result<(), SourceTypeError> {
    if handoff.mode_rhs.is_empty() {
        return Ok(());
    }
    if handoff.mode_rhs.len() != 1 || handoff.expressions.len() != 3 {
        return Err(SourceTypeError::ModeRhsCardinalityMismatch);
    }
    if !handoff.definition_returns.is_empty()
        || !handoff.structure_members.is_empty()
        || !task_249m_base_shape_matches(handoff)
    {
        return Err(SourceTypeError::InvalidModeRhsBase);
    }

    let id = SourceTypeModeRhsId::new(0);
    let Some(mode_rhs) = handoff.mode_rhs.get(id) else {
        return Err(SourceTypeError::InvalidModeRhs { mode_rhs: id });
    };
    let expected_root = SourceTypeExpressionId::new(2);
    let Some(expression) = handoff.expressions.get(expected_root) else {
        return Err(SourceTypeError::InvalidModeRhs { mode_rhs: id });
    };
    let (definition_node, definition_start, definition_end, site, head_site, start, end) =
        TASK_249M_MODE_RHS;
    if mode_rhs.id != id
        || mode_rhs.source_ordinal != 0
        || mode_rhs.root != expected_root
        || mode_rhs.definition_range
            != task_249m_range(handoff.source_id, definition_start, definition_end)
        || !valid_range(handoff.source_id, mode_rhs.definition_range)
        || !range_contains(mode_rhs.definition_range, expression.source_range)
        || expression.id != expected_root
        || expression.source_id != handoff.source_id
        || expression.module_id != handoff.module_id
        || expression.source_range != task_249m_range(handoff.source_id, start, end)
        || expression.head_range != task_249m_range(handoff.source_id, start, end)
    {
        return Err(SourceTypeError::InvalidModeRhs { mode_rhs: id });
    }

    let mut sites = BTreeSet::new();
    for index in 0..2 {
        let base = handoff
            .expressions
            .get(SourceTypeExpressionId::new(index))
            .expect("Task 249M base shape checked");
        sites.insert(base.site.clone());
        sites.insert(base.head_site.clone());
    }
    if !is_node_site(&mode_rhs.definition_site, definition_node)
        || !sites.insert(mode_rhs.definition_site.clone())
        || !is_node_site(&expression.site, site)
        || !is_node_site(&expression.head_site, head_site)
        || !sites.insert(expression.site.clone())
        || !sites.insert(expression.head_site.clone())
    {
        return Err(SourceTypeError::InvalidModeRhsSite { mode_rhs: id });
    }
    validate_mode_rhs_owner_site(
        id,
        &mode_rhs.definition_site,
        mode_rhs.definition_range,
        arena,
    )?;
    validate_arena_site(
        expected_root,
        &expression.site,
        expression.source_range,
        NodeRecoveryState::Normal,
        arena,
    )
    .map_err(|_| SourceTypeError::InvalidModeRhsSite { mode_rhs: id })?;
    validate_arena_site(
        expected_root,
        &expression.head_site,
        expression.head_range,
        NodeRecoveryState::Normal,
        arena,
    )
    .map_err(|_| SourceTypeError::InvalidModeRhsSite { mode_rhs: id })?;

    if expression.form != SourceTypeApplicationForm::Bare
        || !matches!(expression.head, SourceTypeHead::BuiltinSet)
        || expression.spelling != "set"
        || expression.head_spelling != "set"
        || expression.recovery != NodeRecoveryState::Normal
    {
        return Err(SourceTypeError::UnsupportedModeRhs { mode_rhs: id });
    }
    Ok(())
}

fn validate_mode_rhs_owner_site(
    mode_rhs: SourceTypeModeRhsId,
    site: &TypedSiteRef,
    range: SourceRange,
    arena: &TypedArena,
) -> Result<(), SourceTypeError> {
    let TypedSiteRef::Node(node_id) = site else {
        return Err(SourceTypeError::InvalidModeRhsSite { mode_rhs });
    };
    let Some(node) = arena.node(*node_id) else {
        return Err(SourceTypeError::InvalidModeRhsSite { mode_rhs });
    };
    if node.recovery != NodeRecoveryState::Normal || source_range(&node.anchor) != Some(range) {
        return Err(SourceTypeError::InvalidModeRhsSite { mode_rhs });
    }
    Ok(())
}

fn task_249m_range(source_id: SourceId, start: usize, end: usize) -> SourceRange {
    SourceRange {
        source_id,
        start,
        end,
    }
}

fn validate_property_implementation_structure_member_base(
    base: &SourceTypeApplicationHandoff,
    arena: &TypedArena,
) -> Result<(), SourceTypeError> {
    if !base.structure_members.is_empty()
        || !task_249pi_base_shape_matches(base)
        || base
            .validate_installation(base.source_id, &base.module_id, arena)
            .is_err()
    {
        return Err(SourceTypeError::InvalidStructureMemberBase);
    }
    Ok(())
}

fn task_249pi_base_shape_matches(handoff: &SourceTypeApplicationHandoff) -> bool {
    if handoff.applications.len() != 1
        || handoff.expressions.len() != 1
        || !handoff.arguments.is_empty()
        || !handoff.definition_returns.is_empty()
        || !handoff.mode_rhs.is_empty()
    {
        return false;
    }
    let Some(application) = handoff.applications.get(SourceTypeApplicationId::new(0)) else {
        return false;
    };
    let Some(expression) = handoff.expressions.get(SourceTypeExpressionId::new(0)) else {
        return false;
    };
    let sites_match = (is_node_site(&expression.site, TASK_249PI_MEANS_PARAMETER_SITES.0)
        && is_node_site(&expression.head_site, TASK_249PI_MEANS_PARAMETER_SITES.1))
        || (is_node_site(&expression.site, TASK_249PI_EQUALS_PARAMETER_SITES.0)
            && is_node_site(&expression.head_site, TASK_249PI_EQUALS_PARAMETER_SITES.1));
    let authenticated_symbol_matches = match &expression.head {
        SourceTypeHead::Symbol {
            symbol,
            contribution,
        } => symbol.module() == &handoff.module_id && contribution.index() == 0,
        SourceTypeHead::BuiltinSet | SourceTypeHead::BuiltinObject => false,
    };
    application.id == SourceTypeApplicationId::new(0)
        && application.binding == BindingId::new(0)
        && application.source_ordinal == 0
        && application.root == SourceTypeExpressionId::new(0)
        && expression.id == SourceTypeExpressionId::new(0)
        && expression.source_id == handoff.source_id
        && expression.module_id == handoff.module_id
        && expression.source_range == task_249pi_range(handoff.source_id, 130, 144)
        && expression.head_range == task_249pi_range(handoff.source_id, 130, 144)
        && sites_match
        && expression.spelling == "Task264Carrier"
        && expression.head_spelling == "Task264Carrier"
        && expression.form == SourceTypeApplicationForm::Bare
        && authenticated_symbol_matches
        && expression.recovery == NodeRecoveryState::Normal
}

fn task_249pi_structure_member_profile(
    handoff: &SourceTypeApplicationHandoff,
) -> Option<&'static Task249PiStructureMemberProfile> {
    let expression = handoff.expressions.get(SourceTypeExpressionId::new(0))?;
    match (
        expression.site.node().index(),
        expression.head_site.node().index(),
    ) {
        TASK_249PI_MEANS_PARAMETER_SITES => Some(&TASK_249PI_MEANS_STRUCTURE_MEMBERS),
        TASK_249PI_EQUALS_PARAMETER_SITES => Some(&TASK_249PI_EQUALS_STRUCTURE_MEMBERS),
        _ => None,
    }
}

fn validate_task_249pi_structure_member_handoff(
    handoff: &SourceTypeApplicationHandoff,
    arena: &TypedArena,
) -> Result<(), SourceTypeError> {
    if handoff.structure_members.len() != 2 || handoff.expressions.len() != 3 {
        return Err(SourceTypeError::StructureMemberExtensionCardinalityMismatch);
    }
    if !task_249pi_base_shape_matches_with_extension(handoff) {
        return Err(SourceTypeError::InvalidStructureMemberBase);
    }
    let Some(profile) = task_249pi_structure_member_profile(handoff) else {
        return Err(SourceTypeError::InvalidStructureMemberBase);
    };

    for (index, (_, member_start, member_end, _, _, start, end)) in
        profile.iter().copied().enumerate()
    {
        let id = SourceTypeStructureMemberId::new(index);
        let root = SourceTypeExpressionId::new(index + 1);
        let Some(member) = handoff.structure_members.get(id) else {
            return Err(SourceTypeError::InvalidStructureMember {
                structure_member: id,
            });
        };
        let Some(expression) = handoff.expressions.get(root) else {
            return Err(SourceTypeError::InvalidStructureMember {
                structure_member: id,
            });
        };
        if member.id != id
            || member.source_ordinal != index
            || member.root != root
            || member.member_range != task_249pi_range(handoff.source_id, member_start, member_end)
            || !valid_range(handoff.source_id, member.member_range)
            || !range_contains(member.member_range, expression.source_range)
            || expression.id != root
            || expression.source_id != handoff.source_id
            || expression.module_id != handoff.module_id
            || expression.source_range != task_249pi_range(handoff.source_id, start, end)
            || expression.head_range != task_249pi_range(handoff.source_id, start, end)
        {
            return Err(SourceTypeError::InvalidStructureMember {
                structure_member: id,
            });
        }
    }

    let parameter = handoff
        .expressions
        .get(SourceTypeExpressionId::new(0))
        .expect("Task 249PI base shape checked");
    let mut sites = BTreeSet::from([parameter.site.clone(), parameter.head_site.clone()]);
    for (index, (member_node, _, _, expression_node, head_node, _, _)) in
        profile.iter().copied().enumerate()
    {
        let id = SourceTypeStructureMemberId::new(index);
        let root = SourceTypeExpressionId::new(index + 1);
        let member = handoff
            .structure_members
            .get(id)
            .expect("Task 249PI row shape checked");
        let expression = handoff
            .expressions
            .get(root)
            .expect("Task 249PI expression shape checked");
        if !is_node_site(&member.member_site, member_node)
            || !is_node_site(&expression.site, expression_node)
            || !is_node_site(&expression.head_site, head_node)
            || !sites.insert(member.member_site.clone())
            || !sites.insert(expression.site.clone())
            || !sites.insert(expression.head_site.clone())
        {
            return Err(SourceTypeError::InvalidStructureMemberSite {
                structure_member: id,
            });
        }
        validate_structure_member_owner_site(id, &member.member_site, member.member_range, arena)?;
        validate_arena_site(
            root,
            &expression.site,
            expression.source_range,
            NodeRecoveryState::Normal,
            arena,
        )
        .map_err(|_| SourceTypeError::InvalidStructureMemberSite {
            structure_member: id,
        })?;
        validate_arena_site(
            root,
            &expression.head_site,
            expression.head_range,
            NodeRecoveryState::Normal,
            arena,
        )
        .map_err(|_| SourceTypeError::InvalidStructureMemberSite {
            structure_member: id,
        })?;
    }

    for index in 0..profile.len() {
        let id = SourceTypeStructureMemberId::new(index);
        let root = SourceTypeExpressionId::new(index + 1);
        let expression = handoff
            .expressions
            .get(root)
            .expect("Task 249PI row shape checked");
        if expression.form != SourceTypeApplicationForm::Bare
            || !matches!(expression.head, SourceTypeHead::BuiltinSet)
            || expression.spelling != "set"
            || expression.head_spelling != "set"
            || expression.recovery != NodeRecoveryState::Normal
        {
            return Err(SourceTypeError::UnsupportedStructureMember {
                structure_member: id,
            });
        }
    }
    Ok(())
}

fn task_249pi_base_shape_matches_with_extension(handoff: &SourceTypeApplicationHandoff) -> bool {
    if handoff.expressions.len() != 3 || handoff.structure_members.len() != 2 {
        return false;
    }
    let mut base = handoff.clone();
    base.expressions.entries.truncate(1);
    base.structure_members.entries.clear();
    task_249pi_base_shape_matches(&base)
}

fn task_249pi_range(source_id: SourceId, start: usize, end: usize) -> SourceRange {
    SourceRange {
        source_id,
        start,
        end,
    }
}

fn validate_structure_member_handoff(
    handoff: &SourceTypeApplicationHandoff,
    arena: &TypedArena,
) -> Result<(), SourceTypeError> {
    if handoff.structure_members.is_empty() {
        if task_249pi_has_orphaned_structure_member_expressions(handoff) {
            return Err(SourceTypeError::InvalidStructureMemberBase);
        }
        if handoff.applications.is_empty() && !handoff.expressions.is_empty() {
            return Err(SourceTypeError::StructureMemberCardinalityMismatch);
        }
        return Ok(());
    }
    if handoff.structure_members.len() == 2
        && (!handoff.applications.is_empty()
            || handoff.expressions.len() != handoff.structure_members.len()
            || !handoff.arguments.is_empty()
            || !handoff.definition_returns.is_empty()
            || !handoff.mode_rhs.is_empty())
    {
        return validate_task_249pi_structure_member_handoff(handoff, arena);
    }
    validate_task_249s_structure_member_handoff(handoff, arena)
}

fn task_249pi_has_orphaned_structure_member_expressions(
    handoff: &SourceTypeApplicationHandoff,
) -> bool {
    if handoff.expressions.len() != 3
        || handoff.applications.len() != 1
        || !handoff.arguments.is_empty()
        || !handoff.definition_returns.is_empty()
        || !handoff.mode_rhs.is_empty()
    {
        return false;
    }
    let mut base = handoff.clone();
    base.expressions.entries.truncate(1);
    task_249pi_base_shape_matches(&base)
}

fn validate_task_249s_structure_member_handoff(
    handoff: &SourceTypeApplicationHandoff,
    arena: &TypedArena,
) -> Result<(), SourceTypeError> {
    if handoff.structure_members.len() != 4 || handoff.expressions.len() != 4 {
        return Err(SourceTypeError::StructureMemberCardinalityMismatch);
    }
    if !handoff.applications.is_empty()
        || !handoff.arguments.is_empty()
        || !handoff.definition_returns.is_empty()
        || !handoff.mode_rhs.is_empty()
    {
        return Err(SourceTypeError::InvalidStructureMember {
            structure_member: SourceTypeStructureMemberId::new(0),
        });
    }

    for (index, (_, member_start, member_end, _, _, start, end)) in
        TASK_249S_STRUCTURE_MEMBERS.into_iter().enumerate()
    {
        let id = SourceTypeStructureMemberId::new(index);
        let root = SourceTypeExpressionId::new(index);
        let Some(member) = handoff.structure_members.get(id) else {
            return Err(SourceTypeError::InvalidStructureMember {
                structure_member: id,
            });
        };
        let Some(expression) = handoff.expressions.get(root) else {
            return Err(SourceTypeError::InvalidStructureMember {
                structure_member: id,
            });
        };
        if member.id != id
            || member.source_ordinal != index
            || member.root != root
            || member.member_range != task_249s_range(handoff.source_id, member_start, member_end)
            || !valid_range(handoff.source_id, member.member_range)
            || !range_contains(member.member_range, expression.source_range)
            || expression.id != root
            || expression.source_id != handoff.source_id
            || expression.module_id != handoff.module_id
            || expression.source_range != task_249s_range(handoff.source_id, start, end)
            || expression.head_range != task_249s_range(handoff.source_id, start, end)
        {
            return Err(SourceTypeError::InvalidStructureMember {
                structure_member: id,
            });
        }
    }

    let mut sites = BTreeSet::new();
    for (index, (member_node, _, _, expression_node, head_node, _, _)) in
        TASK_249S_STRUCTURE_MEMBERS.into_iter().enumerate()
    {
        let id = SourceTypeStructureMemberId::new(index);
        let root = SourceTypeExpressionId::new(index);
        let Some(member) = handoff.structure_members.get(id) else {
            return Err(SourceTypeError::InvalidStructureMember {
                structure_member: id,
            });
        };
        let Some(expression) = handoff.expressions.get(root) else {
            return Err(SourceTypeError::InvalidStructureMember {
                structure_member: id,
            });
        };
        if !is_node_site(&member.member_site, member_node)
            || !is_node_site(&expression.site, expression_node)
            || !is_node_site(&expression.head_site, head_node)
            || !sites.insert(member.member_site.clone())
            || !sites.insert(expression.site.clone())
            || !sites.insert(expression.head_site.clone())
        {
            return Err(SourceTypeError::InvalidStructureMemberSite {
                structure_member: id,
            });
        }
        validate_structure_member_owner_site(id, &member.member_site, member.member_range, arena)?;
        validate_arena_site(
            root,
            &expression.site,
            expression.source_range,
            NodeRecoveryState::Normal,
            arena,
        )
        .map_err(|_| SourceTypeError::InvalidStructureMemberSite {
            structure_member: id,
        })?;
        validate_arena_site(
            root,
            &expression.head_site,
            expression.head_range,
            NodeRecoveryState::Normal,
            arena,
        )
        .map_err(|_| SourceTypeError::InvalidStructureMemberSite {
            structure_member: id,
        })?;
    }

    for index in 0..TASK_249S_STRUCTURE_MEMBERS.len() {
        let id = SourceTypeStructureMemberId::new(index);
        let root = SourceTypeExpressionId::new(index);
        let Some(expression) = handoff.expressions.get(root) else {
            return Err(SourceTypeError::InvalidStructureMember {
                structure_member: id,
            });
        };
        if expression.form != SourceTypeApplicationForm::Bare
            || !matches!(expression.head, SourceTypeHead::BuiltinSet)
            || expression.spelling != "set"
            || expression.head_spelling != "set"
            || expression.recovery != NodeRecoveryState::Normal
        {
            return Err(SourceTypeError::UnsupportedStructureMember {
                structure_member: id,
            });
        }
    }
    Ok(())
}

fn validate_structure_member_owner_site(
    structure_member: SourceTypeStructureMemberId,
    site: &TypedSiteRef,
    range: SourceRange,
    arena: &TypedArena,
) -> Result<(), SourceTypeError> {
    let TypedSiteRef::Node(node_id) = site else {
        return Err(SourceTypeError::InvalidStructureMemberSite { structure_member });
    };
    let Some(node) = arena.node(*node_id) else {
        return Err(SourceTypeError::InvalidStructureMemberSite { structure_member });
    };
    if node.recovery != NodeRecoveryState::Normal || source_range(&node.anchor) != Some(range) {
        return Err(SourceTypeError::InvalidStructureMemberSite { structure_member });
    }
    Ok(())
}

fn task_249s_range(source_id: SourceId, start: usize, end: usize) -> SourceRange {
    SourceRange {
        source_id,
        start,
        end,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SourceTypeBindingProfile {
    Generic,
    ProofLocalLet,
    ProofLocalGiven,
}

fn task269ct_binding_env(
    dependency: &SourceProofLocalLetBindingHandoff,
) -> Result<BindingEnv, SourceProofLocalLetTypeError> {
    let mut bindings = BindingTable::new();
    for (id, binding) in dependency.binding_env().bindings().iter() {
        let inserted = bindings.insert(BindingDraft {
            spelling: binding.spelling.clone(),
            kind: binding.kind,
            identity: binding.identity.clone(),
            owner_context: binding.owner_context,
            declaration_range: binding.declaration_range,
            visible_after_ordinal: binding.visible_after_ordinal,
            type_site: if id == BindingId::new(1) {
                BindingTypeSite::Source(task269ct_range(dependency.source_id(), 76, 79))
            } else {
                binding.type_site.clone()
            },
            status: binding.status,
            captured: binding.captured.clone(),
            diagnostics: binding.diagnostics.clone(),
            recovery: binding.recovery,
        });
        if inserted != id {
            return Err(SourceProofLocalLetTypeError::InvalidBindingEnvironment);
        }
    }
    let binding_env = BindingEnv::try_new(BindingEnvParts {
        source_id: dependency.source_id(),
        module_id: dependency.module_id().clone(),
        contexts: dependency.binding_env().contexts().clone(),
        bindings,
        diagnostics: dependency.binding_env().diagnostics().clone(),
    })
    .map_err(|_| SourceProofLocalLetTypeError::InvalidBindingEnvironment)?;
    let reserve = binding_env.bindings().get(BindingId::new(0));
    let local = binding_env.bindings().get(BindingId::new(1));
    if binding_env.contexts().len() != 2
        || binding_env.bindings().len() != 2
        || !binding_env.diagnostics().is_empty()
        || reserve.is_none_or(|binding| {
            binding.type_site
                != BindingTypeSite::Source(task269ct_range(dependency.source_id(), 14, 17))
        })
        || local.is_none_or(|binding| {
            binding.kind != BindingKind::LetBinding
                || binding.type_site
                    != BindingTypeSite::Source(task269ct_range(dependency.source_id(), 76, 79))
        })
    {
        return Err(SourceProofLocalLetTypeError::InvalidBindingEnvironment);
    }
    Ok(binding_env)
}

fn exact_task269ct_input(input: &SourceTypeHandoffInput) -> bool {
    if input.applications.len() != 2 || input.expressions.len() != 2 || !input.arguments.is_empty()
    {
        return false;
    }
    for (index, (start, end)) in [(14, 17), (76, 79)].into_iter().enumerate() {
        let application = &input.applications[index];
        let expression = &input.expressions[index];
        if application.binding != BindingId::new(index)
            || application.source_ordinal != index
            || application.root != SourceTypeExpressionId::new(index)
            || expression.source_id != input.source_id
            || expression.module_id != input.module_id
            || !task269ct_role_matches(
                &expression.site,
                TypedNodeId::new(index),
                "source.type.expression",
            )
            || expression.source_range != task269ct_range(input.source_id, start, end)
            || expression.spelling != "set"
            || !task269ct_role_matches(
                &expression.head_site,
                TypedNodeId::new(index),
                "source.type.head",
            )
            || expression.head_range != task269ct_range(input.source_id, start, end)
            || expression.head_spelling != "set"
            || expression.form != SourceTypeApplicationForm::Bare
            || expression.head != SourceTypeHead::BuiltinSet
            || expression.recovery != NodeRecoveryState::Normal
        {
            return false;
        }
    }
    true
}

fn exact_task269ct_source_type(
    handoff: &SourceTypeApplicationHandoff,
    source_id: SourceId,
    module_id: &ModuleId,
) -> bool {
    if handoff.source_id() != source_id
        || handoff.module_id() != module_id
        || handoff.applications().len() != 2
        || handoff.expressions().len() != 2
        || !handoff.arguments().is_empty()
        || !handoff.definition_returns().is_empty()
        || !handoff.mode_rhs().is_empty()
        || !handoff.structure_members().is_empty()
    {
        return false;
    }
    for (index, (start, end)) in [(14, 17), (76, 79)].into_iter().enumerate() {
        let Some(application) = handoff
            .applications()
            .get(SourceTypeApplicationId::new(index))
        else {
            return false;
        };
        let Some(expression) = handoff
            .expressions()
            .get(SourceTypeExpressionId::new(index))
        else {
            return false;
        };
        if application.id() != SourceTypeApplicationId::new(index)
            || application.binding() != BindingId::new(index)
            || application.source_ordinal() != index
            || application.root() != SourceTypeExpressionId::new(index)
            || expression.id() != SourceTypeExpressionId::new(index)
            || expression.source_id() != source_id
            || expression.module_id() != module_id
            || !task269ct_role_matches(
                expression.site(),
                TypedNodeId::new(index),
                "source.type.expression",
            )
            || expression.source_range() != task269ct_range(source_id, start, end)
            || expression.spelling() != "set"
            || !task269ct_role_matches(
                expression.head_site(),
                TypedNodeId::new(index),
                "source.type.head",
            )
            || expression.head_range() != task269ct_range(source_id, start, end)
            || expression.head_spelling() != "set"
            || expression.form() != SourceTypeApplicationForm::Bare
            || expression.head() != &SourceTypeHead::BuiltinSet
            || expression.recovery() != NodeRecoveryState::Normal
        {
            return false;
        }
    }
    true
}

fn exact_task269ct_arena(source_id: SourceId, arena: &TypedArena) -> bool {
    if arena.len() != 3 || arena.root() != Some(TypedNodeId::new(2)) {
        return false;
    }
    let expected = [
        ("source.proof-local.let.reserve-type", 14, 17, Vec::new()),
        ("source.proof-local.let.type", 76, 79, Vec::new()),
        (
            "source.proof-local.let.type-root",
            0,
            99,
            vec![TypedNodeId::new(0), TypedNodeId::new(1)],
        ),
    ];
    expected
        .into_iter()
        .enumerate()
        .all(|(index, (kind, start, end, children))| {
            arena.node(TypedNodeId::new(index)).is_some_and(|node| {
                node.kind.as_str() == kind
                    && node.resolved_node.is_none()
                    && node.anchor == SourceAnchor::Range(task269ct_range(source_id, start, end))
                    && node.children == children
                    && node.typing == TypingState::Unknown
                    && node.recovery == NodeRecoveryState::Normal
                    && node.links == TypedNodeLinks::default()
            })
        })
}

fn task269ct_role_matches(site: &TypedSiteRef, node: TypedNodeId, expected: &str) -> bool {
    matches!(
        site,
        TypedSiteRef::Role { node: actual, role }
            if *actual == node && role.as_str() == expected
    )
}

fn task269ct_range(source_id: SourceId, start: usize, end: usize) -> SourceRange {
    SourceRange {
        source_id,
        start,
        end,
    }
}

fn task269gt_binding_env(
    dependency: &SourceProofLocalGivenBindingHandoff,
) -> Result<BindingEnv, SourceProofLocalGivenTypeError> {
    let mut bindings = BindingTable::new();
    for (id, binding) in dependency.binding_env().bindings().iter() {
        let inserted = bindings.insert(BindingDraft {
            spelling: binding.spelling.clone(),
            kind: binding.kind,
            identity: binding.identity.clone(),
            owner_context: binding.owner_context,
            declaration_range: binding.declaration_range,
            visible_after_ordinal: binding.visible_after_ordinal,
            type_site: if id == BindingId::new(1) {
                BindingTypeSite::Source(task269gt_range(dependency.source_id(), 84, 87))
            } else {
                binding.type_site.clone()
            },
            status: binding.status,
            captured: binding.captured.clone(),
            diagnostics: binding.diagnostics.clone(),
            recovery: binding.recovery,
        });
        if inserted != id {
            return Err(SourceProofLocalGivenTypeError::InvalidBindingEnvironment);
        }
    }
    let binding_env = BindingEnv::try_new(BindingEnvParts {
        source_id: dependency.source_id(),
        module_id: dependency.module_id().clone(),
        contexts: dependency.binding_env().contexts().clone(),
        bindings,
        diagnostics: dependency.binding_env().diagnostics().clone(),
    })
    .map_err(|_| SourceProofLocalGivenTypeError::InvalidBindingEnvironment)?;
    let reserve = binding_env.bindings().get(BindingId::new(0));
    let local = binding_env.bindings().get(BindingId::new(1));
    if binding_env.contexts().len() != 2
        || binding_env.bindings().len() != 2
        || !binding_env.diagnostics().is_empty()
        || reserve.is_none_or(|binding| {
            binding.type_site
                != BindingTypeSite::Source(task269gt_range(dependency.source_id(), 14, 17))
        })
        || local.is_none_or(|binding| {
            binding.kind != BindingKind::GivenWitness
                || binding.type_site
                    != BindingTypeSite::Source(task269gt_range(dependency.source_id(), 84, 87))
        })
    {
        return Err(SourceProofLocalGivenTypeError::InvalidBindingEnvironment);
    }
    Ok(binding_env)
}

fn exact_task269gt_input(input: &SourceTypeHandoffInput) -> bool {
    if input.applications.len() != 2 || input.expressions.len() != 2 || !input.arguments.is_empty()
    {
        return false;
    }
    for (index, (start, end)) in [(14, 17), (84, 87)].into_iter().enumerate() {
        let application = &input.applications[index];
        let expression = &input.expressions[index];
        if application.binding != BindingId::new(index)
            || application.source_ordinal != index
            || application.root != SourceTypeExpressionId::new(index)
            || expression.source_id != input.source_id
            || expression.module_id != input.module_id
            || !task269ct_role_matches(
                &expression.site,
                TypedNodeId::new(index),
                "source.type.expression",
            )
            || expression.source_range != task269gt_range(input.source_id, start, end)
            || expression.spelling != "set"
            || !task269ct_role_matches(
                &expression.head_site,
                TypedNodeId::new(index),
                "source.type.head",
            )
            || expression.head_range != task269gt_range(input.source_id, start, end)
            || expression.head_spelling != "set"
            || expression.form != SourceTypeApplicationForm::Bare
            || expression.head != SourceTypeHead::BuiltinSet
            || expression.recovery != NodeRecoveryState::Normal
        {
            return false;
        }
    }
    true
}

fn exact_task269gt_source_type(
    handoff: &SourceTypeApplicationHandoff,
    source_id: SourceId,
    module_id: &ModuleId,
) -> bool {
    if handoff.source_id() != source_id
        || handoff.module_id() != module_id
        || handoff.applications().len() != 2
        || handoff.expressions().len() != 2
        || !handoff.arguments().is_empty()
        || !handoff.definition_returns().is_empty()
        || !handoff.mode_rhs().is_empty()
        || !handoff.structure_members().is_empty()
    {
        return false;
    }
    for (index, (start, end)) in [(14, 17), (84, 87)].into_iter().enumerate() {
        let Some(application) = handoff
            .applications()
            .get(SourceTypeApplicationId::new(index))
        else {
            return false;
        };
        let Some(expression) = handoff
            .expressions()
            .get(SourceTypeExpressionId::new(index))
        else {
            return false;
        };
        if application.id() != SourceTypeApplicationId::new(index)
            || application.binding() != BindingId::new(index)
            || application.source_ordinal() != index
            || application.root() != SourceTypeExpressionId::new(index)
            || expression.id() != SourceTypeExpressionId::new(index)
            || expression.source_id() != source_id
            || expression.module_id() != module_id
            || !task269ct_role_matches(
                expression.site(),
                TypedNodeId::new(index),
                "source.type.expression",
            )
            || expression.source_range() != task269gt_range(source_id, start, end)
            || expression.spelling() != "set"
            || !task269ct_role_matches(
                expression.head_site(),
                TypedNodeId::new(index),
                "source.type.head",
            )
            || expression.head_range() != task269gt_range(source_id, start, end)
            || expression.head_spelling() != "set"
            || expression.form() != SourceTypeApplicationForm::Bare
            || expression.head() != &SourceTypeHead::BuiltinSet
            || expression.recovery() != NodeRecoveryState::Normal
        {
            return false;
        }
    }
    true
}

fn exact_task269gt_arena(source_id: SourceId, arena: &TypedArena) -> bool {
    if arena.len() != 3 || arena.root() != Some(TypedNodeId::new(2)) {
        return false;
    }
    let expected = [
        ("source.proof-local.given.reserve-type", 14, 17, Vec::new()),
        ("source.proof-local.given.type", 84, 87, Vec::new()),
        (
            "source.proof-local.given.type-root",
            0,
            128,
            vec![TypedNodeId::new(0), TypedNodeId::new(1)],
        ),
    ];
    expected
        .into_iter()
        .enumerate()
        .all(|(index, (kind, start, end, children))| {
            arena.node(TypedNodeId::new(index)).is_some_and(|node| {
                node.kind.as_str() == kind
                    && node.resolved_node.is_none()
                    && node.anchor == SourceAnchor::Range(task269gt_range(source_id, start, end))
                    && node.children == children
                    && node.typing == TypingState::Unknown
                    && node.recovery == NodeRecoveryState::Normal
                    && node.links == TypedNodeLinks::default()
            })
        })
}

fn task269gt_range(source_id: SourceId, start: usize, end: usize) -> SourceRange {
    SourceRange {
        source_id,
        start,
        end,
    }
}

fn task269gct_binding_env(
    dependency: &SourceProofLocalGivenConditionBindingHandoff,
) -> Result<BindingEnv, SourceProofLocalGivenConditionTypeError> {
    let mut bindings = BindingTable::new();
    for (id, binding) in dependency.binding_env().bindings().iter() {
        let inserted = bindings.insert(BindingDraft {
            spelling: binding.spelling.clone(),
            kind: binding.kind,
            identity: binding.identity.clone(),
            owner_context: binding.owner_context,
            declaration_range: binding.declaration_range,
            visible_after_ordinal: binding.visible_after_ordinal,
            type_site: if id == BindingId::new(1) {
                BindingTypeSite::Source(task269gct_range(dependency.source_id(), 90, 93))
            } else {
                binding.type_site.clone()
            },
            status: binding.status,
            captured: binding.captured.clone(),
            diagnostics: binding.diagnostics.clone(),
            recovery: binding.recovery,
        });
        if inserted != id {
            return Err(SourceProofLocalGivenConditionTypeError::InvalidBindingEnvironment);
        }
    }
    let binding_env = BindingEnv::try_new(BindingEnvParts {
        source_id: dependency.source_id(),
        module_id: dependency.module_id().clone(),
        contexts: dependency.binding_env().contexts().clone(),
        bindings,
        diagnostics: dependency.binding_env().diagnostics().clone(),
    })
    .map_err(|_| SourceProofLocalGivenConditionTypeError::InvalidBindingEnvironment)?;
    let reserve = binding_env.bindings().get(BindingId::new(0));
    let local = binding_env.bindings().get(BindingId::new(1));
    if binding_env.contexts().len() != 2
        || binding_env.bindings().len() != 2
        || !binding_env.diagnostics().is_empty()
        || reserve.is_none_or(|binding| {
            binding.type_site
                != BindingTypeSite::Source(task269gct_range(dependency.source_id(), 14, 17))
        })
        || local.is_none_or(|binding| {
            binding.kind != BindingKind::GivenWitness
                || binding.type_site
                    != BindingTypeSite::Source(task269gct_range(dependency.source_id(), 90, 93))
        })
    {
        return Err(SourceProofLocalGivenConditionTypeError::InvalidBindingEnvironment);
    }
    Ok(binding_env)
}

fn exact_task269gct_input(input: &SourceTypeHandoffInput) -> bool {
    if input.applications.len() != 2 || input.expressions.len() != 2 || !input.arguments.is_empty()
    {
        return false;
    }
    for (index, (start, end)) in [(14, 17), (90, 93)].into_iter().enumerate() {
        let application = &input.applications[index];
        let expression = &input.expressions[index];
        if application.binding != BindingId::new(index)
            || application.source_ordinal != index
            || application.root != SourceTypeExpressionId::new(index)
            || expression.source_id != input.source_id
            || expression.module_id != input.module_id
            || !task269ct_role_matches(
                &expression.site,
                TypedNodeId::new(index),
                "source.type.expression",
            )
            || expression.source_range != task269gct_range(input.source_id, start, end)
            || expression.spelling != "set"
            || !task269ct_role_matches(
                &expression.head_site,
                TypedNodeId::new(index),
                "source.type.head",
            )
            || expression.head_range != task269gct_range(input.source_id, start, end)
            || expression.head_spelling != "set"
            || expression.form != SourceTypeApplicationForm::Bare
            || expression.head != SourceTypeHead::BuiltinSet
            || expression.recovery != NodeRecoveryState::Normal
        {
            return false;
        }
    }
    true
}

fn exact_task269gct_source_type(
    handoff: &SourceTypeApplicationHandoff,
    source_id: SourceId,
    module_id: &ModuleId,
) -> bool {
    if handoff.source_id() != source_id
        || handoff.module_id() != module_id
        || handoff.applications().len() != 2
        || handoff.expressions().len() != 2
        || !handoff.arguments().is_empty()
        || !handoff.definition_returns().is_empty()
        || !handoff.mode_rhs().is_empty()
        || !handoff.structure_members().is_empty()
    {
        return false;
    }
    for (index, (start, end)) in [(14, 17), (90, 93)].into_iter().enumerate() {
        let Some(application) = handoff
            .applications()
            .get(SourceTypeApplicationId::new(index))
        else {
            return false;
        };
        let Some(expression) = handoff
            .expressions()
            .get(SourceTypeExpressionId::new(index))
        else {
            return false;
        };
        if application.id() != SourceTypeApplicationId::new(index)
            || application.binding() != BindingId::new(index)
            || application.source_ordinal() != index
            || application.root() != SourceTypeExpressionId::new(index)
            || expression.id() != SourceTypeExpressionId::new(index)
            || expression.source_id() != source_id
            || expression.module_id() != module_id
            || !task269ct_role_matches(
                expression.site(),
                TypedNodeId::new(index),
                "source.type.expression",
            )
            || expression.source_range() != task269gct_range(source_id, start, end)
            || expression.spelling() != "set"
            || !task269ct_role_matches(
                expression.head_site(),
                TypedNodeId::new(index),
                "source.type.head",
            )
            || expression.head_range() != task269gct_range(source_id, start, end)
            || expression.head_spelling() != "set"
            || expression.form() != SourceTypeApplicationForm::Bare
            || expression.head() != &SourceTypeHead::BuiltinSet
            || expression.recovery() != NodeRecoveryState::Normal
        {
            return false;
        }
    }
    true
}

fn exact_task269gct_arena(source_id: SourceId, arena: &TypedArena) -> bool {
    if arena.len() != 3 || arena.root() != Some(TypedNodeId::new(2)) {
        return false;
    }
    let expected = [
        (
            "source.proof-local.given-condition.reserve-type",
            14,
            17,
            Vec::new(),
        ),
        (
            "source.proof-local.given-condition.type",
            90,
            93,
            Vec::new(),
        ),
        (
            "source.proof-local.given-condition.type-root",
            0,
            133,
            vec![TypedNodeId::new(0), TypedNodeId::new(1)],
        ),
    ];
    expected
        .into_iter()
        .enumerate()
        .all(|(index, (kind, start, end, children))| {
            arena.node(TypedNodeId::new(index)).is_some_and(|node| {
                node.kind.as_str() == kind
                    && node.resolved_node.is_none()
                    && node.anchor == SourceAnchor::Range(task269gct_range(source_id, start, end))
                    && node.children == children
                    && node.typing == TypingState::Unknown
                    && node.recovery == NodeRecoveryState::Normal
                    && node.links == TypedNodeLinks::default()
            })
        })
}

fn task269gct_range(source_id: SourceId, start: usize, end: usize) -> SourceRange {
    SourceRange {
        source_id,
        start,
        end,
    }
}

fn task269gupt_binding_env(
    dependency: &SourceProofLocalGivenUseBindingHandoff,
) -> Result<BindingEnv, SourceProofLocalGivenUseTypeError> {
    let mut bindings = BindingTable::new();
    for (id, binding) in dependency.binding_env().bindings().iter() {
        let inserted = bindings.insert(BindingDraft {
            spelling: binding.spelling.clone(),
            kind: binding.kind,
            identity: binding.identity.clone(),
            owner_context: binding.owner_context,
            declaration_range: binding.declaration_range,
            visible_after_ordinal: binding.visible_after_ordinal,
            type_site: if id == BindingId::new(1) {
                BindingTypeSite::Source(task269gupt_range(dependency.source_id(), 84, 87))
            } else {
                binding.type_site.clone()
            },
            status: binding.status,
            captured: binding.captured.clone(),
            diagnostics: binding.diagnostics.clone(),
            recovery: binding.recovery,
        });
        if inserted != id {
            return Err(SourceProofLocalGivenUseTypeError::InvalidBindingEnvironment);
        }
    }
    let binding_env = BindingEnv::try_new(BindingEnvParts {
        source_id: dependency.source_id(),
        module_id: dependency.module_id().clone(),
        contexts: dependency.binding_env().contexts().clone(),
        bindings,
        diagnostics: dependency.binding_env().diagnostics().clone(),
    })
    .map_err(|_| SourceProofLocalGivenUseTypeError::InvalidBindingEnvironment)?;
    let reserve = binding_env.bindings().get(BindingId::new(0));
    let local = binding_env.bindings().get(BindingId::new(1));
    if binding_env.contexts().len() != 2
        || binding_env.bindings().len() != 2
        || !binding_env.diagnostics().is_empty()
        || reserve.is_none_or(|binding| {
            binding.type_site
                != BindingTypeSite::Source(task269gupt_range(dependency.source_id(), 14, 17))
        })
        || local.is_none_or(|binding| {
            binding.kind != BindingKind::GivenWitness
                || binding.type_site
                    != BindingTypeSite::Source(task269gupt_range(dependency.source_id(), 84, 87))
        })
    {
        return Err(SourceProofLocalGivenUseTypeError::InvalidBindingEnvironment);
    }
    Ok(binding_env)
}

fn exact_task269gupt_input(input: &SourceTypeHandoffInput) -> bool {
    if input.applications.len() != 2 || input.expressions.len() != 2 || !input.arguments.is_empty()
    {
        return false;
    }
    for (index, (start, end)) in [(14, 17), (84, 87)].into_iter().enumerate() {
        let application = &input.applications[index];
        let expression = &input.expressions[index];
        if application.binding != BindingId::new(index)
            || application.source_ordinal != index
            || application.root != SourceTypeExpressionId::new(index)
            || expression.source_id != input.source_id
            || expression.module_id != input.module_id
            || !task269ct_role_matches(
                &expression.site,
                TypedNodeId::new(index),
                "source.type.expression",
            )
            || expression.source_range != task269gupt_range(input.source_id, start, end)
            || expression.spelling != "set"
            || !task269ct_role_matches(
                &expression.head_site,
                TypedNodeId::new(index),
                "source.type.head",
            )
            || expression.head_range != task269gupt_range(input.source_id, start, end)
            || expression.head_spelling != "set"
            || expression.form != SourceTypeApplicationForm::Bare
            || expression.head != SourceTypeHead::BuiltinSet
            || expression.recovery != NodeRecoveryState::Normal
        {
            return false;
        }
    }
    true
}

fn exact_task269gupt_source_type(
    handoff: &SourceTypeApplicationHandoff,
    source_id: SourceId,
    module_id: &ModuleId,
) -> bool {
    if handoff.source_id() != source_id
        || handoff.module_id() != module_id
        || handoff.applications().len() != 2
        || handoff.expressions().len() != 2
        || !handoff.arguments().is_empty()
        || !handoff.definition_returns().is_empty()
        || !handoff.mode_rhs().is_empty()
        || !handoff.structure_members().is_empty()
    {
        return false;
    }
    for (index, (start, end)) in [(14, 17), (84, 87)].into_iter().enumerate() {
        let Some(application) = handoff
            .applications()
            .get(SourceTypeApplicationId::new(index))
        else {
            return false;
        };
        let Some(expression) = handoff
            .expressions()
            .get(SourceTypeExpressionId::new(index))
        else {
            return false;
        };
        if application.id() != SourceTypeApplicationId::new(index)
            || application.binding() != BindingId::new(index)
            || application.source_ordinal() != index
            || application.root() != SourceTypeExpressionId::new(index)
            || expression.id() != SourceTypeExpressionId::new(index)
            || expression.source_id() != source_id
            || expression.module_id() != module_id
            || !task269ct_role_matches(
                expression.site(),
                TypedNodeId::new(index),
                "source.type.expression",
            )
            || expression.source_range() != task269gupt_range(source_id, start, end)
            || expression.spelling() != "set"
            || !task269ct_role_matches(
                expression.head_site(),
                TypedNodeId::new(index),
                "source.type.head",
            )
            || expression.head_range() != task269gupt_range(source_id, start, end)
            || expression.head_spelling() != "set"
            || expression.form() != SourceTypeApplicationForm::Bare
            || expression.head() != &SourceTypeHead::BuiltinSet
            || expression.recovery() != NodeRecoveryState::Normal
        {
            return false;
        }
    }
    true
}

fn exact_task269gupt_arena(source_id: SourceId, arena: &TypedArena) -> bool {
    if arena.len() != 3 || arena.root() != Some(TypedNodeId::new(2)) {
        return false;
    }
    let expected = [
        (
            "source.proof-local.given-use.reserve-type",
            14,
            17,
            Vec::new(),
        ),
        ("source.proof-local.given-use.type", 84, 87, Vec::new()),
        (
            "source.proof-local.given-use.type-root",
            0,
            127,
            vec![TypedNodeId::new(0), TypedNodeId::new(1)],
        ),
    ];
    expected
        .into_iter()
        .enumerate()
        .all(|(index, (kind, start, end, children))| {
            arena.node(TypedNodeId::new(index)).is_some_and(|node| {
                node.kind.as_str() == kind
                    && node.resolved_node.is_none()
                    && node.anchor == SourceAnchor::Range(task269gupt_range(source_id, start, end))
                    && node.children == children
                    && node.typing == TypingState::Unknown
                    && node.recovery == NodeRecoveryState::Normal
                    && node.links == TypedNodeLinks::default()
            })
        })
}

fn task269gupt_range(source_id: SourceId, start: usize, end: usize) -> SourceRange {
    SourceRange {
        source_id,
        start,
        end,
    }
}

fn validate_input(
    input: &SourceTypeHandoffInput,
    bindings: &BindingEnv,
    symbols: &SymbolEnv,
    arena: &TypedArena,
    binding_profile: SourceTypeBindingProfile,
) -> Result<(), SourceTypeError> {
    if input.applications.is_empty() {
        return Err(SourceTypeError::EmptyApplications);
    }
    if input.expressions.is_empty() {
        return Err(SourceTypeError::EmptyExpressions);
    }
    if bindings.source_id() != input.source_id
        || bindings.module_id() != &input.module_id
        || symbols.module_id() != &input.module_id
    {
        return Err(SourceTypeError::EnvironmentMismatch);
    }
    if bindings.bindings().len() != input.applications.len() {
        return Err(SourceTypeError::BindingCardinalityMismatch);
    }

    let mut sites = BTreeSet::new();
    for (index, expression) in input.expressions.iter().enumerate() {
        let id = SourceTypeExpressionId::new(index);
        validate_expression(input, id, expression, symbols, arena)?;
        if !sites.insert(expression.site.clone()) || !sites.insert(expression.head_site.clone()) {
            return Err(SourceTypeError::DuplicateSite);
        }
    }

    let roots = validate_applications(input, bindings, binding_profile)?;
    let validated_arguments = validate_arguments(input, arena, &mut sites)?;
    validate_graph(
        input,
        &roots,
        &validated_arguments.parents,
        &validated_arguments.children,
    )?;
    validate_forms(input)?;
    validate_sibling_ranges(input, &validated_arguments.spans)?;
    Ok(())
}

fn validate_applications(
    input: &SourceTypeHandoffInput,
    bindings: &BindingEnv,
    binding_profile: SourceTypeBindingProfile,
) -> Result<BTreeSet<SourceTypeExpressionId>, SourceTypeError> {
    if !bindings.diagnostics().is_empty() {
        return Err(SourceTypeError::BindingCardinalityMismatch);
    }

    let mut roots = BTreeSet::new();
    let mut previous_root = None;
    let mut previous_range = None;
    for (index, application) in input.applications.iter().enumerate() {
        let id = SourceTypeApplicationId::new(index);
        let Some(root) = input.expressions.get(application.root.index()) else {
            return Err(SourceTypeError::InvalidApplication { application: id });
        };
        if application.source_ordinal != index
            || application.binding != BindingId::new(index)
            || previous_root.is_some_and(|previous| application.root <= previous)
            || !roots.insert(application.root)
        {
            return Err(SourceTypeError::InvalidApplication { application: id });
        }
        if previous_range.is_some_and(|range: SourceRange| range.end > root.source_range.start) {
            return Err(SourceTypeError::OverlappingApplications { application: id });
        }
        let Some(binding) = bindings.bindings().get(application.binding) else {
            return Err(SourceTypeError::InvalidBinding { application: id });
        };
        let Some(context) = bindings.contexts().get(binding.owner_context) else {
            return Err(SourceTypeError::InvalidBinding { application: id });
        };
        let identity_matches = match (&binding.kind, &binding.identity, binding.status) {
            (
                BindingKind::ReservedVariable,
                BinderIdentity::ReservedVariable {
                    spelling,
                    declaration_range,
                },
                BindingStatus::Reserved,
            ) => {
                spelling == &binding.spelling
                    && declaration_range == &binding.declaration_range
                    && matches!(context.owner, BindingContextOwner::Module)
                    && context.layer == BindingContextLayer::Module
            }
            (
                BindingKind::DefinitionParameter,
                BinderIdentity::ResolverLocal {
                    scope,
                    ordinal,
                    declaration_range,
                },
                BindingStatus::Active,
            ) => {
                *ordinal == binding.visible_after_ordinal
                    && declaration_range == &binding.declaration_range
                    && matches!(context.owner, BindingContextOwner::DeclarationShell(_))
                    && context.layer == BindingContextLayer::Declaration
                    && context.parent.is_some()
                    && context.lexical_scope.as_ref() == Some(scope)
            }
            (
                BindingKind::LetBinding,
                BinderIdentity::ResolverLocal {
                    scope,
                    ordinal,
                    declaration_range,
                },
                BindingStatus::Active,
            ) if binding_profile == SourceTypeBindingProfile::ProofLocalLet => {
                *ordinal == binding.visible_after_ordinal
                    && declaration_range == &binding.declaration_range
                    && matches!(context.owner, BindingContextOwner::SourceStatement { .. })
                    && context.layer == BindingContextLayer::Proof
                    && context.parent.is_some()
                    && context.lexical_scope.as_ref() == Some(scope)
            }
            (
                BindingKind::GivenWitness,
                BinderIdentity::ResolverLocal {
                    scope,
                    ordinal,
                    declaration_range,
                },
                BindingStatus::Active,
            ) if binding_profile == SourceTypeBindingProfile::ProofLocalGiven => {
                *ordinal == binding.visible_after_ordinal
                    && declaration_range == &binding.declaration_range
                    && matches!(context.owner, BindingContextOwner::SourceStatement { .. })
                    && context.layer == BindingContextLayer::Proof
                    && context.parent.is_some()
                    && context.lexical_scope.as_ref() == Some(scope)
            }
            _ => false,
        };
        if binding.id != application.binding
            || binding.visible_after_ordinal != application.source_ordinal
            || binding.recovery != BindingRecoveryState::Normal
            || !binding.diagnostics.is_empty()
            || !binding.captured.identities().is_empty()
            || !identity_matches
            || context.recovery != BindingContextRecovery::Normal
            || !context.bindings.contains(&application.binding)
            || !context.visible_bindings.contains(&application.binding)
            || binding.declaration_range.source_id != input.source_id
            || binding.declaration_range.start >= binding.declaration_range.end
            || binding.type_site != BindingTypeSite::Source(root.source_range)
        {
            return Err(SourceTypeError::InvalidBinding { application: id });
        }
        previous_root = Some(application.root);
        previous_range = Some(root.source_range);
    }
    Ok(roots)
}

fn validate_expression(
    input: &SourceTypeHandoffInput,
    id: SourceTypeExpressionId,
    expression: &SourceTypeExpressionInput,
    symbols: &SymbolEnv,
    arena: &TypedArena,
) -> Result<(), SourceTypeError> {
    if expression.source_id != input.source_id
        || expression.module_id != input.module_id
        || !valid_range(input.source_id, expression.source_range)
        || !valid_range(input.source_id, expression.head_range)
        || !range_contains(expression.source_range, expression.head_range)
        || expression.spelling.trim().is_empty()
        || expression.head_spelling.trim().is_empty()
        || expression.site == expression.head_site
    {
        return Err(SourceTypeError::InvalidExpression { expression: id });
    }
    validate_arena_site(
        id,
        &expression.site,
        expression.source_range,
        expression.recovery,
        arena,
    )?;
    validate_arena_site(
        id,
        &expression.head_site,
        expression.head_range,
        expression.recovery,
        arena,
    )?;
    match &expression.head {
        SourceTypeHead::BuiltinSet if expression.head_spelling == "set" => Ok(()),
        SourceTypeHead::BuiltinObject if expression.head_spelling == "object" => Ok(()),
        SourceTypeHead::BuiltinSet | SourceTypeHead::BuiltinObject => {
            Err(SourceTypeError::InvalidHead { expression: id })
        }
        SourceTypeHead::Symbol {
            symbol,
            contribution,
        } => validate_symbol_head(input, id, expression, symbol, *contribution, symbols),
    }
}

fn validate_symbol_head(
    input: &SourceTypeHandoffInput,
    id: SourceTypeExpressionId,
    expression: &SourceTypeExpressionInput,
    symbol: &SymbolId,
    contribution_id: SourceContributionId,
    symbols: &SymbolEnv,
) -> Result<(), SourceTypeError> {
    let invalid = || SourceTypeError::InvalidSymbolHead { expression: id };
    let entry = symbols.symbols().get(symbol).ok_or_else(invalid)?;
    let contribution = symbols
        .contributions()
        .get(contribution_id)
        .ok_or_else(invalid)?;
    if entry.contribution() != contribution_id
        || !matches!(entry.kind(), SymbolKind::Mode | SymbolKind::Structure)
        || !symbol_spelling_matches_form(
            entry.primary_spelling(),
            &expression.head_spelling,
            expression.form,
        )
        || entry.namespace().as_str() != input.module_id.path().as_str()
        || contribution.module() != symbol.module()
        || !contribution.effects().symbols().contains(symbol)
        || entry.origin().is_recovered()
    {
        return Err(invalid());
    }

    if symbol.module() == &input.module_id {
        let origin_range = source_range(entry.origin().anchor()).ok_or_else(invalid)?;
        if contribution.module() != &input.module_id
            || !matches!(
                contribution.kind(),
                ContributionKind::LocalSource { source_id } if *source_id == input.source_id
            )
            || entry.origin().source_id() != input.source_id
            || entry.origin().module_id() != &input.module_id
            || entry.origin().import_edge().is_some()
            || !valid_range(input.source_id, origin_range)
            || origin_range.end > expression.head_range.start
        {
            return Err(invalid());
        }
    } else {
        let contribution_range = source_range(contribution.anchor()).ok_or_else(invalid)?;
        let import_is_authenticated = contribution.effects().imports().iter().any(|import| {
            symbols
                .imports()
                .get(*import)
                .and_then(|entry| entry.module())
                == Some(symbol.module())
        });
        if !matches!(
            contribution.kind(),
            ContributionKind::ImportedSource { source_id } if *source_id == input.source_id
        ) || entry.visibility() != Visibility::Public
            || !matches!(
                entry.export_status(),
                ExportStatus::Exported | ExportStatus::ReExported
            )
            || !valid_range(input.source_id, contribution_range)
            || contribution_range.end > expression.head_range.start
            || entry.origin().module_id() != symbol.module()
            || !import_is_authenticated
        {
            return Err(invalid());
        }
    }
    Ok(())
}

fn symbol_spelling_matches_form(
    primary: &str,
    head: &str,
    form: SourceTypeApplicationForm,
) -> bool {
    match form {
        SourceTypeApplicationForm::Bare => primary == head,
        SourceTypeApplicationForm::Of => primary
            .strip_prefix(head)
            .is_some_and(|suffix| suffix.starts_with(" of ") && suffix.len() > " of ".len()),
        SourceTypeApplicationForm::Over => primary
            .strip_prefix(head)
            .is_some_and(|suffix| suffix.starts_with(" over ") && suffix.len() > " over ".len()),
        SourceTypeApplicationForm::Bracket => primary.strip_prefix(head).is_some_and(|suffix| {
            suffix.starts_with(" [ ") && suffix.ends_with(" ]") && suffix.len() > " [  ]".len()
        }),
    }
}

type ArgumentSpans = Vec<Vec<(usize, SourceRange)>>;

struct ValidatedArguments {
    parents: Vec<Option<SourceTypeExpressionId>>,
    children: Vec<Vec<SourceTypeExpressionId>>,
    spans: ArgumentSpans,
}

fn validate_arguments(
    input: &SourceTypeHandoffInput,
    arena: &TypedArena,
    sites: &mut BTreeSet<TypedSiteRef>,
) -> Result<ValidatedArguments, SourceTypeError> {
    let mut parents = vec![None; input.expressions.len()];
    let mut children = vec![Vec::new(); input.expressions.len()];
    let mut spans = vec![Vec::new(); input.expressions.len()];
    let mut expected_parent = 0;
    let mut expected_ordinal = 0;
    for (index, argument) in input.arguments.iter().enumerate() {
        let id = SourceTypeArgumentId::new(index);
        let Some(parent) = input.expressions.get(argument.parent.index()) else {
            return Err(SourceTypeError::InvalidArgument { argument: id });
        };
        if argument.parent.index() < expected_parent {
            return Err(SourceTypeError::ReorderedArgument { argument: id });
        }
        if argument.parent.index() > expected_parent {
            expected_parent = argument.parent.index();
            expected_ordinal = 0;
        }
        if argument.ordinal != expected_ordinal {
            return Err(SourceTypeError::ReorderedArgument { argument: id });
        }
        expected_ordinal += 1;
        let span = match &argument.argument {
            SourceTypeArgument::TermSite {
                site,
                source_range,
                spelling,
                recovery,
                provenance,
            } => {
                validate_source_argument(
                    input,
                    id,
                    argument,
                    site,
                    *source_range,
                    spelling,
                    *recovery,
                    provenance,
                    arena,
                    sites,
                )?;
                *source_range
            }
            SourceTypeArgument::TypeSite { expression } => {
                add_child(
                    input,
                    id,
                    argument.parent,
                    *expression,
                    &mut parents,
                    &mut children,
                )?;
                input.expressions[expression.index()].source_range
            }
            SourceTypeArgument::QuaSite {
                site,
                source_range,
                spelling,
                recovery,
                provenance,
                radix,
            } => {
                validate_source_argument(
                    input,
                    id,
                    argument,
                    site,
                    *source_range,
                    spelling,
                    *recovery,
                    provenance,
                    arena,
                    sites,
                )?;
                if radix.is_empty() {
                    return Err(SourceTypeError::InvalidArgument { argument: id });
                }
                let mut span = *source_range;
                let mut unique = BTreeSet::new();
                for expression in radix {
                    if !unique.insert(*expression) {
                        return Err(SourceTypeError::DuplicateChild {
                            argument: id,
                            child: *expression,
                        });
                    }
                    add_child(
                        input,
                        id,
                        argument.parent,
                        *expression,
                        &mut parents,
                        &mut children,
                    )?;
                    let child_range = input.expressions[expression.index()].source_range;
                    if span.end > child_range.start {
                        return Err(SourceTypeError::OverlappingSiblings {
                            parent: argument.parent,
                        });
                    }
                    span.end = child_range.end;
                }
                span
            }
        };
        if !range_contains(parent.source_range, span) {
            return Err(SourceTypeError::InvalidArgument { argument: id });
        }
        spans[argument.parent.index()].push((argument.ordinal, span));
    }
    Ok(ValidatedArguments {
        parents,
        children,
        spans,
    })
}

#[allow(clippy::too_many_arguments)] // Rationale: keep every source-site invariant explicit at the validation boundary.
fn validate_source_argument(
    input: &SourceTypeHandoffInput,
    id: SourceTypeArgumentId,
    argument: &SourceTypeArgumentInput,
    site: &TypedSiteRef,
    range: SourceRange,
    spelling: &str,
    recovery: NodeRecoveryState,
    provenance: &SemanticOrigin,
    arena: &TypedArena,
    sites: &mut BTreeSet<TypedSiteRef>,
) -> Result<(), SourceTypeError> {
    if !valid_range(input.source_id, range)
        || spelling.trim().is_empty()
        || !sites.insert(site.clone())
    {
        return Err(SourceTypeError::InvalidArgument { argument: id });
    }
    validate_argument_arena_site(id, site, range, recovery, arena)?;
    let parent = u32::try_from(argument.parent.index())
        .map_err(|_| SourceTypeError::InvalidProvenance { argument: id })?;
    let ordinal = u32::try_from(argument.ordinal)
        .map_err(|_| SourceTypeError::InvalidProvenance { argument: id })?;
    if provenance.source_id() != input.source_id
        || provenance.module_id() != &input.module_id
        || provenance.anchor() != &SourceAnchor::Range(range)
        || provenance.structural_path() != [parent, ordinal]
        || provenance.import_edge().is_some()
        || provenance.is_recovered() != !matches!(recovery, NodeRecoveryState::Normal)
    {
        return Err(SourceTypeError::InvalidProvenance { argument: id });
    }
    Ok(())
}

fn add_child(
    input: &SourceTypeHandoffInput,
    argument: SourceTypeArgumentId,
    parent: SourceTypeExpressionId,
    child: SourceTypeExpressionId,
    parents: &mut [Option<SourceTypeExpressionId>],
    children: &mut [Vec<SourceTypeExpressionId>],
) -> Result<(), SourceTypeError> {
    if input.expressions.get(child.index()).is_none() {
        return Err(SourceTypeError::DanglingChild { argument, child });
    }
    if children[parent.index()].contains(&child) {
        return Err(SourceTypeError::DuplicateChild { argument, child });
    }
    if parents[child.index()].replace(parent).is_some() {
        return Err(SourceTypeError::MultipleParents { child });
    }
    children[parent.index()].push(child);
    Ok(())
}

fn validate_graph(
    input: &SourceTypeHandoffInput,
    roots: &BTreeSet<SourceTypeExpressionId>,
    parents: &[Option<SourceTypeExpressionId>],
    children: &[Vec<SourceTypeExpressionId>],
) -> Result<(), SourceTypeError> {
    for root in roots {
        if parents[root.index()].is_some() {
            return Err(SourceTypeError::RootHasParent { root: *root });
        }
    }
    validate_acyclic(children)?;
    for (parent_index, child_ids) in children.iter().enumerate() {
        let parent = SourceTypeExpressionId::new(parent_index);
        for child in child_ids {
            if parent >= *child {
                return Err(SourceTypeError::ForwardParent {
                    parent,
                    child: *child,
                });
            }
            if !range_contains(
                input.expressions[parent.index()].source_range,
                input.expressions[child.index()].source_range,
            ) {
                return Err(SourceTypeError::ChildOutsideParent {
                    parent,
                    child: *child,
                });
            }
        }
    }
    let mut reachable = vec![false; input.expressions.len()];
    let mut stack = roots.iter().copied().collect::<Vec<_>>();
    while let Some(expression) = stack.pop() {
        if std::mem::replace(&mut reachable[expression.index()], true) {
            continue;
        }
        stack.extend(children[expression.index()].iter().copied());
    }
    for (index, is_reachable) in reachable.into_iter().enumerate() {
        if !is_reachable {
            return Err(SourceTypeError::UnreachableExpression {
                expression: SourceTypeExpressionId::new(index),
            });
        }
    }
    Ok(())
}

fn validate_acyclic(children: &[Vec<SourceTypeExpressionId>]) -> Result<(), SourceTypeError> {
    let mut states = vec![0_u8; children.len()];
    for start in 0..children.len() {
        if states[start] != 0 {
            continue;
        }
        states[start] = 1;
        let mut stack = vec![(SourceTypeExpressionId::new(start), 0_usize)];
        while let Some((expression, next_child)) = stack.last_mut() {
            let Some(child) = children[expression.index()].get(*next_child).copied() else {
                states[expression.index()] = 2;
                stack.pop();
                continue;
            };
            *next_child += 1;
            match states[child.index()] {
                0 => {
                    states[child.index()] = 1;
                    stack.push((child, 0));
                }
                1 => return Err(SourceTypeError::Cycle { expression: child }),
                2 => {}
                _ => unreachable!("source-type traversal state is internal"),
            }
        }
    }
    Ok(())
}

fn validate_forms(input: &SourceTypeHandoffInput) -> Result<(), SourceTypeError> {
    let mut arguments = vec![Vec::new(); input.expressions.len()];
    for argument in &input.arguments {
        arguments[argument.parent.index()].push(&argument.argument);
    }
    for (index, expression) in input.expressions.iter().enumerate() {
        let valid = match expression.form {
            SourceTypeApplicationForm::Bare => arguments[index].is_empty(),
            SourceTypeApplicationForm::Of | SourceTypeApplicationForm::Over => {
                !arguments[index].is_empty()
                    && arguments[index]
                        .iter()
                        .all(|argument| matches!(argument, SourceTypeArgument::TermSite { .. }))
            }
            SourceTypeApplicationForm::Bracket => {
                !arguments[index].is_empty()
                    && arguments[index].iter().all(|argument| {
                        matches!(
                            argument,
                            SourceTypeArgument::TypeSite { .. }
                                | SourceTypeArgument::QuaSite { .. }
                        )
                    })
            }
        };
        if !valid {
            return Err(SourceTypeError::WrongApplicationForm {
                expression: SourceTypeExpressionId::new(index),
            });
        }
    }
    Ok(())
}

fn validate_sibling_ranges(
    input: &SourceTypeHandoffInput,
    spans: &ArgumentSpans,
) -> Result<(), SourceTypeError> {
    for (parent_index, ranges) in spans.iter().enumerate() {
        let mut previous = None;
        for (ordinal, range) in ranges {
            if previous.is_some_and(|previous: SourceRange| previous.end > range.start) {
                return Err(SourceTypeError::OverlappingSiblings {
                    parent: SourceTypeExpressionId::new(parent_index),
                });
            }
            if *ordinal >= input.arguments.len() {
                return Err(SourceTypeError::OverlappingSiblings {
                    parent: SourceTypeExpressionId::new(parent_index),
                });
            }
            previous = Some(*range);
        }
    }
    Ok(())
}

fn validate_arena_site(
    expression: SourceTypeExpressionId,
    site: &TypedSiteRef,
    range: SourceRange,
    recovery: NodeRecoveryState,
    arena: &TypedArena,
) -> Result<(), SourceTypeError> {
    let Some(node) = arena.node(site.node()) else {
        return Err(SourceTypeError::InvalidExpressionSite { expression });
    };
    let Some(anchor) = source_range(&node.anchor) else {
        return Err(SourceTypeError::InvalidExpressionSite { expression });
    };
    if node.recovery != recovery || !range_contains(anchor, range) {
        return Err(SourceTypeError::InvalidExpressionSite { expression });
    }
    Ok(())
}

fn validate_argument_arena_site(
    argument: SourceTypeArgumentId,
    site: &TypedSiteRef,
    range: SourceRange,
    recovery: NodeRecoveryState,
    arena: &TypedArena,
) -> Result<(), SourceTypeError> {
    let Some(node) = arena.node(site.node()) else {
        return Err(SourceTypeError::InvalidArgumentSite { argument });
    };
    let Some(anchor) = source_range(&node.anchor) else {
        return Err(SourceTypeError::InvalidArgumentSite { argument });
    };
    if node.recovery != recovery || !range_contains(anchor, range) {
        return Err(SourceTypeError::InvalidArgumentSite { argument });
    }
    Ok(())
}

fn valid_range(source_id: SourceId, range: SourceRange) -> bool {
    range.source_id == source_id && range.start < range.end
}

fn range_contains(parent: SourceRange, child: SourceRange) -> bool {
    parent.source_id == child.source_id && parent.start <= child.start && child.end <= parent.end
}

fn source_range(anchor: &SourceAnchor) -> Option<SourceRange> {
    match anchor {
        SourceAnchor::Range(range) => Some(*range),
        SourceAnchor::Point { .. } | SourceAnchor::Generated(_) | _ => None,
    }
}

fn form_key(form: SourceTypeApplicationForm) -> &'static str {
    match form {
        SourceTypeApplicationForm::Bare => "bare",
        SourceTypeApplicationForm::Of => "of",
        SourceTypeApplicationForm::Over => "over",
        SourceTypeApplicationForm::Bracket => "bracket",
    }
}

fn recovery_key(recovery: NodeRecoveryState) -> &'static str {
    match recovery {
        NodeRecoveryState::Normal => "normal",
        NodeRecoveryState::Recovered => "recovered",
        NodeRecoveryState::Degraded => "degraded",
    }
}

fn write_site(output: &mut String, site: &TypedSiteRef) {
    match site {
        TypedSiteRef::Node(node) => {
            let _ = write!(output, "node:{}", node.index());
        }
        TypedSiteRef::Role { node, role } => {
            let _ = write!(output, "node:{}:role:{:?}", node.index(), role.as_str());
        }
    }
}

fn write_definition_site(output: &mut String, site: &TypedSiteRef) {
    match site {
        TypedSiteRef::Node(node) => {
            let _ = write!(output, "node#{}", node.index());
        }
        TypedSiteRef::Role { node, role } => {
            let _ = write!(output, "node#{}:role:{:?}", node.index(), role.as_str());
        }
    }
}

fn write_head(output: &mut String, head: &SourceTypeHead) {
    match head {
        SourceTypeHead::BuiltinSet => output.push_str("builtin:set"),
        SourceTypeHead::BuiltinObject => output.push_str("builtin:object"),
        SourceTypeHead::Symbol {
            symbol,
            contribution,
        } => {
            let _ = write!(
                output,
                "symbol:{}:contribution:{}",
                symbol.fqn().as_str(),
                contribution.index()
            );
        }
    }
}

fn write_argument(output: &mut String, argument: &SourceTypeArgument) {
    match argument {
        SourceTypeArgument::TermSite {
            site,
            source_range,
            spelling,
            recovery,
            ..
        } => {
            output.push_str("term site=");
            write_site(output, site);
            let _ = write!(
                output,
                " range={}..{} recovery={} spelling={:?}",
                source_range.start,
                source_range.end,
                recovery_key(*recovery),
                spelling,
            );
        }
        SourceTypeArgument::TypeSite { expression } => {
            let _ = write!(output, "type expression={}", expression.index());
        }
        SourceTypeArgument::QuaSite {
            site,
            source_range,
            spelling,
            recovery,
            radix,
            ..
        } => {
            output.push_str("qua site=");
            write_site(output, site);
            let _ = write!(
                output,
                " range={}..{} recovery={} spelling={:?} radix=",
                source_range.start,
                source_range.end,
                recovery_key(*recovery),
                spelling,
            );
            for (index, expression) in radix.iter().enumerate() {
                if index > 0 {
                    output.push(',');
                }
                let _ = write!(output, "{}", expression.index());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        binding_env::{
            BindingContextDraft, BindingContextId, BindingContextTable, BindingDiagnosticClass,
            BindingDiagnosticDraft, BindingDiagnosticRecovery, BindingDiagnosticSeverity,
            BindingDiagnosticTable, BindingDraft, BindingEnvParts, BindingTable,
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
            ExprId, ExpressionMetadataInput, ResolvedNodeKindHint, ResolvedNodeKindHintKind,
            ResolvedTypedAst, ResolvedTypedAstError, ResolvedTypedAstInputs, SourceNodeRole,
        },
        source_proof_local_declaration::{
            SourceProofLocalGivenBindingHandoffInput, SourceProofLocalGivenBindingProducer,
            SourceProofLocalGivenBindingRecovery,
            SourceProofLocalGivenConditionBindingHandoffInput,
            SourceProofLocalGivenConditionBindingProducer,
            SourceProofLocalGivenUseBindingHandoffInput, SourceProofLocalGivenUseBindingProducer,
            SourceProofLocalLetBindingHandoffInput, SourceProofLocalLetBindingProducer,
            SourceProofLocalLetBindingRecovery,
        },
        source_term::{
            SourcePrimaryTermHandoffInput, SourcePrimaryTermId, SourcePrimaryTermInput,
            SourcePrimaryTermKind, SourcePrimaryTermRecovery, SourcePrimaryTermReferenceInput,
            SourcePrimaryTermReferenceRole, SourcePrimaryTermRole,
            SourceProofLocalGivenUseTermProducer,
        },
        type_checker::{SourceReserveBindingInput, SourceReserveDeclarationBridge, TypeHeadInput},
        typed_ast::{
            CoercionTable, InitialObligationTable, LocalTypeContextTable, TypeDiagnosticTable,
            TypeFactTable, TypeRole, TypeTable, TypedArenaBuilder, TypedAst, TypedAstError,
            TypedAstParts, TypedNode, TypedNodeId,
        },
    };
    use mizar_resolve::{
        env::{
            ContributionKind, DefinitionIndex, DefinitionKind, DefinitionShell, NamespacePath,
            SourceContributionIndex, SymbolEntry, SymbolEnvIndexes,
        },
        names::{LocalTermBinding, LocalTermScope},
        resolved_ast::{FullyQualifiedName, LocalSymbolId, SemanticOrigin, SymbolId},
    };
    use mizar_session::{
        BuildSnapshotId, InMemorySessionIdAllocator, ModulePath, PackageId, SessionIdAllocator as _,
    };

    #[derive(Clone)]
    struct Fixture {
        source: SourceId,
        module: ModuleId,
        input: SourceTypeHandoffInput,
        bindings: BindingEnv,
        symbols: SymbolEnv,
        arena: TypedArena,
    }

    #[derive(Clone)]
    struct Task249RFixture {
        source: SourceId,
        module: ModuleId,
        base: SourceTypeApplicationHandoff,
        arena: TypedArena,
        extension: SourceTypeDefinitionReturnExtensionInput,
    }

    #[derive(Clone)]
    struct Task249MFixture {
        source: SourceId,
        module: ModuleId,
        base: SourceTypeApplicationHandoff,
        arena: TypedArena,
        extension: SourceTypeModeRhsExtensionInput,
    }

    #[derive(Clone)]
    struct Task249SFixture {
        source: SourceId,
        module: ModuleId,
        input: SourceTypeStructureMemberHandoffInput,
        arena: TypedArena,
    }

    #[derive(Clone)]
    struct Task249PiFixture {
        source: SourceId,
        module: ModuleId,
        base: SourceTypeApplicationHandoff,
        extension: SourceTypeStructureMemberHandoffInput,
        arena: TypedArena,
    }

    #[derive(Clone)]
    struct Task269ctFixture {
        source: SourceId,
        module: ModuleId,
        dependency: SourceProofLocalLetBindingHandoff,
        input: SourceTypeHandoffInput,
        symbols: SymbolEnv,
        arena: TypedArena,
    }

    #[derive(Clone)]
    struct Task269gtFixture {
        source: SourceId,
        module: ModuleId,
        dependency: SourceProofLocalGivenBindingHandoff,
        input: SourceTypeHandoffInput,
        symbols: SymbolEnv,
        arena: TypedArena,
    }

    #[derive(Clone)]
    struct Task269gctFixture {
        source: SourceId,
        module: ModuleId,
        dependency: SourceProofLocalGivenConditionBindingHandoff,
        input: SourceTypeHandoffInput,
        symbols: SymbolEnv,
        arena: TypedArena,
    }

    #[derive(Clone)]
    struct Task269guptFixture {
        source: SourceId,
        module: ModuleId,
        dependency: SourceProofLocalGivenUseBindingHandoff,
        input: SourceTypeHandoffInput,
        symbols: SymbolEnv,
        arena: TypedArena,
    }

    #[derive(Clone, Copy, Debug)]
    enum Task249PiProfile {
        Means,
        Equals,
    }

    fn source_id() -> SourceId {
        source_id_for("a7")
    }

    fn other_source_id() -> SourceId {
        let snapshot = BuildSnapshotId::from_published_schema_str(&format!(
            "mizar-session-build-snapshot-v1:{}",
            "a7".repeat(32)
        ))
        .expect("snapshot");
        let allocator = InMemorySessionIdAllocator::new();
        allocator.next_source_id(snapshot).expect("first source");
        allocator.next_source_id(snapshot).expect("second source")
    }

    fn source_id_for(byte: &str) -> SourceId {
        let snapshot = BuildSnapshotId::from_published_schema_str(&format!(
            "mizar-session-build-snapshot-v1:{}",
            byte.repeat(32)
        ))
        .expect("snapshot");
        InMemorySessionIdAllocator::new()
            .next_source_id(snapshot)
            .expect("source")
    }

    fn module(path: &str) -> ModuleId {
        ModuleId::new(PackageId::new("pkg"), ModulePath::new(path))
    }

    fn range(source: SourceId, start: usize, end: usize) -> SourceRange {
        SourceRange {
            source_id: source,
            start,
            end,
        }
    }

    fn role(node: usize, value: &str) -> TypedSiteRef {
        TypedSiteRef::Role {
            node: crate::typed_ast::TypedNodeId::new(node),
            role: TypeRole::new(value),
        }
    }

    fn bare_expression(
        source: SourceId,
        module: &ModuleId,
        node: usize,
        start: usize,
        end: usize,
        head: &str,
    ) -> SourceTypeExpressionInput {
        SourceTypeExpressionInput {
            source_id: source,
            module_id: module.clone(),
            site: role(node, &format!("expression-{node}")),
            source_range: range(source, start, end),
            spelling: head.to_owned(),
            head_site: role(node, &format!("head-{node}")),
            head_range: range(source, start, start + head.len()),
            head_spelling: head.to_owned(),
            form: SourceTypeApplicationForm::Bare,
            head: match head {
                "set" => SourceTypeHead::BuiltinSet,
                "object" => SourceTypeHead::BuiltinObject,
                _ => panic!("test helper only admits builtins"),
            },
            recovery: NodeRecoveryState::Normal,
        }
    }

    fn binding_env(source: SourceId, module: &ModuleId, type_ranges: &[SourceRange]) -> BindingEnv {
        let binding_ids = (0..type_ranges.len())
            .map(BindingId::new)
            .collect::<Vec<_>>();
        let mut contexts = BindingContextTable::new();
        contexts.insert(BindingContextDraft {
            owner: BindingContextOwner::Module,
            parent: None,
            layer: BindingContextLayer::Module,
            lexical_scope: None,
            bindings: binding_ids.clone(),
            visible_bindings: binding_ids,
            recovery: BindingContextRecovery::Normal,
        });
        let mut bindings = BindingTable::new();
        for (index, type_range) in type_ranges.iter().enumerate() {
            let declaration_range = range(source, index * 2, index * 2 + 1);
            let spelling = format!("x{index}");
            bindings.insert(BindingDraft {
                spelling: spelling.clone(),
                kind: BindingKind::ReservedVariable,
                identity: BinderIdentity::ReservedVariable {
                    spelling,
                    declaration_range,
                },
                owner_context: BindingContextId::new(0),
                declaration_range,
                visible_after_ordinal: index,
                type_site: BindingTypeSite::Source(*type_range),
                status: BindingStatus::Reserved,
                captured: CapturedFreeVariables::default(),
                diagnostics: Vec::new(),
                recovery: BindingRecoveryState::Normal,
            });
        }
        BindingEnv::try_new(BindingEnvParts {
            source_id: source,
            module_id: module.clone(),
            contexts,
            bindings,
            diagnostics: BindingDiagnosticTable::new(),
        })
        .expect("binding env")
    }

    fn arena_for(input: &SourceTypeHandoffInput) -> TypedArena {
        let max_node = input
            .expressions
            .iter()
            .flat_map(|expression| [expression.site.node(), expression.head_site.node()])
            .chain(
                input
                    .arguments
                    .iter()
                    .filter_map(|argument| match &argument.argument {
                        SourceTypeArgument::TermSite { site, .. }
                        | SourceTypeArgument::QuaSite { site, .. } => Some(site.node()),
                        SourceTypeArgument::TypeSite { .. } => None,
                    }),
            )
            .map(|node| node.index())
            .max()
            .expect("at least one site");
        let mut anchors = vec![None::<(SourceRange, NodeRecoveryState)>; max_node + 1];
        let mut record = |site: &TypedSiteRef, range: SourceRange, recovery: NodeRecoveryState| {
            let entry = &mut anchors[site.node().index()];
            match entry {
                Some((anchor, existing_recovery)) => {
                    assert_eq!(*existing_recovery, recovery);
                    anchor.start = anchor.start.min(range.start);
                    anchor.end = anchor.end.max(range.end);
                }
                None => *entry = Some((range, recovery)),
            }
        };
        for expression in &input.expressions {
            record(
                &expression.site,
                expression.source_range,
                expression.recovery,
            );
            record(
                &expression.head_site,
                expression.head_range,
                expression.recovery,
            );
        }
        for argument in &input.arguments {
            match &argument.argument {
                SourceTypeArgument::TermSite {
                    site,
                    source_range,
                    recovery,
                    ..
                }
                | SourceTypeArgument::QuaSite {
                    site,
                    source_range,
                    recovery,
                    ..
                } => record(site, *source_range, *recovery),
                SourceTypeArgument::TypeSite { .. } => {}
            }
        }
        TypedArena::try_new(
            None,
            anchors
                .into_iter()
                .enumerate()
                .map(|(index, value)| {
                    let (anchor, recovery) = value.unwrap_or_else(|| {
                        (range(input.source_id, 0, 1), NodeRecoveryState::Normal)
                    });
                    TypedNode::new(
                        format!("source-type-test-{index}"),
                        SourceAnchor::Range(anchor),
                    )
                    .with_recovery(recovery)
                })
                .collect(),
        )
        .expect("arena")
    }

    fn refresh(fixture: &mut Fixture) {
        let root_ranges = fixture
            .input
            .applications
            .iter()
            .map(|application| fixture.input.expressions[application.root.index()].source_range)
            .collect::<Vec<_>>();
        fixture.bindings = binding_env(fixture.source, &fixture.module, &root_ranges);
        fixture.arena = arena_for(&fixture.input);
    }

    fn fixture() -> Fixture {
        let source = source_id();
        let module = module("source.type");
        let mut root = bare_expression(source, &module, 0, 10, 90, "set");
        root.form = SourceTypeApplicationForm::Bracket;
        root.spelling = "set[object, qua x set]".to_owned();
        let expressions = vec![
            root,
            bare_expression(source, &module, 1, 20, 30, "object"),
            bare_expression(source, &module, 2, 50, 60, "set"),
        ];
        let arguments = vec![
            SourceTypeArgumentInput {
                parent: SourceTypeExpressionId::new(0),
                ordinal: 0,
                argument: SourceTypeArgument::TypeSite {
                    expression: SourceTypeExpressionId::new(1),
                },
            },
            SourceTypeArgumentInput {
                parent: SourceTypeExpressionId::new(0),
                ordinal: 1,
                argument: SourceTypeArgument::QuaSite {
                    site: role(3, "qua-0-1"),
                    source_range: range(source, 40, 41),
                    spelling: "x".to_owned(),
                    recovery: NodeRecoveryState::Normal,
                    provenance: SemanticOrigin::new(
                        source,
                        module.clone(),
                        SourceAnchor::Range(range(source, 40, 41)),
                        vec![0, 1],
                    ),
                    radix: vec![SourceTypeExpressionId::new(2)],
                },
            },
        ];
        let input = SourceTypeHandoffInput {
            source_id: source,
            module_id: module.clone(),
            applications: vec![SourceTypeApplicationInput {
                binding: BindingId::new(0),
                source_ordinal: 0,
                root: SourceTypeExpressionId::new(0),
            }],
            expressions,
            arguments,
        };
        let bindings = binding_env(source, &module, &[input.expressions[0].source_range]);
        let symbols = SymbolEnv::new(module.clone(), SymbolEnvIndexes::default());
        let arena = arena_for(&input);
        Fixture {
            source,
            module,
            input,
            bindings,
            symbols,
            arena,
        }
    }

    fn term_fixture(form: SourceTypeApplicationForm) -> Fixture {
        let mut fixture = fixture();
        fixture.input.expressions.truncate(1);
        fixture.input.expressions[0].form = form;
        fixture.input.arguments = vec![SourceTypeArgumentInput {
            parent: SourceTypeExpressionId::new(0),
            ordinal: 0,
            argument: SourceTypeArgument::TermSite {
                site: role(3, "term-0-0"),
                source_range: range(fixture.source, 20, 21),
                spelling: "x".to_owned(),
                recovery: NodeRecoveryState::Normal,
                provenance: SemanticOrigin::new(
                    fixture.source,
                    fixture.module.clone(),
                    SourceAnchor::Range(range(fixture.source, 20, 21)),
                    vec![0, 0],
                ),
            },
        }];
        refresh(&mut fixture);
        fixture
    }

    fn two_root_fixture() -> Fixture {
        let mut fixture = fixture();
        fixture.input.expressions = vec![
            bare_expression(fixture.source, &fixture.module, 0, 10, 20, "set"),
            bare_expression(fixture.source, &fixture.module, 1, 30, 40, "object"),
        ];
        fixture.input.applications = vec![
            SourceTypeApplicationInput {
                binding: BindingId::new(0),
                source_ordinal: 0,
                root: SourceTypeExpressionId::new(0),
            },
            SourceTypeApplicationInput {
                binding: BindingId::new(1),
                source_ordinal: 1,
                root: SourceTypeExpressionId::new(1),
            },
        ];
        fixture.input.arguments.clear();
        refresh(&mut fixture);
        fixture
    }

    fn task_249r_fixture() -> Task249RFixture {
        let source = source_id_for("b8");
        let module = module("task249r.functor_definition");
        let input = SourceTypeHandoffInput {
            source_id: source,
            module_id: module.clone(),
            applications: vec![
                SourceTypeApplicationInput {
                    binding: BindingId::new(0),
                    source_ordinal: 0,
                    root: SourceTypeExpressionId::new(0),
                },
                SourceTypeApplicationInput {
                    binding: BindingId::new(1),
                    source_ordinal: 1,
                    root: SourceTypeExpressionId::new(1),
                },
            ],
            expressions: vec![
                task_249r_expression(source, &module, 63, 62, 22, 25),
                task_249r_expression(source, &module, 67, 66, 38, 41),
            ],
            arguments: Vec::new(),
        };
        let arena = task_249r_arena(source, None);
        let bindings = binding_env(
            source,
            &module,
            &[range(source, 22, 25), range(source, 38, 41)],
        );
        let symbols = SymbolEnv::new(module.clone(), SymbolEnvIndexes::default());
        let base = SourceTypeProducer::build(input, &bindings, &symbols, &arena)
            .expect("exact Task 249 base");
        let extension = SourceTypeDefinitionReturnExtensionInput {
            source_id: source,
            module_id: module.clone(),
            returns: vec![
                SourceTypeDefinitionReturnInput {
                    definition_site: node_site(84),
                    definition_range: range(source, 61, 118),
                    source_ordinal: 0,
                    expression: task_249r_expression(source, &module, 80, 79, 105, 108),
                },
                SourceTypeDefinitionReturnInput {
                    definition_site: node_site(95),
                    definition_range: range(source, 121, 179),
                    source_ordinal: 1,
                    expression: task_249r_expression(source, &module, 87, 86, 163, 166),
                },
            ],
        };
        Task249RFixture {
            source,
            module,
            base,
            arena,
            extension,
        }
    }

    fn task_249m_fixture() -> Task249MFixture {
        let source = source_id_for("d0");
        let module = module("task249m.mode_definition");
        let input = SourceTypeHandoffInput {
            source_id: source,
            module_id: module.clone(),
            applications: vec![
                SourceTypeApplicationInput {
                    binding: BindingId::new(0),
                    source_ordinal: 0,
                    root: SourceTypeExpressionId::new(0),
                },
                SourceTypeApplicationInput {
                    binding: BindingId::new(1),
                    source_ordinal: 1,
                    root: SourceTypeExpressionId::new(1),
                },
            ],
            expressions: vec![
                task_249r_expression(source, &module, 35, 34, 22, 25),
                task_249r_expression(source, &module, 39, 38, 38, 41),
            ],
            arguments: Vec::new(),
        };
        let arena = task_249m_arena(source, None);
        let bindings = binding_env(
            source,
            &module,
            &[range(source, 22, 25), range(source, 38, 41)],
        );
        let symbols = SymbolEnv::new(module.clone(), SymbolEnvIndexes::default());
        let base = SourceTypeProducer::build(input, &bindings, &symbols, &arena)
            .expect("exact Task 249M base");
        let extension = SourceTypeModeRhsExtensionInput {
            source_id: source,
            module_id: module.clone(),
            rhs: vec![SourceTypeModeRhsInput {
                definition_site: node_site(49),
                definition_range: range(source, 45, 135),
                source_ordinal: 0,
                expression: task_249r_expression(source, &module, 44, 43, 95, 98),
            }],
        };
        Task249MFixture {
            source,
            module,
            base,
            arena,
            extension,
        }
    }

    fn task_249s_fixture() -> Task249SFixture {
        let source = source_id_for("e1");
        let module = module("task249s.structure_definition");
        let members = TASK_249S_STRUCTURE_MEMBERS
            .into_iter()
            .enumerate()
            .map(
                |(
                    source_ordinal,
                    (member_node, member_start, member_end, expression_node, head_node, start, end),
                )| SourceTypeStructureMemberInput {
                    member_site: node_site(member_node),
                    member_range: range(source, member_start, member_end),
                    source_ordinal,
                    expression: task_249r_expression(
                        source,
                        &module,
                        expression_node,
                        head_node,
                        start,
                        end,
                    ),
                },
            )
            .collect();
        Task249SFixture {
            source,
            module: module.clone(),
            input: SourceTypeStructureMemberHandoffInput {
                source_id: source,
                module_id: module,
                members,
            },
            arena: task_249s_arena(source, None),
        }
    }

    fn task_249pi_fixture(profile: Task249PiProfile) -> Task249PiFixture {
        let (source, module, parameter_sites, members) = match profile {
            Task249PiProfile::Means => (
                source_id_for("f2"),
                module(
                    "tests.type_elaboration.pass.pass_type_elaboration_property_implementation_means_payload_001",
                ),
                TASK_249PI_MEANS_PARAMETER_SITES,
                TASK_249PI_MEANS_STRUCTURE_MEMBERS,
            ),
            Task249PiProfile::Equals => (
                source_id_for("f3"),
                module(
                    "tests.type_elaboration.pass.pass_type_elaboration_property_implementation_equals_payload_001",
                ),
                TASK_249PI_EQUALS_PARAMETER_SITES,
                TASK_249PI_EQUALS_STRUCTURE_MEMBERS,
            ),
        };
        let (symbols, symbol, contribution) = task_249pi_symbol_env(source, &module, profile);
        let input = SourceTypeHandoffInput {
            source_id: source,
            module_id: module.clone(),
            applications: vec![SourceTypeApplicationInput {
                binding: BindingId::new(0),
                source_ordinal: 0,
                root: SourceTypeExpressionId::new(0),
            }],
            expressions: vec![SourceTypeExpressionInput {
                source_id: source,
                module_id: module.clone(),
                site: node_site(parameter_sites.0),
                source_range: range(source, 130, 144),
                spelling: "Task264Carrier".to_owned(),
                head_site: node_site(parameter_sites.1),
                head_range: range(source, 130, 144),
                head_spelling: "Task264Carrier".to_owned(),
                form: SourceTypeApplicationForm::Bare,
                head: SourceTypeHead::Symbol {
                    symbol,
                    contribution,
                },
                recovery: NodeRecoveryState::Normal,
            }],
            arguments: Vec::new(),
        };
        let bindings = binding_env(source, &module, &[range(source, 130, 144)]);
        let arena = task_249pi_arena(source, profile, None);
        let base = SourceTypeProducer::build(input, &bindings, &symbols, &arena)
            .expect("exact Task 249PI base");
        let extension = SourceTypeStructureMemberHandoffInput {
            source_id: source,
            module_id: module.clone(),
            members: members
                .into_iter()
                .enumerate()
                .map(
                    |(
                        source_ordinal,
                        (
                            member_node,
                            member_start,
                            member_end,
                            expression_node,
                            head_node,
                            start,
                            end,
                        ),
                    )| SourceTypeStructureMemberInput {
                        member_site: node_site(member_node),
                        member_range: range(source, member_start, member_end),
                        source_ordinal,
                        expression: task_249r_expression(
                            source,
                            &module,
                            expression_node,
                            head_node,
                            start,
                            end,
                        ),
                    },
                )
                .collect(),
        };
        Task249PiFixture {
            source,
            module,
            base,
            extension,
            arena,
        }
    }

    fn task_249pi_symbol_env(
        source: SourceId,
        module: &ModuleId,
        profile: Task249PiProfile,
    ) -> (SymbolEnv, SymbolId, SourceContributionId) {
        let mut indexes = SymbolEnvIndexes::default();
        let contribution = indexes.contributions.insert(
            module.clone(),
            ContributionKind::LocalSource { source_id: source },
            SourceAnchor::Range(range(source, 13, 101)),
        );
        let profile_key = match profile {
            Task249PiProfile::Means => "means",
            Task249PiProfile::Equals => "equals",
        };
        let local = format!(
            "contribution=0:namespace={}:owner=definition-block#0/structure-definition#0:shell=structure:kind=structure:name=Task264Carrier:notation=_:arity=_:definition=structure:registration=_:policy=unique:slot={profile_key}",
            module.path().as_str()
        );
        let symbol = SymbolId::new(
            module.clone(),
            LocalSymbolId::new(local.clone()),
            FullyQualifiedName::new(format!(
                "{}::{}::{local}",
                module.package().as_str(),
                module.path().as_str()
            )),
        );
        let entry = SymbolEntry::new(
            symbol.clone(),
            SymbolKind::Structure,
            NamespacePath::new(module.path().as_str()),
            "Task264Carrier",
            SemanticOrigin::new(
                source,
                module.clone(),
                SourceAnchor::Range(range(source, 20, 34)),
                vec![0, 0],
            ),
            contribution,
        );
        indexes.symbols.insert(entry);
        indexes
            .contributions
            .add_symbol(contribution, symbol.clone());
        (
            SymbolEnv::new(module.clone(), indexes),
            symbol,
            contribution,
        )
    }

    fn task_249r_expression(
        source: SourceId,
        module: &ModuleId,
        site: usize,
        head_site: usize,
        start: usize,
        end: usize,
    ) -> SourceTypeExpressionInput {
        SourceTypeExpressionInput {
            source_id: source,
            module_id: module.clone(),
            site: node_site(site),
            source_range: range(source, start, end),
            spelling: "set".to_owned(),
            head_site: node_site(head_site),
            head_range: range(source, start, end),
            head_spelling: "set".to_owned(),
            form: SourceTypeApplicationForm::Bare,
            head: SourceTypeHead::BuiltinSet,
            recovery: NodeRecoveryState::Normal,
        }
    }

    fn node_site(index: usize) -> TypedSiteRef {
        TypedSiteRef::Node(TypedNodeId::new(index))
    }

    fn task_249r_arena(
        source: SourceId,
        mutation: Option<(usize, SourceRange, NodeRecoveryState)>,
    ) -> TypedArena {
        let nodes = (0..=95)
            .map(|index| {
                let source_range = match index {
                    62 | 63 => range(source, 22, 25),
                    66 | 67 => range(source, 38, 41),
                    79 | 80 => range(source, 105, 108),
                    84 => range(source, 61, 118),
                    86 | 87 => range(source, 163, 166),
                    95 => range(source, 121, 179),
                    _ => range(source, 0, 1),
                };
                let (source_range, recovery) = mutation
                    .filter(|(node, _, _)| *node == index)
                    .map(|(_, range, recovery)| (range, recovery))
                    .unwrap_or((source_range, NodeRecoveryState::Normal));
                TypedNode::new(
                    format!("task249r-source-node-{index}"),
                    SourceAnchor::Range(source_range),
                )
                .with_recovery(recovery)
            })
            .collect();
        TypedArena::try_new(None, nodes).expect("Task 249R arena")
    }

    fn task_249m_arena(
        source: SourceId,
        mutation: Option<(usize, SourceRange, NodeRecoveryState)>,
    ) -> TypedArena {
        let nodes = (0..=49)
            .map(|index| {
                let source_range = match index {
                    34 | 35 => range(source, 22, 25),
                    38 | 39 => range(source, 38, 41),
                    43 | 44 => range(source, 95, 98),
                    49 => range(source, 45, 135),
                    _ => range(source, 0, 1),
                };
                let (source_range, recovery) = mutation
                    .filter(|(node, _, _)| *node == index)
                    .map(|(_, range, recovery)| (range, recovery))
                    .unwrap_or((source_range, NodeRecoveryState::Normal));
                TypedNode::new(
                    format!("task249m-source-node-{index}"),
                    SourceAnchor::Range(source_range),
                )
                .with_recovery(recovery)
            })
            .collect();
        TypedArena::try_new(None, nodes).expect("Task 249M arena")
    }

    fn task_249s_arena(
        source: SourceId,
        mutation: Option<(usize, SourceRange, NodeRecoveryState)>,
    ) -> TypedArena {
        let nodes = (0..=64)
            .map(|index| {
                let source_range = match index {
                    51 | 52 => range(source, 59, 62),
                    53 => range(source, 42, 63),
                    54 | 55 => range(source, 87, 90),
                    56 => range(source, 68, 91),
                    59 | 60 => range(source, 151, 154),
                    61 => range(source, 134, 155),
                    62 | 63 => range(source, 179, 182),
                    64 => range(source, 160, 183),
                    _ => range(source, 0, 1),
                };
                let (source_range, recovery) = mutation
                    .filter(|(node, _, _)| *node == index)
                    .map(|(_, range, recovery)| (range, recovery))
                    .unwrap_or((source_range, NodeRecoveryState::Normal));
                TypedNode::new(
                    format!("task249s-source-node-{index}"),
                    SourceAnchor::Range(source_range),
                )
                .with_recovery(recovery)
            })
            .collect();
        TypedArena::try_new(None, nodes).expect("Task 249S arena")
    }

    fn task_249pi_arena(
        source: SourceId,
        profile: Task249PiProfile,
        mutation: Option<(usize, SourceRange, NodeRecoveryState)>,
    ) -> TypedArena {
        let (parameter_sites, members) = match profile {
            Task249PiProfile::Means => (
                TASK_249PI_MEANS_PARAMETER_SITES,
                TASK_249PI_MEANS_STRUCTURE_MEMBERS,
            ),
            Task249PiProfile::Equals => (
                TASK_249PI_EQUALS_PARAMETER_SITES,
                TASK_249PI_EQUALS_STRUCTURE_MEMBERS,
            ),
        };
        let max_node = members
            .iter()
            .flat_map(|(member, _, _, expression, head, _, _)| [*member, *expression, *head])
            .chain([parameter_sites.0, parameter_sites.1])
            .max()
            .expect("Task 249PI sites");
        let nodes = (0..=max_node)
            .map(|index| {
                let source_range = if index == parameter_sites.0 || index == parameter_sites.1 {
                    range(source, 130, 144)
                } else {
                    members
                        .iter()
                        .find_map(
                            |(member, member_start, member_end, expression, head, start, end)| {
                                if index == *member {
                                    Some(range(source, *member_start, *member_end))
                                } else if index == *expression || index == *head {
                                    Some(range(source, *start, *end))
                                } else {
                                    None
                                }
                            },
                        )
                        .unwrap_or_else(|| range(source, 0, 1))
                };
                let (source_range, recovery) = mutation
                    .filter(|(node, _, _)| *node == index)
                    .map(|(_, range, recovery)| (range, recovery))
                    .unwrap_or((source_range, NodeRecoveryState::Normal));
                TypedNode::new(
                    format!("task249pi-source-node-{index}"),
                    SourceAnchor::Range(source_range),
                )
                .with_recovery(recovery)
            })
            .collect();
        TypedArena::try_new(None, nodes).expect("Task 249PI arena")
    }

    fn assemble_task269ct_resolved(
        typed: &TypedAst,
        expressions: Vec<ExpressionMetadataInput>,
        node_hints: Vec<ResolvedNodeKindHint>,
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
            typed_ast: typed,
            cluster_facts: &cluster_facts,
            overload_collection: &collection,
            template_expansion: &expansion,
            viability: &viability,
            specificity: &specificity,
            overload_selection: &selection,
            expressions,
            node_hints,
            statement_semantics: None,
            statement_proofs: None,
        })
    }

    fn assemble_empty_resolved(typed: &TypedAst) -> ResolvedTypedAst {
        assemble_task269ct_resolved(typed, Vec::new(), Vec::new()).expect("empty final assembly")
    }

    fn task_249r_typed_ast(
        source: SourceId,
        module: ModuleId,
        handoff: SourceTypeApplicationHandoff,
        arena: TypedArena,
    ) -> Result<TypedAst, TypedAstError> {
        TypedAst::try_new(TypedAstParts {
            source_id: source,
            module_id: module,
            resolved_root: None,
            source_context: None,
            source_type: Some(handoff),
            source_attribute: None,
            nodes: arena,
            contexts: LocalTypeContextTable::new(),
            types: TypeTable::new(),
            facts: TypeFactTable::new(),
            coercions: CoercionTable::new(),
            initial_obligations: InitialObligationTable::new(),
            diagnostics: TypeDiagnosticTable::new(),
        })
    }

    fn assert_task_249r_extension_error(
        fixture: &Task249RFixture,
        input: SourceTypeDefinitionReturnExtensionInput,
        arena: &TypedArena,
        expected: SourceTypeError,
    ) {
        let baseline = fixture.base.clone();
        assert_eq!(
            SourceTypeDefinitionReturnProducer::extend(&fixture.base, input, arena),
            Err(expected)
        );
        assert_eq!(fixture.base, baseline);
        assert!(fixture.base.definition_returns().is_empty());
    }

    fn assert_task_249m_extension_error(
        fixture: &Task249MFixture,
        input: SourceTypeModeRhsExtensionInput,
        arena: &TypedArena,
        expected: SourceTypeError,
    ) {
        let baseline = fixture.base.clone();
        assert_eq!(
            SourceTypeModeRhsProducer::extend(&fixture.base, input, arena),
            Err(expected)
        );
        assert_eq!(fixture.base, baseline);
        assert!(fixture.base.mode_rhs().is_empty());
    }

    fn assert_task_249s_build_error(
        input: &SourceTypeStructureMemberHandoffInput,
        arena: &TypedArena,
        expected: SourceTypeError,
    ) {
        let baseline = input.clone();
        assert_eq!(
            SourceTypeStructureMemberProducer::build(input.clone(), arena),
            Err(expected)
        );
        assert_eq!(input, &baseline);
    }

    fn assert_task_249pi_extension_error(
        fixture: &Task249PiFixture,
        input: SourceTypeStructureMemberHandoffInput,
        arena: &TypedArena,
        expected: SourceTypeError,
    ) {
        let baseline = fixture.base.clone();
        assert_eq!(
            SourceTypeStructureMemberProducer::extend_property_implementation(
                &fixture.base,
                input,
                arena,
            ),
            Err(expected)
        );
        assert_eq!(fixture.base, baseline);
        assert!(fixture.base.structure_members().is_empty());
    }

    #[derive(Clone, Copy, Debug)]
    enum BindingMutation {
        GlobalDiagnostic,
        Recovery,
        Diagnostic,
        Capture,
        WrongIdentity,
        WrongStatus,
        WrongOrdinal,
        ContextRecovery,
        MissingMembership,
        MissingVisibility,
        EmptyDeclarationRange,
        WrongTypeSite,
    }

    fn binding_env_with_mutation(fixture: &Fixture, mutation: BindingMutation) -> BindingEnv {
        let declaration_range = range(fixture.source, 1, 2);
        let mut diagnostics = BindingDiagnosticTable::new();
        let diagnostic = matches!(
            mutation,
            BindingMutation::GlobalDiagnostic | BindingMutation::Diagnostic
        )
        .then(|| {
            diagnostics.insert(BindingDiagnosticDraft {
                source_range: Some(declaration_range),
                class: BindingDiagnosticClass::ExternalDependencyGap,
                severity: BindingDiagnosticSeverity::Note,
                message_key: "source_type.test.binding_diagnostic".to_owned(),
                recovery: BindingDiagnosticRecovery::Normal,
            })
        });
        let mut binding = BindingDraft {
            spelling: "x0".to_owned(),
            kind: BindingKind::ReservedVariable,
            identity: BinderIdentity::ReservedVariable {
                spelling: "x0".to_owned(),
                declaration_range,
            },
            owner_context: BindingContextId::new(0),
            declaration_range,
            visible_after_ordinal: 0,
            type_site: BindingTypeSite::Source(fixture.input.expressions[0].source_range),
            status: BindingStatus::Reserved,
            captured: CapturedFreeVariables::default(),
            diagnostics: Vec::new(),
            recovery: BindingRecoveryState::Normal,
        };
        let mut context = BindingContextDraft {
            owner: BindingContextOwner::Module,
            parent: None,
            layer: BindingContextLayer::Module,
            lexical_scope: None,
            bindings: vec![BindingId::new(0)],
            visible_bindings: vec![BindingId::new(0)],
            recovery: BindingContextRecovery::Normal,
        };
        match mutation {
            BindingMutation::GlobalDiagnostic => {}
            BindingMutation::Recovery => binding.recovery = BindingRecoveryState::Recovered,
            BindingMutation::Diagnostic => {
                binding.diagnostics = vec![diagnostic.expect("diagnostic")]
            }
            BindingMutation::Capture => {
                binding.captured = CapturedFreeVariables::new(vec![binding.identity.clone()]);
            }
            BindingMutation::WrongIdentity => {
                binding.kind = BindingKind::QuantifierBinder;
                binding.identity = BinderIdentity::ResolverLocal {
                    scope: LocalTermScope::new(vec![0]),
                    ordinal: 0,
                    declaration_range,
                };
                binding.status = BindingStatus::Active;
            }
            BindingMutation::WrongStatus => binding.status = BindingStatus::Active,
            BindingMutation::WrongOrdinal => binding.visible_after_ordinal = 1,
            BindingMutation::ContextRecovery => {
                context.recovery = BindingContextRecovery::Recovered
            }
            BindingMutation::MissingMembership => {
                context.bindings.clear();
                context.visible_bindings.clear();
            }
            BindingMutation::MissingVisibility => context.visible_bindings.clear(),
            BindingMutation::EmptyDeclarationRange => {
                binding.declaration_range = range(fixture.source, 1, 1);
                binding.identity = BinderIdentity::ReservedVariable {
                    spelling: "x0".to_owned(),
                    declaration_range: binding.declaration_range,
                };
            }
            BindingMutation::WrongTypeSite => {
                binding.type_site = BindingTypeSite::Source(range(fixture.source, 10, 19))
            }
        }
        let mut contexts = BindingContextTable::new();
        contexts.insert(context);
        let mut bindings = BindingTable::new();
        bindings.insert(binding);
        BindingEnv::try_new(BindingEnvParts {
            source_id: fixture.source,
            module_id: fixture.module.clone(),
            contexts,
            bindings,
            diagnostics,
        })
        .expect("corrupt source-type binding env must remain structurally valid upstream")
    }

    fn binding_env_with_identity(
        fixture: &Fixture,
        source_id: SourceId,
        module_id: ModuleId,
    ) -> BindingEnv {
        let mut contexts = BindingContextTable::new();
        for (_, context) in fixture.bindings.contexts().iter() {
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
        let mut bindings = BindingTable::new();
        for (_, binding) in fixture.bindings.bindings().iter() {
            let mut declaration_range = binding.declaration_range;
            declaration_range.source_id = source_id;
            let mut identity = binding.identity.clone();
            match &mut identity {
                BinderIdentity::ResolverLocal {
                    declaration_range, ..
                }
                | BinderIdentity::ReservedVariable {
                    declaration_range, ..
                } => declaration_range.source_id = source_id,
                _ => {}
            }
            let mut type_site = binding.type_site.clone();
            if let BindingTypeSite::Source(range) = &mut type_site {
                range.source_id = source_id;
            }
            bindings.insert(BindingDraft {
                spelling: binding.spelling.clone(),
                kind: binding.kind,
                identity,
                owner_context: binding.owner_context,
                declaration_range,
                visible_after_ordinal: binding.visible_after_ordinal,
                type_site,
                status: binding.status,
                captured: binding.captured.clone(),
                diagnostics: binding.diagnostics.clone(),
                recovery: binding.recovery,
            });
        }
        BindingEnv::try_new(BindingEnvParts {
            source_id,
            module_id,
            contexts,
            bindings,
            diagnostics: fixture.bindings.diagnostics().clone(),
        })
        .expect("environment identity corruption must remain structurally valid upstream")
    }

    fn build(fixture: &Fixture) -> Result<SourceTypeApplicationHandoff, SourceTypeError> {
        SourceTypeProducer::build(
            fixture.input.clone(),
            &fixture.bindings,
            &fixture.symbols,
            &fixture.arena,
        )
    }

    fn install_symbol(
        fixture: &mut Fixture,
        imported: bool,
        exported: bool,
        valid_signature: bool,
    ) {
        let symbol_module = if imported {
            module("dependency.types")
        } else {
            fixture.module.clone()
        };
        let mut indexes = SymbolEnvIndexes::default();
        let contribution = indexes.contributions.insert(
            symbol_module.clone(),
            if imported {
                ContributionKind::ImportedSource {
                    source_id: fixture.source,
                }
            } else {
                ContributionKind::LocalSource {
                    source_id: fixture.source,
                }
            },
            SourceAnchor::Range(range(fixture.source, 1, 5)),
        );
        let symbol = SymbolId::new(
            symbol_module.clone(),
            LocalSymbolId::new("mode"),
            FullyQualifiedName::new(format!("{}::ModeT", symbol_module.path().as_str())),
        );
        let mut entry = SymbolEntry::new(
            symbol.clone(),
            SymbolKind::Mode,
            NamespacePath::new(fixture.module.path().as_str()),
            if valid_signature {
                "ModeT [ p ]"
            } else {
                "ModeTX [ p ]"
            },
            SemanticOrigin::new(
                fixture.source,
                symbol_module,
                SourceAnchor::Range(range(fixture.source, 1, 5)),
                vec![0],
            ),
            contribution,
        );
        if imported && exported {
            entry = entry
                .with_visibility(Visibility::Public)
                .with_export_status(ExportStatus::Exported);
        }
        indexes.symbols.insert(entry);
        indexes
            .contributions
            .add_symbol(contribution, symbol.clone());
        fixture.symbols = SymbolEnv::new(fixture.module.clone(), indexes);
        fixture.input.expressions[0].head = SourceTypeHead::Symbol {
            symbol,
            contribution,
        };
        fixture.input.expressions[0].head_spelling = "ModeT".to_owned();
        fixture.input.expressions[0].head_range = range(fixture.source, 10, 15);
        refresh(fixture);
    }

    #[derive(Clone, Copy, Debug)]
    enum LocalSymbolMutation {
        EntryContribution,
        Kind,
        Signature,
        Namespace,
        ContributionModule,
        ContributionKind,
        ContributionSource,
        MissingEffect,
        OriginSource,
        OriginModule,
        OriginAnchor,
        OriginAfterUse,
        OriginRecovery,
    }

    fn install_local_symbol_mutation(fixture: &mut Fixture, mutation: LocalSymbolMutation) {
        let wrong_module = module("wrong.symbol");
        let contribution_module = if matches!(mutation, LocalSymbolMutation::ContributionModule) {
            wrong_module.clone()
        } else {
            fixture.module.clone()
        };
        let contribution_kind = match mutation {
            LocalSymbolMutation::ContributionKind => ContributionKind::ImportedSource {
                source_id: fixture.source,
            },
            LocalSymbolMutation::ContributionSource => ContributionKind::LocalSource {
                source_id: other_source_id(),
            },
            _ => ContributionKind::LocalSource {
                source_id: fixture.source,
            },
        };
        let mut indexes = SymbolEnvIndexes::default();
        let contribution = indexes.contributions.insert(
            contribution_module,
            contribution_kind,
            SourceAnchor::Range(range(fixture.source, 1, 5)),
        );
        let alternate_contribution = indexes.contributions.insert(
            fixture.module.clone(),
            ContributionKind::LocalSource {
                source_id: fixture.source,
            },
            SourceAnchor::Range(range(fixture.source, 1, 5)),
        );
        let symbol = SymbolId::new(
            fixture.module.clone(),
            LocalSymbolId::new("mode"),
            FullyQualifiedName::new(format!("{}::ModeT", fixture.module.path().as_str())),
        );
        let origin_source = if matches!(mutation, LocalSymbolMutation::OriginSource) {
            other_source_id()
        } else {
            fixture.source
        };
        let origin_module = if matches!(mutation, LocalSymbolMutation::OriginModule) {
            wrong_module
        } else {
            fixture.module.clone()
        };
        let origin_anchor = match mutation {
            LocalSymbolMutation::OriginAnchor => SourceAnchor::Point {
                source_id: fixture.source,
                offset: 1,
            },
            LocalSymbolMutation::OriginAfterUse => {
                SourceAnchor::Range(range(fixture.source, 20, 25))
            }
            _ => SourceAnchor::Range(range(fixture.source, 1, 5)),
        };
        let mut origin = SemanticOrigin::new(origin_source, origin_module, origin_anchor, vec![0]);
        if matches!(mutation, LocalSymbolMutation::OriginRecovery) {
            origin = origin.recovered();
        }
        let entry_contribution = if matches!(mutation, LocalSymbolMutation::EntryContribution) {
            alternate_contribution
        } else {
            contribution
        };
        let entry = SymbolEntry::new(
            symbol.clone(),
            if matches!(mutation, LocalSymbolMutation::Kind) {
                SymbolKind::Predicate
            } else {
                SymbolKind::Mode
            },
            if matches!(mutation, LocalSymbolMutation::Namespace) {
                NamespacePath::new("wrong.namespace")
            } else {
                NamespacePath::new(fixture.module.path().as_str())
            },
            if matches!(mutation, LocalSymbolMutation::Signature) {
                "ModeTX [ p ]"
            } else {
                "ModeT [ p ]"
            },
            origin,
            entry_contribution,
        );
        indexes.symbols.insert(entry);
        if !matches!(mutation, LocalSymbolMutation::MissingEffect) {
            indexes
                .contributions
                .add_symbol(contribution, symbol.clone());
            if entry_contribution != contribution {
                indexes
                    .contributions
                    .add_symbol(entry_contribution, symbol.clone());
            }
        }
        fixture.symbols = SymbolEnv::new(fixture.module.clone(), indexes);
        fixture.input.expressions[0].head = SourceTypeHead::Symbol {
            symbol,
            contribution,
        };
        fixture.input.expressions[0].head_spelling = "ModeT".to_owned();
        fixture.input.expressions[0].head_range = range(fixture.source, 10, 15);
        refresh(fixture);
    }

    #[test]
    fn flat_bracket_handoff_is_dense_immutable_and_deterministic() {
        let fixture = fixture();
        let first = build(&fixture).expect("handoff");
        let second = build(&fixture).expect("handoff");
        assert_eq!(first, second);
        assert_eq!(first.applications().len(), 1);
        assert_eq!(first.expressions().len(), 3);
        assert_eq!(first.arguments().len(), 2);
        assert_eq!(
            first
                .applications()
                .get(SourceTypeApplicationId::new(0))
                .map(SourceTypeApplication::root),
            Some(SourceTypeExpressionId::new(0))
        );
        assert_eq!(first.debug_text(), second.debug_text());
        assert!(
            first
                .debug_text()
                .starts_with("source-type-application-debug-v1\n")
        );
    }

    #[test]
    fn of_and_over_term_sites_preserve_provenance_without_binding_ids() {
        for form in [
            SourceTypeApplicationForm::Of,
            SourceTypeApplicationForm::Over,
        ] {
            let mut fixture = fixture();
            fixture.input.expressions.truncate(1);
            fixture.input.expressions[0].form = form;
            fixture.input.arguments = vec![SourceTypeArgumentInput {
                parent: SourceTypeExpressionId::new(0),
                ordinal: 0,
                argument: SourceTypeArgument::TermSite {
                    site: role(3, "term-0-0"),
                    source_range: range(fixture.source, 20, 21),
                    spelling: "x".to_owned(),
                    recovery: NodeRecoveryState::Normal,
                    provenance: SemanticOrigin::new(
                        fixture.source,
                        fixture.module.clone(),
                        SourceAnchor::Range(range(fixture.source, 20, 21)),
                        vec![0, 0],
                    ),
                },
            }];
            refresh(&mut fixture);
            let handoff = build(&fixture).expect("term handoff");
            assert!(matches!(
                handoff
                    .arguments()
                    .get(SourceTypeArgumentId::new(0))
                    .expect("argument")
                    .argument(),
                SourceTypeArgument::TermSite { .. }
            ));
        }
    }

    #[test]
    fn local_and_imported_mode_heads_are_authenticated() {
        let mut local = fixture();
        install_symbol(&mut local, false, false, true);
        build(&local).expect("local mode");

        let mut imported = fixture();
        install_symbol(&mut imported, true, false, true);
        assert!(matches!(
            build(&imported),
            Err(SourceTypeError::InvalidSymbolHead { .. })
        ));

        let mut wrong_signature = fixture();
        install_symbol(&mut wrong_signature, false, false, false);
        assert!(matches!(
            build(&wrong_signature),
            Err(SourceTypeError::InvalidSymbolHead { .. })
        ));

        let mut wrong_form = fixture();
        install_symbol(&mut wrong_form, false, false, true);
        wrong_form.input.expressions[0].form = SourceTypeApplicationForm::Of;
        assert!(matches!(
            build(&wrong_form),
            Err(SourceTypeError::InvalidSymbolHead { .. })
        ));

        let mut missing_import = fixture();
        install_symbol(&mut missing_import, true, true, true);
        assert!(matches!(
            build(&missing_import),
            Err(SourceTypeError::InvalidSymbolHead { .. })
        ));
    }

    #[test]
    fn local_symbol_identity_contribution_and_provenance_matrix_is_rejected() {
        for mutation in [
            LocalSymbolMutation::EntryContribution,
            LocalSymbolMutation::Kind,
            LocalSymbolMutation::Signature,
            LocalSymbolMutation::Namespace,
            LocalSymbolMutation::ContributionModule,
            LocalSymbolMutation::ContributionKind,
            LocalSymbolMutation::ContributionSource,
            LocalSymbolMutation::MissingEffect,
            LocalSymbolMutation::OriginSource,
            LocalSymbolMutation::OriginModule,
            LocalSymbolMutation::OriginAnchor,
            LocalSymbolMutation::OriginAfterUse,
            LocalSymbolMutation::OriginRecovery,
        ] {
            let mut fixture = fixture();
            install_local_symbol_mutation(&mut fixture, mutation);
            assert!(
                matches!(
                    build(&fixture),
                    Err(SourceTypeError::InvalidSymbolHead { .. })
                ),
                "local symbol mutation {mutation:?} was accepted"
            );
        }

        let mut missing_symbol = fixture();
        install_symbol(&mut missing_symbol, false, false, true);
        let contribution = match &missing_symbol.input.expressions[0].head {
            SourceTypeHead::Symbol { contribution, .. } => *contribution,
            _ => unreachable!(),
        };
        missing_symbol.input.expressions[0].head = SourceTypeHead::Symbol {
            symbol: SymbolId::new(
                missing_symbol.module.clone(),
                LocalSymbolId::new("missing"),
                FullyQualifiedName::new("source.type::Missing"),
            ),
            contribution,
        };
        assert!(matches!(
            build(&missing_symbol),
            Err(SourceTypeError::InvalidSymbolHead { .. })
        ));
    }

    #[test]
    fn environment_and_binding_drift_are_rejected_transactionally() {
        let mut wrong_environment = fixture();
        wrong_environment.symbols = SymbolEnv::new(module("wrong"), SymbolEnvIndexes::default());
        assert_eq!(
            build(&wrong_environment),
            Err(SourceTypeError::EnvironmentMismatch)
        );

        let mut wrong_binding_source = fixture();
        wrong_binding_source.bindings = binding_env_with_identity(
            &wrong_binding_source,
            other_source_id(),
            wrong_binding_source.module.clone(),
        );
        assert_eq!(
            build(&wrong_binding_source),
            Err(SourceTypeError::EnvironmentMismatch)
        );

        let mut wrong_binding_module = fixture();
        wrong_binding_module.bindings = binding_env_with_identity(
            &wrong_binding_module,
            wrong_binding_module.source,
            module("wrong.binding"),
        );
        assert_eq!(
            build(&wrong_binding_module),
            Err(SourceTypeError::EnvironmentMismatch)
        );

        let mut wrong_ordinal = fixture();
        wrong_ordinal.input.applications[0].source_ordinal = 1;
        assert!(matches!(
            build(&wrong_ordinal),
            Err(SourceTypeError::InvalidApplication { .. })
        ));

        let mut stale_binding = fixture();
        stale_binding.input.expressions[0].source_range.end -= 1;
        stale_binding.arena = arena_for(&stale_binding.input);
        assert!(matches!(
            build(&stale_binding),
            Err(SourceTypeError::InvalidBinding { .. })
        ));
    }

    #[test]
    fn empty_cardinality_and_application_identity_corruptions_are_rejected() {
        let mut empty_applications = fixture();
        empty_applications.input.applications.clear();
        assert_eq!(
            build(&empty_applications),
            Err(SourceTypeError::EmptyApplications)
        );

        let mut empty_expressions = fixture();
        empty_expressions.input.expressions.clear();
        assert_eq!(
            build(&empty_expressions),
            Err(SourceTypeError::EmptyExpressions)
        );

        let mut cardinality = fixture();
        cardinality.bindings = binding_env(
            cardinality.source,
            &cardinality.module,
            &[
                cardinality.input.expressions[0].source_range,
                cardinality.input.expressions[0].source_range,
            ],
        );
        assert_eq!(
            build(&cardinality),
            Err(SourceTypeError::BindingCardinalityMismatch)
        );

        let mut dangling_root = fixture();
        dangling_root.input.applications[0].root = SourceTypeExpressionId::new(99);
        assert!(matches!(
            build(&dangling_root),
            Err(SourceTypeError::InvalidApplication { .. })
        ));

        let mut duplicate_root = two_root_fixture();
        duplicate_root.input.applications[1].root = SourceTypeExpressionId::new(0);
        assert!(matches!(
            build(&duplicate_root),
            Err(SourceTypeError::InvalidApplication { .. })
        ));

        let mut duplicate_binding = two_root_fixture();
        duplicate_binding.input.applications[1].binding = BindingId::new(0);
        assert!(matches!(
            build(&duplicate_binding),
            Err(SourceTypeError::InvalidApplication { .. })
        ));

        let mut non_monotonic = two_root_fixture();
        non_monotonic.input.applications[0].root = SourceTypeExpressionId::new(1);
        non_monotonic.input.applications[1].root = SourceTypeExpressionId::new(0);
        refresh(&mut non_monotonic);
        assert!(matches!(
            build(&non_monotonic),
            Err(SourceTypeError::InvalidApplication { .. })
        ));
    }

    #[test]
    fn every_checker_specific_binding_invariant_is_rejected() {
        for mutation in [
            BindingMutation::GlobalDiagnostic,
            BindingMutation::Recovery,
            BindingMutation::Diagnostic,
            BindingMutation::Capture,
            BindingMutation::WrongIdentity,
            BindingMutation::WrongStatus,
            BindingMutation::WrongOrdinal,
            BindingMutation::ContextRecovery,
            BindingMutation::MissingMembership,
            BindingMutation::MissingVisibility,
            BindingMutation::EmptyDeclarationRange,
            BindingMutation::WrongTypeSite,
        ] {
            let mut mutated = fixture();
            mutated.bindings = binding_env_with_mutation(&mutated, mutation);
            assert!(
                matches!(
                    build(&mutated),
                    Err(SourceTypeError::BindingCardinalityMismatch)
                        | Err(SourceTypeError::InvalidBinding { .. })
                ),
                "binding mutation was accepted"
            );
        }
    }

    #[test]
    fn expression_identity_range_spelling_and_site_matrix_is_rejected() {
        let mut expression_source = fixture();
        expression_source.input.expressions[0].source_id = other_source_id();
        assert!(matches!(
            build(&expression_source),
            Err(SourceTypeError::InvalidExpression { .. })
        ));

        let mut expression_module = fixture();
        expression_module.input.expressions[0].module_id = module("wrong.expression");
        assert!(matches!(
            build(&expression_module),
            Err(SourceTypeError::InvalidExpression { .. })
        ));

        let mut source_range_source = fixture();
        source_range_source.input.expressions[0]
            .source_range
            .source_id = other_source_id();
        assert!(matches!(
            build(&source_range_source),
            Err(SourceTypeError::InvalidExpression { .. })
        ));

        let mut empty_source_range = fixture();
        empty_source_range.input.expressions[0].source_range.end =
            empty_source_range.input.expressions[0].source_range.start;
        assert!(matches!(
            build(&empty_source_range),
            Err(SourceTypeError::InvalidExpression { .. })
        ));

        let mut head_range_source = fixture();
        head_range_source.input.expressions[0].head_range.source_id = other_source_id();
        assert!(matches!(
            build(&head_range_source),
            Err(SourceTypeError::InvalidExpression { .. })
        ));

        let mut empty_head_range = fixture();
        empty_head_range.input.expressions[0].head_range.end =
            empty_head_range.input.expressions[0].head_range.start;
        assert!(matches!(
            build(&empty_head_range),
            Err(SourceTypeError::InvalidExpression { .. })
        ));

        let mut empty_head_spelling = fixture();
        empty_head_spelling.input.expressions[0]
            .head_spelling
            .clear();
        assert!(matches!(
            build(&empty_head_spelling),
            Err(SourceTypeError::InvalidExpression { .. })
        ));

        let mut same_role_site = fixture();
        same_role_site.input.expressions[0].head_site =
            same_role_site.input.expressions[0].site.clone();
        assert!(matches!(
            build(&same_role_site),
            Err(SourceTypeError::InvalidExpression { .. })
        ));

        let mut duplicate_site = fixture();
        duplicate_site.input.expressions[1].site = duplicate_site.input.expressions[0].site.clone();
        assert_eq!(build(&duplicate_site), Err(SourceTypeError::DuplicateSite));

        let mut missing_head_site = fixture();
        missing_head_site.input.expressions[0].head_site = role(99, "missing-head");
        assert!(matches!(
            build(&missing_head_site),
            Err(SourceTypeError::InvalidExpressionSite { .. })
        ));

        let mut wrong_head_anchor = fixture();
        wrong_head_anchor.input.expressions[0].head_site = role(1, "wrong-head-anchor");
        assert!(matches!(
            build(&wrong_head_anchor),
            Err(SourceTypeError::InvalidExpressionSite { .. })
        ));

        let mut wrong_object_head = fixture();
        wrong_object_head.input.expressions[1].head = SourceTypeHead::BuiltinSet;
        assert!(matches!(
            build(&wrong_object_head),
            Err(SourceTypeError::InvalidHead { .. })
        ));
    }

    #[test]
    fn argument_site_range_spelling_and_provenance_matrix_is_rejected() {
        let mut dangling_parent = term_fixture(SourceTypeApplicationForm::Of);
        dangling_parent.input.arguments[0].parent = SourceTypeExpressionId::new(99);
        assert!(matches!(
            build(&dangling_parent),
            Err(SourceTypeError::InvalidArgument { .. })
        ));

        let mut wrong_range_source = term_fixture(SourceTypeApplicationForm::Of);
        let SourceTypeArgument::TermSite {
            ref mut source_range,
            ..
        } = wrong_range_source.input.arguments[0].argument
        else {
            unreachable!()
        };
        source_range.source_id = other_source_id();
        assert!(matches!(
            build(&wrong_range_source),
            Err(SourceTypeError::InvalidArgument { .. })
        ));

        let mut empty_range = term_fixture(SourceTypeApplicationForm::Of);
        let SourceTypeArgument::TermSite {
            ref mut source_range,
            ..
        } = empty_range.input.arguments[0].argument
        else {
            unreachable!()
        };
        source_range.end = source_range.start;
        assert!(matches!(
            build(&empty_range),
            Err(SourceTypeError::InvalidArgument { .. })
        ));

        let mut empty_spelling = term_fixture(SourceTypeApplicationForm::Of);
        let SourceTypeArgument::TermSite {
            ref mut spelling, ..
        } = empty_spelling.input.arguments[0].argument
        else {
            unreachable!()
        };
        spelling.clear();
        assert!(matches!(
            build(&empty_spelling),
            Err(SourceTypeError::InvalidArgument { .. })
        ));

        let mut duplicate_site = term_fixture(SourceTypeApplicationForm::Of);
        let SourceTypeArgument::TermSite { ref mut site, .. } =
            duplicate_site.input.arguments[0].argument
        else {
            unreachable!()
        };
        *site = duplicate_site.input.expressions[0].site.clone();
        assert!(matches!(
            build(&duplicate_site),
            Err(SourceTypeError::InvalidArgument { .. })
        ));

        let mut missing_site = term_fixture(SourceTypeApplicationForm::Of);
        let SourceTypeArgument::TermSite { ref mut site, .. } =
            missing_site.input.arguments[0].argument
        else {
            unreachable!()
        };
        *site = role(99, "missing-term");
        assert!(matches!(
            build(&missing_site),
            Err(SourceTypeError::InvalidArgumentSite { .. })
        ));

        let mut wrong_anchor = term_fixture(SourceTypeApplicationForm::Of);
        let SourceTypeArgument::TermSite { ref mut site, .. } =
            wrong_anchor.input.arguments[0].argument
        else {
            unreachable!()
        };
        *site = role(1, "wrong-term-anchor");
        assert!(matches!(
            build(&wrong_anchor),
            Err(SourceTypeError::InvalidArgumentSite { .. })
        ));

        let mut recovery_mismatch = term_fixture(SourceTypeApplicationForm::Of);
        let SourceTypeArgument::TermSite {
            ref mut recovery,
            ref mut provenance,
            ..
        } = recovery_mismatch.input.arguments[0].argument
        else {
            unreachable!()
        };
        *recovery = NodeRecoveryState::Recovered;
        *provenance = provenance.clone().recovered();
        assert!(matches!(
            build(&recovery_mismatch),
            Err(SourceTypeError::InvalidArgumentSite { .. })
        ));

        for mutation in 0..5 {
            let mut invalid = term_fixture(SourceTypeApplicationForm::Of);
            let SourceTypeArgument::TermSite {
                provenance: actual,
                source_range,
                ..
            } = &mut invalid.input.arguments[0].argument
            else {
                unreachable!()
            };
            let mut provenance = SemanticOrigin::new(
                invalid.source,
                invalid.module.clone(),
                SourceAnchor::Range(*source_range),
                vec![0, 0],
            );
            match mutation {
                0 => {
                    let source = other_source_id();
                    provenance = SemanticOrigin::new(
                        source,
                        invalid.module.clone(),
                        SourceAnchor::Range(range(source, source_range.start, source_range.end)),
                        vec![0, 0],
                    );
                }
                1 => {
                    provenance = SemanticOrigin::new(
                        invalid.source,
                        module("wrong.provenance"),
                        SourceAnchor::Range(*source_range),
                        vec![0, 0],
                    );
                }
                2 => {
                    provenance = SemanticOrigin::new(
                        invalid.source,
                        invalid.module.clone(),
                        SourceAnchor::Range(range(
                            invalid.source,
                            source_range.start + 1,
                            source_range.end + 1,
                        )),
                        vec![0, 0],
                    );
                }
                3 => {
                    provenance = SemanticOrigin::new(
                        invalid.source,
                        invalid.module.clone(),
                        SourceAnchor::Range(*source_range),
                        vec![0, 1],
                    );
                }
                4 => provenance = provenance.recovered(),
                _ => unreachable!(),
            }
            *actual = provenance;
            assert!(matches!(
                build(&invalid),
                Err(SourceTypeError::InvalidProvenance { .. })
            ));
        }

        let mut empty_radix = fixture();
        let SourceTypeArgument::QuaSite { radix, .. } =
            &mut empty_radix.input.arguments[1].argument
        else {
            unreachable!()
        };
        radix.clear();
        assert!(matches!(
            build(&empty_radix),
            Err(SourceTypeError::InvalidArgument { .. })
        ));
    }

    #[test]
    fn every_application_form_rejects_wrong_argument_shapes() {
        let bare_with_term = term_fixture(SourceTypeApplicationForm::Bare);
        assert!(matches!(
            validate_forms(&bare_with_term.input),
            Err(SourceTypeError::WrongApplicationForm { .. })
        ));

        for form in [
            SourceTypeApplicationForm::Of,
            SourceTypeApplicationForm::Over,
            SourceTypeApplicationForm::Bracket,
        ] {
            let mut empty = fixture();
            empty.input.expressions.truncate(1);
            empty.input.expressions[0].form = form;
            empty.input.arguments.clear();
            assert!(matches!(
                validate_forms(&empty.input),
                Err(SourceTypeError::WrongApplicationForm { .. })
            ));
        }

        for form in [
            SourceTypeApplicationForm::Of,
            SourceTypeApplicationForm::Over,
        ] {
            let mut type_argument = fixture();
            type_argument.input.expressions[0].form = form;
            assert!(matches!(
                validate_forms(&type_argument.input),
                Err(SourceTypeError::WrongApplicationForm { .. })
            ));
        }

        let bracket_with_term = term_fixture(SourceTypeApplicationForm::Bracket);
        assert!(matches!(
            validate_forms(&bracket_with_term.input),
            Err(SourceTypeError::WrongApplicationForm { .. })
        ));
    }

    #[test]
    fn expression_spelling_ranges_heads_and_arena_sites_are_rejected_on_drift() {
        let mut empty_spelling = fixture();
        empty_spelling.input.expressions[0].spelling.clear();
        assert!(matches!(
            build(&empty_spelling),
            Err(SourceTypeError::InvalidExpression { .. })
        ));

        let mut wrong_builtin = fixture();
        wrong_builtin.input.expressions[0].head_spelling = "SET".to_owned();
        assert!(matches!(
            build(&wrong_builtin),
            Err(SourceTypeError::InvalidHead { .. })
        ));

        let mut head_outside = fixture();
        head_outside.input.expressions[0].head_range = range(head_outside.source, 91, 94);
        assert!(matches!(
            build(&head_outside),
            Err(SourceTypeError::InvalidExpression { .. })
        ));

        let mut missing_site = fixture();
        missing_site.input.expressions[0].site = role(99, "missing");
        assert!(matches!(
            build(&missing_site),
            Err(SourceTypeError::InvalidExpressionSite { .. })
        ));

        let mut recovery_drift = fixture();
        let mut nodes = recovery_drift
            .arena
            .iter()
            .map(|(_, node)| node.clone())
            .collect::<Vec<_>>();
        nodes[0].recovery = NodeRecoveryState::Recovered;
        recovery_drift.arena = TypedArena::try_new(None, nodes).expect("arena");
        assert!(matches!(
            build(&recovery_drift),
            Err(SourceTypeError::InvalidExpressionSite { .. })
        ));
    }

    #[test]
    fn argument_order_site_provenance_and_form_are_enforced() {
        let mut wrong_order = fixture();
        wrong_order.input.arguments[1].ordinal = 2;
        assert!(matches!(
            build(&wrong_order),
            Err(SourceTypeError::ReorderedArgument { .. })
        ));

        let mut decreasing_parent = fixture();
        decreasing_parent.input.arguments[0].parent = SourceTypeExpressionId::new(1);
        decreasing_parent.input.arguments[1].parent = SourceTypeExpressionId::new(0);
        decreasing_parent.input.arguments[1].ordinal = 0;
        assert!(matches!(
            build(&decreasing_parent),
            Err(SourceTypeError::ReorderedArgument { .. })
        ));

        let mut wrong_provenance = fixture();
        let SourceTypeArgument::QuaSite {
            ref mut provenance, ..
        } = wrong_provenance.input.arguments[1].argument
        else {
            unreachable!()
        };
        *provenance = SemanticOrigin::new(
            wrong_provenance.source,
            wrong_provenance.module.clone(),
            SourceAnchor::Range(range(wrong_provenance.source, 40, 41)),
            vec![0, 0],
        );
        assert!(matches!(
            build(&wrong_provenance),
            Err(SourceTypeError::InvalidProvenance { .. })
        ));

        let mut wrong_form = fixture();
        wrong_form.input.expressions[0].form = SourceTypeApplicationForm::Bare;
        assert!(matches!(
            build(&wrong_form),
            Err(SourceTypeError::WrongApplicationForm { .. })
        ));

        let mut overlapping = fixture();
        let SourceTypeArgument::QuaSite {
            ref mut source_range,
            ..
        } = overlapping.input.arguments[1].argument
        else {
            unreachable!()
        };
        *source_range = range(overlapping.source, 25, 26);
        overlapping.arena = arena_for(&overlapping.input);
        assert!(matches!(
            build(&overlapping),
            Err(SourceTypeError::InvalidProvenance { .. })
                | Err(SourceTypeError::OverlappingSiblings { .. })
        ));
    }

    #[test]
    fn dangling_duplicate_and_multiple_parent_children_are_rejected() {
        let mut dangling = fixture();
        dangling.input.arguments[0].argument = SourceTypeArgument::TypeSite {
            expression: SourceTypeExpressionId::new(99),
        };
        assert!(matches!(
            build(&dangling),
            Err(SourceTypeError::DanglingChild { .. })
        ));

        let mut duplicate = fixture();
        let SourceTypeArgument::QuaSite { radix, .. } = &mut duplicate.input.arguments[1].argument
        else {
            unreachable!()
        };
        *radix = vec![SourceTypeExpressionId::new(1)];
        assert!(matches!(
            build(&duplicate),
            Err(SourceTypeError::DuplicateChild { .. })
        ));

        let mut multiple = fixture();
        multiple.input.expressions[1].form = SourceTypeApplicationForm::Bracket;
        multiple.input.arguments.push(SourceTypeArgumentInput {
            parent: SourceTypeExpressionId::new(1),
            ordinal: 0,
            argument: SourceTypeArgument::TypeSite {
                expression: SourceTypeExpressionId::new(2),
            },
        });
        assert!(matches!(
            build(&multiple),
            Err(SourceTypeError::MultipleParents { .. })
        ));
    }

    #[test]
    fn cycles_forward_parents_and_unreachable_expressions_are_rejected() {
        let mut cycle = fixture();
        cycle.input.expressions[1].form = SourceTypeApplicationForm::Bracket;
        cycle.input.expressions[2].form = SourceTypeApplicationForm::Bracket;
        cycle.input.expressions[1].source_range = range(cycle.source, 20, 70);
        cycle.input.expressions[2].source_range = range(cycle.source, 20, 70);
        cycle.input.arguments = vec![
            SourceTypeArgumentInput {
                parent: SourceTypeExpressionId::new(1),
                ordinal: 0,
                argument: SourceTypeArgument::TypeSite {
                    expression: SourceTypeExpressionId::new(2),
                },
            },
            SourceTypeArgumentInput {
                parent: SourceTypeExpressionId::new(2),
                ordinal: 0,
                argument: SourceTypeArgument::TypeSite {
                    expression: SourceTypeExpressionId::new(1),
                },
            },
        ];
        refresh(&mut cycle);
        assert!(matches!(build(&cycle), Err(SourceTypeError::Cycle { .. })));

        let mut forward = fixture();
        forward.input.expressions = vec![
            bare_expression(forward.source, &forward.module, 1, 20, 30, "object"),
            {
                let mut parent = bare_expression(forward.source, &forward.module, 0, 10, 90, "set");
                parent.form = SourceTypeApplicationForm::Bracket;
                parent
            },
        ];
        forward.input.applications[0].root = SourceTypeExpressionId::new(1);
        forward.input.arguments = vec![SourceTypeArgumentInput {
            parent: SourceTypeExpressionId::new(1),
            ordinal: 0,
            argument: SourceTypeArgument::TypeSite {
                expression: SourceTypeExpressionId::new(0),
            },
        }];
        refresh(&mut forward);
        assert!(matches!(
            build(&forward),
            Err(SourceTypeError::ForwardParent { .. })
        ));

        let mut unreachable = fixture();
        unreachable.input.arguments.clear();
        unreachable.input.expressions[0].form = SourceTypeApplicationForm::Bare;
        refresh(&mut unreachable);
        assert!(matches!(
            build(&unreachable),
            Err(SourceTypeError::UnreachableExpression { .. })
        ));
    }

    #[test]
    fn root_parent_conflict_is_rejected_before_general_graph_traversal() {
        let fixture = two_root_fixture();
        let roots = BTreeSet::from([
            SourceTypeExpressionId::new(0),
            SourceTypeExpressionId::new(1),
        ]);
        let parents = vec![None, Some(SourceTypeExpressionId::new(0))];
        let children = vec![vec![SourceTypeExpressionId::new(1)], Vec::new()];
        assert_eq!(
            validate_graph(&fixture.input, &roots, &parents, &children),
            Err(SourceTypeError::RootHasParent {
                root: SourceTypeExpressionId::new(1),
            })
        );
    }

    #[test]
    fn deep_forward_graph_is_validated_iteratively_without_stack_growth() {
        const DEPTH: usize = 10_000;

        let source = source_id();
        let module = module("source.type.deep");
        let mut expressions = Vec::with_capacity(DEPTH);
        let mut arguments = Vec::with_capacity(DEPTH - 1);
        for index in 0..DEPTH {
            let start = index + 1;
            let end = DEPTH * 3 - index;
            let mut expression = bare_expression(source, &module, index, start, end, "set");
            if index + 1 < DEPTH {
                expression.form = SourceTypeApplicationForm::Bracket;
                arguments.push(SourceTypeArgumentInput {
                    parent: SourceTypeExpressionId::new(index),
                    ordinal: 0,
                    argument: SourceTypeArgument::TypeSite {
                        expression: SourceTypeExpressionId::new(index + 1),
                    },
                });
            }
            expressions.push(expression);
        }
        let input = SourceTypeHandoffInput {
            source_id: source,
            module_id: module.clone(),
            applications: vec![SourceTypeApplicationInput {
                binding: BindingId::new(0),
                source_ordinal: 0,
                root: SourceTypeExpressionId::new(0),
            }],
            expressions,
            arguments,
        };
        let bindings = binding_env(source, &module, &[input.expressions[0].source_range]);
        let symbols = SymbolEnv::new(module, SymbolEnvIndexes::default());
        let arena = arena_for(&input);
        let handoff = SourceTypeProducer::build(input, &bindings, &symbols, &arena)
            .expect("deep forward source-type graph");
        assert_eq!(handoff.expressions().len(), DEPTH);
        assert_eq!(handoff.arguments().len(), DEPTH - 1);
    }

    #[test]
    fn parent_sibling_and_top_level_ranges_are_enforced() {
        let mut outside = fixture();
        outside.input.expressions[1].source_range = range(outside.source, 95, 105);
        outside.input.expressions[1].head_range = range(outside.source, 95, 101);
        refresh(&mut outside);
        assert!(matches!(
            build(&outside),
            Err(SourceTypeError::InvalidArgument { .. })
                | Err(SourceTypeError::ChildOutsideParent { .. })
        ));

        let mut siblings = fixture();
        let SourceTypeArgument::QuaSite {
            source_range,
            provenance,
            ..
        } = &mut siblings.input.arguments[1].argument
        else {
            unreachable!()
        };
        *source_range = range(siblings.source, 25, 26);
        *provenance = SemanticOrigin::new(
            siblings.source,
            siblings.module.clone(),
            SourceAnchor::Range(*source_range),
            vec![0, 1],
        );
        refresh(&mut siblings);
        assert!(matches!(
            build(&siblings),
            Err(SourceTypeError::OverlappingSiblings { .. })
        ));

        let mut top_level = fixture();
        top_level.input.expressions = vec![
            bare_expression(top_level.source, &top_level.module, 0, 10, 30, "set"),
            bare_expression(top_level.source, &top_level.module, 1, 20, 40, "object"),
        ];
        top_level.input.applications = vec![
            SourceTypeApplicationInput {
                binding: BindingId::new(0),
                source_ordinal: 0,
                root: SourceTypeExpressionId::new(0),
            },
            SourceTypeApplicationInput {
                binding: BindingId::new(1),
                source_ordinal: 1,
                root: SourceTypeExpressionId::new(1),
            },
        ];
        top_level.input.arguments.clear();
        refresh(&mut top_level);
        assert!(matches!(
            build(&top_level),
            Err(SourceTypeError::OverlappingApplications { .. })
        ));
    }

    #[test]
    fn typed_ast_installation_rechecks_the_actual_arena() {
        let fixture = fixture();
        let handoff = build(&fixture).expect("handoff");
        handoff
            .validate_installation(fixture.source, &fixture.module, &fixture.arena)
            .expect("same arena");
        let empty = TypedArena::try_new(None, Vec::new()).expect("empty arena");
        assert!(matches!(
            handoff.validate_installation(fixture.source, &fixture.module, &empty),
            Err(SourceTypeError::InvalidExpressionSite { .. })
        ));
        assert_eq!(
            handoff.validate_installation(fixture.source, &module("wrong"), &fixture.arena),
            Err(SourceTypeError::EnvironmentMismatch)
        );

        let typed = TypedAst::try_new(TypedAstParts {
            source_id: fixture.source,
            module_id: fixture.module.clone(),
            resolved_root: None,
            source_context: None,
            source_type: Some(handoff.clone()),
            source_attribute: None,
            nodes: fixture.arena.clone(),
            contexts: LocalTypeContextTable::new(),
            types: TypeTable::new(),
            facts: TypeFactTable::new(),
            coercions: CoercionTable::new(),
            initial_obligations: InitialObligationTable::new(),
            diagnostics: TypeDiagnosticTable::new(),
        })
        .expect("TypedAst source type ownership");
        assert_eq!(typed.source_type(), Some(&handoff));

        assert_eq!(
            TypedAst::try_new(TypedAstParts {
                source_id: fixture.source,
                module_id: fixture.module,
                resolved_root: None,
                source_context: None,
                source_type: Some(handoff),
                source_attribute: None,
                nodes: empty,
                contexts: LocalTypeContextTable::new(),
                types: TypeTable::new(),
                facts: TypeFactTable::new(),
                coercions: CoercionTable::new(),
                initial_obligations: InitialObligationTable::new(),
                diagnostics: TypeDiagnosticTable::new(),
            }),
            Err(TypedAstError::InvalidSourceType)
        );
    }

    #[test]
    fn legacy_input_only_seam_returns_the_real_binding_env() {
        let source = source_id();
        let module = module("legacy.reserve");
        let symbols = SymbolEnv::new(module.clone(), SymbolEnvIndexes::default());
        let bridge = SourceReserveDeclarationBridge::new(
            source,
            module,
            range(source, 0, 20),
            vec![SourceReserveBindingInput::new(
                "x",
                range(source, 1, 2),
                range(source, 10, 13),
                "set",
                TypeHeadInput::BuiltinSet,
            )],
        )
        .expect("bridge");
        let bindings = bridge.prepare_binding_env(&symbols).expect("binding env");
        assert_eq!(bindings.bindings().len(), 1);
        assert_eq!(
            bindings
                .bindings()
                .get(BindingId::new(0))
                .map(|entry| &entry.type_site),
            Some(&BindingTypeSite::Source(range(source, 10, 13)))
        );
        assert!(bindings.diagnostics().is_empty());
        let checked = bridge.check(&symbols).expect("legacy semantic bridge");
        assert_eq!(checked.binding_env(), &bindings);
    }

    #[test]
    fn generated_declaration_context_is_not_authenticated() {
        let source = source_id();
        let module = module("source.context");
        let expressions = vec![
            bare_expression(source, &module, 0, 10, 13, "set"),
            bare_expression(source, &module, 1, 30, 33, "set"),
        ];
        let input = SourceTypeHandoffInput {
            source_id: source,
            module_id: module.clone(),
            applications: vec![
                SourceTypeApplicationInput {
                    binding: BindingId::new(0),
                    source_ordinal: 0,
                    root: SourceTypeExpressionId::new(0),
                },
                SourceTypeApplicationInput {
                    binding: BindingId::new(1),
                    source_ordinal: 1,
                    root: SourceTypeExpressionId::new(1),
                },
            ],
            expressions,
            arguments: Vec::new(),
        };
        let reserve_range = range(source, 1, 2);
        let parameter_range = range(source, 20, 21);
        let local_scope = LocalTermScope::new(vec![1]);
        let mut bindings = BindingTable::new();
        bindings.insert(BindingDraft {
            spelling: "r".to_owned(),
            kind: BindingKind::ReservedVariable,
            identity: BinderIdentity::ReservedVariable {
                spelling: "r".to_owned(),
                declaration_range: reserve_range,
            },
            owner_context: BindingContextId::new(0),
            declaration_range: reserve_range,
            visible_after_ordinal: 0,
            type_site: BindingTypeSite::Source(input.expressions[0].source_range),
            status: BindingStatus::Reserved,
            captured: CapturedFreeVariables::default(),
            diagnostics: Vec::new(),
            recovery: BindingRecoveryState::Normal,
        });
        bindings.insert(BindingDraft {
            spelling: "x".to_owned(),
            kind: BindingKind::DefinitionParameter,
            identity: BinderIdentity::ResolverLocal {
                scope: local_scope.clone(),
                ordinal: 1,
                declaration_range: parameter_range,
            },
            owner_context: BindingContextId::new(1),
            declaration_range: parameter_range,
            visible_after_ordinal: 1,
            type_site: BindingTypeSite::Source(input.expressions[1].source_range),
            status: BindingStatus::Active,
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
            bindings: vec![BindingId::new(0)],
            visible_bindings: vec![BindingId::new(0)],
            recovery: BindingContextRecovery::Normal,
        });
        contexts.insert(BindingContextDraft {
            owner: BindingContextOwner::Generated("definition".to_owned()),
            parent: Some(BindingContextId::new(0)),
            layer: BindingContextLayer::Declaration,
            lexical_scope: Some(local_scope),
            bindings: vec![BindingId::new(1)],
            visible_bindings: vec![BindingId::new(0), BindingId::new(1)],
            recovery: BindingContextRecovery::Normal,
        });
        let bindings = BindingEnv::try_new(BindingEnvParts {
            source_id: source,
            module_id: module.clone(),
            contexts,
            bindings,
            diagnostics: BindingDiagnosticTable::new(),
        })
        .expect("Task248-shaped binding environment");
        let symbols = SymbolEnv::new(module, SymbolEnvIndexes::default());
        let arena = arena_for(&input);
        assert!(matches!(
            SourceTypeProducer::build(input, &bindings, &symbols, &arena),
            Err(SourceTypeError::InvalidBinding { .. })
        ));
    }

    #[test]
    fn task_249r_exact_definition_return_extension_and_legacy_debug() {
        let fixture = task_249r_fixture();
        let legacy = concat!(
            "source-type-application-debug-v1\n",
            "module: task249r.functor_definition\n",
            "application#0 binding=0 ordinal=0 root=0\n",
            "application#1 binding=1 ordinal=1 root=1\n",
            "expression#0 form=bare range=22..25 site=node:63 head=builtin:set head_range=22..25 head_site=node:62 recovery=normal spelling=\"set\" head_spelling=\"set\"\n",
            "expression#1 form=bare range=38..41 site=node:67 head=builtin:set head_range=38..41 head_site=node:66 recovery=normal spelling=\"set\" head_spelling=\"set\"\n",
        );
        assert!(fixture.base.definition_returns().is_empty());
        assert_eq!(fixture.base.debug_text(), legacy);

        let extended = SourceTypeDefinitionReturnProducer::extend(
            &fixture.base,
            fixture.extension.clone(),
            &fixture.arena,
        )
        .expect("exact definition-return extension");
        assert!(fixture.base.definition_returns().is_empty());
        assert_eq!(fixture.base.debug_text(), legacy);
        assert_eq!(extended.applications().len(), 2);
        assert_eq!(extended.expressions().len(), 4);
        assert!(extended.arguments().is_empty());
        assert_eq!(extended.definition_returns().len(), 2);
        assert_eq!(
            extended
                .definition_returns()
                .iter()
                .map(|(id, row)| (
                    id.index(),
                    row.id().index(),
                    row.source_ordinal(),
                    row.root().index()
                ))
                .collect::<Vec<_>>(),
            vec![(0, 0, 0, 2), (1, 1, 1, 3)]
        );
        let first = extended
            .definition_returns()
            .get(SourceTypeDefinitionReturnId::new(0))
            .expect("first definition return");
        assert_eq!(first.definition_site(), &node_site(84));
        assert_eq!(first.definition_range(), range(fixture.source, 61, 118));
        assert!(
            extended
                .definition_returns()
                .get(SourceTypeDefinitionReturnId::new(2))
                .is_none()
        );

        assert_eq!(
            extended.debug_text(),
            concat!(
                "source-type-application-debug-v1\n",
                "module: task249r.functor_definition\n",
                "application#0 binding=0 ordinal=0 root=0\n",
                "application#1 binding=1 ordinal=1 root=1\n",
                "definition-return#0 ordinal=0 definition_range=61..118 definition_site=node#84 root=2\n",
                "definition-return#1 ordinal=1 definition_range=121..179 definition_site=node#95 root=3\n",
                "expression#0 form=bare range=22..25 site=node:63 head=builtin:set head_range=22..25 head_site=node:62 recovery=normal spelling=\"set\" head_spelling=\"set\"\n",
                "expression#1 form=bare range=38..41 site=node:67 head=builtin:set head_range=38..41 head_site=node:66 recovery=normal spelling=\"set\" head_spelling=\"set\"\n",
                "expression#2 form=bare range=105..108 site=node:80 head=builtin:set head_range=105..108 head_site=node:79 recovery=normal spelling=\"set\" head_spelling=\"set\"\n",
                "expression#3 form=bare range=163..166 site=node:87 head=builtin:set head_range=163..166 head_site=node:86 recovery=normal spelling=\"set\" head_spelling=\"set\"\n",
            )
        );
    }

    #[test]
    fn task_249r_independent_return_corruption_fails_atomically() {
        let fixture = task_249r_fixture();
        let baseline = fixture.base.clone();

        let mut empty = fixture.extension.clone();
        empty.returns.clear();
        assert_eq!(
            SourceTypeDefinitionReturnProducer::extend(&fixture.base, empty, &fixture.arena),
            Err(SourceTypeError::EmptyDefinitionReturns)
        );

        let mut singleton = fixture.extension.clone();
        singleton.returns.pop();
        assert_eq!(
            SourceTypeDefinitionReturnProducer::extend(&fixture.base, singleton, &fixture.arena),
            Err(SourceTypeError::DefinitionReturnCardinalityMismatch)
        );

        let mut reordered = fixture.extension.clone();
        reordered.returns[1].source_ordinal = 0;
        assert!(matches!(
            SourceTypeDefinitionReturnProducer::extend(&fixture.base, reordered, &fixture.arena),
            Err(SourceTypeError::InvalidDefinitionReturn { definition_return })
                if definition_return == SourceTypeDefinitionReturnId::new(1)
        ));

        let mut wrong_owner_range = fixture.extension.clone();
        wrong_owner_range.returns[0].definition_range = range(fixture.source, 60, 118);
        assert!(matches!(
            SourceTypeDefinitionReturnProducer::extend(
                &fixture.base,
                wrong_owner_range,
                &fixture.arena,
            ),
            Err(SourceTypeError::InvalidDefinitionReturn { definition_return })
                if definition_return == SourceTypeDefinitionReturnId::new(0)
        ));

        for definition_range in [
            range(other_source_id(), 61, 118),
            range(fixture.source, 61, 61),
            range(fixture.source, 105, 106),
        ] {
            let mut invalid = fixture.extension.clone();
            invalid.returns[0].definition_range = definition_range;
            assert_task_249r_extension_error(
                &fixture,
                invalid,
                &fixture.arena,
                SourceTypeError::InvalidDefinitionReturn {
                    definition_return: SourceTypeDefinitionReturnId::new(0),
                },
            );
        }

        let mut role_owner = fixture.extension.clone();
        role_owner.returns[0].definition_site = role(84, "definition-owner");
        assert!(matches!(
            SourceTypeDefinitionReturnProducer::extend(&fixture.base, role_owner, &fixture.arena),
            Err(SourceTypeError::InvalidDefinitionReturnSite { definition_return })
                if definition_return == SourceTypeDefinitionReturnId::new(0)
        ));

        let mut wrong_expression_source = fixture.extension.clone();
        wrong_expression_source.returns[0].expression.source_id = other_source_id();
        assert_task_249r_extension_error(
            &fixture,
            wrong_expression_source,
            &fixture.arena,
            SourceTypeError::InvalidDefinitionReturn {
                definition_return: SourceTypeDefinitionReturnId::new(0),
            },
        );
        let mut wrong_expression_module = fixture.extension.clone();
        wrong_expression_module.returns[0].expression.module_id = module("task249r.other");
        assert_task_249r_extension_error(
            &fixture,
            wrong_expression_module,
            &fixture.arena,
            SourceTypeError::InvalidDefinitionReturn {
                definition_return: SourceTypeDefinitionReturnId::new(0),
            },
        );
        for mutate in [
            |input: &mut SourceTypeDefinitionReturnExtensionInput| {
                input.returns[0].expression.source_range.start = 104;
            },
            |input: &mut SourceTypeDefinitionReturnExtensionInput| {
                input.returns[0].expression.head_range.start = 104;
            },
        ] {
            let mut invalid = fixture.extension.clone();
            mutate(&mut invalid);
            assert_task_249r_extension_error(
                &fixture,
                invalid,
                &fixture.arena,
                SourceTypeError::InvalidDefinitionReturn {
                    definition_return: SourceTypeDefinitionReturnId::new(0),
                },
            );
        }
        for mutate in [
            |input: &mut SourceTypeDefinitionReturnExtensionInput| {
                input.returns[0].expression.site = role(80, "return-expression");
            },
            |input: &mut SourceTypeDefinitionReturnExtensionInput| {
                input.returns[0].expression.head_site = role(79, "return-head");
            },
        ] {
            let mut invalid = fixture.extension.clone();
            mutate(&mut invalid);
            assert_task_249r_extension_error(
                &fixture,
                invalid,
                &fixture.arena,
                SourceTypeError::InvalidDefinitionReturnSite {
                    definition_return: SourceTypeDefinitionReturnId::new(0),
                },
            );
        }

        let mut unsupported = fixture.extension.clone();
        unsupported.returns[0].expression.form = SourceTypeApplicationForm::Of;
        assert!(matches!(
            SourceTypeDefinitionReturnProducer::extend(&fixture.base, unsupported, &fixture.arena),
            Err(SourceTypeError::UnsupportedDefinitionReturn { definition_return })
                if definition_return == SourceTypeDefinitionReturnId::new(0)
        ));
        for mutate in [
            |input: &mut SourceTypeDefinitionReturnExtensionInput| {
                input.returns[0].expression.spelling = "Set".to_owned();
            },
            |input: &mut SourceTypeDefinitionReturnExtensionInput| {
                input.returns[0].expression.head_spelling = "Set".to_owned();
            },
            |input: &mut SourceTypeDefinitionReturnExtensionInput| {
                input.returns[0].expression.head = SourceTypeHead::BuiltinObject;
            },
            |input: &mut SourceTypeDefinitionReturnExtensionInput| {
                input.returns[0].expression.recovery = NodeRecoveryState::Recovered;
            },
        ] {
            let mut invalid = fixture.extension.clone();
            mutate(&mut invalid);
            assert_task_249r_extension_error(
                &fixture,
                invalid,
                &fixture.arena,
                SourceTypeError::UnsupportedDefinitionReturn {
                    definition_return: SourceTypeDefinitionReturnId::new(0),
                },
            );
        }

        let mut duplicate_site = fixture.extension.clone();
        duplicate_site.returns[0].expression.site = node_site(84);
        assert!(matches!(
            SourceTypeDefinitionReturnProducer::extend(
                &fixture.base,
                duplicate_site,
                &fixture.arena,
            ),
            Err(SourceTypeError::InvalidDefinitionReturnSite { definition_return })
                if definition_return == SourceTypeDefinitionReturnId::new(0)
        ));

        let mut overlapping = fixture.extension.clone();
        overlapping.returns[1].definition_range = range(fixture.source, 110, 179);
        assert!(matches!(
            SourceTypeDefinitionReturnProducer::extend(&fixture.base, overlapping, &fixture.arena),
            Err(SourceTypeError::OverlappingDefinitionReturns { definition_return })
                if definition_return == SourceTypeDefinitionReturnId::new(1)
        ));
        let mut reordered_owner = fixture.extension.clone();
        reordered_owner.returns[1].definition_range = range(fixture.source, 50, 179);
        assert_task_249r_extension_error(
            &fixture,
            reordered_owner,
            &fixture.arena,
            SourceTypeError::OverlappingDefinitionReturns {
                definition_return: SourceTypeDefinitionReturnId::new(1),
            },
        );

        assert_eq!(fixture.base, baseline);
        assert!(fixture.base.definition_returns().is_empty());
    }

    #[test]
    fn task_249r_one_shot_base_environment_and_arena_drift_fail_closed() {
        let fixture = task_249r_fixture();
        let baseline = fixture.base.clone();
        let extended = SourceTypeDefinitionReturnProducer::extend(
            &fixture.base,
            fixture.extension.clone(),
            &fixture.arena,
        )
        .expect("first extension");
        assert_eq!(
            SourceTypeDefinitionReturnProducer::extend(
                &extended,
                fixture.extension.clone(),
                &fixture.arena,
            ),
            Err(SourceTypeError::DefinitionReturnsAlreadyPresent)
        );

        let mut wrong_source = fixture.extension.clone();
        wrong_source.source_id = other_source_id();
        assert_eq!(
            SourceTypeDefinitionReturnProducer::extend(&fixture.base, wrong_source, &fixture.arena,),
            Err(SourceTypeError::EnvironmentMismatch)
        );
        let mut wrong_module = fixture.extension.clone();
        wrong_module.module_id = module("task249r.wrong");
        assert_eq!(
            SourceTypeDefinitionReturnProducer::extend(&fixture.base, wrong_module, &fixture.arena,),
            Err(SourceTypeError::EnvironmentMismatch)
        );

        let mut invalid_base = fixture.base.clone();
        invalid_base.expressions.entries[0].spelling = "object".to_owned();
        assert_eq!(
            SourceTypeDefinitionReturnProducer::extend(
                &invalid_base,
                fixture.extension.clone(),
                &fixture.arena,
            ),
            Err(SourceTypeError::InvalidDefinitionReturnBase)
        );
        let base_mutations: [fn(&mut SourceTypeApplicationHandoff); 17] = [
            |base| base.applications.entries[0].id = SourceTypeApplicationId::new(1),
            |base| base.applications.entries[0].binding = BindingId::new(1),
            |base| base.applications.entries[0].source_ordinal = 1,
            |base| base.applications.entries[0].root = SourceTypeExpressionId::new(1),
            |base| base.expressions.entries[0].id = SourceTypeExpressionId::new(1),
            |base| base.expressions.entries[0].source_id = other_source_id(),
            |base| base.expressions.entries[0].module_id = module("task249r.other"),
            |base| base.expressions.entries[0].source_range.start = 21,
            |base| base.expressions.entries[0].site = node_site(64),
            |base| base.expressions.entries[0].spelling = "Set".to_owned(),
            |base| base.expressions.entries[0].head_site = node_site(61),
            |base| base.expressions.entries[0].head_range.start = 21,
            |base| base.expressions.entries[0].head_spelling = "Set".to_owned(),
            |base| base.expressions.entries[0].form = SourceTypeApplicationForm::Of,
            |base| base.expressions.entries[0].head = SourceTypeHead::BuiltinObject,
            |base| base.expressions.entries[0].recovery = NodeRecoveryState::Recovered,
            |base| {
                base.arguments.entries.push(SourceTypeArgumentRow {
                    id: SourceTypeArgumentId::new(0),
                    parent: SourceTypeExpressionId::new(0),
                    ordinal: 0,
                    argument: SourceTypeArgument::TypeSite {
                        expression: SourceTypeExpressionId::new(1),
                    },
                });
            },
        ];
        for mutate in base_mutations {
            let mut invalid = fixture.base.clone();
            mutate(&mut invalid);
            assert_eq!(
                SourceTypeDefinitionReturnProducer::extend(
                    &invalid,
                    fixture.extension.clone(),
                    &fixture.arena,
                ),
                Err(SourceTypeError::InvalidDefinitionReturnBase)
            );
        }
        let empty_arena = TypedArena::try_new(None, Vec::new()).expect("empty arena");
        assert_eq!(
            SourceTypeDefinitionReturnProducer::extend(
                &fixture.base,
                fixture.extension.clone(),
                &empty_arena,
            ),
            Err(SourceTypeError::InvalidDefinitionReturnBase)
        );
        let base_drift = task_249r_arena(
            fixture.source,
            Some((
                63,
                range(fixture.source, 22, 25),
                NodeRecoveryState::Recovered,
            )),
        );
        assert_eq!(
            SourceTypeDefinitionReturnProducer::extend(
                &fixture.base,
                fixture.extension.clone(),
                &base_drift,
            ),
            Err(SourceTypeError::InvalidDefinitionReturnBase)
        );
        let owner_drift = task_249r_arena(
            fixture.source,
            Some((
                84,
                range(fixture.source, 61, 117),
                NodeRecoveryState::Normal,
            )),
        );
        assert!(matches!(
            SourceTypeDefinitionReturnProducer::extend(
                &fixture.base,
                fixture.extension.clone(),
                &owner_drift,
            ),
            Err(SourceTypeError::InvalidDefinitionReturnSite { definition_return })
                if definition_return == SourceTypeDefinitionReturnId::new(0)
        ));
        for node in [80, 79] {
            let return_site_drift = task_249r_arena(
                fixture.source,
                Some((
                    node,
                    range(fixture.source, 105, 108),
                    NodeRecoveryState::Recovered,
                )),
            );
            assert!(matches!(
                SourceTypeDefinitionReturnProducer::extend(
                    &fixture.base,
                    fixture.extension.clone(),
                    &return_site_drift,
                ),
                Err(SourceTypeError::InvalidDefinitionReturnSite { definition_return })
                    if definition_return == SourceTypeDefinitionReturnId::new(0)
            ));
        }
        for (node, source_range, recovery) in [
            (
                84,
                range(fixture.source, 61, 117),
                NodeRecoveryState::Normal,
            ),
            (
                80,
                range(fixture.source, 105, 108),
                NodeRecoveryState::Recovered,
            ),
            (
                79,
                range(fixture.source, 105, 108),
                NodeRecoveryState::Recovered,
            ),
        ] {
            let drifted_arena =
                task_249r_arena(fixture.source, Some((node, source_range, recovery)));
            assert_eq!(
                task_249r_typed_ast(
                    fixture.source,
                    fixture.module.clone(),
                    extended.clone(),
                    drifted_arena,
                ),
                Err(TypedAstError::InvalidSourceType)
            );
        }
        assert_eq!(fixture.base, baseline);
        assert_eq!(extended.definition_returns().len(), 2);
    }

    #[test]
    fn task_249r_typed_final_clone_replay_has_no_semantic_output() {
        let fixture = task_249r_fixture();
        let handoff = SourceTypeDefinitionReturnProducer::extend(
            &fixture.base,
            fixture.extension,
            &fixture.arena,
        )
        .expect("definition returns");
        let fingerprint = handoff.debug_text();
        let typed = task_249r_typed_ast(
            fixture.source,
            fixture.module,
            handoff.clone(),
            fixture.arena,
        )
        .expect("typed Task 249R installation");
        assert_eq!(typed.source_type(), Some(&handoff));
        assert!(typed.types().is_empty());
        assert!(typed.facts().is_empty());
        assert!(typed.coercions().is_empty());
        assert!(typed.initial_obligations().is_empty());
        assert!(typed.diagnostics().is_empty());
        let typed_debug = typed.debug_text();
        assert_eq!(typed_debug.matches(fingerprint.as_str()).count(), 1);

        let resolved = assemble_empty_resolved(&typed);
        assert_eq!(resolved.source_type(), Some(&handoff));
        assert!(resolved.expr_metadata().is_empty());
        assert!(resolved.inserted_coercions().is_empty());
        assert!(resolved.cluster_facts().is_empty());
        assert!(resolved.diagnostics().is_empty());
        assert!(resolved.checked_formulas().is_empty());
        assert!(resolved.statement_semantics().is_empty());
        assert!(resolved.checked_proofs().is_empty());
        let resolved_debug = resolved.debug_text();
        assert_eq!(resolved_debug.matches(fingerprint.as_str()).count(), 1);
    }

    #[test]
    fn task_249m_exact_mode_rhs_extension_and_legacy_debug() {
        let fixture = task_249m_fixture();
        let legacy = fixture.base.debug_text();
        let extended = SourceTypeModeRhsProducer::extend(
            &fixture.base,
            fixture.extension.clone(),
            &fixture.arena,
        )
        .expect("exact mode RHS extension");

        assert!(fixture.base.definition_returns().is_empty());
        assert!(fixture.base.mode_rhs().is_empty());
        assert_eq!(fixture.base.debug_text(), legacy);
        assert_eq!(extended.applications().len(), 2);
        assert_eq!(extended.expressions().len(), 3);
        assert!(extended.arguments().is_empty());
        assert!(extended.definition_returns().is_empty());
        assert_eq!(extended.mode_rhs().len(), 1);
        assert_eq!(
            extended
                .mode_rhs()
                .iter()
                .map(|(id, row)| (
                    id.index(),
                    row.id().index(),
                    row.source_ordinal(),
                    row.root().index(),
                ))
                .collect::<Vec<_>>(),
            vec![(0, 0, 0, 2)]
        );
        let row = extended
            .mode_rhs()
            .get(SourceTypeModeRhsId::new(0))
            .expect("mode RHS row");
        assert_eq!(row.definition_site(), &node_site(49));
        assert_eq!(row.definition_range(), range(fixture.source, 45, 135));
        assert!(
            extended
                .mode_rhs()
                .get(SourceTypeModeRhsId::new(1))
                .is_none()
        );
        assert_eq!(
            extended.debug_text(),
            concat!(
                "source-type-application-debug-v1\n",
                "module: task249m.mode_definition\n",
                "application#0 binding=0 ordinal=0 root=0\n",
                "application#1 binding=1 ordinal=1 root=1\n",
                "mode-rhs#0 ordinal=0 definition_range=45..135 definition_site=node#49 root=2\n",
                "expression#0 form=bare range=22..25 site=node:35 head=builtin:set head_range=22..25 head_site=node:34 recovery=normal spelling=\"set\" head_spelling=\"set\"\n",
                "expression#1 form=bare range=38..41 site=node:39 head=builtin:set head_range=38..41 head_site=node:38 recovery=normal spelling=\"set\" head_spelling=\"set\"\n",
                "expression#2 form=bare range=95..98 site=node:44 head=builtin:set head_range=95..98 head_site=node:43 recovery=normal spelling=\"set\" head_spelling=\"set\"\n",
            )
        );

        let task_249r = task_249r_fixture();
        let task_249r_extended = SourceTypeDefinitionReturnProducer::extend(
            &task_249r.base,
            task_249r.extension,
            &task_249r.arena,
        )
        .expect("Task 249R remains accepted");
        assert!(task_249r_extended.mode_rhs().is_empty());
        assert!(!task_249r_extended.debug_text().contains("mode-rhs#"));
    }

    #[test]
    fn task_249m_mode_rhs_corruption_fails_atomically() {
        let fixture = task_249m_fixture();
        let mode_rhs = SourceTypeModeRhsId::new(0);

        let mut empty = fixture.extension.clone();
        empty.rhs.clear();
        assert_task_249m_extension_error(
            &fixture,
            empty.clone(),
            &fixture.arena,
            SourceTypeError::EmptyModeRhs,
        );

        let mut multiple = fixture.extension.clone();
        multiple.rhs.push(multiple.rhs[0].clone());
        assert_task_249m_extension_error(
            &fixture,
            multiple,
            &fixture.arena,
            SourceTypeError::ModeRhsCardinalityMismatch,
        );

        for wrong_environment in [
            {
                let mut input = fixture.extension.clone();
                input.source_id = other_source_id();
                input
            },
            {
                let mut input = fixture.extension.clone();
                input.module_id = module("task249m.other");
                input
            },
        ] {
            assert_task_249m_extension_error(
                &fixture,
                wrong_environment,
                &fixture.arena,
                SourceTypeError::EnvironmentMismatch,
            );
        }

        let invalid_rows: [fn(&mut SourceTypeModeRhsExtensionInput); 8] = [
            |input| input.rhs[0].source_ordinal = 1,
            |input| input.rhs[0].definition_range.start = 44,
            |input| input.rhs[0].definition_range.end = 134,
            |input| input.rhs[0].expression.source_id = other_source_id(),
            |input| input.rhs[0].expression.module_id = module("task249m.other"),
            |input| input.rhs[0].expression.source_range.start = 94,
            |input| input.rhs[0].expression.head_range.start = 94,
            |input| input.rhs[0].expression.source_range.source_id = other_source_id(),
        ];
        for mutate in invalid_rows {
            let mut invalid = fixture.extension.clone();
            mutate(&mut invalid);
            assert_task_249m_extension_error(
                &fixture,
                invalid,
                &fixture.arena,
                SourceTypeError::InvalidModeRhs { mode_rhs },
            );
        }

        let invalid_sites: [fn(&mut SourceTypeModeRhsExtensionInput); 5] = [
            |input| input.rhs[0].definition_site = role(49, "mode-owner"),
            |input| input.rhs[0].definition_site = node_site(48),
            |input| input.rhs[0].expression.site = role(44, "mode-rhs"),
            |input| input.rhs[0].expression.head_site = role(43, "mode-rhs-head"),
            |input| input.rhs[0].expression.site = node_site(49),
        ];
        for mutate in invalid_sites {
            let mut invalid = fixture.extension.clone();
            mutate(&mut invalid);
            assert_task_249m_extension_error(
                &fixture,
                invalid,
                &fixture.arena,
                SourceTypeError::InvalidModeRhsSite { mode_rhs },
            );
        }

        let unsupported: [fn(&mut SourceTypeModeRhsExtensionInput); 5] = [
            |input| input.rhs[0].expression.form = SourceTypeApplicationForm::Of,
            |input| input.rhs[0].expression.head = SourceTypeHead::BuiltinObject,
            |input| input.rhs[0].expression.spelling = "Set".to_owned(),
            |input| input.rhs[0].expression.head_spelling = "Set".to_owned(),
            |input| input.rhs[0].expression.recovery = NodeRecoveryState::Recovered,
        ];
        for mutate in unsupported {
            let mut invalid = fixture.extension.clone();
            mutate(&mut invalid);
            assert_task_249m_extension_error(
                &fixture,
                invalid,
                &fixture.arena,
                SourceTypeError::UnsupportedModeRhs { mode_rhs },
            );
        }

        let extended = SourceTypeModeRhsProducer::extend(
            &fixture.base,
            fixture.extension.clone(),
            &fixture.arena,
        )
        .expect("mode RHS");
        assert_eq!(
            SourceTypeModeRhsProducer::extend(&extended, empty, &fixture.arena),
            Err(SourceTypeError::ModeRhsAlreadyPresent)
        );

        let mut cardinality_over_environment = fixture.extension.clone();
        cardinality_over_environment
            .rhs
            .push(cardinality_over_environment.rhs[0].clone());
        cardinality_over_environment.source_id = other_source_id();
        assert_task_249m_extension_error(
            &fixture,
            cardinality_over_environment,
            &fixture.arena,
            SourceTypeError::ModeRhsCardinalityMismatch,
        );

        let mut invalid_base = fixture.base.clone();
        invalid_base.applications.entries[0].source_ordinal = 1;
        let mut environment_over_base = fixture.extension.clone();
        environment_over_base.source_id = other_source_id();
        assert_eq!(
            SourceTypeModeRhsProducer::extend(&invalid_base, environment_over_base, &fixture.arena,),
            Err(SourceTypeError::EnvironmentMismatch)
        );

        let mut base_over_row = fixture.extension.clone();
        base_over_row.rhs[0].source_ordinal = 1;
        assert_eq!(
            SourceTypeModeRhsProducer::extend(&invalid_base, base_over_row, &fixture.arena),
            Err(SourceTypeError::InvalidModeRhsBase)
        );

        let mut row_over_site = fixture.extension.clone();
        row_over_site.rhs[0].source_ordinal = 1;
        row_over_site.rhs[0].definition_site = role(49, "mode-owner");
        assert_task_249m_extension_error(
            &fixture,
            row_over_site,
            &fixture.arena,
            SourceTypeError::InvalidModeRhs { mode_rhs },
        );

        let mut site_over_unsupported = fixture.extension.clone();
        site_over_unsupported.rhs[0].definition_site = role(49, "mode-owner");
        site_over_unsupported.rhs[0].expression.form = SourceTypeApplicationForm::Of;
        assert_task_249m_extension_error(
            &fixture,
            site_over_unsupported,
            &fixture.arena,
            SourceTypeError::InvalidModeRhsSite { mode_rhs },
        );
        assert_eq!(fixture.base.mode_rhs().len(), 0);
    }

    #[test]
    fn task_249m_one_shot_base_and_arena_drift_fail_closed() {
        let fixture = task_249m_fixture();
        let baseline = fixture.base.clone();
        let extended = SourceTypeModeRhsProducer::extend(
            &fixture.base,
            fixture.extension.clone(),
            &fixture.arena,
        )
        .expect("first extension");
        assert_eq!(
            SourceTypeModeRhsProducer::extend(&extended, fixture.extension.clone(), &fixture.arena,),
            Err(SourceTypeError::ModeRhsAlreadyPresent)
        );

        let base_mutations: [fn(&mut SourceTypeApplicationHandoff); 18] = [
            |base| base.applications.entries[0].id = SourceTypeApplicationId::new(1),
            |base| base.applications.entries[0].binding = BindingId::new(1),
            |base| base.applications.entries[0].source_ordinal = 1,
            |base| base.applications.entries[0].root = SourceTypeExpressionId::new(1),
            |base| base.expressions.entries[0].id = SourceTypeExpressionId::new(1),
            |base| base.expressions.entries[0].source_id = other_source_id(),
            |base| base.expressions.entries[0].module_id = module("task249m.other"),
            |base| base.expressions.entries[0].source_range.start = 21,
            |base| base.expressions.entries[0].site = node_site(36),
            |base| base.expressions.entries[0].spelling = "Set".to_owned(),
            |base| base.expressions.entries[0].head_site = node_site(33),
            |base| base.expressions.entries[0].head_range.start = 21,
            |base| base.expressions.entries[0].head_spelling = "Set".to_owned(),
            |base| base.expressions.entries[0].form = SourceTypeApplicationForm::Of,
            |base| base.expressions.entries[0].head = SourceTypeHead::BuiltinObject,
            |base| base.expressions.entries[0].recovery = NodeRecoveryState::Recovered,
            |base| {
                base.expressions
                    .entries
                    .push(base.expressions.entries[0].clone())
            },
            |base| {
                base.arguments.entries.push(SourceTypeArgumentRow {
                    id: SourceTypeArgumentId::new(0),
                    parent: SourceTypeExpressionId::new(0),
                    ordinal: 0,
                    argument: SourceTypeArgument::TypeSite {
                        expression: SourceTypeExpressionId::new(1),
                    },
                });
            },
        ];
        for mutate in base_mutations {
            let mut invalid = fixture.base.clone();
            mutate(&mut invalid);
            assert_eq!(
                SourceTypeModeRhsProducer::extend(
                    &invalid,
                    fixture.extension.clone(),
                    &fixture.arena,
                ),
                Err(SourceTypeError::InvalidModeRhsBase)
            );
        }

        let empty_arena = TypedArena::try_new(None, Vec::new()).expect("empty arena");
        assert_task_249m_extension_error(
            &fixture,
            fixture.extension.clone(),
            &empty_arena,
            SourceTypeError::InvalidModeRhsBase,
        );
        let base_drift = task_249m_arena(
            fixture.source,
            Some((
                35,
                range(fixture.source, 22, 25),
                NodeRecoveryState::Recovered,
            )),
        );
        assert_task_249m_extension_error(
            &fixture,
            fixture.extension.clone(),
            &base_drift,
            SourceTypeError::InvalidModeRhsBase,
        );

        for (node, source_range, recovery) in [
            (
                49,
                range(fixture.source, 45, 134),
                NodeRecoveryState::Normal,
            ),
            (
                44,
                range(fixture.source, 95, 98),
                NodeRecoveryState::Recovered,
            ),
            (
                43,
                range(fixture.source, 95, 98),
                NodeRecoveryState::Recovered,
            ),
        ] {
            let drifted = task_249m_arena(fixture.source, Some((node, source_range, recovery)));
            assert_task_249m_extension_error(
                &fixture,
                fixture.extension.clone(),
                &drifted,
                SourceTypeError::InvalidModeRhsSite {
                    mode_rhs: SourceTypeModeRhsId::new(0),
                },
            );
            assert_eq!(
                task_249r_typed_ast(
                    fixture.source,
                    fixture.module.clone(),
                    extended.clone(),
                    drifted,
                ),
                Err(TypedAstError::InvalidSourceType)
            );
        }

        for mutate in [
            |handoff: &mut SourceTypeApplicationHandoff| {
                handoff.mode_rhs.entries[0].id = SourceTypeModeRhsId::new(1);
            },
            |handoff: &mut SourceTypeApplicationHandoff| {
                handoff.mode_rhs.entries[0].root = SourceTypeExpressionId::new(1);
            },
            |handoff: &mut SourceTypeApplicationHandoff| {
                handoff.expressions.entries[2].id = SourceTypeExpressionId::new(1);
            },
        ] {
            let mut corrupt = extended.clone();
            mutate(&mut corrupt);
            assert_eq!(
                task_249r_typed_ast(
                    fixture.source,
                    fixture.module.clone(),
                    corrupt,
                    fixture.arena.clone(),
                ),
                Err(TypedAstError::InvalidSourceType)
            );
        }
        assert_eq!(fixture.base, baseline);
        assert_eq!(extended.mode_rhs().len(), 1);
    }

    #[test]
    fn task_249m_typed_final_clone_replay_and_task_249r_isolation() {
        let fixture = task_249m_fixture();
        let baseline = fixture.base.clone();
        let handoff = SourceTypeModeRhsProducer::extend(
            &fixture.base,
            fixture.extension.clone(),
            &fixture.arena,
        )
        .expect("mode RHS");
        let fingerprint = handoff.debug_text();
        let replay = SourceTypeModeRhsProducer::extend(
            &fixture.base,
            fixture.extension.clone(),
            &fixture.arena,
        )
        .expect("clean-base mode RHS replay");
        assert_eq!(replay, handoff);
        assert_eq!(replay.debug_text(), fingerprint);
        assert_eq!(fixture.base, baseline);
        let typed = task_249r_typed_ast(
            fixture.source,
            fixture.module.clone(),
            handoff.clone(),
            fixture.arena.clone(),
        )
        .expect("typed Task 249M installation");
        assert_eq!(typed.source_type(), Some(&handoff));
        assert!(typed.types().is_empty());
        assert!(typed.facts().is_empty());
        assert!(typed.coercions().is_empty());
        assert!(typed.initial_obligations().is_empty());
        assert!(typed.diagnostics().is_empty());
        assert_eq!(typed.debug_text().matches(fingerprint.as_str()).count(), 1);

        let resolved = assemble_empty_resolved(&typed);
        assert_eq!(resolved.source_type(), Some(&handoff));
        assert!(resolved.expr_metadata().is_empty());
        assert!(resolved.inserted_coercions().is_empty());
        assert!(resolved.cluster_facts().is_empty());
        assert!(resolved.diagnostics().is_empty());
        assert!(resolved.checked_formulas().is_empty());
        assert!(resolved.statement_semantics().is_empty());
        assert!(resolved.checked_proofs().is_empty());
        assert_eq!(
            resolved.debug_text().matches(fingerprint.as_str()).count(),
            1
        );
        assert_eq!(
            SourceTypeModeRhsProducer::extend(
                &handoff.clone(),
                fixture.extension.clone(),
                &fixture.arena,
            ),
            Err(SourceTypeError::ModeRhsAlreadyPresent)
        );

        let task_249r = task_249r_fixture();
        let task_249r_handoff = SourceTypeDefinitionReturnProducer::extend(
            &task_249r.base,
            task_249r.extension.clone(),
            &task_249r.arena,
        )
        .expect("Task 249R extension");
        assert!(task_249r_handoff.mode_rhs().is_empty());

        let mut return_after_mode = task_249r.extension.clone();
        return_after_mode.source_id = fixture.source;
        return_after_mode.module_id = fixture.module.clone();
        assert_eq!(
            SourceTypeDefinitionReturnProducer::extend(&handoff, return_after_mode, &fixture.arena,),
            Err(SourceTypeError::InvalidDefinitionReturnBase)
        );

        let mut mode_after_return = fixture.extension.clone();
        mode_after_return.source_id = task_249r.source;
        mode_after_return.module_id = task_249r.module.clone();
        assert_eq!(
            SourceTypeModeRhsProducer::extend(
                &task_249r_handoff,
                mode_after_return,
                &task_249r.arena,
            ),
            Err(SourceTypeError::InvalidModeRhsBase)
        );
    }

    #[test]
    fn task_249s_exact_structure_member_build_and_legacy_debug() {
        let task_fixture = task_249s_fixture();
        let handoff = SourceTypeStructureMemberProducer::build(
            task_fixture.input.clone(),
            &task_fixture.arena,
        )
        .expect("exact structure-member handoff");

        assert!(handoff.applications().is_empty());
        assert_eq!(handoff.expressions().len(), 4);
        assert!(handoff.arguments().is_empty());
        assert!(handoff.definition_returns().is_empty());
        assert!(handoff.mode_rhs().is_empty());
        assert_eq!(handoff.structure_members().len(), 4);
        assert_eq!(
            handoff
                .structure_members()
                .iter()
                .map(|(id, row)| (
                    id.index(),
                    row.id().index(),
                    row.source_ordinal(),
                    row.root().index(),
                    row.member_site().clone(),
                    row.member_range(),
                ))
                .collect::<Vec<_>>(),
            vec![
                (
                    0,
                    0,
                    0,
                    0,
                    node_site(53),
                    range(task_fixture.source, 42, 63)
                ),
                (
                    1,
                    1,
                    1,
                    1,
                    node_site(56),
                    range(task_fixture.source, 68, 91)
                ),
                (
                    2,
                    2,
                    2,
                    2,
                    node_site(61),
                    range(task_fixture.source, 134, 155),
                ),
                (
                    3,
                    3,
                    3,
                    3,
                    node_site(64),
                    range(task_fixture.source, 160, 183),
                ),
            ]
        );
        assert!(
            handoff
                .structure_members()
                .get(SourceTypeStructureMemberId::new(4))
                .is_none()
        );
        assert_eq!(
            handoff.debug_text(),
            concat!(
                "source-type-application-debug-v1\n",
                "module: task249s.structure_definition\n",
                "structure-member#0 ordinal=0 member_range=42..63 member_site=node#53 root=0\n",
                "structure-member#1 ordinal=1 member_range=68..91 member_site=node#56 root=1\n",
                "structure-member#2 ordinal=2 member_range=134..155 member_site=node#61 root=2\n",
                "structure-member#3 ordinal=3 member_range=160..183 member_site=node#64 root=3\n",
                "expression#0 form=bare range=59..62 site=node:52 head=builtin:set head_range=59..62 head_site=node:51 recovery=normal spelling=\"set\" head_spelling=\"set\"\n",
                "expression#1 form=bare range=87..90 site=node:55 head=builtin:set head_range=87..90 head_site=node:54 recovery=normal spelling=\"set\" head_spelling=\"set\"\n",
                "expression#2 form=bare range=151..154 site=node:60 head=builtin:set head_range=151..154 head_site=node:59 recovery=normal spelling=\"set\" head_spelling=\"set\"\n",
                "expression#3 form=bare range=179..182 site=node:63 head=builtin:set head_range=179..182 head_site=node:62 recovery=normal spelling=\"set\" head_spelling=\"set\"\n",
            )
        );

        let legacy_fixture = fixture();
        let legacy = build(&legacy_fixture).expect("legacy source type");
        assert!(legacy.structure_members().is_empty());
        assert!(!legacy.debug_text().contains("structure-member#"));
    }

    #[test]
    fn task_249s_member_corruption_fails_atomically() {
        let fixture = task_249s_fixture();

        let mut empty = fixture.input.clone();
        empty.members.clear();
        assert_task_249s_build_error(
            &empty,
            &fixture.arena,
            SourceTypeError::EmptyStructureMembers,
        );

        for cardinality in [3, 5] {
            let mut invalid = fixture.input.clone();
            if cardinality == 3 {
                invalid.members.pop();
            } else {
                invalid.members.push(invalid.members[0].clone());
            }
            assert_task_249s_build_error(
                &invalid,
                &fixture.arena,
                SourceTypeError::StructureMemberCardinalityMismatch,
            );
        }

        let invalid_rows: [fn(&mut SourceTypeStructureMemberHandoffInput); 10] = [
            |input| input.source_id = other_source_id(),
            |input| input.module_id = module("task249s.other"),
            |input| input.members[0].source_ordinal = 1,
            |input| input.members[0].member_range.start = 41,
            |input| input.members[0].member_range.end = 62,
            |input| input.members[0].member_range.source_id = other_source_id(),
            |input| input.members[0].expression.source_id = other_source_id(),
            |input| input.members[0].expression.module_id = module("task249s.other"),
            |input| input.members[0].expression.source_range.start = 58,
            |input| input.members[0].expression.head_range.start = 58,
        ];
        for mutate in invalid_rows {
            let mut invalid = fixture.input.clone();
            mutate(&mut invalid);
            assert_task_249s_build_error(
                &invalid,
                &fixture.arena,
                SourceTypeError::InvalidStructureMember {
                    structure_member: SourceTypeStructureMemberId::new(0),
                },
            );
        }

        let invalid_sites: [fn(&mut SourceTypeStructureMemberHandoffInput); 5] = [
            |input| input.members[0].member_site = role(53, "member"),
            |input| input.members[0].member_site = node_site(52),
            |input| input.members[0].expression.site = role(52, "expression"),
            |input| input.members[0].expression.head_site = role(51, "head"),
            |input| input.members[0].expression.site = node_site(53),
        ];
        for mutate in invalid_sites {
            let mut invalid = fixture.input.clone();
            mutate(&mut invalid);
            assert_task_249s_build_error(
                &invalid,
                &fixture.arena,
                SourceTypeError::InvalidStructureMemberSite {
                    structure_member: SourceTypeStructureMemberId::new(0),
                },
            );
        }
        let mut duplicate_across_rows = fixture.input.clone();
        duplicate_across_rows.members[1].member_site =
            duplicate_across_rows.members[0].member_site.clone();
        assert_task_249s_build_error(
            &duplicate_across_rows,
            &fixture.arena,
            SourceTypeError::InvalidStructureMemberSite {
                structure_member: SourceTypeStructureMemberId::new(1),
            },
        );
        let mut swapped_expression_and_head = fixture.input.clone();
        let expression_site = swapped_expression_and_head.members[0]
            .expression
            .site
            .clone();
        swapped_expression_and_head.members[0].expression.site = swapped_expression_and_head
            .members[0]
            .expression
            .head_site
            .clone();
        swapped_expression_and_head.members[0].expression.head_site = expression_site;
        assert_task_249s_build_error(
            &swapped_expression_and_head,
            &fixture.arena,
            SourceTypeError::InvalidStructureMemberSite {
                structure_member: SourceTypeStructureMemberId::new(0),
            },
        );

        let unsupported: [fn(&mut SourceTypeStructureMemberHandoffInput); 5] = [
            |input| input.members[0].expression.form = SourceTypeApplicationForm::Of,
            |input| input.members[0].expression.head = SourceTypeHead::BuiltinObject,
            |input| input.members[0].expression.spelling = "Set".to_owned(),
            |input| input.members[0].expression.head_spelling = "Set".to_owned(),
            |input| input.members[0].expression.recovery = NodeRecoveryState::Recovered,
        ];
        for mutate in unsupported {
            let mut invalid = fixture.input.clone();
            mutate(&mut invalid);
            assert_task_249s_build_error(
                &invalid,
                &fixture.arena,
                SourceTypeError::UnsupportedStructureMember {
                    structure_member: SourceTypeStructureMemberId::new(0),
                },
            );
        }

        let mut cardinality_over_row = fixture.input.clone();
        cardinality_over_row.members.pop();
        cardinality_over_row.members[0].source_ordinal = 1;
        assert_task_249s_build_error(
            &cardinality_over_row,
            &fixture.arena,
            SourceTypeError::StructureMemberCardinalityMismatch,
        );
        let mut row_over_site = fixture.input.clone();
        row_over_site.members[0].source_ordinal = 1;
        row_over_site.members[0].member_site = role(53, "member");
        assert_task_249s_build_error(
            &row_over_site,
            &fixture.arena,
            SourceTypeError::InvalidStructureMember {
                structure_member: SourceTypeStructureMemberId::new(0),
            },
        );
        let mut site_over_shape = fixture.input.clone();
        site_over_shape.members[0].member_site = role(53, "member");
        site_over_shape.members[0].expression.form = SourceTypeApplicationForm::Of;
        assert_task_249s_build_error(
            &site_over_shape,
            &fixture.arena,
            SourceTypeError::InvalidStructureMemberSite {
                structure_member: SourceTypeStructureMemberId::new(0),
            },
        );
        let mut later_row_over_earlier_site = fixture.input.clone();
        later_row_over_earlier_site.members[0].member_site = role(53, "member");
        later_row_over_earlier_site.members[1].source_ordinal = 2;
        assert_task_249s_build_error(
            &later_row_over_earlier_site,
            &fixture.arena,
            SourceTypeError::InvalidStructureMember {
                structure_member: SourceTypeStructureMemberId::new(1),
            },
        );
        let mut later_row_over_earlier_shape = fixture.input.clone();
        later_row_over_earlier_shape.members[0].expression.form = SourceTypeApplicationForm::Of;
        later_row_over_earlier_shape.members[1].source_ordinal = 2;
        assert_task_249s_build_error(
            &later_row_over_earlier_shape,
            &fixture.arena,
            SourceTypeError::InvalidStructureMember {
                structure_member: SourceTypeStructureMemberId::new(1),
            },
        );
        let mut later_site_over_earlier_shape = fixture.input.clone();
        later_site_over_earlier_shape.members[0].expression.form = SourceTypeApplicationForm::Of;
        later_site_over_earlier_shape.members[1].member_site = role(56, "member");
        assert_task_249s_build_error(
            &later_site_over_earlier_shape,
            &fixture.arena,
            SourceTypeError::InvalidStructureMemberSite {
                structure_member: SourceTypeStructureMemberId::new(1),
            },
        );
        assert_eq!(fixture.input.members.len(), 4);
    }

    #[test]
    fn task_249s_arena_and_installation_drift_fail_closed() {
        let fixture = task_249s_fixture();
        let handoff =
            SourceTypeStructureMemberProducer::build(fixture.input.clone(), &fixture.arena)
                .expect("structure members");

        let empty_arena = TypedArena::try_new(None, Vec::new()).expect("empty arena");
        assert_task_249s_build_error(
            &fixture.input,
            &empty_arena,
            SourceTypeError::InvalidStructureMemberSite {
                structure_member: SourceTypeStructureMemberId::new(0),
            },
        );

        for (index, (member, _, _, expression, head, start, end)) in
            TASK_249S_STRUCTURE_MEMBERS.into_iter().enumerate()
        {
            for node in [member, expression, head] {
                let exact_range = if node == member {
                    fixture.input.members[index].member_range
                } else {
                    range(fixture.source, start, end)
                };
                for (source_range, recovery) in [
                    (exact_range, NodeRecoveryState::Recovered),
                    (range(fixture.source, 0, 1), NodeRecoveryState::Normal),
                ] {
                    let drifted =
                        task_249s_arena(fixture.source, Some((node, source_range, recovery)));
                    assert_task_249s_build_error(
                        &fixture.input,
                        &drifted,
                        SourceTypeError::InvalidStructureMemberSite {
                            structure_member: SourceTypeStructureMemberId::new(index),
                        },
                    );
                    assert_eq!(
                        task_249r_typed_ast(
                            fixture.source,
                            fixture.module.clone(),
                            handoff.clone(),
                            drifted,
                        ),
                        Err(TypedAstError::InvalidSourceType)
                    );
                }
            }
        }

        let corruptions: [fn(&mut SourceTypeApplicationHandoff); 9] = [
            |value| value.structure_members.entries.clear(),
            |value| value.structure_members.entries[0].id = SourceTypeStructureMemberId::new(1),
            |value| value.structure_members.entries[0].root = SourceTypeExpressionId::new(1),
            |value| value.expressions.entries[0].id = SourceTypeExpressionId::new(1),
            |value| value.expressions.entries.pop().map(|_| ()).unwrap_or(()),
            |value| {
                value.applications.entries.push(SourceTypeApplication {
                    id: SourceTypeApplicationId::new(0),
                    binding: BindingId::new(0),
                    source_ordinal: 0,
                    root: SourceTypeExpressionId::new(0),
                });
            },
            |value| {
                value.arguments.entries.push(SourceTypeArgumentRow {
                    id: SourceTypeArgumentId::new(0),
                    parent: SourceTypeExpressionId::new(0),
                    ordinal: 0,
                    argument: SourceTypeArgument::TypeSite {
                        expression: SourceTypeExpressionId::new(1),
                    },
                });
            },
            |value| {
                value
                    .definition_returns
                    .entries
                    .push(SourceTypeDefinitionReturn {
                        id: SourceTypeDefinitionReturnId::new(0),
                        definition_site: node_site(53),
                        definition_range: range(value.source_id, 42, 63),
                        source_ordinal: 0,
                        root: SourceTypeExpressionId::new(0),
                    });
            },
            |value| {
                value.mode_rhs.entries.push(SourceTypeModeRhs {
                    id: SourceTypeModeRhsId::new(0),
                    definition_site: node_site(53),
                    definition_range: range(value.source_id, 42, 63),
                    source_ordinal: 0,
                    root: SourceTypeExpressionId::new(0),
                });
            },
        ];
        for corrupt in corruptions {
            let mut invalid = handoff.clone();
            corrupt(&mut invalid);
            assert_eq!(
                task_249r_typed_ast(
                    fixture.source,
                    fixture.module.clone(),
                    invalid,
                    fixture.arena.clone(),
                ),
                Err(TypedAstError::InvalidSourceType)
            );
        }
        assert_eq!(handoff.structure_members().len(), 4);
    }

    #[test]
    fn task_249s_typed_final_replay_and_sibling_isolation() {
        let fixture = task_249s_fixture();
        let handoff =
            SourceTypeStructureMemberProducer::build(fixture.input.clone(), &fixture.arena)
                .expect("structure members");
        let fingerprint = handoff.debug_text();
        let replay =
            SourceTypeStructureMemberProducer::build(fixture.input.clone(), &fixture.arena)
                .expect("structure-member replay");
        assert_eq!(replay, handoff);
        assert_eq!(replay.debug_text(), fingerprint);

        let typed = task_249r_typed_ast(
            fixture.source,
            fixture.module.clone(),
            handoff.clone(),
            fixture.arena.clone(),
        )
        .expect("typed Task 249S installation");
        assert_eq!(typed.source_type(), Some(&handoff));
        assert!(typed.types().is_empty());
        assert!(typed.facts().is_empty());
        assert!(typed.coercions().is_empty());
        assert!(typed.initial_obligations().is_empty());
        assert!(typed.diagnostics().is_empty());
        assert_eq!(typed.debug_text().matches(fingerprint.as_str()).count(), 1);

        let resolved = assemble_empty_resolved(&typed);
        let resolved_replay = assemble_empty_resolved(&typed);
        assert_eq!(resolved_replay, resolved);
        assert_eq!(resolved_replay.debug_text(), resolved.debug_text());
        assert_eq!(resolved.source_type(), Some(&handoff));
        assert!(resolved.expr_metadata().is_empty());
        assert!(resolved.inserted_coercions().is_empty());
        assert!(resolved.cluster_facts().is_empty());
        assert!(resolved.diagnostics().is_empty());
        assert!(resolved.checked_formulas().is_empty());
        assert!(resolved.statement_semantics().is_empty());
        assert!(resolved.checked_proofs().is_empty());
        assert_eq!(
            resolved.debug_text().matches(fingerprint.as_str()).count(),
            1
        );

        let task_249r = task_249r_fixture();
        let task_249r_handoff = SourceTypeDefinitionReturnProducer::extend(
            &task_249r.base,
            task_249r.extension.clone(),
            &task_249r.arena,
        )
        .expect("Task 249R");
        assert!(task_249r_handoff.structure_members().is_empty());
        let mut return_after_members = task_249r.extension;
        return_after_members.source_id = fixture.source;
        return_after_members.module_id = fixture.module.clone();
        assert_eq!(
            SourceTypeDefinitionReturnProducer::extend(
                &handoff,
                return_after_members,
                &fixture.arena,
            ),
            Err(SourceTypeError::InvalidDefinitionReturnBase)
        );

        let task_249m = task_249m_fixture();
        let task_249m_handoff = SourceTypeModeRhsProducer::extend(
            &task_249m.base,
            task_249m.extension.clone(),
            &task_249m.arena,
        )
        .expect("Task 249M");
        assert!(task_249m_handoff.structure_members().is_empty());
        let mut mode_after_members = task_249m.extension;
        mode_after_members.source_id = fixture.source;
        mode_after_members.module_id = fixture.module;
        assert_eq!(
            SourceTypeModeRhsProducer::extend(&handoff, mode_after_members, &fixture.arena),
            Err(SourceTypeError::InvalidModeRhsBase)
        );
    }

    #[test]
    fn task_249pi_exact_means_and_equals_extensions_and_debug() {
        let legacy_fixture = task_249s_fixture();
        let mut legacy_non_four = legacy_fixture.input.clone();
        legacy_non_four.members.truncate(2);
        assert_eq!(
            SourceTypeStructureMemberProducer::build(legacy_non_four, &legacy_fixture.arena),
            Err(SourceTypeError::StructureMemberCardinalityMismatch)
        );
        let legacy_before = SourceTypeStructureMemberProducer::build(
            legacy_fixture.input.clone(),
            &legacy_fixture.arena,
        )
        .expect("legacy Task 249S");
        let legacy_fingerprint = legacy_before.debug_text();

        for profile in [Task249PiProfile::Means, Task249PiProfile::Equals] {
            let fixture = task_249pi_fixture(profile);
            let baseline = fixture.base.clone();
            let handoff = SourceTypeStructureMemberProducer::extend_property_implementation(
                &fixture.base,
                fixture.extension.clone(),
                &fixture.arena,
            )
            .expect("Task 249PI extension");
            assert_eq!(fixture.base, baseline);
            assert_eq!(handoff.applications().len(), 1);
            assert_eq!(handoff.expressions().len(), 3);
            assert!(handoff.arguments().is_empty());
            assert!(handoff.definition_returns().is_empty());
            assert!(handoff.mode_rhs().is_empty());
            assert_eq!(handoff.structure_members().len(), 2);

            let (parameter_sites, members) = match profile {
                Task249PiProfile::Means => (
                    TASK_249PI_MEANS_PARAMETER_SITES,
                    TASK_249PI_MEANS_STRUCTURE_MEMBERS,
                ),
                Task249PiProfile::Equals => (
                    TASK_249PI_EQUALS_PARAMETER_SITES,
                    TASK_249PI_EQUALS_STRUCTURE_MEMBERS,
                ),
            };
            assert_eq!(
                handoff
                    .structure_members()
                    .iter()
                    .map(|(id, row)| (
                        id.index(),
                        row.id().index(),
                        row.source_ordinal(),
                        row.root().index(),
                        row.member_site().clone(),
                        row.member_range(),
                    ))
                    .collect::<Vec<_>>(),
                members
                    .into_iter()
                    .enumerate()
                    .map(|(index, (member, member_start, member_end, _, _, _, _))| (
                        index,
                        index,
                        index,
                        index + 1,
                        node_site(member),
                        range(fixture.source, member_start, member_end),
                    ),)
                    .collect::<Vec<_>>()
            );

            let parameter = handoff
                .expressions()
                .get(SourceTypeExpressionId::new(0))
                .expect("parameter expression");
            let SourceTypeHead::Symbol {
                symbol,
                contribution,
            } = parameter.head()
            else {
                panic!("Task 249PI parameter symbol");
            };
            assert_eq!(contribution.index(), 0);
            let expected = format!(
                concat!(
                    "source-type-application-debug-v1\n",
                    "module: {}\n",
                    "application#0 binding=0 ordinal=0 root=0\n",
                    "structure-member#0 ordinal=0 member_range=45..66 member_site=node#{} root=1\n",
                    "structure-member#1 ordinal=1 member_range=71..94 member_site=node#{} root=2\n",
                    "expression#0 form=bare range=130..144 site=node:{} head=symbol:{}:contribution:0 head_range=130..144 head_site=node:{} recovery=normal spelling=\"Task264Carrier\" head_spelling=\"Task264Carrier\"\n",
                    "expression#1 form=bare range=62..65 site=node:{} head=builtin:set head_range=62..65 head_site=node:{} recovery=normal spelling=\"set\" head_spelling=\"set\"\n",
                    "expression#2 form=bare range=90..93 site=node:{} head=builtin:set head_range=90..93 head_site=node:{} recovery=normal spelling=\"set\" head_spelling=\"set\"\n",
                ),
                fixture.module.path().as_str(),
                members[0].0,
                members[1].0,
                parameter_sites.0,
                symbol.fqn().as_str(),
                parameter_sites.1,
                members[0].3,
                members[0].4,
                members[1].3,
                members[1].4,
            );
            assert_eq!(handoff.debug_text(), expected);
            assert!(handoff.debug_text().ends_with('\n'));
            assert!(!handoff.debug_text().ends_with("\n\n"));
        }

        let legacy_after =
            SourceTypeStructureMemberProducer::build(legacy_fixture.input, &legacy_fixture.arena)
                .expect("legacy Task 249S replay");
        assert_eq!(legacy_after, legacy_before);
        assert_eq!(legacy_after.debug_text(), legacy_fingerprint);
        assert_eq!(
            SourceTypeError::StructureMembersAlreadyPresent.to_string(),
            "source type structure members are already installed"
        );
        assert_eq!(
            SourceTypeError::StructureMemberExtensionCardinalityMismatch.to_string(),
            "source type structure-member extension cardinality is not the frozen pair"
        );
        assert_eq!(
            SourceTypeError::InvalidStructureMemberBase.to_string(),
            "source type structure-member base is invalid"
        );
    }

    #[test]
    fn task_249pi_base_and_member_corruption_fail_atomically() {
        for profile in [Task249PiProfile::Means, Task249PiProfile::Equals] {
            let fixture = task_249pi_fixture(profile);

            let mut empty = fixture.extension.clone();
            empty.members.clear();
            assert_task_249pi_extension_error(
                &fixture,
                empty,
                &fixture.arena,
                SourceTypeError::EmptyStructureMembers,
            );
            for cardinality in [1, 3] {
                let mut invalid = fixture.extension.clone();
                if cardinality == 1 {
                    invalid.members.pop();
                } else {
                    invalid.members.push(invalid.members[0].clone());
                }
                assert_task_249pi_extension_error(
                    &fixture,
                    invalid,
                    &fixture.arena,
                    SourceTypeError::StructureMemberExtensionCardinalityMismatch,
                );
            }

            let mut wrong_environment = fixture.extension.clone();
            wrong_environment.source_id = other_source_id();
            assert_task_249pi_extension_error(
                &fixture,
                wrong_environment,
                &fixture.arena,
                SourceTypeError::EnvironmentMismatch,
            );

            let base_corruptions: [fn(&mut SourceTypeApplicationHandoff); 8] = [
                |base| base.applications.entries.clear(),
                |base| base.applications.entries[0].binding = BindingId::new(1),
                |base| base.applications.entries[0].root = SourceTypeExpressionId::new(1),
                |base| base.expressions.entries[0].source_range.start = 129,
                |base| base.expressions.entries[0].site = role(63, "parameter"),
                |base| base.expressions.entries[0].spelling = "Task264CarrierX".to_owned(),
                |base| base.expressions.entries[0].head = SourceTypeHead::BuiltinSet,
                |base| {
                    base.arguments.entries.push(SourceTypeArgumentRow {
                        id: SourceTypeArgumentId::new(0),
                        parent: SourceTypeExpressionId::new(0),
                        ordinal: 0,
                        argument: SourceTypeArgument::TypeSite {
                            expression: SourceTypeExpressionId::new(0),
                        },
                    });
                },
            ];
            for corrupt in base_corruptions {
                let mut base = fixture.base.clone();
                corrupt(&mut base);
                let baseline = base.clone();
                assert_eq!(
                    SourceTypeStructureMemberProducer::extend_property_implementation(
                        &base,
                        fixture.extension.clone(),
                        &fixture.arena,
                    ),
                    Err(SourceTypeError::InvalidStructureMemberBase)
                );
                assert_eq!(base, baseline);
            }

            let invalid_rows: [fn(&mut SourceTypeStructureMemberHandoffInput, usize); 8] = [
                |input, index| input.members[index].source_ordinal = 1 - index,
                |input, index| input.members[index].member_range.start -= 1,
                |input, index| input.members[index].member_range.end -= 1,
                |input, index| input.members[index].member_range.source_id = other_source_id(),
                |input, index| input.members[index].expression.source_id = other_source_id(),
                |input, index| {
                    input.members[index].expression.module_id = module("task249pi.other")
                },
                |input, index| input.members[index].expression.source_range.start -= 1,
                |input, index| input.members[index].expression.head_range.start -= 1,
            ];
            for index in 0..2 {
                for mutate in invalid_rows {
                    let mut invalid = fixture.extension.clone();
                    mutate(&mut invalid, index);
                    assert_task_249pi_extension_error(
                        &fixture,
                        invalid,
                        &fixture.arena,
                        SourceTypeError::InvalidStructureMember {
                            structure_member: SourceTypeStructureMemberId::new(index),
                        },
                    );
                }
            }

            let invalid_sites: [fn(&mut SourceTypeStructureMemberHandoffInput, usize); 4] = [
                |input, index| {
                    input.members[index].member_site =
                        role(input.members[index].member_site.node().index(), "member")
                },
                |input, index| {
                    input.members[index].expression.site = role(
                        input.members[index].expression.site.node().index(),
                        "expression",
                    )
                },
                |input, index| {
                    input.members[index].expression.head_site = role(
                        input.members[index].expression.head_site.node().index(),
                        "head",
                    )
                },
                |input, index| {
                    input.members[index].expression.site = input.members[index].member_site.clone()
                },
            ];
            for index in 0..2 {
                for mutate in invalid_sites {
                    let mut invalid = fixture.extension.clone();
                    mutate(&mut invalid, index);
                    assert_task_249pi_extension_error(
                        &fixture,
                        invalid,
                        &fixture.arena,
                        SourceTypeError::InvalidStructureMemberSite {
                            structure_member: SourceTypeStructureMemberId::new(index),
                        },
                    );
                }
            }
            let mut duplicate_site = fixture.extension.clone();
            duplicate_site.members[1].member_site = duplicate_site.members[0].member_site.clone();
            assert_task_249pi_extension_error(
                &fixture,
                duplicate_site,
                &fixture.arena,
                SourceTypeError::InvalidStructureMemberSite {
                    structure_member: SourceTypeStructureMemberId::new(1),
                },
            );

            let unsupported: [fn(&mut SourceTypeStructureMemberHandoffInput, usize); 5] = [
                |input, index| input.members[index].expression.form = SourceTypeApplicationForm::Of,
                |input, index| input.members[index].expression.head = SourceTypeHead::BuiltinObject,
                |input, index| input.members[index].expression.spelling = "Set".to_owned(),
                |input, index| input.members[index].expression.head_spelling = "Set".to_owned(),
                |input, index| {
                    input.members[index].expression.recovery = NodeRecoveryState::Recovered
                },
            ];
            for index in 0..2 {
                for mutate in unsupported {
                    let mut invalid = fixture.extension.clone();
                    mutate(&mut invalid, index);
                    assert_task_249pi_extension_error(
                        &fixture,
                        invalid,
                        &fixture.arena,
                        SourceTypeError::UnsupportedStructureMember {
                            structure_member: SourceTypeStructureMemberId::new(index),
                        },
                    );
                }
            }

            let mut cardinality_over_environment = fixture.extension.clone();
            cardinality_over_environment.members.pop();
            cardinality_over_environment.source_id = other_source_id();
            assert_task_249pi_extension_error(
                &fixture,
                cardinality_over_environment,
                &fixture.arena,
                SourceTypeError::StructureMemberExtensionCardinalityMismatch,
            );
            let mut invalid_base = fixture.base.clone();
            invalid_base.applications.entries[0].binding = BindingId::new(1);
            let invalid_base_baseline = invalid_base.clone();
            let mut environment_over_base = fixture.extension.clone();
            environment_over_base.source_id = other_source_id();
            assert_eq!(
                SourceTypeStructureMemberProducer::extend_property_implementation(
                    &invalid_base,
                    environment_over_base,
                    &fixture.arena,
                ),
                Err(SourceTypeError::EnvironmentMismatch)
            );
            assert_eq!(invalid_base, invalid_base_baseline);
            let mut base_over_row = fixture.extension.clone();
            base_over_row.members[0].source_ordinal = 1;
            assert_eq!(
                SourceTypeStructureMemberProducer::extend_property_implementation(
                    &invalid_base,
                    base_over_row,
                    &fixture.arena,
                ),
                Err(SourceTypeError::InvalidStructureMemberBase)
            );
            assert_eq!(invalid_base, invalid_base_baseline);
            let mut row_over_site = fixture.extension.clone();
            row_over_site.members[1].source_ordinal = 0;
            row_over_site.members[0].member_site = role(
                row_over_site.members[0].member_site.node().index(),
                "member",
            );
            assert_task_249pi_extension_error(
                &fixture,
                row_over_site,
                &fixture.arena,
                SourceTypeError::InvalidStructureMember {
                    structure_member: SourceTypeStructureMemberId::new(1),
                },
            );
            let mut site_over_shape = fixture.extension.clone();
            site_over_shape.members[0].member_site = role(
                site_over_shape.members[0].member_site.node().index(),
                "member",
            );
            site_over_shape.members[0].expression.form = SourceTypeApplicationForm::Of;
            assert_task_249pi_extension_error(
                &fixture,
                site_over_shape,
                &fixture.arena,
                SourceTypeError::InvalidStructureMemberSite {
                    structure_member: SourceTypeStructureMemberId::new(0),
                },
            );

            let handoff = SourceTypeStructureMemberProducer::extend_property_implementation(
                &fixture.base,
                fixture.extension.clone(),
                &fixture.arena,
            )
            .expect("Task 249PI one-shot base");
            let handoff_baseline = handoff.clone();
            let mut already_present_over_input = fixture.extension.clone();
            already_present_over_input.members.clear();
            already_present_over_input.source_id = other_source_id();
            assert_eq!(
                SourceTypeStructureMemberProducer::extend_property_implementation(
                    &handoff,
                    already_present_over_input,
                    &fixture.arena,
                ),
                Err(SourceTypeError::StructureMembersAlreadyPresent)
            );
            assert_eq!(handoff, handoff_baseline);
        }
    }

    #[test]
    fn task_249pi_arena_and_installation_drift_fail_closed() {
        for profile in [Task249PiProfile::Means, Task249PiProfile::Equals] {
            let fixture = task_249pi_fixture(profile);
            let (parameter_sites, members) = match profile {
                Task249PiProfile::Means => (
                    TASK_249PI_MEANS_PARAMETER_SITES,
                    TASK_249PI_MEANS_STRUCTURE_MEMBERS,
                ),
                Task249PiProfile::Equals => (
                    TASK_249PI_EQUALS_PARAMETER_SITES,
                    TASK_249PI_EQUALS_STRUCTURE_MEMBERS,
                ),
            };
            for node in [parameter_sites.0, parameter_sites.1] {
                let arena = task_249pi_arena(
                    fixture.source,
                    profile,
                    Some((node, range(fixture.source, 0, 1), NodeRecoveryState::Normal)),
                );
                assert_task_249pi_extension_error(
                    &fixture,
                    fixture.extension.clone(),
                    &arena,
                    SourceTypeError::InvalidStructureMemberBase,
                );
            }

            for (index, (member, member_start, member_end, expression, head, start, end)) in
                members.into_iter().enumerate()
            {
                for node in [member, expression, head] {
                    let exact = if node == member {
                        range(fixture.source, member_start, member_end)
                    } else {
                        range(fixture.source, start, end)
                    };
                    for (source_range, recovery) in [
                        (exact, NodeRecoveryState::Recovered),
                        (range(fixture.source, 0, 1), NodeRecoveryState::Normal),
                    ] {
                        let arena = task_249pi_arena(
                            fixture.source,
                            profile,
                            Some((node, source_range, recovery)),
                        );
                        assert_task_249pi_extension_error(
                            &fixture,
                            fixture.extension.clone(),
                            &arena,
                            SourceTypeError::InvalidStructureMemberSite {
                                structure_member: SourceTypeStructureMemberId::new(index),
                            },
                        );
                    }
                }
            }

            let handoff = SourceTypeStructureMemberProducer::extend_property_implementation(
                &fixture.base,
                fixture.extension.clone(),
                &fixture.arena,
            )
            .expect("Task 249PI installation base");
            let corruptions: [fn(&mut SourceTypeApplicationHandoff); 9] = [
                |value| value.applications.entries[0].root = SourceTypeExpressionId::new(1),
                |value| value.expressions.entries[0].id = SourceTypeExpressionId::new(1),
                |value| value.structure_members.entries.clear(),
                |value| value.structure_members.entries[0].id = SourceTypeStructureMemberId::new(1),
                |value| value.structure_members.entries[0].root = SourceTypeExpressionId::new(2),
                |value| value.expressions.entries[1].source_range.start = 61,
                |value| value.expressions.entries.pop().map(|_| ()).unwrap_or(()),
                |value| {
                    value
                        .definition_returns
                        .entries
                        .push(SourceTypeDefinitionReturn {
                            id: SourceTypeDefinitionReturnId::new(0),
                            definition_site: node_site(56),
                            definition_range: range(value.source_id, 45, 66),
                            source_ordinal: 0,
                            root: SourceTypeExpressionId::new(1),
                        });
                },
                |value| {
                    value.mode_rhs.entries.push(SourceTypeModeRhs {
                        id: SourceTypeModeRhsId::new(0),
                        definition_site: node_site(56),
                        definition_range: range(value.source_id, 45, 66),
                        source_ordinal: 0,
                        root: SourceTypeExpressionId::new(1),
                    });
                },
            ];
            for corrupt in corruptions {
                let mut invalid = handoff.clone();
                corrupt(&mut invalid);
                assert_eq!(
                    task_249r_typed_ast(
                        fixture.source,
                        fixture.module.clone(),
                        invalid,
                        fixture.arena.clone(),
                    ),
                    Err(TypedAstError::InvalidSourceType)
                );
            }
        }
    }

    #[test]
    fn task_249pi_typed_final_replay_and_sibling_isolation() {
        for profile in [Task249PiProfile::Means, Task249PiProfile::Equals] {
            let fixture = task_249pi_fixture(profile);
            let baseline = fixture.base.clone();
            let handoff = SourceTypeStructureMemberProducer::extend_property_implementation(
                &fixture.base,
                fixture.extension.clone(),
                &fixture.arena,
            )
            .expect("Task 249PI handoff");
            let fingerprint = handoff.debug_text();
            let replay = SourceTypeStructureMemberProducer::extend_property_implementation(
                &fixture.base,
                fixture.extension.clone(),
                &fixture.arena,
            )
            .expect("Task 249PI replay");
            assert_eq!(fixture.base, baseline);
            assert_eq!(replay, handoff);
            assert_eq!(replay.debug_text(), fingerprint);

            let typed = task_249r_typed_ast(
                fixture.source,
                fixture.module.clone(),
                handoff.clone(),
                fixture.arena.clone(),
            )
            .expect("typed Task 249PI installation");
            assert_eq!(typed.source_type(), Some(&handoff));
            assert!(typed.source_predicate_definition().is_none());
            assert!(typed.source_functor_definition().is_none());
            assert!(typed.types().is_empty());
            assert!(typed.facts().is_empty());
            assert!(typed.coercions().is_empty());
            assert!(typed.initial_obligations().is_empty());
            assert!(typed.diagnostics().is_empty());
            assert_eq!(typed.debug_text().matches(fingerprint.as_str()).count(), 1);

            let resolved = assemble_empty_resolved(&typed);
            let resolved_replay = assemble_empty_resolved(&typed);
            assert_eq!(resolved_replay, resolved);
            assert_eq!(resolved.source_type(), Some(&handoff));
            assert!(resolved.source_predicate_definition().is_none());
            assert!(resolved.source_functor_definition().is_none());
            assert!(resolved.expr_metadata().is_empty());
            assert!(resolved.inserted_coercions().is_empty());
            assert!(resolved.cluster_facts().is_empty());
            assert!(resolved.diagnostics().is_empty());
            assert!(resolved.checked_formulas().is_empty());
            assert!(resolved.statement_semantics().is_empty());
            assert!(resolved.checked_proofs().is_empty());
            assert_eq!(
                resolved.debug_text().matches(fingerprint.as_str()).count(),
                1
            );

            let task_249r = task_249r_fixture();
            let task_249r_handoff = SourceTypeDefinitionReturnProducer::extend(
                &task_249r.base,
                task_249r.extension.clone(),
                &task_249r.arena,
            )
            .expect("Task 249R sibling");
            let mut pi_after_returns = fixture.extension.clone();
            pi_after_returns.source_id = task_249r.source;
            pi_after_returns.module_id = task_249r.module.clone();
            assert_eq!(
                SourceTypeStructureMemberProducer::extend_property_implementation(
                    &task_249r_handoff,
                    pi_after_returns,
                    &task_249r.arena,
                ),
                Err(SourceTypeError::InvalidStructureMemberBase)
            );

            let task_249m = task_249m_fixture();
            let task_249m_handoff = SourceTypeModeRhsProducer::extend(
                &task_249m.base,
                task_249m.extension.clone(),
                &task_249m.arena,
            )
            .expect("Task 249M sibling");
            let mut pi_after_rhs = fixture.extension.clone();
            pi_after_rhs.source_id = task_249m.source;
            pi_after_rhs.module_id = task_249m.module.clone();
            assert_eq!(
                SourceTypeStructureMemberProducer::extend_property_implementation(
                    &task_249m_handoff,
                    pi_after_rhs,
                    &task_249m.arena,
                ),
                Err(SourceTypeError::InvalidStructureMemberBase)
            );

            let task_249s = task_249s_fixture();
            let task_249s_handoff =
                SourceTypeStructureMemberProducer::build(task_249s.input, &task_249s.arena)
                    .expect("Task 249S sibling");
            assert_eq!(
                SourceTypeStructureMemberProducer::extend_property_implementation(
                    &task_249s_handoff,
                    fixture.extension.clone(),
                    &task_249s.arena,
                ),
                Err(SourceTypeError::StructureMembersAlreadyPresent)
            );

            let mut returns_after_pi = task_249r.extension;
            returns_after_pi.source_id = fixture.source;
            returns_after_pi.module_id = fixture.module.clone();
            assert_eq!(
                SourceTypeDefinitionReturnProducer::extend(
                    &handoff,
                    returns_after_pi,
                    &fixture.arena,
                ),
                Err(SourceTypeError::InvalidDefinitionReturnBase)
            );
            let mut rhs_after_pi = task_249m.extension;
            rhs_after_pi.source_id = fixture.source;
            rhs_after_pi.module_id = fixture.module;
            assert_eq!(
                SourceTypeModeRhsProducer::extend(&handoff, rhs_after_pi, &fixture.arena),
                Err(SourceTypeError::InvalidModeRhsBase)
            );
        }
    }

    fn task269ct_fixture() -> Task269ctFixture {
        let source = source_id_for("d7");
        let module = module("task269c");
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
        let lower_fingerprint = format!(
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
            module.package().as_str(),
            module.path().as_str(),
            theorem_symbol.fqn().as_str(),
        );
        let dependency = SourceProofLocalLetBindingProducer::build(
            SourceProofLocalLetBindingHandoffInput {
                source_id: source,
                module_id: module.clone(),
                lower_fingerprint,
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
            },
            &task269ct_base_binding_env(source, module.clone()),
        )
        .expect("Task269CT exact Task269C dependency");
        let input = task269ct_input(source, module.clone());
        let symbols = SymbolEnv::new(module.clone(), SymbolEnvIndexes::default());
        let arena = task269ct_test_arena(source, 2, false);
        Task269ctFixture {
            source,
            module,
            dependency,
            input,
            symbols,
            arena,
        }
    }

    fn task269ct_base_binding_env(source: SourceId, module: ModuleId) -> BindingEnv {
        let mut bindings = BindingTable::new();
        let binding = bindings.insert(BindingDraft {
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
            bindings: vec![binding],
            visible_bindings: vec![binding],
            recovery: BindingContextRecovery::Normal,
        });
        BindingEnv::try_new(BindingEnvParts {
            source_id: source,
            module_id: module,
            contexts,
            bindings,
            diagnostics: BindingDiagnosticTable::new(),
        })
        .expect("Task269CT exact base binding environment")
    }

    fn task269ct_input(source: SourceId, module: ModuleId) -> SourceTypeHandoffInput {
        SourceTypeHandoffInput {
            source_id: source,
            module_id: module.clone(),
            applications: (0..2)
                .map(|index| SourceTypeApplicationInput {
                    binding: BindingId::new(index),
                    source_ordinal: index,
                    root: SourceTypeExpressionId::new(index),
                })
                .collect(),
            expressions: [(14, 17), (76, 79)]
                .into_iter()
                .enumerate()
                .map(|(index, (start, end))| SourceTypeExpressionInput {
                    source_id: source,
                    module_id: module.clone(),
                    site: role(index, "source.type.expression"),
                    source_range: range(source, start, end),
                    spelling: "set".to_owned(),
                    head_site: role(index, "source.type.head"),
                    head_range: range(source, start, end),
                    head_spelling: "set".to_owned(),
                    form: SourceTypeApplicationForm::Bare,
                    head: SourceTypeHead::BuiltinSet,
                    recovery: NodeRecoveryState::Normal,
                })
                .collect(),
            arguments: Vec::new(),
        }
    }

    fn task269ct_test_arena(source: SourceId, root: usize, wrong_kind: bool) -> TypedArena {
        let mut builder = TypedArenaBuilder::new();
        let reserve = builder
            .push(TypedNode::new(
                "source.proof-local.let.reserve-type",
                SourceAnchor::Range(range(source, 14, 17)),
            ))
            .expect("Task269CT reserve type node");
        let local = builder
            .push(TypedNode::new(
                if wrong_kind {
                    "source.proof-local.let.type.wrong"
                } else {
                    "source.proof-local.let.type"
                },
                SourceAnchor::Range(range(source, 76, 79)),
            ))
            .expect("Task269CT local type node");
        let type_root = builder
            .push(
                TypedNode::new(
                    "source.proof-local.let.type-root",
                    SourceAnchor::Range(range(source, 0, 99)),
                )
                .with_children(vec![reserve, local]),
            )
            .expect("Task269CT type root");
        assert_eq!(type_root, TypedNodeId::new(2));
        builder
            .finish(Some(TypedNodeId::new(root)))
            .expect("Task269CT typed arena")
    }

    fn task269ct_empty_typed(source: SourceId, module: ModuleId, arena: TypedArena) -> TypedAst {
        TypedAst::try_new(TypedAstParts {
            source_id: source,
            module_id: module,
            resolved_root: None,
            source_context: None,
            source_type: None,
            source_attribute: None,
            nodes: arena,
            contexts: LocalTypeContextTable::new(),
            types: TypeTable::new(),
            facts: TypeFactTable::new(),
            coercions: CoercionTable::new(),
            initial_obligations: InitialObligationTable::new(),
            diagnostics: TypeDiagnosticTable::new(),
        })
        .expect("Task269CT empty typed AST")
    }

    fn task269gt_fixture() -> Task269gtFixture {
        let source = source_id_for("e7");
        let module = module("task269g");
        let local = concat!(
            "contribution=0:namespace=task269g:owner=theorem#1:shell=theorem:",
            "kind=theorem:name=FormulaStatementGivenSmoke:notation=_:arity=_:",
            "definition=theorem:registration=_:policy=non-overloadable:",
            "slot=non-overloadable:_:theorem:_"
        );
        let theorem_symbol = SymbolId::new(
            module.clone(),
            LocalSymbolId::new(local),
            FullyQualifiedName::new(format!("pkg::task269g::{local}")),
        );
        let mut contributions = SourceContributionIndex::new();
        let contribution = contributions.insert(
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
        let lower_fingerprint = format!(
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
            module.package().as_str(),
            module.path().as_str(),
            theorem_symbol.fqn().as_str(),
        );
        let dependency = SourceProofLocalGivenBindingProducer::build(
            SourceProofLocalGivenBindingHandoffInput {
                source_id: source,
                module_id: module.clone(),
                lower_fingerprint,
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
            },
            &task269ct_base_binding_env(source, module.clone()),
        )
        .expect("Task269GT exact Task269G dependency");
        let input = task269gt_input(source, module.clone());
        let symbols = SymbolEnv::new(module.clone(), SymbolEnvIndexes::default());
        let arena = task269gt_test_arena(source, 2, false);
        Task269gtFixture {
            source,
            module,
            dependency,
            input,
            symbols,
            arena,
        }
    }

    fn task269gt_input(source: SourceId, module: ModuleId) -> SourceTypeHandoffInput {
        SourceTypeHandoffInput {
            source_id: source,
            module_id: module.clone(),
            applications: (0..2)
                .map(|index| SourceTypeApplicationInput {
                    binding: BindingId::new(index),
                    source_ordinal: index,
                    root: SourceTypeExpressionId::new(index),
                })
                .collect(),
            expressions: [(14, 17), (84, 87)]
                .into_iter()
                .enumerate()
                .map(|(index, (start, end))| SourceTypeExpressionInput {
                    source_id: source,
                    module_id: module.clone(),
                    site: role(index, "source.type.expression"),
                    source_range: range(source, start, end),
                    spelling: "set".to_owned(),
                    head_site: role(index, "source.type.head"),
                    head_range: range(source, start, end),
                    head_spelling: "set".to_owned(),
                    form: SourceTypeApplicationForm::Bare,
                    head: SourceTypeHead::BuiltinSet,
                    recovery: NodeRecoveryState::Normal,
                })
                .collect(),
            arguments: Vec::new(),
        }
    }

    fn task269gt_test_arena(source: SourceId, root: usize, wrong_kind: bool) -> TypedArena {
        let mut builder = TypedArenaBuilder::new();
        let reserve = builder
            .push(TypedNode::new(
                "source.proof-local.given.reserve-type",
                SourceAnchor::Range(range(source, 14, 17)),
            ))
            .expect("Task269GT reserve type node");
        let local = builder
            .push(TypedNode::new(
                if wrong_kind {
                    "source.proof-local.given.type.wrong"
                } else {
                    "source.proof-local.given.type"
                },
                SourceAnchor::Range(range(source, 84, 87)),
            ))
            .expect("Task269GT local type node");
        let type_root = builder
            .push(
                TypedNode::new(
                    "source.proof-local.given.type-root",
                    SourceAnchor::Range(range(source, 0, 128)),
                )
                .with_children(vec![reserve, local]),
            )
            .expect("Task269GT type root");
        assert_eq!(type_root, TypedNodeId::new(2));
        builder
            .finish(Some(TypedNodeId::new(root)))
            .expect("Task269GT typed arena")
    }

    fn task269gct_fixture() -> Task269gctFixture {
        let source = source_id_for("f7");
        let module = module("task269gc");
        let local = concat!(
            "contribution=0:namespace=task269gc:owner=theorem#1:shell=theorem:",
            "kind=theorem:name=ProofLocalGivenConditionUseSmoke:notation=_:arity=_:",
            "definition=theorem:registration=_:policy=non-overloadable:",
            "slot=non-overloadable:_:theorem:_"
        );
        let theorem_symbol = SymbolId::new(
            module.clone(),
            LocalSymbolId::new(local),
            FullyQualifiedName::new(format!("pkg::task269gc::{local}")),
        );
        let mut contributions = SourceContributionIndex::new();
        let contribution = contributions.insert(
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
        let lower_fingerprint = format!(
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
            module.package().as_str(),
            module.path().as_str(),
            theorem_symbol.fqn().as_str(),
        );
        let dependency = SourceProofLocalGivenConditionBindingProducer::build(
            SourceProofLocalGivenConditionBindingHandoffInput {
                source_id: source,
                module_id: module.clone(),
                lower_fingerprint,
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
            },
            &task269ct_base_binding_env(source, module.clone()),
        )
        .expect("Task269GCT exact Task269GC dependency");
        let input = task269gct_input(source, module.clone());
        let symbols = SymbolEnv::new(module.clone(), SymbolEnvIndexes::default());
        let arena = task269gct_test_arena(source, 2, false);
        Task269gctFixture {
            source,
            module,
            dependency,
            input,
            symbols,
            arena,
        }
    }

    fn task269gct_input(source: SourceId, module: ModuleId) -> SourceTypeHandoffInput {
        SourceTypeHandoffInput {
            source_id: source,
            module_id: module.clone(),
            applications: (0..2)
                .map(|index| SourceTypeApplicationInput {
                    binding: BindingId::new(index),
                    source_ordinal: index,
                    root: SourceTypeExpressionId::new(index),
                })
                .collect(),
            expressions: [(14, 17), (90, 93)]
                .into_iter()
                .enumerate()
                .map(|(index, (start, end))| SourceTypeExpressionInput {
                    source_id: source,
                    module_id: module.clone(),
                    site: role(index, "source.type.expression"),
                    source_range: range(source, start, end),
                    spelling: "set".to_owned(),
                    head_site: role(index, "source.type.head"),
                    head_range: range(source, start, end),
                    head_spelling: "set".to_owned(),
                    form: SourceTypeApplicationForm::Bare,
                    head: SourceTypeHead::BuiltinSet,
                    recovery: NodeRecoveryState::Normal,
                })
                .collect(),
            arguments: Vec::new(),
        }
    }

    fn task269gct_test_arena(source: SourceId, root: usize, wrong_kind: bool) -> TypedArena {
        let mut builder = TypedArenaBuilder::new();
        let reserve = builder
            .push(TypedNode::new(
                "source.proof-local.given-condition.reserve-type",
                SourceAnchor::Range(range(source, 14, 17)),
            ))
            .expect("Task269GCT reserve type node");
        let local = builder
            .push(TypedNode::new(
                if wrong_kind {
                    "source.proof-local.given-condition.type.wrong"
                } else {
                    "source.proof-local.given-condition.type"
                },
                SourceAnchor::Range(range(source, 90, 93)),
            ))
            .expect("Task269GCT local type node");
        let type_root = builder
            .push(
                TypedNode::new(
                    "source.proof-local.given-condition.type-root",
                    SourceAnchor::Range(range(source, 0, 133)),
                )
                .with_children(vec![reserve, local]),
            )
            .expect("Task269GCT type root");
        assert_eq!(type_root, TypedNodeId::new(2));
        builder
            .finish(Some(TypedNodeId::new(root)))
            .expect("Task269GCT typed arena")
    }

    fn task269gct_given_use_term_input(
        source: SourceId,
        module_id: ModuleId,
    ) -> SourcePrimaryTermHandoffInput {
        SourcePrimaryTermHandoffInput {
            source_id: source,
            module_id,
            terms: [(3, 116, 117), (4, 120, 121)]
                .into_iter()
                .enumerate()
                .map(
                    |(source_ordinal, (node, start, end))| SourcePrimaryTermInput {
                        site: TypedSiteRef::Node(TypedNodeId::new(node)),
                        source_range: range(source, start, end),
                        source_ordinal,
                        context: BindingContextId::new(1),
                        recovery: SourcePrimaryTermRecovery::Normal,
                        spelling: "y".to_owned(),
                        kind: SourcePrimaryTermKind::VariableReference,
                        role: SourcePrimaryTermRole::Value,
                        parent: None,
                    },
                )
                .collect(),
            references: (0..2)
                .map(|index| SourcePrimaryTermReferenceInput {
                    term: SourcePrimaryTermId::new(index),
                    binding: BindingId::new(1),
                    role: SourcePrimaryTermReferenceRole::Variable,
                })
                .collect(),
            numeric_type_requests: Vec::new(),
        }
    }

    fn task269gct_given_use_term_arena(source: SourceId) -> TypedArena {
        let mut builder = TypedArenaBuilder::new();
        let reserve = builder
            .push(TypedNode::new(
                "source.proof-local.given-use.reserve-type",
                SourceAnchor::Range(range(source, 14, 17)),
            ))
            .expect("Task269GCT GU reserve node");
        let local = builder
            .push(TypedNode::new(
                "source.proof-local.given-use.type",
                SourceAnchor::Range(range(source, 84, 87)),
            ))
            .expect("Task269GCT GU type node");
        let type_root = builder
            .push(
                TypedNode::new(
                    "source.proof-local.given-use.type-root",
                    SourceAnchor::Range(range(source, 0, 127)),
                )
                .with_children(vec![reserve, local]),
            )
            .expect("Task269GCT GU type root");
        let first = builder
            .push(TypedNode::new(
                "source.term.variable-reference",
                SourceAnchor::Range(range(source, 116, 117)),
            ))
            .expect("Task269GCT GU first term");
        let second = builder
            .push(TypedNode::new(
                "source.term.variable-reference",
                SourceAnchor::Range(range(source, 120, 121)),
            ))
            .expect("Task269GCT GU second term");
        let root = builder
            .push(
                TypedNode::new(
                    "source.proof-local.given-use.term-root",
                    SourceAnchor::Range(range(source, 0, 127)),
                )
                .with_children(vec![type_root, first, second]),
            )
            .expect("Task269GCT GU term root");
        builder.finish(Some(root)).expect("Task269GCT GU arena")
    }

    #[derive(Debug, Clone, Copy)]
    enum Task269gctInputMutation {
        ApplicationCount,
        ApplicationBinding,
        ApplicationOrdinal,
        ApplicationRoot,
        ExpressionCount,
        ExpressionSource,
        ExpressionModule,
        ExpressionSite,
        ExpressionRange,
        ExpressionSpelling,
        HeadSite,
        HeadRange,
        HeadSpelling,
        Form,
        Head,
        Recovery,
        Argument,
    }

    fn mutated_task269gct_input(
        source: SourceId,
        module_id: ModuleId,
        mutation: Task269gctInputMutation,
    ) -> SourceTypeHandoffInput {
        let mut input = task269gct_input(source, module_id);
        match mutation {
            Task269gctInputMutation::ApplicationCount => {
                input.applications.pop();
            }
            Task269gctInputMutation::ApplicationBinding => {
                input.applications[1].binding = BindingId::new(0);
            }
            Task269gctInputMutation::ApplicationOrdinal => {
                input.applications[1].source_ordinal = 2;
            }
            Task269gctInputMutation::ApplicationRoot => {
                input.applications[1].root = SourceTypeExpressionId::new(0);
            }
            Task269gctInputMutation::ExpressionCount => {
                input.expressions.pop();
            }
            Task269gctInputMutation::ExpressionSource => {
                input.expressions[1].source_id = other_source_id();
            }
            Task269gctInputMutation::ExpressionModule => {
                input.expressions[1].module_id = module("task269gct.expression.wrong");
            }
            Task269gctInputMutation::ExpressionSite => {
                input.expressions[1].site = role(1, "source.type.expression.wrong");
            }
            Task269gctInputMutation::ExpressionRange => {
                input.expressions[1].source_range.end += 1;
            }
            Task269gctInputMutation::ExpressionSpelling => {
                input.expressions[1].spelling = "object".to_owned();
            }
            Task269gctInputMutation::HeadSite => {
                input.expressions[1].head_site = role(1, "source.type.head.wrong");
            }
            Task269gctInputMutation::HeadRange => {
                input.expressions[1].head_range.end += 1;
            }
            Task269gctInputMutation::HeadSpelling => {
                input.expressions[1].head_spelling = "object".to_owned();
            }
            Task269gctInputMutation::Form => {
                input.expressions[1].form = SourceTypeApplicationForm::Of;
            }
            Task269gctInputMutation::Head => {
                input.expressions[1].head = SourceTypeHead::BuiltinObject;
            }
            Task269gctInputMutation::Recovery => {
                input.expressions[1].recovery = NodeRecoveryState::Recovered;
            }
            Task269gctInputMutation::Argument => {
                input.arguments.push(SourceTypeArgumentInput {
                    parent: SourceTypeExpressionId::new(1),
                    ordinal: 0,
                    argument: SourceTypeArgument::TypeSite {
                        expression: SourceTypeExpressionId::new(0),
                    },
                });
            }
        }
        input
    }

    #[derive(Debug, Clone, Copy)]
    enum Task269gctArenaMutation {
        Kind,
        ResolvedNode,
        Anchor,
        Children,
        Typing,
        Recovery,
        Links,
    }

    fn mutated_task269gct_arena(
        source: SourceId,
        node: usize,
        mutation: Task269gctArenaMutation,
    ) -> TypedArena {
        let mut nodes = vec![
            TypedNode::new(
                "source.proof-local.given-condition.reserve-type",
                SourceAnchor::Range(range(source, 14, 17)),
            ),
            TypedNode::new(
                "source.proof-local.given-condition.type",
                SourceAnchor::Range(range(source, 90, 93)),
            ),
            TypedNode::new(
                "source.proof-local.given-condition.type-root",
                SourceAnchor::Range(range(source, 0, 133)),
            )
            .with_children(vec![TypedNodeId::new(0), TypedNodeId::new(1)]),
        ];
        match mutation {
            Task269gctArenaMutation::Kind => {
                nodes[node].kind = "source.proof-local.given-condition.wrong".into();
            }
            Task269gctArenaMutation::ResolvedNode => {
                use mizar_syntax as syntax;

                let mut builder = mizar_resolve::resolved_ast::ResolvedArenaBuilder::new();
                let resolved = builder
                    .push(mizar_resolve::resolved_ast::ResolvedNode::new(
                        syntax::SurfaceNodeKind::CompilationUnit,
                        Vec::new(),
                        SemanticOrigin::new(
                            source,
                            module("task269gct.resolved"),
                            SourceAnchor::Range(range(source, 0, 133)),
                            vec![node as u32],
                        ),
                    ))
                    .expect("Task269GCT resolved-node mutation id");
                nodes[node].resolved_node = Some(resolved);
            }
            Task269gctArenaMutation::Anchor => {
                nodes[node].anchor = SourceAnchor::Range(range(source, 1, 2));
            }
            Task269gctArenaMutation::Children => {
                nodes[node].children = if node == 2 {
                    vec![TypedNodeId::new(1), TypedNodeId::new(0)]
                } else if node == 0 {
                    vec![TypedNodeId::new(1)]
                } else {
                    vec![TypedNodeId::new(0)]
                };
            }
            Task269gctArenaMutation::Typing => {
                nodes[node].typing = TypingState::Successful;
            }
            Task269gctArenaMutation::Recovery => {
                nodes[node].recovery = NodeRecoveryState::Recovered;
            }
            Task269gctArenaMutation::Links => {
                nodes[node]
                    .links
                    .facts
                    .push(crate::typed_ast::TypeFactId::new(0));
            }
        }
        TypedArena::try_new(Some(TypedNodeId::new(2)), nodes)
            .expect("Task269GCT mutated arena remains structurally constructible")
    }

    #[derive(Debug, Clone, Copy)]
    enum Task269gtInputMutation {
        ApplicationCount,
        ApplicationBinding,
        ApplicationOrdinal,
        ApplicationRoot,
        ExpressionCount,
        ExpressionSource,
        ExpressionModule,
        ExpressionSite,
        ExpressionRange,
        ExpressionSpelling,
        HeadSite,
        HeadRange,
        HeadSpelling,
        Form,
        Head,
        Recovery,
        Argument,
    }

    fn mutated_task269gt_input(
        source: SourceId,
        module_id: ModuleId,
        mutation: Task269gtInputMutation,
    ) -> SourceTypeHandoffInput {
        let mut input = task269gt_input(source, module_id);
        match mutation {
            Task269gtInputMutation::ApplicationCount => {
                input.applications.pop();
            }
            Task269gtInputMutation::ApplicationBinding => {
                input.applications[1].binding = BindingId::new(0);
            }
            Task269gtInputMutation::ApplicationOrdinal => {
                input.applications[1].source_ordinal = 2;
            }
            Task269gtInputMutation::ApplicationRoot => {
                input.applications[1].root = SourceTypeExpressionId::new(0);
            }
            Task269gtInputMutation::ExpressionCount => {
                input.expressions.pop();
            }
            Task269gtInputMutation::ExpressionSource => {
                input.expressions[1].source_id = other_source_id();
            }
            Task269gtInputMutation::ExpressionModule => {
                input.expressions[1].module_id = module("task269gt.expression.wrong");
            }
            Task269gtInputMutation::ExpressionSite => {
                input.expressions[1].site = role(1, "source.type.expression.wrong");
            }
            Task269gtInputMutation::ExpressionRange => {
                input.expressions[1].source_range.end += 1;
            }
            Task269gtInputMutation::ExpressionSpelling => {
                input.expressions[1].spelling = "object".to_owned();
            }
            Task269gtInputMutation::HeadSite => {
                input.expressions[1].head_site = role(1, "source.type.head.wrong");
            }
            Task269gtInputMutation::HeadRange => {
                input.expressions[1].head_range.end += 1;
            }
            Task269gtInputMutation::HeadSpelling => {
                input.expressions[1].head_spelling = "object".to_owned();
            }
            Task269gtInputMutation::Form => {
                input.expressions[1].form = SourceTypeApplicationForm::Of;
            }
            Task269gtInputMutation::Head => {
                input.expressions[1].head = SourceTypeHead::BuiltinObject;
            }
            Task269gtInputMutation::Recovery => {
                input.expressions[1].recovery = NodeRecoveryState::Recovered;
            }
            Task269gtInputMutation::Argument => {
                input.arguments.push(SourceTypeArgumentInput {
                    parent: SourceTypeExpressionId::new(1),
                    ordinal: 0,
                    argument: SourceTypeArgument::TypeSite {
                        expression: SourceTypeExpressionId::new(0),
                    },
                });
            }
        }
        input
    }

    #[derive(Debug, Clone, Copy)]
    enum Task269gtArenaMutation {
        Kind,
        ResolvedNode,
        Anchor,
        Children,
        Typing,
        Recovery,
        Links,
    }

    fn mutated_task269gt_arena(
        source: SourceId,
        node: usize,
        mutation: Task269gtArenaMutation,
    ) -> TypedArena {
        let mut nodes = vec![
            TypedNode::new(
                "source.proof-local.given.reserve-type",
                SourceAnchor::Range(range(source, 14, 17)),
            ),
            TypedNode::new(
                "source.proof-local.given.type",
                SourceAnchor::Range(range(source, 84, 87)),
            ),
            TypedNode::new(
                "source.proof-local.given.type-root",
                SourceAnchor::Range(range(source, 0, 128)),
            )
            .with_children(vec![TypedNodeId::new(0), TypedNodeId::new(1)]),
        ];
        match mutation {
            Task269gtArenaMutation::Kind => {
                nodes[node].kind = "source.proof-local.given.wrong".into();
            }
            Task269gtArenaMutation::ResolvedNode => {
                use mizar_syntax as syntax;

                let mut builder = mizar_resolve::resolved_ast::ResolvedArenaBuilder::new();
                let resolved = builder
                    .push(mizar_resolve::resolved_ast::ResolvedNode::new(
                        syntax::SurfaceNodeKind::CompilationUnit,
                        Vec::new(),
                        SemanticOrigin::new(
                            source,
                            module("task269gt.resolved"),
                            SourceAnchor::Range(range(source, 0, 128)),
                            vec![node as u32],
                        ),
                    ))
                    .expect("Task269GT resolved-node mutation id");
                nodes[node].resolved_node = Some(resolved);
            }
            Task269gtArenaMutation::Anchor => {
                nodes[node].anchor = SourceAnchor::Range(range(source, 1, 2));
            }
            Task269gtArenaMutation::Children => {
                nodes[node].children = if node == 2 {
                    vec![TypedNodeId::new(1), TypedNodeId::new(0)]
                } else if node == 0 {
                    vec![TypedNodeId::new(1)]
                } else {
                    vec![TypedNodeId::new(0)]
                };
            }
            Task269gtArenaMutation::Typing => {
                nodes[node].typing = TypingState::Successful;
            }
            Task269gtArenaMutation::Recovery => {
                nodes[node].recovery = NodeRecoveryState::Recovered;
            }
            Task269gtArenaMutation::Links => {
                nodes[node]
                    .links
                    .facts
                    .push(crate::typed_ast::TypeFactId::new(0));
            }
        }
        TypedArena::try_new(Some(TypedNodeId::new(2)), nodes)
            .expect("Task269GT mutated arena remains structurally constructible")
    }

    fn task269gupt_fixture() -> Task269guptFixture {
        let source = source_id_for("f7");
        let module = module("task269gup");
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
        let lower_fingerprint = format!(
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
            module.package().as_str(),
            module.path().as_str(),
            theorem_symbol.fqn().as_str(),
        );
        let dependency = SourceProofLocalGivenUseBindingProducer::build(
            SourceProofLocalGivenUseBindingHandoffInput {
                source_id: source,
                module_id: module.clone(),
                lower_fingerprint,
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
            },
            &task269ct_base_binding_env(source, module.clone()),
        )
        .expect("Task269GUPT exact Task269GUP dependency");
        let input = task269gupt_input(source, module.clone());
        let symbols = SymbolEnv::new(module.clone(), SymbolEnvIndexes::default());
        let arena = task269gupt_test_arena(source, 2, false);
        Task269guptFixture {
            source,
            module,
            dependency,
            input,
            symbols,
            arena,
        }
    }

    fn task269gupt_let_neighbor_fixture() -> Task269ctFixture {
        let source = source_id_for("f7");
        let module = module("task269gup");
        let local = concat!(
            "contribution=0:namespace=task269gup:owner=theorem#1:shell=theorem:",
            "kind=theorem:name=FormulaStatementLetSmoke:notation=_:arity=_:",
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
        let lower_fingerprint = format!(
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
            module.package().as_str(),
            module.path().as_str(),
            theorem_symbol.fqn().as_str(),
        );
        let dependency = SourceProofLocalLetBindingProducer::build(
            SourceProofLocalLetBindingHandoffInput {
                source_id: source,
                module_id: module.clone(),
                lower_fingerprint,
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
            },
            &task269ct_base_binding_env(source, module.clone()),
        )
        .expect("Task269GUPT same-identity let neighbor");
        Task269ctFixture {
            source,
            module: module.clone(),
            dependency,
            input: task269ct_input(source, module.clone()),
            symbols: SymbolEnv::new(module.clone(), SymbolEnvIndexes::default()),
            arena: task269ct_test_arena(source, 2, false),
        }
    }

    fn task269gupt_given_neighbor_fixture() -> Task269gtFixture {
        let source = source_id_for("f7");
        let module = module("task269gup");
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
        let lower_fingerprint = format!(
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
            module.package().as_str(),
            module.path().as_str(),
            theorem_symbol.fqn().as_str(),
        );
        let dependency = SourceProofLocalGivenBindingProducer::build(
            SourceProofLocalGivenBindingHandoffInput {
                source_id: source,
                module_id: module.clone(),
                lower_fingerprint,
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
            },
            &task269ct_base_binding_env(source, module.clone()),
        )
        .expect("Task269GUPT same-identity given neighbor");
        Task269gtFixture {
            source,
            module: module.clone(),
            dependency,
            input: task269gt_input(source, module.clone()),
            symbols: SymbolEnv::new(module.clone(), SymbolEnvIndexes::default()),
            arena: task269gt_test_arena(source, 2, false),
        }
    }

    fn task269gupt_input(source: SourceId, module: ModuleId) -> SourceTypeHandoffInput {
        SourceTypeHandoffInput {
            source_id: source,
            module_id: module.clone(),
            applications: (0..2)
                .map(|index| SourceTypeApplicationInput {
                    binding: BindingId::new(index),
                    source_ordinal: index,
                    root: SourceTypeExpressionId::new(index),
                })
                .collect(),
            expressions: [(14, 17), (84, 87)]
                .into_iter()
                .enumerate()
                .map(|(index, (start, end))| SourceTypeExpressionInput {
                    source_id: source,
                    module_id: module.clone(),
                    site: role(index, "source.type.expression"),
                    source_range: range(source, start, end),
                    spelling: "set".to_owned(),
                    head_site: role(index, "source.type.head"),
                    head_range: range(source, start, end),
                    head_spelling: "set".to_owned(),
                    form: SourceTypeApplicationForm::Bare,
                    head: SourceTypeHead::BuiltinSet,
                    recovery: NodeRecoveryState::Normal,
                })
                .collect(),
            arguments: Vec::new(),
        }
    }

    fn task269gupt_test_arena(source: SourceId, root: usize, wrong_kind: bool) -> TypedArena {
        let mut builder = TypedArenaBuilder::new();
        let reserve = builder
            .push(TypedNode::new(
                "source.proof-local.given-use.reserve-type",
                SourceAnchor::Range(range(source, 14, 17)),
            ))
            .expect("Task269GUPT reserve type node");
        let local = builder
            .push(TypedNode::new(
                if wrong_kind {
                    "source.proof-local.given-use.type.wrong"
                } else {
                    "source.proof-local.given-use.type"
                },
                SourceAnchor::Range(range(source, 84, 87)),
            ))
            .expect("Task269GUPT local type node");
        let type_root = builder
            .push(
                TypedNode::new(
                    "source.proof-local.given-use.type-root",
                    SourceAnchor::Range(range(source, 0, 127)),
                )
                .with_children(vec![reserve, local]),
            )
            .expect("Task269GUPT type root");
        assert_eq!(type_root, TypedNodeId::new(2));
        builder
            .finish(Some(TypedNodeId::new(root)))
            .expect("Task269GUPT typed arena")
    }

    fn mutated_task269gupt_input(
        source: SourceId,
        module_id: ModuleId,
        mutation: Task269gtInputMutation,
    ) -> SourceTypeHandoffInput {
        let mut input = task269gupt_input(source, module_id);
        match mutation {
            Task269gtInputMutation::ApplicationCount => {
                input.applications.pop();
            }
            Task269gtInputMutation::ApplicationBinding => {
                input.applications[1].binding = BindingId::new(0);
            }
            Task269gtInputMutation::ApplicationOrdinal => {
                input.applications[1].source_ordinal = 2;
            }
            Task269gtInputMutation::ApplicationRoot => {
                input.applications[1].root = SourceTypeExpressionId::new(0);
            }
            Task269gtInputMutation::ExpressionCount => {
                input.expressions.pop();
            }
            Task269gtInputMutation::ExpressionSource => {
                input.expressions[1].source_id = other_source_id();
            }
            Task269gtInputMutation::ExpressionModule => {
                input.expressions[1].module_id = module("task269gupt.expression.wrong");
            }
            Task269gtInputMutation::ExpressionSite => {
                input.expressions[1].site = role(1, "source.type.expression.wrong");
            }
            Task269gtInputMutation::ExpressionRange => {
                input.expressions[1].source_range.end += 1;
            }
            Task269gtInputMutation::ExpressionSpelling => {
                input.expressions[1].spelling = "object".to_owned();
            }
            Task269gtInputMutation::HeadSite => {
                input.expressions[1].head_site = role(1, "source.type.head.wrong");
            }
            Task269gtInputMutation::HeadRange => {
                input.expressions[1].head_range.end += 1;
            }
            Task269gtInputMutation::HeadSpelling => {
                input.expressions[1].head_spelling = "object".to_owned();
            }
            Task269gtInputMutation::Form => {
                input.expressions[1].form = SourceTypeApplicationForm::Of;
            }
            Task269gtInputMutation::Head => {
                input.expressions[1].head = SourceTypeHead::BuiltinObject;
            }
            Task269gtInputMutation::Recovery => {
                input.expressions[1].recovery = NodeRecoveryState::Recovered;
            }
            Task269gtInputMutation::Argument => {
                input.arguments.push(SourceTypeArgumentInput {
                    parent: SourceTypeExpressionId::new(1),
                    ordinal: 0,
                    argument: SourceTypeArgument::TypeSite {
                        expression: SourceTypeExpressionId::new(0),
                    },
                });
            }
        }
        input
    }

    fn mutated_task269gupt_arena(
        source: SourceId,
        node: usize,
        mutation: Task269gtArenaMutation,
    ) -> TypedArena {
        let mut nodes = vec![
            TypedNode::new(
                "source.proof-local.given-use.reserve-type",
                SourceAnchor::Range(range(source, 14, 17)),
            ),
            TypedNode::new(
                "source.proof-local.given-use.type",
                SourceAnchor::Range(range(source, 84, 87)),
            ),
            TypedNode::new(
                "source.proof-local.given-use.type-root",
                SourceAnchor::Range(range(source, 0, 127)),
            )
            .with_children(vec![TypedNodeId::new(0), TypedNodeId::new(1)]),
        ];
        match mutation {
            Task269gtArenaMutation::Kind => {
                nodes[node].kind = "source.proof-local.given-use.wrong".into();
            }
            Task269gtArenaMutation::ResolvedNode => {
                use mizar_syntax as syntax;

                let mut builder = mizar_resolve::resolved_ast::ResolvedArenaBuilder::new();
                let resolved = builder
                    .push(mizar_resolve::resolved_ast::ResolvedNode::new(
                        syntax::SurfaceNodeKind::CompilationUnit,
                        Vec::new(),
                        SemanticOrigin::new(
                            source,
                            module("task269gupt.resolved"),
                            SourceAnchor::Range(range(source, 0, 127)),
                            vec![node as u32],
                        ),
                    ))
                    .expect("Task269GUPT resolved-node mutation id");
                nodes[node].resolved_node = Some(resolved);
            }
            Task269gtArenaMutation::Anchor => {
                nodes[node].anchor = SourceAnchor::Range(range(source, 1, 2));
            }
            Task269gtArenaMutation::Children => {
                nodes[node].children = if node == 2 {
                    vec![TypedNodeId::new(1), TypedNodeId::new(0)]
                } else if node == 0 {
                    vec![TypedNodeId::new(1)]
                } else {
                    vec![TypedNodeId::new(0)]
                };
            }
            Task269gtArenaMutation::Typing => {
                nodes[node].typing = TypingState::Successful;
            }
            Task269gtArenaMutation::Recovery => {
                nodes[node].recovery = NodeRecoveryState::Recovered;
            }
            Task269gtArenaMutation::Links => {
                nodes[node]
                    .links
                    .facts
                    .push(crate::typed_ast::TypeFactId::new(0));
            }
        }
        TypedArena::try_new(Some(TypedNodeId::new(2)), nodes)
            .expect("Task269GUPT mutated arena remains structurally constructible")
    }

    #[test]
    fn task269ct_exact_transaction_fingerprints_and_overlay_are_stable() {
        let fixture = task269ct_fixture();
        let handoff = SourceProofLocalLetTypeProducer::build(
            fixture.dependency.clone(),
            fixture.input.clone(),
            &fixture.symbols,
            &fixture.arena,
        )
        .expect("Task269CT exact checker transaction");
        assert_eq!(handoff.source_id(), fixture.source);
        assert_eq!(handoff.module_id(), &fixture.module);
        assert_eq!(handoff.dependency(), &fixture.dependency);
        assert_eq!(
            handoff.dependency_fingerprint(),
            fixture.dependency.debug_text()
        );
        assert_eq!(
            handoff.binding_fingerprint(),
            handoff.binding_env().debug_text()
        );
        assert_eq!(
            handoff.source_type_fingerprint(),
            handoff.source_type().debug_text()
        );
        assert_eq!(
            (
                handoff.binding_env().contexts().len(),
                handoff.binding_env().bindings().len(),
                handoff.binding_env().diagnostics().len(),
            ),
            (2, 2, 0)
        );
        assert_eq!(handoff.binding_env().source_id(), fixture.source);
        assert_eq!(handoff.binding_env().module_id(), &fixture.module);
        assert_eq!(
            handoff.binding_env().contexts(),
            handoff.dependency().binding_env().contexts()
        );
        assert_eq!(
            handoff.binding_env().diagnostics(),
            handoff.dependency().binding_env().diagnostics()
        );
        assert_eq!(
            handoff.binding_env().bindings().get(BindingId::new(0)),
            handoff
                .dependency()
                .binding_env()
                .bindings()
                .get(BindingId::new(0))
        );
        assert_eq!(handoff.binding_env().source_id(), fixture.source);
        assert_eq!(handoff.binding_env().module_id(), &fixture.module);
        assert_eq!(
            handoff.binding_env().contexts(),
            handoff.dependency().binding_env().contexts()
        );
        assert_eq!(
            handoff.binding_env().diagnostics(),
            handoff.dependency().binding_env().diagnostics()
        );
        assert_eq!(
            handoff.binding_env().bindings().get(BindingId::new(0)),
            handoff
                .dependency()
                .binding_env()
                .bindings()
                .get(BindingId::new(0))
        );
        assert_eq!(
            handoff
                .dependency()
                .binding_env()
                .bindings()
                .get(BindingId::new(1))
                .expect("Task269CT dependency binding")
                .type_site,
            BindingTypeSite::Missing
        );
        assert_eq!(
            handoff
                .binding_env()
                .bindings()
                .get(BindingId::new(1))
                .expect("Task269CT typed binding")
                .type_site,
            BindingTypeSite::Source(range(fixture.source, 76, 79))
        );
        let mut expected_local = handoff
            .dependency()
            .binding_env()
            .bindings()
            .get(BindingId::new(1))
            .expect("Task269CT dependency local binding")
            .clone();
        expected_local.type_site = BindingTypeSite::Source(range(fixture.source, 76, 79));
        assert_eq!(
            handoff.binding_env().bindings().get(BindingId::new(1)),
            Some(&expected_local)
        );
        assert_eq!(
            (
                handoff.source_type().applications().len(),
                handoff.source_type().expressions().len(),
                handoff.source_type().arguments().len(),
                handoff.source_type().definition_returns().len(),
                handoff.source_type().mode_rhs().len(),
                handoff.source_type().structure_members().len(),
            ),
            (2, 2, 0, 0, 0, 0)
        );
        assert_eq!(handoff.source_type().source_id(), fixture.source);
        assert_eq!(handoff.source_type().module_id(), &fixture.module);
        for (index, (start, end)) in [(14, 17), (76, 79)].into_iter().enumerate() {
            let application = handoff
                .source_type()
                .applications()
                .get(SourceTypeApplicationId::new(index))
                .expect("Task269CT source type application");
            assert_eq!(application.id(), SourceTypeApplicationId::new(index));
            assert_eq!(application.binding(), BindingId::new(index));
            assert_eq!(application.source_ordinal(), index);
            assert_eq!(application.root(), SourceTypeExpressionId::new(index));

            let expression = handoff
                .source_type()
                .expressions()
                .get(SourceTypeExpressionId::new(index))
                .expect("Task269CT source type expression");
            assert_eq!(expression.id(), SourceTypeExpressionId::new(index));
            assert_eq!(expression.source_id(), fixture.source);
            assert_eq!(expression.module_id(), &fixture.module);
            assert!(task269ct_role_matches(
                expression.site(),
                TypedNodeId::new(index),
                "source.type.expression",
            ));
            assert_eq!(expression.source_range(), range(fixture.source, start, end));
            assert_eq!(expression.spelling(), "set");
            assert!(task269ct_role_matches(
                expression.head_site(),
                TypedNodeId::new(index),
                "source.type.head",
            ));
            assert_eq!(expression.head_range(), range(fixture.source, start, end));
            assert_eq!(expression.head_spelling(), "set");
            assert_eq!(expression.form(), SourceTypeApplicationForm::Bare);
            assert_eq!(expression.head(), &SourceTypeHead::BuiltinSet);
            assert_eq!(expression.recovery(), NodeRecoveryState::Normal);
        }
        assert_eq!(fixture.arena.root(), Some(TypedNodeId::new(2)));
        assert_eq!(fixture.arena.len(), 3);
        for (index, (kind, start, end, children)) in [
            ("source.proof-local.let.reserve-type", 14, 17, Vec::new()),
            ("source.proof-local.let.type", 76, 79, Vec::new()),
            (
                "source.proof-local.let.type-root",
                0,
                99,
                vec![TypedNodeId::new(0), TypedNodeId::new(1)],
            ),
        ]
        .into_iter()
        .enumerate()
        {
            assert_eq!(
                fixture.arena.node(TypedNodeId::new(index)),
                Some(
                    &TypedNode::new(kind, SourceAnchor::Range(range(fixture.source, start, end)),)
                        .with_children(children)
                )
            );
        }
        assert_eq!(
            handoff.debug_text(),
            format!(
                concat!(
                    "source-proof-local-let-type-debug-v1\n",
                    "module: pkg::task269c\n",
                    "dependency-fingerprint: {:?}\n",
                    "binding-fingerprint: {:?}\n",
                    "source-type-fingerprint: {:?}\n",
                ),
                handoff.dependency_fingerprint(),
                handoff.binding_fingerprint(),
                handoff.source_type_fingerprint(),
            )
        );
    }

    #[test]
    fn task269ct_corruption_classes_and_validation_precedence_are_frozen() {
        let fixture = task269ct_fixture();
        let mut wrong_dependency = fixture.input.clone();
        wrong_dependency.module_id = module("task269ct.wrong");
        for expression in &mut wrong_dependency.expressions {
            expression.module_id = wrong_dependency.module_id.clone();
        }
        assert_eq!(
            SourceProofLocalLetTypeProducer::build(
                fixture.dependency.clone(),
                wrong_dependency,
                &fixture.symbols,
                &fixture.arena,
            ),
            Err(SourceProofLocalLetTypeError::InvalidDependency)
        );
        let mut wrong_source_type = fixture.input.clone();
        wrong_source_type.expressions[1].source_range.end += 1;
        assert_eq!(
            SourceProofLocalLetTypeProducer::build(
                fixture.dependency.clone(),
                wrong_source_type,
                &fixture.symbols,
                &fixture.arena,
            ),
            Err(SourceProofLocalLetTypeError::InvalidSourceType)
        );

        let handoff = SourceProofLocalLetTypeProducer::build(
            fixture.dependency,
            fixture.input,
            &fixture.symbols,
            &fixture.arena,
        )
        .expect("Task269CT valid handoff");
        let mut dependency_corrupt = handoff.clone();
        dependency_corrupt
            .dependency_fingerprint
            .push_str("corrupt");
        dependency_corrupt.binding_fingerprint.push_str("corrupt");
        dependency_corrupt
            .source_type_fingerprint
            .push_str("corrupt");
        assert_eq!(
            dependency_corrupt.validate_complete_installation(
                fixture.source,
                &fixture.module,
                &fixture.arena,
                false,
            ),
            Err(SourceProofLocalLetTypeError::InvalidDependency)
        );
        let mut binding_corrupt = handoff.clone();
        binding_corrupt
            .binding_env
            .binding_mut_for_test(BindingId::new(1))
            .expect("Task269CT mutable local binding")
            .type_site = BindingTypeSite::Missing;
        binding_corrupt.binding_fingerprint = binding_corrupt.binding_env.debug_text();
        binding_corrupt.source_type.expressions.entries[1]
            .source_range
            .end += 1;
        binding_corrupt.source_type_fingerprint = binding_corrupt.source_type.debug_text();
        assert_eq!(
            binding_corrupt.validate_complete_installation(
                fixture.source,
                &fixture.module,
                &fixture.arena,
                false,
            ),
            Err(SourceProofLocalLetTypeError::InvalidBindingEnvironment)
        );
        let mut source_corrupt = handoff.clone();
        source_corrupt.source_type.expressions.entries[1]
            .source_range
            .end += 1;
        source_corrupt.source_type_fingerprint = source_corrupt.source_type.debug_text();
        assert_eq!(
            source_corrupt.validate_complete_installation(
                fixture.source,
                &fixture.module,
                &fixture.arena,
                false,
            ),
            Err(SourceProofLocalLetTypeError::InvalidSourceType)
        );
        assert_eq!(
            handoff.validate_installation(
                fixture.source,
                &fixture.module,
                &task269ct_test_arena(fixture.source, 1, false),
            ),
            Err(SourceProofLocalLetTypeError::InvalidSourceType)
        );
        assert_eq!(
            handoff.validate_complete_installation(
                fixture.source,
                &fixture.module,
                &fixture.arena,
                false,
            ),
            Err(SourceProofLocalLetTypeError::InvalidInstallation)
        );
    }

    #[test]
    fn task269ct_typed_and_final_ownership_is_one_shot_and_cross_family_atomic() {
        let fixture = task269ct_fixture();
        let handoff = SourceProofLocalLetTypeProducer::build(
            fixture.dependency.clone(),
            fixture.input,
            &fixture.symbols,
            &fixture.arena,
        )
        .expect("Task269CT handoff");
        let typed = task269ct_empty_typed(
            fixture.source,
            fixture.module.clone(),
            fixture.arena.clone(),
        )
        .with_source_proof_local_let_type(handoff.clone())
        .expect("Task269CT typed installation");
        assert_eq!(typed.source_proof_local_let_type(), Some(&handoff));
        assert!(typed.source_proof_local_let_binding().is_none());
        assert!(typed.source_type().is_none());
        assert_eq!(
            typed
                .clone()
                .with_source_proof_local_let_type(handoff.clone()),
            Err(TypedAstError::InvalidSourceProofLocalLetType)
        );
        assert_eq!(
            typed
                .clone()
                .with_source_proof_local_let_binding(fixture.dependency.clone()),
            Err(TypedAstError::InvalidSourceProofLocalLetBinding)
        );
        let direct_dependency = task269ct_empty_typed(
            fixture.source,
            fixture.module.clone(),
            TypedArena::try_new(None, Vec::new()).expect("Task269C empty arena"),
        )
        .with_source_proof_local_let_binding(fixture.dependency)
        .expect("Task269C direct typed installation");
        assert_eq!(
            direct_dependency.with_source_proof_local_let_type(handoff.clone()),
            Err(TypedAstError::InvalidSourceProofLocalLetType)
        );

        let resolved = assemble_empty_resolved(&typed);
        assert_eq!(resolved.source_proof_local_let_type(), Some(&handoff));
        assert!(resolved.source_proof_local_let_binding().is_none());
        assert!(resolved.source_type().is_none());
        assert_eq!(resolved.nodes().len(), 3);
        assert_eq!(
            resolved.nodes().root(),
            Some(crate::resolved_typed_ast::ResolvedTypedNodeId::new(2))
        );
        for (_, node) in resolved.nodes().iter() {
            assert!(matches!(
                &node.kind,
                crate::resolved_typed_ast::ResolvedTypedNodeKind::SourcePreserved { role }
                    if role.as_str() == "source.proof-local.let.type"
            ));
        }

        let statement_hints = (0..3)
            .map(|index| ResolvedNodeKindHint {
                typed_node: TypedNodeId::new(index),
                kind: ResolvedNodeKindHintKind::SourcePreserved {
                    role: SourceNodeRole::new("source.statement.transport"),
                },
            })
            .collect();
        assert_eq!(
            assemble_task269ct_resolved(&typed, Vec::new(), statement_hints),
            Err(ResolvedTypedAstError::InvalidSourceProofLocalLetType)
        );
        assert_eq!(
            assemble_task269ct_resolved(
                &typed,
                vec![ExpressionMetadataInput {
                    expr: ExprId::new("task269ct.semantic-input"),
                    typed_site: role(0, "source.type.expression"),
                    local_context: None,
                    cluster_facts: Vec::new(),
                }],
                Vec::new(),
            ),
            Err(ResolvedTypedAstError::InvalidSourceProofLocalLetType)
        );
    }

    #[test]
    fn task269ct_generic_admission_and_semantic_owners_remain_isolated() {
        let fixture = task269ct_fixture();
        let handoff = SourceProofLocalLetTypeProducer::build(
            fixture.dependency,
            fixture.input.clone(),
            &fixture.symbols,
            &fixture.arena,
        )
        .expect("Task269CT handoff");
        assert_eq!(
            SourceTypeProducer::build(
                fixture.input,
                handoff.binding_env(),
                &fixture.symbols,
                &fixture.arena,
            ),
            Err(SourceTypeError::InvalidBinding {
                application: SourceTypeApplicationId::new(1),
            })
        );
        let typed = task269ct_empty_typed(fixture.source, fixture.module, fixture.arena)
            .with_source_proof_local_let_type(handoff)
            .expect("Task269CT typed installation");
        assert!(typed.contexts().is_empty());
        assert!(typed.types().is_empty());
        assert!(typed.facts().is_empty());
        assert!(typed.coercions().is_empty());
        assert!(typed.initial_obligations().is_empty());
        assert!(typed.diagnostics().is_empty());
        let resolved = assemble_empty_resolved(&typed);
        assert!(resolved.source_context().is_none());
        assert!(resolved.source_attribute().is_none());
        assert!(resolved.source_evidence().is_none());
        assert!(resolved.source_term().is_none());
        assert!(resolved.source_application().is_none());
        assert!(resolved.source_structure().is_none());
        assert!(resolved.source_set_term().is_none());
        assert!(resolved.source_atomic_formula().is_none());
        assert!(resolved.source_attribute_definition().is_none());
        assert!(resolved.source_functor_definition().is_none());
        assert!(resolved.source_property_implementation().is_none());
        assert!(resolved.source_mode_definition().is_none());
        assert!(resolved.source_structure_definition().is_none());
        assert!(resolved.source_predicate_definition().is_none());
        assert!(resolved.source_composite_formula().is_none());
        assert!(resolved.source_formula_composition().is_none());
        assert!(resolved.source_condition_formula_composition().is_none());
        assert!(resolved.source_predicate_chain_composition().is_none());
        assert!(resolved.source_statement().is_none());
        assert!(resolved.source_statement_references().is_none());
        assert!(resolved.source_statement_witnesses().is_none());
        assert!(resolved.source_proof_local_declaration().is_none());
        assert!(resolved.source_proof_local_let_binding().is_none());
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
        assert!(!resolved.debug_text().contains("initial-obligation#"));
    }

    #[test]
    fn task269gt_exact_transaction_fingerprints_and_overlay_are_stable() {
        let fixture = task269gt_fixture();
        let handoff = SourceProofLocalGivenTypeProducer::build(
            fixture.dependency.clone(),
            fixture.input,
            &fixture.symbols,
            &fixture.arena,
        )
        .expect("Task269GT exact checker transaction");
        assert_eq!(handoff.source_id(), fixture.source);
        assert_eq!(handoff.module_id(), &fixture.module);
        assert_eq!(handoff.dependency(), &fixture.dependency);
        assert_eq!(
            handoff.dependency_fingerprint(),
            fixture.dependency.debug_text()
        );
        assert_eq!(
            handoff.binding_fingerprint(),
            handoff.binding_env().debug_text()
        );
        assert_eq!(
            handoff.source_type_fingerprint(),
            handoff.source_type().debug_text()
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
            handoff
                .dependency()
                .binding_env()
                .bindings()
                .get(BindingId::new(1))
                .expect("Task269GT dependency binding")
                .type_site,
            BindingTypeSite::Missing
        );
        assert_eq!(
            handoff
                .binding_env()
                .bindings()
                .get(BindingId::new(1))
                .expect("Task269GT typed binding")
                .type_site,
            BindingTypeSite::Source(range(fixture.source, 84, 87))
        );
        let mut expected_local = handoff
            .dependency()
            .binding_env()
            .bindings()
            .get(BindingId::new(1))
            .expect("Task269GT dependency local binding")
            .clone();
        expected_local.type_site = BindingTypeSite::Source(range(fixture.source, 84, 87));
        assert_eq!(
            handoff.binding_env().bindings().get(BindingId::new(1)),
            Some(&expected_local)
        );
        assert_eq!(
            (
                handoff.source_type().applications().len(),
                handoff.source_type().expressions().len(),
                handoff.source_type().arguments().len(),
                handoff.source_type().definition_returns().len(),
                handoff.source_type().mode_rhs().len(),
                handoff.source_type().structure_members().len(),
            ),
            (2, 2, 0, 0, 0, 0)
        );
        assert_eq!(handoff.source_type().source_id(), fixture.source);
        assert_eq!(handoff.source_type().module_id(), &fixture.module);
        for (index, (start, end)) in [(14, 17), (84, 87)].into_iter().enumerate() {
            let application = handoff
                .source_type()
                .applications()
                .get(SourceTypeApplicationId::new(index))
                .expect("Task269GT source type application");
            assert_eq!(application.id(), SourceTypeApplicationId::new(index));
            assert_eq!(application.binding(), BindingId::new(index));
            assert_eq!(application.source_ordinal(), index);
            assert_eq!(application.root(), SourceTypeExpressionId::new(index));
            let expression = handoff
                .source_type()
                .expressions()
                .get(SourceTypeExpressionId::new(index))
                .expect("Task269GT source type expression");
            assert_eq!(expression.id(), SourceTypeExpressionId::new(index));
            assert_eq!(expression.source_id(), fixture.source);
            assert_eq!(expression.module_id(), &fixture.module);
            assert!(task269ct_role_matches(
                expression.site(),
                TypedNodeId::new(index),
                "source.type.expression",
            ));
            assert_eq!(expression.source_range(), range(fixture.source, start, end));
            assert_eq!(expression.spelling(), "set");
            assert!(task269ct_role_matches(
                expression.head_site(),
                TypedNodeId::new(index),
                "source.type.head",
            ));
            assert_eq!(expression.head_range(), range(fixture.source, start, end));
            assert_eq!(expression.head_spelling(), "set");
            assert_eq!(expression.form(), SourceTypeApplicationForm::Bare);
            assert_eq!(expression.head(), &SourceTypeHead::BuiltinSet);
            assert_eq!(expression.recovery(), NodeRecoveryState::Normal);
        }
        assert_eq!(fixture.arena.root(), Some(TypedNodeId::new(2)));
        assert_eq!(fixture.arena.len(), 3);
        for (index, (kind, start, end, children)) in [
            ("source.proof-local.given.reserve-type", 14, 17, Vec::new()),
            ("source.proof-local.given.type", 84, 87, Vec::new()),
            (
                "source.proof-local.given.type-root",
                0,
                128,
                vec![TypedNodeId::new(0), TypedNodeId::new(1)],
            ),
        ]
        .into_iter()
        .enumerate()
        {
            assert_eq!(
                fixture.arena.node(TypedNodeId::new(index)),
                Some(
                    &TypedNode::new(kind, SourceAnchor::Range(range(fixture.source, start, end)))
                        .with_children(children)
                )
            );
        }
        assert_eq!(
            handoff.debug_text(),
            format!(
                concat!(
                    "source-proof-local-given-type-debug-v1\n",
                    "module: pkg::task269g\n",
                    "dependency-fingerprint: {:?}\n",
                    "binding-fingerprint: {:?}\n",
                    "source-type-fingerprint: {:?}\n",
                ),
                handoff.dependency_fingerprint(),
                handoff.binding_fingerprint(),
                handoff.source_type_fingerprint(),
            )
        );
    }

    #[test]
    fn task269gt_dependency_binding_source_type_and_precedence_fail_closed() {
        let fixture = task269gt_fixture();
        let mut wrong_dependency = fixture.input.clone();
        wrong_dependency.module_id = module("task269gt.wrong");
        for expression in &mut wrong_dependency.expressions {
            expression.module_id = wrong_dependency.module_id.clone();
        }
        assert_eq!(
            SourceProofLocalGivenTypeProducer::build(
                fixture.dependency.clone(),
                wrong_dependency,
                &fixture.symbols,
                &fixture.arena,
            ),
            Err(SourceProofLocalGivenTypeError::InvalidDependency)
        );
        let mut wrong_source = fixture.input.clone();
        wrong_source.source_id = other_source_id();
        for expression in &mut wrong_source.expressions {
            expression.source_id = wrong_source.source_id;
            expression.source_range.source_id = wrong_source.source_id;
            expression.head_range.source_id = wrong_source.source_id;
        }
        assert_eq!(
            SourceProofLocalGivenTypeProducer::build(
                fixture.dependency.clone(),
                wrong_source,
                &fixture.symbols,
                &fixture.arena,
            ),
            Err(SourceProofLocalGivenTypeError::InvalidDependency)
        );
        for mutation in [
            Task269gtInputMutation::ApplicationCount,
            Task269gtInputMutation::ApplicationBinding,
            Task269gtInputMutation::ApplicationOrdinal,
            Task269gtInputMutation::ApplicationRoot,
            Task269gtInputMutation::ExpressionCount,
            Task269gtInputMutation::ExpressionSource,
            Task269gtInputMutation::ExpressionModule,
            Task269gtInputMutation::ExpressionSite,
            Task269gtInputMutation::ExpressionRange,
            Task269gtInputMutation::ExpressionSpelling,
            Task269gtInputMutation::HeadSite,
            Task269gtInputMutation::HeadRange,
            Task269gtInputMutation::HeadSpelling,
            Task269gtInputMutation::Form,
            Task269gtInputMutation::Head,
            Task269gtInputMutation::Recovery,
            Task269gtInputMutation::Argument,
        ] {
            assert_eq!(
                SourceProofLocalGivenTypeProducer::build(
                    fixture.dependency.clone(),
                    mutated_task269gt_input(fixture.source, fixture.module.clone(), mutation),
                    &fixture.symbols,
                    &fixture.arena,
                ),
                Err(SourceProofLocalGivenTypeError::InvalidSourceType),
                "Task269GT input mutation {mutation:?}",
            );
        }
        let wrong_symbols = SymbolEnv::new(
            module("task269gt.symbols.wrong"),
            SymbolEnvIndexes::default(),
        );
        assert_eq!(
            SourceProofLocalGivenTypeProducer::build(
                fixture.dependency.clone(),
                fixture.input.clone(),
                &wrong_symbols,
                &fixture.arena,
            ),
            Err(SourceProofLocalGivenTypeError::InvalidSourceType)
        );
        for node in 0..3 {
            for mutation in [
                Task269gtArenaMutation::Kind,
                Task269gtArenaMutation::ResolvedNode,
                Task269gtArenaMutation::Anchor,
                Task269gtArenaMutation::Children,
                Task269gtArenaMutation::Typing,
                Task269gtArenaMutation::Recovery,
                Task269gtArenaMutation::Links,
            ] {
                assert_eq!(
                    SourceProofLocalGivenTypeProducer::build(
                        fixture.dependency.clone(),
                        fixture.input.clone(),
                        &fixture.symbols,
                        &mutated_task269gt_arena(fixture.source, node, mutation),
                    ),
                    Err(SourceProofLocalGivenTypeError::InvalidSourceType),
                    "Task269GT arena node {node} mutation {mutation:?}",
                );
            }
        }
        let handoff = SourceProofLocalGivenTypeProducer::build(
            fixture.dependency.clone(),
            fixture.input.clone(),
            &fixture.symbols,
            &fixture.arena,
        )
        .expect("Task269GT valid handoff");
        let mut dependency_lower_corrupt = handoff.clone();
        dependency_lower_corrupt
            .dependency
            .set_lower_fingerprint_for_test("corrupt");
        dependency_lower_corrupt.dependency_fingerprint =
            dependency_lower_corrupt.dependency.debug_text();
        assert_eq!(
            dependency_lower_corrupt.validate_installation(
                fixture.source,
                &fixture.module,
                &fixture.arena,
            ),
            Err(SourceProofLocalGivenTypeError::InvalidDependency)
        );
        let mut dependency_base_corrupt = handoff.clone();
        dependency_base_corrupt
            .dependency
            .set_base_binding_fingerprint_for_task269g_test("corrupt");
        dependency_base_corrupt.dependency_fingerprint =
            dependency_base_corrupt.dependency.debug_text();
        assert_eq!(
            dependency_base_corrupt.validate_installation(
                fixture.source,
                &fixture.module,
                &fixture.arena,
            ),
            Err(SourceProofLocalGivenTypeError::InvalidDependency)
        );
        let mut dependency_aggregate_corrupt = handoff.clone();
        dependency_aggregate_corrupt
            .dependency
            .truncate_task269g_bindings_for_test();
        assert_eq!(
            dependency_aggregate_corrupt.validate_installation(
                fixture.source,
                &fixture.module,
                &fixture.arena,
            ),
            Err(SourceProofLocalGivenTypeError::InvalidDependency)
        );
        let mut dependency_row_corrupt = handoff.clone();
        dependency_row_corrupt
            .dependency
            .corrupt_task269g_binding_row_for_test();
        dependency_row_corrupt.dependency_fingerprint =
            dependency_row_corrupt.dependency.debug_text();
        assert_eq!(
            dependency_row_corrupt.validate_installation(
                fixture.source,
                &fixture.module,
                &fixture.arena,
            ),
            Err(SourceProofLocalGivenTypeError::InvalidDependency)
        );
        let mut dependency_final_corrupt = handoff.clone();
        dependency_final_corrupt
            .dependency
            .set_final_binding_fingerprint_for_task269g_test("corrupt");
        dependency_final_corrupt.dependency_fingerprint =
            dependency_final_corrupt.dependency.debug_text();
        assert_eq!(
            dependency_final_corrupt.validate_installation(
                fixture.source,
                &fixture.module,
                &fixture.arena,
            ),
            Err(SourceProofLocalGivenTypeError::InvalidDependency)
        );
        let mut dependency_corrupt = handoff.clone();
        dependency_corrupt
            .dependency_fingerprint
            .push_str("corrupt");
        dependency_corrupt.binding_fingerprint.push_str("corrupt");
        dependency_corrupt
            .source_type_fingerprint
            .push_str("corrupt");
        assert_eq!(
            dependency_corrupt.validate_complete_installation(
                fixture.source,
                &fixture.module,
                &fixture.arena,
                false,
            ),
            Err(SourceProofLocalGivenTypeError::InvalidDependency)
        );
        let mut binding_corrupt = handoff.clone();
        binding_corrupt
            .binding_env
            .binding_mut_for_test(BindingId::new(1))
            .expect("Task269GT mutable given binding")
            .type_site = BindingTypeSite::Missing;
        binding_corrupt.binding_fingerprint = binding_corrupt.binding_env.debug_text();
        binding_corrupt.source_type.expressions.entries[1]
            .source_range
            .end += 1;
        binding_corrupt.source_type_fingerprint = binding_corrupt.source_type.debug_text();
        assert_eq!(
            binding_corrupt.validate_complete_installation(
                fixture.source,
                &fixture.module,
                &fixture.arena,
                false,
            ),
            Err(SourceProofLocalGivenTypeError::InvalidBindingEnvironment)
        );
        let mut binding_fingerprint_corrupt = handoff.clone();
        binding_fingerprint_corrupt
            .binding_fingerprint
            .push_str("corrupt");
        assert_eq!(
            binding_fingerprint_corrupt.validate_installation(
                fixture.source,
                &fixture.module,
                &fixture.arena,
            ),
            Err(SourceProofLocalGivenTypeError::InvalidBindingEnvironment)
        );
        for binding_id in 0..2 {
            let mut binding_field_corrupt = handoff.clone();
            binding_field_corrupt
                .binding_env
                .binding_mut_for_test(BindingId::new(binding_id))
                .expect("Task269GT mutable binding")
                .spelling
                .push_str("corrupt");
            binding_field_corrupt.binding_fingerprint =
                binding_field_corrupt.binding_env.debug_text();
            assert_eq!(
                binding_field_corrupt.validate_installation(
                    fixture.source,
                    &fixture.module,
                    &fixture.arena,
                ),
                Err(SourceProofLocalGivenTypeError::InvalidBindingEnvironment),
                "Task269GT binding row {binding_id} non-type field",
            );
        }
        let mut source_corrupt = handoff.clone();
        source_corrupt.source_type.expressions.entries[1]
            .source_range
            .end += 1;
        source_corrupt.source_type_fingerprint = source_corrupt.source_type.debug_text();
        assert_eq!(
            source_corrupt.validate_complete_installation(
                fixture.source,
                &fixture.module,
                &fixture.arena,
                false,
            ),
            Err(SourceProofLocalGivenTypeError::InvalidSourceType)
        );
        let mut source_fingerprint_corrupt = handoff.clone();
        source_fingerprint_corrupt
            .source_type_fingerprint
            .push_str("corrupt");
        assert_eq!(
            source_fingerprint_corrupt.validate_installation(
                fixture.source,
                &fixture.module,
                &fixture.arena,
            ),
            Err(SourceProofLocalGivenTypeError::InvalidSourceType)
        );
        assert_eq!(
            handoff.validate_installation(
                fixture.source,
                &fixture.module,
                &task269gt_test_arena(fixture.source, 1, false),
            ),
            Err(SourceProofLocalGivenTypeError::InvalidSourceType)
        );
        assert_eq!(
            handoff.validate_complete_installation(
                fixture.source,
                &fixture.module,
                &fixture.arena,
                false,
            ),
            Err(SourceProofLocalGivenTypeError::InvalidInstallation)
        );
    }

    #[test]
    fn task269gt_typed_and_resolved_ownership_is_atomic() {
        let fixture = task269gt_fixture();
        let handoff = SourceProofLocalGivenTypeProducer::build(
            fixture.dependency.clone(),
            fixture.input,
            &fixture.symbols,
            &fixture.arena,
        )
        .expect("Task269GT handoff");
        let typed = task269ct_empty_typed(
            fixture.source,
            fixture.module.clone(),
            fixture.arena.clone(),
        )
        .with_source_proof_local_given_type(handoff.clone())
        .expect("Task269GT typed installation");
        assert_eq!(typed.source_proof_local_given_type(), Some(&handoff));
        assert!(typed.source_proof_local_given_binding().is_none());
        assert!(typed.source_type().is_none());
        assert_eq!(
            typed
                .clone()
                .with_source_proof_local_given_type(handoff.clone()),
            Err(TypedAstError::InvalidSourceProofLocalGivenType)
        );
        assert_eq!(
            typed
                .clone()
                .with_source_proof_local_given_binding(fixture.dependency.clone()),
            Err(TypedAstError::InvalidSourceProofLocalGivenBinding)
        );
        let direct_dependency = task269ct_empty_typed(
            fixture.source,
            fixture.module.clone(),
            TypedArena::try_new(None, Vec::new()).expect("Task269GT empty arena"),
        )
        .with_source_proof_local_given_binding(fixture.dependency)
        .expect("Task269G direct typed installation");
        assert_eq!(
            direct_dependency.with_source_proof_local_given_type(handoff.clone()),
            Err(TypedAstError::InvalidSourceProofLocalGivenType)
        );
        let let_fixture = task269ct_fixture();
        let mut occupied_neighbor = task269ct_empty_typed(
            fixture.source,
            fixture.module.clone(),
            fixture.arena.clone(),
        );
        occupied_neighbor.inject_source_proof_local_let_binding_for_test(let_fixture.dependency);
        assert_eq!(
            occupied_neighbor.with_source_proof_local_given_type(handoff.clone()),
            Err(TypedAstError::InvalidSourceProofLocalGivenType)
        );

        let resolved = assemble_empty_resolved(&typed);
        assert_eq!(resolved.source_proof_local_given_type(), Some(&handoff));
        assert!(resolved.source_proof_local_given_binding().is_none());
        assert_eq!(resolved.nodes().len(), 3);
        for (_, node) in resolved.nodes().iter() {
            assert!(matches!(
                &node.kind,
                crate::resolved_typed_ast::ResolvedTypedNodeKind::SourcePreserved { role }
                    if role.as_str() == "source.proof-local.given.type"
            ));
        }
        let statement_hints = (0..3)
            .map(|index| ResolvedNodeKindHint {
                typed_node: TypedNodeId::new(index),
                kind: ResolvedNodeKindHintKind::SourcePreserved {
                    role: SourceNodeRole::new("source.statement.transport"),
                },
            })
            .collect();
        assert_eq!(
            assemble_task269ct_resolved(&typed, Vec::new(), statement_hints),
            Err(ResolvedTypedAstError::InvalidSourceProofLocalGivenType)
        );
        assert_eq!(
            assemble_task269ct_resolved(
                &typed,
                vec![ExpressionMetadataInput {
                    expr: ExprId::new("task269gt.semantic-input"),
                    typed_site: role(0, "source.type.expression"),
                    local_context: None,
                    cluster_facts: Vec::new(),
                }],
                Vec::new(),
            ),
            Err(ResolvedTypedAstError::InvalidSourceProofLocalGivenType)
        );
    }

    #[test]
    fn task269gt_generic_and_neighbor_routes_remain_isolated() {
        let fixture = task269gt_fixture();
        let handoff = SourceProofLocalGivenTypeProducer::build(
            fixture.dependency,
            fixture.input.clone(),
            &fixture.symbols,
            &fixture.arena,
        )
        .expect("Task269GT handoff");
        assert_eq!(
            SourceTypeProducer::build(
                fixture.input,
                handoff.binding_env(),
                &fixture.symbols,
                &fixture.arena,
            ),
            Err(SourceTypeError::InvalidBinding {
                application: SourceTypeApplicationId::new(1),
            })
        );
        let typed = task269ct_empty_typed(fixture.source, fixture.module, fixture.arena)
            .with_source_proof_local_given_type(handoff)
            .expect("Task269GT typed installation");
        assert!(typed.source_proof_local_let_binding().is_none());
        assert!(typed.source_proof_local_let_type().is_none());
        assert!(typed.contexts().is_empty());
        assert!(typed.types().is_empty());
        assert!(typed.facts().is_empty());
        assert!(typed.coercions().is_empty());
        assert!(typed.initial_obligations().is_empty());
        assert!(typed.diagnostics().is_empty());
        let resolved = assemble_empty_resolved(&typed);
        assert!(resolved.source_context().is_none());
        assert!(resolved.source_type().is_none());
        assert!(resolved.source_attribute().is_none());
        assert!(resolved.source_evidence().is_none());
        assert!(resolved.source_term().is_none());
        assert!(resolved.source_application().is_none());
        assert!(resolved.source_structure().is_none());
        assert!(resolved.source_set_term().is_none());
        assert!(resolved.source_atomic_formula().is_none());
        assert!(resolved.source_attribute_definition().is_none());
        assert!(resolved.source_functor_definition().is_none());
        assert!(resolved.source_property_implementation().is_none());
        assert!(resolved.source_mode_definition().is_none());
        assert!(resolved.source_structure_definition().is_none());
        assert!(resolved.source_predicate_definition().is_none());
        assert!(resolved.source_composite_formula().is_none());
        assert!(resolved.source_formula_composition().is_none());
        assert!(resolved.source_condition_formula_composition().is_none());
        assert!(resolved.source_predicate_chain_composition().is_none());
        assert!(resolved.source_statement().is_none());
        assert!(resolved.source_statement_references().is_none());
        assert!(resolved.source_statement_witnesses().is_none());
        assert!(resolved.source_proof_local_declaration().is_none());
        assert!(resolved.source_proof_local_let_binding().is_none());
        assert!(resolved.source_proof_local_let_type().is_none());
        assert!(resolved.source_proof_local_given_binding().is_none());
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
        assert!(!resolved.debug_text().contains("initial-obligation#"));
    }

    #[test]
    fn task269gct_exact_condition_type_composition_is_stable() {
        let fixture = task269gct_fixture();
        let handoff = SourceProofLocalGivenConditionTypeProducer::build(
            fixture.dependency.clone(),
            fixture.input,
            &fixture.symbols,
            &fixture.arena,
        )
        .expect("Task269GCT exact checker transaction");
        assert_eq!(handoff.source_id(), fixture.source);
        assert_eq!(handoff.module_id(), &fixture.module);
        assert_eq!(handoff.dependency(), &fixture.dependency);
        assert_eq!(
            handoff.dependency_fingerprint(),
            fixture.dependency.debug_text()
        );
        assert_eq!(
            handoff.binding_fingerprint(),
            handoff.binding_env().debug_text()
        );
        assert_eq!(
            handoff.source_type_fingerprint(),
            handoff.source_type().debug_text()
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
            handoff
                .dependency()
                .binding_env()
                .bindings()
                .get(BindingId::new(0))
                .expect("Task269GCT dependency reserve")
                .type_site,
            BindingTypeSite::Source(range(fixture.source, 14, 17))
        );
        assert_eq!(
            handoff
                .dependency()
                .binding_env()
                .bindings()
                .get(BindingId::new(1))
                .expect("Task269GCT dependency witness")
                .type_site,
            BindingTypeSite::Missing
        );
        let mut expected_local = handoff
            .dependency()
            .binding_env()
            .bindings()
            .get(BindingId::new(1))
            .expect("Task269GCT dependency local binding")
            .clone();
        expected_local.type_site = BindingTypeSite::Source(range(fixture.source, 90, 93));
        assert_eq!(
            handoff.binding_env().bindings().get(BindingId::new(1)),
            Some(&expected_local)
        );
        assert_eq!(
            (
                handoff.source_type().applications().len(),
                handoff.source_type().expressions().len(),
                handoff.source_type().arguments().len(),
                handoff.source_type().definition_returns().len(),
                handoff.source_type().mode_rhs().len(),
                handoff.source_type().structure_members().len(),
            ),
            (2, 2, 0, 0, 0, 0)
        );
        for (index, (start, end)) in [(14, 17), (90, 93)].into_iter().enumerate() {
            let application = handoff
                .source_type()
                .applications()
                .get(SourceTypeApplicationId::new(index))
                .expect("Task269GCT source type application");
            assert_eq!(
                (
                    application.id(),
                    application.binding(),
                    application.source_ordinal(),
                    application.root(),
                ),
                (
                    SourceTypeApplicationId::new(index),
                    BindingId::new(index),
                    index,
                    SourceTypeExpressionId::new(index),
                )
            );
            let expression = handoff
                .source_type()
                .expressions()
                .get(SourceTypeExpressionId::new(index))
                .expect("Task269GCT source type expression");
            assert_eq!(expression.source_id(), fixture.source);
            assert_eq!(expression.module_id(), &fixture.module);
            assert!(task269ct_role_matches(
                expression.site(),
                TypedNodeId::new(index),
                "source.type.expression",
            ));
            assert_eq!(expression.source_range(), range(fixture.source, start, end));
            assert_eq!(expression.spelling(), "set");
            assert!(task269ct_role_matches(
                expression.head_site(),
                TypedNodeId::new(index),
                "source.type.head",
            ));
            assert_eq!(expression.head_range(), range(fixture.source, start, end));
            assert_eq!(expression.head_spelling(), "set");
            assert_eq!(expression.form(), SourceTypeApplicationForm::Bare);
            assert_eq!(expression.head(), &SourceTypeHead::BuiltinSet);
            assert_eq!(expression.recovery(), NodeRecoveryState::Normal);
        }
        assert!(exact_task269gct_arena(fixture.source, &fixture.arena));
        assert_eq!(
            handoff.debug_text(),
            format!(
                concat!(
                    "source-proof-local-given-condition-type-debug-v1\n",
                    "module: pkg::task269gc\n",
                    "dependency-fingerprint: {:?}\n",
                    "binding-fingerprint: {:?}\n",
                    "source-type-fingerprint: {:?}\n",
                ),
                handoff.dependency_fingerprint(),
                handoff.binding_fingerprint(),
                handoff.source_type_fingerprint(),
            )
        );
        assert!(handoff.debug_text().ends_with("\n"));
    }

    #[test]
    fn task269gct_dependency_binding_input_and_arena_corruption_fail_closed() {
        let fixture = task269gct_fixture();
        let mut wrong_dependency = fixture.input.clone();
        wrong_dependency.module_id = module("task269gct.wrong");
        for expression in &mut wrong_dependency.expressions {
            expression.module_id = wrong_dependency.module_id.clone();
        }
        assert_eq!(
            SourceProofLocalGivenConditionTypeProducer::build(
                fixture.dependency.clone(),
                wrong_dependency,
                &fixture.symbols,
                &fixture.arena,
            ),
            Err(SourceProofLocalGivenConditionTypeError::InvalidDependency)
        );
        let mut wrong_source = fixture.input.clone();
        wrong_source.source_id = other_source_id();
        for expression in &mut wrong_source.expressions {
            expression.source_id = wrong_source.source_id;
            expression.source_range.source_id = wrong_source.source_id;
            expression.head_range.source_id = wrong_source.source_id;
        }
        assert_eq!(
            SourceProofLocalGivenConditionTypeProducer::build(
                fixture.dependency.clone(),
                wrong_source,
                &fixture.symbols,
                &fixture.arena,
            ),
            Err(SourceProofLocalGivenConditionTypeError::InvalidDependency)
        );
        for mutation in [
            Task269gctInputMutation::ApplicationCount,
            Task269gctInputMutation::ApplicationBinding,
            Task269gctInputMutation::ApplicationOrdinal,
            Task269gctInputMutation::ApplicationRoot,
            Task269gctInputMutation::ExpressionCount,
            Task269gctInputMutation::ExpressionSource,
            Task269gctInputMutation::ExpressionModule,
            Task269gctInputMutation::ExpressionSite,
            Task269gctInputMutation::ExpressionRange,
            Task269gctInputMutation::ExpressionSpelling,
            Task269gctInputMutation::HeadSite,
            Task269gctInputMutation::HeadRange,
            Task269gctInputMutation::HeadSpelling,
            Task269gctInputMutation::Form,
            Task269gctInputMutation::Head,
            Task269gctInputMutation::Recovery,
            Task269gctInputMutation::Argument,
        ] {
            assert_eq!(
                SourceProofLocalGivenConditionTypeProducer::build(
                    fixture.dependency.clone(),
                    mutated_task269gct_input(fixture.source, fixture.module.clone(), mutation),
                    &fixture.symbols,
                    &fixture.arena,
                ),
                Err(SourceProofLocalGivenConditionTypeError::InvalidSourceType),
                "Task269GCT accepted input mutation {mutation:?}",
            );
        }
        let mut wrong_input = fixture.input.clone();
        wrong_input.applications[1].source_ordinal = 2;
        assert_eq!(
            SourceProofLocalGivenConditionTypeProducer::build(
                fixture.dependency.clone(),
                wrong_input,
                &fixture.symbols,
                &fixture.arena,
            ),
            Err(SourceProofLocalGivenConditionTypeError::InvalidSourceType)
        );
        let mut wrong_range = fixture.input.clone();
        wrong_range.expressions[1].source_range.end += 1;
        assert_eq!(
            SourceProofLocalGivenConditionTypeProducer::build(
                fixture.dependency.clone(),
                wrong_range,
                &fixture.symbols,
                &fixture.arena,
            ),
            Err(SourceProofLocalGivenConditionTypeError::InvalidSourceType)
        );
        for arena in [
            task269gct_test_arena(fixture.source, 1, false),
            task269gct_test_arena(fixture.source, 2, true),
        ] {
            assert_eq!(
                SourceProofLocalGivenConditionTypeProducer::build(
                    fixture.dependency.clone(),
                    fixture.input.clone(),
                    &fixture.symbols,
                    &arena,
                ),
                Err(SourceProofLocalGivenConditionTypeError::InvalidSourceType)
            );
        }
        for node in 0..3 {
            for mutation in [
                Task269gctArenaMutation::Kind,
                Task269gctArenaMutation::ResolvedNode,
                Task269gctArenaMutation::Anchor,
                Task269gctArenaMutation::Children,
                Task269gctArenaMutation::Typing,
                Task269gctArenaMutation::Recovery,
                Task269gctArenaMutation::Links,
            ] {
                assert_eq!(
                    SourceProofLocalGivenConditionTypeProducer::build(
                        fixture.dependency.clone(),
                        fixture.input.clone(),
                        &fixture.symbols,
                        &mutated_task269gct_arena(fixture.source, node, mutation),
                    ),
                    Err(SourceProofLocalGivenConditionTypeError::InvalidSourceType),
                    "Task269GCT accepted arena node {node} mutation {mutation:?}",
                );
            }
        }
        let handoff = SourceProofLocalGivenConditionTypeProducer::build(
            fixture.dependency.clone(),
            fixture.input.clone(),
            &fixture.symbols,
            &fixture.arena,
        )
        .expect("Task269GCT valid handoff");
        let mut stale_dependency_fingerprint = handoff.clone();
        stale_dependency_fingerprint
            .dependency_fingerprint
            .push_str("corrupt");
        assert_eq!(
            stale_dependency_fingerprint.validate_installation(
                fixture.source,
                &fixture.module,
                &fixture.arena,
            ),
            Err(SourceProofLocalGivenConditionTypeError::InvalidDependency)
        );
        let mut wrong_handoff_source = handoff.clone();
        wrong_handoff_source.source_id = other_source_id();
        assert_eq!(
            wrong_handoff_source.validate_installation(
                fixture.source,
                &fixture.module,
                &fixture.arena,
            ),
            Err(SourceProofLocalGivenConditionTypeError::InvalidDependency)
        );
        let mut wrong_handoff_module = handoff.clone();
        wrong_handoff_module.module_id = module("task269gct.handoff.wrong");
        assert_eq!(
            wrong_handoff_module.validate_installation(
                fixture.source,
                &fixture.module,
                &fixture.arena,
            ),
            Err(SourceProofLocalGivenConditionTypeError::InvalidDependency)
        );
        let mut dependency_corrupt = handoff.clone();
        dependency_corrupt
            .dependency
            .set_lower_fingerprint_for_task269gc_test("corrupt");
        dependency_corrupt.dependency_fingerprint = dependency_corrupt.dependency.debug_text();
        dependency_corrupt.binding_fingerprint.push_str("corrupt");
        dependency_corrupt
            .source_type_fingerprint
            .push_str("corrupt");
        assert_eq!(
            dependency_corrupt.validate_complete_installation(
                fixture.source,
                &fixture.module,
                &fixture.arena,
                false,
            ),
            Err(SourceProofLocalGivenConditionTypeError::InvalidDependency)
        );
        let mut base_fingerprint_corrupt = handoff.clone();
        base_fingerprint_corrupt
            .dependency
            .set_base_binding_fingerprint_for_task269gc_test("corrupt");
        base_fingerprint_corrupt.dependency_fingerprint =
            base_fingerprint_corrupt.dependency.debug_text();
        assert_eq!(
            base_fingerprint_corrupt.validate_installation(
                fixture.source,
                &fixture.module,
                &fixture.arena,
            ),
            Err(SourceProofLocalGivenConditionTypeError::InvalidDependency)
        );
        let mut aggregate_corrupt = handoff.clone();
        aggregate_corrupt
            .dependency
            .truncate_task269gc_bindings_for_test();
        assert_eq!(
            aggregate_corrupt.validate_installation(
                fixture.source,
                &fixture.module,
                &fixture.arena,
            ),
            Err(SourceProofLocalGivenConditionTypeError::InvalidDependency)
        );
        let mut declaration_corrupt = handoff.clone();
        declaration_corrupt
            .dependency
            .corrupt_task269gc_binding_row_for_test();
        declaration_corrupt.dependency_fingerprint = declaration_corrupt.dependency.debug_text();
        assert_eq!(
            declaration_corrupt.validate_installation(
                fixture.source,
                &fixture.module,
                &fixture.arena,
            ),
            Err(SourceProofLocalGivenConditionTypeError::InvalidDependency)
        );
        let mut final_binding_corrupt = handoff.clone();
        final_binding_corrupt
            .dependency
            .set_final_binding_fingerprint_for_task269gc_test("corrupt");
        final_binding_corrupt.dependency_fingerprint =
            final_binding_corrupt.dependency.debug_text();
        assert_eq!(
            final_binding_corrupt.validate_installation(
                fixture.source,
                &fixture.module,
                &fixture.arena,
            ),
            Err(SourceProofLocalGivenConditionTypeError::InvalidDependency)
        );
        let mut stale_binding_fingerprint = handoff.clone();
        stale_binding_fingerprint
            .binding_fingerprint
            .push_str("corrupt");
        assert_eq!(
            stale_binding_fingerprint.validate_installation(
                fixture.source,
                &fixture.module,
                &fixture.arena,
            ),
            Err(SourceProofLocalGivenConditionTypeError::InvalidBindingEnvironment)
        );
        for binding in 0..2 {
            let mut wrong_binding_type = handoff.clone();
            wrong_binding_type
                .binding_env
                .binding_mut_for_test(BindingId::new(binding))
                .expect("Task269GCT mutable binding")
                .type_site = BindingTypeSite::Missing;
            wrong_binding_type.binding_fingerprint = wrong_binding_type.binding_env.debug_text();
            assert_eq!(
                wrong_binding_type.validate_installation(
                    fixture.source,
                    &fixture.module,
                    &fixture.arena,
                ),
                Err(SourceProofLocalGivenConditionTypeError::InvalidBindingEnvironment),
                "Task269GCT accepted wrong type site on binding {binding}",
            );

            let mut wrong_binding_field = handoff.clone();
            wrong_binding_field
                .binding_env
                .binding_mut_for_test(BindingId::new(binding))
                .expect("Task269GCT mutable binding")
                .spelling
                .push_str("-wrong");
            wrong_binding_field.binding_fingerprint = wrong_binding_field.binding_env.debug_text();
            assert_eq!(
                wrong_binding_field.validate_installation(
                    fixture.source,
                    &fixture.module,
                    &fixture.arena,
                ),
                Err(SourceProofLocalGivenConditionTypeError::InvalidBindingEnvironment),
                "Task269GCT accepted non-type mutation on binding {binding}",
            );
        }
        let mut binding_corrupt = handoff.clone();
        binding_corrupt
            .binding_env
            .binding_mut_for_test(BindingId::new(1))
            .expect("Task269GCT mutable witness")
            .type_site = BindingTypeSite::Missing;
        binding_corrupt.binding_fingerprint = binding_corrupt.binding_env.debug_text();
        binding_corrupt.source_type.expressions.entries[1]
            .source_range
            .end += 1;
        binding_corrupt.source_type_fingerprint = binding_corrupt.source_type.debug_text();
        assert_eq!(
            binding_corrupt.validate_complete_installation(
                fixture.source,
                &fixture.module,
                &fixture.arena,
                false,
            ),
            Err(SourceProofLocalGivenConditionTypeError::InvalidBindingEnvironment)
        );
        let mut stale_source_type_fingerprint = handoff.clone();
        stale_source_type_fingerprint
            .source_type_fingerprint
            .push_str("corrupt");
        assert_eq!(
            stale_source_type_fingerprint.validate_installation(
                fixture.source,
                &fixture.module,
                &fixture.arena,
            ),
            Err(SourceProofLocalGivenConditionTypeError::InvalidSourceType)
        );
        let mut source_corrupt = handoff.clone();
        source_corrupt.source_type.expressions.entries[1]
            .source_range
            .end += 1;
        source_corrupt.source_type_fingerprint = source_corrupt.source_type.debug_text();
        assert_eq!(
            source_corrupt.validate_complete_installation(
                fixture.source,
                &fixture.module,
                &fixture.arena,
                false,
            ),
            Err(SourceProofLocalGivenConditionTypeError::InvalidSourceType)
        );
        assert_eq!(
            handoff.validate_complete_installation(
                fixture.source,
                &fixture.module,
                &fixture.arena,
                false,
            ),
            Err(SourceProofLocalGivenConditionTypeError::InvalidInstallation)
        );
        assert_eq!(
            SourceProofLocalGivenConditionTypeError::InvalidDependency.to_string(),
            "source proof-local given-condition type dependency is invalid"
        );
        assert_eq!(
            SourceProofLocalGivenConditionTypeError::InvalidBindingEnvironment.to_string(),
            "source proof-local given-condition typed binding environment is invalid"
        );
        assert_eq!(
            SourceProofLocalGivenConditionTypeError::InvalidSourceType.to_string(),
            "source proof-local given-condition source type is invalid"
        );
        assert_eq!(
            SourceProofLocalGivenConditionTypeError::InvalidInstallation.to_string(),
            "source proof-local given-condition type installation is invalid"
        );
    }

    #[test]
    fn task269gct_typed_and_resolved_ownership_is_atomic() {
        let fixture = task269gct_fixture();
        let handoff = SourceProofLocalGivenConditionTypeProducer::build(
            fixture.dependency.clone(),
            fixture.input,
            &fixture.symbols,
            &fixture.arena,
        )
        .expect("Task269GCT handoff");
        let typed = task269ct_empty_typed(
            fixture.source,
            fixture.module.clone(),
            fixture.arena.clone(),
        )
        .with_source_proof_local_given_condition_type(handoff.clone())
        .expect("Task269GCT typed installation");
        assert_eq!(
            typed.source_proof_local_given_condition_type(),
            Some(&handoff)
        );
        assert!(typed.source_proof_local_given_condition_binding().is_none());
        assert_eq!(
            typed
                .clone()
                .with_source_proof_local_given_condition_type(handoff.clone()),
            Err(TypedAstError::InvalidSourceProofLocalGivenConditionType)
        );
        macro_rules! assert_task269gct_sibling_both_orders {
            (
                $sibling:expr,
                $inject:ident,
                $install:ident,
                $sibling_error:expr,
                $source:expr,
                $module:expr,
                $arena:expr
            ) => {{
                let sibling = $sibling;
                let mut sibling_first = task269ct_empty_typed(
                    fixture.source,
                    fixture.module.clone(),
                    fixture.arena.clone(),
                );
                sibling_first.$inject(sibling.clone());
                let sibling_first_before = sibling_first.debug_text();
                assert_eq!(
                    sibling_first
                        .clone()
                        .with_source_proof_local_given_condition_type(handoff.clone()),
                    Err(TypedAstError::InvalidSourceProofLocalGivenConditionType),
                );
                assert_eq!(sibling_first.debug_text(), sibling_first_before);
                assert!(
                    sibling_first
                        .source_proof_local_given_condition_type()
                        .is_none()
                );

                let mut gct_first = task269ct_empty_typed($source, $module, $arena);
                gct_first.inject_source_proof_local_given_condition_type_for_test(handoff.clone());
                let gct_first_before = gct_first.debug_text();
                assert_eq!(gct_first.clone().$install(sibling), Err($sibling_error),);
                assert_eq!(gct_first.debug_text(), gct_first_before);
                assert!(
                    gct_first
                        .source_proof_local_given_condition_type()
                        .is_some()
                );
            }};
        }

        let let_fixture = task269ct_fixture();
        let let_binding = let_fixture.dependency.clone();
        let let_type = SourceProofLocalLetTypeProducer::build(
            let_fixture.dependency,
            let_fixture.input,
            &let_fixture.symbols,
            &let_fixture.arena,
        )
        .expect("Task269GCT let-type sibling");
        assert_task269gct_sibling_both_orders!(
            let_binding,
            inject_source_proof_local_let_binding_for_test,
            with_source_proof_local_let_binding,
            TypedAstError::InvalidSourceProofLocalLetBinding,
            let_fixture.source,
            let_fixture.module.clone(),
            TypedArena::try_new(None, Vec::new()).expect("Task269C binding arena")
        );
        assert_task269gct_sibling_both_orders!(
            let_type,
            inject_source_proof_local_let_type_for_test,
            with_source_proof_local_let_type,
            TypedAstError::InvalidSourceProofLocalLetType,
            let_fixture.source,
            let_fixture.module,
            let_fixture.arena
        );

        let given_fixture = task269gt_fixture();
        let given_binding = given_fixture.dependency.clone();
        let given_type = SourceProofLocalGivenTypeProducer::build(
            given_fixture.dependency,
            given_fixture.input,
            &given_fixture.symbols,
            &given_fixture.arena,
        )
        .expect("Task269GCT given-type sibling");
        assert_task269gct_sibling_both_orders!(
            given_binding,
            inject_source_proof_local_given_binding_for_test,
            with_source_proof_local_given_binding,
            TypedAstError::InvalidSourceProofLocalGivenBinding,
            given_fixture.source,
            given_fixture.module.clone(),
            TypedArena::try_new(None, Vec::new()).expect("Task269G binding arena")
        );
        assert_task269gct_sibling_both_orders!(
            given_type,
            inject_source_proof_local_given_type_for_test,
            with_source_proof_local_given_type,
            TypedAstError::InvalidSourceProofLocalGivenType,
            given_fixture.source,
            given_fixture.module,
            given_fixture.arena
        );

        let given_use_fixture = task269gupt_fixture();
        let given_use_type = SourceProofLocalGivenUseTypeProducer::build(
            given_use_fixture.dependency,
            given_use_fixture.input,
            &given_use_fixture.symbols,
            &given_use_fixture.arena,
        )
        .expect("Task269GCT given-use-type sibling");
        let given_use_term_arena = task269gct_given_use_term_arena(given_use_fixture.source);
        let given_use_term = SourceProofLocalGivenUseTermProducer::build(
            given_use_type.clone(),
            task269gct_given_use_term_input(
                given_use_fixture.source,
                given_use_fixture.module.clone(),
            ),
            &given_use_term_arena,
        )
        .expect("Task269GCT given-use-term sibling");
        let direct_source_term = given_use_term.source_term().clone();
        let source_term_first = task269ct_empty_typed(
            given_use_fixture.source,
            given_use_fixture.module.clone(),
            given_use_term_arena.clone(),
        )
        .with_source_term(direct_source_term.clone())
        .expect("Task269GCT direct source-term neighbor");
        let source_term_first_before = source_term_first.debug_text();
        assert_eq!(
            source_term_first
                .clone()
                .with_source_proof_local_given_condition_type(handoff.clone()),
            Err(TypedAstError::InvalidSourceProofLocalGivenConditionType),
        );
        assert_eq!(source_term_first.debug_text(), source_term_first_before);
        let gct_first_before = typed.debug_text();
        assert_eq!(
            typed.clone().with_source_term(direct_source_term),
            Err(TypedAstError::InvalidSourceTerm),
        );
        assert_eq!(typed.debug_text(), gct_first_before);
        assert_task269gct_sibling_both_orders!(
            given_use_type,
            inject_source_proof_local_given_use_type_for_test,
            with_source_proof_local_given_use_type,
            TypedAstError::InvalidSourceProofLocalGivenUseType,
            given_use_fixture.source,
            given_use_fixture.module.clone(),
            given_use_fixture.arena
        );
        assert_task269gct_sibling_both_orders!(
            given_use_term,
            inject_source_proof_local_given_use_term_for_test,
            with_source_proof_local_given_use_term,
            TypedAstError::InvalidSourceProofLocalGivenUseTerm,
            given_use_fixture.source,
            given_use_fixture.module,
            given_use_term_arena
        );
        assert_task269gct_sibling_both_orders!(
            fixture.dependency.clone(),
            inject_source_proof_local_given_condition_binding_for_test,
            with_source_proof_local_given_condition_binding,
            TypedAstError::InvalidSourceProofLocalGivenConditionBinding,
            fixture.source,
            fixture.module.clone(),
            TypedArena::try_new(None, Vec::new()).expect("Task269GC binding arena")
        );
        assert_eq!(
            typed
                .clone()
                .with_source_proof_local_given_condition_binding(fixture.dependency.clone()),
            Err(TypedAstError::InvalidSourceProofLocalGivenConditionBinding)
        );
        let mut occupied_dependency = task269ct_empty_typed(
            fixture.source,
            fixture.module.clone(),
            fixture.arena.clone(),
        );
        occupied_dependency
            .inject_source_proof_local_given_condition_binding_for_test(fixture.dependency.clone());
        assert_eq!(
            occupied_dependency.with_source_proof_local_given_condition_type(handoff.clone()),
            Err(TypedAstError::InvalidSourceProofLocalGivenConditionType)
        );
        let mut reverse_occupied = task269ct_empty_typed(
            fixture.source,
            fixture.module.clone(),
            fixture.arena.clone(),
        );
        reverse_occupied
            .inject_source_proof_local_given_condition_binding_for_test(fixture.dependency.clone());
        reverse_occupied.inject_source_proof_local_given_condition_type_for_test(handoff.clone());
        assert_eq!(
            assemble_task269ct_resolved(&reverse_occupied, Vec::new(), Vec::new()),
            Err(ResolvedTypedAstError::InvalidSourceProofLocalGivenConditionBinding)
        );
        let resolved = assemble_empty_resolved(&typed);
        assert_eq!(
            resolved.source_proof_local_given_condition_type(),
            Some(&handoff)
        );
        assert!(
            resolved
                .source_proof_local_given_condition_binding()
                .is_none()
        );
        assert_eq!(resolved.nodes().len(), 3);
        for (_, node) in resolved.nodes().iter() {
            assert!(matches!(
                &node.kind,
                crate::resolved_typed_ast::ResolvedTypedNodeKind::SourcePreserved { role }
                    if role.as_str() == "source.proof-local.given-condition.type"
            ));
        }
        let statement_hints = (0..3)
            .map(|index| ResolvedNodeKindHint {
                typed_node: TypedNodeId::new(index),
                kind: ResolvedNodeKindHintKind::SourcePreserved {
                    role: SourceNodeRole::new("source.statement.transport"),
                },
            })
            .collect();
        assert_eq!(
            assemble_task269ct_resolved(&typed, Vec::new(), statement_hints),
            Err(ResolvedTypedAstError::InvalidSourceProofLocalGivenConditionType)
        );
        assert_eq!(
            assemble_task269ct_resolved(
                &typed,
                vec![ExpressionMetadataInput {
                    expr: ExprId::new("task269gct.semantic-input"),
                    typed_site: role(0, "source.type.expression"),
                    local_context: None,
                    cluster_facts: Vec::new(),
                }],
                Vec::new(),
            ),
            Err(ResolvedTypedAstError::InvalidSourceProofLocalGivenConditionType)
        );
    }

    #[test]
    fn task269gct_generic_neighbor_and_condition_use_routes_remain_isolated() {
        let fixture = task269gct_fixture();
        let handoff = SourceProofLocalGivenConditionTypeProducer::build(
            fixture.dependency,
            fixture.input.clone(),
            &fixture.symbols,
            &fixture.arena,
        )
        .expect("Task269GCT handoff");
        assert_eq!(
            SourceTypeProducer::build(
                fixture.input,
                handoff.binding_env(),
                &fixture.symbols,
                &fixture.arena,
            ),
            Err(SourceTypeError::InvalidBinding {
                application: SourceTypeApplicationId::new(1),
            })
        );
        let typed = task269ct_empty_typed(fixture.source, fixture.module, fixture.arena)
            .with_source_proof_local_given_condition_type(handoff)
            .expect("Task269GCT typed installation");
        assert!(typed.source_proof_local_let_binding().is_none());
        assert!(typed.source_proof_local_let_type().is_none());
        assert!(typed.source_proof_local_given_binding().is_none());
        assert!(typed.source_proof_local_given_type().is_none());
        assert!(typed.source_proof_local_given_use_type().is_none());
        assert!(typed.source_proof_local_given_use_term().is_none());
        assert!(typed.contexts().is_empty());
        assert!(typed.types().is_empty());
        assert!(typed.facts().is_empty());
        assert!(typed.coercions().is_empty());
        assert!(typed.initial_obligations().is_empty());
        assert!(typed.diagnostics().is_empty());
        let resolved = assemble_empty_resolved(&typed);
        assert!(resolved.source_context().is_none());
        assert!(resolved.source_type().is_none());
        assert!(resolved.source_attribute().is_none());
        assert!(resolved.source_evidence().is_none());
        assert!(resolved.source_term().is_none());
        assert!(resolved.source_application().is_none());
        assert!(resolved.source_structure().is_none());
        assert!(resolved.source_set_term().is_none());
        assert!(resolved.source_atomic_formula().is_none());
        assert!(resolved.source_statement().is_none());
        assert!(resolved.source_statement_references().is_none());
        assert!(resolved.source_statement_witnesses().is_none());
        assert!(resolved.source_proof_local_declaration().is_none());
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
        assert!(!resolved.debug_text().contains("initial-obligation#"));
    }

    #[test]
    fn task269gupt_exact_transaction_fingerprints_and_overlay_are_stable() {
        let fixture = task269gupt_fixture();
        let handoff = SourceProofLocalGivenUseTypeProducer::build(
            fixture.dependency.clone(),
            fixture.input,
            &fixture.symbols,
            &fixture.arena,
        )
        .expect("Task269GUPT exact checker transaction");
        assert_eq!(handoff.source_id(), fixture.source);
        assert_eq!(handoff.module_id(), &fixture.module);
        assert_eq!(handoff.dependency(), &fixture.dependency);
        assert_eq!(
            handoff.dependency_fingerprint(),
            fixture.dependency.debug_text()
        );
        assert_eq!(
            handoff.binding_fingerprint(),
            handoff.binding_env().debug_text()
        );
        assert_eq!(
            handoff.source_type_fingerprint(),
            handoff.source_type().debug_text()
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
            handoff
                .dependency()
                .binding_env()
                .bindings()
                .get(BindingId::new(0))
                .expect("Task269GUPT dependency reserve")
                .type_site,
            BindingTypeSite::Source(range(fixture.source, 14, 17))
        );
        assert_eq!(
            handoff
                .dependency()
                .binding_env()
                .bindings()
                .get(BindingId::new(1))
                .expect("Task269GUPT dependency local")
                .type_site,
            BindingTypeSite::Missing
        );
        let mut expected_local = handoff
            .dependency()
            .binding_env()
            .bindings()
            .get(BindingId::new(1))
            .expect("Task269GUPT dependency local binding")
            .clone();
        expected_local.type_site = BindingTypeSite::Source(range(fixture.source, 84, 87));
        assert_eq!(
            handoff.binding_env().bindings().get(BindingId::new(1)),
            Some(&expected_local)
        );
        assert_eq!(
            (
                handoff.source_type().applications().len(),
                handoff.source_type().expressions().len(),
                handoff.source_type().arguments().len(),
                handoff.source_type().definition_returns().len(),
                handoff.source_type().mode_rhs().len(),
                handoff.source_type().structure_members().len(),
            ),
            (2, 2, 0, 0, 0, 0)
        );
        for (index, (start, end)) in [(14, 17), (84, 87)].into_iter().enumerate() {
            let application = handoff
                .source_type()
                .applications()
                .get(SourceTypeApplicationId::new(index))
                .expect("Task269GUPT application");
            assert_eq!(
                (
                    application.id(),
                    application.binding(),
                    application.source_ordinal(),
                    application.root(),
                ),
                (
                    SourceTypeApplicationId::new(index),
                    BindingId::new(index),
                    index,
                    SourceTypeExpressionId::new(index),
                )
            );
            let expression = handoff
                .source_type()
                .expressions()
                .get(SourceTypeExpressionId::new(index))
                .expect("Task269GUPT expression");
            assert_eq!(expression.source_id(), fixture.source);
            assert_eq!(expression.module_id(), &fixture.module);
            assert!(task269ct_role_matches(
                expression.site(),
                TypedNodeId::new(index),
                "source.type.expression",
            ));
            assert_eq!(expression.source_range(), range(fixture.source, start, end));
            assert_eq!(expression.spelling(), "set");
            assert!(task269ct_role_matches(
                expression.head_site(),
                TypedNodeId::new(index),
                "source.type.head",
            ));
            assert_eq!(expression.head_range(), range(fixture.source, start, end));
            assert_eq!(expression.head_spelling(), "set");
            assert_eq!(expression.form(), SourceTypeApplicationForm::Bare);
            assert_eq!(expression.head(), &SourceTypeHead::BuiltinSet);
            assert_eq!(expression.recovery(), NodeRecoveryState::Normal);
        }
        for (index, (kind, start, end, children)) in [
            (
                "source.proof-local.given-use.reserve-type",
                14,
                17,
                Vec::new(),
            ),
            ("source.proof-local.given-use.type", 84, 87, Vec::new()),
            (
                "source.proof-local.given-use.type-root",
                0,
                127,
                vec![TypedNodeId::new(0), TypedNodeId::new(1)],
            ),
        ]
        .into_iter()
        .enumerate()
        {
            assert_eq!(
                fixture.arena.node(TypedNodeId::new(index)),
                Some(
                    &TypedNode::new(kind, SourceAnchor::Range(range(fixture.source, start, end)))
                        .with_children(children)
                )
            );
        }
        assert_eq!(
            handoff.debug_text(),
            format!(
                concat!(
                    "source-proof-local-given-use-type-debug-v1\n",
                    "module: pkg::task269gup\n",
                    "dependency-fingerprint: {:?}\n",
                    "binding-fingerprint: {:?}\n",
                    "source-type-fingerprint: {:?}\n",
                ),
                handoff.dependency_fingerprint(),
                handoff.binding_fingerprint(),
                handoff.source_type_fingerprint(),
            )
        );
        let replay = SourceProofLocalGivenUseTypeProducer::build(
            fixture.dependency,
            task269gupt_input(fixture.source, fixture.module),
            &fixture.symbols,
            &fixture.arena,
        )
        .expect("Task269GUPT replay");
        assert_eq!(replay, handoff);
    }

    #[test]
    fn task269gupt_dependency_binding_source_type_and_precedence_fail_closed() {
        let fixture = task269gupt_fixture();
        let mut wrong_dependency = fixture.input.clone();
        wrong_dependency.module_id = module("task269gupt.wrong");
        for expression in &mut wrong_dependency.expressions {
            expression.module_id = wrong_dependency.module_id.clone();
        }
        assert_eq!(
            SourceProofLocalGivenUseTypeProducer::build(
                fixture.dependency.clone(),
                wrong_dependency,
                &fixture.symbols,
                &fixture.arena,
            ),
            Err(SourceProofLocalGivenUseTypeError::InvalidDependency)
        );
        let mut wrong_source = fixture.input.clone();
        wrong_source.source_id = other_source_id();
        for expression in &mut wrong_source.expressions {
            expression.source_id = wrong_source.source_id;
            expression.source_range.source_id = wrong_source.source_id;
            expression.head_range.source_id = wrong_source.source_id;
        }
        assert_eq!(
            SourceProofLocalGivenUseTypeProducer::build(
                fixture.dependency.clone(),
                wrong_source,
                &fixture.symbols,
                &fixture.arena,
            ),
            Err(SourceProofLocalGivenUseTypeError::InvalidDependency)
        );
        for mutation in [
            Task269gtInputMutation::ApplicationCount,
            Task269gtInputMutation::ApplicationBinding,
            Task269gtInputMutation::ApplicationOrdinal,
            Task269gtInputMutation::ApplicationRoot,
            Task269gtInputMutation::ExpressionCount,
            Task269gtInputMutation::ExpressionSource,
            Task269gtInputMutation::ExpressionModule,
            Task269gtInputMutation::ExpressionSite,
            Task269gtInputMutation::ExpressionRange,
            Task269gtInputMutation::ExpressionSpelling,
            Task269gtInputMutation::HeadSite,
            Task269gtInputMutation::HeadRange,
            Task269gtInputMutation::HeadSpelling,
            Task269gtInputMutation::Form,
            Task269gtInputMutation::Head,
            Task269gtInputMutation::Recovery,
            Task269gtInputMutation::Argument,
        ] {
            assert_eq!(
                SourceProofLocalGivenUseTypeProducer::build(
                    fixture.dependency.clone(),
                    mutated_task269gupt_input(fixture.source, fixture.module.clone(), mutation),
                    &fixture.symbols,
                    &fixture.arena,
                ),
                Err(SourceProofLocalGivenUseTypeError::InvalidSourceType),
                "Task269GUPT input mutation {mutation:?}",
            );
        }
        let wrong_symbols = SymbolEnv::new(
            module("task269gupt.symbols.wrong"),
            SymbolEnvIndexes::default(),
        );
        assert_eq!(
            SourceProofLocalGivenUseTypeProducer::build(
                fixture.dependency.clone(),
                fixture.input.clone(),
                &wrong_symbols,
                &fixture.arena,
            ),
            Err(SourceProofLocalGivenUseTypeError::InvalidSourceType)
        );
        for node in 0..3 {
            for mutation in [
                Task269gtArenaMutation::Kind,
                Task269gtArenaMutation::ResolvedNode,
                Task269gtArenaMutation::Anchor,
                Task269gtArenaMutation::Children,
                Task269gtArenaMutation::Typing,
                Task269gtArenaMutation::Recovery,
                Task269gtArenaMutation::Links,
            ] {
                assert_eq!(
                    SourceProofLocalGivenUseTypeProducer::build(
                        fixture.dependency.clone(),
                        fixture.input.clone(),
                        &fixture.symbols,
                        &mutated_task269gupt_arena(fixture.source, node, mutation),
                    ),
                    Err(SourceProofLocalGivenUseTypeError::InvalidSourceType),
                    "Task269GUPT arena node {node} mutation {mutation:?}",
                );
            }
        }
        let handoff = SourceProofLocalGivenUseTypeProducer::build(
            fixture.dependency.clone(),
            fixture.input.clone(),
            &fixture.symbols,
            &fixture.arena,
        )
        .expect("Task269GUPT valid handoff");
        let mut dependency_lower_corrupt = handoff.clone();
        dependency_lower_corrupt
            .dependency
            .set_lower_fingerprint_for_task269gup_test("corrupt");
        dependency_lower_corrupt.dependency_fingerprint =
            dependency_lower_corrupt.dependency.debug_text();
        assert_eq!(
            dependency_lower_corrupt.validate_installation(
                fixture.source,
                &fixture.module,
                &fixture.arena,
            ),
            Err(SourceProofLocalGivenUseTypeError::InvalidDependency)
        );
        let mut dependency_base_corrupt = handoff.clone();
        dependency_base_corrupt
            .dependency
            .set_base_binding_fingerprint_for_task269gup_test("corrupt");
        dependency_base_corrupt.dependency_fingerprint =
            dependency_base_corrupt.dependency.debug_text();
        assert_eq!(
            dependency_base_corrupt.validate_installation(
                fixture.source,
                &fixture.module,
                &fixture.arena,
            ),
            Err(SourceProofLocalGivenUseTypeError::InvalidDependency)
        );
        let mut dependency_aggregate_corrupt = handoff.clone();
        dependency_aggregate_corrupt
            .dependency
            .truncate_task269gup_bindings_for_test();
        assert_eq!(
            dependency_aggregate_corrupt.validate_installation(
                fixture.source,
                &fixture.module,
                &fixture.arena,
            ),
            Err(SourceProofLocalGivenUseTypeError::InvalidDependency)
        );
        let mut dependency_row_corrupt = handoff.clone();
        dependency_row_corrupt
            .dependency
            .corrupt_task269gup_binding_row_for_test();
        dependency_row_corrupt.dependency_fingerprint =
            dependency_row_corrupt.dependency.debug_text();
        assert_eq!(
            dependency_row_corrupt.validate_installation(
                fixture.source,
                &fixture.module,
                &fixture.arena,
            ),
            Err(SourceProofLocalGivenUseTypeError::InvalidDependency)
        );
        let mut dependency_final_corrupt = handoff.clone();
        dependency_final_corrupt
            .dependency
            .set_final_binding_fingerprint_for_task269gup_test("corrupt");
        dependency_final_corrupt.dependency_fingerprint =
            dependency_final_corrupt.dependency.debug_text();
        assert_eq!(
            dependency_final_corrupt.validate_installation(
                fixture.source,
                &fixture.module,
                &fixture.arena,
            ),
            Err(SourceProofLocalGivenUseTypeError::InvalidDependency)
        );
        let mut dependency_corrupt = handoff.clone();
        dependency_corrupt
            .dependency_fingerprint
            .push_str("corrupt");
        dependency_corrupt.binding_fingerprint.push_str("corrupt");
        dependency_corrupt
            .source_type_fingerprint
            .push_str("corrupt");
        assert_eq!(
            dependency_corrupt.validate_complete_installation(
                fixture.source,
                &fixture.module,
                &fixture.arena,
                false,
            ),
            Err(SourceProofLocalGivenUseTypeError::InvalidDependency)
        );
        let mut binding_corrupt = handoff.clone();
        binding_corrupt
            .binding_env
            .binding_mut_for_test(BindingId::new(1))
            .expect("Task269GUPT mutable local")
            .type_site = BindingTypeSite::Missing;
        binding_corrupt.binding_fingerprint = binding_corrupt.binding_env.debug_text();
        binding_corrupt.source_type.expressions.entries[1]
            .source_range
            .end += 1;
        binding_corrupt.source_type_fingerprint = binding_corrupt.source_type.debug_text();
        assert_eq!(
            binding_corrupt.validate_complete_installation(
                fixture.source,
                &fixture.module,
                &fixture.arena,
                false,
            ),
            Err(SourceProofLocalGivenUseTypeError::InvalidBindingEnvironment)
        );
        let mut binding_fingerprint_corrupt = handoff.clone();
        binding_fingerprint_corrupt
            .binding_fingerprint
            .push_str("corrupt");
        assert_eq!(
            binding_fingerprint_corrupt.validate_installation(
                fixture.source,
                &fixture.module,
                &fixture.arena,
            ),
            Err(SourceProofLocalGivenUseTypeError::InvalidBindingEnvironment)
        );
        let mut source_corrupt = handoff.clone();
        source_corrupt.source_type.expressions.entries[1]
            .source_range
            .end += 1;
        source_corrupt.source_type_fingerprint = source_corrupt.source_type.debug_text();
        assert_eq!(
            source_corrupt.validate_complete_installation(
                fixture.source,
                &fixture.module,
                &fixture.arena,
                false,
            ),
            Err(SourceProofLocalGivenUseTypeError::InvalidSourceType)
        );
        let mut source_fingerprint_corrupt = handoff.clone();
        source_fingerprint_corrupt
            .source_type_fingerprint
            .push_str("corrupt");
        assert_eq!(
            source_fingerprint_corrupt.validate_installation(
                fixture.source,
                &fixture.module,
                &fixture.arena,
            ),
            Err(SourceProofLocalGivenUseTypeError::InvalidSourceType)
        );
        assert_eq!(
            handoff.validate_installation(
                fixture.source,
                &fixture.module,
                &task269gupt_test_arena(fixture.source, 1, false),
            ),
            Err(SourceProofLocalGivenUseTypeError::InvalidSourceType)
        );
        assert_eq!(
            handoff.validate_complete_installation(
                fixture.source,
                &fixture.module,
                &fixture.arena,
                false,
            ),
            Err(SourceProofLocalGivenUseTypeError::InvalidInstallation)
        );
        for (error, expected) in [
            (
                SourceProofLocalGivenUseTypeError::InvalidDependency,
                "source proof-local given-use type dependency is invalid",
            ),
            (
                SourceProofLocalGivenUseTypeError::InvalidBindingEnvironment,
                "source proof-local given-use typed binding environment is invalid",
            ),
            (
                SourceProofLocalGivenUseTypeError::InvalidSourceType,
                "source proof-local given-use source type is invalid",
            ),
            (
                SourceProofLocalGivenUseTypeError::InvalidInstallation,
                "source proof-local given-use type installation is invalid",
            ),
        ] {
            assert_eq!(error.to_string(), expected);
            let _: &dyn std::error::Error = &error;
        }
    }

    #[test]
    fn task269gupt_typed_and_resolved_ownership_is_atomic() {
        let fixture = task269gupt_fixture();
        let handoff = SourceProofLocalGivenUseTypeProducer::build(
            fixture.dependency.clone(),
            fixture.input,
            &fixture.symbols,
            &fixture.arena,
        )
        .expect("Task269GUPT handoff");
        let typed = task269ct_empty_typed(
            fixture.source,
            fixture.module.clone(),
            fixture.arena.clone(),
        )
        .with_source_proof_local_given_use_type(handoff.clone())
        .expect("Task269GUPT typed installation");
        assert_eq!(typed.source_proof_local_given_use_type(), Some(&handoff));
        assert!(typed.source_proof_local_given_binding().is_none());
        assert!(typed.source_proof_local_given_type().is_none());
        assert!(typed.source_type().is_none());
        assert_eq!(
            typed
                .clone()
                .with_source_proof_local_given_use_type(handoff.clone()),
            Err(TypedAstError::InvalidSourceProofLocalGivenUseType)
        );

        let let_neighbor = task269gupt_let_neighbor_fixture();
        let let_type = SourceProofLocalLetTypeProducer::build(
            let_neighbor.dependency.clone(),
            let_neighbor.input.clone(),
            &let_neighbor.symbols,
            &let_neighbor.arena,
        )
        .expect("Task269CT same-identity type neighbor");
        let given_neighbor = task269gupt_given_neighbor_fixture();
        let given_type = SourceProofLocalGivenTypeProducer::build(
            given_neighbor.dependency.clone(),
            given_neighbor.input.clone(),
            &given_neighbor.symbols,
            &given_neighbor.arena,
        )
        .expect("Task269GT same-identity type neighbor");
        assert_eq!(let_neighbor.source, fixture.source);
        assert_eq!(let_neighbor.module, fixture.module);
        assert_eq!(given_neighbor.source, fixture.source);
        assert_eq!(given_neighbor.module, fixture.module);

        let empty_arena = TypedArena::try_new(None, Vec::new()).expect("empty neighbor arena");
        let mut use_before_let_binding =
            task269ct_empty_typed(fixture.source, fixture.module.clone(), empty_arena.clone());
        use_before_let_binding.inject_source_proof_local_given_use_type_for_test(handoff.clone());
        assert_eq!(
            use_before_let_binding
                .with_source_proof_local_let_binding(let_neighbor.dependency.clone()),
            Err(TypedAstError::InvalidSourceProofLocalLetBinding)
        );
        let mut use_before_given_binding =
            task269ct_empty_typed(fixture.source, fixture.module.clone(), empty_arena);
        use_before_given_binding.inject_source_proof_local_given_use_type_for_test(handoff.clone());
        assert_eq!(
            use_before_given_binding
                .with_source_proof_local_given_binding(given_neighbor.dependency.clone()),
            Err(TypedAstError::InvalidSourceProofLocalGivenBinding)
        );

        let mut use_before_let_type = task269ct_empty_typed(
            fixture.source,
            fixture.module.clone(),
            let_neighbor.arena.clone(),
        );
        use_before_let_type.inject_source_proof_local_given_use_type_for_test(handoff.clone());
        assert_eq!(
            use_before_let_type.with_source_proof_local_let_type(let_type.clone()),
            Err(TypedAstError::InvalidSourceProofLocalLetType)
        );
        let mut use_before_given_type = task269ct_empty_typed(
            fixture.source,
            fixture.module.clone(),
            given_neighbor.arena.clone(),
        );
        use_before_given_type.inject_source_proof_local_given_use_type_for_test(handoff.clone());
        assert_eq!(
            use_before_given_type.with_source_proof_local_given_type(given_type.clone()),
            Err(TypedAstError::InvalidSourceProofLocalGivenType)
        );

        let mut let_binding_before_use = task269ct_empty_typed(
            fixture.source,
            fixture.module.clone(),
            fixture.arena.clone(),
        );
        let_binding_before_use
            .inject_source_proof_local_let_binding_for_test(let_neighbor.dependency.clone());
        assert_eq!(
            let_binding_before_use.with_source_proof_local_given_use_type(handoff.clone()),
            Err(TypedAstError::InvalidSourceProofLocalGivenUseType)
        );
        let mut given_binding_before_use = task269ct_empty_typed(
            fixture.source,
            fixture.module.clone(),
            fixture.arena.clone(),
        );
        given_binding_before_use
            .inject_source_proof_local_given_binding_for_test(given_neighbor.dependency.clone());
        assert_eq!(
            given_binding_before_use.with_source_proof_local_given_use_type(handoff.clone()),
            Err(TypedAstError::InvalidSourceProofLocalGivenUseType)
        );
        let mut let_type_before_use = task269ct_empty_typed(
            fixture.source,
            fixture.module.clone(),
            fixture.arena.clone(),
        );
        let_type_before_use.inject_source_proof_local_let_type_for_test(let_type.clone());
        assert_eq!(
            let_type_before_use.with_source_proof_local_given_use_type(handoff.clone()),
            Err(TypedAstError::InvalidSourceProofLocalGivenUseType)
        );
        let mut given_type_before_use = task269ct_empty_typed(
            fixture.source,
            fixture.module.clone(),
            fixture.arena.clone(),
        );
        given_type_before_use.inject_source_proof_local_given_type_for_test(given_type.clone());
        assert_eq!(
            given_type_before_use.with_source_proof_local_given_use_type(handoff.clone()),
            Err(TypedAstError::InvalidSourceProofLocalGivenUseType)
        );

        let mut resolved_let_binding_neighbor = typed.clone();
        resolved_let_binding_neighbor
            .inject_source_proof_local_let_binding_for_test(let_neighbor.dependency);
        assert_eq!(
            assemble_task269ct_resolved(&resolved_let_binding_neighbor, Vec::new(), Vec::new()),
            Err(ResolvedTypedAstError::InvalidSourceProofLocalLetBinding)
        );
        let mut resolved_given_binding_neighbor = typed.clone();
        resolved_given_binding_neighbor
            .inject_source_proof_local_given_binding_for_test(given_neighbor.dependency);
        assert_eq!(
            assemble_task269ct_resolved(&resolved_given_binding_neighbor, Vec::new(), Vec::new()),
            Err(ResolvedTypedAstError::InvalidSourceProofLocalGivenBinding)
        );
        let mut resolved_let_type_neighbor =
            task269ct_empty_typed(fixture.source, fixture.module.clone(), let_neighbor.arena)
                .with_source_proof_local_let_type(let_type)
                .expect("Task269CT valid same-identity resolved neighbor");
        resolved_let_type_neighbor
            .inject_source_proof_local_given_use_type_for_test(handoff.clone());
        assert_eq!(
            assemble_task269ct_resolved(&resolved_let_type_neighbor, Vec::new(), Vec::new()),
            Err(ResolvedTypedAstError::InvalidSourceProofLocalLetType)
        );
        let mut resolved_given_type_neighbor =
            task269ct_empty_typed(fixture.source, fixture.module.clone(), given_neighbor.arena)
                .with_source_proof_local_given_type(given_type)
                .expect("Task269GT valid same-identity resolved neighbor");
        resolved_given_type_neighbor
            .inject_source_proof_local_given_use_type_for_test(handoff.clone());
        assert_eq!(
            assemble_task269ct_resolved(&resolved_given_type_neighbor, Vec::new(), Vec::new()),
            Err(ResolvedTypedAstError::InvalidSourceProofLocalGivenType)
        );

        let resolved = assemble_empty_resolved(&typed);
        assert_eq!(resolved.source_proof_local_given_use_type(), Some(&handoff));
        assert!(resolved.source_proof_local_given_binding().is_none());
        assert!(resolved.source_proof_local_given_type().is_none());
        assert_eq!(resolved.nodes().len(), 3);
        for (_, node) in resolved.nodes().iter() {
            assert!(matches!(
                &node.kind,
                crate::resolved_typed_ast::ResolvedTypedNodeKind::SourcePreserved { role }
                    if role.as_str() == "source.proof-local.given-use.type"
            ));
        }
        let statement_hints = (0..3)
            .map(|index| ResolvedNodeKindHint {
                typed_node: TypedNodeId::new(index),
                kind: ResolvedNodeKindHintKind::SourcePreserved {
                    role: SourceNodeRole::new("source.statement.transport"),
                },
            })
            .collect();
        assert_eq!(
            assemble_task269ct_resolved(&typed, Vec::new(), statement_hints),
            Err(ResolvedTypedAstError::InvalidSourceProofLocalGivenUseType)
        );
        assert_eq!(
            assemble_task269ct_resolved(
                &typed,
                vec![ExpressionMetadataInput {
                    expr: ExprId::new("task269gupt.semantic-input"),
                    typed_site: role(0, "source.type.expression"),
                    local_context: None,
                    cluster_facts: Vec::new(),
                }],
                Vec::new(),
            ),
            Err(ResolvedTypedAstError::InvalidSourceProofLocalGivenUseType)
        );
    }

    #[test]
    fn task269gupt_prior_and_neighbor_routes_remain_isolated() {
        let fixture = task269gupt_fixture();
        let handoff = SourceProofLocalGivenUseTypeProducer::build(
            fixture.dependency,
            fixture.input.clone(),
            &fixture.symbols,
            &fixture.arena,
        )
        .expect("Task269GUPT handoff");
        assert_eq!(
            SourceTypeProducer::build(
                fixture.input,
                handoff.binding_env(),
                &fixture.symbols,
                &fixture.arena,
            ),
            Err(SourceTypeError::InvalidBinding {
                application: SourceTypeApplicationId::new(1),
            })
        );
        let typed = task269ct_empty_typed(fixture.source, fixture.module, fixture.arena)
            .with_source_proof_local_given_use_type(handoff)
            .expect("Task269GUPT typed installation");
        assert!(typed.source_proof_local_let_binding().is_none());
        assert!(typed.source_proof_local_let_type().is_none());
        assert!(typed.source_proof_local_given_binding().is_none());
        assert!(typed.source_proof_local_given_type().is_none());
        assert!(typed.contexts().is_empty());
        assert!(typed.types().is_empty());
        assert!(typed.facts().is_empty());
        assert!(typed.coercions().is_empty());
        assert!(typed.initial_obligations().is_empty());
        assert!(typed.diagnostics().is_empty());
        let resolved = assemble_empty_resolved(&typed);
        assert!(resolved.source_context().is_none());
        assert!(resolved.source_type().is_none());
        assert!(resolved.source_attribute().is_none());
        assert!(resolved.source_evidence().is_none());
        assert!(resolved.source_term().is_none());
        assert!(resolved.source_application().is_none());
        assert!(resolved.source_structure().is_none());
        assert!(resolved.source_set_term().is_none());
        assert!(resolved.source_atomic_formula().is_none());
        assert!(resolved.source_attribute_definition().is_none());
        assert!(resolved.source_functor_definition().is_none());
        assert!(resolved.source_property_implementation().is_none());
        assert!(resolved.source_mode_definition().is_none());
        assert!(resolved.source_structure_definition().is_none());
        assert!(resolved.source_predicate_definition().is_none());
        assert!(resolved.source_composite_formula().is_none());
        assert!(resolved.source_formula_composition().is_none());
        assert!(resolved.source_condition_formula_composition().is_none());
        assert!(resolved.source_predicate_chain_composition().is_none());
        assert!(resolved.source_statement().is_none());
        assert!(resolved.source_statement_references().is_none());
        assert!(resolved.source_statement_witnesses().is_none());
        assert!(resolved.source_proof_local_declaration().is_none());
        assert!(resolved.source_proof_local_let_binding().is_none());
        assert!(resolved.source_proof_local_let_type().is_none());
        assert!(resolved.source_proof_local_given_binding().is_none());
        assert!(resolved.source_proof_local_given_type().is_none());
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
        assert!(!resolved.debug_text().contains("initial-obligation#"));
    }

    #[test]
    fn production_boundary_stays_syntax_free_and_has_no_semantic_result_payloads() {
        let source = include_str!("source_type.rs");
        for forbidden in [
            concat!("mizar", "_syntax"),
            concat!("Surface", "NodeId"),
            concat!("Normalized", "Type"),
            concat!("Declaration", "CheckingOutput"),
            concat!("Accepted", "Fact"),
            concat!("Proof", "Context"),
        ] {
            assert!(
                !source[..source.find("#[cfg(test)]").expect("test module")].contains(forbidden),
                "source type handoff exposes {forbidden}"
            );
        }
    }
}
