# Task DOC-258B3N-IMPLEMENTATION-LEDGER-COMPACT: B3N Implementation-Ledger Compaction

> Canonical language: English. Japanese companion:
> [../ja/DOC-258B3N-IMPLEMENTATION-LEDGER-COMPACT.md](../ja/DOC-258B3N-IMPLEMENTATION-LEDGER-COMPACT.md).

## Identity And Status

| Field | Frozen value |
|---|---|
| Task | `DOC-258B3N-IMPLEMENTATION-LEDGER-COMPACT` |
| Status | Migration reviews, final verification, and independent final quality complete; exact staging, the dedicated migration commit, and clean post-commit replay remain. |
| Purpose | Centralize the EN/JA checker TODO implementation checklist for historical Task 258B3N without changing a durable checker or runner owner. |
| Historical owner | [Task 258B3N](./258B3N.md#completion-evidence) |
| Plan indexes | [checker plan](../../mizar-checker/en/00.crate_plan.md#task-index) and [runner plan](../../mizar-test/en/00.crate_plan.md#task-index) |
| Selection HEAD | `440d27ae6e42f0aef6a58578a643ec5461763af3` |
| Repository state | clean `main`, `origin/main...HEAD=0/4`, protected `stash@{0}=f65cf4a13752ec380710814a9ac6392ccb9d75d4` |
| Dependency | Task-258B3N documentation and implementation commits and generic schema-2 ledger support are ancestors of selection HEAD. |

## Authority And Classification

Authority is the user-approved checker-first compaction program,
[`AGENTS.md`](../../../../AGENTS.md), the
[autonomous migration policy](../../autonomous_crate_development.md#migration-policy),
reviewed Task-258B3N history, the selected completed sections, and their
surviving durable owners. Source behavior is not normative.

| Class | Decision |
|---|---|
| `design_drift` | Checker EN/JA TODOs repeat the same completed B3N implementation checklist outside a central historical owner. |
| `test_gap` | None. Existing generic schema-2 lint covers a new owning task row, exact redirects, indexes, links, fragments, hashes, counts, and section anchors. |
| `boundary_violation` | Avoided by selecting only two flat implementation-ledger TODO sections. Every plan, component API/invariant, runner route, audit, and successor section remains. |
| `spec_gap` / `source_drift` | None introduced or repaired. Historical bounded B3N drift remains time-local evidence and active behavior stays unchanged. |
| `source_undocumented_behavior` / `test_expectation_drift` | None inferred or changed. |
| `repo_metadata_conflict` | None at selection. |

## Frozen Sources And Anchors

[`DOC-258B3N-IMPLEMENTATION-LEDGER-COMPACT.sources.tsv`](../DOC-258B3N-IMPLEMENTATION-LEDGER-COMPACT.sources.tsv)
has two byte-sorted data rows, two comments, and final LF. Data-row SHA-256 is
`e58740e2b0e2848a5322c4fd117f67421600dceafca9b8b76c0e5e8bc96f3791`;
complete-file SHA-256 is
`9f7d02439377779afc6d30aaaa02626806bd18a5177784908bf51485627e130d`.
The source-locally unique, unlinked H2 sections have no nested headings,
tables, fences, or redirects and total 22 physical lines.

| Source | Lines | Section SHA-256 | Previous H2 | Next H2 |
|---|---:|---|---|---|
| EN checker TODO `4844-4854` | 11 | `419715ab1199ea82fb118519b5894431cd31a9f1af910dd1cd9cdac26a01020c` | `## Checker Task 258B3 Frozen-Contract Ledger` | `## Checker Task 258B3M1 Documentation Ledger` |
| JA checker TODO `4610-4620` | 11 | `53dfd04417ca08edb0dbc513ed222a8e32a6861694c899f93a364fff8b9c9344` | `## Checker Task 258B3 frozen-contract ledger` | `## Checker Task 258B3M1 documentation ledger` |

Blame assigns the headings and bodies to implementation
`2c6cf9682480893fdb2962b029643a1019c56149` and the trailing separators to
successor prerequisite `412dc7e5734393b66892f2e9a82fd740916321fa`; both are
ancestors.

## Owners, Scope, And Deferrals

The historical contract links the retained checker plan, statement, binding,
Typed/Resolved, runner plan/harness/boundary, authority, bilingual, and
coverage owners. This prerequisite changes exactly nine paths: the new
historical EN/JA pair, this EN/JA pair, the source TSV, and two Task Index rows
(historical task plus batch) in each checker/test EN/JA plan. Selected sources
and ledger remain unchanged; task-contract counts move `62/62 -> 64/64`.

Specifications, `.miz`, expectations, sidecars, trace metadata, coverage
audit, production, Cargo, public APIs, diagnostics, and active behavior are
forbidden. All frozen-contract, successor, runner, module, audit, and unlisted
sections remain. Binding publication, abbreviation, substitution,
obligations, facts, proof results, goals, theorem acceptance, active-corpus
ownership, and later witness-term families retain their existing ownership or
deferral. No coverage-audit edit is needed because mapping, ownership, status,
deferred reason, trace linkage, and coverage credit do not change.

## Protected Baseline And Expected Migration

Specification `64`, `.miz` `343`, expectation `435`, checker production `30`,
runner production `90`, and Cargo `21` path sets retain their frozen path and
content hashes. Trace SHA-256 remains
`55b754c8c4d0d293a1c44e2ba4b0090f407bba1d429b461b6cb4d6ddca9ca2b3`;
coverage audit remains
`2aa808aa033c04fa46ea76f65cfadcf0d8f0e1e53fba6d70edab223a7481685f`.
Ledger baseline is 892 lines with physical SHA-256
`9fbff2dc28e5bd3f331f80f688633c50ce702d80b48448c620ba848a6ae2eeae`,
24 batches, 34 tasks, two task references, 598 redirects, and 232 indexes.

The five CLI hashes remain plan
`700f4bf503783742cefd8004fa095675b7476d46e9a3a6dd439916d237eb6718`,
parse `a8a7aa639d2ebc65eddc923c7e9369ea5637d50e935f808600f446da1bfbda56`,
declaration `71e83ba0b20d4015e07b3bd2c0c4db2837b6151d1251812caed7954530d53c74`,
type `4b2c7bd5ec3cc56e5672fb351126126230ec84fba9bd2bd9049a516d378fab7f`,
and proof `ccf3d2d4d0a3755e00989d97af369a7c560302f76798d0a185d57ec3891e8450`.

After prerequisite commit and fresh replay, migration changes exactly five
paths: the EN/JA checker TODOs, this EN/JA pair, and
`legacy_compactions.tsv`. The 22 selected lines become two language-local
redirects to `258B3N.md#completion-evidence`, with exact source diff
`+2/-20`; all four neighboring H2 anchors remain byte-identical.

Ledger impact is 12 lines, `892 -> 904`: one batch, one task, eight indexes,
and two redirects over two source paths; no `task_ref` is added. Canonical
11-row expanded-inventory SHA-256 is
`c2e7829f540ff5c3a8a0575d7b7635fec23f323434107ade694e80fb2cbdcd57`;
expected physical ledger SHA-256 is
`e98a7d74b69a574e7362026bfafec8e5f9b2832fcae37e295bc02d048faa4abc`.
Final cardinalities are 25 batches, 35 tasks, two task references, 600
redirects, and 240 indexes.

## Reviews, Verification, And Exit

Prerequisite and migration separately require evidence-equivalence,
schema/test-sufficiency, bilingual/boundary, and independent final-quality
reviews as applicable, all ending **NO FINDINGS**. All nine hard gates must
PASS without a score cap at no less than `90/100`.

Verification includes preimage/blame/anchor replay; generic recursive
contract/link/fragment/ledger lint; checker/runner libraries and metadata;
formatting; offline metadata; warnings-denied all-target/all-feature Clippy;
full workspace tests; all five CLIs; protected count/hash; ledger order/hash/
cardinality; `git diff --check`; exact cached review; and unstaged/untracked
inspection. No push, fetch, reset, or stash mutation.

Prerequisite exits with exact nine-path scope, unchanged source sections and
ledger, synchronized EN/JA, all gates, one commit, and clean replay. Migration
exits separately with exact two redirects/five paths, generic schema replay,
all gates, one commit, and clean replay before selecting the next checker
duplication family.

## Documentation-Prerequisite Evidence

Independent evidence-equivalence, schema/test-sufficiency, and bilingual/
boundary reviews all end **NO FINDINGS**. They independently reproduce the EN
and JA 11-line preimages, section/source hashes, blame split, retained-owner
boundary, exact nine-path prerequisite and five-path migration, `+2/-20`,
prospective canonical/physical ledger hashes, and final cardinalities. Every
selected checklist fact is retained in the historical owner or a linked
durable owner; no source-derived semantic claim was added.

Generic lint passes `15/15`; checker `530/530`, runner `600/600`, and metadata
`137/137` tests pass. `cargo fmt --all --check`, offline Cargo metadata,
warnings-denied all-target/all-feature Clippy, and the full offline workspace
suite pass. All five CLIs exit zero and reproduce the frozen
plan/parse/declaration/type/proof hashes.

Protected path counts and NUL-delimited path hashes reproduce exactly as
specification `64`, `.miz` `343`, expectation `435`, checker production `30`,
runner production `90`, and Cargo `21`; zero protected diff preserves every
frozen content hash. Trace, coverage audit, 892-line ledger, selected
preimages, and immutable source TSV reproduce their frozen hashes. Task
contracts measure `64/64`; `git diff --check` passes.

Repository inventory remains selection HEAD on clean-base `main` with the
task-only nine-path worktree, `origin/main...HEAD=0/4`, and protected
`stash@{0}=f65cf4a13752ec380710814a9ac6392ccb9d75d4`. No push, fetch, reset, or
stash mutation occurred. Independent final read-only quality ends **NO
FINDINGS**; all nine hard gates PASS, no score cap applies, and the valid score
is **100/100** (`20/20/15/15/10/10/5/5`). Exact staging, commit, and clean
post-commit replay remain.

## Migration Evidence

The documentation prerequisite committed separately as
`7634d54102aebc75c3623e477ec79ce35e4cca15`. Clean fresh replay reproduced
both frozen preimages, the source TSV hashes, unchanged 892-line ledger,
protected no-ops, `origin/main...HEAD=0/5`, and the protected stash before
migration.

The selected EN/JA TODO sections are now language-local redirects to
`258B3N.md#completion-evidence`. Exact source diff is `+2/-20`; both forbidden
legacy headings and bodies are gone, while all four neighboring H2 anchors and
every unselected TODO section remain.

The ledger adds exactly 12 byte-sorted rows: one batch, one canonical task,
eight indexes, and two redirects. It is 904 lines with physical SHA-256
`e98a7d74b69a574e7362026bfafec8e5f9b2832fcae37e295bc02d048faa4abc`,
reproduces canonical 11-row SHA-256
`c2e7829f540ff5c3a8a0575d7b7635fec23f323434107ade694e80fb2cbdcd57`,
and measures 25 batches, 35 tasks, two task references, 600 redirects, and 240
indexes. The historical contract, source TSV, four plans, protected surfaces,
trace, and coverage audit remain unchanged.

Independent evidence-equivalence, schema/test-sufficiency, and bilingual/
boundary migration reviews all end **NO FINDINGS**. They independently prove
the exact whole-H2 splices, `+2/-20` source delta, language-local redirects,
retained neighboring anchors and unselected sections, complete preservation
of every removed checklist fact, schema-2 ownership, exact ledger rows,
hashes, cardinalities, and protected no-ops.

Generic lint passes `15/15`; checker `530/530`, runner `600/600`, and metadata
`137/137` tests pass. `cargo fmt --all --check`, offline Cargo metadata,
warnings-denied all-target/all-feature Clippy, and the full offline workspace
suite pass. All five CLIs exit zero and reproduce the frozen
plan/parse/declaration/type/proof hashes. Protected path counts and hashes,
trace, coverage audit, source TSV, historical contracts, four plans, and every
frozen content hash reproduce exactly; `git diff --check` passes. No push,
fetch, reset, or stash mutation occurred.

Independent final read-only quality ends **NO FINDINGS**; all nine hard gates
PASS, no score cap applies, and the valid score is **100/100**
(`20/20/15/15/10/10/5/5`). Exact five-path staging, commit, and clean
post-commit replay remain.
