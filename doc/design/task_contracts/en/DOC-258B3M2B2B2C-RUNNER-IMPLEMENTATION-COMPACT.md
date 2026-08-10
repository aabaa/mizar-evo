# Task DOC-258B3M2B2B2C-RUNNER-IMPLEMENTATION-COMPACT: B2C Runner Implementation-Evidence Compaction

> Canonical language: English. Japanese companion:
> [../ja/DOC-258B3M2B2B2C-RUNNER-IMPLEMENTATION-COMPACT.md](../ja/DOC-258B3M2B2B2C-RUNNER-IMPLEMENTATION-COMPACT.md).

## Identity And Status

| Field | Frozen value |
|---|---|
| Task | `DOC-258B3M2B2B2C-RUNNER-IMPLEMENTATION-COMPACT` |
| Status | Documentation prerequisite accepted; exact staging and its dedicated commit remain. Migration has not started. |
| Purpose | Centralize only the completed B2C runner implementation evidence duplicated by five paired runner documents. |
| Historical owner | [Task 258B3M2B2B2C](./258B3M2B2B2C.md#completion-evidence) |
| Plan indexes | [checker](../../mizar-checker/en/00.crate_plan.md#task-index) and [runner](../../mizar-test/en/00.crate_plan.md#task-index) plans |
| Selection HEAD | `b91ca9cfe9eb4789045eda271db8160c226e3133` |
| Repository state | clean `main`, `origin/main...HEAD=0/9`, protected `stash@{0}=f65cf4a13752ec380710814a9ac6392ccb9d75d4` |
| Dependencies | Actual B2C prerequisite `d6076cc757ce675d1b46a720b4f00805923d3c70`, implementation `e8373c683448e524cb98edde83fdf8de83a125cd`, final-review migration `9b356722`, checker-ledger prerequisite `f6ee9758`, and checker-ledger migration `b91ca9cf` are ancestors. |

Authority is the user-authorized checker-first compaction program,
[`AGENTS.md`](../../../../AGENTS.md), the
[autonomous migration policy](../../autonomous_crate_development.md#migration-policy),
the historical owner, the ten selected completed sections, and their retained
owners. `design_drift` is the only repaired classification. No `spec_gap`,
`test_gap`, `source_drift`, `source_undocumented_behavior`,
`test_expectation_drift`, semantic, API, diagnostic, trace, or coverage change
is introduced. The malformed historical prerequisite spelling
`d6076cc758f5974440446104253540e33c99a4c8` remains untouched and report-only
`repo_metadata_conflict`.

## Frozen Source-To-Owner Map

[`DOC-258B3M2B2B2C-RUNNER-IMPLEMENTATION-COMPACT.sources.tsv`](../DOC-258B3M2B2B2C-RUNNER-IMPLEMENTATION-COMPACT.sources.tsv)
contains ten byte-sorted data rows, two comments, and final LF. Data-row
SHA-256 is
`6c7ea8d6053f854ed1a8f7d00ed13fca7cfae38fdb33bb483e7a08fc1147a3ac`;
complete-file SHA-256 is
`8df8f7c3f4f5cd628a56fd70123152f063956dc5560b9d998b6d53f04fa7408a`.
Every hash includes the trailing blank separator.

| Source | Lines | SHA-256 | Previous H2 | Next H2 |
|---|---:|---|---|---|
| `doc/design/mizar-test/en/00.crate_plan.md:5546-5587`, `## Checker Task 258B3M2B2B2C Runner Implementation Completion` | 42 | `462e18bb846a2c6bc5bc08571204390050689d69aacab1881b53824b1381f2c7` | `## Checker Task 258B3M2B2B2C Frozen Runner Contract` | `## Checker Task 258B3M2B2B2C Broad Runner Verification Completion` |
| `doc/design/mizar-test/en/bilingual_sync_audit.md:884-900`, `## Checker Task 258B3M2B2B2C Implementation Synchronization` | 17 | `1ce237a32b078dda52b9a23431b8d11176711b2c5c651236282b372e0ddbde2a` | `## Checker Task 258B3M2B2B2C Frozen-Contract Synchronization` | `## Checker Task 258B3M2B2B2C Broad-Verification Synchronization` |
| `doc/design/mizar-test/en/harness.md:4209-4228`, `## Checker Task 258B3M2B2B2C Implemented Runner Harness` | 20 | `408ae244704110669bdc0e6e609f1e7ea2550e8f365d679c6328f5c87ddc3f36` | `## Checker Task 258B3M2B2B2C Frozen Runner Harness` | `## Checker Task 258B3M2B2B2C Broad Harness Verification` |
| `doc/design/mizar-test/en/module_boundary_audit.md:12075-12093`, `## Checker Task 258B3M2B2B2C Implemented Runner Boundary` | 19 | `f8a319e12088364dcf2aa4fcb10c846e3c1006340614d0cac7b0e3997c3f7692` | `## Checker Task 258B3M2B2B2C Frozen Runner Boundary` | `## Checker Task 258B3M2B2B2C Broad Runner-Boundary Verification` |
| `doc/design/mizar-test/en/todo.md:2496-2523`, `## Checker Task 258B3M2B2B2C Runner Implementation Ledger` | 28 | `f3108251e4ea1a999d978dac99ca2355c99b24398e230d2a776a8a68141763f7` | `## Checker Task 258B3M2B2B2C Runner Frozen-Contract Ledger` | `## Checker Task 258B3M2B2B3P Runner Frozen-Contract Ledger` |
| `doc/design/mizar-test/ja/00.crate_plan.md:5267-5307`, `## Checker Task 258B3M2B2B2C runner implementation completion` | 41 | `a6a6e53e89bb548fdf4390683329c7cb0386b10bb5c73aac7917aca1c2a7972c` | `## Checker Task 258B3M2B2B2C frozen runner contract` | `## Checker Task 258B3M2B2B2C broad runner verification completion` |
| `doc/design/mizar-test/ja/bilingual_sync_audit.md:846-861`, `## Checker Task 258B3M2B2B2C implementation synchronization` | 16 | `23a0c2554384fe8f1bf9f79a4c0d3e53a061279aacf45a68a0d3051fe31b24e4` | `## Checker Task 258B3M2B2B2C frozen-contract synchronization` | `## Checker Task 258B3M2B2B2C broad-verification synchronization` |
| `doc/design/mizar-test/ja/harness.md:3956-3974`, `## Checker Task 258B3M2B2B2C implemented runner harness` | 19 | `beac8bf8a8d5ed8ec5c84689c461cf214e3bf2b5fbd200699fb1170c38ea3724` | `## Checker Task 258B3M2B2B2C frozen runner harness` | `## Checker Task 258B3M2B2B2C broad harness verification` |
| `doc/design/mizar-test/ja/module_boundary_audit.md:10839-10857`, `## Checker Task 258B3M2B2B2C implemented runner boundary` | 19 | `0b195c0be14763b48a7f4f4bd3d3aa69fd375a5f1e2bde37b729886521f2a68d` | `## Checker Task 258B3M2B2B2C frozen runner boundary` | `## Checker Task 258B3M2B2B2C broad runner-boundary verification` |
| `doc/design/mizar-test/ja/todo.md:2296-2320`, `## Checker Task 258B3M2B2B2C runner implementation ledger` | 25 | `b2244ba1bcf94544309c6c84e5b55425316770d83f8eb63460f94f8d2dac4c98` | `## Checker Task 258B3M2B2B2C runner frozen-contract ledger` | `## Checker Task 258B3M2B2B3P runner frozen-contract ledger` |

The exact selection is EN `5/126`, JA `5/120`, total `10/246`. Every EN
section maps to:

```text
Completion evidence: [central Task-258B3M2B2B2C historical contract](../../task_contracts/en/258B3M2B2B2C.md#completion-evidence).
```

Every JA section maps to the language-local equivalent ending in `。` and
linking `../../task_contracts/ja/258B3M2B2B2C.md#completion-evidence`. The
Task Index insertion shifts each selected plan section by one physical line;
identity is frozen by heading, anchors, count, and hash rather than line number.

## Ownership, Scope, And Prohibitions

The historical owner preserves the exact eight-file transaction, five runner
files, unchanged private B2CP seams, exact 181-byte/86-node source, 180-byte
malformed profile, five valid exclusions, Task-48/252/254/256/base tables,
single unnamed witness-to-`Structure(0)` edge, five runner/four checker tests,
libraries `390/444`, focused `4/4`/`5/5`, time-local sizes and hashes,
no-findings reviews, implementation commit, unchanged stash, and B3P handoff.
Durable details remain in the runner-plan frozen, broad, final, and post-commit
sections; runner bilingual frozen, broad, final, and closure sections; harness
frozen, broad, and final sections; boundary frozen, broad, and final sections;
the runner frozen/B3P TODO ledgers; and the checker owners linked by the
historical contract.

The final-review batch owns the sole canonical `task` row and historical Task
Index. The checker-ledger batch owns a `task_ref` over two checker TODO paths.
This batch's ten runner paths are disjoint from both earlier batches and from
each other, so schema v2 permits exactly:

```text
task_ref	DOC-258B3M2B2B2C-RUNNER-IMPLEMENTATION-COMPACT	258B3M2B2B2C
```

No second `task` row or historical-task index is allowed. Every remaining B2C
checker family collides with an already registered task/source path and stays
pending a separate occurrence-safe schema prerequisite.

The documentation prerequisite changes exactly nine paths: this EN/JA pair,
the historical EN/JA pair, source TSV, and one batch-only Task Index row in
each checker/runner EN/JA plan. Selected sections and the ledger remain
byte-identical; task-contract counts move `80/80 -> 81/81`.

After its dedicated commit and clean replay, migration changes exactly
thirteen paths: the ten selected runner documents, this EN/JA pair, and
`legacy_compactions.tsv`. The 246 selected lines become ten redirect-plus-
separator records, exact selected-source diff `+10/-236`. Historical
contracts, source TSV, checker plans, protected surfaces, trace, and coverage
audit remain unchanged; runner-plan batch-index rows remain byte-identical.

No runner frozen, broad, final, post-commit, B3P successor, checker, or
unselected section may change. Specification, `.miz`, fixture, sidecar,
expectation, source, Cargo, public API, diagnostic, active route/result,
semantic/proof/goal/IR behavior, trace status/tests, and coverage credit are
forbidden. `doc/design/spec_coverage_audit.md` remains unchanged because
mapping, status, credit, rationale, and follow-up ownership do not change.

## Protected Baseline And Expected Ledger

Protected sets remain specification `64`, path/content
`d900ba9e43ab094925f36493f830e0c6a2964be2911d5d229014a58842067a25` /
`b30dd5519191a4407399826faf91cc58853d7944df7630734b5ab05de48c9f7b`;
`.miz` `343`, `d94980e167b4b8ac196f91e7694ff044080c6fb4d04c135b3cd5e65b9a019f22` /
`54e6ea1254a0bd963c39026d788711507f353e9c6df3b4f9fd268b2e9f240afb`;
expectations `435`, `22a5ee257a294e3f2ed4b24bf9ca243d037bbd798f009d0d1e3a176dad8b4fea` /
`b5f0ed1a8d73bbfb78af5cad87a8d426e97269f7f70163750893a9fe1f39d2ea`;
and Cargo `21`, `d93f2816b760d8ba45430d6d8570e598864aa7b20b19a001f45171d36fd3a030` /
`146e9b4024d2c344b2eca9c6f5ca6d6a80a3de427e382953a2280bc63cb3ecca`.

Checker production remains `30/186162`, path/content
`c89f43f6abebf7ebeb3ac9394ecd8ea3186ad28934c75526d2cc0b85a66ebad5` /
`aeb472fb32ba2c3252b65fc9b0ceb81001a1b36a6486834bec113bd2bc4142fb`;
runner production remains `37/79769`,
`1f9e2c9c6589412d832eb92015d913c1b2e0f1309cba9c5c991e08b04d67a73d` /
`2b642db1b23a8bb932a434ef7914f696951c998748644999486a107057effdfa`.
Libraries remain `534/604`, raw hashes
`542b3ed2ca7f84d1a78603e1ef3e2ee4ac963b50b4f764cdc819f5a4a43b3ad3` /
`4ca6de65d417874fea0c9d8491beb41a10ccfc2c188b4a7ddc3971a27db55c68`.
Corpus/requirements are `428/395`, pass/fail `235/193`, stages
`101/7/205/1`, type `259=247+12`, warnings/errors `23/0`. Trace SHA-256 is
`55b754c8c4d0d293a1c44e2ba4b0090f407bba1d429b461b6cb4d6ddca9ca2b3`;
coverage audit is
`a31f6fb3bd2b561610630497c58284484d00716dd0b7f210f55bef3bc4bfa6db`.
CLI stdout hashes remain plan
`700f4bf503783742cefd8004fa095675b7476d46e9a3a6dd439916d237eb6718`,
parse `a8a7aa639d2ebc65eddc923c7e9369ea5637d50e935f808600f446da1bfbda56`,
declaration
`71e83ba0b20d4015e07b3bd2c0c4db2837b6151d1251812caed7954530d53c74`,
type `4b2c7bd5ec3cc56e5672fb351126126230ec84fba9bd2bd9049a516d378fab7f`,
and proof
`ccf3d2d4d0a3755e00989d97af369a7c560302f76798d0a185d57ec3891e8450`.

Current ledger is 1008 lines, SHA-256
`5a483ef4358eb8c454542761dfc80007acf8a062577d6672549ab68959e3ada5`,
cardinalities `31/44/3/628/300`. Migration adds exactly sixteen byte-sorted
rows: one batch, four batch indexes, ten redirects, and one `task_ref`.
Canonical fifteen-row expanded-inventory SHA-256 is
`0431940e513a7f54e468827a0135ce8c9bf00c603af7ae79599e5fba303efe87`.
The exact batch row is:

```text
batch	DOC-258B3M2B2B2C-RUNNER-IMPLEMENTATION-COMPACT	doc/design/task_contracts/en/DOC-258B3M2B2B2C-RUNNER-IMPLEMENTATION-COMPACT.md	doc/design/task_contracts/ja/DOC-258B3M2B2B2C-RUNNER-IMPLEMENTATION-COMPACT.md	0431940e513a7f54e468827a0135ce8c9bf00c603af7ae79599e5fba303efe87	1	10	10	4
```

Expected ledger is 1024 lines, SHA-256
`2a66d200a1976861600bcf7686388faa3efb19b2b42a43c756c9e689d7f27359`,
cardinalities `32/44/4/638/304`.

## Reviews, Verification, And Handoff

Prerequisite and migration separately require evidence-equivalence,
schema/test-sufficiency, bilingual/boundary, source/documentation, and final
read-only quality reviews ending **NO FINDINGS**. All nine hard gates must
PASS without a score cap at no less than `90/100`.

Verification includes preimage/anchor/TSV replay; generic recursive contract,
link, fragment, index, and ledger lint; checker/runner libraries, lint, and
metadata; formatting; offline Cargo metadata; warnings-denied Clippy; full
workspace tests; five CLI hashes; protected count/hash/status replay;
`git diff --check`; exact cached review; and unstaged inspection. No push,
fetch, reset, amend, or stash mutation is allowed.

Commit this exact prerequisite and clean-replay it before applying only the
frozen thirteen-path migration. Parent reasoning remains Sol `xhigh`; Luna is
unavailable. Terra `high` supplied the bounded inventory and first-pass review
route, while the parent integrated the exact frozen prerequisite after the
bounded worker produced no repository write.

## Documentation-Prerequisite Evidence

At base `b91ca9cfe9eb4789045eda271db8160c226e3133`, the prerequisite diff is
exactly the frozen nine documentation paths: four one-line batch Task Index
additions, the paired historical-owner extension, the paired batch contract,
and the source TSV. The ten selected runner sections and the 1008-line ledger
remain byte-identical. Specification, `.miz`, expectations, trace, coverage
audit, Rust, Cargo, public API, active behavior, diagnostics, semantics, test
intent, and coverage credit have no diff. Task-contract counts are `81/81`.

Independent schema/test-sufficiency, evidence-equivalence,
bilingual/boundary, and source/documentation reviews ended **NO FINDINGS**
after one Medium `design_drift` finding was repaired by freezing the exact
workspace-relative paths, current replay ranges, raw selected H2s, and raw
neighboring H2 anchors in both contracts. Recursive contract/link/fragment
lint passes. The malformed historical prerequisite remains untouched as
report-only `repo_metadata_conflict`.

All ten preimages replay at EN `5/126`, JA `5/120`, total `10/246`; source TSV
data/file hashes are `6c7ea8d6053f854ed1a8f7d00ed13fca7cfae38fdb33bb483e7a08fc1147a3ac`
/ `8df8f7c3f4f5cd628a56fd70123152f063956dc5560b9d998b6d53f04fa7408a`.
The unchanged ledger is 1008 lines with SHA-256
`5a483ef4358eb8c454542761dfc80007acf8a062577d6672549ab68959e3ada5`;
the expanded-inventory and prospective-ledger hashes remain
`0431940e513a7f54e468827a0135ce8c9bf00c603af7ae79599e5fba303efe87`
and `2a66d200a1976861600bcf7686388faa3efb19b2b42a43c756c9e689d7f27359`.
All protected counts and hashes in the preceding section reproduce with zero
delta. Checker/runner production replay at `30/186162` and `37/79769`; library
lists replay at `534/604` with their frozen raw hashes.

Verification passes checker/runner libraries `534/534` and `604/604`, both
lint-policy suites `15/15`, metadata `137/137`, `cargo fmt --all --check`,
offline Cargo metadata, warnings-denied all-target/all-feature Clippy, full
workspace `cargo test`, and `git diff --check`. The plan, parse, declaration,
type, and proof CLI stdout hashes reproduce exactly as frozen. Known metadata
warnings remain the unchanged `23/0` warning/error baseline and do not alter
exit status or output hashes.

Final read-only quality review ended **NO FINDINGS**. All nine hard gates pass,
no score cap applies, and the accepted score is `100/100`
(`20/20/15/15/10/10/5/5`). Exact task-only staging, cached review, dedicated
commit, and clean post-commit replay remain before migration may start.
