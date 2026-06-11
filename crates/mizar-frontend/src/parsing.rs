//! Parser input assembly and parser seam integration.
//!
//! Canonical behavior is specified in the
//! [parsing design spec](../../../../doc/design/mizar-frontend/en/parsing.md).

use crate::lexical_env::{ActiveLexicalEnvironment, SymbolId};
use crate::lexing::{ParserLexContext, ParserLexingPlan, TokenKind, TokenStream};
use mizar_session::Edition;
use std::sync::Arc;

/// Default cache-key version for custom parser seam implementations.
pub const DEFAULT_PARSER_CACHE_KEY_VERSION: &str = "mizar-frontend/parser-seam/custom-v1";
/// Cache-key version used by the stub parser seam.
pub const STUB_PARSER_CACHE_KEY_VERSION: &str = "mizar-frontend/stub-parser/no-ast-v1";
/// Cache-key version used by the real Mizar parser seam.
pub const MIZAR_PARSER_CACHE_KEY_VERSION: &str = "mizar-parser/surface-ast-v1";

/// Parser request containing a token stream and parser inputs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseRequest<'a> {
    /// Token stream to parse.
    pub tokens: &'a TokenStream,
    /// Parser inputs derived from the active lexical environment.
    pub parser_inputs: ParserInputs,
}

impl<'a> ParseRequest<'a> {
    /// Creates a parser request.
    pub fn new(tokens: &'a TokenStream, parser_inputs: ParserInputs) -> Self {
        Self {
            tokens,
            parser_inputs,
        }
    }
}

/// Inputs assembled for a parser seam.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParserInputs {
    /// Edition of the source being parsed.
    pub edition: Edition,
    /// Operator fixity table visible to the parser.
    pub operator_fixity: OperatorFixityTable,
    /// String-literal context requirements for parser-assisted lexing.
    pub string_required_positions: StringRequiredContext,
}

impl ParserInputs {
    /// Creates parser inputs explicitly.
    pub fn new(
        edition: Edition,
        operator_fixity: OperatorFixityTable,
        string_required_positions: StringRequiredContext,
    ) -> Self {
        Self {
            edition,
            operator_fixity,
            string_required_positions,
        }
    }

    /// Derives parser inputs from the active lexical environment.
    pub fn from_active_environment(
        edition: Edition,
        _environment: &ActiveLexicalEnvironment,
    ) -> Self {
        Self {
            edition,
            operator_fixity: OperatorFixityTable::empty(),
            string_required_positions: StringRequiredContext::PositionSensitive,
        }
    }
}

/// Operator fixity entries supplied to the parser.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct OperatorFixityTable {
    /// Operator fixity entries in deterministic order.
    pub entries: Vec<OperatorFixityEntry>,
}

impl OperatorFixityTable {
    /// Returns an empty fixity table.
    pub fn empty() -> Self {
        Self::default()
    }

    /// Returns whether the table has no entries.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

/// One operator fixity entry supplied to the parser.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OperatorFixityEntry {
    /// Symbol id of the operator.
    pub symbol_id: SymbolId,
    /// Operator spelling.
    pub spelling: Arc<str>,
    /// Pratt precedence assigned to the operator.
    pub precedence: u8,
    /// Operator associativity.
    pub associativity: OperatorAssociativity,
}

/// Operator associativity used by parser fixity entries.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OperatorAssociativity {
    /// Left-associative operator.
    Left,
    /// Right-associative operator.
    Right,
    /// Non-associative operator.
    NonAssociative,
}

/// String-literal context policy used for parser-assisted lexing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum StringRequiredContext {
    /// No string-required lexing contexts are requested.
    #[default]
    None,
    /// String-required ranges are discovered from lexical text positions.
    PositionSensitive,
    /// A uniform string-required context used by tests.
    UniformForTest,
}

impl StringRequiredContext {
    /// Returns the default parser lexing context for this policy.
    pub fn parser_lex_context(self) -> ParserLexContext {
        match self {
            Self::None | Self::PositionSensitive => ParserLexContext::general(),
            Self::UniformForTest => ParserLexContext::string_required(),
        }
    }

