use crate::{
    OperatorAssociativity, OperatorFixityEntry, ParserTokenKind, event::SyntaxEvent,
    grammar::Parser,
};
use mizar_session::SourceRange;
use mizar_syntax::{
    SurfaceBuilderNodeId, SurfaceInfixOperator, SurfaceNodeKind, SurfaceOperatorAssociativity,
    SyntaxDiagnostic, SyntaxDiagnosticCode,
};

impl Parser {
    pub(super) fn parse_expression(&mut self) -> Option<SurfaceBuilderNodeId> {
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
}

struct ExpressionParser<'a> {
    parser: &'a mut Parser,
    position: usize,
    built_infix: bool,
}

impl ExpressionParser<'_> {
    fn parse_binding_power(&mut self, minimum_binding_power: u32) -> Option<SurfaceBuilderNodeId> {
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

    fn next_operand(&mut self) -> Option<SurfaceBuilderNodeId> {
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
        left: SurfaceBuilderNodeId,
        operator: &OperatorFixityEntry,
    ) -> bool {
        matches!(
            self.parser.events.node_kind(left).unwrap(),
            SurfaceNodeKind::InfixExpression(left_operator)
                if left_operator.associativity == SurfaceOperatorAssociativity::NonAssociative
                    && left_operator.spelling == operator.spelling
        )
    }

    fn infix_node(
        &mut self,
        left: SurfaceBuilderNodeId,
        operator_position: usize,
        right: SurfaceBuilderNodeId,
        operator: &OperatorFixityEntry,
    ) -> SurfaceBuilderNodeId {
        let left_range = self.parser.events.node_range(left).unwrap();
        let right_range = self.parser.events.node_range(right).unwrap();
        let operator_id = self.parser.token_node_ids[operator_position];
        self.built_infix = true;
        self.parser.events.emit(SyntaxEvent::Node {
            kind: SurfaceNodeKind::InfixExpression(SurfaceInfixOperator {
                spelling: operator.spelling.clone(),
                precedence: operator.precedence,
                associativity: surface_associativity(operator.associativity),
            }),
            range: SourceRange {
                source_id: left_range.source_id,
                start: left_range.start,
                end: right_range.end,
            },
            children: vec![left, operator_id, right],
        })
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

fn surface_associativity(associativity: OperatorAssociativity) -> SurfaceOperatorAssociativity {
    match associativity {
        OperatorAssociativity::Left => SurfaceOperatorAssociativity::Left,
        OperatorAssociativity::Right => SurfaceOperatorAssociativity::Right,
        OperatorAssociativity::NonAssociative => SurfaceOperatorAssociativity::NonAssociative,
    }
}
