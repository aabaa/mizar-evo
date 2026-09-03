# Task STEP5B1-STRUCTURAL-RULE-CONSOLIDATION: structural task retirement

> Canonical language: English. Japanese pointer:
> [../ja/STEP5B1-STRUCTURAL-RULE-CONSOLIDATION.md](../ja/STEP5B1-STRUCTURAL-RULE-CONSOLIDATION.md).

Owning plans: [mizar-checker](../../mizar-checker/en/00.crate_plan.md#task-index)
and [mizar-test](../../mizar-test/en/00.crate_plan.md#task-index).

## Frozen assignment

| Field | Value |
|---|---|
| Status | Complete; light gates passed, pending local commit |
| Tier | Light: zero-credit structural documentation transport |
| Owner / consumers | `mizar-checker` [type checker design](../../mizar-checker/en/type_checker.md) solely owns the Task-74 structural rule; `mizar-test` consumes it |
| Dependencies | Step 5A.1-5A.9 complete; Step 5 decomposition rule frozen |
| Authority | [Step 5 decomposition rule](../../todo.md#step-5--source-derived-semantic-bridge-) and [Step 5B.1](../../todo.md#step-5b--consolidation-and-pending-prerequisites--) |
| Classification | `design_drift`; no `spec_gap`, `test_gap`, or `repo_metadata_conflict` |
| Semantic-credit throughput | `0 tasks/week`; no requirement or oracle changes |

## Structural ownership rule

Task 74 remains the sole derived owner for the already-supported structural
product of AST-bounded bare local-mode chain depth, builtin terminal
(`set`/`object`), the five existing consumer shapes (equality,
pre-desugaring inequality, right-expected membership, normalized-reflexive type
assertion, and same-symbol asserted head), and reserve multiplicity (one or two
bindings with shared or distinct written ranges). Its traversal budget remains
the source AST's mode-definition count and is a resource guard, not a semantic
depth limit.

Completed tasks and committed matrices that instantiate points in that product
remain regression evidence for their original routes. They do not own a new
structural dimension and are not precedents for another per-depth,
per-terminal, per-consumer-shape, or per-reserve-multiplicity task. A future
task requires an independently semantic dimension, a changed guard, or another
authority-backed behavior change; it cannot be justified solely by a new point
in this product.

Historical task sections, their original semantic consumer ownership, and
their credited coverage remain unchanged. This consolidation neither claims
general type equivalence/reachability nor weakens the exact provenance,
ordering, normalization, fail-closed, and isolation guards owned by those
routes.

The ownership split is exact: Task 74 owns only the structural producer and
the rule that another point in the product creates no task. The existing
consumer owners remain Tasks 166/167 for equality, 168/169 for pre-desugaring
inequality, 164/165 for right-expected membership, 152/153 for the four-edge
normalized-reflexive assertion representatives, and 186/187 for the
same-symbol asserted-head consumers; later relation-specific descendants keep
their own exact semantic/provenance ownership. These assignments are recorded
under checker [Task 8](../../mizar-checker/en/type_checker.md#task-8-declaration-and-local-binding-checking)
and are not transferred to Task 74.

Frozen evidence is the Task-74 [producer implementation](../../../../crates/mizar-test/src/runner/type_elaboration/source_reserve.rs),
[source-extraction](../../../../crates/mizar-test/src/runner/tests/type_elaboration/source_extraction.rs)
and [metadata](../../../../crates/mizar-test/tests/metadata.rs) tests, and the existing [harness contract](../../mizar-test/en/harness.md#consumer-runner-pacing).
They remain unchanged regression evidence; no new test or corpus oracle is
created by this documentation task.

## Scope and protected boundary

Allowed files are exactly this EN/JA contract pair, the paired checker
`type_checker.md`, the paired test `harness.md`, the four EN/JA checker/test
crate plans, and `doc/design/todo.md`. The checker design receives the sole
durable rule. The harness replaces its duplicate summary with a consumer link.
Both plans link this contract and record that task selection creates no new
structural-point task. The Step 5B.1 row records completion. The Japanese
contract remains a pointer skeleton; paired component documents stay synced.

Do not edit Rust sources, `doc/spec`, `.miz` sources, expectations, snapshots,
trace or activation metadata, `doc/design/spec_coverage_audit.md`, completed
task bodies, `legacy_compactions.tsv`, or `doc/design/archive/`. No active case,
diagnostic, public API, source behavior, owner, deferred status, or semantic
credit changes. The coverage audit has no impact because coverage ownership
and status are unchanged. Task 277B remains not-ready and zero-credit.

Frozen baselines are activation map `e9a1c2e6b0b444a2caed6a081b6c4a2ff780ee39d4e357c0268d3f8d7215a34b`, audit-1 gap ledger `a0d161dedd78450110fe25e11634f9cdde5f5f633d30965b887cd70aee383dd8`, coverage audit `9e75f8ea0f7a1ca81a88f47811f21a264803b16fb3101327caaed7c0925af285`, trace ledger `b69d5cce7c50fa99e882fd9a3dc4e5623a74537990fdf06c6d821018e3daf2d3`, and 13-file archive aggregate `934df58f26b3ea1903f9c476452055d7755001fa9e5786293c959e863da72160`.

Light-tier exit requires one independent equivalence review with no findings; exact commands `cargo test --offline -p mizar-checker --test lint_policy`, `cargo test --offline -p mizar-test --test lint_policy`, `cargo test --offline -p mizar-test --test metadata`, and `git diff --check`; exact protected checks; task-only commit; and clean postcommit proof. Any behavior, protected-surface, ownership, or credit change
promotes the task irreversibly to full gates.

## Completion evidence

Outcome: one checker-owned structural rule; all point matrices are regression-only and semantic credit remains `0 tasks/week`.
Reviews: initial contract findings were repaired; finding-specific contract and final equivalence/source-doc/boundary reviews ended with no findings.
Verification: checker lint 17/17, test lint 16/16, metadata 153/153, diff check, protected hashes, and the 13-file archive aggregate passed.
Tier/handoff: light tier passed without promotion; no full-tier numeric score applies. Step 5B.2 is next with Luna xhigh; no Step 5C task may be selected first.
