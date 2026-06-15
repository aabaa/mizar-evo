#![allow(dead_code)] // reason: namespace and qualified path helpers are locked by unit tests before later consumers land.

use crate::{ParserToken, ParserTokenKind, event::SyntaxEvent, grammar::Parser};
use mizar_session::SourceRange;
use mizar_syntax::{SurfaceBuilderNodeId, SurfaceNodeKind};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct ParsedPathNode {
    pub(super) id: SurfaceBuilderNodeId,
    pub(super) next_position: usize,
}

impl Parser {
    pub(super) fn parse_module_path_at(&mut self, position: usize) -> Option<ParsedPathNode> {
        let plan = ModulePathPlan::parse(&self.request.tokens, position)?;
        let mut children = Vec::new();
        if let Some(prefix_position) = plan.prefix_position {
            children
                .push(self.emit_wrapped_token(SurfaceNodeKind::RelativePrefix, prefix_position));
        }
        children.push(self.emit_wrapped_token(SurfaceNodeKind::PathSegment, plan.first_segment));
        for (separator, segment) in &plan.rest {
            children.push(self.token_node_ids[*separator]);
            children.push(self.emit_wrapped_token(SurfaceNodeKind::PathSegment, *segment));
        }
        let id = self.events.emit(SyntaxEvent::Node {
            kind: SurfaceNodeKind::ModulePath,
            range: covering_range(
                self.request.tokens[plan.first_position()].span,
                self.request.tokens[plan.last_segment()].span,
            ),
            children,
        });
        Some(ParsedPathNode {
            id,
            next_position: plan.next_position,
        })
    }

    pub(super) fn parse_namespace_path_at(&mut self, position: usize) -> Option<ParsedPathNode> {
        let plan = NamespacePathPlan::parse(&self.request.tokens, position)?;
        let mut children =
            vec![self.emit_wrapped_token(SurfaceNodeKind::PathSegment, plan.first_segment)];
        for (separator, segment) in &plan.rest {
            children.push(self.token_node_ids[*separator]);
            children.push(self.emit_wrapped_token(SurfaceNodeKind::PathSegment, *segment));
        }
        let id = self.events.emit(SyntaxEvent::Node {
            kind: SurfaceNodeKind::NamespacePath,
            range: covering_range(
                self.request.tokens[plan.first_segment].span,
                self.request.tokens[plan.last_segment()].span,
            ),
            children,
        });
        Some(ParsedPathNode {
            id,
            next_position: plan.next_position,
        })
    }

    pub(super) fn parse_qualified_symbol_at(&mut self, position: usize) -> Option<ParsedPathNode> {
        let plan = QualifiedSymbolPlan::parse(&self.request.tokens, position)?;
        let mut children = Vec::new();
        for (segment, separator) in &plan.namespace_segments {
            children.push(self.emit_wrapped_token(SurfaceNodeKind::PathSegment, *segment));
            children.push(self.token_node_ids[*separator]);
        }
        children.push(self.emit_wrapped_token(SurfaceNodeKind::PathSegment, plan.final_symbol));
        let id = self.events.emit(SyntaxEvent::Node {
            kind: SurfaceNodeKind::QualifiedSymbol,
            range: covering_range(
                self.request.tokens[plan.first_position()].span,
                self.request.tokens[plan.final_symbol].span,
            ),
            children,
        });
        Some(ParsedPathNode {
            id,
            next_position: plan.next_position,
        })
    }

