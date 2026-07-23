# Crate Exit Report: mizar-syntax

## Parser Task 46 Post-Exit Addendum

Parser Task 46 extends the already-exited syntax vocabulary with
`OperatorDeclaration` and append-only `SyntaxKind::OperatorDeclaration = 193`.
The matching surface kind, typed accessor, snapshot/raw/node/rowan contracts,
and tests preserve prior discriminants. This syntax-only increment does not
rescore or reopen the historical crate exit.

> Canonical language: English. Japanese companion:
> [../ja/crate_exit_report.md](../ja/crate_exit_report.md).

## Result

Status: refreshed close-out after the task-25 AST refactor follow-up audit. The
task-35 autonomous `mizar-syntax` milestone remains the historical basis for
the score below; post-exit tasks 22 through 25 are now complete, and S-021
rustdoc summaries remain explicitly deferred by policy trigger. The Parser
Task 48 vocabulary addendum below is a post-exit syntax increment and does not
rescore or reopen that historical milestone.

Quality score: reviewed 94/100.

Score caps applied: none at the time of the historical report. As of the S-025 rerun, no
new unresolved hard gate failure, `source_undocumented_behavior`,
`test_expectation_drift`, boundary violation, or repo metadata conflict is known
in `mizar-syntax` scope. Remaining current work is limited to the
policy-triggered S-021 rustdoc deferral.

## Scope

Milestone scope:

- complete the rowan-backed `SurfaceAst`, syntax vocabulary, trivia, recovery,
  diagnostic, typed-accessor, and snapshot contracts required by
  `mizar-syntax`;
- complete paired parser-facing syntax work through parser tasks 4-36 where
  needed for syntax task completion;
- record that parser task 36 / syntax task 22 predicate-label follow-through,
  task 24 private AST source split, and task 25 follow-up audit landed after
  the original historical report and are tracked by the S-023/S-025 audits;
- keep syntax representation source-shaped and free of semantic name, type,
  proof, and VC behavior;
- record source/spec/test correspondence, bilingual synchronization, and this
  crate exit evidence.

Included:

- rowan-backed storage, deterministic green-tree and text snapshots, raw-kind
  compatibility, typed compatibility views, and current syntax vocabulary
  through task 35, plus the parser-task-36 predicate redefinition label slot;
- private AST implementation partitions for green-tree construction, stable
  snapshot rendering, and AST tests under `src/ast/`, without changing public
  `mizar_syntax` API paths;
- syntax-owned trivia side tables for comments, doc-comment attachment hints,
  skipped-token ranges, and whitespace hints;
- recovery kind and syntax diagnostic surfaces, including active parser
  producers and vocabulary-only future producers;
- parser/syntax pairing evidence for module, import/export, type, term,
  formula, statement, theorem/proof, definition, registration, template,
  algorithm, verification-clause, and annotation surfaces;
- Rust unit/lint/snapshot tests, active parser `.miz` parse-only coverage,
  expectation sidecars, and traceability metadata;
- English and Japanese design documents, including the crate plan, source/spec
  audit, bilingual audit, TODO, and this report.

Excluded:

- changing `doc/spec/en`, `doc/spec/ja`, existing `.miz` tests, expectation
  sidecars, or syntax snapshots merely to match current implementation;
- resolver-owned namespace classification, type checking, overload resolution,
  cluster facts, theorem validity, proof obligations, algorithm VC generation,
  and package/build semantics;
- lexer/raw comment extraction, frontend cache orchestration, parser grammar
  ownership outside the paired task map, and `salsa` query integration;
- producer-backed tests for recovery kinds whose current status is explicitly
  vocabulary-only, and active `.miz` coverage for dotted algorithm `Lvalue`
  until the owning frontend/parser dot-role increment can carry the surface
  without unrelated diagnostics;
- rustdoc summaries until the S-021 re-entry trigger is met.

## Hard Gates

