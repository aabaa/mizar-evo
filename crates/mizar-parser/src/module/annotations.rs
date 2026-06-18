use super::*;

impl Parser {
    pub(super) fn parse_leading_annotations_at(
        &mut self,
        position: usize,
    ) -> ParsedAnnotationPrefix {
        let mut children = Vec::new();
        let mut recovery_nodes = Vec::new();
        let mut cursor = position;

        while let Some(annotation) = self.parse_annotation_at(cursor) {
            let made_progress = annotation.next_position > cursor;
            cursor = annotation.next_position;
            children.push(annotation.id);
            recovery_nodes.extend(annotation.recovery_nodes);
            if !made_progress {
                break;
            }
        }

        ParsedAnnotationPrefix {
            children,
            next_position: cursor,
            recovery_nodes,
        }
    }

    pub(super) fn parse_annotation_at(&mut self, position: usize) -> Option<ParsedTypeNode> {
        if self.is_reserved_symbol_at(position, "@[") {
            return Some(self.parse_library_annotation_at(position));
        }

        let marker = self.annotation_marker_text_at(position)?.to_owned();
        if is_standalone_diagnostic_annotation_marker(marker.as_str()) {
            return None;
        }

        let mut children = vec![self.token_node_ids[position]];
        let mut recovery_nodes = Vec::new();
        let mut cursor = position + 1;

        match marker.as_str() {
            "@proof_hint" => {
                if self.is_reserved_symbol_at(cursor, "(") {
                    let options = self.parse_proof_hint_option_list_at(cursor);
                    cursor = options.next_position;
                    children.push(options.id);
                    recovery_nodes.extend(options.recovery_nodes);
                } else {
                    self.diagnose_malformed_annotation(cursor, "expected `(` after `@proof_hint`");
                    self.push_malformed_annotation(cursor, &mut children, &mut recovery_nodes);
                }
            }
            "@latex" => {
                if self.is_reserved_symbol_at(cursor, "(") {
                    let arguments = self.parse_required_annotation_argument_list_at(
                        cursor,
                        AnnotationArgumentRequirement::StringLiteral,
                        "expected string literal in `@latex` annotation",
                    );
                    cursor = arguments.next_position;
                    children.push(arguments.id);
                    recovery_nodes.extend(arguments.recovery_nodes);
                } else {
                    self.diagnose_malformed_annotation(cursor, "expected annotation argument list");
                    self.push_missing_annotation_argument(
                        cursor,
                        &mut children,
                        &mut recovery_nodes,
                    );
                }
            }
            "@suppress" => {
                if self.is_reserved_symbol_at(cursor, "(") {
                    let arguments = self.parse_required_annotation_argument_list_at(
                        cursor,
                        AnnotationArgumentRequirement::Identifier,
                        "expected identifier in `@suppress` annotation",
                    );
                    cursor = arguments.next_position;
                    children.push(arguments.id);
                    recovery_nodes.extend(arguments.recovery_nodes);
                } else {
                    self.diagnose_malformed_annotation(cursor, "expected annotation argument list");
                    self.push_missing_annotation_argument(
                        cursor,
                        &mut children,
                        &mut recovery_nodes,
                    );
                }
            }
            "@show_thesis" | "@show_resolution" => {
                if self.is_reserved_symbol_at(cursor, "(") {
                    self.diagnose_malformed_annotation(
                        cursor,
                        "unexpected argument list for diagnostic annotation",
                    );
                    let arguments = self.parse_annotation_argument_list_at(cursor);
                    cursor = arguments.next_position;
                    children.push(arguments.id);
                    recovery_nodes.extend(arguments.recovery_nodes);
                }
            }
            _ => {
                if self.is_reserved_symbol_at(cursor, "(") {
                    let arguments = self.parse_annotation_argument_list_at(cursor);
                    cursor = arguments.next_position;
                    children.push(arguments.id);
                    recovery_nodes.extend(arguments.recovery_nodes);
                }
            }
        }

        let id = self.events.emit(SyntaxEvent::Node {
            kind: SurfaceNodeKind::Annotation,
            range: self.covering_token_range(position, cursor),
            children,
        });
        Some(ParsedTypeNode {
            id,
            next_position: cursor.max(position + 1),
            recovery_nodes,
        })
    }

