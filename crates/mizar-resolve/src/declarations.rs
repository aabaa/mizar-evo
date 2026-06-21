//! Source-shaped declaration shell collection.
//!
//! This module walks represented `SurfaceAst` nodes and records resolver-owned
//! declaration shells without assigning final symbol identities, checking
//! signatures, or validating export legality.

use crate::recovery::surface_contains_recovery as contains_recovery;
use crate::resolved_ast::ModuleId;
use mizar_session::SourceRange;
use mizar_syntax::{SurfaceAst, SurfaceNodeId, SurfaceNodeKind, SurfaceNodeView, SyntaxKind};
use std::cmp::Ordering;

/// Stable id for a source-shaped declaration shell.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DeclarationShellId(usize);

impl DeclarationShellId {
    const fn new(index: usize) -> Self {
        Self(index)
    }

    /// Returns the zero-based shell index.
    #[must_use]
    pub const fn index(self) -> usize {
        self.0
    }
}

/// Stable id for a source-shaped export projection shell.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ExportProjectionShellId(usize);

impl ExportProjectionShellId {
    const fn new(index: usize) -> Self {
        Self(index)
    }

    /// Returns the zero-based export projection index.
    #[must_use]
    pub const fn index(self) -> usize {
        self.0
    }
}

/// Source-shaped declaration shell collection result.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DeclarationShellSet {
    declarations: Vec<DeclarationShell>,
    exports: Vec<ExportProjectionShell>,
}

impl DeclarationShellSet {
    fn new(
        mut declarations: Vec<DeclarationShell>,
        mut exports: Vec<ExportProjectionShell>,
    ) -> Self {
        declarations.sort_by(declaration_shell_cmp);
        exports.sort_by(export_projection_shell_cmp);
        Self {
            declarations,
            exports,
        }
    }

    /// Returns declaration shells in source order.
    #[must_use]
    pub fn declarations(&self) -> &[DeclarationShell] {
        &self.declarations
    }

    /// Returns export projection shells in source order.
    #[must_use]
    pub fn exports(&self) -> &[ExportProjectionShell] {
        &self.exports
    }

    /// Returns a declaration shell by id.
    #[must_use]
    pub fn declaration(&self, id: DeclarationShellId) -> Option<&DeclarationShell> {
        self.declarations.iter().find(|shell| shell.id() == id)
    }

    /// Returns an export projection shell by id.
    #[must_use]
    pub fn export(&self, id: ExportProjectionShellId) -> Option<&ExportProjectionShell> {
        self.exports.iter().find(|export| export.id() == id)
    }

    /// Returns whether no shells were collected.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.declarations.is_empty() && self.exports.is_empty()
    }
}

/// Source-shaped declaration shell kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum DeclarationShellKind {
    /// Parser placeholder item preserved as a recovered or unresolved shell.
    Placeholder,
    /// Top-level reserve declaration.
    Reserve,
    /// Theorem item.
    Theorem,
    /// Lemma item.
    Lemma,
    /// Definition block item.
    DefinitionBlock,
    /// Registration block item.
    RegistrationBlock,
    /// Top-level claim block item.
    ClaimBlock,
    /// Attribute definition.
    AttributeDefinition,
    /// Predicate definition.
    PredicateDefinition,
    /// Functor definition.
    FunctorDefinition,
    /// Mode definition.
    ModeDefinition,
    /// Structure definition.
    StructureDefinition,
    /// Algorithm definition.
    AlgorithmDefinition,
    /// Attribute redefinition.
    AttributeRedefinition,
    /// Predicate redefinition.
    PredicateRedefinition,
    /// Functor redefinition.
    FunctorRedefinition,
    /// Notation synonym or antonym alias.
    NotationAlias,
    /// Property clause.
    PropertyClause,
    /// Structure field declaration.
    StructureField,
    /// Structure property declaration.
    StructureProperty,
    /// Inheritance definition.
    InheritanceDefinition,
    /// Field redefinition.
    FieldRedefinition,
    /// Property redefinition.
    PropertyRedefinition,
    /// Existential registration.
    ExistentialRegistration,
    /// Conditional registration.
    ConditionalRegistration,
    /// Functorial registration.
    FunctorialRegistration,
    /// Reduction registration.
    ReductionRegistration,
    /// Recovered visibility wrapper without a represented target.
    VisibilityWrapper,
}

