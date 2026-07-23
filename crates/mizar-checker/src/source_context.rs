//! Syntax-free source-item and binding-context projection for checker phase 6.

use crate::{
    binding_env::{
        BinderIdentity, BindingContextDraft, BindingContextId, BindingContextLayer,
        BindingContextOwner, BindingContextRecovery, BindingContextTable, BindingDiagnosticClass,
        BindingDiagnosticDraft, BindingDiagnosticId, BindingDiagnosticRecovery,
        BindingDiagnosticSeverity, BindingDiagnosticTable, BindingDraft, BindingEnv,
        BindingEnvParts, BindingId, BindingKind, BindingRecoveryState, BindingStatus, BindingTable,
        BindingTypeSite, CapturedFreeVariables,
    },
    typed_ast::{
        BindingTypeRef, ContextRecoveryState, LocalTypeContextDraft, LocalTypeContextId,
        LocalTypeContextTable, TypeContextLayer, TypedSiteRef,
    },
};
use mizar_resolve::{
    declarations::DeclarationShellId,
    names::{LocalTermBinding, LocalTermScope},
    resolved_ast::ModuleId,
};
use mizar_session::{SourceId, SourceRange};
use std::{
    collections::{BTreeMap, BTreeSet},
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

dense_id!(SourceItemId);
dense_id!(SourceDeclarationId);

/// Complete syntax-free input for one source/binding-context transaction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceBindingContextInput {
    pub source_id: SourceId,
    pub module_id: ModuleId,
    pub module_site: TypedSiteRef,
    pub items: Vec<SourceItemInput>,
    pub bindings: Vec<SourceBindingSiteInput>,
}

/// One resolver-shell projection. Raw syntax identities never cross this seam.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceItemInput {
    pub shell: DeclarationShellId,
    pub shell_ordinal: usize,
    pub role: SourceItemRole,
    pub module_id: ModuleId,
    pub source_range: SourceRange,
    pub parent: Option<DeclarationShellId>,
    pub visibility: SourceItemVisibility,
    pub site: TypedSiteRef,
    pub local_scope: Option<LocalTermScope>,
    pub recovery: SourceItemRecovery,
}

/// Source-item roles admitted by Task 248.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum SourceItemRole {
    Reserve,
    DefinitionBlock,
}

/// Source-shaped visibility. Task 248 admits only `Unspecified`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum SourceItemVisibility {
    Unspecified,
    Public,
    Private,
    Recovered,
}

/// Source-item recovery state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum SourceItemRecovery {
    Normal,
    Recovered,
}

/// The context in which a source binding is introduced.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum SourceBindingContextOwner {
    Module,
    Shell(DeclarationShellId),
}

/// One syntax-free source declaration/binding projection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceBindingSiteInput {
    pub shell: DeclarationShellId,
    pub context_owner: SourceBindingContextOwner,
    pub source_ordinal: usize,
    pub spelling: String,
    pub declaration_range: SourceRange,
    pub written_type_range: SourceRange,
    pub site: TypedSiteRef,
    pub role: SourceBindingSiteRole,
    pub recovery: BindingRecoveryState,
}

/// Binding roles admitted by Task 248.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum SourceBindingSiteRole {
    ReserveDefault,
    DefinitionParameter { local: LocalTermBinding },
}

/// Result of validating and building one source-context input.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum SourceBindingContextBuild {
    Complete(SourceBindingContextProjection),
    Incomplete(SourceBindingContextIncomplete),
}

impl SourceBindingContextBuild {
    pub fn into_complete(self) -> Result<SourceBindingContextProjection, SourceContextError> {
        match self {
            Self::Complete(projection) => Ok(projection),
            Self::Incomplete(_) => Err(SourceContextError::IncompleteRecovery),
        }
    }
}

/// Complete transaction ready for `TypedAst` installation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceBindingContextProjection {
    handoff: SourceBindingContextHandoff,
}

impl SourceBindingContextProjection {
    pub const fn handoff(&self) -> &SourceBindingContextHandoff {
        &self.handoff
    }

    pub const fn local_contexts(&self) -> &LocalTypeContextTable {
        self.handoff.local_contexts()
    }

    pub fn into_handoff(self) -> SourceBindingContextHandoff {
        self.handoff
    }
}

