# Module: parsing

> Canonical language: English. Japanese companion: [../ja/parsing.md](../ja/parsing.md).

Status: parser-input assembly, the stub parser seam, and a minimal real
`mizar-parser` seam are implemented; parser recovery passthrough remains gated.

## Purpose

This module implements the frontend pipeline Step 5 (parser invocation). It
calls a parser seam to turn a `TokenStream` into an AST, supplies the parser
inputs derived from the active lexical environment (operator fixity,
string-required grammar contexts), and threads parser-requested lexing context
back to Step 4 when the integration uses parser-driven disambiguation.

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
    UniformForTest,
}

impl StringRequiredContext {
    pub fn parser_lex_context(self) -> ParserLexContext;
}

pub trait ParserSeam {
    type Ast;
    type Diagnostic;

    fn parse(&self, request: ParseRequest<'_>) -> ParseOutput<Self::Ast, Self::Diagnostic>;
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

`operator_fixity` is populated only from data present in dependency lexical
summaries. If the current summary shape does not yet expose fixity, the default
source-to-token path uses `OperatorFixityTable { entries: Vec::new() }`; explicit
synthetic parser inputs can still exercise the real Pratt/fixity seam.
`StringRequiredContext::None` is the normal source-to-token foundation mode, and
`UniformForTest` is only for bounded tests that intentionally run the whole lexer
under `ParserLexContext::string_required()`. Position-specific string-required
spans and parser-driven symbol-kind filters are not represented by this initial
type; they are added by the parser-assisted lexing contract before real source
inputs that require grammar-position string literals are enabled.

With the stub parser seam, `ast = None` is the expected placeholder result. The
task-11 real parser seam returns a minimal `SurfaceAst` for recovered token
streams; later parser recovery tasks may use `ast = None` when parsing cannot
recover enough structure for downstream phases. Lexical and syntax diagnostics
are still returned.

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
source-to-token pipeline or the real parser boundary. The task-11 real seam
preserves source order and `SourceRange`s in `SurfaceAst` token nodes and
supports explicit infix fixity through a small Pratt parser. Later parser tasks
expand that same boundary with full module/item nodes, annotation attachment,
doc-comment attachment, and recovery markers. The frontend passes parser output
through unchanged; it does not rewrite, prune, or interpret nodes.

## Algorithm / Logic

### Parse a token stream

1. Build `ParserInputs` from the active lexical environment and edition.
2. Invoke the configured `ParserSeam` with the `TokenStream` and inputs. The
   stub seam returns `ast = None` with no syntax diagnostics.
3. The task-11 parser preserves token nodes in source order and builds minimal
   infix expression nodes when explicit operator fixity is supplied. Later parser
   tasks add full module/item parsing, annotation and doc-comment attachment, and
   recovery at synchronization points (`;`, `end`, top-level item keywords, EOF).
4. Return the `SurfaceAst` plus syntax diagnostics. Later recovery work may also
   return `None` when parsing cannot recover enough structure for downstream
   phases.

When the integration uses parser-assisted disambiguation, the parser-facing seam
supplies string-required positions or symbol-kind filters back to Step 4 through
a narrow request object. Until that contract lands, real parser integration must
not require source-level annotation/operator string-literal tokenization; tests
for those cases use synthetic token streams or remain gated. The lexer still must
not receive arbitrary parser state, and the parser must not mutate lexer internals
directly.

## Error Handling

Syntax diagnostics come from `mizar-parser`; the frontend does not add syntax
error categories. The task-11 parser can emit `UnexpectedErrorToken`,
`DanglingOperator`, and `NonAssociativeOperatorChain` while preserving a minimal
`SurfaceAst`. Broader diagnostics such as unexpected token, unmatched delimiter,
missing `end`, expected string literal, and malformed annotation argument list
remain future parser/recovery work. A real parser parse that cannot recover a
usable tree may return `ast = None` in later tasks; task 11 currently returns a
minimal `SurfaceAst` for recovered token streams. The stub seam still emits no
syntax diagnostics and returns `ast = None`.

## Tests

Key scenarios:

- the stub seam returns `ast = None` and no diagnostics;
- `ParserInputs` uses an empty `OperatorFixityTable` and
  `StringRequiredContext::None` for the stub source-to-token path;
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
- annotation nodes, missing string literal diagnostics, missing-`end` recovery,
  unrecoverable `ast = None`, and doc-comment attachment remain future
  parser/recovery coverage.

## Constraints and Assumptions

- This module does not own AST node definitions or grammar/recovery logic.
- The parser consumes the token stream produced by the frontend. If
  parser-assisted lexing is needed for string literals, the parser communicates
  only through the agreed narrow context/plan object and never exposes arbitrary
  parser state to the lexer.
- `ParserInputs` carries only grammar-affecting configuration, not resolver state.
- `SurfaceAst` may contain recovery nodes; later phases must tolerate or reject
  them explicitly.
- The `SurfaceAst` cache key is the token-stream hash plus parser version and
  edition.