/// Source-shaped visibility attached to a declaration shell.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeclarationShellVisibility {
    state: DeclarationShellVisibilityState,
    marker_range: Option<SourceRange>,
    spelling: Option<String>,
}

impl DeclarationShellVisibility {
    const fn unspecified() -> Self {
        Self {
            state: DeclarationShellVisibilityState::Unspecified,
            marker_range: None,
            spelling: None,
        }
    }

    fn explicit(
        state: DeclarationShellVisibilityState,
        range: SourceRange,
        spelling: String,
    ) -> Self {
        Self {
            state,
            marker_range: Some(range),
            spelling: Some(spelling),
        }
    }

    fn recovered(range: SourceRange, spelling: Option<String>) -> Self {
        Self {
            state: DeclarationShellVisibilityState::Recovered,
            marker_range: Some(range),
            spelling,
        }
    }

    /// Returns the source-shaped visibility state.
    #[must_use]
    pub const fn state(&self) -> DeclarationShellVisibilityState {
        self.state
    }

    /// Returns the marker range when a marker was represented.
    #[must_use]
    pub const fn marker_range(&self) -> Option<SourceRange> {
        self.marker_range
    }

    /// Returns the represented marker spelling.
    #[must_use]
    pub fn spelling(&self) -> Option<&str> {
        self.spelling.as_deref()
    }
}

/// Source-shaped visibility state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum DeclarationShellVisibilityState {
    /// No explicit visibility marker was represented.
    Unspecified,
    /// Explicit `public` marker.
    Public,
    /// Explicit `private` marker.
    Private,
    /// Malformed or recovered visibility marker/wrapper.
    Recovered,
}

/// Source-shaped declaration shell.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeclarationShell {
    id: DeclarationShellId,
    ordinal: usize,
    kind: DeclarationShellKind,
    module: ModuleId,
    node_id: SurfaceNodeId,
    syntax_kind: SyntaxKind,
    range: SourceRange,
    parent: Option<DeclarationShellId>,
    visibility: DeclarationShellVisibility,
    recovered: bool,
}

impl DeclarationShell {
    /// Returns the shell id.
    #[must_use]
    pub const fn id(&self) -> DeclarationShellId {
        self.id
    }

    /// Returns the source-order ordinal.
    #[must_use]
    pub const fn ordinal(&self) -> usize {
        self.ordinal
    }

    /// Returns the declaration shell kind.
    #[must_use]
    pub const fn kind(&self) -> DeclarationShellKind {
        self.kind
    }

    /// Returns the current module identity.
    #[must_use]
    pub const fn module(&self) -> &ModuleId {
        &self.module
    }

    /// Returns the originating surface node id.
    #[must_use]
    pub const fn node_id(&self) -> SurfaceNodeId {
        self.node_id
    }

    /// Returns the originating syntax kind.
    #[must_use]
    pub const fn syntax_kind(&self) -> SyntaxKind {
        self.syntax_kind
    }

    /// Returns the source range.
    #[must_use]
    pub const fn range(&self) -> SourceRange {
        self.range
    }

    /// Returns the containing declaration shell, if any.
    #[must_use]
    pub const fn parent(&self) -> Option<DeclarationShellId> {
        self.parent
    }

    /// Returns source-shaped visibility metadata.
    #[must_use]
    pub const fn visibility(&self) -> &DeclarationShellVisibility {
        &self.visibility
    }

    /// Returns whether represented recovery was present in this shell subtree.
    #[must_use]
    pub const fn recovered(&self) -> bool {
        self.recovered
    }
}

/// Source-shaped exported module path projection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExportPathShell {
    spelling: String,
    range: SourceRange,
}

impl ExportPathShell {
    fn new(spelling: String, range: SourceRange) -> Self {
        Self { spelling, range }
    }

    /// Returns the source-shaped module path spelling.
    #[must_use]
    pub fn spelling(&self) -> &str {
        &self.spelling
    }

    /// Returns the represented path range.
    #[must_use]
    pub const fn range(&self) -> SourceRange {
        self.range
    }
}

/// Source-shaped export projection shell.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExportProjectionShell {
    id: ExportProjectionShellId,
    ordinal: usize,
    module: ModuleId,
    node_id: SurfaceNodeId,
    range: SourceRange,
    recovered: bool,
    paths: Vec<ExportPathShell>,
}

