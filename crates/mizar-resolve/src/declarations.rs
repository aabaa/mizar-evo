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
mod tests {
    use super::*;
    use crate::resolved_ast::ModuleId;
    use mizar_session::{
        BuildSnapshotId, Hash, InMemorySessionIdAllocator, ModulePath, PackageId,
        SessionIdAllocator, SourceId,
    };
    use mizar_syntax::{
        SurfaceAstBuilder, SurfaceBuilderNodeId, SurfaceTokenKind, SyntaxRecoveryKind,
    };

    #[test]
    fn collector_records_represented_declaration_kinds_in_source_order() {
        let source_id = source_id();
        let mut builder = SurfaceAstBuilder::new(source_id);
        let export = export_item(&mut builder, source_id, 0, &["pub", "math"]);
        let import = node(
            &mut builder,
            SurfaceNodeKind::ImportItem,
            source_id,
            10,
            11,
            vec![],
        );
        let reserve = node(
            &mut builder,
            SurfaceNodeKind::ReserveItem,
            source_id,
            12,
            13,
            vec![],
        );
        let theorem = visible_item(
            &mut builder,
            source_id,
            14,
            "public",
            SurfaceNodeKind::TheoremItem,
            21,
            30,
        );
        let lemma = visible_item(
            &mut builder,
            source_id,
            31,
            "private",
            SurfaceNodeKind::LemmaItem,
            39,
            45,
        );
        let definition_block = block_with_children(
            &mut builder,
            source_id,
            SurfaceNodeKind::DefinitionBlockItem,
            46,
            100,
            &[
                SurfaceNodeKind::AttributeDefinition,
                SurfaceNodeKind::PredicateDefinition,
                SurfaceNodeKind::FunctorDefinition,
                SurfaceNodeKind::ModeDefinition,
                SurfaceNodeKind::StructureDefinition,
                SurfaceNodeKind::AlgorithmDefinition,
                SurfaceNodeKind::AttributeRedefinition,
                SurfaceNodeKind::PredicateRedefinition,
                SurfaceNodeKind::FunctorRedefinition,
                SurfaceNodeKind::NotationAlias,
                SurfaceNodeKind::PropertyClause,
                SurfaceNodeKind::StructureField,
                SurfaceNodeKind::StructureProperty,
                SurfaceNodeKind::InheritanceDefinition,
                SurfaceNodeKind::FieldRedefinition,
                SurfaceNodeKind::PropertyRedefinition,
            ],
        );
        let registration_block = block_with_children(
            &mut builder,
            source_id,
            SurfaceNodeKind::RegistrationBlockItem,
            101,
            130,
            &[
                SurfaceNodeKind::ExistentialRegistration,
                SurfaceNodeKind::ConditionalRegistration,
                SurfaceNodeKind::FunctorialRegistration,
                SurfaceNodeKind::ReductionRegistration,
            ],
        );
        let claim = node(
            &mut builder,
            SurfaceNodeKind::ClaimBlockItem,
            source_id,
            131,
            140,
            vec![],
        );
        let placeholder = node(
            &mut builder,
            SurfaceNodeKind::PlaceholderItem,
            source_id,
            141,
            145,
            vec![],
        );
        let root = finish_module(
            &mut builder,
            source_id,
            vec![
                export,
                import,
                reserve,
                theorem,
                lemma,
                definition_block,
                registration_block,
                claim,
                placeholder,
            ],
        );
        let ast = builder.finish(Some(root), None);

        let shells = DeclarationShellCollector::new(&ast, &module_id()).collect();

        assert_eq!(
            declaration_kinds(&shells),
            vec![
                DeclarationShellKind::Reserve,
                DeclarationShellKind::Theorem,
                DeclarationShellKind::Lemma,
                DeclarationShellKind::DefinitionBlock,
                DeclarationShellKind::AttributeDefinition,
                DeclarationShellKind::PredicateDefinition,
                DeclarationShellKind::FunctorDefinition,
                DeclarationShellKind::ModeDefinition,
                DeclarationShellKind::StructureDefinition,
                DeclarationShellKind::AlgorithmDefinition,
                DeclarationShellKind::AttributeRedefinition,
                DeclarationShellKind::PredicateRedefinition,
                DeclarationShellKind::FunctorRedefinition,
                DeclarationShellKind::NotationAlias,
                DeclarationShellKind::PropertyClause,
                DeclarationShellKind::StructureField,
                DeclarationShellKind::StructureProperty,
                DeclarationShellKind::InheritanceDefinition,
                DeclarationShellKind::FieldRedefinition,
                DeclarationShellKind::PropertyRedefinition,
                DeclarationShellKind::RegistrationBlock,
                DeclarationShellKind::ExistentialRegistration,
                DeclarationShellKind::ConditionalRegistration,
                DeclarationShellKind::FunctorialRegistration,
                DeclarationShellKind::ReductionRegistration,
                DeclarationShellKind::ClaimBlock,
                DeclarationShellKind::Placeholder,
            ]
        );
        assert_eq!(shells.exports().len(), 1);
        assert_eq!(shells.exports()[0].paths()[0].spelling(), "pub.math");
        assert_eq!(
            shells.declarations()[1].visibility().state(),
            DeclarationShellVisibilityState::Public
        );
        assert_eq!(
            shells.declarations()[1].visibility().spelling(),
            Some("public")
        );
        assert_eq!(
            shells.declarations()[2].visibility().state(),
            DeclarationShellVisibilityState::Private
        );
        let definition_id = shells.declarations()[3].id();
        assert!(
            shells.declarations()[4..20]
                .iter()
                .all(|shell| shell.parent() == Some(definition_id))
        );
        let registration_id = shells.declarations()[20].id();
        assert!(
            shells.declarations()[21..25]
                .iter()
                .all(|shell| shell.parent() == Some(registration_id))
        );
    }

