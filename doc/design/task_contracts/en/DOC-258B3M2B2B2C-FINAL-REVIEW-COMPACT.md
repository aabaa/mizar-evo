# Task DOC-258B3M2B2B2C-FINAL-REVIEW-COMPACT: Structure-Update Final-Review Compaction

> Canonical language: English. Japanese companion:
> [../ja/DOC-258B3M2B2B2C-FINAL-REVIEW-COMPACT.md](../ja/DOC-258B3M2B2B2C-FINAL-REVIEW-COMPACT.md).

This documentation-maintenance contract freezes one completed checker review
family before exact whole-section migration. It cannot change language
behavior, test intent, API, diagnostics, traceability, or coverage.

## Identity And Status

| Field | Frozen value |
|---|---|
| Task | `DOC-258B3M2B2B2C-FINAL-REVIEW-COMPACT` |
| Status | Documentation prerequisite committed; exact migration, independent migration reviews, full verification, and final quality complete. Exact staging and commit remain. |
| Purpose | Centralize repeated Task-258B3M2B2B2C final-review evidence while retaining every broad-verification, frozen, implementation, post-commit, runner, todo, and audit owner. |
| Owners | Migration policy, historical [258B3M2B2B2C](./258B3M2B2B2C.md#completion-evidence), [checker plan](../../mizar-checker/en/00.crate_plan.md#task-index), and [runner plan](../../mizar-test/en/00.crate_plan.md#task-index) |
| Consumers | Eighteen checker source paths (nine EN/JA pairs), four Task Indexes, and the post-migration generic schema-v1 ledger/lint |
| Historical sequence | B2CP implementation `b146f0f7` -> B2C prerequisite `d6076cc7` -> B2C implementation `e8373c68` -> B3P prerequisite `285a1f11` |
| Documentation prerequisite | `e2ee5ffc3c73d1642c68f03bb43372b60a0fc292` |
| Readiness | Clean selection HEAD `787c16fb682db58f2a9fddc0d3f9aee1f9fd22bf`, `origin/main...HEAD=0/10`, protected `stash@{0}=f65cf4a13752ec380710814a9ac6392ccb9d75d4`; exact selection is dependency-ready. |

## Authority And Classification

Authority is the user-approved checker-first consolidation program,
[`AGENTS.md`](../../../../AGENTS.md), the
[migration policy](../../autonomous_crate_development.md#migration-policy),
the historical contract's canonical/test owners, and reviewed history. Source
behavior is not normative for this maintenance task.

| Class | Decision |
|---|---|
| `design_drift` | Eighteen checker sections repeat one final review checkpoint; the paired historical contract becomes their shared evidence owner. |
| `spec_gap` / `test_gap` | None for this structural migration; historical B2C authority, tests, and deferrals remain unchanged. |
| `source_drift` / `source_undocumented_behavior` | None introduced; production source is protected. |
| `test_expectation_drift` | None; canonical and executable test-intent artifacts are protected. |
| `boundary_violation` | None. Every selection is one complete H2/H3 section in a distinct source path. It contains only final no-findings/quality evidence; all detailed owner-local contracts and the runner review surface remain. The two `source_structure.md` selections validly terminate at EOF. |
| `repo_metadata_conflict` | Historical remote-ref movement remains report-only and human-owned. Current `0/10` distance is measured, not repaired; no fetch/reset/push is authorized. |

## Frozen Preimage And Anchors

[`DOC-258B3M2B2B2C-FINAL-REVIEW-COMPACT.sources.tsv`](../DOC-258B3M2B2B2C-FINAL-REVIEW-COMPACT.sources.tsv)
contains 18 byte-sorted data rows plus two comments and final LF. Data-row
SHA-256 is
`580a1cf881d5871db1750c26e683ba21b5c762e1873025f9d98d80cc4b05ffba`;
the complete 20-line TSV SHA-256 is
`7f5f682e796af301a698e17dc5948f1b30a18489b8155e8016e630447a4d5059`.

The selection is 18 unique sections over 18 paths, 137 physical lines: EN
`9/72`, JA `9/65`, checker `18/137`, runner `0/0`; sixteen are H2 and two are
H3. No selected section contains a nested heading, table, or fence. After
migration, each final-review section remains after its retained paired B2C
broad-verification section and before the following retained owner:

| Paired checker source | Retained EN following heading |
|---|---|
| `00.crate_plan.md` | `## Task 258B3M2B2B2C Post-Commit Closure` |
| `bilingual_sync_audit.md` | `## Task 258B3M2B2B2C Closure and Task 258B3M2B2B3P Synchronization` |
| `module_boundary_audit.md` | `## Task 258B3M2B2B3A Frozen Module Boundary` |
| `payload_family_decomposition.md` | `### Task 258B3M2B2B3P Frozen Lower Set-Term Reuse` |
| `resolved_typed_ast.md` | `## Task 258B3M2B2B3A Frozen Final-AST Contract` |
| `source_spec_audit.md` | `## Task 258B3M2B2B2C Post-Commit and Task 258B3M2B2B3P Specification Audit` |
| `source_statement.md` | `## Task 258B3M2B2B3P Statement-Owner Deferral` |
| `source_structure.md` | `EOF` |
| `typed_ast.md` | `## Task 258B3M2B2B3A Frozen Typed-AST Installer` |

The JA companions have matching levels and language-local equivalent anchors.
All 18 preimages replay with their frozen hashes and line counts.

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

The protected trace remains
`55b754c8c4d0d293a1c44e2ba4b0090f407bba1d429b461b6cb4d6ddca9ca2b3`;
the coverage audit remains `2aa808aa...685f`; and the prerequisite leaves the
614-line ledger at physical SHA-256
`d3bf34059a5a30dc86a2feee58cf9b3c400daaf49157121960f8096b57e6f2a2`.
Expected CLI hashes remain plan `700f4bf5`, parse `a8a7aa63`, declaration
`71e83ba0`, type `4b2c7bd5`, and proof `ccf3d2d4`.

## Scope, Reviews, Verification, And Exit

The prerequisite changes exactly nine paths: this EN/JA pair, the historical
EN/JA pair, source TSV, and four plans. Each plan receives task and batch Task
Index rows, eight rows total. It changes no selected preimage, ledger,
specification, `.miz`, fixture, sidecar, expectation, trace, coverage audit,
Rust/Cargo, public API, diagnostic, count/hash/status, or behavior.

After a separate prerequisite commit and fresh replay, migration may replace
only the 18 declared sections with language-local redirects to
`258B3M2B2B2C.md#completion-evidence`. It changes exactly the 18 sources, this
EN/JA pair, and `legacy_compactions.tsv`: 21 paths. The 137 physical lines
become 34 redirect-plus-separator lines, a reduction of 103; the two EOF
redirects need no following separator line. Ledger impact is
one batch, one task, 18 redirects over 18 distinct paths, eight index records,
and one expanded-inventory hash. Source TSV, historical contracts, and indexes
become immutable.

The checker-only selection deliberately leaves all `mizar-test` review and
owner documents unchanged. `doc/design/spec_coverage_audit.md` also remains
unchanged because design mapping, trace status, ownership, and credit do not
change. Goal/proof/theorem acceptance, facts, result/update typing,
functional-copy meaning, Core/CFG/VC/ATP, active dispatch, and every language
behavior are forbidden.

Prerequisite and migration each require independent contract/equivalence,
test-sufficiency, boundary, EN/JA/source-document consistency, and final-quality
reviews as applicable, ending **NO FINDINGS**. Verification includes preimage/
anchor replay, recursive contract/link/fragment and generic-ledger lint, full
lint policies, checker/runner/metadata tests, formatting, Cargo metadata,
warnings-denied Clippy, workspace tests, five CLIs, protected counts/hashes,
`git diff --check`, exact staging, all nine hard gates, and an uncapped score
`>=90/100`. No push or stash mutation is authorized.

## Documentation-Prerequisite Evidence

Selection-boundary, contract/equivalence/EN-JA, and test-sufficiency/schema
reviews ended **NO FINDINGS**. Independent replay passed all `18/137`
preimages, both TSV hashes, every retained anchor including two EOF cases,
source/task uniqueness, exact nine-path scope, eight index rows, and the
deliberate ledger/coverage/source/Cargo no-ops. The first focused link check
found three incorrect fragments and two JA companion markers in newly added
contracts; all five prerequisite-local issues were corrected and the exact
recursive check passed on rerun.

Full checker and runner lint passed `15/15` each; checker and runner libraries
passed `530/530` and `600/600`; runner metadata passed `137/137`.
`cargo fmt --all --check`, offline Cargo metadata, warnings-denied
all-target/all-feature Clippy, the full offline workspace test suite, and
`git diff --check` passed. Protected specification, `.miz`, expectation,
checker/runner production, Cargo, trace, coverage-audit, and 614-line ledger
surfaces are unchanged; the trace remains `55b754c8...ca2b3`, the coverage
audit `2aa808aa...685f`, and the ledger `d3bf3405...f2a2`.

All five CLIs exited zero with the unchanged 23-line warning stream. Their
stdout hashes are plan
`700f4bf503783742cefd8004fa095675b7476d46e9a3a6dd439916d237eb6718`,
parse-only
`a8a7aa639d2ebc65eddc923c7e9369ea5637d50e935f808600f446da1bfbda56`,
declaration-symbol
`71e83ba0b20d4015e07b3bd2c0c4db2837b6151d1251812caed7954530d53c74`,
type-elaboration
`4b2c7bd5ec3cc56e5672fb351126126230ec84fba9bd2bd9049a516d378fab7f`,
and proof-verification
`ccf3d2d4d0a3755e00989d97af369a7c560302f76798d0a185d57ec3891e8450`.

Final read-only quality review ended **NO FINDINGS**. All nine hard gates PASS,
no score cap applies, and the valid score is `100/100`
(`20/20/15/15/10/10/5/5`). At that prerequisite checkpoint, only exact staging
and the dedicated commit remained; both closed in `e2ee5ffc`.

## Migration Evidence

The prerequisite committed as `e2ee5ffc3c73d1642c68f03bb43372b60a0fc292`.
Fresh post-commit inventory was clean at `origin/main...HEAD=0/11`; protected
`stash@{0}=f65cf4a13752ec380710814a9ac6392ccb9d75d4` was unchanged, and all
18 frozen preimages replayed at 137 lines before editing.

The mechanical migration changes exactly the 18 declared checker sources,
this EN/JA pair, and `legacy_compactions.tsv`: 21 paths. It replaces only the
18 complete final-review sections with language-local redirects. The 137
physical lines become 34 redirect-plus-separator lines, a reduction of 103;
the two EOF redirects do not need a following separator line.
Every broad-verification, frozen, implementation, post-commit, runner, and
unlisted owner remains.

The ledger now has 642 physical lines. The batch adds exactly one task, 18
redirects over 18 distinct source paths, and eight index records. Its expanded
inventory SHA-256 is
`a8b45aaac013212a4fcc90f28f7204f54ee1353dca25c57a09d799a10df4bc7d`;
its complete physical SHA-256 is
`eb3d7692ac7050e33ceda0708ce137b8af3646a1bc040abacb4c4479377106c3`.
The immutable source TSV remains
`7f5f682e796af301a698e17dc5948f1b30a18489b8155e8016e630447a4d5059`.
Focused generic-ledger/link/fragment lint and `git diff --check` pass.

Independent equivalence/boundary and source-documentation/EN-JA reviews ended
**NO FINDINGS**. The first test-sufficiency/schema review found that the two
EOF redirects need no separator and therefore corrected the predicted and
measured postimage from 36 to 34 lines, and the reduction from 101 to 103;
finding-specific re-review ended **NO FINDINGS**. No semantic, test-intent,
ownership, trace, coverage, or protected-source change was required.

Full checker and runner lint passed `15/15` each; checker and runner libraries
passed `530/530` and `600/600`; runner metadata passed `137/137`.
`cargo fmt --all --check`, offline Cargo metadata, warnings-denied
all-target/all-feature Clippy, the full offline workspace test suite, and
`git diff --check` passed. The five CLIs exited zero with 23 stderr lines each
and reproduced the prerequisite stdout hashes recorded above. Every protected
count and hash in the frozen baseline, including trace and coverage audit,
remains unchanged.

Final read-only quality review ended **NO FINDINGS**. All nine hard gates PASS,
no score cap applies, and the valid score is `100/100`
(`20/20/15/15/10/10/5/5`). Only exact staging and the task-only commit remain.

## Handoff

After both task-only commits and clean post-commit inventory, select exactly
one dependency-ready checker-owned whole-section duplication family. Keep the
parent at `xhigh`, use `high` for independent reviews, and `medium` only for
deterministic inventory.
