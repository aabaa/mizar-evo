mod grammar;
mod pratt;
mod recovery;

use mizar_session::{Edition, SourceId, SourceRange};
use mizar_syntax::{SurfaceAst, SyntaxDiagnostic};
use std::sync::Arc;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseRequest {
    pub source_id: SourceId,
    pub edition: Edition,
    pub tokens: Vec<ParserToken>,
    pub operator_fixity: Vec<OperatorFixityEntry>,
    pub string_required_context: StringRequiredContext,
}

impl ParseRequest {
    pub fn new(
        source_id: SourceId,
        edition: Edition,
        tokens: Vec<ParserToken>,
        operator_fixity: Vec<OperatorFixityEntry>,
    ) -> Self {
        Self {
            source_id,
            edition,
            tokens,
            operator_fixity,
            string_required_context: StringRequiredContext::None,
        }
    }

    pub fn with_string_required_context(mut self, context: StringRequiredContext) -> Self {
        self.string_required_context = context;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParserToken {
    pub kind: ParserTokenKind,
    pub text: Arc<str>,
    pub span: SourceRange,
}

impl ParserToken {
    pub fn new(kind: ParserTokenKind, text: impl Into<Arc<str>>, span: SourceRange) -> Self {
        Self {
            kind,
            text: text.into(),
            span,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParserTokenKind {
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OperatorFixityEntry {
    pub spelling: Arc<str>,
    pub precedence: u8,
    pub associativity: OperatorAssociativity,
}

impl OperatorFixityEntry {
    pub fn new(
        spelling: impl Into<Arc<str>>,
        precedence: u8,
        associativity: OperatorAssociativity,
    ) -> Self {
        Self {
            spelling: spelling.into(),
            precedence,
            associativity,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OperatorAssociativity {
    Left,
    Right,
    NonAssociative,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum StringRequiredContext {
    #[default]
    None,
    UniformForTest,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseOutput {
    pub ast: Option<SurfaceAst>,
    pub diagnostics: Vec<SyntaxDiagnostic>,
}

impl ParseOutput {
    pub fn new(ast: Option<SurfaceAst>, diagnostics: Vec<SyntaxDiagnostic>) -> Self {
        Self { ast, diagnostics }
    }
}

pub fn parse(request: ParseRequest) -> ParseOutput {
    grammar::Parser::new(request).parse()
}

#[cfg(test)]
mod tests {
    use super::{
        OperatorAssociativity, OperatorFixityEntry, ParseRequest, ParserToken, ParserTokenKind,
        StringRequiredContext, parse,
    };
    use mizar_session::{
        BuildSnapshotId, Edition, Hash, InMemorySessionIdAllocator, SessionIdAllocator, SourceId,
        SourceRange,
    };
    use mizar_syntax::{
        SurfaceNodeKind, SurfaceOperatorAssociativity, SyntaxDiagnosticCode, SyntaxRecoveryKind,
    };

    #[test]
    fn well_formed_token_stream_parses_to_surface_ast_preserving_order_and_ranges() {
        let source_id = source_id(1);
        let tokens = vec![
            token(source_id, ParserTokenKind::Identifier, "alpha", 0, 5),
            token(source_id, ParserTokenKind::ReservedSymbol, ";", 5, 6),
            token(source_id, ParserTokenKind::Identifier, "beta", 7, 11),
        ];

        let output = parse(ParseRequest::new(
            source_id,
            Edition::new("2026"),
            tokens,
            Vec::new(),
        ));

        assert!(output.diagnostics.is_empty());
        let ast = output.ast.expect("well-formed token stream should parse");
        assert_eq!(ast.token_texts(), vec!["alpha", ";", "beta"]);
        let ranges = ast
            .token_nodes()
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
    fn operator_fixity_drives_pratt_precedence() {
        let source_id = source_id(2);
        let tokens = vec![
            token(source_id, ParserTokenKind::Identifier, "a", 0, 1),
            token(source_id, ParserTokenKind::UserSymbol, "++", 2, 4),
            token(source_id, ParserTokenKind::Identifier, "b", 5, 6),
            token(source_id, ParserTokenKind::UserSymbol, "**", 7, 9),
            token(source_id, ParserTokenKind::Identifier, "c", 10, 11),
        ];

        let output = parse(ParseRequest::new(
            source_id,
            Edition::new("2026"),
            tokens,
            vec![
                OperatorFixityEntry::new("++", 10, OperatorAssociativity::Left),
                OperatorFixityEntry::new("**", 20, OperatorAssociativity::Right),
            ],
        ));

        assert!(output.diagnostics.is_empty());
        let ast = output.ast.expect("expression should parse");
        let root = ast
            .expression_root()
            .expect("fixity input should build an expression");
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
    fn right_associative_operator_groups_to_the_right() {
        let source_id = source_id(3);
        let tokens = vec![
            token(source_id, ParserTokenKind::Identifier, "a", 0, 1),
            token(source_id, ParserTokenKind::UserSymbol, "**", 2, 4),
            token(source_id, ParserTokenKind::Identifier, "b", 5, 6),
            token(source_id, ParserTokenKind::UserSymbol, "**", 7, 9),
            token(source_id, ParserTokenKind::Identifier, "c", 10, 11),
        ];

        let output = parse(ParseRequest::new(
            source_id,
            Edition::new("2026"),
            tokens,
            vec![OperatorFixityEntry::new(
                "**",
                20,
                OperatorAssociativity::Right,
            )],
        ));

        assert!(output.diagnostics.is_empty());
        let ast = output.ast.expect("expression should parse");
        let root = ast.expression_root().expect("expression should have root");
        let SurfaceNodeKind::InfixExpression(root_operator) = &ast.node(root).unwrap().kind else {
            panic!("expected infix expression root");
        };
        assert_eq!(root_operator.spelling.as_ref(), "**");
        assert_eq!(root_operator.precedence, 20);
        assert_eq!(
            root_operator.associativity,
            SurfaceOperatorAssociativity::Right
        );
        let right_child = ast.node(root).unwrap().children[2];
        let SurfaceNodeKind::InfixExpression(right_operator) = &ast.node(right_child).unwrap().kind
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
    fn error_recovery_tokens_preserve_original_text_and_emit_diagnostic() {
        let source_id = source_id(4);
        let tokens = vec![token(source_id, ParserTokenKind::ErrorRecovery, "@", 4, 5)];

        let output = parse(ParseRequest::new(
            source_id,
            Edition::new("2026"),
            tokens,
            Vec::new(),
        ));

        assert_eq!(output.diagnostics.len(), 1);
        assert_eq!(
            output.diagnostics[0].code,
            SyntaxDiagnosticCode::UnexpectedErrorToken
        );
        let ast = output.ast.expect("recovered token should preserve AST");
        assert_eq!(ast.token_texts(), vec!["@"]);
        let node = ast.node(ast.token_nodes()[0]).unwrap();
        assert!(node.recovered);
        let SurfaceNodeKind::Token(token) = &node.kind else {
            panic!("expected recovered token node");
        };
        assert_eq!(token.kind, mizar_syntax::SurfaceTokenKind::ErrorRecovery);
    }

    #[test]
    fn missing_end_recovers_at_eof_with_error_node() {
        let source_id = source_id(5);
        let tokens = vec![
            token(
                source_id,
                ParserTokenKind::ReservedWord,
                "definition",
                0,
                10,
            ),
            token(source_id, ParserTokenKind::ReservedWord, "theorem", 11, 18),
        ];

        let output = parse(ParseRequest::new(
            source_id,
            Edition::new("2026"),
            tokens,
            Vec::new(),
        ));

        assert_eq!(output.diagnostics.len(), 1);
        assert_eq!(output.diagnostics[0].code, SyntaxDiagnosticCode::MissingEnd);
        assert_eq!(
            output.diagnostics[0].primary,
            SourceRange {
                source_id,
                start: 18,
                end: 18,
            }
        );
        assert_eq!(
            output.diagnostics[0].secondary,
            vec![mizar_session::SourceAnchor::Range(SourceRange {
                source_id,
                start: 0,
                end: 10,
            })]
        );
        let ast = output
            .ast
            .expect("missing end should recover a surface AST");
        let recovery_node = ast
            .nodes()
            .iter()
            .enumerate()
            .find(|node| {
                matches!(
                    &node.1.kind,
                    SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::MissingEnd)
                )
            })
            .expect("missing end should create an explicit recovery node");
        assert!(recovery_node.1.recovered);
        assert_eq!(
            recovery_node.1.range,
            SourceRange {
                source_id,
                start: 18,
                end: 18,
            }
        );
        assert_eq!(recovery_node.1.children, vec![ast.token_nodes()[0]]);
        let root = ast.root().expect("recovered AST should have a root");
        assert!(
            ast.node(root)
                .unwrap()
                .children
                .iter()
                .any(|child| child.index() == recovery_node.0),
            "recovery node should be reachable from the root"
        );
    }

    #[test]
    fn block_content_keywords_do_not_trigger_missing_end_recovery() {
        let source_id = source_id(13);
        let tokens = vec![
            token(
                source_id,
                ParserTokenKind::ReservedWord,
                "definition",
                0,
                10,
            ),
            token(source_id, ParserTokenKind::ReservedWord, "func", 11, 15),
            token(source_id, ParserTokenKind::ReservedWord, "end", 16, 19),
        ];

        let output = parse(ParseRequest::new(
            source_id,
            Edition::new("2026"),
            tokens,
            Vec::new(),
        ));

        assert!(output.diagnostics.is_empty());
        assert!(
            output.ast.is_some(),
            "matching end should keep the minimal surface AST recoverable"
        );
    }

    #[test]
    fn algorithm_do_body_does_not_make_outer_end_unrecoverable() {
        let source_id = source_id(14);
        let tokens = vec![
            token(
                source_id,
                ParserTokenKind::ReservedWord,
                "definition",
                0,
                10,
            ),
            token(
                source_id,
                ParserTokenKind::ReservedWord,
                "algorithm",
                11,
                20,
            ),
            token(source_id, ParserTokenKind::ReservedWord, "do", 21, 23),
            token(source_id, ParserTokenKind::ReservedWord, "end", 24, 27),
            token(source_id, ParserTokenKind::ReservedWord, "end", 28, 31),
        ];

        let output = parse(ParseRequest::new(
            source_id,
            Edition::new("2026"),
            tokens,
            Vec::new(),
        ));

        assert!(output.diagnostics.is_empty());
        assert!(
            output.ast.is_some(),
            "nested algorithm/do ends should not make the outer end stray"
        );
    }

    #[test]
    fn algorithm_control_blocks_match_their_own_end_before_outer_algorithm_end() {
        let source_id = source_id(15);
        for (block_tokens, block_end_start, outer_end_start) in [
            (vec![("if", ParserTokenKind::ReservedWord)], 21, 25),
            (vec![("while", ParserTokenKind::ReservedWord)], 24, 28),
            (
                vec![
                    ("for", ParserTokenKind::ReservedWord),
                    ("i", ParserTokenKind::Identifier),
                    ("=", ParserTokenKind::ReservedSymbol),
                ],
                26,
                30,
            ),
            (
                vec![
                    ("for", ParserTokenKind::ReservedWord),
                    ("x", ParserTokenKind::Identifier),
                    ("in", ParserTokenKind::ReservedWord),
                ],
                27,
                31,
            ),
            (vec![("match", ParserTokenKind::ReservedWord)], 24, 28),
            (vec![("claim", ParserTokenKind::ReservedWord)], 24, 28),
        ] {
            let block_name = block_tokens[0].0;
            let mut tokens = vec![token(
                source_id,
                ParserTokenKind::ReservedWord,
                "algorithm",
                0,
                9,
            )];
            let mut offset = 10;
            for (text, kind) in block_tokens {
                tokens.push(token(source_id, kind, text, offset, offset + text.len()));
                offset += text.len() + 1;
            }
            tokens.extend([
                token(
                    source_id,
                    ParserTokenKind::ReservedWord,
                    "end",
                    block_end_start,
                    block_end_start + 3,
                ),
                token(
                    source_id,
                    ParserTokenKind::ReservedWord,
                    "end",
                    outer_end_start,
                    outer_end_start + 3,
                ),
            ]);

            let output = parse(ParseRequest::new(
                source_id,
                Edition::new("2026"),
                tokens,
                Vec::new(),
            ));

            assert!(
                output.diagnostics.is_empty(),
                "{block_name} should consume the inner end without making the outer end stray"
            );
            assert!(
                output.ast.is_some(),
                "{block_name} nested inside algorithm should remain recoverable"
            );
        }
    }

    #[test]
    fn quantifier_for_does_not_open_recovery_block() {
        let source_id = source_id(16);
        let tokens = vec![
            token(
                source_id,
                ParserTokenKind::ReservedWord,
                "definition",
                0,
                10,
            ),
            token(source_id, ParserTokenKind::ReservedWord, "theorem", 11, 18),
            token(source_id, ParserTokenKind::ReservedWord, "for", 19, 22),
            token(source_id, ParserTokenKind::Identifier, "x", 23, 24),
            token(source_id, ParserTokenKind::ReservedWord, "being", 25, 30),
            token(source_id, ParserTokenKind::Identifier, "Nat", 31, 34),
            token(source_id, ParserTokenKind::ReservedWord, "holds", 35, 40),
            token(source_id, ParserTokenKind::ReservedWord, "end", 41, 44),
        ];

        let output = parse(ParseRequest::new(
            source_id,
            Edition::new("2026"),
            tokens,
            Vec::new(),
        ));

        assert!(
            output.diagnostics.is_empty(),
            "formula quantifiers must not consume a surrounding block end"
        );
        assert!(output.ast.is_some());
    }

    #[test]
    fn match_otherwise_branch_matches_its_own_end() {
        let source_id = source_id(17);
        let tokens = vec![
            token(source_id, ParserTokenKind::ReservedWord, "algorithm", 0, 9),
            token(source_id, ParserTokenKind::ReservedWord, "match", 10, 15),
            token(source_id, ParserTokenKind::ReservedWord, "case", 20, 24),
            token(source_id, ParserTokenKind::ReservedWord, "end", 30, 33),
            token(
                source_id,
                ParserTokenKind::ReservedWord,
                "otherwise",
                34,
                43,
            ),
            token(source_id, ParserTokenKind::ReservedWord, "end", 50, 53),
            token(source_id, ParserTokenKind::ReservedWord, "end", 54, 57),
            token(source_id, ParserTokenKind::ReservedWord, "end", 58, 61),
        ];

        let output = parse(ParseRequest::new(
            source_id,
            Edition::new("2026"),
            tokens,
            Vec::new(),
        ));

        assert!(output.diagnostics.is_empty());
        assert!(output.ast.is_some());
    }

    #[test]
    fn else_if_chain_uses_one_recovery_block_for_the_if_chain() {
        let source_id = source_id(18);
        let tokens = vec![
            token(source_id, ParserTokenKind::ReservedWord, "algorithm", 0, 9),
            token(source_id, ParserTokenKind::ReservedWord, "if", 10, 12),
            token(source_id, ParserTokenKind::ReservedWord, "else", 18, 22),
            token(source_id, ParserTokenKind::ReservedWord, "if", 23, 25),
            token(source_id, ParserTokenKind::ReservedWord, "end", 31, 34),
            token(source_id, ParserTokenKind::ReservedWord, "end", 35, 38),
        ];

        let output = parse(ParseRequest::new(
            source_id,
            Edition::new("2026"),
            tokens,
            Vec::new(),
        ));

        assert!(output.diagnostics.is_empty());
        assert!(output.ast.is_some());
    }

    #[test]
    fn partially_closed_nested_blocks_recover_missing_outer_end() {
        let source_id = source_id(19);
        let tokens = vec![
            token(
                source_id,
                ParserTokenKind::ReservedWord,
                "definition",
                0,
                10,
            ),
            token(
                source_id,
                ParserTokenKind::ReservedWord,
                "algorithm",
                11,
                20,
            ),
            token(source_id, ParserTokenKind::ReservedWord, "end", 21, 24),
        ];

        let output = parse(ParseRequest::new(
            source_id,
            Edition::new("2026"),
            tokens,
            Vec::new(),
        ));

        assert_eq!(output.diagnostics.len(), 1);
        assert_eq!(output.diagnostics[0].code, SyntaxDiagnosticCode::MissingEnd);
        assert_eq!(
            output.diagnostics[0].primary,
            SourceRange {
                source_id,
                start: 24,
                end: 24,
            }
        );
        assert_eq!(
            output.diagnostics[0].secondary,
            vec![mizar_session::SourceAnchor::Range(SourceRange {
                source_id,
                start: 0,
                end: 10,
            })]
        );
        let ast = output
            .ast
            .expect("partially closed nested blocks should recover");
        let recovery_node = ast
            .nodes()
            .iter()
            .find(|node| {
                matches!(
                    &node.kind,
                    SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::MissingEnd)
                )
            })
            .expect("outer missing end should create an explicit recovery node");
        assert_eq!(recovery_node.children, vec![ast.token_nodes()[0]]);
    }

    #[test]
    fn unrecoverable_stream_returns_no_ast_with_diagnostic() {
        let source_id = source_id(6);
        let tokens = vec![token(source_id, ParserTokenKind::ReservedWord, "end", 0, 3)];

        let output = parse(ParseRequest::new(
            source_id,
            Edition::new("2026"),
            tokens,
            Vec::new(),
        ));

        assert!(output.ast.is_none());
        assert_eq!(output.diagnostics.len(), 1);
        assert_eq!(
            output.diagnostics[0].code,
            SyntaxDiagnosticCode::UnrecoverableInput
        );
        assert_eq!(
            output.diagnostics[0].primary,
            SourceRange {
                source_id,
                start: 0,
                end: 3,
            }
        );
    }

    #[test]
    fn stray_end_after_closed_block_is_unrecoverable() {
        let source_id = source_id(20);
        let tokens = vec![
            token(
                source_id,
                ParserTokenKind::ReservedWord,
                "definition",
                0,
                10,
            ),
            token(source_id, ParserTokenKind::ReservedWord, "end", 11, 14),
            token(source_id, ParserTokenKind::ReservedWord, "end", 15, 18),
        ];

        let output = parse(ParseRequest::new(
            source_id,
            Edition::new("2026"),
            tokens,
            Vec::new(),
        ));

        assert!(output.ast.is_none());
        assert_eq!(output.diagnostics.len(), 1);
        assert_eq!(
            output.diagnostics[0].code,
            SyntaxDiagnosticCode::UnrecoverableInput
        );
        assert_eq!(
            output.diagnostics[0].primary,
            SourceRange {
                source_id,
                start: 15,
                end: 18,
            }
        );
    }

    #[test]
    fn missing_string_literal_in_string_required_context_emits_diagnostic() {
        let source_id = source_id(7);
        let tokens = vec![token(
            source_id,
            ParserTokenKind::Identifier,
            "not_a_string",
            0,
            12,
        )];

        let output = parse(
            ParseRequest::new(source_id, Edition::new("2026"), tokens, Vec::new())
                .with_string_required_context(StringRequiredContext::UniformForTest),
        );

        assert_eq!(output.diagnostics.len(), 1);
        assert_eq!(
            output.diagnostics[0].code,
            SyntaxDiagnosticCode::MissingStringLiteral
        );
        assert_eq!(
            output.diagnostics[0].primary,
            SourceRange {
                source_id,
                start: 0,
                end: 12,
            }
        );
        let ast = output
            .ast
            .expect("missing string literal should recover a surface AST");
        let recovery_node = ast
            .nodes()
            .iter()
            .enumerate()
            .find(|node| {
                matches!(
                    &node.1.kind,
                    SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::MissingStringLiteral)
                )
            })
            .expect("missing string literal should create an explicit recovery node");
        assert!(recovery_node.1.recovered);
        assert_eq!(
            recovery_node.1.range,
            SourceRange {
                source_id,
                start: 0,
                end: 0,
            }
        );
        let root = ast.root().expect("recovered AST should have a root");
        assert!(
            ast.node(root)
                .unwrap()
                .children
                .iter()
                .any(|child| child.index() == recovery_node.0),
            "recovery node should be reachable from the root"
        );
    }

    #[test]
    fn non_associative_operator_chains_emit_diagnostic() {
        let source_id = source_id(8);
        let tokens = vec![
            token(source_id, ParserTokenKind::Identifier, "a", 0, 1),
            token(source_id, ParserTokenKind::ReservedSymbol, "=", 2, 3),
            token(source_id, ParserTokenKind::Identifier, "b", 4, 5),
            token(source_id, ParserTokenKind::ReservedSymbol, "=", 6, 7),
            token(source_id, ParserTokenKind::Identifier, "c", 8, 9),
        ];

        let output = parse(ParseRequest::new(
            source_id,
            Edition::new("2026"),
            tokens,
            vec![OperatorFixityEntry::new(
                "=",
                10,
                OperatorAssociativity::NonAssociative,
            )],
        ));

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
        let ast = output.ast.expect("expression should parse");
        let root = ast.expression_root().expect("expression should have root");
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
    fn different_non_associative_operators_at_same_precedence_do_not_chain_diagnose() {
        let source_id = source_id(9);
        let tokens = vec![
            token(source_id, ParserTokenKind::Identifier, "a", 0, 1),
            token(source_id, ParserTokenKind::ReservedSymbol, "=", 2, 3),
            token(source_id, ParserTokenKind::Identifier, "b", 4, 5),
            token(source_id, ParserTokenKind::ReservedSymbol, "<>", 6, 8),
            token(source_id, ParserTokenKind::Identifier, "c", 9, 10),
        ];

        let output = parse(ParseRequest::new(
            source_id,
            Edition::new("2026"),
            tokens,
            vec![
                OperatorFixityEntry::new("=", 10, OperatorAssociativity::NonAssociative),
                OperatorFixityEntry::new("<>", 10, OperatorAssociativity::NonAssociative),
            ],
        ));

        assert!(output.diagnostics.is_empty());
    }

    #[test]
    fn recovered_and_unknown_tokens_do_not_become_infix_operators() {
        let source_id = source_id(10);
        for kind in [ParserTokenKind::ErrorRecovery, ParserTokenKind::Unknown] {
            let tokens = vec![
                token(source_id, ParserTokenKind::Identifier, "a", 0, 1),
                token(source_id, kind, "++", 2, 4),
                token(source_id, ParserTokenKind::Identifier, "b", 5, 6),
            ];

            let output = parse(ParseRequest::new(
                source_id,
                Edition::new("2026"),
                tokens,
                vec![OperatorFixityEntry::new(
                    "++",
                    10,
                    OperatorAssociativity::Left,
                )],
            ));

            let ast = output.ast.expect("parser should preserve token stream");
            assert_eq!(ast.expression_root(), None);
            assert_eq!(ast.token_texts(), vec!["a", "++", "b"]);
        }
    }

    #[test]
    fn separator_and_unknown_tokens_do_not_satisfy_right_operand() {
        let source_id = source_id(11);
        for kind in [
            ParserTokenKind::ReservedSymbol,
            ParserTokenKind::ReservedWord,
            ParserTokenKind::Unknown,
        ] {
            let tokens = vec![
                token(source_id, ParserTokenKind::Identifier, "a", 0, 1),
                token(source_id, ParserTokenKind::UserSymbol, "++", 2, 4),
                token(source_id, kind, ";", 5, 6),
            ];

            let output = parse(ParseRequest::new(
                source_id,
                Edition::new("2026"),
                tokens,
                vec![OperatorFixityEntry::new(
                    "++",
                    10,
                    OperatorAssociativity::Left,
                )],
            ));

            assert_eq!(output.diagnostics.len(), 1);
            assert_eq!(
                output.diagnostics[0].code,
                SyntaxDiagnosticCode::DanglingOperator
            );
            assert_eq!(
                output.diagnostics[0].primary,
                SourceRange {
                    source_id,
                    start: 2,
                    end: 4,
                }
            );
        }
    }

    #[test]
    fn max_precedence_left_and_non_associative_operators_keep_binding_shape() {
        let source_id = source_id(12);
        let tokens = vec![
            token(source_id, ParserTokenKind::Identifier, "a", 0, 1),
            token(source_id, ParserTokenKind::UserSymbol, "++", 2, 4),
            token(source_id, ParserTokenKind::Identifier, "b", 5, 6),
            token(source_id, ParserTokenKind::UserSymbol, "++", 7, 9),
            token(source_id, ParserTokenKind::Identifier, "c", 10, 11),
        ];

        let left_output = parse(ParseRequest::new(
            source_id,
            Edition::new("2026"),
            tokens.clone(),
            vec![OperatorFixityEntry::new(
                "++",
                u8::MAX,
                OperatorAssociativity::Left,
            )],
        ));
        assert!(left_output.diagnostics.is_empty());
        let left_ast = left_output.ast.expect("left expression should parse");
        let left_root = left_ast.expression_root().unwrap();
        let SurfaceNodeKind::InfixExpression(_) = &left_ast.node(left_root).unwrap().kind else {
            panic!("expected infix expression root");
        };
        let left_child = left_ast.node(left_root).unwrap().children[0];
        let SurfaceNodeKind::InfixExpression(_) = &left_ast.node(left_child).unwrap().kind else {
            panic!("expected max-precedence left operator to associate left");
        };

        let non_associative_output = parse(ParseRequest::new(
            source_id,
            Edition::new("2026"),
            tokens,
            vec![OperatorFixityEntry::new(
                "++",
                u8::MAX,
                OperatorAssociativity::NonAssociative,
            )],
        ));
        assert_eq!(non_associative_output.diagnostics.len(), 1);
        assert_eq!(
            non_associative_output.diagnostics[0].code,
            SyntaxDiagnosticCode::NonAssociativeOperatorChain
        );
    }

    fn token(
        source_id: SourceId,
        kind: ParserTokenKind,
        text: &str,
        start: usize,
        end: usize,
    ) -> ParserToken {
        ParserToken::new(
            kind,
            text,
            SourceRange {
                source_id,
                start,
                end,
            },
        )
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