| Gate | Status | Evidence |
|---|---|---|
| Specification consistency | Pass | [00.crate_plan.md](./00.crate_plan.md) maps syntax behavior to `doc/spec/en`, active `.miz` coverage, traceability metadata, and module specs. S-019 found no new `spec_gap` or blocking inconsistency. |
| Test contract | Pass with explicit deferred rationale | Rust tests cover builder/accessor, rowan/raw-kind, snapshot, trivia, recovery, diagnostic, and lint-policy contracts. Active parser `.miz` cases and expectation sidecars cover parser-facing syntax through task 35 plus the parser-task-36 predicate redefinition label repair. Deferred seed rows, vocabulary-only recovery producers, dotted `Lvalue` active `.miz` coverage, and rustdoc summaries have explicit owners and unblock conditions. |
| Traceability | Pass | [source_spec_correspondence.md](./source_spec_correspondence.md) traces public APIs, method-level behavior, enum/diagnostic surfaces, source files, and test evidence. `tests/coverage/spec_trace.toml` records active and planned parser-facing cases. |
| Design/source sync | Pass | S-019 found no unimplemented promised public behavior or undocumented implementation-facing public behavior. S-020 synchronized English and Japanese design docs. S-023 and S-025 reruns found no remaining predicate-label or AST-refactor drift after updating status/source-layout records. |
| Boundary discipline | Pass | [README.md](./README.md), [00.crate_plan.md](./00.crate_plan.md), and module specs keep lexer, parser, resolver, type, proof, VC, cache, and build-system responsibilities outside `mizar-syntax`. |
| Verification | Pass | Current branch verification results are recorded below: format, clippy, workspace tests, and traceability plan all pass. |
| Residual risk | Pass | Remaining items are classified as `MSYN-GAP-001`, `MSYN-GAP-003`, `MSYN-GAP-013`, or deferred S-021 rustdoc work. No blocking/high syntax-owned finding remains. |

## Score Breakdown

| Category | Points |
|---|---:|
| Specification completeness | 18/20 |
| Test contract and coverage | 18/20 |
| Traceability | 14/15 |
| Implementation correctness | 15/15 |
| Design/source synchronization | 10/10 |
| Boundary discipline | 10/10 |
| Verification health | 5/5 |
| Handoff quality | 4/5 |
| Total | 94/100 |

The score keeps small deductions for deferred parser/frontend-owned `.miz`
activation, vocabulary-only recovery producers, the dotted algorithm `Lvalue`
active-corpus gap, and the policy-triggered rustdoc deferral. These are
documented risks, not hard gate failures.

## Deferred Items

| ID | Reason | Owner | Unblock condition |
|---|---|---|---|
| `MSYN-GAP-001` | [parse_only_fixture_seed.md](./parse_only_fixture_seed.md) keeps inactive parser-facing seed rows for future grammar activation points. | Owning future parser/syntax task | Activate each row only when its owning parser grammar task can satisfy the expectation through the frontend parser seam, then add or update `.miz`, `.expect.toml`, and traceability metadata without changing intent. |
| `MSYN-GAP-003` | Some recovery kinds are constructible in `mizar-syntax` but documented as vocabulary-only because current parser producers do not emit them yet. | Owning parser recovery producer tasks | When a future parser producer emits one of these recovery kinds, update [recovery.md](./recovery.md), producer tests, and syntax/parser evidence for that concrete producer. |
| `MSYN-GAP-013` | Dotted algorithm `Lvalue` is represented and parser-unit-tested, but active `.miz` coverage waits for the frontend/parser dot-role increment that can preserve the dotted assignment surface without unrelated diagnostics. | Parser/frontend dot-role integration | Add active `.miz` coverage and traceability once the owning increment can carry the surface through the frontend seam cleanly. |
| S-021 | Rustdoc summaries are deferred until a long-lived consumer outside the frontend pipeline starts coding against `mizar-syntax` or the workspace adopts a rustdoc policy. | Syntax maintenance after policy/consumer trigger | Re-enter S-021, add rustdoc summaries as non-canonical API documentation, and run the workspace rustdoc/lint policy checks that exist at that time. |

Resolved during this milestone:

- `MSYN-GAP-002`, `MSYN-GAP-004`, `MSYN-GAP-005`, `MSYN-GAP-006`,
  `MSYN-GAP-007`, `MSYN-GAP-008`, `MSYN-GAP-011`, and `MSYN-GAP-012` were
  closed by the task sequence recorded in [00.crate_plan.md](./00.crate_plan.md).
- `MSYN-GAP-009` remains a process note: the crate plan pairing map is the
  authority when parser and syntax task numbers disagree.
- `MSYN-GAP-010` remains a metadata watch item; no repository metadata conflict
  is currently observed.

## Human Review Surface

The human reviewer should primarily inspect:

