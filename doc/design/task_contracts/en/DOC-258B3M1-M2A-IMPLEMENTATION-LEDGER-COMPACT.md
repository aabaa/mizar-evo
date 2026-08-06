# Task DOC-258B3M1-M2A-IMPLEMENTATION-LEDGER-COMPACT: Early B3M Implementation-Ledger Compaction

> Canonical language: English. Japanese companion:
> [../ja/DOC-258B3M1-M2A-IMPLEMENTATION-LEDGER-COMPACT.md](../ja/DOC-258B3M1-M2A-IMPLEMENTATION-LEDGER-COMPACT.md).

## Identity And Status

| Field | Frozen value |
|---|---|
| Task | `DOC-258B3M1-M2A-IMPLEMENTATION-LEDGER-COMPACT` |
| Status | Migration complete through independent final quality with all reviews at no findings, all nine hard gates PASS, no score cap, and 100/100. Exact staging, commit, and clean replay remain. |
| Purpose | Centralize the completed Task-258B3M1 and Task-258B3M2A implementation checklists while retaining their documentation ledgers and every durable checker/runner owner. |
| Historical owners | [Task 258B3M1](./258B3M1.md#completion-evidence) and [Task 258B3M2A](./258B3M2A.md#completion-evidence) |
| Plan indexes | [checker plan](../../mizar-checker/en/00.crate_plan.md#task-index) and [runner plan](../../mizar-test/en/00.crate_plan.md#task-index) |
| Selection HEAD | `b4f97b2ea5f9bba17bf084929214b749389b08b9` |
| Repository state | clean `main`, `origin/main...HEAD=0/6`, protected `stash@{0}=f65cf4a13752ec380710814a9ac6392ccb9d75d4` |
| Dependencies | Both historical documentation/implementation pairs, their lower-stage prerequisites, and generic schema-2 ledger support are ancestors of selection HEAD. |

## Authority, Consumers, And Classification

Authority is the user-approved checker-first compaction program,
[`AGENTS.md`](../../../../AGENTS.md), the
[autonomous migration policy](../../autonomous_crate_development.md#migration-policy),
the retained canonical/task evidence linked by the historical contracts, the
four selected completed sections, and their durable owners. Source behavior is
not normative. The generic lint-policy consumer owns recursive contracts,
links, fragments, plan indexes, section anchors, manifest counts, ordering, and
hash replay; human readers consume the language-local redirects.

| Class | Decision |
|---|---|
| `design_drift` | Checker EN/JA TODOs repeat completed B3M1 and B3M2A implementation checklists outside central historical owners; this prerequisite creates the missing owner pairs. |
| `test_gap` | None. Existing generic schema-2 lint covers two owning task rows, four exact redirects, indexes, links, fragments, hashes, counts, and anchors. |
| `boundary_violation` | Avoided by selecting one implementation-ledger section per task/source pair. The adjacent documentation ledgers and every lower-stage, frozen-contract, successor, runner, module, and audit section remain. |
| `spec_gap` / `source_drift` | None introduced or repaired. Historical bounded task drift and its closure remain time-local evidence. |
| `source_undocumented_behavior` / `test_expectation_drift` | None inferred or changed. |
| `repo_metadata_conflict` | None at selection. Historical Task-258B3M2A metadata incidents remain only in their retained audit owner. |

## Frozen Sources, Fingerprints, And Anchors

[`DOC-258B3M1-M2A-IMPLEMENTATION-LEDGER-COMPACT.sources.tsv`](../DOC-258B3M1-M2A-IMPLEMENTATION-LEDGER-COMPACT.sources.tsv)
has four byte-sorted data rows, two comments, and final LF. Data-row SHA-256 is
`b4122c601f5fff6c2628a88163c1817c5fb5439cca9db4c9abcb816b13bb0c15`;
complete-file SHA-256 is
`f0c3a76f37ff98b5e3e0553755eca0a63fb809f48fa50184144452ec68b75f56`.
The source-locally unique, unlinked H2 sections have no nested headings,
tables, fences, or redirects and total 55 physical lines.

| Task/source | Lines | Section SHA-256 | Previous H2 | Next H2 |
|---|---:|---|---|---|
| B3M1 EN checker TODO `4861-4875` | 15 | `d4bcfe36d6ccababe39e50fcb9932d1d3fb3eef8a5e23d3dac1ecbc2ca53ea50` | `## Checker Task 258B3M1 Documentation Ledger` | `## Checker Task 258B3M2A Documentation Ledger` |
| B3M1 JA checker TODO `4626-4639` | 14 | `72b6d7fe7feefec042e62173e6dae56b33fffa6da1e340b7161d5715b8dc16c7` | `## Checker Task 258B3M1 documentation ledger` | `## Checker Task 258B3M2A documentation ledger` |
| B3M2A EN checker TODO `4891-4903` | 13 | `10fbb26471d4389a0796de5488475a9e88208eeadf51f4b5fa95590709607e4d` | `## Checker Task 258B3M2A Documentation Ledger` | `## Checker Task 258B3M2B1 Frozen-Contract Ledger` |
| B3M2A JA checker TODO `4656-4668` | 13 | `9082b77bacd68cdf66536ba634e73106eb3ca2e9847596caf3a43f4bc068933e` | `## Checker Task 258B3M2A documentation ledger` | `## Checker Task 258B3M2B1 frozen-contract ledger` |

Blame assigns the B3M1 headings and bodies to implementation
`cffd46f810fb05f2efc78859382f30678ffe1c3d` and their trailing separators to
B3M2A prerequisite `0847727f7a3d62c2e241aa96de546761a26f5e0c`. It assigns
the B3M2A headings and bodies to implementation
`477fe251fa21a5fb3d0cbb9956a3c61ee14b648d` and their trailing separators to
B3M2B1 prerequisite `da68793d126c3105564d127b08800538f262e789`. All are
ancestors of selection HEAD.

## Owners, Scope, Prohibitions, And Deferrals

The historical contracts link the stable checker plan/statement/binding/
Typed/Resolved owners, runner plan/harness/boundary owners, authority and
bilingual audits, and coverage addenda. This prerequisite changes exactly 11
paths: two new historical EN/JA pairs, this EN/JA pair, the immutable source
TSV, and three Task Index rows (two historical tasks plus this batch) in each
checker/test EN/JA plan. Selected TODO sections and `legacy_compactions.tsv`
remain unchanged; task-contract counts move `64/64 -> 67/67`.

Specifications, `.miz`, fixtures, expectations, sidecars, trace metadata,
coverage audit, production, Cargo, public APIs, diagnostics, and active
behavior are forbidden. The Task-258B3M1/B3M2A documentation ledgers, the
B3M2A lexer prerequisite, every successor section, and all owner-local API,
invariant, runner, audit, and trace material remain. Binding publication,
abbreviation, substitution, obligations, facts, proof results, goals, theorem
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
Ledger baseline is 904 lines with physical SHA-256
`e98a7d74b69a574e7362026bfafec8e5f9b2832fcae37e295bc02d048faa4abc`,
25 batches, 35 tasks, two task references, 600 redirects, and 240 indexes.

The five CLI hashes remain plan
`700f4bf503783742cefd8004fa095675b7476d46e9a3a6dd439916d237eb6718`,
parse `a8a7aa639d2ebc65eddc923c7e9369ea5637d50e935f808600f446da1bfbda56`,
declaration `71e83ba0b20d4015e07b3bd2c0c4db2837b6151d1251812caed7954530d53c74`,
type `4b2c7bd5ec3cc56e5672fb351126126230ec84fba9bd2bd9049a516d378fab7f`,
and proof `ccf3d2d4d0a3755e00989d97af369a7c560302f76798d0a185d57ec3891e8450`.

After prerequisite commit and clean replay, migration changes exactly five
paths: the EN/JA checker TODOs, this EN/JA pair, and the ledger. The 55 selected
lines become four language-local redirects, exact source diff `+4/-51`; every
recorded neighboring anchor and unselected section remains byte-identical.

Ledger impact is 19 lines, `904 -> 923`: one batch, two canonical tasks,
twelve indexes, and four redirects over two source paths; no `task_ref` is
added. Canonical 18-row expanded-inventory SHA-256 is
`103e804ae1fe2e561b4c5047048cba5f0c659c43b625776c60a3f9828b3512cb`;
expected physical ledger SHA-256 is
`a5d809911f8ffe4996db8eb147fd17253c101a5921098ffc51be87fda3f99b3f`.
Final cardinalities are 26 batches, 37 tasks, two task references, 604
redirects, and 252 indexes.

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

## Documentation-Prerequisite Evidence

Independent evidence-equivalence, schema/test-sufficiency, and bilingual/
boundary reviews end **NO FINDINGS**. Review corrected scratch calculations
that first treated `\t` as two literal bytes and then included the fixed schema
row in the data-row sort. Literal-tab replay with the ledger's comment and
schema fixed at lines 1-2 now independently reproduces the frozen 18-row and
923-line physical ledger hashes. The historical owners explicitly preserve
B3M2A's prior debug grammar. Reviewers also reproduce all four selected
preimages, anchors, blame/history, source TSV hashes, exact 11-path scope, Task
Index rows, owner links, classifications, deferrals, and protected no-ops
without authority or semantic expansion.

Generic lint passes `15/15`; checker `530/530`, runner `600/600`, and metadata
`137/137` tests pass. `cargo fmt --all --check`, offline Cargo metadata,
warnings-denied all-target/all-feature Clippy, and the full offline workspace
suite pass. All five CLIs exit zero with the existing 23 warnings and zero
errors and reproduce the frozen plan/parse/declaration/type/proof hashes.

Protected path counts and NUL-delimited path hashes reproduce exactly as
specification `64`, `.miz` `343`, expectation `435`, checker production `30`,
runner production `90`, and Cargo `21`; zero protected diff preserves every
frozen content hash. Trace, coverage audit, unchanged 904-line ledger, four
selected preimages, and immutable source TSV reproduce their frozen hashes.
Task contracts measure `67/67`; `git diff --check` passes.

Repository inventory remains selection HEAD on clean-base `main` with the
task-only 11-path worktree, `origin/main...HEAD=0/6`, and protected
`stash@{0}=f65cf4a13752ec380710814a9ac6392ccb9d75d4`. No push, fetch, reset, or
stash mutation occurred. Independent final read-only quality ends **NO
FINDINGS**; all nine hard gates PASS, no score cap applies, and the valid score
is **100/100** (`20/20/15/15/10/10/5/5`). Exact staging, commit, and clean
post-commit replay remain.

## Migration Evidence

The documentation prerequisite committed separately as
`e604125f8b9be8052ebc686fa294bcb926448906`. Clean fresh replay reproduced all
four frozen preimages, source TSV hashes, unchanged 904-line ledger, protected
no-ops, `67/67` contracts, `origin/main...HEAD=0/7`, and the protected stash
before migration.

The selected Task-258B3M1 and Task-258B3M2A implementation-ledger sections are
now four language-local redirects to their historical completion evidence.
Exact source diff is `+4/-51`; all four forbidden implementation headings and
bodies are gone. Both EN/JA documentation ledgers, every recorded neighboring
anchor, and every unselected TODO section remain.

The ledger adds exactly 19 byte-sorted rows: one batch, two canonical tasks,
twelve indexes, and four redirects. It is 923 lines with physical SHA-256
`a5d809911f8ffe4996db8eb147fd17253c101a5921098ffc51be87fda3f99b3f`,
reproduces canonical 18-row SHA-256
`103e804ae1fe2e561b4c5047048cba5f0c659c43b625776c60a3f9828b3512cb`,
and measures 26 batches, 37 tasks, two task references, 604 redirects, and 252
indexes. Historical contracts, source TSV, four plans, protected surfaces,
trace, and coverage audit remain unchanged. Generic lint passes `15/15` and
`git diff --check` passes.

Independent evidence-equivalence, schema/test-sufficiency, and bilingual/
boundary migration reviews end **NO FINDINGS**. Reviewers independently
reproduce every frozen preimage and retained fact, the exact five-path scope,
language-local redirects and fragments, neighboring anchors, source TSV and
ledger hashes, schema rows/counts/order, protected no-op, and EN/JA parity.

Generic lint passes `15/15`; checker `530/530`, runner `600/600`, and metadata
`137/137` tests pass. `cargo fmt --all --check`, offline Cargo metadata,
warnings-denied all-target/all-feature Clippy, and the full offline workspace
suite pass. All five CLIs exit zero with the existing 23 warnings and zero
errors and reproduce the frozen plan/parse/declaration/type/proof hashes.

Protected path counts and NUL-delimited path hashes reproduce exactly as
specification `64`, `.miz` `343`, expectation `435`, checker production `30`,
runner production `90`, and Cargo `21`; zero protected diff preserves every
frozen content hash. Trace and coverage-audit hashes reproduce. The immutable
source TSV, four historical contracts, and four plan indexes remain unchanged;
the ledger and its cardinalities reproduce the values above. `git diff
--check` passes. The initial metadata test invocation used the nonexistent
target name `metadata_consistency`; repository target discovery identified
`metadata`, whose required `137/137` tests then passed.

Independent final read-only quality ends **NO FINDINGS**. All nine hard gates
PASS, no score cap applies, and the valid score is **100/100**
(`20/20/15/15/10/10/5/5`). Exact five-path staging, commit, and clean
post-commit replay remain.
