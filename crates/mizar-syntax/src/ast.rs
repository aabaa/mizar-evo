use crate::recovery::SyntaxRecoveryKind;
use crate::trivia::{
    SurfaceTrivia, TriviaAttachmentTarget, TriviaNodeTarget, write_trivia_snapshot,
};
use mizar_session::{SourceId, SourceRange};
use std::fmt::Write as _;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_BUILDER_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MizarLanguage {}

impl rowan::Language for MizarLanguage {
    type Kind = SyntaxKind;

    fn kind_from_raw(raw: rowan::SyntaxKind) -> Self::Kind {
        SyntaxKind::from_raw(raw.0)
    }

    fn kind_to_raw(kind: Self::Kind) -> rowan::SyntaxKind {
        rowan::SyntaxKind(kind as u16)
    }
}

pub type RowanSyntaxNode = rowan::SyntaxNode<MizarLanguage>;
pub type RowanSyntaxToken = rowan::SyntaxToken<MizarLanguage>;
pub type RowanSyntaxElement = rowan::SyntaxElement<MizarLanguage>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u16)]
#[non_exhaustive]
pub enum SyntaxKind {
    Unknown = 0,
    Root = 1,
    Token = 2,
    InfixExpression = 3,
    ErrorRecovery = 4,
    ModulePath = 5,
    NamespacePath = 6,
    QualifiedSymbol = 7,
    PathSegment = 8,
    RelativePrefix = 9,
    CompilationUnit = 10,
    ItemList = 11,
    PlaceholderItem = 12,
    ImportItem = 13,
    ImportAliasDecl = 14,
    ModuleBranchImport = 15,
    ExportItem = 16,
    VisibilityMarker = 17,
    VisibleItem = 18,
    ReserveItem = 19,
    ReserveSegment = 20,
    TypeExpression = 21,
    AttributeChain = 22,
    AttributeRef = 23,
    ParameterPrefix = 24,
    TypeHead = 25,
    TypeArguments = 26,
    TermPlaceholder = 27,
    TermExpression = 28,
    TermReference = 29,
    NumeralTerm = 30,
    ItTerm = 31,
    ParenthesizedTerm = 32,
    ChoiceTerm = 33,
    ApplicationTerm = 34,
    StructureConstructor = 35,
    FieldArgument = 36,
    SetEnumeration = 37,
    TokenIdentifier = 100,
    TokenReservedWord = 101,
    TokenReservedSymbol = 102,
    TokenNumeral = 103,
    TokenLexemeRun = 104,
    TokenUserSymbol = 105,
    TokenStringLiteral = 106,
    TokenErrorRecovery = 107,
    TokenUnknown = 108,
}

impl SyntaxKind {
    pub const fn from_raw(raw: u16) -> Self {
        match raw {
            1 => Self::Root,
            2 => Self::Token,
            3 => Self::InfixExpression,
            4 => Self::ErrorRecovery,
            5 => Self::ModulePath,
            6 => Self::NamespacePath,
            7 => Self::QualifiedSymbol,
            8 => Self::PathSegment,
            9 => Self::RelativePrefix,
            10 => Self::CompilationUnit,
            11 => Self::ItemList,
            12 => Self::PlaceholderItem,
            13 => Self::ImportItem,
            14 => Self::ImportAliasDecl,
            15 => Self::ModuleBranchImport,
            16 => Self::ExportItem,
            17 => Self::VisibilityMarker,
            18 => Self::VisibleItem,
            19 => Self::ReserveItem,
            20 => Self::ReserveSegment,
            21 => Self::TypeExpression,
            22 => Self::AttributeChain,
            23 => Self::AttributeRef,
            24 => Self::ParameterPrefix,
            25 => Self::TypeHead,
            26 => Self::TypeArguments,
            27 => Self::TermPlaceholder,
            28 => Self::TermExpression,
            29 => Self::TermReference,
            30 => Self::NumeralTerm,
            31 => Self::ItTerm,
            32 => Self::ParenthesizedTerm,
            33 => Self::ChoiceTerm,
            34 => Self::ApplicationTerm,
            35 => Self::StructureConstructor,
            36 => Self::FieldArgument,
            37 => Self::SetEnumeration,
            100 => Self::TokenIdentifier,
            101 => Self::TokenReservedWord,
            102 => Self::TokenReservedSymbol,
            103 => Self::TokenNumeral,
            104 => Self::TokenLexemeRun,
            105 => Self::TokenUserSymbol,
            106 => Self::TokenStringLiteral,
            107 => Self::TokenErrorRecovery,
            108 => Self::TokenUnknown,
            _ => Self::Unknown,
        }
    }

    pub const fn is_node_kind(self) -> bool {
        matches!(
            self,
            Self::Root
                | Self::Token
                | Self::InfixExpression
                | Self::ErrorRecovery
                | Self::ModulePath
                | Self::NamespacePath
                | Self::QualifiedSymbol
                | Self::PathSegment
                | Self::RelativePrefix
                | Self::CompilationUnit
                | Self::ItemList
                | Self::PlaceholderItem
                | Self::ImportItem
                | Self::ImportAliasDecl
                | Self::ModuleBranchImport
                | Self::ExportItem
                | Self::VisibilityMarker
                | Self::VisibleItem
                | Self::ReserveItem
                | Self::ReserveSegment
                | Self::TypeExpression
                | Self::AttributeChain
                | Self::AttributeRef
                | Self::ParameterPrefix
                | Self::TypeHead
                | Self::TypeArguments
                | Self::TermPlaceholder
                | Self::TermExpression
                | Self::TermReference
                | Self::NumeralTerm
                | Self::ItTerm
                | Self::ParenthesizedTerm
                | Self::ChoiceTerm
                | Self::ApplicationTerm
                | Self::StructureConstructor
                | Self::FieldArgument
                | Self::SetEnumeration
        )
    }

    pub const fn is_token_kind(self) -> bool {
        matches!(
            self,
            Self::TokenIdentifier
                | Self::TokenReservedWord
                | Self::TokenReservedSymbol
                | Self::TokenNumeral
                | Self::TokenLexemeRun
                | Self::TokenUserSymbol
                | Self::TokenStringLiteral
                | Self::TokenErrorRecovery
                | Self::TokenUnknown
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SurfaceAst {
    pub source_id: SourceId,
    nodes: Vec<SurfaceNode>,
    root: Option<SurfaceNodeId>,
    token_nodes: Vec<SurfaceNodeId>,
    expression_root: Option<SurfaceNodeId>,
    green: rowan::GreenNode,
    trivia: SurfaceTrivia,
}

impl SurfaceAst {
    fn new(
        source_id: SourceId,
        nodes: Vec<SurfaceNode>,
        root: Option<SurfaceNodeId>,
        token_nodes: Vec<SurfaceNodeId>,
        expression_root: Option<SurfaceNodeId>,
    ) -> Self {
        let green = build_green_tree(&nodes, root);
        let trivia = SurfaceTrivia::empty(source_id);
        Self {
            source_id,
            nodes,
            root,
            token_nodes,
            expression_root,
            green,
            trivia,
        }
    }

    pub fn node(&self, id: SurfaceNodeId) -> Option<&SurfaceNode> {
        self.nodes.get(id.index())
    }

    pub fn nodes(&self) -> &[SurfaceNode] {
        &self.nodes
    }

    pub const fn root(&self) -> Option<SurfaceNodeId> {
        self.root
    }

    pub fn token_nodes(&self) -> &[SurfaceNodeId] {
        &self.token_nodes
    }

    pub const fn expression_root(&self) -> Option<SurfaceNodeId> {
        self.expression_root
    }

    pub fn node_view(&self, id: SurfaceNodeId) -> Option<SurfaceNodeView<'_>> {
        self.node(id).map(|node| SurfaceNodeView {
            ast: self,
            id,
            node,
        })
    }

    pub fn root_view(&self) -> Option<SurfaceNodeView<'_>> {
        self.root.and_then(|root| self.node_view(root))
    }

    pub fn expression_view(&self) -> Option<SurfaceNodeView<'_>> {
        self.expression_root
            .and_then(|expression| self.node_view(expression))
    }

    pub fn token_views(&self) -> impl Iterator<Item = SurfaceNodeView<'_>> {
        self.token_nodes.iter().filter_map(|id| self.node_view(*id))
    }

    pub fn token_texts(&self) -> Vec<&str> {
        self.token_views()
            .filter_map(|node| node.as_token().map(|token| token.text.as_ref()))
            .collect()
    }

    pub fn green_node(&self) -> &rowan::GreenNode {
        &self.green
    }

    pub fn rowan_root(&self) -> RowanSyntaxNode {
        RowanSyntaxNode::new_root(self.green.clone())
    }

    pub fn trivia(&self) -> &SurfaceTrivia {
        &self.trivia
    }

    pub fn with_trivia(mut self, trivia: SurfaceTrivia) -> Self {
        assert_eq!(
            trivia.source_id(),
            self.source_id,
            "SurfaceAst trivia must belong to the AST source"
        );
        self.assert_trivia_targets(&trivia);
        self.trivia = trivia;
        self
    }

    fn assert_trivia_targets(&self, trivia: &SurfaceTrivia) {
        for attachment in trivia.doc_comment_attachments() {
            self.assert_trivia_target(&attachment.target);
        }
        for skipped in trivia.skipped_token_ranges() {
            if let Some(owner) = &skipped.owner {
                self.assert_trivia_target(owner);
            }
        }
    }

    fn assert_trivia_target(&self, target: &TriviaAttachmentTarget) {
        match target {
            TriviaAttachmentTarget::Node(target) => {
                let node = self.assert_existing_trivia_target(*target, "node");
                assert!(
                    !matches!(node.kind, SurfaceNodeKind::Token(_)),
                    "SurfaceAst trivia node target must not refer to a token node"
                );
            }
            TriviaAttachmentTarget::Token(target) => {
                let node = self.assert_existing_trivia_target(*target, "token");
                assert!(
                    matches!(node.kind, SurfaceNodeKind::Token(_)),
                    "SurfaceAst trivia token target must refer to a token node"
                );
            }
            TriviaAttachmentTarget::Detached(_) => {}
        }
    }

    fn assert_existing_trivia_target(&self, target: TriviaNodeTarget, role: &str) -> &SurfaceNode {
        let node = self
            .node(target.id)
            .unwrap_or_else(|| panic!("SurfaceAst trivia {role} target must exist in the AST"));
        assert_eq!(
            node.range, target.range,
            "SurfaceAst trivia {role} target range must match the AST node range"
        );
        node
    }

    pub fn snapshot_text(&self) -> String {
        let mut output = String::from("surface-ast-snapshot-v1\n");
        output.push_str("root:\n");
        match self.root_view() {
            Some(root) => write_snapshot_node(&mut output, root, 1),
            None => output.push_str("  <none>\n"),
        }
        output.push_str("expression_root:\n");
        match self.expression_view() {
            Some(expression) => write_snapshot_node(&mut output, expression, 1),
            None => output.push_str("  <none>\n"),
        }
        output.push_str("token_nodes:\n");
        let mut token_count = 0;
        for token in self.token_views() {
            token_count += 1;
            write_snapshot_node(&mut output, token, 1);
        }
        if token_count == 0 {
            output.push_str("  <none>\n");
        }
        output
    }

    pub fn snapshot_text_with_trivia(&self) -> String {
        let mut output = self.snapshot_text();
        write_trivia_snapshot(&mut output, &self.trivia, |id| {
            self.node(id).map(|node| node.range)
        });
        output
    }

    pub fn range_contains_child_ranges(&self, id: SurfaceNodeId) -> Option<bool> {
        let parent = self.node(id)?;
        Some(parent.children.iter().all(|child| {
            self.node(*child)
                .is_some_and(|child| contains_range(parent.range, child.range))
        }))
    }
}

pub struct SurfaceAstBuilder {
    source_id: SourceId,
    builder_id: u64,
    nodes: Vec<BuilderNode>,
    token_nodes: Vec<SurfaceBuilderNodeId>,
    recovery_nodes: Vec<SurfaceBuilderNodeId>,
}

impl SurfaceAstBuilder {
    pub fn new(source_id: SourceId) -> Self {
        Self {
            source_id,
            builder_id: NEXT_BUILDER_ID.fetch_add(1, Ordering::Relaxed),
            nodes: Vec::new(),
            token_nodes: Vec::new(),
            recovery_nodes: Vec::new(),
        }
    }

    pub fn add_node(
        &mut self,
        kind: SurfaceNodeKind,
        range: SourceRange,
        children: Vec<SurfaceBuilderNodeId>,
    ) -> SurfaceBuilderNodeId {
        assert!(
            !matches!(
                kind,
                SurfaceNodeKind::Token(_) | SurfaceNodeKind::ErrorRecovery(_)
            ),
            "SurfaceAstBuilder::add_node cannot create token or recovery side-table entries"
        );
        self.assert_existing_children(&children);
        self.push_node(BuilderNode::new(kind, range, children))
    }

    fn add_recovered_node(
        &mut self,
        kind: SurfaceNodeKind,
        range: SourceRange,
        children: Vec<SurfaceBuilderNodeId>,
    ) -> SurfaceBuilderNodeId {
        self.assert_existing_children(&children);
        self.push_node(BuilderNode::recovered(kind, range, children))
    }

    pub fn add_token(
        &mut self,
        kind: SurfaceTokenKind,
        text: impl Into<Arc<str>>,
        range: SourceRange,
    ) -> SurfaceBuilderNodeId {
        let id = self.push_node(BuilderNode::new(
            SurfaceNodeKind::Token(SurfaceToken::new(kind, text)),
            range,
            Vec::new(),
        ));
        self.token_nodes.push(id);
        id
    }

    pub fn add_recovered_token(
        &mut self,
        kind: SurfaceTokenKind,
        text: impl Into<Arc<str>>,
        range: SourceRange,
    ) -> SurfaceBuilderNodeId {
        let id = self.add_recovered_node(
            SurfaceNodeKind::Token(SurfaceToken::new(kind, text)),
            range,
            Vec::new(),
        );
        self.token_nodes.push(id);
        id
    }

    pub fn add_recovery(
        &mut self,
        recovery_kind: SyntaxRecoveryKind,
        range: SourceRange,
        children: Vec<SurfaceBuilderNodeId>,
    ) -> SurfaceBuilderNodeId {
        let id = self.add_recovered_node(
            SurfaceNodeKind::ErrorRecovery(recovery_kind),
            range,
            children,
        );
        self.recovery_nodes.push(id);
        id
    }

    pub fn node(&self, id: SurfaceBuilderNodeId) -> Option<&BuilderNode> {
        self.assert_same_builder(id);
        self.nodes.get(id.index())
    }

    pub fn node_kind(&self, id: SurfaceBuilderNodeId) -> Option<&SurfaceNodeKind> {
        self.node(id).map(|node| &node.kind)
    }

    pub fn node_range(&self, id: SurfaceBuilderNodeId) -> Option<SourceRange> {
        self.node(id).map(|node| node.range)
    }

    pub fn token_node_ids(&self) -> &[SurfaceBuilderNodeId] {
        &self.token_nodes
    }

    pub fn recovery_node_ids(&self) -> &[SurfaceBuilderNodeId] {
        &self.recovery_nodes
    }

    pub fn finish(
        self,
        root: Option<SurfaceBuilderNodeId>,
        expression_root: Option<SurfaceBuilderNodeId>,
    ) -> SurfaceAst {
        self.assert_existing_optional_id(root, "root");
        self.assert_existing_optional_id(expression_root, "expression root");
        self.assert_tree_shaped_except_root_listing(root);
        let nodes = self
            .nodes
            .into_iter()
            .map(BuilderNode::into_surface_node)
            .collect();
        SurfaceAst::new(
            self.source_id,
            nodes,
            root.map(SurfaceBuilderNodeId::into_surface_node_id),
            self.token_nodes
                .into_iter()
                .map(SurfaceBuilderNodeId::into_surface_node_id)
                .collect(),
            expression_root.map(SurfaceBuilderNodeId::into_surface_node_id),
        )
    }

    fn push_node(&mut self, node: BuilderNode) -> SurfaceBuilderNodeId {
        let id = SurfaceBuilderNodeId::new(self.nodes.len(), self.builder_id);
        self.nodes.push(node);
        id
    }

    fn assert_existing_children(&self, children: &[SurfaceBuilderNodeId]) {
        for child in children {
            self.assert_same_builder(*child);
            assert!(
                child.index() < self.nodes.len(),
                "SurfaceAstBuilder child id {child:?} must refer to an existing node in this builder"
            );
        }
    }

    fn assert_existing_optional_id(&self, id: Option<SurfaceBuilderNodeId>, role: &str) {
        if let Some(id) = id {
            self.assert_same_builder(id);
            assert!(
                id.index() < self.nodes.len(),
                "SurfaceAstBuilder {role} id {id:?} must refer to an existing node in this builder"
            );
        }
    }

    fn assert_same_builder(&self, id: SurfaceBuilderNodeId) {
        assert!(
            id.builder_id == self.builder_id,
            "SurfaceAstBuilder node id {id:?} must have been created by this builder"
        );
    }

