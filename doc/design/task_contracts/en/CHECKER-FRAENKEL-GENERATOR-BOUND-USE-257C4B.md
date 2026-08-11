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

**Status:** the exact three-Rust-path implementation is complete in the
completion worktree based on clean preflight
`53987a3fdc1a927dbcbd2b9ed22e9817c8b68f2d`. At that checkpoint
`origin/main...HEAD` was `0/21`, and `stash@{0}` remained
`f65cf4a13752ec380710814a9ac6392ccb9d75d4`. The implementation and test
sufficiency reviews are **NO FINDINGS** after the existing corruption test was
repaired to cover retained structural and `TypedAst` dependency corruption
with dependency-before-row precedence. Final source/documentation review and
independent bilingual/boundary review are also **NO FINDINGS** after the sole
Low baseline/current wording issue was repaired in this contract pair. This
task's final-quality review is **NO FINDINGS**; all `9/9` hard gates pass at a
valid uncapped `100/100` (`20/20/15/15/10/10/5/5`). The task-only commit,
immediate post-commit proof, and accepted fresh successor inventory are
complete at the [historical checkpoint](#historical-immediate-post-implementation-checkpoint).
Exact staging/cached review passes over 23 paths (3 Rust and 20 docs),
including one new private leaf, with zero unstaged paths at review time;
`git diff --cached --check` passes and the cached stat is 1096 insertions / 123
deletions.

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
`design_drift`, closed by this prerequisite. The exact implementation closes
the bounded `source_drift` and Rust `test_gap`. There is no `spec_gap`, no new
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

The completed implementation changes exactly three Rust paths:

1. `crates/mizar-checker/src/source_formula_composition.rs`;
2. `crates/mizar-test/src/runner/tests.rs`; and
3. new private
   `crates/mizar-test/src/runner/tests/type_elaboration/template_fraenkel_generator_bound_use.rs`.

No `binding_env.rs`, `lib.rs`, checker lint Rust, existing private leaf, or
production mizar-test path changes. Completion documentation changes exactly
20 Markdown paths: this contract pair plus the 18 non-plan owner documents in
this prerequisite. The four Task Index plans remain unchanged. The completion
worktree is therefore exactly 23 paths: 3 Rust and 20 Markdown.

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

| Rust path | Baseline lines | Baseline SHA-256 | Frozen impact |
|---|---:|---|---|
| `crates/mizar-checker/src/source_formula_composition.rs` | 7303 | `f6da763061479e74e7b8f39169ecad311bb9bf879e91e93824d9899798017abc` | changes; checker raw tests `+4` |
| `crates/mizar-test/src/runner/tests.rs` | 66 | `85ae891b185ed1eeb5940998c5eef5ece793b472b8f3fa4be3c0b96d217e1f07` | `+1` include registration |
| new private C4B leaf | absent | absent | one new test-only path / one test |

Final Rust measurements are:

| Implemented Rust path | Final lines | Final SHA-256 |
|---|---:|---|
| `crates/mizar-checker/src/source_formula_composition.rs` | 7958 | `90b339d9707f9f8d847b678721e8db0ef6a00e4a15dbb41474a0cf6980f47168` |
| `crates/mizar-test/src/runner/tests.rs` | 67 | `94bc44e8ba47ca568670adeec74d20f6738b3fc337da2422871095137040e8c4` |
| `crates/mizar-test/src/runner/tests/type_elaboration/template_fraenkel_generator_bound_use.rs` | 121 | `bea54489cf0c85d3026f950d080a0ffc609719fda28520b9e7b2f59d5fc162bc` |

The sorted exact-three path hash is
`b55deb1e11851b50d135785ff685dd8df5803cff3d89205903370d5421ac55fa`;
the Rust delta is 777 insertions and no deletions. Checker production remains
32 paths with unchanged path SHA-256
`9dc5b02f26679677e593ea755394d68533173d2be988b7ef1ddcfd84a41b9787`,
grows to 193758 lines, and has final content SHA-256
`90d8e277c6878b372090efbde122f3e95e5c50dce0475c9e50bbcabcb8eb1424`.
Mizar-test production remains the protected 38 paths / 80090 lines with
unchanged path/content hashes. The final raw checker list is 554 /
`78f0291fb13aed8a8adbbc5aa1db9df1a7415fc9d8cf35820e1ad9e40aad2ace`;
the final raw mizar-test list is 614 /
`419ac370d2ec222cc822186db62595b5ebed71e1059e10fa95dc00741acc9778`.

The implementation baseline for checker production was 32 paths / 193103
lines, with path/content SHA-256
`9dc5b02f26679677e593ea755394d68533173d2be988b7ef1ddcfd84a41b9787` /
`cfc9a2bc5359f9baeea39f304e3c9dd15fcbd27749f1c746eb3ab695b84f8dab`.
The implementation was frozen to keep 32 paths and change only the existing
formula-composition line total/content hash. The corresponding mizar-test
production baseline was exactly 38 paths / 80090 lines, with path/content SHA-256
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

The prerequisite specification/API review and the implementation and
test-sufficiency reviews are complete with **NO FINDINGS** after the retained
structural/`TypedAst` corruption repair. Required docs verification is:

```sh
git diff --check
cargo test -q -p mizar-checker --test lint_policy
cargo test -q -p mizar-test --test lint_policy
```

The focused checker `4/4`, private `1/1`, package libraries `554/554` and
`614/614`, `cargo fmt --all -- --check`, package and full-workspace
all-target/all-feature Clippy with `-D warnings`, and full `cargo test` pass.
The full test includes both policy lint suites at `15/15`, metadata at
`137/137`, and the public-enum suite at `2/2`. Rust and completion-document
diff checks pass.

The five unchanged CLI stdout SHA-256 values are plan
`700f4bf503783742cefd8004fa095675b7476d46e9a3a6dd439916d237eb6718`,
parse
`a8a7aa639d2ebc65eddc923c7e9369ea5637d50e935f808600f446da1bfbda56`,
declaration
`71e83ba0b20d4015e07b3bd2c0c4db2837b6151d1251812caed7954530d53c74`,
type
`4b2c7bd5ec3cc56e5672fb351126126230ec84fba9bd2bd9049a516d378fab7f`,
and proof
`ccf3d2d4d0a3755e00989d97af369a7c560302f76798d0a185d57ec3891e8450`.
The existing CLI result remains 23 warnings and no errors.
Final-quality is **NO FINDINGS**; all `9/9` hard gates pass at valid uncapped
`100/100` (`20/20/15/15/10/10/5/5`). Exact staging/cached review passes over
23 paths (3 Rust / 20 docs), with one new private leaf, zero unstaged paths at
review time, cached diff check PASS, cached stat `1096/123`, and both lint
suites `15/15`. The task-only commit, immediate post-commit proof, and accepted
fresh successor STOP disposition are complete at the
[historical checkpoint](#historical-immediate-post-implementation-checkpoint).

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

Any hard-gate failure would invalidate the score. Final review confirms all
`9/9` gates and a valid uncapped `100/100` under the protocol rubric, with
category scores `20/20/15/15/10/10/5/5`.

## Exit criteria and next handoff

The implementation/completion phase exits only after the exact 23-path scope
is proved, EN canonical and JA companion are synchronized, every stable
fragment/link and legacy neighbor anchor resolves, both lint suites and diff
checks pass, protected hashes/counts remain unchanged, and
commit/post-commit/fresh-inventory steps are completed. Those lifecycle claims
are complete; every review, hard gate, and staging/cached gate is also
complete.

Keep Sol at `xhigh` for any separately authorized successor inventory. Use Terra
`xhigh` for any finding-specific bounded
re-review; raise back to Sol for any authority
ambiguity, requested public-API expansion, Task-252/capture/semantic boundary
question, or disputed finding. If Luna is not exposed, do not block.

The accepted fresh inventory returned a protocol semantic STOP and selected no
successor. Do not infer that the next task is capture, sethood, Task 252, or
Task 277B. Resume only through a separately frozen prerequisite satisfying the
conditions recorded below.

## Historical immediate post-implementation checkpoint

This documentation-only closure changes exactly 20 Markdown paths: this paired
contract plus the 18 existing owner documents linked above. The four checker
and mizar-test crate plans, the three implemented Rust paths,
`doc/design/spec_coverage_audit.md`, the compaction ledger, Cargo,
specification, fixtures, expectations, trace, coverage, and metadata are
unchanged by the closure.

Before the implementation commit, exact staging/cached review passed with 23
cached paths: 3 Rust and 20 documentation paths, including one new private
leaf, with zero unstaged paths at review time. `git diff --cached --check`
passed and the historical pre-commit cached stat was exactly 1096 insertions /
123 deletions.

The implementation commit's parent was
`53987a3fdc1a927dbcbd2b9ed22e9817c8b68f2d`. Immediately after the task-only
implementation commit `1b57f1dc97af2993603699d92820c2dd3e84ed0e`, the
observed `HEAD` was that commit, the worktree was clean,
`origin/main...HEAD` was `0/22`, and `stash@{0}` remained unchanged at
`f65cf4a13752ec380710814a9ac6392ccb9d75d4`. The commit contained exactly 23
paths (3 Rust and 20 documentation paths); its reproducible sorted-path SHA-256
was `2a718c8f3a4a7c9d58e5af0d129dd35550dd73f8efa6c5c3ecf7c32f7e05092e`,
and its final stat was 1117 insertions / 123 deletions. These are historical,
immediate-post-implementation observations from before this documentation-only
closure, not claims about any later or current `HEAD`, index, worktree, origin,
or stash state. The task-only commit and immediate post-commit proof are
closed.

The accepted fresh inventory returned a protocol semantic **STOP** and closed
the successor-inventory gate without selecting a successor:

- actual nested-comprehension capture remains a `test_gap`; immutable F5 has no
  outer-generator occurrence and therefore supplies no authority-backed capture
  oracle. Resume only after a separate test-first contract derives and freezes
  an oracle from canonical §13.4.4;
- Task 252 owns the existing occurrence ordinal/site public surfaces, while
  C4B owns no `SourcePrimaryTerm`, reference, site, or generic quantifier-use
  row. Reusing or duplicating those surfaces would be a `boundary_violation`.
  Resume only after a separately reviewed public-API owner resolves the
  ordinal/site association without duplicate ownership;
- template-type interpretation and sethood remain semantic-owner work under
  canonical §§13.4.2 and 18.10.2, and `MC-G020`/`MC-G021` remain open. Resume
  only after a separately frozen source-to-checker payload and sethood
  evidence/request bridge owns that composition and the required external gaps
  are discharged; and
- Task 277B remains not ready with zero semantic credit. No capture, Task-252,
  sethood, Task-277B, or other semantic successor is selected here.
