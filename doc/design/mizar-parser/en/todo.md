# mizar-parser TODO

> Canonical language: English. Japanese companion: [../ja/todo.md](../ja/todo.md).

## Status Legend

- [ ] not started
- [~] in progress
- [x] done

## Module Implementation

| Module | Spec | Source | Status |
|---|---|---|---|
| grammar | [grammar.md](./grammar.md) | `src/grammar.rs` | [~] minimal task-11/12 entry currently lives in `src/lib.rs` |
| pratt | [pratt.md](./pratt.md) | `src/pratt.rs` | [~] minimal explicit-fixity Pratt currently lives in `src/lib.rs` |
| recovery | [recovery.md](./recovery.md) | `src/recovery.rs` | [~] minimal task-12 recovery currently lives in `src/lib.rs` |

`mizar-parser` implements the syntax grammar: frontend-adapted tokens in,
`mizar_syntax::SurfaceAst` plus syntax diagnostics out. It is built as a thin
infrastructure layer (cursor, synchronization, recovery emission, corpus
runner) followed by grammar growth one syntactic category at a time, each
category paired with a `mizar-syntax` node-vocabulary task and a corpus
expansion.

## Crate Prerequisites

The crate depends on `mizar-session` and `mizar-syntax` only. Tokens arrive
already disambiguated by `mizar-frontend` with session `SourceRange`s;
parser-assisted lexing happens only through the precomputed
`ParserLexingPlan` / `StringRequiredContext` contract (resolved at the top
level, see [../../todo.md](../../todo.md) "Resolved And Open Decisions").
`ParseRequest` carries the operator fixity table and string-required context;
summary-driven fixity stays empty until lexical summaries expose fixity
metadata. The corpus harness (`mizar-test`) and the corpus tree
([tests/README.md](../../../../tests/README.md)) already exist.

## Test Corpus Policy

Sufficient corpus coverage is the success criterion for this crate. Every
grammar task ships, in the same change:

- **crate unit tests** for the new productions and their recovery behavior;
- **corpus tests** under `tests/miz/pass/parser/` and `tests/miz/fail/parser/`
  as 5-30 line `.miz` files with `.expect.toml` sidecars at stage
  `parse_only`, following the naming convention
  `pass_parser_<topic>_NNN.miz` / `fail_parser_<topic>_NNN.miz`
  ([tests/README.md](../../../../tests/README.md),
  [staged_model.md](../../mizar-test/en/staged_model.md));
- **coverage entries** in `tests/coverage/spec_trace.toml` mapping each case to
  the spec section it pins
  ([traceability.md](../../mizar-test/en/traceability.md)).

Grow toward the recommended pass/fail mix of
[architecture/en/20.test_strategy.md](../../architecture/en/20.test_strategy.md)
(40% pass / 60% fail overall): every accepted form gets at least one
malformed counterpart that must be rejected or recovered with diagnostics, and
recovery cases assert both the diagnostic and the recovered `SurfaceAst`
shape, not just "did not crash".

## Resolved And Open Decisions

- **Parser-assisted lexing contract: resolved** at the top level. The parser
  never interleaves with the lexer; string-required positions and user-symbol
  kind filters arrive through the precomputed plan.
- **Grammar source of truth and brush-up protocol: resolved.** The chapter
  specs under [doc/spec/en/](../../../spec/en/00.index.md) are normative;
  [appendix_a.grammar_summary.md](../../../spec/en/appendix_a.grammar_summary.md)
  is the consolidated summary and is still being brushed up. Each grammar task
  re-derives its productions from the owning chapter; when implementation
  exposes a gap, ambiguity, or contradiction in the EBNF, fix the owning
  chapter and the appendix (English and Japanese in the same change) as part
  of that task rather than deferring. Grammar brush-up is expected to proceed
  alongside implementation, not ahead of it.
