//! Source-shaped typed AST data tables for checker phase 6.

use crate::{
    source_attribute::SourceAttributeHandoff, source_context::SourceBindingContextHandoff,
    source_evidence::SourceEvidenceHandoff, source_type::SourceTypeApplicationHandoff,
};
use mizar_resolve::resolved_ast::{ModuleId, ResolvedNodeId, SymbolId};
use mizar_session::{GeneratedSpanAnchor, SourceAnchor, SourceId, SourceRange};
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

macro_rules! string_key {
    ($name:ident) => {
        #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $name(String);

        impl $name {
            pub fn new(value: impl Into<String>) -> Self {
                Self(value.into())
            }

            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl From<&str> for $name {
            fn from(value: &str) -> Self {
                Self::new(value)
            }
        }

        impl From<String> for $name {
            fn from(value: String) -> Self {
                Self::new(value)
            }
        }
    };
}

dense_id!(TypedNodeId);
dense_id!(LocalTypeContextId);
dense_id!(TypeEntryId);
dense_id!(NormalizedTypeId);
dense_id!(OpenCandidateSetId);
dense_id!(TypeFactId);
dense_id!(CoercionId);
dense_id!(InitialObligationId);
dense_id!(TypeDiagnosticId);

string_key!(TypedNodeKind);
string_key!(TypeRole);
string_key!(TypePredicateRef);
string_key!(TypeRuleId);
string_key!(TypeAssumptionId);
string_key!(BuiltinRuleId);
string_key!(ResolutionStepId);
string_key!(InitialObligationGoal);
string_key!(InitialObligationProvenance);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypedAst {
    source_id: SourceId,
    module_id: ModuleId,
    resolved_root: Option<ResolvedNodeId>,
    source_context: Option<SourceBindingContextHandoff>,
    source_type: Option<SourceTypeApplicationHandoff>,
    source_attribute: Option<SourceAttributeHandoff>,
    source_evidence: Option<SourceEvidenceHandoff>,
    nodes: TypedArena,
    contexts: LocalTypeContextTable,
    types: TypeTable,
    facts: TypeFactTable,
    coercions: CoercionTable,
    initial_obligations: InitialObligationTable,
    diagnostics: TypeDiagnosticTable,
}

impl TypedAst {
    pub fn try_new(parts: TypedAstParts) -> Result<Self, TypedAstError> {
        validate_typed_ast(&parts)?;
        Ok(Self {
            source_id: parts.source_id,
            module_id: parts.module_id,
            resolved_root: parts.resolved_root,
            source_context: parts.source_context,
            source_type: parts.source_type,
            source_attribute: parts.source_attribute,
            source_evidence: None,
            nodes: parts.nodes,
            contexts: parts.contexts,
            types: parts.types,
            facts: parts.facts,
            coercions: parts.coercions,
            initial_obligations: parts.initial_obligations,
            diagnostics: parts.diagnostics,
        })
    }

    pub const fn source_id(&self) -> SourceId {
        self.source_id
    }

    pub const fn module_id(&self) -> &ModuleId {
        &self.module_id
    }

    pub const fn resolved_root(&self) -> Option<ResolvedNodeId> {
        self.resolved_root
    }

    pub const fn source_context(&self) -> Option<&SourceBindingContextHandoff> {
        self.source_context.as_ref()
    }

    pub const fn source_type(&self) -> Option<&SourceTypeApplicationHandoff> {
        self.source_type.as_ref()
    }

    pub const fn source_attribute(&self) -> Option<&SourceAttributeHandoff> {
        self.source_attribute.as_ref()
    }

    pub const fn source_evidence(&self) -> Option<&SourceEvidenceHandoff> {
        self.source_evidence.as_ref()
    }

    pub fn with_source_evidence(
        mut self,
        handoff: SourceEvidenceHandoff,
    ) -> Result<Self, TypedAstError> {
        if self.source_evidence.is_some() {
            return Err(TypedAstError::InvalidSourceEvidence);
        }
        let source_type = self
            .source_type
            .as_ref()
            .ok_or(TypedAstError::InvalidSourceEvidence)?;
        handoff
            .validate_installation(
                self.source_id,
                &self.module_id,
                source_type,
                self.source_attribute.as_ref(),
                &self.facts,
            )
            .map_err(|_| TypedAstError::InvalidSourceEvidence)?;
        self.source_evidence = Some(handoff);
        Ok(self)
    }

    pub const fn nodes(&self) -> &TypedArena {
        &self.nodes
    }

    pub const fn contexts(&self) -> &LocalTypeContextTable {
        &self.contexts
    }

    pub const fn types(&self) -> &TypeTable {
        &self.types
    }

    pub const fn facts(&self) -> &TypeFactTable {
        &self.facts
    }

    pub const fn coercions(&self) -> &CoercionTable {
        &self.coercions
    }

    pub const fn initial_obligations(&self) -> &InitialObligationTable {
        &self.initial_obligations
    }

    pub const fn diagnostics(&self) -> &TypeDiagnosticTable {
        &self.diagnostics
    }

    pub fn debug_text(&self) -> String {
        let mut output = String::from("typed-ast-debug-v1\n");
        output.push_str("module: ");
        write_module_id(&mut output, &self.module_id);
        output.push('\n');
        output.push_str("root: ");
        write_optional_node_id(&mut output, self.nodes.root());
        output.push('\n');
        output.push_str("resolved_root: ");
        write_optional_resolved_node_id(&mut output, self.resolved_root);
        output.push('\n');
        if let Some(source_context) = &self.source_context {
            output.push_str(&source_context.debug_text());
        }
        if let Some(source_type) = &self.source_type {
            output.push_str(&source_type.debug_text());
        }
        if let Some(source_attribute) = &self.source_attribute {
            output.push_str(&source_attribute.debug_text());
        }
        if let Some(source_evidence) = &self.source_evidence {
            output.push_str(&source_evidence.debug_text());
        }
        write_nodes(&mut output, &self.nodes);
        write_contexts(&mut output, &self.contexts);
        write_type_entries(&mut output, &self.types);
        write_facts(&mut output, &self.facts);
        write_coercions(&mut output, &self.coercions);
        write_initial_obligations(&mut output, &self.initial_obligations);
        write_diagnostics(&mut output, &self.diagnostics);
        output
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypedAstParts {
    pub source_id: SourceId,
    pub module_id: ModuleId,
    pub resolved_root: Option<ResolvedNodeId>,
    pub source_context: Option<SourceBindingContextHandoff>,
    pub source_type: Option<SourceTypeApplicationHandoff>,
    pub source_attribute: Option<SourceAttributeHandoff>,
    pub nodes: TypedArena,
    pub contexts: LocalTypeContextTable,
    pub types: TypeTable,
    pub facts: TypeFactTable,
    pub coercions: CoercionTable,
    pub initial_obligations: InitialObligationTable,
    pub diagnostics: TypeDiagnosticTable,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypedNode {
    pub kind: TypedNodeKind,
    pub resolved_node: Option<ResolvedNodeId>,
    pub anchor: SourceAnchor,
    pub children: Vec<TypedNodeId>,
    pub typing: TypingState,
    pub recovery: NodeRecoveryState,
    pub links: TypedNodeLinks,
}

impl TypedNode {
    pub fn new(kind: impl Into<TypedNodeKind>, anchor: SourceAnchor) -> Self {
        Self {
            kind: kind.into(),
            resolved_node: None,
            anchor,
            children: Vec::new(),
            typing: TypingState::Unknown,
            recovery: NodeRecoveryState::Normal,
            links: TypedNodeLinks::default(),
        }
    }

    pub fn with_children(mut self, children: Vec<TypedNodeId>) -> Self {
        self.children = children;
        self
    }

    pub const fn with_resolved_node(mut self, resolved_node: ResolvedNodeId) -> Self {
        self.resolved_node = Some(resolved_node);
        self
    }

    pub const fn with_typing(mut self, typing: TypingState) -> Self {
        self.typing = typing;
        self
    }

    pub const fn with_recovery(mut self, recovery: NodeRecoveryState) -> Self {
        self.recovery = recovery;
        self
    }

    pub fn with_links(mut self, links: TypedNodeLinks) -> Self {
        self.links = links;
        self
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TypedNodeLinks {
    pub context: Option<LocalTypeContextId>,
    pub type_entry: Option<TypeEntryId>,
    pub facts: Vec<TypeFactId>,
    pub coercions: Vec<CoercionId>,
    pub initial_obligations: Vec<InitialObligationId>,
    pub diagnostics: Vec<TypeDiagnosticId>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum TypingState {
    Successful,
    Assumed,
    Unknown,
    Error,
    Skipped,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum NodeRecoveryState {
    Normal,
    Recovered,
    Degraded,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypedArena {
    root: Option<TypedNodeId>,
    nodes: Vec<TypedNode>,
}

impl TypedArena {
    pub fn try_new(
        root: Option<TypedNodeId>,
        nodes: Vec<TypedNode>,
    ) -> Result<Self, TypedArenaError> {
        validate_nodes(&nodes)?;
        if let Some(root) = root
            && root.index() >= nodes.len()
        {
            return Err(TypedArenaError::InvalidRoot { root });
        }
        Ok(Self { root, nodes })
    }

    pub const fn root(&self) -> Option<TypedNodeId> {
        self.root
    }

    pub fn node(&self, id: TypedNodeId) -> Option<&TypedNode> {
        self.nodes.get(id.index())
    }

    pub fn iter(&self) -> impl Iterator<Item = (TypedNodeId, &TypedNode)> {
        self.nodes
            .iter()
            .enumerate()
            .map(|(index, node)| (TypedNodeId::new(index), node))
    }

    pub const fn len(&self) -> usize {
        self.nodes.len()
    }

    pub const fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }
}

#[derive(Debug, Clone, Default)]
pub struct TypedArenaBuilder {
    nodes: Vec<TypedNode>,
}

impl TypedArenaBuilder {
    pub const fn new() -> Self {
        Self { nodes: Vec::new() }
    }

    pub fn push(&mut self, node: TypedNode) -> Result<TypedNodeId, TypedArenaError> {
        let id = TypedNodeId::new(self.nodes.len());
        for child in &node.children {
            if child.index() >= self.nodes.len() {
                return Err(TypedArenaError::InvalidChild {
                    node: id,
                    child: *child,
                });
            }
        }
        self.nodes.push(node);
        Ok(id)
    }

    pub fn finish(self, root: Option<TypedNodeId>) -> Result<TypedArena, TypedArenaError> {
        TypedArena::try_new(root, self.nodes)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum TypedArenaError {
    InvalidRoot {
        root: TypedNodeId,
    },
    InvalidChild {
        node: TypedNodeId,
        child: TypedNodeId,
    },
    Cycle {
        node: TypedNodeId,
    },
}

impl fmt::Display for TypedArenaError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidRoot { root } => {
                write!(
                    formatter,
                    "typed arena root {} does not exist",
                    root.index()
                )
            }
            Self::InvalidChild { node, child } => write!(
                formatter,
                "typed arena node {} references missing child {}",
                node.index(),
                child.index()
            ),
            Self::Cycle { node } => {
                write!(
                    formatter,
                    "typed arena has a cycle at node {}",
                    node.index()
                )
            }
        }
    }
}

impl Error for TypedArenaError {}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum TypedSiteRef {
    Node(TypedNodeId),
    Role { node: TypedNodeId, role: TypeRole },
}

impl TypedSiteRef {
    pub const fn node(&self) -> TypedNodeId {
        match self {
            Self::Node(node) | Self::Role { node, .. } => *node,
        }
    }
}

pub type TypedSubjectRef = TypedSiteRef;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalTypeContext {
    pub id: LocalTypeContextId,
    pub owner: TypedSiteRef,
    pub parent: Option<LocalTypeContextId>,
    pub layer: TypeContextLayer,
    pub bindings: Vec<BindingTypeRef>,
    pub introduced_assumptions: Vec<TypeFactId>,
    pub visible_facts: Vec<TypeFactId>,
    pub recovery: ContextRecoveryState,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalTypeContextDraft {
    pub owner: TypedSiteRef,
    pub parent: Option<LocalTypeContextId>,
    pub layer: TypeContextLayer,
    pub bindings: Vec<BindingTypeRef>,
    pub introduced_assumptions: Vec<TypeFactId>,
    pub visible_facts: Vec<TypeFactId>,
    pub recovery: ContextRecoveryState,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct LocalTypeContextTable {
    entries: Vec<LocalTypeContext>,
}

impl LocalTypeContextTable {
    pub const fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn insert(&mut self, draft: LocalTypeContextDraft) -> LocalTypeContextId {
        let id = LocalTypeContextId::new(self.entries.len());
        self.entries.push(LocalTypeContext {
            id,
            owner: draft.owner,
            parent: draft.parent,
            layer: draft.layer,
            bindings: draft.bindings,
            introduced_assumptions: draft.introduced_assumptions,
            visible_facts: draft.visible_facts,
            recovery: draft.recovery,
        });
        id
    }

    pub fn get(&self, id: LocalTypeContextId) -> Option<&LocalTypeContext> {
        self.entries.get(id.index())
    }

    pub fn iter(&self) -> impl Iterator<Item = (LocalTypeContextId, &LocalTypeContext)> {
        self.entries.iter().map(|entry| (entry.id, entry))
    }

    pub const fn len(&self) -> usize {
        self.entries.len()
    }

    pub const fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum TypeContextLayer {
    Module,
    Declaration,
    Proof,
    Block,
    Expression,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum BindingTypeRef {
    Symbol(SymbolId),
    Site(TypedSiteRef),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum ContextRecoveryState {
    Normal,
    Recovered,
    Degraded,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeEntry {
    pub id: TypeEntryId,
    pub owner: TypedSiteRef,
    pub expected: Option<NormalizedTypeId>,
    pub actual: TypeEntryActual,
    pub status: TypeStatus,
    pub provenance: TypeProvenance,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeEntryDraft {
    pub owner: TypedSiteRef,
    pub expected: Option<NormalizedTypeId>,
    pub actual: TypeEntryActual,
    pub status: TypeStatus,
    pub provenance: TypeProvenance,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TypeTable {
    entries: Vec<TypeEntry>,
}

impl TypeTable {
    pub const fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn insert(&mut self, draft: TypeEntryDraft) -> TypeEntryId {
        let id = TypeEntryId::new(self.entries.len());
        self.entries.push(TypeEntry {
            id,
            owner: draft.owner,
            expected: draft.expected,
            actual: draft.actual,
            status: draft.status,
            provenance: draft.provenance,
        });
        id
    }

    pub fn get(&self, id: TypeEntryId) -> Option<&TypeEntry> {
        self.entries.get(id.index())
    }

    pub fn iter(&self) -> impl Iterator<Item = (TypeEntryId, &TypeEntry)> {
        self.entries.iter().map(|entry| (entry.id, entry))
    }

    pub fn canonical_iter(&self) -> impl Iterator<Item = (TypeEntryId, &TypeEntry)> {
        let mut entries = self.entries.iter().collect::<Vec<_>>();
        entries.sort_by_key(|entry| type_entry_key(entry));
        entries.into_iter().map(|entry| (entry.id, entry))
    }

    pub const fn len(&self) -> usize {
        self.entries.len()
    }

    pub const fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum TypeStatus {
    Known,
    Assumed,
    Unknown,
    Error,
    Skipped,
}

impl TypeStatus {
    pub const fn is_available_for_handoff(self) -> bool {
        matches!(self, Self::Known | Self::Assumed)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum TypeEntryActual {
    Known(NormalizedTypeId),
    CandidateSet(OpenCandidateSetId),
    Absent,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum TypeProvenance {
    Declared(SourceRangeKey),
    Assumed(TypeAssumptionId),
    Inferred(TypeRuleId),
    Builtin(BuiltinRuleId),
    Recovery(TypeDiagnosticId),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeFact {
    pub id: TypeFactId,
    pub subject: TypedSubjectRef,
    pub predicate: TypePredicateRef,
    pub polarity: Polarity,
    pub provenance: FactProvenance,
    pub status: FactStatus,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeFactDraft {
    pub subject: TypedSubjectRef,
    pub predicate: TypePredicateRef,
    pub polarity: Polarity,
    pub provenance: FactProvenance,
    pub status: FactStatus,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TypeFactTable {
    entries: Vec<TypeFact>,
    keys: BTreeMap<FactKey, TypeFactId>,
}

impl TypeFactTable {
    pub const fn new() -> Self {
        Self {
            entries: Vec::new(),
            keys: BTreeMap::new(),
        }
    }

    pub fn insert(&mut self, draft: TypeFactDraft) -> TypeFactId {
        let key = fact_key(&draft);
        if let Some(id) = self.keys.get(&key) {
            return *id;
        }
        let id = TypeFactId::new(self.entries.len());
        self.entries.push(TypeFact {
            id,
            subject: draft.subject,
            predicate: draft.predicate,
            polarity: draft.polarity,
            provenance: draft.provenance,
            status: draft.status,
        });
        self.keys.insert(key, id);
        id
    }

    pub fn get(&self, id: TypeFactId) -> Option<&TypeFact> {
        self.entries.get(id.index())
    }

    pub fn iter(&self) -> impl Iterator<Item = (TypeFactId, &TypeFact)> {
        self.entries.iter().map(|entry| (entry.id, entry))
    }

    pub fn canonical_iter(&self) -> impl Iterator<Item = (TypeFactId, &TypeFact)> {
        self.keys
            .values()
            .copied()
            .map(|id| (id, &self.entries[id.index()]))
    }

    pub const fn len(&self) -> usize {
        self.entries.len()
    }

    pub const fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum Polarity {
    Positive,
    Negative,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum FactProvenance {
    Declared(SourceRangeKey),
    Assumed(TypeAssumptionId),
    Inferred(TypeRuleId),
    Obligation(InitialObligationId),
    Builtin(BuiltinRuleId),
    Registration(ResolutionStepId),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum FactStatus {
    Known,
    Assumed,
    PendingObligation,
    Degraded,
    Rejected,
}

impl FactStatus {
    pub const fn is_unconditionally_consumable(self) -> bool {
        matches!(self, Self::Known)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CoercionEntry {
    pub id: CoercionId,
    pub site: TypedSiteRef,
    pub from: Option<NormalizedTypeId>,
    pub to: NormalizedTypeId,
    pub kind: CoercionKind,
    pub status: CoercionStatus,
    pub supporting_facts: Vec<TypeFactId>,
    pub obligation: Option<InitialObligationId>,
    pub provenance: CoercionProvenance,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CoercionDraft {
    pub site: TypedSiteRef,
    pub from: Option<NormalizedTypeId>,
    pub to: NormalizedTypeId,
    pub kind: CoercionKind,
    pub status: CoercionStatus,
    pub supporting_facts: Vec<TypeFactId>,
    pub obligation: Option<InitialObligationId>,
    pub provenance: CoercionProvenance,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CoercionTable {
    entries: Vec<CoercionEntry>,
}

impl CoercionTable {
    pub const fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn insert(&mut self, draft: CoercionDraft) -> CoercionId {
        let id = CoercionId::new(self.entries.len());
        self.entries.push(CoercionEntry {
            id,
            site: draft.site,
            from: draft.from,
            to: draft.to,
            kind: draft.kind,
            status: draft.status,
            supporting_facts: draft.supporting_facts,
            obligation: draft.obligation,
            provenance: draft.provenance,
        });
        id
    }

    pub fn get(&self, id: CoercionId) -> Option<&CoercionEntry> {
        self.entries.get(id.index())
    }

    pub fn iter(&self) -> impl Iterator<Item = (CoercionId, &CoercionEntry)> {
        self.entries.iter().map(|entry| (entry.id, entry))
    }

    pub fn canonical_iter(&self) -> impl Iterator<Item = (CoercionId, &CoercionEntry)> {
        let mut entries = self.entries.iter().collect::<Vec<_>>();
        entries.sort_by_key(|entry| coercion_key(entry));
        entries.into_iter().map(|entry| (entry.id, entry))
    }

    pub const fn len(&self) -> usize {
        self.entries.len()
    }

    pub const fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum CoercionKind {
    Widening,
    Narrowing,
    SourceQua,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum CoercionStatus {
    Candidate,
    RequiresObligation,
    Blocked,
    Rejected,
}

impl CoercionStatus {
    pub const fn is_available_for_handoff(self) -> bool {
        matches!(self, Self::Candidate | Self::RequiresObligation)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum CoercionProvenance {
    WideningRule(TypeRuleId),
    NarrowingClaim(SourceRangeKey),
    SourceQua(SourceRangeKey),
    Recovery(TypeDiagnosticId),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InitialObligation {
    pub id: InitialObligationId,
    pub kind: InitialObligationKind,
    pub owner: TypedSiteRef,
    pub source_range: SourceRange,
    pub assumptions: Vec<TypeFactId>,
    pub goal: InitialObligationGoal,
    pub provenance: InitialObligationProvenance,
    pub status: InitialObligationStatus,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InitialObligationDraft {
    pub kind: InitialObligationKind,
    pub owner: TypedSiteRef,
    pub source_range: SourceRange,
    pub assumptions: Vec<TypeFactId>,
    pub goal: InitialObligationGoal,
    pub provenance: InitialObligationProvenance,
    pub status: InitialObligationStatus,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct InitialObligationTable {
    entries: Vec<InitialObligation>,
}

impl InitialObligationTable {
    pub const fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn insert(&mut self, draft: InitialObligationDraft) -> InitialObligationId {
        let id = InitialObligationId::new(self.entries.len());
        self.entries.push(InitialObligation {
            id,
            kind: draft.kind,
            owner: draft.owner,
            source_range: draft.source_range,
            assumptions: draft.assumptions,
            goal: draft.goal,
            provenance: draft.provenance,
            status: draft.status,
        });
        id
    }

    pub fn get(&self, id: InitialObligationId) -> Option<&InitialObligation> {
        self.entries.get(id.index())
    }

    pub fn iter(&self) -> impl Iterator<Item = (InitialObligationId, &InitialObligation)> {
        self.entries.iter().map(|entry| (entry.id, entry))
    }

    pub const fn len(&self) -> usize {
        self.entries.len()
    }

    pub const fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum InitialObligationKind {
    Sethood,
    NonEmptiness,
    Narrowing,
    RegistrationCorrectness,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum InitialObligationStatus {
    Pending,
    Blocked,
    Invalidated,
}

impl InitialObligationStatus {
    pub const fn is_available_for_handoff(self) -> bool {
        matches!(self, Self::Pending)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeDiagnostic {
    pub id: TypeDiagnosticId,
    pub owner: Option<TypedSiteRef>,
    pub source_range: SourceRange,
    pub class: TypeDiagnosticClass,
    pub severity: TypeDiagnosticSeverity,
    pub message_key: String,
    pub recovery: DiagnosticRecoveryState,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeDiagnosticDraft {
    pub owner: Option<TypedSiteRef>,
    pub source_range: SourceRange,
    pub class: TypeDiagnosticClass,
    pub severity: TypeDiagnosticSeverity,
    pub message_key: String,
    pub recovery: DiagnosticRecoveryState,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TypeDiagnosticTable {
    entries: Vec<TypeDiagnostic>,
}

impl TypeDiagnosticTable {
    pub const fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn insert(&mut self, draft: TypeDiagnosticDraft) -> TypeDiagnosticId {
        let id = TypeDiagnosticId::new(self.entries.len());
        self.entries.push(TypeDiagnostic {
            id,
            owner: draft.owner,
            source_range: draft.source_range,
            class: draft.class,
            severity: draft.severity,
            message_key: draft.message_key,
            recovery: draft.recovery,
        });
        id
    }

    pub fn get(&self, id: TypeDiagnosticId) -> Option<&TypeDiagnostic> {
        self.entries.get(id.index())
    }

    pub fn iter(&self) -> impl Iterator<Item = (TypeDiagnosticId, &TypeDiagnostic)> {
        self.entries.iter().map(|entry| (entry.id, entry))
    }

    pub fn canonical_iter(&self) -> impl Iterator<Item = (TypeDiagnosticId, &TypeDiagnostic)> {
        let mut entries = self.entries.iter().collect::<Vec<_>>();
        entries.sort_by_key(|entry| diagnostic_key(entry));
        entries.into_iter().map(|entry| (entry.id, entry))
    }

    pub const fn len(&self) -> usize {
        self.entries.len()
    }

    pub const fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum TypeDiagnosticClass {
    TypeExpression,
    TypeEntry,
    TypeFact,
    Coercion,
    InitialObligation,
    Context,
    Recovery,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum TypeDiagnosticSeverity {
    Error,
    Warning,
    Note,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum DiagnosticRecoveryState {
    Normal,
    Recovery,
    Degraded,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SourceRangeKey {
    pub start: usize,
    pub end: usize,
}

impl From<SourceRange> for SourceRangeKey {
    fn from(range: SourceRange) -> Self {
        Self {
            start: range.start,
            end: range.end,
        }
    }
}

type SiteOrderKey = (usize, u8, String);

type FactKey = (SiteOrderKey, TypePredicateRef, Polarity, FactProvenance);

type CoercionKey = (
    SiteOrderKey,
    CoercionKind,
    NormalizedTypeId,
    CoercionProvenance,
    Vec<TypeFactId>,
    Option<NormalizedTypeId>,
    CoercionId,
);

type TypeEntryKey = (SiteOrderKey, TypeEntryId);

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum TypedAstError {
    Arena(TypedArenaError),
    PayloadSourceMismatch,
    InvalidSourceContext,
    InvalidSourceType,
    InvalidSourceAttribute,
    InvalidSourceEvidence,
    InvalidNodeContext {
        node: TypedNodeId,
        context: LocalTypeContextId,
    },
    InvalidNodeTypeEntry {
        node: TypedNodeId,
        entry: TypeEntryId,
    },
    InvalidNodeFact {
        node: TypedNodeId,
        fact: TypeFactId,
    },
    InvalidNodeCoercion {
        node: TypedNodeId,
        coercion: CoercionId,
    },
    InvalidNodeObligation {
        node: TypedNodeId,
        obligation: InitialObligationId,
    },
    InvalidNodeDiagnostic {
        node: TypedNodeId,
        diagnostic: TypeDiagnosticId,
    },
    InvalidContextOwner {
        context: LocalTypeContextId,
    },
    InvalidContextParent {
        context: LocalTypeContextId,
        parent: LocalTypeContextId,
    },
    ContextCycle {
        context: LocalTypeContextId,
    },
    InvalidContextBinding {
        context: LocalTypeContextId,
    },
    InvalidContextFact {
        context: LocalTypeContextId,
        fact: TypeFactId,
    },
    UnsortedContextFacts {
        context: LocalTypeContextId,
    },
    AssumptionFactNotIntroduced {
        context: LocalTypeContextId,
        fact: TypeFactId,
    },
    IntroducedFactNotAssumed {
        context: LocalTypeContextId,
        fact: TypeFactId,
    },
    InvalidTypeOwner {
        entry: TypeEntryId,
    },
    InvalidTypeActual {
        entry: TypeEntryId,
    },
    InvalidTypeRecoveryDiagnostic {
        entry: TypeEntryId,
        diagnostic: TypeDiagnosticId,
    },
    DuplicateTypeOwner {
        entry: TypeEntryId,
    },
    InvalidFactSubject {
        fact: TypeFactId,
    },
    InvalidFactObligation {
        fact: TypeFactId,
        obligation: InitialObligationId,
    },
    InvalidCoercionSite {
        coercion: CoercionId,
    },
    InvalidCoercionFact {
        coercion: CoercionId,
        fact: TypeFactId,
    },
    InvalidCoercionObligation {
        coercion: CoercionId,
        obligation: InitialObligationId,
    },
    InvalidCoercionRecoveryDiagnostic {
        coercion: CoercionId,
        diagnostic: TypeDiagnosticId,
    },
    MissingCoercionObligation {
        coercion: CoercionId,
    },
    InvalidObligationOwner {
        obligation: InitialObligationId,
    },
    InvalidObligationFact {
        obligation: InitialObligationId,
        fact: TypeFactId,
    },
    InvalidDiagnosticOwner {
        diagnostic: TypeDiagnosticId,
    },
}

impl fmt::Display for TypedAstError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Arena(error) => write!(formatter, "{error}"),
            Self::PayloadSourceMismatch => write!(formatter, "typed AST payload source mismatch"),
            Self::InvalidSourceContext => {
                formatter.write_str("typed AST source context is inconsistent")
            }
            Self::InvalidSourceType => {
                formatter.write_str("typed AST source type handoff is inconsistent")
            }
            Self::InvalidSourceAttribute => {
                formatter.write_str("typed AST source attribute handoff is inconsistent")
            }
            Self::InvalidSourceEvidence => {
                formatter.write_str("typed AST source evidence handoff is inconsistent")
            }
            Self::InvalidNodeContext { node, context } => write!(
                formatter,
                "typed node {} references missing context {}",
                node.index(),
                context.index()
            ),
            Self::InvalidNodeTypeEntry { node, entry } => write!(
                formatter,
                "typed node {} references missing type entry {}",
                node.index(),
                entry.index()
            ),
            Self::InvalidNodeFact { node, fact } => write!(
                formatter,
                "typed node {} references missing fact {}",
                node.index(),
                fact.index()
            ),
            Self::InvalidNodeCoercion { node, coercion } => write!(
                formatter,
                "typed node {} references missing coercion {}",
                node.index(),
                coercion.index()
            ),
            Self::InvalidNodeObligation { node, obligation } => write!(
                formatter,
                "typed node {} references missing initial obligation {}",
                node.index(),
                obligation.index()
            ),
            Self::InvalidNodeDiagnostic { node, diagnostic } => write!(
                formatter,
                "typed node {} references missing diagnostic {}",
                node.index(),
                diagnostic.index()
            ),
            Self::InvalidContextOwner { context } => {
                write!(
                    formatter,
                    "context {} has an invalid owner",
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
            Self::InvalidContextBinding { context } => {
                write!(
                    formatter,
                    "context {} has an invalid binding",
                    context.index()
                )
            }
            Self::InvalidContextFact { context, fact } => write!(
                formatter,
                "context {} references missing fact {}",
                context.index(),
                fact.index()
            ),
            Self::UnsortedContextFacts { context } => {
                write!(
                    formatter,
                    "context {} facts are not sorted",
                    context.index()
                )
            }
            Self::AssumptionFactNotIntroduced { context, fact } => write!(
                formatter,
                "context {} cannot consume assumed fact {}",
                context.index(),
                fact.index()
            ),
            Self::IntroducedFactNotAssumed { context, fact } => write!(
                formatter,
                "context {} introduces non-assumed fact {}",
                context.index(),
                fact.index()
            ),
            Self::InvalidTypeOwner { entry } => {
                write!(
                    formatter,
                    "type entry {} has an invalid owner",
                    entry.index()
                )
            }
            Self::InvalidTypeActual { entry } => write!(
                formatter,
                "type entry {} is available for handoff without an actual type",
                entry.index()
            ),
            Self::InvalidTypeRecoveryDiagnostic { entry, diagnostic } => write!(
                formatter,
                "type entry {} references missing recovery diagnostic {}",
                entry.index(),
                diagnostic.index()
            ),
            Self::DuplicateTypeOwner { entry } => {
                write!(
                    formatter,
                    "type entry {} duplicates an owner",
                    entry.index()
                )
            }
            Self::InvalidFactSubject { fact } => {
                write!(formatter, "fact {} has an invalid subject", fact.index())
            }
            Self::InvalidFactObligation { fact, obligation } => write!(
                formatter,
                "fact {} references missing obligation {}",
                fact.index(),
                obligation.index()
            ),
            Self::InvalidCoercionSite { coercion } => {
                write!(
                    formatter,
                    "coercion {} has an invalid site",
                    coercion.index()
                )
            }
            Self::InvalidCoercionFact { coercion, fact } => write!(
                formatter,
                "coercion {} references missing fact {}",
                coercion.index(),
                fact.index()
            ),
            Self::InvalidCoercionObligation {
                coercion,
                obligation,
            } => write!(
                formatter,
                "coercion {} references missing obligation {}",
                coercion.index(),
                obligation.index()
            ),
            Self::InvalidCoercionRecoveryDiagnostic {
                coercion,
                diagnostic,
            } => write!(
                formatter,
                "coercion {} references missing recovery diagnostic {}",
                coercion.index(),
                diagnostic.index()
            ),
            Self::MissingCoercionObligation { coercion } => write!(
                formatter,
                "coercion {} requires an obligation but has none",
                coercion.index()
            ),
            Self::InvalidObligationOwner { obligation } => write!(
                formatter,
                "initial obligation {} has an invalid owner",
                obligation.index()
            ),
            Self::InvalidObligationFact { obligation, fact } => write!(
                formatter,
                "initial obligation {} references missing fact {}",
                obligation.index(),
                fact.index()
            ),
            Self::InvalidDiagnosticOwner { diagnostic } => write!(
                formatter,
                "diagnostic {} has an invalid owner",
                diagnostic.index()
            ),
        }
    }
}

impl Error for TypedAstError {}

impl From<TypedArenaError> for TypedAstError {
    fn from(error: TypedArenaError) -> Self {
        Self::Arena(error)
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum VisitState {
    Visiting,
    Done,
}

fn validate_typed_ast(parts: &TypedAstParts) -> Result<(), TypedAstError> {
    validate_arena_payloads(parts.source_id, &parts.nodes)?;
    validate_node_links(parts)?;
    validate_contexts(parts)?;
    validate_source_context(parts)?;
    validate_source_type(parts)?;
    validate_source_attribute(parts)?;
    validate_types(parts)?;
    validate_facts(parts)?;
    validate_coercions(parts)?;
    validate_initial_obligations(parts)?;
    validate_diagnostics(parts)?;
    Ok(())
}

fn validate_source_type(parts: &TypedAstParts) -> Result<(), TypedAstError> {
    let Some(handoff) = &parts.source_type else {
        return Ok(());
    };
    handoff
        .validate_installation(parts.source_id, &parts.module_id, &parts.nodes)
        .map_err(|_| TypedAstError::InvalidSourceType)
}

fn validate_source_attribute(parts: &TypedAstParts) -> Result<(), TypedAstError> {
    let Some(handoff) = &parts.source_attribute else {
        return Ok(());
    };
    let source_type = parts
        .source_type
        .as_ref()
        .ok_or(TypedAstError::InvalidSourceAttribute)?;
    handoff
        .validate_installation(parts.source_id, &parts.module_id, source_type, &parts.nodes)
        .map_err(|_| TypedAstError::InvalidSourceAttribute)
}

fn validate_source_context(parts: &TypedAstParts) -> Result<(), TypedAstError> {
    let Some(handoff) = &parts.source_context else {
        return Ok(());
    };
    if handoff.source_id() != parts.source_id
        || handoff.module_id() != &parts.module_id
        || handoff.binding_env().source_id() != parts.source_id
        || handoff.binding_env().module_id() != &parts.module_id
        || handoff.local_contexts() != &parts.contexts
        || handoff.context_links().len() != parts.contexts.len()
    {
        return Err(TypedAstError::InvalidSourceContext);
    }
    let module_context = parts
        .contexts
        .get(LocalTypeContextId::new(0))
        .ok_or(TypedAstError::InvalidSourceContext)?;
    let root = parts
        .nodes
        .root()
        .ok_or(TypedAstError::InvalidSourceContext)?;
    let root_node = parts
        .nodes
        .node(root)
        .ok_or(TypedAstError::InvalidSourceContext)?;
    if module_context.owner != TypedSiteRef::Node(root)
        || root_node.links.context != Some(LocalTypeContextId::new(0))
    {
        return Err(TypedAstError::InvalidSourceContext);
    }
    for (_, link) in handoff.context_links().iter() {
        if handoff
            .binding_env()
            .contexts()
            .get(link.binding_context)
            .is_none()
            || parts.contexts.get(link.local_context).is_none()
        {
            return Err(TypedAstError::InvalidSourceContext);
        }
        match link.item {
            Some(item_id) => {
                let Some(item) = handoff.items().get(item_id) else {
                    return Err(TypedAstError::InvalidSourceContext);
                };
                if item.binding_context != link.binding_context
                    || item.local_context != link.local_context
                {
                    return Err(TypedAstError::InvalidSourceContext);
                }
            }
            None => {
                if link.binding_context.index() != 0 || link.local_context.index() != 0 {
                    return Err(TypedAstError::InvalidSourceContext);
                }
            }
        }
    }
    for (_, item) in handoff.items().iter() {
        validate_source_context_site(parts, &item.site, item.source_range, item.local_context)?;
        if handoff
            .binding_env()
            .contexts()
            .get(item.binding_context)
            .is_none()
            || parts.contexts.get(item.local_context).is_none()
        {
            return Err(TypedAstError::InvalidSourceContext);
        }
    }
    for (_, declaration) in handoff.declarations().iter() {
        validate_source_context_site(
            parts,
            &declaration.site,
            declaration.declaration_range,
            declaration.local_context,
        )?;
        if handoff.items().get(declaration.item).is_none()
            || handoff
                .binding_env()
                .bindings()
                .get(declaration.binding)
                .is_none()
            || handoff
                .binding_env()
                .contexts()
                .get(declaration.binding_context)
                .is_none()
            || parts.contexts.get(declaration.local_context).is_none()
        {
            return Err(TypedAstError::InvalidSourceContext);
        }
    }
    Ok(())
}

fn validate_source_context_site(
    parts: &TypedAstParts,
    site: &TypedSiteRef,
    source_range: SourceRange,
    context: LocalTypeContextId,
) -> Result<(), TypedAstError> {
    validate_site(parts, site).map_err(|()| TypedAstError::InvalidSourceContext)?;
    let node = parts
        .nodes
        .node(site.node())
        .ok_or(TypedAstError::InvalidSourceContext)?;
    if node.anchor != SourceAnchor::Range(source_range) || node.links.context != Some(context) {
        return Err(TypedAstError::InvalidSourceContext);
    }
    Ok(())
}

fn validate_arena_payloads(source_id: SourceId, nodes: &TypedArena) -> Result<(), TypedAstError> {
    for (_, node) in nodes.iter() {
        validate_anchor_source(source_id, &node.anchor)?;
    }
    Ok(())
}

fn validate_node_links(parts: &TypedAstParts) -> Result<(), TypedAstError> {
    for (node_id, node) in parts.nodes.iter() {
        if let Some(context) = node.links.context
            && parts.contexts.get(context).is_none()
        {
            return Err(TypedAstError::InvalidNodeContext {
                node: node_id,
                context,
            });
        }
        if let Some(entry) = node.links.type_entry
            && parts.types.get(entry).is_none()
        {
            return Err(TypedAstError::InvalidNodeTypeEntry {
                node: node_id,
                entry,
            });
        }
        for fact in &node.links.facts {
            if parts.facts.get(*fact).is_none() {
                return Err(TypedAstError::InvalidNodeFact {
                    node: node_id,
                    fact: *fact,
                });
            }
        }
        for coercion in &node.links.coercions {
            if parts.coercions.get(*coercion).is_none() {
                return Err(TypedAstError::InvalidNodeCoercion {
                    node: node_id,
                    coercion: *coercion,
                });
            }
        }
        for obligation in &node.links.initial_obligations {
            if parts.initial_obligations.get(*obligation).is_none() {
                return Err(TypedAstError::InvalidNodeObligation {
                    node: node_id,
                    obligation: *obligation,
                });
            }
        }
        for diagnostic in &node.links.diagnostics {
            if parts.diagnostics.get(*diagnostic).is_none() {
                return Err(TypedAstError::InvalidNodeDiagnostic {
                    node: node_id,
                    diagnostic: *diagnostic,
                });
            }
        }
    }
    Ok(())
}

fn validate_contexts(parts: &TypedAstParts) -> Result<(), TypedAstError> {
    let mut states = vec![None; parts.contexts.len()];
    for (context_id, context) in parts.contexts.iter() {
        validate_site(parts, &context.owner).map_err(|()| TypedAstError::InvalidContextOwner {
            context: context_id,
        })?;
        if let Some(parent) = context.parent
            && parts.contexts.get(parent).is_none()
        {
            return Err(TypedAstError::InvalidContextParent {
                context: context_id,
                parent,
            });
        }
        for binding in &context.bindings {
            if let BindingTypeRef::Site(site) = binding {
                validate_site(parts, site).map_err(|()| TypedAstError::InvalidContextBinding {
                    context: context_id,
                })?;
            }
        }
        if !is_strictly_sorted(&context.introduced_assumptions)
            || !is_strictly_sorted(&context.visible_facts)
        {
            return Err(TypedAstError::UnsortedContextFacts {
                context: context_id,
            });
        }
        for fact in &context.introduced_assumptions {
            let Some(entry) = parts.facts.get(*fact) else {
                return Err(TypedAstError::InvalidContextFact {
                    context: context_id,
                    fact: *fact,
                });
            };
            if entry.status != FactStatus::Assumed {
                return Err(TypedAstError::IntroducedFactNotAssumed {
                    context: context_id,
                    fact: *fact,
                });
            }
        }
        for fact in &context.visible_facts {
            if parts.facts.get(*fact).is_none() {
                return Err(TypedAstError::InvalidContextFact {
                    context: context_id,
                    fact: *fact,
                });
            }
        }
    }
    for context_id in 0..parts.contexts.len() {
        visit_context(
            LocalTypeContextId::new(context_id),
            &parts.contexts,
            &mut states,
        )?;
    }
    for (context_id, context) in parts.contexts.iter() {
        for fact in &context.visible_facts {
            if !context_can_consume_fact(context_id, *fact, &parts.contexts, &parts.facts) {
                return Err(TypedAstError::AssumptionFactNotIntroduced {
                    context: context_id,
                    fact: *fact,
                });
            }
        }
    }
    Ok(())
}

fn validate_types(parts: &TypedAstParts) -> Result<(), TypedAstError> {
    let mut owners = BTreeSet::new();
    for (entry_id, entry) in parts.types.iter() {
        validate_site(parts, &entry.owner)
            .map_err(|()| TypedAstError::InvalidTypeOwner { entry: entry_id })?;
        if entry.status.is_available_for_handoff() && entry.actual == TypeEntryActual::Absent {
            return Err(TypedAstError::InvalidTypeActual { entry: entry_id });
        }
        if let TypeProvenance::Recovery(diagnostic) = &entry.provenance
            && parts.diagnostics.get(*diagnostic).is_none()
        {
            return Err(TypedAstError::InvalidTypeRecoveryDiagnostic {
                entry: entry_id,
                diagnostic: *diagnostic,
            });
        }
        if !owners.insert(site_order_key(&entry.owner)) {
            return Err(TypedAstError::DuplicateTypeOwner { entry: entry_id });
        }
    }
    Ok(())
}

fn validate_facts(parts: &TypedAstParts) -> Result<(), TypedAstError> {
    for (fact_id, fact) in parts.facts.iter() {
        validate_site(parts, &fact.subject)
            .map_err(|()| TypedAstError::InvalidFactSubject { fact: fact_id })?;
        if let FactProvenance::Obligation(obligation) = &fact.provenance
            && parts.initial_obligations.get(*obligation).is_none()
        {
            return Err(TypedAstError::InvalidFactObligation {
                fact: fact_id,
                obligation: *obligation,
            });
        }
    }
    Ok(())
}

fn validate_coercions(parts: &TypedAstParts) -> Result<(), TypedAstError> {
    for (coercion_id, coercion) in parts.coercions.iter() {
        validate_site(parts, &coercion.site).map_err(|()| TypedAstError::InvalidCoercionSite {
            coercion: coercion_id,
        })?;
        for fact in &coercion.supporting_facts {
            if parts.facts.get(*fact).is_none() {
                return Err(TypedAstError::InvalidCoercionFact {
                    coercion: coercion_id,
                    fact: *fact,
                });
            }
        }
        if coercion.status == CoercionStatus::RequiresObligation && coercion.obligation.is_none() {
            return Err(TypedAstError::MissingCoercionObligation {
                coercion: coercion_id,
            });
        }
        if let Some(obligation) = coercion.obligation
            && parts.initial_obligations.get(obligation).is_none()
        {
            return Err(TypedAstError::InvalidCoercionObligation {
                coercion: coercion_id,
                obligation,
            });
        }
        if let CoercionProvenance::Recovery(diagnostic) = &coercion.provenance
            && parts.diagnostics.get(*diagnostic).is_none()
        {
            return Err(TypedAstError::InvalidCoercionRecoveryDiagnostic {
                coercion: coercion_id,
                diagnostic: *diagnostic,
            });
        }
    }
    Ok(())
}

fn validate_initial_obligations(parts: &TypedAstParts) -> Result<(), TypedAstError> {
    for (obligation_id, obligation) in parts.initial_obligations.iter() {
        validate_source_range(parts.source_id, obligation.source_range)?;
        validate_site(parts, &obligation.owner).map_err(|()| {
            TypedAstError::InvalidObligationOwner {
                obligation: obligation_id,
            }
        })?;
        for fact in &obligation.assumptions {
            if parts.facts.get(*fact).is_none() {
                return Err(TypedAstError::InvalidObligationFact {
                    obligation: obligation_id,
                    fact: *fact,
                });
            }
        }
    }
    Ok(())
}

fn validate_diagnostics(parts: &TypedAstParts) -> Result<(), TypedAstError> {
    for (diagnostic_id, diagnostic) in parts.diagnostics.iter() {
        validate_source_range(parts.source_id, diagnostic.source_range)?;
        if let Some(owner) = &diagnostic.owner {
            validate_site(parts, owner).map_err(|()| TypedAstError::InvalidDiagnosticOwner {
                diagnostic: diagnostic_id,
            })?;
        }
    }
    Ok(())
}

fn validate_site(parts: &TypedAstParts, site: &TypedSiteRef) -> Result<(), ()> {
    parts.nodes.node(site.node()).map(|_| ()).ok_or(())
}

fn validate_anchor_source(source_id: SourceId, anchor: &SourceAnchor) -> Result<(), TypedAstError> {
    match anchor {
        SourceAnchor::Range(range) => validate_source_range(source_id, *range),
        SourceAnchor::Point {
            source_id: anchor_source_id,
            ..
        } => {
            if *anchor_source_id == source_id {
                Ok(())
            } else {
                Err(TypedAstError::PayloadSourceMismatch)
            }
        }
        SourceAnchor::Generated(origin) => {
            validate_generated_anchor_source(source_id, origin.anchor())
        }
        _ => Ok(()),
    }
}

fn validate_generated_anchor_source(
    source_id: SourceId,
    anchor: GeneratedSpanAnchor,
) -> Result<(), TypedAstError> {
    match anchor {
        GeneratedSpanAnchor::Range(range) => validate_source_range(source_id, range),
        GeneratedSpanAnchor::Point {
            source_id: anchor_source_id,
            ..
        } => {
            if anchor_source_id == source_id {
                Ok(())
            } else {
                Err(TypedAstError::PayloadSourceMismatch)
            }
        }
        _ => Ok(()),
    }
}

fn validate_source_range(source_id: SourceId, range: SourceRange) -> Result<(), TypedAstError> {
    if range.source_id == source_id {
        Ok(())
    } else {
        Err(TypedAstError::PayloadSourceMismatch)
    }
}

fn context_can_consume_fact(
    context_id: LocalTypeContextId,
    fact_id: TypeFactId,
    contexts: &LocalTypeContextTable,
    facts: &TypeFactTable,
) -> bool {
    let Some(fact) = facts.get(fact_id) else {
        return false;
    };
    if fact.status.is_unconditionally_consumable() {
        return true;
    }
    if fact.status != FactStatus::Assumed {
        return false;
    }

    let mut active = Some(context_id);
    while let Some(id) = active {
        let Some(context) = contexts.get(id) else {
            return false;
        };
        if context
            .introduced_assumptions
            .binary_search(&fact_id)
            .is_ok()
        {
            return true;
        }
        active = context.parent;
    }
    false
}

fn visit_context(
    context_id: LocalTypeContextId,
    contexts: &LocalTypeContextTable,
    states: &mut [Option<VisitState>],
) -> Result<(), TypedAstError> {
    match states[context_id.index()] {
        Some(VisitState::Done) => return Ok(()),
        Some(VisitState::Visiting) => {
            return Err(TypedAstError::ContextCycle {
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

fn validate_nodes(nodes: &[TypedNode]) -> Result<(), TypedArenaError> {
    for (index, node) in nodes.iter().enumerate() {
        let node_id = TypedNodeId::new(index);
        for child in &node.children {
            if child.index() >= nodes.len() {
                return Err(TypedArenaError::InvalidChild {
                    node: node_id,
                    child: *child,
                });
            }
        }
    }
    let mut states = vec![None; nodes.len()];
    for index in 0..nodes.len() {
        visit_node(index, nodes, &mut states)?;
    }
    Ok(())
}

fn visit_node(
    index: usize,
    nodes: &[TypedNode],
    states: &mut [Option<VisitState>],
) -> Result<(), TypedArenaError> {
    match states[index] {
        Some(VisitState::Done) => return Ok(()),
        Some(VisitState::Visiting) => {
            return Err(TypedArenaError::Cycle {
                node: TypedNodeId::new(index),
            });
        }
        None => {}
    }
    states[index] = Some(VisitState::Visiting);
    for child in &nodes[index].children {
        visit_node(child.index(), nodes, states)?;
    }
    states[index] = Some(VisitState::Done);
    Ok(())
}

fn is_strictly_sorted<T: Ord>(values: &[T]) -> bool {
    values.windows(2).all(|window| window[0] < window[1])
}

fn fact_key(draft: &TypeFactDraft) -> FactKey {
    (
        site_order_key(&draft.subject),
        draft.predicate.clone(),
        draft.polarity,
        draft.provenance.clone(),
    )
}

fn coercion_key(entry: &CoercionEntry) -> CoercionKey {
    (
        site_order_key(&entry.site),
        entry.kind,
        entry.to,
        entry.provenance.clone(),
        entry.supporting_facts.clone(),
        entry.from,
        entry.id,
    )
}

fn type_entry_key(entry: &TypeEntry) -> TypeEntryKey {
    (site_order_key(&entry.owner), entry.id)
}

fn diagnostic_key(entry: &TypeDiagnostic) -> (usize, usize, TypeDiagnosticClass, String, usize) {
    (
        entry.source_range.start,
        entry.source_range.end,
        entry.class,
        entry.message_key.clone(),
        entry.id.index(),
    )
}

fn site_order_key(site: &TypedSiteRef) -> SiteOrderKey {
    match site {
        TypedSiteRef::Node(node) => (node.index(), 0, String::new()),
        TypedSiteRef::Role { node, role } => (node.index(), 1, role.as_str().to_owned()),
    }
}

fn site_key(site: &TypedSiteRef) -> String {
    match site {
        TypedSiteRef::Node(node) => format!("node#{}", node.index()),
        TypedSiteRef::Role { node, role } => format!("node#{}:{}", node.index(), role.as_str()),
    }
}

fn write_nodes(output: &mut String, nodes: &TypedArena) {
    output.push_str("nodes:\n");
    if nodes.is_empty() {
        output.push_str("  <none>\n");
        return;
    }
    for (id, node) in nodes.iter() {
        let _ = write!(output, "  node#{} kind=\"", id.index());
        write_escaped(output, node.kind.as_str());
        output.push_str("\" children=");
        write_node_ids(output, &node.children);
        output.push_str(" resolved=");
        write_optional_resolved_node_id(output, node.resolved_node);
        output.push_str(" anchor=");
        write_anchor(output, &node.anchor);
        let _ = write!(
            output,
            " typing={} recovery={}",
            typing_state_name(node.typing),
            node_recovery_state_name(node.recovery)
        );
        output.push_str(" links=");
        write_node_links(output, &node.links);
        output.push('\n');
    }
}

fn write_contexts(output: &mut String, contexts: &LocalTypeContextTable) {
    output.push_str("contexts:\n");
    if contexts.is_empty() {
        output.push_str("  <none>\n");
        return;
    }
    for (id, context) in contexts.iter() {
        let _ = write!(
            output,
            "  context#{} owner={} parent=",
            id.index(),
            site_key(&context.owner)
        );
        match context.parent {
            Some(parent) => {
                let _ = write!(output, "context#{}", parent.index());
            }
            None => output.push_str("<none>"),
        }
        let _ = write!(
            output,
            " layer={} recovery={} bindings=",
            context_layer_name(context.layer),
            context_recovery_name(context.recovery)
        );
        write_bindings(output, &context.bindings);
        output.push_str(" introduced=");
        write_fact_ids(output, &context.introduced_assumptions);
        output.push_str(" visible=");
        write_fact_ids(output, &context.visible_facts);
        output.push('\n');
    }
}

fn write_type_entries(output: &mut String, types: &TypeTable) {
    output.push_str("types:\n");
    if types.is_empty() {
        output.push_str("  <none>\n");
        return;
    }
    for (id, entry) in types.canonical_iter() {
        let _ = write!(
            output,
            "  type#{} owner={} status={} actual=",
            id.index(),
            site_key(&entry.owner),
            type_status_name(entry.status)
        );
        write_type_actual(output, entry.actual);
        output.push_str(" expected=");
        write_optional_type_id(output, entry.expected);
        output.push_str(" provenance=");
        write_type_provenance(output, &entry.provenance);
        output.push('\n');
    }
}

fn write_facts(output: &mut String, facts: &TypeFactTable) {
    output.push_str("facts:\n");
    if facts.is_empty() {
        output.push_str("  <none>\n");
        return;
    }
    for (id, fact) in facts.canonical_iter() {
        let _ = write!(
            output,
            "  fact#{} subject={} predicate=\"",
            id.index(),
            site_key(&fact.subject)
        );
        write_escaped(output, fact.predicate.as_str());
        let _ = write!(
            output,
            "\" polarity={} status={} provenance=",
            polarity_name(fact.polarity),
            fact_status_name(fact.status)
        );
        write_fact_provenance(output, &fact.provenance);
        output.push('\n');
    }
}

fn write_coercions(output: &mut String, coercions: &CoercionTable) {
    output.push_str("coercions:\n");
    if coercions.is_empty() {
        output.push_str("  <none>\n");
        return;
    }
    for (id, coercion) in coercions.canonical_iter() {
        let _ = write!(
            output,
            "  coercion#{} site={} kind={} status={} from=",
            id.index(),
            site_key(&coercion.site),
            coercion_kind_name(coercion.kind),
            coercion_status_name(coercion.status)
        );
        write_optional_type_id(output, coercion.from);
        output.push_str(" to=");
        write_type_id(output, coercion.to);
        output.push_str(" facts=");
        write_fact_ids(output, &coercion.supporting_facts);
        output.push_str(" obligation=");
        write_optional_obligation_id(output, coercion.obligation);
        output.push_str(" provenance=");
        write_coercion_provenance(output, &coercion.provenance);
        output.push('\n');
    }
}

fn write_initial_obligations(output: &mut String, obligations: &InitialObligationTable) {
    output.push_str("initial_obligations:\n");
    if obligations.is_empty() {
        output.push_str("  <none>\n");
        return;
    }
    for (id, obligation) in obligations.iter() {
        let _ = write!(
            output,
            "  obligation#{} kind={} owner={} range=",
            id.index(),
            initial_obligation_kind_name(obligation.kind),
            site_key(&obligation.owner)
        );
        write_range(output, obligation.source_range);
        output.push_str(" assumptions=");
        write_fact_ids(output, &obligation.assumptions);
        let _ = writeln!(
            output,
            " goal=\"{}\" provenance=\"{}\" status={}",
            escaped_display(obligation.goal.as_str()),
            escaped_display(obligation.provenance.as_str()),
            initial_obligation_status_name(obligation.status)
        );
    }
}

fn write_diagnostics(output: &mut String, diagnostics: &TypeDiagnosticTable) {
    output.push_str("diagnostics:\n");
    if diagnostics.is_empty() {
        output.push_str("  <none>\n");
        return;
    }
    for (id, diagnostic) in diagnostics.canonical_iter() {
        let _ = write!(output, "  diagnostic#{} owner=", id.index());
        match &diagnostic.owner {
            Some(owner) => output.push_str(&site_key(owner)),
            None => output.push_str("<none>"),
        }
        output.push_str(" range=");
        write_range(output, diagnostic.source_range);
        let _ = writeln!(
            output,
            " class={} severity={} message_key=\"{}\" recovery={}",
            diagnostic_class_name(diagnostic.class),
            diagnostic_severity_name(diagnostic.severity),
            escaped_display(&diagnostic.message_key),
            diagnostic_recovery_name(diagnostic.recovery)
        );
    }
}

fn write_node_links(output: &mut String, links: &TypedNodeLinks) {
    output.push('{');
    output.push_str("context=");
    match links.context {
        Some(id) => {
            let _ = write!(output, "context#{}", id.index());
        }
        None => output.push_str("<none>"),
    }
    output.push_str(" type=");
    match links.type_entry {
        Some(id) => {
            let _ = write!(output, "type#{}", id.index());
        }
        None => output.push_str("<none>"),
    }
    output.push_str(" facts=");
    write_fact_ids(output, &links.facts);
    output.push_str(" coercions=");
    write_coercion_ids(output, &links.coercions);
    output.push_str(" obligations=");
    write_obligation_ids(output, &links.initial_obligations);
    output.push_str(" diagnostics=");
    write_diagnostic_ids(output, &links.diagnostics);
    output.push('}');
}

fn write_bindings(output: &mut String, bindings: &[BindingTypeRef]) {
    output.push('[');
    for (index, binding) in bindings.iter().enumerate() {
        if index > 0 {
            output.push_str(", ");
        }
        match binding {
            BindingTypeRef::Symbol(symbol) => {
                output.push_str("symbol=");
                write_symbol_id(output, symbol);
            }
            BindingTypeRef::Site(site) => {
                output.push_str("site=");
                output.push_str(&site_key(site));
            }
        }
    }
    output.push(']');
}

fn write_type_actual(output: &mut String, actual: TypeEntryActual) {
    match actual {
        TypeEntryActual::Known(id) => write_type_id(output, id),
        TypeEntryActual::CandidateSet(id) => {
            let _ = write!(output, "candidate_set#{}", id.index());
        }
        TypeEntryActual::Absent => output.push_str("<absent>"),
    }
}

fn write_type_provenance(output: &mut String, provenance: &TypeProvenance) {
    match provenance {
        TypeProvenance::Declared(range) => {
            let _ = write!(output, "declared({}..{})", range.start, range.end);
        }
        TypeProvenance::Assumed(id) => {
            let _ = write!(output, "assumed(\"{}\")", escaped_display(id.as_str()));
        }
        TypeProvenance::Inferred(id) => {
            let _ = write!(output, "inferred(\"{}\")", escaped_display(id.as_str()));
        }
        TypeProvenance::Builtin(id) => {
            let _ = write!(output, "builtin(\"{}\")", escaped_display(id.as_str()));
        }
        TypeProvenance::Recovery(id) => {
            let _ = write!(output, "recovery(diagnostic#{})", id.index());
        }
    }
}

fn write_fact_provenance(output: &mut String, provenance: &FactProvenance) {
    match provenance {
        FactProvenance::Declared(range) => {
            let _ = write!(output, "declared({}..{})", range.start, range.end);
        }
        FactProvenance::Assumed(id) => {
            let _ = write!(output, "assumed(\"{}\")", escaped_display(id.as_str()));
        }
        FactProvenance::Inferred(id) => {
            let _ = write!(output, "inferred(\"{}\")", escaped_display(id.as_str()));
        }
        FactProvenance::Obligation(id) => {
            let _ = write!(output, "obligation#{}", id.index());
        }
        FactProvenance::Builtin(id) => {
            let _ = write!(output, "builtin(\"{}\")", escaped_display(id.as_str()));
        }
        FactProvenance::Registration(id) => {
            let _ = write!(output, "registration(\"{}\")", escaped_display(id.as_str()));
        }
    }
}

fn write_coercion_provenance(output: &mut String, provenance: &CoercionProvenance) {
    match provenance {
        CoercionProvenance::WideningRule(id) => {
            let _ = write!(
                output,
                "widening_rule(\"{}\")",
                escaped_display(id.as_str())
            );
        }
        CoercionProvenance::NarrowingClaim(range) => {
            let _ = write!(output, "narrowing_claim({}..{})", range.start, range.end);
        }
        CoercionProvenance::SourceQua(range) => {
            let _ = write!(output, "source_qua({}..{})", range.start, range.end);
        }
        CoercionProvenance::Recovery(id) => {
            let _ = write!(output, "recovery(diagnostic#{})", id.index());
        }
    }
}

fn write_optional_node_id(output: &mut String, id: Option<TypedNodeId>) {
    match id {
        Some(id) => {
            let _ = write!(output, "node#{}", id.index());
        }
        None => output.push_str("<none>"),
    }
}

fn write_optional_resolved_node_id(output: &mut String, id: Option<ResolvedNodeId>) {
    match id {
        Some(id) => {
            let _ = write!(output, "resolved#{}", id.index());
        }
        None => output.push_str("<none>"),
    }
}

fn write_node_ids(output: &mut String, ids: &[TypedNodeId]) {
    output.push('[');
    for (index, id) in ids.iter().enumerate() {
        if index > 0 {
            output.push_str(", ");
        }
        let _ = write!(output, "node#{}", id.index());
    }
    output.push(']');
}

fn write_fact_ids(output: &mut String, ids: &[TypeFactId]) {
    write_prefixed_ids(output, "fact#", ids.iter().map(|id| id.index()));
}

fn write_coercion_ids(output: &mut String, ids: &[CoercionId]) {
    write_prefixed_ids(output, "coercion#", ids.iter().map(|id| id.index()));
}

fn write_obligation_ids(output: &mut String, ids: &[InitialObligationId]) {
    write_prefixed_ids(output, "obligation#", ids.iter().map(|id| id.index()));
}

fn write_diagnostic_ids(output: &mut String, ids: &[TypeDiagnosticId]) {
    write_prefixed_ids(output, "diagnostic#", ids.iter().map(|id| id.index()));
}

fn write_prefixed_ids(output: &mut String, prefix: &str, ids: impl Iterator<Item = usize>) {
    output.push('[');
    for (index, id) in ids.enumerate() {
        if index > 0 {
            output.push_str(", ");
        }
        let _ = write!(output, "{prefix}{id}");
    }
    output.push(']');
}

fn write_optional_type_id(output: &mut String, id: Option<NormalizedTypeId>) {
    match id {
        Some(id) => write_type_id(output, id),
        None => output.push_str("<none>"),
    }
}

fn write_type_id(output: &mut String, id: NormalizedTypeId) {
    let _ = write!(output, "normalized_type#{}", id.index());
}

fn write_optional_obligation_id(output: &mut String, id: Option<InitialObligationId>) {
    match id {
        Some(id) => {
            let _ = write!(output, "obligation#{}", id.index());
        }
        None => output.push_str("<none>"),
    }
}

fn write_anchor(output: &mut String, anchor: &SourceAnchor) {
    match anchor {
        SourceAnchor::Range(range) => write_range(output, *range),
        SourceAnchor::Point { offset, .. } => {
            let _ = write!(output, "point({offset})");
        }
        SourceAnchor::Generated(origin) => {
            output.push_str("generated(");
            write_generated_anchor(output, origin.anchor());
            output.push_str(", reason=\"");
            write_escaped(output, origin.reason());
            output.push_str("\")");
        }
        _ => output.push_str("<unknown>"),
    }
}

fn write_generated_anchor(output: &mut String, anchor: GeneratedSpanAnchor) {
    match anchor {
        GeneratedSpanAnchor::Range(range) => write_range(output, range),
        GeneratedSpanAnchor::Point { offset, .. } => {
            let _ = write!(output, "point({offset})");
        }
        _ => output.push_str("<unknown>"),
    }
}

fn write_range(output: &mut String, range: SourceRange) {
    let _ = write!(output, "{}..{}", range.start, range.end);
}

fn write_module_id(output: &mut String, module: &ModuleId) {
    write_escaped(output, module.package().as_str());
    output.push_str("::");
    write_escaped(output, module.path().as_str());
}

fn write_symbol_id(output: &mut String, symbol: &SymbolId) {
    output.push_str("{fqn=\"");
    write_escaped(output, symbol.fqn().as_str());
    output.push_str("\" module=");
    write_module_id(output, symbol.module());
    output.push_str(" local=\"");
    write_escaped(output, symbol.local().as_str());
    output.push_str("\"}");
}

fn escaped_display(value: &str) -> String {
    let mut escaped = String::new();
    write_escaped(&mut escaped, value);
    escaped
}

fn write_escaped(output: &mut String, value: &str) {
    for character in value.chars() {
        match character {
            '\\' => output.push_str("\\\\"),
            '"' => output.push_str("\\\""),
            '\n' => output.push_str("\\n"),
            '\r' => output.push_str("\\r"),
            '\t' => output.push_str("\\t"),
            character if character.is_control() => {
                let _ = write!(output, "\\u{{{:x}}}", character as u32);
            }
            character => output.push(character),
        }
    }
}

fn typing_state_name(state: TypingState) -> &'static str {
    match state {
        TypingState::Successful => "successful",
        TypingState::Assumed => "assumed",
        TypingState::Unknown => "unknown",
        TypingState::Error => "error",
        TypingState::Skipped => "skipped",
    }
}

fn node_recovery_state_name(state: NodeRecoveryState) -> &'static str {
    match state {
        NodeRecoveryState::Normal => "normal",
        NodeRecoveryState::Recovered => "recovered",
        NodeRecoveryState::Degraded => "degraded",
    }
}

fn context_layer_name(layer: TypeContextLayer) -> &'static str {
    match layer {
        TypeContextLayer::Module => "module",
        TypeContextLayer::Declaration => "declaration",
        TypeContextLayer::Proof => "proof",
        TypeContextLayer::Block => "block",
        TypeContextLayer::Expression => "expression",
    }
}

fn context_recovery_name(recovery: ContextRecoveryState) -> &'static str {
    match recovery {
        ContextRecoveryState::Normal => "normal",
        ContextRecoveryState::Recovered => "recovered",
        ContextRecoveryState::Degraded => "degraded",
    }
}

fn type_status_name(status: TypeStatus) -> &'static str {
    match status {
        TypeStatus::Known => "known",
        TypeStatus::Assumed => "assumed",
        TypeStatus::Unknown => "unknown",
        TypeStatus::Error => "error",
        TypeStatus::Skipped => "skipped",
    }
}

fn polarity_name(polarity: Polarity) -> &'static str {
    match polarity {
        Polarity::Positive => "positive",
        Polarity::Negative => "negative",
    }
}

fn fact_status_name(status: FactStatus) -> &'static str {
    match status {
        FactStatus::Known => "known",
        FactStatus::Assumed => "assumed",
        FactStatus::PendingObligation => "pending_obligation",
        FactStatus::Degraded => "degraded",
        FactStatus::Rejected => "rejected",
    }
}

fn coercion_kind_name(kind: CoercionKind) -> &'static str {
    match kind {
        CoercionKind::Widening => "widening",
        CoercionKind::Narrowing => "narrowing",
        CoercionKind::SourceQua => "source_qua",
    }
}

fn coercion_status_name(status: CoercionStatus) -> &'static str {
    match status {
        CoercionStatus::Candidate => "candidate",
        CoercionStatus::RequiresObligation => "requires_obligation",
        CoercionStatus::Blocked => "blocked",
        CoercionStatus::Rejected => "rejected",
    }
}

fn initial_obligation_kind_name(kind: InitialObligationKind) -> &'static str {
    match kind {
        InitialObligationKind::Sethood => "sethood",
        InitialObligationKind::NonEmptiness => "non_emptiness",
        InitialObligationKind::Narrowing => "narrowing",
        InitialObligationKind::RegistrationCorrectness => "registration_correctness",
    }
}

fn initial_obligation_status_name(status: InitialObligationStatus) -> &'static str {
    match status {
        InitialObligationStatus::Pending => "pending",
        InitialObligationStatus::Blocked => "blocked",
        InitialObligationStatus::Invalidated => "invalidated",
    }
}

fn diagnostic_class_name(class: TypeDiagnosticClass) -> &'static str {
    match class {
        TypeDiagnosticClass::TypeExpression => "type_expression",
        TypeDiagnosticClass::TypeEntry => "type_entry",
        TypeDiagnosticClass::TypeFact => "type_fact",
        TypeDiagnosticClass::Coercion => "coercion",
        TypeDiagnosticClass::InitialObligation => "initial_obligation",
        TypeDiagnosticClass::Context => "context",
        TypeDiagnosticClass::Recovery => "recovery",
    }
}

fn diagnostic_severity_name(severity: TypeDiagnosticSeverity) -> &'static str {
    match severity {
        TypeDiagnosticSeverity::Error => "error",
        TypeDiagnosticSeverity::Warning => "warning",
        TypeDiagnosticSeverity::Note => "note",
    }
}

fn diagnostic_recovery_name(recovery: DiagnosticRecoveryState) -> &'static str {
    match recovery {
        DiagnosticRecoveryState::Normal => "normal",
        DiagnosticRecoveryState::Recovery => "recovery",
        DiagnosticRecoveryState::Degraded => "degraded",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mizar_resolve::resolved_ast::{FullyQualifiedName, LocalSymbolId};
    use mizar_session::{
        BuildSnapshotId, InMemorySessionIdAllocator, ModulePath, PackageId, SessionIdAllocator,
    };

    #[test]
    fn arena_ids_are_dense_and_debug_rendering_is_stable() {
        let first = typed_ast_fixture();
        let second = typed_ast_fixture();

        assert_eq!(first.debug_text(), second.debug_text());
        assert_eq!(
            first.debug_text(),
            r#"typed-ast-debug-v1
module: pkg::main
root: node#1
resolved_root: <none>
nodes:
  node#0 kind="TermReference" children=[] resolved=<none> anchor=0..1 typing=successful recovery=normal links={context=<none> type=<none> facts=[] coercions=[] obligations=[] diagnostics=[]}
  node#1 kind="CompilationUnit" children=[node#0] resolved=<none> anchor=0..1 typing=successful recovery=normal links={context=<none> type=<none> facts=[] coercions=[] obligations=[] diagnostics=[]}
contexts:
  context#0 owner=node#0 parent=<none> layer=module recovery=normal bindings=[symbol={fqn="pkg::main::x/0" module=pkg::main local="x/0"}] introduced=[] visible=[fact#0]
types:
  type#0 owner=node#0 status=known actual=normalized_type#0 expected=<none> provenance=inferred("term-reference")
facts:
  fact#0 subject=node#0 predicate="set" polarity=positive status=known provenance=inferred("fixture")
coercions:
  coercion#0 site=node#0 kind=widening status=candidate from=normalized_type#0 to=normalized_type#1 facts=[fact#0] obligation=<none> provenance=widening_rule("widen")
initial_obligations:
  obligation#0 kind=sethood owner=node#0 range=0..1 assumptions=[fact#0] goal="term is set" provenance="sethood" status=pending
diagnostics:
  diagnostic#0 owner=node#0 range=0..1 class=recovery severity=note message_key="checker.recovery.note" recovery=normal
"#
        );
        assert!(first.debug_text().starts_with("typed-ast-debug-v1\n"));
        assert!(first.debug_text().contains("root: node#1\n"));
        assert!(first.debug_text().contains("kind=\"CompilationUnit\""));
        assert!(!first.debug_text().contains("SourceId"));
        assert!(!first.debug_text().contains(concat!("V", "cId")));
        assert!(!first.debug_text().contains(concat!("Obligation", "Anchor")));
        assert!(!first.debug_text().contains("proof witness"));
        assert!(
            !first
                .debug_text()
                .contains(concat!("active", "_refinement"))
        );
        assert!(!first.debug_text().contains(concat!("overload", "_root")));
    }

    #[test]
    fn arena_validation_rejects_invalid_references_and_cycles() {
        let source = source_id();
        assert!(matches!(
            TypedArena::try_new(
                Some(TypedNodeId::new(1)),
                vec![TypedNode::new(
                    "CompilationUnit",
                    SourceAnchor::Range(range(source, 0, 1)),
                )],
            ),
            Err(TypedArenaError::InvalidRoot { root }) if root == TypedNodeId::new(1)
        ));

        assert!(matches!(
            TypedArena::try_new(
                None,
                vec![TypedNode::new(
                    "CompilationUnit",
                    SourceAnchor::Range(range(source, 0, 1)),
                )
                .with_children(vec![TypedNodeId::new(1)])],
            ),
            Err(TypedArenaError::InvalidChild { node, child })
                if node == TypedNodeId::new(0) && child == TypedNodeId::new(1)
        ));

        assert!(matches!(
            TypedArena::try_new(
                None,
                vec![TypedNode::new(
                    "CompilationUnit",
                    SourceAnchor::Range(range(source, 0, 1)),
                )
                .with_children(vec![TypedNodeId::new(0)])],
            ),
            Err(TypedArenaError::Cycle { node }) if node == TypedNodeId::new(0)
        ));

        let mut builder = TypedArenaBuilder::new();
        assert!(matches!(
            builder.push(
                TypedNode::new("CompilationUnit", SourceAnchor::Range(range(source, 0, 1)))
                    .with_children(vec![TypedNodeId::new(0)]),
            ),
            Err(TypedArenaError::InvalidChild { node, child })
                if node == TypedNodeId::new(0) && child == TypedNodeId::new(0)
        ));
    }

    #[test]
    fn tables_round_trip_ids_and_deduplicate_facts() {
        let source = source_id();
        let root = single_node_arena(source);
        let owner = TypedSiteRef::Node(root.root().unwrap());
        let mut facts = TypeFactTable::new();
        let fact = facts.insert(fact_draft(owner.clone(), FactStatus::Known));
        let duplicate = facts.insert(fact_draft(owner.clone(), FactStatus::Known));
        assert_eq!(fact, duplicate);
        assert_eq!(facts.len(), 1);

        let mut types = TypeTable::new();
        let type_id = types.insert(TypeEntryDraft {
            owner: owner.clone(),
            expected: None,
            actual: TypeEntryActual::Known(NormalizedTypeId::new(0)),
            status: TypeStatus::Known,
            provenance: TypeProvenance::Declared(range(source, 0, 1).into()),
        });

        let mut obligations = InitialObligationTable::new();
        let obligation = obligations.insert(InitialObligationDraft {
            kind: InitialObligationKind::Narrowing,
            owner: owner.clone(),
            source_range: range(source, 0, 1),
            assumptions: vec![fact],
            goal: InitialObligationGoal::new("x is set"),
            provenance: InitialObligationProvenance::new("reconsider"),
            status: InitialObligationStatus::Pending,
        });

        let mut coercions = CoercionTable::new();
        let coercion = coercions.insert(CoercionDraft {
            site: owner.clone(),
            from: Some(NormalizedTypeId::new(1)),
            to: NormalizedTypeId::new(2),
            kind: CoercionKind::Narrowing,
            status: CoercionStatus::RequiresObligation,
            supporting_facts: vec![fact],
            obligation: Some(obligation),
            provenance: CoercionProvenance::NarrowingClaim(range(source, 0, 1).into()),
        });

        let mut diagnostics = TypeDiagnosticTable::new();
        let diagnostic = diagnostics.insert(TypeDiagnosticDraft {
            owner: Some(owner),
            source_range: range(source, 0, 1),
            class: TypeDiagnosticClass::Coercion,
            severity: TypeDiagnosticSeverity::Note,
            message_key: "checker.coercion.note".to_owned(),
            recovery: DiagnosticRecoveryState::Normal,
        });

        assert_eq!(types.get(type_id).unwrap().id, type_id);
        assert_eq!(facts.get(fact).unwrap().id, fact);
        assert_eq!(coercions.get(coercion).unwrap().id, coercion);
        assert_eq!(obligations.get(obligation).unwrap().id, obligation);
        assert_eq!(diagnostics.get(diagnostic).unwrap().id, diagnostic);
    }

    #[test]
    fn context_validation_enforces_assumed_fact_visibility() {
        let source = source_id();
        let nodes = single_node_arena(source);
        let owner = TypedSiteRef::Node(nodes.root().unwrap());
        let mut facts = TypeFactTable::new();
        let assumed = facts.insert(fact_draft(owner.clone(), FactStatus::Assumed));
        let mut contexts = LocalTypeContextTable::new();
        let context = contexts.insert(LocalTypeContextDraft {
            owner: owner.clone(),
            parent: None,
            layer: TypeContextLayer::Block,
            bindings: Vec::new(),
            introduced_assumptions: vec![assumed],
            visible_facts: vec![assumed],
            recovery: ContextRecoveryState::Normal,
        });

        let ast = TypedAst::try_new(parts_with(source, nodes.clone(), contexts, facts.clone()));
        assert!(ast.is_ok());

        let mut bad_contexts = LocalTypeContextTable::new();
        bad_contexts.insert(LocalTypeContextDraft {
            owner,
            parent: None,
            layer: TypeContextLayer::Block,
            bindings: Vec::new(),
            introduced_assumptions: Vec::new(),
            visible_facts: vec![assumed],
            recovery: ContextRecoveryState::Normal,
        });
        assert!(matches!(
            TypedAst::try_new(parts_with(source, nodes, bad_contexts, facts)),
            Err(TypedAstError::AssumptionFactNotIntroduced { context: failed, fact })
                if failed == context && fact == assumed
        ));
    }

    #[test]
    fn local_context_snapshots_validate_parent_chain_and_visibility() {
        let source = source_id();
        let nodes = single_node_arena(source);
        let owner = TypedSiteRef::Node(nodes.root().unwrap());
        let mut facts = TypeFactTable::new();
        let known = facts.insert(fact_draft_with(
            owner.clone(),
            "known",
            FactStatus::Known,
            FactProvenance::Inferred(TypeRuleId::new("known")),
        ));
        let assumed = facts.insert(fact_draft_with(
            owner.clone(),
            "assumed",
            FactStatus::Assumed,
            FactProvenance::Assumed(TypeAssumptionId::new("A1")),
        ));

        let mut contexts = LocalTypeContextTable::new();
        let parent = contexts.insert(LocalTypeContextDraft {
            owner: owner.clone(),
            parent: None,
            layer: TypeContextLayer::Declaration,
            bindings: vec![BindingTypeRef::Symbol(symbol_id())],
            introduced_assumptions: vec![assumed],
            visible_facts: vec![known],
            recovery: ContextRecoveryState::Normal,
        });
        let child = contexts.insert(LocalTypeContextDraft {
            owner: owner.clone(),
            parent: Some(parent),
            layer: TypeContextLayer::Block,
            bindings: vec![BindingTypeRef::Site(owner.clone())],
            introduced_assumptions: Vec::new(),
            visible_facts: vec![assumed],
            recovery: ContextRecoveryState::Recovered,
        });

        assert_eq!(
            contexts.iter().map(|(id, _)| id).collect::<Vec<_>>(),
            vec![parent, child]
        );

        let ast = TypedAst::try_new(parts_with(source, nodes.clone(), contexts, facts.clone()))
            .expect("child context can consume an assumption introduced by its parent");
        let debug = ast.debug_text();
        assert!(debug.contains("context#1 owner=node#0 parent=context#0"));
        assert!(debug.contains("recovery=recovered"));
        assert!(debug.contains("introduced=[fact#1]"));
        assert!(debug.contains("visible=[fact#1]"));

        let mut missing_parent = LocalTypeContextTable::new();
        let failed = missing_parent.insert(LocalTypeContextDraft {
            owner: owner.clone(),
            parent: Some(LocalTypeContextId::new(99)),
            layer: TypeContextLayer::Block,
            bindings: Vec::new(),
            introduced_assumptions: Vec::new(),
            visible_facts: Vec::new(),
            recovery: ContextRecoveryState::Normal,
        });
        assert!(matches!(
            TypedAst::try_new(parts_with(source, nodes.clone(), missing_parent, facts.clone())),
            Err(TypedAstError::InvalidContextParent { context, parent })
                if context == failed && parent == LocalTypeContextId::new(99)
        ));

        let mut cyclic = LocalTypeContextTable::new();
        let first = cyclic.insert(LocalTypeContextDraft {
            owner: owner.clone(),
            parent: Some(LocalTypeContextId::new(1)),
            layer: TypeContextLayer::Block,
            bindings: Vec::new(),
            introduced_assumptions: Vec::new(),
            visible_facts: Vec::new(),
            recovery: ContextRecoveryState::Normal,
        });
        cyclic.insert(LocalTypeContextDraft {
            owner: owner.clone(),
            parent: Some(first),
            layer: TypeContextLayer::Block,
            bindings: Vec::new(),
            introduced_assumptions: Vec::new(),
            visible_facts: Vec::new(),
            recovery: ContextRecoveryState::Normal,
        });
        assert!(matches!(
            TypedAst::try_new(parts_with(source, nodes.clone(), cyclic, facts.clone())),
            Err(TypedAstError::ContextCycle { context }) if context == first
        ));

        let mut unsorted = LocalTypeContextTable::new();
        let unsorted_context = unsorted.insert(LocalTypeContextDraft {
            owner: owner.clone(),
            parent: None,
            layer: TypeContextLayer::Block,
            bindings: Vec::new(),
            introduced_assumptions: vec![assumed, known],
            visible_facts: Vec::new(),
            recovery: ContextRecoveryState::Normal,
        });
        assert!(matches!(
            TypedAst::try_new(parts_with(source, nodes.clone(), unsorted, facts.clone())),
            Err(TypedAstError::UnsortedContextFacts { context })
                if context == unsorted_context
        ));

        for status in [
            FactStatus::PendingObligation,
            FactStatus::Degraded,
            FactStatus::Rejected,
        ] {
            let mut status_facts = TypeFactTable::new();
            let fact = status_facts.insert(fact_draft_with(
                owner.clone(),
                format!("{status:?}"),
                status,
                FactProvenance::Inferred(TypeRuleId::new(format!("{status:?}"))),
            ));
            let mut status_contexts = LocalTypeContextTable::new();
            let context = status_contexts.insert(LocalTypeContextDraft {
                owner: owner.clone(),
                parent: None,
                layer: TypeContextLayer::Block,
                bindings: Vec::new(),
                introduced_assumptions: Vec::new(),
                visible_facts: vec![fact],
                recovery: ContextRecoveryState::Normal,
            });
            assert!(matches!(
                TypedAst::try_new(parts_with(
                    source,
                    nodes.clone(),
                    status_contexts,
                    status_facts,
                )),
                Err(TypedAstError::AssumptionFactNotIntroduced {
                    context: failed,
                    fact: failed_fact,
                }) if failed == context && failed_fact == fact
            ));
        }

        let mut non_assumed_contexts = LocalTypeContextTable::new();
        let context = non_assumed_contexts.insert(LocalTypeContextDraft {
            owner,
            parent: None,
            layer: TypeContextLayer::Block,
            bindings: Vec::new(),
            introduced_assumptions: vec![known],
            visible_facts: Vec::new(),
            recovery: ContextRecoveryState::Normal,
        });
        assert!(matches!(
            TypedAst::try_new(parts_with(source, nodes, non_assumed_contexts, facts)),
            Err(TypedAstError::IntroducedFactNotAssumed {
                context: failed,
                fact: failed_fact,
            }) if failed == context && failed_fact == known
        ));
    }

    #[test]
    fn validation_rejects_missing_obligations_for_requires_obligation_coercions() {
        let source = source_id();
        let nodes = single_node_arena(source);
        let owner = TypedSiteRef::Node(nodes.root().unwrap());
        let facts = TypeFactTable::new();
        let contexts = LocalTypeContextTable::new();
        let mut coercions = CoercionTable::new();
        let coercion = coercions.insert(CoercionDraft {
            site: owner,
            from: Some(NormalizedTypeId::new(0)),
            to: NormalizedTypeId::new(1),
            kind: CoercionKind::Narrowing,
            status: CoercionStatus::RequiresObligation,
            supporting_facts: Vec::new(),
            obligation: None,
            provenance: CoercionProvenance::NarrowingClaim(range(source, 0, 1).into()),
        });

        let mut parts = parts_with(source, nodes, contexts, facts);
        parts.coercions = coercions;

        assert!(matches!(
            TypedAst::try_new(parts),
            Err(TypedAstError::MissingCoercionObligation { coercion: failed })
                if failed == coercion
        ));
    }

    #[test]
    fn validation_rejects_invalid_type_actual_and_provenance_links() {
        let source = source_id();
        let nodes = single_node_arena(source);
        let owner = TypedSiteRef::Node(nodes.root().unwrap());

        let mut missing_actual = TypeTable::new();
        let entry = missing_actual.insert(TypeEntryDraft {
            owner: owner.clone(),
            expected: None,
            actual: TypeEntryActual::Absent,
            status: TypeStatus::Known,
            provenance: TypeProvenance::Inferred(TypeRuleId::new("invalid")),
        });
        let mut parts = parts_with(
            source,
            nodes.clone(),
            LocalTypeContextTable::new(),
            TypeFactTable::new(),
        );
        parts.types = missing_actual;
        assert!(matches!(
            TypedAst::try_new(parts),
            Err(TypedAstError::InvalidTypeActual { entry: failed }) if failed == entry
        ));

        let mut missing_type_diagnostic = TypeTable::new();
        let entry = missing_type_diagnostic.insert(TypeEntryDraft {
            owner: owner.clone(),
            expected: None,
            actual: TypeEntryActual::Known(NormalizedTypeId::new(0)),
            status: TypeStatus::Known,
            provenance: TypeProvenance::Recovery(TypeDiagnosticId::new(0)),
        });
        let mut parts = parts_with(
            source,
            nodes.clone(),
            LocalTypeContextTable::new(),
            TypeFactTable::new(),
        );
        parts.types = missing_type_diagnostic;
        assert!(matches!(
            TypedAst::try_new(parts),
            Err(TypedAstError::InvalidTypeRecoveryDiagnostic {
                entry: failed,
                diagnostic,
            }) if failed == entry && diagnostic == TypeDiagnosticId::new(0)
        ));

        let mut missing_fact_obligation = TypeFactTable::new();
        let fact = missing_fact_obligation.insert(fact_draft_with(
            owner.clone(),
            "obligation-backed",
            FactStatus::Known,
            FactProvenance::Obligation(InitialObligationId::new(0)),
        ));
        assert!(matches!(
            TypedAst::try_new(parts_with(
                source,
                nodes.clone(),
                LocalTypeContextTable::new(),
                missing_fact_obligation,
            )),
            Err(TypedAstError::InvalidFactObligation {
                fact: failed,
                obligation,
            }) if failed == fact && obligation == InitialObligationId::new(0)
        ));

        let mut missing_coercion_diagnostic = CoercionTable::new();
        let coercion = missing_coercion_diagnostic.insert(CoercionDraft {
            site: owner,
            from: Some(NormalizedTypeId::new(0)),
            to: NormalizedTypeId::new(1),
            kind: CoercionKind::Widening,
            status: CoercionStatus::Blocked,
            supporting_facts: Vec::new(),
            obligation: None,
            provenance: CoercionProvenance::Recovery(TypeDiagnosticId::new(0)),
        });
        let mut parts = parts_with(
            source,
            nodes,
            LocalTypeContextTable::new(),
            TypeFactTable::new(),
        );
        parts.coercions = missing_coercion_diagnostic;
        assert!(matches!(
            TypedAst::try_new(parts),
            Err(TypedAstError::InvalidCoercionRecoveryDiagnostic {
                coercion: failed,
                diagnostic,
            }) if failed == coercion && diagnostic == TypeDiagnosticId::new(0)
        ));
    }

    #[test]
    fn status_variants_preserve_partial_typing_and_handoff_boundaries() {
        let source = source_id();
        let mut builder = TypedArenaBuilder::new();
        let unknown = builder
            .push(
                TypedNode::new("UnknownTerm", SourceAnchor::Range(range(source, 0, 1)))
                    .with_typing(TypingState::Unknown)
                    .with_recovery(NodeRecoveryState::Recovered),
            )
            .unwrap();
        let error = builder
            .push(
                TypedNode::new("ErroredTerm", SourceAnchor::Range(range(source, 1, 2)))
                    .with_typing(TypingState::Error)
                    .with_recovery(NodeRecoveryState::Degraded),
            )
            .unwrap();
        let skipped = builder
            .push(
                TypedNode::new("SkippedTerm", SourceAnchor::Range(range(source, 2, 3)))
                    .with_typing(TypingState::Skipped),
            )
            .unwrap();
        let root = builder
            .push(
                TypedNode::new("CompilationUnit", SourceAnchor::Range(range(source, 0, 3)))
                    .with_children(vec![unknown, error, skipped])
                    .with_typing(TypingState::Assumed),
            )
            .unwrap();
        let nodes = builder.finish(Some(root)).unwrap();
        let mut types = TypeTable::new();
        let owner = TypedSiteRef::Node(root);
        insert_type_entry(
            &mut types,
            TypedSiteRef::Role {
                node: root,
                role: TypeRole::new("known"),
            },
            TypeEntryActual::Known(NormalizedTypeId::new(0)),
            TypeStatus::Known,
        );
        insert_type_entry(
            &mut types,
            TypedSiteRef::Role {
                node: root,
                role: TypeRole::new("assumed"),
            },
            TypeEntryActual::CandidateSet(OpenCandidateSetId::new(0)),
            TypeStatus::Assumed,
        );
        insert_type_entry(
            &mut types,
            TypedSiteRef::Role {
                node: root,
                role: TypeRole::new("unknown"),
            },
            TypeEntryActual::Absent,
            TypeStatus::Unknown,
        );
        insert_type_entry(
            &mut types,
            TypedSiteRef::Role {
                node: root,
                role: TypeRole::new("error"),
            },
            TypeEntryActual::Absent,
            TypeStatus::Error,
        );
        insert_type_entry(
            &mut types,
            TypedSiteRef::Role {
                node: root,
                role: TypeRole::new("skipped"),
            },
            TypeEntryActual::Absent,
            TypeStatus::Skipped,
        );

        assert!(TypeStatus::Known.is_available_for_handoff());
        assert!(TypeStatus::Assumed.is_available_for_handoff());
        assert!(!TypeStatus::Unknown.is_available_for_handoff());
        assert!(!TypeStatus::Error.is_available_for_handoff());
        assert!(!TypeStatus::Skipped.is_available_for_handoff());
        assert!(FactStatus::Known.is_unconditionally_consumable());
        assert!(!FactStatus::Assumed.is_unconditionally_consumable());
        assert!(!FactStatus::PendingObligation.is_unconditionally_consumable());
        assert!(!FactStatus::Degraded.is_unconditionally_consumable());
        assert!(!FactStatus::Rejected.is_unconditionally_consumable());
        assert!(CoercionStatus::Candidate.is_available_for_handoff());
        assert!(CoercionStatus::RequiresObligation.is_available_for_handoff());
        assert!(!CoercionStatus::Blocked.is_available_for_handoff());
        assert!(!CoercionStatus::Rejected.is_available_for_handoff());
        assert!(InitialObligationStatus::Pending.is_available_for_handoff());
        assert!(!InitialObligationStatus::Blocked.is_available_for_handoff());
        assert!(!InitialObligationStatus::Invalidated.is_available_for_handoff());

        let mut facts = TypeFactTable::new();
        let fact = facts.insert(fact_draft_with(
            owner.clone(),
            "pending",
            FactStatus::PendingObligation,
            FactProvenance::Inferred(TypeRuleId::new("pending")),
        ));
        facts.insert(fact_draft_with(
            owner.clone(),
            "degraded",
            FactStatus::Degraded,
            FactProvenance::Builtin(BuiltinRuleId::new("degraded")),
        ));
        facts.insert(fact_draft_with(
            owner,
            "rejected",
            FactStatus::Rejected,
            FactProvenance::Registration(ResolutionStepId::new("rejected")),
        ));

        let mut obligations = InitialObligationTable::new();
        obligations.insert(InitialObligationDraft {
            kind: InitialObligationKind::Sethood,
            owner: TypedSiteRef::Node(root),
            source_range: range(source, 0, 1),
            assumptions: vec![fact],
            goal: InitialObligationGoal::new("blocked goal"),
            provenance: InitialObligationProvenance::new("blocked"),
            status: InitialObligationStatus::Blocked,
        });
        obligations.insert(InitialObligationDraft {
            kind: InitialObligationKind::RegistrationCorrectness,
            owner: TypedSiteRef::Node(root),
            source_range: range(source, 1, 2),
            assumptions: Vec::new(),
            goal: InitialObligationGoal::new("invalidated goal"),
            provenance: InitialObligationProvenance::new("invalidated"),
            status: InitialObligationStatus::Invalidated,
        });

        let mut parts = parts_with(source, nodes, LocalTypeContextTable::new(), facts);
        parts.types = types;
        parts.initial_obligations = obligations;

        let debug = TypedAst::try_new(parts).unwrap().debug_text();
        for expected in [
            "typing=unknown",
            "typing=error",
            "typing=skipped",
            "typing=assumed",
            "recovery=recovered",
            "recovery=degraded",
            "status=known",
            "status=assumed",
            "status=unknown",
            "status=error",
            "status=skipped",
            "status=pending_obligation",
            "status=degraded",
            "status=rejected",
            "status=blocked",
            "status=invalidated",
        ] {
            assert!(debug.contains(expected), "{expected}\n{debug}");
        }
    }

    #[test]
    fn canonical_queries_are_deterministic() {
        let source = source_id();
        let owner = TypedSiteRef::Node(TypedNodeId::new(0));
        let mut facts = TypeFactTable::new();
        let later = facts.insert(TypeFactDraft {
            subject: owner.clone(),
            predicate: TypePredicateRef::new("Z"),
            polarity: Polarity::Positive,
            provenance: FactProvenance::Inferred(TypeRuleId::new("z")),
            status: FactStatus::Known,
        });
        let earlier = facts.insert(TypeFactDraft {
            subject: owner.clone(),
            predicate: TypePredicateRef::new("A"),
            polarity: Polarity::Positive,
            provenance: FactProvenance::Inferred(TypeRuleId::new("a")),
            status: FactStatus::Known,
        });
        assert_eq!(
            facts.canonical_iter().map(|(id, _)| id).collect::<Vec<_>>(),
            vec![earlier, later]
        );

        let mut numeric_facts = TypeFactTable::new();
        let node_ten = numeric_facts.insert(TypeFactDraft {
            subject: TypedSiteRef::Node(TypedNodeId::new(10)),
            predicate: TypePredicateRef::new("same"),
            polarity: Polarity::Positive,
            provenance: FactProvenance::Inferred(TypeRuleId::new("same")),
            status: FactStatus::Known,
        });
        let role_zero = numeric_facts.insert(TypeFactDraft {
            subject: TypedSiteRef::Role {
                node: TypedNodeId::new(0),
                role: TypeRole::new("result"),
            },
            predicate: TypePredicateRef::new("same"),
            polarity: Polarity::Positive,
            provenance: FactProvenance::Inferred(TypeRuleId::new("same")),
            status: FactStatus::Known,
        });
        let node_two = numeric_facts.insert(TypeFactDraft {
            subject: TypedSiteRef::Node(TypedNodeId::new(2)),
            predicate: TypePredicateRef::new("same"),
            polarity: Polarity::Positive,
            provenance: FactProvenance::Inferred(TypeRuleId::new("same")),
            status: FactStatus::Known,
        });
        assert_eq!(
            numeric_facts
                .canonical_iter()
                .map(|(id, _)| id)
                .collect::<Vec<_>>(),
            vec![role_zero, node_two, node_ten]
        );

        let mut types = TypeTable::new();
        let node_ten = insert_type_entry(
            &mut types,
            TypedSiteRef::Node(TypedNodeId::new(10)),
            TypeEntryActual::Known(NormalizedTypeId::new(10)),
            TypeStatus::Known,
        );
        let role_zero = insert_type_entry(
            &mut types,
            TypedSiteRef::Role {
                node: TypedNodeId::new(0),
                role: TypeRole::new("result"),
            },
            TypeEntryActual::Known(NormalizedTypeId::new(0)),
            TypeStatus::Known,
        );
        let node_zero = insert_type_entry(
            &mut types,
            TypedSiteRef::Node(TypedNodeId::new(0)),
            TypeEntryActual::Known(NormalizedTypeId::new(1)),
            TypeStatus::Known,
        );
        assert_eq!(
            types.canonical_iter().map(|(id, _)| id).collect::<Vec<_>>(),
            vec![node_zero, role_zero, node_ten]
        );

        let mut coercions = CoercionTable::new();
        let node_ten = coercions.insert(CoercionDraft {
            site: TypedSiteRef::Node(TypedNodeId::new(10)),
            from: None,
            to: NormalizedTypeId::new(0),
            kind: CoercionKind::Widening,
            status: CoercionStatus::Candidate,
            supporting_facts: Vec::new(),
            obligation: None,
            provenance: CoercionProvenance::WideningRule(TypeRuleId::new("same")),
        });
        let role_zero = coercions.insert(CoercionDraft {
            site: TypedSiteRef::Role {
                node: TypedNodeId::new(0),
                role: TypeRole::new("result"),
            },
            from: None,
            to: NormalizedTypeId::new(0),
            kind: CoercionKind::Widening,
            status: CoercionStatus::Candidate,
            supporting_facts: Vec::new(),
            obligation: None,
            provenance: CoercionProvenance::WideningRule(TypeRuleId::new("same")),
        });
        let node_two = coercions.insert(CoercionDraft {
            site: TypedSiteRef::Node(TypedNodeId::new(2)),
            from: None,
            to: NormalizedTypeId::new(0),
            kind: CoercionKind::Widening,
            status: CoercionStatus::Candidate,
            supporting_facts: Vec::new(),
            obligation: None,
            provenance: CoercionProvenance::WideningRule(TypeRuleId::new("same")),
        });
        assert_eq!(
            coercions
                .canonical_iter()
                .map(|(id, _)| id)
                .collect::<Vec<_>>(),
            vec![role_zero, node_two, node_ten]
        );

        let mut diagnostics = TypeDiagnosticTable::new();
        let second = diagnostics.insert(TypeDiagnosticDraft {
            owner: Some(owner.clone()),
            source_range: range(source, 10, 11),
            class: TypeDiagnosticClass::Recovery,
            severity: TypeDiagnosticSeverity::Note,
            message_key: "z".to_owned(),
            recovery: DiagnosticRecoveryState::Recovery,
        });
        let first = diagnostics.insert(TypeDiagnosticDraft {
            owner: Some(owner),
            source_range: range(source, 0, 1),
            class: TypeDiagnosticClass::Recovery,
            severity: TypeDiagnosticSeverity::Note,
            message_key: "a".to_owned(),
            recovery: DiagnosticRecoveryState::Recovery,
        });
        assert_eq!(
            diagnostics
                .canonical_iter()
                .map(|(id, _)| id)
                .collect::<Vec<_>>(),
            vec![first, second]
        );
    }

    #[test]
    fn coercion_ordering_and_provenance_are_stable() {
        let source = source_id();
        let nodes = single_node_arena(source);
        let owner = TypedSiteRef::Node(nodes.root().unwrap());
        let mut facts = TypeFactTable::new();
        let fact_a = facts.insert(fact_draft_with(
            owner.clone(),
            "A",
            FactStatus::Known,
            FactProvenance::Declared(range(source, 0, 1).into()),
        ));
        let fact_b = facts.insert(fact_draft_with(
            owner.clone(),
            "B",
            FactStatus::Known,
            FactProvenance::Builtin(BuiltinRuleId::new("builtin")),
        ));

        let mut diagnostics = TypeDiagnosticTable::new();
        let diagnostic = diagnostics.insert(TypeDiagnosticDraft {
            owner: Some(owner.clone()),
            source_range: range(source, 3, 4),
            class: TypeDiagnosticClass::Coercion,
            severity: TypeDiagnosticSeverity::Warning,
            message_key: "checker.coercion.recovered".to_owned(),
            recovery: DiagnosticRecoveryState::Recovery,
        });

        let mut obligations = InitialObligationTable::new();
        let obligation = obligations.insert(InitialObligationDraft {
            kind: InitialObligationKind::Narrowing,
            owner: owner.clone(),
            source_range: range(source, 0, 1),
            assumptions: vec![fact_a],
            goal: InitialObligationGoal::new("narrowing goal"),
            provenance: InitialObligationProvenance::new("narrowing"),
            status: InitialObligationStatus::Pending,
        });

        let mut coercions = CoercionTable::new();
        let rejected = coercions.insert(CoercionDraft {
            site: owner.clone(),
            from: Some(NormalizedTypeId::new(2)),
            to: NormalizedTypeId::new(3),
            kind: CoercionKind::SourceQua,
            status: CoercionStatus::Rejected,
            supporting_facts: vec![fact_b],
            obligation: None,
            provenance: CoercionProvenance::SourceQua(range(source, 2, 3).into()),
        });
        let candidate = coercions.insert(CoercionDraft {
            site: owner.clone(),
            from: Some(NormalizedTypeId::new(3)),
            to: NormalizedTypeId::new(4),
            kind: CoercionKind::Widening,
            status: CoercionStatus::Candidate,
            supporting_facts: vec![fact_a, fact_b],
            obligation: None,
            provenance: CoercionProvenance::WideningRule(TypeRuleId::new("widen")),
        });
        let requires_obligation = coercions.insert(CoercionDraft {
            site: owner.clone(),
            from: Some(NormalizedTypeId::new(0)),
            to: NormalizedTypeId::new(1),
            kind: CoercionKind::Narrowing,
            status: CoercionStatus::RequiresObligation,
            supporting_facts: vec![fact_a],
            obligation: Some(obligation),
            provenance: CoercionProvenance::NarrowingClaim(range(source, 0, 1).into()),
        });
        let blocked = coercions.insert(CoercionDraft {
            site: owner,
            from: Some(NormalizedTypeId::new(1)),
            to: NormalizedTypeId::new(2),
            kind: CoercionKind::Widening,
            status: CoercionStatus::Blocked,
            supporting_facts: Vec::new(),
            obligation: None,
            provenance: CoercionProvenance::Recovery(diagnostic),
        });
        let tie_later = coercions.insert(CoercionDraft {
            site: TypedSiteRef::Node(nodes.root().unwrap()),
            from: Some(NormalizedTypeId::new(5)),
            to: NormalizedTypeId::new(6),
            kind: CoercionKind::Widening,
            status: CoercionStatus::Candidate,
            supporting_facts: vec![fact_a, fact_b],
            obligation: None,
            provenance: CoercionProvenance::WideningRule(TypeRuleId::new("tie")),
        });
        let tie_earlier = coercions.insert(CoercionDraft {
            site: TypedSiteRef::Node(nodes.root().unwrap()),
            from: Some(NormalizedTypeId::new(5)),
            to: NormalizedTypeId::new(6),
            kind: CoercionKind::Widening,
            status: CoercionStatus::Candidate,
            supporting_facts: vec![fact_a],
            obligation: None,
            provenance: CoercionProvenance::WideningRule(TypeRuleId::new("tie")),
        });
        let target_later = coercions.insert(CoercionDraft {
            site: TypedSiteRef::Node(nodes.root().unwrap()),
            from: Some(NormalizedTypeId::new(0)),
            to: NormalizedTypeId::new(10),
            kind: CoercionKind::Widening,
            status: CoercionStatus::Candidate,
            supporting_facts: Vec::new(),
            obligation: None,
            provenance: CoercionProvenance::WideningRule(TypeRuleId::new("target")),
        });
        let target_earlier = coercions.insert(CoercionDraft {
            site: TypedSiteRef::Node(nodes.root().unwrap()),
            from: Some(NormalizedTypeId::new(99)),
            to: NormalizedTypeId::new(2),
            kind: CoercionKind::Widening,
            status: CoercionStatus::Candidate,
            supporting_facts: Vec::new(),
            obligation: None,
            provenance: CoercionProvenance::WideningRule(TypeRuleId::new("target")),
        });

        assert_eq!(
            coercions
                .canonical_iter()
                .map(|(id, _)| id)
                .collect::<Vec<_>>(),
            vec![
                target_earlier,
                blocked,
                candidate,
                tie_earlier,
                tie_later,
                target_later,
                requires_obligation,
                rejected,
            ]
        );

        let mut parts = parts_with(source, nodes, LocalTypeContextTable::new(), facts);
        parts.coercions = coercions;
        parts.initial_obligations = obligations;
        parts.diagnostics = diagnostics;

        let debug = TypedAst::try_new(parts).unwrap().debug_text();
        for expected in [
            "coercion#0",
            "status=rejected",
            "source_qua(2..3)",
            "coercion#1",
            "status=candidate",
            "widening_rule(\"widen\")",
            "facts=[fact#0, fact#1]",
            "coercion#2",
            "status=requires_obligation",
            "obligation=obligation#0",
            "narrowing_claim(0..1)",
            "coercion#3",
            "status=blocked",
            "recovery(diagnostic#0)",
        ] {
            assert!(debug.contains(expected), "{expected}\n{debug}");
        }
    }

    #[test]
    fn public_data_shapes_do_not_expose_proof_or_final_overload_fields() {
        let source = include_str!("typed_ast.rs");
        for forbidden in [
            concat!("V", "cId"),
            concat!("Obligation", "Anchor"),
            concat!("Proof", "Witness"),
            concat!("prover", "_result"),
            concat!("accepted", "_verifier", "_status"),
            concat!("overload", "_root"),
            concat!("active", "_refinement"),
            concat!("inserted", "_qua"),
        ] {
            assert!(
                !source.contains(forbidden),
                "typed_ast data shape must not expose {forbidden}"
            );
        }
    }

    fn typed_ast_fixture() -> TypedAst {
        let source = source_id();
        let mut builder = TypedArenaBuilder::new();
        let term = builder
            .push(
                TypedNode::new("TermReference", SourceAnchor::Range(range(source, 0, 1)))
                    .with_typing(TypingState::Successful),
            )
            .unwrap();
        let root = builder
            .push(
                TypedNode::new("CompilationUnit", SourceAnchor::Range(range(source, 0, 1)))
                    .with_children(vec![term])
                    .with_typing(TypingState::Successful),
            )
            .unwrap();
        let nodes = builder.finish(Some(root)).unwrap();
        let owner = TypedSiteRef::Node(term);

        let mut facts = TypeFactTable::new();
        let fact = facts.insert(fact_draft(owner.clone(), FactStatus::Known));

        let mut contexts = LocalTypeContextTable::new();
        contexts.insert(LocalTypeContextDraft {
            owner: owner.clone(),
            parent: None,
            layer: TypeContextLayer::Module,
            bindings: vec![BindingTypeRef::Symbol(symbol_id())],
            introduced_assumptions: Vec::new(),
            visible_facts: vec![fact],
            recovery: ContextRecoveryState::Normal,
        });

        let mut types = TypeTable::new();
        types.insert(TypeEntryDraft {
            owner: owner.clone(),
            expected: None,
            actual: TypeEntryActual::Known(NormalizedTypeId::new(0)),
            status: TypeStatus::Known,
            provenance: TypeProvenance::Inferred(TypeRuleId::new("term-reference")),
        });

        let mut obligations = InitialObligationTable::new();
        let obligation = obligations.insert(InitialObligationDraft {
            kind: InitialObligationKind::Sethood,
            owner: owner.clone(),
            source_range: range(source, 0, 1),
            assumptions: vec![fact],
            goal: InitialObligationGoal::new("term is set"),
            provenance: InitialObligationProvenance::new("sethood"),
            status: InitialObligationStatus::Pending,
        });

        let mut coercions = CoercionTable::new();
        coercions.insert(CoercionDraft {
            site: owner.clone(),
            from: Some(NormalizedTypeId::new(0)),
            to: NormalizedTypeId::new(1),
            kind: CoercionKind::Widening,
            status: CoercionStatus::Candidate,
            supporting_facts: vec![fact],
            obligation: None,
            provenance: CoercionProvenance::WideningRule(TypeRuleId::new("widen")),
        });

        let mut diagnostics = TypeDiagnosticTable::new();
        diagnostics.insert(TypeDiagnosticDraft {
            owner: Some(owner),
            source_range: range(source, 0, 1),
            class: TypeDiagnosticClass::Recovery,
            severity: TypeDiagnosticSeverity::Note,
            message_key: "checker.recovery.note".to_owned(),
            recovery: DiagnosticRecoveryState::Normal,
        });

        TypedAst::try_new(TypedAstParts {
            source_id: source,
            module_id: module_id(),
            resolved_root: None,
            source_context: None,
            source_type: None,
            source_attribute: None,
            nodes,
            contexts,
            types,
            facts,
            coercions,
            initial_obligations: obligations_with_node_link(obligations, obligation),
            diagnostics,
        })
        .unwrap()
    }

    fn obligations_with_node_link(
        obligations: InitialObligationTable,
        _obligation: InitialObligationId,
    ) -> InitialObligationTable {
        obligations
    }

    fn single_node_arena(source: SourceId) -> TypedArena {
        let mut builder = TypedArenaBuilder::new();
        let root = builder
            .push(TypedNode::new(
                "CompilationUnit",
                SourceAnchor::Range(range(source, 0, 1)),
            ))
            .unwrap();
        builder.finish(Some(root)).unwrap()
    }

    fn parts_with(
        source: SourceId,
        nodes: TypedArena,
        contexts: LocalTypeContextTable,
        facts: TypeFactTable,
    ) -> TypedAstParts {
        TypedAstParts {
            source_id: source,
            module_id: module_id(),
            resolved_root: None,
            source_context: None,
            source_type: None,
            source_attribute: None,
            nodes,
            contexts,
            types: TypeTable::new(),
            facts,
            coercions: CoercionTable::new(),
            initial_obligations: InitialObligationTable::new(),
            diagnostics: TypeDiagnosticTable::new(),
        }
    }

    fn fact_draft(owner: TypedSiteRef, status: FactStatus) -> TypeFactDraft {
        fact_draft_with(
            owner,
            "set",
            status,
            FactProvenance::Inferred(TypeRuleId::new("fixture")),
        )
    }

    fn fact_draft_with(
        owner: TypedSiteRef,
        predicate: impl Into<String>,
        status: FactStatus,
        provenance: FactProvenance,
    ) -> TypeFactDraft {
        TypeFactDraft {
            subject: owner,
            predicate: TypePredicateRef::new(predicate),
            polarity: Polarity::Positive,
            provenance,
            status,
        }
    }

    fn insert_type_entry(
        types: &mut TypeTable,
        owner: TypedSiteRef,
        actual: TypeEntryActual,
        status: TypeStatus,
    ) -> TypeEntryId {
        types.insert(TypeEntryDraft {
            owner,
            expected: None,
            actual,
            status,
            provenance: TypeProvenance::Inferred(TypeRuleId::new("status-test")),
        })
    }

    fn source_id() -> SourceId {
        let snapshot = BuildSnapshotId::from_published_schema_str(&format!(
            "mizar-session-build-snapshot-v1:{}",
            "11".repeat(32)
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
            LocalSymbolId::new("x/0"),
            FullyQualifiedName::new("pkg::main::x/0"),
        )
    }
}
