# Task DOC-258B4A-COMPACT: B4A Implementation-Evidence Compaction

> Canonical language: English. Japanese companion:
> [../ja/DOC-258B4A-COMPACT.md](../ja/DOC-258B4A-COMPACT.md).

## Identity And Status

| Field | Frozen value |
|---|---|
| Task | `DOC-258B4A-COMPACT` |
| Status | Documentation prerequisite and lower lint contract implemented; all reviews and verification complete; exact staging and commit remain. Migration is blocked on [DOC-COMPACT-PATH-SCOPE](./DOC-COMPACT-PATH-SCOPE.md). |
| Purpose | Centralize four repeated Task-258B4A implementation-completion sections while retaining frozen contracts and durable owners. |
| Historical owner | [Task 258B4A](./258B4A.md#completion-evidence) |
| Plan indexes | [checker plan](../../mizar-checker/en/00.crate_plan.md#task-index) and [runner plan](../../mizar-test/en/00.crate_plan.md#task-index) |
| Selection HEAD | `75d8af2d5e071f415d1cada9e1a8981aaef2d3b2` |
| Repository state | clean `main`, `origin/main...HEAD=0/4`, protected `stash@{0}=f65cf4a13752ec380710814a9ac6392ccb9d75d4` |

## Authority And Classification

Authority is the user-approved checker-first compaction program, `AGENTS.md`,
the autonomous migration policy, reviewed Git history, the selected completed
sections, and their surviving durable owners. Source behavior is not normative.

| Class | Decision |
|---|---|
| `design_drift` | Plan and TODO repeat historical B4A implementation/review/count/hash/commit evidence without one historical owner. |
| `test_gap` | Generic ledger lint currently treats declared legacy headings as repository-global. The selected TODO headings also occur in unselected mizar-test owners, so a path-scoped regression is required before migration. |
| `boundary_violation` | Tasks 266--268 were rejected because their sections contain and anchor the registered Task-247 redirect. Current global heading lint also crosses the B4A source boundary. Only the four flat B4A completion sections may migrate. |
| `spec_gap` / `source_drift` | None introduced or repaired; historical state remains with its durable owners. |
| `source_undocumented_behavior` / `test_expectation_drift` | None inferred or changed. |
| `repo_metadata_conflict` | The branch is four commits ahead of observed `origin/main`; report only, do not repair. |

## Frozen Sources And Anchors

[`DOC-258B4A-COMPACT.sources.tsv`](../DOC-258B4A-COMPACT.sources.tsv)
contains four byte-sorted data rows, two comments, and final LF. Data-row
SHA-256 is `7892258a006395a7372b4a30195cfe53043782569039bcbb716e1a6660fb1062`;
complete-file SHA-256 is
`73b007dda0100274c678c8a751dbb136604cc7284ffd3046f55a977b433488a4`.
The sections are flat, source-locally unique, unlinked, and total 154 physical
lines: EN/JA plan `52/52`, EN/JA TODO `25/25`.

| Source | Previous H2 | Next H2 |
|---|---|---|
| EN plan | `## Task 258B4A Frozen Explicit-Universal Composite Theorem Root` | `## Task 258B4B Frozen Connective/Grouping Composite Theorem Root` |
| JA plan | `## Checker Task 258B4A composite-theorem-root frozen contract` | `## Task 258B4B frozen connective/grouping composite theorem root` |
| EN TODO | `## Checker Task 258B4A Documentation Prerequisite` | `## Checker Task 258B4B Documentation Prerequisite` |
| JA TODO | `## Checker Task 258B4A documentation prerequisite` | `## Checker Task 258B4B documentation prerequisite` |

Implementation commit `662adbde` introduced the completion sections. Successor
prerequisite `b8a7b8257` added only the immutable commit/post-inventory tail and
the following B4B headings; current `git blame` attributes every selected line
to those two commits.

## Retained Owners And Exclusions

The B4A frozen plan section and all source-statement, formula-composition,
Typed/Resolved AST, payload-family, source/specification, boundary, bilingual,
runner, traceability, and coverage owners remain unchanged. The similarly
named mizar-test TODO section is a distinct unselected owner. B4B onward,
Tasks 265--268, specifications, `.miz`, expectations, sidecars, traceability,
coverage audit, production, Cargo, APIs, and active behavior are forbidden.

## Protected Baseline

Expected prerequisite and migration delta is zero:

| Surface | Paths | Path SHA-256 | Content SHA-256 |
|---|---:|---|---|
| specification | 64 | `d900ba9e43ab094925f36493f830e0c6a2964be2911d5d229014a58842067a25` | `b30dd5519191a4407399826faf91cc58853d7944df7630734b5ab05de48c9f7b` |
| `.miz` | 343 | `d94980e167b4b8ac196f91e7694ff044080c6fb4d04c135b3cd5e65b9a019f22` | `54e6ea1254a0bd963c39026d788711507f353e9c6df3b4f9fd268b2e9f240afb` |
| expectation | 435 | `22a5ee257a294e3f2ed4b24bf9ca243d037bbd798f009d0d1e3a176dad8b4fea` | `b5f0ed1a8d73bbfb78af5cad87a8d426e97269f7f70163750893a9fe1f39d2ea` |
| checker production | 30 | `a41370d7150a587369cea5f7a67b60417dd1372592f55c0d65bec369eb39fdc6` | `05fd5e0eaed4361b824693941e9056a552c476f050915ea5052a85c8c7174dfd` |
| runner production | 90 | `05245a54160dfce17336b476b07885eb6d5afe138c4780a6a6a7b47043e7248c` | `210f294aebfe22c12324ef9919ac68147f8025f0da8de166403dada87bac5eae` |
| Cargo | 21 | `d93f2816b760d8ba45430d6d8570e598864aa7b20b19a001f45171d36fd3a030` | `146e9b4024d2c344b2eca9c6f5ca6d6a80a3de427e382953a2280bc63cb3ecca` |

Trace remains `55b754c8c4d0d293a1c44e2ba4b0090f407bba1d429b461b6cb4d6ddca9ca2b3`;
coverage audit remains `2aa808aa033c04fa46ea76f65cfadcf0d8f0e1e53fba6d70edab223a7481685f`.
The 836-line ledger baseline is
`33c569ebeac13be3f353177f6c23ddf40c581435950e0e47f57bcdcd7f3528cb`.
Five CLI stdout hashes remain the values recorded by the immediately preceding
Task-247 compaction contract.

## Prerequisite And Expected Migration

The prerequisite changes exactly eleven paths: this pair, historical
Task-258B4A pair, the lower-lint contract pair, source TSV, and four plan Task
Indexes. Each plan adds Task `258B4A`, this batch, and the lower-lint task:
twelve index records total. Selected sources and ledger stay unchanged.

Before migration, separate
[Task DOC-COMPACT-PATH-SCOPE](./DOC-COMPACT-PATH-SCOPE.md) must implement its
path-scoped generic-lint correction and regression in a separate commit.

After both prerequisite commits and fresh replay, migration changes exactly seven
paths: four source documents, this pair, and `legacy_compactions.tsv`. Four
sections become four language-local redirects to `258B4A.md#completion-evidence`.
Exact source diff is `+4/-150`, reducing 154 selected lines to eight redirect-
plus-separator lines.

Ledger impact is 14 lines, `836 -> 850`: one batch, one task, four redirects
over four source paths, and eight index records. The canonical 13-row payload
SHA-256 is `6e082203fc14fa303969e13d1deebd3b630adbb3052b67019a874b3ed2643f2d`;
expected physical ledger SHA-256 is
`7bd738ad591a40667cb95421dd68d386213c25c51274cbf5c79d8f24b0b1688a`.
`spec_coverage_audit.md` has no impact because mappings, ownership, status,
deferred reasons, and credit do not change.

## Reviews, Verification, And Exit

Prerequisite and migration separately require equivalence, schema/test-
sufficiency, bilingual/boundary, and final-quality review as applicable, all
ending **NO FINDINGS**. All nine hard gates must PASS, no score cap, with a
valid score at least `90/100`.

Verification includes preimage/history/anchor replay; generic recursive task-
contract/link/fragment/ledger lint; checker/runner lint and libraries; runner
metadata; formatting; offline Cargo metadata; warnings-denied all-target/all-
feature Clippy; full workspace tests; all five CLIs; protected count/hash;
ledger order/hash/cardinality; `git diff --check`; exact cached review; and
unstaged/untracked inspection. No push, fetch, reset, or stash mutation.

Prerequisite exits with exact eleven-path scope, unchanged sources/ledger/
protected owners, synchronized EN/JA, complete reviews/verification, one commit,
and clean replay. Migration exits separately with exact four redirects/seven
paths, ledger replay, all gates, one commit, and clean replay.

## Documentation-Prerequisite Evidence

Independent contract/equivalence, schema/test-sufficiency, and bilingual/
boundary reviews all end **NO FINDINGS**. Their first passes identified missing
runner-owner links and the generic global-heading lint defect. The synchronized
historical contracts now link the retained runner owners, and the latter
`test_gap`/`boundary_violation` is frozen as the separate path-scoped lint task
above. Re-review also corrected that task's future staging boundary to its one
lint-test file. No selected source, ledger row, protected owner, or coverage
claim changed.

Checker and runner lint pass `15/15` each, checker and runner libraries pass
`530/530` and `600/600`, and runner metadata passes `137/137`. Formatting,
offline Cargo metadata, warnings-denied all-target/all-feature Clippy, the full
all-target/all-feature workspace suite (including all three long frontend
benchmarks), generic recursive task-contract/link/fragment/ledger lint, and
`git diff --check` pass. All five CLIs exit zero with 23 known warnings and
zero errors each. Their stdout hashes reproduce as plan
`700f4bf503783742cefd8004fa095675b7476d46e9a3a6dd439916d237eb6718`,
parse `a8a7aa639d2ebc65eddc923c7e9369ea5637d50e935f808600f446da1bfbda56`,
declaration `71e83ba0b20d4015e07b3bd2c0c4db2837b6151d1251812caed7954530d53c74`,
type `4b2c7bd5ec3cc56e5672fb351126126230ec84fba9bd2bd9049a516d378fab7f`,
and proof `ccf3d2d4d0a3755e00989d97af369a7c560302f76798d0a185d57ec3891e8450`.

The four selected sections reproduce the frozen `52/52/25/25` line counts and
hashes, and the source TSV reproduces both frozen hashes. All twelve Task Index
records are present. Protected counts, path hashes, and content hashes reproduce
the baseline exactly with zero protected diff. Trace, coverage audit, and the
unchanged 836-line ledger reproduce their frozen hashes. Final independent
read-only quality review ends **NO FINDINGS**, passes all nine hard gates,
applies no score cap, and assigns **100/100** (`20/20/15/15/10/10/5/5`). Its
independent replay also reproduces the prospective 13-row canonical hash and
850-line physical ledger hash. The classified residual risk is the current
global-heading lint defect: migration remains blocked until the separately
contracted path-scoped correction passes and commits. Exact staging, commit,
and clean post-commit replay remain.

## Handoff

Complete and commit this documentation prerequisite only. Then implement and
commit `DOC-COMPACT-PATH-SCOPE` separately, fresh-replay the four sections, and
perform the frozen migration. Parent remains `xhigh`; bounded independent
reviews use `high`.