/// Incomplete recovered context. It cannot be installed in a typed AST.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceBindingContextIncomplete {
    binding_env: BindingEnv,
    recovered_shell: DeclarationShellId,
    recovered_context: BindingContextId,
    diagnostic: BindingDiagnosticId,
}

impl SourceBindingContextIncomplete {
    pub const fn binding_env(&self) -> &BindingEnv {
        &self.binding_env
    }

    pub const fn recovered_shell(&self) -> DeclarationShellId {
        self.recovered_shell
    }

    pub const fn recovered_context(&self) -> BindingContextId {
        self.recovered_context
    }

    pub const fn diagnostic(&self) -> BindingDiagnosticId {
        self.diagnostic
    }
}

/// Immutable final source/binding-context table bundle.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceBindingContextHandoff {
    source_id: SourceId,
    module_id: ModuleId,
    binding_env: BindingEnv,
    local_contexts: LocalTypeContextTable,
    items: SourceItemTable,
    declarations: SourceDeclarationTable,
    context_links: SourceContextLinkTable,
}

impl SourceBindingContextHandoff {
    pub const fn source_id(&self) -> SourceId {
        self.source_id
    }

    pub const fn module_id(&self) -> &ModuleId {
        &self.module_id
    }

    pub const fn binding_env(&self) -> &BindingEnv {
        &self.binding_env
    }

    pub const fn local_contexts(&self) -> &LocalTypeContextTable {
        &self.local_contexts
    }

    pub const fn items(&self) -> &SourceItemTable {
        &self.items
    }

    pub const fn declarations(&self) -> &SourceDeclarationTable {
        &self.declarations
    }

    pub const fn context_links(&self) -> &SourceContextLinkTable {
        &self.context_links
    }