    /// Builds the parser lexing plan for this policy and lexical text.
    pub fn parser_lexing_plan(self, lexical_text: &str) -> ParserLexingPlan {
        match self {
            Self::None => ParserLexingPlan::uniform(ParserLexContext::general()),
            Self::PositionSensitive => ParserLexingPlan::for_lexical_text(lexical_text),
            Self::UniformForTest => ParserLexingPlan::uniform(ParserLexContext::string_required()),
        }
    }
}

/// Parser output with optional AST and parser diagnostics.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseOutput<A, D> {
    /// Parsed AST, absent for unrecoverable parse failures or stub seams.
    pub ast: Option<A>,
    /// Parser diagnostics in parser emission order.
    pub diagnostics: Vec<D>,
}

impl<A, D> ParseOutput<A, D> {
    /// Creates parser output from an optional AST and diagnostics.
    pub fn new(ast: Option<A>, diagnostics: Vec<D>) -> Self {
        Self { ast, diagnostics }
    }
}

/// Parser seam consumed by frontend orchestration.
pub trait ParserSeam {
    /// AST type returned by the parser seam.
    type Ast;
    /// Diagnostic type returned by the parser seam.
    type Diagnostic;

    /// Returns the parser cache-key version for this seam.
    fn cache_key_version(&self) -> ParserCacheKeyVersion {
        ParserCacheKeyVersion::new(DEFAULT_PARSER_CACHE_KEY_VERSION)
    }

    /// Parses a token stream.
    fn parse(&self, request: ParseRequest<'_>) -> ParseOutput<Self::Ast, Self::Diagnostic>;
}

/// Version tag included in surface-AST cache keys for parser outputs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParserCacheKeyVersion {
    /// Parser cache-key version string.
    pub version: Arc<str>,
}

impl ParserCacheKeyVersion {
    /// Creates a parser cache-key version.
    pub fn new(version: impl Into<Arc<str>>) -> Self {
        Self {
            version: version.into(),
        }
    }
}

/// Parser seam that never produces an AST or diagnostics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct StubParserSeam;

impl ParserSeam for StubParserSeam {
    type Ast = ();
    type Diagnostic = ();

    fn cache_key_version(&self) -> ParserCacheKeyVersion {
        ParserCacheKeyVersion::new(STUB_PARSER_CACHE_KEY_VERSION)
    }

    fn parse(&self, _request: ParseRequest<'_>) -> ParseOutput<Self::Ast, Self::Diagnostic> {
        ParseOutput {
            ast: None,
            diagnostics: Vec::new(),
        }
    }
}

/// Parser seam that adapts frontend tokens into `mizar-parser`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct MizarParserSeam;

impl ParserSeam for MizarParserSeam {
    type Ast = mizar_syntax::SurfaceAst;
    type Diagnostic = mizar_syntax::SyntaxDiagnostic;

    fn cache_key_version(&self) -> ParserCacheKeyVersion {
        ParserCacheKeyVersion::new(MIZAR_PARSER_CACHE_KEY_VERSION)
    }

    fn parse(&self, request: ParseRequest<'_>) -> ParseOutput<Self::Ast, Self::Diagnostic> {
        let parser_request = mizar_parser::ParseRequest::new(
            request.tokens.source_id,
            request.parser_inputs.edition,
            request
                .tokens
                .tokens
                .iter()
                .map(parser_token)
                .collect::<Vec<_>>(),
            request
                .parser_inputs
                .operator_fixity
                .entries
                .into_iter()
                .map(parser_fixity)
                .collect(),
        )
        .with_string_required_context(parser_string_required_context(
            request.parser_inputs.string_required_positions,
        ));
        let output = mizar_parser::parse(parser_request);
        ParseOutput::new(output.ast, output.diagnostics)
    }
}

fn parser_token(token: &crate::lexing::Token) -> mizar_parser::ParserToken {
    mizar_parser::ParserToken::new(
        parser_token_kind(token.kind),
        token.text.clone(),
        token.span,
    )
}

