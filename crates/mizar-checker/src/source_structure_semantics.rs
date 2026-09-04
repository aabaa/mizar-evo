//! Bounded source-derived structure semantic checking for Step 5C.2.
//!
//! This module deliberately consumes syntax-free inputs. Structure and member
//! identities are resolver-authenticated; exact-slice variable bindings use a
//! separate source-local identity scheme authenticated structurally here. It
//! is not a parser and it does not perform name lookup: a missing identity in a
//! selector or inheritance mapping is retained only so that the checker can
//! report the frozen source diagnostic.

use mizar_resolve::{
    env::{DefinitionKind, SymbolEnv, SymbolKind},
    resolved_ast::{ModuleId, SymbolId},
};
use mizar_session::{SourceAnchor, SourceId, SourceRange};
use std::{collections::BTreeSet, error::Error, fmt};

/// An exact structure type admitted by this semantic slice.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum SourceStructureType {
    /// The builtin `set` type.
    Set,
    /// A declared structure application and its bracket arguments.
    Structure {
        /// The resolver identity of the structure declaration.
        symbol: SymbolId,
        /// Exact bracket arguments, in source order.
        arguments: Vec<SourceStructureType>,
    },
}

impl SourceStructureType {
    /// Returns the structure identity, when this is a structure application.
    #[must_use]
    pub const fn symbol(&self) -> Option<&SymbolId> {
        match self {
            Self::Set => None,
            Self::Structure { symbol, .. } => Some(symbol),
        }
    }

    /// Returns bracket arguments, or an empty slice for `set`.
    #[must_use]
    pub fn arguments(&self) -> &[SourceStructureType] {
        match self {
            Self::Set => &[],
            Self::Structure { arguments, .. } => arguments,
        }
    }
}

/// The role of one declared structure member.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum SourceStructureMemberKind {
    /// Stored constructor data.
    Field,
    /// A selector supplied by a property implementation.
    Property,
}

/// One source structure member supplied to a program input.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceStructureMemberInput {
    /// Resolver identity of the selector declaration.
    symbol: SymbolId,
    /// Source-written member spelling.
    spelling: String,
    /// Resolver's canonical spelling for this identity.  This is separate
    /// from the source spelling because a structure field projection can use
    /// the field's result-type token as its resolver primary spelling.
    resolver_spelling: String,
    /// Field or property role.
    kind: SourceStructureMemberKind,
    /// Exact declared member type.
    ty: SourceStructureType,
    /// Source range of the member declaration.
    source_range: SourceRange,
    /// Source-order member ordinal.
    source_ordinal: usize,
    /// Whether the producer observed recovered syntax.
    recovered: bool,
}

impl SourceStructureMemberInput {
    /// Creates a source member record with distinct source and resolver
    /// spellings.
    #[must_use]
    #[allow(clippy::too_many_arguments)] // Rationale: each frozen source/member identity component remains explicit at construction.
    pub fn new(
        symbol: SymbolId,
        spelling: impl Into<String>,
        resolver_spelling: impl Into<String>,
        kind: SourceStructureMemberKind,
        ty: SourceStructureType,
        source_range: SourceRange,
        source_ordinal: usize,
        recovered: bool,
    ) -> Self {
        Self {
            symbol,
            spelling: spelling.into(),
            resolver_spelling: resolver_spelling.into(),
            kind,
            ty,
            source_range,
            source_ordinal,
            recovered,
        }
    }

    /// Returns the resolver identity.
    #[must_use]
    pub const fn symbol(&self) -> &SymbolId {
        &self.symbol
    }

    /// Returns the source-written spelling.
    #[must_use]
    pub fn spelling(&self) -> &str {
        &self.spelling
    }

    /// Returns the resolver canonical spelling.
    #[must_use]
    pub fn resolver_spelling(&self) -> &str {
        &self.resolver_spelling
    }

    /// Returns the field/property role.
    #[must_use]
    pub const fn kind(&self) -> SourceStructureMemberKind {
        self.kind
    }

    /// Returns the exact member type.
    #[must_use]
    pub const fn ty(&self) -> &SourceStructureType {
        &self.ty
    }

    /// Returns the declaration range.
    #[must_use]
    pub const fn source_range(&self) -> SourceRange {
        self.source_range
    }

    /// Returns the source-order ordinal.
    #[must_use]
    pub const fn source_ordinal(&self) -> usize {
        self.source_ordinal
    }

    /// Returns whether syntax recovery was observed.
    #[must_use]
    pub const fn recovered(&self) -> bool {
        self.recovered
    }
}

/// One source structure declaration supplied to a program input.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceStructureDefinitionInput {
    /// Resolver identity of the structure declaration.
    symbol: SymbolId,
    /// Source-written structure spelling.
    spelling: String,
    /// Bracket parameter spellings in declaration order.
    parameters: Vec<String>,
    /// Ordered member declarations.
    members: Vec<SourceStructureMemberInput>,
    /// Source range of the declaration.
    source_range: SourceRange,
    /// Source-order declaration ordinal.
    source_ordinal: usize,
    /// Whether the producer observed recovered syntax.
    recovered: bool,
}

impl SourceStructureDefinitionInput {
    /// Creates a source structure declaration record.
    #[must_use]
    pub fn new(
        symbol: SymbolId,
        spelling: impl Into<String>,
        parameters: Vec<String>,
        members: Vec<SourceStructureMemberInput>,
        source_range: SourceRange,
        source_ordinal: usize,
        recovered: bool,
    ) -> Self {
        Self {
            symbol,
            spelling: spelling.into(),
            parameters,
            members,
            source_range,
            source_ordinal,
            recovered,
        }
    }

    /// Returns the resolver identity.
    #[must_use]
    pub const fn symbol(&self) -> &SymbolId {
        &self.symbol
    }

    /// Returns the source-written structure spelling.
    #[must_use]
    pub fn spelling(&self) -> &str {
        &self.spelling
    }

    /// Returns bracket parameter spellings.
    #[must_use]
    pub fn parameters(&self) -> &[String] {
        &self.parameters
    }

    /// Returns members in source order.
    #[must_use]
    pub fn members(&self) -> &[SourceStructureMemberInput] {
        &self.members
    }

    /// Returns the declaration range.
    #[must_use]
    pub const fn source_range(&self) -> SourceRange {
        self.source_range
    }

    /// Returns the source-order ordinal.
    #[must_use]
    pub const fn source_ordinal(&self) -> usize {
        self.source_ordinal
    }

    /// Returns whether syntax recovery was observed.
    #[must_use]
    pub const fn recovered(&self) -> bool {
        self.recovered
    }
}

/// Parent target of an inheritance declaration.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum SourceStructureInheritanceParent {
    /// A declared structure application.
    Structure {
        /// Resolver identity of the parent structure.
        symbol: SymbolId,
        /// Exact bracket arguments.
        arguments: Vec<SourceStructureType>,
    },
    /// The builtin `set` root.
    Set,
}

/// One explicit source inheritance mapping.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceStructureInheritanceMappingInput {
    /// Resolver identity of the child view, when resolver resolution succeeded.
    child_member: Option<SymbolId>,
    /// Source-written child member spelling.
    child_spelling: String,
    /// Resolver canonical spelling for the child identity, when resolved.
    child_resolver_spelling: Option<String>,
    /// Resolver identity of the parent view, when resolution succeeded.
    parent_member: Option<SymbolId>,
    /// Source-written parent member spelling, or `it` for set inheritance.
    parent_spelling: String,
    /// Resolver canonical spelling for the parent identity, when resolved.
    parent_resolver_spelling: Option<String>,
    /// Whether the source used the explicit `it` target.
    from_it: bool,
    /// Field/property role written by the producer.
    kind: SourceStructureMemberKind,
    /// Source range of the mapping.
    source_range: SourceRange,
    /// Source-order mapping ordinal.
    source_ordinal: usize,
    /// Whether the producer observed recovered syntax.
    recovered: bool,
}

impl SourceStructureInheritanceMappingInput {
    /// Creates one inheritance mapping record.
    #[must_use]
    #[allow(clippy::too_many_arguments)] // Rationale: each frozen mapping identity, role, range, and order component remains explicit.
    pub fn new(
        child_member: Option<SymbolId>,
        child_spelling: impl Into<String>,
        child_resolver_spelling: Option<String>,
        parent_member: Option<SymbolId>,
        parent_spelling: impl Into<String>,
        parent_resolver_spelling: Option<String>,
        from_it: bool,
        kind: SourceStructureMemberKind,
        source_range: SourceRange,
        source_ordinal: usize,
        recovered: bool,
    ) -> Self {
        Self {
            child_member,
            child_spelling: child_spelling.into(),
            child_resolver_spelling,
            parent_member,
            parent_spelling: parent_spelling.into(),
            parent_resolver_spelling,
            from_it,
            kind,
            source_range,
            source_ordinal,
            recovered,
        }
    }

    /// Returns the resolved child identity.
    #[must_use]
    pub const fn child_member(&self) -> Option<&SymbolId> {
        self.child_member.as_ref()
    }

    /// Returns the source child spelling.
    #[must_use]
    pub fn child_spelling(&self) -> &str {
        &self.child_spelling
    }

    /// Returns the resolver child spelling, when resolved.
    #[must_use]
    pub fn child_resolver_spelling(&self) -> Option<&str> {
        self.child_resolver_spelling.as_deref()
    }

    /// Returns the resolved parent identity.
    #[must_use]
    pub const fn parent_member(&self) -> Option<&SymbolId> {
        self.parent_member.as_ref()
    }

    /// Returns the source parent spelling.
    #[must_use]
    pub fn parent_spelling(&self) -> &str {
        &self.parent_spelling
    }

    /// Returns the resolver parent spelling, when resolved.
    #[must_use]
    pub fn parent_resolver_spelling(&self) -> Option<&str> {
        self.parent_resolver_spelling.as_deref()
    }

    /// Returns whether the mapping targets `it`.
    #[must_use]
    pub const fn from_it(&self) -> bool {
        self.from_it
    }

    /// Returns the field/property role.
    #[must_use]
    pub const fn kind(&self) -> SourceStructureMemberKind {
        self.kind
    }

    /// Returns the mapping range.
    #[must_use]
    pub const fn source_range(&self) -> SourceRange {
        self.source_range
    }

    /// Returns the source-order ordinal.
    #[must_use]
    pub const fn source_ordinal(&self) -> usize {
        self.source_ordinal
    }

    /// Returns whether syntax recovery was observed.
    #[must_use]
    pub const fn recovered(&self) -> bool {
        self.recovered
    }
}

/// One source inheritance declaration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceStructureInheritanceInput {
    /// Resolver identity of the child structure.
    child: SymbolId,
    /// Parent structure or builtin set root.
    parent: SourceStructureInheritanceParent,
    /// `true` for an explicit `where` mapping block.
    explicit: bool,
    /// Ordered mappings in the block.  Shorthand declarations use an empty list.
    mappings: Vec<SourceStructureInheritanceMappingInput>,
    /// Whether this edge carries the single coherence block.
    coherence: bool,
    /// Source range of the inheritance declaration.
    source_range: SourceRange,
    /// Source-order inheritance ordinal.
    source_ordinal: usize,
    /// Whether the producer observed recovered syntax.
    recovered: bool,
}

impl SourceStructureInheritanceInput {
    /// Creates one source inheritance declaration record.
    #[must_use]
    #[allow(clippy::too_many_arguments)] // Rationale: each frozen inheritance edge component remains explicit at construction.
    pub fn new(
        child: SymbolId,
        parent: SourceStructureInheritanceParent,
        explicit: bool,
        mappings: Vec<SourceStructureInheritanceMappingInput>,
        coherence: bool,
        source_range: SourceRange,
        source_ordinal: usize,
        recovered: bool,
    ) -> Self {
        Self {
            child,
            parent,
            explicit,
            mappings,
            coherence,
            source_range,
            source_ordinal,
            recovered,
        }
    }

    /// Returns the child structure identity.
    #[must_use]
    pub const fn child(&self) -> &SymbolId {
        &self.child
    }

    /// Returns the parent target.
    #[must_use]
    pub const fn parent(&self) -> &SourceStructureInheritanceParent {
        &self.parent
    }

    /// Returns whether mappings were explicit.
    #[must_use]
    pub const fn explicit(&self) -> bool {
        self.explicit
    }

    /// Returns mappings in source order.
    #[must_use]
    pub fn mappings(&self) -> &[SourceStructureInheritanceMappingInput] {
        &self.mappings
    }

    /// Returns whether this edge carries a coherence block.
    #[must_use]
    pub const fn coherence(&self) -> bool {
        self.coherence
    }