    #[test]
    fn recovered_subtrees_are_retained_and_marked_recovered() {
        let source_id = source_id();
        let mut builder = SurfaceAstBuilder::new(source_id);
        let recovery = builder.add_recovery(
            SyntaxRecoveryKind::MissingTerm,
            range(source_id, 10, 12),
            Vec::new(),
        );
        let recovered_predicate = node(
            &mut builder,
            SurfaceNodeKind::PredicateDefinition,
            source_id,
            8,
            20,
            vec![recovery],
        );
        let clean_functor = node(
            &mut builder,
            SurfaceNodeKind::FunctorDefinition,
            source_id,
            21,
            27,
            Vec::new(),
        );
        let definition = node(
            &mut builder,
            SurfaceNodeKind::DefinitionBlockItem,
            source_id,
            0,
            30,
            vec![recovered_predicate, clean_functor],
        );
        let dangling_visible = visible_item_without_target(&mut builder, source_id, 31, "public");
        let root = finish_module(&mut builder, source_id, vec![definition, dangling_visible]);
        let ast = builder.finish(Some(root), None);

        let shells = DeclarationShellCollector::new(&ast, &module_id()).collect();

        assert_eq!(
            declaration_kinds(&shells),
            vec![
                DeclarationShellKind::DefinitionBlock,
                DeclarationShellKind::PredicateDefinition,
                DeclarationShellKind::FunctorDefinition,
                DeclarationShellKind::VisibilityWrapper,
            ]
        );
        assert!(shells.declarations()[0].recovered());
        assert!(shells.declarations()[1].recovered());
        assert!(!shells.declarations()[2].recovered());
        assert!(shells.declarations()[3].recovered());
        assert_eq!(
            shells.declarations()[3].visibility().state(),
            DeclarationShellVisibilityState::Public
        );
    }

