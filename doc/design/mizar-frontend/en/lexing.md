# Module: lexing

> Canonical language: English. Japanese companion: [../ja/lexing.md](../ja/lexing.md).

Status: in progress (task 7 raw-scan / scope-skeleton skeleton complete; task
8 disambiguation and task 9 recovery passthrough pending).

## Purpose

This module implements the frontend pipeline Step 4 (lexing / disambiguation).
The task-7 implementation drives the `mizar-lexer` raw scanner and
scope-skeleton pre-scan to turn a `PreprocessedSource` into a session-spanned
raw `TokenStream` skeleton. Tasks 8 and 9 extend the same entry point with
context-sensitive disambiguation, final parser-facing token kinds, and
recoverable lexer diagnostic payload mapping.

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
    pub span: SourceRange,
    pub text: InternedText,
}

pub type InternedText = Arc<str>;

pub struct TokenizeRequest<'a> {
    pub preprocessed: &'a PreprocessedSource,
    pub environment: &'a ActiveLexicalEnvironment,
    pub parser_context: ParserLexContext,
}

pub struct LexingDiagnostic {
    pub kind: LexingDiagnosticKind,
    pub message: Arc<str>,
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
}

pub struct ScopeView {
    pub source_id: SourceId,
    pub frames: Vec<ScopeFrame>,
    pub blocks: Vec<ScopeBlock>,
    pub statements: Vec<ScopeStatement>,
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

`TokenKind`, `ParserLexContext`, `BindingShapeKind`, `LexicalBlockKind`,
`LexicalStatementKind`, `ScopeSkeletonDiagnosticCode`, and `LexDiagnosticCode`
are re-exported from `mizar-lexer`. `InternedText` is a frontend-local
`Arc<str>` spelling handle for the first implementation; no global interner is
required, and lexer `String` lexemes are converted with `Arc::<str>::from`. Raw
lexer diagnostic structs are consumed as inputs only: the frontend converts them
immediately into `LexingDiagnostic` so public diagnostics carry session ranges
and never expose raw lexer byte spans. Structured lexer payload variants and
mapped rejected candidates are added by tasks 8 and 9.

`parser_context` is carried now so task 8 can run the same request through the
disambiguator without changing the frontend boundary. The task-7 skeleton records
it but does not yet use it to classify tokens. `environment` is likewise part of
the request boundary for task 8; task 7 proves that imported symbols are not
mixed into the lexical `ScopeView`.

## Dependencies

- Internal: `preprocess` (lexical text + map), `lexical_env`
  (`ActiveLexicalEnvironment`), `span_bridge` (lexical-byte → `SourceRange`),
  `parsing` (consumes `TokenStream`).
- External: `mizar-lexer` (`scan_raw`, `build_scope_skeleton`, `TokenKind`,
  `ParserLexContext`, `LexDiagnosticCode`, `ScopeSkeletonDiagnostic`,
  `ScopeSkeletonDiagnosticCode`, scope block / statement / binding shape enums),
  `mizar-session` (`SourceId`, `SourceRange`, `SourceAnchor`). Tasks 8 and 9
  add `disambiguate`, `LexDiagnostic`, and structured lexer payload mapping.

This module is consumed by parsing and, through the diagnostics, by the
orchestration merge.

## Data Structures

### Token Stream

`TokenStream` is the source-faithful token sequence for one file under the
parser lexing context used for the run. In the task-7 skeleton, non-layout raw
tokens are surfaced as session-spanned `TokenKind::LexemeRun` values, and strict
raw-scan failure emits one coarse `TokenKind::ErrorRecovery`. Task 8 replaces
those raw runs with final parser-facing `UserSymbol`, `ReservedWord`,
`ReservedSymbol`, `Identifier`, `Numeral`, and `StringLiteral` classifications.
Each `Token` preserves its original spelling (`text`) and a session
`SourceRange`.

`LexingDiagnostic` is the mapped frontend diagnostic payload for Step 4. In task
7 it represents raw-scan failures and scope-skeleton diagnostics. Tasks 8 and 9
add disambiguator diagnostics and structured payload variants. It always carries
a session-coordinate primary range and secondary anchors for orchestration. Its
secondary entries are `SourceAnchor`s so composite or degraded preprocess
mappings can preserve point, generated, and adjacent comment anchors. It stores
the mapped code/message, not the raw lexer diagnostic object, because those raw
objects contain lexer byte spans.

### Scope Lex View

The scope-skeleton pre-scan (`build_scope_skeleton`) produces a read-only
`ScopeLexView` over raw lexer output, used by the disambiguator for scoped
identifier override rules. The frontend maps that skeleton into a public
session-spanned `ScopeView` for downstream inspection and diagnostics; it does
not build scopes itself, and the view records lexical shape only — never
resolved bindings. Task 8 runs the disambiguator against the raw scope view
inside the same lexing pass.

## Algorithm / Logic

### Tokenize a preprocessed source

1. Raw-scan the lexical text (`scan_raw`) into raw units with preserved spans.
   If strict raw scanning fails, emit one coarse `ErrorRecovery` token and one
   mapped `LexingDiagnosticKind::RawScan` diagnostic covering the whole lexical
   text (or the source-start zero-length range for empty text), then skip
   scope-skeleton construction and disambiguation for that run. The current
   `mizar_lexer::LexError` has no span or partial-token payload, so finer recovery
   is tracked as a follow-up contract.
2. Build the `ScopeSkeleton` / `ScopeLexView` from the raw tokens for scoped
   identifier override, collecting and mapping `ScopeSkeletonDiagnostic`s.
3. Map non-layout raw units through `span_bridge.lexical_span` into
   session-spanned `TokenKind::LexemeRun` skeleton tokens and map the
   scope-skeleton frames, blocks, statements, and binding shapes into a public
   `ScopeView`.
4. Task 8 runs `disambiguate` (or the parser-integrated `lex`) with longest-match
   against, in order: active user symbols, reserved special symbols, reserved
   words, identifier/numeral rules, and parser expectation / scoped override
   where the language requires it.
5. Task 8 also recognizes `StringLiteral` tokens only when the current parser
   lexing context is `StringRequired`; position-sensitive context for only
   selected byte spans is deferred until the parser-assisted lexing contract
   lands.
6. Tasks 8 and 9 collect lexer diagnostics as `LexingDiagnostic`s, copy non-span
   payload data, and map nested candidate spans into mapped payload structures.

Compound reserved tokens (`.{`, `.*`, `.=`, `...`) are recognized by the lexer;
selector/namespace roles for `.` are left to the parser and resolver. The lexer
never decides definedness, applicability, or overload selection.

## Error Handling

The task-7 skeleton preserves recoverable raw-scan and scope-skeleton problems:

- strict raw-scan hard failure emits one coarse `TokenKind::ErrorRecovery` with
  the best available source range plus a `RawScan` diagnostic;
- `ScopeSkeletonDiagnostic`s are mapped into frontend diagnostics without
  storing raw span-bearing diagnostic structs in `TokenStream`.

For user-recoverable lexical input problems, `tokenize` returns
`Ok(TokenStream)`: strict raw-scan failures degrade to a coarse recovery token
and mapped diagnostic, while scope-skeleton problems are mapped without dropping
recoverable raw tokens. It returns `Err(SpanBridgeError)` only when a token,
scope shape, or diagnostic span cannot be mapped through the registered bridge.
Disambiguator recovery passthrough, including malformed tokens, invalid numerals,
and malformed string literals, is task 9.

## Tests

Task-7 scenarios:

- raw scan preserves `LexemeRun` spans, including spans mapped through
  preprocess mappings such as removed comments;
- the public `ScopeView` reflects lexical block / statement shape and local
  binding shapes without resolved or imported bindings;
- scope-skeleton diagnostics are preserved as mapped `LexingDiagnostic`s;
- every emitted token's `span` is a valid `SourceRange` for `source_id`.

Task-8/9 scenarios add user-symbol longest match, compound reserved tokens,
`StringRequired` runs, recoverable malformed tokens, invalid numerals, malformed
string literals, and structured lexer payload mapping.

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