    /// Returns the inheritance range.
    #[must_use]
    pub const fn source_range(&self) -> SourceRange {
        self.source_range
    }

    /// Returns the source-order ordinal.
    #[must_use]
    pub const fn source_ordinal(&self) -> usize {
        self.source_ordinal
    }

    /// Returns whether syntax recovery was observed.
    #[must_use]
    pub const fn recovered(&self) -> bool {
        self.recovered
    }
}

/// One source variable type binding used by structure terms.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceStructureVariableInput {
    /// Structurally authenticated source-local binding identity.
    symbol: SymbolId,
    /// Source-written binding spelling.
    spelling: String,
    /// Exact declared type.
    ty: SourceStructureType,
    /// Source range of the binding.
    source_range: SourceRange,
    /// Source-order declaration ordinal.
    source_ordinal: usize,
    /// Whether the producer observed recovered syntax.
    recovered: bool,
}

impl SourceStructureVariableInput {
    /// Creates a source variable type binding record.
    #[must_use]
    pub fn new(
        symbol: SymbolId,
        spelling: impl Into<String>,
        ty: SourceStructureType,
        source_range: SourceRange,
        source_ordinal: usize,
        recovered: bool,
    ) -> Self {
        Self {
            symbol,
            spelling: spelling.into(),
            ty,
            source_range,
            source_ordinal,
            recovered,
        }
    }

    /// Returns the variable identity.
    #[must_use]
    pub const fn symbol(&self) -> &SymbolId {
        &self.symbol
    }

    /// Returns the source binding spelling.
    #[must_use]
    pub fn spelling(&self) -> &str {
        &self.spelling
    }

    /// Returns the exact variable type.
    #[must_use]
    pub const fn ty(&self) -> &SourceStructureType {
        &self.ty
    }

    /// Returns the binding range.
    #[must_use]
    pub const fn source_range(&self) -> SourceRange {
        self.source_range
    }

    /// Returns the source-order ordinal.
    #[must_use]
    pub const fn source_ordinal(&self) -> usize {
        self.source_ordinal
    }

    /// Returns whether syntax recovery was observed.
    #[must_use]
    pub const fn recovered(&self) -> bool {
        self.recovered
    }
}

/// A named field argument in a structure constructor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceStructureFieldArgument {
    /// Resolver identity of the field, when resolution succeeded.
    member: Option<SymbolId>,
    /// Source-written argument name.
    spelling: String,
    /// Resolver canonical spelling for the member identity, when resolved.
    resolver_spelling: Option<String>,
    /// Replacement/constructor value.
    value: Box<SourceStructureTerm>,
    /// Source range of the argument.
    source_range: SourceRange,
    /// Source-order argument ordinal.
    source_ordinal: usize,
    /// Whether the producer observed recovered syntax.
    recovered: bool,
}

impl SourceStructureFieldArgument {
    /// Creates one named constructor field argument.
    #[must_use]
    pub fn new(
        member: Option<SymbolId>,
        spelling: impl Into<String>,
        resolver_spelling: Option<String>,
        value: Box<SourceStructureTerm>,
        source_range: SourceRange,
        source_ordinal: usize,
        recovered: bool,
    ) -> Self {
        Self {
            member,
            spelling: spelling.into(),
            resolver_spelling,
            value,
            source_range,
            source_ordinal,
            recovered,
        }
    }

    /// Returns the resolved member identity.
    #[must_use]
    pub const fn member(&self) -> Option<&SymbolId> {
        self.member.as_ref()
    }

    /// Returns the source argument spelling.
    #[must_use]
    pub fn spelling(&self) -> &str {
        &self.spelling
    }

    /// Returns the resolver canonical spelling, when resolved.
    #[must_use]
    pub fn resolver_spelling(&self) -> Option<&str> {
        self.resolver_spelling.as_deref()
    }

    /// Returns the constructor value.
    #[must_use]
    pub const fn value(&self) -> &SourceStructureTerm {
        &self.value
    }

    /// Returns the argument range.
    #[must_use]
    pub const fn source_range(&self) -> SourceRange {
        self.source_range
    }

    /// Returns the source-order ordinal.
    #[must_use]
    pub const fn source_ordinal(&self) -> usize {
        self.source_ordinal
    }

    /// Returns whether syntax recovery was observed.
    #[must_use]
    pub const fn recovered(&self) -> bool {
        self.recovered
    }
}

/// A source structure term occurrence.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum SourceStructureTerm {
    /// A variable reference.
    Variable {
        /// Structurally authenticated source-local binding identity.
        symbol: SymbolId,
        /// Source-written variable spelling.
        spelling: String,
        /// Source range of the occurrence.
        source_range: SourceRange,
        /// Source-order occurrence ordinal.
        source_ordinal: usize,
        /// Whether the producer observed recovered syntax.
        recovered: bool,
    },
    /// A named structure constructor.
    Constructor {
        /// Resolver identity of the structure declaration.
        structure: SymbolId,
        /// Exact bracket arguments.
        type_arguments: Vec<SourceStructureType>,
        /// Named constructor arguments in source order.
        arguments: Vec<SourceStructureFieldArgument>,
        /// Source range of the constructor occurrence.
        source_range: SourceRange,
        /// Source-order occurrence ordinal.
        source_ordinal: usize,
        /// Whether the producer observed recovered syntax.
        recovered: bool,
    },
    /// A field or property selector.
    Select {
        /// Selected subject term.
        subject: Box<SourceStructureTerm>,
        /// Resolver identity of the selected member, when resolved.
        member: Option<SymbolId>,
        /// Source-written selector spelling.
        spelling: String,
        /// Resolver canonical spelling for the member identity, when
        /// resolved.  This is distinct from source spelling for structure
        /// fields whose projection primary spelling is a type token.
        resolver_spelling: Option<String>,
        /// Source range of the selector occurrence.
        source_range: SourceRange,
        /// Source-order occurrence ordinal.
        source_ordinal: usize,
        /// Whether the producer observed recovered syntax.
        recovered: bool,
    },
    /// An immutable field update.
    Update {
        /// Updated subject term.
        subject: Box<SourceStructureTerm>,
        /// Resolver identity of the updated field, when resolved.
        member: Option<SymbolId>,
        /// Source-written field spelling.
        spelling: String,
        /// Resolver canonical spelling for the member identity, when
        /// resolved.
        resolver_spelling: Option<String>,
        /// Replacement value.
        value: Box<SourceStructureTerm>,
        /// Source range of the update occurrence.
        source_range: SourceRange,
        /// Source-order occurrence ordinal.
        source_ordinal: usize,
        /// Whether the producer observed recovered syntax.
        recovered: bool,
    },
}

impl SourceStructureTerm {
    /// Returns the source range of this occurrence.
    #[must_use]
    pub const fn source_range(&self) -> SourceRange {
        match self {
            Self::Variable { source_range, .. }
            | Self::Constructor { source_range, .. }
            | Self::Select { source_range, .. }
            | Self::Update { source_range, .. } => *source_range,
        }
    }

    /// Returns the source-order occurrence ordinal.
    #[must_use]
    pub const fn source_ordinal(&self) -> usize {
        match self {
            Self::Variable { source_ordinal, .. }
            | Self::Constructor { source_ordinal, .. }
            | Self::Select { source_ordinal, .. }
            | Self::Update { source_ordinal, .. } => *source_ordinal,
        }
    }

    /// Returns whether this occurrence came from recovered syntax.
    #[must_use]
    pub const fn recovered(&self) -> bool {
        match self {
            Self::Variable { recovered, .. }
            | Self::Constructor { recovered, .. }
            | Self::Select { recovered, .. }
            | Self::Update { recovered, .. } => *recovered,
        }
    }
}

/// One equality proposition or ordered `thus` conclusion.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceStructureEqualityInput {
    /// Left side of the equality.
    left: SourceStructureTerm,
    /// Right side of the equality.
    right: SourceStructureTerm,
    /// Source range of the equality.
    source_range: SourceRange,
    /// Source-order equality ordinal.
    source_ordinal: usize,
    /// Whether the producer observed recovered syntax.
    recovered: bool,
}

impl SourceStructureEqualityInput {
    /// Creates one exact equality claim.
    #[must_use]
    pub fn new(
        left: SourceStructureTerm,
        right: SourceStructureTerm,
        source_range: SourceRange,
        source_ordinal: usize,
        recovered: bool,
    ) -> Self {
        Self {
            left,
            right,
            source_range,
            source_ordinal,
            recovered,
        }
    }

    /// Returns the left term.
    #[must_use]
    pub const fn left(&self) -> &SourceStructureTerm {
        &self.left
    }

    /// Returns the right term.
    #[must_use]
    pub const fn right(&self) -> &SourceStructureTerm {
        &self.right
    }

    /// Returns the equality range.
    #[must_use]
    pub const fn source_range(&self) -> SourceRange {
        self.source_range
    }

    /// Returns the source-order ordinal.
    #[must_use]
    pub const fn source_ordinal(&self) -> usize {
        self.source_ordinal
    }

    /// Returns whether syntax recovery was observed.
    #[must_use]
    pub const fn recovered(&self) -> bool {
        self.recovered
    }
}

/// One theorem proposition and its ordered `thus` conclusions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceStructureClaimInput {
    /// The theorem proposition.
    proposition: SourceStructureEqualityInput,
    /// Ordered proof conclusions.
    conclusions: Vec<SourceStructureEqualityInput>,
    /// Source range of the theorem claim.
    source_range: SourceRange,
    /// Source-order theorem ordinal.
    source_ordinal: usize,
    /// Whether the producer observed recovered syntax.
    recovered: bool,
}

impl SourceStructureClaimInput {
    /// Creates one theorem proposition with ordered `thus` conclusions.
    #[must_use]
    pub fn new(
        proposition: SourceStructureEqualityInput,
        conclusions: Vec<SourceStructureEqualityInput>,
        source_range: SourceRange,
        source_ordinal: usize,
        recovered: bool,
    ) -> Self {
        Self {
            proposition,
            conclusions,
            source_range,
            source_ordinal,
            recovered,
        }
    }

    /// Returns the theorem proposition.
    #[must_use]
    pub const fn proposition(&self) -> &SourceStructureEqualityInput {
        &self.proposition
    }

    /// Returns ordered conclusions.
    #[must_use]
    pub fn conclusions(&self) -> &[SourceStructureEqualityInput] {
        &self.conclusions
    }

    /// Returns the theorem range.
    #[must_use]
    pub const fn source_range(&self) -> SourceRange {
        self.source_range
    }

    /// Returns the source-order ordinal.
    #[must_use]
    pub const fn source_ordinal(&self) -> usize {
        self.source_ordinal
    }

    /// Returns whether syntax recovery was observed.
    #[must_use]
    pub const fn recovered(&self) -> bool {
        self.recovered
    }
}

/// Complete syntax-free structure program input.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceStructureProgramInput {
    source_id: SourceId,
    module_id: ModuleId,
    definitions: Vec<SourceStructureDefinitionInput>,
    inheritances: Vec<SourceStructureInheritanceInput>,
    variables: Vec<SourceStructureVariableInput>,
    terms: Vec<SourceStructureTerm>,
    claims: Vec<SourceStructureClaimInput>,
}

impl SourceStructureProgramInput {
    /// Creates a syntax-free source structure semantic transaction.
    #[must_use]
    pub const fn new(
        source_id: SourceId,
        module_id: ModuleId,
        definitions: Vec<SourceStructureDefinitionInput>,
        inheritances: Vec<SourceStructureInheritanceInput>,
        variables: Vec<SourceStructureVariableInput>,
        terms: Vec<SourceStructureTerm>,
        claims: Vec<SourceStructureClaimInput>,
    ) -> Self {
        Self {
            source_id,
            module_id,
            definitions,
            inheritances,
            variables,
            terms,
            claims,
        }
    }

    /// Returns the source identity.
    #[must_use]
    pub const fn source_id(&self) -> SourceId {
        self.source_id
    }

    /// Returns the module identity.
    #[must_use]
    pub const fn module_id(&self) -> &ModuleId {
        &self.module_id
    }

    /// Returns structure declarations in source order.
    #[must_use]
    pub fn definitions(&self) -> &[SourceStructureDefinitionInput] {
        &self.definitions
    }

    /// Returns inheritance declarations in source order.
    #[must_use]
    pub fn inheritances(&self) -> &[SourceStructureInheritanceInput] {
        &self.inheritances
    }

    /// Returns variable bindings in source order.
    #[must_use]
    pub fn variables(&self) -> &[SourceStructureVariableInput] {
        &self.variables
    }