    #[test]
    fn annotation_wrappers_are_transparent_for_shell_collection() {
        let source_id = source_id();
        let mut builder = SurfaceAstBuilder::new(source_id);
        let annotation = node(
            &mut builder,
            SurfaceNodeKind::LibraryAnnotation,
            source_id,
            1,
            4,
            Vec::new(),
        );
        let predicate = node(
            &mut builder,
            SurfaceNodeKind::PredicateDefinition,
            source_id,
            6,
            12,
            Vec::new(),
        );
        let recovery = builder.add_recovery(
            SyntaxRecoveryKind::SkippedToken,
            range(source_id, 13, 14),
            Vec::new(),
        );
        let annotated_definition = node(
            &mut builder,
            SurfaceNodeKind::AnnotatedDefinitionContent,
            source_id,
            1,
            14,
            vec![annotation, predicate, recovery],
        );
        let definition = node(
            &mut builder,
            SurfaceNodeKind::DefinitionBlockItem,
            source_id,
            0,
            20,
            vec![annotated_definition],
        );
        let registration = node(
            &mut builder,
            SurfaceNodeKind::ConditionalRegistration,
            source_id,
            31,
            38,
            Vec::new(),
        );
        let annotated_registration = node(
            &mut builder,
            SurfaceNodeKind::AnnotatedRegistrationContent,
            source_id,
            30,
            39,
            vec![registration],
        );
        let registration_block = node(
            &mut builder,
            SurfaceNodeKind::RegistrationBlockItem,
            source_id,
            29,
            45,
            vec![annotated_registration],
        );
        let inline_functor = node(
            &mut builder,
            SurfaceNodeKind::InlineFunctorDefinition,
            source_id,
            51,
            55,
            Vec::new(),
        );
        let annotated_statement = node(
            &mut builder,
            SurfaceNodeKind::AnnotatedStatement,
            source_id,
            50,
            56,
            vec![inline_functor],
        );
        let variable = node(
            &mut builder,
            SurfaceNodeKind::VariableDeclaration,
            source_id,
            57,
            60,
            Vec::new(),
        );
        let annotated_algorithm_statement = node(
            &mut builder,
            SurfaceNodeKind::AnnotatedAlgorithmStatement,
            source_id,
            56,
            61,
            vec![variable],
        );
        let standalone_annotation = node(
            &mut builder,
            SurfaceNodeKind::StandaloneDiagnosticAnnotation,
            source_id,
            62,
            66,
            Vec::new(),
        );
        let root = finish_module(
            &mut builder,
            source_id,
            vec![
                definition,
                registration_block,
                annotated_statement,
                annotated_algorithm_statement,
                standalone_annotation,
            ],
        );
        let ast = builder.finish(Some(root), None);

        let shells = DeclarationShellCollector::new(&ast, &module_id()).collect();

        assert_eq!(
            declaration_kinds(&shells),
            vec![
                DeclarationShellKind::DefinitionBlock,
                DeclarationShellKind::PredicateDefinition,
                DeclarationShellKind::RegistrationBlock,
                DeclarationShellKind::ConditionalRegistration,
            ]
        );
        assert_eq!(
            shells.declarations()[1].parent(),
            Some(shells.declarations()[0].id())
        );
        assert!(shells.declarations()[1].recovered());
        assert_eq!(
            shells.declarations()[3].parent(),
            Some(shells.declarations()[2].id())
        );
        assert!(!shells.declarations()[3].recovered());
    }