fn parser_token_kind(kind: TokenKind) -> mizar_parser::ParserTokenKind {
    match kind {
        TokenKind::Identifier => mizar_parser::ParserTokenKind::Identifier,
        TokenKind::ReservedWord => mizar_parser::ParserTokenKind::ReservedWord,
        TokenKind::ReservedSymbol => mizar_parser::ParserTokenKind::ReservedSymbol,
        TokenKind::Numeral => mizar_parser::ParserTokenKind::Numeral,
        TokenKind::LexemeRun => mizar_parser::ParserTokenKind::LexemeRun,
        TokenKind::UserSymbol => mizar_parser::ParserTokenKind::UserSymbol,
        TokenKind::StringLiteral => mizar_parser::ParserTokenKind::StringLiteral,
        TokenKind::ErrorRecovery => mizar_parser::ParserTokenKind::ErrorRecovery,
        _ => mizar_parser::ParserTokenKind::Unknown,
    }
}

fn parser_fixity(entry: OperatorFixityEntry) -> mizar_parser::OperatorFixityEntry {
    mizar_parser::OperatorFixityEntry::new(
        entry.spelling,
        entry.precedence,
        parser_associativity(entry.associativity),
    )
}

fn parser_associativity(
    associativity: OperatorAssociativity,
) -> mizar_parser::OperatorAssociativity {
    match associativity {
        OperatorAssociativity::Left => mizar_parser::OperatorAssociativity::Left,
        OperatorAssociativity::Right => mizar_parser::OperatorAssociativity::Right,
        OperatorAssociativity::NonAssociative => {
            mizar_parser::OperatorAssociativity::NonAssociative
        }
    }
}

