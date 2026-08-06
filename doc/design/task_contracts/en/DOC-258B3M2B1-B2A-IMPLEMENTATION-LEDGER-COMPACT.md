# Task DOC-258B3M2B1-B2A-IMPLEMENTATION-LEDGER-COMPACT: Early B3M2B Implementation-Ledger Compaction

> Canonical language: English. Japanese companion:
> [../ja/DOC-258B3M2B1-B2A-IMPLEMENTATION-LEDGER-COMPACT.md](../ja/DOC-258B3M2B1-B2A-IMPLEMENTATION-LEDGER-COMPACT.md).

## Identity And Status

| Field | Frozen value |
|---|---|
| Task | `DOC-258B3M2B1-B2A-IMPLEMENTATION-LEDGER-COMPACT` |
| Status | Migration implemented with the exact frozen redirects and ledger rows; migration reviews, full verification, and independent final quality are complete. Exact staging, commit, and clean replay remain. |
| Purpose | Centralize the completed Task-258B3M2B1 and Task-258B3M2B2A implementation checklists while retaining both frozen-contract ledgers and every durable checker/runner owner. |
| Historical owners | [Task 258B3M2B1](./258B3M2B1.md#completion-evidence) and [Task 258B3M2B2A](./258B3M2B2A.md#completion-evidence) |
| Plan indexes | [checker plan](../../mizar-checker/en/00.crate_plan.md#task-index) and [runner plan](../../mizar-test/en/00.crate_plan.md#task-index) |
| Selection HEAD | `a9435046608eeb69c8ac284c65b069729d62cab2` |
| Repository state | clean `main`, `origin/main...HEAD=0/8`, protected `stash@{0}=f65cf4a13752ec380710814a9ac6392ccb9d75d4` |
| Dependencies | Both historical documentation/implementation pairs, their lower-stage prerequisites, prior B3M1/M2A compaction, and generic schema-2 ledger support are ancestors of selection HEAD. |

## Authority, Consumers, And Classification

Authority is the user-approved checker-first compaction program,
[`AGENTS.md`](../../../../AGENTS.md), the
[autonomous migration policy](../../autonomous_crate_development.md#migration-policy),
the retained canonical/test evidence linked by the historical owners, the
four selected completed sections, and their durable owners. Source behavior is
not normative. The generic lint-policy consumer owns recursive contracts,
links, fragments, plan indexes, section anchors, manifest counts, ordering,
and hash replay; human readers consume the language-local redirects.

| Class | Decision |
|---|---|
| `design_drift` | Checker EN/JA TODOs repeat completed B3M2B1 and B3M2B2A implementation checklists outside central historical owners; the owners and their index rows are absent at selection. |
| `test_gap` | None. Existing generic schema-2 lint covers two owning task rows, four exact redirects, indexes, links, fragments, hashes, counts, and anchors. |
| `boundary_violation` | Avoided by selecting one flat implementation-ledger section per task/source pair. Both frozen-contract ledgers and the B2B1P lower prerequisite remain. Historical operational boundary incidents remain only in their retained audits. |
| `spec_gap` / `source_drift` | None introduced or repaired. Historical bounded task drift and closure remain time-local evidence. |
| `source_undocumented_behavior` / `test_expectation_drift` | None inferred or changed. |
| `repo_metadata_conflict` | None at selection. Historical report-only metadata movement remains in its existing owner and is not repaired. |

## Frozen Sources, Fingerprints, And Anchors

[`DOC-258B3M2B1-B2A-IMPLEMENTATION-LEDGER-COMPACT.sources.tsv`](../DOC-258B3M2B1-B2A-IMPLEMENTATION-LEDGER-COMPACT.sources.tsv)
has four byte-sorted data rows, two comments, and final LF. Data-row SHA-256 is
`f357920c98003b90c4406d3b70c0d62e541f9e57ac5c28a0242d8477ca1dd9e6`;
complete-file SHA-256 is
`6e7b38df4a971384f7ce757592feb21b43a2b4115e2d6563037c81698b9ba677`.
The source-locally unique, unlinked H2 sections have no nested headings,
tables, fences, or redirects and total 56 physical lines.

| Task/source | Lines | Section SHA-256 | Previous H2 | Next H2 |
|---|---:|---|---|---|
| B1 EN checker TODO `4898-4911` | 14 | `8ef97be51c99d6a5c08e27267fe9700db613620fa337d7aa15403f3efb023de7` | `## Checker Task 258B3M2B1 Frozen-Contract Ledger` | `## Checker Task 258B3M2B2A Frozen-Contract Ledger` |
| B1 JA checker TODO `4662-4675` | 14 | `c63ea162f91c65212dccd1b62d1b2f528794bab24d81cb2b6070720ab868b037` | `## Checker Task 258B3M2B1 frozen-contract ledger` | `## Checker Task 258B3M2B2A frozen-contract ledger` |
| B2A EN checker TODO `4927-4940` | 14 | `2b296580961f453a76b8ff41e116359b2a90a615ae91a332a518c81e4e25b0cf` | `## Checker Task 258B3M2B2A Frozen-Contract Ledger` | `## Checker Task 258B3M2B2B1P Frozen Lower-Prerequisite Ledger` |
| B2A JA checker TODO `4690-4703` | 14 | `84269124a9de1a46b8462a03f0fd451aff275f307f35e480cd81ae42fc14422e` | `## Checker Task 258B3M2B2A frozen-contract ledger` | `## Checker Task 258B3M2B2B1P frozen lower-prerequisite ledger` |

Blame assigns the B1 headings and bodies to implementation
`71dda758c465621f905d7432da0de246503448a3` and their trailing separators to
B2A prerequisite `2bcf774c42b2e0b464841b1db898db52542eb798`. It assigns
the B2A headings and bodies to implementation
`c60b3d3b8cac1fad7f5cbcddd08f287322206321` and their trailing separators to
B2B1P prerequisite `b196a9ce95c5f0b62fe6f2ae64cee4e3fe9ea704`. All are
ancestors of selection HEAD.

## Owners, Scope, Prohibitions, And Deferrals

The historical contracts link the stable checker plan/statement/binding/
Typed/Resolved owners, runner plan/harness/boundary owners, authority and
bilingual audits, and coverage addenda. This prerequisite changes exactly 11
paths: two new historical EN/JA pairs, this EN/JA pair, the immutable source
TSV, and three Task Index rows in each checker/test EN/JA plan. Selected TODO
sections and `legacy_compactions.tsv` remain unchanged; task-contract counts
move `67/67 -> 70/70`.

Specifications, `.miz`, fixtures, expectations, sidecars, trace metadata,
coverage audit, production, Cargo, public APIs, diagnostics, and active
behavior are forbidden. Both frozen-contract ledgers, the B2B1P lower
prerequisite, every successor, and all owner-local API, invariant, runner,
audit, and trace material remain. Binding publication, typing, existential
introduction, substitution, obligations, facts, proof results, goals, theorem
acceptance, active-corpus ownership, and remaining witness-term families keep
their existing ownership or deferral. No coverage-audit edit is needed because
mapping, status, deferred reason, trace linkage, and coverage credit do not
change.

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
Ledger baseline is 923 lines with physical SHA-256
`a5d809911f8ffe4996db8eb147fd17253c101a5921098ffc51be87fda3f99b3f`,
26 batches, 37 tasks, two task references, 604 redirects, and 252 indexes.

The five CLI hashes remain plan
`700f4bf503783742cefd8004fa095675b7476d46e9a3a6dd439916d237eb6718`,
parse `a8a7aa639d2ebc65eddc923c7e9369ea5637d50e935f808600f446da1bfbda56`,
declaration `71e83ba0b20d4015e07b3bd2c0c4db2837b6151d1251812caed7954530d53c74`,
type `4b2c7bd5ec3cc56e5672fb351126126230ec84fba9bd2bd9049a516d378fab7f`,
and proof `ccf3d2d4d0a3755e00989d97af369a7c560302f76798d0a185d57ec3891e8450`.

After prerequisite commit and clean replay, migration changes exactly five
paths: the EN/JA checker TODOs, this EN/JA pair, and the ledger. The 56 selected
lines become four language-local redirects, exact source diff `+4/-52`; every
recorded neighboring anchor and unselected section remains byte-identical.

Ledger impact is 19 lines, `923 -> 942`: one batch, two canonical tasks,
twelve indexes, and four redirects over two source paths; no `task_ref` is
added. Canonical 18-row expanded-inventory SHA-256 is
`4ce4f8564f99478a229756ea8b9313f627fbe869ab0fb784b96c1e427b3565e5`;
expected physical ledger SHA-256 is
`e5a804a4c5b452610e0024b1de3186445b5179b43d138927f7e3a079bb19af41`.
Final cardinalities are 27 batches, 39 tasks, two task references, 608
redirects, and 264 indexes.

## Reviews, Tests, Audit Impact, And Exit

Prerequisite and migration separately require evidence-equivalence,
schema/test-sufficiency, bilingual/boundary, and independent final-quality
reviews as applicable, all ending **NO FINDINGS**. All nine hard gates must
PASS without a score cap at no less than `90/100`. No new fixture,
expectation, sidecar, trace row, or semantic test is authorized; the existing
generic lint is the only new-contract consumer.

Verification includes source/commit/blame/anchor replay; recursive contract/
link/fragment/ledger lint; checker/runner libraries and metadata; formatting;
offline metadata; warnings-denied all-target/all-feature Clippy; full workspace
tests; all five CLIs; protected count/hash; ledger order/hash/cardinality;
`git diff --check`; exact cached review; and unstaged/untracked inspection. No
push, fetch, reset, or stash mutation.

Prerequisite exits with exact 11-path scope, unchanged selected sections and
ledger, synchronized EN/JA, all gates, one commit, and clean replay. Migration
exits separately with exact four redirects/five paths, complete evidence
preservation, generic schema replay, all gates, one commit, and clean replay
before fresh selection of the next checker duplication family.

## Next Handoff

After the prerequisite commit, freshly replay this contract and implement the
same task's four redirects plus 19 ledger rows. Do not compact either
frozen-contract ledger, the B2B1P lower-prerequisite ledger, any runner or
owner-local section, or any other task.

## Documentation-Prerequisite Evidence

Independent evidence-equivalence, schema/test-sufficiency, and bilingual/
boundary reviews end **NO FINDINGS**. Reviewers independently reproduce all
four 14-line preimages, anchors, blame/history, source TSV hashes, exact
11-path scope, 12 language-local Task Index rows, `67/67 -> 70/70` contract
pairs, owner links, classifications, deferrals, and protected no-ops. They
also reconstruct the prospective 18-row canonical inventory and 942-line
physical ledger hashes, exact `+4/-52` migration, and final cardinalities
without schema, authority, or semantic expansion.

Generic lint passes `15/15`; checker `530/530`, runner `600/600`, and metadata
`137/137` tests pass. `cargo fmt --all --check`, offline Cargo metadata,
warnings-denied all-target/all-feature Clippy, and the full offline workspace
suite pass. All five CLIs exit zero with the existing 23 warnings and zero
errors and reproduce the frozen plan/parse/declaration/type/proof hashes.

Protected path counts and NUL-delimited path hashes reproduce exactly as
specification `64`, `.miz` `343`, expectation `435`, checker production `30`,
runner production `90`, and Cargo `21`; zero protected diff preserves every
frozen content hash. Trace, coverage audit, unchanged 923-line ledger, four
selected preimages, and immutable source TSV reproduce their frozen hashes.
Task contracts measure `70/70`; `git diff --check` passes.

Repository inventory remains selection HEAD on clean-base `main` with the
task-only 11-path worktree, `origin/main...HEAD=0/8`, and protected
`stash@{0}=f65cf4a13752ec380710814a9ac6392ccb9d75d4`. No push, fetch, reset, or
stash mutation occurred from this workflow. During review, the remote-tracking
ref independently moved from `71edf340` to selection HEAD `a9435046` with
reflog reason `update by push`, making the live relation `0/0`. This is a
report-only `repo_metadata_conflict`: task files and ancestry remain unchanged,
the exact commit target is safe, and no repair is authorized. Exact staging,
the dedicated prerequisite commit, and clean post-commit replay remain.

Independent final read-only quality ends **NO FINDINGS**. All nine hard gates
PASS, no score cap applies, and the valid score is **100/100**
(`20/20/15/15/10/10/5/5`). Exact staging, the dedicated prerequisite commit,
and clean post-commit replay remain.

## Migration Evidence

The documentation prerequisite committed separately as
`11d5453b8f6e9f60d5fc11cd8970369de83b5a35`. Clean fresh replay reproduced all
four frozen preimages, source TSV hashes, unchanged 923-line ledger, protected
no-ops, `70/70` contracts, `origin/main...HEAD=0/1`, and the protected stash
before migration.

The selected Task-258B3M2B1 and Task-258B3M2B2A implementation-ledger sections
are now four language-local redirects to their historical completion evidence.
Exact source diff is `+4/-52`; all four forbidden implementation headings and
bodies are gone. Both EN/JA frozen-contract ledgers, the B2B1P lower-
prerequisite ledger, every recorded neighboring anchor, and every unselected
TODO section remain.

The ledger adds exactly 19 byte-sorted rows: one batch, two canonical tasks,
twelve indexes, and four redirects. It is 942 lines with physical SHA-256
`e5a804a4c5b452610e0024b1de3186445b5179b43d138927f7e3a079bb19af41`,
reproduces canonical 18-row SHA-256
`4ce4f8564f99478a229756ea8b9313f627fbe869ab0fb784b96c1e427b3565e5`,
and measures 27 batches, 39 tasks, two task references, 608 redirects, and 264
indexes. Historical contracts, source TSV, four plans, protected surfaces,
trace, and coverage audit remain unchanged. Generic lint passes `15/15` and
`git diff --check` passes.

Independent migration evidence-equivalence, schema/test-sufficiency, and
bilingual/boundary reviews end **NO FINDINGS**. They reproduce every frozen
preimage and unique fact, the exact `+4/-52` redirect delta, the 19 ledger
rows, language-local links and fragments, retained exclusions, ordering,
cardinalities, and both frozen hashes without schema or semantic expansion.

Generic lint passes `15/15`; checker `530/530`, runner `600/600`, and metadata
`137/137` tests pass. `cargo fmt --all --check`, offline Cargo metadata,
warnings-denied all-target/all-feature Clippy, and the full offline workspace
suite pass. All five CLIs exit zero with the unchanged 23 warnings and zero
errors and reproduce the frozen plan/parse/declaration/type/proof hashes.

Protected path counts and NUL-delimited path hashes reproduce exactly as
specification `64`, `.miz` `343`, expectation `435`, checker production `30`,
runner production `90`, and Cargo `21`; zero protected diff preserves every
frozen content hash. Trace, coverage audit, immutable source TSV, and `70/70`
contracts reproduce their frozen counts and hashes. `git diff --check` passes.
Verification first measured prerequisite HEAD on `main` with the exact
task-only five-path worktree, `origin/main...HEAD=0/1`, and protected
`stash@{0}=f65cf4a13752ec380710814a9ac6392ccb9d75d4`. During focused evidence
review, the remote-tracking ref independently moved from `a9435046` to
prerequisite HEAD `11d5453b` with reflog reason `update by push`, making the
live relation `0/0`. This is a report-only `repo_metadata_conflict`: task
files and ancestry remain unchanged, the exact commit target remains safe,
and no repair is authorized. No push, fetch, reset, or stash mutation occurred
from this workflow.

Independent final read-only quality ends **NO FINDINGS**. All nine hard gates
PASS, no score cap applies, and the valid score is **100/100**
(`20/20/15/15/10/10/5/5`). Exact five-path staging, commit, and clean
post-commit replay remain.