    pub(super) fn parse_library_annotation_at(&mut self, position: usize) -> ParsedTypeNode {
        let mut library_children = vec![self.token_node_ids[position]];
        let mut recovery_nodes = Vec::new();
        let mut cursor = position + 1;

        let labels = self.parse_annotation_label_list_at(cursor);
        cursor = labels.next_position;
        library_children.push(labels.id);
        recovery_nodes.extend(labels.recovery_nodes);

        if self.is_reserved_symbol_at(cursor, "]") {
            library_children.push(self.token_node_ids[cursor]);
            cursor += 1;
        } else {
            self.diagnose_unmatched_annotation_delimiter(position, cursor, "]");
            let recovery = self.add_recovery_node(
                SyntaxRecoveryKind::UnmatchedOpeningDelimiter,
                self.zero_range_at(cursor),
                Vec::new(),
            );
            library_children.push(recovery);
            recovery_nodes.push(recovery);
        }

        let library = self.events.emit(SyntaxEvent::Node {
            kind: SurfaceNodeKind::LibraryAnnotation,
            range: self.covering_token_range(position, cursor),
            children: library_children,
        });
        let id = self.events.emit(SyntaxEvent::Node {
            kind: SurfaceNodeKind::Annotation,
            range: self.covering_token_range(position, cursor),
            children: vec![library],
        });
        ParsedTypeNode {
            id,
            next_position: cursor.max(position + 1),
            recovery_nodes,
        }
    }

    pub(super) fn parse_annotation_label_list_at(&mut self, position: usize) -> ParsedTypeNode {
        let mut children = Vec::new();
        let mut recovery_nodes = Vec::new();
        let mut cursor = position;
        let mut expecting_label = true;

        while cursor < self.request.tokens.len()
            && !self.is_reserved_symbol_at(cursor, "]")
            && !self.is_annotation_label_list_recovery_boundary_at(cursor)
        {
            if self.is_reserved_symbol_at(cursor, ",") {
                if expecting_label {
                    self.diagnose_malformed_annotation(cursor, "expected annotation label");
                    self.push_malformed_annotation(cursor, &mut children, &mut recovery_nodes);
                }
                children.push(self.token_node_ids[cursor]);
                cursor += 1;
                expecting_label = true;
                continue;
            }

            if !expecting_label {
                self.diagnose_malformed_annotation(cursor, "expected `,` between labels");
            }

            let label = self.parse_annotation_label_at(cursor);
            let made_progress = label.next_position > cursor;
            cursor = label.next_position;
            children.push(label.id);
            recovery_nodes.extend(label.recovery_nodes);
            expecting_label = false;
            if !made_progress {
                break;
            }

            if self.is_reserved_symbol_at(cursor, ",") {
                children.push(self.token_node_ids[cursor]);
                cursor += 1;
                expecting_label = true;
            }
        }

        if expecting_label && !self.is_annotation_label_list_recovery_boundary_at(cursor) {
            self.diagnose_malformed_annotation(cursor, "expected annotation label");
            self.push_malformed_annotation(cursor, &mut children, &mut recovery_nodes);
        }

        let range = if cursor > position {
            self.covering_token_range(position, cursor)
        } else {
            self.zero_range_at(position)
        };
        let id = self.events.emit(SyntaxEvent::Node {
            kind: SurfaceNodeKind::AnnotationLabelList,
            range,
            children,
        });
        ParsedTypeNode {
            id,
            next_position: cursor,
            recovery_nodes,
        }
    }

    pub(super) fn parse_annotation_label_at(&mut self, position: usize) -> ParsedTypeNode {
        let mut children = Vec::new();
        let mut recovery_nodes = Vec::new();
        let mut cursor = position;

        if self.is_identifier_at(cursor) {
            children.push(self.token_node_ids[cursor]);
            cursor += 1;
        } else {
            self.diagnose_malformed_annotation(cursor, "expected annotation label name");
            self.push_malformed_annotation(cursor, &mut children, &mut recovery_nodes);
        }

        if self.is_reserved_symbol_at(cursor, "(") {
            let arguments = self.parse_annotation_argument_list_at(cursor);
            cursor = arguments.next_position;
            children.push(arguments.id);
            recovery_nodes.extend(arguments.recovery_nodes);
        }

        let range = if cursor > position {
            self.covering_token_range(position, cursor)
        } else {
            self.zero_range_at(position)
        };
        let id = self.events.emit(SyntaxEvent::Node {
            kind: SurfaceNodeKind::AnnotationLabel,
            range,
            children,
        });
        ParsedTypeNode {
            id,
            next_position: cursor.max(position),
            recovery_nodes,
        }
    }

