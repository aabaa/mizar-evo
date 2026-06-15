use crate::{
    ParserToken, ParserTokenKind,
    cursor::is_reserved_word_token,
    diagnostic::{ExpectedToken, expected_token_diagnostic},
    event::SyntaxEvent,
    grammar::Parser,
    path::ParsedPathNode,
    sync::{self, is_top_level_item_keyword},
};
use mizar_session::SourceRange;
use mizar_syntax::{
    SkippedTokenReason, SurfaceBuilderNodeId, SurfaceNodeKind, SyntaxDiagnostic,
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
}

fn is_block_like_top_level_start(token: &ParserToken) -> bool {
    token.kind == ParserTokenKind::ReservedWord
        && matches!(token.text.as_ref(), "definition" | "registration" | "claim")
}

fn is_reserved_symbol_token(token: &ParserToken, spelling: &str) -> bool {
    token.kind == ParserTokenKind::ReservedSymbol && token.text.as_ref() == spelling
}

#[cfg(test)]
mod tests {
    use crate::{ParseRequest, ParserToken, ParserTokenKind, parse};
    use mizar_session::{
        BuildSnapshotId, Edition, Hash, InMemorySessionIdAllocator, SessionIdAllocator, SourceId,
        SourceRange,
    };
    use mizar_syntax::{
        SkippedTokenReason, SurfaceNodeKind, SyntaxDiagnosticCode, SyntaxRecoveryKind,
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

        assert!(output.diagnostics.is_empty());
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

        assert!(output.diagnostics.is_empty());
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

    fn rowan_token_texts(ast: &mizar_syntax::SurfaceAst) -> Vec<String> {
        ast.rowan_root()
            .descendants_with_tokens()
            .filter_map(|element| element.into_token())
            .map(|token| token.text().to_owned())
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
