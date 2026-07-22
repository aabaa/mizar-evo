# mizar-parser: Recovery

Status: recovery is implemented and task-37 consolidated for the parser grammar
surface through task 36: module/import/export, type/term/formula,
statement/proof, S-015 definition and registration content, templates,
algorithms/claims, algorithm control flow and verification clauses,
annotations, and predicate redefinition label repair. Future grammar growth may
add new category-local recovery cases, but no known implemented parser category
falls through to an unintended abort. The documented stray unmatched `end`
path remains intentionally unrecoverable and returns diagnostics with
`ast = None`.

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
  with their own `end`. Parser task 33 owns concrete statement-list recovery
  for `if`, `while`, `for`, and `match`; the recovery prepass keeps a shallow
  syntactic mirror for block-end matching. `for` is opened for loop-like
  `for <identifier> = ...` / `for <identifier> in ...` token shapes and for
  malformed-head shapes that still expose a `do` body marker before the next
  boundary, so formula quantifiers do not consume block ends. `if` opens after obvious
  algorithm/proof control introducers or when a `do` body marker appears before
  the next boundary. `otherwise` opens only in an open algorithm block after
  `end` or `end;`, matching the surface shape of completed match cases;
  expression-level and definition-side `otherwise` without that algorithm
  prefix is not opened. `else if` is treated as one conditional chain rather
  than a nested block opener;
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
- task-29 structure and inheritance parsing uses local member synchronization
  inside `struct ... end` and explicit `inherit ... where ... end` blocks, then
  returns to definition-content synchronization at the block boundary. Empty or
  malformed structure patterns, inheritance targets, member names, redefinition
  names, and malformed member tails use `MalformedTermExpression` with
  `MissingTerm` where an inserted placeholder is needed. Missing member or
  redefinition types use `MalformedTypeExpression` plus
  `MissingTypeExpression`. Missing or malformed inheritance coherence
  justifications use `MalformedJustification` plus `MissingProofStep`;
  inheritance `coherence with ...` is recovered rather than accepted. Missing
  member semicolons and missing outer semicolons use `MissingSemicolon`;
  missing block closers use `MissingEnd`. Malformed member tails skip to a
  semicolon, `field`, `property`, `coherence`, `end`, the next
  definition-content start, a top-level item boundary, or EOF. The
  frontend-facing scope skeleton recognizes nested `struct` blocks and only
  treats `inherit` as block-like when a `where` appears before the statement
  semicolon or `end`;
- task-30 registration parsing uses registration-content synchronization
  inside `registration ... end` blocks. Malformed registration parameters,
  cluster heads, missing labels, missing antecedents or consequents,
  malformed functorial payloads, unsupported nullary functorial ambiguity,
  missing correctness conditions, and malformed reduction justifications use
  the existing term/formula/type/justification recovery vocabulary and skip to
  semicolon, `end`, the next registration-content start, a top-level item
  boundary, or EOF. Missing registration block closers use `MissingEnd`;
- task-31 template parsing recovers malformed template loci and argument
  lists with `MalformedTypeExpression`, `MalformedTermExpression`, or
  `MalformedFormulaExpression` according to the missing child kind. Chained
  non-associative template predicate arguments keep the task-14
  `NonAssociativeOperatorChain` diagnostic. Malformed template tails
  synchronize at bracket, comma, semicolon, block, or item boundaries;
- task-32 algorithm and claim parsing recovers malformed algorithm schema
  loci, parameter lists, missing return types, declaration bindings, ghost
  assignments, snapshot statements, return tails, assignment terms, malformed
  claim targets/content, and missing algorithm or claim semicolons while
  preserving the surrounding definition or top-level block item;
- task-33 algorithm control-flow parsing recovers malformed `if`, `while`,
  range and collection `for`, `match`, `otherwise`/`exhaustive`, `break`, and
  `continue` shapes at algorithm statement-list boundaries. Missing nested
  control-flow closers use `MissingEnd`; malformed heads and tails use the
  nearest term/formula/skipped-token recovery vocabulary without consuming
  following statements;
- task-34 algorithm verification parsing recovers duplicate or out-of-order
  header clauses by skipping to the algorithm body boundary, inserts
  `MissingFormula` after `requires`, `ensures`, loop `invariant`, and `assert`
  when needed, inserts `MissingTerm` inside `TermList` for empty or dangling
  `decreasing` measures, rejects `for ... do decreasing ...;` with
  skipped-token recovery through the clause semicolon, and treats
  `invariant` / `decreasing` after an ordinary loop-body statement as
  misplaced algorithm statements recovered at the clause semicolon;
- task-35 annotation parsing reports malformed annotation arguments,
  proof-hint options, empty slots, invalid fixed-annotation values, standalone
  diagnostic annotation operands, and unmatched annotation delimiters with
  `MalformedAnnotation`, `MissingAnnotationArgument`,
  `MalformedTermExpression`, or `UnmatchedOpeningDelimiter` as appropriate.
  Malformed annotation delimiters synchronize at the following eligible item,
  definition content, registration content, statement, algorithm statement,
  semicolon, `end`, or EOF boundary so a bad prefix does not consume the host
  that follows it;
- task-36 predicate redefinition label repair treats an omitted required
  redefinition label as malformed term syntax and inserts a `MissingTerm`
  child before the predicate pattern while preserving the corrected
  `redefine pred label: pattern` child order;
- task-37 consolidation audited the implemented recovery surfaces, closed the
  malformed-annotation host synchronization gap, and expanded active fail
  corpus coverage for annotation recovery across definition, algorithm,
  top-level, and registration hosts;
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

## Task 47 `reconsider` Tail Recovery

Omitting a `reconsider` justification before the final semicolon is valid
syntax and emits no recovery node or parser diagnostic. An explicit `by` tail
keeps the existing simple-justification recovery. A proof tail reuses the
ordinary `ProofBlock` recovery: a missing `end` emits `MissingEnd` while the
enclosing `ReconsiderStatement` continues to own the final semicolon.

This exception is local to `reconsider_tail`. `consider` and the remaining
simple-justification-only hosts retain mandatory-`by` recovery. The existing
mixed consider/reconsider failure source is unchanged; only its obsolete
omitted-tail `MalformedJustification` expectation was removed.
