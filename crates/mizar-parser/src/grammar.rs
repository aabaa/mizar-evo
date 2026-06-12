use crate::{
    OperatorFixityEntry, ParseOutput, ParseRequest, ParserToken, ParserTokenKind, recovery,
};
use mizar_session::SourceRange;
use mizar_syntax::{
    SurfaceAstBuilder, SurfaceBuilderNodeId, SurfaceNodeKind, SurfaceTokenKind, SyntaxDiagnostic,
    SyntaxDiagnosticCode, SyntaxRecoveryKind,
};
use std::{collections::BTreeMap, sync::Arc};

pub(crate) struct Parser {
    pub(super) request: ParseRequest,
    pub(super) builder: SurfaceAstBuilder,
    pub(super) token_node_ids: Vec<SurfaceBuilderNodeId>,
    pub(super) diagnostics: Vec<SyntaxDiagnostic>,
    pub(super) fixity: BTreeMap<Arc<str>, OperatorFixityEntry>,
}

impl Parser {
    pub(crate) fn new(request: ParseRequest) -> Self {
        let fixity = request
            .operator_fixity
            .iter()
            .cloned()
            .map(|entry| (entry.spelling.clone(), entry))
            .collect();
        Self {
            builder: SurfaceAstBuilder::new(request.source_id),
            request,
            token_node_ids: Vec::new(),
            diagnostics: Vec::new(),
            fixity,
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
        let root = self.add_root(expression_root);
        ParseOutput {
            ast: Some(self.builder.finish(Some(root), expression_root)),
            diagnostics: self.diagnostics,
        }
    }

    fn add_token_nodes(&mut self) {
        let tokens = self.request.tokens.clone();
        for token in tokens {
            let id = if token.kind == ParserTokenKind::ErrorRecovery {
                self.diagnostics.push(SyntaxDiagnostic::new(
                    SyntaxDiagnosticCode::UnexpectedErrorToken,
                    "error-recovery token reached the parser",
                    token.span,
                ));
                self.builder.add_recovered_token(
                    surface_token_kind(token.kind),
                    token.text.clone(),
                    token.span,
                )
            } else {
                self.builder.add_token(
                    surface_token_kind(token.kind),
                    token.text.clone(),
                    token.span,
                )
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
        self.builder.add_recovery(recovery_kind, range, children)
    }

    fn add_root(&mut self, expression_root: Option<SurfaceBuilderNodeId>) -> SurfaceBuilderNodeId {
        let children = self
            .token_node_ids
            .iter()
            .copied()
            .chain(expression_root)
            .chain(self.builder.recovery_node_ids().iter().copied())
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
        self.builder
            .add_node(SurfaceNodeKind::Root, range, children)
    }

    pub(super) fn fixity_for_token(&self, token: &ParserToken) -> Option<&OperatorFixityEntry> {
        if !matches!(
            token.kind,
            ParserTokenKind::UserSymbol | ParserTokenKind::ReservedSymbol
        ) {
            return None;
        }
        self.fixity.get(&token.text)
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
        ParserTokenKind::StringLiteral => SurfaceTokenKind::StringLiteral,
        ParserTokenKind::ErrorRecovery => SurfaceTokenKind::ErrorRecovery,
        ParserTokenKind::Unknown => SurfaceTokenKind::Unknown,
    }
}
