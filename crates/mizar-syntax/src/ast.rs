mod green;
mod snapshot;

use crate::recovery::SyntaxRecoveryKind;
use crate::trivia::{
    SurfaceTrivia, TriviaAttachmentTarget, TriviaNodeTarget, write_trivia_snapshot,
};
use mizar_session::{SourceId, SourceRange};
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
    SelectorAccess = 38,
    StructureUpdate = 39,
    FieldUpdate = 40,
    QuaExpression = 41,
    PrefixExpression = 42,
    PostfixExpression = 43,
    FormulaExpression = 44,
    BuiltinPredicateApplication = 45,
    IsAssertion = 46,
    AttributeTestChain = 47,
    PredicateApplication = 48,
    PredicateSegment = 49,
    PredicateHead = 50,
    InlinePredicateApplication = 51,
    PrefixFormula = 52,
    BinaryFormula = 53,
    ParenthesizedFormula = 54,
    QuantifiedFormula = 55,
    QuantifierVariableSegment = 56,
    FormulaConstant = 57,
    SetComprehension = 58,
    ComprehensionVariableSegment = 59,
    StatementItem = 60,
    LetStatement = 61,
    QualifiedVariableSegment = 62,
    AssumptionStatement = 63,
    Proposition = 64,
    ConditionList = 65,
    GivenStatement = 66,
    TakeStatement = 67,
    Witness = 68,
    SetStatement = 69,
    Equating = 70,
    CompactStatement = 71,
    JustificationClause = 72,
    ReferenceList = 73,
    Reference = 74,
    QualifiedReference = 75,
    GroupedReference = 76,
    GroupedReferenceItem = 77,
    BulkReference = 78,
    ComputationJustification = 79,
    ComputationOption = 80,
    ConsiderStatement = 81,
    ReconsiderStatement = 82,
    ReconsiderItem = 83,
    ConclusionStatement = 84,
    ThenStatement = 85,
    IterativeEqualityStatement = 86,
    IterativeEqualityStep = 87,
    NowStatement = 88,
    HerebyStatement = 89,
    CaseReasoningStatement = 90,
    CaseItem = 91,
    SupposeItem = 92,
    InlineFunctorDefinition = 93,
    InlinePredicateDefinition = 94,
    TypedParameter = 95,
    TheoremItem = 96,
    LemmaItem = 97,
    ProofBlock = 98,
    DefinitionBlockItem = 109,
    DefinitionParameter = 110,
    AttributeDefinition = 111,
    AttributePattern = 112,
    FormulaDefiniens = 113,
    FormulaCase = 114,
    CorrectnessCondition = 115,
    PredicateDefinition = 116,
    PredicatePattern = 117,
    FunctorDefinition = 118,
    FunctorPattern = 119,
    TermDefiniens = 120,
    TermCase = 121,
    ModeDefinition = 122,
    ModePattern = 123,
    ModeProperty = 124,
    AttributeRedefinition = 125,
    PredicateRedefinition = 126,
    FunctorRedefinition = 127,
    CoherenceCondition = 128,
    NotationAlias = 129,
    NotationPattern = 130,
    PropertyClause = 131,
    StructureDefinition = 132,
    StructurePattern = 133,
    StructureField = 134,
    StructureProperty = 135,
    InheritanceDefinition = 136,
    InheritanceTarget = 137,
    FieldRedefinition = 138,
    PropertyRedefinition = 139,
    RegistrationBlockItem = 140,
    RegistrationParameter = 141,
    ExistentialRegistration = 142,
    ConditionalRegistration = 143,
    FunctorialRegistration = 144,
    ReductionRegistration = 145,
    TemplateParameter = 146,
    TemplateLoci = 147,
    TemplateLocus = 148,
    TemplateArguments = 149,
    TemplateArgument = 150,
    AlgorithmDefinition = 151,
    AlgorithmParameters = 152,
    AlgorithmBody = 153,
    AlgorithmStatementList = 154,
    VariableDeclaration = 155,
    VariableBinding = 156,
    AssignmentStatement = 157,
    Lvalue = 158,
    SnapshotStatement = 159,
    ReturnStatement = 160,
    ClaimBlockItem = 161,
    IfStatement = 162,
    WhileStatement = 163,
    ForRangeStatement = 164,
    ForCollectionStatement = 165,
    MatchStatement = 166,
    MatchCase = 167,
    MatchEnding = 168,
    BreakStatement = 169,
    ContinueStatement = 170,
    AlgorithmTerminationClause = 171,
    AlgorithmRequiresClause = 172,
    AlgorithmEnsuresClause = 173,
    AlgorithmDecreasingClause = 174,
    LoopInvariantClause = 175,
    LoopDecreasingClause = 176,
    AssertStatement = 177,
    TermList = 178,
    Annotation = 179,
    LibraryAnnotation = 180,
    AnnotationLabelList = 181,
    AnnotationLabel = 182,
    AnnotationArgumentList = 183,
    AnnotationArgument = 184,
    ProofHintOptionList = 185,
    ProofHintOption = 186,
    StandaloneDiagnosticAnnotation = 187,
    AnnotatedStatement = 188,
    AnnotatedAlgorithmStatement = 189,
    AnnotatedDefinitionContent = 190,
    AnnotatedRegistrationContent = 191,
    TokenAnnotationMarker = 99,
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
            38 => Self::SelectorAccess,
            39 => Self::StructureUpdate,
            40 => Self::FieldUpdate,
            41 => Self::QuaExpression,
            42 => Self::PrefixExpression,
            43 => Self::PostfixExpression,
            44 => Self::FormulaExpression,
            45 => Self::BuiltinPredicateApplication,
            46 => Self::IsAssertion,
            47 => Self::AttributeTestChain,
            48 => Self::PredicateApplication,
            49 => Self::PredicateSegment,
            50 => Self::PredicateHead,
            51 => Self::InlinePredicateApplication,
            52 => Self::PrefixFormula,
            53 => Self::BinaryFormula,
            54 => Self::ParenthesizedFormula,
            55 => Self::QuantifiedFormula,
            56 => Self::QuantifierVariableSegment,
            57 => Self::FormulaConstant,
            58 => Self::SetComprehension,
            59 => Self::ComprehensionVariableSegment,
            60 => Self::StatementItem,
            61 => Self::LetStatement,
            62 => Self::QualifiedVariableSegment,
            63 => Self::AssumptionStatement,
            64 => Self::Proposition,
            65 => Self::ConditionList,
            66 => Self::GivenStatement,
            67 => Self::TakeStatement,
            68 => Self::Witness,
            69 => Self::SetStatement,
            70 => Self::Equating,
            71 => Self::CompactStatement,
            72 => Self::JustificationClause,
            73 => Self::ReferenceList,
            74 => Self::Reference,
            75 => Self::QualifiedReference,
            76 => Self::GroupedReference,
            77 => Self::GroupedReferenceItem,
            78 => Self::BulkReference,
            79 => Self::ComputationJustification,
            80 => Self::ComputationOption,
            81 => Self::ConsiderStatement,
            82 => Self::ReconsiderStatement,
            83 => Self::ReconsiderItem,
            84 => Self::ConclusionStatement,
            85 => Self::ThenStatement,
            86 => Self::IterativeEqualityStatement,
            87 => Self::IterativeEqualityStep,
            88 => Self::NowStatement,
            89 => Self::HerebyStatement,
            90 => Self::CaseReasoningStatement,
            91 => Self::CaseItem,
            92 => Self::SupposeItem,
            93 => Self::InlineFunctorDefinition,
            94 => Self::InlinePredicateDefinition,
            95 => Self::TypedParameter,
            96 => Self::TheoremItem,
            97 => Self::LemmaItem,
            98 => Self::ProofBlock,
            109 => Self::DefinitionBlockItem,
            110 => Self::DefinitionParameter,
            111 => Self::AttributeDefinition,
            112 => Self::AttributePattern,
            113 => Self::FormulaDefiniens,
            114 => Self::FormulaCase,
            115 => Self::CorrectnessCondition,
            116 => Self::PredicateDefinition,
            117 => Self::PredicatePattern,
            118 => Self::FunctorDefinition,
            119 => Self::FunctorPattern,
            120 => Self::TermDefiniens,
            121 => Self::TermCase,
            122 => Self::ModeDefinition,
            123 => Self::ModePattern,
            124 => Self::ModeProperty,
            125 => Self::AttributeRedefinition,
            126 => Self::PredicateRedefinition,
            127 => Self::FunctorRedefinition,
            128 => Self::CoherenceCondition,
            129 => Self::NotationAlias,
            130 => Self::NotationPattern,
            131 => Self::PropertyClause,
            132 => Self::StructureDefinition,
            133 => Self::StructurePattern,
            134 => Self::StructureField,
            135 => Self::StructureProperty,
            136 => Self::InheritanceDefinition,
            137 => Self::InheritanceTarget,
            138 => Self::FieldRedefinition,
            139 => Self::PropertyRedefinition,
            140 => Self::RegistrationBlockItem,
            141 => Self::RegistrationParameter,
            142 => Self::ExistentialRegistration,
            143 => Self::ConditionalRegistration,
            144 => Self::FunctorialRegistration,
            145 => Self::ReductionRegistration,
            146 => Self::TemplateParameter,
            147 => Self::TemplateLoci,
            148 => Self::TemplateLocus,
            149 => Self::TemplateArguments,
            150 => Self::TemplateArgument,
            151 => Self::AlgorithmDefinition,
            152 => Self::AlgorithmParameters,
            153 => Self::AlgorithmBody,
            154 => Self::AlgorithmStatementList,
            155 => Self::VariableDeclaration,
            156 => Self::VariableBinding,
            157 => Self::AssignmentStatement,
            158 => Self::Lvalue,
            159 => Self::SnapshotStatement,
            160 => Self::ReturnStatement,
            161 => Self::ClaimBlockItem,
            162 => Self::IfStatement,
            163 => Self::WhileStatement,
            164 => Self::ForRangeStatement,
            165 => Self::ForCollectionStatement,
            166 => Self::MatchStatement,
            167 => Self::MatchCase,
            168 => Self::MatchEnding,
            169 => Self::BreakStatement,
            170 => Self::ContinueStatement,
            171 => Self::AlgorithmTerminationClause,
            172 => Self::AlgorithmRequiresClause,
            173 => Self::AlgorithmEnsuresClause,
            174 => Self::AlgorithmDecreasingClause,
            175 => Self::LoopInvariantClause,
            176 => Self::LoopDecreasingClause,
            177 => Self::AssertStatement,
            178 => Self::TermList,
            179 => Self::Annotation,
            180 => Self::LibraryAnnotation,
            181 => Self::AnnotationLabelList,
            182 => Self::AnnotationLabel,
            183 => Self::AnnotationArgumentList,
            184 => Self::AnnotationArgument,
            185 => Self::ProofHintOptionList,
            186 => Self::ProofHintOption,
            187 => Self::StandaloneDiagnosticAnnotation,
            188 => Self::AnnotatedStatement,
            189 => Self::AnnotatedAlgorithmStatement,
            190 => Self::AnnotatedDefinitionContent,
            191 => Self::AnnotatedRegistrationContent,
            99 => Self::TokenAnnotationMarker,
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
                | Self::SelectorAccess
                | Self::StructureUpdate
                | Self::FieldUpdate
                | Self::QuaExpression
                | Self::PrefixExpression
                | Self::PostfixExpression
                | Self::FormulaExpression
                | Self::BuiltinPredicateApplication
                | Self::IsAssertion
                | Self::AttributeTestChain
                | Self::PredicateApplication
                | Self::PredicateSegment
                | Self::PredicateHead
                | Self::InlinePredicateApplication
                | Self::PrefixFormula
                | Self::BinaryFormula
                | Self::ParenthesizedFormula
                | Self::QuantifiedFormula
                | Self::QuantifierVariableSegment
                | Self::FormulaConstant
                | Self::SetComprehension
                | Self::ComprehensionVariableSegment
                | Self::StatementItem
                | Self::LetStatement
                | Self::QualifiedVariableSegment
                | Self::AssumptionStatement
                | Self::Proposition
                | Self::ConditionList
                | Self::GivenStatement
                | Self::TakeStatement
                | Self::Witness
                | Self::SetStatement
                | Self::Equating
                | Self::CompactStatement
                | Self::JustificationClause
                | Self::ReferenceList
                | Self::Reference
                | Self::QualifiedReference
                | Self::GroupedReference
                | Self::GroupedReferenceItem
                | Self::BulkReference
                | Self::ComputationJustification
                | Self::ComputationOption
                | Self::ConsiderStatement
                | Self::ReconsiderStatement
                | Self::ReconsiderItem
                | Self::ConclusionStatement
                | Self::ThenStatement
                | Self::IterativeEqualityStatement
                | Self::IterativeEqualityStep
                | Self::NowStatement
                | Self::HerebyStatement
                | Self::CaseReasoningStatement
                | Self::CaseItem
                | Self::SupposeItem
                | Self::InlineFunctorDefinition
                | Self::InlinePredicateDefinition
                | Self::TypedParameter
                | Self::TheoremItem
                | Self::LemmaItem
                | Self::ProofBlock
                | Self::DefinitionBlockItem
                | Self::DefinitionParameter
                | Self::AttributeDefinition
                | Self::AttributePattern
                | Self::FormulaDefiniens
                | Self::FormulaCase
                | Self::CorrectnessCondition
                | Self::PredicateDefinition
                | Self::PredicatePattern
                | Self::FunctorDefinition
                | Self::FunctorPattern
                | Self::TermDefiniens
                | Self::TermCase
                | Self::ModeDefinition
                | Self::ModePattern
                | Self::ModeProperty
                | Self::AttributeRedefinition
                | Self::PredicateRedefinition
                | Self::FunctorRedefinition
                | Self::CoherenceCondition
                | Self::NotationAlias
                | Self::NotationPattern
                | Self::PropertyClause
                | Self::StructureDefinition
                | Self::StructurePattern
                | Self::StructureField
                | Self::StructureProperty
                | Self::InheritanceDefinition
                | Self::InheritanceTarget
                | Self::FieldRedefinition
                | Self::PropertyRedefinition
                | Self::RegistrationBlockItem
                | Self::RegistrationParameter
                | Self::ExistentialRegistration
                | Self::ConditionalRegistration
                | Self::FunctorialRegistration
                | Self::ReductionRegistration
                | Self::TemplateParameter
                | Self::TemplateLoci
                | Self::TemplateLocus
                | Self::TemplateArguments
                | Self::TemplateArgument
                | Self::AlgorithmDefinition
                | Self::AlgorithmParameters
                | Self::AlgorithmBody
                | Self::AlgorithmStatementList
                | Self::VariableDeclaration
                | Self::VariableBinding
                | Self::AssignmentStatement
                | Self::Lvalue
                | Self::SnapshotStatement
                | Self::ReturnStatement
                | Self::ClaimBlockItem
                | Self::IfStatement
                | Self::WhileStatement
                | Self::ForRangeStatement
                | Self::ForCollectionStatement
                | Self::MatchStatement
                | Self::MatchCase
                | Self::MatchEnding
                | Self::BreakStatement
                | Self::ContinueStatement
                | Self::AlgorithmTerminationClause
                | Self::AlgorithmRequiresClause
                | Self::AlgorithmEnsuresClause
                | Self::AlgorithmDecreasingClause
                | Self::LoopInvariantClause
                | Self::LoopDecreasingClause
                | Self::AssertStatement
                | Self::TermList
                | Self::Annotation
                | Self::LibraryAnnotation
                | Self::AnnotationLabelList
                | Self::AnnotationLabel
                | Self::AnnotationArgumentList
                | Self::AnnotationArgument
                | Self::ProofHintOptionList
                | Self::ProofHintOption
                | Self::StandaloneDiagnosticAnnotation
                | Self::AnnotatedStatement
                | Self::AnnotatedAlgorithmStatement
                | Self::AnnotatedDefinitionContent
                | Self::AnnotatedRegistrationContent
        )
    }

    pub const fn is_token_kind(self) -> bool {
        matches!(
            self,
            Self::TokenAnnotationMarker
                | Self::TokenIdentifier
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
        let green = green::build_green_tree(&nodes, root);
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
        snapshot::snapshot_text(self)
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
            _ => None,
        }
    }

    pub const fn as_infix_expression(self) -> Option<&'a SurfaceInfixOperator> {
        match &self.node.kind {
            SurfaceNodeKind::InfixExpression(operator) => Some(operator),
            _ => None,
        }
    }

    pub const fn as_prefix_expression(self) -> Option<&'a SurfacePrefixOperator> {
        match &self.node.kind {
            SurfaceNodeKind::PrefixExpression(operator) => Some(operator),
            _ => None,
        }
    }

    pub const fn as_postfix_expression(self) -> Option<&'a SurfacePostfixOperator> {
        match &self.node.kind {
            SurfaceNodeKind::PostfixExpression(operator) => Some(operator),
            _ => None,
        }
    }

    pub const fn as_recovery(self) -> Option<SyntaxRecoveryKind> {
        match self.node.kind {
            SurfaceNodeKind::ErrorRecovery(kind) => Some(kind),
            _ => None,
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

    pub fn as_template_loci(self) -> Option<Self> {
        match &self.node.kind {
            SurfaceNodeKind::TemplateLoci => Some(self),
            _ => None,
        }
    }

    pub fn as_template_locus(self) -> Option<Self> {
        match &self.node.kind {
            SurfaceNodeKind::TemplateLocus => Some(self),
            _ => None,
        }
    }

    pub fn as_template_arguments(self) -> Option<Self> {
        match &self.node.kind {
            SurfaceNodeKind::TemplateArguments => Some(self),
            _ => None,
        }
    }

    pub fn as_template_argument(self) -> Option<Self> {
        match &self.node.kind {
            SurfaceNodeKind::TemplateArgument => Some(self),
            _ => None,
        }
    }

    pub fn as_algorithm_definition(self) -> Option<Self> {
        match &self.node.kind {
            SurfaceNodeKind::AlgorithmDefinition => Some(self),
            _ => None,
        }
    }

    pub fn as_algorithm_parameters(self) -> Option<Self> {
        match &self.node.kind {
            SurfaceNodeKind::AlgorithmParameters => Some(self),
            _ => None,
        }
    }

    pub fn as_algorithm_body(self) -> Option<Self> {
        match &self.node.kind {
            SurfaceNodeKind::AlgorithmBody => Some(self),
            _ => None,
        }
    }

    pub fn as_algorithm_statement_list(self) -> Option<Self> {
        match &self.node.kind {
            SurfaceNodeKind::AlgorithmStatementList => Some(self),
            _ => None,
        }
    }

    pub fn as_variable_declaration(self) -> Option<Self> {
        match &self.node.kind {
            SurfaceNodeKind::VariableDeclaration => Some(self),
            _ => None,
        }
    }

    pub fn as_variable_binding(self) -> Option<Self> {
        match &self.node.kind {
            SurfaceNodeKind::VariableBinding => Some(self),
            _ => None,
        }
    }

    pub fn as_assignment_statement(self) -> Option<Self> {
        match &self.node.kind {
            SurfaceNodeKind::AssignmentStatement => Some(self),
            _ => None,
        }
    }

    pub fn as_lvalue(self) -> Option<Self> {
        match &self.node.kind {
            SurfaceNodeKind::Lvalue => Some(self),
            _ => None,
        }
    }

    pub fn as_snapshot_statement(self) -> Option<Self> {
        match &self.node.kind {
            SurfaceNodeKind::SnapshotStatement => Some(self),
            _ => None,
        }
    }

    pub fn as_return_statement(self) -> Option<Self> {
        match &self.node.kind {
            SurfaceNodeKind::ReturnStatement => Some(self),
            _ => None,
        }
    }

    pub fn as_claim_block_item(self) -> Option<Self> {
        match &self.node.kind {
            SurfaceNodeKind::ClaimBlockItem => Some(self),
            _ => None,
        }
    }

    pub fn as_if_statement(self) -> Option<Self> {
        match &self.node.kind {
            SurfaceNodeKind::IfStatement => Some(self),
            _ => None,
        }
    }

    pub fn as_while_statement(self) -> Option<Self> {
        match &self.node.kind {
            SurfaceNodeKind::WhileStatement => Some(self),
            _ => None,
        }
    }

    pub fn as_for_range_statement(self) -> Option<Self> {
        match &self.node.kind {
            SurfaceNodeKind::ForRangeStatement => Some(self),
            _ => None,
        }
    }

    pub fn as_for_collection_statement(self) -> Option<Self> {
        match &self.node.kind {
            SurfaceNodeKind::ForCollectionStatement => Some(self),
            _ => None,
        }
    }

    pub fn as_match_statement(self) -> Option<Self> {
        match &self.node.kind {
            SurfaceNodeKind::MatchStatement => Some(self),
            _ => None,
        }
    }

    pub fn as_match_case(self) -> Option<Self> {
        match &self.node.kind {
            SurfaceNodeKind::MatchCase => Some(self),
            _ => None,
        }
    }

    pub fn as_match_ending(self) -> Option<Self> {
        match &self.node.kind {
            SurfaceNodeKind::MatchEnding => Some(self),
            _ => None,
        }
    }

    pub fn as_break_statement(self) -> Option<Self> {
        match &self.node.kind {
            SurfaceNodeKind::BreakStatement => Some(self),
            _ => None,
        }
    }

    pub fn as_continue_statement(self) -> Option<Self> {
        match &self.node.kind {
            SurfaceNodeKind::ContinueStatement => Some(self),
            _ => None,
        }
    }

    pub fn as_algorithm_termination_clause(self) -> Option<Self> {
        match &self.node.kind {
            SurfaceNodeKind::AlgorithmTerminationClause => Some(self),
            _ => None,
        }
    }

    pub fn as_algorithm_requires_clause(self) -> Option<Self> {
        match &self.node.kind {
            SurfaceNodeKind::AlgorithmRequiresClause => Some(self),
            _ => None,
        }
    }

    pub fn as_algorithm_ensures_clause(self) -> Option<Self> {
        match &self.node.kind {
            SurfaceNodeKind::AlgorithmEnsuresClause => Some(self),
            _ => None,
        }
    }

    pub fn as_algorithm_decreasing_clause(self) -> Option<Self> {
        match &self.node.kind {
            SurfaceNodeKind::AlgorithmDecreasingClause => Some(self),
            _ => None,
        }
    }

    pub fn as_loop_invariant_clause(self) -> Option<Self> {
        match &self.node.kind {
            SurfaceNodeKind::LoopInvariantClause => Some(self),
            _ => None,
        }
    }

    pub fn as_loop_decreasing_clause(self) -> Option<Self> {
        match &self.node.kind {
            SurfaceNodeKind::LoopDecreasingClause => Some(self),
            _ => None,
        }
    }

    pub fn as_assert_statement(self) -> Option<Self> {
        match &self.node.kind {
            SurfaceNodeKind::AssertStatement => Some(self),
            _ => None,
        }
    }

    pub fn as_term_list(self) -> Option<Self> {
        match &self.node.kind {
            SurfaceNodeKind::TermList => Some(self),
            _ => None,
        }
    }

    pub fn as_annotation(self) -> Option<Self> {
        match &self.node.kind {
            SurfaceNodeKind::Annotation => Some(self),
            _ => None,
        }
    }

    pub fn as_library_annotation(self) -> Option<Self> {
        match &self.node.kind {
            SurfaceNodeKind::LibraryAnnotation => Some(self),
            _ => None,
        }
    }

    pub fn as_annotation_label_list(self) -> Option<Self> {
        match &self.node.kind {
            SurfaceNodeKind::AnnotationLabelList => Some(self),
            _ => None,
        }
    }

    pub fn as_annotation_label(self) -> Option<Self> {
        match &self.node.kind {
            SurfaceNodeKind::AnnotationLabel => Some(self),
            _ => None,
        }
    }

    pub fn as_annotation_argument_list(self) -> Option<Self> {
        match &self.node.kind {
            SurfaceNodeKind::AnnotationArgumentList => Some(self),
            _ => None,
        }
    }

    pub fn as_annotation_argument(self) -> Option<Self> {
        match &self.node.kind {
            SurfaceNodeKind::AnnotationArgument => Some(self),
            _ => None,
        }
    }

    pub fn as_proof_hint_option_list(self) -> Option<Self> {
        match &self.node.kind {
            SurfaceNodeKind::ProofHintOptionList => Some(self),
            _ => None,
        }
    }

    pub fn as_proof_hint_option(self) -> Option<Self> {
        match &self.node.kind {
            SurfaceNodeKind::ProofHintOption => Some(self),
            _ => None,
        }
    }

    pub fn as_standalone_diagnostic_annotation(self) -> Option<Self> {
        match &self.node.kind {
            SurfaceNodeKind::StandaloneDiagnosticAnnotation => Some(self),
            _ => None,
        }
    }

    pub fn as_annotated_statement(self) -> Option<Self> {
        match &self.node.kind {
            SurfaceNodeKind::AnnotatedStatement => Some(self),
            _ => None,
        }
    }

    pub fn as_annotated_algorithm_statement(self) -> Option<Self> {
        match &self.node.kind {
            SurfaceNodeKind::AnnotatedAlgorithmStatement => Some(self),
            _ => None,
        }
    }

    pub fn as_annotated_definition_content(self) -> Option<Self> {
        match &self.node.kind {
            SurfaceNodeKind::AnnotatedDefinitionContent => Some(self),
            _ => None,
        }
    }

    pub fn as_annotated_registration_content(self) -> Option<Self> {
        match &self.node.kind {
            SurfaceNodeKind::AnnotatedRegistrationContent => Some(self),
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

    pub fn as_set_comprehension(self) -> Option<Self> {
        match &self.node.kind {
            SurfaceNodeKind::SetComprehension => Some(self),
            _ => None,
        }
    }

    pub fn as_comprehension_variable_segment(self) -> Option<Self> {
        match &self.node.kind {
            SurfaceNodeKind::ComprehensionVariableSegment => Some(self),
            _ => None,
        }
    }

    pub fn as_statement_item(self) -> Option<Self> {
        match &self.node.kind {
            SurfaceNodeKind::StatementItem => Some(self),
            _ => None,
        }
    }

    pub fn as_theorem_item(self) -> Option<Self> {
        match &self.node.kind {
            SurfaceNodeKind::TheoremItem => Some(self),
            _ => None,
        }
    }

    pub fn as_lemma_item(self) -> Option<Self> {
        match &self.node.kind {
            SurfaceNodeKind::LemmaItem => Some(self),
            _ => None,
        }
    }

    pub fn as_proof_block(self) -> Option<Self> {
        match &self.node.kind {
            SurfaceNodeKind::ProofBlock => Some(self),
            _ => None,
        }
    }

    pub fn as_definition_block_item(self) -> Option<Self> {
        match &self.node.kind {
            SurfaceNodeKind::DefinitionBlockItem => Some(self),
            _ => None,
        }
    }

    pub fn as_definition_parameter(self) -> Option<Self> {
        match &self.node.kind {
            SurfaceNodeKind::DefinitionParameter => Some(self),
            _ => None,
        }
    }

    pub fn as_template_parameter(self) -> Option<Self> {
        match &self.node.kind {
            SurfaceNodeKind::TemplateParameter => Some(self),
            _ => None,
        }
    }

    pub fn as_attribute_definition(self) -> Option<Self> {
        match &self.node.kind {
            SurfaceNodeKind::AttributeDefinition => Some(self),
            _ => None,
        }
    }

    pub fn as_attribute_pattern(self) -> Option<Self> {
        match &self.node.kind {
            SurfaceNodeKind::AttributePattern => Some(self),
            _ => None,
        }
    }

    pub fn as_formula_definiens(self) -> Option<Self> {
        match &self.node.kind {
            SurfaceNodeKind::FormulaDefiniens => Some(self),
            _ => None,
        }
    }

    pub fn as_formula_case(self) -> Option<Self> {
        match &self.node.kind {
            SurfaceNodeKind::FormulaCase => Some(self),
            _ => None,
        }
    }

    pub fn as_correctness_condition(self) -> Option<Self> {
        match &self.node.kind {
            SurfaceNodeKind::CorrectnessCondition => Some(self),
            _ => None,
        }
    }

    pub fn as_predicate_definition(self) -> Option<Self> {
        match &self.node.kind {
            SurfaceNodeKind::PredicateDefinition => Some(self),
            _ => None,
        }
    }

    pub fn as_predicate_pattern(self) -> Option<Self> {
        match &self.node.kind {
            SurfaceNodeKind::PredicatePattern => Some(self),
            _ => None,
        }
    }

    pub fn as_functor_definition(self) -> Option<Self> {
        match &self.node.kind {
            SurfaceNodeKind::FunctorDefinition => Some(self),
            _ => None,
        }
    }

    pub fn as_functor_pattern(self) -> Option<Self> {
        match &self.node.kind {
            SurfaceNodeKind::FunctorPattern => Some(self),
            _ => None,
        }
    }

    pub fn as_term_definiens(self) -> Option<Self> {
        match &self.node.kind {
            SurfaceNodeKind::TermDefiniens => Some(self),
            _ => None,
        }
    }

    pub fn as_term_case(self) -> Option<Self> {
        match &self.node.kind {
            SurfaceNodeKind::TermCase => Some(self),
            _ => None,
        }
    }

    pub fn as_mode_definition(self) -> Option<Self> {
        match &self.node.kind {
            SurfaceNodeKind::ModeDefinition => Some(self),
            _ => None,
        }
    }

    pub fn as_mode_pattern(self) -> Option<Self> {
        match &self.node.kind {
            SurfaceNodeKind::ModePattern => Some(self),
            _ => None,
        }
    }

    pub fn as_mode_property(self) -> Option<Self> {
        match &self.node.kind {
            SurfaceNodeKind::ModeProperty => Some(self),
            _ => None,
        }
    }

    pub fn as_attribute_redefinition(self) -> Option<Self> {
        match &self.node.kind {
            SurfaceNodeKind::AttributeRedefinition => Some(self),
            _ => None,
        }
    }

    pub fn as_predicate_redefinition(self) -> Option<Self> {
        match &self.node.kind {
            SurfaceNodeKind::PredicateRedefinition => Some(self),
            _ => None,
        }
    }

    pub fn as_functor_redefinition(self) -> Option<Self> {
        match &self.node.kind {
            SurfaceNodeKind::FunctorRedefinition => Some(self),
            _ => None,
        }
    }

    pub fn as_coherence_condition(self) -> Option<Self> {
        match &self.node.kind {
            SurfaceNodeKind::CoherenceCondition => Some(self),
            _ => None,
        }
    }

    pub fn as_notation_alias(self) -> Option<Self> {
        match &self.node.kind {
            SurfaceNodeKind::NotationAlias => Some(self),
            _ => None,
        }
    }

    pub fn as_notation_pattern(self) -> Option<Self> {
        match &self.node.kind {
            SurfaceNodeKind::NotationPattern => Some(self),
            _ => None,
        }
    }

    pub fn as_property_clause(self) -> Option<Self> {
        match &self.node.kind {
            SurfaceNodeKind::PropertyClause => Some(self),
            _ => None,
        }
    }

    pub fn as_structure_definition(self) -> Option<Self> {
        match &self.node.kind {
            SurfaceNodeKind::StructureDefinition => Some(self),
            _ => None,
        }
    }

    pub fn as_structure_pattern(self) -> Option<Self> {
        match &self.node.kind {
            SurfaceNodeKind::StructurePattern => Some(self),
            _ => None,
        }
    }

    pub fn as_structure_field(self) -> Option<Self> {
        match &self.node.kind {
            SurfaceNodeKind::StructureField => Some(self),
            _ => None,
        }
    }

    pub fn as_structure_property(self) -> Option<Self> {
        match &self.node.kind {
            SurfaceNodeKind::StructureProperty => Some(self),
            _ => None,
        }
    }

    pub fn as_inheritance_definition(self) -> Option<Self> {
        match &self.node.kind {
            SurfaceNodeKind::InheritanceDefinition => Some(self),
            _ => None,
        }
    }

    pub fn as_inheritance_target(self) -> Option<Self> {
        match &self.node.kind {
            SurfaceNodeKind::InheritanceTarget => Some(self),
            _ => None,
        }
    }

    pub fn as_field_redefinition(self) -> Option<Self> {
        match &self.node.kind {
            SurfaceNodeKind::FieldRedefinition => Some(self),
            _ => None,
        }
    }

    pub fn as_property_redefinition(self) -> Option<Self> {
        match &self.node.kind {
            SurfaceNodeKind::PropertyRedefinition => Some(self),
            _ => None,
        }
    }

    pub fn as_registration_block_item(self) -> Option<Self> {
        match &self.node.kind {
            SurfaceNodeKind::RegistrationBlockItem => Some(self),
            _ => None,
        }
    }

    pub fn as_registration_parameter(self) -> Option<Self> {
        match &self.node.kind {
            SurfaceNodeKind::RegistrationParameter => Some(self),
            _ => None,
        }
    }

    pub fn as_existential_registration(self) -> Option<Self> {
        match &self.node.kind {
            SurfaceNodeKind::ExistentialRegistration => Some(self),
            _ => None,
        }
    }

    pub fn as_conditional_registration(self) -> Option<Self> {
        match &self.node.kind {
            SurfaceNodeKind::ConditionalRegistration => Some(self),
            _ => None,
        }
    }

    pub fn as_functorial_registration(self) -> Option<Self> {
        match &self.node.kind {
            SurfaceNodeKind::FunctorialRegistration => Some(self),
            _ => None,
        }
    }

    pub fn as_reduction_registration(self) -> Option<Self> {
        match &self.node.kind {
            SurfaceNodeKind::ReductionRegistration => Some(self),
            _ => None,
        }
    }

    pub fn as_let_statement(self) -> Option<Self> {
        match &self.node.kind {
            SurfaceNodeKind::LetStatement => Some(self),
            _ => None,
        }
    }

    pub fn as_qualified_variable_segment(self) -> Option<Self> {
        match &self.node.kind {
            SurfaceNodeKind::QualifiedVariableSegment => Some(self),
            _ => None,
        }
    }

    pub fn as_assumption_statement(self) -> Option<Self> {
        match &self.node.kind {
            SurfaceNodeKind::AssumptionStatement => Some(self),
            _ => None,
        }
    }

    pub fn as_proposition(self) -> Option<Self> {
        match &self.node.kind {
            SurfaceNodeKind::Proposition => Some(self),
            _ => None,
        }
    }

    pub fn as_condition_list(self) -> Option<Self> {
        match &self.node.kind {
            SurfaceNodeKind::ConditionList => Some(self),
            _ => None,
        }
    }

    pub fn as_given_statement(self) -> Option<Self> {
        match &self.node.kind {
            SurfaceNodeKind::GivenStatement => Some(self),
            _ => None,
        }
    }

    pub fn as_take_statement(self) -> Option<Self> {
        match &self.node.kind {
            SurfaceNodeKind::TakeStatement => Some(self),
            _ => None,
        }
    }

    pub fn as_witness(self) -> Option<Self> {
        match &self.node.kind {
            SurfaceNodeKind::Witness => Some(self),
            _ => None,
        }
    }

    pub fn as_set_statement(self) -> Option<Self> {
        match &self.node.kind {
            SurfaceNodeKind::SetStatement => Some(self),
            _ => None,
        }
    }

    pub fn as_equating(self) -> Option<Self> {
        match &self.node.kind {
            SurfaceNodeKind::Equating => Some(self),
            _ => None,
        }
    }

    pub fn as_compact_statement(self) -> Option<Self> {
        match &self.node.kind {
            SurfaceNodeKind::CompactStatement => Some(self),
            _ => None,
        }
    }

    pub fn as_justification_clause(self) -> Option<Self> {
        match &self.node.kind {
            SurfaceNodeKind::JustificationClause => Some(self),
            _ => None,
        }
    }

    pub fn as_reference_list(self) -> Option<Self> {
        match &self.node.kind {
            SurfaceNodeKind::ReferenceList => Some(self),
            _ => None,
        }
    }

    pub fn as_reference(self) -> Option<Self> {
        match &self.node.kind {
            SurfaceNodeKind::Reference => Some(self),
            _ => None,
        }
    }

    pub fn as_qualified_reference(self) -> Option<Self> {
        match &self.node.kind {
            SurfaceNodeKind::QualifiedReference => Some(self),
            _ => None,
        }
    }

    pub fn as_grouped_reference(self) -> Option<Self> {
        match &self.node.kind {
            SurfaceNodeKind::GroupedReference => Some(self),
            _ => None,
        }
    }

    pub fn as_grouped_reference_item(self) -> Option<Self> {
        match &self.node.kind {
            SurfaceNodeKind::GroupedReferenceItem => Some(self),
            _ => None,
        }
    }

    pub fn as_bulk_reference(self) -> Option<Self> {
        match &self.node.kind {
            SurfaceNodeKind::BulkReference => Some(self),
            _ => None,
        }
    }

    pub fn as_computation_justification(self) -> Option<Self> {
        match &self.node.kind {
            SurfaceNodeKind::ComputationJustification => Some(self),
            _ => None,
        }
    }

    pub fn as_computation_option(self) -> Option<Self> {
        match &self.node.kind {
            SurfaceNodeKind::ComputationOption => Some(self),
            _ => None,
        }
    }

    pub fn as_consider_statement(self) -> Option<Self> {
        match &self.node.kind {
            SurfaceNodeKind::ConsiderStatement => Some(self),
            _ => None,
        }
    }

    pub fn as_reconsider_statement(self) -> Option<Self> {
        match &self.node.kind {
            SurfaceNodeKind::ReconsiderStatement => Some(self),
            _ => None,
        }
    }

    pub fn as_reconsider_item(self) -> Option<Self> {
        match &self.node.kind {
            SurfaceNodeKind::ReconsiderItem => Some(self),
            _ => None,
        }
    }

    pub fn as_conclusion_statement(self) -> Option<Self> {
        match &self.node.kind {
            SurfaceNodeKind::ConclusionStatement => Some(self),
            _ => None,
        }
    }

    pub fn as_then_statement(self) -> Option<Self> {
        match &self.node.kind {
            SurfaceNodeKind::ThenStatement => Some(self),
            _ => None,
        }
    }

    pub fn as_iterative_equality_statement(self) -> Option<Self> {
        match &self.node.kind {
            SurfaceNodeKind::IterativeEqualityStatement => Some(self),
            _ => None,
        }
    }

    pub fn as_iterative_equality_step(self) -> Option<Self> {
        match &self.node.kind {
            SurfaceNodeKind::IterativeEqualityStep => Some(self),
            _ => None,
        }
    }

    pub fn as_now_statement(self) -> Option<Self> {
        match &self.node.kind {
            SurfaceNodeKind::NowStatement => Some(self),
            _ => None,
        }
    }

    pub fn as_hereby_statement(self) -> Option<Self> {
        match &self.node.kind {
            SurfaceNodeKind::HerebyStatement => Some(self),
            _ => None,
        }
    }

    pub fn as_case_reasoning_statement(self) -> Option<Self> {
        match &self.node.kind {
            SurfaceNodeKind::CaseReasoningStatement => Some(self),
            _ => None,
        }
    }

    pub fn as_case_item(self) -> Option<Self> {
        match &self.node.kind {
            SurfaceNodeKind::CaseItem => Some(self),
            _ => None,
        }
    }

    pub fn as_suppose_item(self) -> Option<Self> {
        match &self.node.kind {
            SurfaceNodeKind::SupposeItem => Some(self),
            _ => None,
        }
    }

    pub fn as_inline_functor_definition(self) -> Option<Self> {
        match &self.node.kind {
            SurfaceNodeKind::InlineFunctorDefinition => Some(self),
            _ => None,
        }
    }

    pub fn as_inline_predicate_definition(self) -> Option<Self> {
        match &self.node.kind {
            SurfaceNodeKind::InlinePredicateDefinition => Some(self),
            _ => None,
        }
    }

    pub fn as_typed_parameter(self) -> Option<Self> {
        match &self.node.kind {
            SurfaceNodeKind::TypedParameter => Some(self),
            _ => None,
        }
    }

    pub fn as_selector_access(self) -> Option<Self> {
        match &self.node.kind {
            SurfaceNodeKind::SelectorAccess => Some(self),
            _ => None,
        }
    }

    pub fn as_structure_update(self) -> Option<Self> {
        match &self.node.kind {
            SurfaceNodeKind::StructureUpdate => Some(self),
            _ => None,
        }
    }

    pub fn as_field_update(self) -> Option<Self> {
        match &self.node.kind {
            SurfaceNodeKind::FieldUpdate => Some(self),
            _ => None,
        }
    }

    pub fn as_qua_expression(self) -> Option<Self> {
        match &self.node.kind {
            SurfaceNodeKind::QuaExpression => Some(self),
            _ => None,
        }
    }

    pub fn as_formula_expression(self) -> Option<Self> {
        match &self.node.kind {
            SurfaceNodeKind::FormulaExpression => Some(self),
            _ => None,
        }
    }

    pub fn as_builtin_predicate_application(self) -> Option<Self> {
        match &self.node.kind {
            SurfaceNodeKind::BuiltinPredicateApplication => Some(self),
            _ => None,
        }
    }

    pub fn as_is_assertion(self) -> Option<Self> {
        match &self.node.kind {
            SurfaceNodeKind::IsAssertion => Some(self),
            _ => None,
        }
    }

    pub fn as_attribute_test_chain(self) -> Option<Self> {
        match &self.node.kind {
            SurfaceNodeKind::AttributeTestChain => Some(self),
            _ => None,
        }
    }

    pub fn as_predicate_application(self) -> Option<Self> {
        match &self.node.kind {
            SurfaceNodeKind::PredicateApplication => Some(self),
            _ => None,
        }
    }

    pub fn as_predicate_segment(self) -> Option<Self> {
        match &self.node.kind {
            SurfaceNodeKind::PredicateSegment => Some(self),
            _ => None,
        }
    }

    pub fn as_predicate_head(self) -> Option<Self> {
        match &self.node.kind {
            SurfaceNodeKind::PredicateHead => Some(self),
            _ => None,
        }
    }

    pub fn as_inline_predicate_application(self) -> Option<Self> {
        match &self.node.kind {
            SurfaceNodeKind::InlinePredicateApplication => Some(self),
            _ => None,
        }
    }

    pub fn as_prefix_formula(self) -> Option<Self> {
        match &self.node.kind {
            SurfaceNodeKind::PrefixFormula(_) => Some(self),
            _ => None,
        }
    }

    pub fn as_binary_formula(self) -> Option<Self> {
        match &self.node.kind {
            SurfaceNodeKind::BinaryFormula(_) => Some(self),
            _ => None,
        }
    }

    pub fn as_parenthesized_formula(self) -> Option<Self> {
        match &self.node.kind {
            SurfaceNodeKind::ParenthesizedFormula => Some(self),
            _ => None,
        }
    }

    pub fn as_quantified_formula(self) -> Option<Self> {
        match &self.node.kind {
            SurfaceNodeKind::QuantifiedFormula(_) => Some(self),
            _ => None,
        }
    }

    pub fn as_quantifier_variable_segment(self) -> Option<Self> {
        match &self.node.kind {
            SurfaceNodeKind::QuantifierVariableSegment => Some(self),
            _ => None,
        }
    }

    pub fn as_formula_constant(self) -> Option<Self> {
        match &self.node.kind {
            SurfaceNodeKind::FormulaConstant(_) => Some(self),
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
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum SurfaceNodeKind {
    Root,
    Token(SurfaceToken),
    InfixExpression(SurfaceInfixOperator),
    PrefixExpression(SurfacePrefixOperator),
    PostfixExpression(SurfacePostfixOperator),
    FormulaExpression,
    BuiltinPredicateApplication,
    IsAssertion,
    AttributeTestChain,
    PredicateApplication,
    PredicateSegment,
    PredicateHead,
    InlinePredicateApplication,
    PrefixFormula(SurfaceFormulaPrefixOperator),
    BinaryFormula(SurfaceFormulaBinaryOperator),
    ParenthesizedFormula,
    QuantifiedFormula(SurfaceQuantifierKind),
    QuantifierVariableSegment,
    FormulaConstant(SurfaceFormulaConstant),
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
    TemplateLoci,
    TemplateLocus,
    TemplateArguments,
    TemplateArgument,
    AlgorithmDefinition,
    AlgorithmParameters,
    AlgorithmBody,
    AlgorithmStatementList,
    VariableDeclaration,
    VariableBinding,
    AssignmentStatement,
    Lvalue,
    SnapshotStatement,
    ReturnStatement,
    ClaimBlockItem,
    IfStatement,
    WhileStatement,
    ForRangeStatement,
    ForCollectionStatement,
    MatchStatement,
    MatchCase,
    MatchEnding,
    BreakStatement,
    ContinueStatement,
    AlgorithmTerminationClause,
    AlgorithmRequiresClause,
    AlgorithmEnsuresClause,
    AlgorithmDecreasingClause,
    LoopInvariantClause,
    LoopDecreasingClause,
    AssertStatement,
    TermList,
    Annotation,
    LibraryAnnotation,
    AnnotationLabelList,
    AnnotationLabel,
    AnnotationArgumentList,
    AnnotationArgument,
    ProofHintOptionList,
    ProofHintOption,
    StandaloneDiagnosticAnnotation,
    AnnotatedStatement,
    AnnotatedAlgorithmStatement,
    AnnotatedDefinitionContent,
    AnnotatedRegistrationContent,
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
    SetComprehension,
    ComprehensionVariableSegment,
    StatementItem,
    LetStatement,
    QualifiedVariableSegment,
    AssumptionStatement,
    Proposition,
    ConditionList,
    GivenStatement,
    TakeStatement,
    Witness,
    SetStatement,
    Equating,
    CompactStatement,
    JustificationClause,
    ReferenceList,
    Reference,
    QualifiedReference,
    GroupedReference,
    GroupedReferenceItem,
    BulkReference,
    ComputationJustification,
    ComputationOption,
    ConsiderStatement,
    ReconsiderStatement,
    ReconsiderItem,
    ConclusionStatement,
    ThenStatement,
    IterativeEqualityStatement,
    IterativeEqualityStep,
    NowStatement,
    HerebyStatement,
    CaseReasoningStatement,
    CaseItem,
    SupposeItem,
    InlineFunctorDefinition,
    InlinePredicateDefinition,
    TypedParameter,
    TheoremItem,
    LemmaItem,
    ProofBlock,
    DefinitionBlockItem,
    DefinitionParameter,
    TemplateParameter,
    AttributeDefinition,
    AttributePattern,
    FormulaDefiniens,
    FormulaCase,
    CorrectnessCondition,
    PredicateDefinition,
    PredicatePattern,
    FunctorDefinition,
    FunctorPattern,
    TermDefiniens,
    TermCase,
    ModeDefinition,
    ModePattern,
    ModeProperty,
    AttributeRedefinition,
    PredicateRedefinition,
    FunctorRedefinition,
    CoherenceCondition,
    NotationAlias,
    NotationPattern,
    PropertyClause,
    StructureDefinition,
    StructurePattern,
    StructureField,
    StructureProperty,
    InheritanceDefinition,
    InheritanceTarget,
    FieldRedefinition,
    PropertyRedefinition,
    RegistrationBlockItem,
    RegistrationParameter,
    ExistentialRegistration,
    ConditionalRegistration,
    FunctorialRegistration,
    ReductionRegistration,
    SelectorAccess,
    StructureUpdate,
    FieldUpdate,
    QuaExpression,
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
            Self::PrefixExpression(_) => SyntaxKind::PrefixExpression,
            Self::PostfixExpression(_) => SyntaxKind::PostfixExpression,
            Self::FormulaExpression => SyntaxKind::FormulaExpression,
            Self::BuiltinPredicateApplication => SyntaxKind::BuiltinPredicateApplication,
            Self::IsAssertion => SyntaxKind::IsAssertion,
            Self::AttributeTestChain => SyntaxKind::AttributeTestChain,
            Self::PredicateApplication => SyntaxKind::PredicateApplication,
            Self::PredicateSegment => SyntaxKind::PredicateSegment,
            Self::PredicateHead => SyntaxKind::PredicateHead,
            Self::InlinePredicateApplication => SyntaxKind::InlinePredicateApplication,
            Self::PrefixFormula(_) => SyntaxKind::PrefixFormula,
            Self::BinaryFormula(_) => SyntaxKind::BinaryFormula,
            Self::ParenthesizedFormula => SyntaxKind::ParenthesizedFormula,
            Self::QuantifiedFormula(_) => SyntaxKind::QuantifiedFormula,
            Self::QuantifierVariableSegment => SyntaxKind::QuantifierVariableSegment,
            Self::FormulaConstant(_) => SyntaxKind::FormulaConstant,
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
            Self::TemplateLoci => SyntaxKind::TemplateLoci,
            Self::TemplateLocus => SyntaxKind::TemplateLocus,
            Self::TemplateArguments => SyntaxKind::TemplateArguments,
            Self::TemplateArgument => SyntaxKind::TemplateArgument,
            Self::AlgorithmDefinition => SyntaxKind::AlgorithmDefinition,
            Self::AlgorithmParameters => SyntaxKind::AlgorithmParameters,
            Self::AlgorithmBody => SyntaxKind::AlgorithmBody,
            Self::AlgorithmStatementList => SyntaxKind::AlgorithmStatementList,
            Self::VariableDeclaration => SyntaxKind::VariableDeclaration,
            Self::VariableBinding => SyntaxKind::VariableBinding,
            Self::AssignmentStatement => SyntaxKind::AssignmentStatement,
            Self::Lvalue => SyntaxKind::Lvalue,
            Self::SnapshotStatement => SyntaxKind::SnapshotStatement,
            Self::ReturnStatement => SyntaxKind::ReturnStatement,
            Self::ClaimBlockItem => SyntaxKind::ClaimBlockItem,
            Self::IfStatement => SyntaxKind::IfStatement,
            Self::WhileStatement => SyntaxKind::WhileStatement,
            Self::ForRangeStatement => SyntaxKind::ForRangeStatement,
            Self::ForCollectionStatement => SyntaxKind::ForCollectionStatement,
            Self::MatchStatement => SyntaxKind::MatchStatement,
            Self::MatchCase => SyntaxKind::MatchCase,
            Self::MatchEnding => SyntaxKind::MatchEnding,
            Self::BreakStatement => SyntaxKind::BreakStatement,
            Self::ContinueStatement => SyntaxKind::ContinueStatement,
            Self::AlgorithmTerminationClause => SyntaxKind::AlgorithmTerminationClause,
            Self::AlgorithmRequiresClause => SyntaxKind::AlgorithmRequiresClause,
            Self::AlgorithmEnsuresClause => SyntaxKind::AlgorithmEnsuresClause,
            Self::AlgorithmDecreasingClause => SyntaxKind::AlgorithmDecreasingClause,
            Self::LoopInvariantClause => SyntaxKind::LoopInvariantClause,
            Self::LoopDecreasingClause => SyntaxKind::LoopDecreasingClause,
            Self::AssertStatement => SyntaxKind::AssertStatement,
            Self::TermList => SyntaxKind::TermList,
            Self::Annotation => SyntaxKind::Annotation,
            Self::LibraryAnnotation => SyntaxKind::LibraryAnnotation,
            Self::AnnotationLabelList => SyntaxKind::AnnotationLabelList,
            Self::AnnotationLabel => SyntaxKind::AnnotationLabel,
            Self::AnnotationArgumentList => SyntaxKind::AnnotationArgumentList,
            Self::AnnotationArgument => SyntaxKind::AnnotationArgument,
            Self::ProofHintOptionList => SyntaxKind::ProofHintOptionList,
            Self::ProofHintOption => SyntaxKind::ProofHintOption,
            Self::StandaloneDiagnosticAnnotation => SyntaxKind::StandaloneDiagnosticAnnotation,
            Self::AnnotatedStatement => SyntaxKind::AnnotatedStatement,
            Self::AnnotatedAlgorithmStatement => SyntaxKind::AnnotatedAlgorithmStatement,
            Self::AnnotatedDefinitionContent => SyntaxKind::AnnotatedDefinitionContent,
            Self::AnnotatedRegistrationContent => SyntaxKind::AnnotatedRegistrationContent,
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
            Self::SetComprehension => SyntaxKind::SetComprehension,
            Self::ComprehensionVariableSegment => SyntaxKind::ComprehensionVariableSegment,
            Self::StatementItem => SyntaxKind::StatementItem,
            Self::LetStatement => SyntaxKind::LetStatement,
            Self::QualifiedVariableSegment => SyntaxKind::QualifiedVariableSegment,
            Self::AssumptionStatement => SyntaxKind::AssumptionStatement,
            Self::Proposition => SyntaxKind::Proposition,
            Self::ConditionList => SyntaxKind::ConditionList,
            Self::GivenStatement => SyntaxKind::GivenStatement,
            Self::TakeStatement => SyntaxKind::TakeStatement,
            Self::Witness => SyntaxKind::Witness,
            Self::SetStatement => SyntaxKind::SetStatement,
            Self::Equating => SyntaxKind::Equating,
            Self::CompactStatement => SyntaxKind::CompactStatement,
            Self::JustificationClause => SyntaxKind::JustificationClause,
            Self::ReferenceList => SyntaxKind::ReferenceList,
            Self::Reference => SyntaxKind::Reference,
            Self::QualifiedReference => SyntaxKind::QualifiedReference,
            Self::GroupedReference => SyntaxKind::GroupedReference,
            Self::GroupedReferenceItem => SyntaxKind::GroupedReferenceItem,
            Self::BulkReference => SyntaxKind::BulkReference,
            Self::ComputationJustification => SyntaxKind::ComputationJustification,
            Self::ComputationOption => SyntaxKind::ComputationOption,
            Self::ConsiderStatement => SyntaxKind::ConsiderStatement,
            Self::ReconsiderStatement => SyntaxKind::ReconsiderStatement,
            Self::ReconsiderItem => SyntaxKind::ReconsiderItem,
            Self::ConclusionStatement => SyntaxKind::ConclusionStatement,
            Self::ThenStatement => SyntaxKind::ThenStatement,
            Self::IterativeEqualityStatement => SyntaxKind::IterativeEqualityStatement,
            Self::IterativeEqualityStep => SyntaxKind::IterativeEqualityStep,
            Self::NowStatement => SyntaxKind::NowStatement,
            Self::HerebyStatement => SyntaxKind::HerebyStatement,
            Self::CaseReasoningStatement => SyntaxKind::CaseReasoningStatement,
            Self::CaseItem => SyntaxKind::CaseItem,
            Self::SupposeItem => SyntaxKind::SupposeItem,
            Self::InlineFunctorDefinition => SyntaxKind::InlineFunctorDefinition,
            Self::InlinePredicateDefinition => SyntaxKind::InlinePredicateDefinition,
            Self::TypedParameter => SyntaxKind::TypedParameter,
            Self::TheoremItem => SyntaxKind::TheoremItem,
            Self::LemmaItem => SyntaxKind::LemmaItem,
            Self::ProofBlock => SyntaxKind::ProofBlock,
            Self::DefinitionBlockItem => SyntaxKind::DefinitionBlockItem,
            Self::DefinitionParameter => SyntaxKind::DefinitionParameter,
            Self::TemplateParameter => SyntaxKind::TemplateParameter,
            Self::AttributeDefinition => SyntaxKind::AttributeDefinition,
            Self::AttributePattern => SyntaxKind::AttributePattern,
            Self::FormulaDefiniens => SyntaxKind::FormulaDefiniens,
            Self::FormulaCase => SyntaxKind::FormulaCase,
            Self::CorrectnessCondition => SyntaxKind::CorrectnessCondition,
            Self::PredicateDefinition => SyntaxKind::PredicateDefinition,
            Self::PredicatePattern => SyntaxKind::PredicatePattern,
            Self::FunctorDefinition => SyntaxKind::FunctorDefinition,
            Self::FunctorPattern => SyntaxKind::FunctorPattern,
            Self::TermDefiniens => SyntaxKind::TermDefiniens,
            Self::TermCase => SyntaxKind::TermCase,
            Self::ModeDefinition => SyntaxKind::ModeDefinition,
            Self::ModePattern => SyntaxKind::ModePattern,
            Self::ModeProperty => SyntaxKind::ModeProperty,
            Self::AttributeRedefinition => SyntaxKind::AttributeRedefinition,
            Self::PredicateRedefinition => SyntaxKind::PredicateRedefinition,
            Self::FunctorRedefinition => SyntaxKind::FunctorRedefinition,
            Self::CoherenceCondition => SyntaxKind::CoherenceCondition,
            Self::NotationAlias => SyntaxKind::NotationAlias,
            Self::NotationPattern => SyntaxKind::NotationPattern,
            Self::PropertyClause => SyntaxKind::PropertyClause,
            Self::StructureDefinition => SyntaxKind::StructureDefinition,
            Self::StructurePattern => SyntaxKind::StructurePattern,
            Self::StructureField => SyntaxKind::StructureField,
            Self::StructureProperty => SyntaxKind::StructureProperty,
            Self::InheritanceDefinition => SyntaxKind::InheritanceDefinition,
            Self::InheritanceTarget => SyntaxKind::InheritanceTarget,
            Self::FieldRedefinition => SyntaxKind::FieldRedefinition,
            Self::PropertyRedefinition => SyntaxKind::PropertyRedefinition,
            Self::RegistrationBlockItem => SyntaxKind::RegistrationBlockItem,
            Self::RegistrationParameter => SyntaxKind::RegistrationParameter,
            Self::ExistentialRegistration => SyntaxKind::ExistentialRegistration,
            Self::ConditionalRegistration => SyntaxKind::ConditionalRegistration,
            Self::FunctorialRegistration => SyntaxKind::FunctorialRegistration,
            Self::ReductionRegistration => SyntaxKind::ReductionRegistration,
            Self::SelectorAccess => SyntaxKind::SelectorAccess,
            Self::StructureUpdate => SyntaxKind::StructureUpdate,
            Self::FieldUpdate => SyntaxKind::FieldUpdate,
            Self::QuaExpression => SyntaxKind::QuaExpression,
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
    AnnotationMarker,
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
            Self::AnnotationMarker => SyntaxKind::TokenAnnotationMarker,
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
            Self::AnnotationMarker => "AnnotationMarker",
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SurfacePrefixOperator {
    pub spelling: Arc<str>,
    pub precedence: u8,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SurfacePostfixOperator {
    pub spelling: Arc<str>,
    pub precedence: u8,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SurfaceFormulaPrefixOperator {
    Not,
}

impl SurfaceFormulaPrefixOperator {
    const fn snapshot_name(self) -> &'static str {
        match self {
            Self::Not => "Not",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SurfaceFormulaConnective {
    And,
    Or,
    Implies,
    Iff,
}

impl SurfaceFormulaConnective {
    const fn snapshot_name(self) -> &'static str {
        match self {
            Self::And => "And",
            Self::Or => "Or",
            Self::Implies => "Implies",
            Self::Iff => "Iff",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SurfaceFormulaBinaryOperator {
    pub connective: SurfaceFormulaConnective,
    pub repeated: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SurfaceQuantifierKind {
    Universal,
    Existential,
}

impl SurfaceQuantifierKind {
    const fn snapshot_name(self) -> &'static str {
        match self {
            Self::Universal => "Universal",
            Self::Existential => "Existential",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SurfaceFormulaConstant {
    Thesis,
    Contradiction,
}

impl SurfaceFormulaConstant {
    const fn snapshot_name(self) -> &'static str {
        match self {
            Self::Thesis => "Thesis",
            Self::Contradiction => "Contradiction",
        }
    }
}

fn contains_range(parent: SourceRange, child: SourceRange) -> bool {
    parent.source_id == child.source_id && parent.start <= child.start && child.end <= parent.end
}

#[cfg(test)]
fn recovery_snapshot_name(kind: SyntaxRecoveryKind) -> &'static str {
    snapshot::recovery_snapshot_name(kind)
}

#[cfg(test)]
mod tests;