    /// Returns top-level terms in source order.
    #[must_use]
    pub fn terms(&self) -> &[SourceStructureTerm] {
        &self.terms
    }

    /// Returns theorem claims in source order.
    #[must_use]
    pub fn claims(&self) -> &[SourceStructureClaimInput] {
        &self.claims
    }
}

/// A checker-owned immutable structure declaration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceStructureDefinition {
    symbol: SymbolId,
    spelling: String,
    parameters: Vec<String>,
    members: Vec<SourceStructureMember>,
    source_range: SourceRange,
    source_ordinal: usize,
}

impl SourceStructureDefinition {
    /// Returns the resolver structure identity.
    #[must_use]
    pub const fn symbol(&self) -> &SymbolId {
        &self.symbol
    }

    /// Returns the source spelling.
    #[must_use]
    pub fn spelling(&self) -> &str {
        &self.spelling
    }

    /// Returns bracket parameter spellings.
    #[must_use]
    pub fn parameters(&self) -> &[String] {
        &self.parameters
    }

    /// Returns declared members in source order.
    #[must_use]
    pub fn members(&self) -> &[SourceStructureMember] {
        &self.members
    }

    /// Returns the declaration source range.
    #[must_use]
    pub const fn source_range(&self) -> SourceRange {
        self.source_range
    }

    /// Returns the declaration source ordinal.
    #[must_use]
    pub const fn source_ordinal(&self) -> usize {
        self.source_ordinal
    }

    /// Finds a declared member by resolver identity.
    #[must_use]
    pub fn member(&self, symbol: &SymbolId) -> Option<&SourceStructureMember> {
        self.members.iter().find(|member| member.symbol() == symbol)
    }
}

/// A checker-owned immutable structure member.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceStructureMember {
    symbol: SymbolId,
    spelling: String,
    resolver_spelling: String,
    kind: SourceStructureMemberKind,
    ty: SourceStructureType,
    source_range: SourceRange,
    source_ordinal: usize,
}

impl SourceStructureMember {
    /// Returns the resolver member identity.
    #[must_use]
    pub const fn symbol(&self) -> &SymbolId {
        &self.symbol
    }

    /// Returns the source spelling.
    #[must_use]
    pub fn spelling(&self) -> &str {
        &self.spelling
    }

    /// Returns the resolver canonical spelling.
    #[must_use]
    pub fn resolver_spelling(&self) -> &str {
        &self.resolver_spelling
    }

    /// Returns the field/property role.
    #[must_use]
    pub const fn kind(&self) -> SourceStructureMemberKind {
        self.kind
    }

    /// Returns the exact declared type.
    #[must_use]
    pub const fn ty(&self) -> &SourceStructureType {
        &self.ty
    }

    /// Returns the member declaration range.
    #[must_use]
    pub const fn source_range(&self) -> SourceRange {
        self.source_range
    }

    /// Returns the member declaration ordinal.
    #[must_use]
    pub const fn source_ordinal(&self) -> usize {
        self.source_ordinal
    }
}

/// A checker-owned immutable equality claim.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceStructureClaim {
    proposition: SourceStructureEqualityInput,
    conclusions: Vec<SourceStructureEqualityInput>,
    source_range: SourceRange,
    source_ordinal: usize,
}

impl SourceStructureClaim {
    /// Returns the theorem proposition.
    #[must_use]
    pub const fn proposition(&self) -> &SourceStructureEqualityInput {
        &self.proposition
    }

    /// Returns ordered `thus` conclusions.
    #[must_use]
    pub fn conclusions(&self) -> &[SourceStructureEqualityInput] {
        &self.conclusions
    }

    /// Returns the theorem source range.
    #[must_use]
    pub const fn source_range(&self) -> SourceRange {
        self.source_range
    }

    /// Returns the theorem source ordinal.
    #[must_use]
    pub const fn source_ordinal(&self) -> usize {
        self.source_ordinal
    }
}

/// The semantic phase that owns a structure diagnostic.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum SourceStructureDiagnosticPhase {
    /// Resolver/definition identity phase.
    Resolve,
    /// Structure and term type-checking phase.
    TypeCheck,
}

/// One stable structure semantic diagnostic.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SourceStructureDiagnostic {
    phase: SourceStructureDiagnosticPhase,
    source_range: SourceRange,
    detail_key: &'static str,
}

impl SourceStructureDiagnostic {
    /// Returns the diagnostic phase.
    #[must_use]
    pub const fn phase(&self) -> SourceStructureDiagnosticPhase {
        self.phase
    }

    /// Returns the source range associated with the diagnostic.
    #[must_use]
    pub const fn source_range(&self) -> SourceRange {
        self.source_range
    }

    /// Returns the frozen semantic detail key.
    #[must_use]
    pub const fn detail_key(&self) -> &'static str {
        self.detail_key
    }
}

/// Immutable output of source structure semantic checking.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceStructureSemanticsOutput {
    source_id: SourceId,
    module_id: ModuleId,
    structures: Vec<SourceStructureDefinition>,
    terms: Vec<SourceStructureTerm>,
    claims: Vec<SourceStructureClaim>,
    diagnostics: Vec<SourceStructureDiagnostic>,
}

impl SourceStructureSemanticsOutput {
    /// Returns the authenticated source identity.
    #[must_use]
    pub const fn source_id(&self) -> SourceId {
        self.source_id
    }

    /// Returns the authenticated module identity.
    #[must_use]
    pub const fn module_id(&self) -> &ModuleId {
        &self.module_id
    }

    /// Returns structures accepted before the first diagnostic.
    #[must_use]
    pub fn structures(&self) -> &[SourceStructureDefinition] {
        &self.structures
    }

    /// Returns terms accepted before the first diagnostic.
    #[must_use]
    pub fn terms(&self) -> &[SourceStructureTerm] {
        &self.terms
    }

    /// Returns claims accepted before the first diagnostic.
    #[must_use]
    pub fn claims(&self) -> &[SourceStructureClaim] {
        &self.claims
    }

    /// Returns diagnostics in source order.
    #[must_use]
    pub fn diagnostics(&self) -> &[SourceStructureDiagnostic] {
        &self.diagnostics
    }
}

/// Malformed or unauthenticated payload rejected before semantic publication.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum SourceStructurePayloadError {
    /// A range belongs to a different source.
    SourceMismatch,
    /// A resolver identity belongs to a different module.
    ModuleMismatch,
    /// A source range or source ordering relation is malformed.
    InvalidRange,
    /// A source-ordered vector was reordered, duplicated, or otherwise malformed.
    InvalidOrder,
    /// Recovered syntax is not admitted by this semantic slice.
    RecoveredSyntax,
    /// A resolver identity, spelling, kind, or origin does not authenticate.
    InvalidIdentity,
    /// The resolver supplied duplicate identity entries.
    DuplicateIdentity,
    /// The payload shape is outside the bounded slice.
    UnsupportedShape,
    /// An exact type relation required by this slice does not hold.
    TypeMismatch,
    /// An inheritance edge would introduce a cycle.
    Cycle,
}

impl fmt::Display for SourceStructurePayloadError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::SourceMismatch => "source structure payload source mismatch",
            Self::ModuleMismatch => "source structure payload module mismatch",
            Self::InvalidRange => "invalid source structure range",
            Self::InvalidOrder => "invalid source structure source order",
            Self::RecoveredSyntax => "recovered structure syntax is not admitted",
            Self::InvalidIdentity => "invalid resolver identity in structure payload",
            Self::DuplicateIdentity => "duplicate resolver identity in structure payload",
            Self::UnsupportedShape => "unsupported structure payload shape",
            Self::TypeMismatch => "exact structure type mismatch",
            Self::Cycle => "cyclic structure inheritance",
        })
    }
}

impl Error for SourceStructurePayloadError {}

#[derive(Debug)]
enum SourceStructureSemanticEvent {
    Definition(SourceStructureDefinitionInput),
    Inheritance(SourceStructureInheritanceInput),
    Term(SourceStructureTerm),
    Claim(Box<SourceStructureClaimInput>),
}

impl SourceStructureSemanticEvent {
    const fn source_start(&self) -> usize {
        match self {
            Self::Definition(definition) => definition.source_range.start,
            Self::Inheritance(inheritance) => inheritance.source_range.start,
            Self::Term(term) => term.source_range().start,
            Self::Claim(claim) => claim.source_range.start,
        }
    }
}

/// Checker for the bounded Step 5C.2 source structure semantic slice.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct SourceStructureSemanticsChecker;

impl SourceStructureSemanticsChecker {
    /// Checks one syntax-free source structure program against resolver identities.
    pub fn check(
        input: SourceStructureProgramInput,
        symbols: &SymbolEnv,
    ) -> Result<SourceStructureSemanticsOutput, SourceStructurePayloadError> {
        validate_input_shape(&input, symbols)?;

        let SourceStructureProgramInput {
            source_id,
            module_id,
            definitions,
            inheritances,
            variables,
            terms,
            claims,
        } = input;

        let mut state = SemanticState {
            source_id,
            module_id,
            structures: Vec::new(),
            terms: Vec::new(),
            claims: Vec::new(),
            diagnostics: Vec::new(),
            variables: variables
                .into_iter()
                .map(|variable| (variable.symbol, variable.ty))
                .collect(),
            inherited: Vec::new(),
        };

        let mut events = definitions
            .into_iter()
            .map(SourceStructureSemanticEvent::Definition)
            .chain(
                inheritances
                    .into_iter()
                    .map(SourceStructureSemanticEvent::Inheritance),
            )
            .chain(terms.into_iter().map(SourceStructureSemanticEvent::Term))
            .chain(
                claims
                    .into_iter()
                    .map(Box::new)
                    .map(SourceStructureSemanticEvent::Claim),
            )
            .collect::<Vec<_>>();
        events.sort_by_key(SourceStructureSemanticEvent::source_start);
        if events
            .windows(2)
            .any(|window| window[0].source_start() == window[1].source_start())
        {
            return Err(SourceStructurePayloadError::InvalidOrder);
        }

        for event in events {
            if state.has_diagnostic() {
                break;
            }
            match event {
                SourceStructureSemanticEvent::Definition(definition) => {
                    let members = definition
                        .members
                        .iter()
                        .map(|member| SourceStructureMember {
                            symbol: member.symbol.clone(),
                            spelling: member.spelling.clone(),
                            resolver_spelling: member.resolver_spelling.clone(),
                            kind: member.kind,
                            ty: member.ty.clone(),
                            source_range: member.source_range,
                            source_ordinal: member.source_ordinal,
                        })
                        .collect::<Vec<_>>();
                    let mut spellings = Vec::new();
                    for member in &definition.members {
                        if spellings
                            .iter()
                            .any(|spelling| spelling == &member.spelling)
                        {
                            state.diagnostics.push(diagnostic(
                                SourceStructureDiagnosticPhase::Resolve,
                                member.source_range,
                                "structures.definition.duplicate_member",
                            ));
                            break;
                        }
                        spellings.push(member.spelling.clone());
                    }
                    if state.has_diagnostic() {
                        continue;
                    }
                    state.structures.push(SourceStructureDefinition {
                        symbol: definition.symbol,
                        spelling: definition.spelling,
                        parameters: definition.parameters,
                        members,
                        source_range: definition.source_range,
                        source_ordinal: definition.source_ordinal,
                    });
                    state.inherited.push(Vec::new());
                }
                SourceStructureSemanticEvent::Inheritance(inheritance) => {
                    check_inheritance(&mut state, &inheritance)?;
                }
                SourceStructureSemanticEvent::Term(term) => {
                    check_term(&mut state, &term)?;
                    if !state.has_diagnostic() {
                        state.terms.push(term);
                    }
                }
                SourceStructureSemanticEvent::Claim(claim) => {
                    let claim = *claim;
                    check_equality(&mut state, &claim.proposition)?;
                    for conclusion in &claim.conclusions {
                        if state.has_diagnostic() {
                            break;
                        }
                        check_equality(&mut state, conclusion)?;
                    }
                    if !state.has_diagnostic() {
                        state.claims.push(SourceStructureClaim {
                            proposition: claim.proposition,
                            conclusions: claim.conclusions,
                            source_range: claim.source_range,
                            source_ordinal: claim.source_ordinal,
                        });
                    }
                }
            }
        }

        Ok(SourceStructureSemanticsOutput {
            source_id: state.source_id,
            module_id: state.module_id,
            structures: state.structures,
            terms: state.terms,
            claims: state.claims,
            diagnostics: state.diagnostics,
        })
    }
}

