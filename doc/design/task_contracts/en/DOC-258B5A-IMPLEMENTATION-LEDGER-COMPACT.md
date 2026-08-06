# Task DOC-258B5A-IMPLEMENTATION-LEDGER-COMPACT: B5A Implementation-Ledger Compaction

> Canonical language: English. Japanese companion:
> [../ja/DOC-258B5A-IMPLEMENTATION-LEDGER-COMPACT.md](../ja/DOC-258B5A-IMPLEMENTATION-LEDGER-COMPACT.md).

## Identity And Status

| Field | Frozen value |
|---|---|
| Task | `DOC-258B5A-IMPLEMENTATION-LEDGER-COMPACT` |
| Status | Migration complete; exact staging and the dedicated migration commit remain. |
| Purpose | Centralize the EN/JA checker TODO implementation-completion checklists for historical Task 258B5A without changing any durable checker or runner owner. |
| Historical owner | [Task 258B5A](./258B5A.md#completion-evidence) |
| Plan indexes | [checker plan](../../mizar-checker/en/00.crate_plan.md#task-index) and [runner plan](../../mizar-test/en/00.crate_plan.md#task-index) |
| Selection HEAD | `71edf3400bd8da556322c0510d6824bb62302c60` |
| Repository state | clean `main`, `origin/main...HEAD=0/3`, protected `stash@{0}=f65cf4a13752ec380710814a9ac6392ccb9d75d4` |
| Dependency | Schema-2 `task_ref` support and its first real B4C route are committed; prior batch `DOC-258B5A-COMPACT` remains immutable. |

## Authority And Classification

Authority is the user-approved checker-first compaction program,
[`AGENTS.md`](../../../../AGENTS.md), the
[autonomous migration policy](../../autonomous_crate_development.md#migration-policy),
reviewed Git history, the two selected completed sections, and their surviving
durable owners. Source behavior is not normative.

| Class | Decision |
|---|---|
| `design_drift` | Checker EN/JA TODOs repeat the same historical B5A upper-implementation scope, provenance, tests, reviews, gates, commit, and handoff evidence outside their central historical owner. |
| `test_gap` | None. Schema-2 synthetic vectors and the real B4C `task_ref` route are committed; generic lint will replay this second real reference in the separate migration. |
| `boundary_violation` | Avoided by selecting only two flat TODO sections. All plan, module, audit, runner, final-quality, lower-stage, and successor sections retain their durable local facts. |
| `spec_gap` / `source_drift` | None introduced or repaired; language semantics and completed implementation remain unchanged. |
| `source_undocumented_behavior` / `test_expectation_drift` | None inferred or changed. |
| `repo_metadata_conflict` | None at selection. During prerequisite review, `origin/main` moved externally by push from the frozen `0/3` relation to the same `71edf340` HEAD (`0/0`). This is report-only; the current safe commit target remains unambiguous. |

## Frozen Sources And Anchors

[`DOC-258B5A-IMPLEMENTATION-LEDGER-COMPACT.sources.tsv`](../DOC-258B5A-IMPLEMENTATION-LEDGER-COMPACT.sources.tsv)
contains two byte-sorted data rows, two comments, and final LF. Data-row
SHA-256 is `8c0456cbea112f83755cc52c360bb38ad74ae8b737b59d4ac10215b9c9f0547b`;
complete-file SHA-256 is
`729303b32e50414274ee15dc573aeb9449e50e431f97579cd7210722b862b341`.
The flat, source-locally unique, unlinked sections contain no nested headings,
tables, fences, or redirects and total 54 physical lines.

| Source | Lines | Section SHA-256 | Previous H2 | Next H2 |
|---|---:|---|---|---|
| EN checker TODO `5962-5989` | 28 | `798408cf0f85b4ec67a65c2422dbe813fc160eb760b1424bb43bdfe897deeb39` | `## Checker Task 258B5A Frozen-Contract Documentation Prerequisite` | `## Checker Task 258B5B Frozen-Contract Documentation Prerequisite` |
| JA checker TODO `5697-5722` | 26 | `3bba29a62093be333492e51339baeb26df14dc880b005f203910ea89a184dfca` | `## Checker Task 258B5A frozen-contract documentation prerequisite` | `## Checker Task 258B5B frozen-contract documentation prerequisite` |

Implementation commit `4a79116c1a6f71155e4f366950fee8335b4dc8f1`
introduced 24 EN and 22 JA selected lines. Successor prerequisite
`141dc44a757555e8d4837756515e1577f672348b` added only the four-line exact
staging/commit/post-commit tail in each language. Current blame attributes all
selected lines to those two commits.

The prior Task-258B5A batch uses 14 other source paths. Its task row and four
historical Task Index records stay solely owned by `DOC-258B5A-COMPACT`; this
batch adds one `task_ref` and indexes only its own batch contract. The old and
new source-file sets are disjoint, satisfying schema 2.

## Retained Owners And Exclusions

The checker and runner frozen/implemented plans, statement, binding, Typed and
Resolved AST, harness, module-boundary, bilingual, source/specification,
traceability, coverage, and final-quality owners remain unchanged. Every
checker and runner TODO section other than the two selected H2s remains.

Specifications, `.miz`, expectations, sidecars, traceability, coverage audit,
production, Cargo, public APIs, diagnostics, and active behavior are forbidden.
No ancestor/descendant visibility, label/citation scope, resolver behavior,
rollback/replay meaning, B1/B5B/B5C semantics, proof, fact, acceptance, Core,
CFG, or VC behavior is inferred or changed.

## Protected Baseline

Specification `64`, `.miz` `343`, expectation `435`, checker production `30`,
runner production `90`, and Cargo `21` path sets retain the frozen path/content
hashes. Trace SHA-256 remains
`55b754c8c4d0d293a1c44e2ba4b0090f407bba1d429b461b6cb4d6ddca9ca2b3`;
coverage audit remains
`2aa808aa033c04fa46ea76f65cfadcf0d8f0e1e53fba6d70edab223a7481685f`.
The schema-2 ledger baseline is 872 lines with physical SHA-256
`5ac307e25074e8a776024a0a060fab9d45ca68a631ca39a40283f14bfe6d485b`,
22 batches, 33 tasks, one task reference, 594 redirects, and 220 indexes.

The five CLI hashes remain plan
`700f4bf503783742cefd8004fa095675b7476d46e9a3a6dd439916d237eb6718`,
parse `a8a7aa639d2ebc65eddc923c7e9369ea5637d50e935f808600f446da1bfbda56`,
declaration `71e83ba0b20d4015e07b3bd2c0c4db2837b6151d1251812caed7954530d53c74`,
type `4b2c7bd5ec3cc56e5672fb351126126230ec84fba9bd2bd9049a516d378fab7f`,
and proof `ccf3d2d4d0a3755e00989d97af369a7c560302f76798d0a185d57ec3891e8450`.

## Prerequisite And Expected Migration

The prerequisite changes exactly nine paths: this EN/JA pair, the historical
Task-258B5A pair, the source TSV, and one batch Task Index row in each
checker/test EN/JA plan. It expands the historical completion owner and
authorizes only these two TODO sections. Selected sources and ledger stay
unchanged; task-contract counts move from `59/59` to `60/60`.

After prerequisite commit and fresh replay, migration changes exactly five
paths: the EN/JA checker TODOs, this EN/JA pair, and
`legacy_compactions.tsv`. The two sections totaling 54 physical lines become
two language-local redirects to `258B5A.md#completion-evidence`, with exact
source diff `+2/-52`; all four neighboring H2 anchors remain byte-identical.

Ledger impact is eight lines, `872 -> 880`: one batch, four batch indexes, two
redirects over two source paths, and one `task_ref`; no second task row or
historical index is added. The canonical seven-row expanded-inventory SHA-256
is `93c964b12ac36314e1731317a081eb2c08077a5ec35e69cf30776ee0a55e2daf`;
expected physical ledger SHA-256 is
`ecaba8321e82f662b436460d1e41cb936c6284b7503621863a3f59e903113026`.
Final cardinalities are 23 batches, 33 tasks, two task references, 596
redirects, and 224 indexes. `doc/design/spec_coverage_audit.md` has no impact
because no mapping, ownership, status, deferred reason, or coverage credit
changes.

## Reviews, Verification, And Exit

Prerequisite and migration separately require evidence-equivalence,
schema/test-sufficiency, bilingual/boundary, and final-quality review as
applicable, all ending **NO FINDINGS**. All nine hard gates must PASS without a
score cap at no less than `90/100`.

Verification includes preimage/blame/anchor replay; generic recursive contract,
link, fragment, and ledger lint; checker/runner libraries and metadata;
formatting; offline metadata; warnings-denied all-target/all-feature Clippy;
full workspace tests; all five CLIs; protected count/hash; ledger order/hash/
cardinality; `git diff --check`; exact cached review; and unstaged/untracked
inspection. No push, fetch, reset, or stash mutation.

Prerequisite exits with exact nine-path scope, unchanged sources/ledger,
synchronized EN/JA, complete reviews/verification, one commit, and clean
replay. Migration exits separately with exact two redirects/five paths,
schema-2 real-reference replay, all gates, one commit, and clean replay before
the next checker duplication-family inventory.

## Documentation-Prerequisite Evidence

Independent evidence-equivalence, schema/test-sufficiency, and bilingual/
boundary reviews all end **NO FINDINGS** after one JA `boundary_violation` was
fixed by removing a stale phrase from the historical migration boundary. They
reproduce the `28/26`-line preimages, 54-line total, source TSV hashes, blame
split, four batch indexes, prospective seven-row canonical hash, 880-line
physical ledger hash, `+2/-52` migration delta, schema-2 ownership, and
disjoint source sets. Every removed-checklist fact is present in the historical
owner or its linked durable owner, and all language-local links and fragments
resolve.

Generic lint passes `15/15`; warnings-denied all-target/all-feature Clippy, the
full workspace suite, formatting, offline Cargo metadata, and
`git diff --check` pass. All five CLIs exit zero and reproduce their frozen
stdout hashes; the 23 known warnings and zero errors remain unchanged.

The immutable source TSV is four lines with its frozen complete/data hashes,
and task-contract counts are `60/60`. Selected TODO sections and the 872-line
ledger remain unchanged; the ledger, trace, and coverage-audit hashes reproduce
their frozen values. Protected counts and NUL-delimited path hashes reproduce
as specification 64, `.miz` 343, expectation 435, checker production 30,
runner production 90, and Cargo 21; zero protected diff preserves every frozen
content hash.

During review, externally pushed `origin/main` advanced from the selection
relation `0/3` to the same `71edf340` HEAD (`0/0`). No agent fetch, push, reset,
or stash action occurred; the event remains report-only and the exact nine-path
commit target is unambiguous. Final independent read-only quality review ends
**NO FINDINGS**, passes all nine hard gates, applies no score cap, and assigns
**100/100** (`20/20/15/15/10/10/5/5`). Exact staging, commit, and clean replay
then completed as recorded below.

## Migration Evidence

The documentation prerequisite committed separately as
`b213e68bae54a8b2a5c7415195e89a398761558b`. Clean fresh replay reproduced the
two frozen preimages, source TSV hashes, unchanged 872-line ledger, protected
surfaces, trace, coverage audit, origin relation `0/1`, and stash fingerprint
before migration.

The two selected TODO sections are now language-local redirects to
`258B5A.md#completion-evidence`. The exact Git source diff is `+2/-52`; each
selected heading and body is gone, while all four neighboring H2 anchors and
every retained TODO section remain byte-identical.

The ledger adds exactly eight byte-sorted lines: one batch, four batch indexes,
two redirects, and one `task_ref`. It is 880 lines with physical SHA-256
`ecaba8321e82f662b436460d1e41cb936c6284b7503621863a3f59e903113026`,
reproduces canonical seven-row SHA-256
`93c964b12ac36314e1731317a081eb2c08077a5ec35e69cf30776ee0a55e2daf`,
and measures 23 batches, 33 tasks, two task references, 596 redirects, and 224
indexes. There is no second task row or historical Task Index. The historical
contract, source TSV, four plans, protected surfaces, trace, and coverage audit
remain unchanged.

Independent migration-equivalence, schema/test-sufficiency, and bilingual/
boundary reviews all end **NO FINDINGS**. Generic lint passes `15/15` against
the second real schema-2 reference. Warnings-denied all-target/all-feature
Clippy, the full workspace suite, formatting, offline Cargo metadata, and
`git diff --check` pass. All five CLIs exit zero and reproduce their frozen
stdout hashes; the 23 known warnings and zero errors are unchanged.

Protected counts and NUL-delimited path hashes reproduce as specification 64,
`.miz` 343, expectation 435, checker production 30, runner production 90, and
Cargo 21. Zero protected diff preserves every frozen content hash. Trace,
coverage audit, immutable source TSV, seven-row canonical payload, and 880-line
ledger reproduce their frozen hashes. Final independent read-only quality
review ends **NO FINDINGS**, passes all nine hard gates, applies no score cap,
and assigns **100/100** (`20/20/15/15/10/10/5/5`). Exact five-path staging,
commit, and clean replay remain.
