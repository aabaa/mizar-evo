# mizar-parser: Recovery

Status: minimal task-12 recovery, task-28 nested block-end recovery, task-5
module-skeleton recovery, task-6 import recovery, task-7 export/visibility
recovery, task-8 type-expression recovery, task-9 primary-term recovery,
task-13 atomic-formula recovery, task-14 formula recovery, S-013/S-014
statement/proof recovery, and S-015 definition recovery through task 28 are
implemented. Full grammar recovery remains planned.

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
- task-9 primary-term parsing diagnoses malformed term-list and primary-term
  syntax with `MalformedTermExpression`. Pure missing term arguments insert
  `MissingTerm`; malformed non-empty tails may use `SkippedToken` recovery
  owned by the nearest term node. Parenthesized, application, set-enumeration,
  and reserved bracket-functor delimiters that reach synchronization before the
  expected closer get `MalformedTermExpression`, a secondary opener anchor, and
  `UnmatchedOpeningDelimiter` recovery under the nearest term node. A
  `ChoiceTerm` with `the` but no following type expression uses
  type-expression recovery (`MalformedTypeExpression` plus
  `MissingTypeExpression`) because the missing child is the choice term's type
  operand;
- task-10 selector/update parsing diagnoses malformed selector postfixes,
  selector-call arguments, and functional update lists with
  `MalformedTermExpression`. Missing field-update values insert `MissingTerm`.
  Selector-call and functional-update delimiters that reach synchronization
  before the expected closer get `MalformedTermExpression`, a secondary opener
  anchor, and `UnmatchedOpeningDelimiter` recovery under the nearest
  selector/update term node;
- task-12 operator parsing reports same-operator non-associative chains with
  `NonAssociativeOperatorChain`. A dangling infix operator reports
  `DanglingOperator`, consumes the operator, and leaves the partial left
  expression represented without requiring a recovery node. A dangling prefix
  operator reports `DanglingOperator` and keeps a recoverable
  `PrefixExpression` by inserting a `MissingTerm` operand;
- task-13 atomic-formula parsing reuses term/type recovery for malformed
  atomic operands. Built-in predicate applications with a missing right term
  insert `MissingTerm` and report `MalformedTermExpression`; `is` assertions
  with a missing body insert `MissingTypeExpression` and report
  `MalformedTypeExpression`;
- task-14 formula parsing inserts `MissingFormula` and reports
  `MalformedFormulaExpression` when a formula is required after prefix `not`,
  a binary connective, quantifier `st`, or `holds`. Quantifier headers are
  preserved after at least one variable segment is represented; missing
  explicit types after `be` / `being` reuse `MissingTypeExpression` and
  `MalformedTypeExpression`, while malformed header separators or tails report
  `MalformedFormulaExpression`. A parenthesized formula with no matching `)`
  before synchronization emits `UnmatchedOpeningDelimiter`, reports
  `MalformedFormulaExpression`, and uses the opener as a secondary diagnostic
  anchor;
- task-27 redefinition and notation-alias parsing reuses definition-content
  synchronization. Missing redefinition labels, subjects, raw patterns, term
  bodies, or raw notation-pattern sides use `MalformedTermExpression` with
  `MissingTerm` where an inserted child is needed. Missing `redefine func`
  return types use `MalformedTypeExpression` plus `MissingTypeExpression`.
  Missing delimiters, formula bodies, notation `for`, or the mandatory
  `coherence` keyword use `MalformedFormulaExpression`; malformed mandatory
  coherence justifications and optional `with` labels use
  `MalformedJustification` plus `MissingProofStep` where appropriate. Malformed
  tails skip to a semicolon, `end`, the next definition-content start, a
  top-level item boundary, or EOF;
- task-28 property-clause parsing reuses definition-content synchronization.
  Missing or malformed mandatory property justifications use
  `MalformedJustification` plus `MissingProofStep` where an inserted proof
  placeholder is needed. Malformed property tails skip to a semicolon, `end`,
  the next definition-content start, a top-level item boundary, or EOF. Missing
  property semicolons use `MissingSemicolon` and preserve a following
  definition item, including another property clause;
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