fn parser_string_required_context(
    context: StringRequiredContext,
) -> mizar_parser::StringRequiredContext {
    match context {
        StringRequiredContext::None => mizar_parser::StringRequiredContext::None,
        StringRequiredContext::PositionSensitive => mizar_parser::StringRequiredContext::None,
        StringRequiredContext::UniformForTest => {
            mizar_parser::StringRequiredContext::UniformForTest
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        MizarParserSeam, OperatorAssociativity, OperatorFixityEntry, OperatorFixityTable,
        ParseRequest, ParserInputs, ParserSeam, StringRequiredContext, StubParserSeam,
    };
    use crate::lexical_env::{ActiveLexicalEnvironment, ExportRank, ExportedSymbolShape};
    use crate::lexical_env::{
        LexicalSummaryFingerprint, ModuleId, ModuleLexicalSummary, ResolvedImport, SymbolId,
        UserSymbolArity, UserSymbolKind,
    };
    use crate::lexing::{
        ParserLexContext, ParserLexingPlan, ScopeView, Token, TokenKind, TokenStream,
    };
    use mizar_session::{
        BuildSnapshotId, Edition, Hash, InMemorySessionIdAllocator, SessionIdAllocator,
        SourceAnchor, SourceId, SourceRange,
    };
    use mizar_syntax::{
        SurfaceNodeKind, SurfaceOperatorAssociativity, SurfaceTokenKind, SyntaxDiagnosticCode,
        SyntaxRecoveryKind,
    };
    use std::sync::Arc;

    #[test]
    fn parser_inputs_carry_edition_and_currently_empty_fixity_table() {
        let edition = Edition::new("2026");
        let environment = environment_with_imported_symbol("++");

        let inputs = ParserInputs::from_active_environment(edition.clone(), &environment);

        assert_eq!(inputs.edition, edition);
        assert!(inputs.operator_fixity.is_empty());
        assert_eq!(
            inputs.string_required_positions,
            StringRequiredContext::PositionSensitive
        );
        assert_eq!(
            inputs.string_required_positions.parser_lex_context(),
            ParserLexContext::general()
        );
    }

    #[test]
    fn parser_inputs_do_not_carry_resolver_state_from_active_environment() {
        let environment = environment_with_imported_symbol("++");

        let inputs = ParserInputs::from_active_environment(Edition::new("2026"), &environment);

        assert!(
            environment.user_symbol("++").is_some(),
            "fixture must include resolver-visible symbol state"
        );
        assert!(
            inputs.operator_fixity.entries.is_empty(),
            "current lexical summaries expose no fixity data to carry into ParserInputs"
        );
    }

    #[test]
    fn uniform_test_string_context_maps_to_string_required_lexer_context() {
        assert_eq!(
            StringRequiredContext::UniformForTest.parser_lex_context(),
            ParserLexContext::string_required()
        );
    }

    #[test]
    fn position_sensitive_string_context_builds_parser_lexing_plan() {
        let plan = StringRequiredContext::PositionSensitive
            .parser_lexing_plan("@[label(\"name\")]\n\"symbol\"");

        assert_eq!(plan.default_context, ParserLexContext::general());
        assert_eq!(plan.contexts.len(), 1);
        assert_eq!(plan.contexts[0].range.start, "@[label(".len());
        assert_eq!(plan.contexts[0].range.end, "@[label(\"name\"".len());
        assert_eq!(
            plan.contexts[0].context,
            ParserLexContext::string_required()
        );
    }

    #[test]
    fn stub_parser_seam_returns_no_ast_and_no_diagnostics() {
        let environment = environment_without_imports();
        let inputs = ParserInputs::from_active_environment(Edition::new("2026"), &environment);
        let tokens = empty_token_stream(source_id(1));
        let seam = StubParserSeam;

        let output = seam.parse(ParseRequest::new(&tokens, inputs));

        assert_eq!(output.ast, None);
        assert!(output.diagnostics.is_empty());
    }

    #[test]
    fn real_parser_seam_returns_surface_ast_with_preserved_token_order_and_ranges() {
        let source_id = source_id(2);
        let tokens = token_stream(
            source_id,
            vec![
                token(source_id, TokenKind::Identifier, "alpha", 0, 5),
                token(source_id, TokenKind::ReservedSymbol, ";", 5, 6),
                token(source_id, TokenKind::Identifier, "beta", 7, 11),
            ],
        );
        let inputs = ParserInputs::new(
            Edition::new("2026"),
            OperatorFixityTable::empty(),
            StringRequiredContext::None,
        );
        let seam = MizarParserSeam;

        let output = seam.parse(ParseRequest::new(&tokens, inputs));

        assert!(output.diagnostics.is_empty());
        let ast = output
            .ast
            .expect("real parser seam should return SurfaceAst");
        assert_eq!(ast.token_texts(), vec!["alpha", ";", "beta"]);
        let ranges = ast
            .token_nodes
            .iter()
            .map(|id| ast.node(*id).unwrap().range)
            .collect::<Vec<_>>();
        assert_eq!(
            ranges,
            vec![
                SourceRange {
                    source_id,
                    start: 0,
                    end: 5,
                },
                SourceRange {
                    source_id,
                    start: 5,
                    end: 6,
                },
                SourceRange {
                    source_id,
                    start: 7,
                    end: 11,
                },
            ]
        );
    }

    #[test]
    fn real_parser_seam_preserves_token_kind_adaptation() {
        let source_id = source_id(3);
        let tokens = token_stream(
            source_id,
            vec![
                token(source_id, TokenKind::Identifier, "alpha", 0, 5),
                token(source_id, TokenKind::ReservedWord, "theorem", 6, 13),
                token(source_id, TokenKind::ReservedSymbol, ";", 13, 14),
                token(source_id, TokenKind::Numeral, "42", 15, 17),
                token(source_id, TokenKind::LexemeRun, "raw", 18, 21),
                token(source_id, TokenKind::UserSymbol, "++", 22, 24),
                token(source_id, TokenKind::StringLiteral, "\"x\"", 25, 28),
            ],
        );
        let inputs = ParserInputs::new(
            Edition::new("2026"),
            OperatorFixityTable::empty(),
            StringRequiredContext::None,
        );
        let seam = MizarParserSeam;

        let output = seam.parse(ParseRequest::new(&tokens, inputs));

        assert!(output.diagnostics.is_empty());
        let ast = output
            .ast
            .expect("real parser seam should return SurfaceAst");
        let kinds = ast
            .token_nodes
            .iter()
            .map(|id| match &ast.node(*id).unwrap().kind {
                SurfaceNodeKind::Token(token) => token.kind,
                other => panic!("expected token node, got {other:?}"),
            })
            .collect::<Vec<_>>();
        assert_eq!(
            kinds,
            vec![
                SurfaceTokenKind::Identifier,
                SurfaceTokenKind::ReservedWord,
                SurfaceTokenKind::ReservedSymbol,
                SurfaceTokenKind::Numeral,
                SurfaceTokenKind::LexemeRun,
                SurfaceTokenKind::UserSymbol,
                SurfaceTokenKind::StringLiteral,
            ]
        );
    }

    #[test]
    fn real_parser_seam_preserves_error_recovery_tokens_and_diagnostics() {
        let source_id = source_id(4);
        let tokens = token_stream(
            source_id,
            vec![token(source_id, TokenKind::ErrorRecovery, "@", 4, 5)],
        );
        let inputs = ParserInputs::new(
            Edition::new("2026"),
            OperatorFixityTable::empty(),
            StringRequiredContext::None,
        );
        let seam = MizarParserSeam;

        let output = seam.parse(ParseRequest::new(&tokens, inputs));

        assert_eq!(output.diagnostics.len(), 1);
        assert_eq!(
            output.diagnostics[0].code,
            SyntaxDiagnosticCode::UnexpectedErrorToken
        );
        assert_eq!(
            output.diagnostics[0].primary,
            SourceRange {
                source_id,
                start: 4,
                end: 5,
            }
        );
        let ast = output
            .ast
            .expect("real parser seam should preserve recovered token streams");
        assert_eq!(ast.token_texts(), vec!["@"]);
        let node = ast.node(ast.token_nodes[0]).unwrap();
        assert!(node.recovered);
        assert_eq!(
            node.range,
            SourceRange {
                source_id,
                start: 4,
                end: 5,
            }
        );
        let SurfaceNodeKind::Token(token) = &node.kind else {
            panic!("expected recovered token node");
        };
        assert_eq!(token.kind, SurfaceTokenKind::ErrorRecovery);
        assert_eq!(token.text.as_ref(), "@");
    }

    #[test]
    fn real_parser_seam_preserves_missing_end_recovery_nodes() {
        let source_id = source_id(9);
        let tokens = token_stream(
            source_id,
            vec![
                token(source_id, TokenKind::ReservedWord, "definition", 0, 10),
                token(source_id, TokenKind::ReservedWord, "theorem", 11, 18),
            ],
        );
        let inputs = ParserInputs::new(
            Edition::new("2026"),
            OperatorFixityTable::empty(),
            StringRequiredContext::None,
        );
        let seam = MizarParserSeam;

        let output = seam.parse(ParseRequest::new(&tokens, inputs));

        assert_eq!(output.diagnostics.len(), 1);
        let diagnostic = &output.diagnostics[0];
        assert_eq!(diagnostic.code, SyntaxDiagnosticCode::MissingEnd);
        assert_eq!(
            diagnostic.message.as_ref(),
            "missing `end` for `definition` block"
        );
        assert_eq!(
            diagnostic.primary,
            SourceRange {
                source_id,
                start: 18,
                end: 18,
            }
        );
        assert_eq!(
            diagnostic.secondary,
            vec![SourceAnchor::Range(SourceRange {
                source_id,
                start: 0,
                end: 10,
            })]
        );
        assert_eq!(
            diagnostic.recovery_note.as_deref(),
            Some("insert `end` before this synchronization point")
        );
        let ast = output
            .ast
            .expect("real parser seam should preserve recovered AST");
        let (recovery_index, recovery_node) = ast
            .nodes
            .iter()
            .enumerate()
            .find(|(_, node)| {
                matches!(
                    &node.kind,
                    SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::MissingEnd)
                )
            })
            .expect("missing end recovery node should pass through unchanged");
        assert!(recovery_node.recovered);
        assert_eq!(
            recovery_node.range,
            SourceRange {
                source_id,
                start: 18,
                end: 18,
            }
        );
        assert_eq!(recovery_node.children, vec![ast.token_nodes[0]]);
        let root = ast.root.expect("recovered AST should have a root");
        assert!(
            ast.node(root)
                .unwrap()
                .children
                .iter()
                .any(|child| child.index() == recovery_index),
            "missing end recovery node should remain root-reachable"
        );
    }

    #[test]
    fn real_parser_seam_preserves_unrecoverable_none_ast() {
        let source_id = source_id(10);
        let tokens = token_stream(
            source_id,
            vec![token(source_id, TokenKind::ReservedWord, "end", 0, 3)],
        );
        let inputs = ParserInputs::new(
            Edition::new("2026"),
            OperatorFixityTable::empty(),
            StringRequiredContext::None,
        );
        let seam = MizarParserSeam;

        let output = seam.parse(ParseRequest::new(&tokens, inputs));

        assert!(output.ast.is_none());
        assert_eq!(output.diagnostics.len(), 1);
        let diagnostic = &output.diagnostics[0];
        assert_eq!(diagnostic.code, SyntaxDiagnosticCode::UnrecoverableInput);
        assert_eq!(
            diagnostic.message.as_ref(),
            "`end` has no matching block opener"
        );
        assert_eq!(
            diagnostic.primary,
            SourceRange {
                source_id,
                start: 0,
                end: 3,
            }
        );
        assert_eq!(
            diagnostic.recovery_note.as_deref(),
            Some("remove the stray `end` or add a matching block opener before it")
        );
    }

    #[test]
    fn real_parser_seam_forwards_string_required_context() {
        let source_id = source_id(11);
        let tokens = token_stream(
            source_id,
            vec![token(
                source_id,
                TokenKind::Identifier,
                "not_a_string",
                0,
                12,
            )],
        );
        let inputs = ParserInputs::new(
            Edition::new("2026"),
            OperatorFixityTable::empty(),
            StringRequiredContext::UniformForTest,
        );
        let seam = MizarParserSeam;

        let output = seam.parse(ParseRequest::new(&tokens, inputs));

        assert_eq!(output.diagnostics.len(), 1);
        let diagnostic = &output.diagnostics[0];
        assert_eq!(diagnostic.code, SyntaxDiagnosticCode::MissingStringLiteral);
        assert_eq!(
            diagnostic.message.as_ref(),
            "expected string literal at this grammar position"
        );
        assert_eq!(
            diagnostic.primary,
            SourceRange {
                source_id,
                start: 0,
                end: 12,
            }
        );
        assert_eq!(
            diagnostic.recovery_note.as_deref(),
            Some("insert a string literal before continuing")
        );
        let ast = output
            .ast
            .expect("missing string literal should recover an AST");
        let (recovery_index, recovery_node) = ast
            .nodes
            .iter()
            .enumerate()
            .find(|(_, node)| {
                matches!(
                    &node.kind,
                    SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::MissingStringLiteral)
                )
            })
            .expect("missing string recovery node should pass through unchanged");
        assert!(recovery_node.recovered);
        assert_eq!(
            recovery_node.range,
            SourceRange {
                source_id,
                start: 0,
                end: 0,
            }
        );
        let root = ast.root.expect("recovered AST should have a root");
        assert!(
            ast.node(root)
                .unwrap()
                .children
                .iter()
                .any(|child| child.index() == recovery_index),
            "missing string recovery node should remain root-reachable"
        );
    }

    #[test]
    fn real_parser_seam_passes_operator_fixity_to_pratt_parser() {
        let source_id = source_id(5);
        let tokens = token_stream(
            source_id,
            vec![
                token(source_id, TokenKind::Identifier, "a", 0, 1),
                token(source_id, TokenKind::UserSymbol, "++", 2, 4),
                token(source_id, TokenKind::Identifier, "b", 5, 6),
                token(source_id, TokenKind::UserSymbol, "**", 7, 9),
                token(source_id, TokenKind::Identifier, "c", 10, 11),
            ],
        );
        let inputs = ParserInputs::new(
            Edition::new("2026"),
            OperatorFixityTable {
                entries: vec![
                    OperatorFixityEntry {
                        symbol_id: SymbolId::new("fixture.plus"),
                        spelling: Arc::<str>::from("++"),
                        precedence: 10,
                        associativity: OperatorAssociativity::Left,
                    },
                    OperatorFixityEntry {
                        symbol_id: SymbolId::new("fixture.pow"),
                        spelling: Arc::<str>::from("**"),
                        precedence: 20,
                        associativity: OperatorAssociativity::Right,
                    },
                ],
            },
            StringRequiredContext::None,
        );
        let seam = MizarParserSeam;

        let output = seam.parse(ParseRequest::new(&tokens, inputs));

        assert!(output.diagnostics.is_empty());
        let ast = output
            .ast
            .expect("real parser seam should return SurfaceAst");
        let root = ast
            .expression_root
            .expect("fixity should build an expression");
        let SurfaceNodeKind::InfixExpression(root_operator) = &ast.node(root).unwrap().kind else {
            panic!("expected infix expression root");
        };
        assert_eq!(root_operator.spelling.as_ref(), "++");
        assert_eq!(root_operator.precedence, 10);
        assert_eq!(
            root_operator.associativity,
            SurfaceOperatorAssociativity::Left
        );
        let right = ast.node(root).unwrap().children[2];
        let SurfaceNodeKind::InfixExpression(right_operator) = &ast.node(right).unwrap().kind
        else {
            panic!("expected higher-precedence right expression");
        };
        assert_eq!(right_operator.spelling.as_ref(), "**");
        assert_eq!(right_operator.precedence, 20);
        assert_eq!(
            right_operator.associativity,
            SurfaceOperatorAssociativity::Right
        );
    }

    #[test]
    fn real_parser_seam_forwards_right_associativity() {
        let source_id = source_id(6);
        let tokens = token_stream(
            source_id,
            vec![
                token(source_id, TokenKind::Identifier, "a", 0, 1),
                token(source_id, TokenKind::UserSymbol, "**", 2, 4),
                token(source_id, TokenKind::Identifier, "b", 5, 6),
                token(source_id, TokenKind::UserSymbol, "**", 7, 9),
                token(source_id, TokenKind::Identifier, "c", 10, 11),
            ],
        );
        let inputs = ParserInputs::new(
            Edition::new("2026"),
            OperatorFixityTable {
                entries: vec![OperatorFixityEntry {
                    symbol_id: SymbolId::new("fixture.pow"),
                    spelling: Arc::<str>::from("**"),
                    precedence: 20,
                    associativity: OperatorAssociativity::Right,
                }],
            },
            StringRequiredContext::None,
        );
        let seam = MizarParserSeam;

        let output = seam.parse(ParseRequest::new(&tokens, inputs));

        assert!(output.diagnostics.is_empty());
        let ast = output
            .ast
            .expect("real parser seam should return SurfaceAst");
        let root = ast
            .expression_root
            .expect("fixity should build an expression");
        let SurfaceNodeKind::InfixExpression(root_operator) = &ast.node(root).unwrap().kind else {
            panic!("expected infix expression root");
        };
        assert_eq!(root_operator.spelling.as_ref(), "**");
        assert_eq!(root_operator.precedence, 20);
        assert_eq!(
            root_operator.associativity,
            SurfaceOperatorAssociativity::Right
        );
        let right = ast.node(root).unwrap().children[2];
        let SurfaceNodeKind::InfixExpression(right_operator) = &ast.node(right).unwrap().kind
        else {
            panic!("expected right-associative child expression");
        };
        assert_eq!(right_operator.spelling.as_ref(), "**");
        assert_eq!(right_operator.precedence, 20);
        assert_eq!(
            right_operator.associativity,
            SurfaceOperatorAssociativity::Right
        );
    }

    #[test]
    fn real_parser_seam_forwards_non_associativity() {
        let source_id = source_id(7);
        let tokens = token_stream(
            source_id,
            vec![
                token(source_id, TokenKind::Identifier, "a", 0, 1),
                token(source_id, TokenKind::ReservedSymbol, "=", 2, 3),
                token(source_id, TokenKind::Identifier, "b", 4, 5),
                token(source_id, TokenKind::ReservedSymbol, "=", 6, 7),
                token(source_id, TokenKind::Identifier, "c", 8, 9),
            ],
        );
        let inputs = ParserInputs::new(
            Edition::new("2026"),
            OperatorFixityTable {
                entries: vec![OperatorFixityEntry {
                    symbol_id: SymbolId::new("fixture.equals"),
                    spelling: Arc::<str>::from("="),
                    precedence: 10,
                    associativity: OperatorAssociativity::NonAssociative,
                }],
            },
            StringRequiredContext::None,
        );
        let seam = MizarParserSeam;

        let output = seam.parse(ParseRequest::new(&tokens, inputs));

        assert_eq!(output.diagnostics.len(), 1);
        assert_eq!(
            output.diagnostics[0].code,
            SyntaxDiagnosticCode::NonAssociativeOperatorChain
        );
        assert_eq!(
            output.diagnostics[0].primary,
            SourceRange {
                source_id,
                start: 6,
                end: 7,
            }
        );
        let ast = output
            .ast
            .expect("real parser seam should return SurfaceAst");
        let root = ast
            .expression_root
            .expect("fixity should build an expression");
        let SurfaceNodeKind::InfixExpression(root_operator) = &ast.node(root).unwrap().kind else {
            panic!("expected infix expression root");
        };
        assert_eq!(root_operator.spelling.as_ref(), "=");
        assert_eq!(root_operator.precedence, 10);
        assert_eq!(
            root_operator.associativity,
            SurfaceOperatorAssociativity::NonAssociative
        );
    }

    #[test]
    fn real_parser_seam_returns_syntax_diagnostics_unchanged() {
        let source_id = source_id(8);
        let tokens = token_stream(
            source_id,
            vec![
                token(source_id, TokenKind::Identifier, "a", 0, 1),
                token(source_id, TokenKind::UserSymbol, "++", 2, 4),
            ],
        );
        let inputs = ParserInputs::new(
            Edition::new("2026"),
            OperatorFixityTable {
                entries: vec![OperatorFixityEntry {
                    symbol_id: SymbolId::new("fixture.plus"),
                    spelling: Arc::<str>::from("++"),
                    precedence: 10,
                    associativity: OperatorAssociativity::Left,
                }],
            },
            StringRequiredContext::None,
        );
        let seam = MizarParserSeam;

        let output = seam.parse(ParseRequest::new(&tokens, inputs));

        let diagnostic = output
            .diagnostics
            .first()
            .expect("dangling operator should produce a syntax diagnostic");
        assert_eq!(output.diagnostics.len(), 1);
        assert_eq!(diagnostic.code, SyntaxDiagnosticCode::DanglingOperator);
        assert_eq!(diagnostic.message.as_ref(), "operator has no right operand");
        assert_eq!(
            diagnostic.primary,
            SourceRange {
                source_id,
                start: 2,
                end: 4,
            }
        );
        assert!(diagnostic.secondary.is_empty());
        assert_eq!(diagnostic.recovery_note, None);
    }

    fn environment_without_imports() -> ActiveLexicalEnvironment {
        mizar_lexer::build_lexical_environment(&[], &[]).unwrap()
    }

    fn environment_with_imported_symbol(spelling: &str) -> ActiveLexicalEnvironment {
        mizar_lexer::build_lexical_environment(
            &[ResolvedImport {
                module_id: ModuleId::new("fixture"),
            }],
            &[ModuleLexicalSummary {
                module_id: ModuleId::new("fixture"),
                exported_symbols: vec![ExportedSymbolShape {
                    spelling: spelling.to_owned(),
                    symbol_id: SymbolId::new("fixture.symbol"),
                    source_module: ModuleId::new("fixture"),
                    export_rank: ExportRank::new(0),
                    kind: UserSymbolKind::Functor,
                    arity: UserSymbolArity::exact(2),
                }],
                fingerprint: LexicalSummaryFingerprint::new(1),
            }],
        )
        .unwrap()
    }

    fn empty_token_stream(source_id: SourceId) -> TokenStream {
        token_stream(source_id, Vec::new())
    }

    fn token_stream(source_id: SourceId, tokens: Vec<Token>) -> TokenStream {
        TokenStream {
            source_id,
            parser_context: ParserLexContext::general(),
            parser_lexing_plan: ParserLexingPlan::uniform(ParserLexContext::general()),
            tokens,
            scope_view: ScopeView::empty(source_id),
            diagnostics: Vec::new(),
        }
    }

    fn token(source_id: SourceId, kind: TokenKind, text: &str, start: usize, end: usize) -> Token {
        Token {
            kind,
            text: Arc::<str>::from(text),
            span: SourceRange {
                source_id,
                start,
                end,
            },
        }
    }

    fn source_id(byte: u8) -> SourceId {
        InMemorySessionIdAllocator::new()
            .next_source_id(snapshot_id(byte))
            .unwrap()
    }

    fn snapshot_id(byte: u8) -> BuildSnapshotId {
        let hex = format!("{byte:02x}").repeat(Hash::BYTE_LEN);
        BuildSnapshotId::from_published_schema_str(&format!(
            "mizar-session-build-snapshot-v1:{hex}"
        ))
        .unwrap()
    }
}