impl ExportProjectionShell {
    fn new(
        id: ExportProjectionShellId,
        ordinal: usize,
        module: ModuleId,
        view: SurfaceNodeView<'_>,
        recovered: bool,
        paths: Vec<ExportPathShell>,
    ) -> Self {
        Self {
            id,
            ordinal,
            module,
            node_id: view.id(),
            range: view.range(),
            recovered,
            paths,
        }
    }

    /// Returns the export projection shell id.
    #[must_use]
    pub const fn id(&self) -> ExportProjectionShellId {
        self.id
    }

    /// Returns the source-order ordinal.
    #[must_use]
    pub const fn ordinal(&self) -> usize {
        self.ordinal
    }

    /// Returns the current module identity.
    #[must_use]
    pub const fn module(&self) -> &ModuleId {
        &self.module
    }

    /// Returns the originating export item node id.
    #[must_use]
    pub const fn node_id(&self) -> SurfaceNodeId {
        self.node_id
    }

    /// Returns the export item source range.
    #[must_use]
    pub const fn range(&self) -> SourceRange {
        self.range
    }

    /// Returns whether represented recovery was present in this export subtree.
    #[must_use]
    pub const fn recovered(&self) -> bool {
        self.recovered
    }

    /// Returns source-shaped exported module paths.
    #[must_use]
    pub fn paths(&self) -> &[ExportPathShell] {
        &self.paths
    }
}

/// Collects source-shaped declaration shells from `SurfaceAst`.
#[derive(Debug, Clone, Copy)]
pub struct DeclarationShellCollector<'a> {
    ast: &'a SurfaceAst,
    module: &'a ModuleId,
}

impl<'a> DeclarationShellCollector<'a> {
    /// Creates a declaration shell collector.
    #[must_use]
    pub const fn new(ast: &'a SurfaceAst, module: &'a ModuleId) -> Self {
        Self { ast, module }
    }

    /// Collects represented declaration and export projection shells.
    #[must_use]
    pub fn collect(self) -> DeclarationShellSet {
        let mut state = CollectionState::new(self.module);
        if let Some(root) = self.ast.root_view() {
            state.walk(root, None, false);
        }
        state.finish()
    }
}

struct CollectionState<'a> {
    module: &'a ModuleId,
    declarations: Vec<DeclarationShell>,
    exports: Vec<ExportProjectionShell>,
    next_declaration_id: usize,
    next_export_id: usize,
    next_declaration_ordinal: usize,
    next_export_ordinal: usize,
}

impl<'a> CollectionState<'a> {
    fn new(module: &'a ModuleId) -> Self {
        Self {
            module,
            declarations: Vec::new(),
            exports: Vec::new(),
            next_declaration_id: 0,
            next_export_id: 0,
            next_declaration_ordinal: 0,
            next_export_ordinal: 0,
        }
    }

    fn finish(self) -> DeclarationShellSet {
        DeclarationShellSet::new(self.declarations, self.exports)
    }

    fn walk(
        &mut self,
        view: SurfaceNodeView<'_>,
        parent: Option<DeclarationShellId>,
        inherited_recovery: bool,
    ) {
        if view.as_visible_item().is_some() {
            self.walk_visible_item(view, parent, inherited_recovery);
            return;
        }

        if view.as_export_item().is_some() {
            self.push_export(view, inherited_recovery);
            return;
        }

        if let Some(kind) = declaration_shell_kind(view.kind()) {
            let recovered = inherited_recovery || contains_recovery(view);
            let id = self.push_declaration(
                view,
                kind,
                parent,
                DeclarationShellVisibility::unspecified(),
                recovered,
            );
            self.walk_children(view, Some(id), inherited_recovery);
            return;
        }

        let child_recovery = if transparent_declaration_wrapper(view.kind()) {
            inherited_recovery || contains_recovery(view)
        } else {
            inherited_recovery
        };
        self.walk_children(view, parent, child_recovery);
    }

    fn walk_visible_item(
        &mut self,
        view: SurfaceNodeView<'_>,
        parent: Option<DeclarationShellId>,
        inherited_recovery: bool,
    ) {
        let visibility = visibility_from_wrapper(view);
        let recovered = inherited_recovery || contains_recovery(view);
        if let Some(target) = visible_target(view)
            && let Some(kind) = declaration_shell_kind(target.kind())
        {
            let id = self.push_declaration(target, kind, parent, visibility, recovered);
            self.walk_children(target, Some(id), inherited_recovery);
            return;
        }

        let id = self.push_declaration(
            view,
            DeclarationShellKind::VisibilityWrapper,
            parent,
            visibility,
            true,
        );
        self.walk_children(view, Some(id), inherited_recovery);
    }

