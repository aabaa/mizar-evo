# mizar-syntax TODO

> Canonical language: English. Japanese companion: [../ja/todo.md](../ja/todo.md).

## Status Legend

- [ ] not started
- [~] in progress
- [x] done

## Module Implementation

| Module | Spec | Source | Status |
|---|---|---|---|
| ast | [ast.md](./ast.md) | `src/ast.rs` | [~] minimal task-12 surface split into module |
| trivia | [trivia.md](./trivia.md) | `src/trivia.rs` | [ ] |
| recovery | [recovery.md](./recovery.md) | `src/recovery.rs` | [~] minimal task-12 recovery kinds split into module |

`mizar-syntax` is a data-definition crate: it owns the `SurfaceAst` shape shared
by `mizar-parser`, `mizar-frontend`, and future resolver/LSP/formatter
consumers, and it owns no parsing logic and no semantics. The target syntax-tree
backend is `rowan`: the early representation work must adopt a rowan-backed
green tree and may keep only compatibility wrappers needed to preserve the
current minimal boundary during the migration. It is built in two waves: first
the representation foundation (rowan storage boundary, rendering, trivia,
recovery vocabulary), then the node vocabulary, which grows in lockstep with
`mizar-parser` grammar tasks.

Dependency order: `ast` foundation → `trivia` / `recovery` → node vocabulary
paired with `mizar-parser`.

## Crate Prerequisites

The crate currently depends only on `mizar-session` (for `SourceId`,
`SourceRange`, `SourceAnchor`). Task 2 adds `rowan`; if the task-11/12
`SurfaceNode`-style API needs a transition path, keep it as a compatibility
wrapper over the rowan-backed representation rather than deferring the backend
choice. Do not add `salsa` here: the query engine belongs to the
frontend/build/resolver/checker layers, while this crate must stay an
immutable, query-friendly syntax data boundary. The task-11/12 minimal boundary
(`SurfaceAst`, `SurfaceNode`, recovery kinds, `SyntaxDiagnostic`) is already
consumed by `mizar-parser` and `mizar-frontend::parsing::MizarParserSeam`;
every change here must keep `cargo test -p mizar-parser` and
`cargo test -p mizar-frontend` green in the same change.

## Resolved And Open Decisions

- **Syntax-tree backend: open, resolved by task 2.** `rowan` is the required
  target backend for `SurfaceAst`. The default decision is to introduce a
  rowan-backed green tree in task 2. Compatibility wrappers for existing
  task-11/12 names may remain, but parser tasks 5-7 must not grow against a
  custom arena backend.
- **Trivia ownership: open, resolved by task 4.** `mizar-frontend` already
  extracts comments and doc comments into `PreprocessedSource`; decide whether
  `SurfaceAst` carries attached trivia, references frontend-owned trivia by
  range, or stores only attachment hints.
- **Salsa integration: deferred from this crate, required later.** `salsa` is
  required in the compiler's query and cache layers, not in `mizar-syntax`.
  Keep `SurfaceAst` immutable,
  deterministic, and cheap to share so later `salsa` queries can use it as an
  output value without changing this crate's semantic-free boundary.
- **Dot-role surface shape: open, owned by `mizar-parser` task 10.** The
  parser cannot fully separate selector access from namespace separation
  without the resolver (spec
  [§A.2.5](../../../spec/en/appendix_a.grammar_summary.md)); the AST must
  represent unresolved dot chains syntactically. Also registered at the top
  level ([../../todo.md](../../todo.md) "Resolved And Open Decisions");
  tracked in [../../mizar-parser/en/todo.md](../../mizar-parser/en/todo.md).
- **Public enum forward compatibility: open, initially resolved by the
  pre-consumer gate.** Apply the same per-enum `#[non_exhaustive]`-versus-
  exhaustive decision procedure that `mizar-frontend` task 25 established
  before `mizar-syntax` becomes a resolver/LSP input, and revisit it as new
  vocabulary enums are added.

## Ordered Task List

