# Task DOC-COMPACT-MANIFEST-TASK-REF: Cross-Batch Historical-Task References

> Canonical language: English. Japanese companion:
> [../ja/DOC-COMPACT-MANIFEST-TASK-REF.md](../ja/DOC-COMPACT-MANIFEST-TASK-REF.md).

This derived documentation/test-policy prerequisite extends the legacy-
compaction ledger without changing language behavior, test intent, diagnostics,
public API, coverage, or any already completed migration. It exists so one
canonical historical task contract can receive several independently frozen
whole-section compaction batches with disjoint source-file sets, without moving
or duplicating that contract's ownership.

## Identity And Status

| Field | Frozen value |
|---|---|
| Task | `DOC-COMPACT-MANIFEST-TASK-REF` |
| Status | Documentation prerequisite committed as `31d35e71e809a51c421098e5c9c9eb2853a73a93`; generic schema-2 implementation, independent reviews, required verification, and final quality review are complete. Exact task-only staging and commit remain. |
| Primary owner | Repository legacy-compaction schema and its `mizar-test` lint consumer |
| Consumers | Later coherent batches that compact additional sections for an already registered historical task; first intended consumer `DOC-258B4C-IMPLEMENTATION-LEDGER-COMPACT` |
| Dependencies | `DOC-COMPACT-MANIFEST`, `DOC-COMPACT-PATH-SCOPE`, and the completed `DOC-258B4C-DOC-REVIEW-COMPACT` batch |
| Readiness | Dependency-ready. Current schema version 1 cannot express an additional Task-258B4C batch without violating its global task-row ownership rule; changing the completed earlier batch would violate its frozen migration boundary. |

