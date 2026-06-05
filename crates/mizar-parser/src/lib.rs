use mizar_session::{Edition, SourceId, SourceRange};
use mizar_syntax::{
    SurfaceAst, SurfaceInfixOperator, SurfaceNode, SurfaceNodeId, SurfaceNodeKind,
    SurfaceOperatorAssociativity, SurfaceToken, SurfaceTokenKind, SyntaxDiagnostic,
    SyntaxDiagnosticCode,
};
use std::collections::BTreeMap;
use std::sync::Arc;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseRequest {
    pub source_id: SourceId,
    pub edition: Edition,
    pub tokens: Vec<ParserToken>,
    pub operator_fixity: Vec<OperatorFixityEntry>,
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
        }
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
    Parser::new(request).parse()
}

struct Parser {
    request: ParseRequest,
    nodes: Vec<SurfaceNode>,
    token_node_ids: Vec<SurfaceNodeId>,
    diagnostics: Vec<SyntaxDiagnostic>,
    fixity: BTreeMap<Arc<str>, OperatorFixityEntry>,
}

impl Parser {
    fn new(request: ParseRequest) -> Self {
        let fixity = request
            .operator_fixity
            .iter()
            .cloned()
            .map(|entry| (entry.spelling.clone(), entry))
            .collect();
        Self {
            request,
            nodes: Vec::new(),
            token_node_ids: Vec::new(),
            diagnostics: Vec::new(),
            fixity,
        }
    }

    fn parse(mut self) -> ParseOutput {
        self.add_token_nodes();
        let expression_root = self.parse_expression();
        let root = self.add_root(expression_root);
        ParseOutput {
            ast: Some(SurfaceAst::new(
                self.request.source_id,
                self.nodes,
                Some(root),
                self.token_node_ids,
                expression_root,
            )),
            diagnostics: self.diagnostics,
        }
    }

    fn add_token_nodes(&mut self) {
        let tokens = self.request.tokens.clone();
        for token in tokens {
            let kind = SurfaceNodeKind::Token(SurfaceToken::new(
                surface_token_kind(token.kind),
                token.text.clone(),
            ));
            let node = if token.kind == ParserTokenKind::ErrorRecovery {
                self.diagnostics.push(SyntaxDiagnostic::new(
                    SyntaxDiagnosticCode::UnexpectedErrorToken,
                    "error-recovery token reached the parser",
                    token.span,
                ));
                SurfaceNode::recovered(kind, token.span, Vec::new())
            } else {
                SurfaceNode::new(kind, token.span, Vec::new())
            };
            let id = self.push_node(node);
            self.token_node_ids.push(id);
        }
    }

    fn add_root(&mut self, expression_root: Option<SurfaceNodeId>) -> SurfaceNodeId {
        let children = self
            .token_node_ids
            .iter()
            .copied()
            .chain(expression_root)
            .collect::<Vec<_>>();
        let range = self
            .request
            .tokens
            .first()
            .zip(self.request.tokens.last())
            .map_or_else(
                || SourceRange {
                    source_id: self.request.source_id,
                    start: 0,
                    end: 0,
                },
                |(first, last)| SourceRange {
                    source_id: self.request.source_id,
                    start: first.span.start,
                    end: last.span.end,
                },
            );
        self.push_node(SurfaceNode::new(SurfaceNodeKind::Root, range, children))
    }

    fn parse_expression(&mut self) -> Option<SurfaceNodeId> {
        if self.fixity.is_empty() || self.token_node_ids.is_empty() {
            return None;
        }

        let mut expression = ExpressionParser {
            parser: self,
            position: 0,
            built_infix: false,
        };
        let root = expression.parse_binding_power(0);
        root.filter(|_| expression.built_infix)
    }

    fn push_node(&mut self, node: SurfaceNode) -> SurfaceNodeId {
        let id = SurfaceNodeId::new(self.nodes.len());
        self.nodes.push(node);
        id
    }

    fn fixity_for_token(&self, token: &ParserToken) -> Option<&OperatorFixityEntry> {
        if !matches!(
            token.kind,
            ParserTokenKind::UserSymbol | ParserTokenKind::ReservedSymbol
        ) {
            return None;
        }
        self.fixity.get(&token.text)
    }
}

struct ExpressionParser<'a> {
    parser: &'a mut Parser,
    position: usize,
    built_infix: bool,
}