    fn assert_tree_shaped_except_root_listing(&self, root: Option<SurfaceBuilderNodeId>) {
        let mut non_root_parent_counts = vec![0_u8; self.nodes.len()];
        for (parent_index, node) in self.nodes.iter().enumerate() {
            if Some(SurfaceBuilderNodeId::new(parent_index, self.builder_id)) == root {
                continue;
            }
            for child in &node.children {
                non_root_parent_counts[child.index()] =
                    non_root_parent_counts[child.index()].saturating_add(1);
            }
        }
        for (index, count) in non_root_parent_counts.iter().copied().enumerate() {
            assert!(
                count <= 1,
                "SurfaceAstBuilder node id {:?} cannot be shared by multiple non-root parents",
                SurfaceBuilderNodeId::new(index, self.builder_id)
            );
        }
        if let Some(root) = root {
            let root_node = &self.nodes[root.index()];
            for child in &root_node.children {
                if self.nodes[child.index()].kind.is_structural()
                    && non_root_parent_counts[child.index()] > 0
                {
                    panic!(
                        "SurfaceAstBuilder structural root child {child:?} cannot also have a non-root parent"
                    );
                }
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuilderNode {
    pub kind: SurfaceNodeKind,
    pub range: SourceRange,
    pub children: Vec<SurfaceBuilderNodeId>,
    pub recovered: bool,
}

impl BuilderNode {
    fn new(kind: SurfaceNodeKind, range: SourceRange, children: Vec<SurfaceBuilderNodeId>) -> Self {
        Self {
            kind,
            range,
            children,
            recovered: false,
        }
    }

    fn recovered(
        kind: SurfaceNodeKind,
        range: SourceRange,
        children: Vec<SurfaceBuilderNodeId>,
    ) -> Self {
        Self {
            kind,
            range,
            children,
            recovered: true,
        }
    }

    fn into_surface_node(self) -> SurfaceNode {
        SurfaceNode {
            kind: self.kind,
            range: self.range,
            children: self
                .children
                .into_iter()
                .map(SurfaceBuilderNodeId::into_surface_node_id)
                .collect(),
            recovered: self.recovered,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SurfaceBuilderNodeId {
    index: usize,
    builder_id: u64,
}

impl SurfaceBuilderNodeId {
    const fn new(index: usize, builder_id: u64) -> Self {
        Self { index, builder_id }
    }

    const fn index(self) -> usize {
        self.index
    }

    const fn into_surface_node_id(self) -> SurfaceNodeId {
        SurfaceNodeId::new(self.index)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct SurfaceNodeView<'a> {
    ast: &'a SurfaceAst,
    id: SurfaceNodeId,
    node: &'a SurfaceNode,
}

impl<'a> SurfaceNodeView<'a> {
    pub const fn id(self) -> SurfaceNodeId {
        self.id
    }

    pub const fn kind(self) -> &'a SurfaceNodeKind {
        &self.node.kind
    }

    pub const fn syntax_kind(self) -> SyntaxKind {
        self.node.kind.syntax_kind()
    }

    pub const fn range(self) -> SourceRange {
        self.node.range
    }

    pub fn children(self) -> &'a [SurfaceNodeId] {
        &self.node.children
    }

    pub const fn is_recovered(self) -> bool {
        self.node.recovered
    }

    pub const fn as_token(self) -> Option<&'a SurfaceToken> {
        match &self.node.kind {
            SurfaceNodeKind::Token(token) => Some(token),
            SurfaceNodeKind::Root
            | SurfaceNodeKind::CompilationUnit
            | SurfaceNodeKind::ItemList
            | SurfaceNodeKind::PlaceholderItem
            | SurfaceNodeKind::ImportItem
            | SurfaceNodeKind::ImportAliasDecl
            | SurfaceNodeKind::ModuleBranchImport
            | SurfaceNodeKind::ExportItem
            | SurfaceNodeKind::VisibilityMarker
            | SurfaceNodeKind::VisibleItem
            | SurfaceNodeKind::ReserveItem
            | SurfaceNodeKind::ReserveSegment
            | SurfaceNodeKind::TypeExpression
            | SurfaceNodeKind::AttributeChain
            | SurfaceNodeKind::AttributeRef
            | SurfaceNodeKind::ParameterPrefix
            | SurfaceNodeKind::TypeHead
            | SurfaceNodeKind::TypeArguments
            | SurfaceNodeKind::TermPlaceholder
            | SurfaceNodeKind::TermExpression
            | SurfaceNodeKind::TermReference
            | SurfaceNodeKind::NumeralTerm
            | SurfaceNodeKind::ItTerm
            | SurfaceNodeKind::ParenthesizedTerm
            | SurfaceNodeKind::ChoiceTerm
            | SurfaceNodeKind::ApplicationTerm
            | SurfaceNodeKind::StructureConstructor
            | SurfaceNodeKind::FieldArgument
            | SurfaceNodeKind::SetEnumeration
            | SurfaceNodeKind::ModulePath
            | SurfaceNodeKind::NamespacePath
            | SurfaceNodeKind::QualifiedSymbol
            | SurfaceNodeKind::PathSegment
            | SurfaceNodeKind::RelativePrefix
            | SurfaceNodeKind::InfixExpression(_)
            | SurfaceNodeKind::ErrorRecovery(_) => None,
        }
    }

    pub const fn as_infix_expression(self) -> Option<&'a SurfaceInfixOperator> {
        match &self.node.kind {
            SurfaceNodeKind::InfixExpression(operator) => Some(operator),
            SurfaceNodeKind::Root
            | SurfaceNodeKind::CompilationUnit
            | SurfaceNodeKind::ItemList
            | SurfaceNodeKind::PlaceholderItem
            | SurfaceNodeKind::ImportItem
            | SurfaceNodeKind::ImportAliasDecl
            | SurfaceNodeKind::ModuleBranchImport
            | SurfaceNodeKind::ExportItem
            | SurfaceNodeKind::VisibilityMarker
            | SurfaceNodeKind::VisibleItem
            | SurfaceNodeKind::ReserveItem
            | SurfaceNodeKind::ReserveSegment
            | SurfaceNodeKind::TypeExpression
            | SurfaceNodeKind::AttributeChain
            | SurfaceNodeKind::AttributeRef
            | SurfaceNodeKind::ParameterPrefix
            | SurfaceNodeKind::TypeHead
            | SurfaceNodeKind::TypeArguments
            | SurfaceNodeKind::TermPlaceholder
            | SurfaceNodeKind::TermExpression
            | SurfaceNodeKind::TermReference
            | SurfaceNodeKind::NumeralTerm
            | SurfaceNodeKind::ItTerm
            | SurfaceNodeKind::ParenthesizedTerm
            | SurfaceNodeKind::ChoiceTerm
            | SurfaceNodeKind::ApplicationTerm
            | SurfaceNodeKind::StructureConstructor
            | SurfaceNodeKind::FieldArgument
            | SurfaceNodeKind::SetEnumeration
            | SurfaceNodeKind::ModulePath
            | SurfaceNodeKind::NamespacePath
            | SurfaceNodeKind::QualifiedSymbol
            | SurfaceNodeKind::PathSegment
            | SurfaceNodeKind::RelativePrefix
            | SurfaceNodeKind::Token(_)
            | SurfaceNodeKind::ErrorRecovery(_) => None,
        }
    }

    pub const fn as_recovery(self) -> Option<SyntaxRecoveryKind> {
        match self.node.kind {
            SurfaceNodeKind::ErrorRecovery(kind) => Some(kind),
            SurfaceNodeKind::Root
            | SurfaceNodeKind::CompilationUnit
            | SurfaceNodeKind::ItemList
            | SurfaceNodeKind::PlaceholderItem
            | SurfaceNodeKind::ImportItem
            | SurfaceNodeKind::ImportAliasDecl
            | SurfaceNodeKind::ModuleBranchImport
            | SurfaceNodeKind::ExportItem
            | SurfaceNodeKind::VisibilityMarker
            | SurfaceNodeKind::VisibleItem
            | SurfaceNodeKind::ReserveItem
            | SurfaceNodeKind::ReserveSegment
            | SurfaceNodeKind::TypeExpression
            | SurfaceNodeKind::AttributeChain
            | SurfaceNodeKind::AttributeRef
            | SurfaceNodeKind::ParameterPrefix
            | SurfaceNodeKind::TypeHead
            | SurfaceNodeKind::TypeArguments
            | SurfaceNodeKind::TermPlaceholder
            | SurfaceNodeKind::TermExpression
            | SurfaceNodeKind::TermReference
            | SurfaceNodeKind::NumeralTerm
            | SurfaceNodeKind::ItTerm
            | SurfaceNodeKind::ParenthesizedTerm
            | SurfaceNodeKind::ChoiceTerm
            | SurfaceNodeKind::ApplicationTerm
            | SurfaceNodeKind::StructureConstructor
            | SurfaceNodeKind::FieldArgument
            | SurfaceNodeKind::SetEnumeration
            | SurfaceNodeKind::ModulePath
            | SurfaceNodeKind::NamespacePath
            | SurfaceNodeKind::QualifiedSymbol
            | SurfaceNodeKind::PathSegment
            | SurfaceNodeKind::RelativePrefix
            | SurfaceNodeKind::Token(_)
            | SurfaceNodeKind::InfixExpression(_) => None,
        }
    }

    pub fn as_compilation_unit(self) -> Option<Self> {
        match &self.node.kind {
            SurfaceNodeKind::CompilationUnit => Some(self),
            _ => None,
        }
    }

    pub fn as_item_list(self) -> Option<Self> {
        match &self.node.kind {
            SurfaceNodeKind::ItemList => Some(self),
            _ => None,
        }
    }

    pub fn as_placeholder_item(self) -> Option<Self> {
        match &self.node.kind {
            SurfaceNodeKind::PlaceholderItem => Some(self),
            _ => None,
        }
    }

    pub fn as_import_item(self) -> Option<Self> {
        match &self.node.kind {
            SurfaceNodeKind::ImportItem => Some(self),
            _ => None,
        }
    }

    pub fn as_import_alias_decl(self) -> Option<Self> {
        match &self.node.kind {
            SurfaceNodeKind::ImportAliasDecl => Some(self),
            _ => None,
        }
    }

    pub fn as_module_branch_import(self) -> Option<Self> {
        match &self.node.kind {
            SurfaceNodeKind::ModuleBranchImport => Some(self),
            _ => None,
        }
    }

    pub fn as_export_item(self) -> Option<Self> {
        match &self.node.kind {
            SurfaceNodeKind::ExportItem => Some(self),
            _ => None,
        }
    }

    pub fn as_visibility_marker(self) -> Option<Self> {
        match &self.node.kind {
            SurfaceNodeKind::VisibilityMarker => Some(self),
            _ => None,
        }
    }

    pub fn as_visible_item(self) -> Option<Self> {
        match &self.node.kind {
            SurfaceNodeKind::VisibleItem => Some(self),
            _ => None,
        }
    }

    pub fn as_reserve_item(self) -> Option<Self> {
        match &self.node.kind {
            SurfaceNodeKind::ReserveItem => Some(self),
            _ => None,
        }
    }

    pub fn as_reserve_segment(self) -> Option<Self> {
        match &self.node.kind {
            SurfaceNodeKind::ReserveSegment => Some(self),
            _ => None,
        }
    }

    pub fn as_type_expression(self) -> Option<Self> {
        match &self.node.kind {
            SurfaceNodeKind::TypeExpression => Some(self),
            _ => None,
        }
    }

    pub fn as_attribute_chain(self) -> Option<Self> {
        match &self.node.kind {
            SurfaceNodeKind::AttributeChain => Some(self),
            _ => None,
        }
    }

    pub fn as_attribute_ref(self) -> Option<Self> {
        match &self.node.kind {
            SurfaceNodeKind::AttributeRef => Some(self),
            _ => None,
        }
    }

    pub fn as_parameter_prefix(self) -> Option<Self> {
        match &self.node.kind {
            SurfaceNodeKind::ParameterPrefix => Some(self),
            _ => None,
        }
    }

    pub fn as_type_head(self) -> Option<Self> {
        match &self.node.kind {
            SurfaceNodeKind::TypeHead => Some(self),
            _ => None,
        }
    }

    pub fn as_type_arguments(self) -> Option<Self> {
        match &self.node.kind {
            SurfaceNodeKind::TypeArguments => Some(self),
            _ => None,
        }
    }

    pub fn as_term_placeholder(self) -> Option<Self> {
        match &self.node.kind {
            SurfaceNodeKind::TermPlaceholder => Some(self),
            _ => None,
        }
    }

    pub fn as_term_expression(self) -> Option<Self> {
        match &self.node.kind {
            SurfaceNodeKind::TermExpression => Some(self),
            _ => None,
        }
    }

    pub fn as_term_reference(self) -> Option<Self> {
        match &self.node.kind {
            SurfaceNodeKind::TermReference => Some(self),
            _ => None,
        }
    }

    pub fn as_numeral_term(self) -> Option<Self> {
        match &self.node.kind {
            SurfaceNodeKind::NumeralTerm => Some(self),
            _ => None,
        }
    }

    pub fn as_it_term(self) -> Option<Self> {
        match &self.node.kind {
            SurfaceNodeKind::ItTerm => Some(self),
            _ => None,
        }
    }

    pub fn as_parenthesized_term(self) -> Option<Self> {
        match &self.node.kind {
            SurfaceNodeKind::ParenthesizedTerm => Some(self),
            _ => None,
        }
    }

    pub fn as_choice_term(self) -> Option<Self> {
        match &self.node.kind {
            SurfaceNodeKind::ChoiceTerm => Some(self),
            _ => None,
        }
    }

    pub fn as_application_term(self) -> Option<Self> {
        match &self.node.kind {
            SurfaceNodeKind::ApplicationTerm => Some(self),
            _ => None,
        }
    }

    pub fn as_structure_constructor(self) -> Option<Self> {
        match &self.node.kind {
            SurfaceNodeKind::StructureConstructor => Some(self),
            _ => None,
        }
    }

    pub fn as_field_argument(self) -> Option<Self> {
        match &self.node.kind {
            SurfaceNodeKind::FieldArgument => Some(self),
            _ => None,
        }
    }

    pub fn as_set_enumeration(self) -> Option<Self> {
        match &self.node.kind {
            SurfaceNodeKind::SetEnumeration => Some(self),
            _ => None,
        }
    }

    pub fn as_module_path(self) -> Option<Self> {
        match &self.node.kind {
            SurfaceNodeKind::ModulePath => Some(self),
            _ => None,
        }
    }

    pub fn as_namespace_path(self) -> Option<Self> {
        match &self.node.kind {
            SurfaceNodeKind::NamespacePath => Some(self),
            _ => None,
        }
    }

    pub fn as_qualified_symbol(self) -> Option<Self> {
        match &self.node.kind {
            SurfaceNodeKind::QualifiedSymbol => Some(self),
            _ => None,
        }
    }

    pub fn as_path_segment(self) -> Option<Self> {
        match &self.node.kind {
            SurfaceNodeKind::PathSegment => Some(self),
            _ => None,
        }
    }

    pub fn as_relative_prefix(self) -> Option<Self> {
        match &self.node.kind {
            SurfaceNodeKind::RelativePrefix => Some(self),
            _ => None,
        }
    }

    pub fn child_views(self) -> impl Iterator<Item = SurfaceNodeView<'a>> {
        self.node
            .children
            .iter()
            .filter_map(|child| self.ast.node_view(*child))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SurfaceNodeId(usize);

impl SurfaceNodeId {
    const fn new(index: usize) -> Self {
        Self(index)
    }

    pub const fn index(self) -> usize {
        self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SurfaceNode {
    pub kind: SurfaceNodeKind,
    pub range: SourceRange,
    pub children: Vec<SurfaceNodeId>,
    pub recovered: bool,
}

impl SurfaceNode {
    pub fn new(kind: SurfaceNodeKind, range: SourceRange, children: Vec<SurfaceNodeId>) -> Self {
        Self {
            kind,
            range,
            children,
            recovered: false,
        }
    }

    pub fn recovered(
        kind: SurfaceNodeKind,
        range: SourceRange,
        children: Vec<SurfaceNodeId>,
    ) -> Self {
        Self {
            kind,
            range,
            children,
            recovered: true,
        }
    }

    pub fn token_text(&self) -> Option<&str> {
        match &self.kind {
            SurfaceNodeKind::Token(token) => Some(token.text.as_ref()),
            SurfaceNodeKind::Root
            | SurfaceNodeKind::CompilationUnit
            | SurfaceNodeKind::ItemList
            | SurfaceNodeKind::PlaceholderItem
            | SurfaceNodeKind::ImportItem
            | SurfaceNodeKind::ImportAliasDecl
            | SurfaceNodeKind::ModuleBranchImport
            | SurfaceNodeKind::ExportItem
            | SurfaceNodeKind::VisibilityMarker
            | SurfaceNodeKind::VisibleItem
            | SurfaceNodeKind::ReserveItem
            | SurfaceNodeKind::ReserveSegment
            | SurfaceNodeKind::TypeExpression
            | SurfaceNodeKind::AttributeChain
            | SurfaceNodeKind::AttributeRef
            | SurfaceNodeKind::ParameterPrefix
            | SurfaceNodeKind::TypeHead
            | SurfaceNodeKind::TypeArguments
            | SurfaceNodeKind::TermPlaceholder
            | SurfaceNodeKind::TermExpression
            | SurfaceNodeKind::TermReference
            | SurfaceNodeKind::NumeralTerm
            | SurfaceNodeKind::ItTerm
            | SurfaceNodeKind::ParenthesizedTerm
            | SurfaceNodeKind::ChoiceTerm
            | SurfaceNodeKind::ApplicationTerm
            | SurfaceNodeKind::StructureConstructor
            | SurfaceNodeKind::FieldArgument
            | SurfaceNodeKind::SetEnumeration
            | SurfaceNodeKind::ModulePath
            | SurfaceNodeKind::NamespacePath
            | SurfaceNodeKind::QualifiedSymbol
            | SurfaceNodeKind::PathSegment
            | SurfaceNodeKind::RelativePrefix
            | SurfaceNodeKind::InfixExpression(_)
            | SurfaceNodeKind::ErrorRecovery(_) => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum SurfaceNodeKind {
    Root,
    Token(SurfaceToken),
    InfixExpression(SurfaceInfixOperator),
    ErrorRecovery(SyntaxRecoveryKind),
    CompilationUnit,
    ItemList,
    PlaceholderItem,
    ImportItem,
    ImportAliasDecl,
    ModuleBranchImport,
    ExportItem,
    VisibilityMarker,
    VisibleItem,
    ReserveItem,
    ReserveSegment,
    TypeExpression,
    AttributeChain,
    AttributeRef,
    ParameterPrefix,
    TypeHead,
    TypeArguments,
    TermPlaceholder,
    TermExpression,
    TermReference,
    NumeralTerm,
    ItTerm,
    ParenthesizedTerm,
    ChoiceTerm,
    ApplicationTerm,
    StructureConstructor,
    FieldArgument,
    SetEnumeration,
    ModulePath,
    NamespacePath,
    QualifiedSymbol,
    PathSegment,
    RelativePrefix,
}

impl SurfaceNodeKind {
    pub const fn syntax_kind(&self) -> SyntaxKind {
        match self {
            Self::Root => SyntaxKind::Root,
            Self::Token(_) => SyntaxKind::Token,
            Self::InfixExpression(_) => SyntaxKind::InfixExpression,
            Self::ErrorRecovery(_) => SyntaxKind::ErrorRecovery,
            Self::CompilationUnit => SyntaxKind::CompilationUnit,
            Self::ItemList => SyntaxKind::ItemList,
            Self::PlaceholderItem => SyntaxKind::PlaceholderItem,
            Self::ImportItem => SyntaxKind::ImportItem,
            Self::ImportAliasDecl => SyntaxKind::ImportAliasDecl,
            Self::ModuleBranchImport => SyntaxKind::ModuleBranchImport,
            Self::ExportItem => SyntaxKind::ExportItem,
            Self::VisibilityMarker => SyntaxKind::VisibilityMarker,
            Self::VisibleItem => SyntaxKind::VisibleItem,
            Self::ReserveItem => SyntaxKind::ReserveItem,
            Self::ReserveSegment => SyntaxKind::ReserveSegment,
            Self::TypeExpression => SyntaxKind::TypeExpression,
            Self::AttributeChain => SyntaxKind::AttributeChain,
            Self::AttributeRef => SyntaxKind::AttributeRef,
            Self::ParameterPrefix => SyntaxKind::ParameterPrefix,
            Self::TypeHead => SyntaxKind::TypeHead,
            Self::TypeArguments => SyntaxKind::TypeArguments,
            Self::TermPlaceholder => SyntaxKind::TermPlaceholder,
            Self::TermExpression => SyntaxKind::TermExpression,
            Self::TermReference => SyntaxKind::TermReference,
            Self::NumeralTerm => SyntaxKind::NumeralTerm,
            Self::ItTerm => SyntaxKind::ItTerm,
            Self::ParenthesizedTerm => SyntaxKind::ParenthesizedTerm,
            Self::ChoiceTerm => SyntaxKind::ChoiceTerm,
            Self::ApplicationTerm => SyntaxKind::ApplicationTerm,
            Self::StructureConstructor => SyntaxKind::StructureConstructor,
            Self::FieldArgument => SyntaxKind::FieldArgument,
            Self::SetEnumeration => SyntaxKind::SetEnumeration,
            Self::ModulePath => SyntaxKind::ModulePath,
            Self::NamespacePath => SyntaxKind::NamespacePath,
            Self::QualifiedSymbol => SyntaxKind::QualifiedSymbol,
            Self::PathSegment => SyntaxKind::PathSegment,
            Self::RelativePrefix => SyntaxKind::RelativePrefix,
        }
    }

    pub const fn is_structural(&self) -> bool {
        !matches!(self, Self::Token(_) | Self::ErrorRecovery(_))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SurfaceToken {
    pub kind: SurfaceTokenKind,
    pub text: Arc<str>,
}

impl SurfaceToken {
    pub fn new(kind: SurfaceTokenKind, text: impl Into<Arc<str>>) -> Self {
        Self {
            kind,
            text: text.into(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum SurfaceTokenKind {
    Identifier,
    ReservedWord,
    ReservedSymbol,
    Numeral,
    LexemeRun,
    UserSymbol,
    StringLiteral,
    ErrorRecovery,
    Unknown,
}

impl SurfaceTokenKind {
    pub const fn syntax_kind(self) -> SyntaxKind {
        match self {
            Self::Identifier => SyntaxKind::TokenIdentifier,
            Self::ReservedWord => SyntaxKind::TokenReservedWord,
            Self::ReservedSymbol => SyntaxKind::TokenReservedSymbol,
            Self::Numeral => SyntaxKind::TokenNumeral,
            Self::LexemeRun => SyntaxKind::TokenLexemeRun,
            Self::UserSymbol => SyntaxKind::TokenUserSymbol,
            Self::StringLiteral => SyntaxKind::TokenStringLiteral,
            Self::ErrorRecovery => SyntaxKind::TokenErrorRecovery,
            Self::Unknown => SyntaxKind::TokenUnknown,
        }
    }

    const fn snapshot_name(self) -> &'static str {
        match self {
            Self::Identifier => "Identifier",
            Self::ReservedWord => "ReservedWord",
            Self::ReservedSymbol => "ReservedSymbol",
            Self::Numeral => "Numeral",
            Self::LexemeRun => "LexemeRun",
            Self::UserSymbol => "UserSymbol",
            Self::StringLiteral => "StringLiteral",
            Self::ErrorRecovery => "ErrorRecovery",
            Self::Unknown => "Unknown",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SurfaceInfixOperator {
    pub spelling: Arc<str>,
    pub precedence: u8,
    pub associativity: SurfaceOperatorAssociativity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SurfaceOperatorAssociativity {
    Left,
    Right,
    NonAssociative,
}

impl SurfaceOperatorAssociativity {
    const fn snapshot_name(self) -> &'static str {
        match self {
            Self::Left => "Left",
            Self::Right => "Right",
            Self::NonAssociative => "NonAssociative",
        }
    }
}

fn write_snapshot_node(output: &mut String, view: SurfaceNodeView<'_>, indent: usize) {
    write_snapshot_indent(output, indent);
    match view.kind() {
        SurfaceNodeKind::Root => output.push_str("Root"),
        SurfaceNodeKind::CompilationUnit => output.push_str("CompilationUnit"),
        SurfaceNodeKind::ItemList => output.push_str("ItemList"),
        SurfaceNodeKind::PlaceholderItem => output.push_str("PlaceholderItem"),
        SurfaceNodeKind::ImportItem => output.push_str("ImportItem"),
        SurfaceNodeKind::ImportAliasDecl => output.push_str("ImportAliasDecl"),
        SurfaceNodeKind::ModuleBranchImport => output.push_str("ModuleBranchImport"),
        SurfaceNodeKind::ExportItem => output.push_str("ExportItem"),
        SurfaceNodeKind::VisibilityMarker => output.push_str("VisibilityMarker"),
        SurfaceNodeKind::VisibleItem => output.push_str("VisibleItem"),
        SurfaceNodeKind::ReserveItem => output.push_str("ReserveItem"),
        SurfaceNodeKind::ReserveSegment => output.push_str("ReserveSegment"),
        SurfaceNodeKind::TypeExpression => output.push_str("TypeExpression"),
        SurfaceNodeKind::AttributeChain => output.push_str("AttributeChain"),
        SurfaceNodeKind::AttributeRef => output.push_str("AttributeRef"),
        SurfaceNodeKind::ParameterPrefix => output.push_str("ParameterPrefix"),
        SurfaceNodeKind::TypeHead => output.push_str("TypeHead"),
        SurfaceNodeKind::TypeArguments => output.push_str("TypeArguments"),
        SurfaceNodeKind::TermPlaceholder => output.push_str("TermPlaceholder"),
        SurfaceNodeKind::TermExpression => output.push_str("TermExpression"),
        SurfaceNodeKind::TermReference => output.push_str("TermReference"),
        SurfaceNodeKind::NumeralTerm => output.push_str("NumeralTerm"),
        SurfaceNodeKind::ItTerm => output.push_str("ItTerm"),
        SurfaceNodeKind::ParenthesizedTerm => output.push_str("ParenthesizedTerm"),
        SurfaceNodeKind::ChoiceTerm => output.push_str("ChoiceTerm"),
        SurfaceNodeKind::ApplicationTerm => output.push_str("ApplicationTerm"),
        SurfaceNodeKind::StructureConstructor => output.push_str("StructureConstructor"),
        SurfaceNodeKind::FieldArgument => output.push_str("FieldArgument"),
        SurfaceNodeKind::SetEnumeration => output.push_str("SetEnumeration"),
        SurfaceNodeKind::ModulePath => output.push_str("ModulePath"),
        SurfaceNodeKind::NamespacePath => output.push_str("NamespacePath"),
        SurfaceNodeKind::QualifiedSymbol => output.push_str("QualifiedSymbol"),
        SurfaceNodeKind::PathSegment => output.push_str("PathSegment"),
        SurfaceNodeKind::RelativePrefix => output.push_str("RelativePrefix"),
        SurfaceNodeKind::Token(token) => {
            let _ = write!(
                output,
                "Token kind={} text=\"{}\"",
                token.kind.snapshot_name(),
                SnapshotEscaped(token.text.as_ref())
            );
        }
        SurfaceNodeKind::InfixExpression(operator) => {
            let _ = write!(
                output,
                "InfixExpression spelling=\"{}\" precedence={} associativity={}",
                SnapshotEscaped(operator.spelling.as_ref()),
                operator.precedence,
                operator.associativity.snapshot_name()
            );
        }
        SurfaceNodeKind::ErrorRecovery(kind) => {
            let _ = write!(
                output,
                "ErrorRecovery kind={}",
                recovery_snapshot_name(*kind)
            );
        }
    }
    let range = view.range();
    let _ = writeln!(
        output,
        " range={}..{} recovered={}",
        range.start,
        range.end,
        view.is_recovered()
    );
    for child in view.child_views() {
        write_snapshot_node(output, child, indent + 1);
    }
}

fn write_snapshot_indent(output: &mut String, indent: usize) {
    for _ in 0..indent {
        output.push_str("  ");
    }
}

fn recovery_snapshot_name(kind: SyntaxRecoveryKind) -> &'static str {
    match kind {
        SyntaxRecoveryKind::ErrorToken => "ErrorToken",
        SyntaxRecoveryKind::MissingEnd => "MissingEnd",
        SyntaxRecoveryKind::MissingStringLiteral => "MissingStringLiteral",
        SyntaxRecoveryKind::MissingItem => "MissingItem",
        SyntaxRecoveryKind::MissingTypeExpression => "MissingTypeExpression",
        SyntaxRecoveryKind::MissingTerm => "MissingTerm",
        SyntaxRecoveryKind::MissingFormula => "MissingFormula",
        SyntaxRecoveryKind::MissingStatement => "MissingStatement",
        SyntaxRecoveryKind::MissingProofStep => "MissingProofStep",
        SyntaxRecoveryKind::MissingAnnotationArgument => "MissingAnnotationArgument",
        SyntaxRecoveryKind::SkippedToken => "SkippedToken",
        SyntaxRecoveryKind::UnmatchedOpeningDelimiter => "UnmatchedOpeningDelimiter",
        SyntaxRecoveryKind::UnmatchedClosingDelimiter => "UnmatchedClosingDelimiter",
        SyntaxRecoveryKind::MalformedAnnotation => "MalformedAnnotation",
    }
}

struct SnapshotEscaped<'a>(&'a str);

impl std::fmt::Display for SnapshotEscaped<'_> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for character in self.0.chars() {
            for escaped in character.escape_default() {
                formatter.write_char(escaped)?;
            }
        }
        Ok(())
    }
}

fn build_green_tree(nodes: &[SurfaceNode], root: Option<SurfaceNodeId>) -> rowan::GreenNode {
    let mut builder = rowan::GreenNodeBuilder::new();
    builder.start_node(rowan::SyntaxKind(SyntaxKind::Root as u16));
    if let Some(root) = root.and_then(|root| nodes.get(root.index()).map(|node| (root, node))) {
        append_root_contents(&mut builder, nodes, root.1);
    }
    builder.finish_node();
    builder.finish()
}

fn append_root_contents(
    builder: &mut rowan::GreenNodeBuilder<'_>,
    nodes: &[SurfaceNode],
    root: &SurfaceNode,
) {
    let structural_children = root
        .children
        .iter()
        .copied()
        .filter(|child| {
            node_kind(nodes, *child).is_some_and(|kind| {
                !matches!(
                    kind,
                    SurfaceNodeKind::Token(_) | SurfaceNodeKind::ErrorRecovery(_)
                )
            })
        })
        .collect::<Vec<_>>();
    let structural_tokens = structural_children
        .iter()
        .flat_map(|child| collect_token_descendants(nodes, *child))
        .collect::<Vec<_>>();
    let mut appended_structural = Vec::new();

    for child in root.children.iter().copied() {
        if structural_tokens.contains(&child) {
            let containing_structural = structural_children.iter().copied().find(|structure| {
                collect_token_descendants(nodes, *structure)
                    .first()
                    .copied()
                    == Some(child)
            });
            if let Some(structure) = containing_structural {
                append_green_node(builder, nodes, structure);
                appended_structural.push(structure);
            }
            continue;
        }
        if structural_children.contains(&child) {
            if !appended_structural.contains(&child) {
                append_green_node(builder, nodes, child);
                appended_structural.push(child);
            }
            continue;
        }
        append_green_node(builder, nodes, child);
    }
}

fn append_green_node(
    builder: &mut rowan::GreenNodeBuilder<'_>,
    nodes: &[SurfaceNode],
    id: SurfaceNodeId,
) {
    let Some(node) = nodes.get(id.index()) else {
        return;
    };
    builder.start_node(rowan::SyntaxKind(node.kind.syntax_kind() as u16));
    if let SurfaceNodeKind::Token(token) = &node.kind {
        builder.token(
            rowan::SyntaxKind(token.kind.syntax_kind() as u16),
            token.text.as_ref(),
        );
    }
    for child in children_to_append(nodes, node) {
        append_green_node(builder, nodes, child);
    }
    builder.finish_node();
}

fn children_to_append(nodes: &[SurfaceNode], node: &SurfaceNode) -> Vec<SurfaceNodeId> {
    if matches!(node.kind, SurfaceNodeKind::ErrorRecovery(_)) {
        node.children
            .iter()
            .copied()
            .filter(|child| {
                nodes
                    .get(child.index())
                    .is_some_and(|child| contains_range(node.range, child.range))
            })
            .collect()
    } else {
        node.children.clone()
    }
}

fn collect_token_descendants(nodes: &[SurfaceNode], id: SurfaceNodeId) -> Vec<SurfaceNodeId> {
    let Some(node) = nodes.get(id.index()) else {
        return Vec::new();
    };
    if matches!(node.kind, SurfaceNodeKind::Token(_)) {
        return vec![id];
    }
    node.children
        .iter()
        .flat_map(|child| collect_token_descendants(nodes, *child))
        .collect()
}

fn node_kind(nodes: &[SurfaceNode], id: SurfaceNodeId) -> Option<&SurfaceNodeKind> {
    nodes.get(id.index()).map(|node| &node.kind)
}

fn contains_range(parent: SourceRange, child: SourceRange) -> bool {
    parent.source_id == child.source_id && parent.start <= child.start && child.end <= parent.end
}

#[cfg(test)]
mod tests {
    use super::{
        SurfaceAstBuilder, SurfaceInfixOperator, SurfaceNodeKind, SurfaceOperatorAssociativity,
        SurfaceTokenKind, SyntaxKind,
    };
    use crate::SyntaxRecoveryKind;
    use crate::{
        SkippedTokenReason, SurfaceTriviaBuilder, TriviaAttachmentTarget, TriviaNodeTarget,
        TriviaPlacement, WhitespaceHintKind,
    };
    use mizar_session::{
        BuildSnapshotId, CommentKind, Hash, InMemorySessionIdAllocator, SessionIdAllocator,
        SourceAnchor, SourceId, SourceRange,
    };

    #[test]
    fn builder_round_trips_into_rowan_backed_tree() {
        let source_id = source_id(1);
        let ast = expression_ast(source_id);

        assert_eq!(ast.token_texts(), vec!["a", "++", "b"]);
        assert_eq!(ast.rowan_root().kind(), SyntaxKind::Root);
        let rowan_kinds = ast
            .rowan_root()
            .descendants_with_tokens()
            .map(|element| element.kind())
            .collect::<Vec<_>>();
        assert_eq!(
            rowan_kinds,
            vec![
                SyntaxKind::Root,
                SyntaxKind::InfixExpression,
                SyntaxKind::Token,
                SyntaxKind::TokenIdentifier,
                SyntaxKind::Token,
                SyntaxKind::TokenUserSymbol,
                SyntaxKind::Token,
                SyntaxKind::TokenIdentifier,
            ]
        );
        let rowan_tokens = ast
            .rowan_root()
            .descendants_with_tokens()
            .filter_map(|element| element.into_token())
            .map(|token| (token.kind(), token.text().to_owned()))
            .collect::<Vec<_>>();
        assert_eq!(
            rowan_tokens,
            vec![
                (SyntaxKind::TokenIdentifier, "a".to_owned()),
                (SyntaxKind::TokenUserSymbol, "++".to_owned()),
                (SyntaxKind::TokenIdentifier, "b".to_owned()),
            ]
        );
        assert_eq!(
            ast.rowan_root()
                .descendants_with_tokens()
                .filter(|element| element.as_token().is_some())
                .count(),
            3,
            "the rowan storage is source-shaped even while compatibility views keep dense token ids"
        );
        assert_eq!(ast.green_node(), expression_ast(source_id).green_node());
    }

    #[test]
    fn typed_accessors_cover_current_node_and_token_kinds() {
        let source_id = source_id(2);
        let mut builder = SurfaceAstBuilder::new(source_id);
        let token_kinds = [
            SurfaceTokenKind::Identifier,
            SurfaceTokenKind::ReservedWord,
            SurfaceTokenKind::ReservedSymbol,
            SurfaceTokenKind::Numeral,
            SurfaceTokenKind::LexemeRun,
            SurfaceTokenKind::UserSymbol,
            SurfaceTokenKind::StringLiteral,
            SurfaceTokenKind::ErrorRecovery,
            SurfaceTokenKind::Unknown,
        ];
        let mut token_ids = Vec::new();
        for (index, kind) in token_kinds.into_iter().enumerate() {
            token_ids.push(builder.add_token(
                kind,
                format!("t{index}"),
                range(source_id, index, index + 1),
            ));
        }
        let recovered_token = builder.add_recovered_token(
            SurfaceTokenKind::ErrorRecovery,
            "bad",
            range(source_id, 20, 21),
        );
        let module_prefix_token = builder.add_token(
            SurfaceTokenKind::ReservedSymbol,
            "..",
            range(source_id, 30, 32),
        );
        let module_segment_a = builder.add_token(
            SurfaceTokenKind::Identifier,
            "std",
            range(source_id, 32, 35),
        );
        let module_dot_token = builder.add_token(
            SurfaceTokenKind::ReservedSymbol,
            ".",
            range(source_id, 35, 36),
        );
        let module_segment_b = builder.add_token(
            SurfaceTokenKind::Identifier,
            "algebra",
            range(source_id, 36, 43),
        );
        let namespace_segment_a = builder.add_token(
            SurfaceTokenKind::Identifier,
            "mml",
            range(source_id, 44, 47),
        );
        let namespace_dot_token = builder.add_token(
            SurfaceTokenKind::ReservedSymbol,
            ".",
            range(source_id, 47, 48),
        );
        let namespace_segment_b = builder.add_token(
            SurfaceTokenKind::Identifier,
            "nat",
            range(source_id, 48, 51),
        );
        let qualified_segment_a = builder.add_token(
            SurfaceTokenKind::Identifier,
            "algebra",
            range(source_id, 52, 59),
        );
        let qualified_dot_token = builder.add_token(
            SurfaceTokenKind::ReservedSymbol,
            ".",
            range(source_id, 59, 60),
        );
        let qualified_symbol_token = builder.add_token(
            SurfaceTokenKind::UserSymbol,
            "Group",
            range(source_id, 60, 65),
        );
        let item_keyword = builder.add_token(
            SurfaceTokenKind::ReservedWord,
            "theorem",
            range(source_id, 66, 73),
        );
        let item_semicolon = builder.add_token(
            SurfaceTokenKind::ReservedSymbol,
            ";",
            range(source_id, 73, 74),
        );
        let import_keyword = builder.add_token(
            SurfaceTokenKind::ReservedWord,
            "import",
            range(source_id, 75, 81),
        );
        let import_path_prefix = builder.add_token(
            SurfaceTokenKind::ReservedSymbol,
            ".",
            range(source_id, 82, 83),
        );
        let import_path_tools = builder.add_token(
            SurfaceTokenKind::Identifier,
            "Tools",
            range(source_id, 83, 88),
        );
        let import_branch_opener = builder.add_token(
            SurfaceTokenKind::ReservedSymbol,
            ".{",
            range(source_id, 88, 90),
        );
        let import_branch_segment = builder.add_token(
            SurfaceTokenKind::Identifier,
            "Group",
            range(source_id, 90, 95),
        );
        let import_branch_close = builder.add_token(
            SurfaceTokenKind::ReservedSymbol,
            "}",
            range(source_id, 95, 96),
        );
        let import_comma = builder.add_token(
            SurfaceTokenKind::ReservedSymbol,
            ",",
            range(source_id, 96, 97),
        );
        let import_alias_path = builder.add_token(
            SurfaceTokenKind::Identifier,
            "Std",
            range(source_id, 98, 101),
        );
        let import_as = builder.add_token(
            SurfaceTokenKind::ReservedWord,
            "as",
            range(source_id, 102, 104),
        );
        let import_alias = builder.add_token(
            SurfaceTokenKind::Identifier,
            "G",
            range(source_id, 105, 106),
        );
        let import_semicolon = builder.add_token(
            SurfaceTokenKind::ReservedSymbol,
            ";",
            range(source_id, 106, 107),
        );
        let export_keyword = builder.add_token(
            SurfaceTokenKind::ReservedWord,
            "export",
            range(source_id, 108, 114),
        );
        let export_path_std = builder.add_token(
            SurfaceTokenKind::Identifier,
            "Std",
            range(source_id, 115, 118),
        );
        let export_semicolon = builder.add_token(
            SurfaceTokenKind::ReservedSymbol,
            ";",
            range(source_id, 118, 119),
        );
        let visibility_public = builder.add_token(
            SurfaceTokenKind::ReservedWord,
            "public",
            range(source_id, 120, 126),
        );
        let visible_theorem = builder.add_token(
            SurfaceTokenKind::ReservedWord,
            "theorem",
            range(source_id, 127, 134),
        );
        let visible_semicolon = builder.add_token(
            SurfaceTokenKind::ReservedSymbol,
            ";",
            range(source_id, 134, 135),
        );
        let recovery = builder.add_recovery(
            SyntaxRecoveryKind::ErrorToken,
            range(source_id, 9, 9),
            Vec::new(),
        );
        let infix = builder.add_node(
            SurfaceNodeKind::InfixExpression(SurfaceInfixOperator {
                spelling: "++".into(),
                precedence: 10,
                associativity: SurfaceOperatorAssociativity::Left,
            }),
            range(source_id, 0, 3),
            token_ids[..3].to_vec(),
        );
        let module_prefix = builder.add_node(
            SurfaceNodeKind::RelativePrefix,
            range(source_id, 30, 32),
            vec![module_prefix_token],
        );
        let module_path_segment_a = builder.add_node(
            SurfaceNodeKind::PathSegment,
            range(source_id, 32, 35),
            vec![module_segment_a],
        );
        let module_path_segment_b = builder.add_node(
            SurfaceNodeKind::PathSegment,
            range(source_id, 36, 43),
            vec![module_segment_b],
        );
        let module_path = builder.add_node(
            SurfaceNodeKind::ModulePath,
            range(source_id, 30, 43),
            vec![
                module_prefix,
                module_path_segment_a,
                module_dot_token,
                module_path_segment_b,
            ],
        );
        let namespace_path_segment_a = builder.add_node(
            SurfaceNodeKind::PathSegment,
            range(source_id, 44, 47),
            vec![namespace_segment_a],
        );
        let namespace_path_segment_b = builder.add_node(
            SurfaceNodeKind::PathSegment,
            range(source_id, 48, 51),
            vec![namespace_segment_b],
        );
        let namespace_path = builder.add_node(
            SurfaceNodeKind::NamespacePath,
            range(source_id, 44, 51),
            vec![
                namespace_path_segment_a,
                namespace_dot_token,
                namespace_path_segment_b,
            ],
        );
        let qualified_path_segment = builder.add_node(
            SurfaceNodeKind::PathSegment,
            range(source_id, 52, 59),
            vec![qualified_segment_a],
        );
        let qualified_final_segment = builder.add_node(
            SurfaceNodeKind::PathSegment,
            range(source_id, 60, 65),
            vec![qualified_symbol_token],
        );
        let qualified_symbol = builder.add_node(
            SurfaceNodeKind::QualifiedSymbol,
            range(source_id, 52, 65),
            vec![
                qualified_path_segment,
                qualified_dot_token,
                qualified_final_segment,
            ],
        );
        let placeholder_item = builder.add_node(
            SurfaceNodeKind::PlaceholderItem,
            range(source_id, 66, 74),
            vec![item_keyword, item_semicolon],
        );
        let import_path_prefix_node = builder.add_node(
            SurfaceNodeKind::RelativePrefix,
            range(source_id, 82, 83),
            vec![import_path_prefix],
        );
        let import_path_tools_node = builder.add_node(
            SurfaceNodeKind::PathSegment,
            range(source_id, 83, 88),
            vec![import_path_tools],
        );
        let import_branch_path = builder.add_node(
            SurfaceNodeKind::ModulePath,
            range(source_id, 82, 88),
            vec![import_path_prefix_node, import_path_tools_node],
        );
        let import_branch_segment_node = builder.add_node(
            SurfaceNodeKind::PathSegment,
            range(source_id, 90, 95),
            vec![import_branch_segment],
        );
        let module_branch_import = builder.add_node(
            SurfaceNodeKind::ModuleBranchImport,
            range(source_id, 82, 96),
            vec![
                import_branch_path,
                import_branch_opener,
                import_branch_segment_node,
                import_branch_close,
            ],
        );
        let import_alias_path_node = builder.add_node(
            SurfaceNodeKind::PathSegment,
            range(source_id, 98, 101),
            vec![import_alias_path],
        );
        let import_alias_module_path = builder.add_node(
            SurfaceNodeKind::ModulePath,
            range(source_id, 98, 101),
            vec![import_alias_path_node],
        );
        let import_alias_node = builder.add_node(
            SurfaceNodeKind::PathSegment,
            range(source_id, 105, 106),
            vec![import_alias],
        );
        let import_alias_decl = builder.add_node(
            SurfaceNodeKind::ImportAliasDecl,
            range(source_id, 98, 106),
            vec![import_alias_module_path, import_as, import_alias_node],
        );
        let import_item = builder.add_node(
            SurfaceNodeKind::ImportItem,
            range(source_id, 75, 107),
            vec![
                import_keyword,
                module_branch_import,
                import_comma,
                import_alias_decl,
                import_semicolon,
            ],
        );
        let export_path_segment = builder.add_node(
            SurfaceNodeKind::PathSegment,
            range(source_id, 115, 118),
            vec![export_path_std],
        );
        let export_module_path = builder.add_node(
            SurfaceNodeKind::ModulePath,
            range(source_id, 115, 118),
            vec![export_path_segment],
        );
        let export_item = builder.add_node(
            SurfaceNodeKind::ExportItem,
            range(source_id, 108, 119),
            vec![export_keyword, export_module_path, export_semicolon],
        );
        let visibility_marker = builder.add_node(
            SurfaceNodeKind::VisibilityMarker,
            range(source_id, 120, 126),
            vec![visibility_public],
        );
        let visible_placeholder = builder.add_node(
            SurfaceNodeKind::PlaceholderItem,
            range(source_id, 127, 135),
            vec![visible_theorem, visible_semicolon],
        );
        let visible_item = builder.add_node(
            SurfaceNodeKind::VisibleItem,
            range(source_id, 120, 135),
            vec![visibility_marker, visible_placeholder],
        );
        let item_list = builder.add_node(
            SurfaceNodeKind::ItemList,
            range(source_id, 66, 135),
            vec![placeholder_item, import_item, export_item, visible_item],
        );
        let compilation_unit = builder.add_node(
            SurfaceNodeKind::CompilationUnit,
            range(source_id, 66, 135),
            vec![item_list],
        );
        let path_tokens = [
            module_prefix_token,
            module_segment_a,
            module_dot_token,
            module_segment_b,
            namespace_segment_a,
            namespace_dot_token,
            namespace_segment_b,
            qualified_segment_a,
            qualified_dot_token,
            qualified_symbol_token,
        ];
        let import_tokens = [
            import_keyword,
            import_path_prefix,
            import_path_tools,
            import_branch_opener,
            import_branch_segment,
            import_branch_close,
            import_comma,
            import_alias_path,
            import_as,
            import_alias,
            import_semicolon,
        ];
        let task7_tokens = [
            export_keyword,
            export_path_std,
            export_semicolon,
            visibility_public,
            visible_theorem,
            visible_semicolon,
        ];
        let root_children = token_ids
            .iter()
            .copied()
            .chain([recovered_token])
            .chain(path_tokens)
            .chain([item_keyword, item_semicolon])
            .chain(import_tokens)
            .chain(task7_tokens)
            .chain([
                infix,
                module_path,
                namespace_path,
                qualified_symbol,
                compilation_unit,
                recovery,
            ])
            .collect::<Vec<_>>();
        let root = builder.add_node(
            SurfaceNodeKind::Root,
            range(source_id, 0, 135),
            root_children.clone(),
        );
        let ast = builder.finish(Some(root), Some(infix));

        let root_view = ast.root_view().unwrap();
        assert_eq!(root_view.id(), sid(root));
        assert_eq!(root_view.kind(), &SurfaceNodeKind::Root);
        assert_eq!(root_view.syntax_kind(), SyntaxKind::Root);
        assert_eq!(root_view.range(), range(source_id, 0, 135));
        assert!(!root_view.is_recovered());
        assert!(root_view.as_token().is_none());
        assert!(root_view.as_infix_expression().is_none());
        assert!(root_view.as_recovery().is_none());
        assert_eq!(
            root_view.children(),
            &root_children.iter().copied().map(sid).collect::<Vec<_>>()
        );
        assert_eq!(
            root_view
                .child_views()
                .map(super::SurfaceNodeView::id)
                .collect::<Vec<_>>(),
            root_view.children()
        );

        let expression_view = ast.expression_view().unwrap();
        let infix_operator = expression_view.as_infix_expression().unwrap();
        assert_eq!(expression_view.id(), sid(infix));
        assert_eq!(expression_view.syntax_kind(), SyntaxKind::InfixExpression);
        assert_eq!(expression_view.range(), range(source_id, 0, 3));
        assert_eq!(
            expression_view.children(),
            &token_ids[..3].iter().copied().map(sid).collect::<Vec<_>>()
        );
        assert_eq!(infix_operator.spelling.as_ref(), "++");
        assert_eq!(infix_operator.precedence, 10);
        assert_eq!(
            infix_operator.associativity,
            SurfaceOperatorAssociativity::Left
        );
        assert!(!expression_view.is_recovered());
        assert!(expression_view.as_token().is_none());
        assert!(expression_view.as_recovery().is_none());
        assert!(expression_view.as_module_path().is_none());
        assert!(expression_view.as_namespace_path().is_none());
        assert!(expression_view.as_qualified_symbol().is_none());

        let module_path_view = ast.node_view(sid(module_path)).unwrap();
        assert_eq!(module_path_view.syntax_kind(), SyntaxKind::ModulePath);
        assert_eq!(
            module_path_view.as_module_path().unwrap().id(),
            sid(module_path)
        );
        assert_eq!(module_path_view.range(), range(source_id, 30, 43));
        assert_eq!(
            module_path_view.children(),
            &[
                sid(module_prefix),
                sid(module_path_segment_a),
                sid(module_dot_token),
                sid(module_path_segment_b),
            ]
        );
        assert!(module_path_view.as_token().is_none());
        assert!(module_path_view.as_infix_expression().is_none());
        assert!(module_path_view.as_recovery().is_none());

        let namespace_path_view = ast.node_view(sid(namespace_path)).unwrap();
        assert_eq!(namespace_path_view.syntax_kind(), SyntaxKind::NamespacePath);
        assert_eq!(
            namespace_path_view.as_namespace_path().unwrap().id(),
            sid(namespace_path)
        );
        assert_eq!(
            namespace_path_view.children(),
            &[
                sid(namespace_path_segment_a),
                sid(namespace_dot_token),
                sid(namespace_path_segment_b),
            ]
        );

        let qualified_symbol_view = ast.node_view(sid(qualified_symbol)).unwrap();
        assert_eq!(
            qualified_symbol_view.syntax_kind(),
            SyntaxKind::QualifiedSymbol
        );
        assert_eq!(
            qualified_symbol_view.as_qualified_symbol().unwrap().id(),
            sid(qualified_symbol)
        );
        assert_eq!(
            qualified_symbol_view.children(),
            &[
                sid(qualified_path_segment),
                sid(qualified_dot_token),
                sid(qualified_final_segment),
            ]
        );

        let compilation_unit_view = ast.node_view(sid(compilation_unit)).unwrap();
        assert_eq!(
            compilation_unit_view.syntax_kind(),
            SyntaxKind::CompilationUnit
        );
        assert_eq!(
            compilation_unit_view
                .as_compilation_unit()
                .unwrap()
                .children(),
            &[sid(item_list)]
        );
        assert!(compilation_unit_view.as_token().is_none());
        assert!(compilation_unit_view.as_infix_expression().is_none());
        assert!(compilation_unit_view.as_recovery().is_none());
        let item_list_view = ast.node_view(sid(item_list)).unwrap();
        assert_eq!(item_list_view.syntax_kind(), SyntaxKind::ItemList);
        assert_eq!(
            item_list_view.as_item_list().unwrap().children(),
            &[
                sid(placeholder_item),
                sid(import_item),
                sid(export_item),
                sid(visible_item)
            ]
        );
        let placeholder_item_view = ast.node_view(sid(placeholder_item)).unwrap();
        assert_eq!(
            placeholder_item_view.syntax_kind(),
            SyntaxKind::PlaceholderItem
        );
        assert_eq!(
            placeholder_item_view
                .as_placeholder_item()
                .unwrap()
                .children(),
            &[sid(item_keyword), sid(item_semicolon)]
        );
        let import_item_view = ast.node_view(sid(import_item)).unwrap();
        assert_eq!(import_item_view.syntax_kind(), SyntaxKind::ImportItem);
        assert_eq!(
            import_item_view.as_import_item().unwrap().children(),
            &[
                sid(import_keyword),
                sid(module_branch_import),
                sid(import_comma),
                sid(import_alias_decl),
                sid(import_semicolon),
            ]
        );
        let branch_import_view = ast.node_view(sid(module_branch_import)).unwrap();
        assert_eq!(
            branch_import_view.syntax_kind(),
            SyntaxKind::ModuleBranchImport
        );
        assert_eq!(
            branch_import_view
                .as_module_branch_import()
                .unwrap()
                .children(),
            &[
                sid(import_branch_path),
                sid(import_branch_opener),
                sid(import_branch_segment_node),
                sid(import_branch_close),
            ]
        );
        let import_alias_view = ast.node_view(sid(import_alias_decl)).unwrap();
        assert_eq!(import_alias_view.syntax_kind(), SyntaxKind::ImportAliasDecl);
        assert_eq!(
            import_alias_view.as_import_alias_decl().unwrap().children(),
            &[
                sid(import_alias_module_path),
                sid(import_as),
                sid(import_alias_node)
            ]
        );
        let export_item_view = ast.node_view(sid(export_item)).unwrap();
        assert_eq!(export_item_view.syntax_kind(), SyntaxKind::ExportItem);
        assert_eq!(
            export_item_view.as_export_item().unwrap().children(),
            &[
                sid(export_keyword),
                sid(export_module_path),
                sid(export_semicolon)
            ]
        );
        let visibility_marker_view = ast.node_view(sid(visibility_marker)).unwrap();
        assert_eq!(
            visibility_marker_view.syntax_kind(),
            SyntaxKind::VisibilityMarker
        );
        assert_eq!(
            visibility_marker_view
                .as_visibility_marker()
                .unwrap()
                .children(),
            &[sid(visibility_public)]
        );
        let visible_item_view = ast.node_view(sid(visible_item)).unwrap();
        assert_eq!(visible_item_view.syntax_kind(), SyntaxKind::VisibleItem);
        assert_eq!(
            visible_item_view.as_visible_item().unwrap().children(),
            &[sid(visibility_marker), sid(visible_placeholder)]
        );

        let module_segment_view = ast.node_view(sid(module_path_segment_a)).unwrap();
        assert_eq!(module_segment_view.syntax_kind(), SyntaxKind::PathSegment);
        assert_eq!(
            module_segment_view.as_path_segment().unwrap().children(),
            &[sid(module_segment_a)]
        );
        let prefix_view = ast.node_view(sid(module_prefix)).unwrap();
        assert_eq!(prefix_view.syntax_kind(), SyntaxKind::RelativePrefix);
        assert_eq!(
            prefix_view.as_relative_prefix().unwrap().children(),
            &[sid(module_prefix_token)]
        );

        let recovery_view = ast.node_view(sid(recovery)).unwrap();
        assert_eq!(
            recovery_view.as_recovery(),
            Some(SyntaxRecoveryKind::ErrorToken)
        );
        assert_eq!(recovery_view.syntax_kind(), SyntaxKind::ErrorRecovery);
        assert_eq!(recovery_view.range(), range(source_id, 9, 9));
        assert!(recovery_view.is_recovered());
        assert!(recovery_view.children().is_empty());
        assert!(recovery_view.as_token().is_none());
        assert!(recovery_view.as_infix_expression().is_none());

        let recovered_token_view = ast.node_view(sid(recovered_token)).unwrap();
        assert!(recovered_token_view.is_recovered());
        assert_eq!(recovered_token_view.range(), range(source_id, 20, 21));
        assert_eq!(
            recovered_token_view.as_token().unwrap().text.as_ref(),
            "bad"
        );
        assert!(recovered_token_view.as_infix_expression().is_none());
        assert!(recovered_token_view.as_recovery().is_none());

        let actual_token_kinds = ast
            .token_views()
            .map(|view| view.as_token().unwrap().kind.syntax_kind())
            .collect::<Vec<_>>();
        assert_eq!(
            actual_token_kinds,
            vec![
                SyntaxKind::TokenIdentifier,
                SyntaxKind::TokenReservedWord,
                SyntaxKind::TokenReservedSymbol,
                SyntaxKind::TokenNumeral,
                SyntaxKind::TokenLexemeRun,
                SyntaxKind::TokenUserSymbol,
                SyntaxKind::TokenStringLiteral,
                SyntaxKind::TokenErrorRecovery,
                SyntaxKind::TokenUnknown,
                SyntaxKind::TokenErrorRecovery,
                SyntaxKind::TokenReservedSymbol,
                SyntaxKind::TokenIdentifier,
                SyntaxKind::TokenReservedSymbol,
                SyntaxKind::TokenIdentifier,
                SyntaxKind::TokenIdentifier,
                SyntaxKind::TokenReservedSymbol,
                SyntaxKind::TokenIdentifier,
                SyntaxKind::TokenIdentifier,
                SyntaxKind::TokenReservedSymbol,
                SyntaxKind::TokenUserSymbol,
                SyntaxKind::TokenReservedWord,
                SyntaxKind::TokenReservedSymbol,
                SyntaxKind::TokenReservedWord,
                SyntaxKind::TokenReservedSymbol,
                SyntaxKind::TokenIdentifier,
                SyntaxKind::TokenReservedSymbol,
                SyntaxKind::TokenIdentifier,
                SyntaxKind::TokenReservedSymbol,
                SyntaxKind::TokenReservedSymbol,
                SyntaxKind::TokenIdentifier,
                SyntaxKind::TokenReservedWord,
                SyntaxKind::TokenIdentifier,
                SyntaxKind::TokenReservedSymbol,
                SyntaxKind::TokenReservedWord,
                SyntaxKind::TokenIdentifier,
                SyntaxKind::TokenReservedSymbol,
                SyntaxKind::TokenReservedWord,
                SyntaxKind::TokenReservedWord,
                SyntaxKind::TokenReservedSymbol,
            ]
        );
        for (index, token_view) in ast.token_views().take(token_kinds.len()).enumerate() {
            assert_eq!(token_view.id(), sid(token_ids[index]));
            assert_eq!(token_view.syntax_kind(), SyntaxKind::Token);
            assert_eq!(token_view.range(), range(source_id, index, index + 1));
            assert_eq!(
                token_view.as_token().unwrap().text.as_ref(),
                format!("t{index}")
            );
            assert!(!token_view.is_recovered());
            assert!(token_view.children().is_empty());
            assert!(token_view.as_infix_expression().is_none());
            assert!(token_view.as_recovery().is_none());
        }
    }

    #[test]
    fn surface_node_raw_kinds_round_trip_through_rowan_boundary() {
        let ast = current_vocabulary_snapshot_ast(source_id(21));
        let rowan_kinds = ast
            .rowan_root()
            .descendants_with_tokens()
            .map(|element| element.kind())
            .collect::<Vec<_>>();

        for kind in [
            SyntaxKind::CompilationUnit,
            SyntaxKind::ItemList,
            SyntaxKind::PlaceholderItem,
            SyntaxKind::ImportItem,
            SyntaxKind::ImportAliasDecl,
            SyntaxKind::ModuleBranchImport,
            SyntaxKind::ExportItem,
            SyntaxKind::VisibilityMarker,
            SyntaxKind::VisibleItem,
            SyntaxKind::ReserveItem,
            SyntaxKind::ReserveSegment,
            SyntaxKind::TypeExpression,
            SyntaxKind::AttributeChain,
            SyntaxKind::AttributeRef,
            SyntaxKind::ParameterPrefix,
            SyntaxKind::TypeHead,
            SyntaxKind::TypeArguments,
            SyntaxKind::TermPlaceholder,
            SyntaxKind::TermExpression,
            SyntaxKind::TermReference,
            SyntaxKind::NumeralTerm,
            SyntaxKind::ItTerm,
            SyntaxKind::ParenthesizedTerm,
            SyntaxKind::ChoiceTerm,
            SyntaxKind::ApplicationTerm,
            SyntaxKind::StructureConstructor,
            SyntaxKind::FieldArgument,
            SyntaxKind::SetEnumeration,
            SyntaxKind::ModulePath,
            SyntaxKind::NamespacePath,
            SyntaxKind::QualifiedSymbol,
            SyntaxKind::PathSegment,
            SyntaxKind::RelativePrefix,
        ] {
            assert_eq!(SyntaxKind::from_raw(kind as u16), kind);
            assert!(kind.is_node_kind());
            assert!(!kind.is_token_kind());
            assert!(
                rowan_kinds.contains(&kind),
                "rowan tree should emit {kind:?} for current structural nodes"
            );
        }
    }

    #[test]
    fn task8_typed_accessors_cover_type_expression_nodes() {
        let source_id = source_id(23);
        let mut builder = SurfaceAstBuilder::new(source_id);
        let reserve = builder.add_token(
            SurfaceTokenKind::ReservedWord,
            "reserve",
            range(source_id, 0, 7),
        );
        let identifier =
            builder.add_token(SurfaceTokenKind::Identifier, "x", range(source_id, 8, 9));
        let for_keyword = builder.add_token(
            SurfaceTokenKind::ReservedWord,
            "for",
            range(source_id, 10, 13),
        );
        let non = builder.add_token(
            SurfaceTokenKind::ReservedWord,
            "non",
            range(source_id, 14, 17),
        );
        let n = builder.add_token(SurfaceTokenKind::Identifier, "n", range(source_id, 18, 19));
        let hyphen = builder.add_token(
            SurfaceTokenKind::ReservedSymbol,
            "-",
            range(source_id, 19, 20),
        );
        let empty = builder.add_token(
            SurfaceTokenKind::UserSymbol,
            "empty",
            range(source_id, 20, 25),
        );
        let type_symbol =
            builder.add_token(SurfaceTokenKind::UserSymbol, "T", range(source_id, 26, 27));
        let of = builder.add_token(
            SurfaceTokenKind::ReservedWord,
            "of",
            range(source_id, 28, 30),
        );
        let term = builder.add_token(SurfaceTokenKind::Identifier, "a", range(source_id, 31, 32));
        let semicolon = builder.add_token(
            SurfaceTokenKind::ReservedSymbol,
            ";",
            range(source_id, 32, 33),
        );

        let prefix = builder.add_node(
            SurfaceNodeKind::ParameterPrefix,
            range(source_id, 18, 20),
            vec![n, hyphen],
        );
        let empty_segment = builder.add_node(
            SurfaceNodeKind::PathSegment,
            range(source_id, 20, 25),
            vec![empty],
        );
        let empty_symbol = builder.add_node(
            SurfaceNodeKind::QualifiedSymbol,
            range(source_id, 20, 25),
            vec![empty_segment],
        );
        let attribute = builder.add_node(
            SurfaceNodeKind::AttributeRef,
            range(source_id, 14, 25),
            vec![non, prefix, empty_symbol],
        );
        let attribute_chain = builder.add_node(
            SurfaceNodeKind::AttributeChain,
            range(source_id, 14, 25),
            vec![attribute],
        );
        let type_segment = builder.add_node(
            SurfaceNodeKind::PathSegment,
            range(source_id, 26, 27),
            vec![type_symbol],
        );
        let type_symbol_node = builder.add_node(
            SurfaceNodeKind::QualifiedSymbol,
            range(source_id, 26, 27),
            vec![type_segment],
        );
        let term_placeholder = builder.add_node(
            SurfaceNodeKind::TermPlaceholder,
            range(source_id, 31, 32),
            vec![term],
        );
        let type_arguments = builder.add_node(
            SurfaceNodeKind::TypeArguments,
            range(source_id, 28, 32),
            vec![of, term_placeholder],
        );
        let type_head = builder.add_node(
            SurfaceNodeKind::TypeHead,
            range(source_id, 26, 32),
            vec![type_symbol_node, type_arguments],
        );
        let type_expression = builder.add_node(
            SurfaceNodeKind::TypeExpression,
            range(source_id, 14, 32),
            vec![attribute_chain, type_head],
        );
        let reserve_segment = builder.add_node(
            SurfaceNodeKind::ReserveSegment,
            range(source_id, 8, 32),
            vec![identifier, for_keyword, type_expression],
        );
        let reserve_item = builder.add_node(
            SurfaceNodeKind::ReserveItem,
            range(source_id, 0, 33),
            vec![reserve, reserve_segment, semicolon],
        );
        let root = builder.add_node(
            SurfaceNodeKind::Root,
            range(source_id, 0, 33),
            vec![
                reserve,
                identifier,
                for_keyword,
                non,
                n,
                hyphen,
                empty,
                type_symbol,
                of,
                term,
                semicolon,
                reserve_item,
            ],
        );
        let ast = builder.finish(Some(root), None);

        assert!(
            ast.node_view(sid(reserve_item))
                .unwrap()
                .as_reserve_item()
                .is_some()
        );
        assert!(
            ast.node_view(sid(reserve_segment))
                .unwrap()
                .as_reserve_segment()
                .is_some()
        );
        assert!(
            ast.node_view(sid(type_expression))
                .unwrap()
                .as_type_expression()
                .is_some()
        );
        assert!(
            ast.node_view(sid(attribute_chain))
                .unwrap()
                .as_attribute_chain()
                .is_some()
        );
        assert!(
            ast.node_view(sid(attribute))
                .unwrap()
                .as_attribute_ref()
                .is_some()
        );
        assert!(
            ast.node_view(sid(prefix))
                .unwrap()
                .as_parameter_prefix()
                .is_some()
        );
        assert!(
            ast.node_view(sid(type_head))
                .unwrap()
                .as_type_head()
                .is_some()
        );
        assert!(
            ast.node_view(sid(type_arguments))
                .unwrap()
                .as_type_arguments()
                .is_some()
        );
        assert!(
            ast.node_view(sid(term_placeholder))
                .unwrap()
                .as_term_placeholder()
                .is_some()
        );
    }

    #[test]
    fn task9_typed_accessors_cover_primary_term_nodes() {
        let source_id = source_id(24);
        let mut builder = SurfaceAstBuilder::new(source_id);
        let identifier =
            builder.add_token(SurfaceTokenKind::Identifier, "x", range(source_id, 0, 1));
        let numeral = builder.add_token(SurfaceTokenKind::Numeral, "42", range(source_id, 2, 4));
        let it = builder.add_token(SurfaceTokenKind::ReservedWord, "it", range(source_id, 5, 7));
        let open = builder.add_token(
            SurfaceTokenKind::ReservedSymbol,
            "(",
            range(source_id, 8, 9),
        );
        let paren_identifier =
            builder.add_token(SurfaceTokenKind::Identifier, "p", range(source_id, 9, 10));
        let close = builder.add_token(
            SurfaceTokenKind::ReservedSymbol,
            ")",
            range(source_id, 10, 11),
        );
        let the = builder.add_token(
            SurfaceTokenKind::ReservedWord,
            "the",
            range(source_id, 12, 15),
        );
        let set = builder.add_token(
            SurfaceTokenKind::ReservedWord,
            "set",
            range(source_id, 16, 19),
        );
        let function =
            builder.add_token(SurfaceTokenKind::UserSymbol, "F", range(source_id, 20, 21));
        let app_open = builder.add_token(
            SurfaceTokenKind::ReservedSymbol,
            "(",
            range(source_id, 21, 22),
        );
        let app_close = builder.add_token(
            SurfaceTokenKind::ReservedSymbol,
            ")",
            range(source_id, 22, 23),
        );
        let structure =
            builder.add_token(SurfaceTokenKind::UserSymbol, "S", range(source_id, 24, 25));
        let struct_open = builder.add_token(
            SurfaceTokenKind::ReservedSymbol,
            "(",
            range(source_id, 25, 26),
        );
        let field = builder.add_token(SurfaceTokenKind::Identifier, "x", range(source_id, 26, 27));
        let colon = builder.add_token(
            SurfaceTokenKind::ReservedSymbol,
            ":",
            range(source_id, 27, 28),
        );
        let value = builder.add_token(SurfaceTokenKind::Identifier, "y", range(source_id, 28, 29));
        let struct_close = builder.add_token(
            SurfaceTokenKind::ReservedSymbol,
            ")",
            range(source_id, 29, 30),
        );
        let set_open = builder.add_token(
            SurfaceTokenKind::ReservedSymbol,
            "{",
            range(source_id, 31, 32),
        );
        let set_close = builder.add_token(
            SurfaceTokenKind::ReservedSymbol,
            "}",
            range(source_id, 32, 33),
        );

        let term_reference = builder.add_node(
            SurfaceNodeKind::TermReference,
            range(source_id, 0, 1),
            vec![identifier],
        );
        let term_expression = builder.add_node(
            SurfaceNodeKind::TermExpression,
            range(source_id, 0, 1),
            vec![term_reference],
        );
        let numeral_term = builder.add_node(
            SurfaceNodeKind::NumeralTerm,
            range(source_id, 2, 4),
            vec![numeral],
        );
        let it_term = builder.add_node(SurfaceNodeKind::ItTerm, range(source_id, 5, 7), vec![it]);
        let paren_reference = builder.add_node(
            SurfaceNodeKind::TermReference,
            range(source_id, 9, 10),
            vec![paren_identifier],
        );
        let paren_expression = builder.add_node(
            SurfaceNodeKind::TermExpression,
            range(source_id, 9, 10),
            vec![paren_reference],
        );
        let paren_term = builder.add_node(
            SurfaceNodeKind::ParenthesizedTerm,
            range(source_id, 8, 11),
            vec![open, paren_expression, close],
        );
        let type_head = builder.add_node(
            SurfaceNodeKind::TypeHead,
            range(source_id, 16, 19),
            vec![set],
        );
        let type_expression = builder.add_node(
            SurfaceNodeKind::TypeExpression,
            range(source_id, 16, 19),
            vec![type_head],
        );
        let choice_term = builder.add_node(
            SurfaceNodeKind::ChoiceTerm,
            range(source_id, 12, 19),
            vec![the, type_expression],
        );
        let function_segment = builder.add_node(
            SurfaceNodeKind::PathSegment,
            range(source_id, 20, 21),
            vec![function],
        );
        let function_symbol = builder.add_node(
            SurfaceNodeKind::QualifiedSymbol,
            range(source_id, 20, 21),
            vec![function_segment],
        );
        let function_reference = builder.add_node(
            SurfaceNodeKind::TermReference,
            range(source_id, 20, 21),
            vec![function_symbol],
        );
        let application = builder.add_node(
            SurfaceNodeKind::ApplicationTerm,
            range(source_id, 20, 23),
            vec![function_reference, app_open, app_close],
        );
        let structure_segment = builder.add_node(
            SurfaceNodeKind::PathSegment,
            range(source_id, 24, 25),
            vec![structure],
        );
        let structure_symbol = builder.add_node(
            SurfaceNodeKind::QualifiedSymbol,
            range(source_id, 24, 25),
            vec![structure_segment],
        );
        let value_reference = builder.add_node(
            SurfaceNodeKind::TermReference,
            range(source_id, 28, 29),
            vec![value],
        );
        let value_expression = builder.add_node(
            SurfaceNodeKind::TermExpression,
            range(source_id, 28, 29),
            vec![value_reference],
        );
        let field_argument = builder.add_node(
            SurfaceNodeKind::FieldArgument,
            range(source_id, 26, 29),
            vec![field, colon, value_expression],
        );
        let structure_constructor = builder.add_node(
            SurfaceNodeKind::StructureConstructor,
            range(source_id, 24, 30),
            vec![structure_symbol, struct_open, field_argument, struct_close],
        );
        let set_enumeration = builder.add_node(
            SurfaceNodeKind::SetEnumeration,
            range(source_id, 31, 33),
            vec![set_open, set_close],
        );
        let root = builder.add_node(
            SurfaceNodeKind::Root,
            range(source_id, 0, 33),
            vec![
                identifier,
                numeral,
                it,
                open,
                paren_identifier,
                close,
                the,
                set,
                function,
                app_open,
                app_close,
                structure,
                struct_open,
                field,
                colon,
                value,
                struct_close,
                set_open,
                set_close,
                term_expression,
                numeral_term,
                it_term,
                paren_term,
                choice_term,
                application,
                structure_constructor,
                set_enumeration,
            ],
        );
        let ast = builder.finish(Some(root), None);

        assert!(
            ast.node_view(sid(term_expression))
                .unwrap()
                .as_term_expression()
                .is_some()
        );
        assert!(
            ast.node_view(sid(term_reference))
                .unwrap()
                .as_term_reference()
                .is_some()
        );
        assert!(
            ast.node_view(sid(numeral_term))
                .unwrap()
                .as_numeral_term()
                .is_some()
        );
        assert!(ast.node_view(sid(it_term)).unwrap().as_it_term().is_some());
        assert!(
            ast.node_view(sid(paren_term))
                .unwrap()
                .as_parenthesized_term()
                .is_some()
        );
        assert!(
            ast.node_view(sid(choice_term))
                .unwrap()
                .as_choice_term()
                .is_some()
        );
        assert!(
            ast.node_view(sid(application))
                .unwrap()
                .as_application_term()
                .is_some()
        );
        assert!(
            ast.node_view(sid(structure_constructor))
                .unwrap()
                .as_structure_constructor()
                .is_some()
        );
        assert!(
            ast.node_view(sid(field_argument))
                .unwrap()
                .as_field_argument()
                .is_some()
        );
        assert!(
            ast.node_view(sid(set_enumeration))
                .unwrap()
                .as_set_enumeration()
                .is_some()
        );
    }

    #[test]
    fn parent_ranges_contain_child_ranges_except_recovery_attachments() {
        let source_id = source_id(3);
        let ast = expression_ast(source_id);
        let expression = ast.expression_root().unwrap();
        let root = ast.root().unwrap();

        assert_eq!(ast.range_contains_child_ranges(expression), Some(true));
        assert_eq!(ast.range_contains_child_ranges(root), Some(true));

        let mut builder = SurfaceAstBuilder::new(source_id);
        let opener = builder.add_token(
            SurfaceTokenKind::ReservedWord,
            "definition",
            range(source_id, 0, 10),
        );
        let recovery = builder.add_recovery(
            SyntaxRecoveryKind::MissingEnd,
            range(source_id, 10, 10),
            vec![opener],
        );
        let root = builder.add_node(
            SurfaceNodeKind::Root,
            range(source_id, 0, 10),
            vec![opener, recovery],
        );
        let recovered_ast = builder.finish(Some(root), None);

        for node_id in [opener, root].map(sid) {
            assert_eq!(
                recovered_ast.range_contains_child_ranges(node_id),
                Some(true),
                "ordinary non-recovery node {node_id:?} should contain all child ranges"
            );
        }
        assert_eq!(
            recovered_ast.range_contains_child_ranges(sid(recovery)),
            Some(false),
            "missing-end recovery attaches the opener as context even though the zero-width insertion range does not contain it"
        );

        let mut missing_string_builder = SurfaceAstBuilder::new(source_id);
        let missing_string = missing_string_builder.add_recovery(
            SyntaxRecoveryKind::MissingStringLiteral,
            range(source_id, 12, 12),
            Vec::new(),
        );
        let root = missing_string_builder.add_node(
            SurfaceNodeKind::Root,
            range(source_id, 12, 12),
            vec![missing_string],
        );
        let missing_string_ast = missing_string_builder.finish(Some(root), None);
        assert_eq!(
            missing_string_ast.range_contains_child_ranges(sid(missing_string)),
            Some(true),
            "zero-width missing string recovery without context children still satisfies containment"
        );
        assert_eq!(
            missing_string_ast.range_contains_child_ranges(sid(root)),
            Some(true)
        );
    }

    #[test]
    fn recovery_kinds_are_constructible_with_documented_ranges() {
        let source_id = source_id(4);

        for fixture in recovery_fixtures(source_id) {
            let mut builder = SurfaceAstBuilder::new(source_id);
            let context = fixture.has_context_child.then(|| {
                builder.add_token(
                    SurfaceTokenKind::ReservedWord,
                    "context",
                    range(source_id, 0, 7),
                )
            });
            let recovery_children = context.into_iter().collect::<Vec<_>>();
            let recovery =
                builder.add_recovery(fixture.kind, fixture.range, recovery_children.clone());
            let root_children = recovery_children
                .iter()
                .copied()
                .chain([recovery])
                .collect::<Vec<_>>();
            let root = builder.add_node(
                SurfaceNodeKind::Root,
                range(source_id, 0, 40),
                root_children,
            );
            let ast = builder.finish(Some(root), None);
            let recovery_view = ast.node_view(sid(recovery)).unwrap();

            assert_eq!(recovery_view.as_recovery(), Some(fixture.kind));
            assert_eq!(recovery_view.syntax_kind(), SyntaxKind::ErrorRecovery);
            assert_eq!(recovery_view.range(), fixture.range);
            assert!(recovery_view.is_recovered());
            assert_eq!(
                recovery_view.children(),
                &recovery_children
                    .iter()
                    .copied()
                    .map(sid)
                    .collect::<Vec<_>>()
            );
            assert_eq!(
                ast.range_contains_child_ranges(sid(recovery)),
                Some(!fixture.has_context_child),
                "{:?} should follow its documented context-child range rule",
                fixture.kind
            );

            let snapshot_line = format!(
                "ErrorRecovery kind={} range={}..{} recovered=true",
                super::recovery_snapshot_name(fixture.kind),
                fixture.range.start,
                fixture.range.end
            );
            assert!(
                ast.snapshot_text().contains(&snapshot_line),
                "snapshot should render {:?} with its distinct recovery kind name",
                fixture.kind
            );
        }
    }

    #[test]
    fn repeated_construction_produces_deterministic_green_tree_and_views() {
        let source_id = source_id(5);
        let first = expression_ast(source_id);
        let second = expression_ast(source_id);

        assert_eq!(first.green_node(), second.green_node());
        assert_eq!(first.nodes(), second.nodes());
        assert_eq!(first.token_nodes(), second.token_nodes());
        assert_eq!(first.expression_root(), second.expression_root());
    }

    #[test]
    fn repeated_snapshot_rendering_is_byte_identical() {
        let ast = expression_ast(source_id(6));

        assert_eq!(ast.snapshot_text(), ast.snapshot_text());
        assert_eq!(
            ast.snapshot_text(),
            expression_ast(source_id(6)).snapshot_text()
        );
    }

    #[test]
    fn snapshot_rendering_matches_current_vocabulary_baseline() {
        const EXPECTED: &str = include_str!(
            "../../../tests/snapshots/mizar_syntax_surface_ast_current_vocabulary.snap"
        );
        let ast = current_vocabulary_snapshot_ast(source_id(7));
        let actual = ast.snapshot_text();

        assert_eq!(actual, EXPECTED);
        assert!(actual.contains("ErrorRecovery kind=MissingEnd range=89..89 recovered=true"));
        assert!(actual.contains("text=\"line\\nvalue\""));
        assert!(
            !actual.contains("SourceId"),
            "snapshot text must not expose opaque source-id debug output"
        );
    }

    #[test]
    fn snapshot_payload_names_cover_current_variants() {
        let source_id = source_id(8);

        for (associativity, expected) in [
            (SurfaceOperatorAssociativity::Left, "associativity=Left"),
            (SurfaceOperatorAssociativity::Right, "associativity=Right"),
            (
                SurfaceOperatorAssociativity::NonAssociative,
                "associativity=NonAssociative",
            ),
        ] {
            let ast = expression_ast_with_associativity(source_id, associativity);

            assert!(ast.snapshot_text().contains(expected));
        }

        for recovery_kind in all_recovery_kinds() {
            let expected = format!("kind={}", super::recovery_snapshot_name(recovery_kind));
            let ast = recovery_ast(source_id, recovery_kind);

            assert!(ast.snapshot_text().contains(&expected));
        }

        let mut recovery_snapshot_names = std::collections::BTreeSet::new();
        for recovery_kind in all_recovery_kinds() {
            assert!(
                recovery_snapshot_names.insert(super::recovery_snapshot_name(recovery_kind)),
                "recovery snapshot names must distinguish every SyntaxRecoveryKind"
            );
        }
    }

    #[test]
    fn recovery_snapshot_names_are_unique_and_fully_fixture_backed() {
        let source_id = source_id(9);
        let fixtures = recovery_fixtures(source_id);
        let fixture_kinds = fixtures
            .iter()
            .map(|fixture| fixture.kind)
            .collect::<Vec<_>>();
        let all_kinds = all_recovery_kinds();

        assert_eq!(fixture_kinds, all_kinds);

        let mut names = std::collections::BTreeSet::new();
        for recovery_kind in all_kinds {
            let name = super::recovery_snapshot_name(recovery_kind);

            assert!(!name.is_empty());
            assert!(
                names.insert(name),
                "recovery snapshot name {name:?} should be unique"
            );
        }
    }

    #[test]
    fn snapshot_rendering_includes_trivia_when_requested() {
        let source_id = source_id(10);
        let ast = expression_ast(source_id);
        let expression = ast.expression_root().unwrap();
        let token = ast.token_nodes()[0];
        let mut trivia = SurfaceTriviaBuilder::new(source_id);
        trivia.add_comment(CommentKind::SingleLine, range(source_id, 22, 31));
        trivia.add_doc_comment_attachment(
            range(source_id, 0, 8),
            TriviaAttachmentTarget::Node(TriviaNodeTarget::new(expression, range(source_id, 0, 6))),
            TriviaPlacement::Leading,
        );
        trivia.add_skipped_token_range(
            range(source_id, 32, 35),
            Some(TriviaAttachmentTarget::Token(TriviaNodeTarget::new(
                token,
                range(source_id, 0, 1),
            ))),
            SkippedTokenReason::UnexpectedToken,
        );
        trivia.add_whitespace_hint(
            WhitespaceHintKind::RequiresSeparation,
            range(source_id, 1, 2),
        );
        let ast = ast.with_trivia(trivia.finish());
        let actual = ast.snapshot_text_with_trivia();

        assert!(actual.starts_with(&ast.snapshot_text()));
        assert!(actual.contains("trivia:\n"));
        assert!(actual.contains("Comment kind=SingleLine range=22..31"));
        assert!(actual.contains("DocComment range=0..8 placement=Leading target=node:range:0..6"));
        assert!(
            actual.contains(
                "SkippedTokens reason=UnexpectedToken range=32..35 owner=token:range:0..1"
            )
        );
        assert!(actual.contains("WhitespaceHint kind=RequiresSeparation range=1..2"));
        assert!(
            !ast.snapshot_text().contains("trivia:"),
            "default snapshot rendering stays compatible with task-3 baselines"
        );
    }

    #[test]
    fn doc_comment_can_attach_to_following_placeholder_item_node() {
        let source_id = source_id(22);
        let mut builder = SurfaceAstBuilder::new(source_id);
        let theorem = builder.add_token(
            SurfaceTokenKind::ReservedWord,
            "theorem",
            range(source_id, 4, 11),
        );
        let semicolon = builder.add_token(
            SurfaceTokenKind::ReservedSymbol,
            ";",
            range(source_id, 11, 12),
        );
        let item = builder.add_node(
            SurfaceNodeKind::PlaceholderItem,
            range(source_id, 4, 12),
            vec![theorem, semicolon],
        );
        let item_list = builder.add_node(
            SurfaceNodeKind::ItemList,
            range(source_id, 4, 12),
            vec![item],
        );
        let compilation_unit = builder.add_node(
            SurfaceNodeKind::CompilationUnit,
            range(source_id, 4, 12),
            vec![item_list],
        );
        let root = builder.add_node(
            SurfaceNodeKind::Root,
            range(source_id, 4, 12),
            vec![theorem, semicolon, compilation_unit],
        );
        let ast = builder.finish(Some(root), None);
        let mut trivia = SurfaceTriviaBuilder::new(source_id);
        trivia.add_doc_comment_attachment(
            range(source_id, 0, 3),
            TriviaAttachmentTarget::Node(TriviaNodeTarget::new(sid(item), range(source_id, 4, 12))),
            TriviaPlacement::Leading,
        );
        let ast = ast.with_trivia(trivia.finish());

        assert!(ast.snapshot_text().contains("PlaceholderItem range=4..12"));
        assert!(
            ast.snapshot_text_with_trivia()
                .contains("DocComment range=0..3 placement=Leading target=node:range:4..12")
        );
    }

    #[test]
    fn trivia_snapshot_rendering_is_sorted_and_byte_identical() {
        let source_id = source_id(9);
        let first = expression_ast(source_id).with_trivia(scrambled_trivia(source_id).finish());
        let second = expression_ast(source_id).with_trivia(scrambled_trivia(source_id).finish());
        let expected = format!(
            "{}{}",
            expression_ast(source_id).snapshot_text(),
            concat!(
                "trivia:\n",
                "  Comment kind=MultiLine range=2..5\n",
                "  Comment kind=SingleLine range=30..39\n",
                "  DocComment range=6..8 placement=Leading target=detached:point:9\n",
                "  DocComment range=10..12 placement=Trailing target=detached:range:12..13\n",
                "  SkippedTokens reason=MalformedAnnotation range=40..44 owner=<none>\n",
                "  SkippedTokens reason=UnexpectedToken range=50..51 owner=detached:range:48..52\n",
                "  WhitespaceHint kind=LineBreakBefore range=14..15\n",
                "  WhitespaceHint kind=SyntheticBoundary range=60..61\n",
            )
        );

        assert_eq!(first.snapshot_text_with_trivia(), expected);
        assert_eq!(
            first.snapshot_text_with_trivia(),
            second.snapshot_text_with_trivia()
        );
    }

    #[test]
    fn trivia_snapshot_target_sorting_breaks_collisions_deterministically() {
        let source_id = source_id(10);
        let first = expression_ast(source_id)
            .with_trivia(colliding_target_trivia(source_id, false).finish());
        let second = expression_ast(source_id)
            .with_trivia(colliding_target_trivia(source_id, true).finish());
        let expected = format!(
            "{}{}",
            expression_ast(source_id).snapshot_text(),
            concat!(
                "trivia:\n",
                "  DocComment range=20..22 placement=Leading target=detached:point:30\n",
                "  DocComment range=20..22 placement=Leading target=detached:range:30..31\n",
            )
        );

        assert_eq!(first.snapshot_text_with_trivia(), expected);
        assert_eq!(
            first.snapshot_text_with_trivia(),
            second.snapshot_text_with_trivia()
        );
    }

    #[test]
    #[should_panic(expected = "SurfaceAst trivia node target must not refer to a token node")]
    fn ast_rejects_token_node_as_trivia_node_target() {
        let source_id = source_id(11);
        let ast = expression_ast(source_id);
        let token = ast.token_nodes()[0];
        let mut trivia = SurfaceTriviaBuilder::new(source_id);
        trivia.add_doc_comment_attachment(
            range(source_id, 0, 8),
            TriviaAttachmentTarget::Node(TriviaNodeTarget::new(token, range(source_id, 0, 1))),
            TriviaPlacement::Leading,
        );

        let _ = ast.with_trivia(trivia.finish());
    }

    #[test]
    #[should_panic(expected = "SurfaceAst trivia token target must refer to a token node")]
    fn ast_rejects_non_token_trivia_token_target() {
        let source_id = source_id(12);
        let ast = expression_ast(source_id);
        let expression = ast.expression_root().unwrap();
        let mut trivia = SurfaceTriviaBuilder::new(source_id);
        trivia.add_skipped_token_range(
            range(source_id, 32, 35),
            Some(TriviaAttachmentTarget::Token(TriviaNodeTarget::new(
                expression,
                range(source_id, 0, 6),
            ))),
            SkippedTokenReason::UnexpectedToken,
        );

        let _ = ast.with_trivia(trivia.finish());
    }

    #[test]
    #[should_panic(expected = "SurfaceAst trivia node target range must match the AST node range")]
    fn ast_rejects_trivia_target_with_mismatched_range() {
        let source_id = source_id(13);
        let ast = expression_ast(source_id);
        let expression = ast.expression_root().unwrap();
        let mut trivia = SurfaceTriviaBuilder::new(source_id);
        trivia.add_doc_comment_attachment(
            range(source_id, 0, 8),
            TriviaAttachmentTarget::Node(TriviaNodeTarget::new(expression, range(source_id, 0, 5))),
            TriviaPlacement::Leading,
        );

        let _ = ast.with_trivia(trivia.finish());
    }

    #[test]
    #[should_panic(expected = "SurfaceAst trivia must belong to the AST source")]
    fn ast_rejects_trivia_from_another_source() {
        let ids = InMemorySessionIdAllocator::new();
        let ast_source_id = ids.next_source_id(snapshot_id(14)).unwrap();
        let trivia_source_id = ids.next_source_id(snapshot_id(15)).unwrap();
        let ast = expression_ast(ast_source_id);
        let mut trivia = SurfaceTriviaBuilder::new(trivia_source_id);
        trivia.add_doc_comment_attachment(
            range(trivia_source_id, 0, 8),
            TriviaAttachmentTarget::Detached(SourceAnchor::Point {
                source_id: trivia_source_id,
                offset: 8,
            }),
            TriviaPlacement::Leading,
        );

        let _ = ast.with_trivia(trivia.finish());
    }

    #[test]
    #[should_panic(expected = "must have been created by this builder")]
    fn builder_rejects_child_ids_not_created_by_this_builder() {
        let source_id = source_id(16);
        let other_id = {
            let mut other = SurfaceAstBuilder::new(source_id);
            other.add_token(SurfaceTokenKind::Identifier, "x", range(source_id, 0, 1))
        };
        let mut builder = SurfaceAstBuilder::new(source_id);

        builder.add_node(
            SurfaceNodeKind::Root,
            range(source_id, 0, 1),
            vec![other_id],
        );
    }

    #[test]
    #[should_panic(expected = "cannot be shared by multiple non-root parents")]
    fn builder_rejects_token_sharing_between_multiple_structural_parents() {
        let source_id = source_id(17);
        let mut builder = SurfaceAstBuilder::new(source_id);
        let token = builder.add_token(SurfaceTokenKind::Identifier, "x", range(source_id, 0, 1));
        let left_expression = builder.add_node(
            SurfaceNodeKind::InfixExpression(SurfaceInfixOperator {
                spelling: "left".into(),
                precedence: 1,
                associativity: SurfaceOperatorAssociativity::Left,
            }),
            range(source_id, 0, 1),
            vec![token],
        );
        let right_expression = builder.add_node(
            SurfaceNodeKind::InfixExpression(SurfaceInfixOperator {
                spelling: "right".into(),
                precedence: 1,
                associativity: SurfaceOperatorAssociativity::Left,
            }),
            range(source_id, 0, 1),
            vec![token],
        );
        let root = builder.add_node(
            SurfaceNodeKind::Root,
            range(source_id, 0, 1),
            vec![token, left_expression, right_expression],
        );

        let _ = builder.finish(Some(root), Some(left_expression));
    }

    fn scrambled_trivia(source_id: SourceId) -> SurfaceTriviaBuilder {
        let mut trivia = SurfaceTriviaBuilder::new(source_id);
        trivia.add_whitespace_hint(
            WhitespaceHintKind::SyntheticBoundary,
            range(source_id, 60, 61),
        );
        trivia.add_comment(CommentKind::SingleLine, range(source_id, 30, 39));
        trivia.add_skipped_token_range(
            range(source_id, 50, 51),
            Some(TriviaAttachmentTarget::Detached(SourceAnchor::Range(
                range(source_id, 48, 52),
            ))),
            SkippedTokenReason::UnexpectedToken,
        );
        trivia.add_doc_comment_attachment(
            range(source_id, 10, 12),
            TriviaAttachmentTarget::Detached(SourceAnchor::Range(range(source_id, 12, 13))),
            TriviaPlacement::Trailing,
        );
        trivia.add_comment(CommentKind::MultiLine, range(source_id, 2, 5));
        trivia.add_whitespace_hint(
            WhitespaceHintKind::LineBreakBefore,
            range(source_id, 14, 15),
        );
        trivia.add_skipped_token_range(
            range(source_id, 40, 44),
            None,
            SkippedTokenReason::MalformedAnnotation,
        );
        trivia.add_doc_comment_attachment(
            range(source_id, 6, 8),
            TriviaAttachmentTarget::Detached(SourceAnchor::Point {
                source_id,
                offset: 9,
            }),
            TriviaPlacement::Leading,
        );
        trivia
    }

    fn colliding_target_trivia(source_id: SourceId, reverse: bool) -> SurfaceTriviaBuilder {
        let mut trivia = SurfaceTriviaBuilder::new(source_id);
        let point = (
            range(source_id, 20, 22),
            TriviaAttachmentTarget::Detached(SourceAnchor::Point {
                source_id,
                offset: 30,
            }),
        );
        let range_target = (
            range(source_id, 20, 22),
            TriviaAttachmentTarget::Detached(SourceAnchor::Range(range(source_id, 30, 31))),
        );
        let entries = if reverse {
            [range_target, point]
        } else {
            [point, range_target]
        };
        for (range, target) in entries {
            trivia.add_doc_comment_attachment(range, target, TriviaPlacement::Leading);
        }
        trivia
    }

    fn expression_ast(source_id: SourceId) -> crate::SurfaceAst {
        let mut builder = SurfaceAstBuilder::new(source_id);
        let left = builder.add_token(SurfaceTokenKind::Identifier, "a", range(source_id, 0, 1));
        let operator =
            builder.add_token(SurfaceTokenKind::UserSymbol, "++", range(source_id, 2, 4));
        let right = builder.add_token(SurfaceTokenKind::Identifier, "b", range(source_id, 5, 6));
        let expression = builder.add_node(
            SurfaceNodeKind::InfixExpression(SurfaceInfixOperator {
                spelling: "++".into(),
                precedence: 10,
                associativity: SurfaceOperatorAssociativity::Left,
            }),
            range(source_id, 0, 6),
            vec![left, operator, right],
        );
        let root = builder.add_node(
            SurfaceNodeKind::Root,
            range(source_id, 0, 6),
            vec![left, operator, right, expression],
        );
        builder.finish(Some(root), Some(expression))
    }

    fn expression_ast_with_associativity(
        source_id: SourceId,
        associativity: SurfaceOperatorAssociativity,
    ) -> crate::SurfaceAst {
        let mut builder = SurfaceAstBuilder::new(source_id);
        let left = builder.add_token(SurfaceTokenKind::Identifier, "a", range(source_id, 0, 1));
        let operator =
            builder.add_token(SurfaceTokenKind::UserSymbol, "++", range(source_id, 2, 4));
        let right = builder.add_token(SurfaceTokenKind::Identifier, "b", range(source_id, 5, 6));
        let expression = builder.add_node(
            SurfaceNodeKind::InfixExpression(SurfaceInfixOperator {
                spelling: "++".into(),
                precedence: 10,
                associativity,
            }),
            range(source_id, 0, 6),
            vec![left, operator, right],
        );
        let root = builder.add_node(
            SurfaceNodeKind::Root,
            range(source_id, 0, 6),
            vec![left, operator, right, expression],
        );
        builder.finish(Some(root), Some(expression))
    }

    fn recovery_ast(source_id: SourceId, recovery_kind: SyntaxRecoveryKind) -> crate::SurfaceAst {
        let mut builder = SurfaceAstBuilder::new(source_id);
        let recovery = builder.add_recovery(recovery_kind, range(source_id, 0, 0), Vec::new());
        let root = builder.add_node(
            SurfaceNodeKind::Root,
            range(source_id, 0, 0),
            vec![recovery],
        );
        builder.finish(Some(root), None)
    }

    #[derive(Clone, Copy)]
    struct RecoveryFixture {
        kind: SyntaxRecoveryKind,
        range: SourceRange,
        has_context_child: bool,
    }

    fn recovery_fixtures(source_id: SourceId) -> Vec<RecoveryFixture> {
        vec![
            RecoveryFixture {
                kind: SyntaxRecoveryKind::ErrorToken,
                range: range(source_id, 8, 11),
                has_context_child: false,
            },
            RecoveryFixture {
                kind: SyntaxRecoveryKind::MissingEnd,
                range: range(source_id, 12, 12),
                has_context_child: true,
            },
            RecoveryFixture {
                kind: SyntaxRecoveryKind::MissingStringLiteral,
                range: range(source_id, 13, 13),
                has_context_child: false,
            },
            RecoveryFixture {
                kind: SyntaxRecoveryKind::MissingItem,
                range: range(source_id, 14, 14),
                has_context_child: true,
            },
            RecoveryFixture {
                kind: SyntaxRecoveryKind::MissingTypeExpression,
                range: range(source_id, 15, 15),
                has_context_child: true,
            },
            RecoveryFixture {
                kind: SyntaxRecoveryKind::MissingTerm,
                range: range(source_id, 16, 16),
                has_context_child: true,
            },
            RecoveryFixture {
                kind: SyntaxRecoveryKind::MissingFormula,
                range: range(source_id, 17, 17),
                has_context_child: true,
            },
            RecoveryFixture {
                kind: SyntaxRecoveryKind::MissingStatement,
                range: range(source_id, 18, 18),
                has_context_child: true,
            },
            RecoveryFixture {
                kind: SyntaxRecoveryKind::MissingProofStep,
                range: range(source_id, 19, 19),
                has_context_child: true,
            },
            RecoveryFixture {
                kind: SyntaxRecoveryKind::MissingAnnotationArgument,
                range: range(source_id, 20, 20),
                has_context_child: true,
            },
            RecoveryFixture {
                kind: SyntaxRecoveryKind::SkippedToken,
                range: range(source_id, 21, 24),
                has_context_child: false,
            },
            RecoveryFixture {
                kind: SyntaxRecoveryKind::UnmatchedOpeningDelimiter,
                range: range(source_id, 25, 25),
                has_context_child: true,
            },
            RecoveryFixture {
                kind: SyntaxRecoveryKind::UnmatchedClosingDelimiter,
                range: range(source_id, 26, 27),
                has_context_child: false,
            },
            RecoveryFixture {
                kind: SyntaxRecoveryKind::MalformedAnnotation,
                range: range(source_id, 28, 35),
                has_context_child: false,
            },
        ]
    }

    fn all_recovery_kinds() -> [SyntaxRecoveryKind; 14] {
        [
            SyntaxRecoveryKind::ErrorToken,
            SyntaxRecoveryKind::MissingEnd,
            SyntaxRecoveryKind::MissingStringLiteral,
            SyntaxRecoveryKind::MissingItem,
            SyntaxRecoveryKind::MissingTypeExpression,
            SyntaxRecoveryKind::MissingTerm,
            SyntaxRecoveryKind::MissingFormula,
            SyntaxRecoveryKind::MissingStatement,
            SyntaxRecoveryKind::MissingProofStep,
            SyntaxRecoveryKind::MissingAnnotationArgument,
            SyntaxRecoveryKind::SkippedToken,
            SyntaxRecoveryKind::UnmatchedOpeningDelimiter,
            SyntaxRecoveryKind::UnmatchedClosingDelimiter,
            SyntaxRecoveryKind::MalformedAnnotation,
        ]
    }

    fn current_vocabulary_snapshot_ast(source_id: SourceId) -> crate::SurfaceAst {
        let mut builder = SurfaceAstBuilder::new(source_id);
        let identifier =
            builder.add_token(SurfaceTokenKind::Identifier, "id", range(source_id, 0, 2));
        let reserved_word = builder.add_token(
            SurfaceTokenKind::ReservedWord,
            "theorem",
            range(source_id, 3, 10),
        );
        let reserved_symbol = builder.add_token(
            SurfaceTokenKind::ReservedSymbol,
            ";",
            range(source_id, 10, 11),
        );
        let numeral = builder.add_token(SurfaceTokenKind::Numeral, "42", range(source_id, 12, 14));
        let lexeme_run =
            builder.add_token(SurfaceTokenKind::LexemeRun, "abc", range(source_id, 15, 18));
        let user_symbol =
            builder.add_token(SurfaceTokenKind::UserSymbol, "++", range(source_id, 19, 21));
        let string_literal = builder.add_token(
            SurfaceTokenKind::StringLiteral,
            "line\nvalue",
            range(source_id, 22, 32),
        );
        let error_token = builder.add_token(
            SurfaceTokenKind::ErrorRecovery,
            "<error>",
            range(source_id, 32, 39),
        );
        let unknown = builder.add_token(SurfaceTokenKind::Unknown, "?", range(source_id, 39, 40));
        let recovered_token = builder.add_recovered_token(
            SurfaceTokenKind::ErrorRecovery,
            "bad\ttext",
            range(source_id, 40, 48),
        );
        let module_prefix = builder.add_token(
            SurfaceTokenKind::ReservedSymbol,
            "..",
            range(source_id, 49, 51),
        );
        let module_std = builder.add_token(
            SurfaceTokenKind::Identifier,
            "std",
            range(source_id, 51, 54),
        );
        let module_dot = builder.add_token(
            SurfaceTokenKind::ReservedSymbol,
            ".",
            range(source_id, 54, 55),
        );
        let module_algebra = builder.add_token(
            SurfaceTokenKind::Identifier,
            "algebra",
            range(source_id, 55, 62),
        );
        let namespace_mml = builder.add_token(
            SurfaceTokenKind::Identifier,
            "mml",
            range(source_id, 63, 66),
        );
        let namespace_dot = builder.add_token(
            SurfaceTokenKind::ReservedSymbol,
            ".",
            range(source_id, 66, 67),
        );
        let namespace_nat = builder.add_token(
            SurfaceTokenKind::Identifier,
            "nat",
            range(source_id, 67, 70),
        );
        let qualified_top = builder.add_token(
            SurfaceTokenKind::Identifier,
            "top",
            range(source_id, 71, 74),
        );
        let qualified_dot = builder.add_token(
            SurfaceTokenKind::ReservedSymbol,
            ".",
            range(source_id, 74, 75),
        );
        let qualified_space = builder.add_token(
            SurfaceTokenKind::UserSymbol,
            "Space",
            range(source_id, 75, 80),
        );
        let item_theorem = builder.add_token(
            SurfaceTokenKind::ReservedWord,
            "theorem",
            range(source_id, 81, 88),
        );
        let item_semicolon = builder.add_token(
            SurfaceTokenKind::ReservedSymbol,
            ";",
            range(source_id, 88, 89),
        );
        let import_keyword = builder.add_token(
            SurfaceTokenKind::ReservedWord,
            "import",
            range(source_id, 90, 96),
        );
        let import_path_prefix = builder.add_token(
            SurfaceTokenKind::ReservedSymbol,
            ".",
            range(source_id, 97, 98),
        );
        let import_path_core = builder.add_token(
            SurfaceTokenKind::Identifier,
            "core",
            range(source_id, 98, 102),
        );
        let import_branch_opener = builder.add_token(
            SurfaceTokenKind::ReservedSymbol,
            ".{",
            range(source_id, 102, 104),
        );
        let import_branch_segment = builder.add_token(
            SurfaceTokenKind::Identifier,
            "linear",
            range(source_id, 104, 110),
        );
        let import_branch_close = builder.add_token(
            SurfaceTokenKind::ReservedSymbol,
            "}",
            range(source_id, 110, 111),
        );
        let import_comma = builder.add_token(
            SurfaceTokenKind::ReservedSymbol,
            ",",
            range(source_id, 111, 112),
        );
        let import_alias_path = builder.add_token(
            SurfaceTokenKind::Identifier,
            "algebra",
            range(source_id, 113, 120),
        );
        let import_alias_as = builder.add_token(
            SurfaceTokenKind::ReservedWord,
            "as",
            range(source_id, 121, 123),
        );
        let import_alias = builder.add_token(
            SurfaceTokenKind::Identifier,
            "A",
            range(source_id, 124, 125),
        );
        let import_semicolon = builder.add_token(
            SurfaceTokenKind::ReservedSymbol,
            ";",
            range(source_id, 125, 126),
        );
        let export_keyword = builder.add_token(
            SurfaceTokenKind::ReservedWord,
            "export",
            range(source_id, 127, 133),
        );
        let export_path_std = builder.add_token(
            SurfaceTokenKind::Identifier,
            "Std",
            range(source_id, 134, 137),
        );
        let export_semicolon = builder.add_token(
            SurfaceTokenKind::ReservedSymbol,
            ";",
            range(source_id, 137, 138),
        );
        let visibility_public = builder.add_token(
            SurfaceTokenKind::ReservedWord,
            "public",
            range(source_id, 139, 145),
        );
        let visible_theorem = builder.add_token(
            SurfaceTokenKind::ReservedWord,
            "theorem",
            range(source_id, 146, 153),
        );
        let visible_semicolon = builder.add_token(
            SurfaceTokenKind::ReservedSymbol,
            ";",
            range(source_id, 153, 154),
        );
        let reserve_keyword = builder.add_token(
            SurfaceTokenKind::ReservedWord,
            "reserve",
            range(source_id, 155, 162),
        );
        let reserve_x = builder.add_token(
            SurfaceTokenKind::Identifier,
            "x",
            range(source_id, 163, 164),
        );
        let reserve_for = builder.add_token(
            SurfaceTokenKind::ReservedWord,
            "for",
            range(source_id, 165, 168),
        );
        let reserve_non = builder.add_token(
            SurfaceTokenKind::ReservedWord,
            "non",
            range(source_id, 169, 172),
        );
        let reserve_empty = builder.add_token(
            SurfaceTokenKind::UserSymbol,
            "empty",
            range(source_id, 173, 178),
        );
        let reserve_type_symbol = builder.add_token(
            SurfaceTokenKind::UserSymbol,
            "T",
            range(source_id, 179, 180),
        );
        let reserve_arg_open = builder.add_token(
            SurfaceTokenKind::ReservedSymbol,
            "[",
            range(source_id, 181, 182),
        );
        let reserve_set = builder.add_token(
            SurfaceTokenKind::ReservedWord,
            "set",
            range(source_id, 182, 185),
        );
        let reserve_arg_comma = builder.add_token(
            SurfaceTokenKind::ReservedSymbol,
            ",",
            range(source_id, 185, 186),
        );
        let reserve_arg_identifier = builder.add_token(
            SurfaceTokenKind::Identifier,
            "V",
            range(source_id, 187, 188),
        );
        let reserve_qua = builder.add_token(
            SurfaceTokenKind::ReservedWord,
            "qua",
            range(source_id, 189, 192),
        );
        let reserve_radix = builder.add_token(
            SurfaceTokenKind::UserSymbol,
            "R",
            range(source_id, 193, 194),
        );
        let reserve_arg_close = builder.add_token(
            SurfaceTokenKind::ReservedSymbol,
            "]",
            range(source_id, 194, 195),
        );
        let reserve_semicolon = builder.add_token(
            SurfaceTokenKind::ReservedSymbol,
            ";",
            range(source_id, 195, 196),
        );
        let standalone_prefix_n = builder.add_token(
            SurfaceTokenKind::Identifier,
            "n",
            range(source_id, 197, 198),
        );
        let standalone_prefix_hyphen = builder.add_token(
            SurfaceTokenKind::ReservedSymbol,
            "-",
            range(source_id, 198, 199),
        );
        let term_reference_token = builder.add_token(
            SurfaceTokenKind::Identifier,
            "x",
            range(source_id, 200, 201),
        );
        let term_numeral_token =
            builder.add_token(SurfaceTokenKind::Numeral, "99", range(source_id, 202, 204));
        let term_it_token = builder.add_token(
            SurfaceTokenKind::ReservedWord,
            "it",
            range(source_id, 205, 207),
        );
        let term_paren_open = builder.add_token(
            SurfaceTokenKind::ReservedSymbol,
            "(",
            range(source_id, 208, 209),
        );
        let term_paren_identifier = builder.add_token(
            SurfaceTokenKind::Identifier,
            "p",
            range(source_id, 209, 210),
        );
        let term_paren_close = builder.add_token(
            SurfaceTokenKind::ReservedSymbol,
            ")",
            range(source_id, 210, 211),
        );
        let term_the = builder.add_token(
            SurfaceTokenKind::ReservedWord,
            "the",
            range(source_id, 212, 215),
        );
        let term_choice_set = builder.add_token(
            SurfaceTokenKind::ReservedWord,
            "set",
            range(source_id, 216, 219),
        );
        let term_apply_symbol = builder.add_token(
            SurfaceTokenKind::UserSymbol,
            "F",
            range(source_id, 220, 221),
        );
        let term_apply_open = builder.add_token(
            SurfaceTokenKind::ReservedSymbol,
            "(",
            range(source_id, 221, 222),
        );
        let term_apply_arg = builder.add_token(
            SurfaceTokenKind::Identifier,
            "a",
            range(source_id, 222, 223),
        );
        let term_apply_close = builder.add_token(
            SurfaceTokenKind::ReservedSymbol,
            ")",
            range(source_id, 223, 224),
        );
        let term_struct_symbol = builder.add_token(
            SurfaceTokenKind::UserSymbol,
            "S",
            range(source_id, 225, 226),
        );
        let term_struct_open = builder.add_token(
            SurfaceTokenKind::ReservedSymbol,
            "(",
            range(source_id, 226, 227),
        );
        let term_field_name = builder.add_token(
            SurfaceTokenKind::Identifier,
            "x",
            range(source_id, 227, 228),
        );
        let term_field_colon = builder.add_token(
            SurfaceTokenKind::ReservedSymbol,
            ":",
            range(source_id, 228, 229),
        );
        let term_field_value = builder.add_token(
            SurfaceTokenKind::Identifier,
            "y",
            range(source_id, 229, 230),
        );
        let term_struct_close = builder.add_token(
            SurfaceTokenKind::ReservedSymbol,
            ")",
            range(source_id, 230, 231),
        );
        let term_set_open = builder.add_token(
            SurfaceTokenKind::ReservedSymbol,
            "{",
            range(source_id, 232, 233),
        );
        let term_set_first = builder.add_token(
            SurfaceTokenKind::Identifier,
            "a",
            range(source_id, 233, 234),
        );
        let term_set_comma = builder.add_token(
            SurfaceTokenKind::ReservedSymbol,
            ",",
            range(source_id, 234, 235),
        );
        let term_set_second = builder.add_token(
            SurfaceTokenKind::Identifier,
            "b",
            range(source_id, 235, 236),
        );
        let term_set_close = builder.add_token(
            SurfaceTokenKind::ReservedSymbol,
            "}",
            range(source_id, 236, 237),
        );
        let expression = builder.add_node(
            SurfaceNodeKind::InfixExpression(SurfaceInfixOperator {
                spelling: "++".into(),
                precedence: 10,
                associativity: SurfaceOperatorAssociativity::Right,
            }),
            range(source_id, 0, 21),
            vec![identifier, user_symbol, numeral],
        );
        let module_prefix_node = builder.add_node(
            SurfaceNodeKind::RelativePrefix,
            range(source_id, 49, 51),
            vec![module_prefix],
        );
        let module_std_node = builder.add_node(
            SurfaceNodeKind::PathSegment,
            range(source_id, 51, 54),
            vec![module_std],
        );
        let module_algebra_node = builder.add_node(
            SurfaceNodeKind::PathSegment,
            range(source_id, 55, 62),
            vec![module_algebra],
        );
        let module_path = builder.add_node(
            SurfaceNodeKind::ModulePath,
            range(source_id, 49, 62),
            vec![
                module_prefix_node,
                module_std_node,
                module_dot,
                module_algebra_node,
            ],
        );
        let namespace_mml_node = builder.add_node(
            SurfaceNodeKind::PathSegment,
            range(source_id, 63, 66),
            vec![namespace_mml],
        );
        let namespace_nat_node = builder.add_node(
            SurfaceNodeKind::PathSegment,
            range(source_id, 67, 70),
            vec![namespace_nat],
        );
        let namespace_path = builder.add_node(
            SurfaceNodeKind::NamespacePath,
            range(source_id, 63, 70),
            vec![namespace_mml_node, namespace_dot, namespace_nat_node],
        );
        let qualified_top_node = builder.add_node(
            SurfaceNodeKind::PathSegment,
            range(source_id, 71, 74),
            vec![qualified_top],
        );
        let qualified_space_node = builder.add_node(
            SurfaceNodeKind::PathSegment,
            range(source_id, 75, 80),
            vec![qualified_space],
        );
        let qualified_symbol = builder.add_node(
            SurfaceNodeKind::QualifiedSymbol,
            range(source_id, 71, 80),
            vec![qualified_top_node, qualified_dot, qualified_space_node],
        );
        let placeholder_item = builder.add_node(
            SurfaceNodeKind::PlaceholderItem,
            range(source_id, 81, 89),
            vec![item_theorem, item_semicolon],
        );
        let import_path_prefix_node = builder.add_node(
            SurfaceNodeKind::RelativePrefix,
            range(source_id, 97, 98),
            vec![import_path_prefix],
        );
        let import_path_core_node = builder.add_node(
            SurfaceNodeKind::PathSegment,
            range(source_id, 98, 102),
            vec![import_path_core],
        );
        let import_branch_path = builder.add_node(
            SurfaceNodeKind::ModulePath,
            range(source_id, 97, 102),
            vec![import_path_prefix_node, import_path_core_node],
        );
        let import_branch_segment_node = builder.add_node(
            SurfaceNodeKind::PathSegment,
            range(source_id, 104, 110),
            vec![import_branch_segment],
        );
        let module_branch_import = builder.add_node(
            SurfaceNodeKind::ModuleBranchImport,
            range(source_id, 97, 111),
            vec![
                import_branch_path,
                import_branch_opener,
                import_branch_segment_node,
                import_branch_close,
            ],
        );
        let import_alias_path_node = builder.add_node(
            SurfaceNodeKind::PathSegment,
            range(source_id, 113, 120),
            vec![import_alias_path],
        );
        let import_alias_module_path = builder.add_node(
            SurfaceNodeKind::ModulePath,
            range(source_id, 113, 120),
            vec![import_alias_path_node],
        );
        let import_alias_node = builder.add_node(
            SurfaceNodeKind::PathSegment,
            range(source_id, 124, 125),
            vec![import_alias],
        );
        let import_alias_decl = builder.add_node(
            SurfaceNodeKind::ImportAliasDecl,
            range(source_id, 113, 125),
            vec![import_alias_module_path, import_alias_as, import_alias_node],
        );
        let import_item = builder.add_node(
            SurfaceNodeKind::ImportItem,
            range(source_id, 90, 126),
            vec![
                import_keyword,
                module_branch_import,
                import_comma,
                import_alias_decl,
                import_semicolon,
            ],
        );
        let export_path_segment = builder.add_node(
            SurfaceNodeKind::PathSegment,
            range(source_id, 134, 137),
            vec![export_path_std],
        );
        let export_module_path = builder.add_node(
            SurfaceNodeKind::ModulePath,
            range(source_id, 134, 137),
            vec![export_path_segment],
        );
        let export_item = builder.add_node(
            SurfaceNodeKind::ExportItem,
            range(source_id, 127, 138),
            vec![export_keyword, export_module_path, export_semicolon],
        );
        let visibility_marker = builder.add_node(
            SurfaceNodeKind::VisibilityMarker,
            range(source_id, 139, 145),
            vec![visibility_public],
        );
        let visible_placeholder = builder.add_node(
            SurfaceNodeKind::PlaceholderItem,
            range(source_id, 146, 154),
            vec![visible_theorem, visible_semicolon],
        );
        let visible_item = builder.add_node(
            SurfaceNodeKind::VisibleItem,
            range(source_id, 139, 154),
            vec![visibility_marker, visible_placeholder],
        );
        let reserve_empty_segment = builder.add_node(
            SurfaceNodeKind::PathSegment,
            range(source_id, 173, 178),
            vec![reserve_empty],
        );
        let reserve_empty_symbol = builder.add_node(
            SurfaceNodeKind::QualifiedSymbol,
            range(source_id, 173, 178),
            vec![reserve_empty_segment],
        );
        let reserve_attribute = builder.add_node(
            SurfaceNodeKind::AttributeRef,
            range(source_id, 169, 178),
            vec![reserve_non, reserve_empty_symbol],
        );
        let reserve_attribute_chain = builder.add_node(
            SurfaceNodeKind::AttributeChain,
            range(source_id, 169, 178),
            vec![reserve_attribute],
        );
        let reserve_type_segment = builder.add_node(
            SurfaceNodeKind::PathSegment,
            range(source_id, 179, 180),
            vec![reserve_type_symbol],
        );
        let reserve_type_symbol_node = builder.add_node(
            SurfaceNodeKind::QualifiedSymbol,
            range(source_id, 179, 180),
            vec![reserve_type_segment],
        );
        let reserve_set_head = builder.add_node(
            SurfaceNodeKind::TypeHead,
            range(source_id, 182, 185),
            vec![reserve_set],
        );
        let reserve_set_expression = builder.add_node(
            SurfaceNodeKind::TypeExpression,
            range(source_id, 182, 185),
            vec![reserve_set_head],
        );
        let reserve_qua_placeholder = builder.add_node(
            SurfaceNodeKind::TermPlaceholder,
            range(source_id, 187, 194),
            vec![reserve_arg_identifier, reserve_qua, reserve_radix],
        );
        let reserve_type_arguments = builder.add_node(
            SurfaceNodeKind::TypeArguments,
            range(source_id, 181, 195),
            vec![
                reserve_arg_open,
                reserve_set_expression,
                reserve_arg_comma,
                reserve_qua_placeholder,
                reserve_arg_close,
            ],
        );
        let reserve_type_head = builder.add_node(
            SurfaceNodeKind::TypeHead,
            range(source_id, 179, 195),
            vec![reserve_type_symbol_node, reserve_type_arguments],
        );
        let reserve_type_expression = builder.add_node(
            SurfaceNodeKind::TypeExpression,
            range(source_id, 169, 195),
            vec![reserve_attribute_chain, reserve_type_head],
        );
        let reserve_segment = builder.add_node(
            SurfaceNodeKind::ReserveSegment,
            range(source_id, 163, 195),
            vec![reserve_x, reserve_for, reserve_type_expression],
        );
        let reserve_item = builder.add_node(
            SurfaceNodeKind::ReserveItem,
            range(source_id, 155, 196),
            vec![reserve_keyword, reserve_segment, reserve_semicolon],
        );
        let standalone_parameter_prefix = builder.add_node(
            SurfaceNodeKind::ParameterPrefix,
            range(source_id, 197, 199),
            vec![standalone_prefix_n, standalone_prefix_hyphen],
        );
        let term_reference = builder.add_node(
            SurfaceNodeKind::TermReference,
            range(source_id, 200, 201),
            vec![term_reference_token],
        );
        let term_reference_expression = builder.add_node(
            SurfaceNodeKind::TermExpression,
            range(source_id, 200, 201),
            vec![term_reference],
        );
        let numeral_term = builder.add_node(
            SurfaceNodeKind::NumeralTerm,
            range(source_id, 202, 204),
            vec![term_numeral_token],
        );
        let numeral_expression = builder.add_node(
            SurfaceNodeKind::TermExpression,
            range(source_id, 202, 204),
            vec![numeral_term],
        );
        let it_term = builder.add_node(
            SurfaceNodeKind::ItTerm,
            range(source_id, 205, 207),
            vec![term_it_token],
        );
        let it_expression = builder.add_node(
            SurfaceNodeKind::TermExpression,
            range(source_id, 205, 207),
            vec![it_term],
        );
        let paren_reference = builder.add_node(
            SurfaceNodeKind::TermReference,
            range(source_id, 209, 210),
            vec![term_paren_identifier],
        );
        let paren_inner_expression = builder.add_node(
            SurfaceNodeKind::TermExpression,
            range(source_id, 209, 210),
            vec![paren_reference],
        );
        let parenthesized_term = builder.add_node(
            SurfaceNodeKind::ParenthesizedTerm,
            range(source_id, 208, 211),
            vec![term_paren_open, paren_inner_expression, term_paren_close],
        );
        let parenthesized_expression = builder.add_node(
            SurfaceNodeKind::TermExpression,
            range(source_id, 208, 211),
            vec![parenthesized_term],
        );
        let choice_set_head = builder.add_node(
            SurfaceNodeKind::TypeHead,
            range(source_id, 216, 219),
            vec![term_choice_set],
        );
        let choice_type_expression = builder.add_node(
            SurfaceNodeKind::TypeExpression,
            range(source_id, 216, 219),
            vec![choice_set_head],
        );
        let choice_term = builder.add_node(
            SurfaceNodeKind::ChoiceTerm,
            range(source_id, 212, 219),
            vec![term_the, choice_type_expression],
        );
        let choice_expression = builder.add_node(
            SurfaceNodeKind::TermExpression,
            range(source_id, 212, 219),
            vec![choice_term],
        );
        let apply_symbol_segment = builder.add_node(
            SurfaceNodeKind::PathSegment,
            range(source_id, 220, 221),
            vec![term_apply_symbol],
        );
        let apply_symbol_node = builder.add_node(
            SurfaceNodeKind::QualifiedSymbol,
            range(source_id, 220, 221),
            vec![apply_symbol_segment],
        );
        let apply_reference = builder.add_node(
            SurfaceNodeKind::TermReference,
            range(source_id, 220, 221),
            vec![apply_symbol_node],
        );
        let apply_argument_reference = builder.add_node(
            SurfaceNodeKind::TermReference,
            range(source_id, 222, 223),
            vec![term_apply_arg],
        );
        let apply_argument = builder.add_node(
            SurfaceNodeKind::TermExpression,
            range(source_id, 222, 223),
            vec![apply_argument_reference],
        );
        let application_term = builder.add_node(
            SurfaceNodeKind::ApplicationTerm,
            range(source_id, 220, 224),
            vec![
                apply_reference,
                term_apply_open,
                apply_argument,
                term_apply_close,
            ],
        );
        let application_expression = builder.add_node(
            SurfaceNodeKind::TermExpression,
            range(source_id, 220, 224),
            vec![application_term],
        );
        let struct_symbol_segment = builder.add_node(
            SurfaceNodeKind::PathSegment,
            range(source_id, 225, 226),
            vec![term_struct_symbol],
        );
        let struct_symbol_node = builder.add_node(
            SurfaceNodeKind::QualifiedSymbol,
            range(source_id, 225, 226),
            vec![struct_symbol_segment],
        );
        let field_value_reference = builder.add_node(
            SurfaceNodeKind::TermReference,
            range(source_id, 229, 230),
            vec![term_field_value],
        );
        let field_value_expression = builder.add_node(
            SurfaceNodeKind::TermExpression,
            range(source_id, 229, 230),
            vec![field_value_reference],
        );
        let field_argument = builder.add_node(
            SurfaceNodeKind::FieldArgument,
            range(source_id, 227, 230),
            vec![term_field_name, term_field_colon, field_value_expression],
        );
        let structure_constructor = builder.add_node(
            SurfaceNodeKind::StructureConstructor,
            range(source_id, 225, 231),
            vec![
                struct_symbol_node,
                term_struct_open,
                field_argument,
                term_struct_close,
            ],
        );
        let structure_expression = builder.add_node(
            SurfaceNodeKind::TermExpression,
            range(source_id, 225, 231),
            vec![structure_constructor],
        );
        let set_first_reference = builder.add_node(
            SurfaceNodeKind::TermReference,
            range(source_id, 233, 234),
            vec![term_set_first],
        );
        let set_first_expression = builder.add_node(
            SurfaceNodeKind::TermExpression,
            range(source_id, 233, 234),
            vec![set_first_reference],
        );
        let set_second_reference = builder.add_node(
            SurfaceNodeKind::TermReference,
            range(source_id, 235, 236),
            vec![term_set_second],
        );
        let set_second_expression = builder.add_node(
            SurfaceNodeKind::TermExpression,
            range(source_id, 235, 236),
            vec![set_second_reference],
        );
        let set_enumeration = builder.add_node(
            SurfaceNodeKind::SetEnumeration,
            range(source_id, 232, 237),
            vec![
                term_set_open,
                set_first_expression,
                term_set_comma,
                set_second_expression,
                term_set_close,
            ],
        );
        let set_expression = builder.add_node(
            SurfaceNodeKind::TermExpression,
            range(source_id, 232, 237),
            vec![set_enumeration],
        );
        let item_list = builder.add_node(
            SurfaceNodeKind::ItemList,
            range(source_id, 81, 154),
            vec![placeholder_item, import_item, export_item, visible_item],
        );
        let compilation_unit = builder.add_node(
            SurfaceNodeKind::CompilationUnit,
            range(source_id, 81, 154),
            vec![item_list],
        );
        let recovery = builder.add_recovery(
            SyntaxRecoveryKind::MissingEnd,
            range(source_id, 89, 89),
            vec![reserved_word],
        );
        let root = builder.add_node(
            SurfaceNodeKind::Root,
            range(source_id, 0, 237),
            vec![
                identifier,
                reserved_word,
                reserved_symbol,
                numeral,
                lexeme_run,
                user_symbol,
                string_literal,
                error_token,
                unknown,
                recovered_token,
                module_prefix,
                module_std,
                module_dot,
                module_algebra,
                namespace_mml,
                namespace_dot,
                namespace_nat,
                qualified_top,
                qualified_dot,
                qualified_space,
                item_theorem,
                item_semicolon,
                import_keyword,
                import_path_prefix,
                import_path_core,
                import_branch_opener,
                import_branch_segment,
                import_branch_close,
                import_comma,
                import_alias_path,
                import_alias_as,
                import_alias,
                import_semicolon,
                export_keyword,
                export_path_std,
                export_semicolon,
                visibility_public,
                visible_theorem,
                visible_semicolon,
                reserve_keyword,
                reserve_x,
                reserve_for,
                reserve_non,
                reserve_empty,
                reserve_type_symbol,
                reserve_arg_open,
                reserve_set,
                reserve_arg_comma,
                reserve_arg_identifier,
                reserve_qua,
                reserve_radix,
                reserve_arg_close,
                reserve_semicolon,
                standalone_prefix_n,
                standalone_prefix_hyphen,
                term_reference_token,
                term_numeral_token,
                term_it_token,
                term_paren_open,
                term_paren_identifier,
                term_paren_close,
                term_the,
                term_choice_set,
                term_apply_symbol,
                term_apply_open,
                term_apply_arg,
                term_apply_close,
                term_struct_symbol,
                term_struct_open,
                term_field_name,
                term_field_colon,
                term_field_value,
                term_struct_close,
                term_set_open,
                term_set_first,
                term_set_comma,
                term_set_second,
                term_set_close,
                expression,
                module_path,
                namespace_path,
                qualified_symbol,
                compilation_unit,
                reserve_item,
                standalone_parameter_prefix,
                term_reference_expression,
                numeral_expression,
                it_expression,
                parenthesized_expression,
                choice_expression,
                application_expression,
                structure_expression,
                set_expression,
                recovery,
            ],
        );
        builder.finish(Some(root), Some(expression))
    }

    const fn range(source_id: SourceId, start: usize, end: usize) -> SourceRange {
        SourceRange {
            source_id,
            start,
            end,
        }
    }

    fn source_id(byte: u8) -> SourceId {
        InMemorySessionIdAllocator::new()
            .next_source_id(snapshot_id(byte))
            .unwrap()
    }

    const fn sid(id: super::SurfaceBuilderNodeId) -> super::SurfaceNodeId {
        id.into_surface_node_id()
    }

    fn snapshot_id(byte: u8) -> BuildSnapshotId {
        let hex = format!("{byte:02x}").repeat(Hash::BYTE_LEN);
        BuildSnapshotId::from_published_schema_str(&format!(
            "mizar-session-build-snapshot-v1:{hex}"
        ))
        .unwrap()
    }
}
