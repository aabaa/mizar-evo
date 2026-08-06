# Task DOC-249S-ACTIVE-EVIDENCE-COMPACT: Structure-Member Active-Evidence Compaction

> Canonical language: English. Japanese companion:
> [../ja/DOC-249S-ACTIVE-EVIDENCE-COMPACT.md](../ja/DOC-249S-ACTIVE-EVIDENCE-COMPACT.md).

This maintenance contract freezes one coherent checker/runner historical
completion-evidence family. It cannot change language behavior, test intent,
public API, diagnostics, traceability, or coverage.

## Identity And Status

| Field | Frozen value |
|---|---|
| Task | `DOC-249S-ACTIVE-EVIDENCE-COMPACT` |
| Status | Documentation prerequisite, all independent reviews, full verification, and final quality complete; exact staging and prerequisite commit remain. |
| Purpose | Centralize repeated Task-249S active implementation/no-runner evidence while retaining every durable/frozen owner and mixed section. |
| Owners | Migration policy, historical [249S](./249S.md#completion-evidence), [checker plan](../../mizar-checker/en/00.crate_plan.md#task-index), and [runner plan](../../mizar-test/en/00.crate_plan.md#task-index) |
| Consumers | Twenty-four checker/runner source paths, four Task Indexes, and the future schema-v1 ledger/lint |
| Sequence | `274917ab` -> `93d64c33` -> `1fe0b156` -> `f11a517e` |
| Readiness | Selection HEAD `331fdc055d9416225ccc6e2acb22d199c17cb8ee`, `origin/main...HEAD=0/1`, protected `stash@{0}=f65cf4a13752ec380710814a9ac6392ccb9d75d4`; revised selection review ended **NO FINDINGS**. |

## Authority And Classification

Authority is the user-approved checker-first compaction program,
[`AGENTS.md`](../../../../AGENTS.md), the
[migration policy](../../autonomous_crate_development.md#migration-policy),
the historical contract's retained canonical/test owners, and reviewed Git
history. Source behavior is not normative.

| Class | Decision |
|---|---|
| `design_drift` | Twenty-four active sections repeat Task-249S completion/no-runner evidence over twelve paired component paths; the historical contract becomes their shared completion-evidence owner. |
| `spec_gap` / `test_gap` | None for this structural task; authority, test intent, prior closure, and deferrals stay unchanged. |
| `source_drift` / `source_undocumented_behavior` | None introduced; production source is protected. |
| `test_expectation_drift` | None; specification, `.miz`, fixture, sidecar, expectation, and trace are protected. |
| `boundary_violation` | The initial 26-section proposal included both `resolved_typed_ast.md` active H2s. The JA H2 already contains a Task-269B completion redirect, so schema v1 cannot wholly migrate it to Task 249S; both EN/JA H2s are excluded for bilingual symmetry. The live coverage owner, all frozen/addendum/TODO/Task-263-and-later records, and every unlisted artifact are also excluded. |
| `repo_metadata_conflict` | Selection measured `0/1`. During final review `origin/main` advanced externally to the same `331fdc05`, producing current `0/0`; reflog records `update by push` at 2026-08-06 09:33:55 +0900. The parent did not push, the task-only target remains identifiable, and fetch/reset/push/stash mutation remain unauthorized. |

## Frozen Preimage And Anchors

[`DOC-249S-ACTIVE-EVIDENCE-COMPACT.sources.tsv`](../DOC-249S-ACTIVE-EVIDENCE-COMPACT.sources.tsv)
contains 24 byte-sorted data rows plus two comments and final LF. Data-row
SHA-256 is `981aca5b86370ef4513070334b8c7fc5710fb6e337fbbc56b2f1bee0bdef40d9`;
complete-file SHA-256 is
`53e8b44ee40078f613f633a355c25691460adb28e4919e6f9d2c9d32a7bdf434`.

The selection is 24 globally exhaustive H2 sections over 24 distinct paths,
256 physical lines: checker EN `7/87`, checker JA `7/82`, runner EN `5/44`,
and runner JA `5/43`. Every section is flat, with no nested heading, table,
fence, existing redirect, or inbound link to its removable fragment. Each raw
heading occurs once repository-wide. Retained anchors are:

| Source | EN preceding / following owner | JA preceding / following owner |
|---|---|---|
| checker `00.crate_plan.md` | `## Task 263 Fresh Preflight: Mandatory Checker Task 249S` / `## Task 263 Frozen Structure-Definition Contract` | `## Task 263 fresh preflight: mandatory checker Task 249S` / `## Task 263 frozen structure-definition contract` |
| checker `bilingual_sync_audit.md` | `## Task 249S Frozen-Contract Synchronization` / `## Task 263 Frozen-Contract Synchronization` | `## Task 249S frozen-contract synchronization` / `## Task 263 frozen-contract synchronization` |
| checker `module_boundary_audit.md` | `## Task 249S Frozen Module Boundary` / `## Task 263 Frozen Module Boundary` | `## Task 249S frozen module boundary` / `## Task 263 frozen module boundary` |
| checker `payload_family_decomposition.md` | `## Task 249S Structure-Member Type Lower Family` / `## Task 264 Property-Implementation Family` | `## Task 249S structure-member type lower family` / `## Task 264 property-implementation family` |
| checker `source_spec_audit.md` | `## Task 249S Frozen Future Public-Surface Audit` / `## Task 263 Frozen Source/API Audit` | `## Task 249S frozen future public-surface audit` / `## Task 263 frozen source/API audit` |
| checker `source_type.md` | `## Task 249S Frozen Standalone Structure-Member Type Intake` / `## Task 263 Test-Only Lower Replay Seam` | `## Task 249S standalone structure-member type intake frozen contract` / `## Task 263 test-only lower replay seam` |
| checker `typed_ast.md` | `## Task 249S Standalone Member-Type Ownership Addendum` / `## Task 263 Frozen Typed Ownership` | `## Task 249S standalone member-type ownership addendum` / `## Task 263 frozen Typed ownership` |
| runner `00.crate_plan.md` | `## Checker Task 249S Frozen No-Runner Prerequisite` / `## Checker Task 263 Frozen Consumer Plan` | `## Checker Task 249S frozen no-runner prerequisite` / `## Checker Task 263 frozen consumer plan` |
| runner `bilingual_sync_audit.md` | `## Checker Task 249S Synchronization Addendum` / `## Checker Task 263 Frozen Consumer Synchronization` | `## Checker Task 249S synchronization addendum` / `## Checker Task 263 frozen consumer synchronization` |
| runner `harness.md` | `## Checker Task 249S No-Consumer Harness Boundary` / `## Checker Task 263 Frozen Harness Route` | `## Checker Task 249S no-consumer harness boundary` / `## Checker Task 263 frozen harness route` |
| runner `module_boundary_audit.md` | `## Checker Task 249S No-Runner Boundary` / `## Checker Task 263 Frozen Runner Boundary` | `## Checker Task 249S no-runner boundary` / `## Checker Task 263 frozen runner boundary` |
| runner `traceability.md` | `## Checker Task 249S Frozen Traceability No-Op` / `## Checker Task 263 Frozen Trace Intent` | `## Checker Task 249S frozen traceability no-op` / `## Checker Task 263 frozen trace intent` |

The excluded final pair and live coverage owner remain independent evidence
owners. No historical `249S` contract, batch/source-inventory identity, Task
Index row, ledger task, or redirect existed in the selected preimage.

## Frozen Protected Baseline

Expected prerequisite and migration delta is zero for every row:

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
The 788-line ledger remains
`1702d79a198685ce8603f65dbdd2947f7d2c78e7b9ea3e76a150caac29a48da7`,
with expanded inventory
`bb38229607a2a3eaa81e7b8d4ab8218c8ce42f0f86de91dd7471b3f205ed0b66`.
Expected CLI stdout hashes are plan
`700f4bf503783742cefd8004fa095675b7476d46e9a3a6dd439916d237eb6718`,
parse `a8a7aa639d2ebc65eddc923c7e9369ea5637d50e935f808600f446da1bfbda56`,
declaration `71e83ba0b20d4015e07b3bd2c0c4db2837b6151d1251812caed7954530d53c74`,
type `4b2c7bd5ec3cc56e5672fb351126126230ec84fba9bd2bd9049a516d378fab7f`,
and proof `ccf3d2d4d0a3755e00989d97af369a7c560302f76798d0a185d57ec3891e8450`.

## Scope, Verification, And Exit

The documentation prerequisite changes exactly nine paths: this pair, the
historical pair, the source TSV, and four plans. Plans add one historical-task
and one batch row in both languages/components, eight index records total.
Selected preimages, the ledger, protected artifacts, counts, hashes, statuses,
public behavior, and `spec_coverage_audit.md` remain unchanged. The audit has
no impact because ownership, trace status, coverage credit, and deferral state
do not change.

After a separate prerequisite commit and fresh replay, migration may replace
only the 24 declared sections with language-local redirects to
`249S.md#completion-evidence`. It changes exactly those 24 sources, this pair,
and `legacy_compactions.tsv`: 27 paths. The 256 lines become 48 redirect-plus-
separator lines, a reduction of 208; expected source diff is 24 additions and
232 deletions. Ledger impact is 34 lines, `788 -> 822`: one batch, one task, 24
redirects over 24 distinct paths, eight index records, and one expanded-
inventory hash. The canonical 33-row task/redirect/index payload excludes the
batch row and has frozen SHA-256
`71017a5197eb6bac76a8d6e079ee17f24301db20a19ca84c00df120e24155acf`.
The source TSV, historical pair, and indexes then become immutable.

Both commits require independent contract/equivalence, schema/test-sufficiency,
boundary, source/document/EN-JA, and final-quality reviews as applicable,
ending **NO FINDINGS**. Verification includes preimage/anchor replay, generic
schema/link/fragment and full lint, checker/runner/metadata tests, formatting,
Cargo metadata, warnings-denied Clippy, workspace tests, five CLIs, protected
counts/hashes, `git diff --check`, exact staging, all nine hard gates, and an
uncapped score `>=90/100`. No push or stash mutation is authorized.

## Documentation-Prerequisite Evidence

The initial selection included 26 active H2s. Review found one High
`boundary_violation`: the JA final active H2 already contained a Task-269B
completion redirect. The parent excluded both final H2s. Revised selection
review ended **NO FINDINGS**, freezing exact `7/87 + 7/82 + 5/44 + 5/43 =
24/256` scope, 24 globally unique headings, stable retained anchors, and the
live coverage owner outside the migration.

Independent contract/equivalence and source-documentation/EN-JA reviews ended
**NO FINDINGS**. Schema/test-sufficiency review found one Medium
`design_drift`: the first draft froze the future inventory cardinality but not
its hash. The parent independently generated the schema-v1 payload, froze the
33-row `71017a...` hash above in EN/JA, and the same reviewer re-reviewed it as
**NO FINDINGS**. Reviews replayed chronology, authority, API/profile,
validation/precedence, Typed/final and unchanged-obligation ownership, exact
tests and historical hashes, consumer/no-runner boundary, deferrals, every
link/fragment, 24 preimages/anchors, exclusions, nine-path scope, future delta,
ledger arithmetic, and audit no-impact.

Full prerequisite-state verification passes checker/runner lint `15/15` each,
checker/runner libraries `530/530` and `600/600`, and metadata `137/137`.
`cargo fmt --all --check`, offline Cargo metadata, warnings-denied all-target/
all-feature Clippy, and the full all-target/all-feature workspace suite with
frontend benchmarks pass. All five CLIs exit zero with 23 warnings and zero
errors each and reproduce all five frozen stdout hashes.

Protected counts and path hashes reproduce exactly as specification 64, `.miz`
343, expectation 435, checker production 30, runner production 90, and Cargo
21; zero protected diff preserves every frozen content hash. Trace, coverage
audit, the 788-line ledger and its physical hash, the current expanded-
inventory hash, both source-TSV hashes, all 24 preimages, the future 33-row
hash, exact nine-path scope, and `git diff --check` reproduce. Exact staging,
commit, and fresh inventory remain. The external
`origin/main` advance from selection `0/1` to verification `0/0` is the
report-only `repo_metadata_conflict` recorded above; worktree scope and the
protected stash did not change.

Final independent read-only quality review ends **NO FINDINGS**. All nine hard
gates PASS, no score cap applies, and the valid score is `100/100`
(`20/20/15/15/10/10/5/5`). Independent replay confirms exact scope, all
preimages, source/inventory hashes, exclusions, protected surfaces, EN/JA
ownership, review closure, verification health, and the external-origin
classification. Residual risk is limited to exact staging, commit, and fresh
inventory before the separate migration.

## Handoff

Exact-stage and commit this prerequisite, fresh-replay the 24 preimages and
anchors, then perform the separately frozen 27-path migration. The parent
remains `xhigh`; bounded independent reviews use `high`.
