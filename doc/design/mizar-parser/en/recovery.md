# mizar-parser: Recovery

Status: minimal task-12 recovery, task-28 nested block-end recovery, task-5
module-skeleton recovery, task-6 import recovery, task-7 export/visibility
recovery, and task-8 type-expression recovery are implemented; full grammar
recovery planned.

## Purpose

This module defines parser synchronization and recovery policy.

## Responsibilities

- synchronize at stable boundaries such as `;`, `end`, top-level item keywords, and EOF;
- emit syntax diagnostics while preserving recoverable syntax structure;
- create `mizar-syntax` recovery nodes instead of inventing semantic facts.

Current behavior:

- the parser has a private token cursor with bounded lookahead, an
  expected-token diagnostic helper that reuses existing `SyntaxDiagnosticCode`
  variants, synchronization sets, and recovery-node emission helpers. These are
  internal infrastructure only and do not change the crate-root public API;
- the synchronization set stops at `;`, `end`, EOF, and the task-5 top-level
  dispatch starts documented in [grammar.md](./grammar.md): `import`, `export`,
  `definition`, `reserve`, `registration`, `claim`, `theorem`, `lemma`,
  `open`, `assumed`, `conditional`, `private`, `public`, `infix_operator`,
  `prefix_operator`, `postfix_operator`, `synonym`, and `antonym`. Later item
  grammar tasks expand or narrow this set when they add concrete dispatch;
- missing `end` for block-like keywords is diagnosed at EOF when the parser's
  block stack remains open after matching available `end` tokens, and each
  missing close is represented with an explicit recovered `MissingEnd` node.
  The diagnostic keeps the block opener as a secondary anchor; the recovery
  node itself has no required context child so later module skeleton nodes can
  own the source tokens without duplicating non-root parents.
  The current stack includes top-level blocks plus algorithm control blocks
  with their own `end`. `for` is opened only for loop-like
  `for <identifier> = ...` / `for <identifier> in ...` token shapes so formula
  quantifiers do not consume block ends. Until concrete statement and match
  parsers land, `if` uses a syntactic heuristic: it opens after obvious
  algorithm/proof control introducers or when a `do` body marker appears before
  the next boundary. `otherwise` likewise opens after `end` or `end;`, matching
  the surface shape of completed match cases; expression-level `otherwise`
  without that prefix is not opened. `else if` is treated as one conditional
  chain rather than a nested block opener;
- missing string literals in synthetic string-required parser contexts are
  diagnosed and represented with an explicit recovered `MissingStringLiteral`
  node;
- task-5 module skeleton parsing diagnoses missing top-level item semicolons
  with `MissingSemicolon`, and skips unexpected top-level tokens with
  `UnexpectedTopLevelToken`, an explicit recovered `SkippedToken` node, and a
  `SurfaceTrivia::skipped_token_ranges` entry using `SkippedTokenReason::Recovery`;
- task-6 import parsing keeps late imports after the import prelude in the
  task-5 skipped-token recovery path, diagnoses missing import statement
  semicolons with `MissingSemicolon`, and diagnoses import-internal syntax that
  can continue at the current statement boundary, such as a missing alias after
  `as` or a missing branch-import `}`, with `MalformedImport`;
- task-7 export parsing keeps late exports after the export prelude in the
  task-5 skipped-token recovery path, diagnoses missing export statement
  semicolons with `MissingSemicolon`, and diagnoses export-internal syntax that
  can continue at the current statement boundary, such as a missing path after
  `export` or after a comma, with `MalformedExport`. Task-7 visibility parsing
  diagnoses duplicate markers, dangling markers, and visibility applied to a
  non-theorem/non-notation top-level declaration with `MalformedVisibility`;
- task-8 reserve/type-expression parsing diagnoses malformed reserve-hosted type
  syntax with `MalformedTypeExpression`. A pure missing type after `reserve ...
  for` or inside bracket `type_arg_list` gets an explicit recovered
  `MissingTypeExpression`; malformed non-empty tails use `SkippedToken` recovery
  owned by the nearest reserve/type node. A bracket type-argument list that
  reaches `;`, a top-level item boundary, or EOF before `]` gets
  `MalformedTypeExpression`, a secondary anchor on `[`, and an
  `UnmatchedOpeningDelimiter` recovery node under `TypeArguments`;
- a stray `end` that has no matching block opener returns syntax diagnostics
  with `ast = None`.

## Public Enum Compatibility

`StringRequiredContext` is `#[non_exhaustive]` for downstream crates. Current
parser behavior only distinguishes `None` from the synthetic `UniformForTest`
context, but real grammar growth will add parser-facing string-required
positions for operator declarations and annotation arguments. Downstream
matches must keep wildcard fallback arms, while matches inside `mizar-parser`
remain exhaustive so new contexts force local recovery and token-adaptation
updates.
