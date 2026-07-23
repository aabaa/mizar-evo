//! Binding environment data tables for checker phase 6 context construction.

use mizar_resolve::{
    declarations::DeclarationShellId,
    env::{ResolverShellId, SymbolEnv},
    names::{LocalTermBinding, LocalTermScope},
    resolved_ast::{ModuleId, NameResolution, ResolvedAst, SymbolId},
};
use mizar_session::{SourceId, SourceRange};
use std::{
    cmp::Ordering,
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

dense_id!(BindingContextId);
dense_id!(BindingId);
dense_id!(BindingDiagnosticId);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BindingEnv {
    source_id: SourceId,
    module_id: ModuleId,
    contexts: BindingContextTable,
    bindings: BindingTable,
    diagnostics: BindingDiagnosticTable,
}

impl BindingEnv {
    pub fn try_new(parts: BindingEnvParts) -> Result<Self, BindingEnvError> {
        validate_binding_env(&parts)?;
        Ok(Self {
            source_id: parts.source_id,
            module_id: parts.module_id,
            contexts: parts.contexts,
            bindings: parts.bindings,
            diagnostics: parts.diagnostics,
        })
    }

    pub fn module_shell(
        resolved: &ResolvedAst,
        symbols: &SymbolEnv,
    ) -> Result<Self, BindingEnvError> {
        Self::module_shell_from_parts(
            resolved.source_id(),
            resolved.module_id().clone(),
            symbols.module_id().clone(),
        )
    }

    pub const fn source_id(&self) -> SourceId {
        self.source_id
    }

    pub const fn module_id(&self) -> &ModuleId {
        &self.module_id
    }

    pub const fn contexts(&self) -> &BindingContextTable {
        &self.contexts
    }

    pub const fn bindings(&self) -> &BindingTable {
        &self.bindings
    }

    pub const fn diagnostics(&self) -> &BindingDiagnosticTable {
        &self.diagnostics
    }

    pub fn lookup(&self, site: &BindingLookupSite) -> Result<BindingLookupResult, BindingEnvError> {
        let context =
            self.contexts
                .get(site.context)
                .ok_or(BindingEnvError::InvalidLookupContext {
                    context: site.context,
                })?;
        let mut seen = BTreeSet::new();
        let mut local_candidates = Vec::new();
        let mut forward_candidates = Vec::new();
        let mut missing_local_payload = false;

        for binding_id in context.visible_bindings.iter().copied() {
            if !seen.insert(binding_id) {
                continue;
            }
            let Some(binding) = self.bindings.get(binding_id) else {
                return Err(BindingEnvError::InvalidVisibleBinding {
                    context: site.context,
                    binding: binding_id,
                });
            };
            if binding.spelling != site.spelling {
                continue;
            }
            if site.lexical_scope.is_none()
                && matches!(binding.identity, BinderIdentity::ResolverLocal { .. })
            {
                missing_local_payload = true;
                continue;
            }
            let Some(priority) = lookup_priority(binding, site) else {
                continue;
            };
            if binding.visible_after_ordinal >= site.ordinal {
                forward_candidates.push(binding_id);
            } else {
                local_candidates.push((binding_id, priority));
            }
        }

        if missing_local_payload {
            if let Some(resolution) = &site.resolver_resolution {
                return Ok(BindingLookupResult::Resolver(resolution.clone()));
            }
            return Ok(BindingLookupResult::MissingExternalPayload {
                diagnostic: lookup_diagnostic(
                    BindingDiagnosticClass::ExternalDependencyGap,
                    BindingDiagnosticSeverity::Note,
                    "checker.binding.external.use_site_scope",
                ),
            });
        }
        if let Some(result) = select_local_binding(local_candidates) {
            return Ok(result);
        }
        if !forward_candidates.is_empty() {
            forward_candidates.sort();
            return Ok(BindingLookupResult::ForwardReference {
                candidates: forward_candidates,
                diagnostic: lookup_diagnostic(
                    BindingDiagnosticClass::ForwardLocalReference,
                    BindingDiagnosticSeverity::Error,
                    "checker.binding.forward_reference",
                ),
            });
        }
        if let Some(resolution) = &site.resolver_resolution {
            return Ok(BindingLookupResult::Resolver(resolution.clone()));
        }
        if context.lexical_scope.is_none() && site.lexical_scope.is_none() {
            return Ok(BindingLookupResult::MissingExternalPayload {
                diagnostic: lookup_diagnostic(
                    BindingDiagnosticClass::ExternalDependencyGap,
                    BindingDiagnosticSeverity::Note,
                    "checker.binding.external.use_site_scope",
                ),
            });
        }
        Ok(BindingLookupResult::Unresolved)
    }

    pub fn debug_text(&self) -> String {
        let mut output = String::from("binding-env-debug-v1\n");
        output.push_str("module: ");
        write_module_id(&mut output, &self.module_id);
        output.push('\n');
        write_contexts(&mut output, &self.contexts);
        write_bindings(&mut output, &self.bindings);
        write_diagnostics(&mut output, &self.diagnostics);
        output
    }

    fn module_shell_from_parts(
        source_id: SourceId,
        module_id: ModuleId,
        symbol_module_id: ModuleId,
    ) -> Result<Self, BindingEnvError> {
        if module_id != symbol_module_id {
            return Err(BindingEnvError::ModuleMismatch {
                resolved: module_id,
                symbols: symbol_module_id,
            });
        }

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

        let mut diagnostics = BindingDiagnosticTable::new();
        for message_key in [
            "checker.binding.external.local_bindings",
            "checker.binding.external.use_site_scope",
            "checker.binding.external.reserve_payload",
            "checker.binding.external.closure_payload",
        ] {
            diagnostics.insert(BindingDiagnosticDraft {
                source_range: None,
                class: BindingDiagnosticClass::ExternalDependencyGap,
                severity: BindingDiagnosticSeverity::Note,
                message_key: message_key.to_owned(),
                recovery: BindingDiagnosticRecovery::Degraded,
            });
        }

        Self::try_new(BindingEnvParts {
            source_id,
            module_id,
            contexts,
            bindings: BindingTable::new(),
            diagnostics,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BindingEnvParts {
    pub source_id: SourceId,
    pub module_id: ModuleId,
    pub contexts: BindingContextTable,
    pub bindings: BindingTable,
    pub diagnostics: BindingDiagnosticTable,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct BindingContextTable {
    contexts: Vec<BindingContext>,
}

impl BindingContextTable {
    pub const fn new() -> Self {
        Self {
            contexts: Vec::new(),
        }
    }

    pub fn insert(&mut self, mut draft: BindingContextDraft) -> BindingContextId {
        normalize_ids(&mut draft.bindings);
        normalize_ids(&mut draft.visible_bindings);
        let id = BindingContextId::new(self.contexts.len());
        self.contexts.push(BindingContext {
            id,
            owner: draft.owner,
            parent: draft.parent,
            layer: draft.layer,
            lexical_scope: draft.lexical_scope,
            bindings: draft.bindings,
            visible_bindings: draft.visible_bindings,
            recovery: draft.recovery,
        });
        id
    }

    pub fn get(&self, id: BindingContextId) -> Option<&BindingContext> {
        self.contexts.get(id.index())
    }

    pub fn iter(&self) -> impl Iterator<Item = (BindingContextId, &BindingContext)> {
        self.contexts
            .iter()
            .enumerate()
            .map(|(index, context)| (BindingContextId::new(index), context))
    }

    pub const fn len(&self) -> usize {
        self.contexts.len()
    }

    pub const fn is_empty(&self) -> bool {
        self.contexts.is_empty()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BindingContext {
    pub id: BindingContextId,
    pub owner: BindingContextOwner,
    pub parent: Option<BindingContextId>,
    pub layer: BindingContextLayer,
    pub lexical_scope: Option<LocalTermScope>,
    pub bindings: Vec<BindingId>,
    pub visible_bindings: Vec<BindingId>,
    pub recovery: BindingContextRecovery,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BindingContextDraft {
    pub owner: BindingContextOwner,
    pub parent: Option<BindingContextId>,
    pub layer: BindingContextLayer,
    pub lexical_scope: Option<LocalTermScope>,
    pub bindings: Vec<BindingId>,
    pub visible_bindings: Vec<BindingId>,
    pub recovery: BindingContextRecovery,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum BindingContextOwner {
    Module,
    DeclarationShell(DeclarationShellId),
    Generated(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum BindingContextLayer {
    Module,
    Declaration,
    Proof,
    Block,
    Expression,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum BindingContextRecovery {
    Normal,
    Recovered,
    Degraded,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct BindingTable {
    bindings: Vec<BindingEntry>,
}

impl BindingTable {
    pub const fn new() -> Self {
        Self {
            bindings: Vec::new(),
        }
    }

    pub fn insert(&mut self, mut draft: BindingDraft) -> BindingId {
        normalize_ids(&mut draft.diagnostics);
        draft.captured = CapturedFreeVariables::new(draft.captured.into_vec());
        let id = BindingId::new(self.bindings.len());
        self.bindings.push(BindingEntry {
            id,
            spelling: draft.spelling,
            kind: draft.kind,
            identity: draft.identity,
            owner_context: draft.owner_context,
            declaration_range: draft.declaration_range,
            visible_after_ordinal: draft.visible_after_ordinal,
            type_site: draft.type_site,
            status: draft.status,
            captured: draft.captured,
            diagnostics: draft.diagnostics,
            recovery: draft.recovery,
        });
        id
    }

    pub fn get(&self, id: BindingId) -> Option<&BindingEntry> {
        self.bindings.get(id.index())
    }

    pub fn iter(&self) -> impl Iterator<Item = (BindingId, &BindingEntry)> {
        self.bindings
            .iter()
            .enumerate()
            .map(|(index, binding)| (BindingId::new(index), binding))
    }

    pub fn canonical_iter(&self) -> impl Iterator<Item = (BindingId, &BindingEntry)> {
        let mut ids = self.iter().map(|(id, _)| id).collect::<Vec<_>>();
        ids.sort_by(|left, right| {
            let left_entry = self.get(*left).expect("id came from iterator");
            let right_entry = self.get(*right).expect("id came from iterator");
            binding_order_key(left_entry).cmp(&binding_order_key(right_entry))
        });
        ids.into_iter()
            .map(|id| (id, self.get(id).expect("id came from iterator")))
    }

    pub const fn len(&self) -> usize {
        self.bindings.len()
    }

    pub const fn is_empty(&self) -> bool {
        self.bindings.is_empty()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BindingEntry {
    pub id: BindingId,
    pub spelling: String,
    pub kind: BindingKind,
    pub identity: BinderIdentity,
    pub owner_context: BindingContextId,
    pub declaration_range: SourceRange,
    pub visible_after_ordinal: usize,
    pub type_site: BindingTypeSite,
    pub status: BindingStatus,
    pub captured: CapturedFreeVariables,
    pub diagnostics: Vec<BindingDiagnosticId>,
    pub recovery: BindingRecoveryState,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BindingDraft {
    pub spelling: String,
    pub kind: BindingKind,
    pub identity: BinderIdentity,
    pub owner_context: BindingContextId,
    pub declaration_range: SourceRange,
    pub visible_after_ordinal: usize,
    pub type_site: BindingTypeSite,
    pub status: BindingStatus,
    pub captured: CapturedFreeVariables,
    pub diagnostics: Vec<BindingDiagnosticId>,
    pub recovery: BindingRecoveryState,
}

impl BindingDraft {
    pub fn from_local_term(
        owner_context: BindingContextId,
        kind: BindingKind,
        binding: &LocalTermBinding,
    ) -> Self {
        Self {
            spelling: binding.spelling().to_owned(),
            kind,
            identity: BinderIdentity::ResolverLocal {
                scope: binding.scope().clone(),
                ordinal: binding.visible_after_ordinal(),
                declaration_range: binding.declaration_range(),
            },
            owner_context,
            declaration_range: binding.declaration_range(),
            visible_after_ordinal: binding.visible_after_ordinal(),
            type_site: BindingTypeSite::Missing,
            status: BindingStatus::Active,
            captured: CapturedFreeVariables::default(),
            diagnostics: Vec::new(),
            recovery: BindingRecoveryState::Normal,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum BindingKind {
    QuantifierBinder,
    DefinitionParameter,
    LocalAbbreviation,
    ReservedVariable,
    LetBinding,
    Generated,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum BinderIdentity {
    ResolverLocal {
        scope: LocalTermScope,
        ordinal: usize,
        declaration_range: SourceRange,
    },
    DefinitionShell {
        symbol: SymbolId,
        shell: ResolverShellId,
    },
    ReservedVariable {
        spelling: String,
        declaration_range: SourceRange,
    },
    Generated {
        context: BindingContextId,
        counter: u32,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum BindingTypeSite {
    Missing,
    Source(SourceRange),
    Deferred(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum BindingStatus {
    Active,
    Reserved,
    Degraded,
    Omitted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum BindingRecoveryState {
    Normal,
    Recovered,
    Degraded,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CapturedFreeVariables {
    identities: Vec<BinderIdentity>,
}

impl CapturedFreeVariables {
    pub fn new(mut identities: Vec<BinderIdentity>) -> Self {
        identities.sort_by(binder_identity_cmp);
        identities.dedup();
        Self { identities }
    }

    pub fn identities(&self) -> &[BinderIdentity] {
        &self.identities
    }

    pub fn into_vec(self) -> Vec<BinderIdentity> {
        self.identities
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct BindingDiagnosticTable {
    diagnostics: Vec<BindingDiagnostic>,
}

impl BindingDiagnosticTable {
    pub const fn new() -> Self {
        Self {
            diagnostics: Vec::new(),
        }
    }

    pub fn insert(&mut self, draft: BindingDiagnosticDraft) -> BindingDiagnosticId {
        let id = BindingDiagnosticId::new(self.diagnostics.len());
        self.diagnostics.push(BindingDiagnostic {
            id,
            source_range: draft.source_range,
            class: draft.class,
            severity: draft.severity,
            message_key: draft.message_key,
            recovery: draft.recovery,
        });
        id
    }

    pub fn get(&self, id: BindingDiagnosticId) -> Option<&BindingDiagnostic> {
        self.diagnostics.get(id.index())
    }

    pub fn iter(&self) -> impl Iterator<Item = (BindingDiagnosticId, &BindingDiagnostic)> {
        self.diagnostics
            .iter()
            .enumerate()
            .map(|(index, diagnostic)| (BindingDiagnosticId::new(index), diagnostic))
    }

    pub fn canonical_iter(
        &self,
    ) -> impl Iterator<Item = (BindingDiagnosticId, &BindingDiagnostic)> {
        let mut ids = self.iter().map(|(id, _)| id).collect::<Vec<_>>();
        ids.sort_by(|left, right| {
            let left_entry = self.get(*left).expect("id came from iterator");
            let right_entry = self.get(*right).expect("id came from iterator");
            diagnostic_order_key(left_entry).cmp(&diagnostic_order_key(right_entry))
        });
        ids.into_iter()
            .map(|id| (id, self.get(id).expect("id came from iterator")))
    }

    pub const fn len(&self) -> usize {
        self.diagnostics.len()
    }

    pub const fn is_empty(&self) -> bool {
        self.diagnostics.is_empty()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BindingDiagnostic {
    pub id: BindingDiagnosticId,
    pub source_range: Option<SourceRange>,
    pub class: BindingDiagnosticClass,
    pub severity: BindingDiagnosticSeverity,
    pub message_key: String,
    pub recovery: BindingDiagnosticRecovery,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BindingDiagnosticDraft {
    pub source_range: Option<SourceRange>,
    pub class: BindingDiagnosticClass,
    pub severity: BindingDiagnosticSeverity,
    pub message_key: String,
    pub recovery: BindingDiagnosticRecovery,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum BindingDiagnosticClass {
    DuplicateLocalBinding,
    ForwardLocalReference,
    UnsupportedSourceShape,
    ExternalDependencyGap,
    IllegalNestedReserve,
    RecoveredContextBoundary,
    AmbiguousLocalBinding,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum BindingDiagnosticSeverity {
    Error,
    Warning,
    Note,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum BindingDiagnosticRecovery {
    Normal,
    Recovery,
    Degraded,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BindingLookupSite {
    pub spelling: String,
    pub context: BindingContextId,
    pub lexical_scope: Option<LocalTermScope>,
    pub ordinal: usize,
    /// Resolver `NameRefEntry::resolution()` payload already extracted by the
    /// caller. `ReferenceSite` and `ResolvedNodeId` remain resolver-owned.
    pub resolver_resolution: Option<NameResolution>,
}

impl BindingLookupSite {
    pub fn new(
        spelling: impl Into<String>,
        context: BindingContextId,
        lexical_scope: Option<LocalTermScope>,
        ordinal: usize,
    ) -> Self {
        Self {
            spelling: spelling.into(),
            context,
            lexical_scope,
            ordinal,
            resolver_resolution: None,
        }
    }

    pub fn with_resolver_resolution(mut self, resolution: NameResolution) -> Self {
        self.resolver_resolution = Some(resolution);
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum BindingLookupResult {
    Local(BindingId),
    Resolver(NameResolution),
    Ambiguous {
        candidates: Vec<BindingId>,
        diagnostic: BindingDiagnosticDraft,
    },
    ForwardReference {
        candidates: Vec<BindingId>,
        diagnostic: BindingDiagnosticDraft,
    },
    MissingExternalPayload {
        diagnostic: BindingDiagnosticDraft,
    },
    Unresolved,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum BindingEnvError {
    ModuleMismatch {
        resolved: ModuleId,
        symbols: ModuleId,
    },
    MissingModuleContext,
    InvalidModuleContext {
        context: BindingContextId,
    },
    MultipleRootContexts {
        context: BindingContextId,
    },
    InvalidLookupContext {
        context: BindingContextId,
    },
    InvalidContextParent {
        context: BindingContextId,
        parent: BindingContextId,
    },
    ContextCycle {
        context: BindingContextId,
    },
    InvalidContextBinding {
        context: BindingContextId,
        binding: BindingId,
    },
    InvalidVisibleBinding {
        context: BindingContextId,
        binding: BindingId,
    },
    InvalidVisibleBindingOwner {
        context: BindingContextId,
        binding: BindingId,
        owner: BindingContextId,
    },
    InvalidBindingOwner {
        binding: BindingId,
        owner: BindingContextId,
    },
    InvalidReservedBindingOwner {
        binding: BindingId,
        owner: BindingContextId,
    },
    InconsistentResolverLocalIdentity {
        binding: BindingId,
    },
    InconsistentReservedVariableIdentity {
        binding: BindingId,
    },
    InvalidBindingKindIdentity {
        binding: BindingId,
    },
    InvalidGeneratedIdentityOwner {
        binding: BindingId,
        context: BindingContextId,
        owner: BindingContextId,
    },
    InvalidGeneratedIdentityContext {
        binding: BindingId,
        context: BindingContextId,
    },
    InvalidBindingRange {
        binding: BindingId,
    },
    InvalidBindingDiagnostic {
        binding: BindingId,
        diagnostic: BindingDiagnosticId,
    },
    InvalidDiagnosticRange {
        diagnostic: BindingDiagnosticId,
    },
}

impl fmt::Display for BindingEnvError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ModuleMismatch { resolved, symbols } => {
                formatter.write_str("resolved AST module ")?;
                write_module_id(formatter, resolved);
                formatter.write_str(" does not match symbol environment module ")?;
                write_module_id(formatter, symbols);
                Ok(())
            }
            Self::MissingModuleContext => {
                formatter.write_str("binding environment has no module context")
            }
            Self::InvalidModuleContext { context } => write!(
                formatter,
                "context {} is not the module root context",
                context.index()
            ),
            Self::MultipleRootContexts { context } => write!(
                formatter,
                "context {} is an extra root context",
                context.index()
            ),
            Self::InvalidLookupContext { context } => {
                write!(
                    formatter,
                    "lookup context {} does not exist",
                    context.index()
                )
            }
            Self::InvalidContextParent { context, parent } => write!(
                formatter,
                "context {} references missing parent {}",
                context.index(),
                parent.index()
            ),
            Self::ContextCycle { context } => {
                write!(
                    formatter,
                    "context {} participates in a cycle",
                    context.index()
                )
            }
            Self::InvalidContextBinding { context, binding } => write!(
                formatter,
                "context {} references missing owned binding {}",
                context.index(),
                binding.index()
            ),
            Self::InvalidVisibleBinding { context, binding } => write!(
                formatter,
                "context {} references missing visible binding {}",
                context.index(),
                binding.index()
            ),
            Self::InvalidVisibleBindingOwner {
                context,
                binding,
                owner,
            } => write!(
                formatter,
                "context {} exposes binding {} from non-ancestor context {}",
                context.index(),
                binding.index(),
                owner.index()
            ),
            Self::InvalidBindingOwner { binding, owner } => write!(
                formatter,
                "binding {} references missing owner context {}",
                binding.index(),
                owner.index()
            ),
            Self::InvalidReservedBindingOwner { binding, owner } => write!(
                formatter,
                "reserved binding {} is owned by non-module context {}",
                binding.index(),
                owner.index()
            ),
            Self::InconsistentResolverLocalIdentity { binding } => write!(
                formatter,
                "binding {} has resolver-local identity fields inconsistent with lookup fields",
                binding.index()
            ),
            Self::InconsistentReservedVariableIdentity { binding } => write!(
                formatter,
                "binding {} has reserved-variable identity fields inconsistent with lookup fields",
                binding.index()
            ),
            Self::InvalidBindingKindIdentity { binding } => write!(
                formatter,
                "binding {} has a kind inconsistent with its identity",
                binding.index()
            ),
            Self::InvalidGeneratedIdentityOwner {
                binding,
                context,
                owner,
            } => write!(
                formatter,
                "binding {} generated identity context {} does not match owner context {}",
                binding.index(),
                context.index(),
                owner.index()
            ),
            Self::InvalidGeneratedIdentityContext { binding, context } => write!(
                formatter,
                "binding {} references missing generated identity context {}",
                binding.index(),
                context.index()
            ),
            Self::InvalidBindingRange { binding } => {
                write!(
                    formatter,
                    "binding {} has a range from another source",
                    binding.index()
                )
            }
            Self::InvalidBindingDiagnostic {
                binding,
                diagnostic,
            } => write!(
                formatter,
                "binding {} references missing diagnostic {}",
                binding.index(),
                diagnostic.index()
            ),
            Self::InvalidDiagnosticRange { diagnostic } => write!(
                formatter,
                "diagnostic {} has a range from another source",
                diagnostic.index()
            ),
        }
    }
}

impl Error for BindingEnvError {}

#[derive(Clone, Copy, PartialEq, Eq)]
enum VisitState {
    Visiting,
    Done,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct LookupPriority {
    scope_depth: usize,
    visible_after_ordinal: usize,
    declaration_range: (usize, usize),
}

fn validate_binding_env(parts: &BindingEnvParts) -> Result<(), BindingEnvError> {
    validate_module_root(&parts.contexts)?;
    validate_contexts(parts)?;
    validate_bindings(parts)?;
    validate_diagnostics(parts)?;
    Ok(())
}

fn validate_module_root(contexts: &BindingContextTable) -> Result<(), BindingEnvError> {
    let Some(root) = contexts.get(BindingContextId::new(0)) else {
        return Err(BindingEnvError::MissingModuleContext);
    };
    if root.parent.is_some()
        || root.layer != BindingContextLayer::Module
        || root.owner != BindingContextOwner::Module
    {
        return Err(BindingEnvError::InvalidModuleContext {
            context: BindingContextId::new(0),
        });
    }
    for (context_id, context) in contexts.iter().skip(1) {
        if context.parent.is_none() {
            return Err(BindingEnvError::MultipleRootContexts {
                context: context_id,
            });
        }
    }
    Ok(())
}

fn validate_contexts(parts: &BindingEnvParts) -> Result<(), BindingEnvError> {
    let mut states = vec![None; parts.contexts.len()];
    for (context_id, context) in parts.contexts.iter() {
        if let Some(parent) = context.parent
            && parts.contexts.get(parent).is_none()
        {
            return Err(BindingEnvError::InvalidContextParent {
                context: context_id,
                parent,
            });
        }
        for binding in &context.bindings {
            let Some(entry) = parts.bindings.get(*binding) else {
                return Err(BindingEnvError::InvalidContextBinding {
                    context: context_id,
                    binding: *binding,
                });
            };
            if entry.owner_context != context_id {
                return Err(BindingEnvError::InvalidContextBinding {
                    context: context_id,
                    binding: *binding,
                });
            }
        }
        for binding in &context.visible_bindings {
            if parts.bindings.get(*binding).is_none() {
                return Err(BindingEnvError::InvalidVisibleBinding {
                    context: context_id,
                    binding: *binding,
                });
            }
        }
    }
    for index in 0..parts.contexts.len() {
        visit_context(BindingContextId::new(index), &parts.contexts, &mut states)?;
    }
    validate_visible_binding_owners(parts)?;
    Ok(())
}

fn visit_context(
    context_id: BindingContextId,
    contexts: &BindingContextTable,
    states: &mut [Option<VisitState>],
) -> Result<(), BindingEnvError> {
    match states[context_id.index()] {
        Some(VisitState::Done) => return Ok(()),
        Some(VisitState::Visiting) => {
            return Err(BindingEnvError::ContextCycle {
                context: context_id,
            });
        }
        None => {}
    }
    states[context_id.index()] = Some(VisitState::Visiting);
    if let Some(parent) = contexts.get(context_id).and_then(|context| context.parent) {
        visit_context(parent, contexts, states)?;
    }
    states[context_id.index()] = Some(VisitState::Done);
    Ok(())
}

fn validate_visible_binding_owners(parts: &BindingEnvParts) -> Result<(), BindingEnvError> {
    for (context_id, context) in parts.contexts.iter() {
        for binding in &context.visible_bindings {
            let entry = parts
                .bindings
                .get(*binding)
                .expect("visible binding existence already checked");
            let Some(owner_context) = parts.contexts.get(entry.owner_context) else {
                return Err(BindingEnvError::InvalidBindingOwner {
                    binding: *binding,
                    owner: entry.owner_context,
                });
            };
            if !owner_context.bindings.contains(binding)
                || !context_can_see_binding_owner(context_id, entry.owner_context, &parts.contexts)
            {
                return Err(BindingEnvError::InvalidVisibleBindingOwner {
                    context: context_id,
                    binding: *binding,
                    owner: entry.owner_context,
                });
            }
        }
    }
    Ok(())
}

fn context_can_see_binding_owner(
    context: BindingContextId,
    owner: BindingContextId,
    contexts: &BindingContextTable,
) -> bool {
    let mut cursor = Some(context);
    while let Some(context_id) = cursor {
        if context_id == owner {
            return true;
        }
        cursor = contexts.get(context_id).and_then(|context| context.parent);
    }
    false
}

fn validate_bindings(parts: &BindingEnvParts) -> Result<(), BindingEnvError> {
    for (binding_id, binding) in parts.bindings.iter() {
        if parts.contexts.get(binding.owner_context).is_none() {
            return Err(BindingEnvError::InvalidBindingOwner {
                binding: binding_id,
                owner: binding.owner_context,
            });
        }
        if binding.kind == BindingKind::ReservedVariable
            && binding.owner_context != BindingContextId::new(0)
        {
            return Err(BindingEnvError::InvalidReservedBindingOwner {
                binding: binding_id,
                owner: binding.owner_context,
            });
        }
        validate_binding_identity_coherence(parts, binding_id, binding)?;
        if !range_has_source(parts.source_id, binding.declaration_range)
            || !type_site_has_source(parts.source_id, &binding.type_site)
            || !identity_has_source(parts.source_id, &binding.identity)
            || !binding
                .captured
                .identities()
                .iter()
                .all(|identity| identity_has_source(parts.source_id, identity))
        {
            return Err(BindingEnvError::InvalidBindingRange {
                binding: binding_id,
            });
        }
        for diagnostic in &binding.diagnostics {
            if parts.diagnostics.get(*diagnostic).is_none() {
                return Err(BindingEnvError::InvalidBindingDiagnostic {
                    binding: binding_id,
                    diagnostic: *diagnostic,
                });
            }
        }
    }
    Ok(())
}

fn validate_binding_identity_coherence(
    parts: &BindingEnvParts,
    binding_id: BindingId,
    binding: &BindingEntry,
) -> Result<(), BindingEnvError> {
    if let BinderIdentity::ResolverLocal {
        ordinal,
        declaration_range,
        ..
    } = &binding.identity
        && (*ordinal != binding.visible_after_ordinal
            || *declaration_range != binding.declaration_range)
    {
        return Err(BindingEnvError::InconsistentResolverLocalIdentity {
            binding: binding_id,
        });
    }
    if let BinderIdentity::ReservedVariable {
        spelling,
        declaration_range,
    } = &binding.identity
        && (spelling != &binding.spelling || *declaration_range != binding.declaration_range)
    {
        return Err(BindingEnvError::InconsistentReservedVariableIdentity {
            binding: binding_id,
        });
    }
    if matches!(&binding.identity, BinderIdentity::ReservedVariable { .. })
        != (binding.kind == BindingKind::ReservedVariable)
    {
        return Err(BindingEnvError::InvalidBindingKindIdentity {
            binding: binding_id,
        });
    }
    if matches!(&binding.identity, BinderIdentity::Generated { .. })
        != (binding.kind == BindingKind::Generated)
    {
        return Err(BindingEnvError::InvalidBindingKindIdentity {
            binding: binding_id,
        });
    }
    validate_own_generated_identity_context(parts, binding_id, binding)?;
    for identity in binding.captured.identities() {
        validate_generated_identity_context(parts, binding_id, identity)?;
    }
    Ok(())
}

fn validate_own_generated_identity_context(
    parts: &BindingEnvParts,
    binding: BindingId,
    entry: &BindingEntry,
) -> Result<(), BindingEnvError> {
    if let BinderIdentity::Generated { context, .. } = &entry.identity {
        validate_generated_identity_context(parts, binding, &entry.identity)?;
        if *context != entry.owner_context {
            return Err(BindingEnvError::InvalidGeneratedIdentityOwner {
                binding,
                context: *context,
                owner: entry.owner_context,
            });
        }
    }
    Ok(())
}

fn validate_generated_identity_context(
    parts: &BindingEnvParts,
    binding: BindingId,
    identity: &BinderIdentity,
) -> Result<(), BindingEnvError> {
    if let BinderIdentity::Generated { context, .. } = identity
        && parts.contexts.get(*context).is_none()
    {
        return Err(BindingEnvError::InvalidGeneratedIdentityContext {
            binding,
            context: *context,
        });
    }
    Ok(())
}

fn validate_diagnostics(parts: &BindingEnvParts) -> Result<(), BindingEnvError> {
    for (diagnostic_id, diagnostic) in parts.diagnostics.iter() {
        if let Some(range) = diagnostic.source_range
            && !range_has_source(parts.source_id, range)
        {
            return Err(BindingEnvError::InvalidDiagnosticRange {
                diagnostic: diagnostic_id,
            });
        }
    }
    Ok(())
}

fn range_has_source(source_id: SourceId, range: SourceRange) -> bool {
    range.source_id == source_id
}

fn type_site_has_source(source_id: SourceId, site: &BindingTypeSite) -> bool {
    match site {
        BindingTypeSite::Missing | BindingTypeSite::Deferred(_) => true,
        BindingTypeSite::Source(range) => range_has_source(source_id, *range),
    }
}

fn identity_has_source(source_id: SourceId, identity: &BinderIdentity) -> bool {
    match identity {
        BinderIdentity::ResolverLocal {
            declaration_range, ..
        }
        | BinderIdentity::ReservedVariable {
            declaration_range, ..
        } => range_has_source(source_id, *declaration_range),
        BinderIdentity::DefinitionShell { .. } | BinderIdentity::Generated { .. } => true,
    }
}

fn lookup_priority(binding: &BindingEntry, site: &BindingLookupSite) -> Option<LookupPriority> {
    let scope_depth = match &binding.identity {
        BinderIdentity::ResolverLocal { scope, .. } => {
            let use_scope = site.lexical_scope.as_ref()?;
            scope_contains(scope, use_scope).then_some(scope.path().len())?
        }
        BinderIdentity::ReservedVariable { .. }
        | BinderIdentity::DefinitionShell { .. }
        | BinderIdentity::Generated { .. } => 0,
    };
    Some(LookupPriority {
        scope_depth,
        visible_after_ordinal: binding.visible_after_ordinal,
        declaration_range: range_key(binding.declaration_range),
    })
}

fn select_local_binding(
    candidates: Vec<(BindingId, LookupPriority)>,
) -> Option<BindingLookupResult> {
    let best_priority = candidates
        .iter()
        .map(|(_, priority)| *priority)
        .max_by(compare_lookup_priority)?;
    let mut best = candidates
        .into_iter()
        .filter_map(|(binding, priority)| (priority == best_priority).then_some(binding))
        .collect::<Vec<_>>();
    best.sort();
    if best.len() == 1 {
        Some(BindingLookupResult::Local(best[0]))
    } else {
        Some(BindingLookupResult::Ambiguous {
            candidates: best,
            diagnostic: lookup_diagnostic(
                BindingDiagnosticClass::AmbiguousLocalBinding,
                BindingDiagnosticSeverity::Error,
                "checker.binding.ambiguous_local",
            ),
        })
    }
}

fn compare_lookup_priority(left: &LookupPriority, right: &LookupPriority) -> Ordering {
    left.scope_depth
        .cmp(&right.scope_depth)
        .then_with(|| left.visible_after_ordinal.cmp(&right.visible_after_ordinal))
        .then_with(|| left.declaration_range.cmp(&right.declaration_range))
}

fn lookup_diagnostic(
    class: BindingDiagnosticClass,
    severity: BindingDiagnosticSeverity,
    message_key: &str,
) -> BindingDiagnosticDraft {
    BindingDiagnosticDraft {
        source_range: None,
        class,
        severity,
        message_key: message_key.to_owned(),
        recovery: BindingDiagnosticRecovery::Degraded,
    }
}

fn scope_contains(scope: &LocalTermScope, use_scope: &LocalTermScope) -> bool {
    use_scope.path().starts_with(scope.path())
}

fn normalize_ids<T: Ord>(ids: &mut Vec<T>) {
    ids.sort();
    ids.dedup();
}

fn binding_order_key(binding: &BindingEntry) -> (String, (usize, usize), usize) {
    (
        binding.spelling.clone(),
        range_key(binding.declaration_range),
        binding.id.index(),
    )
}

fn diagnostic_order_key(
    diagnostic: &BindingDiagnostic,
) -> (
    Option<(usize, usize)>,
    BindingDiagnosticClass,
    String,
    BindingDiagnosticId,
) {
    (
        diagnostic.source_range.map(range_key),
        diagnostic.class,
        diagnostic.message_key.clone(),
        diagnostic.id,
    )
}

fn binder_identity_cmp(left: &BinderIdentity, right: &BinderIdentity) -> Ordering {
    binder_identity_key(left).cmp(&binder_identity_key(right))
}

fn binder_identity_key(identity: &BinderIdentity) -> String {
    let mut output = String::new();
    write_binder_identity(&mut output, identity);
    output
}

fn range_key(range: SourceRange) -> (usize, usize) {
    (range.start, range.end)
}

fn write_contexts(output: &mut String, contexts: &BindingContextTable) {
    output.push_str("contexts:\n");
    for (_, context) in contexts.iter() {
        let _ = write!(output, "  context#{} owner=", context.id.index());
        write_context_owner(output, &context.owner);
        output.push_str(" parent=");
        write_optional_context_id(output, context.parent);
        output.push_str(" layer=");
        output.push_str(context_layer_name(context.layer));
        output.push_str(" scope=");
        write_optional_scope(output, context.lexical_scope.as_ref());
        output.push_str(" bindings=");
        write_binding_ids(output, &context.bindings);
        output.push_str(" visible=");
        write_binding_ids(output, &context.visible_bindings);
        output.push_str(" recovery=");
        output.push_str(context_recovery_name(context.recovery));
        output.push('\n');
    }
}

fn write_bindings(output: &mut String, bindings: &BindingTable) {
    output.push_str("bindings:\n");
    for (_, binding) in bindings.canonical_iter() {
        let _ = write!(output, "  binding#{} spelling=", binding.id.index());
        write_quoted(output, &binding.spelling);
        output.push_str(" kind=");
        output.push_str(binding_kind_name(binding.kind));
        let _ = write!(output, " owner=context#{}", binding.owner_context.index());
        output.push_str(" identity=");
        write_binder_identity(output, &binding.identity);
        output.push_str(" range=");
        write_source_range(output, binding.declaration_range);
        let _ = write!(
            output,
            " visible_after={} type=",
            binding.visible_after_ordinal
        );
        write_type_site(output, &binding.type_site);
        output.push_str(" status=");
        output.push_str(binding_status_name(binding.status));
        output.push_str(" captured=");
        write_captured(output, &binding.captured);
        output.push_str(" diagnostics=");
        write_diagnostic_ids(output, &binding.diagnostics);
        output.push_str(" recovery=");
        output.push_str(binding_recovery_name(binding.recovery));
        output.push('\n');
    }
}

fn write_diagnostics(output: &mut String, diagnostics: &BindingDiagnosticTable) {
    output.push_str("diagnostics:\n");
    for (_, diagnostic) in diagnostics.canonical_iter() {
        let _ = write!(output, "  diagnostic#{} range=", diagnostic.id.index());
        write_optional_source_range(output, diagnostic.source_range);
        output.push_str(" class=");
        output.push_str(diagnostic_class_name(diagnostic.class));
        output.push_str(" severity=");
        output.push_str(diagnostic_severity_name(diagnostic.severity));
        output.push_str(" key=");
        write_quoted(output, &diagnostic.message_key);
        output.push_str(" recovery=");
        output.push_str(diagnostic_recovery_name(diagnostic.recovery));
        output.push('\n');
    }
}

fn write_module_id(output: &mut impl fmt::Write, module: &ModuleId) {
    let _ = write!(
        output,
        "{}::{}",
        module.package().as_str(),
        module.path().as_str()
    );
}

fn write_context_owner(output: &mut String, owner: &BindingContextOwner) {
    match owner {
        BindingContextOwner::Module => output.push_str("module"),
        BindingContextOwner::DeclarationShell(shell) => {
            let _ = write!(output, "declaration-shell({})", shell.index());
        }
        BindingContextOwner::Generated(key) => {
            output.push_str("generated(");
            write_quoted(output, key);
            output.push(')');
        }
    }
}

fn write_optional_context_id(output: &mut String, context: Option<BindingContextId>) {
    if let Some(context) = context {
        let _ = write!(output, "context#{}", context.index());
    } else {
        output.push_str("none");
    }
}

fn write_scope(output: &mut String, scope: &LocalTermScope) {
    output.push('[');
    for (index, segment) in scope.path().iter().enumerate() {
        if index > 0 {
            output.push('.');
        }
        let _ = write!(output, "{segment}");
    }
    output.push(']');
}

fn write_optional_scope(output: &mut String, scope: Option<&LocalTermScope>) {
    if let Some(scope) = scope {
        write_scope(output, scope);
    } else {
        output.push_str("none");
    }
}

fn write_binding_ids(output: &mut String, bindings: &[BindingId]) {
    output.push('[');
    for (index, binding) in bindings.iter().enumerate() {
        if index > 0 {
            output.push_str(", ");
        }
        let _ = write!(output, "binding#{}", binding.index());
    }
    output.push(']');
}

fn write_diagnostic_ids(output: &mut String, diagnostics: &[BindingDiagnosticId]) {
    output.push('[');
    for (index, diagnostic) in diagnostics.iter().enumerate() {
        if index > 0 {
            output.push_str(", ");
        }
        let _ = write!(output, "diagnostic#{}", diagnostic.index());
    }
    output.push(']');
}

fn write_binder_identity(output: &mut String, identity: &BinderIdentity) {
    match identity {
        BinderIdentity::ResolverLocal {
            scope,
            ordinal,
            declaration_range,
        } => {
            output.push_str("resolver_local(scope=");
            write_scope(output, scope);
            let _ = write!(output, ", ordinal={ordinal}, range=");
            write_source_range(output, *declaration_range);
            output.push(')');
        }
        BinderIdentity::DefinitionShell { symbol, shell } => {
            output.push_str("definition_shell(symbol=");
            write_symbol_id(output, symbol);
            output.push_str(", shell=");
            write_quoted(output, shell.as_str());
            output.push(')');
        }
        BinderIdentity::ReservedVariable {
            spelling,
            declaration_range,
        } => {
            output.push_str("reserved_variable(spelling=");
            write_quoted(output, spelling);
            output.push_str(", range=");
            write_source_range(output, *declaration_range);
            output.push(')');
        }
        BinderIdentity::Generated { context, counter } => {
            let _ = write!(
                output,
                "generated(context#{}, counter={counter})",
                context.index()
            );
        }
    }
}

fn write_type_site(output: &mut String, site: &BindingTypeSite) {
    match site {
        BindingTypeSite::Missing => output.push_str("missing"),
        BindingTypeSite::Source(range) => {
            output.push_str("source(");
            write_source_range(output, *range);
            output.push(')');
        }
        BindingTypeSite::Deferred(key) => {
            output.push_str("deferred(");
            write_quoted(output, key);
            output.push(')');
        }
    }
}

fn write_captured(output: &mut String, captured: &CapturedFreeVariables) {
    output.push('[');
    for (index, identity) in captured.identities().iter().enumerate() {
        if index > 0 {
            output.push_str(", ");
        }
        write_binder_identity(output, identity);
    }
    output.push(']');
}

fn write_symbol_id(output: &mut String, symbol: &SymbolId) {
    output.push_str(symbol.fqn().as_str());
}

fn write_source_range(output: &mut String, range: SourceRange) {
    let _ = write!(output, "{}..{}", range.start, range.end);
}

fn write_optional_source_range(output: &mut String, range: Option<SourceRange>) {
    if let Some(range) = range {
        write_source_range(output, range);
    } else {
        output.push_str("none");
    }
}

fn write_quoted(output: &mut String, value: &str) {
    output.push('"');
    for character in value.chars() {
        match character {
            '\\' => output.push_str("\\\\"),
            '"' => output.push_str("\\\""),
            '\n' => output.push_str("\\n"),
            '\r' => output.push_str("\\r"),
            '\t' => output.push_str("\\t"),
            _ => output.push(character),
        }
    }
    output.push('"');
}

fn context_layer_name(layer: BindingContextLayer) -> &'static str {
    match layer {
        BindingContextLayer::Module => "module",
        BindingContextLayer::Declaration => "declaration",
        BindingContextLayer::Proof => "proof",
        BindingContextLayer::Block => "block",
        BindingContextLayer::Expression => "expression",
    }
}

fn context_recovery_name(recovery: BindingContextRecovery) -> &'static str {
    match recovery {
        BindingContextRecovery::Normal => "normal",
        BindingContextRecovery::Recovered => "recovered",
        BindingContextRecovery::Degraded => "degraded",
    }
}

fn binding_kind_name(kind: BindingKind) -> &'static str {
    match kind {
        BindingKind::QuantifierBinder => "quantifier_binder",
        BindingKind::DefinitionParameter => "definition_parameter",
        BindingKind::LocalAbbreviation => "local_abbreviation",
        BindingKind::ReservedVariable => "reserved_variable",
        BindingKind::LetBinding => "let_binding",
        BindingKind::Generated => "generated",
    }
}

fn binding_status_name(status: BindingStatus) -> &'static str {
    match status {
        BindingStatus::Active => "active",
        BindingStatus::Reserved => "reserved",
        BindingStatus::Degraded => "degraded",
        BindingStatus::Omitted => "omitted",
    }
}

fn binding_recovery_name(recovery: BindingRecoveryState) -> &'static str {
    match recovery {
        BindingRecoveryState::Normal => "normal",
        BindingRecoveryState::Recovered => "recovered",
        BindingRecoveryState::Degraded => "degraded",
    }
}

fn diagnostic_class_name(class: BindingDiagnosticClass) -> &'static str {
    match class {
        BindingDiagnosticClass::DuplicateLocalBinding => "duplicate_local_binding",
        BindingDiagnosticClass::ForwardLocalReference => "forward_local_reference",
        BindingDiagnosticClass::UnsupportedSourceShape => "unsupported_source_shape",
        BindingDiagnosticClass::ExternalDependencyGap => "external_dependency_gap",
        BindingDiagnosticClass::IllegalNestedReserve => "illegal_nested_reserve",
        BindingDiagnosticClass::RecoveredContextBoundary => "recovered_context_boundary",
        BindingDiagnosticClass::AmbiguousLocalBinding => "ambiguous_local_binding",
    }
}

fn diagnostic_severity_name(severity: BindingDiagnosticSeverity) -> &'static str {
    match severity {
        BindingDiagnosticSeverity::Error => "error",
        BindingDiagnosticSeverity::Warning => "warning",
        BindingDiagnosticSeverity::Note => "note",
    }
}

fn diagnostic_recovery_name(recovery: BindingDiagnosticRecovery) -> &'static str {
    match recovery {
        BindingDiagnosticRecovery::Normal => "normal",
        BindingDiagnosticRecovery::Recovery => "recovery",
        BindingDiagnosticRecovery::Degraded => "degraded",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mizar_resolve::resolved_ast::{
        AmbiguousNameRef, BuiltinRef, FullyQualifiedName, LocalSymbolId, SymbolRef,
        UnresolvedNameRef,
    };
    use mizar_session::{
        BuildSnapshotId, InMemorySessionIdAllocator, ModulePath, PackageId, SessionIdAllocator,
    };

    #[test]
    fn module_shell_records_external_gaps_and_debug_rendering_is_stable() {
        let source = source_id();
        let first = BindingEnv::module_shell_from_parts(source, module_id(), module_id()).unwrap();
        let second = BindingEnv::module_shell_from_parts(source, module_id(), module_id()).unwrap();
        let message_keys = first
            .diagnostics()
            .iter()
            .map(|(_, diagnostic)| diagnostic.message_key.as_str())
            .collect::<Vec<_>>();

        assert_eq!(first.debug_text(), second.debug_text());
        assert_eq!(message_keys.len(), 4);
        assert!(first.debug_text().starts_with("binding-env-debug-v1\n"));
        assert!(first.debug_text().contains("context#0 owner=module"));
        assert!(first.debug_text().contains("class=external_dependency_gap"));
        for key in [
            "checker.binding.external.local_bindings",
            "checker.binding.external.use_site_scope",
            "checker.binding.external.reserve_payload",
            "checker.binding.external.closure_payload",
        ] {
            assert!(message_keys.contains(&key));
            assert!(first.debug_text().contains(key));
        }
        assert!(!first.debug_text().contains("SourceId"));
        assert!(!first.debug_text().contains(concat!("V", "cId")));
        assert!(!first.debug_text().contains(concat!("Proof", "Witness")));
        assert!(!first.debug_text().contains("verifier status"));
        assert!(!first.debug_text().contains(concat!("overload", "_root")));
    }

    #[test]
    fn public_module_shell_signature_stays_on_resolver_shell_boundary() {
        fn assert_signature(
            _: fn(&ResolvedAst, &SymbolEnv) -> Result<BindingEnv, BindingEnvError>,
        ) {
        }

        assert_signature(BindingEnv::module_shell);
    }

    #[test]
    fn module_shell_rejects_mismatched_symbol_environment_module() {
        let result = BindingEnv::module_shell_from_parts(
            source_id(),
            module_id(),
            ModuleId::new(PackageId::new("other"), ModulePath::new("main")),
        );
        assert!(matches!(
            result,
            Err(BindingEnvError::ModuleMismatch { .. })
        ));
    }

    #[test]
    fn context_layers_and_validation_cover_parent_chain_and_recovery() {
        let source = source_id();
        let mut contexts = BindingContextTable::new();
        let module = contexts.insert(context_draft(
            BindingContextOwner::Module,
            None,
            BindingContextLayer::Module,
            None,
        ));
        let declaration = contexts.insert(context_draft(
            BindingContextOwner::Generated("definition".to_owned()),
            Some(module),
            BindingContextLayer::Declaration,
            Some(LocalTermScope::new(vec![0])),
        ));
        let proof = contexts.insert(context_draft(
            BindingContextOwner::Generated("proof".to_owned()),
            Some(declaration),
            BindingContextLayer::Proof,
            Some(LocalTermScope::new(vec![0, 0])),
        ));
        let block = contexts.insert(context_draft(
            BindingContextOwner::Generated("block".to_owned()),
            Some(proof),
            BindingContextLayer::Block,
            Some(LocalTermScope::new(vec![0, 0, 0])),
        ));
        let expression = contexts.insert(context_draft(
            BindingContextOwner::Generated("expression".to_owned()),
            Some(block),
            BindingContextLayer::Expression,
            Some(LocalTermScope::new(vec![0, 0, 0, 0])),
        ));
        contexts.contexts[expression.index()].recovery = BindingContextRecovery::Recovered;

        let env = BindingEnv::try_new(parts_with(source, contexts, BindingTable::new()))
            .expect("layered contexts validate");
        let debug = env.debug_text();
        for expected in [
            "layer=module",
            "layer=declaration",
            "layer=proof",
            "layer=block",
            "layer=expression",
            "recovery=recovered",
        ] {
            assert!(debug.contains(expected), "{expected}\n{debug}");
        }

        let mut missing_parent = BindingContextTable::new();
        missing_parent.insert(context_draft(
            BindingContextOwner::Module,
            None,
            BindingContextLayer::Module,
            None,
        ));
        let context = missing_parent.insert(context_draft(
            BindingContextOwner::Generated("missing-parent".to_owned()),
            Some(BindingContextId::new(99)),
            BindingContextLayer::Block,
            None,
        ));
        assert!(matches!(
            BindingEnv::try_new(parts_with(
                source,
                missing_parent,
                BindingTable::new(),
            )),
            Err(BindingEnvError::InvalidContextParent { context: failed, parent })
                if failed == context && parent == BindingContextId::new(99)
        ));

        let mut cyclic = BindingContextTable::new();
        cyclic.insert(context_draft(
            BindingContextOwner::Module,
            None,
            BindingContextLayer::Module,
            None,
        ));
        let first = cyclic.insert(context_draft(
            BindingContextOwner::Generated("a".to_owned()),
            Some(BindingContextId::new(2)),
            BindingContextLayer::Block,
            None,
        ));
        cyclic.insert(context_draft(
            BindingContextOwner::Generated("b".to_owned()),
            Some(first),
            BindingContextLayer::Block,
            None,
        ));
        assert!(matches!(
            BindingEnv::try_new(parts_with(source, cyclic, BindingTable::new())),
            Err(BindingEnvError::ContextCycle { context }) if context == first
        ));
    }

    #[test]
    fn lookup_uses_deepest_scope_shadowing_and_blocks_forward_reference() {
        let source = source_id();
        let mut contexts = BindingContextTable::new();
        let root = contexts.insert(context_draft(
            BindingContextOwner::Module,
            None,
            BindingContextLayer::Module,
            Some(LocalTermScope::new(vec![0])),
        ));
        let child = contexts.insert(context_draft(
            BindingContextOwner::Generated("block".to_owned()),
            Some(root),
            BindingContextLayer::Block,
            Some(LocalTermScope::new(vec![0, 1])),
        ));
        let mut bindings = BindingTable::new();
        let outer = bindings.insert(local_binding(root, "x", vec![0], range(source, 0, 1), 1));
        let inner = bindings.insert(local_binding(
            child,
            "x",
            vec![0, 1],
            range(source, 10, 11),
            3,
        ));
        contexts.contexts[root.index()].bindings = vec![outer];
        contexts.contexts[root.index()].visible_bindings = vec![outer];
        contexts.contexts[child.index()].bindings = vec![inner];
        contexts.contexts[child.index()].visible_bindings = vec![outer, inner];
        let env = BindingEnv::try_new(parts_with(source, contexts, bindings)).unwrap();

        let lookup = env.lookup(&BindingLookupSite::new(
            "x",
            child,
            Some(LocalTermScope::new(vec![0, 1, 2])),
            4,
        ));
        assert!(matches!(lookup, Ok(BindingLookupResult::Local(id)) if id == inner));

        let forward = env.lookup(&BindingLookupSite::new(
            "x",
            child,
            Some(LocalTermScope::new(vec![0, 1, 2])),
            3,
        ));
        assert!(matches!(
            forward,
            Ok(BindingLookupResult::Local(id)) if id == outer
        ));

        let before_outer = env.lookup(&BindingLookupSite::new(
            "x",
            child,
            Some(LocalTermScope::new(vec![0, 1, 2])),
            1,
        ));
        assert!(matches!(
            before_outer,
            Ok(BindingLookupResult::ForwardReference { candidates, diagnostic })
                if candidates == vec![outer, inner]
                    && diagnostic.class == BindingDiagnosticClass::ForwardLocalReference
                    && diagnostic.recovery == BindingDiagnosticRecovery::Degraded
        ));
    }

    #[test]
    fn lookup_handles_ambiguity_missing_payload_and_visible_boundaries() {
        let source = source_id();
        let mut contexts = BindingContextTable::new();
        let context = contexts.insert(context_draft(
            BindingContextOwner::Module,
            None,
            BindingContextLayer::Module,
            Some(LocalTermScope::new(vec![0])),
        ));
        let mut bindings = BindingTable::new();
        let first = bindings.insert(local_binding(context, "x", vec![0], range(source, 0, 1), 1));
        let second = bindings.insert(local_binding(context, "x", vec![0], range(source, 0, 1), 1));
        contexts.contexts[context.index()].bindings = vec![first, second];
        contexts.contexts[context.index()].visible_bindings = vec![first, second];
        let env = BindingEnv::try_new(parts_with(source, contexts, bindings)).unwrap();
        let ambiguous = env.lookup(&BindingLookupSite::new(
            "x",
            context,
            Some(LocalTermScope::new(vec![0, 1])),
            2,
        ));
        assert!(matches!(
            ambiguous,
            Ok(BindingLookupResult::Ambiguous { candidates, diagnostic })
                if candidates == vec![first, second]
                    && diagnostic.class == BindingDiagnosticClass::AmbiguousLocalBinding
                    && diagnostic.message_key == "checker.binding.ambiguous_local"
        ));

        let missing_payload = env.lookup(&BindingLookupSite::new("x", context, None, 2));
        assert!(matches!(
            missing_payload,
            Ok(BindingLookupResult::MissingExternalPayload { diagnostic })
                if diagnostic.class == BindingDiagnosticClass::ExternalDependencyGap
                    && diagnostic.message_key == "checker.binding.external.use_site_scope"
        ));
        let resolver_resolution =
            NameResolution::Resolved(SymbolRef::new(symbol_id(), range(source, 20, 21)));
        let resolver_fallback = env.lookup(
            &BindingLookupSite::new("x", context, None, 2)
                .with_resolver_resolution(resolver_resolution.clone()),
        );
        assert!(matches!(
            resolver_fallback,
            Ok(BindingLookupResult::Resolver(resolution)) if resolution == resolver_resolution
        ));

        let mut hidden_contexts = BindingContextTable::new();
        let hidden_context = hidden_contexts.insert(context_draft(
            BindingContextOwner::Module,
            None,
            BindingContextLayer::Module,
            Some(LocalTermScope::new(vec![0])),
        ));
        let mut hidden_bindings = BindingTable::new();
        let hidden = hidden_bindings.insert(local_binding(
            hidden_context,
            "x",
            vec![0],
            range(source, 0, 1),
            1,
        ));
        hidden_contexts.contexts[hidden_context.index()].bindings = vec![hidden];
        let hidden_env =
            BindingEnv::try_new(parts_with(source, hidden_contexts, hidden_bindings)).unwrap();
        assert!(matches!(
            hidden_env.lookup(&BindingLookupSite::new(
                "x",
                hidden_context,
                Some(LocalTermScope::new(vec![0])),
                2,
            )),
            Ok(BindingLookupResult::Unresolved)
        ));

        let mut shadow_contexts = BindingContextTable::new();
        let shadow_context = shadow_contexts.insert(context_draft(
            BindingContextOwner::Module,
            None,
            BindingContextLayer::Module,
            Some(LocalTermScope::new(vec![0])),
        ));
        let mut shadow_bindings = BindingTable::new();
        let reserved = shadow_bindings.insert(reserved_binding(
            shadow_context,
            "x",
            range(source, 0, 1),
            1,
        ));
        let local = shadow_bindings.insert(local_binding(
            shadow_context,
            "x",
            vec![0, 1],
            range(source, 2, 3),
            1,
        ));
        shadow_contexts.contexts[shadow_context.index()].bindings = vec![reserved, local];
        shadow_contexts.contexts[shadow_context.index()].visible_bindings = vec![reserved, local];
        let shadow_env =
            BindingEnv::try_new(parts_with(source, shadow_contexts, shadow_bindings)).unwrap();
        assert!(matches!(
            shadow_env.lookup(&BindingLookupSite::new("x", shadow_context, None, 2)),
            Ok(BindingLookupResult::MissingExternalPayload { diagnostic })
                if diagnostic.message_key == "checker.binding.external.use_site_scope"
        ));

        let mut omitted_contexts = BindingContextTable::new();
        let parent = omitted_contexts.insert(context_draft(
            BindingContextOwner::Module,
            None,
            BindingContextLayer::Module,
            Some(LocalTermScope::new(vec![0])),
        ));
        let child = omitted_contexts.insert(context_draft(
            BindingContextOwner::Generated("recovered-child".to_owned()),
            Some(parent),
            BindingContextLayer::Block,
            Some(LocalTermScope::new(vec![0, 1])),
        ));
        let mut omitted_bindings = BindingTable::new();
        let omitted =
            omitted_bindings.insert(local_binding(parent, "x", vec![0], range(source, 0, 1), 1));
        omitted_contexts.contexts[parent.index()].bindings = vec![omitted];
        omitted_contexts.contexts[parent.index()].visible_bindings = vec![omitted];
        let omitted_env =
            BindingEnv::try_new(parts_with(source, omitted_contexts, omitted_bindings)).unwrap();
        assert!(matches!(
            omitted_env.lookup(&BindingLookupSite::new(
                "x",
                child,
                Some(LocalTermScope::new(vec![0, 1])),
                2,
            )),
            Ok(BindingLookupResult::Unresolved)
        ));
    }

    #[test]
    fn lookup_tie_breaks_same_scope_by_later_declaration_range() {
        let source = source_id();
        let mut contexts = BindingContextTable::new();
        let context = contexts.insert(context_draft(
            BindingContextOwner::Module,
            None,
            BindingContextLayer::Module,
            Some(LocalTermScope::new(vec![0])),
        ));
        let mut bindings = BindingTable::new();
        let earlier = bindings.insert(local_binding(context, "x", vec![0], range(source, 0, 1), 1));
        let later = bindings.insert(local_binding(
            context,
            "x",
            vec![0],
            range(source, 10, 11),
            1,
        ));
        contexts.contexts[context.index()].bindings = vec![earlier, later];
        contexts.contexts[context.index()].visible_bindings = vec![earlier, later];
        let env = BindingEnv::try_new(parts_with(source, contexts, bindings)).unwrap();

        assert!(matches!(
            env.lookup(&BindingLookupSite::new(
                "x",
                context,
                Some(LocalTermScope::new(vec![0])),
                2,
            )),
            Ok(BindingLookupResult::Local(id)) if id == later
        ));
    }

    #[test]
    fn lookup_falls_back_to_resolver_resolution_without_global_lookup() {
        let source = source_id();
        let mut contexts = BindingContextTable::new();
        let context = contexts.insert(context_draft(
            BindingContextOwner::Module,
            None,
            BindingContextLayer::Module,
            None,
        ));
        let env = BindingEnv::try_new(parts_with(source, contexts, BindingTable::new())).unwrap();

        for resolution in [
            NameResolution::Resolved(SymbolRef::new(symbol_id(), range(source, 0, 1))),
            NameResolution::ResolvedBuiltin(BuiltinRef::new(
                mizar_resolve::resolved_ast::BuiltinId::new("set"),
                range(source, 0, 1),
                "set",
            )),
            NameResolution::Ambiguous(AmbiguousNameRef::new("f", range(source, 0, 1), Vec::new())),
            NameResolution::Unresolved(UnresolvedNameRef::new(
                "f",
                range(source, 0, 1),
                mizar_resolve::resolved_ast::NameLookupClass::Symbol,
            )),
        ] {
            let result = env.lookup(
                &BindingLookupSite::new("f", context, None, 0)
                    .with_resolver_resolution(resolution.clone()),
            );

            assert!(matches!(
                result,
                Ok(BindingLookupResult::Resolver(actual)) if actual == resolution
            ));
        }
    }

    #[test]
    fn reserved_variables_are_visible_and_local_binders_shadow_them() {
        let source = source_id();
        let mut contexts = BindingContextTable::new();
        let module = contexts.insert(context_draft(
            BindingContextOwner::Module,
            None,
            BindingContextLayer::Module,
            Some(LocalTermScope::new(vec![0])),
        ));
        let block = contexts.insert(context_draft(
            BindingContextOwner::Generated("block".to_owned()),
            Some(module),
            BindingContextLayer::Block,
            Some(LocalTermScope::new(vec![0, 1])),
        ));
        let mut bindings = BindingTable::new();
        let reserved = bindings.insert(reserved_binding(module, "x", range(source, 0, 1), 1));
        let local = bindings.insert(local_binding(
            block,
            "x",
            vec![0, 1],
            range(source, 5, 6),
            2,
        ));
        contexts.contexts[module.index()].bindings = vec![reserved];
        contexts.contexts[module.index()].visible_bindings = vec![reserved];
        contexts.contexts[block.index()].bindings = vec![local];
        contexts.contexts[block.index()].visible_bindings = vec![reserved, local];
        let env = BindingEnv::try_new(parts_with(source, contexts, bindings)).unwrap();

        assert!(matches!(
            env.lookup(&BindingLookupSite::new(
                "x",
                module,
                Some(LocalTermScope::new(vec![0])),
                2,
            )),
            Ok(BindingLookupResult::Local(id)) if id == reserved
        ));
        assert!(matches!(
            env.lookup(&BindingLookupSite::new(
                "x",
                block,
                Some(LocalTermScope::new(vec![0, 1, 0])),
                3,
            )),
            Ok(BindingLookupResult::Local(id)) if id == local
        ));
    }

    #[test]
    fn binder_identity_and_closure_metadata_are_stable() {
        let source = source_id();
        let identity = BinderIdentity::ResolverLocal {
            scope: LocalTermScope::new(vec![0]),
            ordinal: 1,
            declaration_range: range(source, 0, 1),
        };
        let first = BindingDraft {
            spelling: "x".to_owned(),
            identity: identity.clone(),
            ..binding_draft_defaults(BindingContextId::new(0), range(source, 0, 1))
        };
        let second = BindingDraft {
            spelling: "renamed".to_owned(),
            identity: identity.clone(),
            ..binding_draft_defaults(BindingContextId::new(0), range(source, 0, 1))
        };
        assert_eq!(first.identity, second.identity);
        assert_ne!(first.spelling, second.spelling);

        let captured_standalone = CapturedFreeVariables::new(vec![
            BinderIdentity::Generated {
                context: BindingContextId::new(2),
                counter: 0,
            },
            identity.clone(),
            identity,
        ]);
        assert_eq!(captured_standalone.identities().len(), 2);

        let mut contexts = BindingContextTable::new();
        let context = contexts.insert(context_draft(
            BindingContextOwner::Module,
            None,
            BindingContextLayer::Module,
            Some(LocalTermScope::new(vec![0])),
        ));
        let mut diagnostics = BindingDiagnosticTable::new();
        let gap = diagnostics.insert(BindingDiagnosticDraft {
            source_range: None,
            class: BindingDiagnosticClass::ExternalDependencyGap,
            severity: BindingDiagnosticSeverity::Note,
            message_key: "checker.binding.external.closure_payload".to_owned(),
            recovery: BindingDiagnosticRecovery::Degraded,
        });
        let mut bindings = BindingTable::new();
        let abbreviation = bindings.insert(BindingDraft {
            kind: BindingKind::LocalAbbreviation,
            captured: CapturedFreeVariables::new(vec![BinderIdentity::Generated {
                context,
                counter: 0,
            }]),
            diagnostics: vec![gap],
            status: BindingStatus::Degraded,
            recovery: BindingRecoveryState::Degraded,
            ..binding_draft_defaults(context, range(source, 0, 1))
        });
        contexts.contexts[context.index()].bindings = vec![abbreviation];
        contexts.contexts[context.index()].visible_bindings = vec![abbreviation];
        let mut parts = parts_with(source, contexts, bindings);
        parts.diagnostics = diagnostics;
        let debug = BindingEnv::try_new(parts).unwrap().debug_text();
        assert!(debug.contains("kind=local_abbreviation"));
        assert!(debug.contains("class=external_dependency_gap"));
        assert!(debug.contains("status=degraded"));

        let mut shell_contexts = BindingContextTable::new();
        let shell_context = shell_contexts.insert(context_draft(
            BindingContextOwner::Module,
            None,
            BindingContextLayer::Module,
            None,
        ));
        let mut shell_bindings = BindingTable::new();
        let shell_binding = shell_bindings.insert(BindingDraft {
            kind: BindingKind::DefinitionParameter,
            identity: BinderIdentity::DefinitionShell {
                symbol: symbol_id(),
                shell: ResolverShellId::new("definition-shell#0"),
            },
            ..binding_draft_defaults(shell_context, range(source, 2, 3))
        });
        shell_contexts.contexts[shell_context.index()].bindings = vec![shell_binding];
        shell_contexts.contexts[shell_context.index()].visible_bindings = vec![shell_binding];
        let shell_debug = BindingEnv::try_new(parts_with(source, shell_contexts, shell_bindings))
            .unwrap()
            .debug_text();
        assert!(shell_debug.contains("identity=definition_shell(symbol="));
        assert!(shell_debug.contains("shell=\"definition-shell#0\""));
    }

    #[test]
    fn diagnostics_cover_duplicate_unsupported_and_external_gap_states() {
        let source = source_id();
        let mut diagnostics = BindingDiagnosticTable::new();
        let duplicate = diagnostics.insert(BindingDiagnosticDraft {
            source_range: Some(range(source, 0, 1)),
            class: BindingDiagnosticClass::DuplicateLocalBinding,
            severity: BindingDiagnosticSeverity::Error,
            message_key: "checker.binding.duplicate".to_owned(),
            recovery: BindingDiagnosticRecovery::Normal,
        });
        let unsupported = diagnostics.insert(BindingDiagnosticDraft {
            source_range: Some(range(source, 2, 3)),
            class: BindingDiagnosticClass::UnsupportedSourceShape,
            severity: BindingDiagnosticSeverity::Warning,
            message_key: "checker.binding.unsupported_shape".to_owned(),
            recovery: BindingDiagnosticRecovery::Recovery,
        });
        let gap = diagnostics.insert(BindingDiagnosticDraft {
            source_range: None,
            class: BindingDiagnosticClass::ExternalDependencyGap,
            severity: BindingDiagnosticSeverity::Note,
            message_key: "checker.binding.external.closure_payload".to_owned(),
            recovery: BindingDiagnosticRecovery::Degraded,
        });
        let illegal_reserve = diagnostics.insert(BindingDiagnosticDraft {
            source_range: Some(range(source, 4, 5)),
            class: BindingDiagnosticClass::IllegalNestedReserve,
            severity: BindingDiagnosticSeverity::Error,
            message_key: "checker.binding.illegal_nested_reserve".to_owned(),
            recovery: BindingDiagnosticRecovery::Normal,
        });
        let recovered_boundary = diagnostics.insert(BindingDiagnosticDraft {
            source_range: Some(range(source, 6, 7)),
            class: BindingDiagnosticClass::RecoveredContextBoundary,
            severity: BindingDiagnosticSeverity::Warning,
            message_key: "checker.binding.recovered_context_boundary".to_owned(),
            recovery: BindingDiagnosticRecovery::Recovery,
        });
        assert_eq!(duplicate.index(), 0);
        assert_eq!(unsupported.index(), 1);
        assert_eq!(gap.index(), 2);
        assert_eq!(illegal_reserve.index(), 3);
        assert_eq!(recovered_boundary.index(), 4);

        let mut contexts = BindingContextTable::new();
        let context = contexts.insert(context_draft(
            BindingContextOwner::Module,
            None,
            BindingContextLayer::Module,
            None,
        ));
        let mut bindings = BindingTable::new();
        let binding = bindings.insert(BindingDraft {
            diagnostics: vec![unsupported, duplicate, duplicate],
            recovery: BindingRecoveryState::Degraded,
            status: BindingStatus::Degraded,
            ..binding_draft_defaults(context, range(source, 0, 1))
        });
        contexts.contexts[context.index()].bindings = vec![binding];
        let mut env_parts = parts_with(source, contexts, bindings);
        env_parts.diagnostics = diagnostics;
        let env = BindingEnv::try_new(env_parts).unwrap();
        let binding = env.bindings().get(binding).unwrap();
        assert_eq!(binding.diagnostics, vec![duplicate, unsupported]);
        assert!(env.debug_text().contains("class=unsupported_source_shape"));
        assert!(env.debug_text().contains("class=external_dependency_gap"));
        assert!(env.debug_text().contains("class=illegal_nested_reserve"));
        assert!(
            env.debug_text()
                .contains("class=recovered_context_boundary")
        );

        let mut duplicate_contexts = BindingContextTable::new();
        let duplicate_context = duplicate_contexts.insert(context_draft(
            BindingContextOwner::Module,
            None,
            BindingContextLayer::Module,
            Some(LocalTermScope::new(vec![0])),
        ));
        let mut duplicate_bindings = BindingTable::new();
        let first = duplicate_bindings.insert(BindingDraft {
            diagnostics: vec![duplicate],
            recovery: BindingRecoveryState::Degraded,
            ..local_binding(duplicate_context, "x", vec![0], range(source, 0, 1), 1)
        });
        let second = duplicate_bindings.insert(BindingDraft {
            diagnostics: vec![duplicate],
            recovery: BindingRecoveryState::Degraded,
            ..local_binding(duplicate_context, "x", vec![0], range(source, 0, 1), 1)
        });
        duplicate_contexts.contexts[duplicate_context.index()].bindings = vec![first, second];
        duplicate_contexts.contexts[duplicate_context.index()].visible_bindings =
            vec![first, second];
        let mut duplicate_parts = parts_with(source, duplicate_contexts, duplicate_bindings);
        duplicate_parts.diagnostics = BindingDiagnosticTable {
            diagnostics: env.diagnostics().diagnostics.clone(),
        };
        let duplicate_env = BindingEnv::try_new(duplicate_parts).unwrap();
        for binding in duplicate_env.bindings().iter().map(|(_, binding)| binding) {
            assert_eq!(binding.diagnostics, vec![duplicate]);
        }

        let mut ordered_diagnostics = BindingDiagnosticTable::new();
        let later = ordered_diagnostics.insert(BindingDiagnosticDraft {
            source_range: Some(range(source, 10, 11)),
            class: BindingDiagnosticClass::UnsupportedSourceShape,
            severity: BindingDiagnosticSeverity::Warning,
            message_key: "checker.binding.later".to_owned(),
            recovery: BindingDiagnosticRecovery::Recovery,
        });
        let earlier = ordered_diagnostics.insert(BindingDiagnosticDraft {
            source_range: Some(range(source, 0, 1)),
            class: BindingDiagnosticClass::ExternalDependencyGap,
            severity: BindingDiagnosticSeverity::Note,
            message_key: "checker.binding.earlier".to_owned(),
            recovery: BindingDiagnosticRecovery::Degraded,
        });
        assert_eq!(
            ordered_diagnostics
                .canonical_iter()
                .map(|(id, _)| id)
                .collect::<Vec<_>>(),
            vec![earlier, later]
        );
        let mut ordered_contexts = BindingContextTable::new();
        ordered_contexts.insert(context_draft(
            BindingContextOwner::Module,
            None,
            BindingContextLayer::Module,
            None,
        ));
        let mut ordered_parts = parts_with(source, ordered_contexts, BindingTable::new());
        ordered_parts.diagnostics = ordered_diagnostics;
        let debug = BindingEnv::try_new(ordered_parts).unwrap().debug_text();
        let earlier_offset = debug.find("checker.binding.earlier").unwrap();
        let later_offset = debug.find("checker.binding.later").unwrap();
        assert!(earlier_offset < later_offset, "{debug}");
    }

    #[test]
    fn validation_rejects_invalid_ranges_and_diagnostic_links() {
        let (source, other_source) = distinct_source_ids();
        let mut contexts = BindingContextTable::new();
        let context = contexts.insert(context_draft(
            BindingContextOwner::Module,
            None,
            BindingContextLayer::Module,
            None,
        ));
        let mut bindings = BindingTable::new();
        let binding = bindings.insert(binding_draft_defaults(context, range(other_source, 0, 1)));
        contexts.contexts[context.index()].bindings = vec![binding];
        assert!(matches!(
            BindingEnv::try_new(parts_with(source, contexts.clone(), bindings)),
            Err(BindingEnvError::InvalidBindingRange { binding: failed }) if failed == binding
        ));

        let mut bindings = BindingTable::new();
        let binding = bindings.insert(BindingDraft {
            diagnostics: vec![BindingDiagnosticId::new(9)],
            ..binding_draft_defaults(context, range(source, 0, 1))
        });
        contexts.contexts[context.index()].bindings = vec![binding];
        assert!(matches!(
            BindingEnv::try_new(parts_with(source, contexts.clone(), bindings)),
            Err(BindingEnvError::InvalidBindingDiagnostic { binding: failed, diagnostic })
                if failed == binding && diagnostic == BindingDiagnosticId::new(9)
        ));

        let mut visibility_contexts = BindingContextTable::new();
        let root = visibility_contexts.insert(context_draft(
            BindingContextOwner::Module,
            None,
            BindingContextLayer::Module,
            None,
        ));
        let left = visibility_contexts.insert(context_draft(
            BindingContextOwner::Generated("left".to_owned()),
            Some(root),
            BindingContextLayer::Block,
            None,
        ));
        let right = visibility_contexts.insert(context_draft(
            BindingContextOwner::Generated("right".to_owned()),
            Some(root),
            BindingContextLayer::Block,
            None,
        ));
        let mut visibility_bindings = BindingTable::new();
        let sibling = visibility_bindings.insert(binding_draft_defaults(left, range(source, 0, 1)));
        visibility_contexts.contexts[left.index()].bindings = vec![sibling];
        visibility_contexts.contexts[right.index()].visible_bindings = vec![sibling];
        assert!(matches!(
            BindingEnv::try_new(parts_with(
                source,
                visibility_contexts,
                visibility_bindings,
            )),
            Err(BindingEnvError::InvalidVisibleBindingOwner {
                context,
                binding,
                owner
            }) if context == right && binding == sibling && owner == left
        ));

        let mut orphan_contexts = BindingContextTable::new();
        let orphan_context = orphan_contexts.insert(context_draft(
            BindingContextOwner::Module,
            None,
            BindingContextLayer::Module,
            Some(LocalTermScope::new(vec![0])),
        ));
        let mut orphan_bindings = BindingTable::new();
        let orphan = orphan_bindings.insert(local_binding(
            orphan_context,
            "x",
            vec![0],
            range(source, 0, 1),
            1,
        ));
        orphan_contexts.contexts[orphan_context.index()].visible_bindings = vec![orphan];
        assert!(matches!(
            BindingEnv::try_new(parts_with(source, orphan_contexts, orphan_bindings)),
            Err(BindingEnvError::InvalidVisibleBindingOwner {
                context,
                binding,
                owner
            }) if context == orphan_context && binding == orphan && owner == orphan_context
        ));

        let mut reserved_contexts = BindingContextTable::new();
        let root = reserved_contexts.insert(context_draft(
            BindingContextOwner::Module,
            None,
            BindingContextLayer::Module,
            None,
        ));
        let child = reserved_contexts.insert(context_draft(
            BindingContextOwner::Generated("child".to_owned()),
            Some(root),
            BindingContextLayer::Block,
            None,
        ));
        let mut reserved_bindings = BindingTable::new();
        let reserved =
            reserved_bindings.insert(reserved_binding(child, "x", range(source, 0, 1), 1));
        reserved_contexts.contexts[child.index()].bindings = vec![reserved];
        assert!(matches!(
            BindingEnv::try_new(parts_with(source, reserved_contexts, reserved_bindings)),
            Err(BindingEnvError::InvalidReservedBindingOwner { binding, owner })
                if binding == reserved && owner == child
        ));

        let empty_parts = parts_with(source, BindingContextTable::new(), BindingTable::new());
        assert!(matches!(
            BindingEnv::try_new(empty_parts),
            Err(BindingEnvError::MissingModuleContext)
        ));

        let mut generated_root_contexts = BindingContextTable::new();
        generated_root_contexts.insert(context_draft(
            BindingContextOwner::Generated("not-module".to_owned()),
            None,
            BindingContextLayer::Module,
            None,
        ));
        assert!(matches!(
            BindingEnv::try_new(parts_with(
                source,
                generated_root_contexts,
                BindingTable::new(),
            )),
            Err(BindingEnvError::InvalidModuleContext { context })
                if context == BindingContextId::new(0)
        ));

        let mut multiple_root_contexts = BindingContextTable::new();
        multiple_root_contexts.insert(context_draft(
            BindingContextOwner::Module,
            None,
            BindingContextLayer::Module,
            None,
        ));
        let extra_root = multiple_root_contexts.insert(context_draft(
            BindingContextOwner::Generated("extra-root".to_owned()),
            None,
            BindingContextLayer::Block,
            None,
        ));
        assert!(matches!(
            BindingEnv::try_new(parts_with(
                source,
                multiple_root_contexts,
                BindingTable::new(),
            )),
            Err(BindingEnvError::MultipleRootContexts { context }) if context == extra_root
        ));

        let mut missing_owner_contexts = BindingContextTable::new();
        let root = missing_owner_contexts.insert(context_draft(
            BindingContextOwner::Module,
            None,
            BindingContextLayer::Module,
            Some(LocalTermScope::new(vec![0])),
        ));
        let mut missing_owner_bindings = BindingTable::new();
        let missing_owner = missing_owner_bindings.insert(local_binding(
            BindingContextId::new(99),
            "x",
            vec![0],
            range(source, 0, 1),
            1,
        ));
        missing_owner_contexts.contexts[root.index()].visible_bindings = vec![missing_owner];
        assert!(matches!(
            BindingEnv::try_new(parts_with(
                source,
                missing_owner_contexts,
                missing_owner_bindings,
            )),
            Err(BindingEnvError::InvalidBindingOwner { binding, owner })
                if binding == missing_owner && owner == BindingContextId::new(99)
        ));

        let mut inconsistent_contexts = BindingContextTable::new();
        let inconsistent_context = inconsistent_contexts.insert(context_draft(
            BindingContextOwner::Module,
            None,
            BindingContextLayer::Module,
            Some(LocalTermScope::new(vec![0])),
        ));
        let mut inconsistent_bindings = BindingTable::new();
        let inconsistent = inconsistent_bindings.insert(BindingDraft {
            identity: BinderIdentity::ResolverLocal {
                scope: LocalTermScope::new(vec![0]),
                ordinal: 99,
                declaration_range: range(source, 0, 1),
            },
            ..local_binding(inconsistent_context, "x", vec![0], range(source, 0, 1), 1)
        });
        inconsistent_contexts.contexts[inconsistent_context.index()].bindings = vec![inconsistent];
        assert!(matches!(
            BindingEnv::try_new(parts_with(
                source,
                inconsistent_contexts,
                inconsistent_bindings,
            )),
            Err(BindingEnvError::InconsistentResolverLocalIdentity { binding })
                if binding == inconsistent
        ));

        let mut inconsistent_range_contexts = BindingContextTable::new();
        let inconsistent_range_context = inconsistent_range_contexts.insert(context_draft(
            BindingContextOwner::Module,
            None,
            BindingContextLayer::Module,
            Some(LocalTermScope::new(vec![0])),
        ));
        let mut inconsistent_range_bindings = BindingTable::new();
        let inconsistent_range = inconsistent_range_bindings.insert(BindingDraft {
            identity: BinderIdentity::ResolverLocal {
                scope: LocalTermScope::new(vec![0]),
                ordinal: 1,
                declaration_range: range(source, 10, 11),
            },
            ..local_binding(
                inconsistent_range_context,
                "x",
                vec![0],
                range(source, 0, 1),
                1,
            )
        });
        inconsistent_range_contexts.contexts[inconsistent_range_context.index()].bindings =
            vec![inconsistent_range];
        assert!(matches!(
            BindingEnv::try_new(parts_with(
                source,
                inconsistent_range_contexts,
                inconsistent_range_bindings,
            )),
            Err(BindingEnvError::InconsistentResolverLocalIdentity { binding })
                if binding == inconsistent_range
        ));

        let mut inconsistent_reserved_contexts = BindingContextTable::new();
        let inconsistent_reserved_context = inconsistent_reserved_contexts.insert(context_draft(
            BindingContextOwner::Module,
            None,
            BindingContextLayer::Module,
            None,
        ));
        let mut inconsistent_reserved_bindings = BindingTable::new();
        let inconsistent_reserved = inconsistent_reserved_bindings.insert(BindingDraft {
            identity: BinderIdentity::ReservedVariable {
                spelling: "other".to_owned(),
                declaration_range: range(source, 0, 1),
            },
            ..reserved_binding(inconsistent_reserved_context, "x", range(source, 0, 1), 1)
        });
        inconsistent_reserved_contexts.contexts[inconsistent_reserved_context.index()].bindings =
            vec![inconsistent_reserved];
        assert!(matches!(
            BindingEnv::try_new(parts_with(
                source,
                inconsistent_reserved_contexts,
                inconsistent_reserved_bindings,
            )),
            Err(BindingEnvError::InconsistentReservedVariableIdentity { binding })
                if binding == inconsistent_reserved
        ));

        let mut inconsistent_reserved_range_contexts = BindingContextTable::new();
        let inconsistent_reserved_range_context =
            inconsistent_reserved_range_contexts.insert(context_draft(
                BindingContextOwner::Module,
                None,
                BindingContextLayer::Module,
                None,
            ));
        let mut inconsistent_reserved_range_bindings = BindingTable::new();
        let inconsistent_reserved_range =
            inconsistent_reserved_range_bindings.insert(BindingDraft {
                identity: BinderIdentity::ReservedVariable {
                    spelling: "x".to_owned(),
                    declaration_range: range(source, 10, 11),
                },
                ..reserved_binding(
                    inconsistent_reserved_range_context,
                    "x",
                    range(source, 0, 1),
                    1,
                )
            });
        inconsistent_reserved_range_contexts.contexts
            [inconsistent_reserved_range_context.index()]
        .bindings = vec![inconsistent_reserved_range];
        assert!(matches!(
            BindingEnv::try_new(parts_with(
                source,
                inconsistent_reserved_range_contexts,
                inconsistent_reserved_range_bindings,
            )),
            Err(BindingEnvError::InconsistentReservedVariableIdentity { binding })
                if binding == inconsistent_reserved_range
        ));

        let mut reserved_kind_contexts = BindingContextTable::new();
        let reserved_kind_context = reserved_kind_contexts.insert(context_draft(
            BindingContextOwner::Module,
            None,
            BindingContextLayer::Module,
            None,
        ));
        let mut reserved_kind_bindings = BindingTable::new();
        let reserved_kind = reserved_kind_bindings.insert(BindingDraft {
            kind: BindingKind::QuantifierBinder,
            identity: BinderIdentity::ReservedVariable {
                spelling: "x".to_owned(),
                declaration_range: range(source, 0, 1),
            },
            ..reserved_binding(reserved_kind_context, "x", range(source, 0, 1), 1)
        });
        reserved_kind_contexts.contexts[reserved_kind_context.index()].bindings =
            vec![reserved_kind];
        assert!(matches!(
            BindingEnv::try_new(parts_with(
                source,
                reserved_kind_contexts,
                reserved_kind_bindings,
            )),
            Err(BindingEnvError::InvalidBindingKindIdentity { binding })
                if binding == reserved_kind
        ));

        let mut generated_kind_contexts = BindingContextTable::new();
        let generated_kind_context = generated_kind_contexts.insert(context_draft(
            BindingContextOwner::Module,
            None,
            BindingContextLayer::Module,
            None,
        ));
        let mut generated_kind_bindings = BindingTable::new();
        let generated_kind = generated_kind_bindings.insert(BindingDraft {
            kind: BindingKind::QuantifierBinder,
            identity: BinderIdentity::Generated {
                context: generated_kind_context,
                counter: 0,
            },
            ..binding_draft_defaults(generated_kind_context, range(source, 0, 1))
        });
        generated_kind_contexts.contexts[generated_kind_context.index()].bindings =
            vec![generated_kind];
        assert!(matches!(
            BindingEnv::try_new(parts_with(
                source,
                generated_kind_contexts,
                generated_kind_bindings,
            )),
            Err(BindingEnvError::InvalidBindingKindIdentity { binding })
                if binding == generated_kind
        ));

        let mut generated_kind_identity_contexts = BindingContextTable::new();
        let generated_kind_identity_context =
            generated_kind_identity_contexts.insert(context_draft(
                BindingContextOwner::Module,
                None,
                BindingContextLayer::Module,
                Some(LocalTermScope::new(vec![0])),
            ));
        let mut generated_kind_identity_bindings = BindingTable::new();
        let generated_kind_identity = generated_kind_identity_bindings.insert(BindingDraft {
            kind: BindingKind::Generated,
            ..binding_draft_defaults(generated_kind_identity_context, range(source, 0, 1))
        });
        generated_kind_identity_contexts.contexts[generated_kind_identity_context.index()]
            .bindings = vec![generated_kind_identity];
        assert!(matches!(
            BindingEnv::try_new(parts_with(
                source,
                generated_kind_identity_contexts,
                generated_kind_identity_bindings,
            )),
            Err(BindingEnvError::InvalidBindingKindIdentity { binding })
                if binding == generated_kind_identity
        ));

        let mut generated_identity_contexts = BindingContextTable::new();
        let generated_identity_context = generated_identity_contexts.insert(context_draft(
            BindingContextOwner::Module,
            None,
            BindingContextLayer::Module,
            None,
        ));
        let mut generated_identity_bindings = BindingTable::new();
        let generated_identity = generated_identity_bindings.insert(BindingDraft {
            captured: CapturedFreeVariables::new(vec![BinderIdentity::Generated {
                context: BindingContextId::new(99),
                counter: 0,
            }]),
            ..binding_draft_defaults(generated_identity_context, range(source, 0, 1))
        });
        generated_identity_contexts.contexts[generated_identity_context.index()].bindings =
            vec![generated_identity];
        assert!(matches!(
            BindingEnv::try_new(parts_with(
                source,
                generated_identity_contexts,
                generated_identity_bindings,
            )),
            Err(BindingEnvError::InvalidGeneratedIdentityContext { binding, context })
                if binding == generated_identity && context == BindingContextId::new(99)
        ));

        let mut generated_owner_contexts = BindingContextTable::new();
        let root = generated_owner_contexts.insert(context_draft(
            BindingContextOwner::Module,
            None,
            BindingContextLayer::Module,
            None,
        ));
        let child = generated_owner_contexts.insert(context_draft(
            BindingContextOwner::Generated("child".to_owned()),
            Some(root),
            BindingContextLayer::Block,
            None,
        ));
        let mut generated_owner_bindings = BindingTable::new();
        let generated_owner = generated_owner_bindings.insert(BindingDraft {
            kind: BindingKind::Generated,
            identity: BinderIdentity::Generated {
                context: root,
                counter: 0,
            },
            owner_context: child,
            ..binding_draft_defaults(child, range(source, 0, 1))
        });
        generated_owner_contexts.contexts[child.index()].bindings = vec![generated_owner];
        assert!(matches!(
            BindingEnv::try_new(parts_with(
                source,
                generated_owner_contexts,
                generated_owner_bindings,
            )),
            Err(BindingEnvError::InvalidGeneratedIdentityOwner {
                binding,
                context,
                owner
            }) if binding == generated_owner && context == root && owner == child
        ));

        let mut direct_missing_generated_contexts = BindingContextTable::new();
        let direct_missing_generated_context =
            direct_missing_generated_contexts.insert(context_draft(
                BindingContextOwner::Module,
                None,
                BindingContextLayer::Module,
                None,
            ));
        let mut direct_missing_generated_bindings = BindingTable::new();
        let direct_missing_generated = direct_missing_generated_bindings.insert(BindingDraft {
            kind: BindingKind::Generated,
            identity: BinderIdentity::Generated {
                context: BindingContextId::new(99),
                counter: 0,
            },
            ..binding_draft_defaults(direct_missing_generated_context, range(source, 0, 1))
        });
        direct_missing_generated_contexts.contexts[direct_missing_generated_context.index()]
            .bindings = vec![direct_missing_generated];
        assert!(matches!(
            BindingEnv::try_new(parts_with(
                source,
                direct_missing_generated_contexts,
                direct_missing_generated_bindings,
            )),
            Err(BindingEnvError::InvalidGeneratedIdentityContext { binding, context })
                if binding == direct_missing_generated && context == BindingContextId::new(99)
        ));

        let mut diagnostics = BindingDiagnosticTable::new();
        let diagnostic = diagnostics.insert(BindingDiagnosticDraft {
            source_range: Some(range(other_source, 0, 1)),
            class: BindingDiagnosticClass::ExternalDependencyGap,
            severity: BindingDiagnosticSeverity::Note,
            message_key: "checker.binding.external".to_owned(),
            recovery: BindingDiagnosticRecovery::Degraded,
        });
        let mut diagnostic_contexts = BindingContextTable::new();
        diagnostic_contexts.insert(context_draft(
            BindingContextOwner::Module,
            None,
            BindingContextLayer::Module,
            None,
        ));
        let mut env_parts = parts_with(source, diagnostic_contexts, BindingTable::new());
        env_parts.diagnostics = diagnostics;
        assert!(matches!(
            BindingEnv::try_new(env_parts),
            Err(BindingEnvError::InvalidDiagnosticRange { diagnostic: failed }) if failed == diagnostic
        ));
    }

    #[test]
    fn canonical_iteration_and_name_resolution_rendering_are_deterministic() {
        let source = source_id();
        let mut table = BindingTable::new();
        let later = table.insert(BindingDraft {
            spelling: "z".to_owned(),
            declaration_range: range(source, 10, 11),
            identity: BinderIdentity::ReservedVariable {
                spelling: "z".to_owned(),
                declaration_range: range(source, 10, 11),
            },
            ..binding_draft_defaults(BindingContextId::new(0), range(source, 10, 11))
        });
        let earlier = table.insert(BindingDraft {
            spelling: "a".to_owned(),
            declaration_range: range(source, 0, 1),
            identity: BinderIdentity::ReservedVariable {
                spelling: "a".to_owned(),
                declaration_range: range(source, 0, 1),
            },
            ..binding_draft_defaults(BindingContextId::new(0), range(source, 0, 1))
        });
        assert_eq!(
            table.canonical_iter().map(|(id, _)| id).collect::<Vec<_>>(),
            vec![earlier, later]
        );

        let name_resolutions = [
            NameResolution::Resolved(SymbolRef::new(symbol_id(), range(source, 0, 1))),
            NameResolution::ResolvedBuiltin(BuiltinRef::new(
                mizar_resolve::resolved_ast::BuiltinId::new("set"),
                range(source, 0, 1),
                "set",
            )),
            NameResolution::Ambiguous(AmbiguousNameRef::new("x", range(source, 0, 1), Vec::new())),
            NameResolution::Unresolved(UnresolvedNameRef::new(
                "x",
                range(source, 0, 1),
                mizar_resolve::resolved_ast::NameLookupClass::Symbol,
            )),
        ];
        for resolution in name_resolutions {
            let site = BindingLookupSite::new("x", BindingContextId::new(0), None, 0)
                .with_resolver_resolution(resolution.clone());
            assert_eq!(site.resolver_resolution, Some(resolution));
        }
    }

    #[test]
    fn public_data_shapes_do_not_expose_later_checker_boundary_fields() {
        let source = include_str!("binding_env.rs");
        for forbidden in [
            concat!("V", "cId"),
            concat!("Obligation", "Anchor"),
            concat!("Proof", "Witness"),
            concat!("prover", "_result"),
            concat!("accepted", "_verifier", "_status"),
            concat!("overload", "_root"),
            concat!("active", "_registration"),
            concat!("inserted", "_qua"),
            concat!("BindingContextOwner::", "ResolvedNode"),
            concat!("Resolved", "Node("),
            concat!("resolved", "_node#"),
            concat!("ResolvedNode::", "kind"),
            concat!(".", "kind", "()"),
        ] {
            assert!(
                !source.contains(forbidden),
                "binding_env data shape must not expose {forbidden}"
            );
        }
        for (line_number, line) in source.lines().enumerate() {
            if line.trim_start().starts_with("//") {
                continue;
            }
            for forbidden in [concat!("Resolved", "NodeId"), concat!("Reference", "Site")] {
                assert!(
                    !line.contains(forbidden),
                    "binding_env non-comment line {} must not expose {forbidden}",
                    line_number + 1
                );
            }
        }
    }

    fn context_draft(
        owner: BindingContextOwner,
        parent: Option<BindingContextId>,
        layer: BindingContextLayer,
        lexical_scope: Option<LocalTermScope>,
    ) -> BindingContextDraft {
        BindingContextDraft {
            owner,
            parent,
            layer,
            lexical_scope,
            bindings: Vec::new(),
            visible_bindings: Vec::new(),
            recovery: BindingContextRecovery::Normal,
        }
    }

    fn binding_draft_defaults(
        owner_context: BindingContextId,
        declaration_range: SourceRange,
    ) -> BindingDraft {
        BindingDraft {
            spelling: "x".to_owned(),
            kind: BindingKind::QuantifierBinder,
            identity: BinderIdentity::ResolverLocal {
                scope: LocalTermScope::new(vec![0]),
                ordinal: 1,
                declaration_range,
            },
            owner_context,
            declaration_range,
            visible_after_ordinal: 1,
            type_site: BindingTypeSite::Missing,
            status: BindingStatus::Active,
            captured: CapturedFreeVariables::default(),
            diagnostics: Vec::new(),
            recovery: BindingRecoveryState::Normal,
        }
    }

    fn local_binding(
        owner_context: BindingContextId,
        spelling: &str,
        scope: Vec<u32>,
        declaration_range: SourceRange,
        visible_after_ordinal: usize,
    ) -> BindingDraft {
        BindingDraft::from_local_term(
            owner_context,
            BindingKind::QuantifierBinder,
            &LocalTermBinding::new(
                spelling,
                LocalTermScope::new(scope),
                declaration_range,
                visible_after_ordinal,
            ),
        )
    }

    fn reserved_binding(
        owner_context: BindingContextId,
        spelling: &str,
        declaration_range: SourceRange,
        visible_after_ordinal: usize,
    ) -> BindingDraft {
        BindingDraft {
            spelling: spelling.to_owned(),
            kind: BindingKind::ReservedVariable,
            identity: BinderIdentity::ReservedVariable {
                spelling: spelling.to_owned(),
                declaration_range,
            },
            owner_context,
            declaration_range,
            visible_after_ordinal,
            type_site: BindingTypeSite::Source(declaration_range),
            status: BindingStatus::Reserved,
            captured: CapturedFreeVariables::default(),
            diagnostics: Vec::new(),
            recovery: BindingRecoveryState::Normal,
        }
    }

    fn parts_with(
        source_id: SourceId,
        contexts: BindingContextTable,
        bindings: BindingTable,
    ) -> BindingEnvParts {
        BindingEnvParts {
            source_id,
            module_id: module_id(),
            contexts,
            bindings,
            diagnostics: BindingDiagnosticTable::new(),
        }
    }

    fn source_id() -> SourceId {
        source_id_with_suffix("11")
    }

    fn distinct_source_ids() -> (SourceId, SourceId) {
        let snapshot = BuildSnapshotId::from_published_schema_str(&format!(
            "mizar-session-build-snapshot-v1:{}",
            "33".repeat(32)
        ))
        .unwrap();
        let allocator = InMemorySessionIdAllocator::new();
        (
            allocator.next_source_id(snapshot).unwrap(),
            allocator.next_source_id(snapshot).unwrap(),
        )
    }

    fn source_id_with_suffix(suffix: &str) -> SourceId {
        let snapshot = BuildSnapshotId::from_published_schema_str(&format!(
            "mizar-session-build-snapshot-v1:{}",
            suffix.repeat(32)
        ))
        .unwrap();
        InMemorySessionIdAllocator::new()
            .next_source_id(snapshot)
            .unwrap()
    }

    fn range(source_id: SourceId, start: usize, end: usize) -> SourceRange {
        SourceRange {
            source_id,
            start,
            end,
        }
    }

    fn module_id() -> ModuleId {
        ModuleId::new(PackageId::new("pkg"), ModulePath::new("main"))
    }

    fn symbol_id() -> SymbolId {
        SymbolId::new(
            module_id(),
            LocalSymbolId::new("f/0"),
            FullyQualifiedName::new("pkg::main::f/0"),
        )
    }
}
