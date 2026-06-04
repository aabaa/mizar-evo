# Module: lexing

> Canonical language: English. Japanese companion: [../ja/lexing.md](../ja/lexing.md).

Status: in progress (task 7 raw-scan / scope-skeleton wiring and task 8
context-sensitive disambiguation complete; task 9 recovery passthrough pending).

## Purpose

This module implements the frontend pipeline Step 4 (lexing / disambiguation).
The task-7/8 implementation drives the `mizar-lexer` raw scanner,
scope-skeleton pre-scan, and disambiguator to turn a `PreprocessedSource` into
a session-spanned parser-facing `TokenStream`. Task 9 extends the same entry
point with the remaining recovery passthrough coverage.

It owns the wiring and the span bridging; it does not own the longest-match
rules, the scope-skeleton construction, or the parser-lex context semantics
(those live in `mizar-lexer`). It makes no semantic decisions about whether an
identifier is defined or which overload applies.

See
[architecture/en/02.source_and_frontend.md](../../architecture/en/02.source_and_frontend.md)
"Step 4: Lex", "Lexing Is Raw First, Then Contextually Disambiguated",
"Dot Handling Is Split Across Lexer, Parser, and Resolver", and
"String Literals Are Fully Tokenized by the Lexer".

## Public API

```rust
pub struct TokenStream {
    pub source_id: SourceId,
    pub parser_context: ParserLexContext,
    pub tokens: Vec<Token>,
    pub scope_view: ScopeView,
    pub diagnostics: Vec<LexingDiagnostic>,
}

pub struct Token {
    pub kind: TokenKind,
    pub text: InternedText,
    pub span: SourceRange,
}

pub type InternedText = Arc<str>;

pub struct TokenizeRequest<'a> {
    pub preprocessed: &'a PreprocessedSource,
    pub environment: &'a ActiveLexicalEnvironment,
    pub parser_context: ParserLexContext,
}

impl<'a> TokenizeRequest<'a> {
    pub fn new(
        preprocessed: &'a PreprocessedSource,
        environment: &'a ActiveLexicalEnvironment,
        parser_context: ParserLexContext,
    ) -> Self;
}

pub struct LexingDiagnostic {
    pub kind: LexingDiagnosticKind,
    pub message: InternedText,
    pub primary: SourceRange,
    pub secondary: Vec<SourceAnchor>,
    pub payload: LexingDiagnosticPayload,
}

pub enum LexingDiagnosticKind {
    RawScan,
    ScopeSkeleton(ScopeSkeletonDiagnosticCode),
    Lexer(LexDiagnosticCode),
}

pub enum LexingDiagnosticPayload {
    None,
    NoValidTokenCandidate {
        rejected_lexeme: InternedText,
        recovery: LexRecoveryHint,
    },
    ParserContextRejectedCandidate {
        mode: ParserLexMode,
        rejected_lexeme: InternedText,
        candidates: Vec<LexingRejectedTokenCandidate>,
        recovery: LexRecoveryHint,
    },
    MalformedStringLiteral {
        opening_quote: char,
        reason: MalformedStringLiteralReason,
        recovery: LexRecoveryHint,
    },
    UnsupportedRawToken {
        raw_kind: RawTokenKind,
        raw_lexeme: InternedText,
        recovery: LexRecoveryHint,
    },
    UnsupportedLexerPayload,
}

pub struct LexingRejectedTokenCandidate {
    pub kind: TokenKind,
    pub text: InternedText,
    pub span: SourceRange,
    pub secondary: Vec<SourceAnchor>,
}

pub struct ScopeView {
    pub source_id: SourceId,
    pub frames: Vec<ScopeFrame>,
    pub blocks: Vec<ScopeBlock>,
    pub statements: Vec<ScopeStatement>,
}

impl TokenStream {
    pub fn tokens(&self) -> &[Token];
    pub fn diagnostics(&self) -> &[LexingDiagnostic];
    pub fn scope_view(&self) -> &ScopeView;
    pub fn into_parts(self) -> (Vec<Token>, ScopeView, Vec<LexingDiagnostic>);
}

impl ScopeView {
    pub fn empty(source_id: SourceId) -> Self;
    pub fn binding_overrides_symbol(&self, spelling: &str, position: usize) -> bool;
}

pub struct ScopeFrame {
    pub range: SourceRange,
    pub bindings: Vec<ScopedBinding>,
}

pub struct ScopedBinding {
    pub spelling: InternedText,
    pub introduced_at: SourceRange,
    pub kind: BindingShapeKind,
}

pub struct ScopeBlock {
    pub kind: LexicalBlockKind,
    pub range: SourceRange,
}

pub struct ScopeStatement {
    pub kind: LexicalStatementKind,
    pub range: SourceRange,
}

pub fn tokenize(
    request: TokenizeRequest<'_>,
    bridge: &SpanBridge,
) -> Result<TokenStream, SpanBridgeError>;
```