    pub(super) fn emit_wrapped_token(
        &mut self,
        kind: SurfaceNodeKind,
        token_position: usize,
    ) -> SurfaceBuilderNodeId {
        self.events.emit(SyntaxEvent::Node {
            kind,
            range: self.request.tokens[token_position].span,
            children: vec![self.token_node_ids[token_position]],
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ModulePathPlan {
    prefix_position: Option<usize>,
    first_segment: usize,
    rest: Vec<(usize, usize)>,
    next_position: usize,
}

impl ModulePathPlan {
    fn parse(tokens: &[ParserToken], position: usize) -> Option<Self> {
        let mut cursor = position;
        let prefix_position = tokens
            .get(cursor)
            .is_some_and(|token| {
                is_reserved_symbol_token(token, ".") || is_reserved_symbol_token(token, "..")
            })
            .then(|| {
                cursor += 1;
                position
            });
        let first_segment = cursor;
        if !tokens
            .get(first_segment)
            .is_some_and(|token| token.kind == ParserTokenKind::Identifier)
        {
            return None;
        }
        cursor += 1;
        let rest = parse_identifier_dot_tail(tokens, &mut cursor);
        Some(Self {
            prefix_position,
            first_segment,
            rest,
            next_position: cursor,
        })
    }

    fn first_position(&self) -> usize {
        self.prefix_position.unwrap_or(self.first_segment)
    }

    fn last_segment(&self) -> usize {
        self.rest
            .last()
            .map_or(self.first_segment, |(_, segment)| *segment)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct NamespacePathPlan {
    first_segment: usize,
    rest: Vec<(usize, usize)>,
    next_position: usize,
}

impl NamespacePathPlan {
    fn parse(tokens: &[ParserToken], position: usize) -> Option<Self> {
        if !tokens
            .get(position)
            .is_some_and(|token| token.kind == ParserTokenKind::Identifier)
        {
            return None;
        }
        let mut cursor = position + 1;
        let rest = parse_identifier_dot_tail(tokens, &mut cursor);
        Some(Self {
            first_segment: position,
            rest,
            next_position: cursor,
        })
    }

    fn last_segment(&self) -> usize {
        self.rest
            .last()
            .map_or(self.first_segment, |(_, segment)| *segment)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct QualifiedSymbolPlan {
    namespace_segments: Vec<(usize, usize)>,
    final_symbol: usize,
    next_position: usize,
}

impl QualifiedSymbolPlan {
    fn parse(tokens: &[ParserToken], position: usize) -> Option<Self> {
        let mut cursor = position;
        let mut namespace_segments = Vec::new();
        while tokens
            .get(cursor)
            .is_some_and(|token| token.kind == ParserTokenKind::Identifier)
            && tokens
                .get(cursor + 1)
                .is_some_and(|token| is_reserved_symbol_token(token, "."))
        {
            namespace_segments.push((cursor, cursor + 1));
            cursor += 2;
        }
        if !tokens
            .get(cursor)
            .is_some_and(|token| token.kind == ParserTokenKind::UserSymbol)
        {
            return None;
        }
        Some(Self {
            namespace_segments,
            final_symbol: cursor,
            next_position: cursor + 1,
        })
    }

    fn first_position(&self) -> usize {
        self.namespace_segments
            .first()
            .map_or(self.final_symbol, |(segment, _)| *segment)
    }
}

fn parse_identifier_dot_tail(tokens: &[ParserToken], cursor: &mut usize) -> Vec<(usize, usize)> {
    let mut rest = Vec::new();
    while tokens
        .get(*cursor)
        .is_some_and(|token| is_reserved_symbol_token(token, "."))
        && tokens
            .get(*cursor + 1)
            .is_some_and(|token| token.kind == ParserTokenKind::Identifier)
    {
        rest.push((*cursor, *cursor + 1));
        *cursor += 2;
    }
    rest
}

fn is_reserved_symbol_token(token: &ParserToken, spelling: &str) -> bool {
    token.kind == ParserTokenKind::ReservedSymbol && token.text.as_ref() == spelling
}

fn covering_range(first: SourceRange, last: SourceRange) -> SourceRange {
    SourceRange {
        source_id: first.source_id,
        start: first.start,
        end: last.end,
    }
}

#[cfg(test)]
mod tests {
    use super::ParsedPathNode;
    use crate::{ParseRequest, ParserToken, ParserTokenKind, event::SyntaxEvent, grammar::Parser};
    use mizar_session::{
        BuildSnapshotId, Edition, Hash, InMemorySessionIdAllocator, SessionIdAllocator, SourceId,
        SourceRange,
    };
    use mizar_syntax::{SurfaceAst, SurfaceNode, SurfaceNodeKind};

    #[test]
    fn module_path_helper_builds_relative_path_shape() {
        let source_id = source_id(30);
        let mut parser = parser_with_tokens(
            source_id,
            vec![
                token(source_id, ParserTokenKind::ReservedSymbol, "..", 0, 2),
                token(source_id, ParserTokenKind::Identifier, "std", 2, 5),
                token(source_id, ParserTokenKind::ReservedSymbol, ".", 5, 6),
                token(source_id, ParserTokenKind::Identifier, "algebra", 6, 13),
                token(source_id, ParserTokenKind::ReservedSymbol, ";", 13, 14),
            ],
        );

        let parsed = parser
            .parse_module_path_at(0)
            .expect("relative module path should parse");
        assert_eq!(parsed.next_position, 4);
        let ast = finish_with_root(parser, source_id, parsed);
        let module_path = single_node(&ast, |node| {
            matches!(node.kind, SurfaceNodeKind::ModulePath)
        });

        assert_eq!(module_path.range, range(source_id, 0, 13));
        assert_eq!(module_path.children.len(), 4);
        assert!(matches!(
            ast.node(module_path.children[0]).unwrap().kind,
            SurfaceNodeKind::RelativePrefix
        ));
        assert_eq!(wrapped_token_text(&ast, module_path.children[0]), "..");
        assert!(matches!(
            ast.node(module_path.children[1]).unwrap().kind,
            SurfaceNodeKind::PathSegment
        ));
        assert_eq!(wrapped_token_text(&ast, module_path.children[1]), "std");
        assert_eq!(
            ast.node(module_path.children[2]).unwrap().token_text(),
            Some(".")
        );
        assert_eq!(wrapped_token_text(&ast, module_path.children[3]), "algebra");
        assert!(ast.snapshot_text().contains("ModulePath range=0..13"));
        assert!(ast.snapshot_text().contains("RelativePrefix range=0..2"));
    }

    #[test]
    fn module_path_helper_accepts_absolute_and_current_relative_prefixes() {
        for (byte, tokens, expected_next, expected_children, first_child_kind) in [
            (
                35,
                vec![("std", ParserTokenKind::Identifier, 0, 3)],
                1,
                1,
                "PathSegment",
            ),
            (
                36,
                vec![
                    (".", ParserTokenKind::ReservedSymbol, 0, 1),
                    ("std", ParserTokenKind::Identifier, 1, 4),
                ],
                2,
                2,
                "RelativePrefix",
            ),
        ] {
            let source_id = source_id(byte);
            let mut parser = parser_with_tokens(
                source_id,
                tokens
                    .into_iter()
                    .map(|(text, kind, start, end)| token(source_id, kind, text, start, end))
                    .collect(),
            );

            let parsed = parser
                .parse_module_path_at(0)
                .expect("module path variant should parse");
            assert_eq!(parsed.next_position, expected_next);
            let ast = finish_with_root(parser, source_id, parsed);
            let module_path = single_node(&ast, |node| {
                matches!(node.kind, SurfaceNodeKind::ModulePath)
            });

            assert_eq!(module_path.children.len(), expected_children);
            let first_child = ast.node(module_path.children[0]).unwrap();
            match first_child_kind {
                "PathSegment" => assert!(matches!(first_child.kind, SurfaceNodeKind::PathSegment)),
                "RelativePrefix" => {
                    assert!(matches!(first_child.kind, SurfaceNodeKind::RelativePrefix));
                    assert_eq!(wrapped_token_text(&ast, module_path.children[0]), ".");
                }
                _ => unreachable!("unexpected fixture kind"),
            }
        }
    }

    #[test]
    fn namespace_path_helper_rejects_relative_prefixes() {
        let current_source_id = source_id(31);
        let mut parser = parser_with_tokens(
            current_source_id,
            vec![
                token(
                    current_source_id,
                    ParserTokenKind::ReservedSymbol,
                    ".",
                    0,
                    1,
                ),
                token(
                    current_source_id,
                    ParserTokenKind::Identifier,
                    "local",
                    1,
                    6,
                ),
            ],
        );

        assert!(parser.parse_namespace_path_at(0).is_none());
        let ast = parser.events.finish(None, None);
        assert_eq!(
            ast.nodes().len(),
            2,
            "failed helper validation should not emit orphan path nodes"
        );

        let source_id = source_id(37);
        let mut parser = parser_with_tokens(
            source_id,
            vec![
                token(source_id, ParserTokenKind::ReservedSymbol, "..", 0, 2),
                token(source_id, ParserTokenKind::Identifier, "parent", 2, 8),
            ],
        );

        assert!(parser.parse_namespace_path_at(0).is_none());
        let ast = parser.events.finish(None, None);
        assert_eq!(ast.nodes().len(), 2);
    }

    #[test]
    fn namespace_path_helper_builds_identifier_dot_chain() {
        let source_id = source_id(32);
        let mut parser = parser_with_tokens(
            source_id,
            vec![
                token(source_id, ParserTokenKind::Identifier, "mml", 0, 3),
                token(source_id, ParserTokenKind::ReservedSymbol, ".", 3, 4),
                token(source_id, ParserTokenKind::Identifier, "nat", 4, 7),
            ],
        );

        let parsed = parser
            .parse_namespace_path_at(0)
            .expect("namespace path should parse");
        assert_eq!(parsed.next_position, 3);
        let ast = finish_with_root(parser, source_id, parsed);
        let namespace_path = single_node(&ast, |node| {
            matches!(node.kind, SurfaceNodeKind::NamespacePath)
        });

        assert_eq!(namespace_path.range, range(source_id, 0, 7));
        assert_eq!(wrapped_token_text(&ast, namespace_path.children[0]), "mml");
        assert_eq!(
            ast.node(namespace_path.children[1]).unwrap().token_text(),
            Some(".")
        );
        assert_eq!(wrapped_token_text(&ast, namespace_path.children[2]), "nat");
    }

    #[test]
    fn namespace_path_helper_accepts_single_identifier() {
        let source_id = source_id(38);
        let mut parser = parser_with_tokens(
            source_id,
            vec![token(source_id, ParserTokenKind::Identifier, "local", 0, 5)],
        );

        let parsed = parser
            .parse_namespace_path_at(0)
            .expect("single-segment namespace path should parse");
        assert_eq!(parsed.next_position, 1);
        let ast = finish_with_root(parser, source_id, parsed);
        let namespace_path = single_node(&ast, |node| {
            matches!(node.kind, SurfaceNodeKind::NamespacePath)
        });

        assert_eq!(namespace_path.children.len(), 1);
        assert_eq!(
            wrapped_token_text(&ast, namespace_path.children[0]),
            "local"
        );
    }

    #[test]
    fn qualified_symbol_helper_requires_final_user_symbol() {
        let source_id = source_id(33);
        let mut parser = parser_with_tokens(
            source_id,
            vec![
                token(source_id, ParserTokenKind::Identifier, "top", 0, 3),
                token(source_id, ParserTokenKind::ReservedSymbol, ".", 3, 4),
                token(source_id, ParserTokenKind::UserSymbol, "Space", 4, 9),
            ],
        );

        let parsed = parser
            .parse_qualified_symbol_at(0)
            .expect("qualified symbol should parse");
        assert_eq!(parsed.next_position, 3);
        let ast = finish_with_root(parser, source_id, parsed);
        let qualified_symbol = single_node(&ast, |node| {
            matches!(node.kind, SurfaceNodeKind::QualifiedSymbol)
        });

        assert_eq!(qualified_symbol.range, range(source_id, 0, 9));
        assert_eq!(
            wrapped_token_text(&ast, qualified_symbol.children[0]),
            "top"
        );
        assert_eq!(
            ast.node(qualified_symbol.children[1]).unwrap().token_text(),
            Some(".")
        );
        assert_eq!(
            wrapped_token_text(&ast, qualified_symbol.children[2]),
            "Space"
        );
    }

    #[test]
    fn qualified_symbol_helper_accepts_bare_user_symbol() {
        let source_id = source_id(39);
        let mut parser = parser_with_tokens(
            source_id,
            vec![token(source_id, ParserTokenKind::UserSymbol, "Space", 0, 5)],
        );

        let parsed = parser
            .parse_qualified_symbol_at(0)
            .expect("bare user symbol should parse as qualified_symbol");
        assert_eq!(parsed.next_position, 1);
        let ast = finish_with_root(parser, source_id, parsed);
        let qualified_symbol = single_node(&ast, |node| {
            matches!(node.kind, SurfaceNodeKind::QualifiedSymbol)
        });

        assert_eq!(qualified_symbol.children.len(), 1);
        assert_eq!(
            wrapped_token_text(&ast, qualified_symbol.children[0]),
            "Space"
        );
    }

    #[test]
    fn qualified_symbol_helper_rejects_identifier_final_segment() {
        let source_id = source_id(34);
        let mut parser = parser_with_tokens(
            source_id,
            vec![
                token(source_id, ParserTokenKind::Identifier, "top", 0, 3),
                token(source_id, ParserTokenKind::ReservedSymbol, ".", 3, 4),
                token(source_id, ParserTokenKind::Identifier, "space", 4, 9),
            ],
        );

        assert!(parser.parse_qualified_symbol_at(0).is_none());
        let ast = parser.events.finish(None, None);
        assert_eq!(ast.nodes().len(), 3);
    }

    fn parser_with_tokens(source_id: SourceId, tokens: Vec<ParserToken>) -> Parser {
        let mut parser = Parser::new(ParseRequest::new(
            source_id,
            Edition::new("2026"),
            tokens,
            Vec::new(),
        ));
        parser.add_token_nodes();
        parser
    }

    fn finish_with_root(
        mut parser: Parser,
        source_id: SourceId,
        parsed: ParsedPathNode,
    ) -> SurfaceAst {
        let root_children = parser
            .token_node_ids
            .iter()
            .copied()
            .chain([parsed.id])
            .collect();
        let root = parser.events.emit(SyntaxEvent::Node {
            kind: SurfaceNodeKind::Root,
            range: range(source_id, 0, parser.request.tokens.last().unwrap().span.end),
            children: root_children,
        });
        parser.events.finish(Some(root), None)
    }

    fn single_node(ast: &SurfaceAst, predicate: impl Fn(&SurfaceNode) -> bool) -> &SurfaceNode {
        ast.nodes()
            .iter()
            .filter(|node| predicate(node))
            .exactly_one()
            .expect("expected exactly one matching node")
    }

    fn wrapped_token_text(ast: &SurfaceAst, id: mizar_syntax::SurfaceNodeId) -> &str {
        let node = ast.node(id).unwrap();
        let token_id = *node
            .children
            .first()
            .expect("wrapper should have a token child");
        ast.node(token_id)
            .unwrap()
            .token_text()
            .expect("wrapper child should be a token")
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

    trait ExactlyOne: Iterator + Sized {
        fn exactly_one(mut self) -> Result<Self::Item, ()> {
            let Some(item) = self.next() else {
                return Err(());
            };
            if self.next().is_some() {
                return Err(());
            }
            Ok(item)
        }
    }

    impl<I: Iterator> ExactlyOne for I {}
}