    #[test]
    fn excluded_context_body_statement_and_recovery_nodes_do_not_create_shells() {
        let source_id = source_id();
        let mut builder = SurfaceAstBuilder::new(source_id);
        let module_path = module_path(&mut builder, source_id, 1, &["pkg", "names"]);
        let parameter = node(
            &mut builder,
            SurfaceNodeKind::DefinitionParameter,
            source_id,
            12,
            15,
            Vec::new(),
        );
        let correctness = node(
            &mut builder,
            SurfaceNodeKind::CorrectnessCondition,
            source_id,
            16,
            21,
            Vec::new(),
        );
        let proof = node(
            &mut builder,
            SurfaceNodeKind::ProofBlock,
            source_id,
            22,
            28,
            Vec::new(),
        );
        let pattern = node(
            &mut builder,
            SurfaceNodeKind::PredicatePattern,
            source_id,
            29,
            33,
            Vec::new(),
        );
        let inline_functor = node(
            &mut builder,
            SurfaceNodeKind::InlineFunctorDefinition,
            source_id,
            34,
            42,
            Vec::new(),
        );
        let inline_predicate = node(
            &mut builder,
            SurfaceNodeKind::InlinePredicateDefinition,
            source_id,
            43,
            51,
            Vec::new(),
        );
        let raw_recovery = builder.add_recovery(
            SyntaxRecoveryKind::SkippedToken,
            range(source_id, 52, 53),
            Vec::new(),
        );
        let definition = node(
            &mut builder,
            SurfaceNodeKind::DefinitionBlockItem,
            source_id,
            10,
            54,
            vec![
                parameter,
                correctness,
                proof,
                pattern,
                inline_functor,
                inline_predicate,
                raw_recovery,
            ],
        );
        let variable = node(
            &mut builder,
            SurfaceNodeKind::VariableDeclaration,
            source_id,
            61,
            65,
            Vec::new(),
        );
        let algorithm_body = node(
            &mut builder,
            SurfaceNodeKind::AlgorithmBody,
            source_id,
            60,
            66,
            vec![variable],
        );
        let algorithm = node(
            &mut builder,
            SurfaceNodeKind::AlgorithmDefinition,
            source_id,
            56,
            67,
            vec![algorithm_body],
        );
        let root = finish_module(
            &mut builder,
            source_id,
            vec![module_path, definition, algorithm],
        );
        let ast = builder.finish(Some(root), None);

        let shells = DeclarationShellCollector::new(&ast, &module_id()).collect();

        assert_eq!(
            declaration_kinds(&shells),
            vec![
                DeclarationShellKind::DefinitionBlock,
                DeclarationShellKind::AlgorithmDefinition,
            ]
        );
        assert!(shells.declarations()[0].recovered());
        assert!(!shells.declarations()[1].recovered());
    }

    #[test]
    fn malformed_export_projection_is_retained_without_target_validation() {
        let source_id = source_id();
        let mut builder = SurfaceAstBuilder::new(source_id);
        let good_export = export_item(&mut builder, source_id, 0, &["pkg", "core"]);
        let recovery = builder.add_recovery(
            SyntaxRecoveryKind::SkippedToken,
            range(source_id, 20, 25),
            Vec::new(),
        );
        let bad_export = node(
            &mut builder,
            SurfaceNodeKind::ExportItem,
            source_id,
            18,
            26,
            vec![recovery],
        );
        let root = finish_module(&mut builder, source_id, vec![good_export, bad_export]);
        let ast = builder.finish(Some(root), None);

        let shells = DeclarationShellCollector::new(&ast, &module_id()).collect();

        assert!(shells.declarations().is_empty());
        assert_eq!(shells.exports().len(), 2);
        assert_eq!(shells.exports()[0].paths()[0].spelling(), "pkg.core");
        assert!(!shells.exports()[0].recovered());
        assert!(shells.exports()[1].paths().is_empty());
        assert!(shells.exports()[1].recovered());
    }

    fn declaration_kinds(shells: &DeclarationShellSet) -> Vec<DeclarationShellKind> {
        shells
            .declarations()
            .iter()
            .map(DeclarationShell::kind)
            .collect()
    }

    fn block_with_children(
        builder: &mut SurfaceAstBuilder,
        source_id: SourceId,
        kind: SurfaceNodeKind,
        start: usize,
        end: usize,
        child_kinds: &[SurfaceNodeKind],
    ) -> SurfaceBuilderNodeId {
        let mut children = Vec::new();
        for (index, child_kind) in child_kinds.iter().enumerate() {
            let child_start = start + 1 + index * 2;
            children.push(node(
                builder,
                child_kind.clone(),
                source_id,
                child_start,
                child_start + 1,
                Vec::new(),
            ));
        }
        node(builder, kind, source_id, start, end, children)
    }

