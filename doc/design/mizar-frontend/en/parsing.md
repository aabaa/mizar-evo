# Module: parsing

> Canonical language: English. Japanese companion: [../ja/parsing.md](../ja/parsing.md).

Status: parser-input assembly, the stub parser seam, a minimal real
`mizar-parser` seam, task-12 parser recovery passthrough, and task-20
position-sensitive parser lexing-plan integration are implemented; full grammar
recovery remains gated.

## Purpose

This module implements the frontend pipeline Step 5 (parser invocation). It
calls a parser seam to turn a `TokenStream` into an AST, supplies the parser
inputs derived from the active lexical environment (operator fixity,
string-required grammar contexts), and defines the narrow parser-facing contract
that orchestration uses to precompute parser lexing plans before Step 4
tokenization.

It does not own `SurfaceAst` node definitions (those live in `mizar-syntax` for
the real parser seam) or the grammar, Pratt precedence, recovery, or
annotation-attachment logic (those live in `mizar-parser` for the real parser
seam). It only adapts inputs and outputs at the crate boundary.

See
[architecture/en/02.source_and_frontend.md](../../architecture/en/02.source_and_frontend.md)
"Step 5: Parse", "Syntax and Parser Are Separate Crate Boundaries", and
"Annotations Are Parser-Owned Syntax".

## Public API

```rust
pub const DEFAULT_PARSER_CACHE_KEY_VERSION: &str;
pub const STUB_PARSER_CACHE_KEY_VERSION: &str;
pub const MIZAR_PARSER_CACHE_KEY_VERSION: &str;

pub struct ParseRequest<'a> {
    pub tokens: &'a TokenStream,
    pub parser_inputs: ParserInputs,
}

impl<'a> ParseRequest<'a> {
    pub fn new(tokens: &'a TokenStream, parser_inputs: ParserInputs) -> Self;
}

pub struct ParserInputs {
    pub edition: Edition,
    pub operator_fixity: OperatorFixityTable,
    pub string_required_positions: StringRequiredContext,
}

impl ParserInputs {
    pub fn new(
        edition: Edition,
        operator_fixity: OperatorFixityTable,
        string_required_positions: StringRequiredContext,
    ) -> Self;

    pub fn from_active_environment(
        edition: Edition,
        environment: &ActiveLexicalEnvironment,
    ) -> Self;
}

pub struct OperatorFixityTable {
    pub entries: Vec<OperatorFixityEntry>,
}

impl OperatorFixityTable {
    pub fn empty() -> Self;
    pub fn is_empty(&self) -> bool;
}

pub struct OperatorFixityEntry {
    pub symbol_id: SymbolId,
    pub spelling: Arc<str>,
    pub precedence: u8,
    pub associativity: OperatorAssociativity,
}

pub enum OperatorAssociativity {
    Left,
    Right,
    NonAssociative,
}

pub enum StringRequiredContext {
    None,
    PositionSensitive,
    UniformForTest,
}

impl StringRequiredContext {
    pub fn parser_lex_context(self) -> ParserLexContext;
    pub fn parser_lexing_plan(self, lexical_text: &str) -> ParserLexingPlan;
}

pub trait ParserSeam {
    type Ast;
    type Diagnostic;

    fn cache_key_version(&self) -> ParserCacheKeyVersion;
    fn parse(&self, request: ParseRequest<'_>) -> ParseOutput<Self::Ast, Self::Diagnostic>;
}

pub struct ParserCacheKeyVersion {
    pub version: Arc<str>,
}

impl ParserCacheKeyVersion {
    pub fn new(version: impl Into<Arc<str>>) -> Self;
}

pub struct ParseOutput<A, D> {
    pub ast: Option<A>,
    pub diagnostics: Vec<D>,
}

impl<A, D> ParseOutput<A, D> {
    pub fn new(ast: Option<A>, diagnostics: Vec<D>) -> Self;
}

pub struct StubParserSeam;

pub struct MizarParserSeam;
```

`SurfaceAst` and `SyntaxDiagnostic` are owned by `mizar-syntax`. The real
`MizarParserSeam` delegates to the `mizar-parser` entry point and returns those
outputs unchanged. `StubParserSeam` remains available for source-to-token
coordinator paths and returns `ast = None` plus an empty diagnostic list.
`ParserInputs` is derived by the frontend from the active lexical environment
and edition after Step 3; callers do not supply it to the top-level frontend
coordinator.
`ParserSeam::cache_key_version` supplies the parser component of
`SurfaceAstCacheKey`; `MizarParserSeam` and `StubParserSeam` use explicit
versions, while custom seams inherit a conservative custom-seam version unless
they override it.

`operator_fixity` is populated only from data present in dependency lexical
summaries. If the current summary shape does not yet expose fixity, the default
source-to-token path uses `OperatorFixityTable { entries: Vec::new() }`; explicit
synthetic parser inputs can still exercise the real Pratt/fixity seam.
`StringRequiredContext::PositionSensitive` is the normal source-to-token mode:
it asks the frontend to precompute a `ParserLexingPlan` over lexical byte ranges
before tokenization. `StringRequiredContext::None` remains available for
synthetic parser inputs that intentionally disable parser-assisted lexing, and
`UniformForTest` is only for bounded tests that intentionally run the whole lexer
under `ParserLexContext::string_required()`. Position-specific string-required
spans and parser-driven symbol-kind filters are represented by
`ParserLexingPlan`; they do not expose arbitrary parser state to the lexer.

