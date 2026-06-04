# Module: parsing

> Canonical language: English. Japanese companion: [../ja/parsing.md](../ja/parsing.md).

Status: planned.

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

pub struct ParserInputs {
    pub edition: Edition,
    pub operator_fixity: OperatorFixityTable,
    pub string_required_positions: StringRequiredContext,
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

pub struct StubParserSeam;
```

`SurfaceAst` and `SyntaxDiagnostic` are owned by `mizar-syntax`; once
`mizar-parser` exists, the real parser seam delegates to its entry point. Until
those crates exist, `StubParserSeam` returns `ast = None` and an empty diagnostic
list without introducing hard dependencies on non-existent crates. `ParserInputs`
is derived by the frontend from the active lexical environment and edition after
Step 3; callers do not supply it to the top-level frontend coordinator.

`operator_fixity` is populated only from data present in dependency lexical
summaries. If the current summary shape does not yet expose fixity, the stub
path uses an empty table and the TODO keeps real Pratt/fixity tests gated on
`mizar-parser` / `mizar-syntax` availability. `string_required_positions` is the
parser-facing description of grammar positions that require string tokens; the
exact way it becomes a lexer `ParserLexContext` / lexing plan is the
parser-assisted lexing contract.

`ast = None` means the parser could not recover enough structure for later
phases; lexical and syntax diagnostics are still returned.

## Dependencies

- Internal: `lexing` (provides `TokenStream`), `lexical_env` (provides the data
  for `ParserInputs`), `orchestration` (consumes `ParseOutput`).
- External: `mizar-session` (`Edition`, `SourceRange` carried inside AST nodes).
  The real parser seam additionally depends on `mizar-parser` (grammar entry
  point, Pratt parsing, recovery, annotation/doc-comment attachment) and
  `mizar-syntax` (`SurfaceAst`, `SyntaxDiagnostic`) once those crates exist.

This module is consumed by orchestration to assemble `FrontendOutput`.

## Data Structures

### Parser Inputs

`ParserInputs` is the frontend-assembled bundle of grammar-affecting
configuration: language edition, operator fixity derived from the active lexical
environment, and the registry of string-required argument positions. It is the
parser-facing counterpart to the lexer's `ParserLexContext`: it never carries
arbitrary scope or resolver state.

### Parser Seam and Surface AST Handoff

The parser seam lets the frontend compile and test the source-to-token pipeline
before `mizar-syntax` and `mizar-parser` land. In the real seam, `SurfaceAst`
(from `mizar-syntax`) is source-shaped: it preserves source order and
`SourceRange`s, attaches annotations as syntax nodes, attaches doc comments to
nearby documentable items, and marks recovery nodes explicitly. The frontend
passes it through unchanged; it does not rewrite, prune, or interpret nodes.

## Algorithm / Logic

### Parse a token stream

1. Build `ParserInputs` from the active lexical environment and edition.
2. Invoke the configured `ParserSeam` with the `TokenStream` and inputs. The
   stub seam returns `ast = None` with no syntax diagnostics.
3. The parser parses modules, definitions, registrations, statements, terms,
   formulas, theorems, proofs, and algorithms; uses Pratt/precedence parsing for
   term and formula expressions; parses annotation argument lists and attaches
   annotations; attaches doc comments to nearby documentable items; and recovers
   at synchronization points (`;`, `end`, top-level item keywords, EOF).
4. Return the `SurfaceAst` (or `None` when recovery failed) plus syntax
   diagnostics.

When the integration uses parser-assisted disambiguation, the parser-facing seam
supplies string-required positions or symbol-kind filters back to Step 4 through
a narrow request object. The lexer still must not receive arbitrary parser
state, and the parser must not mutate lexer internals directly.

## Error Handling

Syntax diagnostics come from `mizar-parser` (unexpected token, unmatched
delimiter, missing `end`, expected string literal token missing, malformed
annotation argument list). The frontend does not add syntax error categories. A
parse that cannot recover a usable tree returns `ast = None` with diagnostics so
later phases can skip or degrade gracefully. Recovery nodes inside a returned
`SurfaceAst` are marked by the parser; the frontend preserves those markers.
The stub seam emits no syntax diagnostics; tests that require syntax diagnostics
are gated on the real parser seam.

## Tests

Key scenarios:

- the stub seam returns `ast = None` and no diagnostics without depending on
  `mizar-syntax` or `mizar-parser`;
- once the real seam is available, a well-formed token stream parses to a
  `SurfaceAst` with preserved source order and `SourceRange`s;
- once lexical summaries expose fixity, operator fixity from the active lexical
  environment drives correct Pratt precedence for a user-defined infix operator;
- a `StringLiteral` token at an annotation argument parses into an annotation
  node; a missing string literal at a string-required position yields the
  expected syntax diagnostic;
- a missing `end` recovers at a synchronization point and produces an explicit
  error node, with `ast` still `Some`;
- an unrecoverable token stream returns `ast = None` with diagnostics;
- doc comments are attached to the following documentable item when possible and
  kept near their source location otherwise.

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
