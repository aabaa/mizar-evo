# Module: parsing

> Canonical language: English. Japanese companion: [../ja/parsing.md](../ja/parsing.md).

Status: planned.

## Purpose

This module implements the frontend pipeline Step 5 (parser invocation). It calls
`mizar-parser` to turn a `TokenStream` into a `mizar-syntax::SurfaceAst`, supplies
the parser inputs derived from the active lexical environment (operator fixity,
string-required grammar contexts), and threads parser-requested lexing context
back to Step 4 when the integration uses parser-driven disambiguation.

It does not own `SurfaceAst` node definitions (those live in `mizar-syntax`) or
the grammar, Pratt precedence, recovery, or annotation-attachment logic (those
live in `mizar-parser`). It only adapts inputs and outputs at the crate boundary.

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

pub struct ParseOutput {
    pub ast: Option<SurfaceAst>,
    pub diagnostics: Vec<SyntaxDiagnostic>,
}

pub fn parse(request: ParseRequest<'_>) -> ParseOutput;
```

`SurfaceAst` and `SyntaxDiagnostic` are owned by `mizar-syntax`; `parse`
delegates to `mizar-parser`'s entry point. `ParserInputs` is derived by the
frontend from the active lexical environment and edition; `operator_fixity` and
`string_required_positions` are the only lexing-relevant signals the parser
needs, matching the narrow `ParserLexContext` contract on the lexer side.

`ast = None` means the parser could not recover enough structure for later
phases; lexical and syntax diagnostics are still returned.

## Dependencies

- Internal: `lexing` (provides `TokenStream`), `lexical_env` (provides the data
  for `ParserInputs`), `orchestration` (consumes `ParseOutput`).
- External: `mizar-parser` (grammar entry point, Pratt parsing, recovery,
  annotation/doc-comment attachment), `mizar-syntax` (`SurfaceAst`,
  `SyntaxDiagnostic`), `mizar-session` (`Edition`, `SourceRange` carried inside
  AST nodes).

This module is consumed by orchestration to assemble `FrontendOutput`.

## Data Structures

### Parser Inputs

`ParserInputs` is the frontend-assembled bundle of grammar-affecting
configuration: language edition, operator fixity derived from the active lexical
environment, and the registry of string-required argument positions. It is the
parser-facing counterpart to the lexer's `ParserLexContext`: it never carries
arbitrary scope or resolver state.

### Surface AST Handoff

`SurfaceAst` (from `mizar-syntax`) is source-shaped: it preserves source order
and `SourceRange`s, attaches annotations as syntax nodes, attaches doc comments
to nearby documentable items, and marks recovery nodes explicitly. The frontend
passes it through unchanged; it does not rewrite, prune, or interpret nodes.

## Algorithm / Logic

### Parse a token stream

1. Build `ParserInputs` from the active lexical environment and edition.
2. Invoke the `mizar-parser` entry point with the `TokenStream` and inputs.
3. The parser parses modules, definitions, registrations, statements, terms,
   formulas, theorems, proofs, and algorithms; uses Pratt/precedence parsing for
   term and formula expressions; parses annotation argument lists and attaches
   annotations; attaches doc comments to nearby documentable items; and recovers
   at synchronization points (`;`, `end`, top-level item keywords, EOF).
4. Return the `SurfaceAst` (or `None` when recovery failed) plus syntax
   diagnostics.

When the integration uses parser-driven disambiguation, the parser supplies
string-required positions back to Step 4 through the narrow context object; the
parser never drives the lexer cursor directly.

## Error Handling

Syntax diagnostics come from `mizar-parser` (unexpected token, unmatched
delimiter, missing `end`, expected string literal token missing, malformed
annotation argument list). The frontend does not add syntax error categories. A
parse that cannot recover a usable tree returns `ast = None` with diagnostics so
later phases can skip or degrade gracefully. Recovery nodes inside a returned
`SurfaceAst` are marked by the parser; the frontend preserves those markers.

## Tests

Key scenarios:

- a well-formed token stream parses to a `SurfaceAst` with preserved source
  order and `SourceRange`s;
- operator fixity from the active lexical environment drives correct Pratt
  precedence for a user-defined infix operator;
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
- The parser consumes already-tokenized `StringLiteral` tokens and never drives
  the lexer cursor directly.
- `ParserInputs` carries only grammar-affecting configuration, not resolver state.
- `SurfaceAst` may contain recovery nodes; later phases must tolerate or reject
  them explicitly.
- The `SurfaceAst` cache key is the token-stream hash plus parser version and
  edition.
