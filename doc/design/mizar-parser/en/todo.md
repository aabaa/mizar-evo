# mizar-parser TODO

> Canonical language: English. Japanese companion: [../ja/todo.md](../ja/todo.md).

## Status Legend

- [ ] not started
- [~] in progress
- [x] done

## Module Implementation

| Module | Spec | Source | Status |
|---|---|---|---|
| grammar | [grammar.md](./grammar.md) | `src/grammar.rs` | [~] task-14 formula connectives and quantifiers use private task-2 cursor/event infrastructure |
| pratt | [pratt.md](./pratt.md) | `src/pratt.rs` | [~] task-12 term Pratt over active prefix/postfix/infix operators is implemented; task-14 fixed formula Pratt is implemented separately from term fixity |
| recovery | [recovery.md](./recovery.md) | `src/recovery.rs` | [~] task-14 formula recovery plus mizar-frontend task-28 nested block-end matching uses task-2 cursor/diagnostic/sync helpers |

`mizar-parser` implements the syntax grammar: frontend-adapted tokens in,
`mizar_syntax::SurfaceAst` plus syntax diagnostics out. It is built as a thin
infrastructure layer (cursor, syntax-event/builder emission, synchronization,
recovery emission, corpus runner) followed by grammar growth a few productions
at a time. Each grammar task is paired with a `mizar-syntax` node-vocabulary
increment and a corpus expansion, and is deliberately sized so that one task
can be implemented, tested, and committed autonomously without holding the rest
of the grammar in flight.

## Crate Prerequisites

The crate depends on `mizar-session` and `mizar-syntax` only. Tokens arrive
already disambiguated by `mizar-frontend` with session `SourceRange`s;
parser-assisted lexing happens only through the precomputed
`ParserLexingPlan` / `StringRequiredContext` contract (resolved at the top
level, see [../../todo.md](../../todo.md) "Resolved And Open Decisions").
`ParseRequest` carries the operator fixity table and string-required context.
Task 12 exposes fixture lexical-summary fixity metadata so active parse-only
cases can exercise the frontend-visible source path without synthetic parser
inputs. The corpus harness (`mizar-test`) and the corpus tree
([tests/README.md](../../../../tests/README.md)) already exist.

`mizar-parser` should not depend on an ad hoc `SurfaceAst` arena layout. The
storage backend belongs to `mizar-syntax` task 2, whose target is rowan-backed
syntax; parser code must construct trees through the `mizar-syntax`
builder/event boundary and consume only documented accessors in tests. The
parser also stays `salsa`-free: later query layers can wrap
`ParseRequest -> ParseOutput` as a pure query only if this crate avoids global
state, hidden caches, and resolver/build-system dependencies.

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

Exception: a helper task whose production has no stand-alone grammar position
(for example task 4, qualified symbols) ships unit tests in its own change and
notes which later task delivers its corpus coverage; that later task must list
the helper's corpus cases explicitly.

Grow toward the recommended pass/fail mix of
[architecture/en/20.test_strategy.md](../../architecture/en/20.test_strategy.md)
(40% pass / 60% fail overall): every accepted form gets at least one
malformed counterpart that must be rejected or recovered with diagnostics, and
recovery cases assert both the diagnostic and the recovered `SurfaceAst`
shape, not just "did not crash".

## Review-Audit Parser Coverage Backlog

The grammar/VC review follow-up in `tests/coverage/spec_trace.toml` records
parser-facing cases that should become executable as their owning grammar tasks
land. Do not treat these as immediate coverage obligations before the
parse-only runner and the relevant productions exist.

- Template arguments: make
  `pass_parser_template_arguments_001` and
  `fail_parser_template_arguments_chained_iff_001` executable once definition,
  formula, predicate/functor, and template productions can parse them.
- Accepted syntax cases still needed: `let` constraints with `by` references,
  take-with-witness examples, conditional definiens, Fraenkel generators,
  `qua` chains, predicate chains, and template predicate/functor uses.
- Rejection cases still needed: non-associative operator chains,
  builtin/user predicate-chain mixing, and incomplete term-headed formulas.

## Resolved And Open Decisions

- **Parser-assisted lexing contract: resolved** at the top level. The parser
  never interleaves with the lexer; string-required positions and user-symbol
  kind filters arrive through the precomputed plan.
- **Grammar source of truth and brush-up protocol: resolved.** The chapter
  specs under [doc/spec/en/](../../../spec/en/00.index.md) are normative;
  [appendix_a.grammar_summary.md](../../../spec/en/appendix_a.grammar_summary.md)
  is the consolidated summary and is still being brushed up. Each grammar task
  starts by transcribing its production inventory into a named section of
  [grammar.md](./grammar.md) (English and Japanese in the same change); that
  section is the task's bounded normative reference. When implementation
  exposes a gap, ambiguity, or contradiction in the EBNF, fix the owning
  chapter and the appendix (English and Japanese together) as part of that
  task rather than deferring. Grammar brush-up proceeds alongside
  implementation, not ahead of it.
