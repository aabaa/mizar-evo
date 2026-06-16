use crate::{
    OperatorAssociativity, OperatorFixity, OperatorFixityEntry, ParserToken, ParserTokenKind,
    cursor::is_reserved_word_token,
    diagnostic::{ExpectedToken, expected_token_diagnostic},
    event::SyntaxEvent,
    grammar::Parser,
    path::ParsedPathNode,
    sync::{self, is_top_level_item_keyword},
};
use mizar_session::{SourceAnchor, SourceRange};
use mizar_syntax::{
    SkippedTokenReason, SurfaceBuilderNodeId, SurfaceFormulaBinaryOperator,
    SurfaceFormulaConnective, SurfaceFormulaConstant, SurfaceFormulaPrefixOperator,
    SurfaceInfixOperator, SurfaceNodeKind, SurfaceOperatorAssociativity, SurfacePostfixOperator,
    SurfacePrefixOperator, SurfaceQuantifierKind, SyntaxDiagnostic, SyntaxDiagnosticCode,
    SyntaxRecoveryKind,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ParsedCompilationUnit {
    pub(super) id: SurfaceBuilderNodeId,
    pub(super) recovery_nodes: Vec<SurfaceBuilderNodeId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ParsedItem {
    id: SurfaceBuilderNodeId,
    next_position: usize,
    recovery_nodes: Vec<SurfaceBuilderNodeId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ParsedTypeNode {
    id: SurfaceBuilderNodeId,
    next_position: usize,
    recovery_nodes: Vec<SurfaceBuilderNodeId>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum QuaTargetGrammar {
    TypeExpression,
    RadixType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CaseBranchKind {
    Case,
    Suppose,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum InheritanceTargetBoundary {
    Extends,
    Tail,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct AttributeRefPlan {
    start_position: usize,
    next_position: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct AttributeSymbolPlan {
    prefix_segments: Vec<(usize, usize)>,
    final_symbol: usize,
    next_position: usize,
}

impl AttributeSymbolPlan {
    fn first_position(&self) -> usize {
        self.prefix_segments
            .first()
            .map_or(self.final_symbol, |(segment, _)| *segment)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CompoundReferencePlan {
    namespace_segments: Vec<usize>,
    namespace_separators: Vec<usize>,
    operator: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct QualifiedReferencePlan {
    namespace_segments: Vec<usize>,
    namespace_separators: Vec<usize>,
    final_separator: usize,
    final_reference: usize,
    next_position: usize,
}

impl QualifiedReferencePlan {
    fn first_position(&self) -> usize {
        self.namespace_segments[0]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct FormulaConnectiveToken {
    connective: SurfaceFormulaConnective,
    repeated: bool,
    token_count: usize,
    left_binding_power: u32,
    right_binding_power: u32,
}

impl Parser {
    pub(super) fn parse_compilation_unit(&mut self) -> ParsedCompilationUnit {
        let module_range = self.module_range();
        let mut item_children = Vec::new();
        let mut recovery_nodes = Vec::new();

        if self.should_parse_module_skeleton() {
            let mut position = 0;
            let mut import_prelude_open = true;
            let mut export_prelude_open = true;
            while position < self.request.tokens.len() {
                if let Some(item_head) = self.item_head_position(position) {
                    let direct_item_head = item_head == position;
                    if self.is_reserved_word_at(item_head, "import") {
                        if import_prelude_open && direct_item_head {
                            let item = self.parse_import_item(position);
                            item_children.push(item.id);
                            recovery_nodes.extend(item.recovery_nodes);
                            position = item.next_position;
                            continue;
                        }
                        import_prelude_open = false;
                        export_prelude_open = false;
                        let recovery = self.recover_unexpected_top_level_tokens(position);
                        item_children.push(recovery.id);
                        recovery_nodes.extend(recovery.recovery_nodes);
                        position = recovery.next_position;
                        continue;
                    }
                    import_prelude_open = false;
                    if self.is_reserved_word_at(item_head, "export") {
                        if export_prelude_open && direct_item_head {
                            let item = self.parse_export_item(position);
                            item_children.push(item.id);
                            recovery_nodes.extend(item.recovery_nodes);
                            position = item.next_position;
                            continue;
                        }
                        export_prelude_open = false;
                        let recovery = self.recover_unexpected_top_level_tokens(position);
                        item_children.push(recovery.id);
                        recovery_nodes.extend(recovery.recovery_nodes);
                        position = recovery.next_position;
                        continue;
                    }
                    export_prelude_open = false;
                    if self.is_reserved_word_at(item_head, "reserve") {
                        let item = self.parse_reserve_item(position);
                        item_children.push(item.id);
                        recovery_nodes.extend(item.recovery_nodes);
                        position = item.next_position;
                        continue;
                    }
                    if let Some(item) = self.parse_definition_block_item(position) {
                        item_children.push(item.id);
                        recovery_nodes.extend(item.recovery_nodes);
                        position = item.next_position;
                        continue;
                    }
                    if let Some(item) = self.parse_theorem_item(position) {
                        item_children.push(item.id);
                        recovery_nodes.extend(item.recovery_nodes);
                        position = item.next_position;
                        continue;
                    }
                    if self.is_notation_alias_start_at(item_head) {
                        let item = self.parse_notation_alias_item(position);
                        item_children.push(item.id);
                        recovery_nodes.extend(item.recovery_nodes);
                        position = item.next_position;
                        continue;
                    }
                    if self.is_visibility_marker_at(item_head) {
                        let item = self.parse_visible_item(position);
                        item_children.push(item.id);
                        recovery_nodes.extend(item.recovery_nodes);
                        position = item.next_position;
                        continue;
                    }
                    let item = self.parse_placeholder_item(position);
                    item_children.push(item.id);
                    position = item.next_position;
                    continue;
                }
                if self.is_statement_start_at(position) {
                    import_prelude_open = false;
                    export_prelude_open = false;
                    let item = self.parse_statement_item_or_placeholder(position);
                    item_children.push(item.id);
                    recovery_nodes.extend(item.recovery_nodes);
                    position = item.next_position;
                    continue;
                }
                let recovery = self.recover_unexpected_top_level_tokens(position);
                item_children.push(recovery.id);
                recovery_nodes.extend(recovery.recovery_nodes);
                position = recovery.next_position;
            }
        }

        let item_list = self.events.emit(SyntaxEvent::Node {
            kind: SurfaceNodeKind::ItemList,
            range: module_range,
            children: item_children,
        });
        let compilation_unit = self.events.emit(SyntaxEvent::Node {
            kind: SurfaceNodeKind::CompilationUnit,
            range: module_range,
            children: vec![item_list],
        });
        ParsedCompilationUnit {
            id: compilation_unit,
            recovery_nodes,
        }
    }

    fn parse_statement_item_or_placeholder(&mut self, position: usize) -> ParsedItem {
        let statement = self.parse_statement_at(position);
        let Some(statement) = statement else {
            return self.parse_deferred_statement_placeholder_item(position);
        };
        let id = self.events.emit(SyntaxEvent::Node {
            kind: SurfaceNodeKind::StatementItem,
            range: self.covering_token_range(position, statement.next_position),
            children: vec![statement.id],
        });
        ParsedItem {
            id,
            next_position: statement.next_position,
            recovery_nodes: statement.recovery_nodes,
        }
    }

    fn parse_statement_at(&mut self, position: usize) -> Option<ParsedTypeNode> {
        self.parse_then_statement_at(position)
            .or_else(|| self.parse_conclusion_statement_at(position))
            .or_else(|| self.parse_now_statement_at(position))
            .or_else(|| self.parse_hereby_statement_at(position))
            .or_else(|| self.parse_case_reasoning_statement_at(position))
            .or_else(|| self.parse_iterative_equality_statement_at(position))
            .or_else(|| self.parse_simple_statement_at(position))
            .or_else(|| self.parse_compact_statement_at(position))
    }

    fn parse_linkable_statement_at(&mut self, position: usize) -> Option<ParsedTypeNode> {
        self.parse_conclusion_statement_at(position)
            .or_else(|| self.parse_case_reasoning_statement_at(position))
            .or_else(|| self.parse_iterative_equality_statement_at(position))
            .or_else(|| {
                self.is_reserved_word_at(position, "consider")
                    .then(|| self.parse_consider_statement_at(position))
            })
            .or_else(|| {
                self.is_reserved_word_at(position, "reconsider")
                    .then(|| self.parse_reconsider_statement_at(position))
            })
            .or_else(|| self.parse_compact_statement_at(position))
    }

    fn parse_simple_statement_at(&mut self, position: usize) -> Option<ParsedTypeNode> {
        let token = self.request.tokens.get(position)?;
        if token.kind != ParserTokenKind::ReservedWord {
            return None;
        }
        match token.text.as_ref() {
            "let" => self.parse_let_statement_at(position),
            "assume" => Some(self.parse_assumption_statement_at(position)),
            "given" => Some(self.parse_given_statement_at(position)),
            "take" => Some(self.parse_take_statement_at(position)),
            "set" => Some(self.parse_set_statement_at(position)),
            "consider" => Some(self.parse_consider_statement_at(position)),
            "reconsider" => Some(self.parse_reconsider_statement_at(position)),
            "deffunc" => Some(self.parse_inline_functor_definition_at(position)),
            "defpred" => Some(self.parse_inline_predicate_definition_at(position)),
            _ => None,
        }
    }

    fn parse_then_statement_at(&mut self, position: usize) -> Option<ParsedTypeNode> {
        if !self.is_reserved_word_at(position, "then") {
            return None;
        }

        let mut children = vec![self.token_node_ids[position]];
        let mut recovery_nodes = Vec::new();
        let mut cursor = position + 1;

        if let Some(statement) = self.parse_linkable_statement_at(cursor) {
            cursor = statement.next_position;
            children.push(statement.id);
            recovery_nodes.extend(statement.recovery_nodes);
            let id = self.events.emit(SyntaxEvent::Node {
                kind: SurfaceNodeKind::ThenStatement,
                range: self.covering_token_range(position, cursor),
                children,
            });
            return Some(ParsedTypeNode {
                id,
                next_position: cursor.max(position + 1),
                recovery_nodes,
            });
        } else {
            self.diagnose_malformed_formula_expression(
                cursor,
                "expected linkable statement after `then`",
            );
            let missing = self.add_missing_statement(cursor);
            children.push(missing);
            recovery_nodes.push(missing);
        }

        Some(self.finish_simple_statement_node(
            position,
            cursor,
            SurfaceNodeKind::ThenStatement,
            children,
            recovery_nodes,
            "unexpected token in then statement",
        ))
    }

    fn parse_now_statement_at(&mut self, position: usize) -> Option<ParsedTypeNode> {
        let mut children = Vec::new();
        let mut cursor = position;

        if self.is_identifier_at(cursor) && self.is_reserved_symbol_at(cursor + 1, ":") {
            if !self.is_reserved_word_at(cursor + 2, "now") {
                return None;
            }
            children.push(self.token_node_ids[cursor]);
            children.push(self.token_node_ids[cursor + 1]);
            cursor += 2;
        } else if !self.is_reserved_word_at(cursor, "now") {
            return None;
        }

        let opener = cursor;
        children.push(self.token_node_ids[cursor]);
        cursor += 1;
        let mut recovery_nodes = Vec::new();

        cursor = self.parse_reasoning_body_at(cursor, &mut children, &mut recovery_nodes);
        cursor =
            self.parse_required_end_semicolon(opener, cursor, &mut children, &mut recovery_nodes);

        let id = self.events.emit(SyntaxEvent::Node {
            kind: SurfaceNodeKind::NowStatement,
            range: self.covering_token_range(position, cursor),
            children,
        });
        Some(ParsedTypeNode {
            id,
            next_position: cursor.max(position + 1),
            recovery_nodes,
        })
    }

    fn parse_hereby_statement_at(&mut self, position: usize) -> Option<ParsedTypeNode> {
        if !self.is_reserved_word_at(position, "hereby") {
            return None;
        }
        let mut children = vec![self.token_node_ids[position]];
        let mut recovery_nodes = Vec::new();
        let mut cursor = position + 1;

        cursor = self.parse_reasoning_body_at(cursor, &mut children, &mut recovery_nodes);
        cursor =
            self.parse_required_end_semicolon(position, cursor, &mut children, &mut recovery_nodes);

        let id = self.events.emit(SyntaxEvent::Node {
            kind: SurfaceNodeKind::HerebyStatement,
            range: self.covering_token_range(position, cursor),
            children,
        });
        Some(ParsedTypeNode {
            id,
            next_position: cursor.max(position + 1),
            recovery_nodes,
        })
    }

    fn parse_case_reasoning_statement_at(&mut self, position: usize) -> Option<ParsedTypeNode> {
        if !self.is_case_reasoning_start_at(position) {
            return None;
        }
        let mut children = vec![
            self.token_node_ids[position],
            self.token_node_ids[position + 1],
        ];
        let mut recovery_nodes = Vec::new();
        let mut cursor = position + 2;

        if self.is_reserved_word_at(cursor, "by") {
            let justification = self.parse_justification_clause_at(cursor, false);
            cursor = justification.next_position;
            children.push(justification.id);
            recovery_nodes.extend(justification.recovery_nodes);
        }
        cursor = self.parse_required_statement_semicolon(cursor, &mut children);

        let mut branch_kind = None;
        while cursor < self.request.tokens.len() {
            if self.is_end_keyword_at(cursor) || self.is_item_start_at(cursor) {
                break;
            }
            let current_kind = if self.is_reserved_word_at(cursor, "case") {
                Some(CaseBranchKind::Case)
            } else if self.is_reserved_word_at(cursor, "suppose") {
                Some(CaseBranchKind::Suppose)
            } else {
                None
            };

            let Some(current_kind) = current_kind else {
                break;
            };
            if let Some(expected_kind) = branch_kind {
                if current_kind != expected_kind {
                    self.diagnose_malformed_formula_expression(
                        cursor,
                        "case reasoning branches must not mix `case` and `suppose`",
                    );
                    break;
                }
            } else {
                branch_kind = Some(current_kind);
            }

            let branch = self.parse_case_branch_item_at(cursor, current_kind);
            cursor = branch.next_position;
            children.push(branch.id);
            recovery_nodes.extend(branch.recovery_nodes);
        }

        let id = self.events.emit(SyntaxEvent::Node {
            kind: SurfaceNodeKind::CaseReasoningStatement,
            range: self.covering_token_range(position, cursor),
            children,
        });
        Some(ParsedTypeNode {
            id,
            next_position: cursor.max(position + 1),
            recovery_nodes,
        })
    }

    fn parse_case_branch_item_at(
        &mut self,
        position: usize,
        branch_kind: CaseBranchKind,
    ) -> ParsedTypeNode {
        let surface_kind = match branch_kind {
            CaseBranchKind::Case => SurfaceNodeKind::CaseItem,
            CaseBranchKind::Suppose => SurfaceNodeKind::SupposeItem,
        };
        let mut children = vec![self.token_node_ids[position]];
        let mut recovery_nodes = Vec::new();
        let mut cursor = position + 1;

        if let Some(conditions) = self.parse_condition_list_at(cursor) {
            cursor = conditions.next_position;
            children.push(conditions.id);
            recovery_nodes.extend(conditions.recovery_nodes);
        } else {
            let proposition = self.parse_proposition_at(cursor);
            cursor = proposition.next_position;
            children.push(proposition.id);
            recovery_nodes.extend(proposition.recovery_nodes);
        }

        cursor = self.parse_required_statement_semicolon(cursor, &mut children);
        cursor = self.parse_reasoning_body_at(cursor, &mut children, &mut recovery_nodes);
        cursor =
            self.parse_required_end_semicolon(position, cursor, &mut children, &mut recovery_nodes);

        let id = self.events.emit(SyntaxEvent::Node {
            kind: surface_kind,
            range: self.covering_token_range(position, cursor),
            children,
        });
        ParsedTypeNode {
            id,
            next_position: cursor,
            recovery_nodes,
        }
    }

    fn parse_conclusion_statement_at(&mut self, position: usize) -> Option<ParsedTypeNode> {
        if !self.is_conclusion_keyword_at(position) {
            return None;
        }
        let mut children = vec![self.token_node_ids[position]];
        let mut recovery_nodes = Vec::new();
        let mut cursor = position + 1;

        let proposition = self.parse_proposition_at(cursor);
        cursor = proposition.next_position;
        children.push(proposition.id);
        recovery_nodes.extend(proposition.recovery_nodes);

        if let Some(justification) = self.parse_general_justification_at(cursor, true) {
            cursor = justification.next_position;
            children.push(justification.id);
            recovery_nodes.extend(justification.recovery_nodes);
        }

        Some(self.finish_simple_statement_node(
            position,
            cursor,
            SurfaceNodeKind::ConclusionStatement,
            children,
            recovery_nodes,
            "unexpected token in conclusion statement",
        ))
    }

    fn parse_iterative_equality_statement_at(&mut self, position: usize) -> Option<ParsedTypeNode> {
        if !self.is_iterative_equality_statement_start_at(position) {
            return None;
        }
        let mut children = Vec::new();
        let mut recovery_nodes = Vec::new();
        let mut cursor = position;

        if self.is_identifier_at(cursor) && self.is_reserved_symbol_at(cursor + 1, ":") {
            children.push(self.token_node_ids[cursor]);
            children.push(self.token_node_ids[cursor + 1]);
            cursor += 2;
        }

        cursor = self.parse_required_iterative_equality_term(
            cursor,
            &mut children,
            &mut recovery_nodes,
            "expected left term in iterative equality",
        );

        if self.is_reserved_symbol_at(cursor, "=") {
            children.push(self.token_node_ids[cursor]);
            cursor += 1;
        } else {
            self.diagnose_malformed_term_expression(cursor, "expected `=` in iterative equality");
        }

        cursor = self.parse_required_iterative_equality_term(
            cursor,
            &mut children,
            &mut recovery_nodes,
            "expected right term in iterative equality",
        );

        if self.is_reserved_word_at(cursor, "by") {
            let justification = self.parse_justification_clause_at(cursor, false);
            cursor = justification.next_position;
            children.push(justification.id);
            recovery_nodes.extend(justification.recovery_nodes);
        }

        while self.is_reserved_symbol_at(cursor, ".=") {
            let step = self.parse_iterative_equality_step_at(cursor);
            cursor = step.next_position;
            children.push(step.id);
            recovery_nodes.extend(step.recovery_nodes);
        }

        Some(self.finish_simple_statement_node(
            position,
            cursor,
            SurfaceNodeKind::IterativeEqualityStatement,
            children,
            recovery_nodes,
            "unexpected token in iterative equality statement",
        ))
    }

    fn parse_iterative_equality_step_at(&mut self, position: usize) -> ParsedTypeNode {
        let mut children = vec![self.token_node_ids[position]];
        let mut recovery_nodes = Vec::new();
        let mut cursor = position + 1;

        cursor = self.parse_required_iterative_equality_term(
            cursor,
            &mut children,
            &mut recovery_nodes,
            "expected term after `.=`",
        );

        if self.is_reserved_word_at(cursor, "by") {
            let justification = self.parse_justification_clause_at(cursor, false);
            cursor = justification.next_position;
            children.push(justification.id);
            recovery_nodes.extend(justification.recovery_nodes);
        }

        let id = self.events.emit(SyntaxEvent::Node {
            kind: SurfaceNodeKind::IterativeEqualityStep,
            range: self.covering_token_range(position, cursor.max(position + 1)),
            children,
        });
        ParsedTypeNode {
            id,
            next_position: cursor.max(position + 1),
            recovery_nodes,
        }
    }

    fn parse_required_iterative_equality_term(
        &mut self,
        cursor: usize,
        children: &mut Vec<SurfaceBuilderNodeId>,
        recovery_nodes: &mut Vec<SurfaceBuilderNodeId>,
        message: &'static str,
    ) -> usize {
        if let Some(term) = self.parse_term_expression_at(cursor) {
            children.push(term.id);
            recovery_nodes.extend(term.recovery_nodes);
            term.next_position
        } else {
            self.diagnose_malformed_term_expression(cursor, message);
            self.push_missing_term(cursor, children, recovery_nodes);
            cursor
        }
    }

    fn parse_let_statement_at(&mut self, position: usize) -> Option<ParsedTypeNode> {
        let mut children = vec![self.token_node_ids[position]];
        let mut recovery_nodes = Vec::new();
        let mut cursor = position + 1;

        cursor = self.parse_qualified_variable_segments(
            cursor,
            &mut children,
            &mut recovery_nodes,
            "expected variable segment after `let`",
            "expected variable segment after `,` in let statement",
        );
        cursor =
            self.parse_optional_such_condition_list(cursor, &mut children, &mut recovery_nodes);
        if self.is_reserved_word_at(cursor, "by") {
            let justification = self.parse_justification_clause_at(cursor, false);
            cursor = justification.next_position;
            children.push(justification.id);
            recovery_nodes.extend(justification.recovery_nodes);
        }

        Some(self.finish_simple_statement_node(
            position,
            cursor,
            SurfaceNodeKind::LetStatement,
            children,
            recovery_nodes,
            "unexpected token in let statement",
        ))
    }

    fn parse_assumption_statement_at(&mut self, position: usize) -> ParsedTypeNode {
        let mut children = vec![self.token_node_ids[position]];
        let mut recovery_nodes = Vec::new();
        let mut cursor = position + 1;

        if let Some(condition_list) = self.parse_condition_list_at(cursor) {
            cursor = condition_list.next_position;
            children.push(condition_list.id);
            recovery_nodes.extend(condition_list.recovery_nodes);
        } else {
            let proposition = self.parse_proposition_at(cursor);
            cursor = proposition.next_position;
            children.push(proposition.id);
            recovery_nodes.extend(proposition.recovery_nodes);
        }

        self.finish_simple_statement_node(
            position,
            cursor,
            SurfaceNodeKind::AssumptionStatement,
            children,
            recovery_nodes,
            "unexpected token in assumption statement",
        )
    }

    fn parse_given_statement_at(&mut self, position: usize) -> ParsedTypeNode {
        let mut children = vec![self.token_node_ids[position]];
        let mut recovery_nodes = Vec::new();
        let mut cursor = position + 1;

        cursor = self.parse_qualified_variable_segments(
            cursor,
            &mut children,
            &mut recovery_nodes,
            "expected variable segment after `given`",
            "expected variable segment after `,` in given statement",
        );
        cursor =
            self.parse_optional_such_condition_list(cursor, &mut children, &mut recovery_nodes);

        self.finish_simple_statement_node(
            position,
            cursor,
            SurfaceNodeKind::GivenStatement,
            children,
            recovery_nodes,
            "unexpected token in given statement",
        )
    }

    fn parse_take_statement_at(&mut self, position: usize) -> ParsedTypeNode {
        let mut children = vec![self.token_node_ids[position]];
        let mut recovery_nodes = Vec::new();
        let mut cursor = position + 1;

        let witness = self.parse_witness_at(cursor);
        cursor = witness.next_position;
        children.push(witness.id);
        recovery_nodes.extend(witness.recovery_nodes);

        while self.is_reserved_symbol_at(cursor, ",") {
            children.push(self.token_node_ids[cursor]);
            cursor += 1;
            let witness = self.parse_witness_at(cursor);
            let made_progress = witness.next_position > cursor;
            cursor = witness.next_position;
            children.push(witness.id);
            recovery_nodes.extend(witness.recovery_nodes);
            if !made_progress {
                break;
            }
        }

        self.finish_simple_statement_node(
            position,
            cursor,
            SurfaceNodeKind::TakeStatement,
            children,
            recovery_nodes,
            "unexpected token in take statement",
        )
    }

    fn parse_set_statement_at(&mut self, position: usize) -> ParsedTypeNode {
        let mut children = vec![self.token_node_ids[position]];
        let mut recovery_nodes = Vec::new();
        let mut cursor = position + 1;

        let equating = self.parse_equating_at(cursor);
        cursor = equating.next_position;
        children.push(equating.id);
        recovery_nodes.extend(equating.recovery_nodes);

        while self.is_reserved_symbol_at(cursor, ",") {
            children.push(self.token_node_ids[cursor]);
            cursor += 1;
            let equating = self.parse_equating_at(cursor);
            let made_progress = equating.next_position > cursor;
            cursor = equating.next_position;
            children.push(equating.id);
            recovery_nodes.extend(equating.recovery_nodes);
            if !made_progress {
                break;
            }
        }

        self.finish_simple_statement_node(
            position,
            cursor,
            SurfaceNodeKind::SetStatement,
            children,
            recovery_nodes,
            "unexpected token in set statement",
        )
    }

    fn parse_consider_statement_at(&mut self, position: usize) -> ParsedTypeNode {
        let mut children = vec![self.token_node_ids[position]];
        let mut recovery_nodes = Vec::new();
        let mut cursor = position + 1;

        cursor = self.parse_qualified_variable_segments(
            cursor,
            &mut children,
            &mut recovery_nodes,
            "expected variable segment after `consider`",
            "expected variable segment after `,` in consider statement",
        );
        cursor = self.parse_mandatory_such_condition_list(
            cursor,
            &mut children,
            &mut recovery_nodes,
            "expected `such` in consider statement",
            "expected `that` after `such` in consider statement",
        );
        cursor = self.parse_mandatory_simple_justification(
            cursor,
            &mut children,
            &mut recovery_nodes,
            "expected `by` justification in consider statement",
        );

        self.finish_simple_statement_node(
            position,
            cursor,
            SurfaceNodeKind::ConsiderStatement,
            children,
            recovery_nodes,
            "unexpected token in consider statement",
        )
    }

    fn parse_reconsider_statement_at(&mut self, position: usize) -> ParsedTypeNode {
        let mut children = vec![self.token_node_ids[position]];
        let mut recovery_nodes = Vec::new();
        let mut cursor = position + 1;

        let item = self.parse_reconsider_item_at(cursor);
        cursor = item.next_position;
        children.push(item.id);
        recovery_nodes.extend(item.recovery_nodes);

        while self.is_reserved_symbol_at(cursor, ",") {
            children.push(self.token_node_ids[cursor]);
            cursor += 1;
            let item = self.parse_reconsider_item_at(cursor);
            let made_progress = item.next_position > cursor;
            cursor = item.next_position;
            children.push(item.id);
            recovery_nodes.extend(item.recovery_nodes);
            if !made_progress {
                break;
            }
        }

        if self.is_reserved_word_at(cursor, "as") {
            children.push(self.token_node_ids[cursor]);
            cursor += 1;
            if let Some(type_expression) = self.parse_type_expression_at(cursor) {
                cursor = type_expression.next_position;
                children.push(type_expression.id);
                recovery_nodes.extend(type_expression.recovery_nodes);
            } else {
                self.diagnose_malformed_type_expression(
                    cursor,
                    "expected target type after `as` in reconsider statement",
                );
                let missing = self.add_missing_type_expression(cursor);
                children.push(missing);
                recovery_nodes.push(missing);
            }
        } else {
            self.diagnose_malformed_type_expression(
                cursor,
                "expected `as` in reconsider statement",
            );
            let missing = self.add_missing_type_expression(cursor);
            children.push(missing);
            recovery_nodes.push(missing);
        }

        cursor = self.parse_mandatory_simple_justification(
            cursor,
            &mut children,
            &mut recovery_nodes,
            "expected `by` justification in reconsider statement",
        );

        self.finish_simple_statement_node(
            position,
            cursor,
            SurfaceNodeKind::ReconsiderStatement,
            children,
            recovery_nodes,
            "unexpected token in reconsider statement",
        )
    }

    fn parse_inline_functor_definition_at(&mut self, position: usize) -> ParsedTypeNode {
        let mut children = vec![self.token_node_ids[position]];
        let mut recovery_nodes = Vec::new();
        let mut cursor = position + 1;

        cursor = self.parse_inline_definition_name_at(cursor, &mut children, &mut recovery_nodes);
        cursor =
            self.parse_inline_definition_parameters_at(cursor, &mut children, &mut recovery_nodes);

        if self.is_reserved_symbol_at(cursor, "->") {
            children.push(self.token_node_ids[cursor]);
            cursor += 1;
        } else {
            self.diagnose_malformed_type_expression(
                cursor,
                "expected `->` before inline functor return type",
            );
        }

        cursor = self.parse_required_inline_definition_type(
            cursor,
            &mut children,
            &mut recovery_nodes,
            "expected inline functor return type",
        );

        if self.is_reserved_word_at(cursor, "equals") {
            children.push(self.token_node_ids[cursor]);
            cursor += 1;
        } else {
            self.diagnose_malformed_term_expression(
                cursor,
                "expected `equals` before inline functor body",
            );
        }

        cursor = self.parse_required_inline_definition_term(
            cursor,
            &mut children,
            &mut recovery_nodes,
            "expected inline functor body term",
        );

        self.finish_simple_statement_node(
            position,
            cursor,
            SurfaceNodeKind::InlineFunctorDefinition,
            children,
            recovery_nodes,
            "unexpected token in inline functor definition",
        )
    }

    fn parse_inline_predicate_definition_at(&mut self, position: usize) -> ParsedTypeNode {
        let mut children = vec![self.token_node_ids[position]];
        let mut recovery_nodes = Vec::new();
        let mut cursor = position + 1;

        cursor = self.parse_inline_definition_name_at(cursor, &mut children, &mut recovery_nodes);
        cursor =
            self.parse_inline_definition_parameters_at(cursor, &mut children, &mut recovery_nodes);

        if self.is_reserved_word_at(cursor, "means") {
            children.push(self.token_node_ids[cursor]);
            cursor += 1;
        } else {
            self.diagnose_malformed_formula_expression(
                cursor,
                "expected `means` before inline predicate body",
            );
        }

        cursor = self.parse_required_inline_definition_formula(
            cursor,
            &mut children,
            &mut recovery_nodes,
            "expected inline predicate body formula",
        );

        self.finish_simple_statement_node(
            position,
            cursor,
            SurfaceNodeKind::InlinePredicateDefinition,
            children,
            recovery_nodes,
            "unexpected token in inline predicate definition",
        )
    }

    fn parse_inline_definition_name_at(
        &mut self,
        cursor: usize,
        children: &mut Vec<SurfaceBuilderNodeId>,
        recovery_nodes: &mut Vec<SurfaceBuilderNodeId>,
    ) -> usize {
        if self.is_identifier_at(cursor) {
            children.push(self.token_node_ids[cursor]);
            cursor + 1
        } else {
            self.diagnose_malformed_term_expression(cursor, "expected inline definition name");
            self.push_missing_term(cursor, children, recovery_nodes);
            cursor
        }
    }

    fn parse_inline_definition_parameters_at(
        &mut self,
        mut cursor: usize,
        children: &mut Vec<SurfaceBuilderNodeId>,
        recovery_nodes: &mut Vec<SurfaceBuilderNodeId>,
    ) -> usize {
        if self.is_reserved_symbol_at(cursor, "(") {
            let opener = cursor;
            children.push(self.token_node_ids[cursor]);
            cursor += 1;

            if self.is_reserved_symbol_at(cursor, ")") {
                children.push(self.token_node_ids[cursor]);
                return cursor + 1;
            }

            loop {
                if self.is_inline_definition_parameter_boundary_at(cursor) {
                    if !self.is_reserved_symbol_at(cursor, ")") {
                        self.diagnose_malformed_term_expression(
                            cursor,
                            "expected inline definition parameter",
                        );
                    }
                    break;
                }

                let parameter = self.parse_typed_parameter_at(cursor);
                let made_progress = parameter.next_position > cursor;
                cursor = parameter.next_position;
                children.push(parameter.id);
                recovery_nodes.extend(parameter.recovery_nodes);

                if self.is_reserved_symbol_at(cursor, ",") {
                    children.push(self.token_node_ids[cursor]);
                    cursor += 1;
                    if self.is_reserved_symbol_at(cursor, ")") {
                        self.diagnose_malformed_term_expression(
                            cursor,
                            "expected inline definition parameter after `,`",
                        );
                    }
                    continue;
                }
                if !made_progress {
                    break;
                }
                break;
            }

            if self.is_reserved_symbol_at(cursor, ")") {
                children.push(self.token_node_ids[cursor]);
                cursor += 1;
            } else {
                self.diagnose_unmatched_term_delimiter(opener, cursor, ")");
                let recovery = self.add_recovery_node(
                    SyntaxRecoveryKind::UnmatchedOpeningDelimiter,
                    self.zero_range_at(cursor),
                    Vec::new(),
                );
                children.push(recovery);
                recovery_nodes.push(recovery);
            }
        } else {
            self.diagnose_malformed_term_expression(
                cursor,
                "expected `(` after inline definition name",
            );
            cursor = self.parse_inline_definition_parameter_sequence_without_parens(
                cursor,
                children,
                recovery_nodes,
            );
        }

        cursor
    }

    fn parse_inline_definition_parameter_sequence_without_parens(
        &mut self,
        mut cursor: usize,
        children: &mut Vec<SurfaceBuilderNodeId>,
        recovery_nodes: &mut Vec<SurfaceBuilderNodeId>,
    ) -> usize {
        while !self.is_inline_definition_parameter_boundary_at(cursor) {
            let parameter = self.parse_typed_parameter_at(cursor);
            let made_progress = parameter.next_position > cursor;
            cursor = parameter.next_position;
            children.push(parameter.id);
            recovery_nodes.extend(parameter.recovery_nodes);

            if self.is_reserved_symbol_at(cursor, ",") {
                children.push(self.token_node_ids[cursor]);
                cursor += 1;
                continue;
            }
            if !made_progress
                && let Some(recovery) = self.recover_malformed_inline_definition_tail(cursor)
            {
                cursor = recovery.next_position;
                children.push(recovery.id);
                recovery_nodes.extend(recovery.recovery_nodes);
            }
            break;
        }
        cursor
    }

    fn parse_typed_parameter_at(&mut self, position: usize) -> ParsedTypeNode {
        let mut children = Vec::new();
        let mut recovery_nodes = Vec::new();
        let mut cursor = position;

        if self.is_identifier_at(cursor) {
            children.push(self.token_node_ids[cursor]);
            cursor += 1;
        } else {
            self.diagnose_malformed_term_expression(
                cursor,
                "expected parameter name in inline definition",
            );
            self.push_missing_term(cursor, &mut children, &mut recovery_nodes);
        }

        if self.is_reserved_word_at(cursor, "be") || self.is_reserved_word_at(cursor, "being") {
            children.push(self.token_node_ids[cursor]);
            cursor += 1;
        } else {
            self.diagnose_malformed_type_expression(
                cursor,
                "expected `be` or `being` in inline definition parameter",
            );
        }

        match self.parse_type_expression_at(cursor) {
            Some(type_expression) => {
                cursor = type_expression.next_position;
                children.push(type_expression.id);
                recovery_nodes.extend(type_expression.recovery_nodes);
            }
            None if self.is_inline_definition_parameter_boundary_at(cursor) => {
                self.diagnose_malformed_type_expression(
                    cursor,
                    "expected type in inline definition parameter",
                );
                let missing = self.add_missing_type_expression(cursor);
                children.push(missing);
                recovery_nodes.push(missing);
            }
            None => {
                self.diagnose_malformed_type_expression(
                    cursor,
                    "malformed type in inline definition parameter",
                );
                if let Some(recovery) = self.recover_malformed_inline_definition_tail(cursor) {
                    cursor = recovery.next_position;
                    children.push(recovery.id);
                    recovery_nodes.extend(recovery.recovery_nodes);
                }
                let missing = self.add_missing_type_expression(cursor);
                children.push(missing);
                recovery_nodes.push(missing);
            }
        }

        let range = if cursor > position {
            self.covering_token_range(position, cursor)
        } else {
            self.zero_range_at(position)
        };
        let id = self.events.emit(SyntaxEvent::Node {
            kind: SurfaceNodeKind::TypedParameter,
            range,
            children,
        });
        ParsedTypeNode {
            id,
            next_position: cursor.max(position + 1),
            recovery_nodes,
        }
    }

    fn parse_required_inline_definition_type(
        &mut self,
        cursor: usize,
        children: &mut Vec<SurfaceBuilderNodeId>,
        recovery_nodes: &mut Vec<SurfaceBuilderNodeId>,
        message: &'static str,
    ) -> usize {
        if let Some(type_expression) = self.parse_type_expression_at(cursor) {
            children.push(type_expression.id);
            recovery_nodes.extend(type_expression.recovery_nodes);
            type_expression.next_position
        } else {
            self.diagnose_malformed_type_expression(cursor, message);
            let mut cursor = cursor;
            if !self.is_inline_definition_parameter_boundary_at(cursor)
                && let Some(recovery) = self.recover_malformed_inline_definition_tail(cursor)
            {
                cursor = recovery.next_position;
                children.push(recovery.id);
                recovery_nodes.extend(recovery.recovery_nodes);
            }
            let missing = self.add_missing_type_expression(cursor);
            children.push(missing);
            recovery_nodes.push(missing);
            cursor
        }
    }

    fn parse_required_inline_definition_term(
        &mut self,
        cursor: usize,
        children: &mut Vec<SurfaceBuilderNodeId>,
        recovery_nodes: &mut Vec<SurfaceBuilderNodeId>,
        message: &'static str,
    ) -> usize {
        if let Some(term) = self.parse_term_expression_at(cursor) {
            children.push(term.id);
            recovery_nodes.extend(term.recovery_nodes);
            term.next_position
        } else {
            self.diagnose_malformed_term_expression(cursor, message);
            self.push_missing_term(cursor, children, recovery_nodes);
            cursor
        }
    }

    fn parse_required_inline_definition_formula(
        &mut self,
        cursor: usize,
        children: &mut Vec<SurfaceBuilderNodeId>,
        recovery_nodes: &mut Vec<SurfaceBuilderNodeId>,
        message: &'static str,
    ) -> usize {
        if let Some(formula) = self.parse_formula_expression_at(cursor) {
            children.push(formula.id);
            recovery_nodes.extend(formula.recovery_nodes);
            formula.next_position
        } else {
            self.diagnose_malformed_formula_expression(cursor, message);
            let missing = self.add_missing_formula(cursor);
            children.push(missing);
            recovery_nodes.push(missing);
            cursor
        }
    }

    fn parse_compact_statement_at(&mut self, position: usize) -> Option<ParsedTypeNode> {
        if !self.is_compact_statement_start_at(position) {
            return None;
        }
        let mut children = Vec::new();
        let mut recovery_nodes = Vec::new();

        let proposition = self.parse_proposition_at(position);
        let mut cursor = proposition.next_position;
        children.push(proposition.id);
        recovery_nodes.extend(proposition.recovery_nodes);

        let justification = self.parse_general_justification_at(cursor, true)?;
        cursor = justification.next_position;
        children.push(justification.id);
        recovery_nodes.extend(justification.recovery_nodes);

        Some(self.finish_simple_statement_node(
            position,
            cursor,
            SurfaceNodeKind::CompactStatement,
            children,
            recovery_nodes,
            "unexpected token in compact statement",
        ))
    }

    fn parse_general_justification_at(
        &mut self,
        position: usize,
        allow_computation: bool,
    ) -> Option<ParsedTypeNode> {
        if self.is_reserved_word_at(position, "by") {
            Some(self.parse_justification_clause_at(position, allow_computation))
        } else if self.is_reserved_word_at(position, "proof") {
            Some(self.parse_proof_block_at(position))
        } else {
            None
        }
    }

    fn parse_definition_content_general_justification_at(
        &mut self,
        position: usize,
        allow_computation: bool,
    ) -> Option<ParsedTypeNode> {
        if self.is_reserved_word_at(position, "by") {
            Some(self.parse_definition_content_justification_clause_at(position, allow_computation))
        } else if self.is_reserved_word_at(position, "proof") {
            Some(self.parse_definition_content_proof_block_at(position))
        } else {
            None
        }
    }

    fn parse_justification_clause_at(
        &mut self,
        position: usize,
        allow_computation: bool,
    ) -> ParsedTypeNode {
        self.parse_justification_clause_with_definition_boundary_at(
            position,
            allow_computation,
            false,
        )
    }

    fn parse_definition_content_justification_clause_at(
        &mut self,
        position: usize,
        allow_computation: bool,
    ) -> ParsedTypeNode {
        self.parse_justification_clause_with_definition_boundary_at(
            position,
            allow_computation,
            true,
        )
    }

    fn parse_justification_clause_with_definition_boundary_at(
        &mut self,
        position: usize,
        allow_computation: bool,
        stop_at_definition_content_start: bool,
    ) -> ParsedTypeNode {
        let mut children = vec![self.token_node_ids[position]];
        let mut recovery_nodes = Vec::new();
        let mut cursor = position + 1;

        if allow_computation && self.is_reserved_word_at(cursor, "computation") {
            let computation = self.parse_computation_justification_with_definition_boundary_at(
                cursor,
                stop_at_definition_content_start,
            );
            cursor = computation.next_position;
            children.push(computation.id);
            recovery_nodes.extend(computation.recovery_nodes);
            cursor = self.recover_unexpected_justification_tail_with_definition_boundary(
                cursor,
                &mut children,
                &mut recovery_nodes,
                stop_at_definition_content_start,
            );
        } else {
            let references = self.parse_reference_list_with_definition_boundary_at(
                cursor,
                stop_at_definition_content_start,
            );
            cursor = references.next_position;
            children.push(references.id);
            recovery_nodes.extend(references.recovery_nodes);
        }

        let id = self.events.emit(SyntaxEvent::Node {
            kind: SurfaceNodeKind::JustificationClause,
            range: self.covering_token_range(position, cursor),
            children,
        });
        ParsedTypeNode {
            id,
            next_position: cursor.max(position + 1),
            recovery_nodes,
        }
    }

    fn parse_reference_list_with_definition_boundary_at(
        &mut self,
        position: usize,
        stop_at_definition_content_start: bool,
    ) -> ParsedTypeNode {
        let mut children = Vec::new();
        let mut recovery_nodes = Vec::new();
        let mut cursor = position;
        let mut expecting_reference = true;

        loop {
            if self.is_reserved_symbol_at(cursor, ",") {
                if expecting_reference {
                    self.diagnose_malformed_justification(cursor, "expected reference before `,`");
                    self.push_missing_proof_step(cursor, &mut children, &mut recovery_nodes);
                }
                children.push(self.token_node_ids[cursor]);
                cursor += 1;
                expecting_reference = true;
                continue;
            }

            if self.is_justification_recovery_boundary_at(cursor, stop_at_definition_content_start)
            {
                if expecting_reference {
                    self.diagnose_malformed_justification(cursor, "expected reference after `by`");
                    self.push_missing_proof_step(cursor, &mut children, &mut recovery_nodes);
                }
                break;
            }

            if let Some(reference) = self.parse_reference_with_definition_boundary_at(
                cursor,
                stop_at_definition_content_start,
            ) {
                cursor = reference.next_position;
                children.push(reference.id);
                recovery_nodes.extend(reference.recovery_nodes);
                cursor = self.recover_deferred_reference_template_tail(
                    cursor,
                    &mut children,
                    &mut recovery_nodes,
                );
            } else {
                self.diagnose_malformed_justification(cursor, "expected reference after `by`");
                self.push_missing_proof_step(cursor, &mut children, &mut recovery_nodes);
                if let Some(recovery) = self
                    .recover_malformed_justification_tail_with_definition_boundary(
                        cursor,
                        stop_at_definition_content_start,
                    )
                {
                    cursor = recovery.next_position;
                    children.push(recovery.id);
                    recovery_nodes.extend(recovery.recovery_nodes);
                }
            }

            if self.is_reserved_symbol_at(cursor, ",") {
                children.push(self.token_node_ids[cursor]);
                cursor += 1;
                expecting_reference = true;
                continue;
            }

            if !self.is_justification_recovery_boundary_at(cursor, stop_at_definition_content_start)
            {
                self.diagnose_malformed_justification(cursor, "unexpected token in reference list");
                if let Some(recovery) = self
                    .recover_malformed_justification_tail_with_definition_boundary(
                        cursor,
                        stop_at_definition_content_start,
                    )
                {
                    cursor = recovery.next_position;
                    children.push(recovery.id);
                    recovery_nodes.extend(recovery.recovery_nodes);
                }
            }
            break;
        }

        let range = if cursor > position {
            self.covering_token_range(position, cursor)
        } else {
            self.zero_range_at(position)
        };
        let id = self.events.emit(SyntaxEvent::Node {
            kind: SurfaceNodeKind::ReferenceList,
            range,
            children,
        });
        ParsedTypeNode {
            id,
            next_position: cursor.max(position),
            recovery_nodes,
        }
    }

    fn parse_reference_with_definition_boundary_at(
        &mut self,
        position: usize,
        stop_at_definition_content_start: bool,
    ) -> Option<ParsedTypeNode> {
        self.parse_grouped_reference_with_definition_boundary_at(
            position,
            stop_at_definition_content_start,
        )
        .or_else(|| self.parse_bulk_reference_at(position))
        .or_else(|| self.parse_qualified_reference_at(position))
        .or_else(|| self.parse_local_reference_at(position))
    }

    fn parse_local_reference_at(&mut self, position: usize) -> Option<ParsedTypeNode> {
        if !self.is_identifier_at(position) {
            return None;
        }
        let id = self.events.emit(SyntaxEvent::Node {
            kind: SurfaceNodeKind::Reference,
            range: self.request.tokens[position].span,
            children: vec![self.token_node_ids[position]],
        });
        Some(ParsedTypeNode {
            id,
            next_position: position + 1,
            recovery_nodes: Vec::new(),
        })
    }

    fn parse_qualified_reference_at(&mut self, position: usize) -> Option<ParsedTypeNode> {
        let plan = self.qualified_reference_plan_at(position)?;
        let namespace = self
            .emit_namespace_path_from_parts(&plan.namespace_segments, &plan.namespace_separators);
        let children = vec![
            namespace,
            self.token_node_ids[plan.final_separator],
            self.token_node_ids[plan.final_reference],
        ];
        let id = self.events.emit(SyntaxEvent::Node {
            kind: SurfaceNodeKind::QualifiedReference,
            range: self.covering_token_range(plan.first_position(), plan.next_position),
            children,
        });
        Some(ParsedTypeNode {
            id,
            next_position: plan.next_position,
            recovery_nodes: Vec::new(),
        })
    }

    fn parse_grouped_reference_with_definition_boundary_at(
        &mut self,
        position: usize,
        stop_at_definition_content_start: bool,
    ) -> Option<ParsedTypeNode> {
        let plan = self.compound_reference_plan_at(position, ".{")?;
        let namespace = self
            .emit_namespace_path_from_parts(&plan.namespace_segments, &plan.namespace_separators);
        let mut children = vec![namespace, self.token_node_ids[plan.operator]];
        let mut recovery_nodes = Vec::new();
        let mut cursor = plan.operator + 1;
        let mut expecting_item = true;

        loop {
            if self.is_reserved_symbol_at(cursor, ",") {
                if expecting_item {
                    self.diagnose_malformed_justification(
                        cursor,
                        "expected reference label before `,` in grouped citation",
                    );
                    self.push_missing_proof_step(cursor, &mut children, &mut recovery_nodes);
                }
                children.push(self.token_node_ids[cursor]);
                cursor += 1;
                expecting_item = true;
                continue;
            }

            if self.is_reserved_symbol_at(cursor, "}") {
                if expecting_item {
                    self.diagnose_malformed_justification(
                        cursor,
                        "expected reference label in grouped citation",
                    );
                    self.push_missing_proof_step(cursor, &mut children, &mut recovery_nodes);
                }
                break;
            }
            if self.is_justification_recovery_boundary_at(cursor, stop_at_definition_content_start)
            {
                if expecting_item {
                    self.diagnose_malformed_justification(
                        cursor,
                        "expected reference label in grouped citation",
                    );
                    self.push_missing_proof_step(cursor, &mut children, &mut recovery_nodes);
                }
                break;
            }

            if let Some(item) = self.parse_grouped_reference_item_at(cursor) {
                cursor = item.next_position;
                children.push(item.id);
                recovery_nodes.extend(item.recovery_nodes);
            } else {
                self.diagnose_malformed_justification(
                    cursor,
                    "expected reference label in grouped citation",
                );
                self.push_missing_proof_step(cursor, &mut children, &mut recovery_nodes);
                if let Some(recovery) = self
                    .recover_malformed_justification_tail_with_definition_boundary(
                        cursor,
                        stop_at_definition_content_start,
                    )
                {
                    cursor = recovery.next_position;
                    children.push(recovery.id);
                    recovery_nodes.extend(recovery.recovery_nodes);
                }
            }

            if self.is_reserved_symbol_at(cursor, ",") {
                children.push(self.token_node_ids[cursor]);
                cursor += 1;
                expecting_item = true;
                continue;
            }
            break;
        }

        if self.is_reserved_symbol_at(cursor, "}") {
            children.push(self.token_node_ids[cursor]);
            cursor += 1;
        } else {
            self.diagnose_unmatched_justification_delimiter(plan.operator, cursor, "}");
            let recovery = self.add_recovery_node(
                SyntaxRecoveryKind::UnmatchedOpeningDelimiter,
                self.zero_range_at(cursor),
                Vec::new(),
            );
            children.push(recovery);
            recovery_nodes.push(recovery);
        }

        let id = self.events.emit(SyntaxEvent::Node {
            kind: SurfaceNodeKind::GroupedReference,
            range: self.covering_token_range(position, cursor),
            children,
        });
        Some(ParsedTypeNode {
            id,
            next_position: cursor.max(position + 1),
            recovery_nodes,
        })
    }

    fn parse_grouped_reference_item_at(&mut self, position: usize) -> Option<ParsedTypeNode> {
        if !self.is_identifier_at(position) {
            return None;
        }
        let id = self.events.emit(SyntaxEvent::Node {
            kind: SurfaceNodeKind::GroupedReferenceItem,
            range: self.request.tokens[position].span,
            children: vec![self.token_node_ids[position]],
        });
        Some(ParsedTypeNode {
            id,
            next_position: position + 1,
            recovery_nodes: Vec::new(),
        })
    }

    fn parse_bulk_reference_at(&mut self, position: usize) -> Option<ParsedTypeNode> {
        let plan = self.compound_reference_plan_at(position, ".*")?;
        let namespace = self
            .emit_namespace_path_from_parts(&plan.namespace_segments, &plan.namespace_separators);
        let children = vec![namespace, self.token_node_ids[plan.operator]];
        let id = self.events.emit(SyntaxEvent::Node {
            kind: SurfaceNodeKind::BulkReference,
            range: self.covering_token_range(position, plan.operator + 1),
            children,
        });
        Some(ParsedTypeNode {
            id,
            next_position: plan.operator + 1,
            recovery_nodes: Vec::new(),
        })
    }

    fn parse_computation_justification_with_definition_boundary_at(
        &mut self,
        position: usize,
        stop_at_definition_content_start: bool,
    ) -> ParsedTypeNode {
        let mut children = vec![self.token_node_ids[position]];
        let mut recovery_nodes = Vec::new();
        let mut cursor = position + 1;

        if self.is_reserved_symbol_at(cursor, "(") {
            let opener = cursor;
            children.push(self.token_node_ids[cursor]);
            cursor += 1;
            let mut expecting_option = true;

            loop {
                if self.is_reserved_symbol_at(cursor, ",") {
                    if expecting_option {
                        self.diagnose_malformed_justification(
                            cursor,
                            "expected computation option before `,`",
                        );
                        self.push_missing_proof_step(cursor, &mut children, &mut recovery_nodes);
                    }
                    children.push(self.token_node_ids[cursor]);
                    cursor += 1;
                    expecting_option = true;
                    continue;
                }

                if self.is_reserved_symbol_at(cursor, ")") {
                    if expecting_option {
                        self.diagnose_malformed_justification(
                            cursor,
                            "expected computation option before `)`",
                        );
                        self.push_missing_proof_step(cursor, &mut children, &mut recovery_nodes);
                    }
                    break;
                }
                if self
                    .is_justification_recovery_boundary_at(cursor, stop_at_definition_content_start)
                {
                    if expecting_option {
                        self.diagnose_malformed_justification(
                            cursor,
                            "expected computation option",
                        );
                        self.push_missing_proof_step(cursor, &mut children, &mut recovery_nodes);
                    }
                    break;
                }

                if let Some(option) = self.parse_computation_option_at(cursor) {
                    cursor = option.next_position;
                    children.push(option.id);
                    recovery_nodes.extend(option.recovery_nodes);
                } else {
                    self.diagnose_malformed_justification(cursor, "expected computation option");
                    self.push_missing_proof_step(cursor, &mut children, &mut recovery_nodes);
                    if let Some(recovery) = self
                        .recover_malformed_justification_tail_with_definition_boundary(
                            cursor,
                            stop_at_definition_content_start,
                        )
                    {
                        cursor = recovery.next_position;
                        children.push(recovery.id);
                        recovery_nodes.extend(recovery.recovery_nodes);
                    }
                }

                if self.is_reserved_symbol_at(cursor, ",") {
                    children.push(self.token_node_ids[cursor]);
                    cursor += 1;
                    expecting_option = true;
                    continue;
                }

                if !self.is_reserved_symbol_at(cursor, ")")
                    && !self.is_justification_recovery_boundary_at(
                        cursor,
                        stop_at_definition_content_start,
                    )
                {
                    self.diagnose_malformed_justification(
                        cursor,
                        "unexpected token in computation justification",
                    );
                    if let Some(recovery) = self
                        .recover_malformed_justification_tail_with_definition_boundary(
                            cursor,
                            stop_at_definition_content_start,
                        )
                    {
                        cursor = recovery.next_position;
                        children.push(recovery.id);
                        recovery_nodes.extend(recovery.recovery_nodes);
                    }
                }
                break;
            }

            if self.is_reserved_symbol_at(cursor, ")") {
                children.push(self.token_node_ids[cursor]);
                cursor += 1;
            } else {
                self.diagnose_unmatched_justification_delimiter(opener, cursor, ")");
                let recovery = self.add_recovery_node(
                    SyntaxRecoveryKind::UnmatchedOpeningDelimiter,
                    self.zero_range_at(cursor),
                    Vec::new(),
                );
                children.push(recovery);
                recovery_nodes.push(recovery);
            }
        }

        let id = self.events.emit(SyntaxEvent::Node {
            kind: SurfaceNodeKind::ComputationJustification,
            range: self.covering_token_range(position, cursor),
            children,
        });
        ParsedTypeNode {
            id,
            next_position: cursor.max(position + 1),
            recovery_nodes,
        }
    }

    fn parse_computation_option_at(&mut self, position: usize) -> Option<ParsedTypeNode> {
        if !self.is_computation_option_keyword_at(position) {
            return None;
        }
        let mut children = vec![self.token_node_ids[position]];
        let mut recovery_nodes = Vec::new();
        let mut cursor = position + 1;

        if self.is_reserved_symbol_at(cursor, ":") {
            children.push(self.token_node_ids[cursor]);
            cursor += 1;
        } else {
            self.diagnose_malformed_justification(
                cursor,
                "expected `:` after computation option name",
            );
        }

        if self.is_numeral_at(cursor) {
            children.push(self.token_node_ids[cursor]);
            cursor += 1;
        } else {
            self.diagnose_malformed_justification(
                cursor,
                "expected numeral after computation option `:`",
            );
            self.push_missing_proof_step(cursor, &mut children, &mut recovery_nodes);
        }

        let id = self.events.emit(SyntaxEvent::Node {
            kind: SurfaceNodeKind::ComputationOption,
            range: self.covering_token_range(position, cursor),
            children,
        });
        Some(ParsedTypeNode {
            id,
            next_position: cursor.max(position + 1),
            recovery_nodes,
        })
    }

    fn parse_proof_block_at(&mut self, position: usize) -> ParsedTypeNode {
        self.parse_proof_block_with_definition_boundary_at(position, false)
    }

    fn parse_definition_content_proof_block_at(&mut self, position: usize) -> ParsedTypeNode {
        self.parse_proof_block_with_definition_boundary_at(position, true)
    }

    fn parse_proof_block_with_definition_boundary_at(
        &mut self,
        position: usize,
        stop_at_definition_content_start: bool,
    ) -> ParsedTypeNode {
        let mut children = vec![self.token_node_ids[position]];
        let mut recovery_nodes = Vec::new();
        let mut cursor = position + 1;

        cursor = self.parse_reasoning_body_with_definition_boundary_at(
            cursor,
            &mut children,
            &mut recovery_nodes,
            stop_at_definition_content_start,
        );
        if self.is_end_keyword_at(cursor) {
            children.push(self.token_node_ids[cursor]);
            cursor += 1;
        } else {
            let missing = self.add_missing_end(position, cursor);
            children.push(missing);
            recovery_nodes.push(missing);
        }

        let id = self.events.emit(SyntaxEvent::Node {
            kind: SurfaceNodeKind::ProofBlock,
            range: self.covering_token_range(position, cursor),
            children,
        });
        ParsedTypeNode {
            id,
            next_position: cursor.max(position + 1),
            recovery_nodes,
        }
    }

    fn emit_namespace_path_from_parts(
        &mut self,
        segments: &[usize],
        separators: &[usize],
    ) -> SurfaceBuilderNodeId {
        let mut children = Vec::new();
        for (index, segment) in segments.iter().enumerate() {
            children.push(self.emit_wrapped_token(SurfaceNodeKind::PathSegment, *segment));
            if let Some(separator) = separators.get(index) {
                children.push(self.token_node_ids[*separator]);
            }
        }
        self.events.emit(SyntaxEvent::Node {
            kind: SurfaceNodeKind::NamespacePath,
            range: self.covering_token_range(segments[0], segments[segments.len() - 1] + 1),
            children,
        })
    }

    fn parse_qualified_variable_segments(
        &mut self,
        mut cursor: usize,
        children: &mut Vec<SurfaceBuilderNodeId>,
        recovery_nodes: &mut Vec<SurfaceBuilderNodeId>,
        first_message: &'static str,
        comma_message: &'static str,
    ) -> usize {
        if let Some(segment) = self.parse_qualified_variable_segment_at(cursor) {
            cursor = segment.next_position;
            children.push(segment.id);
            recovery_nodes.extend(segment.recovery_nodes);
        } else {
            self.diagnose_malformed_type_expression(cursor, first_message);
            return cursor;
        }

        while self.is_reserved_symbol_at(cursor, ",") {
            children.push(self.token_node_ids[cursor]);
            cursor += 1;
            if let Some(segment) = self.parse_qualified_variable_segment_at(cursor) {
                cursor = segment.next_position;
                children.push(segment.id);
                recovery_nodes.extend(segment.recovery_nodes);
            } else {
                self.diagnose_malformed_type_expression(cursor, comma_message);
                break;
            }
        }

        cursor
    }

    fn parse_qualified_variable_segment_at(&mut self, position: usize) -> Option<ParsedTypeNode> {
        if !self.is_identifier_at(position) {
            return None;
        }
        let mut children = vec![self.token_node_ids[position]];
        let mut recovery_nodes = Vec::new();
        let mut cursor = position + 1;

        while self.is_reserved_symbol_at(cursor, ",") && self.is_identifier_at(cursor + 1) {
            children.push(self.token_node_ids[cursor]);
            children.push(self.token_node_ids[cursor + 1]);
            cursor += 2;
        }

        if self.is_reserved_word_at(cursor, "be") || self.is_reserved_word_at(cursor, "being") {
            children.push(self.token_node_ids[cursor]);
            cursor += 1;
            if let Some(type_expression) = self.parse_type_expression_at(cursor) {
                cursor = type_expression.next_position;
                children.push(type_expression.id);
                recovery_nodes.extend(type_expression.recovery_nodes);
            } else {
                self.diagnose_malformed_type_expression(
                    cursor,
                    "expected type after qualified variable binder",
                );
                let missing = self.add_missing_type_expression(cursor);
                children.push(missing);
                recovery_nodes.push(missing);
            }
        }

        let id = self.events.emit(SyntaxEvent::Node {
            kind: SurfaceNodeKind::QualifiedVariableSegment,
            range: self.covering_token_range(position, cursor),
            children,
        });
        Some(ParsedTypeNode {
            id,
            next_position: cursor.max(position + 1),
            recovery_nodes,
        })
    }

    fn parse_optional_such_condition_list(
        &mut self,
        mut cursor: usize,
        children: &mut Vec<SurfaceBuilderNodeId>,
        recovery_nodes: &mut Vec<SurfaceBuilderNodeId>,
    ) -> usize {
        if !self.is_reserved_word_at(cursor, "such") {
            return cursor;
        }
        children.push(self.token_node_ids[cursor]);
        cursor += 1;

        if let Some(condition_list) = self.parse_condition_list_at(cursor) {
            cursor = condition_list.next_position;
            children.push(condition_list.id);
            recovery_nodes.extend(condition_list.recovery_nodes);
        } else {
            self.diagnose_malformed_formula_expression(cursor, "expected `that` after `such`");
            if let Some(recovery) = self.recover_malformed_statement_tail(cursor) {
                cursor = recovery.next_position;
                children.push(recovery.id);
                recovery_nodes.extend(recovery.recovery_nodes);
            }
        }

        cursor
    }

    fn parse_mandatory_such_condition_list(
        &mut self,
        mut cursor: usize,
        children: &mut Vec<SurfaceBuilderNodeId>,
        recovery_nodes: &mut Vec<SurfaceBuilderNodeId>,
        missing_such_message: &'static str,
        missing_that_message: &'static str,
    ) -> usize {
        if self.is_reserved_word_at(cursor, "such") {
            children.push(self.token_node_ids[cursor]);
            cursor += 1;

            if let Some(condition_list) = self.parse_condition_list_at(cursor) {
                cursor = condition_list.next_position;
                children.push(condition_list.id);
                recovery_nodes.extend(condition_list.recovery_nodes);
            } else {
                self.diagnose_malformed_formula_expression(cursor, missing_that_message);
                self.push_missing_formula(cursor, children, recovery_nodes);
            }
            return cursor;
        }

        self.diagnose_malformed_formula_expression(cursor, missing_such_message);
        self.push_missing_formula(cursor, children, recovery_nodes);
        cursor
    }

    fn parse_mandatory_simple_justification(
        &mut self,
        cursor: usize,
        children: &mut Vec<SurfaceBuilderNodeId>,
        recovery_nodes: &mut Vec<SurfaceBuilderNodeId>,
        missing_by_message: &'static str,
    ) -> usize {
        if self.is_reserved_word_at(cursor, "by") {
            let justification = self.parse_justification_clause_at(cursor, false);
            children.push(justification.id);
            recovery_nodes.extend(justification.recovery_nodes);
            justification.next_position
        } else {
            self.diagnose_malformed_justification(cursor, missing_by_message);
            self.push_missing_proof_step(cursor, children, recovery_nodes);
            cursor
        }
    }

    fn parse_condition_list_at(&mut self, position: usize) -> Option<ParsedTypeNode> {
        if !self.is_reserved_word_at(position, "that") {
            return None;
        }
        let mut children = vec![self.token_node_ids[position]];
        let mut recovery_nodes = Vec::new();
        let mut cursor = position + 1;

        let proposition = self.parse_proposition_at(cursor);
        cursor = proposition.next_position;
        children.push(proposition.id);
        recovery_nodes.extend(proposition.recovery_nodes);

        while self.is_reserved_word_at(cursor, "and") {
            children.push(self.token_node_ids[cursor]);
            cursor += 1;
            let proposition = self.parse_proposition_at(cursor);
            let made_progress = proposition.next_position > cursor;
            cursor = proposition.next_position;
            children.push(proposition.id);
            recovery_nodes.extend(proposition.recovery_nodes);
            if !made_progress {
                break;
            }
        }

        let id = self.events.emit(SyntaxEvent::Node {
            kind: SurfaceNodeKind::ConditionList,
            range: self.covering_token_range(position, cursor),
            children,
        });
        Some(ParsedTypeNode {
            id,
            next_position: cursor.max(position + 1),
            recovery_nodes,
        })
    }

    fn parse_proposition_at(&mut self, position: usize) -> ParsedTypeNode {
        let mut children = Vec::new();
        let mut recovery_nodes = Vec::new();
        let mut cursor = position;

        if self.is_identifier_at(cursor) && self.is_reserved_symbol_at(cursor + 1, ":") {
            children.push(self.token_node_ids[cursor]);
            children.push(self.token_node_ids[cursor + 1]);
            cursor += 2;
        }

        if let Some(formula) = self.parse_formula_expression_at(cursor) {
            cursor = formula.next_position;
            children.push(formula.id);
            recovery_nodes.extend(formula.recovery_nodes);
        } else {
            self.diagnose_malformed_formula_expression(cursor, "expected formula in proposition");
            let missing = self.add_missing_formula(cursor);
            children.push(missing);
            recovery_nodes.push(missing);
        }

        let range = if cursor > position {
            self.covering_token_range(position, cursor)
        } else {
            self.zero_range_at(position)
        };
        let id = self.events.emit(SyntaxEvent::Node {
            kind: SurfaceNodeKind::Proposition,
            range,
            children,
        });
        ParsedTypeNode {
            id,
            next_position: cursor,
            recovery_nodes,
        }
    }

    fn push_missing_formula(
        &mut self,
        position: usize,
        children: &mut Vec<SurfaceBuilderNodeId>,
        recovery_nodes: &mut Vec<SurfaceBuilderNodeId>,
    ) {
        let recovery = self.add_missing_formula(position);
        children.push(recovery);
        recovery_nodes.push(recovery);
    }

    fn parse_witness_at(&mut self, position: usize) -> ParsedTypeNode {
        let mut children = Vec::new();
        let mut recovery_nodes = Vec::new();
        let mut cursor = position;

        if self.is_identifier_at(cursor) && self.is_reserved_symbol_at(cursor + 1, "=") {
            children.push(self.token_node_ids[cursor]);
            children.push(self.token_node_ids[cursor + 1]);
            cursor += 2;
            if let Some(term) = self.parse_term_expression_at(cursor) {
                cursor = term.next_position;
                children.push(term.id);
                recovery_nodes.extend(term.recovery_nodes);
            } else {
                self.diagnose_malformed_term_expression(cursor, "expected witness term after `=`");
                self.push_missing_term(cursor, &mut children, &mut recovery_nodes);
            }
        } else if let Some(term) = self.parse_term_expression_at(cursor) {
            cursor = term.next_position;
            children.push(term.id);
            recovery_nodes.extend(term.recovery_nodes);
        } else {
            self.diagnose_malformed_term_expression(cursor, "expected witness in take statement");
            self.push_missing_term(cursor, &mut children, &mut recovery_nodes);
        }

        let range = if cursor > position {
            self.covering_token_range(position, cursor)
        } else {
            self.zero_range_at(position)
        };
        let id = self.events.emit(SyntaxEvent::Node {
            kind: SurfaceNodeKind::Witness,
            range,
            children,
        });
        ParsedTypeNode {
            id,
            next_position: cursor,
            recovery_nodes,
        }
    }

    fn parse_reconsider_item_at(&mut self, position: usize) -> ParsedTypeNode {
        let mut children = Vec::new();
        let mut recovery_nodes = Vec::new();
        let mut cursor = position;

        if self.is_identifier_at(cursor) {
            children.push(self.token_node_ids[cursor]);
            cursor += 1;
        } else {
            self.diagnose_malformed_term_expression(
                cursor,
                "expected identifier in reconsider statement",
            );
            self.push_missing_term(cursor, &mut children, &mut recovery_nodes);
            if !self.is_reconsider_item_boundary_at(cursor)
                && let Some(recovery) = self.recover_malformed_reconsider_item_tail(cursor)
            {
                cursor = recovery.next_position;
                children.push(recovery.id);
                recovery_nodes.extend(recovery.recovery_nodes);
            }
        }

        if self.is_reserved_symbol_at(cursor, "=") {
            children.push(self.token_node_ids[cursor]);
            cursor += 1;
            if let Some(term) = self.parse_term_expression_at(cursor) {
                cursor = term.next_position;
                children.push(term.id);
                recovery_nodes.extend(term.recovery_nodes);
            } else {
                self.diagnose_malformed_term_expression(
                    cursor,
                    "expected term after `=` in reconsider statement",
                );
                self.push_missing_term(cursor, &mut children, &mut recovery_nodes);
            }
        }

        let range = if cursor > position {
            self.covering_token_range(position, cursor)
        } else {
            self.zero_range_at(position)
        };
        let id = self.events.emit(SyntaxEvent::Node {
            kind: SurfaceNodeKind::ReconsiderItem,
            range,
            children,
        });
        ParsedTypeNode {
            id,
            next_position: cursor,
            recovery_nodes,
        }
    }

    fn parse_equating_at(&mut self, position: usize) -> ParsedTypeNode {
        let mut children = Vec::new();
        let mut recovery_nodes = Vec::new();
        let mut cursor = position;

        if self.is_identifier_at(cursor) {
            children.push(self.token_node_ids[cursor]);
            cursor += 1;
        } else {
            self.diagnose_malformed_term_expression(cursor, "expected identifier in set statement");
            self.push_missing_term(cursor, &mut children, &mut recovery_nodes);
        }

        if self.is_reserved_symbol_at(cursor, "=") {
            children.push(self.token_node_ids[cursor]);
            cursor += 1;
            if let Some(term) = self.parse_term_expression_at(cursor) {
                cursor = term.next_position;
                children.push(term.id);
                recovery_nodes.extend(term.recovery_nodes);
            } else {
                self.diagnose_malformed_term_expression(cursor, "expected term after `=`");
                self.push_missing_term(cursor, &mut children, &mut recovery_nodes);
            }
        } else {
            self.diagnose_malformed_term_expression(cursor, "expected `=` in set statement");
            self.push_missing_term(cursor, &mut children, &mut recovery_nodes);
        }

        let range = if cursor > position {
            self.covering_token_range(position, cursor)
        } else {
            self.zero_range_at(position)
        };
        let id = self.events.emit(SyntaxEvent::Node {
            kind: SurfaceNodeKind::Equating,
            range,
            children,
        });
        ParsedTypeNode {
            id,
            next_position: cursor,
            recovery_nodes,
        }
    }

    fn finish_simple_statement_node(
        &mut self,
        position: usize,
        mut cursor: usize,
        kind: SurfaceNodeKind,
        mut children: Vec<SurfaceBuilderNodeId>,
        mut recovery_nodes: Vec<SurfaceBuilderNodeId>,
        unexpected_message: &'static str,
    ) -> ParsedTypeNode {
        if self.is_semicolon_at(cursor) {
            children.push(self.token_node_ids[cursor]);
            cursor += 1;
        } else if cursor < self.request.tokens.len()
            && !self.is_statement_synchronization_boundary_at(cursor)
        {
            self.diagnose_malformed_term_expression(cursor, unexpected_message);
            if let Some(recovery) = self.recover_malformed_statement_tail(cursor) {
                cursor = recovery.next_position;
                children.push(recovery.id);
                recovery_nodes.extend(recovery.recovery_nodes);
            }
            if self.is_semicolon_at(cursor) {
                children.push(self.token_node_ids[cursor]);
                cursor += 1;
            } else {
                self.diagnose_missing_semicolon(cursor);
            }
        } else {
            self.diagnose_missing_semicolon(cursor);
        }

        let id = self.events.emit(SyntaxEvent::Node {
            kind,
            range: self.covering_token_range(position, cursor),
            children,
        });
        ParsedTypeNode {
            id,
            next_position: cursor.max(position + 1),
            recovery_nodes,
        }
    }

    fn parse_reasoning_body_at(
        &mut self,
        cursor: usize,
        children: &mut Vec<SurfaceBuilderNodeId>,
        recovery_nodes: &mut Vec<SurfaceBuilderNodeId>,
    ) -> usize {
        self.parse_reasoning_body_with_definition_boundary_at(
            cursor,
            children,
            recovery_nodes,
            false,
        )
    }

    fn parse_reasoning_body_with_definition_boundary_at(
        &mut self,
        mut cursor: usize,
        children: &mut Vec<SurfaceBuilderNodeId>,
        recovery_nodes: &mut Vec<SurfaceBuilderNodeId>,
        stop_at_definition_content_start: bool,
    ) -> usize {
        while cursor < self.request.tokens.len() {
            if self.is_end_keyword_at(cursor)
                || self.is_item_start_at(cursor)
                || self.is_case_branch_keyword_at(cursor)
                || (stop_at_definition_content_start && self.is_definition_content_start_at(cursor))
            {
                break;
            }

            if let Some(statement) = self.parse_statement_at(cursor) {
                cursor = statement.next_position;
                children.push(statement.id);
                recovery_nodes.extend(statement.recovery_nodes);
                continue;
            }

            if self.is_semicolon_at(cursor) {
                self.diagnose_malformed_formula_expression(
                    cursor,
                    "expected statement before `;` in reasoning block",
                );
                children.push(self.token_node_ids[cursor]);
                cursor += 1;
                continue;
            }

            self.diagnose_malformed_formula_expression(
                cursor,
                "expected statement in reasoning block",
            );
            if let Some(recovery) = self.recover_malformed_statement_tail(cursor) {
                cursor = recovery.next_position;
                children.push(recovery.id);
                recovery_nodes.extend(recovery.recovery_nodes);
            } else {
                break;
            }
        }
        cursor
    }

    fn parse_required_statement_semicolon(
        &mut self,
        mut cursor: usize,
        children: &mut Vec<SurfaceBuilderNodeId>,
    ) -> usize {
        if self.is_semicolon_at(cursor) {
            children.push(self.token_node_ids[cursor]);
            cursor += 1;
        } else {
            self.diagnose_missing_semicolon(cursor);
        }
        cursor
    }

    fn parse_required_end_semicolon(
        &mut self,
        opener: usize,
        mut cursor: usize,
        children: &mut Vec<SurfaceBuilderNodeId>,
        recovery_nodes: &mut Vec<SurfaceBuilderNodeId>,
    ) -> usize {
        if self.is_end_keyword_at(cursor) {
            children.push(self.token_node_ids[cursor]);
            cursor += 1;
        } else {
            let missing = self.add_missing_end(opener, cursor);
            children.push(missing);
            recovery_nodes.push(missing);
        }
        self.parse_required_statement_semicolon(cursor, children)
    }

    fn parse_deferred_statement_placeholder_item(&mut self, position: usize) -> ParsedItem {
        let end_exclusive = self.deferred_statement_placeholder_end(position);
        self.emit_placeholder_item(position, end_exclusive)
    }

    fn deferred_statement_placeholder_end(&mut self, position: usize) -> usize {
        let mut cursor = position + 1;
        let mut paren_depth = 0_usize;
        let mut bracket_depth = 0_usize;
        let mut brace_depth = 0_usize;
        let mut in_deferred_tail = false;
        while cursor < self.request.tokens.len() {
            let top_level = paren_depth == 0 && bracket_depth == 0 && brace_depth == 0;
            if top_level {
                if self.is_semicolon_at(cursor) {
                    return cursor + 1;
                }
                if self.is_item_start_at(cursor)
                    || self.is_end_keyword_at(cursor)
                    || (in_deferred_tail && self.is_statement_boundary_keyword_at(cursor))
                    || (!in_deferred_tail
                        && self.is_statement_boundary_keyword_at(cursor)
                        && !self.is_let_type_set_keyword_at(position, cursor))
                {
                    self.diagnose_missing_semicolon(cursor);
                    return cursor;
                }
                if self.is_reserved_word_at(cursor, "by") {
                    in_deferred_tail = true;
                }
            }

            if self.is_reserved_symbol_at(cursor, "(") {
                paren_depth += 1;
            } else if self.is_reserved_symbol_at(cursor, ")") {
                if paren_depth == 0 {
                    return cursor;
                }
                paren_depth -= 1;
            } else if self.is_reserved_symbol_at(cursor, "[") {
                bracket_depth += 1;
            } else if self.is_reserved_symbol_at(cursor, "]") {
                if bracket_depth == 0 {
                    return cursor;
                }
                bracket_depth -= 1;
            } else if self.is_reserved_symbol_at(cursor, "{") {
                brace_depth += 1;
            } else if self.is_reserved_symbol_at(cursor, "}") {
                if brace_depth == 0 {
                    return cursor;
                }
                brace_depth -= 1;
            }
            cursor += 1;
        }

        self.diagnose_missing_semicolon(self.request.tokens.len());
        self.request.tokens.len()
    }

    fn parse_export_item(&mut self, position: usize) -> ParsedItem {
        let mut children = vec![self.token_node_ids[position]];
        let mut recovery_nodes = Vec::new();
        let mut cursor = position + 1;

        if let Some(path) = self.parse_module_path_at(cursor) {
            children.push(path.id);
            cursor = path.next_position;

            while self.is_reserved_symbol_at(cursor, ",") {
                children.push(self.token_node_ids[cursor]);
                cursor += 1;
                match self.parse_module_path_at(cursor) {
                    Some(path) => {
                        children.push(path.id);
                        cursor = path.next_position;
                    }
                    None => {
                        self.diagnose_malformed_export(
                            cursor,
                            "expected module path after `,` in export statement",
                        );
                        if let Some(recovery) = self.recover_malformed_tail(cursor) {
                            children.push(recovery.id);
                            recovery_nodes.extend(recovery.recovery_nodes);
                            cursor = recovery.next_position;
                        }
                        break;
                    }
                }
            }
        } else {
            self.diagnose_malformed_export(
                cursor,
                "expected module path after `export` in export statement",
            );
            if let Some(recovery) = self.recover_malformed_tail(cursor) {
                children.push(recovery.id);
                recovery_nodes.extend(recovery.recovery_nodes);
                cursor = recovery.next_position;
            }
        }

        if self.is_semicolon_at(cursor) {
            children.push(self.token_node_ids[cursor]);
            cursor += 1;
        } else if cursor < self.request.tokens.len() && !self.is_item_start_at(cursor) {
            self.diagnose_malformed_export(cursor, "unexpected token in export statement");
            if let Some(recovery) = self.recover_malformed_tail(cursor) {
                children.push(recovery.id);
                recovery_nodes.extend(recovery.recovery_nodes);
                cursor = recovery.next_position;
            }
            if self.is_semicolon_at(cursor) {
                children.push(self.token_node_ids[cursor]);
                cursor += 1;
            }
        } else {
            self.diagnose_missing_semicolon(cursor);
        }

        let id = self.events.emit(SyntaxEvent::Node {
            kind: SurfaceNodeKind::ExportItem,
            range: self.covering_token_range(position, cursor),
            children,
        });
        ParsedItem {
            id,
            next_position: cursor.max(position + 1),
            recovery_nodes,
        }
    }

    fn parse_visible_item(&mut self, position: usize) -> ParsedItem {
        let marker_position = self
            .item_head_position(position)
            .expect("visible item parsing starts at an item boundary");
        let mut children = self.token_node_ids[position..marker_position].to_vec();
        let mut recovery_nodes = Vec::new();
        let marker = self.events.emit(SyntaxEvent::Node {
            kind: SurfaceNodeKind::VisibilityMarker,
            range: self.request.tokens[marker_position].span,
            children: vec![self.token_node_ids[marker_position]],
        });
        children.push(marker);

        let target_position = marker_position + 1;
        let cursor = if self.is_visibility_target_start_at(target_position) {
            let target = if self.is_notation_alias_start_at(target_position) {
                let alias = self.parse_notation_alias_at(target_position);
                ParsedItem {
                    id: alias.id,
                    next_position: alias.next_position,
                    recovery_nodes: alias.recovery_nodes,
                }
            } else {
                self.parse_theorem_item(target_position)
                    .unwrap_or_else(|| self.parse_placeholder_item(target_position))
            };
            children.push(target.id);
            recovery_nodes.extend(target.recovery_nodes);
            target.next_position
        } else {
            self.diagnose_malformed_visibility(target_position);
            let mut cursor = target_position;
            if let Some(recovery) = self.recover_malformed_visibility_tail(target_position) {
                children.push(recovery.id);
                recovery_nodes.extend(recovery.recovery_nodes);
                cursor = recovery.next_position;
            }
            if self.is_semicolon_at(cursor) {
                children.push(self.token_node_ids[cursor]);
                cursor += 1;
            }
            cursor
        };

        let id = self.events.emit(SyntaxEvent::Node {
            kind: SurfaceNodeKind::VisibleItem,
            range: self.covering_token_range(position, cursor),
            children,
        });
        ParsedItem {
            id,
            next_position: cursor.max(position + 1),
            recovery_nodes,
        }
    }

    fn parse_theorem_item(&mut self, position: usize) -> Option<ParsedItem> {
        let head = self.item_head_position(position)?;
        let role = self.theorem_role_position_at(head)?;
        if !self.looks_like_theorem_item_after_role(role) {
            return None;
        }
        if let Some(formula_start) = self.theorem_formula_start_after_role(role)
            && self.formula_payload_contains_deferred_predicate_template_args_at(formula_start)
        {
            return None;
        }
        let surface_kind = if self.is_reserved_word_at(role, "lemma") {
            SurfaceNodeKind::LemmaItem
        } else {
            SurfaceNodeKind::TheoremItem
        };
        let mut children = self.token_node_ids[position..=role].to_vec();
        let mut recovery_nodes = Vec::new();
        let mut cursor = role + 1;

        if self.is_identifier_at(cursor) {
            children.push(self.token_node_ids[cursor]);
            cursor += 1;
        } else {
            self.diagnose_malformed_term_expression(cursor, "expected theorem label");
            self.push_missing_term(cursor, &mut children, &mut recovery_nodes);
        }

        if self.is_reserved_symbol_at(cursor, ":") {
            children.push(self.token_node_ids[cursor]);
            cursor += 1;
        } else {
            self.diagnose_malformed_formula_expression(cursor, "expected `:` after theorem label");
        }

        if let Some(formula) = self.parse_formula_expression_at(cursor) {
            cursor = formula.next_position;
            children.push(formula.id);
            recovery_nodes.extend(formula.recovery_nodes);
        } else {
            self.diagnose_malformed_formula_expression(cursor, "expected theorem formula");
            self.push_missing_formula(cursor, &mut children, &mut recovery_nodes);
            if !self.is_theorem_item_tail_boundary_at(cursor)
                && let Some(recovery) = self.recover_malformed_theorem_tail(cursor)
            {
                cursor = recovery.next_position;
                children.push(recovery.id);
                recovery_nodes.extend(recovery.recovery_nodes);
            }
        }

        if let Some(justification) = self.parse_general_justification_at(cursor, true) {
            cursor = justification.next_position;
            children.push(justification.id);
            recovery_nodes.extend(justification.recovery_nodes);
        }

        if self.is_semicolon_at(cursor) {
            children.push(self.token_node_ids[cursor]);
            cursor += 1;
        } else if cursor < self.request.tokens.len()
            && !self.is_statement_synchronization_boundary_at(cursor)
        {
            self.diagnose_malformed_term_expression(cursor, "unexpected token in theorem item");
            if let Some(recovery) = self.recover_malformed_theorem_tail(cursor) {
                cursor = recovery.next_position;
                children.push(recovery.id);
                recovery_nodes.extend(recovery.recovery_nodes);
            }
            if self.is_semicolon_at(cursor) {
                children.push(self.token_node_ids[cursor]);
                cursor += 1;
            } else {
                self.diagnose_missing_semicolon(cursor);
            }
        } else {
            self.diagnose_missing_semicolon(cursor);
        }

        let id = self.events.emit(SyntaxEvent::Node {
            kind: surface_kind,
            range: self.covering_token_range(position, cursor),
            children,
        });
        Some(ParsedItem {
            id,
            next_position: cursor.max(position + 1),
            recovery_nodes,
        })
    }

    fn parse_definition_block_item(&mut self, position: usize) -> Option<ParsedItem> {
        let head = self.item_head_position(position)?;
        if !self.is_reserved_word_at(head, "definition") {
            return None;
        }

        let mut children = self.token_node_ids[position..=head].to_vec();
        let mut recovery_nodes = Vec::new();
        let mut cursor = head + 1;
        let mut template_ambiguous_content_seen = false;

        while cursor < self.request.tokens.len() && !self.is_end_keyword_at(cursor) {
            if self.definition_parameter_starts_template_ambiguous_at(cursor) {
                template_ambiguous_content_seen = true;
            }
            let content = if template_ambiguous_content_seen {
                self.parse_definition_content_placeholder(cursor)
            } else {
                self.parse_definition_content_at(cursor)
                    .unwrap_or_else(|| self.parse_definition_content_placeholder(cursor))
            };
            let made_progress = content.next_position > cursor;
            cursor = content.next_position;
            children.push(content.id);
            recovery_nodes.extend(content.recovery_nodes);
            if !made_progress {
                break;
            }
        }

        if self.is_end_keyword_at(cursor) {
            children.push(self.token_node_ids[cursor]);
            cursor += 1;
        } else {
            let missing = self.add_missing_end(head, cursor);
            children.push(missing);
            recovery_nodes.push(missing);
        }
        cursor = self.parse_required_statement_semicolon(cursor, &mut children);

        let id = self.events.emit(SyntaxEvent::Node {
            kind: SurfaceNodeKind::DefinitionBlockItem,
            range: self.covering_token_range(position, cursor),
            children,
        });
        Some(ParsedItem {
            id,
            next_position: cursor.max(position + 1),
            recovery_nodes,
        })
    }

    fn parse_definition_content_at(&mut self, position: usize) -> Option<ParsedTypeNode> {
        if self.definition_parameter_starts_template_ambiguous_at(position) {
            return None;
        }
        if self.is_reserved_word_at(position, "let") {
            return Some(self.parse_definition_parameter_at(position));
        }
        if self.is_reserved_word_at(position, "assume") {
            return Some(self.parse_assumption_statement_at(position));
        }
        if self.is_correctness_condition_keyword_at(position) {
            return Some(self.parse_correctness_condition_at(position));
        }
        if self.is_reserved_word_at(position, "attr") {
            return Some(self.parse_attribute_definition_at(position));
        }
        if self.is_reserved_word_at(position, "pred") {
            return Some(self.parse_predicate_definition_at(position));
        }
        if self.is_reserved_word_at(position, "func") {
            return Some(self.parse_functor_definition_at(position));
        }
        if self.is_reserved_word_at(position, "mode") {
            return Some(self.parse_mode_definition_at(position));
        }
        if self.is_reserved_word_at(position, "struct") {
            return Some(self.parse_structure_definition_at(position));
        }
        if self.is_reserved_word_at(position, "inherit") {
            return Some(self.parse_inheritance_definition_at(position));
        }
        if self.is_supported_redefinition_start_at(position) {
            return Some(self.parse_redefinition_at(position));
        }
        if self.is_notation_alias_start_at(position) {
            return Some(self.parse_notation_alias_at(position));
        }
        if self.is_property_clause_keyword_at(position) {
            return Some(self.parse_property_clause_at(position));
        }
        if self.is_visibility_marker_at(position) && self.is_reserved_word_at(position + 1, "pred")
        {
            return Some(self.parse_visible_predicate_definition_at(position));
        }
        if self.is_visibility_marker_at(position) && self.is_reserved_word_at(position + 1, "func")
        {
            return Some(self.parse_visible_functor_definition_at(position));
        }
        if self.is_visibility_marker_at(position) && self.is_reserved_word_at(position + 1, "mode")
        {
            return Some(self.parse_visible_mode_definition_at(position));
        }
        if self.is_visibility_marker_at(position)
            && self.is_reserved_word_at(position + 1, "struct")
        {
            return Some(self.parse_visible_structure_definition_at(position));
        }
        if self.is_visibility_marker_at(position)
            && self.is_reserved_word_at(position + 1, "inherit")
        {
            return Some(self.parse_visible_inheritance_definition_at(position));
        }
        if self.is_visibility_marker_at(position)
            && self.is_supported_redefinition_start_at(position + 1)
        {
            return Some(self.parse_visible_redefinition_at(position));
        }
        if self.is_visibility_marker_at(position) && self.is_notation_alias_start_at(position + 1) {
            return Some(self.parse_visible_notation_alias_at(position));
        }
        if let Some(item) = self.parse_theorem_item(position) {
            return Some(ParsedTypeNode {
                id: item.id,
                next_position: item.next_position,
                recovery_nodes: item.recovery_nodes,
            });
        }
        if self.is_visibility_marker_at(position)
            && self.is_visible_theorem_target_start_at(position + 1)
        {
            let item = self.parse_visible_item(position);
            return Some(ParsedTypeNode {
                id: item.id,
                next_position: item.next_position,
                recovery_nodes: item.recovery_nodes,
            });
        }
        None
    }

    fn parse_definition_parameter_at(&mut self, position: usize) -> ParsedTypeNode {
        let mut children = vec![self.token_node_ids[position]];
        let mut recovery_nodes = Vec::new();
        let mut cursor = position + 1;

        cursor = self.parse_qualified_variable_segments(
            cursor,
            &mut children,
            &mut recovery_nodes,
            "expected variable segment after definition `let`",
            "expected variable segment after `,` in definition parameter",
        );

        if self.is_reserved_word_at(cursor, "such") {
            children.push(self.token_node_ids[cursor]);
            cursor += 1;
            if self.is_reserved_word_at(cursor, "that") {
                if self
                    .definition_parameter_tail_justification_position_at(cursor + 1)
                    .is_some()
                {
                    children.push(self.token_node_ids[cursor]);
                    cursor += 1;
                    if let Some(formula) = self.parse_formula_expression_at(cursor) {
                        cursor = formula.next_position;
                        children.push(formula.id);
                        recovery_nodes.extend(formula.recovery_nodes);
                    } else {
                        self.diagnose_malformed_formula_expression(
                            cursor,
                            "expected formula after `such that`",
                        );
                        self.push_missing_formula(cursor, &mut children, &mut recovery_nodes);
                    }
                    if self.is_reserved_word_at(cursor, "by") {
                        let justification = self.parse_justification_clause_at(cursor, false);
                        cursor = justification.next_position;
                        children.push(justification.id);
                        recovery_nodes.extend(justification.recovery_nodes);
                    } else if self.is_reserved_word_at(cursor, "proof") {
                        let proof = self.parse_proof_block_at(cursor);
                        cursor = proof.next_position;
                        children.push(proof.id);
                        recovery_nodes.extend(proof.recovery_nodes);
                    }
                } else if let Some(condition_list) = self.parse_condition_list_at(cursor) {
                    cursor = condition_list.next_position;
                    children.push(condition_list.id);
                    recovery_nodes.extend(condition_list.recovery_nodes);
                } else {
                    self.diagnose_malformed_formula_expression(
                        cursor,
                        "expected definition parameter conditions after `such`",
                    );
                    self.push_missing_formula(cursor, &mut children, &mut recovery_nodes);
                }
            } else {
                self.diagnose_malformed_formula_expression(
                    cursor,
                    "expected `that` after `such` in definition parameter",
                );
                self.push_missing_formula(cursor, &mut children, &mut recovery_nodes);
            }
        } else if self.is_reserved_word_at(cursor, "by") {
            let justification = self.parse_justification_clause_at(cursor, false);
            cursor = justification.next_position;
            children.push(justification.id);
            recovery_nodes.extend(justification.recovery_nodes);
        }

        self.finish_definition_content_node(
            position,
            cursor,
            SurfaceNodeKind::DefinitionParameter,
            children,
            recovery_nodes,
            "unexpected token in definition parameter",
        )
    }

    fn parse_correctness_condition_at(&mut self, position: usize) -> ParsedTypeNode {
        let mut children = vec![self.token_node_ids[position]];
        let mut recovery_nodes = Vec::new();
        let mut cursor = position + 1;

        if let Some(justification) =
            self.parse_definition_content_general_justification_at(cursor, true)
        {
            cursor = justification.next_position;
            children.push(justification.id);
            recovery_nodes.extend(justification.recovery_nodes);
        } else if !self.is_semicolon_at(cursor)
            && !self.is_definition_content_synchronization_boundary_at(cursor)
        {
            self.diagnose_malformed_justification(
                cursor,
                "expected correctness-condition justification",
            );
            if let Some(recovery) = self.recover_malformed_definition_content_tail(cursor) {
                cursor = recovery.next_position;
                children.push(recovery.id);
                recovery_nodes.extend(recovery.recovery_nodes);
            }
        }

        self.finish_definition_content_node(
            position,
            cursor,
            SurfaceNodeKind::CorrectnessCondition,
            children,
            recovery_nodes,
            "unexpected token in correctness condition",
        )
    }

    fn parse_attribute_definition_at(&mut self, position: usize) -> ParsedTypeNode {
        let mut children = vec![self.token_node_ids[position]];
        let mut recovery_nodes = Vec::new();
        let mut cursor = position + 1;

        if self.is_identifier_at(cursor) {
            children.push(self.token_node_ids[cursor]);
            cursor += 1;
        } else {
            self.diagnose_malformed_term_expression(cursor, "expected attribute definition label");
            self.push_missing_term(cursor, &mut children, &mut recovery_nodes);
        }

        if self.is_reserved_symbol_at(cursor, ":") {
            children.push(self.token_node_ids[cursor]);
            cursor += 1;
        } else {
            self.diagnose_malformed_formula_expression(
                cursor,
                "expected `:` after attribute definition label",
            );
        }

        if self.is_identifier_at(cursor) {
            children.push(self.token_node_ids[cursor]);
            cursor += 1;
        } else {
            self.diagnose_malformed_term_expression(cursor, "expected attribute subject");
            self.push_missing_term(cursor, &mut children, &mut recovery_nodes);
        }

        if self.is_reserved_word_at(cursor, "is") {
            children.push(self.token_node_ids[cursor]);
            cursor += 1;
        } else {
            self.diagnose_malformed_formula_expression(
                cursor,
                "expected `is` before attribute pattern",
            );
        }

        let pattern = self.parse_attribute_pattern_at(cursor);
        cursor = pattern.next_position;
        children.push(pattern.id);
        recovery_nodes.extend(pattern.recovery_nodes);

        if self.is_reserved_word_at(cursor, "means") {
            children.push(self.token_node_ids[cursor]);
            cursor += 1;
        } else {
            self.diagnose_malformed_formula_expression(
                cursor,
                "expected `means` in attribute definition",
            );
        }

        let definiens = self.parse_formula_definiens_at(cursor);
        cursor = definiens.next_position;
        children.push(definiens.id);
        recovery_nodes.extend(definiens.recovery_nodes);

        self.finish_definition_content_node(
            position,
            cursor,
            SurfaceNodeKind::AttributeDefinition,
            children,
            recovery_nodes,
            "unexpected token in attribute definition",
        )
    }

    fn parse_attribute_pattern_at(&mut self, position: usize) -> ParsedTypeNode {
        let mut children = Vec::new();
        let mut recovery_nodes = Vec::new();
        let mut cursor = position;

        if let Some(prefix_end) = self.parameter_prefix_next_at(cursor) {
            let prefix = self.events.emit(SyntaxEvent::Node {
                kind: SurfaceNodeKind::ParameterPrefix,
                range: self.covering_token_range(cursor, prefix_end),
                children: self.token_node_ids[cursor..prefix_end].to_vec(),
            });
            children.push(prefix);
            cursor = prefix_end;
        }

        if self.is_identifier_at(cursor) || self.is_user_symbol_at(cursor) {
            children.push(self.token_node_ids[cursor]);
            cursor += 1;
        } else {
            self.diagnose_malformed_term_expression(cursor, "expected attribute pattern name");
            self.push_missing_term(cursor, &mut children, &mut recovery_nodes);
        }

        let range = if cursor > position {
            self.covering_token_range(position, cursor)
        } else {
            self.zero_range_at(position)
        };
        let id = self.events.emit(SyntaxEvent::Node {
            kind: SurfaceNodeKind::AttributePattern,
            range,
            children,
        });
        ParsedTypeNode {
            id,
            next_position: cursor.max(position),
            recovery_nodes,
        }
    }

    fn parse_visible_predicate_definition_at(&mut self, position: usize) -> ParsedTypeNode {
        let marker = self.events.emit(SyntaxEvent::Node {
            kind: SurfaceNodeKind::VisibilityMarker,
            range: self.request.tokens[position].span,
            children: vec![self.token_node_ids[position]],
        });
        let predicate = self.parse_predicate_definition_at(position + 1);
        let cursor = predicate.next_position;
        let children = vec![marker, predicate.id];
        let recovery_nodes = predicate.recovery_nodes;

        let id = self.events.emit(SyntaxEvent::Node {
            kind: SurfaceNodeKind::VisibleItem,
            range: self.covering_token_range(position, cursor),
            children,
        });
        ParsedTypeNode {
            id,
            next_position: cursor.max(position + 1),
            recovery_nodes,
        }
    }

    fn parse_visible_functor_definition_at(&mut self, position: usize) -> ParsedTypeNode {
        let marker = self.events.emit(SyntaxEvent::Node {
            kind: SurfaceNodeKind::VisibilityMarker,
            range: self.request.tokens[position].span,
            children: vec![self.token_node_ids[position]],
        });
        let functor = self.parse_functor_definition_at(position + 1);
        let cursor = functor.next_position;
        let children = vec![marker, functor.id];
        let recovery_nodes = functor.recovery_nodes;

        let id = self.events.emit(SyntaxEvent::Node {
            kind: SurfaceNodeKind::VisibleItem,
            range: self.covering_token_range(position, cursor),
            children,
        });
        ParsedTypeNode {
            id,
            next_position: cursor.max(position + 1),
            recovery_nodes,
        }
    }

    fn parse_visible_mode_definition_at(&mut self, position: usize) -> ParsedTypeNode {
        let marker = self.events.emit(SyntaxEvent::Node {
            kind: SurfaceNodeKind::VisibilityMarker,
            range: self.request.tokens[position].span,
            children: vec![self.token_node_ids[position]],
        });
        let mode = self.parse_mode_definition_at(position + 1);
        let cursor = mode.next_position;
        let children = vec![marker, mode.id];
        let recovery_nodes = mode.recovery_nodes;

        let id = self.events.emit(SyntaxEvent::Node {
            kind: SurfaceNodeKind::VisibleItem,
            range: self.covering_token_range(position, cursor),
            children,
        });
        ParsedTypeNode {
            id,
            next_position: cursor.max(position + 1),
            recovery_nodes,
        }
    }

    fn parse_visible_structure_definition_at(&mut self, position: usize) -> ParsedTypeNode {
        let structure = self.parse_structure_definition_at(position + 1);
        self.wrap_visible_definition_content(position, structure)
    }

    fn parse_visible_inheritance_definition_at(&mut self, position: usize) -> ParsedTypeNode {
        let inheritance = self.parse_inheritance_definition_at(position + 1);
        self.wrap_visible_definition_content(position, inheritance)
    }

    fn parse_visible_redefinition_at(&mut self, position: usize) -> ParsedTypeNode {
        let redefinition = self.parse_redefinition_at(position + 1);
        self.wrap_visible_definition_content(position, redefinition)
    }

    fn parse_visible_notation_alias_at(&mut self, position: usize) -> ParsedTypeNode {
        let alias = self.parse_notation_alias_at(position + 1);
        self.wrap_visible_definition_content(position, alias)
    }

    fn wrap_visible_definition_content(
        &mut self,
        position: usize,
        content: ParsedTypeNode,
    ) -> ParsedTypeNode {
        let marker = self.events.emit(SyntaxEvent::Node {
            kind: SurfaceNodeKind::VisibilityMarker,
            range: self.request.tokens[position].span,
            children: vec![self.token_node_ids[position]],
        });
        let cursor = content.next_position;
        let children = vec![marker, content.id];
        let recovery_nodes = content.recovery_nodes;

        let id = self.events.emit(SyntaxEvent::Node {
            kind: SurfaceNodeKind::VisibleItem,
            range: self.covering_token_range(position, cursor),
            children,
        });
        ParsedTypeNode {
            id,
            next_position: cursor.max(position + 1),
            recovery_nodes,
        }
    }

    fn parse_predicate_definition_at(&mut self, position: usize) -> ParsedTypeNode {
        let mut children = vec![self.token_node_ids[position]];
        let mut recovery_nodes = Vec::new();
        let mut cursor = position + 1;

        if self.is_identifier_at(cursor) {
            children.push(self.token_node_ids[cursor]);
            cursor += 1;
        } else {
            self.diagnose_malformed_term_expression(cursor, "expected predicate definition label");
            self.push_missing_term(cursor, &mut children, &mut recovery_nodes);
        }

        if self.is_reserved_symbol_at(cursor, ":") {
            children.push(self.token_node_ids[cursor]);
            cursor += 1;
        } else {
            self.diagnose_malformed_formula_expression(
                cursor,
                "expected `:` after predicate definition label",
            );
        }

        let pattern = self.parse_predicate_pattern_at(cursor);
        cursor = pattern.next_position;
        children.push(pattern.id);
        recovery_nodes.extend(pattern.recovery_nodes);

        if self.is_reserved_word_at(cursor, "means") {
            children.push(self.token_node_ids[cursor]);
            cursor += 1;
        } else {
            self.diagnose_malformed_formula_expression(
                cursor,
                "expected `means` in predicate definition",
            );
        }

        let definiens = self.parse_formula_definiens_at(cursor);
        cursor = definiens.next_position;
        children.push(definiens.id);
        recovery_nodes.extend(definiens.recovery_nodes);

        self.finish_definition_content_node(
            position,
            cursor,
            SurfaceNodeKind::PredicateDefinition,
            children,
            recovery_nodes,
            "unexpected token in predicate definition",
        )
    }

    fn parse_predicate_pattern_at(&mut self, position: usize) -> ParsedTypeNode {
        let cursor = self.predicate_pattern_boundary_at(position);
        let mut children = self.token_node_ids[position..cursor].to_vec();
        let mut recovery_nodes = Vec::new();

        if !self.predicate_pattern_can_match(position, cursor) {
            self.diagnose_malformed_term_expression(cursor, "expected predicate pattern");
            self.push_missing_term(cursor, &mut children, &mut recovery_nodes);
        }

        let range = if cursor > position {
            self.covering_token_range(position, cursor)
        } else {
            self.zero_range_at(position)
        };
        let id = self.events.emit(SyntaxEvent::Node {
            kind: SurfaceNodeKind::PredicatePattern,
            range,
            children,
        });
        ParsedTypeNode {
            id,
            next_position: cursor.max(position),
            recovery_nodes,
        }
    }

    fn parse_functor_definition_at(&mut self, position: usize) -> ParsedTypeNode {
        let mut children = vec![self.token_node_ids[position]];
        let mut recovery_nodes = Vec::new();
        let mut cursor = position + 1;

        if self.is_identifier_at(cursor) {
            children.push(self.token_node_ids[cursor]);
            cursor += 1;
        } else {
            self.diagnose_malformed_term_expression(cursor, "expected functor definition label");
            self.push_missing_term(cursor, &mut children, &mut recovery_nodes);
        }

        if self.is_reserved_symbol_at(cursor, ":") {
            children.push(self.token_node_ids[cursor]);
            cursor += 1;
        } else {
            self.diagnose_malformed_formula_expression(
                cursor,
                "expected `:` after functor definition label",
            );
        }

        let pattern = self.parse_functor_pattern_at(cursor);
        cursor = pattern.next_position;
        children.push(pattern.id);
        recovery_nodes.extend(pattern.recovery_nodes);

        if self.is_reserved_symbol_at(cursor, "->") {
            children.push(self.token_node_ids[cursor]);
            cursor += 1;
        } else {
            self.diagnose_malformed_formula_expression(
                cursor,
                "expected `->` before functor return type",
            );
        }

        if let Some(return_type) = self.parse_type_expression_at(cursor) {
            cursor = return_type.next_position;
            children.push(return_type.id);
            recovery_nodes.extend(return_type.recovery_nodes);
        } else {
            self.diagnose_malformed_type_expression(cursor, "expected functor return type");
            let missing = self.add_missing_type_expression(cursor);
            children.push(missing);
            recovery_nodes.push(missing);
        }

        if self.is_reserved_word_at(cursor, "means") {
            children.push(self.token_node_ids[cursor]);
            cursor += 1;
            let definiens = self.parse_formula_definiens_at(cursor);
            cursor = definiens.next_position;
            children.push(definiens.id);
            recovery_nodes.extend(definiens.recovery_nodes);
        } else if self.is_reserved_word_at(cursor, "equals") {
            children.push(self.token_node_ids[cursor]);
            cursor += 1;
            let definiens = self.parse_term_definiens_at(cursor);
            cursor = definiens.next_position;
            children.push(definiens.id);
            recovery_nodes.extend(definiens.recovery_nodes);
        } else {
            self.diagnose_malformed_formula_expression(
                cursor,
                "expected `means` or `equals` in functor definition",
            );
            if self.can_start_formula_at(cursor) {
                let definiens = self.parse_formula_definiens_at(cursor);
                cursor = definiens.next_position;
                children.push(definiens.id);
                recovery_nodes.extend(definiens.recovery_nodes);
            } else if self.can_start_term_operator_operand_at(cursor, false) {
                let definiens = self.parse_term_definiens_at(cursor);
                cursor = definiens.next_position;
                children.push(definiens.id);
                recovery_nodes.extend(definiens.recovery_nodes);
            } else {
                let definiens = self.parse_formula_definiens_at(cursor);
                cursor = definiens.next_position;
                children.push(definiens.id);
                recovery_nodes.extend(definiens.recovery_nodes);
            }
        }

        self.finish_definition_content_node(
            position,
            cursor,
            SurfaceNodeKind::FunctorDefinition,
            children,
            recovery_nodes,
            "unexpected token in functor definition",
        )
    }

    fn parse_functor_pattern_at(&mut self, position: usize) -> ParsedTypeNode {
        let cursor = self.functor_pattern_boundary_at(position);
        let mut children = self.token_node_ids[position..cursor].to_vec();
        let mut recovery_nodes = Vec::new();

        if !self.functor_pattern_can_match(position, cursor) {
            self.diagnose_malformed_term_expression(cursor, "expected functor pattern");
            self.push_missing_term(cursor, &mut children, &mut recovery_nodes);
        }

        let range = if cursor > position {
            self.covering_token_range(position, cursor)
        } else {
            self.zero_range_at(position)
        };
        let id = self.events.emit(SyntaxEvent::Node {
            kind: SurfaceNodeKind::FunctorPattern,
            range,
            children,
        });
        ParsedTypeNode {
            id,
            next_position: cursor.max(position),
            recovery_nodes,
        }
    }

    fn parse_mode_definition_at(&mut self, position: usize) -> ParsedTypeNode {
        let mut children = vec![self.token_node_ids[position]];
        let mut recovery_nodes = Vec::new();
        let mut cursor = position + 1;

        if self.is_identifier_at(cursor) {
            children.push(self.token_node_ids[cursor]);
            cursor += 1;
        } else {
            self.diagnose_malformed_term_expression(cursor, "expected mode definition label");
            self.push_missing_term(cursor, &mut children, &mut recovery_nodes);
        }

        if self.is_reserved_symbol_at(cursor, ":") {
            children.push(self.token_node_ids[cursor]);
            cursor += 1;
        } else {
            self.diagnose_malformed_formula_expression(
                cursor,
                "expected `:` after mode definition label",
            );
        }

        let pattern = self.parse_mode_pattern_at(cursor);
        cursor = pattern.next_position;
        children.push(pattern.id);
        recovery_nodes.extend(pattern.recovery_nodes);

        if self.is_reserved_word_at(cursor, "is") {
            children.push(self.token_node_ids[cursor]);
            cursor += 1;
        } else {
            self.diagnose_malformed_formula_expression(cursor, "expected `is` in mode definition");
        }

        if let Some(body) = self.parse_type_expression_at(cursor) {
            cursor = body.next_position;
            children.push(body.id);
            recovery_nodes.extend(body.recovery_nodes);
        } else {
            self.diagnose_malformed_type_expression(cursor, "expected mode body type");
            let missing = self.add_missing_type_expression(cursor);
            children.push(missing);
            recovery_nodes.push(missing);
        }

        cursor = self.parse_mode_definition_semicolon(cursor, &mut children, &mut recovery_nodes);

        if self.is_reserved_word_at(cursor, "sethood") {
            let property = self.parse_mode_property_at(cursor);
            cursor = property.next_position;
            children.push(property.id);
            recovery_nodes.extend(property.recovery_nodes);
        }

        if cursor < self.request.tokens.len()
            && !self.is_definition_content_synchronization_boundary_at(cursor)
        {
            self.diagnose_malformed_term_expression(cursor, "unexpected token in mode definition");
            if let Some(recovery) = self.recover_malformed_definition_content_tail(cursor) {
                cursor = recovery.next_position;
                children.push(recovery.id);
                recovery_nodes.extend(recovery.recovery_nodes);
            }
            if self.is_semicolon_at(cursor) {
                children.push(self.token_node_ids[cursor]);
                cursor += 1;
            }
        }

        let id = self.events.emit(SyntaxEvent::Node {
            kind: SurfaceNodeKind::ModeDefinition,
            range: self.covering_token_range(position, cursor),
            children,
        });
        ParsedTypeNode {
            id,
            next_position: cursor.max(position + 1),
            recovery_nodes,
        }
    }

    fn parse_mode_definition_semicolon(
        &mut self,
        mut cursor: usize,
        children: &mut Vec<SurfaceBuilderNodeId>,
        recovery_nodes: &mut Vec<SurfaceBuilderNodeId>,
    ) -> usize {
        if self.is_semicolon_at(cursor) {
            children.push(self.token_node_ids[cursor]);
            return cursor + 1;
        }

        self.diagnose_missing_semicolon(cursor);
        if cursor < self.request.tokens.len()
            && !self.is_reserved_word_at(cursor, "sethood")
            && !self.is_definition_content_synchronization_boundary_at(cursor)
        {
            self.diagnose_malformed_term_expression(cursor, "unexpected token in mode definition");
            if let Some(recovery) = self.recover_malformed_definition_content_tail(cursor) {
                cursor = recovery.next_position;
                children.push(recovery.id);
                recovery_nodes.extend(recovery.recovery_nodes);
            }
            if self.is_semicolon_at(cursor) {
                children.push(self.token_node_ids[cursor]);
                cursor += 1;
            }
        }
        cursor
    }

    fn parse_mode_pattern_at(&mut self, position: usize) -> ParsedTypeNode {
        let cursor = self.mode_pattern_boundary_at(position);
        let mut children = self.token_node_ids[position..cursor].to_vec();
        let mut recovery_nodes = Vec::new();

        if !self.mode_pattern_can_match(position, cursor) {
            self.diagnose_malformed_term_expression(cursor, "expected mode pattern");
            self.push_missing_term(cursor, &mut children, &mut recovery_nodes);
        }

        let range = if cursor > position {
            self.covering_token_range(position, cursor)
        } else {
            self.zero_range_at(position)
        };
        let id = self.events.emit(SyntaxEvent::Node {
            kind: SurfaceNodeKind::ModePattern,
            range,
            children,
        });
        ParsedTypeNode {
            id,
            next_position: cursor.max(position),
            recovery_nodes,
        }
    }

    fn parse_mode_property_at(&mut self, position: usize) -> ParsedTypeNode {
        let mut children = vec![self.token_node_ids[position]];
        let mut recovery_nodes = Vec::new();
        let mut cursor = position + 1;

        if let Some(justification) =
            self.parse_definition_content_general_justification_at(cursor, true)
        {
            cursor = justification.next_position;
            children.push(justification.id);
            recovery_nodes.extend(justification.recovery_nodes);
        } else {
            self.diagnose_malformed_justification(cursor, "expected sethood justification");
            if !self.is_semicolon_at(cursor)
                && !self.is_definition_content_synchronization_boundary_at(cursor)
                && let Some(recovery) = self.recover_malformed_definition_content_tail(cursor)
            {
                cursor = recovery.next_position;
                children.push(recovery.id);
                recovery_nodes.extend(recovery.recovery_nodes);
            }
        }

        if self.is_semicolon_at(cursor) {
            children.push(self.token_node_ids[cursor]);
            cursor += 1;
        } else {
            self.diagnose_missing_semicolon(cursor);
        }

        let id = self.events.emit(SyntaxEvent::Node {
            kind: SurfaceNodeKind::ModeProperty,
            range: self.covering_token_range(position, cursor),
            children,
        });
        ParsedTypeNode {
            id,
            next_position: cursor.max(position + 1),
            recovery_nodes,
        }
    }

    fn parse_property_clause_at(&mut self, position: usize) -> ParsedTypeNode {
        let mut children = vec![self.token_node_ids[position]];
        let mut recovery_nodes = Vec::new();
        let mut cursor = position + 1;

        if let Some(justification) =
            self.parse_definition_content_general_justification_at(cursor, true)
        {
            cursor = justification.next_position;
            children.push(justification.id);
            recovery_nodes.extend(justification.recovery_nodes);
        } else {
            self.diagnose_malformed_justification(cursor, "expected property justification");
            self.push_missing_proof_step(cursor, &mut children, &mut recovery_nodes);
            if !self.is_semicolon_at(cursor)
                && !self.is_definition_content_synchronization_boundary_at(cursor)
                && let Some(recovery) = self.recover_malformed_definition_content_tail(cursor)
            {
                cursor = recovery.next_position;
                children.push(recovery.id);
                recovery_nodes.extend(recovery.recovery_nodes);
            }
        }

        if self.is_semicolon_at(cursor) {
            children.push(self.token_node_ids[cursor]);
            cursor += 1;
        } else {
            self.diagnose_missing_semicolon(cursor);
        }

        let id = self.events.emit(SyntaxEvent::Node {
            kind: SurfaceNodeKind::PropertyClause,
            range: self.covering_token_range(position, cursor),
            children,
        });
        ParsedTypeNode {
            id,
            next_position: cursor.max(position + 1),
            recovery_nodes,
        }
    }

    fn parse_structure_definition_at(&mut self, position: usize) -> ParsedTypeNode {
        let mut children = vec![self.token_node_ids[position]];
        let mut recovery_nodes = Vec::new();
        let mut cursor = position + 1;

        let pattern = self.parse_structure_pattern_at(cursor);
        cursor = pattern.next_position;
        children.push(pattern.id);
        recovery_nodes.extend(pattern.recovery_nodes);

        if self.is_reserved_word_at(cursor, "where") {
            children.push(self.token_node_ids[cursor]);
            cursor += 1;
        } else {
            self.diagnose_malformed_formula_expression(
                cursor,
                "expected `where` in structure definition",
            );
        }

        let mut saw_member = false;
        while cursor < self.request.tokens.len()
            && !self.is_end_keyword_at(cursor)
            && (!self.is_definition_content_start_at(cursor)
                || self.is_structure_member_start_at(cursor))
        {
            if let Some(member) = self.parse_structure_member_at(cursor) {
                saw_member = true;
                cursor = member.next_position;
                children.push(member.id);
                recovery_nodes.extend(member.recovery_nodes);
                continue;
            }

            self.diagnose_malformed_term_expression(cursor, "expected structure member");
            if let Some(recovery) = self.recover_malformed_structure_member_tail(cursor) {
                cursor = recovery.next_position;
                children.push(recovery.id);
                recovery_nodes.extend(recovery.recovery_nodes);
                if self.is_semicolon_at(cursor) {
                    children.push(self.token_node_ids[cursor]);
                    cursor += 1;
                }
            } else {
                break;
            }
        }

        if !saw_member {
            self.diagnose_malformed_term_expression(cursor, "expected structure member");
        }

        if self.is_end_keyword_at(cursor) {
            children.push(self.token_node_ids[cursor]);
            cursor += 1;
        } else {
            let missing = self.add_missing_end(position, cursor);
            children.push(missing);
            recovery_nodes.push(missing);
        }

        if self.is_semicolon_at(cursor) {
            children.push(self.token_node_ids[cursor]);
            cursor += 1;
        } else {
            self.diagnose_missing_semicolon(cursor);
        }

        let id = self.events.emit(SyntaxEvent::Node {
            kind: SurfaceNodeKind::StructureDefinition,
            range: self.covering_token_range(position, cursor),
            children,
        });
        ParsedTypeNode {
            id,
            next_position: cursor.max(position + 1),
            recovery_nodes,
        }
    }

    fn parse_structure_pattern_at(&mut self, position: usize) -> ParsedTypeNode {
        let cursor = self.structure_pattern_boundary_at(position);
        let mut children = self.token_node_ids[position..cursor].to_vec();
        let mut recovery_nodes = Vec::new();

        if !self.structure_pattern_can_match(position, cursor) {
            self.diagnose_malformed_term_expression(cursor, "expected structure pattern");
            self.push_missing_term(cursor, &mut children, &mut recovery_nodes);
        }

        let range = if cursor > position {
            self.covering_token_range(position, cursor)
        } else {
            self.zero_range_at(position)
        };
        let id = self.events.emit(SyntaxEvent::Node {
            kind: SurfaceNodeKind::StructurePattern,
            range,
            children,
        });
        ParsedTypeNode {
            id,
            next_position: cursor.max(position),
            recovery_nodes,
        }
    }

    fn parse_structure_member_at(&mut self, position: usize) -> Option<ParsedTypeNode> {
        if self.is_reserved_word_at(position, "field") {
            Some(self.parse_structure_field_at(position))
        } else if self.is_reserved_word_at(position, "property") {
            Some(self.parse_structure_property_at(position))
        } else {
            None
        }
    }

    fn parse_structure_field_at(&mut self, position: usize) -> ParsedTypeNode {
        let mut children = vec![self.token_node_ids[position]];
        let mut recovery_nodes = Vec::new();
        let mut cursor = position + 1;

        if self.is_identifier_at(cursor) {
            children.push(self.token_node_ids[cursor]);
            cursor += 1;
        } else {
            self.diagnose_malformed_term_expression(cursor, "expected structure field name");
            self.push_missing_term(cursor, &mut children, &mut recovery_nodes);
        }

        if self.is_reserved_symbol_at(cursor, "->") {
            children.push(self.token_node_ids[cursor]);
            cursor += 1;
        } else {
            self.diagnose_malformed_type_expression(cursor, "expected `->` after field name");
        }

        if let Some(field_type) = self.parse_type_expression_at(cursor) {
            cursor = field_type.next_position;
            children.push(field_type.id);
            recovery_nodes.extend(field_type.recovery_nodes);
        } else {
            self.diagnose_malformed_type_expression(cursor, "expected structure field type");
            let missing = self.add_missing_type_expression(cursor);
            children.push(missing);
            recovery_nodes.push(missing);
        }

        if self.is_reserved_symbol_at(cursor, ":=") {
            children.push(self.token_node_ids[cursor]);
            cursor += 1;
            if let Some(default) = self.parse_term_expression_at(cursor) {
                cursor = default.next_position;
                children.push(default.id);
                recovery_nodes.extend(default.recovery_nodes);
            } else {
                self.diagnose_malformed_term_expression(
                    cursor,
                    "expected structure field initializer",
                );
                self.push_missing_term(cursor, &mut children, &mut recovery_nodes);
            }
        }

        self.finish_structure_member_node(
            position,
            cursor,
            SurfaceNodeKind::StructureField,
            children,
            recovery_nodes,
            "unexpected token in structure field",
        )
    }

    fn parse_structure_property_at(&mut self, position: usize) -> ParsedTypeNode {
        let mut children = vec![self.token_node_ids[position]];
        let mut recovery_nodes = Vec::new();
        let mut cursor = position + 1;

        if self.is_identifier_at(cursor) {
            children.push(self.token_node_ids[cursor]);
            cursor += 1;
        } else {
            self.diagnose_malformed_term_expression(cursor, "expected structure property name");
            self.push_missing_term(cursor, &mut children, &mut recovery_nodes);
        }

        if self.is_reserved_symbol_at(cursor, "->") {
            children.push(self.token_node_ids[cursor]);
            cursor += 1;
        } else {
            self.diagnose_malformed_type_expression(cursor, "expected `->` after property name");
        }

        if let Some(property_type) = self.parse_type_expression_at(cursor) {
            cursor = property_type.next_position;
            children.push(property_type.id);
            recovery_nodes.extend(property_type.recovery_nodes);
        } else {
            self.diagnose_malformed_type_expression(cursor, "expected structure property type");
            let missing = self.add_missing_type_expression(cursor);
            children.push(missing);
            recovery_nodes.push(missing);
        }

        self.finish_structure_member_node(
            position,
            cursor,
            SurfaceNodeKind::StructureProperty,
            children,
            recovery_nodes,
            "unexpected token in structure property",
        )
    }

    fn finish_structure_member_node(
        &mut self,
        position: usize,
        mut cursor: usize,
        kind: SurfaceNodeKind,
        mut children: Vec<SurfaceBuilderNodeId>,
        mut recovery_nodes: Vec<SurfaceBuilderNodeId>,
        unexpected_message: &'static str,
    ) -> ParsedTypeNode {
        if self.is_semicolon_at(cursor) {
            children.push(self.token_node_ids[cursor]);
            cursor += 1;
        } else if cursor < self.request.tokens.len()
            && !self.is_structure_member_boundary_at(cursor)
        {
            self.diagnose_malformed_term_expression(cursor, unexpected_message);
            if let Some(recovery) = self.recover_malformed_structure_member_tail(cursor) {
                cursor = recovery.next_position;
                children.push(recovery.id);
                recovery_nodes.extend(recovery.recovery_nodes);
            }
            if self.is_semicolon_at(cursor) {
                children.push(self.token_node_ids[cursor]);
                cursor += 1;
            } else {
                self.diagnose_missing_semicolon(cursor);
            }
        } else {
            self.diagnose_missing_semicolon(cursor);
        }

        let id = self.events.emit(SyntaxEvent::Node {
            kind,
            range: self.covering_token_range(position, cursor),
            children,
        });
        ParsedTypeNode {
            id,
            next_position: cursor.max(position + 1),
            recovery_nodes,
        }
    }

    fn parse_inheritance_definition_at(&mut self, position: usize) -> ParsedTypeNode {
        let mut children = vec![self.token_node_ids[position]];
        let mut recovery_nodes = Vec::new();
        let mut cursor = position + 1;

        let child = self.parse_inheritance_target_at(
            cursor,
            InheritanceTargetBoundary::Extends,
            false,
            "expected inherited structure name",
        );
        cursor = child.next_position;
        children.push(child.id);
        recovery_nodes.extend(child.recovery_nodes);

        if self.is_reserved_word_at(cursor, "extends") {
            children.push(self.token_node_ids[cursor]);
            cursor += 1;
        } else {
            self.diagnose_malformed_formula_expression(
                cursor,
                "expected `extends` in inheritance definition",
            );
        }

        let parent = self.parse_inheritance_target_at(
            cursor,
            InheritanceTargetBoundary::Tail,
            true,
            "expected parent structure type",
        );
        cursor = parent.next_position;
        children.push(parent.id);
        recovery_nodes.extend(parent.recovery_nodes);

        if self.is_reserved_word_at(cursor, "where") {
            children.push(self.token_node_ids[cursor]);
            cursor += 1;

            let mut saw_member = false;
            while cursor < self.request.tokens.len()
                && !self.is_end_keyword_at(cursor)
                && !self.is_reserved_word_at(cursor, "coherence")
                && (!self.is_definition_content_start_at(cursor)
                    || self.is_inheritance_member_start_at(cursor))
            {
                if let Some(member) = self.parse_inheritance_member_at(cursor) {
                    saw_member = true;
                    cursor = member.next_position;
                    children.push(member.id);
                    recovery_nodes.extend(member.recovery_nodes);
                    continue;
                }

                self.diagnose_malformed_term_expression(cursor, "expected inheritance member");
                if let Some(recovery) = self.recover_malformed_inheritance_member_tail(cursor) {
                    cursor = recovery.next_position;
                    children.push(recovery.id);
                    recovery_nodes.extend(recovery.recovery_nodes);
                    if self.is_semicolon_at(cursor) {
                        children.push(self.token_node_ids[cursor]);
                        cursor += 1;
                    }
                } else {
                    break;
                }
            }

            if !saw_member {
                self.diagnose_malformed_term_expression(cursor, "expected inheritance member");
            }

            if self.is_reserved_word_at(cursor, "coherence") {
                let coherence = self.parse_inheritance_coherence_condition_at(cursor);
                cursor = coherence.next_position;
                children.push(coherence.id);
                recovery_nodes.extend(coherence.recovery_nodes);
            }

            if self.is_end_keyword_at(cursor) {
                children.push(self.token_node_ids[cursor]);
                cursor += 1;
            } else {
                let missing = self.add_missing_end(position, cursor);
                children.push(missing);
                recovery_nodes.push(missing);
            }
        }

        if self.is_semicolon_at(cursor) {
            children.push(self.token_node_ids[cursor]);
            cursor += 1;
        } else {
            self.diagnose_missing_semicolon(cursor);
        }

        let id = self.events.emit(SyntaxEvent::Node {
            kind: SurfaceNodeKind::InheritanceDefinition,
            range: self.covering_token_range(position, cursor),
            children,
        });
        ParsedTypeNode {
            id,
            next_position: cursor.max(position + 1),
            recovery_nodes,
        }
    }

    fn parse_inheritance_target_at(
        &mut self,
        position: usize,
        boundary: InheritanceTargetBoundary,
        allow_set: bool,
        diagnostic_message: &'static str,
    ) -> ParsedTypeNode {
        let cursor = self.inheritance_target_boundary_at(position, boundary);
        let mut children = self.token_node_ids[position..cursor].to_vec();
        let mut recovery_nodes = Vec::new();

        if !self.inheritance_target_can_match(position, cursor, allow_set) {
            self.diagnose_malformed_term_expression(cursor, diagnostic_message);
            self.push_missing_term(cursor, &mut children, &mut recovery_nodes);
        }

        let range = if cursor > position {
            self.covering_token_range(position, cursor)
        } else {
            self.zero_range_at(position)
        };
        let id = self.events.emit(SyntaxEvent::Node {
            kind: SurfaceNodeKind::InheritanceTarget,
            range,
            children,
        });
        ParsedTypeNode {
            id,
            next_position: cursor.max(position),
            recovery_nodes,
        }
    }

    fn parse_inheritance_member_at(&mut self, position: usize) -> Option<ParsedTypeNode> {
        if self.is_reserved_word_at(position, "field") {
            Some(self.parse_field_redefinition_at(position))
        } else if self.is_reserved_word_at(position, "property") {
            Some(self.parse_property_redefinition_at(position))
        } else {
            None
        }
    }

    fn parse_field_redefinition_at(&mut self, position: usize) -> ParsedTypeNode {
        let mut children = vec![self.token_node_ids[position]];
        let mut recovery_nodes = Vec::new();
        let mut cursor = position + 1;

        if self.is_identifier_at(cursor) {
            children.push(self.token_node_ids[cursor]);
            cursor += 1;
        } else {
            self.diagnose_malformed_term_expression(cursor, "expected inherited field name");
            self.push_missing_term(cursor, &mut children, &mut recovery_nodes);
        }

        if self.is_reserved_symbol_at(cursor, "->") {
            children.push(self.token_node_ids[cursor]);
            cursor += 1;
            if let Some(field_type) = self.parse_type_expression_at(cursor) {
                cursor = field_type.next_position;
                children.push(field_type.id);
                recovery_nodes.extend(field_type.recovery_nodes);
            } else {
                self.diagnose_malformed_type_expression(cursor, "expected inherited field type");
                let missing = self.add_missing_type_expression(cursor);
                children.push(missing);
                recovery_nodes.push(missing);
            }
        }

        if self.is_reserved_word_at(cursor, "from") {
            children.push(self.token_node_ids[cursor]);
            cursor += 1;
            if self.is_identifier_at(cursor) || self.is_reserved_word_at(cursor, "it") {
                children.push(self.token_node_ids[cursor]);
                cursor += 1;
            } else {
                self.diagnose_malformed_term_expression(cursor, "expected inherited field source");
                self.push_missing_term(cursor, &mut children, &mut recovery_nodes);
            }
        } else {
            self.diagnose_malformed_term_expression(cursor, "expected `from` in field inheritance");
        }

        self.finish_inheritance_member_node(
            position,
            cursor,
            SurfaceNodeKind::FieldRedefinition,
            children,
            recovery_nodes,
            "unexpected token in field inheritance",
        )
    }

    fn parse_property_redefinition_at(&mut self, position: usize) -> ParsedTypeNode {
        let mut children = vec![self.token_node_ids[position]];
        let mut recovery_nodes = Vec::new();
        let mut cursor = position + 1;

        if self.is_identifier_at(cursor) {
            children.push(self.token_node_ids[cursor]);
            cursor += 1;
        } else {
            self.diagnose_malformed_term_expression(cursor, "expected inherited property name");
            self.push_missing_term(cursor, &mut children, &mut recovery_nodes);
        }

        if self.is_reserved_symbol_at(cursor, "->") {
            children.push(self.token_node_ids[cursor]);
            cursor += 1;
            if let Some(property_type) = self.parse_type_expression_at(cursor) {
                cursor = property_type.next_position;
                children.push(property_type.id);
                recovery_nodes.extend(property_type.recovery_nodes);
            } else {
                self.diagnose_malformed_type_expression(cursor, "expected inherited property type");
                let missing = self.add_missing_type_expression(cursor);
                children.push(missing);
                recovery_nodes.push(missing);
            }
        }

        if self.is_reserved_word_at(cursor, "from") {
            children.push(self.token_node_ids[cursor]);
            cursor += 1;
            if self.is_identifier_at(cursor) {
                children.push(self.token_node_ids[cursor]);
                cursor += 1;
            } else {
                self.diagnose_malformed_term_expression(
                    cursor,
                    "expected inherited property source",
                );
                self.push_missing_term(cursor, &mut children, &mut recovery_nodes);
            }
        } else {
            self.diagnose_malformed_term_expression(
                cursor,
                "expected `from` in property inheritance",
            );
        }

        self.finish_inheritance_member_node(
            position,
            cursor,
            SurfaceNodeKind::PropertyRedefinition,
            children,
            recovery_nodes,
            "unexpected token in property inheritance",
        )
    }

    fn finish_inheritance_member_node(
        &mut self,
        position: usize,
        mut cursor: usize,
        kind: SurfaceNodeKind,
        mut children: Vec<SurfaceBuilderNodeId>,
        mut recovery_nodes: Vec<SurfaceBuilderNodeId>,
        unexpected_message: &'static str,
    ) -> ParsedTypeNode {
        if self.is_semicolon_at(cursor) {
            children.push(self.token_node_ids[cursor]);
            cursor += 1;
        } else if cursor < self.request.tokens.len()
            && !self.is_inheritance_member_boundary_at(cursor)
        {
            self.diagnose_malformed_term_expression(cursor, unexpected_message);
            if let Some(recovery) = self.recover_malformed_inheritance_member_tail(cursor) {
                cursor = recovery.next_position;
                children.push(recovery.id);
                recovery_nodes.extend(recovery.recovery_nodes);
            }
            if self.is_semicolon_at(cursor) {
                children.push(self.token_node_ids[cursor]);
                cursor += 1;
            } else {
                self.diagnose_missing_semicolon(cursor);
            }
        } else {
            self.diagnose_missing_semicolon(cursor);
        }

        let id = self.events.emit(SyntaxEvent::Node {
            kind,
            range: self.covering_token_range(position, cursor),
            children,
        });
        ParsedTypeNode {
            id,
            next_position: cursor.max(position + 1),
            recovery_nodes,
        }
    }

    fn parse_inheritance_coherence_condition_at(&mut self, position: usize) -> ParsedTypeNode {
        let mut children = vec![self.token_node_ids[position]];
        let mut recovery_nodes = Vec::new();
        let mut cursor = position + 1;

        if let Some(justification) =
            self.parse_definition_content_general_justification_at(cursor, true)
        {
            cursor = justification.next_position;
            children.push(justification.id);
            recovery_nodes.extend(justification.recovery_nodes);
        } else {
            self.diagnose_malformed_justification(
                cursor,
                "expected inheritance coherence justification",
            );
            self.push_missing_proof_step(cursor, &mut children, &mut recovery_nodes);
            if !self.is_semicolon_at(cursor)
                && !self.is_inheritance_member_boundary_at(cursor)
                && let Some(recovery) = self.recover_malformed_inheritance_member_tail(cursor)
            {
                cursor = recovery.next_position;
                children.push(recovery.id);
                recovery_nodes.extend(recovery.recovery_nodes);
            }
        }

        if self.is_semicolon_at(cursor) {
            children.push(self.token_node_ids[cursor]);
            cursor += 1;
        } else {
            self.diagnose_missing_semicolon(cursor);
        }

        let id = self.events.emit(SyntaxEvent::Node {
            kind: SurfaceNodeKind::CoherenceCondition,
            range: self.covering_token_range(position, cursor),
            children,
        });
        ParsedTypeNode {
            id,
            next_position: cursor.max(position + 1),
            recovery_nodes,
        }
    }

    fn parse_redefinition_at(&mut self, position: usize) -> ParsedTypeNode {
        if self.is_reserved_word_at(position + 1, "attr") {
            self.parse_attribute_redefinition_at(position)
        } else if self.is_reserved_word_at(position + 1, "pred") {
            self.parse_predicate_redefinition_at(position)
        } else {
            self.parse_functor_redefinition_at(position)
        }
    }

    fn parse_attribute_redefinition_at(&mut self, position: usize) -> ParsedTypeNode {
        let mut children = vec![
            self.token_node_ids[position],
            self.token_node_ids[position + 1],
        ];
        let mut recovery_nodes = Vec::new();
        let mut cursor = position + 2;

        if self.is_identifier_at(cursor) {
            children.push(self.token_node_ids[cursor]);
            cursor += 1;
        } else {
            self.diagnose_malformed_term_expression(
                cursor,
                "expected attribute redefinition label",
            );
            self.push_missing_term(cursor, &mut children, &mut recovery_nodes);
        }

        if self.is_reserved_symbol_at(cursor, ":") {
            children.push(self.token_node_ids[cursor]);
            cursor += 1;
        } else {
            self.diagnose_malformed_formula_expression(
                cursor,
                "expected `:` after attribute redefinition label",
            );
        }

        if self.is_identifier_at(cursor) {
            children.push(self.token_node_ids[cursor]);
            cursor += 1;
        } else {
            self.diagnose_malformed_term_expression(cursor, "expected attribute subject");
            self.push_missing_term(cursor, &mut children, &mut recovery_nodes);
        }

        if self.is_reserved_word_at(cursor, "is") {
            children.push(self.token_node_ids[cursor]);
            cursor += 1;
        } else {
            self.diagnose_malformed_formula_expression(
                cursor,
                "expected `is` before attribute redefinition pattern",
            );
        }

        let pattern = self.parse_attribute_pattern_at(cursor);
        cursor = pattern.next_position;
        children.push(pattern.id);
        recovery_nodes.extend(pattern.recovery_nodes);

        if self.is_reserved_word_at(cursor, "means") {
            children.push(self.token_node_ids[cursor]);
            cursor += 1;
        } else {
            self.diagnose_malformed_formula_expression(
                cursor,
                "expected `means` in attribute redefinition",
            );
        }

        let definiens = self.parse_formula_definiens_at(cursor);
        cursor = definiens.next_position;
        children.push(definiens.id);
        recovery_nodes.extend(definiens.recovery_nodes);

        cursor = self.parse_redefinition_body_semicolon(
            cursor,
            &mut children,
            &mut recovery_nodes,
            "unexpected token in attribute redefinition",
        );
        let coherence = self.parse_coherence_condition_at(cursor);
        cursor = coherence.next_position;
        children.push(coherence.id);
        recovery_nodes.extend(coherence.recovery_nodes);

        let id = self.events.emit(SyntaxEvent::Node {
            kind: SurfaceNodeKind::AttributeRedefinition,
            range: self.covering_token_range(position, cursor),
            children,
        });
        ParsedTypeNode {
            id,
            next_position: cursor.max(position + 1),
            recovery_nodes,
        }
    }

    fn parse_predicate_redefinition_at(&mut self, position: usize) -> ParsedTypeNode {
        let mut children = vec![
            self.token_node_ids[position],
            self.token_node_ids[position + 1],
        ];
        let mut recovery_nodes = Vec::new();
        let mut cursor = position + 2;

        let pattern = self.parse_predicate_pattern_at(cursor);
        cursor = pattern.next_position;
        children.push(pattern.id);
        recovery_nodes.extend(pattern.recovery_nodes);

        if self.is_reserved_word_at(cursor, "means") {
            children.push(self.token_node_ids[cursor]);
            cursor += 1;
        } else {
            self.diagnose_malformed_formula_expression(
                cursor,
                "expected `means` in predicate redefinition",
            );
        }

        let definiens = self.parse_formula_definiens_at(cursor);
        cursor = definiens.next_position;
        children.push(definiens.id);
        recovery_nodes.extend(definiens.recovery_nodes);

        cursor = self.parse_redefinition_body_semicolon(
            cursor,
            &mut children,
            &mut recovery_nodes,
            "unexpected token in predicate redefinition",
        );
        let coherence = self.parse_coherence_condition_at(cursor);
        cursor = coherence.next_position;
        children.push(coherence.id);
        recovery_nodes.extend(coherence.recovery_nodes);

        let id = self.events.emit(SyntaxEvent::Node {
            kind: SurfaceNodeKind::PredicateRedefinition,
            range: self.covering_token_range(position, cursor),
            children,
        });
        ParsedTypeNode {
            id,
            next_position: cursor.max(position + 1),
            recovery_nodes,
        }
    }

    fn parse_functor_redefinition_at(&mut self, position: usize) -> ParsedTypeNode {
        let mut children = vec![
            self.token_node_ids[position],
            self.token_node_ids[position + 1],
        ];
        let mut recovery_nodes = Vec::new();
        let mut cursor = position + 2;

        if self.is_identifier_at(cursor) {
            children.push(self.token_node_ids[cursor]);
            cursor += 1;
        } else {
            self.diagnose_malformed_term_expression(cursor, "expected functor redefinition label");
            self.push_missing_term(cursor, &mut children, &mut recovery_nodes);
        }

        if self.is_reserved_symbol_at(cursor, ":") {
            children.push(self.token_node_ids[cursor]);
            cursor += 1;
        } else {
            self.diagnose_malformed_formula_expression(
                cursor,
                "expected `:` after functor redefinition label",
            );
        }

        let pattern = self.parse_functor_pattern_at(cursor);
        cursor = pattern.next_position;
        children.push(pattern.id);
        recovery_nodes.extend(pattern.recovery_nodes);

        if self.is_reserved_symbol_at(cursor, "->") {
            children.push(self.token_node_ids[cursor]);
            cursor += 1;
        } else {
            self.diagnose_malformed_formula_expression(
                cursor,
                "expected `->` before functor redefinition return type",
            );
        }

        if let Some(return_type) = self.parse_type_expression_at(cursor) {
            cursor = return_type.next_position;
            children.push(return_type.id);
            recovery_nodes.extend(return_type.recovery_nodes);
        } else {
            self.diagnose_malformed_type_expression(
                cursor,
                "expected functor redefinition return type",
            );
            let missing = self.add_missing_type_expression(cursor);
            children.push(missing);
            recovery_nodes.push(missing);
        }

        if self.is_reserved_word_at(cursor, "means") {
            children.push(self.token_node_ids[cursor]);
            cursor += 1;
            let definiens = self.parse_formula_definiens_at(cursor);
            cursor = definiens.next_position;
            children.push(definiens.id);
            recovery_nodes.extend(definiens.recovery_nodes);
        } else if self.is_reserved_word_at(cursor, "equals") {
            children.push(self.token_node_ids[cursor]);
            cursor += 1;
            let definiens = self.parse_term_definiens_at(cursor);
            cursor = definiens.next_position;
            children.push(definiens.id);
            recovery_nodes.extend(definiens.recovery_nodes);
        } else {
            self.diagnose_malformed_formula_expression(
                cursor,
                "expected `means` or `equals` in functor redefinition",
            );
            if self.can_start_formula_at(cursor) {
                let definiens = self.parse_formula_definiens_at(cursor);
                cursor = definiens.next_position;
                children.push(definiens.id);
                recovery_nodes.extend(definiens.recovery_nodes);
            } else if self.can_start_term_operator_operand_at(cursor, false) {
                let definiens = self.parse_term_definiens_at(cursor);
                cursor = definiens.next_position;
                children.push(definiens.id);
                recovery_nodes.extend(definiens.recovery_nodes);
            } else {
                let definiens = self.parse_formula_definiens_at(cursor);
                cursor = definiens.next_position;
                children.push(definiens.id);
                recovery_nodes.extend(definiens.recovery_nodes);
            }
        }

        cursor = self.parse_redefinition_body_semicolon(
            cursor,
            &mut children,
            &mut recovery_nodes,
            "unexpected token in functor redefinition",
        );
        let coherence = self.parse_coherence_condition_at(cursor);
        cursor = coherence.next_position;
        children.push(coherence.id);
        recovery_nodes.extend(coherence.recovery_nodes);

        let id = self.events.emit(SyntaxEvent::Node {
            kind: SurfaceNodeKind::FunctorRedefinition,
            range: self.covering_token_range(position, cursor),
            children,
        });
        ParsedTypeNode {
            id,
            next_position: cursor.max(position + 1),
            recovery_nodes,
        }
    }

    fn parse_redefinition_body_semicolon(
        &mut self,
        mut cursor: usize,
        children: &mut Vec<SurfaceBuilderNodeId>,
        recovery_nodes: &mut Vec<SurfaceBuilderNodeId>,
        unexpected_message: &'static str,
    ) -> usize {
        if self.is_semicolon_at(cursor) {
            children.push(self.token_node_ids[cursor]);
            return cursor + 1;
        }

        if cursor < self.request.tokens.len()
            && !self.is_definition_content_synchronization_boundary_at(cursor)
        {
            self.diagnose_malformed_term_expression(cursor, unexpected_message);
            if let Some(recovery) = self.recover_malformed_definition_content_tail(cursor) {
                cursor = recovery.next_position;
                children.push(recovery.id);
                recovery_nodes.extend(recovery.recovery_nodes);
            }
            if self.is_semicolon_at(cursor) {
                children.push(self.token_node_ids[cursor]);
                cursor += 1;
            } else {
                self.diagnose_missing_semicolon(cursor);
            }
        } else {
            self.diagnose_missing_semicolon(cursor);
        }
        cursor
    }

    fn parse_coherence_condition_at(&mut self, position: usize) -> ParsedTypeNode {
        let mut children = Vec::new();
        let mut recovery_nodes = Vec::new();
        let mut cursor = position;

        if self.is_reserved_word_at(cursor, "coherence") {
            children.push(self.token_node_ids[cursor]);
            cursor += 1;
        } else {
            self.diagnose_malformed_formula_expression(
                cursor,
                "expected `coherence` after redefinition body",
            );
        }

        if self.is_reserved_word_at(cursor, "with") {
            children.push(self.token_node_ids[cursor]);
            cursor += 1;
            if self.is_identifier_at(cursor) {
                children.push(self.token_node_ids[cursor]);
                cursor += 1;
            } else {
                self.diagnose_malformed_justification(
                    cursor,
                    "expected label after `coherence with`",
                );
                self.push_missing_proof_step(cursor, &mut children, &mut recovery_nodes);
            }
        }

        if let Some(justification) = self.parse_general_justification_at(cursor, true) {
            cursor = justification.next_position;
            children.push(justification.id);
            recovery_nodes.extend(justification.recovery_nodes);
        } else {
            self.diagnose_malformed_justification(cursor, "expected coherence justification");
            self.push_missing_proof_step(cursor, &mut children, &mut recovery_nodes);
            if !self.is_semicolon_at(cursor)
                && !self.is_definition_content_synchronization_boundary_at(cursor)
                && let Some(recovery) = self.recover_malformed_definition_content_tail(cursor)
            {
                cursor = recovery.next_position;
                children.push(recovery.id);
                recovery_nodes.extend(recovery.recovery_nodes);
            }
        }

        if self.is_semicolon_at(cursor) {
            children.push(self.token_node_ids[cursor]);
            cursor += 1;
        } else {
            self.diagnose_missing_semicolon(cursor);
        }

        let id = self.events.emit(SyntaxEvent::Node {
            kind: SurfaceNodeKind::CoherenceCondition,
            range: if cursor > position {
                self.covering_token_range(position, cursor)
            } else {
                self.zero_range_at(position)
            },
            children,
        });
        ParsedTypeNode {
            id,
            next_position: cursor,
            recovery_nodes,
        }
    }

    fn parse_notation_alias_item(&mut self, position: usize) -> ParsedItem {
        let alias = self.parse_notation_alias_at(position);
        ParsedItem {
            id: alias.id,
            next_position: alias.next_position,
            recovery_nodes: alias.recovery_nodes,
        }
    }

    fn parse_notation_alias_at(&mut self, position: usize) -> ParsedTypeNode {
        let head = self.item_head_position(position).unwrap_or(position);
        let mut children = self.token_node_ids[position..=head].to_vec();
        let mut recovery_nodes = Vec::new();
        let mut cursor = head + 1;

        let alternate = self.parse_notation_pattern_at(cursor, true);
        cursor = alternate.next_position;
        children.push(alternate.id);
        recovery_nodes.extend(alternate.recovery_nodes);

        if self.is_reserved_word_at(cursor, "for") {
            children.push(self.token_node_ids[cursor]);
            cursor += 1;
        } else {
            self.diagnose_malformed_formula_expression(cursor, "expected `for` in notation alias");
        }

        let original = self.parse_notation_pattern_at(cursor, false);
        cursor = original.next_position;
        children.push(original.id);
        recovery_nodes.extend(original.recovery_nodes);

        if self.is_semicolon_at(cursor) {
            children.push(self.token_node_ids[cursor]);
            cursor += 1;
        } else if cursor < self.request.tokens.len()
            && !self.is_notation_alias_synchronization_boundary_at(cursor)
        {
            self.diagnose_malformed_term_expression(cursor, "unexpected token in notation alias");
            if let Some(recovery) = self.recover_malformed_definition_content_tail(cursor) {
                cursor = recovery.next_position;
                children.push(recovery.id);
                recovery_nodes.extend(recovery.recovery_nodes);
            }
            if self.is_semicolon_at(cursor) {
                children.push(self.token_node_ids[cursor]);
                cursor += 1;
            } else {
                self.diagnose_missing_semicolon(cursor);
            }
        } else {
            self.diagnose_missing_semicolon(cursor);
        }

        let id = self.events.emit(SyntaxEvent::Node {
            kind: SurfaceNodeKind::NotationAlias,
            range: self.covering_token_range(position, cursor),
            children,
        });
        ParsedTypeNode {
            id,
            next_position: cursor.max(position + 1),
            recovery_nodes,
        }
    }

    fn parse_notation_pattern_at(&mut self, position: usize, stop_at_for: bool) -> ParsedTypeNode {
        let cursor = self.notation_pattern_boundary_at(position, stop_at_for);
        let mut children = self.token_node_ids[position..cursor].to_vec();
        let mut recovery_nodes = Vec::new();

        if cursor == position {
            self.diagnose_malformed_term_expression(cursor, "expected notation pattern");
            self.push_missing_term(cursor, &mut children, &mut recovery_nodes);
        }

        let range = if cursor > position {
            self.covering_token_range(position, cursor)
        } else {
            self.zero_range_at(position)
        };
        let id = self.events.emit(SyntaxEvent::Node {
            kind: SurfaceNodeKind::NotationPattern,
            range,
            children,
        });
        ParsedTypeNode {
            id,
            next_position: cursor.max(position),
            recovery_nodes,
        }
    }

    fn notation_pattern_boundary_at(&self, position: usize, stop_at_for: bool) -> usize {
        let mut cursor = position;
        let mut paren_depth = 0_usize;
        let mut bracket_depth = 0_usize;
        let mut brace_depth = 0_usize;

        while cursor < self.request.tokens.len() {
            let top_level = paren_depth == 0 && bracket_depth == 0 && brace_depth == 0;
            if top_level
                && (self.is_semicolon_at(cursor)
                    || self.is_end_keyword_at(cursor)
                    || self.is_item_start_at(cursor)
                    || self.is_definition_content_start_at(cursor)
                    || (stop_at_for && self.is_reserved_word_at(cursor, "for")))
            {
                break;
            }

            if self.is_reserved_symbol_at(cursor, "(") {
                paren_depth += 1;
            } else if self.is_reserved_symbol_at(cursor, ")") {
                if paren_depth == 0 {
                    break;
                }
                paren_depth -= 1;
            } else if self.is_reserved_symbol_at(cursor, "[") {
                bracket_depth += 1;
            } else if self.is_reserved_symbol_at(cursor, "]") {
                if bracket_depth == 0 {
                    break;
                }
                bracket_depth -= 1;
            } else if self.is_reserved_symbol_at(cursor, "{") {
                brace_depth += 1;
            } else if self.is_reserved_symbol_at(cursor, "}") {
                if brace_depth == 0 {
                    break;
                }
                brace_depth -= 1;
            }
            cursor += 1;
        }
        cursor
    }

    fn is_notation_alias_synchronization_boundary_at(&self, position: usize) -> bool {
        position >= self.request.tokens.len()
            || self.is_semicolon_at(position)
            || self.is_end_keyword_at(position)
            || self.is_item_start_at(position)
            || self.is_definition_content_start_at(position)
    }

    fn parse_formula_definiens_at(&mut self, position: usize) -> ParsedTypeNode {
        let mut children = Vec::new();
        let mut recovery_nodes = Vec::new();
        let mut cursor = position;

        let first_formula = if let Some(formula) = self.parse_formula_expression_at(cursor) {
            cursor = formula.next_position;
            formula
        } else {
            self.diagnose_malformed_formula_expression(cursor, "expected formula definiens");
            let missing = self.add_missing_formula(cursor);
            ParsedTypeNode {
                id: missing,
                next_position: cursor,
                recovery_nodes: vec![missing],
            }
        };

        if self.is_reserved_word_at(cursor, "if") {
            let case = self.finish_formula_case(position, first_formula, cursor);
            cursor = case.next_position;
            children.push(case.id);
            recovery_nodes.extend(case.recovery_nodes);

            while self.is_reserved_symbol_at(cursor, ",") {
                children.push(self.token_node_ids[cursor]);
                cursor += 1;
                let case = self.parse_formula_case_at(cursor);
                let made_progress = case.next_position > cursor;
                cursor = case.next_position;
                children.push(case.id);
                recovery_nodes.extend(case.recovery_nodes);
                if !made_progress {
                    break;
                }
            }

            if self.is_reserved_word_at(cursor, "otherwise") {
                children.push(self.token_node_ids[cursor]);
                cursor += 1;
                if let Some(formula) = self.parse_formula_expression_at(cursor) {
                    cursor = formula.next_position;
                    children.push(formula.id);
                    recovery_nodes.extend(formula.recovery_nodes);
                } else {
                    self.diagnose_malformed_formula_expression(
                        cursor,
                        "expected formula after `otherwise`",
                    );
                    self.push_missing_formula(cursor, &mut children, &mut recovery_nodes);
                }
            }
        } else {
            recovery_nodes.extend(first_formula.recovery_nodes.clone());
            children.push(first_formula.id);
        }

        let range = if cursor > position {
            self.covering_token_range(position, cursor)
        } else {
            self.zero_range_at(position)
        };
        let id = self.events.emit(SyntaxEvent::Node {
            kind: SurfaceNodeKind::FormulaDefiniens,
            range,
            children,
        });
        ParsedTypeNode {
            id,
            next_position: cursor.max(position),
            recovery_nodes,
        }
    }

    fn parse_term_definiens_at(&mut self, position: usize) -> ParsedTypeNode {
        let mut children = Vec::new();
        let mut recovery_nodes = Vec::new();
        let mut cursor = position;

        let first_term = if let Some(term) = self.parse_term_expression_at(cursor) {
            cursor = term.next_position;
            term
        } else {
            self.diagnose_malformed_term_expression(cursor, "expected term definiens");
            let missing = self.add_missing_term(cursor);
            ParsedTypeNode {
                id: missing,
                next_position: cursor,
                recovery_nodes: vec![missing],
            }
        };

        if self.is_reserved_word_at(cursor, "if") {
            let case = self.finish_term_case(position, first_term, cursor);
            cursor = case.next_position;
            children.push(case.id);
            recovery_nodes.extend(case.recovery_nodes);

            while self.is_reserved_symbol_at(cursor, ",") {
                children.push(self.token_node_ids[cursor]);
                cursor += 1;
                let case = self.parse_term_case_at(cursor);
                let made_progress = case.next_position > cursor;
                cursor = case.next_position;
                children.push(case.id);
                recovery_nodes.extend(case.recovery_nodes);
                if !made_progress {
                    break;
                }
            }

            if self.is_reserved_word_at(cursor, "otherwise") {
                children.push(self.token_node_ids[cursor]);
                cursor += 1;
                if let Some(term) = self.parse_term_expression_at(cursor) {
                    cursor = term.next_position;
                    children.push(term.id);
                    recovery_nodes.extend(term.recovery_nodes);
                } else {
                    self.diagnose_malformed_term_expression(
                        cursor,
                        "expected term after `otherwise`",
                    );
                    self.push_missing_term(cursor, &mut children, &mut recovery_nodes);
                }
            }
        } else {
            recovery_nodes.extend(first_term.recovery_nodes.clone());
            children.push(first_term.id);
        }

        let range = if cursor > position {
            self.covering_token_range(position, cursor)
        } else {
            self.zero_range_at(position)
        };
        let id = self.events.emit(SyntaxEvent::Node {
            kind: SurfaceNodeKind::TermDefiniens,
            range,
            children,
        });
        ParsedTypeNode {
            id,
            next_position: cursor.max(position),
            recovery_nodes,
        }
    }

    fn parse_term_case_at(&mut self, position: usize) -> ParsedTypeNode {
        let mut cursor = position;
        let value = if let Some(term) = self.parse_term_expression_at(cursor) {
            cursor = term.next_position;
            term
        } else {
            self.diagnose_malformed_term_expression(cursor, "expected term before `if`");
            let missing = self.add_missing_term(cursor);
            ParsedTypeNode {
                id: missing,
                next_position: cursor,
                recovery_nodes: vec![missing],
            }
        };
        self.finish_term_case(position, value, cursor)
    }

    fn finish_term_case(
        &mut self,
        position: usize,
        value: ParsedTypeNode,
        mut cursor: usize,
    ) -> ParsedTypeNode {
        let mut children = vec![value.id];
        let mut recovery_nodes = value.recovery_nodes;

        if self.is_reserved_word_at(cursor, "if") {
            children.push(self.token_node_ids[cursor]);
            cursor += 1;
        } else {
            self.diagnose_malformed_formula_expression(cursor, "expected `if` in term case");
        }

        if let Some(condition) = self.parse_formula_expression_at(cursor) {
            cursor = condition.next_position;
            children.push(condition.id);
            recovery_nodes.extend(condition.recovery_nodes);
        } else {
            self.diagnose_malformed_formula_expression(cursor, "expected formula after `if`");
            self.push_missing_formula(cursor, &mut children, &mut recovery_nodes);
        }

        let id = self.events.emit(SyntaxEvent::Node {
            kind: SurfaceNodeKind::TermCase,
            range: self.covering_token_range(position, cursor.max(position + 1)),
            children,
        });
        ParsedTypeNode {
            id,
            next_position: cursor.max(position + 1),
            recovery_nodes,
        }
    }

    fn parse_formula_case_at(&mut self, position: usize) -> ParsedTypeNode {
        let mut cursor = position;
        let value = if let Some(formula) = self.parse_formula_expression_at(cursor) {
            cursor = formula.next_position;
            formula
        } else {
            self.diagnose_malformed_formula_expression(cursor, "expected formula before `if`");
            let missing = self.add_missing_formula(cursor);
            ParsedTypeNode {
                id: missing,
                next_position: cursor,
                recovery_nodes: vec![missing],
            }
        };
        self.finish_formula_case(position, value, cursor)
    }

    fn finish_formula_case(
        &mut self,
        position: usize,
        value: ParsedTypeNode,
        mut cursor: usize,
    ) -> ParsedTypeNode {
        let mut children = vec![value.id];
        let mut recovery_nodes = value.recovery_nodes;

        if self.is_reserved_word_at(cursor, "if") {
            children.push(self.token_node_ids[cursor]);
            cursor += 1;
        } else {
            self.diagnose_malformed_formula_expression(cursor, "expected `if` in formula case");
        }

        if let Some(condition) = self.parse_formula_expression_at(cursor) {
            cursor = condition.next_position;
            children.push(condition.id);
            recovery_nodes.extend(condition.recovery_nodes);
        } else {
            self.diagnose_malformed_formula_expression(cursor, "expected formula after `if`");
            self.push_missing_formula(cursor, &mut children, &mut recovery_nodes);
        }

        let id = self.events.emit(SyntaxEvent::Node {
            kind: SurfaceNodeKind::FormulaCase,
            range: self.covering_token_range(position, cursor.max(position + 1)),
            children,
        });
        ParsedTypeNode {
            id,
            next_position: cursor.max(position + 1),
            recovery_nodes,
        }
    }

    fn finish_definition_content_node(
        &mut self,
        position: usize,
        mut cursor: usize,
        kind: SurfaceNodeKind,
        mut children: Vec<SurfaceBuilderNodeId>,
        mut recovery_nodes: Vec<SurfaceBuilderNodeId>,
        unexpected_message: &'static str,
    ) -> ParsedTypeNode {
        if self.is_semicolon_at(cursor) {
            children.push(self.token_node_ids[cursor]);
            cursor += 1;
        } else if cursor < self.request.tokens.len()
            && !self.is_definition_content_synchronization_boundary_at(cursor)
        {
            self.diagnose_malformed_term_expression(cursor, unexpected_message);
            if let Some(recovery) = self.recover_malformed_definition_content_tail(cursor) {
                cursor = recovery.next_position;
                children.push(recovery.id);
                recovery_nodes.extend(recovery.recovery_nodes);
            }
            if self.is_semicolon_at(cursor) {
                children.push(self.token_node_ids[cursor]);
                cursor += 1;
            } else {
                self.diagnose_missing_semicolon(cursor);
            }
        } else {
            self.diagnose_missing_semicolon(cursor);
        }

        let id = self.events.emit(SyntaxEvent::Node {
            kind,
            range: self.covering_token_range(position, cursor),
            children,
        });
        ParsedTypeNode {
            id,
            next_position: cursor.max(position + 1),
            recovery_nodes,
        }
    }

    fn parse_definition_content_placeholder(&mut self, position: usize) -> ParsedTypeNode {
        let end = self.definition_content_placeholder_end(position);
        let item = self.emit_placeholder_item(position, end);
        ParsedTypeNode {
            id: item.id,
            next_position: item.next_position,
            recovery_nodes: item.recovery_nodes,
        }
    }

    fn parse_reserve_item(&mut self, position: usize) -> ParsedItem {
        let head = self
            .item_head_position(position)
            .expect("reserve parsing starts at an item boundary");
        let mut children = self.token_node_ids[position..=head].to_vec();
        let mut recovery_nodes = Vec::new();
        let mut cursor = head + 1;

        if let Some(segment) = self.parse_reserve_segment(cursor) {
            children.push(segment.id);
            recovery_nodes.extend(segment.recovery_nodes);
            cursor = segment.next_position;
        } else {
            self.diagnose_malformed_type_expression(
                cursor,
                "expected reserved identifier list after `reserve`",
            );
            if let Some(recovery) = self.recover_malformed_tail(cursor) {
                children.push(recovery.id);
                recovery_nodes.extend(recovery.recovery_nodes);
                cursor = recovery.next_position;
            }
        }

        if self.is_semicolon_at(cursor) {
            children.push(self.token_node_ids[cursor]);
            cursor += 1;
        } else if cursor < self.request.tokens.len() && !self.is_item_start_at(cursor) {
            self.diagnose_malformed_type_expression(cursor, "unexpected token in reserve item");
            if let Some(recovery) = self.recover_malformed_tail(cursor) {
                children.push(recovery.id);
                recovery_nodes.extend(recovery.recovery_nodes);
                cursor = recovery.next_position;
            }
            if self.is_semicolon_at(cursor) {
                children.push(self.token_node_ids[cursor]);
                cursor += 1;
            }
        } else {
            self.diagnose_missing_semicolon(cursor);
        }

        let id = self.events.emit(SyntaxEvent::Node {
            kind: SurfaceNodeKind::ReserveItem,
            range: self.covering_token_range(position, cursor),
            children,
        });
        ParsedItem {
            id,
            next_position: cursor.max(position + 1),
            recovery_nodes,
        }
    }

    fn parse_reserve_segment(&mut self, position: usize) -> Option<ParsedItem> {
        if !self.is_identifier_at(position) {
            return None;
        }

        let mut children = vec![self.token_node_ids[position]];
        let mut recovery_nodes = Vec::new();
        let mut cursor = position + 1;
        while self.is_reserved_symbol_at(cursor, ",") {
            children.push(self.token_node_ids[cursor]);
            cursor += 1;
            if self.is_identifier_at(cursor) {
                children.push(self.token_node_ids[cursor]);
                cursor += 1;
            } else {
                self.diagnose_malformed_type_expression(
                    cursor,
                    "expected identifier after `,` in reserve statement",
                );
                if let Some(recovery) = self.recover_malformed_type_tail(cursor) {
                    children.push(recovery.id);
                    recovery_nodes.extend(recovery.recovery_nodes);
                    cursor = recovery.next_position;
                }
                break;
            }
        }

        if self.is_reserved_word_at(cursor, "for") {
            children.push(self.token_node_ids[cursor]);
            cursor += 1;
        } else {
            self.diagnose_malformed_type_expression(
                cursor,
                "expected `for` after reserve identifier list",
            );
            if let Some(recovery) = self.recover_malformed_type_tail(cursor) {
                children.push(recovery.id);
                recovery_nodes.extend(recovery.recovery_nodes);
                cursor = recovery.next_position;
            }
            let id = self.events.emit(SyntaxEvent::Node {
                kind: SurfaceNodeKind::ReserveSegment,
                range: self.covering_token_range(position, cursor),
                children,
            });
            return Some(ParsedItem {
                id,
                next_position: cursor.max(position + 1),
                recovery_nodes,
            });
        }

        match self.parse_type_expression_at(cursor) {
            Some(type_expression) => {
                children.push(type_expression.id);
                recovery_nodes.extend(type_expression.recovery_nodes);
                cursor = type_expression.next_position;
            }
            None if self.is_type_expression_boundary_at(cursor) => {
                self.diagnose_malformed_type_expression(cursor, "expected type expression");
                let recovery = self.add_missing_type_expression(cursor);
                children.push(recovery);
                recovery_nodes.push(recovery);
            }
            None => {
                self.diagnose_malformed_type_expression(cursor, "malformed type expression");
                if let Some(recovery) = self.recover_malformed_type_tail(cursor) {
                    children.push(recovery.id);
                    recovery_nodes.extend(recovery.recovery_nodes);
                    cursor = recovery.next_position;
                }
            }
        }

        let id = self.events.emit(SyntaxEvent::Node {
            kind: SurfaceNodeKind::ReserveSegment,
            range: self.covering_token_range(position, cursor),
            children,
        });
        Some(ParsedItem {
            id,
            next_position: cursor.max(position + 1),
            recovery_nodes,
        })
    }

    fn parse_import_item(&mut self, position: usize) -> ParsedItem {
        let mut children = vec![self.token_node_ids[position]];
        let mut recovery_nodes = Vec::new();
        let mut cursor = position + 1;

        if let Some(decl) = self.parse_import_decl(cursor) {
            children.push(decl.id);
            recovery_nodes.extend(decl.recovery_nodes);
            cursor = decl.next_position;

            while self.is_reserved_symbol_at(cursor, ",") {
                children.push(self.token_node_ids[cursor]);
                cursor += 1;
                match self.parse_import_decl(cursor) {
                    Some(decl) => {
                        children.push(decl.id);
                        recovery_nodes.extend(decl.recovery_nodes);
                        cursor = decl.next_position;
                    }
                    None => {
                        self.diagnose_malformed_import(
                            cursor,
                            "expected module path after `,` in import statement",
                        );
                        if let Some(recovery) = self.recover_malformed_tail(cursor) {
                            children.push(recovery.id);
                            recovery_nodes.extend(recovery.recovery_nodes);
                            cursor = recovery.next_position;
                        }
                        break;
                    }
                }
            }
        } else {
            self.diagnose_malformed_import(
                cursor,
                "expected module path after `import` in import statement",
            );
            if let Some(recovery) = self.recover_malformed_tail(cursor) {
                children.push(recovery.id);
                recovery_nodes.extend(recovery.recovery_nodes);
                cursor = recovery.next_position;
            }
        }

        if self.is_semicolon_at(cursor) {
            children.push(self.token_node_ids[cursor]);
            cursor += 1;
        } else {
            self.diagnose_missing_semicolon(cursor);
        }

        let id = self.events.emit(SyntaxEvent::Node {
            kind: SurfaceNodeKind::ImportItem,
            range: self.covering_token_range(position, cursor),
            children,
        });
        ParsedItem {
            id,
            next_position: cursor.max(position + 1),
            recovery_nodes,
        }
    }

    fn parse_import_decl(&mut self, position: usize) -> Option<ParsedItem> {
        let path = self.parse_module_path_at(position)?;
        if self.is_reserved_symbol_at(path.next_position, ".{") {
            return Some(self.parse_module_branch_import(position, path));
        }

        let mut children = vec![path.id];
        let mut recovery_nodes = Vec::new();
        let mut cursor = path.next_position;
        if self.is_reserved_word_at(cursor, "as") {
            children.push(self.token_node_ids[cursor]);
            cursor += 1;
            if self.is_identifier_at(cursor) {
                children.push(self.emit_wrapped_token(SurfaceNodeKind::PathSegment, cursor));
                cursor += 1;
            } else {
                self.diagnose_malformed_import(
                    cursor,
                    "expected module alias after `as` in import statement",
                );
                if let Some(recovery) = self.recover_malformed_tail(cursor) {
                    children.push(recovery.id);
                    recovery_nodes.extend(recovery.recovery_nodes);
                    cursor = recovery.next_position;
                }
            }
        }

        let id = self.events.emit(SyntaxEvent::Node {
            kind: SurfaceNodeKind::ImportAliasDecl,
            range: self.covering_token_range(position, cursor),
            children,
        });
        Some(ParsedItem {
            id,
            next_position: cursor.max(position + 1),
            recovery_nodes,
        })
    }

    fn parse_module_branch_import(&mut self, position: usize, path: ParsedPathNode) -> ParsedItem {
        let mut children = vec![path.id, self.token_node_ids[path.next_position]];
        let mut recovery_nodes = Vec::new();
        let mut cursor = path.next_position + 1;
        let mut malformed_tail = false;

        loop {
            if self.is_identifier_at(cursor) {
                children.push(self.emit_wrapped_token(SurfaceNodeKind::PathSegment, cursor));
                cursor += 1;
            } else {
                self.diagnose_malformed_import(
                    cursor,
                    "expected module identifier in branch import",
                );
                if let Some(recovery) = self.recover_malformed_tail(cursor) {
                    children.push(recovery.id);
                    recovery_nodes.extend(recovery.recovery_nodes);
                    cursor = recovery.next_position;
                }
                malformed_tail = true;
                break;
            }

            if self.is_reserved_symbol_at(cursor, ",") {
                children.push(self.token_node_ids[cursor]);
                cursor += 1;
                continue;
            }
            break;
        }

        if malformed_tail {
            // A missing branch component already explains the malformed branch;
            // avoid reporting a second import diagnostic at the same boundary.
        } else if self.is_reserved_symbol_at(cursor, "}") {
            children.push(self.token_node_ids[cursor]);
            cursor += 1;
        } else {
            self.diagnose_malformed_import(cursor, "expected `}` to close branch import");
            if let Some(recovery) = self.recover_malformed_tail(cursor) {
                children.push(recovery.id);
                recovery_nodes.extend(recovery.recovery_nodes);
                cursor = recovery.next_position;
            }
        }

        let id = self.events.emit(SyntaxEvent::Node {
            kind: SurfaceNodeKind::ModuleBranchImport,
            range: self.covering_token_range(position, cursor),
            children,
        });
        ParsedItem {
            id,
            next_position: cursor.max(position + 1),
            recovery_nodes,
        }
    }

    fn parse_type_expression_at(&mut self, position: usize) -> Option<ParsedTypeNode> {
        if self.is_type_expression_boundary_at(position) {
            return None;
        }

        let mut attribute_plans = Vec::new();
        let mut cursor = position;
        while let Some(plan) = self.plan_attribute_ref_at(cursor) {
            if !self.can_form_type_expression_at(plan.next_position) {
                break;
            }
            cursor = plan.next_position;
            attribute_plans.push(plan);
        }

        if !self.can_start_type_head_at(cursor) {
            return None;
        }

        let mut children = Vec::new();
        let mut recovery_nodes = Vec::new();
        if !attribute_plans.is_empty() {
            let mut attribute_children = Vec::new();
            for plan in attribute_plans {
                let attribute = self.parse_attribute_ref_from_plan(plan);
                attribute_children.push(attribute.id);
                recovery_nodes.extend(attribute.recovery_nodes);
            }
            let attribute_range = self.covering_token_range(position, cursor);
            let attribute_chain = self.events.emit(SyntaxEvent::Node {
                kind: SurfaceNodeKind::AttributeChain,
                range: attribute_range,
                children: attribute_children,
            });
            children.push(attribute_chain);
        }

        let head = self.parse_type_head_at(cursor)?;
        cursor = head.next_position;
        children.push(head.id);
        recovery_nodes.extend(head.recovery_nodes);

        let id = self.events.emit(SyntaxEvent::Node {
            kind: SurfaceNodeKind::TypeExpression,
            range: self.covering_token_range(position, cursor),
            children,
        });
        Some(ParsedTypeNode {
            id,
            next_position: cursor.max(position + 1),
            recovery_nodes,
        })
    }

    fn parse_attribute_ref_from_plan(&mut self, plan: AttributeRefPlan) -> ParsedTypeNode {
        let mut children = Vec::new();
        let mut recovery_nodes = Vec::new();
        let mut cursor = plan.start_position;

        if self.is_reserved_word_at(cursor, "non") {
            children.push(self.token_node_ids[cursor]);
            cursor += 1;
        }
        if let Some(prefix_end) = self.parameter_prefix_next_at(cursor) {
            let prefix_children = self.token_node_ids[cursor..prefix_end].to_vec();
            let prefix = self.events.emit(SyntaxEvent::Node {
                kind: SurfaceNodeKind::ParameterPrefix,
                range: self.covering_token_range(cursor, prefix_end),
                children: prefix_children,
            });
            children.push(prefix);
            cursor = prefix_end;
        }

        let symbol = self
            .parse_attribute_symbol_at(cursor)
            .expect("planned attribute reference should contain a syntactic attribute symbol");
        children.push(symbol.id);
        cursor = symbol.next_position;

        if self.is_reserved_symbol_at(cursor, "(") {
            let opener = cursor;
            children.push(self.token_node_ids[cursor]);
            cursor += 1;
            cursor = self.parse_term_list_until(
                cursor,
                ")",
                false,
                Some(plan.next_position),
                &mut children,
                &mut recovery_nodes,
            );
            if self.is_reserved_symbol_at(cursor, ")") {
                children.push(self.token_node_ids[cursor]);
                cursor += 1;
            } else {
                self.diagnose_unmatched_term_delimiter(opener, cursor, ")");
                let recovery = self.add_recovery_node(
                    SyntaxRecoveryKind::UnmatchedOpeningDelimiter,
                    self.zero_range_at(cursor),
                    Vec::new(),
                );
                children.push(recovery);
                recovery_nodes.push(recovery);
            }
        }

        let id = self.events.emit(SyntaxEvent::Node {
            kind: SurfaceNodeKind::AttributeRef,
            range: self.covering_token_range(plan.start_position, cursor),
            children,
        });
        ParsedTypeNode {
            id,
            next_position: cursor.max(plan.start_position + 1),
            recovery_nodes,
        }
    }

    fn parse_type_head_at(&mut self, position: usize) -> Option<ParsedTypeNode> {
        let mut children = Vec::new();
        let mut recovery_nodes = Vec::new();
        let mut cursor = position;

        if self.is_builtin_type_word_at(cursor) {
            children.push(self.token_node_ids[cursor]);
            cursor += 1;
        } else {
            let symbol = self.parse_qualified_symbol_at(cursor)?;
            children.push(symbol.id);
            cursor = symbol.next_position;
            if self.is_type_arguments_start_at(cursor) {
                let arguments = self.parse_type_arguments_at(cursor);
                children.push(arguments.id);
                recovery_nodes.extend(arguments.recovery_nodes);
                cursor = arguments.next_position;
            }
        }

        let id = self.events.emit(SyntaxEvent::Node {
            kind: SurfaceNodeKind::TypeHead,
            range: self.covering_token_range(position, cursor),
            children,
        });
        Some(ParsedTypeNode {
            id,
            next_position: cursor.max(position + 1),
            recovery_nodes,
        })
    }

    fn parse_formula_expression_at(&mut self, position: usize) -> Option<ParsedTypeNode> {
        if !self.can_start_formula_at(position)
            || self.formula_payload_contains_deferred_predicate_template_args_at(position)
        {
            return None;
        }
        let formula = self.parse_formula_at(position)?;
        let id = self.events.emit(SyntaxEvent::Node {
            kind: SurfaceNodeKind::FormulaExpression,
            range: self.covering_token_range(position, formula.next_position),
            children: vec![formula.id],
        });
        Some(ParsedTypeNode {
            id,
            next_position: formula.next_position.max(position + 1),
            recovery_nodes: formula.recovery_nodes,
        })
    }

    fn parse_formula_at(&mut self, position: usize) -> Option<ParsedTypeNode> {
        if self.is_formula_expression_boundary_at(position) {
            return None;
        }
        if self.is_formula_quantifier_at(position) {
            return self.parse_quantified_formula_at(position);
        }
        self.parse_formula_binary_at(position, 0)
    }

    fn parse_formula_binary_at(
        &mut self,
        position: usize,
        minimum_binding_power: u32,
    ) -> Option<ParsedTypeNode> {
        let mut left = self.parse_formula_prefix_or_primary_at(position)?;
        loop {
            let cursor = left.next_position;
            let Some(connective) = self.formula_connective_at(cursor) else {
                break;
            };
            if connective.left_binding_power < minimum_binding_power {
                break;
            }

            if connective.connective == SurfaceFormulaConnective::Iff
                && self.left_is_iff_formula_chain(left.id)
            {
                let span = self.request.tokens[cursor].span;
                self.diagnostics.push(SyntaxDiagnostic::new(
                    SyntaxDiagnosticCode::NonAssociativeOperatorChain,
                    "non-associative formula connective chain requires explicit grouping",
                    span,
                ));
            }

            let operator_end = cursor + connective.token_count;
            let mut children = vec![left.id];
            children.extend(self.token_node_ids[cursor..operator_end].iter().copied());
            let mut recovery_nodes = left.recovery_nodes;
            let (right_id, next_position) =
                match self.parse_formula_right_operand_at(operator_end, connective) {
                    Some(right) => {
                        let next_position = right.next_position;
                        recovery_nodes.extend(right.recovery_nodes);
                        (right.id, next_position)
                    }
                    None => {
                        self.diagnose_malformed_formula_expression(
                            operator_end,
                            "expected formula after connective",
                        );
                        let missing = self.add_missing_formula(operator_end);
                        recovery_nodes.push(missing);
                        (missing, operator_end)
                    }
                };
            children.push(right_id);

            let id = self.events.emit(SyntaxEvent::Node {
                kind: SurfaceNodeKind::BinaryFormula(SurfaceFormulaBinaryOperator {
                    connective: connective.connective,
                    repeated: connective.repeated,
                }),
                range: self.covering_token_range(position, next_position.max(operator_end)),
                children,
            });
            left = ParsedTypeNode {
                id,
                next_position: next_position.max(operator_end),
                recovery_nodes,
            };
        }
        Some(left)
    }

    fn parse_formula_right_operand_at(
        &mut self,
        position: usize,
        connective: FormulaConnectiveToken,
    ) -> Option<ParsedTypeNode> {
        if self.is_formula_quantifier_at(position) {
            self.parse_quantified_formula_at(position)
        } else {
            self.parse_formula_binary_at(position, connective.right_binding_power)
        }
    }

    fn parse_formula_prefix_or_primary_at(&mut self, position: usize) -> Option<ParsedTypeNode> {
        if self.is_reserved_word_at(position, "not") {
            return Some(self.parse_prefix_formula_at(position));
        }
        if let Some(parenthesized) = self.parse_parenthesized_formula_at(position) {
            return Some(parenthesized);
        }
        if let Some(constant) = self.parse_formula_constant_at(position) {
            return Some(constant);
        }
        self.parse_atomic_formula_at(position)
    }

    fn parse_prefix_formula_at(&mut self, position: usize) -> ParsedTypeNode {
        let operand_position = position + 1;
        let mut children = vec![self.token_node_ids[position]];
        let mut recovery_nodes = Vec::new();
        let (operand, cursor) = if self.is_formula_quantifier_at(operand_position) {
            match self.parse_quantified_formula_at(operand_position) {
                Some(operand) => {
                    let cursor = operand.next_position;
                    recovery_nodes.extend(operand.recovery_nodes);
                    (operand.id, cursor)
                }
                None => {
                    self.diagnose_malformed_formula_expression(
                        operand_position,
                        "expected formula after `not`",
                    );
                    let missing = self.add_missing_formula(operand_position);
                    recovery_nodes.push(missing);
                    (missing, operand_position)
                }
            }
        } else {
            match self.parse_formula_prefix_or_primary_at(operand_position) {
                Some(operand) => {
                    let cursor = operand.next_position;
                    recovery_nodes.extend(operand.recovery_nodes);
                    (operand.id, cursor)
                }
                None => {
                    self.diagnose_malformed_formula_expression(
                        operand_position,
                        "expected formula after `not`",
                    );
                    let missing = self.add_missing_formula(operand_position);
                    recovery_nodes.push(missing);
                    (missing, operand_position)
                }
            }
        };
        children.push(operand);

        let id = self.events.emit(SyntaxEvent::Node {
            kind: SurfaceNodeKind::PrefixFormula(SurfaceFormulaPrefixOperator::Not),
            range: self.covering_token_range(position, cursor.max(position + 1)),
            children,
        });
        ParsedTypeNode {
            id,
            next_position: cursor.max(position + 1),
            recovery_nodes,
        }
    }

    fn parse_parenthesized_formula_at(&mut self, position: usize) -> Option<ParsedTypeNode> {
        if !self.is_reserved_symbol_at(position, "(") {
            return None;
        }
        let inner_position = position + 1;
        let inner = self.parse_formula_expression_at(inner_position)?;
        let mut children = vec![self.token_node_ids[position], inner.id];
        let mut recovery_nodes = inner.recovery_nodes;
        let mut cursor = inner.next_position;

        if self.is_reserved_symbol_at(cursor, ")") {
            children.push(self.token_node_ids[cursor]);
            cursor += 1;
        } else {
            self.diagnose_unmatched_formula_delimiter(position, cursor, ")");
            let recovery = self.add_recovery_node(
                SyntaxRecoveryKind::UnmatchedOpeningDelimiter,
                self.zero_range_at(cursor),
                Vec::new(),
            );
            children.push(recovery);
            recovery_nodes.push(recovery);
        }

        let id = self.events.emit(SyntaxEvent::Node {
            kind: SurfaceNodeKind::ParenthesizedFormula,
            range: self.covering_token_range(position, cursor),
            children,
        });
        Some(ParsedTypeNode {
            id,
            next_position: cursor.max(position + 1),
            recovery_nodes,
        })
    }

    fn parse_formula_constant_at(&mut self, position: usize) -> Option<ParsedTypeNode> {
        let constant = if self.is_reserved_word_at(position, "thesis") {
            SurfaceFormulaConstant::Thesis
        } else if self.is_reserved_word_at(position, "contradiction") {
            SurfaceFormulaConstant::Contradiction
        } else {
            return None;
        };
        let cursor = position + 1;
        let id = self.events.emit(SyntaxEvent::Node {
            kind: SurfaceNodeKind::FormulaConstant(constant),
            range: self.covering_token_range(position, cursor),
            children: vec![self.token_node_ids[position]],
        });
        Some(ParsedTypeNode {
            id,
            next_position: cursor,
            recovery_nodes: Vec::new(),
        })
    }

    fn parse_quantified_formula_at(&mut self, position: usize) -> Option<ParsedTypeNode> {
        let quantifier = if self.is_reserved_word_at(position, "for") {
            SurfaceQuantifierKind::Universal
        } else if self.is_reserved_word_at(position, "ex") {
            SurfaceQuantifierKind::Existential
        } else {
            return None;
        };
        let mut children = vec![self.token_node_ids[position]];
        let mut recovery_nodes = Vec::new();
        let mut cursor = position + 1;

        let first_segment = self.parse_quantifier_variable_segment_at(cursor)?;
        cursor = first_segment.next_position;
        children.push(first_segment.id);
        recovery_nodes.extend(first_segment.recovery_nodes);

        while self.is_reserved_symbol_at(cursor, ",") {
            children.push(self.token_node_ids[cursor]);
            cursor += 1;
            if let Some(segment) = self.parse_quantifier_variable_segment_at(cursor) {
                cursor = segment.next_position;
                children.push(segment.id);
                recovery_nodes.extend(segment.recovery_nodes);
            } else {
                self.diagnose_malformed_formula_expression(
                    cursor,
                    "expected quantified variable segment after `,`",
                );
                break;
            }
        }

        match quantifier {
            SurfaceQuantifierKind::Universal => {
                self.parse_universal_formula_tail(&mut cursor, &mut children, &mut recovery_nodes);
            }
            SurfaceQuantifierKind::Existential => {
                self.parse_existential_formula_tail(
                    &mut cursor,
                    &mut children,
                    &mut recovery_nodes,
                );
            }
        }

        let id = self.events.emit(SyntaxEvent::Node {
            kind: SurfaceNodeKind::QuantifiedFormula(quantifier),
            range: self.covering_token_range(position, cursor.max(position + 1)),
            children,
        });
        Some(ParsedTypeNode {
            id,
            next_position: cursor.max(position + 1),
            recovery_nodes,
        })
    }

    fn parse_universal_formula_tail(
        &mut self,
        cursor: &mut usize,
        children: &mut Vec<SurfaceBuilderNodeId>,
        recovery_nodes: &mut Vec<SurfaceBuilderNodeId>,
    ) {
        if self.is_reserved_word_at(*cursor, "st") {
            children.push(self.token_node_ids[*cursor]);
            *cursor += 1;
            self.parse_required_formula_child(
                cursor,
                children,
                recovery_nodes,
                "expected formula after `st`",
            );
        }

        if self.is_reserved_word_at(*cursor, "holds") {
            children.push(self.token_node_ids[*cursor]);
            *cursor += 1;
            self.parse_required_formula_child(
                cursor,
                children,
                recovery_nodes,
                "expected formula after `holds`",
            );
        } else if self.is_formula_quantifier_at(*cursor) {
            if let Some(body) = self.parse_quantified_formula_at(*cursor) {
                *cursor = body.next_position;
                children.push(body.id);
                recovery_nodes.extend(body.recovery_nodes);
            }
        } else {
            self.diagnose_malformed_formula_expression(
                *cursor,
                "expected `holds` formula or nested quantified formula",
            );
            let missing = self.add_missing_formula(*cursor);
            children.push(missing);
            recovery_nodes.push(missing);
            self.consume_malformed_quantifier_tail_formula(cursor, children, recovery_nodes);
        }
    }

    fn parse_existential_formula_tail(
        &mut self,
        cursor: &mut usize,
        children: &mut Vec<SurfaceBuilderNodeId>,
        recovery_nodes: &mut Vec<SurfaceBuilderNodeId>,
    ) {
        if self.is_reserved_word_at(*cursor, "st") {
            children.push(self.token_node_ids[*cursor]);
            *cursor += 1;
            self.parse_required_formula_child(
                cursor,
                children,
                recovery_nodes,
                "expected formula after `st`",
            );
        } else {
            self.diagnose_malformed_formula_expression(
                *cursor,
                "expected `st` formula in existential quantifier",
            );
            let missing = self.add_missing_formula(*cursor);
            children.push(missing);
            recovery_nodes.push(missing);
            self.consume_malformed_quantifier_tail_formula(cursor, children, recovery_nodes);
        }
    }

    fn consume_malformed_quantifier_tail_formula(
        &mut self,
        cursor: &mut usize,
        children: &mut Vec<SurfaceBuilderNodeId>,
        recovery_nodes: &mut Vec<SurfaceBuilderNodeId>,
    ) {
        if self.can_start_formula_at(*cursor)
            && let Some(formula) = self.parse_formula_at(*cursor)
        {
            *cursor = formula.next_position;
            children.push(formula.id);
            recovery_nodes.extend(formula.recovery_nodes);
        }
    }

    fn parse_required_formula_child(
        &mut self,
        cursor: &mut usize,
        children: &mut Vec<SurfaceBuilderNodeId>,
        recovery_nodes: &mut Vec<SurfaceBuilderNodeId>,
        message: &'static str,
    ) {
        if self.can_start_formula_at(*cursor)
            && let Some(formula) = self.parse_formula_at(*cursor)
        {
            *cursor = formula.next_position;
            children.push(formula.id);
            recovery_nodes.extend(formula.recovery_nodes);
        } else {
            self.diagnose_malformed_formula_expression(*cursor, message);
            let missing = self.add_missing_formula(*cursor);
            children.push(missing);
            recovery_nodes.push(missing);
        }
    }

    fn parse_quantifier_variable_segment_at(&mut self, position: usize) -> Option<ParsedTypeNode> {
        if !self.is_identifier_at(position) {
            return None;
        }
        let mut children = vec![self.token_node_ids[position]];
        let mut recovery_nodes = Vec::new();
        let mut cursor = position + 1;

        while self.is_reserved_symbol_at(cursor, ",") && self.is_identifier_at(cursor + 1) {
            children.push(self.token_node_ids[cursor]);
            children.push(self.token_node_ids[cursor + 1]);
            cursor += 2;
        }

        if self.is_reserved_word_at(cursor, "be") || self.is_reserved_word_at(cursor, "being") {
            children.push(self.token_node_ids[cursor]);
            cursor += 1;
            if let Some(type_expression) = self.parse_type_expression_at(cursor) {
                cursor = type_expression.next_position;
                children.push(type_expression.id);
                recovery_nodes.extend(type_expression.recovery_nodes);
            } else {
                self.diagnose_malformed_type_expression(
                    cursor,
                    "expected type after quantified variable binder",
                );
                let missing = self.add_missing_type_expression(cursor);
                children.push(missing);
                recovery_nodes.push(missing);
            }
        }

        let id = self.events.emit(SyntaxEvent::Node {
            kind: SurfaceNodeKind::QuantifierVariableSegment,
            range: self.covering_token_range(position, cursor),
            children,
        });
        Some(ParsedTypeNode {
            id,
            next_position: cursor.max(position + 1),
            recovery_nodes,
        })
    }

    fn parse_atomic_formula_at(&mut self, position: usize) -> Option<ParsedTypeNode> {
        if self.should_try_head_first_predicate_at(position)
            && let Some(predicate) = self.parse_user_predicate_application_at(position)
        {
            return Some(predicate);
        }
        if self.should_try_inline_predicate_at(position) {
            return self.parse_inline_predicate_application_at(position);
        }

        let left = self.parse_term_expression_at(position)?;
        let cursor = left.next_position;
        if self.is_builtin_predicate_at(cursor) {
            return Some(
                self.parse_builtin_predicate_application_after_left(position, left, cursor),
            );
        }
        if self.is_reserved_word_at(cursor, "is") {
            return Some(self.parse_is_assertion_after_subject(position, left, cursor));
        }
        if self.is_reserved_symbol_at(cursor, ",") || self.can_start_predicate_tail_at(cursor) {
            return self.parse_user_predicate_application_after_left(position, left);
        }
        self.parse_inline_predicate_application_at(position)
    }

    fn parse_builtin_predicate_application_after_left(
        &mut self,
        start_position: usize,
        left: ParsedTypeNode,
        predicate_position: usize,
    ) -> ParsedTypeNode {
        let mut children = vec![left.id, self.token_node_ids[predicate_position]];
        let mut recovery_nodes = left.recovery_nodes;
        let mut cursor = predicate_position + 1;

        if let Some(right) = self.parse_term_expression_at(cursor) {
            children.push(right.id);
            recovery_nodes.extend(right.recovery_nodes);
            cursor = right.next_position;
        } else {
            self.diagnose_malformed_term_expression(
                cursor,
                "expected term after built-in predicate",
            );
            self.push_missing_term(cursor, &mut children, &mut recovery_nodes);
        }

        let id = self.events.emit(SyntaxEvent::Node {
            kind: SurfaceNodeKind::BuiltinPredicateApplication,
            range: self.covering_token_range(start_position, cursor.max(predicate_position + 1)),
            children,
        });
        ParsedTypeNode {
            id,
            next_position: cursor.max(predicate_position + 1),
            recovery_nodes,
        }
    }

    fn parse_is_assertion_after_subject(
        &mut self,
        start_position: usize,
        subject: ParsedTypeNode,
        is_position: usize,
    ) -> ParsedTypeNode {
        let mut children = vec![subject.id, self.token_node_ids[is_position]];
        let mut recovery_nodes = subject.recovery_nodes;
        let mut cursor = is_position + 1;

        if self.is_reserved_word_at(cursor, "not") {
            children.push(self.token_node_ids[cursor]);
            cursor += 1;
        }

        if self.should_parse_bare_attribute_test_body_at(cursor)
            && let Some(body) = self.parse_attribute_test_chain_at(cursor)
        {
            children.push(body.id);
            recovery_nodes.extend(body.recovery_nodes);
            cursor = body.next_position;
        } else if let Some(body) = self.parse_type_expression_at(cursor) {
            children.push(body.id);
            recovery_nodes.extend(body.recovery_nodes);
            cursor = body.next_position;
        } else if let Some(body) = self.parse_attribute_test_chain_at(cursor) {
            children.push(body.id);
            recovery_nodes.extend(body.recovery_nodes);
            cursor = body.next_position;
        } else {
            self.diagnose_malformed_type_expression(
                cursor,
                "expected type or attribute assertion body after `is`",
            );
            let missing = self.add_missing_type_expression(cursor);
            children.push(missing);
            recovery_nodes.push(missing);
        }

        let id = self.events.emit(SyntaxEvent::Node {
            kind: SurfaceNodeKind::IsAssertion,
            range: self.covering_token_range(start_position, cursor.max(is_position + 1)),
            children,
        });
        ParsedTypeNode {
            id,
            next_position: cursor.max(is_position + 1),
            recovery_nodes,
        }
    }

    fn parse_attribute_test_chain_at(&mut self, position: usize) -> Option<ParsedTypeNode> {
        if self.is_formula_expression_boundary_at(position) {
            return None;
        }
        let mut cursor = position;
        let mut children = Vec::new();
        let mut recovery_nodes = Vec::new();

        while let Some(plan) = self.plan_attribute_ref_at(cursor) {
            let attribute = self.parse_attribute_ref_from_plan(plan);
            children.push(attribute.id);
            recovery_nodes.extend(attribute.recovery_nodes);
            cursor = attribute.next_position;
            if self.is_formula_expression_boundary_at(cursor) {
                break;
            }
        }

        if children.is_empty() {
            return None;
        }
        let id = self.events.emit(SyntaxEvent::Node {
            kind: SurfaceNodeKind::AttributeTestChain,
            range: self.covering_token_range(position, cursor),
            children,
        });
        Some(ParsedTypeNode {
            id,
            next_position: cursor.max(position + 1),
            recovery_nodes,
        })
    }

    fn parse_inline_predicate_application_at(&mut self, position: usize) -> Option<ParsedTypeNode> {
        if !self.is_identifier_at(position) || !self.is_reserved_symbol_at(position + 1, "(") {
            return None;
        }
        let mut children = vec![
            self.token_node_ids[position],
            self.token_node_ids[position + 1],
        ];
        let mut recovery_nodes = Vec::new();
        let opener = position + 1;
        let mut cursor = self.parse_term_list_until(
            opener + 1,
            ")",
            true,
            None,
            &mut children,
            &mut recovery_nodes,
        );
        if self.is_reserved_symbol_at(cursor, ")") {
            children.push(self.token_node_ids[cursor]);
            cursor += 1;
        } else {
            self.diagnose_unmatched_term_delimiter(opener, cursor, ")");
            let recovery = self.add_recovery_node(
                SyntaxRecoveryKind::UnmatchedOpeningDelimiter,
                self.zero_range_at(cursor),
                Vec::new(),
            );
            children.push(recovery);
            recovery_nodes.push(recovery);
        }

        let id = self.events.emit(SyntaxEvent::Node {
            kind: SurfaceNodeKind::InlinePredicateApplication,
            range: self.covering_token_range(position, cursor),
            children,
        });
        Some(ParsedTypeNode {
            id,
            next_position: cursor.max(position + 1),
            recovery_nodes,
        })
    }

    fn parse_user_predicate_application_at(&mut self, position: usize) -> Option<ParsedTypeNode> {
        let first_segment = self.parse_predicate_segment_from_parts(
            position,
            Vec::new(),
            Vec::new(),
            position,
            false,
        )?;
        Some(self.parse_predicate_application_from_first_segment(position, first_segment))
    }

    fn parse_user_predicate_application_after_left(
        &mut self,
        position: usize,
        left: ParsedTypeNode,
    ) -> Option<ParsedTypeNode> {
        let mut children = vec![left.id];
        let mut recovery_nodes = left.recovery_nodes;
        let mut cursor = left.next_position;

        while self.is_reserved_symbol_at(cursor, ",") {
            children.push(self.token_node_ids[cursor]);
            cursor += 1;
            if let Some(term) = self.parse_term_expression_at(cursor) {
                children.push(term.id);
                recovery_nodes.extend(term.recovery_nodes);
                cursor = term.next_position;
            } else {
                self.diagnose_malformed_term_expression(
                    cursor,
                    "expected term after `,` in predicate argument list",
                );
                self.push_missing_term(cursor, &mut children, &mut recovery_nodes);
                break;
            }
        }

        let first_segment = self.parse_predicate_segment_from_parts(
            position,
            children,
            recovery_nodes,
            cursor,
            false,
        )?;
        Some(self.parse_predicate_application_from_first_segment(position, first_segment))
    }

    fn parse_predicate_application_from_first_segment(
        &mut self,
        position: usize,
        first_segment: ParsedTypeNode,
    ) -> ParsedTypeNode {
        let mut children = vec![first_segment.id];
        let mut recovery_nodes = first_segment.recovery_nodes;
        let mut cursor = first_segment.next_position;

        while self.can_start_predicate_tail_at(cursor) {
            let Some(segment) = self.parse_predicate_segment_from_parts(
                cursor,
                Vec::new(),
                Vec::new(),
                cursor,
                true,
            ) else {
                break;
            };
            cursor = segment.next_position;
            recovery_nodes.extend(segment.recovery_nodes);
            children.push(segment.id);
        }

        let id = self.events.emit(SyntaxEvent::Node {
            kind: SurfaceNodeKind::PredicateApplication,
            range: self.covering_token_range(position, cursor),
            children,
        });
        ParsedTypeNode {
            id,
            next_position: cursor.max(position + 1),
            recovery_nodes,
        }
    }

    fn parse_predicate_segment_from_parts(
        &mut self,
        start_position: usize,
        mut children: Vec<SurfaceBuilderNodeId>,
        mut recovery_nodes: Vec<SurfaceBuilderNodeId>,
        mut cursor: usize,
        require_right_terms: bool,
    ) -> Option<ParsedTypeNode> {
        if let Some(negation_end) = self.predicate_negation_end_at(cursor) {
            children.extend_from_slice(&self.token_node_ids[cursor..negation_end]);
            cursor = negation_end;
        }

        let head_end = self.qualified_symbol_next_at(cursor)?;
        if self.is_reserved_symbol_at(head_end, "[") {
            return None;
        }
        let head = self
            .parse_predicate_head_at(cursor)
            .expect("qualified predicate head lookahead should parse");
        children.push(head.id);
        recovery_nodes.extend(head.recovery_nodes);
        cursor = head.next_position;
        let right_child_start = children.len();
        cursor =
            self.parse_optional_predicate_term_list_at(cursor, &mut children, &mut recovery_nodes);
        if require_right_terms && children.len() == right_child_start {
            self.diagnose_malformed_term_expression(
                cursor,
                "expected term after predicate-chain head",
            );
            self.push_missing_term(cursor, &mut children, &mut recovery_nodes);
        }

        let id = self.events.emit(SyntaxEvent::Node {
            kind: SurfaceNodeKind::PredicateSegment,
            range: self.covering_token_range(start_position, cursor),
            children,
        });
        Some(ParsedTypeNode {
            id,
            next_position: cursor.max(start_position + 1),
            recovery_nodes,
        })
    }

    fn parse_predicate_head_at(&mut self, position: usize) -> Option<ParsedTypeNode> {
        let symbol = self.parse_qualified_symbol_at(position)?;
        let id = self.events.emit(SyntaxEvent::Node {
            kind: SurfaceNodeKind::PredicateHead,
            range: self.covering_token_range(position, symbol.next_position),
            children: vec![symbol.id],
        });
        Some(ParsedTypeNode {
            id,
            next_position: symbol.next_position.max(position + 1),
            recovery_nodes: Vec::new(),
        })
    }

    fn parse_optional_predicate_term_list_at(
        &mut self,
        mut cursor: usize,
        children: &mut Vec<SurfaceBuilderNodeId>,
        recovery_nodes: &mut Vec<SurfaceBuilderNodeId>,
    ) -> usize {
        if !self.can_start_formula_term_at(cursor) {
            return cursor;
        }

        loop {
            let Some(term) = self.parse_term_expression_at(cursor) else {
                self.diagnose_malformed_term_expression(
                    cursor,
                    "expected term in predicate argument list",
                );
                self.push_missing_term(cursor, children, recovery_nodes);
                break;
            };
            children.push(term.id);
            recovery_nodes.extend(term.recovery_nodes);
            cursor = term.next_position;

            if !self.is_reserved_symbol_at(cursor, ",") {
                break;
            }
            children.push(self.token_node_ids[cursor]);
            cursor += 1;
            if !self.can_start_formula_term_at(cursor) {
                self.diagnose_malformed_term_expression(
                    cursor,
                    "expected term after `,` in predicate argument list",
                );
                self.push_missing_term(cursor, children, recovery_nodes);
                break;
            }
        }

        cursor
    }

    fn parse_type_arguments_at(&mut self, position: usize) -> ParsedTypeNode {
        if self.is_reserved_word_at(position, "of") || self.is_reserved_word_at(position, "over") {
            return self.parse_of_over_type_arguments_at(position);
        }
        self.parse_bracket_type_arguments_at(position)
    }

    fn parse_type_arguments_for_structure_constructor_at(
        &mut self,
        position: usize,
    ) -> ParsedTypeNode {
        if self.is_reserved_word_at(position, "of") || self.is_reserved_word_at(position, "over") {
            return self.parse_of_over_type_arguments_before_structure_fields_at(position);
        }
        self.parse_bracket_type_arguments_at(position)
    }

    fn parse_of_over_type_arguments_at(&mut self, position: usize) -> ParsedTypeNode {
        self.parse_of_over_type_arguments_at_with_options(position, false)
    }

    fn parse_of_over_type_arguments_before_structure_fields_at(
        &mut self,
        position: usize,
    ) -> ParsedTypeNode {
        self.parse_of_over_type_arguments_at_with_options(position, true)
    }

    fn parse_of_over_type_arguments_at_with_options(
        &mut self,
        position: usize,
        stop_before_field_list: bool,
    ) -> ParsedTypeNode {
        let mut children = vec![self.token_node_ids[position]];
        let mut recovery_nodes = Vec::new();
        let mut cursor = position + 1;
        let mut expecting_argument = true;
        let mut saw_argument = false;
        while cursor < self.request.tokens.len() && !self.is_term_argument_list_boundary_at(cursor)
        {
            if stop_before_field_list && self.is_structure_field_list_opener_at(cursor) {
                if expecting_argument {
                    self.diagnose_malformed_term_expression(
                        cursor,
                        if saw_argument {
                            "expected term after `,`"
                        } else {
                            "expected term after type-argument keyword"
                        },
                    );
                    self.push_missing_term(cursor, &mut children, &mut recovery_nodes);
                }
                break;
            }
            if self.is_reserved_symbol_at(cursor, ",") {
                if expecting_argument {
                    self.diagnose_malformed_term_expression(cursor, "expected term before `,`");
                    self.push_missing_term(cursor, &mut children, &mut recovery_nodes);
                }
                children.push(self.token_node_ids[cursor]);
                cursor += 1;
                expecting_argument = true;
                continue;
            }
            let term = if stop_before_field_list {
                self.parse_term_expression_before_structure_field_list_at(cursor)
            } else {
                self.parse_term_expression_at(cursor)
            };
            if let Some(term) = term {
                children.push(term.id);
                recovery_nodes.extend(term.recovery_nodes);
                cursor = term.next_position;
                expecting_argument = false;
                saw_argument = true;
                if self.is_reserved_symbol_at(cursor, ",") {
                    children.push(self.token_node_ids[cursor]);
                    cursor += 1;
                    expecting_argument = true;
                } else if stop_before_field_list && self.is_structure_field_list_opener_at(cursor) {
                    break;
                } else if !self.is_term_argument_list_boundary_at(cursor) {
                    self.diagnose_malformed_term_expression(
                        cursor,
                        "expected `,` between term arguments",
                    );
                    if let Some(recovery) = self.recover_malformed_term_tail(cursor, "") {
                        children.push(recovery.id);
                        recovery_nodes.extend(recovery.recovery_nodes);
                        cursor = recovery.next_position;
                    }
                } else {
                    break;
                }
            } else {
                self.diagnose_malformed_term_expression(
                    cursor,
                    "expected term after type-argument keyword",
                );
                self.push_missing_term(cursor, &mut children, &mut recovery_nodes);
                if let Some(recovery) = self.recover_malformed_term_tail(cursor, "") {
                    children.push(recovery.id);
                    recovery_nodes.extend(recovery.recovery_nodes);
                    cursor = recovery.next_position;
                }
                break;
            }
        }
        if expecting_argument {
            self.diagnose_malformed_term_expression(
                cursor,
                if saw_argument {
                    "expected term after `,`"
                } else {
                    "expected term after type-argument keyword"
                },
            );
            self.push_missing_term(cursor, &mut children, &mut recovery_nodes);
        }
        let id = self.events.emit(SyntaxEvent::Node {
            kind: SurfaceNodeKind::TypeArguments,
            range: self.covering_token_range(position, cursor),
            children,
        });
        ParsedTypeNode {
            id,
            next_position: cursor.max(position + 1),
            recovery_nodes,
        }
    }

    fn parse_bracket_type_arguments_at(&mut self, position: usize) -> ParsedTypeNode {
        let mut children = vec![self.token_node_ids[position]];
        let mut recovery_nodes = Vec::new();
        let mut cursor = position + 1;
        let mut expecting_argument = true;

        while cursor < self.request.tokens.len() && !self.is_reserved_symbol_at(cursor, "]") {
            if self.is_type_expression_boundary_at(cursor) {
                break;
            }
            if self.is_reserved_symbol_at(cursor, ",") {
                self.diagnose_malformed_type_expression(
                    cursor,
                    "expected type argument before `,`",
                );
                let recovery = self.add_missing_type_expression(cursor);
                children.push(recovery);
                recovery_nodes.push(recovery);
                children.push(self.token_node_ids[cursor]);
                cursor += 1;
                expecting_argument = true;
                continue;
            }

            if let Some(type_argument) = self.parse_type_expression_at(cursor) {
                children.push(type_argument.id);
                recovery_nodes.extend(type_argument.recovery_nodes);
                cursor = type_argument.next_position;
                expecting_argument = false;
            } else if let Some(term) = self.parse_bracket_qua_argument_at(cursor) {
                children.push(term.id);
                recovery_nodes.extend(term.recovery_nodes);
                cursor = term.next_position;
                expecting_argument = false;
            } else {
                self.diagnose_malformed_type_expression(cursor, "malformed type argument");
                if let Some(recovery) = self.recover_malformed_type_tail(cursor) {
                    children.push(recovery.id);
                    recovery_nodes.extend(recovery.recovery_nodes);
                    cursor = recovery.next_position;
                }
                break;
            }

            if self.is_reserved_symbol_at(cursor, ",") {
                children.push(self.token_node_ids[cursor]);
                cursor += 1;
                expecting_argument = true;
            } else {
                break;
            }
        }

        if self.is_reserved_symbol_at(cursor, "]") {
            if expecting_argument {
                self.diagnose_malformed_type_expression(
                    cursor,
                    "expected type argument before `]`",
                );
                let recovery = self.add_missing_type_expression(cursor);
                children.push(recovery);
                recovery_nodes.push(recovery);
            }
            children.push(self.token_node_ids[cursor]);
            cursor += 1;
        } else {
            if expecting_argument {
                let recovery = self.add_missing_type_expression(cursor);
                children.push(recovery);
                recovery_nodes.push(recovery);
            }
            self.diagnose_unmatched_type_argument_opener(position, cursor);
            let recovery = self.add_recovery_node(
                SyntaxRecoveryKind::UnmatchedOpeningDelimiter,
                self.zero_range_at(cursor),
                Vec::new(),
            );
            children.push(recovery);
            recovery_nodes.push(recovery);
        }

        let id = self.events.emit(SyntaxEvent::Node {
            kind: SurfaceNodeKind::TypeArguments,
            range: self.covering_token_range(position, cursor),
            children,
        });
        ParsedTypeNode {
            id,
            next_position: cursor.max(position + 1),
            recovery_nodes,
        }
    }

    fn parse_term_expression_at(&mut self, position: usize) -> Option<ParsedTypeNode> {
        if self.is_term_expression_boundary_at(position) {
            return None;
        }

        let operator = self.parse_term_operator_expression_at(position, 0, false)?;
        let term = self.parse_qua_chain_at(position, operator, QuaTargetGrammar::TypeExpression);
        Some(self.wrap_term_expression(position, term))
    }

    fn parse_term_expression_before_structure_field_list_at(
        &mut self,
        position: usize,
    ) -> Option<ParsedTypeNode> {
        if self.is_identifier_at(position) && self.is_structure_field_list_opener_at(position + 1) {
            let primary = self.parse_identifier_reference_term_at(position);
            let postfix =
                self.parse_term_postfix_chain_before_structure_field_list_at(position, primary);
            let operator = self.parse_term_operator_tail_at(position, postfix, 0, true);
            let term =
                self.parse_qua_chain_at(position, operator, QuaTargetGrammar::TypeExpression);
            return Some(self.wrap_term_expression(position, term));
        }

        if let Some(symbol_end) = self.qualified_symbol_next_at(position)
            && self.is_structure_field_list_opener_at(symbol_end)
        {
            let primary = self.parse_qualified_symbol_reference_term_at(position)?;
            let postfix =
                self.parse_term_postfix_chain_before_structure_field_list_at(position, primary);
            let operator = self.parse_term_operator_tail_at(position, postfix, 0, true);
            let term =
                self.parse_qua_chain_at(position, operator, QuaTargetGrammar::TypeExpression);
            return Some(self.wrap_term_expression(position, term));
        }

        if self.is_term_expression_boundary_at(position) {
            return None;
        }
        let operator = self.parse_term_operator_expression_at(position, 0, true)?;
        let term = self.parse_qua_chain_at(position, operator, QuaTargetGrammar::TypeExpression);
        Some(self.wrap_term_expression(position, term))
    }

    fn wrap_term_expression(&mut self, position: usize, primary: ParsedTypeNode) -> ParsedTypeNode {
        let id = self.events.emit(SyntaxEvent::Node {
            kind: SurfaceNodeKind::TermExpression,
            range: self.covering_token_range(position, primary.next_position),
            children: vec![primary.id],
        });
        ParsedTypeNode {
            id,
            next_position: primary.next_position.max(position + 1),
            recovery_nodes: primary.recovery_nodes,
        }
    }

    fn parse_term_operator_expression_at(
        &mut self,
        position: usize,
        minimum_binding_power: u32,
        stop_before_structure_field_list: bool,
    ) -> Option<ParsedTypeNode> {
        let left = if let Some(operator) = self.prefix_operator_at(position).cloned() {
            self.parse_prefix_operator_expression_at(
                position,
                operator,
                stop_before_structure_field_list,
            )
        } else {
            let primary = self.parse_term_primary_at(position)?;
            self.parse_term_postfix_chain(position, primary, stop_before_structure_field_list)
        };

        Some(self.parse_term_operator_tail_at(
            position,
            left,
            minimum_binding_power,
            stop_before_structure_field_list,
        ))
    }

    fn parse_term_operator_tail_at(
        &mut self,
        start_position: usize,
        mut left: ParsedTypeNode,
        minimum_binding_power: u32,
        stop_before_structure_field_list: bool,
    ) -> ParsedTypeNode {
        loop {
            let cursor = left.next_position;
            if stop_before_structure_field_list && self.is_structure_field_list_opener_at(cursor) {
                break;
            }

            let postfix_operator = self.postfix_operator_at(cursor).cloned();
            let postfix_eligible = postfix_operator
                .as_ref()
                .is_some_and(|operator| u32::from(operator.precedence) >= minimum_binding_power);

            let Some(infix_operator) = self.infix_operator_at(cursor).cloned() else {
                if let Some(operator) = postfix_operator.filter(|_| postfix_eligible) {
                    left = self.parse_postfix_operator_expression_after_base(
                        start_position,
                        left,
                        cursor,
                        &operator,
                    );
                    continue;
                }
                break;
            };
            let (left_binding_power, right_binding_power) = infix_binding_powers(&infix_operator);
            if left_binding_power < minimum_binding_power {
                if let Some(operator) = postfix_operator.filter(|_| postfix_eligible) {
                    left = self.parse_postfix_operator_expression_after_base(
                        start_position,
                        left,
                        cursor,
                        &operator,
                    );
                    continue;
                }
                break;
            }

            if postfix_eligible
                && !self.can_start_term_operator_operand_at(
                    cursor + 1,
                    stop_before_structure_field_list,
                )
            {
                let operator = postfix_operator.expect("postfix eligibility implies operator");
                left = self.parse_postfix_operator_expression_after_base(
                    start_position,
                    left,
                    cursor,
                    &operator,
                );
                continue;
            }

            if infix_associativity(&infix_operator) == OperatorAssociativity::NonAssociative
                && self.left_is_non_associative_chain(left.id, &infix_operator)
            {
                let span = self.request.tokens[cursor].span;
                self.diagnostics.push(SyntaxDiagnostic::new(
                    SyntaxDiagnosticCode::NonAssociativeOperatorChain,
                    "non-associative operator chain requires explicit grouping",
                    span,
                ));
            }

            let Some(right) = self.parse_term_operator_expression_at(
                cursor + 1,
                right_binding_power,
                stop_before_structure_field_list,
            ) else {
                let span = self.request.tokens[cursor].span;
                self.diagnostics.push(SyntaxDiagnostic::new(
                    SyntaxDiagnosticCode::DanglingOperator,
                    "operator has no right operand",
                    span,
                ));
                left.next_position = cursor + 1;
                break;
            };

            left = self.parse_infix_operator_expression_after_base(
                start_position,
                left,
                cursor,
                right,
                &infix_operator,
            );
        }
        left
    }

    fn can_start_term_operator_operand_at(
        &self,
        position: usize,
        stop_before_structure_field_list: bool,
    ) -> bool {
        if self.is_term_expression_boundary_at(position)
            || (stop_before_structure_field_list
                && self.is_structure_field_list_opener_at(position))
        {
            return false;
        }

        self.prefix_operator_at(position).is_some()
            || self.qualified_symbol_next_at(position).is_some()
            || self.is_identifier_at(position)
            || self
                .request
                .tokens
                .get(position)
                .is_some_and(|token| token.kind == ParserTokenKind::Numeral)
            || self.is_reserved_word_at(position, "it")
            || self.is_reserved_word_at(position, "the")
            || self.is_reserved_symbol_at(position, "(")
            || self.is_reserved_symbol_at(position, "[")
            || self.is_reserved_symbol_at(position, "{")
    }

    fn parse_prefix_operator_expression_at(
        &mut self,
        position: usize,
        operator: OperatorFixityEntry,
        stop_before_structure_field_list: bool,
    ) -> ParsedTypeNode {
        let operand_position = position + 1;
        let mut children = vec![self.token_node_ids[position]];
        let mut recovery_nodes = Vec::new();
        let (operand, cursor) = match self.parse_term_operator_expression_at(
            operand_position,
            u32::from(operator.precedence),
            stop_before_structure_field_list,
        ) {
            Some(operand) => {
                let cursor = operand.next_position;
                let operand_id = operand.id;
                recovery_nodes.extend(operand.recovery_nodes);
                (operand_id, cursor)
            }
            None => {
                let span = self.request.tokens[position].span;
                self.diagnostics.push(SyntaxDiagnostic::new(
                    SyntaxDiagnosticCode::DanglingOperator,
                    "operator has no operand",
                    span,
                ));
                let missing = self.add_missing_term(operand_position);
                recovery_nodes.push(missing);
                (missing, operand_position)
            }
        };
        children.push(operand);
        let id = self.events.emit(SyntaxEvent::Node {
            kind: SurfaceNodeKind::PrefixExpression(SurfacePrefixOperator {
                spelling: operator.spelling,
                precedence: operator.precedence,
            }),
            range: self.covering_token_range(position, cursor.max(position + 1)),
            children,
        });
        ParsedTypeNode {
            id,
            next_position: cursor.max(position + 1),
            recovery_nodes,
        }
    }

    fn parse_postfix_operator_expression_after_base(
        &mut self,
        start_position: usize,
        base: ParsedTypeNode,
        operator_position: usize,
        operator: &OperatorFixityEntry,
    ) -> ParsedTypeNode {
        let recovery_nodes = base.recovery_nodes;
        let cursor = operator_position + 1;
        let id = self.events.emit(SyntaxEvent::Node {
            kind: SurfaceNodeKind::PostfixExpression(SurfacePostfixOperator {
                spelling: operator.spelling.clone(),
                precedence: operator.precedence,
            }),
            range: self.covering_token_range(start_position, cursor),
            children: vec![base.id, self.token_node_ids[operator_position]],
        });
        ParsedTypeNode {
            id,
            next_position: cursor,
            recovery_nodes,
        }
    }

    fn parse_infix_operator_expression_after_base(
        &mut self,
        start_position: usize,
        left: ParsedTypeNode,
        operator_position: usize,
        right: ParsedTypeNode,
        operator: &OperatorFixityEntry,
    ) -> ParsedTypeNode {
        let mut recovery_nodes = left.recovery_nodes;
        recovery_nodes.extend(right.recovery_nodes);
        let cursor = right.next_position;
        let id = self.events.emit(SyntaxEvent::Node {
            kind: SurfaceNodeKind::InfixExpression(SurfaceInfixOperator {
                spelling: operator.spelling.clone(),
                precedence: operator.precedence,
                associativity: surface_associativity(infix_associativity(operator)),
            }),
            range: self.covering_token_range(start_position, cursor),
            children: vec![left.id, self.token_node_ids[operator_position], right.id],
        });
        ParsedTypeNode {
            id,
            next_position: cursor,
            recovery_nodes,
        }
    }

    fn prefix_operator_at(&self, position: usize) -> Option<&OperatorFixityEntry> {
        let token = self.request.tokens.get(position)?;
        self.prefix_fixity_for_token(token)
    }

    fn postfix_operator_at(&self, position: usize) -> Option<&OperatorFixityEntry> {
        let token = self.request.tokens.get(position)?;
        self.postfix_fixity_for_token(token)
    }

    fn infix_operator_at(&self, position: usize) -> Option<&OperatorFixityEntry> {
        let token = self.request.tokens.get(position)?;
        self.infix_fixity_for_token(token)
    }

    fn left_is_non_associative_chain(
        &self,
        left: SurfaceBuilderNodeId,
        operator: &OperatorFixityEntry,
    ) -> bool {
        matches!(
            self.events.node_kind(left).unwrap(),
            SurfaceNodeKind::InfixExpression(left_operator)
                if left_operator.associativity == SurfaceOperatorAssociativity::NonAssociative
                    && left_operator.spelling == operator.spelling
        )
    }

    fn parse_term_postfix_chain_before_structure_field_list_at(
        &mut self,
        start_position: usize,
        primary: ParsedTypeNode,
    ) -> ParsedTypeNode {
        self.parse_term_postfix_chain(start_position, primary, true)
    }

    fn parse_term_postfix_chain(
        &mut self,
        start_position: usize,
        mut term: ParsedTypeNode,
        stop_before_structure_field_list: bool,
    ) -> ParsedTypeNode {
        loop {
            let cursor = term.next_position;
            if stop_before_structure_field_list && self.is_structure_field_list_opener_at(cursor) {
                break;
            }
            if self.is_reserved_symbol_at(cursor, ".") {
                term = self.parse_selector_access_after_base(
                    start_position,
                    term,
                    cursor,
                    stop_before_structure_field_list,
                );
                continue;
            }
            if self.is_reserved_word_at(cursor, "with") {
                term = self.parse_structure_update_after_base(start_position, term, cursor);
                continue;
            }
            break;
        }
        term
    }

    fn parse_qua_chain_at(
        &mut self,
        start_position: usize,
        mut term: ParsedTypeNode,
        target_grammar: QuaTargetGrammar,
    ) -> ParsedTypeNode {
        while self.is_reserved_word_at(term.next_position, "qua") {
            let qua_position = term.next_position;
            term = self.parse_qua_expression_after_base(
                start_position,
                term,
                qua_position,
                target_grammar,
            );
        }
        term
    }

    fn parse_qua_expression_after_base(
        &mut self,
        start_position: usize,
        base: ParsedTypeNode,
        qua_position: usize,
        target_grammar: QuaTargetGrammar,
    ) -> ParsedTypeNode {
        let mut children = vec![base.id, self.token_node_ids[qua_position]];
        let mut recovery_nodes = base.recovery_nodes;
        let mut cursor = qua_position + 1;

        match self.parse_qua_target_type_at(cursor, target_grammar) {
            Some(target) => {
                children.push(target.id);
                recovery_nodes.extend(target.recovery_nodes);
                cursor = target.next_position;
            }
            None => {
                self.diagnose_malformed_type_expression(
                    cursor,
                    "expected type expression after `qua`",
                );
                let missing = self.add_missing_type_expression(cursor);
                children.push(missing);
                recovery_nodes.push(missing);
                if let Some(recovery) = self.recover_malformed_type_tail(cursor) {
                    children.push(recovery.id);
                    recovery_nodes.extend(recovery.recovery_nodes);
                    cursor = recovery.next_position;
                }
            }
        }

        let id = self.events.emit(SyntaxEvent::Node {
            kind: SurfaceNodeKind::QuaExpression,
            range: self.covering_token_range(start_position, cursor),
            children,
        });
        ParsedTypeNode {
            id,
            next_position: cursor.max(qua_position + 1),
            recovery_nodes,
        }
    }

    fn parse_qua_target_type_at(
        &mut self,
        position: usize,
        target_grammar: QuaTargetGrammar,
    ) -> Option<ParsedTypeNode> {
        match target_grammar {
            QuaTargetGrammar::TypeExpression => self.parse_type_expression_at(position),
            QuaTargetGrammar::RadixType => self.parse_radix_type_expression_at(position),
        }
    }

    fn parse_radix_type_expression_at(&mut self, position: usize) -> Option<ParsedTypeNode> {
        if self.is_type_expression_boundary_at(position) {
            return None;
        }
        let head = self.parse_type_head_at(position)?;
        let id = self.events.emit(SyntaxEvent::Node {
            kind: SurfaceNodeKind::TypeExpression,
            range: self.covering_token_range(position, head.next_position),
            children: vec![head.id],
        });
        Some(ParsedTypeNode {
            id,
            next_position: head.next_position.max(position + 1),
            recovery_nodes: head.recovery_nodes,
        })
    }

    fn parse_term_primary_at(&mut self, position: usize) -> Option<ParsedTypeNode> {
        if self.qualified_symbol_next_at(position).is_some() {
            return self.parse_qualified_symbol_term_at(position);
        }
        if self.is_identifier_at(position) {
            return Some(self.parse_identifier_term_at(position));
        }
        if self
            .request
            .tokens
            .get(position)
            .is_some_and(|token| token.kind == ParserTokenKind::Numeral)
        {
            return Some(self.parse_single_token_term_at(position, SurfaceNodeKind::NumeralTerm));
        }
        if self.is_reserved_word_at(position, "it") {
            return Some(self.parse_single_token_term_at(position, SurfaceNodeKind::ItTerm));
        }
        if self.is_reserved_word_at(position, "the") {
            return Some(self.parse_choice_term_at(position));
        }
        if self.is_reserved_symbol_at(position, "(") {
            return Some(self.parse_parenthesized_term_at(position));
        }
        if self.is_reserved_symbol_at(position, "[") {
            return Some(self.parse_reserved_bracket_application_at(position));
        }
        if self.is_reserved_symbol_at(position, "{") {
            return Some(self.parse_set_expression_at(position));
        }
        None
    }

    fn parse_identifier_term_at(&mut self, position: usize) -> ParsedTypeNode {
        let reference = self.parse_identifier_reference_term_at(position).id;
        if self.is_reserved_symbol_at(position + 1, "(") {
            self.parse_application_term_after_callee(position, reference, position + 1)
        } else {
            ParsedTypeNode {
                id: reference,
                next_position: position + 1,
                recovery_nodes: Vec::new(),
            }
        }
    }

    fn parse_identifier_reference_term_at(&mut self, position: usize) -> ParsedTypeNode {
        let reference = self.events.emit(SyntaxEvent::Node {
            kind: SurfaceNodeKind::TermReference,
            range: self.covering_token_range(position, position + 1),
            children: vec![self.token_node_ids[position]],
        });
        ParsedTypeNode {
            id: reference,
            next_position: position + 1,
            recovery_nodes: Vec::new(),
        }
    }

    fn parse_qualified_symbol_term_at(&mut self, position: usize) -> Option<ParsedTypeNode> {
        let symbol_end = self.qualified_symbol_next_at(position)?;
        if self.can_parse_structure_constructor_after_symbol(symbol_end) {
            return Some(self.parse_structure_constructor_at(position));
        }

        let reference = self.parse_qualified_symbol_reference_term_at(position)?;
        if self.is_reserved_symbol_at(reference.next_position, "(") {
            Some(self.parse_application_term_after_callee(
                position,
                reference.id,
                reference.next_position,
            ))
        } else {
            Some(reference)
        }
    }

    fn parse_qualified_symbol_reference_term_at(
        &mut self,
        position: usize,
    ) -> Option<ParsedTypeNode> {
        let symbol = self.parse_qualified_symbol_at(position)?;
        let reference = self.events.emit(SyntaxEvent::Node {
            kind: SurfaceNodeKind::TermReference,
            range: self.covering_token_range(position, symbol.next_position),
            children: vec![symbol.id],
        });
        Some(ParsedTypeNode {
            id: reference,
            next_position: symbol.next_position.max(position + 1),
            recovery_nodes: Vec::new(),
        })
    }

    fn parse_single_token_term_at(
        &mut self,
        position: usize,
        kind: SurfaceNodeKind,
    ) -> ParsedTypeNode {
        let id = self.events.emit(SyntaxEvent::Node {
            kind,
            range: self.covering_token_range(position, position + 1),
            children: vec![self.token_node_ids[position]],
        });
        ParsedTypeNode {
            id,
            next_position: position + 1,
            recovery_nodes: Vec::new(),
        }
    }

    fn parse_choice_term_at(&mut self, position: usize) -> ParsedTypeNode {
        let mut children = vec![self.token_node_ids[position]];
        let mut recovery_nodes = Vec::new();
        let mut cursor = position + 1;
        match self.parse_type_expression_at(cursor) {
            Some(type_expression) => {
                children.push(type_expression.id);
                recovery_nodes.extend(type_expression.recovery_nodes);
                cursor = type_expression.next_position;
            }
            None => {
                self.diagnose_malformed_type_expression(
                    cursor,
                    "expected type expression after `the`",
                );
                let recovery = self.add_missing_type_expression(cursor);
                children.push(recovery);
                recovery_nodes.push(recovery);
            }
        }

        let id = self.events.emit(SyntaxEvent::Node {
            kind: SurfaceNodeKind::ChoiceTerm,
            range: self.covering_token_range(position, cursor),
            children,
        });
        ParsedTypeNode {
            id,
            next_position: cursor.max(position + 1),
            recovery_nodes,
        }
    }

    fn parse_parenthesized_term_at(&mut self, position: usize) -> ParsedTypeNode {
        let mut children = vec![self.token_node_ids[position]];
        let mut recovery_nodes = Vec::new();
        let mut cursor = position + 1;
        match self.parse_term_expression_at(cursor) {
            Some(term) => {
                children.push(term.id);
                recovery_nodes.extend(term.recovery_nodes);
                cursor = term.next_position;
            }
            None => {
                self.diagnose_malformed_term_expression(cursor, "expected term after `(`");
                self.push_missing_term(cursor, &mut children, &mut recovery_nodes);
            }
        }

        if self.is_reserved_symbol_at(cursor, ")") {
            children.push(self.token_node_ids[cursor]);
            cursor += 1;
        } else {
            self.diagnose_unmatched_term_delimiter(position, cursor, ")");
            let recovery = self.add_recovery_node(
                SyntaxRecoveryKind::UnmatchedOpeningDelimiter,
                self.zero_range_at(cursor),
                Vec::new(),
            );
            children.push(recovery);
            recovery_nodes.push(recovery);
        }

        let id = self.events.emit(SyntaxEvent::Node {
            kind: SurfaceNodeKind::ParenthesizedTerm,
            range: self.covering_token_range(position, cursor),
            children,
        });
        ParsedTypeNode {
            id,
            next_position: cursor.max(position + 1),
            recovery_nodes,
        }
    }

    fn parse_application_term_after_callee(
        &mut self,
        start_position: usize,
        callee: SurfaceBuilderNodeId,
        opener: usize,
    ) -> ParsedTypeNode {
        let mut children = vec![callee, self.token_node_ids[opener]];
        let mut recovery_nodes = Vec::new();
        let mut cursor = self.parse_term_list_until(
            opener + 1,
            ")",
            true,
            None,
            &mut children,
            &mut recovery_nodes,
        );
        if self.is_reserved_symbol_at(cursor, ")") {
            children.push(self.token_node_ids[cursor]);
            cursor += 1;
        } else {
            self.diagnose_unmatched_term_delimiter(opener, cursor, ")");
            let recovery = self.add_recovery_node(
                SyntaxRecoveryKind::UnmatchedOpeningDelimiter,
                self.zero_range_at(cursor),
                Vec::new(),
            );
            children.push(recovery);
            recovery_nodes.push(recovery);
        }

        let id = self.events.emit(SyntaxEvent::Node {
            kind: SurfaceNodeKind::ApplicationTerm,
            range: self.covering_token_range(start_position, cursor),
            children,
        });
        ParsedTypeNode {
            id,
            next_position: cursor.max(start_position + 1),
            recovery_nodes,
        }
    }

    fn parse_selector_access_after_base(
        &mut self,
        start_position: usize,
        base: ParsedTypeNode,
        dot_position: usize,
        stop_before_structure_field_list: bool,
    ) -> ParsedTypeNode {
        let mut children = vec![base.id, self.token_node_ids[dot_position]];
        let mut recovery_nodes = base.recovery_nodes;
        let mut cursor = dot_position + 1;
        let mut has_selector_name = false;

        if self.is_identifier_at(cursor) {
            children.push(self.token_node_ids[cursor]);
            cursor += 1;
            has_selector_name = true;
        } else {
            self.diagnose_malformed_term_expression(cursor, "expected selector name after `.`");
        }

        if has_selector_name
            && self.is_reserved_symbol_at(cursor, "(")
            && !(stop_before_structure_field_list && self.is_structure_field_list_opener_at(cursor))
        {
            let opener = cursor;
            children.push(self.token_node_ids[cursor]);
            cursor = self.parse_term_list_until(
                cursor + 1,
                ")",
                true,
                None,
                &mut children,
                &mut recovery_nodes,
            );
            if self.is_reserved_symbol_at(cursor, ")") {
                children.push(self.token_node_ids[cursor]);
                cursor += 1;
            } else {
                self.diagnose_unmatched_term_delimiter(opener, cursor, ")");
                let recovery = self.add_recovery_node(
                    SyntaxRecoveryKind::UnmatchedOpeningDelimiter,
                    self.zero_range_at(cursor),
                    Vec::new(),
                );
                children.push(recovery);
                recovery_nodes.push(recovery);
            }
        }

        let id = self.events.emit(SyntaxEvent::Node {
            kind: SurfaceNodeKind::SelectorAccess,
            range: self.covering_token_range(start_position, cursor),
            children,
        });
        ParsedTypeNode {
            id,
            next_position: cursor.max(dot_position + 1),
            recovery_nodes,
        }
    }

    fn parse_structure_update_after_base(
        &mut self,
        start_position: usize,
        base: ParsedTypeNode,
        with_position: usize,
    ) -> ParsedTypeNode {
        let mut children = vec![base.id, self.token_node_ids[with_position]];
        let mut recovery_nodes = base.recovery_nodes;
        let mut cursor = with_position + 1;

        if !self.is_reserved_symbol_at(cursor, "(") {
            self.diagnose_malformed_term_expression(cursor, "expected `(` after `with`");
            if let Some(recovery) = self.recover_malformed_term_tail(cursor, "") {
                children.push(recovery.id);
                recovery_nodes.extend(recovery.recovery_nodes);
                cursor = recovery.next_position;
            }
        } else {
            let opener = cursor;
            children.push(self.token_node_ids[cursor]);
            cursor = self.parse_field_update_list_until(
                cursor + 1,
                ")",
                &mut children,
                &mut recovery_nodes,
            );
            if self.is_reserved_symbol_at(cursor, ")") {
                children.push(self.token_node_ids[cursor]);
                cursor += 1;
            } else {
                self.diagnose_unmatched_term_delimiter(opener, cursor, ")");
                let recovery = self.add_recovery_node(
                    SyntaxRecoveryKind::UnmatchedOpeningDelimiter,
                    self.zero_range_at(cursor),
                    Vec::new(),
                );
                children.push(recovery);
                recovery_nodes.push(recovery);
            }
        }

        let id = self.events.emit(SyntaxEvent::Node {
            kind: SurfaceNodeKind::StructureUpdate,
            range: self.covering_token_range(start_position, cursor),
            children,
        });
        ParsedTypeNode {
            id,
            next_position: cursor.max(with_position + 1),
            recovery_nodes,
        }
    }

    fn parse_reserved_bracket_application_at(&mut self, position: usize) -> ParsedTypeNode {
        let mut children = vec![self.token_node_ids[position]];
        let mut recovery_nodes = Vec::new();
        let mut cursor = self.parse_term_list_until(
            position + 1,
            "]",
            false,
            None,
            &mut children,
            &mut recovery_nodes,
        );
        if self.is_reserved_symbol_at(cursor, "]") {
            children.push(self.token_node_ids[cursor]);
            cursor += 1;
        } else {
            self.diagnose_unmatched_term_delimiter(position, cursor, "]");
            let recovery = self.add_recovery_node(
                SyntaxRecoveryKind::UnmatchedOpeningDelimiter,
                self.zero_range_at(cursor),
                Vec::new(),
            );
            children.push(recovery);
            recovery_nodes.push(recovery);
        }

        let id = self.events.emit(SyntaxEvent::Node {
            kind: SurfaceNodeKind::ApplicationTerm,
            range: self.covering_token_range(position, cursor),
            children,
        });
        ParsedTypeNode {
            id,
            next_position: cursor.max(position + 1),
            recovery_nodes,
        }
    }

    fn parse_set_expression_at(&mut self, position: usize) -> ParsedTypeNode {
        if self
            .set_comprehension_where_before_first_separator_at(position)
            .is_some()
        {
            self.parse_set_comprehension_at(position)
        } else {
            self.parse_set_enumeration_at(position)
        }
    }

    fn set_comprehension_where_before_first_separator_at(&self, position: usize) -> Option<usize> {
        let mut cursor = position + 1;
        let mut paren_depth = 0_usize;
        let mut bracket_depth = 0_usize;
        let mut brace_depth = 0_usize;

        while cursor < self.request.tokens.len() {
            let top_level = paren_depth == 0 && bracket_depth == 0 && brace_depth == 0;
            if top_level {
                if self.is_reserved_word_at(cursor, "where") {
                    return Some(cursor);
                }
                if self.is_reserved_symbol_at(cursor, ",")
                    || self.is_reserved_symbol_at(cursor, "}")
                    || self.is_semicolon_at(cursor)
                    || self.is_item_start_at(cursor)
                {
                    return None;
                }
            }

            if self.is_reserved_symbol_at(cursor, "(") {
                paren_depth += 1;
            } else if self.is_reserved_symbol_at(cursor, ")") {
                if paren_depth == 0 {
                    return None;
                }
                paren_depth -= 1;
            } else if self.is_reserved_symbol_at(cursor, "[") {
                bracket_depth += 1;
            } else if self.is_reserved_symbol_at(cursor, "]") {
                if bracket_depth == 0 {
                    return None;
                }
                bracket_depth -= 1;
            } else if self.is_reserved_symbol_at(cursor, "{") {
                brace_depth += 1;
            } else if self.is_reserved_symbol_at(cursor, "}") {
                if brace_depth == 0 {
                    return None;
                }
                brace_depth -= 1;
            }
            cursor += 1;
        }

        None
    }

    fn parse_set_enumeration_at(&mut self, position: usize) -> ParsedTypeNode {
        let mut children = vec![self.token_node_ids[position]];
        let mut recovery_nodes = Vec::new();
        let mut cursor = self.parse_term_list_until(
            position + 1,
            "}",
            true,
            None,
            &mut children,
            &mut recovery_nodes,
        );
        if self.is_reserved_symbol_at(cursor, "}") {
            children.push(self.token_node_ids[cursor]);
            cursor += 1;
        } else {
            self.diagnose_unmatched_term_delimiter(position, cursor, "}");
            let recovery = self.add_recovery_node(
                SyntaxRecoveryKind::UnmatchedOpeningDelimiter,
                self.zero_range_at(cursor),
                Vec::new(),
            );
            children.push(recovery);
            recovery_nodes.push(recovery);
        }

        let id = self.events.emit(SyntaxEvent::Node {
            kind: SurfaceNodeKind::SetEnumeration,
            range: self.covering_token_range(position, cursor),
            children,
        });
        ParsedTypeNode {
            id,
            next_position: cursor.max(position + 1),
            recovery_nodes,
        }
    }

    fn parse_set_comprehension_at(&mut self, position: usize) -> ParsedTypeNode {
        let mut children = vec![self.token_node_ids[position]];
        let mut recovery_nodes = Vec::new();
        let mut cursor = position + 1;

        if self.is_reserved_word_at(cursor, "where") {
            self.diagnose_malformed_term_expression(cursor, "expected mapper term before `where`");
            self.push_missing_term(cursor, &mut children, &mut recovery_nodes);
        } else if let Some(mapper) = self.parse_term_expression_at(cursor) {
            cursor = mapper.next_position;
            children.push(mapper.id);
            recovery_nodes.extend(mapper.recovery_nodes);
        } else {
            self.diagnose_malformed_term_expression(cursor, "expected mapper term after `{`");
            self.push_missing_term(cursor, &mut children, &mut recovery_nodes);
            if let Some(recovery) = self.recover_malformed_set_comprehension_mapper_tail(cursor) {
                cursor = recovery.next_position;
                children.push(recovery.id);
                recovery_nodes.extend(recovery.recovery_nodes);
            }
        }

        if self.is_reserved_word_at(cursor, "where") {
            children.push(self.token_node_ids[cursor]);
            cursor += 1;
        } else {
            self.diagnose_malformed_term_expression(
                cursor,
                "expected `where` in set comprehension",
            );
            if let Some(recovery) = self.recover_malformed_set_comprehension_mapper_tail(cursor) {
                cursor = recovery.next_position;
                children.push(recovery.id);
                recovery_nodes.extend(recovery.recovery_nodes);
            }
            if self.is_reserved_word_at(cursor, "where") {
                children.push(self.token_node_ids[cursor]);
                cursor += 1;
            }
        }

        if let Some(segment) = self.parse_comprehension_variable_segment_at(cursor) {
            cursor = segment.next_position;
            children.push(segment.id);
            recovery_nodes.extend(segment.recovery_nodes);
        } else {
            self.diagnose_malformed_term_expression(
                cursor,
                "expected set-comprehension generator after `where`",
            );
            let segment = self.add_missing_comprehension_variable_segment(cursor);
            children.push(segment.id);
            recovery_nodes.extend(segment.recovery_nodes);
        }

        while self.is_reserved_symbol_at(cursor, ",") {
            children.push(self.token_node_ids[cursor]);
            cursor += 1;
            if let Some(segment) = self.parse_comprehension_variable_segment_at(cursor) {
                cursor = segment.next_position;
                children.push(segment.id);
                recovery_nodes.extend(segment.recovery_nodes);
            } else {
                self.diagnose_malformed_term_expression(
                    cursor,
                    "expected set-comprehension generator after `,`",
                );
                let segment = self.add_missing_comprehension_variable_segment(cursor);
                children.push(segment.id);
                recovery_nodes.extend(segment.recovery_nodes);
                break;
            }
        }

        if self.is_reserved_symbol_at(cursor, ":") {
            children.push(self.token_node_ids[cursor]);
            cursor += 1;
            if let Some(condition) = self.parse_formula_expression_at(cursor) {
                cursor = condition.next_position;
                children.push(condition.id);
                recovery_nodes.extend(condition.recovery_nodes);
            } else {
                self.diagnose_malformed_formula_expression(
                    cursor,
                    "expected formula after set-comprehension `:`",
                );
                let missing = self.add_missing_formula(cursor);
                children.push(missing);
                recovery_nodes.push(missing);
            }
        }

        if self.is_reserved_symbol_at(cursor, "}") {
            children.push(self.token_node_ids[cursor]);
            cursor += 1;
        } else {
            self.diagnose_unmatched_term_delimiter(position, cursor, "}");
            let recovery = self.add_recovery_node(
                SyntaxRecoveryKind::UnmatchedOpeningDelimiter,
                self.zero_range_at(cursor),
                Vec::new(),
            );
            children.push(recovery);
            recovery_nodes.push(recovery);
        }

        let id = self.events.emit(SyntaxEvent::Node {
            kind: SurfaceNodeKind::SetComprehension,
            range: self.covering_token_range(position, cursor),
            children,
        });
        ParsedTypeNode {
            id,
            next_position: cursor.max(position + 1),
            recovery_nodes,
        }
    }

    fn parse_comprehension_variable_segment_at(
        &mut self,
        position: usize,
    ) -> Option<ParsedTypeNode> {
        let mut children = Vec::new();
        let mut recovery_nodes = Vec::new();
        let mut cursor = position;

        if self.is_identifier_at(cursor) {
            children.push(self.token_node_ids[cursor]);
            cursor += 1;
        } else if self.is_reserved_word_at(cursor, "is") {
            self.diagnose_malformed_term_expression(
                cursor,
                "expected set-comprehension generator before `is`",
            );
            self.push_missing_term(cursor, &mut children, &mut recovery_nodes);
        } else {
            return None;
        }

        if self.is_reserved_word_at(cursor, "is") {
            children.push(self.token_node_ids[cursor]);
            cursor += 1;
            if let Some(type_expression) = self.parse_type_expression_at(cursor) {
                cursor = type_expression.next_position;
                children.push(type_expression.id);
                recovery_nodes.extend(type_expression.recovery_nodes);
            } else {
                self.diagnose_malformed_type_expression(
                    cursor,
                    "expected type after set-comprehension generator `is`",
                );
                let missing = self.add_missing_type_expression(cursor);
                children.push(missing);
                recovery_nodes.push(missing);
            }
        } else {
            self.diagnose_malformed_term_expression(
                cursor,
                "expected `is` in set-comprehension generator",
            );
            if let Some(recovery) = self.recover_malformed_comprehension_generator_tail(cursor) {
                cursor = recovery.next_position;
                children.push(recovery.id);
                recovery_nodes.extend(recovery.recovery_nodes);
            }
        }

        let id = self.events.emit(SyntaxEvent::Node {
            kind: SurfaceNodeKind::ComprehensionVariableSegment,
            range: self.covering_token_range(position, cursor.max(position + 1)),
            children,
        });
        Some(ParsedTypeNode {
            id,
            next_position: cursor.max(position + 1),
            recovery_nodes,
        })
    }

    fn add_missing_comprehension_variable_segment(&mut self, position: usize) -> ParsedTypeNode {
        let missing = self.add_missing_term(position);
        let id = self.events.emit(SyntaxEvent::Node {
            kind: SurfaceNodeKind::ComprehensionVariableSegment,
            range: self.zero_range_at(position),
            children: vec![missing],
        });
        ParsedTypeNode {
            id,
            next_position: position,
            recovery_nodes: vec![missing],
        }
    }

    fn parse_structure_constructor_at(&mut self, position: usize) -> ParsedTypeNode {
        let symbol = self
            .parse_qualified_symbol_at(position)
            .expect("structure constructor starts with a qualified symbol");
        let mut children = vec![symbol.id];
        let mut recovery_nodes = Vec::new();
        let mut cursor = symbol.next_position;

        if self.is_type_arguments_start_at(cursor) {
            let arguments = self.parse_type_arguments_for_structure_constructor_at(cursor);
            children.push(arguments.id);
            recovery_nodes.extend(arguments.recovery_nodes);
            cursor = arguments.next_position;
        }

        let opener = cursor;
        children.push(self.token_node_ids[cursor]);
        cursor += 1;
        cursor =
            self.parse_field_argument_list_until(cursor, ")", &mut children, &mut recovery_nodes);
        if self.is_reserved_symbol_at(cursor, ")") {
            children.push(self.token_node_ids[cursor]);
            cursor += 1;
        } else {
            self.diagnose_unmatched_term_delimiter(opener, cursor, ")");
            let recovery = self.add_recovery_node(
                SyntaxRecoveryKind::UnmatchedOpeningDelimiter,
                self.zero_range_at(cursor),
                Vec::new(),
            );
            children.push(recovery);
            recovery_nodes.push(recovery);
        }

        let id = self.events.emit(SyntaxEvent::Node {
            kind: SurfaceNodeKind::StructureConstructor,
            range: self.covering_token_range(position, cursor),
            children,
        });
        ParsedTypeNode {
            id,
            next_position: cursor.max(position + 1),
            recovery_nodes,
        }
    }

    fn parse_field_argument_at(&mut self, position: usize) -> Option<ParsedTypeNode> {
        if !self.is_field_argument_start_at(position) {
            return None;
        }
        let mut children = vec![
            self.token_node_ids[position],
            self.token_node_ids[position + 1],
        ];
        let mut recovery_nodes = Vec::new();
        let mut cursor = position + 2;
        if let Some(term) = self.parse_term_expression_at(cursor) {
            children.push(term.id);
            recovery_nodes.extend(term.recovery_nodes);
            cursor = term.next_position;
        } else {
            self.diagnose_malformed_term_expression(
                cursor,
                "expected term after field argument `:`",
            );
            self.push_missing_term(cursor, &mut children, &mut recovery_nodes);
            if let Some(recovery) = self.recover_malformed_term_tail(cursor, ")") {
                children.push(recovery.id);
                recovery_nodes.extend(recovery.recovery_nodes);
                cursor = recovery.next_position;
            }
        }

        let id = self.events.emit(SyntaxEvent::Node {
            kind: SurfaceNodeKind::FieldArgument,
            range: self.covering_token_range(position, cursor),
            children,
        });
        Some(ParsedTypeNode {
            id,
            next_position: cursor.max(position + 1),
            recovery_nodes,
        })
    }

    fn parse_field_update_at(&mut self, position: usize) -> Option<ParsedTypeNode> {
        if !self.is_identifier_at(position) {
            return None;
        }

        let mut children = vec![self.token_node_ids[position]];
        let mut recovery_nodes = Vec::new();
        let mut cursor = position + 1;
        let mut malformed_selector_path = false;
        while self.is_reserved_symbol_at(cursor, ".") {
            children.push(self.token_node_ids[cursor]);
            cursor += 1;
            if self.is_identifier_at(cursor) {
                children.push(self.token_node_ids[cursor]);
                cursor += 1;
            } else {
                self.diagnose_malformed_term_expression(
                    cursor,
                    "expected field selector after `.` in structure update",
                );
                malformed_selector_path = true;
                break;
            }
        }

        if !self.is_reserved_symbol_at(cursor, ":=") {
            if malformed_selector_path {
                let id = self.events.emit(SyntaxEvent::Node {
                    kind: SurfaceNodeKind::FieldUpdate,
                    range: self.covering_token_range(position, cursor),
                    children,
                });
                return Some(ParsedTypeNode {
                    id,
                    next_position: cursor.max(position + 1),
                    recovery_nodes,
                });
            }
            return None;
        }
        children.push(self.token_node_ids[cursor]);
        cursor += 1;

        if let Some(term) = self.parse_term_expression_at(cursor) {
            children.push(term.id);
            recovery_nodes.extend(term.recovery_nodes);
            cursor = term.next_position;
        } else {
            self.diagnose_malformed_term_expression(
                cursor,
                "expected term after field update `:=`",
            );
            self.push_missing_term(cursor, &mut children, &mut recovery_nodes);
            if let Some(recovery) = self.recover_malformed_term_tail(cursor, ")") {
                children.push(recovery.id);
                recovery_nodes.extend(recovery.recovery_nodes);
                cursor = recovery.next_position;
            }
        }

        let id = self.events.emit(SyntaxEvent::Node {
            kind: SurfaceNodeKind::FieldUpdate,
            range: self.covering_token_range(position, cursor),
            children,
        });
        Some(ParsedTypeNode {
            id,
            next_position: cursor.max(position + 1),
            recovery_nodes,
        })
    }

    fn parse_term_list_until(
        &mut self,
        mut cursor: usize,
        close_symbol: &str,
        allow_empty: bool,
        stop_position: Option<usize>,
        children: &mut Vec<SurfaceBuilderNodeId>,
        recovery_nodes: &mut Vec<SurfaceBuilderNodeId>,
    ) -> usize {
        let mut expecting_argument = true;
        let mut saw_argument = false;
        if allow_empty && self.is_reserved_symbol_at(cursor, close_symbol) {
            return cursor;
        }

        while cursor < self.request.tokens.len()
            && stop_position.is_none_or(|stop| cursor < stop)
            && !self.is_reserved_symbol_at(cursor, close_symbol)
            && !self.is_term_argument_list_boundary_at(cursor)
        {
            if self.is_reserved_symbol_at(cursor, ",") {
                if expecting_argument {
                    self.diagnose_malformed_term_expression(cursor, "expected term before `,`");
                    self.push_missing_term(cursor, children, recovery_nodes);
                }
                children.push(self.token_node_ids[cursor]);
                cursor += 1;
                expecting_argument = true;
                continue;
            }

            if let Some(term) = self.parse_term_expression_at(cursor) {
                children.push(term.id);
                recovery_nodes.extend(term.recovery_nodes);
                cursor = term.next_position;
                expecting_argument = false;
                saw_argument = true;
            } else {
                self.diagnose_malformed_term_expression(cursor, "malformed term expression");
                self.push_missing_term(cursor, children, recovery_nodes);
                if let Some(recovery) = self.recover_malformed_term_tail(cursor, close_symbol) {
                    children.push(recovery.id);
                    recovery_nodes.extend(recovery.recovery_nodes);
                    cursor = recovery.next_position;
                }
                expecting_argument = false;
                saw_argument = true;
            }

            if stop_position.is_some_and(|stop| cursor >= stop) {
                break;
            }
            if self.is_reserved_symbol_at(cursor, ",") {
                children.push(self.token_node_ids[cursor]);
                cursor += 1;
                expecting_argument = true;
            } else if self.is_reserved_symbol_at(cursor, close_symbol)
                || self.is_term_argument_list_boundary_at(cursor)
            {
                break;
            } else {
                self.diagnose_malformed_term_expression(cursor, "expected `,` between terms");
                if let Some(recovery) = self.recover_malformed_term_tail(cursor, close_symbol) {
                    children.push(recovery.id);
                    recovery_nodes.extend(recovery.recovery_nodes);
                    cursor = recovery.next_position;
                }
                break;
            }
        }

        if expecting_argument && (!allow_empty || saw_argument) {
            self.diagnose_malformed_term_expression(
                cursor,
                if saw_argument {
                    "expected term after `,`"
                } else {
                    "expected term before closing delimiter"
                },
            );
            self.push_missing_term(cursor, children, recovery_nodes);
        }
        cursor
    }

    fn parse_field_argument_list_until(
        &mut self,
        mut cursor: usize,
        close_symbol: &str,
        children: &mut Vec<SurfaceBuilderNodeId>,
        recovery_nodes: &mut Vec<SurfaceBuilderNodeId>,
    ) -> usize {
        let mut expecting_argument = true;
        let mut saw_argument = false;
        while cursor < self.request.tokens.len()
            && !self.is_reserved_symbol_at(cursor, close_symbol)
            && !self.is_term_argument_list_boundary_at(cursor)
        {
            if self.is_reserved_symbol_at(cursor, ",") {
                if expecting_argument {
                    self.diagnose_malformed_term_expression(
                        cursor,
                        "expected field argument before `,`",
                    );
                }
                children.push(self.token_node_ids[cursor]);
                cursor += 1;
                expecting_argument = true;
                continue;
            }

            if let Some(field) = self.parse_field_argument_at(cursor) {
                children.push(field.id);
                recovery_nodes.extend(field.recovery_nodes);
                cursor = field.next_position;
                expecting_argument = false;
                saw_argument = true;
            } else {
                self.diagnose_malformed_term_expression(cursor, "expected field argument");
                if let Some(recovery) = self.recover_malformed_term_tail(cursor, close_symbol) {
                    children.push(recovery.id);
                    recovery_nodes.extend(recovery.recovery_nodes);
                    cursor = recovery.next_position;
                }
                break;
            }

            if self.is_reserved_symbol_at(cursor, ",") {
                children.push(self.token_node_ids[cursor]);
                cursor += 1;
                expecting_argument = true;
            } else if self.is_reserved_symbol_at(cursor, close_symbol)
                || self.is_term_argument_list_boundary_at(cursor)
            {
                break;
            } else {
                self.diagnose_malformed_term_expression(
                    cursor,
                    "expected `,` between field arguments",
                );
                if let Some(recovery) = self.recover_malformed_term_tail(cursor, close_symbol) {
                    children.push(recovery.id);
                    recovery_nodes.extend(recovery.recovery_nodes);
                    cursor = recovery.next_position;
                }
                break;
            }
        }

        if expecting_argument && saw_argument {
            self.diagnose_malformed_term_expression(cursor, "expected field argument after `,`");
        }
        cursor
    }

    fn parse_field_update_list_until(
        &mut self,
        mut cursor: usize,
        close_symbol: &str,
        children: &mut Vec<SurfaceBuilderNodeId>,
        recovery_nodes: &mut Vec<SurfaceBuilderNodeId>,
    ) -> usize {
        let mut expecting_update = true;
        let mut saw_update = false;
        while cursor < self.request.tokens.len()
            && !self.is_reserved_symbol_at(cursor, close_symbol)
            && !self.is_term_argument_list_boundary_at(cursor)
        {
            if self.is_reserved_symbol_at(cursor, ",") {
                if expecting_update {
                    self.diagnose_malformed_term_expression(
                        cursor,
                        "expected field update before `,`",
                    );
                }
                children.push(self.token_node_ids[cursor]);
                cursor += 1;
                expecting_update = true;
                continue;
            }

            if let Some(update) = self.parse_field_update_at(cursor) {
                children.push(update.id);
                recovery_nodes.extend(update.recovery_nodes);
                cursor = update.next_position;
                expecting_update = false;
                saw_update = true;
            } else {
                self.diagnose_malformed_term_expression(cursor, "expected field update");
                if let Some(recovery) = self.recover_malformed_term_tail(cursor, close_symbol) {
                    children.push(recovery.id);
                    recovery_nodes.extend(recovery.recovery_nodes);
                    cursor = recovery.next_position;
                }
                break;
            }

            if self.is_reserved_symbol_at(cursor, ",") {
                children.push(self.token_node_ids[cursor]);
                cursor += 1;
                expecting_update = true;
            } else if self.is_reserved_symbol_at(cursor, close_symbol)
                || self.is_term_argument_list_boundary_at(cursor)
            {
                break;
            } else {
                self.diagnose_malformed_term_expression(
                    cursor,
                    "expected `,` between field updates",
                );
                if let Some(recovery) = self.recover_malformed_term_tail(cursor, close_symbol) {
                    children.push(recovery.id);
                    recovery_nodes.extend(recovery.recovery_nodes);
                    cursor = recovery.next_position;
                }
                break;
            }
        }

        if expecting_update {
            if saw_update {
                self.diagnose_malformed_term_expression(cursor, "expected field update after `,`");
            } else {
                self.diagnose_malformed_term_expression(
                    cursor,
                    "expected field update before closing delimiter",
                );
            }
        }
        cursor
    }

    fn parse_bracket_qua_argument_at(&mut self, position: usize) -> Option<ParsedTypeNode> {
        if !self.is_identifier_at(position) {
            return None;
        }
        let base = self.parse_identifier_reference_term_at(position);
        let term = self.parse_qua_chain_at(position, base, QuaTargetGrammar::RadixType);
        Some(self.wrap_term_expression(position, term))
    }

    fn parse_attribute_symbol_at(&mut self, position: usize) -> Option<ParsedPathNode> {
        let plan = self.attribute_symbol_plan_at(position)?;
        let mut children = Vec::new();
        for (segment, separator) in &plan.prefix_segments {
            children.push(self.emit_wrapped_token(SurfaceNodeKind::PathSegment, *segment));
            children.push(self.token_node_ids[*separator]);
        }
        children.push(self.emit_wrapped_token(SurfaceNodeKind::PathSegment, plan.final_symbol));
        let id = self.events.emit(SyntaxEvent::Node {
            kind: SurfaceNodeKind::QualifiedSymbol,
            range: self.covering_token_range(plan.first_position(), plan.next_position),
            children,
        });
        Some(ParsedPathNode {
            id,
            next_position: plan.next_position,
        })
    }

    fn parse_placeholder_item(&mut self, position: usize) -> ParsedItem {
        let head = self
            .item_head_position(position)
            .expect("placeholder parsing starts at an item boundary");
        if is_block_like_top_level_start(&self.request.tokens[head]) {
            self.parse_block_placeholder_item(position, head)
        } else {
            self.parse_semicolon_placeholder_item(position, head)
        }
    }

    fn parse_semicolon_placeholder_item(&mut self, position: usize, head: usize) -> ParsedItem {
        if let Some(item) = self.parse_theorem_formula_placeholder_item(position, head) {
            return item;
        }

        let mut cursor = head + 1;
        let mut nested_depth = 0_usize;
        while cursor < self.request.tokens.len() {
            if nested_depth == 0 {
                if self.is_semicolon_at(cursor) {
                    return self.emit_placeholder_item(position, cursor + 1);
                }
                if (self.is_item_start_at(cursor) && !self.is_prefix_continuation_at(head, cursor))
                    || self.is_end_keyword_at(cursor)
                {
                    self.diagnose_missing_semicolon(cursor);
                    return self.emit_placeholder_item(position, cursor);
                }
            }

            if self.is_end_keyword_at(cursor) && nested_depth > 0 {
                nested_depth -= 1;
            } else if sync::opens_recovery_block_at(&self.request.tokens, cursor) {
                nested_depth += 1;
            }
            cursor += 1;
        }

        self.diagnose_missing_semicolon(self.request.tokens.len());
        self.emit_placeholder_item(position, self.request.tokens.len())
    }

    fn parse_theorem_formula_placeholder_item(
        &mut self,
        position: usize,
        head: usize,
    ) -> Option<ParsedItem> {
        if !(self.is_reserved_word_at(head, "theorem") || self.is_reserved_word_at(head, "lemma")) {
            return None;
        }
        let label = head + 1;
        let colon = label + 1;
        if !self.is_identifier_at(label) || !self.is_reserved_symbol_at(colon, ":") {
            return None;
        }
        let formula_start = colon + 1;
        if !self.can_start_formula_at(formula_start)
            || self.formula_payload_contains_deferred_predicate_template_args_at(formula_start)
            || self.formula_payload_contains_theorem_tail_at(formula_start)
        {
            return None;
        }

        let mut children = self.token_node_ids[position..=colon].to_vec();
        let mut recovery_nodes = Vec::new();
        let formula = self.parse_formula_expression_at(formula_start)?;
        children.push(formula.id);
        recovery_nodes.extend(formula.recovery_nodes);
        let mut cursor = formula.next_position;

        if self.is_semicolon_at(cursor) {
            children.push(self.token_node_ids[cursor]);
            cursor += 1;
        } else {
            if cursor >= self.request.tokens.len()
                || self.is_item_start_at(cursor)
                || self.is_end_keyword_at(cursor)
            {
                self.diagnose_missing_semicolon(cursor);
            } else {
                self.diagnose_malformed_term_expression(cursor, "unexpected token after formula");
                if let Some(recovery) = self.recover_malformed_term_tail(cursor, "") {
                    children.push(recovery.id);
                    recovery_nodes.extend(recovery.recovery_nodes);
                    cursor = recovery.next_position;
                }
                if self.is_semicolon_at(cursor) {
                    children.push(self.token_node_ids[cursor]);
                    cursor += 1;
                } else {
                    self.diagnose_missing_semicolon(cursor);
                }
            }
        }

        let id = self.events.emit(SyntaxEvent::Node {
            kind: SurfaceNodeKind::PlaceholderItem,
            range: self.covering_token_range(position, cursor),
            children,
        });
        Some(ParsedItem {
            id,
            next_position: cursor.max(position + 1),
            recovery_nodes,
        })
    }

    fn parse_block_placeholder_item(&mut self, position: usize, head: usize) -> ParsedItem {
        let mut cursor = head + 1;
        let mut block_depth = 1_usize;

        while cursor < self.request.tokens.len() {
            if self.is_end_keyword_at(cursor) {
                block_depth -= 1;
                if block_depth == 0 {
                    let semicolon = cursor + 1;
                    if semicolon < self.request.tokens.len() && self.is_semicolon_at(semicolon) {
                        return self.emit_placeholder_item(position, semicolon + 1);
                    }
                    self.diagnose_missing_semicolon(semicolon);
                    return self.emit_placeholder_item(position, semicolon);
                }
            } else if sync::opens_recovery_block_at(&self.request.tokens, cursor) {
                block_depth += 1;
            }
            cursor += 1;
        }

        self.diagnose_missing_semicolon(self.request.tokens.len());
        self.emit_placeholder_item(position, self.request.tokens.len())
    }

    fn emit_placeholder_item(&mut self, start: usize, end_exclusive: usize) -> ParsedItem {
        let children = self.token_node_ids[start..end_exclusive].to_vec();
        let range = self.covering_token_range(start, end_exclusive);
        let id = self.events.emit(SyntaxEvent::Node {
            kind: SurfaceNodeKind::PlaceholderItem,
            range,
            children,
        });
        ParsedItem {
            id,
            next_position: end_exclusive.max(start + 1),
            recovery_nodes: Vec::new(),
        }
    }

    fn recover_unexpected_top_level_tokens(&mut self, position: usize) -> ParsedItem {
        let mut cursor = position;
        let prefixed_head = self
            .item_head_position(position)
            .filter(|head| *head > position);
        while cursor < self.request.tokens.len() {
            let is_same_prefixed_item = prefixed_head.is_some_and(|head| {
                cursor <= head && self.item_head_position(cursor) == Some(head)
            });
            if cursor > position
                && !is_same_prefixed_item
                && self.is_compilation_item_or_statement_start_at(cursor)
            {
                break;
            }
            let is_synchronizing_token =
                self.is_semicolon_at(cursor) || self.is_end_keyword_at(cursor);
            cursor += 1;
            if is_synchronizing_token {
                break;
            }
        }
        let end_exclusive = cursor.max(position + 1);
        let range = self.covering_token_range(position, end_exclusive);
        self.diagnostics.push(
            SyntaxDiagnostic::new(
                SyntaxDiagnosticCode::UnexpectedTopLevelToken,
                "unexpected token while parsing top-level module item",
                self.request.tokens[position].span,
            )
            .with_recovery_note("skip tokens until the next top-level item boundary"),
        );
        self.trivia
            .add_skipped_token_range(range, None, SkippedTokenReason::Recovery);
        let id = self.add_recovery_node(SyntaxRecoveryKind::SkippedToken, range, Vec::new());

        ParsedItem {
            id,
            next_position: end_exclusive,
            recovery_nodes: vec![id],
        }
    }

    fn diagnose_missing_semicolon(&mut self, position: usize) {
        let tokens = &self.request.tokens;
        let cursor = crate::cursor::TokenCursor::at(self.request.source_id, tokens, position);
        self.diagnostics.push(
            expected_token_diagnostic(
                SyntaxDiagnosticCode::MissingSemicolon,
                ExpectedToken::new("`;`"),
                cursor.current(),
                cursor.eof_range(),
                "expected `;` before continuing at the next syntax boundary",
            )
            .with_recovery_note("insert `;` before continuing"),
        );
    }

    fn diagnose_missing_end(&mut self, opener: usize, position: usize) {
        let tokens = &self.request.tokens;
        let cursor = crate::cursor::TokenCursor::at(self.request.source_id, tokens, position);
        let primary = cursor
            .current()
            .map_or(cursor.eof_range(), |token| token.span);
        let opener_span = self.request.tokens[opener].span;
        self.diagnostics.push(
            SyntaxDiagnostic::new(
                SyntaxDiagnosticCode::MissingEnd,
                "expected `end` for block",
                primary,
            )
            .with_secondary([SourceAnchor::Range(opener_span)])
            .with_recovery_note("insert `end` before continuing"),
        );
    }

    fn diagnose_malformed_import(&mut self, position: usize, message: &'static str) {
        let tokens = &self.request.tokens;
        let cursor = crate::cursor::TokenCursor::at(self.request.source_id, tokens, position);
        let primary = cursor
            .current()
            .map_or(cursor.eof_range(), |token| token.span);
        self.diagnostics.push(
            SyntaxDiagnostic::new(SyntaxDiagnosticCode::MalformedImport, message, primary)
                .with_recovery_note("repair the import syntax before continuing"),
        );
    }

    fn diagnose_malformed_export(&mut self, position: usize, message: &'static str) {
        let tokens = &self.request.tokens;
        let cursor = crate::cursor::TokenCursor::at(self.request.source_id, tokens, position);
        let primary = cursor
            .current()
            .map_or(cursor.eof_range(), |token| token.span);
        self.diagnostics.push(
            SyntaxDiagnostic::new(SyntaxDiagnosticCode::MalformedExport, message, primary)
                .with_recovery_note("repair the export syntax before continuing"),
        );
    }

    fn diagnose_malformed_visibility(&mut self, position: usize) {
        let tokens = &self.request.tokens;
        let cursor = crate::cursor::TokenCursor::at(self.request.source_id, tokens, position);
        let primary = cursor
            .current()
            .map_or(cursor.eof_range(), |token| token.span);
        self.diagnostics.push(
            SyntaxDiagnostic::new(
                SyntaxDiagnosticCode::MalformedVisibility,
                "expected theorem or notation item after visibility marker",
                primary,
            )
            .with_recovery_note("skip the malformed visible item tail before continuing"),
        );
    }

    fn diagnose_malformed_type_expression(&mut self, position: usize, message: &'static str) {
        let tokens = &self.request.tokens;
        let cursor = crate::cursor::TokenCursor::at(self.request.source_id, tokens, position);
        let primary = cursor
            .current()
            .map_or(cursor.eof_range(), |token| token.span);
        self.diagnostics.push(
            SyntaxDiagnostic::new(
                SyntaxDiagnosticCode::MalformedTypeExpression,
                message,
                primary,
            )
            .with_recovery_note("repair the type expression before continuing"),
        );
    }

    fn diagnose_malformed_term_expression(
        &mut self,
        position: usize,
        message: impl Into<std::sync::Arc<str>>,
    ) {
        let tokens = &self.request.tokens;
        let cursor = crate::cursor::TokenCursor::at(self.request.source_id, tokens, position);
        let primary = cursor
            .current()
            .map_or(cursor.eof_range(), |token| token.span);
        self.diagnostics.push(
            SyntaxDiagnostic::new(
                SyntaxDiagnosticCode::MalformedTermExpression,
                message,
                primary,
            )
            .with_recovery_note("repair the term expression before continuing"),
        );
    }

    fn diagnose_malformed_formula_expression(
        &mut self,
        position: usize,
        message: impl Into<std::sync::Arc<str>>,
    ) {
        let tokens = &self.request.tokens;
        let cursor = crate::cursor::TokenCursor::at(self.request.source_id, tokens, position);
        let primary = cursor
            .current()
            .map_or(cursor.eof_range(), |token| token.span);
        self.diagnostics.push(
            SyntaxDiagnostic::new(
                SyntaxDiagnosticCode::MalformedFormulaExpression,
                message,
                primary,
            )
            .with_recovery_note("repair the formula expression before continuing"),
        );
    }

    fn diagnose_malformed_justification(
        &mut self,
        position: usize,
        message: impl Into<std::sync::Arc<str>>,
    ) {
        let tokens = &self.request.tokens;
        let cursor = crate::cursor::TokenCursor::at(self.request.source_id, tokens, position);
        let primary = cursor
            .current()
            .map_or(cursor.eof_range(), |token| token.span);
        self.diagnostics.push(
            SyntaxDiagnostic::new(
                SyntaxDiagnosticCode::MalformedJustification,
                message,
                primary,
            )
            .with_recovery_note("repair the justification syntax before continuing"),
        );
    }

    fn diagnose_unmatched_type_argument_opener(&mut self, opener: usize, position: usize) {
        let tokens = &self.request.tokens;
        let cursor = crate::cursor::TokenCursor::at(self.request.source_id, tokens, position);
        let primary = cursor
            .current()
            .map_or(cursor.eof_range(), |token| token.span);
        let opener_span = self.request.tokens[opener].span;
        self.diagnostics.push(
            SyntaxDiagnostic::new(
                SyntaxDiagnosticCode::MalformedTypeExpression,
                "expected `]` to close type arguments",
                primary,
            )
            .with_secondary([SourceAnchor::Range(opener_span)])
            .with_recovery_note("insert `]` before continuing"),
        );
    }

    fn diagnose_unmatched_term_delimiter(
        &mut self,
        opener: usize,
        position: usize,
        close_symbol: &'static str,
    ) {
        let tokens = &self.request.tokens;
        let cursor = crate::cursor::TokenCursor::at(self.request.source_id, tokens, position);
        let primary = cursor
            .current()
            .map_or(cursor.eof_range(), |token| token.span);
        let opener_span = self.request.tokens[opener].span;
        self.diagnostics.push(
            SyntaxDiagnostic::new(
                SyntaxDiagnosticCode::MalformedTermExpression,
                format!("expected `{close_symbol}` to close term delimiter"),
                primary,
            )
            .with_secondary([SourceAnchor::Range(opener_span)])
            .with_recovery_note(format!("insert `{close_symbol}` before continuing")),
        );
    }

    fn diagnose_unmatched_formula_delimiter(
        &mut self,
        opener: usize,
        position: usize,
        close_symbol: &'static str,
    ) {
        let tokens = &self.request.tokens;
        let cursor = crate::cursor::TokenCursor::at(self.request.source_id, tokens, position);
        let primary = cursor
            .current()
            .map_or(cursor.eof_range(), |token| token.span);
        let opener_span = self.request.tokens[opener].span;
        self.diagnostics.push(
            SyntaxDiagnostic::new(
                SyntaxDiagnosticCode::MalformedFormulaExpression,
                format!("expected `{close_symbol}` to close formula delimiter"),
                primary,
            )
            .with_secondary([SourceAnchor::Range(opener_span)])
            .with_recovery_note(format!("insert `{close_symbol}` before continuing")),
        );
    }

    fn diagnose_unmatched_justification_delimiter(
        &mut self,
        opener: usize,
        position: usize,
        close_symbol: &'static str,
    ) {
        let tokens = &self.request.tokens;
        let cursor = crate::cursor::TokenCursor::at(self.request.source_id, tokens, position);
        let primary = cursor
            .current()
            .map_or(cursor.eof_range(), |token| token.span);
        let opener_span = self.request.tokens[opener].span;
        self.diagnostics.push(
            SyntaxDiagnostic::new(
                SyntaxDiagnosticCode::MalformedJustification,
                format!("expected `{close_symbol}` to close justification delimiter"),
                primary,
            )
            .with_secondary([SourceAnchor::Range(opener_span)])
            .with_recovery_note(format!("insert `{close_symbol}` before continuing")),
        );
    }

    fn add_missing_type_expression(&mut self, position: usize) -> SurfaceBuilderNodeId {
        self.add_recovery_node(
            SyntaxRecoveryKind::MissingTypeExpression,
            self.zero_range_at(position),
            Vec::new(),
        )
    }

    fn add_missing_formula(&mut self, position: usize) -> SurfaceBuilderNodeId {
        self.add_recovery_node(
            SyntaxRecoveryKind::MissingFormula,
            self.zero_range_at(position),
            Vec::new(),
        )
    }

    fn add_missing_statement(&mut self, position: usize) -> SurfaceBuilderNodeId {
        self.add_recovery_node(
            SyntaxRecoveryKind::MissingStatement,
            self.zero_range_at(position),
            Vec::new(),
        )
    }

    fn add_missing_end(&mut self, opener: usize, position: usize) -> SurfaceBuilderNodeId {
        if !self.has_missing_end_diagnostic(opener, position) {
            self.diagnose_missing_end(opener, position);
        }
        self.add_recovery_node(
            SyntaxRecoveryKind::MissingEnd,
            self.zero_range_at(position),
            Vec::new(),
        )
    }

    fn has_missing_end_diagnostic(&self, opener: usize, position: usize) -> bool {
        let primary = self.zero_range_at(position);
        let opener_span = self.request.tokens[opener].span;
        self.diagnostics.iter().any(|diagnostic| {
            diagnostic.code == SyntaxDiagnosticCode::MissingEnd
                && diagnostic.primary == primary
                && diagnostic.secondary.iter().any(
                    |anchor| matches!(anchor, SourceAnchor::Range(range) if *range == opener_span),
                )
        })
    }

    fn add_missing_term(&mut self, position: usize) -> SurfaceBuilderNodeId {
        self.add_recovery_node(
            SyntaxRecoveryKind::MissingTerm,
            self.zero_range_at(position),
            Vec::new(),
        )
    }

    fn add_missing_proof_step(&mut self, position: usize) -> SurfaceBuilderNodeId {
        self.add_recovery_node(
            SyntaxRecoveryKind::MissingProofStep,
            self.zero_range_at(position),
            Vec::new(),
        )
    }

    fn push_missing_term(
        &mut self,
        position: usize,
        children: &mut Vec<SurfaceBuilderNodeId>,
        recovery_nodes: &mut Vec<SurfaceBuilderNodeId>,
    ) {
        let recovery = self.add_missing_term(position);
        children.push(recovery);
        recovery_nodes.push(recovery);
    }

    fn push_missing_proof_step(
        &mut self,
        position: usize,
        children: &mut Vec<SurfaceBuilderNodeId>,
        recovery_nodes: &mut Vec<SurfaceBuilderNodeId>,
    ) {
        let recovery = self.add_missing_proof_step(position);
        children.push(recovery);
        recovery_nodes.push(recovery);
    }

    fn recover_malformed_type_tail(&mut self, position: usize) -> Option<ParsedItem> {
        let mut cursor = position;
        while cursor < self.request.tokens.len()
            && !self.is_semicolon_at(cursor)
            && !self.is_reserved_symbol_at(cursor, ",")
            && !self.is_reserved_symbol_at(cursor, "]")
            && !self.is_reserved_symbol_at(cursor, ")")
            && !self.is_item_start_at(cursor)
        {
            cursor += 1;
        }
        self.emit_malformed_tail_recovery(position, cursor)
    }

    fn recover_malformed_term_tail(
        &mut self,
        position: usize,
        close_symbol: &str,
    ) -> Option<ParsedItem> {
        let mut cursor = position;
        while cursor < self.request.tokens.len()
            && !self.is_semicolon_at(cursor)
            && !self.is_reserved_symbol_at(cursor, ",")
            && !self.is_reserved_symbol_at(cursor, ")")
            && !self.is_reserved_symbol_at(cursor, "]")
            && !self.is_reserved_symbol_at(cursor, "}")
            && (close_symbol.is_empty() || !self.is_reserved_symbol_at(cursor, close_symbol))
            && !self.is_item_start_at(cursor)
        {
            cursor += 1;
        }
        self.emit_malformed_tail_recovery(position, cursor)
    }

    fn recover_malformed_comprehension_generator_tail(
        &mut self,
        position: usize,
    ) -> Option<ParsedItem> {
        let mut cursor = position;
        while cursor < self.request.tokens.len()
            && !self.is_semicolon_at(cursor)
            && !self.is_reserved_symbol_at(cursor, ",")
            && !self.is_reserved_symbol_at(cursor, ":")
            && !self.is_reserved_symbol_at(cursor, "}")
            && !self.is_reserved_symbol_at(cursor, ")")
            && !self.is_reserved_symbol_at(cursor, "]")
            && !self.is_item_start_at(cursor)
        {
            cursor += 1;
        }
        self.emit_malformed_tail_recovery(position, cursor)
    }

    fn recover_malformed_set_comprehension_mapper_tail(
        &mut self,
        position: usize,
    ) -> Option<ParsedItem> {
        let mut cursor = position;
        let mut paren_depth = 0_usize;
        let mut bracket_depth = 0_usize;
        let mut brace_depth = 0_usize;

        while cursor < self.request.tokens.len() {
            let top_level = paren_depth == 0 && bracket_depth == 0 && brace_depth == 0;
            if top_level
                && (self.is_reserved_word_at(cursor, "where")
                    || self.is_reserved_symbol_at(cursor, "}")
                    || self.is_semicolon_at(cursor)
                    || self.is_item_start_at(cursor))
            {
                break;
            }

            if self.is_reserved_symbol_at(cursor, "(") {
                paren_depth += 1;
            } else if self.is_reserved_symbol_at(cursor, ")") {
                if paren_depth == 0 {
                    break;
                }
                paren_depth -= 1;
            } else if self.is_reserved_symbol_at(cursor, "[") {
                bracket_depth += 1;
            } else if self.is_reserved_symbol_at(cursor, "]") {
                if bracket_depth == 0 {
                    break;
                }
                bracket_depth -= 1;
            } else if self.is_reserved_symbol_at(cursor, "{") {
                brace_depth += 1;
            } else if self.is_reserved_symbol_at(cursor, "}") {
                if brace_depth == 0 {
                    break;
                }
                brace_depth -= 1;
            }
            cursor += 1;
        }

        self.emit_malformed_tail_recovery(position, cursor)
    }

    fn recover_malformed_tail(&mut self, position: usize) -> Option<ParsedItem> {
        let mut cursor = position;
        while cursor < self.request.tokens.len() && !self.is_semicolon_at(cursor) {
            cursor += 1;
        }
        self.emit_malformed_tail_recovery(position, cursor)
    }

    fn recover_malformed_statement_tail(&mut self, position: usize) -> Option<ParsedItem> {
        let mut cursor = position;
        let mut paren_depth = 0_usize;
        let mut bracket_depth = 0_usize;
        let mut brace_depth = 0_usize;

        while cursor < self.request.tokens.len() {
            let top_level = paren_depth == 0 && bracket_depth == 0 && brace_depth == 0;
            if top_level
                && (self.is_semicolon_at(cursor)
                    || self.is_end_keyword_at(cursor)
                    || self.is_case_branch_keyword_at(cursor)
                    || self.is_compilation_item_or_statement_start_at(cursor))
            {
                break;
            }

            if self.is_reserved_symbol_at(cursor, "(") {
                paren_depth += 1;
            } else if self.is_reserved_symbol_at(cursor, ")") {
                if paren_depth == 0 {
                    break;
                }
                paren_depth -= 1;
            } else if self.is_reserved_symbol_at(cursor, "[") {
                bracket_depth += 1;
            } else if self.is_reserved_symbol_at(cursor, "]") {
                if bracket_depth == 0 {
                    break;
                }
                bracket_depth -= 1;
            } else if self.is_reserved_symbol_at(cursor, "{") {
                brace_depth += 1;
            } else if self.is_reserved_symbol_at(cursor, "}") {
                if brace_depth == 0 {
                    break;
                }
                brace_depth -= 1;
            }
            cursor += 1;
        }

        self.emit_malformed_tail_recovery(position, cursor)
    }

    fn recover_malformed_theorem_tail(&mut self, position: usize) -> Option<ParsedItem> {
        let mut cursor = position;
        let mut paren_depth = 0_usize;
        let mut bracket_depth = 0_usize;
        let mut brace_depth = 0_usize;

        while cursor < self.request.tokens.len() {
            let top_level = paren_depth == 0 && bracket_depth == 0 && brace_depth == 0;
            if top_level
                && (self.is_semicolon_at(cursor)
                    || self.is_end_keyword_at(cursor)
                    || self.is_case_branch_keyword_at(cursor)
                    || self.is_compilation_item_or_statement_start_at(cursor))
            {
                break;
            }

            if self.is_reserved_symbol_at(cursor, "(") {
                paren_depth += 1;
            } else if self.is_reserved_symbol_at(cursor, ")") {
                if paren_depth == 0 {
                    break;
                }
                paren_depth -= 1;
            } else if self.is_reserved_symbol_at(cursor, "[") {
                bracket_depth += 1;
            } else if self.is_reserved_symbol_at(cursor, "]") {
                if bracket_depth == 0 {
                    break;
                }
                bracket_depth -= 1;
            } else if self.is_reserved_symbol_at(cursor, "{") {
                brace_depth += 1;
            } else if self.is_reserved_symbol_at(cursor, "}") {
                if brace_depth == 0 {
                    break;
                }
                brace_depth -= 1;
            }
            cursor += 1;
        }

        self.emit_malformed_tail_recovery_without_children(position, cursor)
    }

    fn recover_malformed_definition_content_tail(&mut self, position: usize) -> Option<ParsedItem> {
        let mut cursor = position;
        let mut paren_depth = 0_usize;
        let mut bracket_depth = 0_usize;
        let mut brace_depth = 0_usize;

        while cursor < self.request.tokens.len() {
            let top_level = paren_depth == 0 && bracket_depth == 0 && brace_depth == 0;
            if top_level && self.is_definition_content_synchronization_boundary_at(cursor) {
                break;
            }

            if self.is_reserved_symbol_at(cursor, "(") {
                paren_depth += 1;
            } else if self.is_reserved_symbol_at(cursor, ")") {
                if paren_depth == 0 {
                    break;
                }
                paren_depth -= 1;
            } else if self.is_reserved_symbol_at(cursor, "[") {
                bracket_depth += 1;
            } else if self.is_reserved_symbol_at(cursor, "]") {
                if bracket_depth == 0 {
                    break;
                }
                bracket_depth -= 1;
            } else if self.is_reserved_symbol_at(cursor, "{") {
                brace_depth += 1;
            } else if self.is_reserved_symbol_at(cursor, "}") {
                if brace_depth == 0 {
                    break;
                }
                brace_depth -= 1;
            }
            cursor += 1;
        }

        self.emit_malformed_tail_recovery(position, cursor)
    }

    fn recover_malformed_structure_member_tail(&mut self, position: usize) -> Option<ParsedItem> {
        let mut cursor = position;
        let mut paren_depth = 0_usize;
        let mut bracket_depth = 0_usize;
        let mut brace_depth = 0_usize;

        while cursor < self.request.tokens.len() {
            let top_level = paren_depth == 0 && bracket_depth == 0 && brace_depth == 0;
            if top_level && self.is_structure_member_boundary_at(cursor) {
                break;
            }

            if self.is_reserved_symbol_at(cursor, "(") {
                paren_depth += 1;
            } else if self.is_reserved_symbol_at(cursor, ")") {
                if paren_depth == 0 {
                    break;
                }
                paren_depth -= 1;
            } else if self.is_reserved_symbol_at(cursor, "[") {
                bracket_depth += 1;
            } else if self.is_reserved_symbol_at(cursor, "]") {
                if bracket_depth == 0 {
                    break;
                }
                bracket_depth -= 1;
            } else if self.is_reserved_symbol_at(cursor, "{") {
                brace_depth += 1;
            } else if self.is_reserved_symbol_at(cursor, "}") {
                if brace_depth == 0 {
                    break;
                }
                brace_depth -= 1;
            }
            cursor += 1;
        }

        self.emit_malformed_tail_recovery(position, cursor)
    }

    fn recover_malformed_inheritance_member_tail(&mut self, position: usize) -> Option<ParsedItem> {
        let mut cursor = position;
        let mut paren_depth = 0_usize;
        let mut bracket_depth = 0_usize;
        let mut brace_depth = 0_usize;

        while cursor < self.request.tokens.len() {
            let top_level = paren_depth == 0 && bracket_depth == 0 && brace_depth == 0;
            if top_level && self.is_inheritance_member_boundary_at(cursor) {
                break;
            }

            if self.is_reserved_symbol_at(cursor, "(") {
                paren_depth += 1;
            } else if self.is_reserved_symbol_at(cursor, ")") {
                if paren_depth == 0 {
                    break;
                }
                paren_depth -= 1;
            } else if self.is_reserved_symbol_at(cursor, "[") {
                bracket_depth += 1;
            } else if self.is_reserved_symbol_at(cursor, "]") {
                if bracket_depth == 0 {
                    break;
                }
                bracket_depth -= 1;
            } else if self.is_reserved_symbol_at(cursor, "{") {
                brace_depth += 1;
            } else if self.is_reserved_symbol_at(cursor, "}") {
                if brace_depth == 0 {
                    break;
                }
                brace_depth -= 1;
            }
            cursor += 1;
        }

        self.emit_malformed_tail_recovery(position, cursor)
    }

    fn recover_malformed_inline_definition_tail(&mut self, position: usize) -> Option<ParsedItem> {
        let mut cursor = position;
        let mut paren_depth = 0_usize;
        let mut bracket_depth = 0_usize;
        let mut brace_depth = 0_usize;

        while cursor < self.request.tokens.len() {
            let top_level = paren_depth == 0 && bracket_depth == 0 && brace_depth == 0;
            if top_level && self.is_inline_definition_parameter_boundary_at(cursor) {
                break;
            }

            if self.is_reserved_symbol_at(cursor, "(") {
                paren_depth += 1;
            } else if self.is_reserved_symbol_at(cursor, ")") {
                if paren_depth == 0 {
                    break;
                }
                paren_depth -= 1;
            } else if self.is_reserved_symbol_at(cursor, "[") {
                bracket_depth += 1;
            } else if self.is_reserved_symbol_at(cursor, "]") {
                if bracket_depth == 0 {
                    break;
                }
                bracket_depth -= 1;
            } else if self.is_reserved_symbol_at(cursor, "{") {
                brace_depth += 1;
            } else if self.is_reserved_symbol_at(cursor, "}") {
                if brace_depth == 0 {
                    break;
                }
                brace_depth -= 1;
            }
            cursor += 1;
        }

        self.emit_malformed_tail_recovery(position, cursor)
    }

    fn recover_malformed_reconsider_item_tail(&mut self, position: usize) -> Option<ParsedItem> {
        let mut cursor = position;
        let mut paren_depth = 0_usize;
        let mut bracket_depth = 0_usize;
        let mut brace_depth = 0_usize;

        while cursor < self.request.tokens.len() {
            let top_level = paren_depth == 0 && bracket_depth == 0 && brace_depth == 0;
            if top_level && self.is_reconsider_item_boundary_at(cursor) {
                break;
            }

            if self.is_reserved_symbol_at(cursor, "(") {
                paren_depth += 1;
            } else if self.is_reserved_symbol_at(cursor, ")") {
                if paren_depth == 0 {
                    break;
                }
                paren_depth -= 1;
            } else if self.is_reserved_symbol_at(cursor, "[") {
                bracket_depth += 1;
            } else if self.is_reserved_symbol_at(cursor, "]") {
                if bracket_depth == 0 {
                    break;
                }
                bracket_depth -= 1;
            } else if self.is_reserved_symbol_at(cursor, "{") {
                brace_depth += 1;
            } else if self.is_reserved_symbol_at(cursor, "}") {
                if brace_depth == 0 {
                    break;
                }
                brace_depth -= 1;
            }
            cursor += 1;
        }

        self.emit_malformed_tail_recovery(position, cursor)
    }

    fn recover_malformed_justification_tail(&mut self, position: usize) -> Option<ParsedItem> {
        self.recover_malformed_justification_tail_with_definition_boundary(position, false)
    }

    fn recover_malformed_justification_tail_with_definition_boundary(
        &mut self,
        position: usize,
        stop_at_definition_content_start: bool,
    ) -> Option<ParsedItem> {
        let mut cursor = position;
        let mut paren_depth = 0_usize;
        let mut bracket_depth = 0_usize;
        let mut brace_depth = 0_usize;

        while cursor < self.request.tokens.len() {
            let top_level = paren_depth == 0 && bracket_depth == 0 && brace_depth == 0;
            if top_level
                && self
                    .is_justification_recovery_boundary_at(cursor, stop_at_definition_content_start)
            {
                break;
            }

            if self.is_reserved_symbol_at(cursor, "(") {
                paren_depth += 1;
            } else if self.is_reserved_symbol_at(cursor, ")") {
                if paren_depth == 0 {
                    break;
                }
                paren_depth -= 1;
            } else if self.is_reserved_symbol_at(cursor, "[") {
                bracket_depth += 1;
            } else if self.is_reserved_symbol_at(cursor, "]") {
                if bracket_depth == 0 {
                    break;
                }
                bracket_depth -= 1;
            } else if self.is_reserved_symbol_at(cursor, "{") {
                brace_depth += 1;
            } else if self.is_reserved_symbol_at(cursor, "}") {
                if brace_depth == 0 {
                    break;
                }
                brace_depth -= 1;
            }
            cursor += 1;
        }

        self.emit_malformed_tail_recovery(position, cursor)
    }

    fn recover_deferred_reference_template_tail(
        &mut self,
        cursor: usize,
        children: &mut Vec<SurfaceBuilderNodeId>,
        recovery_nodes: &mut Vec<SurfaceBuilderNodeId>,
    ) -> usize {
        if !self.is_reserved_symbol_at(cursor, "[") {
            return cursor;
        }
        self.diagnose_malformed_justification(
            cursor,
            "reference template arguments are not parsed yet",
        );
        if let Some(recovery) = self.recover_malformed_justification_tail(cursor) {
            let next_position = recovery.next_position;
            children.push(recovery.id);
            recovery_nodes.extend(recovery.recovery_nodes);
            next_position
        } else {
            cursor
        }
    }

    fn recover_unexpected_justification_tail_with_definition_boundary(
        &mut self,
        cursor: usize,
        children: &mut Vec<SurfaceBuilderNodeId>,
        recovery_nodes: &mut Vec<SurfaceBuilderNodeId>,
        stop_at_definition_content_start: bool,
    ) -> usize {
        if self.is_justification_recovery_boundary_at(cursor, stop_at_definition_content_start) {
            return cursor;
        }
        self.diagnose_malformed_justification(cursor, "unexpected token in justification");
        if let Some(recovery) = self.recover_malformed_justification_tail_with_definition_boundary(
            cursor,
            stop_at_definition_content_start,
        ) {
            let next_position = recovery.next_position;
            children.push(recovery.id);
            recovery_nodes.extend(recovery.recovery_nodes);
            next_position
        } else {
            cursor
        }
    }

    fn recover_malformed_visibility_tail(&mut self, position: usize) -> Option<ParsedItem> {
        let cursor = if let Some(block_head) = self.malformed_visibility_block_head(position) {
            self.block_like_tail_semicolon_position(block_head)
        } else {
            let mut cursor = position;
            while cursor < self.request.tokens.len() && !self.is_semicolon_at(cursor) {
                cursor += 1;
            }
            cursor
        };
        self.emit_malformed_tail_recovery(position, cursor)
    }

    fn malformed_visibility_block_head(&self, position: usize) -> Option<usize> {
        let mut cursor = position;
        while cursor < self.request.tokens.len() && self.is_item_prefix_keyword_at(cursor) {
            cursor += 1;
        }
        self.request
            .tokens
            .get(cursor)
            .is_some_and(is_block_like_top_level_start)
            .then_some(cursor)
    }

    fn block_like_tail_semicolon_position(&self, head: usize) -> usize {
        let mut cursor = head + 1;
        let mut block_depth = 1_usize;
        while cursor < self.request.tokens.len() {
            if self.is_end_keyword_at(cursor) {
                block_depth -= 1;
                if block_depth == 0 {
                    let semicolon = cursor + 1;
                    return if semicolon < self.request.tokens.len()
                        && self.is_semicolon_at(semicolon)
                    {
                        semicolon
                    } else {
                        semicolon.min(self.request.tokens.len())
                    };
                }
            } else if sync::opens_recovery_block_at(&self.request.tokens, cursor) {
                block_depth += 1;
            }
            cursor += 1;
        }
        self.request.tokens.len()
    }

    fn emit_malformed_tail_recovery(
        &mut self,
        position: usize,
        cursor: usize,
    ) -> Option<ParsedItem> {
        if cursor == position {
            return None;
        }
        let range = self.covering_token_range(position, cursor);
        self.trivia
            .add_skipped_token_range(range, None, SkippedTokenReason::Recovery);
        let children = self.token_node_ids[position..cursor].to_vec();
        let id = self.add_recovery_node(SyntaxRecoveryKind::SkippedToken, range, children);
        Some(ParsedItem {
            id,
            next_position: cursor,
            recovery_nodes: vec![id],
        })
    }

    fn emit_malformed_tail_recovery_without_children(
        &mut self,
        position: usize,
        cursor: usize,
    ) -> Option<ParsedItem> {
        if cursor == position {
            return None;
        }
        let range = self.covering_token_range(position, cursor);
        self.trivia
            .add_skipped_token_range(range, None, SkippedTokenReason::Recovery);
        let id = self.add_recovery_node(SyntaxRecoveryKind::SkippedToken, range, Vec::new());
        Some(ParsedItem {
            id,
            next_position: cursor,
            recovery_nodes: vec![id],
        })
    }

    fn should_parse_module_skeleton(&self) -> bool {
        self.first_item_start_position().is_some_and(|first_start| {
            first_start == 0
                || !(0..first_start)
                    .any(|position| sync::opens_recovery_block_at(&self.request.tokens, position))
        })
    }

    fn first_item_start_position(&self) -> Option<usize> {
        let mut position = 0;
        while position < self.request.tokens.len() {
            if self.is_compilation_item_or_statement_start_at(position) {
                return Some(position);
            }
            let next = self.skip_annotations(position);
            position = if next > position { next } else { position + 1 };
        }
        None
    }

    fn is_item_start_at(&self, position: usize) -> bool {
        self.item_head_position(position).is_some()
    }

    fn is_compilation_item_or_statement_start_at(&self, position: usize) -> bool {
        self.is_item_start_at(position) || self.is_statement_start_at(position)
    }

    fn is_statement_synchronization_boundary_at(&self, position: usize) -> bool {
        position >= self.request.tokens.len()
            || self.is_semicolon_at(position)
            || self.is_end_keyword_at(position)
            || self.is_compilation_item_or_statement_start_at(position)
    }

    fn is_statement_start_at(&self, position: usize) -> bool {
        self.is_reserved_word_at(position, "then")
            || self.is_conclusion_keyword_at(position)
            || self.is_now_statement_start_at(position)
            || self.is_reserved_word_at(position, "hereby")
            || self.is_case_reasoning_start_at(position)
            || self.is_iterative_equality_statement_start_at(position)
            || self.is_simple_statement_keyword_at(position)
            || self.is_compact_statement_start_at(position)
    }

    fn is_statement_boundary_keyword_at(&self, position: usize) -> bool {
        self.request.tokens.get(position).is_some_and(|token| {
            token.kind == ParserTokenKind::ReservedWord
                && matches!(
                    token.text.as_ref(),
                    "then"
                        | "thus"
                        | "hence"
                        | "now"
                        | "hereby"
                        | "per"
                        | "let"
                        | "assume"
                        | "given"
                        | "take"
                        | "set"
                        | "consider"
                        | "reconsider"
                        | "deffunc"
                        | "defpred"
                )
        })
    }

    fn is_simple_statement_keyword_at(&self, position: usize) -> bool {
        self.request.tokens.get(position).is_some_and(|token| {
            token.kind == ParserTokenKind::ReservedWord
                && matches!(
                    token.text.as_ref(),
                    "let"
                        | "assume"
                        | "given"
                        | "take"
                        | "set"
                        | "consider"
                        | "reconsider"
                        | "deffunc"
                        | "defpred"
                )
        })
    }

    fn is_inline_definition_delimiter_at(&self, position: usize) -> bool {
        self.is_reserved_symbol_at(position, "->")
            || self.is_reserved_word_at(position, "equals")
            || self.is_reserved_word_at(position, "means")
    }

    fn is_inline_definition_parameter_boundary_at(&self, position: usize) -> bool {
        position >= self.request.tokens.len()
            || self.is_semicolon_at(position)
            || self.is_reserved_symbol_at(position, ",")
            || self.is_reserved_symbol_at(position, ")")
            || self.is_inline_definition_delimiter_at(position)
            || self.is_end_keyword_at(position)
            || self.is_case_branch_keyword_at(position)
            || self.is_item_start_at(position)
            || self.is_statement_boundary_keyword_at(position)
    }

    fn is_conclusion_keyword_at(&self, position: usize) -> bool {
        self.request.tokens.get(position).is_some_and(|token| {
            token.kind == ParserTokenKind::ReservedWord
                && matches!(token.text.as_ref(), "thus" | "hence")
        })
    }

    fn is_case_reasoning_start_at(&self, position: usize) -> bool {
        self.is_reserved_word_at(position, "per") && self.is_reserved_word_at(position + 1, "cases")
    }

    fn is_now_statement_start_at(&self, position: usize) -> bool {
        self.is_reserved_word_at(position, "now")
            || (self.is_identifier_at(position)
                && self.is_reserved_symbol_at(position + 1, ":")
                && self.is_reserved_word_at(position + 2, "now"))
    }

    fn is_case_branch_keyword_at(&self, position: usize) -> bool {
        self.is_reserved_word_at(position, "case") || self.is_reserved_word_at(position, "suppose")
    }

    fn is_iterative_equality_statement_start_at(&self, position: usize) -> bool {
        let body_start = self.statement_label_body_start_at(position);
        if !self.can_start_term_operator_operand_at(body_start, false) {
            return false;
        }
        let Some(equals) = self.top_level_symbol_before_statement_boundary_at(body_start, "=")
        else {
            return false;
        };
        self.top_level_symbol_before_statement_boundary_at(equals + 1, ".=")
            .is_some()
    }

    fn statement_label_body_start_at(&self, position: usize) -> usize {
        if self.is_identifier_at(position) && self.is_reserved_symbol_at(position + 1, ":") {
            position + 2
        } else {
            position
        }
    }

    fn is_compact_statement_start_at(&self, position: usize) -> bool {
        let body_start = self.statement_label_body_start_at(position);
        self.can_start_formula_at(body_start)
            && self.top_level_general_justification_before_statement_boundary_at(body_start)
    }

    fn top_level_general_justification_before_statement_boundary_at(
        &self,
        position: usize,
    ) -> bool {
        let mut cursor = position;
        let mut paren_depth = 0_usize;
        let mut bracket_depth = 0_usize;
        let mut brace_depth = 0_usize;

        while cursor < self.request.tokens.len() {
            let top_level = paren_depth == 0 && bracket_depth == 0 && brace_depth == 0;
            if top_level {
                if cursor > position
                    && (self.is_reserved_word_at(cursor, "by")
                        || self.is_reserved_word_at(cursor, "proof"))
                {
                    return true;
                }
                if self.is_semicolon_at(cursor)
                    || self.is_end_keyword_at(cursor)
                    || self.is_item_start_at(cursor)
                    || (cursor > position && self.is_statement_boundary_keyword_at(cursor))
                {
                    return false;
                }
            }

            if self.is_reserved_symbol_at(cursor, "(") {
                paren_depth += 1;
            } else if self.is_reserved_symbol_at(cursor, ")") {
                if paren_depth == 0 {
                    return false;
                }
                paren_depth -= 1;
            } else if self.is_reserved_symbol_at(cursor, "[") {
                bracket_depth += 1;
            } else if self.is_reserved_symbol_at(cursor, "]") {
                if bracket_depth == 0 {
                    return false;
                }
                bracket_depth -= 1;
            } else if self.is_reserved_symbol_at(cursor, "{") {
                brace_depth += 1;
            } else if self.is_reserved_symbol_at(cursor, "}") {
                if brace_depth == 0 {
                    return false;
                }
                brace_depth -= 1;
            }
            cursor += 1;
        }
        false
    }

    fn definition_parameter_tail_justification_position_at(
        &self,
        position: usize,
    ) -> Option<usize> {
        let mut cursor = position;
        let mut paren_depth = 0_usize;
        let mut bracket_depth = 0_usize;
        let mut brace_depth = 0_usize;

        while cursor < self.request.tokens.len() {
            let top_level = paren_depth == 0 && bracket_depth == 0 && brace_depth == 0;
            if top_level {
                if self.is_reserved_word_at(cursor, "by")
                    || self.is_reserved_word_at(cursor, "proof")
                {
                    return Some(cursor);
                }
                if self.is_semicolon_at(cursor)
                    || self.is_end_keyword_at(cursor)
                    || (cursor > position && self.is_definition_content_start_at(cursor))
                {
                    return None;
                }
            }

            if self.is_reserved_symbol_at(cursor, "(") {
                paren_depth += 1;
            } else if self.is_reserved_symbol_at(cursor, ")") {
                if paren_depth == 0 {
                    return None;
                }
                paren_depth -= 1;
            } else if self.is_reserved_symbol_at(cursor, "[") {
                bracket_depth += 1;
            } else if self.is_reserved_symbol_at(cursor, "]") {
                if bracket_depth == 0 {
                    return None;
                }
                bracket_depth -= 1;
            } else if self.is_reserved_symbol_at(cursor, "{") {
                brace_depth += 1;
            } else if self.is_reserved_symbol_at(cursor, "}") {
                if brace_depth == 0 {
                    return None;
                }
                brace_depth -= 1;
            }
            cursor += 1;
        }
        None
    }

    fn top_level_symbol_before_statement_boundary_at(
        &self,
        position: usize,
        symbol: &str,
    ) -> Option<usize> {
        let mut cursor = position;
        let mut paren_depth = 0_usize;
        let mut bracket_depth = 0_usize;
        let mut brace_depth = 0_usize;

        while cursor < self.request.tokens.len() {
            let top_level = paren_depth == 0 && bracket_depth == 0 && brace_depth == 0;
            if top_level {
                if self.is_reserved_symbol_at(cursor, symbol) {
                    return Some(cursor);
                }
                if cursor > position
                    && (self.is_semicolon_at(cursor)
                        || self.is_end_keyword_at(cursor)
                        || self.is_item_start_at(cursor)
                        || self.is_statement_boundary_keyword_at(cursor))
                {
                    return None;
                }
            }

            if self.is_reserved_symbol_at(cursor, "(") {
                paren_depth += 1;
            } else if self.is_reserved_symbol_at(cursor, ")") {
                if paren_depth == 0 {
                    return None;
                }
                paren_depth -= 1;
            } else if self.is_reserved_symbol_at(cursor, "[") {
                bracket_depth += 1;
            } else if self.is_reserved_symbol_at(cursor, "]") {
                if bracket_depth == 0 {
                    return None;
                }
                bracket_depth -= 1;
            } else if self.is_reserved_symbol_at(cursor, "{") {
                brace_depth += 1;
            } else if self.is_reserved_symbol_at(cursor, "}") {
                if brace_depth == 0 {
                    return None;
                }
                brace_depth -= 1;
            }
            cursor += 1;
        }
        None
    }

    fn is_let_type_set_keyword_at(&self, let_position: usize, position: usize) -> bool {
        if !self.is_reserved_word_at(position, "set") || position <= let_position {
            return false;
        }
        let mut cursor = let_position + 1;
        while cursor < position {
            if (self.is_reserved_word_at(cursor, "be") || self.is_reserved_word_at(cursor, "being"))
                && cursor + 1 == position
            {
                return true;
            }
            cursor += 1;
        }
        false
    }

    fn is_visibility_marker_at(&self, position: usize) -> bool {
        self.request.tokens.get(position).is_some_and(|token| {
            token.kind == ParserTokenKind::ReservedWord
                && matches!(token.text.as_ref(), "private" | "public")
        })
    }

    fn theorem_role_position_at(&self, head: usize) -> Option<usize> {
        if self.is_reserved_word_at(head, "theorem") || self.is_reserved_word_at(head, "lemma") {
            Some(head)
        } else if self.is_theorem_status_keyword_at(head)
            && (self.is_reserved_word_at(head + 1, "theorem")
                || self.is_reserved_word_at(head + 1, "lemma"))
        {
            Some(head + 1)
        } else {
            None
        }
    }

    fn looks_like_theorem_item_after_role(&self, role: usize) -> bool {
        let label_or_colon = role + 1;
        if self.is_reserved_symbol_at(label_or_colon, ":")
            || self.can_start_formula_at(label_or_colon)
        {
            return true;
        }
        self.is_identifier_at(label_or_colon)
            && (self.is_reserved_symbol_at(label_or_colon + 1, ":")
                || self.can_start_formula_at(label_or_colon + 1)
                || self.is_reserved_word_at(label_or_colon + 1, "by")
                || self.is_reserved_word_at(label_or_colon + 1, "proof"))
    }

    fn theorem_formula_start_after_role(&self, role: usize) -> Option<usize> {
        let label_or_colon = role + 1;
        if self.is_reserved_symbol_at(label_or_colon, ":") {
            Some(label_or_colon + 1)
        } else if self.is_identifier_at(label_or_colon)
            && self.is_reserved_symbol_at(label_or_colon + 1, ":")
        {
            Some(label_or_colon + 2)
        } else {
            None
        }
    }

    fn is_theorem_status_keyword_at(&self, position: usize) -> bool {
        self.request.tokens.get(position).is_some_and(|token| {
            token.kind == ParserTokenKind::ReservedWord
                && matches!(token.text.as_ref(), "open" | "assumed" | "conditional")
        })
    }

    fn is_visibility_target_start_at(&self, position: usize) -> bool {
        let Some(token) = self.request.tokens.get(position) else {
            return false;
        };
        if token.kind != ParserTokenKind::ReservedWord {
            return false;
        }
        match token.text.as_ref() {
            "theorem" | "lemma" | "infix_operator" | "prefix_operator" | "postfix_operator"
            | "synonym" | "antonym" => true,
            "open" | "assumed" | "conditional" => {
                self.request.tokens.get(position + 1).is_some_and(|next| {
                    next.kind == ParserTokenKind::ReservedWord
                        && matches!(next.text.as_ref(), "theorem" | "lemma")
                })
            }
            _ => false,
        }
    }

    fn is_supported_redefinition_start_at(&self, position: usize) -> bool {
        self.is_reserved_word_at(position, "redefine")
            && (self.is_reserved_word_at(position + 1, "attr")
                || self.is_reserved_word_at(position + 1, "pred")
                || self.is_reserved_word_at(position + 1, "func"))
    }

    fn is_notation_alias_start_at(&self, position: usize) -> bool {
        self.is_reserved_word_at(position, "synonym")
            || self.is_reserved_word_at(position, "antonym")
    }

    fn is_property_clause_keyword_at(&self, position: usize) -> bool {
        self.request.tokens.get(position).is_some_and(|token| {
            token.kind == ParserTokenKind::ReservedWord
                && matches!(
                    token.text.as_ref(),
                    "symmetry"
                        | "asymmetry"
                        | "connectedness"
                        | "reflexivity"
                        | "irreflexivity"
                        | "commutativity"
                        | "idempotence"
                        | "involutiveness"
                        | "projectivity"
                        | "sethood"
                )
        })
    }

    fn is_visible_theorem_target_start_at(&self, position: usize) -> bool {
        self.is_reserved_word_at(position, "theorem")
            || self.is_reserved_word_at(position, "lemma")
            || (self.is_theorem_status_keyword_at(position)
                && (self.is_reserved_word_at(position + 1, "theorem")
                    || self.is_reserved_word_at(position + 1, "lemma")))
    }

    fn is_identifier_at(&self, position: usize) -> bool {
        self.request
            .tokens
            .get(position)
            .is_some_and(|token| token.kind == ParserTokenKind::Identifier)
    }

    fn is_user_symbol_at(&self, position: usize) -> bool {
        self.request
            .tokens
            .get(position)
            .is_some_and(|token| token.kind == ParserTokenKind::UserSymbol)
    }

    fn is_numeral_at(&self, position: usize) -> bool {
        self.request
            .tokens
            .get(position)
            .is_some_and(|token| token.kind == ParserTokenKind::Numeral)
    }

    fn is_computation_option_keyword_at(&self, position: usize) -> bool {
        self.request.tokens.get(position).is_some_and(|token| {
            matches!(
                token.kind,
                ParserTokenKind::Identifier | ParserTokenKind::ReservedWord
            ) && matches!(token.text.as_ref(), "steps" | "timeout" | "nest")
        })
    }

    fn qualified_reference_plan_at(&self, position: usize) -> Option<QualifiedReferencePlan> {
        if !self.is_identifier_at(position) {
            return None;
        }
        let mut segments = vec![position];
        let mut separators = Vec::new();
        let mut cursor = position + 1;
        while self.is_reserved_symbol_at(cursor, ".") && self.is_identifier_at(cursor + 1) {
            separators.push(cursor);
            segments.push(cursor + 1);
            cursor += 2;
        }
        if segments.len() < 2 {
            return None;
        }
        let final_reference = segments[segments.len() - 1];
        let final_separator = separators[separators.len() - 1];
        let namespace_segments = segments[..segments.len() - 1].to_vec();
        let namespace_separators = separators[..separators.len() - 1].to_vec();
        Some(QualifiedReferencePlan {
            namespace_segments,
            namespace_separators,
            final_separator,
            final_reference,
            next_position: cursor,
        })
    }

    fn compound_reference_plan_at(
        &self,
        position: usize,
        operator: &'static str,
    ) -> Option<CompoundReferencePlan> {
        if !self.is_identifier_at(position) {
            return None;
        }
        let mut namespace_segments = vec![position];
        let mut namespace_separators = Vec::new();
        let mut cursor = position + 1;
        loop {
            if self.is_reserved_symbol_at(cursor, operator) {
                return Some(CompoundReferencePlan {
                    namespace_segments,
                    namespace_separators,
                    operator: cursor,
                });
            }
            if self.is_reserved_symbol_at(cursor, ".") && self.is_identifier_at(cursor + 1) {
                namespace_separators.push(cursor);
                namespace_segments.push(cursor + 1);
                cursor += 2;
                continue;
            }
            return None;
        }
    }

    fn is_justification_recovery_boundary_at(
        &self,
        position: usize,
        stop_at_definition_content_start: bool,
    ) -> bool {
        position >= self.request.tokens.len()
            || self.is_semicolon_at(position)
            || self.is_end_keyword_at(position)
            || self.is_reserved_symbol_at(position, ",")
            || self.is_reserved_symbol_at(position, ")")
            || self.is_reserved_symbol_at(position, "]")
            || self.is_reserved_symbol_at(position, "}")
            || self.is_reserved_symbol_at(position, ".=")
            || self.is_item_start_at(position)
            || self.is_case_branch_keyword_at(position)
            || self.is_statement_boundary_keyword_at(position)
            || (stop_at_definition_content_start && self.is_definition_content_start_at(position))
    }

    fn is_theorem_item_tail_boundary_at(&self, position: usize) -> bool {
        position >= self.request.tokens.len()
            || self.is_semicolon_at(position)
            || self.is_reserved_word_at(position, "by")
            || self.is_reserved_word_at(position, "proof")
            || self.is_end_keyword_at(position)
            || self.is_compilation_item_or_statement_start_at(position)
            || self.is_case_branch_keyword_at(position)
    }

    fn definition_parameter_starts_template_ambiguous_at(&self, position: usize) -> bool {
        self.definition_parameter_template_keyword_position_at(position)
            .is_some()
    }

    fn definition_parameter_template_keyword_position_at(&self, position: usize) -> Option<usize> {
        if !self.is_reserved_word_at(position, "let") {
            return None;
        }

        let mut cursor = position + 1;
        let mut paren_depth = 0_usize;
        let mut bracket_depth = 0_usize;
        let mut brace_depth = 0_usize;

        while cursor < self.request.tokens.len() {
            let top_level = paren_depth == 0 && bracket_depth == 0 && brace_depth == 0;
            if top_level {
                if self.is_semicolon_at(cursor) || self.is_end_keyword_at(cursor) {
                    return None;
                }
                if self.is_reserved_word_at(cursor, "be")
                    || self.is_reserved_word_at(cursor, "being")
                {
                    let template_keyword = cursor + 1;
                    if self.is_reserved_word_at(template_keyword, "type")
                        || self.is_reserved_word_at(template_keyword, "pred")
                        || self.is_reserved_word_at(template_keyword, "func")
                    {
                        return Some(template_keyword);
                    }
                    return None;
                }
            }

            if self.is_reserved_symbol_at(cursor, "(") {
                paren_depth += 1;
            } else if self.is_reserved_symbol_at(cursor, ")") {
                if paren_depth == 0 {
                    return None;
                }
                paren_depth -= 1;
            } else if self.is_reserved_symbol_at(cursor, "[") {
                bracket_depth += 1;
            } else if self.is_reserved_symbol_at(cursor, "]") {
                if bracket_depth == 0 {
                    return None;
                }
                bracket_depth -= 1;
            } else if self.is_reserved_symbol_at(cursor, "{") {
                brace_depth += 1;
            } else if self.is_reserved_symbol_at(cursor, "}") {
                if brace_depth == 0 {
                    return None;
                }
                brace_depth -= 1;
            }
            cursor += 1;
        }
        None
    }

    fn is_correctness_condition_keyword_at(&self, position: usize) -> bool {
        self.request.tokens.get(position).is_some_and(|token| {
            token.kind == ParserTokenKind::ReservedWord
                && matches!(
                    token.text.as_ref(),
                    "existence"
                        | "uniqueness"
                        | "coherence"
                        | "compatibility"
                        | "consistency"
                        | "reducibility"
                )
        })
    }

    fn is_definition_content_synchronization_boundary_at(&self, position: usize) -> bool {
        position >= self.request.tokens.len()
            || self.is_semicolon_at(position)
            || self.is_end_keyword_at(position)
            || self.is_definition_content_start_at(position)
    }

    fn is_definition_content_start_at(&self, position: usize) -> bool {
        if self.definition_parameter_starts_template_ambiguous_at(position)
            || self.is_reserved_word_at(position, "let")
            || self.is_reserved_word_at(position, "assume")
            || self.is_correctness_condition_keyword_at(position)
            || self.is_reserved_word_at(position, "attr")
            || self.is_reserved_word_at(position, "pred")
            || self.is_reserved_word_at(position, "func")
            || self.is_reserved_word_at(position, "mode")
            || self.is_reserved_word_at(position, "struct")
            || self.is_reserved_word_at(position, "inherit")
            || self.theorem_role_position_at(position).is_some()
            || self.is_property_clause_keyword_at(position)
            || (self.is_visibility_marker_at(position)
                && self.is_reserved_word_at(position + 1, "pred"))
            || (self.is_visibility_marker_at(position)
                && self.is_reserved_word_at(position + 1, "func"))
            || (self.is_visibility_marker_at(position)
                && self.is_reserved_word_at(position + 1, "mode"))
            || (self.is_visibility_marker_at(position)
                && self.is_reserved_word_at(position + 1, "struct"))
            || (self.is_visibility_marker_at(position)
                && self.is_reserved_word_at(position + 1, "inherit"))
            || (self.is_visibility_marker_at(position)
                && self.is_visible_theorem_target_start_at(position + 1))
        {
            return true;
        }

        self.request.tokens.get(position).is_some_and(|token| {
            token.kind == ParserTokenKind::ReservedWord
                && matches!(
                    token.text.as_ref(),
                    "pred"
                        | "func"
                        | "mode"
                        | "sethood"
                        | "symmetry"
                        | "asymmetry"
                        | "connectedness"
                        | "reflexivity"
                        | "irreflexivity"
                        | "commutativity"
                        | "idempotence"
                        | "involutiveness"
                        | "projectivity"
                        | "struct"
                        | "inherit"
                        | "redefine"
                        | "property"
                        | "registration"
                        | "cluster"
                        | "reduce"
                        | "notation"
                        | "synonym"
                        | "antonym"
                        | "definition"
                        | "private"
                        | "public"
                )
        })
    }

    fn is_structure_member_start_at(&self, position: usize) -> bool {
        self.is_reserved_word_at(position, "field")
            || self.is_reserved_word_at(position, "property")
    }

    fn is_structure_member_boundary_at(&self, position: usize) -> bool {
        position >= self.request.tokens.len()
            || self.is_semicolon_at(position)
            || self.is_end_keyword_at(position)
            || self.is_structure_member_start_at(position)
            || self.is_definition_content_start_at(position)
    }

    fn is_inheritance_member_start_at(&self, position: usize) -> bool {
        self.is_reserved_word_at(position, "field")
            || self.is_reserved_word_at(position, "property")
    }

    fn is_inheritance_member_boundary_at(&self, position: usize) -> bool {
        position >= self.request.tokens.len()
            || self.is_semicolon_at(position)
            || self.is_end_keyword_at(position)
            || self.is_reserved_word_at(position, "coherence")
            || self.is_inheritance_member_start_at(position)
            || self.is_definition_content_start_at(position)
    }

    fn definition_content_placeholder_end(&self, position: usize) -> usize {
        let mut cursor = position;
        let mut block_depth = 0_usize;
        let template_keyword = self.definition_parameter_template_keyword_position_at(position);

        while cursor < self.request.tokens.len() {
            let top_level = block_depth == 0;
            if top_level
                && cursor > position
                && (self.is_end_keyword_at(cursor)
                    || (self.is_definition_content_start_at(cursor)
                        && Some(cursor) != template_keyword))
            {
                break;
            }
            if top_level && self.is_semicolon_at(cursor) {
                return cursor + 1;
            }

            if self.is_end_keyword_at(cursor) {
                if block_depth == 0 {
                    break;
                }
                block_depth -= 1;
                cursor += 1;
                if block_depth == 0 {
                    if self.is_semicolon_at(cursor) {
                        cursor += 1;
                    }
                    break;
                }
                continue;
            }

            if self.opens_definition_content_placeholder_block_at(cursor) {
                block_depth += 1;
            }
            cursor += 1;
        }

        cursor.max(position + 1)
    }

    fn opens_definition_content_placeholder_block_at(&self, position: usize) -> bool {
        sync::opens_recovery_block_at(&self.request.tokens, position)
            || self.is_reserved_word_at(position, "struct")
    }

    fn structure_pattern_boundary_at(&self, position: usize) -> usize {
        if !self.is_structure_definition_symbol_at(position) {
            return self.structure_pattern_delimiter_at(position);
        }

        let cursor = position + 1;
        if self.is_structure_type_params_start_at(cursor) {
            let delimiter = self.structure_pattern_delimiter_at(cursor);
            if let Some(params_end) = self.structure_type_params_end_at(cursor, delimiter)
                && (params_end == delimiter
                    || !self.structure_pattern_has_where_before_delimiter(params_end))
            {
                return params_end;
            }
            return delimiter;
        }

        if self.structure_pattern_has_where_before_delimiter(cursor) {
            self.structure_pattern_delimiter_at(cursor)
        } else {
            cursor
        }
    }

    fn structure_pattern_delimiter_at(&self, position: usize) -> usize {
        let mut cursor = position;
        while cursor < self.request.tokens.len() {
            if self.is_reserved_word_at(cursor, "where")
                || self.is_semicolon_at(cursor)
                || self.is_end_keyword_at(cursor)
                || self.is_structure_member_start_at(cursor)
                || (cursor > position && self.is_definition_content_start_at(cursor))
            {
                break;
            }
            cursor += 1;
        }
        cursor
    }

    fn structure_pattern_has_where_before_delimiter(&self, position: usize) -> bool {
        let mut cursor = position;
        while cursor < self.request.tokens.len() {
            if self.is_reserved_word_at(cursor, "where") {
                return true;
            }
            if self.is_semicolon_at(cursor)
                || self.is_end_keyword_at(cursor)
                || self.is_structure_member_start_at(cursor)
                || self.is_definition_content_start_at(cursor)
            {
                return false;
            }
            cursor += 1;
        }
        false
    }

    fn structure_pattern_can_match(&self, position: usize, end: usize) -> bool {
        let Some(name_end) = self.structure_definition_symbol_end_at(position, end) else {
            return false;
        };
        name_end == end || self.structure_type_params_end_at(name_end, end) == Some(end)
    }

    fn structure_definition_symbol_end_at(&self, position: usize, end: usize) -> Option<usize> {
        (position < end && self.is_structure_definition_symbol_at(position)).then_some(position + 1)
    }

    fn is_structure_definition_symbol_at(&self, position: usize) -> bool {
        self.request.tokens.get(position).is_some_and(|token| {
            matches!(
                token.kind,
                ParserTokenKind::Identifier | ParserTokenKind::UserSymbol
            )
        })
    }

    fn is_structure_type_params_start_at(&self, position: usize) -> bool {
        self.is_reserved_word_at(position, "of")
            || self.is_reserved_word_at(position, "over")
            || self.is_reserved_symbol_at(position, "[")
    }

    fn structure_type_params_end_at(&self, position: usize, end: usize) -> Option<usize> {
        if position >= end {
            return None;
        }
        if self.is_reserved_word_at(position, "of") || self.is_reserved_word_at(position, "over") {
            let list_end = self.structure_type_parameter_list_end_at(position + 1, end)?;
            return (list_end == end).then_some(list_end);
        }
        if self.is_reserved_symbol_at(position, "[") {
            let list_end = self.structure_type_parameter_list_end_at(position + 1, end)?;
            return (list_end + 1 == end && self.is_reserved_symbol_at(list_end, "]"))
                .then_some(end);
        }
        None
    }

    fn structure_type_parameter_list_end_at(&self, position: usize, end: usize) -> Option<usize> {
        if position >= end || !self.is_identifier_at(position) {
            return None;
        }
        let mut cursor = position + 1;
        while cursor + 1 < end
            && self.is_reserved_symbol_at(cursor, ",")
            && self.is_identifier_at(cursor + 1)
        {
            cursor += 2;
        }
        Some(cursor)
    }

    fn inheritance_target_boundary_at(
        &self,
        position: usize,
        boundary: InheritanceTargetBoundary,
    ) -> usize {
        let mut cursor = position;
        while cursor < self.request.tokens.len() {
            if self.is_semicolon_at(cursor)
                || self.is_end_keyword_at(cursor)
                || self.is_definition_content_start_at(cursor)
                || matches!(boundary, InheritanceTargetBoundary::Extends)
                    && self.is_reserved_word_at(cursor, "extends")
                || matches!(boundary, InheritanceTargetBoundary::Tail)
                    && self.is_reserved_word_at(cursor, "where")
            {
                break;
            }
            cursor += 1;
        }
        cursor
    }

    fn inheritance_target_can_match(&self, position: usize, end: usize, allow_set: bool) -> bool {
        if position >= end {
            return false;
        }
        if allow_set && end == position + 1 && self.is_reserved_word_at(position, "set") {
            return true;
        }

        let Some(name_end) = self.structure_reference_name_end_at(position, end) else {
            return false;
        };
        name_end == end || self.raw_inheritance_type_args_can_match(name_end, end)
    }

    fn structure_reference_name_end_at(&self, position: usize, end: usize) -> Option<usize> {
        if position >= end || !self.is_structure_reference_name_token_at(position) {
            return None;
        }
        let mut cursor = position + 1;
        while cursor + 1 < end
            && self.is_reserved_symbol_at(cursor, ".")
            && self.is_structure_reference_name_token_at(cursor + 1)
        {
            cursor += 2;
        }
        Some(cursor)
    }

    fn is_structure_reference_name_token_at(&self, position: usize) -> bool {
        self.request.tokens.get(position).is_some_and(|token| {
            matches!(
                token.kind,
                ParserTokenKind::Identifier | ParserTokenKind::UserSymbol
            )
        })
    }

    fn raw_inheritance_type_args_can_match(&self, position: usize, end: usize) -> bool {
        if position >= end {
            return false;
        }
        if self.is_reserved_word_at(position, "of") || self.is_reserved_word_at(position, "over") {
            return position + 1 < end;
        }
        self.is_reserved_symbol_at(position, "[")
            && position + 2 < end
            && self.is_reserved_symbol_at(end - 1, "]")
    }

    fn predicate_pattern_boundary_at(&self, position: usize) -> usize {
        let mut cursor = position;
        while cursor < self.request.tokens.len() {
            if self.is_reserved_word_at(cursor, "means")
                || self.is_semicolon_at(cursor)
                || self.is_end_keyword_at(cursor)
                || (cursor > position && self.is_definition_content_start_at(cursor))
            {
                break;
            }
            cursor += 1;
        }
        cursor
    }

    fn functor_pattern_boundary_at(&self, position: usize) -> usize {
        let mut cursor = position;
        while cursor < self.request.tokens.len() {
            if self.is_reserved_symbol_at(cursor, "->")
                || self.is_reserved_word_at(cursor, "means")
                || self.is_reserved_word_at(cursor, "equals")
                || self.is_semicolon_at(cursor)
                || self.is_end_keyword_at(cursor)
                || (cursor > position && self.is_definition_content_start_at(cursor))
            {
                break;
            }
            cursor += 1;
        }
        cursor
    }

    fn functor_pattern_can_match(&self, position: usize, end: usize) -> bool {
        self.predicate_pattern_can_match(position, end)
            || self.functor_circumfix_pattern_can_match(position, end)
    }

    fn functor_circumfix_pattern_can_match(&self, position: usize, end: usize) -> bool {
        if position + 2 >= end
            || !self.is_predicate_definition_symbol_at(position)
            || !self.is_predicate_definition_symbol_at(end - 1)
        {
            return false;
        }

        self.predicate_loci_ends_at(position + 1, end - 1)
            .contains(&(end - 1))
    }

    fn mode_pattern_boundary_at(&self, position: usize) -> usize {
        if !self.is_mode_definition_symbol_at(position) {
            return self.mode_pattern_delimiter_at(position);
        }

        let cursor = position + 1;
        if self.is_mode_type_params_start_at(cursor) {
            let delimiter = self.mode_pattern_delimiter_at(cursor);
            if let Some(params_end) = self.mode_type_params_end_at(cursor, delimiter)
                && (params_end == delimiter
                    || !self.mode_pattern_has_is_before_delimiter(params_end))
            {
                return params_end;
            }
            return delimiter;
        }

        if self.mode_pattern_has_is_before_delimiter(cursor) {
            self.mode_pattern_delimiter_at(cursor)
        } else {
            cursor
        }
    }

    fn mode_pattern_delimiter_at(&self, position: usize) -> usize {
        let mut cursor = position;
        while cursor < self.request.tokens.len() {
            if self.is_reserved_word_at(cursor, "is")
                || self.is_semicolon_at(cursor)
                || self.is_end_keyword_at(cursor)
                || (cursor > position && self.is_definition_content_start_at(cursor))
            {
                break;
            }
            cursor += 1;
        }
        cursor
    }

    fn mode_pattern_has_is_before_delimiter(&self, position: usize) -> bool {
        let mut cursor = position;
        while cursor < self.request.tokens.len() {
            if self.is_reserved_word_at(cursor, "is") {
                return true;
            }
            if self.is_semicolon_at(cursor)
                || self.is_end_keyword_at(cursor)
                || self.is_definition_content_start_at(cursor)
            {
                return false;
            }
            cursor += 1;
        }
        false
    }

    fn mode_pattern_can_match(&self, position: usize, end: usize) -> bool {
        let Some(name_end) = self.mode_definition_symbol_end_at(position, end) else {
            return false;
        };
        name_end == end || self.mode_type_params_end_at(name_end, end) == Some(end)
    }

    fn mode_definition_symbol_end_at(&self, position: usize, end: usize) -> Option<usize> {
        (position < end && self.is_mode_definition_symbol_at(position)).then_some(position + 1)
    }

    fn is_mode_definition_symbol_at(&self, position: usize) -> bool {
        self.request.tokens.get(position).is_some_and(|token| {
            matches!(
                token.kind,
                ParserTokenKind::Identifier | ParserTokenKind::UserSymbol
            )
        })
    }

    fn is_mode_type_params_start_at(&self, position: usize) -> bool {
        self.is_reserved_word_at(position, "of")
            || self.is_reserved_word_at(position, "over")
            || self.is_reserved_symbol_at(position, "[")
    }

    fn mode_type_params_end_at(&self, position: usize, end: usize) -> Option<usize> {
        if position >= end {
            return None;
        }
        if self.is_reserved_word_at(position, "of") || self.is_reserved_word_at(position, "over") {
            let list_end = self.mode_type_parameter_list_end_at(position + 1, end)?;
            return (list_end == end).then_some(list_end);
        }
        if self.is_reserved_symbol_at(position, "[") {
            let list_end = self.mode_type_parameter_list_end_at(position + 1, end)?;
            return (list_end + 1 == end && self.is_reserved_symbol_at(list_end, "]"))
                .then_some(end);
        }
        None
    }

    fn mode_type_parameter_list_end_at(&self, position: usize, end: usize) -> Option<usize> {
        if position >= end || !self.is_identifier_at(position) {
            return None;
        }
        let mut cursor = position + 1;
        while cursor + 1 < end
            && self.is_reserved_symbol_at(cursor, ",")
            && self.is_identifier_at(cursor + 1)
        {
            cursor += 2;
        }
        Some(cursor)
    }

    fn predicate_pattern_can_match(&self, position: usize, end: usize) -> bool {
        if position >= end {
            return false;
        }

        let mut left_loci_ends = vec![position];
        left_loci_ends.extend(self.predicate_loci_ends_at(position, end));

        for left_end in left_loci_ends {
            let Some(symbol_end) = self.predicate_symbol_end_at(left_end, end) else {
                continue;
            };
            if self.predicate_pattern_tail_can_match(symbol_end, end) {
                return true;
            }
            if let Some(template_end) = self.predicate_template_loci_end_at(symbol_end, end)
                && self.predicate_pattern_tail_can_match(template_end, end)
            {
                return true;
            }
        }

        false
    }

    fn predicate_pattern_tail_can_match(&self, position: usize, end: usize) -> bool {
        position == end || self.predicate_loci_ends_at(position, end).contains(&end)
    }

    fn predicate_symbol_end_at(&self, position: usize, end: usize) -> Option<usize> {
        if position < end && self.is_predicate_definition_symbol_at(position) {
            Some(position + 1)
        } else {
            None
        }
    }

    fn is_predicate_definition_symbol_at(&self, position: usize) -> bool {
        self.request.tokens.get(position).is_some_and(|token| {
            matches!(
                token.kind,
                ParserTokenKind::Identifier
                    | ParserTokenKind::UserSymbol
                    | ParserTokenKind::LexemeRun
            )
        })
    }

    fn predicate_loci_ends_at(&self, position: usize, end: usize) -> Vec<usize> {
        if position >= end {
            return Vec::new();
        }

        if self.is_reserved_symbol_at(position, "(") {
            let Some(close) = self.predicate_locus_list_end_at(position + 1, end) else {
                return Vec::new();
            };
            if close < end && self.is_reserved_symbol_at(close, ")") {
                return vec![close + 1];
            }
            return Vec::new();
        }

        let mut ends = Vec::new();
        if !self.is_identifier_at(position) {
            return ends;
        }

        let mut cursor = position + 1;
        ends.push(cursor);
        while cursor + 1 < end
            && self.is_reserved_symbol_at(cursor, ",")
            && self.is_identifier_at(cursor + 1)
        {
            cursor += 2;
            ends.push(cursor);
        }
        ends
    }

    fn predicate_template_loci_end_at(&self, position: usize, end: usize) -> Option<usize> {
        if position >= end || !self.is_reserved_symbol_at(position, "[") {
            return None;
        }
        let close = self.predicate_locus_list_end_at(position + 1, end)?;
        (close < end && self.is_reserved_symbol_at(close, "]")).then_some(close + 1)
    }

    fn predicate_locus_list_end_at(&self, position: usize, end: usize) -> Option<usize> {
        if position >= end || !self.is_identifier_at(position) {
            return None;
        }
        let mut cursor = position + 1;
        while cursor + 1 < end
            && self.is_reserved_symbol_at(cursor, ",")
            && self.is_identifier_at(cursor + 1)
        {
            cursor += 2;
        }
        Some(cursor)
    }

    fn is_reconsider_item_boundary_at(&self, position: usize) -> bool {
        position >= self.request.tokens.len()
            || self.is_semicolon_at(position)
            || self.is_end_keyword_at(position)
            || self.is_reserved_symbol_at(position, ",")
            || self.is_reserved_word_at(position, "as")
            || self.is_reserved_word_at(position, "by")
            || self.is_item_start_at(position)
            || self.is_statement_boundary_keyword_at(position)
    }

    fn can_form_type_expression_at(&self, position: usize) -> bool {
        if self.is_type_expression_boundary_at(position) {
            return false;
        }
        if self.can_start_type_head_at(position) {
            return true;
        }
        let mut cursor = position;
        while let Some(plan) = self.plan_attribute_ref_at(cursor) {
            cursor = plan.next_position;
            if self.can_start_type_head_at(cursor) {
                return true;
            }
            if self.is_type_expression_boundary_at(cursor) {
                return false;
            }
        }
        false
    }

    fn can_start_formula_at(&self, position: usize) -> bool {
        !self.is_formula_expression_boundary_at(position)
            && (self.is_reserved_word_at(position, "not")
                || self.is_formula_quantifier_at(position)
                || self.is_reserved_word_at(position, "thesis")
                || self.is_reserved_word_at(position, "contradiction")
                || self.is_reserved_symbol_at(position, "(")
                || self.looks_like_atomic_formula_at(position))
    }

    fn looks_like_atomic_formula_at(&self, position: usize) -> bool {
        if self.is_formula_expression_boundary_at(position) {
            return false;
        }
        if self.is_identifier_at(position) && self.is_reserved_symbol_at(position + 1, "(") {
            return true;
        }
        if self.should_try_head_first_predicate_at(position) {
            return true;
        }

        let mut cursor = position;
        let mut paren_depth = 0_usize;
        let mut bracket_depth = 0_usize;
        let mut brace_depth = 0_usize;
        while cursor < self.request.tokens.len() {
            let at_top_level = paren_depth == 0 && bracket_depth == 0 && brace_depth == 0;
            if at_top_level {
                if cursor > position
                    && (self.is_builtin_predicate_at(cursor)
                        || self.is_reserved_word_at(cursor, "is")
                        || self.request.tokens[cursor].kind == ParserTokenKind::UserSymbol
                        || self.predicate_negation_end_at(cursor).is_some())
                {
                    return true;
                }
                if self.is_formula_expression_boundary_at(cursor)
                    && !self.is_reserved_symbol_at(cursor, ",")
                {
                    return false;
                }
            }

            if self.is_reserved_symbol_at(cursor, "(") {
                paren_depth += 1;
            } else if self.is_reserved_symbol_at(cursor, ")") {
                if paren_depth == 0 {
                    return false;
                }
                paren_depth -= 1;
            } else if self.is_reserved_symbol_at(cursor, "[") {
                bracket_depth += 1;
            } else if self.is_reserved_symbol_at(cursor, "]") {
                if bracket_depth == 0 {
                    return false;
                }
                bracket_depth -= 1;
            } else if self.is_reserved_symbol_at(cursor, "{") {
                brace_depth += 1;
            } else if self.is_reserved_symbol_at(cursor, "}") {
                if brace_depth == 0 {
                    return false;
                }
                brace_depth -= 1;
            }
            cursor += 1;
        }
        false
    }

    fn formula_payload_contains_theorem_tail_at(&self, position: usize) -> bool {
        let mut cursor = position;
        let mut paren_depth = 0_usize;
        let mut bracket_depth = 0_usize;
        let mut brace_depth = 0_usize;
        while cursor < self.request.tokens.len() && !self.is_semicolon_at(cursor) {
            let at_top_level = paren_depth == 0 && bracket_depth == 0 && brace_depth == 0;
            if at_top_level
                && (self.is_reserved_word_at(cursor, "by")
                    || self.is_reserved_word_at(cursor, "proof"))
            {
                return true;
            }

            if self.is_reserved_symbol_at(cursor, "(") {
                paren_depth += 1;
            } else if self.is_reserved_symbol_at(cursor, ")") {
                if paren_depth == 0 {
                    break;
                }
                paren_depth -= 1;
            } else if self.is_reserved_symbol_at(cursor, "[") {
                bracket_depth += 1;
            } else if self.is_reserved_symbol_at(cursor, "]") {
                if bracket_depth == 0 {
                    break;
                }
                bracket_depth -= 1;
            } else if self.is_reserved_symbol_at(cursor, "{") {
                brace_depth += 1;
            } else if self.is_reserved_symbol_at(cursor, "}") {
                if brace_depth == 0 {
                    break;
                }
                brace_depth -= 1;
            }
            cursor += 1;
        }
        false
    }

    fn formula_payload_contains_deferred_predicate_template_args_at(
        &self,
        position: usize,
    ) -> bool {
        let mut cursor = position;
        let mut paren_depth = 0_usize;
        let mut bracket_depth = 0_usize;
        let mut brace_depth = 0_usize;
        let mut in_is_assertion_body = false;
        while cursor < self.request.tokens.len() && !self.is_semicolon_at(cursor) {
            if let Some(condition_start) = self.set_comprehension_condition_start_at(cursor)
                && self
                    .formula_payload_contains_deferred_predicate_template_args_at(condition_start)
            {
                return true;
            }
            let at_top_level = paren_depth == 0 && bracket_depth == 0 && brace_depth == 0;
            if at_top_level {
                if self.formula_connective_at(cursor).is_some()
                    || self.is_reserved_word_at(cursor, "st")
                    || self.is_reserved_word_at(cursor, "holds")
                {
                    in_is_assertion_body = false;
                } else if self.is_reserved_word_at(cursor, "is") {
                    in_is_assertion_body = true;
                } else if !in_is_assertion_body
                    && self
                        .request
                        .tokens
                        .get(cursor)
                        .is_some_and(|token| token.kind == ParserTokenKind::UserSymbol)
                    && self.is_reserved_symbol_at(cursor + 1, "[")
                {
                    return true;
                }
            }

            if self.is_reserved_symbol_at(cursor, "(") {
                paren_depth += 1;
            } else if self.is_reserved_symbol_at(cursor, ")") {
                if paren_depth == 0 {
                    break;
                }
                paren_depth -= 1;
            } else if self.is_reserved_symbol_at(cursor, "[") {
                bracket_depth += 1;
            } else if self.is_reserved_symbol_at(cursor, "]") {
                if bracket_depth == 0 {
                    break;
                }
                bracket_depth -= 1;
            } else if self.is_reserved_symbol_at(cursor, "{") {
                brace_depth += 1;
            } else if self.is_reserved_symbol_at(cursor, "}") {
                if brace_depth == 0 {
                    break;
                }
                brace_depth -= 1;
            }
            cursor += 1;
        }
        false
    }

    fn set_comprehension_condition_start_at(&self, position: usize) -> Option<usize> {
        if !self.is_reserved_symbol_at(position, "{")
            || self
                .set_comprehension_where_before_first_separator_at(position)
                .is_none()
        {
            return None;
        }

        let mut cursor = position + 1;
        let mut paren_depth = 0_usize;
        let mut bracket_depth = 0_usize;
        let mut brace_depth = 1_usize;
        while cursor < self.request.tokens.len() {
            let at_comprehension_top = paren_depth == 0 && bracket_depth == 0 && brace_depth == 1;
            if at_comprehension_top {
                if self.is_reserved_symbol_at(cursor, ":") {
                    return Some(cursor + 1);
                }
                if self.is_reserved_symbol_at(cursor, "}")
                    || self.is_semicolon_at(cursor)
                    || self.is_item_start_at(cursor)
                {
                    return None;
                }
            }

            if self.is_reserved_symbol_at(cursor, "(") {
                paren_depth += 1;
            } else if self.is_reserved_symbol_at(cursor, ")") {
                if paren_depth == 0 {
                    return None;
                }
                paren_depth -= 1;
            } else if self.is_reserved_symbol_at(cursor, "[") {
                bracket_depth += 1;
            } else if self.is_reserved_symbol_at(cursor, "]") {
                if bracket_depth == 0 {
                    return None;
                }
                bracket_depth -= 1;
            } else if self.is_reserved_symbol_at(cursor, "{") {
                brace_depth += 1;
            } else if self.is_reserved_symbol_at(cursor, "}") {
                if brace_depth == 1 {
                    return None;
                }
                brace_depth -= 1;
            }
            cursor += 1;
        }
        None
    }

    fn should_try_head_first_predicate_at(&self, position: usize) -> bool {
        if self.predicate_negation_end_at(position).is_some() {
            return true;
        }
        let Some(symbol_end) = self.qualified_symbol_next_at(position) else {
            return false;
        };
        !self.is_builtin_predicate_at(symbol_end)
            && !self.is_reserved_word_at(symbol_end, "is")
            && !self.is_reserved_symbol_at(symbol_end, "(")
    }

    fn should_try_inline_predicate_at(&self, position: usize) -> bool {
        if !self.is_identifier_at(position) || !self.is_reserved_symbol_at(position + 1, "(") {
            return false;
        }
        let Some(end) = self.delimited_term_arguments_end_at(position + 1) else {
            return true;
        };
        self.is_formula_expression_boundary_at(end)
    }

    fn can_start_predicate_tail_at(&self, position: usize) -> bool {
        self.predicate_negation_end_at(position).is_some()
            || self.qualified_symbol_next_at(position).is_some()
    }

    fn predicate_negation_end_at(&self, position: usize) -> Option<usize> {
        ((self.is_reserved_word_at(position, "does") || self.is_reserved_word_at(position, "do"))
            && self.is_reserved_word_at(position + 1, "not"))
        .then_some(position + 2)
    }

    fn can_start_formula_term_at(&self, position: usize) -> bool {
        self.can_start_term_operator_operand_at(position, false)
    }

    fn should_parse_bare_attribute_test_body_at(&self, position: usize) -> bool {
        let mut cursor = position;
        let saw_non = self.is_reserved_word_at(cursor, "non");
        if saw_non {
            cursor += 1;
        }
        if let Some(prefix_end) = self.parameter_prefix_next_at(cursor) {
            cursor = prefix_end;
        }

        let Some(symbol) = self.attribute_symbol_plan_at(cursor) else {
            return false;
        };
        let final_symbol = &self.request.tokens[symbol.final_symbol];
        let lowercase_attribute_like = final_symbol
            .text
            .chars()
            .next()
            .is_some_and(char::is_lowercase);
        if !saw_non && !lowercase_attribute_like {
            return false;
        }

        let Some(plan) = self.plan_attribute_ref_at(position) else {
            return false;
        };
        !self.can_form_type_expression_at(plan.next_position)
    }

    fn plan_attribute_ref_at(&self, position: usize) -> Option<AttributeRefPlan> {
        let mut cursor = position;
        if self.is_reserved_word_at(cursor, "non") {
            cursor += 1;
        }
        if let Some(prefix_end) = self.parameter_prefix_next_at(cursor) {
            cursor = prefix_end;
        }
        cursor = self.attribute_symbol_next_at(cursor)?;
        if self.is_reserved_symbol_at(cursor, "(") {
            cursor = self.delimited_term_arguments_end_at(cursor)?;
        }
        Some(AttributeRefPlan {
            start_position: position,
            next_position: cursor,
        })
    }

    fn parameter_prefix_next_at(&self, position: usize) -> Option<usize> {
        if self.is_parameter_prefix_atom_at(position)
            && self.is_reserved_symbol_at(position + 1, "-")
        {
            return Some(position + 2);
        }
        if !self.is_reserved_symbol_at(position, "(") {
            return None;
        }
        let mut cursor = position + 1;
        if !self.is_parameter_prefix_atom_at(cursor) {
            return None;
        }
        cursor += 1;
        while self.is_reserved_symbol_at(cursor, ",") {
            cursor += 1;
            if !self.is_parameter_prefix_atom_at(cursor) {
                return None;
            }
            cursor += 1;
        }
        if self.is_reserved_symbol_at(cursor, ")") && self.is_reserved_symbol_at(cursor + 1, "-") {
            Some(cursor + 2)
        } else {
            None
        }
    }

    fn is_parameter_prefix_atom_at(&self, position: usize) -> bool {
        self.request.tokens.get(position).is_some_and(|token| {
            matches!(
                token.kind,
                ParserTokenKind::Identifier | ParserTokenKind::Numeral
            )
        })
    }

    fn qualified_symbol_next_at(&self, position: usize) -> Option<usize> {
        let mut cursor = position;
        while self.is_identifier_at(cursor) && self.is_reserved_symbol_at(cursor + 1, ".") {
            cursor += 2;
        }
        self.request
            .tokens
            .get(cursor)
            .is_some_and(|token| token.kind == ParserTokenKind::UserSymbol)
            .then_some(cursor + 1)
    }

    fn attribute_symbol_next_at(&self, position: usize) -> Option<usize> {
        self.attribute_symbol_plan_at(position)
            .map(|plan| plan.next_position)
    }

    fn attribute_symbol_plan_at(&self, position: usize) -> Option<AttributeSymbolPlan> {
        let mut cursor = position;
        let mut prefix_segments = Vec::new();
        while self.is_attribute_symbol_prefix_at(cursor)
            && self.is_reserved_symbol_at(cursor + 1, ".")
            && self.request.tokens.get(cursor + 2).is_some_and(|token| {
                matches!(
                    token.kind,
                    ParserTokenKind::Identifier | ParserTokenKind::UserSymbol
                )
            })
        {
            prefix_segments.push((cursor, cursor + 1));
            cursor += 2;
        }
        self.request
            .tokens
            .get(cursor)
            .is_some_and(|token| token.kind == ParserTokenKind::UserSymbol)
            .then_some(AttributeSymbolPlan {
                prefix_segments,
                final_symbol: cursor,
                next_position: cursor + 1,
            })
    }

    fn is_attribute_symbol_prefix_at(&self, position: usize) -> bool {
        self.request.tokens.get(position).is_some_and(|token| {
            matches!(
                token.kind,
                ParserTokenKind::Identifier | ParserTokenKind::UserSymbol
            )
        })
    }

    fn can_start_type_head_at(&self, position: usize) -> bool {
        self.is_builtin_type_word_at(position) || self.qualified_symbol_next_at(position).is_some()
    }

    fn can_parse_structure_constructor_after_symbol(&self, symbol_end: usize) -> bool {
        if self.is_reserved_symbol_at(symbol_end, "(") {
            return self.is_field_argument_start_at(symbol_end + 1);
        }
        if !self.is_type_arguments_start_at(symbol_end) {
            return false;
        }
        let Some(arguments_end) = self.type_arguments_end_before_structure_fields_at(symbol_end)
        else {
            return false;
        };
        self.is_reserved_symbol_at(arguments_end, "(")
            && self.is_field_argument_start_at(arguments_end + 1)
    }

    fn is_builtin_type_word_at(&self, position: usize) -> bool {
        self.request.tokens.get(position).is_some_and(|token| {
            token.kind == ParserTokenKind::ReservedWord
                && matches!(token.text.as_ref(), "object" | "set")
        })
    }

    fn is_type_arguments_start_at(&self, position: usize) -> bool {
        self.is_reserved_word_at(position, "of")
            || self.is_reserved_word_at(position, "over")
            || self.is_reserved_symbol_at(position, "[")
    }

    fn delimited_term_arguments_end_at(&self, position: usize) -> Option<usize> {
        if !self.is_reserved_symbol_at(position, "(") {
            return None;
        }
        let mut cursor = position + 1;
        let mut depth = 1_usize;
        let mut bracket_depth = 0_usize;
        let mut brace_depth = 0_usize;
        let mut expecting_argument = true;
        let mut saw_argument = false;
        while cursor < self.request.tokens.len() {
            let at_top_level = depth == 1 && bracket_depth == 0 && brace_depth == 0;
            if at_top_level
                && self.is_reserved_word_at(cursor, "the")
                && let Some(choice_type_end) = self.type_expression_end_at(cursor + 1)
            {
                saw_argument = true;
                expecting_argument = false;
                cursor = choice_type_end;
                continue;
            }
            if at_top_level
                && expecting_argument
                && let Some(symbol_end) = self.qualified_symbol_next_at(cursor)
            {
                saw_argument = true;
                expecting_argument = false;
                cursor = symbol_end;
                continue;
            }
            if at_top_level
                && saw_argument
                && !expecting_argument
                && self.is_reserved_word_at(cursor, "qua")
                && let Some(target_end) = self.type_expression_end_at(cursor + 1)
            {
                cursor = target_end;
                continue;
            }
            if at_top_level
                && saw_argument
                && !expecting_argument
                && self.can_start_type_head_at(cursor)
            {
                return Some(cursor);
            }
            if self.is_reserved_symbol_at(cursor, "(") {
                depth += 1;
            } else if self.is_reserved_symbol_at(cursor, ")") {
                if depth == 1 && bracket_depth == 0 && brace_depth == 0 {
                    return Some(cursor + 1);
                }
                if depth > 1 {
                    depth -= 1;
                }
            } else if self.is_reserved_symbol_at(cursor, "[") {
                bracket_depth += 1;
            } else if self.is_reserved_symbol_at(cursor, "]") {
                if bracket_depth == 0 {
                    return None;
                }
                bracket_depth -= 1;
            } else if self.is_reserved_symbol_at(cursor, "{") {
                brace_depth += 1;
            } else if self.is_reserved_symbol_at(cursor, "}") {
                if brace_depth == 0 {
                    return None;
                }
                brace_depth -= 1;
            } else if depth == 1
                && bracket_depth == 0
                && brace_depth == 0
                && self.is_term_argument_list_boundary_at(cursor)
            {
                return Some(cursor);
            }
            if at_top_level {
                if self.is_reserved_symbol_at(cursor, ",") {
                    expecting_argument = true;
                } else {
                    saw_argument = true;
                    expecting_argument = false;
                }
            }
            cursor += 1;
        }
        Some(cursor)
    }

    fn type_expression_end_at(&self, position: usize) -> Option<usize> {
        let mut cursor = position;
        while let Some(plan) = self.plan_attribute_ref_at(cursor) {
            if !self.can_form_type_expression_at(plan.next_position) {
                break;
            }
            cursor = plan.next_position;
        }
        if self.is_builtin_type_word_at(cursor) {
            return Some(cursor + 1);
        }
        let mut cursor = self.qualified_symbol_next_at(cursor)?;
        if self.is_type_arguments_start_at(cursor) {
            cursor = self.type_arguments_end_at(cursor)?;
        }
        Some(cursor)
    }

    fn type_arguments_end_before_structure_fields_at(&self, position: usize) -> Option<usize> {
        if self.is_reserved_word_at(position, "of") || self.is_reserved_word_at(position, "over") {
            return self.of_over_type_arguments_end_before_structure_fields_at(position);
        }
        self.type_arguments_end_at(position)
    }

    fn of_over_type_arguments_end_before_structure_fields_at(
        &self,
        position: usize,
    ) -> Option<usize> {
        let mut cursor = position + 1;
        let mut paren_depth = 0_usize;
        let mut bracket_depth = 0_usize;
        let mut brace_depth = 0_usize;
        while cursor < self.request.tokens.len() {
            let at_top_level = paren_depth == 0 && bracket_depth == 0 && brace_depth == 0;
            if at_top_level && self.is_structure_field_list_opener_at(cursor) {
                return Some(cursor);
            }
            if at_top_level
                && (self.is_semicolon_at(cursor)
                    || self.is_reserved_symbol_at(cursor, ")")
                    || self.is_reserved_symbol_at(cursor, "]")
                    || self.is_reserved_symbol_at(cursor, "}")
                    || self.is_item_start_at(cursor))
            {
                return Some(cursor);
            }
            if self.is_reserved_symbol_at(cursor, "(") {
                paren_depth += 1;
            } else if self.is_reserved_symbol_at(cursor, ")") {
                if paren_depth == 0 {
                    break;
                }
                paren_depth -= 1;
            } else if self.is_reserved_symbol_at(cursor, "[") {
                bracket_depth += 1;
            } else if self.is_reserved_symbol_at(cursor, "]") {
                if bracket_depth == 0 {
                    break;
                }
                bracket_depth -= 1;
            } else if self.is_reserved_symbol_at(cursor, "{") {
                brace_depth += 1;
            } else if self.is_reserved_symbol_at(cursor, "}") {
                if brace_depth == 0 {
                    break;
                }
                brace_depth -= 1;
            }
            cursor += 1;
        }
        None
    }

    fn term_placeholder_end_at(&self, position: usize) -> Option<usize> {
        if position >= self.request.tokens.len()
            || self.is_term_placeholder_boundary_at(position)
            || self.is_item_start_at(position)
        {
            return None;
        }
        let mut cursor = position;
        let mut paren_depth = 0_usize;
        let mut bracket_depth = 0_usize;
        let mut brace_depth = 0_usize;
        while cursor < self.request.tokens.len() {
            if paren_depth == 0
                && bracket_depth == 0
                && brace_depth == 0
                && (self.is_term_placeholder_boundary_at(cursor) || self.is_item_start_at(cursor))
            {
                break;
            }
            if self.is_reserved_symbol_at(cursor, "(") {
                paren_depth += 1;
            } else if self.is_reserved_symbol_at(cursor, ")") {
                if paren_depth == 0 {
                    break;
                }
                paren_depth -= 1;
            } else if self.is_reserved_symbol_at(cursor, "[") {
                bracket_depth += 1;
            } else if self.is_reserved_symbol_at(cursor, "]") {
                if bracket_depth == 0 {
                    break;
                }
                bracket_depth -= 1;
            } else if self.is_reserved_symbol_at(cursor, "{") {
                brace_depth += 1;
            } else if self.is_reserved_symbol_at(cursor, "}") {
                if brace_depth == 0 {
                    break;
                }
                brace_depth -= 1;
            }
            cursor += 1;
        }
        (cursor > position).then_some(cursor)
    }

    fn type_arguments_end_at(&self, position: usize) -> Option<usize> {
        if self.is_reserved_word_at(position, "of") || self.is_reserved_word_at(position, "over") {
            return self.term_placeholder_end_at(position + 1);
        }
        if !self.is_reserved_symbol_at(position, "[") {
            return None;
        }
        let mut cursor = position + 1;
        let mut depth = 1_usize;
        while cursor < self.request.tokens.len() {
            if self.is_reserved_symbol_at(cursor, "[") {
                depth += 1;
            } else if self.is_reserved_symbol_at(cursor, "]") {
                depth -= 1;
                if depth == 0 {
                    return Some(cursor + 1);
                }
            } else if depth == 1 && (self.is_semicolon_at(cursor) || self.is_item_start_at(cursor))
            {
                return Some(cursor);
            }
            cursor += 1;
        }
        Some(cursor)
    }

    fn is_type_expression_boundary_at(&self, position: usize) -> bool {
        position >= self.request.tokens.len()
            || self.is_semicolon_at(position)
            || self.is_reserved_word_at(position, "by")
            || self.is_reserved_symbol_at(position, ",")
            || self.is_reserved_symbol_at(position, ")")
            || self.is_reserved_symbol_at(position, "]")
            || self.is_reserved_symbol_at(position, "}")
            || self.is_inline_definition_delimiter_at(position)
            || self.is_formula_syntax_boundary_at(position)
            || self.is_item_start_at(position)
    }

    fn is_formula_expression_boundary_at(&self, position: usize) -> bool {
        position >= self.request.tokens.len()
            || self.is_semicolon_at(position)
            || self.is_reserved_word_at(position, "by")
            || self.is_reserved_word_at(position, "proof")
            || self.is_reserved_symbol_at(position, ",")
            || self.is_reserved_symbol_at(position, ")")
            || self.is_reserved_symbol_at(position, "]")
            || self.is_reserved_symbol_at(position, "}")
            || self.is_inline_definition_delimiter_at(position)
            || self.is_formula_syntax_boundary_at(position)
            || self.is_item_start_at(position)
    }

    fn is_term_expression_boundary_at(&self, position: usize) -> bool {
        position >= self.request.tokens.len()
            || self.is_semicolon_at(position)
            || self.is_reserved_word_at(position, "by")
            || self.is_reserved_word_at(position, "proof")
            || self.is_reserved_symbol_at(position, ",")
            || self.is_reserved_symbol_at(position, ")")
            || self.is_reserved_symbol_at(position, "]")
            || self.is_reserved_symbol_at(position, "}")
            || self.is_inline_definition_delimiter_at(position)
            || self.is_formula_syntax_boundary_at(position)
            || self.is_item_start_at(position)
    }

    fn is_term_placeholder_boundary_at(&self, position: usize) -> bool {
        self.is_semicolon_at(position)
            || self.is_reserved_word_at(position, "by")
            || self.is_reserved_symbol_at(position, ",")
            || self.is_reserved_symbol_at(position, ")")
            || self.is_reserved_symbol_at(position, "]")
            || self.is_reserved_symbol_at(position, "}")
            || self.is_inline_definition_delimiter_at(position)
            || self.is_formula_syntax_boundary_at(position)
    }

    fn is_term_argument_list_boundary_at(&self, position: usize) -> bool {
        position >= self.request.tokens.len()
            || self.is_semicolon_at(position)
            || self.is_reserved_symbol_at(position, ")")
            || self.is_reserved_symbol_at(position, "]")
            || self.is_reserved_symbol_at(position, "}")
            || self.is_item_start_at(position)
    }

    fn is_field_argument_start_at(&self, position: usize) -> bool {
        self.is_identifier_at(position) && self.is_reserved_symbol_at(position + 1, ":")
    }

    fn is_structure_field_list_opener_at(&self, position: usize) -> bool {
        self.is_reserved_symbol_at(position, "(") && self.is_field_argument_start_at(position + 1)
    }

    fn item_head_position(&self, position: usize) -> Option<usize> {
        if self.is_top_level_keyword_at(position) {
            return Some(position);
        }
        let head = self.skip_annotations(position);
        if head > position && self.is_top_level_keyword_at(head) {
            Some(head)
        } else {
            None
        }
    }

    fn skip_annotations(&self, mut position: usize) -> usize {
        while let Some(next) = self.after_library_annotation(position) {
            position = next;
        }
        position
    }

    fn after_library_annotation(&self, position: usize) -> Option<usize> {
        if !self.is_reserved_symbol_at(position, "@[") {
            return None;
        }
        let mut cursor = position + 1;
        let mut depth = 1_usize;
        while cursor < self.request.tokens.len() {
            if self.is_reserved_symbol_at(cursor, "@[") {
                depth += 1;
            } else if self.is_reserved_symbol_at(cursor, "]") {
                depth -= 1;
                if depth == 0 {
                    return Some(cursor + 1);
                }
            }
            cursor += 1;
        }
        None
    }

    fn is_top_level_keyword_at(&self, position: usize) -> bool {
        self.request
            .tokens
            .get(position)
            .is_some_and(is_top_level_item_keyword)
    }

    fn is_prefix_continuation_at(&self, head: usize, cursor: usize) -> bool {
        self.is_top_level_keyword_at(cursor)
            && cursor > head
            && (head..cursor).all(|position| self.is_item_prefix_keyword_at(position))
    }

    fn is_item_prefix_keyword_at(&self, position: usize) -> bool {
        self.request.tokens.get(position).is_some_and(|token| {
            token.kind == ParserTokenKind::ReservedWord
                && matches!(
                    token.text.as_ref(),
                    "open" | "assumed" | "conditional" | "private" | "public"
                )
        })
    }

    fn is_semicolon_at(&self, position: usize) -> bool {
        self.is_reserved_symbol_at(position, ";")
    }

    fn is_builtin_predicate_at(&self, position: usize) -> bool {
        self.is_reserved_word_at(position, "in")
            || self.is_reserved_symbol_at(position, "=")
            || self.is_reserved_symbol_at(position, "<>")
    }

    fn formula_connective_at(&self, position: usize) -> Option<FormulaConnectiveToken> {
        if self.is_reserved_symbol_at(position, "&") {
            let repeated = self.is_reserved_symbol_at(position + 1, "...")
                && self.is_reserved_symbol_at(position + 2, "&");
            return Some(FormulaConnectiveToken {
                connective: SurfaceFormulaConnective::And,
                repeated,
                token_count: if repeated { 3 } else { 1 },
                left_binding_power: 50,
                right_binding_power: 51,
            });
        }
        if self.is_reserved_word_at(position, "or") {
            let repeated = self.is_reserved_symbol_at(position + 1, "...")
                && self.is_reserved_word_at(position + 2, "or");
            return Some(FormulaConnectiveToken {
                connective: SurfaceFormulaConnective::Or,
                repeated,
                token_count: if repeated { 3 } else { 1 },
                left_binding_power: 40,
                right_binding_power: 41,
            });
        }
        if self.is_reserved_word_at(position, "implies") {
            return Some(FormulaConnectiveToken {
                connective: SurfaceFormulaConnective::Implies,
                repeated: false,
                token_count: 1,
                left_binding_power: 30,
                right_binding_power: 30,
            });
        }
        if self.is_reserved_word_at(position, "iff") {
            return Some(FormulaConnectiveToken {
                connective: SurfaceFormulaConnective::Iff,
                repeated: false,
                token_count: 1,
                left_binding_power: 20,
                right_binding_power: 21,
            });
        }
        None
    }

    fn is_formula_quantifier_at(&self, position: usize) -> bool {
        self.is_reserved_word_at(position, "for") || self.is_reserved_word_at(position, "ex")
    }

    fn is_formula_syntax_boundary_at(&self, position: usize) -> bool {
        self.formula_connective_at(position).is_some()
            || self.is_reserved_word_at(position, "st")
            || self.is_reserved_word_at(position, "holds")
    }

    fn left_is_iff_formula_chain(&self, left: SurfaceBuilderNodeId) -> bool {
        matches!(
            self.events.node_kind(left).unwrap(),
            SurfaceNodeKind::BinaryFormula(left_operator)
                if left_operator.connective == SurfaceFormulaConnective::Iff
        )
    }

    fn is_reserved_symbol_at(&self, position: usize, spelling: &str) -> bool {
        self.request
            .tokens
            .get(position)
            .is_some_and(|token| is_reserved_symbol_token(token, spelling))
    }

    fn is_end_keyword_at(&self, position: usize) -> bool {
        self.is_reserved_word_at(position, "end")
    }

    fn is_reserved_word_at(&self, position: usize, spelling: &str) -> bool {
        self.request
            .tokens
            .get(position)
            .is_some_and(|token| is_reserved_word_token(token, spelling))
    }

    fn module_range(&self) -> SourceRange {
        self.request
            .tokens
            .first()
            .zip(self.request.tokens.last())
            .map_or(
                SourceRange {
                    source_id: self.request.source_id,
                    start: 0,
                    end: 0,
                },
                |(first, last)| SourceRange {
                    source_id: self.request.source_id,
                    start: first.span.start,
                    end: last.span.end,
                },
            )
    }

    fn covering_token_range(&self, start: usize, end_exclusive: usize) -> SourceRange {
        if start >= end_exclusive || start >= self.request.tokens.len() {
            let offset = self.request.tokens.last().map_or(0, |token| token.span.end);
            return SourceRange {
                source_id: self.request.source_id,
                start: offset,
                end: offset,
            };
        }
        let first = self.request.tokens[start].span;
        let last = self.request.tokens[end_exclusive - 1].span;
        SourceRange {
            source_id: self.request.source_id,
            start: first.start,
            end: last.end,
        }
    }

    fn zero_range_at(&self, position: usize) -> SourceRange {
        let offset = self.request.tokens.get(position).map_or_else(
            || self.request.tokens.last().map_or(0, |token| token.span.end),
            |token| token.span.start,
        );
        SourceRange {
            source_id: self.request.source_id,
            start: offset,
            end: offset,
        }
    }
}

fn is_block_like_top_level_start(token: &ParserToken) -> bool {
    token.kind == ParserTokenKind::ReservedWord
        && matches!(token.text.as_ref(), "definition" | "registration" | "claim")
}

fn is_reserved_symbol_token(token: &ParserToken, spelling: &str) -> bool {
    token.kind == ParserTokenKind::ReservedSymbol && token.text.as_ref() == spelling
}

fn infix_binding_powers(operator: &OperatorFixityEntry) -> (u32, u32) {
    let precedence = u32::from(operator.precedence);
    match infix_associativity(operator) {
        OperatorAssociativity::Left | OperatorAssociativity::NonAssociative => {
            (precedence, precedence + 1)
        }
        OperatorAssociativity::Right => (precedence, precedence),
    }
}

fn infix_associativity(operator: &OperatorFixityEntry) -> OperatorAssociativity {
    match operator.fixity {
        OperatorFixity::Infix(associativity) => associativity,
        OperatorFixity::Prefix | OperatorFixity::Postfix => {
            unreachable!("term Pratt requested infix data for a non-infix operator")
        }
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
    use crate::{
        OperatorAssociativity, OperatorFixityEntry, ParseRequest, ParserToken, ParserTokenKind,
        parse,
    };
    use mizar_session::{
        BuildSnapshotId, Edition, Hash, InMemorySessionIdAllocator, SessionIdAllocator, SourceId,
        SourceRange,
    };
    use mizar_syntax::{
        SkippedTokenReason, SurfaceFormulaConnective, SurfaceFormulaConstant, SurfaceNodeKind,
        SurfaceOperatorAssociativity, SurfaceQuantifierKind, SyntaxDiagnosticCode,
        SyntaxRecoveryKind,
    };

    #[test]
    fn parser_builds_compilation_unit_and_placeholder_items() {
        let source_id = source_id(40);
        let output = parse(ParseRequest::new(
            source_id,
            Edition::new("2026"),
            vec![
                token(source_id, ParserTokenKind::ReservedWord, "theorem", 0, 7),
                token(source_id, ParserTokenKind::Identifier, "T", 8, 9),
                token(source_id, ParserTokenKind::ReservedSymbol, ":", 9, 10),
                token(source_id, ParserTokenKind::ReservedWord, "thesis", 11, 17),
                token(source_id, ParserTokenKind::ReservedSymbol, ";", 17, 18),
                token(source_id, ParserTokenKind::ReservedWord, "lemma", 19, 24),
                token(source_id, ParserTokenKind::Identifier, "L", 25, 26),
                token(source_id, ParserTokenKind::ReservedSymbol, ";", 26, 27),
            ],
            Vec::new(),
        ));

        assert!(
            output.diagnostics.is_empty(),
            "unexpected diagnostics: {:?}",
            output.diagnostics
        );
        let ast = output.ast.expect("module skeleton should parse");
        let compilation_unit = single_node(&ast, |kind| {
            matches!(kind, SurfaceNodeKind::CompilationUnit)
        });
        let item_list_id = compilation_unit.children[0];
        assert!(matches!(
            ast.node(item_list_id).unwrap().kind,
            SurfaceNodeKind::ItemList
        ));
        let items = &ast.node(item_list_id).unwrap().children;
        assert_eq!(items.len(), 2);
        assert!(matches!(
            ast.node(items[0]).unwrap().kind,
            SurfaceNodeKind::TheoremItem
        ));
        assert!(matches!(
            ast.node(items[1]).unwrap().kind,
            SurfaceNodeKind::PlaceholderItem
        ));
        assert_eq!(ast.node(items[0]).unwrap().range, range(source_id, 0, 18));
        assert_eq!(ast.node(items[1]).unwrap().range, range(source_id, 19, 27));
        assert!(ast.snapshot_text().contains("CompilationUnit range=0..27"));
        assert!(ast.snapshot_text().contains("ItemList range=0..27"));
        assert!(ast.trivia().skipped_token_ranges().is_empty());
    }

    #[test]
    fn parser_dispatches_every_documented_top_level_start() {
        let cases: &[(&str, &[(&str, ParserTokenKind)])] = &[
            (
                "import",
                &[
                    ("import", ParserTokenKind::ReservedWord),
                    ("Std", ParserTokenKind::Identifier),
                    (";", ParserTokenKind::ReservedSymbol),
                ],
            ),
            (
                "export",
                &[
                    ("export", ParserTokenKind::ReservedWord),
                    ("Std", ParserTokenKind::Identifier),
                    (";", ParserTokenKind::ReservedSymbol),
                ],
            ),
            (
                "definition",
                &[
                    ("definition", ParserTokenKind::ReservedWord),
                    ("end", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                ],
            ),
            (
                "reserve",
                &[
                    ("reserve", ParserTokenKind::ReservedWord),
                    ("x", ParserTokenKind::Identifier),
                    ("for", ParserTokenKind::ReservedWord),
                    ("set", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                ],
            ),
            (
                "registration",
                &[
                    ("registration", ParserTokenKind::ReservedWord),
                    ("end", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                ],
            ),
            (
                "claim",
                &[
                    ("claim", ParserTokenKind::ReservedWord),
                    ("end", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                ],
            ),
            (
                "theorem",
                &[
                    ("theorem", ParserTokenKind::ReservedWord),
                    ("T", ParserTokenKind::Identifier),
                    (";", ParserTokenKind::ReservedSymbol),
                ],
            ),
            (
                "lemma",
                &[
                    ("lemma", ParserTokenKind::ReservedWord),
                    ("L", ParserTokenKind::Identifier),
                    (";", ParserTokenKind::ReservedSymbol),
                ],
            ),
            (
                "open",
                &[
                    ("open", ParserTokenKind::ReservedWord),
                    ("theorem", ParserTokenKind::ReservedWord),
                    ("O", ParserTokenKind::Identifier),
                    (";", ParserTokenKind::ReservedSymbol),
                ],
            ),
            (
                "assumed",
                &[
                    ("assumed", ParserTokenKind::ReservedWord),
                    ("theorem", ParserTokenKind::ReservedWord),
                    ("A", ParserTokenKind::Identifier),
                    (";", ParserTokenKind::ReservedSymbol),
                ],
            ),
            (
                "conditional",
                &[
                    ("conditional", ParserTokenKind::ReservedWord),
                    ("lemma", ParserTokenKind::ReservedWord),
                    ("C", ParserTokenKind::Identifier),
                    (";", ParserTokenKind::ReservedSymbol),
                ],
            ),
            (
                "private",
                &[
                    ("private", ParserTokenKind::ReservedWord),
                    ("theorem", ParserTokenKind::ReservedWord),
                    ("P", ParserTokenKind::Identifier),
                    (";", ParserTokenKind::ReservedSymbol),
                ],
            ),
            (
                "public",
                &[
                    ("public", ParserTokenKind::ReservedWord),
                    ("theorem", ParserTokenKind::ReservedWord),
                    ("Q", ParserTokenKind::Identifier),
                    (";", ParserTokenKind::ReservedSymbol),
                ],
            ),
            (
                "infix_operator",
                &[
                    ("infix_operator", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                ],
            ),
            (
                "prefix_operator",
                &[
                    ("prefix_operator", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                ],
            ),
            (
                "postfix_operator",
                &[
                    ("postfix_operator", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                ],
            ),
            (
                "synonym",
                &[
                    ("synonym", ParserTokenKind::ReservedWord),
                    ("A", ParserTokenKind::Identifier),
                    ("for", ParserTokenKind::ReservedWord),
                    ("B", ParserTokenKind::Identifier),
                    (";", ParserTokenKind::ReservedSymbol),
                ],
            ),
            (
                "antonym",
                &[
                    ("antonym", ParserTokenKind::ReservedWord),
                    ("A", ParserTokenKind::Identifier),
                    ("for", ParserTokenKind::ReservedWord),
                    ("B", ParserTokenKind::Identifier),
                    (";", ParserTokenKind::ReservedSymbol),
                ],
            ),
        ];

        for (index, (name, entries)) in cases.iter().enumerate() {
            let source_id = source_id(50 + index as u8);
            let tokens = token_sequence(source_id, entries);
            let expected_range = tokens
                .first()
                .zip(tokens.last())
                .map(|(first, last)| range(source_id, first.span.start, last.span.end))
                .unwrap();
            let output = parse(ParseRequest::new(
                source_id,
                Edition::new("2026"),
                tokens,
                Vec::new(),
            ));

            assert!(output.diagnostics.is_empty(), "{name} should dispatch");
            let ast = output.ast.expect("top-level start should recover an AST");
            let item_list = single_node(&ast, |kind| matches!(kind, SurfaceNodeKind::ItemList));
            assert_eq!(item_list.children.len(), 1, "{name} should make one item");
            let item = ast.node(item_list.children[0]).unwrap();
            let expected_kind = match *name {
                "import" => SurfaceNodeKind::ImportItem,
                "export" => SurfaceNodeKind::ExportItem,
                "private" | "public" => SurfaceNodeKind::VisibleItem,
                "reserve" => SurfaceNodeKind::ReserveItem,
                "definition" => SurfaceNodeKind::DefinitionBlockItem,
                "synonym" | "antonym" => SurfaceNodeKind::NotationAlias,
                _ => SurfaceNodeKind::PlaceholderItem,
            };
            assert_eq!(item.kind, expected_kind);
            assert_eq!(item.range, expected_range, "{name} placeholder range");
        }
    }

    #[test]
    fn parser_parses_export_prelude_and_visibility_items() {
        let source_id = source_id(82);
        let tokens = token_sequence(
            source_id,
            &[
                ("import", ParserTokenKind::ReservedWord),
                ("Std", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
                ("export", ParserTokenKind::ReservedWord),
                ("Std", ParserTokenKind::Identifier),
                (".", ParserTokenKind::ReservedSymbol),
                ("Core", ParserTokenKind::Identifier),
                (",", ParserTokenKind::ReservedSymbol),
                (".", ParserTokenKind::ReservedSymbol),
                ("Facade", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
                ("public", ParserTokenKind::ReservedWord),
                ("open", ParserTokenKind::ReservedWord),
                ("theorem", ParserTokenKind::ReservedWord),
                ("T", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
                ("private", ParserTokenKind::ReservedWord),
                ("synonym", ParserTokenKind::ReservedWord),
                ("P", ParserTokenKind::Identifier),
                ("for", ParserTokenKind::ReservedWord),
                ("Q", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
            ],
        );
        let output = parse(ParseRequest::new(
            source_id,
            Edition::new("2026"),
            tokens,
            Vec::new(),
        ));

        assert!(
            output.diagnostics.is_empty(),
            "unexpected diagnostics: {:?}",
            output.diagnostics
        );
        let ast = output.ast.expect("export and visible items should parse");
        let item_list = single_node(&ast, |kind| matches!(kind, SurfaceNodeKind::ItemList));
        assert_eq!(item_list.children.len(), 4);
        assert!(matches!(
            ast.node(item_list.children[0]).unwrap().kind,
            SurfaceNodeKind::ImportItem
        ));
        let export = ast.node(item_list.children[1]).unwrap();
        assert!(matches!(export.kind, SurfaceNodeKind::ExportItem));
        assert_eq!(export.children.len(), 5);
        assert_eq!(
            ast.node(export.children[0]).unwrap().token_text(),
            Some("export")
        );
        assert!(matches!(
            ast.node(export.children[1]).unwrap().kind,
            SurfaceNodeKind::ModulePath
        ));
        assert_eq!(
            ast.node(export.children[2]).unwrap().token_text(),
            Some(",")
        );
        assert!(matches!(
            ast.node(export.children[3]).unwrap().kind,
            SurfaceNodeKind::ModulePath
        ));
        assert_eq!(
            ast.node(export.children[4]).unwrap().token_text(),
            Some(";")
        );

        let visible_theorem = ast.node(item_list.children[2]).unwrap();
        assert!(matches!(visible_theorem.kind, SurfaceNodeKind::VisibleItem));
        assert_eq!(visible_theorem.children.len(), 2);
        assert!(matches!(
            ast.node(visible_theorem.children[0]).unwrap().kind,
            SurfaceNodeKind::VisibilityMarker
        ));
        assert!(matches!(
            ast.node(visible_theorem.children[1]).unwrap().kind,
            SurfaceNodeKind::PlaceholderItem
        ));

        let visible_alias = ast.node(item_list.children[3]).unwrap();
        assert!(matches!(visible_alias.kind, SurfaceNodeKind::VisibleItem));
        assert_eq!(visible_alias.children.len(), 2);
        assert!(matches!(
            ast.node(visible_alias.children[0]).unwrap().kind,
            SurfaceNodeKind::VisibilityMarker
        ));
        assert!(matches!(
            ast.node(visible_alias.children[1]).unwrap().kind,
            SurfaceNodeKind::NotationAlias
        ));
    }

    #[test]
    fn parser_recovers_export_after_ordinary_item() {
        let source_id = source_id(83);
        let tokens = token_sequence(
            source_id,
            &[
                ("theorem", ParserTokenKind::ReservedWord),
                ("T", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
                ("export", ParserTokenKind::ReservedWord),
                ("Late", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
                ("lemma", ParserTokenKind::ReservedWord),
                ("U", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
            ],
        );
        let export_range = tokens[3].span;
        let skipped_range = range(source_id, tokens[3].span.start, tokens[5].span.end);
        let output = parse(ParseRequest::new(
            source_id,
            Edition::new("2026"),
            tokens,
            Vec::new(),
        ));

        assert_eq!(output.diagnostics.len(), 1);
        assert_eq!(
            output.diagnostics[0].code,
            SyntaxDiagnosticCode::UnexpectedTopLevelToken
        );
        assert_eq!(output.diagnostics[0].primary, export_range);
        let ast = output.ast.expect("late export should recover");
        let item_list = single_node(&ast, |kind| matches!(kind, SurfaceNodeKind::ItemList));
        assert_eq!(item_list.children.len(), 3);
        assert!(matches!(
            ast.node(item_list.children[0]).unwrap().kind,
            SurfaceNodeKind::PlaceholderItem
        ));
        let skipped = ast.node(item_list.children[1]).unwrap();
        assert!(matches!(
            skipped.kind,
            SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::SkippedToken)
        ));
        assert_eq!(skipped.range, skipped_range);
        assert!(matches!(
            ast.node(item_list.children[2]).unwrap().kind,
            SurfaceNodeKind::PlaceholderItem
        ));
    }

    #[test]
    fn parser_recovers_annotation_prefixed_import_and_export_as_unexpected_items() {
        let source_id = source_id(94);
        let tokens = token_sequence(
            source_id,
            &[
                ("theorem", ParserTokenKind::ReservedWord),
                ("T", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
                ("@[", ParserTokenKind::ReservedSymbol),
                ("note", ParserTokenKind::Identifier),
                ("]", ParserTokenKind::ReservedSymbol),
                ("@[", ParserTokenKind::ReservedSymbol),
                ("note2", ParserTokenKind::Identifier),
                ("]", ParserTokenKind::ReservedSymbol),
                ("export", ParserTokenKind::ReservedWord),
                ("Late", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
                ("@[", ParserTokenKind::ReservedSymbol),
                ("note", ParserTokenKind::Identifier),
                ("]", ParserTokenKind::ReservedSymbol),
                ("import", ParserTokenKind::ReservedWord),
                ("Late", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
                ("lemma", ParserTokenKind::ReservedWord),
                ("U", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
            ],
        );
        let export_annotation_range = tokens[3].span;
        let import_annotation_range = tokens[12].span;
        let export_skipped_range = range(source_id, tokens[3].span.start, tokens[11].span.end);
        let import_skipped_range = range(source_id, tokens[12].span.start, tokens[17].span.end);
        let output = parse(ParseRequest::new(
            source_id,
            Edition::new("2026"),
            tokens,
            Vec::new(),
        ));

        assert_eq!(output.diagnostics.len(), 2);
        assert_eq!(
            output.diagnostics[0].code,
            SyntaxDiagnosticCode::UnexpectedTopLevelToken
        );
        assert_eq!(output.diagnostics[0].primary, export_annotation_range);
        assert_eq!(
            output.diagnostics[1].code,
            SyntaxDiagnosticCode::UnexpectedTopLevelToken
        );
        assert_eq!(output.diagnostics[1].primary, import_annotation_range);
        let ast = output
            .ast
            .expect("annotated import/export should recover as unexpected top-level input");
        let item_list = single_node(&ast, |kind| matches!(kind, SurfaceNodeKind::ItemList));
        assert_eq!(item_list.children.len(), 4);
        assert!(matches!(
            ast.node(item_list.children[0]).unwrap().kind,
            SurfaceNodeKind::PlaceholderItem
        ));
        assert_eq!(
            ast.node(item_list.children[1]).unwrap().range,
            export_skipped_range
        );
        assert!(matches!(
            ast.node(item_list.children[1]).unwrap().kind,
            SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::SkippedToken)
        ));
        assert_eq!(
            ast.node(item_list.children[2]).unwrap().range,
            import_skipped_range
        );
        assert!(matches!(
            ast.node(item_list.children[2]).unwrap().kind,
            SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::SkippedToken)
        ));
        assert!(matches!(
            ast.node(item_list.children[3]).unwrap().kind,
            SurfaceNodeKind::PlaceholderItem
        ));
    }

    #[test]
    fn parser_recovers_annotation_prefixed_import_while_import_prelude_open() {
        let source_id = source_id(96);
        let tokens = token_sequence(
            source_id,
            &[
                ("@[", ParserTokenKind::ReservedSymbol),
                ("note", ParserTokenKind::Identifier),
                ("]", ParserTokenKind::ReservedSymbol),
                ("import", ParserTokenKind::ReservedWord),
                ("Core", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
                ("theorem", ParserTokenKind::ReservedWord),
                ("T", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
            ],
        );
        let annotation_range = tokens[0].span;
        let skipped_range = range(source_id, tokens[0].span.start, tokens[5].span.end);
        let output = parse(ParseRequest::new(
            source_id,
            Edition::new("2026"),
            tokens,
            Vec::new(),
        ));

        assert_eq!(output.diagnostics.len(), 1);
        assert_eq!(
            output.diagnostics[0].code,
            SyntaxDiagnosticCode::UnexpectedTopLevelToken
        );
        assert_eq!(output.diagnostics[0].primary, annotation_range);
        let ast = output
            .ast
            .expect("annotated import should recover even while import prelude is open");
        let item_list = single_node(&ast, |kind| matches!(kind, SurfaceNodeKind::ItemList));
        assert_eq!(item_list.children.len(), 2);
        let recovery = ast.node(item_list.children[0]).unwrap();
        assert!(matches!(
            recovery.kind,
            SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::SkippedToken)
        ));
        assert_eq!(recovery.range, skipped_range);
        assert!(matches!(
            ast.node(item_list.children[1]).unwrap().kind,
            SurfaceNodeKind::PlaceholderItem
        ));
    }

    #[test]
    fn parser_recovers_annotation_prefixed_export_while_export_prelude_open() {
        let source_id = source_id(97);
        let tokens = token_sequence(
            source_id,
            &[
                ("import", ParserTokenKind::ReservedWord),
                ("Core", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
                ("@[", ParserTokenKind::ReservedSymbol),
                ("note", ParserTokenKind::Identifier),
                ("]", ParserTokenKind::ReservedSymbol),
                ("export", ParserTokenKind::ReservedWord),
                ("Facade", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
                ("theorem", ParserTokenKind::ReservedWord),
                ("T", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
            ],
        );
        let annotation_range = tokens[3].span;
        let skipped_range = range(source_id, tokens[3].span.start, tokens[8].span.end);
        let output = parse(ParseRequest::new(
            source_id,
            Edition::new("2026"),
            tokens,
            Vec::new(),
        ));

        assert_eq!(output.diagnostics.len(), 1);
        assert_eq!(
            output.diagnostics[0].code,
            SyntaxDiagnosticCode::UnexpectedTopLevelToken
        );
        assert_eq!(output.diagnostics[0].primary, annotation_range);
        let ast = output
            .ast
            .expect("annotated export should recover even while export prelude is open");
        let item_list = single_node(&ast, |kind| matches!(kind, SurfaceNodeKind::ItemList));
        assert_eq!(item_list.children.len(), 3);
        assert!(matches!(
            ast.node(item_list.children[0]).unwrap().kind,
            SurfaceNodeKind::ImportItem
        ));
        let recovery = ast.node(item_list.children[1]).unwrap();
        assert!(matches!(
            recovery.kind,
            SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::SkippedToken)
        ));
        assert_eq!(recovery.range, skipped_range);
        assert!(matches!(
            ast.node(item_list.children[2]).unwrap().kind,
            SurfaceNodeKind::PlaceholderItem
        ));
    }

    #[test]
    fn parser_recovers_bad_export_path_tail_inside_export_item() {
        let source_id = source_id(84);
        let tokens = token_sequence(
            source_id,
            &[
                ("export", ParserTokenKind::ReservedWord),
                ("123", ParserTokenKind::Numeral),
                (";", ParserTokenKind::ReservedSymbol),
                ("theorem", ParserTokenKind::ReservedWord),
                ("T", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
            ],
        );
        let bad_range = tokens[1].span;
        let output = parse(ParseRequest::new(
            source_id,
            Edition::new("2026"),
            tokens,
            Vec::new(),
        ));

        assert_eq!(output.diagnostics.len(), 1);
        assert_eq!(
            output.diagnostics[0].code,
            SyntaxDiagnosticCode::MalformedExport
        );
        assert_eq!(output.diagnostics[0].primary, bad_range);
        let ast = output
            .ast
            .expect("bad export path should recover inside export item");
        let item_list = single_node(&ast, |kind| matches!(kind, SurfaceNodeKind::ItemList));
        assert_eq!(item_list.children.len(), 2);
        let export_item = ast.node(item_list.children[0]).unwrap();
        assert!(matches!(export_item.kind, SurfaceNodeKind::ExportItem));
        let recovery = export_item
            .children
            .iter()
            .filter_map(|id| ast.node(*id))
            .find(|node| {
                matches!(
                    node.kind,
                    SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::SkippedToken)
                )
            })
            .expect("bad export path token should be owned by a recovery node");
        assert_eq!(recovery.range, bad_range);
        assert_eq!(
            rowan_token_texts(&ast),
            vec!["export", "123", ";", "theorem", "T", ";"]
        );
    }

    #[test]
    fn parser_recovers_bad_export_tail_after_path_inside_export_item() {
        let source_id = source_id(89);
        let tokens = token_sequence(
            source_id,
            &[
                ("export", ParserTokenKind::ReservedWord),
                ("Std", ParserTokenKind::Identifier),
                (".", ParserTokenKind::ReservedSymbol),
                (";", ParserTokenKind::ReservedSymbol),
                ("theorem", ParserTokenKind::ReservedWord),
                ("T", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
            ],
        );
        let bad_range = tokens[2].span;
        let output = parse(ParseRequest::new(
            source_id,
            Edition::new("2026"),
            tokens,
            Vec::new(),
        ));

        assert_eq!(output.diagnostics.len(), 1);
        assert_eq!(
            output.diagnostics[0].code,
            SyntaxDiagnosticCode::MalformedExport
        );
        assert_eq!(output.diagnostics[0].primary, bad_range);
        let ast = output
            .ast
            .expect("bad export tail should recover inside export item");
        let export_item = single_node(&ast, |kind| matches!(kind, SurfaceNodeKind::ExportItem));
        let recovery = export_item
            .children
            .iter()
            .filter_map(|id| ast.node(*id))
            .find(|node| {
                matches!(
                    node.kind,
                    SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::SkippedToken)
                )
            })
            .expect("bad tail token should be owned by export recovery");
        assert_eq!(recovery.range, bad_range);
        assert_eq!(
            rowan_token_texts(&ast),
            vec!["export", "Std", ".", ";", "theorem", "T", ";"]
        );
    }

    #[test]
    fn parser_recovers_bad_export_path_after_comma_inside_export_item() {
        let source_id = source_id(90);
        let tokens = token_sequence(
            source_id,
            &[
                ("export", ParserTokenKind::ReservedWord),
                ("Std", ParserTokenKind::Identifier),
                (",", ParserTokenKind::ReservedSymbol),
                ("123", ParserTokenKind::Numeral),
                (";", ParserTokenKind::ReservedSymbol),
            ],
        );
        let bad_range = tokens[3].span;
        let output = parse(ParseRequest::new(
            source_id,
            Edition::new("2026"),
            tokens,
            Vec::new(),
        ));

        assert_eq!(output.diagnostics.len(), 1);
        assert_eq!(
            output.diagnostics[0].code,
            SyntaxDiagnosticCode::MalformedExport
        );
        assert_eq!(output.diagnostics[0].primary, bad_range);
        let ast = output
            .ast
            .expect("bad export path after comma should recover inside export item");
        let export_item = single_node(&ast, |kind| matches!(kind, SurfaceNodeKind::ExportItem));
        let recovery = export_item
            .children
            .iter()
            .filter_map(|id| ast.node(*id))
            .find(|node| {
                matches!(
                    node.kind,
                    SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::SkippedToken)
                )
            })
            .expect("bad comma tail should be owned by export recovery");
        assert_eq!(recovery.range, bad_range);
        assert_eq!(
            rowan_token_texts(&ast),
            vec!["export", "Std", ",", "123", ";"]
        );
    }

    #[test]
    fn parser_diagnoses_missing_export_semicolon_before_next_item() {
        let source_id = source_id(85);
        let tokens = token_sequence(
            source_id,
            &[
                ("export", ParserTokenKind::ReservedWord),
                ("Std", ParserTokenKind::Identifier),
                ("theorem", ParserTokenKind::ReservedWord),
                ("T", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
            ],
        );
        let theorem_range = tokens[2].span;
        let output = parse(ParseRequest::new(
            source_id,
            Edition::new("2026"),
            tokens,
            Vec::new(),
        ));

        assert_eq!(output.diagnostics.len(), 1);
        assert_eq!(
            output.diagnostics[0].code,
            SyntaxDiagnosticCode::MissingSemicolon
        );
        assert_eq!(output.diagnostics[0].primary, theorem_range);
        let ast = output.ast.expect("missing export semicolon should recover");
        let item_list = single_node(&ast, |kind| matches!(kind, SurfaceNodeKind::ItemList));
        assert_eq!(item_list.children.len(), 2);
        assert!(matches!(
            ast.node(item_list.children[0]).unwrap().kind,
            SurfaceNodeKind::ExportItem
        ));
        assert!(matches!(
            ast.node(item_list.children[1]).unwrap().kind,
            SurfaceNodeKind::PlaceholderItem
        ));
    }

    #[test]
    fn parser_recovers_import_after_export_prelude() {
        let source_id = source_id(91);
        let tokens = token_sequence(
            source_id,
            &[
                ("export", ParserTokenKind::ReservedWord),
                ("Facade", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
                ("import", ParserTokenKind::ReservedWord),
                ("Late", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
            ],
        );
        let import_range = tokens[3].span;
        let output = parse(ParseRequest::new(
            source_id,
            Edition::new("2026"),
            tokens,
            Vec::new(),
        ));

        assert_eq!(output.diagnostics.len(), 1);
        assert_eq!(
            output.diagnostics[0].code,
            SyntaxDiagnosticCode::UnexpectedTopLevelToken
        );
        assert_eq!(output.diagnostics[0].primary, import_range);
        let ast = output.ast.expect("late import after export should recover");
        let item_list = single_node(&ast, |kind| matches!(kind, SurfaceNodeKind::ItemList));
        assert_eq!(item_list.children.len(), 2);
        assert!(matches!(
            ast.node(item_list.children[0]).unwrap().kind,
            SurfaceNodeKind::ExportItem
        ));
        assert!(matches!(
            ast.node(item_list.children[1]).unwrap().kind,
            SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::SkippedToken)
        ));
    }

    #[test]
    fn parser_wraps_annotated_visible_item_in_source_order() {
        let source_id = source_id(86);
        let tokens = token_sequence(
            source_id,
            &[
                ("@[", ParserTokenKind::ReservedSymbol),
                ("label", ParserTokenKind::Identifier),
                ("]", ParserTokenKind::ReservedSymbol),
                ("private", ParserTokenKind::ReservedWord),
                ("theorem", ParserTokenKind::ReservedWord),
                ("T", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
            ],
        );
        let expected_range = range(source_id, 0, tokens.last().unwrap().span.end);
        let target_range = range(source_id, tokens[4].span.start, tokens[6].span.end);
        let output = parse(ParseRequest::new(
            source_id,
            Edition::new("2026"),
            tokens,
            Vec::new(),
        ));

        assert!(output.diagnostics.is_empty());
        let ast = output.ast.expect("annotated visible item should parse");
        let visible = single_node(&ast, |kind| matches!(kind, SurfaceNodeKind::VisibleItem));
        assert_eq!(visible.range, expected_range);
        assert_eq!(visible.children.len(), 5);
        assert_eq!(
            ast.node(visible.children[0]).unwrap().token_text(),
            Some("@[")
        );
        assert_eq!(
            ast.node(visible.children[1]).unwrap().token_text(),
            Some("label")
        );
        assert_eq!(
            ast.node(visible.children[2]).unwrap().token_text(),
            Some("]")
        );
        let marker = ast.node(visible.children[3]).unwrap();
        assert!(matches!(marker.kind, SurfaceNodeKind::VisibilityMarker));
        assert_eq!(
            ast.node(marker.children[0]).unwrap().token_text(),
            Some("private")
        );
        let item = ast.node(visible.children[4]).unwrap();
        assert!(matches!(item.kind, SurfaceNodeKind::PlaceholderItem));
        assert_eq!(item.range, target_range);
    }

    #[test]
    fn parser_diagnoses_dangling_visibility_marker_inside_visible_item() {
        let source_id = source_id(92);
        let tokens = token_sequence(
            source_id,
            &[
                ("private", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
            ],
        );
        let semicolon_range = tokens[1].span;
        let output = parse(ParseRequest::new(
            source_id,
            Edition::new("2026"),
            tokens,
            Vec::new(),
        ));

        assert_eq!(output.diagnostics.len(), 1);
        assert_eq!(
            output.diagnostics[0].code,
            SyntaxDiagnosticCode::MalformedVisibility
        );
        assert_eq!(output.diagnostics[0].primary, semicolon_range);
        let ast = output
            .ast
            .expect("dangling visibility should recover inside visible item");
        let visible = single_node(&ast, |kind| matches!(kind, SurfaceNodeKind::VisibleItem));
        assert_eq!(visible.children.len(), 2);
        assert!(matches!(
            ast.node(visible.children[0]).unwrap().kind,
            SurfaceNodeKind::VisibilityMarker
        ));
        assert_eq!(
            ast.node(visible.children[1]).unwrap().token_text(),
            Some(";")
        );
    }

    #[test]
    fn parser_diagnoses_duplicate_visibility_marker_inside_visible_item() {
        let source_id = source_id(87);
        let tokens = token_sequence(
            source_id,
            &[
                ("public", ParserTokenKind::ReservedWord),
                ("private", ParserTokenKind::ReservedWord),
                ("theorem", ParserTokenKind::ReservedWord),
                ("T", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
            ],
        );
        let duplicate_range = tokens[1].span;
        let skipped_range = range(source_id, tokens[1].span.start, tokens[3].span.end);
        let output = parse(ParseRequest::new(
            source_id,
            Edition::new("2026"),
            tokens,
            Vec::new(),
        ));

        assert_eq!(output.diagnostics.len(), 1);
        assert_eq!(
            output.diagnostics[0].code,
            SyntaxDiagnosticCode::MalformedVisibility
        );
        assert_eq!(output.diagnostics[0].primary, duplicate_range);
        let ast = output
            .ast
            .expect("duplicate visibility should recover inside one visible item");
        let item_list = single_node(&ast, |kind| matches!(kind, SurfaceNodeKind::ItemList));
        assert_eq!(item_list.children.len(), 1);
        let visible = ast.node(item_list.children[0]).unwrap();
        assert!(matches!(visible.kind, SurfaceNodeKind::VisibleItem));
        assert_eq!(visible.children.len(), 3);
        assert!(matches!(
            ast.node(visible.children[0]).unwrap().kind,
            SurfaceNodeKind::VisibilityMarker
        ));
        let recovery = ast.node(visible.children[1]).unwrap();
        assert!(matches!(
            recovery.kind,
            SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::SkippedToken)
        ));
        assert_eq!(recovery.range, skipped_range);
        assert_eq!(
            ast.node(visible.children[2]).unwrap().token_text(),
            Some(";")
        );
        assert_eq!(
            rowan_token_texts(&ast),
            vec!["public", "private", "theorem", "T", ";"]
        );
    }

    #[test]
    fn parser_recovers_block_like_invalid_visibility_target_as_one_visible_item() {
        let source_id = source_id(93);
        let tokens = token_sequence(
            source_id,
            &[
                ("private", ParserTokenKind::ReservedWord),
                ("definition", ParserTokenKind::ReservedWord),
                ("theorem", ParserTokenKind::ReservedWord),
                ("T", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
                ("end", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("theorem", ParserTokenKind::ReservedWord),
                ("U", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
            ],
        );
        let definition_range = tokens[1].span;
        let skipped_range = range(source_id, tokens[1].span.start, tokens[5].span.end);
        let output = parse(ParseRequest::new(
            source_id,
            Edition::new("2026"),
            tokens,
            Vec::new(),
        ));

        assert_eq!(output.diagnostics.len(), 1);
        assert_eq!(
            output.diagnostics[0].code,
            SyntaxDiagnosticCode::MalformedVisibility
        );
        assert_eq!(output.diagnostics[0].primary, definition_range);
        let ast = output
            .ast
            .expect("block-like invalid visible target should recover inside one visible item");
        let item_list = single_node(&ast, |kind| matches!(kind, SurfaceNodeKind::ItemList));
        assert_eq!(item_list.children.len(), 2);
        let visible = ast.node(item_list.children[0]).unwrap();
        assert!(matches!(visible.kind, SurfaceNodeKind::VisibleItem));
        let recovery = visible
            .children
            .iter()
            .filter_map(|id| ast.node(*id))
            .find(|node| {
                matches!(
                    node.kind,
                    SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::SkippedToken)
                )
            })
            .expect("invalid definition target should be owned by visible recovery");
        assert_eq!(recovery.range, skipped_range);
        assert!(matches!(
            ast.node(item_list.children[1]).unwrap().kind,
            SurfaceNodeKind::PlaceholderItem
        ));
    }

    #[test]
    fn parser_recovers_prefixed_block_like_invalid_visibility_target_as_one_visible_item() {
        let source_id = source_id(95);
        let tokens = token_sequence(
            source_id,
            &[
                ("public", ParserTokenKind::ReservedWord),
                ("private", ParserTokenKind::ReservedWord),
                ("open", ParserTokenKind::ReservedWord),
                ("definition", ParserTokenKind::ReservedWord),
                ("theorem", ParserTokenKind::ReservedWord),
                ("T", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
                ("end", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("theorem", ParserTokenKind::ReservedWord),
                ("U", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
            ],
        );
        let duplicate_range = tokens[1].span;
        let skipped_range = range(source_id, tokens[1].span.start, tokens[7].span.end);
        let output = parse(ParseRequest::new(
            source_id,
            Edition::new("2026"),
            tokens,
            Vec::new(),
        ));

        assert_eq!(output.diagnostics.len(), 1);
        assert_eq!(
            output.diagnostics[0].code,
            SyntaxDiagnosticCode::MalformedVisibility
        );
        assert_eq!(output.diagnostics[0].primary, duplicate_range);
        let ast = output
            .ast
            .expect("prefixed block-like invalid visible target should recover inside one item");
        let item_list = single_node(&ast, |kind| matches!(kind, SurfaceNodeKind::ItemList));
        assert_eq!(item_list.children.len(), 2);
        let visible = ast.node(item_list.children[0]).unwrap();
        assert!(matches!(visible.kind, SurfaceNodeKind::VisibleItem));
        let recovery = visible
            .children
            .iter()
            .filter_map(|id| ast.node(*id))
            .find(|node| {
                matches!(
                    node.kind,
                    SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::SkippedToken)
                )
            })
            .expect("prefixed invalid definition target should be owned by visible recovery");
        assert_eq!(recovery.range, skipped_range);
        assert_eq!(
            ast.node(visible.children[2]).unwrap().token_text(),
            Some(";")
        );
        assert!(matches!(
            ast.node(item_list.children[1]).unwrap().kind,
            SurfaceNodeKind::PlaceholderItem
        ));
    }

    #[test]
    fn parser_diagnoses_visibility_on_non_visible_top_level_item() {
        let source_id = source_id(88);
        let tokens = token_sequence(
            source_id,
            &[
                ("private", ParserTokenKind::ReservedWord),
                ("reserve", ParserTokenKind::ReservedWord),
                ("x", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
            ],
        );
        let reserve_range = tokens[1].span;
        let output = parse(ParseRequest::new(
            source_id,
            Edition::new("2026"),
            tokens,
            Vec::new(),
        ));

        assert_eq!(output.diagnostics.len(), 1);
        assert_eq!(
            output.diagnostics[0].code,
            SyntaxDiagnosticCode::MalformedVisibility
        );
        assert_eq!(output.diagnostics[0].primary, reserve_range);
        let ast = output
            .ast
            .expect("invalid visible target should recover inside visible item");
        let visible = single_node(&ast, |kind| matches!(kind, SurfaceNodeKind::VisibleItem));
        assert_eq!(visible.children.len(), 3);
        assert!(matches!(
            ast.node(visible.children[1]).unwrap().kind,
            SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::SkippedToken)
        ));
    }

    #[test]
    fn parser_parses_import_alias_relative_and_branch_items() {
        let source_id = source_id(74);
        let tokens = token_sequence(
            source_id,
            &[
                ("import", ParserTokenKind::ReservedWord),
                ("..", ParserTokenKind::ReservedSymbol),
                ("Core", ParserTokenKind::Identifier),
                (".", ParserTokenKind::ReservedSymbol),
                ("Algebra", ParserTokenKind::Identifier),
                (".{", ParserTokenKind::ReservedSymbol),
                ("Group", ParserTokenKind::Identifier),
                (",", ParserTokenKind::ReservedSymbol),
                ("Ring", ParserTokenKind::Identifier),
                ("}", ParserTokenKind::ReservedSymbol),
                (",", ParserTokenKind::ReservedSymbol),
                (".", ParserTokenKind::ReservedSymbol),
                ("Tools", ParserTokenKind::Identifier),
                ("as", ParserTokenKind::ReservedWord),
                ("T", ParserTokenKind::Identifier),
                (",", ParserTokenKind::ReservedSymbol),
                ("Std", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
                ("theorem", ParserTokenKind::ReservedWord),
                ("A", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
            ],
        );
        let output = parse(ParseRequest::new(
            source_id,
            Edition::new("2026"),
            tokens,
            Vec::new(),
        ));

        assert!(output.diagnostics.is_empty());
        let ast = output.ast.expect("import item should parse");
        let item_list = single_node(&ast, |kind| matches!(kind, SurfaceNodeKind::ItemList));
        assert_eq!(item_list.children.len(), 2);
        let import_item = ast.node(item_list.children[0]).unwrap();
        assert!(matches!(import_item.kind, SurfaceNodeKind::ImportItem));
        assert_eq!(import_item.children.len(), 7);
        assert_eq!(
            ast.node(import_item.children[0]).unwrap().token_text(),
            Some("import")
        );
        assert!(matches!(
            ast.node(import_item.children[1]).unwrap().kind,
            SurfaceNodeKind::ModuleBranchImport
        ));
        assert_eq!(
            ast.node(import_item.children[2]).unwrap().token_text(),
            Some(",")
        );
        assert!(matches!(
            ast.node(import_item.children[3]).unwrap().kind,
            SurfaceNodeKind::ImportAliasDecl
        ));
        assert_eq!(
            ast.node(import_item.children[4]).unwrap().token_text(),
            Some(",")
        );
        assert!(matches!(
            ast.node(import_item.children[5]).unwrap().kind,
            SurfaceNodeKind::ImportAliasDecl
        ));
        assert_eq!(
            ast.node(import_item.children[6]).unwrap().token_text(),
            Some(";")
        );
        assert!(matches!(
            ast.node(item_list.children[1]).unwrap().kind,
            SurfaceNodeKind::PlaceholderItem
        ));

        let branch = import_item
            .children
            .iter()
            .filter_map(|id| ast.node(*id))
            .find(|node| matches!(node.kind, SurfaceNodeKind::ModuleBranchImport))
            .expect("branch import decl should be a concrete child");
        assert_eq!(branch.children.len(), 6);
        assert!(matches!(
            ast.node(branch.children[0]).unwrap().kind,
            SurfaceNodeKind::ModulePath
        ));
        assert_eq!(
            ast.node(branch.children[1]).unwrap().token_text(),
            Some(".{")
        );
        assert!(matches!(
            ast.node(branch.children[2]).unwrap().kind,
            SurfaceNodeKind::PathSegment
        ));
        assert_eq!(
            ast.node(branch.children[3]).unwrap().token_text(),
            Some(",")
        );
        assert!(matches!(
            ast.node(branch.children[4]).unwrap().kind,
            SurfaceNodeKind::PathSegment
        ));
        assert_eq!(
            ast.node(branch.children[5]).unwrap().token_text(),
            Some("}")
        );

        let alias = import_item
            .children
            .iter()
            .filter_map(|id| ast.node(*id))
            .find(|node| matches!(node.kind, SurfaceNodeKind::ImportAliasDecl))
            .expect("alias import decl should be a concrete child");
        assert_eq!(alias.children.len(), 3);
        assert!(matches!(
            ast.node(alias.children[0]).unwrap().kind,
            SurfaceNodeKind::ModulePath
        ));
        assert_eq!(
            ast.node(alias.children[1]).unwrap().token_text(),
            Some("as")
        );
        assert!(matches!(
            ast.node(alias.children[2]).unwrap().kind,
            SurfaceNodeKind::PathSegment
        ));

        let simple = import_item
            .children
            .iter()
            .filter_map(|id| ast.node(*id))
            .filter(|node| matches!(node.kind, SurfaceNodeKind::ImportAliasDecl))
            .find(|node| node.children.len() == 1)
            .expect("plain module-path import should be an alias decl without alias children");
        assert!(matches!(
            ast.node(simple.children[0]).unwrap().kind,
            SurfaceNodeKind::ModulePath
        ));
    }

    #[test]
    fn parser_recovers_unexpected_top_level_tokens_before_next_item() {
        let source_id = source_id(41);
        let output = parse(ParseRequest::new(
            source_id,
            Edition::new("2026"),
            vec![
                token(source_id, ParserTokenKind::Identifier, "bad", 0, 3),
                token(source_id, ParserTokenKind::ReservedWord, "theorem", 4, 11),
                token(source_id, ParserTokenKind::Identifier, "T", 12, 13),
                token(source_id, ParserTokenKind::ReservedSymbol, ";", 13, 14),
            ],
            Vec::new(),
        ));

        assert_eq!(output.diagnostics.len(), 1);
        assert_eq!(
            output.diagnostics[0].code,
            SyntaxDiagnosticCode::UnexpectedTopLevelToken
        );
        assert_eq!(output.diagnostics[0].primary, range(source_id, 0, 3));
        let ast = output
            .ast
            .expect("unexpected top-level token should recover");
        let item_list = single_node(&ast, |kind| matches!(kind, SurfaceNodeKind::ItemList));
        assert_eq!(item_list.children.len(), 2);
        let skipped = ast.node(item_list.children[0]).unwrap();
        assert!(matches!(
            skipped.kind,
            SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::SkippedToken)
        ));
        assert_eq!(skipped.range, range(source_id, 0, 3));
        let item = ast.node(item_list.children[1]).unwrap();
        assert!(matches!(item.kind, SurfaceNodeKind::PlaceholderItem));
        assert_eq!(item.range, range(source_id, 4, 14));
        let recovery = ast
            .nodes()
            .iter()
            .find(|node| {
                matches!(
                    node.kind,
                    SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::SkippedToken)
                )
            })
            .expect("skipped token recovery should be present");
        assert_eq!(recovery.range, range(source_id, 0, 3));
        assert_eq!(ast.trivia().skipped_token_ranges().len(), 1);
        assert_eq!(
            ast.trivia().skipped_token_ranges()[0].reason,
            SkippedTokenReason::Recovery
        );
        assert_eq!(
            ast.trivia().skipped_token_ranges()[0].range,
            range(source_id, 0, 3)
        );
    }

    #[test]
    fn parser_recovers_import_after_non_import_item() {
        let source_id = source_id(75);
        let tokens = token_sequence(
            source_id,
            &[
                ("theorem", ParserTokenKind::ReservedWord),
                ("T", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
                ("import", ParserTokenKind::ReservedWord),
                ("Late", ParserTokenKind::Identifier),
                (".", ParserTokenKind::ReservedSymbol),
                ("Module", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
                ("theorem", ParserTokenKind::ReservedWord),
                ("U", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
            ],
        );
        let import_token_range = tokens[3].span;
        let skipped_range = range(source_id, tokens[3].span.start, tokens[7].span.end);
        let output = parse(ParseRequest::new(
            source_id,
            Edition::new("2026"),
            tokens,
            Vec::new(),
        ));

        assert_eq!(output.diagnostics.len(), 1);
        assert_eq!(
            output.diagnostics[0].code,
            SyntaxDiagnosticCode::UnexpectedTopLevelToken
        );
        assert_eq!(output.diagnostics[0].primary, import_token_range);
        let ast = output
            .ast
            .expect("late import should recover as skipped tokens");
        let item_list = single_node(&ast, |kind| matches!(kind, SurfaceNodeKind::ItemList));
        assert_eq!(item_list.children.len(), 3);
        assert!(matches!(
            ast.node(item_list.children[0]).unwrap().kind,
            SurfaceNodeKind::PlaceholderItem
        ));
        let skipped = ast.node(item_list.children[1]).unwrap();
        assert!(matches!(
            skipped.kind,
            SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::SkippedToken)
        ));
        assert_eq!(skipped.range, skipped_range);
        assert!(matches!(
            ast.node(item_list.children[2]).unwrap().kind,
            SurfaceNodeKind::PlaceholderItem
        ));
    }

    #[test]
    fn parser_diagnoses_missing_import_semicolon_before_next_item() {
        let source_id = source_id(76);
        let tokens = token_sequence(
            source_id,
            &[
                ("import", ParserTokenKind::ReservedWord),
                ("Std", ParserTokenKind::Identifier),
                ("theorem", ParserTokenKind::ReservedWord),
                ("T", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
            ],
        );
        let theorem_range = tokens[2].span;
        let output = parse(ParseRequest::new(
            source_id,
            Edition::new("2026"),
            tokens,
            Vec::new(),
        ));

        assert_eq!(output.diagnostics.len(), 1);
        assert_eq!(
            output.diagnostics[0].code,
            SyntaxDiagnosticCode::MissingSemicolon
        );
        assert_eq!(output.diagnostics[0].primary, theorem_range);
        let ast = output.ast.expect("missing import semicolon should recover");
        let item_list = single_node(&ast, |kind| matches!(kind, SurfaceNodeKind::ItemList));
        assert_eq!(item_list.children.len(), 2);
        assert!(matches!(
            ast.node(item_list.children[0]).unwrap().kind,
            SurfaceNodeKind::ImportItem
        ));
        assert!(matches!(
            ast.node(item_list.children[1]).unwrap().kind,
            SurfaceNodeKind::PlaceholderItem
        ));
    }

    #[test]
    fn parser_diagnoses_missing_import_alias_after_as() {
        let source_id = source_id(77);
        let tokens = token_sequence(
            source_id,
            &[
                ("import", ParserTokenKind::ReservedWord),
                ("Std", ParserTokenKind::Identifier),
                ("as", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
            ],
        );
        let semicolon_range = tokens[3].span;
        let output = parse(ParseRequest::new(
            source_id,
            Edition::new("2026"),
            tokens,
            Vec::new(),
        ));

        assert_eq!(output.diagnostics.len(), 1);
        assert_eq!(
            output.diagnostics[0].code,
            SyntaxDiagnosticCode::MalformedImport
        );
        assert_eq!(output.diagnostics[0].primary, semicolon_range);
        let ast = output.ast.expect("malformed import alias should recover");
        let import_item = single_node(&ast, |kind| matches!(kind, SurfaceNodeKind::ImportItem));
        let alias = import_item
            .children
            .iter()
            .filter_map(|id| ast.node(*id))
            .find(|node| matches!(node.kind, SurfaceNodeKind::ImportAliasDecl))
            .expect("malformed alias decl should still be represented");
        assert_eq!(alias.children.len(), 2);
        assert_eq!(
            ast.node(alias.children[1]).unwrap().token_text(),
            Some("as")
        );
    }

    #[test]
    fn parser_recovers_bad_import_path_tail_inside_import_item() {
        let source_id = source_id(79);
        let tokens = token_sequence(
            source_id,
            &[
                ("import", ParserTokenKind::ReservedWord),
                ("123", ParserTokenKind::Numeral),
                (";", ParserTokenKind::ReservedSymbol),
                ("theorem", ParserTokenKind::ReservedWord),
                ("T", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
            ],
        );
        let bad_range = tokens[1].span;
        let output = parse(ParseRequest::new(
            source_id,
            Edition::new("2026"),
            tokens,
            Vec::new(),
        ));

        assert_eq!(output.diagnostics.len(), 1);
        assert_eq!(
            output.diagnostics[0].code,
            SyntaxDiagnosticCode::MalformedImport
        );
        assert_eq!(output.diagnostics[0].primary, bad_range);
        let ast = output
            .ast
            .expect("bad import path should recover inside import item");
        let item_list = single_node(&ast, |kind| matches!(kind, SurfaceNodeKind::ItemList));
        assert_eq!(item_list.children.len(), 2);
        let import_item = ast.node(item_list.children[0]).unwrap();
        assert!(matches!(import_item.kind, SurfaceNodeKind::ImportItem));
        let recovery = import_item
            .children
            .iter()
            .filter_map(|id| ast.node(*id))
            .find(|node| {
                matches!(
                    node.kind,
                    SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::SkippedToken)
                )
            })
            .expect("bad import path token should be owned by a recovery node");
        assert_eq!(recovery.range, bad_range);
        assert_eq!(ast.trivia().skipped_token_ranges().len(), 1);
        assert_eq!(ast.trivia().skipped_token_ranges()[0].range, bad_range);
        assert_eq!(
            rowan_token_texts(&ast),
            vec!["import", "123", ";", "theorem", "T", ";"]
        );
    }

    #[test]
    fn parser_recovers_bad_alias_tail_without_splitting_a_phantom_item() {
        let source_id = source_id(80);
        let tokens = token_sequence(
            source_id,
            &[
                ("import", ParserTokenKind::ReservedWord),
                ("Std", ParserTokenKind::Identifier),
                ("as", ParserTokenKind::ReservedWord),
                ("theorem", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
            ],
        );
        let bad_range = tokens[3].span;
        let output = parse(ParseRequest::new(
            source_id,
            Edition::new("2026"),
            tokens,
            Vec::new(),
        ));

        assert_eq!(output.diagnostics.len(), 1);
        assert_eq!(
            output.diagnostics[0].code,
            SyntaxDiagnosticCode::MalformedImport
        );
        assert_eq!(output.diagnostics[0].primary, bad_range);
        let ast = output
            .ast
            .expect("bad alias tail should recover inside import item");
        let item_list = single_node(&ast, |kind| matches!(kind, SurfaceNodeKind::ItemList));
        assert_eq!(
            item_list.children.len(),
            1,
            "bad alias token must not become a phantom theorem item"
        );
        let import_item = ast.node(item_list.children[0]).unwrap();
        let recovery = import_item
            .children
            .iter()
            .filter_map(|id| ast.node(*id))
            .flat_map(|node| node.children.iter().filter_map(|child| ast.node(*child)))
            .find(|node| {
                matches!(
                    node.kind,
                    SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::SkippedToken)
                )
            })
            .expect("bad alias token should be owned by a nested recovery node");
        assert_eq!(recovery.range, bad_range);
    }

    #[test]
    fn parser_diagnoses_missing_branch_import_close() {
        let source_id = source_id(78);
        let tokens = token_sequence(
            source_id,
            &[
                ("import", ParserTokenKind::ReservedWord),
                ("Std", ParserTokenKind::Identifier),
                (".{", ParserTokenKind::ReservedSymbol),
                ("Branch", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
                ("theorem", ParserTokenKind::ReservedWord),
                ("T", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
            ],
        );
        let semicolon_range = tokens[4].span;
        let output = parse(ParseRequest::new(
            source_id,
            Edition::new("2026"),
            tokens,
            Vec::new(),
        ));

        assert_eq!(output.diagnostics.len(), 1);
        assert_eq!(
            output.diagnostics[0].code,
            SyntaxDiagnosticCode::MalformedImport
        );
        assert_eq!(output.diagnostics[0].primary, semicolon_range);
        let ast = output
            .ast
            .expect("missing branch close should recover at semicolon");
        let item_list = single_node(&ast, |kind| matches!(kind, SurfaceNodeKind::ItemList));
        assert_eq!(item_list.children.len(), 2);
        assert!(matches!(
            ast.node(item_list.children[0]).unwrap().kind,
            SurfaceNodeKind::ImportItem
        ));
        assert!(matches!(
            ast.node(item_list.children[1]).unwrap().kind,
            SurfaceNodeKind::PlaceholderItem
        ));
    }

    #[test]
    fn parser_reports_empty_branch_import_once() {
        let source_id = source_id(81);
        let tokens = token_sequence(
            source_id,
            &[
                ("import", ParserTokenKind::ReservedWord),
                ("Std", ParserTokenKind::Identifier),
                (".{", ParserTokenKind::ReservedSymbol),
                (";", ParserTokenKind::ReservedSymbol),
            ],
        );
        let semicolon_range = tokens[3].span;
        let output = parse(ParseRequest::new(
            source_id,
            Edition::new("2026"),
            tokens,
            Vec::new(),
        ));

        assert_eq!(output.diagnostics.len(), 1);
        assert_eq!(
            output.diagnostics[0].code,
            SyntaxDiagnosticCode::MalformedImport
        );
        assert_eq!(output.diagnostics[0].primary, semicolon_range);
        let ast = output.ast.expect("empty branch import should recover");
        let branch = single_node(&ast, |kind| {
            matches!(kind, SurfaceNodeKind::ModuleBranchImport)
        });
        assert_eq!(branch.children.len(), 2);
    }

    #[test]
    fn parser_diagnoses_missing_semicolon_before_next_item_boundary() {
        let source_id = source_id(42);
        let output = parse(ParseRequest::new(
            source_id,
            Edition::new("2026"),
            vec![
                token(source_id, ParserTokenKind::ReservedWord, "theorem", 0, 7),
                token(source_id, ParserTokenKind::Identifier, "T", 8, 9),
                token(source_id, ParserTokenKind::ReservedWord, "lemma", 10, 15),
                token(source_id, ParserTokenKind::Identifier, "L", 16, 17),
                token(source_id, ParserTokenKind::ReservedSymbol, ";", 17, 18),
            ],
            Vec::new(),
        ));

        assert_eq!(output.diagnostics.len(), 1);
        assert_eq!(
            output.diagnostics[0].code,
            SyntaxDiagnosticCode::MissingSemicolon
        );
        assert_eq!(output.diagnostics[0].primary, range(source_id, 10, 15));
        let ast = output.ast.expect("missing semicolon should recover");
        let item_list = single_node(&ast, |kind| matches!(kind, SurfaceNodeKind::ItemList));
        assert_eq!(item_list.children.len(), 2);
        assert_eq!(
            ast.node(item_list.children[0]).unwrap().range,
            range(source_id, 0, 9)
        );
        assert_eq!(
            ast.node(item_list.children[1]).unwrap().range,
            range(source_id, 10, 18)
        );
    }

    #[test]
    fn block_placeholder_requires_semicolon_after_matching_end() {
        let source_id = source_id(43);
        let output = parse(ParseRequest::new(
            source_id,
            Edition::new("2026"),
            vec![
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
                token(source_id, ParserTokenKind::ReservedWord, "end", 25, 28),
                token(source_id, ParserTokenKind::ReservedWord, "theorem", 29, 36),
                token(source_id, ParserTokenKind::Identifier, "T", 37, 38),
                token(source_id, ParserTokenKind::ReservedSymbol, ";", 38, 39),
            ],
            Vec::new(),
        ));

        assert_eq!(output.diagnostics.len(), 1);
        assert_eq!(
            output.diagnostics[0].code,
            SyntaxDiagnosticCode::MissingSemicolon
        );
        assert_eq!(output.diagnostics[0].primary, range(source_id, 29, 36));
        let ast = output.ast.expect("missing block semicolon should recover");
        let item_list = single_node(&ast, |kind| matches!(kind, SurfaceNodeKind::ItemList));
        assert_eq!(item_list.children.len(), 2);
        assert_eq!(
            ast.node(item_list.children[0]).unwrap().range,
            range(source_id, 0, 28)
        );
        assert_eq!(
            ast.node(item_list.children[1]).unwrap().range,
            range(source_id, 29, 39)
        );
    }

    #[test]
    fn proof_block_semicolons_stay_inside_the_theorem_item() {
        let source_id = source_id(68);
        let tokens = token_sequence(
            source_id,
            &[
                ("theorem", ParserTokenKind::ReservedWord),
                ("T", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("thesis", ParserTokenKind::ReservedWord),
                ("proof", ParserTokenKind::ReservedWord),
                ("thus", ParserTokenKind::ReservedWord),
                ("thesis", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("end", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
            ],
        );
        let expected_range = range(source_id, 0, tokens.last().unwrap().span.end);
        let output = parse(ParseRequest::new(
            source_id,
            Edition::new("2026"),
            tokens,
            Vec::new(),
        ));

        assert!(output.diagnostics.is_empty());
        let ast = output.ast.expect("proof-bearing theorem should parse");
        let item_list = single_node(&ast, |kind| matches!(kind, SurfaceNodeKind::ItemList));
        assert_eq!(item_list.children.len(), 1);
        assert_eq!(
            ast.node(item_list.children[0]).unwrap().range,
            expected_range
        );
        assert!(matches!(
            ast.node(item_list.children[0]).unwrap().kind,
            SurfaceNodeKind::TheoremItem
        ));
        assert_eq!(
            count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::ProofBlock)),
            1
        );
        assert_eq!(
            count_nodes(&ast, |kind| matches!(
                kind,
                SurfaceNodeKind::ConclusionStatement
            )),
            1
        );
    }

    #[test]
    fn parser_wraps_visible_theorem_targets_and_preserves_status_tokens() {
        let source_id = source_id(171);
        let output = parse(ParseRequest::new(
            source_id,
            Edition::new("2026"),
            token_sequence(
                source_id,
                &[
                    ("public", ParserTokenKind::ReservedWord),
                    ("open", ParserTokenKind::ReservedWord),
                    ("theorem", ParserTokenKind::ReservedWord),
                    ("VisibleT", ParserTokenKind::Identifier),
                    (":", ParserTokenKind::ReservedSymbol),
                    ("thesis", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("private", ParserTokenKind::ReservedWord),
                    ("conditional", ParserTokenKind::ReservedWord),
                    ("lemma", ParserTokenKind::ReservedWord),
                    ("VisibleL", ParserTokenKind::Identifier),
                    (":", ParserTokenKind::ReservedSymbol),
                    ("thesis", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("assumed", ParserTokenKind::ReservedWord),
                    ("theorem", ParserTokenKind::ReservedWord),
                    ("AssumedT", ParserTokenKind::Identifier),
                    (":", ParserTokenKind::ReservedSymbol),
                    ("thesis", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                ],
            ),
            Vec::new(),
        ));

        assert!(
            output.diagnostics.is_empty(),
            "visible theorem targets should parse without diagnostics: {:?}",
            output.diagnostics
        );
        let ast = output
            .ast
            .expect("visible theorem targets should keep an AST");
        let item_list = single_node(&ast, |kind| matches!(kind, SurfaceNodeKind::ItemList));
        assert_eq!(item_list.children.len(), 3);
        let visible_items = item_list
            .children
            .iter()
            .take(2)
            .map(|item| ast.node(*item).unwrap())
            .collect::<Vec<_>>();
        assert!(
            visible_items
                .iter()
                .all(|item| matches!(item.kind, SurfaceNodeKind::VisibleItem))
        );

        let assumed = ast.node(item_list.children[2]).unwrap();
        assert!(matches!(assumed.kind, SurfaceNodeKind::TheoremItem));
        assert_eq!(
            direct_token_texts(&ast, assumed),
            vec!["assumed", "theorem", "AssumedT", ":", ";"]
        );
        assert!(direct_child_has_kind(&ast, assumed, |kind| {
            matches!(kind, SurfaceNodeKind::FormulaExpression)
        }));

        let theorem = structural_children(&ast, visible_items[0])
            .into_iter()
            .find(|child| matches!(child.kind, SurfaceNodeKind::TheoremItem))
            .expect("public open theorem should be the visible target");
        assert_eq!(
            direct_token_texts(&ast, theorem),
            vec!["open", "theorem", "VisibleT", ":", ";"]
        );
        assert!(direct_child_has_kind(&ast, theorem, |kind| {
            matches!(kind, SurfaceNodeKind::FormulaExpression)
        }));

        let lemma = structural_children(&ast, visible_items[1])
            .into_iter()
            .find(|child| matches!(child.kind, SurfaceNodeKind::LemmaItem))
            .expect("private conditional lemma should be the visible target");
        assert_eq!(
            direct_token_texts(&ast, lemma),
            vec!["conditional", "lemma", "VisibleL", ":", ";"]
        );
        assert!(direct_child_has_kind(&ast, lemma, |kind| {
            matches!(kind, SurfaceNodeKind::FormulaExpression)
        }));
    }

    #[test]
    fn statement_proof_blocks_are_owned_by_statement_hosts() {
        let source_id = source_id(172);
        let output = parse(ParseRequest::new(
            source_id,
            Edition::new("2026"),
            token_sequence(
                source_id,
                &[
                    ("theorem", ParserTokenKind::ReservedWord),
                    ("StatementProofs", ParserTokenKind::Identifier),
                    (":", ParserTokenKind::ReservedSymbol),
                    ("thesis", ParserTokenKind::ReservedWord),
                    ("proof", ParserTokenKind::ReservedWord),
                    ("A", ParserTokenKind::Identifier),
                    (":", ParserTokenKind::ReservedSymbol),
                    ("thesis", ParserTokenKind::ReservedWord),
                    ("proof", ParserTokenKind::ReservedWord),
                    ("thus", ParserTokenKind::ReservedWord),
                    ("thesis", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("end", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("thus", ParserTokenKind::ReservedWord),
                    ("thesis", ParserTokenKind::ReservedWord),
                    ("proof", ParserTokenKind::ReservedWord),
                    ("thus", ParserTokenKind::ReservedWord),
                    ("thesis", ParserTokenKind::ReservedWord),
                    ("by", ParserTokenKind::ReservedWord),
                    ("A", ParserTokenKind::Identifier),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("end", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("end", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                ],
            ),
            Vec::new(),
        ));

        assert!(
            output.diagnostics.is_empty(),
            "statement-owned proof blocks should parse without diagnostics: {:?}",
            output.diagnostics
        );
        let ast = output
            .ast
            .expect("statement-owned proof blocks should keep an AST");
        assert_eq!(
            count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::ProofBlock)),
            3,
            "the theorem proof and both statement proof tails should be represented"
        );

        let compact = single_node(&ast, |kind| {
            matches!(kind, SurfaceNodeKind::CompactStatement)
        });
        let compact_children = structural_children(&ast, compact);
        assert!(
            compact_children
                .iter()
                .any(|child| matches!(child.kind, SurfaceNodeKind::ProofBlock))
        );
        assert_eq!(
            direct_token_texts(&ast, compact),
            vec![";"],
            "the compact statement keeps its enclosing semicolon directly"
        );
        let compact_proposition = compact_children
            .iter()
            .find(|child| matches!(child.kind, SurfaceNodeKind::Proposition))
            .expect("compact statement should own a proposition");
        assert_eq!(
            direct_token_texts(&ast, compact_proposition),
            vec!["A", ":"],
            "the compact statement label belongs to its proposition child"
        );

        let conclusions = ast
            .nodes()
            .iter()
            .filter(|node| matches!(node.kind, SurfaceNodeKind::ConclusionStatement))
            .collect::<Vec<_>>();
        let proof_conclusion = conclusions
            .iter()
            .copied()
            .find(|node| {
                structural_children(&ast, node)
                    .iter()
                    .any(|child| matches!(child.kind, SurfaceNodeKind::ProofBlock))
            })
            .expect("one conclusion statement should own a proof block");
        assert_eq!(
            direct_token_texts(&ast, proof_conclusion),
            vec!["thus", ";"],
            "the conclusion statement keeps its enclosing semicolon directly"
        );
    }

    #[test]
    fn parser_recovers_task22_theorem_and_proof_shapes() {
        let source_id = source_id(173);
        let output = parse(ParseRequest::new(
            source_id,
            Edition::new("2026"),
            token_sequence(
                source_id,
                &[
                    ("theorem", ParserTokenKind::ReservedWord),
                    (":", ParserTokenKind::ReservedSymbol),
                    ("thesis", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("theorem", ParserTokenKind::ReservedWord),
                    ("MissingColon", ParserTokenKind::Identifier),
                    ("thesis", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("theorem", ParserTokenKind::ReservedWord),
                    ("MissingFormula", ParserTokenKind::Identifier),
                    (":", ParserTokenKind::ReservedSymbol),
                    ("by", ParserTokenKind::ReservedWord),
                    ("Ref", ParserTokenKind::Identifier),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("theorem", ParserTokenKind::ReservedWord),
                    ("MissingProofEnd", ParserTokenKind::Identifier),
                    (":", ParserTokenKind::ReservedSymbol),
                    ("thesis", ParserTokenKind::ReservedWord),
                    ("proof", ParserTokenKind::ReservedWord),
                    ("thus", ParserTokenKind::ReservedWord),
                    ("thesis", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("theorem", ParserTokenKind::ReservedWord),
                    ("Next", ParserTokenKind::Identifier),
                    (":", ParserTokenKind::ReservedSymbol),
                    ("thesis", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("thus", ParserTokenKind::ReservedWord),
                    ("thesis", ParserTokenKind::ReservedWord),
                    ("proof", ParserTokenKind::ReservedWord),
                    ("thus", ParserTokenKind::ReservedWord),
                    ("thesis", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("theorem", ParserTokenKind::ReservedWord),
                    ("AfterStatementProof", ParserTokenKind::Identifier),
                    (":", ParserTokenKind::ReservedSymbol),
                    ("thesis", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                ],
            ),
            Vec::new(),
        ));

        assert!(
            output
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == SyntaxDiagnosticCode::MissingEnd)
        );
        assert!(output.diagnostics.iter().any(|diagnostic| {
            diagnostic.code == SyntaxDiagnosticCode::MalformedTermExpression
        }));
        assert!(output.diagnostics.iter().any(|diagnostic| {
            diagnostic.code == SyntaxDiagnosticCode::MalformedFormulaExpression
        }));
        let ast = output
            .ast
            .expect("task-22 malformed theorem/proof shapes should recover an AST");
        assert!(
            count_nodes(&ast, |kind| matches!(
                kind,
                SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::MissingTerm)
            )) >= 1,
            "missing theorem labels should insert MissingTerm"
        );
        assert!(
            count_nodes(&ast, |kind| matches!(
                kind,
                SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::MissingFormula)
            )) >= 1,
            "missing theorem formulas should insert MissingFormula"
        );
        assert!(
            count_nodes(&ast, |kind| matches!(
                kind,
                SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::MissingEnd)
            )) >= 2,
            "the theorem proof and statement proof should both report MissingEnd"
        );
        assert!(ast.nodes().iter().any(|node| {
            matches!(node.kind, SurfaceNodeKind::TheoremItem)
                && direct_token_texts(&ast, node) == vec!["theorem", "MissingColon", ";"]
                && direct_child_has_kind(&ast, node, |kind| {
                    matches!(kind, SurfaceNodeKind::FormulaExpression)
                })
        }));
        assert!(ast.nodes().iter().any(|node| {
            matches!(node.kind, SurfaceNodeKind::TheoremItem)
                && direct_token_texts(&ast, node).contains(&"Next".to_owned())
        }));
        assert!(ast.nodes().iter().any(|node| {
            matches!(node.kind, SurfaceNodeKind::TheoremItem)
                && direct_token_texts(&ast, node).contains(&"AfterStatementProof".to_owned())
        }));
    }

    #[test]
    fn parser_parses_task23_definition_blocks_and_attributes() {
        let source_id = source_id(175);
        let output = parse(ParseRequest::new(
            source_id,
            Edition::new("2026"),
            token_sequence(
                source_id,
                &[
                    ("definition", ParserTokenKind::ReservedWord),
                    ("let", ParserTokenKind::ReservedWord),
                    ("x", ParserTokenKind::Identifier),
                    ("be", ParserTokenKind::ReservedWord),
                    ("set", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("attr", ParserTokenKind::ReservedWord),
                    ("AttrDef", ParserTokenKind::Identifier),
                    (":", ParserTokenKind::ReservedSymbol),
                    ("x", ParserTokenKind::Identifier),
                    ("is", ParserTokenKind::ReservedWord),
                    ("odd", ParserTokenKind::UserSymbol),
                    ("means", ParserTokenKind::ReservedWord),
                    ("thesis", ParserTokenKind::ReservedWord),
                    ("if", ParserTokenKind::ReservedWord),
                    ("thesis", ParserTokenKind::ReservedWord),
                    (",", ParserTokenKind::ReservedSymbol),
                    ("contradiction", ParserTokenKind::ReservedWord),
                    ("if", ParserTokenKind::ReservedWord),
                    ("thesis", ParserTokenKind::ReservedWord),
                    ("otherwise", ParserTokenKind::ReservedWord),
                    ("thesis", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("attr", ParserTokenKind::ReservedWord),
                    ("SingleDef", ParserTokenKind::Identifier),
                    (":", ParserTokenKind::ReservedSymbol),
                    ("x", ParserTokenKind::Identifier),
                    ("is", ParserTokenKind::ReservedWord),
                    ("identifierPattern", ParserTokenKind::Identifier),
                    ("means", ParserTokenKind::ReservedWord),
                    ("thesis", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("attr", ParserTokenKind::ReservedWord),
                    ("PrefixedDef", ParserTokenKind::Identifier),
                    (":", ParserTokenKind::ReservedSymbol),
                    ("x", ParserTokenKind::Identifier),
                    ("is", ParserTokenKind::ReservedWord),
                    ("x", ParserTokenKind::Identifier),
                    ("-", ParserTokenKind::ReservedSymbol),
                    ("dimensional", ParserTokenKind::UserSymbol),
                    ("means", ParserTokenKind::ReservedWord),
                    ("thesis", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("assume", ParserTokenKind::ReservedWord),
                    ("thesis", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("existence", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("uniqueness", ParserTokenKind::ReservedWord),
                    ("by", ParserTokenKind::ReservedWord),
                    ("Ref", ParserTokenKind::Identifier),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("coherence", ParserTokenKind::ReservedWord),
                    ("by", ParserTokenKind::ReservedWord),
                    ("Ref", ParserTokenKind::Identifier),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("compatibility", ParserTokenKind::ReservedWord),
                    ("by", ParserTokenKind::ReservedWord),
                    ("computation", ParserTokenKind::ReservedWord),
                    ("(", ParserTokenKind::ReservedSymbol),
                    ("steps", ParserTokenKind::Identifier),
                    (":", ParserTokenKind::ReservedSymbol),
                    ("1", ParserTokenKind::Numeral),
                    (")", ParserTokenKind::ReservedSymbol),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("reducibility", ParserTokenKind::ReservedWord),
                    ("by", ParserTokenKind::ReservedWord),
                    ("Ref", ParserTokenKind::Identifier),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("consistency", ParserTokenKind::ReservedWord),
                    ("proof", ParserTokenKind::ReservedWord),
                    ("thus", ParserTokenKind::ReservedWord),
                    ("thesis", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("end", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("theorem", ParserTokenKind::ReservedWord),
                    ("Plain", ParserTokenKind::Identifier),
                    (":", ParserTokenKind::ReservedSymbol),
                    ("thesis", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("lemma", ParserTokenKind::ReservedWord),
                    ("InnerLemma", ParserTokenKind::Identifier),
                    (":", ParserTokenKind::ReservedSymbol),
                    ("thesis", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("public", ParserTokenKind::ReservedWord),
                    ("theorem", ParserTokenKind::ReservedWord),
                    ("Inner", ParserTokenKind::Identifier),
                    (":", ParserTokenKind::ReservedSymbol),
                    ("thesis", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("end", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("definition", ParserTokenKind::ReservedWord),
                    ("let", ParserTokenKind::ReservedWord),
                    ("T", ParserTokenKind::Identifier),
                    ("be", ParserTokenKind::ReservedWord),
                    ("type", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("pred", ParserTokenKind::ReservedWord),
                    ("Deferred", ParserTokenKind::Identifier),
                    ("means", ParserTokenKind::ReservedWord),
                    ("thesis", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("end", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                ],
            ),
            Vec::new(),
        ));

        assert!(
            output.diagnostics.is_empty(),
            "task-23 definition blocks should parse without diagnostics: {:?}",
            output.diagnostics
        );
        let ast = output
            .ast
            .expect("task-23 definition blocks should keep an AST");
        assert_eq!(
            count_nodes(&ast, |kind| matches!(
                kind,
                SurfaceNodeKind::DefinitionBlockItem
            )),
            2
        );
        assert_eq!(
            count_nodes(&ast, |kind| matches!(
                kind,
                SurfaceNodeKind::DefinitionParameter
            )),
            1,
            "template-ambiguous `let T be type;` stays a placeholder"
        );
        assert_eq!(
            count_nodes(&ast, |kind| matches!(
                kind,
                SurfaceNodeKind::AttributeDefinition
            )),
            3
        );
        assert_eq!(
            count_nodes(&ast, |kind| matches!(
                kind,
                SurfaceNodeKind::AttributePattern
            )),
            3
        );
        assert_eq!(
            count_nodes(&ast, |kind| matches!(
                kind,
                SurfaceNodeKind::ParameterPrefix
            )),
            1,
            "prefixed attribute patterns preserve the local parameter-prefix split"
        );
        assert_eq!(
            count_nodes(&ast, |kind| matches!(
                kind,
                SurfaceNodeKind::FormulaDefiniens
            )),
            3,
            "case-form and single-formula definiens bodies are both concrete"
        );
        assert_eq!(
            count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::FormulaCase)),
            2
        );
        assert_eq!(
            count_nodes(&ast, |kind| matches!(
                kind,
                SurfaceNodeKind::CorrectnessCondition
            )),
            6
        );
        assert_eq!(
            count_nodes(&ast, |kind| matches!(
                kind,
                SurfaceNodeKind::ComputationJustification
            )),
            1,
            "correctness conditions accept computation justifications"
        );
        assert_eq!(
            count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::ProofBlock)),
            1
        );
        assert_eq!(
            count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::VisibleItem)),
            1
        );
        assert_eq!(
            count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::TheoremItem)),
            2
        );
        assert_eq!(
            count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::LemmaItem)),
            1
        );
        assert_eq!(
            count_nodes(&ast, |kind| matches!(
                kind,
                SurfaceNodeKind::AssumptionStatement
            )),
            1,
            "definition-block content dispatch accepts assumptions"
        );
        assert_eq!(
            count_nodes(&ast, |kind| matches!(
                kind,
                SurfaceNodeKind::PlaceholderItem
            )),
            2,
            "template parameter and predicate definition content remain placeholders"
        );
    }

    #[test]
    fn parser_preserves_task23_template_ambiguous_definition_content_as_placeholders() {
        let source_id = source_id(176);
        let output = parse(ParseRequest::new(
            source_id,
            Edition::new("2026"),
            token_sequence(
                source_id,
                &[
                    ("definition", ParserTokenKind::ReservedWord),
                    ("let", ParserTokenKind::ReservedWord),
                    ("T", ParserTokenKind::Identifier),
                    ("be", ParserTokenKind::ReservedWord),
                    ("type", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("pred", ParserTokenKind::ReservedWord),
                    ("P", ParserTokenKind::Identifier),
                    ("means", ParserTokenKind::ReservedWord),
                    ("thesis", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("func", ParserTokenKind::ReservedWord),
                    ("F", ParserTokenKind::Identifier),
                    ("means", ParserTokenKind::ReservedWord),
                    ("thesis", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("end", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                ],
            ),
            Vec::new(),
        ));

        assert!(
            output.diagnostics.is_empty(),
            "task-23 deferred template content should preserve without diagnostics: {:?}",
            output.diagnostics
        );
        let ast = output
            .ast
            .expect("template-ambiguous definition content should keep an AST");
        assert_eq!(
            count_nodes(&ast, |kind| matches!(
                kind,
                SurfaceNodeKind::DefinitionBlockItem
            )),
            1
        );
        assert_eq!(
            count_nodes(&ast, |kind| matches!(
                kind,
                SurfaceNodeKind::DefinitionParameter
            )),
            0
        );
        assert_eq!(
            count_nodes(&ast, |kind| matches!(
                kind,
                SurfaceNodeKind::PlaceholderItem
            )),
            3
        );
    }

    #[test]
    fn parser_recovers_task23_definition_block_gaps() {
        let source_id = source_id(177);
        let output = parse(ParseRequest::new(
            source_id,
            Edition::new("2026"),
            token_sequence(
                source_id,
                &[
                    ("definition", ParserTokenKind::ReservedWord),
                    ("attr", ParserTokenKind::ReservedWord),
                    (":", ParserTokenKind::ReservedSymbol),
                    ("is", ParserTokenKind::ReservedWord),
                    ("means", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("existence", ParserTokenKind::ReservedWord),
                    ("junk", ParserTokenKind::Identifier),
                    (";", ParserTokenKind::ReservedSymbol),
                ],
            ),
            Vec::new(),
        ));

        assert!(
            output
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == SyntaxDiagnosticCode::MalformedTermExpression)
        );
        assert!(output.diagnostics.iter().any(|diagnostic| {
            diagnostic.code == SyntaxDiagnosticCode::MalformedFormulaExpression
        }));
        assert!(
            output
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == SyntaxDiagnosticCode::MalformedJustification)
        );
        assert!(
            output
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == SyntaxDiagnosticCode::MissingEnd)
        );
        assert!(
            output
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == SyntaxDiagnosticCode::MissingSemicolon)
        );
        let ast = output
            .ast
            .expect("task-23 malformed definition content should recover an AST");
        assert_eq!(
            count_nodes(&ast, |kind| matches!(
                kind,
                SurfaceNodeKind::DefinitionBlockItem
            )),
            1
        );
        assert_eq!(
            count_nodes(&ast, |kind| matches!(
                kind,
                SurfaceNodeKind::AttributeDefinition
            )),
            1
        );
        assert_eq!(
            count_nodes(&ast, |kind| matches!(
                kind,
                SurfaceNodeKind::CorrectnessCondition
            )),
            1
        );
        assert!(
            count_nodes(&ast, |kind| matches!(
                kind,
                SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::MissingTerm)
            )) >= 3
        );
        assert!(
            count_nodes(&ast, |kind| matches!(
                kind,
                SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::MissingFormula)
            )) >= 1
        );
        assert!(
            count_nodes(&ast, |kind| matches!(
                kind,
                SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::SkippedToken)
            )) >= 1
        );
        assert!(
            count_nodes(&ast, |kind| matches!(
                kind,
                SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::MissingEnd)
            )) >= 1
        );
    }

    #[test]
    fn parser_recovers_task23_definition_content_at_end_boundary() {
        let source_id = source_id(178);
        let output = parse(ParseRequest::new(
            source_id,
            Edition::new("2026"),
            token_sequence(
                source_id,
                &[
                    ("definition", ParserTokenKind::ReservedWord),
                    ("attr", ParserTokenKind::ReservedWord),
                    ("EndSync", ParserTokenKind::Identifier),
                    (":", ParserTokenKind::ReservedSymbol),
                    ("x", ParserTokenKind::Identifier),
                    ("is", ParserTokenKind::ReservedWord),
                    ("odd", ParserTokenKind::UserSymbol),
                    ("means", ParserTokenKind::ReservedWord),
                    ("thesis", ParserTokenKind::ReservedWord),
                    ("end", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                ],
            ),
            Vec::new(),
        ));

        assert!(
            output
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == SyntaxDiagnosticCode::MissingSemicolon),
            "definition content ending at `end` without `;` should recover at the block boundary"
        );
        assert!(
            !output
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == SyntaxDiagnosticCode::MissingEnd),
            "the existing block `end` should close the definition after content recovery"
        );
        let ast = output
            .ast
            .expect("definition content synchronized at `end` should keep an AST");
        assert_eq!(
            count_nodes(&ast, |kind| matches!(
                kind,
                SurfaceNodeKind::DefinitionBlockItem
            )),
            1
        );
        assert_eq!(
            count_nodes(&ast, |kind| matches!(
                kind,
                SurfaceNodeKind::AttributeDefinition
            )),
            1
        );
    }

    #[test]
    fn parser_recovers_task23_definition_content_at_next_content_start() {
        let source_id = source_id(179);
        let output = parse(ParseRequest::new(
            source_id,
            Edition::new("2026"),
            token_sequence(
                source_id,
                &[
                    ("definition", ParserTokenKind::ReservedWord),
                    ("attr", ParserTokenKind::ReservedWord),
                    ("NextSync", ParserTokenKind::Identifier),
                    (":", ParserTokenKind::ReservedSymbol),
                    ("x", ParserTokenKind::Identifier),
                    ("is", ParserTokenKind::ReservedWord),
                    ("odd", ParserTokenKind::UserSymbol),
                    ("means", ParserTokenKind::ReservedWord),
                    ("thesis", ParserTokenKind::ReservedWord),
                    ("existence", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("end", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                ],
            ),
            Vec::new(),
        ));

        assert!(
            output
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == SyntaxDiagnosticCode::MissingSemicolon),
            "definition content should recover when the next content starts without `;`"
        );
        let ast = output
            .ast
            .expect("definition content synchronized at next content should keep an AST");
        assert_eq!(
            count_nodes(&ast, |kind| matches!(
                kind,
                SurfaceNodeKind::AttributeDefinition
            )),
            1
        );
        assert_eq!(
            count_nodes(&ast, |kind| matches!(
                kind,
                SurfaceNodeKind::CorrectnessCondition
            )),
            1
        );
    }

    #[test]
    fn parser_parses_task24_predicate_definitions() {
        let source_id = source_id(180);
        let output = parse(ParseRequest::new(
            source_id,
            Edition::new("2026"),
            token_sequence(
                source_id,
                &[
                    ("definition", ParserTokenKind::ReservedWord),
                    ("let", ParserTokenKind::ReservedWord),
                    ("x", ParserTokenKind::Identifier),
                    (",", ParserTokenKind::ReservedSymbol),
                    ("y", ParserTokenKind::Identifier),
                    ("be", ParserTokenKind::ReservedWord),
                    ("set", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("pred", ParserTokenKind::ReservedWord),
                    ("RelDef", ParserTokenKind::Identifier),
                    (":", ParserTokenKind::ReservedSymbol),
                    ("x", ParserTokenKind::Identifier),
                    ("R", ParserTokenKind::Identifier),
                    ("y", ParserTokenKind::Identifier),
                    ("means", ParserTokenKind::ReservedWord),
                    ("thesis", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("pred", ParserTokenKind::ReservedWord),
                    ("PrefixDef", ParserTokenKind::Identifier),
                    (":", ParserTokenKind::ReservedSymbol),
                    ("R", ParserTokenKind::Identifier),
                    ("x", ParserTokenKind::Identifier),
                    ("means", ParserTokenKind::ReservedWord),
                    ("thesis", ParserTokenKind::ReservedWord),
                    ("if", ParserTokenKind::ReservedWord),
                    ("thesis", ParserTokenKind::ReservedWord),
                    (",", ParserTokenKind::ReservedSymbol),
                    ("contradiction", ParserTokenKind::ReservedWord),
                    ("if", ParserTokenKind::ReservedWord),
                    ("thesis", ParserTokenKind::ReservedWord),
                    ("otherwise", ParserTokenKind::ReservedWord),
                    ("thesis", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("pred", ParserTokenKind::ReservedWord),
                    ("MultiDef", ParserTokenKind::Identifier),
                    (":", ParserTokenKind::ReservedSymbol),
                    ("x", ParserTokenKind::Identifier),
                    (",", ParserTokenKind::ReservedSymbol),
                    ("y", ParserTokenKind::Identifier),
                    ("partition", ParserTokenKind::Identifier),
                    ("z", ParserTokenKind::Identifier),
                    ("means", ParserTokenKind::ReservedWord),
                    ("thesis", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("pred", ParserTokenKind::ReservedWord),
                    ("LessOrEqualDef", ParserTokenKind::Identifier),
                    (":", ParserTokenKind::ReservedSymbol),
                    ("x", ParserTokenKind::Identifier),
                    ("<=", ParserTokenKind::LexemeRun),
                    ("y", ParserTokenKind::Identifier),
                    ("means", ParserTokenKind::ReservedWord),
                    ("thesis", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("pred", ParserTokenKind::ReservedWord),
                    ("TemplateLociDef", ParserTokenKind::Identifier),
                    (":", ParserTokenKind::ReservedSymbol),
                    ("x", ParserTokenKind::Identifier),
                    ("Rel", ParserTokenKind::Identifier),
                    ("[", ParserTokenKind::ReservedSymbol),
                    ("T", ParserTokenKind::Identifier),
                    (",", ParserTokenKind::ReservedSymbol),
                    ("U", ParserTokenKind::Identifier),
                    ("]", ParserTokenKind::ReservedSymbol),
                    ("y", ParserTokenKind::Identifier),
                    ("means", ParserTokenKind::ReservedWord),
                    ("thesis", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("public", ParserTokenKind::ReservedWord),
                    ("pred", ParserTokenKind::ReservedWord),
                    ("PublicDef", ParserTokenKind::Identifier),
                    (":", ParserTokenKind::ReservedSymbol),
                    ("public_rel", ParserTokenKind::Identifier),
                    ("x", ParserTokenKind::Identifier),
                    ("means", ParserTokenKind::ReservedWord),
                    ("thesis", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("private", ParserTokenKind::ReservedWord),
                    ("pred", ParserTokenKind::ReservedWord),
                    ("HiddenDef", ParserTokenKind::Identifier),
                    (":", ParserTokenKind::ReservedSymbol),
                    ("hidden", ParserTokenKind::Identifier),
                    ("x", ParserTokenKind::Identifier),
                    ("means", ParserTokenKind::ReservedWord),
                    ("thesis", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("end", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                ],
            ),
            Vec::new(),
        ));

        assert!(
            output.diagnostics.is_empty(),
            "task-24 predicate definitions should parse without diagnostics: {:?}",
            output.diagnostics
        );
        let ast = output
            .ast
            .expect("task-24 predicate definitions should keep an AST");
        assert_eq!(
            count_nodes(&ast, |kind| matches!(
                kind,
                SurfaceNodeKind::PredicateDefinition
            )),
            7
        );
        assert_eq!(
            count_nodes(&ast, |kind| matches!(
                kind,
                SurfaceNodeKind::PredicatePattern
            )),
            7
        );
        assert_eq!(
            count_nodes(&ast, |kind| matches!(
                kind,
                SurfaceNodeKind::FormulaDefiniens
            )),
            7
        );
        assert_eq!(
            count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::FormulaCase)),
            2
        );
        assert_eq!(
            count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::VisibleItem)),
            2,
            "definition-local visibility wraps concrete predicate definitions"
        );
        let pattern_token_texts = ast
            .nodes()
            .iter()
            .filter(|node| matches!(node.kind, SurfaceNodeKind::PredicatePattern))
            .map(|node| direct_token_texts(&ast, node))
            .collect::<Vec<_>>();
        assert!(
            pattern_token_texts
                .iter()
                .any(|texts| texts.iter().map(String::as_str).eq(["x", "<=", "y"])),
            "fresh symbolic predicate definitions should preserve LexemeRun symbols"
        );
        assert!(
            pattern_token_texts.iter().any(|texts| texts
                .iter()
                .map(String::as_str)
                .eq(["x", "Rel", "[", "T", ",", "U", "]", "y"])),
            "template-loci tokens should remain direct raw PredicatePattern children"
        );
        assert_eq!(
            count_nodes(&ast, |kind| matches!(
                kind,
                SurfaceNodeKind::PlaceholderItem
            )),
            0
        );
    }

    #[test]
    fn parser_recovers_task24_predicate_definition_gaps() {
        let source_id = source_id(181);
        let output = parse(ParseRequest::new(
            source_id,
            Edition::new("2026"),
            token_sequence(
                source_id,
                &[
                    ("definition", ParserTokenKind::ReservedWord),
                    ("pred", ParserTokenKind::ReservedWord),
                    (":", ParserTokenKind::ReservedSymbol),
                    ("x", ParserTokenKind::Identifier),
                    ("=", ParserTokenKind::ReservedSymbol),
                    ("y", ParserTokenKind::Identifier),
                    ("means", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("pred", ParserTokenKind::ReservedWord),
                    ("BuiltinIn", ParserTokenKind::Identifier),
                    (":", ParserTokenKind::ReservedSymbol),
                    ("x", ParserTokenKind::Identifier),
                    ("in", ParserTokenKind::ReservedWord),
                    ("y", ParserTokenKind::Identifier),
                    ("means", ParserTokenKind::ReservedWord),
                    ("thesis", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("pred", ParserTokenKind::ReservedWord),
                    ("BuiltinNe", ParserTokenKind::Identifier),
                    (":", ParserTokenKind::ReservedSymbol),
                    ("x", ParserTokenKind::Identifier),
                    ("<>", ParserTokenKind::ReservedSymbol),
                    ("y", ParserTokenKind::Identifier),
                    ("means", ParserTokenKind::ReservedWord),
                    ("thesis", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("pred", ParserTokenKind::ReservedWord),
                    ("EmptyTemplate", ParserTokenKind::Identifier),
                    (":", ParserTokenKind::ReservedSymbol),
                    ("R", ParserTokenKind::Identifier),
                    ("[", ParserTokenKind::ReservedSymbol),
                    ("]", ParserTokenKind::ReservedSymbol),
                    ("means", ParserTokenKind::ReservedWord),
                    ("thesis", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("pred", ParserTokenKind::ReservedWord),
                    ("MissingMeans", ParserTokenKind::Identifier),
                    (":", ParserTokenKind::ReservedSymbol),
                    ("x", ParserTokenKind::Identifier),
                    ("R", ParserTokenKind::Identifier),
                    ("y", ParserTokenKind::Identifier),
                    ("thesis", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("end", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                ],
            ),
            Vec::new(),
        ));

        assert!(
            output
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == SyntaxDiagnosticCode::MalformedTermExpression)
        );
        assert!(output.diagnostics.iter().any(|diagnostic| {
            diagnostic.code == SyntaxDiagnosticCode::MalformedFormulaExpression
        }));
        let ast = output
            .ast
            .expect("task-24 malformed predicate definitions should recover an AST");
        assert_eq!(
            count_nodes(&ast, |kind| matches!(
                kind,
                SurfaceNodeKind::PredicateDefinition
            )),
            5
        );
        assert_eq!(
            count_nodes(&ast, |kind| matches!(
                kind,
                SurfaceNodeKind::PredicatePattern
            )),
            5
        );
        assert!(
            count_nodes(&ast, |kind| matches!(
                kind,
                SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::MissingTerm)
            )) >= 5,
            "label and malformed predicate patterns should insert missing-term recovery"
        );
        assert!(
            count_nodes(&ast, |kind| matches!(
                kind,
                SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::MissingFormula)
            )) >= 2,
            "missing definiens bodies and missing means recovery should insert missing formulas"
        );
    }

    #[test]
    fn parser_parses_task25_functor_definitions() {
        let source_id = source_id(182);
        let output = parse(ParseRequest::new(
            source_id,
            Edition::new("2026"),
            token_sequence(
                source_id,
                &[
                    ("definition", ParserTokenKind::ReservedWord),
                    ("func", ParserTokenKind::ReservedWord),
                    ("CaseDef", ParserTokenKind::Identifier),
                    (":", ParserTokenKind::ReservedSymbol),
                    ("x", ParserTokenKind::Identifier),
                    ("++", ParserTokenKind::UserSymbol),
                    ("y", ParserTokenKind::Identifier),
                    ("->", ParserTokenKind::ReservedSymbol),
                    ("set", ParserTokenKind::ReservedWord),
                    ("equals", ParserTokenKind::ReservedWord),
                    ("x", ParserTokenKind::Identifier),
                    ("if", ParserTokenKind::ReservedWord),
                    ("thesis", ParserTokenKind::ReservedWord),
                    (",", ParserTokenKind::ReservedSymbol),
                    ("y", ParserTokenKind::Identifier),
                    ("if", ParserTokenKind::ReservedWord),
                    ("contradiction", ParserTokenKind::ReservedWord),
                    ("otherwise", ParserTokenKind::ReservedWord),
                    ("x", ParserTokenKind::Identifier),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("func", ParserTokenKind::ReservedWord),
                    ("PrefixDef", ParserTokenKind::Identifier),
                    (":", ParserTokenKind::ReservedSymbol),
                    ("~", ParserTokenKind::UserSymbol),
                    ("x", ParserTokenKind::Identifier),
                    ("->", ParserTokenKind::ReservedSymbol),
                    ("set", ParserTokenKind::ReservedWord),
                    ("equals", ParserTokenKind::ReservedWord),
                    ("x", ParserTokenKind::Identifier),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("func", ParserTokenKind::ReservedWord),
                    ("PostfixDef", ParserTokenKind::Identifier),
                    (":", ParserTokenKind::ReservedSymbol),
                    ("x", ParserTokenKind::Identifier),
                    ("!", ParserTokenKind::UserSymbol),
                    ("->", ParserTokenKind::ReservedSymbol),
                    ("set", ParserTokenKind::ReservedWord),
                    ("equals", ParserTokenKind::ReservedWord),
                    ("x", ParserTokenKind::Identifier),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("func", ParserTokenKind::ReservedWord),
                    ("CallDef", ParserTokenKind::Identifier),
                    (":", ParserTokenKind::ReservedSymbol),
                    ("f", ParserTokenKind::Identifier),
                    ("(", ParserTokenKind::ReservedSymbol),
                    ("x", ParserTokenKind::Identifier),
                    (",", ParserTokenKind::ReservedSymbol),
                    ("y", ParserTokenKind::Identifier),
                    (")", ParserTokenKind::ReservedSymbol),
                    ("->", ParserTokenKind::ReservedSymbol),
                    ("set", ParserTokenKind::ReservedWord),
                    ("equals", ParserTokenKind::ReservedWord),
                    ("x", ParserTokenKind::Identifier),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("func", ParserTokenKind::ReservedWord),
                    ("CircumfixDef", ParserTokenKind::Identifier),
                    (":", ParserTokenKind::ReservedSymbol),
                    ("|.", ParserTokenKind::UserSymbol),
                    ("x", ParserTokenKind::Identifier),
                    (".|", ParserTokenKind::UserSymbol),
                    ("->", ParserTokenKind::ReservedSymbol),
                    ("set", ParserTokenKind::ReservedWord),
                    ("equals", ParserTokenKind::ReservedWord),
                    ("x", ParserTokenKind::Identifier),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("func", ParserTokenKind::ReservedWord),
                    ("LexemeRunDef", ParserTokenKind::Identifier),
                    (":", ParserTokenKind::ReservedSymbol),
                    ("x", ParserTokenKind::Identifier),
                    ("<+>", ParserTokenKind::LexemeRun),
                    ("y", ParserTokenKind::Identifier),
                    ("->", ParserTokenKind::ReservedSymbol),
                    ("set", ParserTokenKind::ReservedWord),
                    ("equals", ParserTokenKind::ReservedWord),
                    ("x", ParserTokenKind::Identifier),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("func", ParserTokenKind::ReservedWord),
                    ("TemplateLociDef", ParserTokenKind::Identifier),
                    (":", ParserTokenKind::ReservedSymbol),
                    ("op", ParserTokenKind::Identifier),
                    ("[", ParserTokenKind::ReservedSymbol),
                    ("T", ParserTokenKind::Identifier),
                    (",", ParserTokenKind::ReservedSymbol),
                    ("U", ParserTokenKind::Identifier),
                    ("]", ParserTokenKind::ReservedSymbol),
                    ("x", ParserTokenKind::Identifier),
                    ("->", ParserTokenKind::ReservedSymbol),
                    ("set", ParserTokenKind::ReservedWord),
                    ("equals", ParserTokenKind::ReservedWord),
                    ("x", ParserTokenKind::Identifier),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("func", ParserTokenKind::ReservedWord),
                    ("MeansCaseDef", ParserTokenKind::Identifier),
                    (":", ParserTokenKind::ReservedSymbol),
                    ("decide", ParserTokenKind::Identifier),
                    ("x", ParserTokenKind::Identifier),
                    ("->", ParserTokenKind::ReservedSymbol),
                    ("set", ParserTokenKind::ReservedWord),
                    ("means", ParserTokenKind::ReservedWord),
                    ("thesis", ParserTokenKind::ReservedWord),
                    ("if", ParserTokenKind::ReservedWord),
                    ("thesis", ParserTokenKind::ReservedWord),
                    ("otherwise", ParserTokenKind::ReservedWord),
                    ("thesis", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("func", ParserTokenKind::ReservedWord),
                    ("MeansDef", ParserTokenKind::Identifier),
                    (":", ParserTokenKind::ReservedSymbol),
                    ("measure", ParserTokenKind::Identifier),
                    ("x", ParserTokenKind::Identifier),
                    ("->", ParserTokenKind::ReservedSymbol),
                    ("set", ParserTokenKind::ReservedWord),
                    ("means", ParserTokenKind::ReservedWord),
                    ("thesis", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("public", ParserTokenKind::ReservedWord),
                    ("func", ParserTokenKind::ReservedWord),
                    ("PublicDef", ParserTokenKind::Identifier),
                    (":", ParserTokenKind::ReservedSymbol),
                    ("public_f", ParserTokenKind::Identifier),
                    ("x", ParserTokenKind::Identifier),
                    ("->", ParserTokenKind::ReservedSymbol),
                    ("set", ParserTokenKind::ReservedWord),
                    ("equals", ParserTokenKind::ReservedWord),
                    ("x", ParserTokenKind::Identifier),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("private", ParserTokenKind::ReservedWord),
                    ("func", ParserTokenKind::ReservedWord),
                    ("HiddenDef", ParserTokenKind::Identifier),
                    (":", ParserTokenKind::ReservedSymbol),
                    ("hidden_f", ParserTokenKind::Identifier),
                    ("x", ParserTokenKind::Identifier),
                    ("->", ParserTokenKind::ReservedSymbol),
                    ("set", ParserTokenKind::ReservedWord),
                    ("equals", ParserTokenKind::ReservedWord),
                    ("x", ParserTokenKind::Identifier),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("end", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                ],
            ),
            Vec::new(),
        ));

        assert!(
            output.diagnostics.is_empty(),
            "task-25 functor definitions should parse without diagnostics: {:?}",
            output.diagnostics
        );
        let ast = output
            .ast
            .expect("task-25 functor definitions should keep an AST");
        assert_eq!(
            count_nodes(&ast, |kind| matches!(
                kind,
                SurfaceNodeKind::FunctorDefinition
            )),
            11
        );
        assert_eq!(
            count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::FunctorPattern)),
            11
        );
        assert_eq!(
            count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::TermDefiniens)),
            9
        );
        assert_eq!(
            count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::TermCase)),
            2
        );
        assert_eq!(
            count_nodes(&ast, |kind| matches!(
                kind,
                SurfaceNodeKind::FormulaDefiniens
            )),
            2
        );
        assert_eq!(
            count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::FormulaCase)),
            1
        );
        assert_eq!(
            count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::VisibleItem)),
            2,
            "definition-local visibility wraps concrete functor definitions"
        );
        let pattern_token_texts = ast
            .nodes()
            .iter()
            .filter(|node| matches!(node.kind, SurfaceNodeKind::FunctorPattern))
            .map(|node| direct_token_texts(&ast, node))
            .collect::<Vec<_>>();
        for expected in [
            &["x", "++", "y"][..],
            &["~", "x"][..],
            &["x", "!"][..],
            &["f", "(", "x", ",", "y", ")"][..],
            &["|.", "x", ".|"][..],
            &["x", "<+>", "y"][..],
            &["op", "[", "T", ",", "U", "]", "x"][..],
        ] {
            assert!(
                pattern_token_texts.iter().any(|texts| texts
                    .iter()
                    .map(String::as_str)
                    .eq(expected.iter().copied())),
                "raw FunctorPattern children should contain {expected:?}"
            );
        }
        assert_eq!(
            count_nodes(&ast, |kind| matches!(
                kind,
                SurfaceNodeKind::PlaceholderItem
            )),
            0
        );
    }

    #[test]
    fn parser_preserves_task25_template_functor_parameters_as_placeholders() {
        let source_id = source_id(183);
        let output = parse(ParseRequest::new(
            source_id,
            Edition::new("2026"),
            token_sequence(
                source_id,
                &[
                    ("definition", ParserTokenKind::ReservedWord),
                    ("let", ParserTokenKind::ReservedWord),
                    ("F", ParserTokenKind::Identifier),
                    ("be", ParserTokenKind::ReservedWord),
                    ("func", ParserTokenKind::ReservedWord),
                    ("(", ParserTokenKind::ReservedSymbol),
                    ("T", ParserTokenKind::Identifier),
                    (")", ParserTokenKind::ReservedSymbol),
                    ("->", ParserTokenKind::ReservedSymbol),
                    ("S", ParserTokenKind::Identifier),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("func", ParserTokenKind::ReservedWord),
                    ("Deferred", ParserTokenKind::Identifier),
                    (":", ParserTokenKind::ReservedSymbol),
                    ("f", ParserTokenKind::Identifier),
                    ("->", ParserTokenKind::ReservedSymbol),
                    ("set", ParserTokenKind::ReservedWord),
                    ("equals", ParserTokenKind::ReservedWord),
                    ("x", ParserTokenKind::Identifier),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("end", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                ],
            ),
            Vec::new(),
        ));

        assert!(
            output.diagnostics.is_empty(),
            "task-25 deferred functor-template content should preserve without diagnostics: {:?}",
            output.diagnostics
        );
        let ast = output
            .ast
            .expect("template-ambiguous functor content should keep an AST");
        assert_eq!(
            count_nodes(&ast, |kind| matches!(
                kind,
                SurfaceNodeKind::FunctorDefinition
            )),
            0
        );
        assert_eq!(
            count_nodes(&ast, |kind| matches!(
                kind,
                SurfaceNodeKind::DefinitionParameter
            )),
            0
        );
        assert_eq!(
            count_nodes(&ast, |kind| matches!(
                kind,
                SurfaceNodeKind::PlaceholderItem
            )),
            2
        );
    }

    #[test]
    fn parser_recovers_task25_functor_definition_gaps() {
        let source_id = source_id(184);
        let output = parse(ParseRequest::new(
            source_id,
            Edition::new("2026"),
            token_sequence(
                source_id,
                &[
                    ("definition", ParserTokenKind::ReservedWord),
                    ("func", ParserTokenKind::ReservedWord),
                    (":", ParserTokenKind::ReservedSymbol),
                    ("->", ParserTokenKind::ReservedSymbol),
                    ("equals", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("func", ParserTokenKind::ReservedWord),
                    ("MalformedPattern", ParserTokenKind::Identifier),
                    (":", ParserTokenKind::ReservedSymbol),
                    ("x", ParserTokenKind::Identifier),
                    ("=", ParserTokenKind::ReservedSymbol),
                    ("y", ParserTokenKind::Identifier),
                    ("->", ParserTokenKind::ReservedSymbol),
                    ("set", ParserTokenKind::ReservedWord),
                    ("equals", ParserTokenKind::ReservedWord),
                    ("x", ParserTokenKind::Identifier),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("func", ParserTokenKind::ReservedWord),
                    ("MissingColon", ParserTokenKind::Identifier),
                    ("f", ParserTokenKind::Identifier),
                    ("x", ParserTokenKind::Identifier),
                    ("->", ParserTokenKind::ReservedSymbol),
                    ("set", ParserTokenKind::ReservedWord),
                    ("equals", ParserTokenKind::ReservedWord),
                    ("x", ParserTokenKind::Identifier),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("func", ParserTokenKind::ReservedWord),
                    ("MissingArrow", ParserTokenKind::Identifier),
                    (":", ParserTokenKind::ReservedSymbol),
                    ("f", ParserTokenKind::Identifier),
                    ("x", ParserTokenKind::Identifier),
                    ("set", ParserTokenKind::ReservedWord),
                    ("equals", ParserTokenKind::ReservedWord),
                    ("x", ParserTokenKind::Identifier),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("func", ParserTokenKind::ReservedWord),
                    ("MissingReturn", ParserTokenKind::Identifier),
                    (":", ParserTokenKind::ReservedSymbol),
                    ("f", ParserTokenKind::Identifier),
                    ("x", ParserTokenKind::Identifier),
                    ("->", ParserTokenKind::ReservedSymbol),
                    ("equals", ParserTokenKind::ReservedWord),
                    ("x", ParserTokenKind::Identifier),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("func", ParserTokenKind::ReservedWord),
                    ("MissingBodyFormula", ParserTokenKind::Identifier),
                    (":", ParserTokenKind::ReservedSymbol),
                    ("f", ParserTokenKind::Identifier),
                    ("x", ParserTokenKind::Identifier),
                    ("->", ParserTokenKind::ReservedSymbol),
                    ("set", ParserTokenKind::ReservedWord),
                    ("thesis", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("func", ParserTokenKind::ReservedWord),
                    ("MissingBodyTerm", ParserTokenKind::Identifier),
                    (":", ParserTokenKind::ReservedSymbol),
                    ("f", ParserTokenKind::Identifier),
                    ("x", ParserTokenKind::Identifier),
                    ("->", ParserTokenKind::ReservedSymbol),
                    ("set", ParserTokenKind::ReservedWord),
                    ("x", ParserTokenKind::Identifier),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("func", ParserTokenKind::ReservedWord),
                    ("MissingBodyEmpty", ParserTokenKind::Identifier),
                    (":", ParserTokenKind::ReservedSymbol),
                    ("f", ParserTokenKind::Identifier),
                    ("x", ParserTokenKind::Identifier),
                    ("->", ParserTokenKind::ReservedSymbol),
                    ("set", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("func", ParserTokenKind::ReservedWord),
                    ("MissingMeansFormula", ParserTokenKind::Identifier),
                    (":", ParserTokenKind::ReservedSymbol),
                    ("f", ParserTokenKind::Identifier),
                    ("x", ParserTokenKind::Identifier),
                    ("->", ParserTokenKind::ReservedSymbol),
                    ("set", ParserTokenKind::ReservedWord),
                    ("means", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("func", ParserTokenKind::ReservedWord),
                    ("MissingMeansOtherwiseFormula", ParserTokenKind::Identifier),
                    (":", ParserTokenKind::ReservedSymbol),
                    ("f", ParserTokenKind::Identifier),
                    ("x", ParserTokenKind::Identifier),
                    ("->", ParserTokenKind::ReservedSymbol),
                    ("set", ParserTokenKind::ReservedWord),
                    ("means", ParserTokenKind::ReservedWord),
                    ("thesis", ParserTokenKind::ReservedWord),
                    ("if", ParserTokenKind::ReservedWord),
                    ("thesis", ParserTokenKind::ReservedWord),
                    ("otherwise", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("func", ParserTokenKind::ReservedWord),
                    ("MissingOtherwiseTerm", ParserTokenKind::Identifier),
                    (":", ParserTokenKind::ReservedSymbol),
                    ("f", ParserTokenKind::Identifier),
                    ("x", ParserTokenKind::Identifier),
                    ("->", ParserTokenKind::ReservedSymbol),
                    ("set", ParserTokenKind::ReservedWord),
                    ("equals", ParserTokenKind::ReservedWord),
                    ("x", ParserTokenKind::Identifier),
                    ("if", ParserTokenKind::ReservedWord),
                    ("thesis", ParserTokenKind::ReservedWord),
                    ("otherwise", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("func", ParserTokenKind::ReservedWord),
                    ("MissingTermCaseCondition", ParserTokenKind::Identifier),
                    (":", ParserTokenKind::ReservedSymbol),
                    ("f", ParserTokenKind::Identifier),
                    ("x", ParserTokenKind::Identifier),
                    ("->", ParserTokenKind::ReservedSymbol),
                    ("set", ParserTokenKind::ReservedWord),
                    ("equals", ParserTokenKind::ReservedWord),
                    ("x", ParserTokenKind::Identifier),
                    ("if", ParserTokenKind::ReservedWord),
                    ("otherwise", ParserTokenKind::ReservedWord),
                    ("y", ParserTokenKind::Identifier),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("func", ParserTokenKind::ReservedWord),
                    ("MissingTermCase", ParserTokenKind::Identifier),
                    (":", ParserTokenKind::ReservedSymbol),
                    ("f", ParserTokenKind::Identifier),
                    ("x", ParserTokenKind::Identifier),
                    ("->", ParserTokenKind::ReservedSymbol),
                    ("set", ParserTokenKind::ReservedWord),
                    ("equals", ParserTokenKind::ReservedWord),
                    ("x", ParserTokenKind::Identifier),
                    ("if", ParserTokenKind::ReservedWord),
                    ("thesis", ParserTokenKind::ReservedWord),
                    (",", ParserTokenKind::ReservedSymbol),
                    ("if", ParserTokenKind::ReservedWord),
                    ("thesis", ParserTokenKind::ReservedWord),
                    ("otherwise", ParserTokenKind::ReservedWord),
                    ("x", ParserTokenKind::Identifier),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("end", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                ],
            ),
            Vec::new(),
        ));

        assert!(
            output
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == SyntaxDiagnosticCode::MalformedTermExpression)
        );
        assert!(output.diagnostics.iter().any(|diagnostic| {
            diagnostic.code == SyntaxDiagnosticCode::MalformedFormulaExpression
        }));
        assert!(
            output
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == SyntaxDiagnosticCode::MalformedTypeExpression)
        );
        let ast = output
            .ast
            .expect("task-25 malformed functor definitions should recover an AST");
        assert_eq!(
            count_nodes(&ast, |kind| matches!(
                kind,
                SurfaceNodeKind::FunctorDefinition
            )),
            13
        );
        assert_eq!(
            count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::FunctorPattern)),
            13
        );
        assert!(count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::TermDefiniens)) >= 4);
        assert!(
            count_nodes(&ast, |kind| matches!(
                kind,
                SurfaceNodeKind::FormulaDefiniens
            )) >= 1,
            "missing body-keyword recovery should keep formula-start bodies as FormulaDefiniens"
        );
        assert!(
            count_nodes(&ast, |kind| matches!(
                kind,
                SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::MissingTerm)
            )) >= 5,
            "labels, malformed patterns, missing term bodies, and term cases insert MissingTerm"
        );
        assert!(
            count_nodes(&ast, |kind| matches!(
                kind,
                SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::MissingTypeExpression)
            )) >= 2,
            "missing arrows/return types insert MissingTypeExpression"
        );
        assert!(
            count_nodes(&ast, |kind| matches!(
                kind,
                SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::MissingFormula)
            )) >= 1,
            "missing formula-start body keywords insert MissingFormula when needed"
        );
    }

    #[test]
    fn parser_parses_task26_mode_definitions() {
        let source_id = source_id(185);
        let output = parse(ParseRequest::new(
            source_id,
            Edition::new("2026"),
            token_sequence(
                source_id,
                &[
                    ("definition", ParserTokenKind::ReservedWord),
                    ("mode", ParserTokenKind::ReservedWord),
                    ("SimpleDef", ParserTokenKind::Identifier),
                    (":", ParserTokenKind::ReservedSymbol),
                    ("Simple", ParserTokenKind::Identifier),
                    ("is", ParserTokenKind::ReservedWord),
                    ("set", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("mode", ParserTokenKind::ReservedWord),
                    ("AttrDef", ParserTokenKind::Identifier),
                    (":", ParserTokenKind::ReservedSymbol),
                    ("NonEmpty", ParserTokenKind::Identifier),
                    ("is", ParserTokenKind::ReservedWord),
                    ("non", ParserTokenKind::ReservedWord),
                    ("empty", ParserTokenKind::UserSymbol),
                    ("set", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("mode", ParserTokenKind::ReservedWord),
                    ("OfDef", ParserTokenKind::Identifier),
                    (":", ParserTokenKind::ReservedSymbol),
                    ("Subset", ParserTokenKind::Identifier),
                    ("of", ParserTokenKind::ReservedWord),
                    ("X", ParserTokenKind::Identifier),
                    (",", ParserTokenKind::ReservedSymbol),
                    ("Y", ParserTokenKind::Identifier),
                    ("is", ParserTokenKind::ReservedWord),
                    ("set", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("mode", ParserTokenKind::ReservedWord),
                    ("OverDef", ParserTokenKind::Identifier),
                    (":", ParserTokenKind::ReservedSymbol),
                    ("Matrix", ParserTokenKind::Identifier),
                    ("over", ParserTokenKind::ReservedWord),
                    ("R", ParserTokenKind::Identifier),
                    ("is", ParserTokenKind::ReservedWord),
                    ("set", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("mode", ParserTokenKind::ReservedWord),
                    ("BracketDef", ParserTokenKind::Identifier),
                    (":", ParserTokenKind::ReservedSymbol),
                    ("Family", ParserTokenKind::Identifier),
                    ("[", ParserTokenKind::ReservedSymbol),
                    ("A", ParserTokenKind::Identifier),
                    (",", ParserTokenKind::ReservedSymbol),
                    ("B", ParserTokenKind::Identifier),
                    ("]", ParserTokenKind::ReservedSymbol),
                    ("is", ParserTokenKind::ReservedWord),
                    ("set", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("mode", ParserTokenKind::ReservedWord),
                    ("SymbolicDef", ParserTokenKind::Identifier),
                    (":", ParserTokenKind::ReservedSymbol),
                    ("T", ParserTokenKind::UserSymbol),
                    ("is", ParserTokenKind::ReservedWord),
                    ("set", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("mode", ParserTokenKind::ReservedWord),
                    ("SethoodCitationDef", ParserTokenKind::Identifier),
                    (":", ParserTokenKind::ReservedSymbol),
                    ("Finite", ParserTokenKind::Identifier),
                    ("is", ParserTokenKind::ReservedWord),
                    ("set", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("sethood", ParserTokenKind::ReservedWord),
                    ("by", ParserTokenKind::ReservedWord),
                    ("Ref", ParserTokenKind::Identifier),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("mode", ParserTokenKind::ReservedWord),
                    ("SethoodComputationDef", ParserTokenKind::Identifier),
                    (":", ParserTokenKind::ReservedSymbol),
                    ("Computable", ParserTokenKind::Identifier),
                    ("is", ParserTokenKind::ReservedWord),
                    ("set", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("sethood", ParserTokenKind::ReservedWord),
                    ("by", ParserTokenKind::ReservedWord),
                    ("computation", ParserTokenKind::ReservedWord),
                    ("(", ParserTokenKind::ReservedSymbol),
                    ("steps", ParserTokenKind::Identifier),
                    (":", ParserTokenKind::ReservedSymbol),
                    ("1", ParserTokenKind::Numeral),
                    (")", ParserTokenKind::ReservedSymbol),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("mode", ParserTokenKind::ReservedWord),
                    ("SethoodProofDef", ParserTokenKind::Identifier),
                    (":", ParserTokenKind::ReservedSymbol),
                    ("Ordinal", ParserTokenKind::Identifier),
                    ("is", ParserTokenKind::ReservedWord),
                    ("set", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("sethood", ParserTokenKind::ReservedWord),
                    ("proof", ParserTokenKind::ReservedWord),
                    ("thus", ParserTokenKind::ReservedWord),
                    ("thesis", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("end", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("public", ParserTokenKind::ReservedWord),
                    ("mode", ParserTokenKind::ReservedWord),
                    ("PublicDef", ParserTokenKind::Identifier),
                    (":", ParserTokenKind::ReservedSymbol),
                    ("PublicMode", ParserTokenKind::Identifier),
                    ("is", ParserTokenKind::ReservedWord),
                    ("set", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("private", ParserTokenKind::ReservedWord),
                    ("mode", ParserTokenKind::ReservedWord),
                    ("HiddenDef", ParserTokenKind::Identifier),
                    (":", ParserTokenKind::ReservedSymbol),
                    ("HiddenMode", ParserTokenKind::Identifier),
                    ("is", ParserTokenKind::ReservedWord),
                    ("set", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("end", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                ],
            ),
            Vec::new(),
        ));

        assert!(
            output.diagnostics.is_empty(),
            "task-26 mode definitions should parse without diagnostics: {:?}",
            output.diagnostics
        );
        let ast = output
            .ast
            .expect("task-26 mode definitions should keep an AST");
        assert_eq!(
            count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::ModeDefinition)),
            11
        );
        assert_eq!(
            count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::ModePattern)),
            11
        );
        assert_eq!(
            count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::ModeProperty)),
            3
        );
        assert_eq!(
            count_nodes(&ast, |kind| matches!(
                kind,
                SurfaceNodeKind::JustificationClause
            )),
            2
        );
        assert_eq!(
            count_nodes(&ast, |kind| matches!(
                kind,
                SurfaceNodeKind::ComputationJustification
            )),
            1
        );
        assert_eq!(
            count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::ProofBlock)),
            1
        );
        assert_eq!(
            count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::VisibleItem)),
            2,
            "definition-local visibility wraps concrete mode definitions"
        );
        assert!(
            ast.nodes().iter().any(|node| {
                matches!(node.kind, SurfaceNodeKind::ModeDefinition)
                    && subtree_token_texts(&ast, node)
                        .iter()
                        .map(String::as_str)
                        .eq([
                            "mode", "AttrDef", ":", "NonEmpty", "is", "non", "empty", "set", ";",
                        ])
                    && structural_children(&ast, node).iter().any(|child| {
                        matches!(child.kind, SurfaceNodeKind::TypeExpression)
                            && structural_children(&ast, child).iter().any(|grandchild| {
                                matches!(grandchild.kind, SurfaceNodeKind::AttributeChain)
                            })
                    })
            }),
            "mode bodies with attributes should be represented as TypeExpression nodes owning AttributeChain children"
        );
        let pattern_token_texts = ast
            .nodes()
            .iter()
            .filter(|node| matches!(node.kind, SurfaceNodeKind::ModePattern))
            .map(|node| direct_token_texts(&ast, node))
            .collect::<Vec<_>>();
        for expected in [
            &["Subset", "of", "X", ",", "Y"][..],
            &["Matrix", "over", "R"][..],
            &["Family", "[", "A", ",", "B", "]"][..],
            &["T"][..],
        ] {
            assert!(
                pattern_token_texts.iter().any(|texts| texts
                    .iter()
                    .map(String::as_str)
                    .eq(expected.iter().copied())),
                "raw ModePattern children should contain {expected:?}"
            );
        }
        assert_eq!(
            count_nodes(&ast, |kind| matches!(
                kind,
                SurfaceNodeKind::PlaceholderItem
            )),
            0
        );
    }

    #[test]
    fn parser_recovers_task26_mode_definition_gaps() {
        let source_id = source_id(186);
        let output = parse(ParseRequest::new(
            source_id,
            Edition::new("2026"),
            token_sequence(
                source_id,
                &[
                    ("definition", ParserTokenKind::ReservedWord),
                    ("mode", ParserTokenKind::ReservedWord),
                    (":", ParserTokenKind::ReservedSymbol),
                    ("is", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("mode", ParserTokenKind::ReservedWord),
                    ("MissingColon", ParserTokenKind::Identifier),
                    ("Simple", ParserTokenKind::Identifier),
                    ("is", ParserTokenKind::ReservedWord),
                    ("set", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("mode", ParserTokenKind::ReservedWord),
                    ("EmptyParams", ParserTokenKind::Identifier),
                    (":", ParserTokenKind::ReservedSymbol),
                    ("Bad", ParserTokenKind::Identifier),
                    ("[", ParserTokenKind::ReservedSymbol),
                    ("]", ParserTokenKind::ReservedSymbol),
                    ("is", ParserTokenKind::ReservedWord),
                    ("set", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("mode", ParserTokenKind::ReservedWord),
                    ("DanglingParam", ParserTokenKind::Identifier),
                    (":", ParserTokenKind::ReservedSymbol),
                    ("Bad", ParserTokenKind::Identifier),
                    ("of", ParserTokenKind::ReservedWord),
                    ("X", ParserTokenKind::Identifier),
                    (",", ParserTokenKind::ReservedSymbol),
                    ("is", ParserTokenKind::ReservedWord),
                    ("set", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("mode", ParserTokenKind::ReservedWord),
                    ("MultipleParamGroups", ParserTokenKind::Identifier),
                    (":", ParserTokenKind::ReservedSymbol),
                    ("Bad", ParserTokenKind::Identifier),
                    ("of", ParserTokenKind::ReservedWord),
                    ("X", ParserTokenKind::Identifier),
                    ("over", ParserTokenKind::ReservedWord),
                    ("Y", ParserTokenKind::Identifier),
                    ("is", ParserTokenKind::ReservedWord),
                    ("set", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("mode", ParserTokenKind::ReservedWord),
                    ("LexemeRunName", ParserTokenKind::Identifier),
                    (":", ParserTokenKind::ReservedSymbol),
                    ("<+>", ParserTokenKind::LexemeRun),
                    ("is", ParserTokenKind::ReservedWord),
                    ("set", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("mode", ParserTokenKind::ReservedWord),
                    ("MissingIs", ParserTokenKind::Identifier),
                    (":", ParserTokenKind::ReservedSymbol),
                    ("Bad", ParserTokenKind::Identifier),
                    ("set", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("mode", ParserTokenKind::ReservedWord),
                    ("MissingBody", ParserTokenKind::Identifier),
                    (":", ParserTokenKind::ReservedSymbol),
                    ("Bad", ParserTokenKind::Identifier),
                    ("is", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("mode", ParserTokenKind::ReservedWord),
                    ("MissingSemicolon", ParserTokenKind::Identifier),
                    (":", ParserTokenKind::ReservedSymbol),
                    ("Bad", ParserTokenKind::Identifier),
                    ("is", ParserTokenKind::ReservedWord),
                    ("set", ParserTokenKind::ReservedWord),
                    ("sethood", ParserTokenKind::ReservedWord),
                    ("by", ParserTokenKind::ReservedWord),
                    ("Ref", ParserTokenKind::Identifier),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("mode", ParserTokenKind::ReservedWord),
                    ("MissingSethoodJustification", ParserTokenKind::Identifier),
                    (":", ParserTokenKind::ReservedSymbol),
                    ("Bad", ParserTokenKind::Identifier),
                    ("is", ParserTokenKind::ReservedWord),
                    ("set", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("sethood", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("mode", ParserTokenKind::ReservedWord),
                    ("LegacyMeans", ParserTokenKind::Identifier),
                    (":", ParserTokenKind::ReservedSymbol),
                    ("Bad", ParserTokenKind::Identifier),
                    ("means", ParserTokenKind::ReservedWord),
                    ("thesis", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("end", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                ],
            ),
            Vec::new(),
        ));

        assert!(
            output
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == SyntaxDiagnosticCode::MalformedTermExpression)
        );
        assert!(output.diagnostics.iter().any(|diagnostic| {
            diagnostic.code == SyntaxDiagnosticCode::MalformedFormulaExpression
        }));
        assert!(
            output
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == SyntaxDiagnosticCode::MalformedTypeExpression)
        );
        assert!(
            output
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == SyntaxDiagnosticCode::MalformedJustification)
        );
        assert!(
            output
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == SyntaxDiagnosticCode::MissingSemicolon)
        );
        let ast = output
            .ast
            .expect("task-26 malformed mode definitions should recover an AST");
        assert_eq!(
            count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::ModeDefinition)),
            11
        );
        assert_eq!(
            count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::ModePattern)),
            11
        );
        assert_eq!(
            count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::ModeProperty)),
            2
        );
        assert!(
            count_nodes(&ast, |kind| matches!(
                kind,
                SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::MissingTerm)
            )) >= 5,
            "labels and malformed mode patterns should insert MissingTerm"
        );
        assert!(
            count_nodes(&ast, |kind| matches!(
                kind,
                SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::MissingTypeExpression)
            )) >= 3,
            "missing mode bodies and legacy means recovery should insert MissingTypeExpression"
        );
    }

    #[test]
    fn parser_parses_task27_redefinitions_and_notation_aliases() {
        let source_id = source_id(187);
        let output = parse(ParseRequest::new(
            source_id,
            Edition::new("2026"),
            token_sequence(
                source_id,
                &[
                    ("synonym", ParserTokenKind::ReservedWord),
                    ("FinSeq", ParserTokenKind::Identifier),
                    ("of", ParserTokenKind::ReservedWord),
                    ("G", ParserTokenKind::Identifier),
                    ("for", ParserTokenKind::ReservedWord),
                    ("FinSequence", ParserTokenKind::Identifier),
                    ("of", ParserTokenKind::ReservedWord),
                    ("G", ParserTokenKind::Identifier),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("definition", ParserTokenKind::ReservedWord),
                    ("redefine", ParserTokenKind::ReservedWord),
                    ("attr", ParserTokenKind::ReservedWord),
                    ("AttrR", ParserTokenKind::Identifier),
                    (":", ParserTokenKind::ReservedSymbol),
                    ("x", ParserTokenKind::Identifier),
                    ("is", ParserTokenKind::ReservedWord),
                    ("empty", ParserTokenKind::UserSymbol),
                    ("means", ParserTokenKind::ReservedWord),
                    ("thesis", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("coherence", ParserTokenKind::ReservedWord),
                    ("by", ParserTokenKind::ReservedWord),
                    ("Ref", ParserTokenKind::Identifier),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("redefine", ParserTokenKind::ReservedWord),
                    ("pred", ParserTokenKind::ReservedWord),
                    ("x", ParserTokenKind::Identifier),
                    ("R", ParserTokenKind::UserSymbol),
                    ("y", ParserTokenKind::Identifier),
                    ("means", ParserTokenKind::ReservedWord),
                    ("thesis", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("coherence", ParserTokenKind::ReservedWord),
                    ("with", ParserTokenKind::ReservedWord),
                    ("C", ParserTokenKind::Identifier),
                    ("by", ParserTokenKind::ReservedWord),
                    ("Ref", ParserTokenKind::Identifier),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("redefine", ParserTokenKind::ReservedWord),
                    ("func", ParserTokenKind::ReservedWord),
                    ("MeansR", ParserTokenKind::Identifier),
                    (":", ParserTokenKind::ReservedSymbol),
                    ("g", ParserTokenKind::Identifier),
                    ("x", ParserTokenKind::Identifier),
                    ("->", ParserTokenKind::ReservedSymbol),
                    ("set", ParserTokenKind::ReservedWord),
                    ("means", ParserTokenKind::ReservedWord),
                    ("thesis", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("coherence", ParserTokenKind::ReservedWord),
                    ("by", ParserTokenKind::ReservedWord),
                    ("Ref", ParserTokenKind::Identifier),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("public", ParserTokenKind::ReservedWord),
                    ("redefine", ParserTokenKind::ReservedWord),
                    ("func", ParserTokenKind::ReservedWord),
                    ("FuncR", ParserTokenKind::Identifier),
                    (":", ParserTokenKind::ReservedSymbol),
                    ("f", ParserTokenKind::Identifier),
                    ("x", ParserTokenKind::Identifier),
                    ("->", ParserTokenKind::ReservedSymbol),
                    ("set", ParserTokenKind::ReservedWord),
                    ("equals", ParserTokenKind::ReservedWord),
                    ("x", ParserTokenKind::Identifier),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("coherence", ParserTokenKind::ReservedWord),
                    ("proof", ParserTokenKind::ReservedWord),
                    ("thus", ParserTokenKind::ReservedWord),
                    ("thesis", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("end", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("synonym", ParserTokenKind::ReservedWord),
                    ("finite", ParserTokenKind::Identifier),
                    ("for", ParserTokenKind::ReservedWord),
                    ("empty", ParserTokenKind::UserSymbol),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("private", ParserTokenKind::ReservedWord),
                    ("antonym", ParserTokenKind::ReservedWord),
                    ("x", ParserTokenKind::Identifier),
                    (">=", ParserTokenKind::UserSymbol),
                    ("y", ParserTokenKind::Identifier),
                    ("for", ParserTokenKind::ReservedWord),
                    ("x", ParserTokenKind::Identifier),
                    ("<", ParserTokenKind::UserSymbol),
                    ("y", ParserTokenKind::Identifier),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("redefine", ParserTokenKind::ReservedWord),
                    ("mode", ParserTokenKind::ReservedWord),
                    ("ModeR", ParserTokenKind::Identifier),
                    (":", ParserTokenKind::ReservedSymbol),
                    ("Bad", ParserTokenKind::Identifier),
                    ("is", ParserTokenKind::ReservedWord),
                    ("set", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("coherence", ParserTokenKind::ReservedWord),
                    ("by", ParserTokenKind::ReservedWord),
                    ("Ref", ParserTokenKind::Identifier),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("end", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                ],
            ),
            Vec::new(),
        ));

        assert!(
            output.diagnostics.is_empty(),
            "task-27 redefinitions and aliases should parse without diagnostics: {:?}",
            output.diagnostics
        );
        let ast = output
            .ast
            .expect("task-27 redefinitions and aliases should keep an AST");
        assert_eq!(
            count_nodes(&ast, |kind| matches!(
                kind,
                SurfaceNodeKind::AttributeRedefinition
            )),
            1
        );
        assert_eq!(
            count_nodes(&ast, |kind| matches!(
                kind,
                SurfaceNodeKind::PredicateRedefinition
            )),
            1
        );
        assert_eq!(
            count_nodes(&ast, |kind| matches!(
                kind,
                SurfaceNodeKind::FunctorRedefinition
            )),
            2
        );
        assert_eq!(
            count_nodes(&ast, |kind| matches!(
                kind,
                SurfaceNodeKind::CoherenceCondition
            )),
            4
        );
        assert_eq!(
            count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::NotationAlias)),
            3
        );
        assert_eq!(
            count_nodes(&ast, |kind| matches!(
                kind,
                SurfaceNodeKind::NotationPattern
            )),
            6
        );
        assert_eq!(
            count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::ProofBlock)),
            1
        );
        let redefinition_nodes = ast
            .nodes()
            .iter()
            .filter(|node| {
                matches!(
                    node.kind,
                    SurfaceNodeKind::AttributeRedefinition
                        | SurfaceNodeKind::PredicateRedefinition
                        | SurfaceNodeKind::FunctorRedefinition
                )
            })
            .collect::<Vec<_>>();
        assert_eq!(redefinition_nodes.len(), 4);
        let mut direct_coherence_children = 0;
        for node in redefinition_nodes {
            let children = structural_children(&ast, node);
            assert!(
                children
                    .iter()
                    .any(|child| matches!(child.kind, SurfaceNodeKind::CoherenceCondition)),
                "{:?} should own a direct CoherenceCondition child",
                node.kind
            );
            assert!(
                !children
                    .iter()
                    .any(|child| matches!(child.kind, SurfaceNodeKind::CorrectnessCondition)),
                "{:?} should not own a CorrectnessCondition child",
                node.kind
            );
            direct_coherence_children += children
                .iter()
                .filter(|child| matches!(child.kind, SurfaceNodeKind::CoherenceCondition))
                .count();
        }
        assert_eq!(
            direct_coherence_children,
            count_nodes(&ast, |kind| matches!(
                kind,
                SurfaceNodeKind::CoherenceCondition
            )),
            "all task-27 coherence conditions should be nested under redefinitions"
        );
        assert_eq!(
            count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::VisibleItem)),
            2,
            "visibility should wrap the functor redefinition and private alias"
        );
        assert!(
            ast.nodes().iter().any(|node| {
                matches!(node.kind, SurfaceNodeKind::VisibleItem)
                    && node.children.get(1).is_some_and(|child_id| {
                        matches!(
                            ast.node(*child_id).unwrap().kind,
                            SurfaceNodeKind::FunctorRedefinition
                        )
                    })
            }),
            "definition-local visibility should wrap redefinitions"
        );
        assert!(
            ast.nodes().iter().any(|node| {
                matches!(node.kind, SurfaceNodeKind::VisibleItem)
                    && node.children.get(1).is_some_and(|child_id| {
                        matches!(
                            ast.node(*child_id).unwrap().kind,
                            SurfaceNodeKind::NotationAlias
                        )
                    })
            }),
            "definition-local visibility should wrap notation aliases"
        );
        let notation_pattern_texts = ast
            .nodes()
            .iter()
            .filter(|node| matches!(node.kind, SurfaceNodeKind::NotationPattern))
            .map(|node| direct_token_texts(&ast, node))
            .collect::<Vec<_>>();
        for expected in [
            &["FinSeq", "of", "G"][..],
            &["FinSequence", "of", "G"][..],
            &["finite"][..],
            &["empty"][..],
            &["x", ">=", "y"][..],
            &["x", "<", "y"][..],
        ] {
            assert!(
                notation_pattern_texts.iter().any(|texts| texts
                    .iter()
                    .map(String::as_str)
                    .eq(expected.iter().copied())),
                "raw NotationPattern children should contain {expected:?}"
            );
        }
        assert_eq!(
            count_nodes(&ast, |kind| matches!(
                kind,
                SurfaceNodeKind::PlaceholderItem
            )),
            1,
            "`redefine mode` is not a spec-defined concrete redefinition"
        );
    }

    #[test]
    fn parser_preserves_following_item_when_redefinition_coherence_is_missing() {
        let source_id = source_id(189);
        let output = parse(ParseRequest::new(
            source_id,
            Edition::new("2026"),
            token_sequence(
                source_id,
                &[
                    ("definition", ParserTokenKind::ReservedWord),
                    ("redefine", ParserTokenKind::ReservedWord),
                    ("attr", ParserTokenKind::ReservedWord),
                    ("AttrR", ParserTokenKind::Identifier),
                    (":", ParserTokenKind::ReservedSymbol),
                    ("x", ParserTokenKind::Identifier),
                    ("is", ParserTokenKind::ReservedWord),
                    ("empty", ParserTokenKind::UserSymbol),
                    ("means", ParserTokenKind::ReservedWord),
                    ("thesis", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("redefine", ParserTokenKind::ReservedWord),
                    ("pred", ParserTokenKind::ReservedWord),
                    ("x", ParserTokenKind::Identifier),
                    ("R", ParserTokenKind::UserSymbol),
                    ("y", ParserTokenKind::Identifier),
                    ("means", ParserTokenKind::ReservedWord),
                    ("thesis", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("coherence", ParserTokenKind::ReservedWord),
                    ("by", ParserTokenKind::ReservedWord),
                    ("Ref", ParserTokenKind::Identifier),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("end", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                ],
            ),
            Vec::new(),
        ));

        assert!(
            output.diagnostics.iter().any(|diagnostic| {
                diagnostic.code == SyntaxDiagnosticCode::MalformedFormulaExpression
            }),
            "missing coherence should be diagnosed without swallowing the next item: {:?}",
            output.diagnostics
        );
        let ast = output
            .ast
            .expect("missing coherence before the next item should still recover an AST");
        assert_eq!(
            count_nodes(&ast, |kind| matches!(
                kind,
                SurfaceNodeKind::AttributeRedefinition
            )),
            1
        );
        assert_eq!(
            count_nodes(&ast, |kind| matches!(
                kind,
                SurfaceNodeKind::PredicateRedefinition
            )),
            1,
            "the following redefinition item must remain parseable"
        );
    }

    #[test]
    fn parser_recovers_task27_redefinition_and_notation_gaps() {
        let source_id = source_id(190);
        let output = parse(ParseRequest::new(
            source_id,
            Edition::new("2026"),
            token_sequence(
                source_id,
                &[
                    ("definition", ParserTokenKind::ReservedWord),
                    ("redefine", ParserTokenKind::ReservedWord),
                    ("func", ParserTokenKind::ReservedWord),
                    ("MissingReturn", ParserTokenKind::Identifier),
                    (":", ParserTokenKind::ReservedSymbol),
                    ("f", ParserTokenKind::Identifier),
                    ("x", ParserTokenKind::Identifier),
                    ("->", ParserTokenKind::ReservedSymbol),
                    ("equals", ParserTokenKind::ReservedWord),
                    ("x", ParserTokenKind::Identifier),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("coherence", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("redefine", ParserTokenKind::ReservedWord),
                    ("pred", ParserTokenKind::ReservedWord),
                    ("means", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("coherence", ParserTokenKind::ReservedWord),
                    ("with", ParserTokenKind::ReservedWord),
                    ("by", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("synonym", ParserTokenKind::ReservedWord),
                    ("for", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("antonym", ParserTokenKind::ReservedWord),
                    ("x", ParserTokenKind::Identifier),
                    ("y", ParserTokenKind::Identifier),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("end", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                ],
            ),
            Vec::new(),
        ));

        assert!(
            output
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == SyntaxDiagnosticCode::MalformedTermExpression)
        );
        assert!(output.diagnostics.iter().any(|diagnostic| {
            diagnostic.code == SyntaxDiagnosticCode::MalformedFormulaExpression
        }));
        assert!(
            output
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == SyntaxDiagnosticCode::MalformedTypeExpression)
        );
        assert!(
            output
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == SyntaxDiagnosticCode::MalformedJustification)
        );
        let ast = output
            .ast
            .expect("task-27 malformed redefinitions and aliases should recover an AST");
        assert_eq!(
            count_nodes(&ast, |kind| matches!(
                kind,
                SurfaceNodeKind::FunctorRedefinition
            )),
            1
        );
        assert_eq!(
            count_nodes(&ast, |kind| matches!(
                kind,
                SurfaceNodeKind::PredicateRedefinition
            )),
            1
        );
        assert_eq!(
            count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::NotationAlias)),
            2
        );
        assert!(
            count_nodes(&ast, |kind| matches!(
                kind,
                SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::MissingTypeExpression)
            )) >= 1,
            "missing redefinition return types should insert MissingTypeExpression"
        );
        assert!(
            count_nodes(&ast, |kind| matches!(
                kind,
                SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::MissingTerm)
            )) >= 3,
            "malformed patterns and notation sides should insert MissingTerm"
        );
        assert!(
            count_nodes(&ast, |kind| matches!(
                kind,
                SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::MissingFormula)
            )) >= 1,
            "missing formula definiens bodies should insert MissingFormula"
        );
        assert!(
            count_nodes(&ast, |kind| matches!(
                kind,
                SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::MissingProofStep)
            )) >= 2,
            "malformed coherence tails should insert MissingProofStep"
        );
    }

    #[test]
    fn parser_parses_task28_property_clauses() {
        let source_id = source_id(191);
        let output = parse(ParseRequest::new(
            source_id,
            Edition::new("2026"),
            token_sequence(
                source_id,
                &[
                    ("definition", ParserTokenKind::ReservedWord),
                    ("symmetry", ParserTokenKind::ReservedWord),
                    ("by", ParserTokenKind::ReservedWord),
                    ("Ref", ParserTokenKind::Identifier),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("asymmetry", ParserTokenKind::ReservedWord),
                    ("proof", ParserTokenKind::ReservedWord),
                    ("thus", ParserTokenKind::ReservedWord),
                    ("thesis", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("end", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("connectedness", ParserTokenKind::ReservedWord),
                    ("by", ParserTokenKind::ReservedWord),
                    ("computation", ParserTokenKind::ReservedWord),
                    ("(", ParserTokenKind::ReservedSymbol),
                    ("steps", ParserTokenKind::ReservedWord),
                    (":", ParserTokenKind::ReservedSymbol),
                    ("1", ParserTokenKind::Numeral),
                    (")", ParserTokenKind::ReservedSymbol),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("reflexivity", ParserTokenKind::ReservedWord),
                    ("by", ParserTokenKind::ReservedWord),
                    ("Ref", ParserTokenKind::Identifier),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("irreflexivity", ParserTokenKind::ReservedWord),
                    ("by", ParserTokenKind::ReservedWord),
                    ("Ref", ParserTokenKind::Identifier),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("commutativity", ParserTokenKind::ReservedWord),
                    ("by", ParserTokenKind::ReservedWord),
                    ("Ref", ParserTokenKind::Identifier),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("idempotence", ParserTokenKind::ReservedWord),
                    ("by", ParserTokenKind::ReservedWord),
                    ("Ref", ParserTokenKind::Identifier),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("involutiveness", ParserTokenKind::ReservedWord),
                    ("proof", ParserTokenKind::ReservedWord),
                    ("thus", ParserTokenKind::ReservedWord),
                    ("thesis", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("end", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("projectivity", ParserTokenKind::ReservedWord),
                    ("by", ParserTokenKind::ReservedWord),
                    ("computation", ParserTokenKind::ReservedWord),
                    ("(", ParserTokenKind::ReservedSymbol),
                    ("steps", ParserTokenKind::ReservedWord),
                    (":", ParserTokenKind::ReservedSymbol),
                    ("1", ParserTokenKind::Numeral),
                    (")", ParserTokenKind::ReservedSymbol),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("sethood", ParserTokenKind::ReservedWord),
                    ("by", ParserTokenKind::ReservedWord),
                    ("Ref", ParserTokenKind::Identifier),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("end", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                ],
            ),
            Vec::new(),
        ));

        assert!(
            output.diagnostics.is_empty(),
            "task-28 property clauses should parse without diagnostics: {:?}",
            output.diagnostics
        );
        let ast = output
            .ast
            .expect("task-28 property clauses should keep an AST");
        assert_eq!(
            count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::PropertyClause)),
            10
        );
        assert_eq!(
            count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::ModeProperty)),
            0,
            "standalone sethood property items use the generic task-28 node"
        );
        assert_eq!(
            count_nodes(&ast, |kind| matches!(
                kind,
                SurfaceNodeKind::JustificationClause
            )),
            8
        );
        assert_eq!(
            count_nodes(&ast, |kind| matches!(
                kind,
                SurfaceNodeKind::ComputationJustification
            )),
            2
        );
        assert_eq!(
            count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::ProofBlock)),
            2
        );

        let property_texts = ast
            .nodes()
            .iter()
            .filter(|node| matches!(node.kind, SurfaceNodeKind::PropertyClause))
            .map(|node| subtree_token_texts(&ast, node))
            .collect::<Vec<_>>();
        for expected_keyword in [
            "symmetry",
            "asymmetry",
            "connectedness",
            "reflexivity",
            "irreflexivity",
            "commutativity",
            "idempotence",
            "involutiveness",
            "projectivity",
            "sethood",
        ] {
            assert!(
                property_texts.iter().any(|texts| texts
                    .first()
                    .is_some_and(|text| text.as_str() == expected_keyword)),
                "property clause should preserve keyword {expected_keyword}"
            );
        }
    }

    #[test]
    fn parser_recovers_task28_property_clause_gaps() {
        let source_id = source_id(192);
        let output = parse(ParseRequest::new(
            source_id,
            Edition::new("2026"),
            token_sequence(
                source_id,
                &[
                    ("definition", ParserTokenKind::ReservedWord),
                    ("symmetry", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("commutativity", ParserTokenKind::ReservedWord),
                    ("junk", ParserTokenKind::Identifier),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("sethood", ParserTokenKind::ReservedWord),
                    ("by", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("reflexivity", ParserTokenKind::ReservedWord),
                    ("proof", ParserTokenKind::ReservedWord),
                    ("end", ParserTokenKind::ReservedWord),
                    ("irreflexivity", ParserTokenKind::ReservedWord),
                    ("by", ParserTokenKind::ReservedWord),
                    ("Ref", ParserTokenKind::Identifier),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("end", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                ],
            ),
            Vec::new(),
        ));

        assert!(
            output
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == SyntaxDiagnosticCode::MalformedJustification)
        );
        assert!(
            output
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == SyntaxDiagnosticCode::MissingSemicolon)
        );
        let ast = output
            .ast
            .expect("task-28 malformed property clauses should recover an AST");
        assert_eq!(
            count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::PropertyClause)),
            5
        );
        assert!(
            count_nodes(&ast, |kind| matches!(
                kind,
                SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::MissingProofStep)
            )) >= 3,
            "missing property justifications should insert MissingProofStep recovery"
        );
        assert!(
            ast.nodes().iter().any(|node| {
                matches!(node.kind, SurfaceNodeKind::PropertyClause)
                    && subtree_token_texts(&ast, node)
                        .iter()
                        .map(String::as_str)
                        .eq(["irreflexivity", "by", "Ref", ";"])
            }),
            "missing semicolon after the preceding property must preserve the next property item"
        );
    }

    #[test]
    fn parser_preserves_property_items_after_malformed_property_justifications() {
        let source_id = source_id(193);
        let output = parse(ParseRequest::new(
            source_id,
            Edition::new("2026"),
            token_sequence(
                source_id,
                &[
                    ("definition", ParserTokenKind::ReservedWord),
                    ("symmetry", ParserTokenKind::ReservedWord),
                    ("by", ParserTokenKind::ReservedWord),
                    ("irreflexivity", ParserTokenKind::ReservedWord),
                    ("by", ParserTokenKind::ReservedWord),
                    ("Ref", ParserTokenKind::Identifier),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("asymmetry", ParserTokenKind::ReservedWord),
                    ("proof", ParserTokenKind::ReservedWord),
                    ("connectedness", ParserTokenKind::ReservedWord),
                    ("by", ParserTokenKind::ReservedWord),
                    ("Ref", ParserTokenKind::Identifier),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("mode", ParserTokenKind::ReservedWord),
                    ("Broken", ParserTokenKind::Identifier),
                    (":", ParserTokenKind::ReservedSymbol),
                    ("Carrier", ParserTokenKind::Identifier),
                    ("is", ParserTokenKind::ReservedWord),
                    ("set", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("sethood", ParserTokenKind::ReservedWord),
                    ("by", ParserTokenKind::ReservedWord),
                    ("projectivity", ParserTokenKind::ReservedWord),
                    ("by", ParserTokenKind::ReservedWord),
                    ("Ref", ParserTokenKind::Identifier),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("end", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                ],
            ),
            Vec::new(),
        ));

        assert!(
            output
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == SyntaxDiagnosticCode::MalformedJustification)
        );
        assert!(
            output
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == SyntaxDiagnosticCode::MissingSemicolon)
        );
        let ast = output
            .ast
            .expect("malformed property justifications should recover an AST");
        assert_eq!(
            count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::PropertyClause)),
            5,
            "malformed property justifications must preserve following property items"
        );
        assert_eq!(
            count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::ModeProperty)),
            1,
            "the malformed immediate sethood property remains attached to the mode definition"
        );
        for expected in [
            &["irreflexivity", "by", "Ref", ";"][..],
            &["connectedness", "by", "Ref", ";"][..],
            &["projectivity", "by", "Ref", ";"][..],
        ] {
            assert!(
                ast.nodes().iter().any(|node| {
                    matches!(node.kind, SurfaceNodeKind::PropertyClause)
                        && subtree_token_texts(&ast, node)
                            .iter()
                            .map(String::as_str)
                            .eq(expected.iter().copied())
                }),
                "following property item {expected:?} should remain parseable"
            );
        }
    }

    #[test]
    fn parser_parses_task29_structure_definitions_and_inheritance() {
        let source_id = source_id(194);
        let output = parse(ParseRequest::new(
            source_id,
            Edition::new("2026"),
            token_sequence(
                source_id,
                &[
                    ("definition", ParserTokenKind::ReservedWord),
                    ("struct", ParserTokenKind::ReservedWord),
                    ("Simple", ParserTokenKind::Identifier),
                    ("where", ParserTokenKind::ReservedWord),
                    ("field", ParserTokenKind::ReservedWord),
                    ("carrier", ParserTokenKind::Identifier),
                    ("->", ParserTokenKind::ReservedSymbol),
                    ("set", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("property", ParserTokenKind::ReservedWord),
                    ("size", ParserTokenKind::Identifier),
                    ("->", ParserTokenKind::ReservedSymbol),
                    ("set", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("end", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("struct", ParserTokenKind::ReservedWord),
                    ("Polynomial", ParserTokenKind::Identifier),
                    ("[", ParserTokenKind::ReservedSymbol),
                    ("R", ParserTokenKind::Identifier),
                    ("]", ParserTokenKind::ReservedSymbol),
                    ("where", ParserTokenKind::ReservedWord),
                    ("field", ParserTokenKind::ReservedWord),
                    ("coeffs", ParserTokenKind::Identifier),
                    ("->", ParserTokenKind::ReservedSymbol),
                    ("set", ParserTokenKind::ReservedWord),
                    (":=", ParserTokenKind::ReservedSymbol),
                    ("x", ParserTokenKind::Identifier),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("end", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("struct", ParserTokenKind::ReservedWord),
                    ("OfStruct", ParserTokenKind::Identifier),
                    ("of", ParserTokenKind::ReservedWord),
                    ("R", ParserTokenKind::Identifier),
                    ("where", ParserTokenKind::ReservedWord),
                    ("field", ParserTokenKind::ReservedWord),
                    ("base", ParserTokenKind::Identifier),
                    ("->", ParserTokenKind::ReservedSymbol),
                    ("set", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("end", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("struct", ParserTokenKind::ReservedWord),
                    ("OverStruct", ParserTokenKind::Identifier),
                    ("over", ParserTokenKind::ReservedWord),
                    ("R", ParserTokenKind::Identifier),
                    ("where", ParserTokenKind::ReservedWord),
                    ("field", ParserTokenKind::ReservedWord),
                    ("scalar", ParserTokenKind::Identifier),
                    ("->", ParserTokenKind::ReservedSymbol),
                    ("set", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("end", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("inherit", ParserTokenKind::ReservedWord),
                    ("Polynomial", ParserTokenKind::Identifier),
                    ("[", ParserTokenKind::ReservedSymbol),
                    ("R", ParserTokenKind::Identifier),
                    ("]", ParserTokenKind::ReservedSymbol),
                    ("extends", ParserTokenKind::ReservedWord),
                    ("TypeCaseStruct", ParserTokenKind::Identifier),
                    ("over", ParserTokenKind::ReservedWord),
                    ("r", ParserTokenKind::Identifier),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("inherit", ParserTokenKind::ReservedWord),
                    ("Derived", ParserTokenKind::Identifier),
                    ("extends", ParserTokenKind::ReservedWord),
                    ("set", ParserTokenKind::ReservedWord),
                    ("where", ParserTokenKind::ReservedWord),
                    ("field", ParserTokenKind::ReservedWord),
                    ("carrier", ParserTokenKind::Identifier),
                    ("from", ParserTokenKind::ReservedWord),
                    ("it", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("property", ParserTokenKind::ReservedWord),
                    ("size", ParserTokenKind::Identifier),
                    ("->", ParserTokenKind::ReservedSymbol),
                    ("set", ParserTokenKind::ReservedWord),
                    ("from", ParserTokenKind::ReservedWord),
                    ("size", ParserTokenKind::Identifier),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("coherence", ParserTokenKind::ReservedWord),
                    ("by", ParserTokenKind::ReservedWord),
                    ("Ref", ParserTokenKind::Identifier),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("end", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("inherit", ParserTokenKind::ReservedWord),
                    ("ProofChild", ParserTokenKind::Identifier),
                    ("extends", ParserTokenKind::ReservedWord),
                    ("ProofParent", ParserTokenKind::Identifier),
                    ("where", ParserTokenKind::ReservedWord),
                    ("field", ParserTokenKind::ReservedWord),
                    ("carrier", ParserTokenKind::Identifier),
                    ("from", ParserTokenKind::ReservedWord),
                    ("carrier", ParserTokenKind::Identifier),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("coherence", ParserTokenKind::ReservedWord),
                    ("proof", ParserTokenKind::ReservedWord),
                    ("thus", ParserTokenKind::ReservedWord),
                    ("thesis", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("end", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("end", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("public", ParserTokenKind::ReservedWord),
                    ("struct", ParserTokenKind::ReservedWord),
                    ("PublicStruct", ParserTokenKind::Identifier),
                    ("where", ParserTokenKind::ReservedWord),
                    ("field", ParserTokenKind::ReservedWord),
                    ("p", ParserTokenKind::Identifier),
                    ("->", ParserTokenKind::ReservedSymbol),
                    ("set", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("end", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("private", ParserTokenKind::ReservedWord),
                    ("inherit", ParserTokenKind::ReservedWord),
                    ("Hidden", ParserTokenKind::Identifier),
                    ("extends", ParserTokenKind::ReservedWord),
                    ("set", ParserTokenKind::ReservedWord),
                    ("where", ParserTokenKind::ReservedWord),
                    ("field", ParserTokenKind::ReservedWord),
                    ("h", ParserTokenKind::Identifier),
                    ("from", ParserTokenKind::ReservedWord),
                    ("it", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("end", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("end", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                ],
            ),
            Vec::new(),
        ));

        assert!(
            output.diagnostics.is_empty(),
            "task-29 structure syntax should parse without diagnostics: {:?}",
            output.diagnostics
        );
        let ast = output.ast.expect("task-29 structures should keep an AST");
        assert_eq!(
            count_nodes(&ast, |kind| matches!(
                kind,
                SurfaceNodeKind::StructureDefinition
            )),
            5
        );
        assert_eq!(
            count_nodes(&ast, |kind| matches!(
                kind,
                SurfaceNodeKind::StructurePattern
            )),
            5
        );
        assert_eq!(
            count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::StructureField)),
            5
        );
        assert_eq!(
            count_nodes(&ast, |kind| matches!(
                kind,
                SurfaceNodeKind::StructureProperty
            )),
            1
        );
        assert_eq!(
            count_nodes(&ast, |kind| matches!(
                kind,
                SurfaceNodeKind::InheritanceDefinition
            )),
            4
        );
        assert_eq!(
            count_nodes(&ast, |kind| matches!(
                kind,
                SurfaceNodeKind::InheritanceTarget
            )),
            8
        );
        assert_eq!(
            count_nodes(&ast, |kind| matches!(
                kind,
                SurfaceNodeKind::FieldRedefinition
            )),
            3
        );
        assert_eq!(
            count_nodes(&ast, |kind| matches!(
                kind,
                SurfaceNodeKind::PropertyRedefinition
            )),
            1
        );
        assert_eq!(
            count_nodes(&ast, |kind| matches!(
                kind,
                SurfaceNodeKind::CoherenceCondition
            )),
            2
        );
        assert_eq!(
            count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::ProofBlock)),
            1
        );
        assert_eq!(
            count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::VisibleItem)),
            2,
            "definition-local visibility wraps struct and inherit definitions"
        );
    }

    #[test]
    fn parser_recovers_task29_structure_definition_gaps() {
        let source_id = source_id(195);
        let output = parse(ParseRequest::new(
            source_id,
            Edition::new("2026"),
            token_sequence(
                source_id,
                &[
                    ("definition", ParserTokenKind::ReservedWord),
                    ("struct", ParserTokenKind::ReservedWord),
                    ("Empty", ParserTokenKind::Identifier),
                    ("where", ParserTokenKind::ReservedWord),
                    ("end", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("struct", ParserTokenKind::ReservedWord),
                    ("MissingSemi", ParserTokenKind::Identifier),
                    ("where", ParserTokenKind::ReservedWord),
                    ("field", ParserTokenKind::ReservedWord),
                    ("carrier", ParserTokenKind::Identifier),
                    ("->", ParserTokenKind::ReservedSymbol),
                    ("set", ParserTokenKind::ReservedWord),
                    ("property", ParserTokenKind::ReservedWord),
                    ("size", ParserTokenKind::Identifier),
                    ("->", ParserTokenKind::ReservedSymbol),
                    ("set", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("field", ParserTokenKind::ReservedWord),
                    ("missing", ParserTokenKind::Identifier),
                    ("->", ParserTokenKind::ReservedSymbol),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("end", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("inherit", ParserTokenKind::ReservedWord),
                    ("Memberless", ParserTokenKind::Identifier),
                    ("extends", ParserTokenKind::ReservedWord),
                    ("Base", ParserTokenKind::Identifier),
                    ("where", ParserTokenKind::ReservedWord),
                    ("end", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("inherit", ParserTokenKind::ReservedWord),
                    ("CoherenceOnly", ParserTokenKind::Identifier),
                    ("extends", ParserTokenKind::ReservedWord),
                    ("Base", ParserTokenKind::Identifier),
                    ("where", ParserTokenKind::ReservedWord),
                    ("coherence", ParserTokenKind::ReservedWord),
                    ("by", ParserTokenKind::ReservedWord),
                    ("Ref", ParserTokenKind::Identifier),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("end", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("inherit", ParserTokenKind::ReservedWord),
                    ("Broken", ParserTokenKind::Identifier),
                    ("extends", ParserTokenKind::ReservedWord),
                    ("where", ParserTokenKind::ReservedWord),
                    ("field", ParserTokenKind::ReservedWord),
                    ("from", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("field", ParserTokenKind::ReservedWord),
                    ("typed", ParserTokenKind::Identifier),
                    ("->", ParserTokenKind::ReservedSymbol),
                    ("from", ParserTokenKind::ReservedWord),
                    ("carrier", ParserTokenKind::Identifier),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("property", ParserTokenKind::ReservedWord),
                    ("p", ParserTokenKind::Identifier),
                    ("->", ParserTokenKind::ReservedSymbol),
                    ("set", ParserTokenKind::ReservedWord),
                    ("from", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("coherence", ParserTokenKind::ReservedWord),
                    ("with", ParserTokenKind::ReservedWord),
                    ("C", ParserTokenKind::Identifier),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("end", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("struct", ParserTokenKind::ReservedWord),
                    ("Follows", ParserTokenKind::Identifier),
                    ("where", ParserTokenKind::ReservedWord),
                    ("field", ParserTokenKind::ReservedWord),
                    ("ok", ParserTokenKind::Identifier),
                    ("->", ParserTokenKind::ReservedSymbol),
                    ("set", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("end", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("end", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                ],
            ),
            Vec::new(),
        ));

        assert!(
            output
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == SyntaxDiagnosticCode::MalformedTermExpression)
        );
        assert!(
            output
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == SyntaxDiagnosticCode::MalformedTypeExpression)
        );
        assert!(
            output
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == SyntaxDiagnosticCode::MalformedJustification)
        );
        assert!(
            output
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == SyntaxDiagnosticCode::MissingSemicolon)
        );
        let ast = output
            .ast
            .expect("task-29 malformed structures should recover an AST");
        assert_eq!(
            count_nodes(&ast, |kind| matches!(
                kind,
                SurfaceNodeKind::StructureDefinition
            )),
            3
        );
        assert_eq!(
            count_nodes(&ast, |kind| matches!(
                kind,
                SurfaceNodeKind::InheritanceDefinition
            )),
            3
        );
        assert!(
            ast.nodes().iter().any(|node| {
                matches!(node.kind, SurfaceNodeKind::StructureProperty)
                    && subtree_token_texts(&ast, node)
                        .iter()
                        .map(String::as_str)
                        .eq(["property", "size", "->", "set", ";"])
            }),
            "missing field semicolon must preserve following structure property"
        );
        assert!(
            ast.nodes().iter().any(|node| {
                matches!(node.kind, SurfaceNodeKind::StructureDefinition)
                    && subtree_token_texts(&ast, node)
                        .iter()
                        .any(|text| text == "Follows")
            }),
            "malformed inheritance content must preserve following structure definitions"
        );
    }

    #[test]
    fn proof_blocks_remain_rejected_for_simple_justification_only_hosts() {
        let source_id = source_id(174);
        let output = parse(ParseRequest::new(
            source_id,
            Edition::new("2026"),
            token_sequence(
                source_id,
                &[
                    ("let", ParserTokenKind::ReservedWord),
                    ("x", ParserTokenKind::Identifier),
                    ("proof", ParserTokenKind::ReservedWord),
                    ("end", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("consider", ParserTokenKind::ReservedWord),
                    ("y", ParserTokenKind::Identifier),
                    ("being", ParserTokenKind::ReservedWord),
                    ("set", ParserTokenKind::ReservedWord),
                    ("such", ParserTokenKind::ReservedWord),
                    ("that", ParserTokenKind::ReservedWord),
                    ("thesis", ParserTokenKind::ReservedWord),
                    ("proof", ParserTokenKind::ReservedWord),
                    ("end", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("reconsider", ParserTokenKind::ReservedWord),
                    ("z", ParserTokenKind::Identifier),
                    ("as", ParserTokenKind::ReservedWord),
                    ("set", ParserTokenKind::ReservedWord),
                    ("proof", ParserTokenKind::ReservedWord),
                    ("end", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("x", ParserTokenKind::Identifier),
                    ("=", ParserTokenKind::ReservedSymbol),
                    ("y", ParserTokenKind::Identifier),
                    (".=", ParserTokenKind::ReservedSymbol),
                    ("z", ParserTokenKind::Identifier),
                    ("proof", ParserTokenKind::ReservedWord),
                    ("end", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("per", ParserTokenKind::ReservedWord),
                    ("cases", ParserTokenKind::ReservedWord),
                    ("proof", ParserTokenKind::ReservedWord),
                    ("end", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                ],
            ),
            Vec::new(),
        ));

        assert!(
            output
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == SyntaxDiagnosticCode::MalformedTermExpression),
            "unexpected proof tails on simple hosts should be diagnosed: {:?}",
            output.diagnostics
        );
        let ast = output
            .ast
            .expect("simple-host proof tails should recover an AST");
        assert_eq!(
            count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::ProofBlock)),
            0,
            "simple-justification-only hosts must not accept proof blocks"
        );
        assert!(
            count_nodes(&ast, |kind| matches!(
                kind,
                SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::SkippedToken)
            )) >= 4,
            "each non-proof host should recover unexpected proof syntax as skipped tokens"
        );
    }

    #[test]
    fn annotation_prefix_stays_with_the_following_placeholder_item() {
        let source_id = source_id(69);
        let tokens = token_sequence(
            source_id,
            &[
                ("@[", ParserTokenKind::ReservedSymbol),
                ("label", ParserTokenKind::Identifier),
                ("]", ParserTokenKind::ReservedSymbol),
                ("theorem", ParserTokenKind::ReservedWord),
                ("T", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
            ],
        );
        let expected_range = range(source_id, 0, tokens.last().unwrap().span.end);
        let output = parse(ParseRequest::new(
            source_id,
            Edition::new("2026"),
            tokens,
            Vec::new(),
        ));

        assert!(output.diagnostics.is_empty());
        let ast = output.ast.expect("annotated placeholder item should parse");
        let item_list = single_node(&ast, |kind| matches!(kind, SurfaceNodeKind::ItemList));
        assert_eq!(item_list.children.len(), 1);
        let item = ast.node(item_list.children[0]).unwrap();
        assert_eq!(item.range, expected_range);
        assert_eq!(item.children.len(), 6);
    }

    #[test]
    fn reserved_word_garbage_before_first_item_is_recovered() {
        let source_id = source_id(70);
        let tokens = token_sequence(
            source_id,
            &[
                ("and", ParserTokenKind::ReservedWord),
                ("theorem", ParserTokenKind::ReservedWord),
                ("T", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
            ],
        );
        let output = parse(ParseRequest::new(
            source_id,
            Edition::new("2026"),
            tokens,
            Vec::new(),
        ));

        assert_eq!(output.diagnostics.len(), 1);
        assert_eq!(
            output.diagnostics[0].code,
            SyntaxDiagnosticCode::UnexpectedTopLevelToken
        );
        assert_eq!(output.diagnostics[0].primary, range(source_id, 0, 3));
        let ast = output
            .ast
            .expect("reserved garbage before an item should recover");
        let item_list = single_node(&ast, |kind| matches!(kind, SurfaceNodeKind::ItemList));
        assert_eq!(item_list.children.len(), 2);
        assert!(matches!(
            ast.node(item_list.children[0]).unwrap().kind,
            SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::SkippedToken)
        ));
    }

    #[test]
    fn formula_if_and_otherwise_do_not_open_nested_block_depth() {
        let source_id = source_id(71);
        let tokens = token_sequence(
            source_id,
            &[
                ("definition", ParserTokenKind::ReservedWord),
                ("property", ParserTokenKind::ReservedWord),
                ("F", ParserTokenKind::Identifier),
                ("means", ParserTokenKind::ReservedWord),
                ("P", ParserTokenKind::Identifier),
                ("if", ParserTokenKind::ReservedWord),
                ("Q", ParserTokenKind::Identifier),
                ("otherwise", ParserTokenKind::ReservedWord),
                ("R", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
                ("end", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
            ],
        );
        let output = parse(ParseRequest::new(
            source_id,
            Edition::new("2026"),
            tokens,
            Vec::new(),
        ));

        assert!(output.diagnostics.is_empty());
        let ast = output
            .ast
            .expect("conditional formula keywords should not unbalance the block");
        let item_list = single_node(&ast, |kind| matches!(kind, SurfaceNodeKind::ItemList));
        assert_eq!(item_list.children.len(), 1);
    }

    #[test]
    fn statement_if_after_separator_stays_inside_block_placeholder() {
        let source_id = source_id(73);
        let tokens = token_sequence(
            source_id,
            &[
                ("definition", ParserTokenKind::ReservedWord),
                ("algorithm", ParserTokenKind::ReservedWord),
                ("x", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
                ("if", ParserTokenKind::ReservedWord),
                ("P", ParserTokenKind::Identifier),
                ("do", ParserTokenKind::ReservedWord),
                ("end", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("end", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("end", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
            ],
        );
        let expected_range = range(source_id, 0, tokens.last().unwrap().span.end);
        let output = parse(ParseRequest::new(
            source_id,
            Edition::new("2026"),
            tokens,
            Vec::new(),
        ));

        assert!(output.diagnostics.is_empty());
        let ast = output
            .ast
            .expect("statement if should not close its container early");
        let item_list = single_node(&ast, |kind| matches!(kind, SurfaceNodeKind::ItemList));
        assert_eq!(item_list.children.len(), 1);
        assert_eq!(
            ast.node(item_list.children[0]).unwrap().range,
            expected_range
        );
    }

    #[test]
    fn eof_missing_semicolon_placeholder_keeps_consumed_tokens() {
        let source_id = source_id(72);
        let output = parse(ParseRequest::new(
            source_id,
            Edition::new("2026"),
            vec![
                token(source_id, ParserTokenKind::ReservedWord, "theorem", 0, 7),
                token(source_id, ParserTokenKind::Identifier, "T", 8, 9),
            ],
            Vec::new(),
        ));

        assert_eq!(output.diagnostics.len(), 1);
        assert_eq!(
            output.diagnostics[0].code,
            SyntaxDiagnosticCode::MissingSemicolon
        );
        let ast = output.ast.expect("EOF recovery should keep an AST");
        let item_list = single_node(&ast, |kind| matches!(kind, SurfaceNodeKind::ItemList));
        assert_eq!(item_list.children.len(), 1);
        let item = ast.node(item_list.children[0]).unwrap();
        assert_eq!(item.range, range(source_id, 0, 9));
        assert_eq!(item.children.len(), 2);
    }

    #[test]
    fn parser_parses_reserve_type_expression_attribute_chain_and_of_arguments() {
        let source_id = source_id(98);
        let tokens = token_sequence(
            source_id,
            &[
                ("reserve", ParserTokenKind::ReservedWord),
                ("x", ParserTokenKind::Identifier),
                (",", ParserTokenKind::ReservedSymbol),
                ("y", ParserTokenKind::Identifier),
                ("for", ParserTokenKind::ReservedWord),
                ("non", ParserTokenKind::ReservedWord),
                ("empty", ParserTokenKind::UserSymbol),
                ("T", ParserTokenKind::UserSymbol),
                ("of", ParserTokenKind::ReservedWord),
                ("a", ParserTokenKind::Identifier),
                (",", ParserTokenKind::ReservedSymbol),
                ("b", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
            ],
        );
        let output = parse(ParseRequest::new(
            source_id,
            Edition::new("2026"),
            tokens,
            Vec::new(),
        ));

        assert!(
            output.diagnostics.is_empty(),
            "unexpected diagnostics: {:?}",
            output.diagnostics
        );
        let ast = output.ast.expect("reserve type expression should parse");
        let reserve = single_node(&ast, |kind| matches!(kind, SurfaceNodeKind::ReserveItem));
        assert_eq!(reserve.children.len(), 3);
        let segment = single_node(&ast, |kind| matches!(kind, SurfaceNodeKind::ReserveSegment));
        assert_eq!(
            segment
                .children
                .iter()
                .filter_map(|id| ast.node(*id))
                .filter_map(mizar_syntax::SurfaceNode::token_text)
                .collect::<Vec<_>>(),
            vec!["x", ",", "y", "for"]
        );
        let type_expression =
            single_node(&ast, |kind| matches!(kind, SurfaceNodeKind::TypeExpression));
        assert_eq!(type_expression.children.len(), 2);
        assert!(matches!(
            ast.node(type_expression.children[0]).unwrap().kind,
            SurfaceNodeKind::AttributeChain
        ));
        assert!(matches!(
            ast.node(type_expression.children[1]).unwrap().kind,
            SurfaceNodeKind::TypeHead
        ));
        let attribute = single_node(&ast, |kind| matches!(kind, SurfaceNodeKind::AttributeRef));
        assert_eq!(
            ast.node(attribute.children[0]).unwrap().token_text(),
            Some("non")
        );
        assert!(matches!(
            ast.node(attribute.children[1]).unwrap().kind,
            SurfaceNodeKind::QualifiedSymbol
        ));
        let type_arguments =
            single_node(&ast, |kind| matches!(kind, SurfaceNodeKind::TypeArguments));
        assert_eq!(
            ast.node(type_arguments.children[0]).unwrap().token_text(),
            Some("of")
        );
        assert_eq!(
            type_arguments
                .children
                .iter()
                .filter_map(|id| ast.node(*id))
                .filter(|node| matches!(node.kind, SurfaceNodeKind::TermExpression))
                .count(),
            2
        );
    }

    #[test]
    fn parser_parses_over_type_arguments_as_term_expressions() {
        let source_id = source_id(116);
        let tokens = token_sequence(
            source_id,
            &[
                ("reserve", ParserTokenKind::ReservedWord),
                ("w", ParserTokenKind::Identifier),
                ("for", ParserTokenKind::ReservedWord),
                ("T", ParserTokenKind::UserSymbol),
                ("over", ParserTokenKind::ReservedWord),
                ("c", ParserTokenKind::Identifier),
                (",", ParserTokenKind::ReservedSymbol),
                ("d", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
            ],
        );
        let output = parse(ParseRequest::new(
            source_id,
            Edition::new("2026"),
            tokens,
            Vec::new(),
        ));

        assert!(
            output.diagnostics.is_empty(),
            "unexpected diagnostics: {:?}",
            output.diagnostics
        );
        let ast = output.ast.expect("over arguments should parse");
        let type_arguments =
            single_node(&ast, |kind| matches!(kind, SurfaceNodeKind::TypeArguments));
        assert_eq!(
            ast.node(type_arguments.children[0]).unwrap().token_text(),
            Some("over")
        );
        assert_eq!(
            type_arguments
                .children
                .iter()
                .filter_map(|id| ast.node(*id))
                .filter(|node| matches!(node.kind, SurfaceNodeKind::TermExpression))
                .count(),
            2
        );
    }

    #[test]
    fn parser_parses_qualified_symbol_term_references() {
        let source_id = source_id(117);
        let tokens = token_sequence(
            source_id,
            &[
                ("reserve", ParserTokenKind::ReservedWord),
                ("x", ParserTokenKind::Identifier),
                ("for", ParserTokenKind::ReservedWord),
                ("T", ParserTokenKind::UserSymbol),
                ("of", ParserTokenKind::ReservedWord),
                ("Ns", ParserTokenKind::Identifier),
                (".", ParserTokenKind::ReservedSymbol),
                ("Value", ParserTokenKind::UserSymbol),
                (";", ParserTokenKind::ReservedSymbol),
            ],
        );
        let output = parse(ParseRequest::new(
            source_id,
            Edition::new("2026"),
            tokens,
            Vec::new(),
        ));

        assert!(
            output.diagnostics.is_empty(),
            "unexpected diagnostics: {:?}",
            output.diagnostics
        );
        let ast = output.ast.expect("qualified term should parse");
        let term_reference =
            single_node(&ast, |kind| matches!(kind, SurfaceNodeKind::TermReference));
        let qualified = term_reference
            .children
            .iter()
            .filter_map(|id| ast.node(*id))
            .find(|node| matches!(node.kind, SurfaceNodeKind::QualifiedSymbol))
            .expect("term reference should own a qualified symbol");
        assert_eq!(
            qualified_symbol_token_texts(&ast, qualified),
            vec!["Ns", ".", "Value"]
        );
    }

    #[test]
    fn parser_recovers_missing_commas_between_of_over_terms() {
        let source_id = source_id(118);
        let tokens = token_sequence(
            source_id,
            &[
                ("reserve", ParserTokenKind::ReservedWord),
                ("x", ParserTokenKind::Identifier),
                ("for", ParserTokenKind::ReservedWord),
                ("T", ParserTokenKind::UserSymbol),
                ("of", ParserTokenKind::ReservedWord),
                ("a", ParserTokenKind::Identifier),
                ("b", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
            ],
        );
        let b_range = tokens[6].span;
        let output = parse(ParseRequest::new(
            source_id,
            Edition::new("2026"),
            tokens,
            Vec::new(),
        ));

        assert_eq!(output.diagnostics.len(), 1);
        assert_eq!(
            output.diagnostics[0].code,
            SyntaxDiagnosticCode::MalformedTermExpression
        );
        assert_eq!(output.diagnostics[0].primary, b_range);
        let ast = output.ast.expect("missing comma should recover");
        let type_arguments =
            single_node(&ast, |kind| matches!(kind, SurfaceNodeKind::TypeArguments));
        assert_eq!(
            type_arguments
                .children
                .iter()
                .filter_map(|id| ast.node(*id))
                .filter(|node| matches!(node.kind, SurfaceNodeKind::TermExpression))
                .count(),
            1
        );
        assert!(type_arguments.children.iter().any(|id| {
            matches!(
                ast.node(*id).unwrap().kind,
                SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::SkippedToken)
            )
        }));
    }

    #[test]
    fn parser_recovers_missing_commas_inside_delimited_term_lists() {
        let source_id = source_id(119);
        let tokens = token_sequence(
            source_id,
            &[
                ("reserve", ParserTokenKind::ReservedWord),
                ("x", ParserTokenKind::Identifier),
                ("for", ParserTokenKind::ReservedWord),
                ("T", ParserTokenKind::UserSymbol),
                ("of", ParserTokenKind::ReservedWord),
                ("f", ParserTokenKind::Identifier),
                ("(", ParserTokenKind::ReservedSymbol),
                ("a", ParserTokenKind::Identifier),
                ("b", ParserTokenKind::Identifier),
                (")", ParserTokenKind::ReservedSymbol),
                (";", ParserTokenKind::ReservedSymbol),
            ],
        );
        let b_range = tokens[8].span;
        let output = parse(ParseRequest::new(
            source_id,
            Edition::new("2026"),
            tokens,
            Vec::new(),
        ));

        assert_eq!(output.diagnostics.len(), 1);
        assert_eq!(
            output.diagnostics[0].code,
            SyntaxDiagnosticCode::MalformedTermExpression
        );
        assert_eq!(output.diagnostics[0].primary, b_range);
        let ast = output
            .ast
            .expect("missing comma in application should recover");
        let application = single_node(&ast, |kind| {
            matches!(kind, SurfaceNodeKind::ApplicationTerm)
        });
        assert!(application.children.iter().any(|id| {
            matches!(
                ast.node(*id).unwrap().kind,
                SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::SkippedToken)
            )
        }));
        assert!(!application.children.iter().any(|id| {
            matches!(
                ast.node(*id).unwrap().kind,
                SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::UnmatchedOpeningDelimiter)
            )
        }));
    }

    #[test]
    fn parser_recovers_missing_attribute_argument_close_before_type_head() {
        let source_id = source_id(120);
        let tokens = token_sequence(
            source_id,
            &[
                ("reserve", ParserTokenKind::ReservedWord),
                ("x", ParserTokenKind::Identifier),
                ("for", ParserTokenKind::ReservedWord),
                ("empty", ParserTokenKind::UserSymbol),
                ("(", ParserTokenKind::ReservedSymbol),
                ("a", ParserTokenKind::Identifier),
                ("T", ParserTokenKind::UserSymbol),
                (";", ParserTokenKind::ReservedSymbol),
            ],
        );
        let type_head_range = tokens[6].span;
        let output = parse(ParseRequest::new(
            source_id,
            Edition::new("2026"),
            tokens,
            Vec::new(),
        ));

        assert_eq!(output.diagnostics.len(), 1);
        assert_eq!(
            output.diagnostics[0].code,
            SyntaxDiagnosticCode::MalformedTermExpression
        );
        assert_eq!(output.diagnostics[0].primary, type_head_range);
        let ast = output.ast.expect("missing attribute close should recover");
        let attribute = single_node(&ast, |kind| matches!(kind, SurfaceNodeKind::AttributeRef));
        assert!(attribute.children.iter().any(|id| {
            matches!(
                ast.node(*id).unwrap().kind,
                SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::UnmatchedOpeningDelimiter)
            )
        }));
        let type_head = single_node(&ast, |kind| matches!(kind, SurfaceNodeKind::TypeHead));
        assert_eq!(
            qualified_symbol_token_texts(
                &ast,
                ast.node(type_head.children[0])
                    .expect("type head should own a qualified symbol")
            ),
            vec!["T"]
        );
    }

    #[test]
    fn parser_parses_structure_constructors_with_of_arguments() {
        let source_id = source_id(121);
        let tokens = token_sequence(
            source_id,
            &[
                ("reserve", ParserTokenKind::ReservedWord),
                ("x", ParserTokenKind::Identifier),
                ("for", ParserTokenKind::ReservedWord),
                ("T", ParserTokenKind::UserSymbol),
                ("of", ParserTokenKind::ReservedWord),
                ("S", ParserTokenKind::UserSymbol),
                ("of", ParserTokenKind::ReservedWord),
                ("a", ParserTokenKind::Identifier),
                ("(", ParserTokenKind::ReservedSymbol),
                ("x", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("b", ParserTokenKind::Identifier),
                (")", ParserTokenKind::ReservedSymbol),
                (";", ParserTokenKind::ReservedSymbol),
            ],
        );
        let output = parse(ParseRequest::new(
            source_id,
            Edition::new("2026"),
            tokens,
            Vec::new(),
        ));

        assert!(
            output.diagnostics.is_empty(),
            "unexpected diagnostics: {:?}",
            output.diagnostics
        );
        let ast = output
            .ast
            .expect("structure constructor with of args should parse");
        let constructor = single_node(&ast, |kind| {
            matches!(kind, SurfaceNodeKind::StructureConstructor)
        });
        assert!(
            constructor.children.iter().any(|id| {
                matches!(ast.node(*id).unwrap().kind, SurfaceNodeKind::TypeArguments)
            })
        );
        assert!(
            constructor.children.iter().any(|id| {
                matches!(ast.node(*id).unwrap().kind, SurfaceNodeKind::FieldArgument)
            })
        );
    }

    #[test]
    fn parser_parses_selector_access_chains_and_calls() {
        let source_id = source_id(124);
        let tokens = token_sequence(
            source_id,
            &[
                ("reserve", ParserTokenKind::ReservedWord),
                ("x", ParserTokenKind::Identifier),
                ("for", ParserTokenKind::ReservedWord),
                ("T", ParserTokenKind::UserSymbol),
                ("of", ParserTokenKind::ReservedWord),
                ("p", ParserTokenKind::Identifier),
                (".", ParserTokenKind::ReservedSymbol),
                ("x", ParserTokenKind::Identifier),
                (",", ParserTokenKind::ReservedSymbol),
                ("line", ParserTokenKind::Identifier),
                (".", ParserTokenKind::ReservedSymbol),
                ("finish", ParserTokenKind::Identifier),
                (".", ParserTokenKind::ReservedSymbol),
                ("y", ParserTokenKind::Identifier),
                (",", ParserTokenKind::ReservedSymbol),
                ("M", ParserTokenKind::Identifier),
                (".", ParserTokenKind::ReservedSymbol),
                ("binop", ParserTokenKind::Identifier),
                ("(", ParserTokenKind::ReservedSymbol),
                ("a", ParserTokenKind::Identifier),
                (",", ParserTokenKind::ReservedSymbol),
                ("b", ParserTokenKind::Identifier),
                (")", ParserTokenKind::ReservedSymbol),
                (";", ParserTokenKind::ReservedSymbol),
            ],
        );
        let output = parse(ParseRequest::new(
            source_id,
            Edition::new("2026"),
            tokens,
            Vec::new(),
        ));

        assert!(
            output.diagnostics.is_empty(),
            "unexpected diagnostics: {:?}",
            output.diagnostics
        );
        let ast = output.ast.expect("selector terms should parse");
        assert_eq!(
            ast.nodes()
                .iter()
                .filter(|node| matches!(node.kind, SurfaceNodeKind::SelectorAccess))
                .count(),
            4
        );
        assert!(
            ast.nodes()
                .iter()
                .filter(|node| matches!(node.kind, SurfaceNodeKind::SelectorAccess))
                .any(|node| {
                    node.children
                        .iter()
                        .filter_map(|id| ast.node(*id))
                        .filter_map(mizar_syntax::SurfaceNode::token_text)
                        .collect::<Vec<_>>()
                        == vec![".", "binop", "(", ",", ")"]
                })
        );
    }

    #[test]
    fn parser_parses_functional_structure_updates() {
        let source_id = source_id(125);
        let tokens = token_sequence(
            source_id,
            &[
                ("reserve", ParserTokenKind::ReservedWord),
                ("x", ParserTokenKind::Identifier),
                ("for", ParserTokenKind::ReservedWord),
                ("T", ParserTokenKind::UserSymbol),
                ("of", ParserTokenKind::ReservedWord),
                ("p", ParserTokenKind::Identifier),
                ("with", ParserTokenKind::ReservedWord),
                ("(", ParserTokenKind::ReservedSymbol),
                ("x", ParserTokenKind::Identifier),
                (":=", ParserTokenKind::ReservedSymbol),
                ("y", ParserTokenKind::Identifier),
                (",", ParserTokenKind::ReservedSymbol),
                ("start", ParserTokenKind::Identifier),
                (".", ParserTokenKind::ReservedSymbol),
                ("x", ParserTokenKind::Identifier),
                (":=", ParserTokenKind::ReservedSymbol),
                ("f", ParserTokenKind::Identifier),
                ("(", ParserTokenKind::ReservedSymbol),
                ("a", ParserTokenKind::Identifier),
                (")", ParserTokenKind::ReservedSymbol),
                (")", ParserTokenKind::ReservedSymbol),
                (";", ParserTokenKind::ReservedSymbol),
            ],
        );
        let output = parse(ParseRequest::new(
            source_id,
            Edition::new("2026"),
            tokens,
            Vec::new(),
        ));

        assert!(
            output.diagnostics.is_empty(),
            "unexpected diagnostics: {:?}",
            output.diagnostics
        );
        let ast = output.ast.expect("structure update should parse");
        single_node(&ast, |kind| {
            matches!(kind, SurfaceNodeKind::StructureUpdate)
        });
        assert_eq!(
            ast.nodes()
                .iter()
                .filter(|node| matches!(node.kind, SurfaceNodeKind::FieldUpdate))
                .count(),
            2
        );
        assert!(
            ast.nodes()
                .iter()
                .any(|node| matches!(node.kind, SurfaceNodeKind::ApplicationTerm))
        );
    }

    #[test]
    fn parser_keeps_selector_arguments_before_structure_fields() {
        let source_id = source_id(126);
        let tokens = token_sequence(
            source_id,
            &[
                ("reserve", ParserTokenKind::ReservedWord),
                ("x", ParserTokenKind::Identifier),
                ("for", ParserTokenKind::ReservedWord),
                ("T", ParserTokenKind::UserSymbol),
                ("of", ParserTokenKind::ReservedWord),
                ("S", ParserTokenKind::UserSymbol),
                ("of", ParserTokenKind::ReservedWord),
                ("p", ParserTokenKind::Identifier),
                (".", ParserTokenKind::ReservedSymbol),
                ("x", ParserTokenKind::Identifier),
                ("(", ParserTokenKind::ReservedSymbol),
                ("f", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("y", ParserTokenKind::Identifier),
                (")", ParserTokenKind::ReservedSymbol),
                (";", ParserTokenKind::ReservedSymbol),
            ],
        );
        let output = parse(ParseRequest::new(
            source_id,
            Edition::new("2026"),
            tokens,
            Vec::new(),
        ));

        assert!(
            output.diagnostics.is_empty(),
            "unexpected diagnostics: {:?}",
            output.diagnostics
        );
        let ast = output
            .ast
            .expect("selector type arg before fields should parse");
        let constructor = single_node(&ast, |kind| {
            matches!(kind, SurfaceNodeKind::StructureConstructor)
        });
        assert!(
            constructor.children.iter().any(|id| {
                matches!(ast.node(*id).unwrap().kind, SurfaceNodeKind::FieldArgument)
            })
        );
        single_node(&ast, |kind| matches!(kind, SurfaceNodeKind::SelectorAccess));
    }

    #[test]
    fn parser_recovers_missing_structure_update_value() {
        let source_id = source_id(127);
        let tokens = token_sequence(
            source_id,
            &[
                ("reserve", ParserTokenKind::ReservedWord),
                ("x", ParserTokenKind::Identifier),
                ("for", ParserTokenKind::ReservedWord),
                ("T", ParserTokenKind::UserSymbol),
                ("of", ParserTokenKind::ReservedWord),
                ("p", ParserTokenKind::Identifier),
                ("with", ParserTokenKind::ReservedWord),
                ("(", ParserTokenKind::ReservedSymbol),
                ("x", ParserTokenKind::Identifier),
                (":=", ParserTokenKind::ReservedSymbol),
                (")", ParserTokenKind::ReservedSymbol),
                (";", ParserTokenKind::ReservedSymbol),
            ],
        );
        let close_range = tokens[10].span;
        let output = parse(ParseRequest::new(
            source_id,
            Edition::new("2026"),
            tokens,
            Vec::new(),
        ));

        assert_eq!(output.diagnostics.len(), 1);
        assert_eq!(
            output.diagnostics[0].code,
            SyntaxDiagnosticCode::MalformedTermExpression
        );
        assert_eq!(output.diagnostics[0].primary, close_range);
        let ast = output.ast.expect("missing update value should recover");
        let field_update = single_node(&ast, |kind| matches!(kind, SurfaceNodeKind::FieldUpdate));
        assert!(field_update.children.iter().any(|id| {
            matches!(
                ast.node(*id).unwrap().kind,
                SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::MissingTerm)
            )
        }));
    }

    #[test]
    fn parser_recovers_missing_structure_update_close() {
        let source_id = source_id(128);
        let tokens = token_sequence(
            source_id,
            &[
                ("reserve", ParserTokenKind::ReservedWord),
                ("x", ParserTokenKind::Identifier),
                ("for", ParserTokenKind::ReservedWord),
                ("T", ParserTokenKind::UserSymbol),
                ("of", ParserTokenKind::ReservedWord),
                ("p", ParserTokenKind::Identifier),
                ("with", ParserTokenKind::ReservedWord),
                ("(", ParserTokenKind::ReservedSymbol),
                ("x", ParserTokenKind::Identifier),
                (":=", ParserTokenKind::ReservedSymbol),
                ("y", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
            ],
        );
        let semicolon_range = tokens[11].span;
        let output = parse(ParseRequest::new(
            source_id,
            Edition::new("2026"),
            tokens,
            Vec::new(),
        ));

        assert_eq!(output.diagnostics.len(), 1);
        assert_eq!(
            output.diagnostics[0].code,
            SyntaxDiagnosticCode::MalformedTermExpression
        );
        assert_eq!(output.diagnostics[0].primary, semicolon_range);
        let ast = output.ast.expect("missing update close should recover");
        let structure_update = single_node(&ast, |kind| {
            matches!(kind, SurfaceNodeKind::StructureUpdate)
        });
        assert!(structure_update.children.iter().any(|id| {
            matches!(
                ast.node(*id).unwrap().kind,
                SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::UnmatchedOpeningDelimiter)
            )
        }));
    }

    #[test]
    fn parser_recovers_missing_selector_name() {
        let source_id = source_id(129);
        let tokens = token_sequence(
            source_id,
            &[
                ("reserve", ParserTokenKind::ReservedWord),
                ("x", ParserTokenKind::Identifier),
                ("for", ParserTokenKind::ReservedWord),
                ("T", ParserTokenKind::UserSymbol),
                ("of", ParserTokenKind::ReservedWord),
                ("p", ParserTokenKind::Identifier),
                (".", ParserTokenKind::ReservedSymbol),
                (";", ParserTokenKind::ReservedSymbol),
            ],
        );
        let semicolon_range = tokens[7].span;
        let output = parse(ParseRequest::new(
            source_id,
            Edition::new("2026"),
            tokens,
            Vec::new(),
        ));

        assert_eq!(output.diagnostics.len(), 1);
        assert_eq!(
            output.diagnostics[0].code,
            SyntaxDiagnosticCode::MalformedTermExpression
        );
        assert_eq!(output.diagnostics[0].primary, semicolon_range);
        let ast = output.ast.expect("missing selector name should recover");
        single_node(&ast, |kind| matches!(kind, SurfaceNodeKind::SelectorAccess));
    }

    #[test]
    fn parser_recovers_missing_selector_call_close() {
        let source_id = source_id(130);
        let tokens = token_sequence(
            source_id,
            &[
                ("reserve", ParserTokenKind::ReservedWord),
                ("x", ParserTokenKind::Identifier),
                ("for", ParserTokenKind::ReservedWord),
                ("T", ParserTokenKind::UserSymbol),
                ("of", ParserTokenKind::ReservedWord),
                ("p", ParserTokenKind::Identifier),
                (".", ParserTokenKind::ReservedSymbol),
                ("f", ParserTokenKind::Identifier),
                ("(", ParserTokenKind::ReservedSymbol),
                ("a", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
            ],
        );
        let semicolon_range = tokens[10].span;
        let output = parse(ParseRequest::new(
            source_id,
            Edition::new("2026"),
            tokens,
            Vec::new(),
        ));

        assert_eq!(output.diagnostics.len(), 1);
        assert_eq!(
            output.diagnostics[0].code,
            SyntaxDiagnosticCode::MalformedTermExpression
        );
        assert_eq!(output.diagnostics[0].primary, semicolon_range);
        let ast = output
            .ast
            .expect("missing selector-call close should recover");
        let selector = single_node(&ast, |kind| matches!(kind, SurfaceNodeKind::SelectorAccess));
        assert!(selector.children.iter().any(|id| {
            matches!(
                ast.node(*id).unwrap().kind,
                SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::UnmatchedOpeningDelimiter)
            )
        }));
    }

    #[test]
    fn parser_recovers_malformed_structure_update_selector_path_once() {
        let source_id = source_id(131);
        let tokens = token_sequence(
            source_id,
            &[
                ("reserve", ParserTokenKind::ReservedWord),
                ("x", ParserTokenKind::Identifier),
                ("for", ParserTokenKind::ReservedWord),
                ("T", ParserTokenKind::UserSymbol),
                ("of", ParserTokenKind::ReservedWord),
                ("p", ParserTokenKind::Identifier),
                ("with", ParserTokenKind::ReservedWord),
                ("(", ParserTokenKind::ReservedSymbol),
                ("start", ParserTokenKind::Identifier),
                (".", ParserTokenKind::ReservedSymbol),
                (")", ParserTokenKind::ReservedSymbol),
                (";", ParserTokenKind::ReservedSymbol),
            ],
        );
        let close_range = tokens[10].span;
        let output = parse(ParseRequest::new(
            source_id,
            Edition::new("2026"),
            tokens,
            Vec::new(),
        ));

        assert_eq!(output.diagnostics.len(), 1);
        assert_eq!(
            output.diagnostics[0].code,
            SyntaxDiagnosticCode::MalformedTermExpression
        );
        assert_eq!(output.diagnostics[0].primary, close_range);
        let ast = output
            .ast
            .expect("malformed update selector path should recover");
        single_node(&ast, |kind| matches!(kind, SurfaceNodeKind::FieldUpdate));
    }

    #[test]
    fn parser_parses_qua_qualification_chains_left_associatively() {
        let source_id = source_id(132);
        let tokens = token_sequence(
            source_id,
            &[
                ("reserve", ParserTokenKind::ReservedWord),
                ("x", ParserTokenKind::Identifier),
                ("for", ParserTokenKind::ReservedWord),
                ("T", ParserTokenKind::UserSymbol),
                ("of", ParserTokenKind::ReservedWord),
                ("a", ParserTokenKind::Identifier),
                ("qua", ParserTokenKind::ReservedWord),
                ("R", ParserTokenKind::UserSymbol),
                ("qua", ParserTokenKind::ReservedWord),
                ("S", ParserTokenKind::UserSymbol),
                (";", ParserTokenKind::ReservedSymbol),
            ],
        );
        let output = parse(ParseRequest::new(
            source_id,
            Edition::new("2026"),
            tokens,
            Vec::new(),
        ));

        assert!(
            output.diagnostics.is_empty(),
            "unexpected diagnostics: {:?}",
            output.diagnostics
        );
        let ast = output.ast.expect("qua chain should parse");
        assert_eq!(
            ast.nodes()
                .iter()
                .filter(|node| matches!(node.kind, SurfaceNodeKind::QuaExpression))
                .count(),
            2
        );
        assert!(ast.nodes().iter().any(|node| {
            matches!(node.kind, SurfaceNodeKind::QuaExpression)
                && node
                    .children
                    .iter()
                    .any(|id| matches!(ast.node(*id).unwrap().kind, SurfaceNodeKind::QuaExpression))
        }));
    }

    #[test]
    fn parser_parses_qua_after_selector_application_and_parentheses() {
        let source_id = source_id(133);
        let tokens = token_sequence(
            source_id,
            &[
                ("reserve", ParserTokenKind::ReservedWord),
                ("x", ParserTokenKind::Identifier),
                ("for", ParserTokenKind::ReservedWord),
                ("T", ParserTokenKind::UserSymbol),
                ("of", ParserTokenKind::ReservedWord),
                ("p", ParserTokenKind::Identifier),
                (".", ParserTokenKind::ReservedSymbol),
                ("x", ParserTokenKind::Identifier),
                ("qua", ParserTokenKind::ReservedWord),
                ("R", ParserTokenKind::UserSymbol),
                (",", ParserTokenKind::ReservedSymbol),
                ("f", ParserTokenKind::Identifier),
                ("(", ParserTokenKind::ReservedSymbol),
                ("a", ParserTokenKind::Identifier),
                (")", ParserTokenKind::ReservedSymbol),
                ("qua", ParserTokenKind::ReservedWord),
                ("R", ParserTokenKind::UserSymbol),
                (",", ParserTokenKind::ReservedSymbol),
                ("(", ParserTokenKind::ReservedSymbol),
                ("p", ParserTokenKind::Identifier),
                ("qua", ParserTokenKind::ReservedWord),
                ("R", ParserTokenKind::UserSymbol),
                (")", ParserTokenKind::ReservedSymbol),
                (".", ParserTokenKind::ReservedSymbol),
                ("x", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
            ],
        );
        let output = parse(ParseRequest::new(
            source_id,
            Edition::new("2026"),
            tokens,
            Vec::new(),
        ));

        assert!(
            output.diagnostics.is_empty(),
            "unexpected diagnostics: {:?}",
            output.diagnostics
        );
        let ast = output.ast.expect("qua precedence examples should parse");
        assert_eq!(
            ast.nodes()
                .iter()
                .filter(|node| matches!(node.kind, SurfaceNodeKind::QuaExpression))
                .count(),
            3
        );
        assert_eq!(
            ast.nodes()
                .iter()
                .filter(|node| matches!(node.kind, SurfaceNodeKind::SelectorAccess))
                .count(),
            2
        );
        assert!(
            ast.nodes()
                .iter()
                .any(|node| matches!(node.kind, SurfaceNodeKind::ApplicationTerm))
        );
        assert!(
            ast.nodes()
                .iter()
                .any(|node| matches!(node.kind, SurfaceNodeKind::ParenthesizedTerm))
        );

        assert!(ast.nodes().iter().any(|node| {
            matches!(node.kind, SurfaceNodeKind::QuaExpression)
                && direct_child_has_kind(&ast, node, |kind| {
                    matches!(kind, SurfaceNodeKind::SelectorAccess)
                })
        }));
        assert!(ast.nodes().iter().any(|node| {
            matches!(node.kind, SurfaceNodeKind::QuaExpression)
                && direct_child_has_kind(&ast, node, |kind| {
                    matches!(kind, SurfaceNodeKind::ApplicationTerm)
                })
        }));
        assert!(ast.nodes().iter().any(|node| {
            matches!(node.kind, SurfaceNodeKind::SelectorAccess)
                && direct_child_has_kind(&ast, node, |kind| {
                    matches!(kind, SurfaceNodeKind::ParenthesizedTerm)
                })
        }));
        assert!(ast.nodes().iter().any(|node| {
            matches!(node.kind, SurfaceNodeKind::ParenthesizedTerm)
                && subtree_contains_kind(&ast, node.children[1], |kind| {
                    matches!(kind, SurfaceNodeKind::QuaExpression)
                })
        }));
    }

    #[test]
    fn parser_parses_qua_inside_target_type_arguments_before_outer_chain() {
        let source_id = source_id(134);
        let tokens = token_sequence(
            source_id,
            &[
                ("reserve", ParserTokenKind::ReservedWord),
                ("x", ParserTokenKind::Identifier),
                ("for", ParserTokenKind::ReservedWord),
                ("T", ParserTokenKind::UserSymbol),
                ("of", ParserTokenKind::ReservedWord),
                ("x", ParserTokenKind::Identifier),
                ("qua", ParserTokenKind::ReservedWord),
                ("Element", ParserTokenKind::UserSymbol),
                ("of", ParserTokenKind::ReservedWord),
                ("S", ParserTokenKind::UserSymbol),
                ("qua", ParserTokenKind::ReservedWord),
                ("Magma", ParserTokenKind::UserSymbol),
                (";", ParserTokenKind::ReservedSymbol),
            ],
        );
        let output = parse(ParseRequest::new(
            source_id,
            Edition::new("2026"),
            tokens,
            Vec::new(),
        ));

        assert!(
            output.diagnostics.is_empty(),
            "unexpected diagnostics: {:?}",
            output.diagnostics
        );
        let ast = output
            .ast
            .expect("qua inside target type argument should parse");
        assert_eq!(
            ast.nodes()
                .iter()
                .filter(|node| matches!(node.kind, SurfaceNodeKind::QuaExpression))
                .count(),
            2
        );
        assert!(ast.nodes().iter().any(|node| {
            matches!(node.kind, SurfaceNodeKind::QuaExpression)
                && node.children.iter().any(|id| {
                    matches!(ast.node(*id).unwrap().kind, SurfaceNodeKind::TypeExpression)
                        && subtree_contains_kind(&ast, *id, |kind| {
                            matches!(kind, SurfaceNodeKind::QuaExpression)
                        })
                })
        }));
    }

    #[test]
    fn parser_recovers_missing_qua_target_type() {
        let source_id = source_id(135);
        let tokens = token_sequence(
            source_id,
            &[
                ("reserve", ParserTokenKind::ReservedWord),
                ("x", ParserTokenKind::Identifier),
                ("for", ParserTokenKind::ReservedWord),
                ("T", ParserTokenKind::UserSymbol),
                ("of", ParserTokenKind::ReservedWord),
                ("x", ParserTokenKind::Identifier),
                ("qua", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
            ],
        );
        let semicolon_range = tokens[7].span;
        let output = parse(ParseRequest::new(
            source_id,
            Edition::new("2026"),
            tokens,
            Vec::new(),
        ));

        assert_eq!(output.diagnostics.len(), 1);
        assert_eq!(
            output.diagnostics[0].code,
            SyntaxDiagnosticCode::MalformedTypeExpression
        );
        assert_eq!(output.diagnostics[0].primary, semicolon_range);
        let ast = output.ast.expect("missing qua target should recover");
        let qua = single_node(&ast, |kind| matches!(kind, SurfaceNodeKind::QuaExpression));
        assert!(qua.children.iter().any(|id| {
            matches!(
                ast.node(*id).unwrap().kind,
                SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::MissingTypeExpression)
            )
        }));
    }

    #[test]
    fn parser_recovers_malformed_qua_target_tail() {
        let source_id = source_id(136);
        let tokens = token_sequence(
            source_id,
            &[
                ("reserve", ParserTokenKind::ReservedWord),
                ("x", ParserTokenKind::Identifier),
                ("for", ParserTokenKind::ReservedWord),
                ("T", ParserTokenKind::UserSymbol),
                ("of", ParserTokenKind::ReservedWord),
                ("x", ParserTokenKind::Identifier),
                ("qua", ParserTokenKind::ReservedWord),
                ("bad", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
            ],
        );
        let bad_range = tokens[7].span;
        let output = parse(ParseRequest::new(
            source_id,
            Edition::new("2026"),
            tokens,
            Vec::new(),
        ));

        assert_eq!(output.diagnostics.len(), 1);
        assert_eq!(
            output.diagnostics[0].code,
            SyntaxDiagnosticCode::MalformedTypeExpression
        );
        assert_eq!(output.diagnostics[0].primary, bad_range);
        let ast = output.ast.expect("malformed qua target should recover");
        let qua = single_node(&ast, |kind| matches!(kind, SurfaceNodeKind::QuaExpression));
        assert!(qua.children.iter().any(|id| {
            matches!(
                ast.node(*id).unwrap().kind,
                SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::MissingTypeExpression)
            )
        }));
        assert!(qua.children.iter().any(|id| {
            matches!(
                ast.node(*id).unwrap().kind,
                SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::SkippedToken)
            )
        }));
    }

    #[test]
    fn parser_recovers_missing_bracket_qua_target_type() {
        let source_id = source_id(137);
        let tokens = token_sequence(
            source_id,
            &[
                ("reserve", ParserTokenKind::ReservedWord),
                ("x", ParserTokenKind::Identifier),
                ("for", ParserTokenKind::ReservedWord),
                ("T", ParserTokenKind::UserSymbol),
                ("[", ParserTokenKind::ReservedSymbol),
                ("V", ParserTokenKind::Identifier),
                ("qua", ParserTokenKind::ReservedWord),
                ("]", ParserTokenKind::ReservedSymbol),
                (";", ParserTokenKind::ReservedSymbol),
            ],
        );
        let close_range = tokens[7].span;
        let output = parse(ParseRequest::new(
            source_id,
            Edition::new("2026"),
            tokens,
            Vec::new(),
        ));

        assert_eq!(output.diagnostics.len(), 1);
        assert_eq!(
            output.diagnostics[0].code,
            SyntaxDiagnosticCode::MalformedTypeExpression
        );
        assert_eq!(output.diagnostics[0].primary, close_range);
        let ast = output
            .ast
            .expect("missing bracket qua target should recover");
        let qua = single_node(&ast, |kind| matches!(kind, SurfaceNodeKind::QuaExpression));
        assert!(qua.children.iter().any(|id| {
            matches!(
                ast.node(*id).unwrap().kind,
                SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::MissingTypeExpression)
            )
        }));
    }

    #[test]
    fn parser_rejects_attribute_bearing_bracket_qua_target_as_radix_only() {
        let source_id = source_id(138);
        let tokens = token_sequence(
            source_id,
            &[
                ("reserve", ParserTokenKind::ReservedWord),
                ("x", ParserTokenKind::Identifier),
                ("for", ParserTokenKind::ReservedWord),
                ("T", ParserTokenKind::UserSymbol),
                ("[", ParserTokenKind::ReservedSymbol),
                ("V", ParserTokenKind::Identifier),
                ("qua", ParserTokenKind::ReservedWord),
                ("non", ParserTokenKind::ReservedWord),
                ("empty", ParserTokenKind::UserSymbol),
                ("R", ParserTokenKind::UserSymbol),
                ("]", ParserTokenKind::ReservedSymbol),
                (";", ParserTokenKind::ReservedSymbol),
            ],
        );
        let non_range = tokens[7].span;
        let output = parse(ParseRequest::new(
            source_id,
            Edition::new("2026"),
            tokens,
            Vec::new(),
        ));

        assert_eq!(output.diagnostics.len(), 1);
        assert_eq!(
            output.diagnostics[0].code,
            SyntaxDiagnosticCode::MalformedTypeExpression
        );
        assert_eq!(output.diagnostics[0].primary, non_range);
        let ast = output
            .ast
            .expect("attribute-bearing bracket qua target should recover");
        let qua = single_node(&ast, |kind| matches!(kind, SurfaceNodeKind::QuaExpression));
        assert!(qua.children.iter().any(|id| {
            matches!(
                ast.node(*id).unwrap().kind,
                SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::MissingTypeExpression)
            )
        }));
        assert!(qua.children.iter().any(|id| {
            matches!(
                ast.node(*id).unwrap().kind,
                SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::SkippedToken)
            )
        }));
    }

    #[test]
    fn parser_recovers_choice_terms_missing_type_expression() {
        let source_id = source_id(122);
        let tokens = token_sequence(
            source_id,
            &[
                ("reserve", ParserTokenKind::ReservedWord),
                ("x", ParserTokenKind::Identifier),
                ("for", ParserTokenKind::ReservedWord),
                ("T", ParserTokenKind::UserSymbol),
                ("of", ParserTokenKind::ReservedWord),
                ("the", ParserTokenKind::ReservedWord),
                (",", ParserTokenKind::ReservedSymbol),
                ("a", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
            ],
        );
        let comma_range = tokens[6].span;
        let output = parse(ParseRequest::new(
            source_id,
            Edition::new("2026"),
            tokens,
            Vec::new(),
        ));

        assert_eq!(output.diagnostics.len(), 1);
        assert_eq!(
            output.diagnostics[0].code,
            SyntaxDiagnosticCode::MalformedTypeExpression
        );
        assert_eq!(output.diagnostics[0].primary, comma_range);
        let ast = output
            .ast
            .expect("choice missing type expression should recover");
        let choice = single_node(&ast, |kind| matches!(kind, SurfaceNodeKind::ChoiceTerm));
        assert!(choice.children.iter().any(|id| {
            matches!(
                ast.node(*id).unwrap().kind,
                SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::MissingTypeExpression)
            )
        }));
    }

    #[test]
    fn parser_parses_choice_terms_with_set_enumeration_type_args_in_attribute_args() {
        let source_id = source_id(123);
        let tokens = token_sequence(
            source_id,
            &[
                ("reserve", ParserTokenKind::ReservedWord),
                ("x", ParserTokenKind::Identifier),
                ("for", ParserTokenKind::ReservedWord),
                ("empty", ParserTokenKind::UserSymbol),
                ("(", ParserTokenKind::ReservedSymbol),
                ("the", ParserTokenKind::ReservedWord),
                ("T", ParserTokenKind::UserSymbol),
                ("of", ParserTokenKind::ReservedWord),
                ("{", ParserTokenKind::ReservedSymbol),
                ("a", ParserTokenKind::Identifier),
                (",", ParserTokenKind::ReservedSymbol),
                ("b", ParserTokenKind::Identifier),
                ("}", ParserTokenKind::ReservedSymbol),
                (")", ParserTokenKind::ReservedSymbol),
                ("U", ParserTokenKind::UserSymbol),
                (";", ParserTokenKind::ReservedSymbol),
            ],
        );
        let output = parse(ParseRequest::new(
            source_id,
            Edition::new("2026"),
            tokens,
            Vec::new(),
        ));

        assert!(
            output.diagnostics.is_empty(),
            "unexpected diagnostics: {:?}",
            output.diagnostics
        );
        let ast = output
            .ast
            .expect("choice term with set enumeration type arg should parse");
        let attribute = single_node(&ast, |kind| matches!(kind, SurfaceNodeKind::AttributeRef));
        assert!(
            attribute.children.iter().any(|id| {
                matches!(ast.node(*id).unwrap().kind, SurfaceNodeKind::TermExpression)
            })
        );
        single_node(&ast, |kind| matches!(kind, SurfaceNodeKind::ChoiceTerm));
        single_node(&ast, |kind| matches!(kind, SurfaceNodeKind::SetEnumeration));
    }

    #[test]
    fn parser_preserves_parameter_prefix_and_bracket_qua_arguments() {
        let source_id = source_id(99);
        let tokens = token_sequence(
            source_id,
            &[
                ("reserve", ParserTokenKind::ReservedWord),
                ("x", ParserTokenKind::Identifier),
                ("for", ParserTokenKind::ReservedWord),
                ("n", ParserTokenKind::Identifier),
                ("-", ParserTokenKind::ReservedSymbol),
                ("empty", ParserTokenKind::UserSymbol),
                ("T", ParserTokenKind::UserSymbol),
                ("[", ParserTokenKind::ReservedSymbol),
                ("set", ParserTokenKind::ReservedWord),
                (",", ParserTokenKind::ReservedSymbol),
                ("Ns", ParserTokenKind::Identifier),
                (".", ParserTokenKind::ReservedSymbol),
                ("U", ParserTokenKind::UserSymbol),
                (",", ParserTokenKind::ReservedSymbol),
                ("V", ParserTokenKind::Identifier),
                (",", ParserTokenKind::ReservedSymbol),
                ("W", ParserTokenKind::Identifier),
                ("qua", ParserTokenKind::ReservedWord),
                ("R", ParserTokenKind::UserSymbol),
                ("]", ParserTokenKind::ReservedSymbol),
                (";", ParserTokenKind::ReservedSymbol),
            ],
        );
        let output = parse(ParseRequest::new(
            source_id,
            Edition::new("2026"),
            tokens,
            Vec::new(),
        ));

        assert!(output.diagnostics.is_empty());
        let ast = output
            .ast
            .expect("parameter prefix and bracket args should parse");
        let prefix = single_node(&ast, |kind| {
            matches!(kind, SurfaceNodeKind::ParameterPrefix)
        });
        assert_eq!(
            prefix
                .children
                .iter()
                .filter_map(|id| ast.node(*id))
                .filter_map(mizar_syntax::SurfaceNode::token_text)
                .collect::<Vec<_>>(),
            vec!["n", "-"]
        );
        let type_arguments =
            single_node(&ast, |kind| matches!(kind, SurfaceNodeKind::TypeArguments));
        assert_eq!(
            ast.node(type_arguments.children[0]).unwrap().token_text(),
            Some("[")
        );
        assert_eq!(
            ast.node(*type_arguments.children.last().unwrap())
                .unwrap()
                .token_text(),
            Some("]")
        );
        assert_eq!(
            type_arguments
                .children
                .iter()
                .filter_map(|id| ast.node(*id))
                .filter(|node| matches!(node.kind, SurfaceNodeKind::TypeExpression))
                .count(),
            2
        );
        assert!(
            !type_arguments
                .children
                .iter()
                .filter_map(|id| ast.node(*id))
                .any(|node| matches!(node.kind, SurfaceNodeKind::TermPlaceholder)),
            "bracket qua_arg should no longer use a temporary term placeholder"
        );
        let qua_term = type_arguments
            .children
            .iter()
            .filter_map(|id| ast.node(*id))
            .find(|node| {
                matches!(node.kind, SurfaceNodeKind::TermExpression)
                    && direct_child_has_kind(&ast, node, |kind| {
                        matches!(kind, SurfaceNodeKind::QuaExpression)
                    })
            })
            .expect("bracket qua_arg should be a term expression after task 11");
        assert_eq!(
            type_arguments
                .children
                .iter()
                .filter_map(|id| ast.node(*id))
                .filter(|node| matches!(node.kind, SurfaceNodeKind::TermExpression))
                .count(),
            2
        );
        assert!(type_arguments.children.iter().any(|id| {
            matches!(ast.node(*id).unwrap().kind, SurfaceNodeKind::TermExpression)
                && direct_child_has_kind(&ast, ast.node(*id).unwrap(), |kind| {
                    matches!(kind, SurfaceNodeKind::TermReference)
                })
        }));
        assert!(
            qua_term.children.iter().any(|id| {
                matches!(ast.node(*id).unwrap().kind, SurfaceNodeKind::QuaExpression)
            })
        );
    }

    #[test]
    fn parser_preserves_struct_qualified_attribute_spelling() {
        let source_id = source_id(102);
        let tokens = token_sequence(
            source_id,
            &[
                ("reserve", ParserTokenKind::ReservedWord),
                ("x", ParserTokenKind::Identifier),
                ("for", ParserTokenKind::ReservedWord),
                ("TypeCaseStruct", ParserTokenKind::UserSymbol),
                (".", ParserTokenKind::ReservedSymbol),
                ("TypeCaseAttr", ParserTokenKind::UserSymbol),
                ("TypeCaseMode", ParserTokenKind::UserSymbol),
                (";", ParserTokenKind::ReservedSymbol),
            ],
        );
        let output = parse(ParseRequest::new(
            source_id,
            Edition::new("2026"),
            tokens,
            Vec::new(),
        ));

        assert!(
            output.diagnostics.is_empty(),
            "unexpected diagnostics: {:?}",
            output.diagnostics
        );
        let ast = output.ast.expect("struct-qualified attribute should parse");
        let attribute = single_node(&ast, |kind| matches!(kind, SurfaceNodeKind::AttributeRef));
        let qualified = attribute
            .children
            .iter()
            .filter_map(|id| ast.node(*id))
            .find(|node| matches!(node.kind, SurfaceNodeKind::QualifiedSymbol))
            .expect("attribute should own a qualified symbol");
        assert_eq!(
            qualified_symbol_token_texts(&ast, qualified),
            vec!["TypeCaseStruct", ".", "TypeCaseAttr"]
        );
        let type_head = single_node(&ast, |kind| matches!(kind, SurfaceNodeKind::TypeHead));
        assert_eq!(
            qualified_symbol_token_texts(
                &ast,
                ast.node(type_head.children[0])
                    .expect("type head should own its qualified symbol")
            ),
            vec!["TypeCaseMode"]
        );
    }

    #[test]
    fn parser_parses_primary_terms_in_type_and_attribute_arguments() {
        let source_id = source_id(111);
        let tokens = token_sequence(
            source_id,
            &[
                ("reserve", ParserTokenKind::ReservedWord),
                ("x", ParserTokenKind::Identifier),
                ("for", ParserTokenKind::ReservedWord),
                ("empty", ParserTokenKind::UserSymbol),
                ("(", ParserTokenKind::ReservedSymbol),
                ("it", ParserTokenKind::ReservedWord),
                (",", ParserTokenKind::ReservedSymbol),
                ("7", ParserTokenKind::Numeral),
                (",", ParserTokenKind::ReservedSymbol),
                ("(", ParserTokenKind::ReservedSymbol),
                ("a", ParserTokenKind::Identifier),
                (")", ParserTokenKind::ReservedSymbol),
                (",", ParserTokenKind::ReservedSymbol),
                ("the", ParserTokenKind::ReservedWord),
                ("set", ParserTokenKind::ReservedWord),
                (",", ParserTokenKind::ReservedSymbol),
                ("f", ParserTokenKind::Identifier),
                ("(", ParserTokenKind::ReservedSymbol),
                ("a", ParserTokenKind::Identifier),
                (",", ParserTokenKind::ReservedSymbol),
                ("1", ParserTokenKind::Numeral),
                (")", ParserTokenKind::ReservedSymbol),
                (",", ParserTokenKind::ReservedSymbol),
                ("S", ParserTokenKind::UserSymbol),
                ("(", ParserTokenKind::ReservedSymbol),
                ("x", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("a", ParserTokenKind::Identifier),
                (",", ParserTokenKind::ReservedSymbol),
                ("y", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("{", ParserTokenKind::ReservedSymbol),
                ("a", ParserTokenKind::Identifier),
                (",", ParserTokenKind::ReservedSymbol),
                ("[", ParserTokenKind::ReservedSymbol),
                ("b", ParserTokenKind::Identifier),
                ("]", ParserTokenKind::ReservedSymbol),
                ("}", ParserTokenKind::ReservedSymbol),
                (")", ParserTokenKind::ReservedSymbol),
                (",", ParserTokenKind::ReservedSymbol),
                ("[", ParserTokenKind::ReservedSymbol),
                ("a", ParserTokenKind::Identifier),
                (",", ParserTokenKind::ReservedSymbol),
                ("b", ParserTokenKind::Identifier),
                ("]", ParserTokenKind::ReservedSymbol),
                (")", ParserTokenKind::ReservedSymbol),
                ("T", ParserTokenKind::UserSymbol),
                ("of", ParserTokenKind::ReservedWord),
                ("{", ParserTokenKind::ReservedSymbol),
                ("a", ParserTokenKind::Identifier),
                (",", ParserTokenKind::ReservedSymbol),
                ("b", ParserTokenKind::Identifier),
                ("}", ParserTokenKind::ReservedSymbol),
                (";", ParserTokenKind::ReservedSymbol),
            ],
        );
        let output = parse(ParseRequest::new(
            source_id,
            Edition::new("2026"),
            tokens,
            Vec::new(),
        ));

        assert!(
            output.diagnostics.is_empty(),
            "unexpected diagnostics: {:?}",
            output.diagnostics
        );
        let ast = output.ast.expect("primary terms should parse");
        assert!(
            ast.nodes()
                .iter()
                .any(|node| matches!(node.kind, SurfaceNodeKind::ItTerm))
        );
        assert!(
            ast.nodes()
                .iter()
                .any(|node| matches!(node.kind, SurfaceNodeKind::NumeralTerm))
        );
        assert!(
            ast.nodes()
                .iter()
                .any(|node| matches!(node.kind, SurfaceNodeKind::ParenthesizedTerm))
        );
        assert!(
            ast.nodes()
                .iter()
                .any(|node| matches!(node.kind, SurfaceNodeKind::ChoiceTerm))
        );
        assert!(
            ast.nodes()
                .iter()
                .any(|node| matches!(node.kind, SurfaceNodeKind::ApplicationTerm))
        );
        assert!(
            ast.nodes()
                .iter()
                .any(|node| matches!(node.kind, SurfaceNodeKind::StructureConstructor))
        );
        assert_eq!(
            ast.nodes()
                .iter()
                .filter(|node| matches!(node.kind, SurfaceNodeKind::FieldArgument))
                .count(),
            2
        );
        assert!(
            ast.nodes()
                .iter()
                .any(|node| matches!(node.kind, SurfaceNodeKind::SetEnumeration))
        );
    }

    #[test]
    fn parser_keeps_zero_field_constructor_syntax_as_application() {
        let source_id = source_id(112);
        let tokens = token_sequence(
            source_id,
            &[
                ("reserve", ParserTokenKind::ReservedWord),
                ("x", ParserTokenKind::Identifier),
                ("for", ParserTokenKind::ReservedWord),
                ("T", ParserTokenKind::UserSymbol),
                ("of", ParserTokenKind::ReservedWord),
                ("S", ParserTokenKind::UserSymbol),
                ("(", ParserTokenKind::ReservedSymbol),
                (")", ParserTokenKind::ReservedSymbol),
                (";", ParserTokenKind::ReservedSymbol),
            ],
        );
        let output = parse(ParseRequest::new(
            source_id,
            Edition::new("2026"),
            tokens,
            Vec::new(),
        ));

        assert!(output.diagnostics.is_empty());
        let ast = output
            .ast
            .expect("zero-field constructor syntax should parse");
        assert_eq!(
            ast.nodes()
                .iter()
                .filter(|node| matches!(node.kind, SurfaceNodeKind::ApplicationTerm))
                .count(),
            1
        );
        assert!(
            !ast.nodes()
                .iter()
                .any(|node| matches!(node.kind, SurfaceNodeKind::StructureConstructor))
        );
    }

    #[test]
    fn parser_recovers_missing_type_argument_bracket_close() {
        let source_id = source_id(100);
        let tokens = token_sequence(
            source_id,
            &[
                ("reserve", ParserTokenKind::ReservedWord),
                ("x", ParserTokenKind::Identifier),
                ("for", ParserTokenKind::ReservedWord),
                ("T", ParserTokenKind::UserSymbol),
                ("[", ParserTokenKind::ReservedSymbol),
                ("set", ParserTokenKind::ReservedWord),
                (",", ParserTokenKind::ReservedSymbol),
                ("object", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
            ],
        );
        let semicolon_range = tokens[8].span;
        let output = parse(ParseRequest::new(
            source_id,
            Edition::new("2026"),
            tokens,
            Vec::new(),
        ));

        assert_eq!(output.diagnostics.len(), 1);
        assert_eq!(
            output.diagnostics[0].code,
            SyntaxDiagnosticCode::MalformedTypeExpression
        );
        assert_eq!(output.diagnostics[0].primary, semicolon_range);
        let ast = output.ast.expect("missing bracket close should recover");
        let type_arguments =
            single_node(&ast, |kind| matches!(kind, SurfaceNodeKind::TypeArguments));
        assert!(type_arguments.children.iter().any(|id| {
            matches!(
                ast.node(*id).unwrap().kind,
                SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::UnmatchedOpeningDelimiter)
            )
        }));
    }

    #[test]
    fn parser_recovers_empty_and_trailing_bracket_type_arguments() {
        let empty_source_id = source_id(103);
        let tokens = token_sequence(
            empty_source_id,
            &[
                ("reserve", ParserTokenKind::ReservedWord),
                ("x", ParserTokenKind::Identifier),
                ("for", ParserTokenKind::ReservedWord),
                ("T", ParserTokenKind::UserSymbol),
                ("[", ParserTokenKind::ReservedSymbol),
                ("]", ParserTokenKind::ReservedSymbol),
                (";", ParserTokenKind::ReservedSymbol),
            ],
        );
        let close_range = tokens[5].span;
        let output = parse(ParseRequest::new(
            empty_source_id,
            Edition::new("2026"),
            tokens,
            Vec::new(),
        ));

        assert_eq!(output.diagnostics.len(), 1);
        assert_eq!(
            output.diagnostics[0].code,
            SyntaxDiagnosticCode::MalformedTypeExpression
        );
        assert_eq!(output.diagnostics[0].primary, close_range);
        let ast = output.ast.expect("empty bracket args should recover");
        let type_arguments =
            single_node(&ast, |kind| matches!(kind, SurfaceNodeKind::TypeArguments));
        assert!(type_arguments.children.iter().any(|id| {
            matches!(
                ast.node(*id).unwrap().kind,
                SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::MissingTypeExpression)
            )
        }));

        let trailing_source_id = source_id(104);
        let tokens = token_sequence(
            trailing_source_id,
            &[
                ("reserve", ParserTokenKind::ReservedWord),
                ("x", ParserTokenKind::Identifier),
                ("for", ParserTokenKind::ReservedWord),
                ("T", ParserTokenKind::UserSymbol),
                ("[", ParserTokenKind::ReservedSymbol),
                ("set", ParserTokenKind::ReservedWord),
                (",", ParserTokenKind::ReservedSymbol),
                ("]", ParserTokenKind::ReservedSymbol),
                (";", ParserTokenKind::ReservedSymbol),
            ],
        );
        let close_range = tokens[7].span;
        let output = parse(ParseRequest::new(
            trailing_source_id,
            Edition::new("2026"),
            tokens,
            Vec::new(),
        ));

        assert_eq!(output.diagnostics.len(), 1);
        assert_eq!(
            output.diagnostics[0].code,
            SyntaxDiagnosticCode::MalformedTypeExpression
        );
        assert_eq!(output.diagnostics[0].primary, close_range);
        let ast = output
            .ast
            .expect("trailing comma bracket args should recover");
        let type_arguments =
            single_node(&ast, |kind| matches!(kind, SurfaceNodeKind::TypeArguments));
        assert!(type_arguments.children.iter().any(|id| {
            matches!(
                ast.node(*id).unwrap().kind,
                SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::MissingTypeExpression)
            )
        }));
    }

    #[test]
    fn parser_diagnoses_missing_of_over_type_arguments_around_commas() {
        let leading_source_id = source_id(105);
        let tokens = token_sequence(
            leading_source_id,
            &[
                ("reserve", ParserTokenKind::ReservedWord),
                ("x", ParserTokenKind::Identifier),
                ("for", ParserTokenKind::ReservedWord),
                ("T", ParserTokenKind::UserSymbol),
                ("over", ParserTokenKind::ReservedWord),
                (",", ParserTokenKind::ReservedSymbol),
                ("c", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
            ],
        );
        let comma_range = tokens[5].span;
        let output = parse(ParseRequest::new(
            leading_source_id,
            Edition::new("2026"),
            tokens,
            Vec::new(),
        ));

        assert_eq!(output.diagnostics.len(), 1);
        assert_eq!(
            output.diagnostics[0].code,
            SyntaxDiagnosticCode::MalformedTermExpression
        );
        assert_eq!(output.diagnostics[0].primary, comma_range);
        let ast = output.ast.expect("leading missing term should recover");
        let type_arguments =
            single_node(&ast, |kind| matches!(kind, SurfaceNodeKind::TypeArguments));
        assert!(type_arguments.children.iter().any(|id| {
            matches!(
                ast.node(*id).unwrap().kind,
                SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::MissingTerm)
            )
        }));

        let trailing_source_id = source_id(106);
        let tokens = token_sequence(
            trailing_source_id,
            &[
                ("reserve", ParserTokenKind::ReservedWord),
                ("x", ParserTokenKind::Identifier),
                ("for", ParserTokenKind::ReservedWord),
                ("T", ParserTokenKind::UserSymbol),
                ("of", ParserTokenKind::ReservedWord),
                ("a", ParserTokenKind::Identifier),
                (",", ParserTokenKind::ReservedSymbol),
                (";", ParserTokenKind::ReservedSymbol),
            ],
        );
        let semicolon_range = tokens[7].span;
        let output = parse(ParseRequest::new(
            trailing_source_id,
            Edition::new("2026"),
            tokens,
            Vec::new(),
        ));

        assert_eq!(output.diagnostics.len(), 1);
        assert_eq!(
            output.diagnostics[0].code,
            SyntaxDiagnosticCode::MalformedTermExpression
        );
        assert_eq!(output.diagnostics[0].primary, semicolon_range);
        let ast = output.ast.expect("trailing missing term should recover");
        let type_arguments =
            single_node(&ast, |kind| matches!(kind, SurfaceNodeKind::TypeArguments));
        assert!(type_arguments.children.iter().any(|id| {
            matches!(
                ast.node(*id).unwrap().kind,
                SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::MissingTerm)
            )
        }));
    }

    #[test]
    fn parser_diagnoses_missing_attribute_arguments_around_commas() {
        let empty_source_id = source_id(107);
        let tokens = token_sequence(
            empty_source_id,
            &[
                ("reserve", ParserTokenKind::ReservedWord),
                ("x", ParserTokenKind::Identifier),
                ("for", ParserTokenKind::ReservedWord),
                ("empty", ParserTokenKind::UserSymbol),
                ("(", ParserTokenKind::ReservedSymbol),
                (")", ParserTokenKind::ReservedSymbol),
                ("T", ParserTokenKind::UserSymbol),
                (";", ParserTokenKind::ReservedSymbol),
            ],
        );
        let close_range = tokens[5].span;
        let output = parse(ParseRequest::new(
            empty_source_id,
            Edition::new("2026"),
            tokens,
            Vec::new(),
        ));

        assert_eq!(output.diagnostics.len(), 1);
        assert_eq!(
            output.diagnostics[0].code,
            SyntaxDiagnosticCode::MalformedTermExpression
        );
        assert_eq!(output.diagnostics[0].primary, close_range);
        let ast = output.ast.expect("empty attribute args should recover");
        let attribute = single_node(&ast, |kind| matches!(kind, SurfaceNodeKind::AttributeRef));
        assert!(attribute.children.iter().any(|id| {
            matches!(
                ast.node(*id).unwrap().kind,
                SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::MissingTerm)
            )
        }));

        let leading_source_id = source_id(108);
        let tokens = token_sequence(
            leading_source_id,
            &[
                ("reserve", ParserTokenKind::ReservedWord),
                ("x", ParserTokenKind::Identifier),
                ("for", ParserTokenKind::ReservedWord),
                ("empty", ParserTokenKind::UserSymbol),
                ("(", ParserTokenKind::ReservedSymbol),
                (",", ParserTokenKind::ReservedSymbol),
                ("a", ParserTokenKind::Identifier),
                (")", ParserTokenKind::ReservedSymbol),
                ("T", ParserTokenKind::UserSymbol),
                (";", ParserTokenKind::ReservedSymbol),
            ],
        );
        let comma_range = tokens[5].span;
        let output = parse(ParseRequest::new(
            leading_source_id,
            Edition::new("2026"),
            tokens,
            Vec::new(),
        ));

        assert_eq!(output.diagnostics.len(), 1);
        assert_eq!(
            output.diagnostics[0].code,
            SyntaxDiagnosticCode::MalformedTermExpression
        );
        assert_eq!(output.diagnostics[0].primary, comma_range);

        let trailing_source_id = source_id(109);
        let tokens = token_sequence(
            trailing_source_id,
            &[
                ("reserve", ParserTokenKind::ReservedWord),
                ("x", ParserTokenKind::Identifier),
                ("for", ParserTokenKind::ReservedWord),
                ("empty", ParserTokenKind::UserSymbol),
                ("(", ParserTokenKind::ReservedSymbol),
                ("a", ParserTokenKind::Identifier),
                (",", ParserTokenKind::ReservedSymbol),
                (")", ParserTokenKind::ReservedSymbol),
                ("T", ParserTokenKind::UserSymbol),
                (";", ParserTokenKind::ReservedSymbol),
            ],
        );
        let close_range = tokens[7].span;
        let output = parse(ParseRequest::new(
            trailing_source_id,
            Edition::new("2026"),
            tokens,
            Vec::new(),
        ));

        assert_eq!(output.diagnostics.len(), 1);
        assert_eq!(
            output.diagnostics[0].code,
            SyntaxDiagnosticCode::MalformedTermExpression
        );
        assert_eq!(output.diagnostics[0].primary, close_range);
    }

    #[test]
    fn parser_recovers_missing_primary_term_delimiters() {
        for (case, opener, expected_recovery_parent) in [
            (
                113,
                ("(", ParserTokenKind::ReservedSymbol),
                SurfaceNodeKind::ParenthesizedTerm,
            ),
            (
                114,
                ("[", ParserTokenKind::ReservedSymbol),
                SurfaceNodeKind::ApplicationTerm,
            ),
            (
                115,
                ("{", ParserTokenKind::ReservedSymbol),
                SurfaceNodeKind::SetEnumeration,
            ),
        ] {
            let source_id = source_id(case);
            let tokens = token_sequence(
                source_id,
                &[
                    ("reserve", ParserTokenKind::ReservedWord),
                    ("x", ParserTokenKind::Identifier),
                    ("for", ParserTokenKind::ReservedWord),
                    ("T", ParserTokenKind::UserSymbol),
                    ("of", ParserTokenKind::ReservedWord),
                    opener,
                    ("a", ParserTokenKind::Identifier),
                    (";", ParserTokenKind::ReservedSymbol),
                ],
            );
            let semicolon_range = tokens[7].span;
            let output = parse(ParseRequest::new(
                source_id,
                Edition::new("2026"),
                tokens,
                Vec::new(),
            ));

            assert_eq!(output.diagnostics.len(), 1);
            assert_eq!(
                output.diagnostics[0].code,
                SyntaxDiagnosticCode::MalformedTermExpression
            );
            assert_eq!(output.diagnostics[0].primary, semicolon_range);
            let ast = output.ast.expect("missing term delimiter should recover");
            let parent = single_node(&ast, |kind| *kind == expected_recovery_parent);
            assert!(parent.children.iter().any(|id| {
                matches!(
                    ast.node(*id).unwrap().kind,
                    SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::UnmatchedOpeningDelimiter)
                )
            }));
        }
    }

    #[test]
    fn parser_recovers_trailing_bracket_type_argument_before_boundary() {
        let source_id = source_id(110);
        let tokens = token_sequence(
            source_id,
            &[
                ("reserve", ParserTokenKind::ReservedWord),
                ("x", ParserTokenKind::Identifier),
                ("for", ParserTokenKind::ReservedWord),
                ("T", ParserTokenKind::UserSymbol),
                ("[", ParserTokenKind::ReservedSymbol),
                ("set", ParserTokenKind::ReservedWord),
                (",", ParserTokenKind::ReservedSymbol),
                (";", ParserTokenKind::ReservedSymbol),
            ],
        );
        let semicolon_range = tokens[7].span;
        let output = parse(ParseRequest::new(
            source_id,
            Edition::new("2026"),
            tokens,
            Vec::new(),
        ));

        assert_eq!(output.diagnostics.len(), 1);
        assert_eq!(
            output.diagnostics[0].code,
            SyntaxDiagnosticCode::MalformedTypeExpression
        );
        assert_eq!(output.diagnostics[0].primary, semicolon_range);
        let ast = output
            .ast
            .expect("trailing bracket arg before boundary should recover");
        let type_arguments =
            single_node(&ast, |kind| matches!(kind, SurfaceNodeKind::TypeArguments));
        assert!(type_arguments.children.iter().any(|id| {
            matches!(
                ast.node(*id).unwrap().kind,
                SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::MissingTypeExpression)
            )
        }));
        assert!(type_arguments.children.iter().any(|id| {
            matches!(
                ast.node(*id).unwrap().kind,
                SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::UnmatchedOpeningDelimiter)
            )
        }));
    }

    #[test]
    fn parser_recovers_malformed_reserve_type_expression_tail() {
        let source_id = source_id(101);
        let tokens = token_sequence(
            source_id,
            &[
                ("reserve", ParserTokenKind::ReservedWord),
                ("x", ParserTokenKind::Identifier),
                ("for", ParserTokenKind::ReservedWord),
                ("non", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
            ],
        );
        let non_range = tokens[3].span;
        let output = parse(ParseRequest::new(
            source_id,
            Edition::new("2026"),
            tokens,
            Vec::new(),
        ));

        assert_eq!(output.diagnostics.len(), 1);
        assert_eq!(
            output.diagnostics[0].code,
            SyntaxDiagnosticCode::MalformedTypeExpression
        );
        assert_eq!(output.diagnostics[0].primary, non_range);
        let ast = output.ast.expect("malformed reserve type should recover");
        let segment = single_node(&ast, |kind| matches!(kind, SurfaceNodeKind::ReserveSegment));
        assert!(segment.children.iter().any(|id| {
            matches!(
                ast.node(*id).unwrap().kind,
                SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::SkippedToken)
            )
        }));
    }

    #[test]
    fn parser_groups_active_operator_terms_before_qua() {
        let source_id = source_id(160);
        let tokens = token_sequence(
            source_id,
            &[
                ("reserve", ParserTokenKind::ReservedWord),
                ("x", ParserTokenKind::Identifier),
                ("for", ParserTokenKind::ReservedWord),
                ("T", ParserTokenKind::UserSymbol),
                ("of", ParserTokenKind::ReservedWord),
                ("~", ParserTokenKind::UserSymbol),
                ("f", ParserTokenKind::Identifier),
                ("(", ParserTokenKind::ReservedSymbol),
                ("a", ParserTokenKind::Identifier),
                (")", ParserTokenKind::ReservedSymbol),
                ("!", ParserTokenKind::UserSymbol),
                ("++", ParserTokenKind::UserSymbol),
                ("p", ParserTokenKind::Identifier),
                (".", ParserTokenKind::ReservedSymbol),
                ("x", ParserTokenKind::Identifier),
                ("**", ParserTokenKind::UserSymbol),
                ("b", ParserTokenKind::Identifier),
                ("qua", ParserTokenKind::ReservedWord),
                ("R", ParserTokenKind::UserSymbol),
                (";", ParserTokenKind::ReservedSymbol),
            ],
        );
        let output = parse(ParseRequest::new(
            source_id,
            Edition::new("2026"),
            tokens,
            operator_fixture_fixities(),
        ));

        assert!(
            output.diagnostics.is_empty(),
            "unexpected diagnostics: {:?}",
            output.diagnostics
        );
        let ast = output.ast.expect("operator term should parse");
        assert_eq!(
            ast.nodes()
                .iter()
                .filter(|node| matches!(node.kind, SurfaceNodeKind::PrefixExpression(_)))
                .count(),
            1
        );
        assert_eq!(
            ast.nodes()
                .iter()
                .filter(|node| matches!(node.kind, SurfaceNodeKind::PostfixExpression(_)))
                .count(),
            1
        );
        assert_eq!(
            ast.nodes()
                .iter()
                .filter(|node| matches!(node.kind, SurfaceNodeKind::InfixExpression(_)))
                .count(),
            2
        );

        let qua = single_node(&ast, |kind| matches!(kind, SurfaceNodeKind::QuaExpression));
        assert!(direct_child_has_kind(&ast, qua, |kind| {
            matches!(kind, SurfaceNodeKind::InfixExpression(operator) if operator.spelling.as_ref() == "++")
        }));

        let prefix = single_node(&ast, |kind| {
            matches!(kind, SurfaceNodeKind::PrefixExpression(_))
        });
        let SurfaceNodeKind::PrefixExpression(prefix_operator) = &prefix.kind else {
            panic!("expected prefix expression");
        };
        assert_eq!(prefix_operator.spelling.as_ref(), "~");
        assert_eq!(prefix_operator.precedence, 70);
        assert!(direct_child_has_kind(&ast, prefix, |kind| {
            matches!(kind, SurfaceNodeKind::PostfixExpression(_))
        }));

        let postfix = single_node(&ast, |kind| {
            matches!(kind, SurfaceNodeKind::PostfixExpression(_))
        });
        let SurfaceNodeKind::PostfixExpression(postfix_operator) = &postfix.kind else {
            panic!("expected postfix expression");
        };
        assert_eq!(postfix_operator.spelling.as_ref(), "!");
        assert_eq!(postfix_operator.precedence, 90);
        assert!(direct_child_has_kind(&ast, postfix, |kind| {
            matches!(kind, SurfaceNodeKind::ApplicationTerm)
        }));

        assert!(ast.nodes().iter().any(|node| {
            matches!(
                &node.kind,
                SurfaceNodeKind::InfixExpression(operator)
                    if operator.spelling.as_ref() == "**"
                        && operator.associativity == SurfaceOperatorAssociativity::Right
                        && direct_child_has_kind(&ast, node, |kind| {
                            matches!(kind, SurfaceNodeKind::SelectorAccess)
                        })
            )
        }));
    }

    #[test]
    fn parser_diagnoses_non_associative_operator_terms() {
        let source_id = source_id(161);
        let tokens = token_sequence(
            source_id,
            &[
                ("reserve", ParserTokenKind::ReservedWord),
                ("x", ParserTokenKind::Identifier),
                ("for", ParserTokenKind::ReservedWord),
                ("T", ParserTokenKind::UserSymbol),
                ("of", ParserTokenKind::ReservedWord),
                ("a", ParserTokenKind::Identifier),
                ("%%", ParserTokenKind::UserSymbol),
                ("b", ParserTokenKind::Identifier),
                ("%%", ParserTokenKind::UserSymbol),
                ("c", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
            ],
        );
        let second_operator = tokens[8].span;
        let output = parse(ParseRequest::new(
            source_id,
            Edition::new("2026"),
            tokens,
            operator_fixture_fixities(),
        ));

        assert_eq!(output.diagnostics.len(), 1);
        assert_eq!(
            output.diagnostics[0].code,
            SyntaxDiagnosticCode::NonAssociativeOperatorChain
        );
        assert_eq!(output.diagnostics[0].primary, second_operator);
        let ast = output
            .ast
            .expect("non-associative chain should keep an AST");
        assert_eq!(
            ast.nodes()
                .iter()
                .filter(|node| {
                    matches!(
                        &node.kind,
                        SurfaceNodeKind::InfixExpression(operator)
                            if operator.spelling.as_ref() == "%%"
                    )
                })
                .count(),
            2
        );
    }

    #[test]
    fn parser_diagnoses_dangling_infix_operator_terms_once() {
        let source_id = source_id(162);
        let tokens = token_sequence(
            source_id,
            &[
                ("reserve", ParserTokenKind::ReservedWord),
                ("x", ParserTokenKind::Identifier),
                ("for", ParserTokenKind::ReservedWord),
                ("T", ParserTokenKind::UserSymbol),
                ("of", ParserTokenKind::ReservedWord),
                ("a", ParserTokenKind::Identifier),
                ("++", ParserTokenKind::UserSymbol),
                (";", ParserTokenKind::ReservedSymbol),
            ],
        );
        let operator_range = tokens[6].span;
        let output = parse(ParseRequest::new(
            source_id,
            Edition::new("2026"),
            tokens,
            operator_fixture_fixities(),
        ));

        assert_eq!(output.diagnostics.len(), 1);
        assert_eq!(
            output.diagnostics[0].code,
            SyntaxDiagnosticCode::DanglingOperator
        );
        assert_eq!(output.diagnostics[0].primary, operator_range);
        let ast = output.ast.expect("dangling operator should keep an AST");
        single_node(&ast, |kind| matches!(kind, SurfaceNodeKind::TermExpression));
        assert!(
            !ast.nodes()
                .iter()
                .any(|node| matches!(node.kind, SurfaceNodeKind::InfixExpression(_)))
        );
    }

    #[test]
    fn parser_diagnoses_dangling_prefix_operator_terms() {
        let source_id = source_id(163);
        let tokens = token_sequence(
            source_id,
            &[
                ("reserve", ParserTokenKind::ReservedWord),
                ("x", ParserTokenKind::Identifier),
                ("for", ParserTokenKind::ReservedWord),
                ("T", ParserTokenKind::UserSymbol),
                ("of", ParserTokenKind::ReservedWord),
                ("~", ParserTokenKind::UserSymbol),
                (";", ParserTokenKind::ReservedSymbol),
            ],
        );
        let operator_range = tokens[5].span;
        let output = parse(ParseRequest::new(
            source_id,
            Edition::new("2026"),
            tokens,
            operator_fixture_fixities(),
        ));

        assert_eq!(output.diagnostics.len(), 1);
        assert_eq!(
            output.diagnostics[0].code,
            SyntaxDiagnosticCode::DanglingOperator
        );
        assert_eq!(
            output.diagnostics[0].message.as_ref(),
            "operator has no operand"
        );
        assert_eq!(output.diagnostics[0].primary, operator_range);
        let ast = output.ast.expect("dangling prefix should keep an AST");
        let prefix = single_node(&ast, |kind| {
            matches!(kind, SurfaceNodeKind::PrefixExpression(_))
        });
        assert!(direct_child_has_kind(&ast, prefix, |kind| {
            matches!(
                kind,
                SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::MissingTerm)
            )
        }));
    }

    #[test]
    fn parser_considers_same_spelling_infix_when_postfix_binding_is_too_low() {
        let source_id = source_id(164);
        let tokens = token_sequence(
            source_id,
            &[
                ("reserve", ParserTokenKind::ReservedWord),
                ("x", ParserTokenKind::Identifier),
                ("for", ParserTokenKind::ReservedWord),
                ("T", ParserTokenKind::UserSymbol),
                ("of", ParserTokenKind::ReservedWord),
                ("a", ParserTokenKind::Identifier),
                ("++", ParserTokenKind::UserSymbol),
                ("b", ParserTokenKind::Identifier),
                ("@", ParserTokenKind::UserSymbol),
                ("c", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
            ],
        );
        let output = parse(ParseRequest::new(
            source_id,
            Edition::new("2026"),
            tokens,
            vec![
                OperatorFixityEntry::infix("++", 10, OperatorAssociativity::Left),
                OperatorFixityEntry::postfix("@", 5),
                OperatorFixityEntry::infix("@", 20, OperatorAssociativity::Left),
            ],
        ));

        assert!(
            output.diagnostics.is_empty(),
            "unexpected diagnostics: {:?}",
            output.diagnostics
        );
        let ast = output
            .ast
            .expect("same-spelling postfix/infix case should parse");
        assert!(ast.nodes().iter().any(|node| {
            matches!(
                &node.kind,
                SurfaceNodeKind::InfixExpression(operator)
                    if operator.spelling.as_ref() == "@"
                        && operator.precedence == 20
                        && operator.associativity == SurfaceOperatorAssociativity::Left
            )
        }));
        assert!(!ast.nodes().iter().any(|node| {
            matches!(
                &node.kind,
                SurfaceNodeKind::PostfixExpression(operator)
                    if operator.spelling.as_ref() == "@"
            )
        }));
    }

    #[test]
    fn parser_prefers_same_spelling_infix_when_right_operand_is_present() {
        let source_id = source_id(165);
        let tokens = token_sequence(
            source_id,
            &[
                ("reserve", ParserTokenKind::ReservedWord),
                ("x", ParserTokenKind::Identifier),
                ("for", ParserTokenKind::ReservedWord),
                ("T", ParserTokenKind::UserSymbol),
                ("of", ParserTokenKind::ReservedWord),
                ("a", ParserTokenKind::Identifier),
                ("@", ParserTokenKind::UserSymbol),
                ("b", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
            ],
        );
        let output = parse(ParseRequest::new(
            source_id,
            Edition::new("2026"),
            tokens,
            vec![
                OperatorFixityEntry::postfix("@", 90),
                OperatorFixityEntry::infix("@", 20, OperatorAssociativity::Left),
            ],
        ));

        assert!(
            output.diagnostics.is_empty(),
            "unexpected diagnostics: {:?}",
            output.diagnostics
        );
        let ast = output
            .ast
            .expect("same-spelling infix with right operand should parse");
        assert!(ast.nodes().iter().any(|node| {
            matches!(
                &node.kind,
                SurfaceNodeKind::InfixExpression(operator)
                    if operator.spelling.as_ref() == "@"
                        && operator.precedence == 20
                        && operator.associativity == SurfaceOperatorAssociativity::Left
            )
        }));
        assert!(!ast.nodes().iter().any(|node| {
            matches!(
                &node.kind,
                SurfaceNodeKind::PostfixExpression(operator)
                    if operator.spelling.as_ref() == "@"
            )
        }));
    }

    #[test]
    fn legacy_token_preservation_streams_without_item_starts_stay_diagnostic_free() {
        let source_id = source_id(44);
        let output = parse(ParseRequest::new(
            source_id,
            Edition::new("2026"),
            vec![
                token(source_id, ParserTokenKind::Identifier, "alpha", 0, 5),
                token(source_id, ParserTokenKind::ReservedSymbol, ";", 5, 6),
                token(source_id, ParserTokenKind::Identifier, "beta", 7, 11),
            ],
            Vec::new(),
        ));

        assert!(output.diagnostics.is_empty());
        let ast = output.ast.expect("legacy stream should keep an AST");
        let item_list = single_node(&ast, |kind| matches!(kind, SurfaceNodeKind::ItemList));
        assert!(item_list.children.is_empty());
    }

    #[test]
    fn parser_parses_atomic_formula_placeholder_payloads() {
        let source_id = source_id(45);
        let output = parse(ParseRequest::new(
            source_id,
            Edition::new("2026"),
            token_sequence(
                source_id,
                &[
                    ("theorem", ParserTokenKind::ReservedWord),
                    ("I", ParserTokenKind::Identifier),
                    (":", ParserTokenKind::ReservedSymbol),
                    ("x", ParserTokenKind::Identifier),
                    ("in", ParserTokenKind::ReservedWord),
                    ("X", ParserTokenKind::Identifier),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("theorem", ParserTokenKind::ReservedWord),
                    ("E", ParserTokenKind::Identifier),
                    (":", ParserTokenKind::ReservedSymbol),
                    ("x", ParserTokenKind::Identifier),
                    ("=", ParserTokenKind::ReservedSymbol),
                    ("y", ParserTokenKind::Identifier),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("theorem", ParserTokenKind::ReservedWord),
                    ("N", ParserTokenKind::Identifier),
                    (":", ParserTokenKind::ReservedSymbol),
                    ("x", ParserTokenKind::Identifier),
                    ("<>", ParserTokenKind::ReservedSymbol),
                    ("y", ParserTokenKind::Identifier),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("theorem", ParserTokenKind::ReservedWord),
                    ("A", ParserTokenKind::Identifier),
                    (":", ParserTokenKind::ReservedSymbol),
                    ("x", ParserTokenKind::Identifier),
                    ("is", ParserTokenKind::ReservedWord),
                    ("non", ParserTokenKind::ReservedWord),
                    ("empty", ParserTokenKind::UserSymbol),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("theorem", ParserTokenKind::ReservedWord),
                    ("BA", ParserTokenKind::Identifier),
                    (":", ParserTokenKind::ReservedSymbol),
                    ("x", ParserTokenKind::Identifier),
                    ("is", ParserTokenKind::ReservedWord),
                    ("empty", ParserTokenKind::UserSymbol),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("theorem", ParserTokenKind::ReservedWord),
                    ("T", ParserTokenKind::Identifier),
                    (":", ParserTokenKind::ReservedSymbol),
                    ("x", ParserTokenKind::Identifier),
                    ("is", ParserTokenKind::ReservedWord),
                    ("T", ParserTokenKind::UserSymbol),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("theorem", ParserTokenKind::ReservedWord),
                    ("IN", ParserTokenKind::Identifier),
                    (":", ParserTokenKind::ReservedSymbol),
                    ("x", ParserTokenKind::Identifier),
                    ("is", ParserTokenKind::ReservedWord),
                    ("not", ParserTokenKind::ReservedWord),
                    ("T", ParserTokenKind::UserSymbol),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("theorem", ParserTokenKind::ReservedWord),
                    ("P", ParserTokenKind::Identifier),
                    (":", ParserTokenKind::ReservedSymbol),
                    ("x", ParserTokenKind::Identifier),
                    ("divides", ParserTokenKind::UserSymbol),
                    ("y", ParserTokenKind::Identifier),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("theorem", ParserTokenKind::ReservedWord),
                    ("PL", ParserTokenKind::Identifier),
                    (":", ParserTokenKind::ReservedSymbol),
                    ("x", ParserTokenKind::Identifier),
                    (",", ParserTokenKind::ReservedSymbol),
                    ("y", ParserTokenKind::Identifier),
                    ("divides", ParserTokenKind::UserSymbol),
                    ("z", ParserTokenKind::Identifier),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("theorem", ParserTokenKind::ReservedWord),
                    ("DN", ParserTokenKind::Identifier),
                    (":", ParserTokenKind::ReservedSymbol),
                    ("x", ParserTokenKind::Identifier),
                    ("do", ParserTokenKind::ReservedWord),
                    ("not", ParserTokenKind::ReservedWord),
                    ("divides", ParserTokenKind::UserSymbol),
                    ("y", ParserTokenKind::Identifier),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("theorem", ParserTokenKind::ReservedWord),
                    ("CH", ParserTokenKind::Identifier),
                    (":", ParserTokenKind::ReservedSymbol),
                    ("x", ParserTokenKind::Identifier),
                    ("divides", ParserTokenKind::UserSymbol),
                    ("y", ParserTokenKind::Identifier),
                    ("does", ParserTokenKind::ReservedWord),
                    ("not", ParserTokenKind::ReservedWord),
                    ("divides", ParserTokenKind::UserSymbol),
                    ("z", ParserTokenKind::Identifier),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("lemma", ParserTokenKind::ReservedWord),
                    ("L", ParserTokenKind::Identifier),
                    (":", ParserTokenKind::ReservedSymbol),
                    ("x", ParserTokenKind::Identifier),
                    ("=", ParserTokenKind::ReservedSymbol),
                    ("z", ParserTokenKind::Identifier),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("theorem", ParserTokenKind::ReservedWord),
                    ("C", ParserTokenKind::Identifier),
                    (":", ParserTokenKind::ReservedSymbol),
                    ("IsSmall", ParserTokenKind::Identifier),
                    ("(", ParserTokenKind::ReservedSymbol),
                    ("x", ParserTokenKind::Identifier),
                    (")", ParserTokenKind::ReservedSymbol),
                    (";", ParserTokenKind::ReservedSymbol),
                ],
            ),
            Vec::new(),
        ));

        assert!(
            output.diagnostics.is_empty(),
            "unexpected diagnostics: {:?}",
            output.diagnostics
        );
        let ast = output.ast.expect("atomic formulas should parse");
        assert_eq!(
            count_nodes(&ast, |kind| matches!(
                kind,
                SurfaceNodeKind::FormulaExpression
            )),
            13
        );
        assert_eq!(
            count_nodes(&ast, |kind| {
                matches!(kind, SurfaceNodeKind::BuiltinPredicateApplication)
            }),
            4
        );
        assert_eq!(
            count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::IsAssertion)),
            4
        );
        assert_eq!(
            count_nodes(&ast, |kind| matches!(
                kind,
                SurfaceNodeKind::AttributeTestChain
            )),
            2
        );
        assert_eq!(
            count_nodes(&ast, |kind| {
                matches!(kind, SurfaceNodeKind::PredicateApplication)
            }),
            4
        );
        let predicate_segments = ast
            .nodes()
            .iter()
            .filter(|node| matches!(node.kind, SurfaceNodeKind::PredicateSegment))
            .collect::<Vec<_>>();
        assert_eq!(predicate_segments.len(), 5);
        assert!(
            predicate_segments.iter().any(|segment| {
                segment.children.iter().any(|child| {
                    ast.node(*child)
                        .and_then(mizar_syntax::SurfaceNode::token_text)
                        == Some("does")
                })
            }),
            "predicate chain segment should preserve `does not` negation tokens"
        );
        assert!(
            predicate_segments.iter().any(|segment| {
                segment.children.iter().any(|child| {
                    ast.node(*child)
                        .and_then(mizar_syntax::SurfaceNode::token_text)
                        == Some("do")
                })
            }),
            "predicate segment should preserve `do not` negation tokens"
        );
        assert_eq!(
            count_nodes(&ast, |kind| {
                matches!(kind, SurfaceNodeKind::InlinePredicateApplication)
            }),
            1
        );

        let item_list = single_node(&ast, |kind| matches!(kind, SurfaceNodeKind::ItemList));
        let first_item = ast.node(item_list.children[0]).unwrap();
        assert!(matches!(first_item.kind, SurfaceNodeKind::TheoremItem));
        let formula_id = first_item
            .children
            .iter()
            .copied()
            .find(|child| {
                ast.node(*child)
                    .is_some_and(|node| matches!(node.kind, SurfaceNodeKind::FormulaExpression))
            })
            .expect("theorem item should own a concrete FormulaExpression child");
        let formula = ast.node(formula_id).unwrap();
        assert_eq!(formula.children.len(), 1);
        assert!(matches!(
            ast.node(formula.children[0]).unwrap().kind,
            SurfaceNodeKind::BuiltinPredicateApplication
        ));
        assert!(
            item_list.children.iter().any(|item| {
                let Some(item) = ast.node(*item) else {
                    return false;
                };
                item.children.iter().any(|child| {
                    ast.node(*child)
                        .and_then(mizar_syntax::SurfaceNode::token_text)
                        == Some("lemma")
                }) && direct_child_has_kind(&ast, item, |kind| {
                    matches!(kind, SurfaceNodeKind::FormulaExpression)
                })
            }),
            "lemma item should host a concrete formula payload"
        );
    }

    #[test]
    fn parser_parses_formula_connectives_constants_and_grouping() {
        let source_id = source_id(49);
        let output = parse(ParseRequest::new(
            source_id,
            Edition::new("2026"),
            token_sequence(
                source_id,
                &[
                    ("theorem", ParserTokenKind::ReservedWord),
                    ("Thesis", ParserTokenKind::Identifier),
                    (":", ParserTokenKind::ReservedSymbol),
                    ("thesis", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("theorem", ParserTokenKind::ReservedWord),
                    ("Conj", ParserTokenKind::Identifier),
                    (":", ParserTokenKind::ReservedSymbol),
                    ("x", ParserTokenKind::Identifier),
                    ("=", ParserTokenKind::ReservedSymbol),
                    ("y", ParserTokenKind::Identifier),
                    ("&", ParserTokenKind::ReservedSymbol),
                    ("z", ParserTokenKind::Identifier),
                    ("=", ParserTokenKind::ReservedSymbol),
                    ("w", ParserTokenKind::Identifier),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("theorem", ParserTokenKind::ReservedWord),
                    ("Repeat", ParserTokenKind::Identifier),
                    (":", ParserTokenKind::ReservedSymbol),
                    ("x", ParserTokenKind::Identifier),
                    ("=", ParserTokenKind::ReservedSymbol),
                    ("y", ParserTokenKind::Identifier),
                    ("&", ParserTokenKind::ReservedSymbol),
                    ("...", ParserTokenKind::ReservedSymbol),
                    ("&", ParserTokenKind::ReservedSymbol),
                    ("z", ParserTokenKind::Identifier),
                    ("=", ParserTokenKind::ReservedSymbol),
                    ("w", ParserTokenKind::Identifier),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("theorem", ParserTokenKind::ReservedWord),
                    ("OrImplies", ParserTokenKind::Identifier),
                    (":", ParserTokenKind::ReservedSymbol),
                    ("x", ParserTokenKind::Identifier),
                    ("=", ParserTokenKind::ReservedSymbol),
                    ("y", ParserTokenKind::Identifier),
                    ("or", ParserTokenKind::ReservedWord),
                    ("z", ParserTokenKind::Identifier),
                    ("=", ParserTokenKind::ReservedSymbol),
                    ("w", ParserTokenKind::Identifier),
                    ("implies", ParserTokenKind::ReservedWord),
                    ("thesis", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("theorem", ParserTokenKind::ReservedWord),
                    ("IffGroup", ParserTokenKind::Identifier),
                    (":", ParserTokenKind::ReservedSymbol),
                    ("(", ParserTokenKind::ReservedSymbol),
                    ("x", ParserTokenKind::Identifier),
                    ("=", ParserTokenKind::ReservedSymbol),
                    ("y", ParserTokenKind::Identifier),
                    (")", ParserTokenKind::ReservedSymbol),
                    ("iff", ParserTokenKind::ReservedWord),
                    ("contradiction", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("theorem", ParserTokenKind::ReservedWord),
                    ("NotAnd", ParserTokenKind::Identifier),
                    (":", ParserTokenKind::ReservedSymbol),
                    ("not", ParserTokenKind::ReservedWord),
                    ("x", ParserTokenKind::Identifier),
                    ("=", ParserTokenKind::ReservedSymbol),
                    ("y", ParserTokenKind::Identifier),
                    ("&", ParserTokenKind::ReservedSymbol),
                    ("z", ParserTokenKind::Identifier),
                    ("=", ParserTokenKind::ReservedSymbol),
                    ("w", ParserTokenKind::Identifier),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("theorem", ParserTokenKind::ReservedWord),
                    ("RepeatOr", ParserTokenKind::Identifier),
                    (":", ParserTokenKind::ReservedSymbol),
                    ("x", ParserTokenKind::Identifier),
                    ("=", ParserTokenKind::ReservedSymbol),
                    ("y", ParserTokenKind::Identifier),
                    ("or", ParserTokenKind::ReservedWord),
                    ("...", ParserTokenKind::ReservedSymbol),
                    ("or", ParserTokenKind::ReservedWord),
                    ("z", ParserTokenKind::Identifier),
                    ("=", ParserTokenKind::ReservedSymbol),
                    ("w", ParserTokenKind::Identifier),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("theorem", ParserTokenKind::ReservedWord),
                    ("AndLeft", ParserTokenKind::Identifier),
                    (":", ParserTokenKind::ReservedSymbol),
                    ("x", ParserTokenKind::Identifier),
                    ("=", ParserTokenKind::ReservedSymbol),
                    ("y", ParserTokenKind::Identifier),
                    ("&", ParserTokenKind::ReservedSymbol),
                    ("z", ParserTokenKind::Identifier),
                    ("=", ParserTokenKind::ReservedSymbol),
                    ("w", ParserTokenKind::Identifier),
                    ("&", ParserTokenKind::ReservedSymbol),
                    ("u", ParserTokenKind::Identifier),
                    ("=", ParserTokenKind::ReservedSymbol),
                    ("v", ParserTokenKind::Identifier),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("theorem", ParserTokenKind::ReservedWord),
                    ("OrLeft", ParserTokenKind::Identifier),
                    (":", ParserTokenKind::ReservedSymbol),
                    ("x", ParserTokenKind::Identifier),
                    ("=", ParserTokenKind::ReservedSymbol),
                    ("y", ParserTokenKind::Identifier),
                    ("or", ParserTokenKind::ReservedWord),
                    ("z", ParserTokenKind::Identifier),
                    ("=", ParserTokenKind::ReservedSymbol),
                    ("w", ParserTokenKind::Identifier),
                    ("or", ParserTokenKind::ReservedWord),
                    ("u", ParserTokenKind::Identifier),
                    ("=", ParserTokenKind::ReservedSymbol),
                    ("v", ParserTokenKind::Identifier),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("theorem", ParserTokenKind::ReservedWord),
                    ("ImpliesRight", ParserTokenKind::Identifier),
                    (":", ParserTokenKind::ReservedSymbol),
                    ("x", ParserTokenKind::Identifier),
                    ("=", ParserTokenKind::ReservedSymbol),
                    ("y", ParserTokenKind::Identifier),
                    ("implies", ParserTokenKind::ReservedWord),
                    ("z", ParserTokenKind::Identifier),
                    ("=", ParserTokenKind::ReservedSymbol),
                    ("w", ParserTokenKind::Identifier),
                    ("implies", ParserTokenKind::ReservedWord),
                    ("thesis", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("theorem", ParserTokenKind::ReservedWord),
                    ("AndBeforeOr", ParserTokenKind::Identifier),
                    (":", ParserTokenKind::ReservedSymbol),
                    ("x", ParserTokenKind::Identifier),
                    ("=", ParserTokenKind::ReservedSymbol),
                    ("y", ParserTokenKind::Identifier),
                    ("&", ParserTokenKind::ReservedSymbol),
                    ("z", ParserTokenKind::Identifier),
                    ("=", ParserTokenKind::ReservedSymbol),
                    ("w", ParserTokenKind::Identifier),
                    ("or", ParserTokenKind::ReservedWord),
                    ("u", ParserTokenKind::Identifier),
                    ("=", ParserTokenKind::ReservedSymbol),
                    ("v", ParserTokenKind::Identifier),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("theorem", ParserTokenKind::ReservedWord),
                    ("ImpliesBeforeIff", ParserTokenKind::Identifier),
                    (":", ParserTokenKind::ReservedSymbol),
                    ("thesis", ParserTokenKind::ReservedWord),
                    ("implies", ParserTokenKind::ReservedWord),
                    ("contradiction", ParserTokenKind::ReservedWord),
                    ("iff", ParserTokenKind::ReservedWord),
                    ("thesis", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("theorem", ParserTokenKind::ReservedWord),
                    ("ParenOverride", ParserTokenKind::Identifier),
                    (":", ParserTokenKind::ReservedSymbol),
                    ("(", ParserTokenKind::ReservedSymbol),
                    ("x", ParserTokenKind::Identifier),
                    ("=", ParserTokenKind::ReservedSymbol),
                    ("y", ParserTokenKind::Identifier),
                    ("or", ParserTokenKind::ReservedWord),
                    ("z", ParserTokenKind::Identifier),
                    ("=", ParserTokenKind::ReservedSymbol),
                    ("w", ParserTokenKind::Identifier),
                    (")", ParserTokenKind::ReservedSymbol),
                    ("&", ParserTokenKind::ReservedSymbol),
                    ("u", ParserTokenKind::Identifier),
                    ("=", ParserTokenKind::ReservedSymbol),
                    ("v", ParserTokenKind::Identifier),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("theorem", ParserTokenKind::ReservedWord),
                    ("ImpliesQuantifier", ParserTokenKind::Identifier),
                    (":", ParserTokenKind::ReservedSymbol),
                    ("x", ParserTokenKind::Identifier),
                    ("=", ParserTokenKind::ReservedSymbol),
                    ("y", ParserTokenKind::Identifier),
                    ("implies", ParserTokenKind::ReservedWord),
                    ("for", ParserTokenKind::ReservedWord),
                    ("q", ParserTokenKind::Identifier),
                    ("being", ParserTokenKind::ReservedWord),
                    ("set", ParserTokenKind::ReservedWord),
                    ("holds", ParserTokenKind::ReservedWord),
                    ("thesis", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                ],
            ),
            Vec::new(),
        ));

        assert!(
            output.diagnostics.is_empty(),
            "task-14 formula forms should parse without diagnostics: {:?}",
            output.diagnostics
        );
        let ast = output.ast.expect("task-14 formulas should keep an AST");
        assert_eq!(
            count_nodes(&ast, |kind| matches!(
                kind,
                SurfaceNodeKind::FormulaExpression
            )),
            16
        );
        assert_eq!(
            count_nodes(&ast, |kind| matches!(
                kind,
                SurfaceNodeKind::BinaryFormula(_)
            )),
            20
        );
        assert_eq!(
            count_nodes(&ast, |kind| matches!(
                kind,
                SurfaceNodeKind::PrefixFormula(_)
            )),
            1
        );
        assert_eq!(
            count_nodes(&ast, |kind| {
                matches!(kind, SurfaceNodeKind::ParenthesizedFormula)
            }),
            2
        );
        for connective in [
            SurfaceFormulaConnective::And,
            SurfaceFormulaConnective::Or,
            SurfaceFormulaConnective::Implies,
            SurfaceFormulaConnective::Iff,
        ] {
            assert!(
                ast.nodes().iter().any(|node| {
                    matches!(
                        node.kind,
                        SurfaceNodeKind::BinaryFormula(operator)
                            if operator.connective == connective
                    )
                }),
                "formula AST should contain {connective:?}"
            );
        }
        assert!(ast.nodes().iter().any(|node| {
            matches!(
                node.kind,
                SurfaceNodeKind::BinaryFormula(operator)
                    if operator.connective == SurfaceFormulaConnective::And && operator.repeated
            )
        }));
        assert!(ast.nodes().iter().any(|node| {
            matches!(
                node.kind,
                SurfaceNodeKind::BinaryFormula(operator)
                    if operator.connective == SurfaceFormulaConnective::Or && operator.repeated
            )
        }));
        assert!(ast.nodes().iter().any(|node| {
            matches!(
                node.kind,
                SurfaceNodeKind::FormulaConstant(SurfaceFormulaConstant::Thesis)
            )
        }));
        assert!(ast.nodes().iter().any(|node| {
            matches!(
                node.kind,
                SurfaceNodeKind::FormulaConstant(SurfaceFormulaConstant::Contradiction)
            )
        }));

        let or_implies = formula_root_for_label(&ast, "OrImplies");
        assert_binary_formula(or_implies, SurfaceFormulaConnective::Implies, false);
        let or_implies_children = structural_children(&ast, or_implies);
        assert_binary_formula(or_implies_children[0], SurfaceFormulaConnective::Or, false);
        assert_formula_constant(or_implies_children[1], SurfaceFormulaConstant::Thesis);

        let not_and = formula_root_for_label(&ast, "NotAnd");
        assert_binary_formula(not_and, SurfaceFormulaConnective::And, false);
        let not_and_children = structural_children(&ast, not_and);
        assert!(matches!(
            not_and_children[0].kind,
            SurfaceNodeKind::PrefixFormula(_)
        ));
        assert!(matches!(
            structural_children(&ast, not_and_children[0])[0].kind,
            SurfaceNodeKind::BuiltinPredicateApplication
        ));

        let iff_group = formula_root_for_label(&ast, "IffGroup");
        assert_binary_formula(iff_group, SurfaceFormulaConnective::Iff, false);
        let iff_children = structural_children(&ast, iff_group);
        assert!(matches!(
            iff_children[0].kind,
            SurfaceNodeKind::ParenthesizedFormula
        ));
        assert_formula_constant(iff_children[1], SurfaceFormulaConstant::Contradiction);

        let and_left = formula_root_for_label(&ast, "AndLeft");
        assert_binary_formula(and_left, SurfaceFormulaConnective::And, false);
        let and_children = structural_children(&ast, and_left);
        assert_binary_formula(and_children[0], SurfaceFormulaConnective::And, false);
        assert!(matches!(
            and_children[1].kind,
            SurfaceNodeKind::BuiltinPredicateApplication
        ));

        let or_left = formula_root_for_label(&ast, "OrLeft");
        assert_binary_formula(or_left, SurfaceFormulaConnective::Or, false);
        let or_children = structural_children(&ast, or_left);
        assert_binary_formula(or_children[0], SurfaceFormulaConnective::Or, false);
        assert!(matches!(
            or_children[1].kind,
            SurfaceNodeKind::BuiltinPredicateApplication
        ));

        let implies_right = formula_root_for_label(&ast, "ImpliesRight");
        assert_binary_formula(implies_right, SurfaceFormulaConnective::Implies, false);
        let implies_children = structural_children(&ast, implies_right);
        assert!(matches!(
            implies_children[0].kind,
            SurfaceNodeKind::BuiltinPredicateApplication
        ));
        assert_binary_formula(
            implies_children[1],
            SurfaceFormulaConnective::Implies,
            false,
        );

        let repeat = formula_root_for_label(&ast, "Repeat");
        assert_binary_formula(repeat, SurfaceFormulaConnective::And, true);
        assert_eq!(direct_token_texts(&ast, repeat), vec!["&", "...", "&"]);

        let repeat_or = formula_root_for_label(&ast, "RepeatOr");
        assert_binary_formula(repeat_or, SurfaceFormulaConnective::Or, true);
        assert_eq!(direct_token_texts(&ast, repeat_or), vec!["or", "...", "or"]);

        let and_before_or = formula_root_for_label(&ast, "AndBeforeOr");
        assert_binary_formula(and_before_or, SurfaceFormulaConnective::Or, false);
        let and_before_or_children = structural_children(&ast, and_before_or);
        assert_binary_formula(
            and_before_or_children[0],
            SurfaceFormulaConnective::And,
            false,
        );
        assert!(matches!(
            and_before_or_children[1].kind,
            SurfaceNodeKind::BuiltinPredicateApplication
        ));

        let implies_before_iff = formula_root_for_label(&ast, "ImpliesBeforeIff");
        assert_binary_formula(implies_before_iff, SurfaceFormulaConnective::Iff, false);
        let implies_before_iff_children = structural_children(&ast, implies_before_iff);
        assert_binary_formula(
            implies_before_iff_children[0],
            SurfaceFormulaConnective::Implies,
            false,
        );
        assert_formula_constant(
            implies_before_iff_children[1],
            SurfaceFormulaConstant::Thesis,
        );

        let paren_override = formula_root_for_label(&ast, "ParenOverride");
        assert_binary_formula(paren_override, SurfaceFormulaConnective::And, false);
        let paren_override_children = structural_children(&ast, paren_override);
        assert!(matches!(
            paren_override_children[0].kind,
            SurfaceNodeKind::ParenthesizedFormula
        ));
        let parenthesized_children = structural_children(&ast, paren_override_children[0]);
        assert_eq!(parenthesized_children.len(), 1);
        assert!(matches!(
            parenthesized_children[0].kind,
            SurfaceNodeKind::FormulaExpression
        ));
        assert_binary_formula(
            structural_children(&ast, parenthesized_children[0])[0],
            SurfaceFormulaConnective::Or,
            false,
        );
        assert!(matches!(
            paren_override_children[1].kind,
            SurfaceNodeKind::BuiltinPredicateApplication
        ));

        let implies_quantifier = formula_root_for_label(&ast, "ImpliesQuantifier");
        assert_binary_formula(implies_quantifier, SurfaceFormulaConnective::Implies, false);
        let implies_quantifier_children = structural_children(&ast, implies_quantifier);
        assert!(matches!(
            implies_quantifier_children[0].kind,
            SurfaceNodeKind::BuiltinPredicateApplication
        ));
        assert_quantified_formula(
            implies_quantifier_children[1],
            SurfaceQuantifierKind::Universal,
        );
    }

    #[test]
    fn parser_parses_set_comprehension_terms() {
        let source_id = source_id(168);
        let output = parse(ParseRequest::new(
            source_id,
            Edition::new("2026"),
            token_sequence(
                source_id,
                &[
                    ("theorem", ParserTokenKind::ReservedWord),
                    ("OmittedCondition", ParserTokenKind::Identifier),
                    (":", ParserTokenKind::ReservedSymbol),
                    ("{", ParserTokenKind::ReservedSymbol),
                    ("f", ParserTokenKind::Identifier),
                    (".", ParserTokenKind::ReservedSymbol),
                    ("x", ParserTokenKind::Identifier),
                    ("where", ParserTokenKind::ReservedWord),
                    ("x", ParserTokenKind::Identifier),
                    ("is", ParserTokenKind::ReservedWord),
                    ("set", ParserTokenKind::ReservedWord),
                    ("}", ParserTokenKind::ReservedSymbol),
                    ("=", ParserTokenKind::ReservedSymbol),
                    ("y", ParserTokenKind::Identifier),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("theorem", ParserTokenKind::ReservedWord),
                    ("Conditioned", ParserTokenKind::Identifier),
                    (":", ParserTokenKind::ReservedSymbol),
                    ("{", ParserTokenKind::ReservedSymbol),
                    ("x", ParserTokenKind::Identifier),
                    ("where", ParserTokenKind::ReservedWord),
                    ("x", ParserTokenKind::Identifier),
                    ("is", ParserTokenKind::ReservedWord),
                    ("set", ParserTokenKind::ReservedWord),
                    (":", ParserTokenKind::ReservedSymbol),
                    ("thesis", ParserTokenKind::ReservedWord),
                    ("}", ParserTokenKind::ReservedSymbol),
                    ("=", ParserTokenKind::ReservedSymbol),
                    ("y", ParserTokenKind::Identifier),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("theorem", ParserTokenKind::ReservedWord),
                    ("MultipleGenerators", ParserTokenKind::Identifier),
                    (":", ParserTokenKind::ReservedSymbol),
                    ("{", ParserTokenKind::ReservedSymbol),
                    ("[", ParserTokenKind::ReservedSymbol),
                    ("x", ParserTokenKind::Identifier),
                    (",", ParserTokenKind::ReservedSymbol),
                    ("y", ParserTokenKind::Identifier),
                    ("]", ParserTokenKind::ReservedSymbol),
                    ("where", ParserTokenKind::ReservedWord),
                    ("x", ParserTokenKind::Identifier),
                    ("is", ParserTokenKind::ReservedWord),
                    ("set", ParserTokenKind::ReservedWord),
                    (",", ParserTokenKind::ReservedSymbol),
                    ("y", ParserTokenKind::Identifier),
                    ("is", ParserTokenKind::ReservedWord),
                    ("set", ParserTokenKind::ReservedWord),
                    (":", ParserTokenKind::ReservedSymbol),
                    ("thesis", ParserTokenKind::ReservedWord),
                    ("}", ParserTokenKind::ReservedSymbol),
                    ("=", ParserTokenKind::ReservedSymbol),
                    ("p", ParserTokenKind::Identifier),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("theorem", ParserTokenKind::ReservedWord),
                    ("OperatorMapper", ParserTokenKind::Identifier),
                    (":", ParserTokenKind::ReservedSymbol),
                    ("{", ParserTokenKind::ReservedSymbol),
                    ("a", ParserTokenKind::Identifier),
                    ("++", ParserTokenKind::UserSymbol),
                    ("b", ParserTokenKind::Identifier),
                    ("where", ParserTokenKind::ReservedWord),
                    ("x", ParserTokenKind::Identifier),
                    ("is", ParserTokenKind::ReservedWord),
                    ("set", ParserTokenKind::ReservedWord),
                    (":", ParserTokenKind::ReservedSymbol),
                    ("thesis", ParserTokenKind::ReservedWord),
                    ("}", ParserTokenKind::ReservedSymbol),
                    ("=", ParserTokenKind::ReservedSymbol),
                    ("q", ParserTokenKind::Identifier),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("theorem", ParserTokenKind::ReservedWord),
                    ("Nested", ParserTokenKind::Identifier),
                    (":", ParserTokenKind::ReservedSymbol),
                    ("{", ParserTokenKind::ReservedSymbol),
                    ("{", ParserTokenKind::ReservedSymbol),
                    ("y", ParserTokenKind::Identifier),
                    ("where", ParserTokenKind::ReservedWord),
                    ("y", ParserTokenKind::Identifier),
                    ("is", ParserTokenKind::ReservedWord),
                    ("set", ParserTokenKind::ReservedWord),
                    (":", ParserTokenKind::ReservedSymbol),
                    ("thesis", ParserTokenKind::ReservedWord),
                    ("}", ParserTokenKind::ReservedSymbol),
                    ("where", ParserTokenKind::ReservedWord),
                    ("x", ParserTokenKind::Identifier),
                    ("is", ParserTokenKind::ReservedWord),
                    ("set", ParserTokenKind::ReservedWord),
                    (":", ParserTokenKind::ReservedSymbol),
                    ("thesis", ParserTokenKind::ReservedWord),
                    ("}", ParserTokenKind::ReservedSymbol),
                    ("=", ParserTokenKind::ReservedSymbol),
                    ("z", ParserTokenKind::Identifier),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("theorem", ParserTokenKind::ReservedWord),
                    ("Enumeration", ParserTokenKind::Identifier),
                    (":", ParserTokenKind::ReservedSymbol),
                    ("{", ParserTokenKind::ReservedSymbol),
                    ("x", ParserTokenKind::Identifier),
                    (",", ParserTokenKind::ReservedSymbol),
                    ("y", ParserTokenKind::Identifier),
                    ("}", ParserTokenKind::ReservedSymbol),
                    ("=", ParserTokenKind::ReservedSymbol),
                    ("z", ParserTokenKind::Identifier),
                    (";", ParserTokenKind::ReservedSymbol),
                ],
            ),
            operator_fixture_fixities(),
        ));

        assert!(
            output.diagnostics.is_empty(),
            "set-comprehension terms should parse without diagnostics: {:?}",
            output.diagnostics
        );
        let ast = output
            .ast
            .expect("set-comprehension terms should keep an AST");
        assert_eq!(
            count_nodes(&ast, |kind| matches!(
                kind,
                SurfaceNodeKind::SetComprehension
            )),
            6
        );
        assert_eq!(
            count_nodes(&ast, |kind| matches!(
                kind,
                SurfaceNodeKind::ComprehensionVariableSegment
            )),
            7
        );
        assert_eq!(
            count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::SetEnumeration)),
            1
        );

        let omitted = set_comprehension_term_for_label(&ast, "OmittedCondition");
        assert_eq!(direct_token_texts(&ast, omitted), vec!["{", "where", "}"]);
        assert!(
            subtree_contains_kind(&ast, set_comprehension_mapper(&ast, omitted), |kind| {
                matches!(kind, SurfaceNodeKind::SelectorAccess)
            }),
            "selector mapper should preserve the existing term surface"
        );
        assert_eq!(
            structural_children(&ast, omitted)
                .iter()
                .filter(|child| matches!(child.kind, SurfaceNodeKind::ComprehensionVariableSegment))
                .count(),
            1
        );

        let conditioned = set_comprehension_term_for_label(&ast, "Conditioned");
        assert_eq!(
            direct_token_texts(&ast, conditioned),
            vec!["{", "where", ":", "}"]
        );
        assert!(
            structural_children(&ast, conditioned)
                .iter()
                .any(|child| matches!(child.kind, SurfaceNodeKind::FormulaExpression))
        );

        let multiple = set_comprehension_term_for_label(&ast, "MultipleGenerators");
        assert_eq!(
            structural_children(&ast, multiple)
                .iter()
                .filter(|child| matches!(child.kind, SurfaceNodeKind::ComprehensionVariableSegment))
                .count(),
            2
        );
        assert_eq!(
            direct_token_texts(&ast, multiple),
            vec!["{", "where", ",", ":", "}"]
        );

        let operator_mapper = set_comprehension_term_for_label(&ast, "OperatorMapper");
        assert!(
            subtree_contains_kind(
                &ast,
                set_comprehension_mapper(&ast, operator_mapper),
                |kind| {
                    matches!(
                        kind,
                        SurfaceNodeKind::InfixExpression(operator)
                            if operator.spelling.as_ref() == "++"
                    )
                }
            ),
            "operator mapper should preserve Pratt grouping inside SetComprehension"
        );

        let nested = set_comprehension_term_for_label(&ast, "Nested");
        assert!(
            subtree_contains_kind(&ast, set_comprehension_mapper(&ast, nested), |kind| {
                matches!(kind, SurfaceNodeKind::SetComprehension)
            }),
            "nested mapper should preserve the inner SetComprehension term"
        );

        let enumeration_formula = formula_root_for_label(&ast, "Enumeration");
        let enumeration_left = builtin_predicate_left_term_payload(&ast, enumeration_formula);
        assert!(matches!(
            enumeration_left.kind,
            SurfaceNodeKind::SetEnumeration
        ));
    }

    #[test]
    fn parser_recovers_malformed_set_comprehensions() {
        let source_id = source_id(169);
        let output = parse(ParseRequest::new(
            source_id,
            Edition::new("2026"),
            token_sequence(
                source_id,
                &[
                    ("theorem", ParserTokenKind::ReservedWord),
                    ("MissingGeneratorType", ParserTokenKind::Identifier),
                    (":", ParserTokenKind::ReservedSymbol),
                    ("{", ParserTokenKind::ReservedSymbol),
                    ("x", ParserTokenKind::Identifier),
                    ("where", ParserTokenKind::ReservedWord),
                    ("x", ParserTokenKind::Identifier),
                    ("is", ParserTokenKind::ReservedWord),
                    (":", ParserTokenKind::ReservedSymbol),
                    ("thesis", ParserTokenKind::ReservedWord),
                    ("}", ParserTokenKind::ReservedSymbol),
                    ("=", ParserTokenKind::ReservedSymbol),
                    ("y", ParserTokenKind::Identifier),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("theorem", ParserTokenKind::ReservedWord),
                    ("MissingCondition", ParserTokenKind::Identifier),
                    (":", ParserTokenKind::ReservedSymbol),
                    ("{", ParserTokenKind::ReservedSymbol),
                    ("x", ParserTokenKind::Identifier),
                    ("where", ParserTokenKind::ReservedWord),
                    ("x", ParserTokenKind::Identifier),
                    ("is", ParserTokenKind::ReservedWord),
                    ("set", ParserTokenKind::ReservedWord),
                    (":", ParserTokenKind::ReservedSymbol),
                    ("}", ParserTokenKind::ReservedSymbol),
                    ("=", ParserTokenKind::ReservedSymbol),
                    ("y", ParserTokenKind::Identifier),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("theorem", ParserTokenKind::ReservedWord),
                    ("MissingClose", ParserTokenKind::Identifier),
                    (":", ParserTokenKind::ReservedSymbol),
                    ("IsSmall", ParserTokenKind::Identifier),
                    ("(", ParserTokenKind::ReservedSymbol),
                    ("{", ParserTokenKind::ReservedSymbol),
                    ("x", ParserTokenKind::Identifier),
                    ("where", ParserTokenKind::ReservedWord),
                    ("x", ParserTokenKind::Identifier),
                    ("is", ParserTokenKind::ReservedWord),
                    ("set", ParserTokenKind::ReservedWord),
                    (")", ParserTokenKind::ReservedSymbol),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("theorem", ParserTokenKind::ReservedWord),
                    ("MissingGenerator", ParserTokenKind::Identifier),
                    (":", ParserTokenKind::ReservedSymbol),
                    ("{", ParserTokenKind::ReservedSymbol),
                    ("x", ParserTokenKind::Identifier),
                    ("where", ParserTokenKind::ReservedWord),
                    ("is", ParserTokenKind::ReservedWord),
                    ("set", ParserTokenKind::ReservedWord),
                    (":", ParserTokenKind::ReservedSymbol),
                    ("thesis", ParserTokenKind::ReservedWord),
                    ("}", ParserTokenKind::ReservedSymbol),
                    ("=", ParserTokenKind::ReservedSymbol),
                    ("y", ParserTokenKind::Identifier),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("theorem", ParserTokenKind::ReservedWord),
                    ("MissingIs", ParserTokenKind::Identifier),
                    (":", ParserTokenKind::ReservedSymbol),
                    ("{", ParserTokenKind::ReservedSymbol),
                    ("x", ParserTokenKind::Identifier),
                    ("where", ParserTokenKind::ReservedWord),
                    ("x", ParserTokenKind::Identifier),
                    ("set", ParserTokenKind::ReservedWord),
                    (":", ParserTokenKind::ReservedSymbol),
                    ("thesis", ParserTokenKind::ReservedWord),
                    ("}", ParserTokenKind::ReservedSymbol),
                    ("=", ParserTokenKind::ReservedSymbol),
                    ("y", ParserTokenKind::Identifier),
                    (";", ParserTokenKind::ReservedSymbol),
                ],
            ),
            Vec::new(),
        ));

        assert_eq!(
            output
                .diagnostics
                .iter()
                .map(|diagnostic| &diagnostic.code)
                .collect::<Vec<_>>(),
            vec![
                &SyntaxDiagnosticCode::MalformedTypeExpression,
                &SyntaxDiagnosticCode::MalformedFormulaExpression,
                &SyntaxDiagnosticCode::MalformedTermExpression,
                &SyntaxDiagnosticCode::MalformedTermExpression,
                &SyntaxDiagnosticCode::MalformedTermExpression,
            ]
        );
        let ast = output
            .ast
            .expect("malformed set comprehensions should recover an AST");
        assert_eq!(
            count_nodes(&ast, |kind| matches!(
                kind,
                SurfaceNodeKind::SetComprehension
            )),
            5
        );
        assert_eq!(
            count_nodes(&ast, |kind| matches!(
                kind,
                SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::MissingTypeExpression)
            )),
            1
        );
        assert_eq!(
            count_nodes(&ast, |kind| matches!(
                kind,
                SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::MissingFormula)
            )),
            1
        );
        assert_eq!(
            count_nodes(&ast, |kind| matches!(
                kind,
                SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::MissingTerm)
            )),
            1
        );
        assert_eq!(
            count_nodes(&ast, |kind| matches!(
                kind,
                SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::UnmatchedOpeningDelimiter)
            )),
            1
        );
        assert_eq!(
            count_nodes(&ast, |kind| matches!(
                kind,
                SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::SkippedToken)
            )),
            1
        );
    }

    #[test]
    fn parser_defers_template_predicate_args_inside_set_comprehension_conditions() {
        let source_id = source_id(170);
        let output = parse(ParseRequest::new(
            source_id,
            Edition::new("2026"),
            token_sequence(
                source_id,
                &[
                    ("theorem", ParserTokenKind::ReservedWord),
                    ("TemplateCondition", ParserTokenKind::Identifier),
                    (":", ParserTokenKind::ReservedSymbol),
                    ("{", ParserTokenKind::ReservedSymbol),
                    ("x", ParserTokenKind::Identifier),
                    ("where", ParserTokenKind::ReservedWord),
                    ("x", ParserTokenKind::Identifier),
                    ("is", ParserTokenKind::ReservedWord),
                    ("set", ParserTokenKind::ReservedWord),
                    (":", ParserTokenKind::ReservedSymbol),
                    ("y", ParserTokenKind::Identifier),
                    ("R", ParserTokenKind::UserSymbol),
                    ("[", ParserTokenKind::ReservedSymbol),
                    ("T", ParserTokenKind::UserSymbol),
                    ("]", ParserTokenKind::ReservedSymbol),
                    ("z", ParserTokenKind::Identifier),
                    ("}", ParserTokenKind::ReservedSymbol),
                    ("=", ParserTokenKind::ReservedSymbol),
                    ("w", ParserTokenKind::Identifier),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("theorem", ParserTokenKind::ReservedWord),
                    ("NestedTemplateCondition", ParserTokenKind::Identifier),
                    (":", ParserTokenKind::ReservedSymbol),
                    ("IsSmall", ParserTokenKind::Identifier),
                    ("(", ParserTokenKind::ReservedSymbol),
                    ("{", ParserTokenKind::ReservedSymbol),
                    ("x", ParserTokenKind::Identifier),
                    ("where", ParserTokenKind::ReservedWord),
                    ("x", ParserTokenKind::Identifier),
                    ("is", ParserTokenKind::ReservedWord),
                    ("set", ParserTokenKind::ReservedWord),
                    (":", ParserTokenKind::ReservedSymbol),
                    ("y", ParserTokenKind::Identifier),
                    ("R", ParserTokenKind::UserSymbol),
                    ("[", ParserTokenKind::ReservedSymbol),
                    ("T", ParserTokenKind::UserSymbol),
                    ("]", ParserTokenKind::ReservedSymbol),
                    ("z", ParserTokenKind::Identifier),
                    ("}", ParserTokenKind::ReservedSymbol),
                    (")", ParserTokenKind::ReservedSymbol),
                    (";", ParserTokenKind::ReservedSymbol),
                ],
            ),
            Vec::new(),
        ));

        assert!(
            output.diagnostics.is_empty(),
            "template predicate args inside comprehension conditions remain deferred: {:?}",
            output.diagnostics
        );
        let ast = output
            .ast
            .expect("deferred comprehension condition should keep an AST");
        assert_eq!(
            count_nodes(&ast, |kind| matches!(
                kind,
                SurfaceNodeKind::FormulaExpression
            )),
            0
        );
        assert_eq!(
            count_nodes(&ast, |kind| matches!(
                kind,
                SurfaceNodeKind::SetComprehension
            )),
            0
        );
        let item_list = single_node(&ast, |kind| matches!(kind, SurfaceNodeKind::ItemList));
        assert_eq!(item_list.children.len(), 2);
        let item = single_node(&ast, |kind| {
            matches!(kind, SurfaceNodeKind::PlaceholderItem)
        });
        assert!(!direct_child_has_kind(&ast, item, |kind| {
            matches!(kind, SurfaceNodeKind::FormulaExpression)
        }));
    }

    #[test]
    fn parser_keeps_template_predicate_args_deferred() {
        let source_id = source_id(72);
        let output = parse(ParseRequest::new(
            source_id,
            Edition::new("2026"),
            token_sequence(
                source_id,
                &[
                    ("theorem", ParserTokenKind::ReservedWord),
                    ("TemplateArgs", ParserTokenKind::Identifier),
                    (":", ParserTokenKind::ReservedSymbol),
                    ("x", ParserTokenKind::Identifier),
                    ("divides", ParserTokenKind::UserSymbol),
                    ("[", ParserTokenKind::ReservedSymbol),
                    ("T", ParserTokenKind::UserSymbol),
                    ("]", ParserTokenKind::ReservedSymbol),
                    ("y", ParserTokenKind::Identifier),
                    (";", ParserTokenKind::ReservedSymbol),
                ],
            ),
            Vec::new(),
        ));

        assert!(
            output.diagnostics.is_empty(),
            "template predicate args remain task-31/S-016 work: {:?}",
            output.diagnostics
        );
        let ast = output
            .ast
            .expect("deferred template args should keep an AST");
        assert_eq!(
            count_nodes(&ast, |kind| matches!(
                kind,
                SurfaceNodeKind::FormulaExpression
            )),
            0
        );
        let item = single_node(&ast, |kind| {
            matches!(kind, SurfaceNodeKind::PlaceholderItem)
        });
        assert!(!direct_child_has_kind(&ast, item, |kind| {
            matches!(kind, SurfaceNodeKind::FormulaExpression)
        }));
    }

    #[test]
    fn parser_defers_template_predicate_args_after_formula_boundaries() {
        let source_id = source_id(166);
        let output = parse(ParseRequest::new(
            source_id,
            Edition::new("2026"),
            token_sequence(
                source_id,
                &[
                    ("theorem", ParserTokenKind::ReservedWord),
                    ("IsAssertionThenTemplate", ParserTokenKind::Identifier),
                    (":", ParserTokenKind::ReservedSymbol),
                    ("x", ParserTokenKind::Identifier),
                    ("is", ParserTokenKind::ReservedWord),
                    ("set", ParserTokenKind::ReservedWord),
                    ("&", ParserTokenKind::ReservedSymbol),
                    ("y", ParserTokenKind::Identifier),
                    ("R", ParserTokenKind::UserSymbol),
                    ("[", ParserTokenKind::ReservedSymbol),
                    ("T", ParserTokenKind::UserSymbol),
                    ("]", ParserTokenKind::ReservedSymbol),
                    ("z", ParserTokenKind::Identifier),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("theorem", ParserTokenKind::ReservedWord),
                    ("QuantifierHoldsTemplate", ParserTokenKind::Identifier),
                    (":", ParserTokenKind::ReservedSymbol),
                    ("for", ParserTokenKind::ReservedWord),
                    ("x", ParserTokenKind::Identifier),
                    ("st", ParserTokenKind::ReservedWord),
                    ("x", ParserTokenKind::Identifier),
                    ("is", ParserTokenKind::ReservedWord),
                    ("set", ParserTokenKind::ReservedWord),
                    ("holds", ParserTokenKind::ReservedWord),
                    ("y", ParserTokenKind::Identifier),
                    ("R", ParserTokenKind::UserSymbol),
                    ("[", ParserTokenKind::ReservedSymbol),
                    ("T", ParserTokenKind::UserSymbol),
                    ("]", ParserTokenKind::ReservedSymbol),
                    ("z", ParserTokenKind::Identifier),
                    (";", ParserTokenKind::ReservedSymbol),
                ],
            ),
            Vec::new(),
        ));

        assert!(
            output.diagnostics.is_empty(),
            "template predicate args past formula boundaries remain deferred: {:?}",
            output.diagnostics
        );
        let ast = output
            .ast
            .expect("deferred template args should keep an AST");
        assert_eq!(
            count_nodes(&ast, |kind| matches!(
                kind,
                SurfaceNodeKind::FormulaExpression
            )),
            0
        );
        let item_list = single_node(&ast, |kind| matches!(kind, SurfaceNodeKind::ItemList));
        assert_eq!(item_list.children.len(), 2);
        assert!(item_list.children.iter().all(|item| {
            let item = ast.node(*item).unwrap();
            matches!(item.kind, SurfaceNodeKind::PlaceholderItem)
                && !direct_child_has_kind(&ast, item, |kind| {
                    matches!(kind, SurfaceNodeKind::FormulaExpression)
                })
        }));
    }

    #[test]
    fn parser_parses_formula_quantifiers_and_variable_segments() {
        let source_id = source_id(73);
        let output = parse(ParseRequest::new(
            source_id,
            Edition::new("2026"),
            token_sequence(
                source_id,
                &[
                    ("theorem", ParserTokenKind::ReservedWord),
                    ("U", ParserTokenKind::Identifier),
                    (":", ParserTokenKind::ReservedSymbol),
                    ("for", ParserTokenKind::ReservedWord),
                    ("x", ParserTokenKind::Identifier),
                    ("being", ParserTokenKind::ReservedWord),
                    ("set", ParserTokenKind::ReservedWord),
                    ("st", ParserTokenKind::ReservedWord),
                    ("thesis", ParserTokenKind::ReservedWord),
                    ("holds", ParserTokenKind::ReservedWord),
                    ("contradiction", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("theorem", ParserTokenKind::ReservedWord),
                    ("Nested", ParserTokenKind::Identifier),
                    (":", ParserTokenKind::ReservedSymbol),
                    ("for", ParserTokenKind::ReservedWord),
                    ("x", ParserTokenKind::Identifier),
                    ("for", ParserTokenKind::ReservedWord),
                    ("y", ParserTokenKind::Identifier),
                    ("holds", ParserTokenKind::ReservedWord),
                    ("thesis", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("theorem", ParserTokenKind::ReservedWord),
                    ("E", ParserTokenKind::Identifier),
                    (":", ParserTokenKind::ReservedSymbol),
                    ("ex", ParserTokenKind::ReservedWord),
                    ("y", ParserTokenKind::Identifier),
                    ("st", ParserTokenKind::ReservedWord),
                    ("thesis", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("theorem", ParserTokenKind::ReservedWord),
                    ("Implicit", ParserTokenKind::Identifier),
                    (":", ParserTokenKind::ReservedSymbol),
                    ("for", ParserTokenKind::ReservedWord),
                    ("x", ParserTokenKind::Identifier),
                    (",", ParserTokenKind::ReservedSymbol),
                    ("y", ParserTokenKind::Identifier),
                    ("holds", ParserTokenKind::ReservedWord),
                    ("thesis", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("theorem", ParserTokenKind::ReservedWord),
                    ("ExplicitImplicit", ParserTokenKind::Identifier),
                    (":", ParserTokenKind::ReservedSymbol),
                    ("for", ParserTokenKind::ReservedWord),
                    ("x", ParserTokenKind::Identifier),
                    ("being", ParserTokenKind::ReservedWord),
                    ("set", ParserTokenKind::ReservedWord),
                    (",", ParserTokenKind::ReservedSymbol),
                    ("y", ParserTokenKind::Identifier),
                    ("holds", ParserTokenKind::ReservedWord),
                    ("thesis", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("theorem", ParserTokenKind::ReservedWord),
                    ("Recursive", ParserTokenKind::Identifier),
                    (":", ParserTokenKind::ReservedSymbol),
                    ("for", ParserTokenKind::ReservedWord),
                    ("x", ParserTokenKind::Identifier),
                    ("st", ParserTokenKind::ReservedWord),
                    ("x", ParserTokenKind::Identifier),
                    ("=", ParserTokenKind::ReservedSymbol),
                    ("y", ParserTokenKind::Identifier),
                    ("or", ParserTokenKind::ReservedWord),
                    ("thesis", ParserTokenKind::ReservedWord),
                    ("holds", ParserTokenKind::ReservedWord),
                    ("not", ParserTokenKind::ReservedWord),
                    ("contradiction", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("theorem", ParserTokenKind::ReservedWord),
                    ("ExistentialRecursive", ParserTokenKind::Identifier),
                    (":", ParserTokenKind::ReservedSymbol),
                    ("ex", ParserTokenKind::ReservedWord),
                    ("x", ParserTokenKind::Identifier),
                    ("st", ParserTokenKind::ReservedWord),
                    ("x", ParserTokenKind::Identifier),
                    ("=", ParserTokenKind::ReservedSymbol),
                    ("y", ParserTokenKind::Identifier),
                    ("&", ParserTokenKind::ReservedSymbol),
                    ("thesis", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                ],
            ),
            Vec::new(),
        ));

        assert!(
            output.diagnostics.is_empty(),
            "quantified formulas should parse without diagnostics: {:?}",
            output.diagnostics
        );
        let ast = output.ast.expect("quantified formulas should keep an AST");
        assert_eq!(
            count_nodes(&ast, |kind| matches!(
                kind,
                SurfaceNodeKind::QuantifiedFormula(_)
            )),
            8
        );
        assert_eq!(
            count_nodes(&ast, |kind| matches!(
                kind,
                SurfaceNodeKind::QuantifierVariableSegment
            )),
            9
        );
        assert!(ast.nodes().iter().any(|node| {
            matches!(
                node.kind,
                SurfaceNodeKind::QuantifiedFormula(SurfaceQuantifierKind::Universal)
            )
        }));
        assert!(ast.nodes().iter().any(|node| {
            matches!(
                node.kind,
                SurfaceNodeKind::QuantifiedFormula(SurfaceQuantifierKind::Existential)
            )
        }));
        assert_eq!(
            count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::TypeExpression)),
            2
        );
        assert!(
            ast.nodes()
                .iter()
                .filter(|node| matches!(node.kind, SurfaceNodeKind::QuantifierVariableSegment))
                .any(|segment| {
                    segment
                        .children
                        .iter()
                        .filter_map(|id| ast.node(*id))
                        .any(|child| child.token_text() == Some(","))
                }),
            "implicit variable segment should preserve comma-separated variables"
        );

        let universal = formula_root_for_label(&ast, "U");
        assert_quantified_formula(universal, SurfaceQuantifierKind::Universal);
        assert_eq!(
            direct_token_texts(&ast, universal),
            vec!["for", "st", "holds"]
        );
        let universal_children = structural_children(&ast, universal);
        assert_eq!(universal_children.len(), 3);
        assert!(matches!(
            universal_children[0].kind,
            SurfaceNodeKind::QuantifierVariableSegment
        ));
        assert_eq!(
            direct_token_texts(&ast, universal_children[0]),
            vec!["x", "being"]
        );
        assert!(
            structural_children(&ast, universal_children[0])
                .iter()
                .any(|child| matches!(child.kind, SurfaceNodeKind::TypeExpression))
        );
        assert_formula_constant(universal_children[1], SurfaceFormulaConstant::Thesis);
        assert_formula_constant(universal_children[2], SurfaceFormulaConstant::Contradiction);

        let nested_outer = formula_root_for_label(&ast, "Nested");
        assert_quantified_formula(nested_outer, SurfaceQuantifierKind::Universal);
        assert_eq!(direct_token_texts(&ast, nested_outer), vec!["for"]);
        let nested_outer_children = structural_children(&ast, nested_outer);
        assert_eq!(nested_outer_children.len(), 2);
        assert_quantified_formula(nested_outer_children[1], SurfaceQuantifierKind::Universal);
        assert_eq!(
            direct_token_texts(&ast, nested_outer_children[1]),
            vec!["for", "holds"]
        );

        let existential = formula_root_for_label(&ast, "E");
        assert_quantified_formula(existential, SurfaceQuantifierKind::Existential);
        assert_eq!(direct_token_texts(&ast, existential), vec!["ex", "st"]);
        let existential_children = structural_children(&ast, existential);
        assert_eq!(existential_children.len(), 2);
        assert!(matches!(
            existential_children[0].kind,
            SurfaceNodeKind::QuantifierVariableSegment
        ));
        assert_formula_constant(existential_children[1], SurfaceFormulaConstant::Thesis);

        let implicit = formula_root_for_label(&ast, "Implicit");
        assert_quantified_formula(implicit, SurfaceQuantifierKind::Universal);
        assert_eq!(direct_token_texts(&ast, implicit), vec!["for", "holds"]);
        let implicit_children = structural_children(&ast, implicit);
        assert_eq!(
            direct_token_texts(&ast, implicit_children[0]),
            vec!["x", ",", "y"]
        );

        let explicit_implicit = formula_root_for_label(&ast, "ExplicitImplicit");
        assert_quantified_formula(explicit_implicit, SurfaceQuantifierKind::Universal);
        assert_eq!(
            direct_token_texts(&ast, explicit_implicit),
            vec!["for", ",", "holds"]
        );
        let explicit_implicit_children = structural_children(&ast, explicit_implicit);
        assert_eq!(explicit_implicit_children.len(), 3);
        assert_eq!(
            direct_token_texts(&ast, explicit_implicit_children[0]),
            vec!["x", "being"]
        );
        assert!(
            structural_children(&ast, explicit_implicit_children[0])
                .iter()
                .any(|child| matches!(child.kind, SurfaceNodeKind::TypeExpression))
        );
        assert_eq!(
            direct_token_texts(&ast, explicit_implicit_children[1]),
            vec!["y"]
        );
        assert!(structural_children(&ast, explicit_implicit_children[1]).is_empty());
        assert_formula_constant(
            explicit_implicit_children[2],
            SurfaceFormulaConstant::Thesis,
        );

        let recursive = formula_root_for_label(&ast, "Recursive");
        assert_quantified_formula(recursive, SurfaceQuantifierKind::Universal);
        assert_eq!(
            direct_token_texts(&ast, recursive),
            vec!["for", "st", "holds"]
        );
        let recursive_children = structural_children(&ast, recursive);
        assert_eq!(recursive_children.len(), 3);
        assert_binary_formula(recursive_children[1], SurfaceFormulaConnective::Or, false);
        assert!(matches!(
            recursive_children[2].kind,
            SurfaceNodeKind::PrefixFormula(_)
        ));

        let existential_recursive = formula_root_for_label(&ast, "ExistentialRecursive");
        assert_quantified_formula(existential_recursive, SurfaceQuantifierKind::Existential);
        assert_eq!(
            direct_token_texts(&ast, existential_recursive),
            vec!["ex", "st"]
        );
        let existential_recursive_children = structural_children(&ast, existential_recursive);
        assert_eq!(existential_recursive_children.len(), 2);
        assert_binary_formula(
            existential_recursive_children[1],
            SurfaceFormulaConnective::And,
            false,
        );
    }

    #[test]
    fn parser_parses_task16_simple_statement_nodes() {
        let source_id = source_id(76);
        let output = parse(ParseRequest::new(
            source_id,
            Edition::new("2026"),
            token_sequence(
                source_id,
                &[
                    ("reserve", ParserTokenKind::ReservedWord),
                    ("r", ParserTokenKind::Identifier),
                    ("for", ParserTokenKind::ReservedWord),
                    ("set", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("let", ParserTokenKind::ReservedWord),
                    ("x", ParserTokenKind::Identifier),
                    (",", ParserTokenKind::ReservedSymbol),
                    ("y", ParserTokenKind::Identifier),
                    ("be", ParserTokenKind::ReservedWord),
                    ("set", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("assume", ParserTokenKind::ReservedWord),
                    ("that", ParserTokenKind::ReservedWord),
                    ("thesis", ParserTokenKind::ReservedWord),
                    ("and", ParserTokenKind::ReservedWord),
                    ("contradiction", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("assume", ParserTokenKind::ReservedWord),
                    ("A", ParserTokenKind::Identifier),
                    (":", ParserTokenKind::ReservedSymbol),
                    ("thesis", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("given", ParserTokenKind::ReservedWord),
                    ("z", ParserTokenKind::Identifier),
                    ("being", ParserTokenKind::ReservedWord),
                    ("set", ParserTokenKind::ReservedWord),
                    ("such", ParserTokenKind::ReservedWord),
                    ("that", ParserTokenKind::ReservedWord),
                    ("thesis", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("take", ParserTokenKind::ReservedWord),
                    ("a", ParserTokenKind::Identifier),
                    ("=", ParserTokenKind::ReservedSymbol),
                    ("x", ParserTokenKind::Identifier),
                    (",", ParserTokenKind::ReservedSymbol),
                    ("y", ParserTokenKind::Identifier),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("set", ParserTokenKind::ReservedWord),
                    ("f", ParserTokenKind::Identifier),
                    ("=", ParserTokenKind::ReservedSymbol),
                    ("x", ParserTokenKind::Identifier),
                    (",", ParserTokenKind::ReservedSymbol),
                    ("g", ParserTokenKind::Identifier),
                    ("=", ParserTokenKind::ReservedSymbol),
                    ("y", ParserTokenKind::Identifier),
                    (";", ParserTokenKind::ReservedSymbol),
                ],
            ),
            Vec::new(),
        ));

        assert!(
            output.diagnostics.is_empty(),
            "simple statements should parse without diagnostics: {:?}",
            output.diagnostics
        );
        let ast = output.ast.expect("simple statements should keep an AST");
        let item_list = single_node(&ast, |kind| matches!(kind, SurfaceNodeKind::ItemList));
        assert_eq!(item_list.children.len(), 7);
        assert_eq!(
            count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::ReserveItem)),
            1,
            "top-level reserve should keep the existing ReserveItem path"
        );
        assert_eq!(
            count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::StatementItem)),
            6
        );
        assert_eq!(
            count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::LetStatement)),
            1
        );
        assert_eq!(
            count_nodes(&ast, |kind| matches!(
                kind,
                SurfaceNodeKind::QualifiedVariableSegment
            )),
            2
        );
        assert_eq!(
            count_nodes(&ast, |kind| matches!(
                kind,
                SurfaceNodeKind::AssumptionStatement
            )),
            2
        );
        assert_eq!(
            count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::ConditionList)),
            2
        );
        assert_eq!(
            count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::Proposition)),
            4
        );
        assert_eq!(
            count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::GivenStatement)),
            1
        );
        assert_eq!(
            count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::TakeStatement)),
            1
        );
        assert_eq!(
            count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::Witness)),
            2
        );
        assert_eq!(
            count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::SetStatement)),
            1
        );
        assert_eq!(
            count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::Equating)),
            2
        );

        let let_statement = single_node(&ast, |kind| matches!(kind, SurfaceNodeKind::LetStatement));
        assert_eq!(direct_token_texts(&ast, let_statement), vec!["let", ";"]);
        let let_segment = structural_children(&ast, let_statement)
            .into_iter()
            .find(|child| matches!(child.kind, SurfaceNodeKind::QualifiedVariableSegment))
            .expect("let should own a qualified variable segment");
        assert_eq!(
            direct_token_texts(&ast, let_segment),
            vec!["x", ",", "y", "be"]
        );

        let assumption_conditions =
            single_node(&ast, |kind| matches!(kind, SurfaceNodeKind::ConditionList));
        assert_eq!(
            direct_token_texts(&ast, assumption_conditions),
            vec!["that", "and"]
        );
        assert!(
            ast.nodes().iter().any(|node| {
                matches!(node.kind, SurfaceNodeKind::Proposition)
                    && direct_token_texts(&ast, node) == vec!["A", ":"]
            }),
            "statement propositions should preserve optional label tokens"
        );

        let set_statement = single_node(&ast, |kind| matches!(kind, SurfaceNodeKind::SetStatement));
        assert_eq!(
            direct_token_texts(&ast, set_statement),
            vec!["set", ",", ";"]
        );
        let equatings = structural_children(&ast, set_statement)
            .into_iter()
            .filter(|child| matches!(child.kind, SurfaceNodeKind::Equating))
            .collect::<Vec<_>>();
        assert_eq!(equatings.len(), 2);
        assert_eq!(direct_token_texts(&ast, equatings[0]), vec!["f", "="]);
        assert_eq!(direct_token_texts(&ast, equatings[1]), vec!["g", "="]);
    }

    #[test]
    fn parser_recovers_task16_simple_statement_gaps() {
        let source_id = source_id(77);
        let output = parse(ParseRequest::new(
            source_id,
            Edition::new("2026"),
            token_sequence(
                source_id,
                &[
                    ("let", ParserTokenKind::ReservedWord),
                    ("x", ParserTokenKind::Identifier),
                    ("be", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("assume", ParserTokenKind::ReservedWord),
                    ("that", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("take", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("set", ParserTokenKind::ReservedWord),
                    ("=", ParserTokenKind::ReservedSymbol),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("set", ParserTokenKind::ReservedWord),
                    ("f", ParserTokenKind::Identifier),
                    ("x", ParserTokenKind::Identifier),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("take", ParserTokenKind::ReservedWord),
                    ("x", ParserTokenKind::Identifier),
                    ("assume", ParserTokenKind::ReservedWord),
                    ("thesis", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                ],
            ),
            Vec::new(),
        ));

        assert!(
            output
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == SyntaxDiagnosticCode::MalformedTypeExpression)
        );
        assert!(output.diagnostics.iter().any(|diagnostic| {
            diagnostic.code == SyntaxDiagnosticCode::MalformedFormulaExpression
        }));
        assert!(
            output
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == SyntaxDiagnosticCode::MalformedTermExpression)
        );
        assert!(
            output
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == SyntaxDiagnosticCode::MissingSemicolon)
        );
        let ast = output.ast.expect("statement recovery should keep an AST");
        assert_eq!(
            count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::StatementItem)),
            7
        );
        assert_eq!(
            count_nodes(&ast, |kind| {
                matches!(
                    kind,
                    SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::MissingTypeExpression)
                )
            }),
            1
        );
        assert_eq!(
            count_nodes(&ast, |kind| {
                matches!(
                    kind,
                    SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::MissingFormula)
                )
            }),
            1
        );
        assert_eq!(
            count_nodes(&ast, |kind| {
                matches!(
                    kind,
                    SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::MissingTerm)
                )
            }),
            4
        );
        assert_eq!(
            count_nodes(&ast, |kind| {
                matches!(
                    kind,
                    SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::SkippedToken)
                )
            }),
            1
        );
        assert_eq!(ast.trivia().skipped_token_ranges().len(), 1);
        assert_eq!(
            ast.trivia().skipped_token_ranges()[0].reason,
            SkippedTokenReason::Recovery
        );
    }

    #[test]
    fn parser_keeps_set_type_and_set_statement_contexts_separate() {
        let source_id = source_id(78);
        let output = parse(ParseRequest::new(
            source_id,
            Edition::new("2026"),
            token_sequence(
                source_id,
                &[
                    ("assume", ParserTokenKind::ReservedWord),
                    ("x", ParserTokenKind::Identifier),
                    ("is", ParserTokenKind::ReservedWord),
                    ("set", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("set", ParserTokenKind::ReservedWord),
                    ("f", ParserTokenKind::Identifier),
                    ("=", ParserTokenKind::ReservedSymbol),
                    ("x", ParserTokenKind::Identifier),
                    (";", ParserTokenKind::ReservedSymbol),
                ],
            ),
            Vec::new(),
        ));

        assert!(
            output.diagnostics.is_empty(),
            "`set` type and `set` statement should not conflict: {:?}",
            output.diagnostics
        );
        let ast = output.ast.expect("set contexts should keep an AST");
        assert_eq!(
            count_nodes(&ast, |kind| matches!(
                kind,
                SurfaceNodeKind::AssumptionStatement
            )),
            1
        );
        assert_eq!(
            count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::IsAssertion)),
            1
        );
        assert_eq!(
            count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::SetStatement)),
            1
        );
    }

    #[test]
    fn parser_parses_task17_justification_nodes() {
        let source_id = source_id(79);
        let output = parse(ParseRequest::new(
            source_id,
            Edition::new("2026"),
            token_sequence(
                source_id,
                &[
                    ("let", ParserTokenKind::ReservedWord),
                    ("x", ParserTokenKind::Identifier),
                    ("be", ParserTokenKind::ReservedWord),
                    ("set", ParserTokenKind::ReservedWord),
                    ("by", ParserTokenKind::ReservedWord),
                    ("A", ParserTokenKind::Identifier),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("thesis", ParserTokenKind::ReservedWord),
                    ("by", ParserTokenKind::ReservedWord),
                    ("A", ParserTokenKind::Identifier),
                    (",", ParserTokenKind::ReservedSymbol),
                    ("mml", ParserTokenKind::Identifier),
                    (".", ParserTokenKind::ReservedSymbol),
                    ("foo", ParserTokenKind::Identifier),
                    (".", ParserTokenKind::ReservedSymbol),
                    ("Th1", ParserTokenKind::Identifier),
                    (",", ParserTokenKind::ReservedSymbol),
                    ("mml", ParserTokenKind::Identifier),
                    (".", ParserTokenKind::ReservedSymbol),
                    ("foo", ParserTokenKind::Identifier),
                    (".{", ParserTokenKind::ReservedSymbol),
                    ("G1", ParserTokenKind::Identifier),
                    (",", ParserTokenKind::ReservedSymbol),
                    ("G2", ParserTokenKind::Identifier),
                    ("}", ParserTokenKind::ReservedSymbol),
                    (",", ParserTokenKind::ReservedSymbol),
                    ("mml", ParserTokenKind::Identifier),
                    (".", ParserTokenKind::ReservedSymbol),
                    ("foo", ParserTokenKind::Identifier),
                    (".*", ParserTokenKind::ReservedSymbol),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("thesis", ParserTokenKind::ReservedWord),
                    ("by", ParserTokenKind::ReservedWord),
                    ("computation", ParserTokenKind::ReservedWord),
                    ("(", ParserTokenKind::ReservedSymbol),
                    ("steps", ParserTokenKind::Identifier),
                    (":", ParserTokenKind::ReservedSymbol),
                    ("10", ParserTokenKind::Numeral),
                    (",", ParserTokenKind::ReservedSymbol),
                    ("timeout", ParserTokenKind::Identifier),
                    (":", ParserTokenKind::ReservedSymbol),
                    ("20", ParserTokenKind::Numeral),
                    (",", ParserTokenKind::ReservedSymbol),
                    ("nest", ParserTokenKind::ReservedWord),
                    (":", ParserTokenKind::ReservedSymbol),
                    ("3", ParserTokenKind::Numeral),
                    (")", ParserTokenKind::ReservedSymbol),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("thesis", ParserTokenKind::ReservedWord),
                    ("by", ParserTokenKind::ReservedWord),
                    ("computation", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                ],
            ),
            Vec::new(),
        ));

        assert!(
            output.diagnostics.is_empty(),
            "task-17 justifications should parse without diagnostics: {:?}",
            output.diagnostics
        );
        let ast = output
            .ast
            .expect("justification statements should keep an AST");
        let item_list = single_node(&ast, |kind| matches!(kind, SurfaceNodeKind::ItemList));
        assert_eq!(item_list.children.len(), 4);
        assert_eq!(
            count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::StatementItem)),
            4
        );
        assert_eq!(
            count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::LetStatement)),
            1
        );
        assert_eq!(
            count_nodes(&ast, |kind| matches!(
                kind,
                SurfaceNodeKind::CompactStatement
            )),
            3
        );
        assert_eq!(
            count_nodes(&ast, |kind| matches!(
                kind,
                SurfaceNodeKind::JustificationClause
            )),
            4
        );
        assert_eq!(
            count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::ReferenceList)),
            2
        );
        assert_eq!(
            count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::Reference)),
            2
        );
        assert_eq!(
            count_nodes(&ast, |kind| matches!(
                kind,
                SurfaceNodeKind::QualifiedReference
            )),
            1
        );
        assert_eq!(
            count_nodes(&ast, |kind| matches!(
                kind,
                SurfaceNodeKind::GroupedReference
            )),
            1
        );
        assert_eq!(
            count_nodes(&ast, |kind| matches!(
                kind,
                SurfaceNodeKind::GroupedReferenceItem
            )),
            2
        );
        assert_eq!(
            count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::BulkReference)),
            1
        );
        assert_eq!(
            count_nodes(&ast, |kind| matches!(
                kind,
                SurfaceNodeKind::ComputationJustification
            )),
            2
        );
        assert_eq!(
            count_nodes(&ast, |kind| matches!(
                kind,
                SurfaceNodeKind::ComputationOption
            )),
            3
        );

        let let_statement = single_node(&ast, |kind| matches!(kind, SurfaceNodeKind::LetStatement));
        assert!(direct_child_has_kind(&ast, let_statement, |kind| {
            matches!(kind, SurfaceNodeKind::JustificationClause)
        }));
        let grouped = single_node(&ast, |kind| {
            matches!(kind, SurfaceNodeKind::GroupedReference)
        });
        assert_eq!(direct_token_texts(&ast, grouped), vec![".{", ",", "}"]);
        let computation = single_node(&ast, |kind| {
            matches!(kind, SurfaceNodeKind::ComputationJustification)
        });
        assert_eq!(
            direct_token_texts(&ast, computation),
            vec!["computation", "(", ",", ",", ")"]
        );
    }

    #[test]
    fn parser_recovers_task17_justification_gaps() {
        let source_id = source_id(80);
        let output = parse(ParseRequest::new(
            source_id,
            Edition::new("2026"),
            token_sequence(
                source_id,
                &[
                    ("thesis", ParserTokenKind::ReservedWord),
                    ("by", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("thesis", ParserTokenKind::ReservedWord),
                    ("by", ParserTokenKind::ReservedWord),
                    (",", ParserTokenKind::ReservedSymbol),
                    ("A", ParserTokenKind::Identifier),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("thesis", ParserTokenKind::ReservedWord),
                    ("by", ParserTokenKind::ReservedWord),
                    ("A", ParserTokenKind::Identifier),
                    (",", ParserTokenKind::ReservedSymbol),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("thesis", ParserTokenKind::ReservedWord),
                    ("by", ParserTokenKind::ReservedWord),
                    ("A", ParserTokenKind::Identifier),
                    ("[", ParserTokenKind::ReservedSymbol),
                    ("T", ParserTokenKind::Identifier),
                    ("]", ParserTokenKind::ReservedSymbol),
                    (",", ParserTokenKind::ReservedSymbol),
                    ("B", ParserTokenKind::Identifier),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("thesis", ParserTokenKind::ReservedWord),
                    ("by", ParserTokenKind::ReservedWord),
                    ("mml", ParserTokenKind::Identifier),
                    (".", ParserTokenKind::ReservedSymbol),
                    ("foo", ParserTokenKind::Identifier),
                    (".{", ParserTokenKind::ReservedSymbol),
                    ("G1", ParserTokenKind::Identifier),
                    (",", ParserTokenKind::ReservedSymbol),
                    ("}", ParserTokenKind::ReservedSymbol),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("thesis", ParserTokenKind::ReservedWord),
                    ("by", ParserTokenKind::ReservedWord),
                    ("mml", ParserTokenKind::Identifier),
                    (".", ParserTokenKind::ReservedSymbol),
                    ("foo", ParserTokenKind::Identifier),
                    (".{", ParserTokenKind::ReservedSymbol),
                    (",", ParserTokenKind::ReservedSymbol),
                    ("G1", ParserTokenKind::Identifier),
                    ("}", ParserTokenKind::ReservedSymbol),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("thesis", ParserTokenKind::ReservedWord),
                    ("by", ParserTokenKind::ReservedWord),
                    ("mml", ParserTokenKind::Identifier),
                    (".", ParserTokenKind::ReservedSymbol),
                    ("foo", ParserTokenKind::Identifier),
                    (".{", ParserTokenKind::ReservedSymbol),
                    ("G1", ParserTokenKind::Identifier),
                    (",", ParserTokenKind::ReservedSymbol),
                    (",", ParserTokenKind::ReservedSymbol),
                    ("G2", ParserTokenKind::Identifier),
                    ("}", ParserTokenKind::ReservedSymbol),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("thesis", ParserTokenKind::ReservedWord),
                    ("by", ParserTokenKind::ReservedWord),
                    ("mml", ParserTokenKind::Identifier),
                    (".", ParserTokenKind::ReservedSymbol),
                    ("foo", ParserTokenKind::Identifier),
                    (".{", ParserTokenKind::ReservedSymbol),
                    ("G1", ParserTokenKind::Identifier),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("thesis", ParserTokenKind::ReservedWord),
                    ("by", ParserTokenKind::ReservedWord),
                    ("computation", ParserTokenKind::ReservedWord),
                    ("(", ParserTokenKind::ReservedSymbol),
                    (",", ParserTokenKind::ReservedSymbol),
                    ("steps", ParserTokenKind::Identifier),
                    (":", ParserTokenKind::ReservedSymbol),
                    ("1", ParserTokenKind::Numeral),
                    (")", ParserTokenKind::ReservedSymbol),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("thesis", ParserTokenKind::ReservedWord),
                    ("by", ParserTokenKind::ReservedWord),
                    ("computation", ParserTokenKind::ReservedWord),
                    ("(", ParserTokenKind::ReservedSymbol),
                    ("steps", ParserTokenKind::Identifier),
                    (":", ParserTokenKind::ReservedSymbol),
                    ("1", ParserTokenKind::Numeral),
                    (",", ParserTokenKind::ReservedSymbol),
                    (",", ParserTokenKind::ReservedSymbol),
                    ("timeout", ParserTokenKind::Identifier),
                    (":", ParserTokenKind::ReservedSymbol),
                    ("2", ParserTokenKind::Numeral),
                    (")", ParserTokenKind::ReservedSymbol),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("thesis", ParserTokenKind::ReservedWord),
                    ("by", ParserTokenKind::ReservedWord),
                    ("computation", ParserTokenKind::ReservedWord),
                    ("(", ParserTokenKind::ReservedSymbol),
                    ("steps", ParserTokenKind::Identifier),
                    (":", ParserTokenKind::ReservedSymbol),
                    (")", ParserTokenKind::ReservedSymbol),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("let", ParserTokenKind::ReservedWord),
                    ("x", ParserTokenKind::Identifier),
                    ("be", ParserTokenKind::ReservedWord),
                    ("set", ParserTokenKind::ReservedWord),
                    ("by", ParserTokenKind::ReservedWord),
                    ("computation", ParserTokenKind::ReservedWord),
                    ("(", ParserTokenKind::ReservedSymbol),
                    ("steps", ParserTokenKind::Identifier),
                    (":", ParserTokenKind::ReservedSymbol),
                    ("1", ParserTokenKind::Numeral),
                    (")", ParserTokenKind::ReservedSymbol),
                    (";", ParserTokenKind::ReservedSymbol),
                ],
            ),
            Vec::new(),
        ));

        assert!(
            output
                .diagnostics
                .iter()
                .all(|diagnostic| diagnostic.code == SyntaxDiagnosticCode::MalformedJustification),
            "only justification diagnostics should be reported: {:?}",
            output.diagnostics
        );
        assert!(
            output.diagnostics.len() >= 5,
            "expected each malformed justification shape to diagnose: {:?}",
            output.diagnostics
        );
        let ast = output
            .ast
            .expect("malformed justifications should recover an AST");
        assert_eq!(
            count_nodes(&ast, |kind| matches!(
                kind,
                SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::MissingProofStep)
            )),
            10
        );
        assert!(
            count_nodes(&ast, |kind| matches!(
                kind,
                SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::SkippedToken)
            )) >= 2
        );
        assert_eq!(
            count_nodes(&ast, |kind| matches!(
                kind,
                SurfaceNodeKind::JustificationClause
            )),
            12
        );
        assert_eq!(
            count_nodes(&ast, |kind| matches!(
                kind,
                SurfaceNodeKind::ComputationJustification
            )),
            3,
            "`let ... by computation` should recover as malformed references, not computation"
        );
        assert_eq!(ast.trivia().skipped_token_ranges().len(), 2);
        assert!(
            ast.trivia()
                .skipped_token_ranges()
                .iter()
                .all(|range| range.reason == SkippedTokenReason::Recovery)
        );
    }

    #[test]
    fn parser_recovers_let_by_before_following_statement() {
        let source_id = source_id(90);
        let output = parse(ParseRequest::new(
            source_id,
            Edition::new("2026"),
            token_sequence(
                source_id,
                &[
                    ("let", ParserTokenKind::ReservedWord),
                    ("x", ParserTokenKind::Identifier),
                    ("be", ParserTokenKind::ReservedWord),
                    ("set", ParserTokenKind::ReservedWord),
                    ("by", ParserTokenKind::ReservedWord),
                    ("A", ParserTokenKind::Identifier),
                    ("assume", ParserTokenKind::ReservedWord),
                    ("thesis", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                ],
            ),
            Vec::new(),
        ));

        assert_eq!(
            output
                .diagnostics
                .iter()
                .map(|diagnostic| &diagnostic.code)
                .collect::<Vec<_>>(),
            vec![&SyntaxDiagnosticCode::MissingSemicolon]
        );
        let ast = output.ast.expect("let-by recovery should keep an AST");
        let item_list = single_node(&ast, |kind| matches!(kind, SurfaceNodeKind::ItemList));
        assert_eq!(item_list.children.len(), 2);
        let let_item = ast.node(item_list.children[0]).unwrap();
        assert!(matches!(let_item.kind, SurfaceNodeKind::StatementItem));
        let assumption_item = ast.node(item_list.children[1]).unwrap();
        assert!(matches!(
            assumption_item.kind,
            SurfaceNodeKind::StatementItem
        ));
        assert_eq!(
            count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::LetStatement)),
            1
        );
        assert_eq!(
            count_nodes(&ast, |kind| matches!(
                kind,
                SurfaceNodeKind::JustificationClause
            )),
            1
        );
        assert_eq!(
            count_nodes(&ast, |kind| matches!(
                kind,
                SurfaceNodeKind::AssumptionStatement
            )),
            1
        );
    }

    #[test]
    fn parser_does_not_treat_later_set_statement_as_let_type_word() {
        let source_id = source_id(81);
        let output = parse(ParseRequest::new(
            source_id,
            Edition::new("2026"),
            token_sequence(
                source_id,
                &[
                    ("let", ParserTokenKind::ReservedWord),
                    ("x", ParserTokenKind::Identifier),
                    ("be", ParserTokenKind::ReservedWord),
                    ("set", ParserTokenKind::ReservedWord),
                    ("set", ParserTokenKind::ReservedWord),
                    ("f", ParserTokenKind::Identifier),
                    ("=", ParserTokenKind::ReservedSymbol),
                    ("x", ParserTokenKind::Identifier),
                    ("by", ParserTokenKind::ReservedWord),
                    ("A", ParserTokenKind::Identifier),
                    ("assume", ParserTokenKind::ReservedWord),
                    ("thesis", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                ],
            ),
            Vec::new(),
        ));

        assert!(
            output
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == SyntaxDiagnosticCode::MissingSemicolon),
            "missing semicolon before the second `set` should be diagnosed: {:?}",
            output.diagnostics
        );
        let ast = output
            .ast
            .expect("set-statement boundary recovery should keep an AST");
        let item_list = single_node(&ast, |kind| matches!(kind, SurfaceNodeKind::ItemList));
        assert_eq!(item_list.children.len(), 3);
        let let_item = ast.node(item_list.children[0]).unwrap();
        assert!(matches!(let_item.kind, SurfaceNodeKind::StatementItem));
        let set_item = ast.node(item_list.children[1]).unwrap();
        assert!(matches!(set_item.kind, SurfaceNodeKind::StatementItem));
        let assumption_item = ast.node(item_list.children[2]).unwrap();
        assert!(matches!(
            assumption_item.kind,
            SurfaceNodeKind::StatementItem
        ));
        assert_eq!(
            count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::SetStatement)),
            1
        );
        assert_eq!(
            count_nodes(&ast, |kind| matches!(
                kind,
                SurfaceNodeKind::AssumptionStatement
            )),
            1
        );
        assert_eq!(
            count_nodes(&ast, |kind| matches!(
                kind,
                SurfaceNodeKind::PlaceholderItem
            )),
            0
        );
    }

    #[test]
    fn parser_diagnoses_non_associative_formula_iff_chain() {
        let source_id = source_id(74);
        let tokens = token_sequence(
            source_id,
            &[
                ("theorem", ParserTokenKind::ReservedWord),
                ("IffChain", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("thesis", ParserTokenKind::ReservedWord),
                ("iff", ParserTokenKind::ReservedWord),
                ("contradiction", ParserTokenKind::ReservedWord),
                ("iff", ParserTokenKind::ReservedWord),
                ("thesis", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
            ],
        );
        let second_iff = tokens[6].span;
        let output = parse(ParseRequest::new(
            source_id,
            Edition::new("2026"),
            tokens,
            Vec::new(),
        ));

        assert_eq!(
            output
                .diagnostics
                .iter()
                .map(|diagnostic| &diagnostic.code)
                .collect::<Vec<_>>(),
            vec![&SyntaxDiagnosticCode::NonAssociativeOperatorChain]
        );
        assert_eq!(output.diagnostics[0].primary, second_iff);
        let ast = output.ast.expect("iff chain should keep an AST");
        assert_eq!(
            count_nodes(&ast, |kind| matches!(
                kind,
                SurfaceNodeKind::BinaryFormula(operator)
                    if operator.connective == SurfaceFormulaConnective::Iff
            )),
            2
        );
    }

    #[test]
    fn parser_recovers_missing_formula_operands_and_unmatched_formula_grouping() {
        let source_id = source_id(75);
        let output = parse(ParseRequest::new(
            source_id,
            Edition::new("2026"),
            token_sequence(
                source_id,
                &[
                    ("theorem", ParserTokenKind::ReservedWord),
                    ("MissingNot", ParserTokenKind::Identifier),
                    (":", ParserTokenKind::ReservedSymbol),
                    ("not", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("theorem", ParserTokenKind::ReservedWord),
                    ("MissingConnective", ParserTokenKind::Identifier),
                    (":", ParserTokenKind::ReservedSymbol),
                    ("x", ParserTokenKind::Identifier),
                    ("=", ParserTokenKind::ReservedSymbol),
                    ("y", ParserTokenKind::Identifier),
                    ("&", ParserTokenKind::ReservedSymbol),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("theorem", ParserTokenKind::ReservedWord),
                    ("MissingSt", ParserTokenKind::Identifier),
                    (":", ParserTokenKind::ReservedSymbol),
                    ("ex", ParserTokenKind::ReservedWord),
                    ("x", ParserTokenKind::Identifier),
                    ("st", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("theorem", ParserTokenKind::ReservedWord),
                    ("MissingHolds", ParserTokenKind::Identifier),
                    (":", ParserTokenKind::ReservedSymbol),
                    ("for", ParserTokenKind::ReservedWord),
                    ("x", ParserTokenKind::Identifier),
                    ("holds", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("theorem", ParserTokenKind::ReservedWord),
                    ("MissingHoldsKeyword", ParserTokenKind::Identifier),
                    (":", ParserTokenKind::ReservedSymbol),
                    ("for", ParserTokenKind::ReservedWord),
                    ("x", ParserTokenKind::Identifier),
                    ("thesis", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("theorem", ParserTokenKind::ReservedWord),
                    ("MissingStKeyword", ParserTokenKind::Identifier),
                    (":", ParserTokenKind::ReservedSymbol),
                    ("ex", ParserTokenKind::ReservedWord),
                    ("x", ParserTokenKind::Identifier),
                    ("thesis", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("theorem", ParserTokenKind::ReservedWord),
                    ("MissingParen", ParserTokenKind::Identifier),
                    (":", ParserTokenKind::ReservedSymbol),
                    ("(", ParserTokenKind::ReservedSymbol),
                    ("x", ParserTokenKind::Identifier),
                    ("=", ParserTokenKind::ReservedSymbol),
                    ("y", ParserTokenKind::Identifier),
                    (";", ParserTokenKind::ReservedSymbol),
                ],
            ),
            Vec::new(),
        ));

        assert_eq!(
            output
                .diagnostics
                .iter()
                .map(|diagnostic| &diagnostic.code)
                .collect::<Vec<_>>(),
            vec![
                &SyntaxDiagnosticCode::MalformedFormulaExpression,
                &SyntaxDiagnosticCode::MalformedFormulaExpression,
                &SyntaxDiagnosticCode::MalformedFormulaExpression,
                &SyntaxDiagnosticCode::MalformedFormulaExpression,
                &SyntaxDiagnosticCode::MalformedFormulaExpression,
                &SyntaxDiagnosticCode::MalformedFormulaExpression,
                &SyntaxDiagnosticCode::MalformedFormulaExpression,
            ]
        );
        let ast = output.ast.expect("formula recovery should keep an AST");
        assert_eq!(
            count_nodes(&ast, |kind| {
                matches!(
                    kind,
                    SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::MissingFormula)
                )
            }),
            6
        );
        assert_eq!(
            count_nodes(&ast, |kind| {
                matches!(
                    kind,
                    SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::UnmatchedOpeningDelimiter)
                )
            }),
            1
        );
    }

    #[test]
    fn parser_parses_theorem_tails_after_formulas_as_concrete_items() {
        let source_id = source_id(50);
        let output = parse(ParseRequest::new(
            source_id,
            Edition::new("2026"),
            token_sequence(
                source_id,
                &[
                    ("theorem", ParserTokenKind::ReservedWord),
                    ("ByTail", ParserTokenKind::Identifier),
                    (":", ParserTokenKind::ReservedSymbol),
                    ("x", ParserTokenKind::Identifier),
                    ("=", ParserTokenKind::ReservedSymbol),
                    ("y", ParserTokenKind::Identifier),
                    ("by", ParserTokenKind::ReservedWord),
                    ("A", ParserTokenKind::Identifier),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("theorem", ParserTokenKind::ReservedWord),
                    ("ProofTail", ParserTokenKind::Identifier),
                    (":", ParserTokenKind::ReservedSymbol),
                    ("x", ParserTokenKind::Identifier),
                    ("=", ParserTokenKind::ReservedSymbol),
                    ("y", ParserTokenKind::Identifier),
                    ("proof", ParserTokenKind::ReservedWord),
                    ("end", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("theorem", ParserTokenKind::ReservedWord),
                    ("ConnectiveByTail", ParserTokenKind::Identifier),
                    (":", ParserTokenKind::ReservedSymbol),
                    ("x", ParserTokenKind::Identifier),
                    ("=", ParserTokenKind::ReservedSymbol),
                    ("y", ParserTokenKind::Identifier),
                    ("&", ParserTokenKind::ReservedSymbol),
                    ("z", ParserTokenKind::Identifier),
                    ("=", ParserTokenKind::ReservedSymbol),
                    ("w", ParserTokenKind::Identifier),
                    ("by", ParserTokenKind::ReservedWord),
                    ("A", ParserTokenKind::Identifier),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("theorem", ParserTokenKind::ReservedWord),
                    ("PrefixByTail", ParserTokenKind::Identifier),
                    (":", ParserTokenKind::ReservedSymbol),
                    ("not", ParserTokenKind::ReservedWord),
                    ("x", ParserTokenKind::Identifier),
                    ("=", ParserTokenKind::ReservedSymbol),
                    ("y", ParserTokenKind::Identifier),
                    ("by", ParserTokenKind::ReservedWord),
                    ("A", ParserTokenKind::Identifier),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("theorem", ParserTokenKind::ReservedWord),
                    ("QuantifierProofTail", ParserTokenKind::Identifier),
                    (":", ParserTokenKind::ReservedSymbol),
                    ("for", ParserTokenKind::ReservedWord),
                    ("x", ParserTokenKind::Identifier),
                    ("holds", ParserTokenKind::ReservedWord),
                    ("thesis", ParserTokenKind::ReservedWord),
                    ("proof", ParserTokenKind::ReservedWord),
                    ("end", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("theorem", ParserTokenKind::ReservedWord),
                    ("ConstantByTail", ParserTokenKind::Identifier),
                    (":", ParserTokenKind::ReservedSymbol),
                    ("thesis", ParserTokenKind::ReservedWord),
                    ("by", ParserTokenKind::ReservedWord),
                    ("A", ParserTokenKind::Identifier),
                    (";", ParserTokenKind::ReservedSymbol),
                ],
            ),
            Vec::new(),
        ));

        assert!(
            output.diagnostics.is_empty(),
            "theorem proof/justification tails should parse as concrete items: {:?}",
            output.diagnostics
        );
        let ast = output.ast.expect("theorem tails should keep an AST");
        assert_eq!(
            count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::TheoremItem)),
            6
        );
        assert_eq!(
            count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::ProofBlock)),
            2
        );
        assert_eq!(
            count_nodes(&ast, |kind| matches!(
                kind,
                SurfaceNodeKind::JustificationClause
            )),
            4
        );
        let item_list = single_node(&ast, |kind| matches!(kind, SurfaceNodeKind::ItemList));
        assert_eq!(item_list.children.len(), 6);
        assert!(item_list.children.iter().all(|item| {
            let item = ast.node(*item).unwrap();
            matches!(item.kind, SurfaceNodeKind::TheoremItem)
                && direct_child_has_kind(&ast, item, |kind| {
                    matches!(kind, SurfaceNodeKind::FormulaExpression)
                })
        }));
    }

    #[test]
    fn parser_recovers_missing_user_predicate_chain_term() {
        let source_id = source_id(51);
        let output = parse(ParseRequest::new(
            source_id,
            Edition::new("2026"),
            token_sequence(
                source_id,
                &[
                    ("theorem", ParserTokenKind::ReservedWord),
                    ("T", ParserTokenKind::Identifier),
                    (":", ParserTokenKind::ReservedSymbol),
                    ("x", ParserTokenKind::Identifier),
                    ("divides", ParserTokenKind::UserSymbol),
                    ("y", ParserTokenKind::Identifier),
                    ("divides", ParserTokenKind::UserSymbol),
                    (";", ParserTokenKind::ReservedSymbol),
                ],
            ),
            Vec::new(),
        ));

        assert_eq!(
            output
                .diagnostics
                .iter()
                .map(|diagnostic| &diagnostic.code)
                .collect::<Vec<_>>(),
            vec![&SyntaxDiagnosticCode::MalformedTermExpression]
        );
        let ast = output.ast.expect("missing chain term should recover");
        assert_eq!(
            count_nodes(&ast, |kind| matches!(
                kind,
                SurfaceNodeKind::PredicateSegment
            )),
            2
        );
        assert_eq!(
            count_nodes(&ast, |kind| {
                matches!(
                    kind,
                    SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::MissingTerm)
                )
            }),
            1
        );
    }

    #[test]
    fn parser_recovers_missing_builtin_predicate_right_term() {
        let source_id = source_id(46);
        let output = parse(ParseRequest::new(
            source_id,
            Edition::new("2026"),
            token_sequence(
                source_id,
                &[
                    ("theorem", ParserTokenKind::ReservedWord),
                    ("T", ParserTokenKind::Identifier),
                    (":", ParserTokenKind::ReservedSymbol),
                    ("x", ParserTokenKind::Identifier),
                    ("=", ParserTokenKind::ReservedSymbol),
                    (";", ParserTokenKind::ReservedSymbol),
                ],
            ),
            Vec::new(),
        ));

        assert_eq!(
            output
                .diagnostics
                .iter()
                .map(|diagnostic| &diagnostic.code)
                .collect::<Vec<_>>(),
            vec![&SyntaxDiagnosticCode::MalformedTermExpression]
        );
        let ast = output
            .ast
            .expect("missing predicate operand should recover");
        assert_eq!(
            count_nodes(&ast, |kind| {
                matches!(kind, SurfaceNodeKind::BuiltinPredicateApplication)
            }),
            1
        );
        assert_eq!(
            count_nodes(&ast, |kind| {
                matches!(
                    kind,
                    SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::MissingTerm)
                )
            }),
            1
        );
    }

    #[test]
    fn parser_rejects_mixed_user_and_builtin_predicate_chain() {
        let source_id = source_id(47);
        let output = parse(ParseRequest::new(
            source_id,
            Edition::new("2026"),
            token_sequence(
                source_id,
                &[
                    ("theorem", ParserTokenKind::ReservedWord),
                    ("T", ParserTokenKind::Identifier),
                    (":", ParserTokenKind::ReservedSymbol),
                    ("x", ParserTokenKind::Identifier),
                    ("divides", ParserTokenKind::UserSymbol),
                    ("y", ParserTokenKind::Identifier),
                    ("=", ParserTokenKind::ReservedSymbol),
                    ("z", ParserTokenKind::Identifier),
                    (";", ParserTokenKind::ReservedSymbol),
                ],
            ),
            Vec::new(),
        ));

        assert_eq!(
            output
                .diagnostics
                .iter()
                .map(|diagnostic| &diagnostic.code)
                .collect::<Vec<_>>(),
            vec![&SyntaxDiagnosticCode::MalformedTermExpression]
        );
        let ast = output.ast.expect("mixed predicate chain should recover");
        assert_eq!(
            count_nodes(&ast, |kind| {
                matches!(kind, SurfaceNodeKind::PredicateApplication)
            }),
            1
        );
        assert_eq!(
            count_nodes(&ast, |kind| {
                matches!(
                    kind,
                    SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::SkippedToken)
                )
            }),
            1
        );
    }

    #[test]
    fn parser_recovers_missing_is_assertion_body() {
        let source_id = source_id(48);
        let output = parse(ParseRequest::new(
            source_id,
            Edition::new("2026"),
            token_sequence(
                source_id,
                &[
                    ("theorem", ParserTokenKind::ReservedWord),
                    ("T", ParserTokenKind::Identifier),
                    (":", ParserTokenKind::ReservedSymbol),
                    ("x", ParserTokenKind::Identifier),
                    ("is", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                ],
            ),
            Vec::new(),
        ));

        assert_eq!(
            output
                .diagnostics
                .iter()
                .map(|diagnostic| &diagnostic.code)
                .collect::<Vec<_>>(),
            vec![&SyntaxDiagnosticCode::MalformedTypeExpression]
        );
        let ast = output.ast.expect("missing is body should recover");
        assert_eq!(
            count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::IsAssertion)),
            1
        );
        assert_eq!(
            count_nodes(&ast, |kind| {
                matches!(
                    kind,
                    SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::MissingTypeExpression)
                )
            }),
            1
        );
    }

    #[test]
    fn parser_keeps_task17_from_tails_deferred() {
        let source_id = source_id(91);
        let output = parse(ParseRequest::new(
            source_id,
            Edition::new("2026"),
            token_sequence(
                source_id,
                &[
                    ("thesis", ParserTokenKind::ReservedWord),
                    ("from", ParserTokenKind::ReservedWord),
                    ("A", ParserTokenKind::Identifier),
                    (";", ParserTokenKind::ReservedSymbol),
                ],
            ),
            Vec::new(),
        ));

        assert!(
            output.diagnostics.is_empty(),
            "deferred `from` tails should stay on the legacy placeholder path: {:?}",
            output.diagnostics
        );
        let ast = output.ast.expect("deferred from tails should keep an AST");
        assert_eq!(
            count_nodes(&ast, |kind| matches!(
                kind,
                SurfaceNodeKind::JustificationClause
                    | SurfaceNodeKind::ReferenceList
                    | SurfaceNodeKind::ComputationJustification
            )),
            0
        );
        assert_eq!(
            count_nodes(&ast, |kind| matches!(
                kind,
                SurfaceNodeKind::CompactStatement
            )),
            0
        );
    }

    #[test]
    fn parser_parses_task18_consider_and_reconsider_nodes() {
        let source_id = source_id(93);
        let output = parse(ParseRequest::new(
            source_id,
            Edition::new("2026"),
            token_sequence(
                source_id,
                &[
                    ("consider", ParserTokenKind::ReservedWord),
                    ("x", ParserTokenKind::Identifier),
                    (",", ParserTokenKind::ReservedSymbol),
                    ("y", ParserTokenKind::Identifier),
                    ("being", ParserTokenKind::ReservedWord),
                    ("set", ParserTokenKind::ReservedWord),
                    ("such", ParserTokenKind::ReservedWord),
                    ("that", ParserTokenKind::ReservedWord),
                    ("A", ParserTokenKind::Identifier),
                    (":", ParserTokenKind::ReservedSymbol),
                    ("thesis", ParserTokenKind::ReservedWord),
                    ("and", ParserTokenKind::ReservedWord),
                    ("contradiction", ParserTokenKind::ReservedWord),
                    ("by", ParserTokenKind::ReservedWord),
                    ("A", ParserTokenKind::Identifier),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("consider", ParserTokenKind::ReservedWord),
                    ("z", ParserTokenKind::Identifier),
                    ("being", ParserTokenKind::ReservedWord),
                    ("set", ParserTokenKind::ReservedWord),
                    (",", ParserTokenKind::ReservedSymbol),
                    ("w", ParserTokenKind::Identifier),
                    ("being", ParserTokenKind::ReservedWord),
                    ("set", ParserTokenKind::ReservedWord),
                    ("such", ParserTokenKind::ReservedWord),
                    ("that", ParserTokenKind::ReservedWord),
                    ("thesis", ParserTokenKind::ReservedWord),
                    ("by", ParserTokenKind::ReservedWord),
                    ("B", ParserTokenKind::Identifier),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("reconsider", ParserTokenKind::ReservedWord),
                    ("x", ParserTokenKind::Identifier),
                    (",", ParserTokenKind::ReservedSymbol),
                    ("y", ParserTokenKind::Identifier),
                    ("=", ParserTokenKind::ReservedSymbol),
                    ("z", ParserTokenKind::Identifier),
                    ("as", ParserTokenKind::ReservedWord),
                    ("set", ParserTokenKind::ReservedWord),
                    ("by", ParserTokenKind::ReservedWord),
                    ("A", ParserTokenKind::Identifier),
                    (";", ParserTokenKind::ReservedSymbol),
                ],
            ),
            Vec::new(),
        ));

        assert!(
            output.diagnostics.is_empty(),
            "task-18 statements should parse without diagnostics: {:?}",
            output.diagnostics
        );
        let ast = output
            .ast
            .expect("consider/reconsider statements should keep an AST");
        let item_list = single_node(&ast, |kind| matches!(kind, SurfaceNodeKind::ItemList));
        assert_eq!(item_list.children.len(), 3);
        assert_eq!(
            count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::StatementItem)),
            3
        );
        assert_eq!(
            count_nodes(&ast, |kind| matches!(
                kind,
                SurfaceNodeKind::ConsiderStatement
            )),
            2
        );
        assert_eq!(
            count_nodes(&ast, |kind| matches!(
                kind,
                SurfaceNodeKind::ReconsiderStatement
            )),
            1
        );
        assert_eq!(
            count_nodes(&ast, |kind| matches!(
                kind,
                SurfaceNodeKind::QualifiedVariableSegment
            )),
            3
        );
        assert_eq!(
            count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::ConditionList)),
            2
        );
        assert_eq!(
            count_nodes(&ast, |kind| matches!(
                kind,
                SurfaceNodeKind::JustificationClause
            )),
            3
        );
        assert_eq!(
            count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::ReconsiderItem)),
            2
        );

        let shared_segment = ast
            .nodes()
            .iter()
            .find(|node| {
                matches!(node.kind, SurfaceNodeKind::QualifiedVariableSegment)
                    && direct_token_texts(&ast, node) == vec!["x", ",", "y", "being"]
            })
            .expect("consider should preserve shared-type variable list");
        assert!(
            structural_children(&ast, shared_segment)
                .iter()
                .any(|child| matches!(child.kind, SurfaceNodeKind::TypeExpression))
        );

        let reconsider_statement = single_node(&ast, |kind| {
            matches!(kind, SurfaceNodeKind::ReconsiderStatement)
        });
        assert_eq!(
            direct_token_texts(&ast, reconsider_statement),
            vec!["reconsider", ",", "as", ";"]
        );
        let reconsider_items = structural_children(&ast, reconsider_statement)
            .into_iter()
            .filter(|child| matches!(child.kind, SurfaceNodeKind::ReconsiderItem))
            .collect::<Vec<_>>();
        assert_eq!(direct_token_texts(&ast, reconsider_items[0]), vec!["x"]);
        assert_eq!(
            direct_token_texts(&ast, reconsider_items[1]),
            vec!["y", "="]
        );
    }

    #[test]
    fn parser_recovers_task18_consider_and_reconsider_gaps() {
        let source_id = source_id(94);
        let output = parse(ParseRequest::new(
            source_id,
            Edition::new("2026"),
            token_sequence(
                source_id,
                &[
                    ("consider", ParserTokenKind::ReservedWord),
                    ("such", ParserTokenKind::ReservedWord),
                    ("that", ParserTokenKind::ReservedWord),
                    ("thesis", ParserTokenKind::ReservedWord),
                    ("by", ParserTokenKind::ReservedWord),
                    ("A", ParserTokenKind::Identifier),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("consider", ParserTokenKind::ReservedWord),
                    ("x", ParserTokenKind::Identifier),
                    ("being", ParserTokenKind::ReservedWord),
                    ("set", ParserTokenKind::ReservedWord),
                    ("by", ParserTokenKind::ReservedWord),
                    ("A", ParserTokenKind::Identifier),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("consider", ParserTokenKind::ReservedWord),
                    ("x", ParserTokenKind::Identifier),
                    ("being", ParserTokenKind::ReservedWord),
                    ("set", ParserTokenKind::ReservedWord),
                    ("such", ParserTokenKind::ReservedWord),
                    ("that", ParserTokenKind::ReservedWord),
                    ("by", ParserTokenKind::ReservedWord),
                    ("A", ParserTokenKind::Identifier),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("consider", ParserTokenKind::ReservedWord),
                    ("x", ParserTokenKind::Identifier),
                    ("being", ParserTokenKind::ReservedWord),
                    ("set", ParserTokenKind::ReservedWord),
                    ("such", ParserTokenKind::ReservedWord),
                    ("that", ParserTokenKind::ReservedWord),
                    ("thesis", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("consider", ParserTokenKind::ReservedWord),
                    ("x", ParserTokenKind::Identifier),
                    ("being", ParserTokenKind::ReservedWord),
                    ("set", ParserTokenKind::ReservedWord),
                    ("such", ParserTokenKind::ReservedWord),
                    ("that", ParserTokenKind::ReservedWord),
                    ("thesis", ParserTokenKind::ReservedWord),
                    ("by", ParserTokenKind::ReservedWord),
                    ("computation", ParserTokenKind::ReservedWord),
                    ("(", ParserTokenKind::ReservedSymbol),
                    ("steps", ParserTokenKind::Identifier),
                    (":", ParserTokenKind::ReservedSymbol),
                    ("1", ParserTokenKind::Numeral),
                    (")", ParserTokenKind::ReservedSymbol),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("reconsider", ParserTokenKind::ReservedWord),
                    ("as", ParserTokenKind::ReservedWord),
                    ("set", ParserTokenKind::ReservedWord),
                    ("by", ParserTokenKind::ReservedWord),
                    ("A", ParserTokenKind::Identifier),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("reconsider", ParserTokenKind::ReservedWord),
                    ("x", ParserTokenKind::Identifier),
                    ("by", ParserTokenKind::ReservedWord),
                    ("A", ParserTokenKind::Identifier),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("reconsider", ParserTokenKind::ReservedWord),
                    ("x", ParserTokenKind::Identifier),
                    ("as", ParserTokenKind::ReservedWord),
                    ("by", ParserTokenKind::ReservedWord),
                    ("A", ParserTokenKind::Identifier),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("reconsider", ParserTokenKind::ReservedWord),
                    ("x", ParserTokenKind::Identifier),
                    ("as", ParserTokenKind::ReservedWord),
                    ("set", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("reconsider", ParserTokenKind::ReservedWord),
                    ("x", ParserTokenKind::Identifier),
                    ("=", ParserTokenKind::ReservedSymbol),
                    ("as", ParserTokenKind::ReservedWord),
                    ("set", ParserTokenKind::ReservedWord),
                    ("by", ParserTokenKind::ReservedWord),
                    ("A", ParserTokenKind::Identifier),
                    (";", ParserTokenKind::ReservedSymbol),
                ],
            ),
            Vec::new(),
        ));

        assert!(output.diagnostics.iter().any(|diagnostic| {
            diagnostic.code == SyntaxDiagnosticCode::MalformedFormulaExpression
        }));
        assert!(
            output
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == SyntaxDiagnosticCode::MalformedJustification)
        );
        assert!(
            output
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == SyntaxDiagnosticCode::MalformedTermExpression)
        );
        assert!(
            output
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == SyntaxDiagnosticCode::MalformedTypeExpression)
        );
        let ast = output
            .ast
            .expect("task-18 malformed statements should recover an AST");
        assert_eq!(
            count_nodes(&ast, |kind| matches!(
                kind,
                SurfaceNodeKind::ConsiderStatement
            )),
            5
        );
        assert_eq!(
            count_nodes(&ast, |kind| matches!(
                kind,
                SurfaceNodeKind::ReconsiderStatement
            )),
            5
        );
        assert_eq!(
            count_nodes(&ast, |kind| matches!(
                kind,
                SurfaceNodeKind::ComputationJustification
            )),
            0,
            "`by computation` should recover as malformed simple citation syntax"
        );
        assert!(
            count_nodes(&ast, |kind| matches!(
                kind,
                SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::MissingFormula)
            )) >= 2
        );
        assert!(
            count_nodes(&ast, |kind| matches!(
                kind,
                SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::MissingTerm)
            )) >= 2
        );
        assert!(
            count_nodes(&ast, |kind| matches!(
                kind,
                SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::MissingTypeExpression)
            )) >= 2
        );
        assert!(
            count_nodes(&ast, |kind| matches!(
                kind,
                SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::MissingProofStep)
            )) >= 3
        );
    }

    #[test]
    fn parser_parses_task19_conclusion_then_iterative_nodes() {
        let source_id = source_id(95);
        let output = parse(ParseRequest::new(
            source_id,
            Edition::new("2026"),
            token_sequence(
                source_id,
                &[
                    ("thus", ParserTokenKind::ReservedWord),
                    ("A", ParserTokenKind::Identifier),
                    (":", ParserTokenKind::ReservedSymbol),
                    ("thesis", ParserTokenKind::ReservedWord),
                    ("by", ParserTokenKind::ReservedWord),
                    ("A", ParserTokenKind::Identifier),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("hence", ParserTokenKind::ReservedWord),
                    ("thesis", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("thus", ParserTokenKind::ReservedWord),
                    ("thesis", ParserTokenKind::ReservedWord),
                    ("by", ParserTokenKind::ReservedWord),
                    ("computation", ParserTokenKind::ReservedWord),
                    ("(", ParserTokenKind::ReservedSymbol),
                    ("steps", ParserTokenKind::Identifier),
                    (":", ParserTokenKind::ReservedSymbol),
                    ("1", ParserTokenKind::Numeral),
                    (")", ParserTokenKind::ReservedSymbol),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("then", ParserTokenKind::ReservedWord),
                    ("thesis", ParserTokenKind::ReservedWord),
                    ("by", ParserTokenKind::ReservedWord),
                    ("A", ParserTokenKind::Identifier),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("then", ParserTokenKind::ReservedWord),
                    ("thus", ParserTokenKind::ReservedWord),
                    ("thesis", ParserTokenKind::ReservedWord),
                    ("by", ParserTokenKind::ReservedWord),
                    ("computation", ParserTokenKind::ReservedWord),
                    ("(", ParserTokenKind::ReservedSymbol),
                    ("steps", ParserTokenKind::Identifier),
                    (":", ParserTokenKind::ReservedSymbol),
                    ("1", ParserTokenKind::Numeral),
                    (")", ParserTokenKind::ReservedSymbol),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("A1", ParserTokenKind::Identifier),
                    (":", ParserTokenKind::ReservedSymbol),
                    ("x", ParserTokenKind::Identifier),
                    ("=", ParserTokenKind::ReservedSymbol),
                    ("y", ParserTokenKind::Identifier),
                    ("by", ParserTokenKind::ReservedWord),
                    ("A", ParserTokenKind::Identifier),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("then", ParserTokenKind::ReservedWord),
                    ("x", ParserTokenKind::Identifier),
                    ("=", ParserTokenKind::ReservedSymbol),
                    ("y", ParserTokenKind::Identifier),
                    ("by", ParserTokenKind::ReservedWord),
                    ("A", ParserTokenKind::Identifier),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("A2", ParserTokenKind::Identifier),
                    (":", ParserTokenKind::ReservedSymbol),
                    ("x", ParserTokenKind::Identifier),
                    ("=", ParserTokenKind::ReservedSymbol),
                    ("y", ParserTokenKind::Identifier),
                    ("by", ParserTokenKind::ReservedWord),
                    ("A", ParserTokenKind::Identifier),
                    (".=", ParserTokenKind::ReservedSymbol),
                    ("z", ParserTokenKind::Identifier),
                    ("by", ParserTokenKind::ReservedWord),
                    ("B", ParserTokenKind::Identifier),
                    (".=", ParserTokenKind::ReservedSymbol),
                    ("w", ParserTokenKind::Identifier),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("then", ParserTokenKind::ReservedWord),
                    ("x", ParserTokenKind::Identifier),
                    ("=", ParserTokenKind::ReservedSymbol),
                    ("y", ParserTokenKind::Identifier),
                    ("by", ParserTokenKind::ReservedWord),
                    ("A", ParserTokenKind::Identifier),
                    (".=", ParserTokenKind::ReservedSymbol),
                    ("z", ParserTokenKind::Identifier),
                    ("by", ParserTokenKind::ReservedWord),
                    ("B", ParserTokenKind::Identifier),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("then", ParserTokenKind::ReservedWord),
                    ("consider", ParserTokenKind::ReservedWord),
                    ("c", ParserTokenKind::Identifier),
                    ("being", ParserTokenKind::ReservedWord),
                    ("set", ParserTokenKind::ReservedWord),
                    ("such", ParserTokenKind::ReservedWord),
                    ("that", ParserTokenKind::ReservedWord),
                    ("thesis", ParserTokenKind::ReservedWord),
                    ("by", ParserTokenKind::ReservedWord),
                    ("A", ParserTokenKind::Identifier),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("then", ParserTokenKind::ReservedWord),
                    ("reconsider", ParserTokenKind::ReservedWord),
                    ("r", ParserTokenKind::Identifier),
                    ("as", ParserTokenKind::ReservedWord),
                    ("set", ParserTokenKind::ReservedWord),
                    ("by", ParserTokenKind::ReservedWord),
                    ("A", ParserTokenKind::Identifier),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("then", ParserTokenKind::ReservedWord),
                    ("per", ParserTokenKind::ReservedWord),
                    ("cases", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                ],
            ),
            Vec::new(),
        ));

        assert!(
            output.diagnostics.is_empty(),
            "task-19 statements should parse without diagnostics: {:?}",
            output.diagnostics
        );
        let ast = output.ast.expect("task-19 statements should keep an AST");
        let item_list = single_node(&ast, |kind| matches!(kind, SurfaceNodeKind::ItemList));
        assert_eq!(item_list.children.len(), 12);
        assert_eq!(
            count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::StatementItem)),
            12
        );
        assert_eq!(
            count_nodes(&ast, |kind| matches!(
                kind,
                SurfaceNodeKind::PlaceholderItem
            )),
            0,
            "`then per cases` is upgraded by task 20"
        );
        assert_eq!(
            count_nodes(&ast, |kind| matches!(
                kind,
                SurfaceNodeKind::ConclusionStatement
            )),
            4
        );
        assert_eq!(
            count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::ThenStatement)),
            7
        );
        assert_eq!(
            count_nodes(&ast, |kind| matches!(
                kind,
                SurfaceNodeKind::CompactStatement
            )),
            3,
            "`x = y by A;`, `then thesis by A;`, and `then x = y by A;` should stay compact"
        );
        assert_eq!(
            count_nodes(&ast, |kind| matches!(
                kind,
                SurfaceNodeKind::IterativeEqualityStatement
            )),
            2
        );
        assert_eq!(
            count_nodes(&ast, |kind| matches!(
                kind,
                SurfaceNodeKind::IterativeEqualityStep
            )),
            3
        );
        assert_eq!(
            count_nodes(&ast, |kind| matches!(
                kind,
                SurfaceNodeKind::ConsiderStatement
            )),
            1
        );
        assert_eq!(
            count_nodes(&ast, |kind| matches!(
                kind,
                SurfaceNodeKind::ReconsiderStatement
            )),
            1
        );
        assert_eq!(
            count_nodes(&ast, |kind| matches!(
                kind,
                SurfaceNodeKind::CaseReasoningStatement
            )),
            1
        );
        assert_eq!(
            count_nodes(&ast, |kind| matches!(
                kind,
                SurfaceNodeKind::ComputationJustification
            )),
            2,
            "conclusion statements should allow computation justifications"
        );

        assert!(
            ast.nodes().iter().any(|node| {
                matches!(node.kind, SurfaceNodeKind::CompactStatement)
                    && structural_children(&ast, node).iter().any(|child| {
                        matches!(child.kind, SurfaceNodeKind::Proposition)
                            && direct_token_texts(&ast, child) == vec!["A1", ":"]
                    })
            }),
            "labelled `x = y by A;` should dispatch to CompactStatement"
        );

        let then_child_kinds = ast
            .nodes()
            .iter()
            .filter(|node| matches!(node.kind, SurfaceNodeKind::ThenStatement))
            .flat_map(|node| structural_children(&ast, node))
            .map(|child| &child.kind)
            .collect::<Vec<_>>();
        assert!(
            then_child_kinds
                .iter()
                .any(|kind| matches!(kind, SurfaceNodeKind::ConclusionStatement)),
            "`then` should wrap conclusion statements"
        );
        assert!(
            then_child_kinds
                .iter()
                .any(|kind| matches!(kind, SurfaceNodeKind::ReconsiderStatement)),
            "`then` should wrap reconsider statements"
        );
        assert!(
            then_child_kinds
                .iter()
                .any(|kind| matches!(kind, SurfaceNodeKind::CaseReasoningStatement)),
            "`then` should wrap case reasoning statements"
        );

        let then_compact_token_texts = ast
            .nodes()
            .iter()
            .filter(|node| matches!(node.kind, SurfaceNodeKind::ThenStatement))
            .flat_map(|node| structural_children(&ast, node))
            .filter(|child| matches!(child.kind, SurfaceNodeKind::CompactStatement))
            .map(|child| subtree_token_texts(&ast, child))
            .collect::<Vec<_>>();
        assert!(
            then_compact_token_texts.iter().any(|tokens| tokens
                .iter()
                .map(String::as_str)
                .eq(["x", "=", "y", "by", "A", ";"])),
            "`then x = y by A;` should dispatch to a compact child"
        );
    }

    #[test]
    fn parser_recovers_task19_conclusion_then_iterative_gaps() {
        let source_id = source_id(96);
        let output = parse(ParseRequest::new(
            source_id,
            Edition::new("2026"),
            token_sequence(
                source_id,
                &[
                    ("then", ParserTokenKind::ReservedWord),
                    ("let", ParserTokenKind::ReservedWord),
                    ("x", ParserTokenKind::Identifier),
                    ("be", ParserTokenKind::ReservedWord),
                    ("set", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("thus", ParserTokenKind::ReservedWord),
                    ("by", ParserTokenKind::ReservedWord),
                    ("A", ParserTokenKind::Identifier),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("x", ParserTokenKind::Identifier),
                    ("=", ParserTokenKind::ReservedSymbol),
                    ("by", ParserTokenKind::ReservedWord),
                    ("A", ParserTokenKind::Identifier),
                    (".=", ParserTokenKind::ReservedSymbol),
                    ("z", ParserTokenKind::Identifier),
                    ("by", ParserTokenKind::ReservedWord),
                    ("B", ParserTokenKind::Identifier),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("x", ParserTokenKind::Identifier),
                    ("=", ParserTokenKind::ReservedSymbol),
                    ("y", ParserTokenKind::Identifier),
                    ("by", ParserTokenKind::ReservedWord),
                    ("A", ParserTokenKind::Identifier),
                    (".=", ParserTokenKind::ReservedSymbol),
                    ("by", ParserTokenKind::ReservedWord),
                    ("B", ParserTokenKind::Identifier),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("x", ParserTokenKind::Identifier),
                    ("=", ParserTokenKind::ReservedSymbol),
                    ("y", ParserTokenKind::Identifier),
                    ("by", ParserTokenKind::ReservedWord),
                    ("A", ParserTokenKind::Identifier),
                    (".=", ParserTokenKind::ReservedSymbol),
                    ("z", ParserTokenKind::Identifier),
                    ("by", ParserTokenKind::ReservedWord),
                    ("computation", ParserTokenKind::ReservedWord),
                    ("(", ParserTokenKind::ReservedSymbol),
                    ("steps", ParserTokenKind::Identifier),
                    (":", ParserTokenKind::ReservedSymbol),
                    ("1", ParserTokenKind::Numeral),
                    (")", ParserTokenKind::ReservedSymbol),
                    (";", ParserTokenKind::ReservedSymbol),
                ],
            ),
            Vec::new(),
        ));

        assert!(output.diagnostics.iter().any(|diagnostic| {
            diagnostic.code == SyntaxDiagnosticCode::MalformedFormulaExpression
        }));
        assert!(
            output
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == SyntaxDiagnosticCode::MalformedTermExpression)
        );
        assert!(
            output
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == SyntaxDiagnosticCode::MalformedJustification)
        );
        assert!(
            output
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == SyntaxDiagnosticCode::MissingSemicolon)
        );
        let ast = output
            .ast
            .expect("task-19 malformed statements should recover an AST");
        assert_eq!(
            count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::ThenStatement)),
            1
        );
        assert_eq!(
            count_nodes(&ast, |kind| matches!(
                kind,
                SurfaceNodeKind::ConclusionStatement
            )),
            1
        );
        assert_eq!(
            count_nodes(&ast, |kind| matches!(
                kind,
                SurfaceNodeKind::IterativeEqualityStatement
            )),
            3
        );
        assert_eq!(
            count_nodes(&ast, |kind| matches!(
                kind,
                SurfaceNodeKind::IterativeEqualityStep
            )),
            3
        );
        assert_eq!(
            count_nodes(&ast, |kind| matches!(
                kind,
                SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::MissingStatement)
            )),
            1
        );
        assert_eq!(
            count_nodes(&ast, |kind| matches!(
                kind,
                SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::MissingFormula)
            )),
            1
        );
        assert!(
            count_nodes(&ast, |kind| matches!(
                kind,
                SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::MissingTerm)
            )) >= 2
        );
        assert_eq!(
            count_nodes(&ast, |kind| matches!(
                kind,
                SurfaceNodeKind::ComputationJustification
            )),
            0,
            "`by computation` should recover as malformed simple citation syntax in iterative equality"
        );
    }

    #[test]
    fn parser_parses_task20_block_statement_nodes() {
        let source_id = source_id(97);
        let output = parse(ParseRequest::new(
            source_id,
            Edition::new("2026"),
            token_sequence(
                source_id,
                &[
                    ("A1", ParserTokenKind::Identifier),
                    (":", ParserTokenKind::ReservedSymbol),
                    ("now", ParserTokenKind::ReservedWord),
                    ("assume", ParserTokenKind::ReservedWord),
                    ("thesis", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("hereby", ParserTokenKind::ReservedWord),
                    ("thus", ParserTokenKind::ReservedWord),
                    ("thesis", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("end", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("end", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("hereby", ParserTokenKind::ReservedWord),
                    ("thus", ParserTokenKind::ReservedWord),
                    ("thesis", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("end", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("per", ParserTokenKind::ReservedWord),
                    ("cases", ParserTokenKind::ReservedWord),
                    ("by", ParserTokenKind::ReservedWord),
                    ("A", ParserTokenKind::Identifier),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("case", ParserTokenKind::ReservedWord),
                    ("C1", ParserTokenKind::Identifier),
                    (":", ParserTokenKind::ReservedSymbol),
                    ("thesis", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("thus", ParserTokenKind::ReservedWord),
                    ("thesis", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("end", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("case", ParserTokenKind::ReservedWord),
                    ("that", ParserTokenKind::ReservedWord),
                    ("thesis", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("hence", ParserTokenKind::ReservedWord),
                    ("thesis", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("end", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("per", ParserTokenKind::ReservedWord),
                    ("cases", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("suppose", ParserTokenKind::ReservedWord),
                    ("S1", ParserTokenKind::Identifier),
                    (":", ParserTokenKind::ReservedSymbol),
                    ("thesis", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("thus", ParserTokenKind::ReservedWord),
                    ("thesis", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("end", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("suppose", ParserTokenKind::ReservedWord),
                    ("that", ParserTokenKind::ReservedWord),
                    ("thesis", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("then", ParserTokenKind::ReservedWord),
                    ("thesis", ParserTokenKind::ReservedWord),
                    ("by", ParserTokenKind::ReservedWord),
                    ("A", ParserTokenKind::Identifier),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("end", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("then", ParserTokenKind::ReservedWord),
                    ("per", ParserTokenKind::ReservedWord),
                    ("cases", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("case", ParserTokenKind::ReservedWord),
                    ("thesis", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("thus", ParserTokenKind::ReservedWord),
                    ("thesis", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("end", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                ],
            ),
            Vec::new(),
        ));

        assert!(
            output.diagnostics.is_empty(),
            "task-20 block statements should parse without diagnostics: {:?}",
            output.diagnostics
        );
        let ast = output
            .ast
            .expect("task-20 block statements should keep an AST");
        let item_list = single_node(&ast, |kind| matches!(kind, SurfaceNodeKind::ItemList));
        assert_eq!(item_list.children.len(), 5);
        assert_eq!(
            count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::NowStatement)),
            1
        );
        assert_eq!(
            count_nodes(&ast, |kind| matches!(
                kind,
                SurfaceNodeKind::HerebyStatement
            )),
            2
        );
        assert_eq!(
            count_nodes(&ast, |kind| matches!(
                kind,
                SurfaceNodeKind::CaseReasoningStatement
            )),
            3
        );
        assert_eq!(
            count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::CaseItem)),
            3
        );
        assert_eq!(
            count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::SupposeItem)),
            2
        );
        assert_eq!(
            count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::ThenStatement)),
            2,
            "`then thesis by A;` and `then per cases; ...` should both be represented"
        );
        assert!(
            ast.nodes().iter().any(|node| {
                matches!(node.kind, SurfaceNodeKind::CaseItem)
                    && structural_children(&ast, node)
                        .iter()
                        .any(|child| matches!(child.kind, SurfaceNodeKind::ConditionList))
            }),
            "`case that thesis;` should preserve a condition list branch header"
        );
        assert!(
            ast.nodes().iter().any(|node| {
                matches!(node.kind, SurfaceNodeKind::SupposeItem)
                    && structural_children(&ast, node)
                        .iter()
                        .any(|child| matches!(child.kind, SurfaceNodeKind::ConditionList))
            }),
            "`suppose that thesis;` should preserve a condition list branch header"
        );
        assert!(
            ast.nodes().iter().any(|node| {
                matches!(node.kind, SurfaceNodeKind::ThenStatement)
                    && structural_children(&ast, node)
                        .iter()
                        .any(|child| matches!(child.kind, SurfaceNodeKind::CaseReasoningStatement))
            }),
            "`then per cases` should wrap case reasoning"
        );
    }

    #[test]
    fn parser_recovers_task20_block_statement_gaps() {
        let source_id = source_id(98);
        let output = parse(ParseRequest::new(
            source_id,
            Edition::new("2026"),
            token_sequence(
                source_id,
                &[
                    ("then", ParserTokenKind::ReservedWord),
                    ("now", ParserTokenKind::ReservedWord),
                    ("end", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("then", ParserTokenKind::ReservedWord),
                    ("hereby", ParserTokenKind::ReservedWord),
                    ("end", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("per", ParserTokenKind::ReservedWord),
                    ("cases", ParserTokenKind::ReservedWord),
                    ("by", ParserTokenKind::ReservedWord),
                    ("computation", ParserTokenKind::ReservedWord),
                    ("(", ParserTokenKind::ReservedSymbol),
                    ("steps", ParserTokenKind::Identifier),
                    (":", ParserTokenKind::ReservedSymbol),
                    ("1", ParserTokenKind::Numeral),
                    (")", ParserTokenKind::ReservedSymbol),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("per", ParserTokenKind::ReservedWord),
                    ("cases", ParserTokenKind::ReservedWord),
                    ("by", ParserTokenKind::ReservedWord),
                    ("A", ParserTokenKind::Identifier),
                    ("case", ParserTokenKind::ReservedWord),
                    ("thesis", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("thus", ParserTokenKind::ReservedWord),
                    ("thesis", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("end", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("per", ParserTokenKind::ReservedWord),
                    ("cases", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("case", ParserTokenKind::ReservedWord),
                    ("thesis", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("junk", ParserTokenKind::Identifier),
                    ("case", ParserTokenKind::ReservedWord),
                    ("thesis", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("thus", ParserTokenKind::ReservedWord),
                    ("thesis", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("end", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("per", ParserTokenKind::ReservedWord),
                    ("cases", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("case", ParserTokenKind::ReservedWord),
                    ("thesis", ParserTokenKind::ReservedWord),
                    ("thus", ParserTokenKind::ReservedWord),
                    ("thesis", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("end", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("per", ParserTokenKind::ReservedWord),
                    ("cases", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("case", ParserTokenKind::ReservedWord),
                    ("thesis", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("thus", ParserTokenKind::ReservedWord),
                    ("thesis", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("suppose", ParserTokenKind::ReservedWord),
                    ("thesis", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                ],
            ),
            Vec::new(),
        ));

        assert!(output.diagnostics.iter().any(|diagnostic| {
            diagnostic.code == SyntaxDiagnosticCode::MalformedFormulaExpression
        }));
        assert!(
            output
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == SyntaxDiagnosticCode::MalformedJustification)
        );
        assert!(
            output
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == SyntaxDiagnosticCode::MissingSemicolon)
        );
        assert!(
            output
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == SyntaxDiagnosticCode::MissingEnd)
        );
        let ast = output
            .ast
            .expect("task-20 malformed blocks should recover an AST");
        assert!(
            count_nodes(&ast, |kind| matches!(
                kind,
                SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::MissingStatement)
            )) >= 2,
            "`then now` and `then hereby` should stay non-linkable"
        );
        assert!(
            count_nodes(&ast, |kind| matches!(
                kind,
                SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::MissingProofStep)
            )) >= 1,
            "`per cases by computation(...)` should recover as malformed simple citation syntax"
        );
        assert!(
            count_nodes(&ast, |kind| matches!(
                kind,
                SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::MissingEnd)
            )) >= 1
        );
        assert_eq!(
            count_nodes(&ast, |kind| matches!(
                kind,
                SurfaceNodeKind::ComputationJustification
            )),
            0,
            "case reasoning uses simple justifications only"
        );
        assert!(
            count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::CaseItem)) >= 5,
            "`per cases by A` with a missing header semicolon should still preserve the following case branch"
        );
        assert!(
            ast.nodes().iter().any(|node| {
                matches!(node.kind, SurfaceNodeKind::CaseReasoningStatement)
                    && structural_children(&ast, node)
                        .iter()
                        .filter(|child| matches!(child.kind, SurfaceNodeKind::CaseItem))
                        .count()
                        >= 2
            }),
            "malformed branch-body tokens should not swallow the following `case` header"
        );
        assert_eq!(
            count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::SupposeItem)),
            0,
            "mixed `suppose` branches should stay outside the existing case-list node"
        );
    }

    #[test]
    fn parser_recovers_task20_missing_now_hereby_ends() {
        let source_id = source_id(167);
        let output = parse(ParseRequest::new(
            source_id,
            Edition::new("2026"),
            token_sequence(
                source_id,
                &[
                    ("now", ParserTokenKind::ReservedWord),
                    ("thus", ParserTokenKind::ReservedWord),
                    ("thesis", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("hereby", ParserTokenKind::ReservedWord),
                    ("thus", ParserTokenKind::ReservedWord),
                    ("thesis", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                ],
            ),
            Vec::new(),
        ));

        assert!(
            output
                .diagnostics
                .iter()
                .filter(|diagnostic| diagnostic.code == SyntaxDiagnosticCode::MissingEnd)
                .count()
                >= 2,
            "missing `end` diagnostics should be emitted for both `now` and nested `hereby`"
        );
        let ast = output
            .ast
            .expect("missing task-20 block ends should recover an AST");
        assert_eq!(
            count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::NowStatement)),
            1
        );
        assert_eq!(
            count_nodes(&ast, |kind| matches!(
                kind,
                SurfaceNodeKind::HerebyStatement
            )),
            1
        );
        assert!(
            count_nodes(&ast, |kind| matches!(
                kind,
                SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::MissingEnd)
            )) >= 2
        );
    }

    #[test]
    fn parser_parses_task21_inline_definition_nodes() {
        let source_id = source_id(168);
        let output = parse(ParseRequest::new(
            source_id,
            Edition::new("2026"),
            token_sequence(
                source_id,
                &[
                    ("deffunc", ParserTokenKind::ReservedWord),
                    ("F", ParserTokenKind::Identifier),
                    ("(", ParserTokenKind::ReservedSymbol),
                    ("x", ParserTokenKind::Identifier),
                    ("be", ParserTokenKind::ReservedWord),
                    ("set", ParserTokenKind::ReservedWord),
                    (",", ParserTokenKind::ReservedSymbol),
                    ("y", ParserTokenKind::Identifier),
                    ("being", ParserTokenKind::ReservedWord),
                    ("set", ParserTokenKind::ReservedWord),
                    (")", ParserTokenKind::ReservedSymbol),
                    ("->", ParserTokenKind::ReservedSymbol),
                    ("set", ParserTokenKind::ReservedWord),
                    ("equals", ParserTokenKind::ReservedWord),
                    ("x", ParserTokenKind::Identifier),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("deffunc", ParserTokenKind::ReservedWord),
                    ("C", ParserTokenKind::Identifier),
                    ("(", ParserTokenKind::ReservedSymbol),
                    (")", ParserTokenKind::ReservedSymbol),
                    ("->", ParserTokenKind::ReservedSymbol),
                    ("set", ParserTokenKind::ReservedWord),
                    ("equals", ParserTokenKind::ReservedWord),
                    ("42", ParserTokenKind::Numeral),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("defpred", ParserTokenKind::ReservedWord),
                    ("P", ParserTokenKind::Identifier),
                    ("(", ParserTokenKind::ReservedSymbol),
                    ("x", ParserTokenKind::Identifier),
                    ("be", ParserTokenKind::ReservedWord),
                    ("set", ParserTokenKind::ReservedWord),
                    (")", ParserTokenKind::ReservedSymbol),
                    ("means", ParserTokenKind::ReservedWord),
                    ("thesis", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("defpred", ParserTokenKind::ReservedWord),
                    ("Trivial", ParserTokenKind::Identifier),
                    ("(", ParserTokenKind::ReservedSymbol),
                    (")", ParserTokenKind::ReservedSymbol),
                    ("means", ParserTokenKind::ReservedWord),
                    ("thesis", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("now", ParserTokenKind::ReservedWord),
                    ("deffunc", ParserTokenKind::ReservedWord),
                    ("H", ParserTokenKind::Identifier),
                    ("(", ParserTokenKind::ReservedSymbol),
                    ("y", ParserTokenKind::Identifier),
                    ("being", ParserTokenKind::ReservedWord),
                    ("set", ParserTokenKind::ReservedWord),
                    (")", ParserTokenKind::ReservedSymbol),
                    ("->", ParserTokenKind::ReservedSymbol),
                    ("set", ParserTokenKind::ReservedWord),
                    ("equals", ParserTokenKind::ReservedWord),
                    ("y", ParserTokenKind::Identifier),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("defpred", ParserTokenKind::ReservedWord),
                    ("Q", ParserTokenKind::Identifier),
                    ("(", ParserTokenKind::ReservedSymbol),
                    (")", ParserTokenKind::ReservedSymbol),
                    ("means", ParserTokenKind::ReservedWord),
                    ("thesis", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("end", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                ],
            ),
            Vec::new(),
        ));

        assert!(
            output.diagnostics.is_empty(),
            "task-21 inline definitions should parse without diagnostics: {:?}",
            output.diagnostics
        );
        let ast = output
            .ast
            .expect("task-21 inline definitions should keep an AST");
        let item_list = single_node(&ast, |kind| matches!(kind, SurfaceNodeKind::ItemList));
        assert_eq!(item_list.children.len(), 5);
        assert_eq!(
            count_nodes(&ast, |kind| matches!(
                kind,
                SurfaceNodeKind::InlineFunctorDefinition
            )),
            3
        );
        assert_eq!(
            count_nodes(&ast, |kind| matches!(
                kind,
                SurfaceNodeKind::InlinePredicateDefinition
            )),
            3
        );
        assert_eq!(
            count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::TypedParameter)),
            4
        );
        assert_eq!(
            count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::NowStatement)),
            1,
            "reasoning bodies should accept local inline definitions"
        );
        assert_eq!(
            count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::ThenStatement)),
            0,
            "inline definitions are standalone statements, not linkable children"
        );
    }

    #[test]
    fn parser_recovers_task21_inline_definition_gaps() {
        let source_id = source_id(169);
        let output = parse(ParseRequest::new(
            source_id,
            Edition::new("2026"),
            token_sequence(
                source_id,
                &[
                    ("then", ParserTokenKind::ReservedWord),
                    ("deffunc", ParserTokenKind::ReservedWord),
                    ("F", ParserTokenKind::Identifier),
                    ("(", ParserTokenKind::ReservedSymbol),
                    (")", ParserTokenKind::ReservedSymbol),
                    ("->", ParserTokenKind::ReservedSymbol),
                    ("set", ParserTokenKind::ReservedWord),
                    ("equals", ParserTokenKind::ReservedWord),
                    ("x", ParserTokenKind::Identifier),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("then", ParserTokenKind::ReservedWord),
                    ("defpred", ParserTokenKind::ReservedWord),
                    ("P", ParserTokenKind::Identifier),
                    ("(", ParserTokenKind::ReservedSymbol),
                    (")", ParserTokenKind::ReservedSymbol),
                    ("means", ParserTokenKind::ReservedWord),
                    ("thesis", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("deffunc", ParserTokenKind::ReservedWord),
                    ("(", ParserTokenKind::ReservedSymbol),
                    (")", ParserTokenKind::ReservedSymbol),
                    ("->", ParserTokenKind::ReservedSymbol),
                    ("set", ParserTokenKind::ReservedWord),
                    ("equals", ParserTokenKind::ReservedWord),
                    ("x", ParserTokenKind::Identifier),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("deffunc", ParserTokenKind::ReservedWord),
                    ("N", ParserTokenKind::Identifier),
                    ("x", ParserTokenKind::Identifier),
                    ("be", ParserTokenKind::ReservedWord),
                    ("set", ParserTokenKind::ReservedWord),
                    ("->", ParserTokenKind::ReservedSymbol),
                    ("set", ParserTokenKind::ReservedWord),
                    ("equals", ParserTokenKind::ReservedWord),
                    ("x", ParserTokenKind::Identifier),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("deffunc", ParserTokenKind::ReservedWord),
                    ("G", ParserTokenKind::Identifier),
                    ("(", ParserTokenKind::ReservedSymbol),
                    ("x", ParserTokenKind::Identifier),
                    ("Nat", ParserTokenKind::Identifier),
                    (")", ParserTokenKind::ReservedSymbol),
                    ("->", ParserTokenKind::ReservedSymbol),
                    ("set", ParserTokenKind::ReservedWord),
                    ("equals", ParserTokenKind::ReservedWord),
                    ("x", ParserTokenKind::Identifier),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("deffunc", ParserTokenKind::ReservedWord),
                    ("H", ParserTokenKind::Identifier),
                    ("(", ParserTokenKind::ReservedSymbol),
                    ("x", ParserTokenKind::Identifier),
                    ("be", ParserTokenKind::ReservedWord),
                    (")", ParserTokenKind::ReservedSymbol),
                    ("->", ParserTokenKind::ReservedSymbol),
                    ("set", ParserTokenKind::ReservedWord),
                    ("equals", ParserTokenKind::ReservedWord),
                    ("x", ParserTokenKind::Identifier),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("deffunc", ParserTokenKind::ReservedWord),
                    ("I", ParserTokenKind::Identifier),
                    ("(", ParserTokenKind::ReservedSymbol),
                    ("x", ParserTokenKind::Identifier),
                    ("be", ParserTokenKind::ReservedWord),
                    ("set", ParserTokenKind::ReservedWord),
                    ("->", ParserTokenKind::ReservedSymbol),
                    ("set", ParserTokenKind::ReservedWord),
                    ("equals", ParserTokenKind::ReservedWord),
                    ("x", ParserTokenKind::Identifier),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("deffunc", ParserTokenKind::ReservedWord),
                    ("J", ParserTokenKind::Identifier),
                    ("(", ParserTokenKind::ReservedSymbol),
                    (")", ParserTokenKind::ReservedSymbol),
                    ("set", ParserTokenKind::ReservedWord),
                    ("equals", ParserTokenKind::ReservedWord),
                    ("x", ParserTokenKind::Identifier),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("deffunc", ParserTokenKind::ReservedWord),
                    ("K", ParserTokenKind::Identifier),
                    ("(", ParserTokenKind::ReservedSymbol),
                    (")", ParserTokenKind::ReservedSymbol),
                    ("->", ParserTokenKind::ReservedSymbol),
                    ("equals", ParserTokenKind::ReservedWord),
                    ("x", ParserTokenKind::Identifier),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("deffunc", ParserTokenKind::ReservedWord),
                    ("L", ParserTokenKind::Identifier),
                    ("(", ParserTokenKind::ReservedSymbol),
                    (")", ParserTokenKind::ReservedSymbol),
                    ("->", ParserTokenKind::ReservedSymbol),
                    ("set", ParserTokenKind::ReservedWord),
                    ("x", ParserTokenKind::Identifier),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("deffunc", ParserTokenKind::ReservedWord),
                    ("M", ParserTokenKind::Identifier),
                    ("(", ParserTokenKind::ReservedSymbol),
                    (")", ParserTokenKind::ReservedSymbol),
                    ("->", ParserTokenKind::ReservedSymbol),
                    ("set", ParserTokenKind::ReservedWord),
                    ("equals", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("defpred", ParserTokenKind::ReservedWord),
                    ("R", ParserTokenKind::Identifier),
                    ("(", ParserTokenKind::ReservedSymbol),
                    (")", ParserTokenKind::ReservedSymbol),
                    ("thesis", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("defpred", ParserTokenKind::ReservedWord),
                    ("U", ParserTokenKind::Identifier),
                    ("x", ParserTokenKind::Identifier),
                    ("be", ParserTokenKind::ReservedWord),
                    ("set", ParserTokenKind::ReservedWord),
                    ("means", ParserTokenKind::ReservedWord),
                    ("thesis", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("defpred", ParserTokenKind::ReservedWord),
                    ("S", ParserTokenKind::Identifier),
                    ("(", ParserTokenKind::ReservedSymbol),
                    (")", ParserTokenKind::ReservedSymbol),
                    ("means", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("defpred", ParserTokenKind::ReservedWord),
                    ("T", ParserTokenKind::Identifier),
                    ("(", ParserTokenKind::ReservedSymbol),
                    (")", ParserTokenKind::ReservedSymbol),
                    ("means", ParserTokenKind::ReservedWord),
                    ("thesis", ParserTokenKind::ReservedWord),
                ],
            ),
            Vec::new(),
        ));

        assert!(output.diagnostics.iter().any(|diagnostic| {
            diagnostic.code == SyntaxDiagnosticCode::MalformedTermExpression
        }));
        assert!(
            output
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == SyntaxDiagnosticCode::MalformedTypeExpression)
        );
        assert!(output.diagnostics.iter().any(|diagnostic| {
            diagnostic.code == SyntaxDiagnosticCode::MalformedFormulaExpression
        }));
        assert!(
            output
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == SyntaxDiagnosticCode::MissingSemicolon)
        );
        let ast = output
            .ast
            .expect("task-21 malformed inline definitions should recover an AST");
        assert!(
            count_nodes(&ast, |kind| matches!(
                kind,
                SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::MissingStatement)
            )) >= 2,
            "`then deffunc` and `then defpred` should remain non-linkable"
        );
        assert!(
            count_nodes(&ast, |kind| matches!(
                kind,
                SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::MissingTerm)
            )) >= 2
        );
        assert!(
            count_nodes(&ast, |kind| matches!(
                kind,
                SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::MissingTypeExpression)
            )) >= 2
        );
        assert!(
            count_nodes(&ast, |kind| matches!(
                kind,
                SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::MissingFormula)
            )) >= 1
        );
        assert!(
            count_nodes(&ast, |kind| matches!(
                kind,
                SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::UnmatchedOpeningDelimiter)
            )) >= 1
        );
        assert!(
            count_nodes(&ast, |kind| matches!(
                kind,
                SurfaceNodeKind::InlineFunctorDefinition
            )) >= 8
        );
        assert!(
            count_nodes(&ast, |kind| matches!(
                kind,
                SurfaceNodeKind::InlinePredicateDefinition
            )) >= 4
        );
        assert!(
            ast.nodes().iter().any(|node| {
                matches!(node.kind, SurfaceNodeKind::InlineFunctorDefinition)
                    && subtree_token_texts(&ast, node)
                        .iter()
                        .map(String::as_str)
                        .eq([
                            "deffunc", "N", "x", "be", "set", "->", "set", "equals", "x", ";",
                        ])
                    && structural_children(&ast, node)
                        .iter()
                        .any(|child| matches!(child.kind, SurfaceNodeKind::TypedParameter))
            }),
            "missing `(` in `deffunc N x be set -> ...` should preserve the typed parameter"
        );
        assert!(
            ast.nodes().iter().any(|node| {
                matches!(node.kind, SurfaceNodeKind::InlinePredicateDefinition)
                    && subtree_token_texts(&ast, node)
                        .iter()
                        .map(String::as_str)
                        .eq(["defpred", "U", "x", "be", "set", "means", "thesis", ";"])
                    && structural_children(&ast, node)
                        .iter()
                        .any(|child| matches!(child.kind, SurfaceNodeKind::TypedParameter))
            }),
            "missing `(` in `defpred U x be set means ...` should preserve the typed parameter"
        );
    }

    #[test]
    fn parser_recovers_task21_inline_definition_before_case_branch() {
        let source_id = source_id(170);
        let output = parse(ParseRequest::new(
            source_id,
            Edition::new("2026"),
            token_sequence(
                source_id,
                &[
                    ("per", ParserTokenKind::ReservedWord),
                    ("cases", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("case", ParserTokenKind::ReservedWord),
                    ("thesis", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("deffunc", ParserTokenKind::ReservedWord),
                    ("Bad", ParserTokenKind::Identifier),
                    ("(", ParserTokenKind::ReservedSymbol),
                    (")", ParserTokenKind::ReservedSymbol),
                    ("->", ParserTokenKind::ReservedSymbol),
                    ("1", ParserTokenKind::Numeral),
                    ("case", ParserTokenKind::ReservedWord),
                    ("thesis", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("thus", ParserTokenKind::ReservedWord),
                    ("thesis", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("end", ParserTokenKind::ReservedWord),
                    (";", ParserTokenKind::ReservedSymbol),
                ],
            ),
            Vec::new(),
        ));

        assert!(
            output
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == SyntaxDiagnosticCode::MalformedTypeExpression)
        );
        let ast = output
            .ast
            .expect("malformed inline definition before case branch should recover an AST");
        assert_eq!(
            count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::CaseItem)),
            2,
            "inline-definition recovery must not swallow the following `case` header"
        );
        assert_eq!(
            count_nodes(&ast, |kind| matches!(
                kind,
                SurfaceNodeKind::InlineFunctorDefinition
            )),
            1
        );
        assert!(
            count_nodes(&ast, |kind| matches!(
                kind,
                SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::SkippedToken)
            )) >= 1,
            "the malformed return type should be recovered without consuming the next branch"
        );
    }

    #[test]
    fn parser_rejects_task17_noncanonical_statement_justification_tails() {
        let source_id = source_id(92);
        let output = parse(ParseRequest::new(
            source_id,
            Edition::new("2026"),
            token_sequence(
                source_id,
                &[
                    ("assume", ParserTokenKind::ReservedWord),
                    ("thesis", ParserTokenKind::ReservedWord),
                    ("by", ParserTokenKind::ReservedWord),
                    ("A", ParserTokenKind::Identifier),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("given", ParserTokenKind::ReservedWord),
                    ("x", ParserTokenKind::Identifier),
                    ("being", ParserTokenKind::ReservedWord),
                    ("set", ParserTokenKind::ReservedWord),
                    ("by", ParserTokenKind::ReservedWord),
                    ("A", ParserTokenKind::Identifier),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("take", ParserTokenKind::ReservedWord),
                    ("x", ParserTokenKind::Identifier),
                    ("by", ParserTokenKind::ReservedWord),
                    ("A", ParserTokenKind::Identifier),
                    (";", ParserTokenKind::ReservedSymbol),
                    ("set", ParserTokenKind::ReservedWord),
                    ("f", ParserTokenKind::Identifier),
                    ("=", ParserTokenKind::ReservedSymbol),
                    ("x", ParserTokenKind::Identifier),
                    ("by", ParserTokenKind::ReservedWord),
                    ("A", ParserTokenKind::Identifier),
                    (";", ParserTokenKind::ReservedSymbol),
                ],
            ),
            Vec::new(),
        ));

        assert_eq!(
            output
                .diagnostics
                .iter()
                .map(|diagnostic| &diagnostic.code)
                .collect::<Vec<_>>(),
            vec![
                &SyntaxDiagnosticCode::MalformedTermExpression,
                &SyntaxDiagnosticCode::MalformedTermExpression,
                &SyntaxDiagnosticCode::MalformedTermExpression,
                &SyntaxDiagnosticCode::MalformedTermExpression,
            ]
        );
        let ast = output
            .ast
            .expect("noncanonical justification tails should recover");
        assert_eq!(
            count_nodes(&ast, |kind| matches!(
                kind,
                SurfaceNodeKind::JustificationClause
            )),
            0
        );
        assert_eq!(
            count_nodes(&ast, |kind| matches!(
                kind,
                SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::SkippedToken)
            )),
            4
        );
        assert_eq!(ast.trivia().skipped_token_ranges().len(), 4);
        assert_eq!(
            count_nodes(&ast, |kind| matches!(
                kind,
                SurfaceNodeKind::AssumptionStatement
                    | SurfaceNodeKind::GivenStatement
                    | SurfaceNodeKind::TakeStatement
                    | SurfaceNodeKind::SetStatement
            )),
            4
        );
    }

    fn single_node(
        ast: &mizar_syntax::SurfaceAst,
        predicate: impl Fn(&SurfaceNodeKind) -> bool,
    ) -> &mizar_syntax::SurfaceNode {
        ast.nodes()
            .iter()
            .find(|node| predicate(&node.kind))
            .expect("expected exactly one matching node")
    }

    fn formula_root_for_label<'a>(
        ast: &'a mizar_syntax::SurfaceAst,
        label: &str,
    ) -> &'a mizar_syntax::SurfaceNode {
        let item = ast
            .nodes()
            .iter()
            .find(|node| {
                matches!(
                    node.kind,
                    SurfaceNodeKind::TheoremItem
                        | SurfaceNodeKind::LemmaItem
                        | SurfaceNodeKind::PlaceholderItem
                ) && node.children.iter().any(|child| {
                    ast.node(*child)
                        .and_then(mizar_syntax::SurfaceNode::token_text)
                        == Some(label)
                })
            })
            .unwrap_or_else(|| panic!("expected theorem-like item labelled `{label}`"));
        let formula_expression = item
            .children
            .iter()
            .filter_map(|child| ast.node(*child))
            .find(|child| matches!(child.kind, SurfaceNodeKind::FormulaExpression))
            .unwrap_or_else(|| panic!("expected formula expression for `{label}`"));
        assert_eq!(
            formula_expression.children.len(),
            1,
            "FormulaExpression for `{label}` should wrap exactly one child"
        );
        ast.node(formula_expression.children[0])
            .unwrap_or_else(|| panic!("expected formula child for `{label}`"))
    }

    fn set_comprehension_term_for_label<'a>(
        ast: &'a mizar_syntax::SurfaceAst,
        label: &str,
    ) -> &'a mizar_syntax::SurfaceNode {
        let formula = formula_root_for_label(ast, label);
        let term = builtin_predicate_left_term_payload(ast, formula);
        assert!(
            matches!(term.kind, SurfaceNodeKind::SetComprehension),
            "expected left term for `{label}` to be SetComprehension, got {:?}",
            term.kind
        );
        term
    }

    fn builtin_predicate_left_term_payload<'a>(
        ast: &'a mizar_syntax::SurfaceAst,
        formula: &mizar_syntax::SurfaceNode,
    ) -> &'a mizar_syntax::SurfaceNode {
        assert!(
            matches!(formula.kind, SurfaceNodeKind::BuiltinPredicateApplication),
            "expected builtin predicate application, got {:?}",
            formula.kind
        );
        let terms = structural_children(ast, formula)
            .into_iter()
            .filter(|child| matches!(child.kind, SurfaceNodeKind::TermExpression))
            .collect::<Vec<_>>();
        assert!(
            terms.len() >= 2,
            "builtin predicate application should have left and right terms"
        );
        let left_children = structural_children(ast, terms[0]);
        assert_eq!(
            left_children.len(),
            1,
            "left TermExpression should wrap one concrete term"
        );
        left_children[0]
    }

    fn set_comprehension_mapper(
        ast: &mizar_syntax::SurfaceAst,
        comprehension: &mizar_syntax::SurfaceNode,
    ) -> mizar_syntax::SurfaceNodeId {
        assert!(
            matches!(comprehension.kind, SurfaceNodeKind::SetComprehension),
            "expected SetComprehension, got {:?}",
            comprehension.kind
        );
        comprehension
            .children
            .iter()
            .copied()
            .find(|child| {
                ast.node(*child)
                    .is_some_and(|node| matches!(node.kind, SurfaceNodeKind::TermExpression))
            })
            .expect("SetComprehension should own a mapper TermExpression")
    }

    fn structural_children<'a>(
        ast: &'a mizar_syntax::SurfaceAst,
        node: &mizar_syntax::SurfaceNode,
    ) -> Vec<&'a mizar_syntax::SurfaceNode> {
        node.children
            .iter()
            .filter_map(|child| ast.node(*child))
            .filter(|child| !matches!(child.kind, SurfaceNodeKind::Token(_)))
            .collect()
    }

    fn direct_token_texts(
        ast: &mizar_syntax::SurfaceAst,
        node: &mizar_syntax::SurfaceNode,
    ) -> Vec<String> {
        node.children
            .iter()
            .filter_map(|child| ast.node(*child))
            .filter_map(mizar_syntax::SurfaceNode::token_text)
            .map(str::to_owned)
            .collect()
    }

    fn subtree_token_texts(
        ast: &mizar_syntax::SurfaceAst,
        node: &mizar_syntax::SurfaceNode,
    ) -> Vec<String> {
        let mut texts = Vec::new();
        collect_subtree_token_texts(ast, node, &mut texts);
        texts
    }

    fn collect_subtree_token_texts(
        ast: &mizar_syntax::SurfaceAst,
        node: &mizar_syntax::SurfaceNode,
        texts: &mut Vec<String>,
    ) {
        for child in &node.children {
            let Some(child) = ast.node(*child) else {
                continue;
            };
            if let Some(text) = child.token_text() {
                texts.push(text.to_owned());
            } else {
                collect_subtree_token_texts(ast, child, texts);
            }
        }
    }

    fn assert_binary_formula(
        node: &mizar_syntax::SurfaceNode,
        connective: SurfaceFormulaConnective,
        repeated: bool,
    ) {
        let SurfaceNodeKind::BinaryFormula(operator) = &node.kind else {
            panic!("expected BinaryFormula, got {:?}", node.kind);
        };
        assert_eq!(operator.connective, connective);
        assert_eq!(operator.repeated, repeated);
    }

    fn assert_quantified_formula(
        node: &mizar_syntax::SurfaceNode,
        quantifier: SurfaceQuantifierKind,
    ) {
        let SurfaceNodeKind::QuantifiedFormula(actual) = &node.kind else {
            panic!("expected QuantifiedFormula, got {:?}", node.kind);
        };
        assert_eq!(*actual, quantifier);
    }

    fn assert_formula_constant(node: &mizar_syntax::SurfaceNode, constant: SurfaceFormulaConstant) {
        let SurfaceNodeKind::FormulaConstant(actual) = &node.kind else {
            panic!("expected FormulaConstant, got {:?}", node.kind);
        };
        assert_eq!(*actual, constant);
    }

    fn count_nodes(
        ast: &mizar_syntax::SurfaceAst,
        predicate: impl Fn(&SurfaceNodeKind) -> bool,
    ) -> usize {
        ast.nodes()
            .iter()
            .filter(|node| predicate(&node.kind))
            .count()
    }

    fn subtree_contains_kind(
        ast: &mizar_syntax::SurfaceAst,
        id: mizar_syntax::SurfaceNodeId,
        predicate: impl Fn(&SurfaceNodeKind) -> bool + Copy,
    ) -> bool {
        let Some(node) = ast.node(id) else {
            return false;
        };
        predicate(&node.kind)
            || node
                .children
                .iter()
                .any(|child| subtree_contains_kind(ast, *child, predicate))
    }

    fn direct_child_has_kind(
        ast: &mizar_syntax::SurfaceAst,
        node: &mizar_syntax::SurfaceNode,
        predicate: impl Fn(&SurfaceNodeKind) -> bool,
    ) -> bool {
        node.children
            .iter()
            .filter_map(|child| ast.node(*child))
            .any(|child| predicate(&child.kind))
    }

    fn rowan_token_texts(ast: &mizar_syntax::SurfaceAst) -> Vec<String> {
        ast.rowan_root()
            .descendants_with_tokens()
            .filter_map(|element| element.into_token())
            .map(|token| token.text().to_owned())
            .collect()
    }

    fn qualified_symbol_token_texts(
        ast: &mizar_syntax::SurfaceAst,
        qualified: &mizar_syntax::SurfaceNode,
    ) -> Vec<String> {
        qualified
            .children
            .iter()
            .filter_map(|id| ast.node(*id))
            .filter_map(|node| {
                node.token_text().map(str::to_owned).or_else(|| {
                    node.children
                        .first()
                        .and_then(|id| ast.node(*id))
                        .and_then(mizar_syntax::SurfaceNode::token_text)
                        .map(str::to_owned)
                })
            })
            .collect()
    }

    fn token(
        source_id: SourceId,
        kind: ParserTokenKind,
        text: &str,
        start: usize,
        end: usize,
    ) -> ParserToken {
        ParserToken::new(kind, text, range(source_id, start, end))
    }

    fn token_sequence(
        source_id: SourceId,
        entries: &[(&str, ParserTokenKind)],
    ) -> Vec<ParserToken> {
        let mut cursor = 0;
        entries
            .iter()
            .map(|(text, kind)| {
                let start = cursor;
                let end = start + text.len();
                cursor = end + 1;
                token(source_id, *kind, text, start, end)
            })
            .collect()
    }

    fn operator_fixture_fixities() -> Vec<OperatorFixityEntry> {
        vec![
            OperatorFixityEntry::prefix("~", 70),
            OperatorFixityEntry::postfix("!", 90),
            OperatorFixityEntry::infix("++", 10, OperatorAssociativity::Left),
            OperatorFixityEntry::infix("**", 20, OperatorAssociativity::Right),
            OperatorFixityEntry::infix("%%", 10, OperatorAssociativity::NonAssociative),
        ]
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

    fn snapshot_id(byte: u8) -> BuildSnapshotId {
        let hex = format!("{byte:02x}").repeat(Hash::BYTE_LEN);
        BuildSnapshotId::from_published_schema_str(&format!(
            "mizar-session-build-snapshot-v1:{hex}"
        ))
        .unwrap()
    }
}
