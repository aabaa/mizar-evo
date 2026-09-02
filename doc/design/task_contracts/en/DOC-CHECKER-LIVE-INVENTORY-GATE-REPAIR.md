# Task DOC-CHECKER-LIVE-INVENTORY-GATE-REPAIR: post-compaction hard-gate repair

> Canonical language: English. Japanese pointer:
> [../ja/DOC-CHECKER-LIVE-INVENTORY-GATE-REPAIR.md](../ja/DOC-CHECKER-LIVE-INVENTORY-GATE-REPAIR.md).

Owning plans: [mizar-checker](../../mizar-checker/en/00.crate_plan.md#task-index)
and [mizar-test](../../mizar-test/en/00.crate_plan.md#task-index).

## Frozen assignment

| Field | Value |
|---|---|
| Status | Complete at the commit containing this contract; reviews, 9/9 hard gates, and score 100/100 pass |
| Tier | Full; hard-gate repair with zero semantic credit |
| Owner / consumers | Checker live source ledger / checker lint; workspace rustfmt baseline / full-gate runner |
| Dependencies | Completed CPT-08/CPT-13; suspended consumer Step 5A.1 |
| Authority | [autonomous decisions](../../autonomous_crate_development.md#autonomous-design-decisions-and-stop-conditions), [single owner](../../autonomous_crate_development.md#single-owner-documentation-rule), [compaction Rule 2](../../documentation_compaction_rules.md#rule-2--tabular-ledgers-for-mechanical-measurements) |
| Classification | `design_drift`, non-production `test_gap`; no `spec_gap` or `repo_metadata_conflict` |

CPT-08/CPT-13 correctly froze historical checker audit bodies, but two live
lint tests still parsed the removed bodies as current inventories. One
inherited rustfmt-only expression drift also blocked the workspace format gate.
These derived defects do not change language/test intent, diagnostics,
recovery, soundness, public API, acceptance, or coverage credit.

## Frozen ledger and lint oracle

`doc/design/mizar-checker/checker_source_inventory.tsv` is the sole live
mechanical owner. Its exact prefix and schema are:

```text
# generator: crates/mizar-checker/tests/lint_policy.rs::checker_live_source_inventory_matches_repository
# source-of-truth: crates/mizar-checker/{src,tests,benches,examples}/**/*.rs plus crates/mizar-checker/build.rs when present
schema<TAB>1
path<TAB>lines<TAB>boundary<TAB>owner_doc<TAB>split_required<TAB>hard_gate_finding<TAB>decision<TAB>public_surface
```

It has one row for each lexically sorted Rust source/test-support file. Paths
are unique and normalized; line counts are exact. `src/lib.rs` maps to
`crate-exports`, the EN crate plan, `keep-current-boundary`, and exact
source-order `modules:`. Other `src/<stem>.rs` rows map to `module:<stem>`,
`keep-current-boundary`, exact sorted recognized `items:`, and the matching live
EN module doc when present or the plan otherwise. Non-production rows map to
`test-support`, the plan, `keep-test-support`, and `none`. Both finding fields
are `no`. Traversal/archive/nonexistent/wrong owners and every empty,
malformed, duplicate, missing, extra, reordered, stale, or mismatched value
fail closed.

The two stale lint tests consume this ledger. In-memory mutation vectors cover
the exact comments/schema/header/newline, field arity/emptiness/numeric counts,
file set/order/duplicates, path-derived boundary/owner/decision, owner safety,
split/hard findings, module exports, and public items. Helpers used only by the
old Markdown-body inventories are removed.

Live EN audit skeletons link the ledger for mechanical state and their archives
for history; JA remains pointer-only. The stale "gap rows below" sentence uses
`00.crate_plan.md#known-gaps-and-drift` in EN and
`../en/00.crate_plan.md#known-gaps-and-drift` in JA; those fragments must pass.
The crate plan remains the sole MC-G status owner. `split_required=no` plus
`decision` is the complete live boundary oracle. `metadata.rs` receives only
rustfmt's mechanical wrapping; assertions, branches, constants, and corpus
metadata are unchanged.

## Scope, gates, and exit

Scope is this pair, both owning plans' EN/JA Task Index rows, the TSV, the two
live EN/JA checker audit pairs, checker `lint_policy.rs`, and the one
`metadata.rs` format hunk. `spec_coverage_audit.md` has no impact and remains
unchanged. Forbidden: `doc/spec/**`, `.miz`, expectations, trace/activation,
production/API/Cargo behavior, completed contracts, `legacy_compactions.tsv`,
and `doc/design/archive/**`; archive snapshots are not current lint oracles.

Contract/boundary, test, implementation, and source/docs/API reviews ended
without findings after finding-specific repairs. Checker lint 17/17, test link
lint 15/15, checker 582+17, focused metadata, fmt, workspace warnings-denied
Clippy, and full tests pass. All nine hard gates pass; independent and parent
scores are 100/100 with no cap. Activation/gap/audit/archive hashes remain
`e9a1c2e6…`, `a0d161de…`, `9e75f8ea…`, and `934df58f…`. Exit is exact local
commit plus clean postcommit proof; main then fast-forwards without disturbing
Step 5A.1. No push, PR, external write, stash, destructive action, or semantic
credit.