#[derive(Debug)]
struct SemanticState {
    source_id: SourceId,
    module_id: ModuleId,
    structures: Vec<SourceStructureDefinition>,
    terms: Vec<SourceStructureTerm>,
    claims: Vec<SourceStructureClaim>,
    diagnostics: Vec<SourceStructureDiagnostic>,
    variables: Vec<(SymbolId, SourceStructureType)>,
    inherited: Vec<Vec<InheritedCoverage>>,
}

impl SemanticState {
    fn has_diagnostic(&self) -> bool {
        !self.diagnostics.is_empty()
    }

    fn structure_index(&self, symbol: &SymbolId) -> Option<usize> {
        self.structures
            .iter()
            .position(|structure| structure.symbol() == symbol)
    }

    fn member_for(&self, structure: usize, symbol: &SymbolId) -> Option<&SourceStructureMember> {
        self.structures[structure].member(symbol)
    }

    fn effective_members(&self, structure: usize) -> Vec<&SourceStructureMember> {
        let mut members = self.structures[structure]
            .members()
            .iter()
            .collect::<Vec<_>>();
        for coverage in &self.inherited[structure] {
            if members
                .iter()
                .any(|member| member.symbol() == &coverage.child_member)
            {
                continue;
            }
            if let Some(member) = self.member_for(structure, &coverage.child_member) {
                members.push(member);
            }
        }
        members
    }
}

#[derive(Debug, Clone)]
struct InheritedCoverage {
    child_member: SymbolId,
    parent_structure: usize,
}

fn check_inheritance(
    state: &mut SemanticState,
    inheritance: &SourceStructureInheritanceInput,
) -> Result<(), SourceStructurePayloadError> {
    if inheritance.coherence {
        return Err(SourceStructurePayloadError::UnsupportedShape);
    }
    let Some(child_index) = state.structure_index(&inheritance.child) else {
        return Err(SourceStructurePayloadError::InvalidIdentity);
    };
    let parent_symbol = match &inheritance.parent {
        SourceStructureInheritanceParent::Structure { symbol, .. } => Some(symbol),
        SourceStructureInheritanceParent::Set => None,
    };

    if let Some(parent_symbol) = parent_symbol {
        let Some(parent_index) = state.structure_index(parent_symbol) else {
            return Err(SourceStructurePayloadError::InvalidIdentity);
        };
        match &inheritance.parent {
            SourceStructureInheritanceParent::Structure { arguments, .. }
                if arguments.len() != state.structures[parent_index].parameters().len() =>
            {
                return Err(SourceStructurePayloadError::TypeMismatch);
            }
            SourceStructureInheritanceParent::Structure { .. }
            | SourceStructureInheritanceParent::Set => {}
        }
        if child_index == parent_index || reaches(state, parent_index, child_index) {
            return Err(SourceStructurePayloadError::Cycle);
        }
        let parent_members = state
            .effective_members(parent_index)
            .into_iter()
            .cloned()
            .collect::<Vec<_>>();
        let mappings = if inheritance.explicit {
            inheritance.mappings.clone()
        } else {
            parent_members
                .iter()
                .map(|parent| SourceStructureInheritanceMappingInput {
                    child_member: state.structures[child_index]
                        .members()
                        .iter()
                        .find(|child| child.spelling() == parent.spelling())
                        .map(|member| member.symbol().clone()),
                    child_spelling: parent.spelling().to_owned(),
                    child_resolver_spelling: state.structures[child_index]
                        .members()
                        .iter()
                        .find(|child| child.spelling() == parent.spelling())
                        .map(|member| member.resolver_spelling().to_owned()),
                    parent_member: Some(parent.symbol().clone()),
                    parent_spelling: parent.spelling().to_owned(),
                    parent_resolver_spelling: Some(parent.resolver_spelling().to_owned()),
                    from_it: false,
                    kind: parent.kind(),
                    source_range: inheritance.source_range,
                    source_ordinal: inheritance.source_ordinal,
                    recovered: false,
                })
                .collect()
        };
        let mut covered = Vec::new();
        let mut new_coverages = Vec::new();
        for mapping in mappings {
            if mapping.from_it {
                return Err(SourceStructurePayloadError::InvalidIdentity);
            }
            let Some(parent_member_symbol) = mapping.parent_member.as_ref() else {
                state.diagnostics.push(diagnostic(
                    SourceStructureDiagnosticPhase::TypeCheck,
                    mapping.source_range,
                    "structures.inherit.unknown_source_member",
                ));
                return Ok(());
            };
            let Some(parent_member) = parent_members
                .iter()
                .find(|member| member.symbol() == parent_member_symbol)
            else {
                state.diagnostics.push(diagnostic(
                    SourceStructureDiagnosticPhase::TypeCheck,
                    mapping.source_range,
                    "structures.inherit.unknown_source_member",
                ));
                return Ok(());
            };
            let Some(child_member_symbol) = mapping.child_member.as_ref() else {
                if !inheritance.explicit {
                    // A shorthand edge with no same-spelled child member is
                    // an uncovered base member.  It is not an unresolved
                    // source spelling supplied by the producer.
                    continue;
                }
                state.diagnostics.push(diagnostic(
                    SourceStructureDiagnosticPhase::TypeCheck,
                    mapping.source_range,
                    "structures.inherit.unknown_source_member",
                ));
                return Ok(());
            };
            let Some(child_member) = state.member_for(child_index, child_member_symbol) else {
                state.diagnostics.push(diagnostic(
                    SourceStructureDiagnosticPhase::TypeCheck,
                    mapping.source_range,
                    "structures.inherit.unknown_source_member",
                ));
                return Ok(());
            };
            if child_member.spelling() != mapping.child_spelling
                || parent_member.spelling() != mapping.parent_spelling
                || child_member.kind() != mapping.kind
                || parent_member.kind() != mapping.kind
            {
                return Err(SourceStructurePayloadError::InvalidIdentity);
            }
            let child_member_identity = child_member.symbol().clone();
            if covered
                .iter()
                .any(|symbol: &SymbolId| symbol == parent_member.symbol())
            {
                return Err(SourceStructurePayloadError::InvalidOrder);
            }
            covered.push(parent_member.symbol().clone());
            if child_member.ty() != parent_member.ty() {
                state.diagnostics.push(diagnostic(
                    SourceStructureDiagnosticPhase::TypeCheck,
                    mapping.source_range,
                    "structures.inherit.diamond_inconsistency",
                ));
                return Ok(());
            }
            new_coverages.push(InheritedCoverage {
                child_member: child_member_identity,
                parent_structure: parent_index,
            });
        }
        if covered.len() != parent_members.len() {
            state.diagnostics.push(diagnostic(
                SourceStructureDiagnosticPhase::TypeCheck,
                inheritance.source_range,
                "structures.inherit.uncovered_base_member",
            ));
        } else {
            state.inherited[child_index].extend(new_coverages);
        }
    } else {
        if !inheritance.explicit || inheritance.mappings.is_empty() {
            return Err(SourceStructurePayloadError::UnsupportedShape);
        }
        let mut covered = Vec::new();
        for mapping in &inheritance.mappings {
            if !mapping.from_it
                || mapping.parent_member.is_some()
                || mapping.parent_spelling != "it"
                || mapping.kind != SourceStructureMemberKind::Field
            {
                return Err(SourceStructurePayloadError::InvalidIdentity);
            }
            let Some(child_member_symbol) = mapping.child_member.as_ref() else {
                return Err(SourceStructurePayloadError::InvalidIdentity);
            };
            let Some(child_member) = state.member_for(child_index, child_member_symbol) else {
                return Err(SourceStructurePayloadError::InvalidIdentity);
            };
            if child_member.spelling() != mapping.child_spelling
                || child_member.kind() != SourceStructureMemberKind::Field
                || child_member.ty() != &SourceStructureType::Set
            {
                return Err(SourceStructurePayloadError::TypeMismatch);
            }
            if covered
                .iter()
                .any(|symbol: &SymbolId| symbol == child_member_symbol)
            {
                return Err(SourceStructurePayloadError::InvalidOrder);
            }
            covered.push(child_member_symbol.clone());
        }
    }
    Ok(())
}

fn reaches(state: &SemanticState, from: usize, target: usize) -> bool {
    state.inherited[from].iter().any(|coverage| {
        coverage.parent_structure == target || reaches(state, coverage.parent_structure, target)
    })
}

fn check_term(
    state: &mut SemanticState,
    term: &SourceStructureTerm,
) -> Result<SourceStructureType, SourceStructurePayloadError> {
    if term.recovered() {
        return Err(SourceStructurePayloadError::RecoveredSyntax);
    }
    match term {
        SourceStructureTerm::Variable { symbol, .. } => state
            .variables
            .iter()
            .find(|(variable, _)| variable == symbol)
            .map(|(_, ty)| ty.clone())
            .ok_or(SourceStructurePayloadError::InvalidIdentity),
        SourceStructureTerm::Constructor {
            structure,
            type_arguments,
            arguments,
            source_range,
            ..
        } => {
            let Some(structure_index) = state.structure_index(structure) else {
                return Err(SourceStructurePayloadError::InvalidIdentity);
            };
            let parameter_count = state.structures[structure_index].parameters().len();
            if type_arguments.len() != parameter_count {
                return Err(SourceStructurePayloadError::TypeMismatch);
            }
            let field_count = state.structures[structure_index]
                .members()
                .iter()
                .filter(|member| member.kind() == SourceStructureMemberKind::Field)
                .count();
            let mut seen = Vec::new();
            for argument in arguments {
                if argument.recovered {
                    return Err(SourceStructurePayloadError::RecoveredSyntax);
                }
                let Some(member_symbol) = argument.member.as_ref() else {
                    return Err(SourceStructurePayloadError::InvalidIdentity);
                };
                let Some(member) = state.structures[structure_index].member(member_symbol) else {
                    return Err(SourceStructurePayloadError::InvalidIdentity);
                };
                if member.spelling() != argument.spelling
                    || member.kind() != SourceStructureMemberKind::Field
                    || seen.iter().any(|symbol: &SymbolId| symbol == member_symbol)
                {
                    return Err(SourceStructurePayloadError::InvalidIdentity);
                }
                let member_ty = member.ty().clone();
                let value_ty = check_term(state, &argument.value)?;
                if state.has_diagnostic() {
                    return Ok(SourceStructureType::Structure {
                        symbol: structure.clone(),
                        arguments: type_arguments.clone(),
                    });
                }
                if value_ty != member_ty {
                    return Err(SourceStructurePayloadError::TypeMismatch);
                }
                seen.push(member_symbol.clone());
            }
            if seen.len() != field_count {
                state.diagnostics.push(diagnostic(
                    SourceStructureDiagnosticPhase::TypeCheck,
                    *source_range,
                    "structures.constructor.missing_field_argument",
                ));
            }
            Ok(SourceStructureType::Structure {
                symbol: structure.clone(),
                arguments: type_arguments.clone(),
            })
        }
        SourceStructureTerm::Select {
            subject,
            member,
            spelling,
            source_range,
            ..
        } => {
            let subject_ty = check_term(state, subject)?;
            if state.has_diagnostic() {
                return Ok(SourceStructureType::Set);
            }
            let Some(SourceStructureType::Structure { symbol, .. }) = Some(subject_ty) else {
                return Err(SourceStructurePayloadError::TypeMismatch);
            };
            let Some(structure_index) = state.structure_index(&symbol) else {
                return Err(SourceStructurePayloadError::InvalidIdentity);
            };
            let Some(member_symbol) = member else {
                state.diagnostics.push(diagnostic(
                    SourceStructureDiagnosticPhase::TypeCheck,
                    *source_range,
                    "structures.selector.unknown_field",
                ));
                return Ok(SourceStructureType::Set);
            };
            let Some(selected) = state
                .effective_members(structure_index)
                .into_iter()
                .find(|candidate| candidate.symbol() == member_symbol)
            else {
                state.diagnostics.push(diagnostic(
                    SourceStructureDiagnosticPhase::TypeCheck,
                    *source_range,
                    "structures.selector.unknown_field",
                ));
                return Ok(SourceStructureType::Set);
            };
            if selected.spelling() != spelling {
                return Err(SourceStructurePayloadError::InvalidIdentity);
            }
            Ok(selected.ty().clone())
        }
        SourceStructureTerm::Update {
            subject,
            member,
            spelling,
            value,
            ..
        } => {
            let subject_ty = check_term(state, subject)?;
            if state.has_diagnostic() {
                return Ok(subject_ty);
            }
            let SourceStructureType::Structure { symbol, arguments } = subject_ty else {
                return Err(SourceStructurePayloadError::TypeMismatch);
            };
            let Some(structure_index) = state.structure_index(&symbol) else {
                return Err(SourceStructurePayloadError::InvalidIdentity);
            };
            let Some(member_symbol) = member else {
                return Err(SourceStructurePayloadError::InvalidIdentity);
            };
            let Some(selected) = state
                .effective_members(structure_index)
                .into_iter()
                .find(|candidate| candidate.symbol() == member_symbol)
            else {
                return Err(SourceStructurePayloadError::InvalidIdentity);
            };
            if selected.spelling() != spelling
                || selected.kind() != SourceStructureMemberKind::Field
            {
                return Err(SourceStructurePayloadError::UnsupportedShape);
            }
            let selected_ty = selected.ty().clone();
            let value_ty = check_term(state, value)?;
            if state.has_diagnostic() {
                return Ok(SourceStructureType::Structure { symbol, arguments });
            }
            if value_ty != selected_ty {
                return Err(SourceStructurePayloadError::TypeMismatch);
            }
            Ok(SourceStructureType::Structure { symbol, arguments })
        }
    }
}

