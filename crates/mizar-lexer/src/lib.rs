use std::error::Error;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    pub kind: TokenKind,
    pub lexeme: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawTokenStream {
    pub tokens: Vec<RawToken>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawToken {
    pub kind: RawTokenKind,
    pub lexeme: String,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RawTokenKind {
    LexemeRun,
    NumeralLike,
    AnnotationMarker,
    Layout,
    Error,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SourceSpan {
    pub start: usize,
    pub end: usize,
}

pub type SourcePos = usize;
pub type SourceRange = SourceSpan;

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
pub enum ImportPrescanDiagnosticCode {
    MissingModulePath,
    EmptyModulePathComponent,
    MissingAlias,
    MissingSemicolon,
    UnexpectedToken,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenKind {
    Identifier,
    ReservedWord,
    ReservedSymbol,
    Numeral,
    LexemeRun,
    UserSymbol,
    StringLiteral,
    ErrorRecovery,
}

pub const RESERVED_WORDS: &[&str] = &[
    "algorithm",
    "and",
    "antonym",
    "as",
    "assert",
    "assume",
    "assumed",
    "asymmetry",
    "attr",
    "be",
    "being",
    "break",
    "by",
    "case",
    "cases",
    "claim",
    "cluster",
    "coherence",
    "commutativity",
    "compatibility",
    "computation",
    "conditional",
    "connectedness",
    "const",
    "consider",
    "consistency",
    "continue",
    "contradiction",
    "decreasing",
    "deffunc",
    "definition",
    "defpred",
    "do",
    "does",
    "downto",
    "else",
    "end",
    "ensures",
    "equals",
    "ex",
    "exhaustive",
    "existence",
    "export",
    "extends",
    "field",
    "for",
    "from",
    "func",
    "ghost",
    "given",
    "hence",
    "hereby",
    "holds",
    "idempotence",
    "if",
    "iff",
    "implies",
    "import",
    "in",
    "infix_operator",
    "inherit",
    "invariant",
    "involutiveness",
    "irreflexivity",
    "is",
    "it",
    "left",
    "lemma",
    "let",
    "match",
    "means",
    "mode",
    "nest",
    "non",
    "none",
    "not",
    "now",
    "object",
    "of",
    "open",
    "or",
    "otherwise",
    "over",
    "per",
    "postfix_operator",
    "pred",
    "prefix_operator",
    "private",
    "processed",
    "projectivity",
    "proof",
    "property",
    "public",
    "qua",
    "reconsider",
    "reduce",
    "reducibility",
    "redefine",
    "reflexivity",
    "registration",
    "requires",
    "reserve",
    "return",
    "right",
    "set",
    "sethood",
    "snapshot",
    "st",
    "struct",
    "such",
    "suppose",
    "symmetry",
    "synonym",
    "take",
    "terminating",
    "that",
    "the",
    "then",
    "theorem",
    "thesis",
    "thus",
    "to",
    "transitivity",
    "type",
    "uniqueness",
    "var",
    "where",
    "while",
    "with",
];

pub const RESERVED_SYMBOLS: &[&str] = &[
    "...", ":=", ".{", "<>", "->", ".=", ".*", "@[", ",", ".", ";", ":", "(", ")", "[", "]", "{",
    "}", "=", "&",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LexError {
    message: String,
}

impl LexError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for LexError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl Error for LexError {}

pub fn scan_raw(input: &str) -> Result<RawTokenStream, LexError> {
    let mut tokens = Vec::new();
    let mut chars = input.char_indices().peekable();

    while let Some((start, ch)) = chars.peek().copied() {
        if is_layout(ch) {
            chars.next();
            let mut end = start + ch.len_utf8();

            while let Some((next_start, next_ch)) = chars.peek().copied() {
                if !is_layout(next_ch) {
                    break;
                }

                chars.next();
                end = next_start + next_ch.len_utf8();
            }

            tokens.push(raw_token(input, RawTokenKind::Layout, start, end));
            continue;
        }

        if ch == '@' {
            chars.next();

            let end = match chars.peek().copied() {
                Some((next_start, '[')) => {
                    chars.next();
                    next_start + '['.len_utf8()
                }
                Some((next_start, next_ch)) if is_identifier_start(next_ch) => {
                    chars.next();
                    let mut end = next_start + next_ch.len_utf8();

                    while let Some((body_start, body_ch)) = chars.peek().copied() {
                        if !is_identifier_continue(body_ch) {
                            break;
                        }

                        chars.next();
                        end = body_start + body_ch.len_utf8();
                    }

                    end
                }
                _ => {
                    return Err(LexError::new(format!(
                        "unsupported annotation marker at byte {start}"
                    )));
                }
            };

            tokens.push(raw_token(input, RawTokenKind::AnnotationMarker, start, end));
            continue;
        }

        if is_lexeme_run_char(ch) {
            chars.next();
            let mut end = start + ch.len_utf8();

            while let Some((next_start, next_ch)) = chars.peek().copied() {
                if !is_lexeme_run_char(next_ch) {
                    break;
                }

                chars.next();
                end = next_start + next_ch.len_utf8();
            }

            let kind = if input[start..end].chars().all(|ch| ch.is_ascii_digit()) {
                RawTokenKind::NumeralLike
            } else {
                RawTokenKind::LexemeRun
            };
            tokens.push(raw_token(input, kind, start, end));
            continue;
        }

        return Err(LexError::new(format!(
            "unsupported raw lexer input at byte {start}: {ch:?}"
        )));
    }

    Ok(RawTokenStream { tokens })
}

pub fn lex(input: &str) -> Result<Vec<Token>, LexError> {
    let raw = scan_raw(input)?;
    disambiguate_reserved_shell(&raw)
}

pub fn disambiguate_reserved_shell(raw: &RawTokenStream) -> Result<Vec<Token>, LexError> {
    let mut tokens = Vec::new();

    for raw_token in &raw.tokens {
        match raw_token.kind {
            RawTokenKind::Layout => {}
            RawTokenKind::NumeralLike => {
                tokens.push(Token {
                    kind: TokenKind::Numeral,
                    lexeme: raw_token.lexeme.clone(),
                });
            }
            RawTokenKind::AnnotationMarker if raw_token.lexeme == "@[" => {
                tokens.push(Token {
                    kind: TokenKind::ReservedSymbol,
                    lexeme: raw_token.lexeme.clone(),
                });
            }
            RawTokenKind::LexemeRun => tokens.push(classify_lexeme_run_shell(raw_token)),
            _ => {
                return Err(LexError::new(format!(
                    "unsupported lexer token at byte {}: {:?}",
                    raw_token.span.start, raw_token.lexeme
                )));
            }
        }
    }

    Ok(tokens)
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

fn raw_token(input: &str, kind: RawTokenKind, start: usize, end: usize) -> RawToken {
    RawToken {
        kind,
        lexeme: input[start..end].to_owned(),
        span: SourceSpan { start, end },
    }
}

pub fn is_layout(ch: char) -> bool {
    matches!(ch, ' ' | '\t' | '\n')
}

pub fn is_lexeme_run_char(ch: char) -> bool {
    ch.is_ascii_graphic() && ch != '@'
}

pub fn is_identifier(value: &str) -> bool {
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    is_identifier_start(first) && chars.all(is_identifier_continue) && !is_reserved_word(value)
}

pub fn is_identifier_start(ch: char) -> bool {
    ch.is_ascii_alphabetic() || ch == '_'
}

pub fn is_identifier_continue(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || ch == '_' || ch == '\''
}

pub fn is_numeral(value: &str) -> bool {
    !value.is_empty() && value.chars().all(|ch| ch.is_ascii_digit())
}

pub fn is_reserved_word(value: &str) -> bool {
    RESERVED_WORDS.contains(&value)
}

pub fn is_reserved_symbol(value: &str) -> bool {
    RESERVED_SYMBOLS.contains(&value)
}

pub fn is_user_symbol_spelling(value: &str) -> bool {
    !value.is_empty() && value.chars().all(|ch| ch.is_ascii_graphic() && ch != '@')
}

pub fn is_string_literal_spelling(value: &str) -> bool {
    let Some(quote) = value.chars().next() else {
        return false;
    };
    if quote != '"' && quote != '\'' {
        return false;
    }
    let mut chars = value[quote.len_utf8()..].chars();
    let mut escaped = false;
    while let Some(ch) = chars.next() {
        if escaped {
            if !matches!(ch, '"' | '\'' | '\\') {
                return false;
            }
            escaped = false;
            continue;
        }
        if ch == '\\' {
            escaped = true;
            continue;
        }
        if ch == quote {
            return chars.next().is_none();
        }
    }
    false
}

pub fn longest_reserved_symbol_prefix(value: &str) -> Option<&'static str> {
    RESERVED_SYMBOLS
        .iter()
        .copied()
        .filter(|symbol| value.starts_with(symbol))
        .max_by_key(|symbol| symbol.len())
}

fn classify_lexeme_run_shell(raw_token: &RawToken) -> Token {
    let kind = if is_reserved_symbol(&raw_token.lexeme) {
        TokenKind::ReservedSymbol
    } else if is_reserved_word(&raw_token.lexeme) {
        TokenKind::ReservedWord
    } else if is_identifier(&raw_token.lexeme) {
        TokenKind::Identifier
    } else {
        TokenKind::LexemeRun
    };

    Token {
        kind,
        lexeme: raw_token.lexeme.clone(),
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

#[cfg(test)]
mod tests {
    use super::{
        ImportPrescanDiagnosticCode, RESERVED_SYMBOLS, RESERVED_WORDS, RawModuleRelativePrefix,
        RawToken, RawTokenKind, SourceSpan, Token, TokenKind, is_identifier, is_layout, is_numeral,
        is_reserved_symbol, is_reserved_word, is_string_literal_spelling, is_user_symbol_spelling,
        lex, longest_reserved_symbol_prefix, scan_import_prelude, scan_raw,
    };

    #[test]
    fn lexes_alpha_as_identifier() {
        let tokens = lex("alpha").expect("alpha should lex as an identifier");

        assert_eq!(
            tokens,
            vec![Token {
                kind: TokenKind::Identifier,
                lexeme: "alpha".to_owned(),
            }]
        );
    }

    #[test]
    fn lexes_identifier_body_characters() {
        let tokens = lex("_alpha1'").expect("identifier body characters should be supported");

        assert_eq!(
            tokens,
            vec![Token {
                kind: TokenKind::Identifier,
                lexeme: "_alpha1'".to_owned(),
            }]
        );
    }

    #[test]
    fn lexes_whitespace_separated_identifiers() {
        let tokens = lex("alpha beta\tgamma\n_delta").expect("identifiers should lex");

        assert_eq!(
            tokens,
            vec![
                Token {
                    kind: TokenKind::Identifier,
                    lexeme: "alpha".to_owned(),
                },
                Token {
                    kind: TokenKind::Identifier,
                    lexeme: "beta".to_owned(),
                },
                Token {
                    kind: TokenKind::Identifier,
                    lexeme: "gamma".to_owned(),
                },
                Token {
                    kind: TokenKind::Identifier,
                    lexeme: "_delta".to_owned(),
                },
            ]
        );
    }

    #[test]
    fn keeps_digit_leading_symbol_shapes_unsplit() {
        let tokens = lex("1alpha").expect("digit-leading symbol shape should lex");

        assert_eq!(
            tokens,
            vec![Token {
                kind: TokenKind::LexemeRun,
                lexeme: "1alpha".to_owned(),
            }]
        );
    }

    #[test]
    fn keeps_symbol_shaped_raw_runs_unsplit() {
        let tokens = lex("alpha:=beta").expect("symbol-shaped raw run should lex");

        assert_eq!(
            tokens,
            vec![Token {
                kind: TokenKind::LexemeRun,
                lexeme: "alpha:=beta".to_owned(),
            }]
        );
    }

    #[test]
    fn recognizes_reserved_word_table_entries() {
        for word in RESERVED_WORDS {
            assert!(is_reserved_word(word), "{word} should be reserved");
            assert!(!is_identifier(word), "{word} should not be an identifier");
            assert_eq!(
                lex(word).expect("reserved word should lex"),
                vec![Token {
                    kind: TokenKind::ReservedWord,
                    lexeme: (*word).to_owned(),
                }]
            );
        }
    }

    #[test]
    fn recognizes_reserved_symbol_table_entries() {
        for symbol in RESERVED_SYMBOLS {
            assert!(is_reserved_symbol(symbol), "{symbol} should be reserved");
            assert_eq!(
                lex(symbol).expect("reserved symbol should lex"),
                vec![Token {
                    kind: TokenKind::ReservedSymbol,
                    lexeme: (*symbol).to_owned(),
                }]
            );
        }
    }

    #[test]
    fn reserved_words_are_case_sensitive() {
        assert_eq!(
            lex("Theorem").expect("case-distinct spelling should lex"),
            vec![Token {
                kind: TokenKind::Identifier,
                lexeme: "Theorem".to_owned(),
            }]
        );
    }

    #[test]
    fn helper_recognizes_numerals() {
        assert!(is_numeral("42"));
        assert!(!is_numeral(""));
        assert!(!is_numeral("42abc"));
    }

    #[test]
    fn helpers_recognize_layout_symbol_shapes_and_string_shells() {
        assert!(is_layout(' '));
        assert!(is_layout('\t'));
        assert!(is_layout('\n'));
        assert!(!is_layout('\r'));

        assert!(is_user_symbol_spelling("*+"));
        assert!(is_user_symbol_spelling("succ"));
        assert!(!is_user_symbol_spelling("@latex"));

        assert!(is_string_literal_spelling("\"say \\\"hi\\\"\""));
        assert!(is_string_literal_spelling("'say \"hi\"'"));
        assert!(!is_string_literal_spelling("\"unterminated"));

        assert_eq!(longest_reserved_symbol_prefix("..."), Some("..."));
        assert_eq!(longest_reserved_symbol_prefix(".{"), Some(".{"));
    }

    #[test]
    fn rejects_non_spec_layout_characters() {
        assert!(lex("alpha\rbeta").is_err());
    }

    #[test]
    fn scans_empty_raw_stream() {
        let raw = scan_raw("").expect("empty input should scan");

        assert!(raw.tokens.is_empty());
    }

    #[test]
    fn scans_raw_spans_for_layout_and_runs() {
        let raw = scan_raw("alpha \t\n+").expect("raw input should scan");

        assert_eq!(
            raw.tokens,
            vec![
                RawToken {
                    kind: RawTokenKind::LexemeRun,
                    lexeme: "alpha".to_owned(),
                    span: SourceSpan { start: 0, end: 5 },
                },
                RawToken {
                    kind: RawTokenKind::Layout,
                    lexeme: " \t\n".to_owned(),
                    span: SourceSpan { start: 5, end: 8 },
                },
                RawToken {
                    kind: RawTokenKind::LexemeRun,
                    lexeme: "+".to_owned(),
                    span: SourceSpan { start: 8, end: 9 },
                },
            ]
        );
    }

    #[test]
    fn keeps_digit_leading_mixed_runs_coarse_for_later_disambiguation() {
        let raw = scan_raw("42abc 0*+x").expect("mixed raw input should scan");

        assert_eq!(
            raw.tokens,
            vec![
                RawToken {
                    kind: RawTokenKind::LexemeRun,
                    lexeme: "42abc".to_owned(),
                    span: SourceSpan { start: 0, end: 5 },
                },
                RawToken {
                    kind: RawTokenKind::Layout,
                    lexeme: " ".to_owned(),
                    span: SourceSpan { start: 5, end: 6 },
                },
                RawToken {
                    kind: RawTokenKind::LexemeRun,
                    lexeme: "0*+x".to_owned(),
                    span: SourceSpan { start: 6, end: 10 },
                },
            ]
        );
    }

    #[test]
    fn scans_annotation_markers_without_import_or_parser_context() {
        let raw = scan_raw("@latex @[").expect("annotation marker shapes should scan");

        assert_eq!(
            raw.tokens,
            vec![
                RawToken {
                    kind: RawTokenKind::AnnotationMarker,
                    lexeme: "@latex".to_owned(),
                    span: SourceSpan { start: 0, end: 6 },
                },
                RawToken {
                    kind: RawTokenKind::Layout,
                    lexeme: " ".to_owned(),
                    span: SourceSpan { start: 6, end: 7 },
                },
                RawToken {
                    kind: RawTokenKind::AnnotationMarker,
                    lexeme: "@[".to_owned(),
                    span: SourceSpan { start: 7, end: 9 },
                },
            ]
        );
    }

    #[test]
    fn reports_stable_raw_diagnostics_for_malformed_characters() {
        let error = scan_raw("alpha\rbeta").expect_err("CR is outside lexer layout");

        assert_eq!(
            "unsupported raw lexer input at byte 5: '\\r'",
            error.to_string()
        );
    }

    #[test]
    fn reports_stable_raw_diagnostics_for_malformed_annotation_markers() {
        let error = scan_raw("@-").expect_err("bare annotation marker should be rejected");

        assert_eq!("unsupported annotation marker at byte 0", error.to_string());
    }

    #[test]
    fn scans_empty_import_prelude() {
        let raw = scan_raw("definition\nend;").expect("source should raw scan");
        let prelude = scan_import_prelude(&raw);

        assert!(prelude.imports.is_empty());
        assert_eq!(prelude.end, 0);
        assert!(prelude.diagnostics.is_empty());
    }

    #[test]
    fn scans_imports_aliases_and_relative_paths_from_raw_runs() {
        let raw = scan_raw("import std.algebra.group, ..common as C, .utils;")
            .expect("source should raw scan");
        let prelude = scan_import_prelude(&raw);

        let paths = prelude
            .imports
            .iter()
            .map(|import| import.path.spelling.as_str())
            .collect::<Vec<_>>();
        assert_eq!(paths, vec!["std.algebra.group", "..common", ".utils"]);
        assert_eq!(prelude.imports[1].alias.as_ref().unwrap().spelling, "C");
        assert_eq!(
            prelude.imports[1].path.relative,
            Some(RawModuleRelativePrefix::Parent)
        );
        assert_eq!(
            prelude.imports[2].path.relative,
            Some(RawModuleRelativePrefix::Current)
        );
        assert_eq!(
            prelude.end,
            "import std.algebra.group, ..common as C, .utils;".len()
        );
        assert!(prelude.diagnostics.is_empty());
    }

    #[test]
    fn scans_contiguous_import_statements() {
        let source = "\
import std.algebra.group;
import std.topology.metric_space as Metric;
import pkg.mathcomp_mizar.algebra.ring;";
        let raw = scan_raw(source).expect("source should raw scan");
        let prelude = scan_import_prelude(&raw);

        let paths = prelude
            .imports
            .iter()
            .map(|import| import.path.spelling.as_str())
            .collect::<Vec<_>>();
        assert_eq!(
            paths,
            vec![
                "std.algebra.group",
                "std.topology.metric_space",
                "pkg.mathcomp_mizar.algebra.ring"
            ]
        );
        assert_eq!(
            prelude.imports[1].alias.as_ref().unwrap().spelling,
            "Metric"
        );
        assert_eq!(prelude.end, source.len());
        assert!(prelude.diagnostics.is_empty());
    }

    #[test]
    fn scans_branch_import_paths() {
        let source = "import algebra.linear.{eigen_value, jordan};";
        let raw = scan_raw(source).expect("source should raw scan");
        let prelude = scan_import_prelude(&raw);

        let paths = prelude
            .imports
            .iter()
            .map(|import| import.path.spelling.as_str())
            .collect::<Vec<_>>();
        assert_eq!(
            paths,
            vec!["algebra.linear.eigen_value", "algebra.linear.jordan"]
        );
        assert_eq!(
            prelude.imports[1].path.source_segments,
            vec![
                SourceSpan { start: 7, end: 21 },
                SourceSpan { start: 36, end: 42 },
            ]
        );
        assert_eq!(prelude.end, source.len());
        assert!(prelude.diagnostics.is_empty());
    }

    #[test]
    fn stops_at_first_non_import_top_level_text() {
        let raw = scan_raw("import std.core;\ndefinition\nimport dev.late;")
            .expect("source should raw scan");
        let prelude = scan_import_prelude(&raw);

        assert_eq!(prelude.imports.len(), 1);
        assert_eq!(prelude.imports[0].path.spelling, "std.core");
        assert_eq!(prelude.end, "import std.core;".len());
        assert!(prelude.diagnostics.is_empty());
    }

    #[test]
    fn recovers_malformed_imports_with_diagnostics() {
        let raw = scan_raw("import std., pkg.math as ;").expect("source should raw scan");
        let prelude = scan_import_prelude(&raw);

        let paths = prelude
            .imports
            .iter()
            .map(|import| import.path.spelling.as_str())
            .collect::<Vec<_>>();
        assert_eq!(paths, vec!["std.", "pkg.math"]);
        assert_eq!(
            prelude
                .diagnostics
                .iter()
                .map(|diagnostic| diagnostic.code)
                .collect::<Vec<_>>(),
            vec![
                ImportPrescanDiagnosticCode::EmptyModulePathComponent,
                ImportPrescanDiagnosticCode::MissingAlias,
            ]
        );
    }

    #[test]
    fn comma_separated_import_stub_spans_cover_each_declaration() {
        let raw = scan_raw("import std.core, pkg.math;").expect("source should raw scan");
        let prelude = scan_import_prelude(&raw);

        assert_eq!(prelude.imports[0].span, SourceSpan { start: 7, end: 15 });
        assert_eq!(prelude.imports[1].span, SourceSpan { start: 17, end: 25 });
    }

    #[test]
    fn missing_semicolon_does_not_consume_top_level_terminator() {
        let raw = scan_raw("import std.core\ndefinition\nend;").expect("source should raw scan");
        let prelude = scan_import_prelude(&raw);

        assert_eq!(prelude.imports.len(), 1);
        assert_eq!(prelude.end, "import std.core".len());
        assert_eq!(
            prelude.diagnostics[0].code,
            ImportPrescanDiagnosticCode::MissingSemicolon
        );
    }
}