    pub fn debug_text(&self) -> String {
        let mut output = String::from("source-binding-context-debug-v1\n");
        output.push_str("module: ");
        output.push_str(self.module_id.path().as_str());
        output.push('\n');
        for (id, item) in self.items.iter() {
            let _ = writeln!(
                output,
                "item#{} shell={} ordinal={} role={} range={}..{} parent={} context={} local_context={} predecessor={}",
                id.index(),
                item.shell.index(),
                item.shell_ordinal,
                item_role_key(item.role),
                item.source_range.start,
                item.source_range.end,
                item.parent
                    .map_or_else(|| "none".to_owned(), |value| value.index().to_string()),
                item.binding_context.index(),
                item.local_context.index(),
                item.predecessor
                    .map_or_else(|| "none".to_owned(), |value| value.index().to_string()),
            );
        }
        for (id, declaration) in self.declarations.iter() {
            let _ = writeln!(
                output,
                "declaration#{} item={} binding={} ordinal={} role={} range={}..{} type_range={}..{} context={} local_context={} shadowed={} predecessor={}",
                id.index(),
                declaration.item.index(),
                declaration.binding.index(),
                declaration.source_ordinal,
                binding_role_key(&declaration.role),
                declaration.declaration_range.start,
                declaration.declaration_range.end,
                declaration.written_type_range.start,
                declaration.written_type_range.end,
                declaration.binding_context.index(),
                declaration.local_context.index(),
                declaration
                    .shadowed_binding
                    .map_or_else(|| "none".to_owned(), |value| value.index().to_string()),
                declaration
                    .predecessor
                    .map_or_else(|| "none".to_owned(), |value| value.index().to_string()),
            );
        }
        for (id, link) in self.context_links.iter() {
            let _ = writeln!(
                output,
                "context-link#{} binding_context={} local_context={} item={}",
                id,
                link.binding_context.index(),
                link.local_context.index(),
                link.item
                    .map_or_else(|| "module".to_owned(), |value| value.index().to_string()),
            );
        }
        output
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SourceItemTable {
    entries: Vec<SourceItem>,
}

impl SourceItemTable {
    pub fn get(&self, id: SourceItemId) -> Option<&SourceItem> {
        self.entries.get(id.index())
    }

    pub fn iter(&self) -> impl Iterator<Item = (SourceItemId, &SourceItem)> {
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
pub struct SourceItem {
    pub id: SourceItemId,
    pub shell: DeclarationShellId,
    pub shell_ordinal: usize,
    pub role: SourceItemRole,
    pub source_range: SourceRange,
    pub parent: Option<DeclarationShellId>,
    pub visibility: SourceItemVisibility,
    pub site: TypedSiteRef,
    pub local_scope: Option<LocalTermScope>,
    pub recovery: SourceItemRecovery,
    pub binding_context: BindingContextId,
    pub local_context: LocalTypeContextId,
    pub predecessor: Option<SourceItemId>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SourceDeclarationTable {
    entries: Vec<SourceDeclaration>,
}

impl SourceDeclarationTable {
    pub fn get(&self, id: SourceDeclarationId) -> Option<&SourceDeclaration> {
        self.entries.get(id.index())
    }

    pub fn iter(&self) -> impl Iterator<Item = (SourceDeclarationId, &SourceDeclaration)> {
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
pub struct SourceDeclaration {
    pub id: SourceDeclarationId,
    pub item: SourceItemId,
    pub binding: BindingId,
    pub source_ordinal: usize,
    pub spelling: String,
    pub declaration_range: SourceRange,
    pub written_type_range: SourceRange,
    pub site: TypedSiteRef,
    pub role: SourceBindingSiteRole,
    pub binding_context: BindingContextId,
    pub local_context: LocalTypeContextId,
    pub shadowed_binding: Option<BindingId>,
    pub predecessor: Option<SourceDeclarationId>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SourceContextLinkTable {
    entries: Vec<SourceContextLink>,
}

impl SourceContextLinkTable {
    pub fn get(&self, binding_context: BindingContextId) -> Option<&SourceContextLink> {
        self.entries.get(binding_context.index())
    }

    pub fn iter(&self) -> impl Iterator<Item = (usize, &SourceContextLink)> {
        self.entries.iter().enumerate()
    }

    pub const fn len(&self) -> usize {
        self.entries.len()
    }

    pub const fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceContextLink {
    pub binding_context: BindingContextId,
    pub local_context: LocalTypeContextId,
    pub item: Option<SourceItemId>,
}

/// Builds and validates Task-248 source/binding-context projections.
pub struct SourceBindingContextProducer;

impl SourceBindingContextProducer {
    pub fn build(
        input: SourceBindingContextInput,
    ) -> Result<SourceBindingContextBuild, SourceContextError> {
        let validated = validate_input(&input)?;
        build_projection(input, validated)
    }
}

#[derive(Debug)]
struct ValidatedInput {
    item_by_shell: BTreeMap<DeclarationShellId, usize>,
    context_ids: Vec<BindingContextId>,
    local_context_ids: Vec<LocalTypeContextId>,
}

fn validate_input(input: &SourceBindingContextInput) -> Result<ValidatedInput, SourceContextError> {
    if input.items.is_empty() {
        return Err(SourceContextError::MissingItems);
    }
    let mut item_by_shell = BTreeMap::new();
    let mut sites = BTreeSet::from([input.module_site.clone()]);
    let mut context_ids = Vec::with_capacity(input.items.len());
    let mut local_context_ids = Vec::with_capacity(input.items.len());
    let mut next_context = 1;
    let mut recovered_item = None;
    let mut previous_start = None;

    for (index, item) in input.items.iter().enumerate() {
        if item.shell_ordinal != index {
            return Err(SourceContextError::StaleShellOrdinal { index });
        }
        if item.module_id != input.module_id {
            return Err(SourceContextError::ModuleMismatch { index });
        }
        if item.source_range.source_id != input.source_id {
            return Err(SourceContextError::ItemSourceMismatch { index });
        }
        if previous_start.is_some_and(|start| item.source_range.start < start) {
            return Err(SourceContextError::ReorderedItems { index });
        }
        previous_start = Some(item.source_range.start);
        if item_by_shell.insert(item.shell, index).is_some() {
            return Err(SourceContextError::DuplicateShell { index });
        }
        if !sites.insert(item.site.clone()) {
            return Err(SourceContextError::DuplicateTypedSite);
        }
        if item.visibility != SourceItemVisibility::Unspecified {
            return Err(SourceContextError::UnsupportedVisibility { index });
        }
        match item.role {
            SourceItemRole::Reserve => {
                if item.local_scope.is_some() || item.parent.is_some() {
                    return Err(SourceContextError::InvalidItemContext { index });
                }
                context_ids.push(BindingContextId::new(0));
                local_context_ids.push(LocalTypeContextId::new(0));
            }
            SourceItemRole::DefinitionBlock => {
                if item.parent.is_some() {
                    return Err(SourceContextError::InvalidParent { index });
                }
                if item.local_scope.is_none() {
                    return Err(SourceContextError::InvalidItemContext { index });
                }
                context_ids.push(BindingContextId::new(next_context));
                local_context_ids.push(LocalTypeContextId::new(next_context));
                next_context += 1;
            }
        }
        if item.recovery == SourceItemRecovery::Recovered
            && (item.role != SourceItemRole::DefinitionBlock
                || recovered_item.replace(index).is_some())
        {
            return Err(SourceContextError::UnsupportedRecovery { index });
        }
    }

    for (index, item) in input.items.iter().enumerate() {
        if let Some(parent) = &item.parent {
            let Some(parent_index) = item_by_shell.get(parent).copied() else {
                return Err(SourceContextError::InvalidParent { index });
            };
            if parent_index >= index
                || !range_contains(input.items[parent_index].source_range, item.source_range)
            {
                return Err(SourceContextError::InvalidParent { index });
            }
        }
    }

    let mut binding_sites = BTreeSet::new();
    let mut binding_names = BTreeSet::new();
    let mut counts = vec![0usize; input.items.len()];
    let mut previous_item = 0;
    let mut previous_start = None;
    for (index, binding) in input.bindings.iter().enumerate() {
        if binding.source_ordinal != index {
            return Err(SourceContextError::StaleBindingOrdinal { index });
        }
        if binding.recovery != BindingRecoveryState::Normal {
            return Err(SourceContextError::RecoveredBinding { index });
        }
        if binding.spelling.is_empty() {
            return Err(SourceContextError::EmptyBindingSpelling { index });
        }
        if binding.declaration_range.source_id != input.source_id
            || binding.written_type_range.source_id != input.source_id
        {
            return Err(SourceContextError::BindingSourceMismatch { index });
        }
        if previous_start.is_some_and(|start| binding.declaration_range.start < start) {
            return Err(SourceContextError::ReorderedBindings { index });
        }
        previous_start = Some(binding.declaration_range.start);
        if !binding_sites.insert(binding.site.clone()) || !sites.insert(binding.site.clone()) {
            return Err(SourceContextError::DuplicateTypedSite);
        }
        let Some(item_index) = item_by_shell.get(&binding.shell).copied() else {
            return Err(SourceContextError::UnknownBindingShell { index });
        };
        if index != 0 && item_index < previous_item {
            return Err(SourceContextError::ReorderedBindings { index });
        }
        previous_item = item_index;
        let item = &input.items[item_index];
        if item.recovery == SourceItemRecovery::Recovered {
            return Err(SourceContextError::RecoveredItemClaimsBinding { index });
        }
        if !range_contains(item.source_range, binding.declaration_range)
            || !range_contains(item.source_range, binding.written_type_range)
        {
            return Err(SourceContextError::BindingRangeMismatch { index });
        }
        match (&item.role, &binding.context_owner, &binding.role) {
            (
                SourceItemRole::Reserve,
                SourceBindingContextOwner::Module,
                SourceBindingSiteRole::ReserveDefault,
            ) => {}
            (
                SourceItemRole::DefinitionBlock,
                SourceBindingContextOwner::Shell(owner),
                SourceBindingSiteRole::DefinitionParameter { local },
            ) if owner == &item.shell => {
                if local.spelling() != binding.spelling
                    || local.declaration_range() != binding.declaration_range
                    || local.visible_after_ordinal() != binding.source_ordinal
                    || Some(local.scope()) != item.local_scope.as_ref()
                {
                    return Err(SourceContextError::StaleLocalIdentity { index });
                }
            }
            _ => return Err(SourceContextError::RoleMismatch { index }),
        }
        if matches!(
            &binding.role,
            SourceBindingSiteRole::DefinitionParameter { .. }
        ) && !binding_names.insert((context_ids[item_index], binding.spelling.clone()))
        {
            return Err(SourceContextError::DuplicateSameScopeBinding { index });
        }
        counts[item_index] += 1;
    }
    for (index, item) in input.items.iter().enumerate() {
        if item.recovery == SourceItemRecovery::Normal && counts[index] == 0 {
            return Err(SourceContextError::PartialItem { index });
        }
    }
    if input.items.len() != 2
        || input.items[0].role != SourceItemRole::Reserve
        || input.items[1].role != SourceItemRole::DefinitionBlock
        || counts[0] != 1
        || (input.items[1].recovery == SourceItemRecovery::Normal && counts[1] != 1)
    {
        return Err(SourceContextError::UnsupportedTaskShape);
    }
    if input.items[1].recovery == SourceItemRecovery::Normal
        && input.bindings[0].spelling != input.bindings[1].spelling
    {
        return Err(SourceContextError::MissingRequiredShadow);
    }

    Ok(ValidatedInput {
        item_by_shell,
        context_ids,
        local_context_ids,
    })
}

fn build_projection(
    input: SourceBindingContextInput,
    validated: ValidatedInput,
) -> Result<SourceBindingContextBuild, SourceContextError> {
    let mut binding_tables = BindingTable::new();
    let mut binding_ids_by_item = vec![Vec::new(); input.items.len()];
    let mut declaration_rows = Vec::with_capacity(input.bindings.len());
    let mut visible_by_context: BTreeMap<BindingContextId, Vec<BindingId>> = BTreeMap::new();
    visible_by_context.insert(BindingContextId::new(0), Vec::new());

    for (index, binding) in input.bindings.iter().enumerate() {
        let item_index = validated.item_by_shell[&binding.shell];
        let owner_context = validated.context_ids[item_index];
        let shadowed_binding =
            visible_for_item(item_index, &input, &validated, &visible_by_context)
                .into_iter()
                .rev()
                .find(|binding_id| {
                    binding_tables
                        .get(*binding_id)
                        .is_some_and(|entry| entry.spelling == binding.spelling)
                });
        let (kind, identity, status) = match &binding.role {
            SourceBindingSiteRole::ReserveDefault => (
                BindingKind::ReservedVariable,
                BinderIdentity::ReservedVariable {
                    spelling: binding.spelling.clone(),
                    declaration_range: binding.declaration_range,
                },
                BindingStatus::Reserved,
            ),
            SourceBindingSiteRole::DefinitionParameter { local } => (
                BindingKind::DefinitionParameter,
                BinderIdentity::ResolverLocal {
                    scope: local.scope().clone(),
                    ordinal: local.visible_after_ordinal(),
                    declaration_range: local.declaration_range(),
                },
                BindingStatus::Active,
            ),
        };
        let binding_id = binding_tables.insert(BindingDraft {
            spelling: binding.spelling.clone(),
            kind,
            identity,
            owner_context,
            declaration_range: binding.declaration_range,
            visible_after_ordinal: binding.source_ordinal,
            type_site: BindingTypeSite::Source(binding.written_type_range),
            status,
            captured: CapturedFreeVariables::default(),
            diagnostics: Vec::new(),
            recovery: BindingRecoveryState::Normal,
        });
        binding_ids_by_item[item_index].push(binding_id);
        visible_by_context
            .entry(owner_context)
            .or_default()
            .push(binding_id);
        declaration_rows.push(SourceDeclaration {
            id: SourceDeclarationId::new(index),
            item: SourceItemId::new(item_index),
            binding: binding_id,
            source_ordinal: binding.source_ordinal,
            spelling: binding.spelling.clone(),
            declaration_range: binding.declaration_range,
            written_type_range: binding.written_type_range,
            site: binding.site.clone(),
            role: binding.role.clone(),
            binding_context: owner_context,
            local_context: validated.local_context_ids[item_index],
            shadowed_binding,
            predecessor: index.checked_sub(1).map(SourceDeclarationId::new),
        });
    }

    let module_bindings = input
        .items
        .iter()
        .enumerate()
        .filter(|(_, item)| item.role == SourceItemRole::Reserve)
        .flat_map(|(index, _)| binding_ids_by_item[index].iter().copied())
        .collect::<Vec<_>>();
    let mut contexts = BindingContextTable::new();
    contexts.insert(BindingContextDraft {
        owner: BindingContextOwner::Module,
        parent: None,
        layer: BindingContextLayer::Module,
        lexical_scope: None,
        bindings: module_bindings.clone(),
        visible_bindings: module_bindings.clone(),
        recovery: BindingContextRecovery::Normal,
    });

    let mut local_contexts = LocalTypeContextTable::new();
    local_contexts.insert(LocalTypeContextDraft {
        owner: input.module_site.clone(),
        parent: None,
        layer: TypeContextLayer::Module,
        bindings: binding_sites(&module_bindings, &declaration_rows),
        introduced_assumptions: Vec::new(),
        visible_facts: Vec::new(),
        recovery: ContextRecoveryState::Normal,
    });
    let mut context_links = vec![SourceContextLink {
        binding_context: BindingContextId::new(0),
        local_context: LocalTypeContextId::new(0),
        item: None,
    }];
    let mut diagnostics = BindingDiagnosticTable::new();
    let mut recovered_result = None;

    for (item_index, item) in input.items.iter().enumerate() {
        if item.role != SourceItemRole::DefinitionBlock {
            continue;
        }
        let binding_context = validated.context_ids[item_index];
        let local_context = validated.local_context_ids[item_index];
        let parent_item = item
            .parent
            .as_ref()
            .and_then(|parent| validated.item_by_shell.get(parent).copied());
        let parent_binding_context = parent_item
            .map(|index| validated.context_ids[index])
            .unwrap_or(BindingContextId::new(0));
        let parent_local_context = parent_item
            .map(|index| validated.local_context_ids[index])
            .unwrap_or(LocalTypeContextId::new(0));
        let mut visible =
            visible_context_bindings(parent_binding_context, &contexts, &module_bindings);
        visible.extend(binding_ids_by_item[item_index].iter().copied());
        let recovery = if item.recovery == SourceItemRecovery::Recovered {
            BindingContextRecovery::Recovered
        } else {
            BindingContextRecovery::Normal
        };
        contexts.insert(BindingContextDraft {
            owner: BindingContextOwner::DeclarationShell(item.shell),
            parent: Some(parent_binding_context),
            layer: BindingContextLayer::Declaration,
            lexical_scope: item.local_scope.clone(),
            bindings: binding_ids_by_item[item_index].clone(),
            visible_bindings: visible,
            recovery,
        });
        if item.recovery == SourceItemRecovery::Recovered {
            let diagnostic = diagnostics.insert(BindingDiagnosticDraft {
                source_range: Some(item.source_range),
                class: BindingDiagnosticClass::RecoveredContextBoundary,
                severity: BindingDiagnosticSeverity::Error,
                message_key: "checker.binding.source_context.recovered".to_owned(),
                recovery: BindingDiagnosticRecovery::Recovery,
            });
            recovered_result = Some((item.shell, binding_context, diagnostic));
        } else {
            local_contexts.insert(LocalTypeContextDraft {
                owner: item.site.clone(),
                parent: Some(parent_local_context),
                layer: TypeContextLayer::Declaration,
                bindings: binding_sites(&binding_ids_by_item[item_index], &declaration_rows),
                introduced_assumptions: Vec::new(),
                visible_facts: Vec::new(),
                recovery: ContextRecoveryState::Normal,
            });
            context_links.push(SourceContextLink {
                binding_context,
                local_context,
                item: Some(SourceItemId::new(item_index)),
            });
        }
    }

    let binding_env = BindingEnv::try_new(BindingEnvParts {
        source_id: input.source_id,
        module_id: input.module_id.clone(),
        contexts,
        bindings: binding_tables,
        diagnostics,
    })
    .map_err(|error| SourceContextError::BindingEnv(error.to_string()))?;

    if let Some((recovered_shell, recovered_context, diagnostic)) = recovered_result {
        return Ok(SourceBindingContextBuild::Incomplete(
            SourceBindingContextIncomplete {
                binding_env,
                recovered_shell,
                recovered_context,
                diagnostic,
            },
        ));
    }

    let items = input
        .items
        .into_iter()
        .enumerate()
        .map(|(index, item)| SourceItem {
            id: SourceItemId::new(index),
            shell: item.shell,
            shell_ordinal: item.shell_ordinal,
            role: item.role,
            source_range: item.source_range,
            parent: item.parent,
            visibility: item.visibility,
            site: item.site,
            local_scope: item.local_scope,
            recovery: item.recovery,
            binding_context: validated.context_ids[index],
            local_context: validated.local_context_ids[index],
            predecessor: index.checked_sub(1).map(SourceItemId::new),
        })
        .collect();
    let handoff = SourceBindingContextHandoff {
        source_id: input.source_id,
        module_id: input.module_id,
        binding_env,
        local_contexts,
        items: SourceItemTable { entries: items },
        declarations: SourceDeclarationTable {
            entries: declaration_rows,
        },
        context_links: SourceContextLinkTable {
            entries: context_links,
        },
    };
    Ok(SourceBindingContextBuild::Complete(
        SourceBindingContextProjection { handoff },
    ))
}

fn visible_for_item(
    item_index: usize,
    input: &SourceBindingContextInput,
    validated: &ValidatedInput,
    visible_by_context: &BTreeMap<BindingContextId, Vec<BindingId>>,
) -> Vec<BindingId> {
    let item = &input.items[item_index];
    let parent_context = item
        .parent
        .as_ref()
        .and_then(|parent| validated.item_by_shell.get(parent).copied())
        .map(|index| validated.context_ids[index])
        .unwrap_or(BindingContextId::new(0));
    visible_by_context
        .get(&parent_context)
        .cloned()
        .unwrap_or_default()
}

fn visible_context_bindings(
    context: BindingContextId,
    contexts: &BindingContextTable,
    module_bindings: &[BindingId],
) -> Vec<BindingId> {
    contexts
        .get(context)
        .map(|entry| entry.visible_bindings.clone())
        .unwrap_or_else(|| module_bindings.to_vec())
}

fn binding_sites(
    bindings: &[BindingId],
    declarations: &[SourceDeclaration],
) -> Vec<BindingTypeRef> {
    bindings
        .iter()
        .filter_map(|binding| {
            declarations
                .iter()
                .find(|declaration| declaration.binding == *binding)
                .map(|declaration| BindingTypeRef::Site(declaration.site.clone()))
        })
        .collect()
}

fn range_contains(outer: SourceRange, inner: SourceRange) -> bool {
    outer.source_id == inner.source_id && outer.start <= inner.start && inner.end <= outer.end
}

fn item_role_key(role: SourceItemRole) -> &'static str {
    match role {
        SourceItemRole::Reserve => "reserve",
        SourceItemRole::DefinitionBlock => "definition-block",
    }
}

fn binding_role_key(role: &SourceBindingSiteRole) -> &'static str {
    match role {
        SourceBindingSiteRole::ReserveDefault => "reserve-default",
        SourceBindingSiteRole::DefinitionParameter { .. } => "definition-parameter",
    }
}

/// Validation failure for a source/binding-context transaction.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum SourceContextError {
    MissingItems,
    UnsupportedTaskShape,
    DuplicateShell { index: usize },
    StaleShellOrdinal { index: usize },
    ReorderedItems { index: usize },
    ModuleMismatch { index: usize },
    ItemSourceMismatch { index: usize },
    InvalidParent { index: usize },
    UnsupportedVisibility { index: usize },
    InvalidItemContext { index: usize },
    UnsupportedRecovery { index: usize },
    DuplicateTypedSite,
    StaleBindingOrdinal { index: usize },
    ReorderedBindings { index: usize },
    EmptyBindingSpelling { index: usize },
    BindingSourceMismatch { index: usize },
    UnknownBindingShell { index: usize },
    BindingRangeMismatch { index: usize },
    RoleMismatch { index: usize },
    StaleLocalIdentity { index: usize },
    DuplicateSameScopeBinding { index: usize },
    RecoveredBinding { index: usize },
    RecoveredItemClaimsBinding { index: usize },
    PartialItem { index: usize },
    MissingRequiredShadow,
    BindingEnv(String),
    IncompleteRecovery,
}

impl fmt::Display for SourceContextError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingItems => formatter.write_str("source context requires source items"),
            Self::UnsupportedTaskShape => {
                formatter.write_str("source context is outside the Task 248 shape")
            }
            Self::DuplicateShell { index } => {
                write!(formatter, "source item {index} duplicates a shell")
            }
            Self::StaleShellOrdinal { index } => {
                write!(formatter, "source item {index} has a stale shell ordinal")
            }
            Self::ReorderedItems { index } => write!(formatter, "source item {index} is reordered"),
            Self::ModuleMismatch { index } => {
                write!(formatter, "source item {index} uses another module")
            }
            Self::ItemSourceMismatch { index } => {
                write!(formatter, "source item {index} uses another source")
            }
            Self::InvalidParent { index } => {
                write!(formatter, "source item {index} has an invalid parent")
            }
            Self::UnsupportedVisibility { index } => {
                write!(formatter, "source item {index} has unsupported visibility")
            }
            Self::InvalidItemContext { index } => write!(
                formatter,
                "source item {index} has an invalid context payload"
            ),
            Self::UnsupportedRecovery { index } => {
                write!(formatter, "source item {index} has unsupported recovery")
            }
            Self::DuplicateTypedSite => {
                formatter.write_str("source context contains a duplicate typed site")
            }
            Self::StaleBindingOrdinal { index } => {
                write!(formatter, "source binding {index} has a stale ordinal")
            }
            Self::ReorderedBindings { index } => {
                write!(formatter, "source binding {index} is reordered")
            }
            Self::EmptyBindingSpelling { index } => {
                write!(formatter, "source binding {index} has empty spelling")
            }
            Self::BindingSourceMismatch { index } => {
                write!(formatter, "source binding {index} uses another source")
            }
            Self::UnknownBindingShell { index } => {
                write!(formatter, "source binding {index} has an unknown shell")
            }
            Self::BindingRangeMismatch { index } => write!(
                formatter,
                "source binding {index} is outside its shell range"
            ),
            Self::RoleMismatch { index } => write!(
                formatter,
                "source binding {index} has a wrong role or context"
            ),
            Self::StaleLocalIdentity { index } => {
                write!(formatter, "source binding {index} has stale local identity")
            }
            Self::DuplicateSameScopeBinding { index } => write!(
                formatter,
                "source binding {index} duplicates a same-scope binding"
            ),
            Self::RecoveredBinding { index } => {
                write!(formatter, "source binding {index} is recovered")
            }
            Self::RecoveredItemClaimsBinding { index } => write!(
                formatter,
                "source binding {index} belongs to a recovered item"
            ),
            Self::PartialItem { index } => {
                write!(formatter, "source item {index} has no binding payload")
            }
            Self::MissingRequiredShadow => {
                formatter.write_str("source context is missing the required binding shadow")
            }
            Self::BindingEnv(error) => write!(formatter, "invalid binding environment: {error}"),
            Self::IncompleteRecovery => {
                formatter.write_str("recovered source context is incomplete")
            }
        }
    }
}

impl Error for SourceContextError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_input_is_rejected_without_syntax_or_shell_fabrication() {
        let source = {
            use mizar_session::SessionIdAllocator as _;
            let snapshot = mizar_session::BuildSnapshotId::from_published_schema_str(&format!(
                "mizar-session-build-snapshot-v1:{}",
                "11".repeat(32)
            ))
            .expect("valid snapshot");
            mizar_session::InMemorySessionIdAllocator::new()
                .next_source_id(snapshot)
                .expect("source id")
        };
        assert_eq!(
            SourceBindingContextProducer::build(SourceBindingContextInput {
                source_id: source,
                module_id: ModuleId::new(
                    mizar_session::PackageId::new("pkg"),
                    mizar_session::ModulePath::new("module"),
                ),
                module_site: TypedSiteRef::Node(crate::typed_ast::TypedNodeId::new(0)),
                items: Vec::new(),
                bindings: Vec::new(),
            }),
            Err(SourceContextError::MissingItems)
        );
    }

    #[test]
    fn production_boundary_stays_syntax_free_and_does_not_claim_later_payloads() {
        let source = include_str!("source_context.rs");
        for forbidden in [
            concat!("mizar", "_syntax"),
            concat!("Surface", "NodeId"),
            concat!("Normalized", "Type"),
            concat!("Proof", "Context"),
            concat!("Accepted", "Fact"),
        ] {
            assert!(
                !source.contains(forbidden),
                "source context exposes {forbidden}"
            );
        }
    }
}