impl ExpressionParser<'_> {
    fn parse_binding_power(&mut self, minimum_binding_power: u32) -> Option<SurfaceNodeId> {
        let mut left = self.next_operand()?;

        while let Some(operator) = self.current_operator().cloned() {
            let (left_binding_power, right_binding_power) = binding_powers(&operator);
            if left_binding_power < minimum_binding_power {
                break;
            }

            let operator_position = self.position;
            if operator.associativity == OperatorAssociativity::NonAssociative
                && self.left_is_non_associative_chain(left, &operator)
            {
                let span = self.parser.request.tokens[operator_position].span;
                self.parser.diagnostics.push(SyntaxDiagnostic::new(
                    SyntaxDiagnosticCode::NonAssociativeOperatorChain,
                    "non-associative operator chain requires explicit grouping",
                    span,
                ));
            }

            self.position += 1;
            let Some(right) = self.parse_binding_power(right_binding_power) else {
                let span = self.parser.request.tokens[operator_position].span;
                self.parser.diagnostics.push(SyntaxDiagnostic::new(
                    SyntaxDiagnosticCode::DanglingOperator,
                    "operator has no right operand",
                    span,
                ));
                break;
            };

            left = self.infix_node(left, operator_position, right, &operator);
        }

        Some(left)
    }

    fn next_operand(&mut self) -> Option<SurfaceNodeId> {
        let token = self.parser.request.tokens.get(self.position)?;
        if self.parser.fixity_for_token(token).is_some() || !is_operand_token(token.kind) {
            return None;
        }
        let id = self.parser.token_node_ids[self.position];
        self.position += 1;
        Some(id)
    }

    fn current_operator(&self) -> Option<&OperatorFixityEntry> {
        let token = self.parser.request.tokens.get(self.position)?;
        self.parser.fixity_for_token(token)
    }

    fn left_is_non_associative_chain(
        &self,
        left: SurfaceNodeId,
        operator: &OperatorFixityEntry,
    ) -> bool {
        matches!(
            &self.parser.nodes[left.index()].kind,
            SurfaceNodeKind::InfixExpression(left_operator)
                if left_operator.associativity == SurfaceOperatorAssociativity::NonAssociative
                    && left_operator.spelling == operator.spelling
        )
    }

    fn infix_node(
        &mut self,
        left: SurfaceNodeId,
        operator_position: usize,
        right: SurfaceNodeId,
        operator: &OperatorFixityEntry,
    ) -> SurfaceNodeId {
        let left_range = self.parser.nodes[left.index()].range;
        let right_range = self.parser.nodes[right.index()].range;
        let operator_id = self.parser.token_node_ids[operator_position];
        self.built_infix = true;
        self.parser.push_node(SurfaceNode::new(
            SurfaceNodeKind::InfixExpression(SurfaceInfixOperator {
                spelling: operator.spelling.clone(),
                precedence: operator.precedence,
                associativity: surface_associativity(operator.associativity),
            }),
            SourceRange {
                source_id: left_range.source_id,
                start: left_range.start,
                end: right_range.end,
            },
            vec![left, operator_id, right],
        ))
    }
}

fn binding_powers(operator: &OperatorFixityEntry) -> (u32, u32) {
    let precedence = u32::from(operator.precedence);
    match operator.associativity {
        OperatorAssociativity::Left | OperatorAssociativity::NonAssociative => {
            (precedence, precedence + 1)
        }
        OperatorAssociativity::Right => (precedence, precedence),
    }
}

fn is_operand_token(kind: ParserTokenKind) -> bool {
    matches!(
        kind,
        ParserTokenKind::Identifier | ParserTokenKind::Numeral | ParserTokenKind::StringLiteral
    )
}

fn surface_token_kind(kind: ParserTokenKind) -> SurfaceTokenKind {
    match kind {
        ParserTokenKind::Identifier => SurfaceTokenKind::Identifier,
        ParserTokenKind::ReservedWord => SurfaceTokenKind::ReservedWord,
        ParserTokenKind::ReservedSymbol => SurfaceTokenKind::ReservedSymbol,
        ParserTokenKind::Numeral => SurfaceTokenKind::Numeral,
        ParserTokenKind::LexemeRun => SurfaceTokenKind::LexemeRun,
        ParserTokenKind::UserSymbol => SurfaceTokenKind::UserSymbol,
        ParserTokenKind::StringLiteral => SurfaceTokenKind::StringLiteral,
        ParserTokenKind::ErrorRecovery => SurfaceTokenKind::ErrorRecovery,
        ParserTokenKind::Unknown => SurfaceTokenKind::Unknown,
    }
}

fn surface_associativity(associativity: OperatorAssociativity) -> SurfaceOperatorAssociativity {
    match associativity {
        OperatorAssociativity::Left => SurfaceOperatorAssociativity::Left,
        OperatorAssociativity::Right => SurfaceOperatorAssociativity::Right,
        OperatorAssociativity::NonAssociative => SurfaceOperatorAssociativity::NonAssociative,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        OperatorAssociativity, OperatorFixityEntry, ParseRequest, ParserToken, ParserTokenKind,
        parse,
    };
    use mizar_session::{
        BuildSnapshotId, Edition, Hash, InMemorySessionIdAllocator, SessionIdAllocator, SourceId,
        SourceRange,
    };
    use mizar_syntax::{SurfaceNodeKind, SurfaceOperatorAssociativity, SyntaxDiagnosticCode};

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
            .expression_root
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
        let root = ast.expression_root.expect("expression should have root");
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
        let node = ast.node(ast.token_nodes[0]).unwrap();
        assert!(node.recovered);
        let SurfaceNodeKind::Token(token) = &node.kind else {
            panic!("expected recovered token node");
        };
        assert_eq!(token.kind, mizar_syntax::SurfaceTokenKind::ErrorRecovery);
    }

    #[test]
    fn non_associative_operator_chains_emit_diagnostic() {
        let source_id = source_id(5);
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
        let root = ast.expression_root.expect("expression should have root");
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
        let source_id = source_id(6);
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
        let source_id = source_id(7);
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
            assert_eq!(ast.expression_root, None);
            assert_eq!(ast.token_texts(), vec!["a", "++", "b"]);
        }
    }

    #[test]
    fn separator_and_unknown_tokens_do_not_satisfy_right_operand() {
        let source_id = source_id(8);
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
        let source_id = source_id(9);
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
        let left_root = left_ast.expression_root.unwrap();
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