fn check_equality(
    state: &mut SemanticState,
    equality: &SourceStructureEqualityInput,
) -> Result<(), SourceStructurePayloadError> {
    let left = check_term(state, &equality.left)?;
    if state.has_diagnostic() {
        return Ok(());
    }
    let right = check_term(state, &equality.right)?;
    if state.has_diagnostic() {
        return Ok(());
    }
    if left != right {
        return Err(SourceStructurePayloadError::TypeMismatch);
    }
    Ok(())
}

fn validate_input_shape(
    input: &SourceStructureProgramInput,
    symbols: &SymbolEnv,
) -> Result<(), SourceStructurePayloadError> {
    if input.module_id != *symbols.module_id() {
        return Err(SourceStructurePayloadError::ModuleMismatch);
    }
    let mut definition_symbols = BTreeSet::new();
    let mut member_symbols = BTreeSet::new();
    let mut variable_symbols = BTreeSet::new();
    let mut variable_spellings = BTreeSet::new();
    for definition in &input.definitions {
        if !definition_symbols.insert(definition.symbol.clone()) {
            return Err(SourceStructurePayloadError::DuplicateIdentity);
        }
        for member in &definition.members {
            if !member_symbols.insert(member.symbol.clone()) {
                return Err(SourceStructurePayloadError::DuplicateIdentity);
            }
        }
    }
    for variable in &input.variables {
        if !variable_symbols.insert(variable.symbol.clone())
            || !variable_spellings.insert(variable.spelling.as_str())
        {
            return Err(SourceStructurePayloadError::DuplicateIdentity);
        }
    }
    check_orders(&input.definitions, |definition| definition.source_ordinal)?;
    check_orders(&input.inheritances, |inheritance| {
        inheritance.source_ordinal
    })?;
    check_orders(&input.variables, |variable| variable.source_ordinal)?;
    check_orders(&input.terms, SourceStructureTerm::source_ordinal)?;
    check_orders(&input.claims, |claim| claim.source_ordinal)?;

    for definition in &input.definitions {
        check_range(input.source_id, definition.source_range)?;
        check_recovery(definition.recovered)?;
        authenticate_definition(
            input,
            symbols,
            &definition.symbol,
            &definition.spelling,
            Some(definition.source_range),
        )?;
        check_orders(&definition.members, |member| member.source_ordinal)?;
        for member in &definition.members {
            check_range(input.source_id, member.source_range)?;
            check_recovery(member.recovered)?;
            authenticate_member(
                input,
                symbols,
                &member.symbol,
                &member.spelling,
                &member.resolver_spelling,
                Some(member.source_range),
            )?;
            validate_type_symbols(input, symbols, &member.ty)?;
        }
    }
    for inheritance in &input.inheritances {
        check_range(input.source_id, inheritance.source_range)?;
        check_recovery(inheritance.recovered)?;
        if inheritance.coherence {
            return Err(SourceStructurePayloadError::UnsupportedShape);
        }
        authenticate_definition(input, symbols, &inheritance.child, "", None)?;
        if let SourceStructureInheritanceParent::Structure { symbol, arguments } =
            &inheritance.parent
        {
            authenticate_definition(input, symbols, symbol, "", None)?;
            for argument in arguments {
                validate_type_symbols(input, symbols, argument)?;
            }
        }
        check_orders(&inheritance.mappings, |mapping| mapping.source_ordinal)?;
        for mapping in &inheritance.mappings {
            check_range(input.source_id, mapping.source_range)?;
            check_recovery(mapping.recovered)?;
            if matches!(
                inheritance.parent,
                SourceStructureInheritanceParent::Structure { .. }
            ) && mapping.from_it
            {
                return Err(SourceStructurePayloadError::InvalidIdentity);
            }
            if let Some(symbol) = &mapping.child_member {
                authenticate_member(
                    input,
                    symbols,
                    symbol,
                    &mapping.child_spelling,
                    mapping
                        .child_resolver_spelling
                        .as_deref()
                        .unwrap_or(&mapping.child_spelling),
                    None,
                )?;
            }
            if let Some(symbol) = &mapping.parent_member {
                authenticate_member(
                    input,
                    symbols,
                    symbol,
                    &mapping.parent_spelling,
                    mapping
                        .parent_resolver_spelling
                        .as_deref()
                        .unwrap_or(&mapping.parent_spelling),
                    None,
                )?;
            }
        }
    }
    for variable in &input.variables {
        check_range(input.source_id, variable.source_range)?;
        check_recovery(variable.recovered)?;
        authenticate_variable(input, &variable.symbol, &variable.spelling)?;
        validate_type_symbols(input, symbols, &variable.ty)?;
    }
    for term in &input.terms {
        validate_term(input, symbols, term)?;
    }
    for claim in &input.claims {
        check_range(input.source_id, claim.source_range)?;
        check_recovery(claim.recovered)?;
        validate_equality(input, symbols, &claim.proposition)?;
        check_orders(&claim.conclusions, |equality| equality.source_ordinal)?;
        for conclusion in &claim.conclusions {
            validate_equality(input, symbols, conclusion)?;
        }
    }
    Ok(())
}

fn check_orders<T, F>(values: &[T], mut ordinal: F) -> Result<(), SourceStructurePayloadError>
where
    F: FnMut(&T) -> usize,
{
    if values
        .windows(2)
        .any(|window| ordinal(&window[0]) >= ordinal(&window[1]))
    {
        return Err(SourceStructurePayloadError::InvalidOrder);
    }
    Ok(())
}

fn check_range(source_id: SourceId, range: SourceRange) -> Result<(), SourceStructurePayloadError> {
    if range.source_id != source_id || range.start > range.end {
        return Err(if range.source_id != source_id {
            SourceStructurePayloadError::SourceMismatch
        } else {
            SourceStructurePayloadError::InvalidRange
        });
    }
    Ok(())
}

fn check_recovery(recovered: bool) -> Result<(), SourceStructurePayloadError> {
    if recovered {
        Err(SourceStructurePayloadError::RecoveredSyntax)
    } else {
        Ok(())
    }
}

fn validate_type_symbols(
    input: &SourceStructureProgramInput,
    symbols: &SymbolEnv,
    ty: &SourceStructureType,
) -> Result<(), SourceStructurePayloadError> {
    if let SourceStructureType::Structure { symbol, arguments } = ty {
        authenticate_definition(input, symbols, symbol, "", None)?;
        for argument in arguments {
            validate_type_symbols(input, symbols, argument)?;
        }
    }
    Ok(())
}

fn authenticate_variable(
    input: &SourceStructureProgramInput,
    symbol: &SymbolId,
    spelling: &str,
) -> Result<(), SourceStructurePayloadError> {
    if symbol.module() != &input.module_id {
        return Err(SourceStructurePayloadError::ModuleMismatch);
    }
    let expected_local = format!("step5c2/variable/{spelling}");
    let expected_fqn = format!(
        "{}::step5c2::variable::{spelling}",
        input.module_id.path().as_str()
    );
    if spelling.is_empty()
        || symbol.local().as_str() != expected_local
        || symbol.fqn().as_str() != expected_fqn
    {
        return Err(SourceStructurePayloadError::InvalidIdentity);
    }
    Ok(())
}

fn validate_term(
    input: &SourceStructureProgramInput,
    symbols: &SymbolEnv,
    term: &SourceStructureTerm,
) -> Result<(), SourceStructurePayloadError> {
    check_range(input.source_id, term.source_range())?;
    check_recovery(term.recovered())?;
    match term {
        SourceStructureTerm::Variable {
            symbol, spelling, ..
        } => {
            authenticate_variable(input, symbol, spelling)?;
            if !input
                .variables
                .iter()
                .any(|variable| variable.symbol == *symbol && variable.spelling == *spelling)
            {
                return Err(SourceStructurePayloadError::InvalidIdentity);
            }
        }
        SourceStructureTerm::Constructor {
            structure,
            type_arguments,
            arguments,
            ..
        } => {
            authenticate_definition(input, symbols, structure, "", None)?;
            for argument in type_arguments {
                validate_type_symbols(input, symbols, argument)?;
            }
            check_orders(arguments, |argument| argument.source_ordinal)?;
            for argument in arguments {
                check_range(input.source_id, argument.source_range)?;
                check_recovery(argument.recovered)?;
                if let Some(member) = &argument.member {
                    authenticate_member(
                        input,
                        symbols,
                        member,
                        &argument.spelling,
                        argument
                            .resolver_spelling
                            .as_deref()
                            .unwrap_or(&argument.spelling),
                        None,
                    )?;
                }
                validate_term(input, symbols, &argument.value)?;
            }
        }
        SourceStructureTerm::Select {
            subject,
            member,
            spelling,
            resolver_spelling,
            ..
        } => {
            validate_term(input, symbols, subject)?;
            if let Some(member) = member {
                authenticate_member(
                    input,
                    symbols,
                    member,
                    spelling,
                    resolver_spelling.as_deref().unwrap_or(spelling),
                    None,
                )?;
            }
        }
        SourceStructureTerm::Update {
            subject,
            member,
            spelling,
            resolver_spelling,
            value,
            ..
        } => {
            validate_term(input, symbols, subject)?;
            validate_term(input, symbols, value)?;
            if let Some(member) = member {
                authenticate_member(
                    input,
                    symbols,
                    member,
                    spelling,
                    resolver_spelling.as_deref().unwrap_or(spelling),
                    None,
                )?;
            }
        }
    }
    Ok(())
}

fn validate_equality(
    input: &SourceStructureProgramInput,
    symbols: &SymbolEnv,
    equality: &SourceStructureEqualityInput,
) -> Result<(), SourceStructurePayloadError> {
    check_range(input.source_id, equality.source_range)?;
    check_recovery(equality.recovered)?;
    validate_term(input, symbols, &equality.left)?;
    validate_term(input, symbols, &equality.right)?;
    Ok(())
}

fn authenticate_definition(
    input: &SourceStructureProgramInput,
    symbols: &SymbolEnv,
    symbol: &SymbolId,
    spelling: &str,
    range: Option<SourceRange>,
) -> Result<(), SourceStructurePayloadError> {
    if symbol.module() != &input.module_id {
        return Err(SourceStructurePayloadError::ModuleMismatch);
    }
    let Some(entry) = symbols.symbols().get(symbol) else {
        return Err(SourceStructurePayloadError::InvalidIdentity);
    };
    if entry.kind() != SymbolKind::Structure
        || (!spelling.is_empty() && entry.primary_spelling() != spelling)
        || entry.symbol().module() != &input.module_id
        || entry.origin().module_id() != &input.module_id
    {
        return Err(SourceStructurePayloadError::InvalidIdentity);
    }
    if symbols
        .symbols()
        .iter()
        .filter(|candidate| candidate.symbol() == symbol)
        .count()
        != 1
        || symbols
            .definitions()
            .iter()
            .filter(|definition| definition.symbol() == symbol)
            .count()
            != 1
    {
        return Err(SourceStructurePayloadError::DuplicateIdentity);
    }
    let Some(definition) = symbols.definitions().by_symbol(symbol) else {
        return Err(SourceStructurePayloadError::InvalidIdentity);
    };
    if definition.kind() != DefinitionKind::Structure {
        return Err(SourceStructurePayloadError::InvalidIdentity);
    }
    if let Some(range) = range {
        authenticate_origin(input.source_id, entry.origin().anchor(), range)?;
    }
    Ok(())
}