    fn walk_children(
        &mut self,
        view: SurfaceNodeView<'_>,
        parent: Option<DeclarationShellId>,
        inherited_recovery: bool,
    ) {
        for child in view.child_views() {
            self.walk(child, parent, inherited_recovery);
        }
    }

    fn push_declaration(
        &mut self,
        view: SurfaceNodeView<'_>,
        kind: DeclarationShellKind,
        parent: Option<DeclarationShellId>,
        visibility: DeclarationShellVisibility,
        recovered: bool,
    ) -> DeclarationShellId {
        let id = DeclarationShellId::new(self.next_declaration_id);
        self.next_declaration_id += 1;
        let ordinal = self.next_declaration_ordinal;
        self.next_declaration_ordinal += 1;
        self.declarations.push(DeclarationShell {
            id,
            ordinal,
            kind,
            module: self.module.clone(),
            node_id: view.id(),
            syntax_kind: view.syntax_kind(),
            range: view.range(),
            parent,
            visibility,
            recovered,
        });
        id
    }

    fn push_export(
        &mut self,
        view: SurfaceNodeView<'_>,
        inherited_recovery: bool,
    ) -> ExportProjectionShellId {
        let id = ExportProjectionShellId::new(self.next_export_id);
        self.next_export_id += 1;
        let ordinal = self.next_export_ordinal;
        self.next_export_ordinal += 1;
        self.exports.push(ExportProjectionShell::new(
            id,
            ordinal,
            self.module.clone(),
            view,
            inherited_recovery || contains_recovery(view),
            export_paths(view),
        ));
        id
    }
}

fn declaration_shell_kind(kind: &SurfaceNodeKind) -> Option<DeclarationShellKind> {
    match kind {
        SurfaceNodeKind::PlaceholderItem => Some(DeclarationShellKind::Placeholder),
        SurfaceNodeKind::ReserveItem => Some(DeclarationShellKind::Reserve),
        SurfaceNodeKind::TheoremItem => Some(DeclarationShellKind::Theorem),
        SurfaceNodeKind::LemmaItem => Some(DeclarationShellKind::Lemma),
        SurfaceNodeKind::DefinitionBlockItem => Some(DeclarationShellKind::DefinitionBlock),
        SurfaceNodeKind::RegistrationBlockItem => Some(DeclarationShellKind::RegistrationBlock),
        SurfaceNodeKind::ClaimBlockItem => Some(DeclarationShellKind::ClaimBlock),
        SurfaceNodeKind::AttributeDefinition => Some(DeclarationShellKind::AttributeDefinition),
        SurfaceNodeKind::PredicateDefinition => Some(DeclarationShellKind::PredicateDefinition),
        SurfaceNodeKind::FunctorDefinition => Some(DeclarationShellKind::FunctorDefinition),
        SurfaceNodeKind::ModeDefinition => Some(DeclarationShellKind::ModeDefinition),
        SurfaceNodeKind::StructureDefinition => Some(DeclarationShellKind::StructureDefinition),
        SurfaceNodeKind::AlgorithmDefinition => Some(DeclarationShellKind::AlgorithmDefinition),
        SurfaceNodeKind::AttributeRedefinition => Some(DeclarationShellKind::AttributeRedefinition),
        SurfaceNodeKind::PredicateRedefinition => Some(DeclarationShellKind::PredicateRedefinition),
        SurfaceNodeKind::FunctorRedefinition => Some(DeclarationShellKind::FunctorRedefinition),
        SurfaceNodeKind::NotationAlias => Some(DeclarationShellKind::NotationAlias),
        SurfaceNodeKind::PropertyClause => Some(DeclarationShellKind::PropertyClause),
        SurfaceNodeKind::StructureField => Some(DeclarationShellKind::StructureField),
        SurfaceNodeKind::StructureProperty => Some(DeclarationShellKind::StructureProperty),
        SurfaceNodeKind::InheritanceDefinition => Some(DeclarationShellKind::InheritanceDefinition),
        SurfaceNodeKind::FieldRedefinition => Some(DeclarationShellKind::FieldRedefinition),
        SurfaceNodeKind::PropertyRedefinition => Some(DeclarationShellKind::PropertyRedefinition),
        SurfaceNodeKind::ExistentialRegistration => {
            Some(DeclarationShellKind::ExistentialRegistration)
        }
        SurfaceNodeKind::ConditionalRegistration => {
            Some(DeclarationShellKind::ConditionalRegistration)
        }
        SurfaceNodeKind::FunctorialRegistration => {
            Some(DeclarationShellKind::FunctorialRegistration)
        }
        SurfaceNodeKind::ReductionRegistration => Some(DeclarationShellKind::ReductionRegistration),
        _ => None,
    }
}