`TokenKind`, `ParserLexContext`, `ParserLexMode`, `LexRecoveryHint`,
`MalformedStringLiteralReason`, `RawTokenKind`, `BindingShapeKind`,
`LexicalBlockKind`, `LexicalStatementKind`, `ScopeSkeletonDiagnosticCode`, and
`LexDiagnosticCode` are re-exported from `mizar-lexer`. `InternedText` is a
frontend-local `Arc<str>` spelling handle for the first implementation; no
global interner is required, and lexer `String` lexemes are converted with
`Arc::<str>::from`. Raw lexer diagnostic structs are consumed as inputs only:
the frontend converts them immediately into `LexingDiagnostic` so public
diagnostics carry session ranges and never expose raw lexer byte spans.
Structured lexer payload variants carry copied non-span data, and rejected
candidates are represented as frontend-owned `LexingRejectedTokenCandidate`
values with mapped session spans and secondary anchors. Future lexer payload
variants that this frontend has not learned to map are represented explicitly as
`UnsupportedLexerPayload`, not silently collapsed to `None`.

`parser_context` is carried into `mizar_lexer::disambiguate` so the same
frontend request can produce general, namespace-path, string-required, or
recovery-mode token streams. `environment` provides the active user-symbol
index. Imported symbols participate in token classification but are not mixed
into the public lexical `ScopeView`.

## Dependencies

- Internal: `preprocess` (lexical text + map), `lexical_env`
  (`ActiveLexicalEnvironment`), `span_bridge` (lexical-byte → `SourceRange`),
  `parsing` (consumes `TokenStream`).
- External: `mizar-lexer` (`scan_raw`, `build_scope_skeleton`, `disambiguate`,
  `TokenKind`, `ParserLexContext`, `LexDiagnostic`, `LexDiagnosticCode`,
  structured lexer payload enums, `ScopeSkeletonDiagnostic`,
  `ScopeSkeletonDiagnosticCode`, and scope block / statement / binding shape enums),
  `mizar-session` (`SourceId`, `SourceRange`, `SourceAnchor`).

This module is consumed by parsing and, through the diagnostics, by the
orchestration merge.

## Data Structures

### Token Stream

`TokenStream` is the source-faithful token sequence for one file under the
parser lexing context used for the run. Successful tokenization surfaces final
parser-facing `UserSymbol`, `ReservedWord`, `ReservedSymbol`, `Identifier`,
`Numeral`, and `StringLiteral` classifications. Strict raw-scan failure emits
one coarse `TokenKind::ErrorRecovery`. Each `Token` preserves its original
spelling (`text`) and a session `SourceRange`.

`LexingDiagnostic` is the mapped frontend diagnostic payload for Step 4. It
represents raw-scan failures, scope-skeleton diagnostics, and lexer
disambiguator diagnostics. It always carries a session-coordinate primary range
and secondary anchors for orchestration. Its secondary entries are
`SourceAnchor`s so composite or degraded preprocess mappings can preserve point,
generated, and adjacent comment anchors. It stores the mapped code/message and
frontend-owned structured payloads, not the raw lexer diagnostic object, because
those raw objects contain lexer byte spans.

### Scope Lex View

The scope-skeleton pre-scan (`build_scope_skeleton`) produces a read-only
`ScopeLexView`, used by the disambiguator for scoped identifier override rules.
The frontend first builds a raw skeleton for an initial disambiguation pass, then
rebuilds a contextual skeleton from the final token shapes so strings and
non-identifier user symbols are inert for public scope diagnostics. The frontend
maps that contextual skeleton into a public session-spanned `ScopeView` for
downstream inspection and diagnostics; it does not build scopes itself, and the
view records lexical shape only — never resolved bindings.

## Algorithm / Logic

### Tokenize a preprocessed source

