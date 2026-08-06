# Task DOC-258B4C-IMPLEMENTATION-LEDGER-COMPACT: B4C Implementation-Ledger Compaction

> Canonical language: English. Japanese companion:
> [../ja/DOC-258B4C-IMPLEMENTATION-LEDGER-COMPACT.md](../ja/DOC-258B4C-IMPLEMENTATION-LEDGER-COMPACT.md).

## Identity And Status

| Field | Frozen value |
|---|---|
| Task | `DOC-258B4C-IMPLEMENTATION-LEDGER-COMPACT` |
| Status | Documentation prerequisite complete; selected sources and schema-2 ledger remain unchanged pending exact staging and this dedicated commit. |
| Purpose | Centralize the EN/JA checker TODO implementation-completion checklists for historical Task 258B4C without changing any durable checker or runner owner. |
| Historical owner | [Task 258B4C](./258B4C.md#completion-evidence) |
| Plan indexes | [checker plan](../../mizar-checker/en/00.crate_plan.md#task-index) and [runner plan](../../mizar-test/en/00.crate_plan.md#task-index) |
| Selection HEAD | `eb9286332d1e3800d46a63cb6318275e6fdda014` |
| Repository state | clean `main`, `origin/main...HEAD=0/1`, protected `stash@{0}=f65cf4a13752ec380710814a9ac6392ccb9d75d4` |
| Dependency | Schema-2 `task_ref` support committed in `eb9286332d1e3800d46a63cb6318275e6fdda014`; prior review batch `DOC-258B4C-DOC-REVIEW-COMPACT` remains immutable. |

## Authority And Classification

Authority is the user-approved checker-first compaction program,
[`AGENTS.md`](../../../../AGENTS.md), the
[autonomous migration policy](../../autonomous_crate_development.md#migration-policy),
reviewed Git history, the two selected completed sections, and their surviving
durable owners. Source behavior is not normative.

| Class | Decision |
|---|---|
| `design_drift` | Checker EN/JA TODOs repeat the same historical B4C upper-implementation scope, provenance/counts, reviews, gates, commit, and handoff evidence outside their central historical owner. |
| `test_gap` | None. Schema-2 positive and fail-closed `task_ref` vectors are committed; the existing generic lint will replay this first real reference in the separate migration, where exact real-data replay remains an exit criterion. |
| `boundary_violation` | Avoided by selecting only two flat TODO sections. Plan implementation inventory is a registered neighboring anchor; checker owner/audit and every runner section retain durable local facts and remain unchanged. |
| `spec_gap` / `source_drift` | None introduced or repaired; language semantics and completed implementation remain unchanged. |
| `source_undocumented_behavior` / `test_expectation_drift` | None inferred or changed. |
| `repo_metadata_conflict` | None at selection. The external origin update during the schema task is already report-only; current safe commit target is unambiguous. |

## Frozen Sources And Anchors

[`DOC-258B4C-IMPLEMENTATION-LEDGER-COMPACT.sources.tsv`](../DOC-258B4C-IMPLEMENTATION-LEDGER-COMPACT.sources.tsv)
contains two byte-sorted data rows, two comments, and final LF. Data-row
SHA-256 is `d36ef5ec920b3b0ccbfae3271ca552c8e20964d50f75c72ced9656382bb46c16`;
complete-file SHA-256 is
`9a71e89fee1e7f058156ceb9521d9dd944c10f5a019f5b6996da2dd7f7e3bd5d`.
The flat, source-locally unique, unlinked sections contain no nested headings,
tables, fences, or redirects and total 49 physical lines.

| Source | Lines | Section SHA-256 | Previous H2 | Next H2 |
|---|---:|---|---|---|
| EN checker TODO `5934-5958` | 25 | `b3232c301dc8df4b6da3cccb4d040c9a819b8931ed31d20e311ca574f86ba82e` | `## Checker Task 258B4C Lower-Stage Prerequisite Ledger` | `## Checker Task 258B5A Frozen-Contract Documentation Prerequisite` |
| JA checker TODO `5670-5693` | 24 | `200dcfb5ecd4e44ea25254d70c049338a211009d28c89cc05c147541e727417f` | `## Checker Task 258B4C lower-stage prerequisite ledger` | `## Checker Task 258B5A frozen-contract documentation prerequisite` |

Implementation commit `50ab1ebc747e912fff1f0cf111832e3c2c81ba01`
introduced both checklist bodies. Successor prerequisite
`59021f764f146d669f84877042f0512882c9c5ff` added only the exact commit and
post-commit handoff tails. Current blame attributes every selected line to
those two commits.

The prior Task-258B4C review batch uses eight other checker source paths. Its
four existing plan/bilingual/boundary/source-audit paths and this batch's two
TODO paths are disjoint, satisfying schema 2. The existing canonical `task`
row and four historical Task Index records stay owned by
`DOC-258B4C-DOC-REVIEW-COMPACT`; this batch adds one `task_ref` and indexes
only its own batch contract.

## Retained Owners And Exclusions

The checker plan—including its registered `## Task 258B4C Implementation
Inventory` anchor—plus statement, formula-composition, payload-family,
Typed/Resolved AST, boundary, bilingual, and source/specification documents
remain unchanged. The distinct lower-stage prerequisite ledger and every
`mizar-test` plan, TODO, harness, boundary, bilingual, metadata, and runner
section remain unchanged.

Specifications, `.miz`, expectations, sidecars, traceability, coverage audit,
production, Cargo, public APIs, diagnostics, and active behavior are forbidden.
No equality/quantifier truth, witness/restriction discharge, fact, theorem
acceptance/publication, proof, Core/CFG/VC, B5, or broader visibility meaning is
inferred or changed.

## Protected Baseline

Specification `64`, `.miz` `343`, expectation `435`, checker production `30`,
runner production `90`, and Cargo `21` path sets retain the hashes frozen by
the immediately preceding B4A/B4B batches. Trace SHA-256 remains
`55b754c8c4d0d293a1c44e2ba4b0090f407bba1d429b461b6cb4d6ddca9ca2b3`;
coverage audit remains
`2aa808aa033c04fa46ea76f65cfadcf0d8f0e1e53fba6d70edab223a7481685f`.
The schema-2 ledger baseline is 864 lines with physical SHA-256
`b7e9a943afcca7ee4773e6ac472e8a350624d17f96dbb54ca821fcb1f57d56cc`,
21 batches, 33 tasks, zero task references, 592 redirects, and 216 indexes.

The five CLI hashes remain plan
`700f4bf503783742cefd8004fa095675b7476d46e9a3a6dd439916d237eb6718`,
parse `a8a7aa639d2ebc65eddc923c7e9369ea5637d50e935f808600f446da1bfbda56`,
declaration `71e83ba0b20d4015e07b3bd2c0c4db2837b6151d1251812caed7954530d53c74`,
type `4b2c7bd5ec3cc56e5672fb351126126230ec84fba9bd2bd9049a516d378fab7f`,
and proof `ccf3d2d4d0a3755e00989d97af369a7c560302f76798d0a185d57ec3891e8450`.

## Prerequisite And Expected Migration

The prerequisite changes exactly nine paths: this EN/JA pair, the historical
Task-258B4C pair, the source TSV, and one batch Task Index row in each
checker/test EN/JA plan. It expands the historical completion owner and
authorizes only these two TODO sections. Selected sources and the ledger stay
unchanged; task-contract counts move from 58/58 to 59/59.

After prerequisite commit and fresh replay, migration changes exactly five
paths: the EN/JA checker TODOs, this EN/JA batch pair, and
`legacy_compactions.tsv`. The two 49-line sections become two language-local
redirects to `258B4C.md#completion-evidence`, with exact source diff
`+2/-47`; all four neighboring H2 anchors remain byte-identical.

Ledger impact is eight lines, `864 -> 872`: one batch, four batch indexes, two
redirects over two source paths, and one `task_ref`; no second task row or
historical index is added. The canonical seven-row expanded-inventory SHA-256
is `952749b6af84fab726964089b40cc0812629e117e2f06ba36b3efbb9cdc363c6`;
expected physical ledger SHA-256 is
`5ac307e25074e8a776024a0a060fab9d45ca68a631ca39a40283f14bfe6d485b`.
Final cardinalities are 22 batches, 33 tasks, one task reference, 594 redirects,
and 220 indexes. `doc/design/spec_coverage_audit.md` has no impact because no
mapping, ownership, status, deferred reason, or coverage credit changes.

## Reviews, Verification, And Exit

Prerequisite and migration separately require evidence-equivalence,
schema/test-sufficiency, bilingual/boundary, and final-quality review as
applicable, all ending **NO FINDINGS**. All nine hard gates must PASS without a
score cap at no less than `90/100`.

Verification includes preimage/blame/anchor replay; generic recursive contract,
link, fragment, and ledger lint; checker/runner lint and libraries; runner
metadata; formatting; offline metadata; warnings-denied all-target/all-feature
Clippy; full workspace tests; all five CLIs; protected count/hash; ledger
order/hash/cardinality; `git diff --check`; exact cached review; and
unstaged/untracked inspection. No push, fetch, reset, or stash mutation.

Prerequisite exits with exact nine-path scope, unchanged sources/ledger,
synchronized EN/JA, complete reviews/verification, one commit, and clean
replay. Migration exits separately with exact two redirects/five paths,
schema-2 real-reference replay, all gates, one commit, and clean replay before
the next checker duplication-family inventory.

## Documentation-Prerequisite Evidence

Independent evidence-equivalence, schema/test-sufficiency, and bilingual/
boundary reviews all end **NO FINDINGS** after two review findings were fixed
in both languages: the central historical owner now preserves the exact B3/B4C
pairing, and the expected physical ledger hash is the independently
reconstructed value above. The reviews reproduce the `25/24`-line preimages,
49-line total, source TSV hashes, four batch indexes, prospective seven-row
canonical hash, 872-line physical ledger hash, `+2/-47` migration delta,
schema-2 reference ownership, and disjoint source sets. All retained owner
links and language-local fragments resolve.

Generic lint passes `15/15`; the full workspace suite passes, including
checker library `530/530`, runner library `600/600`, and runner metadata
`137/137`. Formatting, offline Cargo metadata, warnings-denied all-target/all-
feature Clippy, and `git diff --check` pass. All five CLIs exit zero with 23
known warnings and zero errors each and reproduce their five frozen stdout
hashes.

The immutable source TSV is four lines with its frozen complete/data hashes,
and task-contract counts are `59/59`. The selected TODO sections and 864-line
ledger remain unchanged; the ledger, trace, and coverage-audit hashes reproduce
their frozen values. Zero protected diff preserves the specification, `.miz`,
expectation, checker production, runner production, and Cargo path sets and
their frozen counts and hashes. Final independent read-only quality review ends
**NO FINDINGS**, passes all nine hard gates, applies no score cap, and assigns
**100/100** (`20/20/15/15/10/10/5/5`). Exact nine-path staging, cached review,
commit, and clean replay remain.
