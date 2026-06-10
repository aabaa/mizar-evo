# Module: lexing

> Canonical language: English. Japanese companion: [../ja/lexing.md](../ja/lexing.md).

Status: complete for tasks 7-9, task 20 parser-assisted lexing contract, and
task 22 precise raw-scan recovery.

## Purpose

This module implements the frontend pipeline Step 4 (lexing / disambiguation).
The task-7/8/9 implementation drives the `mizar-lexer` raw scanner,
scope-skeleton pre-scan, and disambiguator to turn a `PreprocessedSource` into
a session-spanned parser-facing `TokenStream`, preserving recoverable lexer
diagnostics and `ErrorRecovery` tokens along the same entry point.

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
    pub parser_lexing_plan: ParserLexingPlan,
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
    pub parser_lexing_plan: ParserLexingPlan,
}

impl<'a> TokenizeRequest<'a> {
    pub fn new(
        preprocessed: &'a PreprocessedSource,
        environment: &'a ActiveLexicalEnvironment,
        parser_context: ParserLexContext,
    ) -> Self;

    pub fn with_plan(
        preprocessed: &'a PreprocessedSource,
        environment: &'a ActiveLexicalEnvironment,
        parser_lexing_plan: ParserLexingPlan,
    ) -> Self;
}

pub struct ParserLexingPlan {
    pub default_context: ParserLexContext,
    pub contexts: Vec<ParserLexingPlanContext>,
}

impl ParserLexingPlan {
    pub fn uniform(default_context: ParserLexContext) -> Self;
    pub fn new(
        default_context: ParserLexContext,
        contexts: Vec<ParserLexingPlanContext>,
    ) -> Self;
    pub fn for_lexical_text(lexical_text: &str) -> Self;
    pub fn context_at(&self, offset: usize) -> ParserLexContext;
    pub fn is_uniform(&self) -> bool;
}

pub struct ParserLexingPlanContext {
    pub range: LexicalByteRange,
    pub context: ParserLexContext,
}

impl ParserLexingPlanContext {
    pub fn new(range: LexicalByteRange, context: ParserLexContext) -> Self;
}

pub struct LexicalByteRange {
    pub start: usize,
    pub end: usize,
}

impl LexicalByteRange {
    pub fn new(start: usize, end: usize) -> Self;
    pub fn contains(self, offset: usize) -> bool;
}

pub struct LexingDiagnostic {
    pub kind: LexingDiagnosticKind,
    pub message: InternedText,
    pub primary: SourceRange,
    pub secondary: Vec<SourceAnchor>,
    pub payload: LexingDiagnosticPayload,
}

#[non_exhaustive]
pub enum LexingDiagnosticKind {
    RawScan,
    ScopeSkeleton(ScopeSkeletonDiagnosticCode),
    Lexer(LexDiagnosticCode),
}

#[non_exhaustive]
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

`LexingDiagnosticKind` and `LexingDiagnosticPayload` are `#[non_exhaustive]` for
downstream crates so future lexer/parser-assisted recovery surfaces can be
added without breaking external matches. Matches inside `mizar-frontend` remain
exhaustive.

`parser_lexing_plan` is the task-20 narrow parser-assisted lexing contract. It
contains a default `ParserLexContext` plus lexical-byte ranges whose context
differs from the default. The frontend precomputes this plan before
tokenization, then calls the lexer with only the `ParserLexContext` for the raw
unit being disambiguated. The lexer never receives arbitrary parser state and
the parser does not interleave with lexing. `TokenizeRequest::new` remains a
uniform-context convenience wrapper; source-to-token coordinator paths use
`TokenizeRequest::with_plan`. `environment` provides the active user-symbol
index. Imported symbols participate in token classification but are not mixed
into the public lexical `ScopeView`.
Planned string-required ranges are single-line lexical byte spans; a plan that
would cross `\n` or `\r` is rejected before it can be injected as a protected raw
lexeme run.

## Dependencies

- Internal: `preprocess` (lexical text + map), `lexical_env`
  (`ActiveLexicalEnvironment`), `span_bridge` (lexical-byte → `SourceRange`),
  `parsing` (consumes `TokenStream`).
- External: `mizar-lexer` (`scan_raw_recoverable`, `RawScanDiagnostic`,
  `build_scope_skeleton`, `disambiguate`, `TokenKind`, `ParserLexContext`,
  `LexDiagnostic`, `LexDiagnosticCode`, structured lexer payload enums,
  `ScopeSkeletonDiagnostic`,
  `ScopeSkeletonDiagnosticCode`, and scope block / statement / binding shape enums),
  `mizar-session` (`SourceId`, `SourceRange`, `SourceAnchor`).

This module is consumed by parsing and, through the diagnostics, by the
orchestration merge.

## Data Structures

### Token Stream

`TokenStream` is the source-faithful token sequence for one file under the
parser lexing context used for the run. Successful tokenization surfaces final
parser-facing `UserSymbol`, `ReservedWord`, `ReservedSymbol`, `Identifier`,
`Numeral`, and `StringLiteral` classifications, with `TokenKind::ErrorRecovery`
tokens interleaved where recoverable raw-scan, lexer, or disambiguator
diagnostics consumed input. Raw-scan diagnostics use precise offending spans and
do not discard usable partial raw tokens. Each `Token` preserves its original
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

