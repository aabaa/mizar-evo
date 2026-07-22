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

mod annotations;

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

#[derive(Debug, Clone, PartialEq, Eq)]
struct ParsedAnnotationPrefix {
    children: Vec<SurfaceBuilderNodeId>,
    next_position: usize,
    recovery_nodes: Vec<SurfaceBuilderNodeId>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AnnotationArgumentRequirement {
    Identifier,
    StringLiteral,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AnnotationDelimitedListBoundary {
    Argument,
    ProofHintOption,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ProofHintOptionValueKind {
    NatLiteral,
    SolverName,
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
enum AlgorithmStatementListBoundary {
    Body,
    NestedBlock,
    IfThen,
    MatchCase,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AlgorithmTermListBoundary {
    HeaderClause,
    ClauseStatement,
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
                    if item_head > position
                        && (self.is_reserved_word_at(item_head, "import")
                            || self.is_reserved_word_at(item_head, "export"))
                    {
                        import_prelude_open = false;
                        export_prelude_open = false;
                        let recovery = self.recover_unexpected_top_level_tokens(position);
                        item_children.push(recovery.id);
                        recovery_nodes.extend(recovery.recovery_nodes);
                        position = recovery.next_position;
                        continue;
                    }
                    if self.is_reserved_word_at(item_head, "import") {
                        if import_prelude_open {
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
                        if export_prelude_open {
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
                    if let Some(item) = self.parse_registration_block_item(position) {
                        item_children.push(item.id);
                        recovery_nodes.extend(item.recovery_nodes);
                        position = item.next_position;
                        continue;
                    }
                    if let Some(item) = self.parse_claim_block_item(position) {
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
        if let Some(annotation) = self.parse_standalone_diagnostic_annotation_at(position) {
            return Some(annotation);
        }

        let prefix = self.parse_leading_annotations_at(position);
        if prefix.next_position > position {
            let statement = self.parse_statement_core_at(prefix.next_position);
            return Some(self.finish_annotated_type_node(
                position,
                prefix,
                statement,
                SurfaceNodeKind::AnnotatedStatement,
                "expected statement after annotation",
            ));
        }

        self.parse_statement_core_at(position)
    }

    fn parse_statement_core_at(&mut self, position: usize) -> Option<ParsedTypeNode> {
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
        let mut children = vec![self.token_node_ids[position]];
        let mut recovery_nodes = Vec::new();
        let mut cursor = position + 1;
        if self.is_reserved_symbol_at(cursor, "[") {
            let arguments = self.parse_template_arguments_at(cursor);
            cursor = arguments.next_position;
            children.push(arguments.id);
            recovery_nodes.extend(arguments.recovery_nodes);
        }
        let id = self.events.emit(SyntaxEvent::Node {
            kind: SurfaceNodeKind::Reference,
            range: self.covering_token_range(position, cursor),
            children,
        });
        Some(ParsedTypeNode {
            id,
            next_position: cursor.max(position + 1),
            recovery_nodes,
        })
    }

    fn parse_qualified_reference_at(&mut self, position: usize) -> Option<ParsedTypeNode> {
        let plan = self.qualified_reference_plan_at(position)?;
        let namespace = self
            .emit_namespace_path_from_parts(&plan.namespace_segments, &plan.namespace_separators);
        let mut children = vec![
            namespace,
            self.token_node_ids[plan.final_separator],
            self.token_node_ids[plan.final_reference],
        ];
        let mut recovery_nodes = Vec::new();
        let mut cursor = plan.next_position;
        if self.is_reserved_symbol_at(cursor, "[") {
            let arguments = self.parse_template_arguments_at(cursor);
            cursor = arguments.next_position;
            children.push(arguments.id);
            recovery_nodes.extend(arguments.recovery_nodes);
        }
        let id = self.events.emit(SyntaxEvent::Node {
            kind: SurfaceNodeKind::QualifiedReference,
            range: self.covering_token_range(plan.first_position(), cursor),
            children,
        });
        Some(ParsedTypeNode {
            id,
            next_position: cursor.max(position + 1),
            recovery_nodes,
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
        let mut children = vec![self.token_node_ids[position]];
        let mut recovery_nodes = Vec::new();
        let mut cursor = position + 1;
        if self.is_reserved_symbol_at(cursor, "[") {
            let arguments = self.parse_template_arguments_at(cursor);
            cursor = arguments.next_position;
            children.push(arguments.id);
            recovery_nodes.extend(arguments.recovery_nodes);
        }
        let id = self.events.emit(SyntaxEvent::Node {
            kind: SurfaceNodeKind::GroupedReferenceItem,
            range: self.covering_token_range(position, cursor),
            children,
        });
        Some(ParsedTypeNode {
            id,
            next_position: cursor.max(position + 1),
            recovery_nodes,
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
            let opener = position.min(self.request.tokens.len().saturating_sub(1));
            let missing = self.add_missing_end(opener, cursor);
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
        let head = position;
        let mut children = vec![self.token_node_ids[head]];
        let mut recovery_nodes = Vec::new();
        let mut cursor = head + 1;

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
        let prefix = self.parse_leading_annotations_at(position);
        let mut children = prefix.children;
        let mut recovery_nodes = prefix.recovery_nodes;
        children.extend(
            self.token_node_ids[prefix.next_position..marker_position]
                .iter()
                .copied(),
        );
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
        let surface_kind = if self.is_reserved_word_at(role, "lemma") {
            SurfaceNodeKind::LemmaItem
        } else {
            SurfaceNodeKind::TheoremItem
        };
        let prefix = self.parse_leading_annotations_at(position);
        let mut children = prefix.children;
        let mut recovery_nodes = prefix.recovery_nodes;
        children.extend(
            self.token_node_ids[prefix.next_position..=role]
                .iter()
                .copied(),
        );
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

        let prefix = self.parse_leading_annotations_at(position);
        let mut children = prefix.children;
        let mut recovery_nodes = prefix.recovery_nodes;
        children.extend(
            self.token_node_ids[prefix.next_position..=head]
                .iter()
                .copied(),
        );
        let mut cursor = head + 1;
        let template_definition = self.definition_block_is_template_shaped_at(cursor);
        let mut parsing_leading_template_parameters = template_definition;

        while cursor < self.request.tokens.len() && !self.is_end_keyword_at(cursor) {
            let content =
                if parsing_leading_template_parameters && self.is_reserved_word_at(cursor, "let") {
                    self.parse_template_parameter_at(cursor)
                } else {
                    parsing_leading_template_parameters = false;
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

    fn parse_registration_block_item(&mut self, position: usize) -> Option<ParsedItem> {
        let head = self.item_head_position(position)?;
        if !self.is_reserved_word_at(head, "registration") {
            return None;
        }

        let prefix = self.parse_leading_annotations_at(position);
        let mut children = prefix.children;
        let mut recovery_nodes = prefix.recovery_nodes;
        children.extend(
            self.token_node_ids[prefix.next_position..=head]
                .iter()
                .copied(),
        );
        let mut cursor = head + 1;

        while cursor < self.request.tokens.len() && !self.is_end_keyword_at(cursor) {
            let content_start = self.skip_annotations(cursor);
            if self.is_registration_block_outer_content_boundary_at(content_start) {
                break;
            }

            let content = self
                .parse_registration_content_at(cursor)
                .unwrap_or_else(|| self.parse_registration_content_placeholder(cursor));
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
            kind: SurfaceNodeKind::RegistrationBlockItem,
            range: self.covering_token_range(position, cursor),
            children,
        });
        Some(ParsedItem {
            id,
            next_position: cursor.max(position + 1),
            recovery_nodes,
        })
    }

    fn parse_registration_content_at(&mut self, position: usize) -> Option<ParsedTypeNode> {
        if let Some(annotation) = self.parse_standalone_diagnostic_annotation_at(position) {
            return Some(annotation);
        }

        let prefix = self.parse_leading_annotations_at(position);
        if prefix.next_position > position {
            let content = self.parse_registration_content_core_at(prefix.next_position);
            return Some(self.finish_annotated_type_node(
                position,
                prefix,
                content,
                SurfaceNodeKind::AnnotatedRegistrationContent,
                "expected registration content after annotation",
            ));
        }

        self.parse_registration_content_core_at(position)
    }

    fn parse_registration_content_core_at(&mut self, position: usize) -> Option<ParsedTypeNode> {
        if self.is_reserved_word_at(position, "let") {
            return Some(self.parse_registration_parameter_at(position));
        }
        if self.is_registration_item_start_at(position) {
            return Some(self.parse_registration_item_at(position));
        }
        None
    }

    fn is_registration_block_outer_content_boundary_at(&self, position: usize) -> bool {
        self.is_item_start_at(position) && !self.is_registration_content_start_at(position)
    }

    fn parse_claim_block_item(&mut self, position: usize) -> Option<ParsedItem> {
        let head = self.item_head_position(position)?;
        if !self.is_reserved_word_at(head, "claim") {
            return None;
        }

        let prefix = self.parse_leading_annotations_at(position);
        let mut children = prefix.children;
        let mut recovery_nodes = prefix.recovery_nodes;
        children.extend(
            self.token_node_ids[prefix.next_position..=head]
                .iter()
                .copied(),
        );
        let mut cursor = head + 1;

        if self.is_identifier_at(cursor) {
            children.push(self.token_node_ids[cursor]);
            cursor += 1;
        } else {
            self.diagnose_malformed_term_expression(cursor, "expected claim target algorithm name");
            self.push_missing_term(cursor, &mut children, &mut recovery_nodes);
        }

        if self.is_reserved_word_at(cursor, "do") {
            children.push(self.token_node_ids[cursor]);
            cursor += 1;
        } else {
            self.diagnose_malformed_formula_expression(cursor, "expected `do` in claim block");
        }

        while cursor < self.request.tokens.len() && !self.is_end_keyword_at(cursor) {
            if let Some(item) = self.parse_theorem_item(cursor) {
                let made_progress = item.next_position > cursor;
                cursor = item.next_position;
                children.push(item.id);
                recovery_nodes.extend(item.recovery_nodes);
                if !made_progress {
                    break;
                }
                continue;
            }

            self.diagnose_malformed_formula_expression(
                cursor,
                "expected theorem or lemma in claim block",
            );
            if let Some(recovery) = self.recover_malformed_claim_content_tail(cursor) {
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
            kind: SurfaceNodeKind::ClaimBlockItem,
            range: self.covering_token_range(position, cursor),
            children,
        });
        Some(ParsedItem {
            id,
            next_position: cursor.max(position + 1),
            recovery_nodes,
        })
    }

    fn parse_registration_parameter_at(&mut self, position: usize) -> ParsedTypeNode {
        let mut children = vec![self.token_node_ids[position]];
        let mut recovery_nodes = Vec::new();
        let mut cursor = position + 1;

        cursor = self.parse_qualified_variable_segments(
            cursor,
            &mut children,
            &mut recovery_nodes,
            "expected variable segment after registration `let`",
            "expected variable segment after `,` in registration parameter",
        );
        cursor =
            self.parse_optional_such_condition_list(cursor, &mut children, &mut recovery_nodes);
        if self.is_reserved_word_at(cursor, "by") {
            let justification = self.parse_justification_clause_at(cursor, false);
            cursor = justification.next_position;
            children.push(justification.id);
            recovery_nodes.extend(justification.recovery_nodes);
        }

        self.finish_registration_content_node(
            position,
            cursor,
            SurfaceNodeKind::RegistrationParameter,
            children,
            recovery_nodes,
            "unexpected token in registration parameter",
        )
    }

    fn parse_registration_item_at(&mut self, position: usize) -> ParsedTypeNode {
        if self.is_reserved_word_at(position, "reduce") {
            self.parse_reduction_registration_at(position)
        } else {
            self.parse_cluster_registration_at(position)
        }
    }

    fn parse_cluster_registration_at(&mut self, position: usize) -> ParsedTypeNode {
        let mut children = vec![self.token_node_ids[position]];
        let mut recovery_nodes = Vec::new();
        let mut cursor = position + 1;

        if self.is_identifier_at(cursor) {
            children.push(self.token_node_ids[cursor]);
            cursor += 1;
        } else {
            self.diagnose_malformed_term_expression(cursor, "expected registration label");
            self.push_missing_term(cursor, &mut children, &mut recovery_nodes);
        }

        if self.is_reserved_symbol_at(cursor, ":") {
            children.push(self.token_node_ids[cursor]);
            cursor += 1;
        } else {
            self.diagnose_malformed_formula_expression(
                cursor,
                "expected `:` after registration label",
            );
        }

        let arrow = self.top_level_symbol_before_statement_boundary_at(cursor, "->");
        if let Some(arrow) = arrow {
            if arrow == cursor || self.registration_adjective_list_can_match(cursor, arrow) {
                return self.parse_conditional_registration_tail(
                    position,
                    cursor,
                    arrow,
                    children,
                    recovery_nodes,
                );
            }
            return self.parse_functorial_registration_tail(
                position,
                cursor,
                arrow,
                children,
                recovery_nodes,
            );
        }

        self.parse_existential_registration_tail(position, cursor, children, recovery_nodes)
    }

    fn parse_existential_registration_tail(
        &mut self,
        position: usize,
        mut cursor: usize,
        mut children: Vec<SurfaceBuilderNodeId>,
        mut recovery_nodes: Vec<SurfaceBuilderNodeId>,
    ) -> ParsedTypeNode {
        if self.registration_adjective_ref_plan_at(cursor).is_none() {
            self.diagnose_malformed_type_expression(
                cursor,
                "expected existential registration adjective before type",
            );
        }
        if let Some(argument_position) = self.first_argument_bearing_registration_adjective_before(
            cursor,
            self.registration_header_boundary_at(cursor),
        ) {
            self.diagnose_malformed_type_expression(
                argument_position,
                "registration adjectives cannot have arguments",
            );
        }

        if let Some(type_expression) = self.parse_type_expression_at(cursor) {
            cursor = type_expression.next_position;
            children.push(type_expression.id);
            recovery_nodes.extend(type_expression.recovery_nodes);
        } else {
            self.diagnose_malformed_type_expression(
                cursor,
                "expected existential registration type",
            );
            let missing = self.add_missing_type_expression(cursor);
            children.push(missing);
            recovery_nodes.push(missing);
        }

        cursor = self.parse_required_registration_header_semicolon(cursor, &mut children);
        let correctness = self.parse_registration_correctness_condition_at(cursor, "existence");
        cursor = correctness.next_position;
        children.push(correctness.id);
        recovery_nodes.extend(correctness.recovery_nodes);

        let id = self.events.emit(SyntaxEvent::Node {
            kind: SurfaceNodeKind::ExistentialRegistration,
            range: self.covering_token_range(position, cursor),
            children,
        });
        ParsedTypeNode {
            id,
            next_position: cursor.max(position + 1),
            recovery_nodes,
        }
    }

    fn parse_conditional_registration_tail(
        &mut self,
        position: usize,
        mut cursor: usize,
        arrow: usize,
        mut children: Vec<SurfaceBuilderNodeId>,
        mut recovery_nodes: Vec<SurfaceBuilderNodeId>,
    ) -> ParsedTypeNode {
        self.parse_registration_adjective_list_until(
            cursor,
            arrow,
            &mut children,
            &mut recovery_nodes,
            "expected conditional registration antecedent",
            "malformed conditional registration antecedent",
        );
        cursor = arrow;

        children.push(self.token_node_ids[cursor]);
        cursor += 1;

        cursor = self.parse_registration_consequent_and_type(
            cursor,
            &mut children,
            &mut recovery_nodes,
            "expected conditional registration consequent",
        );
        cursor = self.parse_required_registration_header_semicolon(cursor, &mut children);
        let correctness = self.parse_registration_correctness_condition_at(cursor, "coherence");
        cursor = correctness.next_position;
        children.push(correctness.id);
        recovery_nodes.extend(correctness.recovery_nodes);

        let id = self.events.emit(SyntaxEvent::Node {
            kind: SurfaceNodeKind::ConditionalRegistration,
            range: self.covering_token_range(position, cursor),
            children,
        });
        ParsedTypeNode {
            id,
            next_position: cursor.max(position + 1),
            recovery_nodes,
        }
    }

    fn parse_functorial_registration_tail(
        &mut self,
        position: usize,
        mut cursor: usize,
        arrow: usize,
        mut children: Vec<SurfaceBuilderNodeId>,
        mut recovery_nodes: Vec<SurfaceBuilderNodeId>,
    ) -> ParsedTypeNode {
        if self.registration_functorial_payload_can_match(cursor, arrow) {
            if let Some(term) = self.parse_term_expression_at(cursor) {
                if term.next_position == arrow {
                    cursor = term.next_position;
                    children.push(term.id);
                    recovery_nodes.extend(term.recovery_nodes);
                } else {
                    self.diagnose_malformed_term_expression(
                        term.next_position,
                        "unexpected token in functorial registration payload",
                    );
                    children.push(term.id);
                    recovery_nodes.extend(term.recovery_nodes);
                    if let Some(recovery) =
                        self.emit_malformed_tail_recovery(term.next_position, arrow)
                    {
                        children.push(recovery.id);
                        recovery_nodes.extend(recovery.recovery_nodes);
                    }
                    cursor = arrow;
                }
            } else {
                self.diagnose_malformed_term_expression(
                    cursor,
                    "expected functorial registration payload",
                );
                self.push_missing_term(cursor, &mut children, &mut recovery_nodes);
                if let Some(recovery) = self.emit_malformed_tail_recovery(cursor, arrow) {
                    children.push(recovery.id);
                    recovery_nodes.extend(recovery.recovery_nodes);
                }
                cursor = arrow;
            }
        } else {
            self.diagnose_malformed_term_expression(
                cursor,
                "expected functorial registration application payload",
            );
            self.push_missing_term(cursor, &mut children, &mut recovery_nodes);
            if let Some(recovery) = self.emit_malformed_tail_recovery(cursor, arrow) {
                children.push(recovery.id);
                recovery_nodes.extend(recovery.recovery_nodes);
            }
            cursor = arrow;
        }

        children.push(self.token_node_ids[cursor]);
        cursor += 1;

        cursor = self.parse_registration_consequent_and_type(
            cursor,
            &mut children,
            &mut recovery_nodes,
            "expected functorial registration consequent",
        );
        cursor = self.parse_required_registration_header_semicolon(cursor, &mut children);
        let correctness = self.parse_registration_correctness_condition_at(cursor, "coherence");
        cursor = correctness.next_position;
        children.push(correctness.id);
        recovery_nodes.extend(correctness.recovery_nodes);

        let id = self.events.emit(SyntaxEvent::Node {
            kind: SurfaceNodeKind::FunctorialRegistration,
            range: self.covering_token_range(position, cursor),
            children,
        });
        ParsedTypeNode {
            id,
            next_position: cursor.max(position + 1),
            recovery_nodes,
        }
    }

    fn parse_reduction_registration_at(&mut self, position: usize) -> ParsedTypeNode {
        let mut children = vec![self.token_node_ids[position]];
        let mut recovery_nodes = Vec::new();
        let mut cursor = position + 1;

        if self.is_identifier_at(cursor) {
            children.push(self.token_node_ids[cursor]);
            cursor += 1;
        } else {
            self.diagnose_malformed_term_expression(
                cursor,
                "expected reduction registration label",
            );
            self.push_missing_term(cursor, &mut children, &mut recovery_nodes);
        }

        if self.is_reserved_symbol_at(cursor, ":") {
            children.push(self.token_node_ids[cursor]);
            cursor += 1;
        } else {
            self.diagnose_malformed_formula_expression(
                cursor,
                "expected `:` after reduction registration label",
            );
        }

        if let Some(left) = self.parse_term_expression_at(cursor) {
            cursor = left.next_position;
            children.push(left.id);
            recovery_nodes.extend(left.recovery_nodes);
        } else {
            self.diagnose_malformed_term_expression(cursor, "expected reduction left-hand term");
            self.push_missing_term(cursor, &mut children, &mut recovery_nodes);
        }

        if self.is_reserved_word_at(cursor, "to") {
            children.push(self.token_node_ids[cursor]);
            cursor += 1;
        } else {
            self.diagnose_malformed_term_expression(cursor, "expected `to` in reduction");
        }

        if let Some(right) = self.parse_term_expression_at(cursor) {
            cursor = right.next_position;
            children.push(right.id);
            recovery_nodes.extend(right.recovery_nodes);
        } else {
            self.diagnose_malformed_term_expression(cursor, "expected reduction right-hand term");
            self.push_missing_term(cursor, &mut children, &mut recovery_nodes);
        }

        cursor = self.parse_required_registration_header_semicolon(cursor, &mut children);
        let correctness = self.parse_registration_correctness_condition_at(cursor, "reducibility");
        cursor = correctness.next_position;
        children.push(correctness.id);
        recovery_nodes.extend(correctness.recovery_nodes);

        let id = self.events.emit(SyntaxEvent::Node {
            kind: SurfaceNodeKind::ReductionRegistration,
            range: self.covering_token_range(position, cursor),
            children,
        });
        ParsedTypeNode {
            id,
            next_position: cursor.max(position + 1),
            recovery_nodes,
        }
    }

    fn parse_registration_consequent_and_type(
        &mut self,
        mut cursor: usize,
        children: &mut Vec<SurfaceBuilderNodeId>,
        recovery_nodes: &mut Vec<SurfaceBuilderNodeId>,
        missing_message: &'static str,
    ) -> usize {
        let Some(for_position) =
            self.top_level_reserved_word_before_statement_boundary_at(cursor, "for")
        else {
            let boundary = self.registration_header_boundary_at(cursor);
            self.parse_registration_adjective_list_until(
                cursor,
                boundary,
                children,
                recovery_nodes,
                missing_message,
                "malformed registration consequent",
            );
            self.diagnose_malformed_type_expression(boundary, "expected `for` in registration");
            let missing = self.add_missing_type_expression(boundary);
            children.push(missing);
            recovery_nodes.push(missing);
            return boundary;
        };

        self.parse_registration_adjective_list_until(
            cursor,
            for_position,
            children,
            recovery_nodes,
            missing_message,
            "malformed registration consequent",
        );
        cursor = for_position;
        children.push(self.token_node_ids[cursor]);
        cursor += 1;

        if let Some(type_expression) = self.parse_type_expression_at(cursor) {
            cursor = type_expression.next_position;
            children.push(type_expression.id);
            recovery_nodes.extend(type_expression.recovery_nodes);
        } else {
            self.diagnose_malformed_type_expression(cursor, "expected registration target type");
            let missing = self.add_missing_type_expression(cursor);
            children.push(missing);
            recovery_nodes.push(missing);
        }
        cursor
    }

    fn parse_registration_correctness_condition_at(
        &mut self,
        position: usize,
        expected_keyword: &'static str,
    ) -> ParsedTypeNode {
        let mut children = Vec::new();
        let mut recovery_nodes = Vec::new();
        let mut cursor = position;

        if self.is_reserved_word_at(cursor, expected_keyword) {
            children.push(self.token_node_ids[cursor]);
            cursor += 1;
        } else {
            self.diagnose_malformed_justification(
                cursor,
                format!("expected `{expected_keyword}` registration correctness condition"),
            );
            if self.is_correctness_condition_keyword_at(cursor) {
                children.push(self.token_node_ids[cursor]);
                cursor += 1;
            }
        }

        if let Some(justification) =
            self.parse_definition_content_general_justification_at(cursor, true)
        {
            cursor = justification.next_position;
            children.push(justification.id);
            recovery_nodes.extend(justification.recovery_nodes);
        } else if !self.is_semicolon_at(cursor)
            && !self.is_registration_content_synchronization_boundary_at(cursor)
        {
            self.diagnose_malformed_justification(
                cursor,
                "expected registration correctness-condition justification",
            );
            if let Some(recovery) = self.recover_malformed_registration_content_tail(cursor) {
                cursor = recovery.next_position;
                children.push(recovery.id);
                recovery_nodes.extend(recovery.recovery_nodes);
            }
        } else {
            self.diagnose_malformed_justification(
                cursor,
                "expected registration correctness-condition justification",
            );
            self.push_missing_proof_step(cursor, &mut children, &mut recovery_nodes);
        }

        if self.is_semicolon_at(cursor) {
            children.push(self.token_node_ids[cursor]);
            cursor += 1;
        } else {
            self.diagnose_missing_semicolon(cursor);
        }

        let range = if cursor > position {
            self.covering_token_range(position, cursor)
        } else {
            self.zero_range_at(position)
        };
        let id = self.events.emit(SyntaxEvent::Node {
            kind: SurfaceNodeKind::CorrectnessCondition,
            range,
            children,
        });
        ParsedTypeNode {
            id,
            next_position: cursor,
            recovery_nodes,
        }
    }

    fn parse_registration_adjective_list_until(
        &mut self,
        position: usize,
        end: usize,
        children: &mut Vec<SurfaceBuilderNodeId>,
        recovery_nodes: &mut Vec<SurfaceBuilderNodeId>,
        missing_message: &'static str,
        malformed_message: &'static str,
    ) {
        let mut cursor = position;
        let mut saw_adjective = false;
        while cursor < end {
            let Some(plan) = self.registration_adjective_ref_plan_at(cursor) else {
                break;
            };
            if plan.next_position > end {
                break;
            }
            let adjective = self.parse_registration_adjective_ref_from_plan(plan);
            cursor = adjective.next_position;
            children.push(adjective.id);
            recovery_nodes.extend(adjective.recovery_nodes);
            saw_adjective = true;
        }

        if !saw_adjective {
            self.diagnose_malformed_type_expression(position, missing_message);
            let missing = self.add_missing_type_expression(position);
            children.push(missing);
            recovery_nodes.push(missing);
        }

        if cursor < end {
            self.diagnose_malformed_type_expression(cursor, malformed_message);
            if let Some(recovery) = self.emit_malformed_tail_recovery(cursor, end) {
                children.push(recovery.id);
                recovery_nodes.extend(recovery.recovery_nodes);
            }
        }
    }

    fn parse_registration_adjective_ref_from_plan(
        &mut self,
        plan: AttributeRefPlan,
    ) -> ParsedTypeNode {
        let mut children = Vec::new();
        let recovery_nodes = Vec::new();
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
            .expect("planned registration adjective should contain an attribute symbol");
        children.push(symbol.id);
        cursor = symbol.next_position;

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

    fn finish_registration_content_node(
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
            && !self.is_registration_content_synchronization_boundary_at(cursor)
        {
            self.diagnose_malformed_term_expression(cursor, unexpected_message);
            if let Some(recovery) = self.recover_malformed_registration_content_tail(cursor) {
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

    fn parse_required_registration_header_semicolon(
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

    fn parse_registration_content_placeholder(&mut self, position: usize) -> ParsedTypeNode {
        let end = self.registration_content_placeholder_end(position);
        let item = self.emit_placeholder_item(position, end);
        ParsedTypeNode {
            id: item.id,
            next_position: item.next_position,
            recovery_nodes: item.recovery_nodes,
        }
    }

    fn parse_definition_content_at(&mut self, position: usize) -> Option<ParsedTypeNode> {
        if let Some(annotation) = self.parse_standalone_diagnostic_annotation_at(position) {
            return Some(annotation);
        }

        let prefix = self.parse_leading_annotations_at(position);
        if prefix.next_position > position {
            let content = self.parse_definition_content_core_at(prefix.next_position);
            return Some(self.finish_annotated_type_node(
                position,
                prefix,
                content,
                SurfaceNodeKind::AnnotatedDefinitionContent,
                "expected definition content after annotation",
            ));
        }

        self.parse_definition_content_core_at(position)
    }

    fn parse_definition_content_core_at(&mut self, position: usize) -> Option<ParsedTypeNode> {
        if self.definition_parameter_starts_template_ambiguous_at(position) {
            return Some(self.parse_template_parameter_at(position));
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
        if self.is_algorithm_definition_start_at(position) {
            return Some(self.parse_algorithm_definition_at(position));
        }
        if self.is_registration_item_start_at(position) {
            return Some(self.parse_registration_item_at(position));
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
            && self.is_algorithm_definition_start_at(position + 1)
        {
            return Some(self.parse_visible_algorithm_definition_at(position));
        }
        if self.is_visibility_marker_at(position)
            && self.is_registration_item_start_at(position + 1)
        {
            return Some(self.parse_visible_registration_item_at(position));
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

        cursor =
            self.parse_definition_parameter_tail_at(cursor, &mut children, &mut recovery_nodes);

        self.finish_definition_content_node(
            position,
            cursor,
            SurfaceNodeKind::DefinitionParameter,
            children,
            recovery_nodes,
            "unexpected token in definition parameter",
        )
    }

    fn parse_definition_parameter_tail_at(
        &mut self,
        mut cursor: usize,
        children: &mut Vec<SurfaceBuilderNodeId>,
        recovery_nodes: &mut Vec<SurfaceBuilderNodeId>,
    ) -> usize {
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
                        self.push_missing_formula(cursor, children, recovery_nodes);
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
                    self.push_missing_formula(cursor, children, recovery_nodes);
                }
            } else {
                self.diagnose_malformed_formula_expression(
                    cursor,
                    "expected `that` after `such` in definition parameter",
                );
                self.push_missing_formula(cursor, children, recovery_nodes);
            }
        } else if self.is_reserved_word_at(cursor, "by") {
            let justification = self.parse_justification_clause_at(cursor, false);
            cursor = justification.next_position;
            children.push(justification.id);
            recovery_nodes.extend(justification.recovery_nodes);
        }
        cursor
    }

    fn parse_template_parameter_at(&mut self, position: usize) -> ParsedTypeNode {
        let mut children = vec![self.token_node_ids[position]];
        let mut recovery_nodes = Vec::new();
        let mut cursor = position + 1;

        cursor =
            self.parse_template_parameter_bindings_at(cursor, &mut children, &mut recovery_nodes);

        if self.is_reserved_word_at(cursor, "be") || self.is_reserved_word_at(cursor, "being") {
            children.push(self.token_node_ids[cursor]);
            cursor += 1;
        } else {
            self.diagnose_malformed_type_expression(
                cursor,
                "expected `be` or `being` in template parameter",
            );
        }

        if self.is_reserved_word_at(cursor, "type") {
            children.push(self.token_node_ids[cursor]);
            cursor += 1;
            if self.is_reserved_word_at(cursor, "extends") {
                children.push(self.token_node_ids[cursor]);
                cursor += 1;
                if let Some(bound) = self.parse_template_type_expression_at(cursor) {
                    cursor = bound.next_position;
                    children.push(bound.id);
                    recovery_nodes.extend(bound.recovery_nodes);
                } else {
                    self.diagnose_malformed_type_expression(
                        cursor,
                        "expected type after `extends`",
                    );
                    let missing = self.add_missing_type_expression(cursor);
                    children.push(missing);
                    recovery_nodes.push(missing);
                }
            }
        } else if self.is_reserved_word_at(cursor, "pred") {
            children.push(self.token_node_ids[cursor]);
            cursor += 1;
            cursor = self.parse_template_parameter_type_list_at(
                cursor,
                &mut children,
                &mut recovery_nodes,
            );
        } else if self.is_reserved_word_at(cursor, "func") {
            children.push(self.token_node_ids[cursor]);
            cursor += 1;
            cursor = self.parse_template_parameter_type_list_at(
                cursor,
                &mut children,
                &mut recovery_nodes,
            );
            if self.is_reserved_symbol_at(cursor, "->") {
                children.push(self.token_node_ids[cursor]);
                cursor += 1;
                if let Some(return_type) = self.parse_template_type_expression_at(cursor) {
                    cursor = return_type.next_position;
                    children.push(return_type.id);
                    recovery_nodes.extend(return_type.recovery_nodes);
                } else {
                    self.diagnose_malformed_type_expression(
                        cursor,
                        "expected functor-parameter return type",
                    );
                    let missing = self.add_missing_type_expression(cursor);
                    children.push(missing);
                    recovery_nodes.push(missing);
                }
            } else {
                self.diagnose_malformed_type_expression(
                    cursor,
                    "expected `->` in functor template parameter",
                );
            }
        } else if let Some(type_expression) = self.parse_template_type_expression_at(cursor) {
            cursor = type_expression.next_position;
            children.push(type_expression.id);
            recovery_nodes.extend(type_expression.recovery_nodes);
        } else {
            self.diagnose_malformed_type_expression(cursor, "expected template parameter type");
            let missing = self.add_missing_type_expression(cursor);
            children.push(missing);
            recovery_nodes.push(missing);
        }

        cursor =
            self.parse_definition_parameter_tail_at(cursor, &mut children, &mut recovery_nodes);

        self.finish_definition_content_node(
            position,
            cursor,
            SurfaceNodeKind::TemplateParameter,
            children,
            recovery_nodes,
            "unexpected token in template parameter",
        )
    }

    fn parse_template_parameter_bindings_at(
        &mut self,
        mut cursor: usize,
        children: &mut Vec<SurfaceBuilderNodeId>,
        recovery_nodes: &mut Vec<SurfaceBuilderNodeId>,
    ) -> usize {
        let mut expecting_binding = true;
        let mut saw_binding = false;

        loop {
            if self.is_reserved_word_at(cursor, "be")
                || self.is_reserved_word_at(cursor, "being")
                || self.is_semicolon_at(cursor)
                || self.is_definition_content_synchronization_boundary_at(cursor)
            {
                if expecting_binding {
                    self.diagnose_malformed_term_expression(
                        cursor,
                        if saw_binding {
                            "expected template parameter binding after `,`"
                        } else {
                            "expected template parameter binding after `let`"
                        },
                    );
                    self.push_missing_term(cursor, children, recovery_nodes);
                }
                break;
            }

            if self.is_reserved_symbol_at(cursor, ",") {
                if expecting_binding {
                    self.diagnose_malformed_term_expression(
                        cursor,
                        "expected template parameter binding before `,`",
                    );
                    self.push_missing_term(cursor, children, recovery_nodes);
                }
                children.push(self.token_node_ids[cursor]);
                cursor += 1;
                expecting_binding = true;
                continue;
            }

            if let Some(symbol) = self.parse_qualified_symbol_at(cursor) {
                cursor = symbol.next_position;
                children.push(symbol.id);
            } else if self.is_identifier_at(cursor) || self.is_user_symbol_at(cursor) {
                children.push(self.token_node_ids[cursor]);
                cursor += 1;
            } else {
                self.diagnose_malformed_term_expression(
                    cursor,
                    "expected template parameter binding",
                );
                self.push_missing_term(cursor, children, recovery_nodes);
                break;
            }

            if self.is_reserved_symbol_at(cursor, "[") {
                let arguments = self.parse_template_arguments_at(cursor);
                cursor = arguments.next_position;
                children.push(arguments.id);
                recovery_nodes.extend(arguments.recovery_nodes);
            }

            saw_binding = true;
            if self.is_reserved_symbol_at(cursor, ",") {
                children.push(self.token_node_ids[cursor]);
                cursor += 1;
                expecting_binding = true;
            } else {
                break;
            }
        }

        cursor
    }

    fn parse_template_parameter_type_list_at(
        &mut self,
        mut cursor: usize,
        children: &mut Vec<SurfaceBuilderNodeId>,
        recovery_nodes: &mut Vec<SurfaceBuilderNodeId>,
    ) -> usize {
        if !self.is_reserved_symbol_at(cursor, "(") {
            self.diagnose_malformed_type_expression(
                cursor,
                "expected `(` before template parameter type list",
            );
            return cursor;
        }

        let opener = cursor;
        children.push(self.token_node_ids[cursor]);
        cursor += 1;
        let mut expecting_type = true;
        let mut saw_type = false;

        while cursor < self.request.tokens.len() && !self.is_reserved_symbol_at(cursor, ")") {
            if self.is_reserved_symbol_at(cursor, ",") {
                if expecting_type {
                    self.diagnose_malformed_type_expression(cursor, "expected type before `,`");
                    let missing = self.add_missing_type_expression(cursor);
                    children.push(missing);
                    recovery_nodes.push(missing);
                }
                children.push(self.token_node_ids[cursor]);
                cursor += 1;
                expecting_type = true;
                continue;
            }

            if let Some(type_expression) = self.parse_template_type_expression_at(cursor) {
                cursor = type_expression.next_position;
                children.push(type_expression.id);
                recovery_nodes.extend(type_expression.recovery_nodes);
                expecting_type = false;
                saw_type = true;
            } else {
                self.diagnose_malformed_type_expression(cursor, "expected template parameter type");
                let missing = self.add_missing_type_expression(cursor);
                children.push(missing);
                recovery_nodes.push(missing);
                break;
            }

            if self.is_reserved_symbol_at(cursor, ",") {
                children.push(self.token_node_ids[cursor]);
                cursor += 1;
                expecting_type = true;
            } else {
                break;
            }
        }

        if self.is_reserved_symbol_at(cursor, ")") {
            if expecting_type && saw_type {
                self.diagnose_malformed_type_expression(cursor, "expected type before `)`");
                let missing = self.add_missing_type_expression(cursor);
                children.push(missing);
                recovery_nodes.push(missing);
            }
            children.push(self.token_node_ids[cursor]);
            cursor += 1;
        } else {
            self.diagnose_unmatched_type_argument_opener(opener, cursor);
            let recovery = self.add_recovery_node(
                SyntaxRecoveryKind::UnmatchedOpeningDelimiter,
                self.zero_range_at(cursor),
                Vec::new(),
            );
            children.push(recovery);
            recovery_nodes.push(recovery);
        }

        cursor
    }

    fn parse_template_type_expression_at(&mut self, position: usize) -> Option<ParsedTypeNode> {
        if let Some(type_expression) = self.parse_type_expression_at(position) {
            return Some(type_expression);
        }
        if !(self.is_identifier_at(position) || self.is_user_symbol_at(position)) {
            return None;
        }

        let head = self.events.emit(SyntaxEvent::Node {
            kind: SurfaceNodeKind::TypeHead,
            range: self.covering_token_range(position, position + 1),
            children: vec![self.token_node_ids[position]],
        });
        let id = self.events.emit(SyntaxEvent::Node {
            kind: SurfaceNodeKind::TypeExpression,
            range: self.covering_token_range(position, position + 1),
            children: vec![head],
        });
        Some(ParsedTypeNode {
            id,
            next_position: position + 1,
            recovery_nodes: Vec::new(),
        })
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

    fn parse_visible_algorithm_definition_at(&mut self, position: usize) -> ParsedTypeNode {
        let algorithm = self.parse_algorithm_definition_at(position + 1);
        self.wrap_visible_definition_content(position, algorithm)
    }

    fn parse_visible_registration_item_at(&mut self, position: usize) -> ParsedTypeNode {
        let registration = self.parse_registration_item_at(position + 1);
        self.wrap_visible_definition_content(position, registration)
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

        if self.is_identifier_at(cursor) && self.is_reserved_symbol_at(cursor + 1, ":") {
            children.push(self.token_node_ids[cursor]);
            cursor += 1;
            children.push(self.token_node_ids[cursor]);
            cursor += 1;
        } else if self.is_reserved_symbol_at(cursor, ":") {
            self.diagnose_malformed_term_expression(cursor, "expected predicate definition label");
            self.push_missing_term(cursor, &mut children, &mut recovery_nodes);
            children.push(self.token_node_ids[cursor]);
            cursor += 1;
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
        let mut recovery_nodes = Vec::new();
        let mut children =
            self.pattern_children_with_template_loci(position, cursor, &mut recovery_nodes);

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

        if self.is_identifier_at(cursor) && self.is_reserved_symbol_at(cursor + 1, ":") {
            children.push(self.token_node_ids[cursor]);
            cursor += 1;
            children.push(self.token_node_ids[cursor]);
            cursor += 1;
        } else if self.is_reserved_symbol_at(cursor, ":") {
            self.diagnose_malformed_term_expression(cursor, "expected functor definition label");
            self.push_missing_term(cursor, &mut children, &mut recovery_nodes);
            children.push(self.token_node_ids[cursor]);
            cursor += 1;
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

        if let Some(return_type) = self.parse_template_type_expression_at(cursor) {
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
        let mut recovery_nodes = Vec::new();
        let mut children =
            self.pattern_children_with_template_loci(position, cursor, &mut recovery_nodes);

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

    fn pattern_children_with_template_loci(
        &mut self,
        position: usize,
        end: usize,
        recovery_nodes: &mut Vec<SurfaceBuilderNodeId>,
    ) -> Vec<SurfaceBuilderNodeId> {
        let mut children = Vec::new();
        let mut cursor = position;
        while cursor < end {
            if self.is_reserved_symbol_at(cursor, "[") {
                let loci = self.parse_template_loci_at(cursor);
                let made_progress = loci.next_position > cursor;
                cursor = loci.next_position.min(end);
                children.push(loci.id);
                recovery_nodes.extend(loci.recovery_nodes);
                if !made_progress {
                    break;
                }
            } else {
                children.push(self.token_node_ids[cursor]);
                cursor += 1;
            }
        }
        children
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

    fn parse_algorithm_definition_at(&mut self, position: usize) -> ParsedTypeNode {
        let mut children = Vec::new();
        let mut recovery_nodes = Vec::new();
        let mut cursor = position;

        if self.is_reserved_word_at(cursor, "terminating") {
            let termination = self.parse_algorithm_termination_clause_at(cursor);
            cursor = termination.next_position;
            children.push(termination.id);
            recovery_nodes.extend(termination.recovery_nodes);
        }

        if self.is_reserved_word_at(cursor, "algorithm") {
            children.push(self.token_node_ids[cursor]);
            cursor += 1;
        } else {
            self.diagnose_malformed_formula_expression(cursor, "expected `algorithm` keyword");
        }

        if self.is_identifier_at(cursor) {
            children.push(self.token_node_ids[cursor]);
            cursor += 1;
        } else {
            self.diagnose_malformed_term_expression(cursor, "expected algorithm name");
            self.push_missing_term(cursor, &mut children, &mut recovery_nodes);
        }

        if self.is_reserved_symbol_at(cursor, "[") {
            let loci = self.parse_template_loci_at(cursor);
            cursor = loci.next_position;
            children.push(loci.id);
            recovery_nodes.extend(loci.recovery_nodes);
        }

        let parameters = self.parse_algorithm_parameters_at(cursor);
        cursor = parameters.next_position;
        children.push(parameters.id);
        recovery_nodes.extend(parameters.recovery_nodes);

        if self.is_reserved_symbol_at(cursor, "->") {
            children.push(self.token_node_ids[cursor]);
            cursor += 1;
            if let Some(return_type) = self.parse_type_expression_at(cursor) {
                cursor = return_type.next_position;
                children.push(return_type.id);
                recovery_nodes.extend(return_type.recovery_nodes);
            } else {
                self.diagnose_malformed_type_expression(cursor, "expected algorithm return type");
                let missing = self.add_missing_type_expression(cursor);
                children.push(missing);
                recovery_nodes.push(missing);
            }
        }

        cursor = self.parse_algorithm_header_clauses_at(cursor, &mut children, &mut recovery_nodes);

        let body = self.parse_algorithm_body_at(cursor);
        cursor = body.next_position;
        children.push(body.id);
        recovery_nodes.extend(body.recovery_nodes);

        if self.is_semicolon_at(cursor) {
            children.push(self.token_node_ids[cursor]);
            cursor += 1;
        } else {
            self.diagnose_missing_semicolon(cursor);
        }

        let id = self.events.emit(SyntaxEvent::Node {
            kind: SurfaceNodeKind::AlgorithmDefinition,
            range: self.covering_token_range(position, cursor),
            children,
        });
        ParsedTypeNode {
            id,
            next_position: cursor.max(position + 1),
            recovery_nodes,
        }
    }

    fn parse_algorithm_termination_clause_at(&mut self, position: usize) -> ParsedTypeNode {
        let id = self.events.emit(SyntaxEvent::Node {
            kind: SurfaceNodeKind::AlgorithmTerminationClause,
            range: self.covering_token_range(position, position + 1),
            children: vec![self.token_node_ids[position]],
        });
        ParsedTypeNode {
            id,
            next_position: position + 1,
            recovery_nodes: Vec::new(),
        }
    }

    fn parse_algorithm_header_clauses_at(
        &mut self,
        mut cursor: usize,
        children: &mut Vec<SurfaceBuilderNodeId>,
        recovery_nodes: &mut Vec<SurfaceBuilderNodeId>,
    ) -> usize {
        if self.is_reserved_word_at(cursor, "requires") {
            let clause = self.parse_algorithm_formula_clause_at(
                cursor,
                SurfaceNodeKind::AlgorithmRequiresClause,
                "expected formula after `requires`",
            );
            cursor = clause.next_position;
            children.push(clause.id);
            recovery_nodes.extend(clause.recovery_nodes);
        }

        if self.is_reserved_word_at(cursor, "ensures") {
            let clause = self.parse_algorithm_formula_clause_at(
                cursor,
                SurfaceNodeKind::AlgorithmEnsuresClause,
                "expected formula after `ensures`",
            );
            cursor = clause.next_position;
            children.push(clause.id);
            recovery_nodes.extend(clause.recovery_nodes);
        }

        if self.is_reserved_word_at(cursor, "decreasing") {
            let clause = self.parse_algorithm_decreasing_clause_at(cursor);
            cursor = clause.next_position;
            children.push(clause.id);
            recovery_nodes.extend(clause.recovery_nodes);
        }

        if self.is_algorithm_header_verification_keyword_at(cursor) {
            self.diagnose_malformed_formula_expression(
                cursor,
                "duplicate or out-of-order algorithm verification clause",
            );
            if let Some(recovery) = self.recover_malformed_algorithm_header_tail(cursor) {
                cursor = recovery.next_position;
                children.push(recovery.id);
                recovery_nodes.extend(recovery.recovery_nodes);
            }
        }

        cursor
    }

    fn parse_algorithm_formula_clause_at(
        &mut self,
        position: usize,
        kind: SurfaceNodeKind,
        missing_message: &'static str,
    ) -> ParsedTypeNode {
        let mut children = vec![self.token_node_ids[position]];
        let mut recovery_nodes = Vec::new();
        let mut cursor = position + 1;
        cursor = self.parse_required_algorithm_formula(
            cursor,
            &mut children,
            &mut recovery_nodes,
            missing_message,
        );
        let id = self.events.emit(SyntaxEvent::Node {
            kind,
            range: self.covering_token_range(position, cursor.max(position + 1)),
            children,
        });
        ParsedTypeNode {
            id,
            next_position: cursor.max(position + 1),
            recovery_nodes,
        }
    }

    fn parse_algorithm_decreasing_clause_at(&mut self, position: usize) -> ParsedTypeNode {
        let mut children = vec![self.token_node_ids[position]];
        let mut recovery_nodes = Vec::new();
        let term_list = self.parse_algorithm_term_list_at(
            position + 1,
            AlgorithmTermListBoundary::HeaderClause,
            "expected decreasing measure",
        );
        let cursor = term_list.next_position;
        children.push(term_list.id);
        recovery_nodes.extend(term_list.recovery_nodes);
        let id = self.events.emit(SyntaxEvent::Node {
            kind: SurfaceNodeKind::AlgorithmDecreasingClause,
            range: self.covering_token_range(position, cursor.max(position + 1)),
            children,
        });
        ParsedTypeNode {
            id,
            next_position: cursor.max(position + 1),
            recovery_nodes,
        }
    }

    fn parse_algorithm_parameters_at(&mut self, position: usize) -> ParsedTypeNode {
        let mut children = Vec::new();
        let mut recovery_nodes = Vec::new();
        let mut cursor = position;

        if self.is_reserved_symbol_at(cursor, "(") {
            let opener = cursor;
            children.push(self.token_node_ids[cursor]);
            cursor += 1;
            let mut expecting_parameter = true;
            let mut saw_parameter = false;
            while cursor < self.request.tokens.len() && !self.is_reserved_symbol_at(cursor, ")") {
                if self.is_reserved_symbol_at(cursor, ",") {
                    if expecting_parameter {
                        self.diagnose_malformed_term_expression(
                            cursor,
                            "expected algorithm parameter before `,`",
                        );
                        self.push_missing_term(cursor, &mut children, &mut recovery_nodes);
                    }
                    children.push(self.token_node_ids[cursor]);
                    cursor += 1;
                    expecting_parameter = true;
                    continue;
                }

                if self.is_identifier_at(cursor) {
                    children.push(self.token_node_ids[cursor]);
                    cursor += 1;
                    expecting_parameter = false;
                    saw_parameter = true;
                } else {
                    self.diagnose_malformed_term_expression(cursor, "expected algorithm parameter");
                    self.push_missing_term(cursor, &mut children, &mut recovery_nodes);
                    break;
                }

                if self.is_reserved_symbol_at(cursor, ",") {
                    children.push(self.token_node_ids[cursor]);
                    cursor += 1;
                    expecting_parameter = true;
                } else {
                    break;
                }
            }

            if self.is_reserved_symbol_at(cursor, ")") {
                if expecting_parameter && saw_parameter {
                    self.diagnose_malformed_term_expression(
                        cursor,
                        "expected algorithm parameter before `)`",
                    );
                    self.push_missing_term(cursor, &mut children, &mut recovery_nodes);
                }
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
                "expected `(` before algorithm parameter list",
            );
            let recovery = self.add_recovery_node(
                SyntaxRecoveryKind::UnmatchedOpeningDelimiter,
                self.zero_range_at(cursor),
                Vec::new(),
            );
            children.push(recovery);
            recovery_nodes.push(recovery);
        }

        let id = self.events.emit(SyntaxEvent::Node {
            kind: SurfaceNodeKind::AlgorithmParameters,
            range: if cursor > position {
                self.covering_token_range(position, cursor)
            } else {
                self.zero_range_at(position)
            },
            children,
        });
        ParsedTypeNode {
            id,
            next_position: cursor.max(position),
            recovery_nodes,
        }
    }

    fn parse_algorithm_body_at(&mut self, position: usize) -> ParsedTypeNode {
        let mut children = Vec::new();
        let mut recovery_nodes = Vec::new();
        let mut cursor = position;

        if self.is_reserved_word_at(cursor, "do") {
            children.push(self.token_node_ids[cursor]);
            cursor += 1;
        } else {
            self.diagnose_malformed_formula_expression(
                cursor,
                "expected `do` before algorithm body",
            );
        }

        let statements = self.parse_algorithm_statement_list_at(cursor);
        cursor = statements.next_position;
        children.push(statements.id);
        recovery_nodes.extend(statements.recovery_nodes);

        if self.is_end_keyword_at(cursor) {
            children.push(self.token_node_ids[cursor]);
            cursor += 1;
        } else {
            let opener = position.min(self.request.tokens.len().saturating_sub(1));
            let missing = self.add_missing_end(opener, cursor);
            children.push(missing);
            recovery_nodes.push(missing);
        }

        let id = self.events.emit(SyntaxEvent::Node {
            kind: SurfaceNodeKind::AlgorithmBody,
            range: if cursor > position {
                self.covering_token_range(position, cursor)
            } else {
                self.zero_range_at(position)
            },
            children,
        });
        ParsedTypeNode {
            id,
            next_position: cursor.max(position),
            recovery_nodes,
        }
    }

    fn parse_algorithm_statement_list_at(&mut self, position: usize) -> ParsedTypeNode {
        self.parse_algorithm_statement_list_with_boundary_at(
            position,
            AlgorithmStatementListBoundary::Body,
        )
    }

    fn parse_algorithm_statement_list_with_boundary_at(
        &mut self,
        position: usize,
        boundary: AlgorithmStatementListBoundary,
    ) -> ParsedTypeNode {
        let mut children = Vec::new();
        let mut recovery_nodes = Vec::new();
        let mut cursor = position;

        while cursor < self.request.tokens.len()
            && !self.is_end_keyword_at(cursor)
            && !self.is_algorithm_statement_list_context_boundary_at(cursor, boundary)
        {
            if self.is_item_start_at(cursor) {
                self.diagnose_malformed_formula_expression(
                    cursor,
                    "top-level items are not algorithm statements",
                );
                if let Some(recovery) = self.recover_malformed_algorithm_misplaced_item_tail(cursor)
                {
                    cursor = recovery.next_position;
                    children.push(recovery.id);
                    recovery_nodes.extend(recovery.recovery_nodes);
                    if self.is_semicolon_at(cursor) {
                        children.push(self.token_node_ids[cursor]);
                        cursor += 1;
                    }
                    continue;
                }
                break;
            }

            if self.is_algorithm_statement_list_boundary_for_at(cursor, boundary) {
                break;
            }

            if let Some(statement) = self.parse_algorithm_statement_at(cursor, boundary) {
                let made_progress = statement.next_position > cursor;
                cursor = statement.next_position;
                children.push(statement.id);
                recovery_nodes.extend(statement.recovery_nodes);
                if !made_progress {
                    break;
                }
                continue;
            }

            self.diagnose_malformed_formula_expression(cursor, "expected algorithm statement");
            if let Some(recovery) =
                self.recover_malformed_algorithm_statement_tail(cursor, boundary)
            {
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

        let range = if cursor > position {
            self.covering_token_range(position, cursor)
        } else {
            self.zero_range_at(position)
        };
        let id = self.events.emit(SyntaxEvent::Node {
            kind: SurfaceNodeKind::AlgorithmStatementList,
            range,
            children,
        });
        ParsedTypeNode {
            id,
            next_position: cursor.max(position),
            recovery_nodes,
        }
    }

    fn parse_algorithm_statement_at(
        &mut self,
        position: usize,
        boundary: AlgorithmStatementListBoundary,
    ) -> Option<ParsedTypeNode> {
        if let Some(annotation) = self.parse_standalone_diagnostic_annotation_at(position) {
            return Some(annotation);
        }

        let prefix = self.parse_leading_annotations_at(position);
        if prefix.next_position > position {
            let statement = self.parse_algorithm_statement_core_at(prefix.next_position, boundary);
            return Some(self.finish_annotated_type_node(
                position,
                prefix,
                statement,
                SurfaceNodeKind::AnnotatedAlgorithmStatement,
                "expected algorithm statement after annotation",
            ));
        }

        self.parse_algorithm_statement_core_at(position, boundary)
    }

    fn parse_algorithm_statement_core_at(
        &mut self,
        position: usize,
        boundary: AlgorithmStatementListBoundary,
    ) -> Option<ParsedTypeNode> {
        if self.is_reserved_word_at(position, "if") {
            return Some(self.parse_if_statement_at(position));
        }
        if self.is_reserved_word_at(position, "while") {
            return Some(self.parse_while_statement_at(position));
        }
        if self.is_reserved_word_at(position, "for") {
            return Some(self.parse_for_statement_at(position, boundary));
        }
        if self.is_reserved_word_at(position, "match") {
            return Some(self.parse_match_statement_at(position));
        }
        if self.is_reserved_word_at(position, "break") {
            return Some(self.parse_break_statement_at(position, boundary));
        }
        if self.is_reserved_word_at(position, "continue") {
            return Some(self.parse_continue_statement_at(position, boundary));
        }
        if self.is_reserved_word_at(position, "assert") {
            return Some(self.parse_assert_statement_at(position, boundary));
        }
        if self.is_reserved_word_at(position, "ghost")
            && (self.is_reserved_word_at(position + 1, "var")
                || self.is_reserved_word_at(position + 1, "const"))
        {
            return Some(self.parse_variable_declaration_at(position, boundary));
        }
        if self.is_reserved_word_at(position, "var") || self.is_reserved_word_at(position, "const")
        {
            return Some(self.parse_variable_declaration_at(position, boundary));
        }
        if self.is_reserved_word_at(position, "snapshot") {
            return Some(self.parse_snapshot_statement_at(position, boundary));
        }
        if self.is_reserved_word_at(position, "return") {
            return Some(self.parse_return_statement_at(position, boundary));
        }
        if self.is_reserved_word_at(position, "ghost") {
            return Some(self.parse_assignment_statement_at(position, true, boundary));
        }
        if self.is_lvalue_start_at(position)
            && self.lvalue_assignment_operator_at(position).is_some()
        {
            return Some(self.parse_assignment_statement_at(position, false, boundary));
        }
        None
    }

    fn parse_variable_declaration_at(
        &mut self,
        position: usize,
        boundary: AlgorithmStatementListBoundary,
    ) -> ParsedTypeNode {
        let mut children = Vec::new();
        let mut recovery_nodes = Vec::new();
        let mut cursor = position;
        let mut ghost = false;

        if self.is_reserved_word_at(cursor, "ghost") {
            ghost = true;
            children.push(self.token_node_ids[cursor]);
            cursor += 1;
        }

        let is_const = self.is_reserved_word_at(cursor, "const");
        if is_const || self.is_reserved_word_at(cursor, "var") {
            children.push(self.token_node_ids[cursor]);
            cursor += 1;
        } else {
            self.diagnose_malformed_term_expression(
                cursor,
                if ghost {
                    "expected `var` or `const` after `ghost`"
                } else {
                    "expected variable declaration keyword"
                },
            );
        }

        cursor = self.parse_algorithm_variable_bindings_at(
            cursor,
            is_const,
            &mut children,
            &mut recovery_nodes,
        );

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
                    "expected declaration type after `as`",
                );
                let missing = self.add_missing_type_expression(cursor);
                children.push(missing);
                recovery_nodes.push(missing);
            }
            if let Some(justification) =
                self.parse_definition_content_general_justification_at(cursor, true)
            {
                cursor = justification.next_position;
                children.push(justification.id);
                recovery_nodes.extend(justification.recovery_nodes);
            }
        }

        cursor = self.finish_algorithm_statement_semicolon(
            cursor,
            &mut children,
            &mut recovery_nodes,
            boundary,
            "unexpected token in variable declaration",
        );

        let id = self.events.emit(SyntaxEvent::Node {
            kind: SurfaceNodeKind::VariableDeclaration,
            range: self.covering_token_range(position, cursor),
            children,
        });
        ParsedTypeNode {
            id,
            next_position: cursor.max(position + 1),
            recovery_nodes,
        }
    }

    fn parse_algorithm_variable_bindings_at(
        &mut self,
        mut cursor: usize,
        is_const: bool,
        children: &mut Vec<SurfaceBuilderNodeId>,
        recovery_nodes: &mut Vec<SurfaceBuilderNodeId>,
    ) -> usize {
        let mut expecting_binding = true;
        let mut saw_binding = false;
        loop {
            if self.is_algorithm_declaration_tail_boundary_at(cursor) {
                if expecting_binding {
                    self.diagnose_malformed_term_expression(
                        cursor,
                        if saw_binding {
                            "expected declaration binding after `,`"
                        } else {
                            "expected declaration binding"
                        },
                    );
                    self.push_missing_variable_binding(cursor, children, recovery_nodes);
                }
                break;
            }

            if self.is_reserved_symbol_at(cursor, ",") {
                if expecting_binding {
                    self.diagnose_malformed_term_expression(
                        cursor,
                        "expected declaration binding before `,`",
                    );
                    self.push_missing_variable_binding(cursor, children, recovery_nodes);
                }
                children.push(self.token_node_ids[cursor]);
                cursor += 1;
                expecting_binding = true;
                continue;
            }

            let binding = self.parse_variable_binding_at(cursor, is_const);
            let made_progress = binding.next_position > cursor;
            cursor = binding.next_position;
            children.push(binding.id);
            recovery_nodes.extend(binding.recovery_nodes);
            saw_binding = true;
            if !made_progress {
                break;
            }

            if self.is_reserved_symbol_at(cursor, ",") {
                children.push(self.token_node_ids[cursor]);
                cursor += 1;
                expecting_binding = true;
            } else {
                break;
            }
        }
        cursor
    }

    fn parse_variable_binding_at(&mut self, position: usize, is_const: bool) -> ParsedTypeNode {
        let mut children = Vec::new();
        let mut recovery_nodes = Vec::new();
        let mut cursor = position;

        if self.is_identifier_at(cursor) {
            children.push(self.token_node_ids[cursor]);
            cursor += 1;
        } else {
            self.diagnose_malformed_term_expression(cursor, "expected declaration binding name");
            self.push_missing_term(cursor, &mut children, &mut recovery_nodes);
            let id = self.events.emit(SyntaxEvent::Node {
                kind: SurfaceNodeKind::VariableBinding,
                range: self.zero_range_at(position),
                children,
            });
            return ParsedTypeNode {
                id,
                next_position: cursor,
                recovery_nodes,
            };
        }

        if self.is_reserved_symbol_at(cursor, ":=") {
            children.push(self.token_node_ids[cursor]);
            cursor += 1;
            if let Some(term) = self.parse_term_expression_at(cursor) {
                cursor = term.next_position;
                children.push(term.id);
                recovery_nodes.extend(term.recovery_nodes);
            } else {
                self.diagnose_malformed_term_expression(
                    cursor,
                    "expected initializer term after `:=`",
                );
                self.push_missing_term(cursor, &mut children, &mut recovery_nodes);
            }
        } else if is_const {
            self.diagnose_malformed_term_expression(
                cursor,
                "expected `:=` in const declaration binding",
            );
            self.push_missing_term(cursor, &mut children, &mut recovery_nodes);
        }

        let id = self.events.emit(SyntaxEvent::Node {
            kind: SurfaceNodeKind::VariableBinding,
            range: self.covering_token_range(position, cursor),
            children,
        });
        ParsedTypeNode {
            id,
            next_position: cursor.max(position + 1),
            recovery_nodes,
        }
    }

    fn push_missing_variable_binding(
        &mut self,
        position: usize,
        children: &mut Vec<SurfaceBuilderNodeId>,
        recovery_nodes: &mut Vec<SurfaceBuilderNodeId>,
    ) {
        let missing = self.add_missing_term(position);
        let binding = self.events.emit(SyntaxEvent::Node {
            kind: SurfaceNodeKind::VariableBinding,
            range: self.zero_range_at(position),
            children: vec![missing],
        });
        children.push(binding);
        recovery_nodes.push(missing);
    }

    fn parse_assignment_statement_at(
        &mut self,
        position: usize,
        ghost_prefix: bool,
        boundary: AlgorithmStatementListBoundary,
    ) -> ParsedTypeNode {
        let mut children = Vec::new();
        let mut recovery_nodes = Vec::new();
        let mut cursor = position;

        if ghost_prefix {
            children.push(self.token_node_ids[cursor]);
            cursor += 1;
        }

        let lvalue = self.parse_lvalue_at(cursor);
        cursor = lvalue.next_position;
        children.push(lvalue.id);
        recovery_nodes.extend(lvalue.recovery_nodes);

        if self.is_reserved_symbol_at(cursor, ":=") {
            children.push(self.token_node_ids[cursor]);
            cursor += 1;
        } else {
            self.diagnose_malformed_term_expression(cursor, "expected `:=` in assignment");
        }

        if let Some(term) = self.parse_term_expression_at(cursor) {
            cursor = term.next_position;
            children.push(term.id);
            recovery_nodes.extend(term.recovery_nodes);
        } else {
            self.diagnose_malformed_term_expression(cursor, "expected assignment term");
            self.push_missing_term(cursor, &mut children, &mut recovery_nodes);
        }

        cursor = self.finish_algorithm_statement_semicolon(
            cursor,
            &mut children,
            &mut recovery_nodes,
            boundary,
            "unexpected token in assignment",
        );

        let id = self.events.emit(SyntaxEvent::Node {
            kind: SurfaceNodeKind::AssignmentStatement,
            range: self.covering_token_range(position, cursor),
            children,
        });
        ParsedTypeNode {
            id,
            next_position: cursor.max(position + 1),
            recovery_nodes,
        }
    }

    fn parse_lvalue_at(&mut self, position: usize) -> ParsedTypeNode {
        let mut children = Vec::new();
        let mut recovery_nodes = Vec::new();
        let mut cursor = position;

        if self.is_identifier_at(cursor) {
            children.push(self.token_node_ids[cursor]);
            cursor += 1;
        } else {
            self.diagnose_malformed_term_expression(cursor, "expected assignment target");
            self.push_missing_term(cursor, &mut children, &mut recovery_nodes);
        }

        while self.is_reserved_symbol_at(cursor, ".") {
            children.push(self.token_node_ids[cursor]);
            cursor += 1;
            if self.is_identifier_at(cursor) {
                children.push(self.token_node_ids[cursor]);
                cursor += 1;
            } else {
                self.diagnose_malformed_term_expression(cursor, "expected field name after `.`");
                self.push_missing_term(cursor, &mut children, &mut recovery_nodes);
                break;
            }
        }

        let id = self.events.emit(SyntaxEvent::Node {
            kind: SurfaceNodeKind::Lvalue,
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

    fn parse_snapshot_statement_at(
        &mut self,
        position: usize,
        boundary: AlgorithmStatementListBoundary,
    ) -> ParsedTypeNode {
        let mut children = vec![self.token_node_ids[position]];
        let mut recovery_nodes = Vec::new();
        let mut cursor = position + 1;

        if self.is_identifier_at(cursor) {
            children.push(self.token_node_ids[cursor]);
            cursor += 1;
        } else {
            self.diagnose_malformed_term_expression(cursor, "expected snapshot name");
            self.push_missing_term(cursor, &mut children, &mut recovery_nodes);
        }

        cursor = self.finish_algorithm_statement_semicolon(
            cursor,
            &mut children,
            &mut recovery_nodes,
            boundary,
            "unexpected token in snapshot statement",
        );

        let id = self.events.emit(SyntaxEvent::Node {
            kind: SurfaceNodeKind::SnapshotStatement,
            range: self.covering_token_range(position, cursor),
            children,
        });
        ParsedTypeNode {
            id,
            next_position: cursor.max(position + 1),
            recovery_nodes,
        }
    }

    fn parse_return_statement_at(
        &mut self,
        position: usize,
        boundary: AlgorithmStatementListBoundary,
    ) -> ParsedTypeNode {
        let mut children = vec![self.token_node_ids[position]];
        let mut recovery_nodes = Vec::new();
        let mut cursor = position + 1;

        if !self.is_semicolon_at(cursor)
            && !self.is_algorithm_statement_list_boundary_for_at(cursor, boundary)
            && let Some(term) = self.parse_term_expression_at(cursor)
        {
            cursor = term.next_position;
            children.push(term.id);
            recovery_nodes.extend(term.recovery_nodes);
            if let Some(justification) =
                self.parse_definition_content_general_justification_at(cursor, true)
            {
                cursor = justification.next_position;
                children.push(justification.id);
                recovery_nodes.extend(justification.recovery_nodes);
            }
        }

        cursor = self.finish_algorithm_statement_semicolon(
            cursor,
            &mut children,
            &mut recovery_nodes,
            boundary,
            "unexpected token in return statement",
        );

        let id = self.events.emit(SyntaxEvent::Node {
            kind: SurfaceNodeKind::ReturnStatement,
            range: self.covering_token_range(position, cursor),
            children,
        });
        ParsedTypeNode {
            id,
            next_position: cursor.max(position + 1),
            recovery_nodes,
        }
    }

    fn parse_if_statement_at(&mut self, position: usize) -> ParsedTypeNode {
        let mut children = vec![self.token_node_ids[position]];
        let mut recovery_nodes = Vec::new();
        let mut cursor = position + 1;

        if let Some(condition) = self.parse_formula_expression_at(cursor) {
            cursor = condition.next_position;
            children.push(condition.id);
            recovery_nodes.extend(condition.recovery_nodes);
        } else {
            self.diagnose_malformed_formula_expression(cursor, "expected `if` condition");
            self.push_missing_formula(cursor, &mut children, &mut recovery_nodes);
        }

        cursor = self.expect_algorithm_do_keyword(
            cursor,
            &mut children,
            "expected `do` after `if` condition",
        );

        let then_statements = self.parse_algorithm_statement_list_with_boundary_at(
            cursor,
            AlgorithmStatementListBoundary::IfThen,
        );
        cursor = then_statements.next_position;
        children.push(then_statements.id);
        recovery_nodes.extend(then_statements.recovery_nodes);

        if self.is_reserved_word_at(cursor, "else") {
            children.push(self.token_node_ids[cursor]);
            cursor += 1;
            if self.is_reserved_word_at(cursor, "if") {
                let nested = self.parse_if_statement_at(cursor);
                cursor = nested.next_position;
                children.push(nested.id);
                recovery_nodes.extend(nested.recovery_nodes);
            } else {
                let else_statements = self.parse_algorithm_statement_list_with_boundary_at(
                    cursor,
                    AlgorithmStatementListBoundary::NestedBlock,
                );
                cursor = else_statements.next_position;
                children.push(else_statements.id);
                recovery_nodes.extend(else_statements.recovery_nodes);
                cursor = self.finish_algorithm_block_end_semicolon(
                    position,
                    cursor,
                    &mut children,
                    &mut recovery_nodes,
                );
            }
        } else {
            cursor = self.finish_algorithm_block_end_semicolon(
                position,
                cursor,
                &mut children,
                &mut recovery_nodes,
            );
        }

        let id = self.events.emit(SyntaxEvent::Node {
            kind: SurfaceNodeKind::IfStatement,
            range: self.covering_token_range(position, cursor),
            children,
        });
        ParsedTypeNode {
            id,
            next_position: cursor.max(position + 1),
            recovery_nodes,
        }
    }

    fn parse_while_statement_at(&mut self, position: usize) -> ParsedTypeNode {
        let mut children = vec![self.token_node_ids[position]];
        let mut recovery_nodes = Vec::new();
        let mut cursor = position + 1;

        if let Some(condition) = self.parse_formula_expression_at(cursor) {
            cursor = condition.next_position;
            children.push(condition.id);
            recovery_nodes.extend(condition.recovery_nodes);
        } else {
            self.diagnose_malformed_formula_expression(cursor, "expected `while` condition");
            self.push_missing_formula(cursor, &mut children, &mut recovery_nodes);
        }

        cursor = self.expect_algorithm_do_keyword(
            cursor,
            &mut children,
            "expected `do` after `while` condition",
        );
        cursor = self.parse_loop_verification_clauses_at(
            cursor,
            true,
            &mut children,
            &mut recovery_nodes,
        );

        let statements = self.parse_algorithm_statement_list_with_boundary_at(
            cursor,
            AlgorithmStatementListBoundary::NestedBlock,
        );
        cursor = statements.next_position;
        children.push(statements.id);
        recovery_nodes.extend(statements.recovery_nodes);
        cursor = self.finish_algorithm_block_end_semicolon(
            position,
            cursor,
            &mut children,
            &mut recovery_nodes,
        );

        let id = self.events.emit(SyntaxEvent::Node {
            kind: SurfaceNodeKind::WhileStatement,
            range: self.covering_token_range(position, cursor),
            children,
        });
        ParsedTypeNode {
            id,
            next_position: cursor.max(position + 1),
            recovery_nodes,
        }
    }

    fn parse_for_statement_at(
        &mut self,
        position: usize,
        boundary: AlgorithmStatementListBoundary,
    ) -> ParsedTypeNode {
        let mut children = vec![self.token_node_ids[position]];
        let mut recovery_nodes = Vec::new();
        let mut cursor = position + 1;

        if self.is_identifier_at(cursor) {
            children.push(self.token_node_ids[cursor]);
            cursor += 1;
        } else {
            self.diagnose_malformed_term_expression(cursor, "expected loop variable");
            self.push_missing_term(cursor, &mut children, &mut recovery_nodes);
        }

        let kind = if self.is_reserved_symbol_at(cursor, "=") {
            children.push(self.token_node_ids[cursor]);
            cursor += 1;
            cursor = self.parse_required_algorithm_term(
                cursor,
                &mut children,
                &mut recovery_nodes,
                "expected range lower bound",
            );

            if self.is_reserved_word_at(cursor, "to") || self.is_reserved_word_at(cursor, "downto")
            {
                children.push(self.token_node_ids[cursor]);
                cursor += 1;
            } else {
                self.diagnose_malformed_term_expression(
                    cursor,
                    "expected `to` or `downto` in range loop",
                );
            }

            cursor = self.parse_required_algorithm_term(
                cursor,
                &mut children,
                &mut recovery_nodes,
                "expected range upper bound",
            );

            if self.is_reserved_word_at(cursor, "step") {
                children.push(self.token_node_ids[cursor]);
                cursor += 1;
                cursor = self.parse_required_algorithm_term(
                    cursor,
                    &mut children,
                    &mut recovery_nodes,
                    "expected step term",
                );
            }
            SurfaceNodeKind::ForRangeStatement
        } else if self.is_reserved_word_at(cursor, "in") {
            children.push(self.token_node_ids[cursor]);
            cursor += 1;
            cursor = self.parse_required_algorithm_term(
                cursor,
                &mut children,
                &mut recovery_nodes,
                "expected collection term",
            );

            if self.is_reserved_word_at(cursor, "processed") {
                children.push(self.token_node_ids[cursor]);
                cursor += 1;
                if self.is_identifier_at(cursor) {
                    children.push(self.token_node_ids[cursor]);
                    cursor += 1;
                } else {
                    self.diagnose_malformed_term_expression(
                        cursor,
                        "expected processed binding name",
                    );
                    self.push_missing_term(cursor, &mut children, &mut recovery_nodes);
                }
            }
            SurfaceNodeKind::ForCollectionStatement
        } else {
            self.diagnose_malformed_term_expression(
                cursor,
                "expected `=` or `in` after loop variable",
            );
            if let Some(recovery) =
                self.recover_malformed_algorithm_statement_tail(cursor, boundary)
            {
                cursor = recovery.next_position;
                children.push(recovery.id);
                recovery_nodes.extend(recovery.recovery_nodes);
            }
            SurfaceNodeKind::ForRangeStatement
        };

        cursor = self.expect_algorithm_do_keyword(
            cursor,
            &mut children,
            "expected `do` in loop statement",
        );
        cursor = self.parse_loop_verification_clauses_at(
            cursor,
            false,
            &mut children,
            &mut recovery_nodes,
        );

        let statements = self.parse_algorithm_statement_list_with_boundary_at(
            cursor,
            AlgorithmStatementListBoundary::NestedBlock,
        );
        cursor = statements.next_position;
        children.push(statements.id);
        recovery_nodes.extend(statements.recovery_nodes);
        cursor = self.finish_algorithm_block_end_semicolon(
            position,
            cursor,
            &mut children,
            &mut recovery_nodes,
        );

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

    fn parse_match_statement_at(&mut self, position: usize) -> ParsedTypeNode {
        let mut children = vec![self.token_node_ids[position]];
        let mut recovery_nodes = Vec::new();
        let mut cursor = position + 1;

        cursor = self.parse_required_algorithm_term(
            cursor,
            &mut children,
            &mut recovery_nodes,
            "expected match scrutinee",
        );
        cursor = self.expect_algorithm_do_keyword(
            cursor,
            &mut children,
            "expected `do` after match scrutinee",
        );

        let mut saw_case = false;
        while self.is_reserved_word_at(cursor, "case") {
            let case = self.parse_match_case_at(cursor);
            let made_progress = case.next_position > cursor;
            cursor = case.next_position;
            children.push(case.id);
            recovery_nodes.extend(case.recovery_nodes);
            saw_case = true;
            if !made_progress {
                break;
            }
        }
        if !saw_case {
            self.diagnose_malformed_term_expression(cursor, "expected match case");
            let missing = self.add_missing_statement(cursor);
            children.push(missing);
            recovery_nodes.push(missing);
        }

        if self.is_reserved_word_at(cursor, "otherwise")
            || self.is_reserved_word_at(cursor, "exhaustive")
        {
            let ending = self.parse_match_ending_at(cursor);
            cursor = ending.next_position;
            children.push(ending.id);
            recovery_nodes.extend(ending.recovery_nodes);
        } else {
            self.diagnose_malformed_formula_expression(
                cursor,
                "expected `otherwise` or `exhaustive` match ending",
            );
            let missing = self.add_missing_statement(cursor);
            let ending = self.events.emit(SyntaxEvent::Node {
                kind: SurfaceNodeKind::MatchEnding,
                range: self.zero_range_at(cursor),
                children: vec![missing],
            });
            children.push(ending);
            recovery_nodes.push(missing);
        }

        cursor = self.finish_algorithm_block_end_semicolon(
            position,
            cursor,
            &mut children,
            &mut recovery_nodes,
        );

        let id = self.events.emit(SyntaxEvent::Node {
            kind: SurfaceNodeKind::MatchStatement,
            range: self.covering_token_range(position, cursor),
            children,
        });
        ParsedTypeNode {
            id,
            next_position: cursor.max(position + 1),
            recovery_nodes,
        }
    }

    fn parse_match_case_at(&mut self, position: usize) -> ParsedTypeNode {
        let mut children = vec![self.token_node_ids[position]];
        let mut recovery_nodes = Vec::new();
        let mut cursor = position + 1;

        cursor = self.parse_required_algorithm_term(
            cursor,
            &mut children,
            &mut recovery_nodes,
            "expected match case pattern",
        );
        cursor = self.expect_algorithm_do_keyword(
            cursor,
            &mut children,
            "expected `do` after match case pattern",
        );

        let statements = self.parse_algorithm_statement_list_with_boundary_at(
            cursor,
            AlgorithmStatementListBoundary::MatchCase,
        );
        cursor = statements.next_position;
        children.push(statements.id);
        recovery_nodes.extend(statements.recovery_nodes);
        cursor = self.finish_algorithm_block_end_semicolon(
            position,
            cursor,
            &mut children,
            &mut recovery_nodes,
        );

        let id = self.events.emit(SyntaxEvent::Node {
            kind: SurfaceNodeKind::MatchCase,
            range: self.covering_token_range(position, cursor),
            children,
        });
        ParsedTypeNode {
            id,
            next_position: cursor.max(position + 1),
            recovery_nodes,
        }
    }

    fn parse_match_ending_at(&mut self, position: usize) -> ParsedTypeNode {
        let mut children = vec![self.token_node_ids[position]];
        let mut recovery_nodes = Vec::new();
        let mut cursor = position + 1;

        if self.is_reserved_word_at(position, "otherwise") {
            let statements = self.parse_algorithm_statement_list_with_boundary_at(
                cursor,
                AlgorithmStatementListBoundary::NestedBlock,
            );
            cursor = statements.next_position;
            children.push(statements.id);
            recovery_nodes.extend(statements.recovery_nodes);
            cursor = self.finish_algorithm_block_end_semicolon(
                position,
                cursor,
                &mut children,
                &mut recovery_nodes,
            );
        } else if let Some(justification) =
            self.parse_definition_content_general_justification_at(cursor, true)
        {
            cursor = justification.next_position;
            children.push(justification.id);
            recovery_nodes.extend(justification.recovery_nodes);
            if self.is_semicolon_at(cursor) {
                children.push(self.token_node_ids[cursor]);
                cursor += 1;
            } else {
                self.diagnose_missing_semicolon(cursor);
            }
        } else if self.is_semicolon_at(cursor) {
            children.push(self.token_node_ids[cursor]);
            cursor += 1;
        } else {
            self.diagnose_missing_semicolon(cursor);
        }

        let id = self.events.emit(SyntaxEvent::Node {
            kind: SurfaceNodeKind::MatchEnding,
            range: self.covering_token_range(position, cursor),
            children,
        });
        ParsedTypeNode {
            id,
            next_position: cursor.max(position + 1),
            recovery_nodes,
        }
    }

    fn parse_break_statement_at(
        &mut self,
        position: usize,
        boundary: AlgorithmStatementListBoundary,
    ) -> ParsedTypeNode {
        self.parse_algorithm_keyword_semicolon_statement_at(
            position,
            SurfaceNodeKind::BreakStatement,
            boundary,
            "unexpected token in break statement",
        )
    }

    fn parse_continue_statement_at(
        &mut self,
        position: usize,
        boundary: AlgorithmStatementListBoundary,
    ) -> ParsedTypeNode {
        self.parse_algorithm_keyword_semicolon_statement_at(
            position,
            SurfaceNodeKind::ContinueStatement,
            boundary,
            "unexpected token in continue statement",
        )
    }

    fn parse_assert_statement_at(
        &mut self,
        position: usize,
        boundary: AlgorithmStatementListBoundary,
    ) -> ParsedTypeNode {
        let mut children = vec![self.token_node_ids[position]];
        let mut recovery_nodes = Vec::new();
        let mut cursor = position + 1;

        cursor = self.parse_required_algorithm_formula(
            cursor,
            &mut children,
            &mut recovery_nodes,
            "expected assertion formula",
        );

        if let Some(justification) =
            self.parse_definition_content_general_justification_at(cursor, true)
        {
            cursor = justification.next_position;
            children.push(justification.id);
            recovery_nodes.extend(justification.recovery_nodes);
        }

        cursor = self.finish_algorithm_statement_semicolon(
            cursor,
            &mut children,
            &mut recovery_nodes,
            boundary,
            "unexpected token in assert statement",
        );

        let id = self.events.emit(SyntaxEvent::Node {
            kind: SurfaceNodeKind::AssertStatement,
            range: self.covering_token_range(position, cursor),
            children,
        });
        ParsedTypeNode {
            id,
            next_position: cursor.max(position + 1),
            recovery_nodes,
        }
    }

    fn parse_algorithm_keyword_semicolon_statement_at(
        &mut self,
        position: usize,
        kind: SurfaceNodeKind,
        boundary: AlgorithmStatementListBoundary,
        unexpected_message: &'static str,
    ) -> ParsedTypeNode {
        let mut children = vec![self.token_node_ids[position]];
        let mut recovery_nodes = Vec::new();
        let cursor = self.finish_algorithm_statement_semicolon(
            position + 1,
            &mut children,
            &mut recovery_nodes,
            boundary,
            unexpected_message,
        );
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

    fn parse_required_algorithm_term(
        &mut self,
        cursor: usize,
        children: &mut Vec<SurfaceBuilderNodeId>,
        recovery_nodes: &mut Vec<SurfaceBuilderNodeId>,
        message: &'static str,
    ) -> usize {
        if let Some(term) = self.parse_term_expression_at(cursor) {
            let next_position = term.next_position;
            children.push(term.id);
            recovery_nodes.extend(term.recovery_nodes);
            next_position
        } else {
            self.diagnose_malformed_term_expression(cursor, message);
            self.push_missing_term(cursor, children, recovery_nodes);
            cursor
        }
    }

    fn parse_required_algorithm_formula(
        &mut self,
        cursor: usize,
        children: &mut Vec<SurfaceBuilderNodeId>,
        recovery_nodes: &mut Vec<SurfaceBuilderNodeId>,
        message: &'static str,
    ) -> usize {
        if let Some(formula) = self.parse_formula_expression_at(cursor) {
            let next_position = formula.next_position;
            children.push(formula.id);
            recovery_nodes.extend(formula.recovery_nodes);
            next_position
        } else {
            self.diagnose_malformed_formula_expression(cursor, message);
            self.push_missing_formula(cursor, children, recovery_nodes);
            cursor
        }
    }

    fn parse_algorithm_term_list_at(
        &mut self,
        position: usize,
        boundary: AlgorithmTermListBoundary,
        missing_message: &'static str,
    ) -> ParsedTypeNode {
        let mut children = Vec::new();
        let mut recovery_nodes = Vec::new();
        let mut cursor = position;
        let mut saw_term = false;
        let mut expecting_term = true;

        loop {
            if self.is_algorithm_term_list_boundary_at(cursor, boundary) {
                if expecting_term {
                    let message = if saw_term {
                        "expected term after `,`"
                    } else {
                        missing_message
                    };
                    self.diagnose_malformed_term_expression(cursor, message);
                    self.push_missing_term(cursor, &mut children, &mut recovery_nodes);
                }
                break;
            }

            if self.is_reserved_symbol_at(cursor, ",") {
                if expecting_term {
                    self.diagnose_malformed_term_expression(
                        cursor,
                        "expected decreasing measure before `,`",
                    );
                    self.push_missing_term(cursor, &mut children, &mut recovery_nodes);
                }
                children.push(self.token_node_ids[cursor]);
                cursor += 1;
                expecting_term = true;
                continue;
            }

            if expecting_term {
                if let Some(term) = self.parse_term_expression_at(cursor) {
                    let made_progress = term.next_position > cursor;
                    cursor = term.next_position;
                    children.push(term.id);
                    recovery_nodes.extend(term.recovery_nodes);
                    saw_term = true;
                    expecting_term = false;
                    if !made_progress {
                        break;
                    }
                    continue;
                }

                self.diagnose_malformed_term_expression(cursor, missing_message);
                self.push_missing_term(cursor, &mut children, &mut recovery_nodes);
            } else {
                self.diagnose_malformed_term_expression(
                    cursor,
                    "expected `,` between decreasing measures",
                );
            }

            let Some(recovery) = self.recover_malformed_algorithm_term_list_tail(cursor, boundary)
            else {
                break;
            };
            let made_progress = recovery.next_position > cursor;
            cursor = recovery.next_position;
            children.push(recovery.id);
            recovery_nodes.extend(recovery.recovery_nodes);
            if !made_progress {
                break;
            }
        }

        let range = if cursor > position {
            self.covering_token_range(position, cursor)
        } else {
            self.zero_range_at(position)
        };
        let id = self.events.emit(SyntaxEvent::Node {
            kind: SurfaceNodeKind::TermList,
            range,
            children,
        });
        ParsedTypeNode {
            id,
            next_position: cursor.max(position),
            recovery_nodes,
        }
    }

    fn expect_algorithm_do_keyword(
        &mut self,
        cursor: usize,
        children: &mut Vec<SurfaceBuilderNodeId>,
        message: &'static str,
    ) -> usize {
        if self.is_reserved_word_at(cursor, "do") {
            children.push(self.token_node_ids[cursor]);
            cursor + 1
        } else {
            self.diagnose_malformed_formula_expression(cursor, message);
            cursor
        }
    }

    fn finish_algorithm_block_end_semicolon(
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

        if self.is_semicolon_at(cursor) {
            children.push(self.token_node_ids[cursor]);
            cursor += 1;
        } else {
            self.diagnose_missing_semicolon(cursor);
        }

        cursor
    }

    fn parse_loop_verification_clauses_at(
        &mut self,
        mut cursor: usize,
        allow_decreasing: bool,
        children: &mut Vec<SurfaceBuilderNodeId>,
        recovery_nodes: &mut Vec<SurfaceBuilderNodeId>,
    ) -> usize {
        while self.is_algorithm_loop_task34_clause_at(cursor) {
            if self.is_reserved_word_at(cursor, "invariant") {
                let clause = self.parse_loop_invariant_clause_at(cursor);
                let made_progress = clause.next_position > cursor;
                cursor = clause.next_position;
                children.push(clause.id);
                recovery_nodes.extend(clause.recovery_nodes);
                if !made_progress {
                    break;
                }
            } else if allow_decreasing {
                let clause = self.parse_loop_decreasing_clause_at(cursor);
                let made_progress = clause.next_position > cursor;
                cursor = clause.next_position;
                children.push(clause.id);
                recovery_nodes.extend(clause.recovery_nodes);
                if !made_progress {
                    break;
                }
            } else {
                self.diagnose_malformed_formula_expression(
                    cursor,
                    "range and collection loops accept only invariant verification clauses",
                );
                let Some(recovery) = self.recover_malformed_algorithm_loop_annotation_tail(cursor)
                else {
                    break;
                };
                let made_progress = recovery.next_position > cursor;
                cursor = recovery.next_position;
                children.push(recovery.id);
                recovery_nodes.extend(recovery.recovery_nodes);

                if self.is_semicolon_at(cursor) {
                    children.push(self.token_node_ids[cursor]);
                    cursor += 1;
                } else {
                    self.diagnose_missing_semicolon(cursor);
                }

                if !made_progress {
                    break;
                }
            }
        }
        cursor
    }

    fn parse_loop_invariant_clause_at(&mut self, position: usize) -> ParsedTypeNode {
        let mut children = vec![self.token_node_ids[position]];
        let mut recovery_nodes = Vec::new();
        let mut cursor = position + 1;
        cursor = self.parse_required_algorithm_formula(
            cursor,
            &mut children,
            &mut recovery_nodes,
            "expected invariant formula",
        );
        if let Some(justification) =
            self.parse_definition_content_general_justification_at(cursor, true)
        {
            cursor = justification.next_position;
            children.push(justification.id);
            recovery_nodes.extend(justification.recovery_nodes);
        }
        cursor = self.finish_algorithm_clause_semicolon(cursor, &mut children);
        let id = self.events.emit(SyntaxEvent::Node {
            kind: SurfaceNodeKind::LoopInvariantClause,
            range: self.covering_token_range(position, cursor),
            children,
        });
        ParsedTypeNode {
            id,
            next_position: cursor.max(position + 1),
            recovery_nodes,
        }
    }

    fn parse_loop_decreasing_clause_at(&mut self, position: usize) -> ParsedTypeNode {
        let mut children = vec![self.token_node_ids[position]];
        let mut recovery_nodes = Vec::new();
        let term_list = self.parse_algorithm_term_list_at(
            position + 1,
            AlgorithmTermListBoundary::ClauseStatement,
            "expected decreasing measure",
        );
        let mut cursor = term_list.next_position;
        children.push(term_list.id);
        recovery_nodes.extend(term_list.recovery_nodes);
        if let Some(justification) =
            self.parse_definition_content_general_justification_at(cursor, true)
        {
            cursor = justification.next_position;
            children.push(justification.id);
            recovery_nodes.extend(justification.recovery_nodes);
        }
        cursor = self.finish_algorithm_clause_semicolon(cursor, &mut children);
        let id = self.events.emit(SyntaxEvent::Node {
            kind: SurfaceNodeKind::LoopDecreasingClause,
            range: self.covering_token_range(position, cursor),
            children,
        });
        ParsedTypeNode {
            id,
            next_position: cursor.max(position + 1),
            recovery_nodes,
        }
    }

    fn finish_algorithm_clause_semicolon(
        &mut self,
        cursor: usize,
        children: &mut Vec<SurfaceBuilderNodeId>,
    ) -> usize {
        if self.is_semicolon_at(cursor) {
            children.push(self.token_node_ids[cursor]);
            cursor + 1
        } else {
            self.diagnose_missing_semicolon(cursor);
            cursor
        }
    }

    fn finish_algorithm_statement_semicolon(
        &mut self,
        mut cursor: usize,
        children: &mut Vec<SurfaceBuilderNodeId>,
        recovery_nodes: &mut Vec<SurfaceBuilderNodeId>,
        boundary: AlgorithmStatementListBoundary,
        unexpected_message: &'static str,
    ) -> usize {
        if self.is_semicolon_at(cursor) {
            children.push(self.token_node_ids[cursor]);
            return cursor + 1;
        }
        if cursor < self.request.tokens.len()
            && !self.is_algorithm_statement_list_boundary_for_at(cursor, boundary)
        {
            self.diagnose_malformed_term_expression(cursor, unexpected_message);
            if let Some(recovery) =
                self.recover_malformed_algorithm_statement_tail(cursor, boundary)
            {
                cursor = recovery.next_position;
                children.push(recovery.id);
                recovery_nodes.extend(recovery.recovery_nodes);
            }
            if self.is_semicolon_at(cursor) {
                children.push(self.token_node_ids[cursor]);
                return cursor + 1;
            }
        }
        self.diagnose_missing_semicolon(cursor);
        cursor
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

        if self.is_identifier_at(cursor) {
            children.push(self.token_node_ids[cursor]);
            cursor += 1;
        } else {
            self.diagnose_malformed_term_expression(
                cursor,
                "expected predicate redefinition label",
            );
            self.push_missing_term(cursor, &mut children, &mut recovery_nodes);
        }

        if self.is_reserved_symbol_at(cursor, ":") {
            children.push(self.token_node_ids[cursor]);
            cursor += 1;
        } else {
            self.diagnose_malformed_formula_expression(
                cursor,
                "expected `:` after predicate redefinition label",
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
        let prefix = self.parse_leading_annotations_at(position);
        let mut children = prefix.children;
        let mut recovery_nodes = prefix.recovery_nodes;
        children.extend(
            self.token_node_ids[prefix.next_position..=head]
                .iter()
                .copied(),
        );
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
        let prefix = self.parse_leading_annotations_at(position);
        let mut children = prefix.children;
        let mut recovery_nodes = prefix.recovery_nodes;
        children.extend(
            self.token_node_ids[prefix.next_position..=head]
                .iter()
                .copied(),
        );
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
        let head = position;
        let mut children = vec![self.token_node_ids[head]];
        let mut recovery_nodes = Vec::new();
        let mut cursor = head + 1;

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
        if !self.can_start_formula_at(position) {
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

        self.predicate_head_next_at(cursor)?;
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
        let (head, mut cursor) = if let Some(symbol) = self.parse_qualified_symbol_at(position) {
            (symbol.id, symbol.next_position)
        } else if self.is_identifier_at(position) && self.is_reserved_symbol_at(position + 1, "[") {
            (self.token_node_ids[position], position + 1)
        } else {
            return None;
        };
        let mut children = vec![head];
        let mut recovery_nodes = Vec::new();
        if self.is_reserved_symbol_at(cursor, "[") {
            let arguments = self.parse_template_arguments_at(cursor);
            cursor = arguments.next_position;
            children.push(arguments.id);
            recovery_nodes.extend(arguments.recovery_nodes);
        }
        let id = self.events.emit(SyntaxEvent::Node {
            kind: SurfaceNodeKind::PredicateHead,
            range: self.covering_token_range(position, cursor),
            children,
        });
        Some(ParsedTypeNode {
            id,
            next_position: cursor.max(position + 1),
            recovery_nodes,
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

    fn parse_template_arguments_at(&mut self, position: usize) -> ParsedTypeNode {
        let mut children = vec![self.token_node_ids[position]];
        let mut recovery_nodes = Vec::new();
        let mut cursor = position + 1;
        let mut expecting_argument = true;
        let mut saw_argument = false;

        while cursor < self.request.tokens.len() && !self.is_reserved_symbol_at(cursor, "]") {
            if self.is_reserved_symbol_at(cursor, ",") {
                if expecting_argument {
                    self.diagnose_malformed_type_expression(
                        cursor,
                        "expected template argument before `,`",
                    );
                    let missing = self.add_missing_type_expression(cursor);
                    let argument = self.events.emit(SyntaxEvent::Node {
                        kind: SurfaceNodeKind::TemplateArgument,
                        range: self.zero_range_at(cursor),
                        children: vec![missing],
                    });
                    children.push(argument);
                    recovery_nodes.push(missing);
                }
                children.push(self.token_node_ids[cursor]);
                cursor += 1;
                expecting_argument = true;
                continue;
            }

            let argument = self.parse_template_argument_at(cursor);
            let made_progress = argument.next_position > cursor;
            cursor = argument.next_position;
            children.push(argument.id);
            recovery_nodes.extend(argument.recovery_nodes);
            expecting_argument = false;
            saw_argument = true;
            if !made_progress {
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
                    if saw_argument {
                        "expected template argument before `]`"
                    } else {
                        "expected template argument between `[` and `]`"
                    },
                );
                let missing = self.add_missing_type_expression(cursor);
                let argument = self.events.emit(SyntaxEvent::Node {
                    kind: SurfaceNodeKind::TemplateArgument,
                    range: self.zero_range_at(cursor),
                    children: vec![missing],
                });
                children.push(argument);
                recovery_nodes.push(missing);
            }
            children.push(self.token_node_ids[cursor]);
            cursor += 1;
        } else {
            if expecting_argument {
                let missing = self.add_missing_type_expression(cursor);
                let argument = self.events.emit(SyntaxEvent::Node {
                    kind: SurfaceNodeKind::TemplateArgument,
                    range: self.zero_range_at(cursor),
                    children: vec![missing],
                });
                children.push(argument);
                recovery_nodes.push(missing);
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
            kind: SurfaceNodeKind::TemplateArguments,
            range: self.covering_token_range(position, cursor),
            children,
        });
        ParsedTypeNode {
            id,
            next_position: cursor.max(position + 1),
            recovery_nodes,
        }
    }

    fn parse_template_argument_at(&mut self, position: usize) -> ParsedTypeNode {
        let mut recovery_nodes = Vec::new();
        let payload = if self.template_argument_starts_term_application_at(position) {
            self.parse_term_expression_at(position)
                .expect("template term-application lookahead should parse")
        } else if let Some(type_expression) = self.parse_type_expression_at(position) {
            type_expression
        } else if let Some(term) = self.parse_bracket_qua_argument_at(position) {
            term
        } else if let Some(term) = self.parse_term_expression_at(position) {
            term
        } else {
            self.diagnose_malformed_type_expression(position, "expected template argument");
            let missing = self.add_missing_type_expression(position);
            recovery_nodes.push(missing);
            let id = self.events.emit(SyntaxEvent::Node {
                kind: SurfaceNodeKind::TemplateArgument,
                range: self.zero_range_at(position),
                children: vec![missing],
            });
            return ParsedTypeNode {
                id,
                next_position: position,
                recovery_nodes,
            };
        };
        let next_position = payload.next_position;
        recovery_nodes.extend(payload.recovery_nodes);
        let id = self.events.emit(SyntaxEvent::Node {
            kind: SurfaceNodeKind::TemplateArgument,
            range: self.covering_token_range(position, next_position),
            children: vec![payload.id],
        });
        ParsedTypeNode {
            id,
            next_position: next_position.max(position + 1),
            recovery_nodes,
        }
    }

    fn template_argument_starts_term_application_at(&self, position: usize) -> bool {
        let Some(mut cursor) = self
            .qualified_symbol_next_at(position)
            .or_else(|| self.is_identifier_at(position).then_some(position + 1))
        else {
            return false;
        };
        if self.is_reserved_symbol_at(cursor, "[") {
            let Some(arguments_end) = self.template_arguments_end_at(cursor) else {
                return false;
            };
            cursor = arguments_end;
        }
        self.is_reserved_symbol_at(cursor, "(")
    }

    fn parse_template_loci_at(&mut self, position: usize) -> ParsedTypeNode {
        let mut children = vec![self.token_node_ids[position]];
        let mut recovery_nodes = Vec::new();
        let mut cursor = position + 1;
        let mut expecting_locus = true;
        let mut saw_locus = false;

        while cursor < self.request.tokens.len() && !self.is_reserved_symbol_at(cursor, "]") {
            if self.is_reserved_symbol_at(cursor, ",") {
                if expecting_locus {
                    self.diagnose_malformed_term_expression(
                        cursor,
                        "expected template locus before `,`",
                    );
                    self.push_missing_template_locus(cursor, &mut children, &mut recovery_nodes);
                }
                children.push(self.token_node_ids[cursor]);
                cursor += 1;
                expecting_locus = true;
                continue;
            }

            let locus = self.parse_template_locus_at(cursor);
            let made_progress = locus.next_position > cursor;
            cursor = locus.next_position;
            children.push(locus.id);
            recovery_nodes.extend(locus.recovery_nodes);
            expecting_locus = false;
            saw_locus = true;
            if !made_progress {
                break;
            }

            if self.is_reserved_symbol_at(cursor, ",") {
                children.push(self.token_node_ids[cursor]);
                cursor += 1;
                expecting_locus = true;
            } else {
                break;
            }
        }

        if self.is_reserved_symbol_at(cursor, "]") {
            if expecting_locus {
                self.diagnose_malformed_term_expression(
                    cursor,
                    if saw_locus {
                        "expected template locus before `]`"
                    } else {
                        "expected template locus between `[` and `]`"
                    },
                );
                self.push_missing_template_locus(cursor, &mut children, &mut recovery_nodes);
            }
            children.push(self.token_node_ids[cursor]);
            cursor += 1;
        } else {
            if expecting_locus {
                self.push_missing_template_locus(cursor, &mut children, &mut recovery_nodes);
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
            kind: SurfaceNodeKind::TemplateLoci,
            range: self.covering_token_range(position, cursor),
            children,
        });
        ParsedTypeNode {
            id,
            next_position: cursor.max(position + 1),
            recovery_nodes,
        }
    }

    fn parse_template_locus_at(&mut self, position: usize) -> ParsedTypeNode {
        if self.is_identifier_at(position) {
            let id = self.events.emit(SyntaxEvent::Node {
                kind: SurfaceNodeKind::TemplateLocus,
                range: self.covering_token_range(position, position + 1),
                children: vec![self.token_node_ids[position]],
            });
            return ParsedTypeNode {
                id,
                next_position: position + 1,
                recovery_nodes: Vec::new(),
            };
        }

        self.diagnose_malformed_term_expression(position, "expected template locus");
        let missing = self.add_missing_term(position);
        let id = self.events.emit(SyntaxEvent::Node {
            kind: SurfaceNodeKind::TemplateLocus,
            range: self.zero_range_at(position),
            children: vec![missing],
        });
        ParsedTypeNode {
            id,
            next_position: position,
            recovery_nodes: vec![missing],
        }
    }

    fn push_missing_template_locus(
        &mut self,
        position: usize,
        children: &mut Vec<SurfaceBuilderNodeId>,
        recovery_nodes: &mut Vec<SurfaceBuilderNodeId>,
    ) {
        let missing = self.add_missing_term(position);
        let locus = self.events.emit(SyntaxEvent::Node {
            kind: SurfaceNodeKind::TemplateLocus,
            range: self.zero_range_at(position),
            children: vec![missing],
        });
        children.push(locus);
        recovery_nodes.push(missing);
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
        let reference = self.parse_identifier_reference_term_at(position);
        if self.is_reserved_symbol_at(reference.next_position, "(") {
            self.parse_application_term_after_callee(
                position,
                reference.id,
                reference.next_position,
            )
        } else {
            reference
        }
    }

    fn parse_identifier_reference_term_at(&mut self, position: usize) -> ParsedTypeNode {
        let mut children = vec![self.token_node_ids[position]];
        let mut recovery_nodes = Vec::new();
        let mut cursor = position + 1;
        if self.is_reserved_symbol_at(cursor, "[") {
            let arguments = self.parse_template_arguments_at(cursor);
            cursor = arguments.next_position;
            children.push(arguments.id);
            recovery_nodes.extend(arguments.recovery_nodes);
        }
        let reference = self.events.emit(SyntaxEvent::Node {
            kind: SurfaceNodeKind::TermReference,
            range: self.covering_token_range(position, cursor),
            children,
        });
        ParsedTypeNode {
            id: reference,
            next_position: cursor.max(position + 1),
            recovery_nodes,
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
        let mut children = vec![symbol.id];
        let mut recovery_nodes = Vec::new();
        let mut cursor = symbol.next_position;
        if self.is_reserved_symbol_at(cursor, "[") {
            let arguments = self.parse_template_arguments_at(cursor);
            cursor = arguments.next_position;
            children.push(arguments.id);
            recovery_nodes.extend(arguments.recovery_nodes);
        }
        let reference = self.events.emit(SyntaxEvent::Node {
            kind: SurfaceNodeKind::TermReference,
            range: self.covering_token_range(position, cursor),
            children,
        });
        Some(ParsedTypeNode {
            id: reference,
            next_position: cursor.max(position + 1),
            recovery_nodes,
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
            || self.formula_payload_contains_theorem_tail_at(formula_start)
        {
            return None;
        }

        let prefix = self.parse_leading_annotations_at(position);
        let mut children = prefix.children;
        let mut recovery_nodes = prefix.recovery_nodes;
        children.extend(
            self.token_node_ids[prefix.next_position..=colon]
                .iter()
                .copied(),
        );
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

    fn diagnose_malformed_annotation(
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
            SyntaxDiagnostic::new(SyntaxDiagnosticCode::MalformedAnnotation, message, primary)
                .with_recovery_note("repair the annotation syntax before continuing"),
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

    fn diagnose_unmatched_annotation_delimiter(
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
                SyntaxDiagnosticCode::MalformedAnnotation,
                format!("expected `{close_symbol}` to close annotation delimiter"),
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

    fn add_missing_annotation_argument(&mut self, position: usize) -> SurfaceBuilderNodeId {
        self.add_recovery_node(
            SyntaxRecoveryKind::MissingAnnotationArgument,
            self.zero_range_at(position),
            Vec::new(),
        )
    }

    fn add_malformed_annotation(&mut self, position: usize) -> SurfaceBuilderNodeId {
        self.add_recovery_node(
            SyntaxRecoveryKind::MalformedAnnotation,
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

    fn push_missing_annotation_argument(
        &mut self,
        position: usize,
        children: &mut Vec<SurfaceBuilderNodeId>,
        recovery_nodes: &mut Vec<SurfaceBuilderNodeId>,
    ) {
        let recovery = self.add_missing_annotation_argument(position);
        children.push(recovery);
        recovery_nodes.push(recovery);
    }

    fn push_malformed_annotation(
        &mut self,
        position: usize,
        children: &mut Vec<SurfaceBuilderNodeId>,
        recovery_nodes: &mut Vec<SurfaceBuilderNodeId>,
    ) {
        let recovery = self.add_malformed_annotation(position);
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

    fn recover_malformed_algorithm_header_tail(&mut self, position: usize) -> Option<ParsedItem> {
        let mut cursor = position;
        let mut paren_depth = 0_usize;
        let mut bracket_depth = 0_usize;
        let mut brace_depth = 0_usize;

        while cursor < self.request.tokens.len() {
            let top_level = paren_depth == 0 && bracket_depth == 0 && brace_depth == 0;
            if top_level
                && (self.is_reserved_word_at(cursor, "do")
                    || self.is_semicolon_at(cursor)
                    || self.is_end_keyword_at(cursor)
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

    fn recover_malformed_algorithm_statement_tail(
        &mut self,
        position: usize,
        boundary: AlgorithmStatementListBoundary,
    ) -> Option<ParsedItem> {
        let mut cursor = position;
        let mut paren_depth = 0_usize;
        let mut bracket_depth = 0_usize;
        let mut brace_depth = 0_usize;

        while cursor < self.request.tokens.len() {
            let top_level = paren_depth == 0 && bracket_depth == 0 && brace_depth == 0;
            if top_level
                && (self.is_algorithm_statement_list_boundary_for_at(cursor, boundary)
                    || self.is_semicolon_at(cursor)
                    || (cursor > position && self.is_algorithm_statement_start_at(cursor)))
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

    fn recover_malformed_algorithm_loop_annotation_tail(
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
                && (self.is_semicolon_at(cursor)
                    || self.is_algorithm_statement_list_boundary_for_at(
                        cursor,
                        AlgorithmStatementListBoundary::NestedBlock,
                    )
                    || (cursor > position && self.is_algorithm_statement_start_at(cursor)))
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

    fn recover_malformed_algorithm_term_list_tail(
        &mut self,
        position: usize,
        boundary: AlgorithmTermListBoundary,
    ) -> Option<ParsedItem> {
        let mut cursor = position;
        let mut paren_depth = 0_usize;
        let mut bracket_depth = 0_usize;
        let mut brace_depth = 0_usize;

        while cursor < self.request.tokens.len() {
            let top_level = paren_depth == 0 && bracket_depth == 0 && brace_depth == 0;
            if top_level
                && (self.is_reserved_symbol_at(cursor, ",")
                    || self.is_algorithm_term_list_boundary_at(cursor, boundary))
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

    fn recover_malformed_algorithm_misplaced_item_tail(
        &mut self,
        position: usize,
    ) -> Option<ParsedItem> {
        let head = self.item_head_position(position).unwrap_or(position);
        let cursor = if self
            .request
            .tokens
            .get(head)
            .is_some_and(is_block_like_top_level_start)
        {
            self.block_like_tail_semicolon_position(head)
        } else {
            let mut cursor = position;
            while cursor < self.request.tokens.len() && !self.is_semicolon_at(cursor) {
                cursor += 1;
            }
            cursor
        };
        self.emit_malformed_tail_recovery(position, cursor)
    }

    fn recover_malformed_claim_content_tail(&mut self, position: usize) -> Option<ParsedItem> {
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
                    || (cursor > position && self.theorem_role_position_at(cursor).is_some()))
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
            } else if self.is_reserved_symbol_at(cursor, "@[")
                || self.is_reserved_symbol_at(cursor, "[")
            {
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

    fn recover_malformed_registration_content_tail(
        &mut self,
        position: usize,
    ) -> Option<ParsedItem> {
        let mut cursor = position;
        let mut paren_depth = 0_usize;
        let mut bracket_depth = 0_usize;
        let mut brace_depth = 0_usize;

        while cursor < self.request.tokens.len() {
            let top_level = paren_depth == 0 && bracket_depth == 0 && brace_depth == 0;
            if top_level && self.is_registration_content_synchronization_boundary_at(cursor) {
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
        if self.is_standalone_diagnostic_annotation_start_at(position) {
            return true;
        }
        let head = self.skip_annotations(position);
        if head > position {
            return self.is_statement_core_start_at(head);
        }
        self.is_statement_core_start_at(position)
    }

    fn is_statement_core_start_at(&self, position: usize) -> bool {
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

    fn top_level_reserved_word_before_statement_boundary_at(
        &self,
        position: usize,
        word: &str,
    ) -> Option<usize> {
        let mut cursor = position;
        let mut paren_depth = 0_usize;
        let mut bracket_depth = 0_usize;
        let mut brace_depth = 0_usize;

        while cursor < self.request.tokens.len() {
            let top_level = paren_depth == 0 && bracket_depth == 0 && brace_depth == 0;
            if top_level {
                if self.is_reserved_word_at(cursor, word) {
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

    fn is_string_literal_at(&self, position: usize) -> bool {
        self.request
            .tokens
            .get(position)
            .is_some_and(|token| token.kind == ParserTokenKind::StringLiteral)
    }

    fn annotation_marker_text_at(&self, position: usize) -> Option<&str> {
        self.request.tokens.get(position).and_then(|token| {
            (token.kind == ParserTokenKind::AnnotationMarker).then_some(token.text.as_ref())
        })
    }

    fn is_standalone_diagnostic_annotation_start_at(&self, position: usize) -> bool {
        self.annotation_marker_text_at(position)
            .is_some_and(is_standalone_diagnostic_annotation_marker)
    }

    fn is_annotation_argument_token_at(&self, position: usize) -> bool {
        self.is_identifier_at(position)
            || self.is_numeral_at(position)
            || self.is_string_literal_at(position)
    }

    fn is_annotation_argument_list_boundary_at(
        &self,
        position: usize,
        boundary: AnnotationDelimitedListBoundary,
    ) -> bool {
        position >= self.request.tokens.len()
            || self.is_reserved_symbol_at(position, ")")
            || self.is_reserved_symbol_at(position, "]")
            || self.is_semicolon_at(position)
            || self.is_end_keyword_at(position)
            || self.is_annotation_recovery_host_boundary_at(position, boundary)
    }

    fn is_annotation_recovery_host_boundary_at(
        &self,
        position: usize,
        boundary: AnnotationDelimitedListBoundary,
    ) -> bool {
        self.is_top_level_keyword_at(position)
            || self.is_definition_content_core_start_at(position)
            || self.is_registration_content_core_start_at(position)
            || self.is_algorithm_statement_core_start_at(position)
            || self.is_statement_core_start_at(position)
            || (boundary == AnnotationDelimitedListBoundary::Argument
                && self.is_identifier_statement_recovery_boundary_at(position))
    }

    fn is_identifier_statement_recovery_boundary_at(&self, position: usize) -> bool {
        if !self.is_identifier_at(position) {
            return false;
        }
        if self.is_reserved_symbol_at(position + 1, ":") {
            return true;
        }

        let mut cursor = position + 1;
        while cursor < self.request.tokens.len() {
            if self.is_reserved_symbol_at(cursor, ":")
                || (self.is_identifier_at(cursor) && self.is_reserved_symbol_at(cursor + 1, ":"))
            {
                return false;
            }
            if self.is_reserved_word_at(cursor, "by") || self.is_reserved_word_at(cursor, "proof") {
                return true;
            }
            if self.is_semicolon_at(cursor)
                || self.is_end_keyword_at(cursor)
                || self.is_reserved_symbol_at(cursor, ",")
                || self.is_reserved_symbol_at(cursor, ")")
                || self.is_reserved_symbol_at(cursor, "]")
                || self.is_reserved_symbol_at(cursor, "}")
            {
                return false;
            }
            cursor += 1;
        }
        false
    }

    fn is_annotation_label_list_recovery_boundary_at(&self, position: usize) -> bool {
        position >= self.request.tokens.len()
            || self.is_semicolon_at(position)
            || self.is_end_keyword_at(position)
            || self.is_annotation_recovery_host_boundary_at(
                position,
                AnnotationDelimitedListBoundary::Argument,
            )
    }

    fn annotation_argument_requirement_matches(
        &self,
        position: usize,
        requirement: AnnotationArgumentRequirement,
    ) -> bool {
        match requirement {
            AnnotationArgumentRequirement::Identifier => self.is_identifier_at(position),
            AnnotationArgumentRequirement::StringLiteral => self.is_string_literal_at(position),
        }
    }

    fn is_solver_name_at(&self, position: usize) -> bool {
        self.request.tokens.get(position).is_some_and(|token| {
            matches!(
                token.kind,
                ParserTokenKind::Identifier | ParserTokenKind::ReservedWord
            ) && matches!(
                token.text.as_ref(),
                "vampire" | "e" | "cvc5" | "z3" | "auto"
            )
        })
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

    fn definition_block_is_template_shaped_at(&self, position: usize) -> bool {
        let mut cursor = position;
        let mut saw_leading_parameter = false;

        while self.is_reserved_word_at(cursor, "let") {
            saw_leading_parameter = true;
            if self.definition_parameter_starts_template_ambiguous_at(cursor) {
                return true;
            }
            let next = self.definition_parameter_end_at(cursor);
            if next <= cursor {
                break;
            }
            cursor = next;
        }

        saw_leading_parameter && self.definition_content_is_template_only_at(cursor)
    }

    fn definition_parameter_end_at(&self, position: usize) -> usize {
        let mut cursor = position;
        let mut paren_depth = 0_usize;
        let mut bracket_depth = 0_usize;
        let mut brace_depth = 0_usize;

        while cursor < self.request.tokens.len() {
            let top_level = paren_depth == 0 && bracket_depth == 0 && brace_depth == 0;
            if top_level && self.is_semicolon_at(cursor) {
                return cursor + 1;
            }
            if top_level && self.is_end_keyword_at(cursor) {
                return cursor;
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

        cursor
    }

    fn definition_content_is_template_only_at(&self, position: usize) -> bool {
        if self.theorem_role_position_at(position).is_some()
            || self.is_registration_item_start_at(position)
        {
            return true;
        }
        if self.is_visibility_marker_at(position) {
            return self.theorem_role_position_at(position + 1).is_some()
                || self.is_registration_item_start_at(position + 1)
                || self.definition_definition_has_template_loci_at(position + 1);
        }
        self.definition_definition_has_template_loci_at(position)
    }

    fn definition_definition_has_template_loci_at(&self, position: usize) -> bool {
        if !self.is_reserved_word_at(position, "pred")
            && !self.is_reserved_word_at(position, "func")
        {
            return false;
        }

        let mut cursor = position;
        let mut paren_depth = 0_usize;
        while cursor < self.request.tokens.len() && !self.is_semicolon_at(cursor) {
            if self.is_reserved_word_at(position, "pred")
                && self.is_reserved_word_at(cursor, "means")
                && paren_depth == 0
            {
                return false;
            }
            if self.is_reserved_word_at(position, "func")
                && self.is_reserved_symbol_at(cursor, "->")
                && paren_depth == 0
            {
                return false;
            }
            if self.is_reserved_symbol_at(cursor, "[") && paren_depth == 0 {
                return true;
            }
            if self.is_reserved_symbol_at(cursor, "(") {
                paren_depth += 1;
            } else if self.is_reserved_symbol_at(cursor, ")") {
                if paren_depth == 0 {
                    break;
                }
                paren_depth -= 1;
            }
            cursor += 1;
        }
        false
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

    fn is_registration_content_synchronization_boundary_at(&self, position: usize) -> bool {
        position >= self.request.tokens.len()
            || self.is_semicolon_at(position)
            || self.is_end_keyword_at(position)
            || self.is_registration_content_start_at(position)
            || self.is_item_start_at(position)
    }

    fn is_registration_content_start_at(&self, position: usize) -> bool {
        if self.is_standalone_diagnostic_annotation_start_at(position) {
            return true;
        }
        let head = self.skip_annotations(position);
        if head > position {
            return self.is_registration_content_core_start_at(head);
        }
        self.is_registration_content_core_start_at(position)
    }

    fn is_registration_content_core_start_at(&self, position: usize) -> bool {
        self.is_reserved_word_at(position, "let") || self.is_registration_item_start_at(position)
    }

    fn is_registration_item_start_at(&self, position: usize) -> bool {
        self.is_reserved_word_at(position, "cluster")
            || self.is_reserved_word_at(position, "reduce")
    }

    fn is_algorithm_definition_start_at(&self, position: usize) -> bool {
        let algorithm = if self.is_reserved_word_at(position, "terminating") {
            position + 1
        } else {
            position
        };

        if !self.is_reserved_word_at(algorithm, "algorithm") {
            return false;
        }

        let name = algorithm + 1;
        if !self.is_identifier_at(name) {
            return false;
        }

        let mut cursor = name + 1;
        if self.is_reserved_symbol_at(cursor, "(")
            || self.is_reserved_word_at(cursor, "do")
            || self.is_reserved_symbol_at(cursor, "->")
            || self.is_algorithm_header_clause_start_at(cursor)
        {
            return true;
        }

        if !self.is_reserved_symbol_at(cursor, "[") {
            return false;
        }

        let mut depth = 0_usize;
        while cursor < self.request.tokens.len() {
            if self.is_reserved_symbol_at(cursor, "[") {
                depth += 1;
            } else if self.is_reserved_symbol_at(cursor, "]") {
                depth -= 1;
                if depth == 0 {
                    cursor += 1;
                    return self.is_reserved_symbol_at(cursor, "(")
                        || self.is_reserved_word_at(cursor, "do")
                        || self.is_reserved_symbol_at(cursor, "->")
                        || self.is_algorithm_header_clause_start_at(cursor);
                }
            } else if self.is_semicolon_at(cursor) || self.is_end_keyword_at(cursor) {
                return false;
            }
            cursor += 1;
        }

        true
    }

    fn is_algorithm_header_clause_start_at(&self, position: usize) -> bool {
        self.request.tokens.get(position).is_some_and(|token| {
            token.kind == ParserTokenKind::ReservedWord
                && matches!(token.text.as_ref(), "requires" | "ensures" | "decreasing")
        })
    }

    fn is_algorithm_header_verification_keyword_at(&self, position: usize) -> bool {
        self.request.tokens.get(position).is_some_and(|token| {
            token.kind == ParserTokenKind::ReservedWord
                && matches!(
                    token.text.as_ref(),
                    "terminating" | "requires" | "ensures" | "decreasing"
                )
        })
    }

    fn is_algorithm_statement_start_at(&self, position: usize) -> bool {
        if self.is_standalone_diagnostic_annotation_start_at(position) {
            return true;
        }
        let head = self.skip_annotations(position);
        if head > position {
            return self.is_algorithm_statement_core_start_at(head);
        }
        self.is_algorithm_statement_core_start_at(position)
    }

    fn is_algorithm_statement_core_start_at(&self, position: usize) -> bool {
        self.is_reserved_word_at(position, "if")
            || self.is_reserved_word_at(position, "while")
            || self.is_reserved_word_at(position, "for")
            || self.is_reserved_word_at(position, "match")
            || self.is_reserved_word_at(position, "break")
            || self.is_reserved_word_at(position, "continue")
            || self.is_reserved_word_at(position, "assert")
            || self.is_reserved_word_at(position, "var")
            || self.is_reserved_word_at(position, "const")
            || self.is_reserved_word_at(position, "snapshot")
            || self.is_reserved_word_at(position, "return")
            || self.is_reserved_word_at(position, "ghost")
            || (self.is_lvalue_start_at(position)
                && self.lvalue_assignment_operator_at(position).is_some())
    }

    fn is_algorithm_statement_list_boundary_at(&self, position: usize) -> bool {
        self.is_algorithm_statement_list_boundary_for_at(
            position,
            AlgorithmStatementListBoundary::Body,
        )
    }

    fn is_algorithm_statement_list_boundary_for_at(
        &self,
        position: usize,
        boundary: AlgorithmStatementListBoundary,
    ) -> bool {
        position >= self.request.tokens.len()
            || self.is_end_keyword_at(position)
            || self.is_item_start_at(position)
            || self.is_algorithm_statement_list_context_boundary_at(position, boundary)
    }

    fn is_algorithm_statement_list_context_boundary_at(
        &self,
        position: usize,
        boundary: AlgorithmStatementListBoundary,
    ) -> bool {
        match boundary {
            AlgorithmStatementListBoundary::Body => false,
            AlgorithmStatementListBoundary::NestedBlock => {
                self.is_algorithm_nested_control_boundary_at(position)
            }
            AlgorithmStatementListBoundary::IfThen => {
                self.is_algorithm_nested_control_boundary_at(position)
            }
            AlgorithmStatementListBoundary::MatchCase => {
                self.is_reserved_word_at(position, "case")
                    || self.is_reserved_word_at(position, "otherwise")
                    || self.is_reserved_word_at(position, "exhaustive")
            }
        }
    }

    fn is_algorithm_nested_control_boundary_at(&self, position: usize) -> bool {
        self.is_reserved_word_at(position, "else")
            || self.is_reserved_word_at(position, "case")
            || self.is_reserved_word_at(position, "otherwise")
            || self.is_reserved_word_at(position, "exhaustive")
    }

    fn is_algorithm_loop_task34_clause_at(&self, position: usize) -> bool {
        self.is_reserved_word_at(position, "invariant")
            || self.is_reserved_word_at(position, "decreasing")
    }

    fn is_algorithm_term_list_boundary_at(
        &self,
        position: usize,
        boundary: AlgorithmTermListBoundary,
    ) -> bool {
        if position >= self.request.tokens.len()
            || self.is_semicolon_at(position)
            || self.is_item_start_at(position)
            || self.is_end_keyword_at(position)
        {
            return true;
        }

        match boundary {
            AlgorithmTermListBoundary::HeaderClause => {
                self.is_reserved_word_at(position, "do")
                    || self.is_algorithm_header_verification_keyword_at(position)
            }
            AlgorithmTermListBoundary::ClauseStatement => {
                self.is_reserved_word_at(position, "by")
                    || self.is_reserved_word_at(position, "proof")
                    || self.is_algorithm_statement_list_context_boundary_at(
                        position,
                        AlgorithmStatementListBoundary::NestedBlock,
                    )
                    || self.is_algorithm_statement_start_at(position)
            }
        }
    }

    fn is_algorithm_declaration_tail_boundary_at(&self, position: usize) -> bool {
        self.is_semicolon_at(position)
            || self.is_reserved_word_at(position, "as")
            || self.is_algorithm_statement_list_boundary_at(position)
    }

    fn is_lvalue_start_at(&self, position: usize) -> bool {
        self.is_identifier_at(position)
    }

    fn lvalue_assignment_operator_at(&self, position: usize) -> Option<usize> {
        let mut cursor = position;
        if !self.is_identifier_at(cursor) {
            return None;
        }
        cursor += 1;
        while self.is_reserved_symbol_at(cursor, ".") && self.is_identifier_at(cursor + 1) {
            cursor += 2;
        }
        self.is_reserved_symbol_at(cursor, ":=").then_some(cursor)
    }

    fn is_definition_content_start_at(&self, position: usize) -> bool {
        if self.is_standalone_diagnostic_annotation_start_at(position) {
            return true;
        }
        let head = self.skip_annotations(position);
        if head > position {
            return self.is_definition_content_core_start_at(head);
        }
        self.is_definition_content_core_start_at(position)
    }

    fn is_definition_content_core_start_at(&self, position: usize) -> bool {
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
            || self.is_reserved_word_at(position, "algorithm")
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
                && self.is_reserved_word_at(position + 1, "algorithm"))
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
                        | "algorithm"
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

    fn registration_content_placeholder_end(&self, position: usize) -> usize {
        let mut cursor = position;
        let mut block_depth = 0_usize;

        while cursor < self.request.tokens.len() {
            let top_level = block_depth == 0;
            if top_level
                && cursor > position
                && (self.is_end_keyword_at(cursor) || self.is_registration_content_start_at(cursor))
            {
                break;
            }
            if top_level
                && cursor > position
                && self.is_registration_block_outer_content_boundary_at(cursor)
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

            if sync::opens_recovery_block_at(&self.request.tokens, cursor) {
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

    fn registration_header_boundary_at(&self, position: usize) -> usize {
        let mut cursor = position;
        let mut paren_depth = 0_usize;
        let mut bracket_depth = 0_usize;
        let mut brace_depth = 0_usize;

        while cursor < self.request.tokens.len() {
            let top_level = paren_depth == 0 && bracket_depth == 0 && brace_depth == 0;
            if top_level
                && (self.is_semicolon_at(cursor)
                    || self.is_end_keyword_at(cursor)
                    || self.is_registration_content_start_at(cursor)
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

    fn registration_adjective_ref_plan_at(&self, position: usize) -> Option<AttributeRefPlan> {
        let mut cursor = position;
        if self.is_reserved_word_at(cursor, "non") {
            cursor += 1;
        }
        if let Some(prefix_end) = self.parameter_prefix_next_at(cursor) {
            cursor = prefix_end;
        }
        cursor = self.attribute_symbol_next_at(cursor)?;
        if self.is_reserved_symbol_at(cursor, "(") {
            return None;
        }
        Some(AttributeRefPlan {
            start_position: position,
            next_position: cursor,
        })
    }

    fn first_argument_bearing_registration_adjective_before(
        &self,
        position: usize,
        end: usize,
    ) -> Option<usize> {
        let mut cursor = position;
        let mut paren_depth = 0_usize;
        let mut bracket_depth = 0_usize;
        let mut brace_depth = 0_usize;

        while cursor < end {
            let top_level = paren_depth == 0 && bracket_depth == 0 && brace_depth == 0;
            if top_level {
                if let Some(argument_position) =
                    self.argument_bearing_registration_adjective_at(cursor)
                    && argument_position < end
                {
                    return Some(argument_position);
                }
                if let Some(plan) = self.registration_adjective_ref_plan_at(cursor)
                    && plan.next_position <= end
                {
                    cursor = plan.next_position;
                    continue;
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

    fn argument_bearing_registration_adjective_at(&self, position: usize) -> Option<usize> {
        let mut cursor = position;
        if self.is_reserved_word_at(cursor, "non") {
            cursor += 1;
        }
        if let Some(prefix_end) = self.parameter_prefix_next_at(cursor) {
            cursor = prefix_end;
        }
        cursor = self.attribute_symbol_next_at(cursor)?;
        self.is_reserved_symbol_at(cursor, "(").then_some(cursor)
    }

    fn registration_adjective_list_can_match(&self, position: usize, end: usize) -> bool {
        let mut cursor = position;
        let mut saw_adjective = false;
        while cursor < end {
            let Some(plan) = self.registration_adjective_ref_plan_at(cursor) else {
                return false;
            };
            if plan.next_position > end {
                return false;
            }
            cursor = plan.next_position;
            saw_adjective = true;
        }
        saw_adjective && cursor == end
    }

    fn registration_functorial_payload_can_match(&self, position: usize, end: usize) -> bool {
        if position >= end {
            return false;
        }
        if self.payload_contains_top_level_word(position, end, "qua")
            || self.payload_contains_top_level_word(position, end, "with")
        {
            return false;
        }
        if self.is_identifier_at(position)
            && self.is_reserved_symbol_at(position + 1, ".")
            && self.qualified_symbol_next_at(position).is_none()
        {
            return false;
        }
        if self.prefix_operator_at(position).is_some()
            || self.payload_contains_top_level_term_operator(position, end)
            || self.is_reserved_symbol_at(position, "[")
        {
            return true;
        }
        if let Some(symbol_end) = self.qualified_symbol_next_at(position) {
            if self.can_parse_structure_constructor_after_symbol(symbol_end) {
                return false;
            }
            return symbol_end < end && self.is_reserved_symbol_at(symbol_end, "(");
        }
        if self.is_identifier_at(position) {
            return position + 1 < end && self.is_reserved_symbol_at(position + 1, "(");
        }
        false
    }

    fn payload_contains_top_level_word(&self, position: usize, end: usize, word: &str) -> bool {
        let mut cursor = position;
        let mut paren_depth = 0_usize;
        let mut bracket_depth = 0_usize;
        let mut brace_depth = 0_usize;

        while cursor < end {
            let top_level = paren_depth == 0 && bracket_depth == 0 && brace_depth == 0;
            if top_level && self.is_reserved_word_at(cursor, word) {
                return true;
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

    fn payload_contains_top_level_term_operator(&self, position: usize, end: usize) -> bool {
        let mut cursor = position + 1;
        let mut paren_depth = 0_usize;
        let mut bracket_depth = 0_usize;
        let mut brace_depth = 0_usize;

        while cursor < end {
            let top_level = paren_depth == 0 && bracket_depth == 0 && brace_depth == 0;
            if top_level
                && (self.infix_operator_at(cursor).is_some()
                    || self.postfix_operator_at(cursor).is_some())
            {
                return true;
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
                        || self.predicate_head_next_at(cursor).is_some()
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

    fn should_try_head_first_predicate_at(&self, position: usize) -> bool {
        if self.predicate_negation_end_at(position).is_some() {
            return true;
        }
        let Some(head_end) = self.predicate_head_next_at(position) else {
            return false;
        };
        let mut after_head = head_end;
        if self.is_reserved_symbol_at(head_end, "[") {
            let Some(arguments_end) = self.template_arguments_end_at(head_end) else {
                return true;
            };
            if self.is_reserved_symbol_at(arguments_end, "(") {
                return false;
            }
            after_head = arguments_end;
        }
        !self.is_builtin_predicate_at(head_end)
            && !self.is_reserved_word_at(head_end, "is")
            && !self.is_builtin_predicate_at(after_head)
            && !self.is_reserved_word_at(after_head, "is")
            && !self.is_reserved_symbol_at(head_end, "(")
    }

    fn template_arguments_end_at(&self, position: usize) -> Option<usize> {
        if !self.is_reserved_symbol_at(position, "[") {
            return None;
        }
        let mut cursor = position + 1;
        let mut bracket_depth = 1_usize;
        let mut paren_depth = 0_usize;
        let mut brace_depth = 0_usize;
        while cursor < self.request.tokens.len() {
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
                bracket_depth -= 1;
                if bracket_depth == 0 {
                    return Some(cursor + 1);
                }
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
            || self.predicate_head_next_at(position).is_some()
    }

    fn predicate_head_next_at(&self, position: usize) -> Option<usize> {
        self.qualified_symbol_next_at(position).or_else(|| {
            (self.is_identifier_at(position) && self.is_reserved_symbol_at(position + 1, "["))
                .then_some(position + 1)
        })
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
        while let Some(next) = self.after_annotation(position) {
            position = next;
        }
        position
    }

    fn after_annotation(&self, position: usize) -> Option<usize> {
        if self.is_reserved_symbol_at(position, "@[") {
            return self
                .after_balanced_annotation_delimiter(
                    position,
                    "@[",
                    "]",
                    AnnotationDelimitedListBoundary::Argument,
                )
                .or_else(|| {
                    self.after_malformed_annotation_delimiter(
                        position,
                        AnnotationDelimitedListBoundary::Argument,
                    )
                });
        }

        let marker = self.annotation_marker_text_at(position)?;
        if is_standalone_diagnostic_annotation_marker(marker) {
            return None;
        }

        let cursor = position + 1;
        if self.is_reserved_symbol_at(cursor, "(") {
            let boundary = if marker == "@proof_hint" {
                AnnotationDelimitedListBoundary::ProofHintOption
            } else {
                AnnotationDelimitedListBoundary::Argument
            };
            self.after_balanced_annotation_delimiter(cursor, "(", ")", boundary)
                .or_else(|| self.after_malformed_annotation_delimiter(cursor, boundary))
        } else {
            Some(cursor)
        }
    }

    fn after_malformed_annotation_delimiter(
        &self,
        position: usize,
        boundary: AnnotationDelimitedListBoundary,
    ) -> Option<usize> {
        let mut cursor = position + 1;
        while cursor < self.request.tokens.len() {
            if self.is_annotation_argument_list_boundary_at(cursor, boundary) {
                return Some(cursor);
            }
            cursor += 1;
        }
        None
    }

    fn after_balanced_annotation_delimiter(
        &self,
        position: usize,
        open: &str,
        close: &str,
        boundary: AnnotationDelimitedListBoundary,
    ) -> Option<usize> {
        let mut cursor = position + 1;
        let mut depth = 1_usize;
        while cursor < self.request.tokens.len() {
            if depth == 1 && self.is_annotation_recovery_host_boundary_at(cursor, boundary) {
                return None;
            }
            if self.is_reserved_symbol_at(cursor, open) {
                depth += 1;
            } else if self.is_reserved_symbol_at(cursor, close) {
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

fn is_standalone_diagnostic_annotation_marker(marker: &str) -> bool {
    matches!(marker, "@show_type" | "@eval")
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
mod tests;
