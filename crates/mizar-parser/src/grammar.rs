use crate::{
    OperatorFixity, OperatorFixityEntry, ParseOutput, ParseRequest, ParserToken, ParserTokenKind,
    cursor::TokenCursor, event::SyntaxEvent, event::SyntaxEventSink, recovery,
};
use mizar_session::SourceRange;
use mizar_syntax::{
    SurfaceBuilderNodeId, SurfaceNodeKind, SurfaceTokenKind, SurfaceTriviaBuilder,
    SyntaxDiagnostic, SyntaxDiagnosticCode, SyntaxRecoveryKind,
};
use std::{collections::BTreeMap, sync::Arc};

pub(crate) struct Parser {
    pub(super) request: ParseRequest,
    pub(super) events: SyntaxEventSink,
    pub(super) token_node_ids: Vec<SurfaceBuilderNodeId>,
    pub(super) diagnostics: Vec<SyntaxDiagnostic>,
    pub(super) fixity: BTreeMap<Arc<str>, Vec<OperatorFixityEntry>>,
    pub(super) trivia: SurfaceTriviaBuilder,
}

impl Parser {
    pub(crate) fn new(request: ParseRequest) -> Self {
        let source_id = request.source_id;
        let mut fixity: BTreeMap<Arc<str>, Vec<OperatorFixityEntry>> = BTreeMap::new();
        for entry in request.operator_fixity.iter().cloned() {
            fixity
                .entry(entry.spelling.clone())
                .or_default()
                .push(entry);
        }
        Self {
            events: SyntaxEventSink::new(request.source_id),
            request,
            token_node_ids: Vec::new(),
            diagnostics: Vec::new(),
            fixity,
            trivia: SurfaceTriviaBuilder::new(source_id),
        }
    }

    pub(crate) fn parse(mut self) -> ParseOutput {
        self.add_token_nodes();
        if self.recover_syntax() == recovery::RecoveryOutcome::Unrecoverable {
            return ParseOutput {
                ast: None,
                diagnostics: self.diagnostics,
            };
        }
        let expression_root = self.parse_expression();
        let module = self.parse_compilation_unit();
        let root = self.add_root(expression_root, module.id, &module.recovery_nodes);
        let events = self.events;
        let trivia = self.trivia.finish();
        ParseOutput {
            ast: Some(
                events
                    .finish(Some(root), expression_root)
                    .with_trivia(trivia),
            ),
            diagnostics: self.diagnostics,
        }
    }

    pub(super) fn add_token_nodes(&mut self) {
        let tokens = self.request.tokens.clone();
        let mut cursor = TokenCursor::new(self.request.source_id, &tokens);
        while let Some(token) = cursor.advance() {
            let id = if token.kind == ParserTokenKind::ErrorRecovery {
                self.diagnostics.push(SyntaxDiagnostic::new(
                    SyntaxDiagnosticCode::UnexpectedErrorToken,
                    "error-recovery token reached the parser",
                    token.span,
                ));
                self.events.emit(SyntaxEvent::Token {
                    kind: surface_token_kind(token.kind),
                    text: token.text.clone(),
                    range: token.span,
                    recovered: true,
                })
            } else {
                self.events.emit(SyntaxEvent::Token {
                    kind: surface_token_kind(token.kind),
                    text: token.text.clone(),
                    range: token.span,
                    recovered: false,
                })
            };
            self.token_node_ids.push(id);
        }
    }

    pub(super) fn add_recovery_node(
        &mut self,
        recovery_kind: SyntaxRecoveryKind,
        range: SourceRange,
        children: Vec<SurfaceBuilderNodeId>,
    ) -> SurfaceBuilderNodeId {
        self.events.emit(SyntaxEvent::Recovery {
            kind: recovery_kind,
            range,
            children,
        })
    }

    fn add_root(
        &mut self,
        expression_root: Option<SurfaceBuilderNodeId>,
        compilation_unit: SurfaceBuilderNodeId,
        module_recovery_nodes: &[SurfaceBuilderNodeId],
    ) -> SurfaceBuilderNodeId {
        let children = self
            .token_node_ids
            .iter()
            .copied()
            .chain(expression_root)
            .chain([compilation_unit])
            .chain(
                self.events
                    .recovery_node_ids()
                    .iter()
                    .copied()
                    .filter(|id| !module_recovery_nodes.contains(id)),
            )
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
        self.events.emit(SyntaxEvent::Node {
            kind: SurfaceNodeKind::Root,
            range,
            children,
        })
    }

    pub(super) fn fixities_for_token(&self, token: &ParserToken) -> Option<&[OperatorFixityEntry]> {
        if !matches!(
            token.kind,
            ParserTokenKind::UserSymbol | ParserTokenKind::ReservedSymbol
        ) {
            return None;
        }
        self.fixity.get(&token.text).map(Vec::as_slice)
    }

    pub(super) fn infix_fixity_for_token(
        &self,
        token: &ParserToken,
    ) -> Option<&OperatorFixityEntry> {
        self.fixities_for_token(token)?
            .iter()
            .find(|entry| matches!(entry.fixity, OperatorFixity::Infix(_)))
    }

    pub(super) fn prefix_fixity_for_token(
        &self,
        token: &ParserToken,
    ) -> Option<&OperatorFixityEntry> {
        self.fixities_for_token(token)?
            .iter()
            .find(|entry| matches!(entry.fixity, OperatorFixity::Prefix))
    }

    pub(super) fn postfix_fixity_for_token(
        &self,
        token: &ParserToken,
    ) -> Option<&OperatorFixityEntry> {
        self.fixities_for_token(token)?
            .iter()
            .find(|entry| matches!(entry.fixity, OperatorFixity::Postfix))
    }
}

fn surface_token_kind(kind: ParserTokenKind) -> SurfaceTokenKind {
    match kind {
        ParserTokenKind::Identifier => SurfaceTokenKind::Identifier,
        ParserTokenKind::ReservedWord => SurfaceTokenKind::ReservedWord,
        ParserTokenKind::ReservedSymbol => SurfaceTokenKind::ReservedSymbol,
        ParserTokenKind::Numeral => SurfaceTokenKind::Numeral,
        ParserTokenKind::LexemeRun => SurfaceTokenKind::LexemeRun,
        ParserTokenKind::UserSymbol => SurfaceTokenKind::UserSymbol,
        ParserTokenKind::AnnotationMarker => SurfaceTokenKind::AnnotationMarker,
        ParserTokenKind::StringLiteral => SurfaceTokenKind::StringLiteral,
        ParserTokenKind::ErrorRecovery => SurfaceTokenKind::ErrorRecovery,
        ParserTokenKind::Unknown => SurfaceTokenKind::Unknown,
    }
}
