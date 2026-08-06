# Task DOC-COMPACT-PATH-SCOPE: Path-Scoped Legacy Heading Enforcement

> Canonical language: English. Japanese companion:
> [../ja/DOC-COMPACT-PATH-SCOPE.md](../ja/DOC-COMPACT-PATH-SCOPE.md).

## Identity And Status

| Field | Frozen value |
|---|---|
| Task | `DOC-COMPACT-PATH-SCOPE` |
| Status | Documentation contract frozen; implementation and separate commit remain. |
| Purpose | Make schema-v1 forbidden-heading enforcement honor each redirect's declared source path. |
| Authority | `AGENTS.md` exact-source-path legacy-compaction rule, schema-v1 manifest fields, and the B4A duplicate-heading `test_gap` |
| Consumer | Generic legacy-compaction validation in mizar-test lint policy |
| Plan indexes | [checker plan](../../mizar-checker/en/00.crate_plan.md#task-index) and [runner plan](../../mizar-test/en/00.crate_plan.md#task-index) |

## Frozen Scope And Behavior

Implementation changes only `crates/mizar-test/tests/lint_policy.rs`. The
forbidden-heading check must use each manifest redirect's exact
`(source_path, legacy_heading)` pair instead of a repository-global heading
set. It must continue rejecting a legacy heading in its declared source while
allowing identical text in an unrelated unselected Markdown document.

Add deterministic regression coverage for both cases. Retain every existing
schema, batch/task relation, redirect grammar, language-local target, fragment,
neighbor anchor, source cardinality, sorted order, and inventory-hash check.
This is a lint correctness repair, not a schema extension.

Do not change `AGENTS.md`, the autonomous protocol, documentation, manifest
data, source inventories, production Rust, specifications, corpus fixtures,
expectations, traceability, coverage, Cargo, language behavior, test intent,
or any currently registered compaction result. Do not special-case task IDs,
paths, or headings in Rust.

## Reviews, Verification, And Exit

Require independent specification/design, test-sufficiency, implementation,
and source/document consistency reviews at **NO FINDINGS**. All nine hard gates
must PASS without a score cap at `>=90/100`. Run focused lint regressions, full
mizar-test lint, formatting, warnings-denied Clippy, workspace tests, and
`git diff --check`; the already committed contract/index prerequisite remains
unchanged, and the separately reviewed implementation commit stages only
`crates/mizar-test/tests/lint_policy.rs`. No push, fetch, reset, or stash
mutation.

Handoff: after its clean commit, fresh-replay DOC-258B4A-COMPACT sources and
continue that separately frozen migration.
