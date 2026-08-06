# Task DOC-258B3M2B2B1P-B1A-IMPLEMENTATION-LEDGER-COMPACT: B1P/B1A Implementation-Ledger Compaction

> Canonical language: English. Japanese companion:
> [../ja/DOC-258B3M2B2B1P-B1A-IMPLEMENTATION-LEDGER-COMPACT.md](../ja/DOC-258B3M2B2B1P-B1A-IMPLEMENTATION-LEDGER-COMPACT.md).

## Identity And Status

| Field | Frozen value |
|---|---|
| Task | `DOC-258B3M2B2B1P-B1A-IMPLEMENTATION-LEDGER-COMPACT` |
| Status | Migration implemented with the exact frozen redirects and ledger rows; all reviews, all nine hard gates, and full verification are complete with no findings and a valid `100/100`. Exact staging, commit, and clean replay remain. |
| Purpose | Centralize only the completed Task-258B3M2B2B1P and Task-258B3M2B2B1A checker implementation ledgers while retaining both frozen ledgers and every later task. |
| Historical owners | [Task 258B3M2B2B1P](./258B3M2B2B1P.md#completion-evidence) and [Task 258B3M2B2B1A](./258B3M2B2B1A.md#completion-evidence) |
| Plan indexes | [checker plan](../../mizar-checker/en/00.crate_plan.md#task-index) and [runner plan](../../mizar-test/en/00.crate_plan.md#task-index) |
| Selection HEAD | `e9465ba0ffabf78544cc9ad5663c2d999b6898bf` |
| Repository state | clean `main`, initial `origin/main...HEAD=0/1`, protected `stash@{0}=f65cf4a13752ec380710814a9ac6392ccb9d75d4` |
| Dependencies | B1P is the prerequisite to the exact B1A consumer. B1P documentation `b196a9ce95c5f0b62fe6f2ae64cee4e3fe9ea704` and implementation `5875690175554312b7114ccc9a8c6d21ea57df90`, B1A documentation `2fb6a6752352b6b5925b75dc6d175f3c1d918818` and implementation `0b10b21f36999693d92999ccd98afe3e0c373e1b`, successor B1B1P `406dd2f21d3c82a915899b87b9ab595b0c1754ee`, prior implementation-ledger compaction, and generic schema-2 ledger support are ancestors of selection HEAD. |

## Authority, Consumers, And Classification

Authority is the user-approved checker-first compaction program,
[`AGENTS.md`](../../../../AGENTS.md), the
[autonomous migration policy](../../autonomous_crate_development.md#migration-policy),
canonical Chapters 4 §4.4.3, 13 §§13.2, 13.8.3, and 13.9, 15 §§15.4.4 and
15.11.5, and 16 §§16.3 and 16.7.3, and the retained imported `1++2` fixture.
The historical records and retained component owners are derived; source
behavior is not normative. The four selected complete sections form one
coherent duplication family because both tasks retain the same 143-byte source
boundary and B1P precedes the exact B1A consumer. The generic lint-policy
consumer owns recursive contracts, links, fragments, plan indexes, section
anchors, ledger counts, ordering, and hash replay; readers consume only the
language-local completion-evidence redirects.

| Class | Decision |
|---|---|
| `design_drift` | Checker EN/JA TODOs repeat the completed B1P and B1A implementation ledgers outside central historical owners; those historical contracts and their Task Index rows are absent at selection. |
| `test_gap` | None. Existing generic schema-2 lint covers two canonical task rows, four exact redirects, twelve indexes, paired links/fragments, hashes, counts, and anchors. |
| `boundary_violation` | Avoided by selecting exactly one complete, flat implementation-ledger H2 per task and language. The B1P frozen lower-prerequisite ledger, the B1A frozen-contract ledger, successor B1B1P, and every later task remain. |
| `spec_gap` / `source_drift` | None introduced or repaired. No semantic decision is made by this derived documentation migration. |
| `source_undocumented_behavior` / `test_expectation_drift` | None inferred or changed. |
| `repo_metadata_conflict` | None at selection. The initial one-commit-behind origin relation is report-only; no fetch, push, reset, or stash mutation is authorized. |

## Frozen Sources, Fingerprints, And Anchors

[`DOC-258B3M2B2B1P-B1A-IMPLEMENTATION-LEDGER-COMPACT.sources.tsv`](../DOC-258B3M2B2B1P-B1A-IMPLEMENTATION-LEDGER-COMPACT.sources.tsv)
is immutable prerequisite evidence with four byte-sorted data rows, two
comments, and final LF. Data-row SHA-256 is
`47c68c92330682588b348c701d36e7bb56bc261323ff4185746a0ce61267e658`;
complete-file SHA-256 is
`e76eed6509fd9d33bbdeb79a23c7be1537576dfa25324d013e22c8a3d3a26062`.
The source-locally unique, unlinked H2 sections have no nested headings,
tables, fences, or redirects and total 70 physical lines.

| Task/source | Heading and lines | Section SHA-256 | Previous H2 | Next H2 |
|---|---|---|---|---|
| B1P EN checker TODO | `## Checker Task 258B3M2B2B1P Implementation Ledger`, `4934-4947` (14) | `f80c5d175f2db9055efe90966988ddd030ae42e7ef585155c60c5a303f921000` | `## Checker Task 258B3M2B2B1P Frozen Lower-Prerequisite Ledger` | `## Checker Task 258B3M2B2B1A Frozen-Contract Ledger` |
| B1P JA checker TODO | `## Checker Task 258B3M2B2B1P implementation ledger`, `4697-4710` (14) | `b8e6da3278ef45d49f4319615325eb25c30efa5b6ae826e02acd272df7bb5745` | `## Checker Task 258B3M2B2B1P frozen lower-prerequisite ledger` | `## Checker Task 258B3M2B2B1A frozen-contract ledger` |
| B1A EN checker TODO | `## Checker Task 258B3M2B2B1A Implementation Ledger`, `4968-4988` (21) | `8cba2e758b6851f948e2c5b519bf05488439ad58dc8750be28cc4199c6f3c1bc` | `## Checker Task 258B3M2B2B1A Frozen-Contract Ledger` | `## Checker Task 258B3M2B2B1B1P Frozen-Prerequisite Ledger` |
| B1A JA checker TODO | `## Checker Task 258B3M2B2B1A implementation ledger`, `4729-4749` (21) | `900cd7cb4fbec077b915a810f6a6875c99c9e2b85616cd08c67af0cdc79075e4` | `## Checker Task 258B3M2B2B1A frozen-contract ledger` | `## Checker Task 258B3M2B2B1B1P frozen-prerequisite ledger` |

## Affected-Artifact Index And Boundaries

The documentation prerequisite changes exactly 11 paths:

| Artifact group | Exact paths and delta |
|---|---|
| Historical owners | New `doc/design/task_contracts/{en,ja}/258B3M2B2B1P.md` and `doc/design/task_contracts/{en,ja}/258B3M2B2B1A.md` (two EN/JA pairs, four files) |
| Batch owner | This new EN/JA pair |
| Source inventory | New immutable `doc/design/task_contracts/DOC-258B3M2B2B1P-B1A-IMPLEMENTATION-LEDGER-COMPACT.sources.tsv` |
| Plan consumers | `doc/design/mizar-checker/{en,ja}/00.crate_plan.md` and `doc/design/mizar-test/{en,ja}/00.crate_plan.md`, with exactly three language-local Task Index rows in each file: B1P, B1A, and this batch |

The selected TODO sections and `legacy_compactions.tsv` remain unchanged in
the prerequisite. Task-contract Markdown pairs move `70/70 -> 73/73`. The
manifest must remain absent for this batch until the prerequisite is committed,
cleanly replayed, and the exact migration is correct.

After that clean replay, migration changes exactly five paths: the EN/JA
checker TODOs, this EN/JA pair for status/evidence only, and
`doc/design/task_contracts/legacy_compactions.tsv`. Only the four selected H2
sections become four language-local completion-evidence redirects. The exact
source diff is `+4/-66`; every recorded neighboring anchor and every unselected
section remains byte-identical.

Specifications, `.miz`, the retained fixture, expectations, sidecars, trace
metadata, coverage audit, production, Cargo, public APIs, diagnostics, and
active behavior are forbidden. Active routes, language/proof semantics,
existing goals, diagnostics, all unselected documentation, both frozen
ledgers, successor B1B1P, and all later tasks remain unchanged. This contract
makes no new claim about a goal, discharge, fact, or verification condition.
No coverage-audit edit is needed because mapping, status, deferred reason,
trace linkage, and coverage credit do not change;
`doc/design/spec_coverage_audit.md` remains unchanged.

## Protected Baseline And Expected Migration

| Surface | Paths | Path SHA-256 | Content SHA-256 |
|---|---:|---|---|
| specification | 64 | `d900ba9e43ab094925f36493f830e0c6a2964be2911d5d229014a58842067a25` | `b30dd5519191a4407399826faf91cc58853d7944df7630734b5ab05de48c9f7b` |
| `.miz` | 343 | `d94980e167b4b8ac196f91e7694ff044080c6fb4d04c135b3cd5e65b9a019f22` | `54e6ea1254a0bd963c39026d788711507f353e9c6df3b4f9fd268b2e9f240afb` |
| expectation | 435 | `22a5ee257a294e3f2ed4b24bf9ca243d037bbd798f009d0d1e3a176dad8b4fea` | `b5f0ed1a8d73bbfb78af5cad87a8d426e97269f7f70163750893a9fe1f39d2ea` |
| checker production | 30 | `a41370d7150a587369cea5f7a67b60417dd1372592f55c0d65bec369eb39fdc6` | `05fd5e0eaed4361b824693941e9056a552c476f050915ea5052a85c8c7174dfd` |
| runner production | 90 | `05245a54160dfce17336b476b07885eb6d5afe138c4780a6a6a7b47043e7248c` | `210f294aebfe22c12324ef9919ac68147f8025f0da8de166403dada87bac5eae` |
| Cargo | 21 | `d93f2816b760d8ba45430d6d8570e598864aa7b20b19a001f45171d36fd3a030` | `146e9b4024d2c344b2eca9c6f5ca6d6a80a3de427e382953a2280bc63cb3ecca` |

Trace SHA-256 remains
`55b754c8c4d0d293a1c44e2ba4b0090f407bba1d429b461b6cb4d6ddca9ca2b3`;
coverage audit remains
`2aa808aa033c04fa46ea76f65cfadcf0d8f0e1e53fba6d70edab223a7481685f`.
Ledger baseline is 942 lines with physical SHA-256
`e5a804a4c5b452610e0024b1de3186445b5179b43d138927f7e3a079bb19af41`,
27 batches, 39 canonical tasks, two task references, 608 redirects, and 264
indexes.

The five CLI hashes remain plan
`700f4bf503783742cefd8004fa095675b7476d46e9a3a6dd439916d237eb6718`,
parse `a8a7aa639d2ebc65eddc923c7e9369ea5637d50e935f808600f446da1bfbda56`,
declaration `71e83ba0b20d4015e07b3bd2c0c4db2837b6151d1251812caed7954530d53c74`,
type `4b2c7bd5ec3cc56e5672fb351126126230ec84fba9bd2bd9049a516d378fab7f`,
and proof `ccf3d2d4d0a3755e00989d97af369a7c560302f76798d0a185d57ec3891e8450`.

Ledger impact is 19 lines, `942 -> 961`: one batch, two canonical tasks,
twelve indexes, and four redirects; no `task_ref` is added. Canonical 18-row
expanded-inventory SHA-256 is
`0b8534ed721345098b9af38a4de80460da6c3c145e0bb62679828b3370bee322`;
expected physical ledger SHA-256 is
`d421b3115c780370bb0129463df908f7beb94ad687c679467201d39324fca9c3`.
Final cardinalities are 28 batches, 41 canonical tasks, two task references,
612 redirects, and 276 indexes.

## Reviews, Verification, Audit Impact, And Exit

The prerequisite and migration separately require independent evidence-
equivalence, schema/test-sufficiency, bilingual/boundary, full-implementation,
source/documentation-consistency, and final-quality reviews as applicable,
with every finding fixed and the relevant review repeated to **NO FINDINGS**.
All nine autonomous hard gates must PASS without a score cap, and independent
final quality must be at least `90/100`. No new fixture, expectation, sidecar,
trace row, semantic test, production route, or batch-specific Rust branch is
authorized; the existing generic schema-2 lint is the only new-contract
consumer.

Verification includes source-TSV/commit/blame/anchor replay; recursive
contract/link/fragment/ledger lint; exact three-row-per-plan and `73/73`
contract-pair checks; checker/runner libraries and metadata; formatting;
offline metadata; warnings-denied all-target/all-feature Clippy; full workspace
tests; all five CLIs; protected count/hash replay; ledger order/hash/
cardinality; `git diff --check`; exact cached review; and unstaged/untracked
inspection. No push, fetch, reset, or stash mutation is authorized.

The prerequisite exits with exact 11-path scope, unchanged selected sections
and ledger, synchronized EN/JA, all reviews and gates complete, one dedicated
commit, and clean replay. Only then may migration add the exact four redirects
and 19 ledger rows. Migration exits separately with exact five-path scope,
complete evidence preservation, generic schema replay, all reviews and gates,
one dedicated commit, and clean replay before fresh selection of another
checker duplication family.

## Next Handoff

After the prerequisite commit, freshly replay this contract and implement only
this task's four language-local redirects plus 19 byte-sorted ledger rows. Do
not compact either frozen ledger, successor B1B1P, any later task, runner or
owner-local section, or any other documentation family.

## Documentation-Prerequisite Evidence

Independent evidence/specification, schema/test-sufficiency, and bilingual/
boundary/source-documentation reviews end **NO FINDINGS**. Reviewers
independently reproduce the exact historical ancestry, all four preimages and
anchors, immutable TSV hashes, prospective 18/19-row ledger hashes and
cardinalities, `+4/-66` migration, 12 language-local Task Index rows, `70/70
-> 73/73` contract pairs, every unique completion fact and deferral, and the
exact 11-path prerequisite boundary without authority, schema, or semantic
expansion.

Generic lint passes `15/15`; checker `530/530`, runner `600/600`, and metadata
`137/137` tests pass. `cargo fmt --all --check`, offline Cargo metadata,
warnings-denied all-target/all-feature Clippy, and the full offline workspace
suite pass. All five CLIs exit zero with the unchanged 23 warnings and zero
errors and reproduce the frozen plan/parse/declaration/type/proof hashes.

Protected path counts and NUL-delimited path hashes reproduce exactly as
specification `64`, `.miz` `343`, expectation `435`, checker production `30`,
runner production `90`, and Cargo `21`; zero protected diff preserves every
frozen content hash. Trace, coverage audit, selected TODOs, unchanged
942-line ledger, and immutable source TSV reproduce their frozen hashes.
Task contracts measure `73/73`; `git diff --check` passes.

Repository inventory remains selection HEAD on `main` with the exact
task-only 11-path worktree, `origin/main...HEAD=0/1`, and protected
`stash@{0}=f65cf4a13752ec380710814a9ac6392ccb9d75d4`. No push, fetch, reset, or
stash mutation occurred. Independent final read-only quality ends **NO
FINDINGS**. All nine hard gates PASS, no score cap applies, and the valid
score is **100/100** (`20/20/15/15/10/10/5/5`). Exact staging, the dedicated
prerequisite commit, and clean post-commit replay remain.

## Migration Evidence

The documentation prerequisite committed separately as
`f8b4c8e0fc73c04cd4136a24c62dfdc3ed8c30df`. Clean fresh replay reproduced all
four frozen preimages, source TSV hashes, unchanged 942-line ledger, protected
no-ops, `73/73` contracts, `origin/main...HEAD=0/2`, and the protected stash
before migration.

The selected Task-258B3M2B2B1P and Task-258B3M2B2B1A implementation-ledger
sections are now four language-local redirects to their historical completion
evidence. Exact source diff is `+4/-66`; all four forbidden implementation
headings and bodies are gone. Both EN/JA frozen ledgers, successor B1B1P,
every recorded neighboring anchor, and every unselected TODO section remain.

The ledger adds exactly 19 byte-sorted rows: one batch, two canonical tasks,
twelve indexes, and four redirects. It is 961 lines with physical SHA-256
`d421b3115c780370bb0129463df908f7beb94ad687c679467201d39324fca9c3`,
reproduces canonical 18-row SHA-256
`0b8534ed721345098b9af38a4de80460da6c3c145e0bb62679828b3370bee322`,
and measures 28 batches, 41 canonical tasks, two task references, 612
redirects, and 276 indexes. Historical contracts, source TSV, four plans,
protected surfaces, trace, and coverage audit remain unchanged. Generic lint
passes `15/15` and `git diff --check` passes.

Independent migration evidence-equivalence, schema/test-sufficiency, and
bilingual/boundary/source-documentation reviews end **NO FINDINGS**. They
reproduce every frozen preimage and unique completion fact, the exact
`+4/-66` redirect delta, all 19 ledger rows, language-local links and
fragments, retained exclusions, ordering, hashes, and cardinalities without
schema or semantic expansion.

Generic lint passes `15/15`; checker `530/530`, runner `600/600`, and metadata
`137/137` tests pass. `cargo fmt --all --check`, offline Cargo metadata,
warnings-denied all-target/all-feature Clippy, and the full offline workspace
suite pass. All five CLIs exit zero with the unchanged 23 warnings and zero
errors and reproduce the frozen plan/parse/declaration/type/proof hashes.

Protected path counts and NUL-delimited path hashes reproduce exactly as
specification `64`, `.miz` `343`, expectation `435`, checker production `30`,
runner production `90`, and Cargo `21`; zero protected diff preserves every
frozen content hash. Trace, coverage audit, immutable source TSV, historical
contracts, and four plans remain unchanged; task contracts measure `73/73`.
`git diff --check` passes. Repository inventory remains prerequisite HEAD on
`main` with the exact task-only five-path worktree,
`origin/main...HEAD=0/2`, and protected
`stash@{0}=f65cf4a13752ec380710814a9ac6392ccb9d75d4`. No push, fetch, reset, or
stash mutation occurred.

Independent final read-only quality ends **NO FINDINGS**. All nine hard gates
PASS, no score cap applies, and the valid score is **100/100**
(`20/20/15/15/10/10/5/5`). Exact five-path staging, the dedicated migration
commit, and clean post-commit replay remain.