fn visibility_from_wrapper(view: SurfaceNodeView<'_>) -> DeclarationShellVisibility {
    let Some(marker) = view
        .child_views()
        .find(|child| child.as_visibility_marker().is_some())
    else {
        return DeclarationShellVisibility::recovered(view.range(), None);
    };
    let spelling = marker
        .child_views()
        .find_map(|child| child.as_token().map(|token| token.text.as_ref().to_owned()));
    match spelling.as_deref() {
        Some("public") => DeclarationShellVisibility::explicit(
            DeclarationShellVisibilityState::Public,
            marker.range(),
            "public".to_owned(),
        ),
        Some("private") => DeclarationShellVisibility::explicit(
            DeclarationShellVisibilityState::Private,
            marker.range(),
            "private".to_owned(),
        ),
        _ => DeclarationShellVisibility::recovered(marker.range(), spelling),
    }
}

fn visible_target(view: SurfaceNodeView<'_>) -> Option<SurfaceNodeView<'_>> {
    view.child_views().find(|child| {
        child.as_visibility_marker().is_none()
            && !matches!(
                child.kind(),
                SurfaceNodeKind::Annotation
                    | SurfaceNodeKind::LibraryAnnotation
                    | SurfaceNodeKind::StandaloneDiagnosticAnnotation
            )
            && declaration_shell_kind(child.kind()).is_some()
    })
}

fn transparent_declaration_wrapper(kind: &SurfaceNodeKind) -> bool {
    matches!(
        kind,
        SurfaceNodeKind::AnnotatedDefinitionContent
            | SurfaceNodeKind::AnnotatedRegistrationContent
            | SurfaceNodeKind::AnnotatedStatement
            | SurfaceNodeKind::AnnotatedAlgorithmStatement
    )
}

fn export_paths(view: SurfaceNodeView<'_>) -> Vec<ExportPathShell> {
    view.child_views()
        .filter(|child| child.as_module_path().is_some())
        .filter_map(|path| {
            module_path_spelling(path).map(|spelling| ExportPathShell::new(spelling, path.range()))
        })
        .collect()
}

fn module_path_spelling(view: SurfaceNodeView<'_>) -> Option<String> {
    let mut spelling = String::new();
    let mut saw_segment = false;
    for child in view.child_views() {
        if child.as_relative_prefix().is_some() {
            if let Some(text) = first_token_text(child) {
                spelling.push_str(&text);
            }
        } else if child.as_path_segment().is_some() {
            if saw_segment {
                spelling.push('.');
            }
            let segment = first_token_text(child)?;
            spelling.push_str(&segment);
            saw_segment = true;
        }
    }
    saw_segment.then_some(spelling)
}

fn first_token_text(view: SurfaceNodeView<'_>) -> Option<String> {
    view.child_views().find_map(|child| {
        child
            .as_token()
            .map(|token| token.text.as_ref().to_owned())
            .or_else(|| first_token_text(child))
    })
}

fn declaration_shell_cmp(left: &DeclarationShell, right: &DeclarationShell) -> Ordering {
    left.ordinal()
        .cmp(&right.ordinal())
        .then_with(|| range_key(left.range()).cmp(&range_key(right.range())))
        .then_with(|| left.id().cmp(&right.id()))
}

fn export_projection_shell_cmp(
    left: &ExportProjectionShell,
    right: &ExportProjectionShell,
) -> Ordering {
    left.ordinal()
        .cmp(&right.ordinal())
        .then_with(|| range_key(left.range()).cmp(&range_key(right.range())))
        .then_with(|| left.id().cmp(&right.id()))
}

const fn range_key(range: SourceRange) -> (usize, usize) {
    (range.start, range.end)
}

#[cfg(test)]
mod tests;
