# Task CHECKER-FRAENKEL-GENERATOR-BOUND-USE-257C4B: Fraenkel Generator Bound-Use Transport

> Canonical language: English. Japanese companion: [../ja/CHECKER-FRAENKEL-GENERATOR-BOUND-USE-257C4B.md](../ja/CHECKER-FRAENKEL-GENERATOR-BOUND-USE-257C4B.md).

Owning plans: [mizar-checker](../../mizar-checker/en/00.crate_plan.md#task-index)
and [mizar-test](../../mizar-test/en/00.crate_plan.md#task-index).

Stable owner sections: checker [formula-composition API and test
design](../../mizar-checker/en/source_formula_composition.md#task-257c4b-fraenkel-generator-bound-use-transport),
[source/spec classification](../../mizar-checker/en/source_spec_audit.md#task-257c4b-fraenkel-generator-bound-use-classification),
[module boundary](../../mizar-checker/en/module_boundary_audit.md#task-257c4b-fraenkel-generator-bound-use-boundary),
[TODO](../../mizar-checker/en/todo.md#task-257c4b-fraenkel-generator-bound-use-transport),
and [bilingual record](../../mizar-checker/en/bilingual_sync_audit.md#task-257c4b-frozen-contract-parity);
mizar-test [harness](../../mizar-test/en/harness.md#checker-task-257c4b-private-bound-use-probe),
[boundary](../../mizar-test/en/module_boundary_audit.md#checker-task-257c4b-frozen-module-boundary),
[TODO](../../mizar-test/en/todo.md#checker-task-257c4b-private-bound-use-probe),
and [bilingual audit](../../mizar-test/en/bilingual_sync_audit.md#checker-task-257c4b-frozen-contract-parity).

## Status, authority, and readiness

**Status:** this is the documentation-only implementation prerequisite. The
implementation has not begun. Fresh clean inventory at
`3d6add94f4b29d395a9362b56c05cc9256efa945` selects this separately bounded
Task-257C slice after the completed C4A lifecycle closure. At that checkpoint
`origin/main...HEAD` is `0/20`, and `stash@{0}` remains
`f65cf4a13752ec380710814a9ac6392ccb9d75d4`. Implementation may begin only
after this exact documentation prerequisite is reviewed, committed alone, and
followed by a clean fresh preflight that reproduces the frozen authority, API,
scope, counts, and hashes.

Authority is, in order:

1. canonical [Chapter 13 §13.4](../../../spec/en/13.term_expression.md#134-set-expressions),
   especially §§13.4.2, 13.4.4, and 13.8.6;
2. canonical [Chapter 18 §18.10.2](../../../spec/en/18.templates.md#18102-type-parameter-encoding);
3. immutable F5
   [`fail_template_fraenkel_over_type_param_001.miz`](../../../../tests/miz/fail/templates/fail_template_fraenkel_over_type_param_001.miz);
4. its [trace entry](../../../../tests/coverage/spec_trace.toml);
5. its [expectation](../../../../tests/miz/fail/templates/fail_template_fraenkel_over_type_param_001.expect.toml);
6. [Architecture 16 canonical binder normalization](../../architecture/en/16.substitution_and_binding.md#canonical-binder-normalization);
7. completed [R2](RESOLVE-FRAENKEL-GENERATOR-VAR-277R2.md),
   [277B-L](277B-L.md), [277C](CHECKER-FRAENKEL-TEMPLATE-STRUCTURAL-277C.md),
   and exact predecessor [C4A](CHECKER-FRAENKEL-GENERATOR-BINDING-257C4A.md);
8. derived owner documents and source observations.

The specification makes the generator `x` bound and later capture
identity-based. F5 fixes the exact three uses. C4A already fixes their
source-to-lookup ordinal normalization and the checker-local binding identity.
Therefore this task may transport only those three C4A-normalized uses to the
already existing `BindingId(0)`. It neither decides sethood nor creates a new
term/reference/capture interpretation.

Completed C4A is the sole lower dependency and must be consumed as one opaque
handoff. The planning omission of a separately frozen C4B record was bounded
`design_drift`, closed by this prerequisite. The later implementation owns a
bounded `source_drift` and Rust `test_gap`. There is no `spec_gap`, no new
semantic test intent, and no authority conflict. The immutable fixture still
lacks a nested-comprehension outer-variable occurrence, so actual capture
coverage remains a separate `test_gap`; it is not silently satisfied here.

Task 277B remains not ready with zero semantic credit. Its inactive
`advanced_semantics` expectation and trace seed remain blocked by `MC-G020`
and `MC-G021`; a structural bound-use handoff cannot discharge sethood,
type-checking, diagnostics, activation, or a semantic verdict.

## Frozen logical boundary

C4B is a checker-local association transaction in the existing
`crates/mizar-checker/src/source_formula_composition.rs` owner. It consumes
exactly `&SourceFraenkelGeneratorBindingContextHandoff`, revalidates the entire
opaque C4A snapshot before reading any row, performs `BindingEnv::lookup` at
each C4A-normalized lookup ordinal, and publishes one all-or-nothing dense
three-row handoff. It does not take R2, 277C, `TypedAst`, raw resolver IDs, a
caller DTO, or any role enum as direct input.

The exhaustive public ABI, signatures, getter return types, rendering strings,
error precedence, validation order, and checker test design are owned by the
linked [formula-composition section](../../mizar-checker/en/source_formula_composition.md#task-257c4b-fraenkel-generator-bound-use-transport).
The public family is limited to one ID, immutable row, table, handoff,
non-exhaustive error, and producer. There is no Typed/Resolved installation
slot and no production consumer.

The handoff privately clones the exact C4A handoff inside a version- and
domain-tagged dependency snapshot. Both `build` and private handoff validation
must invoke C4A's full validation before trusting source/module, tables,
`BindingEnv`, or the human summary. This transitively revalidates every stored
R2, 277C, and `TypedAst` dependency field and getter frozen by C4A. A debug
string, table length, spelling, source range, ordinal coincidence, or equal
dense index alone is never authoritative.

## Exact F5 three-row oracle

The result contains exactly these rows, in this order:

| Bound-use ID | C4A use position | C4A binding context | R2 use index | Source ordinal | Lookup ordinal | Context | Binding |
|---:|---:|---:|---:|---:|---:|---:|---:|
| 0 | 0 | 0 | 0 | 0 | 1 | 1 | 0 |
| 1 | 1 | 0 | 1 | 1 | 2 | 1 | 0 |
| 2 | 2 | 0 | 2 | 2 | 3 | 1 | 0 |

For every row, lookup is exactly
`BindingEnv::lookup(BindingLookupSite::new("x", context1, None,
lookup_ordinal)) == Local(binding0)`. C4A lookup ordinal 0 remains the separate
pre-visibility probe and must remain `ForwardReference`; it is never a C4B
mapper or bound-use row. The C4B producer neither derives ordinals from source
ranges nor routes the mapper through generic Task-252 occurrence ordering.

Default-deny permits exactly the one normal C4A binding context and these three
normal positions. Empty, partial, extra, reordered, duplicated, recovered,
nested, multiple-generator, shadowed, mismatched, stale, or non-local results
fail atomically and publish no partial handoff.

## Prohibitions and semantic deferrals

The implementation must not:

- create, reuse, install, or edit `SourcePrimaryTerm`, a primary-term
  reference, `SourceQuantifierBoundUse`, or any Task-252 row/API/test owner;
- copy or expose `FraenkelGeneratorVariableUseRole`, a raw `ResolvedNodeId`,
  R2/277C rows, or another lower owner's identity;
- mutate `CapturedFreeVariables` or create a capture/free-variable table;
- create a formula, term interpretation, type interpretation, sethood
  evidence/request/verdict, semantic request/verdict, diagnostic, fact,
  obligation, proof/core/IR/VC value, or test expectation;
- add a `TypedAst`/`ResolvedTypedAst` slot, installation method, facade,
  dispatcher, production route, runner schema, CLI route, active stage, or
  downstream consumer;
- edit canonical specification, `.miz`, expectation, trace, coverage,
  metadata, Cargo, parser/resolver production, C4A's `BindingEnv` API, or any
  previously completed task contract; or
- claim Task-277B readiness, activation, rejection coverage, or semantic
  credit.

Crossing any of these owners is a `boundary_violation` and requires a newly
frozen separate task. Actual nested-comprehension capture, term/reference
transport, formula composition, type/sethood interpretation, semantic
diagnostics and verdicts, installation, active routing, trace, and coverage
credit all remain explicitly deferred.

## Frozen implementation and test scope

The future implementation changes exactly three Rust paths:

1. `crates/mizar-checker/src/source_formula_composition.rs`;
2. `crates/mizar-test/src/runner/tests.rs`; and
3. new private
   `crates/mizar-test/src/runner/tests/type_elaboration/template_fraenkel_generator_bound_use.rs`.

No `binding_env.rs`, `lib.rs`, checker lint Rust, existing private leaf, or
production mizar-test path changes. Future completion documentation changes
exactly 20 Markdown paths: this contract pair plus the 18 non-plan owner
documents in this prerequisite. The four Task Index plans remain unchanged at
implementation time. The implementation worktree is therefore exactly 23
paths: 3 Rust and 20 Markdown.

Checker adds exactly four tests, changing the raw library list `550 -> 554`:

1. `task257c4b_builds_exact_fraenkel_generator_bound_uses` — proves the exact
   three-row oracle, dense-table/getter ABI, C4A summary, lookup outcomes, and
   literal debug text;
2. `task257c4b_rejects_environment_and_binding_context_dependency_corruption`
   — proves output/source/module mismatch precedence, snapshot version/domain
   rejection, complete C4A snapshot revalidation, and that a stale summary is
   never sufficient;
3. `task257c4b_rejects_bound_use_and_lookup_corruption` — proves missing,
   extra, duplicate, reordered, field-corrupted, non-local, and partial rows
   fail closed, including environment-before-dependency-before-row error
   precedence; and
4. `task257c4b_rebuilds_deterministically_without_mutation` — proves identical
   replay and byte-identical C4A dependency preservation.

The sole private mizar-test test is
`task257c4b_real_fixture_builds_exact_fraenkel_generator_bound_uses`, changing
the raw mizar-test list `613 -> 614`. It reuses the existing private frontend,
resolver, typed-profile, 277B-L, 277C, and C4A construction route for immutable
F5, then invokes C4B directly and asserts only its public summary, exact three
rows, lookup bindings, dense absence, debug text, and non-mutation. It is no
production route, semantic assertion, expectation, or coverage test.

## Baselines, count/hash impact, and exact documentation scope

The pre-documentation checkpoint is the clean
`3d6add94f4b29d395a9362b56c05cc9256efa945`; `origin/main...HEAD=0/20` and
protected `stash@{0}=f65cf4a13752ec380710814a9ac6392ccb9d75d4`.
Contract trees change `88/88 -> 89/89` in this prerequisite.

| Future Rust path | Baseline lines | Baseline SHA-256 | Expected impact |
|---|---:|---|---|
| `crates/mizar-checker/src/source_formula_composition.rs` | 7303 | `f6da763061479e74e7b8f39169ecad311bb9bf879e91e93824d9899798017abc` | changes; checker raw tests `+4` |
| `crates/mizar-test/src/runner/tests.rs` | 66 | `85ae891b185ed1eeb5940998c5eef5ece793b472b8f3fa4be3c0b96d217e1f07` | `+1` include registration |
| new private C4B leaf | absent | absent | one new test-only path / one test |

Checker production is 32 paths / 193103 lines, with path/content SHA-256
`9dc5b02f26679677e593ea755394d68533173d2be988b7ef1ddcfd84a41b9787` /
`cfc9a2bc5359f9baeea39f304e3c9dd15fcbd27749f1c746eb3ab695b84f8dab`.
The future implementation keeps 32 paths, changes only the existing
formula-composition line total/content hash, and records final measurements
after formatting. Mizar-test production remains exactly 38 paths / 80090
lines, with path/content SHA-256
`0ef395004f7feaadf60da0daba7b5da9c52ea4974850adfa2bd9d09081b242aa` /
`990b5ad4798786d9f87c03f76fdbad92fc2edf1f6d84ef3baad67254c79fdd70`.
Raw baseline lists are checker 550 /
`ba24ea98b25617e41c832ef2dc0878f0502249d68d281fdb4e4c1a7e66c71885`
and mizar-test 613 /
`a408a7099e886be8c6f4173325e40e4d9b3e28e42e8cc6cbad9bf88ce95e2741`.

This prerequisite changes exactly these 24 Markdown paths:

- this EN/JA contract pair;
- `doc/design/mizar-checker/{en,ja}/00.crate_plan.md`;
- `doc/design/mizar-test/{en,ja}/00.crate_plan.md`;
- checker EN/JA pairs for `source_formula_composition.md`,
  `source_spec_audit.md`, `module_boundary_audit.md`, `todo.md`, and
  `bilingual_sync_audit.md`; and
- mizar-test EN/JA pairs for `harness.md`, `module_boundary_audit.md`,
  `todo.md`, and `bilingual_sync_audit.md`.

No protected artifact changes. Specification remains 64 files with
path/content SHA-256
`d900ba9e43ab094925f36493f830e0c6a2964be2911d5d229014a58842067a25` /
`b30dd5519191a4407399826faf91cc58853d7944df7630734b5ab05de48c9f7b`;
`.miz` remains 343 with
`d94980e167b4b8ac196f91e7694ff044080c6fb4d04c135b3cd5e65b9a019f22` /
`54e6ea1254a0bd963c39026d788711507f353e9c6df3b4f9fd268b2e9f240afb`;
expectations remain 435 with
`22a5ee257a294e3f2ed4b24bf9ca243d037bbd798f009d0d1e3a176dad8b4fea` /
`b5f0ed1a8d73bbfb78af5cad87a8d426e97269f7f70163750893a9fe1f39d2ea`;
Cargo remains 21 with
`d93f2816b760d8ba45430d6d8570e598864aa7b20b19a001f45171d36fd3a030` /
`146e9b4024d2c344b2eca9c6f5ca6d6a80a3de427e382953a2280bc63cb3ecca`.
F5/expectation/trace physical SHA-256 remain
`32c4a1c1b6c9d98dcb085558a084929e07d4005bf92595865f144456e95854ec`,
`b47ac5113c89cd5703adb0ffd660b52d3e16908c92623dd2f63196aa6a215cb2`,
and `55b754c8c4d0d293a1c44e2ba4b0090f407bba1d429b461b6cb4d6ddca9ca2b3`.

`doc/design/spec_coverage_audit.md` has no impact and remains unchanged at
`a31f6fb3bd2b561610630497c58284484d00716dd0b7f210f55bef3bc4bfa6db`:
no specification mapping, trace owner, activation, deferred rationale, or
semantic coverage credit moves. Schema-v2 legacy compaction is also a no-op;
`legacy_compactions.tsv` remains
`2a66d200a1976861600bcf7686388faa3efb19b2b42a43c756c9e689d7f27359`.
No historical section, redirect, neighbor anchor, Task Index owner, ledger
row, or expanded-inventory hash is changed.

## Review, verification, and nine hard gates

The documentation prerequisite requires independent specification/API,
test-sufficiency, EN/JA, boundary, and source/documentation reviews, repairing
all blocking/high findings before implementation. Required docs verification:

```sh
git diff --check
cargo test -q -p mizar-checker --test lint_policy
cargo test -q -p mizar-test --test lint_policy
```

It also requires exact-24 path validation, EN/JA pair and stable-fragment/link
validation, protected no-delta/hash checks, `88/88 -> 89/89` contract-pair
proof, and unchanged schema-v2 ledger proof. The implementation later runs the
focused `4 + 1`, package libraries `554/554` and `614/614`, formatting,
package and full-workspace Clippy, full workspace tests, both lint suites,
metadata, frozen CLI replays, diff/scope checks, independent reviews, and a
final read-only quality review.

The nine gates are frozen as follows:

1. canonical specification and immutable F5 intent remain consistent;
2. the exact F5 three-row/lookup and corruption matrix pass;
3. inactive trace/expectation and zero semantic credit remain unchanged;
4. the exact public ABI and C4A full-snapshot validation are implemented;
5. EN/JA design and source agree, with final counts/hashes recorded once here;
6. Task-252, capture, semantic, installation, runner, and crate boundaries are
   not crossed;
7. `spec_coverage_audit.md` and schema-v2 ledger remain proven no-ops;
8. all required verification passes; and
9. every residual item is explicitly deferred, out of scope, or human-owned.

Any hard-gate failure invalidates the score. Final acceptance requires all
`9/9` gates and a valid score of at least `90/100` under the protocol rubric.

## Exit criteria and next handoff

This prerequisite exits only when its exact 24 Markdown paths are the sole
worktree delta, EN canonical and JA companion are logically synchronized,
every stable fragment/link and legacy neighbor anchor resolves, both lint
policy suites and `git diff --check` pass, protected hashes/counts are
unchanged, independent reviews report no unresolved blocking/high finding,
and a docs-only commit is followed by a clean fresh inventory.

That fresh inventory may hand off the exact three-Rust-path implementation
above. Keep Sol at `xhigh` for authority/API integration, scope acceptance, and
final hard-gate scoring. Use Terra `xhigh` for bounded implementation and
independent API/test/bilingual/boundary reviews after the contract is frozen;
raise back to Sol for any authority ambiguity, requested public-API expansion,
Task-252/capture/semantic boundary question, or disputed finding. If Luna is
not exposed, do not block.

After C4B completion, run another fresh inventory. Do not infer that the next
task is capture, sethood, or Task 277B. Task 277B remains not ready with zero
credit until separately authorized semantic owners and `MC-G020`/`MC-G021`
are actually discharged.