    fn visible_item(
        builder: &mut SurfaceAstBuilder,
        source_id: SourceId,
        start: usize,
        spelling: &str,
        target_kind: SurfaceNodeKind,
        target_start: usize,
        target_end: usize,
    ) -> SurfaceBuilderNodeId {
        let marker = visibility_marker(builder, source_id, start, spelling);
        let target = node(
            builder,
            target_kind,
            source_id,
            target_start,
            target_end,
            Vec::new(),
        );
        node(
            builder,
            SurfaceNodeKind::VisibleItem,
            source_id,
            start,
            target_end,
            vec![marker, target],
        )
    }

    fn visible_item_without_target(
        builder: &mut SurfaceAstBuilder,
        source_id: SourceId,
        start: usize,
        spelling: &str,
    ) -> SurfaceBuilderNodeId {
        let marker = visibility_marker(builder, source_id, start, spelling);
        node(
            builder,
            SurfaceNodeKind::VisibleItem,
            source_id,
            start,
            start + spelling.len(),
            vec![marker],
        )
    }

    fn visibility_marker(
        builder: &mut SurfaceAstBuilder,
        source_id: SourceId,
        start: usize,
        spelling: &str,
    ) -> SurfaceBuilderNodeId {
        let token = builder.add_token(
            SurfaceTokenKind::ReservedWord,
            spelling,
            range(source_id, start, start + spelling.len()),
        );
        node(
            builder,
            SurfaceNodeKind::VisibilityMarker,
            source_id,
            start,
            start + spelling.len(),
            vec![token],
        )
    }

    fn export_item(
        builder: &mut SurfaceAstBuilder,
        source_id: SourceId,
        start: usize,
        components: &[&str],
    ) -> SurfaceBuilderNodeId {
        let path = module_path(builder, source_id, start + 1, components);
        node(
            builder,
            SurfaceNodeKind::ExportItem,
            source_id,
            start,
            start + 1 + components.join(".").len(),
            vec![path],
        )
    }

    fn module_path(
        builder: &mut SurfaceAstBuilder,
        source_id: SourceId,
        start: usize,
        components: &[&str],
    ) -> SurfaceBuilderNodeId {
        let mut children = Vec::new();
        let mut cursor = start;
        for component in components {
            let token = builder.add_token(
                SurfaceTokenKind::Identifier,
                *component,
                range(source_id, cursor, cursor + component.len()),
            );
            children.push(node(
                builder,
                SurfaceNodeKind::PathSegment,
                source_id,
                cursor,
                cursor + component.len(),
                vec![token],
            ));
            cursor += component.len() + 1;
        }
        node(
            builder,
            SurfaceNodeKind::ModulePath,
            source_id,
            start,
            cursor.saturating_sub(1),
            children,
        )
    }

    fn finish_module(
        builder: &mut SurfaceAstBuilder,
        source_id: SourceId,
        items: Vec<SurfaceBuilderNodeId>,
    ) -> SurfaceBuilderNodeId {
        let item_list = node(builder, SurfaceNodeKind::ItemList, source_id, 0, 200, items);
        let unit = node(
            builder,
            SurfaceNodeKind::CompilationUnit,
            source_id,
            0,
            200,
            vec![item_list],
        );
        node(
            builder,
            SurfaceNodeKind::Root,
            source_id,
            0,
            200,
            vec![unit],
        )
    }

    fn node(
        builder: &mut SurfaceAstBuilder,
        kind: SurfaceNodeKind,
        source_id: SourceId,
        start: usize,
        end: usize,
        children: Vec<SurfaceBuilderNodeId>,
    ) -> SurfaceBuilderNodeId {
        builder.add_node(kind, range(source_id, start, end), children)
    }

    fn module_id() -> ModuleId {
        ModuleId::new(PackageId::new("app"), ModulePath::new("main"))
    }

    fn source_id() -> SourceId {
        let snapshot_id = BuildSnapshotId::from_published_schema_str(&format!(
            "mizar-session-build-snapshot-v1:{}",
            "04".repeat(Hash::BYTE_LEN)
        ))
        .unwrap();
        InMemorySessionIdAllocator::new()
            .next_source_id(snapshot_id)
            .unwrap()
    }

    const fn range(source_id: SourceId, start: usize, end: usize) -> SourceRange {
        SourceRange {
            source_id,
            start,
            end,
        }
    }
}
