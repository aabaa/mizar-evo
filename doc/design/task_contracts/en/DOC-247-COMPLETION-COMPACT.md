# Task DOC-247-COMPLETION-COMPACT: Payload-Family Completion Compaction

> Canonical language: English. Japanese companion:
> [../ja/DOC-247-COMPLETION-COMPACT.md](../ja/DOC-247-COMPLETION-COMPACT.md).

This maintenance contract freezes one checker-only historical completion
family. It cannot change language behavior, test intent, public API,
diagnostics, traceability state, coverage, or descendant-task ownership.

## Identity And Status

| Field | Frozen value |
|---|---|
| Task | `DOC-247-COMPLETION-COMPACT` |
| Status | Complete. The migration is registered in the schema-2 ledger; task-local completion evidence below preserves the committed migration and clean replay. |
| Purpose | Centralize four repeated Task-247 plan/TODO completion sections while retaining every durable graph, audit, runner, trace, coverage, and sequencing owner. |
| Historical owner | [Task 247](./247.md#completion-evidence) |
| Plan indexes | [checker plan](../../mizar-checker/en/00.crate_plan.md#task-index) and [runner plan](../../mizar-test/en/00.crate_plan.md#task-index) |
| Consumers | Four selected checker source paths, four Task Indexes, and the future schema-v1 ledger/lint |
| Sequence | `b0930a0c` -> `0154ad74` -> this prerequisite -> separate migration |
| Readiness | Selection HEAD `cbacea8efa0c7ac60f16636c2932c49b877e3eae`, `origin/main...HEAD=0/2`, protected `stash@{0}=f65cf4a13752ec380710814a9ac6392ccb9d75d4`; repeated selection review ended **NO FINDINGS**. |

## Authority And Classification

Authority is the user-approved checker-first compaction program,
[`AGENTS.md`](../../../../AGENTS.md), the
[migration policy](../../autonomous_crate_development.md#migration-policy),
the accepted graph's row-specific `doc/spec/en/` and `.miz` references, the
five current Task-247 trace deferred-reason records, and reviewed Git history.
Source behavior is not normative.

| Class | Decision |
|---|---|
| `design_drift` | Four plan/TODO sections duplicate historical orchestration/completion evidence and no central Task-247 record existed. |
| `spec_gap` | No compaction-specific gap; the retained MC-G005 external diagnostic-code gate remains nonblocking. |
| `test_gap` / `source_drift` | Historical descendant gaps remain assigned by the accepted graph and are unchanged. |
| `source_undocumented_behavior` | None inferred or introduced. |
| `test_expectation_drift` | Parser Task 47 retains the omitted-`reconsider` disagreement; this task cannot repair or rebaseline it. |
| `boundary_violation` | Only the exact four flat sections may migrate. Selecting the graph, semantic/source/bilingual audits, runner state, trace, coverage, Core/root sequencing, module/API sections, queue sections, or Task-263+ semantics is forbidden. |
| `repo_metadata_conflict` | The branch is clean and two commits ahead of the observed `origin/main`; this report-only metadata state does not obscure the task-only target and must not be repaired. |

## Frozen Preimage And Anchors

[`DOC-247-COMPLETION-COMPACT.sources.tsv`](../DOC-247-COMPLETION-COMPACT.sources.tsv)
contains four byte-sorted data rows plus two comments and final LF. Data-row
SHA-256 is `85059c4125b162e5ab5dec2cd746fde488185027b288a4c19dbb847c48b78045`;
complete-file SHA-256 is
`ad6280a95b24d6d549a0c9a64a0f313b321ccee80f84a0bf78ef0bf21997b2fc`.

The selection is four source-locally unique flat H2 sections over four
distinct paths and 116 physical lines: checker EN plan/TODO `34/27` and
checker JA plan/TODO `32/23`. Each plan heading occurs once repository-wide;
the shared EN/JA TODO heading occurs exactly twice and both occurrences are
selected, so the inventory is globally exhaustive. No selected section has a
nested heading, table, fence, existing redirect, or inbound fragment link.

| Source | Preceding owner | Following owner |
|---|---|---|
| checker EN plan | `## Task 268 Completion` | `## Task 248 Frozen Source/Binding-Context Producer Contract` |
| checker JA plan | `## Task 268 completion` | `## Task 248 source/binding-context producer 確定 contract` |
| checker EN TODO | `## Tasks 266-268 Final Checker Handoff Queue` | `## Tasks 248-264 And 269-279 STEP 5 Source-Payload Producer Queue` |
| checker JA TODO | `## Tasks 266-268 Final Checker Handoff Queue` | `## Tasks 248-264/269-279 STEP 5 ソースペイロードproducer queue` |

The plan sections were introduced at end of file by `0154ad74` as 33/31 lines.
Commit `0ed76c20` appended the following Task-248 sections and thereby added
only the current terminating separator blank, producing the frozen 34/32-line
preimages. The TODO sections changed from pending at `b0930a0c` to the current
completed 27/23-line text in `0154ad74` and are byte-stable since that commit.

## Retained Owners And Exclusions

The accepted [graph](../../mizar-checker/en/payload_family_decomposition.md),
including Existing Boundary And Trace Ownership, Disagreement Classification,
and Task-247 Exit Criteria, remains the durable decomposition owner. The
semantic/source audits, mixed bilingual completion paragraph, runner plan and
TODO state, runner traceability, coverage audit, five current trace deferred-
reason records, root roadmap, Core Task-32 owners, module/API documents, and
the following Tasks 248--264/269--279 queue sections remain unchanged.

Those retained owners preserve every specification/test reference, family
assignment, consumer, boundary, gate, trace/coverage decision, and sequencing
fact. Existing Task-247 trace metadata contains exactly five matching current
records, at lines 2997, 3008, 3019, 3030, and 5907 in the selection state; the
line numbers are inventory evidence, not stable link targets.

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
The 822-line ledger remains
`1a3a07297f4f0aee4b13274df44322b52cf92bf71f0ed40824debd7d0aba6c59`.
Expected CLI stdout hashes remain plan
`700f4bf503783742cefd8004fa095675b7476d46e9a3a6dd439916d237eb6718`,
parse `a8a7aa639d2ebc65eddc923c7e9369ea5637d50e935f808600f446da1bfbda56`,
declaration `71e83ba0b20d4015e07b3bd2c0c4db2837b6151d1251812caed7954530d53c74`,
type `4b2c7bd5ec3cc56e5672fb351126126230ec84fba9bd2bd9049a516d378fab7f`,
and proof `ccf3d2d4d0a3755e00989d97af369a7c560302f76798d0a185d57ec3891e8450`.

## Scope, Expected Migration, And Audit Impact

The documentation prerequisite changes exactly nine paths: this pair, the
historical pair, the source TSV, and four plan Task Indexes. Each plan adds one
`247` row and one batch row, eight index records total. Selected preimages,
the ledger, protected artifacts, current trace/coverage state, public behavior,
and all retained owners remain unchanged.

After a separate prerequisite commit and fresh replay, migration may replace
only the four declared H2 sections with language-local redirects to
`247.md#completion-evidence`. It changes exactly seven paths: four sources,
this pair, and `legacy_compactions.tsv`. The 116 lines become eight redirect-
plus-separator lines, a reduction of 108; exact source diff is `+4/-112`.

Ledger impact is 14 lines, `822 -> 836`: one batch, one task, four redirects
over four distinct paths, and eight index records. The canonical 13-row task/
redirect/index payload excludes the batch row and has frozen SHA-256
`12da6c943ebc9cdd21c2ab8be9d5c72a1350c2bce74e07a5e81cf272c921385c`.
The expected 836-line physical ledger SHA-256 is
`33c569ebeac13be3f353177f6c23ddf40c581435950e0e47f57bcdcd7f3528cb`.
The source TSV, historical pair, and Task Index contents then become immutable.

`spec_coverage_audit.md` has no compaction impact: current owner mapping,
deferred reasons, status, backlinks, counts, and coverage credit do not change.

## Reviews, Verification, And Exit

The prerequisite requires independent contract/equivalence, schema/test-
sufficiency, boundary, and source-document/EN-JA reviews ending **NO FINDINGS**.
Migration repeats applicable equivalence, schema, bilingual, and final-quality
reviews. Both commits require all nine hard gates PASS, no score cap, and a
valid score `>=90/100`.

Verification includes source-preimage and anchor replay; generic schema/link/
fragment lint; checker/runner lint and libraries; metadata tests; formatting;
offline Cargo metadata; warnings-denied all-target/all-feature Clippy; full
workspace tests; all five CLIs; protected counts and hashes; exact ledger
order/hash/cardinality; `git diff --check`; exact cached-scope/content/
whitespace review; and unstaged/untracked inspection. No push, fetch, reset,
or stash mutation is authorized.

The prerequisite exits only with exact nine-path documentation scope, no
selected-preimage or protected-surface change, synchronized EN/JA, complete
reviews and verification, one task-only commit, and clean fresh inventory.
Migration exits only with exact four redirects/seven paths, ledger replay,
separate complete reviews and verification, one task-only commit, and clean
fresh inventory.

## Documentation-Prerequisite Evidence

Fresh inventory and independent selection review end **NO FINDINGS** for the
exact four-section family. Review rejected broader Task-247 audit, runner,
trace, coverage, graph, and queue surfaces as owner-local or mixed, corrected
the current trace inventory to exactly five Task-247 deferred-reason records,
and confirmed the four sections are historical, flat, source-locally unique
and globally exhaustive, unlinked, bilingually paired, and frozen at the
current selection bytes with the plan-separator history above. Contract
review initially found Medium history accuracy and Low raw-heading uniqueness
`design_drift`. EN/JA now record the exact `0154ad74 -> 0ed76c20` separator
history and three raw headings over four path-qualified occurrences.
Independent contract/equivalence, schema/test-sufficiency, and source-
document/EN-JA/boundary re-reviews all end **NO FINDINGS**. Full verification,
including all three long frontend benchmarks, passes. Checker/runner lint pass
`15/15` each, checker/runner libraries pass `530/530` and `600/600`, and runner
metadata passes `137/137`. Formatting, offline Cargo metadata, warnings-denied
all-target/all-feature Clippy, the full all-target/all-feature workspace suite,
and `git diff --check` pass. All five CLIs exit zero with 23 warnings and zero
errors each and reproduce the five frozen stdout hashes.

All four selected sections reproduce their `34/27/32/23` line counts and
frozen hashes; the source TSV reproduces both hashes; and the exact eight Task
Index records are present. Protected counts and path hashes reproduce as
specification 64, `.miz` 343, expectation 435, checker production 30, runner
production 90, and Cargo 21. Zero protected diff retains every frozen content
hash. Trace, coverage audit, the 822-line ledger, and their hashes reproduce
exactly. The final independent read-only quality review ends **NO FINDINGS**,
passes all nine hard gates, applies no score cap, and assigns **100/100**.
Exact nine-path staging produced prerequisite commit
`e22a4fa14f49bf02fa3209f249bbf45b9a2970e3`. Its fresh inventory was clean at
`origin/main...HEAD=0/3`, preserved the protected stash, and replayed all four
preimages before migration.

## Migration Evidence

Fresh replay from prerequisite commit `e22a4fa1` authorized only the frozen
seven-path migration. The four exact legacy sections are now four language-
local completion redirects, with source delta `+4/-112`; every neighboring
anchor and retained owner remains in place. The ledger adds exactly 14 sorted
rows: one batch, one task, four redirects over four paths, and eight existing
Task Index records. It is exactly 836 lines with physical SHA-256
`33c569ebeac13be3f353177f6c23ddf40c581435950e0e47f57bcdcd7f3528cb`,
and generic schema/link/fragment lint reproduces the canonical 13-row hash
`12da6c943ebc9cdd21c2ab8be9d5c72a1350c2bce74e07a5e81cf272c921385c`.
Protected specification, tests, expectations, production, Cargo, trace,
coverage audit, source TSV, historical contracts, and Task Index contents are
unchanged. Independent migration-equivalence, schema/test-sufficiency, and
bilingual/boundary reviews each end **NO FINDINGS**. Generic recursive task-
contract/link/fragment/legacy-ledger lint, checker/runner lint (`15/15` each),
checker/runner libraries (`530/530` and `600/600`), and runner metadata
(`137/137`) pass. Formatting, offline Cargo metadata, warnings-denied all-
target/all-feature Clippy, the full all-target/all-feature workspace suite
including all three long frontend benchmarks, and `git diff --check` pass.

All five CLIs exit zero with 23 known warnings and zero errors each and
reproduce every frozen stdout hash. Protected counts and path hashes reproduce
as specification 64, `.miz` 343, expectation 435, checker production 30,
runner production 90, and Cargo 21; zero protected diff retains all frozen
content hashes. Trace, coverage audit, source TSV, the 836-line ledger, and
their hashes reproduce exactly. The working diff contains only the frozen
seven paths; the four source documents retain the exact `+4/-112` delta.
The final independent read-only quality review ends **NO FINDINGS**, passes
all nine hard gates, applies no score cap, and assigns **100/100**. Its only
residual item is the required exact staging, migration commit, and clean post-
commit replay.

## Handoff

Complete exact staging, commit the seven-path migration, then fresh-inventory
the next checker duplication family. The parent remains `xhigh`; bounded
independent reviews use `high`.
