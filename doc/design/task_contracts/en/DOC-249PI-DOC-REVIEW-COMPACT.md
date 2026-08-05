# Task DOC-249PI-DOC-REVIEW-COMPACT: Property-Type Review-Evidence Compaction

> Canonical language: English. Japanese companion:
> [../ja/DOC-249PI-DOC-REVIEW-COMPACT.md](../ja/DOC-249PI-DOC-REVIEW-COMPACT.md).

This maintenance contract freezes one checker-only historical review family.
It cannot change language behavior, test intent, API, diagnostics,
traceability, or coverage.

## Identity And Status

| Field | Frozen value |
|---|---|
| Task | `DOC-249PI-DOC-REVIEW-COMPACT` |
| Status | Documentation prerequisite reviews, verification, and final read-only quality complete; exact staging and commit remain. Migration is prohibited until that commit and fresh replay. |
| Purpose | Centralize repeated Task-249PI documentation-prerequisite and frozen-review evidence while retaining every durable implementation and runner owner. |
| Owners | Migration policy, historical [249PI](./249PI.md#completion-evidence), [checker plan](../../mizar-checker/en/00.crate_plan.md#task-index), and [runner plan](../../mizar-test/en/00.crate_plan.md#task-index) |
| Consumers | Eight checker source paths, four Task Indexes, and the post-migration schema-v1 ledger/lint |
| Sequence | `4c3f74b0` -> `7e194bb3` -> `73a34f94` -> `52cf07be` |
| Documentation prerequisite | Pending |
| Readiness | Clean selection HEAD `bee5a905c3e0b291018a33165b382d14bb5eb9fd`, `origin/main...HEAD=0/16`, protected `stash@{0}=f65cf4a13752ec380710814a9ac6392ccb9d75d4`; dependency-ready. |

## Authority And Classification

Authority is the user-approved checker-first compaction program,
[`AGENTS.md`](../../../../AGENTS.md), the
[migration policy](../../autonomous_crate_development.md#migration-policy),
the historical contract's retained canonical/test owners, and reviewed
history. Source behavior is not normative.

| Class | Decision |
|---|---|
| `design_drift` | Eight checker sections repeat the `7e194bb3` prerequisite freeze/review checkpoint; the historical contract becomes their shared evidence owner. |
| `spec_gap` / `test_gap` | None for this structural task; Task-249PI's historical classifications, findings, and closures remain unchanged. |
| `source_drift` / `source_undocumented_behavior` | None introduced; production source is protected. |
| `test_expectation_drift` | None; specification, `.miz`, expectations, sidecars, and trace are protected. |
| `boundary_violation` | Avoided by retaining the duplicated plan headings, the source-type API and implementation-verification sections, every Typed/final/payload-family owner, checker and runner TODO, all runner sections, implementation sections, active APIs, audits, and unlisted content. The tempting plan sections are excluded because their headings also occur in TODO owners and would violate schema-v1 global forbidden-heading enforcement. |
| `repo_metadata_conflict` | Current `0/16` is report-only and not repaired; fetch, reset, push, and stash mutation are unauthorized. |

## Frozen Preimage And Anchors

[`DOC-249PI-DOC-REVIEW-COMPACT.sources.tsv`](../DOC-249PI-DOC-REVIEW-COMPACT.sources.tsv)
contains eight byte-sorted rows plus two comments and final LF. Data-row
SHA-256 is `f4acd99daffb0d77a53ef2ca76735f4f88c64f2313f245016be2f6a4cb2341e5`;
complete-file SHA-256 is
`5d61e5c9982432deb1a671ed45168ca2a811b33981cf90cd6d2dfb5657220d2e`.

The selection is eight globally unique H2 sections over eight checker paths,
130 physical lines: EN `4/76`, JA `4/54`. None contains a nested heading, table, or
fence. Retained EN preceding/following owners are:

| Source | Retained anchors |
|---|---|
| `bilingual_sync_audit.md` | `## Task 264 Frozen-Contract Synchronization` / `## Task 249PI Implementation Synchronization` |
| `module_boundary_audit.md` | `## Task 264 Frozen Module Boundary` / `## Task 249PI Implemented Module Boundary` |
| `source_spec_audit.md` | `## Task 264 Frozen Source/Specification Status` / `## Task 249PI Implemented Source/Specification Audit` |
| `source_type.md` | `## Task 249PI Frozen Property-Implementation Composition` / `## Task 249PI Implementation Verification` |

JA companions have matching levels and language-local equivalent anchors. All
eight headings are unique; no Task-249PI contract, index, or ledger identity
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
the 678-line ledger remains
`a26fe1fedd9f6b634de66daff85682d3ef63871242df77953eb4b881ec2a1d3a`.
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
`249PI.md#completion-evidence`. It changes exactly those eight sources, this
pair, and `legacy_compactions.tsv`: eleven paths. The 130 lines become 16
redirect-plus-separator lines, a reduction of 114. Ledger impact is one batch,
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

Initial inventory found a Medium `boundary_violation`: the tempting plan
headings also occur in retained TODO owners and would fail schema-v1 global
forbidden-heading enforcement. The selection was corrected to the globally
unique source-type documentation-verification pair plus six audit summaries.
Their unique historical hashes, counts, and review claims were moved into the
historical pair. Independent specification/equivalence and
test-sufficiency/schema reviews then ended **NO FINDINGS**. A source-document
review's two missing review-history claims and checker-plan owner links were
fixed in both languages; re-review ended **NO FINDINGS**. No Rust, schema,
test, trace, coverage, or additional documentation change is warranted.

All `8/130` preimages replay at their frozen hashes and are globally unique,
flat H2 sections. Both TSV hashes, adjacent anchors, chronology, exact API and
error/site claims, ownership and semantic exclusions, exact nine-path scope
and eight index rows, protected no-ops, audit no-impact, language-local links,
and the future schema-v1 `1/1/8/8` ledger plan were checked. Recursive
pairing/manifest/link/fragment lint passed `1/1`; full checker/runner lint
passed `15/15` each, checker/runner libraries `530/530` and `600/600`, and
runner metadata `137/137`.

`cargo fmt --all --check`, offline Cargo metadata, warnings-denied
all-target/all-feature Clippy, and the full all-target/all-feature workspace
suite including benchmarks passed. All five CLIs exited zero with 23 stderr
lines and zero errors each and reproduced every frozen stdout hash.
Specification, `.miz`, expectation, checker/runner production, Cargo, trace,
coverage audit, and the 678-line ledger have zero delta; the immutable source
TSV has its frozen full-file hash. `git diff --check` passes. Final read-only
quality ended **NO FINDINGS**; all nine hard gates PASS, no score cap applies,
and the valid score is `100/100` (`20/20/15/15/10/10/5/5`). Exact staging and
the dedicated prerequisite commit remain.

## Migration Evidence

Pending the committed prerequisite and fresh preimage replay.

## Handoff

Complete exact staging and the dedicated commit. Then fresh-inventory the same
task and perform only its frozen schema-v1 migration. Parent reasoning remains
`xhigh`; independent reviews use `high`, deterministic inventory may use
`medium`.
