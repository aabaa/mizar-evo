# Task DOC-258B3M2B2B3P-REVIEW-COMPACT: Proof-Context Review-Evidence Compaction

> Canonical language: English. Japanese companion:
> [../ja/DOC-258B3M2B2B3P-REVIEW-COMPACT.md](../ja/DOC-258B3M2B2B3P-REVIEW-COMPACT.md).

This documentation-maintenance contract freezes one completed checker review
family before exact whole-section migration. It cannot change language
behavior, test intent, API, diagnostics, traceability, or coverage.

## Identity And Status

| Field | Frozen value |
|---|---|
| Task | `DOC-258B3M2B2B3P-REVIEW-COMPACT` |
| Status | Documentation prerequisite committed; migration reviewed, fully verified, and final-quality approved. Exact staging and the migration commit remain. |
| Purpose | Centralize repeated Task-258B3M2B2B3P documentation-prerequisite review evidence while retaining every final-quality, frozen, implementation, runner, todo, and audit owner. |
| Owners | Migration policy, historical [258B3M2B2B3P](./258B3M2B2B3P.md#completion-evidence), [checker plan](../../mizar-checker/en/00.crate_plan.md#task-index), and [runner plan](../../mizar-test/en/00.crate_plan.md#task-index) |
| Consumers | Twelve checker source paths (six EN/JA pairs), four Task Indexes, and the post-migration generic schema-v1 ledger/lint |
| Historical sequence | B2C implementation `e8373c68` -> B3P prerequisite `285a1f11` -> B3P implementation `abbfedfc` -> B3A prerequisite `f4ff4596` |
| Documentation prerequisite | `5dca509241fdfa01736202f253cff1870075b8cb` |
| Readiness | Clean selection HEAD `9c31231eae4a0bb1cff9d6bb037ab030eb2d5fef`, `origin/main...HEAD=0/8`, protected `stash@{0}=f65cf4a13752ec380710814a9ac6392ccb9d75d4`; exact selection is dependency-ready. |

## Authority And Classification

Authority is the user-approved checker-first consolidation program,
[`AGENTS.md`](../../../../AGENTS.md), the
[migration policy](../../autonomous_crate_development.md#migration-policy),
the historical contract's canonical/test owners, and reviewed history. Source
behavior is not normative for this maintenance task.

| Class | Decision |
|---|---|
| `design_drift` | Twelve checker sections repeat one prerequisite review checkpoint; the paired historical contract becomes their shared evidence owner. |
| `spec_gap` / `test_gap` | None for this structural migration. Historical B3P `test_gap` is preserved as time-local evidence and was closed by `abbfedfc`. |
| `source_drift` / `source_undocumented_behavior` | None introduced. Historical bounded B3P `source_drift` remains recorded and closed. |
| `test_expectation_drift` | None; canonical and executable test-intent artifacts are protected. |
| `boundary_violation` | The first 24-section draft selected adjacent review and final-quality sections in each path, which schema v1 cannot represent because it permits only one expanded same-task redirect per source. The selection is therefore narrowed to one review section per source. All final-quality, frozen, and implementation sections, every runner document, todo ledger, coverage/source/boundary owner, and every unlisted section remain. The larger 32-section B4A/B4B candidate is deferred because its H2s mix durable owner-local state. |
| `repo_metadata_conflict` | Historical remote-ref movement remains report-only and human-owned. Current `0/8` distance is measured, not repaired; no fetch/reset/push is authorized. |

## Frozen Preimage And Anchors

[`DOC-258B3M2B2B3P-REVIEW-COMPACT.sources.tsv`](../DOC-258B3M2B2B3P-REVIEW-COMPACT.sources.tsv)
contains 12 byte-sorted data rows plus two comments and final LF. Data-row
SHA-256 is
`89079f15b6a8a0d06c5587392cf8916107ae3cabdcc96f0765835bebdf8bdd3f`;
the complete 14-line TSV SHA-256 is
`0f40c4b508344a3bcb411e02d2fef4fca64a5df6f1bce4c2c9b4bd70f8bacfb9`.

The selection is 12 unique H2/H3 sections over 12 paths, 134 physical lines:
EN `6/68`, JA `6/66`, checker `12/134`, runner `0/0`; ten are H2 and two are
H3. No selected section contains a nested heading, table, or fence. After
migration, each review section remains between these retained anchors:

| Paired checker source | Retained EN preceding / following heading |
|---|---|
| `00.crate_plan.md` | `## Task 258B3M2B2B3P Frozen Set-Enumeration Proof-Context Contract` / `## Task 258B3M2B2B3P Final Quality Status` |
| `bilingual_sync_audit.md` | `## Task 258B3M2B2B2C Closure and Task 258B3M2B2B3P Synchronization` / `## Task 258B3M2B2B3P Final-Quality Synchronization` |
| `payload_family_decomposition.md` | `### Task 258B3M2B2B3P Frozen Lower Set-Term Reuse` / `### Task 258B3M2B2B3P Final Family Quality` |
| `source_set_term.md` | `## Task 258B3M2B2B3P Frozen Proof-Context Enumeration Reuse` / `## Task 258B3M2B2B3P Final Quality Status` |
| `source_spec_audit.md` | `## Task 258B3M2B2B2C Post-Commit and Task 258B3M2B2B3P Specification Audit` / `## Task 258B3M2B2B3P Final Quality Audit` |
| `source_statement.md` | `## Task 258B3M2B2B3P Statement-Owner Deferral` / `## Task 258B3M2B2B3P Final Quality Status` |

The JA companions have the same levels and language-local equivalent anchors.
All 12 preimages replay with their frozen hashes and line counts.

## Frozen Protected Baseline

The expected prerequisite and migration delta is zero for every row:

| Surface | Paths | Path SHA-256 | Content SHA-256 |
|---|---:|---|---|
| specification | 64 | `d900ba9e43ab094925f36493f830e0c6a2964be2911d5d229014a58842067a25` | `b30dd5519191a4407399826faf91cc58853d7944df7630734b5ab05de48c9f7b` |
| `.miz` | 343 | `d94980e167b4b8ac196f91e7694ff044080c6fb4d04c135b3cd5e65b9a019f22` | `54e6ea1254a0bd963c39026d788711507f353e9c6df3b4f9fd268b2e9f240afb` |
| expectation | 435 | `22a5ee257a294e3f2ed4b24bf9ca243d037bbd798f009d0d1e3a176dad8b4fea` | `b5f0ed1a8d73bbfb78af5cad87a8d426e97269f7f70163750893a9fe1f39d2ea` |
| checker production | 30 | `a41370d7150a587369cea5f7a67b60417dd1372592f55c0d65bec369eb39fdc6` | `05fd5e0eaed4361b824693941e9056a552c476f050915ea5052a85c8c7174dfd` |
| runner production | 90 | `05245a54160dfce17336b476b07885eb6d5afe138c4780a6a6a7b47043e7248c` | `210f294aebfe22c12324ef9919ac68147f8025f0da8de166403dada87bac5eae` |
| Cargo | 21 | `d93f2816b760d8ba45430d6d8570e598864aa7b20b19a001f45171d36fd3a030` | `146e9b4024d2c344b2eca9c6f5ca6d6a80a3de427e382953a2280bc63cb3ecca` |

The protected trace is
`55b754c8c4d0d293a1c44e2ba4b0090f407bba1d429b461b6cb4d6ddca9ca2b3`.
Expected CLI hashes remain plan `700f4bf5`, parse `a8a7aa63`, declaration
`71e83ba0`, type `4b2c7bd5`, and proof `ccf3d2d4`.

## Scope, Reviews, Verification, And Exit

The prerequisite changes exactly nine paths: this EN/JA pair, historical
EN/JA pair, source TSV, and four plans. Each plan receives task and batch Task
Index rows, eight rows total. It changes no selected preimage, ledger,
specification, `.miz`, fixture, sidecar, expectation, trace, coverage audit,
Rust/Cargo, public API, diagnostic, count/hash/status, or behavior.

After a separate prerequisite commit and fresh replay, migration may replace
only the 12 declared sections with language-local redirects to
`258B3M2B2B3P.md#completion-evidence`. It changes exactly the 12 sources, this
EN/JA pair, and `legacy_compactions.tsv`: 15 paths. The 134 physical lines
become 24 redirect-plus-separator lines, a reduction of 110. Ledger impact is
one batch, one task, 12 redirects over 12 distinct paths, eight index records,
and one expanded-inventory hash. Source TSV, historical contracts, and indexes
become immutable.

The checker-only selection deliberately leaves all `mizar-test` review and
owner documents unchanged. `doc/design/spec_coverage_audit.md` also remains
unchanged because design mapping, trace status, ownership, and credit do not
change. Goal/proof/theorem acceptance, facts, result/sethood/element semantics,
Core/CFG/VC/ATP, active dispatch, B3A+, and every language behavior are
forbidden.

Prerequisite and migration each require independent contract/equivalence,
test-sufficiency, boundary, EN/JA/source-document consistency, and final-quality
reviews as applicable, ending **NO FINDINGS**. Verification includes preimage/
anchor replay, recursive contract/link/fragment and generic-ledger lint, full
lint policies, checker/runner/metadata tests, formatting, Cargo metadata,
warnings-denied Clippy, workspace tests, five CLIs, protected counts/hashes,
`git diff --check`, exact staging, all nine hard gates, and an uncapped score
`>=90/100`. No push or stash mutation is authorized.

## Documentation-Prerequisite Evidence

The first test-sufficiency review found the blocking schema-v1
`boundary_violation` in the adjacent 24-section draft. After narrowing the
inventory to one review section per source, finding-specific test-sufficiency,
contract-completeness, and historical-equivalence/EN-JA re-reviews all ended
**NO FINDINGS**. Independent replay passed all `12/134` preimages, both TSV
hashes, retained anchors, source/task uniqueness, exact nine-path scope, and
the deliberate ledger/coverage/source/Cargo no-ops.

Recursive and full runner lint passed `1/1` and `15/15`; checker lint passed
`15/15`; checker and runner libraries passed `530/530` and `600/600`; runner
metadata passed `137/137`. `cargo fmt --all --check`, offline Cargo metadata,
warnings-denied all-target/all-feature Clippy, the full offline workspace test
suite, and `git diff --check` passed. Protected path counts/hashes reproduced
all six frozen rows, protected bytes are unchanged from selection HEAD, the
trace remains `55b754c8...ca2b3`, the legacy ledger remains
`d261a5c8...fddbb`, and the coverage audit remains `2aa808aa...685f`.

All five CLIs exited zero with the unchanged 23 warnings. Their stdout hashes
are plan `700f4bf503783742cefd8004fa095675b7476d46e9a3a6dd439916d237eb6718`,
parse-only `a8a7aa639d2ebc65eddc923c7e9369ea5637d50e935f808600f446da1bfbda56`,
declaration-symbol `71e83ba0b20d4015e07b3bd2c0c4db2837b6151d1251812caed7954530d53c74`,
type-elaboration `4b2c7bd5ec3cc56e5672fb351126126230ec84fba9bd2bd9049a516d378fab7f`,
and proof-verification
`ccf3d2d4d0a3755e00989d97af369a7c560302f76798d0a185d57ec3891e8450`.

Final read-only quality review ended **NO FINDINGS**. All nine hard gates PASS,
no score cap applies, and the valid score is `98/100`
(`20/20/15/14/10/10/5/4`). At that prerequisite checkpoint, the only residual
state was parent-owned exact staging, the prerequisite commit, and the
separately reviewed migration.

## Migration Evidence

The prerequisite committed as `5dca509241fdfa01736202f253cff1870075b8cb`.
Fresh post-commit inventory was clean at `origin/main...HEAD=0/9`; protected
`stash@{0}=f65cf4a13752ec380710814a9ac6392ccb9d75d4` was unchanged, and all
12 frozen preimages replayed at 134 lines before editing.

The mechanical migration changes exactly the 12 declared checker sources,
this EN/JA pair, and `legacy_compactions.tsv`: 15 paths. It replaces only the
12 complete review sections with language-local redirects. Their 134 physical
lines become 24 redirect-plus-separator lines, a reduction of 110. Every
final-quality, frozen, and implementation section, every runner owner, and
every unlisted section remains.

The ledger now has 614 physical lines. The batch adds exactly one task, 12
redirects over 12 distinct source paths, and eight index records. Its expanded
inventory SHA-256 is
`9b72ed0867a2e459ac989cd11e185f859dd8c1f5390ba923de5544c69e80f8dd`;
its complete physical SHA-256 is
`d3bf34059a5a30dc86a2feee58cf9b3c400daaf49157121960f8096b57e6f2a2`.
The immutable source TSV remains
`0f40c4b508344a3bcb411e02d2fef4fca64a5df6f1bce4c2c9b4bd70f8bacfb9`.
Focused generic-ledger/link/fragment lint and `git diff --check` pass.
Independent test-sufficiency, equivalence/boundary, and source/document/EN-JA
consistency reviews ended **NO FINDINGS**. They replayed every committed
preimage and retained owner, all redirects/anchors/indexes, ledger
ordering/arithmetic/hashes, chronology, bilingual parity, protected scope, and
the audit no-impact decision. Generic schema-v1 lint is sufficient; no new
Rust/schema/fixture/test is required or authorized.

Focused/full runner lint passed `1/1` and `15/15`; checker lint passed `15/15`;
checker/runner libraries passed `530/530` and `600/600`; runner metadata passed
`137/137`. Formatting, offline Cargo metadata, warnings-denied Clippy, the full
offline workspace suite, all six protected count/path/content baselines,
trace/coverage/source-TSV no-ops, and `git diff --check` passed. All five CLIs
exited zero with 23 unchanged warnings and exactly reproduced the prerequisite
stdout hashes. Final read-only quality review ended **NO FINDINGS**. All nine
hard gates PASS, no score cap applies, and the valid score is `98/100`
(`20/20/15/14/10/10/5/4`). Only exact staging and the migration commit remain.

## Handoff

After both task-only commits and clean post-commit inventory, select exactly
one dependency-ready checker-owned whole-section duplication family. Keep the
parent at `xhigh`, use `high` for independent reviews, and `medium` only for
deterministic inventory.
