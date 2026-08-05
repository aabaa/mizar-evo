# Task DOC-260-DOC-REVIEW-COMPACT: Functor Documentation-Review Compaction

> Canonical language: English. Japanese companion:
> [../ja/DOC-260-DOC-REVIEW-COMPACT.md](../ja/DOC-260-DOC-REVIEW-COMPACT.md).

This maintenance contract freezes one checker-only historical documentation-
prerequisite review family. It cannot change language behavior, test intent,
API, diagnostics, traceability, or coverage.

## Identity And Status

| Field | Frozen value |
|---|---|
| Task | `DOC-260-DOC-REVIEW-COMPACT` |
| Status | Documentation prerequisite committed; exact migration, all reviews, and full verification complete. Exact staging and commit remain. |
| Purpose | Centralize repeated Task-260 documentation-prerequisite verification, bilingual synchronization, and completed-checklist evidence while retaining implementation and durable component owners. |
| Owners | Migration policy, historical [260](./260.md#completion-evidence), [checker plan](../../mizar-checker/en/00.crate_plan.md#task-index), and [runner plan](../../mizar-test/en/00.crate_plan.md#task-index) |
| Consumers | Six checker source paths, four Task Indexes, and the post-migration schema-v1 ledger/lint |
| Sequence | `b61be7e5` -> `b587038f` -> `b292b800` -> `c233bfdf` -> `c83e424a` |
| Readiness | Clean selection HEAD `a9d5f40650d2ed694ba9304e2448fbd95e272406`, `origin/main...HEAD=0/20`, protected `stash@{0}=f65cf4a13752ec380710814a9ac6392ccb9d75d4`; dependency-ready after selection re-review. |

## Authority And Classification

Authority is the user-approved checker-first compaction program,
[`AGENTS.md`](../../../../AGENTS.md), the
[migration policy](../../autonomous_crate_development.md#migration-policy),
the historical contract's retained canonical/test owners, and reviewed
history. Source behavior is not normative.

| Class | Decision |
|---|---|
| `design_drift` | Six checker sections repeat the `b587038f` documentation-prerequisite freeze, synchronization, review, and checklist evidence; the historical contract becomes their shared evidence owner. |
| `spec_gap` / `test_gap` | None for this structural task; Task-260 authority, test intent, findings, and closure remain unchanged. |
| `source_drift` / `source_undocumented_behavior` | None introduced; production source is protected. |
| `test_expectation_drift` | None; specification, `.miz`, expectations, sidecars, and trace are protected. |
| `boundary_violation` | The initial eight-section proposal was rejected because schema v1 forbids two redirects for task `260` in one plan path. Both implementation-verification sections and every durable owner remain unselected. No new historical identity or schema extension is invented. |
| `repo_metadata_conflict` | Current `0/20` and unrelated legacy Task Index collisions observed while rejecting other candidate families are report-only and not repaired. Task `260` itself has no contract/index/ledger collision. Fetch, reset, push, and stash mutation are unauthorized. |

## Frozen Preimage And Anchors

[`DOC-260-DOC-REVIEW-COMPACT.sources.tsv`](../DOC-260-DOC-REVIEW-COMPACT.sources.tsv)
contains six byte-sorted rows plus two comments and final LF. Data-row SHA-256
is `703cf1b0ed8b6cb281f76f071d8236c4d5b90027993905e028ef44ce6006e4c0`;
complete-file SHA-256 is
`5f6e31b89902a747fa0ba141cef966e5aff6cb0f0f2b79b56e447584fce6289f`.

The selection is six globally exhaustive source-qualified H2 sections over
six checker paths, 107 physical lines: EN `3/55`, JA `3/52`. Each of the three
raw headings occurs exactly twice globally, once in each selected language
path. No selected section contains a nested heading, table, fence, or existing
ledger redirect. Retained EN preceding/following owners are:

| Source | Retained anchors |
|---|---|
| `00.crate_plan.md` | `## Task 260 Frozen Functor-Definition Producer Contract` / `## Task 260 Lower-Stage Preflight And Task 249R Selection` |
| `bilingual_sync_audit.md` | `## Task 248 Two-Parameter Profile Synchronization` / `## Task 249R Synchronization Addendum` |
| `todo.md` | `## Checker Task 259 Active Implementation` / `## Checker Task 249R Definition-Return Documentation Prerequisite` |

JA companions have matching levels and language-local equivalent anchors. No
Task-260 or batch contract, index, ledger task, redirect, or batch identity
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
the 714-line ledger remains
`0d2cb3968d79e93e1898838e31cc51b6d455f0941301e5347c6534880211e50f`.
Expected CLI stdout hashes are plan
`700f4bf503783742cefd8004fa095675b7476d46e9a3a6dd439916d237eb6718`,
parse `a8a7aa639d2ebc65eddc923c7e9369ea5637d50e935f808600f446da1bfbda56`,
declaration `71e83ba0b20d4015e07b3bd2c0c4db2837b6151d1251812caed7954530d53c74`,
type `4b2c7bd5ec3cc56e5672fb351126126230ec84fba9bd2bd9049a516d378fab7f`,
and proof `ccf3d2d4d0a3755e00989d97af369a7c560302f76798d0a185d57ec3891e8450`.

## Scope, Verification, And Exit

The prerequisite changes exactly nine paths: this pair, the historical pair,
the source TSV, and four plans. Plans add historical-task and batch rows, eight
index rows total. Selected preimages, ledger, protected artifacts,
counts/hashes/statuses, public behavior, and `spec_coverage_audit.md` remain
unchanged; the audit has no impact because ownership, trace status, and credit
do not change.

After a separate prerequisite commit and fresh replay, migration may replace
only the six declared sections with language-local redirects to
`260.md#completion-evidence`. It changes exactly
those six sources, this pair, and `legacy_compactions.tsv`: nine paths. The 107
lines become 12 redirect-plus-separator lines, a reduction of 95; expected
source diff is six additions and 101 deletions. Ledger impact is one batch, one
task, six redirects over six distinct paths, eight index records, and one
expanded-inventory hash. Source TSV, historical pair, and indexes then become
immutable.

Both commits require independent contract/equivalence, test-sufficiency,
boundary, source/document/EN-JA, and final-quality reviews as applicable,
ending **NO FINDINGS**. Verification includes preimage/anchor replay, generic
schema/link/fragment and full lint, checker/runner/metadata tests, formatting,
Cargo metadata, warnings-denied Clippy, workspace tests, five CLIs, protected
counts/hashes, `git diff --check`, exact staging, all nine hard gates, and an
uncapped score `>=90/100`. No push or stash mutation is authorized.

## Documentation-Prerequisite Evidence

Initial selection review found one blocking `boundary_violation`: the first
eight-section proposal put two task-`260` redirects in each plan path, which
schema v1 rejects. The parent narrowed the family to the exact six
documentation-prerequisite sections above and retained both implementation
sections. Selection re-review ended **NO FINDINGS**.

Contract/equivalence review then found one Medium overstatement that `Equals`
created no correctness obligation; EN/JA now say only that it appends no
existence or uniqueness obligation. Schema review found one High future-link
defect because the validator requires exact `#completion-evidence`; the
historical headings and every batch link now use that reserved destination.
The first focused lint also found the literal JA `canonical English:` marker
missing; both companion markers were corrected. Independent equivalence,
schema/test-sufficiency/boundary, and source-documentation/EN-JA re-reviews all
end **NO FINDINGS**.

All six immutable preimages replay at 107 lines and their frozen hashes. Both
source-TSV hashes, byte order/final LF, globally exhaustive source-qualified
headings, adjacent anchors, exact historical authority and chronology,
Task-249R correction, Task-259 separation, semantic exclusions, nine-path
scope, eight index rows, protected no-ops, audit no-impact, language-local
future links, and schema-v1 `1/1/6/6/8` plan were checked. Recursive
contract/index/link/fragment lint passes `1/1`; full checker/runner lint passes
`15/15` each, checker/runner libraries `530/530` and `600/600`, and runner
metadata `137/137`.

`cargo fmt --all --check`, offline Cargo metadata, warnings-denied
all-target/all-feature Clippy, and the full all-target/all-feature workspace
suite including all frontend and lexer benchmarks pass. All five CLIs exit
zero with 23 warnings and zero errors each and reproduce every frozen stdout
hash. The six protected path counts and path hashes reproduce exactly; zero
protected diff from the verified clean starting HEAD retains every frozen
content hash. Trace, coverage audit, the 714-line ledger, source TSV, and
`git diff --check` also reproduce. Final independent read-only quality ends
**NO FINDINGS**; all nine hard gates PASS, no score cap applies, and the valid
score is `100/100` (`20/20/15/15/10/10/5/5`). Exact staging and the task-only
commit remain.

## Migration Evidence

The prerequisite committed separately as
`9469d2a0868a39b4cce9685afb69b42f591524c0`. Fresh inventory was clean at
`origin/main...HEAD=0/21`; protected
`stash@{0}=f65cf4a13752ec380710814a9ac6392ccb9d75d4` was unchanged. All six
immutable preimages replayed from that commit at 107 lines and their frozen
hashes before editing.

The mechanical migration changes exactly the six declared checker sources,
this EN/JA pair, and `legacy_compactions.tsv`: nine paths. Only the six whole
sections become language-local completion-evidence redirects. Their 107
physical lines become 12 redirect-plus-separator lines, a reduction of 95;
source diff is six additions and 101 deletions. Both implementation-
verification sections and all durable owners remain.

The ledger now has 730 physical lines. This batch adds exactly one task, six
redirects over six distinct source paths, and eight index records. Its
expanded-inventory SHA-256 is
`0685c2259dbf909f4e8724d479ddd979f5695084df18484dd74ade26eb99f9e1`;
its complete physical SHA-256 is
`fbb5bae996031bb0137302ae375eab64c14a0475fdfff4a5478964d3ae7a9c87`.
The immutable source TSV remains
`5f6e31b89902a747fa0ba141cef966e5aff6cb0f0f2b79b56e447584fce6289f`.

The first focused lint correctly rejected retained legacy headings and their
resulting wrong anchors. Removing the six headings as required by schema v1
produced the frozen `+6/-101` source diff; focused recursive
schema/link/fragment lint and `git diff --check` now pass. Independent
migration equivalence/boundary, schema/test-sufficiency, and source-documentation/
EN-JA reviews end **NO FINDINGS** after correcting one Low stale-consumer phrase.
All six preimages replay exactly from prerequisite commit `9469d2a`; forbidden
headings are absent, redirects total six, and ledger schema/cardinality/expanded
inventory reproduce `1/1/6/6/8` and the frozen hashes above.

Full migration-state verification passes checker/runner lint `15/15` each,
checker/runner libraries `530/530` and `600/600`, metadata `137/137`,
`cargo fmt --all --check`, offline Cargo metadata, warnings-denied Clippy, and
`cargo test --workspace --all-targets --all-features --no-fail-fast`, including
all frontend and lexer benchmarks. All five CLIs exit zero with 23 warnings and
zero errors each and reproduce every frozen stdout hash. The six protected path
counts and path hashes reproduce exactly; zero protected diff from `9469d2a`
retains every frozen content hash. Trace and coverage hashes, the immutable
source TSV, ledger `730`/physical and expanded hashes, and `git diff --check`
also reproduce.

Final independent read-only quality ends **NO FINDINGS**. All nine hard gates
PASS, no score cap applies, and the valid score is `100/100`
(`20/20/15/15/10/10/5/5`). Residual risk is none in scope; origin divergence
remains report-only and the protected stash remains unchanged. Exact staging
and the task-only commit remain.

## Handoff

Complete exact staging and the task-only migration commit; then fresh-inventory
the next checker duplication family.
Parent reasoning remains `xhigh`; independent reviews use `high`, deterministic
inventory may use `medium`.