fn authenticate_member(
    input: &SourceStructureProgramInput,
    symbols: &SymbolEnv,
    symbol: &SymbolId,
    source_spelling: &str,
    resolver_spelling: &str,
    range: Option<SourceRange>,
) -> Result<(), SourceStructurePayloadError> {
    if source_spelling.is_empty() || resolver_spelling.is_empty() {
        return Err(SourceStructurePayloadError::InvalidIdentity);
    }
    if symbol.module() != &input.module_id {
        return Err(SourceStructurePayloadError::ModuleMismatch);
    }
    let Some(entry) = symbols.symbols().get(symbol) else {
        return Err(SourceStructurePayloadError::InvalidIdentity);
    };
    if entry.kind() != SymbolKind::Selector
        || entry.primary_spelling() != resolver_spelling
        || entry.origin().module_id() != &input.module_id
    {
        return Err(SourceStructurePayloadError::InvalidIdentity);
    }
    if symbols
        .symbols()
        .iter()
        .filter(|candidate| candidate.symbol() == symbol)
        .count()
        != 1
        || symbols
            .definitions()
            .iter()
            .filter(|definition| definition.symbol() == symbol)
            .count()
            != 1
    {
        return Err(SourceStructurePayloadError::DuplicateIdentity);
    }
    let Some(definition) = symbols.definitions().by_symbol(symbol) else {
        return Err(SourceStructurePayloadError::InvalidIdentity);
    };
    if definition.kind() != DefinitionKind::Selector {
        return Err(SourceStructurePayloadError::InvalidIdentity);
    }
    if let Some(range) = range {
        authenticate_origin(input.source_id, entry.origin().anchor(), range)?;
    }
    Ok(())
}

fn authenticate_origin(
    source_id: SourceId,
    anchor: &SourceAnchor,
    range: SourceRange,
) -> Result<(), SourceStructurePayloadError> {
    match anchor {
        SourceAnchor::Range(origin_range) => {
            if origin_range.source_id != source_id || *origin_range != range {
                return Err(SourceStructurePayloadError::InvalidIdentity);
            }
        }
        SourceAnchor::Point {
            source_id: origin_source,
            offset,
        } => {
            if *origin_source != source_id || range.start != *offset {
                return Err(SourceStructurePayloadError::InvalidIdentity);
            }
        }
        SourceAnchor::Generated(_) => return Err(SourceStructurePayloadError::InvalidIdentity),
        _ => return Err(SourceStructurePayloadError::InvalidIdentity),
    }
    Ok(())
}

