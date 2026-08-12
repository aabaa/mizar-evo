# Task DOC-258B4B-COMPACT: B4B Implementation-Evidence Compaction

> Canonical language: English. Japanese companion:
> [../ja/DOC-258B4B-COMPACT.md](../ja/DOC-258B4B-COMPACT.md).

## Identity And Status

| Field | Frozen value |
|---|---|
| Task | `DOC-258B4B-COMPACT` |
| Status | Complete. The migration is registered in the schema-2 ledger; task-local completion evidence below preserves the committed migration and clean replay. |
| Purpose | Centralize four repeated Task-258B4B task-wide implementation-completion sections while retaining frozen contracts and durable checker/runner owners. |
| Historical owner | [Task 258B4B](./258B4B.md#completion-evidence) |
| Plan indexes | [checker plan](../../mizar-checker/en/00.crate_plan.md#task-index) and [runner plan](../../mizar-test/en/00.crate_plan.md#task-index) |
| Selection HEAD | `fee14f18c2301b1523250f25843d96b91f759b8e` |
| Repository state | clean `main`, `origin/main...HEAD=0/7`, protected `stash@{0}=f65cf4a13752ec380710814a9ac6392ccb9d75d4` |

## Authority And Classification

Authority is the user-approved checker-first compaction program, `AGENTS.md`,
the autonomous migration policy, reviewed Git history, the four selected
completed sections, and their surviving durable owners. Source behavior is
not normative.

| Class | Decision |
|---|---|
| `design_drift` | Checker plan and TODO repeat historical B4B implementation/review/count/hash/commit evidence without one historical owner. |
| `test_gap` | None; path-scoped generic legacy-heading validation and its regression are already committed in `fa7c3acf89e2d66c1f9f21fd515da650f6226304`. |
| `boundary_violation` | Selecting source-statement, audit, or runner sections would mix durable owner-local evidence. Only the four checker plan/TODO sections may migrate. |
| `spec_gap` / `source_drift` | None introduced or repaired; historical state remains with its durable owners. |
| `source_undocumented_behavior` / `test_expectation_drift` | None inferred or changed. |
| `repo_metadata_conflict` | The branch is seven commits ahead of observed `origin/main`; report only, do not repair. |

## Frozen Sources And Anchors

[`DOC-258B4B-COMPACT.sources.tsv`](../DOC-258B4B-COMPACT.sources.tsv)
contains four byte-sorted data rows, two comments, and final LF. Data-row
SHA-256 is `78395a61a864bbe0fb361151bb998bbba25d81d89dc0ca5307d9fe1166687485`;
complete-file SHA-256 is
`ada3f07eaf309a3e91c210599481738a6074c936a686356db7bfe4ae6424e546`.
The sections are flat, source-locally unique, unlinked, contain no registered
redirect, and total 207 physical lines: EN/JA plan `76/70`, EN/JA TODO
`32/29`.

| Source | Previous H2 | Next H2 |
|---|---|---|
| EN plan | `## Task 258B4B Frozen Connective/Grouping Composite Theorem Root` | `## Task 258B4C Frozen Restricted/Existential/Nested Theorem Root` |
| JA plan | `## Task 258B4B frozen connective/grouping composite theorem root` | `## Task 258B4C Frozen Restricted/Existential/Nested Theorem Root` |
| EN TODO | `## Checker Task 258B4B Documentation Prerequisite` | `## Checker Task 258B4C Documentation Prerequisite` |
| JA TODO | `## Checker Task 258B4B documentation prerequisite` | `## Checker Task 258B4C documentation prerequisite` |

Implementation commit `752c17ae7d552d5268d1028612b8174e480b6f3e`
introduced the completion bodies. Successor prerequisite
`3c723316ae632a867d29e8f4fc36348be30df202` added only the immutable
post-commit/B4C handoff tails and following B4C headings; current `git blame`
attributes every selected line to those two commits.

## Retained Owners And Exclusions

The B4B frozen plan section and every checker source-statement,
formula-composition, Typed/Resolved AST, payload-family, source/specification,
boundary, and bilingual owner remain unchanged. Every runner plan/TODO,
harness, boundary, and bilingual section also remains unchanged as durable
runner evidence, even where it repeats task-wide facts. The four registered
B4A redirects and all of their required neighboring anchors remain outside
the selected sections.

B4C onward, specifications, `.miz`, expectations, sidecars, traceability,
coverage audit, production, Cargo, APIs, and active behavior are forbidden.
The pre-existing status wording in the separate
`DOC-COMPACT-PATH-SCOPE` contract is not reopened by this task.

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
coverage audit remains
`2aa808aa033c04fa46ea76f65cfadcf0d8f0e1e53fba6d70edab223a7481685f`.
The 850-line ledger baseline is
`7bd738ad591a40667cb95421dd68d386213c25c51274cbf5c79d8f24b0b1688a`.
Five current CLI stdout hashes are plan
`700f4bf503783742cefd8004fa095675b7476d46e9a3a6dd439916d237eb6718`,
parse `a8a7aa639d2ebc65eddc923c7e9369ea5637d50e935f808600f446da1bfbda56`,
declaration `71e83ba0b20d4015e07b3bd2c0c4db2837b6151d1251812caed7954530d53c74`,
type `4b2c7bd5ec3cc56e5672fb351126126230ec84fba9bd2bd9049a516d378fab7f`,
and proof `ccf3d2d4d0a3755e00989d97af369a7c560302f76798d0a185d57ec3891e8450`.

## Prerequisite And Expected Migration

The prerequisite changes exactly nine paths: this pair, historical
Task-258B4B pair, source TSV, and four plan Task Indexes. Each plan adds Task
`258B4B` and this batch, eight index records total. Selected sources and ledger
stay unchanged.

After prerequisite commit and fresh replay, migration changes exactly seven
paths: four source documents, this pair, and `legacy_compactions.tsv`. Four
sections become four language-local redirects to
`258B4B.md#completion-evidence`. Exact source diff is `+4/-203`, reducing 207
selected lines to eight redirect-plus-separator lines.

Ledger impact is 14 lines, `850 -> 864`: one batch, one task, four redirects
over four source paths, and eight index records. The canonical 13-row payload
SHA-256 is `13f7b68977d3d669173e987662276d18bec940cbd484089a561c2fec390cb55a`;
expected physical ledger SHA-256 is
`876dbd36c52952d029257d3c16d0ae8f7cb2a9fac9f09fedafae6c4df3e026bf`.
`spec_coverage_audit.md` has no impact because mappings, ownership, status,
deferred reasons, and credit do not change.

## Reviews, Verification, And Exit

Prerequisite and migration separately require equivalence,
schema/test-sufficiency, bilingual/boundary, and final-quality review as
applicable, all ending **NO FINDINGS**. All nine hard gates must PASS, no score
cap, with a valid score at least `90/100`.

Verification includes preimage/history/anchor replay; generic recursive task-
contract/link/fragment/ledger lint; checker/runner lint and libraries; runner
metadata; formatting; offline Cargo metadata; warnings-denied all-target/all-
feature Clippy; full workspace tests; all five CLIs; protected count/hash;
ledger order/hash/cardinality; `git diff --check`; exact cached review; and
unstaged/untracked inspection. No push, fetch, reset, or stash mutation.

Prerequisite exits with exact nine-path scope, unchanged sources/ledger/
protected owners, synchronized EN/JA, complete reviews/verification, one
commit, and clean replay. Migration exits separately with exact four
redirects/seven paths, ledger replay, all gates, one commit, and clean replay.

## Documentation-Prerequisite Evidence

Independent evidence-equivalence, schema/test-sufficiency, and bilingual/
boundary reviews all end **NO FINDINGS**. They independently reproduce the
four `76/70/32/29` preimages, 207-line total, source TSV hashes, eight index
records, prospective 13-row canonical hash, 864-line physical ledger hash,
and `+4/-203` migration delta. Every retained owner link and language-local
fragment resolves. The selected source sections and 850-line ledger remain
unchanged.

Checker and runner lint pass `15/15` each, checker and runner libraries pass
`530/530` and `600/600`, and runner metadata passes `137/137`. Formatting,
offline Cargo metadata, warnings-denied all-target/all-feature Clippy, the full
all-target/all-feature workspace suite including all three long frontend
benchmarks, generic recursive contract/link/fragment/ledger lint, and
`git diff --check` pass.

All five CLIs exit zero with 23 known warnings and zero errors each. Current
plan/requirements is `428/395`, pass/fail is `235/193`, and active
parse/declaration/type/proof is `101/7/205/1`; every stdout reproduces its
frozen hash. Protected counts and path hashes reproduce as specification 64,
`.miz` 343, expectation 435, checker production 30, runner production 90, and
Cargo 21. Zero protected diff preserves every frozen content hash. Trace,
coverage audit, immutable source TSV, and the unchanged 850-line ledger
reproduce their frozen hashes. Final independent read-only quality review ends
**NO FINDINGS**, passes all nine hard gates, applies no score cap, and assigns
**100/100** (`20/20/15/15/10/10/5/5`). Exact nine-path staging, cached
review, commit, and clean replay remain.

## Migration Evidence

The documentation prerequisite committed separately as
`158986616f91898d24c5c1ffc13c9446f38b2306`. Clean fresh replay then
reproduced all four frozen preimages, the source TSV hashes, the unchanged
850-line ledger, protected surfaces, trace, coverage audit, and stash
fingerprint.

The four selected sections are now four language-local redirects to
`258B4B.md#completion-evidence`. Source delta is exactly `+4/-203`: each
`76/70/32/29`-line section became one redirect plus its retained separator.
Every neighboring anchor, registered B4A redirect, and retained owner remains
in place. The ledger adds exactly 14 byte-sorted rows (batch 1, task 1,
redirects 4, indexes 8), is 864 lines, reproduces physical SHA-256
`876dbd36c52952d029257d3c16d0ae8f7cb2a9fac9f09fedafae6c4df3e026bf`,
and reproduces the frozen 13-row canonical hash. The source TSV, historical
contract, Task Index rows, protected surfaces, trace, and coverage audit are
unchanged.

Independent migration-equivalence, schema/test-sufficiency, and bilingual/
boundary reviews all end **NO FINDINGS**. Checker and runner lint pass `15/15`
each, checker and runner libraries pass `530/530` and `600/600`, and runner
metadata passes `137/137`. Formatting, offline Cargo metadata,
warnings-denied all-target/all-feature Clippy, the full all-target/all-feature
workspace suite including all three long frontend benchmarks, generic
recursive contract/link/fragment/ledger lint, and `git diff --check` pass.

All five CLIs exit zero with 23 known warnings and zero errors each and
reproduce the five frozen stdout hashes. Protected counts/path hashes
reproduce as specification 64, `.miz` 343, expectation 435, checker
production 30, runner production 90, and Cargo 21; zero protected diff
preserves every frozen content hash. Trace, coverage audit, immutable source
TSV, 13-row canonical payload, and 864-line ledger reproduce their frozen
hashes. Final independent read-only quality review ends **NO FINDINGS**,
passes all nine hard gates, applies no score cap, and assigns **100/100**
(`20/20/15/15/10/10/5/5`). Exact staging, commit, and clean replay remain.

## Handoff

Complete migration reviews and verification, stage exactly the seven frozen
paths, commit, then fresh-inventory the next checker duplication family.
Parent remains `xhigh`; bounded independent reviews use `high`.