    pub(super) fn parse_required_annotation_argument_list_at(
        &mut self,
        position: usize,
        requirement: AnnotationArgumentRequirement,
        missing_message: &'static str,
    ) -> ParsedTypeNode {
        let mut children = vec![self.token_node_ids[position]];
        let mut recovery_nodes = Vec::new();
        let mut cursor = position + 1;

        let argument =
            self.parse_required_annotation_argument_at(cursor, requirement, missing_message);
        let made_progress = argument.next_position > cursor;
        cursor = argument.next_position;
        children.push(argument.id);
        recovery_nodes.extend(argument.recovery_nodes);

        while made_progress && self.is_reserved_symbol_at(cursor, ",") {
            self.diagnose_malformed_annotation(cursor, "unexpected extra annotation argument");
            children.push(self.token_node_ids[cursor]);
            cursor += 1;
            let extra = self.parse_annotation_argument_at(cursor);
            let extra_progress = extra.next_position > cursor;
            cursor = extra.next_position;
            children.push(extra.id);
            recovery_nodes.extend(extra.recovery_nodes);
            if !extra_progress {
                break;
            }
        }

        if self.is_reserved_symbol_at(cursor, ")") {
            children.push(self.token_node_ids[cursor]);
            cursor += 1;
        } else {
            self.diagnose_unmatched_annotation_delimiter(position, cursor, ")");
            let recovery = self.add_recovery_node(
                SyntaxRecoveryKind::UnmatchedOpeningDelimiter,
                self.zero_range_at(cursor),
                Vec::new(),
            );
            children.push(recovery);
            recovery_nodes.push(recovery);
        }

        let id = self.events.emit(SyntaxEvent::Node {
            kind: SurfaceNodeKind::AnnotationArgumentList,
            range: self.covering_token_range(position, cursor),
            children,
        });
        ParsedTypeNode {
            id,
            next_position: cursor.max(position + 1),
            recovery_nodes,
        }
    }

    pub(super) fn parse_required_annotation_argument_at(
        &mut self,
        position: usize,
        requirement: AnnotationArgumentRequirement,
        missing_message: &'static str,
    ) -> ParsedTypeNode {
        let mut children = Vec::new();
        let mut recovery_nodes = Vec::new();
        let mut cursor = position;

        if self.annotation_argument_requirement_matches(position, requirement) {
            children.push(self.token_node_ids[cursor]);
            cursor += 1;
        } else {
            self.diagnose_malformed_annotation(cursor, missing_message);
            self.push_missing_annotation_argument(cursor, &mut children, &mut recovery_nodes);
            if self.is_annotation_argument_token_at(cursor) {
                children.push(self.token_node_ids[cursor]);
                cursor += 1;
            }
        }

        let range = if cursor > position {
            self.covering_token_range(position, cursor)
        } else {
            self.zero_range_at(position)
        };
        let id = self.events.emit(SyntaxEvent::Node {
            kind: SurfaceNodeKind::AnnotationArgument,
            range,
            children,
        });
        ParsedTypeNode {
            id,
            next_position: cursor.max(position),
            recovery_nodes,
        }
    }

    pub(super) fn parse_annotation_argument_list_at(&mut self, position: usize) -> ParsedTypeNode {
        self.parse_delimited_annotation_items_at(
            position,
            SurfaceNodeKind::AnnotationArgumentList,
            "expected annotation argument",
            AnnotationDelimitedListBoundary::Argument,
            Parser::parse_annotation_argument_at,
        )
    }

    pub(super) fn parse_proof_hint_option_list_at(&mut self, position: usize) -> ParsedTypeNode {
        self.parse_delimited_annotation_items_at(
            position,
            SurfaceNodeKind::ProofHintOptionList,
            "expected proof-hint option",
            AnnotationDelimitedListBoundary::ProofHintOption,
            Parser::parse_proof_hint_option_at,
        )
    }