Each task is sized to be implemented, tested, and committed on its own. Keep
`cargo test -p mizar-syntax` green after each task (see
[Recommended Verification](#recommended-verification)).

### Representation foundation

1. **Module split and lint-policy guard.** [x]
   - Split `src/lib.rs` into `pub mod ast;` and `pub mod recovery;`, moving the
     task-12 types without behavior changes, and re-export everything from the
     crate root so `mizar-parser` and `mizar-frontend` paths stay valid.
   - Add `tests/lint_policy.rs` mirroring the `mizar-frontend` guard: workspace
     lint opt-in, deny baseline, and rationale required next to any future
     `allow`.
   - Tests: existing consumers compile unchanged; lint-policy guard passes.
   - Deps: none. Spec: [ast.md](./ast.md), [recovery.md](./recovery.md).

2. **`rowan` storage boundary and builder/accessor API.** [ ]
   - Adopt a rowan-backed `SurfaceAst` green-tree representation and record the
     decision with its rationale in [ast.md](./ast.md). If existing
     `SurfaceNode`/`SurfaceNodeId` names are kept for compatibility, document
     them as wrappers or views over the rowan-backed representation rather than
     as the storage backend.
   - Define the raw `SyntaxKind`/node-kind mapping, node/token role
     conventions, range attachment rules, and typed accessor/view helpers that
     hide rowan internals from resolver/checker consumers unless exposure is
     deliberately documented.
   - Define the `SurfaceAstBuilder` boundary used by `mizar-parser`: parser code
     constructs nodes and recovery markers through builder/event APIs rather
     than pushing into a concrete arena or depending on rowan's raw tree shape.
   - Record identity rules: rowan green-node identity and any dense indices are
     internal cache details, not stable artifact ids; deterministic snapshots
     and content cache keys are the stability surface.
   - Tests: builder round-trip into the rowan-backed tree, typed accessor
     coverage for every current node/token kind, parent ranges contain child
     ranges except for documented recovery cases, and repeated construction
     produces deterministic snapshots.
   - Deps: 1. Spec: [ast.md](./ast.md) "Public API".

3. **Deterministic snapshot rendering.** [ ]
   - Add a stable, human-readable text rendering of `SurfaceAst` (kind, range,
     recovered flag, children indented) for corpus snapshot baselines required
     by [architecture/en/20.test_strategy.md](../../architecture/en/20.test_strategy.md)
     "Snapshot Tests".
   - Rendering must be byte-identical across runs and platforms and free of
     hash-map iteration order, addresses, and other nondeterminism.
   - Tests: identical output across repeated renders; representative fixture
     covering every current node kind; recovery nodes visibly marked.
   - Deps: 2. Spec: [ast.md](./ast.md); snapshot layout in
     [../../mizar-test/en/snapshot.md](../../mizar-test/en/snapshot.md).

4. **Trivia model.** [ ]
   - Add `pub mod trivia;`. Decide and record the ownership split with
     `mizar-frontend::PreprocessedSource` (which already owns comment and
     doc-comment extraction), then define trivia attachment: doc-comment
     attachment targets, skipped-token ranges, and whitespace-sensitive hints
     needed by formatter and LSP consumers.
   - Doc-comment attachment stays syntactic; no semantic interpretation.
   - Tests: trivia ownership and attachment hints; skipped ranges preserved with
     source ranges; rendering includes trivia deterministically when requested.
     The concrete "doc comment attaches to the following item node" fixture
     lands with the first item-node increment in task 6 / parser task 5.
   - Deps: 2, 3. Spec: [trivia.md](./trivia.md).

5. **Recovery vocabulary expansion.** [ ]
   - Extend `SyntaxRecoveryKind` beyond the task-12 minimum (`ErrorToken`,
     `MissingEnd`, `MissingStringLiteral`) to the full vocabulary promised by
     [recovery.md](./recovery.md): missing constructs, skipped tokens,
     unmatched delimiters, malformed annotations.
   - Keep the `recovered` flag contract: resolver and checker phases must be
     able to skip or reject recovered subtrees without re-parsing.
   - Tests: each recovery kind constructible with correct ranges; recovered
     subtree query helpers; snapshot rendering marks each kind distinctly.
   - Deps: 2. Spec: [recovery.md](./recovery.md).

### Pre-consumer compatibility gate

**Initial public enum forward-compatibility gate.** [ ]
- For each public enum available at the phase-3 boundary (`SurfaceNodeKind`,
  `SurfaceTokenKind`, `SyntaxRecoveryKind`, `SyntaxDiagnosticCode`, and trivia
  kinds introduced by task 4), decide `#[non_exhaustive]` versus deliberate
  exhaustiveness using the `mizar-frontend` task-25 procedure.
- Record each decision next to the enum in the owning module spec and apply the
  attributes before parser tasks 5-7 can make resolver/LSP consumers plausible.
- Deps: 4, 5. Spec: [ast.md](./ast.md), [trivia.md](./trivia.md),
  [recovery.md](./recovery.md).

### Node vocabulary (paired with `mizar-parser` grammar tasks)

Node kinds for each area are added **incrementally**: each increment lands in
the same change as the `mizar-parser` grammar task that constructs it (the
parser todo's numbering governs the change granularity), and each increment
extends snapshot rendering. A vocabulary task below is checked off when the
last of its paired parser tasks lands. Do not add node kinds speculatively
ahead of the parser task that constructs them. Spec references are the
normative grammar chapters under [doc/spec/en/](../../../spec/en/00.index.md).

6. **Module and item nodes.** [ ] — paired with `mizar-parser` tasks 5-7.
   - Module file shape, top-level item list and item kinds dispatchable by
     keyword (parser task 5); import items with aliases and relative prefixes
     (parser task 6); export, `open`/`inherit`, and visibility forms (parser
     task 7).
   - Spec: [12.modules_and_namespaces.md](../../../spec/en/12.modules_and_namespaces.md).

7. **Type expression nodes.** [ ] — paired with `mizar-parser` task 8.
   - Attribute chains (with `non`), radix/mode type heads, `of`/`over`
     arguments, struct-qualified attribute references.
   - Spec: [03.type_system.md](../../../spec/en/03.type_system.md),
     [§A.3.2](../../../spec/en/appendix_a.grammar_summary.md).

8. **Term nodes.** [ ] — paired with `mizar-parser` tasks 4, 9-12, and 15.
   - First increment: qualified-symbol/namespace-path nodes needed by parser
     task 4. Then primary terms (parser task 9), unresolved dot chains and
     selector access/update (parser task 10, including the dot-role surface
     shape decision), functional structure updates, `qua` (parser task 11),
     operator-expression nodes generalizing the task-12 `InfixExpression` to
     prefix/postfix forms (parser task 12), and Fraenkel/set-builder forms
     (parser task 15). Primary-term coverage includes `it`, choice expressions
     (`the type_expression`), structure constructors, set enumeration literals,
     and application forms.
   - Spec: [13.term_expression.md](../../../spec/en/13.term_expression.md),
     [appendix_b.operator_precedence.md](../../../spec/en/appendix_b.operator_precedence.md).

9. **Formula nodes.** [ ] — paired with `mizar-parser` tasks 13-14.
   - Atomic predicate application, `is` formulas, attribute formulas (parser
     task 13); connectives and quantifiers (`for`/`ex`/`st`/`holds`) (parser
     task 14).
   - Spec: [14.formulas.md](../../../spec/en/14.formulas.md).

10. **Statement nodes.** [ ] — paired with `mizar-parser` tasks 16 and 18-21.
    - Simple statements `reserve`, `let`, `assume`, `take`, `set`, `given`
      (parser task 16); `consider`/`reconsider` (parser task 18);
      `thus`/`hence`, `then` chains, iterative equality `.=` (parser task 19);
      `now`/`hereby` and `per cases`/`suppose` blocks (parser task 20);
      `deffunc`/`defpred` local definitions and `claim` (parser task 21).
    - Spec: [15.statements.md](../../../spec/en/15.statements.md).

11. **Theorem, proof, and justification nodes.** [ ] — paired with
    `mizar-parser` tasks 17 and 22.
    - Justification clauses (`by`, `from`), citation forms including `.{ … }`
      and `.*`, plus `by computation(...)` option nodes (parser task 17);
      `theorem`/`lemma` items, labels, `proof … end` nesting (parser task 22).
    - Spec: [16.theorems_and_proofs.md](../../../spec/en/16.theorems_and_proofs.md),
      [20.algorithm_and_verification.md](../../../spec/en/20.algorithm_and_verification.md)
      §20.9.2.

12. **Definition, structure, and registration nodes.** [ ] — paired with
    `mizar-parser` tasks 23-30.
    - Definition block skeleton, correctness-condition clauses, and `attr`
      definitions (parser task 23); `pred`/`func`/`mode` bodies (parser tasks
      24-26); `redefine`, `synonym`/`antonym` (parser task 27); property
      clauses (parser task 28); `struct` definitions with fields and
      inheritance (parser task 29); registration and cluster forms and
      `reduce` (parser task 30).
    - Spec: [06.attributes.md](../../../spec/en/06.attributes.md),
      [07.modes.md](../../../spec/en/07.modes.md),
      [09.predicates.md](../../../spec/en/09.predicates.md),
      [10.functors.md](../../../spec/en/10.functors.md),
      [05.structures.md](../../../spec/en/05.structures.md),
      [17.clusters_and_registrations.md](../../../spec/en/17.clusters_and_registrations.md).

13. **Template, algorithm, and annotation nodes.** [ ] — paired with
    `mizar-parser` tasks 31-35.
    - Template parameters and bracket-form type arguments (parser task 31);
      algorithm blocks, assignment, declarations, ghost declarations/assignments,
      snapshots, and returns (parser task 32); control flow including processed
      collection loops and match endings (parser task 33); verification clauses
      (parser task 34);
      statement-level annotations, `@[...]` library annotations, and
      string-literal annotation arguments (parser task 35).
    - Spec: [18.templates.md](../../../spec/en/18.templates.md),
      [20.algorithm_and_verification.md](../../../spec/en/20.algorithm_and_verification.md),
      [21.source_code_annotation_and_atp.md](../../../spec/en/21.source_code_annotation_and_atp.md).

### Cross-cutting follow-ups

14. **Public enum forward-compatibility policy.** [ ]
    - Revisit the initial public-enum gate once the vocabulary is complete and
      decide `#[non_exhaustive]` versus deliberate exhaustiveness for any public
      enums added by later node-vocabulary increments.
    - Record final decisions next to the enum in the owning module spec and
      apply any remaining attributes.
    - Deps: 13. Spec: all module specs.

15. **Incremental syntax reuse audit.** [ ]
    - Audit the completed rowan-backed syntax tree for fine-grained incremental
      parsing and LSP reuse readiness: stable syntax-kind numbering policy,
      trivia/recovery placement, range attachment, node-role accessors, and
      subtree snapshot behavior under localized edits.
    - This task does not introduce `salsa`; it verifies that `SurfaceAst` can
      be produced and cached by later query layers without exposing unstable
      arena ids or parser internals.
    - Deps: 13, 14. Spec: [ast.md](./ast.md), [trivia.md](./trivia.md),
      [recovery.md](./recovery.md).

16. **Source/spec correspondence audit.** [ ]
    - Mirror the `mizar-frontend` task-16 audit: trace every public API and
      promised behavior in [ast.md](./ast.md), [trivia.md](./trivia.md), and
      [recovery.md](./recovery.md) to implementation and tests, and record
      gaps as follow-up tasks.
    - Deps: 15. Spec: all module specs and this TODO.

17. **Bilingual documentation sync audit.** [ ]
    - Compare each English canonical document under
      `doc/design/mizar-syntax/en/` with its Japanese companion and synchronize
      API lists, statuses, terminology, links, and behavior promises.
    - Deps: 16. Spec: repository documentation policy.

18. **Rustdoc summaries.** [ ] Deferred.
    - Same workspace-level deferral as `mizar-frontend` task 26. Re-entry
      trigger: the first long-lived consumer outside the frontend pipeline
      (resolver or `mizar-lsp`) starts coding against `mizar-syntax`, or the
      workspace adopts a rustdoc policy — whichever comes first.
    - Deps: 14. Spec: repository documentation policy.

## Recommended Verification

Run after each task:

```text
cargo test -p mizar-syntax
cargo fmt --check
cargo clippy -p mizar-syntax --all-targets --all-features -- -D warnings
```

For tasks that move or extend the shared boundary, also run:

```text
cargo test -p mizar-parser
cargo test -p mizar-frontend
```

Check the task off here once tests pass.

## Notes

- `mizar-syntax` owns syntax data shapes only: no grammar logic, no name
  resolution, no typing, no proof semantics. Resolved symbol ids, inferred
  types, and proof obligations never appear in `SurfaceAst`.
- `rowan` is the planned syntax-tree backend; parser and consumer code should
  depend on `mizar-syntax` builder/accessor APIs, not on an ad hoc arena layout.
- `salsa` is a later query/cache layer concern. Preserve pure phase boundaries
  and immutable syntax snapshots here so it can be introduced without rewriting
  the syntax crate.
- Vocabulary growth is paced by `mizar-parser` grammar tasks; do not add node
  kinds speculatively ahead of a parser task that constructs them.
- `SurfaceAst` is internal compiler data, not a stable external schema; the
  snapshot rendering (task 3) is the stability surface for corpus baselines.