The [checker plan](../../mizar-checker/en/00.crate_plan.md#task-index) and
[runner plan](../../mizar-test/en/00.crate_plan.md#task-index) index this
contract. The original [manifest contract](./DOC-COMPACT-MANIFEST.md) remains
the historical schema-1 implementation record; this contract owns the live
schema-2 delta.

## Authority And Classified Gap

Authority is the user's explicit checker-first consolidation direction,
[`AGENTS.md`](../../../../AGENTS.md), and the
[migration policy](../../autonomous_crate_development.md#migration-policy).
No `doc/spec/en/`, `.miz`, expectation, trace, or source behavior is authority
for this derived policy task.

| Class | Decision |
|---|---|
| `design_drift` | Schema 1 globally binds each task row to one batch, although a canonical historical task may retain several disjoint coherent duplication families. |
| `boundary_violation` | Avoided: the extension does not mutate a completed batch, duplicate historical ownership, or make a ledger row authorize deletion. |
| `test_gap` | The lint has no positive two-batch/one-task vector or fail-closed vectors for undeclared, duplicate, self-owned, or wrong-batch task references. |
| `spec_gap` | None; this task defines derived repository policy only. |
| `source_drift`, `source_undocumented_behavior`, `test_expectation_drift` | None; production and semantic artifacts are unchanged. |
| `repo_metadata_conflict` | None. At selection, HEAD is `1d32ed06cc110ed98e9116dd59af82e9ef724b15`, the worktree is clean, `origin/main...HEAD` is `0/9`, and protected `stash@{0}` remains `f65cf4a13752ec380710814a9ac6392ccb9d75d4`. The ahead state is report-only and push remains out of scope. |

## Documentation-Prerequisite Boundary

This prerequisite changes exactly this EN/JA pair and one Task Index row in
each checker/test EN/JA crate plan. It changes no `AGENTS.md`, protocol,
design index, existing task contract, manifest, Rust, Cargo file,
specification, fixture, sidecar, expectation, trace, coverage audit, redirect,
source-section count, or hash. Task-contract Markdown counts move from 57/57 to 58/58; the
864-line schema-1 ledger and its physical SHA-256
`876dbd36c52952d029257d3c16d0ae8f7cb2a9fac9f09fedafae6c4df3e026bf`
remain unchanged.

## Documentation-Prerequisite Evidence

The first policy review found an overbroad cross-batch claim and an ambiguous
old-contract boundary; both were narrowed to disjoint source-file sets and the
schema-owner notice. The bilingual re-review then found and removed an
incorrect document-global one-redirect claim. Final policy and EN/JA re-reviews
report **NO FINDINGS**.

The focused and full 15-test lint-policy target, recursive local links and
fragments, `cargo fmt --all --check`, offline Cargo metadata,
`cargo clippy --all-targets --all-features -- -D warnings`, and full workspace
`cargo test` pass. All five corpus CLIs exit zero with the unchanged hashes
`700f4bf503783742cefd8004fa095675b7476d46e9a3a6dd439916d237eb6718`,
`a8a7aa639d2ebc65eddc923c7e9369ea5637d50e935f808600f446da1bfbda56`,
`71e83ba0b20d4015e07b3bd2c0c4db2837b6151d1251812caed7954530d53c74`,
`4b2c7bd5ec3cc56e5672fb351126126230ec84fba9bd2bd9049a516d378fab7f`,
and `ccf3d2d4d0a3755e00989d97af369a7c560302f76798d0a185d57ec3891e8450`
for plan/parse/declaration/type/proof respectively; their existing 23 warnings
and zero errors are unchanged. Protected specification, test, expectation,
trace, coverage, source, Cargo, manifest, and stash surfaces are unchanged.
`git diff --check` passes.

## Frozen Schema-2 Delta

Implementation changes the first data row from `schema<TAB>1` to
`schema<TAB>2` and adds exactly one record kind:

| Kind | Fields after kind |
|---|---|
| `task_ref` | referencing batch ID; existing canonical historical task ID |

All schema-1 records and their byte grammar remain unchanged. A `task` row
continues to be the sole global owner of one task ID and its EN/JA historical
contract paths. A `task_ref` is only a batch-local relationship to that
existing owner; it carries no contract path and cannot replace or duplicate a
`task` row. Its exact rules are:

1. `(batch ID, task ID)` is unique. Both IDs use the existing ID grammar.
2. The batch and canonical `task` row must exist. The referenced task must be
   owned by a different batch; an owner batch must use its existing `task` row.
3. A batch's existing `task count` field counts distinct participating tasks:
   task rows owned by that batch plus task references declared by that batch.
4. Each redirect must name either a task owned by its batch or a `task_ref`
   declared by its batch. Resolution always uses the one canonical task row's
   language-local contract and `#completion-evidence`.
5. A Task Index record for a historical task remains owned by the task's
   original batch. A referencing batch indexes only its own batch contract;
   it must not add or claim a second historical-task index row.
6. `task_ref` rows participate in their referencing batch's canonical expanded
   inventory bytes and hash. Existing batch inventories and hashes remain
   byte-identical because they gain no references.
7. The record is enforcement metadata only. The referencing batch contract,
   historical task migration boundary, exact source inventory, and equivalence
   review must independently authorize every redirect.
8. For one canonical task, every referencing/owning batch must use a source-file
   set disjoint from every other batch for that task. The existing unique
   `(source path, task ID)` redirect key remains unchanged and enforces this
   restriction; other tasks may still own distinct redirects in the same file.

Schema 2 still represents only exact whole ATX H2-H6 section replacement.
Multiple sections for one task in the same source file, paragraph-level
removal, and mixed owner-local removal remain unrepresentable and still require
a separate reviewed prerequisite with an occurrence-safe evidence identity.

## Frozen Lint And Test Delta

The sole consumer remains
`task_contracts_are_recursively_paired_and_supported_links_resolve`; the test
name and the 15-test lint-policy list remain unchanged. The parser and relation
checker must accept the new strict three-field record, include it in canonical
inventory bytes and participating-task counts, and apply the rules above
without any task ID, source path, or batch-specific branch in Rust.

Focused same-test vectors must prove:

- two batches may use one canonical task, with the second batch declaring one
  reference and resolving a redirect through the original EN/JA contract;
- duplicate `(batch, task)` references, undeclared batches, undeclared tasks,
  references from the owner batch, wrong field counts, and invalid IDs fail;
- a redirect in another batch fails without the exact reference and succeeds
  only with it;
- a second batch for the same task fails when its redirect reuses any source
  path already owned by another batch for that task;
- a referenced historical task cannot be indexed again by the referencing
  batch; and
- count and expanded-inventory-hash mutations still fail closed.

The current lint-policy raw list hash is
`b044e771a655e72131d0371636bbac5684ef93a3ea503984537a4bb9dd13a7cf`.
The implementation changes no test count. With only the schema line changed,
the 864-line ledger's expected physical SHA-256 is
`b7e9a943afcca7ee4773e6ac472e8a350624d17f96dbb54ca821fcb1f57d56cc`;
all 21 current batch hashes, 33 canonical task rows, 592 redirects, and 216
index rows remain unchanged, and the implementation adds zero `task_ref` rows.

## Implementation Scope And Prohibitions

After the prerequisite commit, implementation changes exactly these nine
paths: this EN/JA pair; `AGENTS.md`; `doc/design/README.md`;
`doc/design/autonomous_crate_development.md`; the EN/JA
`DOC-COMPACT-MANIFEST` pair with only a schema-2 supersession notice;
`doc/design/task_contracts/legacy_compactions.tsv`; and
`crates/mizar-test/tests/lint_policy.rs`. The four Task Index rows from this
prerequisite remain unchanged.

It must not add a B4C reference or batch, migrate any legacy section, edit a
completed batch/source inventory, change production code or public API, alter
Cargo dependencies, or touch `doc/spec/**`, `.miz`, fixtures, sidecars,
expectations, traceability, coverage status, diagnostics, CLI behavior, or
protected `stash@{0}`. `doc/design/spec_coverage_audit.md` has no ownership or
coverage impact and remains unchanged.

All completed historical migration task/batch contracts and their registered
inventories remain unchanged. The earlier `DOC-COMPACT-MANIFEST` pair is the
schema owner, not a migration batch; its only delta is the explicit live-policy
supersession notice above.

## Implementation Evidence

- The implementation changes exactly the frozen nine paths. The ledger changes
  only `schema<TAB>1` to `schema<TAB>2`; it remains 864 lines with physical
  SHA-256
  `b7e9a943afcca7ee4773e6ac472e8a350624d17f96dbb54ca821fcb1f57d56cc`,
  21 batches, 33 canonical tasks, zero task references, 592 redirects, and 216
  indexes. All existing batch rows and expanded-inventory hashes are unchanged.
- The generic lint parser owns a unique `(batch ID, task ID)` reference set,
  includes each reference in the declaring batch's participating-task count and
  canonical inventory, requires an existing different owner batch, authorizes
  an owner-batch redirect or a cross-batch redirect only with the exact
  reference, preserves original Task Index ownership, and keeps the existing
  `(source path, task ID)` collision boundary. Rust contains no historical task,
  batch, or source-path allowlist.
- Same-test synthetic EN/JA vectors accept the exact two-batch/one-task route
  and reject duplicate, undeclared, self-owned, malformed, invalid-ID,
  unreferenced redirect, reused same-task source, and repeated-index cases.
  Test-sufficiency, full-implementation, and source/documentation/EN-JA reviews
  independently report **NO FINDINGS**.
- The focused and full 15-test lint-policy target pass with unchanged raw list
  hash
  `b044e771a655e72131d0371636bbac5684ef93a3ea503984537a4bb9dd13a7cf`.
  `cargo fmt --all --check`, offline metadata,
  `cargo clippy --all-targets --all-features -- -D warnings`, and full workspace
  `cargo test` pass. All five CLI routes exit zero with the unchanged hashes
  recorded under Documentation-Prerequisite Evidence and the same 23 warnings/
  zero errors. Protected specification, `.miz`, expectation, trace, coverage,
  production, public-API, Cargo, and active-result surfaces are unchanged;
  `git diff --check` passes.
- During implementation, `origin/main` moved externally through an `update by
  push` from ten commits behind to `31d35e71e809a51c421098e5c9c9eb2853a73a93`.
  The agent did not push. This is a report-only `repo_metadata_conflict`; HEAD,
  the exact nine-path task diff, and protected stash identity remain unambiguous,
  so no metadata repair is attempted.
- Final read-only quality review reports **NO FINDINGS**, all nine hard gates
  PASS, no score cap, and **100/100**. The residual first-real-ledger-row risk is
  bounded by the synthetic EN/JA positive route and its fail-closed mutation
  matrix; the later B4C batch must still replay the actual data path separately.

## Reviews, Verification, And Exit

Independent reviews cover specification/policy completeness, EN/JA logical
equivalence, test sufficiency, implementation correctness, and source/docs
consistency. Findings are fixed and the relevant review repeated to
**NO FINDINGS**. Final read-only review requires all nine autonomous hard gates
PASS, no score cap, and at least 90/100.

Verification includes the focused 15-test lint-policy target, manifest
mutation vectors, local links/fragments, `cargo fmt --all --check`, offline
metadata, `cargo clippy --all-targets --all-features -- -D warnings`, full
workspace `cargo test`, the five repository CLI routes, protected path/count/
content hashes, `git diff --check`, exact staged-content review, and clean
post-commit HEAD/origin/stash inventory.

Exit requires the generic schema-2 implementation to be committed alone with
the frozen counts and hashes reproduced, no B4C-specific data present, and all
reviews/gates passing. Fresh inventory then returns to the exact checker TODO
pair for `DOC-258B4C-IMPLEMENTATION-LEDGER-COMPACT`: EN lines 5934-5958,
25 lines, SHA-256
`b3232c301dc8df4b6da3cccb4d040c9a819b8931ed31d20e311ca574f86ba82e`;
JA lines 5670-5693, 24 lines, SHA-256
`200dcfb5ecd4e44ea25254d70c049338a211009d28c89cc05c147541e727417f`.
All checker plans, owner/audit documents, lower-stage ledgers, and every runner
document remain excluded from that later two-section family.