- [00.crate_plan.md](./00.crate_plan.md)
- this report
- [README.md](./README.md)
- [ast.md](./ast.md)
- [trivia.md](./trivia.md)
- [recovery.md](./recovery.md)
- [grammar_audit.md](./grammar_audit.md)
- [parse_only_acceptance_matrix.md](./parse_only_acceptance_matrix.md)
- [parse_only_fixture_seed.md](./parse_only_fixture_seed.md)
- [source_spec_correspondence.md](./source_spec_correspondence.md)
- [bilingual_documentation_synchronization.md](./bilingual_documentation_synchronization.md)
- [todo.md](./todo.md)
- `crates/mizar-syntax/src/lib.rs`
- `crates/mizar-syntax/src/ast.rs`
- `crates/mizar-syntax/src/ast/green.rs`
- `crates/mizar-syntax/src/ast/snapshot.rs`
- `crates/mizar-syntax/src/ast/tests.rs`
- `crates/mizar-syntax/src/trivia.rs`
- `crates/mizar-syntax/src/recovery.rs`
- `crates/mizar-syntax/tests/lint_policy.rs`
- `tests/snapshots/mizar_syntax_surface_ast_current_vocabulary.snap`
- `tests/coverage/spec_trace.toml`
- active parser `.miz` and `.expect.toml` files referenced by
  [00.crate_plan.md](./00.crate_plan.md)
- Japanese companions for the same `mizar-syntax` design files

Canonical language/test surfaces referenced by this report include:

- `doc/spec/en/03.type_system.md`
- `doc/spec/en/05.structures.md`
- `doc/spec/en/06.attributes.md`
- `doc/spec/en/07.modes.md`
- `doc/spec/en/09.predicates.md`
- `doc/spec/en/10.functors.md`
- `doc/spec/en/11.symbol_management.md`
- `doc/spec/en/12.modules_and_namespaces.md`
- `doc/spec/en/13.term_expression.md`
- `doc/spec/en/14.formulas.md`
- `doc/spec/en/15.statements.md`
- `doc/spec/en/16.theorems_and_proofs.md`
- `doc/spec/en/17.clusters_and_registrations.md`
- `doc/spec/en/18.templates.md`
- `doc/spec/en/20.algorithm_and_verification.md`
- `doc/spec/en/21.source_code_annotation_and_atp.md`
- `doc/spec/en/appendix_a.grammar_summary.md`
- `doc/spec/en/appendix_b.operator_precedence.md`

No source implementation, `.miz` file, `.expect.toml` file, syntax snapshot, or
canonical language specification file is changed by this final report task.

## Test Expectation Summary

This final report task does not change `.expect.toml` files, snapshots, `.miz`
files, or Rust source. The milestone relies on the following expectation groups.

| Test | Intent | Expected outcome | Expected phase | Diagnostics | Spec refs |
|---|---|---|---|---|---|
| `tests/snapshots/mizar_syntax_surface_ast_current_vocabulary.snap` and `crates/mizar-syntax` Rust tests, including `src/ast/tests.rs` | Guard rowan-backed syntax storage, deterministic snapshots, typed accessors through task 35 plus the predicate redefinition label-slot follow-through, trivia attachment, recovery vocabulary, diagnostics, and lint policy. | Pass | Rust unit/lint/snapshot | Panics only in documented negative invariant tests. | [ast.md](./ast.md), [trivia.md](./trivia.md), [recovery.md](./recovery.md) |
| `tests/miz/pass/parser/pass_parser_minimal_token_stream_001.expect.toml` and task-5 module/recovery sidecars | Guard frontend-reachable parser/syntax baseline and module skeleton recovery. | Pass or fail/recover as encoded by sidecar | Parse-only | Syntax diagnostics for expected recovery fixtures only. | `spec.en.syntax.parser_minimal_token_stream`, `spec.en.syntax.parser_recovery.*`, Chapter 12 |
| Import/export/type sidecars for parser tasks 6-8 | Guard import/export/visibility and type-expression syntax surfaces. | Pass or fail/recover as encoded by sidecar | Parse-only | Syntax diagnostics for malformed import/export/type fixtures only. | Chapters 3 and 12; Appendix A |
| Term/formula sidecars for parser tasks 9-15 | Guard primary terms, selector/update, `qua`, operator terms, set comprehensions, atomic formulas, connectives, quantifiers, constants, and formula recovery. | Pass or fail/recover as encoded by sidecar | Parse-only | Syntax diagnostics for missing operands, malformed delimiters, non-associative `iff`, and related recovery fixtures. | Chapters 13 and 14; Appendix B |
| Statement/proof sidecars for parser tasks 16-22 | Guard simple statements, justifications, `consider`/`reconsider`, conclusions, iterative equality, block statements, inline definitions, theorem/proof items, and proof recovery. | Pass or fail/recover as encoded by sidecar | Parse-only | Syntax diagnostics for malformed statements, justifications, and proof recovery fixtures. | Chapters 15, 16, and parser-facing Chapter 20 hosts |
| Definition/structure/registration sidecars for parser tasks 23-30 | Guard definition blocks, predicate/functor/mode definitions, redefinition, notation, properties, structures, inheritance, registrations, clusters, reductions, and recovery. | Pass or fail/recover as encoded by sidecar | Parse-only | Syntax diagnostics for malformed definition-family and registration fixtures. | Chapters 5-7, 9-11, 16, and 17 |
| Template/algorithm/annotation sidecars for parser tasks 31-35 | Guard template arguments/references, algorithms, claims, control flow, verification clauses, annotations, and malformed annotation/algorithm recovery. | Pass or fail/recover as encoded by sidecar | Parse-only | Syntax diagnostics for chained `iff`, malformed algorithm/claim/control-flow/verification/annotation fixtures, and expected recovery nodes. | Chapters 18, 20, 21; Appendix A |
| Deferred seed rows and `MSYN-GAP-013` | Preserve future executable intent without pretending unsupported producers exist today. | Planned/deferred | Parse-only or parser-unit-only until unblocked | No active runner diagnostics required until the owner can produce them. | [parse_only_fixture_seed.md](./parse_only_fixture_seed.md), [00.crate_plan.md](./00.crate_plan.md) |