fn diagnostic(
    phase: SourceStructureDiagnosticPhase,
    source_range: SourceRange,
    detail_key: &'static str,
) -> SourceStructureDiagnostic {
    SourceStructureDiagnostic {
        phase,
        source_range,
        detail_key,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mizar_resolve::env::{
        ContributionKind, DefinitionShell, NamespacePath, SymbolEntry, SymbolEnvIndexes,
        SymbolIndex,
    };
    use mizar_resolve::resolved_ast::{FullyQualifiedName, LocalSymbolId, SemanticOrigin};
    use mizar_session::{
        BuildSnapshotId, InMemorySessionIdAllocator, ModulePath, PackageId, SessionIdAllocator as _,
    };

    fn source() -> SourceId {
        let snapshot = BuildSnapshotId::from_published_schema_str(
            "mizar-session-build-snapshot-v1:5555555555555555555555555555555555555555555555555555555555555555",
        )
        .expect("snapshot");
        InMemorySessionIdAllocator::new()
            .next_source_id(snapshot)
            .expect("source")
    }

    fn module() -> ModuleId {
        ModuleId::new(PackageId::new("test"), ModulePath::new("structure"))
    }

    fn symbol(name: &str) -> SymbolId {
        let module = module();
        SymbolId::new(
            module.clone(),
            LocalSymbolId::new(name),
            FullyQualifiedName::new(format!("{}::{}", module.path().as_str(), name)),
        )
    }

    fn variable_symbol(name: &str) -> SymbolId {
        let module = module();
        SymbolId::new(
            module.clone(),
            LocalSymbolId::new(format!("step5c2/variable/{name}")),
            FullyQualifiedName::new(format!(
                "{}::step5c2::variable::{name}",
                module.path().as_str()
            )),
        )
    }

    fn range(source: SourceId, start: usize, end: usize) -> SourceRange {
        SourceRange {
            source_id: source,
            start,
            end,
        }
    }

    fn member(
        source: SourceId,
        owner: &str,
        name: &str,
        kind: SourceStructureMemberKind,
        ty: SourceStructureType,
        ordinal: usize,
    ) -> SourceStructureMemberInput {
        SourceStructureMemberInput {
            symbol: symbol(&format!("{owner}/{name}")),
            spelling: name.to_owned(),
            resolver_spelling: name.to_owned(),
            kind,
            ty,
            source_range: range(source, 100 + ordinal * 2, 101 + ordinal * 2),
            source_ordinal: ordinal,
            recovered: false,
        }
    }

    fn definition(
        source: SourceId,
        name: &str,
        ordinal: usize,
        members: Vec<SourceStructureMemberInput>,
    ) -> SourceStructureDefinitionInput {
        SourceStructureDefinitionInput {
            symbol: symbol(name),
            spelling: name.to_owned(),
            parameters: Vec::new(),
            members,
            source_range: range(source, ordinal * 20, ordinal * 20 + 10),
            source_ordinal: ordinal,
            recovered: false,
        }
    }

    fn resolver(definitions: &[SourceStructureDefinitionInput], source: SourceId) -> SymbolEnv {
        let module = module();
        let mut symbols = SymbolIndex::new();
        let mut definition_index = mizar_resolve::env::DefinitionIndex::new();
        let mut contributions = mizar_resolve::env::SourceContributionIndex::new();
        let contribution = contributions.insert(
            module.clone(),
            ContributionKind::LocalSource { source_id: source },
            SourceAnchor::Range(range(source, 0, 1)),
        );
        for definition in definitions {
            let structure_origin = SemanticOrigin::new(
                source,
                module.clone(),
                SourceAnchor::Range(definition.source_range),
                vec![definition.source_ordinal as u32],
            );
            symbols.insert(SymbolEntry::new(
                definition.symbol.clone(),
                SymbolKind::Structure,
                NamespacePath::new(module.path().as_str()),
                definition.spelling.clone(),
                structure_origin.clone(),
                contribution,
            ));
            definition_index.insert(DefinitionShell::new(
                definition.symbol.clone(),
                DefinitionKind::Structure,
                structure_origin,
                contribution,
            ));
            for member in &definition.members {
                let origin = SemanticOrigin::new(
                    source,
                    module.clone(),
                    SourceAnchor::Range(member.source_range),
                    vec![
                        definition.source_ordinal as u32,
                        member.source_ordinal as u32,
                    ],
                );
                symbols.insert(SymbolEntry::new(
                    member.symbol.clone(),
                    SymbolKind::Selector,
                    NamespacePath::new(module.path().as_str()),
                    member.resolver_spelling.clone(),
                    origin.clone(),
                    contribution,
                ));
                definition_index.insert(DefinitionShell::new(
                    member.symbol.clone(),
                    DefinitionKind::Selector,
                    origin,
                    contribution,
                ));
            }
        }
        SymbolEnv::new(
            module,
            SymbolEnvIndexes {
                symbols,
                definitions: definition_index,
                contributions,
                ..SymbolEnvIndexes::default()
            },
        )
    }

    fn check(
        source: SourceId,
        definitions: Vec<SourceStructureDefinitionInput>,
        inheritances: Vec<SourceStructureInheritanceInput>,
        variables: Vec<SourceStructureVariableInput>,
        terms: Vec<SourceStructureTerm>,
        claims: Vec<SourceStructureClaimInput>,
    ) -> Result<SourceStructureSemanticsOutput, SourceStructurePayloadError> {
        let env = resolver(&definitions, source);
        SourceStructureSemanticsChecker::check(
            SourceStructureProgramInput::new(
                source,
                module(),
                definitions,
                inheritances,
                variables,
                terms,
                claims,
            ),
            &env,
        )
    }

    fn field_arg(
        source: SourceId,
        owner: &str,
        name: &str,
        value: SourceStructureTerm,
        ordinal: usize,
    ) -> SourceStructureFieldArgument {
        SourceStructureFieldArgument {
            member: Some(symbol(&format!("{owner}/{name}"))),
            spelling: name.to_owned(),
            resolver_spelling: None,
            value: Box::new(value),
            source_range: range(source, 200 + ordinal * 2, 201 + ordinal * 2),
            source_ordinal: ordinal,
            recovered: false,
        }
    }

    fn variable_input(
        source: SourceId,
        name: &str,
        ordinal: usize,
    ) -> SourceStructureVariableInput {
        SourceStructureVariableInput {
            symbol: variable_symbol(name),
            spelling: name.to_owned(),
            ty: SourceStructureType::Set,
            source_range: range(source, 300 + ordinal, 301 + ordinal),
            source_ordinal: ordinal,
            recovered: false,
        }
    }

    fn variable_term(source: SourceId, name: &str, ordinal: usize) -> SourceStructureTerm {
        SourceStructureTerm::Variable {
            symbol: variable_symbol(name),
            spelling: name.to_owned(),
            source_range: range(source, 400 + ordinal, 401 + ordinal),
            source_ordinal: ordinal,
            recovered: false,
        }
    }

    #[test]
    fn basic_and_property_definitions_are_accepted() {
        let source = source();
        let definitions = vec![definition(
            source,
            "Box",
            0,
            vec![
                member(
                    source,
                    "Box",
                    "value",
                    SourceStructureMemberKind::Field,
                    SourceStructureType::Set,
                    0,
                ),
                member(
                    source,
                    "Box",
                    "property",
                    SourceStructureMemberKind::Property,
                    SourceStructureType::Set,
                    1,
                ),
            ],
        )];
        let output = check(source, definitions, vec![], vec![], vec![], vec![]).expect("accepted");
        assert_eq!(output.structures().len(), 1);
        assert_eq!(output.structures()[0].members().len(), 2);
        assert!(output.diagnostics().is_empty());
    }

    #[test]
    fn resolver_projection_spelling_is_separate_from_source_member_spelling() {
        let source = source();
        let mut definition = definition(
            source,
            "GLeft",
            0,
            vec![member(
                source,
                "GLeft",
                "payload",
                SourceStructureMemberKind::Field,
                SourceStructureType::Set,
                0,
            )],
        );
        definition.members[0].resolver_spelling = "GLeftPayload".to_owned();
        let variables = vec![SourceStructureVariableInput::new(
            variable_symbol("g"),
            "g",
            SourceStructureType::Structure {
                symbol: symbol("GLeft"),
                arguments: vec![],
            },
            range(source, 300, 301),
            0,
            false,
        )];
        let select = SourceStructureTerm::Select {
            subject: Box::new(SourceStructureTerm::Variable {
                symbol: variable_symbol("g"),
                spelling: "g".to_owned(),
                source_range: range(source, 400, 401),
                source_ordinal: 0,
                recovered: false,
            }),
            member: Some(symbol("GLeft/payload")),
            spelling: "payload".to_owned(),
            resolver_spelling: Some("GLeftPayload".to_owned()),
            source_range: range(source, 401, 410),
            source_ordinal: 1,
            recovered: false,
        };
        let output = check(
            source,
            vec![definition],
            vec![],
            variables,
            vec![select],
            vec![],
        )
        .expect("resolver spelling is authenticated separately");
        assert!(output.diagnostics().is_empty());
        assert_eq!(output.terms().len(), 1);
    }

    #[test]
    fn duplicate_member_is_resolve_diagnostic_and_stops() {
        let source = source();
        let mut duplicate = definition(
            source,
            "Dup",
            0,
            vec![
                member(
                    source,
                    "Dup",
                    "x",
                    SourceStructureMemberKind::Field,
                    SourceStructureType::Set,
                    0,
                ),
                member(
                    source,
                    "Dup",
                    "x",
                    SourceStructureMemberKind::Field,
                    SourceStructureType::Set,
                    1,
                ),
            ],
        );
        duplicate.members[1].symbol = symbol("Dup/x2");
        let definitions = vec![duplicate];
        let output =
            check(source, definitions, vec![], vec![], vec![], vec![]).expect("diagnostic output");
        assert_eq!(
            output.diagnostics()[0].phase(),
            SourceStructureDiagnosticPhase::Resolve
        );
        assert_eq!(
            output.diagnostics()[0].detail_key(),
            "structures.definition.duplicate_member"
        );
        assert!(output.structures().is_empty());
    }

    #[test]
    fn constructor_missing_field_and_selector_are_type_diagnostics() {
        let source = source();
        let definitions = vec![definition(
            source,
            "Pair",
            0,
            vec![
                member(
                    source,
                    "Pair",
                    "first",
                    SourceStructureMemberKind::Field,
                    SourceStructureType::Set,
                    0,
                ),
                member(
                    source,
                    "Pair",
                    "second",
                    SourceStructureMemberKind::Field,
                    SourceStructureType::Set,
                    1,
                ),
            ],
        )];
        let variables = vec![variable_input(source, "a", 2)];
        let constructor = SourceStructureTerm::Constructor {
            structure: symbol("Pair"),
            type_arguments: vec![],
            arguments: vec![field_arg(
                source,
                "Pair",
                "first",
                variable_term(source, "a", 3),
                3,
            )],
            source_range: range(source, 500, 510),
            source_ordinal: 3,
            recovered: false,
        };
        let output = check(
            source,
            definitions.clone(),
            vec![],
            variables.clone(),
            vec![constructor],
            vec![],
        )
        .expect("diagnostic output");
        assert_eq!(
            output.diagnostics()[0].detail_key(),
            "structures.constructor.missing_field_argument"
        );
        assert!(output.terms().is_empty());

        let unknown = SourceStructureTerm::Select {
            subject: Box::new(SourceStructureTerm::Variable {
                symbol: variable_symbol("pair"),
                spelling: "pair".to_owned(),
                source_range: range(source, 520, 521),
                source_ordinal: 4,
                recovered: false,
            }),
            member: None,
            spelling: "third".to_owned(),
            resolver_spelling: None,
            source_range: range(source, 521, 526),
            source_ordinal: 5,
            recovered: false,
        };
        let mut vars = variables;
        vars.push(SourceStructureVariableInput {
            symbol: variable_symbol("pair"),
            spelling: "pair".to_owned(),
            ty: SourceStructureType::Structure {
                symbol: symbol("Pair"),
                arguments: vec![],
            },
            source_range: range(source, 530, 531),
            source_ordinal: 4,
            recovered: false,
        });
        let output = check(source, definitions, vec![], vars, vec![unknown], vec![])
            .expect("diagnostic output");
        assert_eq!(
            output.diagnostics()[0].detail_key(),
            "structures.selector.unknown_field"
        );
    }

    #[test]
    fn exact_constructor_update_and_claim_are_accepted() {
        let source = source();
        let definitions = vec![definition(
            source,
            "Pair",
            0,
            vec![
                member(
                    source,
                    "Pair",
                    "first",
                    SourceStructureMemberKind::Field,
                    SourceStructureType::Set,
                    0,
                ),
                member(
                    source,
                    "Pair",
                    "second",
                    SourceStructureMemberKind::Field,
                    SourceStructureType::Set,
                    1,
                ),
            ],
        )];
        let variables = vec![
            variable_input(source, "a", 2),
            variable_input(source, "b", 3),
        ];
        let ctor = SourceStructureTerm::Constructor {
            structure: symbol("Pair"),
            type_arguments: vec![],
            arguments: vec![
                field_arg(source, "Pair", "first", variable_term(source, "a", 4), 4),
                field_arg(source, "Pair", "second", variable_term(source, "b", 5), 5),
            ],
            source_range: range(source, 500, 520),
            source_ordinal: 4,
            recovered: false,
        };
        let update = SourceStructureTerm::Update {
            subject: Box::new(ctor.clone()),
            member: Some(symbol("Pair/second")),
            spelling: "second".to_owned(),
            resolver_spelling: None,
            value: Box::new(variable_term(source, "a", 6)),
            source_range: range(source, 530, 540),
            source_ordinal: 6,
            recovered: false,
        };
        let claim = SourceStructureClaimInput {
            proposition: SourceStructureEqualityInput {
                left: SourceStructureTerm::Select {
                    subject: Box::new(ctor.clone()),
                    member: Some(symbol("Pair/first")),
                    spelling: "first".to_owned(),
                    resolver_spelling: None,
                    source_range: range(source, 550, 555),
                    source_ordinal: 7,
                    recovered: false,
                },
                right: variable_term(source, "a", 8),
                source_range: range(source, 550, 560),
                source_ordinal: 7,
                recovered: false,
            },
            conclusions: vec![],
            source_range: range(source, 550, 560),
            source_ordinal: 7,
            recovered: false,
        };
        let output = check(
            source,
            definitions,
            vec![],
            variables,
            vec![ctor, update],
            vec![claim],
        )
        .expect("accepted");
        assert_eq!(output.terms().len(), 2);
        assert_eq!(output.claims().len(), 1);
        assert!(output.diagnostics().is_empty());
    }

    #[test]
    fn inheritance_covers_rename_set_and_diamond_rules() {
        let source = source();
        let base = definition(
            source,
            "Base",
            0,
            vec![member(
                source,
                "Base",
                "data",
                SourceStructureMemberKind::Field,
                SourceStructureType::Set,
                0,
            )],
        );
        let child = definition(
            source,
            "Child",
            1,
            vec![member(
                source,
                "Child",
                "content",
                SourceStructureMemberKind::Field,
                SourceStructureType::Set,
                2,
            )],
        );
        let rename = SourceStructureInheritanceInput {
            child: symbol("Child"),
            parent: SourceStructureInheritanceParent::Structure {
                symbol: symbol("Base"),
                arguments: vec![],
            },
            explicit: true,
            mappings: vec![SourceStructureInheritanceMappingInput {
                child_member: Some(symbol("Child/content")),
                child_spelling: "content".to_owned(),
                child_resolver_spelling: None,
                parent_member: Some(symbol("Base/data")),
                parent_spelling: "data".to_owned(),
                parent_resolver_spelling: None,
                from_it: false,
                kind: SourceStructureMemberKind::Field,
                source_range: range(source, 600, 610),
                source_ordinal: 3,
                recovered: false,
            }],
            coherence: false,
            source_range: range(source, 590, 615),
            source_ordinal: 3,
            recovered: false,
        };
        assert!(
            check(
                source,
                vec![base.clone(), child.clone()],
                vec![rename],
                vec![],
                vec![],
                vec![]
            )
            .expect("accepted")
            .diagnostics()
            .is_empty()
        );

        let set_child = definition(
            source,
            "SetChild",
            2,
            vec![member(
                source,
                "SetChild",
                "carrier",
                SourceStructureMemberKind::Field,
                SourceStructureType::Set,
                4,
            )],
        );
        let set_edge = SourceStructureInheritanceInput {
            child: symbol("SetChild"),
            parent: SourceStructureInheritanceParent::Set,
            explicit: true,
            mappings: vec![SourceStructureInheritanceMappingInput {
                child_member: Some(symbol("SetChild/carrier")),
                child_spelling: "carrier".to_owned(),
                child_resolver_spelling: None,
                parent_member: None,
                parent_spelling: "it".to_owned(),
                parent_resolver_spelling: None,
                from_it: true,
                kind: SourceStructureMemberKind::Field,
                source_range: range(source, 620, 630),
                source_ordinal: 5,
                recovered: false,
            }],
            coherence: false,
            source_range: range(source, 620, 635),
            source_ordinal: 5,
            recovered: false,
        };
        assert!(
            check(
                source,
                vec![base, child, set_child],
                vec![set_edge],
                vec![],
                vec![],
                vec![]
            )
            .expect("accepted")
            .diagnostics()
            .is_empty()
        );
    }

    #[test]
    fn shorthand_uncovered_and_unknown_source_are_distinct() {
        let source = source();
        let base = definition(
            source,
            "Base",
            0,
            vec![member(
                source,
                "Base",
                "data",
                SourceStructureMemberKind::Field,
                SourceStructureType::Set,
                0,
            )],
        );
        let child = definition(
            source,
            "Child",
            1,
            vec![member(
                source,
                "Child",
                "other",
                SourceStructureMemberKind::Field,
                SourceStructureType::Set,
                2,
            )],
        );
        let shorthand = SourceStructureInheritanceInput {
            child: symbol("Child"),
            parent: SourceStructureInheritanceParent::Structure {
                symbol: symbol("Base"),
                arguments: vec![],
            },
            explicit: false,
            mappings: vec![],
            coherence: false,
            source_range: range(source, 590, 595),
            source_ordinal: 3,
            recovered: false,
        };
        let output = check(
            source,
            vec![base.clone(), child.clone()],
            vec![shorthand],
            vec![],
            vec![],
            vec![],
        )
        .expect("diagnostic output");
        assert_eq!(
            output.diagnostics()[0].detail_key(),
            "structures.inherit.uncovered_base_member"
        );

        let explicit = SourceStructureInheritanceInput {
            child: symbol("Child"),
            parent: SourceStructureInheritanceParent::Structure {
                symbol: symbol("Base"),
                arguments: vec![],
            },
            explicit: true,
            mappings: vec![SourceStructureInheritanceMappingInput {
                child_member: Some(symbol("Child/other")),
                child_spelling: "other".to_owned(),
                child_resolver_spelling: None,
                parent_member: None,
                parent_spelling: "payload".to_owned(),
                parent_resolver_spelling: None,
                from_it: false,
                kind: SourceStructureMemberKind::Field,
                source_range: range(source, 600, 610),
                source_ordinal: 4,
                recovered: false,
            }],
            coherence: false,
            source_range: range(source, 600, 615),
            source_ordinal: 4,
            recovered: false,
        };
        let output = check(
            source,
            vec![base, child],
            vec![explicit],
            vec![],
            vec![],
            vec![],
        )
        .expect("diagnostic output");
        assert_eq!(
            output.diagnostics()[0].detail_key(),
            "structures.inherit.unknown_source_member"
        );
    }

    #[test]
    fn diamond_type_identity_is_exact() {
        let source = source();
        let left_type = definition(source, "LeftType", 0, vec![]);
        let right_type = definition(source, "RightType", 1, vec![]);
        let left = definition(
            source,
            "Left",
            2,
            vec![member(
                source,
                "Left",
                "payload",
                SourceStructureMemberKind::Field,
                SourceStructureType::Structure {
                    symbol: symbol("LeftType"),
                    arguments: vec![],
                },
                4,
            )],
        );
        let right = definition(
            source,
            "Right",
            3,
            vec![member(
                source,
                "Right",
                "payload",
                SourceStructureMemberKind::Field,
                SourceStructureType::Structure {
                    symbol: symbol("RightType"),
                    arguments: vec![],
                },
                5,
            )],
        );
        let top = definition(
            source,
            "Top",
            4,
            vec![member(
                source,
                "Top",
                "payload",
                SourceStructureMemberKind::Field,
                SourceStructureType::Structure {
                    symbol: symbol("LeftType"),
                    arguments: vec![],
                },
                6,
            )],
        );
        let edges = vec![
            shorthand_edge(source, "Top", "Left", 7),
            shorthand_edge(source, "Top", "Right", 8),
        ];
        let output = check(
            source,
            vec![left_type, right_type, left, right, top],
            edges,
            vec![],
            vec![],
            vec![],
        )
        .expect("diagnostic output");
        assert_eq!(
            output.diagnostics()[0].detail_key(),
            "structures.inherit.diamond_inconsistency"
        );
    }

    fn shorthand_edge(
        source: SourceId,
        child: &str,
        parent: &str,
        ordinal: usize,
    ) -> SourceStructureInheritanceInput {
        SourceStructureInheritanceInput {
            child: symbol(child),
            parent: SourceStructureInheritanceParent::Structure {
                symbol: symbol(parent),
                arguments: vec![],
            },
            explicit: false,
            mappings: vec![],
            coherence: false,
            source_range: range(source, 700 + ordinal, 701 + ordinal),
            source_ordinal: ordinal,
            recovered: false,
        }
    }

    #[test]
    fn malformed_order_recovery_source_and_identity_fail_closed() {
        let source = source();
        let mut first = definition(
            source,
            "A",
            1,
            vec![member(
                source,
                "A",
                "x",
                SourceStructureMemberKind::Field,
                SourceStructureType::Set,
                2,
            )],
        );
        let second = definition(
            source,
            "B",
            0,
            vec![member(
                source,
                "B",
                "x",
                SourceStructureMemberKind::Field,
                SourceStructureType::Set,
                1,
            )],
        );
        let env = resolver(&[first.clone(), second.clone()], source);
        let input = SourceStructureProgramInput::new(
            source,
            module(),
            vec![first.clone(), second],
            vec![],
            vec![],
            vec![],
            vec![],
        );
        assert_eq!(
            SourceStructureSemanticsChecker::check(input, &env),
            Err(SourceStructurePayloadError::InvalidOrder)
        );

        first.recovered = true;
        let env = resolver(&[first.clone()], source);
        let input = SourceStructureProgramInput::new(
            source,
            module(),
            vec![first],
            vec![],
            vec![],
            vec![],
            vec![],
        );
        assert_eq!(
            SourceStructureSemanticsChecker::check(input, &env),
            Err(SourceStructurePayloadError::RecoveredSyntax)
        );

        let wrong_module = ModuleId::new(PackageId::new("other"), ModulePath::new("structure"));
        let env = resolver(&[definition(source, "C", 0, vec![])], source);
        let input = SourceStructureProgramInput::new(
            source,
            wrong_module,
            vec![],
            vec![],
            vec![],
            vec![],
            vec![],
        );
        assert_eq!(
            SourceStructureSemanticsChecker::check(input, &env),
            Err(SourceStructurePayloadError::ModuleMismatch)
        );
    }

    #[test]
    fn source_range_identity_and_unsupported_boundaries_fail_closed() {
        let source = source();
        let allocator = InMemorySessionIdAllocator::new();
        let snapshot = BuildSnapshotId::from_published_schema_str(
            "mizar-session-build-snapshot-v1:6666666666666666666666666666666666666666666666666666666666666666",
        )
        .expect("snapshot");
        let _ = allocator.next_source_id(snapshot).expect("first source");
        let other_source = allocator.next_source_id(snapshot).expect("other source");
        let empty_env = resolver(&[], source);
        let mut variable = variable_input(source, "x", 0);
        variable.source_range = range(other_source, 0, 1);
        let input = SourceStructureProgramInput::new(
            source,
            module(),
            vec![],
            vec![],
            vec![variable],
            vec![],
            vec![],
        );
        assert_eq!(
            SourceStructureSemanticsChecker::check(input, &empty_env),
            Err(SourceStructurePayloadError::SourceMismatch)
        );

        let mut variable = variable_input(source, "x", 0);
        variable.source_range = range(source, 2, 1);
        let input = SourceStructureProgramInput::new(
            source,
            module(),
            vec![],
            vec![],
            vec![variable],
            vec![],
            vec![],
        );
        assert_eq!(
            SourceStructureSemanticsChecker::check(input, &empty_env),
            Err(SourceStructurePayloadError::InvalidRange)
        );

        let mut forged_definition = definition(source, "Box", 0, vec![]);
        let env = resolver(&[forged_definition.clone()], source);
        forged_definition.spelling = "Forged".to_owned();
        let input = SourceStructureProgramInput::new(
            source,
            module(),
            vec![forged_definition],
            vec![],
            vec![],
            vec![],
            vec![],
        );
        assert_eq!(
            SourceStructureSemanticsChecker::check(input, &env),
            Err(SourceStructurePayloadError::InvalidIdentity)
        );

        let mut duplicate_member = member(
            source,
            "Dup",
            "left",
            SourceStructureMemberKind::Field,
            SourceStructureType::Set,
            0,
        );
        let mut reused = duplicate_member.clone();
        reused.spelling = "right".to_owned();
        reused.source_range = range(source, 104, 105);
        reused.source_ordinal = 1;
        let duplicate = definition(source, "Dup", 0, vec![duplicate_member.clone(), reused]);
        let input = SourceStructureProgramInput::new(
            source,
            module(),
            vec![duplicate],
            vec![],
            vec![],
            vec![],
            vec![],
        );
        assert_eq!(
            SourceStructureSemanticsChecker::check(input, &empty_env),
            Err(SourceStructurePayloadError::DuplicateIdentity)
        );

        duplicate_member.symbol = SymbolId::new(
            ModuleId::new(PackageId::new("other"), ModulePath::new("structure")),
            LocalSymbolId::new("x"),
            FullyQualifiedName::new("other::structure::x"),
        );
        let term = SourceStructureTerm::Variable {
            symbol: duplicate_member.symbol,
            spelling: "x".to_owned(),
            source_range: range(source, 200, 201),
            source_ordinal: 0,
            recovered: false,
        };
        let input = SourceStructureProgramInput::new(
            source,
            module(),
            vec![],
            vec![],
            vec![],
            vec![term],
            vec![],
        );
        assert_eq!(
            SourceStructureSemanticsChecker::check(input, &empty_env),
            Err(SourceStructurePayloadError::ModuleMismatch)
        );

        let forged_variable = SourceStructureVariableInput::new(
            symbol("x"),
            "x",
            SourceStructureType::Set,
            range(source, 300, 301),
            0,
            false,
        );
        let input = SourceStructureProgramInput::new(
            source,
            module(),
            vec![],
            vec![],
            vec![forged_variable],
            vec![],
            vec![],
        );
        assert_eq!(
            SourceStructureSemanticsChecker::check(input, &empty_env),
            Err(SourceStructurePayloadError::InvalidIdentity)
        );

        let variable = variable_input(source, "x", 0);
        let mismatched_reference = SourceStructureTerm::Variable {
            symbol: variable_symbol("x"),
            spelling: "y".to_owned(),
            source_range: range(source, 400, 401),
            source_ordinal: 0,
            recovered: false,
        };
        let input = SourceStructureProgramInput::new(
            source,
            module(),
            vec![],
            vec![],
            vec![variable],
            vec![mismatched_reference],
            vec![],
        );
        assert_eq!(
            SourceStructureSemanticsChecker::check(input, &empty_env),
            Err(SourceStructurePayloadError::InvalidIdentity)
        );
    }

    #[test]
    fn malformed_inheritance_flags_fail_closed() {
        let source = source();
        let base = definition(
            source,
            "Base",
            0,
            vec![member(
                source,
                "Base",
                "value",
                SourceStructureMemberKind::Field,
                SourceStructureType::Set,
                0,
            )],
        );
        let child = definition(
            source,
            "Child",
            1,
            vec![member(
                source,
                "Child",
                "value",
                SourceStructureMemberKind::Field,
                SourceStructureType::Set,
                0,
            )],
        );
        let definitions = vec![base, child];
        let env = resolver(&definitions, source);
        let mapping = SourceStructureInheritanceMappingInput::new(
            Some(symbol("Child/value")),
            "value",
            Some("value".to_owned()),
            Some(symbol("Base/value")),
            "value",
            Some("value".to_owned()),
            true,
            SourceStructureMemberKind::Field,
            range(source, 50, 51),
            0,
            false,
        );
        let inheritance = SourceStructureInheritanceInput::new(
            symbol("Child"),
            SourceStructureInheritanceParent::Structure {
                symbol: symbol("Base"),
                arguments: vec![],
            },
            true,
            vec![mapping],
            false,
            range(source, 45, 55),
            0,
            false,
        );
        let input = SourceStructureProgramInput::new(
            source,
            module(),
            definitions.clone(),
            vec![inheritance],
            vec![],
            vec![],
            vec![],
        );
        assert_eq!(
            SourceStructureSemanticsChecker::check(input, &env),
            Err(SourceStructurePayloadError::InvalidIdentity)
        );

        let coherence = SourceStructureInheritanceInput::new(
            symbol("Child"),
            SourceStructureInheritanceParent::Structure {
                symbol: symbol("Base"),
                arguments: vec![],
            },
            false,
            vec![],
            true,
            range(source, 45, 55),
            0,
            false,
        );
        let input = SourceStructureProgramInput::new(
            source,
            module(),
            definitions,
            vec![coherence],
            vec![],
            vec![],
            vec![],
        );
        assert_eq!(
            SourceStructureSemanticsChecker::check(input, &env),
            Err(SourceStructurePayloadError::UnsupportedShape)
        );
    }

    #[test]
    fn first_diagnostic_is_global_source_order_and_stops_nested_terms() {
        let source = source();
        let base = definition(
            source,
            "Base",
            0,
            vec![member(
                source,
                "Base",
                "value",
                SourceStructureMemberKind::Field,
                SourceStructureType::Set,
                0,
            )],
        );
        let child = definition(
            source,
            "Child",
            1,
            vec![member(
                source,
                "Child",
                "value",
                SourceStructureMemberKind::Field,
                SourceStructureType::Set,
                0,
            )],
        );
        let mut late = definition(
            source,
            "Late",
            2,
            vec![
                member(
                    source,
                    "Late",
                    "dup",
                    SourceStructureMemberKind::Field,
                    SourceStructureType::Set,
                    0,
                ),
                member(
                    source,
                    "Late",
                    "dup",
                    SourceStructureMemberKind::Field,
                    SourceStructureType::Set,
                    1,
                ),
            ],
        );
        late.members[1].symbol = symbol("Late/dup2");
        let definitions = vec![base, child, late];
        let env = resolver(&definitions, source);
        let mapping = SourceStructureInheritanceMappingInput::new(
            Some(symbol("Child/value")),
            "value",
            Some("value".to_owned()),
            None,
            "missing",
            None,
            false,
            SourceStructureMemberKind::Field,
            range(source, 35, 36),
            0,
            false,
        );
        let inheritance = SourceStructureInheritanceInput::new(
            symbol("Child"),
            SourceStructureInheritanceParent::Structure {
                symbol: symbol("Base"),
                arguments: vec![],
            },
            true,
            vec![mapping],
            false,
            range(source, 34, 37),
            0,
            false,
        );
        let output = SourceStructureSemanticsChecker::check(
            SourceStructureProgramInput::new(
                source,
                module(),
                definitions,
                vec![inheritance],
                vec![],
                vec![],
                vec![],
            ),
            &env,
        )
        .expect("semantic diagnostic output");
        assert_eq!(output.diagnostics().len(), 1);
        assert_eq!(
            output.diagnostics()[0].detail_key(),
            "structures.inherit.unknown_source_member"
        );
        assert_eq!(output.structures().len(), 2);

        let pair = definition(
            source,
            "Pair",
            0,
            vec![member(
                source,
                "Pair",
                "value",
                SourceStructureMemberKind::Field,
                SourceStructureType::Set,
                0,
            )],
        );
        let env = resolver(std::slice::from_ref(&pair), source);
        let variables = vec![SourceStructureVariableInput::new(
            variable_symbol("p"),
            "p",
            SourceStructureType::Structure {
                symbol: symbol("Pair"),
                arguments: vec![],
            },
            range(source, 300, 301),
            0,
            false,
        )];
        let unknown = |ordinal| SourceStructureTerm::Select {
            subject: Box::new(SourceStructureTerm::Variable {
                symbol: variable_symbol("p"),
                spelling: "p".to_owned(),
                source_range: range(source, 400 + ordinal, 401 + ordinal),
                source_ordinal: ordinal,
                recovered: false,
            }),
            member: None,
            spelling: "missing".to_owned(),
            resolver_spelling: None,
            source_range: range(source, 410 + ordinal, 411 + ordinal),
            source_ordinal: ordinal,
            recovered: false,
        };
        let equality = SourceStructureEqualityInput::new(
            unknown(0),
            unknown(2),
            range(source, 390, 420),
            0,
            false,
        );
        let claim = SourceStructureClaimInput::new(
            equality.clone(),
            vec![equality],
            range(source, 380, 430),
            0,
            false,
        );
        let output = SourceStructureSemanticsChecker::check(
            SourceStructureProgramInput::new(
                source,
                module(),
                vec![pair],
                vec![],
                variables,
                vec![],
                vec![claim],
            ),
            &env,
        )
        .expect("semantic diagnostic output");
        assert_eq!(output.diagnostics().len(), 1);
        assert_eq!(
            output.diagnostics()[0].detail_key(),
            "structures.selector.unknown_field"
        );
        assert!(output.claims().is_empty());
    }

    #[test]
    fn dependent_parameter_arity_is_checked_without_coercion() {
        let source = source();
        let mut definition = definition(source, "Slice", 0, vec![]);
        definition.parameters = vec!["X".to_owned()];
        let good = SourceStructureTerm::Constructor {
            structure: symbol("Slice"),
            type_arguments: vec![SourceStructureType::Set],
            arguments: vec![],
            source_range: range(source, 800, 810),
            source_ordinal: 1,
            recovered: false,
        };
        let output = check(
            source,
            vec![definition.clone()],
            vec![],
            vec![],
            vec![good],
            vec![],
        )
        .expect("accepted");
        assert!(output.diagnostics().is_empty());
        let bad = SourceStructureTerm::Constructor {
            structure: symbol("Slice"),
            type_arguments: vec![],
            arguments: vec![],
            source_range: range(source, 820, 830),
            source_ordinal: 2,
            recovered: false,
        };
        assert_eq!(
            check(source, vec![definition], vec![], vec![], vec![bad], vec![]),
            Err(SourceStructurePayloadError::TypeMismatch)
        );
    }
}
