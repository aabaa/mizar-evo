# Task DOC-258B3M2B2B2C-IMPLEMENTATION-LEDGER-COMPACT: B2C Implementation-Ledger Compaction

> Canonical language: English. Japanese companion:
> [../ja/DOC-258B3M2B2B2C-IMPLEMENTATION-LEDGER-COMPACT.md](../ja/DOC-258B3M2B2B2C-IMPLEMENTATION-LEDGER-COMPACT.md).

## Identity And Status

| Field | Frozen value |
|---|---|
| Task | `DOC-258B3M2B2B2C-IMPLEMENTATION-LEDGER-COMPACT` |
| Status | Migration reviews, final verification, and independent final quality are complete; exact staging and the dedicated commit remain. |
| Purpose | Centralize only the completed B2C implementation checklist duplicated by the paired checker TODOs. |
| Historical owner | [Task 258B3M2B2B2C](./258B3M2B2B2C.md#completion-evidence) |
| Plan indexes | [checker](../../mizar-checker/en/00.crate_plan.md#task-index) and [runner](../../mizar-test/en/00.crate_plan.md#task-index) plans |
| Selection HEAD | `ecbef11a0907f23ec09a888311108a300f4fe569` |
| Repository state | clean `main`, `origin/main...HEAD=0/7`, protected `stash@{0}=f65cf4a13752ec380710814a9ac6392ccb9d75d4` |

Authority is the approved checker-first compaction program, [`AGENTS.md`](../../../../AGENTS.md), the [autonomous migration policy](../../autonomous_crate_development.md#migration-policy), the historical owner, these two completed TODO sections, and retained owners. `design_drift` is the only classification repaired: the TODO pair duplicates one completed ledger. No `spec_gap`, `test_gap`, `source_drift`, `test_expectation_drift`, semantic, API, behavior, trace, or coverage change is introduced. The malformed historical prerequisite spelling `d6076cc758f5974440446104253540e33c99a4c8` remains untouched and is report-only `repo_metadata_conflict`; actual durable object `d6076cc757ce675d1b46a720b4f00805923d3c70` is recorded only for readiness.

## Frozen Sources And Owners

[`DOC-258B3M2B2B2C-IMPLEMENTATION-LEDGER-COMPACT.sources.tsv`](../DOC-258B3M2B2B2C-IMPLEMENTATION-LEDGER-COMPACT.sources.tsv) has two byte-sorted data rows, two comments, and final LF. Its data-row SHA-256 is `0ea3eb0160818e4de4d9adc2953545b38c07c4ebdc812b4cad8b0096f27c81bb`; complete-file SHA-256 is `203476b91b2e8c578a7e8c225a0e7c2456c78fb6c436227263490942dd829673`.

| Source | Lines | SHA-256 | Previous H2 | Next H2 |
|---|---:|---|---|---|
| EN checker TODO `5253-5281` | 29 | `413d750012e520c55a90a995af5426d74851590b3d170ae45422e7e98d762fef` | `## Checker Task 258B3M2B2B2C Frozen-Contract Ledger` | `## Checker Task 258B3M2B2B3P Frozen-Contract Ledger` |
| JA checker TODO `5011-5037` | 27 | `489d3eee583f34404f31f09dc8ee9cc40165fca41e366b2592ef98db2f2a3c3e` | `## Checker Task 258B3M2B2B2C frozen-contract ledger` | `## Checker Task 258B3M2B2B3P frozen-contract ledger` |

The exact old-section-to-owner redirect map is:

- EN heading -> `Completion evidence: [central Task-258B3M2B2B2C historical contract](../../task_contracts/en/258B3M2B2B2C.md#completion-evidence).`
- JA heading -> `Completion evidence: [central Task-258B3M2B2B2C historical contract](../../task_contracts/ja/258B3M2B2B2C.md#completion-evidence)。`

The selection is `2/56`; the selected TODOs and legacy ledger remain
byte-identical during this prerequisite. The existing owner batch uses 18
redirects over nine other EN/JA checker path pairs, so these two TODO paths
are source-disjoint for Task `258B3M2B2B2C`. The later ledger declares exactly
`task_ref<TAB>DOC-258B3M2B2B2C-IMPLEMENTATION-LEDGER-COMPACT<TAB>258B3M2B2B2C`;
the canonical task row and its four historical Task Index records remain
owned by `DOC-258B3M2B2B2C-FINAL-REVIEW-COMPACT`.

The historical owner retains the exact three-checker/five-runner scope,
public structure-witness surface, private B2CP seam, one unnamed
`Structure(0)` witness, Task-252/254/256/base partition, four checker/five
runner tests, focused `4/4`/`5/5`, libraries `390/444`, sibling `12/12`/`21/21`,
sizes/manifests/test-list hashes, no-findings reviews, implementation
`e8373c683448e524cb98edde83fdf8de83a125cd`, unchanged stash, and B3P handoff.
Durable detail remains in the checker [implementation inventory](../../mizar-checker/en/00.crate_plan.md#task-258b3m2b2b2c-implementation-completion-inventory),
[broad verification](../../mizar-checker/en/00.crate_plan.md#task-258b3m2b2b2c-broad-verification-completion),
and [post-commit closure](../../mizar-checker/en/00.crate_plan.md#task-258b3m2b2b2c-post-commit-closure), plus the runner [contract](../../mizar-test/en/00.crate_plan.md#checker-task-258b3m2b2b2c-frozen-runner-contract),
[historical completion](./258B3M2B2B2C.md#completion-evidence),
[broad verification](../../mizar-test/en/00.crate_plan.md#checker-task-258b3m2b2b2c-broad-runner-verification-completion),
[harness](../../mizar-test/en/harness.md#checker-task-258b3m2b2b2c-frozen-runner-harness),
and [boundary](../../mizar-test/en/module_boundary_audit.md#checker-task-258b3m2b2b2c-frozen-runner-boundary) owners.

## Protected Baseline And Migration

Checker production remains `30/186162`, `c89f43f6abebf7ebeb3ac9394ecd8ea3186ad28934c75526d2cc0b85a66ebad5` / `aeb472fb32ba2c3252b65fc9b0ceb81001a1b36a6486834bec113bd2bc4142fb`; runner remains `37/79769`, `1f9e2c9c6589412d832eb92015d913c1b2e0f1309cba9c5c991e08b04d67a73d` / `2b642db1b23a8bb932a434ef7914f696951c998748644999486a107057effdfa`; libraries are `534/604`, raw hashes `542b3ed2ca7f84d1a78603e1ef3e2ee4ac963b50b4f764cdc819f5a4a43b3ad3` / `4ca6de65d417874fea0c9d8491beb41a10ccfc2c188b4a7ddc3971a27db55c68`. Trace is `55b754c8c4d0d293a1c44e2ba4b0090f407bba1d429b461b6cb4d6ddca9ca2b3`; coverage audit is `a31f6fb3bd2b561610630497c58284484d00716dd0b7f210f55bef3bc4bfa6db`; all five frozen CLI hashes remain those in the retained owners.

Protected authority/test sets remain specification `64`, path/content
`d900ba9e43ab094925f36493f830e0c6a2964be2911d5d229014a58842067a25` /
`b30dd5519191a4407399826faf91cc58853d7944df7630734b5ab05de48c9f7b`;
`.miz` `343`, `d94980e167b4b8ac196f91e7694ff044080c6fb4d04c135b3cd5e65b9a019f22` /
`54e6ea1254a0bd963c39026d788711507f353e9c6df3b4f9fd268b2e9f240afb`;
and expectations `435`, `22a5ee257a294e3f2ed4b24bf9ca243d037bbd798f009d0d1e3a176dad8b4fea` /
`b5f0ed1a8d73bbfb78af5cad87a8d426e97269f7f70163750893a9fe1f39d2ea`.

Current task-contract counts `79/79` become `80/80`. The current schema-2 ledger remains 1000 lines, `114e5215e66e2e77912425b1283629ac1afd4269bff60d93a2540aba53988282`, cardinalities `30/44/2/626/296`. The later migration adds exactly eight byte-sorted rows: one batch, one `task_ref`, two redirects over two paths, and four batch indexes only. Its exact batch row is:

```text
batch	DOC-258B3M2B2B2C-IMPLEMENTATION-LEDGER-COMPACT	doc/design/task_contracts/en/DOC-258B3M2B2B2C-IMPLEMENTATION-LEDGER-COMPACT.md	doc/design/task_contracts/ja/DOC-258B3M2B2B2C-IMPLEMENTATION-LEDGER-COMPACT.md	e1a25734745e7343182e0559d6f96f77260a65b7d0dfa0e28cdfed7990df4bf1	1	2	2	4
```

The canonical seven-row hash is `e1a25734745e7343182e0559d6f96f77260a65b7d0dfa0e28cdfed7990df4bf1`; expected ledger is 1008 lines, `5a483ef4358eb8c454542761dfc80007acf8a062577d6672549ab68959e3ada5`, cardinalities `31/44/3/628/300`.

## Scope, Verification, And Handoff

The prerequisite changes exactly nine paths: this EN/JA pair, historical EN/JA pair, source TSV, and one new batch Task Index row in each checker/runner EN/JA plan. It authorizes only these two TODO sections, adds no ledger row, and does not migrate sources. After a separate commit and fresh replay, migration changes exactly five paths: the TODO pair, this EN/JA pair, and `legacy_compactions.tsv`; it replaces the 56 selected lines with language-local redirects to `258B3M2B2B2C.md#completion-evidence`, exact source diff `+2/-54`, with all four anchors byte-identical. It adds no canonical task row or historical Task Index.

No specification, test, source, trace, coverage, Cargo, API, diagnostic, active behavior, or semantics change is permitted; `doc/design/spec_coverage_audit.md` stays unchanged. Prerequisite and migration separately require evidence-equivalence, test-sufficiency/schema, bilingual/boundary, source-documentation, and final-quality reviews ending **NO FINDINGS**, protected replay, and `git diff --check`. Do not claim migration complete. After prerequisite commit and clean replay, perform only the frozen migration; use parent `xhigh`, independent review `high`, and deterministic inventory `medium`.

## Documentation-Prerequisite Evidence

Independent evidence-equivalence, schema/test-sufficiency, and bilingual/
boundary reviews end **NO FINDINGS**. They reproduce the `29/27`-line source
preimages, source TSV hashes, exact anchors and redirects, source-disjoint
`task_ref` ownership, four batch indexes, seven-row expanded hash, prospective
ledger hash/cardinalities, and every retained unique claim. The malformed
historical prerequisite remains a report-only `repo_metadata_conflict`.

Checker and runner lint-policy pass `15/15` each; runner metadata passes
`137/137`; the complete checker and runner libraries pass `534/534` and
`604/604`. `cargo fmt --all --check`, offline Cargo metadata, warnings-denied
all-target/all-feature Clippy, the full workspace `cargo test`, recursive
contract/link/fragment lint, and `git diff --check` pass. All five CLI stdout
hashes reproduce plan `700f4bf503783742cefd8004fa095675b7476d46e9a3a6dd439916d237eb6718`,
parse `a8a7aa639d2ebc65eddc923c7e9369ea5637d50e935f808600f446da1bfbda56`,
declaration `71e83ba0b20d4015e07b3bd2c0c4db2837b6151d1251812caed7954530d53c74`,
type `4b2c7bd5ec3cc56e5672fb351126126230ec84fba9bd2bd9049a516d378fab7f`,
and proof `ccf3d2d4d0a3755e00989d97af369a7c560302f76798d0a185d57ec3891e8450`.

Protected specification, test, expectation, trace, coverage, source, Cargo,
selected TODO, and ledger surfaces are unchanged. Final read-only quality ends
**NO FINDINGS**; all nine hard gates PASS, no score cap applies, and the valid
score is `100/100` (`20/20/15/15/10/10/5/5`). Luna was unavailable, so the
bounded implementation and first-pass reviews used the documented Terra
`high` fallback while the parent retained Sol `xhigh`. Only exact staging, the
dedicated prerequisite commit, clean replay, and then the separately frozen
migration remain.

## Migration Evidence

The documentation prerequisite committed separately as
`f6ee9758f64866420d67951e93d4054c3b01a0eb`. Clean fresh replay reproduced
both frozen preimages, immutable source TSV hashes, the unchanged 1000-line
ledger, protected no-ops, `origin/main...HEAD=0/8`, and the protected stash
before migration.

The selected EN/JA checker TODO sections are now language-local redirects to
`258B3M2B2B2C.md#completion-evidence`. Exact source diff is `+2/-54`; both
forbidden headings and bodies are gone, while all four neighboring H2 anchors
and every unselected TODO section remain.

The ledger adds exactly eight byte-sorted rows: one batch, four batch indexes,
two redirects, and one source-disjoint `task_ref`. It is 1008 lines with
physical SHA-256
`5a483ef4358eb8c454542761dfc80007acf8a062577d6672549ab68959e3ada5`,
reproduces canonical seven-row SHA-256
`e1a25734745e7343182e0559d6f96f77260a65b7d0dfa0e28cdfed7990df4bf1`,
and measures 31 batches, 44 canonical tasks, three task references, 628
redirects, and 300 indexes. There is no second task row or historical Task
Index. The historical contract, source TSV, four plans, trace, coverage audit,
and all protected surfaces remain unchanged.

Independent evidence-equivalence, schema/test-sufficiency, bilingual/
boundary, and source/documentation migration reviews all end **NO FINDINGS**.
They prove the exact whole-H2 splices, retained claims and neighboring anchors,
language-local redirects, schema-2 ownership, hashes/cardinalities, and
protected no-ops.

Checker and runner lint-policy pass `15/15`; runner metadata passes `137/137`;
the checker and runner libraries pass `534/534` and `604/604`. Formatting,
offline Cargo metadata, warnings-denied all-target/all-feature Clippy, the full
workspace `cargo test`, recursive contract/link/fragment/ledger lint, and
`git diff --check` pass. All five CLI stdout hashes reproduce the frozen
plan/parse/declaration/type/proof values above. The 23 known warnings and zero
errors are unchanged. Independent final read-only quality ends **NO FINDINGS**;
all nine hard gates PASS, no score cap applies, and the valid score is
`100/100` (`20/20/15/15/10/10/5/5`). Exact five-path staging, the dedicated
migration commit, and clean post-commit replay remain.