- **Dot-role surface shape: resolved for parser/syntax by task 10.** Also
  registered at the top level ([../../todo.md](../../todo.md) "Resolved And
  Open Decisions") because it spans `mizar-parser`, `mizar-syntax`, and the
  future resolver. The parser resolves dot roles only as far as syntax allows
  (spec [§A.2.5](../../../spec/en/appendix_a.grammar_summary.md) "Dot
  disambiguation"): compound reserved tokens and registered user symbols are
  lexer-owned, dotted qualified-name heads stay qualified surfaces, and `.`
  after an already parsed term becomes selector/update postfix syntax.
  Selector-versus-namespace separation that depends on variable scope remains
  resolver-owned.
- **Corpus runner location: resolved by task 3.** Parse-only corpus execution
  lives in `mizar-test`, which now deliberately owns the active runner in
  addition to discovery, expectation sidecars, traceability, and CLI reporting.
  The metadata `plan` path remains payload-free; only the `parse-only`
  subcommand depends on the frontend seam and session source loading.
- **Syntax-tree storage dependency: delegated to `mizar-syntax` task 2.** The
  parser's boundary is a builder/event API that can target rowan-backed syntax
  without letting grammar code depend on raw rowan node layout. Do not add a
  direct `rowan` dependency to `mizar-parser`; if grammar work needs a missing
  tree operation, add it to the `mizar-syntax` builder/accessor API first.
- **Salsa integration: deferred from this crate, required later.** `salsa` is
  required in the compiler's query and cache layers, not in `mizar-parser`.
  Keep parsing deterministic and side-effect-free so later build/frontend
  queries can cache `ParseOutput` without changing grammar code.

## Ordered Task List

Each task is sized to be implemented, tested, and committed on its own. Keep
`cargo test -p mizar-parser` green after each task (see
[Recommended Verification](#recommended-verification)).

### Infrastructure

1. **Module split and lint-policy guard.** [x]
   - Split `src/lib.rs` into internal `grammar`, `pratt`, and `recovery`
     implementation modules, moving task-11/12 code without behavior changes,
     and keep `parse`, `ParseRequest`, `ParserToken`, and `ParseOutput`
     reachable at their current crate-root paths. Keep these modules private
     until a later task intentionally exposes module-level parser APIs.
   - Add `tests/lint_policy.rs` mirroring the `mizar-frontend` guard for
     workspace lint opt-in, the shared rustc/clippy baseline, and documented
     inline rationales for intentional `allow` attributes in parser Rust target
     files. This task does not add the later parser public-enum
     forward-compatibility or rustdoc policy gates.
   - Tests: existing parser and frontend seam tests pass unchanged.
   - Deps: none. Spec: [grammar.md](./grammar.md).

2. **Parser infrastructure: cursor, syntax events, expected-token diagnostics, synchronization.** [x]
   - Add a token cursor with bounded lookahead, an expected-token diagnostic
     helper producing `SyntaxDiagnostic` with precise ranges, a syntax-event
     sink that feeds the `mizar-syntax` builder, synchronization sets (`;`,
     `end`, top-level item keywords, EOF), and recovery-node emission helpers
     built on the `mizar-syntax` builder API.
   - Keep grammar code independent of the concrete `SurfaceAst` storage
     backend: no direct pushes into a syntax arena, no reliance on dense node
     indices, and no raw rowan traversal. Missing construction or inspection
     operations must be added to `mizar-syntax` as documented builder/accessor
     APIs before parser grammar code uses them.
   - Generalize the task-12 recovery plus mizar-frontend task-28 block-stack
     matching (missing `end`, missing string literal, unrecoverable input,
     contextual block openers) onto these helpers without changing observable
     behavior.
   - Tests: synchronization skips to each boundary kind and records skipped
     ranges; expected-token diagnostics carry the right primary range at EOF
     and mid-stream.
   - Initial top-level item synchronization keywords are `theorem`,
     `definition`, `registration`, `notation`, `scheme`, `reserve`, `begin`,
     `environ`, `vocabularies`, `constructors`, and `requirements`; later item
     grammar tasks may refine this placeholder when they add real dispatch.
   - Deps: 1, `mizar-syntax` task 2. Spec: [recovery.md](./recovery.md).

3. **Parse-only corpus runner.** [x]
   - Runner location: `mizar-test` owns the parse-only runner because it already
     owns corpus discovery, expectation sidecars, traceability, and CLI
     reporting. The runner depends on `mizar-frontend` and `mizar-session` only
     for this parse-only execution path; the metadata `plan` mode remains
     payload-free.
   - `mizar-test parse-only` discovers expectations through the normal plan and
     runs active `.miz` cases tagged `active_parse_only` at
     `stage = "parse_only"` / `expected_phase = "parse"` through real
     tokenization and `MizarParserSeam`. Inactive planned grammar seeds remain
     discovery and traceability metadata.
   - Active seed coverage now includes current frontend-reachable parser
     behavior: token stream preservation, missing `end`, stray `end`, and,
     beginning with task 12, source-path operator fixity supplied by fixture
     lexical summaries.
   - The committed template-argument seed cases stay out of the active runner
     until tasks 14, 23-25, and 31 can parse their formula, definition, and
     template forms.
   - Tests: active/inactive discovery is deterministic; active-tag mistakes are
     harness errors; a deliberately mismatched sidecar fails; seeded pass and
     fail cases enforce diagnostics; the `parse-only` CLI reports the active
     runner summary.
   - Deps: 2. Spec: [staged_model.md](../../mizar-test/en/staged_model.md),
     [expectation_schema.md](../../mizar-test/en/expectation_schema.md).

### Pre-resolver compatibility gate

**Initial public enum forward-compatibility gate.** [x]
- Decide `#[non_exhaustive]` versus deliberate exhaustiveness for the parser
  public enums that already exist at the phase-3 boundary
  (`ParserTokenKind`, `OperatorAssociativity`, `StringRequiredContext`), using
  the `mizar-frontend` task-25 procedure and the initial `mizar-syntax` gate.
  Later public enums such as task-12 `OperatorFixity` must be classified by the
  same gate.
- Record each decision in the owning module spec and apply the attributes before
  parser tasks 5-7 can become resolver/LSP inputs.
- Result: `ParserTokenKind` and `StringRequiredContext` are
  `#[non_exhaustive]` for downstream crates; `OperatorAssociativity` and the
  task-12 `OperatorFixity` are documented exhaustive exceptions.
  `crates/mizar-parser/tests/lint_policy.rs` guards the classification for
  every current public parser enum.
- Deps: 3 and the initial `mizar-syntax` public-enum gate. Spec:
  [grammar.md](./grammar.md), [pratt.md](./pratt.md),
  [recovery.md](./recovery.md).

### Grammar growth

Each grammar task follows the same template, in one change:

1. transcribe the task's production inventory from the owning spec chapter
   into a named section of [grammar.md](./grammar.md) (English and Japanese
   together), brushing up the chapter and
   [appendix A](../../../spec/en/appendix_a.grammar_summary.md) if the
   transcription exposes gaps;
2. add the paired `mizar-syntax` node increment through the documented
   rowan-backed builder/accessor boundary;
3. implement the productions with synchronization and recovery;
4. ship unit tests plus pass/fail corpus cases with `spec_trace.toml` entries
   per the [Test Corpus Policy](#test-corpus-policy).

In dependency lines, a reference such as `mizar-syntax task 11 / S-011` means
the specific node-vocabulary increment required by that parser task, not
completion of the whole syntax vocabulary bucket. When the crate-plan S-task and
older numeric syntax task references appear to disagree, prefer
`doc/design/mizar-syntax/en/00.crate_plan.md`.

4. **Qualified symbols and namespace paths.** [x]
   - A shared helper for `qualified_symbol = { namespace_segment "." }
     user_symbol` and dotted module paths, used later by imports, type heads,
     terms, and citations. Path shapes only; variable shadowing stays
     resolver-side.
   - Result: task-4 production inventory is recorded in [grammar.md](./grammar.md),
     and shared helpers now emit `ModulePath`, `NamespacePath`,
     `QualifiedSymbol`, `PathSegment`, and `RelativePrefix` syntax nodes with
     unit coverage. Corpus coverage remains with consuming tasks 6 and 8, as
     planned.
   - Corpus exception: unit tests here; corpus coverage lands with the first
     consuming positions (tasks 6 and 8) and must be listed there.
   - Deps: 3, `mizar-syntax` task 9 / S-009 (shared path-node increment). Spec:
    [12.modules_and_namespaces.md](../../../spec/en/12.modules_and_namespaces.md)
    §12.7, [Appendix A](../../../spec/en/appendix_a.grammar_summary.md) A.3/A.12/A.15,
    and [Chapter 2](../../../spec/en/02.lexical_structure.md) §2.5.3/§2.8.

5. **Module skeleton and top-level item dispatch.** [x]
   - Module file shape and top-level item dispatch by keyword with
     synchronization at item boundaries, so every later category drops into a
     stable skeleton.
   - Recovery: unknown top-level token skips to the next item keyword with a
     skipped-tokens node; missing `;` diagnosed at the next boundary.
   - Result: task-5 production inventory is recorded in [grammar.md](./grammar.md).
     The parser now emits `CompilationUnit`, `ItemList`, and `PlaceholderItem`
     syntax nodes, preserves legacy no-item token streams with an empty item
     list, emits `SkippedToken` recovery plus skipped-range trivia for
     unexpected top-level input, diagnoses missing item semicolons, and ships
     active parse-only pass/fail corpus coverage with traceability.
   - Deps: 3, `mizar-syntax` task 9 / S-009. Spec:
     [12.modules_and_namespaces.md](../../../spec/en/12.modules_and_namespaces.md).

6. **Import items.** [x]
   - `import` items with aliases and relative prefixes (`.`/`..`); shapes stay
     consistent with the frontend import-prescan stubs. Includes the deferred
     corpus cases for task 4 path shapes.
   - Result: task-6 production inventory is recorded in
     [grammar.md](./grammar.md). The parser now emits `ImportItem`,
     `ImportAliasDecl`, and `ModuleBranchImport` syntax nodes under the module
     `ItemList`, uses shared `ModulePath` / `RelativePrefix` / `PathSegment`
     nodes for import paths and aliases, keeps imports concrete only while the
     import prelude is open, recovers late imports with
     `UnexpectedTopLevelToken`, diagnoses malformed alias/branch syntax with
     `MalformedImport`, and ships active parse-only pass/fail corpus coverage
     with traceability. `mizar-test` parse-only runs now resolve import stubs
     to empty syntax-only summaries so import syntax can be tested without
     semantic module availability.
   - Deps: 4, 5, `mizar-syntax` task 9 / S-009. Spec:
     [12.modules_and_namespaces.md](../../../spec/en/12.modules_and_namespaces.md).

7. **Export and visibility items.** [x]
   - Export forms and `public`/`private` visibility markers on items, per the
     module chapter.
   - Result: task-7 production inventory is recorded in [grammar.md](./grammar.md).
     The parser now emits `ExportItem`, `VisibilityMarker`, and `VisibleItem`
     syntax nodes, keeps exports concrete only while the export prelude is
     open, recovers late exports with `UnexpectedTopLevelToken`, diagnoses
     malformed export path lists with `MalformedExport`, diagnoses duplicate or
     invalid visibility prefixes with `MalformedVisibility`, preserves
     annotation-prefix token order inside visible wrappers, and ships active
     parse-only pass/fail corpus coverage with traceability.
   - Deps: 5, `mizar-syntax` task 9 / S-009. Spec:
     [12.modules_and_namespaces.md](../../../spec/en/12.modules_and_namespaces.md).

8. **Type expressions.** [x]
   - Attribute chains (with `non`), radix/mode type heads, `of`/`over`
     argument lists, struct-qualified attribute references. Term arguments
     enter through a term-entry stub until task 9 lands (types and terms are
     mutually recursive). Includes the deferred corpus cases for task 4
     qualified type heads. Tests pin the rightmost attribute/type-head split,
     bracket nested type-expression arguments, bracket `qua_arg` placeholders,
     malformed type arguments, and local parameter-prefix preservation when the
     incoming tokens expose the split.
   - Deps: 4, 5, `mizar-syntax` task 10 / S-010. Spec:
     [03.type_system.md](../../../spec/en/03.type_system.md),
     [§A.3.2](../../../spec/en/appendix_a.grammar_summary.md).
   - Result: implemented concrete top-level `reserve` parsing with
     `ReserveItem`/`ReserveSegment`, syntax-only `TypeExpression`,
     `AttributeChain`, `AttributeRef`, generic `TypeHead`, `TypeArguments`, and
     `TermPlaceholder` nodes; added `MalformedTypeExpression` recovery, parser
     unit coverage for local parameter-prefix token splits, and active
     parse-only pass/fail corpus coverage through `parser.type_fixtures`.

9. **Primary terms.** [x]
   - Identifiers, numerals, qualified symbols in term position, parenthesized
     terms, `it`, choice expressions (`the type_expression`), structure
     constructors with named field arguments, set enumeration literals, and
     application forms; replace the task-8 term-entry stub.
   - Deps: 8, `mizar-syntax` task 11 / S-011. Spec:
     [13.term_expression.md](../../../spec/en/13.term_expression.md).
   - Result: implemented syntax-only `TermExpression` primary terms reachable
     through reserve-hosted `of` / `over` arguments and attribute argument
     lists, replaced task-8 term placeholders in those positions, kept bracket
     `type_arg_list` `qua_arg` placeholders for task 11, added
     `MalformedTermExpression` / `MissingTerm` / term-delimiter recovery, and
     shipped parser unit tests plus active parse-only pass/fail corpus coverage.

10. **Selector access/update and the dot-role surface shape.** [x]
    - Selector access and selector-call chains (`p.x`, `line.finish.y`,
      `M.binop(x, y)`), functional structure updates (`p with (...)`), and the
      syntax-only dot-role representation. Introduce the selector-update
      surface vocabulary, but keep standalone in-place assignments such as
      `p.x := t` with their later statement/algorithm hosts. Resolve the dot-role
      surface-shape decision (see Resolved And Open Decisions) and record it
      in [grammar.md](./grammar.md), the spec appendix, and the top-level
      decision list.
    - Deps: 9, `mizar-syntax` task 11 / S-011. Spec:
      [13.term_expression.md](../../../spec/en/13.term_expression.md),
      [§A.2.5](../../../spec/en/appendix_a.grammar_summary.md).
   - Result: implemented syntax-only `SelectorAccess`, `StructureUpdate`, and
     `FieldUpdate` surfaces as term postfix chains, including selector-call
     argument lists, left-associative selector nesting, functional structure
     update lists, `MalformedTermExpression` recovery for malformed selector /
     update syntax, `MissingTerm` for omitted update values, active parse-only
     pass/fail corpus coverage, and traceability entries. Standalone
     `p.x := t` remains assigned to later statement/algorithm hosts.

11. **`qua` qualification.** [x]
    - `term qua type_expression` with precedence against selector and
      application forms. Parse selector/update/application shapes before `qua`,
      fold `qua` chains left-associatively, preserve the
      `x qua Element of S qua Magma` target-type argument binding, and replace
      bracket `qua_arg` `TermPlaceholder` stubs with the task-11
      `TermExpression` / `QuaExpression` surface while respecting the narrower
      Appendix-A `qua_arg ::= identifier { "qua" radix_type }` shape.
      Missing or malformed `qua` targets use `MalformedTypeExpression` plus
      `MissingTypeExpression` or skipped-tail recovery under `QuaExpression`;
      tests must include selector precedence, parenthesized
      selector-after-`qua`, bracket `qua_arg` migration, left-associative
      chains, target recovery, and active parse-only pass/fail traceability.
    - Deps: 8, 9, `mizar-syntax` task 11 / S-011. Spec:
      [13.term_expression.md](../../../spec/en/13.term_expression.md).
   - Result: implemented syntax-only `QuaExpression` surfaces after
     selector/update postfix parsing, left-associative `qua` chains, target
     type-expression parsing with nested term-argument `qua` binding, bracket
     `qua_arg` migration from `TermPlaceholder` to `TermExpression` /
     `QuaExpression`, `MalformedTypeExpression` recovery with
     `MissingTypeExpression` or skipped target tails, parser unit tests, active
     parse-only pass/fail corpus coverage, and traceability entries.

12. **Operator expressions (Pratt over the active lexicon).** [x]
    - Generalize the task-11 explicit-fixity Pratt parser to user prefix,
      infix, and postfix operators driven by `ParserInputs` fixity metadata,
      with precedence and associativity per
      [appendix_b.operator_precedence.md](../../../spec/en/appendix_b.operator_precedence.md);
      diagnose non-associative chaining and dangling operators with
      source-local ranges.
    - Deps: 10, 11, `mizar-syntax` task 11 / S-011 (operator-node increment). Spec:
      [pratt.md](./pratt.md),
      [13.term_expression.md](../../../spec/en/13.term_expression.md).
   - Result: implemented parser-facing `OperatorFixity` metadata,
     summary-derived `ParserInputs` transfer through the frontend seam,
     module-term Pratt parsing for prefix/postfix/infix operators before
     `qua`, `PrefixExpression` / `PostfixExpression` syntax surfaces,
     non-associative and dangling-operator diagnostics, parser unit tests,
     active parse-only pass/fail corpus coverage, and traceability entry
     `spec.en.13.operator_precedence.parser`.

13. **Atomic formulas.** [x]
    - Predicate application (symbolic and identifier forms), built-in
      membership/equality/inequality atoms, and generic `is_assertion` forms
      that resolution later classifies as type or attribute assertions.
    - Deps: 12, `mizar-syntax` task 12 / S-012. Spec:
      [14.formulas.md](../../../spec/en/14.formulas.md).
   - Result: implemented task-13 `FormulaExpression`,
     `BuiltinPredicateApplication`, generic `IsAssertion`,
     `AttributeTestChain`, `PredicateApplication`/`PredicateSegment`/
     `PredicateHead`, and `InlinePredicateApplication` surfaces; theorem/lemma
     `label: formula;` placeholder hosting; term/type recovery for malformed
     atomic formulas; parser unit tests; active parse-only pass/fail corpus
     coverage; and traceability entry `spec.en.14.atomic_formula.parser`.
     Built-in predicates remain single atoms and mixed user/built-in predicate
     chains are rejected.

14. **Connectives and quantifiers.** [x]
    - The fixed connective table (`not`, `&`, `or`, `implies`, `iff`) with its
      formula-level precedence, kept separate from term-level fixity;
      quantifiers `for`/`ex` with `st`/`holds` bodies.
    - Deps: 13, `mizar-syntax` task 12 / S-012. Spec: [14.formulas.md](../../../spec/en/14.formulas.md),
      [appendix_b.operator_precedence.md](../../../spec/en/appendix_b.operator_precedence.md).
   - Result: implemented task-14 `PrefixFormula`, `BinaryFormula`,
     `ParenthesizedFormula`, `QuantifiedFormula`,
     `QuantifierVariableSegment`, and `FormulaConstant` surfaces; fixed
     formula connective precedence and `iff` non-associativity diagnostics;
     `MissingFormula` / `MalformedFormulaExpression` recovery after formula
     operators and quantifier bodies; parser unit tests; active parse-only
     pass/fail corpus coverage; and traceability entry
     `spec.en.14.formula_connectives_quantifiers.parser`. Template predicate
     arguments remain deferred to task 31 / S-016, and Fraenkel/set-builder
     terms that embed formulas remain task 15.

15. **Fraenkel and set-builder terms.** [x]
    - `{ term where … : formula }` and related set-builder/comprehension forms,
      including the omitted-condition form; placed after formulas because the
      separator clause embeds a formula. Set enumeration literals are covered by
      task 9.
    - Deps: 14, `mizar-syntax` task 11 / S-011 (Fraenkel-node increment). Spec:
      [13.term_expression.md](../../../spec/en/13.term_expression.md).
    - Result: implemented task-15 `SetComprehension` and
      `ComprehensionVariableSegment` surfaces, top-level `where`
      disambiguation from `SetEnumeration`, optional condition formula parsing
      through the task-14 formula parser, generator type recovery, missing
      condition recovery, missing brace recovery, parser unit tests, active
      parse-only pass/fail corpus coverage, traceability under
      `spec.en.13.set_expressions.parser`, and the scope-skeleton guard that
      keeps expression-level `is set` type words from being reported as
      malformed `set name =` binder statements.

16. **Simple statements.** [x]
    - `let`, `assume`, `take`, `set`, `given` — the statement forms that carry
      no justification clause. `reserve` remains the existing top-level
      `ReserveItem` path because Chapter 4 forbids block-local
      `reserve`-shaped statements; keep that path covered as a non-regression.
    - Deps: 14, `mizar-syntax` task 13 / S-013. Spec:
      [15.statements.md](../../../spec/en/15.statements.md).
    - Implemented as `StatementItem`-hosted simple statements with
      `QualifiedVariableSegment`, `ConditionList`, `Proposition`, `Witness`,
      and `Equating` children; `let ... by ...` remains a task-17 placeholder
      boundary. Unit tests and active parse-only pass/fail corpus coverage are
      in place for happy paths, multiple set equatings, proposition labels,
      recovery nodes, skipped tails, semicolon-boundary synchronization, and
      the top-level `ReserveItem` non-regression.

17. **Justifications and citations.** [ ]
    - `by`/`from` justification clauses, citation lists, `.{ … }` grouped
      citations, `.*` bulk citations, and the compact justified statement
      (`φ by A;`), including `by computation(...)` options from the algorithm
      chapter.
    - Deps: 16, `mizar-syntax` task 14 / S-014 (justification-node increment). Spec:
      [16.theorems_and_proofs.md](../../../spec/en/16.theorems_and_proofs.md) §16.5,
      [20.algorithm_and_verification.md](../../../spec/en/20.algorithm_and_verification.md)
      §20.9.2.

18. **`consider` and `reconsider`.** [ ]
    - `consider … such that … by …` and `reconsider … as … by …`, both of
      which carry justifications.
    - Deps: 17, `mizar-syntax` task 13 / S-013. Spec: [15.statements.md](../../../spec/en/15.statements.md).

19. **Conclusion steps and iterative equality.** [ ]
    - `thus`/`hence`, `then` chains, and iterative equality `.=` steps with
      their per-step justifications. Include the grammar-audit boundary between
      compact equality statements and zero-step iterative equality (`x = y by
      A;` versus `x = y by A .= z by B;`).
    - Deps: 17, `mizar-syntax` task 13 / S-013. Spec: [15.statements.md](../../../spec/en/15.statements.md).

20. **Block statements.** [ ]
    - `now`/`hereby` blocks and `per cases`/`suppose`/`case` blocks with their
      `end` synchronization.
    - Deps: 19, `mizar-syntax` task 13 / S-013. Spec: [15.statements.md](../../../spec/en/15.statements.md).

21. **Local definitions.** [ ]
    - `deffunc`/`defpred` private local definitions.
    - Deps: 20, `mizar-syntax` task 13 / S-013. Spec: [15.statements.md](../../../spec/en/15.statements.md).

22. **Theorems and proofs.** [ ]
    - `theorem`/`lemma` items, labels, `proof … end` nesting, and proof-body
      statement wiring.
    - Deps: 21, `mizar-syntax` task 14 / S-014. Spec:
      [16.theorems_and_proofs.md](../../../spec/en/16.theorems_and_proofs.md).

23. **Definition block skeleton, correctness conditions, and attribute definitions.** [ ]
    - `definition … end` block shape shared by all definition kinds, the
      correctness-condition clause shape (`existence`, `uniqueness`,
      `coherence`, `consistency`, `compatibility`, … with justifications), and
      `attr` definitions as the first concrete kind.
    - Deps: 22, `mizar-syntax` task 15 / S-015. Spec:
      [06.attributes.md](../../../spec/en/06.attributes.md).

24. **Predicate definitions.** [ ]
    - `pred` definitions with `means` bodies.
    - Deps: 23, `mizar-syntax` task 15 / S-015. Spec: [09.predicates.md](../../../spec/en/09.predicates.md).

25. **Functor definitions.** [ ]
    - `func` definitions with `means`/`equals` bodies.
    - Deps: 23, `mizar-syntax` task 15 / S-015. Spec: [10.functors.md](../../../spec/en/10.functors.md).

26. **Mode definitions.** [ ]
    - `mode` definitions using the canonical `is` form: attribute-chain plus
      radix type, type parameters, and optional `sethood` property clauses.
    - Deps: 23, `mizar-syntax` task 15 / S-015. Spec: [07.modes.md](../../../spec/en/07.modes.md).

27. **`redefine`, `synonym`, and `antonym`.** [ ]
    - Redefinition and notation-aliasing forms across the definition kinds of
      tasks 23-26.
    - Deps: 24, 25, 26, `mizar-syntax` task 15 / S-015. Spec:
      [06.attributes.md](../../../spec/en/06.attributes.md),
      [07.modes.md](../../../spec/en/07.modes.md),
      [09.predicates.md](../../../spec/en/09.predicates.md),
      [10.functors.md](../../../spec/en/10.functors.md),
      [11.symbol_management.md](../../../spec/en/11.symbol_management.md).

28. **Property clauses.** [ ]
    - Property clauses across definition kinds (`commutativity`,
      `idempotence`, `involutiveness`, `projectivity`, `reflexivity`,
      `irreflexivity`, `symmetry`, `asymmetry`, `connectedness`,
      `transitivity`, `sethood`, …) with justifications.
    - Deps: 27, `mizar-syntax` task 15 / S-015. Spec: [06.attributes.md](../../../spec/en/06.attributes.md),
      [07.modes.md](../../../spec/en/07.modes.md),
      [09.predicates.md](../../../spec/en/09.predicates.md),
      [10.functors.md](../../../spec/en/10.functors.md).

29. **Structures.** [ ]
    - `struct` definitions: fields, inheritance/`extends`, selector
      declarations.
    - Deps: 28, `mizar-syntax` task 15 / S-015. Spec:
      [05.structures.md](../../../spec/en/05.structures.md).

30. **Registrations and clusters.** [ ]
    - `registration … end` blocks, existential/conditional/functorial cluster
      forms, `reduce`, and their correctness conditions.
    - Deps: 29, `mizar-syntax` task 15 / S-015. Spec:
      [17.clusters_and_registrations.md](../../../spec/en/17.clusters_and_registrations.md).

31. **Templates.** [ ]
    - Template parameters, bracket-form type arguments and parameter prefixes
      extending the task-8 productions, `nest` forms.
    - Promote the review-audit seed cases
      `tests/miz/pass/parser/pass_parser_template_arguments_001.*` and
      `tests/miz/fail/parser/fail_parser_template_arguments_chained_iff_001.*`
      from traceability metadata into runner-executed parse-only coverage.
    - Deps: 30, `mizar-syntax` task 16 / S-016. Spec:
      [18.templates.md](../../../spec/en/18.templates.md).

32. **Algorithm blocks, assignments, declarations, and claims.** [ ]
    - `algorithm` block shape, assignment statements, `var`/`const`
      declarations, `ghost var`/`ghost const`, ghost assignments, `snapshot`,
      top-level `claim` blocks, and `return` statements with optional
      justifications.
    - Deps: 31, `mizar-syntax` task 16 / S-016. Spec:
      [20.algorithm_and_verification.md](../../../spec/en/20.algorithm_and_verification.md).

33. **Algorithm control flow.** [ ]
    - `while`/`do` (with `to`/`downto`), `if`/`else`, `match`,
      `for ... in ... processed ...`, `otherwise`/`exhaustive` match endings,
      `break`/`continue`.
    - Deps: 32, `mizar-syntax` task 16 / S-016. Spec:
      [20.algorithm_and_verification.md](../../../spec/en/20.algorithm_and_verification.md).

34. **Algorithm verification clauses.** [ ]
    - Header and loop verification clauses: `requires`/`ensures`,
      `decreasing`, `terminating`, `invariant`, `assert`, and their
      justifications.
    - Deps: 33, `mizar-syntax` task 16 / S-016. Spec:
      [20.algorithm_and_verification.md](../../../spec/en/20.algorithm_and_verification.md).

35. **Annotations.** [ ]
    - Statement-level annotations, `@[...]` library annotations, and
      string-literal annotation arguments (the string-required positions are
      already covered by the frontend lexing plan).
    - Deps: 34, `mizar-syntax` task 16 / S-016. Spec:
      [21.source_code_annotation_and_atp.md](../../../spec/en/21.source_code_annotation_and_atp.md).

### Hardening and cross-cutting follow-ups

36. **Recovery consolidation and fail-corpus expansion.** [ ]
    - Audit recovery behavior across all categories: skipped-token nodes,
      unmatched delimiters, malformed annotations; close gaps where a category
      still aborts instead of synchronizing. Expand the fail corpus toward the
      recommended pass/fail mix.
    - Deps: 35. Spec: [recovery.md](./recovery.md),
      [architecture/en/20.test_strategy.md](../../architecture/en/20.test_strategy.md).

37. **`SurfaceAst` snapshot baselines.** [ ]
    - Add deterministic snapshot baselines under `tests/snapshots/` for
      representative corpus cases, using the `mizar-syntax` rendering (its
      task 3); wire snapshot comparison into the corpus runner.
    - Deps: 3, 35, `mizar-syntax` task 3. Spec:
      [../../mizar-test/en/snapshot.md](../../mizar-test/en/snapshot.md).

38. **Determinism property tests.** [ ]
    - Crate-level coverage that identical token streams produce identical
      `SurfaceAst` node orders, ranges, and diagnostic orders, mirroring the
      frontend determinism suite.
    - Deps: 35. Spec:
      [architecture/en/20.test_strategy.md](../../architecture/en/20.test_strategy.md).

39. **Parser fuzz target.** [ ]
    - Add a workspace fuzz target driving tokenization plus parsing over
      arbitrary UTF-8, asserting no panics and recoverable-diagnostics-only
      completion. The `mizar-frontend` task 29 real-parser fuzz follow-up has
      landed the frontend-owned target; this task tracks the parser-owned
      counterpart.
    - Deps: 36. Spec: [recovery.md](./recovery.md),
      [../../mizar-frontend/en/todo.md](../../mizar-frontend/en/todo.md) task 29.

40. **Frontend passthrough follow-through.** [ ]
    - Grammar growth past the current mizar-frontend task-28 parser-recovery
      surface opens a new `mizar-frontend` follow-up:
      keep frontend recovery-marker passthrough, diagnostic merge order, and
      `SurfaceAstCacheKey` invalidation coverage in step with each grammar
      task.
    - Deps: starts with 5, completes with 36. Spec:
      [../../mizar-frontend/en/todo.md](../../mizar-frontend/en/todo.md).

41. **Source/spec correspondence audit and reserved-word coverage.** [ ]
    - Trace every public API and promised behavior in [grammar.md](./grammar.md),
      [pratt.md](./pratt.md), and [recovery.md](./recovery.md) to
      implementation and tests; record gaps as follow-up tasks.
    - Verify that every reserved word of
      [§A.2.4](../../../spec/en/appendix_a.grammar_summary.md) is consumed by
      at least one parser corpus test (or is explicitly recorded as
      reserved-for-future with no grammar position yet), so silently
      unimplemented keywords are detected mechanically.
    - Deps: 36. Spec: all module specs and this TODO.

42. **Bilingual documentation sync audit.** [ ]
    - Compare each English canonical document under
      `doc/design/mizar-parser/en/` with its Japanese companion and
      synchronize API lists, statuses, terminology, links, and behavior
      promises.
    - Deps: 41. Spec: repository documentation policy.

43. **Public enum forward-compatibility policy.** [ ]
    - Revisit the initial public-enum gate after task 35 and decide
      `#[non_exhaustive]` versus deliberate exhaustiveness for any later public
      enums added by grammar growth, aligned with the `mizar-frontend` task-25
      procedure and the `mizar-syntax` task-17 final audit.
    - Deps: 35. Spec: all module specs.

## Recommended Verification

Run after each task:

```text
cargo test -p mizar-parser
cargo test -p mizar-syntax
cargo fmt --check
cargo clippy -p mizar-parser --all-targets --all-features -- -D warnings
cargo clippy -p mizar-syntax --all-targets --all-features -- -D warnings
```

For every grammar task after the parse-only runner lands, also run the crate
that owns the parse-only corpus runner selected in task 3 and validate
`mizar-test` expectations/discovery. If the default frontend-seam runner is
selected, that means:

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
- The parser emits syntax through `mizar-syntax` builder/event APIs. Grammar
  code should not depend on custom arena indices or raw rowan layout.
- `salsa` is a later query/cache layer concern. Preserve deterministic,
  side-effect-free parsing so `ParseRequest -> ParseOutput` can become a query
  without rewriting grammar tasks.
- Grammar growth after the current mizar-frontend task-28 parser-recovery
  surface should open a new `mizar-frontend` follow-up for fuzz coverage,
  recovery-marker passthrough, diagnostic merge ordering, and
  `SurfaceAstCacheKey` invalidation.
- Spec EBNF brush-up is part of each grammar task, not a separate workstream;
  the production inventory transcribed into [grammar.md](./grammar.md) is each
  task's bounded contract, and fixes land in the owning chapter and appendix
  A, English and Japanese together.
