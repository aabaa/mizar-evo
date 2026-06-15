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
    SkippedTokenReason, SurfaceBuilderNodeId, SurfaceInfixOperator, SurfaceNodeKind,
    SurfaceOperatorAssociativity, SurfacePostfixOperator, SurfacePrefixOperator, SyntaxDiagnostic,
    SyntaxDiagnosticCode, SyntaxRecoveryKind,
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
            let target = self.parse_placeholder_item(target_position);
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
            return Some(self.parse_set_enumeration_at(position));
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
            if cursor > position && !is_same_prefixed_item && self.is_item_start_at(cursor) {
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
                "expected `;` before the next top-level item boundary",
            )
            .with_recovery_note("insert `;` before continuing with the next item"),
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

    fn add_missing_type_expression(&mut self, position: usize) -> SurfaceBuilderNodeId {
        self.add_recovery_node(
            SyntaxRecoveryKind::MissingTypeExpression,
            self.zero_range_at(position),
            Vec::new(),
        )
    }

    fn add_missing_term(&mut self, position: usize) -> SurfaceBuilderNodeId {
        self.add_recovery_node(
            SyntaxRecoveryKind::MissingTerm,
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

    fn recover_malformed_tail(&mut self, position: usize) -> Option<ParsedItem> {
        let mut cursor = position;
        while cursor < self.request.tokens.len() && !self.is_semicolon_at(cursor) {
            cursor += 1;
        }
        self.emit_malformed_tail_recovery(position, cursor)
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
            if self.is_item_start_at(position) {
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

    fn is_visibility_marker_at(&self, position: usize) -> bool {
        self.request.tokens.get(position).is_some_and(|token| {
            token.kind == ParserTokenKind::ReservedWord
                && matches!(token.text.as_ref(), "private" | "public")
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

    fn is_identifier_at(&self, position: usize) -> bool {
        self.request
            .tokens
            .get(position)
            .is_some_and(|token| token.kind == ParserTokenKind::Identifier)
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
            || self.is_reserved_symbol_at(position, ",")
            || self.is_reserved_symbol_at(position, ")")
            || self.is_reserved_symbol_at(position, "]")
            || self.is_reserved_symbol_at(position, "}")
            || self.is_item_start_at(position)
    }

    fn is_term_expression_boundary_at(&self, position: usize) -> bool {
        position >= self.request.tokens.len()
            || self.is_semicolon_at(position)
            || self.is_reserved_symbol_at(position, ",")
            || self.is_reserved_symbol_at(position, ")")
            || self.is_reserved_symbol_at(position, "]")
            || self.is_reserved_symbol_at(position, "}")
            || self.is_item_start_at(position)
    }

    fn is_term_placeholder_boundary_at(&self, position: usize) -> bool {
        self.is_semicolon_at(position)
            || self.is_reserved_symbol_at(position, ",")
            || self.is_reserved_symbol_at(position, ")")
            || self.is_reserved_symbol_at(position, "]")
            || self.is_reserved_symbol_at(position, "}")
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
        SkippedTokenReason, SurfaceNodeKind, SurfaceOperatorAssociativity, SyntaxDiagnosticCode,
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
                token(source_id, ParserTokenKind::Identifier, "thesis", 11, 17),
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
        assert!(items.iter().all(|id| {
            matches!(
                ast.node(*id).unwrap().kind,
                SurfaceNodeKind::PlaceholderItem
            )
        }));
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
                    (";", ParserTokenKind::ReservedSymbol),
                ],
            ),
            (
                "antonym",
                &[
                    ("antonym", ParserTokenKind::ReservedWord),
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

        for item_id in &item_list.children[2..] {
            let visible = ast.node(*item_id).unwrap();
            assert!(matches!(visible.kind, SurfaceNodeKind::VisibleItem));
            assert_eq!(visible.children.len(), 2);
            assert!(matches!(
                ast.node(visible.children[0]).unwrap().kind,
                SurfaceNodeKind::VisibilityMarker
            ));
            assert!(matches!(
                ast.node(visible.children[1]).unwrap().kind,
                SurfaceNodeKind::PlaceholderItem
            ));
        }
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
    fn proof_block_semicolons_stay_inside_the_theorem_placeholder() {
        let source_id = source_id(68);
        let tokens = token_sequence(
            source_id,
            &[
                ("theorem", ParserTokenKind::ReservedWord),
                ("T", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("thesis", ParserTokenKind::Identifier),
                ("proof", ParserTokenKind::ReservedWord),
                ("thus", ParserTokenKind::ReservedWord),
                ("thesis", ParserTokenKind::Identifier),
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
                ("theorem", ParserTokenKind::ReservedWord),
                ("T", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
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

    fn single_node(
        ast: &mizar_syntax::SurfaceAst,
        predicate: impl Fn(&SurfaceNodeKind) -> bool,
    ) -> &mizar_syntax::SurfaceNode {
        ast.nodes()
            .iter()
            .find(|node| predicate(&node.kind))
            .expect("expected exactly one matching node")
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
