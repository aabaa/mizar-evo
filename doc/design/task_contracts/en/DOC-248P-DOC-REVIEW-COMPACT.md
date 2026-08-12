# Task DOC-248P-DOC-REVIEW-COMPACT: Property-Context Review-Evidence Compaction

> Canonical language: English. Japanese companion:
> [../ja/DOC-248P-DOC-REVIEW-COMPACT.md](../ja/DOC-248P-DOC-REVIEW-COMPACT.md).

This maintenance contract freezes one checker-only historical review family.
It cannot change language behavior, test intent, API, diagnostics,
traceability, or coverage.

## Identity And Status

| Field | Frozen value |
|---|---|
| Task | `DOC-248P-DOC-REVIEW-COMPACT` |
| Status | Complete. The migration is registered in the schema-2 ledger; task-local completion evidence below preserves the committed migration and clean replay. |
| Purpose | Centralize repeated Task-248P documentation-prerequisite and frozen-review evidence while retaining every durable implementation and runner owner. |
| Owners | Migration policy, historical [248P](./248P.md#completion-evidence), [checker plan](../../mizar-checker/en/00.crate_plan.md#task-index), and [runner plan](../../mizar-test/en/00.crate_plan.md#task-index) |
| Consumers | Eight checker source paths, four Task Indexes, and the post-migration schema-v1 ledger/lint |
| Sequence | `db8c39e3` -> `1e3fa789` -> `1637380d` -> `4c3f74b0` |
| Documentation prerequisite | `b483bc298cc459e2b294bd07726ca6721d9fe298` |
| Readiness | Clean selection HEAD `d94dfd6330c1dd067be8b26c814ac95e077b2639`, `origin/main...HEAD=0/14`, protected `stash@{0}=f65cf4a13752ec380710814a9ac6392ccb9d75d4`; dependency-ready. |

## Authority And Classification

Authority is the user-approved checker-first compaction program,
[`AGENTS.md`](../../../../AGENTS.md), the
[migration policy](../../autonomous_crate_development.md#migration-policy),
the historical contract's retained canonical/test owners, and reviewed
history. Source behavior is not normative.

| Class | Decision |
|---|---|
| `design_drift` | Eight checker sections repeat the `1e3fa789` prerequisite review/freeze checkpoint; the historical contract becomes their shared evidence owner. |
| `spec_gap` / `test_gap` | None for this structural task; Task-248P's historical classifications and closures remain unchanged. |
| `source_drift` / `source_undocumented_behavior` | None introduced; production source is protected. |
| `test_expectation_drift` | None; specification, `.miz`, expectations, sidecars, and trace are protected. |
| `boundary_violation` | Avoided by retaining the frozen plan contract, implementation and verification results, checker/runner TODOs, every runner document, active module/API owners, and all unlisted sections. Task 258B4B is separately excluded because its candidate H2 mixes nested durable content. |
| `repo_metadata_conflict` | Current `0/14` is report-only and not repaired; fetch, reset, push, and stash mutation are unauthorized. |

## Frozen Preimage And Anchors

[`DOC-248P-DOC-REVIEW-COMPACT.sources.tsv`](../DOC-248P-DOC-REVIEW-COMPACT.sources.tsv)
contains eight byte-sorted rows plus two comments and final LF. Data-row
SHA-256 is `cd19b044410fa454125c80ac1ea711dfbd0bb8eb0e6e05cb9c20a81c94510c84`;
complete-file SHA-256 is
`ba3029c35715c3450c2d3bd863e4904ef7e940d568d3321f5644b5faf1e70285`.

The selection is eight unique H2 sections over eight checker paths, 113
physical lines: EN `4/60`, JA `4/53`. None contains a nested heading, table, or
fence. Retained EN preceding/following owners are:

| Source | Retained anchors |
|---|---|
| `00.crate_plan.md` | `## Task 248P Frozen Property Binding-Context Prerequisite` / `## Task 248P Implementation Result` |
| `bilingual_sync_audit.md` | `## Task 264R Implementation Synchronization` / `## Task 248P Implementation Synchronization` |
| `module_boundary_audit.md` | `## Task 264R Implemented No-Checker-Source Boundary` / `## Task 248P Implemented One-File Checker Boundary` |
| `source_spec_audit.md` | `## Task 264R Implemented Source/Specification Status` / `## Task 248P Implemented Source/Specification Status` |

JA companions have matching levels and language-local equivalent anchors. All
eight headings are unique; no Task-248P contract, index, or ledger identity
exists in the preimage.

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
coverage audit remains `2aa808aa033c04fa46ea76f65cfadcf0d8f0e1e53fba6d70edab223a7481685f`;
the 660-line ledger remains
`f3fdbf5111f4c17cf19088f97844dfa4eeb8ac5b2051866e1c86f99b44efc301`.
Expected CLI stdout hashes are plan
`700f4bf503783742cefd8004fa095675b7476d46e9a3a6dd439916d237eb6718`,
parse `a8a7aa639d2ebc65eddc923c7e9369ea5637d50e935f808600f446da1bfbda56`,
declaration `71e83ba0b20d4015e07b3bd2c0c4db2837b6151d1251812caed7954530d53c74`,
type `4b2c7bd5ec3cc56e5672fb351126126230ec84fba9bd2bd9049a516d378fab7f`,
and proof `ccf3d2d4d0a3755e00989d97af369a7c560302f76798d0a185d57ec3891e8450`.

## Scope, Verification, And Exit

The prerequisite changes exactly nine paths: this pair, historical pair,
source TSV, and four plans. The plans gain historical-task and batch rows,
eight index rows total. Selected preimages, ledger, protected artifacts,
counts/hashes/statuses, public behavior, and `spec_coverage_audit.md` remain
unchanged; the audit has no impact because ownership, trace status, and credit
do not change.

After a separate prerequisite commit and fresh replay, migration may replace
only the eight declared sections with language-local redirects to
`248P.md#completion-evidence`. It changes exactly those eight sources, this
pair, and `legacy_compactions.tsv`: eleven paths. The 113 lines become 16
redirect-plus-separator lines, a reduction of 97. Ledger impact is one batch,
one task, eight redirects over eight distinct paths, eight index records, and
one expanded-inventory hash. Source TSV, historical pair, and indexes then
become immutable.

Both commits require independent contract/equivalence, test-sufficiency,
boundary, source/document/EN-JA, and final-quality reviews as applicable,
ending **NO FINDINGS**. Verification includes preimage/anchor replay, generic
schema/link/fragment and full lint, checker/runner/metadata tests, formatting,
Cargo metadata, warnings-denied Clippy, workspace tests, five CLIs, protected
counts/hashes, `git diff --check`, exact staging, all nine hard gates, and an
uncapped score `>=90/100`. No push or stash mutation is authorized.

## Documentation-Prerequisite Evidence

Independent contract/equivalence, test-sufficiency/schema, and
source-documentation/EN-JA reviews ended **NO FINDINGS**. They replayed all
`8/113` preimages, both TSV hashes, chronology, unique API/invariant and
classification claims, ownership/exclusions, exact nine-path scope and eight
index rows, audit no-impact, protected no-ops, language-local links, and the
future schema-v1 `1/1/8/8` ledger plan. No Rust, schema, traceability, coverage,
or additional documentation change is warranted.

Checker and runner lint passed `15/15` each; checker/runner libraries passed
`530/530` and `600/600`; runner metadata passed `137/137`. `cargo fmt --all
--check`, offline Cargo metadata, warnings-denied all-target/all-feature
Clippy, the full offline all-target/all-feature workspace suite, and
`git diff --check` passed. All five CLIs exited zero with 23 stderr lines and
zero errors each and reproduced every frozen stdout hash. Specification,
`.miz`, expectation, checker/runner production, Cargo, trace, coverage audit,
and the 660-line ledger have zero delta; the immutable source TSV retains its
frozen full-file hash. Final read-only quality ended **NO FINDINGS**; all nine
hard gates PASS, no score cap applies, and the valid score is `100/100`
(`20/20/15/15/10/10/5/5`). Exact staging and the dedicated prerequisite commit
remain.

## Migration Evidence

The prerequisite committed as `b483bc298cc459e2b294bd07726ca6721d9fe298`.
Fresh post-commit inventory was clean at `origin/main...HEAD=0/15`; protected
`stash@{0}=f65cf4a13752ec380710814a9ac6392ccb9d75d4` was unchanged, and all eight
frozen preimages replayed at 113 lines before editing.

The mechanical migration changes exactly eight declared checker sources,
this EN/JA pair, and `legacy_compactions.tsv`: eleven paths. It replaces only
the eight complete sections with language-local redirects. Their 113 physical
lines become 16 redirect-plus-separator lines, a reduction of 97; the source
diff is eight additions and 105 deletions. Every TODO, runner, frozen-plan,
implementation, implementation-verification, active API, audit, trace,
coverage, and unlisted owner remains.

The ledger now has 678 physical lines. The batch adds exactly one task, eight
redirects over eight distinct source paths, and eight index records. Its
expanded-inventory SHA-256 is
`d3549958ec578a603d18a15d62175db616cd60d312e733e5bd3574ad9a534a21`;
its complete physical SHA-256 is
`a26fe1fedd9f6b634de66daff85682d3ef63871242df77953eb4b881ec2a1d3a`.
The immutable source TSV remains
`ba3029c35715c3450c2d3bd863e4904ef7e940d568d3321f5644b5faf1e70285`.
Focused generic-ledger/link/fragment lint and `git diff --check` pass.

Independent equivalence/boundary, test-sufficiency/schema, and
source-documentation/EN-JA reviews ended **NO FINDINGS** after one Low
`design_drift` correction to the stale handoff. They replayed every preimage,
postimage, anchor, redirect, unique claim, retained owner, ledger relation and
hash, protected scope, and audit no-impact. Existing generic schema-v1 lint is
sufficient; no Rust, schema, fixture, expectation, test, trace, coverage, or
additional documentation change is warranted.

Checker and runner lint passed `15/15` each; checker/runner libraries passed
`530/530` and `600/600`; runner metadata passed `137/137`. Formatting, offline
Cargo metadata, warnings-denied all-target/all-feature Clippy, the full offline
all-target/all-feature workspace suite, and `git diff --check` passed. Five
CLIs exited zero with 23 stderr lines and zero errors each and reproduced all
frozen stdout hashes. Protected specification, `.miz`, expectation,
checker/runner production, Cargo, trace, coverage-audit, and immutable source
TSV surfaces remain unchanged. Final read-only quality ended **NO FINDINGS**;
all nine hard gates PASS, no score cap applies, and the valid score is
`100/100` (`20/20/15/15/10/10/5/5`). Exact staging and the task-only commit
remain.

## Handoff

Exact staging and the task-only commit remain. After clean post-commit
inventory, select the next schema-v1-safe checker duplication family; `249PI`
is the current lower-risk candidate, while `264`, `269A`, and `259` require
stricter retained-owner review. Parent reasoning remains `xhigh`; independent
reviews use `high`, deterministic inventory may use `medium`.
