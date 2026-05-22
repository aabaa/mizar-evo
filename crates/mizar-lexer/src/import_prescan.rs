use crate::raw_lexer::{
    RawToken, RawTokenKind, RawTokenStream, is_identifier, is_identifier_continue,
    is_identifier_start,
};
use crate::source::{SourcePos, SourceRange, SourceSpan};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportPrelude {
    pub imports: Vec<ImportStub>,
    pub end: SourcePos,
    pub diagnostics: Vec<ImportPrescanDiagnostic>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportStub {
    pub path: RawModulePath,
    pub alias: Option<RawModuleAlias>,
    pub span: SourceRange,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawModulePath {
    pub spelling: String,
    pub relative: Option<RawModuleRelativePrefix>,
    pub components: Vec<RawModulePathComponent>,
    pub source_segments: Vec<SourceRange>,
    pub span: SourceRange,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum RawModuleRelativePrefix {
    Current,
    Parent,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawModulePathComponent {
    pub spelling: String,
    pub span: SourceRange,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawModuleAlias {
    pub spelling: String,
    pub span: SourceRange,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportPrescanDiagnostic {
    pub code: ImportPrescanDiagnosticCode,
    pub message: String,
    pub span: SourceRange,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum ImportPrescanDiagnosticCode {
    MissingModulePath,
    EmptyModulePathComponent,
    MissingAlias,
    MissingSemicolon,
    UnexpectedToken,
}
pub fn scan_import_prelude(raw: &RawTokenStream) -> ImportPrelude {
    let tokens = split_import_prescan_tokens(raw);
    let mut scanner = ImportPrescanner::new(tokens);

    while scanner.peek_is_word("import") {
        scanner.parse_import_stmt();
    }

    ImportPrelude {
        imports: scanner.imports,
        end: scanner.end,
        diagnostics: scanner.diagnostics,
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
struct ImportPrescanToken {
    kind: ImportPrescanTokenKind,
    lexeme: String,
    span: SourceSpan,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ImportPrescanTokenKind {
    Word,
    Dot,
    DotDot,
    Star,
    Comma,
    Semicolon,
    LBrace,
    RBrace,
    Other,
}

struct ImportPrescanner {
    tokens: Vec<ImportPrescanToken>,
    cursor: usize,
    imports: Vec<ImportStub>,
    diagnostics: Vec<ImportPrescanDiagnostic>,
    end: SourcePos,
}

impl ImportPrescanner {
    fn new(tokens: Vec<ImportPrescanToken>) -> Self {
        let end = tokens.first().map_or(0, |token| token.span.start);
        Self {
            tokens,
            cursor: 0,
            imports: Vec::new(),
            diagnostics: Vec::new(),
            end,
        }
    }

    fn peek_is_word(&self, spelling: &str) -> bool {
        self.peek().is_some_and(|token| {
            token.kind == ImportPrescanTokenKind::Word && token.lexeme == spelling
        })
    }

    fn parse_import_stmt(&mut self) {
        self.advance().expect("caller checked import");
        let mut recovered_any = false;

        loop {
            match self.parse_module_alias_decls() {
                Some(imports) => {
                    self.imports.extend(imports);
                    recovered_any = true;
                }
                None => {
                    if self.peek_is(ImportPrescanTokenKind::Comma) {
                        self.diagnostic(
                            ImportPrescanDiagnosticCode::MissingModulePath,
                            "missing module path before comma",
                            self.peek().expect("comma exists").span,
                        );
                    }
                }
            }

            if self.peek_is(ImportPrescanTokenKind::Comma) {
                self.advance();
                continue;
            }

            if self.peek_is(ImportPrescanTokenKind::Semicolon) {
                let semicolon = self.advance().expect("semicolon exists");
                self.end = semicolon.span.end;
                return;
            }

            let Some(token) = self.peek().cloned() else {
                self.diagnostic(
                    ImportPrescanDiagnosticCode::MissingSemicolon,
                    "missing semicolon after import statement",
                    SourceSpan {
                        start: self.end,
                        end: self.end,
                    },
                );
                return;
            };

            let code = if recovered_any {
                ImportPrescanDiagnosticCode::MissingSemicolon
            } else {
                ImportPrescanDiagnosticCode::UnexpectedToken
            };
            self.diagnostic(code, "malformed import statement", token.span);
            if recovered_any && token.kind == ImportPrescanTokenKind::Word {
                return;
            }
            self.recover_to_import_stmt_end();
            return;
        }
    }

    fn parse_module_alias_decls(&mut self) -> Option<Vec<ImportStub>> {
        let path = self.parse_module_path()?;
        if self.peek_is(ImportPrescanTokenKind::Dot)
            && self.peek_n_is(1, ImportPrescanTokenKind::LBrace)
        {
            return Some(self.parse_branch_imports(path));
        }
        let mut span = SourceSpan {
            start: path.span.start,
            end: path.span.end,
        };
        let alias = if self.peek_is_word("as") {
            let as_span = self.advance().expect("as exists").span;
            match self.peek() {
                Some(token)
                    if token.kind == ImportPrescanTokenKind::Word
                        && is_identifier(&token.lexeme) =>
                {
                    let token = self.advance().expect("alias exists");
                    span.end = token.span.end;
                    Some(RawModuleAlias {
                        spelling: token.lexeme,
                        span: token.span,
                    })
                }
                Some(token) => {
                    self.diagnostic(
                        ImportPrescanDiagnosticCode::MissingAlias,
                        "missing module alias after `as`",
                        token.span,
                    );
                    span.end = as_span.end;
                    None
                }
                None => {
                    self.diagnostic(
                        ImportPrescanDiagnosticCode::MissingAlias,
                        "missing module alias after `as`",
                        as_span,
                    );
                    span.end = as_span.end;
                    None
                }
            }
        } else {
            None
        };

        Some(vec![ImportStub { path, alias, span }])
    }

    fn parse_branch_imports(&mut self, base: RawModulePath) -> Vec<ImportStub> {
        self.advance().expect("dot exists");
        self.advance().expect("left brace exists");

        let mut imports = Vec::new();
        loop {
            match self.parse_path_component() {
                Some(component) => {
                    let path = self.extend_base_path(&base, component);
                    imports.push(ImportStub {
                        span: path.span,
                        path,
                        alias: None,
                    });
                }
                None => {
                    let span = self.peek().map_or(base.span, |token| token.span);
                    self.diagnostic(
                        ImportPrescanDiagnosticCode::MissingModulePath,
                        "missing branch module path component",
                        span,
                    );
                    break;
                }
            }

            if self.peek_is(ImportPrescanTokenKind::Comma) {
                self.advance();
                continue;
            }
            break;
        }

        if self.peek_is(ImportPrescanTokenKind::RBrace) {
            self.advance();
        } else {
            let span = self.peek().map_or(base.span, |token| token.span);
            self.diagnostic(
                ImportPrescanDiagnosticCode::UnexpectedToken,
                "missing `}` after branch import list",
                span,
            );
        }

        imports
    }

    fn extend_base_path(
        &self,
        base: &RawModulePath,
        component: RawModulePathComponent,
    ) -> RawModulePath {
        let mut spelling = base.spelling.clone();
        spelling.push('.');
        spelling.push_str(&component.spelling);
        let mut components = base.components.clone();
        components.push(component);
        let end = components
            .last()
            .expect("just pushed branch component")
            .span
            .end;
        let branch_span = components
            .last()
            .expect("just pushed branch component")
            .span;
        RawModulePath {
            spelling,
            relative: base.relative,
            components,
            source_segments: vec![base.span, branch_span],
            span: SourceSpan {
                start: base.span.start,
                end,
            },
        }
    }

    fn parse_module_path(&mut self) -> Option<RawModulePath> {
        let mut relative = None;
        let mut parts = Vec::new();
        let mut spelling = String::new();
        let start = match self.peek()?.kind {
            ImportPrescanTokenKind::Dot => {
                let dot = self.advance().expect("dot exists");
                relative = Some(RawModuleRelativePrefix::Current);
                spelling.push('.');
                dot.span.start
            }
            ImportPrescanTokenKind::DotDot => {
                let dots = self.advance().expect("dotdot exists");
                relative = Some(RawModuleRelativePrefix::Parent);
                spelling.push_str("..");
                dots.span.start
            }
            _ => self.peek()?.span.start,
        };

        let first = self.parse_path_component();
        if first.is_none() {
            let span = self
                .peek()
                .map_or(SourceSpan { start, end: start }, |token| token.span);
            self.diagnostic(
                ImportPrescanDiagnosticCode::MissingModulePath,
                "missing module path in import declaration",
                span,
            );
            return None;
        }
        let first = first.expect("checked component");
        spelling.push_str(&first.spelling);
        let mut end = first.span.end;
        parts.push(first);

        while self.peek_is(ImportPrescanTokenKind::Dot) {
            if self.peek_n_is(1, ImportPrescanTokenKind::LBrace) {
                break;
            }
            let dot = self.advance().expect("dot exists");
            end = dot.span.end;
            spelling.push('.');
            match self.parse_path_component() {
                Some(component) => {
                    end = component.span.end;
                    spelling.push_str(&component.spelling);
                    parts.push(component);
                }
                None => {
                    let span =
                        self.peek()
                            .map_or(SourceSpan { start: end, end }, |token| SourceSpan {
                                start: end,
                                end: token.span.start,
                            });
                    self.diagnostic(
                        ImportPrescanDiagnosticCode::EmptyModulePathComponent,
                        "empty module path component",
                        span,
                    );
                    break;
                }
            }
        }

        Some(RawModulePath {
            spelling,
            relative,
            components: parts,
            source_segments: vec![SourceSpan { start, end }],
            span: SourceSpan { start, end },
        })
    }

    fn parse_path_component(&mut self) -> Option<RawModulePathComponent> {
        match self.peek() {
            Some(token)
                if token.kind == ImportPrescanTokenKind::Word && is_identifier(&token.lexeme) =>
            {
                let token = self.advance().expect("component exists");
                Some(RawModulePathComponent {
                    spelling: token.lexeme,
                    span: token.span,
                })
            }
            _ => None,
        }
    }

    fn recover_to_import_stmt_end(&mut self) {
        while let Some(token) = self.advance() {
            if token.kind == ImportPrescanTokenKind::Semicolon {
                self.end = token.span.end;
                return;
            }
        }
    }

    fn diagnostic(
        &mut self,
        code: ImportPrescanDiagnosticCode,
        message: impl Into<String>,
        span: SourceSpan,
    ) {
        self.diagnostics.push(ImportPrescanDiagnostic {
            code,
            message: message.into(),
            span,
        });
    }

    fn peek_is(&self, kind: ImportPrescanTokenKind) -> bool {
        self.peek().is_some_and(|token| token.kind == kind)
    }

    fn peek_n_is(&self, offset: usize, kind: ImportPrescanTokenKind) -> bool {
        self.tokens
            .get(self.cursor + offset)
            .is_some_and(|token| token.kind == kind)
    }

    fn peek(&self) -> Option<&ImportPrescanToken> {
        self.tokens.get(self.cursor)
    }

    fn advance(&mut self) -> Option<ImportPrescanToken> {
        let token = self.tokens.get(self.cursor).cloned();
        if let Some(token) = &token {
            self.cursor += 1;
            self.end = token.span.end;
        }
        token
    }
}

fn split_import_prescan_tokens(raw: &RawTokenStream) -> Vec<ImportPrescanToken> {
    let mut tokens = Vec::new();
    for raw_token in &raw.tokens {
        if raw_token.kind == RawTokenKind::Layout {
            continue;
        }
        match raw_token.kind {
            RawTokenKind::LexemeRun => split_lexeme_run_for_imports(raw_token, &mut tokens),
            RawTokenKind::NumeralLike | RawTokenKind::AnnotationMarker | RawTokenKind::Error => {
                tokens.push(ImportPrescanToken {
                    kind: ImportPrescanTokenKind::Other,
                    lexeme: raw_token.lexeme.clone(),
                    span: raw_token.span,
                });
            }
            RawTokenKind::Layout => {}
        }
    }
    tokens
}

fn split_lexeme_run_for_imports(raw_token: &RawToken, tokens: &mut Vec<ImportPrescanToken>) {
    let mut cursor = 0;
    let bytes = raw_token.lexeme.as_bytes();
    while cursor < bytes.len() {
        let start = cursor;
        let ch = raw_token.lexeme[cursor..]
            .chars()
            .next()
            .expect("cursor is inside string");

        if is_identifier_start(ch) {
            cursor += ch.len_utf8();
            while cursor < bytes.len() {
                let next = raw_token.lexeme[cursor..]
                    .chars()
                    .next()
                    .expect("cursor is inside string");
                if !is_identifier_continue(next) {
                    break;
                }
                cursor += next.len_utf8();
            }
            push_import_piece(
                raw_token,
                tokens,
                ImportPrescanTokenKind::Word,
                start,
                cursor,
            );
            continue;
        }

        match ch {
            '.' if raw_token.lexeme[cursor + 1..].starts_with('.') => {
                cursor += 2;
                push_import_piece(
                    raw_token,
                    tokens,
                    ImportPrescanTokenKind::DotDot,
                    start,
                    cursor,
                );
            }
            '.' => {
                cursor += 1;
                push_import_piece(
                    raw_token,
                    tokens,
                    ImportPrescanTokenKind::Dot,
                    start,
                    cursor,
                );
            }
            ',' => {
                cursor += 1;
                push_import_piece(
                    raw_token,
                    tokens,
                    ImportPrescanTokenKind::Comma,
                    start,
                    cursor,
                );
            }
            ';' => {
                cursor += 1;
                push_import_piece(
                    raw_token,
                    tokens,
                    ImportPrescanTokenKind::Semicolon,
                    start,
                    cursor,
                );
            }
            '*' => {
                cursor += 1;
                push_import_piece(
                    raw_token,
                    tokens,
                    ImportPrescanTokenKind::Star,
                    start,
                    cursor,
                );
            }
            '{' => {
                cursor += 1;
                push_import_piece(
                    raw_token,
                    tokens,
                    ImportPrescanTokenKind::LBrace,
                    start,
                    cursor,
                );
            }
            '}' => {
                cursor += 1;
                push_import_piece(
                    raw_token,
                    tokens,
                    ImportPrescanTokenKind::RBrace,
                    start,
                    cursor,
                );
            }
            _ => {
                cursor += ch.len_utf8();
                while cursor < bytes.len() {
                    let next = raw_token.lexeme[cursor..]
                        .chars()
                        .next()
                        .expect("cursor is inside string");
                    if is_identifier_start(next)
                        || matches!(next, '.' | ',' | ';' | '*' | '{' | '}')
                    {
                        break;
                    }
                    cursor += next.len_utf8();
                }
                push_import_piece(
                    raw_token,
                    tokens,
                    ImportPrescanTokenKind::Other,
                    start,
                    cursor,
                );
            }
        }
    }
}

fn push_import_piece(
    raw_token: &RawToken,
    tokens: &mut Vec<ImportPrescanToken>,
    kind: ImportPrescanTokenKind,
    start: usize,
    end: usize,
) {
    tokens.push(ImportPrescanToken {
        kind,
        lexeme: raw_token.lexeme[start..end].to_owned(),
        span: SourceSpan {
            start: raw_token.span.start + start,
            end: raw_token.span.start + end,
        },
    });
}
