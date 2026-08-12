# Task DOC-249M-ACTIVE-EVIDENCE-COMPACT: Mode-RHS Active-Evidence Compaction

> Canonical language: English. Japanese companion:
> [../ja/DOC-249M-ACTIVE-EVIDENCE-COMPACT.md](../ja/DOC-249M-ACTIVE-EVIDENCE-COMPACT.md).

This maintenance contract freezes one checker-only historical completion-
evidence family. It cannot change language behavior, test intent, public API,
diagnostics, traceability, or coverage.

## Identity And Status

| Field | Frozen value |
|---|---|
| Task | `DOC-249M-ACTIVE-EVIDENCE-COMPACT` |
| Status | Complete. The migration is registered in the schema-2 ledger; task-local completion evidence below preserves the committed migration and clean replay. |
| Purpose | Centralize repeated Task-249M active implementation evidence while retaining every durable/frozen owner and all excluded mixed evidence. |
| Owners | Migration policy, historical [249M](./249M.md#completion-evidence), [checker plan](../../mizar-checker/en/00.crate_plan.md#task-index), and [runner plan](../../mizar-test/en/00.crate_plan.md#task-index) |
| Consumers | Sixteen checker source paths, four Task Indexes, and the future schema-v1 ledger/lint |
| Sequence | `8c3fa20a` -> `b1b41012` -> `2baf83d3` -> `1fb192e3` |
| Readiness | Clean selection HEAD `1ad52ed39cfa98d9a9b08f639e2d75f123de80cf`, `origin/main...HEAD=0/24`, protected `stash@{0}=f65cf4a13752ec380710814a9ac6392ccb9d75d4`; revised selection review ended **NO FINDINGS**. |

## Authority And Classification

Authority is the user-approved checker-first compaction program,
[`AGENTS.md`](../../../../AGENTS.md), the
[migration policy](../../autonomous_crate_development.md#migration-policy),
the historical contract's retained canonical/test owners, and reviewed Git
history. Source behavior is not normative.

| Class | Decision |
|---|---|
| `design_drift` | Sixteen checker sections repeat Task-249M active completion across eight paired component paths; the historical contract becomes their shared completion-evidence owner. |
| `spec_gap` / `test_gap` | None for this structural task; authority, test intent, prior closure, and deferrals stay unchanged. |
| `source_drift` / `source_undocumented_behavior` | None introduced; production source is protected. |
| `test_expectation_drift` | None; specification, `.miz`, fixture, sidecar, expectation, and trace are protected. |
| `boundary_violation` | The first 18-section proposal included both checker bilingual-audit H2s. The JA H2 contains five older Task-258B4A final-review lines, so schema v1 cannot move it wholly into Task 249M. Both EN/JA bilingual H2s are excluded. Frozen/addendum/selection records, runner/TODO/coverage owners, Task 262+, and every unlisted artifact are also excluded. |
| `repo_metadata_conflict` | Current `0/24` is report-only. Fetch, reset, push, and stash mutation are unauthorized; the task-only commit target is identifiable. |

## Frozen Preimage And Anchors

[`DOC-249M-ACTIVE-EVIDENCE-COMPACT.sources.tsv`](../DOC-249M-ACTIVE-EVIDENCE-COMPACT.sources.tsv)
contains sixteen byte-sorted data rows plus two comments and final LF. Data-row
SHA-256 is `ed26edcf9c657c747383b8c7aaf0f175fefb826ecc24637e92f6d9d2e0ccdfe9`;
complete-file SHA-256 is
`4ffed4391aced54ebfb2ab13ed493f594359c858f9b543be645516be2669b658`.

The selection is sixteen globally exhaustive H2 sections over sixteen distinct
checker paths, 159 physical lines: EN `8/81`, JA `8/78`. Every selected section
is flat, with no nested heading, table, fence, existing redirect, or inbound
link to its removable fragment. The fourteen raw heading strings are globally
exhausted; the EN and JA active-implementation-result headings occur twice in
their language and all four occurrences are selected. Retained anchors are:

| Source | EN preceding / following owner | JA preceding / following owner |
|---|---|---|
| `00.crate_plan.md` | `## Task 262 Upper-Contract Commit And Task 249M Selection` / `## Task 262 Active Implementation Result` | `## Task 262 upper-contract commit と Task 249M selection` / `## Task 262 active implementation result` |
| `module_boundary_audit.md` | `## Task 249M Frozen Boundary` / `## Task 262 Active Module Boundary` | `## Task 249M frozen boundary` / `## Task 262 active module boundary` |
| `payload_family_decomposition.md` | `## Task 249M Mode-RHS Lower Family` / `## Task 249S Structure-Member Type Lower Family` | `## Task 249M mode-RHS lower family` / `## Task 249S structure-member type lower family` |
| `resolved_typed_ast.md` | `## Task 249M Mode-RHS Clone Addendum` / `## Task 262 Active Final Mode-Definition Ownership` | `## Task 249M mode-RHS clone addendum` / `## Task 262 active final mode-definition ownership` |
| `source_mode_definition.md` | `## Task 249M Lower-Contract Link` / `## Task 262 Active Implementation Result` | `## Task 249M lower-contract link` / `## Task 262 active implementation result` |
| `source_spec_audit.md` | `## Task 249M Frozen Future Public-Surface Audit` / `## Task 262 Active Source Audit` | `## Task 249M frozen future public-surface audit` / `## Task 262 active source audit` |
| `source_type.md` | `## Task 249M Frozen Standalone Mode-RHS Extension` / `## Task 249S Frozen Standalone Structure-Member Type Intake` | `## Task 249M frozen standalone mode-RHS extension` / `## Task 249S standalone structure-member type intake frozen contract` |
| `typed_ast.md` | `## Task 249M Mode-RHS Ownership Addendum` / `## Task 262 Active Mode-Definition Transaction` | `## Task 249M mode-RHS ownership addendum` / `## Task 262 active mode-definition transaction` |

No historical `249M` contract, batch/source-inventory identity, Task Index row,
ledger task, or redirect existed in the selected preimage. The excluded checker
bilingual records and retained runner sections remain their own evidence owners.

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
`2aa808aa033c04fa46ea76f65cfadcf0d8f0e1e53fba6d70edab223a7481685f`;
the 762-line ledger remains
`512633c4d6b7f3f8c460a5e5ccd2a5b9717d2826626e08689b4a3205a8dadb11`,
with expanded inventory
`3e081810f038edf8c3a75f9a222e02dcb8ea07d42b957d911df04ce8ad33b96f`.
Expected CLI stdout hashes are plan
`700f4bf503783742cefd8004fa095675b7476d46e9a3a6dd439916d237eb6718`,
parse `a8a7aa639d2ebc65eddc923c7e9369ea5637d50e935f808600f446da1bfbda56`,
declaration `71e83ba0b20d4015e07b3bd2c0c4db2837b6151d1251812caed7954530d53c74`,
type `4b2c7bd5ec3cc56e5672fb351126126230ec84fba9bd2bd9049a516d378fab7f`,
and proof `ccf3d2d4d0a3755e00989d97af369a7c560302f76798d0a185d57ec3891e8450`.

## Scope, Verification, And Exit

The prerequisite changes exactly nine paths: this pair, the historical pair,
the source TSV, and four plans. Plans add historical-task and batch rows, eight
index rows total. Selected preimages, ledger, protected artifacts, counts,
hashes, statuses, public behavior, and `spec_coverage_audit.md` remain unchanged;
the audit has no impact because ownership, trace status, coverage credit, and
deferral state do not change.

After a separate prerequisite commit and fresh replay, migration may replace
only the sixteen declared sections with language-local redirects to
`249M.md#completion-evidence`. It changes exactly those sixteen sources, this
pair, and `legacy_compactions.tsv`: nineteen paths. The 159 lines become 32
redirect-plus-separator lines, a reduction of 127; expected source diff is 16
additions and 143 deletions. Ledger impact is one batch, one task, sixteen
redirects over sixteen distinct paths, eight index records, and one expanded-
inventory hash. Source TSV, historical pair, and indexes then become immutable.

Both commits require independent contract/equivalence, test-sufficiency,
boundary, source/document/EN-JA, and final-quality reviews as applicable,
ending **NO FINDINGS**. Verification includes preimage/anchor replay, generic
schema/link/fragment and full lint, checker/runner/metadata tests, formatting,
Cargo metadata, warnings-denied Clippy, workspace tests, five CLIs, protected
counts/hashes, `git diff --check`, exact staging, all nine hard gates, and an
uncapped score `>=90/100`. No push or stash mutation is authorized.

## Documentation-Prerequisite Evidence

The first selection review found one High `boundary_violation`: the proposed JA
bilingual-audit H2 mixed five Task-258B4A final-review lines into Task-249M
evidence. The parent excluded both EN/JA bilingual H2s. Revised selection
re-review ended **NO FINDINGS**, freezing the exact `8/81 + 8/78 = 16/159`
scope, all fourteen globally exhausted headings, and stable retained anchors.

Independent contract/equivalence, schema/test-sufficiency, and source-
documentation/EN-JA reviews all end **NO FINDINGS**. They replay every section
hash and line count, both TSV hashes, chronology, the eight index rows, owner
links/fragments, exact API/profile/test/consumer/deferral claims, audit no-
impact, nine-path prerequisite scope, and future `159 -> 32`, `+16/-143`,
`1/1/16/16/8` migration plan. An initial recursive-link lint exposed missing
owning-plan links and exact JA canonical markers; those local authorship defects
were corrected before the independent reviews, after which recursive lint
passed.

Full prerequisite-state verification passes checker/runner lint `15/15` each,
checker/runner libraries `530/530` and `600/600`, and metadata `137/137`.
`cargo fmt --all --check`, offline Cargo metadata, warnings-denied all-target/
all-feature Clippy, and the full all-target/all-feature workspace suite including
frontend benchmarks pass. All five CLIs exit zero with 23 warnings and zero
errors each and reproduce every frozen stdout hash.

The six protected counts and path hashes reproduce exactly; zero protected diff
retains all frozen content hashes. Trace, coverage audit, the 762-line ledger,
its expanded-inventory hash, both source-TSV hashes, all sixteen preimages,
exact nine-path scope, and `git diff --check` also reproduce. Final independent
read-only quality ends **NO FINDINGS**: all nine hard gates PASS, no score cap
applies, and the valid score is `100/100` (`20/20/15/15/10/10/5/5`). Residual
risk is limited to exact staging, commit, fresh inventory, and the separately
frozen migration.

## Migration Evidence

The prerequisite was committed as
`3d3f98767aa3818186f75e429dad468d97003ba7`. Its immediate fresh inventory was
clean at `origin/main...HEAD=0/25`, with the protected stash unchanged. All
sixteen source-TSV preimages and anchors replayed before migration.

Only the declared sixteen whole H2 sections have been replaced by sixteen
language-local completion redirects. Their physical shape is exactly
`159 -> 32`, a net reduction of 127 lines. Patience and histogram diff both
reproduce the frozen source delta `+16/-143`. The default Myers presentation
reports `+18/-145` because the two EN/JA `source_type.md` separator lines are
paired as churn; it has the same net delta and the checked postimage still has
one redirect plus one separator per source. All fourteen forbidden raw heading
strings are absent, and the redirect count is sixteen.

The byte-sorted schema-v1 ledger now has 788 physical lines and physical
SHA-256 `1702d79a198685ce8603f65dbdd2947f7d2c78e7b9ea3e76a150caac29a48da7`.
Its generic lint accepts expanded-inventory SHA-256
`bb38229607a2a3eaa81e7b8d4ab8218c8ce42f0f86de91dd7471b3f205ed0b66`
and exact cardinality `1/1/16/16/8` for batch/task/redirect/distinct-path/index.
The migration diff is the exact nineteen paths: sixteen sources, this paired
batch contract, and the ledger. The immutable source TSV, historical contract,
four Task Indexes, protected artifacts, trace, and coverage audit remain
unchanged; the frozen no-impact decision therefore still applies.

Independent equivalence/boundary, schema/test-sufficiency, and source-
documentation/EN-JA reviews all end **NO FINDINGS**. Recursive and full runner
lint `15/15`, checker lint `15/15`, checker/runner libraries `530/530` and
`600/600`, metadata `137/137`, `cargo fmt --all --check`, offline Cargo
metadata, warnings-denied all-target/all-feature Clippy, and the full
all-target/all-feature workspace suite with frontend benchmarks all pass. The
five CLIs each exit zero with 23 warnings and zero errors and reproduce the
five frozen stdout hashes.

Protected counts and path hashes reproduce as specification 64, `.miz` 343,
expectation 435, checker production 30, runner production 90, and Cargo 21;
zero protected diff preserves each frozen content hash. Trace, coverage audit,
source-TSV, ledger, forbidden-heading/redirect, exact-scope, and
`git diff --check` checks pass. Staging evidence remains to be recorded.

Final independent read-only quality review ends **NO FINDINGS**. All nine hard
gates PASS, no score cap applies, and the valid score is `100/100`
(`20/20/15/15/10/10/5/5`). Its independent replay confirms the exact scope,
all preimages and postimages, ledger inventory, protected surfaces, paired
ownership, and lint/format/metadata evidence. Residual risk is limited to exact
staging, commit, and post-commit inventory.

## Handoff

Exact-stage the nineteen migration paths, commit, and fresh-inventory the next
checker duplication family. The parent remains `xhigh`.