- **Dot-role surface shape: open, resolved by task 6.** The parser resolves
  dot roles only as far as syntax allows (spec
  [§A.2.5](../../../spec/en/appendix_a.grammar_summary.md) "Dot
  disambiguation"): compound reserved tokens and registered user symbols are
  lexer-owned; selector-versus-namespace separation depends on variable scope
  and is finalized by the resolver. Decide the `SurfaceAst` shape that keeps
  unresolved dot chains syntactic, with `mizar-syntax` task 8.
- **Corpus runner location: open, resolved by task 3.** Parse-only corpus
  cases need real tokens, so the runner most likely drives the frontend real
  seam (precedent: `crates/mizar-frontend/tests/lexical_corpus.rs`); the
  alternative is a runner inside `mizar-test`. Decide and record.

## Ordered Task List

Each task is sized to be implemented, tested, and committed on its own. Keep
`cargo test -p mizar-parser` green after each task (see
[Recommended Verification](#recommended-verification)).

### Infrastructure

1. **Module split and lint-policy guard.** [ ]
   - Split `src/lib.rs` into `pub mod grammar;`, `pub mod pratt;`, and
     `pub mod recovery;`, moving task-11/12 code without behavior changes, and
     keep `parse`, `ParseRequest`, `ParserToken`, and `ParseOutput` reachable
     at their current paths.
   - Add `tests/lint_policy.rs` mirroring the `mizar-frontend` guard.
   - Tests: existing parser and frontend seam tests pass unchanged.
   - Deps: none. Spec: [grammar.md](./grammar.md).

2. **Parser infrastructure: cursor, expected-token diagnostics, synchronization.** [ ]
   - Add a token cursor with bounded lookahead, an expected-token diagnostic
     helper producing `SyntaxDiagnostic` with precise ranges, synchronization
     sets (`;`, `end`, top-level item keywords, EOF), and recovery-node
     emission helpers built on the `mizar-syntax` builder API.
   - Generalize the task-12 ad-hoc recovery (missing `end`, missing string
     literal, unrecoverable input) onto these helpers without changing
     observable behavior.
   - Tests: synchronization skips to each boundary kind and records skipped
     ranges; expected-token diagnostics carry the right primary range at EOF
     and mid-stream.
   - Deps: 1, `mizar-syntax` task 2. Spec: [recovery.md](./recovery.md).

3. **Parse-only corpus runner.** [ ]
   - Decide the runner location (frontend-seam integration test following the
     `lexical_corpus.rs` precedent, or a `mizar-test` runner), record the
     decision here and in [../../mizar-test/en/harness.md](../../mizar-test/en/harness.md)
     if it changes harness scope.
   - Wire `mizar-test` discovery and `.expect.toml` expectations to run every
     `tests/miz/{pass,fail}/parser/` case at stage `parse_only` through real
     tokenization, asserting outcome, diagnostics, and (where present)
     snapshot expectations.
   - Seed the corpus with cases for the current minimal grammar (token
     streams, explicit-fixity infix expressions, missing `end`, stray `end`)
     so the runner is meaningful from day one.
   - Tests: runner discovers all cases deterministically; a deliberately
     mismatched sidecar fails; seeded pass and fail cases enforce diagnostics.
   - Deps: 2. Spec: [staged_model.md](../../mizar-test/en/staged_model.md),
     [expectation_schema.md](../../mizar-test/en/expectation_schema.md).

### Grammar growth

Each grammar task follows the same template, in one change: re-derive the
EBNF from the owning spec chapter (brush up the spec if implementation exposes
gaps — English and Japanese together), add the paired `mizar-syntax` nodes,
implement the productions with synchronization and recovery, and ship unit
tests plus pass/fail corpus cases with `spec_trace.toml` entries per the
[Test Corpus Policy](#test-corpus-policy).

4. **Module skeleton and top-level item dispatch.** [ ]
   - Module file shape; `import` items with aliases and relative prefixes;
     export/visibility forms; top-level item dispatch by keyword with
     synchronization at item boundaries, so every later category drops into a
     stable skeleton.
   - Recovery: unknown top-level token skips to the next item keyword with a
     skipped-tokens node; missing `;` diagnosed at the next boundary.
   - Deps: 3, `mizar-syntax` task 6. Spec:
     [12.modules_and_namespaces.md](../../../spec/en/12.modules_and_namespaces.md).

5. **Type expressions.** [ ]
   - Attribute chains (with `non`), radix/mode type heads, `of`/`over`
     argument lists, struct-qualified attribute references. Term arguments
     enter through a term-entry stub until task 6 lands (types and terms are
     mutually recursive).
   - Deps: 4, `mizar-syntax` task 7. Spec:
     [03.type_system.md](../../../spec/en/03.type_system.md),
     [§A.3.2](../../../spec/en/appendix_a.grammar_summary.md).

6. **Primary terms and dot-role surface shape.** [ ]
   - Primary terms: identifiers, numerals, qualified symbols/namespace paths,
     parenthesized terms, application forms, selector access/update chains,
     Fraenkel/set-builder forms, `qua`. Resolve the dot-role surface-shape
     decision (see Resolved And Open Decisions) and record it in
     [grammar.md](./grammar.md) and the spec appendix.
   - Deps: 5, `mizar-syntax` task 8. Spec:
     [13.term_expression.md](../../../spec/en/13.term_expression.md),
     [§A.2.5](../../../spec/en/appendix_a.grammar_summary.md).

7. **Operator expressions (Pratt over the active lexicon).** [ ]
   - Generalize the task-11 explicit-fixity Pratt parser to user prefix,
     infix, and postfix operators driven by `ParserInputs` fixity metadata,
     with precedence and associativity per
     [appendix_b.operator_precedence.md](../../../spec/en/appendix_b.operator_precedence.md);
     diagnose non-associative chaining and dangling operators with
     source-local ranges.
   - Deps: 6. Spec: [pratt.md](./pratt.md),
     [13.term_expression.md](../../../spec/en/13.term_expression.md).

8. **Formulas.** [ ]
   - Fixed connective table, quantifiers (`for`/`ex` with `st`/`holds`),
     atomic predicate application, `is` formulas, attribute formulas;
     formula-level precedence stays separate from term-level fixity.
   - Deps: 7, `mizar-syntax` task 9. Spec:
     [14.formulas.md](../../../spec/en/14.formulas.md).

9. **Statements.** [ ]
   - `reserve`, `let`, `assume`, `take`, `consider`, `reconsider`, `set`,
     `given`, `thus`/`hence`, `then` chains, iterative equality `.=`,
     `per cases`/`suppose`, `now`/`hereby`.
   - Deps: 8, `mizar-syntax` task 10. Spec:
     [15.statements.md](../../../spec/en/15.statements.md).

10. **Theorems, proofs, and justifications.** [ ]
    - `theorem`/`lemma` items, labels, `proof … end` nesting, `by`/`from`
      justifications, citation forms including `.{ … }` grouped citations and
      `.*` bulk citations.
    - Deps: 9, `mizar-syntax` task 11. Spec:
      [16.theorems_and_proofs.md](../../../spec/en/16.theorems_and_proofs.md).

11. **Definitions.** [ ]
    - `definition … end` blocks: `attr`/`mode`/`pred`/`func` bodies,
      `means`/`equals`, `redefine`, `synonym`/`antonym`, correctness
      conditions, properties.
    - Deps: 10, `mizar-syntax` task 12. Spec:
      [06.attributes.md](../../../spec/en/06.attributes.md),
      [07.modes.md](../../../spec/en/07.modes.md),
      [09.predicates.md](../../../spec/en/09.predicates.md),
      [10.functors.md](../../../spec/en/10.functors.md).

12. **Structures.** [ ]
    - `struct` definitions: fields, inheritance/`extends`, selector
      declarations.
    - Deps: 11. Spec: [05.structures.md](../../../spec/en/05.structures.md).

13. **Registrations and clusters.** [ ]
    - `registration … end` blocks, cluster forms, `reduce`, related
      correctness conditions.
    - Deps: 12, `mizar-syntax` task 12. Spec:
      [17.clusters_and_registrations.md](../../../spec/en/17.clusters_and_registrations.md).

14. **Templates.** [ ]
    - Template parameters, bracket-form type arguments and parameter
      prefixes extending the task-5 productions.
    - Deps: 13, `mizar-syntax` task 13. Spec:
      [18.templates.md](../../../spec/en/18.templates.md).

15. **Algorithms.** [ ]
    - `algorithm` blocks and algorithmic statements: assignment, `while`,
      `if`, `match`, `break`/`continue`/`return`, `var`/`const`,
      `invariant`/`decreasing`/`terminating`, `assert`, `ghost`,
      `requires`/`ensures`.
    - Deps: 14. Spec:
      [20.algorithm_and_verification.md](../../../spec/en/20.algorithm_and_verification.md).

16. **Annotations.** [ ]
    - Statement-level annotations, `@[...]` library annotations, and
      string-literal annotation arguments (the string-required positions are
      already covered by the frontend lexing plan).
    - Deps: 15, `mizar-syntax` task 13. Spec:
      [21.source_code_annotation_and_atp.md](../../../spec/en/21.source_code_annotation_and_atp.md).

### Hardening and cross-cutting follow-ups

17. **Recovery consolidation and fail-corpus expansion.** [ ]
    - Audit recovery behavior across all categories: skipped-token nodes,
      unmatched delimiters, malformed annotations; close gaps where a category
      still aborts instead of synchronizing. Expand the fail corpus toward the
      recommended pass/fail mix.
    - Deps: 16. Spec: [recovery.md](./recovery.md),
      [architecture/en/20.test_strategy.md](../../architecture/en/20.test_strategy.md).

18. **`SurfaceAst` snapshot baselines.** [ ]
    - Add deterministic snapshot baselines under `tests/snapshots/` for
      representative corpus cases, using the `mizar-syntax` rendering (its
      task 3); wire snapshot comparison into the corpus runner.
    - Deps: 3, 16, `mizar-syntax` task 3. Spec:
      [../../mizar-test/en/snapshot.md](../../mizar-test/en/snapshot.md).

19. **Determinism property tests.** [ ]
    - Crate-level coverage that identical token streams produce identical
      `SurfaceAst` node orders, ranges, and diagnostic orders, mirroring the
      frontend determinism suite.
    - Deps: 16. Spec:
      [architecture/en/20.test_strategy.md](../../architecture/en/20.test_strategy.md).

20. **Parser fuzz target.** [ ]
    - Add a workspace fuzz target driving tokenization plus parsing over
      arbitrary UTF-8, asserting no panics and recoverable-diagnostics-only
      completion. This is the same trigger that re-opens `mizar-frontend`
      task 27; coordinate so the frontend target and this one land together.
    - Deps: 17. Spec: [recovery.md](./recovery.md),
      [../../mizar-frontend/en/todo.md](../../mizar-frontend/en/todo.md) task 27.

21. **Frontend passthrough follow-through.** [ ]
    - Grammar growth past the minimal seam re-opens `mizar-frontend` task 28:
      keep frontend recovery-marker passthrough, diagnostic merge order, and
      `SurfaceAstCacheKey` invalidation coverage in step with each grammar
      task, and flip the frontend `parsing`/`orchestration` statuses to `[x]`
      once the full grammar-recovery contract is in.
    - Deps: starts with 4, completes with 17. Spec:
      [../../mizar-frontend/en/todo.md](../../mizar-frontend/en/todo.md) task 28.

22. **Source/spec correspondence audit.** [ ]
    - Trace every public API and promised behavior in [grammar.md](./grammar.md),
      [pratt.md](./pratt.md), and [recovery.md](./recovery.md) to
      implementation and tests; record gaps as follow-up tasks.
    - Deps: 17. Spec: all module specs and this TODO.

23. **Bilingual documentation sync audit.** [ ]
    - Compare each English canonical document under
      `doc/design/mizar-parser/en/` with its Japanese companion and
      synchronize API lists, statuses, terminology, links, and behavior
      promises.
    - Deps: 22. Spec: repository documentation policy.

24. **Public enum forward-compatibility policy.** [ ]
    - Decide `#[non_exhaustive]` versus deliberate exhaustiveness for
      `ParserTokenKind`, `OperatorAssociativity`, `StringRequiredContext`, and
      any later public enums, aligned with the `mizar-frontend` task-25
      procedure and the `mizar-syntax` task-14 decisions.
    - Deps: 16. Spec: all module specs.

## Recommended Verification

Run after each task:

```text
cargo test -p mizar-parser
cargo test -p mizar-syntax
cargo clippy -p mizar-parser --all-targets -- -D warnings
```

For tasks that touch the frontend seam or the corpus runner, also run:

```text
cargo test -p mizar-frontend
cargo test -p mizar-test
```

Check the task off here once tests pass.

## Notes

- Parsing stays semantic-free: no name resolution, no type inference, no
  overload selection, no proof obligations. Dot roles are resolved only as
  far as syntax allows; the resolver finishes the job.
- The parser consumes frontend-adapted tokens only; it never re-lexes source
  text and never receives arbitrary lexer or resolver state.
- Grammar growth fires the `mizar-frontend` deferred-task triggers (27 fuzz,
  28 grammar-recovery follow-through); check that TODO when expanding
  recovery surfaces.
- Spec EBNF brush-up is part of each grammar task, not a separate workstream;
  fixes land in the owning chapter and appendix A, English and Japanese
  together.
