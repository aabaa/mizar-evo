use mizar_session::{Edition, SourceAnchor, SourceId, SourceRange};
use mizar_syntax::{
    SurfaceAst, SurfaceInfixOperator, SurfaceNode, SurfaceNodeId, SurfaceNodeKind,
    SurfaceOperatorAssociativity, SurfaceToken, SurfaceTokenKind, SyntaxDiagnostic,
    SyntaxDiagnosticCode, SyntaxRecoveryKind,
};
use std::collections::BTreeMap;
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
    Parser::new(request).parse()
}

struct Parser {
    request: ParseRequest,
    nodes: Vec<SurfaceNode>,
    token_node_ids: Vec<SurfaceNodeId>,
    recovery_node_ids: Vec<SurfaceNodeId>,
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
            recovery_node_ids: Vec::new(),
            diagnostics: Vec::new(),
            fixity,
        }
    }

    fn parse(mut self) -> ParseOutput {
        self.add_token_nodes();
        if self.recover_syntax() == RecoveryOutcome::Unrecoverable {
            return ParseOutput {
                ast: None,
                diagnostics: self.diagnostics,
            };
        }
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

    fn recover_syntax(&mut self) -> RecoveryOutcome {
        self.recover_missing_string_literal();
        let block_outcome = self.recover_block_ends();
        if block_outcome == RecoveryOutcome::Unrecoverable {
            RecoveryOutcome::Unrecoverable
        } else {
            RecoveryOutcome::Recovered
        }
    }

    fn recover_missing_string_literal(&mut self) {
        if self.request.string_required_context != StringRequiredContext::UniformForTest {
            return;
        }

        let missing_position = self
            .request
            .tokens
            .iter()
            .position(|token| token.kind != ParserTokenKind::StringLiteral);
        let (position, span) = missing_position.map_or_else(
            || {
                let offset = self.request.tokens.last().map_or(0, |token| token.span.end);
                (
                    None,
                    SourceRange {
                        source_id: self.request.source_id,
                        start: offset,
                        end: offset,
                    },
                )
            },
            |position| {
                let token = &self.request.tokens[position];
                (
                    Some(position),
                    SourceRange {
                        source_id: token.span.source_id,
                        start: token.span.start,
                        end: token.span.start,
                    },
                )
            },
        );

        if position.is_none() && !self.request.tokens.is_empty() {
            return;
        }

        let diagnostic_primary =
            position.map_or(span, |position| self.request.tokens[position].span);
        self.diagnostics.push(
            SyntaxDiagnostic::new(
                SyntaxDiagnosticCode::MissingStringLiteral,
                "expected string literal at this grammar position",
                diagnostic_primary,
            )
            .with_recovery_note("insert a string literal before continuing"),
        );
        self.add_recovery_node(SyntaxRecoveryKind::MissingStringLiteral, span, Vec::new());
    }

    fn recover_block_ends(&mut self) -> RecoveryOutcome {
        let tokens = self.request.tokens.clone();
        let mut stack = Vec::new();

        for (position, token) in tokens.iter().enumerate() {
            if opens_recovery_block(&tokens, position) {
                stack.push(BlockStart {
                    keyword: token.text.clone(),
                    span: token.span,
                    token_node_id: self.token_node_ids[position],
                });
            } else if is_end_keyword(token) && stack.pop().is_none() {
                self.diagnostics.push(
                    SyntaxDiagnostic::new(
                        SyntaxDiagnosticCode::UnrecoverableInput,
                        "`end` has no matching block opener",
                        token.span,
                    )
                    .with_recovery_note(
                        "remove the stray `end` or add a matching block opener before it",
                    ),
                );
                return RecoveryOutcome::Unrecoverable;
            }
        }

        if !stack.is_empty() {
            let offset = tokens.last().map_or(0, |token| token.span.end);
            self.recover_missing_ends(&mut stack, offset);
        }

        RecoveryOutcome::Recovered
    }

    fn recover_missing_ends(&mut self, stack: &mut Vec<BlockStart>, insertion_offset: usize) {
        while let Some(block) = stack.pop() {
            let span = SourceRange {
                source_id: self.request.source_id,
                start: insertion_offset,
                end: insertion_offset,
            };
            self.diagnostics.push(
                SyntaxDiagnostic::new(
                    SyntaxDiagnosticCode::MissingEnd,
                    format!("missing `end` for `{}` block", block.keyword),
                    span,
                )
                .with_secondary([SourceAnchor::Range(block.span)])
                .with_recovery_note("insert `end` before this synchronization point"),
            );
            self.add_recovery_node(
                SyntaxRecoveryKind::MissingEnd,
                span,
                vec![block.token_node_id],
            );
        }
    }

    fn add_recovery_node(
        &mut self,
        recovery_kind: SyntaxRecoveryKind,
        range: SourceRange,
        children: Vec<SurfaceNodeId>,
    ) -> SurfaceNodeId {
        let id = self.push_node(SurfaceNode::recovered(
            SurfaceNodeKind::ErrorRecovery(recovery_kind),
            range,
            children,
        ));
        self.recovery_node_ids.push(id);
        id
    }

    fn add_root(&mut self, expression_root: Option<SurfaceNodeId>) -> SurfaceNodeId {
        let children = self
            .token_node_ids
            .iter()
            .copied()
            .chain(expression_root)
            .chain(self.recovery_node_ids.iter().copied())
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RecoveryOutcome {
    Recovered,
    Unrecoverable,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct BlockStart {
    keyword: Arc<str>,
    span: SourceRange,
    token_node_id: SurfaceNodeId,
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

fn is_reserved_word_token(token: &ParserToken, spelling: &str) -> bool {
    token.kind == ParserTokenKind::ReservedWord && token.text.as_ref() == spelling
}

fn is_end_keyword(token: &ParserToken) -> bool {
    is_reserved_word_token(token, "end")
}

fn is_else_keyword(token: &ParserToken) -> bool {
    is_reserved_word_token(token, "else")
}

fn opens_recovery_block(tokens: &[ParserToken], position: usize) -> bool {
    let token = &tokens[position];
    if !is_block_start_keyword(token) {
        return false;
    }

    if is_reserved_word_token(token, "for") {
        return looks_like_algorithm_for_loop(tokens, position);
    }

    !(is_reserved_word_token(token, "if") && position > 0 && is_else_keyword(&tokens[position - 1]))
}

fn looks_like_algorithm_for_loop(tokens: &[ParserToken], position: usize) -> bool {
    matches!(
        (tokens.get(position + 1), tokens.get(position + 2)),
        (
            Some(ParserToken {
                kind: ParserTokenKind::Identifier,
                ..
            }),
            Some(next)
        ) if is_reserved_word_token(next, "in")
            || (next.kind == ParserTokenKind::ReservedSymbol && next.text.as_ref() == "=")
    )
}

fn is_block_start_keyword(token: &ParserToken) -> bool {
    token.kind == ParserTokenKind::ReservedWord
        && matches!(
            token.text.as_ref(),
            "algorithm"
                | "definition"
                | "registration"
                | "proof"
                | "now"
                | "hereby"
                | "case"
                | "suppose"
                | "if"
                | "while"
                | "for"
                | "match"
                | "claim"
                | "otherwise"
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
            .nodes
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
        assert_eq!(recovery_node.1.children, vec![ast.token_nodes[0]]);
        let root = ast.root.expect("recovered AST should have a root");
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
            .nodes
            .iter()
            .find(|node| {
                matches!(
                    &node.kind,
                    SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::MissingEnd)
                )
            })
            .expect("outer missing end should create an explicit recovery node");
        assert_eq!(recovery_node.children, vec![ast.token_nodes[0]]);
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
            .nodes
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
        let root = ast.root.expect("recovered AST should have a root");
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
            assert_eq!(ast.expression_root, None);
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