## Verification

Commands run for this crate exit task:

```sh
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
cargo run -p mizar-test -- plan --tests-root tests --manifest tests/coverage/spec_trace.toml
```

Results:

- `cargo fmt --check`: passed.
- `cargo clippy --all-targets --all-features -- -D warnings`: passed.
- `cargo test`: passed across the workspace, including `mizar-syntax` unit tests
  and lint-policy tests.
- `cargo run -p mizar-test -- plan --tests-root tests --manifest tests/coverage/spec_trace.toml`: passed with 162 test cases, 90 requirements, 0 errors, and 4 existing planned/no-tests warnings outside `mizar-syntax` scope:
  - `spec.en.algorithm.vc.assignment_loop_exits`
  - `spec.en.binding.substitution.capture_avoidance`
  - `spec.en.elaboration.choice_comprehension.lowering`
  - `spec.en.type_soundness.escape_and_guard_failures`

## Handoff

Next recommended work:

- Start the owning frontend/parser dot-role integration that can unblock
  `MSYN-GAP-013`, or start a future parser recovery producer task that turns a
  vocabulary-only `SyntaxRecoveryKind` into active producer-backed coverage.

Known constraints:

- Do not move resolver, type, proof, VC, lexer, frontend cache, or parser-owned
  grammar decisions into `mizar-syntax`.
- Do not add `.miz`, expectation, or spec changes merely to match current
  implementation behavior.
- Keep syntax identity deterministic but non-persistent; future query layers
  should cache syntax outputs behind frontend/build-owned content keys.
- Keep English design docs and Japanese companions synchronized in the same
  change.

Open questions:

- Which frontend/parser dot-role milestone should first carry dotted algorithm
  assignment through active `.miz` parse-only coverage?
- Which future parser recovery producer should first activate a currently
  vocabulary-only recovery kind?
- When will a long-lived resolver or LSP consumer trigger S-021 rustdoc
  summaries?

Recommended reasoning setting for the next task:

- `high`, because the next useful work crosses parser/frontend/syntax
  boundaries, active `.miz` coverage, traceability metadata, and recovery or
  dot-role diagnostics. Lower to `medium` for documentation-only follow-up, and
  raise above `high` if the task changes canonical grammar or semantic language
  behavior.

## Parser Task 48 Post-Exit Addendum

Parser Task 48 extends the already-exited syntax vocabulary with the top-level
`PropertyImplementation` node and append-only
`SyntaxKind::PropertyImplementation = 192`. The matching `SurfaceNodeKind`,
typed accessor, snapshot rendering, raw-kind and node-kind round trips, and
rowan boundary are covered by the Task 48 syntax changes and tests. The nested
parameter remains source-shaped as
`DefinitionParameter -> TypeHead -> QualifiedSymbol + optional TypeArguments`,
and the parser's active Task 48 pass/fail corpus covers the valid and bounded
recovery paths.

This addendum records syntax-only completion for `SPEC-07-PI-PLACEMENT`. It does
not claim semantic property validation, which remains deferred to semantic
Task 39, and it does not introduce a new `mizar-syntax` task ID. The historical
94/100 score and S-025 exit determination above therefore remain unchanged.