    pub(super) fn parse_delimited_annotation_items_at(
        &mut self,
        position: usize,
        kind: SurfaceNodeKind,
        missing_message: &'static str,
        boundary: AnnotationDelimitedListBoundary,
        mut parse_item: impl FnMut(&mut Self, usize) -> ParsedTypeNode,
    ) -> ParsedTypeNode {
        let mut children = vec![self.token_node_ids[position]];
        let mut recovery_nodes = Vec::new();
        let mut cursor = position + 1;
        let mut expecting_item = true;

        while cursor < self.request.tokens.len()
            && !self.is_annotation_argument_list_boundary_at(cursor, boundary)
        {
            if self.is_reserved_symbol_at(cursor, ",") {
                if expecting_item {
                    self.diagnose_malformed_annotation(cursor, missing_message);
                    self.push_missing_annotation_argument(
                        cursor,
                        &mut children,
                        &mut recovery_nodes,
                    );
                }
                children.push(self.token_node_ids[cursor]);
                cursor += 1;
                expecting_item = true;
                continue;
            }

            if !expecting_item {
                self.diagnose_malformed_annotation(cursor, "expected `,` between annotation items");
            }

            let item = parse_item(self, cursor);
            let made_progress = item.next_position > cursor;
            cursor = item.next_position;
            children.push(item.id);
            recovery_nodes.extend(item.recovery_nodes);
            expecting_item = false;
            if !made_progress {
                break;
            }

            if self.is_reserved_symbol_at(cursor, ",") {
                children.push(self.token_node_ids[cursor]);
                cursor += 1;
                expecting_item = true;
            }
        }

        if expecting_item {
            self.diagnose_malformed_annotation(cursor, missing_message);
            self.push_missing_annotation_argument(cursor, &mut children, &mut recovery_nodes);
        }

        if self.is_reserved_symbol_at(cursor, ")") {
            children.push(self.token_node_ids[cursor]);
            cursor += 1;
        } else {
            self.diagnose_unmatched_annotation_delimiter(position, cursor, ")");
            let recovery = self.add_recovery_node(
                SyntaxRecoveryKind::UnmatchedOpeningDelimiter,
                self.zero_range_at(cursor),
                Vec::new(),
            );
            children.push(recovery);
            recovery_nodes.push(recovery);
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

    pub(super) fn parse_annotation_argument_at(&mut self, position: usize) -> ParsedTypeNode {
        let mut children = Vec::new();
        let mut recovery_nodes = Vec::new();
        let mut cursor = position;

        if self.is_annotation_argument_token_at(cursor) {
            children.push(self.token_node_ids[cursor]);
            cursor += 1;
        } else {
            self.diagnose_malformed_annotation(cursor, "expected annotation argument");
            self.push_missing_annotation_argument(cursor, &mut children, &mut recovery_nodes);
            if cursor < self.request.tokens.len()
                && !self.is_reserved_symbol_at(cursor, ",")
                && !self.is_annotation_argument_list_boundary_at(
                    cursor,
                    AnnotationDelimitedListBoundary::Argument,
                )
            {
                children.push(self.token_node_ids[cursor]);
                cursor += 1;
            }
        }

        let range = if cursor > position {
            self.covering_token_range(position, cursor)
        } else {
            self.zero_range_at(position)
        };
        let id = self.events.emit(SyntaxEvent::Node {
            kind: SurfaceNodeKind::AnnotationArgument,
            range,
            children,
        });
        ParsedTypeNode {
            id,
            next_position: cursor.max(position),
            recovery_nodes,
        }
    }

    pub(super) fn parse_proof_hint_option_at(&mut self, position: usize) -> ParsedTypeNode {
        let mut children = Vec::new();
        let mut recovery_nodes = Vec::new();
        let mut cursor = position;

        let value_kind = if self.is_identifier_at(cursor) {
            let value_kind = match self.request.tokens[cursor].text.as_ref() {
                "max_axioms" | "timeout" => Some(ProofHintOptionValueKind::NatLiteral),
                "solver" => Some(ProofHintOptionValueKind::SolverName),
                _ => {
                    self.diagnose_malformed_annotation(cursor, "unknown proof-hint option name");
                    None
                }
            };
            children.push(self.token_node_ids[cursor]);
            cursor += 1;
            value_kind
        } else {
            self.diagnose_malformed_annotation(cursor, "expected proof-hint option name");
            self.push_missing_annotation_argument(cursor, &mut children, &mut recovery_nodes);
            None
        };

        if self.is_reserved_symbol_at(cursor, ":") {
            children.push(self.token_node_ids[cursor]);
            cursor += 1;
        } else {
            self.diagnose_malformed_annotation(cursor, "expected `:` in proof-hint option");
        }

        let value_matches = match value_kind {
            Some(ProofHintOptionValueKind::NatLiteral) => self.is_numeral_at(cursor),
            Some(ProofHintOptionValueKind::SolverName) => self.is_solver_name_at(cursor),
            None => self.is_identifier_at(cursor) || self.is_numeral_at(cursor),
        };
        if value_matches {
            children.push(self.token_node_ids[cursor]);
            cursor += 1;
        } else {
            let message = match value_kind {
                Some(ProofHintOptionValueKind::NatLiteral) => {
                    "expected numeric proof-hint option value"
                }
                Some(ProofHintOptionValueKind::SolverName) => {
                    "expected solver proof-hint option value"
                }
                None => "expected proof-hint option value",
            };
            self.diagnose_malformed_annotation(cursor, message);
            self.push_missing_annotation_argument(cursor, &mut children, &mut recovery_nodes);
            if cursor < self.request.tokens.len()
                && !self.is_reserved_symbol_at(cursor, ",")
                && !self.is_annotation_argument_list_boundary_at(
                    cursor,
                    AnnotationDelimitedListBoundary::ProofHintOption,
                )
            {
                children.push(self.token_node_ids[cursor]);
                cursor += 1;
            }
        }

        let range = if cursor > position {
            self.covering_token_range(position, cursor)
        } else {
            self.zero_range_at(position)
        };
        let id = self.events.emit(SyntaxEvent::Node {
            kind: SurfaceNodeKind::ProofHintOption,
            range,
            children,
        });
        ParsedTypeNode {
            id,
            next_position: cursor.max(position),
            recovery_nodes,
        }
    }

    pub(super) fn parse_standalone_diagnostic_annotation_at(
        &mut self,
        position: usize,
    ) -> Option<ParsedTypeNode> {
        if !self.is_standalone_diagnostic_annotation_start_at(position) {
            return None;
        }

        let mut children = vec![self.token_node_ids[position]];
        let mut recovery_nodes = Vec::new();
        let mut cursor = position + 1;

        if self.is_reserved_symbol_at(cursor, "(") {
            children.push(self.token_node_ids[cursor]);
            cursor += 1;
        } else {
            self.diagnose_malformed_annotation(cursor, "expected `(` after diagnostic annotation");
        }

        if let Some(term) = self.parse_term_expression_at(cursor) {
            cursor = term.next_position;
            children.push(term.id);
            recovery_nodes.extend(term.recovery_nodes);
        } else {
            self.diagnose_malformed_term_expression(
                cursor,
                "expected term expression in diagnostic annotation",
            );
            self.push_missing_term(cursor, &mut children, &mut recovery_nodes);
        }

        if self.is_reserved_symbol_at(cursor, ")") {
            children.push(self.token_node_ids[cursor]);
            cursor += 1;
        } else {
            self.diagnose_unmatched_annotation_delimiter(position, cursor, ")");
            let recovery = self.add_recovery_node(
                SyntaxRecoveryKind::UnmatchedOpeningDelimiter,
                self.zero_range_at(cursor),
                Vec::new(),
            );
            children.push(recovery);
            recovery_nodes.push(recovery);
        }

        let id = self.events.emit(SyntaxEvent::Node {
            kind: SurfaceNodeKind::StandaloneDiagnosticAnnotation,
            range: self.covering_token_range(position, cursor),
            children,
        });
        Some(ParsedTypeNode {
            id,
            next_position: cursor.max(position + 1),
            recovery_nodes,
        })
    }

    pub(super) fn finish_annotated_type_node(
        &mut self,
        position: usize,
        prefix: ParsedAnnotationPrefix,
        node: Option<ParsedTypeNode>,
        kind: SurfaceNodeKind,
        missing_message: &'static str,
    ) -> ParsedTypeNode {
        let mut children = prefix.children;
        let mut recovery_nodes = prefix.recovery_nodes;
        let mut cursor = prefix.next_position;

        if let Some(node) = node {
            cursor = node.next_position;
            children.push(node.id);
            recovery_nodes.extend(node.recovery_nodes);
        } else {
            self.diagnose_malformed_annotation(cursor, missing_message);
            let recovery = self.add_missing_statement(cursor);
            children.push(recovery);
            recovery_nodes.push(recovery);
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
}