1. Raw-scan the lexical text (`scan_raw`) into raw units with preserved spans.
   If strict raw scanning fails, emit one coarse `ErrorRecovery` token and one
   mapped `LexingDiagnosticKind::RawScan` diagnostic covering the whole lexical
   text (or the source-start zero-length range for empty text), then skip
   scope-skeleton construction and disambiguation for that run. The current
   `mizar_lexer::LexError` has no span or partial-token payload, so finer recovery
   is tracked as a follow-up contract.
2. Build an initial `ScopeSkeleton` / `ScopeLexView` from the raw tokens and run
   `disambiguate` once with the raw token stream, active lexical environment,
   and current `ParserLexContext`.
3. Rebuild the scope skeleton from the first final token shapes, treating
   `StringLiteral`, `ErrorRecovery`, numerals, and non-identifier user symbols as
   scope-inert. Run `disambiguate` again with that contextual skeleton so scoped
   identifier overrides are available without treating string contents or
   user-symbol spellings as scope syntax. Build the public scope skeleton from
   the final token shapes.
4. The disambiguator applies longest match against, in order: active user
   symbols, reserved special symbols, reserved words, identifier/numeral rules,
   and parser expectation / scoped override where the language requires it.
5. Map every final lexer token through `span_bridge.lexical_span` into a
   session-spanned frontend `Token`. Map the scope-skeleton frames, blocks,
   statements, and binding shapes into the public `ScopeView`.
6. Recognize `StringLiteral` tokens only when the current parser lexing context
   is `StringRequired`; position-sensitive context for only selected byte spans
   is deferred until the parser-assisted lexing contract lands.
7. Collect lexer diagnostics as `LexingDiagnostic`s, copy non-span payload data,
   and map nested rejected-candidate spans into frontend payload structures.

Compound reserved tokens (`.{`, `.*`, `.=`, `...`) are recognized by the lexer;
selector/namespace roles for `.` are left to the parser and resolver. The lexer
never decides definedness, applicability, or overload selection.

## Error Handling

The lexing wrapper preserves recoverable raw-scan, scope-skeleton, and current
disambiguator problems:

- strict raw-scan hard failure emits one coarse `TokenKind::ErrorRecovery` with
  the best available source range plus a `RawScan` diagnostic;
- `ScopeSkeletonDiagnostic`s are mapped into frontend diagnostics without
  storing raw span-bearing diagnostic structs in `TokenStream`;
- current `LexDiagnostic`s are mapped into `LexingDiagnosticKind::Lexer` with
  frontend-owned payloads and mapped nested rejected-candidate spans.

For user-recoverable lexical input problems, `tokenize` returns
`Ok(TokenStream)`: strict raw-scan failures degrade to a coarse recovery token
and mapped diagnostic, while scope-skeleton and disambiguator problems are
mapped without dropping recoverable tokens. It returns `Err(SpanBridgeError)`
only when a token, scope shape, or diagnostic span cannot be mapped through the
registered bridge. Task 9 keeps the same boundary and adds the remaining
recovery passthrough coverage and follow-up cases.

## Tests

Implemented task-7/8 scenarios:

- raw scan and disambiguation preserve final token spans, including spans mapped
  through preprocess mappings such as removed comments;
- the public `ScopeView` reflects lexical block / statement shape and local
  binding shapes without resolved or imported bindings;
- scope-skeleton diagnostics are preserved as mapped `LexingDiagnostic`s;
- every emitted token's `span` is a valid `SourceRange` for `source_id`.
- active user symbols and scoped identifier overrides affect final token
  classification;
- compound reserved tokens remain single final tokens;
- uniform `StringRequired` context emits `StringLiteral`, while general context
  reports mapped lexer diagnostics for rejected string candidates;
- lexer diagnostic payloads preserve non-span data and mapped nested candidate
  spans.

Task-9 scenarios add the remaining recovery passthrough coverage for malformed
tokens, future invalid-numeral diagnostics, unsupported raw-token cases, and
scope diagnostics after disambiguation.

## Constraints and Assumptions

- This module orchestrates the lexer; it does not own longest-match, scope
  skeleton, or disambiguation rules.
- The lexer recognizes compound reserved tokens and active user symbols but not
  semantic selector/namespace roles.
- `StringLiteral` tokens are emitted only when the parser lexing context requires
  strings. Grammar-position-specific string recognition is a parser integration
  task, not part of the source-to-token foundation.
- When token streams are cached, the cache key is `PreprocessedSource.lexical_hash`,
  the active lexical environment fingerprint, and a stable encoding of the
  `ParserLexContext` / parser-assisted lexing plan used for the run.
- All token spans are session `SourceRange` values produced through `span_bridge`.