With the stub parser seam, `ast = None` is the expected placeholder result. The
real parser seam returns a minimal `SurfaceAst` for recovered token streams,
including task-12 recovery nodes for missing `end` when no `end` token is present
and for expected string literal positions. It may return `ast = None` when
parsing cannot recover enough structure for downstream phases. Lexical and
syntax diagnostics are still returned.

## Dependencies

- Internal: `lexing` (provides `TokenStream`), `lexical_env` (provides the data
  for `ParserInputs`), `orchestration` (consumes `ParseOutput`).
- External: `mizar-session` (`Edition`, `SourceRange` carried inside AST nodes),
  `mizar-syntax` (`SurfaceAst`, `SyntaxDiagnostic`), and `mizar-parser`
  (grammar entry point and minimal Pratt parsing).

This module is consumed by orchestration to assemble `FrontendOutput`.

## Data Structures

### Parser Inputs

`ParserInputs` is the frontend-assembled bundle of grammar-affecting
configuration: language edition, operator fixity derived from the active lexical
environment, and the registry of string-required argument positions. It is the
parser-facing counterpart to the lexer's `ParserLexContext`: it never carries
arbitrary scope or resolver state.

### Parser Seam and Surface AST Handoff

The parser seam lets the frontend compile and test either the stubbed
source-to-token pipeline or the real parser boundary. The real seam preserves
source order and `SourceRange`s in `SurfaceAst` token nodes, supports explicit
infix fixity through a small Pratt parser, and forwards task-12 recovery markers
unchanged. Later parser tasks expand that same boundary with full module/item
nodes, annotation attachment, doc-comment attachment, and broader recovery
markers. The frontend passes parser output through unchanged; it does not
rewrite, prune, or interpret nodes.

## Algorithm / Logic

### Parse a token stream

1. Build `ParserInputs` from the active lexical environment and edition.
2. Invoke the configured `ParserSeam` with the `TokenStream` and inputs. The
   stub seam returns `ast = None` with no syntax diagnostics.
3. The parser preserves token nodes in source order, builds minimal infix
   expression nodes when explicit operator fixity is supplied, and preserves
   task-12 recovery markers for missing `end` when no `end` token is present and
   expected string literals. Later parser tasks add full module/item parsing,
   annotation and doc-comment attachment, and broader synchronization coverage.
4. Return the `SurfaceAst` plus syntax diagnostics, or `ast = None` when the
   parser reports unrecoverable input.

Parser-assisted disambiguation uses a precomputed, position-sensitive plan rather
than interleaving parser and lexer execution. The parser-facing inputs select
`StringRequiredContext::PositionSensitive`, orchestration derives one
`ParserLexingPlan` from the lexical text, and Step 4 applies the plan by passing
only `ParserLexContext` values to the lexer. The lexer still must not receive
arbitrary parser state, and the parser must not mutate lexer internals directly.

## Error Handling

Syntax diagnostics come from `mizar-parser`; the frontend does not add syntax
error categories. The parser can emit `UnexpectedErrorToken`, `DanglingOperator`,
`NonAssociativeOperatorChain`, `MissingEnd`, `MissingStringLiteral`, and
`UnrecoverableInput`. Recoverable missing constructs are represented by explicit
`SurfaceNodeKind::ErrorRecovery` nodes marked `recovered`; unrecoverable input
returns diagnostics with `ast = None`. Broader diagnostics such as general
unexpected tokens, unmatched delimiters, and malformed annotation argument lists
remain future parser/recovery work. The stub seam still emits no syntax
diagnostics and returns `ast = None`.

## Tests

Key scenarios:

- the stub seam returns `ast = None` and no diagnostics;
- `ParserInputs` uses an empty `OperatorFixityTable` and
  `StringRequiredContext::PositionSensitive` for normal source-to-token paths;
- the real seam parses a well-formed token stream to a `SurfaceAst` with
  preserved source order and `SourceRange`s;
- the real seam preserves token-kind adaptation and returns parser diagnostics
  unchanged;
- explicit operator fixity drives Pratt precedence and associativity for
  supported infix operators; once lexical summaries expose fixity, the active
  lexical environment should populate those same parser inputs;
- recovered and unknown tokens are preserved but do not become infix operators;
- non-associative chains of the same operator are diagnosed while different
  operators at the same precedence remain distinct;
- missing `end` recovers conservatively at EOF with an explicit recovery node
  when no `end` token is present;
- one-token unrecoverable `end` input preserves `ast = None` plus syntax
  diagnostics;
- missing string literals at uniform string-required positions are diagnosed
  using synthetic token streams, while real source-text annotation string
  arguments are tokenized through the position-sensitive plan;
- annotation nodes, malformed annotation recovery, and doc-comment attachment
  remain future parser/recovery coverage.

## Constraints and Assumptions

- This module does not own AST node definitions or grammar/recovery logic.
- The parser consumes the token stream produced by the frontend. If
  parser-assisted lexing is needed for string literals, the parser communicates
  only through the agreed narrow context/plan object and never exposes arbitrary
  parser state to the lexer.
- `ParserInputs` carries only grammar-affecting configuration, not resolver state.
- `SurfaceAst` may contain recovery nodes; later phases must tolerate or reject
  them explicitly.
- The `SurfaceAst` cache key is the token-stream hash plus parser version,
  parser inputs hash, and edition.