1. Build or receive the `ParserLexingPlan`. `ParserInputs` normally requests a
   position-sensitive plan computed from the lexical text; bounded tests may
   still request a uniform context.
2. Recoverably raw-scan the lexical text into raw units with preserved spans.
   String-required ranges in the plan are protected as raw lexeme runs before
   ordinary segments are scanned so Unicode and comment marker text inside
   annotation string arguments remain string contents. Raw-scan diagnostics are
   mapped to `LexingDiagnosticKind::RawScan` at their precise offending spans,
   and matching `ErrorRecovery` tokens are merged with the disambiguated tokens
   in source order. Usable partial raw tokens continue through scope-skeleton
   construction and disambiguation. Only internal parser-plan defects, such as
   non-UTF-8 ranges or line-crossing planned string spans, fall back to the
   older whole-lexical-text recovery.
3. Build an initial `ScopeSkeleton` / `ScopeLexView` from the raw tokens and run
   disambiguation once with the raw token stream, active lexical environment, and
   the `ParserLexContext` selected by the plan for each raw unit.
4. Rebuild the scope skeleton from the first final token shapes, treating
   `StringLiteral`, `ErrorRecovery`, numerals, and non-identifier user symbols as
   scope-inert. Run `disambiguate` again with that contextual skeleton so scoped
   identifier overrides are available without treating string contents or
   user-symbol spellings as scope syntax. Build the public scope skeleton from
   the final token shapes.
5. The disambiguator applies longest match against, in order: active user
   symbols, reserved special symbols, reserved words, identifier/numeral rules,
   and parser expectation / scoped override where the language requires it.
6. Map every final lexer token through `span_bridge.lexical_span` into a
   session-spanned frontend `Token`. Map the scope-skeleton frames, blocks,
   statements, and binding shapes into the public `ScopeView`.
7. Collect lexer diagnostics as `LexingDiagnostic`s, copy non-span payload data,
   and map nested rejected-candidate spans into frontend payload structures.

Compound reserved tokens (`.{`, `.*`, `.=`, `...`) are recognized by the lexer;
selector/namespace roles for `.` are left to the parser and resolver. The lexer
never decides definedness, applicability, or overload selection.

## Error Handling

The lexing wrapper preserves recoverable raw-scan, scope-skeleton, and current
disambiguator problems:

- recoverable raw-scan diagnostics emit precise `TokenKind::ErrorRecovery`
  tokens with `RawScan` diagnostics and preserve usable partial raw tokens for
  scope construction and disambiguation;
- `ScopeSkeletonDiagnostic`s are mapped into frontend diagnostics without
  storing raw span-bearing diagnostic structs in `TokenStream`;
- current `LexDiagnostic`s are mapped into `LexingDiagnosticKind::Lexer` with
  frontend-owned payloads and mapped nested rejected-candidate spans.

For user-recoverable lexical input problems, `tokenize` returns
`Ok(TokenStream)`: raw-scan failures become precise recovery tokens and mapped
diagnostics, while scope-skeleton and disambiguator problems are mapped without
dropping recoverable tokens. It returns `Err(SpanBridgeError)` only when a token,
scope shape, or diagnostic span cannot be mapped through the registered bridge.

## Tests

Implemented task-7/8/9 and task-20 scenarios:

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
- `UnsupportedLexerPayload` remains a constructible fallback for future
  non-exhaustive lexer payload variants and does not invent recovery advice;
- malformed raw input, malformed lexemes, and unsupported raw-token cases emit
  mapped `ErrorRecovery` tokens without preventing later tokens from being
  emitted;
- parser-context-rejected numerals preserve mapped rejected-candidate payloads
  until a dedicated invalid-numeral lexer diagnostic exists;
- scope diagnostics remain preserved with mapped spans after disambiguation even
  when recoverable lexer diagnostics are also present;
- position-sensitive parser lexing plans emit `StringLiteral` only at planned
  single-line lexical byte ranges, including annotation string arguments with
  Unicode and comment-marker contents;
- position-sensitive user-symbol kind filters can classify the same spelling
  differently at different lexical byte ranges;
- real source-to-token-to-parser orchestration carries planned annotation string
  tokens through `MizarParserSeam`.

## Constraints and Assumptions

- This module orchestrates the lexer; it does not own longest-match, scope
  skeleton, or disambiguation rules.
- The lexer recognizes compound reserved tokens and active user symbols but not
  semantic selector/namespace roles.
- `StringLiteral` tokens are emitted only where the `ParserLexingPlan` selects a
  string-required context. The current plan builder recognizes single-line
  string arguments at quote positions after `(` or `,`; later grammar work can
  narrow or expand the planner without exposing arbitrary parser state to the
  lexer.
- When token streams are cached, the cache key is `PreprocessedSource.lexical_hash`,
  the active lexical environment fingerprint, and a stable encoding of the
  `ParserLexContext` / parser-assisted lexing plan used for the run.
- All token spans are session `SourceRange` values produced through `span_bridge`.
